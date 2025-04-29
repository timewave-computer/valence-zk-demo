use constants::{ETHEREUM_HEIGHT_KEY, ETHEREUM_ROOT_KEY, NEUTRON_HEIGHT_KEY, NEUTRON_ROOT_KEY};
use coprocessor::Coprocessor;
use dotenvy::dotenv;
#[cfg(feature = "mailbox")]
use examples::mailbox;
use examples::prove_coprocessor;
#[cfg(feature = "rate")]
use examples::rate;
mod clients;
mod coprocessor;
mod lightclients;
use clients::{ClientInterface, DefaultClient, EthereumClient, NeutronClient};
use sp1_sdk::include_elf;
use ssz_merkleize::merkleize::get_beacon_block_header;
mod constants;
use std::env;
mod examples;
pub const COPROCESSOR_CIRCUIT_ELF: &[u8] = include_elf!("coprocessor-circuit-sp1");
pub const RATE_APPLICATION_CIRCUIT_ELF: &[u8] = include_elf!("zk-rate-application");
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
    coprocessor.trusted_neutron_root = neutron_trusted_root.try_into().unwrap();
    // compute the coprocessor update
    let coprocessor_outputs = prove_coprocessor(&mut coprocessor).await;



    let neutron_header = default_client
        .neutron_client
        .get_header_at_height(coprocessor_outputs.0.target_height)
        .await;
    //let tendermint_header_hash = tendermint_header.hash();

    let beacon_header = get_beacon_block_header(coprocessor_outputs.1.newHead.try_into().unwrap()).await;
    /*let target_header_root = merkleize_keys(vec![
        uint64_to_le_256(target_beaecon_header.slot.parse::<u64>().unwrap()),
        uint64_to_le_256(target_beaecon_header.proposer_index.parse::<u64>().unwrap()),
        alloy::hex::decode(target_beaecon_header.parent_root)
            .unwrap()
            .to_vec(),
        alloy::hex::decode(target_beaecon_header.state_root)
            .unwrap()
            .to_vec(),
        alloy::hex::decode(target_beaecon_header.body_root)
            .unwrap()
            .to_vec(),
    ]);*/






    // pass the headers and proof outputs to the application circuit
    let coprocessor_smt_root = coprocessor.smt_root;
    let neutron_height_opening = coprocessor.smt_tree.get_opening("demo", coprocessor_smt_root, NEUTRON_HEIGHT_KEY).expect("Failed to get neutron height opening").unwrap();
    let ethereum_height_opening = coprocessor.smt_tree.get_opening("demo", coprocessor_smt_root, ETHEREUM_HEIGHT_KEY).expect("Failed to get ethereum height opening").unwrap();
    let neutron_root_opening = coprocessor.smt_tree.get_opening("demo", coprocessor_smt_root, NEUTRON_ROOT_KEY).expect("Failed to get neutron root opening").unwrap();
    let ethereum_root_opening = coprocessor.smt_tree.get_opening("demo", coprocessor_smt_root, ETHEREUM_ROOT_KEY).expect("Failed to get ethereum root opening").unwrap();
    // now pass the smt openings to the applications 
    #[cfg(feature = "mailbox")]
    mailbox::prove(default_client, neutron_height_opening, ethereum_height_opening, neutron_root_opening, ethereum_root_opening, neutron_header, beacon_header).await;

}

/// Reads the Neutron RPC URL from environment variables
///
/// # Returns
/// The Neutron RPC URL as a String
pub(crate) fn read_neutron_rpc_url() -> String {
    dotenvy::dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

/// Reads the Neutron default account address from environment variables
///
/// # Returns
/// The Neutron default account address as a String
#[cfg(feature = "rate")]
pub(crate) fn read_neutron_default_account_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_DEFAULT_ACCOUNT_ADDRESS").expect("Missing Neutron Default Account Address!")
}

/// Reads the Ethereum default account address from environment variables
///
/// # Returns
/// The Ethereum default account address as a String
#[cfg(feature = "rate")]
pub(crate) fn read_ethereum_default_account_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_DEFAULT_ACCOUNT_ADDRESS").expect("Missing Ethereum Default Account Address!")
}

/// Reads the Ethereum RPC URL from environment variables
///
/// # Returns
/// The Ethereum RPC URL as a String
pub(crate) fn read_ethereum_rpc_url() -> String {
    dotenv().ok();
    env::var("ETHEREUM_URL").expect("Missing Sepolia url!")
}


pub (crate) fn read_ethereum_consensus_rpc_url() -> String{
    dotenv().ok();
    env::var("SOURCE_CONSENSUS_RPC_URL").expect("Missing Consensus url!")
}