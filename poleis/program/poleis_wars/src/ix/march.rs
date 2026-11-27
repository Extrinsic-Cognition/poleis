//! The order marches. One public FIFO per raid, quote escrowed here, sale coins
//! removed from their positions. No matching service: anyone supplies the due
//! owners and positions in FIFO order and the program lands all of them atomically.
use super::{minus, plus, position_of, position_standing, raid_window};
use crate::{
    error::{of_law, WarsError},
    state::*,
    util::*,
};
use embertide_war::law::{brought_after_sale, check_march_owner, land_on, trades_until, Landing, Order, OrderKind, Rules};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// Existing instructions take one trailing queue account only for marching configs.
pub(crate) fn split(accounts: &mut [AccountView], n: usize) -> Result<(&mut [AccountView], Option<&mut AccountView>), ProgramError> {
    if accounts.len() < n || accounts.len() > n + 1 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (base, extra) = accounts.split_at_mut(n);
    Ok((base, extra.first_mut()))
}

fn read_or_empty(queue: &AccountView, raid: &Address) -> Result<March, ProgramError> {
    if stands(queue) {
        March::read(queue, raid)
    } else {
        Ok(March::empty(*raid, find_and_expect_pda(queue, &[SEED_MARCH, raid.as_ref()])?))
    }
}

/// No later curve mutation or decision can overtake an order already due.
pub(crate) fn guard(c: &Config, raid: &Address, queue: Option<&AccountView>, now: i64, empty: bool) -> ProgramResult {
    if c.rules.march_s == 0 {
        if queue.is_some() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        return Ok(());
    }
    let q = read_or_empty(queue.ok_or(ProgramError::NotEnoughAccountKeys)?, raid)?;
    if q.count > 0 && (empty || q.orders[0].order.lands_at <= now) {
        return Err(WarsError::OrdersDue.into());
    }
    Ok(())
}

pub(crate) fn queue_for(payer: &AccountView, queue: &mut AccountView, raid: &Address, rules: &Rules) -> Result<March, ProgramError> {
    expect_writable(queue)?;
    let q = read_or_empty(queue, raid)?;
    check_march_owner(rules, payer.address(), q.orders[..q.count].iter().map(|p| &p.owner)).map_err(of_law)?;
    if !stands(queue) {
        create_pda(payer, queue, March::SIZE, &[SEED_MARCH, raid.as_ref()], q.bump)?;
    }
    Ok(q)
}

fn side_of(r: &Raid, house: &Address) -> Result<u8, ProgramError> {
    if house == &r.attacker {
        Ok(0)
    } else if house == &r.defender {
        Ok(1)
    } else {
        Err(WarsError::NotInRaid.into())
    }
}

/// Buy/sell bytes unchanged; marching accounts:
/// [owner(s,w), config, house(w), position(w), fees, system, raid, queue(w)].
pub fn accept(accounts: &mut [AccountView], data: &[u8], kind: OrderKind) -> ProgramResult {
    let [owner, config, house, position, fees, _system, raid, queue] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(16)?;
    let (amount, minimum) = (Data(data).u64(0)?, Data(data).u64(8)?);
    expect_signer(owner)?;
    for a in [&*owner, &*house, &*position, &*queue] {
        expect_writable(a)?;
    }
    let mut h = House::read_pda(house)?;
    let c = Config::read_named(config, &h.config)?;
    if c.rules.march_s == 0 {
        return Err(WarsError::MarchDisabled.into());
    }
    Fees::read_named(fees, &c.fees)?;
    if h.status != HOUSE_IN_RAID {
        return Err(WarsError::NotInRaid.into());
    }
    let now = now_ts()?;
    let r = raid_window(raid, &h, now)?;
    let side = side_of(&r, house.address())?;
    let mut q = queue_for(owner, queue, raid.address(), &c.rules)?;
    let order = Order::accept(&c.rules, kind, amount as u128, minimum as u128, now, Some(q.last_acceptance)).map_err(of_law)?;
    let mut p = match kind {
        OrderKind::Buy => position_of(owner, house, position)?,
        OrderKind::Sale => position_standing(owner, house, position)?,
    };
    let mut basis = 0;
    match kind {
        OrderKind::Buy => pay_in(owner, queue, amount)?,
        OrderKind::Sale => {
            if amount > p.free() {
                return Err(WarsError::NotEnoughCoins.into());
            }
            let left = u64_of(brought_after_sale(p.brought as u128, p.coins as u128, amount as u128).map_err(of_law)?)?;
            basis = p.brought - left;
            p.brought = left;
            p.coins -= amount;
            h.held = minus(h.held, amount as u128)?;
        }
    }
    q.push(Pending {
        owner: *owner.address(),
        side,
        order,
        brought: basis,
        sequence: 0,
    })?;
    p.write(position)?;
    h.write(house)?;
    q.write(queue)
}

/// data: none. [config, raid, attacker(w), defender(w), fees(w), queue(w),
/// (owner, position(w))*]. Exactly every due order, or every pending order
/// after decision. No signer; destinations are the ones escrow recorded.
/// Quote payouts accrue above the position's rent, claimable by its owner. A
/// drained or reassigned wallet cannot block the FIFO by failing a rent check.
pub fn land(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    Data(data).expect_len(0)?;
    if accounts.len() < 6 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (base, pairs) = accounts.split_at_mut(6);
    let [config, raid, attacker, defender, fees, queue] = base else {
        unreachable!()
    };
    let r = Raid::read_pda(raid)?;
    let c = Config::read_named(config, &r.config)?;
    if c.rules.march_s == 0 {
        return Err(WarsError::MarchDisabled.into());
    }
    for a in [&*attacker, &*defender, &*fees, &*queue] {
        expect_writable(a)?;
    }
    let mut a = House::read_named(attacker, &r.attacker)?;
    let mut d = House::read_named(defender, &r.defender)?;
    let mut f = Fees::read_named(fees, &c.fees)?;
    let mut q = March::read(queue, raid.address())?;
    let now = now_ts()?;
    let decided = r.status == RAID_DECIDED;
    if r.status == RAID_SETTLED || r.status == RAID_PROPOSED {
        return Err(WarsError::NotOpen.into());
    }
    let n = q.orders[..q.count].iter().take_while(|p| decided || p.order.lands_at <= now).count();
    if pairs.len() != n * 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    for pair in pairs.chunks_exact_mut(2) {
        let [owner, position] = pair else { unreachable!() };
        expect_writable(position)?;
        let pending = q.orders[0];
        if owner.address() != &pending.owner {
            return Err(WarsError::Unauthorized.into());
        }
        let (h, house) = if pending.side == 0 {
            (&mut a, &mut *attacker)
        } else {
            (&mut d, &mut *defender)
        };
        let mut p = position_standing(owner, house, position)?;
        let expired = r
            .phase()?
            .and_then(|phase| trades_until(phase, r.minutes))
            .is_some_and(|last| r.minute_at(pending.order.lands_at) > last);
        // A raid sale has no post-settlement dump tax. Orders do not survive settlement.
        let (after, outcome) = land_on(&c.rules, &h.curve(), &pending.order, now, decided || expired, 0).map_err(of_law)?;
        match outcome {
            Landing::Buy(b) => {
                move_lamports(queue, house, u64_of(b.quote + b.creator)?)?;
                move_lamports(queue, fees, u64_of(b.protocol)?)?;
                move_lamports(queue, position, u64_of(pending.order.amount - b.paid())?)?;
                p.coins = plus(p.coins, b.coins)?;
                p.brought = plus(p.brought, b.paid())?;
                h.held = plus(h.held, b.coins)?;
                h.creator_owed = plus(h.creator_owed, b.creator)?;
                f.book(b.protocol, 0)?;
            }
            Landing::Sale(s) => {
                move_lamports(house, position, u64_of(s.net())?)?;
                move_lamports(house, fees, u64_of(s.protocol)?)?;
                h.creator_owed = plus(h.creator_owed, s.creator)?;
                f.book(s.protocol, 0)?;
            }
            Landing::Refund => match pending.order.kind {
                OrderKind::Buy => move_lamports(queue, position, u64_of(pending.order.amount)?)?,
                OrderKind::Sale => {
                    p.coins = plus(p.coins, pending.order.amount)?;
                    p.brought = plus(p.brought, pending.brought as u128)?;
                    h.held = plus(h.held, pending.order.amount)?;
                }
            },
        }
        h.set_curve(&after)?;
        p.write(position)?;
        q.pop();
    }
    a.write(attacker)?;
    d.write(defender)?;
    f.write(fees)?;
    q.write(queue)
}

/// [owner(s), position(w), recipient(w)], no data. Quote above this position's
/// rent belongs to its owner (landing proceeds, refunds, change or donations).
/// The owner chooses a recipient: a bad recipient can only fail this claim,
/// never another player's landing. Conversion already closes a position to its
/// owner and therefore returns any unclaimed credit with its rent.
pub fn claim(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, position, recipient] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(owner)?;
    expect_writable(position)?;
    expect_writable(recipient)?;
    let p = Position::read_pda(position)?;
    if &p.owner != owner.address() {
        return Err(WarsError::Unauthorized.into());
    }
    let quote = position.lamports().checked_sub(rent_of(Position::SIZE)?).ok_or(WarsError::MathOverflow)?;
    if quote == 0 {
        return Err(WarsError::NothingOwed.into());
    }
    move_lamports(position, recipient, quote)
}

/// [owner(s,w), config, house, position(w), (raid, queue) while the house is in a raid].
/// Close an empty position in a marching generation and return its rent to its owner.
pub fn close_position(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, config, house, position, raid_accounts @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(owner)?;
    expect_writable(owner)?;
    expect_writable(position)?;
    let h = House::read_pda(house)?;
    let c = Config::read_named(config, &h.config)?;
    if c.rules.march_s == 0 {
        return Err(WarsError::MarchDisabled.into());
    }
    let p = position_standing(owner, house, position)?;
    if p.staked != 0 {
        return Err(WarsError::Staked.into());
    }
    if p.coins != 0 {
        return Err(WarsError::PositionHasCoins.into());
    }
    if position.lamports() > rent_of(Position::SIZE)? {
        return Err(WarsError::PositionHasQuote.into());
    }
    match (h.status, raid_accounts) {
        (HOUSE_IN_RAID, [raid, queue]) => {
            if raid.address() != &h.raid {
                return Err(WarsError::NotInRaid.into());
            }
            let r = Raid::read(raid)?;
            let side = side_of(&r, house.address())?;
            let q = read_or_empty(queue, raid.address())?;
            if q.orders[..q.count].iter().any(|order| order.side == side && order.owner == p.owner) {
                return Err(WarsError::PositionOrderPending.into());
            }
        }
        (HOUSE_ACTIVE | HOUSE_REKT, []) => {} // Settlement requires an empty queue before the house changes status.
        _ => return Err(ProgramError::NotEnoughAccountKeys),
    }
    close_to(position, owner)
}
