//! poleis-seal · A RELEASE IS A VOTE: the one reader of the seal (the law: src/lib.rs).
//!
//!   poleis-seal read <commit> --road R [--json]               is <commit> sealed RATUM by the house's polis? one getMultipleAccounts for every nonce's
//!                                                              proposal and session, then the proposal's signatures and its two transactions
//!   poleis-seal verify <commit> --road R --seal S [--json]    the device's check of a seal a manifest carries: the name, then its two transactions
//!   poleis-seal newest <commits> --road R [--json]            THE SWARM SHIPS THE UPDATE: of the commits in the file <commits> (one a line, newest
//!                                                              first), the newest the polis sealed RATUM: six commits' accounts a call, from the
//!                                                              newest down, and the first that passed read whole as `read` reads it
//!
//! R is the road (JSON: rpc, program, polis), the install's; it names no proposer (any member proposes, the swarm decides); S a seal as `read` prints it. The chain is read at `finalized`: a seal that
//! could still be rolled back is no seal yet. Exit 0 sealed, 1 not sealed (said once, why), 2 not read. The last line of stdout is the JSON.

use poleis_seal::{accounts_word, proposal_at, proposal_of, commit_bytes, session_at, verify, Road, Seal, Word, NONCES};
use serde_json::{json, Value};
use std::io::Write as _;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

struct Rpc {
    url: String,
    calls: std::cell::Cell<u32>,
}
impl Rpc {
    /// The node: https anywhere, or http to this machine only (the practice chain).
    fn new(url: &str) -> Result<Self, String> {
        let local = ["http://127.0.0.1:", "http://[::1]:", "http://localhost:"].iter().any(|p| url.starts_with(p));
        if !(url.starts_with("https://") || local) || url.bytes().any(|b| b <= b' ' || b >= 0x7f || b == b'@' || b == b'\\') {
            return Err(format!("{url:?}: a node is reached by https://, or by http:// on this machine"));
        }
        Ok(Self { url: url.into(), calls: Default::default() })
    }
    fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        self.calls.set(self.calls.get() + 1);
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        let mut child = Command::new("/usr/bin/curl")
            .args(["-sS", "-X", "POST", "-H", "Content-Type: application/json", "--data-binary", "@-", "--max-time", "30", "--connect-timeout", "10"])
            .args(["--proto", "=http,https", "--max-redirs", "0", "--fail-with-body", &self.url])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("curl: {e}"))?;
        child.stdin.take().ok_or("curl's stdin")?.write_all(body.as_bytes()).map_err(|e| format!("curl: {e}"))?;
        let out = child.wait_with_output().map_err(|e| format!("curl: {e}"))?;
        if !out.status.success() {
            return Err(format!("{method}: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        let v: Value = serde_json::from_slice(&out.stdout).map_err(|e| format!("{method}: the node's answer is not JSON ({e})"))?;
        if let Some(e) = v.get("error") {
            return Err(format!("{method}: {e}"));
        }
        Ok(v["result"].clone())
    }
    fn tx(&self, sig: &str) -> Result<Value, String> {
        self.call("getTransaction", json!([sig, { "encoding": "json", "commitment": "finalized", "maxSupportedTransactionVersion": 0 }]))
    }
}

fn opt(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}
fn file_json(path: &str) -> Result<Value, String> {
    serde_json::from_str(&std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?).map_err(|e| format!("{path}: {e}"))
}

type Pairs = Vec<(Option<Vec<u8>>, Option<Vec<u8>>)>;
/// Each commit's eight nonces' proposal and session, in one getMultipleAccounts (six commits at most: 96 accounts of the call's 100), under
/// the program the road names (the launch's Poleis is the sitting's own id).
fn accounts(commits: &[String], road: &Road, rpc: &Rpc) -> Result<Vec<Pairs>, String> {
    let program = road.id()?;
    let mut keys = Vec::new();
    for commit in commits {
        let c = commit_bytes(commit)?;
        for n in 0..NONCES {
            let h = proposal_of(&c, road.polis, n);
            keys.extend([proposal_at(&program, &h).to_string(), session_at(&program, &h, road.polis).to_string()]);
        }
    }
    let v = rpc.call("getMultipleAccounts", json!([keys, { "encoding": "base64", "commitment": "finalized" }]))?;
    let data = |a: &Value| -> Option<Vec<u8>> {
        use base64::Engine as _;
        (a["owner"].as_str() == Some(road.program.as_str())).then(|| a["data"][0].as_str().and_then(|d| base64::engine::general_purpose::STANDARD.decode(d).ok())).flatten()
    };
    let arr = v["value"].as_array().filter(|a| a.len() == keys.len()).ok_or("getMultipleAccounts: no value of every account asked")?;
    Ok(arr.chunks(2 * NONCES as usize).map(|c| c.chunks(2).map(|p| (data(&p[0]), data(&p[1]))).collect()).collect())
}

/// newest: the commits from the newest, six a call, and the first whose accounts say PASSED read whole (`read`); a commit whose read finds
/// its seal no longer standing whole is passed over, said, and the next is read.
fn newest(commits: &[String], road: &Road, rpc: &Rpc) -> Result<Result<Seal, String>, String> {
    let mut notes = Vec::new();
    for six in commits.chunks(6) {
        for (commit, pairs) in six.iter().zip(accounts(six, road, rpc)?) {
            if let Word::Passed { .. } = accounts_word(commit, road, &pairs)? {
                match read(commit, road, rpc)? {
                    Ok(seal) => return Ok(Ok(seal)),
                    Err(why) => notes.push(format!("{}: {why}", &commit[..12.min(commit.len())])),
                }
            }
        }
    }
    Ok(Err(format!("none of the {} commit{} read is sealed RATUM by polis {}{}", commits.len(), if commits.len() == 1 { "" } else { "s" }, road.polis, if notes.is_empty() { String::new() } else { format!(" ({})", notes.join("; ")) })))
}

/// read: the accounts first (one call for every nonce), then, for the nonce that passed, the proposal's own signatures and its two transactions.
fn read(commit: &str, road: &Road, rpc: &Rpc) -> Result<Result<Seal, String>, String> {
    let program = road.id()?;
    let pairs = accounts(&[commit.to_string()], road, rpc)?.remove(0);
    let (nonce, proposal, proposer) = match accounts_word(commit, road, &pairs)? {
        Word::Passed { nonce, proposal, proposer } => (nonce, proposal, proposer),
        Word::Not(why) => return Ok(Err(why)),
    };
    let sigs = rpc.call("getSignaturesForAddress", json!([proposal_at(&program, &proposal).to_string(), { "limit": 100, "commitment": "finalized" }]))?;
    let landed: Vec<(String, u64)> = sigs
        .as_array()
        .ok_or("getSignaturesForAddress: no list")?
        .iter()
        .filter(|s| s["err"].is_null())
        .filter_map(|s| Some((s["signature"].as_str()?.to_string(), s["slot"].as_u64()?)))
        .collect();
    let mut cache = std::collections::HashMap::new();
    let mut tx = |sig: &str| -> Result<Value, String> {
        if let Some(t) = cache.get(sig) {
            return Ok(Value::clone(t));
        }
        let t = rpc.tx(sig)?;
        cache.insert(sig.to_string(), t.clone());
        Ok(t)
    };
    let logs = |t: &Value| -> Vec<String> { t["meta"]["logMessages"].as_array().map(|a| a.iter().filter_map(|l| l.as_str().map(String::from)).collect()).unwrap_or_default() };
    let mut sealed = None;
    for (s, slot) in &landed {
        // newest first: the seal is the last thing a release proposal's address sees (its execute is refused, and fails)
        let t = tx(s)?;
        if poleis_seal::seal_outcome(&logs(&t), &road.program, &proposal, road.polis).is_some() {
            sealed = Some((s.clone(), *slot, t));
            break;
        }
    }
    let mut proposed = None;
    for (s, _) in landed.iter().rev() {
        // oldest first: the proposal is the first thing its address saw
        let t = tx(s)?;
        if poleis_seal::proposer_in(&t, &road.program, &proposal).is_some() {
            proposed = Some((s.clone(), t));
            break;
        }
    }
    let (Some((signature, slot, st)), Some((proposed, pt))) = (sealed, proposed) else {
        return Ok(Err(format!("nonce {nonce}: the accounts say RATUM, and the seal's or the proposal's transaction is not among the address's last 100")));
    };
    let seal = Seal { commit: commit.into(), program: road.program.clone(), polis: road.polis, nonce, proposal, proposer, slot, signature, proposed };
    Ok(verify(&seal, commit, road, &st, &pt).map(|()| seal))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(step), Some(given)) = (args.first().cloned(), args.get(1).cloned()) else {
        eprintln!("usage: poleis-seal read <commit> --road R [--json] | verify <commit> --road R --seal S [--json] | newest <commits> --road R [--json]");
        return ExitCode::from(2);
    };
    let t0 = Instant::now();
    // the commit said: the one given, or the newest the swarm sealed (newest)
    let said = |sealed: Option<bool>, why: &str, seal: Option<&Seal>, rpc: Option<&Rpc>| {
        let commit = seal.map(|s| s.commit.clone()).unwrap_or_else(|| if step == "newest" { "the newest".into() } else { given.clone() });
        let calls = rpc.map(|r| r.calls.get()).unwrap_or(0);
        let ms = t0.elapsed().as_millis() as u64;
        let line = match (sealed, seal) {
            (Some(true), Some(s)) => format!("SEALED {}: polis {}'s proposal {} (nonce {}), class SIGNAL, RATUM at slot {} ({}), proposed by {}; {calls} calls, {ms} ms", &commit[..12.min(commit.len())], s.polis, &poleis_seal::hex(&s.proposal)[..12], s.nonce, s.slot, &s.signature[..12.min(s.signature.len())], s.proposer),
            (Some(_), _) => format!("NOT SEALED {}: {why}; {calls} calls, {ms} ms", &commit[..12.min(commit.len())]),
            (None, _) => format!("NOT READ {}: {why}", &commit[..12.min(commit.len())]),
        };
        if args.iter().any(|a| a == "--json") {
            eprintln!("{line}");
            println!("{}", json!({ "sealed": sealed == Some(true), "read": sealed.is_some(), "commit": commit, "why": if sealed == Some(true) { Value::Null } else { json!(why) }, "seal": seal.map(Seal::to_json), "calls": calls, "ms": ms }));
        } else {
            println!("{line}");
        }
        ExitCode::from(match sealed { Some(true) => 0, Some(false) => 1, None => 2 })
    };
    let road = match opt(&args, "--road").ok_or("no --road".to_string()).and_then(|p| file_json(&p)).and_then(|v| Road::parse(&v)) {
        Ok(r) => r,
        Err(e) => return said(None, &format!("the road: {e}"), None, None),
    };
    let rpc = match Rpc::new(&road.rpc) {
        Ok(r) => r,
        Err(e) => return said(None, &e, None, None),
    };
    match step.as_str() {
        "read" => match read(&given, &road, &rpc) {
            Ok(Ok(seal)) => said(Some(true), "", Some(&seal), Some(&rpc)),
            Ok(Err(why)) => said(Some(false), &why, None, Some(&rpc)),
            Err(e) => said(None, &e, None, Some(&rpc)),
        },
        "verify" => {
            let seal = match opt(&args, "--seal").ok_or("no --seal".to_string()).and_then(|p| file_json(&p)).and_then(|v| Seal::parse(&v)) {
                Ok(s) => s,
                Err(e) => return said(Some(false), &e, None, Some(&rpc)),
            };
            match rpc.tx(&seal.signature).and_then(|st| Ok((st, rpc.tx(&seal.proposed)?))) {
                Ok((st, pt)) => match verify(&seal, &given, &road, &st, &pt) {
                    Ok(()) => said(Some(true), "", Some(&seal), Some(&rpc)),
                    Err(why) => said(Some(false), &why, None, Some(&rpc)),
                },
                Err(e) => said(None, &e, None, Some(&rpc)),
            }
        }
        "newest" => match std::fs::read_to_string(&given).map_err(|e| format!("{given}: {e}")).and_then(|t| newest(&t.split_whitespace().map(str::to_string).collect::<Vec<_>>(), &road, &rpc)) {
            Ok(Ok(seal)) => said(Some(true), "", Some(&seal), Some(&rpc)),
            Ok(Err(why)) => said(Some(false), &why, None, Some(&rpc)),
            Err(e) => said(None, &e, None, Some(&rpc)),
        },
        _ => said(None, &format!("no step {step:?}: read, verify or newest"), None, None),
    }
}
