use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use neutron_sdk::bindings::msg::NeutronMsg;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{COUNTER, MESSAGES},
};
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    MESSAGES.save(
        deps.storage,
        msg.initial_counter.to_string(),
        &msg.initial_message,
    )?;
    COUNTER.save(deps.storage, &msg.initial_counter)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("initial_message", msg.initial_message)
        .add_attribute("initial_counter", msg.initial_counter)
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
        ExecuteMsg::NewMessage { message } => execute_new_message(deps, info, message),
    }
}

pub fn execute_new_message(
    deps: DepsMut,
    info: MessageInfo,
    message: String,
) -> Result<Response<NeutronMsg>, ContractError> {
    let counter = COUNTER.load(deps.storage)?;
    let new_counter = counter + Uint128::from(1u32);
    COUNTER.save(deps.storage, &new_counter)?;
    MESSAGES.save(deps.storage, new_counter.to_string(), &message)?;
    Ok(Response::default()
        .add_attribute("action", "send_message")
        .add_attribute("sender", info.sender))
}

pub fn execute_set_shares(
    deps: DepsMut,
    info: MessageInfo,
    value: Uint128,
) -> Result<Response<NeutronMsg>, ContractError> {
    COUNTER.save(deps.storage, &value)?;
    Ok(Response::default()
        .add_attribute("action", "set_shares")
        .add_attribute("sender", info.sender))
}

#[cw_serde]
pub struct CurrentValueResponse {
    pub current_value: Uint128,
}

#[cw_serde]
pub struct MessageResponse {
    pub message: String,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Message { counter } => query_message(deps, counter),
        QueryMsg::Counter {} => query_counter(deps),
    }
}

pub fn query_message(deps: Deps, counter: Uint128) -> StdResult<Binary> {
    let message = &MESSAGES.load(deps.storage, counter.to_string())?;
    to_json_binary(&MessageResponse {
        message: message.clone(),
    })
}

pub fn query_counter(deps: Deps) -> StdResult<Binary> {
    let current_value = &COUNTER.load(deps.storage)?;
    to_json_binary(&CurrentValueResponse {
        current_value: current_value.clone(),
    })
}
