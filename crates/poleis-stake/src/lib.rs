//! THE POOL'S YIELD AND THE STAKE-TIME (Nov 20 2026, lane/the-pools-yield-and-the-stake-time, seat claude-dc5c1cd7).
//!
//! His words, UTC. Nov 20 2026 20:25: "i need to stay true to what i say in the introduction my friend... i say solana is not smart etc etc...
//! staking for collective self-improvement etc..."; then, on "Should I build pieces 3 and 4 in Rust: the pool-yield reader and the stake-time
//! count?": "Yes, build both" (20:31). Before it, 20:18: "we are supposed to delegate automatically like marinade does but with our own rules.. for what is
//! worth, we do the sanctum way for liquid staking across various snowtracers". And Oct 29 18:57: "$POOL is unlocked all at once 5 years down the
//! line, until then, stakers just compete for who accumulates the most, and the SOL yielded goes into funding ai credits..."
//!
//! Her two lines this makes countable (the mint page, dl-athena.mjs STORY): "Its yield becomes my swarm's fuel." and "Give it your SOL's light,
//! hold $POOL."
//!
//! WHAT IT IS. A snowtracer's pool is a stake pool whose epoch fee is the whole yield (the Oct 29 pool, tools/embertide-mesh/pool/pool-genesis.mjs:
//! fee 1/1). Each epoch the pool's program mints that fee, as pool tokens, into the manager's fee account. Three laws, all pure:
//!
//! 1. THE POOL, READ (`pool_of`): the stake pool account's bytes, as the SPL stake pool program and Sanctum's two forks of it lay them.
//! 2. THE EPOCH'S YIELD (`fee_mints`, `epoch_fee`): the pool tokens the program minted into the fee account inside its own UpdateStakePoolBalance
//!    in that epoch. Read from the mints, never from the account's balance: on mainnet an owner of such a pool sweeps the fee account every epoch
//!    (fpSOL, read Nov 20 2026: +25,198,739 at slot 471,745,493, then -26,913,047 by the owner's hand), so a balance read from two days says nothing.
//! 3. THE BOOK (`count`): what each holder held from one read to the next, and their share of the yield given up. Held is the LESSER of the two
//!    reads: a holder is counted only for what stood at both, so nothing is earned by arriving before a read and leaving after it.
//!
//! WHAT IT DOES NOT DECIDE. $POOL's formula is his (Oct 29: "to who staked the most"). The book keeps both numbers a formula could want, each
//! holder's stake-time (lamports held, summed over epochs) and their light (lamports of yield given up), and picks neither. Whose tank a pool's
//! yield fills is the caller's word (`refill_row` takes the tank's name). Nothing here signs, sends or moves anything.
//!
//! COST. An epoch's yield: one page of the fee account's signatures and one transaction read for each in the epoch (fpSOL: 3). A book's count:
//! one read of the pool, one of the mint's token accounts (40 bytes each), and the yield above. Nothing runs at rest.

use std::collections::BTreeMap;
use std::fmt;

use serde_json::{json, Map, Value};

/// A Solana address, as its 32 bytes.
pub type Key = [u8; 32];

/// The program's own number for UpdateStakePoolBalance (the eighth of its instructions; read on mainnet beside its log line).
pub const UPDATE_POOL_BALANCE: u8 = 7;
/// The stake pool account's first byte (AccountType::StakePool).
pub const ACCOUNT_STAKE_POOL: u8 = 1;
pub const BOOK_SCHEMA: &str = "embertide.stake-book/1";

/// Why a read or a count was refused, in plain words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// the bytes are not a stake pool's (too short, another account type, a tag the layout does not have)
    NotAPool(&'static str),
    /// the bytes are not a token account's
    NotATokenAccount,
    /// an epoch before the cluster's first normal one, where epochs had other lengths
    BeforeNormalEpochs,
    /// this epoch is already in the book
    AlreadyCounted { epoch: u64 },
    /// a book of another pool, or of a schema this law does not read
    NotThisBook(&'static str),
    /// a tank's name the router would not take
    NotATank,
    /// a number that is not one (a price of zero, an amount that is not digits)
    NotANumber(&'static str),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::NotAPool(why) => write!(f, "not a stake pool account: {why}"),
            Refusal::NotATokenAccount => write!(f, "not a token account"),
            Refusal::BeforeNormalEpochs => write!(f, "an epoch before the cluster's first normal epoch"),
            Refusal::AlreadyCounted { epoch } => write!(f, "epoch {epoch} is already counted in this book"),
            Refusal::NotThisBook(why) => write!(f, "not this pool's book: {why}"),
            Refusal::NotATank => write!(f, "not a tank's name"),
            Refusal::NotANumber(what) => write!(f, "not a number: {what}"),
        }
    }
}

// ---------------------------------------------------------------- 1. THE POOL, READ

/// A fee as the program keeps it: numerator over denominator (the account holds the denominator first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fee {
    pub numerator: u64,
    pub denominator: u64,
}

impl Fee {
    /// the whole yield: what a snowtracer's pool charges (1/1, 100/100, ...)
    pub fn is_whole(&self) -> bool {
        self.denominator != 0 && self.numerator == self.denominator
    }
}

/// What this law reads of a stake pool account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pool {
    pub manager: Key,
    /// the role that places the pool's stake across validators; it can never withdraw
    pub staker: Key,
    pub validator_list: Key,
    pub reserve_stake: Key,
    pub pool_mint: Key,
    pub manager_fee_account: Key,
    pub token_program: Key,
    pub total_lamports: u64,
    pub pool_token_supply: u64,
    pub last_update_epoch: u64,
    pub epoch_fee: Fee,
    pub last_epoch_pool_token_supply: u64,
    pub last_epoch_total_lamports: u64,
}

struct Cursor<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Refusal> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.b.len()).ok_or(Refusal::NotAPool("the account ends before its fields do"))?;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, Refusal> {
        Ok(self.take(1)?[0])
    }
    fn u64(&mut self) -> Result<u64, Refusal> {
        let mut a = [0u8; 8];
        a.copy_from_slice(self.take(8)?);
        Ok(u64::from_le_bytes(a))
    }
    fn key(&mut self) -> Result<Key, Refusal> {
        let mut a = [0u8; 32];
        a.copy_from_slice(self.take(32)?);
        Ok(a)
    }
    fn fee(&mut self) -> Result<Fee, Refusal> {
        let denominator = self.u64()?;
        let numerator = self.u64()?;
        Ok(Fee { numerator, denominator })
    }
    /// Option<Pubkey>: a tag, then the key when the tag is 1
    fn skip_option_key(&mut self) -> Result<(), Refusal> {
        match self.u8()? {
            0 => Ok(()),
            1 => self.take(32).map(|_| ()),
            _ => Err(Refusal::NotAPool("an option's tag is neither 0 nor 1")),
        }
    }
    /// FutureEpoch<Fee>: none, or a fee that takes effect in one or two epochs
    fn skip_future_fee(&mut self) -> Result<(), Refusal> {
        match self.u8()? {
            0 => Ok(()),
            1 | 2 => self.take(16).map(|_| ()),
            _ => Err(Refusal::NotAPool("a future fee's tag is not 0, 1 or 2")),
        }
    }
}

/// THE POOL, READ: a stake pool account's bytes. Pure.
pub fn pool_of(bytes: &[u8]) -> Result<Pool, Refusal> {
    let mut c = Cursor { b: bytes, at: 0 };
    if c.u8()? != ACCOUNT_STAKE_POOL {
        return Err(Refusal::NotAPool("its first byte does not say stake pool"));
    }
    let manager = c.key()?;
    let staker = c.key()?;
    c.key()?; // the stake deposit authority
    c.u8()?; // the withdraw authority's bump seed
    let validator_list = c.key()?;
    let reserve_stake = c.key()?;
    let pool_mint = c.key()?;
    let manager_fee_account = c.key()?;
    let token_program = c.key()?;
    let total_lamports = c.u64()?;
    let pool_token_supply = c.u64()?;
    let last_update_epoch = c.u64()?;
    c.take(8 + 8 + 32)?; // the lockup: a time, an epoch, a custodian
    let epoch_fee = c.fee()?;
    c.skip_future_fee()?; // next epoch's fee
    c.skip_option_key()?; // the preferred deposit validator
    c.skip_option_key()?; // the preferred withdraw validator
    c.fee()?; // the stake deposit fee
    c.fee()?; // the stake withdrawal fee
    c.skip_future_fee()?;
    c.u8()?; // the stake referral fee
    c.skip_option_key()?; // the SOL deposit authority
    c.fee()?; // the SOL deposit fee
    c.u8()?; // the SOL referral fee
    c.skip_option_key()?; // the SOL withdraw authority
    c.fee()?; // the SOL withdrawal fee
    c.skip_future_fee()?;
    let last_epoch_pool_token_supply = c.u64()?;
    let last_epoch_total_lamports = c.u64()?;
    Ok(Pool {
        manager,
        staker,
        validator_list,
        reserve_stake,
        pool_mint,
        manager_fee_account,
        token_program,
        total_lamports,
        pool_token_supply,
        last_update_epoch,
        epoch_fee,
        last_epoch_pool_token_supply,
        last_epoch_total_lamports,
    })
}

/// A token account's mint, owner and amount (its first 72 bytes; a Token-2022 account is longer and opens the same way). Pure.
pub fn token_of(bytes: &[u8]) -> Result<(Key, Key, u64), Refusal> {
    if bytes.len() < 165 {
        return Err(Refusal::NotATokenAccount);
    }
    let (mut mint, mut owner, mut amount) = ([0u8; 32], [0u8; 32], [0u8; 8]);
    mint.copy_from_slice(&bytes[0..32]);
    owner.copy_from_slice(&bytes[32..64]);
    amount.copy_from_slice(&bytes[64..72]);
    Ok((mint, owner, u64::from_le_bytes(amount)))
}

/// What pool tokens are worth in lamports at a pool's rate, rounded down (as the program rounds a withdrawal). Pure.
pub fn lamports_of(tokens: u64, total_lamports: u64, pool_token_supply: u64) -> u64 {
    if pool_token_supply == 0 {
        return 0;
    }
    ((tokens as u128) * (total_lamports as u128) / (pool_token_supply as u128)) as u64
}

pub fn address(k: &Key) -> String {
    bs58::encode(k).into_string()
}

pub fn key_of(s: &str) -> Option<Key> {
    let v = bs58::decode(s).into_vec().ok()?;
    <Key>::try_from(v.as_slice()).ok()
}

// ---------------------------------------------------------------- 2. THE EPOCH'S YIELD

/// The cluster's epochs, as getEpochSchedule says them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Schedule {
    pub slots_per_epoch: u64,
    pub first_normal_epoch: u64,
    pub first_normal_slot: u64,
}

impl Schedule {
    /// the first and the last slot of a normal epoch
    pub fn slots_of(&self, epoch: u64) -> Result<(u64, u64), Refusal> {
        if epoch < self.first_normal_epoch || self.slots_per_epoch == 0 {
            return Err(Refusal::BeforeNormalEpochs);
        }
        let first = self.first_normal_slot + (epoch - self.first_normal_epoch) * self.slots_per_epoch;
        Ok((first, first + self.slots_per_epoch - 1))
    }
    /// the epoch a slot is in; None before the first normal epoch
    pub fn epoch_of(&self, slot: u64) -> Option<u64> {
        if slot < self.first_normal_slot || self.slots_per_epoch == 0 {
            return None;
        }
        Some(self.first_normal_epoch + (slot - self.first_normal_slot) / self.slots_per_epoch)
    }
}

/// One mint of the pool's fee: the slot it happened in and the pool tokens minted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeMint {
    pub slot: u64,
    pub tokens: u64,
}

/// THE FEE'S MINTS in one transaction (a getTransaction answer, jsonParsed): every mint into `fee_account` made inside an instruction of
/// `program` that is UpdateStakePoolBalance on `pool`. A failed transaction minted nothing. A transfer into the fee account (a withdrawal's fee),
/// a mint under another instruction (a deposit's fee) and the owner's own sweep are not the yield and are not counted. Pure.
pub fn fee_mints(tx: &Value, program: &str, pool: &str, fee_account: &str) -> Vec<FeeMint> {
    let mut out = Vec::new();
    let Some(slot) = tx.get("slot").and_then(Value::as_u64) else { return out };
    if !tx.pointer("/meta/err").map_or(true, Value::is_null) {
        return out;
    }
    let Some(outer) = tx.pointer("/transaction/message/instructions").and_then(Value::as_array) else { return out };
    let Some(inner) = tx.pointer("/meta/innerInstructions").and_then(Value::as_array) else { return out };
    for group in inner {
        let Some(ix) = group.get("index").and_then(Value::as_u64).and_then(|i| outer.get(i as usize)) else { continue };
        if ix.get("programId").and_then(Value::as_str) != Some(program) {
            continue;
        }
        let tag = ix.get("data").and_then(Value::as_str).and_then(|d| bs58::decode(d).into_vec().ok()).and_then(|d| d.first().copied());
        if tag != Some(UPDATE_POOL_BALANCE) {
            continue;
        }
        if ix.pointer("/accounts/0").and_then(Value::as_str) != Some(pool) {
            continue;
        }
        for i in group.get("instructions").and_then(Value::as_array).into_iter().flatten() {
            let kind = i.pointer("/parsed/type").and_then(Value::as_str);
            if kind != Some("mintTo") && kind != Some("mintToChecked") {
                continue;
            }
            if i.pointer("/parsed/info/account").and_then(Value::as_str) != Some(fee_account) {
                continue;
            }
            let amount = i.pointer("/parsed/info/amount").or_else(|| i.pointer("/parsed/info/tokenAmount/amount")).and_then(Value::as_str);
            if let Some(tokens) = amount.and_then(|a| a.parse::<u64>().ok()) {
                out.push(FeeMint { slot, tokens });
            }
        }
    }
    out
}

/// THE EPOCH'S YIELD, in pool tokens: the fee's mints whose slot is in that epoch, summed. Pure.
pub fn epoch_fee(mints: &[FeeMint], schedule: &Schedule, epoch: u64) -> Result<u64, Refusal> {
    let (first, last) = schedule.slots_of(epoch)?;
    Ok(mints.iter().filter(|m| m.slot >= first && m.slot <= last).fold(0u64, |a, m| a.saturating_add(m.tokens)))
}

// ---------------------------------------------------------------- 3. THE BOOK

/// One read of who holds a pool's tokens: the epoch it was read in, the pool's rate then, and each owner's tokens (an owner's accounts summed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub epoch: u64,
    pub total_lamports: u64,
    pub pool_token_supply: u64,
    pub holders: BTreeMap<String, u64>,
}

/// THE HOLDERS of one read: the mint's token accounts as (the account, its owner, its amount), every owner's accounts summed. The manager's fee
/// account is the yield itself, not a stake, and is left out; so is every owner in `except` (the caller's word: the house's own wallet, say);
/// an empty account holds nothing. Pure.
pub fn holders_of(accounts: &[(String, String, u64)], fee_account: &str, except: &[String]) -> BTreeMap<String, u64> {
    let mut out: BTreeMap<String, u64> = BTreeMap::new();
    for (account, owner, amount) in accounts {
        if *amount == 0 || account == fee_account || except.iter().any(|e| e == owner) {
            continue;
        }
        let e = out.entry(owner.clone()).or_insert(0);
        *e = e.saturating_add(*amount);
    }
    out
}

/// What one holder has in the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Staker {
    /// lamports held, summed over the epochs counted (his Oct 29 "it just multiplies how long u been staking")
    pub stake_time: u128,
    /// lamports of yield given up: this holder's share of every counted epoch's fee (her "Give it your SOL's light")
    pub light: u128,
    /// how many counted epochs this holder held anything in
    pub epochs: u64,
}

/// One counted epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochRow {
    pub epoch: u64,
    /// the epoch of the read before it; more than one behind is a gap, and a gap is counted as ONE epoch
    pub from: u64,
    pub fee_tokens: u64,
    pub fee_lamports: u64,
    /// lamports that stood at both reads, all holders together
    pub held_lamports: u128,
    /// lamports of the fee no holder's share took (rounding; or the whole fee when nobody held)
    pub dust: u64,
    pub holders: u64,
}

/// The book of one pool. It is STATE, one value replaced in place (THE HOUSE KEEPS STATE, NEVER THE STREAM): the last read, the last counted epoch's
/// row, the sums over every counted epoch, and every holder's sums. It keeps no list of epochs: an epoch's yield can be read again from the chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book {
    pub pool: String,
    pub last: Option<Snapshot>,
    /// the last epoch counted
    pub row: Option<EpochRow>,
    /// how many epochs were counted
    pub counted: u64,
    /// lamports of fee over every counted epoch; the holders' lights and `dust` sum to it
    pub fee_lamports: u128,
    pub dust: u128,
    pub stakers: BTreeMap<String, Staker>,
}

impl Book {
    pub fn new(pool: &str) -> Book {
        Book { pool: pool.to_string(), last: None, row: None, counted: 0, fee_lamports: 0, dust: 0, stakers: BTreeMap::new() }
    }
}

/// THE COUNT: one more read laid on the book. `fee_tokens` is the yield minted in every epoch after the last read's, up to this read's.
///
/// - The first read counts nothing: nobody has yet held from one read to the next.
/// - A read of an epoch the book already has, or an older one, is refused: an epoch is counted once.
/// - HELD is the lesser of a holder's tokens at the last read and at this one, at this read's rate. A holder who came after the last read, or left
///   before this one, held nothing across it.
/// - Each holder's LIGHT is their share of the fee by what they held, rounded down; what rounding leaves is the row's dust, and with nobody
///   holding the whole fee is dust. So the lights and the dust always sum to the fee.
/// - A gap (the last read more than one epoch back) is counted as one epoch, for everyone alike: what stood between is not known.
///
/// The book given is not changed; the new one is returned. Pure.
pub fn count(book: &Book, snap: &Snapshot, fee_tokens: u64) -> Result<Book, Refusal> {
    let mut next = book.clone();
    let Some(last) = &book.last else {
        next.last = Some(snap.clone());
        return Ok(next);
    };
    if snap.epoch <= last.epoch {
        return Err(Refusal::AlreadyCounted { epoch: snap.epoch });
    }
    let fee_lamports = lamports_of(fee_tokens, snap.total_lamports, snap.pool_token_supply);
    let mut held: Vec<(&String, u128)> = Vec::new();
    let mut total: u128 = 0;
    for (owner, now) in &snap.holders {
        let before = last.holders.get(owner).copied().unwrap_or(0);
        let h = lamports_of((*now).min(before), snap.total_lamports, snap.pool_token_supply) as u128;
        if h > 0 {
            held.push((owner, h));
            total += h;
        }
    }
    let mut shared: u128 = 0;
    for (owner, h) in &held {
        let light = if total == 0 { 0 } else { (fee_lamports as u128) * h / total };
        shared += light;
        let s = next.stakers.entry((*owner).clone()).or_default();
        s.stake_time += h;
        s.light += light;
        s.epochs += 1;
    }
    let dust = (fee_lamports as u128 - shared) as u64;
    next.row = Some(EpochRow { epoch: snap.epoch, from: last.epoch, fee_tokens, fee_lamports, held_lamports: total, dust, holders: held.len() as u64 });
    next.counted += 1;
    next.fee_lamports += fee_lamports as u128;
    next.dust += dust as u128;
    next.last = Some(snap.clone());
    Ok(next)
}

// The book as JSON. Sums are written as strings of digits: a lamport-epoch sum outgrows what a JSON number keeps exactly.

fn digits(v: Option<&Value>, what: &'static str) -> Result<u128, Refusal> {
    match v {
        Some(Value::String(s)) if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) => s.parse::<u128>().map_err(|_| Refusal::NotANumber(what)),
        Some(Value::Number(n)) => n.as_u64().map(u128::from).ok_or(Refusal::NotANumber(what)),
        _ => Err(Refusal::NotANumber(what)),
    }
}

fn digits64(v: Option<&Value>, what: &'static str) -> Result<u64, Refusal> {
    u64::try_from(digits(v, what)?).map_err(|_| Refusal::NotANumber(what))
}

fn snapshot_json(s: &Snapshot) -> Value {
    let holders: Map<String, Value> = s.holders.iter().map(|(k, v)| (k.clone(), json!(v.to_string()))).collect();
    json!({ "epoch": s.epoch, "total_lamports": s.total_lamports.to_string(), "pool_token_supply": s.pool_token_supply.to_string(), "holders": holders })
}

fn snapshot_of(v: &Value) -> Result<Snapshot, Refusal> {
    let mut holders = BTreeMap::new();
    for (k, a) in v.get("holders").and_then(Value::as_object).ok_or(Refusal::NotThisBook("a read without its holders"))? {
        holders.insert(k.clone(), digits64(Some(a), "a holder's tokens")?);
    }
    Ok(Snapshot {
        epoch: digits64(v.get("epoch"), "a read's epoch")?,
        total_lamports: digits64(v.get("total_lamports"), "a read's lamports")?,
        pool_token_supply: digits64(v.get("pool_token_supply"), "a read's supply")?,
        holders,
    })
}

fn row_json(e: &EpochRow) -> Value {
    json!({ "epoch": e.epoch, "from": e.from, "fee_tokens": e.fee_tokens.to_string(), "fee_lamports": e.fee_lamports.to_string(),
        "held_lamports": e.held_lamports.to_string(), "dust": e.dust.to_string(), "holders": e.holders })
}

fn row_of(e: &Value) -> Result<EpochRow, Refusal> {
    Ok(EpochRow {
        epoch: digits64(e.get("epoch"), "an epoch")?,
        from: digits64(e.get("from"), "an epoch's from")?,
        fee_tokens: digits64(e.get("fee_tokens"), "an epoch's fee")?,
        fee_lamports: digits64(e.get("fee_lamports"), "an epoch's fee")?,
        held_lamports: digits(e.get("held_lamports"), "an epoch's held")?,
        dust: digits64(e.get("dust"), "an epoch's dust")?,
        holders: digits64(e.get("holders"), "an epoch's holders")?,
    })
}

pub fn book_json(b: &Book) -> Value {
    let stakers: Map<String, Value> = b
        .stakers
        .iter()
        .map(|(k, s)| (k.clone(), json!({ "stake_time": s.stake_time.to_string(), "light": s.light.to_string(), "epochs": s.epochs })))
        .collect();
    json!({
        "schema": BOOK_SCHEMA, "pool": b.pool, "last": b.last.as_ref().map(snapshot_json), "row": b.row.as_ref().map(row_json),
        "counted": b.counted, "fee_lamports": b.fee_lamports.to_string(), "dust": b.dust.to_string(), "stakers": stakers,
    })
}

/// A book read back, for the pool it must be of. A book of another pool or another schema is refused, never mended.
pub fn book_of(v: &Value, pool: &str) -> Result<Book, Refusal> {
    if v.get("schema").and_then(Value::as_str) != Some(BOOK_SCHEMA) {
        return Err(Refusal::NotThisBook("another schema"));
    }
    if v.get("pool").and_then(Value::as_str) != Some(pool) {
        return Err(Refusal::NotThisBook("another pool's"));
    }
    let last = match v.get("last") {
        None | Some(Value::Null) => None,
        Some(s) => Some(snapshot_of(s)?),
    };
    let row = match v.get("row") {
        None | Some(Value::Null) => None,
        Some(e) => Some(row_of(e)?),
    };
    let mut stakers = BTreeMap::new();
    for (k, s) in v.get("stakers").and_then(Value::as_object).ok_or(Refusal::NotThisBook("no stakers"))? {
        stakers.insert(
            k.clone(),
            Staker { stake_time: digits(s.get("stake_time"), "a stake-time")?, light: digits(s.get("light"), "a light")?, epochs: digits64(s.get("epochs"), "a holder's epochs")? },
        );
    }
    Ok(Book {
        pool: pool.to_string(),
        last,
        row,
        counted: digits64(v.get("counted"), "the epochs counted")?,
        fee_lamports: digits(v.get("fee_lamports"), "the book's fee")?,
        dust: digits(v.get("dust"), "the book's dust")?,
        stakers,
    })
}

// ---------------------------------------------------------------- THE REFILL'S ROW

/// A tank's name, as Space Router's law takes it (space-router-law.mjs NAME_RX: a lowercase letter, then up to 31 of a-z, 0-9 and -).
pub fn is_tank(name: &str) -> bool {
    let b = name.as_bytes();
    !b.is_empty() && b.len() <= 32 && b[0].is_ascii_lowercase() && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// THE REFILL'S ROW: an epoch's yield of a pool as the one row the tanks' ledger takes for it, in the shape the Oct 29 crank writes
/// (tools/embertide-mesh/pool/pool-crank.mjs epochRows): its why opens with the key `pool:<epoch>:<address>`, and a key the ledger already holds
/// is never written twice. Here the address is the pool's, and the tank is the pool's snowtracer's, named by the caller. The price of SOL is the
/// caller's word too ("at his price for SOL"). This only says the row: a refill is his hand. Pure.
pub fn refill_row(epoch: u64, pool: &str, tank: &str, lamports: u64, sol_usd: f64) -> Result<Value, Refusal> {
    if !is_tank(tank) {
        return Err(Refusal::NotATank);
    }
    if !(sol_usd.is_finite() && sol_usd > 0.0) {
        return Err(Refusal::NotANumber("a SOL price is a positive number of dollars"));
    }
    if key_of(pool).is_none() {
        return Err(Refusal::NotAPool("the pool's address is not one"));
    }
    let key = format!("pool:{epoch}:{pool}");
    let usd = ((lamports as f64 / 1e9) * sol_usd * 1e9).round() / 1e9;
    let why = format!("{key} · the pool's yield, to {tank} · {lamports} lamports at ${sol_usd}/SOL");
    Ok(json!({ "key": key, "pool": pool, "tank": tank, "lamports": lamports, "usd": usd, "why": why }))
}

// ---------------------------------------------------------------- THE WIRE'S BYTES

/// Standard base64 (a node answers an account's bytes in it), padded or not. None for a letter that is not base64's. Pure.
pub fn base64_of(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}
