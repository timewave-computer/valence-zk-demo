use std::time::Instant;

use alloy::{
    dyn_abi::SolType,
    signers::k256::sha2::{Digest, Sha256},
    sol, transports::http::reqwest,
};
use coprocessor_circuit_types::CoprocessorCircuitInputs;
use itertools::Itertools;
use sp1_helios_primitives::types::ProofOutputs;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use sp1_verifier::Groth16Verifier;
use tendermint_program_types::TendermintOutput;


#[derive(Debug, Clone, Deserialize)]
struct SignedBeaconBlockHeader {
    message: BeaconBlockHeader,
    signature: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BeaconHeaderSummary {
    root: String,
    canonical: bool,
    header: SignedBeaconBlockHeader,
}

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlockHeader {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String,    // Hex-encoded 32 bytes (0x-prefixed)
    pub state_root: String,     // Hex-encoded 32 bytes (0x-prefixed)
    pub body_root: String,      // Hex-encoded 32 bytes (0x-prefixed)
}

use crate::{
    COPROCESSOR_CIRCUIT_ELF,
    clients::{DefaultClient, EthereumClient, NeutronClient},
    coprocessor::Coprocessor,
    lightclients::{helios::SP1HeliosOperator, tendermint::SP1TendermintOperator},
    read_ethereum_rpc_url, read_neutron_rpc_url,
};

#[cfg(feature = "mailbox")]
pub mod mailbox;
#[cfg(feature = "rate")]
pub mod rate;

pub async fn prove_coprocessor(coprocessor: &mut Coprocessor) -> (TendermintOutput, ProofOutputs) {
    let start_time = Instant::now();
    // todo: set the trusted values for Ethereum
    let neutron_operator = SP1TendermintOperator::new(
        coprocessor.trusted_neutron_height,
        coprocessor.target_neutron_height,
    );
    let neutron_light_client_proof = neutron_operator.run().await;
    let mut ethereum_operator = SP1HeliosOperator::new();
    // todo: remove hardcoded ethereum height and replace it with a real trusted height
    let ethereum_light_client_proof = ethereum_operator.run(234644 * 32).await;

    let neutron_proof_serialized = neutron_light_client_proof.bytes();
    let neutron_public_values = neutron_light_client_proof.public_values.to_vec();
    let neutron_vk = neutron_operator.get_vk();

    let ethereum_light_client_proof = ethereum_light_client_proof.unwrap().unwrap();
    let helios_proof_serialized = ethereum_light_client_proof.bytes();
    let helios_public_values = ethereum_light_client_proof.public_values.to_vec();
    let helios_vk = ethereum_operator.get_vk();

    let neutron_output: TendermintOutput =
        serde_json::from_slice(&neutron_light_client_proof.public_values.to_vec()).unwrap();
    let helios_output: ProofOutputs =
        ProofOutputs::abi_decode(&ethereum_light_client_proof.public_values.to_vec(), false)
            .unwrap();

    let target_neutron_root: Vec<u8> = neutron_output.target_header_hash.to_vec();
    let target_ethereum_root: Vec<u8> = helios_output.newHeader.to_vec();
    let target_neutron_height: u64 = neutron_output.target_height;
    let target_ethereum_height: u64 = helios_output.newHead.try_into().unwrap();

    let mut coprocessor_root = coprocessor.smt_root;
    let mut hasher = Sha256::new();
    hasher.update(&target_neutron_height.to_be_bytes());
    let neutron_height_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&target_ethereum_height.to_be_bytes());
    let ethereum_height_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&target_neutron_root);
    let neutron_root_key = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&target_ethereum_root);
    let ethereum_root_key = hasher.finalize();

    coprocessor_root = coprocessor
        .smt_tree
        .insert(
            coprocessor_root,
            "demo",
            &neutron_height_key,
            target_neutron_height.to_be_bytes().to_vec(),
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
            &neutron_root_key,
            target_neutron_root.clone(),
        )
        .expect("Failed to insert Ethereum Root");

    coprocessor_root = coprocessor
        .smt_tree
        .insert(
            coprocessor_root,
            "demo",
            &ethereum_root_key,
            target_ethereum_root.clone(),
        )
        .expect("Failed to insert Ethereum Root");

    let neutron_height_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_root, &neutron_height_key)
        .unwrap()
        .unwrap();
    let ethereum_height_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_root, &ethereum_height_key)
        .unwrap()
        .unwrap();
    let neutron_root_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_root, &neutron_root_key)
        .unwrap()
        .unwrap();
    let ethereum_root_opening = coprocessor
        .smt_tree
        .get_opening("demo", coprocessor_root, &ethereum_root_key)
        .unwrap()
        .unwrap();

    coprocessor.trusted_neutron_height = neutron_output.target_height;
    coprocessor.trusted_ethereum_height = helios_output.prevHead.try_into().unwrap();
    coprocessor.trusted_neutron_root = neutron_output.target_header_hash.to_vec();
    coprocessor.trusted_ethereum_root = helios_output.prevHeader.to_vec();

    let coprocessor_inputs = CoprocessorCircuitInputs {
        helios_proof: helios_proof_serialized,
        helios_public_values,
        helios_vk,
        neutron_proof: neutron_proof_serialized,
        neutron_public_values,
        neutron_vk,
        previous_neutron_height: coprocessor.trusted_neutron_height,
        previous_ethereum_height: coprocessor.trusted_ethereum_height,
        previous_neutron_root: coprocessor.trusted_neutron_root.to_vec(),
        previous_ethereum_root: coprocessor.trusted_ethereum_root.to_vec(),
        neutron_height_opening,
        ethereum_height_opening,
        neutron_root_opening,
        ethereum_root_opening,
        coprocessor_root,
    };

    let coprocessor_circuit_inputs_serialized = borsh::to_vec(&coprocessor_inputs).unwrap();
    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(coprocessor_circuit_inputs_serialized);
    let (pk, vk) = client.setup(COPROCESSOR_CIRCUIT_ELF);
    // this is the coprocessor update proof
    // that contains the new roots from the zk light clients
    // all future proofs can be verified against the state in this smt
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

    let default_client = DefaultClient {
        neutron_client: NeutronClient {
            rpc_url: read_neutron_rpc_url(),
        },
        ethereum_client: EthereumClient {
            rpc_url: read_ethereum_rpc_url(),
        },
    };
    // todo: move this to the app circuit
    let tendermint_header = default_client
        .neutron_client
        .get_header_at_height(target_neutron_height)
        .await;
    //let tendermint_header_app_hash = tendermint_header.app_hash.clone();
    let tendermint_header_hash = tendermint_header.hash();
    assert_eq!(tendermint_header_hash.as_bytes(), target_neutron_root);
    // return new state (or update on-chain)
    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));

    let target_beaecon_header = get_beacon_block_header(234644 * 32).await;
    println!("Target Beacon Header: {:?}", target_beaecon_header);

    // todo: move this into the app circuit
    let finalized_header_root = merkleize_keys(vec![
        uint64_to_le_256(target_beaecon_header.slot.parse::<u64>().unwrap()),
        uint64_to_le_256(target_beaecon_header.proposer_index.parse::<u64>().unwrap()),
        alloy::hex::decode(target_beaecon_header.parent_root).unwrap().to_vec(),
        alloy::hex::decode(target_beaecon_header.state_root).unwrap().to_vec(),
        alloy::hex::decode(target_beaecon_header.body_root).unwrap().to_vec(),
    ]);

    assert_eq!(finalized_header_root, target_ethereum_root);

    // next step: store the state root directly in the smt instead of the header root

    (neutron_output, helios_output)
}

pub async fn get_beacon_block_header(slot: u64) -> BeaconBlockHeader {
    let client = reqwest::Client::new();
    let url = format!("{}/eth/v1/beacon/headers/{}", "https://lodestar-sepolia.chainsafe.io", slot);

    let resp = client
        .get(&url)
        .send()
        .await.unwrap()
        .error_for_status().unwrap() // fail if not 200 OK
        .json::<serde_json::Value>() // API returns wrapped {"data": {...}}
        .await.unwrap();

    // Unwrap the structure manually
    let summary: BeaconHeaderSummary = serde_json::from_value(resp["data"].clone()).unwrap();
    summary.header.message
}

// helper function to hash a bunch of nodes as ssz
// yes, Ethereum do be annoying sometimes :D
pub fn merkleize_keys(mut keys: Vec<Vec<u8>>) -> Vec<u8> {
    let height = if keys.len() == 1 {
        1
    } else {
        keys.len().next_power_of_two().ilog2() as usize
    };

    for depth in 0..height {
        let len_even: usize = keys.len() + keys.len() % 2;
        let padded_keys = keys
            .into_iter()
            .pad_using(len_even, |_| ZERO_HASHES[depth].as_slice().to_vec())
            .collect_vec();
        keys = padded_keys
            .into_iter()
            .tuples()
            .map(|(left, right)| compute_digest(&add_left_right(left, &right)))
            .collect::<Vec<Vec<u8>>>();
    }
    keys.pop().unwrap()
}

fn add_left_right(left: Vec<u8>, right: &Vec<u8>) -> Vec<u8> {
    let mut value: Vec<u8> = left;
    value.extend_from_slice(&right);
    value.to_vec()
}

fn compute_digest(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

lazy_static::lazy_static! {
    pub static ref ZERO_HASHES: [[u8; 32]; 2] = {
        std::iter::successors(Some([0; 32]), |&prev| {
            Some(compute_digest(&[prev, prev].concat()).try_into().unwrap())
        })
        .take(2)
        .collect_vec()
        .try_into()
        .unwrap()
    };
}

fn uint64_to_le_256(value: u64) -> Vec<u8> {
    let mut bytes = value.to_le_bytes().to_vec(); // Convert to little-endian 8 bytes
    bytes.extend(vec![0u8; 24]); // Pad with 24 zeros to make it 32 bytes
    bytes
}