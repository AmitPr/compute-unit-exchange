use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint64};
use kujira::Denom;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    /// Duration of a quanta in seconds
    pub quanta_duration: Uint64,
    /// Number of future quanta to launch in advance
    pub advance_markets: u32,
    /// Code ID of the FIN orderbook contract
    pub fin_code_id: u64,
    /// Quote currency for the exchange
    pub quote_currency: Denom,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Launches new markets and compute unit tokens, if the next quanta is ready to be launched
    Crank {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
