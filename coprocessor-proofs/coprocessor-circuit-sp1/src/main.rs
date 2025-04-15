/*
    This circuit verifies merkle proofs from different domains and builds a SMT from them.
    The SMT trie root is committed as a public output.
    The new trie root, alongslide with the proof can be sent to the different domains.
*/
#![no_main]
use borsh;
use common_merkle_proofs::merkle::types::MerkleVerifiable;
use types::{MerkleProofInputs, MerkleProofOutputs};
use valence_smt::MemorySmt;
sp1_zkvm::entrypoint!(main);
pub fn main() {
    let inputs: MerkleProofInputs = borsh::from_slice(&sp1_zkvm::io::read_vec())
        .expect("Failed to deserialize MerkleProofInputs");
    // build the SMT from the merkle proofs
    // later we will want to insert into an existing tree here
    let mut smt_tree = MemorySmt::default();
    let mut root = [0; 32];
    for ethereum_proof in inputs.ethereum_merkle_proofs {
        // verify the storage proof against the account hash
        ethereum_proof
            .1
            .verify(&ethereum_proof.2)
            .expect("Failed to verify Ethereum storage proof");
        // verify the account proof against the ethereum root
        ethereum_proof
            .0
            .verify(&inputs.ethereum_root)
            .expect("Failed to verify Ethereum account proof");
        root = smt_tree
            .insert(root, "demo", ethereum_proof.1.value.clone())
            .expect("Failed to insert");
    }
    for neutron_proof in inputs.neutron_merkle_proofs {
        // verify the proof against the neutron root
        neutron_proof
            .verify(&inputs.neutron_root)
            .expect("Failed to verify Neutron storage proof");
        root = smt_tree
            .insert(root, "demo", neutron_proof.value.clone())
            .expect("Failed to insert");
    }
    sp1_zkvm::io::commit_slice(
        &borsh::to_vec(&MerkleProofOutputs {
            neutron_root: inputs.neutron_root,
            ethereum_root: inputs.ethereum_root,
            coprocessor_root: root.to_vec(),
        })
        .unwrap(),
    );
}
