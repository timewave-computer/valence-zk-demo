#![no_main]

use types::{
    MailboxApplicationCircuitInputs, MailboxApplicationCircuitOutputs,
    deserialize_ethereum_proof_value_as_string, deserialize_neutron_proof_value_as_string,
};
use valence_coprocessor_core::MemorySmt;

sp1_zkvm::entrypoint!(main);
fn main() {
    let inputs: MailboxApplicationCircuitInputs = serde_json::from_slice::<MailboxApplicationCircuitInputs>(&sp1_zkvm::io::read_vec())
        .expect("Failed to deserialize MailboxApplicationCircuitInputs");
}
