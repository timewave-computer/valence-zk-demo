use std::{env, str::FromStr};

use alloy::{
    hex::{self, FromHex},
    providers::{Provider, ProviderBuilder},
    sol_types::SolValue,
};
use alloy_primitives::U256;
use base64::Engine;
use common_merkle_proofs::merkle::types::MerkleClient;
use coprocessor_circuit_types::{
    MerkleProofInputs as CoprocessorCircuitInputs, MerkleProofOutputs as CoprocessorCircuitOutputs,
};
use dotenvy::dotenv;
use ethereum_merkle_proofs::{
    ethereum_rpc::rpc::EvmMerkleRpcClient,
    merkle_lib::{
        keccak::digest_keccak,
        types::{EthereumMerkleProof, decode_rlp_bytes},
    },
};
use ics23_merkle_proofs::{
    keys::Ics23Key, merkle_lib::types::Ics23MerkleProof, rpc::Ics23MerkleRpcClient,
};
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin, include_elf};
use sp1_verifier::Groth16Verifier;
use url::Url;
use valence_smt::MemorySmt;
pub const COPROCESSOR_CIRCUIT_ELF: &[u8] = include_elf!("coprocessor-circuit-sp1");

#[tokio::main]
async fn main() {
    let mut smt_tree = MemorySmt::default();
    let mut root = [0; 32];
    let mut ethereum_merkle_proofs: Vec<(EthereumMerkleProof, EthereumMerkleProof, Vec<u8>)> =
        Vec::new();
    let mut neutron_merkle_proofs: Vec<Ics23MerkleProof> = Vec::new();

    let neutron_rpc_url = read_neutron_rpc_url();
    let neutron_rpc_client = Ics23MerkleRpcClient {
        rpc_url: neutron_rpc_url,
    };
    let neutron_height = read_neutron_height();
    let neutron_vault_balance_key = Ics23Key::new_wasm_account_mapping(
        b"balances",
        &read_neutron_default_account_address(),
        &read_neutron_vault_example_address(),
    );
    let proof = neutron_rpc_client
        .get_proof(&neutron_vault_balance_key.to_string(), "", neutron_height)
        .await
        .unwrap();
    let neutron_vault_balance_proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
    neutron_merkle_proofs.push(neutron_vault_balance_proof.clone());

    let neutron_vault_shares_key =
        Ics23Key::new_wasm_stored_value("shares", &read_neutron_vault_example_address());
    let proof = neutron_rpc_client
        .get_proof(&neutron_vault_shares_key.to_string(), "", neutron_height)
        .await
        .unwrap();
    let neutron_vault_shares_proof: Ics23MerkleProof = serde_json::from_slice(&proof).unwrap();
    neutron_merkle_proofs.push(neutron_vault_shares_proof.clone());

    let ethereum_rpc_url = read_ethereum_rpc_url();
    let ethereum_rpc_client = EvmMerkleRpcClient {
        rpc_url: ethereum_rpc_url,
    };
    let ethereum_height = get_ethereum_height().await;

    let address =
        alloy_primitives::Address::from_hex(read_ethereum_default_account_address()).unwrap();
    let slot: U256 = alloy_primitives::U256::from(0);
    let encoded_key = (address, slot).abi_encode();
    let keccak_key = digest_keccak(&encoded_key).to_vec();
    let (ethereum_vault_shares_account_proof, ethereum_vault_shares_storage_proof) =
        ethereum_rpc_client
            .get_account_and_storage_proof(
                &alloy::hex::encode(&keccak_key),
                &read_ethereum_vault_example_address(),
                ethereum_height,
            )
            .await
            .unwrap();
    let account_decoded = decode_rlp_bytes(&ethereum_vault_shares_account_proof.value).unwrap();

    ethereum_merkle_proofs.push((
        ethereum_vault_shares_account_proof.clone(),
        ethereum_vault_shares_storage_proof.clone(),
        // this is the account hash
        account_decoded.get(2).unwrap().to_vec(),
    ));

    let ethereum_balance_shares_key =
        hex::decode(read_ethereum_vault_balances_storage_key()).unwrap();
    let (ethereum_vault_balance_account_proof, ethereum_vault_balance_storage_proof) =
        ethereum_rpc_client
            .get_account_and_storage_proof(
                &alloy::hex::encode(&ethereum_balance_shares_key),
                &read_ethereum_vault_example_address(),
                ethereum_height,
            )
            .await
            .unwrap();
    let account_decoded = decode_rlp_bytes(&ethereum_vault_balance_account_proof.value).unwrap();
    ethereum_merkle_proofs.push((
        ethereum_vault_balance_account_proof.clone(),
        ethereum_vault_balance_storage_proof.clone(),
        // this is the account hash
        account_decoded.get(2).unwrap().to_vec(),
    ));

    let provider = ProviderBuilder::new().on_http(Url::from_str(&read_ethereum_rpc_url()).unwrap());

    let neutron_root = base64::engine::general_purpose::STANDARD
        .decode(read_neutron_app_hash())
        .unwrap();

    let ethereum_root = provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(ethereum_height))
        .await
        .unwrap()
        .unwrap()
        .header
        .state_root
        .to_vec();

    // build the same SMT outside the circuit - for the demo we do everything
    // in-memory.
    for ethereum_proof in ethereum_merkle_proofs.clone() {
        root = smt_tree
            .insert(root, "demo", borsh::to_vec(&ethereum_proof.0).unwrap())
            .expect("Failed to insert");
        root = smt_tree
            .insert(root, "demo", borsh::to_vec(&ethereum_proof.1).unwrap())
            .expect("Failed to insert");
    }
    for neutron_proof in neutron_merkle_proofs.clone() {
        root = smt_tree
            .insert(root, "demo", borsh::to_vec(&neutron_proof).unwrap())
            .expect("Failed to insert");
    }

    // run the coprocessor update circuit
    // note that this circuit is not yet complete, and for the time being only
    // verifies the merkle proofs against the trusted domain roots
    // later it must do the following:
    /*
    it must have a light client proof and the light client root must
    be at the respective slot in the SMT tree;
    if leaf is not lightclient root then leaf is verified
    against light client root;
    and light client root itself must be a leaf in the tree at a deterministic key;
    and if leaf is light client root it is verified via a light client proof against the previous light client root
    */

    let coprocessor_circuit_inputs = CoprocessorCircuitInputs {
        ethereum_merkle_proofs: ethereum_merkle_proofs.clone(),
        neutron_merkle_proofs: neutron_merkle_proofs.clone(),
        neutron_root,
        ethereum_root,
    };
    let coprocessor_circuit_inputs_serialized = borsh::to_vec(&coprocessor_circuit_inputs).unwrap();
    let client = ProverClient::new();
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(coprocessor_circuit_inputs_serialized);
    let (pk, vk) = client.setup(COPROCESSOR_CIRCUIT_ELF);
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
    let coprocessor_circuit_outputs: CoprocessorCircuitOutputs =
        borsh::from_slice(proof.public_values.as_slice()).unwrap();
    println!(
        "Coprocessor Circuit Outputs: {:?}",
        coprocessor_circuit_outputs
    );

    // prepare the inputs for the first example application
    // todo: move this into the example application circuit
    let neutron_balance = neutron_vault_balance_proof.value.clone();
    let neutron_balance_decoded = &String::from_utf8_lossy(&neutron_balance);
    let neutron_balance_u256 =
        U256::from_str(serde_json::from_str(neutron_balance_decoded).unwrap()).unwrap();
    let neutron_shares = neutron_vault_shares_proof.value.clone();
    let neutron_shares_decoded = &String::from_utf8_lossy(&neutron_shares);
    let neutron_shares_u256 =
        U256::from_str(serde_json::from_str(neutron_shares_decoded).unwrap()).unwrap();
    println!("Neutron Balance: {}", neutron_balance_u256);
    println!("Neutron Shares: {}", neutron_shares_u256);

    let ethereum_shares = ethereum_vault_shares_storage_proof.value.clone();
    let ethereum_balance = ethereum_vault_balance_storage_proof.value.clone();
    let ethereum_shares_u256 = U256::from_be_slice(&ethereum_shares);
    let ethereum_balance_u256 = U256::from_be_slice(&ethereum_balance);
    println!("Ethereum Shares: {}", ethereum_shares_u256);
    println!("Ethereum Balance: {}", ethereum_balance_u256);
    ////////////////////////////////

    // get the SMT openings that will be part of the input for our example application
    let neutron_balance_smt_opening = smt_tree
        .get_opening(
            "demo",
            root,
            &borsh::to_vec(&neutron_vault_balance_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get neutron balance opening");

    let neutron_shares_smt_opening = smt_tree
        .get_opening(
            "demo",
            root,
            &borsh::to_vec(&neutron_vault_shares_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get neutron shares opening");

    let ethereum_shares_smt_opening = smt_tree
        .get_opening(
            "demo",
            root,
            &borsh::to_vec(&ethereum_vault_shares_storage_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get ethereum shares opening");

    let ethereum_balance_smt_opening = smt_tree
        .get_opening(
            "demo",
            root,
            &borsh::to_vec(&ethereum_vault_balance_storage_proof).unwrap(),
        )
        .unwrap()
        .expect("Failed to get ethereum balance opening");
}

// Neutron Data
pub(crate) fn read_neutron_rpc_url() -> String {
    dotenvy::dotenv().ok();
    env::var("NEUTRON_RPC").expect("Missing Neutron RPC url!")
}

pub(crate) fn read_neutron_height() -> u64 {
    dotenv().ok();
    env::var("HEIGHT_NEUTRON")
        .expect("Missing Neutron TEST VECTOR: HEIGHT!")
        .parse::<u64>()
        .expect("Failed to parse test vector as u64: Amount")
}

pub(crate) fn read_neutron_app_hash() -> String {
    dotenv().ok();
    env::var("MERKLE_ROOT_NEUTRON").expect("Missing Neutron TEST VECTOR: ROOT!")
}

pub(crate) fn read_neutron_vault_example_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_PION_1_VAULT_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Pion 1 Vault Contract Address!")
}

pub(crate) fn read_neutron_default_account_address() -> String {
    dotenv().ok();
    env::var("NEUTRON_DEFAULT_ACCOUNT_ADDRESS").expect("Missing Neutron Default Account Address!")
}

// Ethereum Data
pub(crate) fn read_ethereum_vault_example_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_VAULT_EXAMPLE_CONTRACT_ADDRESS")
        .expect("Missing Sepolia Vault Contract Address!")
}

pub(crate) fn read_ethereum_vault_balances_storage_key() -> String {
    dotenv().ok();
    env::var("ETHEREUM_SEPOLIA_VAULT_BALANCES_STORAGE_KEY")
        .expect("Missing Sepolia Vault Balances Storage Key!")
}

pub(crate) fn read_ethereum_default_account_address() -> String {
    dotenv().ok();
    env::var("ETHEREUM_DEFAULT_ACCOUNT_ADDRESS").expect("Missing Ethereum Default Account Address!")
}

pub(crate) fn read_ethereum_rpc_url() -> String {
    dotenv().ok();
    env::var("ETHEREUM_URL").expect("Missing Sepolia url!")
}

pub(crate) async fn get_ethereum_height() -> u64 {
    use alloy;
    use alloy::providers::{Provider, ProviderBuilder};
    use std::str::FromStr;
    use url::Url;
    let provider = ProviderBuilder::new().on_http(Url::from_str(&read_ethereum_rpc_url()).unwrap());
    let block = provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Latest)
        .await
        .unwrap()
        .expect("Failed to get Block!");
    block.header.number
}
