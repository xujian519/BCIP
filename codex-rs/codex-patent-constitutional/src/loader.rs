use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use crate::model::ConstitutionalRules;

#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("failed to read rules file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse YAML rules: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("rules directory not found: {0}")]
    DirNotFound(String),
}

pub struct RuleLoader;

impl RuleLoader {
    pub fn load_rules_from(
        paths: &[PathBuf],
    ) -> Result<HashMap<String, ConstitutionalRules>, LoaderError> {
        let mut all_rules = HashMap::new();
        for path in paths {
            if path.is_dir() {
                let dir_rules = Self::load_dir(path)?;
                all_rules.extend(dir_rules);
            } else if path.is_file() {
                let file_rules = Self::load_file(path)?;
                let key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                all_rules.insert(key, file_rules);
            }
        }
        Ok(all_rules)
    }

    pub fn load_dir(dir: &Path) -> Result<HashMap<String, ConstitutionalRules>, LoaderError> {
        if !dir.is_dir() {
            return Err(LoaderError::DirNotFound(dir.display().to_string()));
        }
        let mut rules_map = HashMap::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                let file_rules = Self::load_file(&path)?;
                let key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                rules_map.insert(key, file_rules);
            }
        }
        Ok(rules_map)
    }

    pub fn load_file(path: &Path) -> Result<ConstitutionalRules, LoaderError> {
        let content = std::fs::read_to_string(path)?;
        let rules: ConstitutionalRules = serde_yaml::from_str(&content)?;
        Ok(rules)
    }
}
