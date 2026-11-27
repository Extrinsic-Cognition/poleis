//! Shared guards: PDA checks, account creation, closing, the clock. Written
//! here in the shape of Poleis's (poleis_treasury/src/util.rs): this program
//! cannot depend on the Poleis crate, so a market's bug can never sit in the
//! contract that guards a treasury.

use crate::error::RoadsError;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, Address,
};

/// Read fixed-width fields out of instruction data.
pub struct Data<'a>(pub &'a [u8]);
impl<'a> Data<'a> {
    pub fn u8(&self, o: usize) -> Result<u8, ProgramError> {
        self.0.get(o).copied().ok_or(ProgramError::InvalidInstructionData)
    }
    pub fn u32(&self, o: usize) -> Result<u32, ProgramError> {
        self.0
            .get(o..o + 4)
            .and_then(|s| s.try_into().ok())
            .map(u32::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)
    }
    pub fn u64(&self, o: usize) -> Result<u64, ProgramError> {
        self.0
            .get(o..o + 8)
            .and_then(|s| s.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)
    }
    pub fn h32(&self, o: usize) -> Result<[u8; 32], ProgramError> {
        self.0
            .get(o..o + 32)
            .and_then(|s| s.try_into().ok())
            .ok_or(ProgramError::InvalidInstructionData)
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
        return Err(RoadsError::InvalidAccount.into());
    }
    Ok(())
}

/// Verify `acc` is the PDA for `seeds` (bump appended) under this program.
pub fn expect_pda(acc: &AccountView, seeds: &[&[u8]], bump: u8) -> Result<(), ProgramError> {
    let mut with_bump: [&[u8]; 8] = [&[]; 8];
    let n = seeds.len();
    if n + 1 > with_bump.len() {
        return Err(RoadsError::InvalidPda.into());
    }
    with_bump[..n].copy_from_slice(seeds);
    let bump_ref = [bump];
    with_bump[n] = &bump_ref;
    let derived = Address::create_program_address(&with_bump[..n + 1], &crate::ID).map_err(|_| RoadsError::InvalidPda)?;
    if acc.address() != &derived {
        return Err(RoadsError::InvalidPda.into());
    }
    Ok(())
}

/// Find the bump for `seeds`, verify `acc` matches, and return the bump.
pub fn find_and_expect_pda(acc: &AccountView, seeds: &[&[u8]]) -> Result<u8, ProgramError> {
    let (derived, bump) = Address::find_program_address(seeds, &crate::ID);
    if acc.address() != &derived {
        return Err(RoadsError::InvalidPda.into());
    }
    Ok(bump)
}

/// Whether an account this program would create already stands (has data or is ours).
pub fn stands(acc: &AccountView) -> bool {
    acc.data_len() != 0 || acc.owned_by(&crate::ID)
}

/// Create a program-owned PDA account, robust to a pre-funded address: if
/// the account already holds lamports, top up + allocate + assign instead
/// of create (an attacker donating dust must not brick account creation).
/// Fails if the account already has data or is ours: creation is init-once.
pub fn create_pda(payer: &AccountView, new_acc: &AccountView, space: usize, seeds: &[&[u8]], bump: u8) -> Result<(), ProgramError> {
    if stands(new_acc) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let rent = Rent::get()?.try_minimum_balance(space)?;

    let bump_ref = [bump];
    let mut seed_arr: [Seed; 8] = core::array::from_fn(|_| Seed::from(&[] as &[u8]));
    let n = seeds.len();
    if n + 1 > seed_arr.len() {
        return Err(RoadsError::InvalidPda.into());
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
    Ok(())
}

/// Close a program-owned account and send every lamport it holds to `dest`.
/// The runtime zeroes the data at the end of the instruction. Only ever
/// called on accounts the law has finished with.
pub fn close_to(acc: &mut AccountView, dest: &mut AccountView) -> Result<(), ProgramError> {
    if !acc.owned_by(&crate::ID) || acc.address() == dest.address() {
        return Err(RoadsError::InvalidAccount.into());
    }
    let refund = acc.lamports();
    let dest_after = dest.lamports().checked_add(refund).ok_or(RoadsError::MathOverflow)?;
    acc.set_lamports(0);
    dest.set_lamports(dest_after);
    acc.close()
}

/// The clock: the slot, as Poleis reads it (`Clock::get()?.slot`), the finest instant the chain orders trades by.
pub fn now_slot() -> Result<u64, ProgramError> {
    Ok(Clock::get()?.slot)
}

/// The clock in the law's milliseconds: the slot times the chain's nominal milliseconds a slot (`crate::SLOT_MS`).
pub fn now_ms() -> Result<u64, ProgramError> {
    ms_of(now_slot()?)
}

/// A count of slots in the law's milliseconds, checked.
pub fn ms_of(slots: u64) -> Result<u64, ProgramError> {
    slots.checked_mul(crate::SLOT_MS).ok_or(RoadsError::MathOverflow.into())
}
