//! Shared guards: reading instruction data, signers, PDA checks, account creation, the clock. Written here in the shape of the roads
//! program's (poleis_roads/src/util.rs) and Poleis's: this program depends on neither crate, so a bug of theirs can never sit in the
//! contract that holds $WARM's mint.

use crate::error::WarmError;
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
    pub fn u64(&self, o: usize) -> Result<u64, ProgramError> {
        self.0.get(o..o + 8).and_then(|s| s.try_into().ok()).map(u64::from_le_bytes).ok_or(ProgramError::InvalidInstructionData)
    }
    pub fn h32(&self, o: usize) -> Result<[u8; 32], ProgramError> {
        self.0.get(o..o + 32).and_then(|s| s.try_into().ok()).ok_or(ProgramError::InvalidInstructionData)
    }
    pub fn h64(&self, o: usize) -> Result<[u8; 64], ProgramError> {
        self.0.get(o..o + 64).and_then(|s| s.try_into().ok()).ok_or(ProgramError::InvalidInstructionData)
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
        return Err(WarmError::InvalidAccount.into());
    }
    Ok(())
}

/// Find the bump for `seeds`, verify `acc` is that PDA under this program, and return the bump. Only where an account is first made
/// (each search costs the chain's address syscall, about 1,500 units a bump tried); a standing account is checked by its stored bump.
pub fn find_and_expect_pda(acc: &AccountView, seeds: &[&[u8]]) -> Result<u8, ProgramError> {
    let (derived, bump) = Address::find_program_address(seeds, &crate::ID);
    if acc.address() != &derived {
        return Err(WarmError::InvalidPda.into());
    }
    Ok(bump)
}

/// Whether an account this program would make already stands (it has data, or is ours).
pub fn stands(acc: &AccountView) -> bool {
    acc.data_len() != 0 || acc.owned_by(&crate::ID)
}

/// Make a PDA of this program, `space` bytes, owned by `owner` (this program, or the token program for the mint and the vault), robust
/// to a pre-funded address: if the address already holds lamports, top up, allocate and assign instead of create (a donor's dust must not
/// brick it). Refuses an account that stands: making is once.
pub fn create_pda(payer: &AccountView, new_acc: &AccountView, space: usize, owner: &Address, seeds: &[&[u8]], bump: u8) -> Result<(), ProgramError> {
    if new_acc.data_len() != 0 || !new_acc.owned_by(&Address::default()) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let rent = Rent::get()?.try_minimum_balance(space)?;
    let bump_ref = [bump];
    let mut seed_arr: [Seed; 4] = core::array::from_fn(|_| Seed::from(&[] as &[u8]));
    let n = seeds.len();
    if n + 1 > seed_arr.len() {
        return Err(WarmError::InvalidPda.into());
    }
    for (i, s) in seeds.iter().enumerate() {
        seed_arr[i] = Seed::from(*s);
    }
    seed_arr[n] = Seed::from(&bump_ref[..]);
    let signer = [Signer::from(&seed_arr[..n + 1])];
    if new_acc.lamports() == 0 {
        pinocchio_system::instructions::CreateAccount { from: payer, to: new_acc, lamports: rent, space: space as u64, owner }.invoke_signed(&signer)?;
    } else {
        let top_up = rent.saturating_sub(new_acc.lamports());
        if top_up > 0 {
            pinocchio_system::instructions::Transfer { from: payer, to: new_acc, lamports: top_up }.invoke()?;
        }
        pinocchio_system::instructions::Allocate { account: new_acc, space: space as u64 }.invoke_signed(&signer)?;
        pinocchio_system::instructions::Assign { account: new_acc, owner }.invoke_signed(&signer)?;
    }
    Ok(())
}

/// The slot, as Poleis and the roads read their clock.
pub fn now_slot() -> Result<u64, ProgramError> {
    Ok(Clock::get()?.slot)
}

/// The chain's clock in the law's milliseconds: its `unix_timestamp` (seconds) times 1,000; an instant before the epoch reads as 0. A
/// spend is "never later than the chain's clock" when its `at_ms` is at most this.
pub fn now_ms() -> Result<u64, ProgramError> {
    let s = Clock::get()?.unix_timestamp;
    Ok(if s <= 0 { 0 } else { (s as u64).saturating_mul(1_000) })
}
