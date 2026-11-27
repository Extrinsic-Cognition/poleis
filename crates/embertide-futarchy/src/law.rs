//! THE MARKET'S ONE LAW, in integers (Nov 11 2026). Pure: no import, no clock but the stamps handed in, no float. The same answer on every reader.
//!
//! A road opens two conditional pools, PASS and FAIL: constant product, the society's token (base) priced in $WARM (quote), a fee of $WARM kept
//! beside it. Every trader holds its purse, opened with its deposit, in both and may buy or sell in either. Each pool keeps an OBSERVATION that
//! follows its spot price, and the average of that observation over the window decides: the road passes when PASS's average beats FAIL's by the
//! threshold at the close.
//!
//! THE WORLDS AND THE RECEIPTS. The two pools are two worlds: PASS is the world where the road is taken, FAIL the world where it is not. In each, the
//! base is a CLAIM on what the road's measure turns out to be in that world, worth between zero and one $WARM. Nothing inside the market says who was
//! right: `score_of` marks what a trader holds at the pool's own average, so whoever moves a price is marked at the price it made, and gains by it.
//! After the close a judge's signed word, the RECEIPTS, says what the claim was worth in the world that happened, and `settle_of` marks every purse
//! there, against what it opened with: that is the result that counts. The world that did not happen counts for nothing.
//!
//! BOTH WAYS. A purse that opens with $WARM alone can only buy first, so a lifted price meets no seller. On a road that can be bet both ways
//! (`both_ways`) every purse opens with its deposit in the pool's own mix, in both worlds: some of it as $WARM and the rest as claims, minted at one
//! $WARM a claim, in the proportion the pool holds them (`purse_of`). There is still no amount to choose, and the mix is the one that is even: from
//! the opening price a purse's whole claim lowers the price by exactly the factor its whole $WARM lifts it (proofs/both-ways.smt2: its claims are
//! to its $WARM as the pool's are), so a trader can bet as much against a road as for it. With a fee it is even to the fee (THE FEE).
//!
//! THE DEPOSIT AND THE ROSTER. A road may name who trades it and what each deposits (`Deposit`, `market_with`): a key on the roster opens its purse
//! with its own deposit, a key off it cannot trade; a road that names none opens every purse with `budget`, the law as it was. A purse cannot lose
//! more than its deposit (proofs/settle-deposit.smt2), so what a road gives back at the settle, the deposit and the result (`back_of`), is never
//! negative: nothing is cut off and nothing is forgiven. The law keeps no balance: it only says what comes back. The pool is nobody's deposit: it is
//! the house's, and what it loses, at most what it opened with, is all a road can add to what the traders hold between them.
//!
//! THE FEE (Nov 20 2026). A road names `fee_bp`, a share of every trade the house keeps; a road that names none keeps none, the law as it was.
//! THE FEE IS $WARM, AND IT NEVER TOUCHES THE POOL: it is taken from what a buy pays before that reaches the pool, and from what a sell receives
//! after that has left it, `fee_of(x)`, the ceiling of `x * fee_bp / 10000`, so no trade is too small to pay. Each branch keeps it in one
//! counter beside its reserves (`Branch::fees`). So no claim is ever in a fee, the reserves move only as a trader's position does, and the pool's
//! price moves only with a position some trader holds: a buy and the sale of what it bought leave the pool where it was, to the rounding, and cost
//! both fees on the trade's whole size, about `a * f * (2 - f)` for `a` in at a fee `f` (900 in and back at 100 bp: 17.91), however large.
//! At the settle the pool's result is its reserves marked at the receipts and its `fees`, against what it opened with: the house's take in the
//! world that happened is a number of its own. The fees of the world that did not happen are void with every trade of that world, by the
//! law's own word: a trader who pays them pays nobody. What it costs: the pool's own mix is even only to the fee. A purse's whole claim lowers
//! the price by `(1 + x)^2` and its whole $WARM lifts it by `(1 + phi x)^2`, `phi = 1 - fee_bp / 10000` (proofs/both-ways.smt2): with a fee a
//! seller of a purse's whole claim moves the price a little more than a buyer with its whole $WARM, by the fee and no more.
//! THE RULE IT REPLACES, kept as its witness (tests/fee.rs): a fee kept IN the reserve moved the price by itself, held by nobody. The trader of
//! THE PRICE THAT STOOD, at 100 bp, lasted 11 round trips of 900, paid 106.96, and PASS stood 10.7 % higher for the rest of the window: the road
//! passed. A fee kept in claims beside the pool does the same by the claims it takes out of circulation: one round trip of 900 leaves the spot
//! at 1.0095.
//!
//! THE GRID (what this law changes from futarchy-law.mjs, and why). The text moved the observation once per call: a trade at any millisecond let it
//! step max_step_bp toward the spot, so a trader who spiked the spot and then sent one dust trade a millisecond walked the average to the spike
//! (tests/law.rs the_spam_that_walked_the_old_average). Here the observation moves ONLY at the grid's boundaries, opens + n * step_ms, by at most
//! max_step_bp of itself. Entries between two boundaries cannot move it, whatever their number.
//!
//! THE PRICE THAT STOOD (Nov 20 2026). Under the grid as it first stood, a boundary stepped the observation toward the spot of that one instant, so
//! a trader who bought at every boundary and sold what it bought a second later, in the pool one second in sixty, walked the observation a full
//! step each time: on a road of three days at a step a minute, a purse of 1000 against pools of 1000 carried PASS's observation from 1.0 past 3.0
//! in 111 steps, on to the pushed spot of 3.61, passed the road and lost 5753 micro $WARM to the rounding, holding no claim the receipts could
//! charge (tests/law.rs the_trader_who_stood_a_second_at_every_boundary_walks_nothing). Now a price counts for the time it stood: between two
//! boundaries the branch sums the spot, held inside the lag band around the observation (`step_toward`), times the milliseconds it stood (`acc`),
//! and at a boundary the observation becomes that sum over the step. The same trader moves it 175 ppm and the road fails. Held, then averaged,
//! never averaged and then held: a raw spot averaged first would still let a spike large enough move the mean a full step however briefly it
//! stood (a spot of 100 for one second in sixty lifts a mean of 1 by 165 %); held first, a spike of any size counts as one band for the time it
//! stood, and the quiet price of the rest of the step pulls it back.
//! A trade counts from its own stamp: a boundary is due the instant the clock reaches it, so a trade stamped on a boundary counts for the step it
//! opens, never for the one that ended there.
//!
//! WHAT THE LAW CANNOT DO ALONE (Nov 20 2026; tests/stood.rs what_the_law_cannot_do_alone_and_what_another_trader_does). A road nobody else trades
//! is decided by whoever trades it, for nothing. It may lift a price, hold it through the window and sell it back before the close: no trade stood
//! between, so the pool gives its $WARM back, and it holds no claim for the receipts to mark. Or it may push the world that then does not happen,
//! which counts for nothing by the law's own word. No rule of an average can close that. What makes a wrong price cost is ANOTHER TRADER who takes
//! the other side while it stands, and the receipts on whatever is still held at the close in the world that happened. What this law owes the
//! others is the time to answer: a price must stand most of every step to move the observation at all, so whoever answers within the hour meets
//! it, where under the tick's rule only a trader as fast as the tick could. "Whoever moves a price pays" is true of a road with someone else in it.
//!
//! A QUIET STRETCH IS ONE STEP (Nov 20 2026; the owner's "Yes, build it" to a market nobody trades costing nothing to keep current). It was:
//! `Branch::advance` walked every boundary of the grid between where a branch stood and the instant asked, so a road nobody traded still cost
//! `window / step` boundaries to bring to its close, a bounded carry on a chain `most` of them a call, and a road laid late was born that far
//! behind. It is: in a whole step during which the pool does not move, the held sum ends as `step_toward(obs, spot) * step_ms` and the boundary
//! takes it over the step, which is `step_toward(obs, spot)` exactly; so once that is `obs` (the observation has reached the spot, or its band is
//! zero wide) every whole step after it leaves the observation where it is, and `k` such steps are one addition: `cum` gains `obs * k * step_ms`,
//! `steps` gains `k`, `acc` stays 0 (`Branch::walk`, proofs/quiet.smt2). What is still walked, one boundary at a time, is a step begun with a
//! partial `acc` and every step of an observation still CHASING a moved spot: each moves it by a floor of its own, and the floors are the
//! answer, so they are walked as they stood, one at a time; a bounded carry (`carry`, `most`) counts only those. A walked step that is whole from
//! its start holds one spot, so by the same fact its boundary takes that held spot itself, with no division. The answer moves by no bit: THE RULE IT
//! REPLACES, kept as its witness (tests/quiet.rs), is the loop as it stood, run beside this one over a seeded sweep and held equal field for
//! field after every carry.
//!
//! WHAT IS PROVEN (proofs/*.smt2, over the unbounded integers, checked by z3 in tests/proofs.rs) and what is checked on this code (tests/law.rs,
//! tests/stood.rs, tests/quiet.rs):
//!   a swap never drains a pool and never lowers its product (a buy's net, a sell's whole claim), and a fee is never more than what it is
//!   taken from; a round trip gives back at most the net of its first leg less the second fee; a spot held inside the band lies within
//!   max_step_bp of the observation and between it and the spot (proofs/step.smt2), and the observation a boundary takes, the floor of the
//!   held spots' sum over the step, lies inside that
//!   band too (proofs/stood.smt2), so it moves by at most max_step_bp a boundary and never leaves what stood; the average lies between the lowest
//!   and the highest observation; at the settle, what the traders win the pool loses, to the rounding, and a
//!   trader cannot lose more than it opened with, nor more than its deposit; the pool's own mix is the even one; a whole step whose held spot is
//!   the observation leaves it where it is and adds `obs * step_ms` to the sum, `k` of them `k * obs * step_ms` (and any whole step from its
//!   start takes its held spot as the observation), and the boundaries a carry
//!   applies from the start of a step are exactly the first `(last - at) div step_ms` (proofs/quiet.smt2); and on this code, over every
//!   trade of a small pool and a seeded sweep of large ones: both tokens are conserved to the micro, a refused entry changes nothing, and a
//!   reading in between changes no later answer.
//! THE RANGE: every amount is at most AMOUNT_MAX and every price inside PRICE_LO..PRICE_HI, so no product below leaves u128 (the consts at the foot
//! are checked by the compiler); a trade that would leave the range is refused, and every sum and product is a checked one.
//!
//! THE CORE AND THE FOLD (Nov 20 2026). The market's law is read two ways. The FOLD (fold.rs) reads a road's whole list of signed entries and
//! answers the market; it needs lists and text. A program on a chain sees ONE trade at a time, keeps the two pools and each purse in accounts, and
//! has no allocator and no std. Both run the same arithmetic, byte for byte, and not a copy of it: this file is the CORE, everything one trade, one
//! boundary of the grid and one purse's settle need, with no allocation and no std (it builds with no feature at all); the fold is built on it, and
//! when the fold is built this module names it too (the foot), so a reader of the law finds every piece here as before. A program does not put
//! the law together from its pieces: a trade is one call (`trade_on`, after `stamp_check`), a bounded carry is one (`carry`), the decision is one
//! (`decide_at`) and a position's settle is one (`settle_at`), so the order the fold keeps is not a caller's to keep.

/// one $WARM, or one base token, in the law's unit
pub const UNIT: u128 = 1_000_000;
/// a price is quote per base, times this
pub const PRICE: u128 = 1_000_000_000_000;
pub const BP: u128 = 10_000;
/// the most a pool or a purse may hold: a billion tokens
pub const AMOUNT_MAX: u128 = 1_000_000_000 * UNIT;
/// a spot must stay inside these, or the trade that would move it out is refused: 1e-6 to 1e9 quote per base
pub const PRICE_LO: u128 = PRICE / 1_000_000;
pub const PRICE_HI: u128 = PRICE * 1_000_000_000;
pub const WINDOW_MAX_MS: u64 = 366 * 86_400_000;
pub const STEP_MIN_MS: u64 = 1_000;

/// a road's market terms: its window on the Nest's clock, the grid, the lag limit, the threshold, the pool both branches open with, what a purse
/// deposits when the road names no roster, and whether it can be bet both ways
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Terms { pub opens_ms: u64, pub closes_ms: u64, pub step_ms: u64, pub max_step_bp: u32, pub threshold_bp: u32, pub pool_base: u128, pub pool_quote: u128,
    /// what every purse deposits when the road names no roster, in micro $WARM
    pub budget: u128,
    /// a road that can be bet both ways: every purse opens with its deposit in the pool's own mix in each branch, part $WARM and the rest claims
    /// (`purse_of`), so it can sell first. false is the law as it was: a purse of $WARM alone, that can only buy first.
    pub both_ways: bool,
    /// the share of every trade the house keeps, in basis points, under 10000: of what a buy pays, of what a sell receives (`fee_of`, THE FEE).
    /// 0 is the law as it was: no fee.
    pub fee_bp: u32 }

impl Terms {
    /// the terms the law can carry, or why not
    pub fn check(&self) -> Result<(), &'static str> {
        if self.closes_ms <= self.opens_ms || self.closes_ms - self.opens_ms > WINDOW_MAX_MS { return Err("the window is empty or longer than 366 days"); }
        if self.step_ms < STEP_MIN_MS { return Err("a step is shorter than a second"); }
        if self.max_step_bp == 0 || self.max_step_bp as u128 > BP { return Err("the lag limit is not 1 to 10000 bp"); }
        if self.threshold_bp as u128 > BP { return Err("the threshold is over 10000 bp"); }
        if self.fee_bp as u128 >= BP { return Err("the fee is 10000 bp or more"); }
        if self.pool_base == 0 || self.pool_base > AMOUNT_MAX || self.pool_quote == 0 || self.pool_quote > AMOUNT_MAX { return Err("the pool is empty or over a billion"); }
        if self.budget == 0 || self.budget > AMOUNT_MAX { return Err("the budget is empty or over a billion"); }
        if self.both_ways && { let p = self.purse_of(self.budget); p.quote == 0 || p.base == 0 } { return Err("a deposit both ways is too small to hold both $WARM and claims"); }
        match spot(self.pool_base, self.pool_quote) { Some(p) if (PRICE_LO..=PRICE_HI).contains(&p) => Ok(()), _ => Err("the opening price is outside the law's range") }
    }
    /// what a purse opens with, in each branch, for a deposit of `deposit` micro $WARM. On a road one way: the deposit, as $WARM. On a road both ways:
    /// the deposit in the pool's own mix, `deposit * pool_quote / (pool_quote + pool_base)` of $WARM, floored, and the rest as claims, a claim for a
    /// $WARM (minting a claim costs one $WARM: the claim and its opposite pay one between them). So quote + base is the deposit, to the micro.
    pub fn purse_of(&self, deposit: u128) -> Purse {
        if !self.both_ways { return Purse { quote: deposit, base: 0 }; }
        let quote = deposit.saturating_mul(self.pool_quote) / self.pool_quote.saturating_add(self.pool_base).max(1);
        Purse { quote, base: deposit.saturating_sub(quote) }
    }
}

/// the spot price of a pool, floored
pub fn spot(base: u128, quote: u128) -> Option<u128> { if base == 0 { None } else { quote.checked_mul(PRICE)?.checked_div(base) } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which { Pass, Fail }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side { Buy, Sell }

/// what a swap gives, floored: buy puts `amount` of quote in and takes base out; sell puts base in and takes quote out. None when a sum leaves u128.
/// With a fee a buy hands it the net of what it pays and a sell all of its claims (`traded`).
pub fn swap_out(base: u128, quote: u128, side: Side, amount: u128) -> Option<u128> {
    match side {
        Side::Buy => base.checked_mul(amount)?.checked_div(quote.checked_add(amount)?),
        Side::Sell => quote.checked_mul(amount)?.checked_div(base.checked_add(amount)?),
    }
}

/// the fee of `amount` of $WARM (what a buy pays, or what a sell receives): the ceiling of `amount * fee_bp / 10000`, so no trade is too small
/// to pay (with any fee, a buy of one micro pays it whole and buys nothing); at most the amount while `fee_bp` is under 10000; saturating
pub fn fee_of(amount: u128, fee_bp: u32) -> u128 { let x = amount.saturating_mul(fee_bp as u128); x / BP + (x % BP != 0) as u128 }

/// the spot held inside the lag band around the observation: clamped into `obs - d ..= obs + d`, `d` being `max_step_bp` of `obs`. It is what a
/// price counts for while it stands (`Branch::advance` sums it times the milliseconds it stood), so a boundary's observation, the average of it
/// over the step, moves by at most `max_step_bp` of itself and never past what stood. While one spot stands the whole step it is that boundary's
/// observation, the rule as the grid first stood.
pub fn step_toward(obs: u128, spot: u128, max_step_bp: u32) -> u128 {
    let d = obs.saturating_mul(max_step_bp as u128) / BP;
    spot.clamp(obs - d, obs.saturating_add(d))
}

/// one conditional pool and its average so far: `cum` is the sum of observation times milliseconds from the opening to `at`; `acc` the sum, over
/// the step so far, of the spot held inside the lag band times the milliseconds it stood; `steps` boundaries applied
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Branch { pub base: u128, pub quote: u128, pub obs: u128, pub cum: u128, pub acc: u128, pub at: u64, pub steps: u64,
    /// the fees this branch's trades paid, micro $WARM, kept beside the reserves and never in them (THE FEE)
    pub fees: u128 }

impl Branch {
    pub fn open(t: &Terms) -> Branch { Branch { base: t.pool_base, quote: t.pool_quote, obs: spot(t.pool_base, t.pool_quote).unwrap_or(0), cum: 0, acc: 0, at: t.opens_ms, steps: 0, fees: 0 } }
    pub fn spot(&self) -> u128 { spot(self.base, self.quote).unwrap_or(0) }
    /// carry the average to `to` (held inside the window), on the spot that stood since `at`: nothing in here moves the pool, so it is read once.
    /// Each stretch adds the observation to `cum` and the spot held inside the band (`step_toward`) to `acc`, each times its milliseconds; at each
    /// boundary up to and including `to` the observation becomes `acc` over the step and `acc` starts again. A boundary is due the instant the
    /// clock reaches it: the step that ends there was whole, and a trade stamped on it, taken after this carry, counts for the step it opens.
    /// No boundary at or past the close is applied: at the close `acc` holds the last step, which the decision does not read.
    /// A stretch at rest (A QUIET STRETCH IS ONE STEP) is carried in one step: `walk` with no bound.
    pub fn advance(&mut self, to: u64, t: &Terms) { self.walk(to, u64::MAX, t); }
    /// carry the average toward `to` as `advance` does, walking at most `most` boundaries one at a time (a 0 is read as 1, so a carry that has a
    /// boundary to walk always walks one); true when the branch stands at `to` (clamped into the window), false when it stopped short.
    /// A QUIET STRETCH IS ONE STEP: at the start of a step (`acc` is 0 and `at` is the last boundary) whose held spot is the observation itself
    /// (`step_toward(obs, spot) == obs`: the observation has reached the spot, or its band is zero wide), every whole step after it leaves the
    /// observation where it is, so the boundaries the loop would apply one by one, those at or before `to` and before the close, the first
    /// `(last - at) / step_ms` of them, `last` the smaller of `to` and the close less one (proofs/quiet.smt2), are applied at once: `cum` gains
    /// `obs` times their milliseconds, `steps` their number. They are never counted against `most`. Every other boundary (a step begun with a
    /// partial `acc`, an observation still chasing a moved spot) is walked, and counted; a walked step whole from its start takes its held spot
    /// as the observation, which is the boundary's division done exactly (proofs/quiet.smt2). It stops only where a boundary was just applied or at
    /// `to`, so every state it stops in is one `advance(x)` reaches for some instant `x`: a reading in between changes no later answer.
    pub fn walk(&mut self, to: u64, most: u64, t: &Terms) -> bool {
        let end = to.clamp(t.opens_ms, t.closes_ms);
        // the next boundary, `opens + (steps + 1) * step`, moved by a step as each is applied (once it saturates it stays at or past the close)
        let mut next = t.opens_ms.saturating_add((self.steps + 1).saturating_mul(t.step_ms));
        // nothing to carry: no boundary due and no time to add, so the pool is not even read
        if (next >= t.closes_ms || next > end) && end.min(next) <= self.at { return true; }
        let spot = self.spot();
        let mut left = most.max(1);
        // A WHOLE STEP from its start (nothing held, the branch on its boundary) holds one spot: `acc` ends as `held * step_ms` and the boundary
        // takes it over the step, which is `held` exactly, so the step is the observation's sum and `obs = held`, no division. `obs` inside the
        // law's range and the step inside its longest window keep `held * step_ms` and `obs * step_ms` inside u128 (the foot), so nothing here
        // saturates; outside them the step is walked by the loop as it stood. A step at rest is one whose held spot is `obs` itself (the spot is
        // the observation, or the band is zero wide): it leaves `obs` as it is, and every whole step after it is one addition.
        let in_window = t.step_ms <= WINDOW_MAX_MS;
        let quiet = |obs: u128| obs == spot || (obs < BP && obs * (t.max_step_bp as u128) < BP);
        let mut whole = in_window && self.acc == 0 && self.at.saturating_add(t.step_ms) == next && self.obs <= PRICE_HI;
        loop {
            if next < t.closes_ms && next <= end {
                if whole && quiet(self.obs) {
                    let k = (end.min(t.closes_ms - 1) - self.at) / t.step_ms.max(1);
                    self.cum = self.cum.saturating_add(self.obs.saturating_mul((k * t.step_ms) as u128));
                    self.at += k * t.step_ms;
                    self.steps += k;
                    next = next.saturating_add(k * t.step_ms);
                    continue;
                }
                if left == 0 { return false; }
                left -= 1;
                if whole {
                    let held = step_toward(self.obs, spot, t.max_step_bp);
                    self.cum = self.cum.saturating_add(self.obs.wrapping_mul(t.step_ms as u128));
                    self.obs = held;
                    self.at = next;
                    self.steps += 1;
                    next = next.saturating_add(t.step_ms);
                    whole = held <= PRICE_HI;
                    continue;
                }
            }
            let upto = end.min(next);
            if upto > self.at {
                let dt = (upto - self.at) as u128;
                self.cum = self.cum.saturating_add(self.obs.saturating_mul(dt));
                self.acc = self.acc.saturating_add(step_toward(self.obs, spot, t.max_step_bp).saturating_mul(dt));
                self.at = upto;
            }
            if next >= t.closes_ms || next > end { return true; }
            self.obs = self.acc / (t.step_ms as u128).max(1);
            self.acc = 0;
            self.steps += 1;
            next = next.saturating_add(t.step_ms);
            whole = in_window && self.at.saturating_add(t.step_ms) == next && self.obs <= PRICE_HI;
        }
    }
    /// the average observation from the opening to where the branch was carried, None before any time passed
    pub fn twap(&self, t: &Terms) -> Option<u128> { let span = (self.at - t.opens_ms) as u128; if span == 0 { None } else { Some(self.cum / span) } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trade { pub branch: Which, pub side: Side, pub amount: u128, pub min_out: u128 }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Purse { pub quote: u128, pub base: u128 }

pub const NOT_ENOUGH: &str = "not enough in that branch";
pub const NOTHING_OUT: &str = "too small to take anything out";
pub const PAST_MIN: &str = "the price moved past min_out";
pub const OUT_OF_RANGE: &str = "it would leave the law's range";
/// the two refusals of time: they are the core's, because a fold reading a list and a program reading its clock must refuse the same stamps
pub const OUTSIDE: &str = "outside the window";
pub const BACKWARDS: &str = "stamped before an earlier entry";

/// a deposit a road can carry, or why not: 1 to AMOUNT_MAX, and on a road both ways large enough to hold both $WARM and claims (the two checks the
/// roster makes of each of its lines, and a program on a chain makes of a deposit as it comes in)
pub fn deposit_check(t: &Terms, amount: u128) -> Result<(), &'static str> {
    if amount == 0 || amount > AMOUNT_MAX { return Err("a deposit is empty or over a billion"); }
    if t.both_ways && { let p = t.purse_of(amount); p.quote == 0 || p.base == 0 } { return Err("a deposit both ways is too small to hold both $WARM and claims"); }
    Ok(())
}

/// the pool after a trade, what the purse gets and the fee; or why the trade is refused. Nothing is changed here. THE FEE is $WARM and never
/// touches the pool: a buy of `amount` pays `fee_of(amount)` and the pool trades the net; a sell of `amount` claims trades all of them, and the
/// fee is taken from the $WARM that left the pool (`gross`), so the purse gets `gross - fee`. What the purse gets is what must not be 0
/// (NOTHING_OUT) and what `min_out` is asked of. The spot and the range are read on the reserves alone.
pub fn traded(t: &Terms, b: &Branch, mine: &Purse, tr: &Trade) -> Result<(u128, u128, u128, u128), &'static str> {
    if tr.amount == 0 || tr.amount > AMOUNT_MAX { return Err(OUT_OF_RANGE); }
    if (match tr.side { Side::Buy => mine.quote, Side::Sell => mine.base }) < tr.amount { return Err(NOT_ENOUGH); }
    let (base, quote, got, fee) = match tr.side {
        Side::Buy => {
            let fee = fee_of(tr.amount, t.fee_bp);
            let net = tr.amount.saturating_sub(fee);
            let out = swap_out(b.base, b.quote, Side::Buy, net).ok_or(OUT_OF_RANGE)?;
            (b.base - out, b.quote.checked_add(net).ok_or(OUT_OF_RANGE)?, out, fee)
        }
        Side::Sell => {
            let gross = swap_out(b.base, b.quote, Side::Sell, tr.amount).ok_or(OUT_OF_RANGE)?;
            let fee = fee_of(gross, t.fee_bp);
            (b.base.checked_add(tr.amount).ok_or(OUT_OF_RANGE)?, b.quote - gross, gross.saturating_sub(fee), fee)
        }
    };
    if got == 0 { return Err(NOTHING_OUT); }
    if got < tr.min_out { return Err(PAST_MIN); }
    if base == 0 || quote == 0 || base > AMOUNT_MAX || quote > AMOUNT_MAX { return Err(OUT_OF_RANGE); }
    match spot(base, quote) { Some(p) if (PRICE_LO..=PRICE_HI).contains(&p) => Ok((base, quote, got, fee)), _ => Err(OUT_OF_RANGE) }
}

/// a trade taken: the pool, its fees and the purse after it, from what `traded` answered (`after`: the pool's base, its quote, what the purse
/// gets and the fee) for this pool, this purse and this trade. Only `trade_on` calls it. After `traded` the purse holds at least
/// the amount, so each subtraction is the plain one; it saturates so that a caller that hands a wrong `after` cannot wrap a purse.
fn take(b: &mut Branch, mine: &mut Purse, tr: &Trade, after: (u128, u128, u128, u128)) {
    let (base, quote, got, fee) = after;
    b.base = base; b.quote = quote; b.fees = b.fees.saturating_add(fee);
    // a purse's gain saturates: in the law's range it never comes near u128, and a roster the law cannot carry panics nothing
    match tr.side { Side::Buy => { mine.quote = mine.quote.saturating_sub(tr.amount); mine.base = mine.base.saturating_add(got); } Side::Sell => { mine.base = mine.base.saturating_sub(tr.amount); mine.quote = mine.quote.saturating_add(got); } }
}

/// THE RULES OF TIME for a trade stamped `ts`, read at `now`, when the last trade taken was stamped `last`: inside the window and not after the
/// reading, else OUTSIDE; not before the last trade taken, else BACKWARDS. For a program on a chain `ts` is its clock and so is `now`; it keeps
/// `last` in the road's own account.
pub fn stamp_check(t: &Terms, ts: u64, last: u64, now: u64) -> Result<(), &'static str> {
    if ts < t.opens_ms || ts >= t.closes_ms || ts > now { return Err(OUTSIDE); }
    if ts < last { return Err(BACKWARDS); }
    Ok(())
}

/// ONE TRADE OF A ROAD, WHOLE, stamped `ts`: the law's refusals of it (`traded`), then BOTH branches carried to its stamp (a boundary there
/// included: the price that stood until `ts` is counted before this one moves it), then the pool and the purse moved. `mine` is the trader's
/// purse in the branch the trade names. What came out, or why it is refused; a refusal changes nothing, the carry included. The fold takes every
/// trade by this one call and so does a program on a chain, so neither can carry one branch and not the other, or take a trade it did not first
/// ask the law about. The caller has asked `stamp_check`, and after an `Ok` its `last` is `ts`.
pub fn trade_on(t: &Terms, pass: &mut Branch, fail: &mut Branch, mine: &mut Purse, tr: &Trade, ts: u64) -> Result<u128, &'static str> {
    let after = traded(t, match tr.branch { Which::Pass => &*pass, Which::Fail => &*fail }, mine, tr)?;
    pass.advance(ts, t); fail.advance(ts, t);
    take(match tr.branch { Which::Pass => pass, Which::Fail => fail }, mine, tr, after);
    Ok(after.2)
}

/// ONE BOUNDED CARRY OF A ROAD toward `now`: each branch walking at most `most` boundaries one at a time (`Branch::walk`; a 0 is read as 1), a
/// stretch at rest before, between or after them free (A QUIET STRETCH IS ONE STEP). True when both stand at `now`. A road nobody touched,
/// however long, is carried to any instant by one call; a branch whose observation is still chasing a moved spot is walked `most` boundaries a
/// call until it has caught up, and the rest in that same call. A program on a chain carries a road by this call alone, as often as it takes,
/// before it takes a trade or decides; however the calls fall, they are the one carry (tests/core.rs, tests/quiet.rs).
pub fn carry(t: &Terms, pass: &mut Branch, fail: &mut Branch, now: u64, most: u64) -> bool {
    let mut both = true;
    for b in [pass, fail] { both &= b.walk(now, most, t); }
    both
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State { Open, Passed, Failed }
/// the decision so far: the two averages and spots, and what PASS's average must exceed (FAIL's plus the threshold)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decision { pub state: State, pub twap_pass: Option<u128>, pub twap_fail: Option<u128>, pub spot_pass: u128, pub spot_fail: u128, pub needs: Option<u128> }

/// THE DECISION, of the two branches as they stand at `now`: at the close the road passes when PASS's sum beats FAIL's by the threshold (the two
/// sums share one span, so no division decides). The fold's `decide` reads its market's two branches here; a program reads its two accounts.
pub(crate) fn decide_of(t: &Terms, pass: &Branch, fail: &Branch, now: u64) -> Decision {
    let (twap_pass, twap_fail) = (pass.twap(t), fail.twap(t));
    let needs = twap_fail.map(|f| f.saturating_mul(BP + t.threshold_bp as u128) / BP);
    let passed = pass.at == fail.at && pass.cum.saturating_mul(BP) > fail.cum.saturating_mul(BP + t.threshold_bp as u128);
    let state = if now < t.closes_ms { State::Open } else if passed { State::Passed } else { State::Failed };
    Decision { state, twap_pass, twap_fail, spot_pass: pass.spot(), spot_fail: fail.spot(), needs }
}

/// THE DECISION AT `now`, WHOLE: both branches carried to `now`, then the decision of the two. The fold ends by this same carry before it
/// decides. A program on a chain decides by this call alone, so it cannot read a road that should pass as failed because one branch was carried
/// to the close and the other was not.
pub fn decide_at(t: &Terms, pass: &mut Branch, fail: &mut Branch, now: u64) -> Decision {
    pass.advance(now, t); fail.advance(now, t);
    decide_of(t, pass, fail, now)
}

/// what the judge read off the receipts: which world happened and what a claim was worth in it, in $WARM times PRICE (0 to PRICE: a claim pays at most one $WARM)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipts { pub world: Which, pub worth: u128 }
pub const STILL_OPEN: &str = "the road has not closed";
pub const OTHER_WORLD: &str = "the receipts are of the world that did not happen";
pub const WORTH_RANGE: &str = "a claim is worth zero to one $WARM";

/// what `holding` of the claim is worth at the receipts, floored, in micro $WARM: holding * worth / PRICE, exact for every holding while the worth is
/// at most PRICE (the holding's whole multiples of PRICE and its remainder are marked apart, so no product leaves u128 however much a purse holds)
pub fn worth_of(holding: u128, worth: u128) -> u128 { (holding / PRICE).saturating_mul(worth).saturating_add((holding % PRICE).saturating_mul(worth) / PRICE) }

/// the world that happened: None while the road is open
pub fn world_of(d: &Decision) -> Option<Which> { match d.state { State::Open => None, State::Passed => Some(Which::Pass), State::Failed => Some(Which::Fail) } }

/// the world that happened, if the receipts can settle it: the worth in range first, then the road closed, then the receipts of that world
pub fn happened(d: &Decision, r: &Receipts) -> Result<Which, &'static str> {
    if r.worth > PRICE { return Err(WORTH_RANGE); }
    let world = world_of(d).ok_or(STILL_OPEN)?;
    if r.world != world { return Err(OTHER_WORLD); }
    Ok(world)
}

/// ONE PURSE'S SETTLE, in micro $WARM: `held`, its quote and its claim marked at `worth`, against the $WARM and the claim its own `deposit` opened
/// its purse with (`purse_of`), marked the same, so a purse that holds what it opened with scores exactly 0. The caller has asked `happened` first
/// (the worth in range, the road closed, the receipts of the world that happened) and hands the purse of that world: the world that did not happen
/// counts for nothing.
pub(crate) fn result_of(t: &Terms, deposit: u128, held: &Purse, worth: u128) -> i128 {
    let o = t.purse_of(deposit);
    let opened = signed(o.quote.saturating_add(worth_of(o.base, worth)));
    signed(held.quote.saturating_add(worth_of(held.base, worth))) - opened
}

/// what a road gives back to a balance at the settle: the deposit and the result. Never under zero, because a purse cannot lose more than its
/// deposit (proofs/settle-deposit.smt2): nothing is cut off here and nothing is forgiven.
pub fn back_of(deposit: u128, result: i128) -> u128 { signed(deposit).saturating_add(result).max(0) as u128 }

/// an amount as a signed one: past what i128 holds it stops there and never wraps (no amount of the law's range comes near)
pub(crate) fn signed(x: u128) -> i128 { i128::try_from(x).unwrap_or(i128::MAX) }

/// what the pool itself gained in the world that happened, by the same mark: `b` is the branch of that world (the caller has asked `happened`),
/// its quote and its claim marked at `worth`, and its `fees` (an amount of $WARM, not marked), against what the pool opened with, marked the same
pub(crate) fn pool_result_of(t: &Terms, b: &Branch, worth: u128) -> i128 {
    signed(b.quote.saturating_add(worth_of(b.base, worth)).saturating_add(b.fees)) - signed(t.pool_quote.saturating_add(worth_of(t.pool_base, worth)))
}

/// ONE POSITION'S SETTLE, WHOLE: the receipts asked of the decision (`happened`), then the purse of the world that happened marked against its own
/// deposit. A program on a chain settles a position by this call alone, so it cannot settle on receipts the law refuses or on the purse of the
/// world that did not happen. What a road then gives back is `back_of`.
pub fn settle_at(t: &Terms, d: &Decision, r: &Receipts, deposit: u128, pass: &Purse, fail: &Purse) -> Result<i128, &'static str> {
    let held = match happened(d, r)? { Which::Pass => pass, Which::Fail => fail };
    Ok(result_of(t, deposit, held, r.worth))
}

/// THE POOL'S SETTLE, WHOLE: the receipts asked of the decision, then the branch of the world that happened by the same mark: what the traders won
/// between them, the pool lost, to the rounding.
pub fn pool_at(t: &Terms, d: &Decision, r: &Receipts, pass: &Branch, fail: &Branch) -> Result<i128, &'static str> {
    let b = match happened(d, r)? { Which::Pass => pass, Which::Fail => fail };
    Ok(pool_result_of(t, b, r.worth))
}

// THE RANGE, checked by the compiler: with every amount at most AMOUNT_MAX, every price at most PRICE_HI and a window at most WINDOW_MAX_MS, none of
// the law's products leaves u128.
const _: () = assert!(AMOUNT_MAX.checked_mul(AMOUNT_MAX).is_some());                       // a swap's numerator
const _: () = assert!(AMOUNT_MAX.checked_mul(PRICE).is_some());                            // a spot's numerator
const _: () = assert!(PRICE_HI.checked_mul(BP).is_some());                                 // a step
const _: () = assert!(PRICE_HI.checked_mul(WINDOW_MAX_MS as u128).is_some());              // an average's sum, and a step's held sum (`acc`)
const _: () = assert!((PRICE_HI * WINDOW_MAX_MS as u128).checked_mul(2 * BP).is_some());   // the decision's two sides
const _: () = assert!(AMOUNT_MAX.checked_mul(PRICE_HI).is_some());                         // a score's mark
const _: () = assert!(PRICE.checked_mul(PRICE).is_some());                                 // a holding's remainder marked at the most a claim is worth (worth_of)
const _: () = assert!(PRICE_LO * 1 / BP >= 1);                                             // the smallest step still moves: PRICE_LO * 1 bp is at least 1

// the fold, when it is built, is named here too: `law::*` gives every piece of the law it gave before the split
#[cfg(feature = "fold")]
pub use crate::fold::*;
