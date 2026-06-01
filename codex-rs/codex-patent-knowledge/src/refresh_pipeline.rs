use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub path: String,
    pub size: u64,
    pub modified_secs: i64,
    pub last_indexed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbVersion {
    pub updated_at: String,
    pub files: HashMap<String, FileSnapshot>,
}

#[derive(Debug)]
pub struct RefreshResult {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub deleted: Vec<String>,
    pub unchanged: usize,
}

pub struct RefreshPipeline {
    kb_root: String,
    version_file: String,
}

impl RefreshPipeline {
    pub fn new(kb_root: &str, version_file: &str) -> Self {
        Self {
            kb_root: kb_root.to_string(),
            version_file: version_file.to_string(),
        }
    }

    pub fn load_version(&self) -> Result<KbVersion, String> {
        if Path::new(&self.version_file).exists() {
            let content = std::fs::read_to_string(&self.version_file)
                .map_err(|e| format!("read version file: {e}"))?;
            serde_json::from_str(&content).map_err(|e| format!("parse version: {e}"))
        } else {
            Ok(KbVersion {
                updated_at: String::new(),
                files: HashMap::new(),
            })
        }
    }

    pub fn detect_changes(&self) -> Result<RefreshResult, String> {
        let old = self.load_version()?;
        let mut current: HashMap<String, FileSnapshot> = HashMap::new();
        let mut added = Vec::new();
        let mut updated = Vec::new();
        let mut unchanged = 0;

        fn scan(
            dir: &Path,
            root: &Path,
            files: &mut HashMap<String, FileSnapshot>,
        ) -> Result<(), String> {
            for entry in std::fs::read_dir(dir).map_err(|e| format!("{e}"))? {
                let entry = entry.map_err(|e| format!("{e}"))?;
                let path = entry.path();
                if path.is_dir() && path.file_name().map(|n| n != ".git").unwrap_or(true) {
                    scan(&path, root, files)?;
                } else if path.is_file()
                    && path
                        .extension()
                        .is_some_and(|e| e == "md" || e == "db" || e == "json" || e == "toml")
                    && let Ok(meta) = std::fs::metadata(&path)
                {
                    let rel = path
                        .strip_prefix(root)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string());
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    files.insert(
                        rel,
                        FileSnapshot {
                            path: String::new(),
                            size: meta.len(),
                            modified_secs: modified,
                            last_indexed: String::new(),
                        },
                    );
                }
            }
            Ok(())
        }

        scan(
            Path::new(&self.kb_root),
            Path::new(&self.kb_root),
            &mut current,
        )?;

        for (path, snapshot) in &current {
            match old.files.get(path) {
                Some(old_snapshot)
                    if old_snapshot.size != snapshot.size
                        || old_snapshot.modified_secs != snapshot.modified_secs =>
                {
                    updated.push(path.clone())
                }
                Some(_) => unchanged += 1,
                None => added.push(path.clone()),
            }
        }

        let deleted: Vec<_> = old
            .files
            .keys()
            .filter(|p| !current.contains_key(*p))
            .cloned()
            .collect();

        Ok(RefreshResult {
            added,
            updated,
            deleted,
            unchanged,
        })
    }

    pub fn status_json(&self) -> Result<serde_json::Value, String> {
        let changes = self.detect_changes()?;
        Ok(serde_json::json!({
            "added": changes.added.len(),
            "updated": changes.updated.len(),
            "deleted": changes.deleted.len(),
            "unchanged": changes.unchanged,
            "added_files": changes.added,
            "updated_files": changes.updated,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_no_version_file() {
        let tmp = std::env::temp_dir().join("kb_refresh_test");
        let _ = std::fs::create_dir_all(&tmp);
        let pipeline = RefreshPipeline::new(
            tmp.to_str().unwrap(),
            tmp.join(".kb-version.json").to_str().unwrap(),
        );
        let version = pipeline.load_version().unwrap();
        assert!(version.files.is_empty());
    }

    #[test]
    #[ignore = "requires local codex-patent-assets directory"]
    fn test_detect_changes_on_real_kb() {
        let pipeline = RefreshPipeline::new(
            "../codex-patent-assets",
            "../codex-patent-assets/.kb-version.json",
        );
        let changes = pipeline.detect_changes().unwrap();
        let status = pipeline.status_json().unwrap();
        // 首次运行所有文件都是新增
        assert!(!changes.added.is_empty());
        assert!(status["added"].as_u64().unwrap_or(0) > 0);
    }
}
