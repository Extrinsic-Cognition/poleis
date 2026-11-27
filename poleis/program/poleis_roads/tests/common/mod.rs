//! What the tests share: the world on Mollusk, the instruction builders and the account readers. The readers read the program's own offsets
//! (poleis_roads::state), so a test reads the bytes the program writes; nothing here is in the program.
#![allow(dead_code)]

use crate::built;
use embertide_futarchy::law::{Branch, Purse, Terms};
use mollusk_svm::{program::keyed_account_for_system_program, result::ProgramResult, Mollusk};
use poleis_roads::error::RoadsError;
use poleis_roads::state::{terms_of, Balance, Board, Position, Road, SEED_BALANCE, SEED_BOARD, SEED_POSITION, SEED_ROAD};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(poleis_roads::ID.to_bytes());
pub const SOL: u64 = 1_000_000_000;

pub fn system_id() -> Pubkey {
    keyed_account_for_system_program().0
}

// ---- PDAs ----
pub fn board_pda(board_id: u64) -> Pubkey {
    Pubkey::find_program_address(&[SEED_BOARD, &board_id.to_le_bytes()], &PROGRAM_ID).0
}
pub fn balance_pda(board_id: u64, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[SEED_BALANCE, &board_id.to_le_bytes(), owner.as_ref()], &PROGRAM_ID).0
}
pub fn road_pda(board_id: u64, road_id: &[u8; 32]) -> Pubkey {
    Pubkey::find_program_address(&[SEED_ROAD, &board_id.to_le_bytes(), road_id], &PROGRAM_ID).0
}
pub fn position_pda(road: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[SEED_POSITION, road.as_ref(), owner.as_ref()], &PROGRAM_ID).0
}

// ---- builders ----
fn ix(tag: u8, data: &[u8], metas: Vec<AccountMeta>) -> Instruction {
    let mut d = vec![tag];
    d.extend_from_slice(data);
    Instruction::new_with_bytes(PROGRAM_ID, &d, metas)
}
/// accounts: [payer (s,w), board (w), system] · the authority is named; the payer only pays
pub fn init_board(payer: Pubkey, board_id: u64, authority: Pubkey) -> Instruction {
    let mut d = board_id.to_le_bytes().to_vec();
    d.extend_from_slice(authority.as_ref());
    ix(poleis_roads::IX_INIT_BOARD, &d, vec![AccountMeta::new(payer, true), AccountMeta::new(board_pda(board_id), false), AccountMeta::new_readonly(system_id(), false)])
}
/// accounts: [authority (s), board (w)] · data [crediter:32][judge:32]: a board's authority is named once, at `init_board`
pub fn set_board(signer: Pubkey, board_id: u64, crediter: Pubkey, judge: Pubkey) -> Instruction {
    let mut d = crediter.as_ref().to_vec();
    d.extend_from_slice(judge.as_ref());
    ix(poleis_roads::IX_SET_BOARD, &d, vec![AccountMeta::new_readonly(signer, true), AccountMeta::new(board_pda(board_id), false)])
}
/// A board's law for the roads laid by rule, as `set_law` takes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleLaw {
    pub polis: Pubkey,
    pub unit: u64,
    pub step_slots: u64,
    pub max_step_bp: u32,
    pub threshold_bp: u32,
    pub fee_bp: u32,
    pub pool_base: u64,
    pub pool_quote: u64,
    pub judge_slots: u64,
}
impl RuleLaw {
    /// the stake of every road laid by this law: what its staker puts in its pool
    pub fn stake(&self) -> u64 {
        self.pool_base + self.pool_quote
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut d = self.polis.as_ref().to_vec();
        d.extend_from_slice(&self.unit.to_le_bytes());
        d.extend_from_slice(&self.step_slots.to_le_bytes());
        for v in [self.max_step_bp, self.threshold_bp, self.fee_bp] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        for v in [self.pool_base, self.pool_quote, self.judge_slots] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        d
    }
}
/// accounts: [authority (s), board (w)]
pub fn set_law(signer: Pubkey, board_id: u64, law: &RuleLaw) -> Instruction {
    ix(poleis_roads::IX_SET_LAW, &law.bytes(), vec![AccountMeta::new_readonly(signer, true), AccountMeta::new(board_pda(board_id), false)])
}
/// accounts: [layer (s,w), board, road (w), the layer's own balance (w), proposal, session, system]
pub fn lay_for(layer: Pubkey, board_id: u64, road_id: &[u8; 32], proposal: Pubkey, session: Pubkey) -> Instruction {
    ix(
        poleis_roads::IX_LAY_FOR,
        &[],
        vec![
            AccountMeta::new(layer, true),
            AccountMeta::new_readonly(board_pda(board_id), false),
            AccountMeta::new(road_pda(board_id, road_id), false),
            AccountMeta::new(balance_pda(board_id, &layer), false),
            AccountMeta::new_readonly(proposal, false),
            AccountMeta::new_readonly(session, false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// A proposal of the polis's program as its bytes stand (Poleis's layout version 8, as `polis.rs` reads it): the hash, the class byte, the
/// origin and the opening slot; `len` its length (a live proposal is long, a closed one shrinks to a 34-byte tombstone).
pub fn polis_proposal(hash: &[u8; 32], class: u8, origin: u64, opened: u64, len: usize) -> Vec<u8> {
    use poleis_roads::polis::*;
    let mut d = vec![0u8; len.max(2)];
    d[0] = DISC_PROPOSAL;
    d[1] = LAYOUT_VERSION;
    let put = |d: &mut Vec<u8>, o: usize, b: &[u8]| {
        if d.len() >= o + b.len() {
            d[o..o + b.len()].copy_from_slice(b);
        }
    };
    put(&mut d, PROPOSAL_OFF_HASH, hash);
    put(&mut d, PROPOSAL_OFF_CLASS, &[class]);
    put(&mut d, PROPOSAL_OFF_ORIGIN, &origin.to_le_bytes());
    put(&mut d, PROPOSAL_OFF_OPENED, &opened.to_le_bytes());
    d
}
/// A session of the polis's program: its hash, unit, deadline and state.
pub fn polis_session(hash: &[u8; 32], unit: u64, deadline: u64, state: u8) -> Vec<u8> {
    use poleis_roads::polis::*;
    let mut d = vec![0u8; 200];
    d[0] = DISC_SESSION;
    d[1] = LAYOUT_VERSION;
    d[SESSION_OFF_HASH..SESSION_OFF_HASH + 32].copy_from_slice(hash);
    d[SESSION_OFF_UNIT..SESSION_OFF_UNIT + 8].copy_from_slice(&unit.to_le_bytes());
    d[SESSION_OFF_DEADLINE..SESSION_OFF_DEADLINE + 8].copy_from_slice(&deadline.to_le_bytes());
    d[SESSION_OFF_STATE] = state;
    d
}
/// The board's law as it stands: what `set_law` wrote.
pub fn law_view(d: &[u8]) -> RuleLaw {
    assert_eq!((d.len(), d[0], d[1]), (Board::SIZE, poleis_roads::state::DISC_BOARD, poleis_roads::state::LAYOUT_VERSION));
    RuleLaw {
        polis: key_at(d, Board::OFF_POLIS),
        unit: u64_at(d, Board::OFF_UNIT),
        step_slots: u64_at(d, Board::OFF_LAW_STEP),
        max_step_bp: u32_at(d, Board::OFF_LAW_MAX_STEP_BP),
        threshold_bp: u32_at(d, Board::OFF_LAW_THRESHOLD_BP),
        fee_bp: u32_at(d, Board::OFF_LAW_FEE_BP),
        pool_base: u64_at(d, Board::OFF_LAW_POOL_BASE),
        pool_quote: u64_at(d, Board::OFF_LAW_POOL_QUOTE),
        judge_slots: u64_at(d, Board::OFF_LAW_JUDGE_SLOTS),
    }
}
pub fn credit(crediter: Pubkey, board_id: u64, owner: Pubkey, amount: u64) -> Instruction {
    ix(
        poleis_roads::IX_CREDIT,
        &amount.to_le_bytes(),
        vec![
            AccountMeta::new(crediter, true),
            AccountMeta::new(board_pda(board_id), false),
            AccountMeta::new_readonly(owner, false),
            AccountMeta::new(balance_pda(board_id, &owner), false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}

/// A road's terms as `lay` takes them: its times in slots.
#[derive(Clone, Copy, Debug)]
pub struct Lay {
    pub road_id: [u8; 32],
    pub opens_slot: u64,
    pub closes_slot: u64,
    pub step_slots: u64,
    pub max_step_bp: u32,
    pub threshold_bp: u32,
    /// the share of every trade's input the pool keeps (the law's THE FEE)
    pub fee_bp: u32,
    pub pool_base: u64,
    pub pool_quote: u64,
    pub judge_by_slot: u64,
}
impl Lay {
    /// the law's terms of the road, in milliseconds (each slot times SLOT_MS)
    pub fn terms(&self) -> Terms {
        terms_of(self.opens_slot, self.closes_slot, self.step_slots, self.max_step_bp, self.threshold_bp, self.fee_bp, self.pool_base, self.pool_quote).unwrap()
    }
    /// the house's stake
    pub fn stake(&self) -> u64 {
        self.pool_base + self.pool_quote
    }
}
/// accounts: [authority (s,w), board, road (w), the authority's balance (w), system]
pub fn lay(authority: Pubkey, board_id: u64, l: &Lay) -> Instruction {
    let mut d = l.road_id.to_vec();
    for v in [l.opens_slot, l.closes_slot, l.step_slots] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    d.extend_from_slice(&l.max_step_bp.to_le_bytes());
    d.extend_from_slice(&l.threshold_bp.to_le_bytes());
    d.extend_from_slice(&l.fee_bp.to_le_bytes());
    for v in [l.pool_base, l.pool_quote, l.judge_by_slot] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    ix(
        poleis_roads::IX_LAY,
        &d,
        vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(board_pda(board_id), false),
            AccountMeta::new(road_pda(board_id, &l.road_id), false),
            AccountMeta::new(balance_pda(board_id, &authority), false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
pub fn deposit(owner: Pubkey, board_id: u64, road: Pubkey, amount: u64) -> Instruction {
    ix(
        poleis_roads::IX_DEPOSIT,
        &amount.to_le_bytes(),
        vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(road, false),
            AccountMeta::new(balance_pda(board_id, &owner), false),
            AccountMeta::new(position_pda(&road, &owner), false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// branch 0 pass, 1 fail; side 0 buy, 1 sell
pub fn trade(owner: Pubkey, road: Pubkey, branch: u8, side: u8, amount: u64, min_out: u64) -> Instruction {
    let mut d = vec![branch, side];
    d.extend_from_slice(&amount.to_le_bytes());
    d.extend_from_slice(&min_out.to_le_bytes());
    ix(poleis_roads::IX_TRADE, &d, vec![AccountMeta::new_readonly(owner, true), AccountMeta::new(road, false), AccountMeta::new(position_pda(&road, &owner), false)])
}
pub fn carry(road: Pubkey) -> Instruction {
    ix(poleis_roads::IX_CARRY, &[], vec![AccountMeta::new(road, false)])
}
pub fn decide(road: Pubkey) -> Instruction {
    ix(poleis_roads::IX_DECIDE, &[], vec![AccountMeta::new(road, false)])
}
/// world 0 pass, 1 fail; worth 0 to PRICE
pub fn receipts(judge: Pubkey, road: Pubkey, world: u8, worth: u64) -> Instruction {
    let mut d = vec![world];
    d.extend_from_slice(&worth.to_le_bytes());
    ix(poleis_roads::IX_RECEIPTS, &d, vec![AccountMeta::new_readonly(judge, true), AccountMeta::new(road, false)])
}
pub fn settle(road: Pubkey, owner: Pubkey, board_id: u64) -> Instruction {
    ix(
        poleis_roads::IX_SETTLE,
        &[],
        vec![
            AccountMeta::new(road, false),
            AccountMeta::new(position_pda(&road, &owner), false),
            AccountMeta::new(balance_pda(board_id, &owner), false),
            AccountMeta::new(owner, false),
        ],
    )
}

/// accounts: [road (w), the staker's balance (w), the road's board (w)]
pub fn collect(road: Pubkey, staker: Pubkey, board_id: u64) -> Instruction {
    ix(
        poleis_roads::IX_COLLECT,
        &[],
        vec![AccountMeta::new(road, false), AccountMeta::new(balance_pda(board_id, &staker), false), AccountMeta::new(board_pda(board_id), false)],
    )
}
/// What the house has earned on a board (`Board::OFF_EARNED`).
pub fn earned_of(d: &[u8]) -> u64 {
    assert_eq!(d.len(), Board::SIZE);
    u64_at(d, Board::OFF_EARNED)
}

// ---- readers: the program's own offsets ----
fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
fn u128_at(d: &[u8], o: usize) -> u128 {
    u128::from_le_bytes(d[o..o + 16].try_into().unwrap())
}
fn key_at(d: &[u8], o: usize) -> Pubkey {
    Pubkey::new_from_array(d[o..o + 32].try_into().unwrap())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardView {
    pub board_id: u64,
    pub authority: Pubkey,
    pub crediter: Pubkey,
    pub judge: Pubkey,
    pub credited: u64,
}
pub fn board_view(d: &[u8]) -> BoardView {
    assert_eq!((d.len(), d[0], d[1]), (Board::SIZE, poleis_roads::state::DISC_BOARD, poleis_roads::state::LAYOUT_VERSION));
    BoardView { board_id: u64_at(d, Board::OFF_BOARD), authority: key_at(d, Board::OFF_AUTHORITY), crediter: key_at(d, Board::OFF_CREDITER), judge: key_at(d, Board::OFF_JUDGE), credited: u64_at(d, Board::OFF_CREDITED) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalanceView {
    pub board_id: u64,
    pub owner: Pubkey,
    pub amount: u64,
    pub in_roads: u64,
}
pub fn balance_view(d: &[u8]) -> BalanceView {
    assert_eq!((d.len(), d[0], d[1]), (Balance::SIZE, poleis_roads::state::DISC_BALANCE, poleis_roads::state::LAYOUT_VERSION));
    BalanceView { board_id: u64_at(d, Balance::OFF_BOARD), owner: key_at(d, Balance::OFF_OWNER), amount: u64_at(d, Balance::OFF_AMOUNT), in_roads: u64_at(d, Balance::OFF_IN_ROADS) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoadView {
    pub board_id: u64,
    pub road_id: [u8; 32],
    pub verdict: u8,
    pub judge: Pubkey,
    pub terms: Terms,
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
    pub staker: Pubkey,
    pub stake: u64,
    pub collected: u8,
    /// what was in hands on the board when the road was laid: its `credited` less its `earned`
    pub credited_at_lay: u64,
}
impl RoadView {
    /// what is in the road: the stake and the deposits, less what was given back
    pub fn left(&self) -> u64 {
        self.stake + self.deposits - self.given_back
    }
}
fn branch_at(d: &[u8], o: usize, fees: usize) -> Branch {
    Branch {
        base: u128_at(d, o + Road::BR_BASE),
        quote: u128_at(d, o + Road::BR_QUOTE),
        obs: u128_at(d, o + Road::BR_OBS),
        cum: u128_at(d, o + Road::BR_CUM),
        acc: u128_at(d, o + Road::BR_ACC),
        at: u64_at(d, o + Road::BR_AT),
        steps: u64_at(d, o + Road::BR_STEPS),
        fees: u128_at(d, fees),
    }
}
pub fn road_view(d: &[u8]) -> RoadView {
    assert_eq!((d.len(), d[0], d[1]), (Road::SIZE, poleis_roads::state::DISC_ROAD, poleis_roads::state::LAYOUT_VERSION));
    RoadView {
        board_id: u64_at(d, Road::OFF_BOARD),
        road_id: d[Road::OFF_ROAD_ID..Road::OFF_ROAD_ID + 32].try_into().unwrap(),
        verdict: d[Road::OFF_VERDICT],
        judge: key_at(d, Road::OFF_JUDGE),
        terms: terms_of(
            u64_at(d, Road::OFF_OPENS),
            u64_at(d, Road::OFF_CLOSES),
            u64_at(d, Road::OFF_STEP),
            u32_at(d, Road::OFF_MAX_STEP_BP),
            u32_at(d, Road::OFF_THRESHOLD_BP),
            u32_at(d, Road::OFF_FEE_BP),
            u64_at(d, Road::OFF_POOL_BASE),
            u64_at(d, Road::OFF_POOL_QUOTE),
        )
        .unwrap(),
        judge_by_slot: u64_at(d, Road::OFF_JUDGE_BY),
        last_slot: u64_at(d, Road::OFF_LAST),
        pass: branch_at(d, Road::OFF_PASS, Road::OFF_PASS_FEES),
        fail: branch_at(d, Road::OFF_FAIL, Road::OFF_FAIL_FEES),
        said: d[Road::OFF_SAID],
        world: d[Road::OFF_WORLD],
        worth: u64_at(d, Road::OFF_WORTH),
        pool_result: i128::from_le_bytes(d[Road::OFF_POOL_RESULT..Road::OFF_POOL_RESULT + 16].try_into().unwrap()),
        positions: u32_at(d, Road::OFF_POSITIONS),
        deposits: u64_at(d, Road::OFF_DEPOSITS),
        given_back: u64_at(d, Road::OFF_GIVEN_BACK),
        staker: key_at(d, Road::OFF_STAKER),
        stake: u64_at(d, Road::OFF_STAKE),
        collected: d[Road::OFF_COLLECTED],
        credited_at_lay: u64_at(d, Road::OFF_CREDITED_AT_LAY),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionView {
    pub road: Pubkey,
    pub owner: Pubkey,
    pub deposit: u64,
    pub pass: Purse,
    pub fail: Purse,
    pub trades: u32,
}
pub fn position_view(d: &[u8]) -> PositionView {
    assert_eq!((d.len(), d[0], d[1]), (Position::SIZE, poleis_roads::state::DISC_POSITION, poleis_roads::state::LAYOUT_VERSION));
    PositionView {
        road: key_at(d, Position::OFF_ROAD),
        owner: key_at(d, Position::OFF_OWNER),
        deposit: u64_at(d, Position::OFF_DEPOSIT),
        pass: Purse { quote: u128_at(d, Position::OFF_PASS_QUOTE), base: u128_at(d, Position::OFF_PASS_BASE) },
        fail: Purse { quote: u128_at(d, Position::OFF_FAIL_QUOTE), base: u128_at(d, Position::OFF_FAIL_BASE) },
        trades: u32_at(d, Position::OFF_TRADES),
    }
}

// ---- the fold's side ----
/// The law's trade of a `trade` instruction's bytes (branch 0 pass, 1 fail; side 0 buy, 1 sell).
pub fn law_trade(branch: u8, side: u8, amount: u64, min_out: u64) -> embertide_futarchy::law::Trade {
    use embertide_futarchy::law::{Side, Trade, Which};
    Trade { branch: if branch == 0 { Which::Pass } else { Which::Fail }, side: if side == 0 { Side::Buy } else { Side::Sell }, amount: amount as u128, min_out: min_out as u128 }
}
/// One signed entry of the fold, stamped `ts`, by `key`.
pub fn entry(seq: u64, ts: u64, key: &Pubkey, trade: embertide_futarchy::law::Trade) -> embertide_futarchy::law::Entry {
    embertide_futarchy::law::Entry { seq, ts, key: key.to_string(), seat: key.to_string(), ok: true, trade: Some(trade) }
}
/// A roster of the fold from (owner, deposit) pairs, in the order of its keys.
pub fn roster(deposits: &[(Pubkey, u64)]) -> Vec<embertide_futarchy::law::Deposit> {
    let mut r: Vec<embertide_futarchy::law::Deposit> = deposits.iter().map(|(k, a)| embertide_futarchy::law::Deposit { key: k.to_string(), amount: *a as u128 }).collect();
    r.sort_by(|a, b| a.key.cmp(&b.key));
    r
}
/// The program's error for a refusal of the law (the program's own mapping, error.rs of_law).
pub fn law_code(why: &'static str) -> ProgramError {
    code(poleis_roads::error::of_law(why))
}

/// xorshift: a seeded sweep with no dependency (embertide-futarchy's tests' own)
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

// ---- the world ----
pub struct World {
    pub m: Mollusk,
    pub accounts: Vec<(Pubkey, Account)>,
    /// everything each board's crediter ever credited, counted from the credits that succeeded
    pub credited: std::collections::BTreeMap<u64, u128>,
    /// whether every instruction that succeeds is followed by THE BOOK of every board (off only where a test writes a road by hand)
    pub keep_book: bool,
}

pub fn code(e: RoadsError) -> ProgramError {
    ProgramError::Custom(e as u32)
}

impl World {
    /// A world whose clock stands at `slot`.
    pub fn new(slot: u64) -> Self {
        std::env::set_var("SBF_OUT_DIR", built::program_dir());
        let mut m = Mollusk::new(&PROGRAM_ID, "poleis_roads");
        m.sysvars.clock.slot = slot;
        Self { m, accounts: vec![keyed_account_for_system_program()], credited: Default::default(), keep_book: true }
    }
    pub fn slot(&self) -> u64 {
        self.m.sysvars.clock.slot
    }
    /// the clock in the law's milliseconds, as the program reads it
    pub fn now_ms(&self) -> u64 {
        self.slot() * poleis_roads::SLOT_MS
    }
    pub fn set_slot(&mut self, slot: u64) {
        self.m.sysvars.clock.slot = slot;
    }
    pub fn wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        key
    }
    pub fn account(&self, key: &Pubkey) -> Account {
        self.accounts.iter().find(|(k, _)| k == key).map(|(_, a)| a.clone()).unwrap_or_default()
    }
    pub fn data(&self, key: &Pubkey) -> Vec<u8> {
        self.account(key).data
    }
    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.account(key).lamports
    }
    /// Open board `board_id` for `auth` (it pays) and name its crediter and judge by its own signature: the two instructions every test of a
    /// board's market starts with, and their compute units.
    pub fn open_board(&mut self, auth: Pubkey, board_id: u64, crediter: Pubkey, judge: Pubkey) -> (u64, u64) {
        let c0 = self.ok(&init_board(auth, board_id, auth), "the board");
        let c1 = self.ok(&set_board(auth, board_id, crediter, judge), "its crediter and its judge");
        (c0, c1)
    }
    pub fn put(&mut self, key: Pubkey, account: Account) {
        match self.accounts.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = account,
            None => self.accounts.push((key, account)),
        }
    }
    pub fn road(&self, key: &Pubkey) -> RoadView {
        road_view(&self.data(key))
    }
    pub fn balance(&self, board_id: u64, owner: &Pubkey) -> BalanceView {
        balance_view(&self.data(&balance_pda(board_id, owner)))
    }
    pub fn position(&self, road: &Pubkey, owner: &Pubkey) -> PositionView {
        position_view(&self.data(&position_pda(road, owner)))
    }
    /// Run one instruction against the world: its compute units, or the program's refusal. Only a success changes the world (a failed
    /// instruction leaves nothing, as on a chain).
    pub fn run(&mut self, ix: &Instruction) -> Result<u64, ProgramError> {
        for meta in &ix.accounts {
            if !self.accounts.iter().any(|(k, _)| *k == meta.pubkey) {
                self.accounts.push((meta.pubkey, Account::default()));
            }
        }
        let r = self.m.process_instruction(ix, &self.accounts);
        match r.program_result {
            ProgramResult::Success => {
                for (key, account) in r.resulting_accounts {
                    self.put(key, account);
                }
                if ix.data[0] == poleis_roads::IX_CREDIT {
                    let board = board_view(&self.data(&ix.accounts[1].pubkey)).board_id;
                    *self.credited.entry(board).or_default() += u64::from_le_bytes(ix.data[1..9].try_into().unwrap()) as u128;
                }
                if self.keep_book {
                    for board in self.credited.keys().copied().collect::<Vec<_>>() {
                        book(self, board);
                    }
                }
                Ok(r.compute_units_consumed)
            }
            ProgramResult::Failure(e) => Err(e),
            ProgramResult::UnknownError(e) => panic!("the runtime failed: {e:?}"),
        }
    }
    /// Run an instruction that must succeed: its compute units.
    pub fn ok(&mut self, ix: &Instruction, what: &str) -> u64 {
        match self.run(ix) {
            Ok(cu) => cu,
            Err(e) => panic!("{what}: refused with {e:?}"),
        }
    }
    /// Run an instruction that must be refused with exactly `e`, and check that no account of the world changed.
    pub fn refuse(&mut self, ix: &Instruction, e: ProgramError, what: &str) {
        let before = self.accounts.clone();
        let got = self.run(ix);
        assert_eq!(got, Err(e), "{what}");
        // an account the instruction named that the world did not hold was added empty, and is empty still
        for (k, a) in &self.accounts {
            if !before.iter().any(|(b, _)| b == k) {
                assert_eq!(a, &Account::default(), "{what}: account {k} was created");
            }
        }
        for (k, a) in &before {
            assert_eq!(&self.account(k), a, "{what}: account {k} changed");
        }
        if self.keep_book {
            for board in self.credited.keys().copied().collect::<Vec<_>>() {
                book(self, board);
            }
        }
    }
}

/// THE BOOK of a board: the sum of every balance's `amount`, plus for every road not yet collected `stake + deposits - given_back`, plus
/// what the house earned on the board (`earned`, from the collects), equals everything the crediter ever credited, the board's own `credited` (held equal first to the world's tally of the credits that succeeded); and the sum of every balance's `in_roads` equals, over the roads, the stake of each road not yet
/// collected plus the deposits of the positions standing. Only `credit` changes the first sum. Only accounts at their own PDAs count (a test's
/// fakes at other addresses are not of the board). Returns what is in the roads not yet collected.
pub fn book(w: &World, board: u64) -> u128 {
    let (mut amounts, mut in_roads, mut left, mut staked, mut standing) = (0u128, 0u128, 0u128, 0u128, 0u128);
    let mut roads: Vec<Pubkey> = Vec::new();
    for (k, a) in &w.accounts {
        // only this layout's accounts: a test's fake of another version is nobody's
        if a.owner != PROGRAM_ID || a.data.len() < 2 || a.data[1] != poleis_roads::state::LAYOUT_VERSION {
            continue;
        }
        let d = &a.data;
        if d[0] == poleis_roads::state::DISC_BALANCE && d.len() == Balance::SIZE {
            let b = balance_view(d);
            if b.board_id == board && *k == balance_pda(board, &b.owner) {
                amounts += b.amount as u128;
                in_roads += b.in_roads as u128;
            }
        } else if d[0] == poleis_roads::state::DISC_ROAD && d.len() == Road::SIZE {
            let r = road_view(d);
            if r.board_id == board && *k == road_pda(board, &r.road_id) {
                roads.push(*k);
                if r.collected == 0 {
                    left += r.left() as u128;
                    staked += r.stake as u128;
                }
            }
        }
    }
    for (k, a) in &w.accounts {
        if a.owner == PROGRAM_ID && a.data.len() == Position::SIZE && a.data[0] == poleis_roads::state::DISC_POSITION && a.data[1] == poleis_roads::state::LAYOUT_VERSION {
            let p = position_view(&a.data);
            if roads.contains(&p.road) && *k == position_pda(&p.road, &p.owner) {
                standing += p.deposit as u128;
            }
        }
    }
    let credited = w.credited.get(&board).copied().unwrap_or(0);
    let on_chain = board_view(&w.data(&board_pda(board))).credited as u128;
    let earned = earned_of(&w.data(&board_pda(board))) as u128;
    assert_eq!(on_chain, credited, "THE BOOK of board {board}: the board's own credited is the world's tally of the credits");
    assert_eq!(amounts + left + earned, on_chain, "THE BOOK of board {board}: the balances' amounts, what is in the roads and what the house earned are what was credited");
    assert_eq!(in_roads, staked + standing, "THE BOOK of board {board}: the balances' in_roads are the stakes standing and the deposits standing");
    left
}
