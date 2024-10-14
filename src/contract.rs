use cosmos_sdk_proto::cosmos::bank::v1beta1::QueryBalanceRequest;
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::AbciQueryRequest;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, IbcMsg, MessageInfo, Response, StdResult, to_json_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use prost::Message;

use crate::error::ContractError;
use crate::msg::{ArithmeticTwapToNowRequest, CosmosQuery, ExecuteMsg, InstantiateMsg, InterchainQueryPacketData, ProtoCoin, QueryBalanceMsg, QueryMsg, QueryTwapMsg, Timestamp};
use crate::state::{CHANNEL_INFO, ICQ_ERRORS, ICQ_PRICE_RESPONSES, ICQ_RESPONSES, LAST_SEQUENCE_ACKNOWLEDGMENT};

const CONTRACT_NAME: &str = "crates.io:cw-ibc-example";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SendQueryBalance(msg) => send_query_balance(deps, env, msg),
        ExecuteMsg::SendQueryTwap(msg) => send_query_twap(deps, env, msg),
    }
}

pub fn send_query_balance(
    deps: DepsMut,
    env: Env,
    msg: QueryBalanceMsg,
) -> Result<Response, ContractError> {
    // ensure the requested channel is registered
    if !CHANNEL_INFO.has(deps.storage, &msg.channel) {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    }

    let query_balance_request: QueryBalanceRequest = QueryBalanceRequest {
        address: msg.address,
        denom: msg.denom,
    };

    let req: AbciQueryRequest = AbciQueryRequest {
        data: query_balance_request.encode_to_vec(),
        path: "/cosmos.bank.v1beta1.Query/Balance".to_string(),
        height: 0,
        prove: false,
    };

    let cosmos_query: CosmosQuery = CosmosQuery {
        requests: vec![req]
    };

    let packet_data: InterchainQueryPacketData = InterchainQueryPacketData {
        data: cosmos_query.encode_to_vec(),
        memo: "test icq request".to_string(),
    };

    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(120);

    // Borrow `msg.channel` as a reference
    let channel_id = &msg.channel;

    // prepare ibc message
    let ibc_msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_json_binary(&packet_data)?,
        timeout: timeout.into(),
    };

    // Get the current sequence number from storage
    // let sequence = get_next_sequence_send(&mut deps)?;

    // save_icq_request(deps, sequence, &query_balance_request);
    // ICQ_REQUESTS.save(deps.storage, sequence, &query_balance_request)?;


    Ok(Response::new()
        .add_attribute("method", "send_query_balance")
        .add_attribute("channel", channel_id)
        // .add_attribute("sequence", sequence.to_string())
        // outbound IBC message, where packet is then received on other chain
        .add_message(ibc_msg))
}

pub fn send_query_twap(
    deps: DepsMut,
    env: Env,
    msg: QueryTwapMsg,
) -> Result<Response, ContractError> {
    // ensure the requested channel is registered
    if !CHANNEL_INFO.has(deps.storage, &msg.channel) {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    }

    let block_time = env.block.time;  // Get the current block time
    let four_hours = 4 * 3600;        // 4 hours in seconds

    // Subtract 4 hours from the block time
    let new_time = block_time.seconds() - four_hours;

    // Create the prost::Timestamp struct
    let timestamp = Timestamp {
        seconds: new_time as i64,
        nanos: 0,
    };

    let query_twap_request: ArithmeticTwapToNowRequest = ArithmeticTwapToNowRequest {
        pool_id: msg.pool_id,
        base_asset: msg.base_asset,
        quote_asset: msg.quote_asset,
        start_time: Some(timestamp),
    };

    let req: AbciQueryRequest = AbciQueryRequest {
        data: query_twap_request.encode_to_vec(),
        path: "/osmosis.twap.v1beta1.Query/ArithmeticTwapToNow".to_string(),
        height: 0,
        prove: false,
    };

    let cosmos_query: CosmosQuery = CosmosQuery {
        requests: vec![req]
    };

    let packet_data: InterchainQueryPacketData = InterchainQueryPacketData {
        data: cosmos_query.encode_to_vec(),
        memo: "test icq request".to_string(),
    };

    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(120);

    // Borrow `msg.channel` as a reference
    let channel_id = &msg.channel;

    // prepare ibc message
    let ibc_msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_json_binary(&packet_data)?,
        timeout: timeout.into(),
    };

    // Get the current sequence number from storage
    // let sequence = get_next_sequence_send(&mut deps)?;

    // save_icq_request(deps, sequence, &query_balance_request);
    // ICQ_REQUESTS.save(deps.storage, sequence, &query_balance_request)?;


    Ok(Response::new()
        .add_attribute("method", "send_query_balance")
        .add_attribute("channel", channel_id)
        // .add_attribute("sequence", sequence.to_string())
        // outbound IBC message, where packet is then received on other chain
        .add_message(ibc_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllBalances {} => to_json_binary(&query_all_balances(deps)?),
        QueryMsg::AllPriceFeeds {} => to_json_binary(&query_all_price_feed(deps)?),
        QueryMsg::AllErrors {} => to_json_binary(&query_all_errors(deps)?),
        QueryMsg::LastSequence {} => {
            let result = LAST_SEQUENCE_ACKNOWLEDGMENT.load(deps.storage)?;
            to_json_binary(&result)
        }
    }
}

fn query_all_balances(deps: Deps) -> StdResult<Vec<(u64, ProtoCoin)>> {
    let balances: StdResult<Vec<(u64, ProtoCoin)>> = ICQ_RESPONSES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();

    // Convert the result to binary
    balances
}

fn query_all_price_feed(deps: Deps) -> StdResult<Vec<(u64, String)>> {
    let prices: StdResult<Vec<(u64, String)>> = ICQ_PRICE_RESPONSES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();

    // Convert the result to binary
    prices
}

fn query_all_errors(deps: Deps) -> StdResult<Vec<(u64, String)>> {
    let balances: StdResult<Vec<(u64, String)>> = ICQ_ERRORS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect();

    // Convert the result to binary
    balances
}