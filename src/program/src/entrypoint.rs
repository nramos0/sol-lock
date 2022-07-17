//! Program entrypoint

#![cfg(not(feature = "no-entrypoint"))]

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};

use crate::{error::SolLockError, processor};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(err) = processor::process_instruction(program_id, accounts, instruction_data) {
        err.print::<SolLockError>();
        Err(err)
    } else {
        Ok(())
    }
}
