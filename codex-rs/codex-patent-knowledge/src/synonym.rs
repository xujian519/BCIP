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
