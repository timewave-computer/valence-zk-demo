use std::fmt::Error;

use crate::ssz::prelude::*;

// Type aliases
pub type Root = Node;
pub type Bytes32 = ByteVector<32>;
pub type Slot = u64;
pub type ValidatorIndex = u64;
pub type Hash32 = Bytes32;
pub type ExecutionAddress = ByteVector<20>;
pub type Transaction<const MAX_BYTES_PER_TRANSACTION: usize> = ByteList<MAX_BYTES_PER_TRANSACTION>;
pub const BYTES_PER_COMMITMENT: usize = 48;
// Constants
const BLS_SIGNATURE_BYTES_LEN: usize = 96;
const BLS_PUBLIC_KEY_BYTES_LEN: usize = 48;
pub const DEPOSIT_CONTRACT_TREE_DEPTH: usize = 2usize.pow(5);
const DEPOSIT_PROOF_LENGTH: usize = get_deposit_contract_tree_depth();
pub type KzgCommitment = ByteVector<BYTES_PER_COMMITMENT>;

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct BeaconBlockHeader {
    #[serde(with = "crate::serde::as_str")]
    pub slot: Slot,
    #[serde(with = "crate::serde::as_str")]
    pub proposer_index: ValidatorIndex,
    pub parent_root: Root,
    pub state_root: Root,
    pub body_root: Root,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    pub signature: BlsSignature,
}

#[derive(
    Debug,
    Clone,
    Default,
    Hash,
    PartialEq,
    Eq,
    SimpleSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct BlsSignature(ByteVector<BLS_SIGNATURE_BYTES_LEN>);

const fn get_deposit_contract_tree_depth() -> usize {
    DEPOSIT_CONTRACT_TREE_DEPTH + 1
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct Eth1Data {
    pub deposit_root: Root,
    #[serde(with = "crate::serde::as_str")]
    pub deposit_count: u64,
    pub block_hash: Hash32,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct BeaconBlockBody<
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_VALIDATORS_PER_SLOT: usize,
    const MAX_COMMITTEES_PER_SLOT: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWALS_PER_PAYLOAD: usize,
    const MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize,
    const MAX_BLS_TO_EXECUTION_CHANGES: usize,
    const MAX_BLOB_COMMITMENTS_PER_BLOCK: usize,
    const MAX_CONSOLIDATIONS: usize,
> {
    pub randao_reveal: BlsSignature,
    pub eth1_data: Eth1Data,
    pub graffiti: Bytes32,
    pub proposer_slashings: List<ProposerSlashing, MAX_PROPOSER_SLASHINGS>,
    pub attester_slashings: List<AttesterSlashing<MAX_VALIDATORS_PER_SLOT>, MAX_ATTESTER_SLASHINGS>,
    pub attestations:
        List<Attestation<MAX_VALIDATORS_PER_SLOT, MAX_COMMITTEES_PER_SLOT>, MAX_ATTESTATIONS>,
    pub deposits: List<Deposit, MAX_DEPOSITS>,
    pub voluntary_exits: List<SignedVoluntaryExit, MAX_VOLUNTARY_EXITS>,
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    pub execution_payload: ExecutionPayload<
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
        MAX_WITHDRAWALS_PER_PAYLOAD,
        MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD,
        MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD,
    >,
    pub bls_to_execution_changes: List<SignedBlsToExecutionChange, MAX_BLS_TO_EXECUTION_CHANGES>,
    pub blob_kzg_commitments: List<KzgCommitment, MAX_BLOB_COMMITMENTS_PER_BLOCK>,
    #[serde(default)]
    pub consolidations: List<SignedConsolidation, MAX_CONSOLIDATIONS>,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct BeaconBlock<
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_VALIDATORS_PER_SLOT: usize,
    const MAX_COMMITTEES_PER_SLOT: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWALS_PER_PAYLOAD: usize,
    const MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize,
    const MAX_BLS_TO_EXECUTION_CHANGES: usize,
    const MAX_BLOB_COMMITMENTS_PER_BLOCK: usize,
    const MAX_CONSOLIDATIONS: usize,
> {
    #[serde(with = "crate::serde::as_str")]
    pub slot: Slot,
    #[serde(with = "crate::serde::as_str")]
    pub proposer_index: ValidatorIndex,
    pub parent_root: Root,
    pub state_root: Root,
    pub body: BeaconBlockBody<
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
    >,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SignedBeaconBlock<
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_VALIDATORS_PER_SLOT: usize,
    const MAX_COMMITTEES_PER_SLOT: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWALS_PER_PAYLOAD: usize,
    const MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize,
    const MAX_BLS_TO_EXECUTION_CHANGES: usize,
    const MAX_BLOB_COMMITMENTS_PER_BLOCK: usize,
    const MAX_CONSOLIDATIONS: usize,
> {
    pub message: BeaconBlock<
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
    >,
    pub signature: BlsSignature,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct AttesterSlashing<const MAX_VALIDATORS_PER_SLOT: usize> {
    pub attestation_1: IndexedAttestation<MAX_VALIDATORS_PER_SLOT>,
    pub attestation_2: IndexedAttestation<MAX_VALIDATORS_PER_SLOT>,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct IndexedAttestation<const MAX_VALIDATORS_PER_SLOT: usize> {
    #[serde(with = "crate::serde::seq_of_str")]
    pub attesting_indices: List<ValidatorIndex, MAX_VALIDATORS_PER_SLOT>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}

#[derive(
    Default, Clone, Debug, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct AttestationData {
    #[serde(with = "crate::serde::as_str")]
    pub slot: Slot,
    #[serde(with = "crate::serde::as_str")]
    pub index: usize,
    pub beacon_block_root: Root,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(
    Default, Clone, Debug, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct Checkpoint {
    #[serde(with = "crate::serde::as_str")]
    pub epoch: Epoch,
    pub root: Root,
}

pub type Epoch = u64;

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct Attestation<const MAX_VALIDATORS_PER_SLOT: usize, const MAX_COMMITTEES_PER_SLOT: usize> {
    pub aggregation_bits: Bitlist<MAX_VALIDATORS_PER_SLOT>,
    pub data: AttestationData,
    pub committee_bits: Bitvector<MAX_COMMITTEES_PER_SLOT>,
    pub signature: BlsSignature,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct Deposit {
    pub proof: Vector<Node, DEPOSIT_PROOF_LENGTH>,
    pub data: DepositData,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct DepositData {
    #[serde(rename = "pubkey")]
    pub public_key: BlsPublicKey,
    pub withdrawal_credentials: Bytes32,
    #[serde(with = "crate::serde::as_str")]
    pub amount: Gwei,
    pub signature: BlsSignature,
}

#[derive(
    Debug,
    Clone,
    Default,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    SimpleSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct BlsPublicKey(ByteVector<BLS_PUBLIC_KEY_BYTES_LEN>);

pub type Gwei = u64;

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: BlsSignature,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct VoluntaryExit {
    #[serde(with = "crate::serde::as_str")]
    pub epoch: Epoch,
    #[serde(with = "crate::serde::as_str")]
    pub validator_index: ValidatorIndex,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SyncAggregate<const SYNC_COMMITTEE_SIZE: usize> {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: BlsSignature,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct ExecutionPayload<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWALS_PER_PAYLOAD: usize,
    const MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD: usize,
    const MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize,
> {
    pub parent_hash: Hash32,
    pub fee_recipient: ExecutionAddress,
    pub state_root: Bytes32,
    pub receipts_root: Bytes32,
    pub logs_bloom: ByteVector<BYTES_PER_LOGS_BLOOM>,
    pub prev_randao: Bytes32,
    #[serde(with = "crate::serde::as_str")]
    pub block_number: u64,
    #[serde(with = "crate::serde::as_str")]
    pub gas_limit: u64,
    #[serde(with = "crate::serde::as_str")]
    pub gas_used: u64,
    #[serde(with = "crate::serde::as_str")]
    pub timestamp: u64,
    pub extra_data: ByteList<MAX_EXTRA_DATA_BYTES>,
    #[serde(with = "crate::serde::as_str")]
    pub base_fee_per_gas: U256,
    pub block_hash: Hash32,
    pub transactions: List<Transaction<MAX_BYTES_PER_TRANSACTION>, MAX_TRANSACTIONS_PER_PAYLOAD>,
    pub withdrawals: List<Withdrawal, MAX_WITHDRAWALS_PER_PAYLOAD>,
    #[serde(with = "crate::serde::as_str")]
    pub blob_gas_used: u64,
    #[serde(with = "crate::serde::as_str")]
    pub excess_blob_gas: u64,
    #[serde(default)]
    pub deposit_receipts: List<DepositReceipt, MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD>,
    #[serde(default)]
    pub withdrawal_requests:
        List<ExecutionLayerWithdrawalRequest, MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD>,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct Withdrawal {
    #[serde(with = "crate::serde::as_str")]
    pub index: WithdrawalIndex,
    #[serde(with = "crate::serde::as_str")]
    pub validator_index: ValidatorIndex,
    pub address: ExecutionAddress,
    #[serde(with = "crate::serde::as_str")]
    pub amount: Gwei,
}
pub type WithdrawalIndex = usize;

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct ExecutionLayerWithdrawalRequest {
    pub source_address: ExecutionAddress,
    #[serde(rename = "pubkey")]
    pub validator_public_key: BlsPublicKey,
    #[serde(with = "crate::serde::as_str")]
    pub amount: Gwei,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct DepositReceipt {
    #[serde(rename = "pubkey")]
    pub public_key: BlsPublicKey,
    pub withdrawal_credentials: Bytes32,
    #[serde(with = "crate::serde::as_str")]
    pub amount: Gwei,
    pub signature: BlsSignature,
    #[serde(with = "crate::serde::as_str")]
    pub index: u64,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct BlsToExecutionChange {
    #[serde(with = "crate::serde::as_str")]
    pub validator_index: ValidatorIndex,
    #[serde(rename = "from_bls_pubkey")]
    pub from_bls_public_key: BlsPublicKey,
    pub to_execution_address: ExecutionAddress,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SignedBlsToExecutionChange {
    pub message: BlsToExecutionChange,
    pub signature: BlsSignature,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SignedConsolidation {
    pub message: Consolidation,
    pub signature: BlsSignature,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct Consolidation {
    #[serde(with = "crate::serde::as_str")]
    pub source_index: ValidatorIndex,
    #[serde(with = "crate::serde::as_str")]
    pub target_index: ValidatorIndex,
    #[serde(with = "crate::serde::as_str")]
    pub epoch: Epoch,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct ExecutionPayloadHeader<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub parent_hash: Hash32,
    pub fee_recipient: ExecutionAddress,
    pub state_root: Bytes32,
    pub receipts_root: Bytes32,
    pub logs_bloom: ByteVector<BYTES_PER_LOGS_BLOOM>,
    pub prev_randao: Bytes32,
    #[serde(with = "crate::serde::as_str")]
    pub block_number: u64,
    #[serde(with = "crate::serde::as_str")]
    pub gas_limit: u64,
    #[serde(with = "crate::serde::as_str")]
    pub gas_used: u64,
    #[serde(with = "crate::serde::as_str")]
    pub timestamp: u64,
    pub extra_data: ByteList<MAX_EXTRA_DATA_BYTES>,
    #[serde(with = "crate::serde::as_str")]
    pub base_fee_per_gas: U256,
    pub block_hash: Hash32,
    pub transactions_root: Root,
    pub withdrawals_root: Root,
    #[serde(with = "crate::serde::as_str")]
    pub blob_gas_used: u64,
    #[serde(with = "crate::serde::as_str")]
    pub excess_blob_gas: u64,
    pub deposit_receipts_root: Root,
    pub withdrawal_requests_root: Root,
}

impl<
        'a,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const MAX_BYTES_PER_TRANSACTION: usize,
        const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
        const MAX_WITHDRAWALS_PER_PAYLOAD: usize,
        const MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD: usize,
        const MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize,
    >
    TryFrom<
        &'a ExecutionPayload<
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
            MAX_BYTES_PER_TRANSACTION,
            MAX_TRANSACTIONS_PER_PAYLOAD,
            MAX_WITHDRAWALS_PER_PAYLOAD,
            MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD,
            MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD,
        >,
    > for ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    type Error = Error;

    fn try_from(
        payload: &'a ExecutionPayload<
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
            MAX_BYTES_PER_TRANSACTION,
            MAX_TRANSACTIONS_PER_PAYLOAD,
            MAX_WITHDRAWALS_PER_PAYLOAD,
            MAX_DEPOSIT_RECEIPTS_PER_PAYLOAD,
            MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD,
        >,
    ) -> Result<ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>, Self::Error>
    {
        let transactions_root = payload.transactions.hash_tree_root().unwrap();
        let withdrawals_root = payload.withdrawals.hash_tree_root().unwrap();
        let deposit_receipts_root = payload.deposit_receipts.hash_tree_root().unwrap();
        let withdrawal_requests_root = payload.withdrawal_requests.hash_tree_root().unwrap();

        Ok(ExecutionPayloadHeader {
            parent_hash: payload.parent_hash.clone(),
            fee_recipient: payload.fee_recipient.clone(),
            state_root: payload.state_root.clone(),
            receipts_root: payload.receipts_root.clone(),
            logs_bloom: payload.logs_bloom.clone(),
            prev_randao: payload.prev_randao.clone(),
            block_number: payload.block_number,
            gas_limit: payload.gas_limit,
            gas_used: payload.gas_used,
            timestamp: payload.timestamp,
            extra_data: payload.extra_data.clone(),
            base_fee_per_gas: payload.base_fee_per_gas,
            block_hash: payload.block_hash.clone(),
            transactions_root,
            withdrawals_root,
            blob_gas_used: payload.blob_gas_used,
            excess_blob_gas: payload.excess_blob_gas,
            deposit_receipts_root,
            withdrawal_requests_root,
        })
    }
}
