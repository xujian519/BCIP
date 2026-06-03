use crate::app_server_manager::{AppServerManager, AppServerStatus};
use crate::bcip_binary::{self, BcipCheckResult};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn check_bcip_installed(app: AppHandle) -> BcipCheckResult {
    bcip_binary::check_bcip_installed(Some(&app))
}

/// 兼容旧前端：启动 stdio 传输（委托单例 `AppServerManager`）。
#[tauri::command]
pub fn start_app_server(
    manager: State<'_, AppServerManager>,
    app: AppHandle,
) -> Result<AppServerStatus, String> {
    manager.connect(app)
}

#[tauri::command]
pub fn stop_app_server(manager: State<'_, AppServerManager>) -> Result<(), String> {
    manager.disconnect()
}

#[tauri::command]
pub fn get_app_server_status(manager: State<'_, AppServerManager>) -> AppServerStatus {
    manager.status()
}

/// 已废弃 WebSocket URL；保留命令避免旧客户端崩溃。
#[tauri::command]
pub fn get_app_server_url(manager: State<'_, AppServerManager>) -> String {
    if manager.status().connected {
        "stdio://".to_string()
    } else {
        String::new()
    }
}

#[tauri::command]
pub fn is_app_server_running(manager: State<'_, AppServerManager>) -> bool {
    manager.status().connected
}

/// 在系统文件管理器中打开路径（macOS: Finder）。
#[tauri::command]
pub fn reveal_path_in_file_manager(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .status()
            .map_err(|e| format!("无法打开路径: {e}"))?;
        return Ok(());
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("当前平台暂不支持在文件管理器中打开".to_string())
    }
}
