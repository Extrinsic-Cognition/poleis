//! THE MARKET ON THE NEST (Nov 11 2026): a road is a ref, a trade is a cause signed by its trader's own key, the decision is a ref any reader can
//! fold again from the same causes. This file reads and writes the Nest and checks every trade's signature itself (the Nest's own body of an entry,
//! tools/embertide-mesh/lib/nest-author.mjs bodyOf): a reader that acts on a signed word checks the signer, never a fold.
//!
//!   futarchy/proposals/<id>      the road, its window on the Nest's clock and its market's terms, and optionally that it can be bet both ways, the
//!                                judge whose signed word is its receipts, and its roster: `deposits`, who trades it and what each deposits (laid
//!                                once, by compare-and-set)
//!   futarchy.trade               a trade: {proposal, branch, side, amount, min_out}, integers of a millionth of a $WARM
//!   futarchy/decisions/<id>      after the close: passed or failed, the two averages, the counts, each trader's paper result (written once)
//!   futarchy/allowances/<month>  for a road of allowances that passed: its grants (the chairs' seal reads this: futarchy/seal-follows.mjs)
//!   futarchy.receipts            the judge's word after the close: {proposal, world, worth}, what a claim was worth in the world that happened, signed
//!                                by the road's judge (the first such word in the Nest's order stands; a word stamped before the close or naming
//!                                the world that did not happen counts for nothing, and the judge's own door refuses to send one)
//!   futarchy/counts/<id>         each trader's result settled on the receipts, its deposit and what the road gives back (the deposit and the
//!                                result), and the pool's (written once)
//!
//! COST: a trade is one signed entry; a reading asks the Nest for the road and its clock (one GET) and for the trades after the road's own seq, by
//! type (one GET a 500), and folds them: a millisecond a thousand. A roster rides in the road's own ref, in the order of its keys: no extra read,
//! one pass to check it, and a trader's deposit found by halving. A count is the same reading as a settle plus one GET a 500 of the receipts' type;
//! the judge's word is one reading and one signed entry. Nothing runs at rest: no timer, no poll.
use crate::law::*;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::{json, Value};
use std::io::{Read, Write};

pub const SCHEMA: &str = "embertide.futarchy/2";
pub const TRADE: &str = "futarchy.trade";
pub const PROPOSALS: &str = "futarchy/proposals/";
pub const DECISIONS: &str = "futarchy/decisions/";
pub const ALLOWANCES: &str = "futarchy/allowances/";
pub const RECEIPTS: &str = "futarchy.receipts";
pub const COUNTS: &str = "futarchy/counts/";
const ED_SPKI: &str = "302a300506032b6570032100";
const ED_PKCS8: [u8; 16] = [0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant { pub seat: String, pub address: String, pub cap: u64 }
/// what a road decides: a month's allowances to agents (the grants are written when it passes), or a road in words
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind { Allowances { month: String, grants: Vec<Grant> }, Road { what: String } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Road { pub id: String, pub kind: Kind, pub by: Option<String>, pub opens_seq: u64, pub terms: Terms, pub judge: Option<String>,
    /// who trades the road and what each deposits, sorted by key; empty when the road names none (every signer then deposits `budget`)
    pub roster: Vec<Deposit> }

fn is_id(s: &str) -> bool { let b = s.as_bytes(); !b.is_empty() && b.len() <= 64 && b[0] != b'-' && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-') }
fn is_seat(s: &str) -> bool { is_id(s) && s.len() <= 32 && s.as_bytes()[0].is_ascii_lowercase() }
fn is_month(s: &str) -> bool { let b = s.as_bytes(); b.len() == 7 && b[4] == b'-' && b[..4].iter().all(u8::is_ascii_digit) && matches!(&s[5..], "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10" | "11" | "12") }
fn is_address(s: &str) -> bool { (32..=44).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_alphanumeric() && !b"0OIl".contains(&c)) }
fn is_key(s: &str) -> bool { s.len() == 88 && s.starts_with(ED_SPKI) && s.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) }

/// road_of(value) -> the road with its defaults, or None: the terms are the law's (Terms::check), every amount an integer of micro; `both_ways`,
/// when named, is true or false; a judge, when named, is an Ed25519 key in hex as the Nest keeps keys; `deposits`, when named, is an object from
/// such a key to an unsigned integer of micro $WARM, a roster the law can carry (roster_check)
pub fn road_of(v: &Value) -> Option<Road> {
    if v.get("schema")?.as_str()? != SCHEMA { return None; }
    let id = v.get("id")?.as_str().filter(|s| is_id(s))?.to_string();
    let kind = match v.get("kind")?.as_str()? {
        "allowances" => {
            let month = v.get("month")?.as_str().filter(|s| is_month(s))?.to_string();
            let grants: Vec<Grant> = v.get("grants")?.as_array()?.iter().map(|g| Some(Grant { seat: g.get("seat")?.as_str().filter(|s| is_seat(s))?.into(), address: g.get("address")?.as_str().filter(|s| is_address(s))?.into(), cap: g.get("cap")?.as_u64().filter(|c| *c > 0)? })).collect::<Option<_>>()?;
            if grants.is_empty() || (1..grants.len()).any(|i| grants[..i].iter().any(|g| g.seat == grants[i].seat)) { return None; }
            Kind::Allowances { month, grants }
        }
        "road" => Kind::Road { what: v.get("what")?.as_str().filter(|s| !s.trim().is_empty() && s.len() <= 400)?.to_string() },
        _ => return None,
    };
    let n = |k: &str, d: u64| v.get(k).and_then(Value::as_u64).unwrap_or(d);
    let pool = |k: &str| v.get("pool").and_then(|p| p.get(k)).and_then(Value::as_u64).unwrap_or(1000 * UNIT as u64) as u128;
    let both_ways = match v.get("both_ways") { None | Some(Value::Null) => false, Some(b) => b.as_bool()? };
    // the fee the pool keeps of every trade: a whole number of bp under 10000; a road that names none keeps none
    let fee_bp = match v.get("fee_bp") { None | Some(Value::Null) => 0, Some(f) => f.as_u64().filter(|f| (*f as u128) < BP)? as u32 };
    let terms = Terms { opens_ms: v.get("opens_ms")?.as_u64()?, closes_ms: v.get("closes_ms")?.as_u64()?, step_ms: n("step_ms", 60_000), max_step_bp: n("max_step_bp", 100).min(u32::MAX as u64) as u32,
        threshold_bp: n("threshold_bp", 300).min(u32::MAX as u64) as u32, pool_base: pool("base"), pool_quote: pool("quote"), budget: n("budget", 1000 * UNIT as u64) as u128, both_ways, fee_bp };
    terms.check().ok()?;
    let judge = match v.get("judge") { None | Some(Value::Null) => None, Some(j) => Some(j.as_str().filter(|s| is_key(s))?.to_string()) };
    let mut roster = match v.get("deposits") {
        None | Some(Value::Null) => Vec::new(),
        Some(d) => d.as_object()?.iter().map(|(k, a)| Some(Deposit { key: Some(k.as_str()).filter(|s| is_key(s))?.to_string(), amount: a.as_u64()? as u128 })).collect::<Option<Vec<_>>>()?,
    };
    roster.sort_by(|a, b| a.key.cmp(&b.key));
    roster_check(&terms, &roster).ok()?;
    Some(Road { id, kind, by: v.get("by").and_then(Value::as_str).map(String::from), opens_seq: n("opens_seq", 0), terms, judge, roster })
}

/// the road as its ref holds it
pub fn road_value(r: &Road) -> Value {
    let t = &r.terms;
    let mut v = json!({ "schema": SCHEMA, "id": r.id, "by": r.by, "opens_ms": t.opens_ms, "closes_ms": t.closes_ms, "opens_seq": r.opens_seq, "step_ms": t.step_ms, "max_step_bp": t.max_step_bp,
        "threshold_bp": t.threshold_bp, "pool": { "base": t.pool_base as u64, "quote": t.pool_quote as u64 }, "budget": t.budget as u64 });
    if t.both_ways { v["both_ways"] = json!(true); }
    if t.fee_bp > 0 { v["fee_bp"] = json!(t.fee_bp); }
    if let Some(j) = &r.judge { v["judge"] = json!(j); }
    if !r.roster.is_empty() { v["deposits"] = Value::Object(r.roster.iter().map(|d| (d.key.clone(), json!(d.amount as u64))).collect()); }
    match &r.kind {
        Kind::Allowances { month, grants } => { v["kind"] = json!("allowances"); v["month"] = json!(month); v["grants"] = grants.iter().map(|g| json!({ "seat": g.seat, "address": g.address, "cap": g.cap })).collect(); }
        Kind::Road { what } => { v["kind"] = json!("road"); v["what"] = json!(what); }
    }
    v
}

/// trade_of(payload, the road's id) -> the trade, or None
pub fn trade_of(p: &Value, id: &str) -> Option<Trade> {
    if p.get("proposal")?.as_str()? != id { return None; }
    let branch = match p.get("branch")?.as_str()? { "pass" => Which::Pass, "fail" => Which::Fail, _ => return None };
    let side = match p.get("side")?.as_str()? { "buy" => Side::Buy, "sell" => Side::Sell, _ => return None };
    Some(Trade { branch, side, amount: p.get("amount")?.as_u64().filter(|a| *a > 0)? as u128, min_out: p.get("min_out").and_then(Value::as_u64).unwrap_or(0) as u128 })
}
pub fn trade_value(id: &str, t: &Trade) -> Value {
    json!({ "proposal": id, "branch": if t.branch == Which::Pass { "pass" } else { "fail" }, "side": if t.side == Side::Buy { "buy" } else { "sell" }, "amount": t.amount as u64, "min_out": t.min_out as u64 })
}

/// receipts_of(entry, the road) -> the judge's receipts, or None: a futarchy.receipts entry whose signature holds under the road's judge, naming this
/// road, a world and a worth of zero to one $WARM, stamped by the Nest at or after the close. A road with no judge has no receipts.
pub fn receipts_of(e: &Value, r: &Road) -> Option<Receipts> {
    if e.get("type")?.as_str()? != RECEIPTS || !verified(e) { return None; }
    if e.get("key")?.as_str()? != r.judge.as_deref()? { return None; }
    if e.get("ts")?.as_u64()? < r.terms.closes_ms { return None; }
    let p = e.get("payload")?;
    if p.get("proposal")?.as_str()? != r.id { return None; }
    let world = match p.get("world")?.as_str()? { "pass" => Which::Pass, "fail" => Which::Fail, _ => return None };
    Some(Receipts { world, worth: p.get("worth")?.as_u64().filter(|w| *w as u128 <= PRICE)? as u128 })
}
pub fn receipts_value(id: &str, r: &Receipts) -> Value {
    json!({ "proposal": id, "world": if r.world == Which::Pass { "pass" } else { "fail" }, "worth": r.worth as u64 })
}
/// receipts_among(entries in the Nest's order, the road, the world that happened) -> the road's receipts: the first entry receipts_of takes that
/// names the world that happened. A word of the other world counts for nothing, as a word before the close does: it cannot stand in the way of the
/// word a count waits for. A later word changes nothing.
pub fn receipts_among<'a>(entries: impl IntoIterator<Item = &'a Value>, r: &Road, world: Which) -> Option<Receipts> {
    entries.into_iter().filter_map(|e| receipts_of(e, r)).find(|x| x.world == world)
}

/// the Nest's canonical text of a value: every object's keys sorted, no space (nest-author.mjs canonical: JSON.stringify with the sorted keys)
pub fn canonical(v: &Value) -> String {
    match v {
        Value::Object(m) => { let mut keys: Vec<&String> = m.keys().collect(); keys.sort(); format!("{{{}}}", keys.iter().map(|k| format!("{}:{}", Value::String((*k).clone()), canonical(&m[*k]))).collect::<Vec<_>>().join(",")) }
        Value::Array(a) => format!("[{}]", a.iter().map(canonical).collect::<Vec<_>>().join(",")),
        other => other.to_string(),
    }
}
/// the bytes an author signs: the entry without the fields the Nest assigns (nest-author.mjs bodyOf)
pub fn body_of(e: &Value) -> String {
    let or = |k: &str, d: Value| e.get(k).filter(|v| !v.is_null()).cloned().unwrap_or(d);
    canonical(&json!({ "seat": or("seat", Value::Null), "type": or("type", Value::Null), "payload": or("payload", Value::Null), "cause": or("cause", json!([])), "blobs": or("blobs", json!([])), "key": or("key", Value::Null), "t": or("t", Value::Null) }))
}
fn unhex(s: &str) -> Option<Vec<u8>> { if s.len() % 2 != 0 { return None; } (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect() }
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

/// verified(entry) -> its Ed25519 signature holds over the Nest's body of it, under the key it names (an SPKI in hex, as the Nest keeps keys)
pub fn verified(e: &Value) -> bool {
    let (Some(key), Some(sig)) = (e.get("key").and_then(Value::as_str), e.get("sig").and_then(Value::as_str)) else { return false };
    let Some(raw) = key.strip_prefix(ED_SPKI).and_then(unhex).and_then(|b| <[u8; 32]>::try_from(b).ok()) else { return false };
    let Some(sig) = unhex(sig).and_then(|b| <[u8; 64]>::try_from(b).ok()) else { return false };
    VerifyingKey::from_bytes(&raw).map(|k| k.verify(body_of(e).as_bytes(), &Signature::from_bytes(&sig)).is_ok()).unwrap_or(false)
}

/// one entry of the Nest's log as the law reads it: the Nest's own seq and stamp, the signer, whether the signature holds, the trade if it is one
pub fn entry_of(e: &Value, id: &str) -> Entry {
    let s = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let is_trade = e.get("type").and_then(Value::as_str) == Some(TRADE);
    Entry { seq: e.get("seq").and_then(Value::as_u64).unwrap_or(0), ts: e.get("ts").and_then(Value::as_u64).unwrap_or(0), key: s("key"), seat: s("seat"), ok: verified(e),
        trade: if is_trade { e.get("payload").and_then(|p| trade_of(p, id)) } else { None } }
}

/// a seat's key as `nest.mjs keygen` wrote it: {seat, key: the SPKI in hex, private_pem}
pub struct Key { pub seat: String, pub key: String, signing: SigningKey }
impl Key {
    pub fn from_seed(seat: &str, seed: [u8; 32]) -> Key { let signing = SigningKey::from_bytes(&seed); Key { seat: seat.into(), key: format!("{ED_SPKI}{}", hex(signing.verifying_key().as_bytes())), signing } }
    /// the key file of a seat in `dir` (NEST_KEYS, else ~/.embertide/nest)
    pub fn load(dir: &std::path::Path, seat: &str) -> Result<Key, String> {
        let f = dir.join(format!("{seat}.key.json"));
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?).map_err(|e| format!("{}: {e}", f.display()))?;
        let pem = v.get("private_pem").and_then(Value::as_str).ok_or("the key file has no private_pem")?;
        let der = unbase64(&pem.lines().filter(|l| !l.starts_with("-----")).collect::<String>()).ok_or("the private_pem is not base64")?;
        if der.len() != 48 || der[..16] != ED_PKCS8 { return Err("the private_pem is not an Ed25519 key".into()); }
        let key = Key::from_seed(seat, der[16..].try_into().unwrap());
        if v.get("key").and_then(Value::as_str) != Some(key.key.as_str()) { return Err("the key file's public key is not its private key's".into()); }
        Ok(key)
    }
    /// the entry signed: its key named, its signature over the Nest's body of it
    pub fn sign(&self, mut e: Value) -> Value {
        e["seat"] = json!(self.seat); e["key"] = json!(self.key);
        e["sig"] = json!(hex(&self.signing.sign(body_of(&e).as_bytes()).to_bytes()));
        e
    }
}
fn unbase64(s: &str) -> Option<Vec<u8>> {
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0);
    for c in s.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=') {
        let v = match c { b'A'..=b'Z' => c - b'A', b'a'..=b'z' => c - b'a' + 26, b'0'..=b'9' => c - b'0' + 52, b'+' => 62, b'/' => 63, _ => return None } as u32;
        acc = (acc << 6) | v; bits += 6;
        if bits >= 8 { bits -= 8; out.push((acc >> bits) as u8); acc &= (1 << bits) - 1; }
    }
    Some(out)
}

/// a price as words: twelve decimals, exact (a JSON number would lose them past 2^53)
pub fn price_text(p: u128) -> String { format!("{}.{:012}", p / PRICE, p % PRICE) }

/// the decision as its ref holds it
pub fn decision_value(r: &Road, m: &Market, d: &Decision, scores: &[Score]) -> Value {
    let p = |x: Option<u128>| x.map(price_text).map(Value::String).unwrap_or(Value::Null);
    json!({ "schema": SCHEMA, "id": r.id, "state": match d.state { State::Open => "open", State::Passed => "passed", State::Failed => "failed" },
        "twap": { "pass": p(d.twap_pass), "fail": p(d.twap_fail) }, "spot": { "pass": price_text(d.spot_pass), "fail": price_text(d.spot_fail) }, "needs": p(d.needs),
        "threshold_bp": r.terms.threshold_bp, "closes_ms": r.terms.closes_ms, "accepted": m.accepted, "refused": m.refused.len(),
        "scores": scores.iter().map(|s| json!({ "seat": s.seat, "trades": s.trades, "result": s.result as i64 })).collect::<Vec<_>>() })
}

/// the month's allowances with a passed road's grants in (a seat granted twice keeps the later road's cap), sorted by seat
pub fn merge_allowances(cur: Option<&Value>, month: &str, grants: &[Grant], road: &str, at: u64) -> Value {
    let mut by: std::collections::BTreeMap<String, Value> = cur.and_then(|c| c.get("grants")).and_then(Value::as_array).into_iter().flatten().filter_map(|g| Some((g.get("seat")?.as_str()?.to_string(), g.clone()))).collect();
    for g in grants { by.insert(g.seat.clone(), json!({ "seat": g.seat, "address": g.address, "cap": g.cap, "proposal": road, "at": at })); }
    json!({ "schema": SCHEMA, "month": month, "grants": by.into_values().collect::<Vec<_>>() })
}

/// a Nest by its door, plain HTTP on this Mac or the LAN: host:port
pub struct Nest { pub host: String }
impl Nest {
    pub fn new(url: &str) -> Nest { Nest { host: url.trim_start_matches("http://").trim_end_matches('/').to_string() } }
    fn ask(&self, method: &str, path: &str, body: Option<&Value>) -> Result<(u16, Value), String> {
        let mut s = std::net::TcpStream::connect(&self.host).map_err(|e| format!("{}: {e}", self.host))?;
        s.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();
        let text = body.map(Value::to_string).unwrap_or_default();
        write!(s, "{method} {path} HTTP/1.1\r\nhost: {}\r\nconnection: close\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{text}", self.host, text.len()).map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        s.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        let cut = raw.windows(4).position(|w| w == b"\r\n\r\n").ok_or("no answer")?;
        let head = String::from_utf8_lossy(&raw[..cut]).to_lowercase();
        let code: u16 = head.split_whitespace().nth(1).and_then(|c| c.parse().ok()).ok_or("no status")?;
        let mut body = raw[cut + 4..].to_vec();
        if head.contains("transfer-encoding: chunked") {
            let (mut out, mut i) = (Vec::new(), 0);
            while let Some(n) = body[i..].windows(2).position(|w| w == b"\r\n") {
                let size = usize::from_str_radix(String::from_utf8_lossy(&body[i..i + n]).trim(), 16).map_err(|e| e.to_string())?;
                if size == 0 || i + n + 2 + size > body.len() { break; }
                out.extend_from_slice(&body[i + n + 2..i + n + 2 + size]); i += n + 2 + size + 2;
            }
            body = out;
        }
        Ok((code, serde_json::from_slice(&body).unwrap_or(Value::Null)))
    }
    fn get(&self, path: &str) -> Result<Value, String> { match self.ask("GET", path, None)? { (200, v) => Ok(v), (c, v) => Err(format!("GET {path}: {c} {}", v.get("error").and_then(Value::as_str).unwrap_or(""))) } }

    /// a ref's value and revision, and the Nest's own clock at that read
    pub fn read(&self, name: &str) -> Result<(Option<Value>, u64, u64), String> {
        let v = self.get(&format!("/nest/refs?prefix={name}"))?;
        let r = v.get("refs").and_then(|r| r.get(name));
        Ok((r.and_then(|r| r.get("value")).filter(|x| !x.is_null()).cloned(), r.and_then(|r| r.get("rev")).and_then(Value::as_u64).unwrap_or(0), v.get("server_ms").and_then(Value::as_u64).unwrap_or(0)))
    }
    pub fn seq(&self) -> Result<u64, String> { Ok(self.get("/nest/refs?prefix=futarchy/none")?.get("seq").and_then(Value::as_u64).unwrap_or(0)) }
    /// move a ref only if it is still at `expect`: Ok(true) when this hand's value went in, Ok(false) when another's stood there first
    pub fn cas(&self, key: &Key, name: &str, value: &Value, expect: u64) -> Result<bool, String> {
        let signed = key.sign(json!({ "type": "ref", "payload": { "name": name, "value": value, "rev": expect + 1 }, "cause": [], "blobs": [], "t": null }));
        match self.ask("POST", &format!("/nest/ref/{name}"), Some(&json!({ "value": value, "expect": expect, "seat": key.seat, "key": signed["key"], "sig": signed["sig"], "cause": [] })))? {
            (200, _) => Ok(true), (409, _) => Ok(false), (c, v) => Err(format!("{name}: {c} {}", v.get("error").and_then(Value::as_str).unwrap_or(""))),
        }
    }
    /// one signed cause onto the log; the entry as the Nest kept it. It carries its author's own instant (`t`, which the law never reads): the Nest
    /// keeps one entry for two that are the same to the byte, and Ed25519 signs the same words the same way, so a trader's second identical trade
    /// would otherwise be the first again.
    pub fn append(&self, key: Option<&Key>, seat: &str, kind: &str, payload: Value) -> Result<Value, String> {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_micros() as u64).unwrap_or(0);
        let e = json!({ "seat": seat, "type": kind, "payload": payload, "cause": [], "blobs": [], "t": t });
        match self.ask("POST", "/nest/append", Some(&key.map(|k| k.sign(e.clone())).unwrap_or(e)))? { (200, v) => Ok(v.get("entry").cloned().unwrap_or(v)), (c, v) => Err(format!("append: {c} {}", v.get("error").and_then(Value::as_str).unwrap_or(""))) }
    }
    /// the road's trade entries after its own seq, in the Nest's order, each read by the law's eyes (its signature checked here)
    pub fn trades(&self, r: &Road) -> Result<Vec<Entry>, String> {
        let (mut out, mut after) = (Vec::new(), r.opens_seq);
        loop {
            let v = self.get(&format!("/nest/log?after={after}&limit=500&types={TRADE}"))?;
            let es = v.get("entries").and_then(Value::as_array).cloned().unwrap_or_default();
            out.extend(es.iter().filter(|e| e.get("payload").and_then(|p| p.get("proposal")).and_then(Value::as_str) == Some(r.id.as_str())).map(|e| entry_of(e, &r.id)));
            match es.last().and_then(|e| e.get("seq")).and_then(Value::as_u64) { Some(s) if es.len() >= 500 && s > after => after = s, _ => return Ok(out) }
        }
    }
    /// the road's receipts of the world that happened: the first entry after its own seq, in the Nest's order, that receipts_among takes. A later
    /// word of the judge changes nothing.
    pub fn receipts(&self, r: &Road, world: Which) -> Result<Option<Receipts>, String> {
        let mut after = r.opens_seq;
        loop {
            let v = self.get(&format!("/nest/log?after={after}&limit=500&types={RECEIPTS}"))?;
            let es = v.get("entries").and_then(Value::as_array).cloned().unwrap_or_default();
            if let Some(x) = receipts_among(&es, r, world) { return Ok(Some(x)); }
            match es.last().and_then(|e| e.get("seq")).and_then(Value::as_u64) { Some(s) if es.len() >= 500 && s > after => after = s, _ => return Ok(None) }
        }
    }
}

/// a market read off the Nest at the Nest's own instant
pub struct Reading { pub road: Road, pub market: Market, pub decision: Decision, pub scores: Vec<Score>, pub now: u64 }
pub fn read_market(nest: &Nest, id: &str) -> Result<Option<Reading>, String> {
    let (v, _, now) = nest.read(&format!("{PROPOSALS}{id}"))?;
    let Some(road) = v.as_ref().and_then(road_of) else { return Ok(None) };
    let market = market_with(&road.terms, &road.roster, &nest.trades(&road)?, now);
    let decision = decide(&road.terms, &market, now);
    let scores = score_of(&road.terms, &market, &decision);
    Ok(Some(Reading { road, market, decision, scores, now }))
}

/// lay a road, once: its opening seq is the Nest's head when it names none. Ok(false): a road of that id already stands.
pub fn propose(nest: &Nest, key: &Key, mut v: Value) -> Result<bool, String> {
    if v.get("opens_seq").and_then(Value::as_u64).is_none() { v["opens_seq"] = json!(nest.seq()?); }
    if v.get("by").is_none() { v["by"] = json!(key.seat); }
    let road = road_of(&v).ok_or("not a road the law can carry (nest.rs road_of, law.rs Terms::check)")?;
    nest.cas(key, &format!("{PROPOSALS}{}", road.id), &road_value(&road), 0)
}

/// one trade, signed by its trader's own key
pub fn trade(nest: &Nest, key: &Key, id: &str, t: &Trade) -> Result<Value, String> { nest.append(Some(key), &key.seat, TRADE, trade_value(id, t)) }

/// after the close, by the Nest's clock: the decision once (a second settle reads the first) and, for a road of allowances that passed, its grants
/// into the month's. Ok(None) while the road is open.
pub fn settle(nest: &Nest, key: &Key, id: &str) -> Result<Option<Value>, String> {
    let r = read_market(nest, id)?.ok_or(format!("no road {id}"))?;
    if r.decision.state == State::Open { return Ok(None); }
    let name = format!("{DECISIONS}{id}");
    nest.cas(key, &name, &decision_value(&r.road, &r.market, &r.decision, &r.scores), 0)?;
    let stood = nest.read(&name)?.0.ok_or("the decision did not stand")?;
    if let (Kind::Allowances { month, grants }, Some("passed")) = (&r.road.kind, stood.get("state").and_then(Value::as_str)) {
        for _ in 0..3 {
            let (cur, rev, now) = nest.read(&format!("{ALLOWANCES}{month}"))?;
            if cur.as_ref().and_then(|c| c.get("grants")).and_then(Value::as_array).map(|g| g.iter().any(|x| x.get("proposal").and_then(Value::as_str) == Some(id))).unwrap_or(false) { break; }
            if nest.cas(key, &format!("{ALLOWANCES}{month}"), &merge_allowances(cur.as_ref(), month, grants, id, now), rev)? { break; }
        }
    }
    Ok(Some(stood))
}

/// after the close and the judge's receipts: each trader's result settled on them, its deposit and what the road gives back (`back_of`), and the
/// pool's, written once (a second count reads the first). Ok(None) while the road is open or while no receipts stand.
pub fn count(nest: &Nest, key: &Key, id: &str) -> Result<Option<Value>, String> {
    let r = read_market(nest, id)?.ok_or(format!("no road {id}"))?;
    let Some(world) = world_of(&r.decision) else { return Ok(None) };
    let Some(receipts) = nest.receipts(&r.road, world)? else { return Ok(None) };
    let t = &r.road.terms;
    let counts = settle_of(t, &r.market, &r.decision, &receipts).map_err(String::from)?;
    let pool = pool_result(t, &r.market, &r.decision, &receipts).map_err(String::from)?;
    let deposits: std::collections::BTreeMap<&str, u128> = r.market.traders.iter().map(|h| (h.key.as_str(), h.deposit)).collect();
    let deposit = |key: &str| deposits.get(key).copied().unwrap_or(0);
    let v = json!({ "schema": SCHEMA, "id": r.road.id, "world": if receipts.world == Which::Pass { "pass" } else { "fail" }, "worth": price_text(receipts.worth), "judge": r.road.judge,
        "counts": counts.iter().map(|s| json!({ "seat": s.seat, "key": s.key, "trades": s.trades, "result": s.result as i64, "deposit": deposit(&s.key) as u64, "back": back_of(deposit(&s.key), s.result) as u64 })).collect::<Vec<_>>(),
        "pool": pool as i64 });
    let name = format!("{COUNTS}{id}");
    nest.cas(key, &name, &v, 0)?;
    Ok(Some(nest.read(&name)?.0.ok_or("the counts did not stand")?))
}

/// the judge's word: the receipts of a road, signed by the road's judge alone, sent only after the close by the Nest's clock and only of the world
/// that happened. A word no count would read is refused here with why, so the judge knows, and never left on the log as if it had been said.
pub fn judge_says(nest: &Nest, key: &Key, id: &str, r: &Receipts) -> Result<Value, String> {
    let m = read_market(nest, id)?.ok_or(format!("no road {id}"))?;
    if m.road.judge.as_deref() != Some(key.key.as_str()) { return Err(format!("{} is not the judge of {id}: only the road's judge says its receipts", key.seat)); }
    if r.worth > PRICE { return Err(WORTH_RANGE.into()); }
    match world_of(&m.decision) { None => return Err(STILL_OPEN.into()), Some(w) if w != r.world => return Err(OTHER_WORLD.into()), Some(_) => {} }
    nest.append(Some(key), &key.seat, RECEIPTS, receipts_value(id, r))
}

/// the road's words for a hand or an agent: one object, prices as text
pub fn reading_value(r: &Reading) -> Value {
    let mut v = decision_value(&r.road, &r.market, &r.decision, &r.scores);
    v["now"] = json!(r.now); v["opens_ms"] = json!(r.road.terms.opens_ms);
    // a road with a fee shows it and each branch's fees, micro $WARM kept beside its pool; a bare road's reading is what it was
    if r.road.terms.fee_bp > 0 { v["fee_bp"] = json!(r.road.terms.fee_bp); v["fees"] = json!({ "pass": r.market.pass.fees as u64, "fail": r.market.fail.fees as u64 }); }
    v["both_ways"] = json!(r.road.terms.both_ways); v["roster"] = json!(r.road.roster.len()); v["judge"] = json!(r.road.judge);
    v["road"] = match &r.road.kind { Kind::Road { what } => json!(what), Kind::Allowances { month, .. } => json!(format!("the allowances of {month}")) };
    v["traders"] = r.market.traders.iter().map(|h| json!({ "seat": h.seat, "trades": h.trades, "deposit": h.deposit as u64 })).collect();
    v["why_refused"] = r.market.refused.iter().map(|x| Value::String(format!("{} ({}): {}", x.seq, x.seat, x.why))).collect();
    v
}
