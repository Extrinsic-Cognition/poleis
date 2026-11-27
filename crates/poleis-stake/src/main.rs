//! THE HAND of poleis-stake: it reads a Solana node and says what the law (lib.rs) makes of it. It signs nothing and sends nothing; the one
//! thing it writes is a book's file, and only with --write.
//!
//!   poleis-stake read  <pool> --rpc <url> [--epoch N]
//!       the pool as the chain holds it, and the yield minted in that epoch (the current one when none is named)
//!   poleis-stake count <pool> --rpc <url> --book <file> [--except <owner>]... [--write]
//!       one more read laid on the pool's book: who held from the last read to this one, and their share of the yield since
//!   poleis-stake row   <pool> --rpc <url> --tank <name> --sol-usd <price> [--epoch N]
//!       that epoch's yield as the one refill row the tanks' ledger takes for it (said, never written: a refill is his hand)
//!
//! The node is the caller's word (--rpc): no address of any is kept here.

use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Duration;

use poleis_stake::*;
use serde_json::{json, Value};

/// The classic token program: its accounts are all 165 bytes, so the node can filter by size.
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

struct Node {
    url: String,
    http: ureq::Agent,
}

impl Node {
    fn ask(&self, method: &str, params: Value) -> Result<Value, String> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        let mut last = String::new();
        for attempt in 0..4u64 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_secs(2 * attempt));
            }
            let sent = self.http.post(&self.url).header("content-type", "application/json").send(body.as_str());
            let text = match sent {
                Ok(mut r) => match r.body_mut().with_config().limit(256 << 20).read_to_string() {
                    Ok(t) => t,
                    Err(e) => {
                        last = format!("{method}: {e}");
                        continue;
                    }
                },
                Err(e) => {
                    last = format!("{method}: {e}");
                    continue;
                }
            };
            let v: Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(e) => {
                    last = format!("{method}: not JSON ({e})");
                    continue;
                }
            };
            if let Some(e) = v.get("error") {
                last = format!("{method}: the node refused: {e}");
                continue;
            }
            return v.get("result").cloned().ok_or_else(|| format!("{method}: no result"));
        }
        Err(last)
    }
}

/// What every verb reads first: the epoch now, the cluster's schedule, the pool and the program that owns it.
struct Chain {
    epoch: u64,
    schedule: Schedule,
    program: String,
    pool: Pool,
}

fn chain(node: &Node, pool: &str) -> Result<Chain, String> {
    let epoch = node.ask("getEpochInfo", json!([]))?.get("epoch").and_then(Value::as_u64).ok_or("getEpochInfo: no epoch")?;
    let s = node.ask("getEpochSchedule", json!([]))?;
    let num = |k: &str| s.get(k).and_then(Value::as_u64).ok_or(format!("getEpochSchedule: no {k}"));
    let schedule = Schedule { slots_per_epoch: num("slotsPerEpoch")?, first_normal_epoch: num("firstNormalEpoch")?, first_normal_slot: num("firstNormalSlot")? };
    let a = node.ask("getAccountInfo", json!([pool, { "encoding": "base64" }]))?;
    let v = a.get("value").filter(|v| !v.is_null()).ok_or(format!("no account at {pool}"))?;
    let program = v.get("owner").and_then(Value::as_str).ok_or("the account has no owner")?.to_string();
    let bytes = v.pointer("/data/0").and_then(Value::as_str).and_then(base64_of).ok_or("the account's bytes did not come as base64")?;
    Ok(Chain { epoch, schedule, program, pool: pool_of(&bytes).map_err(|e| e.to_string())? })
}

/// The pool tokens minted as the fee in every epoch from `first` to `last`, both counted: the fee account's signatures back to `first`'s first
/// slot, one transaction read for each inside the span.
fn fee_between(node: &Node, c: &Chain, pool: &str, first: u64, last: u64) -> Result<(u64, usize), String> {
    let fee_account = address(&c.pool.manager_fee_account);
    let (from, _) = c.schedule.slots_of(first).map_err(|e| e.to_string())?;
    let (_, to) = c.schedule.slots_of(last).map_err(|e| e.to_string())?;
    let (mut tokens, mut read, mut before): (u64, usize, Option<String>) = (0, 0, None);
    loop {
        let mut opts = json!({ "limit": 1000 });
        if let Some(b) = &before {
            opts["before"] = json!(b);
        }
        let page = node.ask("getSignaturesForAddress", json!([fee_account, opts]))?;
        let page = page.as_array().ok_or("getSignaturesForAddress: not a list")?;
        let mut done = page.len() < 1000;
        for s in page {
            let slot = s.get("slot").and_then(Value::as_u64).ok_or("a signature without its slot")?;
            let sig = s.get("signature").and_then(Value::as_str).ok_or("a signature without itself")?;
            before = Some(sig.to_string());
            if slot < from {
                done = true;
                break;
            }
            if slot > to || !s.get("err").map_or(true, Value::is_null) {
                continue;
            }
            let tx = node.ask("getTransaction", json!([sig, { "encoding": "jsonParsed", "maxSupportedTransactionVersion": 0 }]))?;
            read += 1;
            for m in fee_mints(&tx, &c.program, pool, &fee_account) {
                tokens = tokens.saturating_add(m.tokens);
            }
        }
        if done {
            return Ok((tokens, read));
        }
    }
}

/// Every token account of the pool's mint: (the account, its owner, its amount). The node sends 40 bytes of each.
fn token_accounts(node: &Node, c: &Chain) -> Result<Vec<(String, String, u64)>, String> {
    let program = address(&c.pool.token_program);
    let mut filters = vec![json!({ "memcmp": { "offset": 0, "bytes": address(&c.pool.pool_mint) } })];
    if program == TOKEN_PROGRAM {
        filters.push(json!({ "dataSize": 165 }));
    }
    let r = node.ask("getProgramAccounts", json!([program, { "encoding": "base64", "dataSlice": { "offset": 32, "length": 40 }, "filters": filters }]))?;
    let mut out = Vec::new();
    for a in r.as_array().ok_or("getProgramAccounts: not a list")? {
        let account = a.get("pubkey").and_then(Value::as_str).ok_or("a token account without its address")?;
        let b = a.pointer("/account/data/0").and_then(Value::as_str).and_then(base64_of).filter(|b| b.len() == 40).ok_or("a token account's 40 bytes did not come")?;
        let owner = bs58::encode(&b[0..32]).into_string();
        let mut amount = [0u8; 8];
        amount.copy_from_slice(&b[32..40]);
        out.push((account.to_string(), owner, u64::from_le_bytes(amount)));
    }
    Ok(out)
}

fn pool_json(pool: &str, c: &Chain) -> Value {
    let p = &c.pool;
    json!({
        "pool": pool, "program": c.program, "epoch_now": c.epoch,
        "manager": address(&p.manager), "staker": address(&p.staker), "mint": address(&p.pool_mint), "fee_account": address(&p.manager_fee_account),
        "total_lamports": p.total_lamports.to_string(), "pool_token_supply": p.pool_token_supply.to_string(),
        "last_update_epoch": p.last_update_epoch, "updated_this_epoch": p.last_update_epoch == c.epoch,
        "epoch_fee": format!("{}/{}", p.epoch_fee.numerator, p.epoch_fee.denominator), "the_whole_yield": p.epoch_fee.is_whole(),
    })
}

struct Args {
    verb: String,
    pool: String,
    flags: BTreeMap<String, Vec<String>>,
}

fn args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let verb = it.next().ok_or("say read, count or row")?;
    let pool = it.next().filter(|p| key_of(p).is_some()).ok_or("the pool's address comes second")?;
    let mut flags: BTreeMap<String, Vec<String>> = BTreeMap::new();
    while let Some(a) = it.next() {
        let name = a.strip_prefix("--").ok_or(format!("{a}: not a flag"))?.to_string();
        let value = if name == "write" { String::new() } else { it.next().ok_or(format!("--{name} wants a value"))? };
        flags.entry(name).or_default().push(value);
    }
    Ok(Args { verb, pool, flags })
}

fn run() -> Result<Value, String> {
    let a = args()?;
    let one = |k: &str| a.flags.get(k).and_then(|v| v.last()).cloned();
    let _ = rustls::crypto::ring::default_provider().install_default();
    let http: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(90))).build().into();
    let node = Node { url: one("rpc").ok_or("--rpc <url>: the node to read is the caller's word")?, http };
    let c = chain(&node, &a.pool)?;
    let epoch_asked = match one("epoch") {
        Some(e) => e.parse::<u64>().map_err(|_| "--epoch wants a whole number")?,
        None => c.epoch,
    };
    match a.verb.as_str() {
        "read" => {
            let (tokens, read) = fee_between(&node, &c, &a.pool, epoch_asked, epoch_asked)?;
            let lamports = lamports_of(tokens, c.pool.total_lamports, c.pool.pool_token_supply);
            let mut out = pool_json(&a.pool, &c);
            out["yield"] = json!({ "epoch": epoch_asked, "fee_tokens": tokens.to_string(), "fee_lamports": lamports.to_string(), "transactions_read": read });
            Ok(out)
        }
        "row" => {
            let tank = one("tank").ok_or("--tank <name>: whose tank the pool's yield fills")?;
            let price = one("sol-usd").and_then(|p| p.parse::<f64>().ok()).ok_or("--sol-usd <price>: the price of SOL is the caller's word")?;
            let (tokens, _) = fee_between(&node, &c, &a.pool, epoch_asked, epoch_asked)?;
            let lamports = lamports_of(tokens, c.pool.total_lamports, c.pool.pool_token_supply);
            refill_row(epoch_asked, &a.pool, &tank, lamports, price).map_err(|e| e.to_string())
        }
        "count" => {
            let file = one("book").ok_or("--book <file>: where this pool's book is kept")?;
            let book = match std::fs::read_to_string(&file) {
                Ok(t) => book_of(&serde_json::from_str(&t).map_err(|e| format!("{file}: not JSON ({e})"))?, &a.pool).map_err(|e| format!("{file}: {e}"))?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Book::new(&a.pool),
                Err(e) => return Err(format!("{file}: {e}")),
            };
            if c.pool.last_update_epoch != c.epoch {
                return Err(format!("the pool was last updated in epoch {}, and this is epoch {}: its fee for this epoch is not minted yet (anyone may send the pool's update); nothing was counted", c.pool.last_update_epoch, c.epoch));
            }
            let except = a.flags.get("except").cloned().unwrap_or_default();
            let holders = holders_of(&token_accounts(&node, &c)?, &address(&c.pool.manager_fee_account), &except);
            let snap = Snapshot { epoch: c.epoch, total_lamports: c.pool.total_lamports, pool_token_supply: c.pool.pool_token_supply, holders };
            let (fee_tokens, read) = match &book.last {
                Some(last) if last.epoch < c.epoch => fee_between(&node, &c, &a.pool, last.epoch + 1, c.epoch)?,
                _ => (0, 0),
            };
            let first = book.last.is_none();
            let next = count(&book, &snap, fee_tokens).map_err(|e| e.to_string())?;
            let wrote = a.flags.contains_key("write");
            if wrote {
                let tmp = format!("{file}.{}.tmp", std::process::id());
                std::fs::write(&tmp, serde_json::to_string_pretty(&book_json(&next)).map_err(|e| e.to_string())? + "\n").map_err(|e| format!("{tmp}: {e}"))?;
                std::fs::rename(&tmp, &file).map_err(|e| format!("{file}: {e}"))?;
            }
            let row = next.row.as_ref().filter(|_| !first).map(|e| {
                json!({ "epoch": e.epoch, "from": e.from, "fee_tokens": e.fee_tokens.to_string(), "fee_lamports": e.fee_lamports.to_string(),
                    "held_lamports": e.held_lamports.to_string(), "dust": e.dust.to_string(), "holders": e.holders })
            });
            Ok(json!({
                "pool": a.pool, "epoch": c.epoch, "holders_now": snap.holders.len(), "first_read": first, "counted": row,
                "transactions_read": read, "stakers_in_book": next.stakers.len(), "epochs_counted": next.counted, "book": file, "wrote": wrote,
            }))
        }
        other => Err(format!("{other}: say read, count or row")),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(v) => {
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("poleis-stake: {e}");
            ExitCode::from(2)
        }
    }
}
