//! 语义记忆存储
//!
//! 基于 VectorIndex 和 EmbeddingClient 的长期语义记忆：
//! - 存储 agent 执行的经验（成功/失败案例、反思结果）
//! - 支持按语义相似度检索历史经验
//! - 按角色和任务类型组织记忆

use std::path::Path;

use crate::embedding_client::EmbeddingClient;

/// 单条语义记忆
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    /// 关联的 agent 角色
    pub role: String,
    /// 任务类型描述
    pub task_type: String,
    /// 输入摘要
    pub input_summary: String,
    /// 输出摘要
    pub output_summary: String,
    /// 结果（成功/失败）
    pub success: bool,
    /// 质量评分 0.0-1.0
    pub quality_score: f64,
    /// 关键经验教训
    pub lessons: Vec<String>,
    /// 时间戳
    pub timestamp: String,
}

/// 语义记忆检索结果
#[derive(Debug, Clone)]
pub struct RecalledMemory {
    pub entry: MemoryEntry,
    pub relevance: f64,
}

/// 语义记忆存储：持久化 agent 经验并支持相似度检索
pub struct SemanticMemoryStore {
    storage_dir: std::path::PathBuf,
    embedding_client: Option<EmbeddingClient>,
}

impl SemanticMemoryStore {
    /// 创建或打开语义记忆存储
    pub fn new(home_dir: &Path) -> Self {
        let storage_dir = home_dir.join("semantic_memory");
        let _ = std::fs::create_dir_all(&storage_dir);
        let embedding_client = EmbeddingClient::from_env();
        Self {
            storage_dir,
            embedding_client,
        }
    }

    /// 存储一条经验记忆
    pub fn store(&self, entry: MemoryEntry) -> Result<(), String> {
        let path = self.storage_dir.join(format!("{}.json", entry.id));
        let json = serde_json::to_string_pretty(&entry)
            .map_err(|e| format!("serialize memory: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("write memory: {e}"))?;

        // 如果 embedding 服务可用，生成并缓存向量
        if let Some(ref client) = self.embedding_client {
            let text = format!("{} {} {}", entry.role, entry.task_type, entry.input_summary);
            match client.embed(&text) {
                Ok(_embedding) => {
                    // 向量已由 EmbeddingClient 内部缓存
                    tracing::debug!(id = %entry.id, "语义记忆向量已生成");
                }
                Err(e) => {
                    tracing::warn!(id = %entry.id, error = %e, "生成记忆向量失败，仅存储文本");
                }
            }
        }

        Ok(())
    }

    /// 按语义相似度检索相关记忆
    pub fn recall(
        &self,
        query: &str,
        role: Option<&str>,
        top_k: usize,
    ) -> Result<Vec<RecalledMemory>, String> {
        let all_entries = self.load_all()?;

        // 先按角色过滤
        let filtered: Vec<&MemoryEntry> = if let Some(role) = role {
            all_entries.iter().filter(|e| e.role == role).collect()
        } else {
            all_entries.iter().collect()
        };

        if filtered.is_empty() {
            return Ok(Vec::new());
        }

        // 如果 embedding 服务可用，用向量相似度排序
        if let Some(ref client) = self.embedding_client {
            match client.embed(query) {
                Ok(query_embedding) => {
                    return self.recall_by_vector(&filtered, &query_embedding, top_k);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "向量检索失败，降级为关键词匹配");
                }
            }
        }

        // 降级：基于关键词匹配
        self.recall_by_keywords(&filtered, query, top_k)
    }

    /// 获取指定角色的所有记忆
    pub fn recall_by_role(&self, role: &str) -> Result<Vec<MemoryEntry>, String> {
        let all = self.load_all()?;
        Ok(all.into_iter().filter(|e| e.role == role).collect())
    }

    /// 获取成功案例（用于正面经验学习）
    pub fn recall_successes(
        &self,
        role: &str,
        task_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        let all = self.load_all()?;
        let mut results: Vec<MemoryEntry> = all
            .into_iter()
            .filter(|e| {
                e.role == role
                    && e.success
                    && e.quality_score >= 0.7
                    && task_type.map_or(true, |tt| e.task_type.contains(tt))
            })
            .collect();
        results.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    /// 获取失败案例（用于避免重复错误）
    pub fn recall_failures(
        &self,
        role: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, String> {
        let all = self.load_all()?;
        let mut results: Vec<MemoryEntry> = all
            .into_iter()
            .filter(|e| e.role == role && !e.success)
            .collect();
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(limit);
        Ok(results)
    }

    /// 生成唯一 ID
    pub fn generate_id() -> String {
        uuid::Uuid::now_v7().to_string()
    }

    // ---- 内部方法 ----

    fn load_all(&self) -> Result<Vec<MemoryEntry>, String> {
        let mut entries = Vec::new();
        let dir = std::fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("read memory dir: {e}"))?;

        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(memory) = serde_json::from_str::<MemoryEntry>(&content) {
                        entries.push(memory);
                    }
                }
            }
        }

        Ok(entries)
    }

    fn recall_by_vector(
        &self,
        entries: &[&MemoryEntry],
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<RecalledMemory>, String> {
        let client = self.embedding_client.as_ref().ok_or("embedding client unavailable")?;

        let mut scored: Vec<(f64, &MemoryEntry)> = Vec::new();
        for entry in entries {
            let text = format!("{} {} {}", entry.role, entry.task_type, entry.input_summary);
            match client.embed(&text) {
                Ok(entry_embedding) => {
                    let score = cosine_similarity(query_embedding, &entry_embedding);
                    scored.push((score, entry));
                }
                Err(_) => {
                    scored.push((0.0, entry));
                }
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored
            .into_iter()
            .map(|(relevance, entry)| RecalledMemory {
                entry: (*entry).clone(),
                relevance,
            })
            .collect())
    }

    fn recall_by_keywords(
        &self,
        entries: &[&MemoryEntry],
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RecalledMemory>, String> {
        let query_lower = query.to_lowercase();

        // 提取查询中的关键词（空格分隔 + 中文双字gram）
        let mut query_terms: Vec<String> = query_lower
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        // 对中文文本提取 bigram
        let chars: Vec<char> = query_lower.chars().collect();
        for window in chars.windows(2) {
            let bigram: String = window.iter().collect();
            if bigram.chars().all(|c| c as u32 > 0x4E00) {
                query_terms.push(bigram);
            }
        }

        let mut scored: Vec<(f64, &MemoryEntry)> = entries
            .iter()
            .map(|entry| {
                let text = format!(
                    "{} {} {} {}",
                    entry.role,
                    entry.task_type,
                    entry.input_summary,
                    entry.lessons.join(" ")
                )
                .to_lowercase();

                let matches = query_terms.iter().filter(|t| text.contains(t.as_str())).count();
                let score = if query_terms.is_empty() {
                    0.0
                } else {
                    matches as f64 / query_terms.len() as f64
                };
                (score, *entry)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored
            .into_iter()
            .map(|(relevance, entry)| RecalledMemory {
                entry: entry.clone(),
                relevance,
            })
            .collect())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        (dot / (norm_a * norm_b)) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, SemanticMemoryStore) {
        let dir = tempfile::TempDir::new().unwrap();
        let store = SemanticMemoryStore::new(dir.path());
        (dir, store)
    }

    fn sample_entry(role: &str, task: &str, success: bool) -> MemoryEntry {
        MemoryEntry {
            id: SemanticMemoryStore::generate_id(),
            role: role.to_string(),
            task_type: task.to_string(),
            input_summary: "分析专利权利要求的新颖性".to_string(),
            output_summary: "完成分析".to_string(),
            success,
            quality_score: if success { 0.85 } else { 0.3 },
            lessons: vec!["检查独立权利要求的技术特征".to_string()],
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_store_and_recall() {
        let (_dir, store) = setup();
        let entry = sample_entry("analyzer", "novelty", true);
        store.store(entry.clone()).unwrap();

        let recalled = store.recall("新颖性分析", Some("analyzer"), 5).unwrap();
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].entry.id, entry.id);
    }

    #[test]
    fn test_recall_by_role() {
        let (_dir, store) = setup();

        store.store(sample_entry("analyzer", "novelty", true)).unwrap();
        store.store(sample_entry("writer", "drafting", true)).unwrap();
        store.store(sample_entry("analyzer", "inventive", false)).unwrap();

        let analyzer = store.recall_by_role("analyzer").unwrap();
        assert_eq!(analyzer.len(), 2);

        let writer = store.recall_by_role("writer").unwrap();
        assert_eq!(writer.len(), 1);
    }

    #[test]
    fn test_recall_successes() {
        let (_dir, store) = setup();

        store.store(sample_entry("analyzer", "novelty", true)).unwrap();
        store.store(sample_entry("analyzer", "inventive", false)).unwrap();

        let successes = store.recall_successes("analyzer", None, 10).unwrap();
        assert_eq!(successes.len(), 1);
        assert!(successes[0].success);
    }

    #[test]
    fn test_recall_failures() {
        let (_dir, store) = setup();

        store.store(sample_entry("analyzer", "novelty", true)).unwrap();
        store.store(sample_entry("analyzer", "inventive", false)).unwrap();

        let failures = store.recall_failures("analyzer", 10).unwrap();
        assert_eq!(failures.len(), 1);
        assert!(!failures[0].success);
    }

    #[test]
    fn test_keyword_recall_fallback() {
        let (_dir, store) = setup();
        let mut entry = sample_entry("analyzer", "新颖性检查", true);
        entry.input_summary = "对发明专利的权利要求进行新颖性分析".to_string();
        store.store(entry).unwrap();

        let results = store.recall("权利要求新颖性", Some("analyzer"), 5).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].relevance > 0.0);
    }
}
