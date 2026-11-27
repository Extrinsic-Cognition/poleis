//! The cost, printed and bounded: the compute units of every instruction, in each of its shapes (a position born or standing, a raid's minute
//! that moves nothing, one that opens a stand and one that decides, a settlement, a convert that births the winner's position or adds to it, a
//! sale under the dump tax and one after it); each account's size and rent at the cluster's default rate; and the program's size. Every line
//! starts with `cost:`. The dearest of each kind in the four reference wars is printed by tests/wars.rs.
mod built;
mod common;
use common::*;
use poleis_war::law::Rules;
use poleis_wars::state::{Config, Fees, House, Position, Raid, RAID_ACTIVE, RAID_DECIDED, RAID_PROPOSED, RAID_STAND};
use solana_pubkey::Pubkey;

const T0: i64 = 1_800_000_000;

#[test]
fn the_cost() {
    let mut w = World::new(T0);
    let auth = w.wallet(1_000 * SOL);
    let c_config = w.ok(&init_config(auth, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "init_config");
    let config = config_pda(&auth);
    let c_house = w.ok(&init_house(auth, config, 0), "init_house");
    w.ok(&init_house(auth, config, 1), "init_house");
    let (crow, wolf) = (house_pda(&config, 0), house_pda(&config, 1));
    // forty buyers, one SOL each: the position born at each, its bump found by a search whose length is its address's
    let mut keys: Vec<Pubkey> = Vec::new();
    let mut born = Vec::new();
    for i in 0..40 {
        let k = w.wallet(30 * SOL);
        born.push(w.ok(&buy(k, config, if i % 2 == 0 { crow } else { wolf }, SOL, 0, None), "a buy that births its position"));
        keys.push(k);
    }
    let (b_lo, b_hi) = (*born.iter().min().unwrap(), *born.iter().max().unwrap());
    let c_buy = w.ok(&buy(keys[0], config, crow, SOL, 0, None), "a buy into a position that stands");
    let n = w.position(&crow, &keys[0]).unwrap().coins / 4;
    let c_sell = w.ok(&sell(keys[0], config, crow, n, 0), "a sale");
    // the stake: a proposal born, then the stake that opens the raid
    let raid = raid_pda(&crow, &wolf);
    let s1 = w.position(&crow, &keys[38]).unwrap().coins;
    let c_propose = w.ok(&stake(keys[38], config, crow, wolf, s1 / 4), "a stake that births the proposal");
    assert_eq!(w.raid(&raid).status, RAID_PROPOSED);
    let c_unstake = w.ok(&unstake(keys[38], crow, wolf, s1 / 8), "half of that stake taken back");
    // the crow's later buyers stake all their coins until the raid opens (two whole positions of a SOL reach 5% of its market cap)
    let mut c_open = 0;
    let mut c_stake = 0;
    let mut opener = keys[36];
    for i in [36usize, 34, 32] {
        let s2 = w.position(&crow, &keys[i]).unwrap().coins;
        let c = w.ok(&stake(keys[i], config, crow, wolf, s2), "a stake");
        if w.raid(&raid).status == RAID_ACTIVE {
            (c_open, opener) = (c, keys[i]);
            break;
        }
        c_stake = c;
    }
    assert_eq!(w.raid(&raid).status, RAID_ACTIVE);
    let start = w.raid(&raid).start;
    // minute 0 and 1 of the raid: a crank that moves nothing, a desert, changes of sides, buys in a raid
    let c_crank = w.ok(&crank(config, crow, wolf), "a crank that moves nothing");
    w.set_now(start + 60);
    let c_desert = w.ok(&desert(keys[2], config, crow, raid), "a desert");
    let c_change_new = w.ok(&change_sides(keys[1], config, raid, wolf, crow, 0), "a change of sides that births its position");
    w.ok(&buy(keys[3], config, crow, SOL, 0, Some(raid)), "keys[3] holds the crow too");
    let c_change = w.ok(&change_sides(keys[3], config, raid, wolf, crow, 0), "a change of sides into a position that stands");
    let fresh = w.wallet(100 * SOL);
    let c_buy_raid_new = w.ok(&buy(fresh, config, crow, SOL, 0, Some(raid)), "a buy in a raid, its position born");
    let c_buy_raid = w.ok(&buy(fresh, config, crow, 30 * SOL, 0, Some(raid)), "a buy in a raid into a position that stands: the crow climbs");
    let c_stand = w.ok(&crank(config, crow, wolf), "a crank that opens the last stand");
    assert_eq!(w.raid(&raid).status, RAID_STAND);
    w.set_now(start + 120);
    let c_stand_on = w.ok(&crank(config, crow, wolf), "a crank of the stand's first minute");
    w.set_now(start + 180);
    let c_decide = w.ok(&crank(config, crow, wolf), "a crank that decides");
    assert_eq!((w.raid(&raid).status, w.raid(&raid).winner), (RAID_DECIDED, 0));
    let c_settle = w.ok(&settle(config, crow, wolf), "settle");
    // the wolf's holders convert: one with no position in the crow, one with
    let holder = keys[5];
    let c_convert_new = w.ok(&convert(holder, config, raid, wolf, crow), "a convert that births the winner's position");
    let both = keys[7];
    w.ok(&buy(both, config, crow, SOL / 10, 0, None), "keys[7] holds the crow too");
    let c_convert = w.ok(&convert(both, config, raid, wolf, crow), "a convert into a position that stands");
    let c_unlock = w.ok(&unlock(opener, raid, crow), "unlock");
    let h = w.house(&crow);
    w.set_now(h.dump_start + 1);
    let n = w.position(&crow, &keys[0]).unwrap().coins / 4;
    let c_sell_tax = w.ok(&sell(keys[0], config, crow, n, 0), "a sale under the dump tax");
    w.set_now(h.dump_end);
    let c_sell_after = w.ok(&sell(keys[0], config, crow, n, 0), "a sale after the dump tax");
    // THE GAME IS ONE SWARM: a house's creator paid what it owes him (here the config's authority launched both), the publisher paid
    let c_pay_creator = w.ok(&pay_creator(crow, auth), "pay_creator");
    let c_pay_creator_rekt = w.ok(&pay_creator(wolf, auth), "pay_creator of the rekt house");
    let c_pay_publisher = w.ok(&pay_publisher(config, publisher_of(&auth)), "pay_publisher");

    println!("cost: init_config {c_config} CU; init_house {c_house} CU");
    println!("cost: buy, its position born: {b_lo} to {b_hi} CU over 40 buyers (the bump search); into a position that stands {c_buy} CU");
    println!("cost: buy in a raid, its position born {c_buy_raid_new} CU; into one that stands {c_buy_raid} CU");
    println!("cost: sell {c_sell} CU; under the dump tax {c_sell_tax} CU; after it {c_sell_after} CU");
    println!("cost: stake that births the proposal {c_propose} CU; a stake that leaves it proposed {c_stake} CU; the stake that opens the raid {c_open} CU; unstake {c_unstake} CU");
    println!("cost: desert {c_desert} CU; change_sides, its position born {c_change_new} CU, into one that stands {c_change} CU");
    println!("cost: crank that moves nothing {c_crank} CU; that opens the stand {c_stand} CU; of the stand's first minute {c_stand_on} CU; that decides {c_decide} CU");
    println!("cost: settle {c_settle} CU; convert, the winner's position born {c_convert_new} CU, into one that stands {c_convert} CU; unlock {c_unlock} CU");
    println!("cost: pay_creator {c_pay_creator} CU (of a rekt house {c_pay_creator_rekt} CU); pay_publisher {c_pay_publisher} CU");
    for (name, size) in [("Config", Config::SIZE), ("Fees", Fees::SIZE), ("House", House::SIZE), ("Position", Position::SIZE), ("Raid", Raid::SIZE)] {
        println!("cost: {name}: {size} bytes, rent {} lamports", w.rent(size));
    }
    println!("cost: the program: {} bytes", built::program_elf().len());
    let all = [c_config, c_house, b_hi, c_buy, c_sell, c_propose, c_unstake, c_stake, c_open, c_crank, c_desert, c_change_new, c_change, c_buy_raid_new, c_buy_raid, c_stand, c_stand_on, c_decide, c_settle, c_convert_new, c_convert, c_unlock, c_sell_tax, c_sell_after, c_pay_creator, c_pay_creator_rekt, c_pay_publisher];
    let dearest = *all.iter().max().unwrap();
    println!("cost: the dearest instruction {dearest} CU, of 200,000 an instruction may use");
    assert!(dearest < 60_000, "every instruction under 60,000 CU: the dearest {dearest}");
}
