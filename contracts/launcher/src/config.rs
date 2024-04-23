use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, Api, StdError, StdResult, Storage, Uint64};
use cu_interfaces::launcher::InstantiateMsg;
use cw_storage_plus::Item;
use kujira::Denom;

use crate::ContractError;

#[cw_serde]
pub struct Config {
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

impl Config {
    pub fn new(msg: InstantiateMsg) -> Self {
        Self {
            owner: msg.owner,
            quanta_duration: msg.quanta_duration,
            advance_markets: msg.advance_markets,
            fin_code_id: msg.fin_code_id,
            quote_currency: msg.quote_currency,
        }
    }
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        Item::new("config").load(storage)
    }

    pub fn save(&self, storage: &mut dyn Storage, api: &dyn Api) -> Result<(), ContractError> {
        self.validate(api)?;
        ensure!(
            !self.quanta_duration.is_zero(),
            StdError::generic_err("quanta_duration must be greater than zero")
        );
        ensure!(
            self.advance_markets > 0,
            StdError::generic_err("advance_markets must be greater than zero")
        );
        Ok(Item::new("config").save(storage, self)?)
    }

    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        api.addr_validate(self.owner.as_str())?;

        Ok(())
    }
}
