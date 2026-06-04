//! 质量规则配置加载与词汇查询
//!
//! 从 YAML 配置文件加载专利说明书质量相关的关键词列表、正则模式及阈值。
//! 提供 `vague_words()` 等静态词汇表及 `commercial_terms()` 等动态配置查询。

use serde::Deserialize;
use std::sync::OnceLock;

const RULES_YAML: &str = include_str!("../../codex-patent-assets/rules/spec-quality.yaml");

static CONFIG_CACHE: OnceLock<SpecQualityConfig> = OnceLock::new();

fn load_config() -> &'static SpecQualityConfig {
    CONFIG_CACHE.get_or_init(|| serde_yaml::from_str(RULES_YAML).unwrap_or_default())
}

#[derive(Debug, Deserialize, Default)]
struct SpecQualityConfig {
    #[serde(default)]
    keyword_lists: KeywordLists,
    #[serde(default)]
    patterns: Patterns,
    #[serde(default)]
    thresholds: Thresholds,
}

#[derive(Debug, Deserialize, Default)]
struct KeywordLists {
    #[serde(default)]
    commercial_terms: Category,
    #[serde(default)]
    uncertain_terms: Category,
    #[serde(default)]
    vague_range_terms: Category,
    #[serde(default)]
    fuzzy_action_terms: Category,
}

#[derive(Debug, Deserialize, Default)]
struct Category {
    #[serde(default)]
    items: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct Patterns {
    #[serde(default)]
    prohibited_references: PatternEntry,
    #[serde(default)]
    #[allow(dead_code)] // 预留给后续术语提取检查
    term_extraction: PatternEntry,
}

#[derive(Debug, Deserialize, Default)]
struct PatternEntry {
    #[serde(default)]
    regex: String,
}

#[derive(Debug, Deserialize, Default)]
struct Thresholds {
    #[serde(default)]
    #[allow(dead_code)] // 预留给后续术语一致性检查
    term_consistency_ratio: Threshold,
    #[serde(default)]
    enablement_min_words: Threshold,
    #[serde(default)]
    background_min_chars: Threshold,
}

#[derive(Debug, Deserialize, Default)]
struct Threshold {
    #[serde(default)]
    value: f64,
}

impl Threshold {
    fn as_usize(&self) -> usize {
        self.value as usize
    }
}

/// 返回静态定义的模糊词汇列表，如"大约""左右""基本上"等。
///
/// 这些词汇在权利要求中使用时可能影响清晰性判断。
pub fn vague_words() -> Vec<&'static str> {
    vec!["大约", "左右", "基本上", "适当", "一定", "某种"]
}

/// 从 YAML 配置加载商业性用语（如"最佳""世界领先"等）。
///
/// 这些用语在专利文件中应当避免使用，以免引起夸大宣传嫌疑。
pub fn commercial_terms() -> Vec<String> {
    load_config().keyword_lists.commercial_terms.items.clone()
}

/// 从 YAML 配置加载不确定用语（如"大约""左右"等）。
///
/// 这些用语可能导致权利要求保护范围不清晰。
pub fn uncertain_terms() -> Vec<String> {
    load_config().keyword_lists.uncertain_terms.items.clone()
}

/// 从 YAML 配置加载模糊范围用语（如"以上""以下"等）。
///
/// 这些用语在数值范围限定中可能导致保护边界不明确。
pub fn vague_range_terms() -> Vec<String> {
    load_config().keyword_lists.vague_range_terms.items.clone()
}

/// 从 YAML 配置加载模糊动作用语（如"适当调整"等）。
///
/// 这些用语在描述技术方案时可能导致可实施性不足。
pub fn fuzzy_action_terms() -> Vec<String> {
    load_config().keyword_lists.fuzzy_action_terms.items.clone()
}

/// 获取禁止引用的正则模式字符串，用于检测说明书中对权利要求的直接引用。
pub fn prohibited_reference_regex() -> String {
    load_config().patterns.prohibited_references.regex.clone()
}

/// 获取可实施性评估中每项权利要求的最少字数阈值。
pub fn enablement_min_words() -> usize {
    load_config().thresholds.enablement_min_words.as_usize()
}

/// 获取背景技术部分的最少字符数阈值。
pub fn background_min_chars() -> usize {
    load_config().thresholds.background_min_chars.as_usize()
}

/// 合并所有需要检查的质量相关词汇
pub fn all_quality_terms() -> Vec<String> {
    let config = load_config();
    let mut terms = Vec::new();
    terms.extend(config.keyword_lists.commercial_terms.items.clone());
    terms.extend(config.keyword_lists.uncertain_terms.items.clone());
    terms.extend(config.keyword_lists.vague_range_terms.items.clone());
    terms.extend(config.keyword_lists.fuzzy_action_terms.items.clone());
    terms.extend(vague_words().into_iter().map(|s| s.to_string()));
    terms
}

/// 法言法语禁止用语（用于 legal_language_checker）
pub fn forbidden_terms() -> Vec<&'static str> {
    vec!["最好", "最佳", "最先进", "世界领先", "国际领先", "独一无二"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_parses_successfully() {
        let config: SpecQualityConfig = serde_yaml::from_str(RULES_YAML).unwrap();
        assert!(!config.keyword_lists.commercial_terms.items.is_empty());
        assert!(!config.keyword_lists.uncertain_terms.items.is_empty());
        assert!(!config.keyword_lists.vague_range_terms.items.is_empty());
        assert!(!config.keyword_lists.fuzzy_action_terms.items.is_empty());
    }

    #[test]
    fn test_commercial_terms_contains_expected() {
        let terms = commercial_terms();
        assert!(terms.contains(&"最佳".to_string()));
        assert!(terms.contains(&"世界领先".to_string()));
    }

    #[test]
    fn test_prohibited_reference_regex_contains_pattern() {
        let re = prohibited_reference_regex();
        assert!(re.contains("权利要求"));
    }

    #[test]
    fn test_vague_words_static() {
        let words = vague_words();
        assert!(words.contains(&"大约"));
        assert!(words.contains(&"左右"));
    }

    #[test]
    fn test_all_quality_terms_includes_all_categories() {
        let all = all_quality_terms();
        assert!(all.contains(&"最佳".to_string()));
        assert!(all.contains(&"厚".to_string()));
        assert!(all.contains(&"例如".to_string()));
        assert!(all.contains(&"适当调整".to_string()));
    }

    #[test]
    fn test_enablement_min_words_from_threshold() {
        let min = enablement_min_words();
        assert_eq!(min, 300);
    }
}
