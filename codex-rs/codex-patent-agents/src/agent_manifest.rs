//! Agent 状态持久化
//!
//! 提供agent元数据的持久化存储和查询功能。

use codex_patent_core::PatentError;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
    static TEST_STORE_DIR: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub agent_id: String,
    pub name: String,
    pub subagent_type: String,
    pub model: String,
    pub status: String,
    pub output_file: PathBuf,
    pub manifest_file: PathBuf,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

/// 为当前线程设置测试存储目录（仅用于测试）
#[cfg(test)]
pub fn set_test_store_dir(path: PathBuf) {
    TEST_STORE_DIR.with(|cell| *cell.borrow_mut() = Some(path));
}

/// 获取 agent 存储目录
pub fn agent_store_dir() -> Result<PathBuf, PatentError> {
    let test_dir = TEST_STORE_DIR.with(|cell| cell.borrow().clone());
    if let Some(path) = test_dir {
        return Ok(path);
    }

    if let Ok(path) = std::env::var("BCIP_AGENT_STORE") {
        return Ok(PathBuf::from(path));
    }

    let cwd = std::env::current_dir()?;
    if let Some(workspace_root) = cwd.ancestors().nth(2) {
        return Ok(workspace_root.join(".bcip-agents"));
    }

    Ok(cwd.join(".bcip-agents"))
}

/// 生成唯一的 agent ID
pub fn make_agent_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("agent-{nanos}")
}

/// 获取当前时间的 ISO8601 格式字符串
pub fn iso8601_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

/// 持久化 manifest 到文件
pub fn persist_manifest(manifest: &AgentManifest) -> Result<(), PatentError> {
    std::fs::create_dir_all(
        manifest
            .manifest_file
            .parent()
            .ok_or_else(|| PatentError::Config("manifest_file has no parent".to_string()))?,
    )?;

    let json = serde_json::to_string_pretty(manifest)?;
    std::fs::write(&manifest.manifest_file, json)?;
    Ok(())
}

/// 根据 agent_id 加载 manifest
pub fn load_manifest(agent_id: &str) -> Result<AgentManifest, PatentError> {
    let store_dir = agent_store_dir()?;
    let manifest_file = store_dir.join(format!("{agent_id}.json"));

    let content = std::fs::read_to_string(&manifest_file)
        .map_err(|e| PatentError::NotFound(format!("read manifest: {e}")))?;

    serde_json::from_str(&content)
        .map_err(|e| PatentError::Serialization(format!("parse manifest: {e}")))
}

/// 列出所有 agent manifest
pub fn list_agent_manifests() -> Result<Vec<AgentManifest>, PatentError> {
    let store_dir = match agent_store_dir() {
        Ok(dir) => dir,
        Err(_) => return Ok(Vec::new()),
    };

    if !store_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&store_dir)?;

    let mut manifests = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = std::fs::read_to_string(&path).map_err(PatentError::Io)?;

        if let Ok(manifest) = serde_json::from_str::<AgentManifest>(&content) {
            manifests.push(manifest);
        }
    }

    manifests.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(manifests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_store() -> TempDir {
        let dir = TempDir::new().unwrap();
        set_test_store_dir(dir.path().to_path_buf());
        dir
    }

    #[test]
    fn test_make_agent_id() {
        let id1 = make_agent_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = make_agent_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("agent-"));
    }

    #[test]
    fn test_iso8601_now() {
        let now = iso8601_now();
        assert!(!now.is_empty());
    }

    #[test]
    fn test_persist_and_load_manifest() {
        let _temp_dir = setup_store();

        let manifest_file = _temp_dir.path().join("test-agent-123.json");
        let output_file = _temp_dir.path().join("test-agent-123.md");

        let manifest = AgentManifest {
            agent_id: "test-agent-123".to_string(),
            name: "Test Agent".to_string(),
            subagent_type: "analyzer".to_string(),
            model: "deepseek-v4-pro".to_string(),
            status: "running".to_string(),
            output_file,
            manifest_file,
            created_at: iso8601_now(),
            completed_at: None,
            error: None,
        };

        persist_manifest(&manifest).unwrap();

        let loaded = load_manifest("test-agent-123").unwrap();
        assert_eq!(loaded.agent_id, manifest.agent_id);
        assert_eq!(loaded.name, manifest.name);
        assert_eq!(loaded.status, manifest.status);
    }

    #[test]
    fn test_list_agent_manifests() {
        let _temp_dir = setup_store();

        let manifest1 = AgentManifest {
            agent_id: "agent-1".to_string(),
            name: "Agent 1".to_string(),
            subagent_type: "analyzer".to_string(),
            model: "deepseek-v4-pro".to_string(),
            status: "completed".to_string(),
            output_file: _temp_dir.path().join("agent-1.md"),
            manifest_file: _temp_dir.path().join("agent-1.json"),
            created_at: "1".to_string(),
            completed_at: Some("2".to_string()),
            error: None,
        };

        std::thread::sleep(std::time::Duration::from_millis(10));

        let manifest2 = AgentManifest {
            agent_id: "agent-2".to_string(),
            name: "Agent 2".to_string(),
            subagent_type: "writer".to_string(),
            model: "deepseek-v4-pro".to_string(),
            status: "running".to_string(),
            output_file: _temp_dir.path().join("agent-2.md"),
            manifest_file: _temp_dir.path().join("agent-2.json"),
            created_at: "2".to_string(),
            completed_at: None,
            error: None,
        };

        persist_manifest(&manifest1).unwrap();
        persist_manifest(&manifest2).unwrap();

        let manifests = list_agent_manifests().unwrap();
        assert_eq!(manifests.len(), 2);
        assert_eq!(manifests[0].agent_id, "agent-2");
        assert_eq!(manifests[1].agent_id, "agent-1");
    }
}
