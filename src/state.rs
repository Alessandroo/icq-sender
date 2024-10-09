use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, IbcEndpoint, StdResult, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use crate::msg::{QueryBalanceRequest, QueryBalanceResponse};

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

/// indexed by (channel_id, denom) maintaining the balance of the channel in that currency
pub const ICQ_REQUESTS: Map<u64, QueryBalanceRequest> = Map::new("icq_requests");

pub const ICQ_RESPONSES: Map<u64, QueryBalanceResponse> = Map::new("icq_responses");

/// Define a constant for storage key
const NEXT_SEQUENCE_SEND: Item<u64> = Item::new("next_sequence_send");

/// Example function to retrieve the next sequence number
pub fn get_next_sequence_send(deps: &mut DepsMut) -> StdResult<u64> {
    let mut sequence = NEXT_SEQUENCE_SEND.load(deps.storage)?;
    sequence += 1;
    NEXT_SEQUENCE_SEND.save(deps.storage, &sequence)?;
    Ok(sequence)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SequenceState {
    pub next_sequence_send: u64,
}

#[cw_serde]
#[derive(Default)]
pub struct ChannelState {
    pub outstanding: Uint128,
    pub total_sent: Uint128,
}

#[cw_serde]
pub struct ChannelInfo {
    /// id of this channel
    pub id: String,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: String,
}