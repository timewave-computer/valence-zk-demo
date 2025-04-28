use alloy::dyn_abi::SolType;
use sp1_helios_primitives::types::ProofOutputs;
use tendermint_program_types::TendermintOutput;

use crate::{
    coprocessor::Coprocessor,
    lightclients::{helios::SP1HeliosOperator, tendermint::SP1TendermintOperator},
};

#[cfg(feature = "mailbox")]
pub mod mailbox;
#[cfg(feature = "rate")]
pub mod rate;

pub async fn prove_coprocessor(coprocessor: &mut Coprocessor) {
    // todo: set the trusted values for Ethereum
    let tendermint_operator = SP1TendermintOperator::new(
        coprocessor.trusted_neutron_height,
        coprocessor.target_neutron_height,
    );
    let tendermint_light_client_proof = tendermint_operator.run().await;
    let tendermint_output: TendermintOutput =
        serde_json::from_slice(&tendermint_light_client_proof.public_values.to_vec()).unwrap();
    let mut ethereum_operator = SP1HeliosOperator::new();
    // todo: remove hardcoded ethereum height and replace it with a real trusted height
    let ethereum_light_client_proof = ethereum_operator.run(234644 * 32).await;
    let helios_output: ProofOutputs = ProofOutputs::abi_decode(
        &ethereum_light_client_proof
            .unwrap()
            .unwrap()
            .public_values
            .to_vec(),
        false,
    )
    .unwrap();
}
