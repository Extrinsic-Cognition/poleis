//! A war's config, set once by its authority, and its houses, born by anyone.

use crate::error::WarsError;
use crate::state::{Config, Fees, House, HOUSE_ACTIVE, SEED_CONFIG, SEED_FEES, SEED_HOUSE};
use crate::util::{create_pda, expect_signer, expect_writable, find_and_expect_pda, now_ts, stands, Data};
use poleis_war::law::Curve;
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// data: [generation:8][treasury:32][vt:8][vq:8][rt:8][the law's eighteen u32 rules, in its order][shield_age_s:8][shield_mcap:8]
///       [minute_seconds:4] (Config::INIT_LEN)
/// accounts: [payer (s,w), authority (s), config (w), fees (w), system]
/// ONE GENERATION of the war's rules, founded once: its PDA is ["config", authority, generation], so nobody but its authority founds it, and
/// nothing changes it after; the next generation is founded beside it. The authority only signs (on poleis.us the game swarm's seal, by CPI
/// from its sealed proposal, never a seat's key); the payer pays the rents. `treasury` is the publisher's, where `pay_publisher` sends every
/// fee the generation's fee account holds (on poleis.us the game swarm's Poleis treasury): refused when it is none (`NoTreasury`). The rules
/// must be rules the law carries (`Rules::check`) and the minute at least a second, else `InvalidRules`. Its fee account ["fees", config] is
/// born beside it; what that address held before its birth is booked as `donated`.
pub fn init_config(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    init_config_with_march(accounts, data, 0, 0, 0)
}

/// Legacy config bytes followed by march_s:u32, optionally followed by min_order:u64 and max_per_owner:u8.
pub fn init_march_config(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let d = Data(data);
    let march_s = d.u32(Config::INIT_LEN)?;
    let limits_at = Config::INIT_LEN + 4;
    let (min_order, max_per_owner) = if data.len() == limits_at {
        (0, 0)
    } else {
        d.expect_len(limits_at + 8 + 1)?;
        (d.u64(limits_at)?, data[limits_at + 8])
    };
    if march_s == 0 && (min_order != 0 || max_per_owner != 0) {
        return Err(WarsError::InvalidRules.into());
    }
    init_config_with_march(accounts, &data[..Config::INIT_LEN], march_s, min_order, max_per_owner)
}

fn init_config_with_march(
    accounts: &mut [AccountView], data: &[u8], march_s: u32, min_order: u64, max_per_owner: u8,
) -> ProgramResult {
    let [payer, authority, config, fees, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(Config::INIT_LEN)?;
    let generation = d.u64(0)?;
    let treasury = Address::new_from_array(data[8..40].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
    expect_signer(payer)?;
    expect_writable(payer)?;
    expect_signer(authority)?;
    expect_writable(config)?;
    expect_writable(fees)?;
    if treasury == Address::default() {
        return Err(WarsError::NoTreasury.into());
    }
    let (mut rules, shield_age_s, shield_mcap, minute_seconds) = Config::parse(data, 40)?;
    rules.march_s = march_s;
    rules.march_min_order = min_order;
    rules.march_max_per_owner = max_per_owner;
    rules.check().map_err(|_| WarsError::InvalidRules)?;
    if minute_seconds == 0 {
        return Err(WarsError::InvalidRules.into());
    }
    let g = generation.to_le_bytes();
    let cseeds: &[&[u8]] = &[SEED_CONFIG, authority.address().as_ref(), &g];
    let cbump = find_and_expect_pda(config, cseeds)?;
    if stands(config) {
        return Err(WarsError::ConfigStands.into());
    }
    let fseeds: &[&[u8]] = &[SEED_FEES, config.address().as_ref()];
    let fbump = find_and_expect_pda(fees, fseeds)?;
    if stands(fees) {
        return Err(WarsError::ConfigStands.into());
    }
    create_pda(payer, config, Config::size_for(&rules), cseeds, cbump)?;
    let donated = create_pda(payer, fees, Fees::SIZE, fseeds, fbump)?;
    Config { authority: *authority.address(), generation, treasury, fees: *fees.address(), rules, shield_age_s, shield_mcap, minute_seconds, bump: cbump }.write(config)?;
    Fees { config: *config.address(), protocol: 0, taxed: 0, donated, paid: 0, bump: fbump }.write(fees)
}

/// data: [house_id:8] · accounts: [creator (s,w), config, house (w), system]
/// A house is born into the generation the config is, on its curve, at the clock (its age, for the shield, counts from here), its PDA
/// ["house", config, house_id]. Its creator is whoever launched it, the signer, a player's wallet or an agent's key: he pays its rent, and
/// the creator's share of every trade into it is his (`creator_owed`, pushed to him by `pay_creator`). What its address held before its birth
/// goes into its chest, so its lamports stay its rent, its reserve, its chest and what it owes its creator.
pub fn init_house(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [creator, config, house, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let house_id = d.u64(0)?;
    expect_signer(creator)?;
    expect_writable(creator)?;
    expect_writable(house)?;
    let c = Config::read_pda(config)?;
    let id = house_id.to_le_bytes();
    let seeds: &[&[u8]] = &[SEED_HOUSE, config.address().as_ref(), &id];
    let bump = find_and_expect_pda(house, seeds)?;
    if stands(house) {
        return Err(WarsError::HouseStands.into());
    }
    let donated = create_pda(creator, house, House::SIZE, seeds, bump)?;
    let mut h = House {
        config: *config.address(),
        house_id,
        status: HOUSE_ACTIVE,
        vt: 0,
        vq: 0,
        rt: 0,
        rq: 0,
        rt0: 0,
        chest: donated,
        held: 0,
        burned: 0,
        born: now_ts()?,
        raid: Address::default(),
        proposal: Address::default(),
        dump_rate_bps: 0,
        dump_start: 0,
        dump_end: 0,
        creator: *creator.address(),
        creator_owed: 0,
        bump,
    };
    h.set_curve(&Curve::born(&c.rules))?;
    h.write(house)
}
