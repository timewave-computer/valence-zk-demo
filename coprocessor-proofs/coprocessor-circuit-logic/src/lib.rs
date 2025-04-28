use sp1_verifier::Groth16Verifier;
use types::CoprocessorCircuitInputs;

// todo: factor the logic from coprocessor-circuit-sp1 into this file
pub fn coprocessor_logic(inputs: CoprocessorCircuitInputs) -> Vec<u8> {
    let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
    // verify the helios update proof
    Groth16Verifier::verify(
        &inputs.tendermint_proof,
        &inputs.tendermint_public_values,
        &inputs.tendermint_vk,
        groth16_vk,
    )
    .expect("Failed to verify tendermint zk light client update");
    // verify the tendermint update proof
    Groth16Verifier::verify(
        &inputs.helios_proof,
        &inputs.helios_public_values,
        &inputs.helios_vk,
        groth16_vk,
    )
    .expect("Failed to verify helios zk light client update");
    // todo: commit the new SMT root after inserting the new roots at the target values
    vec![]
}

/* Decode the zk light client outputs

let tendermint_output: TendermintOutput =
serde_json::from_slice(&tendermint_light_client_proof.public_values.to_vec()).unwrap();

let helios_output: ProofOutputs = ProofOutputs::abi_decode(
    &ethereum_light_client_proof
        .unwrap()
        .unwrap()
        .public_values
        .to_vec(),
    false,
)
.unwrap();
*/
