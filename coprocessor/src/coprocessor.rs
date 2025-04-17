use ethereum_merkle_proofs::{
    ethereum_rpc::rpc::EvmMerkleRpcClient,
    merkle_lib::types::{EthereumMerkleProof, decode_rlp_bytes},
};
use ics23_merkle_proofs::{
    keys::Ics23Key, merkle_lib::types::Ics23MerkleProof, rpc::Ics23MerkleRpcClient,
};
use sp1_sdk::{ProverClient, SP1Stdin};
use valence_smt::{MemorySmt, SmtOpening};

use crate::{COPROCESSOR_CIRCUIT_ELF, read_ethereum_rpc_url, read_neutron_rpc_url};
use common_merkle_proofs::merkle::types::MerkleClient;
use coprocessor_circuit_types::MerkleProofInputs as CoprocessorCircuitInputs;

pub type EthereumKey = Vec<u8>;

pub struct Coprocessor {
    pub smt_tree: MemorySmt,
    pub smt_root: [u8; 32],
    pub neutron_storage_keys: Vec<Ics23Key>,
    pub ethereum_storage_keys: Vec<(EthereumKey, String)>,
    pub neutron_rpc_client: Ics23MerkleRpcClient,
    pub ethereum_rpc_client: EvmMerkleRpcClient,
}

impl Coprocessor {
    pub fn from_env() -> Self {
        let smt_tree = MemorySmt::default();
        let smt_root = [0; 32];
        let neutron_rpc_client = Ics23MerkleRpcClient {
            rpc_url: read_neutron_rpc_url(),
        };
        let ethereum_rpc_client = EvmMerkleRpcClient {
            rpc_url: read_ethereum_rpc_url(),
        };
        Self {
            smt_tree,
            smt_root,
            neutron_storage_keys: vec![],
            ethereum_storage_keys: vec![],
            neutron_rpc_client,
            ethereum_rpc_client,
        }
    }

    pub fn from_env_with_storage_keys(
        neutron_storage_keys: Vec<Ics23Key>,
        ethereum_storage_keys: Vec<(EthereumKey, String)>,
    ) -> Self {
        let mut coprocessor = Self::from_env();
        coprocessor.neutron_storage_keys = neutron_storage_keys;
        coprocessor.ethereum_storage_keys = ethereum_storage_keys;
        coprocessor
    }

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
                .neutron_rpc_client
                .get_proof(&key.to_string(), "", neutron_height)
                .await
                .unwrap();
            let proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
            neutron_merkle_proofs.push(proof);
        }
        for key in self.ethereum_storage_keys.iter() {
            let (account_proof, storage_proof) = self
                .ethereum_rpc_client
                .get_account_and_storage_proof(&alloy::hex::encode(&key.0), &key.1, ethereum_height)
                .await
                .unwrap();
            let account_decoded = decode_rlp_bytes(&account_proof.value).unwrap();
            ethereum_merkle_proofs.push((
                account_proof,
                storage_proof,
                account_decoded.get(2).unwrap().to_vec(),
            ));
        }

        (neutron_merkle_proofs, ethereum_merkle_proofs)
    }

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
                .insert(self.smt_root, "demo", borsh::to_vec(&proof).unwrap())
                .unwrap();
        }
        for proof in ethereum_merkle_proofs.clone() {
            self.smt_root = self
                .smt_tree
                .insert(self.smt_root, "demo", borsh::to_vec(&proof.1).unwrap())
                .unwrap();
        }

        self.smt_root = self
            .smt_tree
            .insert(self.smt_root, "demo", borsh::to_vec(&neutron_root).unwrap())
            .unwrap();

        self.smt_root = self
            .smt_tree
            .insert(
                self.smt_root,
                "demo",
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

    pub fn get_ethereum_opening(&mut self, key: &Vec<u8>) -> SmtOpening {
        self.smt_tree
            .get_opening("demo", self.smt_root, &key)
            .unwrap()
            .unwrap()
    }

    pub fn get_neutron_opening(&mut self, key: &Vec<u8>) -> SmtOpening {
        self.smt_tree
            .get_opening("demo", self.smt_root, &key)
            .unwrap()
            .unwrap()
    }
}
