/*! Zero Knowledge Light Client integrations for supported domains.

This module provides interfaces and implementations for zero-knowledge light clients that verify
blockchain state roots for different domains. These light clients are essential for ensuring
the security of the valence-zk system by verifying that the roots used in the coprocessor circuit
are valid for their respective domains at specific heights.

## Supported Domains
- Ethereum: Uses SP1 implementation from [spectre-rad](https://github.com/jonas089/spectre-rad)
- Cosmos Ics23 (Tendermint): Uses SP1 implementation from [tendermintx](https://github.com/succinctlabs/tendermintx)

## Production Requirements
To deploy valence-zk in production, the following steps are required:
1. Deploy a coprocessor state contract initialized with genesis states for supported chains
2. Implement the light client interface for zero-knowledge light clients
3. Implement proof verification logic in the coprocessor circuit

## Current Implementation
This module currently provides mock implementations for testing purposes. These should be replaced
with actual zero-knowledge light client implementations before production deployment.
*/

use alloy;
use alloy::providers::{Provider, ProviderBuilder};
use base64::Engine;
use std::str::FromStr;
use tendermint_rpc::{Client, Url as TendermintUrl};

/// Trait defining the interface for light client implementations.
///
/// This trait provides a common interface for retrieving the latest state root
/// and block height from different blockchain networks.
pub trait MockLightClientInterface {
    /// Retrieves the latest state root and block height from the light client.
    ///
    /// # Returns
    /// A tuple containing:
    /// - `Vec<u8>`: The latest state root as a byte vector
    /// - `u64`: The latest block height
    async fn get_latest_root_and_height(&self) -> (Vec<u8>, u64);
}

/// Mock implementation of a light client for Neutron (Tendermint-based chain).
///
/// This implementation uses Tendermint RPC to fetch the latest block information
/// and extract the app hash as the state root.
#[derive(Debug, Clone)]
pub struct MockNeutronLightClient {
    /// The RPC URL endpoint for the Neutron node
    pub rpc_url: String,
}

impl MockLightClientInterface for MockNeutronLightClient {
    async fn get_latest_root_and_height(&self) -> (Vec<u8>, u64) {
        let tendermint_client =
            tendermint_rpc::HttpClient::new(TendermintUrl::from_str(&self.rpc_url).unwrap())
                .unwrap();
        let latest_block = tendermint_client.latest_block().await.unwrap();
        let height = latest_block.block.header.height.value() - 1;
        let app_hash = base64::engine::general_purpose::STANDARD
            .encode(hex::decode(latest_block.block.header.app_hash.to_string()).unwrap());
        (
            base64::engine::general_purpose::STANDARD
                .decode(app_hash)
                .unwrap(),
            height,
        )
    }
}

/// Mock implementation of a light client for Ethereum.
///
/// This implementation uses Ethereum RPC to fetch the latest block information
/// and extract the state root.
#[derive(Debug, Clone)]
pub struct MockEthereumLightClient {
    /// The RPC URL endpoint for the Ethereum node
    pub rpc_url: String,
}

impl MockLightClientInterface for MockEthereumLightClient {
    async fn get_latest_root_and_height(&self) -> (Vec<u8>, u64) {
        let provider = ProviderBuilder::new().on_http(url::Url::from_str(&self.rpc_url).unwrap());
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Latest)
            .await
            .unwrap()
            .expect("Failed to get Block!");
        let ethereum_root = block.header.state_root.to_vec();
        (ethereum_root, block.header.number)
    }
}

/// Combined mock light client implementation for both Neutron and Ethereum.
///
/// This struct provides a unified interface to interact with both Neutron and
/// Ethereum light clients. It's primarily used for testing and development purposes.
#[derive(Debug, Clone)]
pub struct MockLightClient {
    /// Instance of the Neutron light client
    pub neutron_light_client: MockNeutronLightClient,
    /// Instance of the Ethereum light client
    pub ethereum_light_client: MockEthereumLightClient,
}
