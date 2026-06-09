//! 权利要求结构化生成器
//!
//! 从发明理解生成权利要求文本。
//! 三步法：前序构建 → 特征部分构建 → 组装。

use codex_patent_core::*;
use serde::Deserialize;
use serde::Serialize;

/// 权利要求布局策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimLayout {
    Standard {
        dependent_count: usize,
    },
    DualClaim {
        method_dependents: usize,
        product_dependents: usize,
    },
    MultipleDependency {
        primary_deps: usize,
        secondary_deps: usize,
    },
}

/// 权利要求类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimCategory {
    Method,
    Product,
    Use,
}

/// 高级权利要求输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedClaimInput {
    pub invention_name: String,
    pub essential_features: Vec<String>,
    pub optional_features: Vec<Vec<String>>,
    pub layout: ClaimLayout,
    pub claim_type: ClaimCategory,
    pub preamble_text: Option<String>,
}

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

/// 生成独立权利要求
pub fn generate_independent(
    invention_title: &str,
    features: &[TechnicalFeature],
    _options: &ClaimGenerationOptions,
) -> ClaimDraft {
    let preamble = build_preamble(invention_title);
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

    let independent = generate_independent(invention_title, essential_features, options);
    claims.push(independent);

    let max_dependent = (options.claim_count - 1).min(optional_features.len());
    for (i, feature) in optional_features.iter().take(max_dependent).enumerate() {
        let dep = generate_dependent(1, "claim-1", feature, i);
        claims.push(dep);
    }

    claims
}

/// 按布局策略生成权利要求书
pub fn generate_claims_advanced(input: &AdvancedClaimInput) -> Vec<String> {
    match &input.layout {
        ClaimLayout::Standard { dependent_count } => {
            generate_standard_layout(input, *dependent_count)
        }
        ClaimLayout::DualClaim {
            method_dependents,
            product_dependents,
        } => generate_dual_layout(input, *method_dependents, *product_dependents),
        ClaimLayout::MultipleDependency {
            primary_deps,
            secondary_deps,
        } => generate_multi_dep_layout(input, *primary_deps, *secondary_deps),
    }
}

/// 标准布局：1独权 + N从权
fn generate_standard_layout(input: &AdvancedClaimInput, dependent_count: usize) -> Vec<String> {
    let mut claims = Vec::new();
    let name = &input.invention_name;
    let preamble = input.preamble_text.as_deref().unwrap_or("一种");

    // 独立权利要求
    let independent = format!(
        "{preamble}{name}，其特征在于，包括：{}。",
        input.essential_features.join("；")
    );
    claims.push(independent);

    // 从属权利要求
    let dep_count = dependent_count.min(input.optional_features.len());
    for (i, group) in input.optional_features.iter().take(dep_count).enumerate() {
        let claim_num = i + 2; // 1-based, 从权从2开始
        let features_text = group.join("；");
        claims.push(format!(
            "根据权利要求{claim_num}所述的{name}，其特征在于，还包括：{features_text}。"
        ));
    }

    claims
}

/// 双报布局：方法独权 + 产品独权 + 各自从权
fn generate_dual_layout(
    input: &AdvancedClaimInput,
    method_dependents: usize,
    product_dependents: usize,
) -> Vec<String> {
    let mut claims = Vec::new();
    let name = &input.invention_name;
    let mut claim_number: usize = 1;

    // 方法独立权利要求
    let method_independent = format!(
        "一种{name}方法，其特征在于，包括：{}。",
        input.essential_features.join("；")
    );
    claims.push(method_independent);
    let method_independent_num = claim_number;
    claim_number += 1;

    // 方法从属权利要求
    let method_dep_count = method_dependents.min(input.optional_features.len());
    for group in input.optional_features.iter().take(method_dep_count) {
        let features_text = group.join("；");
        claims.push(format!(
            "根据权利要求{method_independent_num}所述的{name}方法，其特征在于，还包括：{features_text}。"
        ));
        claim_number += 1;
    }

    // 产品独立权利要求
    let product_label = match &input.claim_type {
        ClaimCategory::Product => "装置",
        _ => "系统",
    };
    let product_independent = format!(
        "一种{name}{product_label}，其特征在于，包括：{}。",
        input.essential_features.join("；")
    );
    claims.push(product_independent);
    let product_independent_num = claim_number;

    // 产品从属权利要求
    let offset = method_dep_count;
    let product_dep_count =
        product_dependents.min(input.optional_features.len().saturating_sub(offset));
    for group in input
        .optional_features
        .iter()
        .skip(offset)
        .take(product_dep_count)
    {
        let features_text = group.join("；");
        claims.push(format!(
            "根据权利要求{product_independent_num}所述的{name}{product_label}，其特征在于，还包括：{features_text}。"
        ));
    }

    claims
}

/// 多项引用布局：从权引用多项前权
fn generate_multi_dep_layout(
    input: &AdvancedClaimInput,
    primary_deps: usize,
    secondary_deps: usize,
) -> Vec<String> {
    let mut claims = Vec::new();
    let name = &input.invention_name;
    let preamble = input.preamble_text.as_deref().unwrap_or("一种");

    // 独立权利要求
    let independent = format!(
        "{preamble}{name}，其特征在于，包括：{}。",
        input.essential_features.join("；")
    );
    claims.push(independent);
    let mut claim_number: usize = 1;

    // 主从属权利要求（引用独权1）
    let primary_count = primary_deps.min(input.optional_features.len());
    for group in input.optional_features.iter().take(primary_count) {
        claim_number += 1;
        let features_text = group.join("；");
        claims.push(format!(
            "根据权利要求1所述的{name}，其特征在于，还包括：{features_text}。"
        ));
    }

    // 次级从属权利要求（引用前多项权利要求）
    let offset = primary_count;
    let secondary_count = secondary_deps.min(input.optional_features.len().saturating_sub(offset));
    for group in input
        .optional_features
        .iter()
        .skip(offset)
        .take(secondary_count)
    {
        claim_number += 1;
        // 引用前一项权利要求（单项引用，确保确定性）
        let ref_num = claim_number - 1;
        let features_text = group.join("；");
        claims.push(format!(
            "根据权利要求{ref_num}所述的{name}，其特征在于，还包括：{features_text}。"
        ));
    }

    claims
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
        let claim = generate_independent(
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
        let claim = generate_dependent(1, "claim-1", &feature, 0);
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
        let claims = generate_claim_set("测试系统", &essential, &optional, &options);
        assert_eq!(claims.len(), 5);
        assert_eq!(claims[0].claim_type, ClaimType::Independent);
        assert_eq!(claims[1].claim_type, ClaimType::Dependent);
    }

    #[test]
    fn empty_optional_returns_only_independent() {
        let essential = vec![make_feature("f1", "核心模块", FeatureCategory::Structural)];
        let claims =
            generate_claim_set("装置", &essential, &[], &ClaimGenerationOptions::default());
        assert_eq!(claims.len(), 1);
    }

    #[test]
    fn structural_and_functional_separated() {
        let features = vec![
            make_feature("f1", "传感器模块", FeatureCategory::Structural),
            make_feature("f2", "实时采集数据", FeatureCategory::Functional),
        ];
        let claim = generate_independent("检测系统", &features, &ClaimGenerationOptions::default());
        assert!(claim.elements.contains(&"传感器模块".to_string()));
        assert!(claim.elements.contains(&"实时采集数据".to_string()));
    }

    fn make_advanced_input(layout: ClaimLayout, claim_type: ClaimCategory) -> AdvancedClaimInput {
        AdvancedClaimInput {
            invention_name: "数据处理".into(),
            essential_features: vec!["数据采集模块".into(), "处理模块".into()],
            optional_features: vec![
                vec!["缓存单元".into()],
                vec!["压缩模块".into(), "解压模块".into()],
                vec!["加密模块".into()],
                vec!["日志模块".into()],
            ],
            layout,
            claim_type,
            preamble_text: None,
        }
    }

    #[test]
    fn test_standard_layout() {
        let input = make_advanced_input(
            ClaimLayout::Standard { dependent_count: 3 },
            ClaimCategory::Method,
        );
        let claims = generate_claims_advanced(&input);
        // 1 独立 + 3 从属
        assert_eq!(claims.len(), 4);
        // 独立权利要求
        assert!(claims[0].contains("一种数据处理，其特征在于"));
        assert!(claims[0].contains("数据采集模块"));
        // 从属权利要求引用格式
        assert!(claims[1].contains("根据权利要求2所述的数据处理"));
        assert!(claims[2].contains("根据权利要求3所述的数据处理"));
        assert!(claims[3].contains("根据权利要求4所述的数据处理"));
    }

    #[test]
    fn test_dual_claim_layout() {
        let input = make_advanced_input(
            ClaimLayout::DualClaim {
                method_dependents: 2,
                product_dependents: 2,
            },
            ClaimCategory::Product,
        );
        let claims = generate_claims_advanced(&input);
        // 1方法独权 + 2方法从权 + 1产品独权 + 2产品从权 = 6
        assert_eq!(claims.len(), 6);
        // 方法独立权利要求
        assert!(claims[0].contains("一种数据处理方法"));
        // 方法从属权利要求引用1
        assert!(claims[1].contains("根据权利要求1所述的数据处理方法"));
        assert!(claims[2].contains("根据权利要求1所述的数据处理方法"));
        // 产品独立权利要求
        assert!(claims[3].contains("一种数据处理装置"));
        // 产品从属权利要求引用4
        assert!(claims[4].contains("根据权利要求4所述的数据处理装置"));
        assert!(claims[5].contains("根据权利要求4所述的数据处理装置"));
    }

    #[test]
    fn test_multi_dep_layout() {
        let input = make_advanced_input(
            ClaimLayout::MultipleDependency {
                primary_deps: 2,
                secondary_deps: 2,
            },
            ClaimCategory::Method,
        );
        let claims = generate_claims_advanced(&input);
        // 1 独立 + 2 主从属 + 2 次级从属 = 5
        assert_eq!(claims.len(), 5);
        // 独立权利要求
        assert!(claims[0].contains("一种数据处理，其特征在于"));
        // 主从属引用1
        assert!(claims[1].contains("根据权利要求1所述的数据处理"));
        assert!(claims[2].contains("根据权利要求1所述的数据处理"));
        // 次级从属引用前一项
        assert!(claims[3].contains("根据权利要求3所述的数据处理"));
        assert!(claims[4].contains("根据权利要求4所述的数据处理"));
    }
}
