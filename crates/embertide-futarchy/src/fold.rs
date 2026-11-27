//! THE FOLD (Nov 20 2026, split from law.rs): the market of a road from its whole list of entries, built on the core. A trade taken, a boundary of
//! the grid, the decision and one purse's settle are the core's own functions (law.rs), called here as a program on a chain calls them one trade at
//! a time: the same arithmetic, byte for byte, not a copy of it. What lives here is what walks a list: the entries in the Nest's order, the refusals
//! of an entry, the roster, the traders, the scores. It allocates (lists and text) and needs no std.
use alloc::{string::String, vec::Vec};
use crate::law::{decide_of, deposit_check, happened, pool_result_of, result_of, signed, stamp_check, trade_on, Branch, Decision, Purse, Receipts, State, Terms, Trade, Which, PRICE};

/// one entry of the Nest's log as the fold reads it: its seq and the Nest's own stamp, the signer's key and seat, whether its signature verified, and
/// its trade (None: not a trade of this road)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry { pub seq: u64, pub ts: u64, pub key: String, pub seat: String, pub ok: bool, pub trade: Option<Trade> }

/// a trader of a road: its key and seat, its purse in each branch, its count of trades, and what its purse opened with (the same in each branch)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holder { pub key: String, pub seat: String, pub pass: Purse, pub fail: Purse, pub trades: u64, pub deposit: u128 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal { pub seq: u64, pub seat: String, pub why: &'static str }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Market { pub pass: Branch, pub fail: Branch, pub traders: Vec<Holder>, pub refused: Vec<Refusal>, pub accepted: u64, pub at: u64 }

/// the refusals of an entry that are the fold's alone: what a chain checks by other means (an instruction is a trade or it is not, its signer
/// signed or the transaction fails, a purse is an account or there is none). The refusals of time and of a trade are the core's (law.rs OUTSIDE and
/// BACKWARDS; NOT_ENOUGH, NOTHING_OUT, PAST_MIN and OUT_OF_RANGE).
pub const NOT_A_TRADE: &str = "not a trade of this road";
pub const UNSIGNED: &str = "unsigned or its signature does not verify";
pub const NOT_ON_THE_ROSTER: &str = "not on the road's roster";

/// one line of a road's roster: a trader's key and what it deposits in this road, in micro $WARM. A roster is its lines in the order of their keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deposit { pub key: String, pub amount: u128 }

/// the roster a road can carry, or why not: no key empty, the keys in their order with none named twice (so one pass reads it and the fold finds a
/// key by halving, whatever the number of traders), every deposit 1 to AMOUNT_MAX, and on a road both ways every deposit large enough to hold both
/// $WARM and claims (the last two are the core's `deposit_check`). An empty roster is always carried.
pub fn roster_check(t: &Terms, roster: &[Deposit]) -> Result<(), &'static str> {
    for (i, d) in roster.iter().enumerate() {
        if d.key.is_empty() { return Err("a key of the roster is empty"); }
        if i > 0 && roster[i - 1].key >= d.key { return Err("the roster's keys are not in their order, or one is named twice"); }
        deposit_check(t, d.amount)?;
    }
    Ok(())
}

/// THE FOLD: the market of a road from its entries in the Nest's order, read at `now`, every signer depositing `budget`. An entry that is refused
/// changes nothing.
pub fn market_of(t: &Terms, entries: &[Entry], now: u64) -> Market { market_with(t, &[], entries, now) }

/// THE FOLD, with a roster. A road that names one lets only the keys on it trade, each purse opening with its own deposit; with an empty roster
/// every signer deposits `budget`, the law as it was. The roster is not checked here (`roster_check` is the caller's): a key is looked for by halving,
/// so on a roster out of its order a key may not be found and is then refused; a bad roster panics nothing.
pub fn market_with(t: &Terms, roster: &[Deposit], entries: &[Entry], now: u64) -> Market {
    let (mut pass, mut fail) = (Branch::open(t), Branch::open(t));
    let (mut traders, mut refused, mut accepted, mut last) = (Vec::<Holder>::new(), Vec::new(), 0u64, t.opens_ms);
    let mut order: Vec<&Entry> = entries.iter().collect();
    order.sort_by_key(|e| e.seq);
    for e in order {
        let mut no = |why| refused.push(Refusal { seq: e.seq, seat: e.seat.clone(), why });
        let Some(tr) = e.trade else { no(NOT_A_TRADE); continue };
        if !e.ok || e.key.is_empty() { no(UNSIGNED); continue; }
        if let Err(why) = stamp_check(t, e.ts, last, now) { no(why); continue; }
        let at = match traders.iter().position(|h| h.key == e.key) {
            Some(i) => i,
            None => {
                let deposit = if roster.is_empty() { t.budget } else { match roster.binary_search_by(|d| d.key.as_str().cmp(e.key.as_str())) { Ok(i) => roster[i].amount, Err(_) => { no(NOT_ON_THE_ROSTER); continue; } } };
                let opens = t.purse_of(deposit);
                traders.push(Holder { key: e.key.clone(), seat: e.seat.clone(), pass: opens, fail: opens, trades: 0, deposit });
                traders.len() - 1
            }
        };
        let mine = match tr.branch { Which::Pass => &mut traders[at].pass, Which::Fail => &mut traders[at].fail };
        match trade_on(t, &mut pass, &mut fail, mine, &tr, e.ts) {
            Err(why) => { if traders[at].trades == 0 { traders.pop(); } no(why); }
            Ok(_) => { last = e.ts; traders[at].trades += 1; accepted += 1; }
        }
    }
    pass.advance(now, t); fail.advance(now, t);
    Market { pass, fail, traders, refused, accepted, at: now.clamp(t.opens_ms, t.closes_ms) }
}

/// THE DECISION of a market: the core's, on its two branches (law.rs decide_of).
pub fn decide(t: &Terms, m: &Market, now: u64) -> Decision { decide_of(t, &m.pass, &m.fail, now) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Score { pub key: String, pub seat: String, pub trades: u64, pub result: i128 }

/// each trader's paper result in micro $WARM: only the branch that happened is real, what it holds there marked at that branch's own average,
/// against what its own deposit opened its purse with, marked the same. This is the mark at the pool's own average, all a road with no judge has:
/// whoever moves a price is marked at the price it made. On a road both ways the mark is at most one $WARM, whatever the pool's average says: the
/// base there is a claim, minted at one $WARM and never worth more, so this mark too cannot show a loss over a deposit. `settle_of` is the result
/// that counts. Best first; a tie keeps the order of first trades. Empty while the road is open.
pub fn score_of(t: &Terms, m: &Market, d: &Decision) -> Vec<Score> {
    let (won, mark) = match d.state { State::Open => return Vec::new(), State::Passed => (Which::Pass, d.twap_pass.unwrap_or(d.spot_pass)), State::Failed => (Which::Fail, d.twap_fail.unwrap_or(d.spot_fail)) };
    let mark = if t.both_ways { mark.min(PRICE) } else { mark };
    let mut out: Vec<Score> = m.traders.iter().map(|h| {
        let o = t.purse_of(h.deposit);
        let opened = signed(o.quote.saturating_add(o.base.saturating_mul(mark) / PRICE));
        let p = match won { Which::Pass => h.pass, Which::Fail => h.fail };
        let worth = p.quote.saturating_add(p.base.saturating_mul(mark) / PRICE);
        Score { key: h.key.clone(), seat: h.seat.clone(), trades: h.trades, result: signed(worth) - opened }
    }).collect();
    out.sort_by(|a, b| b.result.cmp(&a.result));
    out
}

/// THE SETTLE: each trader's result in micro $WARM in the world that happened: its quote and its claim marked at the receipts, against the $WARM and
/// the claim its own deposit opened its purse with, marked the same (law.rs result_of), so a trader that holds what it opened with scores exactly 0.
/// The world that did not happen counts for nothing. Best first; a tie keeps the order of first trades.
pub fn settle_of(t: &Terms, m: &Market, d: &Decision, r: &Receipts) -> Result<Vec<Score>, &'static str> {
    let world = happened(d, r)?;
    let mut out: Vec<Score> = m.traders.iter().map(|h| {
        let p = match world { Which::Pass => h.pass, Which::Fail => h.fail };
        Score { key: h.key.clone(), seat: h.seat.clone(), trades: h.trades, result: result_of(t, h.deposit, &p, r.worth) }
    }).collect();
    out.sort_by(|a, b| b.result.cmp(&a.result));
    Ok(out)
}

/// what the pool itself gained in the world that happened, by the same mark: what the traders won between them, the pool lost, to the rounding
/// (law.rs pool_result_of, on the branch of that world)
pub fn pool_result(t: &Terms, m: &Market, d: &Decision, r: &Receipts) -> Result<i128, &'static str> {
    let b = match happened(d, r)? { Which::Pass => &m.pass, Which::Fail => &m.fail };
    Ok(pool_result_of(t, b, r.worth))
}
