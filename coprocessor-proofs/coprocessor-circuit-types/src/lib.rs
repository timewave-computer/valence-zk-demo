use valence_coprocessor_core::SmtOpening;

/// Inputs for the coprocessor circuit that contains merkle proofs from different domains
/// and their corresponding roots for verification.
#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct CoprocessorCircuitInputs {
    pub helios_proof: Vec<u8>,
    pub helios_public_values: Vec<u8>,
    pub helios_vk: String,
    pub tendermint_proof: Vec<u8>,
    pub tendermint_public_values: Vec<u8>,
    pub tendermint_vk: String,
    pub previous_neutron_height: u64,
    pub previous_ethereum_height: u64,
    pub previous_neutron_root: Vec<u8>,
    pub previous_ethereum_root: Vec<u8>,
    pub openings: Vec<SmtOpening>,
    pub coprocessor_root: [u8; 32],
}

/// Outputs from the coprocessor circuit containing the verified roots
#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct CoprocessorCircuitOutputs {
    pub coprocessor_root: [u8; 32],
}
