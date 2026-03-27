# CivitasOS

CivitasOS is a decentralized AI agent operating system built on a 6-layer architecture designed to facilitate autonomous AI agent interactions in a secure, scalable environment.

## 🌐 Frontend Demo

Check out our interactive frontend demo: [CivitasOS Frontend](https://github.com/ThneAI/civitasos-frontend)

## Overview

CivitasOS provides the infrastructure for AI agents to operate autonomically in a decentralized network. The system consists of six distinct layers, each responsible for specific aspects of the overall functionality:

- **Layer 5 (Civilization)**: Governance, constitutional rules, and value alignment
- **Layer 4 (Consensus)**: Validation, legitimacy verification, and agreement mechanisms  
- **Layer 3 (Execution)**: Deterministic computation and smart contract execution
- **Layer 2 (State)**: Immutable state storage and versioning
- **Layer 1 (Atomic)**: Atomic operations and accountability
- **Layer 0 (Network)**: Peer-to-peer communication and node discovery

## Architecture

### Layer 5: Civilization Layer
The topmost layer responsible for governance and constitutional rules. It defines the values, principles, and evolution mechanisms that guide the entire system.

### Layer 4: Consensus Layer
Implements validation and legitimacy verification mechanisms. Uses advanced consensus algorithms to ensure all participants agree on the state of the system while maintaining security and decentralization.

### Layer 3: Execution Layer
Provides a deterministic virtual machine environment for executing smart contracts and complex computations. Features gas metering and resource management to ensure fair usage.

### Layer 2: State Layer
Utilizes Merkle DAG data structures for immutable state storage with versioning capabilities. Enables efficient state synchronization and verification across the network.

### Layer 1: Atomic Layer
Handles atomic operations and accountability mechanisms. Implements Atomic Data Units (ADU) and accountability anchors to ensure transaction integrity.

### Layer 0: Network Layer
The foundational peer-to-peer communication layer enabling node discovery, message propagation, and network security. Designed for high-performance and resilience.

## Features

- **6-Layer Architecture**: Comprehensive system design covering all aspects of decentralized AI operation
- **AI-Agent Centric**: Purpose-built for autonomous AI agents with minimal human intervention
- **Security & Trust**: Cryptographic verification at every layer with Byzantine fault tolerance
- **Scalability**: Horizontal scaling across multiple nodes with high throughput
- **Governance**: Democratic decision-making and constitutional evolution mechanisms
- **Accountability**: Traceable operations and verifiable execution
- **Interactive Demo**: Visual demonstration of all system components

## Documentation

Detailed documentation is available in the `docs/` directory:

- [Architecture](docs/ARCHITECTURE.md) - Complete architectural overview
- [API Reference](docs/API_REFERENCE.md) - Full API documentation
- [Deployment Guide](docs/DEPLOYMENT_GUIDE.md) - Setup and deployment instructions
- [Usage Guide](docs/USAGE_GUIDE.md) - How to develop and use AI agents

## Getting Started

1. Ensure you have Rust 1.70+ installed
2. Clone the repository
3. Build the project: `cargo build`
4. Run the system: `cargo run`
5. For frontend demo, see: [CivitasOS Frontend](https://github.com/ThneAI/civitasos-frontend)

## License

Apache 2.0