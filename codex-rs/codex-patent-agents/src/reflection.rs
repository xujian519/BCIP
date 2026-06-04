//! Agent 反思引擎
//!
//! Agent 完成任务后自动审查推理质量，生成改进建议。

use std::path::Path;
use std::path::PathBuf;

use codex_patent_core::PatentError;
use serde::Deserialize;
use serde::Serialize;

use crate::agent_manifest::AgentManifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub category: String,
    pub severity: IssueSeverity,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    pub agent_id: String,
    pub quality_score: f64,
    pub issues: Vec<QualityIssue>,
    pub suggestions: Vec<String>,
    pub timestamp: String,
}

pub struct ReflectionEngine {
    reflection_dir: PathBuf,
}

impl ReflectionEngine {
    pub fn new(home_dir: &Path) -> Self {
        let reflection_dir = home_dir.join("reflections");
        let _ = std::fs::create_dir_all(&reflection_dir);
        Self { reflection_dir }
    }

    pub fn reflect_on_output(&self, manifest: &AgentManifest, output: &str) -> ReflectionResult {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        check_output_length(output, &mut issues, &mut suggestions);
        check_structure(output, &mut issues, &mut suggestions);
        check_domain_keywords(
            output,
            &manifest.subagent_type,
            &mut issues,
            &mut suggestions,
        );
        check_error_indicators(output, &mut issues, &mut suggestions);

        let quality_score = compute_quality_score(&issues);

        ReflectionResult {
            agent_id: manifest.agent_id.clone(),
            quality_score,
            issues,
            suggestions,
            timestamp: crate::agent_manifest::iso8601_now(),
        }
    }

    pub fn save_reflection(&self, result: &ReflectionResult) -> Result<(), PatentError> {
        let path = self
            .reflection_dir
            .join(format!("{}.json", result.agent_id));
        let json = serde_json::to_string_pretty(result)?;
        std::fs::write(&path, json).map_err(|e| PatentError::Reflection(format!("write: {e}")))
    }

    pub fn load_reflection(&self, agent_id: &str) -> Option<ReflectionResult> {
        let path = self.reflection_dir.join(format!("{agent_id}.json"));
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn get_quality_trend(&self, limit: usize) -> Vec<ReflectionResult> {
        let Ok(entries) = std::fs::read_dir(&self.reflection_dir) else {
            return Vec::new();
        };
        let mut results: Vec<ReflectionResult> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let content = std::fs::read_to_string(e.path()).ok()?;
                serde_json::from_str(&content).ok()
            })
            .collect();
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(limit);
        results
    }
}

fn check_output_length(
    output: &str,
    issues: &mut Vec<QualityIssue>,
    _suggestions: &mut Vec<String>,
) {
    let len = output.len();
    if len < 100 {
        issues.push(QualityIssue {
            category: "completeness".to_string(),
            severity: IssueSeverity::High,
            description: format!("Output too short ({len} chars), likely incomplete"),
            suggestion: "Consider increasing max_tokens or splitting the task".to_string(),
        });
    } else if len < 500 {
        issues.push(QualityIssue {
            category: "completeness".to_string(),
            severity: IssueSeverity::Medium,
            description: format!("Output relatively short ({len} chars)"),
            suggestion: "Verify all aspects of the task were addressed".to_string(),
        });
    }
}

fn check_structure(output: &str, issues: &mut Vec<QualityIssue>, _suggestions: &mut Vec<String>) {
    let has_headers = output.contains("##") || output.contains("# ");
    let has_lists = output.contains("- ") || output.contains("* ") || output.contains("1. ");

    if !has_headers && output.len() > 300 {
        issues.push(QualityIssue {
            category: "structure".to_string(),
            severity: IssueSeverity::Low,
            description: "Long output lacks Markdown headers".to_string(),
            suggestion: "Use ## headers to organize sections".to_string(),
        });
    }

    if !has_lists && output.len() > 500 {
        issues.push(QualityIssue {
            category: "structure".to_string(),
            severity: IssueSeverity::Low,
            description: "Long output lacks list formatting".to_string(),
            suggestion: "Use bullet points for key items".to_string(),
        });
    }
}

fn check_domain_keywords(
    output: &str,
    subagent_type: &str,
    issues: &mut Vec<QualityIssue>,
    suggestions: &mut Vec<String>,
) {
    let (keywords, domain) = match subagent_type {
        "analyzer" => (
            vec!["权利要求", "技术特征", "独立权利要求", "从属权利要求"],
            "patent analysis",
        ),
        "writer" => (
            vec!["权利要求书", "说明书", "实施例", "技术领域"],
            "patent drafting",
        ),
        "retriever" => (
            vec!["检索", "对比文件", "相关度", "关键词"],
            "patent search",
        ),
        "novelty-checker" => (
            vec!["新颖性", "现有技术", "对比", "区别"],
            "novelty analysis",
        ),
        "creativity-checker" => (
            vec!["创造性", "显而易见", "技术启示", "结合"],
            "creativity analysis",
        ),
        _ => return,
    };

    let found = keywords.iter().filter(|k| output.contains(*k)).count();
    let ratio = found as f64 / keywords.len() as f64;

    if ratio < 0.25 && output.len() > 200 {
        issues.push(QualityIssue {
            category: "domain_relevance".to_string(),
            severity: IssueSeverity::Medium,
            description: format!(
                "Low domain keyword coverage ({:.0}%) for {domain}",
                ratio * 100.0
            ),
            suggestion: format!(
                "Include more domain-specific terminology: {}",
                keywords.join(", ")
            ),
        });
        suggestions.push(format!("Enhance response with {domain} domain vocabulary"));
    }
}

fn check_error_indicators(
    output: &str,
    issues: &mut Vec<QualityIssue>,
    _suggestions: &mut Vec<String>,
) {
    let error_patterns = [
        ("抱歉，我无法", "refusal indicator"),
        ("作为一个AI", "self-reference"),
        ("I cannot", "refusal indicator"),
        ("作为一名AI", "self-reference"),
        ("错误", "error word"),
    ];

    for (pattern, category) in &error_patterns {
        if output.contains(pattern) {
            issues.push(QualityIssue {
                category: category.to_string(),
                severity: IssueSeverity::Medium,
                description: format!("Output contains '{pattern}'"),
                suggestion: "Review prompt to reduce refusal/self-reference".to_string(),
            });
            break;
        }
    }
}

fn compute_quality_score(issues: &[QualityIssue]) -> f64 {
    let mut score: f64 = 1.0;
    for issue in issues {
        let penalty: f64 = match issue.severity {
            IssueSeverity::High => 0.25,
            IssueSeverity::Medium => 0.15,
            IssueSeverity::Low => 0.05,
        };
        score -= penalty;
    }
    score.max(0.0)
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

pub fn reflect_agent_result(manifest: &AgentManifest, output: &str) {
    let engine = ReflectionEngine::new(&bcip_home());
    let result = engine.reflect_on_output(manifest, output);
    if let Err(e) = engine.save_reflection(&result) {
        tracing::warn!("[bcip-reflection] failed to save: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, ReflectionEngine) {
        let dir = TempDir::new().unwrap();
        let engine = ReflectionEngine::new(dir.path());
        (dir, engine)
    }

    fn fake_manifest(subagent_type: &str) -> AgentManifest {
        AgentManifest {
            agent_id: "test-reflect-1".to_string(),
            name: "test".to_string(),
            subagent_type: subagent_type.to_string(),
            model: "test-model".to_string(),
            status: "completed".to_string(),
            output_file: std::path::PathBuf::from("/tmp/test.md"),
            manifest_file: std::path::PathBuf::from("/tmp/test.json"),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            completed_at: None,
            error: None,
        }
    }

    #[test]
    fn test_high_quality_output() {
        let (_dir, engine) = setup();
        let output = "\
## 权利要求分析

本发明涉及一种技术特征分析方法。

### 独立权利要求

- 技术特征A：实现了核心功能
- 技术特征B：提供了辅助支持

### 从属权利要求

- 权利要求2进一步限定了技术特征A的具体实现方式
- 权利要求3增加了技术特征B的优化方案

## 结论

经过分析，该权利要求书结构完整，逻辑清晰。";

        let result = engine.reflect_on_output(&fake_manifest("analyzer"), output);
        assert!(result.quality_score > 0.7, "score={}", result.quality_score);
    }

    #[test]
    fn test_short_output() {
        let (_dir, engine) = setup();
        let result = engine.reflect_on_output(&fake_manifest("analyzer"), "ok");
        assert!(result.quality_score < 0.9);
        assert!(result.issues.iter().any(|i| i.category == "completeness"));
    }

    #[test]
    fn test_save_and_load() {
        let (_dir, engine) = setup();
        let result = engine.reflect_on_output(&fake_manifest("analyzer"), "test output");
        engine.save_reflection(&result).unwrap();

        let loaded = engine.load_reflection("test-reflect-1").unwrap();
        assert_eq!(loaded.agent_id, "test-reflect-1");
        assert!((loaded.quality_score - result.quality_score).abs() < 0.01);
    }

    #[test]
    fn test_quality_trend() {
        let (_dir, engine) = setup();

        for i in 0..3 {
            let mut manifest = fake_manifest("analyzer");
            manifest.agent_id = format!("trend-{i}");
            let result = engine.reflect_on_output(&manifest, "test");
            engine.save_reflection(&result).unwrap();
        }

        let trend = engine.get_quality_trend(10);
        assert_eq!(trend.len(), 3);
    }

    #[test]
    fn test_error_indicators() {
        let (_dir, engine) = setup();
        let output = "作为一个AI语言模型，我无法提供具体的法律建议。这个话题需要专业的知识产权律师来处理。不过我可以告诉你一些一般性的信息。这是一个比较长的内容来避免触发长度检查的问题。";
        let result = engine.reflect_on_output(&fake_manifest("analyzer"), output);
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.category == "self-reference" || i.category == "refusal indicator")
        );
    }
}
