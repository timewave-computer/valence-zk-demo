use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use neutron_sdk::bindings::msg::NeutronMsg;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{BALANCES, SHARES},
};
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    let addr = deps.api.addr_validate(&msg.initial_address)?;
    BALANCES.save(deps.storage, addr, &msg.initial_balance)?;
    SHARES.save(deps.storage, &msg.initial_shares)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("initial_balance", msg.initial_balance)
        .add_attribute("contract_address", env.contract.address)
        .add_attribute("sender", info.sender.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::SetBalance { address, value } => {
            execute_insert_balance(deps, info, address, value)
        }
        ExecuteMsg::SetShares { value } => execute_set_shares(deps, info, value),
    }
}

pub fn execute_insert_balance(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    value: Uint128,
) -> Result<Response<NeutronMsg>, ContractError> {
    let addr = deps.api.addr_validate(&address)?;
    BALANCES.save(deps.storage, addr, &value)?;
    Ok(Response::default()
        .add_attribute("action", "insert_balance")
        .add_attribute("sender", info.sender))
}

pub fn execute_set_shares(
    deps: DepsMut,
    info: MessageInfo,
    value: Uint128,
) -> Result<Response<NeutronMsg>, ContractError> {
    SHARES.save(deps.storage, &value)?;
    Ok(Response::default()
        .add_attribute("action", "set_shares")
        .add_attribute("sender", info.sender))
}

#[cw_serde]
pub struct CurrentValueResponse {
    pub current_value: Uint128,
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Account { address } => query_current_value(deps, address),
        QueryMsg::Shares {} => query_current_shares(deps),
    }
}

pub fn query_current_value(deps: Deps, address: String) -> StdResult<Binary> {
    let addr = deps.api.addr_validate(&address)?;
    let current_value = &BALANCES.load(deps.storage, addr)?;
    to_json_binary(&CurrentValueResponse {
        current_value: current_value.clone(),
    })
}

pub fn query_current_shares(deps: Deps) -> StdResult<Binary> {
    let current_value = &SHARES.load(deps.storage)?;
    to_json_binary(&CurrentValueResponse {
        current_value: current_value.clone(),
    })
}
