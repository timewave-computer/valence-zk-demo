//! Client implementations for supported domains.
//!
//! This module provides client implementations for interacting with different blockchain networks
//! in a zero-knowledge context. Currently supports Neutron and Ethereum networks.

use alloy;
use alloy::providers::{Provider, ProviderBuilder};
use base64::Engine;
use std::str::FromStr;
use tendermint_rpc::{Client, Url as TendermintUrl};

/// Trait defining the interface for blockchain clients
///
/// This trait provides a common interface for retrieving the latest state root and block height
/// from different blockchain networks.
pub trait ClientInterface {
    /// Retrieves the latest state root and block height from the blockchain
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The state root as a byte vector
    /// - The block height as a u64
    async fn get_latest_root_and_height(&self) -> (Vec<u8>, u64);
}

/// Client implementation for interacting with the Neutron blockchain
#[derive(Debug, Clone)]
pub struct NeutronClient {
    /// The RPC URL endpoint for connecting to the Neutron node
    pub rpc_url: String,
}

impl ClientInterface for NeutronClient {
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

impl NeutronClient {
    pub async fn get_state_at_height(&self, height: u64) -> (Vec<u8>, u64) {
        let tendermint_client =
            tendermint_rpc::HttpClient::new(TendermintUrl::from_str(&self.rpc_url).unwrap())
                .unwrap();
        let latest_block = tendermint_client.latest_block().await.unwrap();
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

/// Client implementation for interacting with the Ethereum blockchain
#[derive(Debug, Clone)]
pub struct EthereumClient {
    /// The RPC URL endpoint for connecting to the Ethereum node
    pub rpc_url: String,
}

impl ClientInterface for EthereumClient {
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

/// A composite client that combines both Neutron and Ethereum clients
///
/// This struct provides a unified interface for interacting with both blockchain networks
/// simultaneously, which is useful for cross-chain operations.
#[derive(Debug, Clone)]
pub struct DefaultClient {
    /// Instance of the Neutron client
    pub neutron_client: NeutronClient,
    /// Instance of the Ethereum client
    pub ethereum_client: EthereumClient,
}
