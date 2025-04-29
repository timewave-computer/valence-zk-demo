// Ethereum SSZ merkleization helper functions

use sha2::{Digest, Sha256};
use itertools::Itertools;

use super::types::{BeaconBlockHeader, BeaconHeaderSummary};

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