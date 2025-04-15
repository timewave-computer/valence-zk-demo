use alloy_primitives::U256;
use borsh::{BorshDeserialize, BorshSerialize};
use ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof;
use ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof;
use valence_smt::SmtOpening;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct RateApplicationCircuitInputs {
    pub neutron_vault_balance_opening: SmtOpening,
    pub neutron_vault_shares_opening: SmtOpening,
    pub ethereum_vault_balance_opening: SmtOpening,
    pub ethereum_vault_shares_opening: SmtOpening,
    pub coprocessor_root: [u8; 32],
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RateApplicationCircuitOutputs {
    pub rate: u64,
}

pub fn deserialize_ethereum_proof_value_as_U256(proof: SmtOpening) -> U256 {
    let ethereum_proof: EthereumMerkleProof = borsh::from_slice(&proof.data).unwrap();
    let ethereum_proof_value = ethereum_proof.value;
    let ethereum_proof_value_u256 = U256::from_be_slice(&ethereum_proof_value);
    ethereum_proof_value_u256
}

pub fn deserialize_neutron_proof_value_as_U256(proof: SmtOpening) -> U256 {
    let neutron_proof: Ics23MerkleProof = borsh::from_slice(&proof.data).unwrap();
    let neutron_proof_value = neutron_proof.value;
    let neutron_value_decoded = &String::from_utf8_lossy(&neutron_proof_value);
    U256::from_str_radix(serde_json::from_str(neutron_value_decoded).unwrap(), 16).unwrap()
}

/*let neutron_balance = neutron_vault_balance_proof.value.clone();
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
println!("Ethereum Balance: {}", ethereum_balance_u256);*/
