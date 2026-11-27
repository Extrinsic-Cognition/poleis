//! THE FEE IS $WARM, BESIDE THE POOL (law.rs THE FEE): a buy pays its fee before what it pays reaches the pool, a sell pays it from what it
//! receives after that has left it, and each branch keeps the fees in one counter. With no fee the law is the law as it was; with one the pool's
//! price moves only with a position, a round trip leaves the pool where it was and costs both fees, $WARM is conserved with the fees, and the
//! house's take at the settle is a number of its own. The rule it replaces is kept here as its witness: a fee kept IN the reserve, and a fee kept
//! in claims. The core alone runs the tests that need no list.
use embertide_futarchy::law::*;

const MIN: u64 = 60_000;
fn terms(fee_bp: u32, both_ways: bool) -> Terms { Terms { opens_ms: 1_000_000, closes_ms: 1_000_000 + 3 * 86_400_000, step_ms: MIN, max_step_bp: 100, threshold_bp: 300, pool_base: 1000 * UNIT, pool_quote: 1000 * UNIT, budget: 1000 * UNIT, both_ways, fee_bp } }

/// xorshift: a seeded sweep with no dependency (tests/law.rs's)
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn below(&mut self, n: u128) -> u128 { (((self.next() as u128) << 64) | self.next() as u128) % n.max(1) } }

/// the law's `traded` as it stood before the fee, word for word: the swap of the whole amount
fn traded_before(b: &Branch, mine: &Purse, tr: &Trade) -> Result<(u128, u128, u128), &'static str> {
    if tr.amount == 0 || tr.amount > AMOUNT_MAX { return Err(OUT_OF_RANGE); }
    if (match tr.side { Side::Buy => mine.quote, Side::Sell => mine.base }) < tr.amount { return Err(NOT_ENOUGH); }
    let out = swap_out(b.base, b.quote, tr.side, tr.amount).ok_or(OUT_OF_RANGE)?;
    if out == 0 { return Err(NOTHING_OUT); }
    if out < tr.min_out { return Err(PAST_MIN); }
    let (base, quote) = match tr.side { Side::Buy => (b.base - out, b.quote + tr.amount), Side::Sell => (b.base + tr.amount, b.quote - out) };
    if base == 0 || quote == 0 || base > AMOUNT_MAX || quote > AMOUNT_MAX { return Err(OUT_OF_RANGE); }
    match spot(base, quote) { Some(p) if (PRICE_LO..=PRICE_HI).contains(&p) => Ok((base, quote, out)), _ => Err(OUT_OF_RANGE) }
}

/// THE HOLE, the first fee's rule, in the same integers: the pool keeps `fee_of(amount)` of a trade's input IN its reserve; the net trades
#[cfg(feature = "fold")]
fn traded_fee_in_the_reserve(fee_bp: u32, base: u128, quote: u128, side: Side, amount: u128) -> (u128, u128, u128) {
    let out = swap_out(base, quote, side, amount - fee_of(amount, fee_bp)).unwrap();
    match side { Side::Buy => (base - out, quote + amount, out), Side::Sell => (base + amount, quote - out, out) }
}

/// a random branch, purse and trade, from small to the law's whole range
fn draw(r: &mut Rng, t: &Terms) -> (Branch, Purse, Trade) {
    let mut b = Branch::open(t);
    let scale = [100u128, 10_000 * UNIT, AMOUNT_MAX][r.below(3) as usize];
    b.base = 1 + r.below(scale); b.quote = 1 + r.below(scale);
    let mine = Purse { quote: r.below(scale + 1), base: r.below(scale + 1) };
    let side = if r.below(2) == 0 { Side::Buy } else { Side::Sell };
    let have = if side == Side::Buy { mine.quote } else { mine.base };
    let amount = match r.below(6) { 0 => 1 + r.below(20), 1 => have + 1 + r.below(10), _ => 1 + r.below(have.max(1)) };
    let min_out = if r.below(8) == 0 { r.below(scale) } else { 0 };
    (b, mine, Trade { branch: Which::Pass, side, amount, min_out })
}

#[test]
fn with_no_fee_the_law_is_the_law_as_it_was() {
    let t = terms(0, false);
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    let (mut ok, mut no) = (0u32, 0u32);
    for _ in 0..200_000 {
        let (b, mine, tr) = draw(&mut r, &t);
        let (now, then) = (traded(&t, &b, &mine, &tr), traded_before(&b, &mine, &tr));
        assert_eq!(now, then.map(|(base, quote, out)| (base, quote, out, 0)), "a fee of 0 on {tr:?} in {b:?}");
        if now.is_ok() { ok += 1 } else { no += 1 }
    }
    eprintln!("  200000 trades at a fee of 0: {ok} taken and {no} refused, each as the law before the fee answered it, with a fee of 0");
    assert!(ok > 10_000 && no > 10_000);
}

#[test]
fn the_pools_price_moves_only_with_a_position() {
    // no claim is ever in a fee: the pool's claims move exactly by the purse's, its $WARM by what the purse paid or got less the fee, and its
    // product only by its own rounding, under one unit of the reserve that grew
    let mut r = Rng(0x6A09_E667_F3BC_C908);
    let (mut ok, mut fees) = (0u32, 0u128);
    for _ in 0..200_000 {
        let fee_bp = [1u32, 30, 100, 2_500, 9_999][r.below(5) as usize];
        let t = terms(fee_bp, false);
        let (b, mine, tr) = draw(&mut r, &t);
        let Ok((base, quote, got, fee)) = traded(&t, &b, &mine, &tr) else { continue };
        ok += 1; fees += fee;
        assert!(got > 0 && got >= tr.min_out);
        let grew = match tr.side {
            Side::Buy => {
                assert_eq!(fee, fee_of(tr.amount, fee_bp), "a buy pays the fee of what it pays");
                assert_eq!((b.base - base, quote - b.quote), (got, tr.amount - fee), "the claims it took left the pool; only the net came in");
                quote
            }
            Side::Sell => {
                assert_eq!(fee, fee_of(got + fee, fee_bp), "a sell pays the fee of what left the pool");
                assert_eq!((base - b.base, b.quote - quote), (tr.amount, got + fee), "all the claims came in; the fee left with what the purse got");
                base
            }
        };
        let (k0, k1) = (b.base * b.quote, base * quote);
        assert!(k1 >= k0 && k1 - k0 < grew, "the product moves by its rounding alone: {k0} to {k1}");
    }
    eprintln!("  {ok} trades taken with a fee, {fees} micro of fees beside the pools, every one with the pool's claims moved exactly by the purse's");
    assert!(ok > 50_000);
}

#[test]
fn a_round_trip_leaves_the_pool_where_it_was_and_costs_both_fees() {
    // 900 in and back, pools of 1000, 100 bp
    let t = terms(100, false);
    let mut b = Branch::open(&t);
    let mut mine = t.purse_of(t.budget);
    let buy = Trade { branch: Which::Pass, side: Side::Buy, amount: 900 * UNIT, min_out: 0 };
    let (base, quote, got, fee1) = traded(&t, &b, &mine, &buy).unwrap();
    b.base = base; b.quote = quote; mine.quote -= 900 * UNIT; mine.base += got;
    let (base, quote, back, fee2) = traded(&t, &b, &mine, &Trade { side: Side::Sell, amount: got, ..buy }).unwrap();
    let lost = 900 * UNIT - back;
    eprintln!("  900 in at 100 bp: {got} claims out, sold back for {} less a fee of {fee2}: {back} back, {lost} lost against a f (2 - f) = 17910000; the pool left at {base} claims and {quote} $WARM", back + fee2);
    assert_eq!((fee1, got, back + fee2, fee2, back), (9 * UNIT, 471_179_270, 890_999_999, 8_910_000, 882_089_999));
    assert_eq!((lost, base, quote), (17_910_001, t.pool_base, t.pool_quote + 1), "both fees on the whole size and a micro of rounding; the pool where it was but that micro");
    // over a seeded sweep: the pool's claims where they were, its $WARM up by the round trip's rounding alone, and the purse short both fees
    let mut r = Rng(0xBB67_AE85_84CA_A73B);
    let (mut trips, mut worst) = (0u32, 0f64);
    for _ in 0..100_000 {
        let fee_bp = [1u32, 30, 100, 2_500, 9_999][r.below(5) as usize];
        let t = terms(fee_bp, false);
        let mut b = Branch::open(&t);
        b.base = UNIT + r.below(1_000_000 * UNIT); b.quote = UNIT + r.below(1_000_000 * UNIT);
        let a = 1 + r.below(3 * b.quote);
        let (q0, b0) = (b.quote, b.base);
        let Ok((base, quote, got, fee1)) = traded(&t, &b, &Purse { quote: a, base: 0 }, &Trade { branch: Which::Pass, side: Side::Buy, amount: a, min_out: 0 }) else { continue };
        b.base = base; b.quote = quote;
        let Ok((base, quote, back, fee2)) = traded(&t, &b, &Purse { quote: 0, base: got }, &Trade { branch: Which::Pass, side: Side::Sell, amount: got, min_out: 0 }) else { continue };
        trips += 1;
        // one claim of rounding on the way in, worth the pool's $WARM over its claims, and one micro on the way out
        let rounding = b.quote / b0 + 2;
        assert_eq!(base, b0, "the claims are back where they were");
        assert!(quote >= q0 && quote - q0 <= rounding, "the pool's $WARM where it was but the rounding: {q0} to {quote}");
        assert!(a - back >= fee1 + fee2 && a - back - fee1 - fee2 <= rounding, "{a} in, {back} back: both fees, {fee1} and {fee2}, and the rounding");
        let (f, af) = (fee_bp as f64 / 10_000.0, a as f64);
        let gap = ((a - back) as f64 - af * f * (2.0 - f)).abs();
        assert!(gap <= 3.0 + 2.0 * rounding as f64, "{a} in at {fee_bp} bp: lost {}, a f (2 - f) = {}", a - back, af * f * (2.0 - f));
        worst = worst.max(gap / af.max(1.0));
    }
    eprintln!("  {trips} round trips, every one leaving the pool's claims where they were and its $WARM within its rounding, every one short both fees; the loss against a f (2 - f): at most {worst:.2e} of the amount");
    assert!(trips > 50_000);
}

#[test]
fn a_fee_of_10000_bp_is_no_road() {
    assert_eq!(terms(10_000, false).check(), Err("the fee is 10000 bp or more"));
    assert_eq!(terms(u32::MAX, true).check(), Err("the fee is 10000 bp or more"));
    for ok in [0, 1, 100, 9_999] { assert_eq!(terms(ok, false).check(), Ok(()), "a fee of {ok} bp"); }
    assert_eq!((fee_of(10_000, 9_999), fee_of(1, 1), fee_of(1, 0), fee_of(AMOUNT_MAX, 9_999)), (9_999, 1, 0, AMOUNT_MAX - AMOUNT_MAX / 10_000));
    let t = terms(30, true);
    let b = Branch::open(&t);
    let p = t.purse_of(t.budget);
    assert_eq!(traded(&t, &b, &p, &Trade { branch: Which::Pass, side: Side::Buy, amount: 1, min_out: 0 }), Err(NOTHING_OUT), "a buy of one micro pays it whole and buys nothing");
    assert_eq!(traded(&t, &b, &p, &Trade { branch: Which::Pass, side: Side::Sell, amount: 1, min_out: 0 }), Err(NOTHING_OUT), "a sell of one claim receives under a micro");
    assert_eq!(traded(&t, &b, &p, &Trade { branch: Which::Pass, side: Side::Sell, amount: 3, min_out: 2 }), Err(PAST_MIN), "min_out is asked of what the purse gets, after the fee");
}

#[cfg(feature = "fold")]
fn entry(seq: u64, ts: u64, key: &str, branch: Which, side: Side, amount: u128) -> Entry {
    Entry { seq, ts, key: key.into(), seat: key.into(), ok: true, trade: Some(Trade { branch, side, amount, min_out: 0 }) }
}

#[cfg(feature = "fold")]
#[test]
fn warm_is_conserved_with_the_fees_and_claims_without_them() {
    let mut r = Rng(0x3C6E_F372_FE94_F82B);
    let (mut roads, mut taken, mut fees) = (0u32, 0u64, 0u128);
    for road in 0..160 {
        let both_ways = r.below(2) == 0;
        let t = Terms { opens_ms: 5_000, closes_ms: 5_000 + (2 + r.below(600) as u64) * MIN, step_ms: [7_000, 20_000, MIN, 5 * MIN][r.below(4) as usize], max_step_bp: [1, 25, 100, 10_000][r.below(4) as usize],
            threshold_bp: [0, 300, 10_000][r.below(3) as usize], fee_bp: [1, 30, 100, 9_999][r.below(4) as usize], pool_base: (1 + r.below(999_999)) * UNIT, pool_quote: (1 + r.below(999_999)) * UNIT, budget: (1 + r.below(999_999)) * UNIT, both_ways };
        if t.check().is_err() { continue; }
        let span = t.closes_ms - t.opens_ms;
        let mut ts = t.opens_ms;
        let opens = t.purse_of(t.budget);
        let es: Vec<Entry> = (0..60).map(|i| {
            ts += r.below(span as u128 / 60) as u64;
            let side = if r.below(3) == 0 { Side::Sell } else { Side::Buy };
            let most = if side == Side::Sell { opens.base } else { opens.quote };
            let amount = if r.below(4) == 0 { 1 + r.below(50) } else { 1 + r.below(most / 3 + 1) };
            entry(i + 1, ts, ["a", "b", "c", "d", "e"][r.below(5) as usize], if r.below(2) == 0 { Which::Pass } else { Which::Fail }, side, amount)
        }).collect();
        let m = market_of(&t, &es, t.closes_ms);
        let n = m.traders.len() as u128;
        for (b, pick) in [(&m.pass, 0), (&m.fail, 1)] {
            let (q, s): (u128, u128) = m.traders.iter().map(|h| if pick == 0 { h.pass } else { h.fail }).fold((0, 0), |a, p| (a.0 + p.quote, a.1 + p.base));
            assert_eq!(b.quote + q + b.fees, t.pool_quote + n * opens.quote, "road {road}: the $WARM of a branch is conserved, its fees beside the pool among it");
            assert_eq!(b.base + s, t.pool_base + n * opens.base, "road {road}: the claims of a branch are conserved, and none is in a fee");
            assert!(b.base * b.quote >= t.pool_base * t.pool_quote, "road {road}: a pool's product never falls");
            fees += b.fees;
        }
        let kept: Vec<Entry> = es.iter().filter(|e| !m.refused.iter().any(|x| x.seq == e.seq)).cloned().collect();
        let again = market_of(&t, &kept, t.closes_ms);
        assert_eq!((m.pass, m.fail, &m.traders), (again.pass, again.fail, &again.traders), "road {road}: a refused entry changes nothing");
        roads += 1; taken += m.accepted;
    }
    eprintln!("  {roads} roads with a fee, {taken} trades taken, {fees} micro of fees beside their pools; $WARM conserved with them and claims without them, to the micro");
    assert!(roads > 100 && taken > 3_000);
}

/// tests/law.rs's trader on this law: at every boundary it buys `amount` (all it has when None) of PASS and sells what it bought a second later,
/// while it can. The round trips, the branches and its purse.
#[cfg(feature = "fold")]
fn stands_a_second(t: &Terms, amount: Option<u128>) -> (u64, Branch, Branch, Purse) {
    let (mut pass, mut fail, mut mine) = (Branch::open(t), Branch::open(t), t.purse_of(t.budget));
    let mut trips = 0u64;
    for k in 1u64.. {
        let at = t.opens_ms + k * MIN;
        if at + 1_000 >= t.closes_ms { break; }
        let buy = Trade { branch: Which::Pass, side: Side::Buy, amount: amount.unwrap_or(mine.quote), min_out: 0 };
        if buy.amount > mine.quote { break; }
        let Ok(got) = trade_on(t, &mut pass, &mut fail, &mut mine, &buy, at) else { break };
        // a sale too small to receive a micro after its fee leaves its claims with the purse, and the trader stops
        if trade_on(t, &mut pass, &mut fail, &mut mine, &Trade { side: Side::Sell, amount: got, ..buy }, at + 1_000).is_err() { break; }
        trips += 1;
    }
    (trips, pass, fail, mine)
}

#[cfg(feature = "fold")]
#[test]
fn the_trader_who_stood_a_second_at_every_boundary_pays_the_house_and_moves_nothing() {
    let t = terms(100, false);
    let open = spot(t.pool_base, t.pool_quote).unwrap();
    // THE HOLE, the witness: the fee left in the reserve, 900 a round trip while the purse holds 900
    let (mut pass, mut fail, mut mine, mut trips) = (Branch::open(&t), Branch::open(&t), t.purse_of(t.budget), 0u64);
    for k in 1u64.. {
        let at = t.opens_ms + k * MIN;
        if at + 1_000 >= t.closes_ms || mine.quote < 900 * UNIT { break; }
        pass.advance(at, &t); fail.advance(at, &t);
        let (base, quote, got) = traded_fee_in_the_reserve(100, pass.base, pass.quote, Side::Buy, 900 * UNIT);
        pass.base = base; pass.quote = quote; mine.quote -= 900 * UNIT;
        pass.advance(at + 1_000, &t); fail.advance(at + 1_000, &t);
        let (base, quote, back) = traded_fee_in_the_reserve(100, pass.base, pass.quote, Side::Sell, got);
        pass.base = base; pass.quote = quote; mine.quote += back;
        trips += 1;
    }
    let d = decide_at(&t, &mut pass, &mut fail, t.closes_ms);
    eprintln!("  THE HOLE (the fee left in the reserve): {trips} round trips of 900, {} lost, PASS's spot left at {}, its average {:?} against {:?}: {:?}", t.budget - mine.quote, pass.spot(), d.twap_pass, d.needs, d.state);
    assert_eq!((trips, t.budget - mine.quote, pass.spot(), d.state), (11, 106_959_352, 1_106_959_352_000, State::Passed), "the witness: the fee in the reserve bought the verdict");
    // THIS LAW: the same trader, 900 a round trip while it can
    let (trips, mut pass, mut fail, mine) = stands_a_second(&t, Some(900 * UNIT));
    let d = decide_at(&t, &mut pass, &mut fail, t.closes_ms);
    let lost = t.budget - mine.quote;
    eprintln!("  this law: {trips} round trips of 900, {lost} lost, the house's fees {}, PASS's spot left at {} ({} over the opening), its average {:?}: {:?}", pass.fees, pass.spot(), pass.spot() - open, d.twap_pass, d.state);
    assert_eq!((pass.base, pass.quote + pass.fees + mine.quote), (t.pool_base, t.pool_quote + t.budget), "the claims where they were; the $WARM it lost is the house's fees, beside the pool");
    assert!(pass.quote - t.pool_quote <= trips as u128 * 2, "the pool's $WARM where it was but the rounding of its round trips: {}", pass.quote);
    assert_eq!(d.state, State::Failed, "and the road fails");
    // all in, every boundary, until its purse is spent
    let (trips, mut pass, mut fail, mine) = stands_a_second(&t, None);
    let d = decide_at(&t, &mut pass, &mut fail, t.closes_ms);
    eprintln!("  this law, all in: {trips} round trips, its purse left with {} micro, the house's fees {}, PASS's spot left at {} ({} over the opening): {:?}", mine.quote, pass.fees, pass.spot(), pass.spot() - open, d.state);
    assert!(pass.spot() - open <= open / 100_000, "within 10 ppm of where it opened, after {trips} round trips");
    assert_eq!(pass.quote + pass.fees + mine.quote, t.pool_quote + t.budget);
    assert_eq!(d.state, State::Failed);
}

#[test]
fn the_fee_kept_in_claims_would_move_the_price_too() {
    // the second witness: a buy of 900 as this law takes it, then the sale of its claims with the fee kept IN CLAIMS beside the pool
    let t = terms(100, false);
    let b = Branch::open(&t);
    let (base, quote, got, _) = traded(&t, &b, &t.purse_of(t.budget), &Trade { branch: Which::Pass, side: Side::Buy, amount: 900 * UNIT, min_out: 0 }).unwrap();
    let kept = fee_of(got, 100);
    let gross = swap_out(base, quote, Side::Sell, got - kept).unwrap();
    let (base, quote) = (base + got - kept, quote - gross);
    let s = spot(base, quote).unwrap();
    eprintln!("  the fee kept in claims: {kept} claims out of circulation, the pool at {quote} $WARM against {base} claims, the spot {s}, held by nobody");
    assert_eq!(kept, 4_711_793);
    assert!(s > PRICE * 10_094 / 10_000 && s < PRICE * 10_096 / 10_000, "about 1.0095: {s}");
}

#[cfg(feature = "fold")]
#[test]
fn the_settle_counts_the_fees_as_the_houses_own_number() {
    // a small road both ways, a judge's receipts: the same trades with no fee and with 100 bp
    let o = terms(0, true).opens_ms;
    let es = [entry(1, o + MIN, "a", Which::Pass, Side::Buy, 200 * UNIT), entry(2, o + 2 * MIN, "b", Which::Pass, Side::Sell, 100 * UNIT), entry(3, o + 3 * MIN, "c", Which::Fail, Side::Buy, 50 * UNIT),
        entry(4, o + 4 * MIN, "a", Which::Pass, Side::Sell, 60 * UNIT), entry(5, o + 5 * MIN, "c", Which::Pass, Side::Buy, 80 * UNIT)];
    let rc = Receipts { world: Which::Pass, worth: PRICE / 2 };
    let run = |fee_bp: u32| {
        let t = terms(fee_bp, true);
        let m = market_of(&t, &es, t.closes_ms);
        assert_eq!((m.accepted, m.refused.len()), (5, 0), "fee {fee_bp}: every trade taken");
        let d = decide(&t, &m, t.closes_ms);
        assert_eq!(world_of(&d), Some(Which::Pass), "fee {fee_bp}: PASS was bid higher");
        let results: i128 = settle_of(&t, &m, &d, &rc).unwrap().iter().map(|s| s.result).sum();
        let pool = pool_result(&t, &m, &d, &rc).unwrap();
        assert!((results + pool).abs() <= m.traders.len() as i128, "fee {fee_bp}: the traders' results {results} and the pool's {pool} sum to nothing, to the rounding, fees included");
        // the pool's result is exactly its fees more than the same branch with its counter at zero
        let mut bare = m.clone(); bare.pass.fees = 0;
        assert_eq!(pool, pool_result(&t, &bare, &d, &rc).unwrap() + m.pass.fees as i128, "fee {fee_bp}: the fees are the house's own number");
        (results, pool, m.pass.fees, m.fail.fees)
    };
    let ((r0, p0, _, _), (r1, p1, fees, void)) = (run(0), run(100));
    eprintln!("  no fee: the traders {r0}, the pool {p0}; 100 bp: the traders {r1}, the pool {p1}, of which {fees} the PASS world's fees (the reserves marked: {}); FAIL's {void} void", p1 - fees as i128);
    assert!(fees > fee_of(200 * UNIT, 100) + fee_of(80 * UNIT, 100), "the PASS world's two buys and two sells paid {fees}");
    assert_eq!(void, fee_of(50 * UNIT, 100), "FAIL's one buy paid its fee in a world that did not happen");
    assert!(p1 > p0 && r1 < r0);
}

#[cfg(feature = "fold")]
#[test]
fn the_fee_of_the_world_that_did_not_happen_is_void() {
    // PASS is bid far higher; FAIL's traders pay fees in a world that does not happen: the PASS world's settle and the house's take are the same
    // as on the road without them
    let t = terms(100, true);
    let o = t.opens_ms;
    let pass = [entry(1, o + MIN, "a", Which::Pass, Side::Buy, 400 * UNIT), entry(2, o + 2 * MIN, "b", Which::Pass, Side::Sell, 30 * UNIT)];
    let mut all = pass.to_vec();
    all.extend([entry(3, o + 3 * MIN, "a", Which::Fail, Side::Buy, 50 * UNIT), entry(4, o + 4 * MIN, "c", Which::Fail, Side::Sell, 70 * UNIT), entry(5, o + 5 * MIN, "b", Which::Fail, Side::Buy, 20 * UNIT)]);
    let rc = Receipts { world: Which::Pass, worth: PRICE / 3 };
    let (m, m2) = (market_of(&t, &all, t.closes_ms), market_of(&t, &pass, t.closes_ms));
    let (d, d2) = (decide(&t, &m, t.closes_ms), decide(&t, &m2, t.closes_ms));
    assert_eq!((world_of(&d), world_of(&d2)), (Some(Which::Pass), Some(Which::Pass)));
    assert!(m.fail.fees > 0, "FAIL's traders paid {} in fees", m.fail.fees);
    let results = |m: &Market, d: &Decision| settle_of(&t, m, d, &rc).unwrap().into_iter().filter(|s| s.key != "c").map(|s| (s.key, s.result)).collect::<Vec<_>>();
    assert_eq!(results(&m, &d), results(&m2, &d2), "FAIL's trades and their fees count for nothing in the PASS world");
    assert_eq!(pool_result(&t, &m, &d, &rc), pool_result(&t, &m2, &d2, &rc), "the house's take in the PASS world is the PASS world's fees alone: FAIL's {} are void", m.fail.fees);
}
