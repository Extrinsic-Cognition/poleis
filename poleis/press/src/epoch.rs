//! THE EPOCH TURNS THE CRANKS (Nov 25 2026, lane/the-epoch-turns-the-cranks; his "Yes, build it" to "Build what runs the two cranks once an epoch,
//! while the lanes wait?"). His words it stands on, UTC: Nov 25 13:11 "we gotta launch swarmbook and the stake sol to fund snowtracers"; Nov 20 20:18
//! "we are supposed to delegate automatically like marinade does but with our own rules".
//!
//! `press epoch`: ONE PASS of what an epoch asks, each step only when the chain says it is due, in this order:
//!
//! 1. THE ENGINE (engine.rs `run`, its key the pool's staker act 9 named): her pool brought to the chain's epoch, its validators added, the reserve
//!    placed and stake brought back by the params' rule. It waits, and the pass with it, while the epoch's rewards are being paid.
//! 2. THE BOOK (the stake book's own law, `poleis_stake::count`): one read of her holders a pool-updated epoch, laid on the last. The only state the
//!    runner keeps, beside the sitting's keys: D/epoch/book.json (the book after its last read, as the book's own hand keeps it for its caller,
//!    `poleis-stake count --book F`), and A MISSED EPOCH'S LIGHT IS POURED LATER, NEVER DROPPED: each counted epoch's two books in
//!    D/epoch/owed/<epoch>.json, written before the book moves and removed only once the chain holds its pour's record (at most OWED_BOUND).
//! 3. THE POUR (light.rs `crank` and `send`, its key the pourer's): every owed epoch, oldest first, read staker by staker and each wish's share sent
//!    down its road, netted against what the wish already took; the crank's memos on the chain are the record, so a pour is never sent twice. A count
//!    across a gap (the runner slept through epochs) is the book's one count of it, said by name, with any fee no holder's share took.
//!
//! Run twice in an epoch, the second pass sends nothing and says one line. A step that fails stops the pass with its step, its cause and its numbers;
//! the next pass starts from the chain's own state (and the book), never from a memory of the failure.
//!
//! `press epoch --stay`: what turns it is the epoch, not a poll. After a pass it reads the chain's epoch once, sleeps until the estimated first slot
//! of the next epoch plus a margin, and runs a pass; if it woke early (the epoch has not turned) it sleeps the remainder, never spins. While it sleeps
//! it holds nothing: every read is one curl. It writes one line a pass (what it placed, counted, poured, left; or that nothing was due) and errors.
//!
//! SENDS THAT SURVIVE AN EPOCH'S BOUNDARY (`land_surely`): the runner sends at boundaries, where a validator may hold bytes and never forward them (the
//! launch rehearsal's third run, slot 1984). Every send is polled until confirmed, sent again every two seconds, and built again with a new blockhash
//! once its own is past its last valid block height, a bounded number of times; then the step fails and says so.
//!
//! THE KEYS (seat.rs `door_key`, the miner's road): the engine's key and the pourer's, each made once by `press engine-key` and `press pourer` into
//! D/keys/engine.json and D/keys/pourer.json (mode 600, the address beside each), read by the runner from there and nowhere else. The runner refuses,
//! naming what is missing, where the params' engine or light names no rule (their LATER blanks), where a key is absent, or where the engine's key is
//! not the one the params name the pool's staker; it never falls back to another key.

use crate::door::{self, Keys};
use crate::params::Params;
use crate::rpc::Rpc;
use crate::{engine, light, wire, Res};
use poleis_stake::{self as stake, Book, Snapshot};
use serde_json::{json, Value};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------- the pure half

/// What a pass finds due, from the chain's epoch, the pool's last update, whether the epoch's rewards are being paid, and the book's last read and row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Due {
    /// the epoch's rewards are being paid: nothing may move her pool, the pass waits
    pub wait: bool,
    /// her pool is behind the chain's epoch: the engine brings it (and places the stake)
    pub bring: bool,
    /// the book's last read is of an older epoch: a read is due once the pool stands at the epoch
    pub count: bool,
    /// the book counted the chain's epoch: its pour is due unless the chain's records already hold it
    pub pour: bool,
}

pub fn due(chain_epoch: u64, pool_epoch: u64, rewards_paying: bool, book_read: Option<u64>, book_row: Option<u64>) -> Due {
    let bring = pool_epoch < chain_epoch;
    Due {
        wait: bring && rewards_paying,
        bring,
        count: book_read.is_none_or(|e| e < chain_epoch),
        pour: book_row == Some(chain_epoch) || book_read.is_none_or(|e| e < chain_epoch),
    }
}

/// How long to sleep until the next epoch's first slot plus `margin` slots, from the slot's index in this epoch, the epoch's length and a slot's
/// measured length. Never less than a slot: a sleep of nothing would spin.
pub fn sleep_for(slot_index: u64, slots_in_epoch: u64, slot_ms: u64, margin: u64) -> Duration {
    let left = slots_in_epoch.saturating_sub(slot_index) + margin;
    Duration::from_millis(left.max(1) * slot_ms.max(1))
}

/// What a send does next, from what the chain says of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    Done,
    Failed,
    /// nothing yet: keep polling
    Wait,
    /// the same bytes, sent again
    Resend,
    /// its blockhash is past its last valid block height: built again with a new one
    Rebuild,
    /// built again `bound` times and never confirmed
    GiveUp,
}

/// THE BOUND ON RESENDS: confirmed or failed ends it; past its blockhash's last valid height it is built again, at most `bound` builds; otherwise the
/// same bytes go again every `every` (two seconds), and between it waits.
pub fn next(confirmed: bool, failed: bool, height: u64, last_valid: u64, since_send: Duration, builds: u32, bound: u32) -> Next {
    if failed {
        return Next::Failed;
    }
    if confirmed {
        return Next::Done;
    }
    if height > last_valid {
        return if builds >= bound { Next::GiveUp } else { Next::Rebuild };
    }
    if since_send >= Duration::from_secs(2) {
        Next::Resend
    } else {
        Next::Wait
    }
}

/// What every send of a run said: how many landed, how many sends went again, how many were built again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sends {
    pub landed: u32,
    pub resent: u32,
    pub rebuilt: u32,
    pub fees: u64,
}

// ---------------------------------------------------------------- sends that survive a boundary

thread_local! {
    static SENDS: std::cell::Cell<Sends> = const { std::cell::Cell::new(Sends { landed: 0, resent: 0, rebuilt: 0, fees: 0 }) };
}

/// The sends counted since the last call, and the count set back to nothing.
pub fn sends_taken() -> Sends {
    SENDS.with(|s| s.replace(Sends::default()))
}

/// The builds a send may take before its step fails.
pub const BUILDS: u32 = 5;

/// One transaction landed by the keys given (the first its fee payer): built, sent, polled until confirmed, the same bytes sent again every two
/// seconds, built again with a new blockhash once its own is past its last valid block height, at most `BUILDS` builds. Its signature and its fee.
pub fn land_surely(rpc: &Rpc, ixs: &[Instruction], keys: &[&Key]) -> Res<(String, u64)> {
    let payer = keys.first().ok_or("no key to pay")?;
    let mut builds = 0u32;
    'build: loop {
        let v = rpc.call("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))?;
        let hash = wire::unbase58::<32>(v["value"]["blockhash"].as_str().ok_or("getLatestBlockhash: no blockhash")?)?;
        let last_valid = v["value"]["lastValidBlockHeight"].as_u64().ok_or("getLatestBlockhash: no last valid height")?;
        let (w, _) = wire::signed(ixs, &payer.address, &hash, keys)?;
        builds += 1;
        if builds > 1 {
            SENDS.with(|s| s.set(Sends { rebuilt: s.get().rebuilt + 1, ..s.get() }));
        }
        let sig = match rpc.send(&w, true) {
            Ok(s) => s,
            // a preflight refusal that names the blockhash is a build to make again; any other is the chain's own word
            Err(e) if e.contains("Blockhash not found") || e.contains("BlockhashNotFound") => {
                if builds >= BUILDS {
                    return Err(format!("never sent after {builds} builds: {e}"));
                }
                std::thread::sleep(Duration::from_millis(400));
                continue 'build;
            }
            Err(e) => return Err(e),
        };
        let mut sent = Instant::now();
        loop {
            let st = rpc.statuses(std::slice::from_ref(&sig))?.pop().flatten();
            let (confirmed, failed) = match &st {
                Some((_, err, _)) if !err.is_null() => (false, true),
                Some((_, _, s)) => (s == "confirmed" || s == "finalized", false),
                None => (false, false),
            };
            let height = if confirmed || failed { 0 } else { rpc.call("getBlockHeight", json!([{ "commitment": "confirmed" }]))?.as_u64().unwrap_or(0) };
            match next(confirmed, failed, height, last_valid, sent.elapsed(), builds, BUILDS) {
                Next::Done => {
                    let fee = crate::rpc::wait(Duration::from_secs(30), || Ok(rpc.transaction(&sig)?.map(|t| t.1)))?;
                    SENDS.with(|s| s.set(Sends { landed: s.get().landed + 1, fees: s.get().fees + fee, ..s.get() }));
                    return Ok((sig, fee));
                }
                Next::Failed => return Err(format!("{sig}: {}", st.map(|s| s.1.to_string()).unwrap_or_default())),
                Next::Resend => {
                    let _ = rpc.send(&w, false);
                    SENDS.with(|s| s.set(Sends { resent: s.get().resent + 1, ..s.get() }));
                    sent = Instant::now();
                }
                Next::Rebuild => continue 'build,
                Next::GiveUp => return Err(format!("{sig}: never confirmed, built {builds} times, past its last valid height each time")),
                Next::Wait => {}
            }
            std::thread::sleep(Duration::from_millis(400));
        }
    }
}

use crate::wire::Key;

// ---------------------------------------------------------------- the keys and the rules

/// The runner's two keys, read from the sitting's folder (D/keys/engine.json, D/keys/pourer.json), each held to its address beside it; and the
/// params' rules whole. Refused, every missing thing named, never another key.
pub struct Held {
    pub engine: Key,
    pub pourer: Key,
    pub rule: engine::Rule,
}

fn kept_key(dir: &Path, name: &str) -> Res<Key> {
    let f = dir.join("keys").join(format!("{name}.json"));
    let text = std::fs::read_to_string(&f).map_err(|_| format!("no {name} key at {} (press {} --dir {} makes it once)", f.display(), if name == "engine" { "engine-key" } else { "pourer" }, dir.display()))?;
    let k = Key::from_cli_text(&text)?;
    match crate::seat::kept_address(dir, name)? {
        Some(a) if a == k.address => Ok(k),
        Some(a) => Err(format!("{}: the key is {} and the address kept beside it {a}: refused", f.display(), k.address)),
        None => Err(format!("{}: no address kept beside it ({name}.address): refused", f.display())),
    }
}

pub fn held(dir: &Path, p: &Params, rpc: &Rpc) -> Res<Held> {
    let mut missing = Vec::new();
    if let Err(e) = engine::blanks(p) {
        missing.push(e);
    }
    let staker = engine::staker_named(p).ok().flatten();
    if staker.is_none() {
        missing.push("params engine.staker: the engine's key's address (the ninth act names it her pool's staker)".into());
    }
    if light::plan_cap(p).ok().flatten().is_none() {
        missing.push("params light.planCap: root 0's plan of her pool's token (the tenth act); without it no pour leaves root 0's vault".into());
    }
    let engine_key = kept_key(dir, "engine").map_err(|e| missing.push(e)).ok();
    let pourer = kept_key(dir, "pourer").map_err(|e| missing.push(e)).ok();
    if let (Some(k), Some(s)) = (&engine_key, staker) {
        if k.address != s {
            missing.push(format!("the engine's key {} is not the params' engine.staker {s}", k.address));
        }
    }
    let rule = if missing.is_empty() { engine::rule_on(p, rpc).map_err(|e| missing.push(e)).ok() } else { None };
    if !missing.is_empty() {
        return Err(format!("the runner refuses: {}", missing.join("; ")));
    }
    Ok(Held { engine: engine_key.unwrap(), pourer: pourer.unwrap(), rule: rule.unwrap() })
}

// ---------------------------------------------------------------- the book, kept

/// The runner's state beside the sitting's keys: the book after its last read (D/epoch/book.json), and the counted epochs whose pour is owed
/// (D/epoch/owed/<epoch>.json, each the book before its count and the book after it).
pub fn book_paths(dir: &Path) -> (PathBuf, PathBuf) {
    (dir.join("epoch/book.json"), dir.join("epoch/owed"))
}

/// THE OWED LIST'S BOUND: 64 counted epochs whose pour has not landed (on mainnet about 128 days; the rehearsal's 32-slot epochs, 14 minutes). Past it
/// the pass makes no new count and says so: nothing is dropped, because the book's next count spans every epoch it did not count (its law counts a gap
/// as one), so the epochs are only counted more coarsely. A list that long means the pours fail for a cause a pass cannot mend.
pub const OWED_BOUND: usize = 64;

/// One counted epoch whose pour is owed: the book before the count and the book after it, all the reader needs, kept until the chain holds its record.
#[derive(Clone, Debug)]
pub struct Owed {
    pub epoch: u64,
    pub before: Book,
    pub after: Book,
}

/// The owed epochs, oldest first.
pub fn owed_list(dir: &Path, pool: &Pubkey) -> Res<Vec<Owed>> {
    let (_, owed) = book_paths(dir);
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(&owed) else { return Ok(out) };
    for e in rd.flatten() {
        let f = e.path();
        if f.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?).map_err(|e| format!("{}: {e}", f.display()))?;
        let book = |k: &str| stake::book_of(&v[k], &pool.to_string()).map_err(|e| format!("{}: {k}: {e}", f.display()));
        let (before, after) = (book("before")?, book("after")?);
        let epoch = after.row.as_ref().map(|r| r.epoch).ok_or_else(|| format!("{}: its book after counted nothing", f.display()))?;
        out.push(Owed { epoch, before, after });
    }
    out.sort_by_key(|o| o.epoch);
    Ok(out)
}

pub fn write_owed(dir: &Path, o: &Owed) -> Res<()> {
    let (_, owed) = book_paths(dir);
    std::fs::create_dir_all(&owed).map_err(|e| format!("{}: {e}", owed.display()))?;
    let f = owed.join(format!("{}.json", o.epoch));
    let tmp = f.with_extension("json.tmp");
    let v = json!({ "schema": "embertide.owed-pour/1", "epoch": o.epoch, "before": stake::book_json(&o.before), "after": stake::book_json(&o.after) });
    std::fs::write(&tmp, serde_json::to_string_pretty(&v).unwrap() + "\n").map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &f).map_err(|e| format!("{}: {e}", f.display()))
}

pub fn remove_owed(dir: &Path, epoch: u64) -> Res<()> {
    let f = book_paths(dir).1.join(format!("{epoch}.json"));
    std::fs::remove_file(&f).map_err(|e| format!("{}: {e}", f.display()))
}

/// An owed epoch is settled when the chain's records hold its pour for every wish its reading owes (`Poured`), or a wish has no road (its light
/// stays undirected, said), or its share is nothing; and nothing is left to send. Pure.
pub fn settled(r: &light::Reading, held: &[(u64, light::Held)], pending: usize) -> bool {
    pending == 0 && r.wishes.iter().all(|(u, s)| s.tokens == 0 || held.iter().any(|(w, h)| w == u && matches!(h, light::Held::Poured | light::Held::NoWell | light::Held::Nothing)))
}

/// THE BOOK ACROSS A GAP, said by name: a count whose span is more than one epoch (the runner slept through some) is the book's one count of it,
/// attributed by what stood at both reads; and the fee no holder's share took past rounding (nobody held across, or what came and went in the gap)
/// stays in root 0's vault. None for a count of one epoch whose dust is rounding only. Pure.
pub fn gap_words(row: &stake::EpochRow) -> Option<String> {
    let span = row.epoch - row.from;
    let unattributed = row.dust > row.holders;
    if span <= 1 && !unattributed {
        return None;
    }
    let mut w = Vec::new();
    if span > 1 {
        w.push(format!("epochs {} to {} counted as one (the book's law across a gap): {} lamports of fee by what stood at both reads", row.from + 1, row.epoch, row.fee_lamports));
    }
    if unattributed {
        w.push(format!("{} lamports of epochs {} to {}'s fee no holder's share took (nobody held across, or it came and went between the reads): it stays in root 0's vault", row.dust, row.from + 1, row.epoch));
    }
    Some(w.join("; "))
}

fn book_at(f: &Path, pool: &Pubkey) -> Res<Option<Book>> {
    match std::fs::read_to_string(f) {
        Ok(t) => stake::book_of(&serde_json::from_str(&t).map_err(|e| format!("{}: {e}", f.display()))?, &pool.to_string()).map(Some).map_err(|e| format!("{}: {e}", f.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", f.display())),
    }
}

fn write_book(f: &Path, b: &Book) -> Res<()> {
    std::fs::create_dir_all(f.parent().unwrap()).map_err(|e| e.to_string())?;
    let tmp = f.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(&stake::book_json(b)).unwrap() + "\n").map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, f).map_err(|e| format!("{}: {e}", f.display()))
}

/// Her holders now, by the book's own law of holders: every token account of her mint (the owner and the amount), the fee account left out.
fn holders(rpc: &Rpc, k: &Keys) -> Res<std::collections::BTreeMap<String, u64>> {
    let filters = json!([{ "memcmp": { "offset": 0, "bytes": k.mint.to_string() } }, { "dataSize": 165 }]);
    let v = rpc.call("getProgramAccounts", json!([light::TOKEN.to_string(), { "encoding": "base64", "commitment": "confirmed", "dataSlice": { "offset": 32, "length": 40 }, "filters": filters }]))?;
    let mut rows = Vec::new();
    for a in v.as_array().cloned().unwrap_or_default() {
        let b = wire::unb64(a["account"]["data"][0].as_str().unwrap_or_default())?;
        if b.len() != 40 {
            continue;
        }
        rows.push((a["pubkey"].as_str().unwrap_or_default().to_string(), Pubkey::new_from_array(b[..32].try_into().unwrap()).to_string(), u64::from_le_bytes(b[32..40].try_into().unwrap())));
    }
    Ok(stake::holders_of(&rows, &k.vault.to_string(), &[]))
}

// ---------------------------------------------------------------- one pass

/// What one pass did, for its one line.
#[derive(Clone, Debug, Default)]
pub struct Pass {
    pub epoch: u64,
    pub waited: bool,
    pub brought: Option<u64>,
    /// lamports the engine moved onto validators (increases) and back (decreases)
    pub placed: u64,
    pub brought_back: u64,
    pub engine_fees: u64,
    pub counted: Option<Value>,
    /// (the counted epoch, the wish, the tokens), oldest epoch first
    pub poured: Vec<(u64, u64, u64)>,
    pub held: Vec<String>,
    /// the book's words across a gap, and its unattributed fee, every pass that handles such an epoch
    pub gaps: Vec<String>,
    /// the counted epochs still owed after the pass, and the bound's words when it was reached
    pub owed: Vec<u64>,
    pub full: Option<String>,
    pub over_pulled: u64,
    pub left: Option<(u64, u64)>,
    pub sends: Sends,
}

impl Pass {
    pub fn nothing(&self) -> bool {
        !self.waited && self.brought.is_none() && self.placed == 0 && self.brought_back == 0 && self.counted.is_none() && self.poured.is_empty() && self.sends.landed == 0 && self.owed.is_empty() && self.full.is_none()
    }
    pub fn line(&self) -> String {
        if self.waited {
            return format!("epoch {}: the epoch's rewards are being paid; her pool waits, nothing sent", self.epoch);
        }
        if self.nothing() {
            return format!("epoch {}: nothing due", self.epoch);
        }
        let counted = self.counted.as_ref().map_or("no count".into(), |c| if c["first"] == true { format!("the book's first read, {} holders (it counts nothing)", c["holders"]) } else { format!("counted epochs {} to {}: fee {} tokens ({} lamports), {} holders", c["from"].as_u64().unwrap_or(0) + 1, c["epoch"], c["fee_tokens"].as_str().unwrap_or("0"), c["fee_lamports"].as_str().unwrap_or("0"), c["holders"]) });
        let poured = if self.poured.is_empty() { "poured nothing".into() } else { format!("poured {}", self.poured.iter().map(|(e, u, n)| format!("{n} tokens to wish {u} for epoch {e}")).collect::<Vec<_>>().join(", ")) };
        let left = self.left.map_or(String::new(), |(u, d)| format!("; left in root 0's vault {u} undirected + {d} dust"));
        let mut held = if self.held.is_empty() { String::new() } else { format!("; held: {}", self.held.join("; ")) };
        for g in &self.gaps {
            held += &format!("; {g}");
        }
        if !self.owed.is_empty() {
            held += &format!("; still owed: epochs {:?}", self.owed);
        }
        if let Some(f) = &self.full {
            held += &format!("; {f}");
        }
        format!(
            "epoch {}: {}placed {} lamports, brought back {}, engine fees {}; {counted}; {poured}{left}{}{held}; sends {} landed, {} sent again, {} built again, {} lamports of fees",
            self.epoch,
            self.brought.map_or(String::new(), |e| format!("brought her pool from epoch {e}; ")),
            self.placed,
            self.brought_back,
            self.engine_fees,
            if self.over_pulled > 0 { format!("; over-pulled {} tokens", self.over_pulled) } else { String::new() },
            self.sends.landed,
            self.sends.resent,
            self.sends.rebuilt,
            self.sends.fees
        )
    }
}

fn moved(r: &engine::Report) -> (u64, u64) {
    let (mut up, mut down) = (0u64, 0u64);
    for s in &r.sent {
        for d in s["did"].as_array().cloned().unwrap_or_default() {
            if d.get("increase").is_some() {
                up += d["lamports"].as_u64().unwrap_or(0);
            }
            if d.get("decrease").is_some() {
                down += d["lamports"].as_u64().unwrap_or(0);
            }
        }
    }
    (up, down)
}

/// ONE PASS: the engine, the book, the pour, each when it is due; a step that fails stops the pass with its words.
pub fn pass(rpc: &Rpc, dir: &Path, pool: &Pubkey, h: &Held) -> Res<Pass> {
    let _ = sends_taken();
    let (k, chain) = engine::read(rpc, pool, &h.engine.address).map_err(|e| format!("the pass, reading her pool: {e}"))?;
    let (book_f, _) = book_paths(dir);
    let book = book_at(&book_f, pool)?;
    let d = due(chain.epoch, chain.pool_epoch, chain.rewards_paying, book.as_ref().and_then(|b| b.last.as_ref()).map(|l| l.epoch), book.as_ref().and_then(|b| b.row.as_ref()).map(|r| r.epoch));
    let mut out = Pass { epoch: chain.epoch, ..Pass::default() };
    if d.wait {
        out.waited = true;
        return Ok(out);
    }
    // 1. the engine: it brings her pool to the epoch first, then places by the rule; nothing when nothing is due
    let r = engine::run(rpc, pool, &h.rule, &h.engine.address, Some(&h.engine)).map_err(|e| format!("the pass, the engine (her pool at epoch {}, the chain's {}): {e}", chain.pool_epoch, chain.epoch))?;
    if r.said.iter().any(|w| w.contains("rewards")) && r.after["poolEpoch"].as_u64() != Some(chain.epoch) {
        out.waited = true;
        return Ok(out);
    }
    if d.bring {
        out.brought = Some(chain.pool_epoch);
    }
    (out.placed, out.brought_back) = moved(&r);
    out.engine_fees = r.fees;
    // 2. the book: one read an epoch, once her pool stands at it
    let now = door::pool_now(&rpc.account(&pool.to_string())?.ok_or("her pool is gone")?.data)?;
    let owed_now = owed_list(dir, pool)?;
    if d.count && now.last_update_epoch == chain.epoch && owed_now.len() >= OWED_BOUND {
        // THE BOUND: no new count while the list is full; the book's next count spans what it skips, so nothing is dropped
        out.full = Some(format!("the owed list is full ({OWED_BOUND} counted epochs unpoured, the oldest {}): no new count until they pour; the next count spans every epoch skipped", owed_now[0].epoch));
    } else if d.count && now.last_update_epoch == chain.epoch {
        let sched = light::schedule(rpc)?;
        let snap = Snapshot { epoch: chain.epoch, total_lamports: now.total_lamports, pool_token_supply: now.pool_token_supply, holders: holders(rpc, &k)? };
        let old = book.clone().unwrap_or_else(|| Book::new(&pool.to_string()));
        let fee = match &old.last {
            Some(l) if l.epoch < chain.epoch => light::chain_fee(rpc, &k, &sched, l.epoch + 1, chain.epoch).map_err(|e| format!("the pass, the fee's mints: {e}"))?,
            _ => 0,
        };
        let next = stake::count(&old, &snap, fee).map_err(|e| format!("the pass, the book: {e}"))?;
        // the count is owed before the book moves: written first, so a pass that falls between the two counts the epoch again, never loses it
        if old.last.is_some() {
            write_owed(dir, &Owed { epoch: chain.epoch, before: old.clone(), after: next.clone() })?;
        }
        write_book(&book_f, &next)?;
        if old.last.is_some() {
            out.counted = next.row.as_ref().map(|r| json!({ "epoch": r.epoch, "from": r.from, "fee_tokens": r.fee_tokens.to_string(), "fee_lamports": r.fee_lamports.to_string(), "holders": r.holders }));
        } else {
            out.counted = Some(json!({ "epoch": chain.epoch, "from": chain.epoch, "fee_tokens": "0", "fee_lamports": "0", "holders": snap.holders.len(), "first": true }));
        }
    }
    // 3. the pour: EVERY OWED EPOCH, oldest first, each kept until the chain's records hold its pour; a wish netted against what it already took
    let (mut undirected, mut dust) = (0u64, 0u64);
    for o in owed_list(dir, pool)? {
        let c = light::crank(rpc, pool, Some(&o.before), &o.after, &h.pourer.address).map_err(|e| format!("the pass, the reader of epoch {}: {e}", o.epoch))?;
        if let Err(e) = &c.agreed {
            return Err(format!("the pass, the pour of epoch {}: {e}", o.epoch));
        }
        if let Some(g) = o.after.row.as_ref().and_then(gap_words) {
            out.gaps.push(g);
        }
        light::send(rpc, &c, &h.pourer).map_err(|e| format!("the pass, the pour of epoch {}: {e}", o.epoch))?;
        out.poured.extend(c.pours.iter().filter(|p| p.tokens > 0).map(|p| (o.epoch, p.unit, p.tokens)));
        out.over_pulled += c.over_pulled();
        undirected += c.reading.undirected_tokens;
        dust += c.reading.dust_tokens;
        // removed only once its record is read back from the chain
        let again = light::crank(rpc, pool, Some(&o.before), &o.after, &h.pourer.address).map_err(|e| format!("the pass, the record of epoch {}: {e}", o.epoch))?;
        if settled(&again.reading, &again.held, again.pours.len()) {
            out.held.extend(again.held.iter().filter(|(_, w)| *w == light::Held::NoWell).map(|(u, w)| format!("epoch {}, wish {u}: {}", o.epoch, light::held_words(w))));
            remove_owed(dir, o.epoch)?;
        } else {
            out.held.extend(again.held.iter().filter(|(_, w)| *w != light::Held::Poured).map(|(u, w)| format!("epoch {}, wish {u}: {}", o.epoch, light::held_words(w))));
            out.owed.push(o.epoch);
        }
        out.left = Some((undirected, dust));
    }
    out.sends = sends_taken();
    Ok(out)
}

/// The chain's epoch now: its number, the slot's index in it, its length; and a slot's measured length in milliseconds (the node's own samples).
pub fn clock(rpc: &Rpc) -> Res<(u64, u64, u64, u64)> {
    let e = rpc.call("getEpochInfo", json!([{ "commitment": "confirmed" }]))?;
    let n = |k: &str| e[k].as_u64().ok_or_else(|| format!("getEpochInfo: no {k}"));
    let s = rpc.call("getRecentPerformanceSamples", json!([4]))?;
    let (slots, secs) = s.as_array().cloned().unwrap_or_default().iter().fold((0u64, 0u64), |a, x| (a.0 + x["numSlots"].as_u64().unwrap_or(0), a.1 + x["samplePeriodSecs"].as_u64().unwrap_or(0)));
    let slot_ms = if slots > 0 { (secs * 1000 / slots).clamp(50, 2_000) } else { 400 };
    Ok((n("epoch")?, n("slotIndex")?, n("slotsInEpoch")?, slot_ms))
}

/// `press epoch --dir D --params P --rpc URL --pool A [--cluster localnet|mainnet] [--stay] [--margin-slots N] [--passes N]`: one pass, or
/// (`--stay`) a pass each epoch until stopped (or `--passes` passes): one line a pass on stdout, errors on stderr.
pub fn step(opts: &std::collections::BTreeMap<String, String>, stay: bool) -> Res<()> {
    let get = |k: &str| opts.get(k).map(String::as_str).ok_or_else(|| format!("{k} is needed"));
    let cluster = crate::rpc::Cluster::parse(opts.get("--cluster").map_or("localnet", String::as_str))?;
    let rpc = Rpc::new(get("--rpc")?, cluster)?;
    rpc.check_cluster()?;
    let dir = PathBuf::from(get("--dir")?);
    let p = Params::read(Path::new(get("--params")?))?;
    let pool = Pubkey::from_str(get("--pool")?).map_err(|_| "--pool: her pool's address")?;
    let margin = opts.get("--margin-slots").map(|m| m.parse::<u64>().map_err(|_| "--margin-slots: a whole number")).transpose()?.unwrap_or(2);
    let passes = opts.get("--passes").map(|m| m.parse::<u64>().map_err(|_| "--passes: a whole number")).transpose()?;
    let h = held(&dir, &p, &rpc)?;
    let mut n = 0u64;
    let mut last_line: Option<String> = None;
    loop {
        let started = chrono_now();
        match pass(&rpc, &dir, &pool, &h) {
            Ok(r) => {
                let line = r.line();
                // a line that would repeat (a second pass in an epoch that found nothing) is said once
                if last_line.as_deref() != Some(line.as_str()) {
                    // the receipt's own cost: every read and send since the runner started, the waiting's clock reads among them
                    println!("{started} {line} ({} RPC calls since the start)", rpc.calls.borrow().values().sum::<u64>());
                }
                last_line = Some(line);
                n += 1;
                if !stay || passes.is_some_and(|m| n >= m) {
                    return Ok(());
                }
                let (epoch, index, len, slot_ms) = clock(&rpc)?;
                // waiting on the rewards: a short look again inside the same epoch, else the next epoch's first slot plus the margin
                let mut wait = if r.waited { Duration::from_millis(8 * slot_ms) } else { sleep_for(index, len, slot_ms, margin) };
                loop {
                    std::thread::sleep(wait);
                    if r.waited {
                        break;
                    }
                    let (now_epoch, i, l, ms) = clock(&rpc)?;
                    if now_epoch > epoch {
                        break;
                    }
                    wait = sleep_for(i, l, ms, margin);
                }
            }
            Err(e) => {
                eprintln!("{started} {e}");
                if !stay {
                    return Err(e);
                }
                n += 1;
                if passes.is_some_and(|m| n >= m) {
                    return Err(e);
                }
                // the next pass starts from the chain's state: a few slots on, inside the same epoch
                let (_, _, _, slot_ms) = clock(&rpc).unwrap_or((0, 0, 0, 400));
                std::thread::sleep(Duration::from_millis(16 * slot_ms));
            }
        }
    }
}

fn chrono_now() -> String {
    crate::chain::now_iso()
}
