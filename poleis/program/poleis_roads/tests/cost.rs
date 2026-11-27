//! The cost, printed and bounded: the compute units of every instruction; of a carry of a quiet road (A QUIET STRETCH IS ONE STEP: the same
//! however long it stood) at 1, MOST, ten thousand and a year of boundaries behind; of a carry that walks 1, MOST / 2 and MOST boundaries of a
//! chase (and so the units a walked boundary), the dearest carry MOST walked boundaries; of a trade on a road up to date and on one carried as far
//! as a trade can walk it; of a trade and a `decide` on a road nobody touched for its whole window; each account's size and rent; the program's
//! size; and how many `carry` a window of three days at a step a minute needs when nobody trades. Every line starts with `cost:`.
mod built;
mod common;
use common::*;
use embertide_futarchy::law::{step_toward, Branch, UNIT};
use poleis_roads::error::RoadsError;
use poleis_roads::state::{Balance, Board, Position, Road};
use poleis_roads::MOST;
use solana_pubkey::Pubkey;

/// the world's first slot
const S0: u64 = 1_000_000;
const U: u64 = UNIT as u64;
const BOARD: u64 = 11;
/// a minute in slots
const MIN: u64 = 60_000 / poleis_roads::SLOT_MS;

struct Fx {
    w: World,
    auth: Pubkey,
    a: Pubkey,
    judge: Pubkey,
    l: Lay,
    road: Pubkey,
    cu: [u64; 5],
}

/// a board, an owner credited, a road of three days at a step a minute opening a minute from now, the owner's deposit
fn fx() -> Fx {
    fx_at(1_000_000 * U, 1_000_000 * U)
}
/// the same, with the pool named
fn fx_at(pool_base: u64, pool_quote: u64) -> Fx {
    fx_with(pool_base, pool_quote, 5_000 * U, 3)
}
/// the same, with the owner's deposit and the window's days named
fn fx_with(pool_base: u64, pool_quote: u64, dep: u64, days: u64) -> Fx {
    let mut w = World::new(S0);
    let auth = w.wallet(10 * SOL);
    let (crediter, judge, a) = (w.wallet(10 * SOL), w.wallet(SOL), w.wallet(SOL));
    let (c0, _) = w.open_board(auth, BOARD, crediter, judge);
    let c1 = w.ok(&credit(crediter, BOARD, a, dep + 5_000 * U), "credit, creating the balance");
    let c2 = w.ok(&credit(crediter, BOARD, a, 1), "credit to a balance that stands");
    let opens_slot = S0 + MIN;
    let closes_slot = opens_slot + days * 1440 * MIN;
    let l = Lay { road_id: [4u8; 32], opens_slot, closes_slot, step_slots: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base, pool_quote, judge_by_slot: closes_slot + 1440 * MIN };
    w.ok(&credit(crediter, BOARD, auth, l.stake()), "the house's stake");
    let c3 = w.ok(&lay(auth, BOARD, &l), "lay");
    let road = road_pda(BOARD, &l.road_id);
    let c4 = w.ok(&deposit(a, BOARD, road, dep), "deposit");
    Fx { w, auth, a, judge, l, road, cu: [c0, c1, c2, c3, c4] }
}

/// a branch whose observation is still chasing its spot: the step it begins would move it
fn chasing(b: &Branch) -> bool {
    step_toward(b.obs, b.spot(), 100) != b.obs
}

/// the units of one `carry` of a road whose `k` boundaries are due (the clock half a step past the k-th), which must bring both branches to
/// the clock, every one of the k applied; `walked`: the observations chase all the way, so every one of them was walked one at a time
fn carry_on(mut f: Fx, k: u64, walked: bool) -> u64 {
    f.w.set_slot(f.l.opens_slot + k * MIN + MIN / 2);
    let cu = f.w.ok(&carry(f.road), "carry");
    let r = f.w.road(&f.road);
    assert_eq!((r.pass.steps, r.fail.steps), (k, k), "the carry applied {k} boundaries to each branch");
    assert_eq!((chasing(&r.pass), chasing(&r.fail)), (walked, walked), "after {k} boundaries the observations chase ({walked}): each was walked, or none");
    cu
}
/// a road of 1,000 a side whose two spots a trade at the opening moved ninefold, PASS up and FAIL down: its observations chase them some 220
/// steps at 1 % a step, so a carry of up to MOST boundaries walks every one
fn moved() -> Fx {
    let mut g = fx_at(1_000 * U, 1_000 * U);
    g.w.set_slot(g.l.opens_slot);
    g.w.ok(&trade(g.a, g.road, 0, 0, 2_000 * U, 0), "a buy that lifts PASS's spot");
    g.w.ok(&trade(g.a, g.road, 1, 1, 2_000 * U, 0), "a sell that lowers FAIL's spot");
    g
}

#[test]
fn the_cost() {
    let f = fx();
    let [c_init, c_credit_new, c_credit, c_lay, c_deposit] = f.cu;

    // A QUIET STRETCH IS ONE STEP: a carry with nothing to carry, and of a road nobody touched at 1, MOST, ten thousand and a year of
    // boundaries behind: the same units however long it stood
    let mut g = fx();
    let c_carry0 = g.w.ok(&carry(g.road), "a carry with nothing to carry");
    let year = 365 * 1440;
    let (q1, qm, q10k, qy) = (carry_on(fx(), 1, false), carry_on(fx(), MOST, false), carry_on(fx_with(1_000_000 * U, 1_000_000 * U, 5_000 * U, 7), 10_000, false), carry_on(fx_with(1_000_000 * U, 1_000_000 * U, 5_000 * U, 366), year, false));
    let quiet = [q1, qm, q10k, qy];
    let spread = quiet.iter().max().unwrap() - quiet.iter().min().unwrap();
    println!("cost: carry, nothing to carry: {c_carry0} CU");
    println!("cost: a quiet carry (nobody traded): at 1 boundary behind {q1} CU; at MOST = {MOST}: {qm} CU; at 10,000: {q10k} CU; at a year ({year}): {qy} CU; the spread {spread} CU");

    // the dearest carry: MOST boundaries walked one at a time, the observations chasing a spot a trade moved
    let (c1, ch, cm) = (carry_on(moved(), 1, true), carry_on(moved(), MOST / 2, true), carry_on(moved(), MOST, true));
    let per = (cm - c1) as f64 / (MOST - 1) as f64;
    println!("cost: a chasing carry walking 1 boundary: {c1} CU; MOST / 2 = {}: {ch} CU; MOST = {MOST}: {cm} CU", MOST / 2);
    println!("cost: a walked boundary (both branches): {per:.1} CU, from (chasing carry at MOST - at 1) / (MOST - 1)");
    assert!(spread <= 40 && *quiet.iter().max().unwrap() < c1 + 1_000 && *quiet.iter().max().unwrap() * 10 < cm, "a quiet carry costs the same however long it stood, and far under one walked MOST: {quiet:?} against {cm}");
    // the law's divisions cost by the size of what they divide, so the carry walking MOST is read at the lowest and the highest opening price
    // the law allows (an owner deposits enough to move it fourfold: some 139 steps of chase), over pools of random sizes, and from a partial acc;
    // the dearest is the one bounded
    let mut g = fx_with(1_000_000_000 * U, 1_000 * U, 1_000_000_000 * U, 3);
    g.w.set_slot(g.l.opens_slot);
    let q = g.w.position(&g.road, &g.a).pass.quote as u64;
    g.w.ok(&trade(g.a, g.road, 0, 0, q, 0), "a buy that lifts PASS's spot off the law's lowest");
    g.w.ok(&trade(g.a, g.road, 1, 0, q, 0), "a buy that lifts FAIL's spot off the law's lowest");
    let lo = carry_on(g, MOST, true);
    let mut g = fx_with(U, 1_000_000_000 * U, 1_000_000_000 * U, 3);
    g.w.set_slot(g.l.opens_slot);
    let b = g.w.position(&g.road, &g.a).pass.base as u64;
    g.w.ok(&trade(g.a, g.road, 0, 1, b, 0), "a sell that lowers PASS's spot off the law's highest");
    g.w.ok(&trade(g.a, g.road, 1, 1, b, 0), "a sell that lowers FAIL's spot off the law's highest");
    let hi = carry_on(g, MOST, true);
    let chasing_cu = carry_on(moved(), MOST, true);
    // a seeded sweep of pools of every size from a hundred thousand to a quarter of a billion a side (so a ninefold move stays in the law's range), an owner deposit twice the pool's, and its whole $WARM bought in
    // PASS and its whole claim sold in FAIL: each spot moved ninefold, its observation chasing
    let mut rng = Rng(0x3C6E_F372_FE94_F82B);
    let mut swept = 0u64;
    for _ in 0..16 {
        let base = 100_000 * U + rng.below(250_000_000 * U - 100_000 * U);
        let quote = 100_000 * U + rng.below(250_000_000 * U - 100_000 * U);
        let mut g = fx_with(base, quote, 2 * (base + quote), 3);
        g.w.set_slot(g.l.opens_slot);
        let a = g.w.position(&g.road, &g.a);
        let (q, b) = (a.pass.quote as u64, a.pass.base as u64);
        g.w.ok(&trade(g.a, g.road, 0, 0, q, 0), "a buy of the whole $WARM");
        g.w.ok(&trade(g.a, g.road, 1, 1, b, 0), "a sell of the whole claim");
        swept = swept.max(carry_on(g, MOST, true));
    }
    println!("cost: a carry walking MOST boundaries over 16 pools of random sizes, the observations chasing: the dearest {swept} CU");
    // and a road whose spot moved INSIDE the step the carry begins in: a trade three slots before the first boundary, so `acc` holds a partial
    // step of a moved spot when the carry starts
    let mut g = fx_at(1_000 * U, 1_000 * U);
    g.w.set_slot(g.l.opens_slot + MIN - 3);
    g.w.ok(&trade(g.a, g.road, 0, 0, 2_000 * U, 0), "a buy three slots before the first boundary");
    g.w.ok(&trade(g.a, g.road, 1, 1, 2_000 * U, 0), "a sell three slots before the first boundary");
    assert!(g.w.road(&g.road).pass.acc > 0, "acc holds a partial step");
    let partial = carry_on(g, MOST, true);
    println!("cost: a carry walking MOST boundaries, the spot moved inside the step it begins in (acc partial): {partial} CU");
    let dearest = cm.max(lo).max(hi).max(chasing_cu).max(swept).max(partial);
    println!("cost: a carry walking MOST boundaries, the opening price at the law's lowest: {lo} CU; at its highest: {hi} CU; the observations chasing a moved spot: {chasing_cu} CU; the dearest: {dearest} CU");
    assert!(dearest < 120_000, "carry at MOST boundaries: {dearest} CU, over 120,000");

    // a trade on a road up to date; on one whose chase is as far behind as a trade walks: the clock on the MOST-th boundary, all MOST due (a
    // boundary is due the instant the clock reaches it); refused one boundary further, taken at MOST again
    let mut g = fx();
    g.w.set_slot(g.l.opens_slot);
    let c_trade = g.w.ok(&trade(g.a, g.road, 0, 0, 100 * U, 0), "a trade on a road up to date");
    let c_trade2 = g.w.ok(&trade(g.a, g.road, 1, 1, 100 * U, 0), "a second trade at the same instant");
    let mut g = moved();
    g.w.set_slot(g.l.opens_slot + MOST * MIN);
    let c_trade_far = g.w.ok(&trade(g.a, g.road, 0, 1, 50 * U, 0), "a trade walking MOST boundaries");
    assert_eq!(g.w.road(&g.road).pass.steps, MOST);
    g.w.set_slot(g.l.opens_slot + (2 * MOST + 1) * MIN);
    assert_eq!(g.w.run(&trade(g.a, g.road, 0, 0, 50 * U, 0)), Err(code(RoadsError::RoadBehind)), "a trade MOST + 1 boundaries of chase behind is refused");
    g.w.set_slot(g.l.opens_slot + 2 * MOST * MIN);
    g.w.ok(&trade(g.a, g.road, 0, 0, 50 * U, 0), "a trade exactly MOST boundaries of chase behind is taken");
    let r = g.w.road(&g.road);
    assert_eq!(r.pass.steps, 2 * MOST);
    assert!(chasing(&r.pass) && chasing(&r.fail), "the chase was walked all the way");
    println!("cost: trade on a road up to date: {c_trade} CU (a second at the same instant: {c_trade2} CU); trade walking the most a trade walks (MOST = {MOST} boundaries of chase): {c_trade_far} CU");
    assert!(c_trade_far < 180_000, "trade at MOST: {c_trade_far} CU, over 180,000");

    // a road nobody touched for its whole window: a trade a slot before its close, and its decide at the close, each with no carry
    let mut g = fx();
    g.w.set_slot(g.l.closes_slot - 1);
    let c_trade_late = g.w.ok(&trade(g.a, g.road, 0, 0, 100 * U, 0), "a trade on a road nobody touched, a slot before its close");
    println!("cost: a trade on a road nobody touched for its whole window (a slot before its close): {c_trade_late} CU, no carry");

    // a window of three days at a step a minute, nobody trading: how many carry before the decision
    let mut g = fx();
    g.w.set_slot(g.l.closes_slot);
    let mut carries = 0u32;
    let c_decide = loop {
        match g.w.run(&decide(g.road)) {
            Ok(cu) => break cu,
            Err(e) if e == code(RoadsError::RoadBehind) => {
                g.w.ok(&carry(g.road), "carry");
                carries += 1;
            }
            Err(e) => panic!("decide: {e:?}"),
        }
    };
    let boundaries = (g.l.closes_slot - g.l.opens_slot) / MIN - 1;
    println!("cost: a window of three days at a step a minute ({boundaries} boundaries), nobody trading: {carries} carry before the decision; the decide alone {c_decide} CU");
    assert_eq!((carries, g.w.road(&g.road).pass.steps), (0, boundaries), "a road nobody traded is decided by one decide");
    let c_receipts = g.w.ok(&receipts(g.judge, g.road, g.w.road(&g.road).verdict - 1, 500_000_000_000), "receipts");
    let c_settle = g.w.ok(&settle(g.road, g.a, BOARD), "settle");
    let c_collect = g.w.ok(&collect(g.road, g.auth, BOARD), "collect");

    println!("cost: init_board {c_init} CU; credit creating the balance {c_credit_new} CU, to a balance that stands {c_credit} CU; lay {c_lay} CU; deposit {c_deposit} CU");
    println!("cost: decide {c_decide} CU; receipts {c_receipts} CU; settle {c_settle} CU; collect {c_collect} CU");

    // each account's size and rent, at the cluster's default rate; the program's size
    let rent = &f.w.m.sysvars.rent;
    for (name, size) in [("Board", Board::SIZE), ("Balance", Balance::SIZE), ("Road", Road::SIZE), ("Position", Position::SIZE)] {
        println!("cost: {name}: {size} bytes, rent {} lamports", rent.minimum_balance(size));
    }
    println!("cost: the program: {} bytes", built::program_elf().len());

    // A ROAD IS LAID BY A RULE: the board's law, and a road laid under it, by a holder from its own balance, for a proposal of a stand-in
    // polis that asked
    let mut w = World::new(S0);
    let auth = w.wallet(10 * SOL);
    w.open_board(auth, BOARD, auth, auth);
    let polis = Pubkey::new_from_array([0x50; 32]);
    let law = RuleLaw { polis, unit: 3, step_slots: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 30, pool_base: 1_000 * U, pool_quote: 1_000 * U, judge_slots: 60 * MIN };
    let c_law = w.ok(&set_law(auth, BOARD, &law), "set_law");
    w.ok(&credit(auth, BOARD, auth, law.stake()), "the house");
    let hash = [7u8; 32];
    let (proposal, session) = (Pubkey::new_unique(), Pubkey::new_unique());
    w.put(proposal, solana_account::Account { lamports: SOL, data: polis_proposal(&hash, 0x22, 3, S0, 900), owner: polis, executable: false, rent_epoch: 0 });
    w.put(session, solana_account::Account { lamports: SOL, data: polis_session(&hash, 3, S0 + 120 * MIN, 0), owner: polis, executable: false, rent_epoch: 0 });
    // the layer stakes from its own balance: here the authority, credited one stake above
    let c_lay_for = w.ok(&lay_for(auth, BOARD, &hash, proposal, session), "lay_for");
    println!("cost: set_law {c_law} CU; lay_for {c_lay_for} CU; collect (the staker back, the rest to earned) {c_collect} CU");
    assert!(c_law < 20_000 && c_lay_for < 40_000, "set_law {c_law}, lay_for {c_lay_for}");
}
