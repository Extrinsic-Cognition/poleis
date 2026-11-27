//! Opt-in timed orders. All instants are supplied by the caller, seconds from raid start.
use crate::{law::*, war::War};
use std::string::String;
use std::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct March {
    pub who: String,
    pub house: Side,
    pub order: Order,
    /// Cost basis removed with escrowed sale coins; restored on a refund.
    pub brought: u128,
}

impl War {
    pub(crate) fn marching(&self) -> bool {
        self.rules.march_s > 0 && self.opening.is_some() && !self.march_settled
    }

    fn accepting(&self, now: i64) -> Result<(), &'static str> {
        self.time_forward(now)?;
        let o = self.opening.ok_or("the raid has not opened")?;
        if now < 0 || trades_until(self.phase, o.minutes).is_none_or(|m| now / 60 > m as i64) {
            return Err(DECIDED);
        }
        Ok(())
    }

    fn time_forward(&self, now: i64) -> Result<(), &'static str> {
        if self.evaluated_at.is_some_and(|t| now < t) || self.last_acceptance.is_some_and(|t| now < t) {
            return Err(ORDER_TIME);
        }
        Ok(())
    }

    /// The caller funds a quote wallet explicitly. Acceptance takes the entire spend out of it.
    pub fn fund(&mut self, who: &str, quote: u128) -> Result<(), &'static str> {
        let balance = self.wallets.entry(who.into()).or_default();
        *balance = balance.checked_add(quote).ok_or(OVERFLOW)?;
        Ok(())
    }

    pub fn enter_at(&mut self, who: &str, house: Side, spend: u128, minimum: u128, now: i64) -> Result<Option<u128>, &'static str> {
        if !self.marching() {
            if buy_on(&self.rules, &self.curves[house.index()], spend)?.1.coins < minimum {
                return Err("past the order's minimum");
            }
            return self.enter(who, house, spend).map(Some);
        }
        self.accepting(now)?;
        if self.orders.len() == MARCH_CAPACITY {
            return Err(MARCH_FULL);
        }
        check_march_owner(&self.rules, who, self.orders.iter().map(|order| order.who.as_str()))?;
        let order = Order::accept(&self.rules, OrderKind::Buy, spend, minimum, now, self.last_acceptance)?;
        let balance = self.wallets.get(who).copied().unwrap_or(0).checked_sub(spend).ok_or("not enough quote")?;
        self.wallets.insert(who.into(), balance);
        self.orders.push_back(March {
            who: who.into(),
            house,
            order,
            brought: 0,
        });
        self.last_acceptance = Some(now);
        Ok(None)
    }

    pub fn sell_at(&mut self, who: &str, house: Side, coins: u128, minimum: u128, now: i64) -> Result<Option<Sale>, &'static str> {
        if !self.marching() {
            if sell_on(&self.rules, &self.curves[house.index()], coins, 0)?.1.net() < minimum {
                return Err("past the order's minimum");
            }
            return self.sell(who, house, coins, 0).map(Some);
        }
        self.accepting(now)?;
        if self.orders.len() == MARCH_CAPACITY {
            return Err(MARCH_FULL);
        }
        if self.is_staker(who) {
            return Err("staked");
        }
        check_march_owner(&self.rules, who, self.orders.iter().map(|order| order.who.as_str()))?;
        let order = Order::accept(&self.rules, OrderKind::Sale, coins, minimum, now, self.last_acceptance)?;
        let key = (who.into(), house);
        let held = self.coins_of(who, house);
        let basis = self.brought.get(&key).copied().unwrap_or(0);
        let left = brought_after_sale(basis, held, coins)?;
        self.coins.insert(key.clone(), held - coins);
        self.brought.insert(key, left);
        self.held[house.index()] -= coins;
        self.orders.push_back(March {
            who: who.into(),
            house,
            order,
            brought: basis - left,
        });
        self.last_acceptance = Some(now);
        Ok(None)
    }

    /// Drain every due order in acceptance order. Late calls use the same curve because
    /// timed mutation methods drain the queue first. Orders beyond the raid window refund.
    pub fn land(&mut self, now: i64) -> Result<Vec<Landing>, &'static str> {
        self.time_forward(now)?;
        let mut next = self.clone();
        let result = next.land_inner(now)?;
        next.evaluated_at = Some(now);
        *self = next;
        Ok(result)
    }
    fn land_inner(&mut self, now: i64) -> Result<Vec<Landing>, &'static str> {
        let decided = matches!(self.phase, Phase::Decided { .. });
        let last = self.opening.and_then(|o| trades_until(self.phase, o.minutes));
        let mut results = Vec::new();
        while let Some(m) = self.orders.front().cloned() {
            if !decided && m.order.lands_at > now {
                break;
            }
            let expired = last.is_some_and(|v| m.order.lands_at / 60 > v as i64);
            let i = m.house.index();
            let (curve, result) = land_on(&self.rules, &self.curves[i], &m.order, now, decided || expired, 0)?;
            self.curves[i] = curve;
            let key = (m.who.clone(), m.house);
            match result {
                Landing::Buy(b) => {
                    self.held[i] += b.coins;
                    *self.coins.entry(key.clone()).or_default() += b.coins;
                    *self.brought.entry(key).or_default() += b.paid();
                    self.protocol_fees += b.protocol;
                    self.creator_fees[i] += b.creator;
                    self.fund(&m.who, m.order.amount - b.paid())?;
                    if !self.came.contains(&m.who) {
                        self.came.push(m.who.clone());
                    }
                }
                Landing::Sale(s) => {
                    self.protocol_fees += s.protocol;
                    self.creator_fees[i] += s.creator;
                    self.fund(&m.who, s.net())?;
                }
                Landing::Refund => match m.order.kind {
                    OrderKind::Buy => self.fund(&m.who, m.order.amount)?,
                    OrderKind::Sale => {
                        *self.coins.entry(key.clone()).or_default() += m.order.amount;
                        *self.brought.entry(key).or_default() += m.brought;
                        self.held[i] += m.order.amount;
                    }
                },
            }
            self.orders.pop_front();
            results.push(result);
        }
        Ok(results)
    }

    /// Read a minute at an exact supplied instant, including arrivals partway
    /// through that minute. The old minute(m) remains minute_at(m * 60).
    pub fn minute_at(&mut self, now: i64) -> Result<Phase, &'static str> {
        if now < 0 {
            return Err(ORDER_TIME);
        }
        let m = u32::try_from(now / 60).map_err(|_| OVERFLOW)?;
        if !self.marching() {
            return self.minute(m);
        }
        let mut next = self.clone();
        next.land(now)?;
        let o = next.opening.ok_or("the raid has not opened")?;
        next.phase = minute_on(
            &next.rules,
            next.phase,
            o.minutes,
            m,
            [&next.curves[0], &next.curves[1]],
            o.m0,
            next.chests[0] + next.chests[1],
        )?;
        if matches!(next.phase, Phase::Decided { .. }) {
            next.land(now)?;
        }
        let phase = next.phase;
        *self = next;
        Ok(phase)
    }

    pub fn desert_at(&mut self, who: &str, now: i64) -> Result<u128, &'static str> {
        if !self.marching() {
            return self.desert(who);
        }
        self.accepting(now)?;
        if self.is_staker(who) {
            return Err("staked");
        }
        let mut next = self.clone();
        next.land(now)?;
        let paid = next.desert_now(who)?;
        next.fund(who, paid)?;
        *self = next;
        Ok(paid)
    }

    pub fn come_in_at(&mut self, who: &str, house: Side, now: i64) {
        if !self.marching() {
            self.come_in(who, house);
            return;
        }
        let mut next = self.clone();
        if next.fund(who, SOL).and_then(|()| next.enter_at(who, house, SOL, 0, now)).is_err() {
            self.refused.push(who.into());
        } else {
            *self = next;
        }
    }

    pub fn cross_at(&mut self, who: &str, from: Side, now: i64) -> Result<Option<u128>, &'static str> {
        if !self.marching() {
            return self.cross(who, from).map(Some);
        }
        self.accepting(now)?;
        if self.is_staker(who) {
            return Err("staked");
        }
        let mut next = self.clone();
        next.land(now)?;
        let paid = next.carry_out(who, from)?;
        next.fund(who, paid)?;
        next.enter_at(who, from.other(), paid, 0, now)?;
        *self = next;
        Ok(None)
    }
}
