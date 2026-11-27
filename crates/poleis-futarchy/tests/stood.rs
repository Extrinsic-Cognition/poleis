//! THE PRICE THAT STOOD, on the core itself (law.rs Branch::advance): a price counts for the time it stood, held inside the lag band around the
//! observation. A branch's spot is moved here by setting its `base` and `quote`, as tests/law.rs does; the core alone runs these (no feature).
use poleis_futarchy::law::*;

const MIN: u64 = 60_000;
/// a road of three days at a step a minute, 100 bp
fn terms() -> Terms { Terms { opens_ms: 1_000_000, closes_ms: 1_000_000 + 3 * 86_400_000, step_ms: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 1000 * UNIT, pool_quote: 1000 * UNIT, budget: 1000 * UNIT, both_ways: false } }
/// a branch's pool set to stand at `price` (times PRICE / 1000, so 1000 is the opening price of `terms`)
fn stand(b: &mut Branch, milli: u128) { b.base = 1000 * UNIT; b.quote = milli * UNIT; }
fn boundary(t: &Terms, k: u64) -> u64 { t.opens_ms + k * t.step_ms }

/// xorshift: a seeded sweep with no dependency (tests/law.rs's)
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn below(&mut self, n: u128) -> u128 { (((self.next() as u128) << 64) | self.next() as u128) % n.max(1) } }

#[test]
fn a_quiet_pool_is_the_law_as_it_was() {
    // while the spot does not change, the held sum is the held spot times the whole step, the division is exact, and the observation is
    // step_toward(obs, spot) at every boundary: the tick's rule, byte for byte, however the carry is cut
    let t = terms();
    let mut r = Rng(0x3C6E_F372_FE94_F82B);
    for _ in 0..400 {
        let mut b = Branch::open(&t);
        b.base = (1 + r.below(5_000)) * UNIT; b.quote = (1 + r.below(5_000)) * UNIT;
        let (spot, mut obs, mut cum) = (b.spot(), b.obs, 0u128);
        let steps = 1 + r.below(300) as u64;
        while b.steps < steps {
            let to = (b.at + 1 + r.below(3 * MIN as u128) as u64).min(boundary(&t, steps));
            let before = b.steps;
            b.advance(to, &t);
            for _ in before..b.steps { cum += obs * MIN as u128; obs = step_toward(obs, spot, t.max_step_bp); }
            if b.at == boundary(&t, b.steps) { assert_eq!((b.obs, b.cum, b.acc), (obs, cum, 0), "on a boundary the quiet pool is the tick's rule"); }
        }
    }
}

#[test]
fn a_trade_counts_from_its_own_stamp() {
    // two roads that differ only by a trade stamped exactly on a boundary have the same observation after that boundary
    let t = terms();
    for k in [1u64, 2, 17, 4319] {
        let at = boundary(&t, k);
        let (mut p, mut f, mut purse) = (Branch::open(&t), Branch::open(&t), t.purse_of(t.budget));
        let (mut qp, mut qf) = (p, f);
        // something stood before: a buy a third of a step before the boundary
        let early = Trade { branch: Which::Pass, side: Side::Buy, amount: 50 * UNIT, min_out: 0 };
        let mut qpurse = purse;
        trade_on(&t, &mut p, &mut f, &mut purse, &early, at - MIN / 3).unwrap();
        trade_on(&t, &mut qp, &mut qf, &mut qpurse, &early, at - MIN / 3).unwrap();
        let on = Trade { branch: Which::Pass, side: Side::Buy, amount: 300 * UNIT, min_out: 0 };
        trade_on(&t, &mut p, &mut f, &mut purse, &on, at).unwrap();
        decide_at(&t, &mut qp, &mut qf, at);
        assert_eq!((p.obs, p.cum, p.acc, p.steps, p.at), (qp.obs, qp.cum, qp.acc, qp.steps, qp.at), "boundary {k}: the step that ended was whole when the clock reached it");
        assert_eq!(f, qf);
    }
}

#[test]
fn a_trade_undone_at_the_same_stamp_counted_for_nothing() {
    let t = terms();
    let mut r = Rng(0xA54F_F53A_5F1D_36F1);
    for _ in 0..500 {
        let at = t.opens_ms + r.below((t.closes_ms - t.opens_ms) as u128 - 2 * MIN as u128) as u64;
        let (mut p, mut f, mut purse) = (Branch::open(&t), Branch::open(&t), t.purse_of(t.budget));
        // a road that already moved: a buy somewhere before
        let first = t.opens_ms + r.below((at - t.opens_ms) as u128 + 1) as u64;
        trade_on(&t, &mut p, &mut f, &mut purse, &Trade { branch: Which::Pass, side: Side::Buy, amount: (1 + r.below(300)) * UNIT, min_out: 0 }, first).unwrap();
        let (mut qp, mut qf) = (p, f);
        let base_before = purse.base;
        trade_on(&t, &mut p, &mut f, &mut purse, &Trade { branch: Which::Pass, side: Side::Buy, amount: (1 + r.below(600)) * UNIT, min_out: 0 }, at).unwrap();
        let bought = purse.base - base_before;
        trade_on(&t, &mut p, &mut f, &mut purse, &Trade { branch: Which::Pass, side: Side::Sell, amount: bought, min_out: 0 }, at).unwrap();
        carry(&t, &mut qp, &mut qf, at, u64::MAX);
        assert_eq!((p.acc, p.obs, p.cum, p.at, p.steps), (qp.acc, qp.obs, qp.cum, qp.at, qp.steps), "a buy and its sale at one instant left the branch as if neither happened");
        // after it the pool differs by the round trip's rounding alone, a few micro of $WARM in the pool's favour: the next boundary within
        // a hundred millionth
        let next = boundary(&t, p.steps + 1);
        decide_at(&t, &mut p, &mut f, next); decide_at(&t, &mut qp, &mut qf, next);
        assert!(p.obs.abs_diff(qp.obs) * 100_000_000 <= qp.obs, "the round trip's rounding moved the next boundary by {} of {}", p.obs.abs_diff(qp.obs), qp.obs);
    }
}

#[test]
fn a_spikes_size_past_the_band_does_not_matter() {
    // a spot of 4 and a spot of 400 standing for the same second of a step give the same observation; a raw mean would have moved it 165 %
    let t = terms();
    let obs = |spike: u128| {
        let mut b = Branch::open(&t);
        b.advance(boundary(&t, 10) + 20_000, &t);
        stand(&mut b, spike * 1000);
        b.advance(boundary(&t, 10) + 21_000, &t);
        stand(&mut b, 1000);
        b.advance(boundary(&t, 11), &t);
        b.obs
    };
    let open = Branch::open(&t).obs;
    let (four, hundred, four_hundred) = (obs(4), obs(100), obs(400));
    assert_eq!((four, hundred), (four_hundred, four_hundred), "the spike counts as one band for the second it stood, whatever its size");
    assert_eq!(four, open + open / 100 / 60, "one band for one second in sixty: a sixtieth of a step");
    let raw = (open * 59 + 100 * open) / 60;
    assert!(raw >= open * 265 / 100, "a mean of the raw spot would have lifted it from {open} to {raw}");
}

/// the observation at every boundary of a whole window, the spot pushed to `far` for the first `share` of each step and the quiet `p` the rest
fn pushed_for(t: &Terms, share: u64, far: u128, quiet: u128) -> Vec<u128> {
    let mut b = Branch::open(t);
    let mut out = vec![b.obs];
    let mut k = 0;
    while boundary(t, k + 1) < t.closes_ms {
        stand(&mut b, far);
        b.advance(boundary(t, k) + share, t);
        stand(&mut b, quiet);
        b.advance(boundary(t, k + 1), t);
        out.push(b.obs);
        k += 1;
    }
    out
}

#[test]
fn under_half_the_time_one_band_at_most() {
    // a price pushed far for a share f of every step, the quiet price p the rest, settles the observation near p + d * f / (1 - f), for ever
    let t = terms();
    let p = Branch::open(&t).obs;
    for (share, f_num, f_den) in [(MIN / 60, 1u128, 60u128), (MIN / 4, 1, 4)] {
        let seen = pushed_for(&t, share, 400_000, 1000);
        // the fixed point of obs = (1 - f) p + f (obs + obs / 100): p (1 - f) / (1 - 1.01 f)
        let fixed = p * (f_den - f_num) * 100 / (100 * f_den - 101 * f_num);
        let highest = *seen.iter().max().unwrap();
        let last = *seen.last().unwrap();
        eprintln!("  pushed far {f_num}/{f_den} of every step for {} steps: the observation at most {highest}, {last} at the end, the fixed point {fixed}, p + d f / (1 - f) = {}",
            seen.len() - 1, p + p / 100 * f_num / (f_den - f_num));
        assert!(highest <= fixed + 1, "f = {f_num}/{f_den}: never past the fixed point {fixed}: {highest}");
        assert!(last + 1_000 >= fixed, "f = {f_num}/{f_den}: it settles there: {last}");
        if f_den == 60 { assert!(highest * 10_000 <= p * 10_002, "one second in sixty: under 0.02 % of p, for ever: {highest}"); }
        // that is p + d * f / (1 - f) with d the band of the observation itself, to a unit of the floor
        assert!((highest - p) * 100 * (f_den - f_num) <= highest * f_num + 100 * (f_den - f_num), "f = {f_num}/{f_den}: within p + d f / (1 - f)");
    }
}

#[test]
fn over_half_the_time_it_walks_slowly_and_only_while_held() {
    // at f = 3/4 the observation moves about (2f - 1) d a step: up, and slower than a price held the whole step (d a step)
    let t = terms();
    let far = 400_000;
    let seen = pushed_for(&t, 3 * MIN / 4, far, 1000);
    let held = pushed_for(&t, MIN, far, 1000);
    let mut steady = 0;
    for k in 1..seen.len() {
        let (was, now) = (seen[k - 1], seen[k]);
        if now + now / 100 >= far * PRICE / 1000 { break; }
        let d = was / 100;
        assert!(now > was && now - was < d, "step {k}: it walks up, and less than one band: {was} to {now}");
        assert!(now - was >= d / 2, "step {k}: by at least (2f - 1) d: {was} to {now}");
        if was - d >= PRICE { assert_eq!(now - was, d / 2, "step {k}: once the quiet price is under the band, exactly (2f - 1) d"); steady += 1; }
        assert!(now < held[k], "step {k}: slower than the price held: {now} against {}", held[k]);
        if held[k - 1] + held[k - 1] / 100 <= far * PRICE / 1000 { assert_eq!(held[k], held[k - 1] + held[k - 1] / 100, "step {k}: a price held the whole step moves d, as the tick's rule did"); }
    }
    eprintln!("  pushed far 3/4 of every step: {steady} steps at exactly d / 2; after a day at {} against the held price's {}", seen[1440], held[1440]);
    assert!(steady > 100);
}

#[test]
fn the_observation_never_leaves_the_band_the_range_or_what_stood() {
    // after a boundary it lies between min(obs, the step's lowest spot) and max(obs, its highest), within max_step_bp of the observation
    // before it, inside the law's range; with spots set anywhere in the range, at random instants, carried in random pieces
    let mut r = Rng(0x510E_527F_ADE6_82D1);
    let mut boundaries = 0u64;
    for _ in 0..300 {
        let step_ms = [1_000, 7_000, MIN, 5 * MIN][r.below(4) as usize];
        let t = Terms { closes_ms: 1_000_000 + 400 * step_ms, step_ms, max_step_bp: [1, 25, 100, 10_000][r.below(4) as usize], ..terms() };
        t.check().unwrap();
        let mut b = Branch::open(&t);
        let set = |b: &mut Branch, r: &mut Rng| loop {
            let (base, quote) = (1 + r.below(AMOUNT_MAX), 1 + r.below(AMOUNT_MAX));
            if let Some(p) = spot(base, quote) { if (PRICE_LO..=PRICE_HI).contains(&p) { b.base = base; b.quote = quote; return; } }
        };
        let (mut lo, mut hi) = (b.spot(), b.spot());
        let mut obs = b.obs;
        while boundary(&t, b.steps + 1) < t.closes_ms {
            // carried in random pieces, never past the next boundary in one call, so each boundary is read against its own step's spots
            let to = (b.at + 1 + r.below(step_ms as u128) as u64).min(boundary(&t, b.steps + 1));
            let before = b.steps;
            b.advance(to, &t);
            if b.steps > before {
                let d = obs * t.max_step_bp as u128 / BP;
                assert!(b.obs >= obs - d && b.obs <= obs + d, "a boundary moved {obs} to {}, past {} bp", b.obs, t.max_step_bp);
                assert!(b.obs >= obs.min(lo) && b.obs <= obs.max(hi), "a boundary took {} from {obs}, outside what stood: {lo} to {hi}", b.obs);
                assert!((PRICE_LO..=PRICE_HI).contains(&b.obs), "the observation {} left the law's range", b.obs);
                boundaries += 1;
                obs = b.obs;
                lo = b.spot(); hi = b.spot();
            }
            if r.below(2) == 0 { set(&mut b, &mut r); lo = lo.min(b.spot()); hi = hi.max(b.spot()); }
        }
    }
    eprintln!("  {boundaries} boundaries read, every one inside its band and what stood");
}

#[test]
fn what_the_law_cannot_do_alone_and_what_another_trader_does() {
    // WHAT THE LAW CANNOT DO ALONE (law.rs): a road nobody else trades is decided by whoever trades it, for nothing.
    let buy = |branch, amount| Trade { branch, side: Side::Buy, amount, min_out: 0 };
    let sell = |branch, amount| Trade { branch, side: Side::Sell, amount, min_out: 0 };
    // 1. Lift PASS at the opening, hold it through the window, sell it back an hour before the close. No trade stood between, so the pool gives
    //    the $WARM back to the rounding; the road passes; no claim is held for the receipts to mark, whatever they say.
    let t = terms();
    let (mut p, mut f, mut mine) = (Branch::open(&t), Branch::open(&t), t.purse_of(t.budget));
    let got = trade_on(&t, &mut p, &mut f, &mut mine, &buy(Which::Pass, 900 * UNIT), t.opens_ms).unwrap();
    trade_on(&t, &mut p, &mut f, &mut mine, &sell(Which::Pass, got), t.closes_ms - 60 * MIN).unwrap();
    let d = decide_at(&t, &mut p, &mut f, t.closes_ms);
    assert_eq!((d.state, mine.base), (State::Passed, 0), "alone, it lifted PASS, held it, left before the close: the road passes and it holds no claim");
    for worth in [0, PRICE / 10, PRICE] {
        let r = settle_at(&t, &d, &Receipts { world: Which::Pass, worth }, t.budget, &mine, &t.purse_of(t.budget)).unwrap();
        assert!((-2..=0).contains(&r), "receipts of {worth}: it settles at the round trip's rounding, {r} micro");
    }
    // 2. On a road both ways, sell the whole FAIL claim at the opening and hold. FAIL's average falls, PASS beats it, the road passes, and the
    //    FAIL world did not happen: by the law's own word that trade counts for nothing. The settle is exactly 0.
    let t = Terms { both_ways: true, ..terms() };
    let opens = t.purse_of(t.budget);
    let (mut p, mut f, mut in_fail) = (Branch::open(&t), Branch::open(&t), opens);
    trade_on(&t, &mut p, &mut f, &mut in_fail, &sell(Which::Fail, opens.base), t.opens_ms).unwrap();
    let d = decide_at(&t, &mut p, &mut f, t.closes_ms);
    assert_eq!(d.state, State::Passed, "alone, it pushed FAIL down and PASS beat it");
    for worth in [0, PRICE / 2, PRICE] {
        assert_eq!(settle_at(&t, &d, &Receipts { world: Which::Pass, worth }, t.budget, &opens, &in_fail), Ok(0), "the world it traded did not happen: nothing");
    }
    // WHAT ANOTHER TRADER DOES: the same push, and a second purse of the same size that buys FAIL back AN HOUR LATER, not a tick later. The
    // push stood an hour of three days. FAIL ends dearer than it opened, the road FAILS, the FAIL world happens, and the receipts mark what
    // each holds there: whoever sold the claim for less than it was worth pays the one who bought it.
    let (mut p, mut f, mut pusher, mut other) = (Branch::open(&t), Branch::open(&t), opens, opens);
    let fetched = trade_on(&t, &mut p, &mut f, &mut pusher, &sell(Which::Fail, opens.base), t.opens_ms).unwrap();
    trade_on(&t, &mut p, &mut f, &mut other, &buy(Which::Fail, opens.quote), t.opens_ms + 60 * MIN).unwrap();
    let d = decide_at(&t, &mut p, &mut f, t.closes_ms);
    assert_eq!(d.state, State::Failed, "answered within the hour, the push did not pass the road");
    assert!(f.spot() > Branch::open(&t).spot(), "a purse of the same size, answering after it, more than undoes it");
    let worth = PRICE * 9 / 10;
    let rc = Receipts { world: Which::Fail, worth };
    let (pusher_result, other_result) = (settle_at(&t, &d, &rc, t.budget, &opens, &pusher).unwrap(), settle_at(&t, &d, &rc, t.budget, &opens, &other).unwrap());
    let pool = pool_at(&t, &d, &rc, &p, &f).unwrap();
    eprintln!("  a claim worth 0.9: the pusher sold {} of it for {fetched} and settles at {pusher_result}; the one who answered settles at {other_result}; the pool at {pool}", opens.base);
    // it sold its whole claim, worth 0.9 a claim, for what the pool gave: its loss is exactly that, to the floor
    assert_eq!(pusher_result, fetched as i128 - worth_of(opens.base, worth) as i128, "the pusher pays the difference between what its claim was worth and what it sold it for");
    assert!(pusher_result < 0 && other_result > 0, "the pusher lost {pusher_result}, the one who answered won {other_result}");
    assert!((pusher_result + other_result + pool).abs() <= 2, "what the two won and lost between them the pool carries, to the rounding");
}
