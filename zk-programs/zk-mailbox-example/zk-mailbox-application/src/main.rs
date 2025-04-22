#![no_main]

use types::{
    MailboxApplicationCircuitInputs, MailboxApplicationCircuitOutputs,
    deserialize_ethereum_proof_value_as_string, deserialize_neutron_proof_value_as_string,
};
use valence_smt::MemorySmt;

sp1_zkvm::entrypoint!(main);
fn main() {
    let inputs: MailboxApplicationCircuitInputs = borsh::from_slice(&sp1_zkvm::io::read_vec())
        .expect("Failed to deserialize MailboxApplicationCircuitInputs");
    let mut messages: Vec<String> = Vec::new();
    for ethereum_proof in inputs.ethereum_message_openings {
        assert!(MemorySmt::verify(
            "demo",
            &inputs.coprocessor_root.try_into().unwrap(),
            &ethereum_proof,
        ));
        messages.push(deserialize_ethereum_proof_value_as_string(ethereum_proof));
    }
    for neutron_proof in inputs.neutron_message_openings {
        assert!(MemorySmt::verify(
            "demo",
            &inputs.coprocessor_root.try_into().unwrap(),
            &neutron_proof,
        ));
        messages.push(deserialize_neutron_proof_value_as_string(neutron_proof));
    }
    let outputs = MailboxApplicationCircuitOutputs { messages };
    sp1_zkvm::io::commit_slice(&borsh::to_vec(&outputs).unwrap());
}
