//! Network module for CivitasOS
//! Implements P2P networking, message propagation, and node discovery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

pub mod discovery;
pub mod message;
pub mod node;

pub use discovery::*;
pub use message::*;
pub use node::*;

use crate::execution::AtomicDecisionUnit;
use crate::governance::GovernanceProposal;

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub max_connections: usize,
    pub boot_nodes: Vec<SocketAddr>,
    pub network_id: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_addr: "0.0.0.0:8080".parse().unwrap(),
            max_connections: 100,
            boot_nodes: vec![],
            network_id: 1, // Main network
        }
    }
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Propagate an atomic decision unit
    Adu(AtomicDecisionUnit),
    /// Propagate a governance proposal
    Proposal(GovernanceProposal),
    /// Request for node discovery
    DiscoveryRequest(DiscoveryRequest),
    /// Response for node discovery
    DiscoveryResponse(DiscoveryResponse),
    /// Ping/Pong for liveness
    Ping(u64), // Timestamp
    Pong(u64), // Echo of timestamp
    /// Transaction or state update
    Transaction(Vec<u8>),
    /// Block proposal in consensus
    BlockProposal(Vec<u8>),
}

/// Discovery request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRequest {
    pub requester_id: String,
    pub network_id: u64,
}

/// Discovery response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub nodes: Vec<NodeInfo>,
    pub timestamp: u64,
}

/// Information about a network node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub address: SocketAddr,
    pub capabilities: Vec<String>,
    pub version: String,
    pub last_seen: u64,
}

/// Main network service
pub struct NetworkService {
    config: NetworkConfig,
    connections: HashMap<String, TcpStream>,
    node_registry: HashMap<String, NodeInfo>,
    message_tx: broadcast::Sender<NetworkMessage>,
    message_rx: broadcast::Receiver<NetworkMessage>,
}

impl NetworkService {
    pub fn new(config: NetworkConfig) -> Self {
        let (message_tx, message_rx) = broadcast::channel(100);

        NetworkService {
            config,
            connections: HashMap::new(),
            node_registry: HashMap::new(),
            message_tx,
            message_rx,
        }
    }

    /// Start the network service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        println!("Network service started on {}", self.config.listen_addr);

        // Connect to boot nodes
        self.connect_to_boot_nodes().await;

        // Accept incoming connections
        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New connection from: {}", addr);

            // Handle the connection in a separate task
            self.handle_connection(socket, addr).await;
        }
    }

    /// Connect to boot nodes
    async fn connect_to_boot_nodes(&self) {
        for boot_node in &self.config.boot_nodes {
            if let Err(e) = self.connect_to_node(*boot_node).await {
                println!("Failed to connect to boot node {}: {}", boot_node, e);
            }
        }
    }

    /// Connect to a specific node
    async fn connect_to_node(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let _stream = TcpStream::connect(addr).await?;
        // TODO: Implement connection handshake and registration
        println!("Connected to node at {}", addr);
        Ok(())
    }

    /// Handle an incoming connection
    async fn handle_connection(&mut self, _socket: TcpStream, _addr: SocketAddr) {
        // TODO: Implement connection handling logic
        println!("Handling connection from {}", _addr);
    }

    /// Broadcast a message to all connected peers
    pub async fn broadcast_message(
        &self,
        message: NetworkMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.message_tx.send(message)?;
        Ok(())
    }

    /// Send a message to a specific peer
    pub async fn send_message_to_peer(
        &mut self,
        peer_id: &str,
        _message: NetworkMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement peer-specific message sending
        println!("Sending message to peer: {}", peer_id);
        Ok(())
    }

    /// Discover new nodes in the network
    pub async fn discover_nodes(&self) -> Result<Vec<NodeInfo>, Box<dyn std::error::Error>> {
        // TODO: Implement node discovery logic
        Ok(vec![])
    }

    /// Register a new node in the registry
    pub fn register_node(&mut self, node_info: NodeInfo) {
        self.node_registry.insert(node_info.id.clone(), node_info);
    }

    /// Get information about a specific node
    pub fn get_node_info(&self, node_id: &str) -> Option<&NodeInfo> {
        self.node_registry.get(node_id)
    }

    /// Get all registered nodes
    pub fn get_all_nodes(&self) -> Vec<&NodeInfo> {
        self.node_registry.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.listen_addr, "0.0.0.0:8080".parse().unwrap());
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.network_id, 1);
    }

    #[tokio::test]
    async fn test_network_service_creation() {
        let config = NetworkConfig::default();
        let service = NetworkService::new(config);

        assert_eq!(service.node_registry.len(), 0);
        assert_eq!(service.connections.len(), 0);
    }
}
