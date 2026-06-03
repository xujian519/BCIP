use serde::Deserialize;

const RULES_YAML: &str = include_str!("../../codex-patent-assets/rules/spec-quality.yaml");

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
    #[allow(dead_code)]
    #[serde(default)]
    term_extraction: PatternEntry,
}

#[derive(Debug, Deserialize, Default)]
struct PatternEntry {
    #[serde(default)]
    regex: String,
}

#[derive(Debug, Deserialize, Default)]
struct Thresholds {
    #[allow(dead_code)]
    #[serde(default)]
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

fn load_config() -> SpecQualityConfig {
    serde_yaml::from_str(RULES_YAML).unwrap_or_default()
}

pub fn vague_words() -> Vec<&'static str> {
    vec!["大约", "左右", "基本上", "适当", "一定", "某种"]
}

pub fn commercial_terms() -> Vec<String> {
    load_config().keyword_lists.commercial_terms.items
}

pub fn uncertain_terms() -> Vec<String> {
    load_config().keyword_lists.uncertain_terms.items
}

pub fn vague_range_terms() -> Vec<String> {
    load_config().keyword_lists.vague_range_terms.items
}

pub fn fuzzy_action_terms() -> Vec<String> {
    load_config().keyword_lists.fuzzy_action_terms.items
}

pub fn prohibited_reference_regex() -> String {
    load_config().patterns.prohibited_references.regex
}

pub fn enablement_min_words() -> usize {
    load_config().thresholds.enablement_min_words.as_usize()
}

pub fn background_min_chars() -> usize {
    load_config().thresholds.background_min_chars.as_usize()
}

/// 合并所有需要检查的质量相关词汇
pub fn all_quality_terms() -> Vec<String> {
    let config = load_config();
    let mut terms = Vec::new();
    terms.extend(config.keyword_lists.commercial_terms.items);
    terms.extend(config.keyword_lists.uncertain_terms.items);
    terms.extend(config.keyword_lists.vague_range_terms.items);
    terms.extend(config.keyword_lists.fuzzy_action_terms.items);
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
