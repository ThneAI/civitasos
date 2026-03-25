use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::execution::AtomicDecisionUnit;
use crate::state::StateStore;

// 账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub balance: u64,
    pub staked_amount: u64,
    pub reputation_score: u64,
    pub last_activity: u64,
    pub risk_score: u64, // 风险评分，影响抵押要求
}

// 交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: u64,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Stake,
    Unstake,
    Penalty,
    Reward,
    Fee,
}

// 抵押记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeRecord {
    pub staker: String,
    pub amount: u64,
    pub start_time: u64,
    pub duration: u64, // 锁定时长
    pub reward_rate: f64, // 年化收益率
    pub status: StakeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StakeStatus {
    Active,
    Locked,
    Withdrawn,
    Slashed,
}

// 经济参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicParameters {
    pub base_fee: u64,                  // 基础费用
    pub stake_requirement: u64,         // 最低抵押要求
    pub slashing_threshold: u64,        // 惩罚阈值
    pub reward_rate: f64,               // 奖励利率
    pub inflation_rate: f64,            // 通胀率
    pub min_reputation: u64,            // 最低声誉分数
    pub risk_multiplier: f64,           // 风险乘数
    pub penalty_rate: f64,              // 惩罚比率
}

// 经济引擎
pub struct EconomicEngine {
    pub accounts: HashMap<String, Account>,
    pub transactions: Vec<Transaction>,
    pub stakes: Vec<StakeRecord>,
    pub parameters: EconomicParameters,
    pub state_store: StateStore,
    pub total_supply: u64,
    pub circulation: u64,
}

impl EconomicEngine {
    pub fn new(state_store: StateStore) -> Self {
        EconomicEngine {
            accounts: HashMap::new(),
            transactions: Vec::new(),
            stakes: Vec::new(),
            parameters: EconomicParameters {
                base_fee: 100,
                stake_requirement: 500,  // 降低抵押要求
                slashing_threshold: 50, // 50%的违规率触发惩罚
                reward_rate: 0.1, // 10%年化收益率
                inflation_rate: 0.01, // 1%通胀率
                min_reputation: 10,
                risk_multiplier: 1.5,
                penalty_rate: 0.1, // 10%惩罚率
            },
            state_store,
            total_supply: 1_000_000_000, // 10亿代币
            circulation: 500_000_000,    // 5亿流通
        }
    }

    // 创建账户
    pub fn create_account(&mut self, account_id: String) -> Result<(), EconomicError> {
        if self.accounts.contains_key(&account_id) {
            return Err(EconomicError::AccountExists);
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let account = Account {
            id: account_id,
            balance: 1000, // 初始余额
            staked_amount: 0,
            reputation_score: 100, // 初始声誉
            last_activity: now,
            risk_score: 0, // 初始风险为0
        };

        self.accounts.insert(account.id.clone(), account);
        Ok(())
    }

    // 执行转账
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), EconomicError> {
        // 检查发送方账户
        let sender_exists = self.accounts.contains_key(from);
        if !sender_exists {
            return Err(EconomicError::AccountNotFound);
        }

        // 检查接收方账户
        let receiver_exists = self.accounts.contains_key(to);
        if !receiver_exists {
            return Err(EconomicError::AccountNotFound);
        }

        // 现在安全地进行转账
        {
            let sender = self.accounts.get_mut(from).unwrap();
            // 检查余额
            if sender.balance < amount {
                return Err(EconomicError::InsufficientFunds);
            }

            // 执行转账
            sender.balance -= amount;

            // 更新最后活动时间
            sender.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        // 更新接收方余额
        {
            let receiver = self.accounts.get_mut(to).unwrap();
            receiver.balance += amount;
        }

        // 记录交易
        let tx_id = format!("tx_{}_to_{}_{}", from, to, self.transactions.len());
        let transaction = Transaction {
            id: tx_id,
            from: from.to_string(),
            to: to.to_string(),
            amount,
            fee: self.parameters.base_fee,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transaction_type: TransactionType::Transfer,
        };

        self.transactions.push(transaction);

        Ok(())
    }

    // 抵押
    pub fn stake(&mut self, account_id: &str, amount: u64) -> Result<(), EconomicError> {
        let account = self.accounts.get_mut(account_id)
            .ok_or(EconomicError::AccountNotFound)?;

        // 检查余额
        if account.balance < amount {
            return Err(EconomicError::InsufficientFunds);
        }

        // 检查是否满足最低抵押要求
        if amount < self.parameters.stake_requirement {
            return Err(EconomicError::InsufficientStake);
        }

        // 执行抵押
        account.balance -= amount;
        account.staked_amount += amount;

        // 创建抵押记录
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let stake_record = StakeRecord {
            staker: account_id.to_string(),
            amount,
            start_time: now,
            duration: 365 * 24 * 3600, // 一年锁定
            reward_rate: self.parameters.reward_rate,
            status: StakeStatus::Active,
        };

        self.stakes.push(stake_record);

        // 记录交易
        let tx_id = format!("stake_{}_{}", account_id, self.transactions.len());
        let transaction = Transaction {
            id: tx_id,
            from: account_id.to_string(),
            to: "staking_pool".to_string(),
            amount,
            fee: 0,
            timestamp: now,
            transaction_type: TransactionType::Stake,
        };

        self.transactions.push(transaction);

        Ok(())
    }

    // 解押
    pub fn unstake(&mut self, account_id: &str, amount: u64) -> Result<(), EconomicError> {
        let account = self.accounts.get_mut(account_id)
            .ok_or(EconomicError::AccountNotFound)?;

        // 检查抵押金额
        if account.staked_amount < amount {
            return Err(EconomicError::InsufficientStakedFunds);
        }

        // 查找对应的抵押记录并检查锁定时间
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut unstaked_amount = 0;
        for stake in self.stakes.iter_mut() {
            if stake.staker == account_id && 
               stake.status == StakeStatus::Active &&
               now >= stake.start_time + stake.duration {
                
                if unstaked_amount + stake.amount <= amount {
                    // 解锁整个抵押记录
                    stake.status = StakeStatus::Withdrawn;
                    unstaked_amount += stake.amount;
                } else {
                    // 部分解锁
                    let remaining_to_unstake = amount - unstaked_amount;
                    // 这里简化处理，实际实现会更复杂
                    stake.amount -= remaining_to_unstake;
                    unstaked_amount += remaining_to_unstake;
                    
                    if stake.amount == 0 {
                        stake.status = StakeStatus::Withdrawn;
                    }
                    
                    break;
                }
            }
        }

        if unstaked_amount < amount {
            return Err(EconomicError::StakeStillLocked);
        }

        // 更新账户
        account.staked_amount -= amount;
        account.balance += amount;

        // 记录交易
        let tx_id = format!("unstake_{}_{}", account_id, self.transactions.len());
        let transaction = Transaction {
            id: tx_id,
            from: "staking_pool".to_string(),
            to: account_id.to_string(),
            amount,
            fee: 0,
            timestamp: now,
            transaction_type: TransactionType::Unstake,
        };

        self.transactions.push(transaction);

        Ok(())
    }

    // 计算执行费用
    pub fn calculate_execution_fee(&self, adu: &AtomicDecisionUnit) -> u64 {
        // 基础费用
        let mut fee = self.parameters.base_fee;

        // 根据风险评分调整费用
        if let Some(account) = self.accounts.get(&adu.accountability_anchor) {
            fee = (fee as f64 * (1.0 + (account.risk_score as f64 / 100.0) * self.parameters.risk_multiplier)) as u64;
        }

        // 根据操作复杂度调整（简化实现）
        fee += (adu.execution_trace.len() as u64) * 10;

        fee
    }

    // 执行经济惩罚
    pub fn apply_penalty(&mut self, account_id: &str, penalty_amount: u64) -> Result<(), EconomicError> {
        let account = self.accounts.get_mut(account_id)
            .ok_or(EconomicError::AccountNotFound)?;

        // 从余额中扣除，如果余额不足则从抵押中扣除
        if account.balance >= penalty_amount {
            account.balance -= penalty_amount;
        } else {
            let remaining_penalty = penalty_amount - account.balance;
            
            if account.staked_amount >= remaining_penalty {
                account.balance = 0;
                account.staked_amount -= remaining_penalty;
                
                // 在抵押记录中标记被惩罚的部分
                for stake in self.stakes.iter_mut() {
                    if stake.staker == account_id && stake.status == StakeStatus::Active {
                        if stake.amount >= remaining_penalty {
                            stake.amount -= remaining_penalty;
                            stake.status = StakeStatus::Slashed;
                            break;
                        }
                    }
                }
            } else {
                return Err(EconomicError::InsufficientFundsForPenalty);
            }
        }

        // 降低声誉分数
        account.reputation_score = account.reputation_score.saturating_sub(10);

        // 记录惩罚交易
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let tx_id = format!("penalty_{}_{}", account_id, self.transactions.len());
        let transaction = Transaction {
            id: tx_id,
            from: account_id.to_string(),
            to: "penalty_pool".to_string(),
            amount: penalty_amount,
            fee: 0,
            timestamp: now,
            transaction_type: TransactionType::Penalty,
        };

        self.transactions.push(transaction);

        Ok(())
    }

    // 发放奖励
    pub fn distribute_reward(&mut self, account_id: &str, reward_amount: u64) -> Result<(), EconomicError> {
        let account = self.accounts.get_mut(account_id)
            .ok_or(EconomicError::AccountNotFound)?;

        account.balance += reward_amount;

        // 提升声誉分数
        account.reputation_score = std::cmp::min(account.reputation_score.saturating_add(1), 1000);

        // 记录奖励交易
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let tx_id = format!("reward_{}_{}", account_id, self.transactions.len());
        let transaction = Transaction {
            id: tx_id,
            from: "reward_pool".to_string(),
            to: account_id.to_string(),
            amount: reward_amount,
            fee: 0,
            timestamp: now,
            transaction_type: TransactionType::Reward,
        };

        self.transactions.push(transaction);

        Ok(())
    }

    // 计算抵押奖励
    pub fn calculate_staking_rewards(&mut self) -> Result<(), EconomicError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 收集需要奖励的记录
        let stakes_to_reward: Vec<(String, u64)> = self.stakes.iter()
            .filter(|stake| stake.status == StakeStatus::Active)
            .map(|stake| {
                // 计算奖励时间（简化：按年化利率计算）
                let time_elapsed = now.saturating_sub(stake.start_time);
                let reward = (stake.amount as f64 * stake.reward_rate * (time_elapsed as f64 / (365.0 * 24.0 * 3600.0))) as u64;
                (stake.staker.clone(), reward)
            })
            .filter(|(_, reward)| *reward > 0)
            .collect();

        // 分发奖励
        for (staker, reward) in stakes_to_reward {
            self.distribute_reward(&staker, reward)?;
        }

        Ok(())
    }

    // 验证ADU的经济约束
    pub fn validate_economic_constraints(&self, adu: &AtomicDecisionUnit) -> Result<bool, EconomicError> {
        // 检查账户是否存在
        let account = self.accounts.get(&adu.accountability_anchor)
            .ok_or(EconomicError::AccountNotFound)?;

        // 检查声誉分数是否足够
        if account.reputation_score < self.parameters.min_reputation {
            return Ok(false);
        }

        // 检查抵押是否足够
        if account.staked_amount < self.parameters.stake_requirement {
            return Ok(false);
        }

        // 计算执行费用
        let fee = self.calculate_execution_fee(adu);

        // 检查是否有足够的资金支付费用
        if account.balance < fee {
            return Ok(false);
        }

        Ok(true)
    }

    // 获取账户信息
    pub fn get_account(&self, account_id: &str) -> Option<&Account> {
        self.accounts.get(account_id)
    }

    // 获取账户抵押总额
    pub fn get_total_staked(&self, account_id: &str) -> u64 {
        self.stakes.iter()
            .filter(|s| s.staker == account_id && s.status == StakeStatus::Active)
            .map(|s| s.amount)
            .sum()
    }

    // 获取账户总余额（余额+抵押）
    pub fn get_total_balance(&self, account_id: &str) -> u64 {
        if let Some(account) = self.accounts.get(account_id) {
            account.balance + account.staked_amount
        } else {
            0
        }
    }
}

// 经济错误
#[derive(Debug)]
pub enum EconomicError {
    AccountNotFound,
    AccountExists,
    InsufficientFunds,
    InsufficientStakedFunds,
    InsufficientStake,
    StakeStillLocked,
    InsufficientFundsForPenalty,
    InvalidTransaction,
}

impl std::fmt::Display for EconomicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EconomicError::AccountNotFound => write!(f, "Account not found"),
            EconomicError::AccountExists => write!(f, "Account already exists"),
            EconomicError::InsufficientFunds => write!(f, "Insufficient funds"),
            EconomicError::InsufficientStakedFunds => write!(f, "Insufficient staked funds"),
            EconomicError::InsufficientStake => write!(f, "Insufficient stake"),
            EconomicError::StakeStillLocked => write!(f, "Stake still locked"),
            EconomicError::InsufficientFundsForPenalty => write!(f, "Insufficient funds for penalty"),
            EconomicError::InvalidTransaction => write!(f, "Invalid transaction"),
        }
    }
}

impl std::error::Error for EconomicError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{AtomicDecisionUnit, OpCode};
    use crate::state::StateStore;

    #[test]
    fn test_economic_engine() {
        let state_store = StateStore::new();
        let mut econ = EconomicEngine::new(state_store);

        // 创建账户
        let result = econ.create_account("account1".to_string());
        assert!(result.is_ok());

        // 检查初始余额
        let account = econ.get_account("account1").unwrap();
        assert_eq!(account.balance, 1000);
        assert_eq!(account.reputation_score, 100);

        // 转账测试
        econ.create_account("account2".to_string()).unwrap();
        let transfer_result = econ.transfer("account1", "account2", 100);
        assert!(transfer_result.is_ok());

        // 检查转账后余额
        let acc1 = econ.get_account("account1").unwrap();
        let acc2 = econ.get_account("account2").unwrap();
        assert_eq!(acc1.balance, 900);
        assert_eq!(acc2.balance, 1100);

        // 抵押测试
        let stake_result = econ.stake("account1", 500);
        assert!(stake_result.is_ok());

        let acc1_after_stake = econ.get_account("account1").unwrap();
        assert_eq!(acc1_after_stake.balance, 400);
        assert_eq!(acc1_after_stake.staked_amount, 500);

        // 验证抵押总额
        assert_eq!(econ.get_total_staked("account1"), 500);
        assert_eq!(econ.get_total_balance("account1"), 900);
    }

    #[test]
    fn test_execution_fee_calculation() {
        let state_store = StateStore::new();
        let econ = EconomicEngine::new(state_store);

        // 创建一个ADU
        let adu = AtomicDecisionUnit {
            input_state_hash: "hash".to_string(),
            rule_id: "rule1".to_string(),
            execution_trace: vec![OpCode::ADD(0, 1, 2)],
            output_proof: "proof".to_string(),
            accountability_anchor: "account1".to_string(),
            risk_stake: 100,
        };

        // 计算执行费用
        let fee = econ.calculate_execution_fee(&adu);
        assert!(fee >= econ.parameters.base_fee);
    }

    #[test]
    fn test_penalty_and_reward() {
        let state_store = StateStore::new();
        let mut econ = EconomicEngine::new(state_store);

        // 创建账户
        econ.create_account("account1".to_string()).unwrap();
        let initial_balance = econ.get_account("account1").unwrap().balance;

        // 施加惩罚
        let penalty_result = econ.apply_penalty("account1", 50);
        assert!(penalty_result.is_ok());

        let after_penalty = econ.get_account("account1").unwrap();
        assert_eq!(after_penalty.balance, initial_balance - 50);
        assert_eq!(after_penalty.reputation_score, 90); // 降低了10分

        // 发放奖励
        let reward_result = econ.distribute_reward("account1", 100);
        assert!(reward_result.is_ok());

        let after_reward = econ.get_account("account1").unwrap();
        assert_eq!(after_reward.balance, initial_balance - 50 + 100);
        assert_eq!(after_reward.reputation_score, 91); // 提升了1分
    }
}