# ZK Valence Programs Demo
A comprehensive demonstration of cross-chain zero-knowledge proof verification, showcasing how to achieve trustless state verification across multiple blockchain ecosystems. This implementation provides a practical example of using ZK proofs to verify state from both Ethereum and Neutron Smart Contracts in a secure and efficient manner.

This demo represents a complete implementation of our ZK development stack with the valence-coprocessor. While the core functionality is fully implemented, two key features are currently in development: zk light-client proofs and a production-ready SMT implementation (currently using an in-memory version). Additionally, we plan to implement abstractions for serialization and type systems across different domains.

Currently the supported domains are `Neutron` (a Cosmos-ecosystem chain) and `Ethereum`. The example contracts were deployed on their respective test networks. 

Our infrastructure can easily be extended to support any Cosmos `Ics23` or EVM-comptaible chain, as the underlying merkle proofs work the same for all of these networks.


[![Learn More About Valence ZK](https://img.shields.io/badge/_Learn_More_About_Valence_ZK-2EA44F?style=for-the-badge&logo=github&logoColor=white)](https://github.com/timewave-computer/recursive-sp1-verifier/blob/master/context.md)


## Getting Started

### 1. Clone the repository:
```bash
git clone <repository-url>
cd valence-zk-demo
```

### 2. Configure your environment:
```bash
cp .env.example .env
# Update .env with your configuration
```
### 3. Run our Example Applications

3.1. Rate calculation example
```rust
cargo run -p coprocessor --release --features rate -- --nocapture
```
3.2. Cross-chain message mailbox example
```rust
cargo run -p coprocessor --release --features mailbox -- --nocapture
```

## Technical Overview

The implementation demonstrates a complete workflow for cross-chain state verification:

1. **State Collection**: Populates the coprocessor tree with verified state from Ethereum and Neutron Smart Contracts
2. **Proof Generation**: Creates opening proofs against the co-processor root using a recursive ZK circuit
3. **State Verification**: Provides these proofs as context, along with the co-processor SMT root, to the Valence ZK program

## Project Structure

- `coprocessor/`: application handling coprocessor logic and proof generation
- `coprocessor-proofs/`: Circuit implementations for proof generation
  - `coprocessor-circuit-types/`: Type-safe definitions for circuit operations
  - `coprocessor-circuit-sp1/`: Optimized SP1-specific circuit implementation
  - `coprocessor-circuit-logic/`: Core verification logic
- `zk-programs/`: Zero-knowledge program implementations
  - `zk-rate-application/`: Main ZK rate application demonstrating practical use case
  - `zk-rate-application-types/`: Type definitions ensuring type safety across the application

## ZK Programs Example

The project includes a practical example of a cross-chain vault rate calculation using zero-knowledge proofs. This example demonstrates how to:

1. Verify state from multiple blockchains (Ethereum and Neutron)
2. Calculate a trustless rate across different chains
3. Generate and verify ZK proofs for the calculation

### ZK rate example program

The ZK rate example consists of three main components:

- `zk-rate-application/`: The main ZK program that performs the rate calculation
- `zk-rate-application-types/`: Type definitions for the circuit inputs and outputs
- `zk-rate-application-contracts/`: Smart contracts deployed on test networks

The example calculates a rate based on vault balances and shares across two chains:
- Ethereum (Sepolia testnet)
- Neutron (Pion-1 testnet)

For detailed information about the example implementation, including contract addresses, storage layouts, and initialization values, please refer to the [ZK Rate Example README](./zk-programs/zk-rate-example/README.md).

## Architecture

The system implements a recursive ZK circuit that verifies merkle proofs against `trusted roots`. This architecture is designed to be upgraded to use `zk light client roots` in future iterations, enabling fully trustless cross-chain verification.

### Current Implementation

| Inputs | Outputs |
|--------|---------|
| Domain merkle proofs | Coprocessor root |
| Domain state roots | Domain state roots |
| | Zero knowledge proof |

### Future Enhancements

- Integration of zk light client roots for trustless verification
- Support for additional blockchain networks
- Optimized proof generation and verification times
- Enhanced security through additional verification layers

## Key Components

### Coprocessor Circuit
- Verifies merkle proofs from multiple domains
- Generates:
  - `new_coprocessor_root`: Updated state root
  - `zk_proof`: Zero-knowledge proof of verification
  - `outputs`: Verified state data
- Designed for cross-domain verification and processing

## Current Limitations and Future Work

### Known Limitations
- Currently uses trusted roots for verification (planned upgrade to zk light client roots)
- Limited to EVM and ICS23 (Cosmos) domains (extensible to additional chains)
- Proof generation time may vary based on state size

### Planned Improvements
- Integration of zk light client roots
- Support for additional blockchain networks
- Performance optimizations for proof generation
- Enhanced security measures
- Additional verification layers

## Common Issues and Solutions

If you encounter a "Failed to get Proof" error on Neutron:
1. Update your `.env` file with the latest Neutron App hash and block height
2. Note: If you select a block with height N, you must provide the app hash for block N + 1

> [!WARNING]
> This repository is under active development. While the core functionality is implemented, some features are still being enhanced and optimized.