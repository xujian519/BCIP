use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// 与 codex-core 一致：BCIP 默认 `~/.bcip`，与 Codex 桌面 `~/.codex` 隔离。
pub const BCIP_HOME_DIR_NAME: &str = ".bcip";

#[derive(Debug, Serialize, Deserialize)]
pub struct BcipConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub model_provider: Option<String>,
    pub app_server: Option<AppServerConfig>,
    pub assets: Option<AssetsConfig>,
    pub embedding: Option<EmbeddingConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppServerConfig {
    pub port: Option<u16>,
    pub listen: Option<String>,
}

/// 内置知识库资产目录（对应 `BCIP_ASSETS_DIR`）。
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetsConfig {
    pub dir: Option<String>,
}

/// BGE-M3 / oMLX 语义嵌入服务（对应 `BCIP_MLX_*`）。
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
}

const DEFAULT_MLX_URL: &str = "http://127.0.0.1:8766";
const DEFAULT_MLX_MODEL: &str = "bge-m3-mlx-8bit";
const DEFAULT_ASSETS_REL: &str = "codex-rs/codex-patent-assets";

/// 读取 BCIP 运行时配置（`$BCIP_HOME/config.toml`，即 `~/.bcip/config.toml`）。
pub fn read_bcip_config() -> Result<BcipConfig, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(BcipConfig {
            api_key: None,
            model: None,
            model_provider: None,
            app_server: None,
            assets: None,
            embedding: None,
        });
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;

    let config: BcipConfig = toml::from_str(&content)
        .map_err(|e| format!("无法解析配置文件: {}", e))?;

    Ok(config)
}

/// BCIP 运行时 `config.toml` 路径（与 app-server / TUI 共用）。
pub fn get_config_path() -> Result<PathBuf, String> {
    Ok(find_bcip_home()?.join("config.toml"))
}

/// 获取 BCIP 项目路径
pub fn get_bcip_project_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;

    let projects_path = home.join("projects").join("BCIP");
    if projects_path.exists() {
        return Ok(projects_path);
    }

    let bcip_path = home.join("BCIP");
    if bcip_path.exists() {
        return Ok(bcip_path);
    }

    Err("未找到 BCIP 项目目录".to_string())
}

fn default_bcip_home_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let path = home.join(BCIP_HOME_DIR_NAME);
    std::fs::create_dir_all(&path).map_err(|e| format!("无法创建 BCIP 配置目录: {e}"))?;
    Ok(path)
}

/// BCIP 专用配置目录。
///
/// 桌面端 **不继承** shell 里指向 `~/.codex` 的 `CODEX_HOME`，避免与 Codex 桌面混用。
/// 仅 `BCIP_HOME` 可覆盖默认 `~/.bcip`。
pub fn find_bcip_home() -> Result<PathBuf, String> {
    if let Ok(val) = env::var("BCIP_HOME") {
        if !val.is_empty() {
            return validate_home_dir(PathBuf::from(val));
        }
    }

    default_bcip_home_path()
}

/// 兼容旧命名；始终解析为 BCIP 专用目录。
pub fn find_codex_home() -> Result<PathBuf, String> {
    find_bcip_home()
}

fn validate_home_dir(path: PathBuf) -> Result<PathBuf, String> {
    let meta = fs::metadata(&path).map_err(|e| format!("无法读取 BCIP_HOME: {e}"))?;
    if !meta.is_dir() {
        return Err(format!("BCIP_HOME 不是目录: {}", path.display()));
    }
    path.canonicalize()
        .map_err(|e| format!("无法规范化 BCIP_HOME: {e}"))
}

pub fn codex_config_toml_path() -> Result<PathBuf, String> {
    Ok(find_bcip_home()?.join("config.toml"))
}

pub fn codex_config_toml_path_display() -> String {
    codex_config_toml_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| format!("~/{BCIP_HOME_DIR_NAME}/config.toml"))
}

/// 部分配置更新参数，仅包含前端可持久化的字段。
/// 所有字段均为 `Option`，`None` 表示不修改该字段。
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PartialBcipConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub model_provider: Option<String>,
}

/// 将部分配置合并写入 `~/.bcip/config.toml`。
///
/// 采用「读取 → 修改 → 写回」策略；如果文件不存在则先创建默认配置再合并。
/// 使用 TOML 序列化整体写入（注释会丢失，但保持格式一致性）。
pub fn write_bcip_config(partial: PartialBcipConfig) -> Result<(), String> {
    let config_path = get_config_path()?;

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;
    }

    // 如果配置文件不存在，先用默认模板创建
    if !config_path.exists() {
        fs::write(&config_path, bcip_default_config_template())
            .map_err(|e| format!("无法创建默认配置文件: {}", e))?;
    }

    // 读取现有文件内容
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;

    // 将现有内容解析为动态 TOML 表，以保留所有字段（包括我们不识别的）
    let mut table: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("无法解析配置文件: {}", e))?;

    // 合并 partial 中的非 None 字段（必须用 insert：HashMap 索引在键不存在时会 panic）
    let root = table
        .as_table_mut()
        .ok_or_else(|| "配置文件根节点不是 TOML 表".to_string())?;
    if let Some(api_key) = &partial.api_key {
        root.insert("api_key".to_string(), toml::Value::String(api_key.clone()));
    }
    if let Some(model) = &partial.model {
        root.insert("model".to_string(), toml::Value::String(model.clone()));
    }
    if let Some(model_provider) = &partial.model_provider {
        root.insert(
            "model_provider".to_string(),
            toml::Value::String(model_provider.clone()),
        );
    }

    // 序列化回 TOML 并写入
    let new_content = toml::to_string_pretty(&table)
        .map_err(|e| format!("无法序列化配置: {}", e))?;

    fs::write(&config_path, new_content)
        .map_err(|e| format!("无法写入配置文件: {}", e))?;

    Ok(())
}

/// 启动 bcip 子进程前注入 BCIP 运行时环境（配置目录、知识库资产、语义嵌入、API Key）。
///
/// API Key 注入策略：
/// 前端 OnboardingWizard 将用户输入的 API Key 写入 `config.toml` 的顶层 `api_key` 字段，
/// 但 bcip 后端通过 `model_providers.<Provider>.env_key` 声明的环境变量名（如 `DEEPSEEK_API_KEY`）
/// 从 `std::env::var` 获取密钥。因此需要在启动子进程前，将 `config.toml` 中的 `api_key`
/// 映射到对应 provider 的 `env_key` 环境变量中。
pub fn apply_bcip_runtime_env(cmd: &mut std::process::Command) {
    if let Ok(home) = find_bcip_home() {
        let home_str = path_to_env_value(&home);
        cmd.env("BCIP_HOME", &home_str);
        cmd.env("CODEX_HOME", home_str);
    }

    if let Some(assets_dir) = resolve_bcip_assets_dir() {
        set_env_if_unset(cmd, "BCIP_ASSETS_DIR", &assets_dir);
    }

    let embedding = resolve_embedding_env();
    set_env_if_unset(cmd, "BCIP_MLX_URL", &embedding.url);
    set_env_if_unset(cmd, "BCIP_MLX_MODEL", &embedding.model);
    if let Some(api_key) = embedding.api_key {
        set_env_if_unset(cmd, "BCIP_MLX_API_KEY", &api_key);
    }

    // 将 config.toml 中的 api_key 注入为对应 model_provider 的 env_key 环境变量
    inject_provider_api_key(cmd);
}

/// 兼容旧命名；等价于 [`apply_bcip_runtime_env`]。
pub fn apply_codex_home_env(cmd: &mut std::process::Command) {
    apply_bcip_runtime_env(cmd);
}

struct ResolvedEmbedding {
    url: String,
    model: String,
    api_key: Option<String>,
}

fn resolve_bcip_assets_dir() -> Option<String> {
    if let Ok(existing) = env::var("BCIP_ASSETS_DIR") {
        if !existing.trim().is_empty() {
            return Some(existing);
        }
    }

    if let Ok(config) = read_bcip_config() {
        if let Some(dir) = config.assets.and_then(|assets| assets.dir) {
            if !dir.trim().is_empty() {
                return Some(dir);
            }
        }
    }

    // Tauri .app 包中捆绑的资产:
    //   tauri.conf.json resources=["target/codex-patent-assets/"] 打包后位于
    //   Contents/Resources/target/codex-patent-assets/。
    //   同时也检查不含 /target/ 前缀的路径作为 fallback。
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let resources = exe_dir.join("../Resources");
            for rel in [
                "target/codex-patent-assets",
                "codex-patent-assets",
            ] {
                let candidate = resources.join(rel);
                if candidate.is_dir() {
                    if let Ok(canonical) = candidate.canonicalize() {
                        return Some(canonical.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }

    get_bcip_project_path().ok().map(|project| {
        project
            .join(DEFAULT_ASSETS_REL)
            .to_string_lossy()
            .into_owned()
    })
}

fn resolve_embedding_env() -> ResolvedEmbedding {
    let config = read_bcip_config().ok();
    let omlx = read_omlx_settings();

    let url = config
        .as_ref()
        .and_then(|cfg| cfg.embedding.as_ref())
        .and_then(|emb| emb.url.clone())
        .or_else(|| omlx.as_ref().map(|settings| settings.url.clone()))
        .unwrap_or_else(|| DEFAULT_MLX_URL.to_string());

    let model = config
        .as_ref()
        .and_then(|cfg| cfg.embedding.as_ref())
        .and_then(|emb| emb.model.clone())
        .unwrap_or_else(|| DEFAULT_MLX_MODEL.to_string());

    let api_key = resolve_mlx_api_key(config.as_ref().and_then(|cfg| cfg.embedding.as_ref()));

    ResolvedEmbedding {
        url,
        model,
        api_key,
    }
}

fn resolve_mlx_api_key(config_embedding: Option<&EmbeddingConfig>) -> Option<String> {
    if let Ok(existing) = env::var("BCIP_MLX_API_KEY") {
        if !existing.trim().is_empty() {
            return Some(existing);
        }
    }

    for key in ["OMLX_API_KEY", "EMBEDDING_API_KEY"] {
        if let Ok(value) = env::var(key) {
            if !value.trim().is_empty() {
                return Some(value);
            }
        }
    }

    if let Some(key) = config_embedding.and_then(|emb| emb.api_key.clone()) {
        if !key.trim().is_empty() {
            return Some(key);
        }
    }

    read_omlx_settings().and_then(|settings| settings.api_key)
}

pub struct OmlxSettingsSnapshot {
    pub url: String,
    pub api_key: Option<String>,
}

pub fn read_omlx_settings() -> Option<OmlxSettingsSnapshot> {
    let home = dirs::home_dir()?;
    let path = home.join(".omlx").join("settings.json");
    let content = fs::read_to_string(path).ok()?;
    let parsed: OmlxSettingsFile = serde_json::from_str(&content).ok()?;

    let host = parsed
        .server
        .as_ref()
        .and_then(|server| server.host.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port = parsed
        .server
        .as_ref()
        .and_then(|server| server.port)
        .unwrap_or(8766);
    let api_key = parsed
        .auth
        .as_ref()
        .and_then(|auth| auth.api_key.clone())
        .filter(|key| !key.trim().is_empty());

    Some(OmlxSettingsSnapshot {
        url: format!("http://{host}:{port}"),
        api_key,
    })
}

#[derive(Debug, Deserialize)]
struct OmlxSettingsFile {
    server: Option<OmlxServerSection>,
    auth: Option<OmlxAuthSection>,
}

#[derive(Debug, Deserialize)]
struct OmlxServerSection {
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct OmlxAuthSection {
    api_key: Option<String>,
}

fn set_env_if_unset(cmd: &mut std::process::Command, key: &str, value: &str) {
    if env::var(key).is_err() {
        cmd.env(key, value);
    }
}

/// 从 `config.toml` 读取 `api_key` 和 `model_provider`，将 api_key 注入为对应
/// `model_providers.<Provider>.env_key` 环境变量。
///
/// 流程：
/// 1. 读取 `config.toml` 中的顶层 `api_key`（前端 OnboardingWizard 写入）
/// 2. 读取 `model_provider`（如 `"DeepSeek"`）
/// 3. 查找 `model_providers.<model_provider>.env_key`（如 `"DEEPSEEK_API_KEY"`）
/// 4. 若 api_key 有效且 env_key 对应的环境变量未设置，则注入
fn inject_provider_api_key(cmd: &mut std::process::Command) {
    let config = match read_bcip_config() {
        Ok(c) => c,
        Err(_) => return,
    };

    let api_key = match config.api_key.as_deref() {
        Some(k) if !k.trim().is_empty() && k.trim() != "local" => k.trim(),
        _ => return,
    };

    let provider_name = match config.model_provider.as_deref() {
        Some(p) if !p.trim().is_empty() => p.trim(),
        _ => return,
    };

    // 从 config.toml 的 [model_providers.<name>] 段查找 env_key
    let env_key = match resolve_model_provider_env_key(provider_name) {
        Some(k) => k,
        None => return,
    };

    set_env_if_unset(cmd, &env_key, api_key);
}

/// 从 `config.toml` 的 `[model_providers.<name>]` 段读取 `env_key` 字段。
fn resolve_model_provider_env_key(provider_name: &str) -> Option<String> {
    let config_path = get_config_path().ok()?;
    let content = fs::read_to_string(&config_path).ok()?;
    let root: toml::Value = toml::from_str(&content).ok()?;

    let env_key = root
        .get("model_providers")?
        .get(provider_name)?
        .get("env_key")?
        .as_str()?
        .trim()
        .to_string();

    if env_key.is_empty() {
        return None;
    }

    Some(env_key)
}

/// 桌面首次启动写入 config.toml 的模板内容。
pub fn bcip_default_config_template() -> &'static str {
    include_str!("../../../../scripts/config/bcip-default-config.toml")
}

fn path_to_env_value(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_omlx_settings_url() {
        let json = r#"{
            "server": { "host": "127.0.0.1", "port": 8766 },
            "auth": { "api_key": "test-key" }
        }"#;
        let parsed: OmlxSettingsFile = serde_json::from_str(json).expect("parse omlx settings");
        let host = parsed
            .server
            .as_ref()
            .and_then(|server| server.host.clone())
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = parsed
            .server
            .as_ref()
            .and_then(|server| server.port)
            .unwrap_or(8766);
        assert_eq!(format!("http://{host}:{port}"), "http://127.0.0.1:8766");
        assert_eq!(
            parsed.auth.as_ref().and_then(|auth| auth.api_key.clone()),
            Some("test-key".to_string())
        );
    }

    #[test]
    fn default_embedding_model_is_bge_m3_mlx_8bit() {
        assert_eq!(DEFAULT_MLX_MODEL, "bge-m3-mlx-8bit");
        assert_eq!(DEFAULT_MLX_URL, "http://127.0.0.1:8766");
    }

    #[test]
    fn write_bcip_config_inserts_missing_top_level_keys() {
        let content = r#"model_provider = "LocalProxy"
model = "glm-5.1"
"#;
        let mut table: toml::Value = toml::from_str(content).expect("parse fixture");
        let root = table.as_table_mut().expect("root table");
        root.insert(
            "api_key".to_string(),
            toml::Value::String("local".to_string()),
        );
        assert_eq!(
            table.get("api_key").and_then(|v| v.as_str()),
            Some("local")
        );
        assert_eq!(
            table.get("model").and_then(|v| v.as_str()),
            Some("glm-5.1")
        );
    }
}
