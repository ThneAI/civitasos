//! Node discovery for the network layer

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;

use super::{NodeInfo, DiscoveryRequest, DiscoveryResponse, NodeDescriptor};

/// Discovery protocol constants
const DISCOVERY_INTERVAL: Duration = Duration::from_secs(30);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_BOOT_NODES: usize = 10;
const MAX_KNOWN_NODES: usize = 1000;

/// Node discovery service
pub struct NodeDiscovery {
    /// Our node ID
    node_id: String,
    /// Our network ID
    network_id: u64,
    /// Boot nodes for initial discovery
    boot_nodes: Vec<SocketAddr>,
    /// Known nodes
    known_nodes: Mutex<HashMap<String, NodeInfo>>,
    /// Recently contacted nodes
    recent_contacts: Mutex<HashMap<String, Instant>>,
    /// Blacklisted nodes
    blacklist: Mutex<HashSet<String>>,
    /// Capabilities of our node
    capabilities: Vec<String>,
    /// Last discovery time
    last_discovery: Mutex<Option<Instant>>,
    /// Maximum number of known nodes
    max_known_nodes: usize,
}

impl NodeDiscovery {
    pub fn new(node_id: String, network_id: u64, boot_nodes: Vec<SocketAddr>) -> Self {
        NodeDiscovery {
            node_id,
            network_id,
            boot_nodes,
            known_nodes: Mutex::new(HashMap::new()),
            recent_contacts: Mutex::new(HashMap::new()),
            blacklist: Mutex::new(HashSet::new()),
            capabilities: vec![
                "execution".to_string(),
                "consensus".to_string(),
                "governance".to_string(),
                "state".to_string(),
            ],
            last_discovery: Mutex::new(None),
            max_known_nodes: MAX_KNOWN_NODES,
        }
    }

    /// Initialize discovery by connecting to boot nodes
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initializing node discovery with {} boot nodes", self.boot_nodes.len());
        
        for boot_node in &self.boot_nodes {
            if let Err(e) = self.discover_from_node(*boot_node).await {
                println!("Failed to discover from boot node {}: {}", boot_node, e);
            }
        }
        
        Ok(())
    }

    /// Perform periodic discovery
    pub async fn run_discovery_cycle(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Instant::now();
        
        // Update last discovery time
        *self.last_discovery.lock().await = Some(now);
        
        // Get random nodes to discover from
        let nodes_to_contact = self.get_random_nodes(3).await;
        
        for node_info in &nodes_to_contact {
            if let Err(e) = self.discover_from_node(node_info.address).await {
                println!("Discovery failed for {}: {}", node_info.id, e);
                
                // Mark node as potentially problematic
                self.mark_node_as_problematic(&node_info.id).await;
            }
        }
        
        Ok(())
    }

    /// Discover nodes from a specific node
    async fn discover_from_node(&self, addr: SocketAddr) -> Result<Vec<NodeInfo>, Box<dyn std::error::Error>> {
        println!("Discovering nodes from {}", addr);
        
        // Create discovery request
        let request = DiscoveryRequest {
            requester_id: self.node_id.clone(),
            network_id: self.network_id,
        };
        
        // In a real implementation, this would send the request over the network
        // For now, we'll simulate a response
        let response = self.simulate_discovery_request(request, addr).await?;
        
        // Process the response and add new nodes
        let mut new_nodes = Vec::new();
        for node_desc in response.nodes {
            let node_info = NodeInfo {
                id: node_desc.id.clone(),
                address: node_desc.address,
                capabilities: node_desc.capabilities,
                version: node_desc.version,
                last_seen: response.timestamp,
            };
            
            if self.add_node_if_new(node_info.clone()).await {
                new_nodes.push(node_info);
            }
        }
        
        // Update recent contacts
        self.update_recent_contact(&format!("responder_{}", addr)).await;
        
        Ok(new_nodes)
    }

    /// Simulate discovery request/response (in a real implementation, this would be network communication)
    async fn simulate_discovery_request(&self, _request: DiscoveryRequest, addr: SocketAddr) -> Result<DiscoveryResponse, Box<dyn std::error::Error>> {
        // In a real implementation, this would make an actual network call
        // For simulation purposes, we'll generate a mock response
        println!("Simulating discovery request to {}", addr);
        
        // Create mock response
        let mut nodes = Vec::new();
        
        // Add some mock nodes (in a real implementation, these would come from the responding node)
        for i in 1..=5 {
            nodes.push(NodeInfo {
                id: format!("mock_node_{}", i),
                address: format!("127.0.0.1:{}", 8080 + i).parse().unwrap(),
                capabilities: vec!["execution".to_string(), "consensus".to_string()],
                version: "1.0.0".to_string(),
                last_seen: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        
        Ok(DiscoveryResponse {
            nodes,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Add a node if it's not already known
    async fn add_node_if_new(&self, node_info: NodeInfo) -> bool {
        let mut known_nodes = self.known_nodes.lock().await;
        
        // Check if node is blacklisted
        {
            let blacklist = self.blacklist.lock().await;
            if blacklist.contains(&node_info.id) {
                return false;
            }
        }
        
        // Check if we've reached the maximum number of known nodes
        if known_nodes.len() >= self.max_known_nodes {
            // Remove oldest nodes to make space
            self.evict_oldest_nodes(&mut known_nodes).await;
        }
        
        // Add if not already known
        if !known_nodes.contains_key(&node_info.id) {
            known_nodes.insert(node_info.id.clone(), node_info);
            return true;
        }
        
        false
    }

    /// Evict oldest nodes when maximum is reached
    async fn evict_oldest_nodes(&self, known_nodes: &mut HashMap<String, NodeInfo>) {
        let recent_contacts = self.recent_contacts.lock().await;
        
        // Find nodes that haven't been contacted recently
        let mut removable_nodes: Vec<(String, u64)> = known_nodes
            .iter()
            .filter_map(|(id, info)| {
                if let Some(last_contact) = recent_contacts.get(id) {
                    Some((id.clone(), last_contact.elapsed().as_secs()))
                } else {
                    // Nodes never contacted are prime candidates for removal
                    Some((id.clone(), std::u64::MAX))
                }
            })
            .collect();
        
        // Sort by time since last contact (descending)
        removable_nodes.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Remove half of the excess nodes
        let excess = known_nodes.len().saturating_sub(self.max_known_nodes / 2);
        let to_remove = std::cmp::min(excess, removable_nodes.len());
        
        for i in 0..to_remove {
            let node_id = &removable_nodes[i].0;
            known_nodes.remove(node_id);
        }
    }

    /// Get random nodes for discovery
    async fn get_random_nodes(&self, count: usize) -> Vec<NodeInfo> {
        let known_nodes = self.known_nodes.lock().await;
        let recent_contacts = self.recent_contacts.lock().await;
        let blacklist = self.blacklist.lock().await;
        
        let mut eligible_nodes: Vec<&NodeInfo> = known_nodes
            .values()
            .filter(|node| {
                // Exclude blacklisted nodes
                !blacklist.contains(&node.id) &&
                // Exclude recently contacted nodes
                match recent_contacts.get(&node.id) {
                    Some(last_contact) => last_contact.elapsed() > Duration::from_secs(60), // 1 minute
                    None => true, // Never contacted
                }
            })
            .collect();
        
        // Shuffle and take the requested count
        eligible_nodes.shuffle(&mut rand::thread_rng());
        
        eligible_nodes
            .into_iter()
            .take(count)
            .cloned()
            .collect()
    }

    /// Update recent contact time for a node
    async fn update_recent_contact(&self, node_id: &str) {
        self.recent_contacts
            .lock()
            .await
            .insert(node_id.to_string(), Instant::now());
    }

    /// Mark a node as problematic (likely to be blacklisted after repeated issues)
    async fn mark_node_as_problematic(&self, node_id: &str) {
        // In a real implementation, we'd track failure counts
        // For now, we'll just print a warning
        println!("Marking node {} as problematic", node_id);
    }

    /// Get known nodes
    pub async fn get_known_nodes(&self) -> Vec<NodeInfo> {
        self.known_nodes
            .lock()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get nodes by capability
    pub async fn get_nodes_by_capability(&self, capability: &str) -> Vec<NodeInfo> {
        self.known_nodes
            .lock()
            .await
            .values()
            .filter(|node| node.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }

    /// Get bootstrap nodes
    pub fn get_boot_nodes(&self) -> &[SocketAddr] {
        &self.boot_nodes
    }

    /// Add a boot node
    pub fn add_boot_node(&mut self, addr: SocketAddr) {
        if !self.boot_nodes.contains(&addr) && self.boot_nodes.len() < MAX_BOOT_NODES {
            self.boot_nodes.push(addr);
        }
    }

    /// Blacklist a node
    pub async fn blacklist_node(&self, node_id: String) {
        self.blacklist.lock().await.insert(node_id);
    }

    /// Remove from blacklist
    pub async fn unblacklist_node(&self, node_id: &str) {
        self.blacklist.lock().await.remove(node_id);
    }

    /// Get discovery statistics
    pub async fn get_stats(&self) -> DiscoveryStats {
        let known_nodes = self.known_nodes.lock().await;
        let recent_contacts = self.recent_contacts.lock().await;
        let blacklist = self.blacklist.lock().await;
        let last_discovery = *self.last_discovery.lock().await;

        DiscoveryStats {
            known_nodes: known_nodes.len(),
            boot_nodes: self.boot_nodes.len(),
            recently_contacted: recent_contacts.len(),
            blacklisted: blacklist.len(),
            last_discovery: last_discovery.map(|i| i.elapsed().as_secs()),
        }
    }

    /// Check if discovery should run again
    pub async fn should_run_discovery(&self) -> bool {
        if let Some(last) = *self.last_discovery.lock().await {
            last.elapsed() > DISCOVERY_INTERVAL
        } else {
            true // Never run discovery yet
        }
    }
}

/// Discovery statistics
#[derive(Debug)]
pub struct DiscoveryStats {
    pub known_nodes: usize,
    pub boot_nodes: usize,
    pub recently_contacted: usize,
    pub blacklisted: usize,
    pub last_discovery: Option<u64>, // Seconds ago
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_node_discovery_creation() {
        let boot_nodes = vec!["127.0.0.1:8080".parse().unwrap()];
        let discovery = NodeDiscovery::new(
            "test_node".to_string(),
            1,
            boot_nodes,
        );

        assert_eq!(discovery.node_id, "test_node");
        assert_eq!(discovery.network_id, 1);
        assert_eq!(discovery.boot_nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_add_node_if_new() {
        let boot_nodes = vec![];
        let discovery = NodeDiscovery::new(
            "test_node".to_string(),
            1,
            boot_nodes,
        );

        let node_info = NodeInfo {
            id: "new_node".to_string(),
            address: "127.0.0.1:8081".parse().unwrap(),
            capabilities: vec!["execution".to_string()],
            version: "1.0.0".to_string(),
            last_seen: 0,
        };

        // Add the node
        let added = discovery.add_node_if_new(node_info.clone()).await;
        assert!(added);

        // Try to add the same node again
        let added_again = discovery.add_node_if_new(node_info).await;
        assert!(!added_again);

        // Verify node is in the list
        let known_nodes = discovery.get_known_nodes().await;
        assert_eq!(known_nodes.len(), 1);
        assert_eq!(known_nodes[0].id, "new_node");
    }

    #[tokio::test]
    async fn test_blacklisting() {
        let boot_nodes = vec![];
        let discovery = NodeDiscovery::new(
            "test_node".to_string(),
            1,
            boot_nodes,
        );

        let node_info = NodeInfo {
            id: "bad_node".to_string(),
            address: "127.0.0.1:8082".parse().unwrap(),
            capabilities: vec!["execution".to_string()],
            version: "1.0.0".to_string(),
            last_seen: 0,
        };

        // Add the node
        let added = discovery.add_node_if_new(node_info.clone()).await;
        assert!(added);

        // Blacklist the node
        discovery.blacklist_node("bad_node".to_string()).await;

        // Try to add the same node again (should fail due to blacklist)
        let added_after_blacklist = discovery.add_node_if_new(node_info).await;
        assert!(!added_after_blacklist);

        // Verify stats
        let stats = discovery.get_stats().await;
        assert_eq!(stats.known_nodes, 1); // Still there but blacklisted
        assert_eq!(stats.blacklisted, 1);
    }

    #[tokio::test]
    async fn test_capabilities_filter() {
        let boot_nodes = vec![];
        let discovery = NodeDiscovery::new(
            "test_node".to_string(),
            1,
            boot_nodes,
        );

        // Add nodes with different capabilities
        let node1 = NodeInfo {
            id: "node_with_exec".to_string(),
            address: "127.0.0.1:8083".parse().unwrap(),
            capabilities: vec!["execution".to_string(), "storage".to_string()],
            version: "1.0.0".to_string(),
            last_seen: 0,
        };
        
        let node2 = NodeInfo {
            id: "node_with_consensus".to_string(),
            address: "127.0.0.1:8084".parse().unwrap(),
            capabilities: vec!["consensus".to_string(), "governance".to_string()],
            version: "1.0.0".to_string(),
            last_seen: 0,
        };

        discovery.add_node_if_new(node1).await;
        discovery.add_node_if_new(node2).await;

        // Filter by execution capability
        let exec_nodes = discovery.get_nodes_by_capability("execution").await;
        assert_eq!(exec_nodes.len(), 1);
        assert_eq!(exec_nodes[0].id, "node_with_exec");

        // Filter by consensus capability
        let consensus_nodes = discovery.get_nodes_by_capability("consensus").await;
        assert_eq!(consensus_nodes.len(), 1);
        assert_eq!(consensus_nodes[0].id, "node_with_consensus");
    }
}