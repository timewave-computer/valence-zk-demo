use anyhow::Result;
use helios_consensus_core::consensus_spec::MainnetConsensusSpec;
use helios_ethereum::consensus::Inner;
use helios_ethereum::rpc::ConsensusRpc;
use helios_ethereum::rpc::http_rpc::HttpRpc;
use helios_operator::{get_checkpoint, get_client, get_updates};
use sp1_helios_primitives::types::ProofInputs;
use sp1_sdk::{
    EnvProver, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
    SP1VerifyingKey,
};

pub const ELF: &[u8] = include_bytes!("../../../elfs/sp1-helios-elf");

pub struct SP1HeliosOperator {
    pub client: EnvProver,
    pub pk: SP1ProvingKey,
    pub vk: SP1VerifyingKey,
}

impl SP1HeliosOperator {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        let client = ProverClient::from_env();
        let (pk, vk) = client.setup(ELF);
        Self { client, pk, vk }
    }

    /// Fetch values and generate an 'update' proof for the SP1 Helios contract.
    async fn request_update(
        &self,
        client: Inner<MainnetConsensusSpec, HttpRpc>,
    ) -> Result<Option<SP1ProofWithPublicValues>> {
        let mut stdin = SP1Stdin::new();
        let updates = get_updates(&client).await;
        let finality_update = client.rpc.get_finality_update().await.unwrap();
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
        Ok(Some(proof))
    }

    /// Start the operator.
    pub async fn run(&mut self, slot: u64) -> Result<Option<SP1ProofWithPublicValues>> {
        let slot: u64 = slot;
        let checkpoint = get_checkpoint(slot).await.unwrap();
        // Get the client from the checkpoint
        let client = get_client(checkpoint).await.unwrap();
        // Request an update
        self.request_update(client).await
    }

    pub fn get_vk(&self) -> String {
        self.vk.bytes32()
    }
}

#[cfg(test)]
mod test {
    use super::SP1HeliosOperator;
    use sp1_sdk::HashableKey;
    use sp1_verifier::Groth16Verifier;
    use std::time::Instant;
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_helios_prover() {
        let start_time = Instant::now();
        dotenvy::dotenv().ok();
        let mut operator = SP1HeliosOperator::new();
        // for testing we hardcode the latest finalized slot from /eth/v1/beacon/states/finalized/finality_checkpoints
        let proof = operator
            .run(234644 * 32)
            .await
            .expect("Failed to prove!")
            .unwrap();

        // verify the light client proof
        let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
        Groth16Verifier::verify(
            &proof.bytes(),
            &proof.public_values.to_vec(),
            &operator.vk.bytes32(),
            groth16_vk,
        )
        .unwrap();

        let end_time = Instant::now();
        println!("Time taken: {:?}", end_time.duration_since(start_time));
    }
}
