use cosmwasm_std::{
    ensure_eq, instantiate2_address, to_json_binary, Addr, Api, Binary, CodeInfoResponse,
    CosmosMsg, Decimal256, DepsMut, Env, MessageInfo, Order, QuerierWrapper, Response, StdResult,
    Timestamp, WasmMsg,
};
use cw_utils::Expiration;
use kujira::{DenomMsg, KujiraMsg, KujiraQuery, Precision};

use crate::{config::Config, state::MARKETS, ContractError};

pub fn crank(
    deps: DepsMut<KujiraQuery>,
    env: Env,
    info: MessageInfo,
    cfg: Config,
) -> Result<Response<KujiraMsg>, ContractError> {
    ensure_eq!(info.sender, cfg.owner, ContractError::Unauthorized {});
    let mut msgs = vec![];

    let markets = MARKETS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<_>, _>>()?;

    // First, garbage collect all expired markets.
    let mut iter = markets.iter().peekable();
    while let Some((id, _)) = iter.next_if(|(expiry, _)| {
        Expiration::AtTime(Timestamp::from_seconds(*expiry)).is_expired(&env.block)
    }) {
        MARKETS.remove(deps.storage, *id);
        todo!("Garbage collect market {}", id);
    }

    // Then, launch any new markets that are ready.
    let remaining_markets = iter.collect::<Vec<_>>();
    let num_to_launch = cfg.advance_markets - remaining_markets.len() as u32;
    let mut new_expiry = remaining_markets
        .last()
        .map(|(expiry, _)| *expiry)
        .unwrap_or(env.block.time.seconds());
    for _ in 0..num_to_launch {
        new_expiry += cfg.quanta_duration.u64();

        let (fin, launch_msgs) = launch_market(deps.api, &deps.querier, &env, &cfg, new_expiry)?;
        msgs.extend(launch_msgs);
        MARKETS.save(deps.storage, new_expiry, &fin)?;

        todo!("Launch new market at {}", new_expiry);
    }

    Ok(Response::default())
}

fn launch_market(
    api: &dyn Api,
    querier: &QuerierWrapper<KujiraQuery>,
    env: &Env,
    cfg: &Config,
    expiry: u64,
) -> StdResult<(Addr, Vec<CosmosMsg<KujiraMsg>>)> {
    let mut msgs = vec![];
    // Create the denomination for the compute unit with the specified expiry.
    let denom_creation = DenomMsg::Create {
        subdenom: format!("cu-{expiry}").into(),
    };
    let full_denom = format!("factory/{}/cu-{expiry}", env.contract.address);
    msgs.push(denom_creation.into());

    // Get the FIN contract address using Instantiate2.
    let salt = Binary::from(format!("cu-{expiry}").as_bytes());
    let self_addr = api.addr_canonicalize(env.contract.address.as_str())?;
    let CodeInfoResponse { checksum, .. } = querier.query_wasm_code_info(cfg.fin_code_id)?;
    let fin_address =
        api.addr_humanize(&instantiate2_address(checksum.as_slice(), &self_addr, &salt).unwrap())?;

    // Instantiate the orrderbook with the new compute units.
    let fin_msg = kujira::fin::InstantiateMsg {
        owner: env.contract.address.clone(),
        denoms: [
            cw20::Denom::Native(full_denom),
            cw20::Denom::Native(cfg.quote_currency.to_string()),
        ],
        decimal_delta: Some(0),
        price_precision: Precision::DecimalPlaces(4u8),
        fee_taker: Decimal256::zero(),
        fee_maker: Decimal256::zero(),
        fee_address: env.contract.address.clone(),
    };
    let fin_launch = WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: cfg.fin_code_id,
        label: format!("CU Exchange: FIN: {expiry}-USD"),
        msg: to_json_binary(&fin_msg)?,
        funds: vec![],
        salt,
    };
    msgs.push(fin_launch.into());

    Ok((fin_address, msgs))
}
