// Where this file came from: poleis/press/src/rpc.rs at d3a23b83af (the launch press's own JSON-RPC, by the system's curl), copied whole below this line for the war's rehearsal; the copy now adds a Devnet cluster (https://api.devnet.solana.com and nothing else, its genesis pinned) and a pace under the public endpoint's limits, for the war on devnet (his "Yes, devnet", Nov 24 2026 12:24:57 UTC); the press keeps refusing devnet.
//! The RPC, by the system's curl: one JSON-RPC call a process, its body on curl's stdin, nothing on its command line but the URL. Two
//! clusters and no third: mainnet, by HTTPS to the endpoint the params name, and a validator on this machine (the rehearsal), by HTTP to
//! 127.0.0.1 or [::1] and nothing else (an address, never a name a resolver could turn elsewhere). No proxy, no redirect. The cluster is
//! held to its genesis hash before anything is sent: devnet and testnet are refused by theirs, mainnet is only mainnet's.

use crate::Res;
use serde_json::{json, Value};
use std::io::Write as _;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Mainnet's genesis hash.
pub const MAINNET_GENESIS: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
/// Devnet's and testnet's: the press never sends there ("devnet is no step of his"). This copy reaches devnet by `Cluster::Devnet` alone, at
/// DEVNET_URL and under DEVNET_GENESIS, for the war on devnet; testnet never.
pub const DEVNET_GENESIS: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";
pub const TESTNET_GENESIS: &str = "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY";

/// Devnet's own endpoint, the one the war's rehearsal on devnet may reach (this copy's addition; the press reaches none)
pub const DEVNET_URL: &str = "https://api.devnet.solana.com";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cluster {
    Mainnet,
    /// a validator on this machine: the rehearsal
    Localnet,
    /// Solana's devnet, by its own public endpoint alone, its genesis pinned (this copy's addition, for the war on devnet)
    Devnet,
}
impl Cluster {
    pub fn parse(s: &str) -> Res<Self> {
        match s {
            "mainnet" => Ok(Self::Mainnet),
            "localnet" => Ok(Self::Localnet),
            "devnet" => Ok(Self::Devnet),
            _ => Err(format!("the cluster is mainnet, localnet (a validator on this machine) or devnet, not {s:?}")),
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Localnet => "localnet",
            Self::Devnet => "devnet",
        }
    }
    /// the least time between two calls: devnet's public endpoint allows about 100 requests and 40 new connections in 10 s from one
    /// address, and every call here is a connection of its own (one curl a call), so it is paced at one call in 300 ms (3.3 a second)
    pub fn pace(&self) -> Duration {
        match self {
            Self::Devnet => Duration::from_millis(300),
            _ => Duration::ZERO,
        }
    }
}

/// The URL a cluster may be reached at: mainnet by https://, the local validator by http:// to a loopback address and a port.
pub fn check_url(url: &str, cluster: Cluster) -> Res<()> {
    let refuse = |why: &str| Err(format!("{url:?}: {why}"));
    if url.bytes().any(|b| b <= b' ' || b >= 0x7f || b == b'@' || b == b'\\') {
        return refuse("a URL with a space, a control byte, a user or a backslash");
    }
    match cluster {
        Cluster::Devnet => {
            if url != DEVNET_URL && url != format!("{DEVNET_URL}/") {
                return refuse("devnet is reached at https://api.devnet.solana.com and nothing else");
            }
            Ok(())
        }
        Cluster::Mainnet => {
            let Some(rest) = url.strip_prefix("https://") else { return refuse("mainnet is reached by https:// only") };
            let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
            if host.is_empty() {
                return refuse("no host");
            }
            Ok(())
        }
        Cluster::Localnet => {
            let Some(rest) = url.strip_prefix("http://") else { return refuse("the rehearsal's validator is reached by http:// to this machine") };
            let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
            if !(path.is_empty() || path == "/") {
                return refuse("a path after the port");
            }
            let port = if let Some(p) = authority.strip_prefix("127.0.0.1:") {
                p
            } else if let Some(p) = authority.strip_prefix("[::1]:") {
                p
            } else {
                return refuse("the rehearsal talks to 127.0.0.1 or [::1] and nothing else");
            };
            match port.parse::<u16>() {
                Ok(p) if p > 0 && port == p.to_string() => Ok(()),
                _ => refuse("not a port"),
            }
        }
    }
}

pub struct Rpc {
    pub url: String,
    pub cluster: Cluster,
    /// every call made, by method: what the certificate says the sitting read and sent
    pub calls: std::cell::RefCell<std::collections::BTreeMap<String, u64>>,
    /// when the last call began (this copy's pace), and how many times the endpoint answered 429 and was waited out
    pub last: std::cell::Cell<Option<std::time::Instant>>,
    pub refused_by_rate: std::cell::Cell<u64>,
}

/// An account as the RPC answers it.
#[derive(Clone, Debug, Default)]
pub struct Account {
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub data: Vec<u8>,
}

impl Rpc {
    pub fn new(url: &str, cluster: Cluster) -> Res<Self> {
        check_url(url, cluster)?;
        Ok(Self { url: url.to_string(), cluster, calls: Default::default(), last: Default::default(), refused_by_rate: Default::default() })
    }

    /// One call, its `result`, or the RPC's own words. A refusal by rate (429) or a fault of the endpoint (5xx, a connection cut) is tried
    /// again, ten times at most, waiting longer each time.
    pub fn call(&self, method: &str, params: Value) -> Res<Value> {
        *self.calls.borrow_mut().entry(method.to_string()).or_default() += 1;
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        // devnet waits longer between tries: its limits are counted over 10 s
        let (mut wait, tries) = if self.cluster == Cluster::Devnet { (Duration::from_secs(2), 12) } else { (Duration::from_millis(300), 10) };
        for attempt in 1..=tries {
            // the pace: never two calls closer than the cluster's least time between them
            if let Some(t) = self.last.get() {
                let since = t.elapsed();
                if since < self.cluster.pace() {
                    std::thread::sleep(self.cluster.pace() - since);
                }
            }
            self.last.set(Some(std::time::Instant::now()));
            match self.post(&body) {
                Ok((200, text)) => {
                    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{method}: the RPC's answer is not JSON ({e})"))?;
                    if let Some(err) = v.get("error") {
                        // a refusal by rate in the JSON-RPC's own words is waited out like an HTTP 429
                        if err.get("code").and_then(Value::as_i64) == Some(429) && attempt < tries {
                            self.refused_by_rate.set(self.refused_by_rate.get() + 1);
                            std::thread::sleep(wait);
                            wait = (wait * 2).min(Duration::from_secs(30));
                            continue;
                        }
                        return Err(format!("{method}: {err}"));
                    }
                    return Ok(v.get("result").cloned().unwrap_or(Value::Null));
                }
                Ok((status, text)) if (status == 429 || status >= 500) && attempt < tries => {
                    if status == 429 {
                        self.refused_by_rate.set(self.refused_by_rate.get() + 1);
                    }
                    let _ = text;
                }
                Ok((status, text)) => return Err(format!("{method}: the RPC answered {status}: {}", text.chars().take(200).collect::<String>())),
                Err(e) if attempt < tries => {
                    let _ = e;
                }
                Err(e) => return Err(format!("{method}: {e}")),
            }
            std::thread::sleep(wait);
            wait = (wait * 2).min(Duration::from_secs(if self.cluster == Cluster::Devnet { 30 } else { 15 }));
        }
        Err(format!("{method}: the RPC never answered"))
    }

    /// One POST by curl: its status and its body.
    fn post(&self, body: &str) -> Res<(u16, String)> {
        let proto = match self.cluster {
            Cluster::Mainnet | Cluster::Devnet => "=https",
            Cluster::Localnet => "=http",
        };
        let mut child = Command::new("/usr/bin/curl")
            .args(["-sS", "-X", "POST", "-H", "Content-Type: application/json", "--data-binary", "@-", "--max-time", "120", "--connect-timeout", "10"])
            .args(["--noproxy", "*", "--proto", proto, "--max-redirs", "0", "-w", "\n%{http_code}", &self.url])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("curl: {e}"))?;
        child.stdin.take().ok_or("curl's stdin")?.write_all(body.as_bytes()).map_err(|e| format!("curl: {e}"))?;
        let out = child.wait_with_output().map_err(|e| format!("curl: {e}"))?;
        if !out.status.success() {
            return Err(format!("curl: {} {}", out.status, String::from_utf8_lossy(&out.stderr).trim()));
        }
        let text = String::from_utf8(out.stdout).map_err(|_| "curl: an answer that is not text")?;
        let (body, status) = text.rsplit_once('\n').ok_or("curl: no status")?;
        Ok((status.trim().parse().map_err(|_| "curl: a status that is not a number")?, body.to_string()))
    }

    /// The cluster this endpoint is, held to the params' word: mainnet's genesis for mainnet, and for the rehearsal anything that is not a
    /// public cluster's.
    pub fn check_cluster(&self) -> Res<String> {
        let g = self.call("getGenesisHash", json!([]))?.as_str().ok_or("getGenesisHash: no hash")?.to_string();
        match self.cluster {
            Cluster::Mainnet if g != MAINNET_GENESIS => Err(format!("{} answers genesis {g}: not mainnet", self.url)),
            Cluster::Devnet if g != DEVNET_GENESIS => Err(format!("{} answers genesis {g}: not devnet", self.url)),
            Cluster::Localnet if [MAINNET_GENESIS, DEVNET_GENESIS, TESTNET_GENESIS].contains(&g.as_str()) => {
                Err(format!("{} answers a public cluster's genesis {g}: the rehearsal runs on a validator of this machine only", self.url))
            }
            _ => Ok(g),
        }
    }

    pub fn slot(&self) -> Res<u64> {
        self.call("getSlot", json!([{ "commitment": "processed" }]))?.as_u64().ok_or_else(|| "getSlot: no slot".into())
    }
    pub fn blockhash(&self) -> Res<[u8; 32]> {
        let v = self.call("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))?;
        crate::wire::unbase58(v["value"]["blockhash"].as_str().ok_or("getLatestBlockhash: no blockhash")?)
    }
    pub fn rent(&self, size: usize) -> Res<u64> {
        self.call("getMinimumBalanceForRentExemption", json!([size]))?.as_u64().ok_or_else(|| "getMinimumBalanceForRentExemption: no lamports".into())
    }
    pub fn balance(&self, key: &str) -> Res<u64> {
        self.call("getBalance", json!([key, { "commitment": "confirmed" }]))?["value"].as_u64().ok_or_else(|| "getBalance: no lamports".into())
    }
    /// An account at the confirmed level, None when there is none.
    pub fn account(&self, key: &str) -> Res<Option<Account>> {
        let v = self.call("getAccountInfo", json!([key, { "encoding": "base64", "commitment": "confirmed" }]))?;
        let a = &v["value"];
        if a.is_null() {
            return Ok(None);
        }
        let data = crate::wire::unb64(a["data"][0].as_str().ok_or("getAccountInfo: no data")?)?;
        Ok(Some(Account {
            lamports: a["lamports"].as_u64().unwrap_or(0),
            owner: a["owner"].as_str().unwrap_or_default().to_string(),
            executable: a["executable"].as_bool().unwrap_or(false),
            data,
        }))
    }
    /// The token accounts `owner` holds under the token program `program` (`getTokenAccountsByOwner`), at the confirmed level: each one's
    /// address and data.
    pub fn token_accounts(&self, owner: &str, program: &str) -> Res<Vec<(String, Vec<u8>)>> {
        let v = self.call("getTokenAccountsByOwner", json!([owner, { "programId": program }, { "encoding": "base64", "commitment": "confirmed" }]))?;
        let mut out = Vec::new();
        for a in v["value"].as_array().cloned().unwrap_or_default() {
            let data = crate::wire::unb64(a["account"]["data"][0].as_str().ok_or("getTokenAccountsByOwner: no data")?)?;
            out.push((a["pubkey"].as_str().unwrap_or_default().to_string(), data));
        }
        Ok(out)
    }
    /// The statuses of up to 256 signatures: each its slot and its error (null when it landed well), None when the cluster has not seen it.
    pub fn statuses(&self, sigs: &[String]) -> Res<Vec<Option<(u64, Value, String)>>> {
        let mut out = Vec::with_capacity(sigs.len());
        for part in sigs.chunks(256) {
            let v = self.call("getSignatureStatuses", json!([part, { "searchTransactionHistory": false }]))?;
            let arr = v["value"].as_array().cloned().unwrap_or_default();
            for s in arr {
                out.push(if s.is_null() {
                    None
                } else {
                    Some((s["slot"].as_u64().unwrap_or(0), s["err"].clone(), s["confirmationStatus"].as_str().unwrap_or_default().to_string()))
                });
            }
        }
        Ok(out)
    }
    /// A landed transaction's slot, fee, units and logs, None while the RPC does not have it.
    pub fn transaction(&self, sig: &str) -> Res<Option<(u64, u64, Option<u64>, Value)>> {
        let t = self.call("getTransaction", json!([sig, { "encoding": "base64", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }]))?;
        if t.is_null() {
            return Ok(None);
        }
        Ok(Some((t["slot"].as_u64().unwrap_or(0), t["meta"]["fee"].as_u64().unwrap_or(0), t["meta"]["computeUnitsConsumed"].as_u64(), t["meta"]["logMessages"].clone())))
    }
    pub fn send(&self, wire: &[u8], preflight: bool) -> Res<String> {
        let v = self.call(
            "sendTransaction",
            json!([crate::wire::b64(wire), { "encoding": "base64", "skipPreflight": !preflight, "preflightCommitment": "confirmed", "maxRetries": 0 }]),
        )?;
        v.as_str().map(str::to_string).ok_or_else(|| "sendTransaction: no signature".into())
    }
    /// A simulation with its signatures verified: its error (null when it would land), its units, its logs.
    pub fn simulate(&self, wire: &[u8]) -> Res<(Value, u64, Value)> {
        let v = self.call("simulateTransaction", json!([crate::wire::b64(wire), { "encoding": "base64", "sigVerify": true, "commitment": "confirmed" }]))?;
        let r = &v["value"];
        Ok((r["err"].clone(), r["unitsConsumed"].as_u64().unwrap_or(0), r["logs"].clone()))
    }
    pub fn block_time(&self, slot: u64) -> Res<Option<i64>> {
        Ok(self.call("getBlockTime", json!([slot]))?.as_i64())
    }
    /// The faucet of a validator on this machine (the rehearsal only).
    pub fn airdrop(&self, key: &str, lamports: u64) -> Res<String> {
        if self.cluster != Cluster::Localnet {
            return Err("an airdrop is the local validator's faucet: the rehearsal's only".into());
        }
        self.call("requestAirdrop", json!([key, lamports]))?.as_str().map(str::to_string).ok_or_else(|| "requestAirdrop: no signature".into())
    }
    /// The runtime's features on this chain, as the feature program holds them (an account a feature, its data `activated_at`: 0, or 1
    /// and the slot): each id, and whether it is active.
    pub fn features(&self) -> Res<Vec<(String, bool)>> {
        let v = self.call("getProgramAccounts", json!(["Feature111111111111111111111111111111111111", { "encoding": "base64", "commitment": "confirmed" }]))?;
        let mut out: Vec<(String, bool)> = v
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|a| {
                let data = a["account"]["data"][0].as_str().and_then(|d| crate::wire::unb64(d).ok()).unwrap_or_default();
                (a["pubkey"].as_str().unwrap_or_default().to_string(), data.first() == Some(&1))
            })
            .collect();
        out.sort();
        Ok(out)
    }
    pub fn signatures_for(&self, key: &str, limit: usize) -> Res<Vec<(String, u64, bool)>> {
        let v = self.call("getSignaturesForAddress", json!([key, { "limit": limit, "commitment": "confirmed" }]))?;
        Ok(v.as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|s| (s["signature"].as_str().unwrap_or_default().to_string(), s["slot"].as_u64().unwrap_or(0), s["err"].is_null()))
            .collect())
    }
}

/// Wait while `done` says not yet, up to `bound`, a pause between: what the press does while a cluster catches up.
pub fn wait<T>(bound: Duration, mut done: impl FnMut() -> Res<Option<T>>) -> Res<T> {
    let begun = std::time::Instant::now();
    loop {
        if let Some(t) = done()? {
            return Ok(t);
        }
        if begun.elapsed() > bound {
            return Err(format!("waited {} s and it never came", bound.as_secs()));
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}
