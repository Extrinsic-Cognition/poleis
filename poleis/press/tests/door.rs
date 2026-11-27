//! THE STAKE GOES INTO HER, proven (Nov 22 2026, lane/the-stake-goes-into-her; his "Yes, build it" to "Build the stake door and the choice,
//! proven on the practice chain?"). Two halves:
//! - the door's instructions as the stake pool program reads them (their bytes and their account orders), and the light read back from a
//!   memo: pure, at every gate;
//! - the door on a validator holding mainnet's stake pool program (DOOR_VALIDATOR, which poleis/ceremony/the-door-rehearsal.sh sets): a pool
//!   of the launch's shape born by a stand-in manager, a stand-in staker's stake, its light chosen and changed, an epoch turned and the pool
//!   brought to it by the door, and the way out at par. Without DOOR_VALIDATOR the live half says so and passes.

use poleis_client::{ACT_SIGNER, ACT_WRITABLE};
use poleis_press::door::{self, Light};
use poleis_press::loader;
use poleis_press::pool::{Pool, Program, MINT_LEN, RESERVE_LEN, STAKE_POOL};
use poleis_press::rpc::{self, Cluster, Rpc};
use poleis_press::wire::{self, Key};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::time::Duration;

const SOL: u64 = 1_000_000_000;

/// A pool of the launch's shape (its epoch fee 1/1, no deposit authority), its base and manager one key: the launch's manager is the polis's
/// hand and its fee account root 0's vault of the mint; here both are the stand-in manager's.
fn pool_for(manager: &Pubkey) -> Pool {
    let mut p = Pool::derive(manager, "door1", 10, &STAKE_POOL, &Pubkey::new_from_array([7; 32])).unwrap();
    p.agent = *manager;
    p.vault = door::receipts_of(manager, &p.mint);
    p
}

fn metas(ix: &Instruction) -> Vec<(Pubkey, bool, bool)> {
    ix.accounts.iter().map(|a| (a.pubkey, a.is_signer, a.is_writable)).collect()
}

#[test]
fn the_stake_is_a_deposit_of_sol_the_staker_signs() {
    let (manager, staker) = (Pubkey::new_from_array([1; 32]), Pubkey::new_from_array([2; 32]));
    let p = pool_for(&manager);
    let ixs = door::stake(&p, &staker, 25 * SOL, None);
    assert_eq!(ixs.len(), 2, "the receipts account, then the deposit");
    let receipts = door::receipts_of(&staker, &p.mint);
    assert_eq!(ixs[0].program_id, door::ATA);
    assert_eq!(ixs[0].data, vec![door::ATA_IDEMPOTENT]);
    assert_eq!(metas(&ixs[0])[..2], [(staker, true, true), (receipts, false, true)]);
    let d = &ixs[1];
    assert_eq!(d.program_id, STAKE_POOL);
    assert_eq!(d.data.len(), 9);
    assert_eq!(d.data[0], door::DEPOSIT_SOL);
    assert_eq!(u64::from_le_bytes(d.data[1..9].try_into().unwrap()), 25 * SOL);
    assert_eq!(
        metas(d)[..8],
        [
            (p.pool, false, true),
            (p.withdraw, false, false),
            (p.reserve, false, true),
            (staker, true, true),
            (receipts, false, true),
            (p.vault, false, true),
            (receipts, false, true),
            (p.mint, false, true),
        ]
    );
    let behind = door::stake(&p, &staker, SOL, Some(&[]));
    assert_eq!(behind.len(), 4, "a pool behind the epoch is brought to it first");
    assert_eq!(behind[0].data, vec![door::UPDATE_LIST, 0, 0, 0, 0, 0]);
    assert_eq!(behind[1].data, vec![door::UPDATE_POOL]);
    assert_eq!(metas(&behind[1])[4], (p.vault, false, true), "the fee account takes the epoch's yield");
}

#[test]
fn the_way_out_burns_the_receipts_and_sends_the_sol_back() {
    let (manager, staker) = (Pubkey::new_from_array([1; 32]), Pubkey::new_from_array([2; 32]));
    let p = pool_for(&manager);
    let ixs = door::unstake(&p, &staker, 3 * SOL, None);
    assert_eq!(ixs.len(), 1);
    let w = &ixs[0];
    assert_eq!(w.data[0], door::WITHDRAW_SOL);
    assert_eq!(u64::from_le_bytes(w.data[1..9].try_into().unwrap()), 3 * SOL);
    let m = metas(w);
    assert_eq!(m.len(), 12);
    assert_eq!(m[2], (staker, true, false), "the staker signs for its receipts");
    assert_eq!(m[3], (door::receipts_of(&staker, &p.mint), false, true));
    assert_eq!(m[4], (p.reserve, false, true));
    assert_eq!(m[5], (staker, false, true), "the SOL comes back to the staker");
    assert_eq!(door::unstake(&p, &staker, SOL, Some(&[])).len(), 3);
}

#[test]
fn the_light_is_a_memo_and_reads_back() {
    let staker = Pubkey::new_from_array([2; 32]);
    assert_eq!(Light::Wish(5).memo(), "embertide.light/1 wish 5");
    assert_eq!(Light::Futarchy.memo(), "embertide.light/1 futarchy");
    assert_eq!(door::light_of("embertide.light/1 wish 5"), Some(Light::Wish(5)));
    assert_eq!(door::light_of("embertide.light/1 wish 5025136712169992152"), Some(Light::Wish(5025136712169992152)));
    assert_eq!(door::light_of("embertide.light/1 futarchy"), Some(Light::Futarchy));
    for no in ["embertide.light/1 wish", "embertide.light/1 wish five", "embertide.light/1 wish 5 6", "embertide.light/2 futarchy", "embertide.light/1futarchy", "gm"] {
        assert_eq!(door::light_of(no), None, "{no}");
    }
    let ix = door::light(&staker, Light::Futarchy);
    assert_eq!(ix.program_id, door::MEMO);
    assert_eq!(metas(&ix), vec![(staker, true, false)]);
    assert_eq!(ix.data, b"embertide.light/1 futarchy".to_vec());
    let logs = ["Program MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr invoke [1]".to_string(), "Program log: Memo (len 24): \"embertide.light/1 wish 5\"".to_string()];
    assert_eq!(door::memo_in_logs(&logs).as_deref(), Some("embertide.light/1 wish 5"));
}

#[test]
fn the_door_waits_while_the_epochs_rewards_are_paid() {
    let mut sysvar = vec![0u8; 81];
    assert!(!door::rewards_paying(&sysvar));
    sysvar[door::REWARDS_ACTIVE] = 1;
    assert!(door::rewards_paying(&sysvar));
    assert!(!door::rewards_paying(&sysvar[..80]), "no flag read, no wait");
}

#[test]
fn the_pool_is_read_at_its_head() {
    let mut data = vec![0u8; 611];
    data[0] = 1;
    data[door::TOTAL_LAMPORTS..door::TOTAL_LAMPORTS + 8].copy_from_slice(&30u64.to_le_bytes());
    data[door::POOL_TOKEN_SUPPLY..door::POOL_TOKEN_SUPPLY + 8].copy_from_slice(&20u64.to_le_bytes());
    data[door::LAST_UPDATE_EPOCH..door::LAST_UPDATE_EPOCH + 8].copy_from_slice(&9u64.to_le_bytes());
    let now = door::pool_now(&data).unwrap();
    assert_eq!((now.total_lamports, now.pool_token_supply, now.last_update_epoch), (30, 20, 9));
    assert_eq!(door::worth(10, &now), 15);
    assert_eq!(door::worth(10, &door::PoolNow { total_lamports: 0, pool_token_supply: 0, last_update_epoch: 0 }), 10);
    assert!(door::pool_now(&data[..100]).is_err());
    data[0] = 2;
    assert!(door::pool_now(&data).is_err(), "a validator list is not a pool");
}

// ---- the live half ----

/// One transaction landed: signed by its keys, sent with its preflight, confirmed. Its signature, or the chain's words.
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

fn receipts(rpc: &Rpc, owner: &Pubkey, p: &Pool) -> u64 {
    rpc.account(&door::receipts_of(owner, &p.mint).to_string()).unwrap().map(|a| door::amount_of(&a.data).unwrap()).unwrap_or(0)
}

/// The light a staker chose last: the newest memo of the light among its signatures.
fn light_now(rpc: &Rpc, staker: &Pubkey) -> Option<Light> {
    for (sig, _, ok) in rpc.signatures_for(&staker.to_string(), 20).unwrap() {
        if !ok {
            continue;
        }
        let Some((_, _, _, logs)) = rpc.transaction(&sig).unwrap() else { continue };
        let logs: Vec<String> = logs.as_array().cloned().unwrap_or_default().into_iter().filter_map(|l| l.as_str().map(str::to_string)).collect();
        if let Some(choice) = door::memo_in_logs(&logs).as_deref().and_then(door::light_of) {
            return Some(choice);
        }
    }
    None
}

#[test]
fn the_door_on_a_validator_with_mainnets_program() {
    let Ok(url) = std::env::var("DOOR_VALIDATOR") else {
        eprintln!("DOOR_VALIDATOR is not set: the live half runs under poleis/ceremony/the-door-rehearsal.sh");
        return;
    };
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };

    // the program on the chain is mainnet's bytes: read from its program data's account as the launch reads them (pool.rs Program::of: the
    // executable bytes after the header, trailing zeros trimmed), held to the fingerprint the rehearsal names
    let data = rpc.account(&loader::programdata_of(&STAKE_POOL).to_string()).unwrap().expect("the stake pool program's data on this validator");
    let program = Program::of(&STAKE_POOL, &data.data).unwrap();
    let sha = wire::hex(&program.sha256);
    if let Ok(want) = std::env::var("DOOR_PROGRAM_SHA256") {
        assert_eq!(sha, want, "the stake pool program's bytes on this validator");
    }
    say("the stake pool program", program.json());

    // two stand-ins, their seed the run's own
    let seed: [u8; 32] = wire::sha256(format!("the door's rehearsal {url} {}", std::process::id()).as_bytes());
    let manager = wire::standin(&seed, "the door's manager");
    let staker = wire::standin(&seed, "the door's staker");
    for k in [&manager, &staker] {
        rpc.airdrop(&k.address.to_string(), 100 * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || Ok((rpc.balance(&staker.address.to_string())? >= 100 * SOL && rpc.balance(&manager.address.to_string())? >= 100 * SOL).then_some(()))).unwrap();

    // the pool born by the manager: its reserve and its mint, the fee account, then its account, its list and its Initialize
    let p = pool_for(&manager.address);
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
    let born = land(&rpc, &birth, &manager, &[&manager]).unwrap();
    let at_birth = pool_now(&rpc, &p);
    say("the pool born", json!({ "pool": p.pool.to_string(), "mint": p.mint.to_string(), "reserve": p.reserve.to_string(), "signature": born, "epoch": at_birth.last_update_epoch }));

    // THE STAKE: 25 SOL into the pool, 25 SOL of receipts back
    let before = rpc.balance(&staker.address.to_string()).unwrap();
    let e = epoch(&rpc);
    let sig = land(&rpc, &door::stake(&p, &staker.address, 25 * SOL, (pool_now(&rpc, &p).last_update_epoch < e).then_some(&[][..])), &staker, &[&staker]).unwrap();
    let got = receipts(&rpc, &staker.address, &p);
    let after = rpc.balance(&staker.address.to_string()).unwrap();
    assert_eq!(got, 25 * SOL, "one receipt for each lamport at the first stake");
    let now = pool_now(&rpc, &p);
    assert_eq!(door::worth(got, &now), 25 * SOL);
    say("the stake", json!({ "lamports": 25 * SOL, "receipts": got, "worth": door::worth(got, &now), "the staker paid": before - after, "signature": sig }));

    // THE LIGHT: a wish, then changed to the futarchy; the newest memo is the staker's choice
    let wish = land(&rpc, &[door::light(&staker.address, Light::Wish(5))], &staker, &[&staker]).unwrap();
    assert_eq!(light_now(&rpc, &staker.address), Some(Light::Wish(5)));
    let fut = land(&rpc, &[door::light(&staker.address, Light::Futarchy)], &staker, &[&staker]).unwrap();
    assert_eq!(light_now(&rpc, &staker.address), Some(Light::Futarchy));
    say("the light", json!({ "first": Light::Wish(5).memo(), "first_signature": wish, "then": Light::Futarchy.memo(), "then_signature": fut, "read_back": format!("{:?}", light_now(&rpc, &staker.address)) }));

    // AN EPOCH TURNS: the pool is behind; while the epoch's rewards are paid nothing moves, then a stake without the door's update is refused
    // and the door's own lands
    let born_epoch = pool_now(&rpc, &p).last_update_epoch;
    rpc::wait(Duration::from_secs(120), || Ok((epoch(&rpc) > born_epoch).then_some(()))).unwrap();
    let turned = std::time::Instant::now();
    let paying = || door::rewards_paying(&rpc.account(&door::EPOCH_REWARDS.to_string()).unwrap().map(|a| a.data).unwrap_or_default());
    let paid_while = paying();
    rpc::wait(Duration::from_secs(120), || Ok((!paying()).then_some(()))).unwrap();
    say("the epoch's rewards", json!({ "paying when the epoch turned": paid_while, "waited (ms)": turned.elapsed().as_millis() as u64 }));
    let blockhash = rpc.blockhash().unwrap();
    let (bare, _) = wire::signed(&door::stake(&p, &staker.address, SOL, None), &staker.address, &blockhash, &[&staker]).unwrap();
    let (err, _, logs) = rpc.simulate(&bare).unwrap();
    assert!(!err.is_null(), "a stake into a pool behind the epoch is refused by the program");
    let words: Vec<String> = logs.as_array().cloned().unwrap_or_default().into_iter().filter_map(|l| l.as_str().map(str::to_string)).filter(|l| l.contains("rror")).collect();
    let sig = land(&rpc, &door::stake(&p, &staker.address, SOL, Some(&[])), &staker, &[&staker]).unwrap();
    let now = pool_now(&rpc, &p);
    assert!(now.last_update_epoch > born_epoch, "the pool brought to the epoch");
    let held = receipts(&rpc, &staker.address, &p);
    assert_eq!(held, 26 * SOL, "no yield without a validator: the rate stands");
    say("an epoch turned", json!({ "pool's epoch before": born_epoch, "pool's epoch after": now.last_update_epoch, "refused without the update": err, "its words": words, "with the door's update": sig, "receipts": held }));

    // THE WAY OUT: every receipt burned, every lamport back from the reserve
    let before = rpc.balance(&staker.address.to_string()).unwrap();
    let back = door::worth(held, &pool_now(&rpc, &p));
    let behind = pool_now(&rpc, &p).last_update_epoch < epoch(&rpc);
    let sig = land(&rpc, &door::unstake(&p, &staker.address, held, behind.then_some(&[][..])), &staker, &[&staker]).unwrap();
    let after = rpc.balance(&staker.address.to_string()).unwrap();
    let fee = rpc.transaction(&sig).unwrap().map(|t| t.1).unwrap_or(0);
    assert_eq!(receipts(&rpc, &staker.address, &p), 0);
    assert_eq!(after + fee - before, back, "the same SOL back, at par");
    assert_eq!(back, 26 * SOL);
    let end = pool_now(&rpc, &p);
    assert_eq!((end.total_lamports, end.pool_token_supply), (0, 0), "the pool holds nobody's stake after the way out");
    say("the way out", json!({ "receipts burned": held, "lamports back": after + fee - before, "fee": fee, "signature": sig, "the pool after": { "total_lamports": end.total_lamports, "supply": end.pool_token_supply } }));

    if let Ok(path) = std::env::var("DOOR_CERTIFICATE") {
        if !path.is_empty() {
            let c = json!({ "schema": "embertide.door-rehearsal/1", "validator": "solana-test-validator, local, 32 slots an epoch", "said": said });
            std::fs::write(&path, serde_json::to_string_pretty(&c).unwrap()).unwrap();
        }
    }
}
