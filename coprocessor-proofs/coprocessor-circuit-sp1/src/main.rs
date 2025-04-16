/*
    This circuit verifies merkle proofs from different domains and builds a SMT from them.
    The SMT trie root is committed as a public output.
    The new trie root, alongslide with the proof can be sent to the different domains.
*/
#![no_main]
use borsh;
use coprocessor_circuit_logic::coprocessor_logic;
use types::MerkleProofInputs;
sp1_zkvm::entrypoint!(main);
pub fn main() {
    let inputs: MerkleProofInputs = borsh::from_slice(&sp1_zkvm::io::read_vec())
        .expect("Failed to deserialize MerkleProofInputs");

    let circuit_outputs = coprocessor_logic(inputs);
    sp1_zkvm::io::commit_slice(&circuit_outputs);
}
