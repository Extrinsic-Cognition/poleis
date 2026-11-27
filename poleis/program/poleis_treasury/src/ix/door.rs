//! The door: a polis that grows by admission. A climb of the ladder in
//! door mode puts the new spots in the pool instead of a sealed sale.
//! Whoever wants in knocks: the price goes into escrow under its own key,
//! with the rent of one holder row on top. The polis admits or refuses
//! each knock by its double seal, one decision each. Admitted, the deposit
//! is the seat's price: the fund's share, the rest to the treasury, the
//! seat to the key, the rent home. Refused, or kept waiting a CODEX
//! window, the deposit goes home whole.
//!
//! The chain is touched when money moves or a seal lands: a knock, an
//! admission, a refusal, a withdrawal. Who is worth a seat is decided in
//! the polis's own channel, and only the verdict lands here.

use crate::error::PoleisError;
use crate::state::{Knock, Pool, Registry, Treasury};
use crate::util::{
    close_to, create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda, grow_account_from,
    move_lamports, receipt, Data,
};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, ProgramResult,
};

pub const POOL_SEED: &[u8] = b"pool";
pub const KNOCK_SEED: &[u8] = b"knock";
const ZERO: [u8; 32] = [0u8; 32];

fn read_pool(acc: &AccountView, unit: u64) -> Result<Pool, ProgramError> {
    let p = Pool::read(acc)?;
    if p.unit_id != unit {
        return Err(PoleisError::UnitMismatch.into());
    }
    expect_pda(acc, &[POOL_SEED, &unit.to_le_bytes()], p.bump)?;
    Ok(p)
}

/// A knock, or NotKnocking when the account was never opened or is closed.
fn read_knock(acc: &AccountView, unit: u64, key: &[u8; 32]) -> Result<Knock, ProgramError> {
    if acc.data_len() == 0 {
        return Err(PoleisError::NotKnocking.into());
    }
    let k = Knock::read(acc)?;
    if k.unit_id != unit || k.key != *key {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(acc, &[KNOCK_SEED, &unit.to_le_bytes(), key], k.bump)?;
    Ok(k)
}

fn treasury_of(acc: &AccountView, unit_id: u64) -> Result<Treasury, ProgramError> {
    let t = Treasury::read(acc)?;
    if t.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    expect_pda(acc, &[b"treasury", &unit_id.to_le_bytes()], t.bump)?;
    Ok(t)
}

/// The ladder in door mode, from the CODEX executor: the increment goes to
/// the pool, at `price` a seat. A later climb adds to the pool and resets
/// the price; a knock already waiting keeps the price it deposited.
pub fn open_door(
    crank: &AccountView,
    pool: &mut AccountView,
    unit: u64,
    new_seats: u64,
    price: u64,
) -> Result<(), ProgramError> {
    expect_writable(pool)?;
    let unit_le = unit.to_le_bytes();
    let seeds: &[&[u8]] = &[POOL_SEED, &unit_le];
    let mut p = if pool.data_len() == 0 {
        let bump = find_and_expect_pda(pool, seeds)?;
        create_pda(crank, pool, Pool::SIZE, seeds, bump)?;
        Pool { bump, unit_id: unit, open: 0, filled: 0, price }
    } else {
        read_pool(pool, unit)?
    };
    p.open = p.open.checked_add(new_seats).ok_or(PoleisError::MathOverflow)?;
    p.price = price;
    p.write(pool)
}

/// data: [unit:8] · accounts: [knocker (s,w), registry, pool, knock (w), system]
/// Anyone, while the pool has a spot. The price and the rent of one holder
/// row go into escrow under the knocker's key; one knock per key per polis.
pub fn knock(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [knocker, registry, pool, knock, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let unit = d.u64(0)?;
    expect_signer(knocker)?;
    expect_writable(knocker)?;
    expect_writable(knock)?;
    let head = super::read_registry(registry, unit)?;
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }
    let p = read_pool(pool, unit)?;
    if p.filled >= p.open {
        return Err(PoleisError::NoSpots.into());
    }
    if knock.data_len() != 0 {
        return Err(PoleisError::KnockOpen.into());
    }
    let key: [u8; 32] = *knocker.address().as_array();
    let unit_le = unit.to_le_bytes();
    let seeds: &[&[u8]] = &[KNOCK_SEED, &unit_le, &key];
    let bump = find_and_expect_pda(knock, seeds)?;
    create_pda(knocker, knock, Knock::SIZE, seeds, bump)?;
    // the row an admission may add to the registry is paid from this escrow,
    // never by the crank that executes the seal; what is not needed goes home
    let rent = Rent::get()?;
    let size = registry.data_len();
    let row = rent
        .try_minimum_balance(size + Registry::HOLDER_LEN)?
        .saturating_sub(rent.try_minimum_balance(size)?);
    let escrow = p.price.checked_add(row).ok_or(PoleisError::MathOverflow)?;
    pinocchio_system::instructions::Transfer { from: knocker, to: knock, lamports: escrow }.invoke()?;
    let now = Clock::get()?.slot;
    Knock { bump, unit_id: unit, key, deposit: p.price, slot: now }.write(knock)?;
    receipt(crate::RCPT_ESCROW, unit, &ZERO, &key, &ZERO, p.price, 0);
    Ok(())
}

/// data: [unit:8] · accounts: [knocker (s,w), registry, knock (w)]
/// The knocker takes its deposit back once the polis has kept it waiting a
/// CODEX window without a verdict, or at once from a dissolved polis.
pub fn withdraw_knock(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [knocker, registry, knock] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let unit = d.u64(0)?;
    expect_signer(knocker)?;
    expect_writable(knocker)?;
    expect_writable(knock)?;
    let head = super::read_registry(registry, unit)?;
    let key: [u8; 32] = *knocker.address().as_array();
    let k = read_knock(knock, unit, &key)?;
    let now = Clock::get()?.slot;
    if !head.frozen && now < k.slot.saturating_add(head.codex_window) {
        return Err(PoleisError::NotYet.into());
    }
    receipt(crate::RCPT_REFUND, unit, &ZERO, &key, &ZERO, k.deposit, 0);
    close_to(knock, knocker)
}

/// fields: [holder:32]
/// accounts: [registry (w), treasury (w), pool (w), knock (w), holder (w), fund (w), tally (w), system]
/// The polis admits a knocker: one seat at the price it deposited, the
/// fund's share taken, the rest to the treasury, the knock closed to the
/// holder with the rent it does not need.
pub fn admit(
    crank: &AccountView,
    rest: &mut [AccountView],
    origin: u64,
    fields: &[u8],
    hash: &[u8; 32],
    now: u64,
) -> Result<(), ProgramError> {
    let f = Data(fields);
    f.expect_len(32)?;
    let holder = f.h32(0)?;
    let [registry, treasury, pool, knock, payee, fund, tally, _system, ..] = rest else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let head = super::read_registry(registry, origin)?;
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }
    treasury_of(treasury, origin)?;
    let mut p = read_pool(pool, origin)?;
    if p.filled >= p.open {
        return Err(PoleisError::NoSpots.into());
    }
    let k = read_knock(knock, origin, &holder)?;
    if payee.address().as_array() != &holder {
        return Err(PoleisError::InvalidAccount.into());
    }
    let price = k.deposit;
    // the fund's share first: its tally may be born here by CPI, and a CPI
    // must precede any lamports this program moves by hand
    let fee = super::fund::charge_fee(fund, tally, crank, origin, head.root, price)?;
    // the seat: a new holder may grow the registry by a row, out of the escrow
    Registry::add_seats_with(registry, crate::HOLDER_KEY, &holder, 1, now, |acc, size| {
        grow_account_from(knock, acc, size)
    })?;
    if fee > 0 {
        move_lamports(knock, fund, fee)?;
    }
    move_lamports(knock, treasury, price - fee)?;
    Treasury::add_in(treasury, price - fee)?;
    p.filled = p.filled.checked_add(1).ok_or(PoleisError::MathOverflow)?;
    p.write(pool)?;
    receipt(crate::RCPT_ADMIT, origin, hash, &holder, &ZERO, price, 1);
    close_to(knock, payee)
}

/// fields: [holder:32] · accounts: [registry, knock (w), holder (w)]
/// The polis refuses a knocker: the deposit goes home whole, with the rent.
pub fn refuse(rest: &mut [AccountView], origin: u64, fields: &[u8], hash: &[u8; 32]) -> Result<(), ProgramError> {
    let f = Data(fields);
    f.expect_len(32)?;
    let holder = f.h32(0)?;
    let [registry, knock, payee, ..] = rest else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    super::read_registry(registry, origin)?;
    let k = read_knock(knock, origin, &holder)?;
    if payee.address().as_array() != &holder {
        return Err(PoleisError::InvalidAccount.into());
    }
    receipt(crate::RCPT_ADMIT, origin, hash, &holder, &ZERO, k.deposit, 0);
    close_to(knock, payee)
}
