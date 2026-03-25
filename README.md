# CivitasOS

CivitasOS is a civilization-grade operating system for AI agents, designed to create a verifiable, accountable, and human-sovereign collaborative civilization system.

## Overview

CivitasOS implements a five-layer architecture:
- **Layer 5: Civilization Layer** - Constitution, value functions, evolution mechanisms
- **Layer 4: Consensus Layer** - Validation, signatures, legitimacy confirmation
- **Layer 3: Execution Layer** - RISC VM, deterministic execution
- **Layer 2: State Layer** - Merkle DAG, versioned state
- **Layer 1: Atomic Layer** - ADU, accountability anchoring, proof system

## Features

- Deterministic execution with RISC-style instruction set
- Verifiable state management with Merkle DAG
- Accountability anchoring for all actions
- Three-party model (Responsibility, Value, Risk Subjects)
- Mathematical security guarantees

## Architecture Highlights

### Atomic Decision Unit (ADU)
Each decision is encapsulated in an atomic unit that includes:
- Input state hash
- Rule ID
- Execution trace
- Output proof
- Accountability anchor
- Risk stake

### RISC Execution Engine
Minimal instruction set for deterministic execution:
- LOAD, STORE
- ADD, SUB
- HASH
- CMP, JZ, JNZ
- RETURN, REVERT

### State Management
- Versioned state storage
- Merkle DAG for state verification
- Immutable historical records

## Getting Started

```bash
# Clone the repository
git clone https://github.com/ThneAI/civitasos.git

# Build the project
cd civitasos
cargo build

# Run tests
cargo test
```

## MVP Development Status

Current progress (Week 1-2):
- ✅ Core execution engine with RISC instruction set
- ✅ State management with Merkle DAG
- ✅ Deterministic execution with trace hashing
- ✅ Basic tests and demo functionality

## License

MIT