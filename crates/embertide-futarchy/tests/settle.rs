//! THE WORLD THAT HAPPENED SETTLES ON ITS RECEIPTS: `score_of` marks a purse at the pool's own average, so whoever lifts a price is paid by the price
//! it made; `settle_of` marks it at the judge's receipts, in the world that happened, against what it opened with. And on a road that can be bet both
//! ways a purse opens with its deposit in the pool's own mix, so it can sell first and a lifted price meets sellers.
#![cfg(feature = "fold")]
use embertide_futarchy::law::*;

const MIN: u64 = 60_000;
fn terms(both_ways: bool) -> Terms { Terms { opens_ms: 1_000_000, closes_ms: 1_000_000 + 3 * 86_400_000, step_ms: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 1000 * UNIT, pool_quote: 1000 * UNIT, budget: 1000 * UNIT, both_ways } }
fn entry(seq: u64, ts: u64, key: &str, branch: Which, side: Side, amount: u128) -> Entry {
    Entry { seq, ts, key: key.into(), seat: key.into(), ok: true, trade: Some(Trade { branch, side, amount, min_out: 0 }) }
}

#[test]
fn the_old_mark_pays_whoever_lifts_a_price_and_the_settle_charges_it() {
    let t = terms(false);
    let es = [entry(1, t.opens_ms + 60 * MIN, "whale", Which::Pass, Side::Buy, 900 * UNIT)];
    let m = market_of(&t, &es, t.closes_ms);
    let d = decide(&t, &m, t.closes_ms);
    assert_eq!(d.state, State::Passed, "the whale's lift held for the window passes the road");
    let old = score_of(&t, &m, &d)[0].result;
    let new = settle_of(&t, &m, &d, &Receipts { world: Which::Pass, worth: PRICE / 10 }).unwrap()[0].result;
    eprintln!("  the whale: {old} micro $WARM by the pool's own average, {new} micro $WARM settled on receipts of a tenth of a $WARM");
    assert!(old > 0, "the old mark pays the whale for the price it made: {old}");
    assert!(new < -700 * UNIT as i128, "the settle charges it: {new}");
}

#[test]
fn every_trader_can_sell_so_whoever_lifts_a_price_meets_sellers() {
    let mut es = vec![entry(1, 1_000_000 + 60 * MIN, "whale", Which::Pass, Side::Buy, 900 * UNIT)];
    for i in 0..10u64 { es.push(entry(2 + i, 1_000_000 + (2 + i) * 60 * MIN, &format!("seller-{i}"), Which::Pass, Side::Sell, 300 * UNIT)); }
    // a deposit of 2000 opens both ways with 1000 of $WARM and 1000 claims, the purse this market had before the deposit: the same market
    let t = Terms { budget: 2000 * UNIT, ..terms(true) };
    assert_eq!(t.purse_of(t.budget), Purse { quote: 1000 * UNIT, base: 1000 * UNIT });
    let m = market_of(&t, &es, t.closes_ms);
    assert_eq!((m.accepted, m.refused.len()), (11, 0), "every seller opened with claims to sell");
    let d = decide(&t, &m, t.closes_ms);
    assert_eq!(d.state, State::Failed, "the lift met ten sellers and the road fails: PASS's average {:?}, it needs {:?}", d.twap_pass, d.needs);
    let t = Terms { budget: 2000 * UNIT, ..terms(false) };
    let m = market_of(&t, &es, t.closes_ms);
    assert_eq!(m.refused.iter().map(|r| r.why).collect::<Vec<_>>(), vec![NOT_ENOUGH; 10], "on a road one way nobody can sell what it did not first buy");
    assert_eq!(decide(&t, &m, t.closes_ms).state, State::Passed, "and the lift passes the road");
}

#[test]
fn a_road_both_ways_opens_with_the_pools_own_mix_or_is_no_road() {
    let t = terms(true);
    assert_eq!((t.check(), t.purse_of(1000 * UNIT)), (Ok(()), Purse { quote: 500 * UNIT, base: 500 * UNIT }), "claims at one $WARM: a deposit of 1000 opens with 500 of $WARM and 500 claims");
    assert_eq!(terms(false).purse_of(1000 * UNIT), Purse { quote: 1000 * UNIT, base: 0 }, "a road one way opens with $WARM alone");
    let half = Terms { pool_base: 2000 * UNIT, ..terms(true) };
    assert_eq!(half.purse_of(1000 * UNIT), Purse { quote: 333_333_333, base: 666_666_667 }, "claims at half a $WARM: the same deposit opens with 333.333333 of $WARM and 666.666667 claims");
    let m = market_of(&t, &[entry(1, t.opens_ms + MIN, "a", Which::Fail, Side::Buy, UNIT)], t.closes_ms);
    assert_eq!(m.traders[0].pass, Purse { quote: 500 * UNIT, base: 500 * UNIT }, "the mix stands in the world it did not trade in");
    assert_eq!(m.traders[0].deposit, t.budget, "with no roster every purse deposits the budget");
    // quote + base is the deposit, to the micro, whatever the pool and the deposit
    let mut r = Rng(0x5851_F42D_4C95_7F2D);
    for _ in 0..20_000 {
        let (pb, pq, d) = (1 + r.below(AMOUNT_MAX), 1 + r.below(AMOUNT_MAX), r.below(AMOUNT_MAX + 1));
        for both_ways in [false, true] {
            let p = Terms { pool_base: pb, pool_quote: pq, both_ways, ..terms(true) }.purse_of(d);
            assert_eq!(p.quote + p.base, d, "a deposit of {d} in a pool of {pb} and {pq}, both ways {both_ways}");
            if both_ways { assert_eq!(p.quote, d * pq / (pq + pb), "the pool's own mix, floored"); }
        }
    }
    // a deposit of one micro both ways cannot hold both $WARM and claims: no road both ways, and a road one way
    for no in [Terms { budget: 1, ..terms(true) }, Terms { budget: 1, ..half }] {
        assert_eq!(no.check(), Err("a deposit both ways is too small to hold both $WARM and claims"), "a purse of {:?} is not carried", no.purse_of(1));
        assert_eq!(Terms { both_ways: false, ..no }.check(), Ok(()), "the same road one way is carried");
    }
}

#[test]
fn a_purses_whole_claim_lowers_a_price_by_the_factor_its_whole_warm_lifts_it() {
    for (pool_base, pool_quote, budget) in [(1000 * UNIT, 1000 * UNIT, 1000 * UNIT), (2000 * UNIT, 1000 * UNIT, 250 * UNIT), (700 * UNIT, 4100 * UNIT, 90 * UNIT), (1_000_000 * UNIT, 300 * UNIT, 200 * UNIT)] {
        let t = Terms { pool_base, pool_quote, budget, ..terms(true) };
        t.check().unwrap();
        let (p, open, at) = (t.purse_of(budget), spot(pool_base, pool_quote).unwrap(), t.opens_ms + MIN);
        assert_eq!((p.quote, p.quote + p.base), (budget * pool_quote / (pool_quote + pool_base), budget), "the pool's own mix");
        // a bull buys with all of its purse's $WARM, a bear of the same deposit sells all of its claims
        let bull = entry(1, at, "bull", Which::Pass, Side::Buy, p.quote);
        let bear = |seq| entry(seq, at + MIN, "bear", Which::Pass, Side::Sell, p.base);
        let lifted = market_of(&t, &[bull.clone()], t.closes_ms).pass.spot();
        let lowered = market_of(&t, &[bear(1)], t.closes_ms).pass.spot();
        assert!(lifted > open && lowered < open);
        let (got, want) = (lifted * lowered, open * open);
        assert!(got.abs_diff(want) * 1_000_000 <= want, "a pool of {pool_base} and {pool_quote}: lifted to {lifted} and lowered to {lowered} from {open}, to a millionth the same factor");
        // and a bear of the same deposit that sells after the bull bought more than undoes it
        let after = market_of(&t, &[bull, bear(2)], t.closes_ms);
        assert_eq!(after.accepted, 2);
        assert!(after.pass.spot() < open, "the bull lifted {open} to {lifted} and a bear of its size brought it to {}", after.pass.spot());
    }
}

#[test]
fn the_world_that_did_not_happen_counts_for_nothing() {
    let t = terms(false);
    let o = t.opens_ms;
    let es = [entry(1, o + MIN, "a", Which::Pass, Side::Buy, 300 * UNIT), entry(2, o + 2 * MIN, "b", Which::Pass, Side::Buy, 50 * UNIT),
        entry(3, o + 3 * MIN, "a", Which::Fail, Side::Buy, 20 * UNIT), entry(4, o + 4 * MIN, "b", Which::Fail, Side::Buy, 10 * UNIT)];
    let m = market_of(&t, &es, t.closes_ms);
    let d = decide(&t, &m, t.closes_ms);
    assert_eq!((d.state, world_of(&d)), (State::Passed, Some(Which::Pass)));
    let pass_only: Vec<Entry> = es.iter().filter(|e| e.trade.unwrap().branch == Which::Pass).cloned().collect();
    let m2 = market_of(&t, &pass_only, t.closes_ms);
    let d2 = decide(&t, &m2, t.closes_ms);
    assert_eq!(d2.state, State::Passed);
    let r = Receipts { world: Which::Pass, worth: PRICE / 3 };
    // a Score also carries its trader's count of trades, which the FAIL trades add to: the results are what must not change
    let results = |m: &Market, d: &Decision| settle_of(&t, m, d, &r).unwrap().into_iter().map(|s| (s.key, s.result)).collect::<Vec<_>>();
    assert_eq!(results(&m, &d), results(&m2, &d2), "the FAIL trades count for nothing in the PASS world");
    assert_eq!(pool_result(&t, &m, &d, &r), pool_result(&t, &m2, &d2, &r));
    assert_eq!(settle_of(&t, &m, &d, &Receipts { world: Which::Fail, worth: PRICE / 3 }), Err(OTHER_WORLD));
    assert_eq!(pool_result(&t, &m, &d, &Receipts { world: Which::Fail, worth: PRICE / 3 }), Err(OTHER_WORLD));
}

/// xorshift: a seeded sweep with no dependency (tests/law.rs's)
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn below(&mut self, n: u128) -> u128 { (((self.next() as u128) << 64) | self.next() as u128) % n.max(1) } }

#[test]
fn what_the_traders_win_the_pool_loses_to_the_rounding() {
    let mut r = Rng(0x2545_F491_4F6C_DD1D);
    let (mut roads, mut both, mut untouched, mut sold) = (0u32, 0u32, 0u32, 0u64);
    for road in 0..160 {
        let t = Terms { opens_ms: 5_000, closes_ms: 5_000 + (2 + r.below(600) as u64) * MIN, step_ms: [7_000, 20_000, MIN, 5 * MIN][r.below(4) as usize], max_step_bp: [1, 25, 100, 10_000][r.below(4) as usize],
            threshold_bp: [0, 300, 10_000][r.below(3) as usize], fee_bp: 0, pool_base: (1 + r.below(999_999_999)) * UNIT, pool_quote: (1 + r.below(999_999_999)) * UNIT, budget: (1 + r.below(999_999)) * UNIT, both_ways: r.below(3) != 0 };
        if t.check().is_err() { continue; }
        let span = t.closes_ms - t.opens_ms;
        let mut ts = t.opens_ms;
        let opened = t.purse_of(t.budget);
        // two traders who each trade in one world only: in the world that happened one of them holds what it opened with
        let mut es = vec![entry(1, ts, "p", Which::Pass, Side::Buy, (opened.quote / 10).max(1)), entry(2, ts, "f", Which::Fail, Side::Buy, (opened.quote / 10).max(1))];
        es.extend((0..60).map(|i| {
            ts += r.below(span as u128 / 60) as u64;
            let key = ["a", "b", "c", "d", "e"][r.below(5) as usize];
            let side = if r.below(3) == 0 { Side::Sell } else { Side::Buy };
            let most = if side == Side::Sell { opened.base } else { opened.quote };
            let amount = if r.below(4) == 0 { 1 + r.below(50) } else { 1 + r.below(most / 3 + 1) };
            entry(i + 3, ts, key, if r.below(2) == 0 { Which::Pass } else { Which::Fail }, side, amount)
        }));
        let m = market_of(&t, &es, t.closes_ms);
        let d = decide(&t, &m, t.closes_ms);
        let rc = Receipts { world: world_of(&d).unwrap(), worth: r.below(PRICE + 1) };
        let counts = settle_of(&t, &m, &d, &rc).unwrap();
        let pool = pool_result(&t, &m, &d, &rc).unwrap();
        // one floor a trader and one for the pool on each side of a sum that is the same on both: the two sides differ by less than n + 1
        let n = m.traders.len() as i128;
        let sum: i128 = counts.iter().map(|s| s.result).sum::<i128>() + pool;
        assert!(sum.abs() <= n, "road {road}: the traders' results and the pool's sum to {sum} micro, past the rounding of {n} traders");
        for s in &counts {
            assert!(s.result >= -((opened.quote + worth_of(opened.base, rc.worth)) as i128), "road {road}: {} lost {} micro, more than it opened with", s.key, s.result);
            assert!(t.budget as i128 + s.result >= 0, "road {road}: {} lost {} micro, more than its deposit of {}", s.key, s.result, t.budget);
            assert_eq!(back_of(t.budget, s.result) as i128, t.budget as i128 + s.result, "road {road}: what comes back to {} is its deposit and its result, never cut off", s.key);
            let h = m.traders.iter().find(|h| h.key == s.key).unwrap();
            if (if rc.world == Which::Pass { h.pass } else { h.fail }) == opened { assert_eq!(s.result, 0, "road {road}: {} holds what it opened with", s.key); untouched += 1; }
        }
        roads += 1; both += t.both_ways as u32;
        sold += es.iter().filter(|e| e.trade.unwrap().side == Side::Sell && !m.refused.iter().any(|x| x.seq == e.seq)).count() as u64;
    }
    eprintln!("  {roads} markets settled, {both} of them both ways, {sold} sells taken; {untouched} traders held in the world that happened what they opened with");
    assert!(roads > 100 && both > 50 && roads - both > 20, "the sweep met roads both ways and roads one way: {both} of {roads}");
    assert!(sold > 200, "the sweep met sells: {sold}");
    assert!(untouched > 0, "the sweep met traders who hold in the world that happened what they opened with");
}

#[test]
fn a_holding_is_marked_exactly_however_large() {
    let mut r = Rng(0xD1B5_4A32_D192_ED03);
    for _ in 0..20_000 {
        let (h, w) = (r.below(2 * AMOUNT_MAX + 1), r.below(PRICE + 1));
        assert_eq!(worth_of(h, w), h * w / PRICE, "{h} of the claim at {w}");
    }
    assert_eq!(worth_of(7 * PRICE + 5, PRICE / 2), 7 * PRICE / 2 + 2, "the whole multiples of PRICE and the remainder are marked apart and floored once");
    assert_eq!(worth_of(u128::MAX, PRICE), u128::MAX, "at one $WARM a holding is worth itself: no product leaves u128");
    assert_eq!(worth_of(u128::MAX, PRICE / 2), u128::MAX / 2, "and at half a $WARM, half");
    assert_eq!(worth_of(u128::MAX, 0), 0);
}

#[test]
fn an_open_road_and_a_worth_over_one_are_refused() {
    let t = terms(true);
    let m = market_of(&t, &[entry(1, t.opens_ms + MIN, "a", Which::Pass, Side::Sell, 100 * UNIT)], t.closes_ms);
    let open = decide(&t, &m, t.closes_ms - 1);
    assert_eq!(world_of(&open), None, "no world has happened while the road is open");
    assert_eq!(settle_of(&t, &m, &open, &Receipts { world: Which::Fail, worth: PRICE / 2 }), Err(STILL_OPEN));
    assert_eq!(pool_result(&t, &m, &open, &Receipts { world: Which::Fail, worth: PRICE / 2 }), Err(STILL_OPEN));
    assert_eq!(settle_of(&t, &m, &open, &Receipts { world: Which::Fail, worth: PRICE + 1 }), Err(WORTH_RANGE), "the range is read first");
    let d = decide(&t, &m, t.closes_ms);
    let w = world_of(&d).unwrap();
    assert_eq!(settle_of(&t, &m, &d, &Receipts { world: w, worth: PRICE + 1 }), Err(WORTH_RANGE));
    assert_eq!(pool_result(&t, &m, &d, &Receipts { world: w, worth: PRICE + 1 }), Err(WORTH_RANGE));
    for worth in [0, PRICE] {
        assert!(settle_of(&t, &m, &d, &Receipts { world: w, worth }).is_ok(), "a worth of {worth} is taken");
        assert!(pool_result(&t, &m, &d, &Receipts { world: w, worth }).is_ok());
    }
}

#[test]
fn at_the_pools_own_average_the_settle_is_the_old_mark() {
    for both_ways in [false, true] {
        let t = Terms { pool_base: 2000 * UNIT, pool_quote: 1000 * UNIT, ..terms(both_ways) };
        let o = t.opens_ms;
        let es = [entry(1, o + MIN, "a", Which::Pass, Side::Buy, 200 * UNIT), entry(2, o + 2 * MIN, "b", Which::Fail, Side::Buy, 20 * UNIT), entry(3, o + 3 * MIN, "b", Which::Pass, Side::Buy, 50 * UNIT),
            entry(4, o + 4 * MIN, "c", Which::Pass, Side::Sell, 100 * UNIT)];
        let m = market_of(&t, &es, t.closes_ms);
        assert_eq!(m.accepted, if both_ways { 4 } else { 3 }, "the sell is taken only on the road both ways");
        let d = decide(&t, &m, t.closes_ms);
        assert_eq!(d.state, State::Passed);
        let twap = d.twap_pass.unwrap();
        assert!(twap <= PRICE, "the winning average {twap} is under one $WARM");
        assert_eq!(settle_of(&t, &m, &d, &Receipts { world: Which::Pass, worth: twap }), Ok(score_of(&t, &m, &d)), "both ways {both_ways}: the old mark is the settle at the price the pool made");
    }
}

#[test]
fn on_a_road_both_ways_the_old_mark_never_marks_a_claim_over_one_warm() {
    // a seller sells all its claims, then two whales lift PASS far over one $WARM a claim and hold it: the pool's own average ends over one
    let o = terms(true).opens_ms;
    let es = [entry(1, o + MIN, "seller", Which::Pass, Side::Sell, 1000 * UNIT), entry(2, o + 2 * MIN, "whale", Which::Pass, Side::Buy, 1000 * UNIT), entry(3, o + 3 * MIN, "whale-2", Which::Pass, Side::Buy, 1000 * UNIT)];
    let t = Terms { budget: 2000 * UNIT, ..terms(true) };
    assert_eq!(t.purse_of(t.budget), Purse { quote: 1000 * UNIT, base: 1000 * UNIT });
    let m = market_of(&t, &es, t.closes_ms);
    assert_eq!((m.accepted, m.refused.len()), (3, 0));
    let d = decide(&t, &m, t.closes_ms);
    let avg = d.twap_pass.unwrap();
    assert!(d.state == State::Passed && avg > 5 * PRICE, "PASS's own average ends at {avg}, over five $WARM a claim");
    let scores = score_of(&t, &m, &d);
    // at the pool's own average the seller, who holds 1500 of $WARM and no claim, would be marked against 1000 claims at over five $WARM each
    for s in &scores { assert!(t.budget as i128 + s.result >= 0, "{} is marked {} micro: no mark shows a loss over a deposit of {}", s.key, s.result, t.budget); }
    let seller = scores.iter().find(|s| s.key == "seller").unwrap().result;
    assert!(seller == -500 * UNIT as i128, "the seller got 500 of $WARM for 1000 claims marked at one $WARM each: {seller}");
    assert_eq!(Ok(scores), settle_of(&t, &m, &d, &Receipts { world: Which::Pass, worth: PRICE }), "over one $WARM the old mark is the settle at one $WARM");
    // a road one way keeps the pool's own average as its mark, whatever it is: the law as it was
    let t = Terms { budget: 2000 * UNIT, ..terms(false) };
    let m = market_of(&t, &es, t.closes_ms);
    assert_eq!((m.accepted, m.refused.len()), (2, 1), "one way the seller has nothing to sell");
    let d = decide(&t, &m, t.closes_ms);
    let (avg, whale) = (d.twap_pass.unwrap(), m.traders.iter().find(|h| h.key == "whale").unwrap().pass);
    assert!(avg > PRICE);
    assert_eq!(score_of(&t, &m, &d).iter().find(|s| s.key == "whale").unwrap().result, (whale.quote + whale.base * avg / PRICE) as i128 - t.budget as i128, "one way the mark is the pool's own average, {avg}");
}
