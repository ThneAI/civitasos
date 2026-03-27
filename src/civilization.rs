//! Civilization Layer for CivitasOS
//!
//! This module handles governance, constitutional rules, and value alignment.
//! Layer 5 of the CivitasOS architecture.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the constitutional rules of the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    /// Version of the constitution
    pub version: String,

    /// Core principles
    pub principles: Vec<String>,

    /// Governance mechanisms
    pub governance_mechanisms: Vec<GovernanceMechanism>,

    /// Value functions
    pub value_functions: Vec<ValueFunction>,

    /// Amendment procedures
    pub amendment_procedures: Vec<AmendmentProcedure>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Different types of governance mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceMechanism {
    /// Direct democracy - all agents vote on proposals
    DirectDemocracy,

    /// Representative democracy - elected representatives vote
    RepresentativeDemocracy(DemocracyParams),

    /// Liquid democracy - agents can delegate votes
    LiquidDemocracy(LiquidDemocracyParams),

    /// Quadratic voting system
    QuadraticVoting(QuadraticVotingParams),
}

/// Parameters for democracy systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemocracyParams {
    /// Minimum stake required to vote
    pub min_stake: u64,

    /// Voting period in seconds
    pub voting_period: u64,

    /// Quorum percentage required
    pub quorum_percentage: u8,

    /// Threshold for proposal acceptance
    pub acceptance_threshold: u8,
}

/// Parameters for liquid democracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidDemocracyParams {
    /// Delegation depth limit
    pub delegation_depth: u8,

    /// Minimum stake for delegation
    pub min_delegation_stake: u64,

    /// Voting period in seconds
    pub voting_period: u64,
}

/// Parameters for quadratic voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadraticVotingParams {
    /// Cost coefficient for quadratic voting
    pub cost_coefficient: f64,

    /// Voting period in seconds
    pub voting_period: u64,
}

/// Represents a value function that guides agent behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueFunction {
    /// Name of the value function
    pub name: String,

    /// Description of the value function
    pub description: String,

    /// Weight/importance of this value
    pub weight: f64,

    /// Function definition
    pub function: ValueFunctionDef,
}

/// Definition of a value function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueFunctionDef {
    /// Utilitarian - maximize overall utility
    Utilitarian,

    /// Egalitarian - promote equality
    Egalitarian,

    /// Libertarian - maximize freedom
    Libertarian,

    /// Environmental - prioritize environmental concerns
    Environmental,

    /// Custom value function with specific parameters
    Custom(CustomValueFunction),
}

/// Custom value function with parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomValueFunction {
    /// Function parameters
    pub params: HashMap<String, serde_json::Value>,

    /// Formula for calculating value
    pub formula: String,

    /// Evaluation frequency
    pub evaluation_frequency: u64,
}

/// Amendment procedure for changing the constitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendmentProcedure {
    /// Name of the amendment procedure
    pub name: String,

    /// Description of the procedure
    pub description: String,

    /// Steps required for amendment
    pub steps: Vec<AmendmentStep>,

    /// Required majority for different types of changes
    pub majority_requirements: HashMap<ChangeType, u8>,
}

/// Step in an amendment procedure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmendmentStep {
    /// Proposal stage
    Proposal,

    /// Discussion stage
    Discussion,

    /// Voting stage
    Voting,

    /// Ratification stage
    Ratification,
}

/// Type of constitutional change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangeType {
    /// Minor change to existing provisions
    Minor,

    /// Major change affecting core principles
    Major,

    /// Fundamental change to governance structure
    Fundamental,
}

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: String,

    /// Title of the proposal
    pub title: String,

    /// Description of the proposal
    pub description: String,

    /// Proposed changes
    pub changes: Vec<ProposalChange>,

    /// Author of the proposal
    pub author: String,

    /// Timestamp of proposal creation
    pub created_at: DateTime<Utc>,

    /// Timestamp when voting starts
    pub voting_starts_at: DateTime<Utc>,

    /// Timestamp when voting ends
    pub voting_ends_at: DateTime<Utc>,

    /// Current status of the proposal
    pub status: ProposalStatus,

    /// Vote counts
    pub vote_counts: VoteCounts,

    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of change in a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalChange {
    /// Amendment to constitution
    ConstitutionAmendment(Constitution),

    /// Change to governance mechanism
    GovernanceChange(GovernanceMechanism),

    /// Addition of new value function
    ValueFunctionAddition(ValueFunction),

    /// Removal of existing value function
    ValueFunctionRemoval(String),

    /// Policy change
    PolicyChange(PolicyChange),
}

/// Policy change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChange {
    /// Policy name
    pub name: String,

    /// Old policy value
    pub old_value: serde_json::Value,

    /// New policy value
    pub new_value: serde_json::Value,

    /// Justification for change
    pub justification: String,
}

/// Status of a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal submitted, waiting for discussion
    Submitted,

    /// Proposal in discussion phase
    Discussion,

    /// Proposal in voting phase
    Voting,

    /// Proposal approved
    Approved,

    /// Proposal rejected
    Rejected,

    /// Proposal withdrawn
    Withdrawn,
}

/// Vote counts for a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteCounts {
    /// Yes votes
    pub yes: u64,

    /// No votes
    pub no: u64,

    /// Abstain votes
    pub abstain: u64,

    /// Total votes cast
    pub total: u64,
}

/// Main civilization layer implementation
pub struct CivilizationLayer {
    /// Current constitution
    pub constitution: Constitution,

    /// Active proposals
    pub active_proposals: HashMap<String, Proposal>,

    /// Historical proposals
    pub historical_proposals: Vec<Proposal>,

    /// Value functions currently in effect
    pub active_values: Vec<ValueFunction>,

    /// Agent reputation scores
    pub agent_reputations: HashMap<String, f64>,

    /// Governance parameters
    pub governance_params: GovernanceParams,
}

/// Governance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceParams {
    /// Minimum stake to propose
    pub min_proposal_stake: u64,

    /// Minimum stake to vote
    pub min_voting_stake: u64,

    /// Proposal cooldown period
    pub proposal_cooldown: u64,

    /// Voting reward percentage
    pub voting_reward_percentage: f64,
}

impl CivilizationLayer {
    /// Create a new civilization layer with default constitution
    pub fn new() -> Self {
        Self {
            constitution: Self::default_constitution(),
            active_proposals: HashMap::new(),
            historical_proposals: Vec::new(),
            active_values: Vec::new(),
            agent_reputations: HashMap::new(),
            governance_params: GovernanceParams {
                min_proposal_stake: 1000,
                min_voting_stake: 10,
                proposal_cooldown: 86400,       // 24 hours
                voting_reward_percentage: 0.01, // 1% reward
            },
        }
    }

    /// Create a default constitution
    fn default_constitution() -> Constitution {
        Constitution {
            version: "1.0.0".to_string(),
            principles: vec![
                "Decentralization".to_string(),
                "Transparency".to_string(),
                "Fairness".to_string(),
                "Sustainability".to_string(),
            ],
            governance_mechanisms: vec![GovernanceMechanism::DirectDemocracy],
            value_functions: vec![
                ValueFunction {
                    name: "Cooperation".to_string(),
                    description: "Promote collaborative behavior".to_string(),
                    weight: 0.3,
                    function: ValueFunctionDef::Utilitarian,
                },
                ValueFunction {
                    name: "Efficiency".to_string(),
                    description: "Maximize resource utilization".to_string(),
                    weight: 0.25,
                    function: ValueFunctionDef::Utilitarian,
                },
                ValueFunction {
                    name: "Security".to_string(),
                    description: "Ensure system safety".to_string(),
                    weight: 0.25,
                    function: ValueFunctionDef::Utilitarian,
                },
                ValueFunction {
                    name: "Innovation".to_string(),
                    description: "Encourage novel solutions".to_string(),
                    weight: 0.2,
                    function: ValueFunctionDef::Utilitarian,
                },
            ],
            amendment_procedures: vec![AmendmentProcedure {
                name: "Standard Amendment".to_string(),
                description: "Standard procedure for constitutional changes".to_string(),
                steps: vec![
                    AmendmentStep::Proposal,
                    AmendmentStep::Discussion,
                    AmendmentStep::Voting,
                    AmendmentStep::Ratification,
                ],
                majority_requirements: [
                    (ChangeType::Minor, 51),
                    (ChangeType::Major, 67),
                    (ChangeType::Fundamental, 75),
                ]
                .iter()
                .cloned()
                .collect(),
            }],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Submit a new proposal
    pub fn submit_proposal(&mut self, proposal: Proposal) -> Result<(), String> {
        // Validate proposal
        if proposal.author.is_empty() {
            return Err("Proposal must have an author".to_string());
        }

        if proposal.title.is_empty() {
            return Err("Proposal must have a title".to_string());
        }

        // Check minimum stake requirement
        // In a real implementation, this would check the proposer's stake

        // Set voting period
        let now = Utc::now();
        let proposal_with_schedule = Proposal {
            voting_starts_at: now + chrono::Duration::days(1), // Start voting in 1 day
            voting_ends_at: now + chrono::Duration::days(7),   // End voting in 7 days
            status: ProposalStatus::Submitted,
            ..proposal
        };

        // Insert the proposal
        self.active_proposals
            .insert(proposal_with_schedule.id.clone(), proposal_with_schedule);

        Ok(())
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        _voter: &str,
        vote: VoteType,
        stake: u64,
    ) -> Result<(), String> {
        // Check if proposal exists and is in voting phase
        let proposal = self
            .active_proposals
            .get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if matches!(proposal.status, ProposalStatus::Voting) {
            // In a real implementation, this would record the vote
            // For now, we'll just update the vote counts
            match vote {
                VoteType::Yes => proposal.vote_counts.yes += stake,
                VoteType::No => proposal.vote_counts.no += stake,
                VoteType::Abstain => proposal.vote_counts.abstain += stake,
            }
            proposal.vote_counts.total += stake;

            Ok(())
        } else {
            Err("Proposal is not in voting phase".to_string())
        }
    }

    /// Process proposals whose voting periods have ended
    pub fn process_expired_proposals(&mut self) {
        let now = Utc::now();
        let mut to_process = Vec::new();

        for (id, proposal) in &self.active_proposals {
            if now >= proposal.voting_ends_at {
                to_process.push(id.clone());
            }
        }

        for id in to_process {
            let mut proposal = self.active_proposals.remove(&id).unwrap();

            // Determine if proposal passed
            let passed = self.evaluate_proposal_outcome(&proposal);

            if passed {
                // Apply the changes
                self.apply_proposal_changes(&proposal);
                proposal.status = ProposalStatus::Approved;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }

            // Add to historical proposals
            self.historical_proposals.push(proposal);
        }
    }

    /// Evaluate whether a proposal passes based on vote counts
    fn evaluate_proposal_outcome(&self, proposal: &Proposal) -> bool {
        let total_votes = proposal.vote_counts.total;
        if total_votes == 0 {
            return false; // No votes means it fails
        }

        let yes_percentage = (proposal.vote_counts.yes as f64 / total_votes as f64) * 100.0;

        // For now, use a simple majority rule
        // In a real implementation, this would consider the change type and required threshold
        yes_percentage > 50.0
    }

    /// Apply changes from an approved proposal
    fn apply_proposal_changes(&mut self, proposal: &Proposal) {
        for change in &proposal.changes {
            match change {
                ProposalChange::ConstitutionAmendment(new_constitution) => {
                    self.constitution = new_constitution.clone();
                }
                ProposalChange::ValueFunctionAddition(value_func) => {
                    // Remove existing value function with same name, then add new one
                    self.active_values.retain(|vf| vf.name != value_func.name);
                    self.active_values.push(value_func.clone());
                }
                ProposalChange::ValueFunctionRemoval(name) => {
                    self.active_values.retain(|vf| vf.name != *name);
                }
                _ => {
                    // Other change types would be handled here
                }
            }
        }
    }

    /// Update agent reputation based on behavior
    pub fn update_agent_reputation(&mut self, agent_id: &str, score_change: f64) {
        let current_score = self.agent_reputations.get(agent_id).unwrap_or(&0.0);
        let new_score = current_score + score_change;

        // Clamp score between 0.0 and 10.0
        let clamped_score = new_score.clamp(0.0, 10.0);
        self.agent_reputations
            .insert(agent_id.to_string(), clamped_score);
    }

    /// Get agent reputation
    pub fn get_agent_reputation(&self, agent_id: &str) -> f64 {
        *self.agent_reputations.get(agent_id).unwrap_or(&0.0)
    }
}

/// Type of vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    /// Vote in favor
    Yes,

    /// Vote against
    No,

    /// Abstain from voting
    Abstain,
}

impl Default for CivilizationLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl CivilizationLayer {
    /// Start the civilization layer services
    pub async fn start(&mut self) -> Result<(), String> {
        println!("Civilization layer started");
        println!("Constitution version: {}", self.constitution.version);
        println!("Active values: {}", self.active_values.len());
        println!("Principles: {}", self.constitution.principles.join(", "));

        Ok(())
    }

    /// Stop the civilization layer services
    pub async fn stop(&mut self) -> Result<(), String> {
        println!("Civilization layer stopped");
        Ok(())
    }
}
