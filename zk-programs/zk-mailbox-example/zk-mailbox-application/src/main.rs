#![no_main]

// Important Note! There is a constraint missing in this example!
// We also need to constrain the keys that are being opened on the different domains.
// How exactly we do this might depend on the type of application we are writing.

use common_merkle_proofs::merkle::types::MerkleVerifiable;
use ssz_merkleize::merkleize::{merkleize_keys, uint64_to_le_256};
use types::{
    MailboxApplicationCircuitInputs, MailboxApplicationCircuitOutputs,
    deserialize_ethereum_proof_value_as_string, deserialize_neutron_proof_value_as_string,
};
use valence_coprocessor_core::MemorySmt;

sp1_zkvm::entrypoint!(main);
fn main() {
    let mut messages: Vec<String> = Vec::new();
    let inputs: MailboxApplicationCircuitInputs =
        serde_json::from_slice::<MailboxApplicationCircuitInputs>(&sp1_zkvm::io::read_vec())
            .expect("Failed to deserialize MailboxApplicationCircuitInputs");
    // todo: constrain the keys that are being used to match the deterministic keys for the domains
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.ethereum_height_opening,
    );
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.neutron_height_opening,
    );
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.neutron_root_opening,
    );
    MemorySmt::verify(
        "demo",
        &inputs.coprocessor_root,
        &inputs.ethereum_root_opening,
    );
    let tendermint_header_hash = inputs.neutron_block_header.hash().as_bytes().to_vec();
    // verify the neutron app hash against the header root
    assert_eq!(tendermint_header_hash, inputs.neutron_root_opening.data);
    let target_header_root = merkleize_keys(vec![
        uint64_to_le_256(inputs.beacon_block_header.slot.parse::<u64>().unwrap()),
        uint64_to_le_256(
            inputs
                .beacon_block_header
                .proposer_index
                .parse::<u64>()
                .unwrap(),
        ),
        hex::decode(
            inputs
                .beacon_block_header
                .parent_root
                .trim_start_matches("0x"),
        )
        .unwrap()
        .to_vec(),
        hex::decode(
            inputs
                .beacon_block_header
                .state_root
                .trim_start_matches("0x"),
        )
        .unwrap()
        .to_vec(),
        hex::decode(
            inputs
                .beacon_block_header
                .body_root
                .trim_start_matches("0x"),
        )
        .unwrap()
        .to_vec(),
    ]);
    assert_eq!(target_header_root, inputs.ethereum_root_opening.data);
    // the ethereum body root against which we verify our execution client state root
    let ethereum_header_body_root = hex::decode(
        &inputs
            .beacon_block_header
            .body_root
            .trim_start_matches("0x"),
    )
    .unwrap();
    // the neutron app hash against which we verify our storage proofs
    let neutron_app_hash = inputs.neutron_block_header.app_hash.as_bytes().to_vec();
    // verify the ethereum storage proofs
    for ethereum_proof in inputs.ethereum_storage_proofs {
        ethereum_proof
            .1
            .verify(&ethereum_proof.2)
            .expect("Failed to verify Ethereum storage proof");
        messages.push(deserialize_ethereum_proof_value_as_string(
            ethereum_proof.1.value,
        ));
        ethereum_proof
            .0
            .verify(&inputs.temporary_debug_state_root)
            .expect("Failed to verify Ethereum account proof");
    }

    // verify the neutron storage proofs
    for neutron_proof in inputs.neutron_storage_proofs {
        // verify the proof against the neutron root
        neutron_proof
            .verify(&neutron_app_hash)
            .expect("Failed to verify Neutron storage proof");
        messages.push(deserialize_neutron_proof_value_as_string(
            neutron_proof.value,
        ));
    }
    let output = MailboxApplicationCircuitOutputs { messages };

    sp1_zkvm::io::commit_slice(&borsh::to_vec(&output).unwrap());
}
