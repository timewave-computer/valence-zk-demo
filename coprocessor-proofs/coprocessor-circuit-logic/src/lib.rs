use common_merkle_proofs::merkle::types::MerkleVerifiable;
use types::{CoprocessorCircuitInputs, CoprocessorCircuitOutputs};
use valence_smt::MemorySmt;

// todo: factor the logic from coprocessor-circuit-sp1 into this file
pub fn coprocessor_logic(inputs: CoprocessorCircuitInputs) -> Vec<u8> {
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

    for ethereum_proof in inputs.ethereum_merkle_proofs.clone() {
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
    for neutron_proof in inputs.neutron_merkle_proofs.clone() {
        // verify the proof against the neutron root
        neutron_proof
            .verify(&inputs.neutron_root)
            .expect("Failed to verify Neutron storage proof");
    }

    // todo: commit the keys so we know which values were actually verified
    let verified_ethereum_keys: Vec<Vec<u8>> = inputs
        .ethereum_merkle_proofs
        .iter()
        .map(|proof| proof.1.key.clone())
        .collect();

    let verified_neutron_keys: Vec<String> = inputs
        .neutron_merkle_proofs
        .iter()
        .map(|proof| proof.key.to_string())
        .collect();

    borsh::to_vec(&CoprocessorCircuitOutputs {
        neutron_root: inputs.neutron_root,
        ethereum_root: inputs.ethereum_root,
        coprocessor_root: inputs.coprocessor_root,
        ethereum_keys: verified_ethereum_keys,
        neutron_keys: verified_neutron_keys,
    })
    .expect("Failed to serialize circuit outputs")
}
