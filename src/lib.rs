pub mod advanced_consensus;
pub mod atomic;
pub mod civilization;
pub mod consensus;
pub mod cross_chain;
pub mod economics;
pub mod execution;
pub mod governance;
pub mod governance_policy;
pub mod monitoring;
pub mod network;
pub mod state;

pub use consensus::*;
pub use economics::*;
pub use execution::*;
pub use governance::*;
pub use governance_policy::*;
pub use monitoring::*;
pub use network::*;
pub use state::*;

// 避免冲突，分别导出高级共识和跨链功能
// pub use advanced_consensus::*;
// pub use cross_chain::*;

// 为避免命名冲突，我们只导出高级类型
pub use advanced_consensus::{
    Block, ConsensusError as AdvancedConsensusError, ConsensusPhase, ConsensusRound,
    ConsensusTransaction, PBFTConsensusEngine, Vote, VoteType,
};
pub use cross_chain::{
    CrossChainBridge, CrossChainBridgeType, CrossChainError, CrossChainMessage, CrossChainStatus,
    CrossChainTransaction, CrossChainTxType, CrossChainValidator,
};
