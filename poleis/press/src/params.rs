//! The params of a sitting: every number and address that is his, in one JSON file. A value not yet ruled is a string that begins
//! `PLACEHOLDER` and says what it waits for; a sitting whose params hold one writes what it would send, for reading, and sends nothing
//! (as tools/warm-launch/params.mainnet.json does). The rehearsal's params name its stand-ins' addresses `STAND-IN`, filled from the
//! stand-ins' own file (the rehearsal's agents, roles and permit issuer, made on the local chain by their own process), and only in a
//! rehearsal. THE WHOLE TREE DECIDES (his Nov 22 2026 09:50:51 UTC "you gotta fucking stop treating the keystones as what make the
//! decisions porcoddio. the fucking entire tree perfect bicameralism gvs posiverse neghiverse molecularity"): no list of his keys is a
//! param. A params file that names `keystones`, or the first sale's terms the minted floor replaced, is refused by words that say why.

use crate::rpc::Cluster;
use crate::Res;
use serde_json::Value;
use solana_pubkey::Pubkey;
use std::str::FromStr;

pub const PLACEHOLDER: &str = "PLACEHOLDER";
pub const STAND_IN: &str = "STAND-IN";

/// The words a params file that names his Keystones is refused by.
pub const KEYSTONES_NAMED: &str = "params keystones: no key of his decides. Root 0's floor is the minted Lyssajudrons (`floor`: each seat minted by its own buyer on the curve, with a permit of the mint door), the hexagons are the commons, and his own Lyssajudron is one hexagon like anyone's: a list of his keys is no seat, no chair and no proposer. Take the line out";
/// The first sale's terms (a sealed bid for a seat), which the minted floor replaced.
pub const FIRST_SALE: [&str; 5] = ["genesis.reserve", "genesis.deposit", "genesis.commitSlots", "genesis.revealSlots", "genesis.bid"];
/// The words a params file that still names one of them is refused by.
pub const FIRST_SALE_NAMED: &str = "the first sale is gone: root 0's seats are minted on the curve the params name under `floor` (startLamports, stepLamports, opensAfterSlots, windowSlots, issuer), never bid for in a sealed sale. Take the line out";

/// THE STAKE EARNS (Nov 25 2026): the sections of a later act, voted after the launch's eight. Their blanks hold only their own steps (`press
/// engine`, `press engine-act`, the ninth act in the crank, a vote and a chair); the sitting and every step of the launch's own set are held
/// only by the launch's values.
pub const LATER: [&str; 2] = ["engine", "light"];

fn section_of(path: &str) -> &str {
    path.split(['.', '[']).next().unwrap_or(path)
}

pub struct Params {
    pub raw: Value,
    pub sha256: String,
    pub cluster: Cluster,
}

/// Every path in `v` whose value is a string beginning with `mark`.
fn marked(v: &Value, path: &str, mark: &str, out: &mut Vec<String>) {
    match v {
        Value::String(s) if s.starts_with(mark) => out.push(path.to_string()),
        Value::Array(a) => a.iter().enumerate().for_each(|(i, x)| marked(x, &format!("{path}[{i}]"), mark, out)),
        Value::Object(o) => o.iter().filter(|(k, _)| k.as_str() != "note").for_each(|(k, x)| marked(x, &if path.is_empty() { k.clone() } else { format!("{path}.{k}") }, mark, out)),
        _ => {}
    }
}

impl Params {
    /// The params file, read whole; its sha256 is the sitting's: a record made under other params is not this sitting's.
    pub fn read(path: &std::path::Path) -> Res<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_text(&text)
    }
    pub fn from_text(text: &str) -> Res<Self> {
        let raw: Value = serde_json::from_str(text).map_err(|e| format!("the params: {e}"))?;
        let cluster = Cluster::parse(raw["cluster"].as_str().ok_or("the params name no cluster")?)?;
        // THE WHOLE TREE DECIDES: no key of his is a param, and no seat is bid for
        if raw.get("keystones").is_some() {
            return Err(KEYSTONES_NAMED.into());
        }
        if let Some(path) = FIRST_SALE.iter().find(|path| {
            let (parent, leaf) = path.split_once('.').unwrap();
            raw.get(parent).and_then(|g| g.get(leaf)).is_some()
        }) {
            return Err(format!("params {path}: {FIRST_SALE_NAMED}"));
        }
        Ok(Self { sha256: crate::wire::hex(&crate::wire::sha256(text.as_bytes())), raw, cluster })
    }
    /// A rehearsal's stand-ins put where its params say STAND-IN: `stand_ins` maps a path (`posiverse.agents`, `floor.issuer`...) to its value.
    pub fn fill(&mut self, stand_ins: &Value) -> Res<()> {
        if self.cluster != Cluster::Localnet {
            return Err("stand-ins are a rehearsal's: mainnet's params name his own".into());
        }
        let mut paths = Vec::new();
        marked(&self.raw, "", STAND_IN, &mut paths);
        for p in paths {
            let key = p.split('[').next().unwrap_or(&p).to_string();
            let v = stand_ins.get(&key).ok_or_else(|| format!("the stand-ins name nothing for {key}"))?;
            let slot = self.at_mut(&key)?;
            *slot = v.clone();
        }
        Ok(())
    }
    fn at_mut(&mut self, path: &str) -> Res<&mut Value> {
        let mut v = &mut self.raw;
        for part in path.split('.') {
            v = v.get_mut(part).ok_or_else(|| format!("the params have no {path}"))?;
        }
        Ok(v)
    }
    /// The value at a dotted path.
    pub fn at(&self, path: &str) -> &Value {
        let mut v = &self.raw;
        for part in path.split('.') {
            v = &v[part];
        }
        v
    }
    /// Every value of the launch's own still waiting on his word (`PLACEHOLDER`) or on a stand-in (`STAND-IN`). A later act's section
    /// (`LATER`) is not the launch's: its blanks hold only its own steps (`later_placeholders`).
    pub fn placeholders(&self) -> Vec<String> {
        let mut out = self.all_placeholders();
        out.retain(|p| !LATER.contains(&section_of(p)));
        out
    }
    /// The blanks of one later act's section (`engine`): they hold that act and its steps, never the launch's.
    pub fn later_placeholders(&self, section: &str) -> Vec<String> {
        let mut out = self.all_placeholders();
        out.retain(|p| section_of(p) == section);
        out
    }
    fn all_placeholders(&self) -> Vec<String> {
        let mut out = Vec::new();
        marked(&self.raw, "", PLACEHOLDER, &mut out);
        marked(&self.raw, "", STAND_IN, &mut out);
        out
    }
    /// A sitting sends nothing while one value is not his.
    pub fn sendable(&self) -> Res<()> {
        let p = self.placeholders();
        if p.is_empty() {
            Ok(())
        } else {
            Err(format!("the params still hold {} values that are not his yet ({}): what the sitting would send is written for reading, and nothing is sent", p.len(), p.join(", ")))
        }
    }
    pub fn is_placeholder(&self, path: &str) -> bool {
        self.at(path).as_str().is_some_and(|s| s.starts_with(PLACEHOLDER) || s.starts_with(STAND_IN))
    }

    /// A number: a JSON integer, or its decimal text. A placeholder reads as `stand_in` (only a set written for reading ever uses one).
    pub fn u64_or(&self, path: &str, stand_in: u64) -> Res<u64> {
        if self.is_placeholder(path) {
            return Ok(stand_in);
        }
        let v = self.at(path);
        v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())).ok_or_else(|| format!("params {path}: a whole number, not {v}"))
    }
    pub fn u64(&self, path: &str) -> Res<u64> {
        if self.is_placeholder(path) {
            return Err(format!("params {path} is not his yet: {}", self.at(path)));
        }
        self.u64_or(path, 0)
    }
    pub fn text(&self, path: &str) -> Res<String> {
        self.at(path).as_str().map(str::to_string).ok_or_else(|| format!("params {path}: a text"))
    }
    /// An address; a placeholder reads as an address nobody holds a key for: sha256 of its words.
    pub fn key_or_standin(&self, path: &str) -> Res<Pubkey> {
        let s = self.text(path)?;
        if s.starts_with(PLACEHOLDER) || s.starts_with(STAND_IN) {
            return Ok(placeholder_address(&s));
        }
        Pubkey::from_str(&s).map_err(|_| format!("params {path}: {s:?} is not an address"))
    }
    pub fn key(&self, path: &str) -> Res<Pubkey> {
        if self.is_placeholder(path) {
            return Err(format!("params {path} is not his yet"));
        }
        self.key_or_standin(path)
    }
    /// His posiverse's agents, the chairs of unit 1 in their order: one to twelve, each distinct. An agent's Nest public key is its Solana
    /// address, read off the Nest by whoever fills the params.
    pub fn agents(&self) -> Res<Vec<Pubkey>> {
        let out = self.addresses("posiverse.agents", "his posiverse's agents' addresses, its chairs in order")?;
        if out.is_empty() || out.len() > poleis_treasury::PENTAGONS {
            return Err(format!("params posiverse.agents: one to {} agents, not {}", poleis_treasury::PENTAGONS, out.len()));
        }
        Ok(out)
    }
    /// A list of distinct addresses at `path` (a placeholder reads as an address nobody holds a key for).
    fn addresses(&self, path: &str, what: &str) -> Res<Vec<Pubkey>> {
        let a = self.at(path).as_array().ok_or_else(|| format!("params {path}: {what}"))?;
        let mut out = Vec::new();
        for (i, v) in a.iter().enumerate() {
            let s = v.as_str().ok_or_else(|| format!("params {path}: addresses"))?;
            let k = if s.starts_with(PLACEHOLDER) || s.starts_with(STAND_IN) {
                placeholder_address(s)
            } else {
                Pubkey::from_str(s).map_err(|_| format!("params {path}[{i}]: {s:?} is not an address"))?
            };
            if out.contains(&k) {
                return Err(format!("params {path}: the same address twice"));
            }
            out.push(k);
        }
        Ok(out)
    }
}

/// The address a placeholder stands for in a set written for reading: sha256 of its words, so nobody holds its key and the set names it.
pub fn placeholder_address(words: &str) -> Pubkey {
    Pubkey::new_from_array(crate::wire::sha256(format!("launch press placeholder: {words}").as_bytes()))
}

/// The opening the mine counts its halvings from: milliseconds of UTC, from an ISO time (`2026-11-21T20:00:00Z`) or a number.
pub fn ms_of(v: &Value) -> Res<u64> {
    if let Some(n) = v.as_u64() {
        return Ok(n);
    }
    let s = v.as_str().ok_or("a time: an ISO UTC instant or milliseconds")?;
    iso_ms(s)
}

/// `YYYY-MM-DDTHH:MM:SS(.mmm)Z` in milliseconds since the epoch, by the civil calendar (Howard Hinnant's days_from_civil).
pub fn iso_ms(s: &str) -> Res<u64> {
    let bad = || format!("{s:?}: an instant as YYYY-MM-DDTHH:MM:SSZ");
    let b = s.as_bytes();
    if b.len() < 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' || *b.last().unwrap() != b'Z' {
        return Err(bad());
    }
    let num = |r: std::ops::Range<usize>| s.get(r).and_then(|t| t.parse::<i64>().ok()).ok_or_else(bad);
    let (y, m, d, hh, mm, ss) = (num(0..4)?, num(5..7)?, num(8..10)?, num(11..13)?, num(14..16)?, num(17..19)?);
    let ms = if b[19] == b'.' { num(20..23)? } else { 0 };
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 60 {
        return Err(bad());
    }
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let t = ((days * 86_400 + hh * 3_600 + mm * 60 + ss) * 1_000) + ms;
    u64::try_from(t).map_err(|_| bad())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instant_is_its_milliseconds() {
        assert_eq!(iso_ms("1970-01-01T00:00:00Z").unwrap(), 0);
        // his word's instant, Nov 21 2026 17:34:37 UTC: the warm program's tests' clock
        assert_eq!(iso_ms("2026-11-21T17:34:37Z").unwrap(), 1_795_282_477_000);
        assert_eq!(iso_ms("2024-02-29T12:00:00.250Z").unwrap(), 1_709_208_000_250);
        for bad in ["2026-13-01T00:00:00Z", "2026-11-21 17:34:37Z", "2026-11-21T17:34:37", "yesterday"] {
            assert!(iso_ms(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_placeholder_is_read_and_never_sent() {
        let p = Params::from_text(r#"{"cluster":"mainnet","note":"PLACEHOLDER in a note is no value","a":{"b":"PLACEHOLDER: his","c":5},"posiverse":{"agents":["PLACEHOLDER: one","PLACEHOLDER: two","PLACEHOLDER: three"]}}"#).unwrap();
        assert_eq!(p.placeholders(), vec!["a.b", "posiverse.agents[0]", "posiverse.agents[1]", "posiverse.agents[2]"]);
        assert!(p.sendable().is_err());
        assert_eq!(p.u64_or("a.b", 9).unwrap(), 9);
        assert!(p.u64("a.b").is_err());
        assert_eq!(p.u64("a.c").unwrap(), 5);
        assert_eq!(p.agents().unwrap().len(), 3);
        assert!(Params::from_text(r#"{"cluster":"devnet"}"#).is_err(), "devnet is no step of his");
    }

    #[test]
    fn stand_ins_fill_a_rehearsal_and_never_mainnet() {
        let mut p = Params::from_text(r#"{"cluster":"localnet","w":{"t":"STAND-IN: a treasury"},"floor":{"issuer":"STAND-IN: the issuer"}}"#).unwrap();
        let k = |b: u8| Pubkey::new_from_array([b; 32]).to_string();
        let s = serde_json::json!({ "w.t": k(1), "floor.issuer": k(2) });
        p.fill(&s).unwrap();
        assert!(p.sendable().is_ok());
        assert_eq!(p.key("w.t").unwrap(), Pubkey::new_from_array([1; 32]));
        assert_eq!(p.key("floor.issuer").unwrap(), Pubkey::new_from_array([2; 32]));
        let mut m = Params::from_text(r#"{"cluster":"mainnet","w":{"t":"STAND-IN"}}"#).unwrap();
        assert!(m.fill(&s).is_err());
    }

    #[test]
    fn no_key_of_his_is_a_param_and_no_seat_is_bid_for() {
        // THE WHOLE TREE DECIDES: a params file that names his Keystones is refused, placeholders or not, and says why
        for ks in [r#"["PLACEHOLDER: one","PLACEHOLDER: two","PLACEHOLDER: three"]"#, "[]"] {
            let e = Params::from_text(&format!(r#"{{"cluster":"mainnet","keystones":{ks}}}"#)).err().unwrap();
            assert_eq!(e, KEYSTONES_NAMED);
        }
        assert!(KEYSTONES_NAMED.contains("no key of his decides") && KEYSTONES_NAMED.contains("one hexagon like anyone's"));
        // the first sale's terms: the minted floor replaced them
        for path in FIRST_SALE {
            let (parent, leaf) = path.split_once('.').unwrap();
            let e = Params::from_text(&format!(r#"{{"cluster":"localnet","{parent}":{{"{leaf}":1}}}}"#)).err().unwrap();
            assert!(e.starts_with(&format!("params {path}: ")) && e.contains(FIRST_SALE_NAMED), "{path}: {e}");
        }
        // a note that says the word is no param
        Params::from_text(r#"{"cluster":"localnet","note":"no keystones here","genesis":{"note":"no reserve"}}"#).unwrap();
    }
}
