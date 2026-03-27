use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{StateStore, ExecutionResult};

// 跨链桥接类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossChainBridgeType {
    LockAndMint,      // 锁定铸造模式
    BurnAndMint,      // 燃烧铸造模式
    LiquidityPool,    // 流动性池模式
    OracleBased,      // 预言机模式
    StateChannel,     // 状态通道
}

// 跨链消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    pub id: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub sender: String,
    pub recipient: String,
    pub payload: String,           // 消息负载
    pub nonce: u64,               // 消息序号
    pub timestamp: u64,           // 时间戳
    pub signatures: Vec<String>,   // 多重签名
    pub status: CrossChainStatus,  // 消息状态
    pub relayer: Option<String>,   // 中继者
}

// 跨链状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossChainStatus {
    Pending,
    Verified,
    Executed,
    Failed,
    Reverted,
}

// 跨链事务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainTransaction {
    pub message_id: String,
    pub transaction_type: CrossChainTxType,
    pub amount: u64,
    pub asset: String,
    pub fees: u64,
    pub gas_limit: u64,
    pub execution_proof: Option<String>, // 执行证明
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossChainTxType {
    Transfer,
    Swap,
    Deposit,
    Withdraw,
    ContractCall,
    DataSync,
}

// 跨链验证器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainValidator {
    pub id: String,
    pub public_key: String,
    pub stake: u64,
    pub reputation: u64,
    pub supported_chains: Vec<String>,
    pub last_active: u64,
}

// 轻客户端证明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightClientProof {
    pub block_header: String,
    pub merkle_proof: Vec<String>,
    pub target_hash: String,
    pub validator_signatures: Vec<String>,
    pub quorum_size: u64,
    pub verified: bool,
}

// 跨链桥接器
pub struct CrossChainBridge {
    pub bridge_type: CrossChainBridgeType,
    pub supported_assets: Vec<String>,
    pub validators: Vec<CrossChainValidator>,
    pub pending_messages: HashMap<String, CrossChainMessage>,
    pub processed_messages: HashMap<String, CrossChainMessage>,
    pub state_store: StateStore,
    pub fee_structure: FeeStructure,
    pub security_threshold: u64,  // 安全阈值（需要的验证节点数量）
    pub relayers: Vec<String>,    // 中继者列表
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    pub base_fee: u64,
    pub percentage_fee: f64,      // 百分比费用
    pub min_fee: u64,             // 最低费用
    pub max_fee: u64,             // 最高费用
    pub dynamic_adjustment: bool,  // 是否动态调整
}

impl CrossChainBridge {
    pub fn new(bridge_type: CrossChainBridgeType, state_store: StateStore) -> Self {
        CrossChainBridge {
            bridge_type,
            supported_assets: vec!["CIV".to_string()], // 默认支持CIV代币
            validators: Vec::new(),
            pending_messages: HashMap::new(),
            processed_messages: HashMap::new(),
            state_store,
            fee_structure: FeeStructure {
                base_fee: 100,
                percentage_fee: 0.001, // 0.1%
                min_fee: 50,
                max_fee: 10000,
                dynamic_adjustment: true,
            },
            security_threshold: 2, // 默认需要2个验证节点
            relayers: Vec::new(),
        }
    }

    // 添加验证节点
    pub fn add_validator(&mut self, validator: CrossChainValidator) {
        self.validators.push(validator);
    }

    // 添加中继者
    pub fn add_relayer(&mut self, relayer_id: String) {
        self.relayers.push(relayer_id);
    }

    // 发送跨链消息
    pub fn send_message(&mut self, mut message: CrossChainMessage) -> Result<String, CrossChainError> {
        // 验证消息格式
        self.validate_message(&message)?;

        // 计算费用
        let fee = self.compute_fee(&message);
        
        // 在实际实现中，这里会从发送者那里收取费用
        println!("Charging fee: {}", fee);

        // 生成消息ID
        message.id = self.generate_message_id(&message);
        message.status = CrossChainStatus::Pending;
        message.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 添加到待处理消息
        self.pending_messages.insert(message.id.clone(), message.clone());

        Ok(message.id)
    }

    // 验证消息
    fn validate_message(&self, message: &CrossChainMessage) -> Result<(), CrossChainError> {
        if message.source_chain.is_empty() {
            return Err(CrossChainError::InvalidSourceChain);
        }

        if message.destination_chain.is_empty() {
            return Err(CrossChainError::InvalidDestinationChain);
        }

        if message.sender.is_empty() {
            return Err(CrossChainError::InvalidSender);
        }

        if message.recipient.is_empty() {
            return Err(CrossChainError::InvalidRecipient);
        }

        // 检查目标链是否支持
        if !self.is_chain_supported(&message.destination_chain) {
            return Err(CrossChainError::UnsupportedChain);
        }

        Ok(())
    }

    // 计算费用
    fn compute_fee(&self, message: &CrossChainMessage) -> u64 {
        let mut fee = self.fee_structure.base_fee;

        // 基于负载大小的费用
        fee += (message.payload.len() / 1024) as u64 * 10; // 每KB 10个单位

        // 基于距离的费用（简化实现）
        let distance_factor = if message.source_chain == message.destination_chain {
            0.5  // 同链便宜
        } else {
            1.0  // 跨链正常费用
        };

        fee = (fee as f64 * distance_factor) as u64;

        // 应用百分比费用
        let percentage_fee = (message.payload.len() as f64 * self.fee_structure.percentage_fee) as u64;
        fee += percentage_fee;

        // 应用最小/最大限制
        fee = std::cmp::max(fee, self.fee_structure.min_fee);
        fee = std::cmp::min(fee, self.fee_structure.max_fee);

        fee
    }

    // 生成消息ID
    fn generate_message_id(&self, message: &CrossChainMessage) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&format!("{}{}{}{}{}", 
            message.source_chain, 
            message.destination_chain, 
            message.sender, 
            message.nonce, 
            message.timestamp
        ));
        format!("{:x}", hasher.finalize())
    }

    // 检查链是否支持
    fn is_chain_supported(&self, _chain_id: &str) -> bool {
        // 在实际实现中，这里会有更复杂的链支持检查
        // 简化实现：检查是否在支持列表中
        true  // 为简化先返回true
    }

    // 验证跨链消息
    pub fn verify_message(&mut self, message_id: &str) -> Result<bool, CrossChainError> {
        let message = self.pending_messages.get_mut(message_id)
            .ok_or(CrossChainError::MessageNotFound)?;

        // 检查验证节点数量是否满足安全阈值
        if self.validators.len() < self.security_threshold as usize {
            return Err(CrossChainError::InsufficientValidators);
        }

        // 在实际实现中，这里会执行复杂的验证过程
        // 例如：验证轻客户端证明、检查源链状态等
        // 简化实现：标记为已验证
        message.status = CrossChainStatus::Verified;

        // 移动到已处理消息
        if let Some(msg) = self.pending_messages.remove(message_id) {
            self.processed_messages.insert(message_id.to_string(), msg);
        }

        Ok(true)
    }

    // 执行跨链事务
    pub fn execute_transaction(&mut self, tx: CrossChainTransaction) -> Result<ExecutionResult, CrossChainError> {
        // 验证消息是否已验证
        let message = self.processed_messages.get(&tx.message_id)
            .ok_or(CrossChainError::MessageNotFound)?;

        if !matches!(message.status, CrossChainStatus::Verified) {
            return Err(CrossChainError::MessageNotVerified);
        }

        // 执行跨链操作
        match tx.transaction_type {
            CrossChainTxType::Transfer => {
                self.execute_transfer(&tx)?;
            },
            CrossChainTxType::Swap => {
                self.execute_swap(&tx)?;
            },
            CrossChainTxType::Deposit => {
                self.execute_deposit(&tx)?;
            },
            CrossChainTxType::Withdraw => {
                self.execute_withdraw(&tx)?;
            },
            CrossChainTxType::ContractCall => {
                self.execute_contract_call(&tx)?;
            },
            CrossChainTxType::DataSync => {
                self.execute_data_sync(&tx)?;
            },
        }

        // 更新消息状态
        if let Some(msg) = self.processed_messages.get_mut(&tx.message_id) {
            msg.status = CrossChainStatus::Executed;
        }

        Ok(ExecutionResult {
            state_diffs: vec![],
            trace_hash: format!("cross_chain_{}", tx.message_id),
            gas_used: 1000,
            success: true,
        })
    }

    // 执行转账
    fn execute_transfer(&mut self, tx: &CrossChainTransaction) -> Result<(), CrossChainError> {
        // 在实际实现中，这里会执行代币转账逻辑
        // 根据桥接类型采取不同操作：
        // - LockAndMint: 锁定源链资产，目标链铸造
        // - BurnAndMint: 源链燃烧资产，目标链铸造
        // - LiquidityPool: 从目标链流动性池转移
        
        println!("Executing cross-chain transfer: {} units of {}", tx.amount, tx.asset);
        
        Ok(())
    }

    // 执行兑换
    fn execute_swap(&mut self, tx: &CrossChainTransaction) -> Result<(), CrossChainError> {
        println!("Executing cross-chain swap: {} units", tx.amount);
        Ok(())
    }

    // 执行存款
    fn execute_deposit(&mut self, tx: &CrossChainTransaction) -> Result<(), CrossChainError> {
        println!("Executing cross-chain deposit: {} units", tx.amount);
        Ok(())
    }

    // 执行提取
    fn execute_withdraw(&mut self, tx: &CrossChainTransaction) -> Result<(), CrossChainError> {
        println!("Executing cross-chain withdraw: {} units", tx.amount);
        Ok(())
    }

    // 执行合约调用
    fn execute_contract_call(&mut self, tx: &CrossChainTransaction) -> Result<(), CrossChainError> {
        println!("Executing cross-chain contract call with payload: {}", tx.message_id);
        Ok(())
    }

    // 执行数据同步
    fn execute_data_sync(&mut self, _tx: &CrossChainTransaction) -> Result<(), CrossChainError> {
        println!("Executing cross-chain data sync");
        Ok(())
    }

    // 验证轻客户端证明
    pub fn verify_light_client_proof(&self, proof: &LightClientProof) -> Result<bool, CrossChainError> {
        // 验证签名数量是否达到法定人数
        if proof.validator_signatures.len() < proof.quorum_size as usize {
            return Ok(false);
        }

        // 验证证明的有效性
        // 在实际实现中，这里会验证区块头、默克尔证明等
        // 简化实现：检查证明是否被标记为已验证
        Ok(proof.verified)
    }

    // 获取待处理消息数量
    pub fn get_pending_message_count(&self) -> usize {
        self.pending_messages.len()
    }

    // 获取已处理消息数量
    pub fn get_processed_message_count(&self) -> usize {
        self.processed_messages.len()
    }
    
    // 获取验证节点数量
    pub fn get_validator_count(&self) -> usize {
        self.validators.len()
    }

}

// 跨链错误
#[derive(Debug)]
pub enum CrossChainError {
    InvalidSourceChain,
    InvalidDestinationChain,
    InvalidSender,
    InvalidRecipient,
    UnsupportedChain,
    MessageNotFound,
    MessageNotVerified,
    InsufficientValidators,
    InvalidProof,
    ExecutionFailed,
    FeeCalculationError,
}

impl std::fmt::Display for CrossChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrossChainError::InvalidSourceChain => write!(f, "Invalid source chain"),
            CrossChainError::InvalidDestinationChain => write!(f, "Invalid destination chain"),
            CrossChainError::InvalidSender => write!(f, "Invalid sender"),
            CrossChainError::InvalidRecipient => write!(f, "Invalid recipient"),
            CrossChainError::UnsupportedChain => write!(f, "Unsupported chain"),
            CrossChainError::MessageNotFound => write!(f, "Message not found"),
            CrossChainError::MessageNotVerified => write!(f, "Message not verified"),
            CrossChainError::InsufficientValidators => write!(f, "Insufficient validators"),
            CrossChainError::InvalidProof => write!(f, "Invalid proof"),
            CrossChainError::ExecutionFailed => write!(f, "Execution failed"),
            CrossChainError::FeeCalculationError => write!(f, "Fee calculation error"),
        }
    }
}

impl std::error::Error for CrossChainError {}

// 跨链治理
pub struct CrossChainGovernance {
    pub bridge_id: String,
    pub proposals: Vec<CrossChainProposal>,
    pub voting_power: HashMap<String, u64>,
    pub min_stake_for_proposal: u64,
    pub voting_period: u64,
    pub approval_threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: CrossChainProposalType,
    pub proposer: String,
    pub created_at: u64,
    pub voting_deadline: u64,
    pub executed: bool,
    pub passed: bool,
    pub votes_for: u64,
    pub votes_against: u64,
    pub total_stake_voted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossChainProposalType {
    AddSupportedAsset(String),
    RemoveSupportedAsset(String),
    ChangeFeeStructure(FeeStructure),
    UpdateSecurityThreshold(u64),
    AddRelayer(String),
    RemoveRelayer(String),
    BridgeUpgrade(String),
}

impl CrossChainGovernance {
    pub fn new(bridge_id: String) -> Self {
        CrossChainGovernance {
            bridge_id,
            proposals: Vec::new(),
            voting_power: HashMap::new(),
            min_stake_for_proposal: 1000,
            voting_period: 7 * 24 * 3600, // 7天
            approval_threshold: 67, // 67%通过
        }
    }

    // 创建跨链治理提案
    pub fn create_proposal(&mut self, proposal: CrossChainProposal, proposer_stake: u64) -> Result<String, CrossChainError> {
        if proposer_stake < self.min_stake_for_proposal {
            return Err(CrossChainError::FeeCalculationError); // 重用错误类型
        }

        let proposal_id = format!("crosschain_proposal_{}", self.proposals.len() + 1);
        let mut new_proposal = proposal;
        new_proposal.id = proposal_id.clone();
        new_proposal.created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        new_proposal.voting_deadline = new_proposal.created_at + self.voting_period;
        new_proposal.executed = false;
        new_proposal.passed = false;
        new_proposal.votes_for = 0;
        new_proposal.votes_against = 0;
        new_proposal.total_stake_voted = 0;

        self.proposals.push(new_proposal);

        Ok(proposal_id)
    }

    // 对跨链提案投票
    pub fn vote_on_proposal(&mut self, proposal_id: &str, voter: String, vote_for: bool, stake: u64) -> Result<(), CrossChainError> {
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id)
            .ok_or(CrossChainError::MessageNotFound)?;

        if vote_for {
            proposal.votes_for += stake;
        } else {
            proposal.votes_against += stake;
        }
        proposal.total_stake_voted += stake;

        // 更新投票权
        *self.voting_power.entry(voter).or_insert(0) += stake;

        Ok(())
    }

    // 计算提案结果
    pub fn compute_proposal_result(&self, proposal_id: &str) -> Result<bool, CrossChainError> {
        let proposal = self.proposals.iter().find(|p| p.id == proposal_id)
            .ok_or(CrossChainError::MessageNotFound)?;

        if proposal.total_stake_voted == 0 {
            return Ok(false);
        }

        let approval_ratio = (proposal.votes_for * 100) / proposal.total_stake_voted;
        Ok(approval_ratio >= self.approval_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StateStore;

    #[test]
    fn test_cross_chain_bridge() {
        let state_store = StateStore::new();
        let mut bridge = CrossChainBridge::new(CrossChainBridgeType::LockAndMint, state_store);

        // 添加验证节点
        bridge.add_validator(CrossChainValidator {
            id: "validator1".to_string(),
            public_key: "pubkey1".to_string(),
            stake: 1000,
            reputation: 100,
            supported_chains: vec!["ethereum".to_string(), "civitasos".to_string()],
            last_active: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        bridge.add_validator(CrossChainValidator {
            id: "validator2".to_string(),
            public_key: "pubkey2".to_string(),
            stake: 1000,
            reputation: 100,
            supported_chains: vec!["bitcoin".to_string(), "civitasos".to_string()],
            last_active: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        println!("✓ Created cross-chain bridge with {} validators", bridge.get_validator_count());

        // 创建跨链消息
        let message = CrossChainMessage {
            id: "".to_string(), // 将由函数设置
            source_chain: "ethereum".to_string(),
            destination_chain: "civitasos".to_string(),
            sender: "user1".to_string(),
            recipient: "user2".to_string(),
            payload: "transfer_data".to_string(),
            nonce: 1,
            timestamp: 0, // 将由函数设置
            signatures: vec!["sig1".to_string(), "sig2".to_string()],
            status: CrossChainStatus::Pending,
            relayer: Some("relayer1".to_string()),
        };

        // 发送消息
        let message_id = bridge.send_message(message).unwrap();
        assert!(!message_id.is_empty());
        println!("✓ Sent cross-chain message: {}", message_id);

        // 验证消息
        let verification_result = bridge.verify_message(&message_id);
        assert!(verification_result.is_ok());
        println!("✓ Verified cross-chain message");

        // 检查消息状态
        assert_eq!(bridge.get_pending_message_count(), 0);
        assert_eq!(bridge.get_processed_message_count(), 1);
        println!("✓ Message moved from pending to processed");

        // 创建跨链事务
        let tx = CrossChainTransaction {
            message_id: message_id.clone(),
            transaction_type: CrossChainTxType::Transfer,
            amount: 1000,
            asset: "ETH".to_string(),
            fees: 10,
            gas_limit: 21000,
            execution_proof: Some("proof_data".to_string()),
        };

        // 执行事务
        let execution_result = bridge.execute_transaction(tx);
        assert!(execution_result.is_ok());
        println!("✓ Executed cross-chain transaction");

        println!("🎉 Cross-chain bridge test passed!");
    }

    #[test]
    fn test_cross_chain_governance() {
        let mut gov = CrossChainGovernance::new("bridge1".to_string());

        // 创建治理提案
        let proposal = CrossChainProposal {
            id: "".to_string(),
            title: "Add Support for NEW Token".to_string(),
            description: "Add support for NEW token on cross-chain bridge".to_string(),
            proposal_type: CrossChainProposalType::AddSupportedAsset("NEW".to_string()),
            proposer: "validator1".to_string(),
            created_at: 0, // 将由函数设置
            voting_deadline: 0, // 将由函数设置
            executed: false,
            passed: false,
            votes_for: 0,
            votes_against: 0,
            total_stake_voted: 0,
        };

        let proposal_id = gov.create_proposal(proposal, 1500).unwrap();
        assert!(!proposal_id.is_empty());
        println!("✓ Created cross-chain governance proposal: {}", proposal_id);

        // 投票
        let vote_result = gov.vote_on_proposal(&proposal_id, "validator1".to_string(), true, 1000);
        assert!(vote_result.is_ok());
        println!("✓ Cast vote on proposal");

        // 计算结果
        let result = gov.compute_proposal_result(&proposal_id).unwrap();
        println!("✓ Proposal result: {}", result);
    }

    #[test]
    fn test_fee_calculation() {
        let state_store = StateStore::new();
        let bridge = CrossChainBridge::new(CrossChainBridgeType::LockAndMint, state_store);

        let message = CrossChainMessage {
            id: "test".to_string(),
            source_chain: "ethereum".to_string(),
            destination_chain: "civitasos".to_string(),
            sender: "user1".to_string(),
            recipient: "user2".to_string(),
            payload: "a".repeat(2048), // 2KB payload
            nonce: 1,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signatures: vec![],
            status: CrossChainStatus::Pending,
            relayer: None,
        };

        let fee = bridge.compute_fee(&message);
        println!("✓ Calculated fee for 2KB message: {}", fee);
        
        // 基本费用(100) + 大小费用(2*10) + 百分比费用
        assert!(fee >= 100 + 20); // 至少基本费用+大小费用
    }
}