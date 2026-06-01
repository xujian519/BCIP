use codex_patent_core::KgEdge;
use codex_patent_core::KgNode;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use rusqlite::params;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug)]
pub struct KgStats {
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug)]
pub struct NodeTypeCount {
    pub node_type: String,
    pub count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IpcSearchResult {
    pub code: String,
    pub description: String,
    pub level: i32,
    pub parent_code: Option<String>,
}

pub struct SqliteKnowledgeGraph {
    conn: Connection,
    query_cache: Mutex<HashMap<String, Vec<KgNode>>>,
}

impl SqliteKnowledgeGraph {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("failed to open kg db: {e}"))?;

        conn.execute_batch(
            "PRAGMA cache_size = -8000;
             PRAGMA locking_mode = NORMAL;",
        )
        .map_err(|e| format!("pragma setup: {e}"))?;

        Ok(Self {
            conn,
            query_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn from_connection(conn: Connection) -> Self {
        Self {
            conn,
            query_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn stats(&self) -> Result<KgStats, String> {
        let node_count = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get::<_, usize>(0))
            .map_err(|e| format!("{e}"))?;
        let edge_count = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |r| r.get::<_, usize>(0))
            .map_err(|e| format!("{e}"))?;
        Ok(KgStats {
            node_count,
            edge_count,
        })
    }

    pub fn search_nodes(
        &self,
        query: &str,
        node_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<KgNode>, String> {
        let cache_key = format!("{}|{:?}|{}", query, node_type, limit);

        {
            let cache = self.query_cache.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key)
                && cached.len() >= limit
            {
                return Ok(cached[..limit].to_vec());
            }
        }

        let fts_query = format!("\"{}\"", query.replace('"', "\"\""));
        let nodes = if let Some(nt) = node_type {
            let sql = "SELECT n.id, n.node_type, n.name, n.title, n.content, n.law_refs_count, n.source, n.full_ref, n.chapter, n.article_number \
                      FROM nodes_fts f \
                      JOIN nodes n ON n.rowid = f.rowid \
                      WHERE nodes_fts MATCH ?1 AND n.node_type = ?2 \
                      ORDER BY f.rank \
                      LIMIT ?3";
            self.query_nodes(sql, params![fts_query, nt, limit])?
        } else {
            let sql = "SELECT n.id, n.node_type, n.name, n.title, n.content, n.law_refs_count, n.source, n.full_ref, n.chapter, n.article_number \
                      FROM nodes_fts f \
                      JOIN nodes n ON n.rowid = f.rowid \
                      WHERE nodes_fts MATCH ?1 \
                      ORDER BY f.rank \
                      LIMIT ?2";
            self.query_nodes(sql, params![fts_query, limit])?
        };

        {
            let mut cache = self.query_cache.lock().unwrap();
            if cache.len() > 100 {
                cache.clear();
            }
            cache.insert(cache_key, nodes.clone());
        }

        Ok(nodes)
    }

    pub fn clear_cache(&self) {
        self.query_cache.lock().unwrap().clear();
    }

    pub fn get_edges(&self, node_id: &str) -> Result<Vec<KgEdge>, String> {
        let sql = "SELECT id, source, target, relation FROM edges WHERE source = ?1 OR target = ?1";
        let mut stmt = self.conn.prepare(sql).map_err(|e| format!("{e}"))?;

        let rows = stmt
            .query_map(params![node_id], |row| {
                Ok(KgEdge {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    relation: row.get(3)?,
                })
            })
            .map_err(|e| format!("{e}"))?;

        let edges: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(edges)
    }

    pub fn get_nodes_by_type(&self, node_type: &str, limit: usize) -> Result<Vec<KgNode>, String> {
        let sql = "SELECT id, node_type, name, title, content, law_refs_count, source, full_ref, chapter, article_number \
                   FROM nodes WHERE node_type = ?1 LIMIT ?2";
        let mut stmt = self.conn.prepare(sql).map_err(|e| format!("{e}"))?;

        let rows = stmt
            .query_map(params![node_type, limit], |row| {
                Ok(KgNode {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    law_refs_count: row.get(5)?,
                    source: row.get(6)?,
                    full_ref: row.get(7)?,
                    chapter: row.get(8)?,
                    article_number: row.get(9)?,
                })
            })
            .map_err(|e| format!("{e}"))?;

        let nodes: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(nodes)
    }

    /// 多跳 BFS 遍历：从 node_id 出发，按 relation_type 过滤，最大 depth 跳
    pub fn traverse(
        &self,
        start_id: &str,
        relation_filter: Option<&[&str]>,
        max_depth: usize,
    ) -> Result<Vec<(KgEdge, usize)>, String> {
        let mut visited = HashSet::new();
        let mut results = Vec::new();
        let mut current = vec![start_id.to_string()];
        let mut depth = 0;

        while depth < max_depth && !current.is_empty() {
            depth += 1;
            let mut next = Vec::new();
            for node_id in &current {
                if !visited.insert(node_id.clone()) {
                    continue;
                }
                let edges = self.get_edges(node_id)?;
                for edge in edges {
                    if let Some(filter) = relation_filter
                        && !filter.contains(&edge.relation.as_str())
                    {
                        continue;
                    }
                    results.push((edge.clone(), depth));
                    let neighbor = if edge.source == *node_id {
                        edge.target.clone()
                    } else {
                        edge.source.clone()
                    };
                    if !visited.contains(&neighbor) {
                        next.push(neighbor);
                    }
                }
            }
            current = next;
        }

        Ok(results)
    }

    /// 查找两个节点之间的路径（BFS）
    pub fn find_path(
        &self,
        from: &str,
        to: &str,
        max_depth: usize,
    ) -> Result<Vec<Vec<KgEdge>>, String> {
        if from == to {
            return Ok(vec![Vec::new()]);
        }

        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![(from.to_string(), Vec::new())];

        let mut depth = 0;
        while depth < max_depth && !queue.is_empty() && paths.is_empty() {
            depth += 1;
            let mut next_queue = Vec::new();
            for (node_id, path) in &queue {
                if !visited.insert(node_id.clone()) {
                    continue;
                }
                let edges = self.get_edges(node_id)?;
                for edge in &edges {
                    let neighbor = if edge.source == *node_id {
                        &edge.target
                    } else {
                        &edge.source
                    };
                    let mut new_path = path.clone();
                    new_path.push(edge.clone());
                    if neighbor == to {
                        paths.push(new_path);
                    } else {
                        next_queue.push((neighbor.clone(), new_path));
                    }
                }
            }
            queue = next_queue;
        }

        Ok(paths)
    }

    /// 搜索 IPC 分类（通过 FTS5 索引）
    pub fn search_ipc(&self, query: &str, limit: usize) -> Result<Vec<IpcSearchResult>, String> {
        let fts_query = format!("\"{}\"", query.replace('"', "\"\""));
        let sql = "SELECT code, description, level, parent_code FROM ipc_fts WHERE ipc_fts MATCH ? ORDER BY rank LIMIT ?";
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| format!("search_ipc prepare failed (query={:?}): {e}", fts_query))?;
        let rows = stmt
            .query_map(params![fts_query, limit], |row| {
                Ok(IpcSearchResult {
                    code: row.get(0)?,
                    description: row.get(1)?,
                    level: row.get(2)?,
                    parent_code: row.get(3)?,
                })
            })
            .map_err(|e| format!("search_ipc query failed: {e}"))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 三角查询：通过 IPC/Concept/Clause 中的任意组合查找关联节点
    pub fn search_by_triangle(
        &self,
        ipc: Option<&str>,
        concept: Option<&str>,
        clause: Option<&str>,
        limit: usize,
    ) -> Result<Vec<KgNode>, String> {
        let mut node_ids = HashSet::new();

        // IPC → CLASSIFIED_AS → Decision
        if let Some(ipc_code) = ipc {
            let ipc_id = format!("IPC_{}", ipc_code);
            let edges = self.get_edges(&ipc_id)?;
            for e in &edges {
                if e.relation == "CLASSIFIED_AS" {
                    node_ids.insert(e.source.clone());
                    node_ids.insert(e.target.clone());
                }
            }
        }

        // Concept → INVOLVES/DECIDES → Decision/Judgment
        if let Some(concept_name) = concept {
            let concept_nodes = self.search_nodes(concept_name, Some("Concept"), 10)?;
            for cn in &concept_nodes {
                let edges = self.get_edges(&cn.id)?;
                for e in &edges {
                    if matches!(e.relation.as_str(), "INVOLVES" | "DECIDES" | "REFERENCES") {
                        node_ids.insert(e.source.clone());
                        node_ids.insert(e.target.clone());
                    }
                }
            }
        }

        // Clause → APPLIES/CITES → Decision/Judgment
        if let Some(clause_id) = clause {
            let edges = self.get_edges(clause_id)?;
            for e in &edges {
                if matches!(e.relation.as_str(), "APPLIES" | "CITES") {
                    node_ids.insert(e.source.clone());
                    node_ids.insert(e.target.clone());
                }
            }
        }

        // 批量获取节点详情（替代逐条 get_node_by_id）
        if node_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<&String> = node_ids.iter().take(limit).collect();
        self.get_nodes_by_ids(&ids)
    }

    /// 根据 ID 列表批量获取节点
    fn get_nodes_by_ids(&self, ids: &[&String]) -> Result<Vec<KgNode>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect();
        let sql = format!(
            "SELECT id, node_type, name, title, content, law_refs_count, source, full_ref, chapter, article_number \
             FROM nodes WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| format!("{e}"))?;
        let params: Vec<&String> = ids.iter().map(|s| *s).collect();
        let rows = stmt
            .query_map(
                rusqlite::params_from_iter(params.iter().map(|s| *s as &dyn rusqlite::ToSql)),
                |row| {
                    Ok(KgNode {
                        id: row.get(0)?,
                        node_type: row.get(1)?,
                        name: row.get(2)?,
                        title: row.get(3)?,
                        content: row.get(4)?,
                        law_refs_count: row.get(5)?,
                        source: row.get(6)?,
                        full_ref: row.get(7)?,
                        chapter: row.get(8)?,
                        article_number: row.get(9)?,
                    })
                },
            )
            .map_err(|e| format!("get_nodes_by_ids query failed: {e}"))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 根据 ID 获取单个节点
    pub fn get_node_by_id(&self, id: &str) -> Result<KgNode, String> {
        let sql = "SELECT id, node_type, name, title, content, law_refs_count, source, full_ref, chapter, article_number \
                   FROM nodes WHERE id = ?";
        self.conn
            .query_row(sql, params![id], |row| {
                Ok(KgNode {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    law_refs_count: row.get(5)?,
                    source: row.get(6)?,
                    full_ref: row.get(7)?,
                    chapter: row.get(8)?,
                    article_number: row.get(9)?,
                })
            })
            .map_err(|e| format!("{e}"))
    }

    pub fn node_type_distribution(&self) -> Result<Vec<NodeTypeCount>, String> {
        let sql =
            "SELECT node_type, COUNT(*) as cnt FROM nodes GROUP BY node_type ORDER BY cnt DESC";
        let mut stmt = self.conn.prepare(sql).map_err(|e| format!("{e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(NodeTypeCount {
                    node_type: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| format!("{e}"))?;

        let result: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(result)
    }

    fn query_nodes<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<KgNode>, String> {
        let mut stmt = self.conn.prepare(sql).map_err(|e| format!("{e}"))?;

        let rows = stmt
            .query_map(params, |row| {
                Ok(KgNode {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    law_refs_count: row.get(5)?,
                    source: row.get(6)?,
                    full_ref: row.get(7)?,
                    chapter: row.get(8)?,
                    article_number: row.get(9)?,
                })
            })
            .map_err(|e| format!("{e}"))?;

        let mut nodes = Vec::new();
        for node in rows.flatten() {
            nodes.push(node);
        }
        Ok(nodes)
    }
}
