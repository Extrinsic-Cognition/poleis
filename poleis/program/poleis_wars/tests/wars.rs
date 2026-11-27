//! THE FOUR REFERENCE WARS ON THE CHAIN, instruction by instruction, beside the law's whole war (poleis-war's `War`, the same steps): the
//! forty buys, the stakes until the raid opens, then minute by minute the clock warped to the minute, the minute's buys, deserts and changes
//! of sides in the model's order, the stand's reserve in its first minute, and one crank; then the settlement, every loser holder's convert
//! and the winner's stakers' unlock. After every instruction the program's houses are the law's to the unit (curve, held, burned, chest),
//! the raid's phase is the law's after every crank, and THE BOOK holds (common::book). At the end every number is held to the owner's
//! reference (poleis-war/tests/reference/reference-in-sol.json): the winner, how and the minute, every settled integer, every fighter's
//! holds, backed_lamports and took_lamports.
mod built;
mod common;
use common::*;
use poleis_war::law::{backed_of, Phase, Rules, Side, SOL as LAW_SOL};
use poleis_war::war::{how_text, the_four, Kind, Scenario, War, HOUSE, N};
use poleis_wars::state::{HOUSE_ACTIVE, HOUSE_IN_RAID, HOUSE_REKT, RAID_ACTIVE, RAID_PROPOSED, RAID_SETTLED};
use serde_json::Value;
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;

/// the world's clock at the start, unix seconds
const T0: i64 = 1_800_000_000;

fn reference() -> Vec<Value> {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../poleis-war/tests/reference/reference-in-sol.json")).expect("the reference");
    serde_json::from_str::<Value>(&text).unwrap().as_array().unwrap().clone()
}
fn int(v: &Value) -> u128 {
    v.as_u64().unwrap_or_else(|| panic!("an integer: {v}")) as u128
}

/// the program's houses are the law's, to the unit
fn same(w: &World, law: &War, houses: &[Pubkey; 2], what: &str) {
    for s in [Side::Attacker, Side::Defender] {
        let h = w.house(&houses[s.index()]);
        let i = s.index();
        assert_eq!(h.curve, law.curves[i], "{what}: {}'s curve", HOUSE[i]);
        assert_eq!((h.held as u128, h.burned as u128, h.chest as u128), (law.held[i], law.burned[i], law.chests[i]), "{what}: {}'s held, burned, chest", HOUSE[i]);
    }
}

/// what a war on the chain ended with
struct Out {
    winner: Side,
    how: String,
    minutes: u32,
    need: u64,
    m0: [u64; 2],
    mcap_end: [u64; 2],
    pot_in: u64,
    bought_with: u64,
    coins_bought: u64,
    to_chest: u64,
    winner_reserve: u128,
    winner_chest: u64,
    all_held: u64,
    stakers: Vec<String>,
    dump: (u32, i64),
    fighters: BTreeMap<String, (u128, u128, u128)>,
    cu: BTreeMap<&'static str, (u64, u64, u32)>,
    instructions: u32,
    /// what each house's creator and the publisher were paid
    paid_creators: [u128; 2],
    paid_publisher: u128,
}

/// one war replayed on the chain beside the law's war, at `minute_seconds` a minute
fn replay(s: &Scenario, minute_seconds: u32) -> Out {
    let rules = Rules::MEME_ROYALE;
    let mut w = World::new(T0);
    let mut meter = Meter::default();
    let run = |w: &mut World, ix: &solana_instruction::Instruction, kind: &'static str, meter: &mut Meter| -> u64 {
        let c = w.ok(ix, kind);
        meter.note(kind, c);
        c
    };
    let auth = w.wallet(100 * SOL);
    run(&mut w, &init_config(auth, &rules_bytes(&rules, 0, 0, minute_seconds)), "init_config", &mut meter);
    let config = config_pda(&auth);
    let houses = [house_pda(&config, 0), house_pda(&config, 1)];
    // THE GAME IS ONE SWARM: each house is launched by a player of its own, its creator
    let creators = [w.wallet(SOL), w.wallet(SOL)];
    run(&mut w, &init_house(creators[0], config, 0), "init_house", &mut meter);
    run(&mut w, &init_house(creators[1], config, 1), "init_house", &mut meter);
    assert_eq!((w.house(&houses[0]).creator, w.house(&houses[1]).creator), (creators[0], creators[1]), "a house keeps who launched it");
    let mut law = War::new(rules);
    let mut keys: BTreeMap<String, Pubkey> = BTreeMap::new();

    // the forty, one SOL each, the crow's and the wolf's in turn
    for i in 0..N {
        for h in [Side::Attacker, Side::Defender] {
            let name = format!("{}{i}", HOUSE[h.index()]);
            let k = w.wallet(3 * SOL);
            keys.insert(name.clone(), k);
            run(&mut w, &buy(k, config, houses[h.index()], SOL, 0, None), "buy", &mut meter);
            law.enter(&name, h, LAW_SOL).unwrap();
            same(&w, &law, &houses, &format!("{name} came in"));
            assert_eq!(w.position(&houses[h.index()], &k).unwrap().brought, SOL, "{name} brought one SOL");
        }
    }
    // the stake: the crow's last ones in, all their coins, crow19 first and down, until the raid opens
    let raid = raid_pda(&houses[0], &houses[1]);
    for i in (10..N).rev() {
        if law.opening.is_some() {
            break;
        }
        let name = format!("crow{i}");
        let k = keys[&name];
        let coins = w.position(&houses[0], &k).unwrap().coins;
        run(&mut w, &stake(k, config, houses[0], houses[1], coins), "stake", &mut meter);
        let opened = law.stake(&name).unwrap();
        let r = w.raid(&raid);
        assert_eq!(r.status, if opened { RAID_ACTIVE } else { RAID_PROPOSED }, "{name}'s stake: the raid opens with the law");
        assert_eq!(r.staked as u128, law.staked);
        let p = w.position(&houses[0], &k).unwrap();
        assert_eq!((p.staked, p.raid), (coins, raid), "{name} staked all his coins in the raid");
    }
    let o = law.opening.expect("the raid opened");
    let r = w.raid(&raid);
    assert_eq!((r.m0[0] as u128, r.m0[1] as u128, r.need as u128, r.minutes), (o.m0[0], o.m0[1], o.need, o.minutes), "the opening is the law's");
    assert_eq!((r.start, r.minute_seconds, r.seconds), (w.now(), minute_seconds, o.minutes as u64 * minute_seconds as u64));
    assert_eq!((w.house(&houses[0]).status, w.house(&houses[1]).status), (HOUSE_IN_RAID, HOUSE_IN_RAID));
    let start = r.start;
    let t = o.minutes;

    // minute by minute: the clock, the minute's events in the model's order, the stand's reserve, one crank
    let mut events = (s.script)(t);
    events.sort();
    let mut took: BTreeMap<String, u128> = BTreeMap::new();
    for m in 0..=t + rules.stand_minutes {
        w.set_now(start + m as i64 * minute_seconds as i64);
        for e in events.iter().filter(|e| e.minute == m) {
            match e.kind {
                Kind::Reinforce => {
                    let k = w.wallet(3 * SOL);
                    keys.insert(e.who.clone(), k);
                    let got = w.run(&buy(k, config, houses[e.house.index()], SOL, 0, Some(raid)));
                    let refused = law.refused.len();
                    law.come_in(&e.who, e.house);
                    assert_eq!(got.is_err(), law.refused.len() > refused, "{}: the program and the law refuse alike", e.who);
                    if let Ok(c) = got {
                        meter.note("buy in a raid", c);
                    }
                }
                Kind::Cross if !law.is_staker(&e.who) => {
                    let k = keys[&e.who];
                    run(&mut w, &change_sides(k, config, raid, houses[1], houses[0], 0), "change_sides", &mut meter);
                    law.cross(&e.who, Side::Defender).unwrap();
                }
                Kind::Desert if !law.is_staker(&e.who) => {
                    let k = keys[&e.who];
                    let src = if w.position(&houses[0], &k).map(|p| p.coins).unwrap_or(0) > 0 { 0 } else { 1 };
                    let before = w.lamports(&k);
                    run(&mut w, &desert(k, config, houses[src], raid), "desert", &mut meter);
                    let paid = law.desert(&e.who).unwrap();
                    assert_eq!((w.lamports(&k) - before) as u128, paid, "{}: the desert paid him the law's", e.who);
                    took.insert(e.who.clone(), paid);
                }
                _ => {}
            }
            same(&w, &law, &houses, &format!("{} at minute {m}", e.who));
        }
        if let (Some(Phase::Stand { side, ends, .. }), Some((rs, n))) = (w.raid(&raid).phase(), s.reserve) {
            if rs == side && m + rules.stand_minutes == ends + 1 {
                for k in 0..n {
                    let name = format!("{}-stand{k}", HOUSE[side.index()]);
                    let key = w.wallet(3 * SOL);
                    keys.insert(name.clone(), key);
                    run(&mut w, &buy(key, config, houses[side.index()], SOL, 0, Some(raid)), "buy in a raid", &mut meter);
                    law.come_in(&name, side);
                }
                same(&w, &law, &houses, &format!("the stand's reserve at minute {m}"));
            }
        }
        run(&mut w, &crank(config, houses[0], houses[1]), "crank", &mut meter);
        let p = law.minute(m).unwrap();
        assert_eq!(w.raid(&raid).phase(), Some(p), "the program's phase is the law's at minute {m}");
        assert_eq!(w.raid(&raid).next_minute, m + 1);
        if let Phase::Decided { .. } = p {
            break;
        }
    }

    // the settlement, every loser holder's convert, the winner's stakers' unlock
    run(&mut w, &settle(config, houses[0], houses[1]), "settle", &mut meter);
    let done = law.settle().unwrap();
    let (wi, li) = (done.winner.index(), done.winner.other().index());
    let (winner, loser) = (houses[wi], houses[li]);
    let r = w.raid(&raid);
    assert_eq!(r.status, RAID_SETTLED);
    assert_eq!((r.pot_in as u128, r.bought_with as u128, r.coins_bought as u128, r.to_chest as u128), (done.settlement.pot_in, done.settlement.bought_with, done.settlement.buy.coins, done.settlement.to_chest));
    assert_eq!((r.mcap_end[0] as u128, r.mcap_end[1] as u128, r.weights), (done.mcap_end[0], done.mcap_end[1], done.weights));
    let (wh, lh) = (w.house(&winner), w.house(&loser));
    assert_eq!((wh.status, lh.status), (HOUSE_ACTIVE, HOUSE_REKT));
    assert_eq!((lh.curve.rq, lh.chest), (0, 0), "the loser's reserve and chest go to nothing");
    assert_eq!(w.lamports(&loser), w.rent(poleis_wars::state::House::SIZE) + lh.creator_owed, "the loser holds its rent and what it owes its own creator, nothing more");
    assert_eq!(lh.creator_owed as u128, law.creator_fees[li], "what the loser owes its creator stays in it");
    assert_eq!(wh.curve, law.curves[wi], "the winner's curve after the settlement buy is the law's");
    let dump = done.dump.expect("every reference winner climbed");
    assert_eq!((wh.dump_rate_bps, wh.dump_start, wh.dump_end - wh.dump_start), (dump.rate_bps, w.now(), dump.seconds as i64), "the dump tax, from the settlement, in real seconds");
    let holders: Vec<(String, Pubkey)> = keys.iter().filter(|(_, k)| w.position(&loser, k).map(|p| p.coins > 0).unwrap_or(false)).map(|(n, k)| (n.clone(), *k)).collect();
    for (name, k) in &holders {
        run(&mut w, &convert(*k, config, raid, loser, winner), "convert", &mut meter);
        assert!(w.position(&loser, k).is_none(), "{name}'s position in the loser is closed: he converts once");
        assert_eq!(w.position(&winner, k).unwrap().coins as u128, law.coins[&(name.clone(), done.winner)], "{name}'s share is the law's");
    }
    let wh = w.house(&winner);
    assert_eq!(wh.held as u128, done.all_held, "every coin of the winner held, its holders' and the converted");
    let r = w.raid(&raid);
    assert!(r.coins_bought - r.converted < holders.len() as u64 + 1, "what the floors of the shares leave is dust");
    if done.winner == Side::Attacker {
        for name in &done.stakers {
            run(&mut w, &unlock(keys[name], raid, houses[0]), "unlock", &mut meter);
            assert_eq!(w.position(&houses[0], &keys[name]).unwrap().staked, 0, "{name}'s coins are his again");
        }
    }
    // THE GAME IS ONE SWARM: each house owes its creator exactly the law's creator fees of every trade into it (the buys, the changes of
    // sides into it, the settlement's buy into the winner); the publisher's fee account holds exactly the law's protocol fees and the dump
    // tax's fee half; then anyone pays them out, each exactly that
    for i in 0..2 {
        assert_eq!(w.house(&houses[i]).creator_owed as u128, law.creator_fees[i], "the {}'s creator is owed the law's creator fees", HOUSE[i]);
    }
    let f = w.fees(&fees_pda(&config));
    assert_eq!((f.protocol as u128, f.taxed as u128, f.paid), (law.protocol_fees, law.taxed, 0), "the publisher's fee account holds the law's");
    let mut paid_creators = [0u128; 2];
    for i in 0..2 {
        let before = w.lamports(&creators[i]);
        run(&mut w, &pay_creator(houses[i], creators[i]), "pay_creator", &mut meter);
        paid_creators[i] = (w.lamports(&creators[i]) - before) as u128;
        assert_eq!(paid_creators[i], law.creator_fees[i], "the {}'s creator got the law's creator fees", HOUSE[i]);
        assert_eq!(w.house(&houses[i]).creator_owed, 0);
    }
    let publisher = publisher_of(&auth);
    let before = w.lamports(&publisher);
    run(&mut w, &pay_publisher(config, publisher), "pay_publisher", &mut meter);
    let paid_publisher = (w.lamports(&publisher) - before) as u128;
    assert_eq!(paid_publisher, law.protocol_fees + law.taxed, "the publisher got the law's protocol fees and the tax's fee half");
    assert_eq!(w.fees(&fees_pda(&config)).paid as u128, paid_publisher);
    assert_eq!(w.lamports(&fees_pda(&config)), w.rent(poleis_wars::state::Fees::SIZE), "the fee account keeps its rent alone");
    let mut fighters = BTreeMap::new();
    for (name, k) in &keys {
        let holds = w.position(&winner, k).map(|p| p.coins).unwrap_or(0) as u128;
        let backed = backed_of(wh.curve.rq + wh.chest as u128, holds, wh.held as u128).unwrap();
        fighters.insert(name.clone(), (holds, backed, took.get(name).copied().unwrap_or(0)));
    }
    Out {
        winner: done.winner,
        how: how_text(done.how, done.at),
        minutes: r.minutes,
        need: r.need,
        m0: r.m0,
        mcap_end: r.mcap_end,
        pot_in: r.pot_in,
        bought_with: r.bought_with,
        coins_bought: r.coins_bought,
        to_chest: r.to_chest,
        winner_reserve: wh.curve.rq,
        winner_chest: wh.chest,
        all_held: wh.held,
        stakers: done.stakers.clone(),
        dump: (wh.dump_rate_bps, wh.dump_end - wh.dump_start),
        fighters,
        cu: meter.cu,
        instructions: meter.instructions,
        paid_creators,
        paid_publisher,
    }
}

/// the compute units of each kind of instruction a war ran: the least, the most and how many
#[derive(Default)]
struct Meter {
    cu: BTreeMap<&'static str, (u64, u64, u32)>,
    instructions: u32,
}
impl Meter {
    fn note(&mut self, kind: &'static str, c: u64) {
        let e = self.cu.entry(kind).or_insert((u64::MAX, 0, 0));
        e.0 = e.0.min(c);
        e.1 = e.1.max(c);
        e.2 += 1;
        self.instructions += 1;
    }
}

/// a war's end on the chain, held to the owner's reference
fn against_the_reference(out: &Out, r: &Value, name: &str) {
    let st = &r["settled"];
    assert_eq!(HOUSE[out.winner.index()], r["winner"].as_str().unwrap(), "{name}: winner");
    assert_eq!(out.how, r["how"].as_str().unwrap(), "{name}: how");
    assert_eq!(out.minutes as u128, int(&r["minutes"]), "{name}: minutes");
    assert_eq!(out.need as f64, st["stake_need"].as_f64().unwrap().floor(), "{name}: the need, the floor of the model's float");
    assert_eq!((out.m0[0] as u128, out.m0[1] as u128), (int(&st["m0"]["crow"]), int(&st["m0"]["wolf"])), "{name}: m0");
    assert_eq!((out.mcap_end[0] as u128, out.mcap_end[1] as u128), (int(&st["mcap_end"]["crow"]), int(&st["mcap_end"]["wolf"])), "{name}: mcap_end");
    assert_eq!(out.pot_in as u128, int(&st["pot_in"]), "{name}: pot_in");
    assert_eq!(out.bought_with as u128, int(&st["bought_with"]), "{name}: bought_with");
    assert_eq!(out.coins_bought as u128, int(&st["coins_bought"]), "{name}: coins_bought");
    assert_eq!(out.winner_reserve, int(&st["winner_reserve"]), "{name}: winner_reserve");
    assert_eq!(out.winner_chest as u128, int(&st["winner_chest"]), "{name}: winner_chest");
    assert_eq!(out.to_chest, 0, "{name}: no reference settlement leaves a lamport for the chest");
    assert_eq!(out.all_held as u128, int(&st["all_held"]), "{name}: all_held");
    let stakers: Vec<String> = st["stakers"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    assert_eq!(out.stakers, stakers, "{name}: stakers");
    let pf = r["per_fighter"].as_object().unwrap();
    assert_eq!(pf.len(), out.fighters.len(), "{name}: the same fighters");
    for (who, (holds, backed, took)) in &out.fighters {
        let x = &pf[who];
        assert_eq!(*holds, int(&x["holds"]), "{name}: {who} holds");
        assert_eq!(*backed, int(&x["backed_lamports"]), "{name}: {who} backed_lamports");
        assert_eq!(*took, int(&x["took_lamports"]), "{name}: {who} took_lamports");
    }
    let at_once = r["dump_tax_at_once"].as_f64().unwrap();
    assert_eq!(out.dump.0 as f64, (at_once * 10_000.0).floor(), "{name}: the dump tax's rate, the floor of the model's");
}

#[test]
fn the_four_reference_wars_on_the_chain() {
    let refs = reference();
    for (s, r) in the_four().iter().zip(&refs) {
        assert_eq!(r["name"].as_str().unwrap(), s.name);
        let out = replay(s, 60);
        against_the_reference(&out, r, s.name);
        eprintln!(
            "  {}: {} wins, {} | {} instructions | dump tax {} bp for {} s | paid: the crow's creator {}, the wolf's {}, the publisher {} lamports",
            s.name,
            HOUSE[out.winner.index()],
            out.how,
            out.instructions,
            out.dump.0,
            out.dump.1,
            out.paid_creators[0],
            out.paid_creators[1],
            out.paid_publisher
        );
        for (k, (lo, hi, n)) in &out.cu {
            eprintln!("    {k:<14} x{n:<4} {lo} to {hi} CU");
        }
    }
}

/// THE MINUTE OF A LOCAL VALIDATOR: the first reference war at a minute of one second (a raid of 195 minutes in 195 seconds), the same numbers
#[test]
fn the_first_war_at_a_minute_of_one_second_gives_the_same_numbers() {
    let refs = reference();
    let s = &the_four()[0];
    let (fast, slow) = (replay(s, 1), replay(s, 60));
    against_the_reference(&fast, &refs[0], "at one second a minute");
    assert_eq!((fast.winner, &fast.how, fast.coins_bought, fast.all_held, fast.dump), (slow.winner, &slow.how, slow.coins_bought, slow.all_held, slow.dump));
    assert_eq!(fast.fighters, slow.fighters, "every fighter alike at 1 and at 60 seconds a minute");
    assert_eq!((fast.paid_creators, fast.paid_publisher), (slow.paid_creators, slow.paid_publisher), "and every fee paid alike");
}
