//! 审查指南图谱与法律知识图谱
//!
//! 基于 JSON 文件加载审查指南结构化数据和法律实体关系图。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 审查指南元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuidelineMetadata {
    pub title: String,
    pub description: String,
    pub created: String,
    #[serde(default)]
    pub sections: Vec<serde_json::Value>,
    pub total_nodes: usize,
    pub total_relationships: usize,
}

/// 审查指南节点属性
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuidelineNodeProperties {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub level: Option<usize>,
}

/// 审查指南节点
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuidelineNode {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub node_type: String,
    #[serde(default)]
    pub properties: GuidelineNodeProperties,
}

/// 审查指南关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineRelationship {
    #[serde(default)]
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    #[serde(default)]
    pub properties: Option<serde_json::Value>,
}

/// 向量条目(预留)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorEntry {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub vector: Vec<f64>,
}

/// 审查指南图谱
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuidelineGraph {
    #[serde(default)]
    pub metadata: GuidelineMetadata,
    #[serde(default)]
    pub nodes: Vec<GuidelineNode>,
    #[serde(default)]
    pub relationships: Vec<GuidelineRelationship>,
    #[serde(default)]
    pub vectors: Vec<VectorEntry>,
}

/// 法律实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalEntity {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub entity_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub properties: Option<serde_json::Value>,
}

/// 法律关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalRelationship {
    #[serde(default)]
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    #[serde(rename = "type", default)]
    pub rel_type: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// 法律知识图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalKnowledgeGraph {
    #[serde(default)]
    pub entities: HashMap<String, LegalEntity>,
    #[serde(default)]
    pub relationships: Vec<LegalRelationship>,
}

/// 从 JSON 文件加载审查指南图谱
pub fn load_guideline_graph(path: &str) -> Result<GuidelineGraph, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("读取审查指南图谱失败 {path}: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析审查指南图谱失败: {e}"))
}

/// 从 JSON 文件加载法律知识图谱
pub fn load_legal_graph(
    entities_path: &str,
    rels_path: &str,
) -> Result<LegalKnowledgeGraph, String> {
    let entities_content =
        std::fs::read_to_string(entities_path).map_err(|e| format!("读取法律实体失败: {e}"))?;
    let entities: HashMap<String, LegalEntity> =
        serde_json::from_str(&entities_content).map_err(|e| format!("解析法律实体失败: {e}"))?;

    let rels_content =
        std::fs::read_to_string(rels_path).map_err(|e| format!("读取法律关系失败: {e}"))?;
    let relationships: Vec<LegalRelationship> =
        serde_json::from_str(&rels_content).map_err(|e| format!("解析法律关系失败: {e}"))?;

    Ok(LegalKnowledgeGraph {
        entities,
        relationships,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_guideline_graph() {
        let path = "../../../assets/knowledge_graph/guideline_graph.json";
        if !std::path::Path::new(path).exists() {
            eprintln!("Skipping test: guideline_graph.json not found");
            return;
        }
        let graph = load_guideline_graph(path).unwrap();
        assert!(!graph.metadata.title.is_empty());
    }

    #[test]
    fn test_load_legal_graph() {
        let entities = "../../../assets/knowledge_graph/legal_entities.json";
        let rels = "../../../assets/knowledge_graph/legal_relationships.json";
        if !std::path::Path::new(entities).exists() {
            eprintln!("Skipping test: legal entity files not found");
            return;
        }
        let graph = load_legal_graph(entities, rels).unwrap();
        assert!(!graph.entities.is_empty());
    }
}
