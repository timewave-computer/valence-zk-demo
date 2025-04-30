// Ethereum SSZ merkleization helper functions

use itertools::Itertools;
use sha2::{Digest, Sha256};

use super::types::{BeaconBlockHeader, BeaconHeaderSummary};

#[cfg(feature = "reqwest")]
pub async fn get_state_root_at_slot(slot: u64) -> (Vec<u8>, Vec<u8>) {
    use ethereum_consensus::bellatrix::beacon_block::SignedBeaconBlock;
    use ethereum_consensus::ssz::prelude::*;
    // Lodestar Sepolia endpoint
    let endpoint = format!(
        "https://lodestar-sepolia.chainsafe.io/eth/v2/beacon/blocks/{}",
        slot
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

    // Deserialize {"data": {block}}
    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    let block_data = json["data"].clone();
    const MAX_PROPOSER_SLASHINGS: usize = 16;
    const MAX_VALIDATORS_PER_COMMITTEE: usize = 2048;
    const MAX_ATTESTER_SLASHINGS: usize = 2;
    const MAX_ATTESTATIONS: usize = 128;
    const MAX_DEPOSITS: usize = 16;
    const MAX_VOLUNTARY_EXITS: usize = 16;
    const SYNC_COMMITTEE_SIZE: usize = 512;
    const BYTES_PER_LOGS_BLOOM: usize = 256;
    const MAX_EXTRA_DATA_BYTES: usize = 32;
    const MAX_BYTES_PER_TRANSACTION: usize = 1073741824;
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize = 1048576;

    // Deserialize the block as SignedBeaconBlock (Bellatrix format)
    let block: SignedBeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    > = serde_json::from_value(block_data).expect("Deserialization failed");

    // Access the block body
    let body = &block.message.body;

    // Print execution state root
    let state_root: ByteVector<32> = body.execution_payload.state_root.clone();
    (
        state_root.to_vec(),
        body.hash_tree_root()
            .expect("Failed to compute SSZ root")
            .to_vec(),
    )
}

#[cfg(feature = "reqwest")]
pub async fn get_beacon_block_header(slot: u64) -> BeaconBlockHeader {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/eth/v1/beacon/headers/{}",
        "https://lodestar-sepolia.chainsafe.io", slot
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap() // fail if not 200 OK
        .json::<serde_json::Value>() // API returns wrapped {"data": {...}}
        .await
        .unwrap();

    // Unwrap the structure manually
    let summary: BeaconHeaderSummary = serde_json::from_value(resp["data"].clone()).unwrap();
    summary.header.message
}

// helper function to hash a bunch of nodes as ssz
// yes, Ethereum do be annoying sometimes :D
pub fn merkleize_keys(mut keys: Vec<Vec<u8>>) -> Vec<u8> {
    let height = if keys.len() == 1 {
        1
    } else {
        keys.len().next_power_of_two().ilog2() as usize
    };

    for depth in 0..height {
        let len_even: usize = keys.len() + keys.len() % 2;
        let padded_keys = keys
            .into_iter()
            .pad_using(len_even, |_| ZERO_HASHES[depth].as_slice().to_vec())
            .collect_vec();
        keys = padded_keys
            .into_iter()
            .tuples()
            .map(|(left, right)| compute_digest(&add_left_right(left, &right)))
            .collect::<Vec<Vec<u8>>>();
    }
    keys.pop().unwrap()
}

fn add_left_right(left: Vec<u8>, right: &[u8]) -> Vec<u8> {
    let mut value: Vec<u8> = left;
    value.extend_from_slice(right);
    value.to_vec()
}

fn compute_digest(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn uint64_to_le_256(value: u64) -> Vec<u8> {
    let mut bytes = value.to_le_bytes().to_vec(); // Convert to little-endian 8 bytes
    bytes.extend(vec![0u8; 24]); // Pad with 24 zeros to make it 32 bytes
    bytes
}

lazy_static::lazy_static! {
    pub static ref ZERO_HASHES: [[u8; 32]; 2] = {
        std::iter::successors(Some([0; 32]), |&prev| {
            Some(compute_digest(&[prev, prev].concat()).try_into().unwrap())
        })
        .take(2)
        .collect_vec()
        .try_into()
        .unwrap()
    };
}

#[cfg(feature = "reqwest")]
#[tokio::test]
async fn test_get_beacon_block_body() {
    use ethereum_consensus::bellatrix::beacon_block::SignedBeaconBlock;
    use ethereum_consensus::ssz::prelude::*;
    // Lodestar Sepolia endpoint
    let endpoint = "https://lodestar-sepolia.chainsafe.io/eth/v2/beacon/blocks/head";

    // Fetch the latest block
    let client = reqwest::Client::new();
    let resp = client
        .get(endpoint)
        .send()
        .await
        .expect("Request failed")
        .error_for_status()
        .expect("Non-200 response");

    // Deserialize {"data": {block}}
    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    let block_data = json["data"].clone();
    const MAX_PROPOSER_SLASHINGS: usize = 16;
    const MAX_VALIDATORS_PER_COMMITTEE: usize = 2048;
    const MAX_ATTESTER_SLASHINGS: usize = 2;
    const MAX_ATTESTATIONS: usize = 128;
    const MAX_DEPOSITS: usize = 16;
    const MAX_VOLUNTARY_EXITS: usize = 16;
    const SYNC_COMMITTEE_SIZE: usize = 512;
    const BYTES_PER_LOGS_BLOOM: usize = 256;
    const MAX_EXTRA_DATA_BYTES: usize = 32;
    const MAX_BYTES_PER_TRANSACTION: usize = 1073741824;
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize = 1048576;

    // Deserialize the block as SignedBeaconBlock (Bellatrix format)
    let block: SignedBeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    > = serde_json::from_value(block_data).expect("Deserialization failed");

    // Access the block body
    let body = &block.message.body;

    // Print execution state root
    let state_root: ByteVector<32> = body.execution_payload.state_root.clone();
    println!("Execution State Root: 0x{}", hex::encode(state_root));

    // Compute and print SSZ root of the block body
    let body_root = body.hash_tree_root().expect("Failed to compute SSZ root");
    println!("Block Body SSZ Root: 0x{}", hex::encode(body_root));
}
