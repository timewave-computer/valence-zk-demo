# Valence ZK: Trustless Cross-Chain Development Framework

Valence ZK is a revolutionary framework that simplifies cross-blockchain development by abstracting complex cryptographic operations into a developer-friendly interface. By leveraging zero-knowledge proofs, Valence enables trustless verification of state across multiple blockchain ecosystems, making it possible to build truly decentralized cross-chain applications.

## Key Features

- **Trustless Cross-Chain Verification**: Verify state across different blockchains without relying on trusted intermediaries
- **Developer-First Design**: Abstract complex cryptographic operations like Merkle proofs into simple, composable interfaces
- **Modular Architecture**: Extensible trait system allows easy integration with new blockchain networks
- **Production-Ready**: Fully implemented core functionality with support for Ethereum and Cosmos-based chains
- **Future-Proof**: Designed for seamless integration with upcoming zk light client proofs

## Technical Architecture

Valence ZK implements a sophisticated architecture that combines:

1. **Merkle Proof Library**: Generic interface for Merkle proofs across supported chains
   - Account proofs
   - Storage proofs
   - Future: Receipt proofs (ERC20 Transfer logs, L2 chains)

2. **Coprocessor Layer**: 
   - Batches Merkle proofs by target domain
   - Performs recursive ZK verification
   - Maintains a Sparse Merkle Tree (SMT) for trusted roots

3. **Recursive ZK Circuit**:
   - Domain State Proofs: Verifies individual domain-level Merkle proofs
   - SMT Update Proofs: Verifies updates to the trusted root SMT
   - Currently implemented using SP1 prover with Arkworks verifier

The framework currently supports:
- Ethereum (EVM-compatible chains)
- Neutron (Cosmos ecosystem)
- Extensible to any ICS23 or EVM-compatible chain

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

#### Cross-Chain Rate Calculation
```bash
cargo run -p coprocessor --release --features rate -- --nocapture
```

#### Cross-Chain Message Mailbox
```bash
cargo run -p coprocessor --release --features mailbox -- --nocapture
```

#### Advanced: Coprocessor Proof Generation
For production-grade security guarantees:
```bash
cargo run -p coprocessor --release --features rate --features coprocessor -- --nocapture
```

## Project Structure

- `coprocessor/`: Core coprocessor logic and proof generation
- `coprocessor-proofs/`: ZK circuit implementations
  - `coprocessor-circuit-types/`: Type-safe circuit definitions
  - `coprocessor-circuit-sp1/`: Optimized SP1 implementation
  - `coprocessor-circuit-logic/`: Core verification logic
- `zk-programs/`: Example ZK applications
  - `zk-rate-application/`: Cross-chain rate calculation
  - `zk-rate-application-types/`: Type definitions
  - `zk-mailbox-application/`: Cross-chain messaging

## Example: Cross-Chain Vault Rate Calculation

The framework includes a practical example demonstrating trustless rate calculation across Ethereum and Neutron vaults. This example showcases:

1. Trustless state verification from multiple blockchains
2. Secure rate calculation using zero-knowledge proofs
3. Practical implementation of cross-chain financial primitives

For detailed implementation details, see the [ZK Rate Example README](./zk-programs/zk-rate-example/README.md).

## Security and Trust Model

Valence ZK implements a robust security model:

- **Trusted Roots**: Currently uses trusted roots for verification (planned upgrade to zk light client roots)
- **Recursive ZK Circuits**: Verifies Merkle proofs against trusted roots
- **Future-Proof**: Designed for seamless integration with zk light client proofs

## Development Roadmap

### Current Focus
- Integration of zk light client roots
- Production-ready SMT implementation
- Enhanced serialization and type system abstractions
- Performance optimization of recursive proofs

### Future Enhancements
- Support for additional blockchain networks
- Performance optimizations
- Enhanced security measures
- Additional verification layers
- Support for receipt proofs (ERC20 events, L2 chains)

## Contributing

We welcome contributions to the Valence ZK ecosystem. Please see our [contribution guidelines](CONTRIBUTING.md) for more information.

## License

This project is licensed under the [MIT License](LICENSE).

> [!NOTE]
> This repository is under active development. While core functionality is production-ready, some features are being enhanced for optimal performance and security.