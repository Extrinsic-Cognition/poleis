//! THE WAR'S ONE LAW, in integers (Nov 24 2026). Pure: no import, no float, no clock but the minutes and seconds handed in. The same answer on
//! every reader: the owner's model (tests/reference/meme-royale-rulebook-sim.py, `fight(burn=True, nested=False)`), this crate's whole war
//! (war.rs) and the war's program on a chain (poleis/program/poleis_wars), which moves every number by one whole call of this file.
//!
//! THE GROUND IS PUMP.FUN'S CURVE, priced in SOL (the model's `War(nested=False)`). A house is a coin on its own curve: virtual coins `vt`,
//! virtual quote `vq` in lamports, real coins `rt` it has left to sell (`rt0` at its birth) and the real quote `rq` it holds. A buy takes off two
//! fees, each the ceiling of its share (95 and 30 bp), and buys `(quote - 1) * vt / (vq + quote - 1)` coins; a sale gets `coins * vq / (vt +
//! coins)` less the same two fees (war-rules-sim.py `War._buy` and `War._sell`, checked there against pump.fun's own program, trade for trade).
//!
//! WHAT IS MEME ROYALE'S (docs/meme-royale-the-program-read-back-hypertrace-2026-11-24.md, its rules 2 to 9). A raid opens by a stake of the
//! attacker's coins against an opponent between half and twice its size: the stake must reach the larger of 5% of the attacker's market cap and
//! 2% of the gap between the two, and no more than 20% may be staked (`stake_on`). It lasts 15 minutes to 6 hours by the coins the two houses
//! have sold (`raid_minutes`). Each side has a line, the market cap it would have holding both pools and both chests, and the line falls in a
//! straight line to nothing by the raid's end (`reading`, `meets`). The side that did not beat its line gets a last stand of two minutes, to
//! climb its line and a ninth more (`revives`); if both beat their lines, or the clock runs out, the one further along wins, and a tie goes to
//! the defender (`further`, `minute_on`). At the settlement the loser's pot buys the winner's coin on the winner's curve for the loser's holders,
//! stakers 10% better (`settle_on`, `share_of`), and the winners who sell right after pay a dump tax that falls to nothing (`dump_tax_of`).
//!
//! WHAT IS HIS. A deserter, or one who changes sides, during a raid leaves with at most half of what he brought, and his coins are burned, never
//! sold into his house's curve: the house pays him what they would fetch, at most that half, out of its reserve, so leaving sinks his side only by
//! what he is paid (`burn_on`). One who changes sides buys the other house with it, a buy like any other.
//!
//! THE READINGS. Where the model uses floats this law uses the integers the brief names and no others: the stake's need and value compared by
//! cross-multiplying, the line's `req` (`max(line - now, 0) * (T - m) / T`) never divided, the revival's `req / (1 - 0.10)` as `* 10000 / 9000`,
//! the further-along fractions cross-multiplied, his half `brought * 5000 / 10000`. On the four reference wars every one of them gives the
//! model's own numbers (tests/reference.rs), the stake's need aside: the model keeps it as a float, `0.05 * m0` (1637961688.45), and this law
//! as its floor.
//!
//! THE RANGE. Every amount of a curve is at most AMOUNT_MAX (2^56: 72 million SOL in lamports, 72 billion coins of six decimals) and `Rules::check`
//! refuses a curve that could pass it; every product below is a checked one, so no reading wraps, and a product past u128 is refused (OVERFLOW),
//! which no war inside the range reaches.

/// basis points: 10000 is the whole
pub const BPS: u128 = 10_000;
/// per thousand
pub const PM: u128 = 1_000;
/// one SOL, in lamports
pub const SOL: u128 = 1_000_000_000;
/// the law's minute, in seconds: a raid's length is counted in these minutes, and so is its last stand
pub const MINUTE_S: u128 = 60;
/// the most any amount of a curve may be, coins or lamports
pub const AMOUNT_MAX: u128 = 1 << 56;

/// a product or a sum that would leave u128 (none does inside the law's range)
pub const OVERFLOW: &str = "a product would leave the law's integers";
/// pump.fun's NotEnoughTokensToBuy: a buy that would take more coins than the curve has left
pub const NOT_ENOUGH_TOKENS: &str = "the curve has not that many coins left to sell";
/// a sale too small to give its seller anything once the fees and the tax are taken
pub const NOTHING_OUT: &str = "too small to take anything out";
/// a sale whose quote is more than the house's reserve holds
pub const RESERVE_SHORT: &str = "the house's reserve cannot pay that sale";
/// a sale of more coins than the curve ever sold
pub const NOT_SOLD: &str = "more coins than the curve sold";
/// an opponent smaller than half or larger than twice the attacker
pub const UNFAIR: &str = "the opponent is not between half and twice the attacker's market cap";
/// a stake past the most that may be staked
pub const STAKE_CAP: &str = "the stake is past the most of the attacker's market cap that may be staked";
/// a minute read of a raid already decided
pub const DECIDED: &str = "the raid is decided";
/// a sale of more coins than their holder holds
pub const NOT_HELD: &str = "more coins than he holds";

#[inline(always)]
fn mul(a: u128, b: u128) -> Result<u128, &'static str> {
    a.checked_mul(b).ok_or(OVERFLOW)
}
#[inline(always)]
fn add(a: u128, b: u128) -> Result<u128, &'static str> {
    a.checked_add(b).ok_or(OVERFLOW)
}

/// THE WAR'S CONSTANTS, one set for every house of a war: pump.fun's curve and fees, Meme Royale's settings, his half. `MEME_ROYALE` is the
/// set the owner's model runs (its constants at the top of meme-royale-rulebook-sim.py and war-rules-sim.py).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Seconds an order accepted during a raid marches. Zero preserves the original law.
    pub march_s: u32,
    /// Least marching input in lamports for a buy or base coin units for a sale. Zero disables the floor.
    pub march_min_order: u64,
    /// Most pending orders owned by one maker across both sides of a raid. Zero disables this cap.
    pub march_max_per_owner: u8,
    /// a curve at its birth: virtual coins, virtual quote in lamports, and the real coins it can sell
    pub vt: u128,
    pub vq: u128,
    pub rt: u128,
    /// pump.fun's protocol fee and creator fee, in basis points, each the ceiling of its share
    pub protocol_bps: u32,
    pub creator_bps: u32,
    /// the stake that opens a raid, per thousand: of the attacker's market cap, of the gap between the two; and the most that may be staked
    pub stake_own_pm: u32,
    pub stake_gap_pm: u32,
    pub stake_max_pm: u32,
    /// the opponent's market cap, per thousand of the attacker's: at least, at most
    pub opponent_min_pm: u32,
    pub opponent_max_pm: u32,
    /// a raid's shortest and longest length and its step, in seconds
    pub raid_min_s: u32,
    pub raid_max_s: u32,
    pub raid_step_s: u32,
    /// the last stand: its minutes, and the share of what is left to climb a side in it must climb more (1000 bp: its line and a ninth more)
    pub stand_minutes: u32,
    pub revival_bps: u32,
    /// the stakers' bonus at the conversion, per thousand: their staked coins weigh a thousand and this
    pub staker_bonus_pm: u32,
    /// what one who deserts or changes sides may carry away of what he brought, in basis points: his half
    pub keep_bps: u32,
    /// the dump tax: its least starting rate in basis points; its shortest and longest length and its step, in seconds
    pub dump_min_bps: u32,
    pub dump_min_s: u32,
    pub dump_max_s: u32,
    pub dump_step_s: u32,
}

impl Rules {
    /// the owner's model's constants: VT, VQ_SOL, RT, PROTOCOL 95 and CREATOR 30 (war-rules-sim.py); MIN_RAID 900, MAX_RAID 21600, RAID_STEP
    /// 900, STAKE_OWN 0.05, STAKE_DELTA 0.02, STAKE_MAX 0.20, LAST_STAND 2, REVIVAL 0.10, MIN_TAX 0.10, STAKER_BONUS 0.10, OPP_MIN 0.5, OPP_MAX
    /// 2.0, KEEP 0.5 (meme-royale-rulebook-sim.py); the dump tax's 60, 3600 and 60 seconds from Meme Royale's bytecode (the read-back's rule 9)
    pub const MEME_ROYALE: Rules = Rules {
        march_s: 0,
        march_min_order: 0,
        march_max_per_owner: 0,
        vt: 1_073_000_000_000_000,
        vq: 30_000_000_000,
        rt: 793_100_000_000_000,
        protocol_bps: 95,
        creator_bps: 30,
        stake_own_pm: 50,
        stake_gap_pm: 20,
        stake_max_pm: 200,
        opponent_min_pm: 500,
        opponent_max_pm: 2_000,
        raid_min_s: 900,
        raid_max_s: 21_600,
        raid_step_s: 900,
        stand_minutes: 2,
        revival_bps: 1_000,
        staker_bonus_pm: 100,
        keep_bps: 5_000,
        dump_min_bps: 1_000,
        dump_min_s: 60,
        dump_max_s: 3_600,
        dump_step_s: 60,
    };

    /// both fees, in basis points
    pub fn fees_bps(&self) -> u128 {
        self.protocol_bps as u128 + self.creator_bps as u128
    }

    /// the rules the law can carry, or why not
    pub fn check(&self) -> Result<(), &'static str> {
        if self.rt == 0 || self.rt >= self.vt {
            return Err("a curve's real coins are none, or not fewer than its virtual coins");
        }
        if self.vt > AMOUNT_MAX || self.vq == 0 || self.vq > AMOUNT_MAX {
            return Err("a curve's amounts are outside the law's range");
        }
        // the highest a curve's virtual quote reaches, all its real coins sold: vq * vt / (vt - rt)
        if mul(self.vq, self.vt)? / (self.vt - self.rt) >= AMOUNT_MAX {
            return Err("a curve sold out would leave the law's range");
        }
        if self.fees_bps() >= BPS {
            return Err("the fees are the whole of a trade or more");
        }
        if self.stake_own_pm as u128 > PM || self.stake_gap_pm as u128 > PM || self.stake_max_pm as u128 > PM || self.stake_own_pm > self.stake_max_pm {
            return Err("a stake's shares are past the whole, or its need past its most");
        }
        if self.opponent_min_pm as u128 > PM || (self.opponent_max_pm as u128) < PM {
            return Err("a house would be no fair opponent of one its own size");
        }
        if (self.raid_min_s as u128) < MINUTE_S || self.raid_min_s > self.raid_max_s || self.raid_step_s == 0 {
            return Err("a raid would last less than a minute, or its length has no step");
        }
        if self.stand_minutes == 0 || self.revival_bps as u128 >= BPS {
            return Err("a last stand of no minute, or a revival past the whole");
        }
        if self.staker_bonus_pm as u128 > PM || self.keep_bps as u128 > BPS || self.dump_min_bps as u128 > BPS {
            return Err("a bonus, his half or the dump tax past the whole");
        }
        if self.dump_min_s > self.dump_max_s || self.dump_step_s == 0 {
            return Err("the dump tax's length has no step, or its shortest is past its longest");
        }
        if grad_mcap(self)? < 2 {
            return Err("a curve sold out would have no market cap");
        }
        Ok(())
    }
}

impl Default for Rules {
    fn default() -> Self { Self::MEME_ROYALE }
}

/// A publicly visible commitment. The caller supplies the owner and house identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderKind { Buy, Sale }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub kind: OrderKind,
    pub amount: u128,
    pub minimum: u128,
    pub accepted_at: i64,
    pub lands_at: i64,
}

pub const ORDER_EARLY: &str = "the order is still marching";
pub const ORDERS_DUE: &str = "land the earlier orders first";
pub const ORDER_TIME: &str = "acceptance cannot precede the previous acceptance";
pub const MARCH_CAPACITY: usize = 8;
pub const MARCH_FULL: &str = "the march queue is full";
pub const ORDER_TOO_SMALL: &str = "the order is below the march minimum";
pub const OWNER_MARCH_FULL: &str = "the owner has reached the march cap";

/// Enforce the same per-owner cap in the program and the replay, scanning only pending orders.
pub fn check_march_owner<'a, Owner: PartialEq + ?Sized + 'a>(
    r: &Rules, owner: &Owner, pending: impl Iterator<Item = &'a Owner>,
) -> Result<(), &'static str> {
    if r.march_max_per_owner > 0 && pending.filter(|maker| *maker == owner).nth(r.march_max_per_owner as usize - 1).is_some() {
        return Err(OWNER_MARCH_FULL);
    }
    Ok(())
}

impl Order {
    /// Acceptance reserves the input, never a price. Equal stamps retain insertion order.
    pub fn accept(r: &Rules, kind: OrderKind, amount: u128, minimum: u128, now: i64, previous: Option<i64>) -> Result<Self, &'static str> {
        if amount == 0 { return Err(NOTHING_OUT); }
        if r.march_s > 0 && amount < r.march_min_order as u128 { return Err(ORDER_TOO_SMALL); }
        if previous.is_some_and(|p| now < p) { return Err(ORDER_TIME); }
        Ok(Self { kind, amount, minimum, accepted_at: now, lands_at: now.checked_add(r.march_s as i64).ok_or(OVERFLOW)? })
    }
}

/// A failed price limit or an unfillable curve returns the input, allowing the FIFO to progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing { Buy(Buy), Sale(Sale), Refund }

/// Evaluate against the curve at the scheduled landing instant, not the acceptance quote.
/// Callers must drain due orders before any later mutation of that curve. A decided raid
/// refunds outstanding commitments; neither its outcome nor its conversion can consume them.
pub fn land_on(r: &Rules, c: &Curve, order: &Order, now: i64, decided: bool, tax_bps: u32) -> Result<(Curve, Landing), &'static str> {
    if decided { return Ok((*c, Landing::Refund)); }
    if now < order.lands_at { return Err(ORDER_EARLY); }
    match order.kind {
        OrderKind::Buy => match buy_on(r, c, order.amount) {
            Ok((after, b)) if b.coins > 0 && b.coins >= order.minimum => Ok((after, Landing::Buy(b))),
            Ok(_) | Err(NOT_ENOUGH_TOKENS) | Err(NOTHING_OUT) => Ok((*c, Landing::Refund)),
            Err(e) => Err(e),
        },
        OrderKind::Sale => match sell_on(r, c, order.amount, tax_bps) {
            Ok((after, s)) if s.net() >= order.minimum => Ok((after, Landing::Sale(s))),
            Ok(_) | Err(NOTHING_OUT) | Err(RESERVE_SHORT) => Ok((*c, Landing::Refund)),
            Err(e) => Err(e),
        },
    }
}

/// the ceiling of `bps` basis points of `x`: pump.fun's fee, so no trade is too small to pay it
pub fn ceil_fee(x: u128, bps: u32) -> Result<u128, &'static str> {
    Ok(add(mul(x, bps as u128)?, BPS - 1)? / BPS)
}

/// A HOUSE'S CURVE. `rq` is the lamports it holds for its coins; a buy, a sale and a burn's payout move `vq` and `rq` by the same amount, so
/// `vq - rq` stays what it was born with (`vq0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Curve {
    pub vt: u128,
    pub vq: u128,
    pub rt: u128,
    pub rq: u128,
    pub rt0: u128,
}

impl Curve {
    /// a curve at its birth
    pub fn born(r: &Rules) -> Curve {
        Curve { vt: r.vt, vq: r.vq, rt: r.rt, rq: 0, rt0: r.rt }
    }
    /// the coins it has sold, burned ones among them (a burn gives no coin back to the curve)
    pub fn sold(&self) -> u128 {
        self.rt0 - self.rt
    }
    /// the virtual quote it was born with
    pub fn vq0(&self) -> u128 {
        self.vq - self.rq
    }
}

/// what a buy moves: the coins it gets, and the three parts of what it pays
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Buy {
    pub coins: u128,
    pub quote: u128,
    pub protocol: u128,
    pub creator: u128,
}
impl Buy {
    /// what the buyer pays: the quote into the curve and the two fees; never more than his spend, at most a lamport less
    pub fn paid(&self) -> u128 {
        self.quote + self.protocol + self.creator
    }
}

/// A BUY of `spend` lamports on the curve as it stands (pump.fun's; the model's `War._buy`): the raw input `spend * 10000 / (10000 + both
/// fees)`, each fee the ceiling of its share of it; when the three pass the spend the quote gives up the difference, so what is paid is never
/// more than the spend; the coins `(quote - 1) * vt / (vq + quote - 1)`, refused past the coins the curve has left (NOT_ENOUGH_TOKENS). A quote
/// of nothing buys nothing (the model's floor of a negative number would give coins back; no buy of three lamports or more reaches it). The
/// curve after it: vt and rt less the coins, vq and rq more the quote.
pub fn buy_on(r: &Rules, c: &Curve, spend: u128) -> Result<(Curve, Buy), &'static str> {
    let raw = mul(spend, BPS)? / (BPS + r.fees_bps());
    let protocol = ceil_fee(raw, r.protocol_bps)?;
    let creator = ceil_fee(raw, r.creator_bps)?;
    let total = add(add(raw, protocol)?, creator)?;
    let quote = if total > spend { raw.checked_sub(total - spend).ok_or(OVERFLOW)? } else { raw };
    let coins = if quote == 0 { 0 } else { mul(quote - 1, c.vt)? / add(c.vq, quote - 1)? };
    if coins > c.rt {
        return Err(NOT_ENOUGH_TOKENS);
    }
    let after = Curve { vt: c.vt - coins, vq: add(c.vq, quote)?, rt: c.rt - coins, rq: add(c.rq, quote)?, rt0: c.rt0 };
    Ok((after, Buy { coins, quote, protocol, creator }))
}

/// what a sale moves: its quote out of the curve, the two fees and the dump tax taken from it
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sale {
    pub quote: u128,
    pub protocol: u128,
    pub creator: u128,
    pub tax: u128,
}
impl Sale {
    /// what the seller gets: the quote less the fees and the tax
    pub fn net(&self) -> u128 {
        self.quote - self.protocol - self.creator - self.tax
    }
}

/// A SALE of `coins` (pump.fun's; the model's `War._sell`): the quote `coins * vq / (vt + coins)`, each fee the ceiling of its share of it, and
/// the dump tax, `tax_bps` of it floored (0 outside a dump tax: the model's `int(q * tax)`). The seller gets the quote less the three; refused
/// when that is nothing (NOTHING_OUT), when the quote is more than the house's reserve (RESERVE_SHORT: once coins were burned the curve prices
/// the coins left higher than the reserve can pay all of them back), or when it sells more coins than the curve ever sold (NOT_SOLD). The
/// curve after it: vt and rt more the coins, vq and rq less the quote.
pub fn sell_on(r: &Rules, c: &Curve, coins: u128, tax_bps: u32) -> Result<(Curve, Sale), &'static str> {
    if coins == 0 {
        return Err(NOTHING_OUT);
    }
    if add(c.rt, coins)? > c.rt0 {
        return Err(NOT_SOLD);
    }
    let quote = mul(coins, c.vq)? / add(c.vt, coins)?;
    let protocol = ceil_fee(quote, r.protocol_bps)?;
    let creator = ceil_fee(quote, r.creator_bps)?;
    let tax = mul(quote, tax_bps as u128)? / BPS;
    if quote <= add(add(protocol, creator)?, tax)? {
        return Err(NOTHING_OUT);
    }
    if quote > c.rq {
        return Err(RESERVE_SHORT);
    }
    let after = Curve { vt: add(c.vt, coins)?, vq: c.vq - quote, rt: c.rt + coins, rq: c.rq - quote, rt0: c.rt0 };
    Ok((after, Sale { quote, protocol, creator, tax }))
}

/// WHAT HE STILL HAS IN, after a sale (his rule: a deserter leaves with half of what he brought, the SOL he still has in the house). `brought`
/// is what a holder put into a house for his `coins` of it; selling `sold` of them takes their share of it out with them:
/// brought * (coins - sold) / coins, floored. So whoever buys much, sells it all and buys a little back has brought only the little, and his
/// half is half of that. Refused when he sells more than he holds (NOT_HELD); nothing held has brought nothing.
pub fn brought_after_sale(brought: u128, coins: u128, sold: u128) -> Result<u128, &'static str> {
    if sold > coins {
        return Err(NOT_HELD);
    }
    if coins == 0 {
        return Ok(0);
    }
    Ok(mul(brought, coins - sold)? / coins)
}

/// HIS BURN (his "Yes, burn them", Nov 24 2026): what one who deserts during a raid, or changes sides, is paid for his `coins` of a house, and its
/// curve after. Their worth is what selling all of them would give on the curve as it stands (a sale's quote less the two fees; the curve is not
/// moved by it); he is paid the least of their worth, `keep_bps` of what he `brought` into the house (his half) and the house's reserve. The
/// reserve and vq fall by what is paid; vt and rt stay as they are: the coins are burned, never sold back, so his leaving sinks his side only by
/// what he is paid. A burn ends what he brought into that house: it falls to 0 with his coins, so one who comes back in brings only what he
/// pays then, and a second desert is paid at most half of that.
pub fn burn_on(r: &Rules, c: &Curve, coins: u128, brought: u128) -> Result<(Curve, u128), &'static str> {
    let q = if coins == 0 { 0 } else { mul(coins, c.vq)? / add(c.vt, coins)? };
    let worth = q.saturating_sub(add(ceil_fee(q, r.protocol_bps)?, ceil_fee(q, r.creator_bps)?)?);
    let half = mul(brought, r.keep_bps as u128)? / BPS;
    let paid = worth.min(half).min(c.rq);
    Ok((Curve { vq: c.vq - paid, rq: c.rq - paid, ..*c }, paid))
}

/// THE MARKET CAP (Meme Royale's own measure, the price times the coins sold, on pump.fun's curve): vq * (rt0 - rt) / vt. Burned coins count
/// in it: a burn gives them back to nobody.
pub fn mcap(c: &Curve) -> Result<u128, &'static str> {
    Ok(mul(c.vq, c.sold())? / c.vt)
}

/// the market cap the curve would have holding `reserve` lamports: its vq would be vq0 + reserve and its vt the product over it, so
/// (vq0 + reserve) * reserve / vq0
pub fn mcap_at(c: &Curve, reserve: u128) -> Result<u128, &'static str> {
    let vq0 = c.vq0();
    if vq0 == 0 {
        return Err(OVERFLOW);
    }
    Ok(mul(add(vq0, reserve)?, reserve)? / vq0)
}

/// M_GRAD: the market cap of a curve born under `r` once every real coin is sold, `mcap_at` at the reserve where its real coins run out,
/// vq * rt / (vt - rt), taken exactly: vq * vt * rt / (vt - rt)^2, floored once. The product passes u128, so it is divided by parts:
/// vt * rt = q1 * d + r1 and vq * q1 = a1 * d + a2 give vq * vt * rt = a1 * d^2 + a2 * d + vq * r1.
pub fn grad_mcap(r: &Rules) -> Result<u128, &'static str> {
    let d = r.vt.checked_sub(r.rt).filter(|d| *d > 0).ok_or(OVERFLOW)?;
    let d2 = mul(d, d)?;
    let p = mul(r.vt, r.rt)?;
    let (q1, r1) = (p / d, p % d);
    let a = mul(r.vq, q1)?;
    let (a1, a2) = (a / d, a % d);
    add(a1, add(mul(a2, d)?, mul(r.vq, r1)?)? / d2)
}

/// A RAID'S LENGTH in the law's minutes (Meme Royale's `getRaidDurationSeconds`, as the read-back reads it): its shortest, and a step for each
/// share of twice a curve's real coins the two houses have sold between them: (raid_min + sold * (raid_max - raid_min) / (2 * rt) / step *
/// step) / 60, `sold` both houses' `rt0 - rt` when it opens
pub fn raid_minutes(r: &Rules, sold: u128) -> Result<u32, &'static str> {
    let steps = mul(sold, (r.raid_max_s - r.raid_min_s) as u128)? / mul(2, r.rt)? / r.raid_step_s as u128;
    let secs = add(r.raid_min_s as u128, mul(steps, r.raid_step_s as u128)?)?;
    u32::try_from(secs / MINUTE_S).map_err(|_| OVERFLOW)
}

/// a raid as it opens: both houses' market caps, the stake it needed, and its length in the law's minutes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opening {
    pub m0: [u128; 2],
    pub need: u128,
    pub minutes: u32,
}

/// THE STAKE'S NEED, in lamports: the larger of stake_own of the attacker's market cap and stake_gap of the gap between the two, per thousand,
/// floored
pub fn stake_need(r: &Rules, m0_att: u128, m0_def: u128) -> Result<u128, &'static str> {
    Ok(need_pm(r, m0_att, m0_def)? / PM)
}
fn need_pm(r: &Rules, m0_att: u128, m0_def: u128) -> Result<u128, &'static str> {
    Ok(mul(m0_att, r.stake_own_pm as u128)?.max(mul(m0_att.abs_diff(m0_def), r.stake_gap_pm as u128)?))
}

/// THE STAKE, WHOLE: `staked` coins of the attacker (every coin staked so far against this defender), as both curves stand. Refused when the
/// defender is no fair opponent (UNFAIR: its market cap not between opponent_min and opponent_max per thousand of the attacker's) or when the
/// stake's value, `staked * vq / vt`, is past stake_max per thousand of the attacker's market cap (STAKE_CAP), compared exactly:
/// staked * vq * 1000 <= m0_att * stake_max * vt. The raid opens when the value reaches the need, compared exactly:
/// staked * vq * 1000 >= max(m0_att * stake_own, |m0_att - m0_def| * stake_gap) * vt; it opens with both market caps, the need (`stake_need`)
/// and its length (`raid_minutes` of both houses' sold coins). None while it has not reached it.
pub fn stake_on(r: &Rules, att: &Curve, def: &Curve, staked: u128) -> Result<Option<Opening>, &'static str> {
    let (a, d) = (mcap(att)?, mcap(def)?);
    if mul(d, PM)? < mul(a, r.opponent_min_pm as u128)? || mul(d, PM)? > mul(a, r.opponent_max_pm as u128)? {
        return Err(UNFAIR);
    }
    let value = mul(mul(staked, att.vq)?, PM)?;
    if value > mul(mul(a, r.stake_max_pm as u128)?, att.vt)? {
        return Err(STAKE_CAP);
    }
    let need = need_pm(r, a, d)?;
    if value < mul(need, att.vt)? {
        return Ok(None);
    }
    Ok(Some(Opening { m0: [a, d], need: need / PM, minutes: raid_minutes(r, add(att.sold(), def.sold())?)? }))
}

/// the two sides of a raid: the attacker, whose holders staked, and the defender
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    Attacker,
    Defender,
}
impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Attacker => Side::Defender,
            Side::Defender => Side::Attacker,
        }
    }
    /// 0 for the attacker, 1 for the defender
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn of(i: u8) -> Option<Side> {
        match i {
            0 => Some(Side::Attacker),
            1 => Some(Side::Defender),
            _ => None,
        }
    }
}

/// A HOUSE AT A MINUTE OF ITS RAID: its market cap now; its line, the market cap it would have holding `pool` (both houses' reserves and both
/// chests: Meme Royale's `hurdleCriteria`); how far it climbed since the raid opened (`delta`, now - m0); and what it still has to climb to its
/// line (`gap`, line - now, at least 0)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    pub now: u128,
    pub line: u128,
    pub delta: i128,
    pub gap: u128,
}

/// a house's reading, from its curve, its market cap when the raid opened and the pool of the raid
pub fn reading(c: &Curve, m0: u128, pool: u128) -> Result<Reading, &'static str> {
    let now = mcap(c)?;
    let line = mcap_at(c, pool)?;
    let delta = i128::try_from(now).map_err(|_| OVERFLOW)? - i128::try_from(m0).map_err(|_| OVERFLOW)?;
    Ok(Reading { now, line, delta, gap: line.saturating_sub(now) })
}

/// THE LINE IS MET at minute `m` of a raid of `t` minutes when the climb is past what is left to climb times the share of the time left (the
/// line falls in a straight line to nothing at the raid's end): delta > 0 and delta * t > gap * (t - m), t - m at least 0
pub fn meets(rd: &Reading, t: u32, m: u32) -> Result<bool, &'static str> {
    if rd.delta <= 0 {
        return Ok(false);
    }
    Ok(mul(rd.delta as u128, t as u128)? > mul(rd.gap, t.saturating_sub(m) as u128)?)
}

/// A REVIVAL in the last stand: the climb past the line and a ninth more (the model's `req / (1 - 0.10)`): delta > 0 and
/// delta * t * (10000 - revival_bps) > gap * (t - m) * 10000
pub fn revives(r: &Rules, rd: &Reading, t: u32, m: u32) -> Result<bool, &'static str> {
    if rd.delta <= 0 {
        return Ok(false);
    }
    let left = mul(rd.gap, t.saturating_sub(m) as u128)?;
    Ok(mul(mul(rd.delta as u128, t as u128)?, BPS - r.revival_bps as u128)? > mul(left, BPS)?)
}

/// FURTHER ALONG: the attacker's climb over its own m0 against the defender's over its own, cross-multiplied (delta_att * m0_def against
/// delta_def * m0_att). The attacker wins only when it is Greater: a tie goes to the defender.
pub fn further(att: &Reading, m0_att: u128, def: &Reading, m0_def: u128) -> Result<core::cmp::Ordering, &'static str> {
    let (ma, md) = (i128::try_from(m0_att).map_err(|_| OVERFLOW)?, i128::try_from(m0_def).map_err(|_| OVERFLOW)?);
    let a = att.delta.checked_mul(md).ok_or(OVERFLOW)?;
    let d = def.delta.checked_mul(ma).ok_or(OVERFLOW)?;
    Ok(a.cmp(&d))
}

/// how a raid was decided
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    /// both beat their lines in the same minute: the further along won
    BothMet,
    /// the clock ran out: the further along won (`tie`: a tie, which goes to the defender)
    Clock { tie: bool },
    /// the winner beat its line at minute `met`, and the other's last stand failed
    StandFailed { met: u32 },
    /// the winner revived in its last stand
    Revived,
}

/// where a raid stands
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// open: neither side has beaten its line yet
    Open,
    /// `side` is in its last stand: the other beat its line at minute `met`, and the stand ends at minute `ends`
    Stand { side: Side, met: u32, ends: u32 },
    /// decided at minute `at`
    Decided { winner: Side, how: How, at: u32 },
}

/// THE LAST MINUTE A TRADE OF A RAID COUNTS IN: the raid's last minute while it is open, its stand's last minute during a stand; none once it is
/// decided. A trade at that minute counts in that minute's reading; after it nothing moves the houses until the minute is read, so a reading
/// taken late reads what that minute held.
pub fn trades_until(phase: Phase, t: u32) -> Option<u32> {
    match phase {
        Phase::Open => Some(t),
        Phase::Stand { ends, .. } => Some(ends),
        Phase::Decided { .. } => None,
    }
}

/// THE MINUTE, WHOLE (the crank's one call; the model's loop, one minute): the raid's phase after reading it at minute `m` of its `t`, from
/// both houses' curves (attacker first), their market caps when it opened and both chests. Open: one side meeting its line before the end
/// puts the other in its last stand, which ends `stand_minutes` later; both meeting it decides at once for the further along; at the end the
/// clock decides for the further along. In a stand: the side in it revives or, at its last minute, loses; a stand read after its last minute
/// is read at its last minute (no trade moves the houses after it: `trades_until`). Refused on a raid decided (DECIDED). The model reads
/// every minute; a minute nobody reads is simply not read.
pub fn minute_on(r: &Rules, phase: Phase, t: u32, m: u32, curves: [&Curve; 2], m0: [u128; 2], chests: u128) -> Result<Phase, &'static str> {
    let pool = add(add(curves[0].rq, curves[1].rq)?, chests)?;
    let rd = [reading(curves[0], m0[0], pool)?, reading(curves[1], m0[1], pool)?];
    let winner_further = || -> Result<(Side, bool), &'static str> {
        let o = further(&rd[0], m0[0], &rd[1], m0[1])?;
        Ok((if o == core::cmp::Ordering::Greater { Side::Attacker } else { Side::Defender }, o == core::cmp::Ordering::Equal))
    };
    match phase {
        Phase::Decided { .. } => Err(DECIDED),
        Phase::Stand { side, met, ends } => {
            let m = m.min(ends);
            if revives(r, &rd[side.index()], t, m)? {
                Ok(Phase::Decided { winner: side, how: How::Revived, at: m })
            } else if m >= ends {
                Ok(Phase::Decided { winner: side.other(), how: How::StandFailed { met }, at: m })
            } else {
                Ok(phase)
            }
        }
        Phase::Open => {
            let (ma, md) = (meets(&rd[0], t, m)?, meets(&rd[1], t, m)?);
            if m < t && ma != md {
                let side = if ma { Side::Defender } else { Side::Attacker };
                Ok(Phase::Stand { side, met: m, ends: m + r.stand_minutes })
            } else if m < t && ma && md {
                Ok(Phase::Decided { winner: winner_further()?.0, how: How::BothMet, at: m })
            } else if m >= t {
                let (winner, tie) = winner_further()?;
                Ok(Phase::Decided { winner, how: How::Clock { tie }, at: m.min(t) })
            } else {
                Ok(Phase::Open)
            }
        }
    }
}

/// the settlement's numbers: the loser's pot, what of it bought, the buy, and what went to the winner's chest
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settlement {
    pub pot_in: u128,
    pub bought_with: u128,
    pub buy: Buy,
    pub to_chest: u128,
}

/// the most of `pot` a buy on the curve can take before it runs past the coins the curve has left: the largest x at most `pot` whose raw input,
/// x * 10000 / (10000 + both fees), less one, buys at most rt (the model's binary search, with its own predicate)
pub fn bought_with(r: &Rules, c: &Curve, pot: u128) -> Result<u128, &'static str> {
    let fits = |x: u128| -> Result<bool, &'static str> {
        let raw = mul(x, BPS)? / (BPS + r.fees_bps());
        if raw == 0 {
            return Ok(true);
        }
        Ok(mul(raw - 1, c.vt)? / add(c.vq, raw - 1)? <= c.rt)
    };
    let (mut lo, mut hi) = (0u128, pot);
    while lo < hi {
        let mid = lo + (hi - lo + 1) / 2;
        if fits(mid)? {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    Ok(lo)
}

/// THE SETTLEMENT, WHOLE (Meme Royale's: the loser's pot buys the winner's coin on the winner's own curve): `pot_in`, the loser's reserve and
/// chest, buys with the most of it the winner's curve can take (`bought_with`), a buy like any other (`buy_on`, fees and all); the coins bought
/// go to the loser's holders (`share_of`). What the buy did not take goes to the winner's chest (`to_chest`): the rest of the pot and the lamport
/// a buy may leave of its spend (the model drops that lamport; no reference war leaves one). The winner's curve after it.
pub fn settle_on(r: &Rules, winner: &Curve, pot_in: u128) -> Result<(Curve, Settlement), &'static str> {
    let with = bought_with(r, winner, pot_in)?;
    let (after, buy) = if with == 0 { (*winner, Buy::default()) } else { buy_on(r, winner, with)? };
    Ok((after, Settlement { pot_in, bought_with: with, buy, to_chest: pot_in - buy.paid() }))
}

/// a loser holder's weight at the conversion: his coins a thousand each, and the coins he staked in this raid a thousand and the stakers'
/// bonus (the model's `coins * 11` for a staker, `coins * 10` for the others, scaled by a hundred: the same shares). `staked` is at most `coins`.
pub fn weight_of(r: &Rules, coins: u128, staked: u128) -> Result<u128, &'static str> {
    add(mul(coins.saturating_sub(staked), PM)?, mul(staked.min(coins), PM + r.staker_bonus_pm as u128)?)
}

/// the weights of all the loser's holders, fixed at the settlement: every coin it holds a thousand, and the stakers' bonus on the coins staked
/// in the raid (none when the loser is the defender, whose holders staked nothing in it)
pub fn weights_of(r: &Rules, held: u128, staked: u128) -> Result<u128, &'static str> {
    add(mul(held, PM)?, mul(staked, r.staker_bonus_pm as u128)?)
}

/// a loser holder's share of the coins bought: coins_bought * weight / weights, floored (none when nobody held the loser)
pub fn share_of(coins_bought: u128, weight: u128, weights: u128) -> Result<u128, &'static str> {
    if weights == 0 {
        return Ok(0);
    }
    Ok(mul(coins_bought, weight)? / weights)
}

/// what backs a holding of the winner once it lives on: its share, by the coins each holds, of the winner's reserve and chest
pub fn backed_of(reserve_and_chest: u128, holds: u128, all_held: u128) -> Result<u128, &'static str> {
    if all_held == 0 {
        return Ok(0);
    }
    Ok(mul(reserve_and_chest, holds)? / all_held)
}

/// a winner's dump tax: the rate it starts at, in basis points, and how long it lasts, in seconds
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DumpTax {
    pub rate_bps: u32,
    pub seconds: u64,
}

/// THE DUMP TAX of a winner whose market cap went from `m0` to `mcap_end` in its raid (Meme Royale's bytecode, the read-back's rule 9): none
/// when the raid added nothing; else it starts at the larger of dump_min_bps and the share of its market cap the raid added,
/// (mcap_end - m0) * 10000 / mcap_end, and lasts dump_min + floor(gain * (dump_max - dump_min) / (M_grad / 2) / step) * step seconds, M_grad
/// the market cap of a curve sold out (`grad_mcap`), with no cap
pub fn dump_tax_of(r: &Rules, m0: u128, mcap_end: u128) -> Result<Option<DumpTax>, &'static str> {
    if mcap_end <= m0 {
        return Ok(None);
    }
    let gain = mcap_end - m0;
    let rate = (mul(gain, BPS)? / mcap_end).max(r.dump_min_bps as u128);
    let half = grad_mcap(r)? / 2;
    let steps = mul(gain, (r.dump_max_s - r.dump_min_s) as u128)? / half / r.dump_step_s as u128;
    let seconds = add(r.dump_min_s as u128, mul(steps, r.dump_step_s as u128)?)?;
    Ok(Some(DumpTax { rate_bps: rate as u32, seconds: u64::try_from(seconds).map_err(|_| OVERFLOW)? }))
}

/// the dump tax's rate at `now`, in basis points: it falls in a straight line from `rate_bps` at `start` to nothing at `end`,
/// rate * (end - now) / (end - start), floored
pub fn dump_rate_at(rate_bps: u32, start: i64, end: i64, now: i64) -> u32 {
    if end <= start || now >= end {
        return 0;
    }
    if now <= start {
        return rate_bps;
    }
    ((rate_bps as i128 * (end - now) as i128) / (end - start) as i128) as u32
}

/// a tax split between the fee account and the house's chest: the fee account the floor of half, the chest the rest (as the model's chicken
/// tax splits it: `chest += tax - tax // 2`)
pub fn tax_split(tax: u128) -> (u128, u128) {
    (tax / 2, tax - tax / 2)
}

// THE RANGE, checked by the compiler: with every amount at most AMOUNT_MAX none of the law's products leaves u128.
const _: () = assert!(AMOUNT_MAX.checked_mul(AMOUNT_MAX).is_some()); // a sale's and a market cap's numerators
const _: () = assert!((AMOUNT_MAX * AMOUNT_MAX).checked_mul(PM * 8).is_some()); // a stake's value against its need
const _: () = assert!((AMOUNT_MAX * 4).checked_mul(AMOUNT_MAX * 4).is_some()); // a line: (vq0 + pool) * pool, the pool of both houses and both chests
