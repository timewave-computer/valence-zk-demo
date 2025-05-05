# Valence ZK: Trustless Cross-Chain Development Framework

Valence ZK is a revolutionary framework that enables trustless cross-blockchain development through zero-knowledge proofs. This repository contains a complete end-to-end demo that serves as a feasibility proof and includes numerous re-usable components for integration with the Valence-coprocessor (our interchain development stack).

## Key Technical Achievements

- **ZK Light Client Integration**: Successfully abstracted and verified zk light client proofs, storing output roots as verified chain state in our SMT (Sparse Merkle Tree)
- **Universal State Proof System**: Implemented comprehensive state proof verification for both:
  - EVM (Ethereum) chains with full storage proof verification
  - Cosmos (Tendermint) chains with ICS23 compliance
- **Cross-Chain Messaging**: Built a nearly production-ready mailbox application demonstrating trustless message passing between Ethereum and Neutron
- **Modular Architecture**: Created a flexible framework that can be extended to support additional blockchain ecosystems

## Core Components

### 1. ZK Light Client Integration
- **Ethereum Light Client**: SP1-based verification of Ethereum consensus state using Helios
- **Tendermint Light Client**: SP1-based verification of Tendermint chain state
- **SMT State Management**: Stores and verifies chain state roots in a Sparse Merkle Tree

### 2. State Proof System
- **EVM Storage Proofs**: Full verification of account and storage proofs
- **ICS23 Proofs**: Support for Cosmos-style storage proofs
- **Block Header Verification**: Full storage proof verification against zk light client roots

### 3. Cross-Chain Applications
- **Mailbox Application**: Nearly production-ready example of cross-chain messaging
- **Modular Design**: Easy integration of new applications and chains

## Technical Architecture

The framework implements a sophisticated architecture that combines:

1. **ZK Light Client Layer**:
   - SP1-based Ethereum light client (Helios)
   - SP1-based Tendermint light client
   - Recursive ZK verification of chain state

2. **State Proof System**:
   - Generic Merkle proof interface
   - Account and storage proof verification
   - Block header verification

3. **Coprocessor Layer**:
   - SMT-based state management
   - Batched proof verification
   - Recursive ZK circuit implementation

## Getting Started

### Prerequisites
- Rust 1.84.0 or later
- Basic understanding of blockchain concepts and zero-knowledge proofs
- Familiarity with Merkle trees and cryptographic proofs

### Installation
```bash
git clone <repository-url>
cd valence-zk-demo
cp .env.example .env
# Update .env with your configuration
```

### Running Examples

#### Full Production E2E Prover with ZK Light Clients and Mailbox Application
```bash
cargo run -p coprocessor --release --features mailbox -- --nocapture
```

## Project Structure

- `coprocessor/`: Core coprocessor logic and proof generation
- `coprocessor-proofs/`: ZK circuit implementations
  - `coprocessor-circuit-types/`: Type-safe circuit definitions
  - `coprocessor-circuit-sp1/`: Optimized SP1 implementation
  - `coprocessor-circuit-logic/`: Core verification logic
- `zk-programs/`: Example ZK applications
  - `zk-mailbox-application/`: Cross-chain messaging
- `lightclients/`: ZK light client implementations
  - `helios/`: Ethereum light client
  - `tendermint/`: Tendermint light client

## Security and Trust Model

Valence ZK implements a robust security model:

- **ZK Light Client Roots**: Verifies chain state using zk light client proofs
- **Recursive ZK Circuits**: Verifies Merkle proofs against verified roots
- **SMT State Management**: Maintains verified chain state in a Sparse Merkle Tree

## Demo Achievements

- [x] Implement ZK light client integration for Ethereum and Neutron
- [x] Build comprehensive state proof system
- [x] Create nearly production-ready mailbox application

## Next Steps

- Architectural improvements
- Integration of ZK Lightclient proofs into the Valence coprocessor
- Integration of storage proofs and block header validation into the Valence coprocessor