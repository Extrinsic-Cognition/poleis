//! ONE LAW FOR THE FOLD AND THE CHAIN: the core (law.rs) carries a road one trade at a time, as a program on a chain does, and gets the fold's market to
//! the byte. A carry bounded to a number of boundaries a call is one carry; a road driven by the core's whole calls alone (`stamp_check`, `trade_on`,
//! `carry`, `decide_at`, `settle_at`, `pool_at`) is the fold's road, branch, purse, refusal, decision and settle, entry by entry and again as a chain
//! sees it (a clock that never runs back, bounded carries, anyone carrying in between); a deposit the law cannot carry is refused by the core as the
//! roster refused it. The first and the third run with the core alone (`--no-default-features`); the second needs the fold to compare against.
use poleis_futarchy::law::*;

const MIN: u64 = 60_000;
/// a road of three days at a step a minute
fn terms(both_ways: bool) -> Terms { Terms { opens_ms: 1_000_000, closes_ms: 1_000_000 + 3 * 86_400_000, step_ms: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 1000 * UNIT, pool_quote: 1000 * UNIT, budget: 1000 * UNIT, both_ways } }

/// xorshift: a seeded sweep with no dependency (tests/law.rs's)
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn below(&mut self, n: u128) -> u128 { (((self.next() as u128) << 64) | self.next() as u128) % n.max(1) } }

#[test]
fn a_carry_in_bounded_steps_is_one_carry() {
    let t = terms(false);
    let span = t.closes_ms - t.opens_ms;
    let mut r = Rng(0x6A09_E667_F3BC_C908);
    let (mut on, mut past, mut carried, mut calls_all) = (0u32, 0u32, 0u32, 0u64);
    for _ in 0..2_000 {
        let mut b = Branch::open(&t);
        b.base = (1 + r.below(5_000)) * UNIT; b.quote = (1 + r.below(5_000)) * UNIT;
        // the instant: sometimes exactly on a boundary (the close among them), sometimes at or past the close, else anywhere in the window
        let now = match r.below(4) {
            0 => { on += 1; t.opens_ms + r.below((span / t.step_ms) as u128 + 1) as u64 * t.step_ms }
            1 => { past += 1; t.closes_ms + r.below(86_400_000) as u64 }
            _ => t.opens_ms + r.below(span as u128) as u64,
        };
        // sometimes a branch already carried part of the way, to a boundary or inside a step, its held sum (acc) begun
        if r.below(3) == 0 { let a = t.opens_ms + r.below((now.min(t.closes_ms) - t.opens_ms) as u128 + 1) as u64; b.advance(a, &t); carried += 1; }
        let mut one = b;
        one.advance(now, &t);
        for most in [1u64, 2, 7, 100, 5000] {
            let mut c = b;
            let mut calls = 0u64;
            loop {
                let before = c.steps;
                let stands = c.walk(now, most, &t);
                calls += 1;
                if stands { break }
                // the boundaries it walked are `most` exactly (tests/quiet.rs counts them); a stretch at rest among them was free
                assert!(c.steps - before >= most, "a bounded carry short of the instant moved {} boundaries, under {most} (now {now}, at {})", c.steps - before, c.at);
            }
            assert_eq!(c, one, "at most {most} boundaries a call, in {calls} calls, is one carry to {now}");
            calls_all += calls;
        }
    }
    // a whole window, from the opening to the close, at 100 boundaries a call
    let (mut b, mut calls) = (Branch::open(&t), 0u64);
    loop { calls += 1; if b.walk(t.closes_ms, 100, &t) { break } }
    let mut one = Branch::open(&t);
    one.advance(t.closes_ms, &t);
    assert_eq!(b, one);
    eprintln!("  2000 branches ({on} on a boundary, {past} at or past the close, {carried} already carried part of the way), five bounds each: {calls_all} bounded carries, each the one carry; a whole window of {} boundaries took {calls} calls at 100 a call", one.steps);
}

#[cfg(feature = "fold")]
#[test]
fn the_core_alone_carries_a_road_as_the_fold_does() {
    fn entry(seq: u64, ts: u64, key: &str, branch: Which, side: Side, amount: u128) -> Entry {
        Entry { seq, ts, key: key.into(), seat: key.into(), ok: true, trade: Some(Trade { branch, side, amount, min_out: 0 }) }
    }
    fn tr(e: &Entry) -> Trade { e.trade.unwrap() }
    let of_an_entry = [NOT_A_TRADE, UNSIGNED, OUTSIDE, BACKWARDS, NOT_ON_THE_ROSTER];
    let of_the_fold = [NOT_A_TRADE, UNSIGNED, NOT_ON_THE_ROSTER];
    let of_a_trade = [NOT_ENOUGH, NOTHING_OUT, PAST_MIN, OUT_OF_RANGE];
    let mut seen = [0u64; 9];
    let (mut roads, mut both, mut rostered, mut read, mut taken, mut settled, mut carries) = (0u32, 0u32, 0u32, 0u64, 0u64, 0u32, 0u64);
    let keys = ["a", "b", "c", "d", "e"];
    let mut r = Rng(0xBB67_AE85_84CA_A73B);
    for road in 0..240 {
        let both_ways = r.below(2) == 0;
        let t = Terms { opens_ms: 5_000, closes_ms: 5_000 + (2 + r.below(600) as u64) * MIN, step_ms: [7_000, 20_000, MIN, 5 * MIN][r.below(4) as usize], max_step_bp: [1, 25, 100, 10_000][r.below(4) as usize],
            threshold_bp: [0, 300, 10_000][r.below(3) as usize], fee_bp: 0, pool_base: (1000 + r.below(10_000_000)) * UNIT, pool_quote: (1000 + r.below(10_000_000)) * UNIT, budget: (1 + r.below(999_999)) * UNIT, both_ways };
        if t.check().is_err() { continue; }
        // a roster of random deposits, or none (every signer then deposits the budget)
        let mut ros: Vec<Deposit> = if r.below(2) == 0 { Vec::new() } else { keys.iter().map(|k| Deposit { key: k.to_string(), amount: UNIT + r.below(1_000_000 * UNIT - UNIT + 1) }).collect() };
        ros.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(roster_check(&t, &ros), Ok(()), "road {road}");
        let dep = |k: &str| if ros.is_empty() { t.budget } else { ros.iter().find(|d| d.key == k).map(|d| d.amount).unwrap_or(0) };
        let span = t.closes_ms - t.opens_ms;
        let mut ts = t.opens_ms;
        let es: Vec<Entry> = (0..60).map(|i| {
            ts += r.below(span as u128 / 60) as u64;
            // "z" is off the roster when the road names one
            let key = if r.below(12) == 0 { "z" } else { keys[r.below(5) as usize] };
            let side = if r.below(3) == 0 { Side::Sell } else { Side::Buy };
            let p = t.purse_of(match dep(key) { 0 => t.budget, d => d });
            let most = if side == Side::Sell { p.base } else { p.quote };
            let amount = match r.below(10) { 0 => 1 + r.below(50), 1 => most + 1 + r.below(most + 1), _ => 1 + r.below(most / 3 + 1) };
            let mut e = entry(i + 1, ts, key, if r.below(2) == 0 { Which::Pass } else { Which::Fail }, side, amount);
            match r.below(40) {
                0 => e.ok = false,
                1 => e.trade = None,
                2 => e.ts = t.closes_ms + r.below(MIN as u128) as u64,
                3 | 4 => e.ts = ts.saturating_sub(1 + r.below(span as u128 / 20) as u64).max(t.opens_ms),
                5 => e.trade = Some(Trade { min_out: u128::MAX, ..tr(&e) }),
                6 => e.trade = Some(Trade { amount: 0, ..tr(&e) }),
                _ => {}
            }
            e
        }).collect();
        let now = if r.below(5) == 0 { t.opens_ms + r.below(span as u128) as u64 } else { t.closes_ms };
        let m = market_with(&t, &ros, &es, now);

        // THE PROGRAM: two pools, a pair of purses a key, the stamp of the last trade taken, one entry at a time, by the core's whole calls. Only
        // what a chain checks by other means is taken from the fold: that an entry is a trade, that it is signed, who is on the roster. The rules
        // of time and of a trade are asked of the core, as a program asks them.
        let skip: Vec<u64> = m.refused.iter().filter(|x| of_the_fold.contains(&x.why)).map(|x| x.seq).collect();
        let (mut pass, mut fail, mut last) = (Branch::open(&t), Branch::open(&t), t.opens_ms);
        let mut purses: Vec<(String, u128, Purse, Purse)> = Vec::new();
        let (mut refused, mut took): (Vec<(u64, &'static str)>, u64) = (Vec::new(), 0);
        let mut order: Vec<&Entry> = es.iter().collect();
        order.sort_by_key(|e| e.seq);
        for e in order {
            if skip.contains(&e.seq) { continue; }
            let tr = e.trade.unwrap();
            if let Err(why) = stamp_check(&t, e.ts, last, now) { refused.push((e.seq, why)); continue; }
            let i = match purses.iter().position(|p| p.0 == e.key) { Some(i) => i, None => { let d = dep(e.key.as_str()); let o = t.purse_of(d); purses.push((e.key.clone(), d, o, o)); purses.len() - 1 } };
            let mine = match tr.branch { Which::Pass => &mut purses[i].2, Which::Fail => &mut purses[i].3 };
            match trade_on(&t, &mut pass, &mut fail, mine, &tr, e.ts) {
                Err(why) => refused.push((e.seq, why)),
                Ok(_) => { last = e.ts; took += 1; }
            }
        }
        let decided = decide_at(&t, &mut pass, &mut fail, now);

        // the same market, to the field
        assert_eq!((pass, fail, took), (m.pass, m.fail, m.accepted), "road {road}: the two branches and the trades taken");
        taken += took;
        for (key, d, p, f) in &purses {
            match m.traders.iter().find(|h| &h.key == key) {
                Some(h) => assert_eq!((h.deposit, h.pass, h.fail), (*d, *p, *f), "road {road}: {key}'s purses"),
                None => assert_eq!((*p, *f), (t.purse_of(*d), t.purse_of(*d)), "road {road}: {key}, whose every trade was refused, holds what it opened with"),
            }
        }
        assert_eq!(m.traders.len(), purses.iter().filter(|p| m.traders.iter().any(|h| h.key == p.0)).count(), "road {road}: every trader of the fold is a key of the program");
        let fold_refused: Vec<(u64, &'static str)> = m.refused.iter().filter(|x| !of_the_fold.contains(&x.why)).map(|x| (x.seq, x.why)).collect();
        assert_eq!(refused, fold_refused, "road {road}: the same entries refused by the rules of time and by the trade, for the same reason");
        for x in &m.refused { seen[of_an_entry.iter().chain(of_a_trade.iter()).position(|w| *w == x.why).unwrap()] += 1; }
        let d = decide(&t, &m, now);
        assert_eq!(decided, d, "road {road}: the decision");

        // THE CHAIN: the same road as a program on a chain sees it. Its clock never runs back; a refused trade leaves nothing (the instruction is
        // undone, its carry too), so only the trades the fold took are sent; the road is carried in bounded calls before each trade, at 1 to 40
        // boundaries a call, and someone carries it once at an instant between two trades.
        let (mut cp, mut cf, mut clast, mut clock) = (Branch::open(&t), Branch::open(&t), t.opens_ms, t.opens_ms);
        let mut cpurses: Vec<(String, u128, Purse, Purse)> = Vec::new();
        let mut sent: Vec<&Entry> = es.iter().filter(|e| !m.refused.iter().any(|x| x.seq == e.seq)).collect();
        sent.sort_by_key(|e| e.seq);
        for e in sent {
            let between = clock + r.below((e.ts - clock) as u128 + 1) as u64;
            carry(&t, &mut cp, &mut cf, between, 1 + r.below(40) as u64);
            clock = e.ts;
            let most = 1 + r.below(40) as u64;
            while !carry(&t, &mut cp, &mut cf, clock, most) { carries += 1; }
            assert_eq!(stamp_check(&t, clock, clast, clock), Ok(()), "road {road}: a trade the fold took is in time on the chain's clock");
            let tr = e.trade.unwrap();
            let i = match cpurses.iter().position(|p| p.0 == e.key) { Some(i) => i, None => { let d = dep(e.key.as_str()); let o = t.purse_of(d); cpurses.push((e.key.clone(), d, o, o)); cpurses.len() - 1 } };
            let mine = match tr.branch { Which::Pass => &mut cpurses[i].2, Which::Fail => &mut cpurses[i].3 };
            assert!(trade_on(&t, &mut cp, &mut cf, mine, &tr, clock).is_ok(), "road {road}: the chain takes the trade the fold took");
            clast = clock;
        }
        while !carry(&t, &mut cp, &mut cf, now, 1 + r.below(40) as u64) { carries += 1; }
        assert_eq!((decide_at(&t, &mut cp, &mut cf, now), cp, cf), (d, m.pass, m.fail), "road {road}: carried in bounded calls on a clock that never runs back, the chain's road is the fold's");
        for h in &m.traders {
            let (_, dp, p, f) = cpurses.iter().find(|p| p.0 == h.key).unwrap();
            assert_eq!((*dp, *p, *f), (h.deposit, h.pass, h.fail), "road {road}: {}'s purses on the chain", h.key);
        }

        if let Some(world) = world_of(&d) {
            let rc = Receipts { world, worth: if r.below(6) == 0 { [0, PRICE][r.below(2) as usize] } else { r.below(PRICE + 1) } };
            let rows = settle_of(&t, &m, &d, &rc).unwrap();
            assert_eq!(rows.len(), m.traders.len());
            for s in &rows {
                let (_, dp, p, f) = cpurses.iter().find(|p| p.0 == s.key).unwrap();
                assert_eq!(settle_at(&t, &d, &rc, *dp, p, f), Ok(s.result), "road {road}: {}'s settle", s.key);
                assert_eq!(back_of(*dp, s.result) as i128, *dp as i128 + s.result, "road {road}: what comes back to {}", s.key);
            }
            assert_eq!(pool_at(&t, &d, &rc, &cp, &cf), pool_result(&t, &m, &d, &rc), "road {road}: the pool's settle");
            // the receipts the law refuses settle nothing: the other world, a worth over one
            let other = Receipts { world: if world == Which::Pass { Which::Fail } else { Which::Pass }, worth: rc.worth };
            let p0 = t.purse_of(t.budget);
            assert_eq!((settle_at(&t, &d, &other, t.budget, &p0, &p0), pool_at(&t, &d, &other, &cp, &cf)), (Err(OTHER_WORLD), Err(OTHER_WORLD)));
            assert_eq!(settle_at(&t, &d, &Receipts { world, worth: PRICE + 1 }, t.budget, &p0, &p0), Err(WORTH_RANGE));
            settled += 1;
        } else {
            let p0 = t.purse_of(t.budget);
            assert_eq!(settle_at(&t, &d, &Receipts { world: Which::Pass, worth: 0 }, t.budget, &p0, &p0), Err(STILL_OPEN), "road {road}: an open road settles nothing");
        }
        roads += 1; both += both_ways as u32; rostered += !ros.is_empty() as u32; read += es.len() as u64;
    }
    let names = of_an_entry.iter().chain(of_a_trade.iter()).zip(seen.iter()).map(|(w, n)| format!("{n} {w}")).collect::<Vec<_>>().join("; ");
    eprintln!("  {roads} roads ({both} both ways, {rostered} with a roster, {settled} settled on receipts), {read} entries read, {taken} trades taken by the core alone, as the fold took them, and again on a chain's clock with {carries} bounded carries that did not yet reach it; refused: {names}");
    assert!(roads >= 150 && both > 30 && roads - both > 30 && rostered > 30 && roads - rostered > 30, "the sweep met roads both ways and one way, with a roster and with none");
    assert!(seen.iter().all(|n| *n > 0), "the sweep met every refusal: {names}");
}

#[test]
fn a_deposit_a_road_cannot_carry_is_refused() {
    let (both, one) = (terms(true), terms(false));
    let (empty, small) = (Err("a deposit is empty or over a billion"), Err("a deposit both ways is too small to hold both $WARM and claims"));
    for t in [both, one] {
        assert_eq!(deposit_check(&t, 0), empty, "a deposit of nothing, both ways {}", t.both_ways);
        assert_eq!(deposit_check(&t, AMOUNT_MAX), Ok(()), "a billion, both ways {}", t.both_ways);
        assert_eq!(deposit_check(&t, AMOUNT_MAX + 1), empty, "over a billion, both ways {}", t.both_ways);
    }
    assert_eq!(deposit_check(&one, 1), Ok(()), "one micro is carried one way");
    assert_eq!(deposit_check(&both, 1), small, "and refused both ways: it cannot hold both $WARM and claims");
    assert_eq!(deposit_check(&both, 2), Ok(()), "two micro both ways hold one of each in a pool of even sides");
}

#[cfg(feature = "fold")]
#[test]
fn a_road_carried_to_a_boundary_takes_it_and_the_trade_stamped_on_it_changes_nothing_of_it() {
    let t = terms(false);
    let on = t.opens_ms + 5 * MIN;
    let buy = Trade { branch: Which::Pass, side: Side::Buy, amount: 200 * UNIT, min_out: 0 };
    // the fold: a trade stamped on a boundary counts from its own stamp; the boundary took the price that stood before it
    let m = market_of(&t, &[Entry { seq: 1, ts: on, key: "a".into(), seat: "a".into(), ok: true, trade: Some(buy) }], on);
    let open = spot(t.pool_base, t.pool_quote).unwrap();
    assert_eq!((m.pass.obs, m.pass.steps, m.pass.acc), (open, 5, 0));
    let mut quiet = (Branch::open(&t), Branch::open(&t));
    quiet.0.advance(on, &t); quiet.1.advance(on, &t);
    // the chain: someone carries the road to that very instant, in one call or in many, and then the trade comes, stamped there
    for most in [0u64, 1, 2, 3, 5, 1000] {
        let (mut pass, mut fail, mut mine) = (Branch::open(&t), Branch::open(&t), t.purse_of(t.budget));
        let mut calls = 0;
        while !carry(&t, &mut pass, &mut fail, on, most) { calls += 1; assert!(calls < 100, "a carry of at most {most} boundaries a call reaches the instant"); }
        assert_eq!((pass.steps, pass.at, fail.steps, fail.at), (5, on, 5, on), "carried to the boundary, the boundary itself applied: the step that ended there was whole");
        assert_eq!((pass, fail), quiet, "at most {most} boundaries a call: the road at the boundary is the road no trade touched");
        assert_eq!(stamp_check(&t, on, t.opens_ms, on), Ok(()));
        trade_on(&t, &mut pass, &mut fail, &mut mine, &buy, on).unwrap();
        assert_eq!((pass.obs, pass.cum, pass.acc, pass.steps, pass.at), (quiet.0.obs, quiet.0.cum, quiet.0.acc, quiet.0.steps, quiet.0.at), "the trade stamped on it changed nothing of the boundary");
        decide_at(&t, &mut pass, &mut fail, on);
        assert_eq!((pass, fail), (m.pass, m.fail), "at most {most} boundaries a call, the fold's road");
        // and the next boundary is the trade's: its price stood the whole step, held one band away
        decide_at(&t, &mut pass, &mut fail, on + MIN);
        assert_eq!(pass.obs, open + open / 100);
    }
    // a bound of nothing is read as one: a carry always moves. A branch whose observation chases a spot a trade moved walks one boundary at a
    // bound of 0 as at 1; a branch nobody touched is at rest and stands at the close after either
    let mut b = Branch::open(&t);
    b.quote = 2 * b.quote;
    let (mut zero, mut one) = (b, b);
    assert_eq!((zero.walk(t.closes_ms, 0, &t), one.walk(t.closes_ms, 1, &t)), (false, false));
    assert_eq!((zero, zero.steps, zero.at), (one, 1, t.opens_ms + MIN));
    let (mut zero, mut one) = (Branch::open(&t), Branch::open(&t));
    assert_eq!((zero.walk(t.closes_ms, 0, &t), one.walk(t.closes_ms, 1, &t)), (true, true));
    assert_eq!((zero, zero.at), (one, t.closes_ms));
}
