//! Account state, laid out byte by byte. No macros: the layout you read is
//! the layout on chain. Every account starts with a one-byte discriminator
//! and a one-byte layout version; both are checked on every read. All
//! integers little-endian. Offsets are public so clients and tests read the
//! same numbers the program writes. Large accounts (the registry) are
//! accessed in place: the SBF stack is 4 KB.

use crate::error::PoleisError;
use pinocchio::{
    account::{Ref, RefMut},
    error::ProgramError,
    AccountView, Address,
};

/// Bumped whenever any layout below changes shape. Version 8 gives every
/// config and every registry a root, so one program hosts many trees, and
/// lets a registry grow one holder row at a time instead of being born at
/// full size; version 7 named the default mint in the config and every
/// registry and added the token plan account; version 6 added the crank tip; version 5 gave the
/// registry a CODEX window and a dividend policy, the distribution a mint
/// and a payer, and adds allowances, contracts and listings; version 4 gave
/// the registry a token plan and a name; version 3 recorded every voice in
/// the session; version 2 was the double seal; version 1 is refused.
pub const LAYOUT_VERSION: u8 = 8;

pub const DISC_CONFIG: u8 = 1;
pub const DISC_TREASURY: u8 = 2;
pub const DISC_AUCTION: u8 = 3;
pub const DISC_BID: u8 = 4;
pub const DISC_REGISTRY: u8 = 5;
pub const DISC_SEAT_SALE: u8 = 6;
pub const DISC_SEAT_BID: u8 = 7;
pub const DISC_PROPOSAL: u8 = 8;
pub const DISC_SESSION: u8 = 9;
pub const DISC_DISTRIBUTION: u8 = 10;
// 11 was the dividend receipt; claims are a bitmap in the distribution now
pub const DISC_ALLOWANCE: u8 = 12;
pub const DISC_CONTRACT: u8 = 13;
pub const DISC_LISTING: u8 = 14;
pub const DISC_TOKEN_PLAN: u8 = 15;
pub const DISC_RULE: u8 = 26;
pub const DISC_MARKET: u8 = 27;

pub const OFF_DISC: usize = 0;
pub const OFF_VERSION: usize = 1;

#[inline(always)]
fn rd_u16(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(d[o..o + 2].try_into().unwrap())
}
#[inline(always)]
fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
#[inline(always)]
fn rd_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
#[inline(always)]
fn rd_addr(d: &[u8], o: usize) -> Address {
    Address::new_from_array(d[o..o + 32].try_into().unwrap())
}
#[inline(always)]
fn rd_h32(d: &[u8], o: usize) -> [u8; 32] {
    d[o..o + 32].try_into().unwrap()
}
#[inline(always)]
fn wr_u16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_u32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_u64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_addr(d: &mut [u8], o: usize, v: &Address) {
    d[o..o + 32].copy_from_slice(v.as_ref());
}
#[inline(always)]
fn wr_b32(d: &mut [u8], o: usize, v: &[u8; 32]) {
    d[o..o + 32].copy_from_slice(v);
}
#[inline(always)]
fn stamp(d: &mut [u8], disc: u8) {
    d[OFF_DISC] = disc;
    d[OFF_VERSION] = LAYOUT_VERSION;
}

fn checked<'a>(acc: &'a AccountView, disc: u8, size: usize) -> Result<Ref<'a, [u8]>, ProgramError> {
    if !acc.owned_by(&crate::ID) {
        return Err(PoleisError::InvalidAccount.into());
    }
    let data = acc.try_borrow()?;
    if data.len() != size || data[OFF_DISC] != disc {
        return Err(PoleisError::InvalidAccount.into());
    }
    if data[OFF_VERSION] != LAYOUT_VERSION {
        return Err(PoleisError::LayoutVersion.into());
    }
    Ok(data)
}

fn checked_mut<'a>(acc: &'a mut AccountView, disc: u8, size: usize) -> Result<RefMut<'a, [u8]>, ProgramError> {
    if !acc.owned_by(&crate::ID) {
        return Err(PoleisError::InvalidAccount.into());
    }
    let data = acc.try_borrow_mut()?;
    if data.len() != size || data[OFF_DISC] != disc {
        return Err(PoleisError::InvalidAccount.into());
    }
    if data[OFF_VERSION] != LAYOUT_VERSION {
        return Err(PoleisError::LayoutVersion.into());
    }
    Ok(data)
}

/// A unit that holds a seat or a chair is named by its id in the first eight
/// bytes of a 32-byte holder field, so keys and units share one lookup.
pub fn unit_holder(unit_id: u64) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[..8].copy_from_slice(&unit_id.to_le_bytes());
    h
}
pub fn holder_unit(h: &[u8; 32]) -> u64 {
    u64::from_le_bytes(h[..8].try_into().unwrap())
}

// ---- Config: [disc:1][ver:1][authority:32][bump:1][mode:1][default_mint:32][root:8] ----
// One per tree, seeded by the root unit's id: one program hosts every
// polis. The default mint (USDC on a real cluster) is named once here and
// every unit of the tree inherits it: spending it needs no listing. Zero
// means no default.
pub struct Config {
    pub authority: Address,
    pub bump: u8,
    pub mode: u8,
    pub default_mint: [u8; 32],
    pub root: u64,
}
impl Config {
    pub const SIZE: usize = 76;
    pub const OFF_AUTHORITY: usize = 2;
    pub const OFF_BUMP: usize = 34;
    pub const OFF_MODE: usize = 35;
    pub const OFF_DEFAULT_MINT: usize = 36;
    pub const OFF_ROOT: usize = 68;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_CONFIG, Self::SIZE)?;
        Ok(Self {
            authority: rd_addr(&d, Self::OFF_AUTHORITY),
            bump: d[Self::OFF_BUMP],
            mode: d[Self::OFF_MODE],
            default_mint: rd_h32(&d, Self::OFF_DEFAULT_MINT),
            root: rd_u64(&d, Self::OFF_ROOT),
        })
    }
    pub fn init(acc: &mut AccountView, authority: &Address, bump: u8, default_mint: &[u8; 32], root: u64) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_CONFIG);
        wr_addr(&mut d, Self::OFF_AUTHORITY, authority);
        d[Self::OFF_BUMP] = bump;
        d[Self::OFF_MODE] = crate::MODE_SOLO;
        wr_b32(&mut d, Self::OFF_DEFAULT_MINT, default_mint);
        wr_u64(&mut d, Self::OFF_ROOT, root);
        Ok(())
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = checked_mut(acc, DISC_CONFIG, Self::SIZE)?;
        wr_addr(&mut d, Self::OFF_AUTHORITY, &self.authority);
        d[Self::OFF_MODE] = self.mode;
        Ok(())
    }
}

// ---- Treasury: [disc:1][ver:1][unit_id:8][total_in:8][total_out:8][bump:1] ----
pub struct Treasury {
    pub unit_id: u64,
    pub total_in: u64,
    pub total_out: u64,
    pub bump: u8,
}
impl Treasury {
    pub const SIZE: usize = 27;
    pub const OFF_UNIT: usize = 2;
    pub const OFF_TOTAL_IN: usize = 10;
    pub const OFF_TOTAL_OUT: usize = 18;
    pub const OFF_BUMP: usize = 26;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_TREASURY, Self::SIZE)?;
        Ok(Self {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            total_in: rd_u64(&d, Self::OFF_TOTAL_IN),
            total_out: rd_u64(&d, Self::OFF_TOTAL_OUT),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn init(acc: &mut AccountView, unit_id: u64, bump: u8) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_TREASURY);
        wr_u64(&mut d, Self::OFF_UNIT, unit_id);
        wr_u64(&mut d, Self::OFF_TOTAL_IN, 0);
        wr_u64(&mut d, Self::OFF_TOTAL_OUT, 0);
        d[Self::OFF_BUMP] = bump;
        Ok(())
    }
    pub fn add_in(acc: &mut AccountView, amount: u64) -> Result<(), ProgramError> {
        let mut d = checked_mut(acc, DISC_TREASURY, Self::SIZE)?;
        let v = rd_u64(&d, Self::OFF_TOTAL_IN).checked_add(amount).ok_or(PoleisError::MathOverflow)?;
        wr_u64(&mut d, Self::OFF_TOTAL_IN, v);
        Ok(())
    }
    pub fn add_out(acc: &mut AccountView, amount: u64) -> Result<(), ProgramError> {
        let mut d = checked_mut(acc, DISC_TREASURY, Self::SIZE)?;
        let v = rd_u64(&d, Self::OFF_TOTAL_OUT).checked_add(amount).ok_or(PoleisError::MathOverflow)?;
        wr_u64(&mut d, Self::OFF_TOTAL_OUT, v);
        Ok(())
    }
    /// Used only by sync_endowment, which can never lower the number.
    pub fn set_in(acc: &mut AccountView, total_in: u64) -> Result<(), ProgramError> {
        let mut d = checked_mut(acc, DISC_TREASURY, Self::SIZE)?;
        wr_u64(&mut d, Self::OFF_TOTAL_IN, total_in);
        Ok(())
    }
}

// ---- Rule: a unit's own law on how a split is settled. [disc:1][ver:1][unit_id:8][sum_kinds:1][small_cap:8][bump:1] ----
pub struct Rule {
    pub unit_id: u64,
    pub sum_kinds: u8,
    pub small_cap: u64,
    pub bump: u8,
}
impl Rule {
    pub const SIZE: usize = 20;
    pub const OFF_UNIT: usize = 2;
    pub const OFF_SUM_KINDS: usize = 10;
    pub const OFF_SMALL_CAP: usize = 11;
    pub const OFF_BUMP: usize = 19;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_RULE, Self::SIZE)?;
        Ok(Self {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            sum_kinds: d[Self::OFF_SUM_KINDS],
            small_cap: rd_u64(&d, Self::OFF_SMALL_CAP),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_RULE);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        d[Self::OFF_SUM_KINDS] = self.sum_kinds;
        wr_u64(&mut d, Self::OFF_SMALL_CAP, self.small_cap);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ---- Market: a unit's own law on its market, at ["market", unit], written only by its CODEX act (`CODEX_MARKET`).
// [disc:1][ver:1][unit_id:8][program:32][board:8][split:1][plan:1][min_stood_bp:2][since:8][bump:1] ----
// `program` and `board`: the roads program and the board whose roads this unit reads. `split`: the classes (`MARKET_*`) whose split of
// the two chambers goes to the market. `plan`: the classes that may be plans. `min_stood_bp`: the least share of everything credited on
// that board when the road was laid that must have stood in it, as deposits, for its verdict to count (0: any road counts). `since`: the
// slot it was last written; a law written in or after the slot a proposal opened in says nothing in that proposal's seal (a polis that
// sets its market's law and proposes under it does so in a later slot).
pub struct Market {
    pub unit_id: u64,
    pub program: [u8; 32],
    pub board: u64,
    pub split: u8,
    pub plan: u8,
    pub min_stood_bp: u16,
    pub since: u64,
    pub bump: u8,
}
impl Market {
    pub const SIZE: usize = 63;
    pub const OFF_UNIT: usize = 2;
    pub const OFF_PROGRAM: usize = 10;
    pub const OFF_BOARD: usize = 42;
    pub const OFF_SPLIT: usize = 50;
    pub const OFF_PLAN: usize = 51;
    pub const OFF_MIN_STOOD_BP: usize = 52;
    pub const OFF_SINCE: usize = 54;
    pub const OFF_BUMP: usize = 62;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_MARKET, Self::SIZE)?;
        Ok(Self {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            program: rd_h32(&d, Self::OFF_PROGRAM),
            board: rd_u64(&d, Self::OFF_BOARD),
            split: d[Self::OFF_SPLIT],
            plan: d[Self::OFF_PLAN],
            min_stood_bp: rd_u16(&d, Self::OFF_MIN_STOOD_BP),
            since: rd_u64(&d, Self::OFF_SINCE),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_MARKET);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_PROGRAM, &self.program);
        wr_u64(&mut d, Self::OFF_BOARD, self.board);
        d[Self::OFF_SPLIT] = self.split;
        d[Self::OFF_PLAN] = self.plan;
        wr_u16(&mut d, Self::OFF_MIN_STOOD_BP, self.min_stood_bp);
        wr_u64(&mut d, Self::OFF_SINCE, self.since);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}
const _: () = assert!(Market::OFF_BUMP + 1 == Market::SIZE && Market::OFF_SINCE == Market::OFF_MIN_STOOD_BP + 2);

// ---- Auction (the trace lot sale) ----
// [disc:1][ver:1][auction_id:8][unit_id:8][pool_root:32][manifest_hash:32]
// [commit_end_slot:8][reveal_end_slot:8][reserve_price:8][best_bid:8]
// [second_bid:8][clearing_price:8][best_bidder:32][settled:1][bump:1][deposit:8]
pub struct Auction {
    pub auction_id: u64,
    pub unit_id: u64,
    pub pool_root: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub commit_end_slot: u64,
    pub reveal_end_slot: u64,
    pub reserve_price: u64,
    pub best_bid: u64,
    pub second_bid: u64,
    pub clearing_price: u64,
    pub best_bidder: Address,
    pub settled: bool,
    pub bump: u8,
    pub deposit: u64,
}
impl Auction {
    pub const SIZE: usize = 172;
    pub const OFF_AUCTION_ID: usize = 2;
    pub const OFF_UNIT: usize = 10;
    pub const OFF_POOL_ROOT: usize = 18;
    pub const OFF_MANIFEST: usize = 50;
    pub const OFF_COMMIT_END: usize = 82;
    pub const OFF_REVEAL_END: usize = 90;
    pub const OFF_RESERVE: usize = 98;
    pub const OFF_BEST_BID: usize = 106;
    pub const OFF_SECOND_BID: usize = 114;
    pub const OFF_CLEARING: usize = 122;
    pub const OFF_BEST_BIDDER: usize = 130;
    pub const OFF_SETTLED: usize = 162;
    pub const OFF_BUMP: usize = 163;
    pub const OFF_DEPOSIT: usize = 164;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_AUCTION, Self::SIZE)?;
        Ok(Self {
            auction_id: rd_u64(&d, Self::OFF_AUCTION_ID),
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            pool_root: rd_h32(&d, Self::OFF_POOL_ROOT),
            manifest_hash: rd_h32(&d, Self::OFF_MANIFEST),
            commit_end_slot: rd_u64(&d, Self::OFF_COMMIT_END),
            reveal_end_slot: rd_u64(&d, Self::OFF_REVEAL_END),
            reserve_price: rd_u64(&d, Self::OFF_RESERVE),
            best_bid: rd_u64(&d, Self::OFF_BEST_BID),
            second_bid: rd_u64(&d, Self::OFF_SECOND_BID),
            clearing_price: rd_u64(&d, Self::OFF_CLEARING),
            best_bidder: rd_addr(&d, Self::OFF_BEST_BIDDER),
            settled: d[Self::OFF_SETTLED] != 0,
            bump: d[Self::OFF_BUMP],
            deposit: rd_u64(&d, Self::OFF_DEPOSIT),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_AUCTION);
        wr_u64(&mut d, Self::OFF_AUCTION_ID, self.auction_id);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_POOL_ROOT, &self.pool_root);
        wr_b32(&mut d, Self::OFF_MANIFEST, &self.manifest_hash);
        wr_u64(&mut d, Self::OFF_COMMIT_END, self.commit_end_slot);
        wr_u64(&mut d, Self::OFF_REVEAL_END, self.reveal_end_slot);
        wr_u64(&mut d, Self::OFF_RESERVE, self.reserve_price);
        wr_u64(&mut d, Self::OFF_BEST_BID, self.best_bid);
        wr_u64(&mut d, Self::OFF_SECOND_BID, self.second_bid);
        wr_u64(&mut d, Self::OFF_CLEARING, self.clearing_price);
        wr_addr(&mut d, Self::OFF_BEST_BIDDER, &self.best_bidder);
        d[Self::OFF_SETTLED] = self.settled as u8;
        d[Self::OFF_BUMP] = self.bump;
        wr_u64(&mut d, Self::OFF_DEPOSIT, self.deposit);
        Ok(())
    }
}

// ---- Bid (a lot bid) ----
// [disc:1][ver:1][auction_id:8][bidder:32][commitment:32][deposit:8][amount:8]
// [revealed:1][reclaimed:1][bump:1]
pub struct Bid {
    pub auction_id: u64,
    pub bidder: Address,
    pub commitment: [u8; 32],
    pub deposit: u64,
    pub amount: u64,
    pub revealed: bool,
    pub reclaimed: bool,
    pub bump: u8,
}
impl Bid {
    pub const SIZE: usize = 93;
    pub const OFF_AUCTION_ID: usize = 2;
    pub const OFF_BIDDER: usize = 10;
    pub const OFF_COMMITMENT: usize = 42;
    pub const OFF_DEPOSIT: usize = 74;
    pub const OFF_AMOUNT: usize = 82;
    pub const OFF_REVEALED: usize = 90;
    pub const OFF_RECLAIMED: usize = 91;
    pub const OFF_BUMP: usize = 92;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_BID, Self::SIZE)?;
        Ok(Self {
            auction_id: rd_u64(&d, Self::OFF_AUCTION_ID),
            bidder: rd_addr(&d, Self::OFF_BIDDER),
            commitment: rd_h32(&d, Self::OFF_COMMITMENT),
            deposit: rd_u64(&d, Self::OFF_DEPOSIT),
            amount: rd_u64(&d, Self::OFF_AMOUNT),
            revealed: d[Self::OFF_REVEALED] != 0,
            reclaimed: d[Self::OFF_RECLAIMED] != 0,
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_BID);
        wr_u64(&mut d, Self::OFF_AUCTION_ID, self.auction_id);
        wr_addr(&mut d, Self::OFF_BIDDER, &self.bidder);
        wr_b32(&mut d, Self::OFF_COMMITMENT, &self.commitment);
        wr_u64(&mut d, Self::OFF_DEPOSIT, self.deposit);
        wr_u64(&mut d, Self::OFF_AMOUNT, self.amount);
        d[Self::OFF_REVEALED] = self.revealed as u8;
        d[Self::OFF_RECLAIMED] = self.reclaimed as u8;
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ============================ the double seal ============================

// ---- Registry ----
// One per unit: two chambers, the plan, the ladder rung, the charter, and
// the chain of seals. Accessed in place. Born with the head and the twelve
// chairs only; grows one holder row at a time, each row's rent paid by the
// holder it seats, never past MAX_HOLDERS.
// [disc:1][ver:1][unit_id:8][parent_unit:8][chair_in_parent:1][level:1]
// [plan_cap:8][plan_period:8][plan_spent:8][plan_start:8][rung:1][frozen:1]
// [total_seats:8][charter_hash:32][chain_sig:32][holder_count:1][bump:1]
// [window:8][next_sale_id:8][seat_reserve:8][seat_deposit:8][last_clearing:8]
// [token_plan_cap:8][token_plan_spent:8]
// [codex_window:8][dividend_bps:2][dividend_period:8][dividend_start:8][accrued_in:8][dividend_seq:8]
// [crank_tip:8][default_mint:32][root:8][name_len:1][name:MAX_NAME][desc_len:1][desc:MAX_DESC]
// [pentagons: 12 × (holder:32, kind:1, seated_slot:8, term_end:8)]
// [holders: capacity × (holder:32, kind:1, seats:8, weight:8, last_active:8)]
pub struct Registry;

pub struct RegistryHead {
    pub unit_id: u64,
    pub parent_unit: u64,
    pub chair_in_parent: u8,
    pub level: u8,
    pub plan_cap: u64,
    pub plan_period: u64,
    pub plan_spent: u64,
    pub plan_start: u64,
    pub rung: u8,
    pub frozen: bool,
    pub total_seats: u64,
    pub charter_hash: [u8; 32],
    pub chain_sig: [u8; 32],
    pub holder_count: u8,
    pub bump: u8,
    pub window: u64,
    pub next_sale_id: u64,
    pub seat_reserve: u64,
    pub seat_deposit: u64,
    pub last_clearing: u64,
    pub token_plan_cap: u64,
    pub token_plan_spent: u64,
    /// The voting window of CODEX sessions: a society rewrites itself slowly.
    pub codex_window: u64,
    /// The dividend policy: this fraction (basis points) of every lamport of
    /// income since the last accrual is escrowed for the floor each period.
    pub dividend_bps: u16,
    pub dividend_period: u64,
    pub dividend_start: u64,
    /// The treasury's total_in already accrued against.
    pub accrued_in: u64,
    /// Accruals made so far; seeds each accrual's distribution.
    pub dividend_seq: u64,
    /// Paid to whoever lands a batch or a seal, from the treasury, under the plan.
    pub crank_tip: u64,
    /// The mint charged to the registry's own token counter; inherited from the config.
    pub default_mint: [u8; 32],
    /// The root unit of the tree this unit belongs to: the config's key.
    pub root: u64,
}

#[derive(Clone, Copy)]
pub struct Pentagon {
    pub holder: [u8; 32],
    pub kind: u8,
    pub seated_slot: u64,
    pub term_end: u64,
}

#[derive(Clone, Copy)]
pub struct Holder {
    pub holder: [u8; 32],
    pub kind: u8,
    pub seats: u64,
    pub weight: u64,
    pub last_active: u64,
}

impl Registry {
    pub const OFF_UNIT: usize = 2;
    pub const OFF_PARENT: usize = 10;
    pub const OFF_CHAIR: usize = 18;
    pub const OFF_LEVEL: usize = 19;
    pub const OFF_PLAN_CAP: usize = 20;
    pub const OFF_PLAN_PERIOD: usize = 28;
    pub const OFF_PLAN_SPENT: usize = 36;
    pub const OFF_PLAN_START: usize = 44;
    pub const OFF_RUNG: usize = 52;
    pub const OFF_FROZEN: usize = 53;
    pub const OFF_TOTAL_SEATS: usize = 54;
    pub const OFF_CHARTER: usize = 62;
    pub const OFF_CHAIN_SIG: usize = 94;
    pub const OFF_HOLDER_COUNT: usize = 126;
    pub const OFF_BUMP: usize = 127;
    pub const OFF_WINDOW: usize = 128;
    pub const OFF_NEXT_SALE: usize = 136;
    pub const OFF_SEAT_RESERVE: usize = 144;
    pub const OFF_SEAT_DEPOSIT: usize = 152;
    pub const OFF_LAST_CLEARING: usize = 160;
    pub const OFF_TOKEN_PLAN_CAP: usize = 168;
    pub const OFF_TOKEN_PLAN_SPENT: usize = 176;
    pub const OFF_CODEX_WINDOW: usize = 184;
    pub const OFF_DIVIDEND_BPS: usize = 192;
    pub const OFF_DIVIDEND_PERIOD: usize = 194;
    pub const OFF_DIVIDEND_START: usize = 202;
    pub const OFF_ACCRUED_IN: usize = 210;
    pub const OFF_DIVIDEND_SEQ: usize = 218;
    pub const OFF_CRANK_TIP: usize = 226;
    pub const OFF_DEFAULT_MINT: usize = 234;
    pub const OFF_ROOT: usize = 266;
    pub const OFF_NAME_LEN: usize = 274;
    pub const OFF_NAME: usize = 275;
    pub const OFF_DESC_LEN: usize = Self::OFF_NAME + crate::MAX_NAME;
    pub const OFF_DESC: usize = Self::OFF_DESC_LEN + 1;
    pub const OFF_PENTAGONS: usize = Self::OFF_DESC + crate::MAX_DESC;
    pub const PENTAGON_LEN: usize = 49;
    pub const OFF_HOLDERS: usize = Self::OFF_PENTAGONS + crate::PENTAGONS * Self::PENTAGON_LEN;
    pub const HOLDER_LEN: usize = 57;
    /// A registry at birth: the head and the chairs, no holder rows yet.
    pub const MIN_SIZE: usize = Self::OFF_HOLDERS;
    /// A registry that has grown to the cap.
    pub const MAX_SIZE: usize = Self::OFF_HOLDERS + crate::MAX_HOLDERS * Self::HOLDER_LEN;
    /// The size that seats `rows` holders.
    pub const fn size_for(rows: usize) -> usize {
        Self::OFF_HOLDERS + rows * Self::HOLDER_LEN
    }

    /// A registry is owned, stamped, and holds a whole number of rows.
    fn check(acc: &AccountView) -> Result<Ref<'_, [u8]>, ProgramError> {
        if !acc.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        let data = acc.try_borrow()?;
        let n = data.len();
        if !(Self::MIN_SIZE..=Self::MAX_SIZE).contains(&n) || !(n - Self::MIN_SIZE).is_multiple_of(Self::HOLDER_LEN) || data[OFF_DISC] != DISC_REGISTRY {
            return Err(PoleisError::InvalidAccount.into());
        }
        if data[OFF_VERSION] != LAYOUT_VERSION {
            return Err(PoleisError::LayoutVersion.into());
        }
        Ok(data)
    }
    fn check_mut(acc: &mut AccountView) -> Result<RefMut<'_, [u8]>, ProgramError> {
        if !acc.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        let data = acc.try_borrow_mut()?;
        let n = data.len();
        if !(Self::MIN_SIZE..=Self::MAX_SIZE).contains(&n) || !(n - Self::MIN_SIZE).is_multiple_of(Self::HOLDER_LEN) || data[OFF_DISC] != DISC_REGISTRY {
            return Err(PoleisError::InvalidAccount.into());
        }
        if data[OFF_VERSION] != LAYOUT_VERSION {
            return Err(PoleisError::LayoutVersion.into());
        }
        Ok(data)
    }
    /// Holder rows the account has room for.
    pub fn capacity(acc: &AccountView) -> usize {
        acc.data_len().saturating_sub(Self::MIN_SIZE) / Self::HOLDER_LEN
    }

    #[allow(clippy::too_many_arguments)]
    pub fn init(
        acc: &mut AccountView,
        unit_id: u64,
        parent_unit: u64,
        chair_in_parent: u8,
        level: u8,
        charter_hash: &[u8; 32],
        window: u64,
        plan_cap: u64,
        plan_period: u64,
        now: u64,
        bump: u8,
        default_mint: &[u8; 32],
        root: u64,
    ) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_REGISTRY);
        wr_u64(&mut d, Self::OFF_UNIT, unit_id);
        wr_u64(&mut d, Self::OFF_PARENT, parent_unit);
        d[Self::OFF_CHAIR] = chair_in_parent;
        d[Self::OFF_LEVEL] = level;
        wr_u64(&mut d, Self::OFF_PLAN_CAP, plan_cap);
        wr_u64(&mut d, Self::OFF_PLAN_PERIOD, plan_period);
        wr_u64(&mut d, Self::OFF_PLAN_SPENT, 0);
        wr_u64(&mut d, Self::OFF_PLAN_START, now);
        d[Self::OFF_RUNG] = 0;
        d[Self::OFF_FROZEN] = 0;
        wr_u64(&mut d, Self::OFF_TOTAL_SEATS, 0);
        wr_b32(&mut d, Self::OFF_CHARTER, charter_hash);
        wr_b32(&mut d, Self::OFF_CHAIN_SIG, &[0u8; 32]);
        d[Self::OFF_HOLDER_COUNT] = 0;
        d[Self::OFF_BUMP] = bump;
        wr_u64(&mut d, Self::OFF_WINDOW, window);
        wr_u64(&mut d, Self::OFF_NEXT_SALE, 0);
        wr_u64(&mut d, Self::OFF_SEAT_RESERVE, 0);
        wr_u64(&mut d, Self::OFF_SEAT_DEPOSIT, 0);
        wr_u64(&mut d, Self::OFF_LAST_CLEARING, 0);
        wr_u64(&mut d, Self::OFF_TOKEN_PLAN_CAP, 0);
        wr_u64(&mut d, Self::OFF_TOKEN_PLAN_SPENT, 0);
        // CODEX takes three windows by default, never past the ceiling
        let codex_window = window.saturating_mul(crate::CODEX_WINDOW_FACTOR).min(crate::MAX_WINDOW_SLOTS);
        wr_u64(&mut d, Self::OFF_CODEX_WINDOW, codex_window);
        wr_u16(&mut d, Self::OFF_DIVIDEND_BPS, 0);
        wr_u64(&mut d, Self::OFF_DIVIDEND_PERIOD, 0);
        wr_u64(&mut d, Self::OFF_DIVIDEND_START, now);
        wr_u64(&mut d, Self::OFF_ACCRUED_IN, 0);
        wr_u64(&mut d, Self::OFF_DIVIDEND_SEQ, 0);
        wr_u64(&mut d, Self::OFF_CRANK_TIP, 0);
        wr_b32(&mut d, Self::OFF_DEFAULT_MINT, default_mint);
        wr_u64(&mut d, Self::OFF_ROOT, root);
        d[Self::OFF_NAME_LEN] = 0;
        d[Self::OFF_DESC_LEN] = 0;
        Ok(())
    }

    /// The society's name and description, on chain, replaced whole.
    pub fn set_name(acc: &mut AccountView, name: &[u8], desc: &[u8]) -> Result<(), ProgramError> {
        if name.len() > crate::MAX_NAME || desc.len() > crate::MAX_DESC {
            return Err(ProgramError::InvalidInstructionData);
        }
        let mut d = Self::check_mut(acc)?;
        d[Self::OFF_NAME_LEN] = name.len() as u8;
        d[Self::OFF_NAME..Self::OFF_NAME + crate::MAX_NAME].fill(0);
        d[Self::OFF_NAME..Self::OFF_NAME + name.len()].copy_from_slice(name);
        d[Self::OFF_DESC_LEN] = desc.len() as u8;
        d[Self::OFF_DESC..Self::OFF_DESC + crate::MAX_DESC].fill(0);
        d[Self::OFF_DESC..Self::OFF_DESC + desc.len()].copy_from_slice(desc);
        Ok(())
    }

    pub fn head(acc: &AccountView) -> Result<RegistryHead, ProgramError> {
        let d = Self::check(acc)?;
        Ok(RegistryHead {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            parent_unit: rd_u64(&d, Self::OFF_PARENT),
            chair_in_parent: d[Self::OFF_CHAIR],
            level: d[Self::OFF_LEVEL],
            plan_cap: rd_u64(&d, Self::OFF_PLAN_CAP),
            plan_period: rd_u64(&d, Self::OFF_PLAN_PERIOD),
            plan_spent: rd_u64(&d, Self::OFF_PLAN_SPENT),
            plan_start: rd_u64(&d, Self::OFF_PLAN_START),
            rung: d[Self::OFF_RUNG],
            frozen: d[Self::OFF_FROZEN] != 0,
            total_seats: rd_u64(&d, Self::OFF_TOTAL_SEATS),
            charter_hash: rd_h32(&d, Self::OFF_CHARTER),
            chain_sig: rd_h32(&d, Self::OFF_CHAIN_SIG),
            holder_count: d[Self::OFF_HOLDER_COUNT],
            bump: d[Self::OFF_BUMP],
            window: rd_u64(&d, Self::OFF_WINDOW),
            next_sale_id: rd_u64(&d, Self::OFF_NEXT_SALE),
            seat_reserve: rd_u64(&d, Self::OFF_SEAT_RESERVE),
            seat_deposit: rd_u64(&d, Self::OFF_SEAT_DEPOSIT),
            last_clearing: rd_u64(&d, Self::OFF_LAST_CLEARING),
            token_plan_cap: rd_u64(&d, Self::OFF_TOKEN_PLAN_CAP),
            token_plan_spent: rd_u64(&d, Self::OFF_TOKEN_PLAN_SPENT),
            codex_window: rd_u64(&d, Self::OFF_CODEX_WINDOW),
            dividend_bps: rd_u16(&d, Self::OFF_DIVIDEND_BPS),
            dividend_period: rd_u64(&d, Self::OFF_DIVIDEND_PERIOD),
            dividend_start: rd_u64(&d, Self::OFF_DIVIDEND_START),
            accrued_in: rd_u64(&d, Self::OFF_ACCRUED_IN),
            dividend_seq: rd_u64(&d, Self::OFF_DIVIDEND_SEQ),
            crank_tip: rd_u64(&d, Self::OFF_CRANK_TIP),
            default_mint: rd_h32(&d, Self::OFF_DEFAULT_MINT),
            root: rd_u64(&d, Self::OFF_ROOT),
        })
    }

    /// Writes the mutable header fields (identity, level and bump never change).
    pub fn set_head(acc: &mut AccountView, h: &RegistryHead) -> Result<(), ProgramError> {
        let mut d = Self::check_mut(acc)?;
        wr_u64(&mut d, Self::OFF_PLAN_CAP, h.plan_cap);
        wr_u64(&mut d, Self::OFF_PLAN_PERIOD, h.plan_period);
        wr_u64(&mut d, Self::OFF_PLAN_SPENT, h.plan_spent);
        wr_u64(&mut d, Self::OFF_PLAN_START, h.plan_start);
        d[Self::OFF_RUNG] = h.rung;
        d[Self::OFF_FROZEN] = h.frozen as u8;
        wr_u64(&mut d, Self::OFF_TOTAL_SEATS, h.total_seats);
        wr_b32(&mut d, Self::OFF_CHAIN_SIG, &h.chain_sig);
        d[Self::OFF_HOLDER_COUNT] = h.holder_count;
        wr_u64(&mut d, Self::OFF_WINDOW, h.window);
        wr_u64(&mut d, Self::OFF_NEXT_SALE, h.next_sale_id);
        wr_u64(&mut d, Self::OFF_SEAT_RESERVE, h.seat_reserve);
        wr_u64(&mut d, Self::OFF_SEAT_DEPOSIT, h.seat_deposit);
        wr_u64(&mut d, Self::OFF_LAST_CLEARING, h.last_clearing);
        wr_u64(&mut d, Self::OFF_TOKEN_PLAN_CAP, h.token_plan_cap);
        wr_u64(&mut d, Self::OFF_TOKEN_PLAN_SPENT, h.token_plan_spent);
        wr_u64(&mut d, Self::OFF_CODEX_WINDOW, h.codex_window);
        wr_u16(&mut d, Self::OFF_DIVIDEND_BPS, h.dividend_bps);
        wr_u64(&mut d, Self::OFF_DIVIDEND_PERIOD, h.dividend_period);
        wr_u64(&mut d, Self::OFF_DIVIDEND_START, h.dividend_start);
        wr_u64(&mut d, Self::OFF_ACCRUED_IN, h.accrued_in);
        wr_u64(&mut d, Self::OFF_DIVIDEND_SEQ, h.dividend_seq);
        wr_u64(&mut d, Self::OFF_CRANK_TIP, h.crank_tip);
        Ok(())
    }

    // ---- pentagons ----

    fn pent_off(k: usize) -> usize {
        Self::OFF_PENTAGONS + k * Self::PENTAGON_LEN
    }

    pub fn pentagon(acc: &AccountView, k: usize) -> Result<Pentagon, ProgramError> {
        let d = Self::check(acc)?;
        if k >= crate::PENTAGONS {
            return Err(PoleisError::InvalidHolder.into());
        }
        let o = Self::pent_off(k);
        Ok(Pentagon {
            holder: rd_h32(&d, o),
            kind: d[o + 32],
            seated_slot: rd_u64(&d, o + 33),
            term_end: rd_u64(&d, o + 41),
        })
    }

    pub fn set_pentagon(acc: &mut AccountView, k: usize, p: &Pentagon) -> Result<(), ProgramError> {
        let mut d = Self::check_mut(acc)?;
        if k >= crate::PENTAGONS {
            return Err(PoleisError::InvalidHolder.into());
        }
        let o = Self::pent_off(k);
        wr_b32(&mut d, o, &p.holder);
        d[o + 32] = p.kind;
        wr_u64(&mut d, o + 33, p.seated_slot);
        wr_u64(&mut d, o + 41, p.term_end);
        Ok(())
    }

    /// The chairs a key holds, as a twelve-bit mask, expired terms excluded.
    pub fn chairs_of(acc: &AccountView, key: &[u8; 32], now: u64) -> Result<u16, ProgramError> {
        let d = Self::check(acc)?;
        let mut mask = 0u16;
        for k in 0..crate::PENTAGONS {
            let o = Self::pent_off(k);
            if d[o + 32] == crate::HOLDER_KEY && &d[o..o + 32] == key {
                let term = rd_u64(&d, o + 41);
                if term == 0 || now <= term {
                    mask |= 1 << k;
                }
            }
        }
        Ok(mask)
    }

    /// The chairs lit now, as a twelve-bit mask: held by a key, a unit or an
    /// NFT, terms unexpired. A session keeps this mask as it stood at its
    /// opening (`Session::roster`).
    pub fn lit_mask(acc: &AccountView, now: u64) -> Result<u16, ProgramError> {
        let d = Self::check(acc)?;
        let mut mask = 0u16;
        for k in 0..crate::PENTAGONS {
            let o = Self::pent_off(k);
            let term = rd_u64(&d, o + 41);
            if d[o + 32] != crate::HOLDER_EMPTY && (term == 0 || now <= term) {
                mask |= 1 << k;
            }
        }
        Ok(mask)
    }

    /// The chairs lit now, counted: the one rule of `lit_mask`.
    /// The chairs' chamber is a majority of these (`crate::chair_quorum`).
    pub fn lit_chairs(acc: &AccountView, now: u64) -> Result<u8, ProgramError> {
        Ok(Self::lit_mask(acc, now)?.count_ones() as u8)
    }

    /// The chairs nobody has touched since `opened`, as a twelve-bit mask:
    /// every founding, attachment and recall writes `seated_slot = now`, so a
    /// chair seated, reseated or emptied after a session opened has a later
    /// slot and is not the chair that session's roster names.
    pub fn unchanged_since(acc: &AccountView, opened: u64) -> Result<u16, ProgramError> {
        let d = Self::check(acc)?;
        let mut mask = 0u16;
        for k in 0..crate::PENTAGONS {
            if rd_u64(&d, Self::pent_off(k) + 33) <= opened {
                mask |= 1 << k;
            }
        }
        Ok(mask)
    }

    /// The lit chairs a unit holds, as a twelve-bit mask: where its seal
    /// speaks (`mandate`), once in each, as a key's vote does.
    pub fn chairs_of_unit(acc: &AccountView, unit_id: u64, now: u64) -> Result<u16, ProgramError> {
        let d = Self::check(acc)?;
        let h = unit_holder(unit_id);
        let mut mask = 0u16;
        for k in 0..crate::PENTAGONS {
            let o = Self::pent_off(k);
            let term = rd_u64(&d, o + 41);
            if d[o + 32] == crate::HOLDER_UNIT && d[o..o + 32] == h && (term == 0 || now <= term) {
                mask |= 1 << k;
            }
        }
        Ok(mask)
    }

    /// The lit chairs that seat a unit, as a twelve-bit mask: the crews this
    /// unit's every seal waits for, whatever opened their sessions.
    pub fn crew_chairs(acc: &AccountView, now: u64) -> Result<u16, ProgramError> {
        let d = Self::check(acc)?;
        let mut mask = 0u16;
        for k in 0..crate::PENTAGONS {
            let o = Self::pent_off(k);
            let term = rd_u64(&d, o + 41);
            if d[o + 32] == crate::HOLDER_UNIT && (term == 0 || now <= term) {
                mask |= 1 << k;
            }
        }
        Ok(mask)
    }

    // ---- hexagons ----

    fn holder_off(i: usize) -> usize {
        Self::OFF_HOLDERS + i * Self::HOLDER_LEN
    }

    pub fn holder(acc: &AccountView, i: usize) -> Result<Holder, ProgramError> {
        let d = Self::check(acc)?;
        if i >= d[Self::OFF_HOLDER_COUNT] as usize {
            return Err(PoleisError::NotHolder.into());
        }
        let o = Self::holder_off(i);
        Ok(Holder {
            holder: rd_h32(&d, o),
            kind: d[o + 32],
            seats: rd_u64(&d, o + 33),
            weight: rd_u64(&d, o + 41),
            last_active: rd_u64(&d, o + 49),
        })
    }

    pub fn set_holder(acc: &mut AccountView, i: usize, h: &Holder) -> Result<(), ProgramError> {
        let mut d = Self::check_mut(acc)?;
        if i >= crate::MAX_HOLDERS || Self::holder_off(i) + Self::HOLDER_LEN > d.len() {
            return Err(PoleisError::RegistryFull.into());
        }
        let o = Self::holder_off(i);
        wr_b32(&mut d, o, &h.holder);
        d[o + 32] = h.kind;
        wr_u64(&mut d, o + 33, h.seats);
        wr_u64(&mut d, o + 41, h.weight);
        wr_u64(&mut d, o + 49, h.last_active);
        Ok(())
    }

    pub fn find_holder(acc: &AccountView, kind: u8, holder: &[u8; 32]) -> Result<Option<usize>, ProgramError> {
        let d = Self::check(acc)?;
        let n = d[Self::OFF_HOLDER_COUNT] as usize;
        for i in 0..n {
            let o = Self::holder_off(i);
            if d[o + 32] == kind && &d[o..o + 32] == holder {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// Find-or-insert, then add seats; new holders weigh one; active now.
    /// A new holder past the account's capacity grows the account by one
    /// row, its rent paid by `payer`: the floor pays for its own rows.
    pub fn add_seats(
        acc: &mut AccountView,
        payer: &AccountView,
        kind: u8,
        holder: &[u8; 32],
        seats: u64,
        now: u64,
    ) -> Result<(), ProgramError> {
        Self::add_seats_with(acc, kind, holder, seats, now, |acc, size| crate::util::grow_account(payer, acc, size))
    }

    /// The same, with the row's rent found by `grow`: a signer's system
    /// transfer, or lamports an escrow of this program already holds.
    pub fn add_seats_with(
        acc: &mut AccountView,
        kind: u8,
        holder: &[u8; 32],
        seats: u64,
        now: u64,
        grow: impl FnOnce(&mut AccountView, usize) -> Result<(), ProgramError>,
    ) -> Result<(), ProgramError> {
        if kind != crate::HOLDER_KEY && kind != crate::HOLDER_UNIT {
            return Err(PoleisError::InvalidHolder.into());
        }
        let mut head = Self::head(acc)?;
        let entry = match Self::find_holder(acc, kind, holder)? {
            Some(i) => {
                let mut h = Self::holder(acc, i)?;
                h.seats = h.seats.checked_add(seats).ok_or(PoleisError::MathOverflow)?;
                h.last_active = now;
                (i, h)
            }
            None => {
                // a row a leaver emptied is taken first; only then a new row
                let i = match Self::first_empty(acc)? {
                    Some(i) => i,
                    None => {
                        let i = head.holder_count as usize;
                        if i >= crate::MAX_HOLDERS {
                            return Err(PoleisError::RegistryFull.into());
                        }
                        if i >= Self::capacity(acc) {
                            grow(acc, Self::size_for(i + 1))?;
                        }
                        head.holder_count += 1;
                        i
                    }
                };
                (i, Holder { holder: *holder, kind, seats, weight: 1, last_active: now })
            }
        };
        if entry.1.seats > crate::MAX_SEATS_PER_HOLDER {
            return Err(PoleisError::SeatLimit.into());
        }
        head.total_seats = head.total_seats.checked_add(seats).ok_or(PoleisError::MathOverflow)?;
        Self::set_holder(acc, entry.0, &entry.1)?;
        Self::set_head(acc, &head)
    }

    /// Empty a holder's row; returns the seats that left the registry. The
    /// row stays where it is, a tombstone: sessions and distributions record
    /// voices and claims by holder index, so no row may ever move under them
    /// (a swap-remove let the last holder inherit a leaver's unclaimed bit
    /// and vote or claim twice). The next holder to arrive takes it.
    pub fn remove_holder(acc: &mut AccountView, i: usize) -> Result<u64, ProgramError> {
        let mut head = Self::head(acc)?;
        let gone = Self::holder(acc, i)?;
        // NFT row identities never recycle: old session/distribution bits
        // refer to this exact asset even after it has burned or been expelled.
        let (holder, kind) = if gone.kind == crate::HOLDER_NFT {
            (gone.holder, crate::HOLDER_RETIRED_NFT)
        } else {
            ([0u8; 32], crate::HOLDER_EMPTY)
        };
        Self::set_holder(acc, i, &Holder { holder, kind, seats: 0, weight: 0, last_active: 0 })?;
        head.total_seats = head.total_seats.checked_sub(gone.seats).ok_or(PoleisError::MathOverflow)?;
        Self::set_head(acc, &head)?;
        Ok(gone.seats)
    }

    /// The first tombstone below the row count, if any.
    pub fn first_empty(acc: &AccountView) -> Result<Option<usize>, ProgramError> {
        let d = Self::check(acc)?;
        let n = d[Self::OFF_HOLDER_COUNT] as usize;
        for i in 0..n {
            if d[Self::holder_off(i) + 32] == crate::HOLDER_EMPTY {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// Weight of every holder heard from within the activity window: the
    /// hexagon seal's denominator. Silence past the window is abstention.
    pub fn active_weight(acc: &AccountView, now: u64) -> Result<u64, ProgramError> {
        let d = Self::check(acc)?;
        let n = d[Self::OFF_HOLDER_COUNT] as usize;
        let mut total: u64 = 0;
        for i in 0..n {
            let o = Self::holder_off(i);
            let last = rd_u64(&d, o + 49);
            if last.saturating_add(crate::ACTIVE_WINDOW_SLOTS) >= now {
                let w = rd_u64(&d, o + 33).checked_mul(rd_u64(&d, o + 41)).ok_or(PoleisError::MathOverflow)?;
                total = total.checked_add(w).ok_or(PoleisError::MathOverflow)?;
            }
        }
        Ok(total)
    }

    /// Weight of every holder, active or not: a dividend's denominator.
    pub fn total_weight(acc: &AccountView) -> Result<u64, ProgramError> {
        let d = Self::check(acc)?;
        let n = d[Self::OFF_HOLDER_COUNT] as usize;
        let mut total: u64 = 0;
        for i in 0..n {
            let o = Self::holder_off(i);
            let w = rd_u64(&d, o + 33).checked_mul(rd_u64(&d, o + 41)).ok_or(PoleisError::MathOverflow)?;
            total = total.checked_add(w).ok_or(PoleisError::MathOverflow)?;
        }
        Ok(total)
    }
}

// ---- SeatSale ----
// A uniform-price sealed sale for N hexagon seats: the top N revealed
// prices win and all pay the (N+1)th price, floored at the reserve.
// Entries live in the account (variable size: HEAD + seat_count × ENTRY).
// [disc:1][ver:1][unit_id:8][sale_id:8][seat_count:8][reserve:8][deposit:8]
// [commit_end:8][reveal_end:8][clearing:8][overflow_price:8]
// [settled:1][filled:8][bump:1][entries: seat_count × (price:8, bidder:32, nonce:8)]
pub struct SeatSale;

pub struct SaleHead {
    pub unit_id: u64,
    pub sale_id: u64,
    pub seat_count: u64,
    pub reserve: u64,
    pub deposit: u64,
    pub commit_end_slot: u64,
    pub reveal_end_slot: u64,
    pub clearing: u64,
    pub overflow_price: u64,
    pub settled: bool,
    pub filled: u64,
    pub bump: u8,
}

impl SeatSale {
    pub const HEAD: usize = 84;
    pub const ENTRY: usize = 48;
    pub const OFF_UNIT: usize = 2;
    pub const OFF_SALE: usize = 10;
    pub const OFF_SEAT_COUNT: usize = 18;
    pub const OFF_RESERVE: usize = 26;
    pub const OFF_DEPOSIT: usize = 34;
    pub const OFF_COMMIT_END: usize = 42;
    pub const OFF_REVEAL_END: usize = 50;
    pub const OFF_CLEARING: usize = 58;
    pub const OFF_OVERFLOW: usize = 66;
    pub const OFF_SETTLED: usize = 74;
    pub const OFF_FILLED: usize = 75;
    pub const OFF_BUMP: usize = 83;

    pub fn size(seat_count: u64) -> usize {
        Self::HEAD + seat_count as usize * Self::ENTRY
    }

    fn checked_sale(acc: &AccountView) -> Result<Ref<'_, [u8]>, ProgramError> {
        if !acc.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        let d = acc.try_borrow()?;
        if d.len() < Self::HEAD || d[OFF_DISC] != DISC_SEAT_SALE {
            return Err(PoleisError::InvalidAccount.into());
        }
        if d[OFF_VERSION] != LAYOUT_VERSION {
            return Err(PoleisError::LayoutVersion.into());
        }
        if d.len() != Self::size(rd_u64(&d, Self::OFF_SEAT_COUNT)) {
            return Err(PoleisError::InvalidAccount.into());
        }
        Ok(d)
    }

    fn checked_sale_mut(acc: &mut AccountView) -> Result<RefMut<'_, [u8]>, ProgramError> {
        if !acc.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        let d = acc.try_borrow_mut()?;
        if d.len() < Self::HEAD || d[OFF_DISC] != DISC_SEAT_SALE {
            return Err(PoleisError::InvalidAccount.into());
        }
        if d[OFF_VERSION] != LAYOUT_VERSION {
            return Err(PoleisError::LayoutVersion.into());
        }
        if d.len() != Self::size(rd_u64(&d, Self::OFF_SEAT_COUNT)) {
            return Err(PoleisError::InvalidAccount.into());
        }
        Ok(d)
    }

    pub fn init(acc: &mut AccountView, h: &SaleHead) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::size(h.seat_count) {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_SEAT_SALE);
        wr_u64(&mut d, Self::OFF_UNIT, h.unit_id);
        wr_u64(&mut d, Self::OFF_SALE, h.sale_id);
        wr_u64(&mut d, Self::OFF_SEAT_COUNT, h.seat_count);
        wr_u64(&mut d, Self::OFF_RESERVE, h.reserve);
        wr_u64(&mut d, Self::OFF_DEPOSIT, h.deposit);
        wr_u64(&mut d, Self::OFF_COMMIT_END, h.commit_end_slot);
        wr_u64(&mut d, Self::OFF_REVEAL_END, h.reveal_end_slot);
        wr_u64(&mut d, Self::OFF_CLEARING, 0);
        wr_u64(&mut d, Self::OFF_OVERFLOW, 0);
        d[Self::OFF_SETTLED] = 0;
        wr_u64(&mut d, Self::OFF_FILLED, 0);
        d[Self::OFF_BUMP] = h.bump;
        Ok(())
    }

    pub fn head(acc: &AccountView) -> Result<SaleHead, ProgramError> {
        let d = Self::checked_sale(acc)?;
        Ok(SaleHead {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            sale_id: rd_u64(&d, Self::OFF_SALE),
            seat_count: rd_u64(&d, Self::OFF_SEAT_COUNT),
            reserve: rd_u64(&d, Self::OFF_RESERVE),
            deposit: rd_u64(&d, Self::OFF_DEPOSIT),
            commit_end_slot: rd_u64(&d, Self::OFF_COMMIT_END),
            reveal_end_slot: rd_u64(&d, Self::OFF_REVEAL_END),
            clearing: rd_u64(&d, Self::OFF_CLEARING),
            overflow_price: rd_u64(&d, Self::OFF_OVERFLOW),
            settled: d[Self::OFF_SETTLED] != 0,
            filled: rd_u64(&d, Self::OFF_FILLED),
            bump: d[Self::OFF_BUMP],
        })
    }

    pub fn set_settled(acc: &mut AccountView, clearing: u64) -> Result<(), ProgramError> {
        let mut d = Self::checked_sale_mut(acc)?;
        wr_u64(&mut d, Self::OFF_CLEARING, clearing);
        d[Self::OFF_SETTLED] = 1;
        Ok(())
    }

    /// Keep the top N prices. A full board evicts its minimum only for a
    /// strictly higher price (earlier reveals win ties); every price that
    /// does not make the board raises the overflow, the uniform price.
    pub fn insert_reveal(acc: &mut AccountView, price: u64, bidder: &[u8; 32], nonce: u64) -> Result<(), ProgramError> {
        let mut d = Self::checked_sale_mut(acc)?;
        let n = rd_u64(&d, Self::OFF_SEAT_COUNT) as usize;
        let filled = rd_u64(&d, Self::OFF_FILLED) as usize;
        let slot = if filled < n {
            wr_u64(&mut d, Self::OFF_FILLED, filled as u64 + 1);
            filled
        } else {
            let mut min_i = 0usize;
            let mut min_p = u64::MAX;
            for i in 0..n {
                let p = rd_u64(&d, Self::HEAD + i * Self::ENTRY);
                if p < min_p {
                    min_p = p;
                    min_i = i;
                }
            }
            let overflow = rd_u64(&d, Self::OFF_OVERFLOW);
            if price > min_p {
                wr_u64(&mut d, Self::OFF_OVERFLOW, overflow.max(min_p));
                min_i
            } else {
                wr_u64(&mut d, Self::OFF_OVERFLOW, overflow.max(price));
                return Ok(());
            }
        };
        let o = Self::HEAD + slot * Self::ENTRY;
        wr_u64(&mut d, o, price);
        wr_b32(&mut d, o + 8, bidder);
        wr_u64(&mut d, o + 40, nonce);
        Ok(())
    }

    pub fn is_winner(acc: &AccountView, bidder: &[u8; 32], nonce: u64) -> Result<bool, ProgramError> {
        let d = Self::checked_sale(acc)?;
        let filled = rd_u64(&d, Self::OFF_FILLED) as usize;
        for i in 0..filled {
            let o = Self::HEAD + i * Self::ENTRY;
            if &d[o + 8..o + 40] == bidder && rd_u64(&d, o + 40) == nonce {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

// ---- SeatBid ----
// [disc:1][ver:1][unit_id:8][sale_id:8][bidder:32][nonce:8][commitment:32]
// [deposit:8][price:8][revealed:1][done:1][bump:1]
pub struct SeatBid {
    pub unit_id: u64,
    pub sale_id: u64,
    pub bidder: Address,
    pub nonce: u64,
    pub commitment: [u8; 32],
    pub deposit: u64,
    pub price: u64,
    pub revealed: bool,
    pub done: bool,
    pub bump: u8,
}
impl SeatBid {
    pub const SIZE: usize = 109;
    pub const OFF_UNIT: usize = 2;
    pub const OFF_SALE: usize = 10;
    pub const OFF_BIDDER: usize = 18;
    pub const OFF_NONCE: usize = 50;
    pub const OFF_COMMITMENT: usize = 58;
    pub const OFF_DEPOSIT: usize = 90;
    pub const OFF_PRICE: usize = 98;
    pub const OFF_REVEALED: usize = 106;
    pub const OFF_DONE: usize = 107;
    pub const OFF_BUMP: usize = 108;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_SEAT_BID, Self::SIZE)?;
        Ok(Self {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            sale_id: rd_u64(&d, Self::OFF_SALE),
            bidder: rd_addr(&d, Self::OFF_BIDDER),
            nonce: rd_u64(&d, Self::OFF_NONCE),
            commitment: rd_h32(&d, Self::OFF_COMMITMENT),
            deposit: rd_u64(&d, Self::OFF_DEPOSIT),
            price: rd_u64(&d, Self::OFF_PRICE),
            revealed: d[Self::OFF_REVEALED] != 0,
            done: d[Self::OFF_DONE] != 0,
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_SEAT_BID);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_u64(&mut d, Self::OFF_SALE, self.sale_id);
        wr_addr(&mut d, Self::OFF_BIDDER, &self.bidder);
        wr_u64(&mut d, Self::OFF_NONCE, self.nonce);
        wr_b32(&mut d, Self::OFF_COMMITMENT, &self.commitment);
        wr_u64(&mut d, Self::OFF_DEPOSIT, self.deposit);
        wr_u64(&mut d, Self::OFF_PRICE, self.price);
        d[Self::OFF_REVEALED] = self.revealed as u8;
        d[Self::OFF_DONE] = self.done as u8;
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ---- Proposal ----
// The fingerprint of a decision, what it touches, and the payload in clear,
// so a voter reads what will execute and anyone can execute it. Executed at
// most once. Closed to a tombstone (disc, version, hash) once decided, so
// the hash can never be proposed again while its rent goes home.
// [disc:1][ver:1][hash:32][class:1][origin:8][manifest:32][payload_hash:32]
// [opened:8][expires:8][executed:1][count:1][bump:1][impacted: MAX_IMPACTED × 8]
// [payer:32][payload_len:2][payload: MAX_PAYLOAD]
pub struct Proposal {
    pub hash: [u8; 32],
    pub class: u8,
    pub origin: u64,
    pub manifest: [u8; 32],
    pub payload_hash: [u8; 32],
    pub opened: u64,
    pub expires: u64,
    pub executed: bool,
    pub count: u8,
    pub bump: u8,
    pub impacted: [u64; crate::MAX_IMPACTED],
    pub payer: [u8; 32],
    pub payload_len: u16,
}
impl Proposal {
    pub const OFF_HASH: usize = 2;
    pub const OFF_CLASS: usize = 34;
    pub const OFF_ORIGIN: usize = 35;
    pub const OFF_MANIFEST: usize = 43;
    pub const OFF_PAYLOAD_HASH: usize = 75;
    pub const OFF_OPENED: usize = 107;
    pub const OFF_EXPIRES: usize = 115;
    pub const OFF_EXECUTED: usize = 123;
    pub const OFF_COUNT: usize = 124;
    pub const OFF_BUMP: usize = 125;
    pub const OFF_IMPACTED: usize = 126;
    pub const OFF_PAYER: usize = Self::OFF_IMPACTED + crate::MAX_IMPACTED * 8;
    pub const OFF_PAYLOAD_LEN: usize = Self::OFF_PAYER + 32;
    pub const OFF_PAYLOAD: usize = Self::OFF_PAYLOAD_LEN + 2;
    pub const SIZE: usize = Self::OFF_PAYLOAD + crate::MAX_PAYLOAD;
    /// What a closed proposal shrinks to: the hash stays, so the address
    /// stays taken.
    pub const TOMBSTONE_SIZE: usize = 34;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_PROPOSAL, Self::SIZE)?;
        let mut impacted = [0u64; crate::MAX_IMPACTED];
        for (i, slot) in impacted.iter_mut().enumerate() {
            *slot = rd_u64(&d, Self::OFF_IMPACTED + i * 8);
        }
        Ok(Self {
            hash: rd_h32(&d, Self::OFF_HASH),
            class: d[Self::OFF_CLASS],
            origin: rd_u64(&d, Self::OFF_ORIGIN),
            manifest: rd_h32(&d, Self::OFF_MANIFEST),
            payload_hash: rd_h32(&d, Self::OFF_PAYLOAD_HASH),
            opened: rd_u64(&d, Self::OFF_OPENED),
            expires: rd_u64(&d, Self::OFF_EXPIRES),
            executed: d[Self::OFF_EXECUTED] != 0,
            count: d[Self::OFF_COUNT],
            bump: d[Self::OFF_BUMP],
            impacted,
            payer: rd_h32(&d, Self::OFF_PAYER),
            payload_len: rd_u16(&d, Self::OFF_PAYLOAD_LEN),
        })
    }
    /// The head fields; the payload is written separately, in place.
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_PROPOSAL);
        wr_b32(&mut d, Self::OFF_HASH, &self.hash);
        d[Self::OFF_CLASS] = self.class;
        wr_u64(&mut d, Self::OFF_ORIGIN, self.origin);
        wr_b32(&mut d, Self::OFF_MANIFEST, &self.manifest);
        wr_b32(&mut d, Self::OFF_PAYLOAD_HASH, &self.payload_hash);
        wr_u64(&mut d, Self::OFF_OPENED, self.opened);
        wr_u64(&mut d, Self::OFF_EXPIRES, self.expires);
        d[Self::OFF_EXECUTED] = self.executed as u8;
        d[Self::OFF_COUNT] = self.count;
        d[Self::OFF_BUMP] = self.bump;
        for (i, u) in self.impacted.iter().enumerate() {
            wr_u64(&mut d, Self::OFF_IMPACTED + i * 8, *u);
        }
        wr_b32(&mut d, Self::OFF_PAYER, &self.payer);
        wr_u16(&mut d, Self::OFF_PAYLOAD_LEN, self.payload_len);
        Ok(())
    }
    pub fn write_payload(acc: &mut AccountView, payload: &[u8]) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        d[Self::OFF_PAYLOAD..Self::OFF_PAYLOAD + payload.len()].copy_from_slice(payload);
        Ok(())
    }
    /// The payload's kind, its second byte, read in place.
    pub fn payload_kind(acc: &AccountView) -> Result<u8, ProgramError> {
        let d = checked(acc, DISC_PROPOSAL, Self::SIZE)?;
        Ok(d[Self::OFF_PAYLOAD + 1])
    }
    /// Copy the payload out; the caller owns the buffer.
    pub fn payload(acc: &AccountView, len: usize, out: &mut [u8; crate::MAX_PAYLOAD]) -> Result<(), ProgramError> {
        let d = checked(acc, DISC_PROPOSAL, Self::SIZE)?;
        out[..len].copy_from_slice(&d[Self::OFF_PAYLOAD..Self::OFF_PAYLOAD + len]);
        Ok(())
    }
    pub fn touches(&self, unit_id: u64) -> bool {
        self.impacted[..self.count as usize].contains(&unit_id)
    }
    /// The class without its bits.
    pub fn class_of(&self) -> u8 {
        crate::class_of(self.class)
    }
    /// The proposer asked that every chamber seal PASSED before execution.
    pub fn consent(&self) -> bool {
        self.class & crate::CLASS_CONSENT != 0
    }
    /// The proposer asked that a split of the origin's chambers be settled by the SUM.
    pub fn sum(&self) -> bool {
        self.class & crate::CLASS_SUM != 0
    }
    /// The proposer asked that a split of the origin's chambers, the chairs for, go to the market.
    pub fn market(&self) -> bool {
        self.class & crate::CLASS_MARKET != 0
    }
    /// The proposer asked that the origin's floor speak by price: a plan.
    pub fn plan(&self) -> bool {
        self.class & crate::CLASS_PLAN != 0
    }
}

// ---- Session ----
// One per (proposal, impacted unit): the two chambers' tallies, every voice
// by chair and by holder index (two bits each: 0 silent, 1 ABSTENTIO,
// 2 PRO, 3 CONTRA), and the seal. The census paints faces from the voices,
// never from the counts.
// [disc:1][ver:1][hash:32][unit:8][deadline:8][hex_pro:8][hex_contra:8][hex_abst:8]
// [pent_pro:1][pent_contra:1][pent_abst:1][chair_mask:2][state:1][bump:1]
// [counted: 24 bytes, one bit per holder index][votes:4][active_weight:8]
// [chair_voice: 3 bytes][holder_voice: 38 bytes]
// then the roster, written by every session opened since THE ROSTER IS FIXED
// WHEN A VOTE OPENS: who is counted is who stood there when the vote opened.
// ["ROSTER02":8][count:1][lit:2][crews:2][opened:8]
// [count × (identity:32, kind:1, weight:8, admitted:1)]
// `lit` is the chairs lit at the opening, `crews` those of them that seat a
// unit, `opened` the opening's slot; each row is a holder of the registry at
// the opening with its seats × weight then. Votes, the chairs' bar, the wait
// for crews and the consent AND are all judged against it. Two older shapes
// stay readable, for sessions opened before: the bare 158-byte prefix (the
// registry judged at the moment) and "NFTVOTE1" (header 9, the same rows,
// no chairs). Any of the three may end with the convener's 40-byte tail.
pub struct Session {
    pub hash: [u8; 32],
    pub unit_id: u64,
    pub deadline: u64,
    pub hex_pro: u64,
    pub hex_contra: u64,
    pub hex_abst: u64,
    pub pent_pro: u8,
    pub pent_contra: u8,
    pub pent_abst: u8,
    pub chair_mask: u16,
    pub state: u8,
    pub bump: u8,
    pub counted: [u8; 24],
    pub votes: u32,
    pub active_weight: u64,
    pub chair_voice: [u8; Self::CHAIR_VOICE_BYTES],
    pub holder_voice: [u8; Self::HOLDER_VOICE_BYTES],
}
impl Session {
    pub const OFF_HASH: usize = 2;
    pub const OFF_UNIT: usize = 34;
    pub const OFF_DEADLINE: usize = 42;
    pub const OFF_HEX_PRO: usize = 50;
    pub const OFF_HEX_CONTRA: usize = 58;
    pub const OFF_HEX_ABST: usize = 66;
    pub const OFF_PENT_PRO: usize = 74;
    pub const OFF_PENT_CONTRA: usize = 75;
    pub const OFF_PENT_ABST: usize = 76;
    pub const OFF_CHAIR_MASK: usize = 77;
    pub const OFF_STATE: usize = 79;
    pub const OFF_BUMP: usize = 80;
    pub const OFF_COUNTED: usize = 81;
    pub const OFF_VOTES: usize = 105;
    pub const OFF_ACTIVE_WEIGHT: usize = 109;
    pub const OFF_CHAIR_VOICE: usize = 117;
    pub const CHAIR_VOICE_BYTES: usize = (crate::PENTAGONS * 2).div_ceil(8);
    pub const OFF_HOLDER_VOICE: usize = Self::OFF_CHAIR_VOICE + Self::CHAIR_VOICE_BYTES;
    pub const HOLDER_VOICE_BYTES: usize = (crate::MAX_HOLDERS * 2).div_ceil(8);
    pub const SIZE: usize = Self::OFF_HOLDER_VOICE + Self::HOLDER_VOICE_BYTES;
    /// The older electorate body, written by NFT societies' sessions before
    /// the roster: read as it was, never written again. Keep the legacy
    /// 158-byte prefix intact; the bounded extension lives in account data,
    /// never in this stack struct. At most 9 + 42*150 = 6309 extra rent bytes.
    pub const ELECTORATE_MAGIC: [u8; 8] = *b"NFTVOTE1";
    pub const ELECTORATE_HEADER: usize = 9;
    pub const ELECTORATE_ROW: usize = 42;
    /// The roster every session opened since carries: the electorate's rows
    /// and the chairs as they stood at the opening. Its header is the magic,
    /// the count, the chairs lit (u16), the crews among them (u16) and the
    /// opening's slot (u64): at most 21 + 42*150 = 6321 extra rent bytes.
    pub const ROSTER_MAGIC: [u8; 8] = *b"ROSTER02";
    pub const ROSTER_HEADER: usize = 21;
    /// A session a polis convened on its own (`convene`) ends with whoever
    /// paid its rent, so the rent goes home to them (`adjourn`). A session
    /// `propose` opened has no tail: its rent goes home with the proposal's.
    pub const CONVENER_MAGIC: [u8; 8] = *b"CONVENE1";
    pub const CONVENER_TAIL: usize = 40;

    pub fn valid_data_len(data: &[u8]) -> bool {
        Self::valid_body(&data[..Self::body_len(data)])
    }

    /// The length without the convener's tail: the prefix and its body (the
    /// roster, or an older session's electorate). Every body states its count
    /// in its header, so its length is exact, and 40 bytes are never the
    /// difference between two lengths of one shape (rows of 42) nor between
    /// a body and the bare prefix (21 or 9 plus rows of 42): no length is
    /// both a tail's and a body's, so the tail is read only where it fits whole.
    pub fn body_len(data: &[u8]) -> usize {
        let n = data.len();
        if n >= Self::SIZE + Self::CONVENER_TAIL
            && data[n - Self::CONVENER_TAIL..n - 32] == Self::CONVENER_MAGIC
            && Self::valid_body(&data[..n - Self::CONVENER_TAIL])
        {
            n - Self::CONVENER_TAIL
        } else {
            n
        }
    }

    /// Whoever convened this session, when a polis convened it.
    pub fn convener(data: &[u8]) -> Option<[u8; 32]> {
        (Self::body_len(data) != data.len()).then(|| data[data.len() - 32..].try_into().unwrap())
    }

    /// Sign a convened session with whoever paid its rent: its last 40 bytes.
    pub fn write_convener(acc: &mut AccountView, convener: &[u8; 32]) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        let n = d.len();
        d[n - Self::CONVENER_TAIL..n - 32].copy_from_slice(&Self::CONVENER_MAGIC);
        d[n - 32..].copy_from_slice(convener);
        Ok(())
    }

    /// The header's length a body names by its magic: the roster's, the
    /// older electorate's, or none for an unknown or truncated body.
    fn header_len(data: &[u8]) -> Option<usize> {
        if data.len() < Self::SIZE + 8 {
            return None;
        }
        let magic = &data[Self::SIZE..Self::SIZE + 8];
        if magic == Self::ROSTER_MAGIC {
            Some(Self::ROSTER_HEADER)
        } else if magic == Self::ELECTORATE_MAGIC {
            Some(Self::ELECTORATE_HEADER)
        } else {
            None
        }
    }

    /// The bare prefix, or a prefix and a whole body of either shape: the
    /// count explicit and within the cap, the rows exactly that many, and a
    /// roster's chairs within the twelve, its crews among its lit chairs.
    fn valid_body(data: &[u8]) -> bool {
        if data.len() == Self::SIZE {
            return true;
        }
        let Some(header) = Self::header_len(data) else {
            return false;
        };
        if data.len() < Self::SIZE + header {
            return false;
        }
        let count = data[Self::SIZE + 8] as usize;
        if count > crate::MAX_HOLDERS || data.len() != Self::SIZE + header + count * Self::ELECTORATE_ROW {
            return false;
        }
        if header == Self::ROSTER_HEADER {
            let lit = rd_u16(data, Self::SIZE + 9);
            let crews = rd_u16(data, Self::SIZE + 11);
            if lit >> crate::PENTAGONS != 0 || crews & !lit != 0 {
                return false;
            }
        }
        true
    }

    /// A new session's size: the prefix and the roster of every holder row
    /// the registry has, whatever the registry holds (keys, units, NFTs).
    pub fn size_for_registry(registry: &AccountView) -> Result<usize, ProgramError> {
        let count = Registry::head(registry)?.holder_count as usize;
        Ok(Self::SIZE + Self::ROSTER_HEADER + count * Self::ELECTORATE_ROW)
    }

    /// Called once on fresh session data, before writing the session prefix:
    /// writes the roster. The chairs lit at `now` and the crews among them,
    /// and `now` itself, the opening's slot; then every holder row, original
    /// identities and weights, including dormant originals. Only original
    /// active rows enter the denominator immediately; a dormant original
    /// joins once when it casts a vote. Later admissions cannot join. `tail`
    /// is the convener's tail the account keeps at its end (zero from
    /// `propose`). A registry with no holder row writes a roster of none.
    pub fn init_electorate(
        session: &mut AccountView,
        registry: &AccountView,
        now: u64,
        tail: usize,
    ) -> Result<u64, ProgramError> {
        let lit = Registry::lit_mask(registry, now)?;
        let crews = Registry::crew_chairs(registry, now)?;
        let source = Registry::check(registry)?;
        let count = source[Registry::OFF_HOLDER_COUNT] as usize;
        let mut data = session.try_borrow_mut()?;
        if count > crate::MAX_HOLDERS
            || data.len() != Self::SIZE + Self::ROSTER_HEADER + count * Self::ELECTORATE_ROW + tail
        {
            return Err(PoleisError::InvalidAccount.into());
        }
        data[Self::SIZE..Self::SIZE + 8].copy_from_slice(&Self::ROSTER_MAGIC);
        data[Self::SIZE + 8] = count as u8;
        wr_u16(&mut data, Self::SIZE + 9, lit);
        wr_u16(&mut data, Self::SIZE + 11, crews);
        wr_u64(&mut data, Self::SIZE + 13, now);
        let mut active = 0u64;
        // Check both aggregate bounds at opening, even if every row is dormant.
        let mut total = 0u64;
        for i in 0..count {
            let original = Registry::holder_off(i);
            let row = Self::SIZE + Self::ROSTER_HEADER + i * Self::ELECTORATE_ROW;
            data[row..row + 33].copy_from_slice(&source[original..original + 33]);
            let weight = rd_u64(&source, original + 33)
                .checked_mul(rd_u64(&source, original + 41))
                .ok_or(PoleisError::MathOverflow)?;
            let eligible_kind = matches!(
                source[original + 32],
                crate::HOLDER_KEY | crate::HOLDER_UNIT | crate::HOLDER_NFT
            );
            if !eligible_kind && weight != 0 {
                return Err(PoleisError::InvalidHolder.into());
            }
            wr_u64(&mut data, row + 33, weight);
            let admitted = weight > 0
                && rd_u64(&source, original + 49).saturating_add(crate::ACTIVE_WINDOW_SLOTS) >= now;
            data[row + 41] = u8::from(admitted);
            total = total.checked_add(weight).ok_or(PoleisError::MathOverflow)?;
            if admitted {
                active = active
                    .checked_add(weight)
                    .ok_or(PoleisError::MathOverflow)?;
            }
        }
        Ok(active)
    }

    /// Original total and the union of original-active plus awakened originals.
    /// Burn, resale, changed weights, retirement and delayed sealing never
    /// remove a member from this proposal's electorate or alter its weight.
    pub fn electorate_weights(data: &[u8]) -> Result<Option<(u64, u64)>, ProgramError> {
        let body = Self::body_len(data);
        if !Self::valid_body(&data[..body]) {
            return Err(PoleisError::InvalidAccount.into());
        }
        if body == Self::SIZE {
            return Ok(None);
        }
        let rows = Self::SIZE + Self::header_len(data).ok_or(PoleisError::InvalidAccount)?;
        let mut total = 0u64;
        let mut active = 0u64;
        for i in 0..data[Self::SIZE + 8] as usize {
            let row = rows + i * Self::ELECTORATE_ROW;
            let weight = rd_u64(data, row + 33);
            let admitted = data[row + 41];
            if admitted > 1 || (admitted == 1 && weight == 0) {
                return Err(PoleisError::InvalidAccount.into());
            }
            total = total.checked_add(weight).ok_or(PoleisError::MathOverflow)?;
            if admitted == 1 {
                active = active
                    .checked_add(weight)
                    .ok_or(PoleisError::MathOverflow)?;
            }
        }
        Ok(Some((total, active)))
    }

    /// The chairs as a session's opening found them: `(lit, crews, opened)`
    /// for a session that carries the roster, None for one opened before it
    /// (the bare prefix or "NFTVOTE1"), an error for a malformed body. The
    /// convener's tail is set aside first, as `electorate_weights` does.
    pub fn roster(data: &[u8]) -> Result<Option<(u16, u16, u64)>, ProgramError> {
        let body = Self::body_len(data);
        if !Self::valid_body(&data[..body]) {
            return Err(PoleisError::InvalidAccount.into());
        }
        if body == Self::SIZE || data[Self::SIZE..Self::SIZE + 8] != Self::ROSTER_MAGIC {
            return Ok(None);
        }
        Ok(Some((
            rd_u16(data, Self::SIZE + 9),
            rd_u16(data, Self::SIZE + 11),
            rd_u64(data, Self::SIZE + 13),
        )))
    }

    /// Called only for a not-yet-counted holder after runtime authorization.
    /// None excludes a later admission, a reused row or an originally empty
    /// identity. A still-owned original seat votes at its original weight.
    pub fn admit_vote(
        &mut self,
        session: &mut AccountView,
        registry: &AccountView,
        index: usize,
    ) -> Result<Option<u64>, ProgramError> {
        let holder = Registry::holder(registry, index)?;
        // a bare session, opened before the roster, counts the registry at the moment: its length says it all
        let body = if session.data_len() == Self::SIZE { Self::SIZE } else { Self::body_len(&session.try_borrow()?) };
        if body == Self::SIZE {
            return Ok(Some(
                holder
                    .seats
                    .checked_mul(holder.weight)
                    .ok_or(PoleisError::MathOverflow)?,
            ));
        }
        let mut data = session.try_borrow_mut()?;
        if !Self::valid_body(&data[..body]) {
            return Err(PoleisError::InvalidAccount.into());
        }
        if index >= data[Self::SIZE + 8] as usize {
            return Ok(None);
        }
        let header = Self::header_len(&data[..body]).ok_or(PoleisError::InvalidAccount)?;
        let row = Self::SIZE + header + index * Self::ELECTORATE_ROW;
        let weight = rd_u64(&data, row + 33);
        if holder.seats == 0
            || weight == 0
            || data[row..row + 32] != holder.holder
            || data[row + 32] != holder.kind
        {
            return Ok(None);
        }
        match data[row + 41] {
            0 => {
                self.active_weight = self
                    .active_weight
                    .checked_add(weight)
                    .ok_or(PoleisError::MathOverflow)?;
                data[row + 41] = 1;
            }
            1 => {}
            _ => return Err(PoleisError::InvalidAccount.into()),
        }
        Ok(Some(weight))
    }
    /// A voice as stored: silent, then the vote code plus one.
    pub const VOICE_SILENT: u8 = 0;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        if acc.data_len() < Self::SIZE { return Err(PoleisError::InvalidAccount.into()); }
        let d = checked(acc, DISC_SESSION, acc.data_len())?;
        if let Some((_, active)) = Self::electorate_weights(&d)? {
            if rd_u64(&d, Self::OFF_ACTIVE_WEIGHT) != active {
                return Err(PoleisError::InvalidAccount.into());
            }
        }
        Ok(Self {
            hash: rd_h32(&d, Self::OFF_HASH),
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            deadline: rd_u64(&d, Self::OFF_DEADLINE),
            hex_pro: rd_u64(&d, Self::OFF_HEX_PRO),
            hex_contra: rd_u64(&d, Self::OFF_HEX_CONTRA),
            hex_abst: rd_u64(&d, Self::OFF_HEX_ABST),
            pent_pro: d[Self::OFF_PENT_PRO],
            pent_contra: d[Self::OFF_PENT_CONTRA],
            pent_abst: d[Self::OFF_PENT_ABST],
            chair_mask: rd_u16(&d, Self::OFF_CHAIR_MASK),
            state: d[Self::OFF_STATE],
            bump: d[Self::OFF_BUMP],
            counted: d[Self::OFF_COUNTED..Self::OFF_COUNTED + 24].try_into().unwrap(),
            votes: rd_u32(&d, Self::OFF_VOTES),
            active_weight: rd_u64(&d, Self::OFF_ACTIVE_WEIGHT),
            chair_voice: d[Self::OFF_CHAIR_VOICE..Self::OFF_CHAIR_VOICE + Self::CHAIR_VOICE_BYTES].try_into().unwrap(),
            holder_voice: d[Self::OFF_HOLDER_VOICE..Self::OFF_HOLDER_VOICE + Self::HOLDER_VOICE_BYTES].try_into().unwrap(),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_SESSION);
        wr_b32(&mut d, Self::OFF_HASH, &self.hash);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_u64(&mut d, Self::OFF_DEADLINE, self.deadline);
        wr_u64(&mut d, Self::OFF_HEX_PRO, self.hex_pro);
        wr_u64(&mut d, Self::OFF_HEX_CONTRA, self.hex_contra);
        wr_u64(&mut d, Self::OFF_HEX_ABST, self.hex_abst);
        d[Self::OFF_PENT_PRO] = self.pent_pro;
        d[Self::OFF_PENT_CONTRA] = self.pent_contra;
        d[Self::OFF_PENT_ABST] = self.pent_abst;
        wr_u16(&mut d, Self::OFF_CHAIR_MASK, self.chair_mask);
        d[Self::OFF_STATE] = self.state;
        d[Self::OFF_BUMP] = self.bump;
        d[Self::OFF_COUNTED..Self::OFF_COUNTED + 24].copy_from_slice(&self.counted);
        wr_u32(&mut d, Self::OFF_VOTES, self.votes);
        wr_u64(&mut d, Self::OFF_ACTIVE_WEIGHT, self.active_weight);
        d[Self::OFF_CHAIR_VOICE..Self::OFF_CHAIR_VOICE + Self::CHAIR_VOICE_BYTES].copy_from_slice(&self.chair_voice);
        d[Self::OFF_HOLDER_VOICE..Self::OFF_HOLDER_VOICE + Self::HOLDER_VOICE_BYTES].copy_from_slice(&self.holder_voice);
        Ok(())
    }
    pub fn counted(&self, i: usize) -> bool {
        self.counted[i / 8] & (1 << (i % 8)) != 0
    }
    pub fn mark(&mut self, i: usize) {
        self.counted[i / 8] |= 1 << (i % 8);
    }
    fn set_voice(bits: &mut [u8], i: usize, choice: u8) {
        let shift = (i % 4) * 2;
        bits[i / 4] = (bits[i / 4] & !(0b11 << shift)) | ((choice + 1) << shift);
    }
    /// Chair `k`'s voice as stored: silent, or the vote code plus one.
    pub fn chair_voice_of(&self, k: usize) -> u8 {
        (self.chair_voice[k / 4] >> ((k % 4) * 2)) & 0b11
    }
    /// Record chair `k`'s voice.
    pub fn voice_chair(&mut self, k: usize, choice: u8) {
        Self::set_voice(&mut self.chair_voice, k, choice);
    }
    /// Record holder `i`'s voice.
    pub fn voice_holder(&mut self, i: usize, choice: u8) {
        Self::set_voice(&mut self.holder_voice, i, choice);
    }
    pub fn terminal(&self) -> bool {
        self.state != crate::STATE_VOTING
    }
}

// ---- Distribution ----
// A dividend escrow: an amount over the registry's total weight at execute,
// claimed pro-rata by each holder, once, the claim recorded as one bit per
// holder index (no receipt accounts, no rent per claim). Who may claim what
// is fixed at the cut: the roster after the prefix carries each registry
// row's identity and weight (seats × weight) as they stood at the cut, and
// a claim is paid by that row to that identity, whatever moved since (a
// seat bought, sold or moved to a fresh key after the cut claims nothing
// new; a holder who left after the cut still claims). Dissolution is one
// of these over the whole free balance, with the unit frozen. A lamport
// distribution holds its amount itself; a token distribution (mint set)
// holds it in an escrow token account beside it. Closable after its TTL:
// dust goes home to the treasury, rent to whoever paid it. A distribution
// cut before the roster existed is the prefix alone (exactly SIZE bytes)
// and is claimed against the registry at the claim, as it always was.
// [disc:1][ver:1][hash:32][unit:8][amount:8][total_weight:8][paid:8][claims:4][bump:1]
// [mint:32][payer:32][created:8][claimed: CLAIM_BYTES]
// [roster: magic "DIVROST1":8][count:1][count × (identity:32, kind:1, weight:8)]
pub struct Distribution {
    pub hash: [u8; 32],
    pub unit_id: u64,
    pub amount: u64,
    pub total_weight: u64,
    pub paid: u64,
    pub claims: u32,
    pub bump: u8,
    pub mint: [u8; 32],
    pub payer: [u8; 32],
    pub created: u64,
    pub claimed: [u8; Self::CLAIM_BYTES],
}
impl Distribution {
    pub const OFF_HASH: usize = 2;
    pub const OFF_UNIT: usize = 34;
    pub const OFF_AMOUNT: usize = 42;
    pub const OFF_TOTAL_WEIGHT: usize = 50;
    pub const OFF_PAID: usize = 58;
    pub const OFF_CLAIMS: usize = 66;
    pub const OFF_BUMP: usize = 70;
    pub const OFF_MINT: usize = 71;
    pub const OFF_PAYER: usize = 103;
    pub const OFF_CREATED: usize = 135;
    pub const OFF_CLAIMED: usize = 143;
    pub const CLAIM_BYTES: usize = crate::MAX_HOLDERS.div_ceil(8);
    pub const SIZE: usize = Self::OFF_CLAIMED + Self::CLAIM_BYTES;
    /// The roster after the prefix: magic, count, then one row per registry
    /// row at the cut, in the registry's order (the claim bit's index).
    pub const ROSTER_MAGIC: [u8; 8] = *b"DIVROST1";
    pub const ROSTER_HEADER: usize = 9;
    /// identity:32, kind:1, weight:8 (the row's seats × weight at the cut).
    pub const ROSTER_ROW: usize = 41;

    /// The prefix alone (a distribution cut before the roster), or the
    /// prefix and a well-formed roster.
    pub fn valid_data_len(data: &[u8]) -> bool {
        if data.len() == Self::SIZE {
            return true;
        }
        if data.len() < Self::SIZE + Self::ROSTER_HEADER || data[Self::SIZE..Self::SIZE + 8] != Self::ROSTER_MAGIC {
            return false;
        }
        let count = data[Self::SIZE + 8] as usize;
        count <= crate::MAX_HOLDERS && data.len() == Self::SIZE + Self::ROSTER_HEADER + count * Self::ROSTER_ROW
    }

    /// The size of a distribution cut now: the prefix and one roster row per
    /// registry row.
    pub fn size_for_registry(registry: &AccountView) -> Result<usize, ProgramError> {
        let count = Registry::head(registry)?.holder_count as usize;
        Ok(Self::SIZE + Self::ROSTER_HEADER + count * Self::ROSTER_ROW)
    }

    /// Called once on the fresh account at the cut: copies every registry
    /// row's identity and kind, and its seats × weight, into the roster.
    /// Returns the sum of the weights: the distribution's total weight.
    pub fn init_roster(distribution: &mut AccountView, registry: &AccountView) -> Result<u64, ProgramError> {
        let source = Registry::check(registry)?;
        let count = source[Registry::OFF_HOLDER_COUNT] as usize;
        let mut data = distribution.try_borrow_mut()?;
        if count > crate::MAX_HOLDERS || data.len() != Self::SIZE + Self::ROSTER_HEADER + count * Self::ROSTER_ROW {
            return Err(PoleisError::InvalidAccount.into());
        }
        data[Self::SIZE..Self::SIZE + 8].copy_from_slice(&Self::ROSTER_MAGIC);
        data[Self::SIZE + 8] = count as u8;
        let mut total = 0u64;
        for i in 0..count {
            let original = Registry::holder_off(i);
            let row = Self::SIZE + Self::ROSTER_HEADER + i * Self::ROSTER_ROW;
            data[row..row + 33].copy_from_slice(&source[original..original + 33]);
            let weight = rd_u64(&source, original + 33)
                .checked_mul(rd_u64(&source, original + 41))
                .ok_or(PoleisError::MathOverflow)?;
            // the same rule as a session's electorate: only a key, a unit or
            // a live NFT seat carries weight; an emptied or retired row none
            let eligible_kind = matches!(source[original + 32], crate::HOLDER_KEY | crate::HOLDER_UNIT | crate::HOLDER_NFT);
            if !eligible_kind && weight != 0 {
                return Err(PoleisError::InvalidHolder.into());
            }
            wr_u64(&mut data, row + 33, weight);
            total = total.checked_add(weight).ok_or(PoleisError::MathOverflow)?;
        }
        Ok(total)
    }

    /// Row `i` of the roster: (identity, kind, weight at the cut). None for a
    /// distribution with no roster, or past the roster's count.
    pub fn roster_row(data: &[u8], i: usize) -> Result<Option<([u8; 32], u8, u64)>, ProgramError> {
        if data.len() == Self::SIZE {
            return Ok(None);
        }
        if !Self::valid_data_len(data) {
            return Err(PoleisError::InvalidAccount.into());
        }
        if i >= data[Self::SIZE + 8] as usize {
            return Ok(None);
        }
        let row = Self::SIZE + Self::ROSTER_HEADER + i * Self::ROSTER_ROW;
        Ok(Some((rd_h32(data, row), data[row + 32], rd_u64(data, row + 33))))
    }

    /// The row index and weight at the cut of (kind, identity) in the roster.
    /// None for a distribution with no roster, or an identity it does not hold.
    pub fn roster_find(data: &[u8], kind: u8, identity: &[u8; 32]) -> Result<Option<(usize, u64)>, ProgramError> {
        if data.len() == Self::SIZE {
            return Ok(None);
        }
        if !Self::valid_data_len(data) {
            return Err(PoleisError::InvalidAccount.into());
        }
        for i in 0..data[Self::SIZE + 8] as usize {
            let row = Self::SIZE + Self::ROSTER_HEADER + i * Self::ROSTER_ROW;
            if data[row + 32] == kind && &data[row..row + 32] == identity {
                return Ok(Some((i, rd_u64(data, row + 33))));
            }
        }
        Ok(None)
    }

    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        if acc.data_len() < Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        let d = checked(acc, DISC_DISTRIBUTION, acc.data_len())?;
        if !Self::valid_data_len(&d) {
            return Err(PoleisError::InvalidAccount.into());
        }
        Ok(Self {
            hash: rd_h32(&d, Self::OFF_HASH),
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            amount: rd_u64(&d, Self::OFF_AMOUNT),
            total_weight: rd_u64(&d, Self::OFF_TOTAL_WEIGHT),
            paid: rd_u64(&d, Self::OFF_PAID),
            claims: rd_u32(&d, Self::OFF_CLAIMS),
            bump: d[Self::OFF_BUMP],
            mint: rd_h32(&d, Self::OFF_MINT),
            payer: rd_h32(&d, Self::OFF_PAYER),
            created: rd_u64(&d, Self::OFF_CREATED),
            claimed: d[Self::OFF_CLAIMED..Self::OFF_CLAIMED + Self::CLAIM_BYTES].try_into().unwrap(),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_DISTRIBUTION);
        wr_b32(&mut d, Self::OFF_HASH, &self.hash);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_u64(&mut d, Self::OFF_AMOUNT, self.amount);
        wr_u64(&mut d, Self::OFF_TOTAL_WEIGHT, self.total_weight);
        wr_u64(&mut d, Self::OFF_PAID, self.paid);
        wr_u32(&mut d, Self::OFF_CLAIMS, self.claims);
        d[Self::OFF_BUMP] = self.bump;
        wr_b32(&mut d, Self::OFF_MINT, &self.mint);
        wr_b32(&mut d, Self::OFF_PAYER, &self.payer);
        wr_u64(&mut d, Self::OFF_CREATED, self.created);
        d[Self::OFF_CLAIMED..Self::OFF_CLAIMED + Self::CLAIM_BYTES].copy_from_slice(&self.claimed);
        Ok(())
    }
    pub fn claimed(&self, i: usize) -> bool {
        self.claimed[i / 8] & (1 << (i % 8)) != 0
    }
    pub fn mark(&mut self, i: usize) {
        self.claimed[i / 8] |= 1 << (i % 8);
    }
    pub fn is_token(&self) -> bool {
        self.mint != [0u8; 32]
    }
}

// ============================ the economy ============================

// ---- Allowance ----
// A standing authorization: the payee pulls up to `cap` per `period` from
// the unit's treasury (mint zero) or its vault of `mint`, each pull charged
// to the plan, until revoked by a later proposal or past `expires` (zero:
// none). A key payee pulls by its own signature; a unit payee is pulled for
// by anyone into its own treasury or vault. Keyed by the proposal that made it.
// [disc:1][ver:1][hash:32][unit:8][payee:32][payee_kind:1][mint:32][cap:8]
// [period:8][start:8][spent:8][expires:8][payer:32][bump:1]
pub struct Allowance {
    pub hash: [u8; 32],
    pub unit_id: u64,
    pub payee: [u8; 32],
    pub payee_kind: u8,
    pub mint: [u8; 32],
    pub cap: u64,
    pub period: u64,
    pub start: u64,
    pub spent: u64,
    pub expires: u64,
    pub payer: [u8; 32],
    pub bump: u8,
}
impl Allowance {
    pub const OFF_HASH: usize = 2;
    pub const OFF_UNIT: usize = 34;
    pub const OFF_PAYEE: usize = 42;
    pub const OFF_PAYEE_KIND: usize = 74;
    pub const OFF_MINT: usize = 75;
    pub const OFF_CAP: usize = 107;
    pub const OFF_PERIOD: usize = 115;
    pub const OFF_START: usize = 123;
    pub const OFF_SPENT: usize = 131;
    pub const OFF_EXPIRES: usize = 139;
    pub const OFF_PAYER: usize = 147;
    pub const OFF_BUMP: usize = 179;
    pub const SIZE: usize = 180;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_ALLOWANCE, Self::SIZE)?;
        Ok(Self {
            hash: rd_h32(&d, Self::OFF_HASH),
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            payee: rd_h32(&d, Self::OFF_PAYEE),
            payee_kind: d[Self::OFF_PAYEE_KIND],
            mint: rd_h32(&d, Self::OFF_MINT),
            cap: rd_u64(&d, Self::OFF_CAP),
            period: rd_u64(&d, Self::OFF_PERIOD),
            start: rd_u64(&d, Self::OFF_START),
            spent: rd_u64(&d, Self::OFF_SPENT),
            expires: rd_u64(&d, Self::OFF_EXPIRES),
            payer: rd_h32(&d, Self::OFF_PAYER),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_ALLOWANCE);
        wr_b32(&mut d, Self::OFF_HASH, &self.hash);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_PAYEE, &self.payee);
        d[Self::OFF_PAYEE_KIND] = self.payee_kind;
        wr_b32(&mut d, Self::OFF_MINT, &self.mint);
        wr_u64(&mut d, Self::OFF_CAP, self.cap);
        wr_u64(&mut d, Self::OFF_PERIOD, self.period);
        wr_u64(&mut d, Self::OFF_START, self.start);
        wr_u64(&mut d, Self::OFF_SPENT, self.spent);
        wr_u64(&mut d, Self::OFF_EXPIRES, self.expires);
        wr_b32(&mut d, Self::OFF_PAYER, &self.payer);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
    pub fn is_token(&self) -> bool {
        self.mint != [0u8; 32]
    }
    /// Past its expiry (revocation closes the account outright).
    pub fn expired(&self, now: u64) -> bool {
        self.expires != 0 && now > self.expires
    }
}

// ---- Contract ----
// Escrow released by a ruling. The paying unit locks `amount` (lamports in
// the contract account itself, tokens in an escrow token account beside it)
// under a manifest, for a payee, until `deadline`. Released to the payee or
// refunded by a SIGNAL ruling of the paying unit or of the arbiter unit;
// refunded by anyone past the deadline, or by the payee's own resignation.
// Settled contracts close at once: the receipt in the log is the record.
// [disc:1][ver:1][hash:32][unit:8][payee:32][payee_kind:1][mint:32][amount:8]
// [deadline:8][arbiter:8][payer:32][bump:1]
pub struct Contract {
    pub hash: [u8; 32],
    pub unit_id: u64,
    pub payee: [u8; 32],
    pub payee_kind: u8,
    pub mint: [u8; 32],
    pub amount: u64,
    pub deadline: u64,
    pub arbiter: u64,
    pub payer: [u8; 32],
    pub bump: u8,
}
impl Contract {
    pub const OFF_HASH: usize = 2;
    pub const OFF_UNIT: usize = 34;
    pub const OFF_PAYEE: usize = 42;
    pub const OFF_PAYEE_KIND: usize = 74;
    pub const OFF_MINT: usize = 75;
    pub const OFF_AMOUNT: usize = 107;
    pub const OFF_DEADLINE: usize = 115;
    pub const OFF_ARBITER: usize = 123;
    pub const OFF_PAYER: usize = 131;
    pub const OFF_BUMP: usize = 163;
    pub const SIZE: usize = 164;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_CONTRACT, Self::SIZE)?;
        Ok(Self {
            hash: rd_h32(&d, Self::OFF_HASH),
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            payee: rd_h32(&d, Self::OFF_PAYEE),
            payee_kind: d[Self::OFF_PAYEE_KIND],
            mint: rd_h32(&d, Self::OFF_MINT),
            amount: rd_u64(&d, Self::OFF_AMOUNT),
            deadline: rd_u64(&d, Self::OFF_DEADLINE),
            arbiter: rd_u64(&d, Self::OFF_ARBITER),
            payer: rd_h32(&d, Self::OFF_PAYER),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_CONTRACT);
        wr_b32(&mut d, Self::OFF_HASH, &self.hash);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_PAYEE, &self.payee);
        d[Self::OFF_PAYEE_KIND] = self.payee_kind;
        wr_b32(&mut d, Self::OFF_MINT, &self.mint);
        wr_u64(&mut d, Self::OFF_AMOUNT, self.amount);
        wr_u64(&mut d, Self::OFF_DEADLINE, self.deadline);
        wr_u64(&mut d, Self::OFF_ARBITER, self.arbiter);
        wr_b32(&mut d, Self::OFF_PAYER, &self.payer);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
    pub fn is_token(&self) -> bool {
        self.mint != [0u8; 32]
    }
}

// ---- TokenPlan ----
// A mint the society recognizes beyond the default: its own cap per period
// (the registry's period), in the mint's own units, created and changed by
// CODEX AllowMint, closed by AllowMint with cap zero.
// [disc:1][ver:1][unit:8][mint:32][cap:8][spent:8][start:8][payer:32][bump:1]
pub struct TokenPlan {
    pub unit_id: u64,
    pub mint: [u8; 32],
    pub cap: u64,
    pub spent: u64,
    pub start: u64,
    pub payer: [u8; 32],
    pub bump: u8,
}
impl TokenPlan {
    pub const OFF_UNIT: usize = 2;
    pub const OFF_MINT: usize = 10;
    pub const OFF_CAP: usize = 42;
    pub const OFF_SPENT: usize = 50;
    pub const OFF_START: usize = 58;
    pub const OFF_PAYER: usize = 66;
    pub const OFF_BUMP: usize = 98;
    pub const SIZE: usize = 99;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_TOKEN_PLAN, Self::SIZE)?;
        Ok(Self {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            mint: rd_h32(&d, Self::OFF_MINT),
            cap: rd_u64(&d, Self::OFF_CAP),
            spent: rd_u64(&d, Self::OFF_SPENT),
            start: rd_u64(&d, Self::OFF_START),
            payer: rd_h32(&d, Self::OFF_PAYER),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_TOKEN_PLAN);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_MINT, &self.mint);
        wr_u64(&mut d, Self::OFF_CAP, self.cap);
        wr_u64(&mut d, Self::OFF_SPENT, self.spent);
        wr_u64(&mut d, Self::OFF_START, self.start);
        wr_b32(&mut d, Self::OFF_PAYER, &self.payer);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ---- Listing ----
// A seat offer on the market: the seller's seats at a price each, bought in
// part or whole by anyone, the lamports going straight to the seller and the
// seats moving as a transfer would (resting seats cannot be sold). No seats
// are escrowed: the listing is an offer, checked against the registry at
// the moment of purchase.
// [disc:1][ver:1][unit:8][seller:32][nonce:8][count:8][price:8][bump:1]
pub struct Listing {
    pub unit_id: u64,
    pub seller: [u8; 32],
    pub nonce: u64,
    pub count: u64,
    pub price: u64,
    pub bump: u8,
}
impl Listing {
    pub const OFF_UNIT: usize = 2;
    pub const OFF_SELLER: usize = 10;
    pub const OFF_NONCE: usize = 42;
    pub const OFF_COUNT: usize = 50;
    pub const OFF_PRICE: usize = 58;
    pub const OFF_BUMP: usize = 66;
    pub const SIZE: usize = 67;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_LISTING, Self::SIZE)?;
        Ok(Self {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            seller: rd_h32(&d, Self::OFF_SELLER),
            nonce: rd_u64(&d, Self::OFF_NONCE),
            count: rd_u64(&d, Self::OFF_COUNT),
            price: rd_u64(&d, Self::OFF_PRICE),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        stamp(&mut d, DISC_LISTING);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_SELLER, &self.seller);
        wr_u64(&mut d, Self::OFF_NONCE, self.nonce);
        wr_u64(&mut d, Self::OFF_COUNT, self.count);
        wr_u64(&mut d, Self::OFF_PRICE, self.price);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ============================ the fund ============================
// The polis of poleis: one per program, at the one address "fund". Fees
// from every seat sale, market purchase and trace lot settlement land here
// at the share the root polis set; the fund pays the current option's
// recipient each cycle. Twelve chairs: the roots that generated the most
// fees this cycle, each backing one option.
// [disc:1][ver:1][bump:1][fee_bps:2][cycle_slots:8][cycle_start:8][cycle_seq:8]
// [total_in:8][total_out:8][staked_total:8][wp_sum:16][wp_weight:8]
// [range_lo:2][range_hi:2][option_fee:8][chair_quorum:1][current:32][current_since:8][chairs:12×48][current_recipient:32][owed:8][owed_to:32]
// [chairs: 12 × (root:8, fees:8, backing:32)]
pub const DISC_FUND: u8 = 17;
pub const DISC_FEE_TALLY: u8 = 18;
pub const DISC_OPTION: u8 = 19;
pub const DISC_STAKE: u8 = 20;
pub const DISC_POOL: u8 = 21;
pub const DISC_KNOCK: u8 = 22;
/// An empty chair of the fund (root 0 is a real root).
pub const NO_ROOT: u64 = u64::MAX;

#[inline(always)]
fn rd_u128(d: &[u8], o: usize) -> u128 {
    u128::from_le_bytes(d[o..o + 16].try_into().unwrap())
}
#[inline(always)]
fn wr_u128(d: &mut [u8], o: usize, v: u128) {
    d[o..o + 16].copy_from_slice(&v.to_le_bytes());
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Chair {
    pub root: u64,
    pub fees: u64,
    pub backing: [u8; 32],
}

pub struct Fund {
    pub bump: u8,
    pub fee_bps: u16,
    pub cycle_slots: u64,
    pub cycle_start: u64,
    pub cycle_seq: u64,
    pub total_in: u64,
    pub total_out: u64,
    pub staked_total: u64,
    pub wp_sum: u128,
    pub wp_weight: u64,
    pub range_lo: u16,
    pub range_hi: u16,
    pub option_fee: u64,
    pub chair_quorum: u8,
    pub current: [u8; 32],
    pub current_since: u64,
    pub chairs: [Chair; crate::PENTAGONS],
    /// The current option's recipient, copied when it became current, so a
    /// cycle can roll without the option account in the transaction.
    pub current_recipient: [u8; 32],
    /// The grant booked when a cycle rolled and not yet claimed, and to whom.
    /// One slot: a roll that finds it held by another recipient defers its
    /// grant to the next cycle's share instead of losing it.
    pub owed: u64,
    pub owed_to: [u8; 32],
}
impl Fund {
    pub const OFF_BUMP: usize = 2;
    pub const OFF_FEE_BPS: usize = 3;
    pub const OFF_CYCLE_SLOTS: usize = 5;
    pub const OFF_CYCLE_START: usize = 13;
    pub const OFF_CYCLE_SEQ: usize = 21;
    pub const OFF_TOTAL_IN: usize = 29;
    pub const OFF_TOTAL_OUT: usize = 37;
    pub const OFF_STAKED: usize = 45;
    pub const OFF_WP_SUM: usize = 53;
    pub const OFF_WP_WEIGHT: usize = 69;
    pub const OFF_RANGE_LO: usize = 77;
    pub const OFF_RANGE_HI: usize = 79;
    pub const OFF_OPTION_FEE: usize = 81;
    pub const OFF_CHAIR_QUORUM: usize = 89;
    pub const OFF_CURRENT: usize = 90;
    pub const OFF_CURRENT_SINCE: usize = 122;
    pub const OFF_CHAIRS: usize = 130;
    pub const CHAIR_LEN: usize = 48;
    pub const OFF_CURRENT_RECIPIENT: usize = Self::OFF_CHAIRS + crate::PENTAGONS * Self::CHAIR_LEN;
    pub const OFF_OWED: usize = Self::OFF_CURRENT_RECIPIENT + 32;
    pub const OFF_OWED_TO: usize = Self::OFF_OWED + 8;
    pub const SIZE: usize = Self::OFF_OWED_TO + 32;
    /// The older fund (Sept 6 2026's build, devnet's): the same bytes up to the end of the twelve chairs, and nothing after; its
    /// cycle paid the current option straight from the fund, so it owed nothing and kept no recipient (`ix::fund::grow_fund`).
    pub const OLD_SIZE: usize = Self::OFF_CURRENT_RECIPIENT;
    /// A fund just constituted: no money, no stake, no chairs, no current option.
    pub fn empty(bump: u8, now: u64) -> Self {
        Self {
            bump,
            fee_bps: 0,
            cycle_slots: 0,
            cycle_start: now,
            cycle_seq: 0,
            total_in: 0,
            total_out: 0,
            staked_total: 0,
            wp_sum: 0,
            wp_weight: 0,
            range_lo: 0,
            range_hi: 0,
            option_fee: 0,
            chair_quorum: crate::PENTAGON_QUORUM,
            current: [0u8; 32],
            current_since: 0,
            chairs: [Chair { root: NO_ROOT, fees: 0, backing: [0u8; 32] }; crate::PENTAGONS],
            current_recipient: [0u8; 32],
            owed: 0,
            owed_to: [0u8; 32],
        }
    }
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_FUND, Self::SIZE)?;
        let mut chairs = [Chair { root: NO_ROOT, fees: 0, backing: [0u8; 32] }; crate::PENTAGONS];
        for (k, ch) in chairs.iter_mut().enumerate() {
            let o = Self::OFF_CHAIRS + k * Self::CHAIR_LEN;
            *ch = Chair { root: rd_u64(&d, o), fees: rd_u64(&d, o + 8), backing: rd_h32(&d, o + 16) };
        }
        Ok(Self {
            bump: d[Self::OFF_BUMP],
            fee_bps: rd_u16(&d, Self::OFF_FEE_BPS),
            cycle_slots: rd_u64(&d, Self::OFF_CYCLE_SLOTS),
            cycle_start: rd_u64(&d, Self::OFF_CYCLE_START),
            cycle_seq: rd_u64(&d, Self::OFF_CYCLE_SEQ),
            total_in: rd_u64(&d, Self::OFF_TOTAL_IN),
            total_out: rd_u64(&d, Self::OFF_TOTAL_OUT),
            staked_total: rd_u64(&d, Self::OFF_STAKED),
            wp_sum: rd_u128(&d, Self::OFF_WP_SUM),
            wp_weight: rd_u64(&d, Self::OFF_WP_WEIGHT),
            range_lo: rd_u16(&d, Self::OFF_RANGE_LO),
            range_hi: rd_u16(&d, Self::OFF_RANGE_HI),
            option_fee: rd_u64(&d, Self::OFF_OPTION_FEE),
            chair_quorum: d[Self::OFF_CHAIR_QUORUM],
            current: rd_h32(&d, Self::OFF_CURRENT),
            current_since: rd_u64(&d, Self::OFF_CURRENT_SINCE),
            chairs,
            current_recipient: rd_h32(&d, Self::OFF_CURRENT_RECIPIENT),
            owed: rd_u64(&d, Self::OFF_OWED),
            owed_to: rd_h32(&d, Self::OFF_OWED_TO),
        })
    }
    /// The older fund read by its own layout: every field it has at the offset it had, the three it lacks empty. Only an account this
    /// program owns, of exactly `OLD_SIZE` bytes, stamped a fund of this layout byte; anything else is not the older fund.
    pub fn read_older(acc: &AccountView) -> Result<Self, ProgramError> {
        if !acc.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        let d = acc.try_borrow()?;
        if d.len() != Self::OLD_SIZE || d[OFF_DISC] != DISC_FUND || d[OFF_VERSION] != LAYOUT_VERSION {
            return Err(PoleisError::NotAnOlderFund.into());
        }
        let mut chairs = [Chair { root: NO_ROOT, fees: 0, backing: [0u8; 32] }; crate::PENTAGONS];
        for (k, ch) in chairs.iter_mut().enumerate() {
            let o = Self::OFF_CHAIRS + k * Self::CHAIR_LEN;
            *ch = Chair { root: rd_u64(&d, o), fees: rd_u64(&d, o + 8), backing: rd_h32(&d, o + 16) };
        }
        Ok(Self {
            bump: d[Self::OFF_BUMP],
            fee_bps: rd_u16(&d, Self::OFF_FEE_BPS),
            cycle_slots: rd_u64(&d, Self::OFF_CYCLE_SLOTS),
            cycle_start: rd_u64(&d, Self::OFF_CYCLE_START),
            cycle_seq: rd_u64(&d, Self::OFF_CYCLE_SEQ),
            total_in: rd_u64(&d, Self::OFF_TOTAL_IN),
            total_out: rd_u64(&d, Self::OFF_TOTAL_OUT),
            staked_total: rd_u64(&d, Self::OFF_STAKED),
            wp_sum: rd_u128(&d, Self::OFF_WP_SUM),
            wp_weight: rd_u64(&d, Self::OFF_WP_WEIGHT),
            range_lo: rd_u16(&d, Self::OFF_RANGE_LO),
            range_hi: rd_u16(&d, Self::OFF_RANGE_HI),
            option_fee: rd_u64(&d, Self::OFF_OPTION_FEE),
            chair_quorum: d[Self::OFF_CHAIR_QUORUM],
            current: rd_h32(&d, Self::OFF_CURRENT),
            current_since: rd_u64(&d, Self::OFF_CURRENT_SINCE),
            chairs,
            current_recipient: [0u8; 32],
            owed: 0,
            owed_to: [0u8; 32],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_FUND);
        d[Self::OFF_BUMP] = self.bump;
        wr_u16(&mut d, Self::OFF_FEE_BPS, self.fee_bps);
        wr_u64(&mut d, Self::OFF_CYCLE_SLOTS, self.cycle_slots);
        wr_u64(&mut d, Self::OFF_CYCLE_START, self.cycle_start);
        wr_u64(&mut d, Self::OFF_CYCLE_SEQ, self.cycle_seq);
        wr_u64(&mut d, Self::OFF_TOTAL_IN, self.total_in);
        wr_u64(&mut d, Self::OFF_TOTAL_OUT, self.total_out);
        wr_u64(&mut d, Self::OFF_STAKED, self.staked_total);
        wr_u128(&mut d, Self::OFF_WP_SUM, self.wp_sum);
        wr_u64(&mut d, Self::OFF_WP_WEIGHT, self.wp_weight);
        wr_u16(&mut d, Self::OFF_RANGE_LO, self.range_lo);
        wr_u16(&mut d, Self::OFF_RANGE_HI, self.range_hi);
        wr_u64(&mut d, Self::OFF_OPTION_FEE, self.option_fee);
        d[Self::OFF_CHAIR_QUORUM] = self.chair_quorum;
        wr_b32(&mut d, Self::OFF_CURRENT, &self.current);
        wr_u64(&mut d, Self::OFF_CURRENT_SINCE, self.current_since);
        for (k, ch) in self.chairs.iter().enumerate() {
            let o = Self::OFF_CHAIRS + k * Self::CHAIR_LEN;
            wr_u64(&mut d, o, ch.root);
            wr_u64(&mut d, o + 8, ch.fees);
            wr_b32(&mut d, o + 16, &ch.backing);
        }
        wr_b32(&mut d, Self::OFF_CURRENT_RECIPIENT, &self.current_recipient);
        wr_u64(&mut d, Self::OFF_OWED, self.owed);
        wr_b32(&mut d, Self::OFF_OWED_TO, &self.owed_to);
        Ok(())
    }
}

// ---- FeeTally: the fees a root generated, this cycle and for life ----
// [disc:1][ver:1][bump:1][root:8][cycle:8][fees:8][lifetime:8]
pub struct FeeTally {
    pub bump: u8,
    pub root: u64,
    pub cycle: u64,
    pub fees: u64,
    pub lifetime: u64,
}
impl FeeTally {
    pub const OFF_BUMP: usize = 2;
    pub const OFF_ROOT: usize = 3;
    pub const OFF_CYCLE: usize = 11;
    pub const OFF_FEES: usize = 19;
    pub const OFF_LIFETIME: usize = 27;
    pub const SIZE: usize = 35;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_FEE_TALLY, Self::SIZE)?;
        Ok(Self {
            bump: d[Self::OFF_BUMP],
            root: rd_u64(&d, Self::OFF_ROOT),
            cycle: rd_u64(&d, Self::OFF_CYCLE),
            fees: rd_u64(&d, Self::OFF_FEES),
            lifetime: rd_u64(&d, Self::OFF_LIFETIME),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_FEE_TALLY);
        d[Self::OFF_BUMP] = self.bump;
        wr_u64(&mut d, Self::OFF_ROOT, self.root);
        wr_u64(&mut d, Self::OFF_CYCLE, self.cycle);
        wr_u64(&mut d, Self::OFF_FEES, self.fees);
        wr_u64(&mut d, Self::OFF_LIFETIME, self.lifetime);
        Ok(())
    }
}

// ---- Candidate: an option for where the fund goes ----
// [disc:1][ver:1][bump:1][hash:32][recipient:32][manifest:32][proposer:32]
// [vetted:1][floor_stake:8][created:8]
pub struct Candidate {
    pub bump: u8,
    pub hash: [u8; 32],
    pub recipient: [u8; 32],
    pub manifest: [u8; 32],
    pub proposer: [u8; 32],
    pub vetted: bool,
    pub floor_stake: u64,
    pub created: u64,
}
impl Candidate {
    pub const OFF_BUMP: usize = 2;
    pub const OFF_HASH: usize = 3;
    pub const OFF_RECIPIENT: usize = 35;
    pub const OFF_MANIFEST: usize = 67;
    pub const OFF_PROPOSER: usize = 99;
    pub const OFF_VETTED: usize = 131;
    pub const OFF_FLOOR_STAKE: usize = 132;
    pub const OFF_CREATED: usize = 140;
    pub const SIZE: usize = 148;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_OPTION, Self::SIZE)?;
        Ok(Self {
            bump: d[Self::OFF_BUMP],
            hash: rd_h32(&d, Self::OFF_HASH),
            recipient: rd_h32(&d, Self::OFF_RECIPIENT),
            manifest: rd_h32(&d, Self::OFF_MANIFEST),
            proposer: rd_h32(&d, Self::OFF_PROPOSER),
            vetted: d[Self::OFF_VETTED] != 0,
            floor_stake: rd_u64(&d, Self::OFF_FLOOR_STAKE),
            created: rd_u64(&d, Self::OFF_CREATED),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_OPTION);
        d[Self::OFF_BUMP] = self.bump;
        wr_b32(&mut d, Self::OFF_HASH, &self.hash);
        wr_b32(&mut d, Self::OFF_RECIPIENT, &self.recipient);
        wr_b32(&mut d, Self::OFF_MANIFEST, &self.manifest);
        wr_b32(&mut d, Self::OFF_PROPOSER, &self.proposer);
        d[Self::OFF_VETTED] = self.vetted as u8;
        wr_u64(&mut d, Self::OFF_FLOOR_STAKE, self.floor_stake);
        wr_u64(&mut d, Self::OFF_CREATED, self.created);
        Ok(())
    }
}

// ---- Stake: one holder's seats in one unit, on one option ----
// [disc:1][ver:1][bump:1][hash:32][unit:8][holder:32][weight:8][value_bps:2]
pub struct Stake {
    pub bump: u8,
    pub hash: [u8; 32],
    pub unit_id: u64,
    pub holder: [u8; 32],
    pub weight: u64,
    pub value_bps: u16,
}
impl Stake {
    pub const OFF_BUMP: usize = 2;
    pub const OFF_HASH: usize = 3;
    pub const OFF_UNIT: usize = 35;
    pub const OFF_HOLDER: usize = 43;
    pub const OFF_WEIGHT: usize = 75;
    pub const OFF_VALUE: usize = 83;
    pub const SIZE: usize = 85;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_STAKE, Self::SIZE)?;
        Ok(Self {
            bump: d[Self::OFF_BUMP],
            hash: rd_h32(&d, Self::OFF_HASH),
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            holder: rd_h32(&d, Self::OFF_HOLDER),
            weight: rd_u64(&d, Self::OFF_WEIGHT),
            value_bps: rd_u16(&d, Self::OFF_VALUE),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_STAKE);
        d[Self::OFF_BUMP] = self.bump;
        wr_b32(&mut d, Self::OFF_HASH, &self.hash);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_HOLDER, &self.holder);
        wr_u64(&mut d, Self::OFF_WEIGHT, self.weight);
        wr_u16(&mut d, Self::OFF_VALUE, self.value_bps);
        Ok(())
    }
}

// ---- Pool: the spots of a polis open by admission ----
// [disc:1][ver:1][bump:1][unit:8][open:8][filled:8][price:8]
// `open` counts every spot a door-mode climb offered, `filled` the admitted;
// the difference is what a knock may still take. `price` is the last climb's.
pub struct Pool {
    pub bump: u8,
    pub unit_id: u64,
    pub open: u64,
    pub filled: u64,
    pub price: u64,
}
impl Pool {
    pub const OFF_BUMP: usize = 2;
    pub const OFF_UNIT: usize = 3;
    pub const OFF_OPEN: usize = 11;
    pub const OFF_FILLED: usize = 19;
    pub const OFF_PRICE: usize = 27;
    pub const SIZE: usize = 35;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_POOL, Self::SIZE)?;
        Ok(Self {
            bump: d[Self::OFF_BUMP],
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            open: rd_u64(&d, Self::OFF_OPEN),
            filled: rd_u64(&d, Self::OFF_FILLED),
            price: rd_u64(&d, Self::OFF_PRICE),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_POOL);
        d[Self::OFF_BUMP] = self.bump;
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_u64(&mut d, Self::OFF_OPEN, self.open);
        wr_u64(&mut d, Self::OFF_FILLED, self.filled);
        wr_u64(&mut d, Self::OFF_PRICE, self.price);
        Ok(())
    }
}

// ---- Knock: one key waiting at the door of one polis, its price in escrow ----
// [disc:1][ver:1][bump:1][unit:8][key:32][deposit:8][slot:8]
// The account holds `deposit` plus the rent of one holder row above its own
// rent; the polis's verdict, or a CODEX window of silence, closes it.
pub struct Knock {
    pub bump: u8,
    pub unit_id: u64,
    pub key: [u8; 32],
    pub deposit: u64,
    pub slot: u64,
}
impl Knock {
    pub const OFF_BUMP: usize = 2;
    pub const OFF_UNIT: usize = 3;
    pub const OFF_KEY: usize = 11;
    pub const OFF_DEPOSIT: usize = 43;
    pub const OFF_SLOT: usize = 51;
    pub const SIZE: usize = 59;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_KNOCK, Self::SIZE)?;
        Ok(Self {
            bump: d[Self::OFF_BUMP],
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            key: rd_h32(&d, Self::OFF_KEY),
            deposit: rd_u64(&d, Self::OFF_DEPOSIT),
            slot: rd_u64(&d, Self::OFF_SLOT),
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_KNOCK);
        d[Self::OFF_BUMP] = self.bump;
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_KEY, &self.key);
        wr_u64(&mut d, Self::OFF_DEPOSIT, self.deposit);
        wr_u64(&mut d, Self::OFF_SLOT, self.slot);
        Ok(())
    }
}

// ---- BlsKey ----
// A holder's BLS12-381 key for the aggregate vote: one per (unit, holder),
// registered by the holder itself with a proof of possession over these
// very bytes, read by `attest_bls` to add the signers' keys up.
// [disc:1][ver:1][unit:8][holder:32][pk:192, uncompressed G2 big-endian][bump:1]
pub const DISC_BLS_KEY: u8 = 25;
pub struct BlsKey {
    pub unit_id: u64,
    pub holder: [u8; 32],
    pub pk: [u8; 192],
    pub bump: u8,
}
impl BlsKey {
    pub const OFF_UNIT: usize = 2;
    pub const OFF_HOLDER: usize = 10;
    pub const OFF_PK: usize = 42;
    pub const OFF_BUMP: usize = 234;
    pub const SIZE: usize = 235;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_BLS_KEY, Self::SIZE)?;
        Ok(Self {
            unit_id: rd_u64(&d, Self::OFF_UNIT),
            holder: rd_h32(&d, Self::OFF_HOLDER),
            pk: d[Self::OFF_PK..Self::OFF_PK + 192].try_into().unwrap(),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(PoleisError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_BLS_KEY);
        wr_u64(&mut d, Self::OFF_UNIT, self.unit_id);
        wr_b32(&mut d, Self::OFF_HOLDER, &self.holder);
        d[Self::OFF_PK..Self::OFF_PK + 192].copy_from_slice(&self.pk);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROW: usize = Session::ELECTORATE_ROW;
    const TAIL: usize = Session::CONVENER_TAIL;
    /// A roster of two rows with room for a convener's tail after it.
    const ROSTER2: usize = Session::SIZE + Session::ROSTER_HEADER + 2 * ROW;

    fn roster2(lit: u16, crews: u16, opened: u64) -> [u8; ROSTER2 + TAIL] {
        let mut d = [0u8; ROSTER2 + TAIL];
        d[Session::SIZE..Session::SIZE + 8].copy_from_slice(&Session::ROSTER_MAGIC);
        d[Session::SIZE + 8] = 2;
        wr_u16(&mut d, Session::SIZE + 9, lit);
        wr_u16(&mut d, Session::SIZE + 11, crews);
        wr_u64(&mut d, Session::SIZE + 13, opened);
        let a = Session::SIZE + Session::ROSTER_HEADER;
        wr_u64(&mut d, a + 33, 7);
        d[a + 41] = 1;
        wr_u64(&mut d, a + ROW + 33, 3);
        d
    }
    fn sign(d: &mut [u8]) {
        let n = d.len();
        d[n - TAIL..n - 32].copy_from_slice(&Session::CONVENER_MAGIC);
        d[n - 32..].copy_from_slice(&[5u8; 32]);
    }

    #[test]
    fn the_roster_reads_its_chairs_and_its_rows() {
        let d = roster2(0b1111_1111_1111, 0b1100_0000_0000, 4_242);
        let body = &d[..ROSTER2];
        assert!(Session::valid_body(body));
        assert_eq!(Session::body_len(body), ROSTER2);
        assert_eq!(Session::roster(body).unwrap(), Some((0b1111_1111_1111, 0b1100_0000_0000, 4_242)));
        assert_eq!(Session::electorate_weights(body).unwrap(), Some((10, 7)));
        assert_eq!(Session::convener(body), None);
        // the same roster signed by its convener: the tail is set aside
        let mut signed = d;
        sign(&mut signed);
        assert_eq!(Session::body_len(&signed), ROSTER2);
        assert_eq!(Session::convener(&signed), Some([5u8; 32]));
        assert_eq!(Session::roster(&signed).unwrap(), Some((0b1111_1111_1111, 0b1100_0000_0000, 4_242)));
        assert_eq!(Session::electorate_weights(&signed).unwrap(), Some((10, 7)));
    }

    #[test]
    fn a_roster_of_no_holder_is_whole() {
        let mut d = [0u8; Session::SIZE + Session::ROSTER_HEADER];
        d[Session::SIZE..Session::SIZE + 8].copy_from_slice(&Session::ROSTER_MAGIC);
        wr_u16(&mut d, Session::SIZE + 9, 0b111);
        wr_u64(&mut d, Session::SIZE + 13, 9);
        assert_eq!(Session::roster(&d).unwrap(), Some((0b111, 0, 9)));
        assert_eq!(Session::electorate_weights(&d).unwrap(), Some((0, 0)));
        let mut signed = [0u8; Session::SIZE + Session::ROSTER_HEADER + TAIL];
        signed[..d.len()].copy_from_slice(&d);
        sign(&mut signed);
        assert_eq!(Session::body_len(&signed), d.len());
        assert_eq!(Session::roster(&signed).unwrap(), Some((0b111, 0, 9)));
    }

    #[test]
    fn a_malformed_roster_is_refused() {
        let d = roster2(0b11, 0b01, 1);
        // every truncation of the body, and one byte more
        for length in Session::SIZE + 1..ROSTER2 {
            assert!(Session::roster(&d[..length]).is_err(), "{length}");
            assert!(Session::electorate_weights(&d[..length]).is_err(), "{length}");
        }
        assert!(Session::roster(&d[..ROSTER2 + 1]).is_err());
        // a chair past the twelve, a crew not lit, a count past the cap, an unknown magic
        assert!(Session::roster(&roster2(1 << crate::PENTAGONS, 0, 1)[..ROSTER2]).is_err());
        assert!(Session::roster(&roster2(0b01, 0b10, 1)[..ROSTER2]).is_err());
        let mut bad = d;
        bad[Session::SIZE + 8] = (crate::MAX_HOLDERS + 1) as u8;
        assert!(Session::roster(&bad[..ROSTER2]).is_err());
        let mut bad = d;
        bad[Session::SIZE + 7] = b'3';
        assert!(Session::roster(&bad[..ROSTER2]).is_err());
        assert!(Session::electorate_weights(&bad[..ROSTER2]).is_err());
    }

    #[test]
    fn the_older_shapes_read_as_before_and_carry_no_roster() {
        // the bare prefix, alone and signed
        let bare = [0u8; Session::SIZE];
        assert_eq!(Session::roster(&bare).unwrap(), None);
        assert_eq!(Session::electorate_weights(&bare).unwrap(), None);
        let mut bare_signed = [0u8; Session::SIZE + TAIL];
        sign(&mut bare_signed);
        assert_eq!(Session::body_len(&bare_signed), Session::SIZE);
        assert_eq!(Session::roster(&bare_signed).unwrap(), None);
        // "NFTVOTE1" with one row, alone and signed
        const OLD: usize = Session::SIZE + Session::ELECTORATE_HEADER + ROW;
        let mut old = [0u8; OLD + TAIL];
        old[Session::SIZE..Session::SIZE + 8].copy_from_slice(&Session::ELECTORATE_MAGIC);
        old[Session::SIZE + 8] = 1;
        wr_u64(&mut old, Session::SIZE + Session::ELECTORATE_HEADER + 33, 4);
        assert_eq!(Session::roster(&old[..OLD]).unwrap(), None);
        assert_eq!(Session::electorate_weights(&old[..OLD]).unwrap(), Some((4, 0)));
        sign(&mut old);
        assert_eq!(Session::body_len(&old), OLD);
        assert_eq!(Session::roster(&old).unwrap(), None);
        assert_eq!(Session::electorate_weights(&old).unwrap(), Some((4, 0)));
    }

    #[test]
    fn the_roster_is_bounded() {
        assert_eq!(Session::ROSTER_HEADER, 8 + 1 + 2 + 2 + 8);
        assert_eq!(Session::ROSTER_HEADER + ROW * crate::MAX_HOLDERS, 6321);
        // no roster length differs from another body's, or from the bare prefix, by a tail
        for c in 0..=crate::MAX_HOLDERS {
            let roster = Session::ROSTER_HEADER + c * ROW;
            assert_ne!(roster, TAIL);
            for c2 in 0..=crate::MAX_HOLDERS {
                assert_ne!(roster + TAIL, Session::ROSTER_HEADER + c2 * ROW);
            }
        }
    }

    // ---- a distribution's roster ----

    const TWO: usize = Distribution::SIZE + Distribution::ROSTER_HEADER + 2 * Distribution::ROSTER_ROW;

    /// A distribution's bytes with a roster of two rows: a key of weight 3
    /// and an emptied row.
    fn two_rows() -> [u8; TWO] {
        let mut d = [0u8; TWO];
        d[Distribution::SIZE..Distribution::SIZE + 8].copy_from_slice(&Distribution::ROSTER_MAGIC);
        d[Distribution::SIZE + 8] = 2;
        let row = Distribution::SIZE + Distribution::ROSTER_HEADER;
        d[row..row + 32].copy_from_slice(&[7u8; 32]);
        d[row + 32] = crate::HOLDER_KEY;
        d[row + 33..row + 41].copy_from_slice(&3u64.to_le_bytes());
        d
    }

    #[test]
    fn a_roster_is_read_back_row_by_row_and_found_by_identity() {
        let d = two_rows();
        assert!(Distribution::valid_data_len(&d));
        assert_eq!(Distribution::roster_row(&d, 0).unwrap(), Some(([7u8; 32], crate::HOLDER_KEY, 3)));
        assert_eq!(Distribution::roster_row(&d, 1).unwrap(), Some(([0u8; 32], crate::HOLDER_EMPTY, 0)));
        assert_eq!(Distribution::roster_row(&d, 2).unwrap(), None, "past the count");
        assert_eq!(Distribution::roster_find(&d, crate::HOLDER_KEY, &[7u8; 32]).unwrap(), Some((0, 3)));
        assert_eq!(Distribution::roster_find(&d, crate::HOLDER_UNIT, &[7u8; 32]).unwrap(), None, "the kind is part of the identity");
        assert_eq!(Distribution::roster_find(&d, crate::HOLDER_KEY, &[8u8; 32]).unwrap(), None);
    }

    #[test]
    fn a_distribution_without_a_roster_is_the_old_shape_and_a_bad_roster_is_refused() {
        let old = [0u8; Distribution::SIZE];
        assert!(Distribution::valid_data_len(&old));
        assert_eq!(Distribution::roster_row(&old, 0).unwrap(), None);
        assert_eq!(Distribution::roster_find(&old, crate::HOLDER_KEY, &[0u8; 32]).unwrap(), None);

        // a count that disagrees with the length
        let mut d = two_rows();
        d[Distribution::SIZE + 8] = 1;
        assert!(!Distribution::valid_data_len(&d));
        assert!(Distribution::roster_row(&d, 0).is_err());
        assert!(Distribution::roster_find(&d, crate::HOLDER_KEY, &[7u8; 32]).is_err());
        // a body without the magic
        let mut d = two_rows();
        d[Distribution::SIZE] ^= 1;
        assert!(!Distribution::valid_data_len(&d));
        assert!(Distribution::roster_row(&d, 0).is_err());
        // a body shorter than its header
        let short = [0u8; Distribution::SIZE + 4];
        assert!(!Distribution::valid_data_len(&short));
        assert!(Distribution::roster_find(&short, crate::HOLDER_KEY, &[0u8; 32]).is_err());
    }
}
