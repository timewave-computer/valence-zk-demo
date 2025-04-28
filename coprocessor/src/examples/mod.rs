use crate::{
    clients::{ClientInterface, DefaultClient, EthereumClient, NeutronClient},
    coprocessor::Coprocessor,
    read_ethereum_rpc_url, read_neutron_rpc_url,
};

#[cfg(feature = "mailbox")]
pub mod mailbox;
#[cfg(feature = "rate")]
pub mod rate;

pub async fn prove_coprocessor(
    coprocessor: &mut Coprocessor,
    merkle_proofs: (
        Vec<ics23_merkle_proofs::merkle_lib::types::Ics23MerkleProof>,
        Vec<(
            ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof,
            ethereum_merkle_proofs::merkle_lib::types::EthereumMerkleProof,
            Vec<u8>,
        )>,
    ),
    ethereum_root: Vec<u8>,
    neutron_root: Vec<u8>,
) {
    let mut coprocessor = Coprocessor::from_env();
    let default_client = DefaultClient {
        neutron_client: NeutronClient {
            rpc_url: read_neutron_rpc_url(),
        },
        ethereum_client: EthereumClient {
            rpc_url: read_ethereum_rpc_url(),
        },
    };
    let neutron_target_block_height: u64 = default_client
        .neutron_client
        .get_latest_root_and_height()
        .await
        .1;
    // for this example we assume our trusted block height is 10 blocks behind the target height
    let neutron_example_trusted_height: u64 = neutron_target_block_height - 10;
    coprocessor.target_neutron_height = neutron_target_block_height;
    coprocessor.trusted_neutron_height = neutron_example_trusted_height;
    let neutron_trusted_root = default_client
        .neutron_client
        .get_state_at_height(neutron_example_trusted_height)
        .await
        .0;
    coprocessor.trusted_neutron_root = neutron_trusted_root.try_into().unwrap();
    // todo: do the same for ethereum
}
