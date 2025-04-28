use crate::{
    MAILBOX_APPLICATION_CIRCUIT_ELF,
    clients::{ClientInterface, DefaultClient},
    coprocessor::{Coprocessor, CoprocessorInterface},
};
use alloy::sol_types::SolValue;
use alloy_primitives::U256;
use dotenvy::dotenv;
use ethereum_merkle_proofs::merkle_lib::keccak::digest_keccak;
use ics23_merkle_proofs::keys::Ics23Key;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin, client};
use sp1_verifier::Groth16Verifier;
use std::env;
use zk_mailbox_application_types::{
    MailboxApplicationCircuitInputs, MailboxApplicationCircuitOutputs,
};

pub async fn prove(client: DefaultClient) {
    // required neutron storage key(s)
    let (neutron_root, neutron_height) = client.neutron_client.get_latest_root_and_height().await;
    let neutron_mailbox_messages_key = Ics23Key::new_wasm_account_mapping(
        b"messages",
        "1",
        &read_neutron_mailbox_example_contract_address(),
    );

    // required ethereum storage key(s)
    let (ethereum_root, ethereum_height) =
        client.ethereum_client.get_latest_root_and_height().await;
    let slot: U256 = alloy_primitives::U256::from(0);
    let counter = U256::from(1);
    let encoded_key = (counter, slot).abi_encode();
    let ethereum_mailbox_messages_key = digest_keccak(&encoded_key).to_vec();

    let mut coprocessor = Coprocessor::from_env();
    let domain_state_proofs = coprocessor
        .get_storage_merkle_proofs(
            neutron_height,
            ethereum_height,
            vec![neutron_mailbox_messages_key],
            vec![(
                ethereum_mailbox_messages_key,
                read_ethereum_mailbox_example_contract_address(),
            )],
        )
        .await;
}

/// Reads the Ethereum mailbox example contract address from environment variables
///
/// # Returns
/// The Ethereum mailbox contract address as a String
fn read_ethereum_mailbox_example_contract_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_MAILBOX_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Sepolia Mailbox Contract Address!")
}

/// Reads the Neutron mailbox example contract address from environment variables
///
/// # Returns
/// The Neutron mailbox contract address as a String
fn read_neutron_mailbox_example_contract_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_PION_1_MAILBOX_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Pion 1 Mailbox Contract Address!")
}
