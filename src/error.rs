use cosmwasm_std::{RecoverPubkeyError, StdError, VerificationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("InvalidInput")]
    InvalidInput {},

    #[error("NotReady")]
    NotReady {},

    #[error("Settled")]
    Settled {},

    #[error("InvalidWithdrawal")]
    InvalidWithdrawal {},
    
    #[error("InvalidSignature")]
    InvalidSignature {},
}

impl From<VerificationError> for ContractError {
    fn from(err: VerificationError) -> Self {
        return err.into()
    }
}

impl From<RecoverPubkeyError> for ContractError {
    fn from(err: RecoverPubkeyError) -> Self {
        return err.into()
    }
}
