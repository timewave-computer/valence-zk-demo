use crate::read_ethereum_consensus_rpc_url;
use crate::{
    MAILBOX_APPLICATION_CIRCUIT_ELF, coprocessor::Coprocessor, get_execution_block_height,
};
use alloy::sol_types::SolValue;
use alloy_primitives::U256;
use dotenvy::dotenv;
use ethereum_merkle_proofs::merkle_lib::keccak::digest_keccak;
use ics23_merkle_proofs::keys::Ics23Key;
use sp1_sdk::{ProverClient, SP1Stdin};
use ssz_merkleize::merkleize::get_state_root_at_slot;
use ssz_merkleize::types::BeaconBlockHeader;
use std::env;
use tendermint::block::Header;
use valence_coprocessor_core::SmtOpening;
use zk_mailbox_application_types::MailboxApplicationCircuitInputs;

pub async fn prove(
    neutron_height_opening: SmtOpening,
    ethereum_height_opening: SmtOpening,
    neutron_root_opening: SmtOpening,
    ethereum_root_opening: SmtOpening,
    neutron_block_header: Header,
    beacon_block_header: BeaconBlockHeader,
) {
    let neutron_mailbox_messages_key = Ics23Key::new_wasm_account_mapping(
        b"messages",
        "1",
        &read_neutron_mailbox_example_contract_address(),
    );
    let slot: U256 = alloy_primitives::U256::from(0);
    let counter = U256::from(1);
    let encoded_key = (counter, slot).abi_encode();
    let ethereum_mailbox_messages_key = digest_keccak(&encoded_key).to_vec();

    let mut coprocessor = Coprocessor::from_env();
    // todo: get the real ethereum height from the beacon block height
    let beacon_block_slot =
        u64::from_be_bytes(ethereum_height_opening.data.clone().try_into().unwrap());
    let ethereum_height =
        get_execution_block_height(&read_ethereum_consensus_rpc_url(), beacon_block_slot)
            .await
            .unwrap();
    println!("Ethereum Height: {}", ethereum_height);

    let domain_state_proofs = coprocessor
        .get_storage_merkle_proofs(
            u64::from_be_bytes(neutron_height_opening.data.clone().try_into().unwrap()),
            ethereum_height,
            vec![neutron_mailbox_messages_key],
            vec![(
                ethereum_mailbox_messages_key,
                read_ethereum_mailbox_example_contract_address(),
            )],
        )
        .await;

    let mock_block_header_verification_data = get_state_root_at_slot(beacon_block_slot).await;
    assert_eq!(
        mock_block_header_verification_data.1,
        hex::decode(beacon_block_header.body_root.trim_start_matches("0x"))
            .unwrap()
            .to_vec()
    );

    let mailbox_inputs = MailboxApplicationCircuitInputs {
        neutron_storage_proofs: domain_state_proofs.0,
        ethereum_storage_proofs: domain_state_proofs.1,
        neutron_height_opening,
        ethereum_height_opening,
        neutron_root_opening,
        ethereum_root_opening,
        neutron_block_header,
        beacon_block_header,
        coprocessor_root: coprocessor.smt_root,
        temporary_debug_state_root: mock_block_header_verification_data.0.try_into().unwrap(),
    };
    let prover = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();
    let (pk, _) = prover.setup(MAILBOX_APPLICATION_CIRCUIT_ELF);
    stdin.write_slice(&serde_json::to_vec(&mailbox_inputs).unwrap());
    let _proof = prover
        .prove(&pk, &stdin)
        .groth16()
        .run()
        .expect("Failed to prove Mailbox Application!");
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
