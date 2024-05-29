use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NFT not found")]
    NFTNotFound {},

    // Incorrect Lyrics error
    #[error("Incorrect Lyrics")]
    IncorrectLyrics {},

    // Invalid Funds error
    #[error("Invalid Funds")]
    InvalidFunds {},

    // NFTContractALreadyLinked error
    #[error("NFT Contract Already Linked")]
    NFTContractALreadyLinked {},

    // NFTContractNotLinked error
    #[error("NFT Contract Not Linked")]
    UnknownReplyId{reply_id: u64},

    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
