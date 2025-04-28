use crate::coprocessor::Coprocessor;
use coprocessor_circuit_types::CoprocessorCircuitOutputs;
use sp1_sdk::HashableKey;
use sp1_verifier::Groth16Verifier;

#[cfg(feature = "mailbox")]
pub mod mailbox;
#[cfg(feature = "rate")]
pub mod rate;

pub async fn prove_coprocessor(
    coprocessor: &mut Coprocessor,
    merkle_proofs: (
        Vec<ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof>,
        Vec<(
            ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof,
            ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof,
            Vec<u8>,
        )>,
    ),
    ethereum_root: Vec<u8>,
    neutron_root: Vec<u8>,
) {
    #[cfg(feature = "coprocessor")]
    {
        let (proof, vk) = coprocessor
            .prove_progression(
                merkle_proofs.0.clone(),
                merkle_proofs.1.clone(),
                ethereum_root,
                neutron_root,
            )
            .await;
        let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
        Groth16Verifier::verify(
            &proof.bytes(),
            &proof.public_values.to_vec(),
            &vk.bytes32(),
            groth16_vk,
        )
        .unwrap();
        let coprocessor_circuit_outputs: CoprocessorCircuitOutputs =
            borsh::from_slice(proof.public_values.as_slice()).unwrap();
        println!(
            "Coprocessor Circuit Outputs: {:?}",
            coprocessor_circuit_outputs
        );
    }

    #[cfg(not(feature = "coprocessor"))]
    {
        for proof in merkle_proofs.0.clone() {
            coprocessor.smt_root = coprocessor
                .smt_tree
                .insert(
                    coprocessor.smt_root,
                    "demo",
                    &borsh::to_vec(&proof).unwrap(),
                    borsh::to_vec(&proof).unwrap(),
                )
                .unwrap();
        }
        for proof in merkle_proofs.1.clone() {
            coprocessor.smt_root = coprocessor
                .smt_tree
                .insert(
                    coprocessor.smt_root,
                    "demo",
                    &borsh::to_vec(&proof.1).unwrap(),
                    borsh::to_vec(&proof.1).unwrap(),
                )
                .unwrap();
        }
    }
}
