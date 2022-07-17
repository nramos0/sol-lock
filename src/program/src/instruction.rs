//! The definitions for SolLock instructions

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::UnixTimestamp;

/// CreateAccount instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct CreateAccount {
    /// The index of the new account
    pub acc_index: u64,
}

/// AddSol instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct AddSol {
    /// The index of the account to access
    pub acc_index: u64,
    /// The number of lamports to lock
    pub lamports: u64,
    /// Whether a Sol Payer account was passed
    pub has_payer: bool,
}

/// RemoveSol instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct RemoveSol {
    /// The index of the account to access
    pub acc_index: u64,
    /// The number of lamports to lock
    pub lamports: u64,
    /// Whether a Sol Receiver account was passed
    pub has_receiver: bool,
}

/// SetDeadline instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct SetDeadline {
    /// The index of the account to access
    pub acc_index: u64,
    /// The deadline to use
    pub deadline: UnixTimestamp,
}

/// Lock instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Lock {
    /// The index of the account to access
    pub acc_index: u64,
}

/// Unlock instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Unlock {
    /// The index of the account to access
    pub acc_index: u64,
    /// Whether a Sol Receiver account was passed
    pub has_receiver: bool,
}

/// Stake instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Stake {
    /// The index of the account to access
    pub acc_index: u64,
}

/// Unstake instruction data
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Unstake {
    /// The index of the account to access
    pub acc_index: u64,
}

/// A SolLock instruction
#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum SolLockInstruction {
    /// Create a new SolLock account
    /// Requires that the account does not exist.
    /// Creates the account with uninitialized data
    ///
    /// # Account references
    ///   0. `[SIGNER]` Owner account
    ///   1. `[WRITE]` SolLock account
    ///   2. `[]` System program account
    CreateAccount(CreateAccount),

    /// Add Sol to a SolLock account to prepare for locking
    /// Requires that the account is in one of states {Uninitialized, HasFunds, HasDeadline, ReadyUnlocked, Locked}
    /// That is, this instruction can be used in any state.
    ///
    ///
    /// Transitions:
    /// Uninitialized -> HasFunds
    /// HasFunds -> HasFunds
    /// HasDeadline -> ReadyUnlocked
    /// ReadyUnlocked -> ReadyUnlocked
    /// Locked -> Locked
    ///
    /// By default, the lamports will be debited from the Owner account.
    /// If a Sol Payer account is passed, the lamports will be debited from it instead.
    ///
    /// # Account references
    ///   0. `[SIGNER, WRITE]` Owner account
    ///   1. `[WRITE]` SolLock account
    ///   2. `[]` System program account
    ///   3. `[SIGNER, WRITE]` (Optional) Sol Payer account
    AddSol(AddSol),

    /// Remove Sol from an unlocked SolLock account
    /// Requires that the account is in one of states {HasFunds, ReadyUnlocked}
    /// Requires that the number of lamports to remove is less than or equal to the number in the account
    ///
    /// Transitions:
    /// HasFunds -> HasFunds
    ///     Occurs when the number of lamports to remove is less than the number in the account
    /// HasFunds -> Uninitialized
    ///     Occurs when the number of lamports to remove is equal to the number in the account
    /// ReadyUnlocked -> ReadyUnlocked
    ///     Occurs when the number of lamports to remove is less than the number in the account
    /// ReadyUnlocked -> HasDeadline
    ///     Occurs when the number of lamports to remove is equal to the number in the account
    ///
    /// If a Sol Receiver account is passed and has_receiver is true,
    /// the lamports will be credited to the Sol Receiver account instead.
    ///
    /// # Account references
    ///   0. `[SIGNER, WRITE]` Owner account
    ///   1. `[WRITE]` SolLock account
    ///   2. `[WRITE]` (Optional) Sol Receiver account
    RemoveSol(RemoveSol),

    /// Set deadline on a SolLock account
    /// Requires that the account is in one of states {Uninitialized, HasFunds, HasDeadline, ReadyUnlocked, Locked}
    /// That is, this instruction can be used in any state.
    ///
    /// Transitions:
    /// Uninitialized -> HasDeadline
    /// HasFunds -> ReadyUnlocked
    /// HasDeadline -> HasDeadline
    /// ReadyUnlocked -> ReadyUnlocked
    /// Locked -> Locked
    ///     Requires that the new deadline is greater than or equal to the current deadline
    ///
    /// # Account references
    ///   0. `[SIGNER]` Owner account
    ///   1. `[WRITE]` SolLock account
    SetDeadline(SetDeadline),

    /// Lock a SolLock account
    /// Requires that the account is in the state ReadyUnlocked
    ///
    /// Transitions:
    /// ReadyUnlocked -> Locked
    ///
    /// # Account references
    ///   0. `[SIGNER]` Owner account
    ///   1. `[WRITE]` SolLock account
    Lock(Lock),

    /// Unlock a SolLock account
    /// Requires that the account is in state Locked
    ///
    /// Transitions:
    /// Locked -> Uninitialized
    ///     Requires that the current time is greater than or equal to the deadline.
    ///     Transfers the lamports out of the SolLock account into the owner account
    ///     or the Sol Receiver account and sets the SolLock account's state to Uninitialized.
    ///
    /// By default, the lamports will be credited to the Owner account.
    /// If a Sol Receiver account is passed and has_receiver is true,
    /// the lamports will be credited to the Sol Receiver account instead.
    ///
    /// # Account references
    ///   0. `[SIGNER, WRITE]` Owner account
    ///   1. `[WRITE]` SolLock account
    ///   2. `[WRITE]` (Optional) Sol Receiver account
    Unlock(Unlock),

    /// Stake the funds in a SolLock account
    /// Requires that the account is in state Locked
    /// Requires that the Stake program account is not currently staked
    ///
    /// Transitions:
    /// Locked -> Staked
    ///
    /// # Account references
    ///   0. `[SIGNER]` Owner account
    ///   1. `[WRITE]` SolLock account
    ///   2. `[]` Stake program account
    Stake(Stake),

    /// Unstake the funds in a SolLock account
    /// Requires that the account is in state Staked
    /// Requires that the Stake program account is not currently staked
    ///
    /// Transitions:
    /// Staked -> Locked
    ///
    /// # Account references
    ///   0. `[SIGNER]` Owner account
    ///   1. `[WRITE]` SolLock account
    ///   2. `[]` Stake program account
    Unstake(Unstake),
}
