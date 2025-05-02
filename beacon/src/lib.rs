#[cfg(feature = "no-zkvm")]
use consensus_types::{BeaconBlockHeader, SignedBeaconBlock};
#[cfg(feature = "no-zkvm")]
use consensus_types::{MainnetEthSpec, SignedBeaconBlockElectra};
use helpers::merkleize_container;
use itertools::Itertools;
#[cfg(feature = "no-zkvm")]
use tree_hash::TreeHash;
use types::electra::ElectraBlockHeader;
#[cfg(feature = "no-zkvm")]
use types::electra::{ElectraBlockBodyPayloadRoots, ElectraBlockBodyRoots};
pub mod helpers;
pub mod types;

/// Computes the merkle root of an Electra block header
///
/// This function takes an Electra block header and computes its merkle root by:
/// 1. Padding the slot and proposer index to 32 bytes
/// 2. Combining them with the parent root, state root, and body root
/// 3. Computing the merkle root of the resulting container
///
/// # Arguments
/// * `header` - The Electra block header to merkleize
///
/// # Returns
/// The 32-byte merkle root of the header
pub fn merkleize_header(header: ElectraBlockHeader) -> [u8; 32] {
    let slot_padded: Vec<u8> = header
        .slot
        .to_le_bytes()
        .iter()
        .copied() // Convert from &u8 to u8
        .pad_using(32, |_| 0u8) // Pad with zeros to 32 bytes
        .collect();

    let proposer_index_padded: Vec<u8> = header
        .proposer_index
        .to_le_bytes()
        .iter()
        .copied()
        .pad_using(32, |_| 0u8)
        .collect();
    merkleize_container(vec![
        slot_padded.try_into().unwrap(),
        proposer_index_padded.try_into().unwrap(),
        header.parent_root,
        header.state_root,
        header.body_root,
    ])
}

#[cfg(feature = "no-zkvm")]
/// Fetches a beacon block header from a specified Ethereum beacon node
///
/// # Arguments
/// * `slot` - The slot number of the block header to fetch
/// * `url` - The URL of the beacon node to query
///
/// # Returns
/// The requested beacon block header
///
/// # Errors
/// Returns an error if the request fails or the response cannot be parsed
pub async fn get_beacon_block_header(slot: u64, url: &str) -> BeaconBlockHeader {
    let client = reqwest::Client::new();
    let url = format!("{}/eth/v1/beacon/headers/{}", url, slot);
    let resp = client
        .get(&url)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let summary: BeaconBlockHeader =
        serde_json::from_value(resp["data"]["header"]["message"].clone()).unwrap();
    summary
}

#[cfg(feature = "no-zkvm")]
/// Fetches an Electra block from a specified Ethereum beacon node
///
/// # Arguments
/// * `slot` - The slot number of the block to fetch
/// * `url` - The URL of the beacon node to query
///
/// # Returns
/// The requested Electra block
///
/// # Errors
/// Returns an error if the request fails or the response cannot be parsed
pub async fn get_electra_block(slot: u64, url: &str) -> SignedBeaconBlockElectra<MainnetEthSpec> {
    let endpoint = format!("{}/eth/v2/beacon/blocks/{}", url, slot);
    let client = reqwest::Client::new();
    let resp = client
        .get(endpoint)
        .send()
        .await
        .expect("Request failed")
        .error_for_status()
        .expect("Non-200 response");

    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    let block_data = json["data"].clone();
    let block: SignedBeaconBlock<MainnetEthSpec> =
        serde_json::from_value(block_data).expect("Deserialization failed");
    let electra_block = block.as_electra().unwrap();
    electra_block.clone()
}

#[cfg(feature = "no-zkvm")]
/// Extracts and computes the merkle roots of an Electra block body
///
/// This function takes an Electra block and computes the merkle roots for all its components,
/// including the execution payload and various block body fields.
///
/// # Arguments
/// * `electra_block` - The Electra block to process
///
/// # Returns
/// A struct containing all the computed merkle roots for the block body
pub fn extract_electra_block_body(
    electra_block: SignedBeaconBlockElectra<MainnetEthSpec>,
) -> ElectraBlockBodyRoots {
    let electra_block_body = electra_block.message.body;
    let execution_payload = electra_block_body
        .execution_payload
        .execution_payload
        .clone();

    let payload_roots = ElectraBlockBodyPayloadRoots {
        parent_hash: execution_payload.parent_hash.tree_hash_root().into(),
        fee_recipient: execution_payload.fee_recipient.tree_hash_root().into(),
        // raw state root
        state_root: execution_payload.state_root.into(),
        // raw receipts root
        receipts_root: execution_payload.receipts_root.into(),
        logs_bloom: execution_payload.logs_bloom.tree_hash_root().into(),
        prev_randao: execution_payload.prev_randao.tree_hash_root().into(),
        block_number: execution_payload.block_number.tree_hash_root().into(),
        gas_limit: execution_payload.gas_limit.tree_hash_root().into(),
        gas_used: execution_payload.gas_used.tree_hash_root().into(),
        timestamp: execution_payload.timestamp.tree_hash_root().into(),
        extra_data: execution_payload.extra_data.tree_hash_root().into(),
        base_fee_per_gas: execution_payload.base_fee_per_gas.tree_hash_root().into(),
        block_hash: execution_payload.block_hash.tree_hash_root().into(),
        transactions: execution_payload.transactions.tree_hash_root().into(),
        withdrawals: execution_payload.withdrawals.tree_hash_root().into(),
        blob_gas_used: execution_payload.blob_gas_used.tree_hash_root().into(),
        excess_blob_gas: execution_payload.excess_blob_gas.tree_hash_root().into(),
    };
    ElectraBlockBodyRoots {
        randao_reveal: electra_block_body.randao_reveal.tree_hash_root().into(),
        eth1_data: electra_block_body.eth1_data.tree_hash_root().into(),
        graffiti: electra_block_body.graffiti.tree_hash_root().into(),
        proposer_slashings: electra_block_body
            .proposer_slashings
            .tree_hash_root()
            .into(),
        attester_slashings: electra_block_body
            .attester_slashings
            .tree_hash_root()
            .into(),
        attestations: electra_block_body.attestations.tree_hash_root().into(),
        deposits: electra_block_body.deposits.tree_hash_root().into(),
        voluntary_exits: electra_block_body.voluntary_exits.tree_hash_root().into(),
        sync_aggregate: electra_block_body.sync_aggregate.tree_hash_root().into(),
        payload_roots,
        bls_to_execution_changes: electra_block_body
            .bls_to_execution_changes
            .tree_hash_root()
            .into(),
        blob_kzg_commitments: electra_block_body
            .blob_kzg_commitments
            .tree_hash_root()
            .into(),
        execution_requests: electra_block_body
            .execution_requests
            .tree_hash_root()
            .into(),
    }
}

#[cfg(feature = "no-zkvm")]
#[tokio::test]
/// Tests the functionality of fetching and processing beacon block bodies
///
/// This test verifies that:
/// 1. We can fetch a beacon block header
/// 2. We can fetch and process the corresponding block
/// 3. The computed merkle roots match the expected values
async fn test_get_beacon_block_body() {
    let beacon_block_header =
        get_beacon_block_header(7520257, "https://lodestar-sepolia.chainsafe.io").await;
    // Lodestar Sepolia endpoint
    let endpoint = format!(
        "https://lodestar-sepolia.chainsafe.io/eth/v2/beacon/blocks/{}",
        7520257
    );
    // Fetch the latest block
    let client = reqwest::Client::new();
    let resp = client
        .get(endpoint)
        .send()
        .await
        .expect("Request failed")
        .error_for_status()
        .expect("Non-200 response");

    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    let block_data = json["data"].clone();
    let block: SignedBeaconBlock<MainnetEthSpec> =
        serde_json::from_value(block_data).expect("Deserialization failed");
    let electra_block = block.as_electra().unwrap();
    let electra_block_body = extract_electra_block_body(electra_block.clone());
    let electra_block_body_root = electra_block_body.merkelize();

    assert_eq!(
        electra_block_body_root.to_vec(),
        beacon_block_header.body_root.to_vec()
    );
    let electra_block_header = ElectraBlockHeader {
        slot: beacon_block_header.slot.as_u64(),
        proposer_index: beacon_block_header.proposer_index,
        parent_root: beacon_block_header.parent_root.into(),
        state_root: beacon_block_header.state_root.into(),
        body_root: beacon_block_header.body_root.into(),
    };
    let beacon_block_header_root = merkleize_header(electra_block_header);
    assert_eq!(
        beacon_block_header_root.to_vec(),
        beacon_block_header.tree_hash_root().to_vec()
    );
}
