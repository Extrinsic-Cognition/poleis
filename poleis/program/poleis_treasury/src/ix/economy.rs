//! The economy: what a parliament and a bank still lack. Time and promises.
//!
//! An allowance is a standing authorization: a passed proposal lets a payee
//! pull up to a cap per period, each pull charged to the plan, no vote per
//! pull. A contract is escrow released by a ruling: the paying unit locks
//! the price until a deadline, and its own seal or the seal of an arbiter
//! unit releases or refunds it; the tree is the court. The dividend policy
//! is income by rule: a fraction of every inflow escrowed for the floor each
//! period, cranked by anyone. Every move leaves a receipt in the log.

use super::vault::{
    close_token_account, create_token_account, expect_escrow, expect_token_program, expect_vault,
    token_account_owner, transfer_from_vault,
};
use crate::error::PoleisError;
use crate::state::{holder_unit, Allowance, Contract, Distribution, Registry, TokenPlan, Treasury};
use crate::util::{
    close_to, create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda,
    move_lamports, receipt, sha256v, Data,
};
use pinocchio::{
    cpi::Seed,
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, ProgramResult,
};

pub const ZERO: [u8; 32] = [0u8; 32];

pub fn read_allowance(acc: &AccountView, hash: &[u8; 32]) -> Result<Allowance, ProgramError> {
    let a = Allowance::read(acc)?;
    if a.hash != *hash {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(acc, &[b"allowance", hash], a.bump)?;
    Ok(a)
}

pub fn read_contract(acc: &AccountView, hash: &[u8; 32]) -> Result<Contract, ProgramError> {
    let c = Contract::read(acc)?;
    if c.hash != *hash {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(acc, &[b"contract", hash], c.bump)?;
    Ok(c)
}

pub fn read_distribution(acc: &AccountView, hash: &[u8; 32]) -> Result<Distribution, ProgramError> {
    let d = Distribution::read(acc)?;
    if d.hash != *hash {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(acc, &[b"distribution", hash], d.bump)?;
    Ok(d)
}

/// A payee field: a key, or a unit that is not the payer itself.
fn check_payee(payee: &[u8; 32], kind: u8, origin: u64) -> Result<(), ProgramError> {
    if *payee == ZERO || (kind != crate::HOLDER_KEY && kind != crate::HOLDER_UNIT) {
        return Err(PoleisError::InvalidHolder.into());
    }
    if kind == crate::HOLDER_UNIT && holder_unit(payee) == origin {
        return Err(PoleisError::UnitMismatch.into());
    }
    Ok(())
}

/// Where a payment to `payee` lands, verified: a key's own account or a
/// token account it owns; a unit's treasury or its vault of the mint.
fn check_dest(dest: &AccountView, payee: &[u8; 32], kind: u8, mint: &[u8; 32]) -> Result<(), ProgramError> {
    let token = *mint != ZERO;
    if kind == crate::HOLDER_KEY {
        if token {
            if token_account_owner(dest)? != *payee {
                return Err(PoleisError::InvalidAccount.into());
            }
        } else if dest.address().as_array() != payee {
            return Err(PoleisError::InvalidAccount.into());
        }
    } else {
        let unit = holder_unit(payee);
        if token {
            expect_vault(dest, unit, mint)?;
        } else {
            super::read_treasury(dest, unit)?;
        }
    }
    Ok(())
}

// ============================ allowances ============================

/// TREASURY Allowance, at execute.
/// fields: [payee:32][payee_kind:1][mint:32][cap:8][period:8][expires:8]
/// accounts: [registry, allowance (w), system]
pub fn open_allowance(
    crank: &AccountView,
    registry: &AccountView,
    allowance: &mut AccountView,
    hash: &[u8; 32],
    origin: u64,
    fields: &[u8],
    now: u64,
) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(89)?;
    let payee = f.h32(0)?;
    let payee_kind = fields[32];
    let mint = f.h32(33)?;
    let cap = f.u64(65)?;
    let period = f.u64(73)?;
    let expires = f.u64(81)?;
    check_payee(&payee, payee_kind, origin)?;
    if cap == 0 || period == 0 || (expires != 0 && expires <= now) {
        return Err(PoleisError::InvalidAmount.into());
    }
    super::read_registry(registry, origin)?;
    let seeds: &[&[u8]] = &[b"allowance", hash];
    let bump = find_and_expect_pda(allowance, seeds)?;
    create_pda(crank, allowance, Allowance::SIZE, seeds, bump)?;
    Allowance {
        hash: *hash,
        unit_id: origin,
        payee,
        payee_kind,
        mint,
        cap,
        period,
        start: now,
        spent: 0,
        expires,
        payer: *crank.address().as_array(),
        bump,
    }
    .write(allowance)
}

/// TREASURY Revoke, at execute: the allowance closes at once, rent to
/// whoever executed it into being.
/// fields: [allowance hash:32] · accounts: [registry, allowance (w), payer (w)]
pub fn revoke_allowance(registry: &AccountView, allowance: &mut AccountView, payer: &mut AccountView, origin: u64, fields: &[u8]) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(32)?;
    let ahash = f.h32(0)?;
    super::read_registry(registry, origin)?;
    let a = read_allowance(allowance, &ahash)?;
    if a.unit_id != origin {
        return Err(PoleisError::UnitMismatch.into());
    }
    if payer.address().as_array() != &a.payer {
        return Err(PoleisError::InvalidAccount.into());
    }
    close_to(allowance, payer)
}

/// data: [hash:32][amount:8]
/// accounts, lamports: [puller (s), registry (w), treasury (w), allowance (w), dest (w)]
/// accounts, tokens:   [puller (s), registry (w), treasury, allowance (w), dest (w), vault (w), token program, token plan (w)]
/// A key payee pulls by its own signature; a unit payee is pulled for by
/// anyone, into its own treasury or vault. Up to the cap per period, charged
/// to the plan, refused past the expiry.
pub fn pull(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let d = Data(data);
    d.expect_len(40)?;
    let hash = d.h32(0)?;
    let amount = d.u64(32)?;
    if accounts.len() < 5 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, tail) = accounts.split_at_mut(5);
    let [puller, registry, treasury, allowance, dest] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(puller)?;
    expect_writable(allowance)?;
    expect_writable(dest)?;
    if amount == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    let now = Clock::get()?.slot;
    let mut a = read_allowance(allowance, &hash)?;
    if a.expired(now) {
        return Err(PoleisError::AllowanceDead.into());
    }
    let unit = a.unit_id;
    let r = super::read_registry(registry, unit)?;
    if r.frozen {
        return Err(PoleisError::Frozen.into());
    }
    if a.payee_kind == crate::HOLDER_KEY && puller.address().as_array() != &a.payee {
        return Err(PoleisError::Unauthorized.into());
    }
    check_dest(dest, &a.payee, a.payee_kind, &a.mint)?;

    // the allowance's own period
    if now >= a.start.saturating_add(a.period) {
        let periods = (now - a.start) / a.period;
        a.start = a.start.saturating_add(periods.saturating_mul(a.period));
        a.spent = 0;
    }
    let after = a.spent.checked_add(amount).ok_or(PoleisError::MathOverflow)?;
    if after > a.cap {
        return Err(PoleisError::AllowanceBreached.into());
    }
    a.spent = after;
    a.write(allowance)?;

    let t = super::read_treasury(treasury, unit)?;
    if a.is_token() {
        let [vault, token_program, token_plan, ..] = tail else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        expect_vault(vault, unit, &a.mint)?;
        super::charge_token_plan(registry, &a.mint, amount, now, Some(token_plan))?;
        transfer_from_vault(unit, treasury, t.bump, vault, dest, token_program, amount)?;
    } else {
        super::charge_plan(registry, amount, now)?;
        move_lamports(treasury, dest, amount)?;
        Treasury::add_out(treasury, amount)?;
        if a.payee_kind == crate::HOLDER_UNIT {
            Treasury::add_in(dest, amount)?;
        }
    }
    receipt(crate::RCPT_PULL, unit, &hash, &a.payee, &a.mint, amount, a.payee_kind);
    Ok(())
}

/// data: [hash:32] · accounts: [allowance (w), payer (w)]
/// Permissionless once the allowance is past its expiry: rent goes home.
pub fn close_allowance(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [allowance, payer] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(32)?;
    let hash = d.h32(0)?;
    expect_writable(allowance)?;
    expect_writable(payer)?;
    let a = read_allowance(allowance, &hash)?;
    if !a.expired(Clock::get()?.slot) {
        return Err(PoleisError::NotYet.into());
    }
    if payer.address().as_array() != &a.payer {
        return Err(PoleisError::InvalidAccount.into());
    }
    close_to(allowance, payer)
}

// ============================ contracts ============================

/// TREASURY Contract, at execute. The price leaves the treasury now, into
/// the contract account (lamports) or an escrow token account (tokens),
/// charged to the plan now: a promise is a commitment.
/// fields: [payee:32][payee_kind:1][mint:32][amount:8][deadline:8][arbiter:8]
/// accounts, lamports: [registry (w), treasury (w), contract (w), system]
/// accounts, tokens:   [registry (w), treasury, contract (w), system, mint, vault (w), escrow (w), token program]
#[allow(clippy::too_many_arguments)]
pub fn open_contract(
    crank: &AccountView,
    rest: &mut [AccountView],
    hash: &[u8; 32],
    origin: u64,
    fields: &[u8],
    now: u64,
) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(89)?;
    let payee = f.h32(0)?;
    let payee_kind = fields[32];
    let mint = f.h32(33)?;
    let amount = f.u64(65)?;
    let deadline = f.u64(73)?;
    let arbiter = f.u64(81)?;
    check_payee(&payee, payee_kind, origin)?;
    if amount == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    if deadline <= now || deadline > now.saturating_add(crate::MAX_TERM_SLOTS) {
        return Err(PoleisError::InvalidWindow.into());
    }
    if rest.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, tail) = rest.split_at_mut(4);
    let [registry, treasury, contract, _system] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    super::read_registry(registry, origin)?;
    let t = super::read_treasury(treasury, origin)?;
    let seeds: &[&[u8]] = &[b"contract", hash];
    let bump = find_and_expect_pda(contract, seeds)?;
    create_pda(crank, contract, Contract::SIZE, seeds, bump)?;
    Contract {
        hash: *hash,
        unit_id: origin,
        payee,
        payee_kind,
        mint,
        amount,
        deadline,
        arbiter,
        payer: *crank.address().as_array(),
        bump,
    }
    .write(contract)?;

    if mint == ZERO {
        super::charge_plan(registry, amount, now)?;
        move_lamports(treasury, contract, amount)?;
        Treasury::add_out(treasury, amount)?;
    } else {
        let [mint_acc, vault, escrow, token_program, token_plan, ..] = tail else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if mint_acc.address().as_array() != &mint {
            return Err(PoleisError::InvalidAccount.into());
        }
        expect_vault(vault, origin, &mint)?;
        let e_bump = expect_escrow(escrow, hash)?;
        let bump_ref = [e_bump];
        let e_seeds = [Seed::from(&b"escrow"[..]), Seed::from(&hash[..]), Seed::from(&bump_ref[..])];
        create_token_account(crank, escrow, mint_acc, treasury, token_program, &e_seeds)?;
        super::charge_token_plan(registry, &mint, amount, now, Some(token_plan))?;
        transfer_from_vault(origin, treasury, t.bump, vault, escrow, token_program, amount)?;
    }
    receipt(crate::RCPT_ESCROW, origin, hash, &payee, &mint, amount, payee_kind);
    Ok(())
}

/// Release to the payee or refund to the paying unit, then close: the
/// receipt is the record.
/// accounts, lamports: [contract (w), payer (w), treasury (w), dest (w)]
/// accounts, tokens:   [contract (w), payer (w), treasury, dest (w), escrow (w), token program]
/// On a refund `dest` is unused for lamports (the treasury is the
/// destination) and must be the unit's vault for tokens.
fn settle_contract(c: &Contract, accounts: &mut [AccountView], outcome: u8) -> ProgramResult {
    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, tail) = accounts.split_at_mut(4);
    let [contract, payer, treasury, dest] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_writable(contract)?;
    expect_writable(payer)?;
    if payer.address().as_array() != &c.payer {
        return Err(PoleisError::InvalidAccount.into());
    }
    let unit = c.unit_id;
    let t = super::read_treasury(treasury, unit)?;
    let release = outcome == crate::RULE_RELEASE;
    if release {
        check_dest(dest, &c.payee, c.payee_kind, &c.mint)?;
    }
    if c.is_token() {
        let [escrow, token_program, ..] = tail else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        expect_escrow(escrow, &c.hash)?;
        if !release {
            expect_vault(dest, unit, &c.mint)?;
        }
        transfer_from_vault(unit, treasury, t.bump, escrow, dest, token_program, c.amount)?;
        close_token_account(unit, treasury, t.bump, escrow, payer, token_program)?;
    } else if release {
        move_lamports(contract, dest, c.amount)?;
        if c.payee_kind == crate::HOLDER_UNIT {
            Treasury::add_in(dest, c.amount)?;
        }
    } else {
        move_lamports(contract, treasury, c.amount)?;
        Treasury::add_in(treasury, c.amount)?;
    }
    close_to(contract, payer)?;
    receipt(
        if release { crate::RCPT_RELEASE } else { crate::RCPT_REFUND },
        unit,
        &c.hash,
        &c.payee,
        &c.mint,
        c.amount,
        c.payee_kind,
    );
    Ok(())
}

/// SIGNAL Rule, at execute: the ruling unit is the paying unit itself or the
/// arbiter the contract named.
/// fields: [contract hash:32][outcome:1]
/// accounts: [registry (ruling unit), then the settle accounts]
pub fn rule_contract(rest: &mut [AccountView], origin: u64, fields: &[u8]) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(33)?;
    let chash = f.h32(0)?;
    let outcome = fields[32];
    if outcome != crate::RULE_RELEASE && outcome != crate::RULE_REFUND {
        return Err(PoleisError::BadPayload.into());
    }
    if rest.is_empty() {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (registry, tail) = rest.split_first_mut().ok_or(ProgramError::NotEnoughAccountKeys)?;
    super::read_registry(registry, origin)?;
    let contract = tail.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let c = read_contract(contract, &chash)?;
    if origin != c.unit_id && origin != c.arbiter {
        return Err(PoleisError::NotArbiter.into());
    }
    settle_contract(&c, tail, outcome)
}

/// data: [hash:32]
/// accounts: [signer (s), then the settle accounts (contract (w), payer (w), treasury, dest, ...)]
/// Past the deadline anyone refunds; inside it only the payee key may, by
/// resigning the contract.
pub fn expire_contract(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let d = Data(data);
    d.expect_len(32)?;
    let hash = d.h32(0)?;
    if accounts.len() < 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (signer, tail) = accounts.split_first_mut().ok_or(ProgramError::NotEnoughAccountKeys)?;
    expect_signer(signer)?;
    let c = read_contract(&tail[0], &hash)?;
    let now = Clock::get()?.slot;
    let resigned = c.payee_kind == crate::HOLDER_KEY && signer.address().as_array() == &c.payee;
    if now <= c.deadline && !resigned {
        return Err(PoleisError::ContractOpen.into());
    }
    settle_contract(&c, tail, crate::RULE_REFUND)
}

// ============================ the mints a society recognizes ============================

/// CODEX AllowMint, at execute: a mint beyond the default gets its own cap
/// per period in its own units (cap reset, period restarted); cap zero
/// withdraws it and the plan account closes, rent to whoever executed it
/// into being. The default mint needs no listing and cannot be listed.
/// fields: [mint:32][cap:8] · accounts: [registry, token plan (w), payer (w), system]
pub fn allow_mint(
    crank: &AccountView,
    registry: &AccountView,
    plan: &mut AccountView,
    payer: &mut AccountView,
    origin: u64,
    fields: &[u8],
    now: u64,
) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(40)?;
    let mint = f.h32(0)?;
    let cap = f.u64(32)?;
    let head = super::read_registry(registry, origin)?;
    if mint == ZERO || mint == head.default_mint {
        return Err(PoleisError::InvalidAccount.into());
    }
    let unit_le = origin.to_le_bytes();
    let seeds: &[&[u8]] = &[b"tokenplan", &unit_le, &mint];
    let bump = find_and_expect_pda(plan, seeds)?;
    let existing = TokenPlan::read(plan).ok();
    if cap == 0 {
        let tp = existing.ok_or(PoleisError::MintNotAllowed)?;
        if payer.address().as_array() != &tp.payer {
            return Err(PoleisError::InvalidAccount.into());
        }
        return close_to(plan, payer);
    }
    match existing {
        Some(mut tp) => {
            tp.cap = cap;
            tp.spent = 0;
            tp.start = now;
            tp.write(plan)
        }
        None => {
            create_pda(crank, plan, TokenPlan::SIZE, seeds, bump)?;
            TokenPlan { unit_id: origin, mint, cap, spent: 0, start: now, payer: *crank.address().as_array(), bump }.write(plan)
        }
    }
}

// ============================ dividends ============================

/// Cut a distribution of `amount` for the unit's floor: lamports out of the
/// treasury into the distribution itself, or tokens out of the vault into an
/// escrow beside it. Shared by Dividend, Dissolve, TokenDividend and the
/// policy's accrual. The caller has charged the plan, or has a reason not to.
/// Who may claim what is fixed here: the distribution carries the registry's
/// rows as they stand at the cut (the roster), and its total weight is the
/// roster's own sum.
#[allow(clippy::too_many_arguments)]
pub fn open_distribution(
    crank: &AccountView,
    registry: &AccountView,
    treasury: &mut AccountView,
    distribution: &mut AccountView,
    hash: &[u8; 32],
    unit: u64,
    amount: u64,
    now: u64,
    token: Option<(&mut AccountView, &mut AccountView, &AccountView, &AccountView)>,
) -> ProgramResult {
    if amount == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    let t = super::read_treasury(treasury, unit)?;
    let seeds: &[&[u8]] = &[b"distribution", hash];
    let bump = find_and_expect_pda(distribution, seeds)?;
    create_pda(crank, distribution, Distribution::size_for_registry(registry)?, seeds, bump)?;
    let total_weight = Distribution::init_roster(distribution, registry)?;
    if total_weight == 0 {
        return Err(PoleisError::NotHolder.into());
    }
    let mint = match &token {
        Some((_, _, mint_acc, _)) => *mint_acc.address().as_array(),
        None => ZERO,
    };
    Distribution {
        hash: *hash,
        unit_id: unit,
        amount,
        total_weight,
        paid: 0,
        claims: 0,
        bump,
        mint,
        payer: *crank.address().as_array(),
        created: now,
        claimed: [0u8; Distribution::CLAIM_BYTES],
    }
    .write(distribution)?;
    match token {
        None => {
            move_lamports(treasury, distribution, amount)?;
            Treasury::add_out(treasury, amount)?;
        }
        Some((vault, escrow, mint_acc, token_program)) => {
            expect_vault(vault, unit, &mint)?;
            let e_bump = expect_escrow(escrow, hash)?;
            let bump_ref = [e_bump];
            let e_seeds = [Seed::from(&b"escrow"[..]), Seed::from(&hash[..]), Seed::from(&bump_ref[..])];
            create_token_account(crank, escrow, mint_acc, treasury, token_program, &e_seeds)?;
            transfer_from_vault(unit, treasury, t.bump, vault, escrow, token_program, amount)?;
        }
    }
    receipt(crate::RCPT_DIVIDEND, unit, hash, &ZERO, &mint, amount, 0);
    Ok(())
}

/// CODEX DividendPolicy, at execute: from now on, `bps` of every lamport of
/// income is escrowed for the floor each `period`, cranked by anyone.
/// fields: [bps:2][period:8] · accounts: [registry (w), treasury]
pub fn set_dividend_policy(registry: &mut AccountView, treasury: &AccountView, origin: u64, fields: &[u8], now: u64) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(10)?;
    let bps = u16::from_le_bytes([fields[0], fields[1]]);
    let period = f.u64(2)?;
    if bps as u64 > crate::BPS {
        return Err(PoleisError::InvalidAmount.into());
    }
    if bps != 0 && !(crate::MIN_WINDOW_SLOTS..=crate::MAX_WINDOW_SLOTS).contains(&period) {
        return Err(PoleisError::InvalidWindow.into());
    }
    let mut h = super::read_registry(registry, origin)?;
    let t = super::read_treasury(treasury, origin)?;
    h.dividend_bps = bps;
    h.dividend_period = period;
    h.dividend_start = now;
    h.accrued_in = t.total_in;
    Registry::set_head(registry, &h)
}

/// The hash a policy accrual is keyed by: sha256("poleis:accrual" ‖ unit ‖ seq).
pub fn accrual_hash(unit: u64, seq: u64) -> [u8; 32] {
    sha256v(&[b"poleis:accrual", &unit.to_le_bytes(), &seq.to_le_bytes()])
}

/// data: [unit:8] · accounts: [crank (s,w), registry (w), treasury (w), distribution (w), system]
/// Permissionless, once per period: the policy's share of the income since
/// the last accrual becomes a distribution. Not charged to the plan: the
/// policy is a plan. A period with no income rolls without a distribution.
pub fn accrue_dividend(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, registry, treasury, distribution, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let unit = d.u64(0)?;
    expect_signer(crank)?;
    expect_writable(registry)?;
    expect_writable(treasury)?;
    let now = Clock::get()?.slot;
    let mut h = super::read_registry(registry, unit)?;
    if h.dividend_bps == 0 || h.dividend_period == 0 || now < h.dividend_start.saturating_add(h.dividend_period) {
        return Err(PoleisError::NotYet.into());
    }
    let t = super::read_treasury(treasury, unit)?;
    let income = t.total_in.saturating_sub(h.accrued_in);
    let amount: u64 = ((income as u128) * (h.dividend_bps as u128) / (crate::BPS as u128)).try_into().map_err(|_| PoleisError::MathOverflow)?;
    let periods = (now - h.dividend_start) / h.dividend_period;
    h.dividend_start = h.dividend_start.saturating_add(periods.saturating_mul(h.dividend_period));
    h.accrued_in = t.total_in;
    if amount > 0 {
        // the free balance may be below the share if the plan spent it: then
        // the accrual takes what is there and the rest is forgone
        let floor = Rent::get()?.try_minimum_balance(treasury.data_len())?;
        let free = treasury.lamports().saturating_sub(floor);
        let amount = amount.min(free);
        if amount > 0 {
            let hash = accrual_hash(unit, h.dividend_seq);
            h.dividend_seq = h.dividend_seq.checked_add(1).ok_or(PoleisError::MathOverflow)?;
            Registry::set_head(registry, &h)?;
            return open_distribution(crank, registry, treasury, distribution, &hash, unit, amount, now, None);
        }
    }
    Registry::set_head(registry, &h)
}

/// data: [hash:32]
/// accounts, lamports: [distribution (w), treasury (w), payer (w)]
/// accounts, tokens:   [distribution (w), treasury, payer (w), vault (w), escrow (w), token program]
/// Once fully claimed, or past the TTL: the dust goes home to the treasury
/// (or the vault) and the rent to whoever cut it.
pub fn close_distribution(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let d = Data(data);
    d.expect_len(32)?;
    let hash = d.h32(0)?;
    if accounts.len() < 3 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, tail) = accounts.split_at_mut(3);
    let [distribution, treasury, payer] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_writable(distribution)?;
    expect_writable(payer)?;
    let dist = read_distribution(distribution, &hash)?;
    let now = Clock::get()?.slot;
    if dist.paid < dist.amount && now <= dist.created.saturating_add(crate::DISTRIBUTION_TTL_SLOTS) {
        return Err(PoleisError::NotYet.into());
    }
    if payer.address().as_array() != &dist.payer {
        return Err(PoleisError::InvalidAccount.into());
    }
    let unit = dist.unit_id;
    let t = super::read_treasury(treasury, unit)?;
    let dust = dist.amount.saturating_sub(dist.paid);
    if dist.is_token() {
        let [vault, escrow, token_program, ..] = tail else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        expect_vault(vault, unit, &dist.mint)?;
        expect_escrow(escrow, &hash)?;
        expect_token_program(token_program)?;
        if dust > 0 {
            transfer_from_vault(unit, treasury, t.bump, escrow, vault, token_program, dust)?;
        }
        close_token_account(unit, treasury, t.bump, escrow, payer, token_program)?;
    } else if dust > 0 {
        move_lamports(distribution, treasury, dust)?;
        Treasury::add_in(treasury, dust)?;
    }
    close_to(distribution, payer)
}
