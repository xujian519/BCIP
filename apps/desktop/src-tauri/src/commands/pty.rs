use tauri::State;
use crate::pty_manager::{PtyManager, PtySessionInfo};

#[tauri::command]
pub fn pty_spawn(
    pty_manager: State<PtyManager>,
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
) -> Result<PtySessionInfo, String> {
    pty_manager.spawn(&command, args, cwd)
}

#[tauri::command]
pub fn pty_write(
    pty_manager: State<PtyManager>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    pty_manager.write(&session_id, &data)
}

#[tauri::command]
pub fn pty_resize(
    pty_manager: State<PtyManager>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    pty_manager.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn pty_kill(
    pty_manager: State<PtyManager>,
    session_id: String,
) -> Result<(), String> {
    pty_manager.kill(&session_id)
}