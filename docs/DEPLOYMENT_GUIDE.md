# CivitasOS Deployment Guide

## Prerequisites

- Rust 1.70+ installed
- Docker and Docker Compose (for containerized deployments)
- Kubernetes cluster (for production deployments)
- At least 4GB RAM and 50GB storage per node
- Open ports: 8080 (API), 9999 (P2P), 9090 (metrics)

## Installation Methods

### Method 1: From Source

```bash
# Clone the repository
git clone https://github.com/civitasos/civitasos.git
cd civitasos

# Build the project
cargo build --release

# The binary will be available at target/release/civitasos
```

### Method 2: Pre-built Binary

```bash
# Download latest release
curl -L https://github.com/civitasos/civitasos/releases/latest/download/civitasos-linux-amd64.tar.gz | tar xz

# Make executable
chmod +x civitasos
sudo mv civitasos /usr/local/bin/
```

### Method 3: Docker

```bash
# Pull the official image
docker pull civitasos/civitasos:latest

# Or build locally
docker build -t civitasos .
```

## Single Node Deployment

### Basic Setup

```bash
# Initialize configuration
civitasos init --config-path ./config.toml

# Start the node
civitasos --config-path ./config.toml
```

### Configuration File Example

```toml
[network]
listen_port = 9999
external_address = "your-node.example.com:9999"
boot_nodes = []
max_connections = 100

[consensus]
timeout_ms = 30000
validator_count = 1
enable_mock_consensus = true  # For single node setup

[state]
db_path = "./data/state"
snapshot_interval = 1000
prune_height = 10000

[execution]
vm_concurrency = 10
max_gas_per_block = 1000000
enable_debug = false

[security]
rate_limit = 1000
whitelist_only = false
log_level = "info"

[api]
listen_address = "0.0.0.0:8080"
cors_enabled = true
max_request_size = 10485760  # 10MB
```

## Multi-Node Cluster Deployment

### Docker Compose Setup

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  civitasos-node-1:
    image: civitasos:latest
    ports:
      - "8081:8080"
      - "9991:9999"
    volumes:
      - ./node1-data:/data
      - ./config1.toml:/etc/civitasos/config.toml
    environment:
      - NODE_ID=1
    networks:
      - civitasos-net

  civitasos-node-2:
    image: civitasos:latest
    ports:
      - "8082:8080"
      - "9992:9999"
    volumes:
      - ./node2-data:/data
      - ./config2.toml:/etc/civitasos/config.toml
    environment:
      - NODE_ID=2
    networks:
      - civitasos-net

  civitasos-node-3:
    image: civitasos:latest
    ports:
      - "8083:8080"
      - "9993:9999"
    volumes:
      - ./node3-data:/data
      - ./config3.toml:/etc/civitasos/config.toml
    environment:
      - NODE_ID=3
    networks:
      - civitasos-net

networks:
  civitasos-net:
    driver: bridge
```

### Kubernetes Deployment

Create `kustomization.yaml`:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
- namespace.yaml
- configmap.yaml
- service-account.yaml
- rbac.yaml
- statefulset.yaml
- service.yaml
- ingress.yaml

commonLabels:
  app: civitasos
  version: v1.0.0
```

Create `statefulset.yaml`:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: civitasos-nodes
  namespace: civitasos
spec:
  serviceName: civitasos-headless
  replicas: 5
  selector:
    matchLabels:
      app: civitasos-node
  template:
    metadata:
      labels:
        app: civitasos-node
    spec:
      serviceAccountName: civitasos-sa
      containers:
      - name: civitasos
        image: civitasos:latest
        ports:
        - containerPort: 8080
          name: api
        - containerPort: 9999
          name: p2p
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /etc/civitasos
        env:
        - name: POD_IP
          valueFrom:
            fieldRef:
              fieldPath: status.podIP
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: civitasos-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 50Gi
```

## Configuration Options

### Network Configuration

```toml
[network]
# Port for P2P communication
listen_port = 9999

# External address for other nodes to connect
external_address = ""

# Bootstrap nodes to join the network
boot_nodes = [
  "node1.example.com:9999",
  "node2.example.com:9999"
]

# Maximum concurrent connections
max_connections = 100

# Connection timeout in milliseconds
connection_timeout_ms = 10000

# Enable UPnP for automatic port forwarding
upnp_enabled = false
```

### Consensus Configuration

```toml
[consensus]
# Consensus algorithm timeout
timeout_ms = 30000

# Number of validators (for mock consensus)
validator_count = 7

# Enable mock consensus for testing
enable_mock_consensus = false

# Block time in milliseconds
block_time_ms = 5000

# Propose timeout multiplier
propose_timeout_multiplier = 1.5
```

### Security Configuration

```toml
[security]
# Request rate limit per IP
rate_limit = 1000

# Enable whitelist-only mode
whitelist_only = false

# Whitelisted IP addresses
whitelisted_ips = [
  "192.168.1.0/24",
  "10.0.0.0/8"
]

# Log level (trace, debug, info, warn, error)
log_level = "info"

# Enable audit logging
audit_logging = true
```

## Deployment Verification

### Health Checks

```bash
# Check node status
curl http://localhost:8080/health

# Get network statistics
curl http://localhost:8080/stats

# List connected peers
curl http://localhost:8080/peers
```

### Docker Deployment Check

```bash
# Check container status
docker ps | grep civitasos

# View logs
docker logs civitasos-node-1

# Execute commands in container
docker exec -it civitasos-node-1 civitasos-cli status
```

### Kubernetes Deployment Check

```bash
# Check pod status
kubectl get pods -n civitasos

# Check service endpoints
kubectl get svc -n civitasos

# View logs
kubectl logs -f statefulsets/civitasos-nodes -n civitasos

# Scale the deployment
kubectl scale statefulset civitasos-nodes --replicas=7 -n civitasos
```

## Production Considerations

### Security Hardening

1. **Use TLS for API endpoints**
2. **Implement proper firewall rules**
3. **Regular security audits**
4. **Backup and recovery procedures**

### Performance Tuning

1. **Optimize database settings**
2. **Adjust connection pools**
3. **Monitor resource usage**
4. **Load testing**

### Monitoring Setup

```yaml
# Prometheus configuration example
- job_name: 'civitasos'
  static_configs:
    - targets: ['node1:8080', 'node2:8080', 'node3:8080']
  metrics_path: /metrics
  scrape_interval: 15s
```

### Backup Strategy

```bash
# Regular backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/civitasos_$DATE"
mkdir -p $BACKUP_DIR

# Backup state data
tar -czf $BACKUP_DIR/state_data.tar.gz /data/state

# Backup configuration
cp /etc/civitasos/config.toml $BACKUP_DIR/

# Upload to remote storage
aws s3 cp $BACKUP_DIR s3://my-backups/civitasos/ --recursive
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure required ports are available
2. **Insufficient permissions**: Run with appropriate privileges
3. **Network connectivity**: Verify firewall and NAT settings
4. **Resource constraints**: Monitor CPU, memory, and disk usage

### Diagnostic Commands

```bash
# Check node status
civitasos-cli status

# Get detailed network information
civitasos-cli net info

# Check consensus state
civitasos-cli consensus status

# View recent logs
journalctl -u civitasos -f
```