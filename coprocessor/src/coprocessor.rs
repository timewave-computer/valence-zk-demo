use crate::{read_ethereum_rpc_url, read_neutron_rpc_url};
use common_merkle_proofs::merkle::types::MerkleClient;
use ethereum_merkle_proofs::{
    ethereum_rpc::rpc::EvmMerkleRpcClient,
    merkle_lib::types::{EthereumMerkleProof, decode_rlp_bytes},
};
use ics23_merkle_proofs::{
    keys::Ics23Key, merkle_lib::types::Ics23MerkleProof, rpc::Ics23MerkleRpcClient,
};
use valence_coprocessor_core::MemorySmt;

pub type EthereumKey = Vec<u8>;

pub struct NeutronMerkleProofProvider {
    pub neutron_rpc_client: Ics23MerkleRpcClient,
}
impl NeutronMerkleProofProvider {
    /// Fetches a storage proof for a given key at a specific block height
    ///
    /// # Arguments
    /// * `key` - The storage key to fetch the proof for
    /// * `height` - The block height to fetch the proof from
    ///
    /// # Returns
    /// The raw storage proof bytes
    async fn get_storage_proof(&self, key: &Ics23Key, height: u64) -> Vec<u8> {
        self.neutron_rpc_client
            .get_proof(&key.to_string(), "", height)
            .await
            .unwrap()
    }
}
pub struct EthereumMerkleProofProvider {
    pub ethereum_rpc_client: EvmMerkleRpcClient,
}
impl EthereumMerkleProofProvider {
    async fn get_account_and_storage_proof(
        &self,
        key: (EthereumKey, String),
        ethereum_height: u64,
    ) -> (EthereumMerkleProof, EthereumMerkleProof) {
        let (account_proof, storage_proof) = self
            .ethereum_rpc_client
            .get_account_and_storage_proof(&alloy::hex::encode(&key.0), &key.1, ethereum_height)
            .await
            .unwrap();
        (account_proof, storage_proof)
    }
}
pub struct Coprocessor {
    /// The Sparse Merkle Tree used for storing and verifying proofs
    pub smt_tree: MemorySmt,
    /// The current root hash of the SMT
    pub smt_root: [u8; 32],
    /// RPC client for interacting with Neutron chain
    pub neutron_coprocessor: NeutronMerkleProofProvider,
    /// RPC client for interacting with Ethereum chain
    pub ethereum_coprocessor: EthereumMerkleProofProvider,
    // light client specific
    pub trusted_neutron_height: u64,
    pub trusted_ethereum_height: u64,
    pub target_neutron_height: u64,
    pub target_ethereum_height: u64,
    pub trusted_neutron_root: Vec<u8>,
    pub trusted_ethereum_root: Vec<u8>,
}

impl Coprocessor {
    pub fn from_env() -> Self {
        let smt_tree = MemorySmt::default();
        let smt_root = [0; 32];
        let neutron_coprocessor = NeutronMerkleProofProvider {
            neutron_rpc_client: Ics23MerkleRpcClient {
                rpc_url: read_neutron_rpc_url(),
            },
        };
        let ethereum_coprocessor = EthereumMerkleProofProvider {
            ethereum_rpc_client: EvmMerkleRpcClient {
                rpc_url: read_ethereum_rpc_url(),
            },
        };
        Self {
            smt_tree,
            smt_root,
            neutron_coprocessor,
            ethereum_coprocessor,
            target_ethereum_height: 0,
            target_neutron_height: 0,
            trusted_ethereum_height: 0,
            trusted_neutron_height: 0,
            trusted_ethereum_root: vec![],
            trusted_neutron_root: vec![],
        }
    }

    pub async fn get_storage_merkle_proofs(
        &mut self,
        neutron_height: u64,
        ethereum_height: u64,
        neutron_storage_keys: Vec<Ics23Key>,
        ethereum_storage_keys: Vec<(EthereumKey, String)>,
    ) -> (
        Vec<Ics23MerkleProof>,
        Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)>,
    ) {
        let mut neutron_merkle_proofs: Vec<Ics23MerkleProof> = Vec::new();
        let mut ethereum_merkle_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)> =
            Vec::new();
        for key in neutron_storage_keys.iter() {
            let proof = self
                .neutron_coprocessor
                .get_storage_proof(&key, neutron_height)
                .await;
            let proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
            neutron_merkle_proofs.push(proof);
        }
        for key in ethereum_storage_keys.iter() {
            let (account_proof, storage_proof) = self
                .ethereum_coprocessor
                .get_account_and_storage_proof((key.0.clone(), key.1.clone()), ethereum_height)
                .await;
            let account_decoded = decode_rlp_bytes(&account_proof.value).unwrap();
            ethereum_merkle_proofs.push((
                account_proof,
                storage_proof,
                account_decoded.get(2).unwrap().to_vec(),
            ));
        }

        (neutron_merkle_proofs, ethereum_merkle_proofs)
    }
}
