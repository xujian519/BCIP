//! 权利要求解析器
//!
//! 解析专利权利要求文本,提取结构化信息：
//! 前序部分、过渡词、特征列表、引用关系等。

use codex_patent_core::ClaimType;
use codex_patent_core::CorrespondenceType;
use codex_patent_core::FeatureType;
use codex_patent_core::ParsedClaim;
use codex_patent_core::ParsedFeature;

/// 权利要求解析器
pub struct ClaimParser;

impl ClaimParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析权利要求文本
    pub fn parse(&self, claim_number: u32, text: &str) -> ParsedClaim {
        let text = text.trim();
        let (claim_type, dependent_from) = detect_dependency(text);
        let (preamble, transition_word, body) = split_parts(text, claim_type);
        let features = extract_features(&body);

        ParsedClaim {
            claim_number,
            claim_type,
            preamble,
            transition_word,
            body,
            features,
            dependent_from,
        }
    }

    /// 计算两个特征之间的相似度(Jaccard 系数)
    pub fn feature_similarity(a: &ParsedFeature, b: &ParsedFeature) -> f64 {
        let set_a: std::collections::HashSet<&str> = a.description.split_whitespace().collect();
        let set_b: std::collections::HashSet<&str> = b.description.split_whitespace().collect();

        let intersection = set_a.intersection(&set_b).count() as f64;
        let union = set_a.union(&set_b).count() as f64;

        if union == 0.0 {
            return 0.0;
        }
        intersection / union
    }

    /// 根据相似度值判定对应关系类型
    pub fn classify_correspondence(similarity: f64) -> CorrespondenceType {
        if similarity >= 0.9 {
            CorrespondenceType::Exact
        } else if similarity >= 0.6 {
            CorrespondenceType::Equivalent
        } else if similarity >= 0.3 {
            CorrespondenceType::Different
        } else {
            CorrespondenceType::Missing
        }
    }
}

impl Default for ClaimParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 检测是否为从属权利要求,返回类型和引用的父权利要求编号
fn detect_dependency(text: &str) -> (ClaimType, Option<u32>) {
    // 中文:根据权利要求N
    if let Some(num) = extract_reference_cn(text) {
        return (ClaimType::Dependent, Some(num));
    }
    // 英文:according to claim N / of claim N
    if let Some(num) = extract_reference_en(text) {
        return (ClaimType::Dependent, Some(num));
    }
    (ClaimType::Independent, None)
}

fn extract_reference_cn(text: &str) -> Option<u32> {
    let patterns = ["根据权利要求", "按照权利要求", "如权利要求"];
    for pat in patterns {
        if let Some(pos) = text.find(pat) {
            let rest = &text[pos + pat.len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}

fn extract_reference_en(text: &str) -> Option<u32> {
    let lower = text.to_lowercase();
    let patterns = ["according to claim ", "of claim ", "as recited in claim "];
    for pat in patterns {
        if let Some(pos) = lower.find(pat) {
            let rest = &lower[pos + pat.len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}

/// 将权利要求文本分为前序部分、过渡词、主体部分
fn split_parts(text: &str, _claim_type: ClaimType) -> (String, String, String) {
    let transitions = [
        "其特征在于",
        "其特征是",
        "characterized in that",
        "characterized by",
    ];

    for tw in transitions {
        if let Some(pos) = text.find(tw) {
            let preamble = text[..pos].trim().to_string();
            let body = text[pos + tw.len()..].trim().to_string();
            return (preamble, tw.to_string(), body);
        }
    }

    // 无明确过渡词:整段作为 body
    (String::new(), String::new(), text.to_string())
}

/// 从权利要求主体中提取技术特征
fn extract_features(body: &str) -> Vec<ParsedFeature> {
    if body.is_empty() {
        return Vec::new();
    }

    // 按「;」或「,所述」分割特征
    let mut features = Vec::new();
    let mut id_counter = 0u32;

    // 按中文分号分割
    let segments: Vec<&str> = body.split('；').collect();

    for seg in segments {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        id_counter += 1;
        features.push(ParsedFeature {
            id: format!("F{id_counter}"),
            description: seg.to_string(),
            feature_type: classify_feature(seg),
            component: extract_component(seg),
            parameters: Vec::new(),
        });
    }

    // 如果只有一段(无分号),按逗号内的「所述」关键词尝试细分
    if features.len() == 1 && body.contains("所述") {
        let sub_parts: Vec<&str> = body.split("，").collect();
        if sub_parts.len() > 1 {
            features.clear();
            id_counter = 0;
            for part in sub_parts {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                id_counter += 1;
                features.push(ParsedFeature {
                    id: format!("F{id_counter}"),
                    description: part.to_string(),
                    feature_type: classify_feature(part),
                    component: extract_component(part),
                    parameters: Vec::new(),
                });
            }
        }
    }

    features
}

fn classify_feature(text: &str) -> FeatureType {
    if text.contains("步骤") || text.contains("方法") || text.contains("执行") {
        FeatureType::Action
    } else if text.contains("条件") || text.contains("当") || text.contains("若") {
        FeatureType::Condition
    } else if text.contains("获得") || text.contains("产生") || text.contains("实现") {
        FeatureType::Result
    } else if text.contains("参数") || text.contains("阈值") || text.contains("范围") {
        FeatureType::Parameter
    } else {
        FeatureType::Element
    }
}

fn extract_component(text: &str) -> Option<String> {
    // 提取「所述X」中的 X 作为组件名
    if let Some(pos) = text.find("所述") {
        let rest = &text[pos + "所述".len()..];
        let end = rest
            .find(|c: char| !c.is_alphabetic() && !('\u{4e00}'..='\u{9fff}').contains(&c))
            .unwrap_or(rest.len());
        if end > 0 {
            return Some(rest[..end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_independent_claim() {
        let parser = ClaimParser::new();
        let text = "一种数据处理方法,其特征在于,包括以下步骤:获取输入数据;对所述输入数据进行预处理;输出处理结果。";
        let claim = parser.parse(1, text);
        assert_eq!(claim.claim_type, ClaimType::Independent);
        assert!(claim.dependent_from.is_none());
        assert_eq!(claim.transition_word, "其特征在于");
        assert!(!claim.features.is_empty());
    }

    #[test]
    fn test_parse_dependent_claim() {
        let parser = ClaimParser::new();
        let text = "根据权利要求1所述的方法,其特征在于,所述预处理包括数据清洗。";
        let claim = parser.parse(2, text);
        assert_eq!(claim.claim_type, ClaimType::Dependent);
        assert_eq!(claim.dependent_from, Some(1));
    }

    #[test]
    fn test_feature_similarity() {
        let a = ParsedFeature {
            id: "F1".into(),
            description: "获取输入数据".into(),
            feature_type: FeatureType::Action,
            component: None,
            parameters: vec![],
        };
        let b = ParsedFeature {
            id: "F1".into(),
            description: "获取输入数据".into(),
            feature_type: FeatureType::Action,
            component: None,
            parameters: vec![],
        };
        assert_eq!(ClaimParser::feature_similarity(&a, &b), 1.0);
    }

    #[test]
    fn test_classify_correspondence() {
        assert!(matches!(
            ClaimParser::classify_correspondence(0.95),
            CorrespondenceType::Exact
        ));
        assert!(matches!(
            ClaimParser::classify_correspondence(0.7),
            CorrespondenceType::Equivalent
        ));
    }
}
