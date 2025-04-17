use std::{env, str::FromStr};

use crate::{
    RATE_APPLICATION_CIRCUIT_ELF,
    coprocessor::{Coprocessor, EthereumCoprocessor, NeutronCoprocessor},
    get_ethereum_height, read_ethereum_default_account_address, read_ethereum_rpc_url,
    read_neutron_app_hash, read_neutron_default_account_address, read_neutron_height,
};
use alloy::{
    hex::{self, FromHex},
    providers::{Provider, ProviderBuilder},
    sol_types::SolValue,
};
use alloy_primitives::U256;
use base64::Engine;
use coprocessor_circuit_types::MerkleProofOutputs as CoprocessorCircuitOutputs;
use dotenvy::dotenv;
use ethereum_merkle_proofs::merkle_lib::keccak::digest_keccak;
use ics23_merkle_proofs::keys::Ics23Key;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use sp1_verifier::Groth16Verifier;
use url::Url;
use zk_rate_application_types::{RateApplicationCircuitInputs, RateApplicationCircuitOutputs};

pub async fn prove() {
    let neutron_height = read_neutron_height();
    let neutron_vault_balance_key = Ics23Key::new_wasm_account_mapping(
        b"balances",
        &read_neutron_default_account_address(),
        &read_neutron_vault_example_contract_address(),
    );
    let neutron_root = base64::engine::general_purpose::STANDARD
        .decode(read_neutron_app_hash())
        .unwrap();
    let neutron_vault_shares_key =
        Ics23Key::new_wasm_stored_value("shares", &read_neutron_vault_example_contract_address());
    // required ethereum storage key(s)
    let provider = ProviderBuilder::new().on_http(Url::from_str(&read_ethereum_rpc_url()).unwrap());
    let ethereum_height = get_ethereum_height().await;
    let ethereum_root = provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(ethereum_height))
        .await
        .unwrap()
        .unwrap()
        .header
        .state_root
        .to_vec();
    let address =
        alloy_primitives::Address::from_hex(read_ethereum_default_account_address()).unwrap();
    let slot: U256 = alloy_primitives::U256::from(0);
    let encoded_key = (address, slot).abi_encode();
    let ethereum_vault_balances_key = digest_keccak(&encoded_key).to_vec();
    let ethereum_vault_contract_address = read_ethereum_vault_example_contract_address();
    let ethereum_vault_shares_key = hex::decode(read_ethereum_vault_shares_storage_key()).unwrap();
    let mut coprocessor = Coprocessor::from_env_with_storage_keys(
        vec![neutron_vault_balance_key, neutron_vault_shares_key],
        vec![
            (
                ethereum_vault_balances_key,
                ethereum_vault_contract_address.clone(),
            ),
            (ethereum_vault_shares_key, ethereum_vault_contract_address),
        ],
    );
    let merkle_proofs = coprocessor
        .get_storage_merkle_proofs(neutron_height, ethereum_height)
        .await;

    let (proof, vk) = coprocessor
        .prove_progression(
            merkle_proofs.0.clone(),
            merkle_proofs.1.clone(),
            ethereum_root,
            neutron_root,
        )
        .await;
    let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
    Groth16Verifier::verify(
        &proof.bytes(),
        &proof.public_values.to_vec(),
        &vk.bytes32(),
        groth16_vk,
    )
    .unwrap();
    let coprocessor_circuit_outputs: CoprocessorCircuitOutputs =
        borsh::from_slice(proof.public_values.as_slice()).unwrap();
    println!(
        "Coprocessor Circuit Outputs: {:?}",
        coprocessor_circuit_outputs
    );

    // get the SMT openings that will be part of the input for our example application
    let neutron_balance_smt_opening =
        coprocessor.get_neutron_opening(&borsh::to_vec(&merkle_proofs.0.first().unwrap()).unwrap());
    let neutron_shares_smt_opening =
        coprocessor.get_neutron_opening(&borsh::to_vec(&merkle_proofs.0.last().unwrap()).unwrap());
    let ethereum_balance_smt_opening = coprocessor
        .get_ethereum_opening(&borsh::to_vec(&merkle_proofs.1.first().unwrap().1).unwrap());
    let ethereum_shares_smt_opening = coprocessor
        .get_ethereum_opening(&borsh::to_vec(&merkle_proofs.1.last().unwrap().1).unwrap());

    // call the example application circuit with all the inputs
    let rate_application_circuit_inputs = RateApplicationCircuitInputs {
        neutron_vault_balance_opening: neutron_balance_smt_opening,
        neutron_vault_shares_opening: neutron_shares_smt_opening,
        ethereum_vault_balance_opening: ethereum_balance_smt_opening,
        ethereum_vault_shares_opening: ethereum_shares_smt_opening,
        coprocessor_root: coprocessor.smt_root,
    };

    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(
        borsh::to_vec(&rate_application_circuit_inputs)
            .expect("Failed to serialize rate application inputs"),
    );
    let (pk, vk) = client.setup(RATE_APPLICATION_CIRCUIT_ELF);
    let proof = client
        .prove(&pk, &stdin)
        .groth16()
        .run()
        .expect("Failed to prove");

    let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
    Groth16Verifier::verify(
        &proof.bytes(),
        &proof.public_values.to_vec(),
        &vk.bytes32(),
        groth16_vk,
    )
    .unwrap();
    let rate_application_circuit_outputs: RateApplicationCircuitOutputs =
        borsh::from_slice(proof.public_values.as_slice()).unwrap();
    println!(
        "Rate Application Outputs: {:?}",
        rate_application_circuit_outputs
    );
}

/// Reads the Ethereum vault example contract address from environment variables
///
/// # Returns
/// The Ethereum vault contract address as a String
fn read_ethereum_vault_example_contract_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_VAULT_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Sepolia Vault Contract Address!")
}

/// Reads the Ethereum vault balances storage key from environment variables
///
/// # Returns
/// The Ethereum vault balances storage key as a String
fn read_ethereum_vault_shares_storage_key() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_VAULT_SHARES_STORAGE_KEY")
        .expect("Missing Sepolia Vault Shares Storage Key!")
}

/// Reads the Neutron vault example contract address from environment variables
///
/// # Returns
/// The Neutron vault contract address as a String
fn read_neutron_vault_example_contract_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_PION_1_VAULT_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Pion 1 Vault Contract Address!")
}
