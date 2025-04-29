use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues};
use tendermint_operator::{TendermintProver, util::TendermintRPCClient};

pub const TENDERMINT_ELF: &[u8] = include_bytes!("../../../elfs/tendermint-elf");

pub struct SP1TendermintOperator {
    pub trusted_height: u64,
    pub target_height: u64,
}

impl SP1TendermintOperator {
    pub fn new(trusted_height: u64, target_height: u64) -> Self {
        Self {
            trusted_height,
            target_height,
        }
    }
    pub async fn run(&self) -> SP1ProofWithPublicValues {
        // Instantiate a Tendermint prover based on the environment variable.
        let tendermint_rpc_client = TendermintRPCClient::default();
        let prover = TendermintProver::new();
        let trusted_block_height: u64 = self.trusted_height;
        //let chain_latest_block_height = tendermint_rpc_client.get_latest_block_height().await;
        let (trusted_light_block, target_light_block) = tendermint_rpc_client
            .get_light_blocks(trusted_block_height, self.target_height)
            .await;
        // Generate a proof of the transition from the trusted block to the target block.
        prover.generate_tendermint_proof(&trusted_light_block, &target_light_block)
    }

    pub fn get_vk(&self) -> String {
        let client = ProverClient::from_env();
        let (_, vk) = client.setup(TENDERMINT_ELF);
        vk.bytes32().to_string()
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use sp1_sdk::{HashableKey, ProverClient};
    use sp1_verifier::Groth16Verifier;
    use tendermint_program_types::TendermintOutput;

    use crate::{
        clients::{ClientInterface, DefaultClient, EthereumClient, NeutronClient},
        lightclients::tendermint::{SP1TendermintOperator, TENDERMINT_ELF},
        read_ethereum_rpc_url, read_neutron_rpc_url,
    };

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_tendermint_prover_and_verifier() {
        let start_time = Instant::now();
        dotenvy::dotenv().ok();
        let default_client = DefaultClient {
            neutron_client: NeutronClient {
                rpc_url: read_neutron_rpc_url(),
            },
            ethereum_client: EthereumClient {
                rpc_url: read_ethereum_rpc_url(),
            },
        };
        let target_block_height: u64 = default_client
            .neutron_client
            .get_latest_root_and_height()
            .await
            .1;
        let trusted_block_height: u64 = target_block_height - 10;
        let operator = SP1TendermintOperator::new(trusted_block_height, target_block_height);
        let proof = operator.run().await;

        // verify the light client proof
        let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
        let client = ProverClient::from_env();
        let (_, vk) = client.setup(TENDERMINT_ELF);
        Groth16Verifier::verify(
            &proof.bytes(),
            &proof.public_values.to_vec(),
            &vk.bytes32(),
            groth16_vk,
        )
        .unwrap();

        let proof_out: TendermintOutput =
            serde_json::from_slice(&proof.public_values.to_vec()).unwrap();
        println!("proof_out: {:?}", proof_out);
        let end_time = Instant::now();
        println!("Time taken: {:?}", end_time.duration_since(start_time));
    }
}
