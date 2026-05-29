use codex_patent_core::KgEdge;
use codex_patent_core::KgNode;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use rusqlite::params;
use std::collections::HashMap;
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
            if let Some(cached) = cache.get(&cache_key) {
                if cached.len() >= limit {
                    return Ok(cached[..limit].to_vec());
                }
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
        for row in rows {
            if let Ok(node) = row {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }
}
