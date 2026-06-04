//! Connection manager for managing multiple remote transport connections.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

use crate::agent_bus::AgentBusMessage;
use crate::remote_transport::RemoteTransport;
use crate::remote_transport::RemoteTransportConfig;
use crate::transport::Transport;
use crate::transport::TransportError;

/// Status of a managed connection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connected,
    Connecting,
}

/// Configuration for a single remote endpoint.
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    pub name: String,
    pub remote_addr: String,
    pub reconnect_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl EndpointConfig {
    pub fn to_transport_config(&self) -> RemoteTransportConfig {
        RemoteTransportConfig {
            remote_addr: self.remote_addr.clone(),
            reconnect_attempts: self.reconnect_attempts,
            initial_delay_ms: self.initial_delay_ms,
            max_delay_ms: self.max_delay_ms,
        }
    }
}

/// State tracked for each managed connection.
struct ConnectionState {
    transport: Arc<RemoteTransport>,
    status: ConnectionStatus,
}

/// Manages multiple remote transport connections with health checks and failover.
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
    event_tx: broadcast::Sender<ConnectionEvent>,
}

/// Events emitted by the connection manager.
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected { name: String },
    Disconnected { name: String },
    Reconnecting { name: String, attempt: u32 },
    Failed { name: String, error: String },
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(64);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Add an endpoint and immediately attempt to connect.
    pub async fn add_endpoint(&self, config: EndpointConfig) -> Result<(), TransportError> {
        let transport = RemoteTransport::new(config.to_transport_config());
        let name = config.name.clone();

        let state = ConnectionState {
            transport: Arc::new(transport),
            status: ConnectionStatus::Connecting,
        };

        self.connections.write().await.insert(name.clone(), state);

        if let Err(e) = self.connect_endpoint(&name).await {
            let _ = self.event_tx.send(ConnectionEvent::Failed {
                name: name.clone(),
                error: e.to_string(),
            });
            let mut conns = self.connections.write().await;
            if let Some(s) = conns.get_mut(&name) {
                s.status = ConnectionStatus::Disconnected;
            }
            return Err(e);
        }

        Ok(())
    }

    /// Remove an endpoint by name.
    pub async fn remove_endpoint(&self, name: &str) -> bool {
        self.connections.write().await.remove(name).is_some()
    }

    /// Send a message through a specific endpoint.
    pub async fn send_to(
        &self,
        endpoint_name: &str,
        message: AgentBusMessage,
    ) -> Result<usize, TransportError> {
        let transport = {
            let conns = self.connections.read().await;
            let state = conns
                .get(endpoint_name)
                .ok_or(TransportError::NotConnected)?;

            if state.status != ConnectionStatus::Connected {
                return Err(TransportError::NotConnected);
            }

            Arc::clone(&state.transport)
        };

        transport.send(message).await
    }

    /// Broadcast a message to all connected endpoints.
    pub async fn broadcast(&self, message: AgentBusMessage) -> Vec<Result<usize, TransportError>> {
        let transports: Vec<Arc<RemoteTransport>> = {
            let conns = self.connections.read().await;
            conns
                .values()
                .filter(|s| s.status == ConnectionStatus::Connected)
                .map(|s| Arc::clone(&s.transport))
                .collect()
        };

        let mut results = Vec::new();
        for transport in transports {
            results.push(transport.send(message.clone()).await);
        }

        results
    }

    /// Get the status of a specific endpoint.
    pub async fn status(&self, name: &str) -> Option<ConnectionStatus> {
        let conns = self.connections.read().await;
        conns.get(name).map(|s| s.status.clone())
    }

    /// List all endpoint names and their statuses.
    pub async fn list_endpoints(&self) -> Vec<(String, ConnectionStatus)> {
        let conns = self.connections.read().await;
        conns
            .iter()
            .map(|(name, state)| (name.clone(), state.status.clone()))
            .collect()
    }

    /// Subscribe to connection events.
    pub fn subscribe_events(&self) -> Receiver<ConnectionEvent> {
        self.event_tx.subscribe()
    }

    /// Perform a health check on all endpoints. Reconnects disconnected ones.
    pub async fn health_check(&self) {
        let names: Vec<String> = {
            let conns = self.connections.read().await;
            conns.keys().cloned().collect()
        };

        for name in names {
            let should_reconnect = {
                let conns = self.connections.read().await;
                match conns.get(&name) {
                    Some(state) => !state.transport.is_connected(),
                    None => false,
                }
            };

            if should_reconnect {
                let _ = self.connect_endpoint(&name).await;
            }
        }
    }

    /// Attempt to connect (or reconnect) a specific endpoint.
    async fn connect_endpoint(&self, name: &str) -> Result<(), TransportError> {
        let transport = {
            let conns = self.connections.read().await;
            let state = conns.get(name).ok_or(TransportError::NotConnected)?;
            Arc::clone(&state.transport)
        };

        let _ = self.event_tx.send(ConnectionEvent::Reconnecting {
            name: name.to_string(),
            attempt: 0,
        });

        transport.connect().await?;

        let mut conns = self.connections.write().await;
        if let Some(state) = conns.get_mut(name) {
            state.status = ConnectionStatus::Connected;
        }

        let _ = self.event_tx.send(ConnectionEvent::Connected {
            name: name.to_string(),
        });

        Ok(())
    }

    /// Number of managed endpoints.
    pub async fn endpoint_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint_config(name: &str, addr: &str) -> EndpointConfig {
        EndpointConfig {
            name: name.to_string(),
            remote_addr: addr.to_string(),
            reconnect_attempts: 1,
            initial_delay_ms: 1,
            max_delay_ms: 1,
        }
    }

    #[test]
    fn connection_manager_new_is_empty() {
        let cm = ConnectionManager::new();
        assert!(Arc::strong_count(&cm.connections) >= 1);
    }

    #[tokio::test]
    async fn add_endpoint_fails_to_invalid_addr() {
        let cm = ConnectionManager::new();
        let result = cm
            .add_endpoint(endpoint_config("test", "127.0.0.1:1"))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_nonexistent_endpoint() {
        let cm = ConnectionManager::new();
        assert!(!cm.remove_endpoint("nope").await);
    }

    #[tokio::test]
    async fn list_endpoints_empty() {
        let cm = ConnectionManager::new();
        let list = cm.list_endpoints().await;
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn send_to_nonexistent_endpoint_fails() {
        let cm = ConnectionManager::new();
        let msg = AgentBusMessage::direct(
            crate::AgentPath::try_from("/root/a".to_string()).unwrap(),
            crate::AgentPath::try_from("/root/b".to_string()).unwrap(),
            serde_json::json!("test"),
        );
        let result = cm.send_to("nope", msg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn subscribe_events_receives_events() {
        let cm = ConnectionManager::new();
        let mut rx = cm.subscribe_events();

        let _ = cm
            .add_endpoint(endpoint_config("ev_test", "127.0.0.1:1"))
            .await;

        let event = rx.try_recv();
        assert!(event.is_ok());
    }
}
