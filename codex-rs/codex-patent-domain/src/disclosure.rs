use codex_patent_core::*;
use regex::Regex;
use std::collections::HashMap;

pub struct DisclosureParser;

impl DisclosureParser {
    pub fn parse(text: &str) -> DisclosureDoc {
        let sections = Self::extract_sections(text);
        let confidence = Self::calculate_confidence(&sections, text);
        DisclosureDoc {
            raw_text: text.to_string(),
            sections,
            confidence,
        }
    }

    fn extract_sections(text: &str) -> HashMap<String, String> {
        let mut sections = HashMap::new();

        let patterns: &[(&str, &str)] = &[
            ("发明名称", r"(?:发明名称|名称)[：:]\s*(.+?)(?:\n|$)"),
            (
                "技术领域",
                r"(?s)(?:技术领域)[：:]\s*(.+?)(?=\n(?:背景技术|发明内容|现有技术)|$)",
            ),
            (
                "背景技术",
                r"(?s)(?:背景技术|现有技术)[：:]\s*(.+?)(?=\n(?:发明内容|技术问题|技术方案)|$)",
            ),
            (
                "发明内容",
                r"(?s)(?:发明内容)[：:]\s*(.+?)(?=\n(?:具体实施方式|附图说明)|$)",
            ),
            (
                "技术问题",
                r"(?s)(?:技术问题|所要解决的技术问题)[：:]\s*(.+?)(?=\n|$)",
            ),
            (
                "技术方案",
                r"(?s)(?:技术方案|技术解决方案)[：:]\s*(.+?)(?=\n|$)",
            ),
            (
                "技术效果",
                r"(?s)(?:技术效果|有益效果)[：:]\s*(.+?)(?=\n|$)",
            ),
            ("具体实施方式", r"(?s)(?:具体实施方式|实施例)[：:]\s*(.+)$"),
            (
                "附图说明",
                r"(?s)(?:附图说明)[：:]\s*(.+?)(?=\n(?:具体实施方式|发明内容)|$)",
            ),
        ];

        for (name, pattern) in patterns {
            if let Ok(re) = Regex::new(pattern)
                && let Some(cap) = re.captures(text)
                && let Some(m) = cap.get(1)
            {
                sections.insert(name.to_string(), m.as_str().trim().to_string());
                continue;
            }
            sections.insert(name.to_string(), String::new());
        }

        if sections.get("技术领域").is_none_or(|s| s.is_empty())
            && let Some(v) = Self::extract_by_patterns(
                text,
                &[r"(?:所属)?技术领域[：:][^\n]*", r"本发明涉及[^\n]*"],
            )
        {
            sections.insert("技术领域".into(), v);
        }
        if sections.get("技术问题").is_none_or(|s| s.is_empty())
            && let Some(v) = Self::extract_by_patterns(
                text,
                &[r"(?:所要解决)?技术问题[：:][^\n]*", r"为了解决[^\n]*"],
            )
        {
            sections.insert("技术问题".into(), v);
        }
        if sections.get("技术方案").is_none_or(|s| s.is_empty())
            && let Some(v) =
                Self::extract_by_patterns(text, &[r"技术方案[：:][^\n]*", r"采用[^\n]*"])
        {
            sections.insert("技术方案".into(), v);
        }

        sections
    }

    fn extract_by_patterns(text: &str, patterns: &[&str]) -> Option<String> {
        for pat in patterns {
            if let Ok(re) = Regex::new(pat) {
                let matches: Vec<&str> = re.find_iter(text).map(|m| m.as_str()).take(3).collect();
                if !matches.is_empty() {
                    return Some(matches.join(" "));
                }
            }
        }
        None
    }

    fn calculate_confidence(sections: &HashMap<String, String>, raw_text: &str) -> f32 {
        let weights: &[(&str, f32)] = &[
            ("技术领域", 0.1),
            ("背景技术", 0.1),
            ("技术问题", 0.2),
            ("技术方案", 0.3),
            ("技术效果", 0.2),
            ("具体实施方式", 0.1),
        ];
        let mut confidence: f32 = weights
            .iter()
            .filter(|(name, _)| sections.get(*name).is_some_and(|s| !s.is_empty()))
            .map(|(_, w)| w)
            .sum();
        if (100..50000).contains(&raw_text.len()) {
            confidence += 0.1;
        }
        confidence.min(1.0)
    }
}

pub struct FeatureExtractor;

impl FeatureExtractor {
    pub fn extract_features(
        text: &str,
        sections: Option<&HashMap<String, String>>,
    ) -> Vec<TechnicalFeature> {
        let solution_text = sections
            .and_then(|s| s.get("技术方案"))
            .map(String::as_str)
            .unwrap_or_else(|| Self::extract_solution_section(text));

        let mut features = Vec::new();
        features.extend(Self::extract_component_features(solution_text));
        features.extend(Self::extract_step_features(solution_text));
        features.extend(Self::extract_parameter_features(solution_text));
        Self::classify_features(&mut features);
        features
    }

    pub fn extract_problem_feature_effects(
        text: &str,
        sections: Option<&HashMap<String, String>>,
        features: Option<&[TechnicalFeature]>,
    ) -> Vec<ProblemFeatureEffect> {
        let problem = sections
            .and_then(|s| s.get("技术问题"))
            .cloned()
            .unwrap_or_else(|| Self::extract_problem_section(text).to_string());
        let effects_text = sections
            .and_then(|s| s.get("技术效果"))
            .cloned()
            .unwrap_or_else(|| Self::extract_effects_section(text).to_string());
        let effects = Self::parse_effects(&effects_text);
        let related_features: Vec<TechnicalFeature> = features
            .map(|fs| {
                fs.iter()
                    .filter(|f| f.feature_type == FeatureType::Element)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        let mut tuples = Vec::new();
        if !problem.is_empty() && !related_features.is_empty() {
            tuples.push(ProblemFeatureEffect {
                id: "PFE_1".into(),
                technical_problem: problem,
                technical_features: related_features,
                technical_effects: effects,
            });
        }
        tuples
    }

    fn extract_solution_section(text: &str) -> &str {
        let patterns = [
            r"(?s)技术方案[：:](.+?)(?=\n技术效果|\n具体实施方式|$)",
            r"(?s)采用[^\n]*?(?=\n技术效果|\n具体实施方式|$)",
        ];
        for pat in &patterns {
            if let Ok(re) = Regex::new(pat)
                && let Some(m) = re.find(text)
            {
                return m.as_str().trim();
            }
        }
        text
    }

    fn extract_problem_section(text: &str) -> &str {
        let patterns = [
            r"(?s)技术问题[：:](.+?)(?=\n技术方案|$)",
            r"(?s)所要解决的技术问题[：:](.+?)(?=\n技术方案|$)",
        ];
        for pat in &patterns {
            if let Ok(re) = Regex::new(pat)
                && let Some(m) = re.find(text)
            {
                return m.as_str().trim();
            }
        }
        ""
    }

    fn extract_effects_section(text: &str) -> &str {
        let patterns = [
            r"(?s)技术效果[：:](.+?)(?=\n具体实施方式|$)",
            r"(?s)有益效果[：:](.+?)(?=\n具体实施方式|$)",
        ];
        for pat in &patterns {
            if let Ok(re) = Regex::new(pat)
                && let Some(m) = re.find(text)
            {
                return m.as_str().trim();
            }
        }
        ""
    }

    fn extract_component_features(text: &str) -> Vec<TechnicalFeature> {
        let mut features = Vec::new();
        let patterns = [
            r"([\w\u4e00-\u9fff]{1,8})(?:层|模块|单元|部件|装置|器件)",
            r"([\w\u4e00-\u9fff]{1,8})(?:器|机|设备)",
        ];
        for pat in &patterns {
            if let Ok(re) = Regex::new(pat) {
                for cap in re.captures_iter(text) {
                    if let Some(m) = cap.get(1) {
                        let c = m.as_str().to_string();
                        features.push(TechnicalFeature {
                            id: format!("COMP_{}", features.len() + 1),
                            description: c.clone(),
                            feature_type: FeatureType::Element,
                            category: FeatureCategory::Structural,
                            component: Some(c.clone()),
                            function: Some(Self::infer_function(&c, text)),
                        });
                    }
                }
            }
        }
        features
    }

    fn extract_step_features(text: &str) -> Vec<TechnicalFeature> {
        let mut features = Vec::new();
        let patterns = [
            r"步骤[一二三四五六七八九十\d]+[：:]\s*([^\n，。；]+)",
            r"第[一二三四五六七八九十\d]+步[：:]\s*([^\n，。；]+)",
        ];
        for pat in &patterns {
            if let Ok(re) = Regex::new(pat) {
                for cap in re.captures_iter(text) {
                    if let Some(m) = cap.get(1) {
                        let desc = m.as_str().trim().to_string();
                        features.push(TechnicalFeature {
                            id: format!("STEP_{}", features.len() + 1),
                            description: desc.clone(),
                            feature_type: FeatureType::Action,
                            category: FeatureCategory::Method,
                            component: None,
                            function: Some(desc),
                        });
                    }
                }
            }
        }
        features
    }

    fn extract_parameter_features(text: &str) -> Vec<TechnicalFeature> {
        let mut features = Vec::new();
        let patterns = [
            r"([^\s，。]{2,10})(?:大小|数量|长度|宽度|厚度|重量)[：:]\s*([^\n，。]+)",
            r"([^\s，。]{2,10})为\s*([^\n，。]+?)(?:，|。|$)",
        ];
        for pat in &patterns {
            if let Ok(re) = Regex::new(pat) {
                for cap in re.captures_iter(text) {
                    if let (Some(name), Some(value)) = (cap.get(1), cap.get(2)) {
                        let n = name.as_str().to_string();
                        let v = value.as_str().to_string();
                        features.push(TechnicalFeature {
                            id: format!("PARAM_{}", features.len() + 1),
                            description: format!("{n}为{v}"),
                            feature_type: FeatureType::Parameter,
                            category: FeatureCategory::Functional,
                            component: Some(n),
                            function: Some(v),
                        });
                    }
                }
            }
        }
        features
    }

    fn classify_features(features: &mut [TechnicalFeature]) {
        let optional_kw = ["可选", "可以", "优选", "例如"];
        for f in features.iter_mut() {
            if optional_kw.iter().any(|kw| f.description.contains(kw)) {
                f.feature_type = FeatureType::Parameter;
            }
        }
    }

    fn infer_function(component: &str, context: &str) -> String {
        let pat = format!("{}[^\n]*?[，。；]", regex::escape(component));
        if let Ok(re) = Regex::new(&pat)
            && let Some(m) = re.find(context)
        {
            return m.as_str().trim().to_string();
        }
        String::new()
    }

    fn parse_effects(text: &str) -> Vec<String> {
        text.split(&['，', '。', '；', ';'][..])
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() > 5)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_disclosure() {
        let text = "\
发明名称：一种基于深度学习的图像识别方法

技术领域：
本发明涉及计算机视觉和人工智能技术领域。

背景技术：
现有图像识别方法准确率低。

技术问题：
本发明要解决如何提高图像识别准确率的技术问题。

技术方案：
本发明提供一种图像识别方法，包括输入层、卷积层和输出层。

技术效果：
本发明能够提高识别准确率。

具体实施方式：
如图1所示，本发明的图像识别方法采用深度卷积神经网络结构。";
        let doc = DisclosureParser::parse(text);
        assert!(doc.confidence > 0.5);
        assert!(!doc.sections.get("发明名称").unwrap().is_empty());
        assert!(!doc.sections.get("技术方案").unwrap().is_empty());
    }

    #[test]
    fn test_extract_features() {
        let mut sections = HashMap::new();
        sections.insert("技术方案".into(), "本发明提供一种图像识别方法，包括：输入层，用于接收图像；卷积层，用于提取特征；池化层，用于降维。其中卷积核大小为3x3。".into());
        let features = FeatureExtractor::extract_features("", Some(&sections));
        assert!(
            !features.is_empty(),
            "应提取到技术特征，actual: {features:?}"
        );
        assert!(features.iter().any(|f| f.component.is_some()));
    }

    #[test]
    fn test_extract_component_features() {
        let f = FeatureExtractor::extract_component_features(
            "包括输入层、卷积处理层、池化计算层和全连接分析层",
        );
        assert!(f.len() >= 3, "应至少提取到3个组件特征, actual: {f:?}");
    }

    #[test]
    fn test_extract_step_features() {
        let f = FeatureExtractor::extract_step_features(
            "步骤一：输入图像数据；步骤二：进行特征提取；步骤三：分类识别",
        );
        assert!(!f.is_empty(), "应提取到步骤特征");
    }
}
