use crate::config::{read_bcip_config, get_bcip_project_path, write_bcip_config, PartialBcipConfig};

#[derive(serde::Serialize)]
pub struct ConfigResult {
    pub config: crate::config::BcipConfig,
    pub path: String,
}

#[tauri::command]
pub fn read_config() -> Result<ConfigResult, String> {
    let config = read_bcip_config()?;
    let path = crate::config::get_config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "~/.config/bcip/config.toml".to_string());
    
    Ok(ConfigResult { config, path })
}

#[tauri::command]
pub fn get_bcip_path() -> Result<String, String> {
    let path = get_bcip_project_path()?;
    Ok(path.to_string_lossy().to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexHomeInfo {
    pub codex_home: String,
    pub config_toml: String,
}

/// 前端可持久化的部分配置字段。
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WriteConfigParams {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub model_provider: Option<String>,
}

#[tauri::command]
pub fn write_config(params: WriteConfigParams) -> Result<(), String> {
    let partial = PartialBcipConfig {
        api_key: params.api_key,
        model: params.model,
        model_provider: params.model_provider,
    };
    write_bcip_config(partial)
}

#[tauri::command]
pub fn get_codex_home_info() -> Result<CodexHomeInfo, String> {
    let home = crate::config::find_codex_home()?;
    let config = crate::config::codex_config_toml_path()?;
    Ok(CodexHomeInfo {
        codex_home: home.to_string_lossy().to_string(),
        config_toml: config.to_string_lossy().to_string(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmlxStatus {
    pub available: bool,
    pub url: Option<String>,
    pub api_key_configured: bool,
}

/// 检测 oMLX 语义嵌入服务是否已安装。
#[tauri::command]
pub fn check_omlx_installed() -> OmlxStatus {
    let settings = crate::config::read_omlx_settings();

    match settings {
        Some(s) => OmlxStatus {
            available: true,
            url: Some(s.url),
            api_key_configured: s.api_key.map_or(false, |k| !k.trim().is_empty()),
        },
        None => OmlxStatus {
            available: false,
            url: None,
            api_key_configured: false,
        },
    }
}