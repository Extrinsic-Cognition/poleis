//! A raid from its first stake to its decision: the stake that opens it (the law's `stake_on`) or is taken back while it is a proposal, his
//! burn for whoever deserts or changes sides (`burn_on`, and `buy_on` for the house he goes to), and the crank that reads its minutes
//! (`minute_on`).

use super::{minus, plus, position_of, position_standing, raid_window};
use crate::error::{of_law, WarsError};
use crate::state::{Config, Fees, House, Position, Raid, HOUSE_ACTIVE, HOUSE_IN_RAID, HOUSE_REKT, RAID_ACTIVE, RAID_PROPOSED, SEED_RAID};
use crate::util::{create_pda, expect_signer, expect_writable, find_and_expect_pda, move_lamports, now_ts, stands, u64_of, Data};
use poleis_war::law::{buy_on, burn_on, mcap, minute_on, stake_on, Phase};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// a house that can raid or be raided: not lost, not in a raid (one battle at a time)
fn free_to_fight(h: &House) -> Result<(), ProgramError> {
    match h.status {
        HOUSE_ACTIVE => Ok(()),
        HOUSE_REKT => Err(WarsError::Rekt.into()),
        _ => Err(WarsError::InRaid.into()),
    }
}

/// data: [coins:8] · accounts: [staker (s,w), config, attacker (w), defender (w), raid (w), position (w), system]
/// A holder of the attacker stakes `coins` of his free coins against the defender, into the raid ["raid", attacker, defender]: born a
/// proposal with the first stake (its rent the staker's), and gathering stakes until the law's `stake_on` opens it. Refused, in this order:
/// the same house twice (`SameHouse`); houses of two configs (`OtherConfig`); a raid no longer a proposal (`RaidOpen`); a house that lost
/// (`Rekt`) or is in a raid (`InRaid`); an amount of nothing or past his free coins; a defender under its shield (younger than the config's `shield_age_s`, or worth less than its `shield_mcap`:
/// `Shielded`); a position whose stake is in another raid, or an attacker proposing another raid
/// (`OtherRaid`); and the law's refusals, an opponent not between half and twice the attacker's market cap (`Unfair`) and a stake past the
/// most that may be staked (`StakeCap`). When the stake reaches the need the raid opens at the clock: both market caps, the need and its
/// length in the law's minutes are the law's (`Opening`); its seconds are its minutes times the config's `minute_seconds`; both houses are
/// then in it.
pub fn stake(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [staker, config, attacker, defender, raid, position, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let coins = d.u64(0)?;
    expect_signer(staker)?;
    expect_writable(staker)?;
    expect_writable(attacker)?;
    expect_writable(defender)?;
    expect_writable(raid)?;
    expect_writable(position)?;
    if attacker.address() == defender.address() {
        return Err(WarsError::SameHouse.into());
    }
    let mut a = House::read_pda(attacker)?;
    let mut df = House::read_pda(defender)?;
    if a.config != df.config {
        return Err(WarsError::OtherConfig.into());
    }
    let c = Config::read_named(config, &a.config)?;
    // a raid that stands must be this pair's proposal: once open, its stake is closed
    let standing = if stands(raid) {
        let r = Raid::read_pda(raid)?;
        if &r.attacker != attacker.address() || &r.defender != defender.address() {
            return Err(WarsError::InvalidAccount.into());
        }
        if r.status != RAID_PROPOSED {
            return Err(WarsError::RaidOpen.into());
        }
        Some(r)
    } else {
        None
    };
    free_to_fight(&a)?;
    free_to_fight(&df)?;
    if coins == 0 {
        return Err(WarsError::InvalidAmount.into());
    }
    let mut p = position_standing(staker, attacker, position)?;
    if coins > p.free() {
        return Err(WarsError::NotEnoughCoins.into());
    }
    let now = now_ts()?;
    if (now.saturating_sub(df.born) as i128) < c.shield_age_s as i128 || mcap(&df.curve()).map_err(of_law)? < c.shield_mcap as u128 {
        return Err(WarsError::Shielded.into());
    }
    if (a.proposal != Address::default() && &a.proposal != raid.address()) || (p.staked > 0 && &p.raid != raid.address()) {
        return Err(WarsError::OtherRaid.into());
    }
    let staked = plus(standing.as_ref().map(|r| r.staked).unwrap_or(0), coins as u128)?;
    let opening = stake_on(&c.rules, &a.curve(), &df.curve(), staked as u128).map_err(of_law)?;
    let seeds: &[&[u8]] = &[SEED_RAID, attacker.address().as_ref(), defender.address().as_ref()];
    let mut r = if let Some(r) = standing {
        r
    } else {
        let bump = find_and_expect_pda(raid, seeds)?;
        create_pda(staker, raid, Raid::SIZE, seeds, bump)?;
        Raid {
            config: a.config,
            attacker: *attacker.address(),
            defender: *defender.address(),
            status: RAID_PROPOSED,
            staked: 0,
            stakers: 0,
            m0_att: 0,
            m0_def: 0,
            need: 0,
            start: 0,
            minutes: 0,
            seconds: 0,
            minute_seconds: c.minute_seconds,
            next_minute: 0,
            stand_side: 0,
            stand_met: 0,
            stand_ends: 0,
            winner: 0,
            how: 0,
            how_met: 0,
            decided_at: 0,
            mcap_end_att: 0,
            mcap_end_def: 0,
            pot_in: 0,
            bought_with: 0,
            coins_bought: 0,
            to_chest: 0,
            weights: 0,
            converted: 0,
            bump,
        }
    };
    if p.staked == 0 {
        r.stakers = r.stakers.checked_add(1).ok_or(WarsError::MathOverflow)?;
    }
    p.staked += coins;
    p.raid = *raid.address();
    r.staked = staked;
    a.proposal = *raid.address();
    if let Some(o) = opening {
        r.status = RAID_ACTIVE;
        r.m0_att = u64_of(o.m0[0])?;
        r.m0_def = u64_of(o.m0[1])?;
        r.need = u64_of(o.need)?;
        r.start = now;
        r.minutes = o.minutes;
        r.minute_seconds = c.minute_seconds;
        r.seconds = (o.minutes as u64).checked_mul(c.minute_seconds as u64).ok_or(WarsError::MathOverflow)?;
        r.next_minute = 0;
        a.status = HOUSE_IN_RAID;
        a.raid = *raid.address();
        a.proposal = Address::default();
        df.status = HOUSE_IN_RAID;
        df.raid = *raid.address();
    }
    p.write(position)?;
    r.write(raid)?;
    a.write(attacker)?;
    df.write(defender)
}

/// data: [coins:8] · accounts: [staker (s), attacker (w), raid (w), position (w)]
/// A staker takes `coins` of his stake back from a raid that is still a proposal, one that never opened (Meme Royale's
/// withdrawProposalStake): his position's stake and the raid's fall by them; a position whose stake is all taken back leaves the raid, and a
/// proposal whose stake is all taken back leaves the attacker free to propose another (the proposal stands, empty, for a later stake). No
/// law moves: a stake is a lock and nothing else until the raid opens. Refused for every other phase of the raid (`RaidOpen`: open, in a
/// stand, decided or settled, a stake is the raid's to its end; the winner's stakers `unlock`, the loser's convert), for nothing
/// (`InvalidAmount`), for a position that staked nothing in this raid (`NotStaked`) or less than `coins` (`NotEnoughCoins`), for another's
/// position (`Unauthorized`), and for a house or a position that is not the raid's attacker's (`InvalidAccount`).
pub fn unstake(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [staker, attacker, raid, position] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let coins = d.u64(0)?;
    expect_signer(staker)?;
    expect_writable(attacker)?;
    expect_writable(raid)?;
    expect_writable(position)?;
    let mut r = Raid::read_pda(raid)?;
    if r.status != RAID_PROPOSED {
        return Err(WarsError::RaidOpen.into());
    }
    let mut a = House::read_named(attacker, &r.attacker)?;
    let mut p = Position::read_pda(position)?;
    if &p.owner != staker.address() {
        return Err(WarsError::Unauthorized.into());
    }
    if p.house != r.attacker {
        return Err(WarsError::InvalidAccount.into());
    }
    if coins == 0 {
        return Err(WarsError::InvalidAmount.into());
    }
    if &p.raid != raid.address() || p.staked == 0 {
        return Err(WarsError::NotStaked.into());
    }
    if coins > p.staked {
        return Err(WarsError::NotEnoughCoins.into());
    }
    p.staked -= coins;
    r.staked = r.staked.checked_sub(coins).ok_or(WarsError::MathOverflow)?;
    if p.staked == 0 {
        p.raid = Address::default();
        r.stakers = r.stakers.checked_sub(1).ok_or(WarsError::MathOverflow)?;
    }
    if r.staked == 0 && &a.proposal == raid.address() {
        a.proposal = Address::default();
    }
    p.write(position)?;
    r.write(raid)?;
    a.write(attacker)
}

/// data: none · accounts: [owner (s,w), config, house (w), position (w), raid]
/// HIS BURN, a desert: a holder of a house in a raid leaves while a trade of the raid counts (`raid_window`). He is paid the law's
/// `burn_on` of all his coins (the least of their worth, his half of what he brought into the house and its reserve) out of the house, and
/// his coins are burned: the house's reserve and vq fall by what he is paid, its vt and rt stay, and what he brought falls to nothing with
/// them. A staker cannot (`Staked`: his coins are locked in the raid); a position with no coin is `NothingHeld`.
pub fn desert(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let (accounts, queue) = super::march::split(accounts, 5)?;
    let [owner, config, house, position, raid] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(owner)?;
    expect_writable(owner)?;
    expect_writable(house)?;
    expect_writable(position)?;
    let mut h = House::read_pda(house)?;
    let c = Config::read_named(config, &h.config)?;
    if h.status == HOUSE_REKT {
        return Err(WarsError::Rekt.into());
    }
    let now = now_ts()?;
    raid_window(raid, &h, now)?;
    super::march::guard(&c, raid.address(), queue.as_deref(), now, false)?;
    let mut p = position_standing(owner, house, position)?;
    if p.staked > 0 {
        return Err(WarsError::Staked.into());
    }
    if p.coins == 0 {
        return Err(WarsError::NothingHeld.into());
    }
    let (after, paid) = burn_on(&c.rules, &h.curve(), p.coins as u128, p.brought as u128).map_err(of_law)?;
    move_lamports(house, owner, u64_of(paid)?)?;
    h.set_curve(&after)?;
    h.held = minus(h.held, p.coins as u128)?;
    h.burned = plus(h.burned, p.coins as u128)?;
    // a burn ends what he brought (the law's `burn_on`): back in, he brings only what he pays then
    p.coins = 0;
    p.brought = 0;
    p.write(position)?;
    h.write(house)
}

/// data: [min_coins:8] · accounts: [owner (s,w), config, raid, from (w), from position (w), to (w), to position (w), fees (w), system]
/// HIS BURN, a change of sides: a holder of one house of a raid goes to the other while a trade of the raid counts. His coins of `from` are
/// burned and paid for as a desert's are (`burn_on`), what he brought there falling to nothing; what he is paid buys `to` (`buy_on`, fees and
/// all), his position there born if it does
/// not stand (its rent his) and its `brought` what that buy paid; the lamport a buy may leave of its spend goes back to him. A staker cannot
/// (`Staked`); refused for no coin held (`NothingHeld`), a buy of no coin (`NothingOut`) or under its least (`PastMin`), houses not both of
/// this raid (`NotInRaid`), or the same house twice (`SameHouse`).
pub fn change_sides(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let (accounts, mut queue) = super::march::split(accounts, 9)?;
    let [owner, config, raid, from, from_pos, to, to_pos, fees, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let min_coins = d.u64(0)?;
    expect_signer(owner)?;
    expect_writable(owner)?;
    expect_writable(from)?;
    expect_writable(from_pos)?;
    expect_writable(to)?;
    expect_writable(to_pos)?;
    expect_writable(fees)?;
    if from.address() == to.address() {
        return Err(WarsError::SameHouse.into());
    }
    let mut hf = House::read_pda(from)?;
    let mut ht = House::read_pda(to)?;
    if hf.config != ht.config {
        return Err(WarsError::OtherConfig.into());
    }
    let c = Config::read_named(config, &hf.config)?;
    let mut f = Fees::read_named(fees, &c.fees)?;
    if hf.status == HOUSE_REKT || ht.status == HOUSE_REKT {
        return Err(WarsError::Rekt.into());
    }
    let now = now_ts()?;
    let r = raid_window(raid, &hf, now)?;
    super::march::guard(&c, raid.address(), queue.as_deref(), now, false)?;
    if ht.raid != hf.raid {
        return Err(WarsError::NotInRaid.into());
    }
    let mut pf = position_standing(owner, from, from_pos)?;
    if pf.staked > 0 {
        return Err(WarsError::Staked.into());
    }
    if pf.coins == 0 {
        return Err(WarsError::NothingHeld.into());
    }
    let (from_after, paid) = burn_on(&c.rules, &hf.curve(), pf.coins as u128, pf.brought as u128).map_err(of_law)?;
    if c.rules.march_s > 0 {
        use crate::state::Pending;
        use poleis_war::law::{Order, OrderKind};
        let queue = queue.as_deref_mut().ok_or(ProgramError::NotEnoughAccountKeys)?;
        let mut q = super::march::queue_for(owner, queue, raid.address(), &c.rules)?;
        let order = Order::accept(&c.rules, OrderKind::Buy, paid, min_coins as u128, now, Some(q.last_acceptance)).map_err(of_law)?;
        let pt = position_of(owner, to, to_pos)?;
        let side = if to.address() == &r.attacker { 0 } else if to.address() == &r.defender { 1 } else { return Err(WarsError::NotInRaid.into()); };
        q.push(Pending { owner: *owner.address(), side, order, brought: 0, sequence: 0 })?;
        move_lamports(from, queue, u64_of(paid)?)?;
        hf.set_curve(&from_after)?;
        hf.held = minus(hf.held, pf.coins as u128)?;
        hf.burned = plus(hf.burned, pf.coins as u128)?;
        pf.coins = 0;
        pf.brought = 0;
        pf.write(from_pos)?;
        pt.write(to_pos)?;
        hf.write(from)?;
        return q.write(queue);
    }
    let (to_after, b) = buy_on(&c.rules, &ht.curve(), paid).map_err(of_law)?;
    if b.coins == 0 {
        return Err(WarsError::NothingOut.into());
    }
    if b.coins < min_coins as u128 {
        return Err(WarsError::PastMin.into());
    }
    let mut pt = position_of(owner, to, to_pos)?;
    // the buy's quote and its creator's fee into the house he goes to (the fee booked for that house's creator), its protocol's fee to the
    // publisher's fee account, the lamport a buy may leave back to him
    move_lamports(from, to, u64_of(b.quote + b.creator)?)?;
    move_lamports(from, fees, u64_of(b.protocol)?)?;
    move_lamports(from, owner, u64_of(paid - b.paid())?)?;
    hf.set_curve(&from_after)?;
    hf.held = minus(hf.held, pf.coins as u128)?;
    hf.burned = plus(hf.burned, pf.coins as u128)?;
    pf.coins = 0;
    pf.brought = 0;
    ht.set_curve(&to_after)?;
    ht.held = plus(ht.held, b.coins)?;
    ht.creator_owed = plus(ht.creator_owed, b.creator)?;
    pt.coins = plus(pt.coins, b.coins)?;
    pt.brought = plus(pt.brought, b.paid())?;
    f.book(b.protocol, 0)?;
    pf.write(from_pos)?;
    pt.write(to_pos)?;
    hf.write(from)?;
    ht.write(to)?;
    f.write(fees)
}

/// data: none · accounts: [config, raid (w), attacker, defender]
/// Anyone: the raid read at its minute, `(now - start) / minute_seconds`, by the law's `minute_on` (the model's loop, one minute: a line met
/// before the end puts the other side in its last stand, both met decide at once, the clock decides at the end, a stand revives or fails at
/// its last minute). Each minute is read once: a minute before the next one unread is `Cranked`. Refused on a proposal (`NotOpen`) and on a
/// raid decided (`Decided`). A minute nobody cranks is not read; a stand or a raid read after its last minute is read at that minute (no
/// trade counts after it: `raid_window`).
pub fn crank(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let (accounts, queue) = super::march::split(accounts, 4)?;
    let [config, raid, attacker, defender] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(raid)?;
    let mut r = Raid::read_pda(raid)?;
    let c = Config::read_named(config, &r.config)?;
    super::march::guard(&c, raid.address(), queue.as_deref(), now_ts()?, false)?;
    let a = House::read_named(attacker, &r.attacker)?;
    let df = House::read_named(defender, &r.defender)?;
    let phase = match r.phase()? {
        None => return Err(WarsError::NotOpen.into()),
        Some(Phase::Decided { .. }) => return Err(WarsError::Decided.into()),
        Some(p) => p,
    };
    let m = r.minute_at(now_ts()?);
    if m < r.next_minute {
        return Err(WarsError::Cranked.into());
    }
    let chests = (a.chest as u128) + (df.chest as u128);
    let next = minute_on(&c.rules, phase, r.minutes, m, [&a.curve(), &df.curve()], [r.m0_att as u128, r.m0_def as u128], chests).map_err(of_law)?;
    r.set_phase(next);
    r.next_minute = m.checked_add(1).ok_or(WarsError::MathOverflow)?;
    r.write(raid)
}
