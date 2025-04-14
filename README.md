# ZK Valence Programs Demo

This repository contains an end-to-end demonstration of ZK valence programs across different domains. The initial implementation focuses on applications that interact with Ethereum and Neutron Smart Contract state.

## Overview

The first iteration of our demo implements the following workflow:

1. Populates the coprocessor tree with values from Ethereum and Neutron Smart Contracts
2. Generates opening proofs against the co-processor root
3. Provides these opening proofs as context, along with the co-processor SMT root, to the Valence ZK program

## Architecture

The coprocessor verifies original merkle proofs for respective domains against `trusted roots`. These `trusted roots` will be replaced with `zk light client roots` in future iterations.

The verification of merkle proofs occurs in a `recursive zk circuit` with the following structure:

| Inputs | Outputs |
|--------|---------|
| Domain merkle proofs | Coprocessor root |
| Domain state roots | Domain state roots |
| | Zero knowledge proof |

## Key Components

- The `coprocessor-circuit` in `/coprocessor-proofs`:
  - Verifies merkle proofs from domains
  - Produces:
    - `new_coprocessor_root`
    - `zk_proof`
    - `outputs`
  - These outputs will be verified and processed across all supported domains

## Common Issues

If you encounter a "Failed to get Proof" error on Neutron:
1. Update your `.env` file with the latest Neutron App hash and block height
2. Note: If you select a block with height N, you must provide the app hash for block N + 1

> [!WARNING]
> This repository is under heavy development and changes frequently.