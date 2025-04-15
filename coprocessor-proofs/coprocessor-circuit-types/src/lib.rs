use ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof;
use ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof;

#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct MerkleProofInputs {
    // an Ethereum wasm proof consists of an account proof and a storage proof
    // we can later process either of, but for now we process both
    pub ethereum_merkle_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)>,
    pub neutron_merkle_proofs: Vec<Ics23MerkleProof>,
    pub neutron_root: Vec<u8>,
    pub ethereum_root: Vec<u8>,
}

#[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct MerkleProofOutputs {
    pub neutron_root: Vec<u8>,
    pub ethereum_root: Vec<u8>,
    //pub coprocessor_root: Vec<u8>,
}
