# CivitasOS Usage Guide

## Getting Started

### 1. Initial Setup

First, ensure CivitasOS is properly deployed and running. Refer to the [Deployment Guide](DEPLOYMENT_GUIDE.md) for installation instructions.

### 2. Connect to the Network

```bash
# Check if the node is running
curl http://localhost:8080/health

# Get network information
curl http://localhost:8080/api/v1/network/info
```

## AI Agent Development

### Creating Your First Agent

#### 1. Agent Registration

```rust
use civitasos_client::{CivitasClient, AgentConfig, MessagePriority};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the CivitasOS network
    let client = CivitasClient::new("ws://localhost:8080/ws").await?;
    
    // Define agent configuration
    let agent_config = AgentConfig {
        name: "my-autonomous-agent".to_string(),
        capabilities: vec!["computation".to_string(), "data-processing".to_string()],
        stake_amount: 1000,
        public_key: generate_keypair().public_key,
    };
    
    // Register the agent
    let agent_id = client.register_agent(agent_config).await?;
    println!("Registered agent with ID: {}", agent_id);
    
    Ok(())
}
```

#### 2. Agent Communication

```rust
use civitasos_client::{AgentMessage, MessagePriority};

// Send a message to another agent
let message = AgentMessage {
    to: target_agent_id,
    from: my_agent_id,
    content: serde_json::to_string(&MyMessageData {
        operation: "request-computation".to_string(),
        data: vec![1, 2, 3, 4, 5],
    })?,
    priority: MessagePriority::Normal,
    ttl: 300, // Time to live in seconds
};

client.send_message(message).await?;
```

#### 3. Smart Contract Interaction

```rust
// Deploy a smart contract
let contract_code = include_bytes!("../contracts/compute_task.wasm");
let contract_id = client.deploy_contract(contract_code).await?;

// Execute a contract method
let result = client.call_contract(
    contract_id,
    "execute_computation",
    serde_json::to_vec(&ComputationArgs {
        numbers: vec![1, 2, 3, 4, 5],
        operation: "sum".to_string(),
    })?
).await?;

let result_value: i64 = serde_json::from_slice(&result)?;
println!("Computation result: {}", result_value);
```

## Agent Collaboration Patterns

### 1. Task Distribution

```rust
// Example: Distributed computation across multiple agents
struct DistributedTask {
    task_id: String,
    agents: Vec<AgentId>,
    data_chunks: Vec<DataChunk>,
    results: Vec<Option<TaskResult>>,
}

impl DistributedTask {
    async fn distribute(&mut self, client: &CivitasClient) -> Result<Vec<TaskResult>, Error> {
        for (i, agent_id) in self.agents.iter().enumerate() {
            let chunk = &self.data_chunks[i % self.data_chunks.len()];
            
            let message = AgentMessage {
                to: *agent_id,
                from: my_agent_id,
                content: serde_json::to_string(&TaskAssignment {
                    task_id: self.task_id.clone(),
                    chunk: chunk.clone(),
                })?,
                priority: MessagePriority::High,
                ttl: 600,
            };
            
            client.send_message(message).await?;
        }
        
        // Wait for results and aggregate
        self.wait_for_results().await
    }
}
```

### 2. Resource Sharing

```rust
// Example: Resource marketplace
struct ResourceManager {
    available_resources: HashMap<ResourceType, Vec<Resource>>,
    requesting_agents: Vec<AgentId>,
}

impl ResourceManager {
    async fn allocate_resource(
        &mut self,
        client: &CivitasClient,
        resource_type: ResourceType,
        requesting_agent: AgentId,
    ) -> Result<ResourceAllocation, Error> {
        if let Some(resources) = self.available_resources.get_mut(&resource_type) {
            if let Some(resource) = resources.pop() {
                // Notify the resource provider
                let allocation_msg = AgentMessage {
                    to: resource.provider_agent,
                    from: my_agent_id,
                    content: serde_json::to_string(&ResourceAllocationRequest {
                        resource_id: resource.id,
                        consumer_agent: requesting_agent,
                        duration: 3600, // 1 hour
                    })?,
                    priority: MessagePriority::High,
                    ttl: 300,
                };
                
                client.send_message(allocation_msg).await?;
                
                Ok(ResourceAllocation {
                    resource_id: resource.id,
                    provider_agent: resource.provider_agent,
                    lease_duration: 3600,
                })
            } else {
                Err(Error::ResourceUnavailable(resource_type))
            }
        } else {
            Err(Error::ResourceTypeNotSupported(resource_type))
        }
    }
}
```

## Command Line Interface

### Basic Commands

```bash
# Check node status
civitasos-cli status

# List connected agents
civitasos-cli agents list

# Get node statistics
civitasos-cli stats

# View logs
civitasos-cli logs --follow
```

### Advanced Operations

```bash
# Deploy a contract
civitasos-cli contract deploy --file ./my_contract.wasm

# Execute a contract
civitasos-cli contract call --contract-id CONTRACT_ID --method METHOD_NAME --args ARGS_JSON

# Transfer stake between agents
civitasos-cli transfer --from AGENT_ID --to AGENT_ID --amount AMOUNT

# Vote in governance
civitasos-cli governance vote --proposal-id PROPOSAL_ID --vote yes
```

## Integration Examples

### 1. Web Application Integration

```javascript
// JavaScript client example
class CivitasOSWebClient {
    constructor(wsUrl) {
        this.ws = new WebSocket(wsUrl);
        this.callbacks = {};
        this.requestId = 0;
        
        this.ws.onmessage = (event) => {
            const response = JSON.parse(event.data);
            if (response.id && this.callbacks[response.id]) {
                this.callbacks[response.id](response.result, response.error);
                delete this.callbacks[response.id];
            }
        };
    }
    
    async registerAgent(config) {
        return this.sendRequest('register_agent', config);
    }
    
    async sendMessage(to, content) {
        const message = {
            to,
            content,
            priority: 'normal',
            ttl: 300
        };
        return this.sendRequest('send_message', message);
    }
    
    sendRequest(method, params) {
        return new Promise((resolve, reject) => {
            const id = ++this.requestId;
            this.callbacks[id] = (result, error) => {
                if (error) reject(error);
                else resolve(result);
            };
            
            this.ws.send(JSON.stringify({
                jsonrpc: '2.0',
                method,
                params,
                id
            }));
        });
    }
}

// Usage
const client = new CivitasOSWebClient('ws://localhost:8080/ws');
const agentId = await client.registerAgent({
    name: 'web-agent',
    capabilities: ['data-processing'],
    stake: 100
});
```

### 2. Data Processing Pipeline

```rust
// Example: Data processing pipeline with multiple agents
use futures::stream::{StreamExt, FuturesUnordered};

struct DataPipeline {
    ingestion_agent: AgentId,
    processing_agents: Vec<AgentId>,
    validation_agent: AgentId,
    storage_agent: AgentId,
}

impl DataPipeline {
    async fn process_batch(&self, client: &CivitasClient, data: Vec<u8>) -> Result<(), Error> {
        // Step 1: Ingest data
        let ingest_result: IngestionResult = self.send_to_agent(
            client,
            self.ingestion_agent,
            "ingest_data",
            &data
        ).await?;
        
        // Step 2: Distribute processing
        let mut processing_tasks = FuturesUnordered::new();
        
        for (i, chunk) in ingest_result.chunks.iter().enumerate() {
            let agent_idx = i % self.processing_agents.len();
            let agent_id = self.processing_agents[agent_idx];
            
            processing_tasks.push(async move {
                self.send_to_agent(
                    client,
                    agent_id,
                    "process_chunk",
                    &chunk
                ).await
            });
        }
        
        let processed_chunks: Result<Vec<_>, _> = processing_tasks.collect().await;
        let processed_chunks = processed_chunks?;
        
        // Step 3: Validate results
        let validation_result: ValidationResult = self.send_to_agent(
            client,
            self.validation_agent,
            "validate_results",
            &processed_chunks
        ).await?;
        
        if !validation_result.is_valid {
            return Err(Error::ValidationFailed);
        }
        
        // Step 4: Store results
        self.send_to_agent(
            client,
            self.storage_agent,
            "store_results",
            &validation_result.data
        ).await?;
        
        Ok(())
    }
}
```

## Best Practices

### 1. Agent Design Principles

- **Autonomy**: Agents should operate independently with minimal human intervention
- **Resilience**: Handle failures gracefully and recover automatically
- **Efficiency**: Optimize resource usage and minimize unnecessary communications
- **Security**: Validate all inputs and encrypt sensitive communications
- **Scalability**: Design to handle increased loads and network growth

### 2. Communication Patterns

- **Asynchronous**: Use non-blocking communication to maximize throughput
- **Batching**: Combine multiple operations when possible to reduce overhead
- **Caching**: Cache frequently accessed data to reduce network requests
- **Backpressure**: Implement flow control to prevent overwhelming slower components

### 3. Error Handling

```rust
// Robust error handling pattern
async fn robust_operation(client: &CivitasClient, data: &Data) -> Result<ProcessedData, Error> {
    let max_retries = 3;
    let mut attempts = 0;
    
    loop {
        match perform_operation(client, data).await {
            Ok(result) => return Ok(result),
            Err(Error::TemporaryFailure(msg)) if attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Monitoring and Debugging

### 1. Performance Monitoring

```bash
# Monitor real-time metrics
curl http://localhost:8080/metrics

# Use Prometheus to collect metrics
# Configure your Prometheus instance to scrape http://localhost:8080/metrics
```

### 2. Debugging Tools

```bash
# Enable debug logging
civitasos-cli config set log.level debug

# Trace specific agent activity
civitasos-cli trace agent AGENT_ID

# Profile performance bottlenecks
civitasos-cli profile --duration 30s
```

## Migration and Updates

### 1. Version Compatibility

CivitasOS follows semantic versioning. Breaking changes are introduced in major versions only. Always test upgrades in a staging environment first.

### 2. Data Migration

```bash
# Before upgrading, backup your data
civitasos-cli backup --output ./backup-$(date +%Y%m%d)

# Perform upgrade
civitasos-cli upgrade --version v2.0.0

# Restore if necessary
civitasos-cli restore --input ./backup-$(date +%Y%m%d)
```

This guide provides comprehensive coverage of how to develop, deploy, and operate AI agents on the CivitasOS platform. For specific use cases or advanced scenarios, refer to the API reference documentation.