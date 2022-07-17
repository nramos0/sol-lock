use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_memory::{sol_memcpy, sol_memset},
    program_pack::Pack,
};

use crate::{error::SolLockError, state::Account};

pub fn unpack_option<T>(
    src: &[u8],
    build_fn: impl Fn(&[u8]) -> T,
) -> Result<Option<T>, ProgramError> {
    Ok(match src[0] {
        0 => None,
        1 => Some(build_fn(&src[1..])),
        _ => return Err(SolLockError::UnpackError.into()),
    })
}

pub fn pack_option<'a, 'b, T>(
    opt: &'a Option<T>,
    dst: &mut [u8],
    build_fn: impl FnOnce(&'a T) -> &'b [u8],
    size: usize,
) {
    dst[0] = opt.is_some() as u8;

    match opt {
        Some(val) => {
            let bytes = build_fn(val);
            sol_memcpy(&mut dst[1..], bytes, size);
        }
        None => {
            sol_memset(&mut dst[1..], 0, size);
        }
    }
}

pub trait WithData<T> {
    fn with_immut_data(&self, f: impl FnOnce(T) -> ProgramResult) -> ProgramResult;
    fn with_mut_data(&self, f: impl FnOnce(T) -> Result<T, ProgramError>) -> ProgramResult;
}

impl WithData<Account> for AccountInfo<'_> {
    fn with_immut_data(&self, f: impl FnOnce(Account) -> ProgramResult) -> ProgramResult {
        let sol_lock_account_data = Account::unpack(&self.data.borrow())?;
        f(sol_lock_account_data)?;
        Ok(())
    }

    fn with_mut_data(
        &self,
        f: impl FnOnce(Account) -> Result<Account, ProgramError>,
    ) -> ProgramResult {
        let sol_lock_account_data = Account::unpack(&self.data.borrow())?;
        let sol_lock_account_data = f(sol_lock_account_data)?;
        sol_lock_account_data.pack_into_slice(&mut self.data.borrow_mut());
        Ok(())
    }
}
