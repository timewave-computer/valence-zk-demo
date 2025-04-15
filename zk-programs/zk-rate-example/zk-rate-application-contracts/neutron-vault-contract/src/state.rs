use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const BALANCES: Map<Addr, Uint128> = Map::new("balances");
pub const SHARES: Item<Uint128> = Item::new("shares");
