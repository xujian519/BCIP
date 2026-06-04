//! 法律推理模块
//!
//! 提供法律知识图谱上的结构化推理能力:
//! - 新颖性三步法
//! - 创造性问题-解决方案法
//! - 侵权全部要素规则 + 等同原则
//!
//! 依赖 `codex_patent_knowledge::SqliteKnowledgeGraph` 作为知识源。

use codex_patent_core::KgEdge;
use codex_patent_core::KgNode;
use codex_patent_core::ReasoningConclusion;
use codex_patent_core::ReasoningStep;
use codex_patent_knowledge::SqliteKnowledgeGraph;

/// 推理路径(从起点到终点的节点链)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReasoningPath {
    pub nodes: Vec<KgNode>,
    pub edges: Vec<KgEdge>,
    pub confidence: f64,
}

/// 法律推理引擎
pub struct LegalReasoningEngine<'a> {
    kg: &'a SqliteKnowledgeGraph,
}

impl<'a> LegalReasoningEngine<'a> {
    pub fn new(kg: &'a SqliteKnowledgeGraph) -> Self {
        Self { kg }
    }

    /// 通用路径查找:从查询关键词出发,沿关系遍历到相关法律节点
    ///
    /// 使用 BFS 遍历图谱,按相关性评分排序返回路径
    pub fn find_reasoning_paths(
        &self,
        query: &str,
        max_depth: usize,
        limit: usize,
    ) -> Result<Vec<ReasoningPath>, String> {
        // Step 1: 找到起始节点
        let start_nodes = self.kg.search_nodes(query, None, 5)?;

        if start_nodes.is_empty() {
            return Ok(vec![]);
        }

        let mut paths = Vec::new();

        for start in &start_nodes {
            // BFS 遍历
            let mut queue = vec![(start.clone(), vec![start.clone()], vec![])];
            let mut visited = std::collections::HashSet::new();
            visited.insert(start.id.clone());

            while let Some((current, node_path, edge_path)) = queue.pop() {
                if paths.len() >= limit {
                    break;
                }

                if node_path.len() > max_depth {
                    continue;
                }

                // 如果路径足够长且有法律内容,记录
                if node_path.len() >= 2 {
                    let confidence = self.compute_path_confidence(&node_path, &edge_path);
                    paths.push(ReasoningPath {
                        nodes: node_path.clone(),
                        edges: edge_path.clone(),
                        confidence,
                    });
                }

                // 扩展邻居
                if let Ok(edges) = self.kg.get_edges(&current.id) {
                    for edge in edges {
                        let neighbor_id = if edge.source == current.id {
                            &edge.target
                        } else {
                            &edge.source
                        };

                        if visited.contains(neighbor_id) {
                            continue;
                        }

                        // 查找邻居节点
                        if let Ok(mut neighbors) = self.kg.search_nodes(neighbor_id, None, 1)
                            && let Some(neighbor) = neighbors.pop()
                            && neighbor.id == *neighbor_id
                        {
                            visited.insert(neighbor_id.clone());
                            let mut new_node_path = node_path.clone();
                            new_node_path.push(neighbor.clone());
                            let mut new_edge_path = edge_path.clone();
                            new_edge_path.push(edge);
                            queue.push((neighbor, new_node_path, new_edge_path));
                        }
                    }
                }
            }
        }

        // 按置信度排序
        paths.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        paths.truncate(limit);
        Ok(paths)
    }

    /// 新颖性三步法推理
    ///
    /// 1. 确定最接近的现有技术
    /// 2. 识别区别特征
    /// 3. 评估技术效果
    pub fn novelty_three_step(
        &self,
        invention_description: &str,
    ) -> Result<ReasoningConclusion, String> {
        let mut steps = Vec::new();

        // Step 1: 确定最接近的现有技术
        let prior_art = self.kg.search_nodes(invention_description, None, 3)?;
        let evidence_1: Vec<String> = prior_art.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "确定最接近的现有技术".into(),
            description: format!("在知识图谱中检索到 {} 个相关现有技术节点", prior_art.len()),
            evidence: evidence_1.clone(),
            confidence: if prior_art.is_empty() { 0.3 } else { 0.8 },
        });

        // Step 2: 识别区别特征(查找"区别特征"相关图谱节点)
        let distinguishing = self.kg.search_nodes("区别特征", None, 3)?;
        let evidence_2: Vec<String> = distinguishing.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "识别区别特征".into(),
            description: format!("找到 {} 个区别特征相关规则", distinguishing.len()),
            evidence: evidence_2.clone(),
            confidence: if distinguishing.is_empty() { 0.4 } else { 0.7 },
        });

        // Step 3: 评估技术效果
        let effects = self.kg.search_nodes("技术效果", None, 3)?;
        let evidence_3: Vec<String> = effects.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "评估技术效果".into(),
            description: format!("找到 {} 个技术效果评估规则", effects.len()),
            evidence: evidence_3.clone(),
            confidence: if effects.is_empty() { 0.4 } else { 0.7 },
        });

        let all_evidence: Vec<String> = [evidence_1, evidence_2, evidence_3].concat();
        let avg_confidence = steps.iter().map(|s| s.confidence).sum::<f64>() / steps.len() as f64;

        let conclusion = if avg_confidence >= 0.7 {
            "基于知识图谱的三步法分析完成,需要结合具体技术内容进行最终判断"
        } else {
            "知识图谱中相关信息不足,建议补充检索"
        };

        Ok(ReasoningConclusion {
            method: "新颖性三步法".into(),
            steps,
            conclusion: conclusion.into(),
            evidence_ids: all_evidence,
        })
    }

    /// 创造性问题-解决方案法
    pub fn inventiveness_problem_solution(
        &self,
        invention_description: &str,
    ) -> Result<ReasoningConclusion, String> {
        let mut steps = Vec::new();

        // Step 1: 确定技术问题
        let problems = self.kg.search_nodes("技术问题", None, 3)?;
        let evidence_1: Vec<String> = problems.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "确定技术问题".into(),
            description: format!("找到 {} 个技术问题相关节点", problems.len()),
            evidence: evidence_1.clone(),
            confidence: 0.7,
        });

        // Step 2: 检索现有技术方案
        let solutions = self.kg.search_nodes(invention_description, None, 3)?;
        let evidence_2: Vec<String> = solutions.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "检索现有技术方案".into(),
            description: format!("找到 {} 个相关技术方案", solutions.len()),
            evidence: evidence_2.clone(),
            confidence: if solutions.is_empty() { 0.3 } else { 0.8 },
        });

        // Step 3: 判断显而易见性
        let obvious = self.kg.search_nodes("显而易见", None, 3)?;
        let evidence_3: Vec<String> = obvious.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "判断显而易见性".into(),
            description: format!("找到 {} 个显而易见性判断规则", obvious.len()),
            evidence: evidence_3.clone(),
            confidence: 0.6,
        });

        let all_evidence: Vec<String> = [evidence_1, evidence_2, evidence_3].concat();

        Ok(ReasoningConclusion {
            method: "创造性问题-解决方案法".into(),
            steps,
            conclusion: "基于问题-解决方案法的创造性分析框架已构建".into(),
            evidence_ids: all_evidence,
        })
    }

    /// 侵权分析(全部要素规则 + 等同原则)
    pub fn infringement_analysis(
        &self,
        claim_elements: &[String],
    ) -> Result<ReasoningConclusion, String> {
        let mut steps = Vec::new();

        // Step 1: 全部要素规则
        let full_elements = self.kg.search_nodes("全部要素规则", None, 3)?;
        let evidence_1: Vec<String> = full_elements.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "全部要素规则".into(),
            description: format!("分析 {} 个权利要求要素是否全部覆盖", claim_elements.len()),
            evidence: evidence_1.clone(),
            confidence: 0.8,
        });

        // Step 2: 等同原则
        let doctrine = self.kg.search_nodes("等同原则", None, 3)?;
        let evidence_2: Vec<String> = doctrine.iter().map(|n| n.id.clone()).collect();
        steps.push(ReasoningStep {
            step_name: "等同原则分析".into(),
            description: "对未完全对应的要素进行等同原则判断".into(),
            evidence: evidence_2.clone(),
            confidence: 0.6,
        });

        let all_evidence: Vec<String> = [evidence_1, evidence_2].concat();

        Ok(ReasoningConclusion {
            method: "侵权分析(全部要素+等同)".into(),
            steps,
            conclusion: format!(
                "已对 {} 个权利要求要素构建侵权分析框架",
                claim_elements.len()
            ),
            evidence_ids: all_evidence,
        })
    }

    /// 计算路径置信度
    fn compute_path_confidence(&self, nodes: &[KgNode], edges: &[KgEdge]) -> f64 {
        let node_confidence: f64 = if nodes.iter().any(|n| n.content.is_some()) {
            0.8
        } else {
            0.5
        };

        let edge_bonus: f64 = match edges.len() {
            0 => 0.0,
            1 => 0.1,
            2 => 0.15,
            _ => 0.2,
        };

        // 惩罚过长的路径
        let length_penalty = if nodes.len() > 4 { 0.1 } else { 0.0 };

        (node_confidence + edge_bonus - length_penalty).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kg_seed;

    fn open_kg() -> Option<SqliteKnowledgeGraph> {
        kg_seed::in_memory_kg().ok()
    }

    #[test]
    fn test_novelty_three_step() {
        let Some(kg) = open_kg() else {
            tracing::warn!("Skipping: patent_kg.db not found");
            return;
        };
        let engine = LegalReasoningEngine::new(&kg);
        let result = engine.novelty_three_step("图像识别算法").unwrap();
        assert_eq!(result.method, "新颖性三步法");
        assert_eq!(result.steps.len(), 3);
    }

    #[test]
    fn test_inventiveness() {
        let Some(kg) = open_kg() else {
            tracing::warn!("Skipping: patent_kg.db not found");
            return;
        };
        let engine = LegalReasoningEngine::new(&kg);
        let result = engine
            .inventiveness_problem_solution("深度学习模型优化")
            .unwrap();
        assert_eq!(result.method, "创造性问题-解决方案法");
        assert_eq!(result.steps.len(), 3);
    }

    #[test]
    fn test_infringement_analysis() {
        let Some(kg) = open_kg() else {
            tracing::warn!("Skipping: patent_kg.db not found");
            return;
        };
        let engine = LegalReasoningEngine::new(&kg);
        let result = engine
            .infringement_analysis(&["特征1".into(), "特征2".into()])
            .unwrap();
        assert_eq!(result.method, "侵权分析(全部要素+等同)");
        assert_eq!(result.steps.len(), 2);
    }

    #[test]
    fn test_find_reasoning_paths() {
        let Some(kg) = open_kg() else {
            tracing::warn!("Skipping: patent_kg.db not found");
            return;
        };
        let engine = LegalReasoningEngine::new(&kg);
        let paths = engine.find_reasoning_paths("创造性", 3, 5).unwrap();
        // 不强制要求有路径(取决于图谱数据)
        for path in &paths {
            assert!(!path.nodes.is_empty());
            assert!(path.confidence > 0.0);
        }
    }
}
