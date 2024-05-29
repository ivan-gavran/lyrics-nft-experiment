use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw721_base::Extension;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,     
    pub guessing_fee: Uint128,
    pub name: String,
    pub symbol: String,
    pub token_uri: String,
    pub extension: Extension,
    pub next_available_token_id: u32,
    pub token_code_id: u64
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateNFT{
        hash: u64,
        url: String
    },
    Guess{
        nft_id: u32,
        lyrics: String
    }
    
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}
