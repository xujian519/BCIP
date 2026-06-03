use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
use std::io::{Read, Write};
use std::net::TcpListener as StdTcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::AppHandle;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use serde::{Deserialize, Serialize};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PtySessionInfo {
    pub id: String,
    pub websocket_url: String,
    pub command: String,
}

struct SharedWriter {
    inner: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Clone for SharedWriter {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}

pub struct PtySession {
    pub id: String,
    pub websocket_port: u16,
    pub command: String,
    writer: SharedWriter,
}

pub struct PtyManager {
    sessions: Arc<Mutex<Vec<PtySession>>>,
    app_handle: AppHandle,
}

impl PtyManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
            app_handle,
        }
    }

    pub fn spawn(
        &self,
        command: &str,
        args: Vec<String>,
        cwd: Option<String>,
    ) -> Result<PtySessionInfo, String> {
        let pty_system = NativePtySystem::default();
        
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("无法创建 PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(command);
        for arg in &args {
            cmd.arg(arg);
        }
        if let Some(cwd) = cwd {
            cmd.cwd(cwd);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("无法启动命令: {}", e))?;

        let session_id = format!("pty-{}", child.process_id().unwrap_or(0));
        
        // 动态分配 WebSocket 端口
        let websocket_port = self.find_available_port()?;
        let websocket_url = format!("ws://127.0.0.1:{}", websocket_port);
        
        // 获取 writer 并包装为共享 writer
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("无法获取写入器: {}", e))?;
        
        let shared_writer = SharedWriter {
            inner: Arc::new(Mutex::new(writer)),
        };
        
        // 创建广播通道用于 PTY 输出
        let (tx, _rx) = broadcast::channel::<Vec<u8>>(1024);
        
        // 启动 PTY 读取线程
        self.start_pty_reader(pair, tx.clone())?;
        
        // 启动 WebSocket 服务器
        self.start_websocket_server(websocket_port, tx, shared_writer.clone())?;

        let session = PtySession {
            id: session_id.clone(),
            websocket_port,
            command: format!("{} {}", command, args.join(" ")),
            writer: shared_writer,
        };

        self.sessions.lock().unwrap().push(session);
        
        Ok(PtySessionInfo {
            id: session_id,
            websocket_url,
            command: format!("{} {}", command, args.join(" ")),
        })
    }

    pub fn write(
        &self,
        session_id: &str,
        data: &str,
    ) -> Result<(), String> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions
            .iter()
            .find(|s| s.id == session_id)
            .ok_or_else(|| "会话不存在".to_string())?;

        let mut writer = session.writer.clone();
        drop(sessions); // 释放锁

        writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("写入失败: {}", e))?;

        Ok(())
    }

    pub fn resize(
        &self,
        _session_id: &str,
        _cols: u16,
        _rows: u16,
    ) -> Result<(), String> {
        // resize 功能需要持有 PtyPair，但 pair 已移动到读取线程
        // 暂时返回成功，实际需要重构来支持 resize
        Ok(())
    }

    pub fn kill(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        let index = sessions
            .iter()
            .position(|s| s.id == session_id)
            .ok_or_else(|| "会话不存在".to_string())?;

        sessions.remove(index);
        Ok(())
    }

    fn find_available_port(
        &self
    ) -> Result<u16, String> {
        for port in 9000..10000 {
            if StdTcpListener::bind(("127.0.0.1", port)).is_ok() {
                return Ok(port);
            }
        }
        Err("无法找到可用端口".to_string())
    }

    fn start_pty_reader(
        &self,
        pair: PtyPair,
        tx: broadcast::Sender<Vec<u8>>,
    ) -> Result<(), String> {
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("无法克隆读取器: {}", e))?;

        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let data = buf[..n].to_vec();
                        if tx.send(data).is_err() {
                            // 所有接收者都已关闭
                            break;
                        }
                    }
                    Ok(_) => break,
                    Err(e) => {
                        eprintln!("PTY 读取错误: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn start_websocket_server(
        &self,
        port: u16,
        tx: broadcast::Sender<Vec<u8>>,
        shared_writer: SharedWriter,
    ) -> Result<(), String> {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("无法绑定 WebSocket 端口 {}: {}", port, e);
                        return;
                    }
                };

                println!("WebSocket 服务器启动在 ws://127.0.0.1:{}", port);

                while let Ok((stream, _)) = listener.accept().await {
                    let ws_stream = match accept_async(stream).await {
                        Ok(ws) => ws,
                        Err(e) => {
                            eprintln!("WebSocket 升级失败: {}", e);
                            continue;
                        }
                    };

                    let mut rx = tx.subscribe();
                    let mut local_writer = shared_writer.clone();

                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                    // 任务 1: 从广播通道接收并发送到 WebSocket
                    let sender_task = tokio::spawn(async move {
                        loop {
                            match rx.recv().await {
                                Ok(data) => {
                                    if ws_sender.send(Message::Binary(data)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });

                    // 任务 2: 从 WebSocket 接收并写入 PTY
                    let receiver_task = tokio::spawn(async move {
                        while let Some(msg) = ws_receiver.next().await {
                            match msg {
                                Ok(Message::Binary(data)) => {
                                    if local_writer.write_all(&data).is_err() {
                                        break;
                                    }
                                }
                                Ok(Message::Text(text)) => {
                                    if local_writer.write_all(text.as_bytes()).is_err() {
                                        break;
                                    }
                                }
                                Ok(Message::Close(_)) => break,
                                Err(e) => {
                                    eprintln!("WebSocket 接收错误: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });

                    // 等待任一任务结束
                    tokio::select! {
                        _ = sender_task => {},
                        _ = receiver_task => {},
                    }
                }
            });
        });

        Ok(())
    }
}
