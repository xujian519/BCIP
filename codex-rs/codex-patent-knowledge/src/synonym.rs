//! 专利领域同义词词典。
//!
//! 维护专利实务中的常用术语同义词映射（如"新颖性"→"首次公开"、"现有技术"→"prior art"），
//! 用于扩展检索关键词，提高召回率。

use serde::{Deserialize, Serialize};

/// 专利领域同义词词典。
///
/// 内置约 20 组专利高频术语的同义词映射，涵盖新颖性/创造性/侵权/无效等核心概念。
pub struct SynonymDict {
    synonyms: Vec<(&'static str, Vec<&'static str>)>,
}

impl SynonymDict {
    pub fn new() -> Self {
        Self {
            synonyms: vec![
                (
                    "新颖性",
                    vec!["首次公开", "未公开", "现有技术", "newness", "novelty"],
                ),
                (
                    "创造性",
                    vec![
                        "非显而易见",
                        "发明高度",
                        "技术启示",
                        "技术贡献",
                        "inventiveness",
                        "inventive step",
                    ],
                ),
                (
                    "侵权",
                    vec![
                        "侵害",
                        "侵犯专利权",
                        "未经许可实施",
                        "等同侵权",
                        "infringement",
                    ],
                ),
                (
                    "无效",
                    vec!["宣告无效", "专利权无效", "撤销", "无效宣告", "invalidation"],
                ),
                ("权利要求", vec!["保护范围", "权项", "claims", "专利要求"]),
                (
                    "说明书",
                    vec!["公开文本", "specification", "发明内容", "具体实施方式"],
                ),
                (
                    "现有技术",
                    vec![
                        "已知技术",
                        "已有技术",
                        "背景技术",
                        "公知常识",
                        "惯用手段",
                        "prior art",
                    ],
                ),
                (
                    "技术效果",
                    vec!["效果", "技术贡献", "进步", "有益效果", "technical effect"],
                ),
                (
                    "技术问题",
                    vec![
                        "要解决的技术问题",
                        "发明目的",
                        "技术需求",
                        "technical problem",
                    ],
                ),
                (
                    "技术方案",
                    vec!["技术手段", "解决方案", "实现方式", "technical solution"],
                ),
                ("优先权", vec!["priority", "priority right"]),
                ("公布", vec!["publication", "公开", "公开公告"]),
                ("授权", vec!["grant", "授予专利权"]),
                ("实质审查", vec!["substantive examination", "实审"]),
                ("初步审查", vec!["preliminary examination", "初审"]),
                ("驳回", vec!["rejection", "拒绝", "驳回通知"]),
                ("复审", vec!["review", "re-examination"]),
                ("异议", vec!["opposition"]),
                ("专利无效", vec!["patent invalidity", "专利权无效宣告"]),
                ("专利侵权", vec!["patent infringement", "专利侵害"]),
                (
                    "等同原则",
                    vec!["doctrine of equivalents", "等同侵权", "等价原则"],
                ),
                ("全部技术特征", vec!["all technical features", "全部特征"]),
                (
                    "区别技术特征",
                    vec!["distinguishing technical features", "区别特征"],
                ),
                (
                    "技术启示",
                    vec!["technical teaching", "teaching away", "技术教导"],
                ),
                ("公知常识", vec!["common general knowledge", "公知技术常识"]),
                ("惯用手段", vec!["conventional means", "常规手段"]),
                ("所属技术领域", vec!["technical field", "技术领域"]),
                ("背景技术", vec!["background art", "背景技术"]),
            ],
        }
    }

    pub fn expand(&self, term: &str) -> Vec<&str> {
        let mut result = Vec::new();

        for (main, syns) in &self.synonyms {
            if term.contains(main) {
                result.push(*main);
                result.extend(syns.iter().copied());
            }
            for syn in syns {
                if term.contains(syn) {
                    if !result.contains(main) {
                        result.push(*main);
                    }
                    result.extend(syns.iter().copied());
                }
            }
        }

        result.sort();
        result.dedup();
        result
    }

    pub fn search_synonyms(&self, keyword: &str) -> Vec<String> {
        let mut result = Vec::new();

        for (main, syns) in &self.synonyms {
            if main.contains(keyword) || syns.iter().any(|s| s.contains(keyword)) {
                result.push((*main).to_string());
                for syn in syns {
                    result.push((*syn).to_string());
                }
            }
        }

        result.sort();
        result.dedup();
        result
    }
}

impl Default for SynonymDict {
    fn default() -> Self {
        Self::new()
    }
}

/// 专利术语标准化器
pub struct PatentTerminologyNormalizer {
    entries: Vec<TermEntry>,
}

/// 术语条目：标准形式 + 变体 + 领域 + 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermEntry {
    pub canonical: String,
    pub variants: Vec<String>,
    pub domain: String,
    pub definition: Option<String>,
}

/// 标准化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTerm {
    pub original: String,
    pub canonical: String,
    pub confidence: f64,
    pub domain: Option<String>,
}

impl PatentTerminologyNormalizer {
    /// 从内置术语表创建
    pub fn new() -> Self {
        Self {
            entries: Self::builtin_terms(),
        }
    }

    /// 标准化一个术语
    pub fn normalize(&self, term: &str) -> NormalizedTerm {
        let term_lower = term.to_lowercase();

        for entry in &self.entries {
            if entry.canonical.to_lowercase() == term_lower {
                return NormalizedTerm {
                    original: term.to_string(),
                    canonical: entry.canonical.clone(),
                    confidence: 1.0,
                    domain: Some(entry.domain.clone()),
                };
            }
            if entry
                .variants
                .iter()
                .any(|v| v.to_lowercase() == term_lower)
            {
                return NormalizedTerm {
                    original: term.to_string(),
                    canonical: entry.canonical.clone(),
                    confidence: 0.9,
                    domain: Some(entry.domain.clone()),
                };
            }
        }

        NormalizedTerm {
            original: term.to_string(),
            canonical: term.to_string(),
            confidence: 0.5,
            domain: None,
        }
    }

    /// 对文本中的术语进行标准化
    pub fn normalize_text(&self, text: &str) -> Vec<NormalizedTerm> {
        let mut results = Vec::new();
        for entry in &self.entries {
            for variant in &entry.variants {
                if text.contains(variant) {
                    results.push(NormalizedTerm {
                        original: variant.clone(),
                        canonical: entry.canonical.clone(),
                        confidence: 0.9,
                        domain: Some(entry.domain.clone()),
                    });
                }
            }
            if text.contains(&entry.canonical) {
                results.push(NormalizedTerm {
                    original: entry.canonical.clone(),
                    canonical: entry.canonical.clone(),
                    confidence: 1.0,
                    domain: Some(entry.domain.clone()),
                });
            }
        }
        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.dedup_by(|a, b| a.original == b.original);
        results
    }

    fn builtin_terms() -> Vec<TermEntry> {
        vec![
            TermEntry {
                canonical: "权利要求".into(),
                variants: vec!["claim".into(), "权利项".into(), "专利要求".into()],
                domain: "general".into(),
                definition: Some("专利文件中界定保护范围的部分".into()),
            },
            TermEntry {
                canonical: "说明书".into(),
                variants: vec!["specification".into(), "描述".into(), "详细说明".into()],
                domain: "general".into(),
                definition: Some("专利文件中详细描述发明的部分".into()),
            },
            TermEntry {
                canonical: "现有技术".into(),
                variants: vec![
                    "prior art".into(),
                    "公知技术".into(),
                    "已有技术".into(),
                    "背景技术".into(),
                ],
                domain: "general".into(),
                definition: Some("申请日前已公开的技术".into()),
            },
            TermEntry {
                canonical: "技术特征".into(),
                variants: vec![
                    "technical feature".into(),
                    "技术特点".into(),
                    "技术手段".into(),
                ],
                domain: "general".into(),
                definition: Some("构成技术方案的最小技术单元".into()),
            },
            TermEntry {
                canonical: "创造性".into(),
                variants: vec![
                    "inventiveness".into(),
                    "非显而易见性".into(),
                    "inventive step".into(),
                    "突出的实质性特点".into(),
                ],
                domain: "general".into(),
                definition: Some("与现有技术相比具有突出的实质性特点和显著的进步".into()),
            },
            TermEntry {
                canonical: "新颖性".into(),
                variants: vec!["novelty".into(), "新颗性".into()],
                domain: "general".into(),
                definition: Some("不属于现有技术".into()),
            },
            TermEntry {
                canonical: "实施例".into(),
                variants: vec!["embodiment".into(), "实施方式".into(), "实施形态".into()],
                domain: "general".into(),
                definition: Some("实现发明的具体方式".into()),
            },
            TermEntry {
                canonical: "技术效果".into(),
                variants: vec![
                    "technical effect".into(),
                    "有益效果".into(),
                    "技术优势".into(),
                ],
                domain: "general".into(),
                definition: Some("发明带来的技术上的有益结果".into()),
            },
        ]
    }
}

impl Default for PatentTerminologyNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_novelty() {
        let dict = SynonymDict::new();
        let results = dict.expand("新颖性");
        assert!(!results.is_empty());
        assert!(results.contains(&"首次公开"));
        assert!(results.contains(&"novelty"));
    }

    #[test]
    fn expand_creativity() {
        let dict = SynonymDict::new();
        let results = dict.expand("创造性");
        assert!(results.contains(&"非显而易见"));
        assert!(results.contains(&"inventive step"));
    }

    #[test]
    fn expand_no_match() {
        let dict = SynonymDict::new();
        let results = dict.expand("量子纠缠");
        assert!(results.is_empty());
    }

    #[test]
    fn expand_by_synonym() {
        let dict = SynonymDict::new();
        let results = dict.expand("infringement");
        assert!(results.contains(&"侵权"));
    }

    #[test]
    fn search_synonyms_keyword() {
        let dict = SynonymDict::new();
        let results = dict.search_synonyms("prior");
        assert!(results.iter().any(|s| s.contains("prior art")));
    }

    #[test]
    fn search_synonyms_chinese() {
        let dict = SynonymDict::new();
        let results = dict.search_synonyms("无效");
        assert!(results.iter().any(|s| s == "宣告无效"));
    }

    #[test]
    fn default_equals_new() {
        let d1 = SynonymDict::new();
        let d2 = SynonymDict::default();
        assert_eq!(d1.expand("新颖性"), d2.expand("新颖性"));
    }

    #[test]
    fn test_normalize_known_term() {
        let normalizer = PatentTerminologyNormalizer::new();

        // Canonical form → confidence 1.0
        let result = normalizer.normalize("权利要求");
        assert_eq!(result.canonical, "权利要求");
        assert_eq!(result.confidence, 1.0);

        // English variant → confidence 0.9
        let result = normalizer.normalize("claim");
        assert_eq!(result.canonical, "权利要求");
        assert_eq!(result.confidence, 0.9);

        // Unknown term → passthrough, confidence 0.5
        let result = normalizer.normalize("量子计算");
        assert_eq!(result.canonical, "量子计算");
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn test_normalize_text_with_variants() {
        let normalizer = PatentTerminologyNormalizer::new();

        let text = "该发明相对于现有技术具有突出的实质性特点，属于实施例之一。";
        let results = normalizer.normalize_text(text);

        let canonicals: Vec<&str> = results.iter().map(|r| r.canonical.as_str()).collect();
        assert!(canonicals.contains(&"现有技术"), "should find 现有技术");
        assert!(
            canonicals.contains(&"创造性"),
            "should find 创造性 via 突出的实质性特点"
        );
        assert!(canonicals.contains(&"实施例"), "should find 实施例");

        // Exact canonical matches have confidence 1.0
        let exact: Vec<&NormalizedTerm> = results.iter().filter(|r| r.confidence == 1.0).collect();
        assert!(
            !exact.is_empty(),
            "at least one exact match with confidence 1.0"
        );
    }
}
