//! practice · A RELEASE IS A VOTE, on the practice chain: a validator of this machine holding devnet's own Poleis bytes (the way of lane
//! a-rider-steers, landed fb1d6ac473), nothing past 127.0.0.1. Stand-in keys only, made here and kept in the practice's folder; none of his.
//!
//!   cargo run --example practice -- found  --rpc http://127.0.0.1:P --dir D     a polis of eleven founders in its chairs and two riders on its floor,
//!                                                                              founded, sold and ratified as poleis-journey's localnet e2e does it;
//!                                                                              writes D/road.json (its node, its program, its polis: nobody named who proposes)
//!   cargo run --example practice -- decide <commit> --rpc .. --dir D [--by his|rider] [--no]
//!                                                                              a release proposal of <commit>, both chambers' votes (eight chairs of
//!                                                                              eleven and the whole floor, PRO, or CONTRA with --no), and its seal

use poleis_client as p;
use poleis_press::rpc::{wait, Cluster, Rpc};
use poleis_press::wire::{self, Key};
use poleis_seal::{manifest_of, payload_of, proposal_of, NONCES, RELEASE_CLASS};
use poleis_treasury as pt;
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::time::Duration;

type Res<T> = Result<T, String>;
const SOL: u64 = 1_000_000_000;
const NAMES: [&str; 15] = ["payer", "crank", "rider", "other", "f0", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10"];

fn send(rpc: &Rpc, what: &str, ixs: Vec<Instruction>, keys: &[&Key]) -> Res<String> {
    let (w, first) = wire::signed(&ixs, &keys[0].address, &rpc.blockhash()?, keys)?;
    let sig = rpc.send(&w, true).map_err(|e| format!("{what}: {e}"))?;
    if sig != wire::base58(&first) {
        return Err(format!("{what}: another signature came back"));
    }
    wait(Duration::from_secs(60), || match rpc.statuses(&[sig.clone()])?.into_iter().next().flatten() {
        Some((_, e, c)) if e.is_null() && (c == "confirmed" || c == "finalized") => Ok(Some(())),
        Some((_, e, _)) if !e.is_null() => Err(format!("{what}: landed and failed: {e}")),
        _ => Ok(None),
    })?;
    println!("  {what:<52} {}…", &sig[..16]);
    Ok(sig)
}
fn until_slot(rpc: &Rpc, s: u64) -> Res<()> {
    wait(Duration::from_secs(120), || Ok((rpc.slot()? >= s).then_some(())))
}
fn keys(dir: &str) -> Res<Vec<Key>> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/keys.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    NAMES.iter().map(|n| Key::from_cli_text(&v[*n].to_string())).collect()
}
fn salt(n: u64, price: u64) -> [u8; 32] {
    let mut s = [0u8; 32];
    s[0] = n as u8 + 1;
    s[1..9].copy_from_slice(&price.to_le_bytes());
    s
}

fn found(rpc: &Rpc, dir: &str) -> Res<()> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let ks: Vec<Key> = NAMES.iter().map(|_| Key::fresh()).collect::<Res<_>>()?;
    std::fs::write(format!("{dir}/keys.json"), json!(NAMES.iter().zip(&ks).map(|(n, k)| (n.to_string(), serde_json::from_str::<Value>(&k.cli_text()).unwrap())).collect::<serde_json::Map<_, _>>()).to_string())
        .map_err(|e| e.to_string())?;
    let [payer, crank, rider, other, founders @ ..] = &ks[..] else { unreachable!() };
    println!("practice chain {}, program {}, genesis {}", rpc.url, p::PROGRAM_ID, rpc.call("getGenesisHash", json!([]))?);
    for (k, l) in [(payer, 20 * SOL), (crank, 5 * SOL), (&founders[0], SOL), (rider, SOL), (other, SOL)] {
        let s = rpc.airdrop(&k.address.to_string(), l)?;
        wait(Duration::from_secs(60), || Ok(rpc.statuses(&[s.clone()])?.into_iter().next().flatten().map(|_| ())))?;
    }
    let mint = Key::fresh()?;
    let mut init = vec![0u8; 35];
    init[0] = 20; // SPL Token InitializeMint2: no decimals, the payer its authority, no freeze authority
    init[2..34].copy_from_slice(payer.address.as_ref());
    send(rpc, "the default mint", vec![
        poleis_press::loader::create_account(&payer.address, &mint.address, rpc.rent(82)?, 82, &p::TOKEN_PROGRAM_ID),
        Instruction::new_with_bytes(p::TOKEN_PROGRAM_ID, &init, vec![AccountMeta::new(mint.address, false)]),
    ], &[payer, &mint])?;
    send(rpc, "init_config", vec![p::init_config(payer.address, mint.address)], &[payer])?;
    let s0 = rpc.slot()?;
    let g = p::GenesisSpec {
        charter: poleis_seal::sha256(b"THE PRACTICE HOUSE"), plan_cap: 10 * SOL, plan_period: 1_000_000, window: 150, rung: 1, reserve: 1_000_000, deposit: 5_000_000,
        commit_end: s0 + 30, reveal_end: s0 + 60, founders: founders.iter().map(|k| k.address).collect(), name: b"THE PRACTICE HOUSE".to_vec(), desc: b"a polis on a local chain, for a release to be sealed".to_vec(),
    };
    send(rpc, "open_genesis", vec![p::open_genesis(payer.address, &g)], &[payer])?;
    let bids = [(rider, 0u64, 2_000_000u64), (other, 0, 1_500_000), (other, 1, 1_500_000)];
    for (k, n, price) in bids {
        send(rpc, &format!("commit_seat_bid ({n})"), vec![p::commit_seat_bid(k.address, 0, 0, n, p::seat_commitment(price, n, &salt(n, price), &k.address))], &[k])?;
    }
    until_slot(rpc, s0 + 31)?;
    for (k, n, price) in bids {
        send(rpc, &format!("reveal_seat_bid ({n})"), vec![p::reveal_seat_bid(k.address, 0, 0, n, price, salt(n, price))], &[k])?;
    }
    until_slot(rpc, s0 + 61)?;
    send(rpc, "settle_seats", vec![p::settle_seats(crank.address, 0, 0)], &[crank])?;
    for (k, n, _) in bids {
        send(rpc, &format!("claim_seat ({n})"), vec![p::claim_seat(k.address, 0, 0, n, 0)], &[k])?;
    }
    send(rpc, "ratify_genesis", vec![p::ratify_genesis(crank.address)], &[crank])?;
    let road = json!({ "rpc": rpc.url, "program": p::PROGRAM_ID.to_string(), "polis": 0 });
    std::fs::write(format!("{dir}/road.json"), road.to_string()).map_err(|e| e.to_string())?;
    println!("founded: polis 0, 11 chairs, 3 seats on its floor; the road {dir}/road.json names {} (the stand-in for his key) as its one proposer", founders[0].address);
    Ok(())
}

fn decide(rpc: &Rpc, dir: &str, commit: &str, by: &str, choice: u8) -> Res<()> {
    let ks = keys(dir)?;
    let [_, crank, rider, other, founders @ ..] = &ks[..] else { unreachable!() };
    let proposer = if by == "rider" { rider } else { &founders[0] };
    let c = poleis_seal::commit_bytes(commit)?;
    let n = (0..NONCES).find(|&n| rpc.account(&p::proposal_pda(&proposal_of(&c, 0, n)).to_string()).ok().flatten().is_none()).ok_or("every nonce is taken")?;
    let h = proposal_of(&c, 0, n);
    if by == "rider" {
        let s = rpc.airdrop(&rider.address.to_string(), SOL)?; // the proposal's rent
        wait(Duration::from_secs(60), || Ok(rpc.statuses(&[s.clone()])?.into_iter().next().flatten().map(|_| ())))?;
    }
    send(rpc, &format!("propose {} (nonce {n}, by the {by})", &commit[..12]), vec![p::propose_raw(proposer.address, h, RELEASE_CLASS, 0, manifest_of(&c), &payload_of(&c), &[0])], &[proposer])?;
    let voters: Vec<&Key> = founders[..8].iter().chain([rider, other]).collect();
    let signed: Vec<(Pubkey, u8, [u8; 64])> = voters.iter().map(|k| (k.address, choice, k.sign(&p::vote_envelope(&h, 0, choice)))).collect();
    for (i, b) in signed.chunks(4).enumerate() {
        send(rpc, &format!("attest ({} votes, batch {})", b.len(), i + 1), p::attest_batch(crank.address, h, 0, b).to_vec(), &[crank])?;
    }
    let tally = || -> Res<p::SessionView> { p::SessionView::parse(&rpc.account(&p::session_pda(&h, 0).to_string())?.ok_or("no session")?.data).ok_or("no session reads".into()) };
    let t = tally()?;
    println!("  the tally: pentagons {} PRO {} CONTRA, hexagons {} PRO {} CONTRA, {} votes; the window to slot {}", t.pent_pro, t.pent_contra, t.hex_pro, t.hex_contra, t.votes, t.deadline);
    if let Err(e) = send(rpc, "seal", vec![p::seal(crank.address, h, 0)], &[crank]) {
        // the fast path seals early only once no vote still to come can change the outcome; else the seal waits for the window's end
        println!("  the seal before the window's end: {}", e.split("\"logs\"").next().unwrap_or(&e).chars().take(120).collect::<String>());
        until_slot(rpc, t.deadline + 1)?;
        send(rpc, "seal (the window closed)", vec![p::seal(crank.address, h, 0)], &[crank])?;
    }
    let s =p::SessionView::parse(&rpc.account(&p::session_pda(&h, 0).to_string())?.ok_or("no session")?.data).ok_or("no session reads")?;
    println!("sealed: state {} ({}), pentagons {} PRO, hexagons {} PRO of {}", s.state, if s.state == pt::STATE_PASSED { "RATUM" } else { "not RATUM" }, s.pent_pro, s.hex_pro, s.active_weight);
    Ok(())
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let o = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let r = (|| -> Res<()> {
        let rpc = Rpc::new(&o("--rpc").ok_or("no --rpc")?, Cluster::Localnet)?;
        let dir = o("--dir").ok_or("no --dir")?;
        match a.first().map(String::as_str) {
            Some("found") => found(&rpc, &dir),
            Some("decide") => decide(&rpc, &dir, a.get(1).ok_or("no commit")?, &o("--by").unwrap_or("his".into()), if a.iter().any(|x| x == "--no") { pt::VOTE_CONTRA } else { pt::VOTE_PRO }),
            _ => Err("found | decide <commit>".into()),
        }
    })();
    if let Err(e) = r {
        eprintln!("practice: {e}");
        std::process::exit(1);
    }
}
