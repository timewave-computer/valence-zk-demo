use ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof;
use ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof;
use valence_coprocessor_core::SmtOpening;

/// Inputs for the coprocessor circuit that contains merkle proofs from different domains
/// and their corresponding roots for verification.
#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct CoprocessorCircuitInputs {
    /// Vector of Ethereum merkle proofs, each containing:
    /// - Account proof
    /// - Storage proof
    /// - Account hash
    /// - SmtOpening
    /// - SmtOpening
    pub ethereum_merkle_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)>,
    /// Vector of Neutron merkle proofs
    pub neutron_merkle_proofs: Vec<Ics23MerkleProof>,
    /// The root hash of the Neutron state tree
    pub neutron_root: Vec<u8>,
    /// The root hash of the Ethereum state tree
    pub ethereum_root: Vec<u8>,
    /// The opening proof of the ethereum light client root
    pub ethereum_root_opening: SmtOpening,
    /// The opening proof of the neutron light client root
    pub neutron_root_opening: SmtOpening,
    /// The coprocessor root hash
    pub coprocessor_root: [u8; 32],
}

/// Outputs from the coprocessor circuit containing the verified roots
#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct CoprocessorCircuitOutputs {
    /// The verified Neutron root hash
    pub neutron_root: Vec<u8>,
    /// The verified Ethereum root hash
    pub ethereum_root: Vec<u8>,
    /// The verified ethereum keys
    pub ethereum_keys: Vec<Vec<u8>>,
    /// The verified neutron keys
    pub neutron_keys: Vec<String>,
    /// The coprocessor root hash
    pub coprocessor_root: [u8; 32],
}
