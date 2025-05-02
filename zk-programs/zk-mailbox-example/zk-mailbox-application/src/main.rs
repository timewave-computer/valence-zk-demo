#![no_main]
use beacon::merkleize_header;
// Important Note! There is a constraint missing in this example!
// We also need to constrain the keys that are being opened on the different domains.
// How exactly we do this might depend on the type of application we are writing.

// For this example you would also want to constrain the address of the smart contract that is used
// for merkle proof verification on Ethereum, the Key that is used in the merkle proof on Ethereum,
// and the storage key on Neutron that is used for the storage proof verification.

use common_merkle_proofs::merkle::types::MerkleVerifiable;
use types::{
    MailboxApplicationCircuitInputs, MailboxApplicationCircuitOutputs,
    deserialize_ethereum_proof_value_as_string, deserialize_neutron_proof_value_as_string,
};
use valence_coprocessor_core::MemorySmt;
sp1_zkvm::entrypoint!(main);

// the fixed keys for the domain roots and heights in the coprocessor SMT
const neutron_height_key: [u8; 32] = [
    5, 92, 226, 28, 182, 227, 244, 206, 139, 106, 219, 203, 86, 167, 223, 128, 79, 231, 159, 227,
    28, 76, 212, 19, 61, 221, 239, 48, 60, 35, 162, 102,
];
const ethereum_height_key: [u8; 32] = [
    225, 27, 47, 17, 45, 96, 202, 66, 172, 66, 54, 240, 184, 154, 153, 9, 185, 64, 83, 168, 31, 33,
    96, 209, 59, 84, 151, 70, 51, 237, 68, 17,
];
const neutron_root_key: [u8; 32] = [
    100, 199, 198, 130, 151, 99, 36, 184, 143, 64, 220, 2, 6, 249, 213, 207, 53, 9, 111, 146, 62,
    7, 251, 165, 129, 136, 106, 115, 4, 154, 4, 226,
];
const ethereum_root_key: [u8; 32] = [
    219, 255, 51, 188, 30, 184, 227, 102, 147, 124, 35, 50, 152, 96, 225, 175, 84, 57, 208, 125,
    236, 134, 108, 17, 77, 195, 169, 130, 177, 237, 235, 53,
];

fn main() {
    let mut messages: Vec<String> = Vec::new();
    let inputs: MailboxApplicationCircuitInputs =
        serde_json::from_slice::<MailboxApplicationCircuitInputs>(&sp1_zkvm::io::read_vec())
            .expect("Failed to deserialize MailboxApplicationCircuitInputs");
    // constrain that the keys for the merkle openings of the domain roots in the coprocessor are correct
    assert_eq!(inputs.neutron_height_opening.key, neutron_height_key);
    assert_eq!(inputs.ethereum_height_opening.key, ethereum_height_key);
    assert_eq!(inputs.neutron_root_opening.key, neutron_root_key);
    assert_eq!(inputs.ethereum_root_opening.key, ethereum_root_key);
    // constrain that the merkle openings of the domain roots and heights against the coprocessor are correct
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
    let electra_block_header_root = merkleize_header(inputs.electra_block_header.clone());
    let electra_body_roots = inputs.electra_body_roots;
    let electra_body_root = electra_body_roots.merkelize();
    // assert that the height of the neutron header is correct
    assert_eq!(
        inputs.neutron_block_header.height.value(),
        u64::from_be_bytes(inputs.neutron_height_opening.data.try_into().unwrap())
    );
    // assert that the height of the electra header is correct
    assert_eq!(
        inputs.electra_block_header.slot,
        u64::from_be_bytes(inputs.ethereum_height_opening.data.try_into().unwrap())
    );
    // verify the block body root against that in the header
    assert_eq!(inputs.electra_block_header.body_root, electra_body_root);
    // verify the header root against the one from the ethereum zk light client in the SMT
    assert_eq!(
        electra_block_header_root.to_vec(),
        inputs.ethereum_root_opening.data
    );
    // verify the neutron app hash against the header root
    assert_eq!(tendermint_header_hash, inputs.neutron_root_opening.data);
    // the neutron app hash against which we verify our storage proofs
    let neutron_app_hash = inputs.neutron_root_opening.data;
    // verify the ethereum storage proofs
    for ethereum_proof in inputs.ethereum_storage_proofs {
        // for each ethereum proof, we first verify the storage proof against the account hash / address
        ethereum_proof
            .1
            .verify(&ethereum_proof.2)
            .expect("Failed to verify Ethereum storage proof");
        messages.push(deserialize_ethereum_proof_value_as_string(
            ethereum_proof.1.value,
        ));
        // then we verify the account proof against the state root
        ethereum_proof
            .0
            .verify(&inputs.electra_body_roots.payload_roots.state_root)
            .expect("Failed to verify Ethereum account proof");
    }
    // verify the neutron storage proofs
    for neutron_proof in inputs.neutron_storage_proofs {
        // verify the storage proof against the neutron root / app hash
        neutron_proof
            .verify(&neutron_app_hash)
            .expect("Failed to verify Neutron storage proof");
        messages.push(deserialize_neutron_proof_value_as_string(
            neutron_proof.value,
        ));
    }
    let output = MailboxApplicationCircuitOutputs {
        messages,
        coprocessor_root: inputs.coprocessor_root,
    };
    sp1_zkvm::io::commit_slice(&borsh::to_vec(&output).unwrap());
}
