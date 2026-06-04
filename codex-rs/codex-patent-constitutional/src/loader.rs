use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use crate::model::ConstitutionalRules;

/// 规则加载器错误类型。
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("failed to read rules file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse YAML rules: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("rules directory not found: {0}")]
    DirNotFound(String),
}

/// 合规规则加载器。
///
/// 支持从文件或目录加载 YAML 格式的宪法规则。
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
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        // 无法从文件名获取有效的 UTF-8 字符串，使用序号
                        format!("file_{}", all_rules.len())
                    });
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
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        // 无法从文件名获取有效的 UTF-8 字符串，使用序号
                        format!("file_{}", rules_map.len())
                    });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_file_missing_returns_error() {
        let result = RuleLoader::load_file(Path::new("/nonexistent/rules.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn load_file_invalid_yaml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.yaml");
        std::fs::write(&path, "{{invalid yaml:::").unwrap();
        let result = RuleLoader::load_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn load_dir_not_found() {
        let result = RuleLoader::load_dir(Path::new("/nonexistent/dir"));
        assert!(matches!(result, Err(LoaderError::DirNotFound(_))));
    }

    #[test]
    fn load_dir_empty_returns_empty_map() {
        let dir = tempfile::tempdir().unwrap();
        let result = RuleLoader::load_dir(dir.path());
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn load_dir_skips_non_yaml_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("readme.txt"), "not yaml").unwrap();
        std::fs::write(dir.path().join("data.json"), "{}").unwrap();
        let result = RuleLoader::load_dir(dir.path());
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn load_rules_from_empty_paths() {
        let result = RuleLoader::load_rules_from(&[]);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn load_rules_from_nonexistent_path() {
        let result = RuleLoader::load_rules_from(&[PathBuf::from("/nonexistent/path")]);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn load_file_valid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("valid.yaml");
        std::fs::write(
            &path,
            r#"rules:
  test_rule:
    id: "R001"
    name: "测试规则"
    description: "描述"
    phase: "drafting"
    severity: "critical"
    action: "block"
    legal_basis: ""
    check:
      type: "keyword_blocklist"
      keywords: []
      patterns: []
      absolute_ban: []
      context_ban: []
      negation_context: false
      severity_if_found: ""
"#,
        )
        .unwrap();
        let result = RuleLoader::load_file(&path);
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert!(rules.rules.contains_key("test_rule"));
    }

    #[test]
    fn load_dir_with_yaml_files() {
        let dir = tempfile::tempdir().unwrap();
        let yaml_content = r#"rules:
  r1:
    id: "R1"
    name: "规则1"
    description: "d"
    phase: ""
    severity: "minor"
    action: "log"
    legal_basis: ""
    check:
      type: "scope_comparison"
      direction: "narrower"
"#;
        std::fs::write(dir.path().join("rules.yaml"), yaml_content).unwrap();
        std::fs::write(dir.path().join("rules2.yml"), yaml_content).unwrap();

        let result = RuleLoader::load_dir(dir.path());
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn load_rules_from_mixed_file_and_dir() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("sub");
        std::fs::create_dir_all(&subdir).unwrap();

        let yaml_content = r#"rules:
  r1:
    id: "R1"
    name: "规则1"
    description: "d"
    phase: ""
    severity: "minor"
    action: "log"
    legal_basis: ""
    check:
      type: "scope_comparison"
      direction: "narrower"
"#;
        std::fs::write(dir.path().join("file.yaml"), yaml_content).unwrap();
        std::fs::write(subdir.join("inner.yaml"), yaml_content).unwrap();

        let result = RuleLoader::load_rules_from(&[dir.path().join("file.yaml"), subdir]);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.len(), 2);
    }
}
