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
        let event_tx_heartbeat = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                let msg = {
                    let mut rx = msg_rx.lock().await;
                    rx.try_recv().ok()
                };
                if let Some(msg) = msg {
                    let json = serde_json::to_string(&msg).expect("序列化 ClientMessage");
                    if let Err(e) = write.send(Message::Text(json.into())).await {
                        error!(%e, "发送消息失败");
                        *connected_send.lock().await = false;
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

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
                    Ok(Message::Ping(_)) => {}
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

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(heartbeat_interval).await;
                if !*connected_heartbeat.lock().await {
                    break;
                }
                if event_tx_heartbeat.receiver_count() == 0 {
                    break;
                }
            }
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
