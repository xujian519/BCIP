//! 发明类型分类器 — 基于特征对比结果的启发式分类。
//!
//! 根据权利要求特征与对比文件特征的匹配程度，判断发明属于
//! 开拓性发明、选择发明或未知类型。

use codex_patent_core::CompareFeature;
use codex_patent_core::InventionType;

use crate::compare::FeatureMatcher;

/// 基于特征对比结果的启发式发明类型分类器。
pub struct InventionClassifier;

impl InventionClassifier {
    /// 根据权利要求特征与对比文件特征的匹配结果，判断发明类型。
    pub fn classify(
        claim_features: &[CompareFeature],
        prior_art_features: &[CompareFeature],
    ) -> InventionType {
        if prior_art_features.is_empty() {
            return InventionType::Pioneering;
        }

        let result = FeatureMatcher::compare(claim_features, prior_art_features);

        if result.coverage_ratio < 0.1 {
            return InventionType::Pioneering;
        }

        if result.coverage_ratio > 0.9 {
            return InventionType::Unknown;
        }

        let has_numeric = claim_features
            .iter()
            .any(|f| contains_numeric(&f.description));
        if has_numeric
            && result
                .different_features
                .iter()
                .any(|d| contains_numeric(d))
        {
            return InventionType::Selection;
        }

        InventionType::Unknown
    }
}

fn contains_numeric(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pioneering_no_prior_art() {
        let claims = vec![CompareFeature {
            id: "C1".into(),
            description: "量子计算模块".into(),
        }];
        let result = InventionClassifier::classify(&claims, &[]);
        assert_eq!(result, InventionType::Pioneering);
    }

    #[test]
    fn test_selection_with_numeric() {
        let claims = vec![
            CompareFeature {
                id: "C1".into(),
                description: "温度范围200-350度".into(),
            },
            CompareFeature {
                id: "C2".into(),
                description: "催化剂".into(),
            },
        ];
        let prior = vec![
            CompareFeature {
                id: "P1".into(),
                description: "温度范围100-500度".into(),
            },
            CompareFeature {
                id: "P2".into(),
                description: "催化剂".into(),
            },
        ];
        let result = InventionClassifier::classify(&claims, &prior);
        assert_eq!(result, InventionType::Selection);
    }

    #[test]
    fn test_unknown_high_coverage() {
        let claims = vec![CompareFeature {
            id: "C1".into(),
            description: "传感器".into(),
        }];
        let prior = vec![CompareFeature {
            id: "P1".into(),
            description: "传感器".into(),
        }];
        let result = InventionClassifier::classify(&claims, &prior);
        assert_eq!(result, InventionType::Unknown);
    }
}
