use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// YunPat Agent 默认用户数据目录名（与官方 Codex 的 `.codex` 隔离）。
pub const BCIP_HOME_DIR_NAME: &str = ".bcip";

/// BCIP 首次启动时写入的 `config.toml`（不含插件/MCP，避免与 Codex 桌面混用）。
const DEFAULT_CONFIG_TOML: &str = r#"# YunPat Agent 专用配置（~/.bcip）
# 与本地 Codex 桌面版（~/.codex）完全隔离。按需修改 model / env_key。

model_provider = "DeepSeek"
model = "deepseek-v4-pro"
model_reasoning_effort = "medium"
disable_response_storage = true
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.DeepSeek]
name = "DeepSeek"
wire_api = "chat"
base_url = "https://api.deepseek.com/v1/"
env_key = "DEEPSEEK_API_KEY"

[features]
js_repl = false

[desktop]
auto_connect = true
"#;

/// Returns the path to the BCIP configuration directory.
///
/// Resolution order:
/// 1. `BCIP_HOME` — BCIP 专用环境变量
/// 2. `CODEX_HOME` — 兼容测试与高级覆盖（须为已存在目录）
/// 3. `~/.bcip` — 默认，与官方 Codex `~/.codex` 隔离
pub fn find_codex_home() -> io::Result<AbsolutePathBuf> {
    let bcip_home_env = std::env::var("BCIP_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    if bcip_home_env.is_some() {
        return find_codex_home_from_env(bcip_home_env.as_deref());
    }

    let codex_home_env = std::env::var("CODEX_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_codex_home_from_env(codex_home_env.as_deref())
}

fn find_codex_home_from_env(codex_home_env: Option<&str>) -> io::Result<AbsolutePathBuf> {
    match codex_home_env {
        Some(val) => {
            let path = PathBuf::from(val);
            let metadata = fs::metadata(&path).map_err(|err| match err.kind() {
                io::ErrorKind::NotFound => io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("CODEX_HOME points to {val:?}, but that path does not exist"),
                ),
                _ => io::Error::new(
                    err.kind(),
                    format!("failed to read CODEX_HOME {val:?}: {err}"),
                ),
            })?;

            if !metadata.is_dir() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("CODEX_HOME points to {val:?}, but that path is not a directory"),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    io::Error::new(
                        err.kind(),
                        format!("failed to canonicalize CODEX_HOME {val:?}: {err}"),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => default_bcip_home(),
    }
}

/// Default BCIP home: `~/.bcip`.
pub fn default_bcip_home() -> io::Result<AbsolutePathBuf> {
    let mut path = home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find home directory"))?;
    path.push(BCIP_HOME_DIR_NAME);
    AbsolutePathBuf::from_absolute_path(path)
}

/// Create `~/.bcip` layout and seed `config.toml` when missing.
pub fn ensure_bcip_home_layout(home: &Path) -> io::Result<()> {
    fs::create_dir_all(home.join("skills"))?;
    let config_path = home.join("config.toml");
    if !config_path.exists() {
        fs::write(config_path, DEFAULT_CONFIG_TOML)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::BCIP_HOME_DIR_NAME;
    use super::default_bcip_home;
    use super::ensure_bcip_home_layout;
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;

    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-bcip-home");
        let missing_str = missing
            .to_str()
            .expect("missing bcip home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(missing_str)).expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("bcip-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file bcip home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(file_str)).expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp bcip home path should be valid utf-8");

        let resolved = find_codex_home_from_env(Some(temp_str)).expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_bcip_default() {
        let resolved =
            find_codex_home_from_env(/*codex_home_env*/ None).expect("default BCIP home");
        let expected = default_bcip_home().expect("default bcip home path");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn default_bcip_home_uses_bcip_dir_name() {
        let resolved = default_bcip_home().expect("default bcip home");
        assert!(
            resolved.as_path().ends_with(BCIP_HOME_DIR_NAME),
            "expected ~/.bcip, got {}",
            resolved.display()
        );
    }

    #[test]
    fn ensure_bcip_home_layout_seeds_config_once() {
        let temp_home = TempDir::new().expect("temp home");
        ensure_bcip_home_layout(temp_home.path()).expect("seed layout");
        let config_path = temp_home.path().join("config.toml");
        assert!(config_path.exists());
        let first = fs::read_to_string(&config_path).expect("read config");
        assert!(first.contains("YunPat Agent"));

        fs::write(&config_path, "custom = true\n").expect("overwrite config");
        ensure_bcip_home_layout(temp_home.path()).expect("seed again");
        let second = fs::read_to_string(&config_path).expect("read config again");
        assert_eq!(second, "custom = true\n");
    }
}
