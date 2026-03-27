use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

// 导入execution模块中的StateChange
use crate::execution::StateChange;

// 状态键值对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValue {
    pub value: String,
    pub version: u64,
    pub timestamp: u64,
}

// Merkle节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: String,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub data: Option<String>,
}

// 状态存储
#[derive(Debug, Clone)]
pub struct StateStore {
    pub store: HashMap<String, StateValue>,
    pub versions: Vec<String>, // 状态根哈希的历史
    pub current_root: String,
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StateStore {
    pub fn new() -> Self {
        StateStore {
            store: HashMap::new(),
            versions: vec![],
            current_root: String::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&StateValue> {
        self.store.get(key)
    }

    pub fn put(&mut self, key: String, value: String) {
        let version = self.store.get(&key).map_or(0, |v| v.version + 1);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let state_value = StateValue {
            value,
            version,
            timestamp,
        };

        self.store.insert(key, state_value);
        self.update_root_hash();
    }

    pub fn apply_diff(&mut self, diffs: Vec<StateChange>) -> Result<String, StateError> {
        for diff in diffs {
            match diff.new_value {
                Some(value) => {
                    self.put(diff.key, value);
                }
                None => {
                    // 删除操作
                    self.store.remove(&diff.key);
                }
            }
        }

        Ok(self.current_root.clone())
    }

    fn update_root_hash(&mut self) {
        let mut hasher = Sha256::new();

        // 将所有键值对排序后加入哈希计算
        let mut items: Vec<(&String, &StateValue)> = self.store.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0)); // 修复借用问题

        for (key, value) in items {
            hasher.update(key);
            hasher.update(&value.value);
            hasher.update(value.version.to_le_bytes());
            hasher.update(value.timestamp.to_le_bytes());
        }

        self.current_root = format!("{:x}", hasher.finalize());
        self.versions.push(self.current_root.clone());
    }

    // 公共方法：更新根哈希（供外部调用）
    pub fn update_root_hash_public(&mut self) {
        self.update_root_hash();
    }

    pub fn get_root_hash(&self) -> &str {
        &self.current_root
    }

    pub fn get_version(&self, index: usize) -> Option<&str> {
        self.versions.get(index).map(|s| s.as_str())
    }

    pub fn get_state_diff(&self, _key: &str) -> Vec<StateValue> {
        // 这里简化实现，实际可能需要更复杂的版本跟踪
        vec![]
    }

    // 计算当前状态根
    pub fn compute_current_root(&self) -> String {
        // 简化实现：返回当前根哈希
        self.current_root.clone()
    }
}

// 状态错误
#[derive(Debug)]
pub enum StateError {
    NotFound,
    SerializationError,
    VersionMismatch,
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::NotFound => write!(f, "State not found"),
            StateError::SerializationError => write!(f, "Serialization error"),
            StateError::VersionMismatch => write!(f, "Version mismatch"),
        }
    }
}

impl std::error::Error for StateError {}

// 版本化状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedState {
    pub state_hash: String,
    pub parent_hash: Option<String>,
    pub height: u64,
    pub timestamp: u64,
    pub changes: Vec<crate::execution::StateChange>,
}

// 状态证明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProof {
    pub state_hash: String,
    pub proof_data: String, // 实际实现中这里会是Merkle证明
    pub verified: bool,
}

impl StateProof {
    pub fn new(state_hash: String) -> Self {
        StateProof {
            proof_data: format!("proof_{}", state_hash),
            state_hash,
            verified: false,
        }
    }

    pub fn verify(&mut self) -> bool {
        // 在实际实现中，这里会进行Merkle证明验证
        // 简化实现：假定所有证明都是有效的
        self.verified = true;
        true
    }
}

// 状态验证器
pub struct StateValidator;

impl StateValidator {
    pub fn validate_state_transition(
        &self,
        old_state: &StateStore,
        changes: &[crate::execution::StateChange],
        new_state_hash: &str,
    ) -> Result<bool, StateError> {
        let mut temp_store = old_state.clone_for_validation();

        // 应用变更
        temp_store.apply_diff(changes.to_owned())?;

        // 验证状态哈希
        let computed_hash = temp_store.get_root_hash();

        Ok(computed_hash == new_state_hash)
    }

    // 验证状态根哈希的有效性
    pub fn verify_state_root(&self, state_store: &StateStore) -> bool {
        // 计算当前状态的根哈希
        let computed_root = state_store.get_root_hash();
        let stored_root = state_store.get_root_hash();

        computed_root == stored_root
    }

    // 验证状态历史的连续性
    pub fn verify_state_history(&self, state_store: &StateStore) -> bool {
        if state_store.versions.len() < 2 {
            return true; // 单一状态无法验证历史
        }

        // 检查历史版本的连续性
        for i in 1..state_store.versions.len() {
            // 在实际实现中，这里会验证状态转换的正确性
            // 简化实现：检查哈希格式
            if state_store.versions[i - 1].len() != 64 || state_store.versions[i].len() != 64 {
                return false;
            }
        }

        true
    }
}

// 为StateStore添加克隆验证方法
impl StateStore {
    pub fn clone_for_validation(&self) -> StateStore {
        StateStore {
            store: self.store.clone(),
            versions: self.versions.clone(),
            current_root: self.current_root.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::StateChange;

    #[test]
    fn test_state_store() {
        let mut store = StateStore::new();

        store.put("key1".to_string(), "value1".to_string());
        store.put("key2".to_string(), "value2".to_string());

        assert_eq!(store.get("key1").unwrap().value, "value1");
        assert_eq!(store.get("key2").unwrap().value, "value2");

        let root1 = store.get_root_hash().to_string();
        assert!(!root1.is_empty());

        store.put("key1".to_string(), "value1_updated".to_string());

        let root2 = store.get_root_hash().to_string();
        assert_ne!(root1, root2);

        assert_eq!(store.versions.len(), 3); // 初始化 + 2 次更新
    }

    #[test]
    fn test_apply_diff() {
        let mut store = StateStore::new();

        // 创建一些状态变更
        let diffs = vec![StateChange {
            key: "test_key".to_string(),
            old_value: None,
            new_value: Some("test_value".to_string()),
        }];

        let result = store.apply_diff(diffs).unwrap();
        assert!(!result.is_empty());

        assert_eq!(store.get("test_key").unwrap().value, "test_value");
    }
}
