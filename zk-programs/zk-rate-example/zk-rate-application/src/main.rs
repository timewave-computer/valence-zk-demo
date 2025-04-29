#![no_main]

use types::{
    RateApplicationCircuitInputs, RateApplicationCircuitOutputs,
    deserialize_ethereum_proof_value_as_u256, deserialize_neutron_proof_value_as_u256,
};
use valence_coprocessor_core::MemorySmt;

sp1_zkvm::entrypoint!(main);
fn main() {
    let inputs: RateApplicationCircuitInputs = borsh::from_slice(&sp1_zkvm::io::read_vec())
        .expect("Failed to deserialize RateApplicationCircuitInputs");
}
