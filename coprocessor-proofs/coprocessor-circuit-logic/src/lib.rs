use common_merkle_proofs::merkle::types::MerkleVerifiable;
use types::{MerkleProofInputs, MerkleProofOutputs};
use valence_smt::MemorySmt;

// todo: factor the logic from coprocessor-circuit-sp1 into this file
pub fn coprocessor_logic(inputs: MerkleProofInputs) -> Vec<u8> {
    assert!(MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.ethereum_root_opening,
    ));
    assert!(MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.neutron_root_opening,
    ));

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
    }
    for neutron_proof in inputs.neutron_merkle_proofs {
        // verify the proof against the neutron root
        neutron_proof
            .verify(&inputs.neutron_root)
            .expect("Failed to verify Neutron storage proof");
    }
    borsh::to_vec(&MerkleProofOutputs {
        neutron_root: inputs.neutron_root,
        ethereum_root: inputs.ethereum_root,
        coprocessor_root: inputs.coprocessor_root,
    })
    .expect("Failed to serialize circuit outputs")
}
