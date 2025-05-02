use alloy_sol_types::{SolType, sol};
use sp1_verifier::Groth16Verifier;
use tendermint_program_types::TendermintOutput;
use types::CoprocessorCircuitInputs;
use valence_coprocessor_core::MemorySmt;

pub fn coprocessor_logic(inputs: CoprocessorCircuitInputs) -> [u8; 32] {
    let neutron_output: TendermintOutput =
        serde_json::from_slice(&inputs.neutron_public_values).unwrap();
    let helios_output: ProofOutputs =
        ProofOutputs::abi_decode(&inputs.helios_public_values, false).unwrap();
    // assert the trusted values
    assert!(inputs.previous_neutron_height < neutron_output.target_height);
    assert!(inputs.previous_ethereum_height < helios_output.newHead.try_into().unwrap());
    assert!(inputs.previous_neutron_root == neutron_output.target_header_hash.to_vec());
    assert!(inputs.previous_ethereum_root == helios_output.prevHeader.to_vec());

    // these are the targets that we want to insert and commit
    let target_neutron_height: u64 = neutron_output.target_height;
    let target_neutron_root: Vec<u8> = neutron_output.target_header_hash.to_vec();
    let target_ethereum_height: u64 = helios_output.newHead.try_into().unwrap();
    let target_ethereum_root: Vec<u8> = helios_output.newHeader.to_vec();

    // verify the smt inserts of these targets
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.neutron_height_opening,
    );
    assert_eq!(
        &inputs.neutron_height_opening.data,
        &target_neutron_height.to_be_bytes().to_vec()
    );
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.ethereum_height_opening,
    );
    assert_eq!(
        &inputs.ethereum_height_opening.data,
        &target_ethereum_height.to_be_bytes().to_vec()
    );
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.neutron_root_opening,
    );
    assert_eq!(&inputs.neutron_root_opening.data, &target_neutron_root);
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.ethereum_root_opening,
    );
    assert_eq!(&inputs.ethereum_root_opening.data, &target_ethereum_root);

    // the SP1 groth16 verification key
    let groth16_vk: &[u8] = *sp1_verifier::GROTH16_VK_BYTES;

    // verify the neutron update proof
    Groth16Verifier::verify(
        &inputs.neutron_proof,
        &inputs.neutron_public_values,
        &inputs.neutron_vk,
        groth16_vk,
    )
    .expect("Failed to verify neutron zk light client update");

    // verify the ethereum update proof
    Groth16Verifier::verify(
        &inputs.helios_proof,
        &inputs.helios_public_values,
        &inputs.helios_vk,
        groth16_vk,
    )
    .expect("Failed to verify helios zk light client update");
    inputs.coprocessor_root
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
