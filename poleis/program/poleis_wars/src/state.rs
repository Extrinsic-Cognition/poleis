//! Account state, laid out byte by byte as poleis_roads lays its own: no macros, the layout you read is the layout on chain. Every account starts
//! with a one-byte discriminator and a one-byte layout version; both, the owner and the size are checked on every read. All integers
//! little-endian. Offsets are public so clients and tests read the same numbers the program writes.

use crate::error::WarsError;
use crate::util::u64_of;
use poleis_war::law::{Curve, How, Phase, Rules, Side};
use pinocchio::{
    account::{Ref, RefMut},
    error::ProgramError,
    AccountView, Address,
};

/// Bumped whenever any layout below changes shape. 2 since THE GAME IS ONE SWARM (Nov 24 2026): a config a generation naming its publisher's
/// treasury, a house its creator and what it is owed, the fee account what it paid; every account of version 1 (the first devnet war's)
/// is refused, its SOL kept by the chain by his word.
pub const LAYOUT_VERSION: u8 = 2;

pub const DISC_CONFIG: u8 = 1;
pub const DISC_FEES: u8 = 2;
pub const DISC_HOUSE: u8 = 3;
pub const DISC_POSITION: u8 = 4;
pub const DISC_RAID: u8 = 5;

pub const OFF_DISC: usize = 0;
pub const OFF_VERSION: usize = 1;

pub const SEED_CONFIG: &[u8] = b"config";
pub const SEED_FEES: &[u8] = b"fees";
pub const SEED_HOUSE: &[u8] = b"house";
pub const SEED_POSITION: &[u8] = b"pos";
pub const SEED_RAID: &[u8] = b"raid";
pub const SEED_MARCH: &[u8] = b"march";
pub const DISC_MARCH: u8 = 6;

/// A bounded FIFO permits every due order to be landed by one transaction. No allocator,
/// worker or account scan: at most eight owner/position pairs supplied in FIFO order.
pub const MARCH_CAPACITY: usize = poleis_war::law::MARCH_CAPACITY;
#[derive(Clone, Copy)]
pub struct Pending {
    pub owner: Address,
    pub side: u8,
    pub order: poleis_war::law::Order,
    pub brought: u64,
    pub sequence: u64,
}
impl Pending {
    const EMPTY: Self = Self {
        owner: Address::new_from_array([0; 32]), side: 0,
        order: poleis_war::law::Order { kind: poleis_war::law::OrderKind::Buy, amount: 0, minimum: 0, accepted_at: 0, lands_at: 0 },
        brought: 0, sequence: 0,
    };
}
pub struct March {
    pub raid: Address,
    pub next_sequence: u64,
    pub last_acceptance: i64,
    pub count: usize,
    pub bump: u8,
    pub orders: [Pending; MARCH_CAPACITY],
}
impl March {
    pub const OFF_COUNT: usize = 50;
    pub const OFF_ORDERS: usize = 52;
    pub const ORDER_SIZE: usize = 82;
    pub const SIZE: usize = Self::OFF_ORDERS + MARCH_CAPACITY * Self::ORDER_SIZE;
    pub fn empty(raid: Address, bump: u8) -> Self {
        Self { raid, next_sequence: 0, last_acceptance: i64::MIN, count: 0, bump, orders: [Pending::EMPTY; MARCH_CAPACITY] }
    }
    pub fn read(acc: &AccountView, raid: &Address) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_MARCH, Self::SIZE)?;
        let mut q = Self::empty(rd_addr(&d, 2), d[51]);
        if &q.raid != raid { return Err(WarsError::InvalidAccount.into()); }
        crate::util::expect_pda(acc, &[SEED_MARCH, raid.as_ref()], q.bump)?;
        q.next_sequence = rd_u64(&d, 34);
        q.last_acceptance = rd_i64(&d, 42);
        q.count = d[Self::OFF_COUNT] as usize;
        if q.count > MARCH_CAPACITY { return Err(WarsError::InvalidAccount.into()); }
        for (i, p) in q.orders[..q.count].iter_mut().enumerate() {
            let o = Self::OFF_ORDERS + i * Self::ORDER_SIZE;
            p.owner = rd_addr(&d, o);
            p.side = d[o + 32];
            if p.side > 1 { return Err(WarsError::InvalidAccount.into()); }
            p.order = poleis_war::law::Order {
                kind: match d[o + 33] { 0 => poleis_war::law::OrderKind::Buy, 1 => poleis_war::law::OrderKind::Sale, _ => return Err(WarsError::InvalidAccount.into()) },
                amount: rd_u64(&d, o + 34) as u128, minimum: rd_u64(&d, o + 42) as u128,
                accepted_at: rd_i64(&d, o + 50), lands_at: rd_i64(&d, o + 58),
            };
            p.brought = rd_u64(&d, o + 66);
            p.sequence = rd_u64(&d, o + 74);
        }
        Ok(q)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = stamped(acc, DISC_MARCH, Self::SIZE)?;
        wr_addr(&mut d, 2, &self.raid);
        wr_u64(&mut d, 34, self.next_sequence);
        wr_i64(&mut d, 42, self.last_acceptance);
        d[Self::OFF_COUNT] = self.count as u8;
        d[51] = self.bump;
        for (i, p) in self.orders[..self.count].iter().enumerate() {
            let o = Self::OFF_ORDERS + i * Self::ORDER_SIZE;
            wr_addr(&mut d, o, &p.owner);
            d[o + 32] = p.side;
            d[o + 33] = match p.order.kind { poleis_war::law::OrderKind::Buy => 0, poleis_war::law::OrderKind::Sale => 1 };
            wr_u64(&mut d, o + 34, u64_of(p.order.amount)?);
            wr_u64(&mut d, o + 42, u64_of(p.order.minimum)?);
            wr_i64(&mut d, o + 50, p.order.accepted_at);
            wr_i64(&mut d, o + 58, p.order.lands_at);
            wr_u64(&mut d, o + 66, p.brought);
            wr_u64(&mut d, o + 74, p.sequence);
        }
        d[Self::OFF_ORDERS + self.count * Self::ORDER_SIZE..].fill(0);
        Ok(())
    }
    pub fn push(&mut self, mut p: Pending) -> Result<(), ProgramError> {
        if self.count == MARCH_CAPACITY { return Err(WarsError::MarchFull.into()); }
        p.sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.checked_add(1).ok_or(WarsError::MathOverflow)?;
        self.last_acceptance = p.order.accepted_at;
        self.orders[self.count] = p;
        self.count += 1;
        Ok(())
    }
    pub fn pop(&mut self) {
        self.orders.copy_within(1..self.count, 0);
        self.count -= 1;
        self.orders[self.count] = Pending::EMPTY;
    }
}

/// a house trading as it likes, in a raid, or rekt (it lost one: its reserve and chest went to the winner, its holders convert)
pub const HOUSE_ACTIVE: u8 = 0;
pub const HOUSE_IN_RAID: u8 = 1;
pub const HOUSE_REKT: u8 = 2;

/// a raid proposed (its stake not yet at the need), open, in a last stand, decided, settled
pub const RAID_PROPOSED: u8 = 0;
pub const RAID_ACTIVE: u8 = 1;
pub const RAID_STAND: u8 = 2;
pub const RAID_DECIDED: u8 = 3;
pub const RAID_SETTLED: u8 = 4;

/// how a raid was decided, as stored: both met, the clock, the clock on a tie, a stand that failed, a revival
pub const HOW_BOTH_MET: u8 = 0;
pub const HOW_CLOCK: u8 = 1;
pub const HOW_CLOCK_TIE: u8 = 2;
pub const HOW_STAND_FAILED: u8 = 3;
pub const HOW_REVIVED: u8 = 4;

#[inline(always)]
fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
#[inline(always)]
fn rd_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
#[inline(always)]
fn rd_i64(d: &[u8], o: usize) -> i64 {
    i64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
#[inline(always)]
fn rd_u128(d: &[u8], o: usize) -> u128 {
    u128::from_le_bytes(d[o..o + 16].try_into().unwrap())
}
#[inline(always)]
fn rd_addr(d: &[u8], o: usize) -> Address {
    Address::new_from_array(d[o..o + 32].try_into().unwrap())
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
fn wr_i64(d: &mut [u8], o: usize, v: i64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_u128(d: &mut [u8], o: usize, v: u128) {
    d[o..o + 16].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_addr(d: &mut [u8], o: usize, v: &Address) {
    d[o..o + 32].copy_from_slice(v.as_ref());
}

fn checked<'a>(acc: &'a AccountView, disc: u8, size: usize) -> Result<Ref<'a, [u8]>, ProgramError> {
    if !acc.owned_by(&crate::ID) {
        return Err(WarsError::InvalidAccount.into());
    }
    let data = acc.try_borrow()?;
    if data.len() != size || data[OFF_DISC] != disc {
        return Err(WarsError::InvalidAccount.into());
    }
    if data[OFF_VERSION] != LAYOUT_VERSION {
        return Err(WarsError::LayoutVersion.into());
    }
    Ok(data)
}

/// the account's data for a write: of this program, of its size, stamped with its discriminator and the layout's version
fn stamped<'a>(acc: &'a mut AccountView, disc: u8, size: usize) -> Result<RefMut<'a, [u8]>, ProgramError> {
    if !acc.owned_by(&crate::ID) {
        return Err(WarsError::InvalidAccount.into());
    }
    let mut d = acc.try_borrow_mut()?;
    if d.len() != size {
        return Err(WarsError::InvalidAccount.into());
    }
    d[OFF_DISC] = disc;
    d[OFF_VERSION] = LAYOUT_VERSION;
    Ok(d)
}

// ---- Config: [disc:1][ver:1][authority:32][generation:8][treasury:32][fees:32][vt:8][vq:8][rt:8][protocol_bps:4][creator_bps:4]
//              [stake_own_pm:4][stake_gap_pm:4][stake_max_pm:4][opponent_min_pm:4][opponent_max_pm:4][raid_min_s:4][raid_max_s:4][raid_step_s:4]
//              [stand_minutes:4][revival_bps:4][staker_bonus_pm:4][keep_bps:4][dump_min_bps:4][dump_min_s:4][dump_max_s:4][dump_step_s:4]
//              [shield_age_s:8][shield_mcap:8][minute_seconds:4][bump:1] ----
// ONE GENERATION of the war's rules (THE GAME IS ONE SWARM, Nov 24 2026: his "who the fuck cares about parameters? they can be evolved"),
// never the only one: its PDA is ["config", authority, generation]. Its authority is whoever founds it (on poleis.us the game swarm's seal,
// signing by CPI from its sealed proposal); its rules stay fixed while it lives, and the next generation is founded beside it; a house is born
// into the generation it names. `treasury` is the publisher's: on poleis.us the game swarm's Poleis treasury, ["treasury", unit] of
// poleis_treasury, which every fee the fee account holds is pushed to (`pay_publisher`). The rules are the law's `Rules` and the program's
// three of its own: the shield (a house younger than `shield_age_s` seconds, or worth less than `shield_mcap` lamports, cannot be raided;
// the tests set both to zero) and the raid's minute in seconds. `fees` names the generation's fee account (its PDA ["fees", config]).
#[derive(Clone, Copy)]
pub struct Config {
    pub authority: Address,
    pub generation: u64,
    pub treasury: Address,
    pub fees: Address,
    pub rules: Rules,
    pub shield_age_s: u64,
    pub shield_mcap: u64,
    pub minute_seconds: u32,
    pub bump: u8,
}
impl Config {
    pub const SIZE: usize = 223;
    pub const MARCH_SIZE: usize = Self::SIZE + 4;
    pub const MARCH_LIMITS_SIZE: usize = Self::MARCH_SIZE + 8 + 1;
    pub const OFF_MARCH_S: usize = Self::SIZE;
    pub const OFF_MARCH_MIN_ORDER: usize = Self::MARCH_SIZE;
    pub const OFF_MARCH_MAX_PER_OWNER: usize = Self::OFF_MARCH_MIN_ORDER + 8;
    pub const OFF_AUTHORITY: usize = 2;
    pub const OFF_GENERATION: usize = 34;
    pub const OFF_TREASURY: usize = 42;
    pub const OFF_FEES: usize = 74;
    /// the rules, in the order `init_config` takes them: vt, vq, rt as u64, then the eighteen u32 of the law's `Rules` in its order
    pub const OFF_RULES: usize = 106;
    pub const OFF_SHIELD_AGE: usize = 202;
    pub const OFF_SHIELD_MCAP: usize = 210;
    pub const OFF_MINUTE_SECONDS: usize = 218;
    pub const OFF_BUMP: usize = 222;
    /// the rules' bytes: everything from the rules to the minute
    pub const DATA_LEN: usize = Self::OFF_BUMP - Self::OFF_RULES;
    /// the bytes `init_config` takes: the generation, the publisher's treasury, then the rules
    pub const INIT_LEN: usize = 8 + 32 + Self::DATA_LEN;

    /// the rules and the program's own three, read from `d` at `o` (the account at OFF_RULES, or `init_config`'s data at 0)
    pub fn parse(d: &[u8], o: usize) -> Result<(Rules, u64, u64, u32), ProgramError> {
        if d.len() < o + Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        let u = |i: usize| rd_u32(d, o + 24 + 4 * i);
        let rules = Rules {
            march_s: 0,
            march_min_order: 0,
            march_max_per_owner: 0,
            vt: rd_u64(d, o) as u128,
            vq: rd_u64(d, o + 8) as u128,
            rt: rd_u64(d, o + 16) as u128,
            protocol_bps: u(0),
            creator_bps: u(1),
            stake_own_pm: u(2),
            stake_gap_pm: u(3),
            stake_max_pm: u(4),
            opponent_min_pm: u(5),
            opponent_max_pm: u(6),
            raid_min_s: u(7),
            raid_max_s: u(8),
            raid_step_s: u(9),
            stand_minutes: u(10),
            revival_bps: u(11),
            staker_bonus_pm: u(12),
            keep_bps: u(13),
            dump_min_bps: u(14),
            dump_min_s: u(15),
            dump_max_s: u(16),
            dump_step_s: u(17),
        };
        let at = o + 24 + 4 * 18;
        Ok((rules, rd_u64(d, at), rd_u64(d, at + 8), rd_u32(d, at + 16)))
    }
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let size = acc.data_len();
        if !matches!(size, Self::SIZE | Self::MARCH_SIZE | Self::MARCH_LIMITS_SIZE) {
            return Err(WarsError::InvalidAccount.into());
        }
        let d = checked(acc, DISC_CONFIG, size)?;
        let (mut rules, shield_age_s, shield_mcap, minute_seconds) = Self::parse(&d, Self::OFF_RULES)?;
        if size >= Self::MARCH_SIZE { rules.march_s = rd_u32(&d, Self::OFF_MARCH_S); }
        if size == Self::MARCH_LIMITS_SIZE {
            rules.march_min_order = rd_u64(&d, Self::OFF_MARCH_MIN_ORDER);
            rules.march_max_per_owner = d[Self::OFF_MARCH_MAX_PER_OWNER];
        }
        Ok(Self {
            authority: rd_addr(&d, Self::OFF_AUTHORITY),
            generation: rd_u64(&d, Self::OFF_GENERATION),
            treasury: rd_addr(&d, Self::OFF_TREASURY),
            fees: rd_addr(&d, Self::OFF_FEES),
            rules,
            shield_age_s,
            shield_mcap,
            minute_seconds,
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a config and check it is the PDA its authority, its generation and its bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let c = Self::read(acc)?;
        let g = c.generation.to_le_bytes();
        crate::util::expect_pda(acc, &[SEED_CONFIG, c.authority.as_ref(), &g], c.bump)?;
        Ok(c)
    }
    /// the config a house names, read from the account at that address (its PDA was checked when the house was born)
    pub fn read_named(acc: &AccountView, named: &Address) -> Result<Self, ProgramError> {
        if acc.address() != named {
            return Err(WarsError::InvalidAccount.into());
        }
        Self::read(acc)
    }
    pub(crate) fn size_for(rules: &Rules) -> usize {
        if rules.march_s == 0 { Self::SIZE }
        else if rules.march_min_order == 0 && rules.march_max_per_owner == 0 { Self::MARCH_SIZE }
        else { Self::MARCH_LIMITS_SIZE }
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let size = Self::size_for(&self.rules);
        let mut d = stamped(acc, DISC_CONFIG, size)?;
        if size >= Self::MARCH_SIZE { wr_u32(&mut d, Self::OFF_MARCH_S, self.rules.march_s); }
        if size == Self::MARCH_LIMITS_SIZE {
            wr_u64(&mut d, Self::OFF_MARCH_MIN_ORDER, self.rules.march_min_order);
            d[Self::OFF_MARCH_MAX_PER_OWNER] = self.rules.march_max_per_owner;
        }
        wr_addr(&mut d, Self::OFF_AUTHORITY, &self.authority);
        wr_u64(&mut d, Self::OFF_GENERATION, self.generation);
        wr_addr(&mut d, Self::OFF_TREASURY, &self.treasury);
        wr_addr(&mut d, Self::OFF_FEES, &self.fees);
        let r = &self.rules;
        let o = Self::OFF_RULES;
        wr_u64(&mut d, o, u64_of(r.vt)?);
        wr_u64(&mut d, o + 8, u64_of(r.vq)?);
        wr_u64(&mut d, o + 16, u64_of(r.rt)?);
        let us = [
            r.protocol_bps,
            r.creator_bps,
            r.stake_own_pm,
            r.stake_gap_pm,
            r.stake_max_pm,
            r.opponent_min_pm,
            r.opponent_max_pm,
            r.raid_min_s,
            r.raid_max_s,
            r.raid_step_s,
            r.stand_minutes,
            r.revival_bps,
            r.staker_bonus_pm,
            r.keep_bps,
            r.dump_min_bps,
            r.dump_min_s,
            r.dump_max_s,
            r.dump_step_s,
        ];
        for (i, v) in us.iter().enumerate() {
            wr_u32(&mut d, o + 24 + 4 * i, *v);
        }
        wr_u64(&mut d, Self::OFF_SHIELD_AGE, self.shield_age_s);
        wr_u64(&mut d, Self::OFF_SHIELD_MCAP, self.shield_mcap);
        wr_u32(&mut d, Self::OFF_MINUTE_SECONDS, self.minute_seconds);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ---- Fees: [disc:1][ver:1][config:32][protocol:8][taxed:8][donated:8][paid:8][bump:1] ----
// The generation's fee account, the publisher's till: pump.fun's protocol fee of every trade and settlement buy, the fee half of every dump
// tax, and what its address held before its birth (`donated`); `paid` is everything `pay_publisher` has pushed from it to the generation's
// treasury. The creator's fee is no longer here: it stays in its house, for the house's creator (`House::creator_owed`). THE BOOK: its
// lamports are its rent and protocol + taxed + donated - paid.
pub struct Fees {
    pub config: Address,
    pub protocol: u64,
    pub taxed: u64,
    pub donated: u64,
    pub paid: u64,
    pub bump: u8,
}
impl Fees {
    pub const SIZE: usize = 67;
    pub const OFF_CONFIG: usize = 2;
    pub const OFF_PROTOCOL: usize = 34;
    pub const OFF_TAXED: usize = 42;
    pub const OFF_DONATED: usize = 50;
    pub const OFF_PAID: usize = 58;
    pub const OFF_BUMP: usize = 66;
    /// the fee account a config names
    pub fn read_named(acc: &AccountView, named: &Address) -> Result<Self, ProgramError> {
        if acc.address() != named {
            return Err(WarsError::InvalidAccount.into());
        }
        let d = checked(acc, DISC_FEES, Self::SIZE)?;
        Ok(Self {
            config: rd_addr(&d, Self::OFF_CONFIG),
            protocol: rd_u64(&d, Self::OFF_PROTOCOL),
            taxed: rd_u64(&d, Self::OFF_TAXED),
            donated: rd_u64(&d, Self::OFF_DONATED),
            paid: rd_u64(&d, Self::OFF_PAID),
            bump: d[Self::OFF_BUMP],
        })
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = stamped(acc, DISC_FEES, Self::SIZE)?;
        wr_addr(&mut d, Self::OFF_CONFIG, &self.config);
        wr_u64(&mut d, Self::OFF_PROTOCOL, self.protocol);
        wr_u64(&mut d, Self::OFF_TAXED, self.taxed);
        wr_u64(&mut d, Self::OFF_DONATED, self.donated);
        wr_u64(&mut d, Self::OFF_PAID, self.paid);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
    /// the publisher's fees of a trade booked (checked): the protocol's, and the dump tax's fee half
    pub fn book(&mut self, protocol: u128, taxed: u128) -> Result<(), ProgramError> {
        let add = |a: u64, b: u128| a.checked_add(u64_of(b)?).ok_or(ProgramError::from(WarsError::MathOverflow));
        self.protocol = add(self.protocol, protocol)?;
        self.taxed = add(self.taxed, taxed)?;
        Ok(())
    }
    /// every lamport it holds beyond its rent, by its book: protocol + taxed + donated - paid
    pub fn total(&self) -> Result<u64, ProgramError> {
        self.protocol
            .checked_add(self.taxed)
            .and_then(|x| x.checked_add(self.donated))
            .and_then(|x| x.checked_sub(self.paid))
            .ok_or(WarsError::MathOverflow.into())
    }
}

// ---- House: [disc:1][ver:1][config:32][house_id:8][status:1][vt:8][vq:8][rt:8][rq:8][rt0:8][chest:8][held:8][burned:8][born:8]
//             [raid:32][proposal:32][dump_rate_bps:4][dump_start:8][dump_end:8][creator:32][creator_owed:8][bump:1] ----
// A coin on its own curve. `held` is the coins its holders' positions hold, `burned` the coins of those who deserted or changed sides (the
// curve's `rt0 - rt` counts both, and the coins bought at a settlement until their holders convert). `raid` is the raid it is in (zero when
// none), `proposal` its own raid as an attacker while that is a proposal. The dump tax: its rate at `dump_start`, falling to nothing at
// `dump_end`. `creator` is whoever launched it (the signer of `init_house`: a player's wallet, an agent's key), and `creator_owed` the creator
// fee of every trade into it, booked here and never paid inside a trade (a wallet under rent would fail every trade on its house): anyone
// pushes it to the creator by `pay_creator`. THE BOOK: its lamports are its rent, its reserve `rq`, its chest and `creator_owed`, exactly.
#[derive(Clone, Copy)]
pub struct House {
    pub config: Address,
    pub house_id: u64,
    pub status: u8,
    pub vt: u64,
    pub vq: u64,
    pub rt: u64,
    pub rq: u64,
    pub rt0: u64,
    pub chest: u64,
    pub held: u64,
    pub burned: u64,
    pub born: i64,
    pub raid: Address,
    pub proposal: Address,
    pub dump_rate_bps: u32,
    pub dump_start: i64,
    pub dump_end: i64,
    pub creator: Address,
    pub creator_owed: u64,
    pub bump: u8,
}
impl House {
    pub const SIZE: usize = 240;
    pub const OFF_CONFIG: usize = 2;
    pub const OFF_HOUSE_ID: usize = 34;
    pub const OFF_STATUS: usize = 42;
    pub const OFF_VT: usize = 43;
    pub const OFF_VQ: usize = 51;
    pub const OFF_RT: usize = 59;
    pub const OFF_RQ: usize = 67;
    pub const OFF_RT0: usize = 75;
    pub const OFF_CHEST: usize = 83;
    pub const OFF_HELD: usize = 91;
    pub const OFF_BURNED: usize = 99;
    pub const OFF_BORN: usize = 107;
    pub const OFF_RAID: usize = 115;
    pub const OFF_PROPOSAL: usize = 147;
    pub const OFF_DUMP_RATE: usize = 179;
    pub const OFF_DUMP_START: usize = 183;
    pub const OFF_DUMP_END: usize = 191;
    pub const OFF_CREATOR: usize = 199;
    pub const OFF_CREATOR_OWED: usize = 231;
    pub const OFF_BUMP: usize = 239;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_HOUSE, Self::SIZE)?;
        Ok(Self {
            config: rd_addr(&d, Self::OFF_CONFIG),
            house_id: rd_u64(&d, Self::OFF_HOUSE_ID),
            status: d[Self::OFF_STATUS],
            vt: rd_u64(&d, Self::OFF_VT),
            vq: rd_u64(&d, Self::OFF_VQ),
            rt: rd_u64(&d, Self::OFF_RT),
            rq: rd_u64(&d, Self::OFF_RQ),
            rt0: rd_u64(&d, Self::OFF_RT0),
            chest: rd_u64(&d, Self::OFF_CHEST),
            held: rd_u64(&d, Self::OFF_HELD),
            burned: rd_u64(&d, Self::OFF_BURNED),
            born: rd_i64(&d, Self::OFF_BORN),
            raid: rd_addr(&d, Self::OFF_RAID),
            proposal: rd_addr(&d, Self::OFF_PROPOSAL),
            dump_rate_bps: rd_u32(&d, Self::OFF_DUMP_RATE),
            dump_start: rd_i64(&d, Self::OFF_DUMP_START),
            dump_end: rd_i64(&d, Self::OFF_DUMP_END),
            creator: rd_addr(&d, Self::OFF_CREATOR),
            creator_owed: rd_u64(&d, Self::OFF_CREATOR_OWED),
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a house and check it is the PDA its own config, id and bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let h = Self::read(acc)?;
        let id = h.house_id.to_le_bytes();
        crate::util::expect_pda(acc, &[SEED_HOUSE, h.config.as_ref(), &id], h.bump)?;
        Ok(h)
    }
    /// Read a house a raid names: at that address, so its PDA is the one the raid's own seeds were checked with.
    pub fn read_named(acc: &AccountView, named: &Address) -> Result<Self, ProgramError> {
        if acc.address() != named {
            return Err(WarsError::InvalidAccount.into());
        }
        Self::read(acc)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = stamped(acc, DISC_HOUSE, Self::SIZE)?;
        wr_addr(&mut d, Self::OFF_CONFIG, &self.config);
        wr_u64(&mut d, Self::OFF_HOUSE_ID, self.house_id);
        d[Self::OFF_STATUS] = self.status;
        wr_u64(&mut d, Self::OFF_VT, self.vt);
        wr_u64(&mut d, Self::OFF_VQ, self.vq);
        wr_u64(&mut d, Self::OFF_RT, self.rt);
        wr_u64(&mut d, Self::OFF_RQ, self.rq);
        wr_u64(&mut d, Self::OFF_RT0, self.rt0);
        wr_u64(&mut d, Self::OFF_CHEST, self.chest);
        wr_u64(&mut d, Self::OFF_HELD, self.held);
        wr_u64(&mut d, Self::OFF_BURNED, self.burned);
        wr_i64(&mut d, Self::OFF_BORN, self.born);
        wr_addr(&mut d, Self::OFF_RAID, &self.raid);
        wr_addr(&mut d, Self::OFF_PROPOSAL, &self.proposal);
        wr_u32(&mut d, Self::OFF_DUMP_RATE, self.dump_rate_bps);
        wr_i64(&mut d, Self::OFF_DUMP_START, self.dump_start);
        wr_i64(&mut d, Self::OFF_DUMP_END, self.dump_end);
        wr_addr(&mut d, Self::OFF_CREATOR, &self.creator);
        wr_u64(&mut d, Self::OFF_CREATOR_OWED, self.creator_owed);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
    /// the house's curve, as the law reads it
    pub fn curve(&self) -> Curve {
        Curve { vt: self.vt as u128, vq: self.vq as u128, rt: self.rt as u128, rq: self.rq as u128, rt0: self.rt0 as u128 }
    }
    /// the law's curve after a trade, stored (checked)
    pub fn set_curve(&mut self, c: &Curve) -> Result<(), ProgramError> {
        self.vt = u64_of(c.vt)?;
        self.vq = u64_of(c.vq)?;
        self.rt = u64_of(c.rt)?;
        self.rq = u64_of(c.rq)?;
        self.rt0 = u64_of(c.rt0)?;
        Ok(())
    }
    /// its pot: its reserve and its chest (what a settlement takes of a loser; what its creator is owed is not in it)
    pub fn holds(&self) -> Result<u64, ProgramError> {
        self.rq.checked_add(self.chest).ok_or(WarsError::MathOverflow.into())
    }
}

// ---- Position: [disc:1][ver:1][house:32][owner:32][coins:8][staked:8][brought:8][raid:32][bump:1] ----
// One owner's coins of one house: `staked` of them locked in `raid` (zero when none), and `brought`, every lamport he put into this house
// (what his buys paid, fees included; what his change of sides paid; carried over from the house he converted out of), of which his half is
// what a desert or a change of sides may carry.
// Marching quote proceeds, refunds and change are lamports above this account's
// rent. They are the owner's, claimable by IX_CLAIM_MARCH_QUOTE; conversion's
// existing close also returns them. No position bytes or legacy rent change.
pub struct Position {
    pub house: Address,
    pub owner: Address,
    pub coins: u64,
    pub staked: u64,
    pub brought: u64,
    pub raid: Address,
    pub bump: u8,
}
impl Position {
    pub const SIZE: usize = 123;
    pub const OFF_HOUSE: usize = 2;
    pub const OFF_OWNER: usize = 34;
    pub const OFF_COINS: usize = 66;
    pub const OFF_STAKED: usize = 74;
    pub const OFF_BROUGHT: usize = 82;
    pub const OFF_RAID: usize = 90;
    pub const OFF_BUMP: usize = 122;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_POSITION, Self::SIZE)?;
        Ok(Self {
            house: rd_addr(&d, Self::OFF_HOUSE),
            owner: rd_addr(&d, Self::OFF_OWNER),
            coins: rd_u64(&d, Self::OFF_COINS),
            staked: rd_u64(&d, Self::OFF_STAKED),
            brought: rd_u64(&d, Self::OFF_BROUGHT),
            raid: rd_addr(&d, Self::OFF_RAID),
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a position and check it is the PDA its own house, owner and bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let p = Self::read(acc)?;
        crate::util::expect_pda(acc, &[SEED_POSITION, p.house.as_ref(), p.owner.as_ref()], p.bump)?;
        Ok(p)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = stamped(acc, DISC_POSITION, Self::SIZE)?;
        wr_addr(&mut d, Self::OFF_HOUSE, &self.house);
        wr_addr(&mut d, Self::OFF_OWNER, &self.owner);
        wr_u64(&mut d, Self::OFF_COINS, self.coins);
        wr_u64(&mut d, Self::OFF_STAKED, self.staked);
        wr_u64(&mut d, Self::OFF_BROUGHT, self.brought);
        wr_addr(&mut d, Self::OFF_RAID, &self.raid);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
    /// the coins it holds free of a stake
    pub fn free(&self) -> u64 {
        self.coins.saturating_sub(self.staked)
    }
}

// ---- Raid: [disc:1][ver:1][config:32][attacker:32][defender:32][status:1][staked:8][stakers:4][m0_att:8][m0_def:8][need:8][start:8]
//            [minutes:4][seconds:8][minute_seconds:4][next_minute:4][stand_side:1][stand_met:4][stand_ends:4][winner:1][how:1][how_met:4]
//            [decided_at:4][mcap_end_att:8][mcap_end_def:8][pot_in:8][bought_with:8][coins_bought:8][to_chest:8][weights:16][converted:8]
//            [bump:1] ----
// One raid of an attacker on a defender, its PDA ["raid", attacker, defender]. A proposal gathers stakes (`staked` coins of `stakers`
// positions) until the law opens it; it then keeps both market caps at the opening, the need, its start on the clock, its length in the
// law's minutes (`minutes`) and in seconds (`seconds`, at its `minute_seconds`), and the first minute not yet read (`next_minute`). A stand:
// the side in it (0 the attacker, 1 the defender), the minute the other beat its line, the stand's last minute. Decided: the winner, how
// (HOW_*), the minute the line was met for a stand that failed, the minute it was decided. The settlement's numbers: both market caps at the
// end, the loser's pot, what of it bought, the coins bought, what went to the winner's chest, the weights of the loser's holders and the coins
// handed out by their converts so far.
pub struct Raid {
    pub config: Address,
    pub attacker: Address,
    pub defender: Address,
    pub status: u8,
    pub staked: u64,
    pub stakers: u32,
    pub m0_att: u64,
    pub m0_def: u64,
    pub need: u64,
    pub start: i64,
    pub minutes: u32,
    pub seconds: u64,
    pub minute_seconds: u32,
    pub next_minute: u32,
    pub stand_side: u8,
    pub stand_met: u32,
    pub stand_ends: u32,
    pub winner: u8,
    pub how: u8,
    pub how_met: u32,
    pub decided_at: u32,
    pub mcap_end_att: u64,
    pub mcap_end_def: u64,
    pub pot_in: u64,
    pub bought_with: u64,
    pub coins_bought: u64,
    pub to_chest: u64,
    pub weights: u128,
    pub converted: u64,
    pub bump: u8,
}
impl Raid {
    pub const SIZE: usize = 255;
    pub const OFF_CONFIG: usize = 2;
    pub const OFF_ATTACKER: usize = 34;
    pub const OFF_DEFENDER: usize = 66;
    pub const OFF_STATUS: usize = 98;
    pub const OFF_STAKED: usize = 99;
    pub const OFF_STAKERS: usize = 107;
    pub const OFF_M0_ATT: usize = 111;
    pub const OFF_M0_DEF: usize = 119;
    pub const OFF_NEED: usize = 127;
    pub const OFF_START: usize = 135;
    pub const OFF_MINUTES: usize = 143;
    pub const OFF_SECONDS: usize = 147;
    pub const OFF_MINUTE_SECONDS: usize = 155;
    pub const OFF_NEXT_MINUTE: usize = 159;
    pub const OFF_STAND_SIDE: usize = 163;
    pub const OFF_STAND_MET: usize = 164;
    pub const OFF_STAND_ENDS: usize = 168;
    pub const OFF_WINNER: usize = 172;
    pub const OFF_HOW: usize = 173;
    pub const OFF_HOW_MET: usize = 174;
    pub const OFF_DECIDED_AT: usize = 178;
    pub const OFF_MCAP_END_ATT: usize = 182;
    pub const OFF_MCAP_END_DEF: usize = 190;
    pub const OFF_POT_IN: usize = 198;
    pub const OFF_BOUGHT_WITH: usize = 206;
    pub const OFF_COINS_BOUGHT: usize = 214;
    pub const OFF_TO_CHEST: usize = 222;
    pub const OFF_WEIGHTS: usize = 230;
    pub const OFF_CONVERTED: usize = 246;
    pub const OFF_BUMP: usize = 254;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_RAID, Self::SIZE)?;
        Ok(Self {
            config: rd_addr(&d, Self::OFF_CONFIG),
            attacker: rd_addr(&d, Self::OFF_ATTACKER),
            defender: rd_addr(&d, Self::OFF_DEFENDER),
            status: d[Self::OFF_STATUS],
            staked: rd_u64(&d, Self::OFF_STAKED),
            stakers: rd_u32(&d, Self::OFF_STAKERS),
            m0_att: rd_u64(&d, Self::OFF_M0_ATT),
            m0_def: rd_u64(&d, Self::OFF_M0_DEF),
            need: rd_u64(&d, Self::OFF_NEED),
            start: rd_i64(&d, Self::OFF_START),
            minutes: rd_u32(&d, Self::OFF_MINUTES),
            seconds: rd_u64(&d, Self::OFF_SECONDS),
            minute_seconds: rd_u32(&d, Self::OFF_MINUTE_SECONDS),
            next_minute: rd_u32(&d, Self::OFF_NEXT_MINUTE),
            stand_side: d[Self::OFF_STAND_SIDE],
            stand_met: rd_u32(&d, Self::OFF_STAND_MET),
            stand_ends: rd_u32(&d, Self::OFF_STAND_ENDS),
            winner: d[Self::OFF_WINNER],
            how: d[Self::OFF_HOW],
            how_met: rd_u32(&d, Self::OFF_HOW_MET),
            decided_at: rd_u32(&d, Self::OFF_DECIDED_AT),
            mcap_end_att: rd_u64(&d, Self::OFF_MCAP_END_ATT),
            mcap_end_def: rd_u64(&d, Self::OFF_MCAP_END_DEF),
            pot_in: rd_u64(&d, Self::OFF_POT_IN),
            bought_with: rd_u64(&d, Self::OFF_BOUGHT_WITH),
            coins_bought: rd_u64(&d, Self::OFF_COINS_BOUGHT),
            to_chest: rd_u64(&d, Self::OFF_TO_CHEST),
            weights: rd_u128(&d, Self::OFF_WEIGHTS),
            converted: rd_u64(&d, Self::OFF_CONVERTED),
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a raid and check it is the PDA its own attacker, defender and bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let r = Self::read(acc)?;
        crate::util::expect_pda(acc, &[SEED_RAID, r.attacker.as_ref(), r.defender.as_ref()], r.bump)?;
        Ok(r)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = stamped(acc, DISC_RAID, Self::SIZE)?;
        wr_addr(&mut d, Self::OFF_CONFIG, &self.config);
        wr_addr(&mut d, Self::OFF_ATTACKER, &self.attacker);
        wr_addr(&mut d, Self::OFF_DEFENDER, &self.defender);
        d[Self::OFF_STATUS] = self.status;
        wr_u64(&mut d, Self::OFF_STAKED, self.staked);
        wr_u32(&mut d, Self::OFF_STAKERS, self.stakers);
        wr_u64(&mut d, Self::OFF_M0_ATT, self.m0_att);
        wr_u64(&mut d, Self::OFF_M0_DEF, self.m0_def);
        wr_u64(&mut d, Self::OFF_NEED, self.need);
        wr_i64(&mut d, Self::OFF_START, self.start);
        wr_u32(&mut d, Self::OFF_MINUTES, self.minutes);
        wr_u64(&mut d, Self::OFF_SECONDS, self.seconds);
        wr_u32(&mut d, Self::OFF_MINUTE_SECONDS, self.minute_seconds);
        wr_u32(&mut d, Self::OFF_NEXT_MINUTE, self.next_minute);
        d[Self::OFF_STAND_SIDE] = self.stand_side;
        wr_u32(&mut d, Self::OFF_STAND_MET, self.stand_met);
        wr_u32(&mut d, Self::OFF_STAND_ENDS, self.stand_ends);
        d[Self::OFF_WINNER] = self.winner;
        d[Self::OFF_HOW] = self.how;
        wr_u32(&mut d, Self::OFF_HOW_MET, self.how_met);
        wr_u32(&mut d, Self::OFF_DECIDED_AT, self.decided_at);
        wr_u64(&mut d, Self::OFF_MCAP_END_ATT, self.mcap_end_att);
        wr_u64(&mut d, Self::OFF_MCAP_END_DEF, self.mcap_end_def);
        wr_u64(&mut d, Self::OFF_POT_IN, self.pot_in);
        wr_u64(&mut d, Self::OFF_BOUGHT_WITH, self.bought_with);
        wr_u64(&mut d, Self::OFF_COINS_BOUGHT, self.coins_bought);
        wr_u64(&mut d, Self::OFF_TO_CHEST, self.to_chest);
        wr_u128(&mut d, Self::OFF_WEIGHTS, self.weights);
        wr_u64(&mut d, Self::OFF_CONVERTED, self.converted);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
    /// the raid's phase, as the law reads it (an open raid, its stand, its decision); None for a proposal
    pub fn phase(&self) -> Result<Option<Phase>, ProgramError> {
        let side = |b: u8| Side::of(b).ok_or(ProgramError::from(WarsError::InvalidAccount));
        Ok(match self.status {
            RAID_PROPOSED => None,
            RAID_ACTIVE => Some(Phase::Open),
            RAID_STAND => Some(Phase::Stand { side: side(self.stand_side)?, met: self.stand_met, ends: self.stand_ends }),
            RAID_DECIDED | RAID_SETTLED => {
                let how = match self.how {
                    HOW_BOTH_MET => How::BothMet,
                    HOW_CLOCK => How::Clock { tie: false },
                    HOW_CLOCK_TIE => How::Clock { tie: true },
                    HOW_STAND_FAILED => How::StandFailed { met: self.how_met },
                    HOW_REVIVED => How::Revived,
                    _ => return Err(WarsError::InvalidAccount.into()),
                };
                Some(Phase::Decided { winner: side(self.winner)?, how, at: self.decided_at })
            }
            _ => return Err(WarsError::InvalidAccount.into()),
        })
    }
    /// the law's phase, stored
    pub fn set_phase(&mut self, p: Phase) {
        match p {
            Phase::Open => self.status = RAID_ACTIVE,
            Phase::Stand { side, met, ends } => {
                self.status = RAID_STAND;
                self.stand_side = side.index() as u8;
                self.stand_met = met;
                self.stand_ends = ends;
            }
            Phase::Decided { winner, how, at } => {
                self.status = RAID_DECIDED;
                self.winner = winner.index() as u8;
                self.decided_at = at;
                (self.how, self.how_met) = match how {
                    How::BothMet => (HOW_BOTH_MET, 0),
                    How::Clock { tie: false } => (HOW_CLOCK, 0),
                    How::Clock { tie: true } => (HOW_CLOCK_TIE, 0),
                    How::StandFailed { met } => (HOW_STAND_FAILED, met),
                    How::Revived => (HOW_REVIVED, 0),
                };
            }
        }
    }
    /// the address of a side's house
    pub fn house_of(&self, s: Side) -> &Address {
        match s {
            Side::Attacker => &self.attacker,
            Side::Defender => &self.defender,
        }
    }
    /// the raid's minute at `now`: (now - start) / minute_seconds, none before its start
    pub fn minute_at(&self, now: i64) -> u32 {
        let secs = now.saturating_sub(self.start).max(0) as u64;
        u32::try_from(secs / (self.minute_seconds.max(1) as u64)).unwrap_or(u32::MAX)
    }
}

// the offsets add up to the sizes, checked by the compiler
const _: () = assert!(Config::OFF_GENERATION == Config::OFF_AUTHORITY + 32 && Config::OFF_TREASURY == Config::OFF_GENERATION + 8 && Config::OFF_FEES == Config::OFF_TREASURY + 32);
const _: () = assert!(Config::OFF_RULES == Config::OFF_FEES + 32 && Config::OFF_SHIELD_AGE == Config::OFF_RULES + 24 + 4 * 18);
const _: () = assert!(Config::OFF_MINUTE_SECONDS == Config::OFF_SHIELD_MCAP + 8 && Config::OFF_BUMP == Config::OFF_MINUTE_SECONDS + 4 && Config::OFF_BUMP + 1 == Config::SIZE);
const _: () = assert!(Config::DATA_LEN == 116);
const _: () = assert!(Fees::OFF_TAXED == Fees::OFF_PROTOCOL + 8 && Fees::OFF_DONATED == Fees::OFF_TAXED + 8 && Fees::OFF_PAID == Fees::OFF_DONATED + 8);
const _: () = assert!(Fees::OFF_BUMP == Fees::OFF_PAID + 8 && Fees::OFF_BUMP + 1 == Fees::SIZE);
const _: () = assert!(House::OFF_VT == House::OFF_STATUS + 1 && House::OFF_RT0 == House::OFF_RQ + 8 && House::OFF_CHEST == House::OFF_RT0 + 8);
const _: () = assert!(House::OFF_RAID == House::OFF_BORN + 8 && House::OFF_PROPOSAL == House::OFF_RAID + 32 && House::OFF_DUMP_RATE == House::OFF_PROPOSAL + 32);
const _: () = assert!(House::OFF_DUMP_END == House::OFF_DUMP_START + 8 && House::OFF_CREATOR == House::OFF_DUMP_END + 8 && House::OFF_CREATOR_OWED == House::OFF_CREATOR + 32);
const _: () = assert!(House::OFF_BUMP == House::OFF_CREATOR_OWED + 8 && House::OFF_BUMP + 1 == House::SIZE);
const _: () = assert!(Position::OFF_RAID == Position::OFF_BROUGHT + 8 && Position::OFF_BUMP == Position::OFF_RAID + 32 && Position::OFF_BUMP + 1 == Position::SIZE);
const _: () = assert!(Raid::OFF_STATUS == Raid::OFF_DEFENDER + 32 && Raid::OFF_START == Raid::OFF_NEED + 8 && Raid::OFF_NEXT_MINUTE == Raid::OFF_MINUTE_SECONDS + 4);
const _: () = assert!(Raid::OFF_WINNER == Raid::OFF_STAND_ENDS + 4 && Raid::OFF_MCAP_END_ATT == Raid::OFF_DECIDED_AT + 4 && Raid::OFF_WEIGHTS == Raid::OFF_TO_CHEST + 8);
const _: () = assert!(Raid::OFF_CONVERTED == Raid::OFF_WEIGHTS + 16 && Raid::OFF_BUMP == Raid::OFF_CONVERTED + 8 && Raid::OFF_BUMP + 1 == Raid::SIZE);
