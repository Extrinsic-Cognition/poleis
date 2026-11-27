//! Treasuries: one vault per unit. Money enters freely (endow); it leaves
//! only through salary claims and anchored transfers.

use crate::error::PoleisError;
use crate::state::{Config, Treasury};
use crate::util::{create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda, Data};
use pinocchio::{
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};

/// data: [root:8][default_mint:32] · accounts: [payer (s,w), config (w), system], and for tree 0 [Poleis's program data] after them
/// One config per tree, seeded by the id its root unit will carry; the
/// first to take a root id owns its founding. Permissionless: one program
/// hosts every polis. ONLY THE DEPLOYER FOUNDS ROOT 0: tree 0's root passes
/// the code's upgrade and the fund's constitution, so its founding is the
/// payer's only when Poleis's own program data names that payer as its
/// upgrade authority (the deployer, between the deploy and the handoff).
/// The default mint (USDC on a real cluster) is named
/// here once; every unit of the tree inherits it and spends it under its
/// own token counter without a listing. Zero: no default, every mint must
/// be allowed by CODEX.
pub fn init_config(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [payer, config, _system, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(40)?;
    let root = d.u64(0)?;
    let default_mint = d.h32(8)?;
    expect_signer(payer)?;
    if root == crate::ROOT_UNIT {
        let [program_data, ..] = rest else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        super::upgrade::expect_founder(program_data, payer)?;
    }
    let root_le = root.to_le_bytes();
    let seeds: &[&[u8]] = &[b"config", &root_le];
    let bump = find_and_expect_pda(config, seeds)?;
    create_pda(payer, config, Config::SIZE, seeds, bump)?;
    let authority = *payer.address();
    Config::init(config, &authority, bump, &default_mint, root)
}

/// data: [unit_id:8] · accounts: [authority (s,w), config, treasury (w), system]
/// Solo mode only: under a council, units are born by verdict.
pub fn create_treasury(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, config, treasury, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let unit_id = d.u64(0)?;
    super::expect_solo_authority(config, authority)?;

    let unit_le = unit_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"treasury", &unit_le];
    let bump = find_and_expect_pda(treasury, seeds)?;
    create_pda(authority, treasury, Treasury::SIZE, seeds, bump)?;
    Treasury::init(treasury, unit_id, bump)
}

/// data: [unit_id:8][amount:8] · accounts: [donor (s,w), treasury (w), system]
/// Permissionless: anyone may endow any unit's treasury.
pub fn endow(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [donor, treasury, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(16)?;
    let unit_id = d.u64(0)?;
    let amount = d.u64(8)?;
    if amount == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    expect_signer(donor)?;
    expect_writable(treasury)?;

    let t = Treasury::read(treasury)?;
    if t.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    let unit_le = unit_id.to_le_bytes();
    expect_pda(treasury, &[b"treasury", &unit_le], t.bump)?;

    pinocchio_system::instructions::Transfer {
        from: donor,
        to: treasury,
        lamports: amount,
    }
    .invoke()?;
    Treasury::add_in(treasury, amount)
}

/// data: [unit_id:8] · accounts: [treasury (w)]
/// Anyone can wire lamports straight to a treasury address, bypassing endow:
/// those lamports are spendable (solvency reads the balance) but the books
/// never saw them. Anyone may true the books: fold the surplus into total_in
/// so conservation holds as equality again. Moves nothing, needs no signer,
/// idempotent: and it also rights the ledger after a donation was spent
/// before anyone synced (total_out briefly above total_in).
pub fn sync_endowment(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [treasury] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let unit_id = d.u64(0)?;
    expect_writable(treasury)?;

    let t = Treasury::read(treasury)?;
    if t.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    let unit_le = unit_id.to_le_bytes();
    expect_pda(treasury, &[b"treasury", &unit_le], t.bump)?;

    let floor = Rent::get()?.try_minimum_balance(treasury.data_len())?;
    let free = treasury.lamports().saturating_sub(floor);
    // the books must say: everything ever paid out, plus everything still here,
    // came in. target_in ≥ total_in always; the difference is the donation.
    let target_in = t.total_out.checked_add(free).ok_or(PoleisError::MathOverflow)?;
    if target_in > t.total_in {
        Treasury::set_in(treasury, target_in)?;
    }
    Ok(())
}
