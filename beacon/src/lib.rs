use consensus_types::{BeaconBlockHeader, MainnetEthSpec, SignedBeaconBlock};
use itertools::Itertools;
use sha2::{Digest, Sha256};
use tree_hash::TreeHash;
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
    let summary: BeaconBlockHeader = serde_json::from_value(resp["data"]["header"]["message"].clone()).unwrap();
    summary
}


#[tokio::test]
async fn test_get_beacon_block_body() {
    let beacon_block_header = get_beacon_block_header(7520257).await;
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
    let block: SignedBeaconBlock<MainnetEthSpec> = serde_json::from_value(block_data).expect("Deserialization failed");
    let electra_block = block.as_electra().unwrap();
    let electra_block_body: consensus_types::BeaconBlockBodyElectra<MainnetEthSpec> = electra_block.message.body.clone();
    let electra_block_body_root = electra_block_body.tree_hash_root().to_vec();
    let execution_payload = electra_block_body.execution_payload.execution_payload.clone();
    let payload_field_roots: Vec<[u8; 32]> = vec![
        execution_payload.parent_hash.tree_hash_root().into(),
        execution_payload.fee_recipient.tree_hash_root().into(),
        execution_payload.state_root.tree_hash_root().into(),
        execution_payload.receipts_root.tree_hash_root().into(),
        execution_payload.logs_bloom.tree_hash_root().into(),
        execution_payload.prev_randao.tree_hash_root().into(),
        execution_payload.block_number.tree_hash_root().into(),
        execution_payload.gas_limit.tree_hash_root().into(),
        execution_payload.gas_used.tree_hash_root().into(),
        execution_payload.timestamp.tree_hash_root().into(),
        execution_payload.extra_data.tree_hash_root().into(),
        execution_payload.base_fee_per_gas.tree_hash_root().into(),
        execution_payload.block_hash.tree_hash_root().into(),
        execution_payload.transactions.tree_hash_root().into(),
        execution_payload.withdrawals.tree_hash_root().into(),
        execution_payload.blob_gas_used.tree_hash_root().into(),
        execution_payload.excess_blob_gas.tree_hash_root().into(),
    ];
    let payload_root_intermediate = merkleize_container(payload_field_roots);
    let field_roots: Vec<[u8; 32]> = vec![
        electra_block_body.randao_reveal.tree_hash_root().into(),
        electra_block_body.eth1_data.tree_hash_root().into(),
        electra_block_body.graffiti.tree_hash_root().into(),
        electra_block_body.proposer_slashings.tree_hash_root().into(),
        electra_block_body.attester_slashings.tree_hash_root().into(),
        electra_block_body.attestations.tree_hash_root().into(),
        electra_block_body.deposits.tree_hash_root().into(),
        electra_block_body.voluntary_exits.tree_hash_root().into(),
        electra_block_body.sync_aggregate.tree_hash_root().into(),
        payload_root_intermediate,
        electra_block_body.bls_to_execution_changes.tree_hash_root().into(),
        electra_block_body.blob_kzg_commitments.tree_hash_root().into(),
        electra_block_body.execution_requests.tree_hash_root().into(),
    ];
    let x = merkleize_container(field_roots);
    assert_eq!(electra_block_body_root, beacon_block_header.body_root.to_vec());
    assert_eq!(x.to_vec(), beacon_block_header.body_root.to_vec());
}

fn merkleize_container(field_roots: Vec<[u8; 32]>) -> [u8; 32] {
    // Pad to next power of two
    let count = field_roots.len();
    let next_pow2 = count.next_power_of_two();
    let mut leaves = field_roots;
    leaves.extend(vec![[0u8; 32]; next_pow2 - count]);
    // Climb tree level by level
    while leaves.len() > 1 {
        let mut next_level = vec![];
        for i in (0..leaves.len()).step_by(2) {
            let mut hasher = Sha256::new();
            hasher.update(&leaves[i]);
            hasher.update(&leaves[i + 1]);
            next_level.push(hasher.finalize().into());
        }
        leaves = next_level;
    }
    leaves[0]
}