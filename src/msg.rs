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
}

#[cw_serde]
pub struct QueryBalanceMsg {
    pub channel: String,
    pub address: String,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AllBalances {},
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