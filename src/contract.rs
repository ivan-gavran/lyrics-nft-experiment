#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, ReplyOn, 
                    Response, StdResult, SubMsg, WasmMsg, Reply
                };
use cw2::set_contract_version;
use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw721_base::{
    Extension,
    msg::InstantiateMsg as Cw721InstantiateMsg
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use cw_utils::parse_instantiate_response_data;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lyrics-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DENOM: &str = "udenom";
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {


    let config = Config { 
        cw721_address: None,       
        owner: info.sender.clone(),
        nft_ids: HashMap::new(),        
        guessing_fee: msg.guessing_fee,
        name: msg.name,
        symbol: msg.symbol,
        token_uri: msg.token_uri,
        extension: msg.extension,
        next_available_token_id: 0
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;    

    let sub_msg: Vec<SubMsg> = vec![
        SubMsg {
            msg: WasmMsg::Instantiate {
                code_id: msg.token_code_id,
                msg: to_json_binary(&Cw721InstantiateMsg {
                    name: msg.name.clone(),
                    symbol: msg.symbol,
                    minter: env.contract.address.to_string(),
                })?,
                funds: vec![],
                admin: None,
                label: String::from("Instantiate the NFT contract"),
            }
            .into(),
            id: INSTANTIATE_TOKEN_REPLY_ID,
            gas_limit: None,
            reply_on: ReplyOn::Success,
            payload: to_json_binary(&"")?,
        }
    ];

    Ok(Response::new().add_submessages(sub_msg))    
}

// Reply callbacks (here, only for the instantiate token reply)
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INSTANTIATE_TOKEN_REPLY_ID => {
            
            let mut config = CONFIG.load(deps.storage)?;

            if config.cw721_address.is_some() {
                return Err(ContractError::NFTContractALreadyLinked {});
            }

            let reply = parse_instantiate_response_data(reply).unwrap();
            config.cw721_address = Addr::unchecked(reply.contract_address).into();

            
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new())
        }
        _ => Err(ContractError::UnknownReplyId { reply_id: reply.id }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateNFT { hash, url  } => execute::create_nft(deps, info.sender, hash, url),
        ExecuteMsg::Guess { nft_id, lyrics } => execute::guess(deps, info.sender, info.funds, nft_id, lyrics),
        
    }
}

pub mod execute {
    use cosmwasm_std::Coin;

    use super::*;

    pub fn create_nft(deps: DepsMut, sender: Addr, hash: u64, url: String) -> Result<Response, ContractError> {        
        let mut config = CONFIG.load(deps.storage)?;

        if config.owner != sender {
            return Err(ContractError::Unauthorized {});
        }

        config.nft_ids.insert(config.next_available_token_id, hash);
        config.next_available_token_id += 1;


        // mint the NFT
        let mint_msg = Cw721ExecuteMsg::<Extension, Empty>::Mint {
            token_id: config.next_available_token_id.to_string(),
            owner: sender.to_string(),
            token_uri: Some(url),
            extension: config.extension.clone(),
        };


        CONFIG.save(deps.storage, &config)?;

        Ok(Response::new().add_attribute("action", "create_nft"))
    }

    pub fn guess(deps: DepsMut, sender: Addr, funds: Vec<Coin>, nft_id: u32, lyrics: String) -> Result<Response, ContractError> {
        
        // check if the funds are paid in the right denom and the right amount
        if funds.len() != 1 {
            return Err(ContractError::InvalidFunds {});
        }

        if funds[0].denom != DENOM {
            return Err(ContractError::InvalidFunds {});
        }

        if funds[0].amount < CONFIG.load(deps.storage)?.guessing_fee {
            return Err(ContractError::InvalidFunds {});
        }

        

        let config = CONFIG.load(deps.storage)?;

        let hash = *config.nft_ids.get(&nft_id).ok_or(ContractError::NFTNotFound {})?;
        

        let mut hasher = DefaultHasher::new();
        lyrics.hash(&mut hasher);
        let lyrics_hash = hasher.finish();

        if hash != lyrics_hash {
            return Err(ContractError::IncorrectLyrics {});
        }

        // transfer the NFT to the new owner
        let transfer_msg = Cw721ExecuteMsg::<Extension, Empty>::TransferNft {
            recipient: sender.to_string(),
            token_id: nft_id.to_string(),
        };

        Ok(Response::new().add_attribute("action", "guess"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = CONFIG.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
