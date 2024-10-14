use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::AbciQueryResponse;
use cosmwasm_std::{Binary, DepsMut, Env, from_json, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use prost::Message;

use crate::{ContractError, error::Never};
use crate::ack::{Ack, make_ack_success};
use crate::msg::{ArithmeticTwapToNowResponse, CosmosResponse, CosmosResponsePacket, InterchainQueryPacketAck};
use crate::state::{CHANNEL_INFO, ChannelInfo, ICQ_ERRORS, ICQ_PRICE_RESPONSES, LAST_SEQUENCE_ACKNOWLEDGMENT};

pub const IBC_VERSION: &str = "icq-1";

/// Handles the `OpenInit` and `OpenTry` parts of the IBC handshake.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;

    // Initialize the count for this channel to zero.
    // let channel = msg.channel().endpoint.channel_id.clone();
    // CONNECTION_COUNTS.save(deps.storage, channel.clone(), &0)?;

    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        id: channel.endpoint.channel_id,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
    };
    CHANNEL_INFO.save(deps.storage, &info.id, &info)?;

    Ok(IbcBasicResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel().endpoint.channel_id.clone();
    // Reset the state for the channel.
    CHANNEL_INFO.remove(deps.storage, &channel);
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_close")
        .add_attribute("channel", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    Ok(IbcReceiveResponse::new(make_ack_success()).add_attribute("method", "ibc_packet_receive"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    LAST_SEQUENCE_ACKNOWLEDGMENT.save(deps.storage, &msg.original_packet.sequence)?;

    let icq_msg: Ack = from_json(&msg.acknowledgement.data)?;
    match icq_msg {
        Ack::Result(_) => on_packet_success(deps, msg.acknowledgement.data, msg.original_packet),
        Ack::Error(error) => {
            ICQ_ERRORS.save(deps.storage, msg.original_packet.sequence, &error)?;
            Ok(IbcBasicResponse::new()
                   // .set_ack(make_ack_fail(error.to_string()))
                   .add_attribute("method", "ibc_packet_ack")
                   .add_attribute("error", error.to_string())
                   .add_attribute("sequence", msg.original_packet.sequence.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // As with ack above, nothing to do here. If we cared about
    // keeping track of state between the two chains then we'd want to
    // respond to this likely as it means that the packet in question
    // isn't going anywhere.
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_timeout"))
}

pub fn validate_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    // We expect an unordered channel here. Ordered channels have the
    // property that if a message is lost the entire channel will stop
    // working until you start it again.
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::OnlyOrderedChannel {});
    }

    if channel.version != IBC_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            actual: channel.version.to_string(),
            expected: IBC_VERSION.to_string(),
        });
    }

    // Make sure that we're talking with a counterparty who speaks the
    // same "protocol" as us.
    //
    // For a connection between chain A and chain B being established
    // by chain A, chain B knows counterparty information during
    // `OpenTry` and chain A knows counterparty information during
    // `OpenAck`. We verify it when we have it but when we don't it's
    // alright.
    if let Some(counterparty_version) = counterparty_version {
        if counterparty_version != IBC_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                actual: counterparty_version.to_string(),
                expected: IBC_VERSION.to_string(),
            });
        }
    }

    Ok(())
}

// update the balance stored on this (channel, denom) index
fn on_packet_success(deps: DepsMut, result: Binary, packet: IbcPacket) -> Result<IbcBasicResponse, ContractError> {
    let ack_data: InterchainQueryPacketAck = from_json(&result)?;

    let cosmos_response: CosmosResponsePacket = from_json(&ack_data.result)?;

    let query_responses: CosmosResponse = CosmosResponse::decode(cosmos_response.data.as_slice())?;

    if let Some(first_response) = get_first_response_safe(&query_responses) {
        let price_response: ArithmeticTwapToNowResponse = ArithmeticTwapToNowResponse::decode(first_response.value.as_slice())?;
        ICQ_PRICE_RESPONSES.save(deps.storage, packet.sequence, &price_response.arithmetic_twap)?;
    }

    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_packet_ack")
        .add_attribute("sequence", packet.sequence.to_string())
    )
}

fn get_first_response_safe(cosmos_response: &CosmosResponse) -> Option<&AbciQueryResponse> {
    cosmos_response.responses.first()
}