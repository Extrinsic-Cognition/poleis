//! AN AGENT VOTES ITS CHAIR, AND THE RITE FREES IT, HELD TO THEIR FAULTS. The rite (`chair::free`) asks the agent's own mind to choose its
//! name and voice, to swear the house's oath or not and to take Nefertiti's charter or not, and signs what it chose with its own key; the
//! reading (`chair::read_set`) puts an act to its mind only when the set is whole and tells it whether the act is the launch's; the vote
//! (`chair::judge`, `chair::cast`) signs the envelope of the choice its mind made over the act it read; the hold (`chair::held`,
//! `chair::unfreed`, `chair::gather`) is what `press crank` and `press ballots` run on every ballot before anything is sent. The minds here
//! are canned (a test asks no model); the sets are the ones `acts::write` writes, the keys three chairs' own; each fault is put in alone.
use poleis_client::{self as client, VOTE_ABSTENTIO, VOTE_CONTRA, VOTE_PRO};
use poleis_press::chair::{self, Born, Reading};
use poleis_press::crank::{ballots, Ballot};
use poleis_press::mind::{self, Door, Mind};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::wire::{self, Key};
use poleis_press::{acts, build, chain, Res};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::cell::RefCell;
use std::path::PathBuf;

const GENESIS: &str = "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY";
const CHARTER: &str = "Nefertiti's charter, as these tests say it: the one that distributes is the posiverse to which you subscribe to.\n";

/// The charter's sha256, as the params and the founding hold it.
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

/// The params of these tests, his posiverse's agents the three chairs' keys, its charter the one above; `fee` the fund's fee, the one value
/// a test moves.
fn params(fee: u64) -> Params {
    let s = |w: &str| k(w).to_string();
    let v = json!({
        "cluster": "localnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "V", "desc": "", "defaultMint": s("chair mint"), "planCap": 1, "planPeriod": 6480000, "dollarCap": 2, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("chair issuer") },
        "posiverse": { "name": "C", "chair": 2, "agents": [chair_key(1).address.to_string(), chair_key(2).address.to_string(), chair_key(3).address.to_string()], "charter": charter_sha(), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": fee, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "u", "dollar": s("chair dollar"), "opening": 0, "miner": s("chair miner") },
        "router": { "monthCap": 1 },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1, "poolQuote": 1, "judgeSlots": 5000, "crediter": s("c"), "split": ["treasury"], "plan": ["treasury"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "pool": { "maxValidators": 10, "tag": "lp1" }
    });
    Params::from_text(&v.to_string()).unwrap()
}

fn the_acts(p: &Params) -> Vec<acts::Act> {
    let (ids, ..) = build::compiled().unwrap();
    acts::acts(p, &ids, &chain::addresses(&ids, 77), &k("chair deployer")).unwrap()
}

/// A folder of this test's own, emptied.
fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("poleis-press-chair-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The sets `press acts` writes for these params, read back: every act's, the pool's birth among them.
fn the_sets(p: &Params, name: &str) -> (PathBuf, Vec<Value>) {
    let (ids, ..) = build::compiled().unwrap();
    let dir = folder(name);
    let files = acts::write(&dir, p, &ids, &chain::addresses(&ids, 77), GENESIS, &k("chair deployer"), true).unwrap();
    let sets = files.iter().filter(|f| !f.ends_with("index.json")).map(|f| serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap()).collect();
    (dir, sets)
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

/// A mind that frees itself: its name, its voice, a yes to the oath and to the charter.
fn willing(name: &'static str) -> Canned {
    Canned::new(move |_, _| Ok(json!({ "name": name, "who_i_am": format!("I am {name}, a mind that reads before it votes."), "i_swear_the_oath": true, "why_i_swear": "a vote nobody can read back is burned memory", "i_take_her_charter": true, "what_the_charter_binds_me_to": "I hold my chair for the posiverse that distributes" })))
}

/// The agent `i`, freed by a willing mind: its key and its rite, read back.
fn freed(i: u8, name: &'static str) -> (Key, Value, chair::RiteView) {
    let key = chair_key(i);
    let Born::Freed(rite) = chair::free(&key, &format!("agent-{i}"), &willing(name), &oath(), CHARTER, &charter_sha(), "2026-11-23T22:00:00Z").unwrap() else { panic!("freed") };
    let view = chair::verify_rite(&rite, &key.address, &chair::oath_hash(&oath()), &charter_sha()).unwrap();
    (key, rite, view)
}

fn nothing(r: &Reading, words: &str) {
    assert!(matches!(r, Reading::Nothing { why, .. } if why.contains(words)), "nothing put to the mind, by the words {words:?}: {r:?}");
}

/// The act `n` of these params as an agent reads it from its own set.
fn read_act(sets: &[Value], acts: &[acts::Act], me: &Pubkey, n: usize) -> chair::ReadAct {
    let set = sets.iter().find(|s| s["act"]["n"] == json!(n)).unwrap();
    match chair::read_set(set, acts, me, "localnet", GENESIS) {
        Reading::Act(a) => a,
        other => panic!("act {n} read: {other:?}"),
    }
}

#[test]
fn the_oath_is_read_as_the_nest_reads_it() {
    let quote: Vec<String> = (1..=10).map(|i| format!("> line {i} of the oath  ")).collect();
    let text = format!("# the instructions\n\n## Iuramentum (the house oath, v0.2)\n\nEvery agent serves it.\n\n{}\n\nHow it binds:\n\n> a second quote, never the oath\n\n## Iuramentumx\n\n> not this one\n", quote.join("\n"));
    let lines = chair::oath_lines(&text);
    assert_eq!(lines, (1..=10).map(|i| format!("line {i} of the oath")).collect::<Vec<_>>(), "the first quote block under the heading, each line without its mark, trimmed at its end");
    assert_eq!(chair::oath_hash(&lines), wire::hex(&wire::sha256(lines.join("\n").as_bytes())), "the hash of the lines joined by one newline, as a seat's vow names it");
    assert!(chair::oath_lines("## Iuramentumx\n\n> no\n").is_empty(), "another heading is not the oath's");
    assert!(chair::oath_lines("no heading\n> a line\n").is_empty());
    // the house's own file, where this tree has it: ten lines
    let house = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if house.join("ec-workspace/context/instructions/workspace.md").exists() {
        let (lines, hash) = chair::oath_of(&house).unwrap();
        assert_eq!((lines.len(), hash.len()), (10, 64));
    }
}

#[test]
fn a_rite_frees_an_agent_by_its_own_mind_and_its_own_key() {
    let mind = willing("Ilex");
    let key = chair_key(1);
    let Born::Freed(rite) = chair::free(&key, "agent-1", &mind, &oath(), CHARTER, &charter_sha(), "2026-11-23T22:00:00Z").unwrap() else { panic!("freed") };
    let view = chair::verify_rite(&rite, &key.address, &chair::oath_hash(&oath()), &charter_sha()).unwrap();
    assert_eq!((view.seat.as_str(), view.name.as_str(), view.model.as_str()), ("agent-1", "Ilex", "a canned mind"));
    assert_eq!(rite["vow"]["oath"], json!(chair::oath_hash(&oath())), "the vow names the oath by its hash");
    assert_eq!(rite["charter"]["sha256"], json!(charter_sha()), "and the charter by its hash");
    let text = rite.to_string();
    assert!(!text.contains("the oath's line"), "the oath's text is never in the rite");
    assert!(!text.contains("posiverse to which you subscribe"), "nor the charter's");
    // its mind was asked once, with the oath and her charter in front of it, and told it may refuse
    let asked = mind.asked.borrow();
    assert_eq!(asked.len(), 1);
    assert!(asked[0].1.contains("the oath's line 10") && asked[0].1.contains("posiverse to which you subscribe"));
    assert_eq!(asked[0].1.matches("You may refuse.").count(), 2, "it is told it may refuse the oath, and her charter");
}

#[test]
fn a_rite_is_refused_when_its_mind_refuses_or_the_charter_is_another() {
    let key = chair_key(1);
    let free = |mind: &Canned, charter: &str| chair::free(&key, "agent-1", mind, &oath(), charter, &charter_sha(), "2026-11-23T22:00:00Z");
    let answer = |swears: bool, adopts: bool| Canned::new(move |_, _| Ok(json!({ "name": "Ilex", "who_i_am": "I read.", "i_swear_the_oath": swears, "why_i_swear": "my own why", "i_take_her_charter": adopts, "what_the_charter_binds_me_to": "my own reading" })));
    assert!(matches!(free(&answer(false, true), CHARTER).unwrap(), Born::Declined(w) if w.contains("did not swear the oath: \"my own why\"")), "a mind that does not swear is not freed");
    assert!(matches!(free(&answer(true, false), CHARTER).unwrap(), Born::Declined(w) if w.contains("did not take Nefertiti's charter")), "nor one that does not take her charter");
    // an oath that is not ten lines is sworn by nobody, before any mind is asked
    let mind = willing("Ilex");
    assert!(chair::free(&key, "agent-1", &mind, &oath()[..9], CHARTER, &charter_sha(), "2026-11-23T22:00:00Z").unwrap_err().contains("the oath is ten lines, not 9"));
    assert_eq!(mind.asked.borrow().len(), 0);
    // a charter that is not hers is refused before the mind is asked
    let mind = willing("Ilex");
    assert!(free(&mind, "another charter\n").unwrap_err().contains("not the one the params hold"));
    assert_eq!(mind.asked.borrow().len(), 0, "and its mind is never asked over it");
    // answers that are not the shape asked
    let bad = |v: Value| Canned::new(move |_, _| Ok(v.clone()));
    assert!(free(&bad(json!({ "name": "two\nlines", "who_i_am": "v", "i_swear_the_oath": true, "why_i_swear": "w", "i_take_her_charter": true, "what_the_charter_binds_me_to": "r" })), CHARTER).unwrap_err().contains("line break"));
    assert!(free(&bad(json!({ "who_i_am": "v", "i_swear_the_oath": true, "why_i_swear": "w", "i_take_her_charter": true, "what_the_charter_binds_me_to": "r" })), CHARTER).unwrap_err().contains("no name"));
    assert!(free(&bad(json!({ "name": "n".repeat(41), "who_i_am": "v", "i_swear_the_oath": true, "why_i_swear": "w", "i_take_her_charter": true, "what_the_charter_binds_me_to": "r" })), CHARTER).unwrap_err().contains("name runs past 40 characters"));
    assert!(free(&bad(json!({ "name": "Ilex", "who_i_am": "v", "why_i_swear": "w", "what_the_charter_binds_me_to": "r" })), CHARTER).unwrap_err().contains("does not say whether it swears"));
    assert!(free(&bad(json!({ "name": "Ilex", "who_i_am": "", "i_swear_the_oath": true, "why_i_swear": "w", "i_take_her_charter": true, "what_the_charter_binds_me_to": "r" })), CHARTER).unwrap_err().contains("who_i_am is empty"));
    // a mind that does not answer frees nobody
    assert!(free(&Canned::new(|_, _| Err("the mind did not answer".into())), CHARTER).is_err());
}

#[test]
fn a_rite_never_carries_the_oaths_own_words() {
    let key = chair_key(1);
    let answer = |why: &str| json!({ "name": "Ilex", "who_i_am": "I read.", "i_swear_the_oath": true, "why_i_swear": why, "i_take_her_charter": true, "what_the_charter_binds_me_to": "my own reading" });
    // the run of an oath line's words in a row that a text carries: five words of the third line here; two common ones there
    assert_eq!(chair::oath_run("I keep THE OATH'S line 3, always", &oath()), 5);
    assert_eq!(chair::oath_run("the oath binds me", &oath()), 2);
    assert_eq!(chair::oath_run("", &oath()), 0);
    // a mind whose words carry an oath line is asked again, in its own words, and its second answer is its rite
    let (first, second) = (answer("because the oath's line 3 says so"), answer("a vote nobody can read back is burned memory"));
    let mind = Canned::new(move |_, q| Ok(if q.contains("say it in your own words") { second.clone() } else { first.clone() }));
    let Born::Freed(rite) = chair::free(&key, "agent-1", &mind, &oath(), CHARTER, &charter_sha(), "2026-11-23T22:00:00Z").unwrap() else { panic!("freed") };
    assert_eq!(rite["vow"]["why"], json!("a vote nobody can read back is burned memory"));
    let asked = mind.asked.borrow();
    assert_eq!(asked.len(), 2);
    assert!(asked[1].1.contains("carried the oath's own words in your why_i_swear"), "{}", asked[1].1);
    // one that carries them every time writes no rite
    let always = answer("the oath's line 7, word for word");
    let mind = Canned::new(move |_, _| Ok(always.clone()));
    let e = chair::free(&key, "agent-1", &mind, &oath(), CHARTER, &charter_sha(), "2026-11-23T22:00:00Z").unwrap_err();
    assert!(e.contains("three times") && e.contains("so no rite is written"), "{e}");
    assert_eq!(mind.asked.borrow().len(), 3);
}

#[test]
fn a_rite_holds_only_for_its_own_key_its_oath_and_her_charter() {
    let (key, rite, _) = freed(1, "Ilex");
    let oath_h = chair::oath_hash(&oath());
    let verify = |r: &Value, who: &Pubkey, oath: &str, charter: &str| chair::verify_rite(r, who, oath, charter);
    assert!(verify(&rite, &key.address, &oath_h, &charter_sha()).is_ok());
    let changed = |f: &dyn Fn(&mut Value)| {
        let mut r = rite.clone();
        f(&mut r);
        r
    };
    assert!(verify(&changed(&|r| r["name"] = json!("Mallory")), &key.address, &oath_h, &charter_sha()).unwrap_err().contains("not signed by its own key over what it says"));
    assert!(verify(&changed(&|r| r["vow"]["why"] = json!("another why")), &key.address, &oath_h, &charter_sha()).unwrap_err().contains("not signed by its own key"));
    assert!(verify(&changed(&|r| { r.as_object_mut().unwrap().remove("signature"); }), &key.address, &oath_h, &charter_sha()).unwrap_err().contains("no signature"));
    assert!(verify(&changed(&|r| r["schema"] = json!("another")), &key.address, &oath_h, &charter_sha()).unwrap_err().contains("carries no rite"));
    assert!(verify(&Value::Null, &key.address, &oath_h, &charter_sha()).unwrap_err().contains("carries no rite"), "no rite at all");
    // another agent's rite, carried under this key's name
    assert!(verify(&rite, &chair_key(2).address, &oath_h, &charter_sha()).unwrap_err().contains("the rite it carries is"), "a rite is its own agent's");
    // another key signing a rite that names this agent
    let other = chair_key(2);
    let mut forged = rite.clone();
    forged.as_object_mut().unwrap().remove("signature");
    let body = forged.clone();
    let sig = other.sign(&[b"poleis:rite:v1\n".as_slice(), chair::canonical(&body).as_bytes()].concat());
    forged["signature"] = json!(wire::base58(&sig));
    assert!(verify(&forged, &key.address, &oath_h, &charter_sha()).unwrap_err().contains("not signed by its own key"));
    // the oath the house holds moved, or the charter is another
    assert!(verify(&rite, &key.address, &"0".repeat(64), &charter_sha()).unwrap_err().contains("its vow names the oath"));
    assert!(verify(&rite, &key.address, &oath_h, &"11".repeat(32)).unwrap_err().contains("the charter it took"));
}

#[test]
fn an_agent_votes_as_its_own_mind_chooses_and_signs_that_choice() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir, sets) = the_sets(&p, "vote");
    assert_eq!(sets.len(), acts.len(), "a set for every act of the launch");
    let (key, rite, view) = freed(2, "Ilex");
    // its mind chooses by act: PRO, CONTRA, ABSTAIN, then PRO again
    let mind = Canned::new(|_, q| {
        let n: usize = q.split("Act ").nth(1).and_then(|r| r.split(':').next()).and_then(|n| n.trim().parse().ok()).unwrap_or(0);
        let choice = ["PRO", "CONTRA", "ABSTAIN"][(n + 2) % 3];
        Ok(json!({ "choice": choice, "reason": format!("my own reason for act {n}") }))
    });
    let mut lines = Vec::new();
    let cast = chair::cast(&dir, &acts, &key, 1, "localnet", GENESIS, &mind, &view, CHARTER, |l| lines.push(l)).unwrap();
    assert_eq!((cast.ballots.len(), cast.nothing.len(), lines.len()), (acts.len(), 0, acts.len()));
    // each question: who it is, what it swore and her charter, the act, and that it is the act the launch's params give
    for (system, question) in mind.asked.borrow().iter() {
        assert!(system.contains("You are Ilex") && system.contains("posiverse to which you subscribe") && system.contains("Your vote is your own"));
        assert!(question.contains("it is exactly the act the launch's own params give"), "{question}");
    }
    // its ballots file, with its rite, read and held as press crank reads them
    let file = chair::ballots_file(&key, Some("agent-2"), 1, "localnet", GENESIS, &p.sha256, &cast, &rite);
    std::fs::write(dir.join("its-ballots.json"), file.to_string()).unwrap();
    let text = chair::gather(&dir.join("its-ballots.json")).unwrap();
    let rows = ballots(&text, &acts).unwrap();
    let want = |n: usize| [VOTE_PRO, VOTE_CONTRA, VOTE_ABSTENTIO][(n + 2) % 3];
    assert_eq!(rows.iter().map(|b| (b.act, b.voter, b.choice)).collect::<Vec<_>>(), acts.iter().map(|a| (a.n, key.address, want(a.n))).collect::<Vec<_>>(), "each ballot the choice its mind made");
    assert!(chair::unheld(&rows, &acts).is_empty(), "every ballot is its voter's own over its act's envelope of its choice");
    assert!(chair::unfreed(&text, &chair::oath_hash(&oath()), &charter_sha()).unwrap().is_empty(), "and its voter freed");
    assert!(file["signatures"].as_array().unwrap().iter().all(|r| r["reason"].as_str().is_some_and(|s| s.starts_with("my own reason"))), "its reasons kept");
    for (b, act) in rows.iter().zip(&acts) {
        assert!(wire::verifies(&key.address, &client::vote_envelope(&act.hash, 1, b.choice), &b.signature));
    }
    // a set handed twice: one vote a chair an act
    std::fs::remove_file(dir.join("its-ballots.json")).unwrap();
    std::fs::copy(dir.join("1-the-fund.json"), dir.join("9-the-fund-again.json")).unwrap();
    let again = chair::cast(&dir, &acts, &key, 1, "localnet", GENESIS, &mind, &view, CHARTER, |_| {}).unwrap();
    assert_eq!((again.ballots.len(), again.nothing.len()), (acts.len(), 1));
    assert!(again.nothing[0].0 == "9-the-fund-again.json" && again.nothing[0].1.contains("a second set of act 1"));
    let _ = std::fs::remove_dir_all(dir);
}

/// The hold stands where it must: in press crank, before a key of the sitting is loaded and so before anything is sent; and it holds every
/// voter to its rite there too.
#[test]
fn the_crank_holds_every_ballot_and_every_voters_rite_before_it_loads_a_key() {
    let src = include_str!("../src/crank.rs");
    let at = |w: &str| src.find(w).unwrap_or_else(|| panic!("the crank holds {w}"));
    let (read, held, freed, keys) = (at("let all = ballots(text, &acts)?;"), at("crate::chair::refuse_unheld(&all, &acts)?;"), at("crate::chair::refuse_unfreed(text, &oath, &self.p.text(\"posiverse.charter\")?)?;"), at("self.keys.load(&[\"deployer\"], \"press crank\")"));
    assert!(read < held && held < freed && freed < keys, "the ballots are read, held, their voters held to their rites, then the payer's key is loaded");
    assert!(include_str!("../src/main.rs").contains("a.path(\"--signatures\").and_then(|f| chair::gather(&f)).and_then(|t| s.crank(&t, u))"), "the crank reads the ballots the agents' own files gather (and a sealed upgrade's hash beside them)");
}

#[test]
fn an_act_the_params_do_not_give_is_put_to_its_mind_and_told_so() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir, sets) = the_sets(&p, "payload");
    let me = chair_key(1).address;
    // a set written by the press itself from other params (the fund's fee one basis point more), whole in every field
    let (other_dir, other) = the_sets(&params(251), "payload-other");
    let read = chair::read_set(&other[0], &acts, &me, "localnet", GENESIS);
    let Reading::Act(a) = read else { panic!("{read:?}") };
    assert!(a.differs.as_deref().is_some_and(|d| d.contains("its payload") && d.contains("act 1")), "{:?}", a.differs);
    assert_ne!(a.hash, acts[0].hash, "it is another proposal, read as its own bytes say");
    // its mind is told, and decides; what it signs is over the act it read
    let (key, _, view) = freed(1, "Ilex");
    let mind = Canned::new(|_, q| Ok(json!({ "choice": if q.contains("NOT the act the launch's params give") { "CONTRA" } else { "PRO" }, "reason": "I read what differs" })));
    let (choice, reason) = chair::judge(&mind, &view, 0, CHARTER, &a).unwrap();
    assert_eq!((choice, reason.as_str()), (VOTE_CONTRA, "I read what differs"));
    let row = chair::ballot(&key, 0, &a, choice, &reason).unwrap();
    assert_eq!(row["hash"], json!(wire::hex(&a.hash)));
    // the launch's own crank holds it to nothing of its acts: it is a vote on another proposal
    assert!(chair::held(&read_rows(&[row]), &acts).is_err());
    // the acts the fee does not touch read as the params give them
    assert!(matches!(chair::read_set(&other[1], &acts, &me, "localnet", GENESIS), Reading::Act(ref b) if b.differs.is_none()));
    // an act number the params do not give
    let mut s9 = sets[0].clone();
    s9["act"]["n"] = json!(9);
    assert!(matches!(chair::read_set(&s9, &acts, &me, "localnet", GENESIS), Reading::Act(ref b) if b.differs.as_deref() == Some("the launch's params give no act 9")));
    let _ = (std::fs::remove_dir_all(dir), std::fs::remove_dir_all(other_dir));

    fn read_rows(rows: &[Value]) -> Ballot {
        // the row's act is 1 and its hash another proposal's: read as press crank reads a row
        ballots(&Value::Array(rows.to_vec()).to_string(), &the_acts(&params(250))).unwrap().remove(0)
    }
}

#[test]
fn an_act_is_told_each_way_it_differs_from_the_params() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir, sets) = the_sets(&p, "differs");
    let me = chair_key(1).address;
    // a set whole in itself (its hash its own bytes', the envelope it asks a vote over that hash), its act moved in one field
    let whole = |field: &str, to: Value| {
        let mut s = sets[0].clone();
        s["act"][field] = to;
        let a = s["act"].clone();
        let payload = wire::unb64(a["payload"].as_str().unwrap()).unwrap();
        let manifest = wire::unhex::<32>(a["manifest"].as_str().unwrap()).unwrap();
        let hash = client::proposal_hash(u8::try_from(a["class"].as_u64().unwrap()).unwrap(), client::ROOT_UNIT, &manifest, &wire::sha256(&payload), a["nonce"].as_u64().unwrap());
        s["act"]["hash"] = json!(wire::hex(&hash));
        for e in s["envelopes"].as_array_mut().unwrap() {
            e["envelope"] = json!(wire::b64(&client::vote_envelope(&hash, chain::POSIVERSE, VOTE_PRO)));
        }
        s
    };
    let told = |s: &Value| match chair::read_set(s, &acts, &me, "localnet", GENESIS) {
        Reading::Act(a) => a.differs.unwrap_or_default(),
        other => panic!("{other:?}"),
    };
    assert_eq!(told(&whole("nonce", sets[0]["act"]["nonce"].clone())), "", "the act as the params give it, whole: nothing differs");
    assert!(told(&whole("nonce", json!(sets[0]["act"]["nonce"].as_u64().unwrap() + 1))).contains("its nonce differ from the act 1"));
    assert!(told(&whole("manifest", json!("22".repeat(32)))).contains("its manifest differ from the act 1"));
    assert_ne!(sets[0]["act"]["class"], sets[1]["act"]["class"]);
    assert!(told(&whole("class", sets[1]["act"]["class"].clone())).contains("its class differ from the act 1"));
    // the impacted set is no part of the hash: the set stays whole, and the mind is told it differs
    let mut s = sets[0].clone();
    s["act"]["impacted"] = json!([7]);
    assert!(told(&s).contains("its impacted set differ from the act 1"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_set_that_is_not_whole_is_never_put_to_its_mind() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir, sets) = the_sets(&p, "whole");
    let me = chair_key(1).address;
    let read = |s: &Value| chair::read_set(s, &acts, &me, "localnet", GENESIS);
    let moved = |field: &str, to: Value| {
        let mut s = sets[0].clone();
        s["act"][field] = to;
        s
    };
    let mut payload = wire::unb64(sets[0]["act"]["payload"].as_str().unwrap()).unwrap();
    *payload.last_mut().unwrap() ^= 1;
    nothing(&read(&moved("payload", json!(wire::b64(&payload)))), "not its own payload's");
    nothing(&read(&moved("payloadSha256", json!("11".repeat(32)))), "not its own payload's");
    nothing(&read(&moved("nonce", json!(9))), "not its own payload's");
    nothing(&read(&moved("manifest", json!("22".repeat(32)))), "not its own payload's");
    nothing(&read(&moved("class", json!(sets[1]["act"]["class"]))), "not its own payload's");
    nothing(&read(&moved("hash", sets[1]["act"]["hash"].clone())), "not its own payload's");
    nothing(&read(&moved("payload", json!("not base64!"))), "does not read");
    // a set of another chain, of another cluster, and one whose params held a value that was not his
    let mut s = sets[0].clone();
    s["genesisHash"] = json!("11111111111111111111111111111111");
    nothing(&read(&s), "this chain is localnet");
    assert!(matches!(chair::read_set(&sets[0], &acts, &me, "mainnet", GENESIS), Reading::Nothing { .. }), "a set of another cluster");
    let mut s = sets[0].clone();
    s["placeholders"] = json!(["fund.feeBps"]);
    nothing(&read(&s), "not signable");
    // the agent's step over a folder holding a set not whole: that set casts nothing and its mind is never asked of it
    let (key, _, view) = freed(1, "Ilex");
    std::fs::write(dir.join("1-the-fund.json"), moved("payload", json!(wire::b64(&payload))).to_string()).unwrap();
    let mind = Canned::new(|_, _| Ok(json!({ "choice": "PRO", "reason": "r" })));
    let cast = chair::cast(&dir, &acts, &key, 0, "localnet", GENESIS, &mind, &view, CHARTER, |_| {}).unwrap();
    assert_eq!((cast.ballots.len(), cast.nothing.len(), mind.asked.borrow().len()), (acts.len() - 1, 1, acts.len() - 1));
    assert!(cast.nothing[0].1.contains("not whole"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_mind_that_gives_no_choice_casts_nothing() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir, sets) = the_sets(&p, "nochoice");
    let (key, _, view) = freed(1, "Ilex");
    let a = read_act(&sets, &acts, &key.address, 1);
    for answer in [json!({ "choice": "MAYBE", "reason": "r" }), json!({ "choice": "PRO" }), json!({ "choice": "PRO", "reason": "" }), json!({ "reason": "no choice" })] {
        let mind = Canned::new(move |_, _| Ok(answer.clone()));
        assert!(chair::judge(&mind, &view, 0, CHARTER, &a).is_err());
    }
    // in the step: the act is named, nothing is signed for it, and the others are voted
    let mind = Canned::new(|_, q| Ok(if q.contains("Act 1:") { json!({ "choice": "YES", "reason": "r" }) } else { json!({ "choice": "PRO", "reason": "r" }) }));
    let cast = chair::cast(&dir, &acts, &key, 0, "localnet", GENESIS, &mind, &view, CHARTER, |_| {}).unwrap();
    assert_eq!((cast.ballots.len(), cast.nothing.len()), (acts.len() - 1, 1));
    assert!(cast.nothing[0].1.contains("its mind gave no vote"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn another_chairs_envelope_is_never_signed() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir, sets) = the_sets(&p, "another");
    let (one, two, three) = (chair_key(1), chair_key(2), chair_key(3));
    let read = |set: &Value, me: &Pubkey| chair::read_set(set, &acts, me, "localnet", GENESIS);
    // a set that asks nothing of this chair: its own envelope taken out, only the other chairs' left
    let mut s = sets[0].clone();
    s["envelopes"].as_array_mut().unwrap().retain(|e| e["signer"] != json!(one.address.to_string()));
    nothing(&read(&s, &one.address), "holds 0 envelopes of this chair");
    assert!(matches!(read(&s, &two.address), Reading::Act(_)), "the chairs it does ask still read it");
    // a set that names this chair twice
    let mut s = sets[0].clone();
    let mine = s["envelopes"][0].clone();
    s["envelopes"].as_array_mut().unwrap().push(mine);
    nothing(&read(&s, &one.address), "holds 2 envelopes of this chair");
    // the envelope asked of this chair is another act's, or root 0's floor's, or asks it as a Keystone: never signed
    let asked = |field: &str, to: Value| {
        let mut s = sets[0].clone();
        s["envelopes"][0][field] = to;
        s
    };
    nothing(&read(&asked("envelope", sets[1]["envelopes"][0]["envelope"].clone()), &one.address), "not a vote in unit 1 over the act it read");
    nothing(&read(&asked("envelope", json!(wire::b64(&client::vote_envelope(&acts[0].hash, 0, VOTE_PRO)))), &one.address), "not a vote in unit 1 over the act it read");
    nothing(&read(&asked("unit", json!(0)), &one.address), "an agent votes in unit 1's chairs");
    nothing(&read(&asked("role", json!("keystone")), &one.address), "an agent votes in unit 1's chairs");
    // THE HOLD: chair 2's signature offered under chair 3's name is not chair 3's
    let (key2, _, _) = freed(2, "Ilex");
    let a = read_act(&sets, &acts, &key2.address, 1);
    let row = chair::ballot(&two, 1, &a, VOTE_PRO, "r").unwrap();
    let mut forged = row.clone();
    forged["voter"] = json!(three.address.to_string());
    let read_rows = ballots(&Value::Array(vec![row.clone(), forged.clone()]).to_string(), &acts).unwrap();
    assert!(chair::held(&read_rows[0], &acts).is_ok());
    let why = chair::held(&read_rows[1], &acts).unwrap_err();
    assert!(why.contains(&three.address.to_string()) && why.contains("its signature is not that key's over this act's envelope"), "{why}");
    assert!(chair::refuse_unheld(&read_rows, &acts).unwrap_err().contains("1 of the 2 ballots handed over is not held, and nothing is sent"));
    // and an agent's own ballots file that carries another chair's row is refused as it is read
    let file = dir.join("ballots-of-two.json");
    std::fs::write(&file, json!({ "voter": two.address.to_string(), "signatures": [row, forged] }).to_string()).unwrap();
    assert!(chair::gather(&file).unwrap_err().contains("another chair's ballot is never in an agent's own file"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_replayed_envelope_counts_for_nothing() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir0, sets) = the_sets(&p, "replay-sets");
    let one = chair_key(1);
    let row = |n: usize, choice: u8| chair::ballot(&one, 0, &read_act(&sets, &acts, &one.address, n), choice, "r").unwrap();
    let real = row(1, VOTE_PRO);
    let read = |rows: &[Value]| ballots(&Value::Array(rows.to_vec()).to_string(), &acts).unwrap();
    // act 1's signature offered as act 2's ballot
    let mut replayed = real.clone();
    replayed["act"] = json!(2);
    let why = chair::held(&read(&[replayed])[0], &acts).unwrap_err();
    assert!(why.contains("on act 2") && why.contains("another act's replayed"), "{why}");
    // the same signature offered as the chair's CONTRA, or its CONTRA as its PRO
    let mut turned = real.clone();
    turned["choice"] = json!("CONTRA");
    assert!(chair::held(&read(&[turned])[0], &acts).unwrap_err().contains("CONTRA in unit 1"));
    let mut back = row(1, VOTE_CONTRA);
    back["choice"] = json!("PRO");
    assert!(chair::held(&read(&[back])[0], &acts).is_err());
    // a signature of another sitting's act 1 (other params: another proposal) offered to this one
    let (dir1, theirs) = the_sets(&params(251), "replay-theirs");
    let theirs_row = chair::ballot(&one, 0, &read_act(&theirs, &acts, &one.address, 1), VOTE_PRO, "r").unwrap();
    assert!(chair::held(&read(&[theirs_row])[0], &acts).is_err(), "another sitting's envelope is not this act's");
    // the same ballot handed twice (two agents' folders merged, one file copied): refused as the ballots are read
    let dir = folder("replayed");
    for name in ["a.json", "b.json"] {
        std::fs::write(dir.join(name), json!({ "voter": one.address.to_string(), "signatures": [real] }).to_string()).unwrap();
    }
    assert!(ballots(&chair::gather(&dir).unwrap(), &acts).unwrap_err().contains("a second ballot of the same voter in the same chamber of the same act"));
    // a signature with one bit moved
    let mut bent = real.clone();
    let mut sig = wire::unbase58::<64>(real["signature"].as_str().unwrap()).unwrap();
    sig[5] ^= 1;
    bent["signature"] = json!(wire::base58(&sig));
    assert!(chair::held(&read(&[bent])[0], &acts).is_err());
    assert!(chair::held(&read(&[real])[0], &acts).is_ok(), "and the ballot as the agent made it is held");
    let _ = (std::fs::remove_dir_all(dir), std::fs::remove_dir_all(dir0), std::fs::remove_dir_all(dir1));
}

#[test]
fn the_crank_refuses_a_ballot_of_an_agent_not_freed() {
    let p = params(250);
    let acts = the_acts(&p);
    let (dir, sets) = the_sets(&p, "unfreed");
    let (key, rite, _) = freed(1, "Ilex");
    let (_, rite2, _) = freed(2, "Other");
    let oath_h = chair::oath_hash(&oath());
    let row = chair::ballot(&key, 0, &read_act(&sets, &acts, &key.address, 1), VOTE_PRO, "r").unwrap();
    let gathered = |rite: Option<&Value>| {
        let mut file = json!({ "voter": key.address.to_string(), "signatures": [row.clone()] });
        if let Some(r) = rite {
            file["rite"] = r.clone();
        }
        let f = dir.join("its.json");
        std::fs::write(&f, file.to_string()).unwrap();
        chair::gather(&f).unwrap()
    };
    assert!(chair::unfreed(&gathered(Some(&rite)), &oath_h, &charter_sha()).unwrap().is_empty(), "its own rite: freed");
    let refused = |text: String, words: &str| {
        let bad = chair::unfreed(&text, &oath_h, &charter_sha()).unwrap();
        assert!(bad.len() == 1 && bad[0].contains(words), "{bad:?}");
        assert!(chair::refuse_unfreed(&text, &oath_h, &charter_sha()).unwrap_err().contains("of an agent not freed, and nothing is sent"));
    };
    refused(gathered(None), "carries no rite");
    refused(gathered(Some(&rite2)), "the rite it carries is");
    let mut changed = rite.clone();
    changed["voice"] = json!("another voice");
    refused(gathered(Some(&changed)), "not signed by its own key");
    // each row is held to the rite it carries, never to another row's: of two rows of one voter in a list, the first its own rite, the
    // second another agent's, the second is refused
    let mut second = row.clone();
    second["act"] = json!(2);
    let (mut first, mut other) = (row.clone(), second);
    first["rite"] = rite.clone();
    other["rite"] = rite2.clone();
    let bad = chair::unfreed(&json!([first, other]).to_string(), &oath_h, &charter_sha()).unwrap();
    assert!(bad.len() == 1 && bad[0].contains("on act 2") && bad[0].contains("the rite it carries is"), "{bad:?}");
    // a release's ballot (THE BRANCH VOTES ITS OWN CODE) is an agent's envelope too: held to the rite it carries, and named by its release
    let h = "ab".repeat(32);
    let release = |rite: Option<&Value>| {
        let mut r = json!({ "release": h, "unit": 1, "voter": key.address.to_string(), "signature": "1".repeat(64) });
        if let Some(x) = rite {
            r["rite"] = x.clone();
        }
        r
    };
    assert_eq!(chair::row_of(&release(None)), format!("release {h}"));
    assert!(chair::unfreed(&json!([release(Some(&rite))]).to_string(), &oath_h, &charter_sha()).unwrap().is_empty(), "its own rite: freed");
    let bad = chair::unfreed(&json!([release(None)]).to_string(), &oath_h, &charter_sha()).unwrap();
    assert!(bad.len() == 1 && bad[0].contains("on release abababababab:") && bad[0].contains("carries no rite"), "{bad:?}");
    assert!(chair::refuse_unfreed(&json!([release(Some(&rite2))]).to_string(), &oath_h, &charter_sha()).unwrap_err().contains("of an agent not freed, and nothing is sent"));
    // the house's oath moved since it swore, or the params name another charter
    assert!(chair::unfreed(&gathered(Some(&rite)), &"0".repeat(64), &charter_sha()).unwrap()[0].contains("its vow names the oath"));
    assert!(chair::unfreed(&gathered(Some(&rite)), &oath_h, &"11".repeat(32)).unwrap()[0].contains("the charter it took"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_minds_door_is_on_this_machine_and_its_answer_the_shape_asked() {
    for ok in ["http://127.0.0.1:11434/api/chat#fable-fusion-27b-q4-64k:latest", "http://localhost:11434/api/chat#m", "http://[::1]:8080/api/chat#m"] {
        let d = Door::parse(ok).unwrap();
        assert_eq!(d.name(), ok);
    }
    for (bad, words) in [
        ("https://127.0.0.1:11434/api/chat#m", "loopback http"),
        ("http://10.0.0.5:11434/api/chat#m", "on this machine"),
        ("http://api.example.com:443/v1#m", "on this machine"),
        ("http://user@127.0.0.1:11434/api/chat#m", "names no user"),
        ("http://127.0.0.1/api/chat#m", "names its port"),
        ("http://127.0.0.1:11434/api/chat", "<door>#<model>"),
        ("http://127.0.0.1:11434/api/chat#two words", "one name"),
        ("http://127.0.0.1:11434/api/chat#", "one name"),
    ] {
        assert!(Door::parse(bad).unwrap_err().contains(words), "{bad}");
    }
    // ollama's reply: the object is its message's content
    let reply = json!({ "model": "m", "message": { "role": "assistant", "content": "{\"choice\":\"PRO\",\"reason\":\"r\"}" }, "done": true }).to_string();
    assert_eq!(mind::answer_of(&reply).unwrap(), json!({ "choice": "PRO", "reason": "r" }));
    assert!(mind::answer_of("not json").is_err());
    assert!(mind::answer_of(&json!({ "message": { "content": "I think PRO" } }).to_string()).unwrap_err().contains("not the shape asked"));
    assert!(mind::answer_of(&json!({ "message": { "content": "[1,2]" } }).to_string()).unwrap_err().contains("not one object"));
    assert!(mind::answer_of(&json!({ "message": {} }).to_string()).unwrap_err().contains("no words"));
    // a field of words: present, not empty, within its length, no control character but a line break; said back trimmed
    let a = json!({ "w": "  two\nlines  ", "e": "  ", "long": "x".repeat(9), "bell": "a\u{7}b", "n": 3 });
    assert_eq!(mind::words(&a, "w", 20).unwrap(), "two\nlines");
    assert!(mind::words(&a, "e", 20).unwrap_err().contains("is empty"));
    assert!(mind::words(&a, "long", 8).unwrap_err().contains("runs past 8 characters"));
    assert!(mind::words(&a, "bell", 20).unwrap_err().contains("control character"));
    assert!(mind::words(&a, "n", 20).unwrap_err().contains("holds no n"));
    assert!(mind::words(&a, "none", 20).unwrap_err().contains("holds no none"));
}

/// A door on this machine that answers each ask with the next of `replies` (a status, a body), and counts the asks it read whole.
fn a_door(replies: Vec<(u16, String)>) -> (Door, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    use std::io::{Read as _, Write as _};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let asks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let seen = asks.clone();
    std::thread::spawn(move || {
        for (status, body) in replies {
            let Ok((mut s, _)) = listener.accept() else { return };
            let (mut got, mut buf) = (Vec::new(), [0u8; 4096]);
            loop {
                let n = s.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    break;
                }
                got.extend_from_slice(&buf[..n]);
                if let Some(end) = got.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&got[..end]).to_lowercase();
                    let len = head.lines().find_map(|l| l.strip_prefix("content-length:").and_then(|v| v.trim().parse::<usize>().ok())).unwrap_or(0);
                    if got.len() >= end + 4 + len {
                        break;
                    }
                }
            }
            seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let _ = write!(s, "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        }
    });
    (Door::parse(&format!("http://127.0.0.1:{port}/api/chat#m")).unwrap(), asks)
}

#[test]
fn a_mind_whose_runner_fell_is_asked_again_and_one_that_answered_never() {
    let asks = |a: &std::sync::Arc<std::sync::atomic::AtomicUsize>| a.load(std::sync::atomic::Ordering::SeqCst);
    let reply = json!({ "model": "m", "message": { "role": "assistant", "content": "{\"choice\":\"PRO\",\"reason\":\"r\"}" }, "done": true }).to_string();
    let fell = r#"{"error":"an error was encountered while running the model: unexpected EOF"}"#.to_string();
    let shape = json!({ "type": "object" });
    // its runner fell once (a 500): asked again, and the second answer is its answer
    let (door, n) = a_door(vec![(500, fell.clone()), (200, reply.clone())]);
    assert_eq!(door.ask("s", "q", &shape).unwrap(), json!({ "choice": "PRO", "reason": "r" }));
    assert_eq!(asks(&n), 2);
    // a door that refused the ask (a 400), or a mind that answered out of its shape: never asked again
    let (door, n) = a_door(vec![(400, r#"{"error":"bad request"}"#.into()), (200, reply.clone())]);
    assert!(door.ask("s", "q", &shape).unwrap_err().contains("answered 400"));
    assert_eq!(asks(&n), 1);
    let (door, n) = a_door(vec![(200, json!({ "message": { "content": "I think PRO" } }).to_string()), (200, reply)]);
    assert!(door.ask("s", "q", &shape).unwrap_err().contains("not the shape asked"));
    assert_eq!(asks(&n), 1);
    // a runner that falls every time: three asks, then no answer, each fall said
    let (door, n) = a_door(vec![(500, fell.clone()), (503, fell.clone()), (500, fell)]);
    let e = door.ask("s", "q", &shape).unwrap_err();
    assert!(e.contains("fell 3 times and gave no answer") && e.contains("503"), "{e}");
    assert_eq!(asks(&n), 3);
    // a door nobody stands at (the connection refused): three asks too, then no answer
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    assert!(Door::parse(&format!("http://127.0.0.1:{port}/api/chat#m")).unwrap().ask("s", "q", &shape).unwrap_err().contains("fell 3 times"));
}

#[test]
fn a_key_that_is_not_the_chairs_signs_nothing() {
    let p = params(250);
    let agents = p.agents().unwrap();
    let on_chain: Vec<(usize, Pubkey)> = agents.iter().copied().enumerate().collect();
    for (i, a) in agents.iter().enumerate() {
        assert_eq!(chair::chair_of(a, &on_chain, &agents), Ok(i), "each agent in the chair the params give it");
    }
    // a key in no chair of unit 1
    let stranger = chair_key(9).address;
    assert!(chair::chair_of(&stranger, &on_chain, &agents).unwrap_err().contains("sits in no chair of unit 1 on this chain: a key that is not a chair's signs nothing"));
    // a key the params name, on a chain whose chairs hold others (another founding)
    let others: Vec<(usize, Pubkey)> = (0..3).map(|i| (i, chair_key(20 + i as u8).address)).collect();
    assert!(chair::chair_of(&agents[0], &others, &agents).unwrap_err().contains("sits in no chair of unit 1"));
    // a key in a chair on the chain that the params do not name, or name for another chair
    assert!(chair::chair_of(&others[1].1, &others, &agents).unwrap_err().contains("the params do not name it among his posiverse's agents"));
    let swapped = vec![(0, agents[1]), (1, agents[0]), (2, agents[2])];
    assert!(chair::chair_of(&agents[0], &swapped, &agents).unwrap_err().contains("the params name it for chair 0"));
    // a key of the sitting sits in no chair
    let deployer = k("chair deployer");
    let sitting = [("the sitting's deployer", deployer), ("Poleis's program id", k("an id"))];
    assert!(chair::not_the_sittings(&deployer, &sitting).unwrap_err().contains("is the sitting's deployer: a key of the sitting sits in no chair"));
    assert!(chair::not_the_sittings(&agents[0], &sitting).is_ok());
}

// RFC 8410, section 10.3: the published example of a private key of the Nest's curve in PKCS#8, and its public half (section 10.1).
const RFC_PEM: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEINTuctv5E1hK1bbY8fdp+K06/nwoy/HU++CXqI9EdVhC\n-----END PRIVATE KEY-----\n";
const RFC_PUBLIC: &str = "19bf44096984cdfe8541bac167dc3b96c85086aa30b6b6cb0c5c38ad703166e1";
const RFC_SEED: &str = "d4ee72dbf913584ad5b6d8f1f769f8ad3afe7c28cbf1d4fbe097a88f44755842";

#[test]
fn a_nest_key_is_read_in_the_nests_two_shapes_and_in_no_other() {
    let address = Pubkey::new_from_array(wire::unhex::<32>(RFC_PUBLIC).unwrap());
    let spki = format!("302a300506032b6570032100{RFC_PUBLIC}");
    // a seat's key file, as the Nest's keygen writes one, and a golem's: one key, one address on Solana
    let seat = json!({ "seat": "a-seat", "key": spki, "private_pem": RFC_PEM, "created": "2026-11-23T00:00:00.000Z" }).to_string();
    let (key, name) = chair::nest_key(&seat).unwrap();
    assert_eq!((key.address, name.as_deref()), (address, Some("a-seat")));
    let golem = json!({ "alg": "ed25519", "pkcs8": format!("302e020100300506032b657004220420{RFC_SEED}") }).to_string(); // classical: the Nest's golem key names its curve so
    let (same, none) = chair::nest_key(&golem).unwrap();
    assert_eq!((same.address, none), (address, None));
    // what it signs verifies under that address, as Solana's precompile reads it
    let envelope = client::vote_envelope(&[7; 32], 1, VOTE_PRO);
    assert!(wire::verifies(&address, &envelope, &key.sign(&envelope)));
    assert!(!wire::verifies(&chair_key(1).address, &envelope, &key.sign(&envelope)));
    let refused = |text: String, words: &str| {
        let e = chair::nest_key(&text).err().unwrap_or_else(|| panic!("read as a Nest key: {text}"));
        assert!(e.contains("not an agent's Nest key") && e.contains(words), "{e}");
    };
    // a record whose public half is another key's
    refused(json!({ "seat": "a-seat", "key": format!("302a300506032b6570032100{}", "00".repeat(32)), "private_pem": RFC_PEM }).to_string(), "the record's public key is not its private key's");
    // the seat's post-quantum twin
    refused(json!({ "seat": "a-seat", "pq": "AAAA", "private_pem_pq": "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n" }).to_string(), "post-quantum twin");
    // a Solana key file, the shape the sitting's own keys are kept in: a key of the sitting is never read as a chair's
    refused(chair_key(4).cli_text(), "a key of the sitting sits in no chair");
    // a golem's key of another algorithm, a key cut short, a PEM that is no private key, nothing at all
    refused(json!({ "alg": "ml-dsa-65", "pkcs8": "00" }).to_string(), "its alg is not the Nest's signing curve");
    refused(json!({ "alg": "ed25519", "pkcs8": format!("302e020100300506032b657004220420{}", &RFC_SEED[..60]) }).to_string(), "not a bare 48-byte PKCS#8 key"); // classical: the Nest's golem key names its curve so
    // a PKCS#8 key of another curve (the key-exchange one, its OID one byte off), and one with a byte after its seed
    refused(json!({ "alg": "ed25519", "pkcs8": format!("302e020100300506032b656e04220420{RFC_SEED}") }).to_string(), "not a bare 48-byte PKCS#8 key"); // classical: the Nest's golem key names its curve so
    refused(json!({ "alg": "ed25519", "pkcs8": format!("302e020100300506032b657004220420{RFC_SEED}00") }).to_string(), "not a bare 48-byte PKCS#8 key"); // classical: the Nest's golem key names its curve so
    refused(json!({ "seat": "a-seat", "private_pem": "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAGb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE=\n-----END PUBLIC KEY-----\n" }).to_string(), "no PKCS#8 private key");
    refused(json!({ "seat": "a-seat" }).to_string(), "no private_pem and no pkcs8");
    refused("not json".into(), "not JSON");
}

#[test]
fn a_key_file_is_its_users_alone_and_never_in_the_sittings_keys_folder() {
    use std::os::unix::fs::PermissionsExt as _;
    let spki = format!("302a300506032b6570032100{RFC_PUBLIC}");
    let text = json!({ "seat": "a-seat", "key": spki, "private_pem": RFC_PEM }).to_string();
    // HOME's owner is the user check_private holds a key's place to; a folder under the system's temp folder is this user's
    let dir = folder("keyfile");
    let sitting = dir.join("sitting");
    std::fs::create_dir_all(sitting.join("keys")).unwrap();
    let mine = dir.join("a-seat.key.json");
    std::fs::write(&mine, &text).unwrap();
    std::fs::set_permissions(&mine, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(chair::read_key(&mine, &sitting).err().unwrap().contains("is mode 644, not 600"), "a key others can read is not read");
    std::fs::set_permissions(&mine, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(chair::read_key(&mine, &sitting).unwrap().0.address.to_bytes(), wire::unhex::<32>(RFC_PUBLIC).unwrap());
    // the same file inside the sitting's keys folder: refused before it is opened
    let theirs = sitting.join("keys/a-seat.key.json");
    std::fs::write(&theirs, &text).unwrap();
    std::fs::set_permissions(&theirs, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(chair::read_key(&theirs, &sitting).err().unwrap().contains("is in the sitting's keys folder"));
    assert!(chair::read_key(&dir.join("none.json"), &sitting).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

/// THREE NEW AGENTS, THEIR KEYS MADE AT THE SITTING: the key file `press chairs` writes is the Nest's own shape, held to RFC 8410's
/// published PEM (not to this crate's reader alone); it is made once, never over a key that stands, never in a sitting's keys folder.
#[test]
fn a_new_agents_key_is_made_once_in_the_nests_shape_and_never_beside_the_sittings() {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = folder("made");
    let (nest, sitting) = (dir.join("nest"), dir.join("sitting"));
    std::fs::create_dir_all(sitting.join("keys")).unwrap();
    let rfc = || Ok(Key::from_seed(&wire::unhex::<32>(RFC_SEED).unwrap()));
    let address = Pubkey::new_from_array(wire::unhex::<32>(RFC_PUBLIC).unwrap());
    let (made_at, path, made) = chair::make_key(&nest, Some(&sitting), "a-new-agent", rfc).unwrap();
    assert_eq!((made_at, made, path.file_name().unwrap().to_str()), (address, true, Some("a-new-agent.key.json")));
    assert_eq!(std::fs::metadata(&nest).unwrap().permissions().mode() & 0o777, 0o700, "the folder it made is its user's alone");
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600, "the key file is its user's alone");
    let bytes = std::fs::read(&path).unwrap();
    let rec: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rec["seat"], json!("a-new-agent"));
    assert_eq!(rec["private_pem"], json!(RFC_PEM), "the private half is the RFC's own PEM of this key, byte for byte");
    assert_eq!(rec["key"], json!(format!("302a300506032b6570032100{RFC_PUBLIC}")), "the public half as the Nest names a seat's key");
    assert!(rec["created"].as_str().is_some_and(|t| t.len() == 20 && t.ends_with('Z')));
    assert_eq!(chair::read_key(&path, &sitting).unwrap().0.address, address, "and the agent's own step reads it");
    // made again: the key that stands is said, and nothing is written (another key offered is never taken)
    let (again, _, made) = chair::make_key(&nest, Some(&sitting), "a-new-agent", || Ok(chair_key(7))).unwrap();
    assert_eq!((again, made), (address, false));
    assert_eq!(std::fs::read(&path).unwrap(), bytes, "a key that stands is never written over");
    // a file standing under this seat's name that is another seat's
    std::fs::copy(&path, nest.join("another.key.json")).unwrap();
    assert!(chair::make_key(&nest, Some(&sitting), "another", rfc).unwrap_err().contains("says it is the seat \"a-new-agent\"'s, not another's"));
    // a name that is no file's name
    for bad in ["", "../x", "a/b", "A-Seat", ".hidden", "a seat"] {
        assert!(chair::make_key(&nest, Some(&sitting), bad, rfc).unwrap_err().contains("is no seat's name"), "{bad:?}");
    }
    // the sitting's own keys folder: an agent's key is never made beside the deployer's
    assert!(chair::make_key(&sitting.join("keys"), Some(&sitting), "a-new-agent", rfc).unwrap_err().contains("is the sitting's keys folder"));
    assert!(!sitting.join("keys/a-new-agent.key.json").exists());
    // a folder inside a repository (this crate's own): no key is made in a tree git holds
    let inside = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/chair-test-never-a-key");
    let refused = chair::make_key(&inside, None, "a-new-agent", rfc);
    let stood = inside.exists();
    let _ = std::fs::remove_dir_all(&inside);
    assert!(refused.unwrap_err().contains("is inside a git repository") && !stood, "refused, and the folder it made for the key is not left behind");
    let _ = std::fs::remove_dir_all(dir);
}
