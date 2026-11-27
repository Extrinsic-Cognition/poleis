//! The war's instructions, built as the program's tests build them (poleis/program/poleis_wars/tests/common/mod.rs), and its accounts read by
//! the program's own offsets (poleis_wars::state).

use poleis_war::law::{Curve, How, Phase, Rules, Side};
use poleis_wars::state::*;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(poleis_wars::ID.to_bytes());
pub const SYSTEM: Pubkey = Pubkey::new_from_array([0u8; 32]);

/// a generation of the war's rules: ["config", authority, generation]
pub fn config_pda(authority: &Pubkey, generation: u64) -> Pubkey {
    Pubkey::find_program_address(&[SEED_CONFIG, authority.as_ref(), &generation.to_le_bytes()], &PROGRAM_ID).0
}
pub fn fees_pda(config: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[SEED_FEES, config.as_ref()], &PROGRAM_ID).0
}
pub fn house_pda(config: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(&[SEED_HOUSE, config.as_ref(), &id.to_le_bytes()], &PROGRAM_ID).0
}
pub fn position_pda(house: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[SEED_POSITION, house.as_ref(), owner.as_ref()], &PROGRAM_ID).0
}
pub fn raid_pda(attacker: &Pubkey, defender: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[SEED_RAID, attacker.as_ref(), defender.as_ref()], &PROGRAM_ID).0
}

fn ix(tag: u8, data: &[u8], metas: Vec<AccountMeta>) -> Instruction {
    let mut d = vec![tag];
    d.extend_from_slice(data);
    Instruction::new_with_bytes(PROGRAM_ID, &d, metas)
}
/// the bytes `init_config` takes: the law's rules, the shield and the minute
pub fn rules_bytes(r: &Rules, shield_age_s: u64, shield_mcap: u64, minute_seconds: u32) -> Vec<u8> {
    let mut d = Vec::new();
    for v in [r.vt, r.vq, r.rt] {
        d.extend_from_slice(&(v as u64).to_le_bytes());
    }
    for v in [
        r.protocol_bps,
        r.creator_bps,
        r.stake_own_pm,
        r.stake_gap_pm,
        r.stake_max_pm,
        r.opponent_min_pm,
        r.opponent_max_pm,
        r.raid_min_s,
        r.raid_max_s,
        r.raid_step_s,
        r.stand_minutes,
        r.revival_bps,
        r.staker_bonus_pm,
        r.keep_bps,
        r.dump_min_bps,
        r.dump_min_s,
        r.dump_max_s,
        r.dump_step_s,
    ] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    d.extend_from_slice(&shield_age_s.to_le_bytes());
    d.extend_from_slice(&shield_mcap.to_le_bytes());
    d.extend_from_slice(&minute_seconds.to_le_bytes());
    assert_eq!(d.len(), Config::DATA_LEN);
    d
}
/// accounts: [payer (s,w), authority (s), config (w), fees (w), system] · data [generation:8][treasury:32][the rules]
pub fn init_config(payer: Pubkey, authority: Pubkey, generation: u64, treasury: Pubkey, bytes: &[u8]) -> Instruction {
    let config = config_pda(&authority, generation);
    let mut d = generation.to_le_bytes().to_vec();
    d.extend_from_slice(treasury.as_ref());
    d.extend_from_slice(bytes);
    ix(
        poleis_wars::IX_INIT_CONFIG,
        &d,
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(config, false),
            AccountMeta::new(fees_pda(&config), false),
            AccountMeta::new_readonly(SYSTEM, false),
        ],
    )
}
/// accounts: [house (w), creator (w)]
pub fn pay_creator(house: Pubkey, creator: Pubkey) -> Instruction {
    ix(poleis_wars::IX_PAY_CREATOR, &[], vec![AccountMeta::new(house, false), AccountMeta::new(creator, false)])
}
/// accounts: [config, fees (w), treasury (w)]
pub fn pay_publisher(config: Pubkey, treasury: Pubkey) -> Instruction {
    ix(poleis_wars::IX_PAY_PUBLISHER, &[], vec![AccountMeta::new_readonly(config, false), AccountMeta::new(fees_pda(&config), false), AccountMeta::new(treasury, false)])
}
pub fn init_house(creator: Pubkey, config: Pubkey, id: u64) -> Instruction {
    ix(poleis_wars::IX_INIT_HOUSE, &id.to_le_bytes(), vec![AccountMeta::new(creator, true), AccountMeta::new_readonly(config, false), AccountMeta::new(house_pda(&config, id), false), AccountMeta::new_readonly(SYSTEM, false)])
}
pub fn buy(buyer: Pubkey, config: Pubkey, house: Pubkey, spend: u64, min_coins: u64, raid: Option<Pubkey>) -> Instruction {
    let mut d = spend.to_le_bytes().to_vec();
    d.extend_from_slice(&min_coins.to_le_bytes());
    let mut metas = vec![
        AccountMeta::new(buyer, true),
        AccountMeta::new_readonly(config, false),
        AccountMeta::new(house, false),
        AccountMeta::new(position_pda(&house, &buyer), false),
        AccountMeta::new(fees_pda(&config), false),
        AccountMeta::new_readonly(SYSTEM, false),
    ];
    if let Some(r) = raid {
        metas.push(AccountMeta::new_readonly(r, false));
    }
    ix(poleis_wars::IX_BUY, &d, metas)
}
pub fn sell(seller: Pubkey, config: Pubkey, house: Pubkey, coins: u64, min_out: u64) -> Instruction {
    let mut d = coins.to_le_bytes().to_vec();
    d.extend_from_slice(&min_out.to_le_bytes());
    ix(
        poleis_wars::IX_SELL,
        &d,
        vec![
            AccountMeta::new(seller, true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(house, false),
            AccountMeta::new(position_pda(&house, &seller), false),
            AccountMeta::new(fees_pda(&config), false),
        ],
    )
}
pub fn stake(staker: Pubkey, config: Pubkey, attacker: Pubkey, defender: Pubkey, coins: u64) -> Instruction {
    ix(
        poleis_wars::IX_STAKE,
        &coins.to_le_bytes(),
        vec![
            AccountMeta::new(staker, true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(attacker, false),
            AccountMeta::new(defender, false),
            AccountMeta::new(raid_pda(&attacker, &defender), false),
            AccountMeta::new(position_pda(&attacker, &staker), false),
            AccountMeta::new_readonly(SYSTEM, false),
        ],
    )
}
pub fn desert(owner: Pubkey, config: Pubkey, house: Pubkey, raid: Pubkey) -> Instruction {
    ix(
        poleis_wars::IX_DESERT,
        &[],
        vec![
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(house, false),
            AccountMeta::new(position_pda(&house, &owner), false),
            AccountMeta::new_readonly(raid, false),
        ],
    )
}
pub fn change_sides(owner: Pubkey, config: Pubkey, raid: Pubkey, from: Pubkey, to: Pubkey, min_coins: u64) -> Instruction {
    ix(
        poleis_wars::IX_CHANGE_SIDES,
        &min_coins.to_le_bytes(),
        vec![
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new_readonly(raid, false),
            AccountMeta::new(from, false),
            AccountMeta::new(position_pda(&from, &owner), false),
            AccountMeta::new(to, false),
            AccountMeta::new(position_pda(&to, &owner), false),
            AccountMeta::new(fees_pda(&config), false),
            AccountMeta::new_readonly(SYSTEM, false),
        ],
    )
}
pub fn crank(config: Pubkey, attacker: Pubkey, defender: Pubkey) -> Instruction {
    ix(
        poleis_wars::IX_CRANK,
        &[],
        vec![
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(raid_pda(&attacker, &defender), false),
            AccountMeta::new_readonly(attacker, false),
            AccountMeta::new_readonly(defender, false),
        ],
    )
}
pub fn settle(config: Pubkey, attacker: Pubkey, defender: Pubkey) -> Instruction {
    ix(
        poleis_wars::IX_SETTLE,
        &[],
        vec![
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(raid_pda(&attacker, &defender), false),
            AccountMeta::new(attacker, false),
            AccountMeta::new(defender, false),
            AccountMeta::new(fees_pda(&config), false),
        ],
    )
}
pub fn convert(owner: Pubkey, config: Pubkey, raid: Pubkey, loser: Pubkey, winner: Pubkey) -> Instruction {
    ix(
        poleis_wars::IX_CONVERT,
        &[],
        vec![
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(raid, false),
            AccountMeta::new(loser, false),
            AccountMeta::new(position_pda(&loser, &owner), false),
            AccountMeta::new(winner, false),
            AccountMeta::new(position_pda(&winner, &owner), false),
            AccountMeta::new_readonly(SYSTEM, false),
        ],
    )
}
pub fn unlock(owner: Pubkey, raid: Pubkey, house: Pubkey) -> Instruction {
    ix(poleis_wars::IX_UNLOCK, &[], vec![AccountMeta::new_readonly(owner, true), AccountMeta::new_readonly(raid, false), AccountMeta::new(position_pda(&house, &owner), false)])
}

fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
fn i64_at(d: &[u8], o: usize) -> i64 {
    i64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
fn u128_at(d: &[u8], o: usize) -> u128 {
    u128::from_le_bytes(d[o..o + 16].try_into().unwrap())
}
fn key_at(d: &[u8], o: usize) -> Pubkey {
    Pubkey::new_from_array(d[o..o + 32].try_into().unwrap())
}
/// the account's bytes are of this layout: its size, its discriminator, its version
fn of(d: &[u8], size: usize, disc: u8) -> Result<(), String> {
    if d.len() != size || d[0] != disc || d[1] != LAYOUT_VERSION {
        return Err(format!("an account of {} bytes, discriminator {} and version {}: not the layout read", d.len(), d.first().copied().unwrap_or(0), d.get(1).copied().unwrap_or(0)));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseView {
    pub status: u8,
    pub curve: Curve,
    pub chest: u64,
    pub held: u64,
    pub burned: u64,
    pub raid: Pubkey,
    pub dump_rate_bps: u32,
    pub dump_start: i64,
    pub dump_end: i64,
    pub creator: Pubkey,
    pub creator_owed: u64,
}
pub fn house_view(d: &[u8]) -> Result<HouseView, String> {
    of(d, House::SIZE, DISC_HOUSE)?;
    Ok(HouseView {
        status: d[House::OFF_STATUS],
        curve: Curve {
            vt: u64_at(d, House::OFF_VT) as u128,
            vq: u64_at(d, House::OFF_VQ) as u128,
            rt: u64_at(d, House::OFF_RT) as u128,
            rq: u64_at(d, House::OFF_RQ) as u128,
            rt0: u64_at(d, House::OFF_RT0) as u128,
        },
        chest: u64_at(d, House::OFF_CHEST),
        held: u64_at(d, House::OFF_HELD),
        burned: u64_at(d, House::OFF_BURNED),
        raid: key_at(d, House::OFF_RAID),
        dump_rate_bps: u32_at(d, House::OFF_DUMP_RATE),
        dump_start: i64_at(d, House::OFF_DUMP_START),
        dump_end: i64_at(d, House::OFF_DUMP_END),
        creator: key_at(d, House::OFF_CREATOR),
        creator_owed: u64_at(d, House::OFF_CREATOR_OWED),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionView {
    pub house: Pubkey,
    pub owner: Pubkey,
    pub coins: u64,
    pub staked: u64,
    pub brought: u64,
    pub raid: Pubkey,
}
pub fn position_view(d: &[u8]) -> Result<PositionView, String> {
    of(d, Position::SIZE, DISC_POSITION)?;
    Ok(PositionView {
        house: key_at(d, Position::OFF_HOUSE),
        owner: key_at(d, Position::OFF_OWNER),
        coins: u64_at(d, Position::OFF_COINS),
        staked: u64_at(d, Position::OFF_STAKED),
        brought: u64_at(d, Position::OFF_BROUGHT),
        raid: key_at(d, Position::OFF_RAID),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaidView {
    pub status: u8,
    pub staked: u64,
    pub m0: [u64; 2],
    pub need: u64,
    pub start: i64,
    pub minutes: u32,
    pub seconds: u64,
    pub minute_seconds: u32,
    pub next_minute: u32,
    pub stand: (u8, u32, u32),
    pub winner: u8,
    pub how: u8,
    pub how_met: u32,
    pub decided_at: u32,
    pub mcap_end: [u64; 2],
    pub pot_in: u64,
    pub bought_with: u64,
    pub coins_bought: u64,
    pub to_chest: u64,
    pub weights: u128,
    pub converted: u64,
}
impl RaidView {
    /// the raid's phase as the law reads it (None for a proposal)
    pub fn phase(&self) -> Option<Phase> {
        let side = |b: u8| Side::of(b).unwrap_or(Side::Defender);
        match self.status {
            RAID_PROPOSED => None,
            RAID_ACTIVE => Some(Phase::Open),
            RAID_STAND => Some(Phase::Stand { side: side(self.stand.0), met: self.stand.1, ends: self.stand.2 }),
            _ => {
                let how = match self.how {
                    HOW_BOTH_MET => How::BothMet,
                    HOW_CLOCK => How::Clock { tie: false },
                    HOW_CLOCK_TIE => How::Clock { tie: true },
                    HOW_STAND_FAILED => How::StandFailed { met: self.how_met },
                    _ => How::Revived,
                };
                Some(Phase::Decided { winner: side(self.winner), how, at: self.decided_at })
            }
        }
    }
}
pub fn raid_view(d: &[u8]) -> Result<RaidView, String> {
    of(d, Raid::SIZE, DISC_RAID)?;
    Ok(RaidView {
        status: d[Raid::OFF_STATUS],
        staked: u64_at(d, Raid::OFF_STAKED),
        m0: [u64_at(d, Raid::OFF_M0_ATT), u64_at(d, Raid::OFF_M0_DEF)],
        need: u64_at(d, Raid::OFF_NEED),
        start: i64_at(d, Raid::OFF_START),
        minutes: u32_at(d, Raid::OFF_MINUTES),
        seconds: u64_at(d, Raid::OFF_SECONDS),
        minute_seconds: u32_at(d, Raid::OFF_MINUTE_SECONDS),
        next_minute: u32_at(d, Raid::OFF_NEXT_MINUTE),
        stand: (d[Raid::OFF_STAND_SIDE], u32_at(d, Raid::OFF_STAND_MET), u32_at(d, Raid::OFF_STAND_ENDS)),
        winner: d[Raid::OFF_WINNER],
        how: d[Raid::OFF_HOW],
        how_met: u32_at(d, Raid::OFF_HOW_MET),
        decided_at: u32_at(d, Raid::OFF_DECIDED_AT),
        mcap_end: [u64_at(d, Raid::OFF_MCAP_END_ATT), u64_at(d, Raid::OFF_MCAP_END_DEF)],
        pot_in: u64_at(d, Raid::OFF_POT_IN),
        bought_with: u64_at(d, Raid::OFF_BOUGHT_WITH),
        coins_bought: u64_at(d, Raid::OFF_COINS_BOUGHT),
        to_chest: u64_at(d, Raid::OFF_TO_CHEST),
        weights: u128_at(d, Raid::OFF_WEIGHTS),
        converted: u64_at(d, Raid::OFF_CONVERTED),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeesView {
    pub protocol: u64,
    pub taxed: u64,
    pub donated: u64,
    pub paid: u64,
}
impl FeesView {
    /// what its book says it holds beyond its rent: protocol + taxed + donated - paid
    pub fn holds(&self) -> u64 {
        self.protocol + self.taxed + self.donated - self.paid
    }
}
pub fn fees_view(d: &[u8]) -> Result<FeesView, String> {
    of(d, Fees::SIZE, DISC_FEES)?;
    Ok(FeesView { protocol: u64_at(d, Fees::OFF_PROTOCOL), taxed: u64_at(d, Fees::OFF_TAXED), donated: u64_at(d, Fees::OFF_DONATED), paid: u64_at(d, Fees::OFF_PAID) })
}

/// POLEIS'S OWN INSTRUCTIONS this rehearsal needs to found the game swarm's treasury on a local validator (poleis_treasury/src/lib.rs's
/// tags, src/ix/treasury.rs's data): a tree's config (any root but 0, which only Poleis's deployer founds), a unit's treasury in solo mode,
/// and sync_endowment, which books what was wired straight in. Its layout of a treasury (src/state.rs Treasury): 27 bytes, total_in at 10.
pub const POLEIS_TREASURY_SIZE: usize = 27;
pub const POLEIS_TREASURY_OFF_TOTAL_IN: usize = 10;
pub fn poleis_tree(poleis: &Pubkey, root: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"config", &root.to_le_bytes()], poleis).0
}
pub fn poleis_treasury(poleis: &Pubkey, unit: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"treasury", &unit.to_le_bytes()], poleis).0
}
pub fn poleis_init_config(poleis: &Pubkey, payer: Pubkey, root: u64) -> Instruction {
    let mut d = vec![0u8];
    d.extend_from_slice(&root.to_le_bytes());
    d.extend_from_slice(&[0u8; 32]);
    Instruction::new_with_bytes(*poleis, &d, vec![AccountMeta::new(payer, true), AccountMeta::new(poleis_tree(poleis, root), false), AccountMeta::new_readonly(SYSTEM, false)])
}
pub fn poleis_create_treasury(poleis: &Pubkey, authority: Pubkey, root: u64, unit: u64) -> Instruction {
    let mut d = vec![1u8];
    d.extend_from_slice(&unit.to_le_bytes());
    Instruction::new_with_bytes(
        *poleis,
        &d,
        vec![AccountMeta::new(authority, true), AccountMeta::new_readonly(poleis_tree(poleis, root), false), AccountMeta::new(poleis_treasury(poleis, unit), false), AccountMeta::new_readonly(SYSTEM, false)],
    )
}
pub fn poleis_sync_endowment(poleis: &Pubkey, unit: u64) -> Instruction {
    let mut d = vec![3u8];
    d.extend_from_slice(&unit.to_le_bytes());
    Instruction::new_with_bytes(*poleis, &d, vec![AccountMeta::new(poleis_treasury(poleis, unit), false)])
}
