use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const MARKETS: Map<u64, Addr> = Map::new("markets");
