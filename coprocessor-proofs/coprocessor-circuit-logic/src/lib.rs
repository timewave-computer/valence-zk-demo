use alloy_sol_types::{SolType, sol};
use sp1_verifier::Groth16Verifier;
use types::CoprocessorCircuitInputs;

// todo: factor the logic from coprocessor-circuit-sp1 into this file
pub fn coprocessor_logic(inputs: CoprocessorCircuitInputs) -> Vec<u8> {
    let tendermint_output: TendermintOutput =
        TendermintOutput::abi_decode(&inputs.tendermint_public_values, false).unwrap();
    let helios_output: ProofOutputs =
        ProofOutputs::abi_decode(&inputs.helios_public_values, false).unwrap();

    // these are the targets that we want to insert and commit
    /*let target_tendermint_root = tendermint_output.targetHeaderHash.to_vec();
    let target_ethereum_root = helios_output.newHeader.to_vec();
    let target_tendermint_height = tendermint_output.targetHeight;
    let target_ethereum_height = helios_output.newHead;*/
    // verify the smt inserts of these targets

    // todo assert the inputs that we expect to match
    // previous roots must match and previous heights must be less than current heights

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

sol! {
    struct TendermintOutput {
        uint64 trustedHeight;
        uint64 targetHeight;
        bytes32 trustedHeaderHash;
        bytes32 targetHeaderHash;
    }
}

sol! {
    struct ProofOutputs {
        /// The previous beacon block header hash.
        bytes32 prevHeader;
        /// The slot of the previous head.
        uint256 prevHead;
        /// The anchor sync committee hash which was used to verify the proof.
        bytes32 prevSyncCommitteeHash;
        /// The slot of the new head.
        uint256 newHead;
        /// The new beacon block header hash.
        bytes32 newHeader;
        /// The execution state root from the execution payload of the new beacon block.
        bytes32 executionStateRoot;
        /// The sync committee hash of the current period.
        bytes32 syncCommitteeHash;
        /// The sync committee hash of the next period.
        bytes32 nextSyncCommitteeHash;
    }
}
