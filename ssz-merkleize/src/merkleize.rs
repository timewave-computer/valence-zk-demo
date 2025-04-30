// Ethereum SSZ merkleization helper functions

use itertools::Itertools;
use sha2::{Digest, Sha256};

use super::types::{BeaconBlockHeader, BeaconHeaderSummary};

/*

boss:~ chef$ curl https://lodestar-sepolia.chainsafe.io/eth/v1/config/spec
{"data":{"PRESET_BASE":"mainnet","CONFIG_NAME":"sepolia","TERMINAL_TOTAL_DIFFICULTY":"17000000000000000",
"TERMINAL_BLOCK_HASH":"0x0000000000000000000000000000000000000000000000000000000000000000","TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH":"18446744073709551615",
"MIN_GENESIS_ACTIVE_VALIDATOR_COUNT":"1300","MIN_GENESIS_TIME":"1655647200","GENESIS_FORK_VERSION":"0x90000069","GENESIS_DELAY":"86400","ALTAIR_FORK_VERSION":"0x90000070",
"ALTAIR_FORK_EPOCH":"50","BELLATRIX_FORK_VERSION":"0x90000071","BELLATRIX_FORK_EPOCH":"100","CAPELLA_FORK_VERSION":"0x90000072","CAPELLA_FORK_EPOCH":"56832",
"DENEB_FORK_VERSION":"0x90000073","DENEB_FORK_EPOCH":"132608","ELECTRA_FORK_VERSION":"0x90000074","ELECTRA_FORK_EPOCH":"222464","FULU_FORK_VERSION":"0x90000075",
"FULU_FORK_EPOCH":"18446744073709551615","SECONDS_PER_SLOT":"12","SECONDS_PER_ETH1_BLOCK":"14","MIN_VALIDATOR_WITHDRAWABILITY_DELAY":"256","SHARD_COMMITTEE_PERIOD":"256",
"ETH1_FOLLOW_DISTANCE":"2048","INACTIVITY_SCORE_BIAS":"4","INACTIVITY_SCORE_RECOVERY_RATE":"16","EJECTION_BALANCE":"16000000000","MIN_PER_EPOCH_CHURN_LIMIT":"4",
"MAX_PER_EPOCH_ACTIVATION_CHURN_LIMIT":"8","CHURN_LIMIT_QUOTIENT":"65536","MAX_PER_EPOCH_ACTIVATION_EXIT_CHURN_LIMIT":"256000000000","MIN_PER_EPOCH_CHURN_LIMIT_ELECTRA":
"128000000000","PROPOSER_SCORE_BOOST":"40","REORG_HEAD_WEIGHT_THRESHOLD":"20","REORG_PARENT_WEIGHT_THRESHOLD":"160","REORG_MAX_EPOCHS_SINCE_FINALIZATION":"2",
"DEPOSIT_CHAIN_ID":"11155111","DEPOSIT_NETWORK_ID":"11155111","DEPOSIT_CONTRACT_ADDRESS":"0x7f02c3e3c98b133055b8b348b2ac625669ed295d","MIN_EPOCHS_FOR_BLOCK_REQUESTS":"33024",
"MIN_EPOCHS_FOR_BLOB_SIDECARS_REQUESTS":"4096","BLOB_SIDECAR_SUBNET_COUNT":"6","MAX_BLOBS_PER_BLOCK":"6","MAX_REQUEST_BLOB_SIDECARS":"768","BLOB_SIDECAR_SUBNET_COUNT_ELECTRA":"9",
"MAX_BLOBS_PER_BLOCK_ELECTRA":"9","MAX_REQUEST_BLOB_SIDECARS_ELECTRA":"1152","MAX_COMMITTEES_PER_SLOT":"64","TARGET_COMMITTEE_SIZE":"128",
"MAX_VALIDATORS_PER_COMMITTEE":"2048","SHUFFLE_ROUND_COUNT":"90","HYSTERESIS_QUOTIENT":"4","HYSTERESIS_DOWNWARD_MULTIPLIER":"1","HYSTERESIS_UPWARD_MULTIPLIER":"5",
"MIN_DEPOSIT_AMOUNT":"1000000000","MAX_EFFECTIVE_BALANCE":"32000000000","EFFECTIVE_BALANCE_INCREMENT":"1000000000","MIN_ATTESTATION_INCLUSION_DELAY":"1",
"SLOTS_PER_EPOCH":"32","MIN_SEED_LOOKAHEAD":"1","MAX_SEED_LOOKAHEAD":"4","EPOCHS_PER_ETH1_VOTING_PERIOD":"64","SLOTS_PER_HISTORICAL_ROOT":"8192","
MIN_EPOCHS_TO_INACTIVITY_PENALTY":"4","EPOCHS_PER_HISTORICAL_VECTOR":"65536","EPOCHS_PER_SLASHINGS_VECTOR":"8192","HISTORICAL_ROOTS_LIMIT":"16777216",
"VALIDATOR_REGISTRY_LIMIT":"1099511627776","BASE_REWARD_FACTOR":"64","WHISTLEBLOWER_REWARD_QUOTIENT":"512","PROPOSER_REWARD_QUOTIENT":"8",
"INACTIVITY_PENALTY_QUOTIENT":"67108864","MIN_SLASHING_PENALTY_QUOTIENT":"128","PROPORTIONAL_SLASHING_MULTIPLIER":"1","MAX_PROPOSER_SLASHINGS":"16",
"MAX_ATTESTER_SLASHINGS":"2","MAX_ATTESTATIONS":"128","MAX_DEPOSITS":"16","MAX_VOLUNTARY_EXITS":"16","SYNC_COMMITTEE_SIZE":"512","EPOCHS_PER_SYNC_COMMITTEE_PERIOD":"256"
,"INACTIVITY_PENALTY_QUOTIENT_ALTAIR":"50331648","MIN_SLASHING_PENALTY_QUOTIENT_ALTAIR":"64","PROPORTIONAL_SLASHING_MULTIPLIER_ALTAIR":"2","MIN_SYNC_COMMITTEE_PARTICIPANTS":"1",
"UPDATE_TIMEOUT":"8192","INACTIVITY_PENALTY_QUOTIENT_BELLATRIX":"16777216","MIN_SLASHING_PENALTY_QUOTIENT_BELLATRIX":"32","PROPORTIONAL_SLASHING_MULTIPLIER_BELLATRIX":"3",
"MAX_BYTES_PER_TRANSACTION":"1073741824","MAX_TRANSACTIONS_PER_PAYLOAD":"1048576","BYTES_PER_LOGS_BLOOM":"256","MAX_EXTRA_DATA_BYTES":"32","MAX_BLS_TO_EXECUTION_CHANGES":"16",
"MAX_WITHDRAWALS_PER_PAYLOAD":"16","MAX_VALIDATORS_PER_WITHDRAWALS_SWEEP":"16384","FIELD_ELEMENTS_PER_BLOB":"4096","MAX_BLOB_COMMITMENTS_PER_BLOCK":"4096",
"KZG_COMMITMENT_INCLUSION_PROOF_DEPTH":"17","MAX_DEPOSIT_REQUESTS_PER_PAYLOAD":"8192","MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD":"16","MAX_ATTESTER_SLASHINGS_ELECTRA":"1",
"MAX_ATTESTATIONS_ELECTRA":"8","MAX_PENDING_PARTIALS_PER_WITHDRAWALS_SWEEP":"8","MAX_PENDING_DEPOSITS_PER_EPOCH":"16","MAX_EFFECTIVE_BALANCE_ELECTRA":"2048000000000",
"MIN_SLASHING_PENALTY_QUOTIENT_ELECTRA":"4096","MIN_ACTIVATION_BALANCE":"32000000000","PENDING_DEPOSITS_LIMIT":"134217728","PENDING_PARTIAL_WITHDRAWALS_LIMIT":"134217728"
"PENDING_CONSOLIDATIONS_LIMIT":"262144","MAX_CONSOLIDATION_REQUESTS_PER_PAYLOAD":"2","WHISTLEBLOWER_REWARD_QUOTIENT_ELECTRA":"4096","GENESIS_SLOT":"0","GENESIS_EPOCH":"0",
"FAR_FUTURE_EPOCH":"18446744073709551615","BASE_REWARDS_PER_EPOCH":"4","DEPOSIT_CONTRACT_TREE_DEPTH":"32","JUSTIFICATION_BITS_LENGTH":"4","ENDIANNESS":"little","
BLS_WITHDRAWAL_PREFIX":"0","ETH1_ADDRESS_WITHDRAWAL_PREFIX":"1","COMPOUNDING_WITHDRAWAL_PREFIX":"2","DOMAIN_BEACON_PROPOSER":"0x00000000","DOMAIN_BEACON_ATTESTER":"0x01000000"
,"DOMAIN_RANDAO":"0x02000000","DOMAIN_DEPOSIT":"0x03000000","DOMAIN_VOLUNTARY_EXIT":"0x04000000","DOMAIN_SELECTION_PROOF":"0x05000000","DOMAIN_AGGREGATE_AND_PROOF":"0x06000000",
"DOMAIN_APPLICATION_MASK":"0x00000001","DOMAIN_APPLICATION_BUILDER":"0x00000001","TARGET_AGGREGATORS_PER_COMMITTEE":"16","RANDOM_SUBNETS_PER_VALIDATOR":"1",
"EPOCHS_PER_RANDOM_SUBNET_SUBSCRIPTION":"256","ATTESTATION_SUBNET_COUNT":"64","TIMELY_SOURCE_FLAG_INDEX":"0","TIMELY_TARGET_FLAG_INDEX":"1","TIMELY_HEAD_FLAG_INDEX":"2",
"TIMELY_SOURCE_WEIGHT":"14","TIMELY_TARGET_WEIGHT":"26","TIMELY_HEAD_WEIGHT":"14","SYNC_REWARD_WEIGHT":"2","PROPOSER_WEIGHT":"8","WEIGHT_DENOMINATOR":"64"
,"DOMAIN_SYNC_COMMITTEE":"0x07000000","DOMAIN_SYNC_COMMITTEE_SELECTION_PROOF":"0x08000000","DOMAIN_CONTRIBUTION_AND_PROOF":"0x09000000",
"TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE":"16","SYNC_COMMITTEE_SUBNET_COUNT":"4","DOMAIN_BLS_TO_EXECUTION_CHANGE":"0x0a000000",
"BLOB_TX_TYPE":"3","VERSIONED_HASH_VERSION_KZG":"1","UNSET_DEPOSIT_REQUESTS_START_INDEX":"18446744073709551615","FULL_EXIT_REQUEST_AMOUNT":"0",
"DEPOSIT_REQUEST_TYPE":"0","WITHDRAWAL_REQUEST_TYPE":"1","CONSOLIDATION_REQUEST_TYPE":"2"}}boss:~ chef$

*/

pub const MAX_BYTES_PER_TRANSACTION: usize = 1_073_741_824; // 1 GiB
pub const MAX_TRANSACTIONS_PER_PAYLOAD: usize = 1_048_576;
pub const MAX_PROPOSER_SLASHINGS: usize = 16;
pub const MAX_VALIDATORS_PER_COMMITTEE: usize = 2048;
pub const MAX_ATTESTER_SLASHINGS: usize = 1; // 2 non electra
pub const MAX_ATTESTATIONS: usize = 8; // 128 non electra
pub const MAX_DEPOSITS: usize = 16;
pub const MAX_VOLUNTARY_EXITS: usize = 16;
pub const SYNC_COMMITTEE_SIZE: usize = 512;
pub const BYTES_PER_LOGS_BLOOM: usize = 256;
pub const MAX_EXTRA_DATA_BYTES: usize = 32;
pub const MAX_WITHDRAWALS_PER_PAYLOAD: usize = 16;
pub const MAX_BLS_TO_EXECUTION_CHANGES: usize = 16;
pub const MAX_VALIDATORS_PER_WITHDRAWALS_SWEEP: usize = 16384;
pub const MAX_BLOB_COMMITMENTS_PER_BLOCK: usize = 4096;
pub const MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize = 16;
pub const MAX_COMMITTEES_PER_SLOT: usize = 64;
pub const MAX_VALIDATORS_PER_SLOT: usize = 131072;
pub const MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD: usize = 8192;
pub const MAX_CONSOLIDATIONS: usize = 2;

#[cfg(feature = "reqwest")]
pub async fn get_state_root_at_slot(slot: u64) -> (Vec<u8>, Vec<u8>) {
    use ethereum_consensus::electra::beacon_block::SignedBeaconBlock;
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
    // Deserialize the block as SignedBeaconBlock (Bellatrix format)
    let block: SignedBeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_SLOT,
        MAX_COMMITTEES_PER_SLOT,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
        MAX_WITHDRAWALS_PER_PAYLOAD,
        MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD,
        MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD,
        MAX_BLS_TO_EXECUTION_CHANGES,
        MAX_BLOB_COMMITMENTS_PER_BLOCK,
        MAX_CONSOLIDATIONS,
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
    use ethereum_consensus::electra::beacon_block::SignedBeaconBlock;
    use ethereum_consensus::ssz::prelude::*;
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

    // Deserialize {"data": {block}}
    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    let block_data = json["data"].clone();
    println!("block_data: {:?}", block_data);
    // Deserialize the block as SignedBeaconBlock (Bellatrix format)
    let block: SignedBeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_SLOT,
        MAX_COMMITTEES_PER_SLOT,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
        MAX_WITHDRAWALS_PER_PAYLOAD,
        MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD,
        MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD,
        MAX_BLS_TO_EXECUTION_CHANGES,
        MAX_BLOB_COMMITMENTS_PER_BLOCK,
        MAX_CONSOLIDATIONS,
    > = serde_json::from_value(block_data).expect("Deserialization failed");
    let body = &block.message.body;
    // Compute and print SSZ root of the block body
    let body_root = body.hash_tree_root().expect("Failed to compute SSZ root");
    let beacon_block_header = get_beacon_block_header(7520257).await;
    assert_eq!(
        hex::decode(beacon_block_header.body_root.trim_start_matches("0x"))
            .unwrap()
            .to_vec(),
        body_root.to_vec()
    );
}
