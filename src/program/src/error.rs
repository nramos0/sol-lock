use thiserror::Error;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum SolLockError {
    #[error("PublicKeyMismatch")]
    PublicKeyMismatch,
    #[error("UninitializedAccount")]
    UninitializedAccount,
    #[error("IncorrectOwner")]
    IncorrectOwner,
    #[error("PublicKeysShouldBeUnique")]
    PublicKeysShouldBeUnique,
    #[error("NoFunds")]
    NoFunds,
    #[error("InsufficientFunds")]
    InsufficientFunds,
    #[error("NewDeadlineTooEarly")]
    NewDeadlineTooEarly,
    #[error("FundsLocked")]
    FundsLocked,
    #[error("PrematureUnlock")]
    PrematureUnlock,
    #[error("UnpackError")]
    UnpackError,
    #[error("ConflictingPayerInfo")]
    ConflictingPayerInfo,
    #[error("ConflictingReceiverInfo")]
    ConflictingReceiverInfo,
}

impl From<SolLockError> for ProgramError {
    fn from(e: SolLockError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SolLockError {
    fn type_of() -> &'static str {
        "Sol Lock Error"
    }
}

impl PrintProgramError for SolLockError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            SolLockError::PublicKeyMismatch => {
                msg!("Error: Public keys should be different, but are the same")
            }
            SolLockError::UninitializedAccount => msg!("Error: Account is not initialized."),
            SolLockError::IncorrectOwner => msg!("Error: Account owner is incorrect."),
            SolLockError::PublicKeysShouldBeUnique => {
                msg!("Error: Public keys are equivalent, but they should be different.")
            }
            SolLockError::NoFunds => msg!("Error: There are no funds in the account to remove."),
            SolLockError::InsufficientFunds => {
                msg!("Error: Cannot debit more lamports than have been locked in the account.")
            }
            SolLockError::NewDeadlineTooEarly => {
                msg!("Error: Cannot set the deadline earlier than the original after locking.")
            }
            SolLockError::FundsLocked => {
                msg!("Error: Cannot remove funds while the account is locked.")
            }
            SolLockError::PrematureUnlock => {
                msg!("Error: Cannot unlock before deadline is reached.")
            }
            SolLockError::UnpackError => {
                msg!("Error: There was an issue deserializing Account data.")
            }
            SolLockError::ConflictingPayerInfo => {
                msg!("Error: A payer account was passed but has_payer was false, or a payer account wasn't passed but has_payer was true.")
            }
            SolLockError::ConflictingReceiverInfo => {
                msg!("Error: A receiver account was passed but has_receiver was false, or a receiver account wasn't passed but has_receiver was true.")
            }
        }
    }
}
