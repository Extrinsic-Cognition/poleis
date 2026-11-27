//! What the tests share: the world on Mollusk, the instruction builders and the account readers. The readers read the program's own offsets
//! (poleis_wars::state), so a test reads the bytes the program writes; nothing here is in the program. THE BOOK is held after every
//! instruction that succeeds: every lamport of the world is where it was or moved, none made or lost; every house holds exactly its rent,
//! its reserve and its chest; the fee account its rent and what it was paid; and a house's `held` is the coins its positions hold.
#![allow(dead_code)]

use crate::built;
use embertide_war::law::{Curve, How, Phase, Rules, Side};
use mollusk_svm::{program::keyed_account_for_system_program, result::ProgramResult, Mollusk};
use poleis_wars::error::WarsError;
use poleis_wars::state::*;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(poleis_wars::ID.to_bytes());
pub const SOL: u64 = 1_000_000_000;

pub fn system_id() -> Pubkey {
    keyed_account_for_system_program().0
}
pub fn code(e: WarsError) -> ProgramError {
    ProgramError::Custom(e as u32)
}

// ---- PDAs ----
/// generation 0's config of an authority (the tests' one generation, unless a test founds another)
pub fn config_pda(authority: &Pubkey) -> Pubkey {
    config_pda_of(authority, 0)
}
pub fn config_pda_of(authority: &Pubkey, generation: u64) -> Pubkey {
    Pubkey::find_program_address(&[SEED_CONFIG, authority.as_ref(), &generation.to_le_bytes()], &PROGRAM_ID).0
}
/// the publisher's treasury the tests' generations name unless a test names a Poleis one: an address nobody holds a key for, standing in
/// for the game swarm's Poleis treasury (the tests/publisher.rs world names a real one)
pub fn publisher_of(authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"the publisher's stand-in", authority.as_ref()], &PROGRAM_ID).0
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

// ---- builders ----
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
/// generation 0 of `authority`, which pays for it too, naming the publisher's stand-in (`publisher_of`)
pub fn init_config(authority: Pubkey, bytes: &[u8]) -> Instruction {
    init_config_of(authority, authority, 0, publisher_of(&authority), bytes)
}
/// accounts: [payer (s,w), authority (s), config (w), fees (w), system] · data [generation:8][treasury:32][the rules]
pub fn init_config_of(payer: Pubkey, authority: Pubkey, generation: u64, treasury: Pubkey, bytes: &[u8]) -> Instruction {
    let config = config_pda_of(&authority, generation);
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
            AccountMeta::new_readonly(system_id(), false),
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
/// accounts: [creator (s,w), config, house (w), system]
pub fn init_house(creator: Pubkey, config: Pubkey, id: u64) -> Instruction {
    ix(
        poleis_wars::IX_INIT_HOUSE,
        &id.to_le_bytes(),
        vec![AccountMeta::new(creator, true), AccountMeta::new_readonly(config, false), AccountMeta::new(house_pda(&config, id), false), AccountMeta::new_readonly(system_id(), false)],
    )
}
/// accounts: [buyer (s,w), config, house (w), position (w), fees (w), system, raid (for a house in a raid)]
pub fn buy(buyer: Pubkey, config: Pubkey, house: Pubkey, spend: u64, min_coins: u64, raid: Option<Pubkey>) -> Instruction {
    let mut d = spend.to_le_bytes().to_vec();
    d.extend_from_slice(&min_coins.to_le_bytes());
    let mut metas = vec![
        AccountMeta::new(buyer, true),
        AccountMeta::new_readonly(config, false),
        AccountMeta::new(house, false),
        AccountMeta::new(position_pda(&house, &buyer), false),
        AccountMeta::new(fees_pda(&config), false),
        AccountMeta::new_readonly(system_id(), false),
    ];
    if let Some(r) = raid {
        metas.push(AccountMeta::new_readonly(r, false));
    }
    ix(poleis_wars::IX_BUY, &d, metas)
}
/// accounts: [seller (s,w), config, house (w), position (w), fees (w)]
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
/// accounts: [staker (s,w), config, attacker (w), defender (w), raid (w), position (w), system]
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
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// accounts: [owner (s,w), config, house (w), position (w), raid]
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
/// accounts: [owner (s,w), config, raid, from (w), from position (w), to (w), to position (w), fees (w), system]
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
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// accounts: [config, raid (w), attacker, defender]
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
/// accounts: [config, raid (w), attacker (w), defender (w), fees (w)]
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
/// accounts: [owner (s,w), config, raid (w), loser (w), loser position (w), winner (w), winner position (w), system]
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
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// accounts: [staker (s), attacker (w), raid (w), position (w)]
pub fn unstake(staker: Pubkey, attacker: Pubkey, defender: Pubkey, coins: u64) -> Instruction {
    ix(
        poleis_wars::IX_UNSTAKE,
        &coins.to_le_bytes(),
        vec![
            AccountMeta::new_readonly(staker, true),
            AccountMeta::new(attacker, false),
            AccountMeta::new(raid_pda(&attacker, &defender), false),
            AccountMeta::new(position_pda(&attacker, &staker), false),
        ],
    )
}
/// accounts: [owner (s), raid, position (w)]
pub fn unlock(owner: Pubkey, raid: Pubkey, house: Pubkey) -> Instruction {
    ix(poleis_wars::IX_UNLOCK, &[], vec![AccountMeta::new_readonly(owner, true), AccountMeta::new_readonly(raid, false), AccountMeta::new(position_pda(&house, &owner), false)])
}

// ---- readers: the program's own offsets ----
pub fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
pub fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
pub fn i64_at(d: &[u8], o: usize) -> i64 {
    i64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
pub fn u128_at(d: &[u8], o: usize) -> u128 {
    u128::from_le_bytes(d[o..o + 16].try_into().unwrap())
}
pub fn key_at(d: &[u8], o: usize) -> Pubkey {
    Pubkey::new_from_array(d[o..o + 32].try_into().unwrap())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseView {
    pub config: Pubkey,
    pub house_id: u64,
    pub status: u8,
    pub curve: Curve,
    pub chest: u64,
    pub held: u64,
    pub burned: u64,
    pub born: i64,
    pub raid: Pubkey,
    pub proposal: Pubkey,
    pub dump_rate_bps: u32,
    pub dump_start: i64,
    pub dump_end: i64,
    pub creator: Pubkey,
    pub creator_owed: u64,
}
pub fn house_view(d: &[u8]) -> HouseView {
    assert_eq!((d.len(), d[0], d[1]), (House::SIZE, DISC_HOUSE, LAYOUT_VERSION));
    HouseView {
        config: key_at(d, House::OFF_CONFIG),
        house_id: u64_at(d, House::OFF_HOUSE_ID),
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
        born: i64_at(d, House::OFF_BORN),
        raid: key_at(d, House::OFF_RAID),
        proposal: key_at(d, House::OFF_PROPOSAL),
        dump_rate_bps: u32_at(d, House::OFF_DUMP_RATE),
        dump_start: i64_at(d, House::OFF_DUMP_START),
        dump_end: i64_at(d, House::OFF_DUMP_END),
        creator: key_at(d, House::OFF_CREATOR),
        creator_owed: u64_at(d, House::OFF_CREATOR_OWED),
    }
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
pub fn position_view(d: &[u8]) -> PositionView {
    assert_eq!((d.len(), d[0], d[1]), (Position::SIZE, DISC_POSITION, LAYOUT_VERSION));
    PositionView {
        house: key_at(d, Position::OFF_HOUSE),
        owner: key_at(d, Position::OFF_OWNER),
        coins: u64_at(d, Position::OFF_COINS),
        staked: u64_at(d, Position::OFF_STAKED),
        brought: u64_at(d, Position::OFF_BROUGHT),
        raid: key_at(d, Position::OFF_RAID),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaidView {
    pub config: Pubkey,
    pub attacker: Pubkey,
    pub defender: Pubkey,
    pub status: u8,
    pub staked: u64,
    pub stakers: u32,
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
        let side = |b: u8| Side::of(b).unwrap();
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
pub fn raid_view(d: &[u8]) -> RaidView {
    assert_eq!((d.len(), d[0], d[1]), (Raid::SIZE, DISC_RAID, LAYOUT_VERSION));
    RaidView {
        config: key_at(d, Raid::OFF_CONFIG),
        attacker: key_at(d, Raid::OFF_ATTACKER),
        defender: key_at(d, Raid::OFF_DEFENDER),
        status: d[Raid::OFF_STATUS],
        staked: u64_at(d, Raid::OFF_STAKED),
        stakers: u32_at(d, Raid::OFF_STAKERS),
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
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeesView {
    pub config: Pubkey,
    pub protocol: u64,
    pub taxed: u64,
    pub donated: u64,
    pub paid: u64,
}
pub fn fees_view(d: &[u8]) -> FeesView {
    assert_eq!((d.len(), d[0], d[1]), (Fees::SIZE, DISC_FEES, LAYOUT_VERSION));
    FeesView {
        config: key_at(d, Fees::OFF_CONFIG),
        protocol: u64_at(d, Fees::OFF_PROTOCOL),
        taxed: u64_at(d, Fees::OFF_TAXED),
        donated: u64_at(d, Fees::OFF_DONATED),
        paid: u64_at(d, Fees::OFF_PAID),
    }
}

// ---- the world ----
pub struct World {
    pub m: Mollusk,
    pub accounts: Vec<(Pubkey, Account)>,
    /// whether every instruction that succeeds is followed by THE BOOK (off only where a test writes an account by hand)
    pub keep_book: bool,
}

impl World {
    /// a world whose clock stands at `now` (unix seconds)
    pub fn new(now: i64) -> Self {
        std::env::set_var("SBF_OUT_DIR", built::program_dir());
        let mut m = Mollusk::new(&PROGRAM_ID, "poleis_wars");
        m.sysvars.clock.unix_timestamp = now;
        m.sysvars.clock.slot = 1_000;
        Self { m, accounts: vec![keyed_account_for_system_program()], keep_book: true }
    }
    pub fn now(&self) -> i64 {
        self.m.sysvars.clock.unix_timestamp
    }
    /// the clock at `now`, its slot moved on with it (one slot a second, never back)
    pub fn set_now(&mut self, now: i64) {
        let d = now - self.m.sysvars.clock.unix_timestamp;
        self.m.sysvars.clock.unix_timestamp = now;
        if d > 0 {
            self.m.sysvars.clock.slot += d as u64;
        }
    }
    pub fn wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        key
    }
    pub fn account(&self, key: &Pubkey) -> Account {
        self.accounts.iter().find(|(k, _)| k == key).map(|(_, a)| a.clone()).unwrap_or_default()
    }
    pub fn data(&self, key: &Pubkey) -> Vec<u8> {
        self.account(key).data
    }
    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.account(key).lamports
    }
    pub fn put(&mut self, key: Pubkey, account: Account) {
        match self.accounts.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = account,
            None => self.accounts.push((key, account)),
        }
    }
    pub fn rent(&self, size: usize) -> u64 {
        self.m.sysvars.rent.minimum_balance(size)
    }
    pub fn house(&self, key: &Pubkey) -> HouseView {
        house_view(&self.data(key))
    }
    pub fn raid(&self, key: &Pubkey) -> RaidView {
        raid_view(&self.data(key))
    }
    pub fn fees(&self, key: &Pubkey) -> FeesView {
        fees_view(&self.data(key))
    }
    /// an owner's position in a house, if it stands
    pub fn position(&self, house: &Pubkey, owner: &Pubkey) -> Option<PositionView> {
        let d = self.data(&position_pda(house, owner));
        if d.is_empty() {
            None
        } else {
            Some(position_view(&d))
        }
    }
    /// every lamport of the world
    pub fn total(&self) -> u128 {
        self.accounts.iter().map(|(_, a)| a.lamports as u128).sum()
    }
    /// Run one instruction against the world: its compute units, or the program's refusal. Only a success changes the world (a failed
    /// instruction leaves nothing, as on a chain).
    pub fn run(&mut self, ix: &Instruction) -> Result<u64, ProgramError> {
        for meta in &ix.accounts {
            if !self.accounts.iter().any(|(k, _)| *k == meta.pubkey) {
                self.accounts.push((meta.pubkey, Account::default()));
            }
        }
        let before = self.total();
        let r = self.m.process_instruction(ix, &self.accounts);
        match r.program_result {
            ProgramResult::Success => {
                for (key, account) in r.resulting_accounts {
                    self.put(key, account);
                }
                assert_eq!(self.total(), before, "THE BOOK: no lamport is made or lost");
                if self.keep_book {
                    book(self);
                }
                Ok(r.compute_units_consumed)
            }
            ProgramResult::Failure(e) => Err(e),
            ProgramResult::UnknownError(e) => panic!("the runtime failed: {e:?}"),
        }
    }
    /// Run an instruction that must succeed: its compute units.
    pub fn ok(&mut self, ix: &Instruction, what: &str) -> u64 {
        match self.run(ix) {
            Ok(cu) => cu,
            Err(e) => panic!("{what}: refused with {e:?}"),
        }
    }
    /// Run an instruction that must be refused with exactly `e`, and check that no account of the world changed.
    pub fn refuse(&mut self, ix: &Instruction, e: ProgramError, what: &str) {
        let before = self.accounts.clone();
        let got = self.run(ix);
        assert_eq!(got, Err(e), "{what}");
        for (k, a) in &self.accounts {
            if !before.iter().any(|(b, _)| b == k) {
                assert_eq!(a, &Account::default(), "{what}: account {k} was created");
            }
        }
        for (k, a) in &before {
            assert_eq!(&self.account(k), a, "{what}: account {k} changed");
        }
    }
}

/// THE BOOK of the world: every house (of this program, at its own PDA) holds exactly its rent, its reserve, its chest and what it owes its
/// creator, and its `held` is the coins its positions hold; every fee account holds its rent and protocol + taxed + donated - paid.
pub fn book(w: &World) {
    let mut held: std::collections::BTreeMap<Pubkey, u128> = Default::default();
    for (k, a) in &w.accounts {
        // only this layout's positions at their own PDA: a test's fakes at other addresses are nobody's
        if a.owner == PROGRAM_ID && a.data.len() == Position::SIZE && a.data[0] == DISC_POSITION && a.data[1] == LAYOUT_VERSION {
            let p = position_view(&a.data);
            if *k == position_pda(&p.house, &p.owner) {
                *held.entry(p.house).or_default() += p.coins as u128;
            }
        }
    }
    for (k, a) in &w.accounts {
        if a.owner != PROGRAM_ID || a.data.len() < 2 || a.data[1] != LAYOUT_VERSION {
            continue;
        }
        if a.data[0] == DISC_HOUSE && a.data.len() == House::SIZE {
            let h = house_view(&a.data);
            if *k != house_pda(&h.config, h.house_id) {
                continue;
            }
            assert_eq!(a.lamports as u128, w.rent(House::SIZE) as u128 + h.curve.rq + h.chest as u128 + h.creator_owed as u128, "THE BOOK: house {k} holds its rent, its reserve, its chest and what it owes its creator");
            assert_eq!(h.held as u128, held.get(k).copied().unwrap_or(0), "THE BOOK: house {k}'s held is what its positions hold");
        } else if a.data[0] == DISC_FEES && a.data.len() == Fees::SIZE {
            let f = fees_view(&a.data);
            assert_eq!(a.lamports, w.rent(Fees::SIZE) + f.protocol + f.taxed + f.donated - f.paid, "THE BOOK: the fee account holds its rent and protocol + taxed + donated - paid");
        }
    }
}
