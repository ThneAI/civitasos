use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{StateStore, Validator};

// 高级共识类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvancedConsensusType {
    PBFT,           // 实用拜占庭容错
    Tendermint,     // 拜占庭容错共识
    HotStuff,       // 领导者驱动的拜占庭容错
    Optimistic,     // 乐观共识
    Rollup,         // 汇总共识
}

// 共识轮次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub round_number: u64,
    pub height: u64,
    pub proposer: String,
    pub proposed_block: Option<Block>,
    pub votes: HashMap<String, Vote>,
    pub state_root: String,
    pub timestamp: u64,
    pub phase: ConsensusPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusPhase {
    Propose,
    Prevote,
    Precommit,
    Commit,
}

// 区块结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub prev_hash: String,
    pub state_root: String,
    pub transactions: Vec<ConsensusTransaction>,
    pub proposer: String,
    pub signature: String,
    pub timestamp: u64,
    pub validators: Vec<Validator>,
}

// 共识交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusTransaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub data: String,
    pub signature: String,
    pub timestamp: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
}

// 投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub validator_id: String,
    pub round: u64,
    pub height: u64,
    pub block_hash: String,
    pub vote_type: VoteType,
    pub signature: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    Prevote,
    Precommit,
    Commit,
}

// PBFT共识引擎
pub struct PBFTConsensusEngine {
    pub validators: Vec<Validator>,
    pub current_round: u64,
    pub current_height: u64,
    pub state_store: StateStore,
    pub rounds: HashMap<u64, ConsensusRound>,
    pub pending_transactions: Vec<ConsensusTransaction>,
    pub committed_blocks: Vec<Block>,
    pub view_change_timeout: u64,  // 视图变更超时
    pub replica_id: String,        // 副本ID
}

impl PBFTConsensusEngine {
    pub fn new(state_store: StateStore, replica_id: String) -> Self {
        PBFTConsensusEngine {
            validators: Vec::new(),
            current_round: 0,
            current_height: 0,
            state_store,
            rounds: HashMap::new(),
            pending_transactions: Vec::new(),
            committed_blocks: Vec::new(),
            view_change_timeout: 30, // 30秒超时
            replica_id,
        }
    }

    // 添加验证节点
    pub fn add_validator(&mut self, validator: Validator) {
        self.validators.push(validator);
    }

    // 开始共识轮次
    pub fn start_round(&mut self) -> Result<(), ConsensusError> {
        let proposer = self.select_proposer(self.current_round);

        let round = ConsensusRound {
            round_number: self.current_round,
            height: self.current_height,
            proposer: proposer.clone(),
            proposed_block: None,
            votes: HashMap::new(),
            state_root: self.state_store.get_root_hash().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            phase: ConsensusPhase::Propose,
        };

        self.rounds.insert(self.current_round, round);

        // 如果我们是提议者，创建区块
        if proposer == self.replica_id {
            self.propose_block(self.current_round)?;
        }

        Ok(())
    }

    // 提议区块
    fn propose_block(&mut self, round_num: u64) -> Result<(), ConsensusError> {
        // 先创建区块，避免借用冲突
        let block = self.create_block()?;

        // 然后更新轮次信息
        if let Some(round) = self.rounds.get_mut(&round_num) {
            if round.proposed_block.is_none() {
                round.proposed_block = Some(block);
                round.phase = ConsensusPhase::Prevote;
            }
        }

        Ok(())
    }

    // 创建区块
    fn create_block(&mut self) -> Result<Block, ConsensusError> {
        let prev_hash = if self.committed_blocks.is_empty() {
            "genesis".to_string()
        } else {
            self.committed_blocks.last().unwrap().signature.clone()
        };

        // 从待处理交易中选择一部分
        let mut transactions = Vec::new();
        let max_transactions = 10; // 每块最多10笔交易

        for tx in self.pending_transactions.drain(0..std::cmp::min(max_transactions, self.pending_transactions.len())) {
            transactions.push(tx);
        }

        let state_root = self.state_store.get_root_hash().to_string();

        let block = Block {
            height: self.current_height,
            prev_hash: prev_hash.clone(),
            state_root: state_root.clone(),
            transactions,
            proposer: self.replica_id.clone(),
            signature: self.sign_block_data(&format!("{}{}{}", self.current_height, state_root, prev_hash)),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            validators: self.validators.clone(),
        };

        Ok(block)
    }

    // 投票
    pub fn vote(&mut self, round_num: u64, block_hash: String, vote_type: VoteType) -> Result<(), ConsensusError> {
        let validator_id = self.replica_id.clone();
        let current_height = self.current_height;
        let signature_data = format!("{}{}{:?}", round_num, current_height, &validator_id);
        let signature = self.sign_vote_data(&signature_data);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let vote = Vote {
            validator_id: validator_id.clone(),
            round: round_num,
            height: current_height,
            block_hash,
            vote_type,
            signature,
            timestamp,
        };

        if let Some(round) = self.rounds.get_mut(&round_num) {
            round.votes.insert(validator_id, vote);
        } else {
            return Err(ConsensusError::RoundNotFound);
        }

        Ok(())
    }

    // 验证投票
    pub fn validate_vote(&self, vote: &Vote) -> Result<bool, ConsensusError> {
        // 检查验证节点是否在列表中
        let validator_exists = self.validators.iter()
            .any(|v| v.id == vote.validator_id);

        if !validator_exists {
            return Ok(false);
        }

        // 验证签名（简化实现）
        // 在实际实现中，这里会验证签名的有效性
        Ok(true)
    }

    // 检查是否达到多数
    pub fn has_majority(&self, votes: &HashMap<String, Vote>) -> bool {
        let total_validators = self.validators.len();
        let vote_count = votes.len();

        // PBFT需要2f+1个投票，其中f是故障节点数
        // 所以需要超过2/3的投票
        vote_count > (2 * total_validators) / 3
    }

    // 提交区块
    pub fn commit_block(&mut self, block: Block) -> Result<(), ConsensusError> {
        // 验证区块
        if !self.validate_block(&block)? {
            return Err(ConsensusError::InvalidBlock);
        }

        // 应用区块中的交易到状态
        for tx in &block.transactions {
            // 在实际实现中，这里会执行交易
            println!("Executing transaction: {}", tx.id);
        }

        // 更新状态根
        self.state_store.update_root_hash_public();

        // 添加到已提交区块
        self.committed_blocks.push(block);
        self.current_height += 1;
        self.current_round += 1;

        Ok(())
    }

    // 验证区块
    fn validate_block(&self, block: &Block) -> Result<bool, ConsensusError> {
        // 验证前一个哈希
        if !self.committed_blocks.is_empty() {
            let last_block = self.committed_blocks.last().unwrap();
            if block.prev_hash != last_block.signature {
                return Ok(false);
            }
        }

        // 验证签名
        // 在实际实现中，这里会验证区块签名

        // 验证状态根
        // 在实际实现中，这里会验证状态根的有效性

        Ok(true)
    }

    // 选择提议者
    fn select_proposer(&self, round: u64) -> String {
        if self.validators.is_empty() {
            return self.replica_id.clone();
        }

        // 简单的轮询选择
        let index = (round as usize) % self.validators.len();
        self.validators[index].id.clone()
    }

    // 签名区块数据
    fn sign_block_data(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    // 签名投票数据
    fn sign_vote_data(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    // 获取当前高度
    pub fn get_current_height(&self) -> u64 {
        self.current_height
    }

    // 获取已提交的区块数量
    pub fn get_block_count(&self) -> usize {
        self.committed_blocks.len()
    }
}

// 共识错误
#[derive(Debug)]
pub enum ConsensusError {
    RoundNotFound,
    InvalidBlock,
    InvalidVote,
    InsufficientVotes,
    Timeout,
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusError::RoundNotFound => write!(f, "Round not found"),
            ConsensusError::InvalidBlock => write!(f, "Invalid block"),
            ConsensusError::InvalidVote => write!(f, "Invalid vote"),
            ConsensusError::InsufficientVotes => write!(f, "Insufficient votes"),
            ConsensusError::Timeout => write!(f, "Consensus timeout"),
        }
    }
}

impl std::error::Error for ConsensusError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StateStore;

    #[test]
    fn test_pbft_consensus_engine() {
        let state_store = StateStore::new();
        let mut pbft = PBFTConsensusEngine::new(state_store, "replica1".to_string());

        // 添加验证节点
        pbft.add_validator(Validator {
            id: "validator1".to_string(),
            public_key: "pubkey1".to_string(),
            stake: 1000,
            reputation: 100,
        });

        pbft.add_validator(Validator {
            id: "validator2".to_string(),
            public_key: "pubkey2".to_string(),
            stake: 1000,
            reputation: 100,
        });

        pbft.add_validator(Validator {
            id: "validator3".to_string(),
            public_key: "pubkey3".to_string(),
            stake: 1000,
            reputation: 100,
        });

        println!("✓ Created PBFT consensus engine with {} validators", pbft.validators.len());

        // 开始共识轮次
        let result = pbft.start_round();
        assert!(result.is_ok());
        println!("✓ Started consensus round");

        // 检查轮次状态
        assert!(pbft.rounds.contains_key(&0));
        println!("✓ Round 0 created");

        // 检查当前高度
        assert_eq!(pbft.get_current_height(), 0);
        println!("✓ Current height is 0");

        // 添加一些待处理交易
        pbft.pending_transactions.push(ConsensusTransaction {
            id: "tx1".to_string(),
            from: "user1".to_string(),
            to: "user2".to_string(),
            data: "transfer".to_string(),
            signature: "sig1".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            gas_limit: 21000,
            gas_price: 1,
        });

        println!("✓ Added pending transaction");

        // 验证投票功能
        let block_hash = "test_block_hash".to_string();
        let vote_result = pbft.vote(0, block_hash, VoteType::Prevote);
        assert!(vote_result.is_ok());
        println!("✓ Successfully voted");

        // 检查投票数量
        if let Some(round) = pbft.rounds.get(&0) {
            assert_eq!(round.votes.len(), 1);
            println!("✓ Vote recorded in round");
        }

        println!("🎉 PBFT consensus engine test passed!");
    }

    #[test]
    fn test_majority_check() {
        let state_store = StateStore::new();
        let mut pbft = PBFTConsensusEngine::new(state_store, "replica1".to_string());

        // 添加3个验证节点
        for i in 1..=3 {
            pbft.add_validator(Validator {
                id: format!("validator{}", i),
                public_key: format!("pubkey{}", i),
                stake: 1000,
                reputation: 100,
            });
        }

        // 创建2个投票（少于2/3多数）
        let mut votes = HashMap::new();
        votes.insert("validator1".to_string(), Vote {
            validator_id: "validator1".to_string(),
            round: 0,
            height: 0,
            block_hash: "hash".to_string(),
            vote_type: VoteType::Precommit,
            signature: "sig".to_string(),
            timestamp: 0,
        });

        votes.insert("validator2".to_string(), Vote {
            validator_id: "validator2".to_string(),
            round: 0,
            height: 0,
            block_hash: "hash".to_string(),
            vote_type: VoteType::Precommit,
            signature: "sig".to_string(),
            timestamp: 0,
        });

        // 2票对3个验证节点，应该是多数（2 > 2*3/3 = 2）
        // 实际上是2 > 2，这是false
        // 需要是2f+1，即3个中的2个是不够的，需要3个中的3个
        // 不对，3个中的2个是2 > 2*3/3 = 2，即2 > 2是false，所以不是多数
        // 2f+1 = 2*(3-2)+1 = 3，需要全部3票
        // 不对，让我重新计算
        // 对于3个节点，f=1（最多容忍1个故障节点），需要2f+1=3票
        // 2f+1 = 2*1+1 = 3票
        // 所以3个节点需要3票才是多数
        let majority = pbft.has_majority(&votes);
        assert!(!majority); // 2票不是3个节点的多数
        println!("✓ 2 votes not majority for 3 validators: {}", !majority);

        // 添加第3个投票
        votes.insert("validator3".to_string(), Vote {
            validator_id: "validator3".to_string(),
            round: 0,
            height: 0,
            block_hash: "hash".to_string(),
            vote_type: VoteType::Precommit,
            signature: "sig".to_string(),
            timestamp: 0,
        });

        let majority = pbft.has_majority(&votes);
        assert!(majority); // 3票是3个节点的多数
        println!("✓ 3 votes majority for 3 validators: {}", majority);
    }
}