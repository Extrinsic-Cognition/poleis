//! THE WHOLE WAR, minute by minute over a script (feature std): the owner's `fight(name, script, stand_reserve, burn=True, nested=False)`
//! (tests/reference/meme-royale-rulebook-sim.py) on the law's integers. Every number is moved by one call of the core (law.rs): `buy_on`,
//! `stake_on`, `burn_on`, `minute_on`, `settle_on`, `weight_of`, `weights_of`, `share_of`, `backed_of`, `dump_tax_of`. This file keeps the
//! fighters' holdings, the script's order and the model's loop.
//!
//! THE LOOP, as the model runs it. Forty fighters come in, one SOL each, the crow's and the wolf's in turn. The crow's last ones in stake all
//! their coins, crow19 first and down, until the raid opens. Then, minute by minute from 0: the minute's events in the order the model sorts
//! them (by minute, then kind, then name, as strings), the stand's reserve in the stand's first minute, then the minute's reading (`minute_on`),
//! until the raid is decided; then the settlement.
//!
//! ONE READING THAT IS THE BRIEF'S AND NOT THE MODEL'S. What one who deserts may carry is half of what he brought INTO THAT HOUSE (the brief's
//! "brought is the lamports he put into that house"), and what he brought ends with his burn (a desert or a change of sides sets it to 0):
//! the model keeps one `brought` a fighter, his entry, so one who changes sides and later deserts his new house would carry half of his first
//! entry there, and half of what his change bought here. No reference war has a fighter do both, or come back in after a desert, so no number
//! of the reference moves by it.

use crate::law::*;
use std::collections::BTreeMap;
use std::format;
use std::string::String;
use std::vec::Vec;

/// how many come in on each side before the fight
pub const N: usize = 20;
/// the houses' names in the model: the crow attacks, the wolf defends
pub const HOUSE: [&str; 2] = ["crow", "wolf"];

/// an event's kind, in the order the model's strings sort in ('cross' < 'desert' < 'reinforce')
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// one of the wolf's changes sides to the crow
    Cross,
    /// one deserts the house he holds
    Desert,
    /// one comes in with one SOL for `house`
    Reinforce,
}

/// one event of a script, ordered as the model's tuples (minute, kind, who, house) sort
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    pub minute: u32,
    pub kind: Kind,
    pub who: String,
    pub house: Side,
}

/// the model's `base(T)`: ten of the wolf's change sides in the first quarter, ten of the crow's desert in the second
pub fn base(t: u32) -> Vec<Event> {
    let q = (t / 4).max(1);
    let mut ev: Vec<Event> = (0..10).map(|k| Event { minute: 1 + k * q / 10, kind: Kind::Cross, who: format!("wolf{k}"), house: Side::Attacker }).collect();
    ev.extend((0..10).map(|k| Event { minute: t / 4 + k * q / 10, kind: Kind::Desert, who: format!("crow{k}"), house: Side::Attacker }));
    ev
}

/// the model's `spread(house, n, a, b, tag)`: `n` come in for `house`, one SOL each, at minute int(a * T + k * (b - a) * T / n), in the
/// model's own floats and in its order of operations, so every minute is the model's
pub fn spread(house: Side, n: u32, a: f64, b: f64, tag: &str, t: u32) -> Vec<Event> {
    (0..n)
        .map(|k| {
            let x = a * t as f64 + k as f64 * (b - a) * t as f64 / n as f64;
            Event { minute: x as u32, kind: Kind::Reinforce, who: format!("{tag}{k}"), house }
        })
        .collect()
}

/// a war the model ran: its name, its script of the raid's length, and the reserve of the side in a last stand (n come in, one SOL each, in
/// the stand's first minute)
#[derive(Clone, Copy)]
pub struct Scenario {
    pub name: &'static str,
    pub script: fn(u32) -> Vec<Event>,
    pub reserve: Option<(Side, u32)>,
}

fn clean_push_20(t: u32) -> Vec<Event> {
    spread(Side::Attacker, 20, 0.0, 60.0 / t as f64, "crow-new", t)
}
fn clean_push_10(t: u32) -> Vec<Event> {
    spread(Side::Attacker, 10, 0.0, 60.0 / t as f64, "crow-new", t)
}

/// THE FOUR WARS OF THE REFERENCE (the model's `[SCENARIOS[0], SCENARIOS[-1], STANDS[3], STANDS[4]]`, each run with `, in SOL` on its name)
pub fn the_four() -> [Scenario; 4] {
    [
        Scenario { name: "The war as before, now on the clock, in SOL", script: base, reserve: None },
        Scenario { name: "A clean push: nobody deserts, the crow sends twenty in its first hour, in SOL", script: clean_push_20, reserve: None },
        Scenario { name: "A clean push of 10, and the wolf answers 15 in its last stand, in SOL", script: clean_push_10, reserve: Some((Side::Defender, 15)) },
        Scenario { name: "A clean push of 10, and the wolf answers 20 in its last stand, in SOL", script: clean_push_10, reserve: Some((Side::Defender, 20)) },
    ]
}

/// how a raid was decided, in the model's words (its `how`)
pub fn how_text(how: How, at: u32) -> String {
    match how {
        How::StandFailed { met } => format!("beat its line at minute {met}; the last stand failed"),
        How::Revived => format!("revived in its last stand at minute {at}"),
        How::BothMet => format!("both beat their lines at minute {at}; the further along won"),
        How::Clock { tie } => format!("the clock ran out; the further along won{}", if tie { " (a tie goes to the defender)" } else { "" }),
    }
}

/// what a fighter ends with: the coins of the winner he holds, what backs them, and what he took out deserting
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fighter {
    pub holds: u128,
    pub backed: u128,
    pub took: u128,
}

/// a war's end: who won and how, the raid's numbers, the settlement, and every fighter in the order they came in
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Done {
    pub winner: Side,
    pub how: How,
    pub at: u32,
    pub opening: Opening,
    pub stakers: Vec<String>,
    pub mcap_end: [u128; 2],
    pub settlement: Settlement,
    pub winner_reserve: u128,
    pub winner_chest: u128,
    pub all_held: u128,
    pub weights: u128,
    pub dump: Option<DumpTax>,
    pub fighters: Vec<(String, Fighter)>,
}

/// THE WAR AS IT STANDS: both houses (attacker first), what each fighter holds of each and brought into it, the stake and the raid
#[derive(Clone, Debug)]
pub struct War {
    /// Public FIFO and liquid quote balances for the opt-in marching path.
    pub orders: std::collections::VecDeque<crate::march::March>,
    pub wallets: BTreeMap<String, u128>,
    pub last_acceptance: Option<i64>,
    /// Last supplied instant folded by the timed path; never a wall clock.
    pub evaluated_at: Option<i64>,
    pub march_settled: bool,
    pub rules: Rules,
    pub curves: [Curve; 2],
    pub chests: [u128; 2],
    pub held: [u128; 2],
    pub burned: [u128; 2],
    pub coins: BTreeMap<(String, Side), u128>,
    pub brought: BTreeMap<(String, Side), u128>,
    /// every fighter who came in, in the order he first did
    pub came: Vec<String>,
    /// what each deserter took out
    pub took: BTreeMap<String, u128>,
    /// those whose coming in the curve refused (it had no coins left)
    pub refused: Vec<String>,
    pub stakers: Vec<String>,
    pub staked: u128,
    pub opening: Option<Opening>,
    pub phase: Phase,
    /// THE FEES (THE GAME IS ONE SWARM, Nov 24 2026): the protocol's share of every trade, the publisher's; the dump tax's fee half; and
    /// each house's creator share of every trade into it, attacker first
    pub protocol_fees: u128,
    pub taxed: u128,
    pub creator_fees: [u128; 2],
}

impl War {
    pub fn new(rules: Rules) -> War {
        let c = Curve::born(&rules);
        War {
            orders: std::collections::VecDeque::new(),
            wallets: BTreeMap::new(),
            last_acceptance: None,
            evaluated_at: None,
            march_settled: false,
            rules,
            curves: [c, c],
            chests: [0; 2],
            held: [0; 2],
            burned: [0; 2],
            coins: BTreeMap::new(),
            brought: BTreeMap::new(),
            came: Vec::new(),
            took: BTreeMap::new(),
            refused: Vec::new(),
            stakers: Vec::new(),
            staked: 0,
            opening: None,
            phase: Phase::Open,
            protocol_fees: 0,
            taxed: 0,
            creator_fees: [0; 2],
        }
    }
    pub(crate) fn coins_of(&self, who: &str, h: Side) -> u128 {
        self.coins.get(&(who.into(), h)).copied().unwrap_or(0)
    }
    /// a buy of `spend` lamports of `house` by `who` (`buy_on`): the coins it got. A refusal changes nothing.
    pub fn enter(&mut self, who: &str, house: Side, spend: u128) -> Result<u128, &'static str> {
        if self.marching() { return Err("use enter_at with the acceptance instant"); }
        self.enter_now(who, house, spend)
    }
    pub(crate) fn enter_now(&mut self, who: &str, house: Side, spend: u128) -> Result<u128, &'static str> {
        let (after, b) = buy_on(&self.rules, &self.curves[house.index()], spend)?;
        self.curves[house.index()] = after;
        self.held[house.index()] += b.coins;
        *self.coins.entry((who.into(), house)).or_default() += b.coins;
        *self.brought.entry((who.into(), house)).or_default() += b.paid();
        self.protocol_fees += b.protocol;
        self.creator_fees[house.index()] += b.creator;
        if !self.came.iter().any(|c| c == who) {
            self.came.push(who.into());
        }
        Ok(b.coins)
    }
    /// the model's `come_in`: a fighter comes in with one SOL, or is counted refused when the curve has no coins left for him
    pub fn come_in(&mut self, who: &str, house: Side) {
        if self.enter(who, house, SOL).is_err() {
            self.refused.push(who.into());
        }
    }
    pub fn is_staker(&self, who: &str) -> bool {
        self.stakers.iter().any(|s| s == who)
    }
    /// `who` stakes all his coins of the attacker against the defender (`stake_on`): true when the raid opens with his stake
    pub fn stake(&mut self, who: &str) -> Result<bool, &'static str> {
        let staked = self.staked + self.coins_of(who, Side::Attacker);
        let opening = stake_on(&self.rules, &self.curves[0], &self.curves[1], staked)?;
        self.staked = staked;
        self.stakers.push(who.into());
        self.opening = opening;
        Ok(opening.is_some())
    }
    /// his burn of everything `who` holds of `house` (`burn_on`): what he is paid
    pub(crate) fn carry_out(&mut self, who: &str, house: Side) -> Result<u128, &'static str> {
        let i = house.index();
        let t = self.coins_of(who, house);
        let brought = self.brought.get(&(who.into(), house)).copied().unwrap_or(0);
        let (after, paid) = burn_on(&self.rules, &self.curves[i], t, brought)?;
        self.curves[i] = after;
        self.coins.insert((who.into(), house), 0);
        // a burn ends what he brought into the house (law.rs burn_on)
        self.brought.insert((who.into(), house), 0);
        self.held[i] -= t;
        self.burned[i] += t;
        Ok(paid)
    }
    /// `who` deserts the house he holds (the crow's if he holds any of it, else the wolf's, as the model picks it): what he takes out
    pub fn desert(&mut self, who: &str) -> Result<u128, &'static str> {
        if self.marching() { return Err("use desert_at with the instant"); }
        self.desert_now(who)
    }
    pub(crate) fn desert_now(&mut self, who: &str) -> Result<u128, &'static str> {
        let src = if self.coins_of(who, Side::Attacker) > 0 { Side::Attacker } else { Side::Defender };
        let paid = self.carry_out(who, src)?;
        self.took.insert(who.into(), paid);
        Ok(paid)
    }
    /// `who` changes sides from `from` to the other house: his burn there, and what it paid buys the other house (`buy_on`, fees and all)
    pub fn cross(&mut self, who: &str, from: Side) -> Result<u128, &'static str> {
        if self.marching() { return Err("use cross_at with the instant"); }
        let to = from.other();
        // a change of sides is one act: the buy is asked of the law before the burn is taken
        let mut w = self.clone();
        let paid = w.carry_out(who, from)?;
        let (after, b) = buy_on(&w.rules, &w.curves[to.index()], paid)?;
        w.curves[to.index()] = after;
        w.held[to.index()] += b.coins;
        *w.coins.entry((who.into(), to)).or_default() += b.coins;
        *w.brought.entry((who.into(), to)).or_default() += b.paid();
        w.protocol_fees += b.protocol;
        w.creator_fees[to.index()] += b.creator;
        *self = w;
        Ok(b.coins)
    }
    /// a sale of `coins` of `house` by `who` (`sell_on`, at the dump tax's rate `tax_bps`): the protocol's fee and the tax's fee half to the
    /// publisher's fee account, the creator's fee to its house, the tax's other half into the house's chest (`tax_split`); what he gets
    pub fn sell(&mut self, who: &str, house: Side, coins: u128, tax_bps: u32) -> Result<Sale, &'static str> {
        if self.marching() { return Err("use sell_at with the acceptance instant"); }
        let i = house.index();
        if coins > self.coins_of(who, house) {
            return Err(NOT_HELD);
        }
        let (after, s) = sell_on(&self.rules, &self.curves[i], coins, tax_bps)?;
        let brought = self.brought.get(&(who.into(), house)).copied().unwrap_or(0);
        let left = brought_after_sale(brought, self.coins_of(who, house), coins)?;
        let (to_fees, to_chest) = tax_split(s.tax);
        self.curves[i] = after;
        self.held[i] -= coins;
        self.chests[i] += to_chest;
        *self.coins.entry((who.into(), house)).or_default() -= coins;
        self.brought.insert((who.into(), house), left);
        self.protocol_fees += s.protocol;
        self.taxed += to_fees;
        self.creator_fees[i] += s.creator;
        Ok(s)
    }
    /// the raid read at minute `m` (`minute_on`): its phase after it
    pub fn minute(&mut self, m: u32) -> Result<Phase, &'static str> {
        if self.marching() { return self.minute_at(m as i64 * 60); }
        let o = self.opening.ok_or("the raid has not opened")?;
        self.phase = minute_on(&self.rules, self.phase, o.minutes, m, [&self.curves[0], &self.curves[1]], o.m0, self.chests[0] + self.chests[1])?;
        Ok(self.phase)
    }
    /// THE SETTLEMENT (`settle_on`): the loser's pot buys the winner's coin; every holder of the loser receives his share of the coins bought
    /// by his weight (`weight_of` over `weights_of`, stakers of the raid a tenth better); the loser's reserve and chest go to nothing; the
    /// winner's holders are valued by the coins each holds (`backed_of`); the winner's dump tax (`dump_tax_of`)
    pub fn settle(&mut self) -> Result<Done, &'static str> {
        if !self.orders.is_empty() { return Err(ORDERS_DUE); }
        let o = self.opening.ok_or("the raid has not opened")?;
        let Phase::Decided { winner, how, at } = self.phase else { return Err("the raid is not decided") };
        let (wi, li) = (winner.index(), winner.other().index());
        let loser = winner.other();
        let mcap_end = [mcap(&self.curves[0])?, mcap(&self.curves[1])?];
        let pot_in = self.curves[li].rq + self.chests[li];
        let (after, s) = settle_on(&self.rules, &self.curves[wi], pot_in)?;
        self.curves[wi] = after;
        self.chests[wi] += s.to_chest;
        self.protocol_fees += s.buy.protocol;
        self.creator_fees[wi] += s.buy.creator;
        // the stakers' bonus is the loser's only when the loser is the attacker, whose holders staked
        let staked_here = |w: &War, who: &str| if loser == Side::Attacker && w.is_staker(who) { w.coins_of(who, loser) } else { 0 };
        let weights = weights_of(&self.rules, self.held[li], if loser == Side::Attacker { self.staked } else { 0 })?;
        let holders: Vec<(String, u128)> = self.coins.iter().filter(|((_, h), t)| *h == loser && **t > 0).map(|((w, _), t)| (w.clone(), *t)).collect();
        for (who, t) in &holders {
            let share = share_of(s.buy.coins, weight_of(&self.rules, *t, staked_here(self, who))?, weights)?;
            self.coins.insert((who.clone(), loser), 0);
            *self.coins.entry((who.clone(), winner)).or_default() += share;
            self.held[wi] += share;
        }
        self.held[li] = 0;
        self.curves[li].rq = 0;
        self.chests[li] = 0;
        let all_held: u128 = self.coins.iter().filter(|((_, h), _)| *h == winner).map(|(_, t)| *t).sum();
        let backing = self.curves[wi].rq + self.chests[wi];
        let mut fighters = Vec::new();
        for who in &self.came {
            let holds = self.coins_of(who, winner);
            fighters.push((who.clone(), Fighter { holds, backed: backed_of(backing, holds, all_held)?, took: self.took.get(who).copied().unwrap_or(0) }));
        }
        if self.rules.march_s > 0 { self.march_settled = true; }
        Ok(Done {
            winner,
            how,
            at,
            opening: o,
            stakers: self.stakers.clone(),
            mcap_end,
            settlement: s,
            winner_reserve: self.curves[wi].rq,
            winner_chest: self.chests[wi],
            all_held,
            weights,
            dump: dump_tax_of(&self.rules, o.m0[wi], mcap_end[wi])?,
            fighters,
        })
    }
}

/// THE WAR, WHOLE (the model's `fight(burn=True, nested=False)`): forty come in, the crow's last ones stake until the raid opens, then the
/// script minute by minute (the stand's reserve in the stand's first minute), then the settlement. A staker's own events are skipped, as the
/// model skips them.
pub fn fight(rules: &Rules, s: &Scenario) -> Result<(War, Done), &'static str> {
    let mut w = War::new(*rules);
    for i in 0..N {
        for h in [Side::Attacker, Side::Defender] {
            w.enter(&format!("{}{i}", HOUSE[h.index()]), h, SOL)?;
        }
    }
    for i in (10..N).rev() {
        if w.opening.is_some() {
            break;
        }
        w.stake(&format!("crow{i}"))?;
    }
    let t = w.opening.ok_or("the stake never reached the need")?.minutes;
    let mut events = (s.script)(t);
    events.sort();
    for m in 0..=t + rules.stand_minutes {
        let now = m as i64 * 60;
        if rules.march_s > 0 { w.land(now)?; }
        for e in events.iter().filter(|e| e.minute == m) {
            match e.kind {
                Kind::Reinforce => w.come_in_at(&e.who, e.house, now),
                Kind::Cross if !w.is_staker(&e.who) => {
                    w.cross_at(&e.who, Side::Defender, now)?;
                }
                Kind::Desert if !w.is_staker(&e.who) => {
                    w.desert_at(&e.who, now)?;
                }
                _ => {}
            }
        }
        if let (Phase::Stand { side, ends, .. }, Some((rs, n))) = (w.phase, s.reserve) {
            if rs == side && m + rules.stand_minutes == ends + 1 {
                for k in 0..n {
                    w.come_in_at(&format!("{}-stand{k}", HOUSE[side.index()]), side, now);
                }
            }
        }
        if let Phase::Decided { .. } = w.minute(m)? {
            break;
        }
    }
    let done = w.settle()?;
    Ok((w, done))
}
