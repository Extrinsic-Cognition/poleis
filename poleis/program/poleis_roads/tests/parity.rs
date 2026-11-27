//! THE PROGRAM IS THE FOLD, trade by trade. A seeded sweep of roads, each with trades in both worlds (some refused) on a clock moved by random
//! amounts, none among them and long gaps that need several `carry`. After every instruction that succeeds, the Road's two branches and every
//! Position's two purses equal, field by field, the fold's market at that instant (`market_with` over the trades so far, read at the clock's
//! milliseconds). The fold ends its reading with a carry to that instant; a road behind the clock (after a carry that answered false, or a
//! refused trade, which left the road where the instruction before it did) has not taken it yet, so COPIES of the Road's branches are read
//! through the law's own `decide_at` at that instant first (the law as a library, for this one call; nothing is written to the account).
//! On the law's rule a boundary is due the instant the clock reaches it, so after ANY TRADE taken, and after a CARRY THAT ANSWERED TRUE, the
//! road itself IS the fold's road at that instant, with no call in between: that is asserted directly in those two cases (whether a carry
//! answered true is the law's own `carry` on copies, run before the instruction, whose branches the instruction must then have written).
//! Each refused trade is refused by the fold for the same reason. A position that never traded has no
//! trader in the fold: its purses are still `purse_of(deposit)`, its settle is exactly 0 and it gets back its deposit.
mod built;
mod common;
use common::*;
use poleis_futarchy::law::{back_of, decide as fold_decide, decide_at, market_with, pool_result, settle_of, world_of, Deposit, Entry, Receipts, Terms, Which, PRICE, UNIT};
use poleis_roads::error::RoadsError;
use poleis_roads::MOST;
use solana_pubkey::Pubkey;

/// the world's first slot
const S0: u64 = 1_000_000;
/// a day in slots
const DAY: u64 = 86_400_000 / poleis_roads::SLOT_MS;
const U: u64 = UNIT as u64;
const BOARD: u64 = 21;

/// the program's road and every position against the fold's market over `entries`, read at the clock's instant; `at_clock`: the road stands
/// at the clock (a trade taken, a carry that answered true), and its branches themselves are the fold's
fn holds(w: &World, t: &Terms, ros: &[Deposit], entries: &[Entry], road: &Pubkey, owners: &[(Pubkey, u64)], at_clock: bool, what: &str) {
    book(w, BOARD);
    let now = w.now_ms();
    let m = market_with(t, ros, entries, now);
    let r = w.road(road);
    if at_clock {
        assert_eq!((r.pass, r.fail), (m.pass, m.fail), "{what}: the road itself, at {now}, is the fold's road, with no call in between");
    }
    let (mut p, mut f) = (r.pass, r.fail);
    decide_at(t, &mut p, &mut f, now);
    assert_eq!((p, f), (m.pass, m.fail), "{what}: the road's branches, read at {now}, are the fold's");
    for (o, d) in owners {
        let pos = w.position(road, o);
        match m.traders.iter().find(|h| h.key == o.to_string()) {
            Some(h) => assert_eq!((pos.pass, pos.fail, pos.trades as u64, pos.deposit as u128), (h.pass, h.fail, h.trades, h.deposit), "{what}: {o}'s purses are the fold's"),
            None => assert_eq!((pos.pass, pos.fail), (t.purse_of(*d as u128), t.purse_of(*d as u128)), "{what}: {o}, no trader of the fold, holds what it opened with"),
        }
    }
}

/// one `carry` instruction, and whether it answered true: the law's own `carry` on copies of the road's branches, run first, says it, and the
/// instruction must then have written exactly those branches
fn carry_held(w: &mut World, t: &Terms, road: &Pubkey) -> bool {
    let r = w.road(road);
    let (mut p, mut f) = (r.pass, r.fail);
    let answered = poleis_futarchy::law::carry(t, &mut p, &mut f, w.now_ms(), MOST);
    w.ok(&carry(*road), "carry");
    let after = w.road(road);
    assert_eq!((after.pass, after.fail), (p, f), "the carry instruction wrote the law's carry");
    answered
}

#[test]
fn the_program_is_the_fold_trade_by_trade() {
    let mut rng = Rng(0x510E_527F_ADE6_82D1);
    let (mut roads, mut taken, mut refused, mut carries, mut behind, mut checks, mut idle) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u64, 0u32);
    let mut why_seen = std::collections::BTreeMap::<String, u32>::new();
    let (mut credited, mut gap_most, mut staked, mut collected, mut direct, mut trues) = (0u128, 0i128, 0u128, 0u128, 0u32, 0u32);
    let mut earned_total = 0u128;
    for road_n in 0..14u64 {
        let mut w = World::new(S0);
        let auth = w.wallet(100 * SOL);
        let (crediter, judge) = (w.wallet(100 * SOL), w.wallet(SOL));
        w.open_board(auth, BOARD, crediter, judge);
        // a step of 20 s, a minute or five minutes, in slots
        let step_slots = [50u64, 150, 750][rng.below(3) as usize];
        let opens_slot = S0 + 150;
        let closes_slot = opens_slot + (3 * DAY).max(4_000 * step_slots);
        let l = Lay {
            road_id: [road_n as u8 + 1; 32],
            opens_slot,
            closes_slot,
            step_slots,
            max_step_bp: [25u32, 100, 1_000, 10_000][rng.below(4) as usize],
            threshold_bp: [0u32, 300, 1_000][rng.below(3) as usize],
            // half the roads with a fee, so the fold and the program are held equal with one (no draw of the seeded stream: the rest stands)
            fee_bp: [0u32, 30, 0, 100][road_n as usize % 4],
            pool_base: 100 * U + rng.below(10_000_000 * U),
            pool_quote: 100 * U + rng.below(10_000_000 * U),
            judge_by_slot: closes_slot + DAY,
        };
        let t = l.terms();
        w.ok(&credit(crediter, BOARD, auth, l.stake() + rng.below(1_000 * U)), "the house's balance");
        w.ok(&lay(auth, BOARD, &l), "lay");
        let road = road_pda(BOARD, &l.road_id);

        // four owners: three trade, the fourth never does
        let mut owners: Vec<(Pubkey, u64)> = Vec::new();
        for _ in 0..4 {
            let o = w.wallet(SOL);
            let d = loop {
                let d = 10 * U + rng.below(50_000 * U);
                if poleis_futarchy::law::deposit_check(&t, d as u128).is_ok() { break d; }
            };
            w.ok(&credit(crediter, BOARD, o, d + rng.below(1_000 * U)), "credit");
            w.ok(&deposit(o, BOARD, road, d), "deposit");
            owners.push((o, d));
        }
        let ros = roster(&owners);
        let mut entries: Vec<Entry> = Vec::new();
        let mut seq = 0u64;
        holds(&w, &t, &ros, &entries, &road, &owners, false, "after the deposits");

        // the trades, the clock moving by none, a little, or a gap of two to four carries
        let mut slot = opens_slot;
        let mut attempts = 0u32;
        for _ in 0..32 {
            let gap = match rng.below(8) {
                0 | 1 => 0,
                2..=5 => 1 + rng.below(2 * step_slots),
                _ => (MOST * step_slots) * (2 + rng.below(3)) + rng.below(step_slots),
            };
            if slot + gap >= closes_slot { break; }
            slot += gap;
            w.set_slot(slot);
            // sometimes anyone carries the road at the new instant first, a carry that answers true when the road is within MOST of the clock
            if rng.below(4) == 0 {
                let answered = carry_held(&mut w, &t, &road);
                carries += 1;
                direct += answered as u32;
                trues += answered as u32;
                holds(&w, &t, &ros, &entries, &road, &owners, answered, "after a carry by anyone");
                checks += 1;
            }
            let who = rng.below(3) as usize;
            let (o, _) = owners[who];
            let (branch, side) = (rng.below(2) as u8, rng.below(3).min(1) as u8);
            let pos = w.position(&road, &o);
            let purse = if branch == 0 { pos.pass } else { pos.fail };
            let most = if side == 0 { purse.quote } else { purse.base } as u64;
            let (amount, min_out) = match rng.below(12) {
                0 => (most + 1 + rng.below(most + 1), 0),
                1 => (1, 0),
                2 => (1 + rng.below(most / 3 + 1), u64::MAX),
                3 => (0, 0),
                _ => (1 + rng.below(most / 3 + 1), 0),
            };
            let ix = trade(o, road, branch, side, amount, min_out);
            attempts += 1;
            loop {
                match w.run(&ix) {
                    Err(e) if e == code(RoadsError::RoadBehind) => {
                        behind += 1;
                        let answered = carry_held(&mut w, &t, &road);
                        carries += 1;
                        direct += answered as u32;
                        holds(&w, &t, &ros, &entries, &road, &owners, answered, "after a carry");
                        checks += 1;
                    }
                    got => {
                        seq += 1;
                        entries.push(entry(seq, w.now_ms(), &o, law_trade(branch, side, amount, min_out)));
                        let m = market_with(&t, &ros, &entries, w.now_ms());
                        let took = got.is_ok();
                        match got {
                            Ok(_) => {
                                taken += 1;
                                assert!(!m.refused.iter().any(|x| x.seq == seq), "road {road_n}: the program took a trade the fold refused");
                                assert_eq!(w.road(&road).last_slot, w.slot());
                            }
                            Err(e) => {
                                refused += 1;
                                let why = m.refused.iter().find(|x| x.seq == seq).unwrap_or_else(|| panic!("road {road_n}: the program refused {e:?} a trade the fold took")).why;
                                assert_eq!(e, law_code(why), "road {road_n}: refused for the fold's reason, {why}");
                                *why_seen.entry(why.to_string()).or_default() += 1;
                            }
                        }
                        direct += took as u32;
                        holds(&w, &t, &ros, &entries, &road, &owners, took, "after a trade");
                        checks += 1;
                        break;
                    }
                }
            }
        }
        assert!(attempts >= 25, "road {road_n}: {attempts} trades");

        // the close, then carries until the decision
        w.set_slot(closes_slot + rng.below(9_000));
        loop {
            match w.run(&decide(road)) {
                Ok(_) => break,
                Err(e) if e == code(RoadsError::RoadBehind) => {
                    let answered = carry_held(&mut w, &t, &road);
                    carries += 1;
                    direct += answered as u32;
                    holds(&w, &t, &ros, &entries, &road, &owners, answered, "after a carry past the close");
                    checks += 1;
                }
                Err(e) => panic!("road {road_n}: decide {e:?}"),
            }
        }
        let now = w.now_ms();
        let m = market_with(&t, &ros, &entries, now);
        let d = fold_decide(&t, &m, now);
        let world = world_of(&d).unwrap();
        let r = w.road(&road);
        assert_eq!((r.pass, r.fail), (m.pass, m.fail), "road {road_n}: decided, the road is the fold's");
        assert_eq!(r.verdict, if world == Which::Pass { 1 } else { 2 }, "road {road_n}: the verdict");
        holds(&w, &t, &ros, &entries, &road, &owners, true, "after the decision");

        // the judge's receipts, then every settle
        let worth = match rng.below(5) { 0 => 0, 1 => PRICE as u64, _ => rng.below(PRICE as u64 + 1) };
        w.ok(&receipts(judge, road, if world == Which::Pass { 0 } else { 1 }, worth), "receipts");
        let rc = Receipts { world, worth: worth as u128 };
        assert_eq!(w.road(&road).pool_result, pool_result(&t, &m, &d, &rc).unwrap(), "road {road_n}: the pool's result");
        let rows = settle_of(&t, &m, &d, &rc).unwrap();
        for (o, dep) in &owners {
            let before = w.balance(BOARD, o);
            w.ok(&settle(road, *o, BOARD), "settle");
            let after = w.balance(BOARD, o);
            let back = match rows.iter().find(|s| s.key == o.to_string()) {
                Some(s) => back_of(*dep as u128, s.result) as u64,
                None => {
                    idle += 1;
                    *dep
                }
            };
            assert_eq!((after.amount, after.in_roads), (before.amount + back, before.in_roads - dep), "road {road_n}: what came back to {o}");
        }
        assert_eq!(w.road(&road).positions, 0);
        book(&w, BOARD);
        // the house collects what is left; the book closes with nothing in the road
        let left = w.road(&road).left();
        let h0 = w.balance(BOARD, &auth);
        w.ok(&collect(road, auth, BOARD), "collect");
        let h1 = w.balance(BOARD, &auth);
        let back = left.min(l.stake());
        assert_eq!((h1.amount, h1.in_roads), (h0.amount + back, h0.in_roads - l.stake()), "road {road_n}: the staker collects what was left up to its stake");
        earned_total += (left - back) as u128;
        assert_eq!(earned_of(&w.data(&board_pda(BOARD))) as u128, (left - back) as u128, "road {road_n}: the rest is the house's");
        assert_eq!(book(&w, BOARD), 0, "road {road_n}: the book closes with nothing left in the road");
        let gap = left as i128 - back_of(l.stake() as u128, w.road(&road).pool_result) as i128;
        assert!(gap.abs() <= owners.len() as i128, "road {road_n}: collected {left}, back_of(stake, pool_result) off by {gap}");
        gap_most = gap_most.max(gap.abs());
        credited += w.credited[&BOARD];
        staked += l.stake() as u128;
        collected += left as u128;
        roads += 1;
    }
    eprintln!("  the road itself was the fold's road, with no call in between, {direct} times (after every trade taken and every carry that answered true, {trues} of them)");
    assert!(trues > 0, "the sweep met carries that answered true");
    eprintln!("  the book closed after every instruction: {credited} micro credited over the sweep; the stakers staked {staked} and the roads left {collected} (the houses earned {earned_total} of it); the largest gap between what a house collected and back_of(stake, pool_result): {gap_most} micro");
    eprintln!("  {roads} roads, {} trades sent ({taken} taken, {refused} refused by the law as the fold refused them: {why_seen:?}), {behind} refused behind the clock, {carries} carries, {checks} parity checks, {idle} positions that never traded settled at exactly their deposit", taken + refused);
    assert!(roads >= 12 && refused > 0 && carries > roads && idle >= roads);
    assert!(why_seen.len() == 4, "the sweep met the law's four refusals: {why_seen:?}");
}
