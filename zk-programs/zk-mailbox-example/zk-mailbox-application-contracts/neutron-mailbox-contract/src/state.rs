use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

pub const MESSAGES: Map<String, String> = Map::new("messages");
pub const COUNTER: Item<Uint128> = Item::new("counter");
