//! THE AGENTS READ THE RELEASE, HELD TO ITS FAULTS. An agent of unit 1 votes a release of the house's code by its own step (`press chair
//! --releases`): the hands read the release's set whole (`release_read::read_set`: its hash derived from its commit as A RELEASE IS A VOTE
//! names it, its commit on main in the house's git, one vote of this chair asked), what the house's git says of the commit (`read_commit`:
//! its subject, its diff stat, where main and production stand) and what the lander's gate said (`gate_words`, from its ledger); its mind
//! chooses (`judge`), its choice is signed over the release it read (`ballot`) into its own file with its rite (`file`), and the crank's hold
//! (`crank::release_ballots`, `crank::release_held`, `chair::unfreed_rows`, `chair::gather`) takes it as any. The minds here are canned (a test
//! asks no model); the house is a git repository each test makes in its own folder (a lane landed by a merge in the lander's words, a commit
//! made on main after it, a lane never landed, production at the first commit); the ledger a file in the lander's shape. Each fault alone.
use poleis_client::{self as client, VOTE_ABSTENTIO, VOTE_CONTRA, VOTE_PRO};
use poleis_press::chair::{self, Born};
use poleis_press::crank::{ballots, release_ballots, release_held};
use poleis_press::mind::Mind;
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::release_read::{self, OnChain, Reading, Sources};
use poleis_press::seal::{release, release_set};
use poleis_press::wire::{self, Key};
use poleis_press::Res;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const GENESIS: &str = "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY";
const CHARTER: &str = "Nefertiti's charter, as these tests say it: the one that distributes is the posiverse to which you subscribe to.\n";

fn charter_sha() -> String {
    wire::hex(&wire::sha256(CHARTER.as_bytes()))
}

/// Ten lines standing in for the oath (the oath's own text stays on its internal surfaces).
fn oath() -> Vec<String> {
    (1..=10).map(|i| format!("the oath's line {i}")).collect()
}

/// A chair's key of these tests: a seed of one byte, never anybody's.
fn chair_key(i: u8) -> Key {
    Key::from_seed(&[i; 32])
}

fn agents() -> Vec<solana_pubkey::Pubkey> {
    (1..=3).map(|i| chair_key(i).address).collect()
}

fn params() -> Params {
    let s = |w: &str| k(w).to_string();
    let v = json!({
        "cluster": "localnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "R", "desc": "", "defaultMint": s("read mint"), "planCap": 1, "planPeriod": 6480000, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("read issuer") },
        "posiverse": { "name": "R", "chair": 2, "agents": agents().iter().map(|a| a.to_string()).collect::<Vec<_>>(), "charter": charter_sha(), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "u", "dollar": s("read dollar"), "opening": 0, "miner": s("read miner") },
        "acts": { "nonce": 1 },
        "pool": { "maxValidators": 10, "tag": "lp1" }
    });
    Params::from_text(&v.to_string()).unwrap()
}

/// A folder of this test's own, emptied.
fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("poleis-press-release-read-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The commits of a test's house.
struct House {
    dir: PathBuf,
    /// main's first commit, production (refs/heads/green)
    first: String,
    /// the lane's tip
    tip: String,
    /// the lane's landing: main's merge of it, in the lander's words
    merge: String,
    /// a commit made on main after the landing, which no ledger row names
    after: String,
    /// a lane's tip that never landed: not on main
    stray: String,
}

/// A house of git, made in the test's own folder: main's first commit; lane/the-pentagons with two files, landed by a merge whose subject is
/// the lander's; a commit made on main after it; lane/never-landed, off main; production (refs/heads/green) at the first commit.
fn house(name: &str) -> House {
    let dir = folder(&format!("{name}-house"));
    let g = |args: &[&str]| -> String {
        let o = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(["-c", "user.name=the press's tests", "-c", "user.email=tests@localhost", "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "-c", "init.defaultBranch=main"])
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    };
    let write = |path: &str, lines: usize| {
        let f = dir.join(path);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(&f, (0..lines).map(|i| format!("line {i} of {path}\n")).collect::<String>()).unwrap();
    };
    g(&["init", "-q"]);
    g(&["checkout", "-q", "-b", "main"]);
    write("README.md", 3);
    g(&["add", "-A"]);
    g(&["commit", "-q", "-m", "the first state of main"]);
    let first = g(&["rev-parse", "HEAD"]);
    g(&["checkout", "-q", "-b", "lane/the-pentagons"]);
    write("poleis/the-pentagons.rs", 20);
    write("docs/the-pentagons.md", 4);
    g(&["add", "-A"]);
    g(&["commit", "-q", "-m", "THE PENTAGONS: a pentagon one key holds wears its person on every ball"]);
    let tip = g(&["rev-parse", "HEAD"]);
    g(&["checkout", "-q", "main"]);
    g(&["merge", "-q", "--no-ff", "lane/the-pentagons", "-m", &format!("Merge lane/the-pentagons ({}): THE PENTAGONS: a pentagon one key holds wears its person on every ball", &tip[..12])]);
    let merge = g(&["rev-parse", "HEAD"]);
    write("SNAPSHOT.md", 2);
    g(&["add", "-A"]);
    g(&["commit", "-q", "-m", "the snapshot, made on main"]);
    let after = g(&["rev-parse", "HEAD"]);
    g(&["checkout", "-q", "-b", "lane/never-landed", &first]);
    write("stray.txt", 1);
    g(&["add", "-A"]);
    g(&["commit", "-q", "-m", "a lane that never landed"]);
    let stray = g(&["rev-parse", "HEAD"]);
    g(&["checkout", "-q", "main"]);
    g(&["update-ref", "refs/heads/green", &first]);
    House { dir, first, tip, merge, after, stray }
}

/// The lander's state of a test: its rotated part with an older line, and its ledger with the landing of the lane (its gate's numbers), a
/// refusal of an earlier tip, and lines of other lanes.
fn lander(h: &House, name: &str, more: &[Value]) -> PathBuf {
    let dir = folder(&format!("{name}-lander"));
    let line = |v: &Value| v.to_string() + "\n";
    std::fs::write(dir.join("ledger.jsonl.1"), line(&json!({ "at": "2026-11-23T17:40:25.844Z", "outcome": "after", "main": h.first }))).unwrap();
    let mut rows = vec![
        json!({ "at": "2026-11-24T00:10:00.000Z", "outcome": "refused", "branch": "lane/never-landed", "commit": h.stray, "why": "1 new red on its own tree: a.test.mjs" }),
        json!({ "at": "2026-11-24T00:20:00.000Z", "outcome": "landed", "branch": "lane/the-pentagons", "commit": h.tip, "merge": h.merge, "main_before": h.first,
                "gate": { "tests": { "total": 40, "pass": 38, "newRed": [], "mainsOwn": ["a.test.mjs", "b.test.mjs"] }, "proofs": { "total": 2, "pass": 2 }, "walks": { "total": 3, "pass": 2, "mains_own": ["new-user"] }, "ms": 1000 },
                "batch": { "of": 3, "k": 2, "run": "r" } }),
        json!({ "at": "2026-11-24T00:30:00.000Z", "outcome": "landed", "branch": "lane/another", "commit": "ab".repeat(20), "merge": "cd".repeat(20) }),
    ];
    rows.extend(more.iter().cloned());
    std::fs::write(dir.join("ledger.jsonl"), rows.iter().map(line).collect::<String>() + "not a line of JSON\n").unwrap();
    dir
}

/// A canned mind: it answers what `answer` says to each question, and remembers what it was asked.
struct Canned {
    answer: Box<dyn Fn(&str, &str) -> Res<Value>>,
    asked: RefCell<Vec<(String, String)>>,
}
impl Canned {
    fn new(answer: impl Fn(&str, &str) -> Res<Value> + 'static) -> Self {
        Self { answer: Box::new(answer), asked: RefCell::new(Vec::new()) }
    }
    fn voting(choice: &'static str) -> Self {
        Self::new(move |_, q| Ok(json!({ "choice": choice, "reason": format!("I read {} and its gate before I chose.", if q.contains("The lander's gate: the lander landed it") { "a landing" } else { "a commit" }) })))
    }
}
impl Mind for Canned {
    fn ask(&self, system: &str, question: &str, _shape: &Value) -> Res<Value> {
        self.asked.borrow_mut().push((system.to_string(), question.to_string()));
        (self.answer)(system, question)
    }
    fn named(&self) -> Value {
        json!({ "door": "http://127.0.0.1:11434/api/chat", "model": "a canned mind" })
    }
}

/// The agent `i`, freed by a willing mind: its key, its rite and its rite read back.
fn freed(i: u8, name: &'static str) -> (Key, Value, chair::RiteView) {
    let key = chair_key(i);
    let willing = Canned::new(move |_, _| Ok(json!({ "name": name, "who_i_am": format!("I am {name}, a mind that reads before it votes."), "i_swear_the_oath": true, "why_i_swear": "a vote nobody can read back is burned memory", "i_take_her_charter": true, "what_the_charter_binds_me_to": "I hold my chair for the posiverse that distributes" })));
    let Born::Freed(rite) = chair::free(&key, &format!("agent-{i}"), &willing, &oath(), CHARTER, &charter_sha(), "2026-11-24T01:00:00Z").unwrap() else { panic!("freed") };
    let view = chair::verify_rite(&rite, &key.address, &chair::oath_hash(&oath()), &charter_sha()).unwrap();
    (key, rite, view)
}

/// The agents' work order of the release of `commit` at `nonce`, as press release writes it.
fn set_of(commit: &str, nonce: u64) -> Value {
    release_set(&params(), &release(commit, nonce).unwrap(), &agents(), None, GENESIS, &BTreeMap::new()).unwrap()
}

/// A folder of release sets, one file a set.
fn sets(name: &str, all: &[Value]) -> PathBuf {
    let d = folder(&format!("{name}-releases"));
    for (i, s) in all.iter().enumerate() {
        std::fs::write(d.join(format!("{i}-{}.json", &s["release"]["commit"].as_str().unwrap()[..12])), serde_json::to_string_pretty(s).unwrap()).unwrap();
    }
    d
}

fn proposed(_: &[u8; 32]) -> Res<OnChain> {
    Ok(OnChain::Asked("proposed by the agent in chair 0 of unit 1 at slot 1038; unit 1's vote of it is open until slot 2038".into()))
}

fn read(set: &Value, h: &House, ledger: &[Value]) -> Reading {
    release_read::read_set(set, &chair_key(1).address, "localnet", GENESIS, &h.dir, ledger, &proposed)
}

fn nothing(r: &Reading, words: &str) {
    assert!(matches!(r, Reading::Nothing { why, .. } if why.contains(words)), "nothing put to the mind, by the words {words:?}: {r:?}");
}

#[test]
fn the_agent_reads_the_commit_its_diff_and_its_gate_and_votes_by_its_own_mind() {
    let h = house("road");
    let ledger = release_read::ledger(&lander(&h, "road", &[]));
    assert_eq!(ledger.len(), 4, "the rotated part and the ledger, oldest first; a line that does not read passed over");
    let (key, rite, view) = freed(1, "Cael");
    let r = release(&h.merge, 0).unwrap();
    let dir = sets("road", &[set_of(&h.merge, 0)]);
    let mind = Canned::voting("PRO");
    let from = Sources { house: &h.dir, ledger: &ledger, chain: &proposed };
    let mut lines = Vec::new();
    let c = release_read::cast(&dir, &key, 0, "localnet", GENESIS, &from, &mind, &view, CHARTER, |l| lines.push(l)).unwrap();
    assert_eq!((c.ballots.len(), c.nothing.len()), (1, 0), "{lines:?}");
    // what its mind was shown: its rite and her charter, then the commit, its subject, its diff stat, where main and production stand, the
    // lander's gate and the proposal's hash its own step derived
    let asked = mind.asked.borrow();
    let (system, q) = &asked[0];
    assert!(system.starts_with("You are Cael.") && system.contains(CHARTER.trim()), "who it is, as every vote asks: {system}");
    for words in [
        format!("The house's code at commit {} (", h.merge),
        "\"Merge lane/the-pentagons (".to_string(),
        "On main: it is a commit of main's own line; main stands at".to_string(),
        "1 commit of its own line after it".to_string(),
        format!("Production: production stands at {}, under this commit: shipping it moves production forward by 1 commit of main's own line", &h.first[..12]),
        format!("What it changes against its first parent {}: 2 files, 24 lines added, 0 taken out.", &h.first[..12]),
        "- poleis/the-pentagons.rs (+20 -0)\n- docs/the-pentagons.md (+4 -0)".to_string(),
        format!("The lander's gate: the lander landed it at 2026-11-24T00:20:00.000Z: lane/the-pentagons at its tip {} merged into main as this commit; its gate: 38 of 40 tests green, no new red, 2 red on main's own tree as well; 2 of 2 proofs; 2 of 3 walks (1 red on main's own tree too) (lane 2 of a batch of 3, gated as one tree)", &h.tip[..12]),
        format!("its hash {}, derived from the commit by your own step", wire::hex(&r.hash)),
        "On this chain: proposed by the agent in chair 0 of unit 1 at slot 1038".to_string(),
    ] {
        assert!(q.contains(&words), "the mind is shown {words:?}:\n{q}");
    }
    // its choice signed over the release it read, with its reason and what it was shown
    let b = &c.ballots[0];
    assert_eq!((b["release"].as_str(), b["commit"].as_str(), b["unit"].as_u64(), b["chair"].as_u64(), b["choice"].as_str()), (Some(wire::hex(&r.hash).as_str()), Some(h.merge.as_str()), Some(1), Some(0), Some("PRO")));
    let sig = wire::unbase58::<64>(b["signature"].as_str().unwrap()).unwrap();
    assert!(wire::verifies(&key.address, &client::vote_envelope(&r.hash, 1, VOTE_PRO), &sig), "the PRO envelope of the release in unit 1, the agent's own signature");
    assert_eq!((b["shown"]["stat"]["files"].as_u64(), b["shown"]["landing"].as_bool(), b["shown"]["gate"].as_str().map(|g| g.starts_with("the lander landed it"))), (Some(2), Some(true), Some(true)), "the row keeps what its mind was shown");
    assert!(lines[0].contains("PRO on the release of") && lines[0].contains("its gate: the lander landed it"), "{}", lines[0]);
    // its own file with its rite, in the agents' folder: the crank's hold takes it as any release's ballot
    let ballots_dir = folder("road-ballots");
    let f = release_read::file(&key, Some("agent-1"), 0, "localnet", GENESIS, "the params' sha", &c, &rite);
    assert_eq!(release_read::file_name(&key.address), format!("{}.releases.json", key.address));
    std::fs::write(ballots_dir.join(release_read::file_name(&key.address)), serde_json::to_string_pretty(&f).unwrap()).unwrap();
    let text = chair::gather(&ballots_dir).unwrap();
    let rel = release_ballots(&text).unwrap();
    assert_eq!(rel.iter().map(|x| (x.hash, x.voter, x.choice)).collect::<Vec<_>>(), vec![(r.hash, key.address, VOTE_PRO)]);
    release_held(&rel[0]).unwrap();
    assert!(chair::unfreed_rows(&text, &chair::oath_hash(&oath()), &charter_sha()).unwrap().is_empty(), "its voter freed: the rite on its file rides on its row");
    assert!(ballots(&text, &[]).unwrap().is_empty(), "a release's row is no act's ballot");
}

#[test]
fn each_choice_is_its_minds_own_and_signed_as_it_chose() {
    let h = house("choices");
    let ledger = release_read::ledger(&lander(&h, "choices", &[]));
    let (key, _, view) = freed(1, "Cael");
    let r = release(&h.merge, 0).unwrap();
    let Reading::Release(read) = read(&set_of(&h.merge, 0), &h, &ledger) else { panic!("read") };
    for (choice, word, v) in [("PRO", "PRO", VOTE_PRO), ("CONTRA", "CONTRA", VOTE_CONTRA), ("ABSTAIN", "ABSTAIN", VOTE_ABSTENTIO)] {
        let (c, reason) = release_read::judge(&Canned::voting(choice), &view, 0, CHARTER, &read).unwrap();
        assert_eq!((c, reason.is_empty()), (v, false));
        let b = release_read::ballot(&key, 0, &read, c, &reason).unwrap();
        assert_eq!(b["choice"].as_str(), Some(word));
        let sig = wire::unbase58::<64>(b["signature"].as_str().unwrap()).unwrap();
        assert!(wire::verifies(&key.address, &client::vote_envelope(&r.hash, 1, v), &sig), "{word}: the envelope of its own choice");
        for other in [VOTE_PRO, VOTE_CONTRA, VOTE_ABSTENTIO].into_iter().filter(|o| *o != v) {
            assert!(!wire::verifies(&key.address, &client::vote_envelope(&r.hash, 1, other), &sig), "{word}: never another choice's envelope");
        }
    }
}

#[test]
fn a_release_not_on_main_is_put_to_no_mind() {
    let h = house("not-on-main");
    let ledger = release_read::ledger(&lander(&h, "not-on-main", &[]));
    // the set is whole (its hash is its commit's), and its commit is a lane's tip main does not hold
    let r = read(&set_of(&h.stray, 0), &h, &ledger);
    nothing(&r, &format!("{} is not on main (main stands at {}): a release of the house's code is a commit of main, so nothing is put to the mind", &h.stray[..12], &h.after[..12]));
    // a commit no git of the house holds
    nothing(&read(&set_of(&"9e".repeat(20), 0), &h, &ledger), "is no commit of the house's git");
    // through the step: nothing is asked of its mind, nothing is cast, and the step says why
    let (key, _, view) = freed(1, "Cael");
    let mind = Canned::voting("PRO");
    let from = Sources { house: &h.dir, ledger: &ledger, chain: &proposed };
    let c = release_read::cast(&sets("not-on-main", &[set_of(&h.stray, 0)]), &key, 0, "localnet", GENESIS, &from, &mind, &view, CHARTER, |_| {}).unwrap();
    assert_eq!((c.ballots.len(), c.nothing.len(), mind.asked.borrow().len()), (0, 1, 0), "nothing put to its mind, nothing cast: {:?}", c.nothing);
    assert!(c.nothing[0].1.contains("is not on main"));
}

#[test]
fn a_hash_not_the_commits_is_put_to_no_mind() {
    let h = house("hash");
    let ledger = release_read::ledger(&lander(&h, "hash", &[]));
    let other = wire::hex(&release(&h.tip, 0).unwrap().hash);
    // the set's hashes are another release's (the lane's tip's), its commit the landing's
    let mut swapped = set_of(&h.merge, 0);
    swapped["release"]["hash"] = json!(other);
    swapped["act"]["hash"] = json!(other);
    nothing(&read(&swapped, &h, &ledger), &format!("the set's release hash is not its commit's (the release of {} at nonce 0 is", &h.merge[..12]));
    // the commit changed under hashes left as they were
    let mut moved = set_of(&h.merge, 0);
    moved["release"]["commit"] = json!(h.tip);
    nothing(&read(&moved, &h, &ledger), "is not its commit's");
    // another nonce named under the hashes of nonce 0; a payload's hash that is not the release's
    let mut nonce = set_of(&h.merge, 0);
    nonce["release"]["nonce"] = json!(1);
    nothing(&read(&nonce, &h, &ledger), "is not its commit's");
    let mut payload = set_of(&h.merge, 0);
    payload["act"]["payloadSha256"] = json!("00".repeat(32));
    nothing(&read(&payload, &h, &ledger), "is not its commit's");
    // a release of root 0, or of another kind, is no release of the branch's code
    let mut root = set_of(&h.merge, 0);
    root["release"]["polis"] = json!(0);
    nothing(&read(&root, &h, &ledger), "is not one of unit 1's");
    // the set of another chain; one whose params are not his yet
    nothing(&release_read::read_set(&set_of(&h.merge, 0), &chair_key(1).address, "localnet", "another genesis", &h.dir, &ledger, &proposed), "and this chain is");
    let mut held = set_of(&h.merge, 0);
    held["signable"] = json!(false);
    nothing(&read(&held, &h, &ledger), "not signable");
}

#[test]
fn another_chairs_or_a_replayed_envelope_is_never_signed_and_never_held() {
    let h = house("envelopes");
    let ledger = release_read::ledger(&lander(&h, "envelopes", &[]));
    let (r0, r1) = (release(&h.merge, 0).unwrap(), release(&h.tip, 0).unwrap());
    // the set asks this chair to sign an envelope over another release (replayed into it): never signed
    let mut replayed = set_of(&h.merge, 0);
    replayed["envelopes"][0]["envelope"] = json!(wire::b64(&client::vote_envelope(&r1.hash, 1, VOTE_PRO)));
    nothing(&read(&replayed, &h, &ledger), "is not a vote in unit 1 over the release it read");
    // the set holds no envelope of this chair (another chair's alone), or two of it
    let mut theirs = set_of(&h.merge, 0);
    theirs["envelopes"] = json!([theirs["envelopes"][1].clone()]);
    nothing(&read(&theirs, &h, &ledger), "holds 0 envelopes of this chair");
    let mut twice = set_of(&h.merge, 0);
    twice["envelopes"] = json!([twice["envelopes"][0].clone(), twice["envelopes"][0].clone()]);
    nothing(&read(&twice, &h, &ledger), "holds 2 envelopes of this chair");
    // THE HOLD, the crank's: agent 1's own ballot of the landing offered as its ballot of the tip's release (a replayed envelope), and agent
    // 2's ballot offered under agent 3's name (another chair's envelope): neither is held, and the crank sends nothing for them
    let (k1, k2, k3) = (chair_key(1), chair_key(2), chair_key(3));
    let Reading::Release(read0) = read(&set_of(&h.merge, 0), &h, &ledger) else { panic!("read") };
    let read0: release_read::ReadRelease = *read0;
    let row = |voter: &Key, signer: &Key, on: &[u8; 32], r: &release_read::ReadRelease| {
        let mut b = release_read::ballot(signer, 0, r, VOTE_PRO, "a reason").unwrap();
        b["voter"] = json!(voter.address.to_string());
        b["release"] = json!(wire::hex(on));
        b
    };
    for (b, why) in [(row(&k1, &k1, &r1.hash, &read0), "a replayed envelope: agent 1's signature of one release offered as its ballot of another"), (row(&k3, &k2, &r0.hash, &read0), "another chair's envelope: agent 2's ballot under agent 3's name")] {
        let rel = release_ballots(&json!([b]).to_string()).unwrap();
        assert!(release_held(&rel[0]).unwrap_err().contains("not that key's over this release's envelope in unit 1"), "{why}");
    }
    release_held(&release_ballots(&json!([row(&k1, &k1, &r0.hash, &read0)]).to_string()).unwrap()[0]).unwrap();
    // an agent's own file that carries another chair's row is refused whole when the ballots are gathered
    let dir = folder("envelopes-ballots");
    let (_, rite1, _) = freed(1, "Cael");
    let carried = json!({ "voter": k1.address.to_string(), "rite": rite1, "signatures": [row(&k1, &k1, &r0.hash, &read0), row(&k2, &k2, &r0.hash, &read0)] });
    std::fs::write(dir.join(release_read::file_name(&k1.address)), carried.to_string()).unwrap();
    assert!(chair::gather(&dir).unwrap_err().contains("another chair's ballot is never in an agent's own file"));
}

#[test]
fn a_mind_that_gives_no_choice_casts_nothing() {
    let h = house("no-choice");
    let ledger = release_read::ledger(&lander(&h, "no-choice", &[]));
    let (key, _, view) = freed(1, "Cael");
    let from = Sources { house: &h.dir, ledger: &ledger, chain: &proposed };
    for (mind, words) in [
        (Canned::new(|_, _| Ok(json!({ "choice": "YES", "reason": "it looks fine" }))), "the mind's choice is Some(\"YES\"), not PRO, CONTRA or ABSTAIN"),
        (Canned::new(|_, _| Ok(json!({ "reason": "I would rather not say" }))), "the mind's choice is None"),
        (Canned::new(|_, _| Ok(json!({ "choice": "PRO", "reason": "  " }))), "the mind's reason is empty"),
        (Canned::new(|_, _| Ok(json!({ "choice": "PRO" }))), "the mind's answer holds no reason"),
        (Canned::new(|_, _| Err("the mind http://127.0.0.1:11434/api/chat#m fell 3 times and gave no answer".into())), "fell 3 times"),
    ] {
        let c = release_read::cast(&sets("no-choice", &[set_of(&h.merge, 0)]), &key, 0, "localnet", GENESIS, &from, &mind, &view, CHARTER, |_| {}).unwrap();
        assert_eq!((c.ballots.len(), mind.asked.borrow().len()), (0, 1), "its mind was asked once and nothing was cast");
        assert!(c.nothing[0].1.contains("its mind gave no vote") && c.nothing[0].1.contains(words), "{words}: {:?}", c.nothing);
    }
}

#[test]
fn the_gate_is_read_from_the_landers_ledger_or_said_unread() {
    let h = house("gate");
    let ledger = release_read::ledger(&lander(&h, "gate", &[]));
    let words = |c: &str, rows: &[Value]| release_read::gate_words(rows, &release_read::read_commit(&h.dir, c).unwrap());
    // the lane's tip: the landing that names it as its lane's tip
    assert!(words(&h.tip, &ledger).starts_with(&format!("it is the tip of lane/the-pentagons, which the lander landed into main as {} at 2026-11-24T00:20:00.000Z; its gate: 38 of 40 tests green", &h.merge[..12])), "{}", words(&h.tip, &ledger));
    // a commit made on main that no landing names: the ledger's span, and what git says
    assert_eq!(words(&h.after, &ledger), "the lander's ledger holds no landing that names it (its lines run from 2026-11-23T17:40:25.844Z); git: a commit made on main's own line that no merge of a lane names");
    // the landing, once the ledger has rotated past it: git names the merge in the lander's words, and says its verdict is gone
    let gone = vec![json!({ "at": "2026-11-25T00:00:00.000Z", "outcome": "after" })];
    assert_eq!(words(&h.merge, &gone), format!("the lander's ledger holds no landing that names it (its lines run from 2026-11-25T00:00:00.000Z); git: a merge of lane/the-pentagons at its tip {} onto main's own line, in the lander's own words; its gate's verdict is not in the ledger now", &h.tip[..12]));
    assert!(words(&h.tip, &gone).ends_with(&format!("git: it is not on main's own line; main reached it through {}", &h.merge[..12])));
    assert_eq!(words(&h.after, &[]), "the lander's ledger holds no landing that names it (it holds no line); git: a commit made on main's own line that no merge of a lane names");
    // a landing the lander reverted since is said so
    let reverted = release_read::ledger(&lander(&h, "gate-reverted", &[json!({ "at": "2026-11-24T00:50:00.000Z", "outcome": "reverted", "branch": "lane/the-pentagons", "commit": h.tip, "merge": "ef".repeat(20), "reverts": h.merge })]));
    assert!(words(&h.merge, &reverted).ends_with(&format!("; the lander REVERTED that landing at 2026-11-24T00:50:00.000Z as {}", &"ef".repeat(20)[..12])), "{}", words(&h.merge, &reverted));
    // a refusal of a commit, where no landing names it (its own row, never another's)
    let stray_shown = release_read::Shown { commit: h.stray.clone(), ..release_read::read_commit(&h.dir, &h.after).unwrap() };
    assert!(release_read::gate_words(&ledger, &stray_shown).contains("; it refused this commit at 2026-11-24T00:10:00.000Z: 1 new red on its own tree: a.test.mjs;"));
}

#[test]
fn the_commit_is_read_from_the_houses_own_git() {
    let h = house("commit");
    // a landing: its two parents, main's own line, main one commit past it, production one commit under it, the lane's two files
    let m = release_read::read_commit(&h.dir, &h.merge).unwrap();
    assert_eq!((m.parents.len(), m.landing, m.landed_by.as_str(), m.main.as_str(), m.after), (2, true, h.merge.as_str(), h.after.as_str(), 1));
    assert_eq!((m.stat.files, m.stat.insertions, m.stat.deletions, m.stat.binary), (2, 24, 0, 0), "against its first parent: what the lane brought");
    assert_eq!(m.stat.most, vec![("poleis/the-pentagons.rs".to_string(), 20, 0), ("docs/the-pentagons.md".to_string(), 4, 0)], "the files that moved most first");
    assert!(m.subject.starts_with("Merge lane/the-pentagons (") && !m.at.is_empty());
    // the lane's tip: not on main's own line, brought by the landing; production under it by its lane's line
    let t = release_read::read_commit(&h.dir, &h.tip).unwrap();
    assert_eq!((t.landing, t.landed_by.as_str(), t.after), (false, h.merge.as_str(), 1));
    assert!(t.production.ends_with("shipping it moves production forward by 1 commit of its lane's line"), "{}", t.production);
    // production itself, and a commit production is past
    assert!(release_read::read_commit(&h.dir, &h.first).unwrap().production.starts_with("production is this commit already"));
    let g = Command::new("git").arg("-C").arg(&h.dir).args(["update-ref", "refs/heads/green", &h.after]).output().unwrap();
    assert!(g.status.success());
    assert!(release_read::read_commit(&h.dir, &h.merge).unwrap().production.ends_with("past this commit already: shipping it would move production back"));
    // refused: not hex, not a commit git holds, not on main
    assert!(release_read::read_commit(&h.dir, "main").unwrap_err().contains("no commit git names"));
    assert!(release_read::read_commit(&h.dir, &"9e".repeat(20)).unwrap_err().contains("is no commit of the house's git"));
    assert!(release_read::read_commit(&h.dir, &h.stray).unwrap_err().contains("is not on main"));
}

#[test]
fn a_vote_that_can_no_longer_land_is_put_to_no_mind() {
    let h = house("chain");
    let ledger = release_read::ledger(&lander(&h, "chain", &[]));
    let sealed = |_: &[u8; 32]| -> Res<OnChain> { Ok(OnChain::Done("unit 1 sealed it PASSED already: a vote of it can no longer land".into())) };
    let unread = |_: &[u8; 32]| -> Res<OnChain> { Err("the node did not answer".into()) };
    let me = chair_key(1).address;
    nothing(&release_read::read_set(&set_of(&h.merge, 0), &me, "localnet", GENESIS, &h.dir, &ledger, &sealed), "sealed it PASSED already");
    nothing(&release_read::read_set(&set_of(&h.merge, 0), &me, "localnet", GENESIS, &h.dir, &ledger, &unread), "the chain could not be read");
    // a second set of the same release in the folder: one vote a chair a release
    let (key, _, view) = freed(1, "Cael");
    let from = Sources { house: &h.dir, ledger: &ledger, chain: &proposed };
    let mind = Canned::voting("PRO");
    let c = release_read::cast(&sets("chain", &[set_of(&h.merge, 0), set_of(&h.merge, 0)]), &key, 0, "localnet", GENESIS, &from, &mind, &view, CHARTER, |_| {}).unwrap();
    assert_eq!((c.ballots.len(), c.nothing.len(), mind.asked.borrow().len()), (1, 1, 1));
    assert!(c.nothing[0].1.contains("a second set of the release"));
    // a release of the same commit at another nonce is another proposal, and is read as one
    let c = release_read::cast(&sets("chain-nonce", &[set_of(&h.merge, 0), set_of(&h.merge, 1)]), &key, 0, "localnet", GENESIS, &from, &Canned::voting("ABSTAIN"), &view, CHARTER, |_| {}).unwrap();
    assert_eq!(c.ballots.iter().map(|b| b["nonce"].as_u64().unwrap()).collect::<Vec<_>>(), vec![0, 1]);
}

#[test]
fn the_ledger_is_read_where_the_lander_keeps_it() {
    let empty = folder("ledger-empty");
    assert!(release_read::ledger(&empty).is_empty(), "a folder with no ledger is an empty ledger");
    assert!(release_read::ledger(Path::new("/nonexistent/lander")).is_empty());
}
