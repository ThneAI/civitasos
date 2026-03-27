use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::consensus::ConsensusEngine;

// 宪法规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String, // 规则的具体内容
    pub version: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
}

// 治理提案
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GovernanceProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: GovernanceProposalType,
    pub proposer: String,
    pub created_at: u64,
    pub voting_deadline: u64,
    pub executed: bool,
    pub passed: bool,
    pub votes_for: u64,
    pub votes_against: u64,
    pub total_stake_voted: u64,
    pub minimum_stake_threshold: u64, // 最低抵押阈值
    pub quorum_met: bool,             // 是否达到法定人数
    pub approval_percentage: u64,     // 通过百分比
    pub proposer_stake: u64,          // 提案者抵押
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GovernanceProposalType {
    ConstitutionalAmendment(String), // 宪法修正案
    ParameterChange(String),         // 参数变更
    ValidatorManagement(String),     // 验证节点管理
    EmergencyBrake(bool),            // 紧急制动
    UpgradeProtocol(String),         // 升级协议
    BudgetAllocation(String),        // 预算分配
    PolicyChange(String),            // 政策变更
    RiskAssessment(String),          // 风险评估
}

// 治理引擎
pub struct GovernanceEngine {
    pub constitution: Vec<ConstitutionalRule>,
    pub proposals: Vec<GovernanceProposal>,
    pub voting_power: HashMap<String, u64>, // 投票权映射
    pub consensus_engine: ConsensusEngine,
    pub min_stake_for_proposal: u64, // 提案所需的最低抵押
    pub voting_period: u64,          // 投票期（秒）
    pub quorum_percentage: u64,      // 法定人数百分比
    pub approval_threshold: u64,     // 通过所需百分比
}

impl GovernanceEngine {
    pub fn new(consensus_engine: ConsensusEngine) -> Self {
        GovernanceEngine {
            constitution: vec![
                ConstitutionalRule {
                    id: "rule_0".to_string(),
                    name: "Basic Execution Rule".to_string(),
                    description: "All executions must follow basic safety rules".to_string(),
                    content: "F(State, Input) -> NewState where StateChange is verified".to_string(),
                    version: 1,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    updated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    active: true,
                }
            ],
            proposals: Vec::new(),
            voting_power: HashMap::new(),
            consensus_engine,
            min_stake_for_proposal: 1000, // 最低1000个单位的抵押
            voting_period: 7 * 24 * 3600, // 7天投票期
            quorum_percentage: 20,        // 20%法定人数
            approval_threshold: 67,       // 67%通过
        }
    }
    
    // 创建提案
    pub fn create_proposal(&mut self, proposal: GovernanceProposal, proposer_stake: u64) -> Result<String, GovernanceError> {
        // 检查提案者是否有足够的抵押
        if proposer_stake < self.min_stake_for_proposal {
            return Err(GovernanceError::InsufficientStake);
        }

        // 检查提案是否符合宪法
        if !self.is_action_constitutional(&proposal.title) {
            return Err(GovernanceError::InvalidProposal);
        }

        // 检查提案者是否有权限提交此类型提案
        if !self.can_submit_proposal(&proposal.proposer, &proposal.proposal_type)? {
            return Err(GovernanceError::InvalidProposal);
        }

        // 设置截止日期
        let proposal_id = format!("gov_proposal_{}", self.proposals.len() + 1);
        
        let mut new_proposal = proposal;
        new_proposal.id = proposal_id.clone();
        new_proposal.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        new_proposal.voting_deadline = new_proposal.created_at + self.voting_period;
        new_proposal.proposer_stake = proposer_stake;
        new_proposal.minimum_stake_threshold = self.min_stake_for_proposal;

        // 添加提案
        self.proposals.push(new_proposal);

        Ok(proposal_id)
    }

    // 计算提案结果
    pub fn compute_proposal_result(&mut self, proposal_id: &str) -> Result<ProposalOutcome, GovernanceError> {
        // 首先获取提案的只读信息
        let proposal_info = self.proposals.iter()
            .find(|p| p.id == proposal_id)
            .cloned()
            .ok_or(GovernanceError::ProposalNotFound)?;

        // 检查是否到达投票截止时间
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now <= proposal_info.voting_deadline {
            return Err(GovernanceError::VotingPeriodNotEnded);
        }

        // 计算总抵押
        let total_active_stake = 10000; // 在实际实现中，这应该是系统中所有活跃抵押的总数
        let participation_percentage = (proposal_info.total_stake_voted * 100) / total_active_stake;

        // 检查是否达到法定人数
        let is_quorum_met = participation_percentage >= self.quorum_percentage;

        // 计算支持率
        let support_percentage = if proposal_info.votes_for + proposal_info.votes_against > 0 {
            (proposal_info.votes_for * 100) / (proposal_info.votes_for + proposal_info.votes_against)
        } else {
            0
        };

        // 检查是否超过通过阈值
        let is_approved = if is_quorum_met && support_percentage >= self.approval_threshold {
            true
        } else {
            false
        };

        // 现在更新提案状态
        if let Some(proposal) = self.proposals.iter_mut().find(|p| p.id == proposal_id) {
            proposal.passed = is_approved;
            proposal.quorum_met = is_quorum_met;
            proposal.approval_percentage = support_percentage;
        }

        if is_approved {
            Ok(ProposalOutcome::Passed)
        } else {
            Ok(ProposalOutcome::Defeated)
        }
    }

    // 执行通过的提案
    pub fn execute_passed_proposals(&mut self) -> Result<Vec<String>, GovernanceError> {
        let mut executed_proposals = Vec::new();

        // 首先收集所有需要执行的提案ID及信息
        let proposals_to_execute: Vec<(String, GovernanceProposalType)> = self.proposals.iter()
            .filter(|p| !p.executed && p.passed)
            .map(|p| (p.id.clone(), p.proposal_type.clone()))
            .collect();

        // 然后逐个执行
        for (proposal_id, proposal_type) in proposals_to_execute {
            // 检查是否已到执行时间
            let outcome = self.compute_proposal_result(&proposal_id);
            match outcome {
                Ok(ProposalOutcome::Passed) => {
                    // 根据提案类型执行相应操作
                    match &proposal_type {
                        GovernanceProposalType::ConstitutionalAmendment(content) => {
                            self.update_constitution(content, &proposal_id)?;
                        },
                        GovernanceProposalType::ParameterChange(content) => {
                            self.update_parameters(content)?;
                        },
                        GovernanceProposalType::ValidatorManagement(content) => {
                            self.manage_validators(content)?;
                        },
                        GovernanceProposalType::EmergencyBrake(active) => {
                            self.activate_emergency_brake(*active)?;
                        },
                        GovernanceProposalType::UpgradeProtocol(content) => {
                            self.initiate_protocol_upgrade(content)?;
                        },
                        GovernanceProposalType::BudgetAllocation(content) => {
                            self.handle_budget_allocation(content)?;
                        },
                        GovernanceProposalType::PolicyChange(content) => {
                            self.handle_policy_change(content)?;
                        },
                        GovernanceProposalType::RiskAssessment(content) => {
                            self.handle_risk_assessment(content)?;
                        },
                    }

                    // 标记为已执行
                    if let Some(proposal) = self.proposals.iter_mut().find(|p| p.id == proposal_id) {
                        proposal.executed = true;
                    }
                    executed_proposals.push(proposal_id);
                },
                _ => continue, // 未通过的提案跳过
            }
        }

        Ok(executed_proposals)
    }

    // 添加宪法规则
    pub fn add_constitutional_rule(&mut self, rule: ConstitutionalRule) -> Result<(), GovernanceError> {
        // 在实际实现中，这可能需要通过治理流程
        self.constitution.push(rule);
        Ok(())
    }
    
    // 检查提案是否可以提交
    pub fn can_submit_proposal(&self, proposer: &str, proposal_type: &GovernanceProposalType) -> Result<bool, GovernanceError> {
        // 检查提案者是否是验证节点
        let is_validator = self.consensus_engine.validators.iter()
            .any(|v| v.id == proposer);
        
        if !is_validator {
            return Ok(false);
        }
        
        // 检查提案类型是否有效
        match proposal_type {
            GovernanceProposalType::ConstitutionalAmendment(_) => {
                // 宪法修正案需要更高的权限
                Ok(true)
            },
            GovernanceProposalType::EmergencyBrake(active) => {
                // 紧急制动需要特殊权限
                if *active {
                    // 检查是否是紧急情况
                    Ok(self.is_emergency_condition())
                } else {
                    Ok(true) // 解除紧急状态总是可以的
                }
            },
            GovernanceProposalType::ParameterChange(_) => Ok(true),
            GovernanceProposalType::ValidatorManagement(_) => Ok(true),
            GovernanceProposalType::UpgradeProtocol(_) => Ok(true),
            GovernanceProposalType::BudgetAllocation(_) => Ok(true),
            GovernanceProposalType::PolicyChange(_) => Ok(true),
            GovernanceProposalType::RiskAssessment(_) => Ok(true),
        }
    }
    
    // 检查是否为紧急情况
    fn is_emergency_condition(&self) -> bool {
        // 在实际实现中，这里会检查系统是否处于紧急状态
        // 简化实现：假定不是紧急情况
        false
    }
    
    // 查询提案
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&GovernanceProposal> {
        self.proposals.iter().find(|p| p.id == proposal_id)
    }
    
    // 获取活跃提案
    pub fn get_active_proposals(&self) -> Vec<&GovernanceProposal> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.proposals.iter()
            .filter(|p| !p.executed && p.voting_deadline > now)
            .collect()
    }
    
    // 获取已执行提案
    pub fn get_executed_proposals(&self) -> Vec<&GovernanceProposal> {
        self.proposals.iter()
            .filter(|p| p.executed)
            .collect()
    }
    
    // 获取通过的提案
    pub fn get_passed_proposals(&self) -> Vec<&GovernanceProposal> {
        self.proposals.iter()
            .filter(|p| p.passed)
            .collect()
    }
    
    // 获取失败的提案
    pub fn get_failed_proposals(&self) -> Vec<&GovernanceProposal> {
        self.proposals.iter()
            .filter(|p| !p.passed && p.executed)
            .collect()
    }
    
    // 更新参数
    fn update_parameters(&mut self, content: &str) -> Result<(), GovernanceError> {
        // 在实际实现中，这里会解析并应用参数变更
        // 验证参数变更的安全性
        println!("Updating parameters: {}", content);
        
        // 例如，更新治理参数
        if content.contains("quorum_percentage") {
            if let Ok(percentage) = content.split('=').nth(1).unwrap_or("0").trim().parse::<u64>() {
                if percentage >= 10 && percentage <= 90 {
                    self.quorum_percentage = percentage;
                }
            }
        }
        
        if content.contains("approval_threshold") {
            if let Ok(threshold) = content.split('=').nth(1).unwrap_or("0").trim().parse::<u64>() {
                if threshold >= 51 && threshold <= 95 {
                    self.approval_threshold = threshold;
                }
            }
        }
        
        Ok(())
    }
    
    // 管理验证节点
    fn manage_validators(&mut self, content: &str) -> Result<(), GovernanceError> {
        // 在实际实现中，这里会管理验证节点
        println!("Managing validators: {}", content);
        
        // 例如，添加/移除验证节点
        if content.starts_with("add:") {
            let parts: Vec<&str> = content.split(':').collect();
            if parts.len() >= 4 {
                let id = parts[1];
                let pubkey = parts[2];
                let stake: u64 = parts[3].parse().unwrap_or(0);
                
                let validator = crate::consensus::Validator {
                    id: id.to_string(),
                    public_key: pubkey.to_string(),
                    stake,
                    reputation: 100,
                };
                
                self.consensus_engine.add_validator(validator);
            }
        } else if content.starts_with("remove:") {
            let validator_id = content.trim_start_matches("remove:");
            self.consensus_engine.validators.retain(|v| v.id != validator_id);
        }
        
        Ok(())
    }
    
    // 协议升级
    fn initiate_protocol_upgrade(&mut self, content: &str) -> Result<(), GovernanceError> {
        // 在实际实现中，这里会处理协议升级
        println!("Initiating protocol upgrade: {}", content);
        
        // 协议升级通常需要特殊的验证流程
        // 例如：验证升级包的完整性、兼容性等
        
        Ok(())
    }
    
    // 激活紧急制动
    fn activate_emergency_brake(&mut self, active: bool) -> Result<(), GovernanceError> {
        if active {
            println!("Emergency brake activated!");
            // 在实际实现中，这里会执行紧急制动操作
        } else {
            println!("Emergency brake deactivated!");
        }
        Ok(())
    }
    
    // 更新宪法
    fn update_constitution(&mut self, content: &str, proposal_id: &str) -> Result<(), GovernanceError> {
        // 解析宪法更新内容
        println!("Updating constitution based on proposal {}: {}", proposal_id, content);
        
        // 在实际实现中，这里会解析内容并更新宪法规则
        let new_rule = crate::ConstitutionalRule {
            id: format!("rule_{}_{}", proposal_id, self.constitution.len()),
            name: format!("Updated Rule from Prop {}", proposal_id),
            description: content.to_string(),
            content: content.to_string(),
            version: 1,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            active: true,
        };

        self.constitution.push(new_rule);
        
        Ok(())
    }
    
    // 预算分配
    fn handle_budget_allocation(&mut self, content: &str) -> Result<(), GovernanceError> {
        println!("Handling budget allocation: {}", content);
        Ok(())
    }
    
    // 政策变更
    fn handle_policy_change(&mut self, content: &str) -> Result<(), GovernanceError> {
        println!("Handling policy change: {}", content);
        Ok(())
    }
    
    // 风险评估
    fn handle_risk_assessment(&mut self, content: &str) -> Result<(), GovernanceError> {
        println!("Handling risk assessment: {}", content);
        Ok(())
    }

    // 创建提案
    // 获取活跃的宪法规则
    pub fn get_active_constitutional_rules(&self) -> Vec<&ConstitutionalRule> {
        self.constitution.iter().filter(|rule| rule.active).collect()
    }

    pub fn is_action_constitutional(&self, _action: &str) -> bool {
        // 在实际实现中，这里会检查操作是否违反宪法规则
        // 简化实现：假定所有操作都符合宪法
        true
    }
    
    // 投票
    pub fn vote(&mut self, proposal_id: &str, voter: String, vote_for: bool, stake: u64) -> Result<(), GovernanceError> {
        // 查找提案
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        // 检查投票是否仍在进行中
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now > proposal.voting_deadline {
            return Err(GovernanceError::VotingPeriodEnded);
        }

        // 更新投票统计
        if vote_for {
            proposal.votes_for += stake;
        } else {
            proposal.votes_against += stake;
        }
        proposal.total_stake_voted += stake;

        // 更新投票者权益
        *self.voting_power.entry(voter).or_insert(0) += stake;

        Ok(())
    }
}

// 提案结果
#[derive(Debug, PartialEq)]
pub enum ProposalOutcome {
    Passed,
    Defeated,
    Pending,
}

// 治理错误
#[derive(Debug)]
pub enum GovernanceError {
    InsufficientStake,
    ProposalNotFound,
    VotingPeriodEnded,
    VotingPeriodNotEnded,
    ExecutionFailed,
    InvalidProposal,
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovernanceError::InsufficientStake => write!(f, "Insufficient stake for proposal"),
            GovernanceError::ProposalNotFound => write!(f, "Proposal not found"),
            GovernanceError::VotingPeriodEnded => write!(f, "Voting period has ended"),
            GovernanceError::VotingPeriodNotEnded => write!(f, "Voting period has not ended yet"),
            GovernanceError::ExecutionFailed => write!(f, "Proposal execution failed"),
            GovernanceError::InvalidProposal => write!(f, "Invalid proposal"),
        }
    }
}

impl std::error::Error for GovernanceError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::ConsensusEngine;
    use crate::state::StateStore;

    #[test]
    fn test_governance_engine() {
        let state_store = StateStore::new();
        let mut consensus_engine = ConsensusEngine::new(state_store);

        // 添加验证节点，以便提案可以被创建
        let validator = crate::consensus::Validator {
            id: "validator1".to_string(),
            public_key: "pubkey1".to_string(),
            stake: 1000,
            reputation: 100,
        };
        consensus_engine.add_validator(validator);

        let mut gov_engine = GovernanceEngine::new(consensus_engine);

        // 检查初始宪法规则
        let active_rules = gov_engine.get_active_constitutional_rules();
        assert!(!active_rules.is_empty());

        // 创建一个治理提案
        let proposal = GovernanceProposal {
            id: "".to_string(), // 会被函数设置
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposal_type: GovernanceProposalType::ParameterChange("test_param=test_value".to_string()),
            proposer: "validator1".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            voting_deadline: 0, // 会被函数设置
            executed: false,
            passed: false,
            votes_for: 0,
            votes_against: 0,
            total_stake_voted: 0,
            minimum_stake_threshold: 1000,
            quorum_met: false,
            approval_percentage: 0,
            proposer_stake: 2000,
        };

        // 提案需要足够的抵押才能创建
        let result = gov_engine.create_proposal(proposal, 2000); // 足够的抵押
        assert!(result.is_ok());

        let proposal_id = result.unwrap();
        assert!(!proposal_id.is_empty());

        // 投票
        let vote_result = gov_engine.vote(&proposal_id, "validator1".to_string(), true, 1500);
        assert!(vote_result.is_ok());

        println!("Created proposal: {}", proposal_id);
    }

    #[test]
    fn test_constitutional_check() {
        let state_store = StateStore::new();
        let consensus_engine = ConsensusEngine::new(state_store);
        let gov_engine = GovernanceEngine::new(consensus_engine);

        // 检查操作是否符合宪法
        let is_constitutional = gov_engine.is_action_constitutional("test_action");
        assert!(is_constitutional);
    }
}