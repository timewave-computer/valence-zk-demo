use alloy_primitives::U256;
use borsh::{BorshDeserialize, BorshSerialize};
use ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof;
use ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof;
use valence_coprocessor_core::SmtOpening;

/// Inputs for the rate application circuit that contains all necessary merkle proofs
/// and SMT openings for verifying vault balances and shares across different domains.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct RateApplicationCircuitInputs {
    /// SMT opening for the Neutron vault balance proof
    pub neutron_vault_balance_opening: SmtOpening,
    /// SMT opening for the Neutron vault shares proof
    pub neutron_vault_shares_opening: SmtOpening,
    /// SMT opening for the Ethereum vault balance proof
    pub ethereum_vault_balance_opening: SmtOpening,
    /// SMT opening for the Ethereum vault shares proof
    pub ethereum_vault_shares_opening: SmtOpening,
    /// Root of the coprocessor SMT tree
    pub coprocessor_root: [u8; 32],
}

/// Outputs from the rate application circuit containing the calculated rate
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RateApplicationCircuitOutputs {
    /// The calculated rate based on total balances and shares across domains
    pub rate: u64,
}

/// Deserializes an Ethereum merkle proof value into a U256 number
///
/// # Arguments
/// * `proof` - The SMT opening containing the Ethereum merkle proof
///
/// # Returns
/// The deserialized value as a U256 number
pub fn deserialize_ethereum_proof_value_as_u256(proof: SmtOpening) -> U256 {
    let ethereum_proof: EthereumMerkleProof = borsh::from_slice(&proof.data).unwrap();
    decode_rlp_u256_alloy(&ethereum_proof.value).unwrap()
}

/// Deserializes a Neutron merkle proof value into a U256 number
///
/// # Arguments
/// * `proof` - The SMT opening containing the Neutron merkle proof
///
/// # Returns
/// The deserialized value as a U256 number
pub fn deserialize_neutron_proof_value_as_u256(proof: SmtOpening) -> U256 {
    let neutron_proof: Ics23MerkleProof = borsh::from_slice(&proof.data).unwrap();
    let neutron_proof_value = neutron_proof.value;
    let neutron_value_decoded = &String::from_utf8_lossy(&neutron_proof_value);
    U256::from_str_radix(serde_json::from_str(neutron_value_decoded).unwrap(), 10).unwrap()
}

use alloy_rlp::Decodable;
fn decode_rlp_u256_alloy(mut rlp_bytes: &[u8]) -> Result<U256, String> {
    U256::decode(&mut rlp_bytes).map_err(|e| format!("RLP decode error: {:?}", e))
}
