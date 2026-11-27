//! A QUIET STRETCH IS ONE STEP (law.rs, Nov 20 2026), and THE RULE IT REPLACES, kept as its witness: the loop of `Branch::advance` as it stood
//! before it (at d54699f24a), verbatim below as `advance_as_it_stood`, walked every boundary of the grid one at a time. A seeded sweep runs the two
//! side by side, on roads of every terms the law takes, through trades that move the pools, at every kind of instant, by unbounded and bounded
//! carries, and holds them equal field for field after every carry. Then the bounded form, and the long road with no witness to wait for. The core
//! alone runs these (no feature).
use embertide_futarchy::law::*;

/// THE RULE IT REPLACES: `Branch::advance` as it stood, verbatim (`self` is `b`)
fn advance_as_it_stood(b: &mut Branch, to: u64, t: &Terms) {
    let end = to.clamp(t.opens_ms, t.closes_ms);
    let spot = b.spot();
    loop {
        let next = t.opens_ms.saturating_add((b.steps + 1).saturating_mul(t.step_ms));
        let upto = end.min(next);
        if upto > b.at {
            let dt = (upto - b.at) as u128;
            b.cum = b.cum.saturating_add(b.obs.saturating_mul(dt));
            b.acc = b.acc.saturating_add(step_toward(b.obs, spot, t.max_step_bp).saturating_mul(dt));
            b.at = upto;
        }
        if next >= t.closes_ms || next > end { return; }
        b.obs = b.acc / (t.step_ms as u128).max(1);
        b.acc = 0;
        b.steps += 1;
    }
}

/// a step at rest from its start, as this test reads it on the old loop: the step begins on its boundary with nothing held, and the spot held
/// inside the band is the observation itself (and the observation is inside the law's range, the step inside its longest window)
fn at_rest(b: &Branch, t: &Terms) -> bool {
    b.acc == 0 && b.at == t.opens_ms + b.steps * t.step_ms && step_toward(b.obs, b.spot(), t.max_step_bp) == b.obs && b.obs <= PRICE_HI && t.step_ms <= WINDOW_MAX_MS
}

/// the boundaries the old loop applies from `b` toward `to`, one at a time: how many of them began a step that was not at rest (the walked), how
/// many began one at rest (the skipped), and the longest run of skipped in a row
fn count(b: &Branch, to: u64, t: &Terms) -> (u64, u64, u64) {
    let (mut c, end) = (*b, to.clamp(t.opens_ms, t.closes_ms));
    let (mut walked, mut skipped, mut run, mut longest) = (0u64, 0u64, 0u64, 0u64);
    loop {
        let next = t.opens_ms.saturating_add((c.steps + 1).saturating_mul(t.step_ms));
        if next >= t.closes_ms || next > end { return (walked, skipped, longest); }
        if at_rest(&c, t) { skipped += 1; run += 1; longest = longest.max(run); } else { walked += 1; run = 0; }
        advance_as_it_stood(&mut c, next, t);
    }
}

/// xorshift: a seeded sweep with no dependency (tests/law.rs's)
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn below(&mut self, n: u128) -> u128 { (((self.next() as u128) << 64) | self.next() as u128) % n.max(1) } }

const MIN: u64 = 60_000;
const HOUR: u64 = 3_600_000;

/// terms across the law's range: a step from the floor to four hours, a lag limit from 1 to 10000 bp, a window from two steps to fifty
/// thousand (log-uniform), its close on the grid or off it
fn some_terms(r: &mut Rng) -> Terms {
    let step = match r.below(6) { 0 => STEP_MIN_MS, 1 => STEP_MIN_MS + r.below(9_000) as u64, 2 => MIN, 3 => 7 * MIN + r.below(MIN as u128) as u64, 4 => HOUR, _ => 4 * HOUR };
    let max_step_bp = match r.below(5) { 0 => 1, 1 => 10_000, 2 => 1 + r.below(10) as u32, _ => 1 + r.below(10_000) as u32 };
    let most_steps = (WINDOW_MAX_MS / step).min(50_000);
    let ln = (most_steps as f64 / 2.0).ln();
    let steps = ((2.0 * ((r.below(1_000_000) as f64 / 1_000_000.0) * ln).exp()) as u64).clamp(2, most_steps);
    let opens_ms = r.below(1_000_000_000) as u64;
    let window = (steps * step).saturating_sub(if r.below(3) == 0 { r.below(step as u128) as u64 } else { 0 }).max(step + 1).min(WINDOW_MAX_MS);
    let (pool_base, pool_quote) = ((1 + r.below(1_000_000)) * UNIT, (1 + r.below(1_000_000)) * UNIT);
    Terms { opens_ms, closes_ms: opens_ms + window, step_ms: step, max_step_bp, threshold_bp: r.below(10_001) as u32, pool_base, pool_quote, budget: 1000 * UNIT, both_ways: r.below(2) == 0, fee_bp: [0, 30, 100][r.below(3) as usize] }
}

/// what the sweep met
#[derive(Default, Debug)]
struct Met { roads: u64, carries: u64, bounded_calls: u64, short_calls: u64, skipping: u64, skipped: u64, longest: u64, walked: u64, trades: u64,
    rest_from_opening: u64, partial_acc: u64, chasing_up: u64, chasing_down: u64, zero_band: u64, huge_obs: u64,
    before_opening: u64, mid_step: u64, on_boundary: u64, one_before: u64, one_after: u64, at_close: u64, past_close: u64, backwards: u64 }

/// what a branch is at the start of a carry
fn met_state(m: &mut Met, b: &Branch, t: &Terms) {
    let (spot, held) = (b.spot(), step_toward(b.obs, b.spot(), t.max_step_bp));
    if b.steps == 0 && b.at == t.opens_ms && at_rest(b, t) { m.rest_from_opening += 1; }
    if b.acc > 0 && b.at != t.opens_ms + b.steps * t.step_ms { m.partial_acc += 1; }
    if held > b.obs { m.chasing_up += 1; }
    if held < b.obs { m.chasing_down += 1; }
    if b.obs.saturating_mul(t.max_step_bp as u128) / BP == 0 && spot != b.obs { m.zero_band += 1; }
    if b.obs > PRICE_HI { m.huge_obs += 1; }
}

/// an instant of every kind, near where the road stands or anywhere
fn some_instant(m: &mut Met, r: &mut Rng, t: &Terms, at: u64) -> u64 {
    let steps = (t.closes_ms - t.opens_ms) / t.step_ms;
    let here = (at.max(t.opens_ms) - t.opens_ms) / t.step_ms;
    let near = 1 + r.below(2_000);
    let j = if r.below(3) == 0 { r.below(steps as u128 + 1) as u64 } else { (here + r.below(near) as u64).min(steps) };
    let on = t.opens_ms + j * t.step_ms;
    match r.below(10) {
        0 => { m.before_opening += 1; t.opens_ms.saturating_sub(1 + r.below(HOUR as u128) as u64) }
        1 | 2 => { m.mid_step += 1; on + 1 + r.below(t.step_ms as u128 - 1) as u64 }
        3 => { m.on_boundary += 1; on }
        4 => { m.one_before += 1; on.saturating_sub(1) }
        5 => { m.one_after += 1; on + 1 }
        6 => { m.at_close += 1; t.closes_ms }
        7 => { m.past_close += 1; t.closes_ms + 1 + r.below(HOUR as u128) as u64 }
        8 => { m.backwards += 1; at.saturating_sub(r.below(3 * t.step_ms as u128) as u64) }
        _ => { m.on_boundary += 1; on }
    }
}

#[test]
fn the_skip_is_the_walk_field_for_field() {
    let mut r = Rng(0x9B05_688C_2B3E_6C1F);
    let mut m = Met::default();
    for road in 0..1_200u64 {
        let t = some_terms(&mut r);
        if t.check().is_err() { continue; }
        m.roads += 1;
        let (mut np, mut nf) = (Branch::open(&t), Branch::open(&t));
        // sometimes a branch whose band is zero wide (an observation too small for its lag limit to move: it stands, whatever the spot); one
        // whose observation times the step leaves u128; or one at rest at a price past the law's range (both outside the range: the walk,
        // never the skip)
        let short = (t.closes_ms - t.opens_ms) / t.step_ms < 2_000;
        match r.below(20) {
            0 => { np.obs = r.below(BP / t.max_step_bp as u128); nf.obs = r.below(BP / t.max_step_bp as u128 + 1); }
            1 if short => { np.obs = u128::MAX / (1 + r.below(4)); }
            2 if short => { nf.base = 1 + r.below(1_000); nf.quote = AMOUNT_MAX; nf.obs = nf.spot(); }
            _ => {}
        }
        let (mut op, mut of) = (np, nf);
        let big = Purse { quote: AMOUNT_MAX, base: AMOUNT_MAX };
        let (mut npurse, mut opurse) = ([big, big], [big, big]);
        let mut clock = t.opens_ms;
        for _ in 0..(6 + r.below(18)) {
            let now = some_instant(&mut m, &mut r, &t, clock);
            clock = clock.max(now.min(t.closes_ms));
            match r.below(3) {
                // a trade that moves the pool, taken by both on the same stamp: the old one carried by the loop as it stood first
                0 if now >= t.opens_ms && now < t.closes_ms => {
                    let which = if r.below(2) == 0 { Which::Pass } else { Which::Fail };
                    let side = if r.below(2) == 0 { Side::Buy } else { Side::Sell };
                    let b = match which { Which::Pass => &np, Which::Fail => &nf };
                    let share = [2, 20, 1_000][r.below(3) as usize];
                    let amount = 1 + r.below(match side { Side::Buy => b.quote, Side::Sell => b.base } / share);
                    let tr = Trade { branch: which, side, amount, min_out: 0 };
                    let i = (which == Which::Fail) as usize;
                    let ob = match which { Which::Pass => &op, Which::Fail => &of };
                    let asked = traded(&t, ob, &opurse[i], &tr);
                    let new = trade_on(&t, &mut np, &mut nf, &mut npurse[i], &tr, now);
                    if asked.is_ok() { advance_as_it_stood(&mut op, now, &t); advance_as_it_stood(&mut of, now, &t); }
                    let old = trade_on(&t, &mut op, &mut of, &mut opurse[i], &tr, now);
                    assert_eq!(new, old, "road {road}: the same trade, the same answer");
                    m.trades += new.is_ok() as u64;
                }
                // an unbounded carry, as the fold and every whole call of the law carry
                1 => {
                    for (n, o) in [(&mut np, &mut op), (&mut nf, &mut of)] {
                        met_state(&mut m, n, &t);
                        let (w, s, l) = count(o, now, &t);
                        n.advance(now, &t);
                        advance_as_it_stood(o, now, &t);
                        m.carries += 1; m.walked += w; m.skipped += s; m.longest = m.longest.max(l); m.skipping += (s > 0) as u64;
                    }
                }
                // bounded carries until they stand, as a program on a chain sends them: each call held against the old loop carried to where it
                // stopped, a call short of the instant walked exactly its bound
                _ => {
                    let most = match r.below(5) { 0 => 0, 1 => 1, 2 => u64::MAX, _ => r.below(100) as u64 };
                    let end = now.clamp(t.opens_ms, t.closes_ms);
                    let mut calls = 0u64;
                    loop {
                        let (bp, bf) = (np, nf);
                        met_state(&mut m, &np, &t); met_state(&mut m, &nf, &t);
                        let both = carry(&t, &mut np, &mut nf, now, most);
                        calls += 1; m.bounded_calls += 1;
                        for (n, o, before) in [(&np, &mut op, bp), (&nf, &mut of, bf)] {
                            let (w, s, l) = count(&before, n.at, &t);
                            advance_as_it_stood(o, n.at, &t);
                            assert_eq!(*n, *o, "road {road}: a bounded carry at most {most} toward {now} stopped where the old loop stands");
                            m.carries += 1; m.walked += w; m.skipped += s; m.longest = m.longest.max(l); m.skipping += (s > 0) as u64;
                            if n.at < end {
                                m.short_calls += 1;
                                assert_eq!(w, most.max(1), "road {road}: a call short of the instant walked exactly its bound {most}");
                            } else {
                                assert!(w <= most.max(1), "road {road}: a call walked {w} boundaries over its bound {most}");
                            }
                        }
                        assert_eq!(both, np.at >= end && nf.at >= end, "road {road}: the carry says true when both stand at the instant");
                        if both { break; }
                        assert!(calls < 1_000_000, "road {road}: a bounded carry reaches the instant");
                    }
                    // and one unbounded carry of where they started is where the bounded ones ended (the old loop is it)
                }
            }
            assert_eq!((np, nf), (op, of), "road {road}: the skip and the walk leave equal branches");
            assert_eq!(npurse, opurse, "road {road}: and equal purses");
        }
        // the close, then the decision, by the one call
        let d_new = decide_at(&t, &mut np, &mut nf, t.closes_ms);
        advance_as_it_stood(&mut op, t.closes_ms, &t); advance_as_it_stood(&mut of, t.closes_ms, &t);
        assert_eq!((np, nf), (op, of), "road {road}: at the close");
        assert_eq!(d_new.state, if op.at == of.at && op.cum.saturating_mul(BP) > of.cum.saturating_mul(BP + t.threshold_bp as u128) { State::Passed } else { State::Failed }, "road {road}: the decision");
    }
    eprintln!("  the sweep: {m:?}");
    assert!(m.roads >= 1_000 && m.trades > 1_000 && m.short_calls > 1_000 && m.skipping > 1_000 && m.longest > 10_000, "the sweep met the long and the short: {m:?}");
    for (what, n) in [("at rest from the opening", m.rest_from_opening), ("a partial acc", m.partial_acc), ("chasing up", m.chasing_up), ("chasing down", m.chasing_down), ("a band zero wide", m.zero_band), ("an observation past the law's range", m.huge_obs),
        ("before the opening", m.before_opening), ("mid-step", m.mid_step), ("on a boundary", m.on_boundary), ("one before", m.one_before), ("one after", m.one_after), ("at the close", m.at_close), ("past it", m.past_close), ("backwards", m.backwards)] {
        assert!(n >= 10, "the sweep met {what} only {n} times");
    }
}

#[test]
fn bounded_carries_until_true_are_one_carry() {
    let mut r = Rng(0x1F83_D9AB_FB41_BD6B);
    let mut short = 0u64;
    for _ in 0..600 {
        let t = some_terms(&mut r);
        if t.check().is_err() || (t.closes_ms - t.opens_ms) / t.step_ms > 5_000 { continue; }
        let mut b = Branch::open(&t);
        // a spot a trade moved, by a factor of up to a hundred either way, maybe part of the way through a step
        b.quote = (b.quote * (1 + r.below(100))).min(AMOUNT_MAX); b.base = (b.base * (1 + r.below(100))).min(AMOUNT_MAX);
        if r.below(2) == 0 { let x = t.opens_ms + r.below((t.closes_ms - t.opens_ms) as u128) as u64; b.advance(x, &t); b.quote = (b.quote / 2).max(1); }
        let now = t.opens_ms + r.below((t.closes_ms - t.opens_ms) as u128 * 2) as u64;
        let mut one = b;
        one.advance(now, &t);
        let most = r.below(70) as u64;
        let mut c = b;
        loop {
            let before = c;
            let stands = c.walk(now, most, &t);
            let (w, _, _) = count(&before, c.at, &t);
            if stands { assert!(w <= most.max(1)); break; }
            short += 1;
            assert_eq!(w, most.max(1), "a call short of the instant walked exactly {} boundaries", most.max(1));
            let mut o = before;
            advance_as_it_stood(&mut o, c.at, &t);
            assert_eq!(c, o, "it stopped where advance to its own instant stands");
        }
        assert_eq!(c, one, "at most {most} walked a call, until true, is one carry");
    }
    eprintln!("  {short} bounded calls stopped short of the instant, each having walked exactly its bound");
    assert!(short > 100, "{short} calls short of the instant");
}

#[test]
fn a_branch_at_rest_reaches_any_instant_in_one_call() {
    let mut r = Rng(0x5BE0_CD19_137E_2179);
    for _ in 0..2_000 {
        let t = some_terms(&mut r);
        if t.check().is_err() { continue; }
        let now = t.opens_ms.saturating_sub(HOUR) + r.below((t.closes_ms - t.opens_ms + 2 * HOUR) as u128) as u64;
        let (mut p, mut f) = (Branch::open(&t), Branch::open(&t));
        let (mut op, mut of) = (p, f);
        assert!(carry(&t, &mut p, &mut f, now, 1), "a road nobody touched stands at {now} after one call at a bound of 1");
        advance_as_it_stood(&mut op, now, &t); advance_as_it_stood(&mut of, now, &t);
        assert_eq!((p, f), (op, of));
    }
}

#[test]
fn a_branch_at_the_end_of_its_window_is_ended_and_never_handed_the_close_again() {
    let mut r = Rng(0x6A09_E667_BB67_AE85);
    for _ in 0..300 {
        let t = some_terms(&mut r);
        if t.check().is_err() || (t.closes_ms - t.opens_ms) / t.step_ms > 3_000 { continue; }
        // a spot moved a hundredfold a few steps before the close: the chase outlasts the window
        let mut b = Branch::open(&t);
        let last = t.opens_ms + ((t.closes_ms - 1 - t.opens_ms) / t.step_ms) * t.step_ms;
        b.advance(last.saturating_sub(r.below(4) as u64 * t.step_ms), &t);
        b.quote = (b.quote * 100).min(AMOUNT_MAX);
        let (left, _, _) = count(&b, t.closes_ms, &t);
        let (mut one, mut c, mut calls) = (b, b, 0u64);
        one.advance(t.closes_ms + HOUR, &t);
        while !c.walk(t.closes_ms + HOUR, 1, &t) { calls += 1; }
        assert_eq!((c, calls + 1), (one, left.max(1)), "the last boundaries walked one a call, the close in the call that walks the last");
        for _ in 0..5 { assert!(c.walk(t.closes_ms + 7 * HOUR, 1, &t)); assert_eq!(c, one, "at the close a carry stands and moves nothing"); }
    }
    // a carry with a bound of 0 still moves: a chasing branch walks one boundary
    let t = Terms { opens_ms: 0, closes_ms: 100 * MIN, step_ms: MIN, max_step_bp: 100, threshold_bp: 0, pool_base: UNIT, pool_quote: UNIT, budget: UNIT, both_ways: false, fee_bp: 0 };
    let mut b = Branch::open(&t);
    b.quote *= 3;
    assert!(!b.walk(t.closes_ms, 0, &t));
    assert_eq!((b.steps, b.at), (1, MIN));
}

#[test]
fn the_long_road_nobody_touched_is_carried_to_its_close_at_once() {
    // a window of 366 days at the step's floor, a second: 31,622,400 steps. Untouched, both branches stand at the opening price for all of it.
    for short in [0u64, 1, 999] {
        let opens = 1_000_000u64;
        let t = Terms { opens_ms: opens, closes_ms: opens + WINDOW_MAX_MS - short, step_ms: STEP_MIN_MS, max_step_bp: 100, threshold_bp: 300, pool_base: 1000 * UNIT, pool_quote: 2500 * UNIT, budget: UNIT, both_ways: true, fee_bp: 30 };
        t.check().unwrap();
        let (mut p, mut f) = (Branch::open(&t), Branch::open(&t));
        let obs = spot(t.pool_base, t.pool_quote).unwrap();
        assert!(carry(&t, &mut p, &mut f, t.closes_ms, 1), "one call at a bound of 1 carries the year");
        let window = (t.closes_ms - t.opens_ms) as u128;
        // every boundary before the close is applied: the last at opens + (steps) * step, the first boundary at or past the close is not
        let steps = (t.closes_ms - 1 - t.opens_ms) / t.step_ms;
        let tail = (t.closes_ms - t.opens_ms) - steps * t.step_ms;
        for b in [p, f] {
            assert_eq!(b.steps, steps, "every boundary before the close, and none at it");
            assert_eq!(b.at, t.closes_ms, "it stands at the close");
            assert_eq!(b.obs, obs, "the observation never moved off the opening price");
            assert_eq!(b.cum, obs * window, "the sum is the opening price times the whole window");
            assert_eq!(b.acc, obs * tail as u128, "the last step, from the last boundary to the close, held at the opening price");
            assert_eq!(b.twap(&t), Some(obs));
            assert_eq!((b.base, b.quote, b.fees), (t.pool_base, t.pool_quote, 0));
        }
        assert_eq!((steps, tail), match short { 0 => (31_622_399, 1_000), 1 => (31_622_399, 999), _ => (31_622_399, 1) });
        let d = decide_at(&t, &mut p, &mut f, t.closes_ms);
        assert_eq!((d.state, d.twap_pass, d.twap_fail), (State::Failed, Some(obs), Some(obs)), "nobody traded: PASS does not beat FAIL by the threshold");
    }
}

/// from `b`, to each instant of `tos` in turn: one unbounded carry, and bounded walks of at most `most` boundaries a call until they stand,
/// each call held equal field for field to the loop as it stood carried to where it stopped
fn held_to_the_loop_as_it_stood(b: Branch, tos: &[u64], most: u64, t: &Terms, what: &str) {
    let (mut one, mut bounded, mut old) = (b, b, b);
    for &to in tos {
        one.advance(to, t);
        let mut calls = 0u32;
        loop {
            let stands = bounded.walk(to, most, t);
            let mut o = old;
            advance_as_it_stood(&mut o, bounded.at, t);
            assert_eq!(bounded, o, "{what}: a walk of at most {most} boundaries toward {to} stopped where the loop as it stood stands");
            calls += 1;
            if stands { break; }
            assert!(calls < 10_000, "{what}: the walk toward {to} ends");
        }
        advance_as_it_stood(&mut old, to, t);
        assert_eq!((one, bounded), (old, old), "{what}: carried to {to}");
    }
}

#[test]
fn states_no_road_reaches_are_walked_as_the_loop_stood() {
    // Two states the law never makes, kept so the guards that send them to the loop as it stood are held: a held sum on a branch standing on its
    // boundary (a boundary empties it), and an observation climbing past the law's range on reserves past it (the law refuses a trade that would
    // leave its range). In both the skip and the walk must leave what the old loop leaves.
    let t = Terms { opens_ms: 0, closes_ms: 60 * 4 * HOUR, step_ms: 4 * HOUR, max_step_bp: 10_000, threshold_bp: 0, pool_base: UNIT, pool_quote: UNIT, budget: UNIT, both_ways: false, fee_bp: 0 };
    t.check().unwrap();
    // carried by several instants and by one, a boundary a call and fifty: a guard that holds only inside one call is met inside one call
    let (instants, at_once) = ([3 * t.step_ms, 10 * t.step_ms + 7, 40 * t.step_ms, t.closes_ms], [t.closes_ms]);
    for (tos, most) in [(&instants[..], 1u64), (&instants[..], 50), (&at_once[..], 1), (&at_once[..], 50)] {
        // a held sum on the opening's boundary: at rest, and chasing a spot a trade moved
        for quote in [UNIT, 3 * UNIT] {
            let mut b = Branch::open(&t);
            b.quote = quote;
            b.acc = 12_345 * t.step_ms as u128;
            held_to_the_loop_as_it_stood(b, tos, most, &t, &format!("a held sum on the boundary, quote {quote}"));
        }
        // an observation at the top of the law's range under a spot near the top of u128: at 10000 bp it doubles a step and leaves the range
        // at the first; carried on in the same call, its step's held sum needs more than u128 from about the thirty-fourth
        let mut b = Branch::open(&t);
        b.base = 1;
        b.quote = u128::MAX / PRICE;
        b.obs = PRICE_HI;
        held_to_the_loop_as_it_stood(b, tos, most, &t, "an observation climbing past the law's range");
    }
}
