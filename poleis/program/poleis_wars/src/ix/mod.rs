//! The instructions: a generation's config and its houses (config.rs); a buy and a sale (trade.rs); a raid from its first stake to its
//! decision (raid.rs); its settlement, the conversion of the loser's holders and the unlock of the winner's stakers (settle.rs); a house's
//! creator paid and the publisher paid (pay.rs).

pub mod config;
pub mod pay;
pub mod raid;
pub mod settle;
pub mod trade;
pub mod march;

use crate::error::WarsError;
use crate::state::{House, Position, Raid, SEED_POSITION};
use crate::util::{create_pda, find_and_expect_pda, stands};
use poleis_war::law::trades_until;
use pinocchio::{error::ProgramError, AccountView, Address};

/// The raid a house is in, at `now`, if a trade of it still counts: the account at the address the house names, open or in its stand, and
/// the clock no later than the last minute a trade counts in (the law's `trades_until`). A house in no raid is NotInRaid, a raid past it
/// RaidOver.
pub(crate) fn raid_window(raid: &AccountView, h: &House, now: i64) -> Result<Raid, ProgramError> {
    if raid.address() != &h.raid || h.raid == Address::default() {
        return Err(WarsError::NotInRaid.into());
    }
    let r = Raid::read(raid)?;
    let phase = r.phase()?.ok_or(WarsError::NotInRaid)?;
    match trades_until(phase, r.minutes) {
        Some(last) if r.minute_at(now) <= last => Ok(r),
        _ => Err(WarsError::RaidOver.into()),
    }
}

/// An owner's position in a house: the one that stands (checked: of this house and this owner), or one born empty, its rent paid by the
/// owner, who signs.
pub(crate) fn position_of(owner: &AccountView, house: &AccountView, position: &AccountView) -> Result<Position, ProgramError> {
    if stands(position) {
        let p = Position::read_pda(position)?;
        if &p.house != house.address() {
            return Err(WarsError::InvalidAccount.into());
        }
        if &p.owner != owner.address() {
            return Err(WarsError::Unauthorized.into());
        }
        return Ok(p);
    }
    let seeds: &[&[u8]] = &[SEED_POSITION, house.address().as_ref(), owner.address().as_ref()];
    let bump = find_and_expect_pda(position, seeds)?;
    create_pda(owner, position, Position::SIZE, seeds, bump)?;
    Ok(Position { house: *house.address(), owner: *owner.address(), coins: 0, staked: 0, brought: 0, raid: Address::default(), bump })
}

/// A position that must stand: of this house, its owner the signer.
pub(crate) fn position_standing(owner: &AccountView, house: &AccountView, position: &AccountView) -> Result<Position, ProgramError> {
    let p = Position::read_pda(position)?;
    if &p.house != house.address() {
        return Err(WarsError::InvalidAccount.into());
    }
    if &p.owner != owner.address() {
        return Err(WarsError::Unauthorized.into());
    }
    Ok(p)
}

/// a stored amount moved by a law amount, checked both ways
pub(crate) fn plus(a: u64, b: u128) -> Result<u64, ProgramError> {
    a.checked_add(crate::util::u64_of(b)?).ok_or(WarsError::MathOverflow.into())
}
pub(crate) fn minus(a: u64, b: u128) -> Result<u64, ProgramError> {
    a.checked_sub(crate::util::u64_of(b)?).ok_or(WarsError::MathOverflow.into())
}
