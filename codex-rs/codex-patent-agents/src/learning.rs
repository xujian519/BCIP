//! Agent 学习闭环模块
//!
//! 记录 Agent 调用反馈，聚合统计，推荐最优模型。

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use codex_patent_core::PatentError;
use serde::Deserialize;
use serde::Serialize;

use crate::agent_manifest::AgentManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackData {
    pub agent_id: String,
    pub role_id: String,
    pub model: String,
    pub provider: String,
    pub success: bool,
    pub latency_ms: u64,
    pub token_count: Option<u32>,
    pub quality_score: Option<f64>,
    pub error_category: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningStats {
    pub total_calls: u32,
    pub success_count: u32,
    pub total_latency_ms: u64,
    pub avg_quality_score: f64,
    pub last_error: Option<String>,
}

impl LearningStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.total_calls as f64
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        self.total_latency_ms as f64 / self.total_calls as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GroupBy {
    Role,
    Model,
    Provider,
}

pub struct LearningStore {
    feedback_dir: PathBuf,
}

impl LearningStore {
    pub fn new(home_dir: &Path) -> Self {
        let feedback_dir = home_dir.join("learning");
        let _ = std::fs::create_dir_all(&feedback_dir);
        Self { feedback_dir }
    }

    pub fn record_feedback(&self, data: FeedbackData) -> Result<(), PatentError> {
        let path = self.feedback_dir.join("feedback.jsonl");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .map_err(|e| PatentError::Learning(format!("open feedback file: {e}")))?;
        let line = serde_json::to_string(&data)?;
        use std::io::Write;
        writeln!(file, "{line}").map_err(|e| PatentError::Learning(format!("write feedback: {e}")))
    }

    pub fn load_all(&self) -> Result<Vec<FeedbackData>, PatentError> {
        let path = self.feedback_dir.join("feedback.jsonl");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| PatentError::Learning(format!("read feedback: {e}")))?;
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                serde_json::from_str(line)
                    .map_err(|e| PatentError::Learning(format!("parse feedback: {e}")))
            })
            .collect()
    }

    pub fn get_stats(
        &self,
        group_by: GroupBy,
    ) -> Result<HashMap<String, LearningStats>, PatentError> {
        let records = self.load_all()?;
        let mut stats: HashMap<String, LearningStats> = HashMap::new();

        for rec in &records {
            let key = match group_by {
                GroupBy::Role => &rec.role_id,
                GroupBy::Model => &rec.model,
                GroupBy::Provider => &rec.provider,
            };
            let entry = stats.entry(key.clone()).or_default();
            entry.total_calls += 1;
            if rec.success {
                entry.success_count += 1;
            }
            entry.total_latency_ms += rec.latency_ms;
            if let Some(qs) = rec.quality_score {
                let n = entry.total_calls;
                entry.avg_quality_score =
                    entry.avg_quality_score * (n - 1) as f64 / n as f64 + qs / n as f64;
            }
            if !rec.success
                && let Some(ref err) = rec.error_category
            {
                entry.last_error = Some(err.clone());
            }
        }

        Ok(stats)
    }

    pub fn get_recent_failures(&self, limit: usize) -> Result<Vec<FeedbackData>, PatentError> {
        let mut records = self.load_all()?;
        records.retain(|r| !r.success);
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.truncate(limit);
        Ok(records)
    }

    pub fn suggest_model(&self, role_id: &str) -> Option<String> {
        let records = self.load_all().ok()?;
        let mut model_stats: HashMap<String, (u32, u32, u64)> = HashMap::new();

        for rec in &records {
            if rec.role_id != role_id {
                continue;
            }
            let entry = model_stats.entry(rec.model.clone()).or_insert((0, 0, 0));
            entry.0 += 1;
            if rec.success {
                entry.1 += 1;
            }
            entry.2 += rec.latency_ms;
        }

        model_stats
            .into_iter()
            .filter(|(_, (total, _, _))| *total >= 2)
            .max_by(|a, b| {
                let rate_a = a.1.1 as f64 / a.1.0 as f64;
                let rate_b = b.1.1 as f64 / b.1.0 as f64;
                rate_a
                    .partial_cmp(&rate_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(model, _)| model)
    }
}

fn provider_name(model: &str) -> String {
    let lower = model.to_ascii_lowercase();
    if lower.contains("deepseek") || lower.starts_with("ds-") {
        "deepseek".to_string()
    } else if lower.contains("qwen") {
        "qwen".to_string()
    } else if lower.contains("glm") || lower.contains("chatglm") {
        "glm".to_string()
    } else if lower.contains("claude") || lower.contains("opus") || lower.contains("sonnet") {
        "anthropic".to_string()
    } else if lower.contains("gpt-") || lower.starts_with("o1-") || lower.starts_with("o3-") {
        "openai".to_string()
    } else {
        "unknown".to_string()
    }
}

fn bcip_home() -> PathBuf {
    std::env::var("BCIP_HOME")
        .or_else(|_| std::env::var("CODEX_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/"))
                .join(".bcip")
        })
}

pub fn record_agent_feedback(
    manifest: &AgentManifest,
    latency_ms: u64,
    success: bool,
    error: Option<&str>,
) {
    let store = LearningStore::new(&bcip_home());
    let data = FeedbackData {
        agent_id: manifest.agent_id.clone(),
        role_id: manifest.subagent_type.clone(),
        model: manifest.model.clone(),
        provider: provider_name(&manifest.model),
        success,
        latency_ms,
        token_count: None,
        quality_score: None,
        error_category: error.map(|s| s.to_string()),
        timestamp: manifest.created_at.clone(),
    };
    if let Err(e) = store.record_feedback(data) {
        tracing::warn!("[bcip-learning] failed to record feedback: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, LearningStore) {
        let dir = TempDir::new().unwrap();
        let store = LearningStore::new(dir.path());
        (dir, store)
    }

    #[test]
    fn test_record_and_read_feedback() {
        let (_dir, store) = setup();

        let data = FeedbackData {
            agent_id: "test-1".to_string(),
            role_id: "analyzer".to_string(),
            model: "deepseek-v4-pro".to_string(),
            provider: "deepseek".to_string(),
            success: true,
            latency_ms: 1500,
            token_count: Some(500),
            quality_score: Some(0.85),
            error_category: None,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };

        store.record_feedback(data.clone()).unwrap();

        let records = store.load_all().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].agent_id, "test-1");
        assert!(records[0].success);
    }

    #[test]
    fn test_compute_stats() {
        let (_dir, store) = setup();

        for i in 0..5 {
            let data = FeedbackData {
                agent_id: format!("test-{i}"),
                role_id: "analyzer".to_string(),
                model: "deepseek-v4-pro".to_string(),
                provider: "deepseek".to_string(),
                success: i < 4,
                latency_ms: 1000 + i as u64 * 100,
                token_count: None,
                quality_score: Some(0.8),
                error_category: if i >= 4 {
                    Some("timeout".to_string())
                } else {
                    None
                },
                timestamp: format!("2026-01-01T00:0{i}:00Z"),
            };
            store.record_feedback(data).unwrap();
        }

        let stats = store.get_stats(GroupBy::Role).unwrap();
        let analyzer = stats.get("analyzer").unwrap();
        assert_eq!(analyzer.total_calls, 5);
        assert_eq!(analyzer.success_count, 4);
        assert!((analyzer.success_rate() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_suggest_model() {
        let (_dir, store) = setup();

        for i in 0..3 {
            store
                .record_feedback(FeedbackData {
                    agent_id: format!("a-{i}"),
                    role_id: "analyzer".to_string(),
                    model: "model-good".to_string(),
                    provider: "test".to_string(),
                    success: true,
                    latency_ms: 500,
                    token_count: None,
                    quality_score: None,
                    error_category: None,
                    timestamp: format!("2026-01-01T00:0{i}:00Z"),
                })
                .unwrap();
        }

        for i in 0..3 {
            store
                .record_feedback(FeedbackData {
                    agent_id: format!("b-{i}"),
                    role_id: "analyzer".to_string(),
                    model: "model-bad".to_string(),
                    provider: "test".to_string(),
                    success: false,
                    latency_ms: 2000,
                    token_count: None,
                    quality_score: None,
                    error_category: Some("error".to_string()),
                    timestamp: format!("2026-01-01T00:1{i}:00Z"),
                })
                .unwrap();
        }

        let suggested = store.suggest_model("analyzer");
        assert_eq!(suggested.as_deref(), Some("model-good"));
    }
}
