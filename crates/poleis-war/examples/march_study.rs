//! Paired order-march experiments on the first reference war. See studies/march/RECEIPT.md.

use poleis_war::law::{
    land_on, mcap, Curve, Landing, Order, OrderKind, Phase, Rules, Side, BPS, MARCH_CAPACITY, SOL,
};
use poleis_war::war::{base, Done, Kind, War, HOUSE, N};
use std::collections::VecDeque;
use std::io::{self, Write};

const DELAYS: [u32; 5] = [0, 1, 4, 16, 64];
const SEED: u64 = 0x6a09_e667_f3bc_c909;
const EARLY_STEADY_SLOTS: [i64; 2] = [0, 4];
const TARGET_SLOT: i64 = 8;
const SLOT_MS: u64 = 50;

#[derive(Clone, Copy)]
struct Trial {
    curves: [Curve; 2],
    whale_spend: u128,
}

fn reference_trials() -> (Vec<Trial>, Done) {
    let mut war = War::new(Rules::MEME_ROYALE);
    for i in 0..N {
        for side in [Side::Attacker, Side::Defender] {
            war.enter(&format!("{}{i}", HOUSE[side.index()]), side, SOL)
                .unwrap();
        }
    }
    for i in (10..N).rev() {
        if war.stake(&format!("{}{i}", HOUSE[0])).unwrap() {
            break;
        }
    }
    let opening = war.opening.unwrap();
    let mut events = base(opening.minutes);
    events.sort();
    let mut events = events.into_iter().peekable();
    let mut trials = Vec::new();
    for minute in 0..=opening.minutes + war.rules.stand_minutes {
        while events.peek().is_some_and(|event| event.minute == minute) {
            let event = events.next().unwrap();
            if war.is_staker(&event.who) {
                continue;
            }
            match event.kind {
                Kind::Cross => war.cross(&event.who, event.house.other()).unwrap(),
                Kind::Desert => war.desert(&event.who).unwrap(),
                Kind::Reinforce => unreachable!("the first reference has no reinforcements"),
            };
        }
        let phase = war.minute(minute).unwrap();
        if minute < 100 && minute % 5 == 0 {
            assert_eq!(phase, Phase::Open);
            trials.push(Trial {
                curves: war.curves,
                whale_spend: whale_spend(trials.len() as u64),
            });
        }
        if matches!(phase, Phase::Decided { .. }) {
            break;
        }
    }
    (trials, war.settle().unwrap())
}

// SplitMix64 on a fixed seed and trial index, solely to vary the test inputs.
fn whale_spend(index: u64) -> u128 {
    let mut value = SEED.wrapping_add((index + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (4 + u128::from((value ^ (value >> 31)) % 5)) * SOL
}

// Call A is open. Use the attacker's share of the two market caps as the study's front.
fn front(curves: &[Curve; 2]) -> u128 {
    let attacker = mcap(&curves[0]).unwrap();
    let defender = mcap(&curves[1]).unwrap();
    BPS * attacker / (attacker + defender)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Trader {
    Steady,
    Whale,
    Observer,
}

#[derive(Clone, Copy)]
enum Placement {
    Honest,
    // The intake lets the observer insert ahead of orders in the proposed slot batch.
    BeforeAcceptance,
    // A forced round trip after the observer sees the whale already accepted.
    AfterAcceptance,
}

#[derive(Default)]
struct Fills {
    steady_coins: u128,
    observer_coins: u128,
    observer_pnl: i128,
    max_jump_bps: u128,
    min_wait: Option<i64>,
    max_wait: i64,
    count: usize,
    peak_pending: usize,
}

struct Market {
    rules: Rules,
    curves: [Curve; 2],
    queue: VecDeque<(Trader, Order)>,
    last_accepted: Option<i64>,
    fills: Fills,
}

impl Market {
    fn new(curves: [Curve; 2], delay_slots: u32) -> Self {
        // The order law adds and compares integers. This harness gives it slots
        // while the separate reference replay keeps the raid clock in minutes.
        let rules = Rules {
            march_s: delay_slots,
            ..Rules::MEME_ROYALE
        };
        Self {
            rules,
            curves,
            queue: VecDeque::new(),
            last_accepted: None,
            fills: Fills::default(),
        }
    }

    fn submit(&mut self, trader: Trader, kind: OrderKind, amount: u128, slot: i64) {
        assert!(self.queue.len() < MARCH_CAPACITY);
        let order = Order::accept(&self.rules, kind, amount, 0, slot, self.last_accepted).unwrap();
        self.queue.push_back((trader, order));
        self.last_accepted = Some(slot);
        if self.rules.march_s == 0 {
            self.land_through(slot);
        }
        self.fills.peak_pending = self.fills.peak_pending.max(self.queue.len());
    }

    fn land_through(&mut self, slot: i64) {
        while let Some(&(trader, order)) = self.queue.front() {
            let due = order.lands_at;
            if due > slot {
                break;
            }
            self.queue.pop_front();
            let before = front(&self.curves);
            let (curve, fill) =
                land_on(&self.rules, &self.curves[0], &order, due, false, 0).unwrap();
            self.curves[0] = curve;
            match (trader, fill) {
                (Trader::Steady, Landing::Buy(buy)) => self.fills.steady_coins += buy.coins,
                (Trader::Observer, Landing::Buy(buy)) => {
                    self.fills.observer_coins += buy.coins;
                    self.fills.observer_pnl -= i128::try_from(buy.paid()).unwrap();
                }
                (Trader::Observer, Landing::Sale(sale)) => {
                    self.fills.observer_coins -= order.amount;
                    self.fills.observer_pnl += i128::try_from(sale.net()).unwrap();
                }
                (Trader::Whale, Landing::Buy(_)) => {}
                _ => panic!("unexpected fill: {trader:?} {fill:?}"),
            }
            let wait = due - order.accepted_at;
            self.fills.min_wait = Some(self.fills.min_wait.map_or(wait, |min| min.min(wait)));
            self.fills.max_wait = self.fills.max_wait.max(wait);
            self.fills.max_jump_bps = self
                .fills
                .max_jump_bps
                .max(before.abs_diff(front(&self.curves)));
            self.fills.count += 1;
        }
    }
}

fn run(trial: Trial, delay: u32, placement: Placement) -> Fills {
    let mut market = Market::new(trial.curves, delay);
    for slot in EARLY_STEADY_SLOTS {
        market.land_through(slot);
        market.submit(Trader::Steady, OrderKind::Buy, SOL, slot);
    }
    market.land_through(TARGET_SLOT);
    if matches!(placement, Placement::BeforeAcceptance) {
        market.submit(Trader::Observer, OrderKind::Buy, 2 * SOL, TARGET_SLOT);
    }
    market.submit(Trader::Steady, OrderKind::Buy, SOL, TARGET_SLOT);
    market.submit(
        Trader::Whale,
        OrderKind::Buy,
        trial.whale_spend,
        TARGET_SLOT,
    );
    if matches!(placement, Placement::AfterAcceptance) {
        assert!(matches!(market.queue.back(), Some((Trader::Whale, _))));
        market.submit(Trader::Observer, OrderKind::Buy, 2 * SOL, TARGET_SLOT);
    }

    let exit_slot = TARGET_SLOT + i64::from(delay);
    market.land_through(exit_slot);
    if market.fills.observer_coins > 0 {
        market.submit(
            Trader::Observer,
            OrderKind::Sale,
            market.fills.observer_coins,
            exit_slot,
        );
        market.land_through(exit_slot + i64::from(delay));
    }
    assert!(market.queue.is_empty());
    assert_eq!(market.fills.observer_coins, 0);
    market.fills
}

fn write_report(mut out: impl Write) -> io::Result<()> {
    let (trials, done) = reference_trials();
    writeln!(
        out,
        "{} trials, seed {SEED:#018x}, reference: {}-minute raid, {} wins at minute {}",
        trials.len(),
        done.opening.minutes,
        HOUSE[done.winner.index()],
        done.at
    )?;
    let [first, second] = EARLY_STEADY_SLOTS;
    writeln!(
        out,
        "Steady buys at slots {first}, {second}, {TARGET_SLOT}. Whale and observer buys target slot {TARGET_SLOT}."
    )?;
    writeln!(out, "| Delay (slots) | Wait (slots/ms) | Honest max front jump (bp) | Forced FIFO P&L (lamports) | Priority P&L (lamports) | Priority buyer loss (base units, bp) | Priority max front jump (bp) | Fills | Peak pending |")?;
    writeln!(out, "|---:|---:|---:|---:|---:|---:|---:|---:|---:|")?;
    for delay in DELAYS {
        let mut honest_jump = 0;
        let mut priority_jump = 0;
        let mut honest_coins = 0;
        let mut priority_coins = 0;
        let mut priority_pnl = 0;
        let mut fifo_pnl = 0;
        let mut count = 0;
        let mut peak = 0;
        let mut min_wait = i64::MAX;
        let mut max_wait = 0;
        for &trial in &trials {
            let honest = run(trial, delay, Placement::Honest);
            let priority = run(trial, delay, Placement::BeforeAcceptance);
            honest_jump = honest_jump.max(honest.max_jump_bps);
            priority_jump = priority_jump.max(priority.max_jump_bps);
            honest_coins += honest.steady_coins;
            priority_coins += priority.steady_coins;
            priority_pnl += priority.observer_pnl;
            if delay > 0 {
                let fifo = run(trial, delay, Placement::AfterAcceptance);
                assert_eq!(fifo.steady_coins, honest.steady_coins);
                fifo_pnl += fifo.observer_pnl;
                count += fifo.count;
                peak = peak.max(fifo.peak_pending);
                min_wait = min_wait.min(fifo.min_wait.unwrap());
                max_wait = max_wait.max(fifo.max_wait);
            }
            for fills in [honest, priority] {
                count += fills.count;
                peak = peak.max(fills.peak_pending);
                min_wait = min_wait.min(fills.min_wait.unwrap());
                max_wait = max_wait.max(fills.max_wait);
            }
        }
        assert_eq!((min_wait, max_wait), (i64::from(delay), i64::from(delay)));
        let loss = honest_coins - priority_coins;
        let loss_bps = BPS * loss / honest_coins;
        let fifo = if delay == 0 {
            "—".into()
        } else {
            fifo_pnl.to_string()
        };
        let wait_ms = u64::try_from(max_wait).unwrap() * SLOT_MS;
        writeln!(out, "| {delay} | {max_wait}/{wait_ms} | {honest_jump} | {fifo} | {priority_pnl} | {loss}, {loss_bps} | {priority_jump} | {count} | {peak} |")?;
    }
    Ok(())
}

fn main() {
    write_report(io::stdout().lock()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use poleis_war::war::{fight, the_four};

    #[test]
    fn report_pins_the_schedule_and_measured_rows() {
        let mut report = Vec::new();
        write_report(&mut report).unwrap();
        assert_eq!(
            std::str::from_utf8(&report).unwrap(),
            include_str!("studies/march/expected.txt")
        );
    }

    #[test]
    fn checkpoints_replay_the_first_reference_to_settlement() {
        let (trials, done) = reference_trials();
        let (_, reference) = fight(&Rules::MEME_ROYALE, &the_four()[0]).unwrap();
        assert_eq!(done, reference);
        assert_eq!(trials.len(), 20);
        assert_eq!(
            trials[0].curves.map(|curve| mcap(&curve).unwrap()),
            done.opening.m0
        );
        assert!(trials
            .iter()
            .all(|trial| (4 * SOL..=8 * SOL).contains(&trial.whale_spend)));
    }

    #[test]
    fn acceptance_leaves_the_curve_alone_and_same_slot_fills_keep_order() {
        let trial = reference_trials().0[0];
        let mut market = Market::new(trial.curves, 4);
        market.submit(Trader::Steady, OrderKind::Buy, SOL, 0);
        market.submit(Trader::Whale, OrderKind::Buy, trial.whale_spend, 0);
        market.land_through(3);
        assert_eq!(market.curves, trial.curves);
        assert_eq!(market.fills.count, 0);
        market.land_through(4);
        let (_, steady) = poleis_war::law::buy_on(&market.rules, &trial.curves[0], SOL).unwrap();
        assert_eq!(market.fills.steady_coins, steady.coins);
        assert_eq!(market.fills.count, 2);
        assert!(market.queue.is_empty());
    }

    #[test]
    fn delay_keeps_economics_fixed_but_priority_changes_them() {
        for trial in reference_trials().0 {
            let base = run(trial, 0, Placement::Honest);
            let stress = run(trial, 0, Placement::BeforeAcceptance);
            assert!(stress.observer_pnl > 0);
            assert!(stress.steady_coins < base.steady_coins);
            for delay in DELAYS.into_iter().skip(1) {
                let fifo = run(trial, delay, Placement::AfterAcceptance);
                let priority = run(trial, delay, Placement::BeforeAcceptance);
                assert!(fifo.observer_pnl < 0);
                assert_eq!(fifo.steady_coins, base.steady_coins);
                assert_eq!(priority.observer_pnl, stress.observer_pnl);
                assert_eq!(priority.steady_coins, stress.steady_coins);
                assert_eq!(priority.max_jump_bps, stress.max_jump_bps);
                assert_eq!(priority.min_wait, Some(i64::from(delay)));
                assert_eq!(priority.max_wait, i64::from(delay));
            }
        }
    }
}
