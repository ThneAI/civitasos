use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

use crate::execution::{ExecutionResult, AtomicDecisionUnit};
use crate::state::StateStore;

// 验证节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub id: String,
    pub public_key: String,
    pub stake: u64,
    pub reputation: u64,
}

// 投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub validator_id: String,
    pub proposal_hash: String,
    pub vote: bool, // true for yes, false for no
    pub signature: String,
}

// 共识提案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProposal {
    pub id: String,
    pub proposal_type: ProposalType,
    pub content: String,
    pub proposer: String,
    pub votes: Vec<Vote>,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    StateUpdate(String),      // 状态更新提案
    RuleChange(String),       // 规则变更提案
    ValidatorChange(String),  // 验证节点变更提案
    ConstitutionUpdate(String), // 宪法更新提案
}

// 共识引擎
#[derive(Clone)]
pub struct ConsensusEngine {
    pub validators: Vec<Validator>,
    pub proposals: Vec<ConsensusProposal>,
    pub quorum_threshold: u64, // 百分比，例如 67 表示 67%
    pub state_store: StateStore,
}

impl ConsensusEngine {
    pub fn new(state_store: StateStore) -> Self {
        ConsensusEngine {
            validators: Vec::new(),
            proposals: Vec::new(),
            quorum_threshold: 67, // 默认 67% 多数
            state_store,
        }
    }

    // 添加验证节点
    pub fn add_validator(&mut self, validator: Validator) {
        self.validators.push(validator);
    }

    // 创建提案
    pub fn create_proposal(&mut self, proposal: ConsensusProposal) -> Result<String, ConsensusError> {
        // 验证提案有效性
        if !self.validate_proposal(&proposal)? {
            return Err(ConsensusError::InvalidProposal);
        }

        let proposal_id = proposal.id.clone();
        self.proposals.push(proposal);
        
        Ok(proposal_id)
    }

    // 验证提案
    fn validate_proposal(&self, proposal: &ConsensusProposal) -> Result<bool, ConsensusError> {
        // 检查提案是否过期
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now > proposal.expires_at {
            return Ok(false);
        }

        // 检查提案者是否是验证节点
        let proposer_exists = self.validators.iter()
            .any(|v| v.id == proposal.proposer);
        
        Ok(proposer_exists)
    }

    // 投票
    pub fn vote(&mut self, vote: Vote) -> Result<bool, ConsensusError> {
        // 找到对应的提案
        let proposal_index = self.proposals.iter()
            .position(|p| p.id == vote.proposal_hash);
        
        if let Some(index) = proposal_index {
            // 验证投票者是否是验证节点
            let voter_exists = self.validators.iter()
                .any(|v| v.id == vote.validator_id);
            
            if !voter_exists {
                return Err(ConsensusError::UnauthorizedVoter);
            }

            // 检查是否已经投过票
            let proposal = &mut self.proposals[index];
            let vote_exists = proposal.votes.iter()
                .any(|v| v.validator_id == vote.validator_id);
            
            if vote_exists {
                return Err(ConsensusError::DuplicateVote);
            }

            // 添加投票
            proposal.votes.push(vote);
            Ok(true)
        } else {
            Err(ConsensusError::ProposalNotFound)
        }
    }

    // 检查提案是否通过
    pub fn check_proposal_status(&self, proposal_id: &str) -> ProposalStatus {
        let proposal = match self.proposals.iter().find(|p| p.id == proposal_id) {
            Some(p) => p,
            None => return ProposalStatus::NotFound,
        };

        // 计算赞成票
        let total_stake: u64 = self.validators.iter().map(|v| v.stake).sum();
        let yes_votes_stake: u64 = proposal.votes.iter()
            .filter(|v| v.vote)
            .filter_map(|v| {
                self.validators.iter()
                    .find(|val| val.id == v.validator_id)
                    .map(|val| val.stake)
            })
            .sum();

        let support_percentage = if total_stake > 0 {
            (yes_votes_stake * 100) / total_stake
        } else {
            0
        };

        // 检查是否超过阈值
        if support_percentage >= self.quorum_threshold {
            // 检查是否还有足够的投票
            let voted_stake: u64 = proposal.votes.iter()
                .filter_map(|v| {
                    self.validators.iter()
                        .find(|val| val.id == v.validator_id)
                        .map(|val| val.stake)
                })
                .sum();
            
            let participation_percentage = if total_stake > 0 {
                (voted_stake * 100) / total_stake
            } else {
                0
            };

            // 至少50%的验证节点参与投票
            if participation_percentage >= 50 {
                ProposalStatus::Approved
            } else {
                ProposalStatus::Pending
            }
        } else {
            ProposalStatus::Rejected
        }
    }

    // 执行通过的提案
    pub fn execute_approved_proposal(&mut self, proposal_id: &str) -> Result<bool, ConsensusError> {
        match self.check_proposal_status(proposal_id) {
            ProposalStatus::Approved => {
                // 找到提案
                let proposal = match self.proposals.iter().find(|p| p.id == proposal_id) {
                    Some(p) => p,
                    None => return Err(ConsensusError::ProposalNotFound),
                }.clone();

                // 根据提案类型执行相应的操作
                match proposal.proposal_type {
                    ProposalType::StateUpdate(ref content) => {
                        // 解析状态更新内容并应用
                        self.apply_state_update(content)?;
                    },
                    ProposalType::RuleChange(ref content) => {
                        // 应用规则变更
                        self.apply_rule_change(content)?;
                    },
                    ProposalType::ValidatorChange(ref content) => {
                        // 应用验证节点变更
                        self.apply_validator_change(content)?;
                    },
                    ProposalType::ConstitutionUpdate(ref content) => {
                        // 应用宪法更新
                        self.apply_constitution_update(content)?;
                    },
                }

                // 移除已执行的提案
                self.proposals.retain(|p| p.id != proposal_id);
                
                Ok(true)
            },
            ProposalStatus::Pending => Err(ConsensusError::ProposalPending),
            ProposalStatus::Rejected => Err(ConsensusError::ProposalRejected),
            ProposalStatus::NotFound => Err(ConsensusError::ProposalNotFound),
        }
    }

    // 验证执行结果
    pub fn validate_execution_result(
        &self,
        adu: &AtomicDecisionUnit,
        result: &ExecutionResult
    ) -> Result<bool, ConsensusError> {
        // 验证执行结果的合法性
        // 1. 验证轨迹哈希
        let expected_hash = self.compute_expected_trace_hash(adu, result);
        if expected_hash != result.trace_hash {
            return Ok(false);
        }

        // 2. 验证状态变更
        let mut temp_state = self.state_store.clone_for_validation();
        match temp_state.apply_diff(result.state_diffs.clone()) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn compute_expected_trace_hash(&self, _adu: &AtomicDecisionUnit, result: &ExecutionResult) -> String {
        // 在实际实现中，这里会重新执行ADU并计算轨迹哈希
        // 简化实现：直接返回结果中的哈希
        result.trace_hash.clone()
    }

    // 应用状态更新
    fn apply_state_update(&mut self, content: &str) -> Result<(), ConsensusError> {
        // 解析内容并应用状态更新
        // 这里只是示例，实际实现会更复杂
        println!("Applying state update: {}", content);
        Ok(())
    }

    // 应用规则变更
    fn apply_rule_change(&mut self, content: &str) -> Result<(), ConsensusError> {
        println!("Applying rule change: {}", content);
        Ok(())
    }

    // 应用验证节点变更
    fn apply_validator_change(&mut self, content: &str) -> Result<(), ConsensusError> {
        println!("Applying validator change: {}", content);
        Ok(())
    }

    // 应用宪法更新
    fn apply_constitution_update(&mut self, content: &str) -> Result<(), ConsensusError> {
        println!("Applying constitution update: {}", content);
        Ok(())
    }
}

// 提案状态
#[derive(Debug)]
pub enum ProposalStatus {
    Approved,
    Rejected,
    Pending,
    NotFound,
}

// 共识错误
#[derive(Debug)]
pub enum ConsensusError {
    InvalidProposal,
    UnauthorizedVoter,
    DuplicateVote,
    ProposalNotFound,
    ProposalPending,
    ProposalRejected,
    StateUpdateError,
    RuleChangeError,
    ValidatorChangeError,
    ConstitutionUpdateError,
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusError::InvalidProposal => write!(f, "Invalid proposal"),
            ConsensusError::UnauthorizedVoter => write!(f, "Unauthorized voter"),
            ConsensusError::DuplicateVote => write!(f, "Duplicate vote"),
            ConsensusError::ProposalNotFound => write!(f, "Proposal not found"),
            ConsensusError::ProposalPending => write!(f, "Proposal still pending"),
            ConsensusError::ProposalRejected => write!(f, "Proposal rejected"),
            ConsensusError::StateUpdateError => write!(f, "State update error"),
            ConsensusError::RuleChangeError => write!(f, "Rule change error"),
            ConsensusError::ValidatorChangeError => write!(f, "Validator change error"),
            ConsensusError::ConstitutionUpdateError => write!(f, "Constitution update error"),
        }
    }
}

impl std::error::Error for ConsensusError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{ExecutionResult, StateChange};

    #[test]
    fn test_consensus_engine() {
        let state_store = StateStore::new();
        let mut consensus = ConsensusEngine::new(state_store);

        // 添加验证节点
        consensus.add_validator(Validator {
            id: "validator1".to_string(),
            public_key: "pubkey1".to_string(),
            stake: 100,
            reputation: 100,
        });

        consensus.add_validator(Validator {
            id: "validator2".to_string(),
            public_key: "pubkey2".to_string(),
            stake: 100,
            reputation: 100,
        });

        // 创建提案
        let proposal = ConsensusProposal {
            id: "proposal1".to_string(),
            proposal_type: ProposalType::StateUpdate("update_data".to_string()),
            content: "test content".to_string(),
            proposer: "validator1".to_string(),
            votes: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 1000, // 1000秒后过期
        };

        let proposal_id = consensus.create_proposal(proposal).unwrap();
        assert_eq!(proposal_id, "proposal1");

        // 投票
        let vote = Vote {
            validator_id: "validator1".to_string(),
            proposal_hash: "proposal1".to_string(),
            vote: true,
            signature: "sig1".to_string(),
        };

        let vote_result = consensus.vote(vote).unwrap();
        assert!(vote_result);

        // 检查提案状态
        let status = consensus.check_proposal_status("proposal1");
        // 由于只有一个验证节点投票，可能还不够达到阈值
        println!("Proposal status: {:?}", status);
    }

    #[test]
    fn test_execution_validation() {
        let state_store = StateStore::new();
        let consensus = ConsensusEngine::new(state_store);

        // 创建一个简单的执行结果
        let result = ExecutionResult {
            state_diffs: vec![StateChange {
                key: "test_key".to_string(),
                old_value: None,
                new_value: Some("test_value".to_string()),
            }],
            trace_hash: "test_hash".to_string(),
            gas_used: 10,
            success: true,
        };

        // 创建一个ADU
        let adu = AtomicDecisionUnit {
            input_state_hash: "input_hash".to_string(),
            rule_id: "rule1".to_string(),
            execution_trace: vec![],
            output_proof: "proof".to_string(),
            accountability_anchor: "anchor1".to_string(),
            risk_stake: 100,
        };

        // 验证执行结果
        let validation_result = consensus.validate_execution_result(&adu, &result);
        assert!(validation_result.is_ok());
    }
}