use std::{env, str::FromStr};

use alloy::{
    hex::{self, FromHex},
    providers::{Provider, ProviderBuilder},
    sol_types::SolValue,
};
use alloy_primitives::U256;
use base64::Engine;
use common_merkle_proofs::merkle::types::MerkleClient;
use coprocessor_circuit_types::{
    MerkleProofInputs as CoprocessorCircuitInputs, MerkleProofOutputs as CoprocessorCircuitOutputs,
};
use dotenvy::dotenv;
use ethereum_merkle_proofs::{
    ethereum_rpc::rpc::EvmMerkleRpcClient,
    merkle_lib::{
        keccak::digest_keccak,
        types::{EthereumMerkleProof, decode_rlp_bytes},
    },
};
use ics23_merkle_proofs::{
    keys::Ics23Key, merkle_lib::types::Ics23MerkleProof, rpc::Ics23MerkleRpcClient,
};
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use sp1_verifier::Groth16Verifier;
use url::Url;
use valence_smt::MemorySmt;
use zk_rate_application_types::{RateApplicationCircuitInputs, RateApplicationCircuitOutputs};

use crate::{
    COPROCESSOR_CIRCUIT_ELF, RATE_APPLICATION_CIRCUIT_ELF, get_ethereum_height,
    read_ethereum_default_account_address, read_ethereum_rpc_url, read_neutron_app_hash,
    read_neutron_default_account_address, read_neutron_height, read_neutron_rpc_url,
};

pub async fn run_rate_example() {
    let mut smt_tree = MemorySmt::default();
    let mut coprocessor_root = [0; 32];
    let mut ethereum_merkle_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)> =
        Vec::new();
    let mut neutron_merkle_proofs: Vec<Ics23MerkleProof> = Vec::new();

    let neutron_rpc_url = read_neutron_rpc_url();
    let neutron_rpc_client = Ics23MerkleRpcClient {
        rpc_url: neutron_rpc_url,
    };
    let neutron_height = read_neutron_height();
    let neutron_vault_balance_key = Ics23Key::new_wasm_account_mapping(
        b"balances",
        &read_neutron_default_account_address(),
        &read_neutron_vault_example_contract_address(),
    );
    let proof = neutron_rpc_client
        .get_proof(&neutron_vault_balance_key.to_string(), "", neutron_height)
        .await
        .unwrap();
    let neutron_vault_balance_proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
    neutron_merkle_proofs.push(neutron_vault_balance_proof.clone());

    let neutron_vault_shares_key =
        Ics23Key::new_wasm_stored_value("shares", &read_neutron_vault_example_contract_address());
    let proof = neutron_rpc_client
        .get_proof(&neutron_vault_shares_key.to_string(), "", neutron_height)
        .await
        .unwrap();
    let neutron_vault_shares_proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
    neutron_merkle_proofs.push(neutron_vault_shares_proof.clone());

    let ethereum_rpc_url = read_ethereum_rpc_url();
    let ethereum_rpc_client = EvmMerkleRpcClient {
        rpc_url: ethereum_rpc_url,
    };
    let ethereum_height = get_ethereum_height().await;

    let address =
        alloy_primitives::Address::from_hex(read_ethereum_default_account_address()).unwrap();
    let slot: U256 = alloy_primitives::U256::from(0);
    let encoded_key = (address, slot).abi_encode();
    let keccak_key = digest_keccak(&encoded_key).to_vec();
    let (ethereum_vault_balance_account_proof, ethereum_vault_balance_storage_proof) =
        ethereum_rpc_client
            .get_account_and_storage_proof(
                &alloy::hex::encode(&keccak_key),
                &read_ethereum_vault_example_contract_address(),
                ethereum_height,
            )
            .await
            .unwrap();

    let account_decoded = decode_rlp_bytes(&ethereum_vault_balance_account_proof.value).unwrap();
    ethereum_merkle_proofs.push((
        ethereum_vault_balance_account_proof.clone(),
        ethereum_vault_balance_storage_proof.clone(),
        // this is the account hash
        account_decoded.get(2).unwrap().to_vec(),
    ));

    let ethereum_balance_shares_key =
        hex::decode(read_ethereum_vault_balances_storage_key()).unwrap();
    let (ethereum_vault_shares_account_proof, ethereum_vault_shares_storage_proof) =
        ethereum_rpc_client
            .get_account_and_storage_proof(
                &alloy::hex::encode(&ethereum_balance_shares_key),
                &read_ethereum_vault_example_contract_address(),
                ethereum_height,
            )
            .await
            .unwrap();
    let account_decoded = decode_rlp_bytes(&ethereum_vault_shares_account_proof.value).unwrap();

    ethereum_merkle_proofs.push((
        ethereum_vault_shares_account_proof.clone(),
        ethereum_vault_shares_storage_proof.clone(),
        // this is the account hash
        account_decoded.get(2).unwrap().to_vec(),
    ));

    let provider = ProviderBuilder::new().on_http(Url::from_str(&read_ethereum_rpc_url()).unwrap());

    let neutron_root = base64::engine::general_purpose::STANDARD
        .decode(read_neutron_app_hash())
        .unwrap();

    let ethereum_root = provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(ethereum_height))
        .await
        .unwrap()
        .unwrap()
        .header
        .state_root
        .to_vec();

    // build the same SMT outside the circuit - for the demo we do everything
    // in-memory.
    coprocessor_root = smt_tree
        .insert(
            coprocessor_root,
            "demo",
            borsh::to_vec(&ethereum_vault_shares_storage_proof).unwrap(),
        )
        .unwrap();

    coprocessor_root = smt_tree
        .insert(
            coprocessor_root,
            "demo",
            borsh::to_vec(&ethereum_vault_balance_storage_proof).unwrap(),
        )
        .unwrap();

    coprocessor_root = smt_tree
        .insert(
            coprocessor_root,
            "demo",
            borsh::to_vec(&neutron_vault_shares_proof).unwrap(),
        )
        .unwrap();

    coprocessor_root = smt_tree
        .insert(
            coprocessor_root,
            "demo",
            borsh::to_vec(&neutron_vault_balance_proof).unwrap(),
        )
        .unwrap();

    // insert the neutron light client root
    coprocessor_root = smt_tree
        .insert(
            coprocessor_root,
            "demo",
            borsh::to_vec(&neutron_root).unwrap(),
        )
        .unwrap();

    // insert the ethereum light client root
    coprocessor_root = smt_tree
        .insert(
            coprocessor_root,
            "demo",
            borsh::to_vec(&ethereum_root).unwrap(),
        )
        .unwrap();

    let ethereum_root_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&ethereum_root).unwrap(),
        )
        .unwrap()
        .unwrap();

    let neutron_root_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&neutron_root).unwrap(),
        )
        .unwrap()
        .unwrap();

    let coprocessor_circuit_inputs = CoprocessorCircuitInputs {
        ethereum_merkle_proofs: ethereum_merkle_proofs.clone(),
        neutron_merkle_proofs: neutron_merkle_proofs.clone(),
        neutron_root,
        ethereum_root,
        ethereum_root_opening,
        neutron_root_opening,
        coprocessor_root,
    };
    let coprocessor_circuit_inputs_serialized = borsh::to_vec(&coprocessor_circuit_inputs).unwrap();

    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(coprocessor_circuit_inputs_serialized);
    let (pk, vk) = client.setup(COPROCESSOR_CIRCUIT_ELF);

    // prove the coprocessor circuit
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
    let coprocessor_circuit_outputs: CoprocessorCircuitOutputs =
        borsh::from_slice(proof.public_values.as_slice()).unwrap();
    println!(
        "Coprocessor Circuit Outputs: {:?}",
        coprocessor_circuit_outputs
    );

    // get the SMT openings that will be part of the input for our example application
    let neutron_balance_smt_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&neutron_vault_balance_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get neutron balance opening");

    let neutron_shares_smt_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&neutron_vault_shares_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get neutron shares opening");

    let ethereum_balance_smt_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&ethereum_vault_balance_storage_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get ethereum balance opening");

    let ethereum_shares_smt_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&ethereum_vault_shares_storage_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get ethereum shares opening");

    // call the example application circuit with all the inputs
    let rate_application_circuit_inputs = RateApplicationCircuitInputs {
        neutron_vault_balance_opening: neutron_balance_smt_opening,
        neutron_vault_shares_opening: neutron_shares_smt_opening,
        ethereum_vault_balance_opening: ethereum_balance_smt_opening,
        ethereum_vault_shares_opening: ethereum_shares_smt_opening,
        coprocessor_root: coprocessor_root,
    };

    let client = ProverClient::new();
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
fn read_ethereum_vault_balances_storage_key() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_VAULT_BALANCES_STORAGE_KEY")
        .expect("Missing Sepolia Vault Balances Storage Key!")
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
