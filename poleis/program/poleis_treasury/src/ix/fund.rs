//! The fund: the polis of poleis. A constitutional share of every seat
//! sale, every market purchase and every trace lot settlement lands in one
//! account, and the fund pays the current option's recipient each cycle.
//! Its floor is every seat in every polis: a holder stakes the weight of its
//! seats on a vetted option and names how much of the fund should go out.
//! Its twelve chairs are the roots that generated the most fees this cycle,
//! each backing one option through its own double seal. An option is
//! current once the chair quorum backs it and its stake is more than half
//! of all stake, and stays current until another option crosses the same
//! line: first to X, kept until overtaken. The payout is the weighted mean
//! of the stakers' values, clamped to the range the root polis set.
//!
//! The chain is touched when money moves or a seal lands: a fee, a stake,
//! a backing, a grant. The vetting of an option and the constitution of
//! the fund are decisions of the root polis under its double seal.
//!
//! Nobody cranks the fund. A cycle rolls on touch: every instruction that
//! writes the fund first looks at the clock and ends a cycle that has run
//! its slots, booking the grant as owed and emptying the chairs. Paying a
//! fee is racing for a chair. The grant is a pull: its recipient, or anyone
//! for them, claims it. A fund nobody touches does not roll, and says so.

use crate::error::PoleisError;
use crate::state::{unit_holder, Candidate, Chair, FeeTally, Fund, Registry, Stake, NO_ROOT};
use crate::util::{
    close_to, create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda, move_lamports,
    receipt, sha256v, Data,
};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, ProgramResult,
};

pub const FUND_SEED: &[u8] = b"fund";
pub const TALLY_SEED: &[u8] = b"fees";
pub const OPTION_SEED: &[u8] = b"option";
pub const STAKE_SEED: &[u8] = b"stake";
const ZERO: [u8; 32] = [0u8; 32];

/// The fund account is always the one address; empty until the root polis
/// constitutes it. A fee site that is handed any other account refuses, so
/// no purchase can name a stranger's account and dodge the share. An account
/// there that this program owns at any other size is an older fund, and an
/// error: read as no fund, every fee site would charge nothing and its money
/// would be stranded unnoticed. The older fund's road is `grow_fund`, once.
pub fn fund_present(acc: &AccountView) -> Result<bool, ProgramError> {
    if acc.address() != &crate::FUND_ADDRESS {
        return Err(PoleisError::InvalidPda.into());
    }
    if !acc.owned_by(&crate::ID) {
        return Ok(false);
    }
    if acc.data_len() != Fund::SIZE {
        return Err(PoleisError::LayoutVersion.into());
    }
    Ok(true)
}

pub fn read_fund(acc: &AccountView) -> Result<Fund, ProgramError> {
    if acc.address() != &crate::FUND_ADDRESS {
        return Err(PoleisError::InvalidPda.into());
    }
    let f = Fund::read(acc)?;
    if f.bump != crate::FUND_BUMP {
        return Err(PoleisError::InvalidPda.into());
    }
    Ok(f)
}

fn read_option(acc: &AccountView, hash: &[u8; 32]) -> Result<Candidate, ProgramError> {
    let c = Candidate::read(acc)?;
    if c.hash != *hash {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(acc, &[OPTION_SEED, hash], c.bump)?;
    Ok(c)
}

/// The option's name: the recipient and the manifest, and nothing else, so
/// the same proposal is the same account whoever proposes it.
pub fn option_hash(recipient: &[u8; 32], manifest: &[u8; 32]) -> [u8; 32] {
    sha256v(&[crate::OPTION_DOMAIN, recipient, manifest])
}

/// The payout share the floor named, clamped to the root's range; with no
/// stake, the bottom of the range.
fn payout_bps(f: &Fund) -> u128 {
    if f.wp_weight == 0 {
        f.range_lo as u128
    } else {
        (f.wp_sum / f.wp_weight as u128).clamp(f.range_lo as u128, f.range_hi as u128)
    }
}

/// The cycle rolls on touch. Called first by every instruction that writes
/// the fund: a cycle that has run its slots ends here, before the caller's
/// own work. The grant, the floor's share of the free balance (lamports
/// less rent less what is already owed), is booked as owed to the current
/// option's recipient; if an earlier grant to another recipient still waits
/// in the one slot, this cycle's grant is deferred into the next cycle's
/// share, never lost. Then the chairs empty and the next cycle starts now.
/// Returns whether a cycle ended.
fn roll_if_due(f: &mut Fund, fund: &AccountView, now: u64) -> Result<bool, ProgramError> {
    if f.cycle_slots == 0 || now < f.cycle_start.saturating_add(f.cycle_slots) {
        return Ok(false);
    }
    if f.current != ZERO && (f.owed == 0 || f.owed_to == f.current_recipient) {
        let value = payout_bps(f);
        let floor = Rent::get()?.try_minimum_balance(fund.data_len())?;
        let free = fund.lamports().saturating_sub(floor).saturating_sub(f.owed);
        let amount = ((free as u128) * value / (crate::BPS as u128)) as u64;
        if amount > 0 {
            f.owed = f.owed.checked_add(amount).ok_or(PoleisError::MathOverflow)?;
            f.owed_to = f.current_recipient;
            receipt(crate::RCPT_GRANT, crate::ROOT_UNIT, &f.current, &f.owed_to, &ZERO, amount, 0);
        }
    }
    f.cycle_seq = f.cycle_seq.checked_add(1).ok_or(PoleisError::MathOverflow)?;
    f.cycle_start = now;
    for ch in f.chairs.iter_mut() {
        *ch = Chair { root: NO_ROOT, fees: 0, backing: ZERO };
    }
    Ok(true)
}

/// The race for a chair, run whenever a root's tally moves this cycle: a
/// root whose fees beat the least of the twelve takes that chair; an empty
/// chair is taken outright; a seated root only has its fees refreshed. The
/// ousted root's backing dies with its chair. Returns whether the root sits.
fn race(f: &mut Fund, root: u64, fees: u64) -> bool {
    if fees == 0 {
        return false;
    }
    if let Some(k) = f.chairs.iter().position(|ch| ch.root == root) {
        f.chairs[k].fees = fees;
        return true;
    }
    if let Some(k) = f.chairs.iter().position(|ch| ch.root == NO_ROOT) {
        f.chairs[k] = Chair { root, fees, backing: ZERO };
        return true;
    }
    let Some((k, least)) = f.chairs.iter().enumerate().min_by_key(|(_, ch)| ch.fees) else {
        return false;
    };
    if fees <= least.fees {
        return false;
    }
    f.chairs[k] = Chair { root, fees, backing: ZERO };
    true
}

/// The share of `amount` the fund takes, recorded against the root that
/// generated it. Returns the fee; the caller moves the lamports. Zero, and
/// nothing written, until the fund is constituted. Paying is racing: the
/// cycle rolls if due, then the root's tally races for a chair.
pub fn charge_fee(
    fund: &mut AccountView,
    tally: &mut AccountView,
    payer: &AccountView,
    unit_id: u64,
    root: u64,
    amount: u64,
) -> Result<u64, ProgramError> {
    if !fund_present(fund)? {
        return Ok(0);
    }
    let mut f = read_fund(fund)?;
    let fee = ((amount as u128) * (f.fee_bps as u128) / (crate::BPS as u128)) as u64;
    if fee == 0 {
        return Ok(0);
    }
    expect_writable(fund)?;
    expect_writable(tally)?;
    let now = Clock::get()?.slot;
    roll_if_due(&mut f, fund, now)?;
    let root_le = root.to_le_bytes();
    let seeds: &[&[u8]] = &[TALLY_SEED, &root_le];
    let mut t = if tally.data_len() == 0 {
        let bump = find_and_expect_pda(tally, seeds)?;
        create_pda(payer, tally, FeeTally::SIZE, seeds, bump)?;
        FeeTally { bump, root, cycle: f.cycle_seq, fees: 0, lifetime: 0 }
    } else {
        let t = FeeTally::read(tally)?;
        if t.root != root {
            return Err(PoleisError::UnitMismatch.into());
        }
        expect_pda(tally, seeds, t.bump)?;
        t
    };
    if t.cycle != f.cycle_seq {
        // a new cycle: the race restarts, the lifetime keeps counting
        t.cycle = f.cycle_seq;
        t.fees = 0;
    }
    t.fees = t.fees.checked_add(fee).ok_or(PoleisError::MathOverflow)?;
    t.lifetime = t.lifetime.checked_add(fee).ok_or(PoleisError::MathOverflow)?;
    t.write(tally)?;
    f.total_in = f.total_in.checked_add(fee).ok_or(PoleisError::MathOverflow)?;
    race(&mut f, root, t.fees);
    f.write(fund)?;
    receipt(crate::RCPT_FEE, unit_id, &ZERO, &unit_holder(root), &ZERO, fee, 0);
    Ok(fee)
}

/// First to X, kept until overtaken: the option becomes current when the
/// chair quorum backs it and its stake is more than half of all stake.
fn judge(f: &mut Fund, c: &Candidate, now: u64) {
    let chairs = f.chairs.iter().filter(|ch| ch.root != NO_ROOT && ch.backing == c.hash).count();
    if chairs >= f.chair_quorum as usize && (c.floor_stake as u128) * 2 > f.staked_total as u128 && f.current != c.hash {
        f.current = c.hash;
        f.current_since = now;
        f.current_recipient = c.recipient;
        receipt(crate::RCPT_SEAL, crate::ROOT_UNIT, &c.hash, &c.recipient, &ZERO, c.floor_stake, chairs as u8);
    }
}

/// data: [root:8] · accounts: [crank (s), fund (w), tally]
/// Permissionless, and no longer needed: every fee races for a chair as it
/// lands. Kept as an explicit run of the same race for a root's tally this
/// cycle; refused when the root's fees win no chair.
pub fn seat_chair(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, fund, tally] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let root = d.u64(0)?;
    expect_signer(crank)?;
    expect_writable(fund)?;
    let mut f = read_fund(fund)?;
    let now = Clock::get()?.slot;
    roll_if_due(&mut f, fund, now)?;
    let t = FeeTally::read(tally)?;
    if t.root != root {
        return Err(PoleisError::UnitMismatch.into());
    }
    expect_pda(tally, &[TALLY_SEED, &root.to_le_bytes()], t.bump)?;
    if t.cycle != f.cycle_seq || !race(&mut f, root, t.fees) {
        return Err(PoleisError::NoChair.into());
    }
    f.write(fund)
}

/// data: [recipient:32][manifest:32] · accounts: [proposer (s,w), fund (w), option (w), system]
/// Anyone proposes where the fund should go, paying the option fee into the
/// fund and the rent of the option. It is not stakeable until the root
/// polis vets it under its double seal.
pub fn propose_option(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [proposer, fund, option, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(64)?;
    let recipient = d.h32(0)?;
    let manifest = d.h32(32)?;
    expect_signer(proposer)?;
    expect_writable(fund)?;
    // a grant owed to the fund itself could never be paid (one account cannot
    // be both ends of a move), and the one owed slot would hold back every later grant
    if recipient == ZERO || recipient == *crate::FUND_ADDRESS.as_array() {
        return Err(PoleisError::InvalidAccount.into());
    }
    let mut f = read_fund(fund)?;
    let now = Clock::get()?.slot;
    roll_if_due(&mut f, fund, now)?;
    let hash = option_hash(&recipient, &manifest);
    let seeds: &[&[u8]] = &[OPTION_SEED, &hash];
    let bump = find_and_expect_pda(option, seeds)?;
    create_pda(proposer, option, Candidate::SIZE, seeds, bump)?;
    if f.option_fee > 0 {
        pinocchio_system::instructions::Transfer { from: proposer, to: fund, lamports: f.option_fee }.invoke()?;
        f.total_in = f.total_in.checked_add(f.option_fee).ok_or(PoleisError::MathOverflow)?;
        receipt(crate::RCPT_FEE, crate::ROOT_UNIT, &hash, proposer.address().as_array(), &ZERO, f.option_fee, 1);
    }
    f.write(fund)?;
    Candidate { bump, hash, recipient, manifest, proposer: *proposer.address().as_array(), vetted: false, floor_stake: 0, created: now }
        .write(option)
}

/// The root polis vets an option: execute-time body of SIGNAL VetOption.
/// accounts: [registry (origin), option (w)]
pub fn vet_option(rest: &mut [AccountView], origin: u64, fields: &[u8]) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(32)?;
    let hash = f.h32(0)?;
    if origin != crate::ROOT_UNIT {
        return Err(PoleisError::Unauthorized.into());
    }
    let [registry, option, ..] = rest else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    super::read_registry(registry, origin)?;
    let mut c = read_option(option, &hash)?;
    c.vetted = true;
    c.write(option)
}

/// A chair backs an option: execute-time body of SIGNAL BackOption, sealed
/// by a polis whose root holds a chair this cycle. A chair backs one option
/// at a time; backing another replaces it.
/// accounts: [registry (origin), fund (w), option (w)]
pub fn back_option(rest: &mut [AccountView], origin: u64, fields: &[u8], now: u64) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(32)?;
    let hash = f.h32(0)?;
    let [registry, fund, option, ..] = rest else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let head = super::read_registry(registry, origin)?;
    let mut fd = read_fund(fund)?;
    roll_if_due(&mut fd, fund, now)?;
    let c = read_option(option, &hash)?;
    if !c.vetted {
        return Err(PoleisError::NotVetted.into());
    }
    let k = fd.chairs.iter().position(|ch| ch.root != NO_ROOT && ch.root == head.root).ok_or(PoleisError::NoChair)?;
    fd.chairs[k].backing = hash;
    judge(&mut fd, &c, now);
    fd.write(fund)
}

/// The constitution of the fund: execute-time body of CODEX Fund, by the root
/// polis alone. Creates the fund on first passage, the crank paying its rent;
/// later passages change the parameters and never the balances or the chairs.
/// fields: [fee_bps:2][cycle_slots:8][range_lo:2][range_hi:2][option_fee:8][chair_quorum:1]
/// accounts: [registry (origin), fund (w), system]
pub fn codex_fund(crank: &AccountView, rest: &mut [AccountView], origin: u64, fields: &[u8], now: u64) -> ProgramResult {
    let f = Data(fields);
    f.expect_len(23)?;
    let fee_bps = u16::from_le_bytes([fields[0], fields[1]]);
    let cycle_slots = f.u64(2)?;
    let range_lo = u16::from_le_bytes([fields[10], fields[11]]);
    let range_hi = u16::from_le_bytes([fields[12], fields[13]]);
    let option_fee = f.u64(14)?;
    let chair_quorum = fields[22];
    if origin != crate::ROOT_UNIT {
        return Err(PoleisError::Unauthorized.into());
    }
    if fee_bps as u64 > crate::BPS || range_lo > range_hi || range_hi as u64 > crate::BPS {
        return Err(PoleisError::InvalidAmount.into());
    }
    if !(crate::MIN_CYCLE_SLOTS..=crate::MAX_CYCLE_SLOTS).contains(&cycle_slots) {
        return Err(PoleisError::InvalidWindow.into());
    }
    if chair_quorum == 0 || chair_quorum as usize > crate::PENTAGONS {
        return Err(PoleisError::InvalidHolder.into());
    }
    let [registry, fund, _system, ..] = rest else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    super::read_registry(registry, origin)?;
    let mut fd = if fund_present(fund)? {
        read_fund(fund)?
    } else {
        let seeds: &[&[u8]] = &[FUND_SEED];
        let bump = find_and_expect_pda(fund, seeds)?;
        create_pda(crank, fund, Fund::SIZE, seeds, bump)?;
        Fund::empty(bump, now)
    };
    fd.fee_bps = fee_bps;
    fd.cycle_slots = cycle_slots;
    fd.range_lo = range_lo;
    fd.range_hi = range_hi;
    fd.option_fee = option_fee;
    fd.chair_quorum = chair_quorum;
    fd.write(fund)
}

fn stake_math(f: &mut Fund, c: &mut Candidate, weight: u64, value_bps: u16, add: bool) -> Result<(), ProgramError> {
    let wv = (weight as u128) * (value_bps as u128);
    if add {
        c.floor_stake = c.floor_stake.checked_add(weight).ok_or(PoleisError::MathOverflow)?;
        f.staked_total = f.staked_total.checked_add(weight).ok_or(PoleisError::MathOverflow)?;
        f.wp_sum = f.wp_sum.checked_add(wv).ok_or(PoleisError::MathOverflow)?;
        f.wp_weight = f.wp_weight.checked_add(weight).ok_or(PoleisError::MathOverflow)?;
    } else {
        c.floor_stake = c.floor_stake.checked_sub(weight).ok_or(PoleisError::MathOverflow)?;
        f.staked_total = f.staked_total.checked_sub(weight).ok_or(PoleisError::MathOverflow)?;
        f.wp_sum = f.wp_sum.checked_sub(wv).ok_or(PoleisError::MathOverflow)?;
        f.wp_weight = f.wp_weight.checked_sub(weight).ok_or(PoleisError::MathOverflow)?;
    }
    Ok(())
}

/// data: [hash:32][unit:8][value_bps:2]
/// accounts: [voter (s,w), fund (w), option (w), registry, stake (w), system]
/// A holder stakes the weight of its seats in `unit` on a vetted option and
/// names the share of the fund that should go out per cycle. Staking again
/// replaces the earlier stake on the same option from the same seats.
pub fn stake(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [voter, fund, option, registry, stake, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(42)?;
    let hash = d.h32(0)?;
    let unit = d.u64(32)?;
    let value_bps = u16::from_le_bytes([data[40], data[41]]);
    expect_signer(voter)?;
    expect_writable(fund)?;
    expect_writable(option)?;
    expect_writable(stake)?;
    if value_bps as u64 > crate::BPS {
        return Err(PoleisError::InvalidAmount.into());
    }
    let mut f = read_fund(fund)?;
    let now = Clock::get()?.slot;
    roll_if_due(&mut f, fund, now)?;
    let mut c = read_option(option, &hash)?;
    if !c.vetted {
        return Err(PoleisError::NotVetted.into());
    }
    super::read_registry(registry, unit)?;
    let key: [u8; 32] = *voter.address().as_array();
    let i = Registry::find_holder(registry, crate::HOLDER_KEY, &key)?.ok_or(PoleisError::NotHolder)?;
    let h = Registry::holder(registry, i)?;
    let weight = h.seats.checked_mul(h.weight).ok_or(PoleisError::MathOverflow)?;
    if weight == 0 {
        return Err(PoleisError::ShareIsZero.into());
    }
    let unit_le = unit.to_le_bytes();
    let seeds: &[&[u8]] = &[STAKE_SEED, &hash, &unit_le, &key];
    let bump = if stake.data_len() == 0 {
        let bump = find_and_expect_pda(stake, seeds)?;
        create_pda(voter, stake, Stake::SIZE, seeds, bump)?;
        bump
    } else {
        let s = Stake::read(stake)?;
        if s.hash != hash || s.unit_id != unit || s.holder != key {
            return Err(PoleisError::InvalidAccount.into());
        }
        expect_pda(stake, seeds, s.bump)?;
        stake_math(&mut f, &mut c, s.weight, s.value_bps, false)?;
        s.bump
    };
    stake_math(&mut f, &mut c, weight, value_bps, true)?;
    Stake { bump, hash, unit_id: unit, holder: key, weight, value_bps }.write(stake)?;
    c.write(option)?;
    judge(&mut f, &c, now);
    f.write(fund)?;
    receipt(crate::RCPT_STAKE, unit, &hash, &key, &ZERO, weight, 1);
    Ok(())
}

/// data: [hash:32][unit:8] · accounts: [voter (s,w), fund (w), option (w), stake (w)]
/// The stake comes off and its rent goes home.
pub fn unstake(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [voter, fund, option, stake] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(40)?;
    let hash = d.h32(0)?;
    let unit = d.u64(32)?;
    expect_signer(voter)?;
    expect_writable(voter)?;
    expect_writable(fund)?;
    expect_writable(option)?;
    expect_writable(stake)?;
    let mut f = read_fund(fund)?;
    let now = Clock::get()?.slot;
    roll_if_due(&mut f, fund, now)?;
    let mut c = read_option(option, &hash)?;
    let key: [u8; 32] = *voter.address().as_array();
    let s = Stake::read(stake)?;
    if s.hash != hash || s.unit_id != unit || s.holder != key {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(stake, &[STAKE_SEED, &hash, &unit.to_le_bytes(), &key], s.bump)?;
    stake_math(&mut f, &mut c, s.weight, s.value_bps, false)?;
    c.write(option)?;
    judge(&mut f, &c, now);
    f.write(fund)?;
    receipt(crate::RCPT_STAKE, unit, &hash, &key, &ZERO, s.weight, 0);
    close_to(stake, voter)
}

/// data: none · accounts: [crank (s), fund (w)]
/// Permissionless: rolls a cycle that has run its slots, the same roll every
/// write to the fund performs on its own, and nothing else. Refused while
/// the cycle is open. The grant it books is paid by `claim_grant`.
pub fn settle_cycle(accounts: &mut [AccountView], _data: &[u8]) -> ProgramResult {
    let [crank, fund, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(crank)?;
    expect_writable(fund)?;
    let mut f = read_fund(fund)?;
    let now = Clock::get()?.slot;
    if !roll_if_due(&mut f, fund, now)? {
        return Err(PoleisError::CycleOpen.into());
    }
    f.write(fund)
}

/// data: none · accounts: [payer (s,w), fund (w), system, option (only when the older fund has a current option)]
/// AN OLDER FUND HAS ITS ROAD (Nov 24 2026; THE-AUDITS-SMALL-CURES-LANDING.md finding 4, "a program that holds an older one needs its
/// own road for it"). The fund of Sept 6 2026's build is this fund's first 706 bytes and nothing after: every field it has stands at
/// the offset it has here. It paid the current option straight from the fund when a cycle ended, so it owed nothing and kept no
/// recipient; this layout books the grant and pays it by a pull, and keeps the current option's recipient so a cycle rolls without the
/// option in the transaction. The growth appends the three: `owed` 0 and `owed_to` empty (nothing was ever booked), and
/// `current_recipient` copied from the current option, read at its own address and named by its own hash (empty when nothing is
/// current). Anyone may send it; the payer pays the rent of the added bytes and only that, so the fund's free balance, and with it
/// every grant, is what it was (a fund under its own minimum is lifted to the new one); no lamport leaves the fund. Nothing else is written: no cycle rolls, no receipt. Refused, changing
/// nothing, on another address or bump, an account this program does not own, any size but the older one (a fund already grown among
/// them), another discriminator or layout byte, old fields no constitution could have written, or a current option not handed.
pub fn grow_fund(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if !data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let [payer, fund, _system, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(payer)?;
    expect_writable(payer)?;
    expect_writable(fund)?;
    if fund.address() != &crate::FUND_ADDRESS {
        return Err(PoleisError::InvalidPda.into());
    }
    let mut f = Fund::read_older(fund)?;
    if f.bump != crate::FUND_BUMP {
        return Err(PoleisError::InvalidPda.into());
    }
    // what CODEX Fund refuses, an older fund cannot hold
    let bps = crate::BPS;
    if f.fee_bps as u64 > bps
        || f.range_lo > f.range_hi
        || f.range_hi as u64 > bps
        || !(crate::MIN_CYCLE_SLOTS..=crate::MAX_CYCLE_SLOTS).contains(&f.cycle_slots)
        || f.chair_quorum == 0
        || f.chair_quorum as usize > crate::PENTAGONS
    {
        return Err(PoleisError::NotAnOlderFund.into());
    }
    if f.current != ZERO {
        let [option, ..] = rest else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        let c = read_option(option, &f.current)?;
        if option_hash(&c.recipient, &c.manifest) != f.current {
            return Err(PoleisError::InvalidAccount.into());
        }
        f.current_recipient = c.recipient;
    }
    // the rent of the added bytes, from the payer, before the account grows (a system transfer: nothing moved by hand here). A fund under
    // the older size's own minimum (a copy of a chain laid on a chain of a dearer rent, the rehearsal's) is lifted to the new size's
    // minimum instead: a rent-paying account may not change its size, and on a chain where it was exempt the two are the same sum or less
    let rent = Rent::get()?;
    let floor = rent.try_minimum_balance(Fund::SIZE)?;
    let added = floor.saturating_sub(rent.try_minimum_balance(Fund::OLD_SIZE)?);
    let top_up = added.max(floor.saturating_sub(fund.lamports()));
    if top_up > 0 {
        pinocchio_system::instructions::Transfer { from: payer, to: fund, lamports: top_up }.invoke()?;
    }
    {
        use pinocchio::Resize;
        fund.resize(Fund::SIZE)?;
    }
    f.write(fund)
}

/// data: none · accounts: [crank (s), fund (w), recipient (w)]
/// Permissionless: pays `recipient` what the fund owes it, the grant booked
/// when a cycle rolled. The recipient's own incentive is the crank; anyone
/// may send it for them. A claim is a touch, so a due cycle rolls first.
/// Refused when nothing is owed to that account.
pub fn claim_grant(accounts: &mut [AccountView], _data: &[u8]) -> ProgramResult {
    let [crank, fund, recipient] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(crank)?;
    expect_writable(fund)?;
    expect_writable(recipient)?;
    let mut f = read_fund(fund)?;
    let now = Clock::get()?.slot;
    roll_if_due(&mut f, fund, now)?;
    if f.owed == 0 || recipient.address().as_array() != &f.owed_to {
        return Err(PoleisError::NothingOwed.into());
    }
    let amount = f.owed;
    move_lamports(fund, recipient, amount)?;
    f.total_out = f.total_out.checked_add(amount).ok_or(PoleisError::MathOverflow)?;
    receipt(crate::RCPT_GRANT, crate::ROOT_UNIT, &ZERO, &f.owed_to, &ZERO, amount, 1);
    f.owed = 0;
    f.owed_to = ZERO;
    f.write(fund)
}
