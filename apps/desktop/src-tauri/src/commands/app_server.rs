use crate::app_server_manager::{AppServerManager, AppServerStatus};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn app_server_connect(
    manager: State<'_, AppServerManager>,
    app: AppHandle,
) -> Result<AppServerStatus, String> {
    manager.connect(app)
}

#[tauri::command]
pub fn app_server_disconnect(manager: State<'_, AppServerManager>) -> Result<(), String> {
    manager.disconnect()
}

#[tauri::command]
pub fn app_server_send(manager: State<'_, AppServerManager>, line: String) -> Result<(), String> {
    manager.send_line(line)
}

#[tauri::command]
pub fn app_server_status(manager: State<'_, AppServerManager>) -> AppServerStatus {
    manager.status()
}
