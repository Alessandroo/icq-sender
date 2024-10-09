use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_schema::serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, Coin};

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
pub struct InterchainQueryPacketData {
    pub data: CosmosQuery,
    pub memo: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CosmosQuery {
    pub requests: Vec<GrpcQuery>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GrpcQuery {
    /// The fully qualified endpoint path used for routing.
    /// It follows the format `/service_path/method_name`,
    /// eg. "/cosmos.authz.v1beta1.Query/Grants"
    pub path: String,
    /// The expected protobuf message type (not [Any](https://protobuf.dev/programming-guides/proto3/#any)), binary encoded
    pub data: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryBalanceRequest {
    pub address: String,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InterchainQueryPacketAck {
    pub data: CosmosResponse,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CosmosResponse {
    pub responses: Vec<ResponseQuery>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ResponseQuery {
    pub value: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryBalanceResponse {
    // balance is the balance of the coin.
    pub balance: Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(crate::msg::GetIcqStateResponse)]
    IcqState {
        sequence: u64,
    }
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetIcqStateResponse {
    pub request: QueryBalanceRequest,
    pub response: QueryBalanceResponse,
}

