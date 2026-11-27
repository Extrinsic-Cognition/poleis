//! Account state, laid out byte by byte, as Poleis lays its own: no macros, the
//! layout you read is the layout on chain. Every account starts with a
//! one-byte discriminator and a one-byte layout version; both, the owner and
//! the size are checked on every read. All integers little-endian. Offsets are
//! public so clients and tests read the same numbers the program writes.

use crate::error::RoadsError;
use crate::util::ms_of;
use poleis_futarchy::law::{Branch, Purse, Terms, AMOUNT_MAX};
use pinocchio::{
    account::{Ref, RefMut},
    error::ProgramError,
    AccountView, Address,
};

/// Bumped whenever any layout below changes shape.
pub const LAYOUT_VERSION: u8 = 1;

pub const DISC_BOARD: u8 = 1;
pub const DISC_BALANCE: u8 = 2;
pub const DISC_ROAD: u8 = 3;
pub const DISC_POSITION: u8 = 4;

pub const OFF_DISC: usize = 0;
pub const OFF_VERSION: usize = 1;

pub const SEED_BOARD: &[u8] = b"board";
pub const SEED_BALANCE: &[u8] = b"bal";
pub const SEED_ROAD: &[u8] = b"road";
pub const SEED_POSITION: &[u8] = b"pos";

/// verdict: a road open, passed or failed
pub const VERDICT_OPEN: u8 = 0;
pub const VERDICT_PASSED: u8 = 1;
pub const VERDICT_FAILED: u8 = 2;

#[inline(always)]
fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
#[inline(always)]
fn rd_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
#[inline(always)]
fn rd_u128(d: &[u8], o: usize) -> u128 {
    u128::from_le_bytes(d[o..o + 16].try_into().unwrap())
}
#[inline(always)]
fn rd_i128(d: &[u8], o: usize) -> i128 {
    i128::from_le_bytes(d[o..o + 16].try_into().unwrap())
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
fn wr_u32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_u64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_u128(d: &mut [u8], o: usize, v: u128) {
    d[o..o + 16].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_i128(d: &mut [u8], o: usize, v: i128) {
    d[o..o + 16].copy_from_slice(&v.to_le_bytes());
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
        return Err(RoadsError::InvalidAccount.into());
    }
    let data = acc.try_borrow()?;
    if data.len() != size || data[OFF_DISC] != disc {
        return Err(RoadsError::InvalidAccount.into());
    }
    if data[OFF_VERSION] != LAYOUT_VERSION {
        return Err(RoadsError::LayoutVersion.into());
    }
    Ok(data)
}

fn checked_mut<'a>(acc: &'a mut AccountView, disc: u8, size: usize) -> Result<RefMut<'a, [u8]>, ProgramError> {
    if !acc.owned_by(&crate::ID) {
        return Err(RoadsError::InvalidAccount.into());
    }
    let data = acc.try_borrow_mut()?;
    if data.len() != size || data[OFF_DISC] != disc {
        return Err(RoadsError::InvalidAccount.into());
    }
    if data[OFF_VERSION] != LAYOUT_VERSION {
        return Err(RoadsError::LayoutVersion.into());
    }
    Ok(data)
}

// ---- Board: [disc:1][ver:1][board_id:8][authority:32][crediter:32][judge:32][credited:8]
//             [polis:32][unit:8][step_slots:8][max_step_bp:4][threshold_bp:4][fee_bp:4][pool_base:8][pool_quote:8][judge_slots:8][earned:8][bump:1] ----
// A polis's own market place, seeded by its number. Its authority is named ONCE, when it is opened (the payer only pays), and nothing
// changes it after: it alone names the crediter and the judge (both start as the authority). So a board whose authority is a polis's
// agent key was always the polis's: its law, its roles and every credit on it came under the polis's own hand. `credited` is everything its
// crediter ever credited: THE BOOK's own total on chain (below, with `earned`).
//
// A ROAD IS LAID BY A RULE. The board keeps its own law for the roads laid by rule (`set_law`, the authority's signature): `polis`, the
// program whose proposals open roads here, `unit`, whose proposals, and the terms every such road takes (`lay_for`). A board whose `polis`
// is all zero lays nothing by rule. The law sits after `credited`, so every offset a reader read before stays where it stood; only `bump`
// moved. A road keeps the terms it was laid with: a later law changes only roads laid after it.
//
// A ROAD'S GAIN IS THE HOUSE'S. `earned` is what the house (the board's authority) has earned on this board: at a road's `collect` its
// staker gets back what is left up to its stake and no more, and the rest is added here. THE BOOK: every balance's `amount`, what is left in
// every road not collected, and `earned`, together, are `credited`. `earned` sits after the law, so no offset that stood moved. Nothing moves
// it off the board yet: where the house's take goes is the owner's open call (said, not built).
#[derive(Clone, Copy, Default)]
pub struct Law {
    pub polis: [u8; 32],
    pub unit: u64,
    pub step_slots: u64,
    pub max_step_bp: u32,
    pub threshold_bp: u32,
    pub fee_bp: u32,
    pub pool_base: u64,
    pub pool_quote: u64,
    pub judge_slots: u64,
}
impl Law {
    /// Whether this board lays roads by rule at all.
    pub fn stands(&self) -> bool {
        self.polis != [0u8; 32]
    }
}
pub struct Board {
    pub board_id: u64,
    pub authority: Address,
    pub crediter: Address,
    pub judge: Address,
    pub credited: u64,
    pub law: Law,
    pub earned: u64,
    pub bump: u8,
}
impl Board {
    pub const SIZE: usize = 207;
    pub const OFF_BOARD: usize = 2;
    pub const OFF_AUTHORITY: usize = 10;
    pub const OFF_CREDITER: usize = 42;
    pub const OFF_JUDGE: usize = 74;
    pub const OFF_CREDITED: usize = 106;
    pub const OFF_POLIS: usize = 114;
    pub const OFF_UNIT: usize = 146;
    pub const OFF_LAW_STEP: usize = 154;
    pub const OFF_LAW_MAX_STEP_BP: usize = 162;
    pub const OFF_LAW_THRESHOLD_BP: usize = 166;
    pub const OFF_LAW_FEE_BP: usize = 170;
    pub const OFF_LAW_POOL_BASE: usize = 174;
    pub const OFF_LAW_POOL_QUOTE: usize = 182;
    pub const OFF_LAW_JUDGE_SLOTS: usize = 190;
    pub const OFF_EARNED: usize = 198;
    pub const OFF_BUMP: usize = 206;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_BOARD, Self::SIZE)?;
        Ok(Self {
            board_id: rd_u64(&d, Self::OFF_BOARD),
            authority: rd_addr(&d, Self::OFF_AUTHORITY),
            crediter: rd_addr(&d, Self::OFF_CREDITER),
            judge: rd_addr(&d, Self::OFF_JUDGE),
            credited: rd_u64(&d, Self::OFF_CREDITED),
            law: Law {
                polis: rd_h32(&d, Self::OFF_POLIS),
                unit: rd_u64(&d, Self::OFF_UNIT),
                step_slots: rd_u64(&d, Self::OFF_LAW_STEP),
                max_step_bp: rd_u32(&d, Self::OFF_LAW_MAX_STEP_BP),
                threshold_bp: rd_u32(&d, Self::OFF_LAW_THRESHOLD_BP),
                fee_bp: rd_u32(&d, Self::OFF_LAW_FEE_BP),
                pool_base: rd_u64(&d, Self::OFF_LAW_POOL_BASE),
                pool_quote: rd_u64(&d, Self::OFF_LAW_POOL_QUOTE),
                judge_slots: rd_u64(&d, Self::OFF_LAW_JUDGE_SLOTS),
            },
            earned: rd_u64(&d, Self::OFF_EARNED),
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a board and check it is the PDA its own id and bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let b = Self::read(acc)?;
        let id = b.board_id.to_le_bytes();
        crate::util::expect_pda(acc, &[SEED_BOARD, &id], b.bump)?;
        Ok(b)
    }
    /// Write every field: on a freshly created account (stamped here) or one read and checked before.
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(RoadsError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_BOARD);
        wr_u64(&mut d, Self::OFF_BOARD, self.board_id);
        wr_addr(&mut d, Self::OFF_AUTHORITY, &self.authority);
        wr_addr(&mut d, Self::OFF_CREDITER, &self.crediter);
        wr_addr(&mut d, Self::OFF_JUDGE, &self.judge);
        wr_u64(&mut d, Self::OFF_CREDITED, self.credited);
        wr_b32(&mut d, Self::OFF_POLIS, &self.law.polis);
        wr_u64(&mut d, Self::OFF_UNIT, self.law.unit);
        wr_u64(&mut d, Self::OFF_LAW_STEP, self.law.step_slots);
        wr_u32(&mut d, Self::OFF_LAW_MAX_STEP_BP, self.law.max_step_bp);
        wr_u32(&mut d, Self::OFF_LAW_THRESHOLD_BP, self.law.threshold_bp);
        wr_u32(&mut d, Self::OFF_LAW_FEE_BP, self.law.fee_bp);
        wr_u64(&mut d, Self::OFF_LAW_POOL_BASE, self.law.pool_base);
        wr_u64(&mut d, Self::OFF_LAW_POOL_QUOTE, self.law.pool_quote);
        wr_u64(&mut d, Self::OFF_LAW_JUDGE_SLOTS, self.law.judge_slots);
        wr_u64(&mut d, Self::OFF_EARNED, self.earned);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ---- Balance: [disc:1][ver:1][board_id:8][owner:32][amount:8][in_roads:8][bump:1] ----
// An owner's play $WARM on one board: `amount` free to deposit, `in_roads` the
// deposits standing in roads.
pub struct Balance {
    pub board_id: u64,
    pub owner: Address,
    pub amount: u64,
    pub in_roads: u64,
    pub bump: u8,
}
impl Balance {
    pub const SIZE: usize = 59;
    pub const OFF_BOARD: usize = 2;
    pub const OFF_OWNER: usize = 10;
    pub const OFF_AMOUNT: usize = 42;
    pub const OFF_IN_ROADS: usize = 50;
    pub const OFF_BUMP: usize = 58;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_BALANCE, Self::SIZE)?;
        Ok(Self {
            board_id: rd_u64(&d, Self::OFF_BOARD),
            owner: rd_addr(&d, Self::OFF_OWNER),
            amount: rd_u64(&d, Self::OFF_AMOUNT),
            in_roads: rd_u64(&d, Self::OFF_IN_ROADS),
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a balance and check it is the PDA its own board id, owner and bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let b = Self::read(acc)?;
        let id = b.board_id.to_le_bytes();
        crate::util::expect_pda(acc, &[SEED_BALANCE, &id, b.owner.as_ref()], b.bump)?;
        Ok(b)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(RoadsError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_BALANCE);
        wr_u64(&mut d, Self::OFF_BOARD, self.board_id);
        wr_addr(&mut d, Self::OFF_OWNER, &self.owner);
        wr_u64(&mut d, Self::OFF_AMOUNT, self.amount);
        wr_u64(&mut d, Self::OFF_IN_ROADS, self.in_roads);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ---- Road ----
// THE PUBLIC FACE: the first 43 bytes, [disc:1][ver:1][board_id:8][road_id:32][verdict:1], are what Poleis's
// `seal` will read of a road. They must never move: a later field goes after them, and a change of their shape is a
// new program, not a new version. A reader checks the owner program, the discriminator and the VERSION first, and may
// then trust every offset of that version.
// [disc:1][ver:1][board_id:8][road_id:32][verdict:1][judge:32]
// [opens_slot:8][closes_slot:8][step_slots:8][max_step_bp:4][threshold_bp:4][pool_base:8][pool_quote:8][judge_by_slot:8][last_slot:8]
// [PASS: base:16 quote:16 obs:16 cum:16 acc:16 at:8 steps:8][FAIL: the same]
// [said:1][world:1][worth:8][pool_result:16][positions:4][deposits:8][given_back:8][staker:32][stake:8][collected:1]
// [fee_bp:4][PASS fees:16][FAIL fees:16][credited_at_lay:8][bump:1]
//
// THE FEE SITS AT THE END. `fee_bp` (the share of every trade kept in $WARM beside the pool: the law's THE FEE) and each branch's
// `fees` came after every field above was read by its offset (Poleis reads the verdict, the close and the deposits of a road where they
// stand): so they were laid after `collected`, and only `bump` moved. A branch's `fees` is the law's `Branch::fees`, kept apart from the
// branch's other fields for that reason. `lay` takes `fee_bp` beside `threshold_bp`; only the account keeps it here.
//
// THE BOARD AS IT STOOD AT THE LAY. `credited_at_lay` is what was in hands on the board when the road was laid, its `credited` less its
// `earned` (what the house earned back is in nobody's balance and can stand in no road): Poleis's `seal` measures its bar (how much of the
// board stood in the road) against it, so a credit after the lay, however large, cannot make a passed road stop counting.
// It sits at the end for the reason the fees do (every offset a reader reads stays where it stood); only `bump` moved again.
//
// THE CLOCK IS THE SLOT. A road's times are slots, the unit a reader compares with Poleis's windows. The law works in
// milliseconds: it is handed each slot times SLOT_MS, the chain's NOMINAL milliseconds a slot (400), not the wall's.
// So the branches' `at`, `cum` and `acc` are in those nominal milliseconds; the law's floor of a step (a second) is 3 slots,
// and its longest window (366 days) is 79,056,000 slots.
//
// WHOEVER ASKS THE MARKET STAKES ITS POOL. The pool is a purse of its STAKER of deposit `stake = pool_quote + pool_base`, taken from
// the staker's own balance at the lay (`lay`: the authority; `lay_for`: whoever lays it), never from a balance its owner did not sign
// for (`purse_of(stake)` is exactly (pool_quote, pool_base)). What is in the road is always `stake + deposits - given_back`; no settle
// gives back more than that, and `collect` gives the staker what is left up to its stake and the rest to the house (the board's `earned`):
// asking the market can cost the asker and never pays, and the house can gain and never loses. (The field was `house`, at the same offset.)
#[derive(Clone, Copy)]
pub struct Road {
    pub board_id: u64,
    pub road_id: [u8; 32],
    pub verdict: u8,
    pub judge: Address,
    pub opens_slot: u64,
    pub closes_slot: u64,
    pub step_slots: u64,
    pub max_step_bp: u32,
    pub threshold_bp: u32,
    pub pool_base: u64,
    pub pool_quote: u64,
    pub judge_by_slot: u64,
    pub last_slot: u64,
    pub pass: Branch,
    pub fail: Branch,
    pub said: u8,
    pub world: u8,
    pub worth: u64,
    pub pool_result: i128,
    pub positions: u32,
    pub deposits: u64,
    pub given_back: u64,
    pub staker: Address,
    pub stake: u64,
    pub collected: u8,
    pub fee_bp: u32,
    pub credited_at_lay: u64,
    pub bump: u8,
}
impl Road {
    pub const SIZE: usize = 463;
    // the public face: never moves
    pub const OFF_BOARD: usize = 2;
    pub const OFF_ROAD_ID: usize = 10;
    pub const OFF_VERDICT: usize = 42;
    // the rest
    pub const OFF_JUDGE: usize = 43;
    pub const OFF_OPENS: usize = 75;
    pub const OFF_CLOSES: usize = 83;
    pub const OFF_STEP: usize = 91;
    pub const OFF_MAX_STEP_BP: usize = 99;
    pub const OFF_THRESHOLD_BP: usize = 103;
    pub const OFF_POOL_BASE: usize = 107;
    pub const OFF_POOL_QUOTE: usize = 115;
    pub const OFF_JUDGE_BY: usize = 123;
    pub const OFF_LAST: usize = 131;
    pub const OFF_PASS: usize = 139;
    pub const OFF_FAIL: usize = 235;
    /// a branch's fields, from the branch's own offset
    pub const BR_BASE: usize = 0;
    pub const BR_QUOTE: usize = 16;
    pub const BR_OBS: usize = 32;
    pub const BR_CUM: usize = 48;
    pub const BR_ACC: usize = 64;
    pub const BR_AT: usize = 80;
    pub const BR_STEPS: usize = 88;
    pub const BRANCH_SIZE: usize = 96;
    pub const OFF_SAID: usize = 331;
    pub const OFF_WORLD: usize = 332;
    pub const OFF_WORTH: usize = 333;
    pub const OFF_POOL_RESULT: usize = 341;
    pub const OFF_POSITIONS: usize = 357;
    pub const OFF_DEPOSITS: usize = 361;
    pub const OFF_GIVEN_BACK: usize = 369;
    pub const OFF_STAKER: usize = 377;
    pub const OFF_STAKE: usize = 409;
    pub const OFF_COLLECTED: usize = 417;
    pub const OFF_FEE_BP: usize = 418;
    pub const OFF_PASS_FEES: usize = 422;
    pub const OFF_FAIL_FEES: usize = 438;
    pub const OFF_CREDITED_AT_LAY: usize = 454;
    pub const OFF_BUMP: usize = 462;

    /// a branch from its fields at `o` and its fees at `fees` (THE FEE SITS AT THE END)
    fn rd_branch(d: &[u8], o: usize, fees: usize) -> Branch {
        Branch {
            base: rd_u128(d, o + Self::BR_BASE),
            quote: rd_u128(d, o + Self::BR_QUOTE),
            obs: rd_u128(d, o + Self::BR_OBS),
            cum: rd_u128(d, o + Self::BR_CUM),
            acc: rd_u128(d, o + Self::BR_ACC),
            at: rd_u64(d, o + Self::BR_AT),
            steps: rd_u64(d, o + Self::BR_STEPS),
            fees: rd_u128(d, fees),
        }
    }
    fn wr_branch(d: &mut [u8], o: usize, fees: usize, b: &Branch) {
        wr_u128(d, fees, b.fees);
        wr_u128(d, o + Self::BR_BASE, b.base);
        wr_u128(d, o + Self::BR_QUOTE, b.quote);
        wr_u128(d, o + Self::BR_OBS, b.obs);
        wr_u128(d, o + Self::BR_CUM, b.cum);
        wr_u128(d, o + Self::BR_ACC, b.acc);
        wr_u64(d, o + Self::BR_AT, b.at);
        wr_u64(d, o + Self::BR_STEPS, b.steps);
    }
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_ROAD, Self::SIZE)?;
        Ok(Self {
            board_id: rd_u64(&d, Self::OFF_BOARD),
            road_id: rd_h32(&d, Self::OFF_ROAD_ID),
            verdict: d[Self::OFF_VERDICT],
            judge: rd_addr(&d, Self::OFF_JUDGE),
            opens_slot: rd_u64(&d, Self::OFF_OPENS),
            closes_slot: rd_u64(&d, Self::OFF_CLOSES),
            step_slots: rd_u64(&d, Self::OFF_STEP),
            max_step_bp: rd_u32(&d, Self::OFF_MAX_STEP_BP),
            threshold_bp: rd_u32(&d, Self::OFF_THRESHOLD_BP),
            pool_base: rd_u64(&d, Self::OFF_POOL_BASE),
            pool_quote: rd_u64(&d, Self::OFF_POOL_QUOTE),
            judge_by_slot: rd_u64(&d, Self::OFF_JUDGE_BY),
            last_slot: rd_u64(&d, Self::OFF_LAST),
            pass: Self::rd_branch(&d, Self::OFF_PASS, Self::OFF_PASS_FEES),
            fail: Self::rd_branch(&d, Self::OFF_FAIL, Self::OFF_FAIL_FEES),
            said: d[Self::OFF_SAID],
            world: d[Self::OFF_WORLD],
            worth: rd_u64(&d, Self::OFF_WORTH),
            pool_result: rd_i128(&d, Self::OFF_POOL_RESULT),
            positions: rd_u32(&d, Self::OFF_POSITIONS),
            deposits: rd_u64(&d, Self::OFF_DEPOSITS),
            given_back: rd_u64(&d, Self::OFF_GIVEN_BACK),
            staker: rd_addr(&d, Self::OFF_STAKER),
            stake: rd_u64(&d, Self::OFF_STAKE),
            collected: d[Self::OFF_COLLECTED],
            fee_bp: rd_u32(&d, Self::OFF_FEE_BP),
            credited_at_lay: rd_u64(&d, Self::OFF_CREDITED_AT_LAY),
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a road and check it is the PDA its own board id, road id and bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let r = Self::read(acc)?;
        let id = r.board_id.to_le_bytes();
        crate::util::expect_pda(acc, &[SEED_ROAD, &id, &r.road_id], r.bump)?;
        Ok(r)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(RoadsError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_ROAD);
        wr_u64(&mut d, Self::OFF_BOARD, self.board_id);
        wr_b32(&mut d, Self::OFF_ROAD_ID, &self.road_id);
        d[Self::OFF_VERDICT] = self.verdict;
        wr_addr(&mut d, Self::OFF_JUDGE, &self.judge);
        wr_u64(&mut d, Self::OFF_OPENS, self.opens_slot);
        wr_u64(&mut d, Self::OFF_CLOSES, self.closes_slot);
        wr_u64(&mut d, Self::OFF_STEP, self.step_slots);
        wr_u32(&mut d, Self::OFF_MAX_STEP_BP, self.max_step_bp);
        wr_u32(&mut d, Self::OFF_THRESHOLD_BP, self.threshold_bp);
        wr_u64(&mut d, Self::OFF_POOL_BASE, self.pool_base);
        wr_u64(&mut d, Self::OFF_POOL_QUOTE, self.pool_quote);
        wr_u64(&mut d, Self::OFF_JUDGE_BY, self.judge_by_slot);
        wr_u64(&mut d, Self::OFF_LAST, self.last_slot);
        Self::wr_branch(&mut d, Self::OFF_PASS, Self::OFF_PASS_FEES, &self.pass);
        Self::wr_branch(&mut d, Self::OFF_FAIL, Self::OFF_FAIL_FEES, &self.fail);
        d[Self::OFF_SAID] = self.said;
        d[Self::OFF_WORLD] = self.world;
        wr_u64(&mut d, Self::OFF_WORTH, self.worth);
        wr_i128(&mut d, Self::OFF_POOL_RESULT, self.pool_result);
        wr_u32(&mut d, Self::OFF_POSITIONS, self.positions);
        wr_u64(&mut d, Self::OFF_DEPOSITS, self.deposits);
        wr_u64(&mut d, Self::OFF_GIVEN_BACK, self.given_back);
        wr_addr(&mut d, Self::OFF_STAKER, &self.staker);
        wr_u64(&mut d, Self::OFF_STAKE, self.stake);
        d[Self::OFF_COLLECTED] = self.collected;
        wr_u32(&mut d, Self::OFF_FEE_BP, self.fee_bp);
        wr_u64(&mut d, Self::OFF_CREDITED_AT_LAY, self.credited_at_lay);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
    /// Write only the two branches and the stamp of the last trade: what `carry` and `trade` move.
    pub fn write_grid(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = checked_mut(acc, DISC_ROAD, Self::SIZE)?;
        wr_u64(&mut d, Self::OFF_LAST, self.last_slot);
        Self::wr_branch(&mut d, Self::OFF_PASS, Self::OFF_PASS_FEES, &self.pass);
        Self::wr_branch(&mut d, Self::OFF_FAIL, Self::OFF_FAIL_FEES, &self.fail);
        Ok(())
    }
    /// THE LAW'S TERMS OF A ROAD, its slots in the law's milliseconds. On a chain every road is both ways; `budget` is nobody's deposit
    /// (every purse brings its own): it only has to be a deposit the law can carry, and one as large as the pool is.
    pub fn terms(&self) -> Result<Terms, ProgramError> {
        terms_of(self.opens_slot, self.closes_slot, self.step_slots, self.max_step_bp, self.threshold_bp, self.fee_bp, self.pool_base, self.pool_quote)
    }
    /// What is in the road now: the staker's stake and the deposits, less what was given back (checked).
    pub fn left(&self) -> Result<u64, ProgramError> {
        self.stake
            .checked_add(self.deposits)
            .and_then(|x| x.checked_sub(self.given_back))
            .ok_or(RoadsError::MathOverflow.into())
    }
}

/// The law's terms from a road's fields, each slot times SLOT_MS (checked: `MathOverflow`). See `Road::terms`.
pub fn terms_of(opens_slot: u64, closes_slot: u64, step_slots: u64, max_step_bp: u32, threshold_bp: u32, fee_bp: u32, pool_base: u64, pool_quote: u64) -> Result<Terms, ProgramError> {
    let (pool_base, pool_quote) = (pool_base as u128, pool_quote as u128);
    Ok(Terms {
        opens_ms: ms_of(opens_slot)?,
        closes_ms: ms_of(closes_slot)?,
        step_ms: ms_of(step_slots)?,
        max_step_bp,
        threshold_bp,
        pool_base,
        pool_quote,
        budget: (pool_quote + pool_base).min(AMOUNT_MAX),
        both_ways: true,
        fee_bp,
    })
}

// ---- Position: [disc:1][ver:1][road:32][owner:32][deposit:8][pass quote:16][pass base:16][fail quote:16][fail base:16][trades:4][bump:1] ----
// One owner's purses on one road: one a road and owner.
pub struct Position {
    pub road: Address,
    pub owner: Address,
    pub deposit: u64,
    pub pass: Purse,
    pub fail: Purse,
    pub trades: u32,
    pub bump: u8,
}
impl Position {
    pub const SIZE: usize = 143;
    pub const OFF_ROAD: usize = 2;
    pub const OFF_OWNER: usize = 34;
    pub const OFF_DEPOSIT: usize = 66;
    pub const OFF_PASS_QUOTE: usize = 74;
    pub const OFF_PASS_BASE: usize = 90;
    pub const OFF_FAIL_QUOTE: usize = 106;
    pub const OFF_FAIL_BASE: usize = 122;
    pub const OFF_TRADES: usize = 138;
    pub const OFF_BUMP: usize = 142;
    pub fn read(acc: &AccountView) -> Result<Self, ProgramError> {
        let d = checked(acc, DISC_POSITION, Self::SIZE)?;
        Ok(Self {
            road: rd_addr(&d, Self::OFF_ROAD),
            owner: rd_addr(&d, Self::OFF_OWNER),
            deposit: rd_u64(&d, Self::OFF_DEPOSIT),
            pass: Purse { quote: rd_u128(&d, Self::OFF_PASS_QUOTE), base: rd_u128(&d, Self::OFF_PASS_BASE) },
            fail: Purse { quote: rd_u128(&d, Self::OFF_FAIL_QUOTE), base: rd_u128(&d, Self::OFF_FAIL_BASE) },
            trades: rd_u32(&d, Self::OFF_TRADES),
            bump: d[Self::OFF_BUMP],
        })
    }
    /// Read a position and check it is the PDA its own road, owner and bump say.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let p = Self::read(acc)?;
        crate::util::expect_pda(acc, &[SEED_POSITION, p.road.as_ref(), p.owner.as_ref()], p.bump)?;
        Ok(p)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = acc.try_borrow_mut()?;
        if d.len() != Self::SIZE {
            return Err(RoadsError::InvalidAccount.into());
        }
        stamp(&mut d, DISC_POSITION);
        wr_addr(&mut d, Self::OFF_ROAD, &self.road);
        wr_addr(&mut d, Self::OFF_OWNER, &self.owner);
        wr_u64(&mut d, Self::OFF_DEPOSIT, self.deposit);
        wr_u128(&mut d, Self::OFF_PASS_QUOTE, self.pass.quote);
        wr_u128(&mut d, Self::OFF_PASS_BASE, self.pass.base);
        wr_u128(&mut d, Self::OFF_FAIL_QUOTE, self.fail.quote);
        wr_u128(&mut d, Self::OFF_FAIL_BASE, self.fail.base);
        wr_u32(&mut d, Self::OFF_TRADES, self.trades);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// the offsets add up to the sizes, checked by the compiler
const _: () = assert!(Board::OFF_CREDITED == Board::OFF_JUDGE + 32 && Board::OFF_POLIS == Board::OFF_CREDITED + 8 && Board::OFF_UNIT == Board::OFF_POLIS + 32);
const _: () = assert!(Board::OFF_LAW_STEP == Board::OFF_UNIT + 8 && Board::OFF_LAW_MAX_STEP_BP == Board::OFF_LAW_STEP + 8 && Board::OFF_LAW_THRESHOLD_BP == Board::OFF_LAW_MAX_STEP_BP + 4);
const _: () = assert!(Board::OFF_LAW_FEE_BP == Board::OFF_LAW_THRESHOLD_BP + 4 && Board::OFF_LAW_POOL_BASE == Board::OFF_LAW_FEE_BP + 4 && Board::OFF_LAW_POOL_QUOTE == Board::OFF_LAW_POOL_BASE + 8);
const _: () = assert!(Board::OFF_LAW_JUDGE_SLOTS == Board::OFF_LAW_POOL_QUOTE + 8 && Board::OFF_EARNED == Board::OFF_LAW_JUDGE_SLOTS + 8);
const _: () = assert!(Board::OFF_BUMP == Board::OFF_EARNED + 8 && Board::OFF_BUMP + 1 == Board::SIZE);
const _: () = assert!(Road::BR_ACC == Road::BR_CUM + 16 && Road::BR_AT == Road::BR_ACC + 16 && Road::BR_STEPS == Road::BR_AT + 8 && Road::BRANCH_SIZE == Road::BR_STEPS + 8);
const _: () = assert!(Balance::OFF_BUMP + 1 == Balance::SIZE);
const _: () = assert!(Road::OFF_FAIL == Road::OFF_PASS + Road::BRANCH_SIZE && Road::OFF_SAID == Road::OFF_FAIL + Road::BRANCH_SIZE);
const _: () = assert!(Road::OFF_JUDGE == Road::OFF_VERDICT + 1 && Road::OFF_PASS == Road::OFF_LAST + 8);
const _: () = assert!(Road::OFF_STAKER == Road::OFF_GIVEN_BACK + 8 && Road::OFF_STAKE == Road::OFF_STAKER + 32 && Road::OFF_COLLECTED == Road::OFF_STAKE + 8);
const _: () = assert!(Road::OFF_FEE_BP == Road::OFF_COLLECTED + 1 && Road::OFF_PASS_FEES == Road::OFF_FEE_BP + 4 && Road::OFF_FAIL_FEES == Road::OFF_PASS_FEES + 16);
const _: () = assert!(Road::OFF_CREDITED_AT_LAY == Road::OFF_FAIL_FEES + 16 && Road::OFF_BUMP == Road::OFF_CREDITED_AT_LAY + 8 && Road::OFF_BUMP + 1 == Road::SIZE);
const _: () = assert!(Position::OFF_BUMP + 1 == Position::SIZE);
