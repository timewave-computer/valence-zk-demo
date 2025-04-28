// Messaeg Demo with ZK Light Client proof verification
/* Todo
    1. Define some initial state:
    - Neutron Root
    - Neutron Height
    - Ethereum Height


    2. Request an update from each light client:
    - Neutron Proof, State
    - EthereumProof, State

    3. Query for state proofs at the output heights
    - Insert that state into the SMT
    - Insert the light client updates into the SMT

    4. Run the coprocessor circuit to prove this new state
    - Must commit to the keys that were verified!
*/

use alloy::dyn_abi::SolType;
use anyhow::Result;
use helios_consensus_core::consensus_spec::MainnetConsensusSpec;
use helios_ethereum::consensus::Inner;
use helios_ethereum::rpc::ConsensusRpc;
use helios_ethereum::rpc::http_rpc::HttpRpc;
use helios_operator::{get_checkpoint, get_client, get_updates};
use sp1_helios_primitives::types::{ProofInputs, ProofOutputs};
use sp1_sdk::{EnvProver, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin};

const ELF: &[u8] = include_bytes!("../../../elfs/sp1-helios-elf");

struct SP1HeliosOperator {
    client: EnvProver,
    pk: SP1ProvingKey,
}

impl SP1HeliosOperator {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let client = ProverClient::from_env();
        let (pk, _) = client.setup(ELF);

        Self { client, pk }
    }

    /// Fetch values and generate an 'update' proof for the SP1 Helios contract.
    async fn request_update(
        &self,
        client: Inner<MainnetConsensusSpec, HttpRpc>,
    ) -> Result<Option<SP1ProofWithPublicValues>> {
        let mut stdin = SP1Stdin::new();
        let updates = get_updates(&client).await;
        println!("About to prove {:?} light client updates!", updates.len());
        let finality_update = client.rpc.get_finality_update().await.unwrap();
        // Check if contract is up to date
        let latest_block = finality_update.finalized_header().beacon().slot;
        println!("Latest block: {:?}", latest_block);
        // Create program inputs
        let expected_current_slot = client.expected_current_slot();
        let inputs = ProofInputs {
            updates,
            finality_update,
            expected_current_slot,
            store: client.store.clone(),
            genesis_root: client.config.chain.genesis_root,
            forks: client.config.forks.clone(),
        };
        let encoded_proof_inputs = serde_cbor::to_vec(&inputs)?;
        stdin.write_slice(&encoded_proof_inputs);
        // Generate proof.
        let proof = self.client.prove(&self.pk, &stdin).groth16().run()?;
        let outputs = ProofOutputs::abi_decode(proof.public_values.as_slice(), false).unwrap();
        println!("Proof outputs: {:?}", outputs);
        Ok(Some(proof))
    }

    /// Start the operator.
    async fn run(&mut self) {
        // slot multiple of 8192
        let slot: u64 = 11558912;
        let checkpoint = get_checkpoint(slot).await.unwrap();
        // Get the client from the checkpoint
        let client = get_client(checkpoint).await.unwrap();

        // Request an update
        self.request_update(client).await.unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::SP1HeliosOperator;
    use std::time::Instant;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_helios_prover() {
        let start_time = Instant::now();
        dotenvy::dotenv().ok();
        let mut operator = SP1HeliosOperator::new().await;
        operator.run().await;
        let end_time = Instant::now();
        println!("Time taken: {:?}", end_time.duration_since(start_time));
    }
}
