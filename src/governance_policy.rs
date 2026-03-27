use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::consensus::Validator;
use crate::governance::{GovernanceProposal, GovernanceProposalType};

// 治理策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub policy_type: PolicyType,
    pub parameters: HashMap<String, String>,
    pub effective_date: u64,
    pub expiration_date: Option<u64>,
    pub required_majority: u64, // 需要的多数百分比
    pub quorum_threshold: u64,  // 法定人数阈值
    pub cooldown_period: u64,   // 冷却期（秒）
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyType {
    VotingRules,
    TreasuryManagement,
    ValidatorSelection,
    EmergencyProtocols,
    ParameterAdjustment,
    UpgradeProcesses,
}

// 治理策略引擎
pub struct GovernancePolicyEngine {
    pub policies: Vec<GovernancePolicy>,
    pub policy_history: Vec<PolicyChangeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeRecord {
    pub policy_id: String,
    pub old_values: HashMap<String, String>,
    pub new_values: HashMap<String, String>,
    pub changer: String,
    pub timestamp: u64,
    pub justification: String,
}

impl Default for GovernancePolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernancePolicyEngine {
    pub fn new() -> Self {
        GovernancePolicyEngine {
            policies: vec![
                // 默认策略
                GovernancePolicy {
                    id: "default_voting".to_string(),
                    name: "Default Voting Rules".to_string(),
                    description: "Standard voting procedures".to_string(),
                    policy_type: PolicyType::VotingRules,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("quorum_percentage".to_string(), "20".to_string());
                        params.insert("approval_threshold".to_string(), "67".to_string());
                        params.insert("voting_period_seconds".to_string(), "604800".to_string()); // 7天
                        params
                    },
                    effective_date: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    expiration_date: None,
                    required_majority: 67,
                    quorum_threshold: 20,
                    cooldown_period: 24 * 3600, // 24小时冷却期
                },
            ],
            policy_history: Vec::new(),
        }
    }

    // 评估提案是否符合策略
    pub fn evaluate_proposal_compliance(
        &self,
        proposal: &GovernanceProposal,
        validators: &[Validator],
    ) -> ProposalComplianceReport {
        let mut report = ProposalComplianceReport {
            compliant: true,
            issues: Vec::new(),
            recommendations: Vec::new(),
            risk_level: RiskLevel::Low,
        };

        // 检查提案类型是否符合策略
        match &proposal.proposal_type {
            GovernanceProposalType::ConstitutionalAmendment(_) => {
                // 宪法修正案需要特殊处理
                if proposal.approval_percentage < 75 {
                    report.compliant = false;
                    report
                        .issues
                        .push("Constitutional amendment requires 75% approval".to_string());
                    report.risk_level = RiskLevel::High;
                }
            }
            GovernanceProposalType::EmergencyBrake(_) => {
                // 紧急制动需要特殊验证
                if validators.len() < 3 {
                    report.compliant = false;
                    report
                        .issues
                        .push("Insufficient validators for emergency procedures".to_string());
                    report.risk_level = RiskLevel::High;
                }
            }
            _ => {
                // 检查常规提案的合规性
                if proposal.approval_percentage < 50 {
                    report.compliant = false;
                    report
                        .issues
                        .push("Proposal has low approval percentage".to_string());
                    if proposal.approval_percentage < 25 {
                        report.risk_level = RiskLevel::High;
                    } else {
                        report.risk_level = RiskLevel::Medium;
                    }
                }
            }
        }

        // 检查抵押阈值
        if proposal.proposer_stake < proposal.minimum_stake_threshold {
            report.compliant = false;
            report
                .issues
                .push("Insufficient proposer stake".to_string());
        }

        // 检查法定人数
        if !proposal.quorum_met {
            report.compliant = false;
            report.issues.push("Quorum not met".to_string());
        }

        report
    }

    // 应用政策变更
    pub fn apply_policy_change(
        &mut self,
        policy_id: &str,
        new_params: HashMap<String, String>,
        changer: String,
        justification: String,
    ) -> Result<(), PolicyError> {
        // 找到策略
        let policy_index = self
            .policies
            .iter()
            .position(|p| p.id == policy_id)
            .ok_or(PolicyError::PolicyNotFound)?;

        // 保存旧值
        let old_values = self.policies[policy_index].parameters.clone();

        // 更新参数
        let mut updated_policy = self.policies[policy_index].clone();
        for (key, value) in new_params {
            updated_policy.parameters.insert(key, value);
        }

        // 记录变更
        let change_record = PolicyChangeRecord {
            policy_id: policy_id.to_string(),
            old_values,
            new_values: updated_policy.parameters.clone(),
            changer,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            justification,
        };

        self.policy_history.push(change_record);
        self.policies[policy_index] = updated_policy;

        Ok(())
    }

    // 获取策略
    pub fn get_policy(&self, policy_id: &str) -> Option<&GovernancePolicy> {
        self.policies.iter().find(|p| p.id == policy_id)
    }

    // 获取所有策略
    pub fn get_all_policies(&self) -> &[GovernancePolicy] {
        &self.policies
    }
}

// 合规报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalComplianceReport {
    pub compliant: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug)]
pub enum PolicyError {
    PolicyNotFound,
    InsufficientPermissions,
    InvalidParameters,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::PolicyNotFound => write!(f, "Policy not found"),
            PolicyError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            PolicyError::InvalidParameters => write!(f, "Invalid parameters"),
        }
    }
}

impl std::error::Error for PolicyError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::Validator;
    use crate::governance::{GovernanceProposal, GovernanceProposalType};

    #[test]
    fn test_policy_engine() {
        let mut policy_engine = GovernancePolicyEngine::new();

        // 检查默认策略
        assert!(!policy_engine.policies.is_empty());

        // 获取默认投票策略
        let voting_policy = policy_engine.get_policy("default_voting").unwrap();
        assert_eq!(voting_policy.policy_type, PolicyType::VotingRules);

        // 创建一个测试提案
        let test_proposal = GovernanceProposal {
            id: "test_prop".to_string(),
            title: "Test Proposal".to_string(),
            description: "A test proposal".to_string(),
            proposal_type: GovernanceProposalType::ParameterChange("test".to_string()),
            proposer: "validator1".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            voting_deadline: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 1000,
            executed: false,
            passed: false,
            votes_for: 1000,
            votes_against: 500,
            total_stake_voted: 1500,
            minimum_stake_threshold: 1000,
            quorum_met: true,
            approval_percentage: 67,
            proposer_stake: 2000,
        };

        // 创建测试验证节点
        let validators = vec![Validator {
            id: "validator1".to_string(),
            public_key: "pubkey1".to_string(),
            stake: 1000,
            reputation: 100,
        }];

        // 评估提案合规性
        let compliance_report =
            policy_engine.evaluate_proposal_compliance(&test_proposal, &validators);
        assert!(compliance_report.compliant);
        assert_eq!(compliance_report.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_policy_change() {
        let mut policy_engine = GovernancePolicyEngine::new();

        // 修改参数
        let mut new_params = HashMap::new();
        new_params.insert("quorum_percentage".to_string(), "25".to_string());

        let result = policy_engine.apply_policy_change(
            "default_voting",
            new_params,
            "changer1".to_string(),
            "Testing policy change".to_string(),
        );

        assert!(result.is_ok());

        // 检查策略是否更新
        let updated_policy = policy_engine.get_policy("default_voting").unwrap();
        assert_eq!(
            updated_policy.parameters.get("quorum_percentage").unwrap(),
            "25"
        );

        // 检查历史记录
        assert!(!policy_engine.policy_history.is_empty());
        let record = &policy_engine.policy_history[0];
        assert_eq!(record.policy_id, "default_voting");
        assert_eq!(record.changer, "changer1");
    }
}
