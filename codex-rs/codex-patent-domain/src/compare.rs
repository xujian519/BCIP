//! 专利对比矩阵与特征匹配。

use codex_patent_core::CompareFeature;
use codex_patent_core::InfringementType;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

/// 特征矩阵单元格
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatrixCell {
    pub target_index: usize,
    pub prior_index: usize,
    pub lexical_score: f64,
    pub matched: bool,
}

/// 特征矩阵
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatrix {
    pub cells: Vec<FeatureMatrixCell>,
    pub target_only: Vec<String>,
    pub prior_only: Vec<String>,
    pub overlap_ratio: f64,
}

/// 结构化 diff
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredDiff {
    pub feature_matrix: FeatureMatrix,
    pub ipc_alignment: f64,
    pub target_ipc: Vec<String>,
    pub prior_ipc: Vec<String>,
    pub distinguishing_features: Vec<String>,
    pub summary: String,
}

/// 匹配类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Exact,
    Equivalent,
    Different,
    Missing,
}

/// 单条特征匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatch {
    pub target_feature: String,
    pub prior_feature: String,
    pub similarity_score: f64,
    pub match_type: MatchType,
}

/// 特征匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatchResult {
    pub exact_matches: Vec<FeatureMatch>,
    pub equivalent_matches: Vec<FeatureMatch>,
    pub different_features: Vec<String>,
    pub missing_features: Vec<String>,
    pub coverage_ratio: f64,
    pub infringement_type: Option<InfringementType>,
}

// ---- 核心函数 ----

/// 构建特征矩阵
pub fn build_feature_matrix(target: &[CompareFeature], prior: &[CompareFeature]) -> FeatureMatrix {
    let mut cells = Vec::new();
    let mut matched_target = HashSet::new();
    let mut matched_prior = HashSet::new();

    for (ti, tf) in target.iter().enumerate() {
        for (pi, pf) in prior.iter().enumerate() {
            let score = lexical_similarity(&tf.description, &pf.description);
            let matched = score >= 0.45;
            if matched {
                matched_target.insert(ti);
                matched_prior.insert(pi);
            }
            cells.push(FeatureMatrixCell {
                target_index: ti,
                prior_index: pi,
                lexical_score: score,
                matched,
            });
        }
    }

    let target_only: Vec<String> = target
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_target.contains(i))
        .map(|(_, f)| f.description.clone())
        .collect();

    let prior_only: Vec<String> = prior
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_prior.contains(i))
        .map(|(_, f)| f.description.clone())
        .collect();

    let total = target.len() + prior.len();
    let overlap = matched_target.len() + matched_prior.len();
    let overlap_ratio = if total > 0 {
        overlap as f64 / total as f64
    } else {
        0.0
    };

    FeatureMatrix {
        cells,
        target_only,
        prior_only,
        overlap_ratio,
    }
}

/// 词法相似度(Jaccard bigram)
pub fn lexical_similarity(a: &str, b: &str) -> f64 {
    let a_chars: Vec<_> = a.chars().collect();
    let b_chars: Vec<_> = b.chars().collect();
    let a_bigrams: HashSet<_> = a_chars.windows(2).collect();
    let b_bigrams: HashSet<_> = b_chars.windows(2).collect();

    if a_bigrams.is_empty() || b_bigrams.is_empty() {
        return 0.0;
    }

    let intersection: HashSet<_> = a_bigrams.intersection(&b_bigrams).cloned().collect();
    let union: HashSet<_> = a_bigrams.union(&b_bigrams).cloned().collect();

    intersection.len() as f64 / union.len() as f64
}

/// 计算 IPC 对齐度
pub fn ipc_alignment(target_ipc: &[String], prior_ipc: &[String]) -> f64 {
    if target_ipc.is_empty() || prior_ipc.is_empty() {
        return 0.0;
    }

    let target_set: HashSet<_> = target_ipc.iter().cloned().collect();
    let prior_set: HashSet<_> = prior_ipc.iter().cloned().collect();

    let intersection = target_set.intersection(&prior_set).count();
    let union = target_set.union(&prior_set).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// 特征匹配器
pub struct FeatureMatcher;

impl FeatureMatcher {
    /// 对比目标特征与现有技术特征，返回匹配结果
    ///
    /// 使用词法相似度（bigram Jaccard）逐一匹配，自动分类为精确匹配、等同匹配、不同或缺失。
    pub fn compare(target: &[CompareFeature], prior: &[CompareFeature]) -> FeatureMatchResult {
        let mut exact = Vec::new();
        let mut equivalent = Vec::new();
        let mut different = Vec::new();
        let mut missing = Vec::new();
        let mut matched_prior = HashSet::new();

        for tf in target {
            let mut best_score = 0.0;
            let mut best_prior = None;

            for (pi, pf) in prior.iter().enumerate() {
                let score = lexical_similarity(&tf.description, &pf.description);
                if score > best_score {
                    best_score = score;
                    best_prior = Some((pi, pf));
                }
            }

            if let Some((pi, pf)) = best_prior {
                matched_prior.insert(pi);
                if best_score >= 0.9 {
                    exact.push(FeatureMatch {
                        target_feature: tf.description.clone(),
                        prior_feature: pf.description.clone(),
                        similarity_score: best_score,
                        match_type: MatchType::Exact,
                    });
                } else if best_score >= 0.6 {
                    equivalent.push(FeatureMatch {
                        target_feature: tf.description.clone(),
                        prior_feature: pf.description.clone(),
                        similarity_score: best_score,
                        match_type: MatchType::Equivalent,
                    });
                } else {
                    different.push(tf.description.clone());
                }
            } else {
                missing.push(tf.description.clone());
            }
        }

        let coverage = if target.is_empty() {
            0.0
        } else {
            (exact.len() + equivalent.len()) as f64 / target.len() as f64
        };

        let infringement = if exact.len() == target.len() {
            Some(InfringementType::Literal)
        } else if exact.len() + equivalent.len() == target.len() {
            Some(InfringementType::DoctrineOfEquivalents)
        } else {
            Some(InfringementType::NoInfringement)
        };

        FeatureMatchResult {
            exact_matches: exact,
            equivalent_matches: equivalent,
            different_features: different,
            missing_features: missing,
            coverage_ratio: coverage,
            infringement_type: infringement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexical_similarity() {
        let s1 = "一种数据处理系统";
        let s2 = "一种数据处理方法";
        let score = lexical_similarity(s1, s2);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_feature_matrix() {
        let target = vec![
            CompareFeature {
                id: "f1".into(),
                description: "包含传感器模块".into(),
            },
            CompareFeature {
                id: "f2".into(),
                description: "包含处理器".into(),
            },
        ];
        let prior = vec![
            CompareFeature {
                id: "p1".into(),
                description: "包含传感器单元".into(),
            },
            CompareFeature {
                id: "p2".into(),
                description: "包含控制器".into(),
            },
        ];

        let matrix = build_feature_matrix(&target, &prior);
        assert!(!matrix.cells.is_empty());
    }

    #[test]
    fn test_feature_matcher() {
        let target = vec![
            CompareFeature {
                id: "f1".into(),
                description: "A模块".into(),
            },
            CompareFeature {
                id: "f2".into(),
                description: "B模块".into(),
            },
        ];
        let prior = vec![
            CompareFeature {
                id: "p1".into(),
                description: "A模块".into(),
            },
            CompareFeature {
                id: "p2".into(),
                description: "C模块".into(),
            },
        ];

        let result = FeatureMatcher::compare(&target, &prior);
        assert!(!result.exact_matches.is_empty());
        assert!(result.coverage_ratio > 0.0);
    }

    #[test]
    fn test_lexical_similarity_identical() {
        let score = lexical_similarity("一种数据处理系统", "一种数据处理系统");
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_lexical_similarity_empty() {
        assert_eq!(lexical_similarity("", "test"), 0.0);
        assert_eq!(lexical_similarity("test", ""), 0.0);
        assert_eq!(lexical_similarity("a", "b"), 0.0);
    }

    #[test]
    fn test_lexical_similarity_single_char() {
        let score = lexical_similarity("X", "X");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_ipc_alignment_identical() {
        let score = ipc_alignment(
            &["G06F".into(), "H04L".into()],
            &["G06F".into(), "H04L".into()],
        );
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ipc_alignment_empty() {
        let empty: Vec<String> = vec![];
        assert_eq!(ipc_alignment(&[], &["G06F".into()]), 0.0);
        assert_eq!(ipc_alignment(&["G06F".into()], &empty), 0.0);
        assert_eq!(ipc_alignment(&empty, &empty), 0.0);
    }

    #[test]
    fn test_ipc_alignment_no_overlap() {
        let score = ipc_alignment(&["G06F".into()], &["H04L".into()]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_ipc_alignment_partial_overlap() {
        let score = ipc_alignment(
            &["G06F".into(), "H04L".into()],
            &["G06F".into(), "A01B".into()],
        );
        assert!((score - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_build_feature_matrix_empty() {
        let matrix = build_feature_matrix(&[], &[]);
        assert!(matrix.cells.is_empty());
        assert!(matrix.target_only.is_empty());
        assert!(matrix.prior_only.is_empty());
        assert_eq!(matrix.overlap_ratio, 0.0);
    }

    #[test]
    fn test_feature_matcher_all_exact() {
        let features = vec![
            CompareFeature {
                id: "f1".into(),
                description: "包含传感器模块".into(),
            },
            CompareFeature {
                id: "f2".into(),
                description: "包含处理器".into(),
            },
        ];
        let result = FeatureMatcher::compare(&features, &features);
        assert_eq!(result.exact_matches.len(), 2);
        assert_eq!(result.coverage_ratio, 1.0);
        assert_eq!(result.infringement_type, Some(InfringementType::Literal));
    }

    #[test]
    fn test_feature_matcher_empty_target() {
        let prior = vec![CompareFeature {
            id: "p1".into(),
            description: "包含传感器模块".into(),
        }];
        let result = FeatureMatcher::compare(&[], &prior);
        assert_eq!(result.coverage_ratio, 0.0);
        assert!(result.exact_matches.is_empty());
    }

    #[test]
    fn test_feature_matcher_equivalent_match() {
        let target = vec![CompareFeature {
            id: "f1".into(),
            description: "一种数据处理系统包含存储单元".into(),
        }];
        let prior = vec![CompareFeature {
            id: "p1".into(),
            description: "一种数据处理装置包含存储单元".into(),
        }];
        let result = FeatureMatcher::compare(&target, &prior);
        assert_eq!(
            result.infringement_type,
            Some(InfringementType::DoctrineOfEquivalents)
        );
    }

    #[test]
    fn test_feature_matrix_cell_serialize() {
        let cell = FeatureMatrixCell {
            target_index: 0,
            prior_index: 1,
            lexical_score: 0.85,
            matched: true,
        };
        let json = serde_json::to_string(&cell).unwrap();
        assert!(json.contains("targetIndex"));
        assert!(json.contains("lexicalScore"));
    }
}
