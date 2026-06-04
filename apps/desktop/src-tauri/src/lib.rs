pub mod commands;
pub mod pty_manager;
pub mod app_server_manager;
pub mod bcip_binary;
pub mod config;

use pty_manager::PtyManager;
use app_server_manager::AppServerManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::new().build())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .setup(|app| {
      if let Ok(home) = config::find_bcip_home() {
        let _ = std::fs::create_dir_all(&home);
        let _ = std::fs::create_dir_all(home.join("skills"));
        let config_path = home.join("config.toml");
        if !config_path.exists() {
          let _ = std::fs::write(config_path, config::bcip_default_config_template());
        }
      }

      // 初始化 PTY 管理器
      let pty_manager = PtyManager::new(app.handle().clone());
      app.manage(pty_manager);

      // app-server 单例（stdio JSONL）；由前端在就绪时调用 app_server_connect
      app.manage(AppServerManager::new());

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::fs::read_dir,
      commands::fs::read_file,
      commands::fs::read_file_binary,
      commands::fs::write_file,
      commands::fs::write_file_binary,
      commands::fs::create_dir,
      commands::fs::delete_file,
      commands::fs::get_file_info,
      commands::pty::pty_spawn,
      commands::pty::pty_write,
      commands::pty::pty_resize,
      commands::pty::pty_kill,
      commands::system::check_bcip_installed,
      commands::system::start_app_server,
      commands::system::stop_app_server,
      commands::system::get_app_server_url,
      commands::system::get_app_server_status,
      commands::system::is_app_server_running,
      commands::system::reveal_path_in_file_manager,
      commands::app_server::app_server_connect,
      commands::app_server::app_server_disconnect,
      commands::app_server::app_server_send,
      commands::app_server::app_server_status,
      commands::config::read_config,
      commands::config::write_config,
      commands::config::get_bcip_path,
      commands::config::get_codex_home_info,
      commands::config::check_omlx_installed,
      commands::doc_convert::convert_doc_to_docx,
      commands::doc_convert::libreoffice_status,
      commands::project::project_create,
      commands::project::project_list,
      commands::project::project_get_workspace,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}