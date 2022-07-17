//! Program instruction processor
use crate::{
    error::SolLockError,
    instruction::*,
    pack_utils::WithData,
    state::{Account, State, SOL_LOCK_ACCOUNT_SIZE},
    validation_utils::*,
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};
use std::convert::TryInto;

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SolLockInstruction::try_from_slice(instruction_data)?;

    match instruction {
        SolLockInstruction::CreateAccount(ctx) => create_account(program_id, accounts, ctx)?,
        SolLockInstruction::AddSol(ctx) => add_sol(program_id, accounts, ctx)?,
        SolLockInstruction::RemoveSol(ctx) => remove_sol(program_id, accounts, ctx)?,
        SolLockInstruction::SetDeadline(ctx) => set_deadline(program_id, accounts, ctx)?,
        SolLockInstruction::Lock(ctx) => lock(program_id, accounts, ctx)?,
        SolLockInstruction::Unlock(ctx) => unlock(program_id, accounts, ctx)?,
        SolLockInstruction::Stake(_) => unimplemented!(),
        SolLockInstruction::Unstake(_) => unimplemented!(),
    }

    Ok(())
}

/// Creates a SolLock account
pub fn create_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ctx: CreateAccount,
) -> ProgramResult {
    msg!("SolLock::CreateAccount");

    let CreateAccount { acc_index } = ctx;

    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let sol_lock_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    let (sol_lock_account_key, sol_lock_account_bump_seed) = Pubkey::find_program_address(
        &[owner_info.key.as_ref(), &acc_index.to_le_bytes()],
        program_id,
    );

    assert_is_signer(owner_info)?;
    assert_keys_equal(sol_lock_account_key.clone(), *sol_lock_account_info.key)?;
    assert_owned_by(sol_lock_account_info, system_account_info.key)?;

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(SOL_LOCK_ACCOUNT_SIZE);

    invoke_signed(
        &system_instruction::create_account(
            &owner_info.key,
            &sol_lock_account_key,
            lamports,
            SOL_LOCK_ACCOUNT_SIZE.try_into().unwrap(),
            program_id,
        ),
        &[
            owner_info.clone(),
            sol_lock_account_info.clone(),
            system_account_info.clone(),
        ],
        &[&[
            owner_info.key.as_ref(),
            &acc_index.to_le_bytes(),
            &[sol_lock_account_bump_seed],
        ]],
    )?;

    let sol_lock_account_data = Account {
        owner: owner_info.key.clone(),
        lamports: None,
        deadline: None,
        state: State::Initialized,
        stake_account: None,
    };

    sol_lock_account_data.pack_into_slice(&mut sol_lock_account_info.data.borrow_mut());

    msg!(
        "Account {:?} created successfully.",
        sol_lock_account_info.key
    );

    Ok(())
}

fn get_sol_lock_account(program_id: &Pubkey, owner: &Pubkey, acc_index: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[owner.as_ref(), &acc_index.to_le_bytes()], program_id)
}

/// Add Sol to a SolLock account to prepare for locking
pub fn add_sol(program_id: &Pubkey, accounts: &[AccountInfo], ctx: AddSol) -> ProgramResult {
    msg!("SolLock::AddSol");

    let AddSol {
        acc_index,
        lamports,
        has_payer,
    } = ctx;

    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let sol_lock_account_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let sol_payer_account_res = next_account_info(account_info_iter);

    let payer_account_info = if sol_payer_account_res.is_ok() {
        if !has_payer {
            return Err(SolLockError::ConflictingPayerInfo.into());
        }
        sol_payer_account_res.unwrap()
    } else {
        if has_payer {
            return Err(SolLockError::ConflictingPayerInfo.into());
        }
        owner_info
    };

    let sol_lock_account_key = get_sol_lock_account(program_id, owner_info.key, acc_index).0;

    assert_is_signer(owner_info)?;
    assert_is_signer(payer_account_info)?;
    assert_keys_equal(sol_lock_account_key.clone(), *sol_lock_account_info.key)?;
    assert_owned_by(sol_lock_account_info, program_id)?;
    assert_initialized::<Account>(&sol_lock_account_info)?;

    sol_lock_account_info.with_mut_data(|mut sol_lock_account_data| {
        msg!(
            "Adding {} lamports to SolLock account {:#?}",
            lamports,
            owner_info.key,
        );

        invoke(
            &system_instruction::transfer(&owner_info.key, &sol_lock_account_key, lamports),
            &[
                owner_info.clone(),
                sol_lock_account_info.clone(),
                system_account_info.clone(),
            ],
        )?;

        let new_state = match sol_lock_account_data.state {
            State::Uninitialized => unreachable!(),
            State::Initialized => State::HasFunds,
            State::HasFunds => State::HasFunds,
            State::HasDeadline => State::ReadyUnlocked,
            State::ReadyUnlocked => State::ReadyUnlocked,
            State::Locked => State::Locked,
            State::Staked => State::Staked,
        };

        match sol_lock_account_data.state {
            State::Initialized | State::HasDeadline => {
                sol_lock_account_data.lamports = Some(lamports);
            }
            State::HasFunds | State::ReadyUnlocked | State::Locked | State::Staked => {
                sol_lock_account_data.lamports =
                    Some(sol_lock_account_data.lamports.unwrap() + lamports);
            }
            State::Uninitialized => unreachable!(),
        };

        sol_lock_account_data.state = new_state;

        Ok(sol_lock_account_data)
    })?;

    Ok(())
}

/// Remove Sol from an unlocked SolLock account
pub fn remove_sol(program_id: &Pubkey, accounts: &[AccountInfo], ctx: RemoveSol) -> ProgramResult {
    msg!("SolLock::RemoveSol");

    let RemoveSol {
        acc_index,
        lamports,
        has_receiver,
    } = ctx;

    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let sol_lock_account_info = next_account_info(account_info_iter)?;
    let sol_receiver_account_res = next_account_info(account_info_iter);

    let receiver_account_info =
        assert_receiver_validity(owner_info, sol_receiver_account_res, has_receiver)?;

    let sol_lock_key = Pubkey::find_program_address(
        &[owner_info.key.as_ref(), &acc_index.to_le_bytes()],
        program_id,
    )
    .0;

    assert_is_signer(owner_info)?;
    assert_keys_equal(sol_lock_key, *sol_lock_account_info.key)?;
    assert_owned_by(sol_lock_account_info, program_id)?;
    assert_initialized::<Account>(&sol_lock_account_info)?;

    sol_lock_account_info.with_mut_data(|mut sol_lock_account_data| {
        assert_sufficient_funds(&sol_lock_account_data, lamports)?;
        assert_has_funds(&sol_lock_account_data)?;

        macro_rules! has_lamports_remaining {
            ($account:ident, $lamports:ident) => {
                $account.lamports.unwrap() > $lamports
            };
        }

        let new_state = match sol_lock_account_data.state {
            State::HasFunds if has_lamports_remaining!(sol_lock_account_data, lamports) => {
                State::HasFunds
            }
            State::HasFunds => State::Initialized,
            State::ReadyUnlocked => State::ReadyUnlocked,

            State::Initialized | State::HasDeadline => return Err(SolLockError::NoFunds.into()),
            State::Locked | State::Staked => return Err(SolLockError::FundsLocked.into()),

            State::Uninitialized => unreachable!(),
        };

        match sol_lock_account_data.state {
            State::HasFunds | State::ReadyUnlocked => {
                sol_lock_account_data.lamports =
                    Some(sol_lock_account_data.lamports.unwrap() - lamports);
            }
            State::Uninitialized
            | State::Initialized
            | State::HasDeadline
            | State::Locked
            | State::Staked => unreachable!(),
        };

        sol_lock_account_data.state = new_state;

        **sol_lock_account_info.try_borrow_mut_lamports()? -= lamports;
        **receiver_account_info.try_borrow_mut_lamports()? += lamports;

        Ok(sol_lock_account_data)
    })?;

    Ok(())
}

/// Set deadline on a SolLock account
pub fn set_deadline(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ctx: SetDeadline,
) -> ProgramResult {
    msg!("SolLock::SetDeadline");

    let SetDeadline {
        acc_index,
        deadline,
    } = ctx;

    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let sol_lock_account_info = next_account_info(account_info_iter)?;

    let sol_lock_account_key = get_sol_lock_account(program_id, owner_info.key, acc_index).0;

    assert_is_signer(owner_info)?;
    assert_keys_equal(sol_lock_account_key.clone(), *sol_lock_account_info.key)?;
    assert_owned_by(sol_lock_account_info, program_id)?;
    assert_initialized::<Account>(&sol_lock_account_info)?;

    msg!(
        "Setting deadline for SolLock account {:#?} to {:#?}",
        owner_info.key,
        deadline
    );

    sol_lock_account_info.with_mut_data(|mut sol_lock_account_data| {
        assert_valid_new_deadline(&sol_lock_account_data, deadline)?;

        let new_state = match sol_lock_account_data.state {
            State::Initialized => State::HasDeadline,
            State::HasFunds => State::ReadyUnlocked,
            State::HasDeadline => State::HasDeadline,
            State::ReadyUnlocked => State::ReadyUnlocked,
            State::Locked => State::Locked,
            State::Staked => State::Locked,
            State::Uninitialized => unreachable!(),
        };

        match sol_lock_account_data.state {
            State::Initialized
            | State::HasFunds
            | State::HasDeadline
            | State::ReadyUnlocked
            | State::Locked
            | State::Staked => {
                sol_lock_account_data.deadline = Some(deadline);
            }
            State::Uninitialized => unreachable!(),
        };

        sol_lock_account_data.state = new_state;

        Ok(sol_lock_account_data)
    })?;

    Ok(())
}

/// Lock a SolLock account
pub fn lock(program_id: &Pubkey, accounts: &[AccountInfo], ctx: Lock) -> ProgramResult {
    msg!("SolLock::Lock");

    let Lock { acc_index } = ctx;

    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let sol_lock_account_info = next_account_info(account_info_iter)?;

    let sol_lock_account_key = get_sol_lock_account(program_id, owner_info.key, acc_index).0;

    assert_is_signer(owner_info)?;
    assert_keys_equal(sol_lock_account_key.clone(), *sol_lock_account_info.key)?;
    assert_owned_by(sol_lock_account_info, program_id)?;
    assert_initialized::<Account>(&sol_lock_account_info)?;

    sol_lock_account_info.with_mut_data(|mut sol_lock_account_data| {
        assert_can_lock(&sol_lock_account_data)?;

        msg!("Locking SolLock account {:#?}", owner_info.key);

        sol_lock_account_data.state = match sol_lock_account_data.state {
            State::ReadyUnlocked => State::Locked,
            State::Uninitialized
            | State::Initialized
            | State::HasFunds
            | State::HasDeadline
            | State::Locked
            | State::Staked => unreachable!(),
        };

        Ok(sol_lock_account_data)
    })?;

    Ok(())
}

/// Unlock a SolLock account
pub fn unlock(program_id: &Pubkey, accounts: &[AccountInfo], ctx: Unlock) -> ProgramResult {
    msg!("SolLock::Unlock");

    let Unlock {
        acc_index,
        has_receiver,
    } = ctx;

    let account_info_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_info_iter)?;
    let sol_lock_account_info = next_account_info(account_info_iter)?;
    let sol_receiver_account_res = next_account_info(account_info_iter);

    let receiver_account_info =
        assert_receiver_validity(owner_info, sol_receiver_account_res, has_receiver)?;

    let sol_lock_account_key = get_sol_lock_account(program_id, owner_info.key, acc_index).0;

    assert_is_signer(owner_info)?;
    assert_keys_equal(sol_lock_account_key.clone(), *sol_lock_account_info.key)?;
    assert_owned_by(sol_lock_account_info, program_id)?;
    assert_initialized::<Account>(&sol_lock_account_info)?;

    sol_lock_account_info.with_mut_data(|mut sol_lock_account_data| {
        let now = Clock::get()?.unix_timestamp;

        if let Err(err) = assert_can_unlock(&sol_lock_account_data, now) {
            let premature_unlock: ProgramError = SolLockError::PrematureUnlock.into();
            if err == premature_unlock {
                msg!(
                    "Deadline: {}, Now: {}",
                    sol_lock_account_data.deadline.unwrap(),
                    now
                );
            }
            return Err(err);
        }

        let new_state = match sol_lock_account_data.state {
            State::Locked => State::Initialized,
            State::Uninitialized
            | State::Initialized
            | State::HasFunds
            | State::HasDeadline
            | State::ReadyUnlocked
            | State::Staked => unreachable!(),
        };

        let lamports_to_transfer = match sol_lock_account_data.state {
            State::Locked => {
                let lamports = sol_lock_account_data.lamports.take().unwrap();
                sol_lock_account_data.deadline = None;
                sol_lock_account_data.stake_account = None;
                lamports
            }
            State::Initialized
            | State::HasFunds
            | State::HasDeadline
            | State::ReadyUnlocked
            | State::Staked
            | State::Uninitialized => unreachable!(),
        };

        sol_lock_account_data.state = new_state;

        **sol_lock_account_info.try_borrow_mut_lamports()? -= lamports_to_transfer;
        **receiver_account_info.try_borrow_mut_lamports()? += lamports_to_transfer;

        Ok(sol_lock_account_data)
    })?;

    Ok(())
}
