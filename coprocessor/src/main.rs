use base64::Engine;
use dotenvy::dotenv;
#[cfg(feature = "mailbox")]
use examples::mailbox;
#[cfg(feature = "rate")]
use examples::rate;
mod coprocessor;
use sp1_sdk::include_elf;
use std::{env, str::FromStr, time::Instant};
use tendermint_rpc::{Client, Url};
mod examples;
pub const COPROCESSOR_CIRCUIT_ELF: &[u8] = include_elf!("coprocessor-circuit-sp1");
pub const RATE_APPLICATION_CIRCUIT_ELF: &[u8] = include_elf!("zk-rate-application");
pub const MAILBOX_APPLICATION_CIRCUIT_ELF: &[u8] = include_elf!("zk-mailbox-application");

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    #[cfg(feature = "rate")]
    {
        println!("Running rate example");
        rate::prove().await;
    }
    #[cfg(feature = "mailbox")]
    {
        println!("Running mailbox example");
        mailbox::prove().await;
    }
    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("Time taken: {:?}", duration);
}

/// Reads the Neutron RPC URL from environment variables
///
/// # Returns
/// The Neutron RPC URL as a String
pub(crate) fn read_neutron_rpc_url() -> String {
    dotenvy::dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

/// Reads the Neutron app hash from environment variables
///
/// # Returns
/// The Neutron app hash as a String
pub(crate) async fn get_latest_neutron_app_hash_and_height() -> (String, u64) {
    let tendermint_client =
        tendermint_rpc::HttpClient::new(Url::from_str(&read_neutron_rpc_url()).unwrap()).unwrap();
    let latest_block = tendermint_client.latest_block().await.unwrap();
    let height = latest_block.block.header.height.value() - 1;
    let app_hash = base64::engine::general_purpose::STANDARD
        .encode(hex::decode(latest_block.block.header.app_hash.to_string()).unwrap());

    (app_hash, height)
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

/// Gets the latest Ethereum block height from the RPC provider
///
/// # Returns
/// The latest Ethereum block height as a u64
pub(crate) async fn get_ethereum_height() -> u64 {
    use alloy;
    use alloy::providers::{Provider, ProviderBuilder};
    use std::str::FromStr;
    use url::Url;
    let provider = ProviderBuilder::new().on_http(Url::from_str(&read_ethereum_rpc_url()).unwrap());
    let block = provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Latest)
        .await
        .unwrap()
        .expect("Failed to get Block!");
    block.header.number
}
