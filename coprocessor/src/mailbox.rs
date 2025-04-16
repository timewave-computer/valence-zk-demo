use std::{env, str::FromStr};

use alloy::{
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
use zk_mailbox_application_types::{
    MailboxApplicationCircuitInputs, MailboxApplicationCircuitOutputs,
};

use crate::{
    COPROCESSOR_CIRCUIT_ELF, MAILBOX_APPLICATION_CIRCUIT_ELF, get_ethereum_height,
    read_ethereum_rpc_url, read_neutron_app_hash, read_neutron_height, read_neutron_rpc_url,
};

pub async fn run_mailbox_example() {
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
    let neutron_mailbox_messages_key = Ics23Key::new_wasm_account_mapping(
        b"messages",
        "1",
        &read_neutron_mailbox_example_contract_address(),
    );
    let proof = neutron_rpc_client
        .get_proof(
            &neutron_mailbox_messages_key.to_string(),
            "",
            neutron_height,
        )
        .await
        .unwrap();
    let neutron_mailbox_message_proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
    neutron_merkle_proofs.push(neutron_mailbox_message_proof.clone());

    let ethereum_rpc_url = read_ethereum_rpc_url();
    let ethereum_rpc_client = EvmMerkleRpcClient {
        rpc_url: ethereum_rpc_url,
    };
    let ethereum_height = get_ethereum_height().await;

    let slot: U256 = alloy_primitives::U256::from(0);
    // counter is the index of the message, starts at 1
    let counter = U256::from(1);
    let encoded_key = (counter, slot).abi_encode();
    let keccak_key = digest_keccak(&encoded_key).to_vec();
    let (ethereum_mailbox_message_account_proof, ethereum_mailbox_message_storage_proof) =
        ethereum_rpc_client
            .get_account_and_storage_proof(
                &alloy::hex::encode(&keccak_key),
                &read_ethereum_mailbox_example_contract_address(),
                ethereum_height,
            )
            .await
            .unwrap();

    let account_decoded = decode_rlp_bytes(&ethereum_mailbox_message_account_proof.value).unwrap();
    ethereum_merkle_proofs.push((
        ethereum_mailbox_message_account_proof.clone(),
        ethereum_mailbox_message_storage_proof.clone(),
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
            borsh::to_vec(&ethereum_mailbox_message_storage_proof).unwrap(),
        )
        .unwrap();

    coprocessor_root = smt_tree
        .insert(
            coprocessor_root,
            "demo",
            borsh::to_vec(&neutron_mailbox_message_proof).unwrap(),
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
    let ethereum_message_smt_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&ethereum_mailbox_message_storage_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get neutron balance opening");

    let neutron_message_smt_opening = smt_tree
        .get_opening(
            "demo",
            coprocessor_root,
            &borsh::to_vec(&neutron_mailbox_message_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get neutron shares opening");

    // call the example application circuit with all the inputs
    let rate_application_circuit_inputs = MailboxApplicationCircuitInputs {
        neutron_messages_openings: vec![neutron_message_smt_opening],
        ethereum_messages_openings: vec![ethereum_message_smt_opening],
        coprocessor_root: coprocessor_root,
    };

    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(
        borsh::to_vec(&rate_application_circuit_inputs)
            .expect("Failed to serialize rate application inputs"),
    );
    let (pk, vk) = client.setup(MAILBOX_APPLICATION_CIRCUIT_ELF);

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
    let mailbox_application_circuit_outputs: MailboxApplicationCircuitOutputs =
        borsh::from_slice(proof.public_values.as_slice()).unwrap();
    println!(
        "Cross Chain Messages: {:?}",
        mailbox_application_circuit_outputs
    );
}

fn read_ethereum_mailbox_example_contract_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_MAILBOX_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Sepolia Mailbox Contract Address!")
}

fn read_neutron_mailbox_example_contract_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_PION_1_MAILBOX_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Pion 1 Mailbox Contract Address!")
}
