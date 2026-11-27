//! A raid's end: its settlement (the law's `settle_on`, `weights_of`, `dump_tax_of`), each loser holder's conversion (`weight_of`, `share_of`)
//! and the unlock of the winner's stakers.

use super::{minus, plus, position_of};
use crate::error::{of_law, WarsError};
use crate::state::{Config, Fees, House, Position, Raid, HOUSE_ACTIVE, HOUSE_REKT, RAID_DECIDED, RAID_SETTLED};
use crate::util::{close_to, expect_signer, expect_writable, move_lamports, now_ts, u64_of, Data};
use embertide_war::law::{dump_rate_at, dump_tax_of, mcap, settle_on, share_of, weight_of, weights_of, Phase, Side};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// data: none · accounts: [config, raid (w), attacker (w), defender (w), fees (w)]
/// Anyone, once the raid is decided (`NotDecided` before, `AlreadySettled` after): both market caps are read at the end; the loser's pot, its
/// reserve and its chest, buys the winner's coin on the winner's curve by the law's `settle_on`, the fees to the fee account and what the buy
/// did not take to the winner's chest; the weights of the loser's holders are fixed (`weights_of`: every coin it holds, and the raid's
/// stakers' bonus when the loser is the attacker); the loser is rekt, its reserve and chest nothing; the winner is out of the raid with its
/// dump tax from the clock (`dump_tax_of`), which replaces one still running only if it starts higher than that one stands now.
pub fn settle(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let (accounts, queue) = super::march::split(accounts, 5)?;
    let [config, raid, attacker, defender, fees] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(raid)?;
    expect_writable(attacker)?;
    expect_writable(defender)?;
    expect_writable(fees)?;
    let mut r = Raid::read_pda(raid)?;
    let c = Config::read_named(config, &r.config)?;
    super::march::guard(&c, raid.address(), queue.as_deref(), now_ts()?, true)?;
    let mut f = Fees::read_named(fees, &c.fees)?;
    let a = House::read_named(attacker, &r.attacker)?;
    let df = House::read_named(defender, &r.defender)?;
    match r.status {
        RAID_DECIDED => {}
        RAID_SETTLED => return Err(WarsError::AlreadySettled.into()),
        _ => return Err(WarsError::NotDecided.into()),
    }
    let Some(Phase::Decided { winner, .. }) = r.phase()? else { return Err(WarsError::NotDecided.into()) };
    let mcap_end = [mcap(&a.curve()).map_err(of_law)?, mcap(&df.curve()).map_err(of_law)?];
    let (mut wh, mut lh, wacc, lacc, m0_w) = match winner {
        Side::Attacker => (a, df, attacker, defender, r.m0_att),
        Side::Defender => (df, a, defender, attacker, r.m0_def),
    };
    let pot_in = lh.holds()?;
    let (after, s) = settle_on(&c.rules, &wh.curve(), pot_in as u128).map_err(of_law)?;
    let weights = weights_of(&c.rules, lh.held as u128, if winner == Side::Defender { r.staked as u128 } else { 0 }).map_err(of_law)?;
    // the loser's pot: the buy's quote, what it did not take and the buy's creator fee into the winner (that fee booked for the winner's
    // creator), the protocol's fee to the publisher's fee account; what the loser owes its own creator stays in it, his
    move_lamports(lacc, wacc, u64_of(s.buy.quote + s.to_chest + s.buy.creator)?)?;
    move_lamports(lacc, fees, u64_of(s.buy.protocol)?)?;
    wh.set_curve(&after)?;
    wh.chest = plus(wh.chest, s.to_chest)?;
    wh.creator_owed = plus(wh.creator_owed, s.buy.creator)?;
    wh.status = HOUSE_ACTIVE;
    wh.raid = Address::default();
    let now = now_ts()?;
    if let Some(t) = dump_tax_of(&c.rules, m0_w as u128, mcap_end[winner.index()]).map_err(of_law)? {
        if t.rate_bps > dump_rate_at(wh.dump_rate_bps, wh.dump_start, wh.dump_end, now) {
            wh.dump_rate_bps = t.rate_bps;
            wh.dump_start = now;
            wh.dump_end = now.checked_add(i64::try_from(t.seconds).map_err(|_| WarsError::MathOverflow)?).ok_or(WarsError::MathOverflow)?;
        }
    }
    lh.rq = 0;
    lh.chest = 0;
    lh.status = HOUSE_REKT;
    r.status = RAID_SETTLED;
    r.mcap_end_att = u64_of(mcap_end[0])?;
    r.mcap_end_def = u64_of(mcap_end[1])?;
    r.pot_in = pot_in;
    r.bought_with = u64_of(s.bought_with)?;
    r.coins_bought = u64_of(s.buy.coins)?;
    r.to_chest = u64_of(s.to_chest)?;
    r.weights = weights;
    f.book(s.buy.protocol, 0)?;
    wh.write(wacc)?;
    lh.write(lacc)?;
    f.write(fees)?;
    r.write(raid)
}

/// data: none · accounts: [owner (s,w), config, raid (w), loser (w), loser position (w), winner (w), winner position (w), system]
/// Each holder of the house that lost, once the raid is settled (`NotSettled` before): his coins of the loser become his share of the coins
/// the loser's pot bought (the law's `share_of` of his `weight_of`, over the weights fixed at the settlement: the coins he staked in this raid
/// a tenth better). His position in the winner gains the share (born if it does not stand, its rent his) and carries over what he brought;
/// his position in the loser is closed to him, so he converts once. Refused: a position not of the loser (`NotTheLoser`) or with no coin
/// (`NothingHeld`).
pub fn convert(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, config, raid, loser, loser_pos, winner, winner_pos, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(owner)?;
    expect_writable(owner)?;
    expect_writable(raid)?;
    expect_writable(loser)?;
    expect_writable(loser_pos)?;
    expect_writable(winner)?;
    expect_writable(winner_pos)?;
    let mut r = Raid::read_pda(raid)?;
    let c = Config::read_named(config, &r.config)?;
    if r.status != RAID_SETTLED {
        return Err(WarsError::NotSettled.into());
    }
    let won = Side::of(r.winner).ok_or(WarsError::InvalidAccount)?;
    if loser.address() != r.house_of(won.other()) {
        return Err(WarsError::NotTheLoser.into());
    }
    let mut lh = House::read(loser)?;
    let mut wh = House::read_named(winner, r.house_of(won))?;
    let lp = Position::read_pda(loser_pos)?;
    if &lp.house != loser.address() {
        return Err(WarsError::NotTheLoser.into());
    }
    if &lp.owner != owner.address() {
        return Err(WarsError::Unauthorized.into());
    }
    if lp.coins == 0 {
        return Err(WarsError::NothingHeld.into());
    }
    let staked_here = if &lp.raid == raid.address() { lp.staked } else { 0 };
    let weight = weight_of(&c.rules, lp.coins as u128, staked_here as u128).map_err(of_law)?;
    let share = share_of(r.coins_bought as u128, weight, r.weights).map_err(of_law)?;
    let mut wp = position_of(owner, winner, winner_pos)?;
    wp.coins = plus(wp.coins, share)?;
    wp.brought = wp.brought.checked_add(lp.brought).ok_or(WarsError::MathOverflow)?;
    lh.held = minus(lh.held, lp.coins as u128)?;
    wh.held = plus(wh.held, share)?;
    r.converted = plus(r.converted, share)?;
    wp.write(winner_pos)?;
    lh.write(loser)?;
    wh.write(winner)?;
    r.write(raid)?;
    close_to(loser_pos, owner)
}

/// data: none · accounts: [owner (s), raid, position (w)]
/// A staker of the attacker that won takes his coins back from the raid once it is settled (`NotSettled` before): his position's stake is
/// lifted. When the attacker lost its stakers convert instead (`NotTheWinner`); a position with no stake in this raid is `NotStaked`.
pub fn unlock(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, raid, position] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(owner)?;
    expect_writable(position)?;
    let r = Raid::read_pda(raid)?;
    if r.status != RAID_SETTLED {
        return Err(WarsError::NotSettled.into());
    }
    if r.winner != Side::Attacker.index() as u8 {
        return Err(WarsError::NotTheWinner.into());
    }
    let attacker_house = r.attacker;
    let mut p = Position::read_pda(position)?;
    if &p.owner != owner.address() {
        return Err(WarsError::Unauthorized.into());
    }
    if p.house != attacker_house {
        return Err(WarsError::NotTheWinner.into());
    }
    if &p.raid != raid.address() || p.staked == 0 {
        return Err(WarsError::NotStaked.into());
    }
    p.staked = 0;
    p.raid = Address::default();
    p.write(position)
}

