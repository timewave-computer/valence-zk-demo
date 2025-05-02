use crate::helpers::merkleize_container;
use serde::{Deserialize, Serialize};

/// Represents the merkle roots of an Electra block body
///
/// This struct contains the merkle roots for all components of an Electra block body,
/// including the execution payload and various block body fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectraBlockBodyRoots {
    /// Root of the RANDAO reveal
    pub randao_reveal: [u8; 32],
    /// Root of the ETH1 data
    pub eth1_data: [u8; 32],
    /// Root of the graffiti field
    pub graffiti: [u8; 32],
    /// Root of the proposer slashings
    pub proposer_slashings: [u8; 32],
    /// Root of the attester slashings
    pub attester_slashings: [u8; 32],
    /// Root of the attestations
    pub attestations: [u8; 32],
    /// Root of the deposits
    pub deposits: [u8; 32],
    /// Root of the voluntary exits
    pub voluntary_exits: [u8; 32],
    /// Root of the sync aggregate
    pub sync_aggregate: [u8; 32],
    /// Roots of the execution payload fields
    pub payload_roots: ElectraBlockBodyPayloadRoots,
    /// Root of the BLS to execution changes
    pub bls_to_execution_changes: [u8; 32],
    /// Root of the blob KZG commitments
    pub blob_kzg_commitments: [u8; 32],
    /// Root of the execution requests
    pub execution_requests: [u8; 32],
}

/// Represents the merkle roots of an Electra block's execution payload
///
/// This struct contains the merkle roots for all fields in the execution payload
/// of an Electra block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectraBlockBodyPayloadRoots {
    /// Root of the parent hash
    pub parent_hash: [u8; 32],
    /// Root of the fee recipient
    pub fee_recipient: [u8; 32],
    /// Root of the state root
    pub state_root: [u8; 32],
    /// Root of the receipts root
    pub receipts_root: [u8; 32],
    /// Root of the logs bloom
    pub logs_bloom: [u8; 32],
    /// Root of the previous RANDAO
    pub prev_randao: [u8; 32],
    /// Root of the block number
    pub block_number: [u8; 32],
    /// Root of the gas limit
    pub gas_limit: [u8; 32],
    /// Root of the gas used
    pub gas_used: [u8; 32],
    /// Root of the timestamp
    pub timestamp: [u8; 32],
    /// Root of the extra data
    pub extra_data: [u8; 32],
    /// Root of the base fee per gas
    pub base_fee_per_gas: [u8; 32],
    /// Root of the block hash
    pub block_hash: [u8; 32],
    /// Root of the transactions
    pub transactions: [u8; 32],
    /// Root of the withdrawals
    pub withdrawals: [u8; 32],
    /// Root of the blob gas used
    pub blob_gas_used: [u8; 32],
    /// Root of the excess blob gas
    pub excess_blob_gas: [u8; 32],
}

impl ElectraBlockBodyRoots {
    /// Computes the merkle root of the entire block body
    ///
    /// This function combines all the field roots and the payload roots
    /// into a single merkle root.
    ///
    /// # Returns
    /// The 32-byte merkle root of the block body
    pub fn merkelize(&self) -> [u8; 32] {
        let payload_root = self.payload_roots.merkelize();
        merkleize_container(vec![
            self.randao_reveal,
            self.eth1_data,
            self.graffiti,
            self.proposer_slashings,
            self.attester_slashings,
            self.attestations,
            self.deposits,
            self.voluntary_exits,
            self.sync_aggregate,
            payload_root,
            self.bls_to_execution_changes,
            self.blob_kzg_commitments,
            self.execution_requests,
        ])
    }
}

impl ElectraBlockBodyPayloadRoots {
    /// Computes the merkle root of the execution payload
    ///
    /// This function combines all the execution payload field roots
    /// into a single merkle root.
    ///
    /// # Returns
    /// The 32-byte merkle root of the execution payload
    pub fn merkelize(&self) -> [u8; 32] {
        merkleize_container(vec![
            self.parent_hash,
            self.fee_recipient,
            self.state_root,
            self.receipts_root,
            self.logs_bloom,
            self.prev_randao,
            self.block_number,
            self.gas_limit,
            self.gas_used,
            self.timestamp,
            self.extra_data,
            self.base_fee_per_gas,
            self.block_hash,
            self.transactions,
            self.withdrawals,
            self.blob_gas_used,
            self.excess_blob_gas,
        ])
    }
}

/// Represents an Electra block header
///
/// This struct contains the essential fields of an Electra block header,
/// including the slot number, proposer index, and various merkle roots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectraBlockHeader {
    /// The slot number of the block
    pub slot: u64,
    /// The index of the block proposer
    pub proposer_index: u64,
    /// The merkle root of the parent block
    pub parent_root: [u8; 32],
    /// The merkle root of the state
    pub state_root: [u8; 32],
    /// The merkle root of the block body
    pub body_root: [u8; 32],
}
