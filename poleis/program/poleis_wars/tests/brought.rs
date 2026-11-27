//! WHAT HE STILL HAS IN (the coordinator's review of Nov 24 2026: `brought` never fell, so his half could be dodged). His rule: a deserter
//! leaves with half of what he brought, the SOL he still has in the house. A sale takes its share of what he brought out with it (the law's
//! `brought_after_sale`); a desert or a change of sides ends it. The two holes it closes, each desert paid at most `keep_bps` of what that
//! holder still had in: (a) ten SOL in, all of it sold, one SOL back in; (b) a desert, a buy back in the same raid, a second desert. And a
//! partial sale halving it.
mod built;
mod common;
use common::*;
use embertide_war::law::{brought_after_sale, burn_on, Rules, BPS};
use poleis_wars::state::RAID_ACTIVE;
use solana_pubkey::Pubkey;

const T0: i64 = 1_800_000_000;
const R: Rules = Rules::MEME_ROYALE;

struct Fx {
    w: World,
    config: Pubkey,
    crow: Pubkey,
    wolf: Pubkey,
    b: Pubkey,
    c: Pubkey,
    raid: Pubkey,
}
/// a config with no shield; the crow (a four SOL, b the staker half a SOL) and the wolf (c four, d half)
fn fx() -> Fx {
    let mut w = World::new(T0);
    let auth = w.wallet(100 * SOL);
    w.ok(&init_config(auth, &rules_bytes(&R, 0, 0, 60)), "the config");
    let config = config_pda(&auth);
    w.ok(&init_house(auth, config, 0), "the crow");
    w.ok(&init_house(auth, config, 1), "the wolf");
    let (crow, wolf) = (house_pda(&config, 0), house_pda(&config, 1));
    let (a, b, c, d) = (w.wallet(300 * SOL), w.wallet(10 * SOL), w.wallet(300 * SOL), w.wallet(10 * SOL));
    w.ok(&buy(a, config, crow, 4 * SOL, 0, None), "a's crow");
    w.ok(&buy(b, config, crow, SOL / 2, 0, None), "b's crow");
    w.ok(&buy(c, config, wolf, 4 * SOL, 0, None), "c's wolf");
    w.ok(&buy(d, config, wolf, SOL / 2, 0, None), "d's wolf");
    Fx { w, config, crow, wolf, b, c, raid: raid_pda(&crow, &wolf) }
}
fn open(f: &mut Fx) {
    let n = f.w.position(&f.crow, &f.b).unwrap().coins;
    f.w.ok(&stake(f.b, f.config, f.crow, f.wolf, n), "b's stake opens the raid");
    assert_eq!(f.w.raid(&f.raid).status, RAID_ACTIVE);
}
/// `who` deserts `house`: what he was paid, held to the law's burn on the house as it stood and to his half of what he still had in
fn desert_held(f: &mut Fx, who: Pubkey, house: Pubkey, what: &str) -> u64 {
    let (h, p, before) = (f.w.house(&house), f.w.position(&house, &who).unwrap(), f.w.lamports(&who));
    let (_, law_paid) = burn_on(&R, &h.curve, p.coins as u128, p.brought as u128).unwrap();
    f.w.ok(&desert(who, f.config, house, f.raid), what);
    let paid = f.w.lamports(&who) - before;
    assert_eq!(paid as u128, law_paid, "{what}: the law's burn");
    assert!(paid as u128 <= p.brought as u128 * R.keep_bps as u128 / BPS, "{what}: at most his half of the {} he still had in", p.brought);
    assert_eq!(f.w.position(&house, &who).unwrap().brought, 0, "{what}: a desert ends what he brought");
    paid
}

#[test]
fn ten_in_all_sold_one_back_in_deserts_with_half_of_the_one() {
    let mut f = fx();
    let (config, crow) = (f.config, f.crow);
    let h = f.w.wallet(30 * SOL);
    f.w.ok(&buy(h, config, crow, 10 * SOL, 0, None), "ten SOL in, outside a raid");
    let p = f.w.position(&crow, &h).unwrap();
    assert!(p.brought <= 10 * SOL && p.brought + 1 >= 10 * SOL);
    f.w.ok(&sell(h, config, crow, p.coins, 0), "all of it sold");
    assert_eq!(f.w.position(&crow, &h).unwrap().brought, 0, "nothing still in");
    let before = f.w.lamports(&h);
    f.w.ok(&buy(h, config, crow, SOL, 0, None), "one SOL back in");
    let p = f.w.position(&crow, &h).unwrap();
    assert_eq!(p.brought, before - f.w.lamports(&h), "what he still has in is what the one SOL buy paid (his position stood: no rent)");
    assert!(p.brought <= SOL && p.brought + 1 >= SOL);
    open(&mut f);
    let paid = desert_held(&mut f, h, crow, "he deserts");
    assert_eq!(paid, p.brought / 2, "half of the one SOL, not the full worth of his coins (under the old rule his 11 SOL would have lifted the cap)");
}

#[test]
fn a_second_desert_in_the_same_raid_is_paid_half_of_what_he_bought_back() {
    let mut f = fx();
    let (config, crow, raid) = (f.config, f.crow, f.raid);
    let h = f.w.wallet(30 * SOL);
    f.w.ok(&buy(h, config, crow, 3 * SOL, 0, None), "three SOL in");
    open(&mut f);
    let first = desert_held(&mut f, h, crow, "the first desert");
    assert_eq!(first, 3 * SOL / 2, "half of the three");
    f.w.set_now(f.w.raid(&raid).start + 60);
    f.w.ok(&buy(h, config, crow, SOL, 0, Some(raid)), "one SOL back in, in the same raid");
    let p = f.w.position(&crow, &h).unwrap();
    assert!(p.brought <= SOL && p.brought + 1 >= SOL, "what he brought is the one SOL alone: the first desert ended the three");
    let second = desert_held(&mut f, h, crow, "the second desert");
    assert!(second as u128 <= p.brought as u128 / 2 && second <= SOL / 2, "half of the one SOL at most, not of the four he once brought: {second}");
}

#[test]
fn a_partial_sale_takes_its_share_of_what_he_brought_and_a_change_of_sides_ends_it() {
    let mut f = fx();
    let (config, crow, wolf, raid, c) = (f.config, f.crow, f.wolf, f.raid, f.c);
    let g = f.w.wallet(30 * SOL);
    f.w.ok(&buy(g, config, crow, 2 * SOL, 0, None), "two SOL in");
    let p = f.w.position(&crow, &g).unwrap();
    f.w.ok(&sell(g, config, crow, p.coins / 2, 0), "half of his coins sold");
    let q = f.w.position(&crow, &g).unwrap();
    assert_eq!(q.brought as u128, brought_after_sale(p.brought as u128, p.coins as u128, (p.coins / 2) as u128).unwrap());
    assert!(q.brought.abs_diff(p.brought / 2) <= 1, "half of what he brought: {} of {}", q.brought, p.brought);
    // a sale of what he holds is refused past it, so what he brought never falls below nothing
    f.w.refuse(&sell(g, config, crow, q.coins + 1, 0), code(poleis_wars::error::WarsError::NotEnoughCoins), "a sale past his coins");
    // a change of sides ends what he brought into the house he leaves; what he brings into the other is what his change bought
    open(&mut f);
    f.w.ok(&change_sides(c, config, raid, wolf, crow, 0), "c goes to the crow");
    assert_eq!(f.w.position(&wolf, &c).unwrap().brought, 0, "nothing of c still in the wolf");
    let to = f.w.position(&crow, &c).unwrap();
    assert!(to.brought > 0 && to.brought <= 2 * SOL, "in the crow, what his half of four SOL bought: {}", to.brought);
}
