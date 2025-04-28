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
    let neutron_balance =
        deserialize_neutron_proof_value_as_u256(inputs.neutron_vault_balance_opening.clone());
    let neutron_shares =
        deserialize_neutron_proof_value_as_u256(inputs.neutron_vault_shares_opening.clone());
    let ethereum_balance =
        deserialize_ethereum_proof_value_as_u256(inputs.ethereum_vault_balance_opening.clone());
    let ethereum_shares =
        deserialize_ethereum_proof_value_as_u256(inputs.ethereum_vault_shares_opening.clone());

    // verify the SMT opening proofs against the root
    assert!(MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root.try_into().unwrap(),
        &inputs.ethereum_vault_balance_opening,
    ));
    assert!(MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root.try_into().unwrap(),
        &inputs.ethereum_vault_shares_opening,
    ));
    assert!(MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root.try_into().unwrap(),
        &inputs.neutron_vault_balance_opening,
    ));
    assert!(MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root.try_into().unwrap(),
        &inputs.neutron_vault_shares_opening,
    ));
    // commit the rate as a public output
    sp1_zkvm::io::commit_slice(
        &borsh::to_vec(&RateApplicationCircuitOutputs {
            rate: ((neutron_balance + ethereum_balance) / (neutron_shares + ethereum_shares))
                .try_into()
                .unwrap(),
        })
        .unwrap(),
    );
}
