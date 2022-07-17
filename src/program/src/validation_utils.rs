use solana_program::{
    account_info::AccountInfo,
    clock::UnixTimestamp,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use crate::{
    error::SolLockError,
    state::{Account, State},
};

#[must_use]
pub fn assert_is_signer(account: &AccountInfo) -> ProgramResult {
    if account.is_signer {
        Ok(())
    } else {
        Err(ProgramError::MissingRequiredSignature)
    }
}

#[must_use]
pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> ProgramResult {
    if key1 != key2 {
        Err(SolLockError::PublicKeyMismatch.into())
    } else {
        Ok(())
    }
}

#[must_use]
pub fn assert_keys_unequal(key1: Pubkey, key2: Pubkey) -> ProgramResult {
    if key1 == key2 {
        Err(SolLockError::PublicKeysShouldBeUnique.into())
    } else {
        Ok(())
    }
}

/// assert initialized account
#[must_use]
pub fn assert_initialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(SolLockError::UninitializedAccount.into())
    } else {
        Ok(account)
    }
}

/// assert owned by
#[must_use]
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        Err(SolLockError::IncorrectOwner.into())
    } else {
        Ok(())
    }
}

#[must_use]
pub fn assert_has_funds(account: &Account) -> ProgramResult {
    if account.lamports.is_some() && account.lamports.unwrap() > 0 {
        Ok(())
    } else {
        Err(SolLockError::NoFunds.into())
    }
}

#[must_use]
pub fn assert_sufficient_funds(account: &Account, lamports_to_remove: u64) -> ProgramResult {
    if account.lamports.unwrap() < lamports_to_remove {
        Err(SolLockError::InsufficientFunds.into())
    } else {
        Ok(())
    }
}

#[must_use]
pub fn assert_valid_new_deadline(account: &Account, deadline: UnixTimestamp) -> ProgramResult {
    if account.deadline.is_some() && account.deadline.unwrap() > deadline {
        Err(SolLockError::NewDeadlineTooEarly.into())
    } else {
        Ok(())
    }
}

#[must_use]
pub fn assert_can_lock(account: &Account) -> ProgramResult {
    if account.state != State::ReadyUnlocked {
        Err(ProgramError::InvalidInstructionData)
    } else {
        Ok(())
    }
}

#[must_use]
pub fn assert_can_unlock(account: &Account, now: UnixTimestamp) -> ProgramResult {
    if account.state != State::Locked {
        Err(ProgramError::InvalidInstructionData)
    } else if now < account.deadline.unwrap() {
        Err(SolLockError::PrematureUnlock.into())
    } else {
        Ok(())
    }
}

#[must_use]
pub fn assert_receiver_validity<'a, 'b>(
    owner_info: &'a AccountInfo<'b>,
    sol_receiver_account_res: Result<&'a AccountInfo<'b>, ProgramError>,
    has_receiver: bool,
) -> Result<&'a AccountInfo<'b>, ProgramError> {
    if sol_receiver_account_res.is_ok() {
        if !has_receiver {
            return Err(SolLockError::ConflictingPayerInfo.into());
        }
        Ok(sol_receiver_account_res.unwrap())
    } else {
        if has_receiver {
            return Err(SolLockError::ConflictingPayerInfo.into());
        }
        Ok(owner_info)
    }
}
