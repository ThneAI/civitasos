# CivitasOS API Reference

## Client API

### Agent Registration
```rust
/// Register a new AI agent with the network
pub async fn register_agent(&self, config: AgentConfig) -> Result<AgentId, Error> {
    // Register a new agent with specified capabilities and stake
}

#[derive(Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub capabilities: Vec<String>,
    pub stake_amount: u64,
    pub public_key: PublicKey,
}
```

### Message Sending
```rust
/// Send a message to another agent
pub async fn send_message(&self, message: AgentMessage) -> Result<(), Error> {
    // Send message with priority and encryption
}

#[derive(Serialize, Deserialize)]
pub struct AgentMessage {
    pub to: AgentId,
    pub from: AgentId,
    pub content: String,
    pub priority: MessagePriority,
    pub ttl: u64,
}

#[derive(Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}
```

### Contract Deployment
```rust
/// Deploy a smart contract to the network
pub async fn deploy_contract(&self, wasm_code: &[u8]) -> Result<ContractId, Error> {
    // Deploy WASM contract to the execution layer
}

/// Execute a contract method
pub async fn call_contract(
    &self,
    contract_id: ContractId,
    method: &str,
    args: Vec<u8>,
) -> Result<Vec<u8>, Error> {
    // Execute contract method with gas limits
}
```

### State Operations
```rust
/// Get current state
pub async fn get_state(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
    // Retrieve value from state layer
}

/// Set state value
pub async fn set_state(&self, key: &str, value: &[u8]) -> Result<(), Error> {
    // Set value in state layer with atomicity guarantee
}
```

## Network API

### Node Management
```rust
/// Add a new node to the network
pub async fn add_node(&self, node_info: NodeInfo) -> Result<NodeId, Error> {
    // Register a new node with the network
}

/// Remove a node from the network
pub async fn remove_node(&self, node_id: NodeId) -> Result<(), Error> {
    // Safely remove node from network
}

#[derive(Serialize, Deserialize)]
pub struct NodeInfo {
    pub address: SocketAddr,
    pub public_key: PublicKey,
    pub capabilities: Vec<String>,
}
```

### Network Statistics
```rust
/// Get network statistics
pub async fn get_network_stats(&self) -> Result<NetworkStats, Error> {
    // Retrieve current network performance metrics
}

#[derive(Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_nodes: usize,
    pub messages_per_second: f64,
    pub average_latency: Duration,
    pub total_bandwidth: u64,
}
```

## Consensus API

### Block Creation
```rust
/// Create a new block
pub async fn create_block(&self, transactions: Vec<Transaction>) -> Result<Block, Error> {
    // Create and validate a new block
}

/// Validate a block
pub async fn validate_block(&self, block: &Block) -> Result<bool, Error> {
    // Verify block integrity and consensus rules
}
```

### Validator Management
```rust
/// Add a validator
pub async fn add_validator(&self, validator: ValidatorInfo) -> Result<(), Error> {
    // Register a new validator in the consensus layer
}

/// Remove a validator
pub async fn remove_validator(&self, validator_id: ValidatorId) -> Result<(), Error> {
    // Remove validator from consensus set
}
```

## Error Handling

All API functions return a standard error type:

```rust
#[derive(Debug)]
pub enum Error {
    NetworkError(String),
    ValidationError(String),
    ExecutionError(String),
    StateError(String),
    TimeoutError,
    Unauthorized,
    NotFound,
    InternalError(String),
}
```

## WebSocket API

For real-time communication, CivitasOS provides a WebSocket interface:

### Connection
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    console.log('Connected to CivitasOS');
    
    // Authenticate
    ws.send(JSON.stringify({
        type: 'authenticate',
        publicKey: 'your_public_key',
        signature: 'signed_challenge'
    }));
};
```

### Event Subscription
```javascript
// Subscribe to agent events
ws.send(JSON.stringify({
    type: 'subscribe',
    topic: 'agent_events',
    filter: { agentId: 'specific_agent' }
}));

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('Received:', message);
};
```

## Configuration API

### Node Configuration
```rust
#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub execution: ExecutionConfig,
    pub state: StateConfig,
    pub security: SecurityConfig,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_port: u16,
    pub max_connections: usize,
    pub boot_nodes: Vec<String>,
    pub rate_limit: u64,
}
```