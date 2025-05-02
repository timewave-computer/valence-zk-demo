/// A collection of types and utilities for the ZK Mailbox application.
/// This module provides the core data structures and functions needed for cross-chain message verification
/// between Ethereum and Neutron chains using zero-knowledge proofs.
use beacon::types::electra::{ElectraBlockBodyRoots, ElectraBlockHeader};
use ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof;
use ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof;
use serde::{Deserialize, Serialize};
use valence_coprocessor_core::SmtOpening;
/// Inputs for the mailbox application circuit that contains all necessary merkle proofs
/// and SMT openings for verifying messages across different domains.
///
/// This struct encapsulates all the data needed to verify messages between Ethereum and Neutron chains,
/// including merkle proofs for both chains and their respective block headers.
#[derive(Serialize, Deserialize)]
pub struct MailboxApplicationCircuitInputs {
    /// Ethereum storage proofs for message verification, containing account proofs, storage proofs, and account addresses
    pub ethereum_storage_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)>,
    /// Neutron storage proofs for message verification
    pub neutron_storage_proofs: Vec<Ics23MerkleProof>,
    /// SMT opening containing the Neutron chain height
    pub neutron_height_opening: SmtOpening,
    /// SMT opening containing the Ethereum chain height
    pub ethereum_height_opening: SmtOpening,
    /// SMT opening containing the Neutron chain root
    pub neutron_root_opening: SmtOpening,
    /// SMT opening containing the Ethereum chain root
    pub ethereum_root_opening: SmtOpening,
    /// Tendermint block header from Neutron chain
    pub neutron_block_header: tendermint::block::Header,
    /// Electra block header from Ethereum chain
    pub electra_block_header: ElectraBlockHeader,
    /// Electra block body roots from Ethereum chain
    pub electra_body_roots: ElectraBlockBodyRoots,
    /// Root of the coprocessor SMT tree used for cross-chain verification
    pub coprocessor_root: [u8; 32],
}

/// Outputs from the mailbox application circuit containing the verified messages
///
/// This struct represents the result of the ZK circuit execution, containing all verified
/// messages from both chains and the final state of the coprocessor root.
#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct MailboxApplicationCircuitOutputs {
    /// The verified messages from the mailbox, extracted from both Ethereum and Neutron chains
    pub messages: Vec<String>,
    /// The final state of the coprocessor root after verification
    pub coprocessor_root: [u8; 32],
}

/// Deserializes an Ethereum merkle proof value into a string
///
/// # Arguments
/// * `data` - The raw bytes from the Ethereum merkle proof value
///
/// # Returns
/// The deserialized value as a string, with any control characters removed
///
/// # Errors
/// Returns an error if the RLP decoding fails
pub fn deserialize_ethereum_proof_value_as_string(data: Vec<u8>) -> String {
    decode_rlp_string_alloy(&data).unwrap()
}

/// Deserializes a Neutron merkle proof value into a string
///
/// # Arguments
/// * `data` - The raw bytes from the Neutron merkle proof value
///
/// # Returns
/// The deserialized value as a string, with any control characters removed
pub fn deserialize_neutron_proof_value_as_string(data: Vec<u8>) -> String {
    let raw_string = String::from_utf8_lossy(&data).to_string();
    // Clean the string like we do in the RLP decoder
    raw_string
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect::<String>()
        .trim_end_matches(char::is_control)
        .trim_matches('"')
        .to_string()
}

use alloy_rlp::Decodable;
/// Decodes an RLP-encoded string, stripping null bytes and non-printable characters
///
/// # Arguments
/// * `rlp_bytes` - The RLP-encoded bytes to decode
///
/// # Returns
/// A Result containing the decoded and cleaned string, or an error if decoding fails
fn decode_rlp_string_alloy(mut rlp_bytes: &[u8]) -> Result<String, String> {
    match String::decode(&mut rlp_bytes) {
        Ok(s) => {
            // Remove any trailing nulls or control characters
            let cleaned = s
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t') // keep visible chars
                .collect::<String>()
                .trim_end_matches(char::is_control) // just in case
                .to_string();

            Ok(cleaned)
        }
        Err(e) => Err(format!("RLP decode error: {:?}", e)),
    }
}
