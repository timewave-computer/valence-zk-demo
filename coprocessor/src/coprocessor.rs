use ethereum_merkle_proofs::{
    ethereum_rpc::rpc::EvmMerkleRpcClient,
    merkle_lib::types::{EthereumMerkleProof, decode_rlp_bytes},
};
use ics23_merkle_proofs::{
    keys::Ics23Key, merkle_lib::types::Ics23MerkleProof, rpc::Ics23MerkleRpcClient,
};
use sp1_sdk::{ProverClient, SP1Stdin};
use valence_coprocessor_core::{MemorySmt, SmtOpening};

use crate::{COPROCESSOR_CIRCUIT_ELF, read_ethereum_rpc_url, read_neutron_rpc_url};
use common_merkle_proofs::merkle::types::MerkleClient;
use coprocessor_circuit_types::CoprocessorCircuitInputs;

/// A type alias for Ethereum storage keys represented as byte vectors
pub type EthereumKey = Vec<u8>;

/// A trait defining the common interface for SMT operations across different coprocessors
///
/// This trait provides a common interface for retrieving SMT openings, which is the only
/// operation that is truly common between different chain implementations.
pub trait CoprocessorInterface {
    fn get_smt_opening(&mut self, key: &Vec<u8>, tree: &MemorySmt, root: [u8; 32]) -> SmtOpening;
}

/// A struct for interacting with Neutron chain state
///
/// This struct provides methods for retrieving and verifying state from the Neutron chain,
/// including storage proofs. Chain-specific operations are implemented as regular methods
/// rather than being forced into a trait since they are fundamentally different from
/// other chain implementations.
pub struct NeutronCoprocessor {
    pub neutron_rpc_client: Ics23MerkleRpcClient,
}
impl NeutronCoprocessor {
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
impl CoprocessorInterface for NeutronCoprocessor {
    fn get_smt_opening(&mut self, key: &Vec<u8>, tree: &MemorySmt, root: [u8; 32]) -> SmtOpening {
        tree.get_opening("demo", root, &key).unwrap().unwrap()
    }
}
/// A struct for interacting with Ethereum chain state
///
/// This struct provides methods for retrieving and verifying state from the Ethereum chain,
/// including account and storage proofs. Chain-specific operations are implemented as regular
/// methods rather than being forced into a trait since they are fundamentally different from
/// other chain implementations.
pub struct EthereumCoprocessor {
    pub ethereum_rpc_client: EvmMerkleRpcClient,
}
impl EthereumCoprocessor {
    /// Fetches both account and storage proofs for a given key at a specific block height
    ///
    /// # Arguments
    /// * `key` - A tuple containing the storage key and contract address
    /// * `height` - The block height to fetch the proofs from
    ///
    /// # Returns
    /// A tuple containing:
    /// * The account proof
    /// * The storage proof
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
impl CoprocessorInterface for EthereumCoprocessor {
    fn get_smt_opening(&mut self, key: &Vec<u8>, tree: &MemorySmt, root: [u8; 32]) -> SmtOpening {
        tree.get_opening("demo", root, &key).unwrap().unwrap()
    }
}
/// A coprocessor that handles merkle proofs for both Ethereum and Neutron chains
///
/// This struct maintains state for both chains including SMT trees, storage keys,
/// and RPC clients for fetching proofs. It implements the CoprocessorInterface trait
/// for common SMT operations while keeping chain-specific operations as regular methods.
pub struct Coprocessor {
    /// The Sparse Merkle Tree used for storing and verifying proofs
    pub smt_tree: MemorySmt,
    /// The current root hash of the SMT
    pub smt_root: [u8; 32],
    /// Storage keys for Neutron chain
    pub neutron_storage_keys: Vec<Ics23Key>,
    /// Storage keys for Ethereum chain with their associated contract addresses
    pub ethereum_storage_keys: Vec<(EthereumKey, String)>,
    // todo: add keys for ethereum receipts, currently unsupported but necessary for event verification
    // for example: ERC20 Transfer events
    // pub ethereum_receipt_keys: Vec<(EthereumKey, LogFilter)> (or similar)
    /// RPC client for interacting with Neutron chain
    pub neutron_coprocessor: NeutronCoprocessor,
    /// RPC client for interacting with Ethereum chain
    pub ethereum_coprocessor: EthereumCoprocessor,
}

impl Coprocessor {
    /// Creates a new Coprocessor instance with configuration from environment variables
    ///
    /// Initializes empty storage keys and RPC clients using environment variables
    /// for configuration.
    pub fn from_env() -> Self {
        let smt_tree = MemorySmt::default();
        let smt_root = [0; 32];
        let neutron_coprocessor = NeutronCoprocessor {
            neutron_rpc_client: Ics23MerkleRpcClient {
                rpc_url: read_neutron_rpc_url(),
            },
        };
        let ethereum_coprocessor = EthereumCoprocessor {
            ethereum_rpc_client: EvmMerkleRpcClient {
                rpc_url: read_ethereum_rpc_url(),
            },
        };
        Self {
            smt_tree,
            smt_root,
            neutron_storage_keys: vec![],
            ethereum_storage_keys: vec![],
            neutron_coprocessor,
            ethereum_coprocessor,
        }
    }

    /// Creates a new Coprocessor instance with specified storage keys
    ///
    /// # Arguments
    /// * `neutron_storage_keys` - Storage keys for Neutron chain
    /// * `ethereum_storage_keys` - Storage keys for Ethereum chain with contract addresses
    pub fn from_env_with_storage_keys(
        neutron_storage_keys: Vec<Ics23Key>,
        ethereum_storage_keys: Vec<(EthereumKey, String)>,
    ) -> Self {
        let mut coprocessor = Self::from_env();
        coprocessor.neutron_storage_keys = neutron_storage_keys;
        coprocessor.ethereum_storage_keys = ethereum_storage_keys;
        coprocessor
    }

    /// Fetches merkle proofs for all configured storage keys from both chains
    ///
    /// # Arguments
    /// * `neutron_height` - Block height to fetch proofs from on Neutron
    /// * `ethereum_height` - Block height to fetch proofs from on Ethereum
    ///
    /// # Returns
    /// A tuple containing:
    /// * Vector of Neutron merkle proofs
    /// * Vector of Ethereum merkle proofs with associated account data
    pub async fn get_storage_merkle_proofs(
        &mut self,
        neutron_height: u64,
        ethereum_height: u64,
    ) -> (
        Vec<Ics23MerkleProof>,
        Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)>,
    ) {
        let mut neutron_merkle_proofs: Vec<Ics23MerkleProof> = Vec::new();
        let mut ethereum_merkle_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)> =
            Vec::new();
        for key in self.neutron_storage_keys.iter() {
            let proof = self
                .neutron_coprocessor
                .get_storage_proof(&key, neutron_height)
                .await;
            let proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
            neutron_merkle_proofs.push(proof);
        }
        for key in self.ethereum_storage_keys.iter() {
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

    /// Proves the new coprocessor state in zk
    ///
    /// # Arguments
    /// * `neutron_merkle_proofs` - Merkle proofs from Neutron chain
    /// * `ethereum_merkle_proofs` - Merkle proofs from Ethereum chain
    /// * `ethereum_root` - Root hash of Ethereum state
    /// * `neutron_root` - Root hash of Neutron state
    ///
    /// # Returns
    /// A tuple containing the SP1 proof and verifying key
    pub async fn prove_progression(
        &mut self,
        neutron_merkle_proofs: Vec<Ics23MerkleProof>,
        ethereum_merkle_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)>,
        ethereum_root: Vec<u8>,
        neutron_root: Vec<u8>,
    ) -> (sp1_sdk::SP1ProofWithPublicValues, sp1_sdk::SP1VerifyingKey) {
        for proof in neutron_merkle_proofs.clone() {
            self.smt_root = self
                .smt_tree
                .insert(
                    self.smt_root,
                    "demo",
                    &borsh::to_vec(&proof).unwrap(),
                    borsh::to_vec(&proof).unwrap(),
                )
                .unwrap();
        }
        for proof in ethereum_merkle_proofs.clone() {
            self.smt_root = self
                .smt_tree
                .insert(
                    self.smt_root,
                    "demo",
                    &borsh::to_vec(&proof.1).unwrap(),
                    borsh::to_vec(&proof.1).unwrap(),
                )
                .unwrap();
        }

        self.smt_root = self
            .smt_tree
            .insert(
                self.smt_root,
                "demo",
                &borsh::to_vec(&neutron_root).unwrap(),
                borsh::to_vec(&neutron_root).unwrap(),
            )
            .unwrap();

        self.smt_root = self
            .smt_tree
            .insert(
                self.smt_root,
                "demo",
                &borsh::to_vec(&ethereum_root).unwrap(),
                borsh::to_vec(&ethereum_root).unwrap(),
            )
            .unwrap();

        let ethereum_root_opening = self
            .smt_tree
            .get_opening(
                "demo",
                self.smt_root,
                &borsh::to_vec(&ethereum_root).unwrap(),
            )
            .unwrap()
            .unwrap();

        let neutron_root_opening = self
            .smt_tree
            .get_opening(
                "demo",
                self.smt_root,
                &borsh::to_vec(&neutron_root).unwrap(),
            )
            .unwrap()
            .unwrap();

        let inputs = CoprocessorCircuitInputs {
            ethereum_merkle_proofs,
            neutron_merkle_proofs,
            neutron_root,
            ethereum_root,
            ethereum_root_opening,
            neutron_root_opening,
            coprocessor_root: self.smt_root,
        };
        let inputs_serialized = borsh::to_vec(&inputs).unwrap();

        let client = ProverClient::from_env();
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(inputs_serialized);
        let (pk, vk) = client.setup(COPROCESSOR_CIRCUIT_ELF);

        // prove the coprocessor circuit
        let proof = client
            .prove(&pk, &stdin)
            .groth16()
            .run()
            .expect("Failed to prove");
        (proof, vk)
    }
}
