pub mod session;

use std::sync::Arc;
use std::time::Duration;

use codex_im_protocol::ClientMessage;
use codex_im_protocol::ServerMessage;
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::error;
use tracing::info;
use tracing::warn;

use session::SessionStore;

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("WebSocket 连接失败: {0}")]
    Connection(String),
    #[error("消息序列化失败: {0}")]
    Serialization(String),
    #[error("会话错误: {0}")]
    Session(String),
}

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub server_url: String,
    pub max_reconnect: u32,
    pub heartbeat_interval_secs: u64,
    pub session_db_path: String,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://127.0.0.1:3456".into(),
            max_reconnect: 10,
            heartbeat_interval_secs: 30,
            session_db_path: "~/.bcip/adapter-sessions.db".into(),
        }
    }
}

#[derive(Debug)]
pub struct ImBridge {
    config: BridgeConfig,
    #[allow(dead_code)]
    session_store: Arc<SessionStore>,
    message_tx: tokio::sync::mpsc::UnboundedSender<ClientMessage>,
    message_rx: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<ClientMessage>>>,
    event_tx: tokio::sync::broadcast::Sender<ServerMessage>,
    connected: Arc<Mutex<bool>>,
}

impl ImBridge {
    pub fn new(config: BridgeConfig, session_store: Arc<SessionStore>) -> Self {
        let (msg_tx, msg_rx) = tokio::sync::mpsc::unbounded_channel();
        let (evt_tx, _) = tokio::sync::broadcast::channel(256);

        Self {
            config,
            session_store,
            message_tx: msg_tx,
            message_rx: Arc::new(Mutex::new(msg_rx)),
            event_tx: evt_tx,
            connected: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn connect(&self) -> Result<(), BridgeError> {
        let mut attempt = 0u32;
        let mut delay = Duration::from_secs(1);

        loop {
            if attempt >= self.config.max_reconnect {
                return Err(BridgeError::Connection("达到最大重连次数".into()));
            }

            match self.try_connect().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    warn!(attempt, error = %e, "WebSocket 连接失败，即将重试");
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(Duration::from_secs(60));
                    attempt += 1;
                }
            }
        }
    }

    #[allow(clippy::await_holding_invalid_type)]
    async fn try_connect(&self) -> Result<(), BridgeError> {
        let url_str = self.config.server_url.clone();
        info!(url = %url_str, "正在连接 WebSocket");
        let (ws_stream, _) = connect_async(&url_str)
            .await
            .map_err(|e| BridgeError::Connection(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        *self.connected.lock().await = true;
        info!("WebSocket 已连接");

        let heartbeat_interval = Duration::from_secs(self.config.heartbeat_interval_secs);
        let msg_rx = self.message_rx.clone();
        let connected_send = self.connected.clone();
        let connected_recv = self.connected.clone();
        let connected_heartbeat = self.connected.clone();
        let event_tx = self.event_tx.clone();

        // Writer task: sends outgoing messages and heartbeat Pings.
        let (pong_notify_tx, mut pong_notify_rx) = tokio::sync::mpsc::channel::<()>(1);
        tokio::spawn(async move {
            let mut heartbeat_ticker = tokio::time::interval(heartbeat_interval);
            heartbeat_ticker.tick().await; // skip first immediate tick
            let mut missed_pongs: u32 = 0;
            const MAX_MISSED_PONGS: u32 = 3;

            loop {
                tokio::select! {
                    // Outgoing application messages
                    msg = async {
                        let mut rx = msg_rx.lock().await;
                        rx.recv().await
                    } => {
                        match msg {
                            Some(msg) => {
                                let json = serde_json::to_string(&msg).expect("序列化 ClientMessage");
                                if let Err(e) = write.send(Message::Text(json.into())).await {
                                    error!(%e, "发送消息失败");
                                    *connected_send.lock().await = false;
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    // Heartbeat Ping
                    _ = heartbeat_ticker.tick() => {
                        if !*connected_heartbeat.lock().await {
                            break;
                        }
                        tracing::debug!("发送心跳 Ping");
                        if let Err(e) = write.send(Message::Ping(vec![].into())).await {
                            error!(%e, "发送 Ping 失败");
                            *connected_heartbeat.lock().await = false;
                            break;
                        }
                        // Wait for Pong with timeout
                        match tokio::time::timeout(
                            Duration::from_secs(10),
                            pong_notify_rx.recv(),
                        ).await {
                            Ok(Some(())) => {
                                missed_pongs = 0;
                            }
                            _ => {
                                missed_pongs += 1;
                                warn!(missed_pongs, "心跳 Pong 超时");
                                if missed_pongs >= MAX_MISSED_PONGS {
                                    warn!("连续 {} 次未收到 Pong，断开连接", MAX_MISSED_PONGS);
                                    *connected_heartbeat.lock().await = false;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        // Reader task: reads incoming messages and signals Pong reception.
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(event) => {
                            event_tx.send(event).ok();
                        }
                        Err(e) => {
                            warn!(%e, "解析 ServerMessage 失败");
                        }
                    },
                    Ok(Message::Pong(_)) => {
                        tracing::debug!("收到心跳 Pong");
                        let _ = pong_notify_tx.try_send(());
                    }
                    Ok(Message::Ping(data)) => {
                        // Reply to server-initiated Pings
                        tracing::debug!("收到服务器 Ping，回复 Pong");
                        // tungstenite auto-replies to Ping frames, so this is a no-op
                        let _ = data;
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket 连接关闭");
                        break;
                    }
                    Err(e) => {
                        error!(%e, "WebSocket 读取错误");
                        break;
                    }
                    _ => {}
                }
            }
            *connected_recv.lock().await = false;
        });

        Ok(())
    }

    pub fn send_message(&self, msg: ClientMessage) -> Result<(), BridgeError> {
        self.message_tx
            .send(msg)
            .map_err(|e| BridgeError::Serialization(e.to_string()))
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ServerMessage> {
        self.event_tx.subscribe()
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }
}
