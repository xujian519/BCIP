//! 单例 app-server 连接：优先 `bcip app-server proxy` 附着已有 daemon，否则 spawn stdio。
//! 禁止与 `commands/system.rs` 重复维护子进程。

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::bcip_binary;
use crate::config;

const APP_SERVER_EVENT: &str = "app-server-message";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppServerStatus {
    pub connected: bool,
    pub transport: String,
    pub error: Option<String>,
}

pub struct AppServerManager {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    child: Option<Child>,
    connected: bool,
    transport: String,
    last_error: Option<String>,
}

impl AppServerManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                child: None,
                connected: false,
                transport: String::new(),
                last_error: None,
            })),
        }
    }

    fn lock_inner(&self) -> Result<std::sync::MutexGuard<'_, Inner>, String> {
        Ok(self
            .inner
            .lock()
            .unwrap_or_else(|e| {
                eprintln!("[app_server_manager] mutex poisoned, recovering: {e}");
                e.into_inner()
            }))
    }

    pub fn status(&self) -> AppServerStatus {
        let guard = self.lock_inner();
        match guard {
            Ok(g) => AppServerStatus {
                connected: g.connected,
                transport: g.transport.clone(),
                error: g.last_error.clone(),
            },
            Err(_) => AppServerStatus {
                connected: false,
                transport: String::new(),
                error: Some("内部锁错误".to_string()),
            },
        }
    }

    pub fn connect(&self, app: AppHandle) -> Result<AppServerStatus, String> {
        // 如果已有活跃连接且子进程仍在运行，复用
        {
            let mut guard = self.lock_inner()?;
            if let Some(ref mut child) = guard.child {
                if let Ok(status) = child.try_wait() {
                    if status.is_none() {
                        eprintln!("[app_server_manager] reusing existing connection (pid={})", child.id());
                        guard.connected = true;
                        guard.last_error = None;
                        // 不要调用 self.status()——当前仍持有 guard
                        return Ok(AppServerStatus {
                            connected: true,
                            transport: guard.transport.clone(),
                            error: None,
                        });
                    }
                }
            }
        }

        self.disconnect()?;

        let sock = control_socket_path();
        let use_proxy = sock.as_ref().ok().is_some_and(|p| p.exists());

        let (transport, args): (&str, Vec<&str>) = if use_proxy {
            ("proxy", vec!["app-server", "proxy"])
        } else {
            ("stdio", vec!["app-server", "--listen", "stdio://"])
        };

        let resolved = bcip_binary::resolve_bcip_binary(Some(&app)).ok_or_else(|| {
            "未找到 bcip 可执行文件。请安装 CLI 或将 bcip 放入应用 Resources/bin/。".to_string()
        })?;
        eprintln!("[app_server_manager] spawning: {} {} (source={})", resolved.path.display(), args.join(" "), resolved.source);
        let mut child = Self::spawn_bcip_child(&resolved.path, &args)?;
        let child_pid = child.id();
        eprintln!("[app_server_manager] child spawned, pid={}", child_pid);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "app-server stdout 不可用".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "app-server stderr 不可用".to_string())?;

        {
            let mut guard = self.lock_inner()?;
            guard.child = Some(child);
            guard.connected = true;
            guard.transport = transport.to_string();
            guard.last_error = None;
        }

        Self::spawn_stderr_drainer(stderr);
        Self::spawn_stdout_reader(Arc::clone(&self.inner), app, stdout);

        Ok(self.status())
    }

    pub fn disconnect(&self) -> Result<(), String> {
        let mut guard = self.lock_inner()?;
        if let Some(mut child) = guard.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        guard.connected = false;
        guard.transport.clear();
        Ok(())
    }

    pub fn send_line(&self, line: String) -> Result<(), String> {
        let mut guard = self.lock_inner()?;
        let child = guard
            .child
            .as_mut()
            .ok_or_else(|| "app-server 未连接".to_string())?;
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "app-server stdin 不可用".to_string())?;
        eprintln!("[app_server_manager] send_line: {} bytes", line.len());
        writeln!(stdin, "{line}").map_err(|e| format!("写入 app-server 失败: {e}"))?;
        stdin
            .flush()
            .map_err(|e| format!("刷新 app-server stdin 失败: {e}"))?;
        Ok(())
    }

    fn spawn_stderr_drainer(stderr: std::process::ChildStderr) {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(content) => {
                        let preview = content.chars().take(300).collect::<String>();
                        eprintln!("[bcip-child:stderr] {preview}");
                    }
                    Err(_) => {}
                }
            }
            eprintln!("[bcip-child:stderr] drainer exiting");
        });
    }

    fn spawn_bcip_child(bcip_path: &std::path::Path, args: &[&str]) -> Result<Child, String> {
        let mut cmd = Command::new(bcip_path);
        config::apply_codex_home_env(&mut cmd);
        cmd.env("RUST_LOG", "warn");
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!(
                    "无法启动 {} {}: {e}",
                    bcip_path.display(),
                    args.join(" ")
                )
            })
    }

    fn spawn_stdout_reader(inner: Arc<Mutex<Inner>>, app: AppHandle, stdout: std::process::ChildStdout) {
        thread::spawn(move || {
            eprintln!("[app_server_manager] stdout reader started");
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(payload) if !payload.trim().is_empty() => {
                        let preview = payload.chars().take(200).collect::<String>();
                        eprintln!("[app_server_manager] stdout: {preview}");
                        // emit 可能因 WebView 重启暂时失败，不中断 reader
                        let _ = app.emit(APP_SERVER_EVENT, payload);
                    }
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("[app_server_manager] stdout read error: {err}");
                        let mut guard = inner.lock().unwrap_or_else(|e| {
                            eprintln!("[app_server_manager] stdout reader: mutex poisoned, recovering: {e}");
                            e.into_inner()
                        });
                        guard.connected = false;
                        guard.last_error = Some(format!("读取 app-server 输出失败: {err}"));
                        let _ = app.emit(
                            APP_SERVER_EVENT,
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "method": "bcip/desktop/transportError",
                                "params": { "message": guard.last_error.clone() }
                            })
                            .to_string(),
                        );
                        break;
                    }
                }
            }
            eprintln!("[app_server_manager] stdout reader exiting (child exited or error)");
            let mut guard = inner.lock().unwrap_or_else(|e| {
                eprintln!("[app_server_manager] stdout reader exit: mutex poisoned, recovering: {e}");
                e.into_inner()
            });
            guard.connected = false;
        });
    }
}

fn control_socket_path() -> Result<PathBuf, String> {
    Ok(config::find_codex_home()?
        .join("app-server-control")
        .join("app-server-control.sock"))
}

impl Drop for AppServerManager {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
