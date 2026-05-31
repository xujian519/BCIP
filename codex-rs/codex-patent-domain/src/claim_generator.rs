//! 权利要求结构化生成器
//!
//! 从发明理解生成权利要求文本。
//! 三步法：前序构建 → 特征部分构建 → 组装。

use codex_patent_core::*;

/// 权利要求生成选项
#[derive(Debug, Clone)]
pub struct ClaimGenerationOptions {
    pub claim_count: usize,
    pub include_method_claims: bool,
    pub writing_style: String,
}

impl Default for ClaimGenerationOptions {
    fn default() -> Self {
        Self {
            claim_count: 6,
            include_method_claims: true,
            writing_style: "formal".into(),
        }
    }
}

/// 权利要求生成器
pub struct ClaimGenerator;

impl ClaimGenerator {
    pub fn new() -> Self {
        Self
    }

    /// 生成独立权利要求
    pub fn generate_independent(
        invention_title: &str,
        features: &[TechnicalFeature],
        _options: &ClaimGenerationOptions,
    ) -> ClaimDraft {
        let preamble = Self::build_preamble(invention_title);
        let elements = features.iter().map(|f| f.description.clone()).collect();

        ClaimDraft {
            id: "claim-1".into(),
            claim_type: ClaimType::Independent,
            preamble,
            transitional_phrase: "其特征在于".into(),
            elements,
            dependent_on: None,
        }
    }

    /// 生成从属权利要求
    pub fn generate_dependent(
        base_claim_number: u32,
        base_claim_id: &str,
        feature: &TechnicalFeature,
        order: usize,
    ) -> ClaimDraft {
        let id = format!("claim-{}", order + 1);
        let preamble = format!("根据权利要求{base_claim_number}所述的");

        ClaimDraft {
            id,
            claim_type: ClaimType::Dependent,
            preamble,
            transitional_phrase: String::new(),
            elements: vec![feature.description.clone()],
            dependent_on: Some(base_claim_id.into()),
        }
    }

    /// 构建前序部分
    fn build_preamble(title: &str) -> String {
        format!("一种{title}，")
    }

    /// 构建特征部分
    /// 构建特征部分（预留：供生成完整权利要求文本使用）
    #[allow(dead_code)]
    fn build_feature_clause(features: &[TechnicalFeature]) -> String {
        let structural: Vec<String> = features
            .iter()
            .filter(|f| f.category == FeatureCategory::Structural)
            .map(|f| f.description.clone())
            .collect();
        let functional: Vec<String> = features
            .iter()
            .filter(|f| f.category == FeatureCategory::Functional)
            .map(|f| f.description.clone())
            .collect();

        let mut parts = Vec::new();
        if !structural.is_empty() {
            parts.push(format!("包括：{}；", structural.join("，")));
        }
        if !functional.is_empty() {
            parts.push(functional.join("；"));
        }
        parts.join(" ")
    }

    /// 生成完整权利要求集
    pub fn generate_claim_set(
        invention_title: &str,
        essential_features: &[TechnicalFeature],
        optional_features: &[TechnicalFeature],
        options: &ClaimGenerationOptions,
    ) -> Vec<ClaimDraft> {
        let mut claims = Vec::new();

        let independent = Self::generate_independent(invention_title, essential_features, options);
        claims.push(independent);

        let max_dependent = (options.claim_count - 1).min(optional_features.len());
        for (i, feature) in optional_features.iter().take(max_dependent).enumerate() {
            let dep = Self::generate_dependent(1, "claim-1", feature, i);
            claims.push(dep);
        }

        claims
    }
}

impl Default for ClaimGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_feature(id: &str, desc: &str, cat: FeatureCategory) -> TechnicalFeature {
        TechnicalFeature {
            id: id.into(),
            description: desc.into(),
            feature_type: FeatureType::Element,
            category: cat,
            component: None,
            function: None,
        }
    }

    #[test]
    fn independent_claim_has_preamble() {
        let features = vec![make_feature("f1", "处理器", FeatureCategory::Structural)];
        let claim = ClaimGenerator::generate_independent(
            "数据处理装置",
            &features,
            &ClaimGenerationOptions::default(),
        );
        assert_eq!(claim.claim_type, ClaimType::Independent);
        assert!(claim.preamble.contains("数据处理装置"));
        assert!(claim.transitional_phrase.contains("其特征在于"));
    }

    #[test]
    fn dependent_claim_refs_parent() {
        let feature = make_feature("f2", "所述处理器为CPU", FeatureCategory::Structural);
        let claim = ClaimGenerator::generate_dependent(1, "claim-1", &feature, 0);
        assert_eq!(claim.claim_type, ClaimType::Dependent);
        assert!(claim.preamble.contains("根据权利要求1"));
        assert_eq!(claim.dependent_on, Some("claim-1".into()));
    }

    #[test]
    fn claim_set_respects_max_count() {
        let essential = vec![make_feature("f1", "处理器", FeatureCategory::Structural)];
        let optional = (0..7)
            .map(|i| {
                make_feature(
                    &format!("f{}", i + 2),
                    &format!("组件{}", i + 2),
                    FeatureCategory::Structural,
                )
            })
            .collect::<Vec<_>>();
        let options = ClaimGenerationOptions {
            claim_count: 5,
            ..Default::default()
        };
        let claims =
            ClaimGenerator::generate_claim_set("测试系统", &essential, &optional, &options);
        assert_eq!(claims.len(), 5);
        assert_eq!(claims[0].claim_type, ClaimType::Independent);
        assert_eq!(claims[1].claim_type, ClaimType::Dependent);
    }

    #[test]
    fn empty_optional_returns_only_independent() {
        let essential = vec![make_feature("f1", "核心模块", FeatureCategory::Structural)];
        let claims = ClaimGenerator::generate_claim_set(
            "装置",
            &essential,
            &[],
            &ClaimGenerationOptions::default(),
        );
        assert_eq!(claims.len(), 1);
    }

    #[test]
    fn structural_and_functional_separated() {
        let features = vec![
            make_feature("f1", "传感器模块", FeatureCategory::Structural),
            make_feature("f2", "实时采集数据", FeatureCategory::Functional),
        ];
        let claim = ClaimGenerator::generate_independent(
            "检测系统",
            &features,
            &ClaimGenerationOptions::default(),
        );
        assert!(claim.elements.contains(&"传感器模块".to_string()));
        assert!(claim.elements.contains(&"实时采集数据".to_string()));
    }
}
