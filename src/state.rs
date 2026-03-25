use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
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
                },
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
        items.sort_by(|a, b| a.0.cmp(b.0));  // 修复借用问题
        
        for (key, value) in items {
            hasher.update(key);
            hasher.update(&value.value);
            hasher.update(&value.version.to_le_bytes());
            hasher.update(&value.timestamp.to_le_bytes());
        }
        
        self.current_root = format!("{:x}", hasher.finalize());
        self.versions.push(self.current_root.clone());
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
    pub changes: Vec<StateChange>,
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
        let diffs = vec![
            StateChange {
                key: "test_key".to_string(),
                old_value: None,
                new_value: Some("test_value".to_string()),
            }
        ];
        
        let result = store.apply_diff(diffs).unwrap();
        assert!(!result.is_empty());
        
        assert_eq!(store.get("test_key").unwrap().value, "test_value");
    }
}