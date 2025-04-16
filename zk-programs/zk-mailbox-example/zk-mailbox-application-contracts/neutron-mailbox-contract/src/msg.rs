use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_message: String,
    pub initial_counter: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    NewMessage { message: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    Message { counter: Uint128 },
    #[returns(Uint128)]
    Counter {},
}
