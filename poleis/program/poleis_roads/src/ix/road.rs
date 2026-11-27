//! A road, from its lay to its last settle and its collect. Every number of the market is moved by one whole call of the law's core
//! (poleis-futarchy src/law.rs): `carry`, `stamp_check`, `trade_on`, `decide_at`, `happened`, `settle_at`, `pool_at`, `deposit_check`,
//! `purse_of`, `back_of`, `world_of`. This file reads and writes accounts around those calls and checks who may make them; it never puts the
//! law together from pieces. The clock is the slot (state.rs THE CLOCK IS THE SLOT); the law is handed its milliseconds.
//!
//! THE BOOK. A road holds `left = stake + deposits - given_back`: its staker's stake comes in at the lay, a deposit at its deposit, and nothing
//! leaves but a settle (capped at what is left) and the collect (exactly what is left: to the staker up to its stake, the rest to the board's
//! `earned`). So no road ever gives back more than entered it.

use crate::error::{of_law, RoadsError};
use crate::state::{terms_of, Balance, Board, Law, Position, Road, SEED_POSITION, SEED_ROAD, VERDICT_FAILED, VERDICT_OPEN, VERDICT_PASSED};
use crate::util::{close_to, create_pda, expect_signer, expect_writable, find_and_expect_pda, ms_of, now_ms, now_slot, stands, Data};
use crate::MOST;
use poleis_futarchy::law::{self, back_of, decide_at, deposit_check, happened, pool_at, settle_at, stamp_check, trade_on, world_of, Branch, Receipts, Side, Trade, Which, WINDOW_MAX_MS};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// data: [road_id:32][opens_slot:8][closes_slot:8][step_slots:8][max_step_bp:4][threshold_bp:4][fee_bp:4][pool_base:8][pool_quote:8][judge_by_slot:8]
/// accounts: [authority (s,w), board, road (w), balance (w), system]
/// Lays a road: its terms (in the law's milliseconds) must pass the law's `Terms::check`; it opens at or after the clock and at most the law's
/// longest window after it; its judge's deadline is after its close and at most the law's longest window after it. Both branches open as the
/// law's `Branch::open`. The road keeps the board's judge as it stands now. WHOEVER ASKS THE MARKET STAKES ITS POOL: `pool_quote + pool_base`
/// leaves the authority's own balance on this board for its `in_roads`, and the authority is the road's staker. Only the board's authority.
pub fn lay(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, board, road, balance, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(92)?;
    let road_id = d.h32(0)?;
    let (opens_slot, closes_slot, step_slots) = (d.u64(32)?, d.u64(40)?, d.u64(48)?);
    let (max_step_bp, threshold_bp, fee_bp) = (d.u32(56)?, d.u32(60)?, d.u32(64)?);
    let (pool_base, pool_quote, judge_by_slot) = (d.u64(68)?, d.u64(76)?, d.u64(84)?);
    expect_signer(authority)?;
    expect_writable(authority)?;
    let b = Board::read_pda(board)?;
    if authority.address() != &b.authority {
        return Err(RoadsError::Unauthorized.into());
    }
    let law = Law { polis: [0u8; 32], unit: 0, step_slots, max_step_bp, threshold_bp, fee_bp, pool_base, pool_quote, judge_slots: 0 };
    open_road(authority, &b, road, balance, road_id, opens_slot, closes_slot, judge_by_slot, &law, true)
}

/// data: none · accounts: [layer (s,w), board, road (w), the layer's balance (w), proposal, session, system]
/// A ROAD IS LAID BY A RULE: anyone with the stake free on this board lays the road of a proposal that asked for one, under the board's own
/// law (`set_law`). The proposal and its origin's session are read from the law's `polis` program by owner, discriminator, version and
/// offsets (`polis.rs`): a live proposal of the law's `unit` whose class byte asks the market or a plan, and that unit's session of its hash,
/// still voting. The road's id is the proposal's hash; its terms are the law's; its judge's deadline is its close plus the law's
/// `judge_slots`.
///
/// THE WINDOW IS THE VOTE'S, WHENEVER THE ROAD IS LAID: the road opens at the proposal's `opened` and closes at the session's deadline, as
/// Poleis's own late `convene` runs "from the proposal's opening, as if it had opened with it". The time before the lay stands in both branches
/// at the pool's opening price, so a road laid late has already said "no change" for that time, and the lay's timing changes nothing a road
/// says. A road laid late is born at rest at that price, and a stretch at rest costs nothing to carry (the law's A QUIET STRETCH IS ONE STEP):
/// its first trade carries the time before the lay in one step, with no `carry` first. At or after the deadline there is nothing left to lay
/// (`InvalidTerms`).
///
/// WHOEVER ASKS THE MARKET STAKES ITS POOL: the pool comes from the layer's OWN balance on this board (`InvalidAccount` for another's,
/// `NotEnoughBalance` when it cannot), never from a balance its owner did not sign for; the layer is the road's staker and pays its rent.
/// Only a holder with the stake free on the board can lay a road: a proposal nobody stakes has no market, and its split stands as DEADLOCK
/// and its plan cannot pass, as when nobody laid it. Once per hash.
pub fn lay_for(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [layer, board, road, balance, proposal, session, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(layer)?;
    expect_writable(layer)?;
    let b = Board::read_pda(board)?;
    if !b.law.stands() {
        return Err(RoadsError::NoLaw.into());
    }
    let polis = Address::new_from_array(b.law.polis);
    let p = crate::polis::proposal(proposal, &polis)?;
    if p.class & (crate::polis::CLASS_MARKET | crate::polis::CLASS_PLAN) == 0 {
        return Err(RoadsError::NotAsked.into());
    }
    if p.origin != b.law.unit {
        return Err(RoadsError::OtherUnit.into());
    }
    let s = crate::polis::session(session, &polis)?;
    if s.hash != p.hash {
        return Err(RoadsError::InvalidAccount.into());
    }
    if s.unit != b.law.unit {
        return Err(RoadsError::OtherUnit.into());
    }
    if s.state != crate::polis::STATE_VOTING {
        return Err(RoadsError::VoteClosed.into());
    }
    if now_slot()? >= s.deadline {
        return Err(RoadsError::InvalidTerms.into());
    }
    let judge_by = s.deadline.checked_add(b.law.judge_slots).ok_or(RoadsError::InvalidTerms)?;
    let law = b.law;
    open_road(layer, &b, road, balance, p.hash, p.opened, s.deadline, judge_by, &law, false)
}

/// Lay a road on board `b` with the terms of `law` and its window, its pool staked from `staker`'s own balance on this board (`balance`),
/// `staker` paying its rent: what `lay` and `lay_for` share. `from_now`: the road's opening must stand at or after the clock and within the
/// law's longest window of it (`lay`, whose layer picks the window); `lay_for`'s window is the vote's, its opening already past.
#[allow(clippy::too_many_arguments)]
fn open_road(
    staker: &AccountView,
    b: &Board,
    road: &mut AccountView,
    balance: &mut AccountView,
    road_id: [u8; 32],
    opens_slot: u64,
    closes_slot: u64,
    judge_by_slot: u64,
    law: &Law,
    from_now: bool,
) -> ProgramResult {
    expect_writable(road)?;
    expect_writable(balance)?;
    let id = b.board_id.to_le_bytes();
    let seeds: &[&[u8]] = &[SEED_ROAD, &id, &road_id];
    let bump = find_and_expect_pda(road, seeds)?;
    if stands(road) {
        return Err(RoadsError::RoadStands.into());
    }
    let (step_slots, max_step_bp, threshold_bp, fee_bp, pool_base, pool_quote) =
        (law.step_slots, law.max_step_bp, law.threshold_bp, law.fee_bp, law.pool_base, law.pool_quote);
    let t = terms_of(opens_slot, closes_slot, step_slots, max_step_bp, threshold_bp, fee_bp, pool_base, pool_quote).map_err(|_| RoadsError::InvalidTerms)?;
    t.check().map_err(|_| RoadsError::InvalidTerms)?;
    let judge_by_ms = ms_of(judge_by_slot).map_err(|_| RoadsError::InvalidTerms)?;
    let now = now_ms()?;
    // no span of a road is longer than the law's longest window: the wait to its opening, its window (the law's own check), and its judge's
    if from_now && (t.opens_ms < now || t.opens_ms - now > WINDOW_MAX_MS) {
        return Err(RoadsError::InvalidTerms.into());
    }
    if judge_by_ms <= t.closes_ms || judge_by_ms - t.closes_ms > WINDOW_MAX_MS {
        return Err(RoadsError::InvalidTerms.into());
    }
    let mut bal = Balance::read_pda(balance)?;
    if &bal.owner != staker.address() || bal.board_id != b.board_id {
        return Err(RoadsError::InvalidAccount.into());
    }
    let stake = pool_quote.checked_add(pool_base).ok_or(RoadsError::MathOverflow)?;
    if stake > bal.amount {
        return Err(RoadsError::NotEnoughBalance.into());
    }
    bal.amount -= stake;
    bal.in_roads = bal.in_roads.checked_add(stake).ok_or(RoadsError::MathOverflow)?;
    // what was in hands on the board at the lay: what the house earned back is in nobody's balance and can stand in no road
    let in_hands = b.credited.checked_sub(b.earned).ok_or(RoadsError::MathOverflow)?;
    create_pda(staker, road, Road::SIZE, seeds, bump)?;
    bal.write(balance)?;
    Road {
        board_id: b.board_id,
        road_id,
        verdict: VERDICT_OPEN,
        judge: b.judge,
        opens_slot,
        closes_slot,
        step_slots,
        max_step_bp,
        threshold_bp,
        pool_base,
        pool_quote,
        judge_by_slot,
        last_slot: opens_slot,
        pass: Branch::open(&t),
        fail: Branch::open(&t),
        said: 0,
        world: 0,
        worth: 0,
        pool_result: 0,
        positions: 0,
        deposits: 0,
        given_back: 0,
        staker: *staker.address(),
        stake,
        collected: 0,
        fee_bp,
        credited_at_lay: in_hands,
        bump,
    }
    .write(road)
}

/// data: [amount:8] · accounts: [owner (s,w), road (w), balance (w), position (w), system]
/// Before the close: the amount must pass the law's `deposit_check` and be within the balance's free amount. Creates the owner's one position
/// on this road with both purses the law's `purse_of(amount)`; the amount leaves the balance's `amount` for its `in_roads`; the road counts it
/// in a checked sum, and its book (`stake + deposits`) stays inside u64.
pub fn deposit(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, road, balance, position, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let amount = d.u64(0)?;
    expect_signer(owner)?;
    expect_writable(owner)?;
    expect_writable(road)?;
    expect_writable(balance)?;
    expect_writable(position)?;
    let mut r = Road::read_pda(road)?;
    let mut bal = Balance::read_pda(balance)?;
    if &bal.owner != owner.address() {
        return Err(RoadsError::Unauthorized.into());
    }
    if bal.board_id != r.board_id {
        return Err(RoadsError::InvalidAccount.into());
    }
    let seeds: &[&[u8]] = &[SEED_POSITION, road.address().as_ref(), owner.address().as_ref()];
    let bump = find_and_expect_pda(position, seeds)?;
    if stands(position) {
        return Err(RoadsError::AlreadyDeposited.into());
    }
    if r.verdict != VERDICT_OPEN || now_slot()? >= r.closes_slot {
        return Err(RoadsError::RoadClosed.into());
    }
    let t = r.terms()?;
    deposit_check(&t, amount as u128).map_err(|_| RoadsError::InvalidAmount)?;
    if amount > bal.amount {
        return Err(RoadsError::NotEnoughBalance.into());
    }
    bal.amount -= amount;
    bal.in_roads = bal.in_roads.checked_add(amount).ok_or(RoadsError::MathOverflow)?;
    r.positions = r.positions.checked_add(1).ok_or(RoadsError::MathOverflow)?;
    r.deposits = r.deposits.checked_add(amount).ok_or(RoadsError::MathOverflow)?;
    r.stake.checked_add(r.deposits).ok_or(RoadsError::MathOverflow)?;
    create_pda(owner, position, Position::SIZE, seeds, bump)?;
    let opens = t.purse_of(amount as u128);
    Position { road: *road.address(), owner: *owner.address(), deposit: amount, pass: opens, fail: opens, trades: 0, bump }.write(position)?;
    bal.write(balance)?;
    r.write(road)
}

/// data: [branch:1 (0 pass, 1 fail)][side:1 (0 buy, 1 sell)][amount:8][min_out:8] · accounts: [owner (s), road (w), position (w)]
/// One trade, stamped at the clock's slot: the law's bounded `carry` must bring the road to the clock, walking at most `MOST` boundaries of a
/// branch one at a time, a stretch at rest free (else `RoadBehind`: more than `MOST` boundaries are still to walk, a price that moved far and
/// was then left; send `carry` first; this instruction is then undone whole, its carry too), then the law's `stamp_check`, then the law's
/// `trade_on` on the purse of the branch named. A road nobody touched, however long, takes a trade with no `carry` first.
pub fn trade(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, road, position] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(18)?;
    let branch = match d.u8(0)? {
        0 => Which::Pass,
        1 => Which::Fail,
        _ => return Err(ProgramError::InvalidInstructionData),
    };
    let side = match d.u8(1)? {
        0 => Side::Buy,
        1 => Side::Sell,
        _ => return Err(ProgramError::InvalidInstructionData),
    };
    let tr = Trade { branch, side, amount: d.u64(2)? as u128, min_out: d.u64(10)? as u128 };
    expect_signer(owner)?;
    expect_writable(road)?;
    expect_writable(position)?;
    let mut r = Road::read_pda(road)?;
    let mut pos = Position::read_pda(position)?;
    if &pos.road != road.address() {
        return Err(RoadsError::InvalidAccount.into());
    }
    if &pos.owner != owner.address() {
        return Err(RoadsError::Unauthorized.into());
    }
    if r.verdict != VERDICT_OPEN {
        return Err(RoadsError::RoadClosed.into());
    }
    let slot = now_slot()?;
    let now = ms_of(slot)?;
    let t = r.terms()?;
    if !law::carry(&t, &mut r.pass, &mut r.fail, now, MOST) {
        return Err(RoadsError::RoadBehind.into());
    }
    stamp_check(&t, now, ms_of(r.last_slot)?, now).map_err(of_law)?;
    let mine = match branch {
        Which::Pass => &mut pos.pass,
        Which::Fail => &mut pos.fail,
    };
    trade_on(&t, &mut r.pass, &mut r.fail, mine, &tr, now).map_err(of_law)?;
    r.last_slot = slot;
    pos.trades = pos.trades.checked_add(1).ok_or(RoadsError::MathOverflow)?;
    pos.write(position)?;
    r.write_grid(road)
}

/// data: none · accounts: [road (w)]
/// Anyone: one bounded `carry` of the law toward the clock, whatever it answers: at most `MOST` boundaries of a branch walked one at a time
/// (those of an observation still chasing a moved spot), every stretch at rest carried whole. Never refuses a road that stands (nothing to
/// carry is Ok).
pub fn carry(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [road] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(road)?;
    let mut r = Road::read_pda(road)?;
    let now = now_ms()?;
    let t = r.terms()?;
    law::carry(&t, &mut r.pass, &mut r.fail, now, MOST);
    r.write_grid(road)
}

/// data: none · accounts: [road (w)]
/// Anyone, at or after the close, once: the law's bounded `carry` must reach the clock (else `RoadBehind`: more than `MOST` boundaries of a
/// chase are still to walk), then the law's `decide_at` and `world_of` write the verdict, 1 for pass, 2 for fail. A road nobody traded is
/// decided by this one instruction, with no `carry` first.
pub fn decide(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [road] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(road)?;
    let mut r = Road::read_pda(road)?;
    let slot = now_slot()?;
    if slot < r.closes_slot {
        return Err(RoadsError::NotClosed.into());
    }
    if r.verdict != VERDICT_OPEN {
        return Err(RoadsError::AlreadyDecided.into());
    }
    let now = ms_of(slot)?;
    let t = r.terms()?;
    if !law::carry(&t, &mut r.pass, &mut r.fail, now, MOST) {
        return Err(RoadsError::RoadBehind.into());
    }
    let decision = decide_at(&t, &mut r.pass, &mut r.fail, now);
    r.verdict = match world_of(&decision) {
        Some(Which::Pass) => VERDICT_PASSED,
        Some(Which::Fail) => VERDICT_FAILED,
        None => return Err(RoadsError::NotClosed.into()),
    };
    r.write(road)
}

/// data: [world:1 (0 pass, 1 fail)][worth:8] · accounts: [judge (s), road (w)]
/// The road's own judge, once, after the verdict and before `judge_by_slot`: the law's `decide_at` again (on a decided road it moves nothing)
/// gives the decision, the law's `happened` must take the receipts, and the law's `pool_at` gives what the pool gained in the world that happened.
pub fn receipts(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [judge, road] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(9)?;
    let world = match d.u8(0)? {
        0 => Which::Pass,
        1 => Which::Fail,
        _ => return Err(ProgramError::InvalidInstructionData),
    };
    let worth = d.u64(1)?;
    expect_signer(judge)?;
    expect_writable(road)?;
    let mut r = Road::read_pda(road)?;
    if judge.address() != &r.judge {
        return Err(RoadsError::Unauthorized.into());
    }
    if r.verdict == VERDICT_OPEN {
        return Err(RoadsError::NotDecided.into());
    }
    if r.said != 0 {
        return Err(RoadsError::AlreadySaid.into());
    }
    let slot = now_slot()?;
    if slot >= r.judge_by_slot {
        return Err(RoadsError::TooLate.into());
    }
    let t = r.terms()?;
    let decision = decide_at(&t, &mut r.pass, &mut r.fail, ms_of(slot)?);
    let rc = Receipts { world, worth: worth as u128 };
    happened(&decision, &rc).map_err(of_law)?;
    r.pool_result = pool_at(&t, &decision, &rc, &r.pass, &r.fail).map_err(of_law)?;
    r.said = 1;
    r.world = match world {
        Which::Pass => 0,
        Which::Fail => 1,
    };
    r.worth = worth;
    r.write(road)
}

/// data: none · accounts: [road (w), position (w), balance (w), owner (w)]
/// Anyone, for one position, once the judge has spoken: the law's `settle_at` on the receipts, then `back_of`; what comes back enters the
/// owner's balance, the deposit leaves its `in_roads`, the road counts it, and the position is closed to its owner. If the judge has NOT spoken
/// by `judge_by_slot`, the road is void and the position gets back its deposit. Otherwise `NotSettleable`. On both paths a position gets at
/// most what is left in the road: the cap never binds while the law's proofs hold, and no road gives back more than entered it whatever else
/// is wrong.
pub fn settle(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [road, position, balance, owner] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(road)?;
    expect_writable(position)?;
    expect_writable(balance)?;
    expect_writable(owner)?;
    let mut r = Road::read_pda(road)?;
    let pos = Position::read_pda(position)?;
    if &pos.road != road.address() || &pos.owner != owner.address() {
        return Err(RoadsError::InvalidAccount.into());
    }
    let mut bal = Balance::read_pda(balance)?;
    if bal.owner != pos.owner || bal.board_id != r.board_id {
        return Err(RoadsError::InvalidAccount.into());
    }
    let slot = now_slot()?;
    let back = if r.said != 0 {
        let t = r.terms()?;
        let (mut pass, mut fail) = (r.pass, r.fail);
        let decision = decide_at(&t, &mut pass, &mut fail, ms_of(slot)?);
        let rc = Receipts { world: if r.world == 0 { Which::Pass } else { Which::Fail }, worth: r.worth as u128 };
        let result = settle_at(&t, &decision, &rc, pos.deposit as u128, &pos.pass, &pos.fail).map_err(of_law)?;
        u64::try_from(back_of(pos.deposit as u128, result)).map_err(|_| RoadsError::MathOverflow)?
    } else if slot >= r.judge_by_slot {
        // the road is void: nobody's word came, and every position gets back its deposit
        pos.deposit
    } else {
        return Err(RoadsError::NotSettleable.into());
    };
    let back = back.min(r.left()?);
    bal.amount = bal.amount.checked_add(back).ok_or(RoadsError::MathOverflow)?;
    bal.in_roads = bal.in_roads.checked_sub(pos.deposit).ok_or(RoadsError::MathOverflow)?;
    r.positions = r.positions.checked_sub(1).ok_or(RoadsError::MathOverflow)?;
    r.given_back = r.given_back.checked_add(back).ok_or(RoadsError::MathOverflow)?;
    bal.write(balance)?;
    r.write(road)?;
    close_to(position, owner)
}

/// data: none · accounts: [road (w), the staker's balance (w), the road's board (w)]
/// Anyone, once: the road's staker takes back what is left in it, `stake + deposits - given_back`, up to its stake and no more, into its
/// balance's `amount`, and its stake leaves its `in_roads`; the rest is the house's, added to the board's `earned`. So asking the market can
/// cost the asker (the pool's loss, after the fees it took) and never pays it; the house can gain and never loses; whoever pushes a road he
/// staked himself pays the house for it. Only at or after the close and with no position standing; those two are enough: no deposit is taken
/// at or after the close, and a position leaves only by a settle (the judge spoke, or the road is void), so once the window has closed and no
/// position stands, nothing can enter or leave the road again. Whoever stakes a road has the stake back only once no position stands in it:
/// after the judge spoke and every position settled, or after the judge's time (`judge_slots`, the board's law) ran out and the void settles.
/// A road nobody deposited in is collected right after its close, exactly its stake.
pub fn collect(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [road, balance, board] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(road)?;
    expect_writable(balance)?;
    expect_writable(board)?;
    let mut r = Road::read_pda(road)?;
    let mut bal = Balance::read_pda(balance)?;
    if bal.owner != r.staker || bal.board_id != r.board_id {
        return Err(RoadsError::InvalidAccount.into());
    }
    let mut b = Board::read_pda(board)?;
    if b.board_id != r.board_id {
        return Err(RoadsError::InvalidAccount.into());
    }
    if r.collected != 0 {
        return Err(RoadsError::AlreadyCollected.into());
    }
    if now_slot()? < r.closes_slot || r.positions != 0 {
        return Err(RoadsError::NotCollectable.into());
    }
    let left = r.left()?;
    let back = left.min(r.stake);
    bal.amount = bal.amount.checked_add(back).ok_or(RoadsError::MathOverflow)?;
    bal.in_roads = bal.in_roads.checked_sub(r.stake).ok_or(RoadsError::MathOverflow)?;
    b.earned = b.earned.checked_add(left - back).ok_or(RoadsError::MathOverflow)?;
    r.collected = 1;
    bal.write(balance)?;
    b.write(board)?;
    r.write(road)
}
