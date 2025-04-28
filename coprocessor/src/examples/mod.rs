use alloy::{
    dyn_abi::SolType,
    signers::k256::sha2::{Digest, Sha256},
    sol,
};
use coprocessor_circuit_types::CoprocessorCircuitInputs;
use sp1_helios_primitives::types::ProofOutputs;
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

pub async fn prove_coprocessor(coprocessor: &mut Coprocessor) -> (TendermintOutput, ProofOutputs) {
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

    let tendermint_output: TendermintOutput =
        TendermintOutput::abi_decode(&tendermint_light_client_proof.public_values.to_vec(), false)
            .unwrap();
    let helios_output: ProofOutputs =
        ProofOutputs::abi_decode(&ethereum_light_client_proof.public_values.to_vec(), false)
            .unwrap();

    let target_tendermint_root: Vec<u8> = tendermint_output.targetHeaderHash.to_vec();
    let target_ethereum_root: Vec<u8> = helios_output.newHeader.to_vec();
    let target_tendermint_height: u64 = tendermint_output.targetHeight;
    let target_ethereum_height: u64 = helios_output.newHead.try_into().unwrap();

    let mut coprocessor_root = coprocessor.smt_root;
    let mut hasher = Sha256::new();
    hasher.update(&target_tendermint_height.to_be_bytes());
    let neutron_height_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&target_ethereum_height.to_be_bytes());
    let ethereum_height_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&target_tendermint_root);
    let tendermint_root_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&target_ethereum_root);
    let ethereum_root_key = hasher.finalize();

    coprocessor_root = coprocessor
        .smt_tree
        .insert(
            coprocessor_root,
            "demo",
            &neutron_height_key,
            target_tendermint_height.to_be_bytes().to_vec(),
        )
        .expect("Failed to insert Neutron Height");

    coprocessor_root = coprocessor
        .smt_tree
        .insert(
            coprocessor_root,
            "demo",
            &ethereum_height_key,
            target_ethereum_height.to_be_bytes().to_vec(),
        )
        .expect("Failed to insert Ethereum Height");

    coprocessor_root = coprocessor
        .smt_tree
        .insert(
            coprocessor_root,
            "demo",
            &tendermint_root_key,
            target_tendermint_root,
        )
        .expect("Failed to insert Ethereum Root");

    coprocessor_root = coprocessor
        .smt_tree
        .insert(
            coprocessor_root,
            "demo",
            &ethereum_root_key,
            target_ethereum_root,
        )
        .expect("Failed to insert Ethereum Root");

    let coprocessor_inputs = CoprocessorCircuitInputs {
        helios_proof: helios_proof_serialized,
        helios_public_values,
        helios_vk,
        tendermint_proof: tendermint_proof_serialized,
        tendermint_public_values,
        tendermint_vk,
        previous_neutron_height: coprocessor.trusted_neutron_height,
        previous_ethereum_height: coprocessor.trusted_ethereum_height,
        previous_neutron_root: coprocessor.trusted_neutron_root.to_vec(),
        previous_ethereum_root: coprocessor.trusted_ethereum_root.to_vec(),
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

    // this verification should happen on-chain
    // our co-processor must adapt the new state
    // for this we must serialize the outputs so that the
    // target chain can understand them
    let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
    Groth16Verifier::verify(
        &proof.bytes(),
        &proof.public_values.to_vec(),
        &vk.bytes32(),
        groth16_vk,
    )
    .unwrap();

    // return new state (or update on-chain)
    (tendermint_output, helios_output)
}

sol! {
    struct TendermintOutput {
        uint64 trustedHeight;
        uint64 targetHeight;
        bytes32 trustedHeaderHash;
        bytes32 targetHeaderHash;
    }
}
