//! Remote transport using TCP + frame protocol for cross-process agent communication.
//!
//! Uses length-prefixed JSON frames over async TCP streams.

use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

use crate::agent_bus::AgentBusMessage;
use crate::frame::FRAME_HEADER_SIZE;
use crate::frame::FrameDecoder;
use crate::frame::FrameEncoder;
use crate::frame::MAX_FRAME_SIZE;
use crate::transport::Transport;
use crate::transport::TransportError;

#[derive(Debug, Clone)]
pub struct RemoteTransportConfig {
    pub remote_addr: String,
    pub reconnect_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RemoteTransportConfig {
    fn default() -> Self {
        Self {
            remote_addr: "127.0.0.1:0".to_string(),
            reconnect_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

/// Remote transport that connects to a remote AgentBus endpoint via TCP.
///
/// Uses frame protocol for message encoding/decoding.
pub struct RemoteTransport {
    config: RemoteTransportConfig,
    stream: Arc<Mutex<Option<TcpStream>>>,
    tx: broadcast::Sender<AgentBusMessage>,
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl RemoteTransport {
    pub fn new(config: RemoteTransportConfig) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            config,
            stream: Arc::new(Mutex::new(None)),
            tx,
            connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Connect to the remote endpoint.
    pub async fn connect(&self) -> Result<(), TransportError> {
        let mut delay = self.config.initial_delay_ms;
        let mut attempts = 0;

        loop {
            match TcpStream::connect(&self.config.remote_addr).await {
                Ok(stream) => {
                    *self.stream.lock().await = Some(stream);
                    self.connected
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                    tracing::info!(addr = %self.config.remote_addr, "remote transport connected");
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.config.reconnect_attempts {
                        return Err(TransportError::NotConnected);
                    }
                    tracing::warn!(attempt = attempts, error = %e, "connect failed, retrying");
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    delay = (delay * 2).min(self.config.max_delay_ms);
                }
            }
        }
    }

    /// Read a single frame from the stream.
    async fn read_frame(stream: &mut TcpStream) -> Result<AgentBusMessage, TransportError> {
        let mut header = [0u8; FRAME_HEADER_SIZE];
        stream
            .read_exact(&mut header)
            .await
            .map_err(|e| TransportError::ReceiveFailed(format!("header read: {e}")))?;

        let payload_len = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        if FRAME_HEADER_SIZE + payload_len > MAX_FRAME_SIZE {
            return Err(TransportError::ReceiveFailed(
                "frame exceeds max size".to_string(),
            ));
        }

        let mut payload = vec![0u8; payload_len];
        stream
            .read_exact(&mut payload)
            .await
            .map_err(|e| TransportError::ReceiveFailed(format!("payload read: {e}")))?;

        let mut data = Vec::with_capacity(FRAME_HEADER_SIZE + payload_len);
        data.extend_from_slice(&header);
        data.extend_from_slice(&payload);

        let (msg, _) = FrameDecoder::decode(&data)
            .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?;
        Ok(msg)
    }

    pub fn is_connected_flag(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Transport for RemoteTransport {
    fn send(
        &self,
        message: AgentBusMessage,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<usize, TransportError>> + Send + '_>,
    > {
        Box::pin(async move {
            let frame = FrameEncoder::encode(&message)
                .map_err(|e| TransportError::Serialization(e.to_string()))?;

            // Take the stream out of the mutex, write, then put it back.
            // This avoids holding a MutexGuard across an await point.
            let mut stream_opt = self.stream.lock().await.take();
            let result = match stream_opt.as_mut() {
                Some(stream) => stream
                    .write_all(&frame)
                    .await
                    .map_err(|e| TransportError::SendFailed(format!("write: {e}"))),
                None => Err(TransportError::NotConnected),
            };

            // Put stream back
            *self.stream.lock().await = stream_opt;
            result?;
            let _ = self.tx.send(message);
            Ok(1)
        })
    }

    fn subscribe(&self) -> Receiver<AgentBusMessage> {
        self.tx.subscribe()
    }

    fn is_connected(&self) -> bool {
        self.is_connected_flag()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentPath;
    use pretty_assertions::assert_eq;

    fn test_path(name: &str) -> AgentPath {
        AgentPath::try_from(format!("/root/{name}")).unwrap()
    }

    #[test]
    fn remote_config_default() {
        let config = RemoteTransportConfig::default();
        assert_eq!(config.reconnect_attempts, 5);
    }

    #[tokio::test]
    async fn remote_transport_not_connected_initially() {
        let transport = RemoteTransport::new(RemoteTransportConfig::default());
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn remote_transport_connect_fails_to_invalid_addr() {
        let config = RemoteTransportConfig {
            remote_addr: "127.0.0.1:1".to_string(),
            reconnect_attempts: 1,
            initial_delay_ms: 1,
            max_delay_ms: 1,
        };
        let transport = RemoteTransport::new(config);
        assert!(transport.connect().await.is_err());
    }
}
