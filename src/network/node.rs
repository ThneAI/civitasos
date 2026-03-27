//! Node management for the network layer

use std::collections::HashMap;
use std::net::SocketAddr;

use super::NodeInfo;

/// Node status
#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Connected,
    Disconnected,
    Connecting,
    Unreachable,
}

/// Node manager handles node lifecycle, health checks, and connection management
pub struct NodeManager {
    /// Registered nodes
    nodes: HashMap<String, NodeInfo>,
    /// Current connection status of nodes
    node_statuses: HashMap<String, NodeStatus>,
    /// Connection attempts count
    connection_attempts: HashMap<String, u32>,
    /// Maximum connection attempts before marking as unreachable
    max_connection_attempts: u32,
}

impl Default for NodeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeManager {
    pub fn new() -> Self {
        NodeManager {
            nodes: HashMap::new(),
            node_statuses: HashMap::new(),
            connection_attempts: HashMap::new(),
            max_connection_attempts: 3,
        }
    }

    /// Register a new node
    pub fn register_node(&mut self, node_info: NodeInfo) {
        let node_id = node_info.id.clone();
        self.nodes.insert(node_id.clone(), node_info);
        self.node_statuses.insert(node_id, NodeStatus::Disconnected);
    }

    /// Remove a node
    pub fn remove_node(&mut self, node_id: &str) -> Option<NodeInfo> {
        self.node_statuses.remove(node_id);
        self.connection_attempts.remove(node_id);
        self.nodes.remove(node_id)
    }

    /// Update node status
    pub fn update_node_status(&mut self, node_id: &str, status: NodeStatus) {
        self.node_statuses
            .insert(node_id.to_string(), status.clone());

        // Reset connection attempts on successful connection
        if matches!(status, NodeStatus::Connected) {
            self.connection_attempts.remove(node_id);
        }
    }

    /// Mark a connection attempt
    pub fn mark_connection_attempt(&mut self, node_id: &str) {
        let attempts = self
            .connection_attempts
            .entry(node_id.to_string())
            .or_insert(0);
        *attempts += 1;

        // If too many failed attempts, mark as unreachable
        if *attempts >= self.max_connection_attempts {
            self.node_statuses
                .insert(node_id.to_string(), NodeStatus::Unreachable);
        }
    }

    /// Get node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&NodeInfo> {
        self.nodes.get(node_id)
    }

    /// Get node status
    pub fn get_node_status(&self, node_id: &str) -> Option<&NodeStatus> {
        self.node_statuses.get(node_id)
    }

    /// Get all nodes with a specific status
    pub fn get_nodes_by_status(&self, status: NodeStatus) -> Vec<&NodeInfo> {
        self.nodes
            .iter()
            .filter(|(id, _)| {
                if let Some(current_status) = self.node_statuses.get(*id) {
                    *current_status == status
                } else {
                    false
                }
            })
            .map(|(_, node)| node)
            .collect()
    }

    /// Get all connected nodes
    pub fn get_connected_nodes(&self) -> Vec<&NodeInfo> {
        self.get_nodes_by_status(NodeStatus::Connected)
    }

    /// Get all disconnected nodes
    pub fn get_disconnected_nodes(&self) -> Vec<&NodeInfo> {
        self.get_nodes_by_status(NodeStatus::Disconnected)
    }

    /// Get all reachable nodes (not unreachable)
    pub fn get_reachable_nodes(&self) -> Vec<&NodeInfo> {
        self.nodes
            .iter()
            .filter(|(id, _)| {
                if let Some(status) = self.node_statuses.get(*id) {
                    *status != NodeStatus::Unreachable
                } else {
                    true // Assume reachable if not tracked
                }
            })
            .map(|(_, node)| node)
            .collect()
    }

    /// Perform health check on all nodes
    pub async fn perform_health_check(&mut self) {
        for (node_id, node_info) in self.nodes.iter() {
            // Skip unreachable nodes
            if matches!(
                self.node_statuses.get(node_id),
                Some(NodeStatus::Unreachable)
            ) {
                continue;
            }

            let reachable = self.check_node_connectivity(node_info.address).await;

            if reachable {
                if !matches!(self.node_statuses.get(node_id), Some(NodeStatus::Connected)) {
                    self.node_statuses
                        .insert(node_id.clone(), NodeStatus::Connected);
                }
            } else {
                if matches!(self.node_statuses.get(node_id), Some(NodeStatus::Connected)) {
                    self.node_statuses
                        .insert(node_id.clone(), NodeStatus::Disconnected);
                }
            }
        }
    }

    /// Check if a node is reachable
    async fn check_node_connectivity(&self, addr: SocketAddr) -> bool {
        // Simple connectivity check - try to establish TCP connection
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => {
                // Try to send a ping message
                // This would require a more complex implementation in practice
                true
            }
            Err(_) => false,
        }
    }

    /// Get all nodes
    pub fn get_all_nodes(&self) -> Vec<&NodeInfo> {
        self.nodes.values().collect()
    }

    /// Get node statistics
    pub fn get_statistics(&self) -> NodeStatistics {
        let mut connected = 0;
        let mut disconnected = 0;
        let mut connecting = 0;
        let mut unreachable = 0;

        for status in self.node_statuses.values() {
            match status {
                NodeStatus::Connected => connected += 1,
                NodeStatus::Disconnected => disconnected += 1,
                NodeStatus::Connecting => connecting += 1,
                NodeStatus::Unreachable => unreachable += 1,
            }
        }

        NodeStatistics {
            total: self.nodes.len(),
            connected,
            disconnected,
            connecting,
            unreachable,
        }
    }
}

/// Node statistics
#[derive(Debug)]
pub struct NodeStatistics {
    pub total: usize,
    pub connected: usize,
    pub disconnected: usize,
    pub connecting: usize,
    pub unreachable: usize,
}

impl NodeStatistics {
    pub fn connected_ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.connected as f64 / self.total as f64
        }
    }

    pub fn healthy_ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.connected + self.disconnected) as f64 / self.total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_manager() {
        let mut node_manager = NodeManager::new();

        // Create a test node
        let node_info = NodeInfo {
            id: "node1".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            capabilities: vec!["execution".to_string(), "consensus".to_string()],
            version: "1.0.0".to_string(),
            last_seen: 0,
        };

        // Register the node
        node_manager.register_node(node_info.clone());
        assert_eq!(node_manager.get_node("node1").unwrap().id, "node1");

        // Update status
        node_manager.update_node_status("node1", NodeStatus::Connected);
        assert_eq!(
            node_manager.get_node_status("node1"),
            Some(&NodeStatus::Connected)
        );

        // Test statistics
        let stats = node_manager.get_statistics();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.connected, 1);
        assert_eq!(stats.disconnected, 0);
        assert_eq!(stats.unreachable, 0);
    }

    #[test]
    fn test_node_status_transitions() {
        let mut node_manager = NodeManager::new();

        let node_info = NodeInfo {
            id: "node2".to_string(),
            address: "127.0.0.1:8081".parse().unwrap(),
            capabilities: vec!["governance".to_string()],
            version: "1.0.0".to_string(),
            last_seen: 0,
        };

        node_manager.register_node(node_info);

        // Test all status transitions
        node_manager.update_node_status("node2", NodeStatus::Connecting);
        assert_eq!(
            node_manager.get_node_status("node2"),
            Some(&NodeStatus::Connecting)
        );

        node_manager.update_node_status("node2", NodeStatus::Connected);
        assert_eq!(
            node_manager.get_node_status("node2"),
            Some(&NodeStatus::Connected)
        );

        node_manager.update_node_status("node2", NodeStatus::Disconnected);
        assert_eq!(
            node_manager.get_node_status("node2"),
            Some(&NodeStatus::Disconnected)
        );

        node_manager.update_node_status("node2", NodeStatus::Unreachable);
        assert_eq!(
            node_manager.get_node_status("node2"),
            Some(&NodeStatus::Unreachable)
        );
    }
}
