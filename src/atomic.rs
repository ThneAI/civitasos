//! Atomic Layer for CivitasOS
//!
//! This module handles atomic operations and accountability mechanisms.
//! Layer 1 of the CivitasOS architecture.

use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Atomic Data Unit (ADU) - the fundamental unit of atomic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicDataUnit {
    /// Unique identifier for this ADU
    pub id: String,

    /// Hash of the ADU content
    pub hash: String,

    /// Type of operation
    pub operation_type: OperationType,

    /// Content of the ADU
    pub content: Vec<u8>,

    /// Timestamp of creation
    pub created_at: DateTime<Utc>,

    /// Signatures from involved parties
    pub signatures: Vec<Signature>,

    /// References to previous ADUs (for ordering)
    pub prev_refs: Vec<String>,

    /// Metadata associated with the ADU
    pub metadata: HashMap<String, String>,
}

/// Types of operations that can be performed atomically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Simple value transfer
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },

    /// State update operation
    StateUpdate {
        key: String,
        old_value: Vec<u8>,
        new_value: Vec<u8>,
    },

    /// Contract execution
    ContractExecution {
        contract_id: String,
        method: String,
        args: Vec<u8>,
    },

    /// Governance action
    GovernanceAction { proposal_id: String, action: String },

    /// Custom operation
    Custom { operation: String, params: Vec<u8> },
}

/// Signature for verifying authenticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Public key of the signer
    pub public_key: String,

    /// Signature value
    pub signature: String,

    /// Timestamp of signing
    pub signed_at: DateTime<Utc>,
}

/// Accountability anchor - links related ADUs together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountabilityAnchor {
    /// Anchor ID
    pub id: String,

    /// List of ADU IDs linked by this anchor
    pub adu_ids: Vec<String>,

    /// Type of accountability
    pub anchor_type: AnchorType,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Expiration timestamp (if applicable)
    pub expires_at: Option<DateTime<Utc>>,

    /// Verification status
    pub verified: bool,
}

/// Types of accountability anchors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnchorType {
    /// Links all ADUs in a transaction batch
    TransactionBatch,

    /// Links ADUs representing a single logical operation
    LogicalOperation,

    /// Links ADUs across different time periods
    TemporalLink,

    /// Links ADUs from the same agent
    AgentAssociation,

    /// Links ADUs related to a specific resource
    ResourceAssociation,
}

/// Proof of atomicity - demonstrates that operations were executed atomically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicityProof {
    /// Hash of the proof
    pub proof_hash: String,

    /// List of ADUs included in the proof
    pub adus: Vec<String>,

    /// Merkle root of the ADU hashes
    pub merkle_root: String,

    /// Path for merkle verification
    pub merkle_paths: Vec<MerklePath>,

    /// Timestamp of proof generation
    pub generated_at: DateTime<Utc>,

    /// Validity period
    pub valid_until: DateTime<Utc>,

    /// Verifiers who confirmed the proof
    pub verifiers: Vec<String>,
}

/// Merkle path for proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePath {
    /// Position in the tree
    pub position: usize,

    /// Sibling hashes needed for verification
    pub siblings: Vec<String>,

    /// Direction (left/right) at each level
    pub directions: Vec<bool>,
}

/// Transaction - a collection of atomic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction ID
    pub id: String,

    /// Hash of the transaction
    pub hash: String,

    /// List of ADUs in this transaction
    pub adus: Vec<AtomicDataUnit>,

    /// Sender of the transaction
    pub sender: String,

    /// Gas limit for the transaction
    pub gas_limit: u64,

    /// Gas price
    pub gas_price: u64,

    /// Timestamp of creation
    pub created_at: DateTime<Utc>,

    /// Deadline for inclusion
    pub deadline: DateTime<Utc>,

    /// Signatures for the transaction
    pub signatures: Vec<Signature>,

    /// Status of the transaction
    pub status: TransactionStatus,
}

/// Status of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction has been received but not yet processed
    Pending,

    /// Transaction is being processed
    Processing,

    /// Transaction completed successfully
    Success,

    /// Transaction failed
    Failed,

    /// Transaction was rejected
    Rejected,

    /// Transaction timed out
    TimedOut,
}

/// Atomic operation executor
pub struct AtomicExecutor {
    /// Registry of atomic operations
    pub registry: HashMap<String, AtomicDataUnit>,

    /// Accountability anchors
    pub anchors: HashMap<String, AccountabilityAnchor>,

    /// Transaction pool
    pub transaction_pool: Vec<Transaction>,

    /// Completed transactions
    pub completed_transactions: HashMap<String, Transaction>,

    /// Proof store
    pub proof_store: HashMap<String, AtomicityProof>,

    /// Gas accounting
    pub gas_accounting: GasAccounting,
}

/// Gas accounting for atomic operations
#[derive(Debug, Clone)]
pub struct GasAccounting {
    /// Gas used per operation type
    pub gas_usage: HashMap<String, u64>,

    /// Gas prices for different operations
    pub gas_prices: HashMap<String, u64>,

    /// Gas limits for different contexts
    pub gas_limits: HashMap<String, u64>,

    /// Total gas consumed
    pub total_consumed: u64,
}

impl Default for AtomicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicExecutor {
    /// Create a new atomic executor
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            anchors: HashMap::new(),
            transaction_pool: Vec::new(),
            completed_transactions: HashMap::new(),
            proof_store: HashMap::new(),
            gas_accounting: GasAccounting {
                gas_usage: HashMap::new(),
                gas_prices: [
                    ("transfer".to_string(), 100),
                    ("state_update".to_string(), 200),
                    ("contract_execution".to_string(), 500),
                    ("governance_action".to_string(), 300),
                ]
                .iter()
                .cloned()
                .collect(),
                gas_limits: [
                    ("transaction".to_string(), 1_000_000),
                    ("block".to_string(), 10_000_000),
                ]
                .iter()
                .cloned()
                .collect(),
                total_consumed: 0,
            },
        }
    }

    /// Create a new Atomic Data Unit
    pub fn create_adu(
        &mut self,
        operation: OperationType,
        content: Vec<u8>,
    ) -> Result<AtomicDataUnit, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();

        // Calculate gas required for the operation
        let op_type_str = match &operation {
            OperationType::Transfer { .. } => "transfer",
            OperationType::StateUpdate { .. } => "state_update",
            OperationType::ContractExecution { .. } => "contract_execution",
            OperationType::GovernanceAction { .. } => "governance_action",
            OperationType::Custom { .. } => "custom",
        }
        .to_string();

        let gas_required = *self
            .gas_accounting
            .gas_prices
            .get(&op_type_str)
            .unwrap_or(&100);
        let _gas_cost = gas_required; // Simplified gas calculation

        // Create the ADU
        let mut adu = AtomicDataUnit {
            id,
            hash: "".to_string(), // Will be calculated after construction
            operation_type: operation,
            content,
            created_at,
            signatures: Vec::new(),
            prev_refs: Vec::new(),
            metadata: HashMap::new(),
        };

        // Calculate the hash of the ADU
        let serialized = serde_json::to_vec(&adu).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let hash = format!("{:x}", hasher.finalize());
        adu.hash = hash;

        // Add to registry
        self.registry.insert(adu.id.clone(), adu.clone());

        Ok(adu)
    }

    /// Create a transaction from multiple ADUs
    pub fn create_transaction(
        &mut self,
        adus: Vec<AtomicDataUnit>,
        sender: String,
        gas_limit: u64,
    ) -> Result<Transaction, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now();

        // Verify all ADUs are valid
        for adu in &adus {
            if !self.verify_adu(adu)? {
                return Err(format!("Invalid ADU: {}", adu.id));
            }
        }

        // Calculate total gas required
        let total_gas_required: u64 = adus
            .iter()
            .map(|adu| {
                let op_type_str = match &adu.operation_type {
                    OperationType::Transfer { .. } => "transfer",
                    OperationType::StateUpdate { .. } => "state_update",
                    OperationType::ContractExecution { .. } => "contract_execution",
                    OperationType::GovernanceAction { .. } => "governance_action",
                    OperationType::Custom { .. } => "custom",
                }
                .to_string();

                *self
                    .gas_accounting
                    .gas_prices
                    .get(&op_type_str)
                    .unwrap_or(&100)
            })
            .sum();

        if total_gas_required > gas_limit {
            return Err("Gas limit exceeded".to_string());
        }

        // Create transaction
        let mut transaction = Transaction {
            id,
            hash: "".to_string(), // Will be calculated after construction
            adus,
            sender,
            gas_limit,
            gas_price: 1, // Default gas price
            created_at,
            deadline: created_at + chrono::Duration::hours(24), // 24-hour deadline
            signatures: Vec::new(),
            status: TransactionStatus::Pending,
        };

        // Calculate transaction hash
        let serialized = serde_json::to_vec(&transaction).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let hash = format!("{:x}", hasher.finalize());
        transaction.hash = hash;

        // Add to transaction pool
        self.transaction_pool.push(transaction.clone());

        Ok(transaction)
    }

    /// Verify an ADU is valid
    pub fn verify_adu(&self, adu: &AtomicDataUnit) -> Result<bool, String> {
        // Check if ADU content matches its hash
        let mut adu_clone = adu.clone();
        adu_clone.hash = "".to_string(); // Temporarily clear hash for comparison
        let serialized = serde_json::to_vec(&adu_clone).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let expected_hash = format!("{:x}", hasher.finalize());

        if adu.hash != expected_hash {
            return Ok(false);
        }

        // Verify signatures (simplified - in real implementation would verify cryptographic signatures)
        if adu.signatures.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Execute a transaction atomically
    pub fn execute_transaction(&mut self, transaction_id: &str) -> Result<bool, String> {
        // Find the transaction
        let tx_index = self
            .transaction_pool
            .iter()
            .position(|tx| tx.id == transaction_id)
            .ok_or("Transaction not found in pool")?;

        let mut transaction = self.transaction_pool.remove(tx_index);

        // Verify gas limits
        let total_gas_required: u64 = transaction
            .adus
            .iter()
            .map(|adu| {
                let op_type_str = match &adu.operation_type {
                    OperationType::Transfer { .. } => "transfer",
                    OperationType::StateUpdate { .. } => "state_update",
                    OperationType::ContractExecution { .. } => "contract_execution",
                    OperationType::GovernanceAction { .. } => "governance_action",
                    OperationType::Custom { .. } => "custom",
                }
                .to_string();

                *self
                    .gas_accounting
                    .gas_prices
                    .get(&op_type_str)
                    .unwrap_or(&100)
            })
            .sum();

        if total_gas_required > transaction.gas_limit {
            transaction.status = TransactionStatus::Rejected;
            self.completed_transactions
                .insert(transaction.id.clone(), transaction);
            return Err("Gas limit exceeded".to_string());
        }

        // Execute all ADUs in the transaction
        for adu in &transaction.adus {
            if !self.verify_adu(adu)? {
                transaction.status = TransactionStatus::Failed;
                self.completed_transactions
                    .insert(transaction.id.clone(), transaction);
                return Err("Invalid ADU in transaction".to_string());
            }

            // Process the ADU based on its type
            match &adu.operation_type {
                OperationType::Transfer {
                    from: _,
                    to: _,
                    amount: _,
                } => {
                    // In a real implementation, this would process transfers
                    // For now, we'll just note that it was processed
                    println!("Processing transfer ADU: {}", adu.id);
                }
                OperationType::StateUpdate {
                    key: _,
                    old_value: _,
                    new_value: _,
                } => {
                    // In a real implementation, this would update state
                    println!("Processing state update ADU: {}", adu.id);
                }
                OperationType::ContractExecution {
                    contract_id: _,
                    method: _,
                    args: _,
                } => {
                    // In a real implementation, this would execute contracts
                    println!("Processing contract execution ADU: {}", adu.id);
                }
                OperationType::GovernanceAction {
                    proposal_id: _,
                    action: _,
                } => {
                    // In a real implementation, this would process governance actions
                    println!("Processing governance action ADU: {}", adu.id);
                }
                OperationType::Custom {
                    operation: _,
                    params: _,
                } => {
                    // In a real implementation, this would process custom operations
                    println!("Processing custom ADU: {}", adu.id);
                }
            }
        }

        // Create accountability anchors for the transaction
        let anchor_id = self.create_anchor_for_transaction(&transaction)?;

        // Generate atomicity proof
        let proof = self.generate_atomicity_proof(&transaction.adus, &anchor_id)?;
        self.proof_store.insert(proof.proof_hash.clone(), proof);

        // Update gas accounting
        self.gas_accounting.total_consumed += total_gas_required;
        *self
            .gas_accounting
            .gas_usage
            .entry("transactions".to_string())
            .or_insert(0) += total_gas_required;

        // Mark transaction as completed successfully
        transaction.status = TransactionStatus::Success;
        self.completed_transactions
            .insert(transaction.id.clone(), transaction);

        Ok(true)
    }

    /// Create an accountability anchor for a transaction
    fn create_anchor_for_transaction(
        &mut self,
        transaction: &Transaction,
    ) -> Result<String, String> {
        let anchor_id = uuid::Uuid::new_v4().to_string();
        let adu_ids: Vec<String> = transaction.adus.iter().map(|adu| adu.id.clone()).collect();

        let anchor = AccountabilityAnchor {
            id: anchor_id.clone(),
            adu_ids,
            anchor_type: AnchorType::TransactionBatch,
            created_at: Utc::now(),
            expires_at: None,
            verified: true,
        };

        self.anchors.insert(anchor_id.clone(), anchor);
        Ok(anchor_id)
    }

    /// Generate an atomicity proof for a set of ADUs
    fn generate_atomicity_proof(
        &self,
        adus: &[AtomicDataUnit],
        _anchor_id: &str,
    ) -> Result<AtomicityProof, String> {
        let adu_ids: Vec<String> = adus.iter().map(|adu| adu.id.clone()).collect();

        // Calculate merkle root
        let mut hasher = Sha256::new();
        for adu in adus {
            hasher.update(&hex::decode(&adu.hash).map_err(|_| "Invalid hash format")?);
        }
        let merkle_root = format!("{:x}", hasher.finalize());

        let proof = AtomicityProof {
            proof_hash: "".to_string(), // Will calculate after construction
            adus: adu_ids,
            merkle_root,
            merkle_paths: Vec::new(), // Simplified - in reality would compute merkle paths
            generated_at: Utc::now(),
            valid_until: Utc::now() + chrono::Duration::days(365), // Valid for a year
            verifiers: Vec::new(),
        };

        // Calculate proof hash
        let serialized = serde_json::to_vec(&proof).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let proof_hash = format!("{:x}", hasher.finalize());

        let mut proof = proof;
        proof.proof_hash = proof_hash;

        Ok(proof)
    }

    /// Get transaction status
    pub fn get_transaction_status(&self, transaction_id: &str) -> Option<&Transaction> {
        if let Some(tx) = self
            .transaction_pool
            .iter()
            .find(|tx| tx.id == transaction_id)
        {
            Some(tx)
        } else {
            self.completed_transactions.get(transaction_id)
        }
    }

    /// Get accountability anchor
    pub fn get_anchor(&self, anchor_id: &str) -> Option<&AccountabilityAnchor> {
        self.anchors.get(anchor_id)
    }

    /// Verify an atomicity proof
    pub fn verify_atomicity_proof(&self, proof: &AtomicityProof) -> bool {
        // Verify that all ADUs in the proof exist and are valid
        for adu_id in &proof.adus {
            if let Some(adu) = self.registry.get(adu_id) {
                if !self.verify_adu(adu).unwrap_or(false) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Verify merkle root matches the ADUs
        let mut hasher = Sha256::new();
        for adu_id in &proof.adus {
            if let Some(adu) = self.registry.get(adu_id) {
                hasher.update(hex::decode(&adu.hash).unwrap_or_default());
            }
        }
        let computed_root = format!("{:x}", hasher.finalize());

        computed_root == proof.merkle_root
    }
}

impl AtomicExecutor {
    /// Start the atomic layer services
    pub async fn start(&mut self) -> Result<(), String> {
        println!("Atomic layer started");
        println!("Registry size: {}", self.registry.len());
        println!("Anchors: {}", self.anchors.len());
        println!("Transaction pool: {}", self.transaction_pool.len());
        println!("Total gas consumed: {}", self.gas_accounting.total_consumed);

        Ok(())
    }

    /// Stop the atomic layer services
    pub async fn stop(&mut self) -> Result<(), String> {
        println!("Atomic layer stopped");
        Ok(())
    }
}
