pub mod execution;
pub mod state;
pub mod consensus;
pub mod governance;
pub mod economics;
pub mod advanced_consensus;
pub mod cross_chain;
pub mod governance_policy;

pub use execution::*;
pub use state::*;
pub use consensus::*;
pub use governance::*;
pub use economics::*;
pub use governance_policy::*;

// 避免冲突，分别导出高级共识和跨链功能
// pub use advanced_consensus::*;
// pub use cross_chain::*;

// 为避免命名冲突，我们只导出高级类型
pub use advanced_consensus::{
    PBFTConsensusEngine, ConsensusRound, ConsensusPhase, Block, ConsensusTransaction, 
    Vote, VoteType, ConsensusError as AdvancedConsensusError
};
pub use cross_chain::{
    CrossChainBridge, CrossChainBridgeType, CrossChainMessage, CrossChainStatus, 
    CrossChainTransaction, CrossChainTxType, CrossChainValidator, CrossChainError
};