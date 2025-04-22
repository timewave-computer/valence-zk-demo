use borsh::{BorshDeserialize, BorshSerialize};
use ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof;
use ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof;
use valence_smt::SmtOpening;

/// Inputs for the rate application circuit that contains all necessary merkle proofs
/// and SMT openings for verifying vault balances and shares across different domains.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct MailboxApplicationCircuitInputs {
    /// SMT opening for the Neutron vault balance proof
    pub neutron_message_openings: Vec<SmtOpening>,
    /// SMT opening for the Neutron vault shares proof
    pub ethereum_message_openings: Vec<SmtOpening>,
    /// Root of the coprocessor SMT tree
    pub coprocessor_root: [u8; 32],
}

/// Outputs from the rate application circuit containing the calculated rate
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
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
pub fn deserialize_ethereum_proof_value_as_string(proof: SmtOpening) -> String {
    let ethereum_proof: EthereumMerkleProof = borsh::from_slice(&proof.data).unwrap();
    decode_rlp_string_alloy(&ethereum_proof.value).unwrap()
}

/// Deserializes a Neutron merkle proof value into a U256 number
///
/// # Arguments
/// * `proof` - The SMT opening containing the Neutron merkle proof
///
/// # Returns
/// The deserialized value as a U256 number
pub fn deserialize_neutron_proof_value_as_string(proof: SmtOpening) -> String {
    let neutron_proof: Ics23MerkleProof = borsh::from_slice(&proof.data).unwrap();
    let neutron_proof_value = neutron_proof.value;
    let raw_string = String::from_utf8_lossy(&neutron_proof_value).to_string();
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
