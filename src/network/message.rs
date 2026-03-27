//! Message handling for the network layer

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

use crate::execution::AtomicDecisionUnit;
use crate::governance::GovernanceProposal;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Adu(AtomicDecisionUnit),
    Proposal(GovernanceProposal),
    Transaction(Vec<u8>),
    Block(Vec<u8>),
    Ping(u64), // Timestamp
    Pong(u64), // Echo of timestamp
    DiscoveryRequest(DiscoveryRequest),
    DiscoveryResponse(DiscoveryResponse),
    ConsensusMessage(Vec<u8>),
}

/// Discovery request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryRequest {
    pub requester_id: String,
    pub network_id: u64,
    pub capabilities: Vec<String>,
}

/// Node descriptor for discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeDescriptor {
    pub id: String,
    pub address: std::net::SocketAddr,
    pub capabilities: Vec<String>,
    pub version: String,
}

/// Discovery response message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryResponse {
    pub responder_id: String,
    pub nodes: Vec<NodeDescriptor>,
    pub timestamp: u64,
}

/// Message priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Critical = 3,
    High = 2,
    Normal = 1,
    Low = 0,
}

/// Queued message with priority and metadata
#[derive(Debug)]
pub struct QueuedMessage {
    pub message: MessageType,
    pub priority: MessagePriority,
    pub sender: Option<String>, // Node ID of sender
    pub timestamp: Instant,
    pub ttl: u32, // Time to live (hop count)
}

impl QueuedMessage {
    pub fn new(message: MessageType, priority: MessagePriority, sender: Option<String>) -> Self {
        QueuedMessage {
            message,
            priority,
            sender,
            timestamp: Instant::now(),
            ttl: 10, // Default TTL
        }
    }

    /// Check if message has expired
    pub fn is_expired(&self, max_age_ms: u64) -> bool {
        self.timestamp.elapsed().as_millis() > max_age_ms as u128
    }
}

/// Message handler manages message queuing, prioritization, and processing
pub struct MessageHandler {
    /// High priority queue
    high_priority_queue: VecDeque<QueuedMessage>,
    /// Normal priority queue
    normal_priority_queue: VecDeque<QueuedMessage>,
    /// Low priority queue
    low_priority_queue: VecDeque<QueuedMessage>,
    /// Maximum queue sizes
    max_queue_size: usize,
    /// Maximum message age (milliseconds)
    max_message_age: u64,
    /// Statistics
    stats: MessageStats,
}

/// Message statistics
#[derive(Debug, Clone)]
pub struct MessageStats {
    pub total_processed: u64,
    pub total_dropped: u64,
    pub avg_processing_time: u64, // microseconds
    pub queue_sizes: (usize, usize, usize), // (high, normal, low)
}

impl Default for MessageStats {
    fn default() -> Self {
        MessageStats {
            total_processed: 0,
            total_dropped: 0,
            avg_processing_time: 0,
            queue_sizes: (0, 0, 0),
        }
    }
}

impl MessageHandler {
    pub fn new(max_queue_size: usize, max_message_age_ms: u64) -> Self {
        MessageHandler {
            high_priority_queue: VecDeque::new(),
            normal_priority_queue: VecDeque::new(),
            low_priority_queue: VecDeque::new(),
            max_queue_size,
            max_message_age: max_message_age_ms,
            stats: MessageStats::default(),
        }
    }

    /// Queue a message based on priority
    pub fn queue_message(&mut self, queued_message: QueuedMessage) -> Result<(), &'static str> {
        // Clean old messages first
        self.clean_old_messages();

        // Check if queues are at capacity
        if self.get_total_queue_size() >= self.max_queue_size {
            // Drop low priority messages first
            if queued_message.priority == MessagePriority::Low && !self.low_priority_queue.is_empty() {
                self.low_priority_queue.pop_front();
                self.stats.total_dropped += 1;
            } 
            // Then normal priority
            else if queued_message.priority != MessagePriority::Critical && !self.normal_priority_queue.is_empty() {
                self.normal_priority_queue.pop_front();
                self.stats.total_dropped += 1;
            } 
            // Finally high priority (only if critical message is being added to full queue)
            else if queued_message.priority == MessagePriority::Critical && !self.high_priority_queue.is_empty() {
                // For critical messages, we'll still drop if absolutely necessary
                self.high_priority_queue.pop_front();
                self.stats.total_dropped += 1;
            } else {
                return Err("All queues at maximum capacity");
            }
        }

        // Add message to appropriate queue
        match queued_message.priority {
            MessagePriority::Critical | MessagePriority::High => {
                self.high_priority_queue.push_back(queued_message);
            }
            MessagePriority::Normal => {
                self.normal_priority_queue.push_back(queued_message);
            }
            MessagePriority::Low => {
                self.low_priority_queue.push_back(queued_message);
            }
        }

        self.update_queue_stats();
        Ok(())
    }

    /// Dequeue next message based on priority
    pub fn dequeue_next_message(&mut self) -> Option<QueuedMessage> {
        // First try high priority
        if let Some(msg) = self.high_priority_queue.pop_front() {
            return Some(msg);
        }
        // Then normal priority
        if let Some(msg) = self.normal_priority_queue.pop_front() {
            return Some(msg);
        }
        // Finally low priority
        self.low_priority_queue.pop_front()
    }

    /// Process all pending messages up to a limit
    pub fn process_pending_messages<F>(&mut self, limit: usize, processor: F) -> usize
    where
        F: Fn(QueuedMessage) -> bool, // Returns true if processed successfully
    {
        let mut processed_count = 0;
        self.clean_old_messages();

        // Process messages up to the limit
        while processed_count < limit {
            if let Some(message) = self.dequeue_next_message() {
                let start_time = Instant::now();
                
                if processor(message) {
                    processed_count += 1;
                    self.stats.total_processed += 1;
                    
                    // Update average processing time
                    let elapsed = start_time.elapsed().as_micros() as u64;
                    self.stats.avg_processing_time = 
                        (self.stats.avg_processing_time + elapsed) / 2; // Moving average
                } else {
                    // If processing failed, put it back at the end of the queue with lower priority
                    // For simplicity, we'll just drop it in this implementation
                    self.stats.total_dropped += 1;
                }
            } else {
                // No more messages to process
                break;
            }
        }

        self.update_queue_stats();
        processed_count
    }

    /// Get total queue size
    fn get_total_queue_size(&self) -> usize {
        self.high_priority_queue.len() + 
        self.normal_priority_queue.len() + 
        self.low_priority_queue.len()
    }

    /// Clean old messages from all queues
    fn clean_old_messages(&mut self) {
        Self::clean_queue_by_age(&mut self.high_priority_queue);
        Self::clean_queue_by_age(&mut self.normal_priority_queue);
        Self::clean_queue_by_age(&mut self.low_priority_queue);
    }

    /// Clean old messages from a specific queue
    fn clean_queue_by_age(queue: &mut VecDeque<QueuedMessage>) {
        queue.retain(|msg| !msg.is_expired(5000)); // 5 seconds default
    }

    /// Update queue statistics
    fn update_queue_stats(&mut self) {
        self.stats.queue_sizes = (
            self.high_priority_queue.len(),
            self.normal_priority_queue.len(),
            self.low_priority_queue.len(),
        );
    }

    /// Get current statistics
    pub fn get_stats(&self) -> MessageStats {
        self.stats.clone()
    }

    /// Get queue depths
    pub fn get_queue_depths(&self) -> (usize, usize, usize) {
        (
            self.high_priority_queue.len(),
            self.normal_priority_queue.len(),
            self.low_priority_queue.len(),
        )
    }

    /// Purge all queues
    pub fn purge_queues(&mut self) {
        self.high_priority_queue.clear();
        self.normal_priority_queue.clear();
        self.low_priority_queue.clear();
    }
}

/// Broadcast manager handles message broadcasting to multiple peers
pub struct BroadcastManager {
    /// Known peers
    peers: Vec<String>,
    /// Currently broadcasting messages
    active_broadcasts: Vec<BroadcastTask>,
    /// Maximum concurrent broadcasts
    max_concurrent_broadcasts: usize,
}

/// Broadcast task
struct BroadcastTask {
    message: MessageType,
    target_peers: Vec<String>,
    completed_peers: Vec<String>,
    failed_peers: Vec<String>,
    started_at: Instant,
}

impl BroadcastManager {
    pub fn new(max_concurrent_broadcasts: usize) -> Self {
        BroadcastManager {
            peers: Vec::new(),
            active_broadcasts: Vec::new(),
            max_concurrent_broadcasts,
        }
    }

    /// Add a peer to the known peers list
    pub fn add_peer(&mut self, peer_id: String) {
        if !self.peers.contains(&peer_id) {
            self.peers.push(peer_id);
        }
    }

    /// Remove a peer from the known peers list
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.retain(|p| p != peer_id);
    }

    /// Broadcast a message to all or selected peers
    pub fn broadcast_message(&mut self, message: MessageType, target_peers: Option<Vec<String>>) -> usize {
        let targets = match target_peers {
            Some(peers) => peers,
            None => self.peers.clone(), // Broadcast to all known peers
        };

        if targets.is_empty() {
            return 0; // No peers to broadcast to
        }

        // Limit concurrent broadcasts
        if self.active_broadcasts.len() >= self.max_concurrent_broadcasts {
            // Wait for some broadcasts to complete or drop this broadcast
            return 0;
        }

        let broadcast_task = BroadcastTask {
            message,
            target_peers: targets.clone(),
            completed_peers: Vec::new(),
            failed_peers: Vec::new(),
            started_at: Instant::now(),
        };

        self.active_broadcasts.push(broadcast_task);
        targets.len()
    }

    /// Get list of known peers
    pub fn get_peers(&self) -> &[String] {
        &self.peers
    }

    /// Get number of active broadcasts
    pub fn get_active_broadcasts_count(&self) -> usize {
        self.active_broadcasts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_handler_basic() {
        let mut handler = MessageHandler::new(100, 5000);

        // Create a test message
        let msg = QueuedMessage::new(
            MessageType::Ping(12345),
            MessagePriority::Normal,
            Some("test_node".to_string()),
        );

        // Queue the message
        assert!(handler.queue_message(msg).is_ok());

        // Check queue size
        let (high, normal, low) = handler.get_queue_depths();
        assert_eq!(normal, 1);
        assert_eq!(high, 0);
        assert_eq!(low, 0);

        // Dequeue the message
        let dequeued = handler.dequeue_next_message();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().message, MessageType::Ping(12345));
    }

    #[test]
    fn test_message_priorities() {
        let mut handler = MessageHandler::new(100, 5000);

        // Add messages of different priorities
        handler.queue_message(QueuedMessage::new(
            MessageType::Ping(1),
            MessagePriority::Low,
            None,
        )).unwrap();
        
        handler.queue_message(QueuedMessage::new(
            MessageType::Ping(2),
            MessagePriority::High,
            None,
        )).unwrap();
        
        handler.queue_message(QueuedMessage::new(
            MessageType::Ping(3),
            MessagePriority::Normal,
            None,
        )).unwrap();

        // Dequeue should return high priority first
        if let Some(msg) = handler.dequeue_next_message() {
            if let MessageType::Ping(val) = msg.message {
                assert_eq!(val, 2); // High priority message
            }
        }

        // Then normal
        if let Some(msg) = handler.dequeue_next_message() {
            if let MessageType::Ping(val) = msg.message {
                assert_eq!(val, 3); // Normal priority message
            }
        }

        // Then low
        if let Some(msg) = handler.dequeue_next_message() {
            if let MessageType::Ping(val) = msg.message {
                assert_eq!(val, 1); // Low priority message
            }
        }
    }

    #[test]
    fn test_broadcast_manager() {
        let mut broadcast_manager = BroadcastManager::new(10);

        // Add some peers
        broadcast_manager.add_peer("peer1".to_string());
        broadcast_manager.add_peer("peer2".to_string());
        broadcast_manager.add_peer("peer3".to_string());

        assert_eq!(broadcast_manager.get_peers().len(), 3);

        // Broadcast to all peers
        let broadcast_count = broadcast_manager.broadcast_message(
            MessageType::Ping(123),
            None, // All peers
        );
        
        assert_eq!(broadcast_count, 3);
        assert_eq!(broadcast_manager.get_active_broadcasts_count(), 1);

        // Remove a peer
        broadcast_manager.remove_peer("peer2");
        assert_eq!(broadcast_manager.get_peers().len(), 2);
    }
}
