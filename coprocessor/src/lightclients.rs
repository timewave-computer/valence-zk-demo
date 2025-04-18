/* Zero Knowledge Light Client integrations for supported domains
In order for valence-zk to be fully secure, we need to be able to verify that the roots we are using in the coprocessor circuit are actually
valid roots for the domains at the given heights.

This can be achieved by requesting zero knowledge light client proofs at the respective heights and verifying them in the coprocessor circuit.
The verification logic for those will depend on the specific light client impelementation, but for Ethereum and Ics23 Cosmos chains we can
use existing SP1 implementations.

See an Ethereum ZK light client here:
https://github.com/jonas089/spectre-rad

See an Ics23 Cosmos ZK light client here:
https://github.com/succinctlabs/tendermintx

To get this zk-valence demo to production, we are required to:
- Deploy a coprocessor state contract, which is ONCE initialized with the genesis state that we choose for Ethereum and Neutron e.g. roots for Height E and Height N.
- Implement the light client interface located in this filefor these zk light clients and remove the "Mock" prefix.
- Implement the proof verification logic in the coprocessor circuit for these zk light clients.

DONE! At this point we are ready to ship valence-zk in production on Ethereum and Cosmos Ics23 (tendermint)chains.
Initially we will want to deploy tendermintx for Neutron, but each additional tendermint chain will only require its own contract
and instance of the tendermintx prover.
*/

use alloy;
use alloy::providers::{Provider, ProviderBuilder};
use base64::Engine;
use std::str::FromStr;
use tendermint_rpc::{Client, Url as TendermintUrl};

/// Trait defining the interface for interacting with a Neutron light client
pub trait MockNeutronLightClientInterface {
    /// Retrieves the latest root hash and block height from the Neutron chain
    ///
    /// # Returns
    /// A tuple containing:
    /// - The root hash as a byte vector
    /// - The block height as a u64
    async fn get_latest_neutron_root_and_height(&self) -> (Vec<u8>, u64);
    // for a real zk light client we likely want a function like:
    // async fn get_neutron_root_and_proof_at_height(&self, height: u64) -> (Vec<u8>, LightClientProof);
    // this light client proof will then be verified in the coprocessor circuit against the previous ethereum state
}

/// Trait defining the interface for interacting with an Ethereum light client
pub trait MockEthereumLightClientInterface {
    /// Retrieves the latest root hash and block height from the Ethereum chain
    ///
    /// # Returns
    /// A tuple containing:
    /// - The root hash as a byte vector
    /// - The block height as a u64
    async fn get_latest_ethereum_root_and_height(&self) -> (Vec<u8>, u64);
    // for a real zk light client we likely want a function like:
    // async fn get_neutron_root_and_proof_at_height(&self, height: u64) -> (Vec<u8>, LightClientProof);
    // this light client proof will then be verified in the coprocessor circuit against the previous neutron state
}

/// Mock implementation of a Neutron light client
#[derive(Debug, Clone)]
pub struct MockNeutronLightClient {
    /// RPC URL for connecting to the Neutron chain
    pub rpc_url: String,
}

/// Mock implementation of an Ethereum light client
#[derive(Debug, Clone)]
pub struct MockEthereumLightClient {
    /// RPC URL for connecting to the Ethereum chain
    pub rpc_url: String,
}

/// Combined mock light client implementation for both Neutron and Ethereum
#[derive(Debug, Clone)]
pub struct MockLightClient {
    /// Instance of the Neutron light client
    pub neutron_light_client: MockNeutronLightClient,
    /// Instance of the Ethereum light client
    pub ethereum_light_client: MockEthereumLightClient,
}

impl MockNeutronLightClientInterface for MockLightClient {
    /// Implementation of the Neutron light client interface
    ///
    /// Connects to the Neutron chain via RPC and retrieves the latest block's app hash and height
    ///
    /// # Returns
    /// A tuple containing:
    /// - The app hash as a byte vector
    /// - The block height as a u64
    async fn get_latest_neutron_root_and_height(&self) -> (Vec<u8>, u64) {
        let tendermint_client = tendermint_rpc::HttpClient::new(
            TendermintUrl::from_str(&self.neutron_light_client.rpc_url).unwrap(),
        )
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

impl MockEthereumLightClientInterface for MockLightClient {
    /// Implementation of the Ethereum light client interface
    ///
    /// Connects to the Ethereum chain via RPC and retrieves the latest block's state root and height
    ///
    /// # Returns
    /// A tuple containing:
    /// - The state root as a byte vector
    /// - The block height as a u64
    async fn get_latest_ethereum_root_and_height(&self) -> (Vec<u8>, u64) {
        let provider = ProviderBuilder::new()
            .on_http(url::Url::from_str(&self.ethereum_light_client.rpc_url).unwrap());
        let block = provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Latest)
            .await
            .unwrap()
            .expect("Failed to get Block!");
        let ethereum_root = block.header.state_root.to_vec();
        (ethereum_root, block.header.number)
    }
}
