//! 知识图谱种子数据 — 以纯代码生成最小可用 KG 用于测试和开发。
//!
//! 解决 `legal_reasoning.rs` 测试依赖外部 `patent_kg.db` 的问题:
//! 提供一个零依赖的 `in_memory_kg()` 函数,包含专利法律推理所需的核心节点和边。

use rusqlite::{Connection, params};

use codex_patent_knowledge::SqliteKnowledgeGraph;

/// 创建包含种子数据的内存知识图谱
///
/// 包含以下节点类型:
/// - concept: 法律概念(新颖性、创造性、侵权、无效、审查等)
/// - rule: 法律规则(三步法、区别特征、技术效果等)
/// - article: 法条引用
/// - case: 典型案例
///
/// 包含以下边关系:
/// - defined_by: 概念由规则/法条定义
/// - related_to: 概念间关联
/// - applied_by: 规则的应用方式
pub fn in_memory_kg() -> Result<SqliteKnowledgeGraph, String> {
    let conn = Connection::open_in_memory().map_err(|e| format!("in-memory: {e}"))?;

    init_schema(&conn)?;
    seed_nodes(&conn)?;
    seed_edges(&conn)?;

    Ok(SqliteKnowledgeGraph::from_connection(conn))
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL,
            name TEXT NOT NULL,
            title TEXT NOT NULL DEFAULT '',
            content TEXT,
            law_refs_count INTEGER,
            source TEXT,
            full_ref TEXT,
            chapter TEXT,
            article_number TEXT
        );
        CREATE TABLE IF NOT EXISTS edges (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            target TEXT NOT NULL,
            relation TEXT NOT NULL
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
            id, name, title, content,
            content='nodes', content_rowid='rowid'
        );",
    )
    .map_err(|e| format!("schema: {e}"))
}

fn seed_nodes(conn: &Connection) -> Result<(), String> {
    let nodes: Vec<(&str, &str, &str, &str, Option<&str>)> = vec![
        // 概念节点
        (
            "concept",
            "novelty",
            "新颖性",
            "新颖性是指发明不属于现有技术",
            None,
        ),
        (
            "concept",
            "creativity",
            "创造性",
            "创造性是指发明具有突出的实质性特点和显著的进步",
            None,
        ),
        (
            "concept",
            "practicality",
            "实用性",
            "实用性是指发明能够制造或使用并产生积极效果",
            None,
        ),
        (
            "concept",
            "infringement",
            "专利侵权",
            "专利侵权是指未经许可实施专利",
            None,
        ),
        (
            "concept",
            "invalidity",
            "无效宣告",
            "无效宣告是对已授权专利的挑战",
            None,
        ),
        (
            "concept",
            "examination",
            "审查",
            "专利审查是对申请的审查",
            None,
        ),
        (
            "concept",
            "claim",
            "权利要求",
            "权利要求限定专利保护范围",
            None,
        ),
        // 规则节点
        (
            "rule",
            "three_step",
            "新颖性三步法",
            "1.确定最接近现有技术 2.识别区别特征 3.评估技术效果",
            None,
        ),
        (
            "rule",
            "distinguishing_feature",
            "区别特征",
            "区别特征是指发明与现有技术的不同点",
            None,
        ),
        (
            "rule",
            "technical_effect",
            "技术效果",
            "技术效果是指发明的技术贡献",
            None,
        ),
        (
            "rule",
            "problem_solution",
            "问题-解决方案法",
            "1.确定最接近现有技术 2.确定区别特征和实际解决的技术问题 3.判断显而易见性",
            None,
        ),
        (
            "rule",
            "technical_problem",
            "技术问题",
            "技术问题是发明要解决的技术困难",
            None,
        ),
        (
            "rule",
            "obviousness",
            "显而易见",
            "显而易见是指区别特征为本领域技术人员所容易想到",
            None,
        ),
        (
            "rule",
            "all_elements",
            "全部要素规则",
            "被控侵权产品必须包含权利要求全部技术特征",
            None,
        ),
        (
            "rule",
            "doctrine_of_equivalents",
            "等同原则",
            "以基本相同的手段实现基本相同的功能产生基本相同的效果",
            None,
        ),
        (
            "rule",
            "technical_solution",
            "技术方案",
            "技术方案是发明的具体实现方式",
            None,
        ),
        // 法条节点
        (
            "article",
            "a22_2",
            "专利法第22条第2款",
            "新颖性是指该发明或者实用新型不属于现有技术",
            Some("第22条第2款"),
        ),
        (
            "article",
            "a22_3",
            "专利法第22条第3款",
            "创造性是指与现有技术相比具有突出的实质性特点和显著的进步",
            Some("第22条第3款"),
        ),
        (
            "article",
            "a26_3",
            "专利法第26条第3款",
            "说明书应当对发明作出清楚、完整的说明",
            Some("第26条第3款"),
        ),
        (
            "article",
            "a26_4",
            "专利法第26条第4款",
            "权利要求书应当以说明书为依据",
            Some("第26条第4款"),
        ),
        (
            "article",
            "a45",
            "专利法第45条",
            "无效宣告请求",
            Some("第45条"),
        ),
        (
            "article",
            "rule_guideline_p2c3",
            "审查指南第二部分第三章",
            "新颖性审查",
            Some("第二部分第三章"),
        ),
        (
            "article",
            "rule_guideline_p2c4",
            "审查指南第二部分第四章",
            "创造性审查",
            Some("第二部分第四章"),
        ),
    ];

    for (node_type, id, name, title, content) in &nodes {
        let content_val = content.map(|c| c.to_string());
        conn.execute(
            "INSERT INTO nodes (id, node_type, name, title, content, law_refs_count) VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            params![id, node_type, name, title, content_val],
        )
        .map_err(|e| format!("insert node {id}: {e}"))?;
    }

    // 重建 FTS 索引(external content 表需手动触发)
    conn.execute("INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild')", [])
        .map_err(|e| format!("rebuild fts: {e}"))?;

    Ok(())
}

fn seed_edges(conn: &Connection) -> Result<(), String> {
    let edges: Vec<(&str, &str, &str)> = vec![
        // 概念 → 规则
        ("novelty", "three_step", "defined_by"),
        ("novelty", "distinguishing_feature", "defined_by"),
        ("novelty", "technical_effect", "defined_by"),
        ("creativity", "problem_solution", "defined_by"),
        ("creativity", "obviousness", "defined_by"),
        ("creativity", "technical_problem", "defined_by"),
        ("infringement", "all_elements", "defined_by"),
        ("infringement", "doctrine_of_equivalents", "defined_by"),
        ("claim", "all_elements", "related_to"),
        ("claim", "doctrine_of_equivalents", "related_to"),
        // 概念 → 法条
        ("novelty", "a22_2", "governed_by"),
        ("creativity", "a22_3", "governed_by"),
        ("novelty", "rule_guideline_p2c3", "governed_by"),
        ("creativity", "rule_guideline_p2c4", "governed_by"),
        ("infringement", "all_elements", "applied_by"),
        // 概念间关联
        ("novelty", "creativity", "related_to"),
        ("novelty", "examination", "related_to"),
        ("creativity", "examination", "related_to"),
        ("invalidity", "novelty", "related_to"),
        ("invalidity", "creativity", "related_to"),
        ("infringement", "claim", "related_to"),
        // 规则间关联
        ("three_step", "distinguishing_feature", "step_of"),
        ("three_step", "technical_effect", "step_of"),
        ("problem_solution", "technical_problem", "step_of"),
        ("problem_solution", "obviousness", "step_of"),
        ("problem_solution", "distinguishing_feature", "related_to"),
        ("distinguishing_feature", "technical_effect", "related_to"),
    ];

    for (source, target, relation) in &edges {
        conn.execute(
            "INSERT INTO edges (source, target, relation) VALUES (?1, ?2, ?3)",
            params![source, target, relation],
        )
        .map_err(|e| format!("insert edge {source}->{target}: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_kg_creates_nodes() {
        let kg = in_memory_kg().unwrap();
        let stats = kg.stats().unwrap();
        assert!(
            stats.node_count >= 20,
            "Expected >=20 nodes, got {}",
            stats.node_count
        );
        assert!(
            stats.edge_count >= 10,
            "Expected >=10 edges, got {}",
            stats.edge_count
        );
    }

    #[test]
    fn test_in_memory_kg_search_nodes() {
        let kg = in_memory_kg().unwrap();
        let nodes = kg.search_nodes("新颖性", None, 10).unwrap();
        assert!(!nodes.is_empty(), "Should find nodes for '新颖性'");
        assert!(
            nodes.iter().any(|n| n.id == "novelty"),
            "Should contain 'novelty' node"
        );
    }

    #[test]
    fn test_in_memory_kg_get_edges() {
        let kg = in_memory_kg().unwrap();
        let edges = kg.get_edges("novelty").unwrap();
        assert!(!edges.is_empty(), "novelty should have edges");
        assert!(edges.iter().any(|e| e.relation == "defined_by"
            || e.relation == "governed_by"
            || e.relation == "related_to"));
    }
}
