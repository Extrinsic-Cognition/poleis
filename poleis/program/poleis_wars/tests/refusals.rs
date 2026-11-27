//! Every refusal, one test a family: for each instruction a missing signature, a wrong PDA, an account of another owner, discriminator or
//! version where it reads one, an account another names that is not the one it names; and each rule of the war. Each asserts the exact error
//! and that no account of the world changed (World::refuse).
mod built;
mod common;
use common::*;
use poleis_war::law::{Rules, Side};
use poleis_wars::error::WarsError::{self, *};
use poleis_wars::state::*;
#[allow(unused_imports)]
use poleis_wars::state::Config;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

const T0: i64 = 1_800_000_000;

fn e(x: WarsError) -> ProgramError {
    code(x)
}
/// the instruction with its account `i` unsigned
fn unsigned(mut ix: Instruction, i: usize) -> Instruction {
    ix.accounts[i].is_signer = false;
    ix
}
/// the instruction with its account `i` replaced by `key`
fn swap(mut ix: Instruction, i: usize, key: Pubkey) -> Instruction {
    ix.accounts[i].pubkey = key;
    ix
}
/// an account at a fresh address holding `data`, owned by `owner`
fn fake(w: &mut World, data: Vec<u8>, owner: Pubkey) -> Pubkey {
    let key = Pubkey::new_unique();
    w.put(key, Account { lamports: SOL, data, owner, executable: false, rent_epoch: 0 });
    key
}
/// the same bytes as `of`, at a fresh address this program owns (a wrong PDA), owned by the system (another owner), with another
/// discriminator, and with another layout version
fn fakes(w: &mut World, of: &Pubkey) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let d = w.data(of);
    let not_pda = fake(w, d.clone(), PROGRAM_ID);
    let other_owner = fake(w, d.clone(), Pubkey::default());
    let mut dd = d.clone();
    dd[0] = dd[0] % 5 + 1;
    let other_disc = fake(w, dd, PROGRAM_ID);
    let mut dv = d;
    dv[1] = dv[1].wrapping_add(1);
    let other_version = fake(w, dv, PROGRAM_ID);
    (not_pda, other_owner, other_disc, other_version)
}

struct Fx {
    w: World,
    auth: Pubkey,
    config: Pubkey,
    crow: Pubkey,
    wolf: Pubkey,
    /// a and c hold four SOL of the crow and of the wolf; b and d half a SOL
    a: Pubkey,
    b: Pubkey,
    c: Pubkey,
    d: Pubkey,
    raid: Pubkey,
}
fn fx_with(rules: &Rules, shield_age: u64, shield_mcap: u64) -> Fx {
    let mut w = World::new(T0);
    let auth = w.wallet(100 * SOL);
    w.ok(&init_config(auth, &rules_bytes(rules, shield_age, shield_mcap, 60)), "the config");
    let config = config_pda(&auth);
    w.ok(&init_house(auth, config, 0), "the crow");
    w.ok(&init_house(auth, config, 1), "the wolf");
    let (crow, wolf) = (house_pda(&config, 0), house_pda(&config, 1));
    let (a, b, c, d) = (w.wallet(300 * SOL), w.wallet(10 * SOL), w.wallet(300 * SOL), w.wallet(10 * SOL));
    w.ok(&buy(a, config, crow, 4 * SOL, 0, None), "a's crow");
    w.ok(&buy(b, config, crow, SOL / 2, 0, None), "b's crow");
    w.ok(&buy(c, config, wolf, 4 * SOL, 0, None), "c's wolf");
    w.ok(&buy(d, config, wolf, SOL / 2, 0, None), "d's wolf");
    Fx { w, auth, config, crow, wolf, a, b, c, d, raid: raid_pda(&crow, &wolf) }
}
fn fx() -> Fx {
    fx_with(&Rules::MEME_ROYALE, 0, 0)
}
fn coins(f: &Fx, house: &Pubkey, who: &Pubkey) -> u64 {
    f.w.position(house, who).map(|p| p.coins).unwrap_or(0)
}
/// b stakes all his crow: the raid opens
fn open(f: &mut Fx) {
    let n = coins(f, &f.crow, &f.b);
    f.w.ok(&stake(f.b, f.config, f.crow, f.wolf, n), "b's stake opens the raid");
    assert_eq!(f.w.raid(&f.raid).status, RAID_ACTIVE);
}
fn start(f: &Fx) -> i64 {
    f.w.raid(&f.raid).start
}
fn at_minute(f: &mut Fx, m: i64) {
    let s = start(f);
    f.w.set_now(s + m * 60);
}
fn crank_now(f: &mut Fx) {
    f.w.ok(&crank(f.config, f.crow, f.wolf), "crank");
}
/// nobody moves: at the end the clock gives the tie to the defender, the wolf
fn wolf_wins(f: &mut Fx) {
    let t = f.w.raid(&f.raid).minutes as i64;
    at_minute(f, t);
    crank_now(f);
    assert_eq!(f.w.raid(&f.raid).status, RAID_DECIDED);
    assert_eq!(f.w.raid(&f.raid).winner, Side::Defender.index() as u8);
}
/// a pushes the crow ten SOL at minute 1: it beats its line, the wolf's stand fails at minute 3
fn crow_wins(f: &mut Fx) {
    at_minute(f, 1);
    f.w.ok(&buy(f.a, f.config, f.crow, 10 * SOL, 0, Some(f.raid)), "a pushes the crow");
    crank_now(f);
    assert_eq!(f.w.raid(&f.raid).status, RAID_STAND);
    for m in [2, 3] {
        at_minute(f, m);
        crank_now(f);
    }
    assert_eq!((f.w.raid(&f.raid).status, f.w.raid(&f.raid).winner), (RAID_DECIDED, Side::Attacker.index() as u8));
}
fn settle_now(f: &mut Fx) {
    f.w.ok(&settle(f.config, f.crow, f.wolf), "settle");
}

#[test]
fn init_config_refusals() {
    let mut w = World::new(T0);
    let s = w.wallet(10 * SOL);
    let r = Rules::MEME_ROYALE;
    let good = rules_bytes(&r, 0, 0, 60);
    // the payer pays and the authority signs, two keys (one key is both when they are the same): each must sign
    let (payer, seal) = (w.wallet(10 * SOL), w.wallet(SOL));
    w.refuse(&unsigned(init_config_of(payer, seal, 0, publisher_of(&seal), &good), 0), ProgramError::MissingRequiredSignature, "init_config unsigned by its payer");
    w.refuse(&unsigned(init_config_of(payer, seal, 0, publisher_of(&seal), &good), 1), ProgramError::MissingRequiredSignature, "a generation its authority did not sign");
    w.refuse(&swap(init_config(s, &good), 2, config_pda(&Pubkey::new_unique())), e(InvalidPda), "a config at another authority's address");
    w.refuse(&swap(init_config(s, &good), 2, config_pda_of(&s, 1)), e(InvalidPda), "generation 0 at generation 1's address");
    w.refuse(&swap(init_config(s, &good), 3, fees_pda(&Pubkey::new_unique())), e(InvalidPda), "a fee account at another config's address");
    w.refuse(&init_config_of(s, s, 0, Pubkey::default(), &good), e(NoTreasury), "a generation that names no treasury");
    w.refuse(&init_config(s, &rules_bytes(&Rules { rt: 0, ..r }, 0, 0, 60)), e(InvalidRules), "rules the law refuses: no real coins");
    w.refuse(&init_config(s, &rules_bytes(&Rules { keep_bps: 10_001, ..r }, 0, 0, 60)), e(InvalidRules), "rules the law refuses: more than all he brought");
    w.refuse(&init_config(s, &rules_bytes(&r, 0, 0, 0)), e(InvalidRules), "a minute of no second");
    w.refuse(&Instruction::new_with_bytes(PROGRAM_ID, &[0u8; 10], init_config(s, &good).accounts), ProgramError::InvalidInstructionData, "short data");
    w.ok(&init_config(s, &good), "generation 0");
    w.refuse(&init_config(s, &good), e(ConfigStands), "a generation twice");
    let f = w.fees(&fees_pda(&config_pda(&s)));
    assert_eq!((f.protocol, f.taxed, f.donated, f.paid), (0, 0, 0, 0));
    // the next generation is founded beside it, by its own authority, paid for by another, naming its own treasury; the first stands as it was
    let first = w.data(&config_pda(&s));
    let t1 = Pubkey::new_unique();
    w.ok(&init_config_of(payer, s, 1, t1, &rules_bytes(&Rules { keep_bps: 4_000, ..r }, 0, 0, 30)), "generation 1, beside it");
    assert_eq!(w.data(&config_pda(&s)), first, "generation 0 is as it was");
    let g1 = w.data(&config_pda_of(&s, 1));
    assert_eq!((u64_at(&g1, Config::OFF_GENERATION), key_at(&g1, Config::OFF_TREASURY), key_at(&g1, Config::OFF_AUTHORITY)), (1, t1, s));
    // and a generation founded by a seal that only signs, its rents paid by the payer
    let before = w.lamports(&seal);
    w.ok(&init_config_of(payer, seal, 0, publisher_of(&seal), &good), "a generation its authority only signs");
    assert_eq!(w.lamports(&seal), before, "the authority paid nothing");
    w.refuse(&Instruction::new_with_bytes(PROGRAM_ID, &[99], vec![]), ProgramError::InvalidInstructionData, "an instruction the program does not have");
}

#[test]
fn a_donation_before_a_birth_is_booked() {
    let mut w = World::new(T0);
    let s = w.wallet(10 * SOL);
    let config = config_pda(&s);
    // lamports sent to the fee account's and a house's address before they are born: they are booked, the fee account's as donated and the
    // house's in its chest, so each still holds exactly its book
    w.put(fees_pda(&config), Account { lamports: 5_000_000, ..Account::default() });
    w.ok(&init_config(s, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "a config on a pre-funded fee account");
    assert_eq!(w.fees(&fees_pda(&config)).donated, 5_000_000 - w.rent(Fees::SIZE));
    w.put(house_pda(&config, 4), Account { lamports: 9_000_000, ..Account::default() });
    w.ok(&init_house(s, config, 4), "a house on a pre-funded address");
    assert_eq!(w.house(&house_pda(&config, 4)).chest, 9_000_000 - w.rent(House::SIZE));
}

#[test]
fn init_house_refusals() {
    let mut f = fx();
    let (auth, config, w) = (f.auth, f.config, &mut f.w);
    let s = w.wallet(10 * SOL);
    w.refuse(&unsigned(init_house(s, config, 2), 0), ProgramError::MissingRequiredSignature, "init_house unsigned");
    w.refuse(&swap(init_house(s, config, 2), 2, house_pda(&config, 3)), e(InvalidPda), "a house at another id's address");
    w.refuse(&init_house(s, config, 0), e(HouseStands), "a house twice");
    w.refuse(&init_house(auth, config, 1), e(HouseStands), "a house twice, by the config's authority");
    let (not_pda, other_owner, other_disc, other_version) = fakes(w, &config);
    w.refuse(&swap(init_house(s, not_pda, 2), 1, not_pda), e(InvalidPda), "a house under a copy of the config at another address");
    w.refuse(&init_house(s, other_owner, 2), e(InvalidAccount), "a house under a config of another owner");
    w.refuse(&init_house(s, other_disc, 2), e(InvalidAccount), "a house under a wrong discriminator");
    w.refuse(&init_house(s, other_version, 2), e(LayoutVersion), "a house under another layout version");
    w.refuse(&Instruction::new_with_bytes(PROGRAM_ID, &[poleis_wars::IX_INIT_HOUSE, 1, 2], init_house(s, config, 2).accounts), ProgramError::InvalidInstructionData, "short data");
    let h = w.house(&f.crow);
    assert_eq!((h.status, h.born, h.held > 0), (HOUSE_ACTIVE, T0, true));
}

#[test]
fn buy_refusals() {
    let mut f = fx();
    let (config, crow, b) = (f.config, f.crow, f.b);
    let w = &mut f.w;
    let s = w.wallet(400 * SOL);
    w.refuse(&unsigned(buy(s, config, crow, SOL, 0, None), 0), ProgramError::MissingRequiredSignature, "a buy unsigned");
    w.refuse(&buy(s, config, crow, 0, 0, None), e(InvalidAmount), "a buy of nothing");
    w.refuse(&buy(s, config, crow, 2, 0, None), e(NothingOut), "a buy of two lamports buys no coin");
    w.refuse(&buy(s, config, crow, SOL, u64::MAX, None), e(PastMin), "a buy past its least");
    w.refuse(&buy(s, config, crow, 300 * SOL, 0, None), e(NotEnoughTokens), "a buy past the coins the curve has left");
    let (not_pda, other_owner, other_disc, other_version) = fakes(w, &crow);
    w.refuse(&buy(s, config, not_pda, SOL, 0, None), e(InvalidPda), "a buy of a copy of the house at another address");
    w.refuse(&buy(s, config, other_owner, SOL, 0, None), e(InvalidAccount), "a buy of a house of another owner");
    w.refuse(&buy(s, config, other_disc, SOL, 0, None), e(InvalidAccount), "a buy of a wrong discriminator");
    w.refuse(&buy(s, config, other_version, SOL, 0, None), e(LayoutVersion), "a buy of another layout version");
    let other = w.wallet(10 * SOL);
    w.ok(&init_config(other, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "another config");
    w.refuse(&swap(buy(s, config, crow, SOL, 0, None), 1, config_pda(&other)), e(InvalidAccount), "a buy under a config the house does not name");
    w.refuse(&swap(buy(s, config, crow, SOL, 0, None), 4, fees_pda(&config_pda(&other))), e(InvalidAccount), "a buy paying another config's fee account");
    w.refuse(&swap(buy(s, config, crow, SOL, 0, None), 3, position_pda(&crow, &b)), e(Unauthorized), "a buy into another owner's position");
    w.refuse(&swap(buy(s, config, crow, SOL, 0, None), 3, Pubkey::new_unique()), e(InvalidPda), "a new position at an address that is not its PDA");
    // in a raid: the raid named, and only while a trade of it counts
    open(&mut f);
    let (config, crow, wolf, raid) = (f.config, f.crow, f.wolf, f.raid);
    let s = f.w.wallet(10 * SOL);
    f.w.refuse(&buy(s, config, crow, SOL, 0, None), ProgramError::NotEnoughAccountKeys, "a buy of a house in a raid without its raid");
    f.w.refuse(&buy(s, config, crow, SOL, 0, Some(wolf)), e(NotInRaid), "a buy naming another account as its raid");
    let t = f.w.raid(&raid).minutes as i64;
    at_minute(&mut f, t + 1);
    f.w.refuse(&buy(s, config, crow, SOL, 0, Some(raid)), e(RaidOver), "a buy after the raid's last minute");
    at_minute(&mut f, t);
    f.w.ok(&buy(s, config, crow, SOL, 0, Some(raid)), "a buy at the raid's last minute counts");
    crank_now(&mut f);
    f.w.refuse(&buy(s, config, crow, SOL, 0, Some(raid)), e(RaidOver), "a buy of a decided raid");
    settle_now(&mut f);
    let loser = if f.w.raid(&raid).winner == 0 { wolf } else { crow };
    f.w.refuse(&buy(s, config, loser, SOL, 0, None), e(Rekt), "a buy of the house that lost");
    let winner = if loser == crow { wolf } else { crow };
    f.w.refuse(&buy(s, config, winner, SOL, 0, Some(raid)), e(NotInRaid), "a raid named for a house in none");
    f.w.ok(&buy(s, config, winner, SOL, 0, None), "the winner lives on");
}

#[test]
fn sell_refusals() {
    let mut f = fx();
    let (config, crow, wolf, a, b) = (f.config, f.crow, f.wolf, f.a, f.b);
    let n = coins(&f, &crow, &a);
    let w = &mut f.w;
    w.refuse(&unsigned(sell(a, config, crow, 1_000, 0), 0), ProgramError::MissingRequiredSignature, "a sale unsigned");
    w.refuse(&sell(a, config, crow, 0, 0), e(InvalidAmount), "a sale of nothing");
    w.refuse(&sell(a, config, crow, n + 1, 0), e(NotEnoughCoins), "a sale past his coins");
    w.refuse(&sell(a, config, crow, 1, 0), e(NothingOut), "a sale of one coin fetches nothing");
    w.refuse(&sell(a, config, crow, n, u64::MAX), e(PastMin), "a sale under its least");
    w.refuse(&swap(sell(a, config, crow, 1_000, 0), 3, position_pda(&crow, &b)), e(Unauthorized), "a sale of another owner's coins");
    w.refuse(&swap(sell(a, config, crow, 1_000, 0), 3, position_pda(&wolf, &f.c)), e(InvalidAccount), "a sale of a position of another house");
    let (not_pda, other_owner, _, _) = fakes(w, &position_pda(&crow, &a));
    w.refuse(&swap(sell(a, config, crow, 1_000, 0), 3, not_pda), e(InvalidPda), "a sale from a copy of his position at another address");
    w.refuse(&swap(sell(a, config, crow, 1_000, 0), 3, other_owner), e(InvalidAccount), "a sale from a position of another owner");
    // RESERVE_SHORT: a house whose reserve was paid out (as a burn pays it) cannot pay the whole sale of the coins left
    let mut short = w.account(&crow);
    let cut = 3 * SOL;
    let d = &mut short.data;
    let (rq, vq) = (u64_at(d, House::OFF_RQ), u64_at(d, House::OFF_VQ));
    d[House::OFF_RQ..House::OFF_RQ + 8].copy_from_slice(&(rq - cut).to_le_bytes());
    d[House::OFF_VQ..House::OFF_VQ + 8].copy_from_slice(&(vq - cut).to_le_bytes());
    short.lamports -= cut;
    let full = w.account(&crow);
    w.put(crow, short);
    w.refuse(&sell(a, config, crow, n, 0), e(ReserveShort), "a sale past the house's reserve");
    w.put(crow, full);
    // staked coins are not his to sell; a house in a raid sells nothing
    let nb = coins(&f, &crow, &b);
    open(&mut f);
    f.w.refuse(&sell(b, config, crow, nb, 0), e(InRaid), "a sale in a raid");
    f.w.refuse(&sell(f.c, config, wolf, 1_000, 0), e(InRaid), "a sale of the defender in a raid");
    crow_wins(&mut f);
    settle_now(&mut f);
    f.w.refuse(&sell(b, config, crow, nb, 0), e(NotEnoughCoins), "a staker's coins are locked until he unlocks");
    f.w.refuse(&sell(f.c, config, wolf, 1_000, 0), e(Rekt), "a sale of the house that lost");
    f.w.ok(&unlock(b, f.raid, crow), "b unlocks");
    f.w.ok(&sell(b, config, crow, nb, 0), "and sells");
}

#[test]
fn the_dump_tax_on_a_sale_after_the_settlement() {
    let mut f = fx();
    open(&mut f);
    crow_wins(&mut f);
    settle_now(&mut f);
    let (config, crow, a) = (f.config, f.crow, f.a);
    let h = f.w.house(&crow);
    assert!(h.dump_rate_bps >= 1_000 && h.dump_end > h.dump_start, "the crow climbed: its dump tax");
    let mid = (h.dump_start + h.dump_end) / 2;
    f.w.set_now(mid);
    let rate = poleis_war::law::dump_rate_at(h.dump_rate_bps, h.dump_start, h.dump_end, mid);
    let n = coins(&f, &crow, &a) / 3;
    let (sale_of, fees0, chest0, a0) = (
        poleis_war::law::sell_on(&Rules::MEME_ROYALE, &h.curve, n as u128, rate).unwrap().1,
        f.w.fees(&fees_pda(&config)),
        h.chest,
        f.w.lamports(&a),
    );
    f.w.ok(&sell(a, config, crow, n, 0), "a sells under the dump tax");
    let (to_fees, to_chest) = poleis_war::law::tax_split(sale_of.tax);
    assert!(sale_of.tax > 0);
    assert_eq!((f.w.lamports(&a) - a0) as u128, sale_of.net(), "he gets the quote less the fees and the tax");
    assert_eq!((f.w.house(&crow).chest - chest0) as u128, to_chest, "half the tax into the house's chest");
    assert_eq!((f.w.fees(&fees_pda(&config)).taxed - fees0.taxed) as u128, to_fees, "half to the fee account");
    // once it has fallen to nothing a sale pays none
    let end = f.w.house(&crow).dump_end;
    f.w.set_now(end);
    let h = f.w.house(&crow);
    let free = poleis_war::law::sell_on(&Rules::MEME_ROYALE, &h.curve, n as u128, 0).unwrap().1;
    let a1 = f.w.lamports(&a);
    f.w.ok(&sell(a, config, crow, n, 0), "a sells after the dump tax");
    assert_eq!((f.w.lamports(&a) - a1) as u128, free.net());
}

#[test]
fn stake_refusals() {
    let mut f = fx();
    let (config, crow, wolf, a, b, c) = (f.config, f.crow, f.wolf, f.a, f.b, f.c);
    let nb = coins(&f, &crow, &b);
    let w = &mut f.w;
    w.refuse(&unsigned(stake(b, config, crow, wolf, nb), 0), ProgramError::MissingRequiredSignature, "a stake unsigned");
    w.refuse(&stake(b, config, crow, crow, nb), e(SameHouse), "a raid on itself");
    w.refuse(&stake(b, config, crow, wolf, 0), e(InvalidAmount), "a stake of nothing");
    w.refuse(&stake(b, config, crow, wolf, nb + 1), e(NotEnoughCoins), "a stake past his coins");
    w.refuse(&swap(stake(b, config, crow, wolf, nb), 5, position_pda(&crow, &a)), e(Unauthorized), "a stake of another owner's coins");
    w.refuse(&swap(stake(b, config, crow, wolf, nb), 5, position_pda(&wolf, &c)), e(InvalidAccount), "a stake of a position of the defender");
    w.refuse(&swap(stake(b, config, crow, wolf, nb), 4, raid_pda(&wolf, &crow)), e(InvalidPda), "a raid at the address of the other way round");
    // the law: a stake past the most that may be staked, an opponent outside half to twice
    let na = coins(&f, &crow, &a);
    f.w.refuse(&stake(a, config, crow, wolf, na), e(StakeCap), "a's whole crow is past 20% of its market cap");
    let small = house_pda(&config, 7);
    f.w.ok(&init_house(f.auth, config, 7), "a small house");
    let s = f.w.wallet(10 * SOL);
    f.w.ok(&buy(s, config, small, SOL / 2, 0, None), "half a SOL in it");
    f.w.refuse(&stake(b, config, crow, small, nb), e(Unfair), "an opponent under half the attacker");
    f.w.refuse(&stake(s, config, small, crow, coins(&f, &small, &s)), e(Unfair), "an opponent over twice the attacker");
    // a proposal: a stake under the need leaves it proposed; another raid of the same attacker, or of a position staked in this one, is
    // refused while it stands
    let part = nb / 10;
    f.w.ok(&stake(b, config, crow, wolf, part), "a tenth of b's crow: under the need");
    assert_eq!(f.w.raid(&f.raid).status, RAID_PROPOSED);
    assert_eq!(f.w.house(&crow).proposal, f.raid);
    let third = house_pda(&config, 8);
    f.w.ok(&init_house(f.auth, config, 8), "a third house");
    let t = f.w.wallet(10 * SOL);
    f.w.ok(&buy(t, config, third, 4 * SOL, 0, None), "as big as the wolf");
    f.w.refuse(&stake(b, config, crow, third, part), e(OtherRaid), "the crow proposes one raid at a time");
    f.w.ok(&stake(b, config, crow, wolf, nb - part), "the rest of b's crow opens it");
    assert_eq!((f.w.raid(&f.raid).status, f.w.raid(&f.raid).stakers, f.w.raid(&f.raid).staked), (RAID_ACTIVE, 1, nb));
    f.w.refuse(&stake(a, config, crow, wolf, 1_000), e(RaidOpen), "a stake on a raid already open");
    f.w.refuse(&stake(t, config, third, wolf, 1_000), e(InRaid), "a raid on a house in a raid");
    f.w.refuse(&stake(a, config, crow, third, 1_000), e(InRaid), "a raid by a house in a raid");
    // a house of another config
    let other = f.w.wallet(10 * SOL);
    f.w.ok(&init_config(other, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "another config");
    f.w.ok(&init_house(other, config_pda(&other), 0), "its house");
    f.w.refuse(&stake(t, config, third, house_pda(&config_pda(&other), 0), 1_000), e(OtherConfig), "a raid across two configs");
}

#[test]
fn the_shield() {
    // a day old and 25 SOL of market cap: a house younger, or worth less, cannot be raided
    let mut f = fx_with(&Rules::MEME_ROYALE, 86_400, 25 * SOL);
    let (config, crow, wolf, b) = (f.config, f.crow, f.wolf, f.b);
    let nb = coins(&f, &crow, &b);
    f.w.refuse(&stake(b, config, crow, wolf, nb), e(Shielded), "a house a minute old");
    f.w.set_now(T0 + 86_400);
    f.w.refuse(&stake(b, config, crow, wolf, nb), e(Shielded), "a day old, worth less than 25 SOL");
    let mut f = fx_with(&Rules::MEME_ROYALE, 86_400, 1);
    f.w.set_now(T0 + 86_399);
    f.w.refuse(&stake(f.b, f.config, f.crow, f.wolf, nb), e(Shielded), "a second short of a day");
    f.w.set_now(T0 + 86_400);
    f.w.ok(&stake(f.b, f.config, f.crow, f.wolf, nb), "a day old and worth more than a lamport: it can be raided");
}

#[test]
fn desert_and_change_sides_refusals() {
    let mut f = fx();
    let (config, crow, wolf, raid, a, b, c) = (f.config, f.crow, f.wolf, f.raid, f.a, f.b, f.c);
    f.w.refuse(&desert(a, config, crow, raid), e(NotInRaid), "a desert from a house in no raid");
    f.w.refuse(&change_sides(c, config, raid, wolf, crow, 0), e(NotInRaid), "a change of sides with no raid");
    open(&mut f);
    let w = &mut f.w;
    let s = w.wallet(10 * SOL);
    w.refuse(&unsigned(desert(a, config, crow, raid), 0), ProgramError::MissingRequiredSignature, "a desert unsigned");
    w.refuse(&desert(b, config, crow, raid), e(Staked), "a staker cannot desert");
    w.refuse(&change_sides(b, config, raid, crow, wolf, 0), e(Staked), "nor change sides");
    w.refuse(&swap(desert(s, config, crow, raid), 3, position_pda(&crow, &a)), e(Unauthorized), "a desert of another owner's coins");
    w.refuse(&desert(a, config, crow, wolf), e(NotInRaid), "a desert naming another account as the raid");
    w.refuse(&change_sides(c, config, raid, wolf, wolf, 0), e(SameHouse), "a change of sides into the same house");
    w.refuse(&change_sides(c, config, raid, wolf, crow, u64::MAX), e(PastMin), "a change of sides under its least");
    let third = house_pda(&config, 9);
    w.ok(&init_house(f.auth, config, 9), "a house in no raid");
    w.refuse(&change_sides(c, config, raid, wolf, third, 0), e(NotInRaid), "a change of sides into a house not in the raid");
    let (before, h0, na) = (w.lamports(&a), w.house(&crow), w.position(&crow, &a).unwrap().coins);
    w.ok(&desert(a, config, crow, raid), "a deserts");
    assert_eq!(w.lamports(&a) - before, 2 * SOL, "his half of the four SOL he brought");
    let h = f.w.house(&crow);
    assert_eq!(coins(&f, &crow, &a), 0, "his coins are gone");
    assert_eq!((h.burned, h.held), (h0.burned + na, h0.held - na), "burned, not held");
    assert_eq!((h.curve.vt, h.curve.rt), (h0.curve.vt, h0.curve.rt), "the curve's coins stay where they were: nothing was sold back");
    assert_eq!((h.curve.rq, h.curve.vq), (h0.curve.rq - 2 * SOL as u128, h0.curve.vq - 2 * SOL as u128), "the reserve and vq fall by what he was paid");
    f.w.refuse(&desert(a, config, crow, raid), e(NothingHeld), "a desert twice");
    f.w.ok(&change_sides(c, config, raid, wolf, crow, 0), "c goes to the crow");
    let p = f.w.position(&crow, &c).unwrap();
    assert!(p.coins > 0 && p.brought <= 2 * SOL && p.brought + 1 >= 2 * SOL, "what his half of four SOL bought, its fees and all");
    f.w.refuse(&change_sides(c, config, raid, wolf, crow, 0), e(NothingHeld), "a change of sides twice");
    let t = f.w.raid(&raid).minutes as i64;
    at_minute(&mut f, t + 1);
    f.w.refuse(&desert(f.d, config, wolf, raid), e(RaidOver), "a desert after the raid's last minute");
    f.w.refuse(&change_sides(f.d, config, raid, wolf, crow, 0), e(RaidOver), "a change of sides after it");
}

#[test]
fn crank_refusals() {
    let mut f = fx();
    let (config, crow, wolf, b) = (f.config, f.crow, f.wolf, f.b);
    let nb = coins(&f, &crow, &b);
    f.w.ok(&stake(b, config, crow, wolf, nb / 10), "a proposal");
    f.w.refuse(&crank(config, crow, wolf), e(NotOpen), "a crank of a proposal");
    f.w.ok(&stake(b, config, crow, wolf, nb - nb / 10), "it opens");
    crank_now(&mut f);
    f.w.refuse(&crank(config, crow, wolf), e(Cranked), "the same minute twice");
    at_minute(&mut f, 1);
    crank_now(&mut f);
    let third = house_pda(&config, 9);
    f.w.ok(&init_house(f.auth, config, 9), "a third house");
    at_minute(&mut f, 2);
    f.w.refuse(&swap(crank(config, crow, wolf), 2, third), e(InvalidAccount), "a crank naming a house not the raid's");
    let (not_pda, other_owner, other_disc, other_version) = fakes(&mut f.w, &f.raid);
    f.w.refuse(&swap(crank(config, crow, wolf), 1, not_pda), e(InvalidPda), "a copy of the raid at another address");
    f.w.refuse(&swap(crank(config, crow, wolf), 1, other_owner), e(InvalidAccount), "a raid of another owner");
    f.w.refuse(&swap(crank(config, crow, wolf), 1, other_disc), e(InvalidAccount), "a wrong discriminator");
    f.w.refuse(&swap(crank(config, crow, wolf), 1, other_version), e(LayoutVersion), "another layout version");
    let other = f.w.wallet(10 * SOL);
    f.w.ok(&init_config(other, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "another config");
    f.w.refuse(&swap(crank(config, crow, wolf), 0, config_pda(&other)), e(InvalidAccount), "a crank under a config the raid does not name");
    wolf_wins(&mut f);
    at_minute(&mut f, 400);
    f.w.refuse(&crank(config, crow, wolf), e(Decided), "a crank of a raid decided");
}

#[test]
fn settle_convert_and_unlock_refusals() {
    let mut f = fx();
    open(&mut f);
    let (config, crow, wolf, raid, b, c, d) = (f.config, f.crow, f.wolf, f.raid, f.b, f.c, f.d);
    f.w.refuse(&settle(config, crow, wolf), e(NotDecided), "a settle of an open raid");
    f.w.refuse(&convert(c, config, raid, wolf, crow), e(NotSettled), "a convert before the settlement");
    f.w.refuse(&unlock(b, raid, crow), e(NotSettled), "an unlock before the settlement");
    crow_wins(&mut f);
    let third = house_pda(&config, 9);
    f.w.ok(&init_house(f.auth, config, 9), "a third house");
    f.w.refuse(&swap(settle(config, crow, wolf), 3, third), e(InvalidAccount), "a settle naming a house not the raid's");
    let other = f.w.wallet(10 * SOL);
    f.w.ok(&init_config(other, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "another config");
    f.w.refuse(&swap(settle(config, crow, wolf), 4, fees_pda(&config_pda(&other))), e(InvalidAccount), "a settle paying another config's fee account");
    settle_now(&mut f);
    f.w.refuse(&settle(config, crow, wolf), e(AlreadySettled), "a settle twice");
    // the wolf lost: its holders convert, once each
    f.w.refuse(&convert(f.a, config, raid, crow, wolf), e(NotTheLoser), "a convert of the winner");
    f.w.refuse(&swap(convert(c, config, raid, wolf, crow), 4, position_pda(&wolf, &d)), e(Unauthorized), "a convert of another owner's coins");
    f.w.refuse(&unsigned(convert(c, config, raid, wolf, crow), 0), ProgramError::MissingRequiredSignature, "a convert unsigned");
    f.w.ok(&convert(c, config, raid, wolf, crow), "c converts");
    f.w.refuse(&convert(c, config, raid, wolf, crow), e(InvalidAccount), "a convert twice: his position in the loser is closed");
    f.w.ok(&convert(d, config, raid, wolf, crow), "d converts");
    let r = f.w.raid(&raid);
    assert!(r.coins_bought - r.converted <= 2, "two holders, at most a coin of dust each");
    assert_eq!(f.w.house(&wolf).held, 0, "nobody holds the loser any more");
    // the crow won: its staker unlocks, once; nobody else does
    f.w.refuse(&unlock(f.a, raid, crow), e(NotStaked), "an unlock of a holder who staked nothing");
    f.w.refuse(&swap(unlock(f.a, raid, crow), 2, position_pda(&crow, &b)), e(Unauthorized), "an unlock of another owner's stake");
    f.w.refuse(&unsigned(unlock(b, raid, crow), 0), ProgramError::MissingRequiredSignature, "an unlock unsigned");
    f.w.ok(&unlock(b, raid, crow), "b unlocks");
    f.w.refuse(&unlock(b, raid, crow), e(NotStaked), "an unlock twice");
    // when the attacker loses its stakers convert, a tenth better, and unlock nothing
    let mut g = fx();
    open(&mut g);
    wolf_wins(&mut g);
    g.w.ok(&settle(g.config, g.crow, g.wolf), "settle");
    g.w.refuse(&unlock(g.b, g.raid, g.crow), e(NotTheWinner), "an unlock when the attacker lost");
    let r = g.w.raid(&g.raid);
    let (na, nb) = (coins(&g, &g.crow, &g.a), coins(&g, &g.crow, &g.b));
    g.w.ok(&convert(g.b, g.config, g.raid, g.crow, g.wolf), "the staker converts");
    g.w.ok(&convert(g.a, g.config, g.raid, g.crow, g.wolf), "the holder converts");
    let weights = poleis_war::law::weights_of(&Rules::MEME_ROYALE, (na + nb) as u128, nb as u128).unwrap();
    assert_eq!(r.weights, weights, "the weights fixed at the settlement");
    let share = |n: u64, staked: u64| poleis_war::law::share_of(r.coins_bought as u128, poleis_war::law::weight_of(&Rules::MEME_ROYALE, n as u128, staked as u128).unwrap(), weights).unwrap() as u64;
    assert_eq!(g.w.position(&g.wolf, &g.b).unwrap().coins, share(nb, nb), "the staker's share, a tenth better");
    assert_eq!(g.w.position(&g.wolf, &g.a).unwrap().coins, share(na, 0));
    // a deserter's emptied position holds nothing to convert
    let mut h = fx();
    open(&mut h);
    h.w.ok(&desert(h.a, h.config, h.crow, h.raid), "a deserts the crow");
    wolf_wins(&mut h);
    h.w.ok(&settle(h.config, h.crow, h.wolf), "settle");
    h.w.refuse(&convert(h.a, h.config, h.raid, h.crow, h.wolf), e(NothingHeld), "a deserter has nothing to convert");
}

#[test]
fn unstake_takes_a_stake_back_from_a_proposal_and_from_no_other_phase() {
    let mut f = fx();
    let (config, crow, wolf, raid, a, b, c) = (f.config, f.crow, f.wolf, f.raid, f.a, f.b, f.c);
    let nb = coins(&f, &crow, &b);
    let part = nb / 10;
    f.w.ok(&stake(b, config, crow, wolf, part), "a tenth of b's crow: a proposal");
    let w = &mut f.w;
    w.refuse(&unsigned(unstake(b, crow, wolf, part), 0), ProgramError::MissingRequiredSignature, "an unstake unsigned");
    w.refuse(&unstake(b, crow, wolf, 0), e(InvalidAmount), "an unstake of nothing");
    w.refuse(&unstake(b, crow, wolf, part + 1), e(NotEnoughCoins), "more than he staked");
    w.refuse(&unstake(a, crow, wolf, 1), e(NotStaked), "a holder who staked nothing");
    w.refuse(&swap(unstake(a, crow, wolf, 1), 3, position_pda(&crow, &b)), e(Unauthorized), "another owner's stake");
    w.refuse(&swap(unstake(c, crow, wolf, 1), 3, position_pda(&wolf, &c)), e(InvalidAccount), "a position of the defender");
    w.refuse(&swap(unstake(b, crow, wolf, part), 1, wolf), e(InvalidAccount), "a house that is not the raid's attacker");
    let (not_pda, other_owner, _, other_version) = fakes(w, &raid);
    w.refuse(&swap(unstake(b, crow, wolf, part), 2, not_pda), e(InvalidPda), "a copy of the raid at another address");
    w.refuse(&swap(unstake(b, crow, wolf, part), 2, other_owner), e(InvalidAccount), "a raid of another owner");
    w.refuse(&swap(unstake(b, crow, wolf, part), 2, other_version), e(LayoutVersion), "a raid of another layout version");
    // half of it back, then the rest: the position leaves the raid, and the empty proposal leaves the crow free to propose another
    w.ok(&unstake(b, crow, wolf, part / 2), "half of his stake back");
    let (p, r) = (w.position(&crow, &b).unwrap(), w.raid(&raid));
    assert_eq!((p.staked, p.raid, r.staked, r.stakers, r.status), (part - part / 2, raid, part - part / 2, 1, RAID_PROPOSED));
    assert_eq!(w.house(&crow).proposal, raid, "the proposal stands while a stake is in it");
    w.ok(&unstake(b, crow, wolf, part - part / 2), "the rest of it back");
    let (p, r) = (w.position(&crow, &b).unwrap(), w.raid(&raid));
    assert_eq!((p.staked, p.raid, r.staked, r.stakers, r.status), (0, Pubkey::default(), 0, 0, RAID_PROPOSED));
    assert_eq!(w.house(&crow).proposal, Pubkey::default(), "an empty proposal leaves the crow free");
    w.refuse(&unstake(b, crow, wolf, 1), e(NotStaked), "an unstake twice");
    // his coins are his again: he sells a coin's worth, and the crow may propose against another house
    w.ok(&sell(b, config, crow, nb / 2, 0), "b sells half of what he staked and took back");
    let third = house_pda(&config, 8);
    w.ok(&init_house(f.auth, config, 8), "a third house");
    let t = w.wallet(10 * SOL);
    w.ok(&buy(t, config, third, 4 * SOL, 0, None), "as big as the wolf");
    w.ok(&stake(b, config, crow, third, (nb - nb / 2) / 10), "a proposal against another house");
    assert_eq!(w.house(&crow).proposal, raid_pda(&crow, &third));
    w.ok(&unstake(b, crow, third, (nb - nb / 2) / 10), "taken back again");
    // the same proposal takes stakes again and opens; once open, in its stand, decided and settled, no stake comes back
    let nb = coins(&f, &crow, &b);
    open_with(&mut f, nb);
    f.w.refuse(&unstake(b, crow, wolf, 1), e(RaidOpen), "an unstake of an open raid");
    at_minute(&mut f, 1);
    f.w.ok(&buy(f.a, config, crow, 10 * SOL, 0, Some(raid)), "a pushes the crow");
    crank_now(&mut f);
    assert_eq!(f.w.raid(&raid).status, RAID_STAND);
    f.w.refuse(&unstake(b, crow, wolf, 1), e(RaidOpen), "an unstake in the last stand");
    for m in [2, 3] {
        at_minute(&mut f, m);
        crank_now(&mut f);
    }
    assert_eq!(f.w.raid(&raid).status, RAID_DECIDED);
    f.w.refuse(&unstake(b, crow, wolf, 1), e(RaidOpen), "an unstake of a decided raid");
    settle_now(&mut f);
    f.w.refuse(&unstake(b, crow, wolf, 1), e(RaidOpen), "an unstake of a settled raid: the winner's stakers unlock");
    f.w.ok(&unlock(b, raid, crow), "b unlocks");
}

/// b stakes `n` of his crow against the wolf: the proposal that stands takes it and opens
fn open_with(f: &mut Fx, n: u64) {
    f.w.ok(&stake(f.b, f.config, f.crow, f.wolf, n), "b's stake opens the raid");
    assert_eq!(f.w.raid(&f.raid).status, RAID_ACTIVE);
}

/// the bytes of `of` with its layout version set to 1, the first devnet war's, put back at its own address
fn as_version_1(w: &mut World, of: &Pubkey) {
    let mut a = w.account(of);
    a.data[1] = 1;
    w.put(*of, a);
}

#[test]
fn pay_creator_and_pay_publisher_refusals() {
    let mut f = fx();
    let (config, crow, wolf, auth) = (f.config, f.crow, f.wolf, f.auth);
    // anyone pays: no signature is asked; the account named must be the house's creator, and the generation's treasury
    let s = f.w.wallet(SOL);
    let owed = f.w.house(&crow).creator_owed;
    assert!(owed > 0, "the buys of a and b owe the crow's creator");
    f.w.refuse(&pay_creator(crow, s), e(InvalidAccount), "a creator's fee to another address");
    f.w.refuse(&pay_creator(crow, publisher_of(&auth)), e(InvalidAccount), "a creator's fee to the publisher");
    let (not_pda, other_owner, other_disc, other_version) = fakes(&mut f.w, &crow);
    f.w.refuse(&pay_creator(not_pda, auth), e(InvalidPda), "a copy of the house at another address");
    f.w.refuse(&pay_creator(other_owner, auth), e(InvalidAccount), "a house of another owner");
    f.w.refuse(&pay_creator(other_disc, auth), e(InvalidAccount), "a wrong discriminator");
    f.w.refuse(&pay_creator(other_version, auth), e(LayoutVersion), "another layout version");
    let before = f.w.lamports(&auth);
    f.w.ok(&pay_creator(crow, auth), "the crow's creator paid");
    assert_eq!(f.w.lamports(&auth) - before, owed, "exactly what the house owed him");
    f.w.refuse(&pay_creator(crow, auth), e(NothingOwed), "a creator paid twice");
    let publisher = publisher_of(&auth);
    let holds = f.w.lamports(&fees_pda(&config)) - f.w.rent(Fees::SIZE);
    f.w.refuse(&pay_publisher(config, s), e(InvalidAccount), "the publisher's fees to another address");
    f.w.refuse(&pay_publisher(config, auth), e(InvalidAccount), "the publisher's fees to the config's authority");
    let other = f.w.wallet(10 * SOL);
    f.w.ok(&init_config(other, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "another generation");
    f.w.refuse(&pay_publisher(config_pda(&other), publisher), e(InvalidAccount), "another generation's config for this one's treasury");
    let mut ix = pay_publisher(config, publisher);
    ix.accounts[1].pubkey = fees_pda(&config_pda(&other));
    f.w.refuse(&ix, e(InvalidAccount), "another generation's fee account");
    let (cnp, coo, cod, cov) = fakes(&mut f.w, &config);
    f.w.refuse(&pay_publisher(cnp, publisher), e(InvalidPda), "a copy of the config at another address");
    f.w.refuse(&pay_publisher(coo, publisher), e(InvalidAccount), "a config of another owner");
    f.w.refuse(&pay_publisher(cod, publisher), e(InvalidAccount), "a wrong discriminator");
    f.w.refuse(&pay_publisher(cov, publisher), e(LayoutVersion), "a config of another layout version");
    let before = f.w.lamports(&publisher);
    f.w.ok(&pay_publisher(config, publisher), "the publisher paid");
    assert_eq!(f.w.lamports(&publisher) - before, holds, "everything the fee account held beyond its rent");
    let fv = f.w.fees(&fees_pda(&config));
    assert_eq!((fv.paid, fv.protocol + fv.taxed + fv.donated), (holds, holds));
    f.w.refuse(&pay_publisher(config, publisher), e(NothingOwed), "the publisher paid twice");
    // a rekt house still pays its creator what it owes him; a war's fees pay the publisher again
    open(&mut f);
    crow_wins(&mut f);
    settle_now(&mut f);
    assert_eq!(f.w.house(&wolf).status, HOUSE_REKT);
    let owed = f.w.house(&wolf).creator_owed;
    let before = f.w.lamports(&auth);
    f.w.ok(&pay_creator(wolf, auth), "the rekt wolf pays its creator");
    assert_eq!(f.w.lamports(&auth) - before, owed);
    f.w.ok(&pay_publisher(config, publisher), "and the war's fees pay the publisher");
    // THE FIRST DEVNET WAR'S ACCOUNTS ARE VERSION 1: refused, whatever they hold
    f.w.keep_book = false;
    as_version_1(&mut f.w, &crow);
    f.w.refuse(&pay_creator(crow, auth), e(LayoutVersion), "a version 1 house");
    as_version_1(&mut f.w, &config);
    f.w.refuse(&pay_publisher(config, publisher), e(LayoutVersion), "a version 1 config");
    as_version_1(&mut f.w, &fees_pda(&config_pda(&other)));
    f.w.refuse(&pay_publisher(config_pda(&other), publisher_of(&other)), e(LayoutVersion), "a version 1 fee account");
    let s2 = f.w.wallet(10 * SOL);
    f.w.refuse(&buy(s2, config, crow, SOL, 0, None), e(LayoutVersion), "a buy of a version 1 house under a version 1 config");
}

/// the system program's own transfer (its instruction 2), anyone's: `lamports` from a wallet to any address
fn system_transfer(from: Pubkey, to: Pubkey, lamports: u64) -> Instruction {
    let mut d = 2u32.to_le_bytes().to_vec();
    d.extend_from_slice(&lamports.to_le_bytes());
    Instruction::new_with_bytes(system_id(), &d, vec![AccountMeta::new(from, true), AccountMeta::new(to, false)])
}

/// A GIFT CANNOT STOP THE PUBLISHER'S PAYOUT: anyone may send lamports straight to a generation's fee account (a system transfer needs no
/// word of the program's), so a payout that asked its lamports to equal its book could be stopped for good by one lamport. The gift is
/// booked as donated and paid on with the fees; a fee account that holds less than its book pays nothing, and a transfer of the difference
/// mends it.
#[test]
fn a_gift_to_the_fee_account_is_paid_on_to_the_publisher() {
    let mut f = fx();
    let (config, auth) = (f.config, f.auth);
    let (fees, publisher) = (fees_pda(&config), publisher_of(&auth));
    let f0 = f.w.fees(&fees);
    let booked = f0.protocol + f0.taxed + f0.donated - f0.paid;
    assert!(booked > 0, "the fixture's four buys owe the publisher");
    // a fee account one lamport short of its book pays nothing
    f.w.keep_book = false;
    let mut short = f.w.account(&fees);
    short.lamports -= 1;
    f.w.put(fees, short);
    f.w.refuse(&pay_publisher(config, publisher), e(MathOverflow), "a fee account short of its book");
    // anyone's transfer mends it, and anyone's gift of 1,000 lamports more is a gift
    let giver = f.w.wallet(SOL);
    f.w.ok(&system_transfer(giver, fees, 1), "the lamport it was short");
    f.w.ok(&system_transfer(giver, fees, 1_000), "a gift of 1,000 lamports, by the system program");
    f.w.keep_book = true;
    let before = f.w.lamports(&publisher);
    f.w.ok(&pay_publisher(config, publisher), "the publisher paid, the gift with the fees");
    assert_eq!(f.w.lamports(&publisher) - before, booked + 1_000, "the fees and the gift, to the lamport");
    let f1 = f.w.fees(&fees);
    assert_eq!((f1.protocol, f1.taxed), (f0.protocol, f0.taxed), "the fees are the trades' alone");
    assert_eq!((f1.donated, f1.paid), (f0.donated + 1_000, f0.paid + booked + 1_000), "the gift booked as donated, all of it paid");
    assert_eq!(f.w.lamports(&fees), f.w.rent(Fees::SIZE), "the fee account keeps its rent alone");
    f.w.refuse(&pay_publisher(config, publisher), e(NothingOwed), "nothing left to pay");
}
