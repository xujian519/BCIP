//! 工作流配置。

use serde::Deserialize;
use serde::Serialize;

/// 工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub max_retries: u32,
    pub max_parallel_agents: u32,
    pub checkpoint_dir: String,
    pub default_model: String,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            max_parallel_agents: 4,
            checkpoint_dir: ".codex-patent-workflow/checkpoints".to_string(),
            default_model: "claude-3-5-sonnet".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorkflowConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.max_parallel_agents, 4);
        assert_eq!(config.checkpoint_dir, ".codex-patent-workflow/checkpoints");
        assert_eq!(config.default_model, "claude-3-5-sonnet");
    }

    #[test]
    fn test_serialize_config() {
        let config = WorkflowConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("max_retries"));
        assert!(json.contains("max_parallel_agents"));
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{
            "max_retries": 5,
            "max_parallel_agents": 8,
            "checkpoint_dir": "/tmp/checkpoints",
            "default_model": "gpt-4"
        }"#;
        let config: WorkflowConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.max_parallel_agents, 8);
        assert_eq!(config.checkpoint_dir, "/tmp/checkpoints");
        assert_eq!(config.default_model, "gpt-4");
    }
}
