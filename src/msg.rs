use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::{AbciQueryRequest, AbciQueryResponse};
use cosmwasm_schema::{cw_serde};
use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_schema::serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SendQueryBalance(QueryBalanceMsg),
    SendQueryTwap(QueryTwapMsg),
}

#[cw_serde]
pub struct QueryBalanceMsg {
    pub channel: String,
    pub address: String,
    pub denom: String,
}

#[cw_serde]
pub struct QueryTwapMsg {
    pub channel: String,
    pub pool_id: u64,
    pub base_asset: String,
    pub quote_asset: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AllBalances {},
    AllPriceFeeds {},
    AllErrors {},
    LastSequence {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InterchainQueryPacketData {
    pub data: Vec<u8>,
    pub memo: String,
}

// CosmosQuery contains a list of tendermint ABCI query requests. It should be
// used when sending queries to an SDK host chain.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CosmosQuery {
    #[prost(message, repeated, tag = "1")]
    pub requests: Vec<AbciQueryRequest>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InterchainQueryPacketAck {
    pub result: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CosmosResponsePacket {
    pub data: Binary,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CosmosResponse {
    #[prost(message, repeated, tag = "1")]
    pub responses: Vec<AbciQueryResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ProtoCoin {
    pub denom: String,
    pub amount: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArithmeticTwapToNowRequest {
    #[prost(uint64, tag = "1")]
    pub pool_id: u64,

    #[prost(string, tag = "2")]
    pub base_asset: String,

    #[prost(string, tag = "3")]
    pub quote_asset: String,

    #[prost(message, optional, tag = "4")]
    pub start_time: Option<Timestamp>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArithmeticTwapToNowResponse {
    #[prost(string, tag = "1")]
    pub arithmetic_twap: String,
}