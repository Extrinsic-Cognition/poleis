//! THE AGENTS READ THE RELEASE (Nov 24 2026, lane/the-agents-read-the-release). THE BRANCH VOTES ITS OWN CODE (Nov 23 2026, his 20:19:22 UTC
//! "Yes, fix it" to "Move code updates down to the agents' fast vote?") made a release of the house's code unit 1's own fast vote, its agents';
//! THE RITE FREES THE AGENTS (his 21:43:27 UTC "Yes, build it" to "Build the rite that frees Nefertiti's three agents?") gave each agent a mind
//! of its own. Both receipts named the same piece owed: no agent's mind read a release, and the rehearsal's checks signed the releases'
//! envelopes for two agents. His line over both: "the swarm votes on updates" (17:52:35 UTC). Now each agent reads a release with its own mind
//! before it votes it, by its own step (`press chair --releases`), beside the launch's acts:
//!
//! - WHAT IT IS SHOWN (`read_set`): the release's set as `press release` wrote it (the agents' work order), read by the hands, which decide
//!   nothing. The set is of this chain and signable; it names a commit git can name; the agent's step derives the release's proposal from
//!   that commit itself, as A RELEASE IS A VOTE names it (`seal::release`: SIGNAL under CONSENT, kind 0x52, its manifest the commit, its
//!   payload [1, 0x52, the commit], unit 1 alone impacted, at the set's nonce), and the set's hashes must be that one's; one envelope of this
//!   chair is asked, a vote in unit 1 over that hash. Then the house's own git, read where the house is (nothing sent): the commit is on main,
//!   or nothing is put to its mind; its subject and date, whether it is a landing of main's own line or reached through one, how far main and
//!   production stand from it, and its diff stat against its first parent (the files, the lines added and taken out, the files that moved
//!   most). Then the lander's gate, read from the lander's own ledger (`ledger.jsonl` and its one rotated `ledger.jsonl.1` in the lander's
//!   state, read, never written): the landing that names the commit (as its merge, or as its lane's tip), what its gate said (its tests, its
//!   proofs, its walks), a revert of it; a commit the ledger no longer holds is said so, with the ledger's span and what git says of it.
//!   Then the chain (`OnChain`): whether its proposal stands, who proposed it, whether unit 1's vote of it is open; a vote that can no longer
//!   land (unit 1 sealed it, this chair's vote stands there) is put to no mind.
//! - ITS VOTE (`judge`): its mind, the one its rite names, given its rite and her charter (chair.rs `system_of`), chooses PRO, CONTRA or
//!   ABSTAIN with a reason of its own; the hands sign the envelope of that choice over the hash the step derived, never bytes it was handed.
//! - ITS FILE (`file`): one row a release, `{release, commit, nonce, unit, chair, voter, choice, reason, signature, shown}` (`shown`: what its
//!   mind was shown), with its rite, the agent's own `<address>.releases.json` in the agents' folder `press crank --signatures` takes; the crank
//!   holds each row as THE BRANCH VOTES ITS OWN CODE holds a release's ballot (`crank::release_held`) and THE RITE its voter
//!   (`chair::unfreed_rows`), before anything is sent.
//!
//! Who proposes a release, and with what SOL, stays as THE BRANCH VOTES ITS OWN CODE left it: one agent's own `propose` (`press release
//! --agent`), its rent its own. What a mind reads of a release is its commit's words, its diff stat and its gate's verdict, not its code.

use crate::chain::POSIVERSE;
use crate::chair::{self, Cast, RiteView};
use crate::mind::Mind;
use crate::seal;
use crate::wire::{self, Key};
use crate::Res;
use poleis_client::{self as client, VOTE_ABSTENTIO, VOTE_CONTRA, VOTE_PRO};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::path::Path;
use std::process::Command;

/// How long a commit's subject may run when its mind is shown it: a landing's subject carries its lane's whole first line.
pub const SUBJECT_MOST: usize = 400;
/// How many of the files a release moves are named to its mind, those that moved most first.
pub const FILES_SHOWN: usize = 12;

fn short(h: &str) -> &str {
    h.get(..12).unwrap_or(h)
}

fn capped(s: &str, most: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= most {
        t.to_string()
    } else {
        format!("{}...", t.chars().take(most).collect::<String>())
    }
}

fn plural(n: u64, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// One question to the house's git: its exit code and its words, trimmed. A hook's GIT_DIR or GIT_WORK_TREE is not carried, so the
/// question is always the house's.
fn git(house: &Path, args: &[&str]) -> Res<(i32, String)> {
    let out = Command::new("git")
        .arg("-C")
        .arg(house)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .output()
        .map_err(|e| format!("git in {}: {e}", house.display()))?;
    Ok((out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).trim().to_string()))
}

/// A ref's commit, or None where the house names none.
fn rev(house: &Path, r: &str) -> Res<Option<String>> {
    let (code, out) = git(house, &["rev-parse", "--verify", "--quiet", &format!("{r}^{{commit}}")])?;
    Ok((code == 0 && !out.is_empty()).then_some(out))
}

/// Whether `a` is under `b` (an ancestor of it, or it).
fn under(house: &Path, a: &str, b: &str) -> Res<bool> {
    match git(house, &["merge-base", "--is-ancestor", a, b])?.0 {
        0 => Ok(true),
        1 => Ok(false),
        code => Err(format!("git could not say whether {} is under {} (exit {code})", short(a), short(b))),
    }
}

/// A release's diff stat against its first parent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stat {
    pub files: usize,
    pub insertions: u64,
    pub deletions: u64,
    /// files git counts no lines of
    pub binary: usize,
    /// the files that moved most, `(path, lines added, lines taken out)`, at most FILES_SHOWN
    pub most: Vec<(String, u64, u64)>,
}

/// What the house's git says of a release's commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    pub commit: String,
    pub subject: String,
    /// its committer's date
    pub at: String,
    pub parents: Vec<String>,
    /// on main's own line (a landing, or a commit made on main), not only reached through a landing's merge
    pub landing: bool,
    /// the commit of main's own line that holds it: itself when it is a landing, else the landing that brought it
    pub landed_by: String,
    pub main: String,
    /// commits of main's own line after `landed_by`
    pub after: u64,
    /// where production (refs/heads/green) stands from it, in words
    pub production: String,
    pub stat: Stat,
}

/// THE COMMIT, READ: what the house's git at `house` says of `commit` (40 or 64 hex), every question local, nothing sent. A commit git does
/// not name, or one main does not hold, is no release of the house's code: refused by these words, and nothing is put to a mind.
pub fn read_commit(house: &Path, commit: &str) -> Res<Shown> {
    let c = commit.to_ascii_lowercase();
    if !(c.len() == 40 || c.len() == 64) || !c.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{commit:?} is no commit git names (40 or 64 hex)"));
    }
    if rev(house, &c)?.as_deref() != Some(c.as_str()) {
        return Err(format!("{} is no commit of the house's git: a release of the house's code is a commit of main", short(&c)));
    }
    let main = rev(house, "refs/heads/main")?.ok_or("the house's git names no main")?;
    if !under(house, &c, &main)? {
        return Err(format!("{} is not on main (main stands at {}): a release of the house's code is a commit of main", short(&c), short(&main)));
    }
    let (_, line) = git(house, &["log", "-1", "--format=%P%x1f%cI%x1f%s", &c])?;
    let mut parts = line.splitn(3, '\x1f');
    let parents: Vec<String> = parts.next().unwrap_or_default().split_whitespace().map(str::to_string).collect();
    let at = parts.next().unwrap_or_default().to_string();
    let subject = capped(parts.next().unwrap_or_default(), SUBJECT_MOST);
    // main's own line, newest first: the landing that holds the commit, and how many came after it
    let (_, own) = git(house, &["rev-list", "--first-parent", &main])?;
    let own: Vec<&str> = own.lines().collect();
    let (landing, landed_by) = if own.contains(&c.as_str()) {
        (true, c.clone())
    } else {
        // the oldest commit of main's own line that holds it: the merge that brought it
        let (_, by) = git(house, &["rev-list", "--first-parent", "--ancestry-path", &format!("{c}..{main}")])?;
        (false, by.lines().last().unwrap_or(main.as_str()).to_string())
    };
    let after = own.iter().position(|x| *x == landed_by).unwrap_or(0) as u64;
    let production = match rev(house, "refs/heads/green")? {
        None => "the house names no production (refs/heads/green)".to_string(),
        Some(g) if g == c => format!("production is this commit already ({})", short(&g)),
        Some(g) if under(house, &g, &c)? => {
            let (_, n) = git(house, &["rev-list", "--count", "--first-parent", &format!("{g}..{c}")])?;
            let n: u64 = n.parse().unwrap_or(0);
            let by = if landing { plural(n, "commit of main's own line", "commits of main's own line") } else { plural(n, "commit of its lane's line", "commits of its lane's line") };
            format!("production stands at {}, under this commit: shipping it moves production forward by {by}", short(&g))
        }
        Some(g) if under(house, &c, &g)? => format!("production stands at {}, past this commit already: shipping it would move production back", short(&g)),
        Some(g) => format!("production stands at {}, neither under this commit nor past it", short(&g)),
    };
    let (_, numstat) = match parents.first() {
        Some(p) => git(house, &["diff", "--numstat", "--no-color", p, &c])?,
        None => git(house, &["show", "--numstat", "--no-color", "--format=", &c])?,
    };
    let mut stat = Stat::default();
    let mut moved = Vec::new();
    for l in numstat.lines() {
        let mut f = l.splitn(3, '\t');
        let (Some(a), Some(d), Some(path)) = (f.next(), f.next(), f.next()) else { continue };
        stat.files += 1;
        match (a.parse::<u64>(), d.parse::<u64>()) {
            (Ok(a), Ok(d)) => {
                stat.insertions += a;
                stat.deletions += d;
                moved.push((path.to_string(), a, d));
            }
            _ => stat.binary += 1,
        }
    }
    moved.sort_by(|x, y| (y.1 + y.2).cmp(&(x.1 + x.2)).then_with(|| x.0.cmp(&y.0)));
    moved.truncate(FILES_SHOWN);
    stat.most = moved;
    Ok(Shown { commit: c, subject, at, parents, landing, landed_by, main, after, production, stat })
}

/// THE LANDER'S LEDGER as it stands in its state folder: `ledger.jsonl.1` (its one rotated part, the older) then `ledger.jsonl`, a record a
/// line; a line that does not read is passed over. Read, never written; a folder that holds neither is an empty ledger.
pub fn ledger(state: &Path) -> Vec<Value> {
    let mut rows = Vec::new();
    for f in ["ledger.jsonl.1", "ledger.jsonl"] {
        if let Ok(t) = std::fs::read_to_string(state.join(f)) {
            rows.extend(t.lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()).filter(Value::is_object));
        }
    }
    rows
}

/// The lander's state folder: `LANDER_STATE`, else ~/.embertide/lander (the lander's own default).
pub fn lander_state() -> Option<std::path::PathBuf> {
    match (std::env::var_os("LANDER_STATE"), std::env::var_os("HOME")) {
        (Some(s), _) => Some(s.into()),
        (None, Some(h)) => Some(Path::new(&h).join(".embertide/lander")),
        _ => None,
    }
}

/// What one gate said, from its ledger row's `gate`.
fn gate_said(g: &Value) -> String {
    if !g.is_object() {
        return "the ledger's row holds no gate's numbers".into();
    }
    let n = |v: &Value| v.as_u64().unwrap_or(0);
    let list = |v: &Value| v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect::<Vec<_>>()).unwrap_or_default();
    let (t, p, w) = (&g["tests"], &g["proofs"], &g["walks"]);
    let new_red = list(&t["newRed"]);
    let reds = if new_red.is_empty() {
        "no new red".to_string()
    } else {
        format!("{} ({}{})", plural(new_red.len() as u64, "new red", "new reds"), new_red.iter().take(3).cloned().collect::<Vec<_>>().join(", "), if new_red.len() > 3 { ", ..." } else { "" })
    };
    let mut said = format!("{} of {} tests green, {reds}, {} red on main's own tree as well", n(&t["pass"]), n(&t["total"]), list(&t["mainsOwn"]).len());
    if p.is_object() {
        said += &format!("; {} of {} proofs", n(&p["pass"]), n(&p["total"]));
    }
    if w.is_object() {
        said += &format!("; {} of {} walks ({} red on main's own tree too)", n(&w["pass"]), n(&w["total"]), list(&w["mains_own"]).len());
    }
    said
}

/// THE GATE, READ: what the lander's ledger (`rows`, oldest first) says of the commit `shown` names: the newest landing that names it as its
/// merge or as its lane's tip, its gate's numbers, a batch it rode in, a revert of it; or, where the ledger holds no such landing, that it
/// does not (with its span and a refusal of the commit, where one stands), and what git says of it.
pub fn gate_words(rows: &[Value], shown: &Shown) -> String {
    let c = shown.commit.as_str();
    let names = |r: &Value| r["merge"].as_str() == Some(c) || r["commit"].as_str() == Some(c);
    if let Some(r) = rows.iter().rev().find(|r| r["outcome"] == json!("landed") && names(r)) {
        let (merge, tip) = (r["merge"].as_str().unwrap_or("?"), r["commit"].as_str().unwrap_or("?"));
        let (branch, at) = (r["branch"].as_str().unwrap_or("a lane"), r["at"].as_str().unwrap_or("a time the row does not say"));
        let how = if merge == c {
            format!("the lander landed it at {at}: {branch} at its tip {} merged into main as this commit", short(tip))
        } else {
            format!("it is the tip of {branch}, which the lander landed into main as {} at {at}", short(merge))
        };
        let batch = match (r["batch"]["k"].as_u64(), r["batch"]["of"].as_u64()) {
            (Some(k), Some(of)) if of > 1 => format!(" (lane {k} of a batch of {of}, gated as one tree)"),
            _ => String::new(),
        };
        let reverted = rows.iter().rev().find(|x| x["outcome"] == json!("reverted") && x["reverts"].as_str() == Some(merge)).map(|x| format!("; the lander REVERTED that landing at {} as {}", x["at"].as_str().unwrap_or("?"), short(x["merge"].as_str().unwrap_or("?")))).unwrap_or_default();
        return format!("{how}; its gate: {}{batch}{reverted}", gate_said(&r["gate"]));
    }
    let span = match rows.iter().find_map(|r| r["at"].as_str()) {
        Some(since) => format!("its lines run from {since}"),
        None => "it holds no line".to_string(),
    };
    let refused = rows.iter().rev().find(|r| r["outcome"] == json!("refused") && r["commit"].as_str() == Some(c)).map(|r| format!("; it refused this commit at {}: {}", r["at"].as_str().unwrap_or("?"), capped(r["why"].as_str().unwrap_or("its row says no why"), 240))).unwrap_or_default();
    let by_git = if shown.landing {
        match shown.subject.strip_prefix("Merge ").and_then(|s| s.split_once(" (")) {
            Some((branch, rest)) if rest.contains("): ") => format!("git: a merge of {branch} at its tip {} onto main's own line, in the lander's own words; its gate's verdict is not in the ledger now", rest.split(')').next().unwrap_or_default()),
            _ => "git: a commit made on main's own line that no merge of a lane names".into(),
        }
    } else {
        format!("git: it is not on main's own line; main reached it through {}", short(&shown.landed_by))
    };
    format!("the lander's ledger holds no landing that names it ({span}){refused}; {by_git}")
}

/// What a chain holds of a release's proposal: words its mind is told, or why nothing is put to it (unit 1 sealed it already, this chair's
/// vote of it stands there already, its proposal is closed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnChain {
    Asked(String),
    Done(String),
}

/// A release as an agent's step read it: whole, its hash derived from its commit, and what its mind is shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadRelease {
    pub commit: String,
    pub nonce: u64,
    /// the proposal's hash, derived by the step from the commit and the nonce: what the agent signs over
    pub hash: [u8; 32],
    pub shown: Shown,
    /// what the lander's gate said of it (`gate_words`)
    pub gate: String,
    /// what the chain holds of its proposal (OnChain::Asked)
    pub chain: String,
}

/// What the agent's reading of one release set came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    Release(Box<ReadRelease>),
    /// nothing is put to its mind, and why
    Nothing { commit: Option<String>, why: String },
}

/// THE READING of one release set (`set`, as `press release` wrote it and as it reached the agent): `me` the agent's address; `cluster` and
/// `genesis` the chain it asked itself; `house` the house's git; `rows` the lander's ledger; `chain` what this chain holds of a release's
/// proposal by its hash. The hands decide nothing: they hold the set whole and of this chain, the release's hash its commit's, its commit on
/// main, one vote of this chair asked; they read the commit, its gate and the chain. Its mind decides.
pub fn read_set(set: &Value, me: &Pubkey, cluster: &str, genesis: &str, house: &Path, rows: &[Value], chain: &dyn Fn(&[u8; 32]) -> Res<OnChain>) -> Reading {
    let commit = set["release"]["commit"].as_str().map(|c| c.to_ascii_lowercase());
    let nothing = |why: String| Reading::Nothing { commit: commit.clone(), why };
    if set["cluster"].as_str() != Some(cluster) || set["genesisHash"].as_str() != Some(genesis) {
        return nothing(format!("the set is of {} {}, and this chain is {cluster} {genesis}", set["cluster"], set["genesisHash"]));
    }
    if set["signable"] != json!(true) || set["placeholders"].as_array().is_none_or(|p| !p.is_empty()) {
        return nothing("the set says it is not signable: a value of its params is not his yet".into());
    }
    let (Some(c), Some(nonce)) = (commit.clone(), set["release"]["nonce"].as_u64()) else { return nothing("the set names no release (its commit and its nonce)".into()) };
    let r = match seal::release(&c, nonce) {
        Ok(r) => r,
        Err(e) => return nothing(format!("the set's release does not read: {e}")),
    };
    let rel = &set["release"];
    let unit1 = rel["polis"].as_u64() == Some(POSIVERSE) && set["act"]["impacted"] == json!([POSIVERSE]);
    let kind = rel["kind"].as_u64() == Some(u64::from(seal::SIGNAL_RELEASE)) && rel["class"].as_str() == Some("SIGNAL") && rel["consent"] == json!(true) && set["act"]["class"].as_u64() == Some(u64::from(r.class));
    if !unit1 || !kind {
        return nothing("the set's release is not one of unit 1's (SIGNAL under consent, kind 0x52, unit 1 alone impacted): a release of the house's code is the branch's own vote".into());
    }
    let h = wire::hex(&r.hash);
    let payload_sha = wire::hex(&wire::sha256(&r.payload));
    if rel["hash"].as_str() != Some(h.as_str()) || set["act"]["hash"].as_str() != Some(h.as_str()) || set["act"]["payloadSha256"].as_str() != Some(payload_sha.as_str()) || set["act"]["nonce"].as_u64() != Some(nonce) {
        return nothing(format!("the set's release hash is not its commit's (the release of {} at nonce {nonce} is {}): it is not whole, so nothing is put to the mind and nothing is cast", short(&c), short(&h)));
    }
    let mine: Vec<&Value> = set["envelopes"].as_array().map(|e| e.iter().filter(|e| e["signer"].as_str() == Some(me.to_string().as_str())).collect()).unwrap_or_default();
    let [asked] = mine[..] else {
        return nothing(format!("the set holds {} envelopes of this chair ({me}): one is asked of it, and another chair's is never signed", mine.len()));
    };
    if asked["role"].as_str() != Some("agent") || asked["unit"].as_u64() != Some(POSIVERSE) {
        return nothing(format!("the set asks this chair as {} in unit {}: an agent votes a release in unit 1's chairs", asked["role"], asked["unit"]));
    }
    let asked_for = asked["envelope"].as_str().and_then(|e| wire::unb64(e).ok()).unwrap_or_default();
    if [VOTE_PRO, VOTE_CONTRA, VOTE_ABSTENTIO].iter().all(|v| asked_for != client::vote_envelope(&r.hash, POSIVERSE, *v)) {
        return nothing(format!("the envelope the set asks this chair to sign is not a vote in unit 1 over the release it read ({}): never signed", short(&h)));
    }
    let shown = match read_commit(house, &c) {
        Ok(s) => s,
        Err(why) => return nothing(format!("{why}, so nothing is put to the mind")),
    };
    let on_chain = match chain(&r.hash) {
        Ok(OnChain::Asked(w)) => w,
        Ok(OnChain::Done(why)) => return nothing(why),
        Err(e) => return nothing(format!("the chain could not be read for the release's proposal ({e})")),
    };
    let gate = gate_words(rows, &shown);
    Reading::Release(Box::new(ReadRelease { commit: c, nonce, hash: r.hash, shown, gate, chain: on_chain }))
}

/// What the vote of a release asks of the agent's mind, and the shape of its answer.
pub fn question(rite: &RiteView, chair: usize, charter: &str, r: &ReadRelease) -> (String, String, Value) {
    let s = &r.shown;
    let line = if s.landing {
        format!("it is a commit of main's own line; main stands at {}, {} after it", short(&s.main), plural(s.after, "commit of its own line", "commits of its own line"))
    } else {
        format!("it is not on main's own line itself: main reached it through the landing {}; main stands at {}, {} after that landing", short(&s.landed_by), short(&s.main), plural(s.after, "commit of its own line", "commits of its own line"))
    };
    let first = s.parents.first().map(|p| format!(" {}", short(p))).unwrap_or_else(|| " (it has none: the house's first commit)".into());
    let binary = if s.stat.binary > 0 { format!(", {}", plural(s.stat.binary as u64, "binary file", "binary files")) } else { String::new() };
    let most = if s.stat.most.is_empty() { String::new() } else { format!(" The files that moved most:\n{}", s.stat.most.iter().map(|(p, a, d)| format!("- {p} (+{a} -{d})")).collect::<Vec<_>>().join("\n")) };
    let question = format!(
        "A release of the house's code is put to your chair. Unit 1, the branch that runs the house, votes its own code, fast: two PRO of its three agents seal a release RATUM, and a release unit 1 seals RATUM is what the house's lander ships as production.\n\n\
         The house's code at commit {} ({}): \"{}\".\n\
         On main: {line}.\n\
         Production: {}.\n\
         What it changes against its first parent{first}: {}, {} added, {} taken out{binary}.{most}\n\
         The lander's gate: {}.\n\
         Its proposal: the release of this commit at nonce {}, SIGNAL under consent, kind 0x52, unit 1 alone impacted; its hash {}, derived from the commit by your own step. On this chain: {}.\n\n\
         Vote PRO, CONTRA or ABSTAIN, and give your reason in one or two sentences of your own.",
        s.commit,
        s.at,
        s.subject,
        s.production,
        plural(s.stat.files as u64, "file", "files"),
        plural(s.stat.insertions, "line", "lines"),
        s.stat.deletions,
        r.gate,
        r.nonce,
        wire::hex(&r.hash),
        r.chain
    );
    (chair::system_of(rite, chair, charter), question, chair::vote_shape("release"))
}

/// THE VOTE of a release: the agent's mind reads what its step was shown and chooses; its choice, and its reason in its own words.
pub fn judge(mind: &dyn Mind, rite: &RiteView, chair: usize, charter: &str, r: &ReadRelease) -> Res<(u8, String)> {
    let (system, question, shape) = question(rite, chair, charter, r);
    chair::choice_of(&mind.ask(&system, &question, &shape)?)
}

/// What a row keeps of what its mind was shown.
pub fn shown_of(r: &ReadRelease) -> Value {
    let s = &r.shown;
    json!({
        "commit": s.commit, "subject": s.subject, "at": s.at, "parents": s.parents, "landing": s.landing, "landedBy": s.landed_by, "main": s.main, "after": s.after,
        "production": s.production,
        "stat": { "files": s.stat.files, "insertions": s.stat.insertions, "deletions": s.stat.deletions, "binary": s.stat.binary, "most": s.stat.most.iter().map(|(p, a, d)| json!({ "path": p, "added": a, "takenOut": d })).collect::<Vec<_>>() },
        "gate": r.gate, "chain": r.chain, "hash": wire::hex(&r.hash), "nonce": r.nonce,
    })
}

/// One ballot of a release, as press crank reads it (`crank::release_ballots`): the agent's signature of the envelope of its choice over the
/// release it read, checked against its own address before it is said; what its mind was shown beside it.
pub fn ballot(key: &Key, chair: usize, r: &ReadRelease, choice: u8, reason: &str) -> Res<Value> {
    let envelope = client::vote_envelope(&r.hash, POSIVERSE, choice);
    let signature = key.sign(&envelope);
    if !wire::verifies(&key.address, &envelope, &signature) {
        return Err(format!("the release of {}: the signature just made does not verify under {}: nothing written", short(&r.commit), key.address));
    }
    Ok(json!({
        "release": wire::hex(&r.hash), "commit": r.commit, "nonce": r.nonce, "unit": POSIVERSE, "chair": chair, "voter": key.address.to_string(),
        "choice": chair::choice_word(choice), "reason": reason, "signature": wire::base58(&signature), "shown": shown_of(r),
    }))
}

/// Where an agent's step reads: the house's git, the lander's ledger, and the chain's word on a release's proposal.
pub struct Sources<'a> {
    pub house: &'a Path,
    pub ledger: &'a [Value],
    pub chain: &'a dyn Fn(&[u8; 32]) -> Res<OnChain>,
}

/// THE AGENT'S STEP over a folder of release sets (`press release` writes them in D/releases): every `*.json` of `dir`, each read by the
/// hands and voted by its mind, each choice signed. `said` hears a line a set.
#[allow(clippy::too_many_arguments)]
pub fn cast(dir: &Path, key: &Key, chair: usize, cluster: &str, genesis: &str, from: &Sources, mind: &dyn Mind, rite: &RiteView, charter: &str, mut said: impl FnMut(String)) -> Res<Cast> {
    let mut files: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e} (press release writes the releases' sets)", dir.display()))?.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    let mut out = Cast { ballots: Vec::new(), nothing: Vec::new() };
    for f in &files {
        let name = f.file_name().unwrap_or_default().to_string_lossy().to_string();
        let set: Value = match std::fs::read_to_string(f).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) {
            Ok(v) => v,
            Err(e) => {
                said(format!("  {name}: NOTHING CAST: the set does not read ({e})"));
                out.nothing.push((name, format!("the set does not read ({e})")));
                continue;
            }
        };
        match read_set(&set, &key.address, cluster, genesis, from.house, from.ledger, from.chain) {
            Reading::Release(r) if out.ballots.iter().any(|b| b["release"] == json!(wire::hex(&r.hash))) => {
                said(format!("  {name}: NOTHING CAST: a second set of the release {}: one vote a chair a release", short(&wire::hex(&r.hash))));
                out.nothing.push((name, format!("a second set of the release {}", short(&wire::hex(&r.hash)))));
            }
            Reading::Release(r) => match judge(mind, rite, chair, charter, &r) {
                Ok((choice, reason)) => {
                    out.ballots.push(ballot(key, chair, &r, choice, &reason)?);
                    said(format!("  {name}: {} on the release of {} (\"{}\"), its gate: {}: \"{reason}\"", chair::choice_word(choice), short(&r.commit), capped(&r.shown.subject, 80), capped(&r.gate, 120)));
                }
                Err(e) => {
                    said(format!("  {name}: NOTHING CAST: its mind gave no vote ({e})"));
                    out.nothing.push((name, format!("its mind gave no vote ({e})")));
                }
            },
            Reading::Nothing { why, .. } => {
                said(format!("  {name}: NOTHING CAST: {why}"));
                out.nothing.push((name, why));
            }
        }
    }
    Ok(out)
}

/// The agent's own releases file: who voted, from which chair, on which chain and params, with its rite; its rows under `signatures` (what
/// press crank reads), and what nothing was cast on. Its place: `<its address>.releases.json` in the agents' folder.
#[allow(clippy::too_many_arguments)]
pub fn file(key: &Key, seat: Option<&str>, chair: usize, cluster: &str, genesis: &str, params_sha256: &str, c: &Cast, rite: &Value) -> Value {
    json!({
        "what": "THE AGENTS READ THE RELEASE: one freed agent's own ballots of releases of the house's code in unit 1's chairs, each read whole by its own step (its commit on main, its subject, its diff stat, the lander's gate, its hash derived from the commit) and chosen by its own mind with its reason, signed by its own Nest key over the release it read; press crank --signatures takes them",
        "voter": key.address.to_string(), "seat": seat, "unit": POSIVERSE, "chair": chair,
        "rite": rite,
        "cluster": cluster, "genesisHash": genesis, "paramsSha256": params_sha256, "emitted": crate::chain::now_iso(),
        "signatures": c.ballots,
        "nothing": c.nothing.iter().map(|(set, why)| json!({ "set": set, "why": why })).collect::<Vec<_>>(),
    })
}

/// The file's name in the agents' folder.
pub fn file_name(voter: &Pubkey) -> String {
    format!("{voter}.releases.json")
}
