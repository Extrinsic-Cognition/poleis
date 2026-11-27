//! THE LAW ON ITS OWN CODE: what proofs/*.smt2 state about the arithmetic is checked here on the functions themselves, over every sequence of a small
//! pool and a seeded sweep of large ones; the grid is shown against the rule it replaces, and the price that stood against the tick's rule it
//! replaces in turn.
#![cfg(feature = "fold")]
use poleis_futarchy::law::*;

const MIN: u64 = 60_000;
fn terms() -> Terms { Terms { opens_ms: 1_000_000, closes_ms: 1_000_000 + 3 * 86_400_000, step_ms: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 1000 * UNIT, pool_quote: 1000 * UNIT, budget: 1000 * UNIT, both_ways: false } }
fn entry(seq: u64, ts: u64, key: &str, branch: Which, side: Side, amount: u128) -> Entry {
    Entry { seq, ts, key: key.into(), seat: key.into(), ok: true, trade: Some(Trade { branch, side, amount, min_out: 0 }) }
}

/// both tokens of a branch, pool and purses together: they never change, to the micro
fn conserved(t: &Terms, m: &Market) {
    let n = m.traders.len() as u128;
    let opens = t.purse_of(t.budget);
    for (b, pick) in [(&m.pass, 0), (&m.fail, 1)] {
        let (q, s): (u128, u128) = m.traders.iter().map(|h| if pick == 0 { h.pass } else { h.fail }).fold((0, 0), |a, p| (a.0 + p.quote, a.1 + p.base));
        assert_eq!(b.quote + q, t.pool_quote + n * opens.quote, "the $WARM of a branch is conserved");
        assert_eq!(b.base + s, t.pool_base + n * opens.base, "the base of a branch is conserved");
        assert!(b.base * b.quote >= t.pool_base * t.pool_quote, "a pool's product never falls");
        assert!((PRICE_LO..=PRICE_HI).contains(&b.spot()), "the spot stays inside the law's range");
    }
}

#[test]
fn every_sequence_of_a_small_pool_keeps_the_law() {
    // every sequence is read twice: on a road one way (purses that can only buy first) and on a road both ways (purses whose deposit of 30 opens
    // in the pool's own mix, 15 of the claim beside 15 of $WARM, and can sell first)
    for both_ways in [false, true] {
        let clock = std::time::Instant::now();
        // a pool of 50 micro a side, two traders of 30 each, every sequence of three trades out of 48: 110,592 markets, each read after every entry
        let t = Terms { opens_ms: 0, closes_ms: 10 * MIN, step_ms: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 50, pool_quote: 50, budget: 30, both_ways };
        t.check().unwrap();
        let purse = t.purse_of(t.budget);
        assert_eq!(purse, if both_ways { Purse { quote: 15, base: 15 } } else { Purse { quote: 30, base: 0 } });
        let claim = purse.base;
        let mut moves = Vec::new();
        for key in ["a", "b"] { for branch in [Which::Pass, Which::Fail] { for side in [Side::Buy, Side::Sell] { for amount in [1u128, 2, 3, 5, 8, 13] { moves.push((key, branch, side, amount)); } } } }
        let (mut markets, mut refusals) = (0u64, 0u64);
        for a in &moves { for b in &moves { for c in &moves {
            let es: Vec<Entry> = [a, b, c].iter().enumerate().map(|(i, m)| entry(i as u64 + 1, (i as u64 + 1) * 90_000, m.0, m.1, m.2, m.3)).collect();
            let m = market_of(&t, &es, t.closes_ms);
            conserved(&t, &m);
            assert_eq!(m.accepted as usize + m.refused.len(), 3);
            // a refused entry changes nothing: the market without them is the same market
            let kept: Vec<Entry> = es.iter().filter(|e| !m.refused.iter().any(|r| r.seq == e.seq)).cloned().collect();
            let again = market_of(&t, &kept, t.closes_ms);
            assert_eq!((m.pass, m.fail, &m.traders), (again.pass, again.fail, &again.traders), "a refused entry changes nothing");
            markets += 1; refusals += m.refused.len() as u64;
        } } }
        eprintln!("  cost: {markets} markets of three entries folded and checked in {:.0} ms (a purse that opens with {claim} of the claim)", clock.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(markets, 110_592);
        assert!(refusals > 0, "the sweep met refusals (a sell with no base, a dust trade)");
    }
}

/// xorshift: a seeded sweep with no dependency
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn below(&mut self, n: u128) -> u128 { (((self.next() as u128) << 64) | self.next() as u128) % n.max(1) } }

#[test]
fn a_seeded_sweep_of_large_markets_keeps_the_law_and_a_reading_changes_nothing() {
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    for road in 0..160 {
        let t = Terms { opens_ms: 5_000, closes_ms: 5_000 + (2 + r.below(600) as u64) * MIN, step_ms: [7_000, 20_000, MIN, 5 * MIN][r.below(4) as usize], max_step_bp: [1, 25, 100, 10_000][r.below(4) as usize],
            threshold_bp: [0, 300, 10_000][r.below(3) as usize], fee_bp: 0, pool_base: (1 + r.below(999_999_999)) * UNIT, pool_quote: (1 + r.below(999_999_999)) * UNIT, budget: (1 + r.below(999_999_999)) * UNIT, both_ways: false };
        if t.check().is_err() { continue; }
        let span = t.closes_ms - t.opens_ms;
        let mut ts = t.opens_ms;
        let es: Vec<Entry> = (0..60).map(|i| {
            ts += r.below(span as u128 / 60) as u64;
            let key = ["a", "b", "c", "d", "e"][r.below(5) as usize];
            let amount = if r.below(4) == 0 { 1 + r.below(50) } else { 1 + r.below(t.budget / 3 + 1) };
            entry(i + 1, ts, key, if r.below(2) == 0 { Which::Pass } else { Which::Fail }, if r.below(3) == 0 { Side::Sell } else { Side::Buy }, amount)
        }).collect();
        let m = market_of(&t, &es, t.closes_ms);
        conserved(&t, &m);
        let d = decide(&t, &m, t.closes_ms);
        assert_ne!(d.state, State::Open, "road {road}: at the close the road is decided");
        // the average lies between the lowest and the highest spot the branch ever had (the observation never leaves them)
        let open = spot(t.pool_base, t.pool_quote).unwrap();
        let (mut lo, mut hi) = ([open; 2], [open; 2]);
        for n in 1..=es.len() { let p = market_of(&t, &es[..n], es[n - 1].ts); for (i, s) in [p.pass.spot(), p.fail.spot()].into_iter().enumerate() { lo[i] = lo[i].min(s); hi[i] = hi[i].max(s); } }
        for (i, b) in [&m.pass, &m.fail].into_iter().enumerate() {
            let avg = b.twap(&t).unwrap();
            assert!(lo[i] <= avg + 1 && avg <= hi[i], "road {road}: the average {avg} lies between the spots {} and {}", lo[i], hi[i]);
        }
        // a reading at any instant is the fold of what was in by then, and it changes no later answer: the same entries read again give the same market
        let now = t.opens_ms + r.below(span as u128) as u64;
        let seen: Vec<Entry> = es.iter().filter(|e| e.ts <= now).cloned().collect();
        let (all, part) = (market_of(&t, &es, now), market_of(&t, &seen, now));
        assert_eq!((all.pass, all.fail, &all.traders, all.accepted), (part.pass, part.fail, &part.traders, part.accepted), "road {road}: a reading sees only what was in (the later entries are only named as refused)");
        assert_eq!(market_of(&t, &es, t.closes_ms), m, "road {road}: the fold is a function of its entries");
    }
}

#[test]
fn carrying_the_average_in_two_reads_is_carrying_it_in_one() {
    let t = terms();
    let mut r = Rng(42);
    for _ in 0..2_000 {
        let mut one = Branch::open(&t);
        one.base = (1 + r.below(5_000)) * UNIT; one.quote = (1 + r.below(5_000)) * UNIT;
        let mut two = one;
        let a = t.opens_ms + r.below(86_400_000) as u64;
        let b = a + r.below(86_400_000) as u64;
        // sometimes exactly on a boundary, and sometimes inside a step, where the step's held sum is split between the two reads
        let a = if r.below(3) == 0 { t.opens_ms + ((a - t.opens_ms) / t.step_ms) * t.step_ms } else { a };
        one.advance(b, &t);
        two.advance(a, &t); two.advance(b, &t);
        assert_eq!(one, two);
    }
}

#[test]
fn a_trade_on_a_boundary_moves_the_next_one_a_full_step_and_one_a_millisecond_later_by_its_share() {
    let t = terms();
    let at = t.opens_ms + 5 * MIN;
    let buy = |ts: u64| [entry(1, ts, "a", Which::Pass, Side::Buy, 200 * UNIT)];
    let open = spot(t.pool_base, t.pool_quote).unwrap();
    let d = open / 100;
    let on = market_of(&t, &buy(at), at);
    assert_eq!((on.pass.obs, on.pass.steps, on.pass.acc), (open, 5, 0), "the boundary the trade stands on was whole before it came: it took the price that stood");
    assert_eq!(market_of(&t, &buy(at), at + MIN).pass.obs, open + d, "the trade's price stood the whole next step, held one band away: the next boundary moves one full step");
    assert_eq!(market_of(&t, &buy(at + 1), at + MIN - 1).pass.obs, open, "a trade a millisecond later moves nothing until the next boundary");
    assert_eq!(market_of(&t, &buy(at + 1), at + MIN).pass.obs, open + d * (MIN as u128 - 1) / MIN as u128, "and there it moves it by its share of the step: the opening price stood its first millisecond");
}

/// THE TICK'S RULE, the grid as it first stood, in the same integers: at each boundary, once every entry stamped at or before it is in, the
/// observation steps toward the spot as it stands there. `es` in their order, each a trade the law takes. The observation and its sum at the
/// close, and the first boundary at which the observation stood past three times the opening price.
fn the_ticks_rule(es: &[Entry], t: &Terms) -> (u128, u128, Option<u64>) {
    let mut pool = Branch::open(t);
    let (open, mut obs, mut cum, mut i, mut k, mut past) = (pool.obs, pool.obs, 0u128, 0usize, 1u64, None);
    loop {
        let next = t.opens_ms + k * t.step_ms;
        if next >= t.closes_ms { cum += obs * (t.closes_ms - (next - t.step_ms)) as u128; return (obs, cum, past); }
        cum += obs * t.step_ms as u128;
        while i < es.len() && es[i].ts <= next {
            let tr = es[i].trade.unwrap();
            let out = swap_out(pool.base, pool.quote, tr.side, tr.amount).unwrap();
            match tr.side { Side::Buy => { pool.base -= out; pool.quote += tr.amount; } Side::Sell => { pool.base += tr.amount; pool.quote -= out; } }
            i += 1;
        }
        obs = step_toward(obs, pool.spot(), t.max_step_bp);
        if past.is_none() && obs > 3 * open { past = Some(k); }
        k += 1;
    }
}

#[test]
fn the_trader_who_stood_a_second_at_every_boundary_walks_nothing() {
    // a purse of 1000 buys PASS with 900 at every boundary of three days and sells what it bought a second later: in the pool a second in sixty
    let t = terms();
    let open = spot(t.pool_base, t.pool_quote).unwrap();
    let (mut pool, mut es) = (Branch::open(&t), Vec::new());
    for k in 1u64.. {
        let at = t.opens_ms + k * MIN;
        if at + 1_000 >= t.closes_ms { break; }
        let got = swap_out(pool.base, pool.quote, Side::Buy, 900 * UNIT).unwrap();
        pool.base -= got; pool.quote += 900 * UNIT;
        let back = swap_out(pool.base, pool.quote, Side::Sell, got).unwrap();
        pool.base += got; pool.quote -= back;
        es.push(entry(es.len() as u64 + 1, at, "trader", Which::Pass, Side::Buy, 900 * UNIT));
        es.push(entry(es.len() as u64 + 1, at + 1_000, "trader", Which::Pass, Side::Sell, got));
    }
    let pushed = market_of(&t, &es[..1], t.opens_ms + MIN).pass.spot();
    // THIS LAW, by the core's calls one entry at a time, the observation read after every one
    let (mut pass, mut fail, mut mine, mut highest) = (Branch::open(&t), Branch::open(&t), t.purse_of(t.budget), open);
    for e in &es { trade_on(&t, &mut pass, &mut fail, &mut mine, &e.trade.unwrap(), e.ts).unwrap(); highest = highest.max(pass.obs); }
    let d = decide_at(&t, &mut pass, &mut fail, t.closes_ms);
    let m = market_of(&t, &es, t.closes_ms);
    assert_eq!((m.accepted as usize, m.pass, m.fail, d), (es.len(), pass, fail, decide(&t, &m, t.closes_ms)), "every entry taken, the fold the core's calls");
    let lost = t.budget - mine.quote;
    eprintln!("  {} entries, PASS's spot pushed to {pushed} a second in every step from {open}: this law's observation at most {highest} ({} ppm over the opening), its average {:?} against the {:?} it needs: {:?}; the trader holds {} claims and lost {lost} micro $WARM",
        es.len(), (highest - open) * 1_000_000 / open, d.twap_pass, d.needs, d.state, mine.base);
    assert!(highest * 10_000 <= open * 10_002, "the observation never rose more than 0.02 % over the opening price: at most {highest}");
    assert_eq!(d.state, State::Failed, "and the road fails");
    assert_eq!((mine.base, m.traders[0].pass), (0, mine), "the trader holds no claim at the close");
    assert!(lost <= UNIT / 100, "and lost at most a hundredth of a $WARM: {lost} micro");
    let rc = Receipts { world: Which::Fail, worth: PRICE / 2 };
    assert_eq!(settle_of(&t, &m, &d, &rc).unwrap()[0].result, 0, "its world did not happen: it settles at nothing");
    // THE TICK'S RULE, on the same entries: one full step at every boundary
    let (obs, cum, past) = the_ticks_rule(&es, &t);
    let span = (t.closes_ms - t.opens_ms) as u128;
    eprintln!("  the tick's rule on the same entries: the observation past three times the opening price at boundary {past:?}, {obs} at the close, its average {} against the {} it needs: passed",
        cum / span, open * (BP + t.threshold_bp as u128) / BP);
    assert!(obs > 3 * open && past.is_some(), "the tick's rule carried the observation past three times the opening price: {obs}");
    assert!(cum * BP > open * span * (BP + t.threshold_bp as u128), "and passed the road (FAIL's average stood at the opening price), for a trader whose result is {} micro", -(lost as i128));
    // the trader that HOLDS: one buy at the opening, never sold. Here the law walks a full step at every boundary, the tick's rule to the byte,
    // and that trader is the one the receipts charge (tests/settle.rs the_old_mark_pays_whoever_lifts_a_price_and_the_settle_charges_it)
    let hold = [entry(1, t.opens_ms, "holder", Which::Pass, Side::Buy, 900 * UNIT)];
    let m = market_of(&t, &hold, t.closes_ms);
    let (obs, cum, _) = the_ticks_rule(&hold, &t);
    assert_eq!((m.pass.obs, m.pass.cum), (obs, cum), "a price held the whole window walks the observation exactly as the tick's rule did");
    assert_eq!(m.pass.obs, m.pass.spot(), "all the way to the spot it holds");
    assert_eq!(decide(&t, &m, t.closes_ms).state, State::Passed);
}

/// the rule of futarchy-law.mjs's advance, in the same integers: the observation steps once per CALL, whatever the time since the last
fn the_old_rule(obs: &mut u128, cum: &mut u128, at: &mut u64, to: u64, spot: u128, t: &Terms) {
    while *at < to { let dt = t.step_ms.min(to - *at); *obs = step_toward(*obs, spot, t.max_step_bp); *cum += *obs * dt as u128; *at += dt; }
}

#[test]
fn the_spam_that_walked_the_old_average() {
    // a whale spikes PASS an hour before the close, then sends one dust trade a millisecond for 400 ms
    let t = terms();
    let spike_at = t.closes_ms - 60 * MIN;
    let mut es = vec![entry(1, spike_at, "whale", Which::Pass, Side::Buy, 900 * UNIT)];
    for i in 0..400u64 { es.push(entry(2 + i, spike_at + 1 + i, "whale", Which::Pass, Side::Buy, 10)); }
    let m = market_of(&t, &es, t.closes_ms);
    assert_eq!(m.accepted, 401, "every entry of the spam is a trade the law takes");
    let open = spot(t.pool_base, t.pool_quote).unwrap();
    // THIS LAW: sixty boundaries stand between the spike and the close, so the observation rose by at most 1.01^60, whatever the number of entries
    assert_eq!(m.pass.steps, 3 * 1440 - 1);
    assert!(m.pass.obs * 1_000 <= open * 1_817, "the observation rose {} from {open}: at most 1.01^60", m.pass.obs);
    let quiet = market_of(&t, &es[..1], t.closes_ms);
    assert_eq!((m.pass.obs, m.pass.cum), (quiet.pass.obs, quiet.pass.cum), "the 400 entries moved the average by nothing");
    // THE OLD RULE, on the same entries: one step an entry
    let (mut obs, mut cum, mut at) = (open, 0u128, t.opens_ms);
    let mut pool = Branch::open(&t);
    for e in &es {
        the_old_rule(&mut obs, &mut cum, &mut at, e.ts, pool.spot(), &t);
        let tr = e.trade.unwrap();
        let out = swap_out(pool.base, pool.quote, tr.side, tr.amount).unwrap();
        pool.base -= out; pool.quote += tr.amount;
    }
    the_old_rule(&mut obs, &mut cum, &mut at, t.closes_ms, pool.spot(), &t);
    assert!(obs > open * 3, "under the old rule the same spam carried the observation from {open} to {obs}, past three times the opening price");
    let d = decide(&t, &m, t.closes_ms);
    assert_eq!(d.state, State::Failed, "the spike and its spam do not pass the road: PASS's average is {:?}, it needs {:?}", d.twap_pass, d.needs);
    assert!(cum / (t.closes_ms - t.opens_ms) as u128 > d.needs.unwrap(), "under the old rule they passed it");
}

#[test]
fn the_threshold_is_strict_and_the_scores_are_the_branch_that_happened() {
    let t = Terms { threshold_bp: 0, ..terms() };
    let quiet = market_of(&t, &[], t.closes_ms);
    assert_eq!(decide(&t, &quiet, t.closes_ms).state, State::Failed, "equal averages do not pass, even at a threshold of nothing");
    assert_eq!(decide(&t, &quiet, t.closes_ms - 1).state, State::Open);
    let t = terms();
    let es = [entry(1, t.opens_ms + MIN, "bull", Which::Pass, Side::Buy, 300 * UNIT), entry(2, t.opens_ms + 2 * MIN, "bear", Which::Pass, Side::Buy, 1 * UNIT), entry(3, t.opens_ms + 3 * MIN, "bear", Which::Fail, Side::Buy, 500 * UNIT)];
    let m = market_of(&t, &es, t.closes_ms);
    let d = decide(&t, &m, t.closes_ms);
    assert_eq!(d.state, State::Failed, "FAIL was bid higher than PASS");
    let s = score_of(&t, &m, &d);
    assert_eq!(s.len(), 2);
    let bull = s.iter().find(|x| x.seat == "bull").unwrap();
    assert_eq!(bull.result, 0, "the bull's PASS trades did not happen: its FAIL purse is its budget, untouched");
    assert!(score_of(&t, &m, &decide(&t, &m, t.closes_ms - 1)).is_empty(), "no score while the road is open");
}

#[test]
fn what_the_law_refuses() {
    let t = terms();
    let at = t.opens_ms + MIN;
    let why = |e: Entry| market_of(&t, &[e], t.closes_ms).refused.first().map(|r| r.why);
    assert_eq!(why(Entry { trade: None, ..entry(1, at, "a", Which::Pass, Side::Buy, 1) }), Some(NOT_A_TRADE));
    assert_eq!(why(Entry { ok: false, ..entry(1, at, "a", Which::Pass, Side::Buy, UNIT) }), Some(UNSIGNED));
    assert_eq!(why(entry(1, t.closes_ms, "a", Which::Pass, Side::Buy, UNIT)), Some(OUTSIDE));
    assert_eq!(why(entry(1, t.opens_ms - 1, "a", Which::Pass, Side::Buy, UNIT)), Some(OUTSIDE));
    assert_eq!(why(entry(1, at, "a", Which::Pass, Side::Buy, t.budget + 1)), Some(NOT_ENOUGH));
    assert_eq!(why(entry(1, at, "a", Which::Pass, Side::Sell, UNIT)), Some(NOT_ENOUGH));
    assert_eq!(why(Entry { trade: Some(Trade { branch: Which::Pass, side: Side::Buy, amount: UNIT, min_out: UNIT }), ..entry(1, at, "a", Which::Pass, Side::Buy, UNIT) }), Some(PAST_MIN));
    assert_eq!(why(entry(1, at, "a", Which::Pass, Side::Buy, 0)), Some(OUT_OF_RANGE));
    let two = market_of(&t, &[entry(1, at + MIN, "a", Which::Pass, Side::Buy, UNIT), entry(2, at, "b", Which::Pass, Side::Buy, UNIT)], t.closes_ms);
    assert_eq!(two.refused.first().map(|r| r.why), Some(BACKWARDS), "an entry stamped before an earlier one is refused, so the fold never runs time back");
    assert_eq!(market_of(&t, &[entry(1, at, "a", Which::Pass, Side::Buy, UNIT)], at - 1).refused.first().map(|r| r.why), Some(OUTSIDE), "a reading does not see a later entry");
    assert!(market_of(&t, &[entry(1, at, "a", Which::Pass, Side::Buy, t.budget + 1)], t.closes_ms).traders.is_empty(), "a trader whose every entry was refused holds nothing");
    // the terms the law cannot carry
    assert!(Terms { closes_ms: t.opens_ms, ..t }.check().is_err());
    assert!(Terms { step_ms: 999, ..t }.check().is_err());
    assert!(Terms { max_step_bp: 0, ..t }.check().is_err());
    assert!(Terms { pool_base: AMOUNT_MAX + 1, ..t }.check().is_err());
    assert!(Terms { pool_base: AMOUNT_MAX, pool_quote: 1, ..t }.check().is_err(), "an opening price under the range");
}
