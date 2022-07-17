//! Program state
#![deny(missing_docs)]

use std::convert::TryInto;

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{
    clock::UnixTimestamp,
    program_error::ProgramError,
    program_memory::sol_memcpy,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::{error::SolLockError, pack_utils::unpack_option};

/// A SolLock account modeled as a state machine
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
#[repr(C)]
pub struct Account {
    // 32
    /// The owner of the account
    pub owner: Pubkey,
    // 8
    /// The number of lamports to lock
    pub lamports: Option<u64>,
    // 8
    /// The time the lamports should be locked until
    pub deadline: Option<UnixTimestamp>,
    // 32
    /// The stake account that lamports may be stored in while locked
    pub stake_account: Option<Pubkey>,
    // 1
    /// The account state
    pub state: State,
}

/// The size of a SolLock account
pub const SOL_LOCK_ACCOUNT_SIZE: usize =
    OWNER_LEN + LAMPORTS_LEN + DEADLINE_LEN + STAKE_ACC_LEN + STATE_LEN;

pub const OWNER_LEN: usize = 32;
pub const LAMPORTS_LEN: usize = 1 + 8;
pub const DEADLINE_LEN: usize = 1 + 8;
pub const STAKE_ACC_LEN: usize = 1 + 32;
pub const STATE_LEN: usize = 1;

impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.state != State::Uninitialized
    }
}

impl Sealed for Account {}
impl Pack for Account {
    const LEN: usize = SOL_LOCK_ACCOUNT_SIZE;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, SOL_LOCK_ACCOUNT_SIZE];

        let (owner_dst, lamports_dst, deadline_dst, stake_account_dst, state_dst) = mut_array_refs![
            dst,
            OWNER_LEN,
            LAMPORTS_LEN,
            DEADLINE_LEN,
            STAKE_ACC_LEN,
            STATE_LEN
        ];

        sol_memcpy(owner_dst, &self.owner.to_bytes()[..], 32);

        let bytes8_zero = [0; 8];
        let bytes32_zero = [0; 32];

        let mut lamports_bytes = [0; 8];
        lamports_dst[0] = self.lamports.is_some() as u8;
        sol_memcpy(
            &mut lamports_dst[1..],
            self.lamports.map_or(&bytes8_zero, |lamports| {
                lamports_bytes = lamports.to_le_bytes();
                &lamports_bytes
            }),
            8,
        );

        let mut deadline_bytes = [0; 8];
        deadline_dst[0] = self.deadline.is_some() as u8;
        sol_memcpy(
            &mut deadline_dst[1..],
            self.deadline.map_or(&bytes8_zero, |deadline| {
                deadline_bytes = deadline.to_le_bytes();
                &deadline_bytes
            }),
            8,
        );

        let mut stake_account_bytes = [0; 32];
        stake_account_dst[0] = self.stake_account.is_some() as u8;
        sol_memcpy(
            &mut stake_account_dst[1..],
            self.stake_account.map_or(&bytes32_zero, |stake_account| {
                stake_account_bytes = stake_account.to_bytes();
                &stake_account_bytes
            }),
            32,
        );

        state_dst[0] = self.state as u8;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, SOL_LOCK_ACCOUNT_SIZE];

        let (owner_src, lamports_src, deadline_src, stake_account_src, state_src) = array_refs![
            src,
            OWNER_LEN,
            LAMPORTS_LEN,
            DEADLINE_LEN,
            STAKE_ACC_LEN,
            STATE_LEN
        ];

        let owner = Pubkey::new(owner_src);

        let lamports = unpack_option(lamports_src, |src| {
            u64::from_le_bytes(src.try_into().unwrap())
        })?;

        let deadline = unpack_option(deadline_src, |src| {
            i64::from_le_bytes(src.try_into().unwrap())
        })?;

        let stake_account = unpack_option(stake_account_src, |src| Pubkey::new(src))?;

        let state_opt: Option<State> = num::FromPrimitive::from_u8(state_src[0]);
        if state_opt.is_none() {
            return Err(SolLockError::UnpackError.into());
        }
        let state = state_opt.unwrap();

        let account = Account {
            owner,
            lamports,
            deadline,
            stake_account,
            state,
        };

        Ok(account)
    }
}

/// Account state
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, FromPrimitive, Clone, Copy)]
#[repr(C)]
pub enum State {
    /// The account is not yet initialized
    Uninitialized,
    /// The account is initialized. State after account creation or after fund withdrawal
    Initialized,
    /// The account knows how many lamports to lock and has received them, but doesn't know how long to lock for
    HasFunds,
    /// The account knows how long to lock the lamports for, but doesn't know how many lamports to lock
    HasDeadline,
    /// The account is ready to be locked, but is currently unlocked
    ReadyUnlocked,
    /// The account is currently locked, and cannot leave this state until the current time is >= deadline (except if going to Stake)
    Locked,
    /// The account is locked and its funds have been sent to a Stake account to delegate to validators
    Staked,
}

impl Default for State {
    fn default() -> Self {
        State::Uninitialized
    }
}
