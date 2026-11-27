//! THE STAKE EARNS, proven (Nov 25 2026, lane/the-stake-earns; his "Yes, build them" to "Build the three missing links now?", link 3). Two halves:
//! - pure, at every gate: the rule read from the params and refused where it is bad, its arithmetic (the kept share, the targets by weight),
//!   the engine's order (the update first, the adds, the moves by weight, one transient a validator an epoch, stake brought back when the
//!   reserve is short, nothing due twice), the stake pool program's instructions by their bytes and account orders, and the ninth act (the
//!   pool's SetStaker by the polis's hand, naming the engine's key) as the launch's acts are written;
//! - live, on a validator holding mainnet's stake pool program (ENGINE_VALIDATOR, which poleis/ceremony/the-engine-rehearsal.sh sets): a pool of
//!   the launch's shape born by a stand-in manager, a stake, the SetStaker the act carries sent by that manager, the crank run (the validator
//!   added, the reserve above its kept share placed), run again (nothing), epochs turned until the stake earns, the fee read in the fee account,
//!   and the way out at par, the crank bringing stake back to keep it open. Without ENGINE_VALIDATOR the live half says so and passes.

use poleis_client::{self as client, Payload, ACT_SIGNER, ACT_WRITABLE, CLASS_ACT, ROOT_UNIT};
use poleis_press::door;
use poleis_press::engine::{self, Chain, Entry, Move, Rule, Step, Validator};
use poleis_press::loader;
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::pool::{Pool, Program, MINT_LEN, RESERVE_LEN, STAKE, STAKE_POOL};
use poleis_press::rpc::{self, Cluster, Rpc};
use poleis_press::wire::{self, Key};
use poleis_press::{acts, build, chain};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::time::Duration;

const SOL: u64 = 1_000_000_000;
const RENT: u64 = 2_282_880;

fn rehearsal() -> Params {
    Params::read(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json"))).unwrap()
}
fn mainnet() -> Params {
    Params::read(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.mainnet.json"))).unwrap()
}
fn with(p: &Params, engine: Value) -> Params {
    let mut v = p.raw.clone();
    v["engine"] = engine;
    Params::from_text(&v.to_string()).unwrap()
}
fn keys() -> door::Keys {
    let p = Pool::derive(&k("engine base"), "eng1", 10, &STAKE_POOL, &k("poleis")).unwrap();
    door::keys(&p)
}
fn entry(vote: Pubkey, active: u64, transient: u64) -> Entry {
    Entry { active, transient, last_update_epoch: 7, transient_seed: 0, validator_seed: 0, status: 0, vote }
}
fn chain_of(engine: &Pubkey, total: u64, reserve: u64, entries: Vec<Entry>) -> Chain {
    Chain { epoch: 7, pool_epoch: 7, staker: *engine, total, max_validators: 10, entries, reserve, stake_rent: RENT, min_delegation: SOL, rewards_paying: false }
}
fn metas(ix: &Instruction) -> Vec<(Pubkey, bool, bool)> {
    ix.accounts.iter().map(|a| (a.pubkey, a.is_signer, a.is_writable)).collect()
}

#[test]
fn the_rule_is_the_params_and_a_bad_one_is_refused() {
    let own = k("this validator's vote");
    let r = Rule::of(&rehearsal(), Some(&own)).unwrap();
    assert_eq!(r, Rule { validators: vec![Validator { vote: own, weight: 1 }], reserve_bp: 1000 }, "the rehearsal's: its validator's own, a tenth kept");
    assert!(Rule::of(&rehearsal(), None).unwrap_err().contains("not read"), "the rehearsal's word needs the validator read");
    let mr = Rule::of(&mainnet(), None).unwrap();
    assert_eq!((mr.reserve_bp, mr.validators.len()), (1000, 10), "mainnet's starting rule: 10% reserve, 10 live 0%-commission validators");
    let blank = with(&mainnet(), json!({ "staker": "PLACEHOLDER", "reserveBp": "PLACEHOLDER: his", "validators": "PLACEHOLDER: his" }));
    assert!(Rule::of(&blank, None).unwrap_err().contains("not his yet"), "a blank rule is refused");
    let m = with(&mainnet(), json!({ "staker": "PLACEHOLDER", "reserveBp": 1000, "validators": [{ "vote": "THIS VALIDATOR", "weight": 1 }] }));
    assert!(Rule::of(&m, Some(&own)).unwrap_err().contains("rehearsal's word"), "the word is never mainnet's");
    let (a, b) = (k("validator a").to_string(), k("validator b").to_string());
    for (bad, says) in [
        (json!({ "reserveBp": 10001, "validators": [{ "vote": a, "weight": 1 }] }), "0 to 10000"),
        (json!({ "reserveBp": 1000, "validators": [] }), "at least one"),
        (json!({ "reserveBp": 1000, "validators": [{ "vote": a, "weight": 0 }] }), "more than nothing"),
        (json!({ "reserveBp": 1000, "validators": [{ "vote": a, "weight": 1 }, { "vote": a, "weight": 2 }] }), "twice"),
        (json!({ "reserveBp": 1000, "validators": [{ "vote": "nobody", "weight": 1 }] }), "not an address"),
        (json!({ "validators": [{ "vote": a, "weight": 1 }] }), "names it"),
        (json!({ "reserveBp": 1000, "validators": [{ "vote": a, "weight": 1 }, { "vote": "PLACEHOLDER: his", "weight": 1 }] }), "not his yet"),
    ] {
        let e = Rule::of(&with(&rehearsal(), bad.clone()), Some(&own)).unwrap_err();
        assert!(e.contains(says), "{bad}: {e}");
    }
    let two = Rule::of(&with(&rehearsal(), json!({ "reserveBp": 1000, "validators": [{ "vote": a, "weight": 3 }, { "vote": b, "weight": 1 }] })), None).unwrap();
    assert_eq!(two.kept(100 * SOL), 10 * SOL);
    assert_eq!(two.kept(1), 1, "the kept share rounds up: the way out is never short by a lamport");
    assert_eq!((two.target(0, 100 * SOL), two.target(1, 100 * SOL)), (67_500_000_000, 22_500_000_000), "90 SOL placed, three to one");
}

#[test]
fn the_engine_brings_the_pool_to_the_epoch_first_and_never_moves_it_while_rewards_are_paid() {
    let e = k("the engine");
    let rule = Rule { validators: vec![Validator { vote: k("v"), weight: 1 }], reserve_bp: 1000 };
    let mut c = chain_of(&e, 100 * SOL, 100 * SOL + RENT, vec![]);
    c.epoch = 8;
    assert_eq!(engine::next(&rule, &c, &e).unwrap(), Step::Update);
    c.rewards_paying = true;
    assert!(matches!(engine::next(&rule, &c, &e).unwrap(), Step::Wait(_)));
    let not = engine::next(&rule, &c, &k("another key")).unwrap_err();
    assert!(not.contains("ninth"), "a pool whose staker is not this key: nothing, and why: {not}");
    let big = Rule { validators: (0..11).map(|i| Validator { vote: k(&format!("v{i}")), weight: 1 }).collect(), reserve_bp: 0 };
    assert!(engine::next(&big, &chain_of(&e, 0, RENT, vec![]), &e).unwrap_err().contains("10 at most"), "the list's size bounds the rule");
}

#[test]
fn the_rules_validators_are_added_then_the_reserve_placed_by_weight_once_an_epoch() {
    let e = k("the engine");
    let (a, b) = (k("validator a"), k("validator b"));
    let rule = Rule { validators: vec![Validator { vote: a, weight: 3 }, Validator { vote: b, weight: 1 }], reserve_bp: 1000 };
    // 100 SOL staked, nothing placed: both added, each stake account the reserve's rent and a delegation's minimum
    let c = chain_of(&e, 100 * SOL, 100 * SOL + RENT, vec![]);
    assert_eq!(engine::next(&rule, &c, &e).unwrap(), Step::Moves(vec![Move::Add { vote: a }, Move::Add { vote: b }], vec![]));
    // added (each holds rent + 1 SOL, out of the reserve): the 90 SOL placed three to one, each move paying its transient's rent
    let each = RENT + SOL;
    let c = chain_of(&e, 100 * SOL, 100 * SOL + RENT - 2 * each, vec![entry(a, each, 0), entry(b, each, 0)]);
    let Step::Moves(m, _) = engine::next(&rule, &c, &e).unwrap() else { panic!("moves") };
    // the transients' rents come out of what is placed, never out of the kept share: the last validator is short by them
    assert_eq!(m, vec![Move::Increase { vote: a, lamports: 67_500_000_000 - each }, Move::Increase { vote: b, lamports: 22_500_000_000 - each - 2 * RENT }]);
    let placed: u64 = m.iter().map(|x| if let Move::Increase { lamports, .. } = x { lamports + RENT } else { 0 }).sum();
    assert!(c.free() - placed >= rule.kept(c.total), "the reserve keeps its share after the moves");
    // the same epoch again: both transients in flight, nothing is due
    let c2 = chain_of(&e, 100 * SOL, c.reserve - placed, vec![entry(a, each, 67_500_000_000 - each), entry(b, each, 22_500_000_000 - each - 2 * RENT)]);
    assert!(matches!(engine::next(&rule, &c2, &e).unwrap(), Step::Nothing(_)), "idempotent: one transient a validator an epoch");
    // an epoch on, merged and balanced: nothing is due
    let c3 = chain_of(&e, 100 * SOL, 10 * SOL + RENT, vec![entry(a, 67_500_000_000, 0), entry(b, 22_500_000_000, 0)]);
    assert!(matches!(engine::next(&rule, &c3, &e).unwrap(), Step::Nothing(_)));
    // a list that is full: the rule's validator is not added, and that is said
    let mut full = chain_of(&e, 100 * SOL, 100 * SOL + RENT, vec![entry(k("someone else's"), each, 0)]);
    full.max_validators = 1;
    let one = Rule { validators: vec![Validator { vote: a, weight: 1 }], reserve_bp: 0 };
    let Step::Nothing(words) = engine::next(&one, &full, &e).unwrap() else { panic!("nothing") };
    assert!(words.iter().any(|w| w.contains("list is full")) && words.iter().any(|w| w.contains("removal is owed")), "{words:?}");
}

#[test]
fn a_reserve_below_its_share_brings_stake_back_so_the_way_out_stays_open() {
    let e = k("the engine");
    let (a, b) = (k("validator a"), k("validator b"));
    let rule = Rule { validators: vec![Validator { vote: a, weight: 3 }, Validator { vote: b, weight: 1 }], reserve_bp: 1000 };
    // a staker left with 9 of the 10 SOL kept: the pool holds 91, its reserve 1 above its rent, against 9.1 kept
    let c = chain_of(&e, 91 * SOL, SOL + RENT, vec![entry(a, 67_500_000_000, 0), entry(b, 22_500_000_000, 0)]);
    let Step::Moves(m, _) = engine::next(&rule, &c, &e).unwrap() else { panic!("moves") };
    assert_eq!(m, vec![Move::Decrease { vote: a, lamports: 8_100_000_000 }], "the shortfall, from the validator that holds the most");
    // a move in flight there already: the other gives it back
    let c = chain_of(&e, 91 * SOL, SOL + RENT, vec![entry(a, 60 * SOL, 7_500_000_000), entry(b, 22_500_000_000, 0)]);
    let Step::Moves(m, _) = engine::next(&rule, &c, &e).unwrap() else { panic!("moves") };
    assert_eq!(m, vec![Move::Decrease { vote: b, lamports: 8_100_000_000 }]);
    // a staker emptied the reserve to its rent, so it cannot fund the transient's: the engine's key sends that rent in first (found on the
    // rehearsal's validator: the program refuses the decrease otherwise)
    let c = chain_of(&e, 91 * SOL, RENT, vec![entry(a, 70 * SOL, 0)]);
    let Step::Moves(m, words) = engine::next(&rule, &c, &e).unwrap() else { panic!("moves") };
    assert_eq!(m, vec![Move::Fund { lamports: RENT }, Move::Decrease { vote: a, lamports: 9_100_000_000 }]);
    assert!(words.iter().any(|w| w.contains("sends the")), "{words:?}");
    let fund = engine::instruction(&keys(), &e, &m[0], &c.entries).unwrap();
    assert_eq!((fund.program_id, metas(&fund)), (Pubkey::default(), vec![(e, true, true), (keys().reserve, false, true)]), "a transfer from the engine's key into the reserve");
    // nothing above any validator's floor: nothing can come back, and that is said
    let c = chain_of(&e, 91 * SOL, RENT, vec![entry(a, RENT + SOL, 0)]);
    let Step::Nothing(words) = engine::next(&rule, &c, &e).unwrap() else { panic!("nothing") };
    assert!(words.iter().any(|w| w.contains("no validator can give stake back")), "{words:?}");
}

#[test]
fn the_instructions_are_the_stake_pool_programs_own() {
    let kk = keys();
    let (e, v) = (k("the engine"), k("a vote account"));
    let stake = engine::validator_stake(&STAKE_POOL, &v, &kk.pool, 0);
    assert_eq!(stake, Pubkey::find_program_address(&[v.as_ref(), kk.pool.as_ref()], &STAKE_POOL).0, "no seed: [vote, pool]");
    assert_eq!(engine::validator_stake(&STAKE_POOL, &v, &kk.pool, 5), Pubkey::find_program_address(&[v.as_ref(), kk.pool.as_ref(), &5u32.to_le_bytes()], &STAKE_POOL).0);
    let transient = engine::transient_stake(&STAKE_POOL, &v, &kk.pool, 0);
    let add = engine::add(&kk, &e, &v);
    assert_eq!(add.data, vec![1, 0, 0, 0, 0]);
    assert_eq!(metas(&add)[..7], [(kk.pool, false, true), (e, true, false), (kk.reserve, false, true), (kk.withdraw, false, false), (kk.list, false, true), (stake, false, true), (v, false, false)]);
    assert_eq!(add.accounts.len(), 13);
    let en = entry(v, 0, 0);
    let inc = engine::increase(&kk, &e, &en, 3 * SOL);
    assert_eq!(inc.data, [vec![4], (3 * SOL).to_le_bytes().to_vec(), 0u64.to_le_bytes().to_vec()].concat());
    assert_eq!(metas(&inc)[..8], [(kk.pool, false, false), (e, true, false), (kk.withdraw, false, false), (kk.list, false, true), (kk.reserve, false, true), (transient, false, true), (stake, false, false), (v, false, false)]);
    assert_eq!(inc.accounts.len(), 14);
    let dec = engine::decrease(&kk, &e, &en, 2 * SOL);
    assert_eq!(dec.data[0], 21, "DecreaseValidatorStakeWithReserve: the program's version deprecates the plain decrease");
    assert_eq!(metas(&dec)[..7], [(kk.pool, false, false), (e, true, false), (kk.withdraw, false, false), (kk.list, false, true), (kk.reserve, false, true), (stake, false, true), (transient, false, true)]);
    assert_eq!(dec.accounts.len(), 11);
    // the update names every validator's pair, five a transaction, the pool's totals last
    let seven: Vec<Entry> = (0..7).map(|i| entry(k(&format!("v{i}")), 0, 0)).collect();
    let u = engine::update(&kk, &seven);
    assert_eq!(u.len(), 2);
    assert_eq!((u[0].len(), u[1].len()), (1, 2));
    assert_eq!((u[0][0].data.clone(), u[1][0].data.clone()), (vec![6, 0, 0, 0, 0, 0], vec![6, 5, 0, 0, 0, 0]), "the list from 0, then from 5");
    assert_eq!((u[0][0].accounts.len(), u[1][0].accounts.len()), (7 + 10, 7 + 4));
    assert_eq!(u[1][1].data, vec![7], "the pool's totals, the fee minted");
    assert_eq!(engine::update(&kk, &[]).concat(), poleis_door::update(&kk, &[]), "with no validator, the door's own update");
    for tx in &u {
        let (m, _) = wire::message(tx, &e, &[0; 32]);
        assert!(1 + 64 + m.len() <= wire::PACKET, "a transaction of the update holds in a packet");
    }
    let set = engine::set_staker(&kk.pool, &STAKE_POOL, &k("the manager"), &e);
    assert_eq!((set.data.clone(), metas(&set)), (vec![13], vec![(kk.pool, false, true), (k("the manager"), true, false), (e, false, false)]));
}

#[test]
fn the_ninth_act_names_the_engines_key_her_pools_staker_by_the_polis_hand() {
    let (ids, ..) = build::compiled().unwrap();
    let at = chain::addresses(&ids, 77);
    let deployer = k("engine deployer");
    let staker = k("the engine's key");
    let p = with(&mainnet(), json!({ "staker": staker.to_string(), "reserveBp": 1000, "validators": [{ "vote": k("a validator").to_string(), "weight": 1 }] }));
    let later = engine::later(&p, &ids, &at, &deployer).unwrap();
    assert_eq!(later.iter().map(|a| a.n).collect::<Vec<_>>(), [9, 10]);
    let nine = &later[0];
    assert_eq!((nine.n, nine.file, nine.class, nine.impacted.clone(), nine.convene(), nine.nonce, nine.needs, nine.before.len()), (9, "9-the-engine.json", CLASS_ACT, vec![ROOT_UNIT], true, 9, Some(STAKE_POOL), 0), "an ACT_RAW of root 0 after the eight, unit 1 convened by the crank");
    let pool = Pool::of(&p, &deployer, &ids.poleis).unwrap();
    assert_eq!(pool.agent, at.agent);
    let want = Payload::Raw { origin: ROOT_UNIT, program: STAKE_POOL, accounts: vec![(pool.pool, ACT_WRITABLE), (at.agent, ACT_SIGNER), (staker, 0)], data: vec![13] };
    assert_eq!((nine.payload.clone(), nine.payload_sha256), (want.bytes(), want.hash()));
    assert_eq!(nine.hash, client::proposal_hash(CLASS_ACT, ROOT_UNIT, &nine.manifest, &want.hash(), 9));
    // the instruction it carries is the one the live half sends with a stand-in manager in the hand's place
    let inner = engine::set_staker(&pool.pool, &STAKE_POOL, &at.agent, &staker);
    let Payload::Raw { accounts, data, .. } = &want else { unreachable!() };
    assert_eq!(accounts.iter().map(|(a, f)| (*a, f & ACT_SIGNER != 0, f & ACT_WRITABLE != 0)).collect::<Vec<_>>(), metas(&inner));
    assert_eq!(*data, inner.data);
    let e = nine.execute_by(&k("a crank"));
    let n = e.accounts.len();
    assert_eq!(e.accounts[n - 4..].iter().map(|m| (m.pubkey, m.is_signer, m.is_writable)).collect::<Vec<_>>(), vec![(STAKE_POOL, false, false), (pool.pool, false, true), (at.agent, false, false), (staker, false, false)]);
    let (execute, proposed) = (acts::execute_bytes(&p, nine, &k("a crank"), false).unwrap(), poleis_press::floor::proposal_bytes(&p, nine, &poleis_press::floor::placeholder_owner()).unwrap());
    assert!(execute <= wire::PACKET && proposed <= wire::PACKET);
    eprintln!("  act 9: its payload {} bytes, its execute's transaction {execute} bytes, a seat owner's proposal of it {proposed} bytes", nine.payload.len());
    // the eight before it do not move, and a ballot of it is read by its number
    let eight = acts::acts(&p, &ids, &at, &deployer).unwrap();
    assert_eq!(eight.len(), 8);
    assert!(eight.iter().all(|a| a.hash != nine.hash));
    // the rehearsal's params write no ninth act (their key is the engine rehearsal's own), and mainnet's as it stands writes one for reading
    assert!(engine::later(&rehearsal(), &ids, &at, &deployer).unwrap().is_empty());
    assert_eq!(engine::later(&mainnet(), &ids, &at, &deployer).unwrap().iter().map(|a| a.n).collect::<Vec<_>>(), [10], "mainnet as it stands: act 10 stands, act 9 waits on engine.staker");
    // once it executed, her pool is held under the engine's key, the manager still the hand
    assert_eq!(pool.engine, Some(staker));
    assert_eq!(Pool::of(&mainnet(), &deployer, &ids.poleis).unwrap().engine, None, "a placeholder names no staker the pool may stand under");
}

#[test]
fn the_list_is_read_as_the_program_writes_it() {
    let v = k("a vote account");
    let mut d = vec![0u8; 9 + 73 * 3];
    d[0] = 2;
    d[1..5].copy_from_slice(&3u32.to_le_bytes());
    d[5..9].copy_from_slice(&1u32.to_le_bytes());
    let e = &mut d[9..82];
    e[0..8].copy_from_slice(&11u64.to_le_bytes());
    e[8..16].copy_from_slice(&22u64.to_le_bytes());
    e[16..24].copy_from_slice(&33u64.to_le_bytes());
    e[24..32].copy_from_slice(&44u64.to_le_bytes());
    e[36..40].copy_from_slice(&55u32.to_le_bytes());
    e[40] = 1;
    e[41..73].copy_from_slice(v.as_ref());
    let (max, entries) = engine::list_of(&d).unwrap();
    assert_eq!((max, entries), (3, vec![Entry { active: 11, transient: 22, last_update_epoch: 33, transient_seed: 44, validator_seed: 55, status: 1, vote: v }]));
    d[0] = 1;
    assert!(engine::list_of(&d).is_err(), "a pool is not a list");
}

/// THE DOOR BRINGS A POOL WITH VALIDATORS TO THE EPOCH: a stake (and a way out) into a pool behind the epoch names every validator of her
/// list in its update; how many fit the staker's one transaction.
#[test]
fn a_stake_behind_the_epoch_names_the_lists_validators_and_fits_one_transaction_up_to_a_count() {
    let kk = keys();
    let p = Pool::derive(&k("engine base"), "eng1", 10, &STAKE_POOL, &k("poleis")).unwrap();
    let staker = k("a staker");
    let bytes = |n: usize, out: bool| {
        let v: Vec<Entry> = (0..n).map(|i| entry(k(&format!("validator {i}")), SOL, 0)).collect();
        let pairs: Vec<poleis_door::Pair> = v.iter().map(|e| (engine::validator_stake(&STAKE_POOL, &e.vote, &kk.pool, 0), engine::transient_stake(&STAKE_POOL, &e.vote, &kk.pool, 0))).collect();
        let ixs = if out { door::unstake(&p, &staker, SOL, Some(&pairs)) } else { door::stake(&p, &staker, SOL, Some(&pairs)) };
        let (m, _) = wire::message(&ixs, &staker, &[0; 32]);
        1 + 64 + m.len()
    };
    let fits = |out: bool| (0..40).take_while(|n| bytes(*n, out) <= wire::PACKET).last().unwrap();
    let (stake_fits, out_fits) = (fits(false), fits(true));
    eprintln!("  a stake behind the epoch: {} bytes with 1 validator, {} with {stake_fits} (the most that fit); the way out: {} with 1, {out_fits} at most", bytes(1, false), bytes(stake_fits, false), bytes(1, true));
    assert!(bytes(1, false) <= wire::PACKET && bytes(1, true) <= wire::PACKET, "one validator: one transaction");
    assert!(bytes(stake_fits + 1, false) > wire::PACKET);
    // the door's update names each validator's own pair, in the program's order
    let list = {
        let mut d = vec![2u8];
        d.extend_from_slice(&10u32.to_le_bytes());
        d.extend_from_slice(&1u32.to_le_bytes());
        let mut e = [0u8; 73];
        e[41..73].copy_from_slice(k("v").as_ref());
        d.extend_from_slice(&e);
        d
    };
    let pairs = door::pairs(&p, &list).unwrap();
    assert_eq!(pairs, vec![(engine::validator_stake(&STAKE_POOL, &k("v"), &p.pool, 0), engine::transient_stake(&STAKE_POOL, &k("v"), &p.pool, 0))]);
    assert_eq!(engine::list_of(&list).unwrap().1.len(), 1);
}

// ---- the live half ----

fn land(rpc: &Rpc, ixs: &[Instruction], payer: &Key, keys: &[&Key]) -> Result<String, String> {
    let blockhash = rpc.blockhash()?;
    let (w, _) = wire::signed(ixs, &payer.address, &blockhash, keys)?;
    let sig = rpc.send(&w, true)?;
    rpc::wait(Duration::from_secs(60), || match rpc.statuses(&[sig.clone()])?.pop().flatten() {
        Some((_, err, _)) if !err.is_null() => Err(format!("{sig}: {err}")),
        Some((_, _, s)) if s == "confirmed" || s == "finalized" => Ok(Some(())),
        _ => Ok(None),
    })?;
    Ok(sig)
}

fn epoch(rpc: &Rpc) -> u64 {
    rpc.call("getEpochInfo", json!([{ "commitment": "confirmed" }])).unwrap()["epoch"].as_u64().unwrap()
}

fn pool_now(rpc: &Rpc, p: &Pool) -> door::PoolNow {
    door::pool_now(&rpc.account(&p.pool.to_string()).unwrap().expect("the pool's account").data).unwrap()
}

fn tokens(rpc: &Rpc, at: &Pubkey) -> u64 {
    rpc.account(&at.to_string()).unwrap().map(|a| door::amount_of(&a.data).unwrap()).unwrap_or(0)
}

/// An epoch turned and its rewards paid: then the crank, which brings the pool to it.
fn turn(rpc: &Rpc) -> u64 {
    let e = epoch(rpc);
    rpc::wait(Duration::from_secs(120), || Ok((epoch(rpc) > e).then_some(()))).unwrap();
    let paying = || door::rewards_paying(&rpc.account(&door::EPOCH_REWARDS.to_string()).unwrap().map(|a| a.data).unwrap_or_default());
    rpc::wait(Duration::from_secs(120), || Ok((!paying()).then_some(()))).unwrap();
    epoch(rpc)
}

#[test]
fn the_stake_earns_on_a_validator_with_mainnets_program() {
    let Ok(url) = std::env::var("ENGINE_VALIDATOR") else {
        eprintln!("ENGINE_VALIDATOR is not set: the live half runs under poleis/ceremony/the-engine-rehearsal.sh");
        return;
    };
    let begun = std::time::Instant::now();
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };

    let data = rpc.account(&loader::programdata_of(&STAKE_POOL).to_string()).unwrap().expect("the stake pool program's data on this validator");
    let program = Program::of(&STAKE_POOL, &data.data).unwrap();
    if let Ok(want) = std::env::var("ENGINE_PROGRAM_SHA256") {
        assert_eq!(wire::hex(&program.sha256), want, "the stake pool program's bytes on this validator");
    }
    say("the stake pool program", program.json());

    // three stand-ins, their seed the run's own: the pool's manager (the polis's hand's place), a staker, the engine's key
    let seed: [u8; 32] = wire::sha256(format!("the engine's rehearsal {url} {}", std::process::id()).as_bytes());
    let manager = wire::standin(&seed, "the engine's manager");
    let staker = wire::standin(&seed, "the engine's staker");
    let key = wire::standin(&seed, "the engine's key");
    for (k, sol) in [(&manager, 10), (&staker, 300), (&key, 10)] {
        rpc.airdrop(&k.address.to_string(), sol * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || Ok((rpc.balance(&staker.address.to_string())? >= 300 * SOL && rpc.balance(&key.address.to_string())? >= 10 * SOL && rpc.balance(&manager.address.to_string())? >= 10 * SOL).then_some(()))).unwrap();

    // the pool of the launch's shape, born by the manager (door.rs's test does the same): its fee account the manager's receipts account
    let mut p = Pool::derive(&manager.address, "eng1", 10, &STAKE_POOL, &Pubkey::new_from_array([7; 32])).unwrap();
    p.agent = manager.address;
    p.vault = door::receipts_of(&manager.address, &p.mint);
    let mut first = p.reserve_seeded().making(&p.base, rpc.rent(RESERVE_LEN).unwrap(), None).unwrap();
    first.push(p.reserve_init());
    first.extend(p.mint_seeded().making(&p.base, rpc.rent(MINT_LEN).unwrap(), None).unwrap());
    first.push(p.mint_init());
    land(&rpc, &first, &manager, &[&manager]).unwrap();
    land(&rpc, &[door::receipts_made(&manager.address, &manager.address, &p.mint)], &manager, &[&manager]).unwrap();
    let mut birth = Vec::new();
    for s in p.in_the_execute() {
        birth.extend(s.making(&p.base, rpc.rent(s.space).unwrap(), None).unwrap());
    }
    let accounts = p.initialize_accounts().into_iter().map(|(k, f)| AccountMeta { pubkey: k, is_signer: f & ACT_SIGNER != 0, is_writable: f & ACT_WRITABLE != 0 }).collect();
    birth.push(Instruction::new_with_bytes(p.program, &p.initialize_data(), accounts));
    land(&rpc, &birth, &manager, &[&manager]).unwrap();
    say("the pool born", json!({ "pool": p.pool.to_string(), "epoch fee": "1/1", "fee account": p.vault.to_string() }));

    // THE STAKE: 200 SOL in
    let e = epoch(&rpc);
    land(&rpc, &door::stake(&p, &staker.address, 200 * SOL, (pool_now(&rpc, &p).last_update_epoch < e).then_some(&[][..])), &staker, &[&staker]).unwrap();
    let receipts = tokens(&rpc, &door::receipts_of(&staker.address, &p.mint));
    assert_eq!(receipts, 200 * SOL);

    // THE ACT'S INSTRUCTION: SetStaker, the manager signing as the hand does by Poleis once the ninth act is sealed
    let set = engine::set_staker(&p.pool, &STAKE_POOL, &manager.address, &key.address);
    let sig = land(&rpc, &[set], &manager, &[&manager]).unwrap();
    let staker_now = Pubkey::new_from_array(rpc.account(&p.pool.to_string()).unwrap().unwrap().data[engine::STAKER..engine::STAKER + 32].try_into().unwrap());
    assert_eq!(staker_now, key.address, "her pool's staker is the engine's key");
    say("the staker named", json!({ "staker": staker_now.to_string(), "manager": manager.address.to_string(), "signature": sig }));

    // THE RULE: the rehearsal's params, its validator this one
    let rule = engine::rule_on(&rehearsal(), &rpc).unwrap();
    let vote = rule.validators[0].vote;
    let min = engine::stake_minimum(&rpc, &key.address).unwrap();
    say("the rule", json!({ "vote": vote.to_string(), "reserveBp": rule.reserve_bp, "the stake program's minimum delegation": min }));

    // THE CRANK: the validator added, the reserve above its tenth placed on it
    let r1 = engine::run(&rpc, &p.pool, &rule, &key.address, Some(&key)).unwrap();
    say("the crank, first run", json!({ "epoch": r1.epoch, "sent": r1.sent, "fees": r1.fees, "said": r1.said, "after": r1.after }));
    let v = &r1.after["validators"][0];
    assert_eq!(v["vote"], json!(vote.to_string()));
    let placed = v["active"].as_u64().unwrap() + v["transient"].as_u64().unwrap();
    assert!(placed > 170 * SOL, "most of the 200 SOL is on the validator: {placed}");
    assert!(r1.after["reserveFree"].as_u64().unwrap() >= r1.after["kept"].as_u64().unwrap(), "the reserve keeps its share");

    // AGAIN, the same epoch: nothing
    let r2 = engine::run(&rpc, &p.pool, &rule, &key.address, Some(&key)).unwrap();
    assert!(r2.sent.is_empty() && r2.fees == 0, "a second run in the same epoch sends nothing: {:?}", r2.sent);
    say("the crank, again in the same epoch", json!({ "epoch": r2.epoch, "sent": r2.sent.len(), "said": r2.said }));

    // EPOCHS TURN: the transient merges, the stake is active, and its rewards come in as the pool's fee
    let (total0, supply0, vault0) = (pool_now(&rpc, &p).total_lamports, pool_now(&rpc, &p).pool_token_supply, tokens(&rpc, &p.vault));
    let mut turns = Vec::new();
    let mut earned = 0u64;
    for _ in 0..6 {
        let at = turn(&rpc);
        let r = engine::run(&rpc, &p.pool, &rule, &key.address, Some(&key)).unwrap();
        let now = pool_now(&rpc, &p);
        let vault = tokens(&rpc, &p.vault);
        turns.push(json!({ "epoch": at, "total": now.total_lamports, "supply": now.pool_token_supply, "fee account": vault, "fees paid": r.fees, "after": r.after }));
        earned = now.total_lamports - total0;
        if earned > 0 && vault > vault0 && r.after["validators"][0]["transient"].as_u64() == Some(0) {
            break;
        }
    }
    let now = pool_now(&rpc, &p);
    let vault = tokens(&rpc, &p.vault);
    say("epochs turned", json!({ "turns": turns }));
    assert!(earned > 0, "the pool's total grew by a reward");
    assert!(vault > vault0, "the reward was minted to the fee account as the pool's token");
    assert_eq!(now.pool_token_supply - supply0, vault - vault0, "every new token is the fee account's: the staker's receipts did not grow");
    assert_eq!(door::worth(receipts, &now), 200 * SOL, "the epoch fee 1/1: the staker's receipts are worth what was staked, at par");
    say("the stake earned", json!({ "reward (lamports)": earned, "fee minted to the fee account": vault - vault0, "the staker's receipts worth": door::worth(receipts, &now) }));

    // THE DOOR BRINGS A POOL WITH VALIDATORS TO THE EPOCH: an epoch turns and the engine does not run; a new staker's stake lands by the
    // door alone, its update naming the list's validator; the door's old update (naming none) is refused by the program
    let newer = wire::standin(&seed, "a new staker");
    rpc.airdrop(&newer.address.to_string(), 10 * SOL).unwrap();
    rpc::wait(Duration::from_secs(60), || Ok((rpc.balance(&newer.address.to_string())? >= 10 * SOL).then_some(()))).unwrap();
    let at = turn(&rpc);
    assert!(pool_now(&rpc, &p).last_update_epoch < at, "the pool is behind the epoch, the engine not run");
    let pairs = door::pairs(&p, &rpc.account(&p.list.to_string()).unwrap().unwrap().data).unwrap();
    assert_eq!(pairs.len(), 1);
    let (bare, _) = wire::signed(&door::stake(&p, &newer.address, 5 * SOL, Some(&[])), &newer.address, &rpc.blockhash().unwrap(), &[&newer]).unwrap();
    let (refused, _, logs) = rpc.simulate(&bare).unwrap();
    assert!(!refused.is_null(), "an update naming no validator is refused once one stands");
    let words: Vec<String> = logs.as_array().cloned().unwrap_or_default().into_iter().filter_map(|l| l.as_str().map(str::to_string)).filter(|l| l.contains("rror")).collect();
    let ixs = door::stake(&p, &newer.address, 5 * SOL, Some(&pairs));
    let (w, _) = wire::signed(&ixs, &newer.address, &rpc.blockhash().unwrap(), &[&newer]).unwrap();
    let before = rpc.balance(&newer.address.to_string()).unwrap();
    let sig = land(&rpc, &ixs, &newer, &[&newer]).unwrap();
    let (_, fee, units, _) = rpc.transaction(&sig).unwrap().unwrap();
    let got = tokens(&rpc, &door::receipts_of(&newer.address, &p.mint));
    let paid = before - rpc.balance(&newer.address.to_string()).unwrap();
    assert_eq!(pool_now(&rpc, &p).last_update_epoch, at, "the door brought the pool to the epoch");
    assert!(door::worth(got, &pool_now(&rpc, &p)).abs_diff(5 * SOL) <= 1, "receipts worth the 5 SOL staked");
    say("a stake by the door alone, the pool behind the epoch with a validator", json!({ "epoch": at, "the old update refused": refused, "its words": words, "signature": sig, "transaction bytes": w.len(), "instructions": ixs.len(), "fee": fee, "units": units, "receipts": got, "the staker paid": paid }));
    // another epoch, still no engine: the way out by the door alone, at par
    let at = turn(&rpc);
    assert!(pool_now(&rpc, &p).last_update_epoch < at);
    let pairs = door::pairs(&p, &rpc.account(&p.list.to_string()).unwrap().unwrap().data).unwrap();
    let before = rpc.balance(&newer.address.to_string()).unwrap();
    let ixs = door::unstake(&p, &newer.address, got, Some(&pairs));
    let sig = land(&rpc, &ixs, &newer, &[&newer]).unwrap();
    let (_, fee, units, _) = rpc.transaction(&sig).unwrap().unwrap();
    let back = rpc.balance(&newer.address.to_string()).unwrap() + fee - before;
    assert_eq!(tokens(&rpc, &door::receipts_of(&newer.address, &p.mint)), 0);
    assert!(back.abs_diff(5 * SOL) <= 2, "the way out at par: {back} for 5 SOL staked");
    say("the way out by the door alone, the pool behind the epoch with a validator", json!({ "epoch": at, "signature": sig, "instructions": ixs.len(), "fee": fee, "units": units, "receipts burned": got, "lamports back": back }));

    // THE WAY OUT: at once up to the reserve, at par; then the crank brings stake back and the way out opens again
    let free = rpc.account(&p.reserve.to_string()).unwrap().unwrap().lamports - rpc.rent(RESERVE_LEN).unwrap();
    let out = free.min(receipts);
    let before = rpc.balance(&staker.address.to_string()).unwrap();
    let sig = land(&rpc, &door::unstake(&p, &staker.address, out, None), &staker, &[&staker]).unwrap();
    let fee = rpc.transaction(&sig).unwrap().map(|t| t.1).unwrap_or(0);
    let back = rpc.balance(&staker.address.to_string()).unwrap() + fee - before;
    assert_eq!(back, out, "the way out at par: a lamport a receipt");
    let rb = engine::run(&rpc, &p.pool, &rule, &key.address, Some(&key)).unwrap();
    assert!(rb.sent.iter().any(|s| s["did"].as_array().is_some_and(|d| d.iter().any(|x| x.get("decrease").is_some()))), "the crank brings stake back: {:?}", rb.sent);
    turn(&rpc);
    let rm = engine::run(&rpc, &p.pool, &rule, &key.address, Some(&key)).unwrap();
    let free2 = rpc.account(&p.reserve.to_string()).unwrap().unwrap().lamports - rpc.rent(RESERVE_LEN).unwrap();
    assert!(free2 >= rm.after["kept"].as_u64().unwrap().saturating_sub(SOL / 1000), "the reserve holds its share again");
    let left = tokens(&rpc, &door::receipts_of(&staker.address, &p.mint));
    let out2 = free2.min(left);
    let before = rpc.balance(&staker.address.to_string()).unwrap();
    let sig = land(&rpc, &door::unstake(&p, &staker.address, out2, None), &staker, &[&staker]).unwrap();
    let fee = rpc.transaction(&sig).unwrap().map(|t| t.1).unwrap_or(0);
    let back2 = rpc.balance(&staker.address.to_string()).unwrap() + fee - before;
    assert_eq!(back2, out2, "and again at par");
    say("the way out", json!({ "at once": out, "back": back, "the crank brought back": rb.sent, "an epoch on, the reserve above its rent": free2, "then out": out2, "back again": back2, "receipts left": left - out2 }));

    // the engine's own fees and what the run took
    say("the run", json!({ "wall (s)": begun.elapsed().as_secs(), "engine key's SOL spent": 10 * SOL - rpc.balance(&key.address.to_string()).unwrap() }));
    if let Ok(path) = std::env::var("ENGINE_CERTIFICATE") {
        if !path.is_empty() {
            let c = json!({ "schema": "embertide.engine-rehearsal/1", "validator": "solana-test-validator, local, 32 slots an epoch", "said": said });
            std::fs::write(&path, serde_json::to_string_pretty(&c).unwrap()).unwrap();
        }
    }
    let _ = STAKE;
}
