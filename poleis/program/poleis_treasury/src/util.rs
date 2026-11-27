//! Shared guards: PDA checks, account creation, lamport movement, hashing.
//! Every check the old framework generated is written here by hand, once.

use crate::error::PoleisError;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address,
};

/// Read fixed-width fields out of instruction data.
pub struct Data<'a>(pub &'a [u8]);
impl<'a> Data<'a> {
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
        return Err(PoleisError::InvalidAccount.into());
    }
    Ok(())
}

/// Verify `acc` is the PDA for `seeds` (bump appended) under this program.
pub fn expect_pda(acc: &AccountView, seeds: &[&[u8]], bump: u8) -> Result<(), ProgramError> {
    let mut with_bump: [&[u8]; 8] = [&[]; 8];
    let n = seeds.len();
    if n + 1 > with_bump.len() {
        return Err(PoleisError::InvalidPda.into());
    }
    with_bump[..n].copy_from_slice(seeds);
    let bump_ref = [bump];
    with_bump[n] = &bump_ref;
    let derived = Address::create_program_address(&with_bump[..n + 1], &crate::ID)
        .map_err(|_| PoleisError::InvalidPda)?;
    if acc.address() != &derived {
        return Err(PoleisError::InvalidPda.into());
    }
    Ok(())
}

/// Find the bump for `seeds`, verify `acc` matches, and return the bump.
pub fn find_and_expect_pda(acc: &AccountView, seeds: &[&[u8]]) -> Result<u8, ProgramError> {
    let (derived, bump) = Address::find_program_address(seeds, &crate::ID);
    if acc.address() != &derived {
        return Err(PoleisError::InvalidPda.into());
    }
    Ok(bump)
}

/// Create a program-owned PDA account, robust to a pre-funded address:
/// if the account already holds lamports, top up + allocate + assign instead
/// of create (an attacker donating dust must not brick account creation).
/// Fails if the account already has data or a non-system owner: creation is
/// init-once by construction.
pub fn create_pda(
    payer: &AccountView,
    new_acc: &AccountView,
    space: usize,
    seeds: &[&[u8]],
    bump: u8,
) -> Result<(), ProgramError> {
    if new_acc.data_len() != 0 || new_acc.owned_by(&crate::ID) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let rent = Rent::get()?.try_minimum_balance(space)?;

    let bump_ref = [bump];
    let mut seed_arr: [Seed; 8] = core::array::from_fn(|_| Seed::from(&[] as &[u8]));
    let n = seeds.len();
    if n + 1 > seed_arr.len() {
        return Err(PoleisError::InvalidPda.into());
    }
    for (i, s) in seeds.iter().enumerate() {
        seed_arr[i] = Seed::from(*s);
    }
    seed_arr[n] = Seed::from(&bump_ref[..]);

    if new_acc.lamports() == 0 {
        pinocchio_system::instructions::CreateAccount {
            from: payer,
            to: new_acc,
            lamports: rent,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[Signer::from(&seed_arr[..n + 1])])?;
    } else {
        let top_up = rent.saturating_sub(new_acc.lamports());
        if top_up > 0 {
            pinocchio_system::instructions::Transfer {
                from: payer,
                to: new_acc,
                lamports: top_up,
            }
            .invoke()?;
        }
        pinocchio_system::instructions::Allocate {
            account: new_acc,
            space: space as u64,
        }
        .invoke_signed(&[Signer::from(&seed_arr[..n + 1])])?;
        pinocchio_system::instructions::Assign {
            account: new_acc,
            owner: &crate::ID,
        }
        .invoke_signed(&[Signer::from(&seed_arr[..n + 1])])?;
    }
    Ok(())
}

/// Grow a program-owned account to `new_len`, the payer funding the rent
/// the extra bytes need. A registry grows one holder row at a time this
/// way: the floor pays for its own rows, and nothing is born at full size.
/// The top-up is a system transfer, so the caller must invoke this before
/// it moves any lamports by hand in the same instruction: the runtime
/// syncs only the callee's accounts across a CPI, and hand-moved lamports
/// on other accounts would leave that boundary unbalanced.
pub fn grow_account(payer: &AccountView, acc: &mut AccountView, new_len: usize) -> Result<(), ProgramError> {
    use pinocchio::Resize;
    if new_len <= acc.data_len() {
        return Ok(());
    }
    let need = Rent::get()?.try_minimum_balance(new_len)?;
    let top_up = need.saturating_sub(acc.lamports());
    if top_up > 0 {
        pinocchio_system::instructions::Transfer {
            from: payer,
            to: acc,
            lamports: top_up,
        }
        .invoke()?;
    }
    acc.resize(new_len)
}

/// Grow an account with the rent it needs moved out of an account this
/// program owns (an escrow), never by a system transfer: for the row a
/// polis seats from a knock's deposit, where no signer pays.
pub fn grow_account_from(source: &mut AccountView, acc: &mut AccountView, new_len: usize) -> Result<(), ProgramError> {
    use pinocchio::Resize;
    if new_len <= acc.data_len() {
        return Ok(());
    }
    let need = Rent::get()?.try_minimum_balance(new_len)?;
    let top_up = need.saturating_sub(acc.lamports());
    if top_up > 0 {
        move_lamports(source, acc, top_up)?;
    }
    acc.resize(new_len)
}

/// Move lamports out of a program-owned account into any account.
/// The debit must not push the source below its own rent-exempt minimum.
pub fn move_lamports(
    source: &mut AccountView,
    dest: &mut AccountView,
    amount: u64,
) -> Result<(), ProgramError> {
    // the same account passed twice would read its balance before either
    // write and mint the amount out of nothing: refused here, before the
    // runtime's sum check could refuse the whole transaction
    if source.address() == dest.address() {
        return Err(PoleisError::InvalidAccount.into());
    }
    let floor = Rent::get()?.try_minimum_balance(source.data_len())?;
    let after = source
        .lamports()
        .checked_sub(amount)
        .ok_or(PoleisError::TreasuryInsufficient)?;
    if after < floor {
        return Err(PoleisError::TreasuryInsufficient.into());
    }
    let dest_after = dest
        .lamports()
        .checked_add(amount)
        .ok_or(PoleisError::MathOverflow)?;
    source.set_lamports(after);
    dest.set_lamports(dest_after);
    Ok(())
}

/// Close a program-owned account and send every lamport it holds to `dest`.
/// The runtime zeroes the data at the end of the instruction. Only ever
/// called on accounts the law has finished with.
pub fn close_to(acc: &mut AccountView, dest: &mut AccountView) -> Result<(), ProgramError> {
    if !acc.owned_by(&crate::ID) || acc.address() == dest.address() {
        return Err(PoleisError::InvalidAccount.into());
    }
    let refund = acc.lamports();
    let dest_after = dest.lamports().checked_add(refund).ok_or(PoleisError::MathOverflow)?;
    acc.set_lamports(0);
    dest.set_lamports(dest_after);
    acc.close()
}

/// sha256 over a list of byte slices, via the runtime syscall.
pub fn sha256v(slices: &[&[u8]]) -> [u8; 32] {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    {
        let mut out = [0u8; 32];
        unsafe {
            pinocchio::syscalls::sol_sha256(
                slices.as_ptr() as *const u8,
                slices.len() as u64,
                out.as_mut_ptr(),
            );
        }
        out
    }
    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    {
        let _ = slices;
        unimplemented!("sha256 is computed on-chain only")
    }
}

/// The receipt: one structured line in the transaction log for every money
/// move, seal and claim, so that history and reputation are reads of the
/// record and never need an account. 125 bytes:
/// RECEIPT_DOMAIN ‖ kind:1 ‖ unit:8 ‖ hash:32 ‖ counterparty:32 ‖ mint:32 ‖ amount:8 ‖ outcome:1
const RD: usize = crate::RECEIPT_DOMAIN.len();
pub const RECEIPT_LEN: usize = RD + 1 + 8 + 32 + 32 + 32 + 8 + 1;
pub fn receipt(kind: u8, unit_id: u64, hash: &[u8; 32], counterparty: &[u8; 32], mint: &[u8; 32], amount: u64, outcome: u8) {
    let mut b = [0u8; RECEIPT_LEN];
    b[..RD].copy_from_slice(crate::RECEIPT_DOMAIN);
    b[RD] = kind;
    b[RD + 1..RD + 9].copy_from_slice(&unit_id.to_le_bytes());
    b[RD + 9..RD + 41].copy_from_slice(hash);
    b[RD + 41..RD + 73].copy_from_slice(counterparty);
    b[RD + 73..RD + 105].copy_from_slice(mint);
    b[RD + 105..RD + 113].copy_from_slice(&amount.to_le_bytes());
    b[RD + 113] = outcome;
    log_data(&[&b]);
}

/// `sol_log_data`: the bytes land in the log as "Program data: <base64>".
pub fn log_data(slices: &[&[u8]]) {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    unsafe {
        pinocchio::syscalls::sol_log_data(slices.as_ptr() as *const u8, slices.len() as u64);
    }
    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    {
        let _ = slices;
    }
}

/// The vote envelope a holder signs off-chain:
/// VOTE_DOMAIN ‖ proposal hash ‖ unit ‖ choice (55 bytes). The voter is not
/// in the message: the signature binds the key, and every voter of one
/// choice signs the same bytes, so a batch carries the message once and an
/// aggregate signature can carry it later.
pub const VOTE_DOMAIN: &[u8] = b"poleis:vote:v1";
const VD: usize = VOTE_DOMAIN.len();
pub const ENVELOPE_LEN: usize = VD + 32 + 8 + 1;

pub fn envelope(hash: &[u8; 32], unit_id: u64, choice: u8) -> [u8; ENVELOPE_LEN] {
    let mut e = [0u8; ENVELOPE_LEN];
    e[..VD].copy_from_slice(VOTE_DOMAIN);
    e[VD..VD + 32].copy_from_slice(hash);
    e[VD + 32..VD + 40].copy_from_slice(&unit_id.to_le_bytes());
    e[VD + 40] = choice;
    e
}
