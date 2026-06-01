use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;

use crate::models::*;

pub struct KgDatabase {
    conn: Connection,
}

impl KgDatabase {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("打开数据库失败: {}", path.display()))?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))?;
        Ok(KgDatabase { conn })
    }

    pub fn begin_transaction(&mut self) -> Result<rusqlite::Transaction<'_>> {
        self.conn.transaction().context("开始事务失败")
    }

    // ── IPC 表操作 ──

    pub fn purge_old_ipc_versions(conn: &Connection) -> Result<usize> {
        let count: usize = conn.query_row(
            "SELECT COUNT(*) FROM ipc_classification WHERE version != '2026.01'",
            [],
            |row| row.get(0),
        )?;
        conn.execute(
            "DELETE FROM ipc_classification WHERE version != '2026.01'",
            [],
        )?;
        println!("      删除旧版本 IPC: {} 条", count);
        Ok(count)
    }

    pub fn insert_ipc_entries(conn: &Connection, entries: &[IpcEntry]) -> Result<usize> {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO ipc_classification \
             (code, section, class, subclass, group_code, level, parent_code, description, version, source_file) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;
        let mut count = 0;
        for entry in entries {
            stmt.execute(rusqlite::params![
                &entry.code,
                &entry.section,
                &entry.class,
                &entry.subclass,
                &entry.group_code,
                entry.level,
                &entry.parent_code,
                &entry.description,
                &entry.version,
                &entry.source_file,
            ])?;
            count += 1;
        }
        Ok(count)
    }

    pub fn insert_ipc_nodes(conn: &Connection, entries: &[IpcEntry]) -> Result<usize> {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO nodes \
             (id, node_type, name, title, content, law_refs_count, source, full_ref, chapter, article_number, version) \
             VALUES (?, 'IPC', ?, ?, ?, 0, ?, '', '', '', ?)",
        )?;
        let mut count = 0;
        for entry in entries {
            if entry.level != -1 {
                continue;
            }
            let id = format!("IPC_{}", entry.code);
            stmt.execute(rusqlite::params![
                &id,
                &entry.code,
                &entry.description,
                &entry.description,
                &entry.source_file,
                &entry.version,
            ])?;
            count += 1;
        }
        println!("      插入 {} 个 IPC 小类节点", count);
        Ok(count)
    }

    pub fn insert_ipc_edges(conn: &Connection, entries: &[IpcEntry]) -> Result<usize> {
        let mut stmt = conn.prepare(
            "INSERT OR IGNORE INTO edges (source, target, relation) VALUES (?, ?, 'SUBCLASS_OF')",
        )?;
        let mut count = 0;
        for entry in entries {
            if entry.parent_code.is_none() || entry.level < -1 {
                continue;
            }
            let source_id = format!("IPC_{}", entry.code);
            let parent = entry.parent_code.as_ref().unwrap();
            let target_id = format!("IPC_{}", parent);
            stmt.execute([&source_id, &target_id])?;
            count += 1;
        }
        println!("      插入 {} 条 IPC 层级边", count);
        Ok(count)
    }

    // ── InvalidDecision 节点操作 ──

    pub fn insert_decision_nodes(
        conn: &Connection,
        decisions: &[InvalidDecision],
    ) -> Result<usize> {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO nodes \
             (id, node_type, name, title, content, law_refs_count, source, full_ref, chapter, article_number, version) \
             VALUES (?, 'InvalidDecision', ?, ?, ?, ?, ?, ?, ?, ?, '')",
        )?;
        let mut count = 0;
        for d in decisions {
            if d.decision_number.is_empty() {
                continue;
            }
            let id = format!("ID_{}", d.decision_number);
            let title = format!("{}（第{}号）", d.conclusion, d.decision_number);
            let law_refs = d.law_articles.join(",");
            let reasons = d.reasons.join(",");

            stmt.execute(rusqlite::params![
                &id,
                &d.decision_number,
                &title,
                &d.summary,
                d.law_articles.len() as i64,
                &d.source_file,
                &d.patent_number,
                &reasons,
                &law_refs,
            ])?;
            count += 1;
        }
        println!("      插入 {} 个 InvalidDecision 节点", count);
        Ok(count)
    }

    pub fn insert_decision_clause_edges(
        conn: &Connection,
        decisions: &[InvalidDecision],
    ) -> Result<usize> {
        let mut edge_stmt = conn.prepare(
            "INSERT OR IGNORE INTO edges (source, target, relation) VALUES (?, ?, 'APPLIES')",
        )?;
        let mut count = 0;
        for d in decisions {
            if d.decision_number.is_empty() {
                continue;
            }
            let source_id = format!("ID_{}", d.decision_number);
            for article in &d.law_articles {
                if let Some(target_id) = find_clause_node(conn, article) {
                    edge_stmt.execute([&source_id, &target_id])?;
                    count += 1;
                }
            }
        }
        println!("      插入 {} 条 Decision→Clause 边", count);
        Ok(count)
    }

    pub fn insert_decision_ipc_edges(
        conn: &Connection,
        decisions: &[InvalidDecision],
    ) -> Result<usize> {
        let mut exists_stmt = conn.prepare("SELECT 1 FROM nodes WHERE id = ? LIMIT 1")?;
        let mut edge_stmt = conn.prepare(
            "INSERT OR IGNORE INTO edges (source, target, relation) VALUES (?, ?, 'CLASSIFIED_AS')",
        )?;
        let mut count = 0;
        for d in decisions {
            if d.decision_number.is_empty() || d.ipc_code.is_none() {
                continue;
            }
            let source_id = format!("ID_{}", d.decision_number);
            let ipc = d.ipc_code.as_ref().unwrap();
            let subclass = extract_ipc_subclass(ipc);
            if subclass.is_empty() {
                continue;
            }
            let target_id = format!("IPC_{}", subclass);
            let exists: bool = exists_stmt
                .query_row([&target_id], |_| Ok(true))
                .unwrap_or(false);
            if exists {
                edge_stmt.execute([&source_id, &target_id])?;
                count += 1;
            }
        }
        println!("      插入 {} 条 Decision→IPC 边", count);
        Ok(count)
    }

    // ── Judgment 节点操作 ──

    pub fn insert_judgment_nodes(conn: &Connection, judgments: &[JudgmentEntry]) -> Result<usize> {
        let mut check_stmt = conn.prepare(
            "SELECT 1 FROM nodes WHERE name = ? AND node_type IN ('SupremeCourtJudgment', 'RegionalCourtJudgment') LIMIT 1",
        )?;
        let mut insert_stmt = conn.prepare(
            "INSERT OR IGNORE INTO nodes \
             (id, node_type, name, title, content, law_refs_count, source, full_ref, chapter, article_number, version) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '')",
        )?;
        let mut count = 0;
        for j in judgments {
            if j.case_number.is_empty() {
                continue;
            }
            let node_type = if j.is_guiding {
                "GuidingJudgment"
            } else {
                "PatentJudgment"
            };

            let exists: bool = check_stmt
                .query_row([&j.case_number], |_| Ok(true))
                .unwrap_or(false);
            if exists {
                continue;
            }

            let id = sanitize_judgment_id(&j.case_number);
            let keywords = j.keywords.join(",");
            let law_refs = j.law_articles.join(",");
            let content = if j.is_guiding && !j.key_points.is_empty() {
                format!("{}\n\n{}", j.key_points, j.summary)
            } else {
                j.summary.clone()
            };

            insert_stmt.execute(rusqlite::params![
                &id,
                node_type,
                &j.case_number,
                &j.case_number,
                &content,
                j.law_articles.len() as i64,
                &j.source_file,
                &j.court,
                &keywords,
                &law_refs,
            ])?;
            count += 1;
        }
        println!(
            "      插入 {} 个 {} 节点",
            count,
            if judgments.first().map(|j| j.is_guiding).unwrap_or(false) {
                "GuidingJudgment"
            } else {
                "PatentJudgment"
            }
        );
        Ok(count)
    }

    pub fn insert_judgment_clause_edges(
        conn: &Connection,
        judgments: &[JudgmentEntry],
    ) -> Result<usize> {
        let mut edge_stmt = conn.prepare(
            "INSERT OR IGNORE INTO edges (source, target, relation) VALUES (?, ?, 'CITES')",
        )?;
        let mut count = 0;
        for j in judgments {
            if j.case_number.is_empty() {
                continue;
            }
            let source_id = sanitize_judgment_id(&j.case_number);
            for article in &j.law_articles {
                if let Some(target_id) = find_clause_node(conn, article) {
                    edge_stmt.execute([&source_id, &target_id])?;
                    count += 1;
                }
            }
        }
        println!("      插入 {} 条 Judgment→Clause 边", count);
        Ok(count)
    }

    pub fn insert_judgment_concept_edges(
        conn: &Connection,
        judgments: &[JudgmentEntry],
    ) -> Result<usize> {
        let mut check_stmt = conn.prepare(
            "SELECT 1 FROM nodes WHERE id = ? AND node_type IN ('Concept', 'ConceptDetail') LIMIT 1",
        )?;
        let mut edge_stmt = conn.prepare(
            "INSERT OR IGNORE INTO edges (source, target, relation) VALUES (?, ?, 'DECIDES')",
        )?;
        let mut count = 0;
        for j in judgments {
            if !j.is_guiding || j.case_number.is_empty() {
                continue;
            }
            let source_id = sanitize_judgment_id(&j.case_number);
            for keyword in &j.keywords {
                let concept_id = sanitize_id(keyword);
                let exists: bool = check_stmt
                    .query_row([&concept_id], |_| Ok(true))
                    .unwrap_or(false);
                if exists {
                    edge_stmt.execute([&source_id, &concept_id])?;
                    count += 1;
                }
            }
        }
        println!("      插入 {} 条 Judgment→Concept 边", count);
        Ok(count)
    }

    // ── 统计 ──

    pub fn count_edges_by_relation(&self, relation: &str) -> usize {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE relation = ?",
                [relation],
                |row| row.get(0),
            )
            .unwrap_or(0)
    }

    pub fn count_ipc_decision_edges(&self) -> usize {
        self.conn
            .query_row(
                "SELECT COUNT(DISTINCT e.source) FROM edges e
                 JOIN nodes n ON e.target = n.id
                 WHERE n.node_type = 'IPC' AND e.relation = 'CLASSIFIED_AS'",
                [],
                |row: &rusqlite::Row| -> rusqlite::Result<usize> { row.get(0) },
            )
            .unwrap_or(0)
    }

    pub fn count_nodes_by_type(&self, node_type: &str) -> usize {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type = ?",
                [node_type],
                |row| row.get(0),
            )
            .unwrap_or(0)
    }

    pub fn get_stats(&self) -> Result<(usize, usize, HashMap<String, usize>)> {
        let total_nodes: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))?;
        let total_edges: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))?;
        let mut stmt = self
            .conn
            .prepare("SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type")?;
        let type_counts: HashMap<String, usize> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok((total_nodes, total_edges, type_counts))
    }
}

fn find_clause_node(conn: &Connection, article: &str) -> Option<String> {
    let re = regex::Regex::new(r"^(?:A|R)(\d+)").ok()?;
    let caps = re.captures(article)?;
    let article_num = &caps[1];
    let clause_id = format!("A{}", article_num);
    let result: String = conn
        .query_row(
            "SELECT id FROM nodes WHERE node_type = 'Clause' AND id = ? LIMIT 1",
            [&clause_id],
            |row| row.get(0),
        )
        .ok()?;
    Some(result)
}

fn extract_ipc_subclass(ipc: &str) -> String {
    let cleaned = ipc.replace(' ', "");
    regex::Regex::new(r"^([A-H]\d{2}[A-Z])")
        .unwrap()
        .captures(&cleaned)
        .map(|c| c[1].to_string())
        .unwrap_or_default()
}

pub fn sanitize_id(name: &str) -> String {
    regex::Regex::new(r"_+")
        .unwrap()
        .replace_all(
            &name
                .replace(' ', "_")
                .replace('/', "_")
                .replace('\\', "_")
                .replace('(', "_")
                .replace(')', "_")
                .replace('（', "_")
                .replace('）', "_")
                .replace('、', "_")
                .replace('，', "_")
                .replace('；', "_")
                .replace('：', "_")
                .replace('？', "_")
                .replace('！', "_"),
            "_",
        )
        .to_string()
}

fn sanitize_judgment_id(case_number: &str) -> String {
    format!(
        "J_{}",
        case_number
            .replace('（', "(")
            .replace('）', ")")
            .replace(' ', "_")
            .replace('/', "_")
    )
}
