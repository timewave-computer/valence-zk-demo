use crate::read_ethereum_consensus_rpc_url;
use crate::{
    MAILBOX_APPLICATION_CIRCUIT_ELF, coprocessor::Coprocessor, get_execution_block_height,
};
use alloy::sol_types::SolValue;
use alloy_primitives::U256;
use beacon::types::electra::ElectraBlockHeader;
use beacon::{extract_electra_block_body, get_beacon_block_header, get_electra_block};
use dotenvy::dotenv;
use ethereum_merkle_proofs::merkle_lib::keccak::digest_keccak;
use ics23_merkle_proofs::keys::Ics23Key;
use sp1_sdk::{ProverClient, SP1Stdin};
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
) {
    // we want to prove the Neutron mailbox message at key 1 e.g. the first message that is "Hello Ethereum!"
    // when proving a value in ZK, the app developer should be confident that it exists on the target domain
    // If it doesn't exist, then the prover will fail
    let neutron_mailbox_messages_key = Ics23Key::new_wasm_account_mapping(
        b"messages",
        "1",
        &read_neutron_mailbox_example_contract_address(),
    );
    let slot: U256 = alloy_primitives::U256::from(0);
    let counter = U256::from(1);
    let encoded_key = (counter, slot).abi_encode();
    // we want to prove the Ethereum mailbox message at key 1 e.g. the first message that is "Hello Neutron!"
    let ethereum_mailbox_messages_key = digest_keccak(&encoded_key).to_vec();
    let mut coprocessor = Coprocessor::from_env();
    // todo: get the real ethereum height from the beacon block height
    let beacon_block_slot =
        u64::from_be_bytes(ethereum_height_opening.data.clone().try_into().unwrap());
    // Get the Ethereum execution layer block height for the beacon consensus slot
    let ethereum_height =
        get_execution_block_height(&read_ethereum_consensus_rpc_url(), beacon_block_slot)
            .await
            .unwrap();
    let neutron_target_height =
        u64::from_be_bytes(neutron_height_opening.data.clone().try_into().unwrap());
    // Get the Merkle proofs for the Neutron and Ethereum mailbox keys that we constructed above
    let domain_state_proofs = coprocessor
        .get_storage_merkle_proofs(
            // the app hash of the current block attests to the state of the previous block
            neutron_target_height - 1,
            ethereum_height,
            vec![neutron_mailbox_messages_key],
            vec![(
                ethereum_mailbox_messages_key,
                read_ethereum_mailbox_example_contract_address(),
            )],
        )
        .await;
    let ethereum_slot =
        u64::from_be_bytes(ethereum_height_opening.data.clone().try_into().unwrap());
    // Get the Electra signed block object from the RPC
    let electra_block = get_electra_block(ethereum_slot, &read_ethereum_consensus_rpc_url()).await;
    // Extract the body roots from the Electra block
    let electra_body_roots = extract_electra_block_body(electra_block);
    // Get the Electra block header from the RPC
    let electra_block_header =
        get_beacon_block_header(ethereum_slot, &read_ethereum_consensus_rpc_url()).await;
    // Construct the Zk-friendly Electra block header object
    let electra_block_header = ElectraBlockHeader {
        slot: electra_block_header.slot.as_u64(),
        proposer_index: electra_block_header.proposer_index,
        parent_root: electra_block_header.parent_root.into(),
        state_root: electra_block_header.state_root.into(),
        body_root: electra_block_header.body_root.into(),
    };

    // Construct the Zk-friendly Mailbox Application Circuit inputs
    let mailbox_inputs = MailboxApplicationCircuitInputs {
        neutron_storage_proofs: domain_state_proofs.0,
        ethereum_storage_proofs: domain_state_proofs.1,
        neutron_height_opening,
        ethereum_height_opening,
        neutron_root_opening,
        ethereum_root_opening,
        neutron_block_header,
        electra_block_header,
        electra_body_roots,
        coprocessor_root: coprocessor.smt_root,
    };
    // Run the Prover for the Application Circuit
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
