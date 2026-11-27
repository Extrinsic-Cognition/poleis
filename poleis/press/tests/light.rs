//! THE LIGHT PAYS A WISH AND ITS RIDER, proven (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider; his "Yes, build them" at 13:25 UTC to
//! "Build the three missing links now?"). Two halves:
//! - the reader and the pour's plan, pure, on books made by the stake book's own law (`embertide_stake::count`): at every gate;
//! - the whole road on a validator holding this tree's Poleis and mainnet's stake pool program (LIGHT_VALIDATOR, LIGHT_BOOK: set by
//!   poleis/ceremony/the-light-rehearsal.sh): root 0 and her pool born as the launch births them (the pool by root 0's own raw act), a wish founded as
//!   the wish walker founds one, two stakers, one light sent to the wish and one left unchosen, an epoch's yield, the book read twice by its own hand,
//!   the reader, the pour (and the pour again, which sends nothing), the wish's vote of a rider, the rider's draw and its way out to SOL. Without
//!   LIGHT_VALIDATOR the live half says so and passes.

use embertide_stake::{self as stake, Book, Snapshot};
use poleis_client::{self as client, ROOT_UNIT, STATE_PASSED, VOTE_PRO};
use poleis_press::door::{self, Light};
use poleis_press::light::{self, Allowed, Held, Lights};
use poleis_press::loader;
use poleis_press::pool::{Pool, Program, MINT_LEN, RESERVE_LEN, STAKE_POOL};
use poleis_press::rider;
use poleis_press::rpc::{self, Cluster, Rpc};
use poleis_press::wire::{self, Key};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

const SOL: u64 = 1_000_000_000;

fn snap(epoch: u64, total: u64, supply: u64, holders: &[(&str, u64)]) -> Snapshot {
    Snapshot { epoch, total_lamports: total, pool_token_supply: supply, holders: holders.iter().map(|(o, t)| (o.to_string(), *t)).collect() }
}

/// Two reads of a pool as the book's own law counts them: four stakers across epoch 5 to 6, a fee of 1,000,003 tokens.
fn books() -> (Book, Book) {
    let holders = [("s1", 10 * SOL), ("s2", 5 * SOL), ("s3", 3 * SOL), ("s4", 2 * SOL)];
    let b0 = stake::count(&Book::new("pool"), &snap(5, 20 * SOL, 20 * SOL, &holders), 0).unwrap();
    let b1 = stake::count(&b0, &snap(6, 20 * SOL + 1_000_003, 20 * SOL + 1_000_003, &holders), 1_000_003).unwrap();
    (b0, b1)
}

fn choices(c: &[(&str, Option<Light>)]) -> BTreeMap<String, Option<Light>> {
    c.iter().map(|(o, l)| (o.to_string(), *l)).collect()
}

#[test]
fn each_light_goes_where_its_staker_chose_and_the_rest_stays() {
    let (b0, b1) = books();
    let l = light::lights_of(Some(&b0), &b1).unwrap();
    assert_eq!((l.epoch, l.from, l.fee_tokens), (6, 5, 1_000_003));
    assert_eq!(l.of.values().sum::<u128>() + l.dust_lamports as u128, l.fee_lamports as u128, "the lights and the dust are the fee");
    let c = choices(&[("s1", Some(Light::Wish(7))), ("s2", None), ("s3", Some(Light::Futarchy)), ("s4", Some(Light::Wish(99)))]);
    let r = light::read(&l, &c, &BTreeSet::from([7]));
    assert_eq!(r.wishes.len(), 1, "unit 99 is no swarm: its light is undirected");
    let w = &r.wishes[&7];
    assert_eq!(w.stakers, vec!["s1".to_string()]);
    assert_eq!(w.light_lamports, l.of["s1"]);
    assert_eq!(w.tokens as u128, l.of["s1"] * l.fee_tokens as u128 / l.fee_lamports as u128, "a wish's share in her token, rounded down");
    assert_eq!(r.undirected_lamports, l.of["s2"] + l.of["s3"] + l.of["s4"]);
    assert_eq!(r.to_wishes() + r.undirected_tokens + r.dust_tokens, r.fee_tokens, "nothing past the fee; the dust stays in the vault");
    assert_eq!(r.stakers.iter().find(|s| s.0 == "s4").unwrap().2, light::Goes::NotASwarm(99));
    assert_eq!(r.stakers.iter().find(|s| s.0 == "s2").unwrap().2, light::Goes::NoChoice);
    // a light sent to root 0 itself is undirected: it is already there
    let r0 = light::read(&l, &choices(&[("s1", Some(Light::Wish(ROOT_UNIT)))]), &BTreeSet::from([ROOT_UNIT]));
    assert!(r0.wishes.is_empty());
    assert!(light::agree(&r, 1_000_003).is_ok());
    let e = light::agree(&r, 1_000_002).unwrap_err();
    assert!(e.contains("1000003") && e.contains("1000002") && e.contains("nothing is poured"), "{e}");
}

#[test]
fn the_reader_refuses_books_that_are_not_one_count_apart() {
    let (b0, b1) = books();
    assert!(light::lights_of(Some(&b1), &b1).is_err(), "no count between them");
    assert!(light::lights_of(None, &b0).is_err(), "a first read counts nothing");
    let mut other = b0.clone();
    other.pool = "another".into();
    assert!(light::lights_of(Some(&other), &b1).unwrap_err().contains("two pools"));
    let mut lost = b1.clone();
    lost.stakers.get_mut("s1").unwrap().light += 1;
    assert!(light::lights_of(Some(&b0), &lost).unwrap_err().contains("do not sum"));
}

#[test]
fn shares_never_pass_the_fee_over_many_books() {
    let mut seed = 0x9e3779b97f4a7c15u64;
    let mut next = |m: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % m
    };
    for _ in 0..300 {
        let n = 1 + next(6) as usize;
        let names: Vec<String> = (0..n).map(|i| format!("h{i}")).collect();
        let h0: Vec<(&str, u64)> = names.iter().map(|o| (o.as_str(), 1 + next(50 * SOL))).collect();
        let h1: Vec<(&str, u64)> = names.iter().map(|o| (o.as_str(), next(50 * SOL))).collect();
        let total: u64 = h0.iter().map(|h| h.1).sum();
        let fee = next(5 * SOL);
        let b0 = stake::count(&Book::new("p"), &snap(1, total, total, &h0), 0).unwrap();
        let b1 = stake::count(&b0, &snap(2, total + fee, total + 1 + next(total), &h1), fee).unwrap();
        let l = light::lights_of(Some(&b0), &b1).unwrap();
        let c: BTreeMap<String, Option<Light>> = names.iter().map(|o| (o.clone(), match next(4) { 0 => None, 1 => Some(Light::Futarchy), u => Some(Light::Wish(u)) })).collect();
        let r = light::read(&l, &c, &BTreeSet::from([2, 3]));
        assert!(r.to_wishes() <= r.fee_tokens);
        assert_eq!(r.to_wishes() + r.undirected_tokens + r.dust_tokens, r.fee_tokens);
        assert!(light::agree(&r, fee).is_ok());
    }
}

fn lights(of: &[(&str, u128)], fee_tokens: u64, fee_lamports: u64) -> Lights {
    Lights { epoch: 9, from: 8, fee_tokens, fee_lamports, dust_lamports: fee_lamports - of.iter().map(|x| x.1).sum::<u128>() as u64, of: of.iter().map(|(o, l)| (o.to_string(), *l)).collect() }
}

#[test]
fn the_pour_runs_on_a_standing_allowance_and_is_said_once() {
    let (crank, mint) = (Pubkey::new_from_array([1; 32]), Pubkey::new_from_array([7; 32]));
    let l = lights(&[("a", 600), ("b", 300), ("c", 100)], 1_000, 1_000);
    let c = choices(&[("a", Some(Light::Wish(7))), ("b", Some(Light::Wish(8))), ("c", Some(Light::Wish(9)))]);
    let r = light::read(&l, &c, &BTreeSet::from([7, 8, 9]));
    let well = |cap| Allowed { hash: [cap as u8; 32], cap, period: 32, start: 100, spent: 0, expires: 0 };
    let wells = BTreeMap::from([(7, well(10_000)), (8, well(200))]);
    let done = BTreeSet::from([(9u64, 99u64)]);
    let (pours, held) = light::plan(&crank, &mint, &r, &wells, &BTreeSet::from([8]), &done, &BTreeMap::new(), 1_000, 110).unwrap();
    assert_eq!(pours.len(), 1);
    let p = &pours[0];
    assert_eq!((p.unit, p.tokens), (7, 600));
    assert_eq!(p.ixs.len(), 3, "wish 7's vault made, the pull, the memo");
    assert_eq!(p.ixs[0], client::create_vault(crank, 7, mint, light::TOKEN));
    assert_eq!(p.ixs[1], client::pull(crank, wells[&7].hash, ROOT_UNIT, 600, client::vault_pda(7, &mint), Some((mint, light::TOKEN))), "out of root 0's vault, into the wish's own");
    assert_eq!(p.ixs[2].program_id, door::MEMO);
    assert_eq!(light::pour_of(std::str::from_utf8(&p.ixs[2].data).unwrap()), Some((9, 7, 600, 600)));
    assert_eq!(held, vec![(8, Held::NoRoom { room: 200 }), (9, Held::NoWell)]);
    // poured once: the crank's memo for the epoch and the wish stands, and the second plan sends nothing
    let (again, held) = light::plan(&crank, &mint, &r, &wells, &BTreeSet::from([7, 8]), &BTreeSet::from([(9, 7)]), &BTreeMap::new(), 1_000, 110).unwrap();
    assert!(again.is_empty());
    assert_eq!(held[0], (7, Held::Poured));
    // a vault that holds less than the pours: nothing poured
    assert!(light::plan(&crank, &mint, &r, &wells, &BTreeSet::new(), &BTreeSet::new(), &BTreeMap::new(), 599, 110).unwrap_err().contains("nothing is poured"));
    // a period turned gives the whole cap again; an expired allowance gives nothing
    let spent = Allowed { spent: 200, ..well(200) };
    assert_eq!((spent.room(110), spent.room(132)), (0, 200));
    assert_eq!(Allowed { expires: 120, ..well(200) }.room(121), 0);
    assert_eq!(light::pour_of("embertide.pour/1 epoch 3 wish 7"), None);
}

fn stranger_pull(tokens: u64, slot: u64) -> light::Pull {
    light::Pull { sig: format!("stranger-{slot}"), slot, puller: Pubkey::new_from_array([13; 32]), tokens, pour: None }
}

/// A stranger's pull of the wish's road before the crank runs: said by name as an over-pull, and the pour is only what the light is still owed.
#[test]
fn a_strangers_pull_before_the_pour_is_netted() {
    let (crank, mint) = (Pubkey::new_from_array([1; 32]), Pubkey::new_from_array([7; 32]));
    let r = light::read(&lights(&[("a", 1_000)], 1_000, 1_000), &choices(&[("a", Some(Light::Wish(7)))]), &BTreeSet::from([7]));
    let wells = BTreeMap::from([(7, Allowed { hash: [7; 32], cap: 10_000, period: 32, start: 100, spent: 300, expires: 0 })]);
    let ledger = light::Ledger::of(&[], &[stranger_pull(300, 105)]);
    assert_eq!(ledger.new_over_pulls().len(), 1, "the stranger's pull, by name");
    let (pours, _) = light::plan(&crank, &mint, &r, &wells, &BTreeSet::from([7]), &BTreeSet::new(), &BTreeMap::from([(7, ledger)]), 1_000, 110).unwrap();
    let p = &pours[0];
    assert_eq!((p.owed, p.tokens, p.over), (1_000, 700, 0), "owed 1,000, took 300 already: 700 poured");
    assert_eq!(p.ixs[0], client::pull(crank, [7; 32], ROOT_UNIT, 700, client::vault_pda(7, &mint), Some((mint, light::TOKEN))));
    assert_eq!(light::pour_of(std::str::from_utf8(&p.ixs[1].data).unwrap()), Some((9, 7, 1_000, 700)));
}

/// A wish over its light across epochs: nothing pulled (a record alone) while it is over, its over-take counted against its next epochs' light until
/// even; the crank's own pulls are never over-pulls; root 0 never pays twice.
#[test]
fn a_wish_over_its_light_is_evened_by_its_next_epochs() {
    let (crank, mint) = (Pubkey::new_from_array([1; 32]), Pubkey::new_from_array([7; 32]));
    let wells = BTreeMap::from([(7, Allowed { hash: [7; 32], cap: 10_000, period: 32, start: 100, spent: 0, expires: 0 })]);
    let epoch = |e: u64| {
        let mut l = lights(&[("a", 100)], 100, 100);
        l.epoch = e;
        l.from = e - 1;
        light::read(&l, &choices(&[("a", Some(Light::Wish(7)))]), &BTreeSet::from([7]))
    };
    // epoch 1: a stranger took 250 before anything was owed; 100 is owed now, so the wish is over by 150 and nothing is pulled
    let l1 = light::Ledger::of(&[], &[stranger_pull(250, 50)]);
    let (p1, _) = light::plan(&crank, &mint, &epoch(1), &wells, &BTreeSet::from([7]), &BTreeSet::new(), &BTreeMap::from([(7, l1)]), 1_000, 110).unwrap();
    assert_eq!((p1[0].owed, p1[0].tokens, p1[0].over), (100, 0, 150));
    assert_eq!(p1[0].ixs.len(), 1, "the record alone, no pull");
    // epoch 2: 200 owed in all, 250 taken: still over by 50
    let l2 = light::Ledger::of(&[(100, 60)], &[stranger_pull(250, 50)]);
    assert!(l2.new_over_pulls().is_empty(), "the stranger's pull was said before the last record");
    let (p2, _) = light::plan(&crank, &mint, &epoch(2), &wells, &BTreeSet::from([7]), &BTreeSet::new(), &BTreeMap::from([(7, l2)]), 1_000, 110).unwrap();
    assert_eq!((p2[0].tokens, p2[0].over), (0, 50));
    // epoch 3: 300 owed, 250 taken: 50 poured, even
    let l3 = light::Ledger::of(&[(100, 60), (100, 70)], &[stranger_pull(250, 50)]);
    let (p3, _) = light::plan(&crank, &mint, &epoch(3), &wells, &BTreeSet::from([7]), &BTreeSet::new(), &BTreeMap::from([(7, l3)]), 1_000, 110).unwrap();
    assert_eq!((p3[0].tokens, p3[0].over), (50, 0));
    // epoch 4, with the crank's own 50 among the pulls: 400 owed, 300 taken, 100 poured; the crank's pull is no over-pull
    let mine = light::Pull { sig: "crank".into(), slot: 80, puller: crank, tokens: 50, pour: Some(3) };
    let l4 = light::Ledger::of(&[(100, 60), (100, 70), (100, 80)], &[stranger_pull(250, 50), mine]);
    assert_eq!(l4.over_pulls.len(), 1);
    let (p4, _) = light::plan(&crank, &mint, &epoch(4), &wells, &BTreeSet::from([7]), &BTreeSet::new(), &BTreeMap::from([(7, l4.clone())]), 1_000, 110).unwrap();
    assert_eq!((p4[0].tokens, p4[0].over), (100, 0));
    assert_eq!(l4.taken + p4[0].tokens as u128, l4.owed + 100, "what the wish took ends equal to what its light was owed: never twice");
}

/// THE LIGHT'S PLAN IS THE TENTH ACT: on the later acts' road (`engine::later`), after the ninth; no act while `light.planCap` is his blank or the
/// rehearsal's word; with a figure, root 0's AllowMint of her pool's own mint, a CODEX act of the whole tree.
#[test]
fn the_lights_plan_is_the_tenth_act() {
    use poleis_press::params::{placeholder_address, Params};
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/params.mainnet.json")).unwrap();
    let mut v: Value = serde_json::from_str(&text).unwrap();
    let k = |w: &str| json!(placeholder_address(w).to_string());
    for (a, b) in [("floor", "issuer"), ("warm", "miner"), ("market", "crediter")] {
        v[a][b] = k(b);
    }
    v["posiverse"]["agents"] = json!([k("agent 1"), k("agent 2"), k("agent 3")]);
    v["warm"]["opening"] = json!("2026-11-15T00:00:00Z");
    let p = Params::from_text(&v.to_string()).unwrap();
    assert!(p.sendable().is_ok(), "mainnet params are sendable once sitting keys are filled");
    assert_eq!(p.later_placeholders("light"), Vec::<String>::new());
    let (ids, ..) = poleis_press::build::compiled().unwrap();
    let at = poleis_press::chain::addresses(&ids, 77);
    let d = placeholder_address("light deployer");
    assert_eq!(light::later(&p, &ids, &d).unwrap().len(), 1, "mainnet's 100 SOL light.planCap writes act 10");
    let mut blank_light = v.clone();
    blank_light["light"]["planCap"] = json!("PLACEHOLDER: his");
    let p_blank = Params::from_text(&blank_light.to_string()).unwrap();
    assert_eq!(p_blank.later_placeholders("light"), ["light.planCap"]);
    assert!(light::later(&p_blank, &ids, &d).unwrap().is_empty(), "no tenth act while its cap is blank");
    v["light"]["planCap"] = json!(5_000_000_000u64);
    v["engine"]["staker"] = k("the engine");
    v["engine"]["reserveBp"] = json!(1000);
    v["engine"]["validators"] = json!([{ "vote": k("a validator"), "weight": 1 }]);
    let p = Params::from_text(&v.to_string()).unwrap();
    let later = poleis_press::engine::later(&p, &ids, &at, &d).unwrap();
    assert_eq!(later.iter().map(|a| a.n).collect::<Vec<_>>(), [9, 10], "the engine's, then the light's plan");
    let ten = &later[1];
    let mint = Pool::of(&p, &d, &ids.poleis).unwrap().mint;
    assert_eq!(ten.payload, rider::plan_act(ROOT_UNIT, &mint, 5_000_000_000, &d).bytes(), "root 0's plan of her pool's own token");
    assert_eq!(ten.impacted, vec![ROOT_UNIT, 1], "a CODEX act: the whole tree, both chambers");
    let rehearsal = Params::from_text(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json")).unwrap()).unwrap();
    assert!(light::later(&rehearsal, &ids, &d).unwrap().is_empty(), "the rehearsal's word writes no tenth act");
}

// ---- the live half ----

/// One transaction landed by the runner's own sender (epoch.rs `land_surely`): the epoch rehearsal's second run lost a stake of these tests' own,
/// sent once and polled 60 s, at a 32-slot epoch's boundary; every send here is now sent again and built again, as the runner's are.
fn land(rpc: &Rpc, ixs: &[Instruction], payer: &Key, keys: &[&Key]) -> Result<String, String> {
    let mut ks = vec![payer];
    ks.extend(keys.iter().copied().filter(|k| k.address != payer.address));
    poleis_press::epoch::land_surely(rpc, ixs, &ks).map(|(sig, _)| sig)
}

fn epoch(rpc: &Rpc) -> u64 {
    rpc.call("getEpochInfo", json!([{ "commitment": "confirmed" }])).unwrap()["epoch"].as_u64().unwrap()
}

fn until_slot(rpc: &Rpc, s: u64) {
    rpc::wait(Duration::from_secs(600), || Ok((rpc.slot()? > s).then_some(()))).unwrap();
}

fn budget(units: u32) -> Instruction {
    let mut d = vec![2u8];
    d.extend_from_slice(&units.to_le_bytes());
    Instruction::new_with_bytes(loader::COMPUTE_BUDGET, &d, vec![])
}

fn transfer(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let mut d = 2u32.to_le_bytes().to_vec();
    d.extend_from_slice(&lamports.to_le_bytes());
    Instruction::new_with_bytes(loader::SYSTEM, &d, vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false)])
}

/// A tree as the wish walker founds one (poleis/seal/examples/wish.rs): its config and its genesis in one transaction, its starter alone in its
/// chairs; a first sealed bid for a hexagon.
fn found(rpc: &Rpc, root: u64, starter: &Key, bidder: &Key, name: &str, words: &str, salt: &[u8; 32]) -> client::GenesisSpec {
    let s0 = rpc.slot().unwrap();
    let g = client::GenesisSpec {
        charter: wire::sha256(words.as_bytes()), plan_cap: 10 * SOL, plan_period: 1_000_000, window: client::MIN_WINDOW_SLOTS, rung: 1, reserve: 1_000_000,
        deposit: 5_000_000, commit_end: s0 + 50, reveal_end: s0 + 120, founders: vec![starter.address], name: name.as_bytes().to_vec(), desc: words.as_bytes().to_vec(),
    };
    land(rpc, &[client::init_config_of(root, starter.address, Pubkey::default()), client::open_genesis_of(root, starter.address, &g)], starter, &[starter]).unwrap();
    land(rpc, &[client::commit_seat_bid(bidder.address, root, 0, 0, client::seat_commitment(2_000_000, 0, salt, &bidder.address))], bidder, &[bidder]).unwrap();
    g
}

/// The first sale closed and the birth made good: the bid revealed, the sale settled, the first hexagon claimed, the genesis ratified.
/// Several trees side by side: every bid revealed inside its window, then every sale settled once its window has closed.
fn ratify(rpc: &Rpc, trees: &[(u64, &client::GenesisSpec, &Key, &[u8; 32])], crank: &Key) {
    for (root, g, bidder, salt) in trees {
        until_slot(rpc, g.commit_end);
        land(rpc, &[client::reveal_seat_bid(bidder.address, *root, 0, 0, 2_000_000, **salt)], bidder, &[bidder]).unwrap();
    }
    for (root, g, bidder, _) in trees {
        until_slot(rpc, g.reveal_end);
        land(rpc, &[client::settle_seats(crank.address, *root, 0)], crank, &[crank]).unwrap();
        land(rpc, &[client::claim_seat(bidder.address, *root, 0, 0, *root)], bidder, &[bidder]).unwrap();
        land(rpc, &[client::ratify_genesis_of(*root, crank.address)], crank, &[crank]).unwrap();
    }
}

/// One whole decision of a tree: proposed by one of its own, every voice it has saying yes, sealed, carried out by the crank (`before` sent in the
/// execute's own transaction, first). Its hash.
fn decide(rpc: &Rpc, root: u64, voices: &[&Key], crank: &Key, d: &rider::Decision, before: Vec<Instruction>) -> [u8; 32] {
    let h = d.hash;
    let unit = d.payload.origin();
    land(rpc, &[client::propose_of(root, voices[0].address, h, d.manifest, &d.payload, &[unit])], voices[0], &[voices[0]]).unwrap();
    let signed: Vec<(Pubkey, u8, [u8; 64])> = voices.iter().map(|k| (k.address, VOTE_PRO, k.sign(&client::vote_envelope(&h, unit, VOTE_PRO)))).collect();
    land(rpc, &client::attest_batch(crank.address, h, unit, &signed), crank, &[crank]).unwrap();
    if land(rpc, &[client::seal(crank.address, h, unit)], crank, &[crank]).is_err() {
        let s = client::SessionView::parse(&rpc.account(&client::session_pda(&h, unit).to_string()).unwrap().unwrap().data).unwrap();
        until_slot(rpc, s.deadline);
        land(rpc, &[client::seal(crank.address, h, unit)], crank, &[crank]).unwrap();
    }
    let s = client::SessionView::parse(&rpc.account(&client::session_pda(&h, unit).to_string()).unwrap().unwrap().data).unwrap();
    assert_eq!(s.state, STATE_PASSED, "{}: the swarm passed it", d.what);
    let mut ixs = vec![budget(400_000)];
    ixs.extend(before);
    ixs.push(client::execute_of(root, crank.address, h, &[unit], &d.payload));
    land(rpc, &ixs, crank, &[crank]).unwrap();
    h
}

/// Her list's validators as the update names them, read off the chain.
fn list(rpc: &Rpc, p: &Pool) -> Vec<door::Pair> {
    door::pairs(p, &rpc.account(&p.list.to_string()).unwrap().expect("her pool's list").data).unwrap()
}

fn pool_now(rpc: &Rpc, p: &Pool) -> door::PoolNow {
    door::pool_now(&rpc.account(&p.pool.to_string()).unwrap().expect("the pool's account").data).unwrap()
}

/// The pool brought to the epoch when it is behind, after the epoch's rewards are paid; whether it was behind.
fn bring(rpc: &Rpc, p: &Pool, crank: &Key) -> bool {
    if pool_now(rpc, p).last_update_epoch >= epoch(rpc) {
        return false;
    }
    let paying = || door::rewards_paying(&rpc.account(&door::EPOCH_REWARDS.to_string()).unwrap().map(|a| a.data).unwrap_or_default());
    rpc::wait(Duration::from_secs(120), || Ok((!paying()).then_some(()))).unwrap();
    land(rpc, &door::update(p, &list(rpc, p)), crank, &[crank]).unwrap();
    true
}

/// One read of her pool's book by its own hand (`embertide-stake count --write`), the pool brought to the epoch first; tried again when an epoch
/// turns between the two.
fn count(rpc: &Rpc, url: &str, bin: &str, p: &Pool, book: &std::path::Path, crank: &Key) -> Value {
    // the hand reads at the node's default level, finalized: what was just sent is seen there some 32 slots later
    for _ in 0..12 {
        bring(rpc, p, crank);
        let now = rpc.slot().unwrap();
        rpc::wait(Duration::from_secs(120), || Ok((rpc.call("getSlot", json!([{ "commitment": "finalized" }]))?.as_u64().unwrap_or(0) >= now).then_some(()))).unwrap();
        let out = std::process::Command::new(bin).args(["count", &p.pool.to_string(), "--rpc", url, "--book", book.to_str().unwrap(), "--write"]).output().unwrap();
        if out.status.success() {
            return serde_json::from_slice(&out.stdout).unwrap();
        }
        eprintln!("the book's read, again: {}", String::from_utf8_lossy(&out.stderr).trim());
        std::thread::sleep(Duration::from_secs(4));
    }
    panic!("the book's hand never read the pool");
}

fn book_at(path: &std::path::Path, pool: &Pool) -> Book {
    stake::book_of(&serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap(), &pool.pool.to_string()).unwrap()
}

#[test]
fn the_light_pays_a_wish_and_its_rider_on_a_validator() {
    let (Ok(url), Ok(bin)) = (std::env::var("LIGHT_VALIDATOR"), std::env::var("LIGHT_BOOK")) else {
        eprintln!("LIGHT_VALIDATOR and LIGHT_BOOK are not set: the live half runs under poleis/ceremony/the-light-rehearsal.sh");
        return;
    };
    let begun = std::time::Instant::now();
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };
    let dir = std::path::PathBuf::from(std::env::var("LIGHT_DIR").unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into()));

    // the stake pool program: mainnet's bytes, held to the fingerprint the rehearsal names
    let data = rpc.account(&loader::programdata_of(&STAKE_POOL).to_string()).unwrap().expect("the stake pool program's data on this validator");
    let program = Program::of(&STAKE_POOL, &data.data).unwrap();
    if let Ok(want) = std::env::var("LIGHT_PROGRAM_SHA256") {
        assert_eq!(wire::hex(&program.sha256), want, "the stake pool program's bytes on this validator");
    }

    // the cast, stand-ins of this run's own seed: root 0's starter and first hexagon, the wish's, a crank, two stakers, a rider
    let seed = wire::sha256(format!("the light's rehearsal {url} {}", std::process::id()).as_bytes());
    let names = ["root 0's starter", "root 0's first hexagon", "the wish's starter", "the wish's first hexagon", "the crank", "staker 1", "staker 2", "the rider", "a stranger"];
    let mut ks: Vec<Key> = names.iter().map(|n| wire::standin(&seed, n)).collect();
    // ONLY THE DEPLOYER FOUNDS ROOT 0: root 0's starter is the stand-in the rehearsal made Poleis's upgrade authority, as the launch's deployer is
    if let Ok(f) = std::env::var("LIGHT_DEPLOYER") {
        ks[0] = Key::from_cli_text(&std::fs::read_to_string(f).unwrap()).unwrap();
    }
    let [s0, h0, sw, hw, crank, st1, st2, rd, stranger] = [&ks[0], &ks[1], &ks[2], &ks[3], &ks[4], &ks[5], &ks[6], &ks[7], &ks[8]];
    for (k, sol) in [(s0, 5), (h0, 5), (sw, 5), (hw, 5), (crank, 50), (st1, 20), (st2, 20), (rd, 1), (stranger, 1)] {
        rpc.airdrop(&k.address.to_string(), sol * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || {
        for k in &ks {
            if rpc.balance(&k.address.to_string())? < SOL {
                return Ok(None);
            }
        }
        Ok(Some(()))
    })
    .unwrap();

    // ROOT 0 AND A WISH, founded the wish walker's way, side by side
    let words = "A publisher of games that use the blockchain as their multiplayer.";
    let wish = u64::from_le_bytes(wire::sha256(words.as_bytes())[..8].try_into().unwrap()).max(1);
    let (salt0, saltw) = ([1u8; 32], [2u8; 32]);
    let g0 = found(&rpc, ROOT_UNIT, s0, h0, "The meta-swarm", "Her: the root of them all.", &salt0);
    let gw = found(&rpc, wish, sw, hw, "The Game Publisher", words, &saltw);
    ratify(&rpc, &[(ROOT_UNIT, &g0, h0, &salt0), (wish, &gw, hw, &saltw)], crank);
    say("root 0 and a wish", json!({ "root 0": client::registry_pda(ROOT_UNIT).to_string(), "the wish": wish.to_string(), "its registry": client::registry_pda(wish).to_string() }));

    // HER POOL, as the launch births it: its reserve, its mint and root 0's vault of the mint made ahead; then root 0's raw act, the pool's
    // Initialize by the polis's own hand, its account and its list made in the execute's own transaction by the key that executes
    let p = Pool::derive(&crank.address, "light1", 10, &STAKE_POOL, &client::PROGRAM_ID).unwrap();
    let (_, ahead) = p.ahead(rpc.rent(RESERVE_LEN).unwrap(), rpc.rent(MINT_LEN).unwrap(), [None, None, None]).unwrap();
    land(&rpc, &ahead, crank, &[crank]).unwrap();
    let mut makings = Vec::new();
    for s in p.in_the_execute() {
        makings.extend(s.making(&crank.address, rpc.rent(s.space).unwrap(), None).unwrap());
    }
    let born = decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("her pool is born", p.payload(), 1), makings);
    let keys = door::keys(&p);
    let read_keys = door::Keys::of_account(&p.pool, &STAKE_POOL, &rpc.account(&p.pool.to_string()).unwrap().unwrap().data).unwrap();
    assert_eq!(read_keys, keys, "her pool as the door reads it off the chain: fee 1/1 into root 0's vault");
    say("her pool born by root 0's act", json!({ "pool": p.pool.to_string(), "mint": p.mint.to_string(), "fee account (root 0's vault)": p.vault.to_string(), "act": wire::hex(&born) }));

    // ROOT 0'S ROAD INTO THE WISH'S WELL: its plan of her token, then its standing allowance to the wish's unit, each voted once
    let cap = 1_000 * SOL;
    decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("root 0 may spend her token", rider::plan_act(ROOT_UNIT, &p.mint, cap, &crank.address), 2), vec![]);
    let well = decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("root 0's road into the wish's well", rider::well_act(wish, &p.mint, cap, 32), 3), vec![]);
    let wells = light::wells(&rpc, &p.mint).unwrap();
    assert_eq!(wells.get(&wish).map(|a| a.hash), Some(well), "the crank finds the wish's road on the chain");
    say("root 0's road into the wish's well", json!({ "allowance": client::allowance_pda(&well).to_string(), "cap a period": cap.to_string(), "period (slots)": 32, "payee": format!("unit {wish}") }));

    // TWO STAKERS: one sends its light to the wish, the other chooses nothing
    for (k, sol) in [(st1, 10), (st2, 5)] {
        let behind = pool_now(&rpc, &p).last_update_epoch < epoch(&rpc);
        if behind {
            bring(&rpc, &p, crank);
        }
        land(&rpc, &door::stake(&p, &k.address, sol * SOL, None), k, &[k]).unwrap();
    }
    let chose = land(&rpc, &[door::light(&st1.address, Light::Wish(wish))], st1, &[st1]).unwrap();
    say("two stakers", json!({ "staker 1": { "staked": 10 * SOL, "light": Light::Wish(wish).memo(), "signature": chose }, "staker 2": { "staked": 5 * SOL, "light": "none chosen" } }));

    // THE BOOK, FIRST READ, by its own hand
    let book = dir.join("her-book.json");
    let first = count(&rpc, &url, &bin, &p, &book, crank);
    assert_eq!(first["first_read"], true);
    let before_path = dir.join("her-book-before.json");
    std::fs::copy(&book, &before_path).unwrap();
    let b0 = book_at(&book, &p);

    // AN EPOCH'S YIELD: a pool's reward is the growth of its total, so lamports sent into her reserve before the epoch's update are that epoch's
    // reward, minted as the fee into root 0's vault (the engine that places her stake on validators is lane/the-stake-earns)
    let reward = 3 * SOL / 2;
    land(&rpc, &[transfer(&crank.address, &p.reserve, reward)], crank, &[crank]).unwrap();
    let read_at = b0.last.as_ref().unwrap().epoch;
    rpc::wait(Duration::from_secs(120), || Ok((epoch(&rpc) > read_at).then_some(()))).unwrap();
    let vault_before_fee = light::balance(&rpc, &p.vault).unwrap();
    let second = count(&rpc, &url, &bin, &p, &book, crank);
    let b1 = book_at(&book, &p);
    let row = b1.row.clone().unwrap();
    let minted = light::balance(&rpc, &p.vault).unwrap() - vault_before_fee;
    assert_eq!(row.fee_tokens, minted, "the book's fee is what reached root 0's vault");
    say("the epoch's yield", json!({ "sent into the reserve (lamports)": reward, "fee minted into root 0's vault (tokens)": row.fee_tokens.to_string(), "worth (lamports)": row.fee_lamports.to_string(), "epochs": [row.from, row.epoch], "the book's read": second["counted"] }));

    // A STRANGER PULLS THE WISH'S ROAD before the crank runs: Poleis lets anyone pull a unit payee's allowance, into that unit's own vault only
    let took = 300_000_000u64;
    let over = land(
        &rpc,
        &[client::create_vault(stranger.address, wish, p.mint, light::TOKEN), client::pull(stranger.address, well, ROOT_UNIT, took, client::vault_pda(wish, &p.mint), Some((p.mint, light::TOKEN)))],
        stranger,
        &[stranger],
    )
    .unwrap();
    say("a stranger's pull of the wish's road", json!({ "puller": stranger.address.to_string(), "tokens": took.to_string(), "transaction": over }));

    // THE READER AND THE POUR
    let c = light::crank(&rpc, &p.pool, Some(&b0), &b1, &crank.address).unwrap();
    assert!(c.agreed.is_ok(), "{:?}", c.agreed);
    let share = c.reading.wishes.get(&wish).cloned().expect("the wish's share");
    let l1 = c.reading.stakers.iter().find(|s| s.0 == st1.address.to_string()).unwrap().1;
    let l2 = c.reading.stakers.iter().find(|s| s.0 == st2.address.to_string()).unwrap().1;
    assert_eq!(c.reading.undirected_lamports, l2, "staker 2 chose nothing: its light stays");
    assert_eq!(c.pours.len(), 1);
    assert_eq!(c.over_pulled(), took, "the stranger's pull is read back");
    let o = &c.ledgers[&wish].new_over_pulls()[0];
    assert_eq!((o.tokens, o.puller, o.sig.as_str()), (took, stranger.address, over.as_str()), "said by name: the amount, the puller, the transaction");
    assert_eq!((c.pours[0].owed, c.pours[0].tokens), (share.tokens, share.tokens - took), "netted: only what its light is still owed");
    let sent = light::send(&rpc, &c, crank).unwrap();
    let in_well = light::balance(&rpc, &client::vault_pda(wish, &p.mint)).unwrap();
    let in_root = light::balance(&rpc, &p.vault).unwrap();
    assert_eq!(in_well, share.tokens, "the wish's well holds its share, the stranger's pull and the crank's pour together");
    assert_eq!(in_root, vault_before_fee + minted - share.tokens, "root 0's vault paid the wish's light once; the rest stayed");
    say("the reader", c.json());
    say("the pour", json!({ "signatures": sent, "over-pulled (tokens)": c.over_pulled().to_string(), "the crank poured (tokens)": c.pours[0].tokens.to_string(), "in the wish's well (tokens)": in_well.to_string(), "left in root 0's vault (tokens)": in_root.to_string(), "staker 1's light (lamports)": l1.to_string(), "staker 2's light (lamports)": l2.to_string() }));

    // AGAIN: the crank's memo stands, nothing is sent; and a book that disagrees with the chain pours nothing
    let again = light::crank(&rpc, &p.pool, Some(&b0), &b1, &crank.address).unwrap();
    assert!(again.pours.is_empty());
    assert_eq!(again.held, vec![(wish, Held::Poured)]);
    let mut lying = b1.clone();
    lying.row.as_mut().unwrap().fee_tokens += 1;
    let refused = light::crank(&rpc, &p.pool, Some(&b0), &lying, &crank.address).unwrap();
    assert!(refused.pours.is_empty() && refused.agreed.is_err());
    say("the pour again, and a book that disagrees", json!({ "again": light::held_words(&again.held[0].1), "a book one token over": refused.agreed.clone().unwrap_err() }));

    // THE WISH VOTES ITS RIDER: its plan of her token, then its allowance to the rider's key
    let month = share.tokens;
    decide(&rpc, wish, &[sw, hw], crank, &rider::decision("the wish may spend her token", rider::plan_act(wish, &p.mint, month, &crank.address), 1), vec![]);
    let admitted = decide(&rpc, wish, &[sw, hw], crank, &rider::decision("the wish's rider", rider::rider_act(wish, &rd.address, &p.mint, month, 10_000), 2), vec![]);
    say("the wish's rider", json!({ "rider": rd.address.to_string(), "allowance": client::allowance_pda(&admitted).to_string(), "cap a period (tokens)": month.to_string(), "period (slots)": 10_000 }));

    // THE RIDER DRAWS AND TAKES IT TO SOL, in one transaction its own wallet signs
    let sol_before = rpc.balance(&rd.address.to_string()).unwrap();
    let behind = pool_now(&rpc, &p).last_update_epoch < epoch(&rpc);
    if behind {
        let paying = || door::rewards_paying(&rpc.account(&door::EPOCH_REWARDS.to_string()).unwrap().map(|a| a.data).unwrap_or_default());
        rpc::wait(Duration::from_secs(120), || Ok((!paying()).then_some(()))).unwrap();
    }
    let worth = door::worth(month, &pool_now(&rpc, &p));
    let drew = land(&rpc, &poleis_door::draw_to_sol(&keys, &client::PROGRAM_ID, wish, &admitted, &rd.address, month, behind.then_some(list(&rpc, &p).as_slice())), rd, &[rd]).unwrap();
    let sol_after = rpc.balance(&rd.address.to_string()).unwrap();
    let fee = rpc.transaction(&drew).unwrap().map(|t| t.1).unwrap_or(0);
    let rent = rpc.rent(165).unwrap();
    assert_eq!(light::balance(&rpc, &client::vault_pda(wish, &p.mint)).unwrap(), 0, "the well paid its rider");
    assert_eq!(sol_after + fee + rent - sol_before, worth, "the rider's draw came out of her reserve at par");
    // a second draw past the cap is refused by Poleis
    let past = land(&rpc, &poleis_door::draw(&keys, &client::PROGRAM_ID, wish, &admitted, &rd.address, 1), rd, &[rd]);
    assert!(past.is_err(), "a draw past the period's cap is refused");
    say("the rider's draw", json!({ "drew (tokens)": month.to_string(), "took to SOL (lamports)": worth.to_string(), "the rider's SOL before": sol_before.to_string(), "after": sol_after.to_string(), "fee": fee, "its receipts account's rent": rent, "signature": drew, "a draw past the cap": past.unwrap_err().chars().take(160).collect::<String>() }));
    say("wall", json!({ "seconds": begun.elapsed().as_secs() }));

    if let Ok(path) = std::env::var("LIGHT_CERTIFICATE") {
        if !path.is_empty() {
            let c = json!({ "schema": "embertide.light-rehearsal/1", "validator": "solana-test-validator, local, 32 slots an epoch", "program": wire::hex(&program.sha256), "said": said });
            std::fs::write(&path, serde_json::to_string_pretty(&c).unwrap()).unwrap();
        }
    }
}

/// THE STAKE PAYS ITS SNOWTRACER (Nov 25 2026, the three links joined): the same road with the yield a validator's own reward, no transfer. Root 0 and
/// her pool born by root 0's act; act 9 (the engine's staker) and act 10 (root 0's plan of her token) voted and executed through Poleis; a wish and its
/// road; two stakers; the engine places her stake on this validator; epochs turn until its reward is minted as fee into root 0's vault; the book, the
/// reader, the pour; the wish's rider; the rider's draw to SOL through a pool that has a validator. Runs under
/// poleis/ceremony/the-stake-pays-its-snowtracer-rehearsal.sh (SNOWTRACER_VALIDATOR).
#[test]
fn the_stake_pays_its_snowtracer_on_a_validator() {
    let (Ok(url), Ok(bin)) = (std::env::var("SNOWTRACER_VALIDATOR"), std::env::var("LIGHT_BOOK")) else {
        eprintln!("SNOWTRACER_VALIDATOR and LIGHT_BOOK are not set: the joint half runs under poleis/ceremony/the-stake-pays-its-snowtracer-rehearsal.sh");
        return;
    };
    use poleis_press::engine;
    let begun = std::time::Instant::now();
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };
    let dir = std::path::PathBuf::from(std::env::var("LIGHT_DIR").unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into()));
    let data = rpc.account(&loader::programdata_of(&STAKE_POOL).to_string()).unwrap().expect("the stake pool program's data on this validator");
    let program = Program::of(&STAKE_POOL, &data.data).unwrap();
    if let Ok(want) = std::env::var("LIGHT_PROGRAM_SHA256") {
        assert_eq!(wire::hex(&program.sha256), want, "the stake pool program's bytes on this validator");
    }

    let seed = wire::sha256(format!("the stake pays its snowtracer {url} {}", std::process::id()).as_bytes());
    let names = ["root 0's starter", "root 0's first hexagon", "the wish's starter", "the wish's first hexagon", "the crank", "staker 1", "staker 2", "the rider", "the engine"];
    let mut ks: Vec<Key> = names.iter().map(|n| wire::standin(&seed, n)).collect();
    if let Ok(f) = std::env::var("LIGHT_DEPLOYER") {
        ks[0] = Key::from_cli_text(&std::fs::read_to_string(f).unwrap()).unwrap();
    }
    let [s0, h0, sw, hw, crank, st1, st2, rd, eng] = [&ks[0], &ks[1], &ks[2], &ks[3], &ks[4], &ks[5], &ks[6], &ks[7], &ks[8]];
    for (k, sol) in [(s0, 5), (h0, 5), (sw, 5), (hw, 5), (crank, 50), (st1, 160), (st2, 60), (rd, 1), (eng, 10)] {
        rpc.airdrop(&k.address.to_string(), sol * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || {
        for k in &ks {
            if rpc.balance(&k.address.to_string())? < SOL {
                return Ok(None);
            }
        }
        Ok(Some(()))
    })
    .unwrap();

    // ROOT 0 AND A WISH
    let words = "A publisher of games that use the blockchain as their multiplayer.";
    let wish = u64::from_le_bytes(wire::sha256(words.as_bytes())[..8].try_into().unwrap()).max(1);
    let (salt0, saltw) = ([1u8; 32], [2u8; 32]);
    let g0 = found(&rpc, ROOT_UNIT, s0, h0, "The meta-swarm", "Her: the root of them all.", &salt0);
    let gw = found(&rpc, wish, sw, hw, "The Game Publisher", words, &saltw);
    ratify(&rpc, &[(ROOT_UNIT, &g0, h0, &salt0), (wish, &gw, hw, &saltw)], crank);

    // ACT 5, HER POOL: born by root 0's raw act, as the launch births it
    let p = Pool::derive(&crank.address, "stake1", 10, &STAKE_POOL, &client::PROGRAM_ID).unwrap();
    let (_, ahead) = p.ahead(rpc.rent(RESERVE_LEN).unwrap(), rpc.rent(MINT_LEN).unwrap(), [None, None, None]).unwrap();
    land(&rpc, &ahead, crank, &[crank]).unwrap();
    let mut makings = Vec::new();
    for s in p.in_the_execute() {
        makings.extend(s.making(&crank.address, rpc.rent(s.space).unwrap(), None).unwrap());
    }
    decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("act 5: her pool is born", p.payload(), 5), makings);
    let keys = door::keys(&p);

    // ACT 9, THE ENGINE'S STAKER: SetStaker by the polis's own hand, through Poleis
    let nine = decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("act 9: the engine's staker", engine::payload(&p, &eng.address), 9), vec![]);
    let staker_now = Pubkey::new_from_array(rpc.account(&p.pool.to_string()).unwrap().unwrap().data[engine::STAKER..engine::STAKER + 32].try_into().unwrap());
    assert_eq!(staker_now, eng.address, "her pool's staker is the engine's key, named by the swarm's act");
    // ACT 10, THE LIGHT'S PLAN: root 0's AllowMint of her token, through Poleis
    let cap = 1_000 * SOL;
    let ten = decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("act 10: root 0's plan of her token", rider::plan_act(ROOT_UNIT, &p.mint, cap, &crank.address), 10), vec![]);
    // THE WISH'S ROAD, a set press well writes once the wish is born
    let road = decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("root 0's road into the wish's well", rider::well_act(wish, &p.mint, cap, 32), 11), vec![]);
    say("the acts, in order", json!({ "act 5 (her pool)": p.pool.to_string(), "act 9 (the engine's staker)": { "hash": wire::hex(&nine), "staker": staker_now.to_string() }, "act 10 (root 0's plan of her token)": wire::hex(&ten), "the wish's road": client::allowance_pda(&road).to_string(), "the wish": wish.to_string() }));

    // TWO STAKERS: one light to the wish, one none
    for (k, sol) in [(st1, 150), (st2, 50)] {
        bring(&rpc, &p, crank);
        land(&rpc, &door::stake(&p, &k.address, sol * SOL, None), k, &[k]).unwrap();
    }
    land(&rpc, &[door::light(&st1.address, Light::Wish(wish))], st1, &[st1]).unwrap();

    // THE BOOK, FIRST READ
    let book = dir.join("her-book.json");
    count(&rpc, &url, &bin, &p, &book, crank);
    let b0 = book_at(&book, &p);

    // THE ENGINE: the rule's validator (this one) added, the reserve above its kept tenth placed on it
    let params = poleis_press::params::Params::read(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json"))).unwrap();
    let rule = engine::rule_on(&params, &rpc).unwrap();
    let r1 = engine::run(&rpc, &p.pool, &rule, &eng.address, Some(eng)).unwrap();
    say("the engine places her stake", json!({ "epoch": r1.epoch, "fees": r1.fees, "after": r1.after }));

    // EPOCHS TURN until a validator's reward is minted as fee into root 0's vault; the engine runs each epoch (it brings the pool to the epoch)
    let vault0 = light::balance(&rpc, &p.vault).unwrap();
    let mut turns = Vec::new();
    for _ in 0..10 {
        let e = epoch(&rpc);
        rpc::wait(Duration::from_secs(120), || Ok((epoch(&rpc) > e).then_some(()))).unwrap();
        let paying = || door::rewards_paying(&rpc.account(&door::EPOCH_REWARDS.to_string()).unwrap().map(|a| a.data).unwrap_or_default());
        rpc::wait(Duration::from_secs(120), || Ok((!paying()).then_some(()))).unwrap();
        let r = engine::run(&rpc, &p.pool, &rule, &eng.address, Some(eng)).unwrap();
        let v = light::balance(&rpc, &p.vault).unwrap();
        turns.push(json!({ "epoch": r.epoch, "fee account (tokens)": v.to_string(), "total": r.after["total"], "validator": r.after["validators"][0] }));
        if v > vault0 && r.after["validators"][0]["transient"].as_u64() == Some(0) {
            break;
        }
    }
    let minted = light::balance(&rpc, &p.vault).unwrap() - vault0;
    assert!(minted > 0, "a validator's reward reached root 0's vault as her token");
    say("epochs turned", json!({ "turns": turns, "fee minted into root 0's vault (tokens)": minted.to_string() }));

    // THE BOOK, SECOND READ; THE READER AND THE POUR
    count(&rpc, &url, &bin, &p, &book, crank);
    let b1 = book_at(&book, &p);
    let row = b1.row.clone().unwrap();
    let c = light::crank(&rpc, &p.pool, Some(&b0), &b1, &crank.address).unwrap();
    assert!(c.agreed.is_ok(), "{:?}", c.agreed);
    assert_eq!(row.fee_tokens, c.chain_fee, "the book and the chain");
    let share = c.reading.wishes.get(&wish).cloned().expect("the wish's share");
    let l = |k: &Key| c.reading.stakers.iter().find(|s| s.0 == k.address.to_string()).map(|s| s.1).unwrap_or(0);
    let sent = light::send(&rpc, &c, crank).unwrap();
    let in_well = light::balance(&rpc, &client::vault_pda(wish, &p.mint)).unwrap();
    assert_eq!(in_well, share.tokens);
    say("the reader and the pour", json!({
        "epochs": [row.from, row.epoch], "fee (tokens)": row.fee_tokens.to_string(), "fee (lamports)": row.fee_lamports.to_string(), "the chain's mints (tokens)": c.chain_fee.to_string(),
        "staker 1's light (lamports)": l(st1).to_string(), "staker 2's light (lamports)": l(st2).to_string(), "to the wish (tokens)": share.tokens.to_string(),
        "undirected, in root 0's vault (tokens)": c.reading.undirected_tokens.to_string(), "dust (tokens)": c.reading.dust_tokens.to_string(), "over-pulled (tokens)": c.over_pulled().to_string(),
        "in the wish's well (tokens)": in_well.to_string(), "left in root 0's vault (tokens)": light::balance(&rpc, &p.vault).unwrap().to_string(), "signatures": sent,
    }));
    let again = light::crank(&rpc, &p.pool, Some(&b0), &b1, &crank.address).unwrap();
    assert!(again.pours.is_empty(), "poured once");

    // THE WISH'S RIDER, and the draw to SOL through a pool with a validator
    let month = share.tokens;
    decide(&rpc, wish, &[sw, hw], crank, &rider::decision("the wish may spend her token", rider::plan_act(wish, &p.mint, month, &crank.address), 1), vec![]);
    let admitted = decide(&rpc, wish, &[sw, hw], crank, &rider::decision("the wish's rider", rider::rider_act(wish, &rd.address, &p.mint, month, 10_000), 2), vec![]);
    let sol_before = rpc.balance(&rd.address.to_string()).unwrap();
    let behind = pool_now(&rpc, &p).last_update_epoch < epoch(&rpc);
    if behind {
        let paying = || door::rewards_paying(&rpc.account(&door::EPOCH_REWARDS.to_string()).unwrap().map(|a| a.data).unwrap_or_default());
        rpc::wait(Duration::from_secs(120), || Ok((!paying()).then_some(()))).unwrap();
    }
    let validators = list(&rpc, &p);
    let worth = door::worth(month, &pool_now(&rpc, &p));
    let drew = land(&rpc, &poleis_door::draw_to_sol(&keys, &client::PROGRAM_ID, wish, &admitted, &rd.address, month, behind.then_some(validators.as_slice())), rd, &[rd]).unwrap();
    let (_, fee, units, _) = rpc.transaction(&drew).unwrap().unwrap();
    let sol_after = rpc.balance(&rd.address.to_string()).unwrap();
    let rent = rpc.rent(165).unwrap();
    assert_eq!(light::balance(&rpc, &client::vault_pda(wish, &p.mint)).unwrap(), 0, "the well paid its rider");
    say("the rider's draw to SOL", json!({ "validators in her list": validators.len(), "the pool behind the epoch": behind, "drew (tokens)": month.to_string(), "worth at the rate before (lamports)": worth.to_string(), "the rider's SOL before": sol_before.to_string(), "after": sol_after.to_string(), "fee": fee, "units": units, "its receipts account's rent": rent, "SOL the draw brought (lamports)": (sol_after + fee + rent - sol_before).to_string(), "signature": drew }));
    assert_eq!(sol_after + fee + rent - sol_before, worth, "at par, out of her reserve");
    say("wall", json!({ "seconds": begun.elapsed().as_secs() }));
    if let Ok(path) = std::env::var("LIGHT_CERTIFICATE") {
        if !path.is_empty() {
            let c = json!({ "schema": "embertide.stake-pays-its-snowtracer-rehearsal/1", "validator": "solana-test-validator, local, 32 slots an epoch; its own vote account the rule's one validator", "program": wire::hex(&program.sha256), "said": said });
            std::fs::write(&path, serde_json::to_string_pretty(&c).unwrap()).unwrap();
        }
    }
}

/// THE MONEY'S ROAD ON THE LAUNCH'S TREE (THE NEW ACTS IN THE LAUNCH REHEARSAL, Nov 25 2026): not run by any script yet (the launch rehearsal runs on the
/// validator's default epochs, and this road needs short ones: poleis/ceremony/THE-NEW-ACTS-IN-THE-LAUNCH-REHEARSAL-LANDING.md says what is owed). Given a
/// validator where acts 9 and 10 passed root 0's two chambers (LAUNCH_ROAD_* below): her pool as the launch's act 5 left it and act 9
/// named its staker; a staker stakes through the door; the engine (its key the one act 9 named) places the stake on this validator; epochs turn
/// until the validator's reward is minted as fee into root 0's vault, read by the book's own law of the fee's mints. It stops there: a wish under
/// root 0 and its road would be more acts of the whole tree than the press carries (its sets are the launch's and the later acts' alone).
#[test]
fn the_money_road_on_the_launchs_tree() {
    let Ok(url) = std::env::var("LAUNCH_ROAD_VALIDATOR") else {
        eprintln!("LAUNCH_ROAD_VALIDATOR is not set: the money's road on the launch's tree runs under the launch rehearsal (LAUNCH_LATER_ACTS=1)");
        return;
    };
    use poleis_press::engine;
    let env = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k} is not set"));
    let begun = std::time::Instant::now();
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let pool: Pubkey = std::str::FromStr::from_str(&env("LAUNCH_ROAD_POOL")).expect("LAUNCH_ROAD_POOL: her pool's address");
    let manager: Pubkey = std::str::FromStr::from_str(&env("LAUNCH_ROAD_MANAGER")).expect("LAUNCH_ROAD_MANAGER: the polis's hand");
    let eng = Key::from_cli_text(&std::fs::read_to_string(env("LAUNCH_ROAD_ENGINE_KEY")).unwrap()).unwrap();
    let params = poleis_press::params::Params::read(std::path::Path::new(&env("LAUNCH_ROAD_PARAMS"))).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("THE MONEY'S ROAD: {what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };
    let a = rpc.account(&pool.to_string()).unwrap().expect("her pool on this chain");
    let keys = door::Keys::of_account(&pool, &std::str::FromStr::from_str(&a.owner).unwrap(), &a.data).unwrap();
    let at = |o: usize| Pubkey::new_from_array(a.data[o..o + 32].try_into().unwrap());
    assert_eq!((at(1), at(engine::STAKER)), (manager, eng.address), "her pool's manager the polis's hand, its staker the engine's key act 9 named");
    say("her pool, after acts 5 and 9", json!({ "pool": pool.to_string(), "manager": manager.to_string(), "staker": eng.address.to_string(), "fee account (root 0's vault)": keys.vault.to_string(), "epoch fee": "1/1" }));

    let seed = wire::sha256(format!("the money's road {url} {}", std::process::id()).as_bytes());
    let staker = wire::standin(&seed, "a staker of the launch's tree");
    for k in [&staker, &eng] {
        rpc.airdrop(&k.address.to_string(), 120 * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || Ok((rpc.balance(&staker.address.to_string())? >= 120 * SOL && rpc.balance(&eng.address.to_string())? >= SOL).then_some(()))).unwrap();
    let turn = |rpc: &Rpc| {
        let paying = || door::rewards_paying(&rpc.account(&door::EPOCH_REWARDS.to_string()).unwrap().map(|a| a.data).unwrap_or_default());
        rpc::wait(Duration::from_secs(120), || Ok((!paying()).then_some(()))).unwrap();
    };
    let behind = |rpc: &Rpc| door::pool_now(&rpc.account(&pool.to_string()).unwrap().unwrap().data).unwrap().last_update_epoch < epoch(rpc);
    // THE STAKE, through the door: her list read off the chain and named when the pool is behind
    turn(&rpc);
    let list = |rpc: &Rpc| poleis_door::pairs(&keys, &rpc.account(&keys.list.to_string()).unwrap().unwrap().data).unwrap();
    let b = behind(&rpc).then(|| list(&rpc));
    let paid0 = rpc.balance(&staker.address.to_string()).unwrap();
    let sig = land(&rpc, &poleis_door::stake(&keys, &staker.address, 100 * SOL, b.as_deref()), &staker, &[&staker]).unwrap();
    let receipts = light::balance(&rpc, &door::receipts_of(&staker.address, &keys.mint)).unwrap();
    say("the stake, through the door", json!({ "lamports": 100 * SOL, "receipts": receipts.to_string(), "the staker paid": paid0 - rpc.balance(&staker.address.to_string()).unwrap(), "signature": sig }));

    // THE ENGINE: the rule's validator (this validator's own vote account) added, the reserve above its kept share placed on it
    let rule = engine::rule_on(&params, &rpc).unwrap();
    let r1 = engine::run(&rpc, &pool, &rule, &eng.address, Some(&eng)).unwrap();
    say("the engine places the stake", json!({ "epoch": r1.epoch, "fees": r1.fees, "said": r1.said, "after": r1.after }));
    let first_epoch = epoch(&rpc);

    // EPOCHS TURN until the validator's reward is minted as fee into root 0's vault; the engine runs each epoch (it brings the pool to it)
    let vault0 = light::balance(&rpc, &keys.vault).unwrap();
    let mut turns = Vec::new();
    for _ in 0..12 {
        let e = epoch(&rpc);
        rpc::wait(Duration::from_secs(180), || Ok((epoch(&rpc) > e).then_some(()))).unwrap();
        turn(&rpc);
        let r = engine::run(&rpc, &pool, &rule, &eng.address, Some(&eng)).unwrap();
        let v = light::balance(&rpc, &keys.vault).unwrap();
        turns.push(json!({ "epoch": r.epoch, "fee account (tokens)": v.to_string(), "total": r.after["total"], "validators": r.after["validators"], "engine fees": r.fees }));
        if v > vault0 && r.after["validators"][0]["transient"].as_u64() == Some(0) {
            break;
        }
    }
    let minted = light::balance(&rpc, &keys.vault).unwrap() - vault0;
    assert!(minted > 0, "a validator's reward reached root 0's vault as her pool's token");
    let sched = light::schedule(&rpc).unwrap();
    let chain = light::chain_fee(&rpc, &keys, &sched, first_epoch, epoch(&rpc)).unwrap();
    assert_eq!(chain, minted, "the vault's growth is the pool program's own fee mints, as the book's law reads them");
    let now = door::pool_now(&rpc.account(&pool.to_string()).unwrap().unwrap().data).unwrap();
    say("the validator's reward, minted as fee into root 0's vault", json!({ "turns": turns, "fee minted (tokens)": minted.to_string(), "the fee's mints read by the book's law (tokens)": chain.to_string(), "worth (lamports)": door::worth(minted, &now).to_string(), "the staker's receipts worth (lamports)": door::worth(receipts, &now).to_string() }));
    say("where it stops", json!("here, at the fee in root 0's vault: founding a wish under root 0 and voting root 0's road into its well on this tree are acts of the whole tree the press does not carry (press vote, press chair and press crank take the launch's sets and the later acts' alone; press well's set is not one of them)"));
    say("wall", json!({ "seconds": begun.elapsed().as_secs() }));
    if let Ok(path) = std::env::var("LAUNCH_ROAD_CERTIFICATE") {
        let c = json!({ "schema": "embertide.the-money-road-on-the-launchs-tree/1", "said": said });
        std::fs::write(&path, serde_json::to_string_pretty(&c).unwrap()).unwrap();
    }
}

/// THE EPOCH TURNS THE CRANKS, live (Nov 25 2026): the joint rehearsal's world (her pool by act 5, acts 9 and 10, a wish and its road, two stakers,
/// one light to the wish), then `press epoch --stay` started as its own process and NO crank run by hand: it carries the epochs by itself while this
/// test only reads, a staker leaving in a middle epoch; then it is stopped and nothing of it is left. Runs under poleis/ceremony/the-epoch-rehearsal.sh
/// (EPOCH_VALIDATOR).
#[test]
fn the_epoch_turns_the_cranks_on_a_validator() {
    let Ok(url) = std::env::var("EPOCH_VALIDATOR") else {
        eprintln!("EPOCH_VALIDATOR is not set: the runner's live half runs under poleis/ceremony/the-epoch-rehearsal.sh");
        return;
    };
    use poleis_press::engine;
    let begun = std::time::Instant::now();
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };
    let dir = std::path::PathBuf::from(std::env::var("LIGHT_DIR").unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into())).join("sitting");
    let seed = wire::sha256(format!("the epoch turns the cranks {url} {}", std::process::id()).as_bytes());
    let names = ["root 0's starter", "root 0's first hexagon", "the wish's starter", "the wish's first hexagon", "the crank", "staker 1", "staker 2", "the engine", "the pourer"];
    let mut ks: Vec<Key> = names.iter().map(|n| wire::standin(&seed, n)).collect();
    if let Ok(f) = std::env::var("LIGHT_DEPLOYER") {
        ks[0] = Key::from_cli_text(&std::fs::read_to_string(f).unwrap()).unwrap();
    }
    let [s0, h0, sw, hw, crank, st1, st2, eng, pourer] = [&ks[0], &ks[1], &ks[2], &ks[3], &ks[4], &ks[5], &ks[6], &ks[7], &ks[8]];
    for (k, sol) in [(s0, 5), (h0, 5), (sw, 5), (hw, 5), (crank, 50), (st1, 160), (st2, 60), (eng, 10), (pourer, 5)] {
        rpc.airdrop(&k.address.to_string(), sol * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || {
        for k in &ks {
            if rpc.balance(&k.address.to_string())? < SOL {
                return Ok(None);
            }
        }
        Ok(Some(()))
    })
    .unwrap();

    // THE WORLD, as the joint rehearsal makes it
    let words = "A publisher of games that use the blockchain as their multiplayer.";
    let wish = u64::from_le_bytes(wire::sha256(words.as_bytes())[..8].try_into().unwrap()).max(1);
    let (salt0, saltw) = ([1u8; 32], [2u8; 32]);
    let g0 = found(&rpc, ROOT_UNIT, s0, h0, "The meta-swarm", "Her: the root of them all.", &salt0);
    let gw = found(&rpc, wish, sw, hw, "The Game Publisher", words, &saltw);
    ratify(&rpc, &[(ROOT_UNIT, &g0, h0, &salt0), (wish, &gw, hw, &saltw)], crank);
    let p = Pool::derive(&crank.address, "epoch1", 10, &STAKE_POOL, &client::PROGRAM_ID).unwrap();
    let (_, ahead) = p.ahead(rpc.rent(RESERVE_LEN).unwrap(), rpc.rent(MINT_LEN).unwrap(), [None, None, None]).unwrap();
    land(&rpc, &ahead, crank, &[crank]).unwrap();
    let mut makings = Vec::new();
    for s in p.in_the_execute() {
        makings.extend(s.making(&crank.address, rpc.rent(s.space).unwrap(), None).unwrap());
    }
    decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("act 5: her pool is born", p.payload(), 5), makings);
    decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("act 9: the engine's staker", engine::payload(&p, &eng.address), 9), vec![]);
    let cap = 1_000 * SOL;
    decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("act 10: root 0's plan of her token", rider::plan_act(ROOT_UNIT, &p.mint, cap, &crank.address), 10), vec![]);
    decide(&rpc, ROOT_UNIT, &[s0, h0], crank, &rider::decision("root 0's road into the wish's well", rider::well_act(wish, &p.mint, cap, 32), 11), vec![]);
    for (k, sol) in [(st1, 150), (st2, 50)] {
        bring(&rpc, &p, crank);
        land(&rpc, &door::stake(&p, &k.address, sol * SOL, None), k, &[k]).unwrap();
    }
    land(&rpc, &[door::light(&st1.address, Light::Wish(wish))], st1, &[st1]).unwrap();
    say("the world", json!({ "pool": p.pool.to_string(), "wish": wish.to_string(), "staker 1": "150 SOL, light to the wish", "staker 2": "50 SOL, no choice" }));

    // THE RUNNER'S SITTING: its two keys made once beside the sitting's keys, the params naming the engine's key and the light's cap
    poleis_press::seat::engine_key(&dir, || Ok(wire::standin(&seed, "the engine"))).unwrap();
    poleis_press::seat::pourer_key(&dir, || Ok(wire::standin(&seed, "the pourer"))).unwrap();
    let mut pv: Value = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json")).unwrap()).unwrap();
    pv["engine"]["staker"] = json!(eng.address.to_string());
    pv["light"]["planCap"] = json!(cap);
    let params = dir.join("params.json");
    std::fs::write(&params, pv.to_string()).unwrap();
    let log = dir.join("epoch.log");
    // A MISSED EPOCH'S LIGHT IS POURED LATER: EPOCH_STOP=1 ends the runner's process for two whole epochs in the middle, then starts it again
    let stop = std::env::var("EPOCH_STOP").is_ok_and(|v| v == "1");
    let spawn = || {
        let open = |f: &std::path::Path| std::fs::OpenOptions::new().create(true).append(true).open(f).unwrap();
        std::process::Command::new(env!("CARGO_BIN_EXE_press"))
            .args(["epoch", "--stay", "--dir", dir.to_str().unwrap(), "--params", params.to_str().unwrap(), "--rpc", &url, "--pool", &p.pool.to_string(), "--margin-slots", "0"])
            .stdout(open(&log))
            .stderr(open(&dir.join("epoch.err")))
            .spawn()
            .unwrap()
    };
    let mut child = spawn();
    let mut pid = child.id();
    // a test that panics ends its runner too: the epoch rehearsal's first two runs left theirs running, past the validator's end
    struct Ends(std::rc::Rc<std::cell::Cell<u32>>);
    impl Drop for Ends {
        fn drop(&mut self) {
            let _ = std::process::Command::new("kill").arg(self.0.get().to_string()).status();
        }
    }
    let running = std::rc::Rc::new(std::cell::Cell::new(pid));
    let _ends = Ends(running.clone());
    let mut stopped: Option<(u64, u64)> = None;
    say("the runner started", json!({ "pid": pid, "its keys": { "engine": eng.address.to_string(), "pourer": pourer.address.to_string() }, "its state": dir.join("epoch").display().to_string() }));

    // ITS EPOCHS, read and never touched: per epoch, her pool, root 0's vault and the wish's well; staker 2 leaves in a middle epoch
    let well = client::vault_pda(wish, &p.mint);
    let mut rows: Vec<Value> = Vec::new();
    let mut poured_epochs: Vec<u64> = Vec::new();
    let mut left_at: Option<u64> = None;
    let start_epoch = epoch(&rpc);
    let run_of = |v: &[u64]| v.windows(3).any(|w| w[1] == w[0] + 1 && w[2] == w[1] + 1);
    let until: u64 = std::env::var("EPOCH_UNTIL").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    while (!run_of(&poured_epochs) || epoch(&rpc) < until) && epoch(&rpc) < start_epoch + 18 {
        let e = epoch(&rpc);
        rpc::wait(Duration::from_secs(180), || Ok((epoch(&rpc) > e).then_some(()))).unwrap();
        // the runner's pass at the epoch's start: let it finish (its line for this epoch, or a few seconds)
        let target = epoch(&rpc);
        let _ = rpc::wait(Duration::from_secs(40), || Ok(std::fs::read_to_string(&log).unwrap_or_default().lines().any(|l| l.contains(&format!(" epoch {target}: "))).then_some(())));
        std::thread::sleep(Duration::from_secs(2));
        let now = door::pool_now(&rpc.account(&p.pool.to_string()).unwrap().unwrap().data).unwrap();
        let line = std::fs::read_to_string(&log).unwrap().lines().filter(|l| l.contains(&format!(" epoch {target}: "))).last().unwrap_or_default().to_string();
        let row = json!({ "the runner's epoch": target, "read at epoch": epoch(&rpc), "pool's epoch": now.last_update_epoch, "pool total": now.total_lamports.to_string(), "root 0's vault": light::balance(&rpc, &p.vault).unwrap().to_string(), "the wish's well": light::balance(&rpc, &well).unwrap().to_string(), "the runner said": line });
        eprintln!("EPOCH: {row}");
        rows.push(row);
        // the stop: after the runner's second pour, its process ended, two whole epochs pass with no runner at all, then it is started again
        if stop && stopped.is_none() && poured_epochs.len() == 2 {
            std::process::Command::new("kill").arg(pid.to_string()).status().unwrap();
            let _ = child.wait();
            let down = epoch(&rpc);
            rpc::wait(Duration::from_secs(180), || Ok((epoch(&rpc) >= down + 3).then_some(()))).unwrap();
            child = spawn();
            pid = child.id();
            running.set(pid);
            let up = epoch(&rpc);
            stopped = Some((down, up));
            say("the runner stopped and started again", json!({ "ended in epoch": down, "started again in epoch": up, "epochs with no runner": up - down - 1, "new pid": pid }));
            continue;
        }
        if line.contains("tokens to wish") {
            poured_epochs.push(target);
            // the middle epoch: after the first pour, staker 2 takes back what her reserve holds
            if left_at.is_none() {
                let held = light::balance(&rpc, &door::receipts_of(&st2.address, &p.mint)).unwrap();
                let free = rpc.account(&p.reserve.to_string()).unwrap().unwrap().lamports - rpc.rent(RESERVE_LEN).unwrap();
                let now = door::pool_now(&rpc.account(&p.pool.to_string()).unwrap().unwrap().data).unwrap();
                let out = held.min((free as u128 * now.pool_token_supply as u128 / now.total_lamports.max(1) as u128) as u64).saturating_sub(1);
                let behind = now.last_update_epoch < epoch(&rpc);
                let sig = land(&rpc, &door::unstake(&p, &st2.address, out, behind.then_some(list(&rpc, &p).as_slice())), st2, &[st2]).unwrap();
                left_at = Some(target);
                say("staker 2 leaves", json!({ "epoch": target, "receipts burned": out.to_string(), "of": held.to_string(), "the reserve held (lamports)": free.to_string(), "signature": sig }));
            }
        }
    }
    // its cost at rest, read off the process before it is stopped
    let cpu = std::process::Command::new("ps").args(["-o", "time=", "-p", &pid.to_string()]).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    std::process::Command::new("kill").arg(pid.to_string()).status().unwrap();
    let status = child.wait().unwrap();
    let alive = std::process::Command::new("ps").args(["-p", &pid.to_string()]).output().map(|o| o.status.success()).unwrap_or(false);
    let lines: Vec<String> = std::fs::read_to_string(&log).unwrap().lines().map(str::to_string).collect();
    let errs = std::fs::read_to_string(dir.join("epoch.err")).unwrap_or_default();
    say("the runner stopped", json!({ "its CPU time": cpu, "ended by": format!("{status}"), "still running": alive, "its lines": lines, "its errors": errs }));
    say("its epochs", json!(rows));
    // EVERY EPOCH'S LIGHT THE BOOK COUNTED REACHED THE WELL: the runner's own pours summed, against the well, and the book's own sums beside them
    let mut pours: Vec<(u64, u64)> = Vec::new();
    for l in &lines {
        for part in l.split(", ") {
            let w: Vec<&str> = part.split_whitespace().collect();
            if let Some(i) = w.iter().position(|x| *x == "tokens") {
                if w.get(i + 1) == Some(&"to") && w.get(i + 3) == Some(&wish.to_string().as_str()) && w.get(i + 4) == Some(&"for") {
                    let n = w[i - 1].trim_start_matches("poured ").parse::<u64>().unwrap_or(0);
                    let e = w.get(i + 6).and_then(|x| x.trim_end_matches(';').parse::<u64>().ok()).unwrap_or(0);
                    pours.push((e, n));
                }
            }
        }
    }
    let poured_total: u64 = pours.iter().map(|x| x.1).sum();
    let in_well = light::balance(&rpc, &well).unwrap();
    let owed_left: Vec<u64> = poleis_press::epoch::owed_list(&dir, &p.pool).unwrap().iter().map(|o| o.epoch).collect();
    let book: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("epoch/book.json")).unwrap()).unwrap();
    let light_of_1 = book["stakers"][st1.address.to_string()]["light"].as_str().unwrap_or("0").to_string();
    say("every epoch's light reached the well", json!({ "the runner's pours, by counted epoch": pours, "their sum (tokens)": poured_total.to_string(), "the wish's well (tokens)": in_well.to_string(), "epochs still owed": owed_left, "staker 1's light in the book (lamports, every count)": light_of_1, "the book's fee over every count (lamports)": book["fee_lamports"], "the book's dust": book["dust"], "stopped": stopped }));
    assert_eq!(in_well, poured_total, "the well holds exactly what the runner poured");
    assert!(owed_left.is_empty(), "no counted epoch is left owed: {owed_left:?}");
    if stop {
        assert!(stopped.is_some(), "the runner was stopped and started again");
    }
    // ONE PASS, TWICE: the runner's single pass run by hand twice in a row, after it was stopped; the second, in the same epoch, sends nothing
    let once = || {
        let o = std::process::Command::new(env!("CARGO_BIN_EXE_press"))
            .args(["epoch", "--dir", dir.to_str().unwrap(), "--params", params.to_str().unwrap(), "--rpc", &url, "--pool", &p.pool.to_string()])
            .output()
            .unwrap();
        (epoch(&rpc), String::from_utf8_lossy(&o.stdout).trim().to_string(), String::from_utf8_lossy(&o.stderr).trim().to_string())
    };
    let (e1, first, _) = once();
    let (e2, second, err2) = once();
    say("one pass, twice", json!({ "first": first, "second": second, "its errors": err2, "epochs": [e1, e2] }));
    if e1 == e2 {
        assert!(second.contains("nothing due"), "the second pass in an epoch sends nothing: {second}");
    }
    // this test's own sends (the world, the staker's leave), by the runner's sender: how many went again and were built again
    let mine = poleis_press::epoch::sends_taken();
    say("this test's own sends, by the runner's sender", json!({ "landed": mine.landed, "sent again": mine.resent, "built again": mine.rebuilt, "fees": mine.fees }));
    assert!(!alive, "nothing of the runner is left");
    assert!(run_of(&poured_epochs), "three consecutive epochs poured by the runner alone: {poured_epochs:?}");
    assert!(left_at.is_some(), "a staker left in a middle epoch");
    say("wall", json!({ "seconds": begun.elapsed().as_secs() }));
    if let Ok(path) = std::env::var("EPOCH_CERTIFICATE") {
        let c = json!({ "schema": "embertide.the-epoch-turns-the-cranks-rehearsal/1", "validator": "solana-test-validator, local, 32 slots an epoch; its own vote account the rule's one validator", "said": said });
        std::fs::write(&path, serde_json::to_string_pretty(&c).unwrap()).unwrap();
    }
}
