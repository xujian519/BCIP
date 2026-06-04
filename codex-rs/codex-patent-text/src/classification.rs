use std::collections::HashMap;

/// IPC 分类结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpcResult {
    pub section: String,
    pub class: String,
    pub subclass: String,
    pub group: String,
    pub description: String,
    pub score: f64,
}

/// 基于关键词匹配的 IPC 部级分类器。
pub struct IpcClassifier {
    section_keywords: HashMap<String, Vec<String>>,
}

impl Default for IpcClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcClassifier {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(
            "A".into(),
            vec![
                "农业".into(),
                "食品".into(),
                "服装".into(),
                "医药".into(),
                "卫生".into(),
                "生活".into(),
                "家具".into(),
                "运动".into(),
            ],
        );
        m.insert(
            "B".into(),
            vec![
                "加工".into(),
                "成型".into(),
                "印刷".into(),
                "运输".into(),
                "包装".into(),
                "分离".into(),
                "机床".into(),
                "刀具".into(),
            ],
        );
        m.insert(
            "C".into(),
            vec![
                "化学".into(),
                "冶金".into(),
                "玻璃".into(),
                "水泥".into(),
                "聚合物".into(),
                "催化剂".into(),
                "发酵".into(),
                "涂料".into(),
            ],
        );
        m.insert(
            "D".into(),
            vec![
                "纺织".into(),
                "造纸".into(),
                "纤维".into(),
                "织物".into(),
                "纱线".into(),
            ],
        );
        m.insert(
            "E".into(),
            vec![
                "建筑".into(),
                "采矿".into(),
                "道路".into(),
                "桥梁".into(),
                "锁具".into(),
                "门窗".into(),
            ],
        );
        m.insert(
            "F".into(),
            vec![
                "发动机".into(),
                "泵".into(),
                "阀".into(),
                "轴承".into(),
                "齿轮".into(),
                "照明".into(),
                "加热".into(),
                "武器".into(),
            ],
        );
        m.insert(
            "G".into(),
            vec![
                "计算".into(),
                "测量".into(),
                "信号".into(),
                "控制".into(),
                "仪器".into(),
                "导航".into(),
                "物理".into(),
            ],
        );
        m.insert(
            "H".into(),
            vec![
                "电".into(),
                "通信".into(),
                "半导体".into(),
                "电路".into(),
                "天线".into(),
                "电池".into(),
                "光电器件".into(),
            ],
        );
        Self {
            section_keywords: m,
        }
    }

    pub fn classify(&self, text: &str) -> Vec<IpcResult> {
        let text_lower = text.to_lowercase();
        let mut scores: Vec<(String, f64)> = Vec::new();

        for (section, keywords) in &self.section_keywords {
            let count = keywords
                .iter()
                .filter(|kw| text_lower.contains(*kw))
                .count();
            if count > 0 {
                scores.push((section.clone(), count as f64 / keywords.len() as f64));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores
            .into_iter()
            .map(|(section, score)| IpcResult {
                description: Self::section_desc(&section),
                section,
                score,
                class: String::new(),
                subclass: String::new(),
                group: String::new(),
            })
            .collect()
    }

    fn section_desc(section: &str) -> String {
        match section {
            "A" => "人类生活必需",
            "B" => "作业、运输",
            "C" => "化学、冶金",
            "D" => "纺织、造纸",
            "E" => "固定建筑物",
            "F" => "机械工程、照明、加热、武器、爆破",
            "G" => "物理",
            "H" => "电学",
            _ => "未知",
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_electronics() {
        let c = IpcClassifier::new();
        let r = c.classify("一种半导体通信电路装置");
        assert_eq!(r[0].section, "H");
    }

    #[test]
    fn test_classify_mechanical() {
        let c = IpcClassifier::new();
        let r = c.classify("一种新型齿轮泵的发动机轴承结构");
        assert!(r.iter().any(|x| x.section == "F"));
    }

    #[test]
    fn test_classify_chemistry() {
        let c = IpcClassifier::new();
        let r = c.classify("聚合物催化剂涂料组合物");
        assert_eq!(r[0].section, "C");
    }
}
