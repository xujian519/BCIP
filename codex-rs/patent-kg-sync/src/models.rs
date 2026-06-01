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
