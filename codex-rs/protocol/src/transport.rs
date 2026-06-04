//! Transport abstraction for AgentBus — decouples message routing from transport mechanism.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

use crate::agent_bus::AgentBusMessage;

/// Error type for transport operations.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("send failed: {0}")]
    SendFailed(String),
    #[error("receive failed: {0}")]
    ReceiveFailed(String),
    #[error("not connected")]
    NotConnected,
    #[error("timeout")]
    Timeout,
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Type-erased async result for transport operations.
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Transport trait for sending and receiving AgentBus messages.
///
/// This abstraction allows AgentBus to work with different transport backends
/// (in-process broadcast, WebSocket, etc.) without changing its core logic.
pub trait Transport: Send + Sync {
    fn send(&self, message: AgentBusMessage) -> BoxFuture<'_, Result<usize, TransportError>>;
    fn subscribe(&self) -> Receiver<AgentBusMessage>;
    fn is_connected(&self) -> bool;
}

/// In-process transport using tokio broadcast channels.
pub struct LocalTransport {
    tx: broadcast::Sender<AgentBusMessage>,
    buffer_size: usize,
}

impl LocalTransport {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_size);
        Self { tx, buffer_size }
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl Transport for LocalTransport {
    fn send(&self, message: AgentBusMessage) -> BoxFuture<'_, Result<usize, TransportError>> {
        let result = self.tx.send(message);
        Box::pin(async move {
            match result {
                Ok(n) => Ok(n),
                Err(_) => Ok(0),
            }
        })
    }

    fn subscribe(&self) -> Receiver<AgentBusMessage> {
        self.tx.subscribe()
    }

    fn is_connected(&self) -> bool {
        true
    }
}

/// Type-erased transport handle for dynamic dispatch.
pub type TransportHandle = Arc<dyn Transport>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentPath;
    use pretty_assertions::assert_eq;

    fn test_path(name: &str) -> AgentPath {
        AgentPath::try_from(format!("/root/{name}")).unwrap()
    }

    #[tokio::test]
    async fn local_transport_send_receive() {
        let transport = LocalTransport::new(16);
        let mut rx = transport.subscribe();

        let msg =
            AgentBusMessage::direct(test_path("a"), test_path("b"), serde_json::json!("hello"));
        transport.send(msg.clone()).await.unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.id, msg.id);
    }

    #[tokio::test]
    async fn local_transport_multiple_subscribers() {
        let transport = LocalTransport::new(16);
        let mut rx1 = transport.subscribe();
        let mut rx2 = transport.subscribe();

        let msg = AgentBusMessage::direct(
            test_path("a"),
            test_path("b"),
            serde_json::json!("broadcast"),
        );
        transport.send(msg.clone()).await.unwrap();

        assert_eq!(rx1.try_recv().unwrap().id, msg.id);
        assert_eq!(rx2.try_recv().unwrap().id, msg.id);
    }

    #[tokio::test]
    async fn local_transport_always_connected() {
        let transport = LocalTransport::new(16);
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn local_transport_no_receivers_returns_zero() {
        let transport = LocalTransport::new(16);
        let msg =
            AgentBusMessage::direct(test_path("a"), test_path("b"), serde_json::json!("orphan"));
        let result = transport.send(msg).await.unwrap();
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn transport_handle_dynamic_dispatch() {
        let handle: TransportHandle = Arc::new(LocalTransport::new(16));
        let mut rx = handle.subscribe();

        let msg =
            AgentBusMessage::direct(test_path("a"), test_path("b"), serde_json::json!("via_dyn"));
        handle.send(msg.clone()).await.unwrap();
        assert_eq!(rx.try_recv().unwrap().id, msg.id);
        assert!(handle.is_connected());
    }
}
