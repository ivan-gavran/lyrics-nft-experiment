use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use cw721_base::Extension;

type NFTId = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    // a mapping from ids to hashes
    pub nft_ids: HashMap<u32, u64>,
    pub owner: Addr,
    pub cw721_address: Option<Addr>,    
    pub guessing_fee: Uint128,
    pub name: String,
    pub symbol: String,
    pub token_uri: String,
    pub extension: Extension,
    pub next_available_token_id: u32,
}

pub const CONFIG: Item<Config> = Item::new("config");