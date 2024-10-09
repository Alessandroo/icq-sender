use cosmwasm_std::{Binary, Deps, DepsMut, Env, IbcMsg, MessageInfo, Response, StdResult, to_json_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CosmosQuery, ExecuteMsg, GetIcqStateResponse, GrpcQuery, InstantiateMsg, InterchainQueryPacketData, QueryBalanceMsg, QueryBalanceRequest, QueryMsg};
use crate::state::{CHANNEL_INFO, get_next_sequence_send, ICQ_REQUESTS, ICQ_RESPONSES};

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
        ExecuteMsg::SendQueryBalance(msg) => send_query_balance(deps, env, msg)
    }
}

pub fn send_query_balance(
    mut deps: DepsMut,
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

    let req: GrpcQuery = GrpcQuery {
        path: "/cosmos.bank.v1beta1.Query/Balance".to_string(),
        data: to_json_binary(&query_balance_request)?,
    };

    let cosmos_query: CosmosQuery = CosmosQuery {
        requests: vec![req]
    };

    let packet_data: InterchainQueryPacketData = InterchainQueryPacketData {
        data: cosmos_query.into(),
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
    let sequence = get_next_sequence_send(&mut deps)?;

    // save_icq_request(deps, sequence, &query_balance_request);
    ICQ_REQUESTS.save(deps.storage, sequence, &query_balance_request)?;


    Ok(Response::new()
        .add_attribute("method", "send_query_balance")
        .add_attribute("channel", channel_id)
        .add_attribute("sequence", sequence.to_string())
        // outbound IBC message, where packet is then received on other chain
        .add_message(ibc_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IcqState { sequence } => to_json_binary(&query_icq_state(deps, sequence)?),
    }
}

fn query_icq_state(deps: Deps, sequence: u64) -> StdResult<GetIcqStateResponse> {
    let bank_query = ICQ_REQUESTS.load(deps.storage, sequence)?;
    let query_response = ICQ_RESPONSES.load(deps.storage, sequence)?;
    let res = GetIcqStateResponse {
        request: bank_query,
        response: query_response,
    };
    Ok(res)
}
