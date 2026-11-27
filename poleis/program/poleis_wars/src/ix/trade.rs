//! A buy and a sale on a house's curve: pump.fun's, by the law's `buy_on` and `sell_on`.

use super::{minus, plus, position_of, position_standing, raid_window};
use crate::error::{of_law, WarsError};
use crate::state::{Config, Fees, House, HOUSE_ACTIVE, HOUSE_IN_RAID, HOUSE_REKT};
use crate::util::{expect_signer, expect_writable, move_lamports, now_ts, pay_in, u64_of, Data};
use embertide_war::law::{brought_after_sale, buy_on, dump_rate_at, sell_on, tax_split};
use pinocchio::{error::ProgramError, AccountView, ProgramResult};

/// data: [spend:8][min_coins:8] · accounts: [buyer (s,w), config, house (w), position (w), fees (w), system, raid (only for a house in a raid)]
/// A buy of `spend` lamports (the law's `buy_on`): the buyer pays the quote and the creator's fee into the house (the fee booked for its
/// creator) and the protocol's fee into the generation's fee account, never more than his spend; his position (born if it does not stand, its rent his) gains the coins and `brought` what he paid. Refused on a house that
/// lost (`Rekt`); on a house in a raid only while a trade of the raid counts (`raid_window`: the raid named, else `NotInRaid`, `RaidOver`):
/// a buy is how a side climbs its line, and how the reserve of a last stand comes in. A buy of no coin is `NothingOut`, one under its least
/// `PastMin`.
pub fn buy(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() == 8 { return super::march::accept(accounts, data, embertide_war::law::OrderKind::Buy); }
    let (buyer, config, house, position, fees, raid) = match accounts {
        [buyer, config, house, position, fees, _system] => (buyer, config, house, position, fees, None),
        [buyer, config, house, position, fees, _system, raid] => (buyer, config, house, position, fees, Some(raid)),
        _ => return Err(ProgramError::NotEnoughAccountKeys),
    };
    let d = Data(data);
    d.expect_len(16)?;
    let (spend, min_coins) = (d.u64(0)?, d.u64(8)?);
    expect_signer(buyer)?;
    expect_writable(buyer)?;
    expect_writable(house)?;
    expect_writable(position)?;
    expect_writable(fees)?;
    let mut h = House::read_pda(house)?;
    let c = Config::read_named(config, &h.config)?;
    let mut f = Fees::read_named(fees, &c.fees)?;
    if spend == 0 {
        return Err(WarsError::InvalidAmount.into());
    }
    match (h.status, raid) {
        (HOUSE_REKT, _) => return Err(WarsError::Rekt.into()),
        (HOUSE_IN_RAID, Some(r)) => {
            raid_window(r, &h, now_ts()?)?;
            if c.rules.march_s > 0 { return Err(ProgramError::NotEnoughAccountKeys); }
        }
        (HOUSE_IN_RAID, None) => return Err(ProgramError::NotEnoughAccountKeys),
        (HOUSE_ACTIVE, None) => {}
        _ => return Err(WarsError::NotInRaid.into()),
    }
    let (after, b) = buy_on(&c.rules, &h.curve(), spend as u128).map_err(of_law)?;
    if b.coins == 0 {
        return Err(WarsError::NothingOut.into());
    }
    if b.coins < min_coins as u128 {
        return Err(WarsError::PastMin.into());
    }
    let mut p = position_of(buyer, house, position)?;
    pay_in(buyer, house, u64_of(b.paid())?)?;
    // the protocol's fee to the publisher's fee account; the creator's stays in the house, his
    move_lamports(house, fees, u64_of(b.protocol)?)?;
    h.set_curve(&after)?;
    h.held = plus(h.held, b.coins)?;
    h.creator_owed = plus(h.creator_owed, b.creator)?;
    p.coins = plus(p.coins, b.coins)?;
    p.brought = plus(p.brought, b.paid())?;
    f.book(b.protocol, 0)?;
    p.write(position)?;
    h.write(house)?;
    f.write(fees)
}

/// data: [coins:8][min_out:8] · accounts: [seller (s,w), config, house (w), position (w), fees (w)]
/// A sale of `coins` the position holds free of a stake (the law's `sell_on`), outside a raid (`InRaid`: a raid's ways out are `desert` and
/// `change_sides`), on a house that did not lose (`Rekt`). Under the house's dump tax (its rate at the clock: the law's `dump_rate_at`) the
/// sale also pays the tax, half to the fee account and half into the house's chest (the law's `tax_split`); the protocol's fee goes to the
/// fee account and the creator's stays in the house, booked for its creator. The seller gets the quote less
/// the fees and the tax; refused under its least (`PastMin`), past the house's reserve (`ReserveShort`) or for nothing (`NothingOut`). What
/// he brought into the house falls with what he sold (`brought_after_sale`): his half, should he desert later, is half of what he still has in.
pub fn sell(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() == 8 { return super::march::accept(accounts, data, embertide_war::law::OrderKind::Sale); }
    let [seller, config, house, position, fees] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(16)?;
    let (coins, min_out) = (d.u64(0)?, d.u64(8)?);
    expect_signer(seller)?;
    expect_writable(seller)?;
    expect_writable(house)?;
    expect_writable(position)?;
    expect_writable(fees)?;
    let mut h = House::read_pda(house)?;
    let c = Config::read_named(config, &h.config)?;
    let mut f = Fees::read_named(fees, &c.fees)?;
    match h.status {
        HOUSE_REKT => return Err(WarsError::Rekt.into()),
        HOUSE_IN_RAID => return Err(WarsError::InRaid.into()),
        _ => {}
    }
    if coins == 0 {
        return Err(WarsError::InvalidAmount.into());
    }
    let mut p = position_standing(seller, house, position)?;
    if coins > p.free() {
        return Err(WarsError::NotEnoughCoins.into());
    }
    let rate = dump_rate_at(h.dump_rate_bps, h.dump_start, h.dump_end, now_ts()?);
    let (after, s) = sell_on(&c.rules, &h.curve(), coins as u128, rate).map_err(of_law)?;
    if s.net() < min_out as u128 {
        return Err(WarsError::PastMin.into());
    }
    let (tax_fees, tax_chest) = tax_split(s.tax);
    move_lamports(house, seller, u64_of(s.net())?)?;
    // the protocol's fee and the tax's fee half to the publisher's fee account; the creator's fee stays in the house, his
    move_lamports(house, fees, u64_of(s.protocol + tax_fees)?)?;
    h.set_curve(&after)?;
    h.chest = plus(h.chest, tax_chest)?;
    h.creator_owed = plus(h.creator_owed, s.creator)?;
    h.held = minus(h.held, coins as u128)?;
    // what he still has in falls with what he sold (the law's `brought_after_sale`): his half is half of what stays in
    p.brought = u64_of(brought_after_sale(p.brought as u128, p.coins as u128, coins as u128).map_err(of_law)?)?;
    p.coins -= coins;
    f.book(s.protocol, tax_fees)?;
    p.write(position)?;
    h.write(house)?;
    f.write(fees)
}
