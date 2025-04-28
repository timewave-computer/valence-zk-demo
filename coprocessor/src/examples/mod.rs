use coprocessor_circuit_types::CoprocessorCircuitInputs;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use sp1_verifier::Groth16Verifier;

use crate::{
    COPROCESSOR_CIRCUIT_ELF,
    coprocessor::Coprocessor,
    lightclients::{helios::SP1HeliosOperator, tendermint::SP1TendermintOperator},
};

#[cfg(feature = "mailbox")]
pub mod mailbox;
#[cfg(feature = "rate")]
pub mod rate;

pub async fn prove_coprocessor(coprocessor: &mut Coprocessor) {
    // todo: set the trusted values for Ethereum
    let tendermint_operator = SP1TendermintOperator::new(
        coprocessor.trusted_neutron_height,
        coprocessor.target_neutron_height,
    );
    let tendermint_light_client_proof = tendermint_operator.run().await;
    let mut ethereum_operator = SP1HeliosOperator::new();
    // todo: remove hardcoded ethereum height and replace it with a real trusted height
    let ethereum_light_client_proof = ethereum_operator.run(234644 * 32).await;

    let tendermint_proof_serialized = tendermint_light_client_proof.bytes();
    let tendermint_public_values = tendermint_light_client_proof.public_values.to_vec();
    let tendermint_vk =
        "0x00846ef8de8afd003f9c7638d009bbbd22ffcefe4720bbeb35ac467958e7ca76".to_string();

    let ethereum_light_client_proof = ethereum_light_client_proof.unwrap().unwrap();
    let helios_proof_serialized = ethereum_light_client_proof.bytes();
    let helios_public_values = ethereum_light_client_proof.public_values.to_vec();
    let helios_vk = ethereum_operator.get_vk();

    let coprocessor_inputs = CoprocessorCircuitInputs {
        helios_proof: helios_proof_serialized,
        helios_public_values,
        helios_vk,
        tendermint_proof: tendermint_proof_serialized,
        tendermint_public_values,
        tendermint_vk,
    };

    let coprocessor_circuit_inputs_serialized = borsh::to_vec(&coprocessor_inputs).unwrap();

    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();

    stdin.write_vec(coprocessor_circuit_inputs_serialized);
    let (pk, vk) = client.setup(COPROCESSOR_CIRCUIT_ELF);
    // generate the coprocessor update zkp
    let proof = client
        .prove(&pk, &stdin)
        .groth16()
        .run()
        .expect("Failed to prove");
    let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
    Groth16Verifier::verify(
        &proof.bytes(),
        &proof.public_values.to_vec(),
        &vk.bytes32(),
        groth16_vk,
    )
    .unwrap();
}
