use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, IbcEndpoint, StdResult, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use crate::msg::{ProtoCoin};

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

pub const ICQ_RESPONSES: Map<u64, ProtoCoin> = Map::new("icq_responses");

pub const ICQ_PRICE_RESPONSES: Map<u64, String> = Map::new("icq_price_responses");

pub const LAST_SEQUENCE_RECEIVE: Item<u64> = Item::new("last_sequence_receive");

pub const ICQ_ERRORS: Map<u64, String> = Map::new("icq_errors");

pub const LAST_SEQUENCE_ACKNOWLEDGMENT: Item<u64> = Item::new("last_sequence_acknowledgment");

/// Define a constant for storage key
const NEXT_SEQUENCE_SEND: Item<u64> = Item::new("next_sequence_send");

/// Example function to retrieve the next sequence number
pub fn get_next_sequence_send(deps: &mut DepsMut) -> StdResult<u64> {
    match NEXT_SEQUENCE_SEND.may_load(deps.storage)? {
        Some(sequence) => {
            // let mut sequence = NEXT_SEQUENCE_SEND.load(deps.storage)?;
            let new_sequence: u64 = sequence + 1;
            NEXT_SEQUENCE_SEND.save(deps.storage, &sequence)?;
            Ok(new_sequence)
        },  // If it exists, return the current value
        None => {
            // If it doesn't exist, initialize to 0
            NEXT_SEQUENCE_SEND.save(deps.storage, &0)?;
            Ok(0)
        }
    }
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