use dotenvy::dotenv;
#[cfg(feature = "mailbox")]
use mailbox::run_mailbox_example;
#[cfg(feature = "rate")]
use rate::run_rate_example;
use sp1_sdk::include_elf;
use std::env;

#[cfg(feature = "mailbox")]
mod mailbox;
#[cfg(feature = "rate")]
mod rate;

pub const COPROCESSOR_CIRCUIT_ELF: &[u8] = include_elf!("coprocessor-circuit-sp1");
pub const RATE_APPLICATION_CIRCUIT_ELF: &[u8] = include_elf!("zk-rate-application");
pub const MAILBOX_APPLICATION_CIRCUIT_ELF: &[u8] = include_elf!("zk-mailbox-application");

#[tokio::main]
async fn main() {
    #[cfg(feature = "rate")]
    {
        println!("Running rate example");
        run_rate_example().await;
    }
    #[cfg(feature = "mailbox")]
    {
        println!("Running mailbox example");
        run_mailbox_example().await;
    }
}

/// Reads the Neutron RPC URL from environment variables
///
/// # Returns
/// The Neutron RPC URL as a String
pub(crate) fn read_neutron_rpc_url() -> String {
    dotenvy::dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

/// Reads the Neutron block height from environment variables
///
/// # Returns
/// The Neutron block height as a u64
pub(crate) fn read_neutron_height() -> u64 {
    dotenv().ok();
    env::var("HEIGHT_NEUTRON")
        .expect("Missing Neutron TEST VECTOR: HEIGHT!")
        .parse::<u64>()
        .expect("Failed to parse test vector as u64: Amount")
}

/// Reads the Neutron app hash from environment variables
///
/// # Returns
/// The Neutron app hash as a String
pub(crate) fn read_neutron_app_hash() -> String {
    dotenv().ok();
    env::var("MERKLE_ROOT_NEUTRON").expect("Missing Neutron TEST VECTOR: ROOT!")
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
