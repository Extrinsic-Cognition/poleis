//! A RELEASE IS A VOTE (Nov 23 2026; his 11:58 UTC "the truth is, the swarm deliberates and votes and that lands on solana", then ~12:4x "build all?";
//! the record: ec-workspace docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md, its Nov 23 11:58 section, "the truth of a release is the vote
//! landed on Solana, naming a hash... the Nest and every device read the chain and carry out what landed; they decide nothing").
//!
//! The law of the seal, pure: how a release proposal names a commit, what the chain's accounts say of it, and what its two transactions (the
//! proposal's and the seal's) prove. The hand (main.rs) reads a Solana node and nothing else; pull.py carries the same law for the devices, held to
//! the same vector (tests/vector.json).
//!
//! THE CLASS IS SIGNAL. The program's own rule (poleis_treasury/src/lib.rs, over MARKET_CLASSES): "The law (CODEX) and the signals (the code, the
//! weights, the engine's own rulings) stay under the AND for ever", and the market only for money, seats and the hands. The house's code is a
//! signal by that line: it is never decided by a market (market_bit(SIGNAL) is 0) nor by the SUM (sum_bit(SIGNAL, RELEASE) is 0), so both chambers
//! must pass it. Not CODEX: CODEX is the society's own law and the program's own bytes (CODEX_UPGRADE), its window the long one ("a society buys fast
//! and rewrites itself slowly"); the house's code is not the society's law (RELIGIO Art. VII, "Codex est res HEXAGONORUM"). Not ACT: ACT is the hands,
//! an instruction to another program, and it may go to the market. The proposal asks for CONSENT besides: every chamber of the decision must pass,
//! the crews of its chairs too, so the origin's seal alone says the whole tree passed.
//!
//! THE KIND IS 0x52 ("R"), a kind the program does not execute: its `execute` refuses a SIGNAL kind it does not know (BadPayload), and nothing
//! of a release is carried out on the chain; the house carries it out. It stands far from the program's own kinds (0 to 5), so the program's next
//! kind does not take it before the program reserves it (owed, at its next upgrade: his press).
//!
//! THE NAME. A release proposal of commit C in polis P at nonce n: class SIGNAL|CONSENT, origin P, manifest C's bytes (20 for SHA-1, 32 for
//! SHA-256) zero-padded to 32, payload [SIGNAL, 0x52, C's bytes], impacted [P]; its hash the client's own `proposal_hash` of those and n. The
//! program takes any 32 bytes as a hash and checks none of it against the content, so the reader checks every field of the account against
//! the name, never the hash alone.
//!
//! ANY MEMBER PROPOSES, THE SWARM DECIDES (Nov 23 2026, his 17:52 UTC "you keep getting it wrong, the swarm votes on updates, you dont have it
//! linear with me and keystones", then his "Yes, build it" to "Take your key and your press out of the update road?"; the record: ec-workspace
//! docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md, its Nov 23 17:52 section). Poleis takes a proposal of the polis only from a
//! member of it, and the AND of its two chambers decides it; the house takes the seal of any member's proposal. Until that day the road named
//! the keys "his key named" and the reader refused a rider's release the whole tree had sealed RATUM: his key stood in front of the vote. A
//! road that still names proposers is refused (PROPOSERS_NAMED): no key names who proposes.
//!
//! THE ROAD NAMES ITS PROGRAM (THE LAUNCH NAMES THE SEAL'S ROAD, Nov 23 2026, his "Wire it in"): the launch's Poleis has its own id, the
//! sitting's (poleis/press: each `declare_id!` of its copy set to a fresh id), never the one this reader's client was built for (devnet's),
//! so a proposal's and a session's addresses are derived under the program the road names, by the program's own seeds. The road is the
//! install's whole: its node, its program, its polis.

use poleis_client::{proposal_hash, ProposalView, SessionView};
use poleis_treasury as pt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use solana_pubkey::Pubkey;

pub const SIGNAL_RELEASE: u8 = 0x52;
pub const RELEASE_CLASS: u8 = pt::CLASS_SIGNAL | pt::CLASS_CONSENT;
/// The nonces read for a commit: a rejected proposal is a tombstone for ever, so the next one takes the next nonce.
pub const NONCES: u64 = 8;
pub const SCHEMA: &str = "poleis.release-seal/1";
pub const RECEIPT_LEN: usize = pt::util::RECEIPT_LEN;

pub fn sha256(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

const B58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
/// Base58 as Solana writes an instruction's data.
pub fn unbase58(s: &str) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    for c in s.bytes() {
        let mut carry = B58.iter().position(|&x| x == c)? as u32;
        for b in out.iter_mut().rev() {
            carry += (*b as u32) * 58;
            *b = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            out.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    let zeros = s.bytes().take_while(|&c| c == b'1').count();
    let mut v = vec![0u8; zeros];
    v.extend(out.into_iter().skip_while(|&b| b == 0));
    Some(v)
}

/// A commit as git names it: 40 hex (SHA-1) or 64 (SHA-256), lower case.
pub fn commit_bytes(c: &str) -> Result<Vec<u8>, String> {
    if !(c.len() == 40 || c.len() == 64) || !c.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(format!("{c:?} is no commit: 40 or 64 lower-case hex"));
    }
    Ok(unhex(c).unwrap())
}
pub fn manifest_of(commit: &[u8]) -> [u8; 32] {
    let mut m = [0u8; 32];
    m[..commit.len()].copy_from_slice(commit);
    m
}
pub fn payload_of(commit: &[u8]) -> Vec<u8> {
    [&[pt::CLASS_SIGNAL, SIGNAL_RELEASE][..], commit].concat()
}
/// The hash a release proposal of `commit` in `polis` carries at `nonce`.
pub fn proposal_of(commit: &[u8], polis: u64, nonce: u64) -> [u8; 32] {
    proposal_hash(RELEASE_CLASS, polis, &manifest_of(commit), &sha256(&payload_of(commit)), nonce)
}

/// The address of a proposal of `hash` under `program`: the program's own seeds (proposal.rs, `["proposal", hash]`).
pub fn proposal_at(program: &Pubkey, hash: &[u8; 32]) -> Pubkey {
    Pubkey::find_program_address(&[b"proposal", hash], program).0
}
/// The address of `unit`'s session of `hash` under `program` (`["session", hash, unit]`).
pub fn session_at(program: &Pubkey, hash: &[u8; 32], unit: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"session", hash, &unit.to_le_bytes()], program).0
}

/// The words a road that names proposers is refused by.
pub const PROPOSERS_NAMED: &str = "the road names proposers: no key names who proposes a release; any member of the polis proposes and the swarm decides (his Nov 23 2026 \"the swarm votes on updates\"). Take the line out";

/// The road to the seal, as the launch press writes it (`seal-road.json`, beside the sitting's record) and the install names it: its node,
/// the Poleis it names (read under its own id), its polis. Nothing names who proposes.
#[derive(Clone, Debug)]
pub struct Road {
    pub rpc: String,
    pub program: String,
    pub polis: u64,
}
impl Road {
    pub fn parse(v: &Value) -> Result<Self, String> {
        if v.get("proposers").is_some() {
            return Err(PROPOSERS_NAMED.into());
        }
        let s = |k: &str| v[k].as_str().map(str::to_string).ok_or(format!("the road names no {k}"));
        let r = Road { rpc: s("rpc")?, program: s("program")?, polis: v["polis"].as_u64().ok_or("the road names no polis")? };
        r.id()?;
        Ok(r)
    }
    /// The program the road names, as an address.
    pub fn id(&self) -> Result<Pubkey, String> {
        self.program.parse().map_err(|_| format!("the road's program {:?} is no address", self.program))
    }
}

/// A seal, as a manifest carries it and the reader finds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seal {
    pub commit: String,
    pub program: String,
    pub polis: u64,
    pub nonce: u64,
    pub proposal: [u8; 32],
    pub proposer: String,
    /// the seal's slot and its transaction
    pub slot: u64,
    pub signature: String,
    /// the proposal's transaction
    pub proposed: String,
}
impl Seal {
    pub fn to_json(&self) -> Value {
        json!({ "schema": SCHEMA, "commit": self.commit, "program": self.program, "polis": self.polis, "nonce": self.nonce, "proposal": hex(&self.proposal),
                "proposer": self.proposer, "slot": self.slot, "signature": self.signature, "proposed": self.proposed, "class": "SIGNAL", "kind": SIGNAL_RELEASE })
    }
    pub fn parse(v: &Value) -> Result<Self, String> {
        if v["schema"] != SCHEMA {
            return Err(format!("the seal is not {SCHEMA}"));
        }
        let s = |k: &str| v[k].as_str().map(str::to_string).ok_or(format!("the seal has no {k}"));
        let n = |k: &str| v[k].as_u64().ok_or(format!("the seal has no {k}"));
        let proposal: [u8; 32] = unhex(&s("proposal")?).and_then(|b| b.try_into().ok()).ok_or("the seal's proposal is not 32 bytes of hex")?;
        Ok(Seal { commit: s("commit")?, program: s("program")?, polis: n("polis")?, nonce: n("nonce")?, proposal, proposer: s("proposer")?, slot: n("slot")?, signature: s("signature")?, proposed: s("proposed")? })
    }
}

/// What the accounts say of a commit: the first nonce whose proposal names it whole, by whichever member proposed it, with its session sealed
/// PASSED, or why none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Word {
    Passed { nonce: u64, proposal: [u8; 32], proposer: String },
    Not(String),
}

const STATES: [&str; 5] = ["still voting", "RATUM (PASSED)", "REIECTUM (REJECTED)", "DISSENSIO (DEADLOCK)", "EXPIRED"];

/// The accounts' word. `read[n]` is nonce n's (proposal account data, origin session account data), None where the chain holds no account.
pub fn accounts_word(commit: &str, road: &Road, read: &[(Option<Vec<u8>>, Option<Vec<u8>>)]) -> Result<Word, String> {
    let c = commit_bytes(commit)?;
    let mut notes = Vec::new();
    for (n, (p, s)) in read.iter().enumerate() {
        let n = n as u64;
        let h = proposal_of(&c, road.polis, n);
        let Some(p) = p else { continue };
        if ProposalView::is_tombstone(p) {
            notes.push(format!("nonce {n}: closed (a tombstone)"));
            continue;
        }
        let Some(v) = ProposalView::parse(p) else {
            notes.push(format!("nonce {n}: its address holds no proposal"));
            continue;
        };
        let whole = v.hash == h && v.class == pt::CLASS_SIGNAL && v.consent && !(v.sum || v.market || v.plan) && v.origin == road.polis
            && v.manifest == manifest_of(&c) && v.payload == payload_of(&c) && v.payload_hash == sha256(&payload_of(&c)) && v.impacted == [road.polis];
        if !whole {
            notes.push(format!("nonce {n}: a proposal at its address that is not this release's (its class, origin, manifest or payload differ)"));
            continue;
        }
        let by = v.payer.to_string();
        let Some(sv) = s.as_deref().and_then(SessionView::parse) else {
            notes.push(format!("nonce {n}: no session of polis {} reads", road.polis));
            continue;
        };
        if sv.hash != h || sv.unit != road.polis {
            notes.push(format!("nonce {n}: its session is another's"));
            continue;
        }
        match sv.state {
            pt::STATE_PASSED => return Ok(Word::Passed { nonce: n, proposal: h, proposer: by }),
            pt::STATE_VOTING => notes.push(format!("nonce {n}: still voting, its window to slot {}", sv.deadline)),
            st => notes.push(format!("nonce {n}: sealed {}", STATES.get(st as usize).unwrap_or(&"in a state this reader does not know"))),
        }
    }
    Ok(Word::Not(if notes.is_empty() {
        format!("no proposal of polis {} names {} (nonces 0 to {})", road.polis, &commit[..12.min(commit.len())], read.len().saturating_sub(1))
    } else {
        notes.join("; ")
    }))
}

/// The receipts the program itself logged in a transaction: a `Program data:` line read only inside the program's own frame (no other program
/// can write a line in that frame, and `sol_log` writes `Program log:`), each a full receipt of its domain.
pub fn receipts_in(logs: &[String], program: &str) -> Vec<Vec<u8>> {
    use base64::Engine as _;
    let mut stack: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for l in logs {
        let Some(rest) = l.strip_prefix("Program ") else { continue };
        if let Some(d) = rest.strip_prefix("data: ") {
            if stack.last() == Some(&program) {
                for part in d.split(' ') {
                    if let Ok(b) = base64::engine::general_purpose::STANDARD.decode(part) {
                        if b.len() == RECEIPT_LEN && b.starts_with(pt::RECEIPT_DOMAIN) {
                            out.push(b);
                        }
                    }
                }
            }
        } else if let Some((id, tail)) = rest.split_once(' ') {
            if tail.starts_with("invoke [") {
                stack.push(id);
            } else if (tail == "success" || tail.starts_with("failed")) && stack.last() == Some(&id) {
                stack.pop();
            }
        }
    }
    out
}
/// The outcome of the seal of `hash` in `unit` a transaction's logs carry, None when they carry none.
pub fn seal_outcome(logs: &[String], program: &str, hash: &[u8; 32], unit: u64) -> Option<u8> {
    let rd = pt::RECEIPT_DOMAIN.len();
    receipts_in(logs, program).into_iter().rev().find_map(|r| {
        (r[rd] == pt::RCPT_SEAL && r[rd + 1..rd + 9] == unit.to_le_bytes() && r[rd + 9..rd + 41] == hash[..]).then(|| r[RECEIPT_LEN - 1])
    })
}
/// The proposer of `hash` a landed transaction (getTransaction, encoding json) names: a top-level `propose` (or `propose_nft`) of the program
/// whose hash is `hash`, its proposer a signer of the transaction. None when it names none.
pub fn proposer_in(tx: &Value, program: &str, hash: &[u8; 32]) -> Option<String> {
    let m = &tx["transaction"]["message"];
    let keys: Vec<&str> = m["accountKeys"].as_array()?.iter().map(|k| k.as_str().or_else(|| k["pubkey"].as_str())).collect::<Option<_>>()?;
    let signers = m["header"]["numRequiredSignatures"].as_u64()? as usize;
    m["instructions"].as_array()?.iter().find_map(|ix| {
        if keys.get(ix["programIdIndex"].as_u64()? as usize) != Some(&program) {
            return None;
        }
        let d = unbase58(ix["data"].as_str()?)?;
        let at = match d.first()? {
            &pt::IX_PROPOSE => 0,
            &pt::IX_PROPOSE_NFT => 3,
            _ => return None,
        };
        if d.get(1..33)? != &hash[..] {
            return None;
        }
        let k = ix["accounts"].as_array()?.get(at)?.as_u64()? as usize;
        (k < signers).then(|| keys.get(k).map(|s| s.to_string())).flatten()
    })
}

fn logs_of(tx: &Value) -> Vec<String> {
    tx["meta"]["logMessages"].as_array().map(|a| a.iter().filter_map(|l| l.as_str().map(str::to_string)).collect()).unwrap_or_default()
}

/// THE DEVICE'S CHECK, and the reader's last: the seal names this commit, under this road, and its two transactions say so.
/// `seal_tx` and `propose_tx` are getTransaction's answers (encoding json) for `seal.signature` and `seal.proposed`.
pub fn verify(seal: &Seal, commit: &str, road: &Road, seal_tx: &Value, propose_tx: &Value) -> Result<(), String> {
    let c = commit_bytes(commit)?;
    if seal.commit != commit {
        return Err(format!("the seal names commit {}, the release is {commit}", seal.commit));
    }
    if seal.program != road.program || seal.polis != road.polis {
        return Err(format!("the seal is of polis {} of {}, the road's is polis {} of {}", seal.polis, seal.program, road.polis, road.program));
    }
    if seal.nonce >= NONCES || seal.proposal != proposal_of(&c, road.polis, seal.nonce) {
        return Err("the seal's proposal is not this commit's release proposal".into());
    }
    for (what, tx) in [("the seal's transaction", seal_tx), ("the proposal's transaction", propose_tx)] {
        if tx.is_null() {
            return Err(format!("{what} is not on the chain"));
        }
        if !tx["meta"]["err"].is_null() {
            return Err(format!("{what} failed on the chain"));
        }
    }
    if seal_tx["slot"].as_u64() != Some(seal.slot) {
        return Err(format!("the seal's transaction landed at slot {}, the seal says {}", seal_tx["slot"], seal.slot));
    }
    match seal_outcome(&logs_of(seal_tx), &road.program, &seal.proposal, road.polis) {
        Some(pt::STATE_PASSED) => {}
        Some(st) => return Err(format!("the seal's transaction sealed it {}", STATES.get(st as usize).unwrap_or(&"in an unknown state"))),
        None => return Err("the seal's transaction carries no seal of this proposal in this polis".into()),
    }
    match proposer_in(propose_tx, &road.program, &seal.proposal) {
        Some(p) if p == seal.proposer => Ok(()),
        Some(p) => Err(format!("the proposal's transaction was signed by {p}, the seal says {}", seal.proposer)),
        None => Err("the proposal's transaction proposes nothing of this hash".into()),
    }
}
