use serde::Deserialize;
use serde::Serialize;

// ── IPC Entry ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEntry {
    pub code: String,
    pub section: String,
    pub class: String,
    pub subclass: String,
    pub group_code: String,
    pub level: i32,
    pub parent_code: Option<String>,
    pub description: String,
    pub version: String,
    pub source_file: String,
}

// ── Invalidity Decision ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidDecision {
    pub decision_number: String,
    pub patent_number: String,
    pub conclusion: String,
    pub law_articles: Vec<String>,
    pub reasons: Vec<String>,
    pub ipc_code: Option<String>,
    pub summary: String,
    pub source_file: String,
}

// ── Judgment ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentEntry {
    pub case_number: String,
    pub court: String,
    pub date: String,
    pub cause: String,
    pub law_articles: Vec<String>,
    pub keywords: Vec<String>,
    pub key_points: String,
    pub summary: String,
    pub source_file: String,
    pub is_guiding: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_entry_roundtrip() {
        let entry = IpcEntry {
            code: "G06F3/00".into(),
            section: "G".into(),
            class: "G06".into(),
            subclass: "G06F".into(),
            group_code: "3/00".into(),
            level: 0,
            parent_code: Some("G06F".into()),
            description: "输入装置".into(),
            version: "2026.01".into(),
            source_file: "test.txt".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: IpcEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.code, entry.code);
        assert_eq!(back.level, entry.level);
    }

    #[test]
    fn invalid_decision_roundtrip() {
        let d = InvalidDecision {
            decision_number: "561695".into(),
            patent_number: "202010123456.7".into(),
            conclusion: "宣告专利权全部无效".into(),
            law_articles: vec!["A22.3".into()],
            reasons: vec!["创造性".into()],
            ipc_code: Some("G06F3/00".into()),
            summary: "test".into(),
            source_file: "test.md".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: InvalidDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back.decision_number, d.decision_number);
        assert_eq!(back.law_articles.len(), 1);
    }

    #[test]
    fn judgment_entry_roundtrip() {
        let j = JudgmentEntry {
            case_number: "(2023)最高法知行终475号".into(),
            court: "最高人民法院".into(),
            date: "2023年6月15日".into(),
            cause: "专利侵权".into(),
            law_articles: vec!["A22".into()],
            keywords: vec!["创造性".into()],
            key_points: "要点".into(),
            summary: "摘要".into(),
            source_file: "test.md".into(),
            is_guiding: true,
        };
        let json = serde_json::to_string(&j).unwrap();
        let back: JudgmentEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.case_number, j.case_number);
        assert!(back.is_guiding);
    }
}
