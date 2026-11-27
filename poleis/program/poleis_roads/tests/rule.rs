//! A ROAD IS LAID BY A RULE: a board's law (`set_law`), and the road of a proposal that asked for one laid by anyone with the stake free on
//! the board under it (`lay_for`), its window the vote's, its pool staked from the layer's own balance, its gain the house's. A proposal and a
//! session of the polis are bytes written by hand into accounts owned by a stand-in program id the board's law names, as Poleis's tests wrote
//! roads. Each refusal asserts the exact error and that no account changed (World::refuse); THE BOOK of every board, with what its house
//! earned, is held after every instruction that succeeds.
mod built;
mod common;
use common::*;
use poleis_futarchy::law::{UNIT, WINDOW_MAX_MS};
use poleis_roads::error::RoadsError::{self, *};
use poleis_roads::polis::{CLASS_MARKET, CLASS_PLAN, STATE_VOTING};
use poleis_roads::state::{VERDICT_FAILED, VERDICT_PASSED};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

const S0: u64 = 1_000_000;
const MIN: u64 = 60_000 / poleis_roads::SLOT_MS;
const U: u64 = UNIT as u64;
const BOARD: u64 = 5;
/// the polis's program, a stand-in, and the unit whose proposals open roads on this board
const POLIS: Pubkey = Pubkey::new_from_array([0x50; 32]);
const UNIT_ID: u64 = 3;
const HASH: [u8; 32] = [7u8; 32];
/// a treasury class (2) with the market's bit
const ASKS: u8 = 2 | CLASS_MARKET;
/// the proposal opened ten slots before the world's first; its session votes two hours from then
const OPENED: u64 = S0 - 10;
const DEADLINE: u64 = OPENED + 120 * MIN;

fn e(x: RoadsError) -> ProgramError {
    code(x)
}

struct Fx {
    w: World,
    auth: Pubkey,
    crediter: Pubkey,
    judge: Pubkey,
    /// a staker: an owner credited three stakes
    k: Pubkey,
    a: Pubkey,
    b: Pubkey,
    law: RuleLaw,
    proposal: Pubkey,
    session: Pubkey,
}

fn the_law() -> RuleLaw {
    RuleLaw { polis: POLIS, unit: UNIT_ID, step_slots: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 30, pool_base: 1_000 * U, pool_quote: 1_000 * U, judge_slots: 60 * MIN }
}

/// an account at a fresh address holding `data`, owned by `owner`
fn at(w: &mut World, data: Vec<u8>, owner: Pubkey) -> Pubkey {
    let key = Pubkey::new_unique();
    w.put(key, Account { lamports: SOL, data, owner, executable: false, rent_epoch: 0 });
    key
}

/// a board with its law; the authority (the house) credited a balance of its own that nothing here may touch; a staker credited three
/// stakes, two owners credited 5,000 each; a proposal of the polis that asked, opened before the clock, and its origin's session
fn fx() -> Fx {
    let mut w = World::new(S0);
    let auth = w.wallet(10 * SOL);
    let (crediter, judge) = (w.wallet(10 * SOL), w.wallet(SOL));
    let (k, a, b) = (w.wallet(SOL), w.wallet(SOL), w.wallet(SOL));
    w.open_board(auth, BOARD, crediter, judge);
    let law = the_law();
    w.ok(&set_law(auth, BOARD, &law), "the board's law");
    w.ok(&credit(crediter, BOARD, auth, 10 * law.stake()), "the house's own balance");
    w.ok(&credit(crediter, BOARD, k, 3 * law.stake()), "the staker");
    w.ok(&credit(crediter, BOARD, a, 5_000 * U), "credit a");
    w.ok(&credit(crediter, BOARD, b, 5_000 * U), "credit b");
    let proposal = at(&mut w, polis_proposal(&HASH, ASKS, UNIT_ID, OPENED, 900), POLIS);
    let session = at(&mut w, polis_session(&HASH, UNIT_ID, DEADLINE, STATE_VOTING), POLIS);
    Fx { w, auth, crediter, judge, k, a, b, law, proposal, session }
}

#[test]
fn a_boards_law_is_set_by_its_authority_alone_and_takes_only_terms_the_law_takes() {
    let mut f = fx();
    let (auth, w) = (f.auth, &mut f.w);
    assert_eq!(law_view(&w.data(&board_pda(BOARD))), f.law);
    let s = w.wallet(SOL);
    let mut unsigned = set_law(auth, BOARD, &f.law);
    unsigned.accounts[0].is_signer = false;
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "set_law unsigned");
    w.refuse(&set_law(s, BOARD, &f.law), e(Unauthorized), "set_law by a stranger");
    w.refuse(&set_law(f.crediter, BOARD, &f.law), e(Unauthorized), "set_law by the crediter");
    let l = f.law;
    let bad = [
        ("a step of nothing", RuleLaw { step_slots: 0, ..l }),
        ("a step under a second", RuleLaw { step_slots: 2, ..l }),
        ("a step whose two-step window overflows", RuleLaw { step_slots: u64::MAX, ..l }),
        ("a step whose two-step window is longer than the law's longest", RuleLaw { step_slots: WINDOW_MAX_MS / poleis_roads::SLOT_MS / 2 + 1, ..l }),
        ("a lag limit of 0", RuleLaw { max_step_bp: 0, ..l }),
        ("a lag limit over 10000", RuleLaw { max_step_bp: 10_001, ..l }),
        ("a threshold over 10000", RuleLaw { threshold_bp: 10_001, ..l }),
        ("a fee of 10000", RuleLaw { fee_bp: 10_000, ..l }),
        ("an empty base", RuleLaw { pool_base: 0, ..l }),
        ("an empty quote", RuleLaw { pool_quote: 0, ..l }),
        ("a pool over a billion", RuleLaw { pool_base: u64::MAX, ..l }),
        ("an opening price under the law's range", RuleLaw { pool_base: 1_000_000_000 * U, pool_quote: 1, ..l }),
        ("no time for the judge", RuleLaw { judge_slots: 0, ..l }),
        ("the judge longer than the law's longest window", RuleLaw { judge_slots: WINDOW_MAX_MS / poleis_roads::SLOT_MS + 1, ..l }),
    ];
    for (what, law) in bad {
        w.refuse(&set_law(auth, BOARD, &law), e(InvalidTerms), what);
    }
    // the longest the judge may have, and a step at the law's floor, are taken
    w.ok(&set_law(auth, BOARD, &RuleLaw { judge_slots: WINDOW_MAX_MS / poleis_roads::SLOT_MS, step_slots: 3, ..l }), "the edges");
    // a polis of all zero turns the rule off and keeps no terms
    w.ok(&set_law(auth, BOARD, &RuleLaw { polis: Pubkey::default(), ..l }), "the rule off");
    assert_eq!(law_view(&w.data(&board_pda(BOARD))), RuleLaw { polis: Pubkey::default(), unit: 0, step_slots: 0, max_step_bp: 0, threshold_bp: 0, fee_bp: 0, pool_base: 0, pool_quote: 0, judge_slots: 0 });
    w.refuse(&lay_for(f.k, BOARD, &HASH, f.proposal, f.session), e(NoLaw), "a board with no law lays nothing by rule");

    // a later law changes only the roads laid after it
    w.ok(&set_law(auth, BOARD, &l), "the law again");
    w.ok(&lay_for(f.k, BOARD, &HASH, f.proposal, f.session), "a road by rule");
    let road = road_pda(BOARD, &HASH);
    let before = w.road(&road);
    w.ok(&set_law(auth, BOARD, &RuleLaw { fee_bp: 0, threshold_bp: 50, pool_base: 2_000 * U, step_slots: 2 * MIN, judge_slots: MIN, ..l }), "a later law");
    assert_eq!(w.road(&road), before, "a road keeps the terms it was laid with");
}

#[test]
fn whoever_lays_a_road_stakes_it_from_its_own_balance_and_its_window_is_the_votes() {
    let mut f = fx();
    let w = &mut f.w;
    w.set_slot(S0 + 7);
    let (k0, credited) = (w.balance(BOARD, &f.k), board_view(&w.data(&board_pda(BOARD))).credited);
    let house = w.data(&balance_pda(BOARD, &f.auth));
    let lamports = w.lamports(&f.k);
    let ix = lay_for(f.k, BOARD, &HASH, f.proposal, f.session);
    assert!(ix.accounts.iter().all(|m| m.pubkey != f.auth && m.pubkey != balance_pda(BOARD, &f.auth)), "no account of the authority is in the instruction");
    let cu = w.ok(&ix, "a road laid by a staker, by rule");
    eprintln!("  lay_for: {cu} CU");
    let road = road_pda(BOARD, &HASH);
    let r = w.road(&road);
    let l = f.law;
    assert_eq!(r.road_id, HASH, "the road's id is the proposal's hash");
    let terms = poleis_roads::state::terms_of(OPENED, DEADLINE, l.step_slots, l.max_step_bp, l.threshold_bp, l.fee_bp, l.pool_base, l.pool_quote).unwrap();
    assert_eq!(r.terms, terms, "its terms are the board's law to the field; it opens at the proposal's opening and closes at the session's deadline");
    assert_eq!(r.judge_by_slot, DEADLINE + l.judge_slots, "its judge has the law's slots after the close");
    assert_eq!((r.staker, r.judge, r.stake, r.credited_at_lay, r.verdict), (f.k, f.judge, l.stake(), credited, 0));
    let k1 = w.balance(BOARD, &f.k);
    assert_eq!((k0.amount - k1.amount, k1.in_roads - k0.in_roads), (l.stake(), l.stake()), "the stake left the layer's own balance for its in_roads");
    assert_eq!(w.data(&balance_pda(BOARD, &f.auth)), house, "the house's balance is byte for byte as it was");
    assert_eq!(lamports - w.lamports(&f.k), w.lamports(&road), "the layer paid the road's rent");
    w.refuse(&lay_for(f.k, BOARD, &HASH, f.proposal, f.session), e(RoadStands), "the road of a hash twice");
    // a plan asks too
    let plan = [8u8; 32];
    let p2 = at(w, polis_proposal(&plan, 4 | CLASS_PLAN, UNIT_ID, S0, 900), POLIS);
    let s2 = at(w, polis_session(&plan, UNIT_ID, DEADLINE, STATE_VOTING), POLIS);
    w.ok(&lay_for(f.a, BOARD, &plan, p2, s2), "a plan's road, staked by another holder");
    assert_eq!(w.road(&road_pda(BOARD, &plan)).staker, f.a);
}

#[test]
fn lay_for_refusals() {
    let mut f = fx();
    let (auth, k, w) = (f.auth, f.k, &mut f.w);
    let (p, ss) = (f.proposal, f.session);
    let go = |p: Pubkey, ss: Pubkey| lay_for(k, BOARD, &HASH, p, ss);

    // the layer signs
    let mut unsigned = go(p, ss);
    unsigned.accounts[0].is_signer = false;
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "lay_for unsigned");
    // a board with no law
    let auth2 = w.wallet(10 * SOL);
    w.ok(&init_board(auth2, BOARD + 1, auth2), "board 6, no law");
    w.ok(&credit(auth2, BOARD + 1, k, 10 * f.law.stake()), "the staker on board 6");
    w.refuse(&lay_for(k, BOARD + 1, &HASH, p, ss), e(NoLaw), "a board with no law");

    // the proposal: of another program, of another unit, that did not ask, closed to its tombstone, of another kind or version
    let other_program = at(w, polis_proposal(&HASH, ASKS, UNIT_ID, OPENED, 900), Pubkey::new_from_array([0x51; 32]));
    w.refuse(&go(other_program, ss), e(InvalidAccount), "a proposal of another program");
    let ours = at(w, polis_proposal(&HASH, ASKS, UNIT_ID, OPENED, 900), PROGRAM_ID);
    w.refuse(&go(ours, ss), e(InvalidAccount), "a proposal-shaped account of the roads program itself");
    let other_unit = at(w, polis_proposal(&HASH, ASKS, UNIT_ID + 1, OPENED, 900), POLIS);
    w.refuse(&go(other_unit, ss), e(OtherUnit), "a proposal of another unit");
    for class in [2u8, 2 | 0x40, 2 | 0x80, 0] {
        let quiet = at(w, polis_proposal(&HASH, class, UNIT_ID, OPENED, 900), POLIS);
        w.refuse(&go(quiet, ss), e(NotAsked), "a proposal that did not ask the market");
    }
    let tomb = at(w, polis_proposal(&HASH, ASKS, UNIT_ID, OPENED, 34), POLIS);
    w.refuse(&go(tomb, ss), e(InvalidAccount), "a proposal closed to its tombstone");
    let short = at(w, polis_proposal(&HASH, ASKS, UNIT_ID, OPENED, 114), POLIS);
    w.refuse(&go(short, ss), e(InvalidAccount), "a proposal one byte short of `opened`");
    let mut disc = polis_proposal(&HASH, ASKS, UNIT_ID, OPENED, 900);
    disc[0] = poleis_roads::polis::DISC_SESSION;
    let disc = at(w, disc, POLIS);
    w.refuse(&go(disc, ss), e(InvalidAccount), "a session in the proposal's place");
    let mut ver = polis_proposal(&HASH, ASKS, UNIT_ID, OPENED, 900);
    ver[1] = 7;
    let ver = at(w, ver, POLIS);
    w.refuse(&go(ver, ss), e(LayoutVersion), "a proposal of another layout version");
    let empty = w.wallet(SOL);
    w.refuse(&go(empty, ss), e(InvalidAccount), "a wallet in the proposal's place");

    // the session: of another hash, another unit, sealed, of another program or kind
    let other_hash = at(w, polis_session(&[9u8; 32], UNIT_ID, DEADLINE, STATE_VOTING), POLIS);
    w.refuse(&go(p, other_hash), e(InvalidAccount), "a proposal of another hash than the session's");
    let crew = at(w, polis_session(&HASH, UNIT_ID + 1, DEADLINE, STATE_VOTING), POLIS);
    w.refuse(&go(p, crew), e(OtherUnit), "the session of another unit (a crew's)");
    for state in 1u8..=4 {
        let sealed = at(w, polis_session(&HASH, UNIT_ID, DEADLINE, state), POLIS);
        w.refuse(&go(p, sealed), e(VoteClosed), "a session already sealed");
    }
    let foreign = at(w, polis_session(&HASH, UNIT_ID, DEADLINE, STATE_VOTING), Pubkey::new_from_array([0x51; 32]));
    w.refuse(&go(p, foreign), e(InvalidAccount), "a session of another program");
    w.refuse(&go(p, p), e(InvalidAccount), "the proposal in the session's place");

    // the clock: at the deadline's own slot, and after it, there is nothing left to lay
    let now = w.slot();
    w.set_slot(DEADLINE);
    w.refuse(&go(p, ss), e(InvalidTerms), "at the deadline's own slot");
    w.set_slot(DEADLINE + 1);
    w.refuse(&go(p, ss), e(InvalidTerms), "after the deadline");
    w.set_slot(now);
    // a vote longer than the law's longest window: no road is that long
    let far = at(w, polis_session(&HASH, UNIT_ID, OPENED + WINDOW_MAX_MS / poleis_roads::SLOT_MS + 1, STATE_VOTING), POLIS);
    w.refuse(&go(p, far), e(InvalidTerms), "a window longer than the law's longest");

    // WHOEVER ASKS THE MARKET STAKES ITS POOL: from its own balance on this board, and enough of it
    let swap = |mut ix: Instruction, key: Pubkey| {
        ix.accounts[3].pubkey = key;
        ix
    };
    w.refuse(&swap(go(p, ss), balance_pda(BOARD, &auth)), e(InvalidAccount), "the house's balance: a balance that is another's");
    w.refuse(&swap(go(p, ss), balance_pda(BOARD, &f.a)), e(InvalidAccount), "another owner's balance");
    w.refuse(&swap(go(p, ss), balance_pda(BOARD + 1, &k)), e(InvalidAccount), "the layer's balance of another board");
    let stranger = w.wallet(SOL);
    w.refuse(&lay_for(stranger, BOARD, &HASH, p, ss), e(InvalidAccount), "a stranger with no balance on the board");
    let poor = w.wallet(SOL);
    w.ok(&credit(f.crediter, BOARD, poor, f.law.stake() - 1), "a holder one micro short of the stake");
    w.refuse(&lay_for(poor, BOARD, &HASH, p, ss), e(NotEnoughBalance), "a layer short of the stake");

    // the board and the road at their own addresses
    let mut wrong_board = go(p, ss);
    wrong_board.accounts[1].pubkey = board_pda(BOARD + 9);
    w.refuse(&wrong_board, e(InvalidAccount), "a board that does not stand");
    let mut wrong_road = go(p, ss);
    wrong_road.accounts[2].pubkey = road_pda(BOARD, &[9u8; 32]);
    w.refuse(&wrong_road, e(InvalidPda), "the road of another hash");
    let mut readonly_road = go(p, ss);
    readonly_road.accounts[2].is_writable = false;
    w.refuse(&readonly_road, e(InvalidAccount), "a road not writable");

    // and the one that stands takes it, once
    w.ok(&go(p, ss), "the road by rule");
    w.refuse(&go(p, ss), e(RoadStands), "twice");
    let _ = (f.b, f.judge);
}

/// At `at_slot` the deposits of a (1,000) and b (2,000), then `trades` (slot, 0 a / 1 b, branch, side, amount), each sent until the road is
/// not behind (a `carry` between); then the close and `decide`. The road at its verdict, and how many carries it took.
fn drive(w: &mut World, a: Pubkey, b: Pubkey, at_slot: u64, trades: &[(u64, usize, u8, u8, u64)]) -> (RoadView, u32) {
    let road = road_pda(BOARD, &HASH);
    w.set_slot(at_slot);
    for (o, d) in [(a, 1_000 * U), (b, 2_000 * U)] {
        w.ok(&deposit(o, BOARD, road, d), "a deposit");
    }
    let owners = [a, b];
    let mut carries = 0;
    for &(slot, who, branch, side, amount) in trades {
        w.set_slot(slot);
        loop {
            match w.run(&trade(owners[who], road, branch, side, amount, 0)) {
                Ok(_) => break,
                Err(x) if x == e(RoadBehind) => {
                    w.ok(&carry(road), "a carry");
                    carries += 1;
                }
                Err(x) => panic!("a trade at slot {slot} refused: {x:?}"),
            }
        }
    }
    w.set_slot(DEADLINE);
    loop {
        match w.run(&decide(road)) {
            Ok(_) => break,
            Err(x) if x == e(RoadBehind) => {
                w.ok(&carry(road), "a carry to the close");
                carries += 1;
            }
            Err(x) => panic!("decide refused: {x:?}"),
        }
    }
    (w.road(&road), carries)
}

#[test]
fn the_lays_timing_changes_nothing_a_road_says() {
    let late = OPENED + 80 * MIN;
    let trades: [(u64, usize, u8, u8, u64); 5] =
        [(late + 2, 0, 0, 0, 200 * U), (late + 10 * MIN, 1, 1, 0, 100 * U), (late + 20 * MIN, 0, 0, 0, 100 * U), (late + 30 * MIN, 1, 0, 1, 50 * U), (DEADLINE - 3, 1, 0, 0, 80 * U)];
    // one road laid at the opening, the other more than 66 steps into the vote; then the same deposits and trades at the same slots
    let mut f1 = fx();
    f1.w.set_slot(OPENED);
    f1.w.ok(&lay_for(f1.k, BOARD, &HASH, f1.proposal, f1.session), "laid at the opening");
    let (r1, c1) = drive(&mut f1.w, f1.a, f1.b, late + 1, &trades);
    let mut f2 = fx();
    f2.w.set_slot(late);
    f2.w.ok(&lay_for(f2.k, BOARD, &HASH, f2.proposal, f2.session), "laid 80 steps into the vote");
    assert_eq!(f2.w.road(&road_pda(BOARD, &HASH)).last_slot, OPENED, "a road laid late stands at the vote's opening");
    let (r2, c2) = drive(&mut f2.w, f2.a, f2.b, late + 1, &trades);
    // A QUIET STRETCH IS ONE STEP: the eighty steps the late road was born behind stood at the opening price, at rest, as they did on the early
    // road nobody touched, so each takes its first trade with no carry at all; and no gap between the trades leaves more than MOST boundaries
    // of chase to walk, so neither road needs a carry anywhere
    assert_eq!((c1, c2), (0, 0), "the late road took its first trade with no carry, as the early one untouched ({c1} and {c2} carries)");
    assert_eq!((r1.pass, r1.fail, r1.verdict, r1.terms), (r2.pass, r2.fail, r2.verdict, r2.terms), "the two roads are equal field for field");

    // the push that passes a road held from the opening does not pass it made only in the window's last steps
    let held: Vec<(u64, usize, u8, u8, u64)> = (0..4).map(|i| (OPENED + 1 + i * MIN, (i % 2) as usize, 0u8, 0u8, 200 * U)).collect();
    let mut f3 = fx();
    f3.w.set_slot(OPENED);
    f3.w.ok(&lay_for(f3.k, BOARD, &HASH, f3.proposal, f3.session), "laid at the opening");
    let (held_road, _) = drive(&mut f3.w, f3.a, f3.b, OPENED + 1, &held);
    let last: Vec<(u64, usize, u8, u8, u64)> = (0..4).map(|i| (DEADLINE - 4 + i, (i % 2) as usize, 0u8, 0u8, 200 * U)).collect();
    let mut f4 = fx();
    f4.w.set_slot(DEADLINE - 5);
    f4.w.ok(&lay_for(f4.k, BOARD, &HASH, f4.proposal, f4.session), "laid five slots before the deadline");
    let (rushed, _) = drive(&mut f4.w, f4.a, f4.b, DEADLINE - 5, &last);
    assert_eq!((held_road.verdict, rushed.verdict), (VERDICT_PASSED, VERDICT_FAILED), "held from the opening it passes; made in the last steps it does not");
}

/// A road laid by rule at the opening, a's and b's deposits, `trades`, decided, the judge's receipts at `worth` a claim, settled: the
/// world just before the collect, and the road.
fn judged(trades: &[(u64, usize, u8, u8, u64)], worth: u64) -> (Fx, Pubkey) {
    let mut f = fx();
    f.w.set_slot(OPENED);
    f.w.ok(&lay_for(f.k, BOARD, &HASH, f.proposal, f.session), "the road by rule");
    let (r, _) = drive(&mut f.w, f.a, f.b, OPENED + 1, trades);
    let road = road_pda(BOARD, &HASH);
    let world = if r.verdict == VERDICT_PASSED { 0 } else { 1 };
    f.w.ok(&receipts(f.judge, road, world, worth), "the judge's receipts");
    let (a, b) = (f.a, f.b);
    for o in [a, b] {
        f.w.ok(&settle(road, o, BOARD), "a settle");
    }
    (f, road)
}

#[test]
fn collect_gives_the_staker_at_most_its_stake_and_the_house_the_rest() {
    // a road the pool gained in: the traders bought PASS claims that were worth nothing in the PASS world
    let (mut f, road) = judged(&[(OPENED + 2, 0, 0, 0, 300 * U), (OPENED + 5, 1, 0, 0, 600 * U)], 0);
    let (stake, left) = (f.law.stake(), f.w.road(&road).left());
    assert!(left > stake, "the pool gained: {left} left of a stake of {stake}");
    let k0 = f.w.balance(BOARD, &f.k);
    let cu = f.w.ok(&collect(road, f.k, BOARD), "collect, by anyone");
    eprintln!("  collect: {cu} CU");
    let k1 = f.w.balance(BOARD, &f.k);
    assert_eq!((k1.amount - k0.amount, k0.in_roads - k1.in_roads), (stake, stake), "the stake back whole, and no more");
    assert_eq!(earned_of(&f.w.data(&board_pda(BOARD))), left - stake, "the gain is the house's");
    f.w.refuse(&collect(road, f.k, BOARD), e(AlreadyCollected), "twice");
    // what was in hands at the next lay is what was credited less what the house earned
    let plan = [8u8; 32];
    let p2 = at(&mut f.w, polis_proposal(&plan, 4 | CLASS_PLAN, UNIT_ID, DEADLINE, 900), POLIS);
    let s2 = at(&mut f.w, polis_session(&plan, UNIT_ID, DEADLINE + 120 * MIN, STATE_VOTING), POLIS);
    f.w.ok(&lay_for(f.k, BOARD, &plan, p2, s2), "a second road");
    let board = board_view(&f.w.data(&board_pda(BOARD)));
    assert_eq!(f.w.road(&road_pda(BOARD, &plan)).credited_at_lay, board.credited - (left - stake), "credited_at_lay is credited less earned at the lay");

    // a road the pool lost in: b sold FAIL claims for $WARM, the FAIL world happened, and they were worth nothing
    let (mut f, road) = judged(&[(OPENED + 2, 0, 0, 1, 400 * U), (OPENED + 5, 1, 1, 1, 300 * U)], 0);
    assert_eq!(f.w.road(&road).verdict, VERDICT_FAILED);
    let (stake, left) = (f.law.stake(), f.w.road(&road).left());
    assert!(left < stake, "the pool lost: {left} left of a stake of {stake}");
    let k0 = f.w.balance(BOARD, &f.k);
    let mut other_board = collect(road, f.k, BOARD);
    other_board.accounts[2].pubkey = board_pda(BOARD + 1);
    f.w.refuse(&other_board, e(InvalidAccount), "a board that is not the road's (none stands there)");
    f.w.ok(&collect(road, f.k, BOARD), "collect");
    let k1 = f.w.balance(BOARD, &f.k);
    assert_eq!((k1.amount - k0.amount, k0.in_roads - k1.in_roads), (left, stake), "the staker gets what was left, under its stake");
    assert_eq!(earned_of(&f.w.data(&board_pda(BOARD))), 0, "the house never loses: its earned does not move");

    // a road nobody deposited in: exactly the stake, right after the close
    let mut f = fx();
    f.w.set_slot(OPENED);
    f.w.ok(&lay_for(f.k, BOARD, &HASH, f.proposal, f.session), "a road nobody deposits in");
    let road = road_pda(BOARD, &HASH);
    f.w.set_slot(DEADLINE - 1);
    f.w.refuse(&collect(road, f.k, BOARD), e(NotCollectable), "a slot before the close");
    f.w.set_slot(DEADLINE);
    let k0 = f.w.balance(BOARD, &f.k);
    f.w.ok(&collect(road, f.k, BOARD), "collect at the close");
    let k1 = f.w.balance(BOARD, &f.k);
    assert_eq!((k1.amount - k0.amount, k0.in_roads - k1.in_roads), (f.law.stake(), f.law.stake()), "exactly the stake");
    assert_eq!(earned_of(&f.w.data(&board_pda(BOARD))), 0);
    assert_eq!(book(&f.w, BOARD), 0);
}
