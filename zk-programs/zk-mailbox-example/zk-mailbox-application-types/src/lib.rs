use ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof;
use ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof;
use valence_coprocessor_core::SmtOpening;
use serde::{Serialize, Deserialize};
/// Inputs for the rate application circuit that contains all necessary merkle proofs
/// and SMT openings for verifying vault balances and shares across different domains.
#[derive(Serialize, Deserialize)]
pub struct MailboxApplicationCircuitInputs {
    /// SMT opening for the Neutron vault balance proof
    pub ethereum_storage_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)>,
    /// SMT opening for the Neutron vault shares proof
    pub neutron_storage_proofs: Vec<Ics23MerkleProof>,
    pub neutron_height_opening: SmtOpening,
    pub ethereum_height_opening: SmtOpening,
    pub neutron_root_opening: SmtOpening,
    pub ethereum_root_opening: SmtOpening,
    pub neutron_block_header: tendermint::block::Header,
    pub beacon_block_header: ssz_merkleize::types::BeaconBlockHeader,
    /// Root of the coprocessor SMT tree
    pub coprocessor_root: [u8; 32],
}

/// Outputs from the rate application circuit containing the calculated rate
#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct MailboxApplicationCircuitOutputs {
    /// The calculated rate based on total balances and shares across domains
    pub messages: Vec<String>,
}

/// Deserializes an Ethereum merkle proof value into a U256 number
///
/// # Arguments
/// * `proof` - The SMT opening containing the Ethereum merkle proof
///
/// # Returns
/// The deserialized value as a U256 number
pub fn deserialize_ethereum_proof_value_as_string(data: Vec<u8>) -> String {
    decode_rlp_string_alloy(&data).unwrap()
}

/// Deserializes a Neutron merkle proof value into a U256 number
///
/// # Arguments
/// * `proof` - The SMT opening containing the Neutron merkle proof
///
/// # Returns
/// The deserialized value as a U256 number
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
