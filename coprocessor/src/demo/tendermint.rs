#[cfg(test)]
mod test {
    use std::time::Instant;
    use tendermint_operator::{TendermintProver, util::TendermintRPCClient};
    use tendermint_program_types::TendermintOutput;

    use crate::{
        clients::{ClientInterface, DefaultClient, EthereumClient, NeutronClient},
        read_ethereum_rpc_url, read_neutron_rpc_url,
    };

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_tendermint_prover() {
        let start_time = Instant::now();
        dotenvy::dotenv().ok();

        let mock_light_client = DefaultClient {
            neutron_client: NeutronClient {
                rpc_url: read_neutron_rpc_url(),
            },
            ethereum_client: EthereumClient {
                rpc_url: read_ethereum_rpc_url(),
            },
        };

        // Instantiate a Tendermint prover based on the environment variable.
        let tendermint_rpc_client = TendermintRPCClient::default();
        let prover = TendermintProver::new();
        let target_block_height: u64 = mock_light_client
            .neutron_client
            .get_latest_root_and_height()
            .await
            .1;
        let trusted_block_height: u64 = target_block_height - 10;
        if trusted_block_height == 0 {
            panic!(
                "No trusted height found on the contract. Something is wrong with the contract."
            );
        }

        //let chain_latest_block_height = tendermint_rpc_client.get_latest_block_height().await;
        let (trusted_light_block, target_light_block) = tendermint_rpc_client
            .get_light_blocks(trusted_block_height, target_block_height)
            .await;
        // Generate a proof of the transition from the trusted block to the target block.
        let proof_data =
            prover.generate_tendermint_proof(&trusted_light_block, &target_light_block);
        let proof_out: TendermintOutput =
            serde_json::from_slice(&proof_data.public_values.to_vec()).unwrap();

        println!("proof_out: {:?}", proof_out);
        let end_time = Instant::now();
        println!("Time taken: {:?}", end_time.duration_since(start_time));
    }
}
