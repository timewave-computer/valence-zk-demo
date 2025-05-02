use crate::helpers::merkleize_container;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectraBlockBodyRoots {
    pub randao_reveal: [u8; 32],
    pub eth1_data: [u8; 32],
    pub graffiti: [u8; 32],
    pub proposer_slashings: [u8; 32],
    pub attester_slashings: [u8; 32],
    pub attestations: [u8; 32],
    pub deposits: [u8; 32],
    pub voluntary_exits: [u8; 32],
    pub sync_aggregate: [u8; 32],
    pub payload_roots: ElectraBlockBodyPayloadRoots,
    pub bls_to_execution_changes: [u8; 32],
    pub blob_kzg_commitments: [u8; 32],
    pub execution_requests: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectraBlockBodyPayloadRoots {
    pub parent_hash: [u8; 32],
    pub fee_recipient: [u8; 32],
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub logs_bloom: [u8; 32],
    pub prev_randao: [u8; 32],
    pub block_number: [u8; 32],
    pub gas_limit: [u8; 32],
    pub gas_used: [u8; 32],
    pub timestamp: [u8; 32],
    pub extra_data: [u8; 32],
    pub base_fee_per_gas: [u8; 32],
    pub block_hash: [u8; 32],
    pub transactions: [u8; 32],
    pub withdrawals: [u8; 32],
    pub blob_gas_used: [u8; 32],
    pub excess_blob_gas: [u8; 32],
}

impl ElectraBlockBodyRoots {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectraBlockHeader {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: [u8; 32],
    pub state_root: [u8; 32],
    pub body_root: [u8; 32],
}
