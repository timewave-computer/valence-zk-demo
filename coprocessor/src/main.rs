use alloy::transports::http::reqwest;
use constants::{ETHEREUM_HEIGHT_KEY, ETHEREUM_ROOT_KEY, NEUTRON_HEIGHT_KEY, NEUTRON_ROOT_KEY};
use coprocessor::Coprocessor;
use dotenvy::dotenv;
#[cfg(feature = "mailbox")]
use examples::mailbox;
use examples::prove_coprocessor;
mod clients;
mod coprocessor;
mod lightclients;
use clients::{ClientInterface, DefaultClient, EthereumClient, NeutronClient};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sp1_sdk::include_elf;
use ssz_merkleize::merkleize::get_beacon_block_header;
mod constants;
use std::env;
mod examples;
pub const COPROCESSOR_CIRCUIT_ELF: &[u8] = include_elf!("coprocessor-circuit-sp1");
pub const MAILBOX_APPLICATION_CIRCUIT_ELF: &[u8] = include_elf!("zk-mailbox-application");

#[tokio::main]
async fn main() {
    let mut coprocessor = Coprocessor::from_env();
    let default_client = DefaultClient {
        neutron_client: NeutronClient {
            rpc_url: read_neutron_rpc_url(),
        },
        ethereum_client: EthereumClient {
            rpc_url: read_ethereum_rpc_url(),
        },
    };
    let neutron_target_block_height: u64 = default_client
        .neutron_client
        .get_latest_root_and_height()
        .await
        .1;
    let neutron_example_trusted_height: u64 = neutron_target_block_height - 10;
    coprocessor.target_neutron_height = neutron_target_block_height;
    coprocessor.trusted_neutron_height = neutron_example_trusted_height;
    let neutron_trusted_root = default_client
        .neutron_client
        .get_state_at_height(neutron_example_trusted_height)
        .await
        .0;
    // initialize the trusted root for neutron
    coprocessor.trusted_neutron_root = neutron_trusted_root;
    // compute the coprocessor update
    let coprocessor_outputs = prove_coprocessor(&mut coprocessor).await;
    let neutron_header = default_client
        .neutron_client
        .get_header_at_height(coprocessor_outputs.0.target_height)
        .await;
    let beacon_header =
        get_beacon_block_header(coprocessor_outputs.1.newHead.try_into().unwrap()).await;
    // pass the headers and proof outputs to the application circuit
    let coprocessor_smt_root = coprocessor.smt_root;

    let mut hasher = Sha256::new();
    hasher.update(NEUTRON_HEIGHT_KEY);
    let neutron_height_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(ETHEREUM_HEIGHT_KEY);
    let ethereum_height_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(NEUTRON_ROOT_KEY);
    let neutron_root_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(ETHEREUM_ROOT_KEY);
    let ethereum_root_key = hasher.finalize();

    let neutron_height_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_smt_root, &neutron_height_key)
        .expect("Failed to get neutron height opening")
        .unwrap();
    let ethereum_height_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_smt_root, &ethereum_height_key)
        .expect("Failed to get ethereum height opening")
        .unwrap();
    let neutron_root_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_smt_root, &neutron_root_key)
        .expect("Failed to get neutron root opening")
        .unwrap();
    let ethereum_root_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_smt_root, &ethereum_root_key)
        .expect("Failed to get ethereum root opening")
        .unwrap();
    // now pass the smt openings to the applications
    #[cfg(feature = "mailbox")]
    mailbox::prove(
        neutron_height_opening,
        ethereum_height_opening,
        neutron_root_opening,
        ethereum_root_opening,
        neutron_header,
        beacon_header,
    )
    .await;
}

pub async fn get_execution_block_height(
    beacon_node_url: &str,
    slot: u64,
) -> Result<u64, Box<dyn std::error::Error>> {
    let url = format!("{}/eth/v2/beacon/blocks/{}", beacon_node_url, slot);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?.error_for_status()?;
    let json: Value = res.json().await?;
    let block_number_hex = json["data"]["message"]["body"]["execution_payload"]["block_number"]
        .as_str()
        .ok_or("Missing block_number")?;
    let block_number = u64::from_str_radix(block_number_hex.trim_start_matches("0x"), 16)?;
    Ok(block_number)
}

/// Reads the Neutron RPC URL from environment variables
///
/// # Returns
/// The Neutron RPC URL as a String
pub(crate) fn read_neutron_rpc_url() -> String {
    dotenvy::dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

/// Reads the Ethereum RPC URL from environment variables
///
/// # Returns
/// The Ethereum RPC URL as a String
pub(crate) fn read_ethereum_rpc_url() -> String {
    dotenv().ok();
    env::var("ETHEREUM_URL").expect("Missing Sepolia url!")
}

pub(crate) fn read_ethereum_consensus_rpc_url() -> String {
    dotenv().ok();
    env::var("SOURCE_CONSENSUS_RPC_URL").expect("Missing Consensus url!")
}
