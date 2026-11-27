//! Shared guards: instruction data, signatures, PDA checks, account creation and closing, lamport moves and the clock. Written in the shape of
//! poleis_roads' (src/util.rs), which this program does not depend on: a war's bug can never sit in another program.

use crate::error::WarsError;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, Address,
};

/// Read fixed-width fields out of instruction data.
pub struct Data<'a>(pub &'a [u8]);
impl<'a> Data<'a> {
    pub fn u32(&self, o: usize) -> Result<u32, ProgramError> {
        self.0.get(o..o + 4).and_then(|s| s.try_into().ok()).map(u32::from_le_bytes).ok_or(ProgramError::InvalidInstructionData)
    }
    pub fn u64(&self, o: usize) -> Result<u64, ProgramError> {
        self.0.get(o..o + 8).and_then(|s| s.try_into().ok()).map(u64::from_le_bytes).ok_or(ProgramError::InvalidInstructionData)
    }
    pub fn expect_len(&self, n: usize) -> Result<(), ProgramError> {
        if self.0.len() != n {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(())
    }
}

pub fn expect_signer(acc: &AccountView) -> Result<(), ProgramError> {
    if !acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

pub fn expect_writable(acc: &AccountView) -> Result<(), ProgramError> {
    if !acc.is_writable() {
        return Err(WarsError::InvalidAccount.into());
    }
    Ok(())
}

/// Verify `acc` is the PDA for `seeds` (bump appended) under this program.
pub fn expect_pda(acc: &AccountView, seeds: &[&[u8]], bump: u8) -> Result<(), ProgramError> {
    let mut with_bump: [&[u8]; 8] = [&[]; 8];
    let n = seeds.len();
    if n + 1 > with_bump.len() {
        return Err(WarsError::InvalidPda.into());
    }
    with_bump[..n].copy_from_slice(seeds);
    let bump_ref = [bump];
    with_bump[n] = &bump_ref;
    let derived = Address::create_program_address(&with_bump[..n + 1], &crate::ID).map_err(|_| WarsError::InvalidPda)?;
    if acc.address() != &derived {
        return Err(WarsError::InvalidPda.into());
    }
    Ok(())
}

/// Find the bump for `seeds`, verify `acc` matches, and return the bump.
pub fn find_and_expect_pda(acc: &AccountView, seeds: &[&[u8]]) -> Result<u8, ProgramError> {
    let (derived, bump) = Address::find_program_address(seeds, &crate::ID);
    if acc.address() != &derived {
        return Err(WarsError::InvalidPda.into());
    }
    Ok(bump)
}

/// Whether an account this program would create already stands (has data or is ours).
pub fn stands(acc: &AccountView) -> bool {
    acc.data_len() != 0 || acc.owned_by(&crate::ID)
}

/// The rent of an account of `space` bytes, the cluster's.
pub fn rent_of(space: usize) -> Result<u64, ProgramError> {
    Rent::get()?.try_minimum_balance(space)
}

/// Create a program-owned PDA account, robust to a pre-funded address: if the account already holds lamports, top up + allocate + assign
/// instead of create (an attacker donating dust must not brick account creation). Fails if the account already has data or is ours: creation
/// is init-once. Returns what the account held beyond its rent once created (a donation before its birth; the caller books it).
pub fn create_pda(payer: &AccountView, new_acc: &AccountView, space: usize, seeds: &[&[u8]], bump: u8) -> Result<u64, ProgramError> {
    if stands(new_acc) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let rent = rent_of(space)?;
    let bump_ref = [bump];
    let mut seed_arr: [Seed; 8] = core::array::from_fn(|_| Seed::from(&[] as &[u8]));
    let n = seeds.len();
    if n + 1 > seed_arr.len() {
        return Err(WarsError::InvalidPda.into());
    }
    for (i, s) in seeds.iter().enumerate() {
        seed_arr[i] = Seed::from(*s);
    }
    seed_arr[n] = Seed::from(&bump_ref[..]);
    if new_acc.lamports() == 0 {
        pinocchio_system::instructions::CreateAccount { from: payer, to: new_acc, lamports: rent, space: space as u64, owner: &crate::ID }
            .invoke_signed(&[Signer::from(&seed_arr[..n + 1])])?;
    } else {
        let top_up = rent.saturating_sub(new_acc.lamports());
        if top_up > 0 {
            pinocchio_system::instructions::Transfer { from: payer, to: new_acc, lamports: top_up }.invoke()?;
        }
        pinocchio_system::instructions::Allocate { account: new_acc, space: space as u64 }.invoke_signed(&[Signer::from(&seed_arr[..n + 1])])?;
        pinocchio_system::instructions::Assign { account: new_acc, owner: &crate::ID }.invoke_signed(&[Signer::from(&seed_arr[..n + 1])])?;
    }
    Ok(new_acc.lamports().saturating_sub(rent))
}

/// Lamports from a signer's wallet into one of this program's accounts, by the system program.
pub fn pay_in(from: &AccountView, to: &AccountView, lamports: u64) -> Result<(), ProgramError> {
    if lamports == 0 {
        return Ok(());
    }
    pinocchio_system::instructions::Transfer { from, to, lamports }.invoke()
}

/// Lamports out of one of this program's accounts into another account, moved directly (the program owns `from`). Never between one account
/// and itself.
pub fn move_lamports(from: &mut AccountView, to: &mut AccountView, lamports: u64) -> Result<(), ProgramError> {
    if lamports == 0 {
        return Ok(());
    }
    if !from.owned_by(&crate::ID) || from.address() == to.address() {
        return Err(WarsError::InvalidAccount.into());
    }
    let left = from.lamports().checked_sub(lamports).ok_or(WarsError::MathOverflow)?;
    let to_after = to.lamports().checked_add(lamports).ok_or(WarsError::MathOverflow)?;
    from.set_lamports(left);
    to.set_lamports(to_after);
    Ok(())
}

/// Close a program-owned account and send every lamport it holds to `dest`. The runtime zeroes the data at the end of the instruction. Only
/// ever called on accounts the law has finished with.
pub fn close_to(acc: &mut AccountView, dest: &mut AccountView) -> Result<(), ProgramError> {
    if !acc.owned_by(&crate::ID) || acc.address() == dest.address() {
        return Err(WarsError::InvalidAccount.into());
    }
    let refund = acc.lamports();
    let dest_after = dest.lamports().checked_add(refund).ok_or(WarsError::MathOverflow)?;
    acc.set_lamports(0);
    dest.set_lamports(dest_after);
    acc.close()
}

/// The clock: the chain's unix timestamp, in seconds.
pub fn now_ts() -> Result<i64, ProgramError> {
    Ok(Clock::get()?.unix_timestamp)
}

/// A law amount as a stored one, checked.
pub fn u64_of(x: u128) -> Result<u64, ProgramError> {
    u64::try_from(x).map_err(|_| WarsError::MathOverflow.into())
}
