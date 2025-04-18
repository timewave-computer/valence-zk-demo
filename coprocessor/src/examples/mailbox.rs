use crate::{
    MAILBOX_APPLICATION_CIRCUIT_ELF,
    coprocessor::{Coprocessor, EthereumCoprocessor, NeutronCoprocessor},
    examples::prove_coprocessor,
    lightclients::{
        MockEthereumLightClientInterface, MockLightClient, MockNeutronLightClientInterface,
    },
};
use alloy::sol_types::SolValue;
use alloy_primitives::U256;
use dotenvy::dotenv;
use ethereum_merkle_proofs::merkle_lib::keccak::digest_keccak;
use ics23_merkle_proofs::keys::Ics23Key;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use sp1_verifier::Groth16Verifier;
use std::env;
use zk_mailbox_application_types::{
    MailboxApplicationCircuitInputs, MailboxApplicationCircuitOutputs,
};

pub async fn prove(mock_light_client: MockLightClient) {
    // required neutron storage key(s)
    let (neutron_root, neutron_height) =
        mock_light_client.get_latest_neutron_root_and_height().await;
    let neutron_mailbox_messages_key = Ics23Key::new_wasm_account_mapping(
        b"messages",
        "1",
        &read_neutron_mailbox_example_contract_address(),
    );

    // required ethereum storage key(s)
    let (ethereum_root, ethereum_height) = mock_light_client
        .get_latest_ethereum_root_and_height()
        .await;
    let slot: U256 = alloy_primitives::U256::from(0);
    let counter = U256::from(1);
    let encoded_key = (counter, slot).abi_encode();
    let ethereum_mailbox_messages_key = digest_keccak(&encoded_key).to_vec();

    let mut coprocessor = Coprocessor::from_env_with_storage_keys(
        vec![neutron_mailbox_messages_key],
        vec![(
            ethereum_mailbox_messages_key,
            read_ethereum_mailbox_example_contract_address(),
        )],
    );
    let merkle_proofs = coprocessor
        .get_storage_merkle_proofs(neutron_height, ethereum_height)
        .await;

    prove_coprocessor(
        &mut coprocessor,
        merkle_proofs.clone(),
        ethereum_root,
        neutron_root,
    )
    .await;

    // get the SMT openings that will be part of the input for our example application
    let ethereum_message_smt_opening = coprocessor
        .get_ethereum_opening(&borsh::to_vec(&merkle_proofs.1.first().unwrap().1).unwrap());
    let neutron_message_smt_opening =
        coprocessor.get_neutron_opening(&borsh::to_vec(&merkle_proofs.0.first().unwrap()).unwrap());
    // call the example application circuit with all the inputs
    let mailbox_application_circuit_inputs = MailboxApplicationCircuitInputs {
        neutron_messages_openings: vec![neutron_message_smt_opening],
        ethereum_messages_openings: vec![ethereum_message_smt_opening],
        coprocessor_root: coprocessor.smt_root,
    };
    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(
        borsh::to_vec(&mailbox_application_circuit_inputs)
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
