//! A PURSE OPENS WITH ITS OWN DEPOSIT: a road may name who trades it and what each deposits. Each purse opens with its own deposit (in the pool's own
//! mix on a road both ways), a key off the roster cannot trade, the settle marks each trader against its own deposit, and what a road gives back, the
//! deposit and the result, is never negative: a purse cannot lose more than its deposit, so nothing is cut off and nothing is forgiven.
#![cfg(feature = "fold")]
use poleis_futarchy::law::*;

const MIN: u64 = 60_000;
/// a road of three days, claims at half a $WARM (a pool of 2000 claims and 1000 $WARM)
fn terms(both_ways: bool) -> Terms { Terms { opens_ms: 1_000_000, closes_ms: 1_000_000 + 3 * 86_400_000, step_ms: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 2000 * UNIT, pool_quote: 1000 * UNIT, budget: 1000 * UNIT, both_ways } }
fn entry(seq: u64, ts: u64, key: &str, branch: Which, side: Side, amount: u128) -> Entry {
    Entry { seq, ts, key: key.into(), seat: key.into(), ok: true, trade: Some(Trade { branch, side, amount, min_out: 0 }) }
}
/// a roster as the law reads one: its lines in the order of their keys
fn roster(lines: &[(&str, u128)]) -> Vec<Deposit> { let mut r: Vec<Deposit> = lines.iter().map(|(k, a)| Deposit { key: k.to_string(), amount: *a }).collect(); r.sort_by(|a, b| a.key.cmp(&b.key)); r }
fn holder<'a>(m: &'a Market, key: &str) -> &'a Holder { m.traders.iter().find(|h| h.key == key).unwrap_or_else(|| panic!("{key} holds a purse")) }

/// xorshift: a seeded sweep with no dependency (tests/law.rs's)
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn below(&mut self, n: u128) -> u128 { (((self.next() as u128) << 64) | self.next() as u128) % n.max(1) } }

#[test]
fn each_purse_opens_with_its_own_deposit_in_the_pools_own_mix() {
    let ros = roster(&[("small", 300 * UNIT), ("large", 3000 * UNIT), ("tiny", 30 * UNIT)]);
    for both_ways in [true, false] {
        let t = terms(both_ways);
        assert_eq!(roster_check(&t, &ros), Ok(()));
        let o = t.opens_ms;
        let es = [entry(1, o + MIN, "small", Which::Pass, Side::Buy, UNIT), entry(2, o + 2 * MIN, "large", Which::Pass, Side::Buy, UNIT), entry(3, o + 3 * MIN, "tiny", Which::Pass, Side::Buy, UNIT)];
        let m = market_with(&t, &ros, &es, t.closes_ms);
        assert_eq!((m.accepted, m.refused.len()), (3, 0));
        for d in &ros {
            let h = holder(&m, &d.key);
            assert_eq!(h.deposit, d.amount, "{}'s purse opened with its own deposit", d.key);
            assert_eq!(h.fail, t.purse_of(d.amount), "{}'s untouched branch is its deposit's purse", d.key);
            assert_eq!(h.fail.quote + h.fail.base, d.amount);
            if both_ways { assert_eq!(h.fail.base, 2 * h.fail.quote, "the pool's own mix: two claims for a $WARM"); }
            else { assert_eq!(h.fail, Purse { quote: d.amount, base: 0 }, "a road one way opens each purse with its deposit as $WARM"); }
        }
        if both_ways {
            assert_eq!(holder(&m, "small").fail, Purse { quote: 100 * UNIT, base: 200 * UNIT });
            assert_eq!(holder(&m, "large").fail, Purse { quote: 1000 * UNIT, base: 2000 * UNIT });
            assert_eq!(holder(&m, "tiny").fail, Purse { quote: 10 * UNIT, base: 20 * UNIT });
        }
    }
}

#[test]
fn a_key_that_is_not_on_the_roster_cannot_trade() {
    let t = terms(true);
    let o = t.opens_ms;
    let ros = roster(&[("a", 500 * UNIT)]);
    let es = [entry(1, o + MIN, "a", Which::Pass, Side::Buy, 10 * UNIT), entry(2, o + 2 * MIN, "z", Which::Pass, Side::Buy, 10 * UNIT), entry(3, o + 3 * MIN, "a", Which::Fail, Side::Buy, 5 * UNIT)];
    let m = market_with(&t, &ros, &es, t.closes_ms);
    assert_eq!(m.refused, vec![Refusal { seq: 2, seat: "z".into(), why: NOT_ON_THE_ROSTER }]);
    let without = market_with(&t, &ros, &[es[0].clone(), es[2].clone()], t.closes_ms);
    assert_eq!((m.pass, m.fail, &m.traders, m.accepted), (without.pass, without.fail, &without.traders, without.accepted), "the market is the market without it");
    // the refusals before it stand in their order: an unsigned or outside entry of a key off the roster is refused for that
    let why = |e: Entry| market_with(&t, &ros, &[e], t.closes_ms).refused[0].why;
    assert_eq!(why(Entry { ok: false, ..es[1].clone() }), UNSIGNED);
    assert_eq!(why(Entry { ts: t.closes_ms, ..es[1].clone() }), OUTSIDE);
    assert_eq!(why(Entry { trade: None, ..es[1].clone() }), NOT_A_TRADE);
    assert_eq!(why(es[1].clone()), NOT_ON_THE_ROSTER);
    // with an empty roster the same entry is taken, and its purse is the budget's
    let open = market_with(&t, &[], &es, t.closes_ms);
    assert_eq!((open.accepted, open.refused.len()), (3, 0));
    let z = holder(&open, "z");
    assert_eq!((z.deposit, z.fail), (t.budget, t.purse_of(t.budget)));
    assert_eq!(open, market_of(&t, &es, t.closes_ms), "market_of is market_with an empty roster");
}

#[test]
fn sellers_whose_deposits_match_a_whales_undo_its_lift() {
    let t = terms(true);
    let o = t.opens_ms;
    let open = spot(t.pool_base, t.pool_quote).unwrap();
    for (seller, fails) in [(4500 * UNIT, true), (450 * UNIT, false)] {
        let ros = roster(&[("whale", 9000 * UNIT), ("s1", seller), ("s2", seller)]);
        assert_eq!(roster_check(&t, &ros), Ok(()));
        let (w, s) = (t.purse_of(9000 * UNIT), t.purse_of(seller));
        let es = [entry(1, o + MIN, "whale", Which::Pass, Side::Buy, w.quote), entry(2, o + 2 * MIN, "s1", Which::Pass, Side::Sell, s.base), entry(3, o + 3 * MIN, "s2", Which::Pass, Side::Sell, s.base)];
        let lifted = market_with(&t, &ros, &es[..1], t.closes_ms).pass.spot();
        let m = market_with(&t, &ros, &es, t.closes_ms);
        assert_eq!((m.accepted, m.refused.len()), (3, 0));
        let d = decide(&t, &m, t.closes_ms);
        let (end, avg, needs) = (m.pass.spot(), d.twap_pass.unwrap(), d.needs.unwrap());
        eprintln!("  a whale of 9000 buys with its {} $WARM: PASS from {} to {}; two sellers of {} sell {} claims each: PASS ends at {}, its average {} against the {} it needs; {:?}",
            w.quote / UNIT, price(open), price(lifted), seller / UNIT, s.base / UNIT, price(end), price(avg), price(needs), d.state);
        if fails {
            assert!(end < open, "sellers of 4500 each: PASS ends at {} under where it opened, {}", price(end), price(open));
            assert_eq!(d.state, State::Failed, "and the road fails: PASS's average {} against the {} it needs", price(avg), price(needs));
        } else {
            assert!(end > open, "sellers of 450 each: PASS stays lifted at {}, over {}", price(end), price(open));
            assert_eq!(d.state, State::Passed, "and the road passes: PASS's average {} against the {} it needs", price(avg), price(needs));
        }
    }
}
fn price(p: u128) -> String { format!("{}.{:06}", p / PRICE, p % PRICE / 1_000_000) }

#[test]
fn the_settle_marks_each_trader_against_its_own_deposit() {
    let t = terms(true);
    let o = t.opens_ms;
    let ros = roster(&[("a", 3000 * UNIT), ("b", 300 * UNIT), ("c", 30 * UNIT)]);
    let es = [entry(1, o + MIN, "a", Which::Pass, Side::Buy, 300 * UNIT), entry(2, o + 2 * MIN, "b", Which::Pass, Side::Buy, 50 * UNIT), entry(3, o + 3 * MIN, "c", Which::Fail, Side::Buy, 5 * UNIT),
        entry(4, o + 4 * MIN, "b", Which::Pass, Side::Sell, 120 * UNIT)];
    let m = market_with(&t, &ros, &es, t.closes_ms);
    assert_eq!((m.accepted, m.refused.len()), (4, 0));
    let d = decide(&t, &m, t.closes_ms);
    assert_eq!(world_of(&d), Some(Which::Pass), "PASS was bid higher");
    for worth in [0, PRICE / 3, PRICE] {
        let r = Receipts { world: Which::Pass, worth };
        let counts = settle_of(&t, &m, &d, &r).unwrap();
        let pool = pool_result(&t, &m, &d, &r).unwrap();
        let c = counts.iter().find(|s| s.key == "c").unwrap();
        assert_eq!((c.result, back_of(30 * UNIT, c.result)), (0, 30 * UNIT), "c traded only in the world that did not happen: it scores 0 and gets back its deposit");
        for s in &counts {
            let dep = holder(&m, &s.key).deposit;
            assert_eq!(back_of(dep, s.result) as i128, dep as i128 + s.result, "at a worth of {worth}: {} gets back its deposit {dep} and its result {}", s.key, s.result);
            // its own deposit, not the budget: the mark against the budget's purse would differ
            let own = t.purse_of(dep);
            let p = holder(&m, &s.key).pass;
            assert_eq!(s.result, (p.quote + worth_of(p.base, worth)) as i128 - (own.quote + worth_of(own.base, worth)) as i128);
        }
        let sum: i128 = counts.iter().map(|s| s.result).sum::<i128>() + pool;
        assert!(sum.abs() <= counts.len() as i128, "at a worth of {worth}: the results and the pool's sum to {sum} micro");
    }
}

#[test]
fn no_purse_loses_more_than_its_deposit_and_two_wallets_that_bet_against_each_other_end_where_they_began() {
    // (a) a seeded sweep of rostered roads
    let mut r = Rng(0x2F1C_7A3B_9D04_E651);
    let (mut roads, mut both, mut zeros, mut all_lost, mut worst_ppm, mut traders) = (0u32, 0u32, 0u32, 0u32, 0u128, 0u64);
    let keys = ["a", "b", "c", "d", "e"];
    for road in 0..200 {
        let both_ways = r.below(2) == 0;
        let t = Terms { opens_ms: 5_000, closes_ms: 5_000 + (2 + r.below(600) as u64) * MIN, step_ms: [7_000, 20_000, MIN, 5 * MIN][r.below(4) as usize], max_step_bp: [1, 25, 100, 10_000][r.below(4) as usize],
            threshold_bp: [0, 300, 10_000][r.below(3) as usize], fee_bp: 0, pool_base: (1000 + r.below(10_000_000)) * UNIT, pool_quote: (1000 + r.below(10_000_000)) * UNIT, budget: 1000 * UNIT, both_ways };
        if t.check().is_err() { continue; }
        let mut ros: Vec<Deposit> = keys.iter().chain(["all-in-pass", "all-in-fail"].iter()).map(|k| Deposit { key: k.to_string(), amount: UNIT + r.below(1_000_000 * UNIT - UNIT + 1) }).collect();
        ros.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(roster_check(&t, &ros), Ok(()), "road {road}");
        let dep = |k: &str| ros.iter().find(|d| d.key == k).unwrap().amount;
        let span = t.closes_ms - t.opens_ms;
        let mut ts = t.opens_ms;
        let mut es: Vec<Entry> = (0..60).map(|i| {
            ts += r.below(span as u128 / 60) as u64;
            let key = keys[r.below(5) as usize];
            let side = if r.below(3) == 0 { Side::Sell } else { Side::Buy };
            let p = t.purse_of(dep(key));
            let most = if side == Side::Sell { p.base } else { p.quote };
            let amount = if r.below(4) == 0 { 1 + r.below(50) } else { 1 + r.below(most / 3 + 1) };
            entry(i + 1, ts, key, if r.below(2) == 0 { Which::Pass } else { Which::Fail }, side, amount)
        }).collect();
        // one more trader buys with its whole $WARM in each world, at the last instant of the window
        es.push(entry(61, t.closes_ms - 1, "all-in-pass", Which::Pass, Side::Buy, t.purse_of(dep("all-in-pass")).quote));
        es.push(entry(62, t.closes_ms - 1, "all-in-fail", Which::Fail, Side::Buy, t.purse_of(dep("all-in-fail")).quote));
        let m = market_with(&t, &ros, &es, t.closes_ms);
        let d = decide(&t, &m, t.closes_ms);
        let world = world_of(&d).unwrap();
        let rc = Receipts { world, worth: if r.below(6) == 0 { 0 } else { r.below(PRICE + 1) } };
        let counts = settle_of(&t, &m, &d, &rc).unwrap();
        let pool = pool_result(&t, &m, &d, &rc).unwrap();
        let n = m.traders.len() as i128;
        let sum: i128 = counts.iter().map(|s| s.result).sum::<i128>() + pool;
        assert!(sum.abs() <= n, "road {road}: the traders' results and the pool's sum to {sum} micro, past the rounding of {n} traders");
        for s in &counts {
            let h = holder(&m, &s.key);
            assert_eq!(h.deposit, dep(&s.key));
            assert!(h.deposit as i128 + s.result >= 0, "road {road}: {} lost {} micro, more than its deposit of {}", s.key, -s.result, h.deposit);
            assert_eq!(back_of(h.deposit, s.result) as i128, h.deposit as i128 + s.result);
            if s.result < 0 { worst_ppm = worst_ppm.max((-s.result) as u128 * 1_000_000 / h.deposit); }
            traders += 1;
        }
        let happened = if world == Which::Pass { "all-in-pass" } else { "all-in-fail" };
        if !both_ways && rc.worth == 0 {
            if let Some(s) = counts.iter().find(|s| s.key == happened) {
                assert_eq!((back_of(dep(happened), s.result), s.result), (0, -(dep(happened) as i128)), "road {road}: receipts of 0 on a road one way: its whole deposit is lost and not a micro more");
                all_lost += 1;
            }
        }
        roads += 1; both += both_ways as u32; zeros += (rc.worth == 0) as u32;
    }
    eprintln!("  {roads} rostered roads settled ({both} both ways, {zeros} on receipts of 0), {traders} traders; {all_lost} all-in traders of the world that happened, on a road one way with receipts of 0, got back exactly 0; the worst loss was {worst_ppm} ppm of a deposit");
    assert!(roads >= 120 && both > 30 && roads - both > 30, "the sweep met roads both ways and one way: {both} of {roads}");
    assert!(all_lost >= 3, "the sweep met at least three traders who lost their whole deposit and not a micro more: {all_lost}");
    assert!(worst_ppm <= 1_000_000);

    // (b) two keys of the same deposit bet against each other on a road both ways, claims at half a $WARM: what one buys the other sells
    let t = terms(true);
    let ros = roster(&[("first", 600 * UNIT), ("second", 600 * UNIT)]);
    let at = t.opens_ms + MIN;
    for x in [UNIT, 7_654_321, 13 * UNIT + 1, 40 * UNIT] {
        let m1 = market_with(&t, &ros, &[entry(1, at, "first", Which::Fail, Side::Buy, x)], t.closes_ms);
        let y = holder(&m1, "first").fail.base - t.purse_of(600 * UNIT).base;
        let m = market_with(&t, &ros, &[entry(1, at, "first", Which::Fail, Side::Buy, x), entry(2, at, "second", Which::Fail, Side::Sell, y)], t.closes_ms);
        assert_eq!((m.accepted, m.refused.len()), (2, 0));
        let d = decide(&t, &m, t.closes_ms);
        assert_eq!(world_of(&d), Some(Which::Fail), "the two traded in the world that happened");
        for worth in [0, PRICE / 3, PRICE] {
            let rc = Receipts { world: Which::Fail, worth };
            let counts = settle_of(&t, &m, &d, &rc).unwrap();
            let pool = pool_result(&t, &m, &d, &rc).unwrap();
            let (r1, r2) = (counts.iter().find(|s| s.key == "first").unwrap().result, counts.iter().find(|s| s.key == "second").unwrap().result);
            eprintln!("  first buys with {x} micro and gets {y} claims, second sells them; at a worth of {worth}: {r1} and {r2} micro, the pool {pool}");
            assert!((-4..=1).contains(&(r1 + r2)), "x {x}, worth {worth}: the two results sum to {} micro", r1 + r2);
            assert!(pool >= 0 && (pool as u128) * t.pool_base < t.pool_quote + x + t.pool_base, "x {x}: the pool keeps the dust of its two floors, {pool} micro, under (pool quote + {x}) / pool base + 1");
            let back = back_of(600 * UNIT, r1) + back_of(600 * UNIT, r2);
            assert!(back.abs_diff(1200 * UNIT) <= 4, "x {x}, worth {worth}: the two get back {back} for their two deposits of 600 $WARM: nothing is farmed");
        }
    }
}

#[test]
fn a_roster_the_law_cannot_carry_is_refused() {
    let (both, one) = (terms(true), terms(false));
    // the last: a purse of every micro u128 holds, beside a trader who lifts the price it then sells into (its $WARM would pass u128 in the fold)
    let out_of_order = vec![Deposit { key: "b".into(), amount: 1000 * UNIT }, Deposit { key: "a".into(), amount: 1000 * UNIT }];
    assert_eq!(roster_check(&one, &out_of_order), Err("the roster's keys are not in their order, or one is named twice"));
    assert_eq!(roster_check(&one, &roster(&[("a", UNIT), ("a", 2 * UNIT)])), Err("the roster's keys are not in their order, or one is named twice"));
    let bad = [roster(&[("a", UNIT), ("a", 2 * UNIT)]), roster(&[("", UNIT)]), roster(&[("a", 0)]), roster(&[("a", AMOUNT_MAX + 1)]), roster(&[("a", u128::MAX), ("b", 1000 * UNIT)]), out_of_order];
    for ros in &bad {
        assert!(roster_check(&both, ros).is_err() && roster_check(&one, ros).is_err(), "{ros:?} is refused");
    }
    assert_eq!(roster_check(&both, &roster(&[("a", 1)])), Err("a deposit both ways is too small to hold both $WARM and claims"), "a deposit of one micro both ways");
    assert_eq!(roster_check(&one, &roster(&[("a", 1)])), Ok(()), "is carried one way");
    assert_eq!(roster_check(&both, &roster(&[("a", AMOUNT_MAX), ("b", 3)])), Ok(()), "a good roster is carried");
    assert_eq!(roster_check(&both, &[]), Ok(()), "an empty roster is carried");
    // the fold does not check a roster, and a bad one panics nothing
    let o = both.opens_ms;
    let es = [entry(1, o + MIN, "a", Which::Pass, Side::Buy, UNIT), entry(2, o + 2 * MIN, "b", Which::Pass, Side::Buy, 500 * UNIT), entry(3, o + 3 * MIN, "a", Which::Pass, Side::Sell, UNIT),
        entry(4, o + 4 * MIN, "", Which::Fail, Side::Buy, UNIT)];
    for ros in bad.iter().chain([roster(&[("a", 1)])].iter()) {
        for t in [both, one] {
            let m = market_with(&t, ros, &es, t.closes_ms);
            assert_eq!(m.accepted as usize + m.refused.len(), es.len());
            let d = decide(&t, &m, t.closes_ms);
            let _ = score_of(&t, &m, &d);
            let rc = Receipts { world: world_of(&d).unwrap(), worth: PRICE / 2 };
            for s in settle_of(&t, &m, &d, &rc).unwrap() { let _ = back_of(holder(&m, &s.key).deposit, s.result); }
        }
    }
}

#[test]
fn the_fold_finds_a_key_among_a_thousand_by_halving() {
    // a roster of a thousand keys in their order, each with its own deposit: 1000 $WARM and its own number
    let t = terms(true);
    let ros: Vec<Deposit> = (0..1000u128).map(|i| Deposit { key: format!("k{i:04}"), amount: (1000 + i) * UNIT }).collect();
    assert_eq!(roster_check(&t, &ros), Ok(()));
    let o = t.opens_ms;
    let asked = ["k0000", "k0001", "k0499", "k0500", "k0998", "k0999"];
    let mut es: Vec<Entry> = asked.iter().enumerate().map(|(i, k)| entry(i as u64 + 1, o + (i as u64 + 1) * MIN, k, Which::Pass, Side::Buy, UNIT)).collect();
    // keys that stand before, between and after the roster's, and one that only begins like one of them
    for (i, k) in ["a", "k0499x", "k1000", "k", "z"].iter().enumerate() { es.push(entry(100 + i as u64, o + (10 + i as u64) * MIN, k, Which::Pass, Side::Buy, UNIT)); }
    let m = market_with(&t, &ros, &es, t.closes_ms);
    assert_eq!((m.accepted, m.refused.len()), (6, 5));
    assert!(m.refused.iter().all(|r| r.why == NOT_ON_THE_ROSTER), "{:?}", m.refused);
    for k in asked {
        let n: u128 = k[1..].parse().unwrap();
        assert_eq!((holder(&m, k).deposit, holder(&m, k).fail), ((1000 + n) * UNIT, t.purse_of((1000 + n) * UNIT)), "{k} opened with its own deposit");
    }
}
