//! war-rehearsal: THE FIRST REFERENCE WAR ON A REAL CHAIN (Nov 24 2026): a local validator (lane/the-war-is-its-own-program) or Solana's
//! devnet (lane/the-war-on-devnet; his "Yes, devnet", Nov 24 2026 12:24:57 UTC, to "Put the war on devnet next, Solana's free test chain, so
//! screens and phones can read a live war?", and his "Yes, use it", 12:30 UTC, to "Use your devnet wallet's free test SOL for the deploy and
//! one live war (up to 40, rest sent back)?").
//!
//! usage, local:  war-rehearsal --rpc http://127.0.0.1:<port> --program <poleis_wars.so> --reference <reference-in-sol.json> [--receipt <out.json>]
//! usage, devnet: war-rehearsal --cluster devnet --rpc https://api.devnet.solana.com --payer <keypair.json> --program <poleis_wars.so>
//!                --reference <reference-in-sol.json> [--minute-seconds 10] [--seed-file <path>] [--receipt <out.json>]
//!                war-rehearsal --cluster devnet ... --refund     (every stand-in of the seed file sends its whole balance back to the payer)
//!                war-rehearsal --cluster devnet ... --finish --treasury <address>   (a war a run left standing mid-raid, finished by the clock:
//!                the raid's end waited out, a crank until it is decided, the settlement, every convert and unlock, the dump tax waited out,
//!                every holder's sale, each creator and the publisher paid, every stand-in's balance back to the payer; read off the chain alone)
//!                war-rehearsal --plan --reference <...>          (no chain: the run's SOL, by the law, before it is run)
//!
//! On a local validator (the press's RPC refuses anything but 127.0.0.1 and [::1], and a public cluster's genesis), with poleis_wars loaded
//! at genesis: stand-in keys made in memory from a seed this process draws and holds (never a file), funded by the validator's faucet.
//! On devnet (the copied RPC's Devnet cluster: https://api.devnet.solana.com and nothing else, its genesis pinned, every call paced under the
//! public endpoint's limits), with poleis_wars deployed at its devnet id by its upgrade authority (never by this binary): the stand-in keys
//! made from a seed this run writes to its seed file first (mode 600, never in git, never printed), so a run that dies is refunded by
//! --refund; funded by transfers from the payer's own keypair file (devnet's faucet is closed); every transaction held to its confirmation.
//!
//! On both: the config at `minute_seconds` (4 locally, 10 on devnet by default) and no shield; the two houses; the forty buys in the model's
//! order, four a transaction; the stakes, crow19's alone, and crow18's, which opens the raid, in one transaction with minute 0's crank; then
//! for each minute m the chain's own clock read (the Clock sysvar) until the raid's minute m has begun, and ONE transaction holding that
//! minute's events and its crank, so they land in that minute in order: its crank must have read minute m, else the run fails and says
//! which. After the decision: the settlement, every convert of the loser's holders, the unlock of the winner's stakers; then every account
//! read back and held to the owner's reference to the unit (the winner, how and the minute, every settled integer, every fighter's holds,
//! backed and took) and to the book, and beside it the law's whole war run step by step. THE GAME IS ONE SWARM (lane/the-game-pays-its-swarm):
//! the war is one generation of its rules, founded by the game swarm's seal (a stand-in key here, signing only) and naming the publisher's
//! treasury: on a local validator the game swarm's own Poleis treasury, poleis_treasury loaded at genesis beside the war (--poleis <its
//! id>), its tree and its unit founded first; each house is launched by a creator of its own; at the end every house's creator is paid what
//! it owes him and the publisher everything the fee account holds, each held to the law's fees, and Poleis's own sync_endowment books the
//! publisher's share. On devnet then: the winner's dump tax waited out
//! (its `dump_end`, on the chain's clock), every fighter's coins sold, every stand-in's whole balance sent back to the payer, and the SOL in,
//! back and spent for good, each by its cause; the war's accounts stay standing, named in the receipt with their explorer links.
//! The receipt (JSON, on stdout, and at --receipt): the cluster's version, the program's sha256, the wall time, each instruction kind's
//! compute units as the chain reported them, the minutes, and any refusal. Exit 0 when everything held, 1 when anything did not (its words
//! in the receipt).

use poleis_war::law::{backed_of, sell_on, Phase, Rules, Side, SOL as LAW_SOL};
use poleis_war::war::{how_text, the_four, Event, Kind, War, HOUSE, N};
use poleis_wars::error::WarsError;
use poleis_wars::state::{Config, Fees, House, Position, Raid, HOUSE_ACTIVE, HOUSE_REKT, RAID_ACTIVE, RAID_PROPOSED, RAID_SETTLED, RAID_STAND};
use poleis_wars_rehearsal::rpc::{Cluster, Rpc};
use poleis_wars_rehearsal::war::*;
use poleis_wars_rehearsal::wire::{self, Key};
use poleis_wars_rehearsal::Res;
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// the raid's minute on a local validator, in seconds: its clock cannot be warped
// 4 s a minute: at 2 the second run (11:50 UTC Nov 24 2026) missed minute 68, its transaction landing in minute 69 of the chain's clock (whole
// seconds, corrected toward its votes' time, on a Mac other seats were loading); the first run's 150 minutes at 2 had all landed
const MINUTE_SECONDS_LOCAL: u32 = 4;
/// the raid's minute on devnet, in seconds, unless --minute-seconds says otherwise: its blocks come from leaders across the world, a public
/// endpoint relays every call, and the first reference war's 150 minutes take 25 of them
const MINUTE_SECONDS_DEVNET: u32 = 10;
const CLOCK: &str = "SysvarC1ock11111111111111111111111111111111";
const LAMPORTS: u64 = 1_000_000_000;
const R: Rules = Rules::MEME_ROYALE;
/// what each stand-in is given beyond what the war takes from it, so no fee or rent of the chain's leaves one short; it all comes back
const MARGIN: u64 = 1_000_000;
const SYSTEM: Pubkey = Pubkey::new_from_array([0u8; 32]);

struct Args {
    cluster: Cluster,
    rpc: Option<String>,
    program: Option<String>,
    reference: String,
    receipt: Option<String>,
    payer: Option<String>,
    minute_seconds: Option<u32>,
    seed_file: Option<String>,
    refund: bool,
    /// devnet: a war an earlier run left standing mid-raid (its seed still in the seed file) is finished by the clock
    finish: bool,
    plan: bool,
    /// devnet, after the run: what the payer holds beyond `keep` goes back to this address, in one transfer the payer signs
    return_to: Option<String>,
    keep: u64,
    /// a local validator: Poleis's program id, loaded at genesis, under which the game swarm's treasury is founded
    poleis: Option<String>,
    /// devnet: the publisher's treasury the generation names (the game swarm's Poleis treasury); a stand-in of the seed when none
    treasury: Option<String>,
}
fn args() -> Res<Args> {
    let usage = "usage: war-rehearsal [--cluster localnet|devnet] --rpc <url> --program <.so> --reference <.json> [--receipt <out.json>] [--payer <keypair.json>] [--minute-seconds n] [--seed-file <path>] [--refund] [--plan] [--return-to <address> [--keep <lamports>]] [--poleis <id>] [--treasury <address>]";
    let mut a = std::env::args().skip(1);
    let mut out = Args {
        cluster: Cluster::Localnet,
        rpc: None,
        program: None,
        reference: String::new(),
        receipt: None,
        payer: None,
        minute_seconds: None,
        seed_file: None,
        refund: false,
        finish: false,
        plan: false,
        return_to: None,
        keep: 10_000_000,
        poleis: None,
        treasury: None,
    };
    while let Some(k) = a.next() {
        match k.as_str() {
            "--refund" => {
                out.refund = true;
                continue;
            }
            "--finish" => {
                out.finish = true;
                continue;
            }
            "--plan" => {
                out.plan = true;
                continue;
            }
            _ => {}
        }
        let v = a.next().ok_or_else(|| format!("{k} needs a value; {usage}"))?;
        match k.as_str() {
            "--cluster" => out.cluster = Cluster::parse(&v)?,
            "--rpc" => out.rpc = Some(v),
            "--program" => out.program = Some(v),
            "--reference" => out.reference = v,
            "--receipt" => out.receipt = Some(v),
            "--payer" => out.payer = Some(v),
            "--minute-seconds" => out.minute_seconds = Some(v.parse().map_err(|_| format!("--minute-seconds {v}: not a number of seconds"))?),
            "--seed-file" => out.seed_file = Some(v),
            "--return-to" => out.return_to = Some(v),
            "--keep" => out.keep = v.parse().map_err(|_| format!("--keep {v}: not a number of lamports"))?,
            "--poleis" => out.poleis = Some(v),
            "--treasury" => out.treasury = Some(v),
            _ => return Err(format!("{k}: {usage}")),
        }
    }
    if out.reference.is_empty() && out.return_to.is_none() {
        return Err(format!("--reference: {usage}"));
    }
    if out.return_to.is_some() && out.cluster != Cluster::Devnet {
        return Err("--return-to is the devnet run's last step".into());
    }
    if out.cluster == Cluster::Mainnet {
        return Err("the war's rehearsal runs on a local validator or on devnet, never on mainnet".into());
    }
    Ok(out)
}

/// the program's refusal by its number, in its own words (error.rs)
fn refusal_name(n: u64) -> String {
    use WarsError::*;
    let all = [
        Unauthorized, InvalidPda, InvalidAccount, LayoutVersion, InvalidAmount, InvalidRules, MathOverflow, ConfigStands, HouseStands, NotEnoughTokens, NothingOut, PastMin,
        NotEnoughCoins, InRaid, Rekt, Shielded, Unfair, StakeCap, OtherRaid, RaidOpen, NotInRaid, RaidOver, Staked, Cranked, NotOpen, Decided, NotDecided, AlreadySettled,
        NotSettled, NotTheLoser, NotTheWinner, NothingHeld, NotStaked, ReserveShort, SameHouse, LawRefused, NotSold, OtherConfig,
    ];
    all.iter().find(|e| **e as u64 == n).map(|e| format!("{e:?}")).unwrap_or_else(|| format!("Custom({n})"))
}
/// a transaction's error in words: the instruction that refused and the program's refusal by name
fn error_words(err: &Value) -> String {
    if let Some(ie) = err.get("InstructionError") {
        let at = ie.get(0).and_then(Value::as_u64).unwrap_or(0);
        if let Some(n) = ie.get(1).and_then(|e| e.get("Custom")).and_then(Value::as_u64) {
            return format!("instruction {at} refused with {} (Custom({n}))", refusal_name(n));
        }
        return format!("instruction {at} refused with {}", ie.get(1).cloned().unwrap_or(Value::Null));
    }
    err.to_string()
}

/// one transaction sent: what it was, its signature, the kinds of its program instructions in order
struct Sent {
    label: String,
    sig: String,
    kinds: Vec<&'static str>,
    /// its getTransaction meta (its units and its logs), read once it was confirmed, while it was recent
    meta: Option<Value>,
}

/// the chain this rehearsal talks to, and every transaction it sent
struct Chain {
    rpc: Rpc,
    sent: Vec<Sent>,
    last_blockhash: [u8; 32],
}

impl Chain {
    /// the commitment a transaction is held to before the run reads what it did: on a validator of this machine its own bank; on devnet the
    /// cluster's vote (a transaction only processed can still be dropped with its fork)
    fn landed_at(&self) -> &'static str {
        if self.rpc.cluster == Cluster::Devnet {
            "confirmed"
        } else {
            "processed"
        }
    }
    /// how often a status or the clock is asked while the run waits: about every 400 ms on devnet (its public endpoint's limits), 40 here
    fn poll(&self) -> Duration {
        Duration::from_millis(if self.rpc.cluster == Cluster::Devnet { 400 } else { 40 })
    }
}

impl Chain {
    /// an account at a commitment: its lamports and its data, None when there is none
    fn account_at(&self, key: &Pubkey, commitment: &str) -> Res<Option<(u64, Vec<u8>)>> {
        let v = self.rpc.call("getAccountInfo", json!([key.to_string(), { "encoding": "base64", "commitment": commitment }]))?;
        let a = &v["value"];
        if a.is_null() {
            return Ok(None);
        }
        Ok(Some((a["lamports"].as_u64().unwrap_or(0), wire::unb64(a["data"][0].as_str().ok_or("getAccountInfo: no data")?)?)))
    }
    fn data_at(&self, key: &Pubkey, commitment: &str) -> Res<Vec<u8>> {
        Ok(self.account_at(key, commitment)?.ok_or_else(|| format!("{key}: no account"))?.1)
    }
    fn balance_at(&self, key: &Pubkey, commitment: &str) -> Res<u64> {
        self.rpc.call("getBalance", json!([key.to_string(), { "commitment": commitment }]))?["value"].as_u64().ok_or_else(|| "getBalance: no lamports".into())
    }
    /// the chain's own clock, the Clock sysvar of the newest bank: its unix_timestamp (bytes 32..40: after slot, epoch_start_timestamp, epoch,
    /// leader_schedule_epoch)
    fn clock(&self) -> Res<i64> {
        let d = self.data_at(&CLOCK.parse::<Pubkey>().map_err(|e| format!("{e:?}"))?, "processed")?;
        Ok(i64::from_le_bytes(d.get(32..40).ok_or("the Clock sysvar is short")?.try_into().unwrap()))
    }
    /// a blockhash that is not the last one used: two crank-only messages under one blockhash would be the same transaction
    fn blockhash(&mut self) -> Res<[u8; 32]> {
        let begun = Instant::now();
        loop {
            let v = self.rpc.call("getLatestBlockhash", json!([{ "commitment": self.landed_at() }]))?;
            let bh = wire::unbase58::<32>(v["value"]["blockhash"].as_str().ok_or("getLatestBlockhash: no blockhash")?)?;
            if bh != self.last_blockhash {
                self.last_blockhash = bh;
                return Ok(bh);
            }
            if begun.elapsed() > Duration::from_secs(10) {
                return Err("the validator's blockhash did not move in 10 s".into());
            }
            std::thread::sleep(self.poll());
        }
    }
    /// One transaction, its fee paid by `payer`, signed by every key it names, sent with no preflight and sent again, the same bytes, until
    /// it lands (processed) or `bound` passes; a transaction the chain refused is the run's failure, in the program's own words.
    fn land(&mut self, label: &str, ixs: &[Instruction], payer: &Key, signers: &[&Key], kinds: Vec<&'static str>, bound: Duration) -> Res<String> {
        let bh = self.blockhash()?;
        let mut keys: Vec<&Key> = vec![payer];
        keys.extend(signers.iter().copied());
        let (tx, first) = wire::signed(ixs, &payer.address, &bh, &keys)?;
        if tx.len() > wire::PACKET {
            return Err(format!("{label}: a transaction of {} bytes, past the packet's {}", tx.len(), wire::PACKET));
        }
        let sig = wire::base58(&first);
        let begun = Instant::now();
        let resend = Duration::from_millis(if self.rpc.cluster == Cluster::Devnet { 2_000 } else { 500 });
        let mut last_send = Instant::now() - resend;
        let devnet = self.rpc.cluster == Cluster::Devnet;
        loop {
            if last_send.elapsed() >= resend {
                self.rpc.send(&tx, false)?;
                last_send = Instant::now();
            }
            if let Some(Some((_, err, c))) = self.rpc.statuses(&[sig.clone()])?.into_iter().next() {
                if !err.is_null() {
                    self.sent.push(Sent { label: label.to_string(), sig: sig.clone(), kinds, meta: None });
                    return Err(format!("{label}: the chain refused it: {}", error_words(&err)));
                }
                if !devnet || c == "confirmed" || c == "finalized" {
                    self.sent.push(Sent { label: label.to_string(), sig: sig.clone(), kinds, meta: None });
                    self.keep_up()?;
                    return Ok(sig);
                }
            }
            if begun.elapsed() > bound {
                return Err(format!("{label}: not landed in {} s", bound.as_secs()));
            }
            std::thread::sleep(self.poll());
        }
    }
    /// Every transaction sent and not yet confirmed, asked once: each confirmed one gets its getTransaction meta (its units, its logs) while it
    /// is recent. The status cache answers for about the last two minutes of signatures and nothing older, with or without the history
    /// searched (the first and the third runs, 11:42 and 11:54 UTC Nov 24 2026, were red at their end on 118 and 137 of their 178, every one
    /// of them landed in its minute): so each is confirmed as the run goes, after every landing, never all at the end.
    fn keep_up(&mut self) -> Res<()> {
        let open: Vec<usize> = (0..self.sent.len()).filter(|&i| self.sent[i].meta.is_none()).collect();
        if open.is_empty() {
            return Ok(());
        }
        let sigs: Vec<String> = open.iter().map(|&i| self.sent[i].sig.clone()).collect();
        let st = self.rpc.statuses(&sigs)?;
        for (&i, s) in open.iter().zip(st) {
            if let Some((_, _, c)) = s {
                if c == "confirmed" || c == "finalized" {
                    let t = self.rpc.call("getTransaction", json!([self.sent[i].sig, { "encoding": "json", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }]))?;
                    if !t["meta"].is_null() {
                        self.sent[i].meta = Some(t["meta"].clone());
                    }
                }
            }
        }
        Ok(())
    }
    /// wait until every transaction sent is confirmed and its meta read (`keep_up`, again and again, within `bound`)
    fn confirmed(&mut self, bound: Duration) -> Res<()> {
        let begun = Instant::now();
        loop {
            self.keep_up()?;
            let open: Vec<&str> = self.sent.iter().filter(|s| s.meta.is_none()).map(|s| s.label.as_str()).collect();
            if open.is_empty() {
                return Ok(());
            }
            if begun.elapsed() > bound {
                return Err(format!("{} of {} transactions not confirmed in {} s (the first: {})", open.len(), self.sent.len(), bound.as_secs(), open[0]));
            }
            std::thread::sleep(self.poll().max(Duration::from_millis(200)));
        }
    }
}

/// the rehearsal's fighters: a stand-in key a name, made in memory from the rehearsal's seed
struct Keys {
    seed: [u8; 32],
    made: BTreeMap<String, Key>,
}
impl Keys {
    fn of(&mut self, name: &str) -> &Key {
        let seed = self.seed;
        self.made.entry(name.to_string()).or_insert_with(|| wire::standin(&seed, &format!("the war's rehearsal: {name}")))
    }
    fn addr(&mut self, name: &str) -> Pubkey {
        self.of(name).address
    }
}
impl Drop for Keys {
    fn drop(&mut self) {
        self.seed.fill(0);
    }
}

/// the houses on the chain are the law's, to the unit (curve, held, burned, chest)
fn same(chain: &Chain, law: &War, houses: &[Pubkey; 2], commitment: &str, what: &str) -> Res<()> {
    for s in [Side::Attacker, Side::Defender] {
        let h = house_view(&chain.data_at(&houses[s.index()], commitment)?)?;
        let i = s.index();
        if h.curve != law.curves[i] || (h.held as u128, h.burned as u128, h.chest as u128) != (law.held[i], law.burned[i], law.chests[i]) {
            return Err(format!("{what}: the {} on the chain is not the law's: chain {:?} held {} burned {} chest {}, law {:?} held {} burned {} chest {}", HOUSE[i], h.curve, h.held, h.burned, h.chest, law.curves[i], law.held[i], law.burned[i], law.chests[i]));
        }
    }
    Ok(())
}

fn int(v: &Value) -> Res<u128> {
    v.as_u64().map(|x| x as u128).ok_or_else(|| format!("an integer: {v}"))
}
fn check(ok: bool, what: String, failed: &mut Vec<String>) {
    if !ok {
        failed.push(what);
    }
}

fn main() {
    let began = Instant::now();
    let a = args();
    let mut receipt = json!({ "rehearsal": "the first reference war of reference-in-sol.json (The war as before, now on the clock, in SOL)" });
    let result = match &a {
        Ok(a) => rehearse(a, &mut receipt),
        Err(e) => Err(e.clone()),
    };
    receipt["wall_seconds"] = json!((began.elapsed().as_secs_f64() * 10.0).round() / 10.0);
    match &result {
        Ok(()) => receipt["result"] = json!("pass"),
        Err(e) => {
            receipt["result"] = json!("fail");
            receipt["why"] = json!(e);
        }
    }
    let text = serde_json::to_string_pretty(&receipt).unwrap_or_default();
    println!("{text}");
    if let Ok(Args { receipt: Some(p), .. }) = &a {
        if let Err(e) = std::fs::write(p, format!("{text}\n")) {
            eprintln!("war-rehearsal: the receipt could not be written to {p}: {e}");
        }
    }
    std::process::exit(if result.is_ok() { 0 } else { 1 });
}

/// the names of the first reference war's forty, in the model's order
fn the_forty() -> Vec<(String, Side)> {
    (0..N).flat_map(|i| [Side::Attacker, Side::Defender].map(|h| (format!("{}{i}", HOUSE[h.index()]), h))).collect()
}
/// the stand-ins besides the forty: the game swarm's seal, which founds the war's generation (it only signs); each house's creator, who
/// launches it and pays its rent; on a local validator the game swarm itself, the solo authority of its Poleis tree, which founds its unit's
/// treasury; on devnet with no --treasury, a publisher standing in for the swarm's treasury
const SEAL: &str = "the game swarm's seal";
const CREATORS: [&str; 2] = ["the crow's creator", "the wolf's creator"];
const SWARM: &str = "the game swarm";
const PUBLISHER: &str = "the publisher";
/// the game swarm's Poleis tree and unit on a local validator: any root but 0, which only Poleis's deployer founds
const SWARM_ROOT: u64 = 7;
const SWARM_UNIT: u64 = 7;

/// the cluster's rents, of each account the war makes
#[derive(Clone, Copy)]
struct Rents {
    config: u64,
    fees: u64,
    house: u64,
    raid: u64,
    position: u64,
}
impl Rents {
    /// a cluster's default rent (3,480 lamports a byte-year, two years exempt, 128 bytes of overhead): what --plan reads without a chain
    fn default_rent() -> Rents {
        let r = |size: usize| (128 + size as u64) * 3_480 * 2;
        Rents { config: r(Config::SIZE), fees: r(Fees::SIZE), house: r(House::SIZE), raid: r(Raid::SIZE), position: r(Position::SIZE) }
    }
    fn of(rpc: &Rpc) -> Res<Rents> {
        Ok(Rents { config: rpc.rent(Config::SIZE)?, fees: rpc.rent(Fees::SIZE)?, house: rpc.rent(House::SIZE)?, raid: rpc.rent(Raid::SIZE)?, position: rpc.rent(Position::SIZE)? })
    }
}

/// WHAT EACH STAND-IN IS GIVEN on devnet: what the war takes from its own wallet, by its role in the first reference war, and MARGIN.
/// Every buyer one SOL and the rent of his position; the wolf's twenty a second position's rent (the ten who change sides open one in the
/// crow; the ten who convert open theirs before their wolf's closes); crow19, who stakes first, the raid's rent; each house's creator its
/// house's rent. The fee payer pays every transaction's fee and the rents of the generation (its config and its fee account); the seal only
/// signs.
fn needs(rents: &Rents) -> Vec<(String, u64)> {
    let mut out: Vec<(String, u64)> = the_forty()
        .into_iter()
        .map(|(n, h)| {
            let mut need = LAMPORTS + rents.position + MARGIN;
            if h == Side::Defender {
                need += rents.position;
            }
            if n == "crow19" {
                need += rents.raid;
            }
            (n, need)
        })
        .collect();
    for c in CREATORS {
        out.push((c.to_string(), rents.house + MARGIN));
    }
    out
}

/// THE PLAN (--plan): the first reference war by the law alone, then every holder of the winner selling all his coins in the order they came
/// in (as the devnet run does once the dump tax has fallen to nothing): what the run gives its stand-ins, what comes back, and what is spent
/// for good by its cause. No chain is read; the rents are a cluster's default and the network's fees an estimate of 5,000 lamports a
/// signature.
fn plan(receipt: &mut Value) -> Res<()> {
    let rents = Rents::default_rent();
    let (w, d) = poleis_war::war::fight(&R, &the_four()[0])?;
    let wi = d.winner.index();
    let mut c = w.curves[wi];
    let mut sold = 0u128;
    for who in &w.came {
        let coins = w.coins.get(&(who.clone(), d.winner)).copied().unwrap_or(0);
        if coins > 0 {
            let (after, s) = sell_on(&R, &c, coins, 0)?;
            c = after;
            sold += s.net();
        }
    }
    let given: u64 = needs(&rents).iter().map(|(_, n)| n).sum::<u64>() + rents.config + rents.fees;
    let took: u128 = w.took.values().sum();
    // what the houses and the fee account took in is the forty's SOL, less what the deserters took and the sales paid out: so the war's fees
    // are that, less what stays in the houses (the lamports a change of sides may leave its changer, ten at most, are counted as fees here)
    let stays_all: u128 = w.curves.iter().map(|c| c.rq).sum::<u128>() + w.chests.iter().sum::<u128>() - w.curves[wi].rq + c.rq;
    let war_fees = 40 * LAMPORTS as u128 - took - sold - stays_all;
    let rents_kept = rents.config + rents.fees + 2 * rents.house + rents.raid + 50 * rents.position;
    let stays = stays_all;
    // what comes back: every desert's payout, every sale, the margins, and the rents of the ten loser positions closed at their converts
    let back = took + sold + 42 * MARGIN as u128 + 10 * rents.position as u128;
    let signatures: u64 = 5 + 3 * 2 + 10 * 5 + 2 * 2 + 150 + 20 + 1 + 10 * 2 + 2 * 2 + 30 * 2 + 6 + 41;
    receipt["plan"] = json!({
        "the payer gives its stand-ins": sol(given as u128),
        "the payer needs, at most at once": sol(given as u128 + signatures as u128 * 5_000),
        "comes back": sol(back),
        "spent for good": sol(given as u128 - back + signatures as u128 * 5_000),
        "spent for good, by cause": {
            "what stays in the winner's house (its reserve and chest, behind the burned coins)": sol(stays),
            "the war's own fees, in its fee account (95 + 30 bp of every trade)": sol(war_fees),
            "the rents of the accounts left standing (the config, the fee account, two houses, the raid, 50 positions)": sol(rents_kept as u128),
            "the network's fees (an estimate: 5,000 lamports a signature)": sol(signatures as u128 * 5_000),
        },
        "check": sol(stays + war_fees + rents_kept as u128 + signatures as u128 * 5_000),
    });
    Ok(())
}
fn sol(lamports: u128) -> Value {
    json!({ "lamports": lamports as u64, "sol": (lamports as f64 / LAMPORTS as f64 * 1e6).round() / 1e6 })
}

/// the run's seed file on devnet: its path (by default ~/.embertide/poleis-wars-devnet/run-seed)
fn seed_path(a: &Args) -> Res<std::path::PathBuf> {
    if let Some(p) = &a.seed_file {
        return Ok(p.into());
    }
    let home = std::env::var_os("HOME").ok_or("no HOME")?;
    Ok(std::path::PathBuf::from(home).join(".embertide/poleis-wars-devnet/run-seed"))
}
/// a new run's seed: 32 bytes of the system's randomness written first to the seed file, created mode 600 and never over another one (an
/// earlier run's stand-ins may still hold SOL: --refund them first, which moves the file aside)
fn seed_new(path: &std::path::Path) -> Res<[u8; 32]> {
    use std::io::{Read as _, Write as _};
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut seed = [0u8; 32];
    std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut seed)).map_err(|e| format!("the system's randomness: {e}"))?;
    let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path).map_err(|e| {
        format!("{}: {e} (a seed of an earlier run stands: its stand-ins may still hold SOL; run --refund first, which moves it aside)", path.display())
    })?;
    f.write_all(&seed).and_then(|_| f.sync_all()).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(seed)
}
/// an earlier run's seed, read back for --refund: 32 bytes, a file only its owner reads
fn seed_read(path: &std::path::Path) -> Res<[u8; 32]> {
    use std::os::unix::fs::PermissionsExt as _;
    let m = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if m.permissions().mode() & 0o077 != 0 {
        return Err(format!("{}: readable by others than its owner (mode {:o}): not read", path.display(), m.permissions().mode() & 0o777));
    }
    let b = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    b.try_into().map_err(|_| format!("{}: not a seed of 32 bytes", path.display()))
}
/// the seed file moved aside once every stand-in of it is empty, so the next run makes its own
fn seed_retire(path: &std::path::Path) -> Res<String> {
    let to = format!("{}.refunded-{}", path.display(), utc_now());
    std::fs::rename(path, &to).map_err(|e| format!("{}: {e}", path.display()))?;
    // said with the home folder as ~, so a receipt names no user
    let home = std::env::var("HOME").unwrap_or_default();
    Ok(if !home.is_empty() && to.starts_with(&home) { format!("~{}", &to[home.len()..]) } else { to })
}
fn utc_now() -> String {
    let out = std::process::Command::new("/bin/date").args(["-u", "+%Y-%m-%dT%H%M%SZ"]).output().ok();
    out.and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_string()).unwrap_or_else(|| "unknown".into())
}

/// a system transfer: [2u32 LE][lamports u64 LE], from (s,w) to (w)
fn transfer(from: Pubkey, to: Pubkey, lamports: u64) -> Instruction {
    let mut d = 2u32.to_le_bytes().to_vec();
    d.extend_from_slice(&lamports.to_le_bytes());
    Instruction::new_with_bytes(SYSTEM, &d, vec![AccountMeta::new(from, true), AccountMeta::new(to, false)])
}

/// every stand-in of the seed sends its whole balance back to the payer, eight a transaction (each signs its own transfer, the payer pays
/// the fees): what came back
fn send_back(chain: &mut Chain, keys: &mut Keys, payer: &Key, names: &[String], t: Duration) -> Res<u128> {
    let mut back = 0u128;
    let mut holding: Vec<(String, u64)> = Vec::new();
    for n in names {
        let who = keys.addr(n);
        let b = chain.balance_at(&who, "confirmed")?;
        if b > 0 {
            holding.push((n.clone(), b));
        }
    }
    for chunk in holding.chunks(8) {
        let ixs: Vec<Instruction> = chunk.iter().map(|(n, b)| transfer(keys.made[n].address, payer.address, *b)).collect();
        let signers: Vec<&Key> = chunk.iter().map(|(n, _)| keys.made.get(n).unwrap()).collect();
        chain.land("back to the payer", &ixs, payer, &signers, vec![], t)?;
        back += chunk.iter().map(|(_, b)| *b as u128).sum::<u128>();
    }
    Ok(back)
}

/// the explorer's page of an address on devnet
fn explorer(a: &Pubkey) -> String {
    format!("https://explorer.solana.com/address/{a}?cluster=devnet")
}

/// THE RETURN (devnet, after the run): everything the payer holds beyond `keep` goes back to the wallet that funded it, in one transfer the
/// payer signs and pays the fee of (5,000 lamports, one signature), so the payer is left holding `keep` exactly
fn give_back(a: &Args, to: &str, receipt: &mut Value) -> Res<()> {
    // the address and the key first, so a wrong one is refused before the chain is asked anything
    let dest: Pubkey = to.parse().map_err(|e| format!("--return-to {to}: not an address ({e:?})"))?;
    let payer_text = std::fs::read_to_string(a.payer.as_ref().ok_or("--payer <keypair.json>: the key that gives back")?).map_err(|e| format!("the payer's keypair file: {e}"))?;
    let payer = Key::from_cli_text(&payer_text)?;
    if dest == payer.address {
        return Err("--return-to names the payer itself".into());
    }
    let rpc = Rpc::new(poleis_wars_rehearsal::rpc::DEVNET_URL, Cluster::Devnet)?;
    receipt["cluster"] = json!(rpc.cluster.name());
    receipt["genesis"] = json!(rpc.check_cluster()?);
    let mut chain = Chain { rpc, sent: Vec::new(), last_blockhash: [0; 32] };
    let before = chain.balance_at(&payer.address, "confirmed")?;
    let to_before = chain.balance_at(&dest, "confirmed")?;
    let fee = 5_000u64;
    if before <= a.keep + fee {
        return Err(format!("the payer holds {before} lamports, not more than the {} it keeps and the fee", a.keep));
    }
    let amount = before - a.keep - fee;
    let sig = chain.land("back to his wallet", &[transfer(payer.address, dest, amount)], &payer, &[], vec![], Duration::from_secs(90))?;
    chain.confirmed(Duration::from_secs(120))?;
    let after = chain.balance_at(&payer.address, "confirmed")?;
    let to_after = chain.balance_at(&dest, "confirmed")?;
    let fee_paid = chain.sent.iter().filter_map(|s| s.meta.as_ref().and_then(|m| m["fee"].as_u64())).sum::<u64>();
    receipt["return"] = json!({
        "from_the_payer": payer.address.to_string(), "to": dest.to_string(), "signature": sig,
        "explorer": format!("https://explorer.solana.com/tx/{sig}?cluster=devnet"),
        "sent": sol(amount as u128), "the network's fee": sol(fee_paid as u128),
        "the payer, before and after": [sol(before as u128), sol(after as u128)], "his wallet, before and after": [sol(to_before as u128), sol(to_after as u128)],
    });
    if after != a.keep || to_after != to_before + amount {
        return Err(format!("the payer holds {after}, not the {} it keeps; his wallet gained {}, not {amount}", a.keep, to_after as i128 - to_before as i128));
    }
    Ok(())
}

/// THE WAR THAT FELL IS FINISHED BY THE CLOCK (Nov 24 2026, lane/the-publisher-stands-on-devnet): a devnet run that missed a minute (17:27 UTC, its
/// minute-62 transaction 11.1 s to land, this Mac at load 240) left its war standing mid-raid, its fighters' SOL in the two houses. The program reads
/// no minute nobody cranked and reads a raid after its last minute at that minute, so the war is ended as any war nobody attends would be: the raid's
/// end waited out on the chain's clock, a crank until it is decided, the settlement, every convert of the loser's holders, the winner's stakers'
/// unlock, the winner's dump tax waited out, every holder's sale, each creator and the publisher paid, every stand-in's whole balance back to the
/// payer. Nothing here is held to the reference (the war left its script at the missed minute): every step is read off the chain, and the book is
/// the SOL's.
fn finish(a: &Args, chain: &mut Chain, keys: &mut Keys, payer: &Key, all: &[String], receipt: &mut Value) -> Res<()> {
    let (t5, at) = (Duration::from_secs(90), "confirmed");
    let treasury: Pubkey = a.treasury.as_ref().ok_or("--finish --treasury <address>: the publisher's treasury the war's generation names")?.parse().map_err(|e| format!("--treasury: {e:?}"))?;
    receipt["rehearsal"] = json!("a war a run left standing mid-raid, finished by the clock: read off the chain alone, held to no reference");
    let seal = wire::standin(&keys.seed, &format!("the war's rehearsal: {SEAL}"));
    let config = config_pda(&seal.address, 0);
    let houses = [house_pda(&config, 0), house_pda(&config, 1)];
    let raid = raid_pda(&houses[0], &houses[1]);
    let payer_before = chain.balance_at(&payer.address, at)?;
    let mut r = raid_view(&chain.data_at(&raid, at)?)?;
    if r.status == RAID_PROPOSED {
        return Err("the raid was never opened: --refund sends the stand-ins' SOL back".into());
    }
    receipt["the war as it stood"] = json!({ "config": config.to_string(), "raid": raid.to_string(), "status": r.status, "next_minute": r.next_minute, "minutes": r.minutes, "minute_seconds": r.minute_seconds });
    // the raid's end (a stand's own last minute when one stands), on the chain's clock; then a crank until it is decided
    let mut cranks = Vec::new();
    while r.status == RAID_ACTIVE || r.status == RAID_STAND {
        if cranks.len() >= 12 {
            return Err(format!("the raid is not decided after 12 cranks: {cranks:?}"));
        }
        let last = if r.status == RAID_STAND { r.stand.2.max(r.minutes) } else { r.minutes }.max(r.next_minute);
        let due = r.start + (last as i64 + 1) * r.minute_seconds as i64;
        loop {
            let ts = chain.clock()?;
            if ts >= due {
                break;
            }
            std::thread::sleep(Duration::from_secs(((due - ts).max(1) as u64).min(20)));
        }
        match chain.land("the crank after the end", &[crank(config, houses[0], houses[1])], payer, &[], vec!["crank"], t5) {
            Ok(sig) => cranks.push(sig),
            Err(e) => {
                cranks.push(format!("refused: {e}"));
                std::thread::sleep(Duration::from_secs(r.minute_seconds as u64));
            }
        }
        chain.confirmed(Duration::from_secs(120))?;
        r = raid_view(&chain.data_at(&raid, at)?)?;
    }
    let Some(Phase::Decided { winner, how, at: decided_at }) = r.phase() else { return Err("the raid reads as no decision".into()) };
    if r.status != RAID_SETTLED {
        chain.land("settle", &[settle(config, houses[0], houses[1])], payer, &[], vec!["settle"], t5)?;
    }
    let (wi, li) = (winner.index(), winner.other().index());
    let fighters: Vec<String> = the_forty().into_iter().map(|(n, _)| n).collect();
    let (mut converts, mut unlocks) = (0u32, 0u32);
    for n in &fighters {
        let who = keys.addr(n);
        if let Some((_, d)) = chain.account_at(&position_pda(&houses[li], &who), at)? {
            if position_view(&d)?.coins > 0 {
                chain.land("convert", &[convert(who, config, raid, houses[li], houses[wi])], payer, &[keys.made.get(n).unwrap()], vec!["convert"], t5)?;
                converts += 1;
            }
        }
    }
    if winner == Side::Attacker {
        for n in &fighters {
            let who = keys.addr(n);
            if let Some((_, d)) = chain.account_at(&position_pda(&houses[0], &who), at)? {
                if position_view(&d)?.staked > 0 {
                    chain.land("unlock", &[unlock(who, raid, houses[0])], payer, &[keys.made.get(n).unwrap()], vec!["unlock"], t5)?;
                    unlocks += 1;
                }
            }
        }
    }
    chain.confirmed(Duration::from_secs(120))?;
    // the winner's dump tax waited out on the chain's clock, then every holder of the winner sells all his coins, in the order they came in
    let dump_end = house_view(&chain.data_at(&houses[wi], at)?)?.dump_end;
    loop {
        let ts = chain.clock()?;
        if ts >= dump_end {
            break;
        }
        std::thread::sleep(Duration::from_secs(((dump_end - ts).max(1) as u64).min(20)));
    }
    let (mut got_sum, mut sellers) = (0u128, 0u32);
    for n in &fighters {
        let who = keys.addr(n);
        let Some((_, d)) = chain.account_at(&position_pda(&houses[wi], &who), at)? else { continue };
        let coins = position_view(&d)?.coins;
        if coins == 0 {
            continue;
        }
        let before = chain.balance_at(&who, at)?;
        chain.land("a sale after the dump tax", &[sell(who, config, houses[wi], coins, 0)], payer, &[keys.made.get(n).unwrap()], vec!["sell"], t5)?;
        got_sum += chain.balance_at(&who, at)?.saturating_sub(before) as u128;
        sellers += 1;
    }
    // each house's creator paid what it owes him, the publisher everything the fee account holds beyond its rent
    let mut paid_creators = Vec::new();
    for i in 0..2 {
        let h = house_view(&chain.data_at(&houses[i], at)?)?;
        if h.creator_owed == 0 {
            continue;
        }
        let before = chain.balance_at(&h.creator, at)?;
        chain.land("pay_creator", &[pay_creator(houses[i], h.creator)], payer, &[], vec!["pay_creator"], t5)?;
        let got = chain.balance_at(&h.creator, at)?.saturating_sub(before);
        if got != h.creator_owed {
            return Err(format!("the {}'s creator got {got}; the house owed him {}", HOUSE[i], h.creator_owed));
        }
        paid_creators.push(json!({ "house": HOUSE[i], "creator": h.creator.to_string(), "paid": sol(got as u128) }));
    }
    let treasury_before = chain.balance_at(&treasury, at)?;
    chain.land("pay_publisher", &[pay_publisher(config, treasury)], payer, &[], vec!["pay_publisher"], t5)?;
    let to_publisher = chain.balance_at(&treasury, at)?.saturating_sub(treasury_before);
    chain.confirmed(Duration::from_secs(120))?;
    let (wh, lh) = (house_view(&chain.data_at(&houses[wi], at)?)?, house_view(&chain.data_at(&houses[li], at)?)?);
    let back = send_back(chain, keys, payer, all, t5)?;
    chain.confirmed(Duration::from_secs(120))?;
    let moved = seed_retire(&seed_path(a)?)?;
    let payer_after = chain.balance_at(&payer.address, at)?;
    receipt["the war as it ended"] = json!({
        "winner": HOUSE[wi], "how": how_text(how, decided_at), "decided_at_minute": decided_at, "cranks": cranks,
        "converts": converts, "unlocks": unlocks, "sellers": sellers, "sol_the_sellers_got": sol(got_sum),
        "the winner's reserve left behind the burned coins": sol(wh.curve.rq), "the winner's chest": sol(wh.chest as u128), "the loser's status": lh.status,
    });
    receipt["paid"] = json!({
        "creators": paid_creators,
        "publisher": { "treasury": treasury.to_string(), "before": sol(treasury_before as u128), "paid": sol(to_publisher as u128), "after": sol((treasury_before + to_publisher) as u128) },
    });
    receipt["sol"] = json!({ "back: the stand-ins sent back": sol(back), "payer_before": sol(payer_before as u128), "payer_after": sol(payer_after as u128), "seed_moved_to": moved });
    receipt["accounts_left_standing"] = json!({ "config": explorer(&config), "raid": explorer(&raid), "the attacker's house": explorer(&houses[0]), "the defender's house": explorer(&houses[1]) });
    Ok(())
}

fn rehearse(a: &Args, receipt: &mut Value) -> Res<()> {
    if a.plan {
        return plan(receipt);
    }
    if let Some(to) = &a.return_to {
        return give_back(a, to, receipt);
    }
    let devnet = a.cluster == Cluster::Devnet;
    let url = match (&a.rpc, devnet) {
        (Some(u), _) => u.clone(),
        (None, true) => poleis_wars_rehearsal::rpc::DEVNET_URL.to_string(),
        (None, false) => return Err("--rpc http://127.0.0.1:<port>".into()),
    };
    let minute_seconds = a.minute_seconds.unwrap_or(if devnet { MINUTE_SECONDS_DEVNET } else { MINUTE_SECONDS_LOCAL });
    receipt["cluster"] = json!(a.cluster.name());
    receipt["minute_seconds"] = json!(minute_seconds);
    let rpc = Rpc::new(&url, a.cluster)?;
    let genesis = rpc.check_cluster()?;
    let version = rpc.call("getVersion", json!([]))?;
    receipt["validator"] = json!({ "solana-core": version["solana-core"], "feature-set": version["feature-set"], "genesis": genesis });
    let program = rpc.account(&PROGRAM_ID.to_string())?.ok_or("poleis_wars is not at its id on this chain")?;
    if !program.executable {
        return Err("the program's account is not executable".into());
    }
    let mut prog = json!({ "id": PROGRAM_ID.to_string() });
    if let Some(path) = &a.program {
        let elf = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        prog["this_tree's_build"] = json!({ "sha256": wire::hex(&wire::sha256(&elf)), "bytes": elf.len() });
        if devnet {
            // the upgradeable loader's program account names its program data: [3u32][slot u64][Some u8][authority 32][the ELF, padded]
            let pd = Pubkey::new_from_array(program.data.get(4..36).ok_or("the program account is short")?.try_into().unwrap());
            let d = rpc.account(&pd.to_string())?.ok_or("the program's data account")?.data;
            let body = d.get(45..).ok_or("the program data is short")?;
            let deployed = body.get(..elf.len()).ok_or("the deployed program is shorter than this tree's build")?;
            let same = deployed == elf.as_slice() && body[elf.len()..].iter().all(|b| *b == 0);
            prog["deployed"] = json!({
                "program_data": pd.to_string(), "slot": u64::from_le_bytes(d[4..12].try_into().unwrap()),
                "upgrade_authority": if d[12] == 1 { json!(Pubkey::new_from_array(d[13..45].try_into().unwrap()).to_string()) } else { Value::Null },
                "sha256_of_this_tree's_length": wire::hex(&wire::sha256(deployed)), "the same bytes as this tree's build": same,
            });
        }
    }
    receipt["program"] = prog;
    let reference: Vec<Value> = serde_json::from_str::<Value>(&std::fs::read_to_string(&a.reference).map_err(|e| format!("{}: {e}", a.reference))?)
        .map_err(|e| format!("the reference: {e}"))?
        .as_array()
        .cloned()
        .ok_or("the reference is a list of wars")?;
    let scenario = the_four()[0];
    let r0 = reference.first().ok_or("the reference holds no war")?;
    if r0["name"].as_str() != Some(scenario.name) {
        return Err(format!("the reference's first war is {:?}, not {:?}", r0["name"], scenario.name));
    }
    let mut chain = Chain { rpc, sent: Vec::new(), last_blockhash: [0; 32] };
    let rents = Rents::of(&chain.rpc)?;
    let (rent_house, rent_fees) = (rents.house, rents.fees);
    let names = the_forty();
    let mut all: Vec<String> = names.iter().map(|(n, _)| n.clone()).collect();
    all.extend(CREATORS.iter().map(|c| c.to_string()));
    all.push(SEAL.to_string());
    all.push(if devnet { PUBLISHER } else { SWARM }.to_string());

    // the stand-in keys and their funding: in memory and the faucet's on a local validator; from the seed file and the payer's own
    // transfers on devnet
    let local_payer;
    let devnet_payer;
    let seed_file = if devnet { Some(seed_path(a)?) } else { None };
    let mut keys = if let Some(path) = &seed_file {
        let payer_text = std::fs::read_to_string(a.payer.as_ref().ok_or("--payer <keypair.json>: the devnet run's payer")?).map_err(|e| format!("the payer's keypair file: {e}"))?;
        devnet_payer = Some(Key::from_cli_text(&payer_text)?);
        let seed = if a.refund || a.finish { seed_read(path)? } else { seed_new(path)? };
        Keys { seed, made: BTreeMap::new() }
    } else {
        devnet_payer = None;
        let mut seed = [0u8; 32];
        {
            use std::io::Read as _;
            std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut seed)).map_err(|e| format!("the system's randomness: {e}"))?;
        }
        let k = Keys { seed, made: BTreeMap::new() };
        seed.fill(0);
        k
    };
    let t5 = Duration::from_secs(if devnet { 90 } else { 20 });
    if a.refund {
        let payer = devnet_payer.as_ref().ok_or("--refund is devnet's")?;
        let before = chain.balance_at(&payer.address, "confirmed")?;
        let back = send_back(&mut chain, &mut keys, payer, &all, t5)?;
        chain.confirmed(Duration::from_secs(120))?;
        let moved = seed_retire(seed_file.as_ref().unwrap())?;
        receipt["refund"] = json!({ "back_to_the_payer": sol(back), "payer_before": sol(before as u128), "payer_after": sol(chain.balance_at(&payer.address, "confirmed")? as u128), "seed_moved_to": moved });
        return Ok(());
    }
    if a.finish {
        let payer = devnet_payer.as_ref().ok_or("--finish is devnet's")?;
        return finish(a, &mut chain, &mut keys, payer, &all, receipt);
    }
    let mut given = 0u128;
    let payer_before;
    let payer: &Key = if let Some(p) = &devnet_payer {
        payer_before = chain.balance_at(&p.address, "confirmed")?;
        let need = needs(&rents);
        let total: u64 = need.iter().map(|(_, n)| n).sum();
        receipt["payer"] = json!({ "address": p.address.to_string(), "before": sol(payer_before as u128) });
        if payer_before < total + 5 * LAMPORTS / 1_000 {
            return Err(format!("the payer holds {} lamports; the run gives its stand-ins {total} and needs about 0.005 SOL of fees more", payer_before));
        }
        for chunk in need.chunks(10) {
            let ixs: Vec<Instruction> = chunk.iter().map(|(n, l)| transfer(p.address, keys.addr(n), *l)).collect();
            chain.land("the payer funds its stand-ins", &ixs, p, &[], vec![], t5)?;
            given += chunk.iter().map(|(_, l)| *l as u128).sum::<u128>();
        }
        p
    } else {
        payer_before = 0;
        let mut drops = Vec::new();
        for (name, lamports) in [("the fee payer", 50 * LAMPORTS), (SWARM, 10 * LAMPORTS), (CREATORS[0], LAMPORTS), (CREATORS[1], LAMPORTS)].into_iter().chain(names.iter().map(|(n, _)| (n.as_str(), 3 * LAMPORTS))) {
            let addr = keys.addr(name);
            drops.push(chain.rpc.airdrop(&addr.to_string(), lamports)?);
        }
        let begun = Instant::now();
        loop {
            let st = chain.rpc.statuses(&drops)?;
            if st.iter().all(|s| matches!(s, Some((_, e, c)) if e.is_null() && (c == "confirmed" || c == "finalized"))) {
                break;
            }
            if begun.elapsed() > Duration::from_secs(60) {
                return Err("the faucet's airdrops were not confirmed in 60 s".into());
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        local_payer = wire::standin(&keys.seed, "the war's rehearsal: the fee payer");
        &local_payer
    };
    let seal = wire::standin(&keys.seed, &format!("the war's rehearsal: {SEAL}"));
    let creators = [wire::standin(&keys.seed, &format!("the war's rehearsal: {}", CREATORS[0])), wire::standin(&keys.seed, &format!("the war's rehearsal: {}", CREATORS[1]))];
    let config = config_pda(&seal.address, 0);
    let houses = [house_pda(&config, 0), house_pda(&config, 1)];
    let raid = raid_pda(&houses[0], &houses[1]);
    let at = chain.landed_at();

    // THE GAME IS ONE SWARM: the publisher's treasury. On a local validator the game swarm's own, of Poleis loaded at genesis: its tree and
    // its unit's treasury founded first by the swarm (the solo authority of its tree). On devnet the treasury --treasury names, else a stand-in.
    let poleis: Option<Pubkey> = if devnet { None } else { Some(a.poleis.as_ref().ok_or("--poleis <id>: Poleis's program, loaded at genesis")?.parse().map_err(|e| format!("--poleis: {e:?}"))?) };
    let treasury: Pubkey = match (&poleis, &a.treasury) {
        (Some(p), _) => {
            let swarm = wire::standin(&keys.seed, &format!("the war's rehearsal: {SWARM}"));
            let prog = chain.rpc.account(&p.to_string())?.ok_or("Poleis is not at its id on this chain")?;
            if !prog.executable {
                return Err("Poleis's program account is not executable".into());
            }
            chain.land("Poleis: the game swarm's tree", &[poleis_init_config(p, swarm.address, SWARM_ROOT)], payer, &[&swarm], vec![], t5)?;
            chain.land("Poleis: the game swarm's treasury", &[poleis_create_treasury(p, swarm.address, SWARM_ROOT, SWARM_UNIT)], payer, &[&swarm], vec![], t5)?;
            poleis_treasury(p, SWARM_UNIT)
        }
        (None, Some(t)) => t.parse().map_err(|e| format!("--treasury {t}: {e:?}"))?,
        (None, None) => keys.addr(PUBLISHER),
    };
    let treasury_before = chain.balance_at(&treasury, at)?;

    // the war's generation 0, at its minute and no shield, founded by the swarm's seal (it signs, the payer pays) and naming the publisher's
    // treasury; the two houses, each launched by its creator
    chain.land("init_config", &[init_config(payer.address, seal.address, 0, treasury, &rules_bytes(&R, 0, 0, minute_seconds))], payer, &[&seal], vec!["init_config"], t5)?;
    chain.land("init_house", &[init_house(creators[0].address, config, 0)], payer, &[&creators[0]], vec!["init_house"], t5)?;
    chain.land("init_house", &[init_house(creators[1].address, config, 1)], payer, &[&creators[1]], vec!["init_house"], t5)?;

    // the forty, one SOL each, in the model's order, four a transaction (the order kept inside each), the law beside them
    let mut law = War::new(R);
    for chunk in names.chunks(4) {
        let ixs: Vec<Instruction> = chunk.iter().map(|(n, h)| buy(keys.addr(n), config, houses[h.index()], LAMPORTS, 0, None)).collect();
        let signers: Vec<&Key> = chunk.iter().map(|(n, _)| keys.made.get(n).unwrap()).collect();
        chain.land("the forty's buys", &ixs, payer, &signers, vec!["buy"; chunk.len()], t5)?;
        for (n, h) in chunk {
            law.enter(n, *h, LAW_SOL)?;
        }
    }
    same(&chain, &law, &houses, at, "after the forty")?;

    // the stakes: the crow's last ones in, all their coins, crow19 first and down; the stake that opens the raid goes with minute 0's events
    // and its crank, so minute 0 is read in minute 0
    let mut events: Vec<Event> = Vec::new();
    for i in (10..N).rev() {
        if law.opening.is_some() {
            break;
        }
        let name = format!("crow{i}");
        let who = keys.addr(&name);
        let coins = position_view(&chain.data_at(&position_pda(&houses[0], &who), at)?)?.coins;
        let mut probe = law.clone();
        if !probe.stake(&name)? {
            chain.land("a stake", &[stake(who, config, houses[0], houses[1], coins)], payer, &[keys.made.get(&name).unwrap()], vec!["stake"], t5)?;
            law.stake(&name)?;
            let r = raid_view(&chain.data_at(&raid, at)?)?;
            if r.status != RAID_PROPOSED {
                return Err(format!("{name}'s stake: the raid is {} on the chain, a proposal in the law", r.status));
            }
            continue;
        }
        let t = probe.opening.map(|o| o.minutes).unwrap_or(0);
        events = (scenario.script)(t);
        events.sort();
        let (mut ixs, mut kinds, owners) = minute_ixs(&events, 0, &law, &mut keys, config, houses, raid)?;
        ixs.insert(0, stake(who, config, houses[0], houses[1], coins));
        kinds.insert(0, "stake");
        ixs.push(crank(config, houses[0], houses[1]));
        kinds.push("crank");
        let mut signers: Vec<&Key> = vec![keys.made.get(&name).unwrap()];
        signers.extend(owners.iter().map(|n| keys.made.get(n).unwrap()));
        chain.land("the opening stake and minute 0", &ixs, payer, &signers, kinds, t5)?;
        law.stake(&name)?;
        play_on_law(&mut law, &events, 0)?;
        law.minute(0)?;
    }
    let o = law.opening.ok_or("the stake never opened the raid")?;
    let r = raid_view(&chain.data_at(&raid, at)?)?;
    if (r.m0[0] as u128, r.m0[1] as u128, r.need as u128, r.minutes) != (o.m0[0], o.m0[1], o.need, o.minutes) {
        return Err(format!("the opening on the chain ({:?}, {}, {}) is not the law's ({:?}, {}, {})", r.m0, r.need, r.minutes, o.m0, o.need, o.minutes));
    }
    if r.next_minute != 1 || r.phase() != Some(law.phase) || r.minute_seconds != minute_seconds || r.seconds != o.minutes as u64 * minute_seconds as u64 {
        return Err(format!("minute 0 missed: the crank read minute {} at {} s a minute; the chain's phase {:?}, the law's {:?}", r.next_minute as i64 - 1, r.minute_seconds, r.phase(), law.phase));
    }
    let start = r.start;
    let t = o.minutes;
    let mut played = 1u32;
    let mut took: BTreeMap<String, u128> = BTreeMap::new();
    let mut lateness: Vec<i64> = Vec::new();
    let mut landing: Vec<f64> = Vec::new();
    let ms = minute_seconds as i64;
    // the chain's clock as last read, and when: on devnet the run sleeps until a little before the next minute is due, then asks
    let (mut last_ts, mut last_at) = (chain.clock()?, Instant::now());

    // minute by minute: the chain's own clock until the minute has begun, then that minute's events and its crank in one transaction
    if !matches!(law.phase, Phase::Decided { .. }) {
        for m in 1..=t + R.stand_minutes {
            let due = start + (m as i64) * ms;
            if devnet {
                let ahead = due - last_ts - last_at.elapsed().as_secs() as i64;
                if ahead > 1 {
                    std::thread::sleep(Duration::from_secs((ahead - 1) as u64));
                }
            }
            let waited = Instant::now();
            let (cm, ts) = loop {
                let ts = chain.clock()?;
                (last_ts, last_at) = (ts, Instant::now());
                let cm = ((ts - start).max(0) / ms) as u32;
                if cm >= m {
                    break (cm, ts);
                }
                if waited.elapsed() > Duration::from_secs(5 * minute_seconds as u64 + 20) {
                    return Err(format!("minute {m}: the chain's clock stood at {ts} for {} s", waited.elapsed().as_secs()));
                }
                std::thread::sleep(chain.poll());
            };
            if cm > m {
                return Err(format!("minute {m} missed: the chain's clock was at minute {cm} ({ts}) before its transaction could be sent"));
            }
            lateness.push(ts - due);
            let (mut ixs, mut kinds, owners) = minute_ixs(&events, m, &law, &mut keys, config, houses, raid)?;
            ixs.push(crank(config, houses[0], houses[1]));
            kinds.push("crank");
            let deserters: Vec<String> = events.iter().filter(|e| e.minute == m && e.kind == Kind::Desert && !law.is_staker(&e.who)).map(|e| e.who.clone()).collect();
            let mut before = BTreeMap::new();
            for n in &deserters {
                before.insert(n.clone(), chain.balance_at(&keys.addr(n), at)?);
            }
            let signers: Vec<&Key> = owners.iter().map(|n| keys.made.get(n).unwrap()).collect();
            let sent_at = Instant::now();
            chain.land(&format!("minute {m}"), &ixs, payer, &signers, kinds, Duration::from_secs(minute_seconds as u64 * 4))?;
            landing.push(sent_at.elapsed().as_secs_f64());
            played += 1;
            let r = raid_view(&chain.data_at(&raid, at)?)?;
            if r.next_minute != m + 1 {
                return Err(format!(
                    "minute {m} missed: its transaction landed in minute {} of the chain's clock (sent {} s into its minute, {:.1} s to land at {at})",
                    r.next_minute as i64 - 1,
                    ts - due,
                    sent_at.elapsed().as_secs_f64()
                ));
            }
            let payouts = play_on_law(&mut law, &events, m)?;
            for (n, paid) in payouts {
                let got = chain.balance_at(&keys.addr(&n), at)? as u128 - before[&n] as u128;
                if got != paid {
                    return Err(format!("minute {m}: {n} deserted and was paid {got} on the chain, {paid} by the law"));
                }
                took.insert(n, paid);
            }
            let p = law.minute(m)?;
            if r.phase() != Some(p) {
                return Err(format!("minute {m}: the chain's phase {:?}, the law's {:?}", r.phase(), p));
            }
            // both houses against the law: every minute on a local validator; on devnet a minute an event moved them, and the last
            if !devnet || !deserters.is_empty() || events.iter().any(|e| e.minute == m) || matches!(p, Phase::Decided { .. }) {
                same(&chain, &law, &houses, at, &format!("minute {m}"))?;
            }
            if let Phase::Decided { .. } = p {
                break;
            }
        }
    }
    let Phase::Decided { winner, how, at: decided_at } = law.phase else { return Err(format!("the raid was never decided: {:?}", law.phase)) };

    // the settlement, every convert of the loser's holders, the winner's stakers' unlock
    chain.land("settle", &[settle(config, houses[0], houses[1])], payer, &[], vec!["settle"], t5)?;
    let done = law.settle()?;
    let (wi, li) = (winner.index(), winner.other().index());
    let all_names: Vec<String> = law.came.clone();
    let mut converts = Vec::new();
    for n in &all_names {
        let who = keys.addr(n);
        if let Some((_, d)) = chain.account_at(&position_pda(&houses[li], &who), at)? {
            if position_view(&d)?.coins > 0 {
                converts.push(n.clone());
            }
        }
    }
    for n in &converts {
        let who = keys.addr(n);
        chain.land("convert", &[convert(who, config, raid, houses[li], houses[wi])], payer, &[keys.made.get(n).unwrap()], vec!["convert"], t5)?;
    }
    if winner == Side::Attacker {
        for n in &done.stakers {
            let who = keys.addr(n);
            chain.land("unlock", &[unlock(who, raid, houses[0])], payer, &[keys.made.get(n).unwrap()], vec!["unlock"], t5)?;
        }
    }
    chain.confirmed(Duration::from_secs(120))?;

    // every account read back, confirmed, and held to the law, to the reference and to the book
    let mut failed: Vec<String> = Vec::new();
    let r = raid_view(&chain.data_at(&raid, "confirmed")?)?;
    let (wl, wd) = chain.account_at(&houses[wi], "confirmed")?.ok_or("the winner's house")?;
    let (ll, ld) = chain.account_at(&houses[li], "confirmed")?.ok_or("the loser's house")?;
    let (wh, lh) = (house_view(&wd)?, house_view(&ld)?);
    let (fl, fd) = chain.account_at(&fees_pda(&config), "confirmed")?.ok_or("the fee account")?;
    let f = fees_view(&fd)?;
    let st = &r0["settled"];
    let chain_how = r.phase().map(|p| match p {
        Phase::Decided { how, at, .. } => how_text(how, at),
        other => format!("{other:?}"),
    });
    check(r.status == RAID_SETTLED, format!("the raid's status {}", r.status), &mut failed);
    check(HOUSE[r.winner as usize % 2] == r0["winner"].as_str().unwrap_or(""), format!("the winner: chain {}, reference {}", r.winner, r0["winner"]), &mut failed);
    check(chain_how.as_deref() == r0["how"].as_str(), format!("how: chain {chain_how:?}, reference {}", r0["how"]), &mut failed);
    check(how_text(how, decided_at) == r0["how"].as_str().unwrap_or(""), format!("how: the law {}, reference {}", how_text(how, decided_at), r0["how"]), &mut failed);
    check(r.minutes as u128 == int(&r0["minutes"])?, format!("minutes: chain {}, reference {}", r.minutes, r0["minutes"]), &mut failed);
    check(r.need as f64 == st["stake_need"].as_f64().unwrap_or(-1.0).floor(), format!("the need: chain {}, reference {}", r.need, st["stake_need"]), &mut failed);
    for (k, chain_v) in [
        ("pot_in", r.pot_in as u128),
        ("bought_with", r.bought_with as u128),
        ("coins_bought", r.coins_bought as u128),
        ("winner_reserve", wh.curve.rq),
        ("winner_chest", wh.chest as u128),
        ("all_held", wh.held as u128),
    ] {
        check(chain_v == int(&st[k])?, format!("{k}: chain {chain_v}, reference {}", st[k]), &mut failed);
    }
    for (i, h) in HOUSE.iter().enumerate() {
        check(r.m0[i] as u128 == int(&st["m0"][h])?, format!("m0 {h}: chain {}, reference {}", r.m0[i], st["m0"][h]), &mut failed);
        check(r.mcap_end[i] as u128 == int(&st["mcap_end"][h])?, format!("mcap_end {h}: chain {}, reference {}", r.mcap_end[i], st["mcap_end"][h]), &mut failed);
    }
    check(wh.curve == law.curves[wi] && wh.held as u128 == done.all_held, "the winner's house is not the law's".into(), &mut failed);
    check((wh.status, lh.status, wh.raid) == (HOUSE_ACTIVE, HOUSE_REKT, Pubkey::default()), format!("the houses' statuses {} and {}", wh.status, lh.status), &mut failed);
    let stakers: Vec<String> = st["stakers"].as_array().cloned().unwrap_or_default().iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
    check(done.stakers == stakers, format!("the stakers: law {:?}, reference {stakers:?}", done.stakers), &mut failed);
    // every fighter, to the unit
    let pf = r0["per_fighter"].as_object().ok_or("the reference's per_fighter")?;
    check(pf.len() == all_names.len(), format!("fighters: {} on the chain, {} in the reference", all_names.len(), pf.len()), &mut failed);
    let mut held = [0u128; 2];
    let mut holds_of: BTreeMap<String, u64> = BTreeMap::new();
    for n in &all_names {
        let who = keys.addr(n);
        let mut holds = 0u128;
        for (i, house) in houses.iter().enumerate() {
            if let Some((_, d)) = chain.account_at(&position_pda(house, &who), "confirmed")? {
                let p = position_view(&d)?;
                held[i] += p.coins as u128;
                if i == wi {
                    holds = p.coins as u128;
                }
                if p.staked != 0 {
                    failed.push(format!("{n} still has {} coins staked in the {}", p.staked, HOUSE[i]));
                }
            }
        }
        holds_of.insert(n.clone(), holds as u64);
        let backed = backed_of(wh.curve.rq + wh.chest as u128, holds, wh.held as u128)?;
        let took_n = took.get(n).copied().unwrap_or(0);
        let x = pf.get(n).ok_or_else(|| format!("{n} is not in the reference"))?;
        for (k, v) in [("holds", holds), ("backed_lamports", backed), ("took_lamports", took_n)] {
            check(v == int(&x[k])?, format!("{n} {k}: chain {v}, reference {}", x[k]), &mut failed);
        }
    }
    // the book: every house holds its rent, its reserve and its chest; the fee account its rent and what it was paid; held is what positions hold
    check(wl as u128 == rent_house as u128 + wh.curve.rq + wh.chest as u128 + wh.creator_owed as u128, format!("the winner holds {wl} lamports, not its rent, reserve, chest and what it owes its creator"), &mut failed);
    check(ll as u128 == rent_house as u128 + lh.curve.rq + lh.chest as u128 + lh.creator_owed as u128, format!("the loser holds {ll} lamports, not its rent, reserve, chest and what it owes its creator"), &mut failed);
    check(fl == rent_fees + f.holds(), format!("the fee account holds {fl} lamports, not its rent and protocol + taxed + donated - paid"), &mut failed);
    // THE GAME IS ONE SWARM: what each house owes its creator, and what the fee account holds for the publisher, are the law's fees
    for (i, h) in [(wi, &wh), (li, &lh)] {
        check(h.creator_owed as u128 == law.creator_fees[i], format!("the {} owes its creator {}, the law {}", HOUSE[i], h.creator_owed, law.creator_fees[i]), &mut failed);
        check(h.creator == creators[i].address, format!("the {}'s creator on the chain is not the one who launched it", HOUSE[i]), &mut failed);
    }
    check((f.protocol as u128, f.taxed as u128) == (law.protocol_fees, law.taxed), format!("the fee account holds protocol {} and taxed {}, the law {} and {}", f.protocol, f.taxed, law.protocol_fees, law.taxed), &mut failed);
    check(held[wi] == wh.held as u128 && held[li] == lh.held as u128, format!("held: positions {held:?}, houses {} and {}", wh.held, lh.held), &mut failed);
    if !failed.is_empty() {
        receipt["held_to_the_reference"] = json!({ "fighters": all_names.len(), "deserters_paid": took.len(), "converts": converts.len(), "differences": failed });
        return Err(format!("{} numbers differ from the reference or the book; the first: {}", failed.len(), failed[0]));
    }

    // ON DEVNET, THE SALES: the winner's dump tax waited out on the chain's clock, every holder of the winner selling all his coins in the
    // order they came in, each sale held to the law's own (`War::sell`, whose fees are the law's)
    let mut sales = json!(null);
    if devnet {
        let dump_end = wh.dump_end;
        loop {
            let ts = chain.clock()?;
            if ts >= dump_end {
                break;
            }
            std::thread::sleep(Duration::from_secs(((dump_end - ts).max(1) as u64).min(20)));
        }
        let (mut got_sum, mut n_sold) = (0u128, 0u32);
        for n in &all_names {
            let coins = holds_of.get(n).copied().unwrap_or(0);
            if coins == 0 {
                continue;
            }
            let s = law.sell(n, winner, coins as u128, 0)?;
            let who = keys.addr(n);
            let before = chain.balance_at(&who, "confirmed")?;
            chain.land("a sale after the dump tax", &[sell(who, config, houses[wi], coins, 0)], payer, &[keys.made.get(n).unwrap()], vec!["sell"], t5)?;
            let got = chain.balance_at(&who, "confirmed")? as u128 - before as u128;
            if got != s.net() {
                return Err(format!("{n} sold {coins} coins of the {} and got {got} on the chain, {} by the law", HOUSE[wi], s.net()));
            }
            got_sum += got;
            n_sold += 1;
        }
        let wh2 = house_view(&chain.data_at(&houses[wi], "confirmed")?)?;
        if wh2.curve != law.curves[wi] || wh2.held != 0 {
            return Err(format!("after the sales the {} is {:?} holding {} coins; the law's {:?}", HOUSE[wi], wh2.curve, wh2.held, law.curves[wi]));
        }
        sales = json!({ "sellers": n_sold, "sol_they_got": sol(got_sum), "the winner's reserve left behind the burned coins": sol(wh2.curve.rq), "burned": wh2.burned });
    }

    // THE GAME IS ONE SWARM: every house's creator paid what it owes him, the publisher everything the fee account holds beyond its rent,
    // each held to the law's fees of every trade; on a local validator Poleis's own sync_endowment then books the publisher's share. Every
    // read here is at the commitment a transaction is held to before the run reads it (`at`, as the war's own checks read): on a local
    // validator a read at "confirmed" right after a landing at "processed" can see the account as it stood before (the first run of this
    // lane read the publisher's pay as 0 while Poleis booked all of it)
    let mut paid_creators = Vec::new();
    let mut to_creators = 0u128;
    for i in 0..2 {
        let h = house_view(&chain.data_at(&houses[i], at)?)?;
        let before = chain.balance_at(&creators[i].address, at)?;
        chain.land("pay_creator", &[pay_creator(houses[i], creators[i].address)], payer, &[], vec!["pay_creator"], t5)?;
        let got = (chain.balance_at(&creators[i].address, at)? - before) as u128;
        check(got == law.creator_fees[i] && got == h.creator_owed as u128, format!("the {}'s creator got {got}; it owed {}, the law {}", HOUSE[i], h.creator_owed, law.creator_fees[i]), &mut failed);
        to_creators += got;
        paid_creators.push(json!({ "house": HOUSE[i], "creator": creators[i].address.to_string(), "paid": sol(got) }));
    }
    let before = chain.balance_at(&treasury, at)?;
    chain.land("pay_publisher", &[pay_publisher(config, treasury)], payer, &[], vec!["pay_publisher"], t5)?;
    let to_publisher = (chain.balance_at(&treasury, at)? - before) as u128;
    check(to_publisher == law.protocol_fees + law.taxed, format!("the publisher got {to_publisher}; the law's protocol fees and tax's fee half are {}", law.protocol_fees + law.taxed), &mut failed);
    let booked = if let Some(p) = &poleis {
        let total_in = |chain: &Chain| -> Res<u64> {
            let d = chain.data_at(&treasury, at)?;
            if d.len() != POLEIS_TREASURY_SIZE {
                return Err("the publisher's treasury is not Poleis's".into());
            }
            Ok(u64::from_le_bytes(d[POLEIS_TREASURY_OFF_TOTAL_IN..POLEIS_TREASURY_OFF_TOTAL_IN + 8].try_into().unwrap()))
        };
        let in_before = total_in(&chain)?;
        chain.land("Poleis: sync_endowment", &[poleis_sync_endowment(p, SWARM_UNIT)], payer, &[], vec![], t5)?;
        let in_after = total_in(&chain)?;
        check((in_after - in_before) as u128 == to_publisher, format!("Poleis booked {} in the treasury's total_in; the publisher was paid {to_publisher}", in_after - in_before), &mut failed);
        json!({ "total_in_before": in_before, "total_in_after": in_after })
    } else {
        Value::Null
    };
    let f3 = fees_view(&chain.data_at(&fees_pda(&config), at)?)?;
    check(f3.holds() == 0 && f3.paid as u128 == to_publisher, format!("after the publisher was paid, its fee account holds {} beyond its rent and paid {}", f3.holds(), f3.paid), &mut failed);
    receipt["paid"] = json!({
        "the houses' creators (whoever launched each house)": paid_creators,
        "the publisher": {
            "treasury": treasury.to_string(),
            "what it is": if poleis.is_some() { format!("the game swarm's Poleis treasury: unit {SWARM_UNIT} of tree {SWARM_ROOT}, poleis_treasury loaded at genesis") }
                else if a.treasury.is_some() { "the treasury --treasury named".to_string() }
                else { "a stand-in of the seed, no --treasury named: its SOL goes back with the stand-ins'".to_string() },
            "paid": sol(to_publisher), "the treasury before": sol(treasury_before as u128),
            "Poleis's own books after its sync_endowment": booked,
        },
    });

    // every payment confirmed and its meta read while it is recent, before the units are read below: `keep_up` reads a meta only once its
    // transaction is confirmed, and the war's own landings were confirmed before its read-back, but the payments land after it (the second
    // local run of this lane was red on "no meta read" for its last landing, Poleis's sync_endowment, still only processed when it returned)
    chain.confirmed(Duration::from_secs(120))?;

    // ON DEVNET, THE END: every stand-in's whole balance back to the payer, and the SOL's book
    let mut sol_book = json!(null);
    if devnet {
        let back = send_back(&mut chain, &mut keys, payer, &all, t5)?;
        chain.confirmed(Duration::from_secs(120))?;
        let payer_after = chain.balance_at(&payer.address, "confirmed")?;
        let network_fees: u128 = chain.sent.iter().filter_map(|s| s.meta.as_ref().and_then(|m| m["fee"].as_u64())).map(|f| f as u128).sum();
        let (fl2, fd2) = chain.account_at(&fees_pda(&config), "confirmed")?.ok_or("the fee account")?;
        let f2 = fees_view(&fd2)?;
        let (wl2, wd2) = chain.account_at(&houses[wi], "confirmed")?.ok_or("the winner's house")?;
        let (ll2, ld2) = chain.account_at(&houses[li], "confirmed")?.ok_or("the loser's house")?;
        let (wh2, lh2) = (house_view(&wd2)?, house_view(&ld2)?);
        let in_houses = wh2.curve.rq + wh2.chest as u128 + wh2.creator_owed as u128 + lh2.curve.rq + lh2.chest as u128 + lh2.creator_owed as u128;
        // the positions left standing, and every rent the war leaves behind
        let mut positions = 0u128;
        for n in &all_names {
            for house in &houses {
                if chain.account_at(&position_pda(house, &keys.addr(n)), "confirmed")?.is_some() {
                    positions += 1;
                }
            }
        }
        let rents_kept = (rents.config + rents.fees + 2 * rents.house + rents.raid) as u128 + positions * rents.position as u128;
        let mut left_with_stand_ins = 0u128;
        for n in &all {
            left_with_stand_ins += chain.balance_at(&keys.addr(n), "confirmed")? as u128;
        }
        // what a real treasury kept of the publisher's share (a stand-in's came back with the stand-ins')
        let kept_by_the_publisher = if a.treasury.is_some() { to_publisher } else { 0 };
        let spent = payer_before as u128 - payer_after as u128;
        let causes = network_fees + rents_kept + in_houses + f2.holds() as u128 + kept_by_the_publisher + left_with_stand_ins;
        check(spent == causes, format!("the payer spent {spent} lamports for good; its causes add up to {causes}"), &mut failed);
        check(
            wl2 as u128 == rent_house as u128 + wh2.curve.rq + wh2.chest as u128 + wh2.creator_owed as u128 && ll2 as u128 == rent_house as u128 + lh2.curve.rq + lh2.chest as u128 + lh2.creator_owed as u128,
            "a house's lamports are not its rent, reserve, chest and what it owes its creator at the end".into(),
            &mut failed,
        );
        check(fl2 == rent_fees + f2.holds(), "the fee account's lamports are not its rent and its book at the end".into(), &mut failed);
        sol_book = json!({
            "in: the payer gave its stand-ins (and paid the generation's rents)": sol(given + (rents.config + rents.fees) as u128), "back: the stand-ins sent back": sol(back),
            "the payer, before and after": [sol(payer_before as u128), sol(payer_after as u128)],
            "spent for good": sol(spent),
            "spent for good, by cause": {
                "the network's fees (every transaction's, the payer's)": sol(network_fees),
                "rents of the accounts left standing (the config, the fee account, two houses, the raid, the positions)": { "lamports": rents_kept as u64, "positions": positions },
                "what stays in the houses (their reserves and chests; the winner's behind its burned coins)": sol(in_houses),
                "what the fee account still holds": sol(f2.holds() as u128),
                "what the publisher's treasury kept": sol(kept_by_the_publisher),
                "left with the stand-ins": sol(left_with_stand_ins),
            },
            "paid out of the war: the houses' creators and the publisher": [sol(to_creators), sol(to_publisher)],
        });
        if left_with_stand_ins == 0 {
            receipt["seed"] = json!({ "moved_aside_to": seed_retire(seed_file.as_ref().unwrap())? });
        }
        receipt["accounts_left_standing"] = json!({
            "config": { "address": config.to_string(), "explorer": explorer(&config) },
            "crow": { "address": houses[0].to_string(), "explorer": explorer(&houses[0]) },
            "wolf": { "address": houses[1].to_string(), "explorer": explorer(&houses[1]) },
            "raid": { "address": raid.to_string(), "explorer": explorer(&raid) },
            "fees": { "address": fees_pda(&config).to_string(), "explorer": explorer(&fees_pda(&config)) },
            "program": { "address": PROGRAM_ID.to_string(), "explorer": explorer(&PROGRAM_ID) },
        });
    }

    // the compute units, as the chain reported them: each transaction's computeUnitsConsumed, and each of its program instructions' own
    // line in its logs, in order
    let mut cu: BTreeMap<&'static str, (u64, u64, u64, u64)> = BTreeMap::new();
    let mut per_tx: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
    let marker = format!("Program {} consumed ", PROGRAM_ID);
    for s in &chain.sent {
        let meta = s.meta.as_ref().ok_or_else(|| format!("{}: no meta read", s.label))?;
        let total = meta["computeUnitsConsumed"].as_u64().ok_or_else(|| format!("{}: no computeUnitsConsumed", s.label))?;
        let lines: Vec<u64> = meta["logMessages"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|l| l.as_str().and_then(|l| l.strip_prefix(&marker)).and_then(|rest| rest.split(' ').next()).and_then(|n| n.parse().ok()))
            .collect();
        if lines.len() != s.kinds.len() {
            failed.push(format!("{}: {} instructions sent, {} reported in its logs", s.label, s.kinds.len(), lines.len()));
        }
        for (k, c) in s.kinds.iter().zip(&lines) {
            let e = cu.entry(k).or_insert((u64::MAX, 0, 0, 0));
            *e = (e.0.min(*c), e.1.max(*c), e.2 + c, e.3 + 1);
        }
        let label = if s.label.starts_with("minute ") { "a minute (its events and its crank)".to_string() } else { s.label.clone() };
        let e = per_tx.entry(label).or_insert((u64::MAX, 0, 0));
        *e = (e.0.min(total), e.1.max(total), e.2 + 1);
    }
    let mean = |v: &[f64]| if v.is_empty() { 0.0 } else { (v.iter().sum::<f64>() / v.len() as f64 * 100.0).round() / 100.0 };
    receipt["raid"] = json!({
        "minutes": t, "seconds": r.seconds, "minute_seconds": r.minute_seconds, "decided_at_minute": decided_at, "winner": HOUSE[wi], "how": how_text(how, decided_at),
        "minutes_played": played, "minutes_missed": [], "clock_seconds_into_each_minute_when_sent": {
            "min": lateness.iter().min(), "max": lateness.iter().max(), "mean": if lateness.is_empty() { 0.0 } else { lateness.iter().sum::<i64>() as f64 / lateness.len() as f64 } },
        "seconds_from_send_to_landed": { "at": at, "min": landing.iter().cloned().fold(f64::INFINITY, f64::min), "max": landing.iter().cloned().fold(0.0, f64::max), "mean": mean(&landing) },
    });
    receipt["transactions"] = json!(chain.sent.len());
    receipt["compute_units"] = Value::Object(cu.iter().map(|(k, (lo, hi, sum, n))| (k.to_string(), json!({ "n": n, "min": lo, "max": hi, "mean": sum / n.max(&1) }))).collect());
    receipt["transaction_compute_units"] = Value::Object(per_tx.iter().map(|(k, (lo, hi, n))| (k.clone(), json!({ "n": n, "min": lo, "max": hi }))).collect());
    receipt["held_to_the_reference"] = json!({ "fighters": all_names.len(), "deserters_paid": took.len(), "converts": converts.len(), "differences": failed });
    if devnet {
        receipt["after_the_dump_tax"] = sales;
        receipt["sol"] = sol_book;
    }
    receipt["refusals"] = json!([]);
    receipt["rpc_calls"] = json!(chain.rpc.calls.borrow().values().sum::<u64>());
    receipt["rpc_refused_by_rate_and_waited_out"] = json!(chain.rpc.refused_by_rate.get());
    if !failed.is_empty() {
        return Err(format!("{} numbers differ from the reference or the book; the first: {}", failed.len(), failed[0]));
    }
    Ok(())
}

/// minute `m`'s events as instructions, in the model's order (a staker's own events skipped, as the model skips them): each one's kind and the
/// names of the fighters who sign
fn minute_ixs(events: &[Event], m: u32, law: &War, keys: &mut Keys, config: Pubkey, houses: [Pubkey; 2], raid: Pubkey) -> Res<(Vec<Instruction>, Vec<&'static str>, Vec<String>)> {
    let (mut ixs, mut kinds, mut owners) = (Vec::new(), Vec::new(), Vec::new());
    for e in events.iter().filter(|e| e.minute == m) {
        match e.kind {
            Kind::Reinforce => {
                let who = keys.addr(&e.who);
                ixs.push(buy(who, config, houses[e.house.index()], LAMPORTS, 0, Some(raid)));
                kinds.push("buy in a raid");
                owners.push(e.who.clone());
            }
            Kind::Cross if !law.is_staker(&e.who) => {
                let who = keys.addr(&e.who);
                ixs.push(change_sides(who, config, raid, houses[1], houses[0], 0));
                kinds.push("change_sides");
                owners.push(e.who.clone());
            }
            Kind::Desert if !law.is_staker(&e.who) => {
                let who = keys.addr(&e.who);
                let src = if law.coins.get(&(e.who.clone(), Side::Attacker)).copied().unwrap_or(0) > 0 { 0 } else { 1 };
                ixs.push(desert(who, config, houses[src], raid));
                kinds.push("desert");
                owners.push(e.who.clone());
            }
            _ => {}
        }
    }
    Ok((ixs, kinds, owners))
}

/// minute `m`'s events on the law's war, in the model's order: what each deserter was paid
fn play_on_law(law: &mut War, events: &[Event], m: u32) -> Res<Vec<(String, u128)>> {
    let mut paid = Vec::new();
    for e in events.iter().filter(|e| e.minute == m) {
        match e.kind {
            Kind::Reinforce => law.come_in(&e.who, e.house),
            Kind::Cross if !law.is_staker(&e.who) => {
                law.cross(&e.who, Side::Defender)?;
            }
            Kind::Desert if !law.is_staker(&e.who) => {
                paid.push((e.who.clone(), law.desert(&e.who)?));
            }
            _ => {}
        }
    }
    Ok(paid)
}
