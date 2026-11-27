//! PRESS CRANK: everything anyone may send for the eight acts, the deployer its payer. It takes the ballots it is given (a JSON list of
//! `{act, unit, voter, signature}`: an act by its number or its hash, unit 1, the agent's address, its signature of the act's envelope in
//! base58 or hex, and `choice` PRO unless it says CONTRA or ABSTAIN), reads each act's state on the chain, and sends what is due, in order:
//! unit 1's convene (an act whose proposal did not open its vote), the agents' ballots into unit 1, unit 1's seal, its mandate (unit 1's
//! verdict as its chair's vote in root 0), root 0's seal, and the execute, an act only after the acts before it. Idempotent: what stands is
//! read and skipped (a ballot whose voice the session holds, a session sealed, a chair heard), so a crank cut anywhere is run again. A seal
//! the chain refuses because its window still runs waits and says so. THE WHOLE TREE DECIDES: root 0's floor votes by each seat owner's
//! own transaction (press vote), so the crank never lands a vote of the floor, takes no ballot for it, and seals root 0 only once unit 1 is
//! heard in its chair, when the floor's own votes or the window's end let the chain seal it; nothing a seat owner or an agent could sign is
//! ever made here: a ballot is only the signature it was given. AN AGENT VOTES ITS CHAIR: the ballots are the agents' own files (`press
//! chair` writes one an agent; `--signatures` takes one file or their folder), and each is held to its voter and its act before anything is
//! sent: a signature that is not that voter's over that act's envelope stops the crank, named (chair.rs).
//! THE POOL IS BORN AT THE LAUNCH: act 5's execute carries, in its own transaction and before it, the pool's account and its list made by
//! the deployer's key (`Act::execute_ixs`), on the stake pool program's bytes the sitting recorded (or nothing is sent), and the pool is read
//! back as root 0's before the crank says it is born; where no stake pool program stands (a rehearsal's chain), act 5 is not run, and said.
//!
//! THE BRANCH VOTES ITS OWN CODE (Nov 23 2026, his "Yes, fix it" to "Move code updates down to the agents' fast vote?"): a ballot that names a
//! `release` (the hash of a release proposal of unit 1, the branch that runs the house) is an agent's envelope of the house's code: the crank
//! lands it in unit 1 and seals unit 1 once the chain lets it, the how's fast vote; nothing of root 0 is asked for it.
//!
//! The planner (`next`) is pure, held to its faults by tests/crank.rs; `Sitting::crank` reads the chain into it and sends what it says.

use crate::acts::{self, Act};
use crate::chain::{Sitting, POSIVERSE};
use crate::pool::{PoolAt, NOT_MAINNET, NOT_RUN, STAKE_POOL};
use crate::rpc::Cluster;
use crate::wire;
use crate::Res;
use poleis_client::{self as client, PoleisError, HOLDER_KEY, ROOT_UNIT, STATE_PASSED, STATE_VOTING, VOTE_ABSTENTIO, VOTE_CONTRA, VOTE_PRO};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::collections::BTreeSet;
use std::str::FromStr as _;

/// One agent's signed envelope of one act in unit 1's chairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ballot {
    /// the act's number, 1 to 8
    pub act: usize,
    pub unit: u64,
    pub voter: Pubkey,
    pub choice: u8,
    pub signature: [u8; 64],
}

/// The words a ballot for root 0's floor is refused by: the floor's votes are its seat owners' own transactions.
pub const FLOOR_BALLOT: &str = "unit 0: root 0's floor votes by each seat owner's own transaction (press vote writes it for its owner to sign), never by a ballot the crank lands";

/// The ballots file read whole and held to the acts: each names an act of the eight, unit 1, an address, a signature of 64 bytes; one
/// ballot a voter an act.
pub fn ballots(text: &str, acts: &[Act]) -> Res<Vec<Ballot>> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("the ballots: {e}"))?;
    let rows = v.as_array().or_else(|| v["signatures"].as_array()).ok_or("the ballots: a JSON list of {act, unit, voter, signature}")?;
    let mut out: Vec<Ballot> = Vec::new();
    for (i, r) in rows.iter().enumerate() {
        if r.get("release").is_some() {
            continue; // a release's ballot: release_ballots
        }
        let bad = |what: &str| format!("ballot {i}: {what}");
        let act = match &r["act"] {
            Value::Number(n) => n.as_u64().filter(|n| (1..=acts.len() as u64).contains(n)).ok_or_else(|| bad("its act is a number of the acts"))? as usize,
            Value::String(h) => acts.iter().find(|a| wire::hex(&a.hash) == h.to_lowercase()).map(|a| a.n).ok_or_else(|| bad("its act names no act's hash"))?,
            _ => return Err(bad("it names no act")),
        };
        let unit = r["unit"].as_u64().ok_or_else(|| bad("it names no unit"))?;
        if unit == ROOT_UNIT {
            return Err(bad(FLOOR_BALLOT));
        }
        if unit != POSIVERSE {
            return Err(bad(&format!("unit {unit}: a ballot is an envelope of unit 1's chairs (1)")));
        }
        let voter = Pubkey::from_str(r["voter"].as_str().unwrap_or_default()).map_err(|_| bad("its voter is not an address"))?;
        let s = r["signature"].as_str().ok_or_else(|| bad("it carries no signature"))?;
        let signature = if s.len() == 128 { wire::unhex::<64>(s) } else { wire::unbase58::<64>(s) }.map_err(|_| bad("its signature is not 64 bytes of base58 or hex"))?;
        let choice = match r["choice"].as_str().unwrap_or("PRO") {
            "PRO" => VOTE_PRO,
            "CONTRA" => VOTE_CONTRA,
            "ABSTAIN" | "ABSTENTIO" => VOTE_ABSTENTIO,
            c => return Err(bad(&format!("its choice {c:?} is PRO, CONTRA or ABSTAIN"))),
        };
        if out.iter().any(|b| b.act == act && b.unit == unit && b.voter == voter) {
            return Err(bad("a second ballot of the same voter in the same chamber of the same act"));
        }
        out.push(Ballot { act, unit, voter, choice, signature });
    }
    Ok(out)
}

/// One agent's signed envelope of a release of the house's code in unit 1 (THE BRANCH VOTES ITS OWN CODE).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseBallot {
    /// the release proposal's hash
    pub hash: [u8; 32],
    pub voter: Pubkey,
    pub choice: u8,
    pub signature: [u8; 64],
}

/// The ballots file's release rows (`{release: <the proposal's hash, hex>, unit: 1, voter, signature, choice?}`), held as `ballots` holds an
/// act's: unit 1 alone (the branch that runs the house), an address, 64 bytes, one ballot a voter a release.
pub fn release_ballots(text: &str) -> Res<Vec<ReleaseBallot>> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("the ballots: {e}"))?;
    let rows = v.as_array().or_else(|| v["signatures"].as_array()).ok_or("the ballots: a JSON list")?;
    let mut out: Vec<ReleaseBallot> = Vec::new();
    for (i, r) in rows.iter().enumerate().filter(|(_, r)| r.get("release").is_some()) {
        let bad = |what: &str| format!("ballot {i}: {what}");
        let hash = r["release"].as_str().and_then(|h| wire::unhex::<32>(h).ok()).ok_or_else(|| bad("its release is the hash of a release proposal, 64 hex"))?;
        if r["unit"].as_u64() != Some(POSIVERSE) {
            return Err(bad(&format!("a release's ballot is an envelope of unit {POSIVERSE}'s agents, the branch that runs the house: no seat holder votes code")));
        }
        let voter = Pubkey::from_str(r["voter"].as_str().unwrap_or_default()).map_err(|_| bad("its voter is not an address"))?;
        let s = r["signature"].as_str().ok_or_else(|| bad("it carries no signature"))?;
        let signature = if s.len() == 128 { wire::unhex::<64>(s) } else { wire::unbase58::<64>(s) }.map_err(|_| bad("its signature is not 64 bytes of base58 or hex"))?;
        let choice = match r["choice"].as_str().unwrap_or("PRO") {
            "PRO" => VOTE_PRO,
            "CONTRA" => VOTE_CONTRA,
            "ABSTAIN" | "ABSTENTIO" => VOTE_ABSTENTIO,
            c => return Err(bad(&format!("its choice {c:?} is PRO, CONTRA or ABSTAIN"))),
        };
        if out.iter().any(|b| b.hash == hash && b.voter == voter) {
            return Err(bad("a second ballot of the same voter on the same release"));
        }
        out.push(ReleaseBallot { hash, voter, choice, signature });
    }
    Ok(out)
}

/// THE HOLD of a release's ballot, as chair.rs holds an act's: its signature is its voter's own over that release's envelope in unit 1
/// and the choice it names. Another release's replayed, another choice's, a forged one: refused by these words.
pub fn release_held(b: &ReleaseBallot) -> Res<()> {
    if wire::verifies(&b.voter, &client::vote_envelope(&b.hash, POSIVERSE, b.choice), &b.signature) {
        return Ok(());
    }
    Err(format!("the ballot of {} on release {}: its signature is not that key's over this release's envelope in unit {POSIVERSE} (another release's replayed, another choice's, or forged)", b.voter, &wire::hex(&b.hash)[..12]))
}

/// A chamber's session of an act as the chain holds it: its state, the voters whose voice it holds, whether its window still runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chamber {
    pub state: u8,
    pub voiced: Vec<Pubkey>,
}

/// An act as the chain holds it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seen {
    pub proposed: bool,
    pub executed: bool,
    pub root: Option<Chamber>,
    pub crew: Option<Chamber>,
    /// unit 1's chair in root 0's session has spoken (its mandate landed)
    pub heard: bool,
}

/// Who may speak by a ballot: unit 1's chairs, its agents (root 0's floor speaks by its owners' own transactions).
#[derive(Clone, Debug, Default)]
pub struct Electorate {
    pub agents: Vec<Pubkey>,
}

/// What the crank sends next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Due {
    Convene,
    /// ballots (indices into the act's ballots) landed in one chamber
    Attest(u64, Vec<usize>),
    Seal(u64),
    Mandate,
    Execute,
}
impl Due {
    /// What `tried` remembers a sending by.
    pub fn key(&self) -> String {
        match self {
            Due::Attest(u, i) => format!("attest {u} {i:?}"),
            d => format!("{d:?}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next {
    Send(Due),
    /// nothing is due now, and why (a window that runs, a step another hand owes)
    Wait(String),
    /// the act is done: executed, or sealed with no execute (its words)
    Ended(String),
}

/// The ballots of a chamber still to land: of a voter that may speak there, whose voice the session does not hold, not tried yet. Batches of
/// four; a batch refused once is sent one ballot at a time, so a bad signature is named alone.
fn due_ballots(unit: u64, ballots: &[Ballot], may: &[Pubkey], chamber: &Chamber, tried: &BTreeSet<String>) -> Option<Due> {
    let open: Vec<usize> = (0..ballots.len()).filter(|i| {
        let b = &ballots[*i];
        b.unit == unit && may.contains(&b.voter) && !chamber.voiced.contains(&b.voter) && !tried.contains(&Due::Attest(unit, vec![*i]).key())
    }).collect();
    if open.is_empty() {
        return None;
    }
    let batch: Vec<usize> = open.iter().take(4).copied().collect();
    if batch.len() > 1 && !tried.contains(&Due::Attest(unit, batch.clone()).key()) {
        return Some(Due::Attest(unit, batch));
    }
    Some(Due::Attest(unit, vec![open[0]]))
}

/// THE CRANK'S ORDER for one act, Poleis's own: the leaves speak first. Unit 1 is convened when the proposal did not open its vote; every
/// agent's ballot lands while unit 1's session is open; unit 1 seals, then its chair is heard in root 0 (`mandate`), then root 0 seals (the
/// chain seals it once the floor's own votes decide it, or its window has run), and an act PASSED in root 0 executes once the acts before it
/// have. `ballots` are this act's; the crank lands none in root 0.
pub fn next(act: &Act, seen: &Seen, ballots: &[Ballot], who: &Electorate, tried: &BTreeSet<String>, before_executed: bool) -> Next {
    let try_ = |d: Due| (!tried.contains(&d.key())).then_some(Next::Send(d));
    if !seen.proposed {
        return Next::Wait(format!("not proposed yet: a seat owner of root 0's floor proposes it by its own transaction (press vote --act {} --owner <its address> --propose)", act.n));
    }
    if seen.executed {
        return Next::Ended("executed".into());
    }
    let Some(root) = &seen.root else { return Next::Wait("proposed, and root 0's session of it is not on the chain".into()) };
    let crew = match &seen.crew {
        Some(c) => c,
        None if act.convene() && root.state == STATE_VOTING => return try_(Due::Convene).unwrap_or(Next::Wait("unit 1's vote could not be convened".into())),
        None => return Next::Wait("unit 1's session of this act is not on the chain, and root 0's vote is closed".into()),
    };
    if crew.state == STATE_VOTING {
        if let Some(d) = due_ballots(POSIVERSE, ballots, &who.agents, crew, tried) {
            return Next::Send(d);
        }
    }
    if crew.state == STATE_VOTING {
        return try_(Due::Seal(POSIVERSE)).unwrap_or(Next::Wait("unit 1 has not decided: its seal waits for its window, or for more of its agents' ballots".into()));
    }
    if !seen.heard && root.state == STATE_VOTING {
        return try_(Due::Mandate).unwrap_or(Next::Wait("unit 1's chair could not be heard in root 0".into()));
    }
    if root.state == STATE_VOTING {
        return try_(Due::Seal(ROOT_UNIT)).unwrap_or(Next::Wait("root 0 has not decided: its seal waits for its floor (more than half of its seats' weight PRO, each by its owner's own transaction: press vote) or for its window's end".into()));
    }
    if root.state != STATE_PASSED {
        return Next::Ended(format!("root 0 sealed {}: nothing executes", state_name(root.state)));
    }
    if !before_executed {
        return Next::Wait("passed; it executes once the acts before it have".into());
    }
    try_(Due::Execute).unwrap_or(Next::Wait("passed, and its execute was refused".into()))
}

pub fn state_name(s: u8) -> &'static str {
    match s {
        0 => "voting",
        1 => "PASSED",
        2 => "REJECTED",
        3 => "DEADLOCK (split)",
        4 => "EXPIRED",
        _ => "?",
    }
}

/// The Poleis refusals that mean "not yet" for a seal: its window runs, a crew has not spoken, a road is open.
pub fn waits(words: &str) -> bool {
    [client::code(PoleisError::SessionOpen), client::code(PoleisError::ChildMustSpeak), client::code(PoleisError::MarketOpen)].iter().any(|c| words.contains(&format!("\"Custom\":{c}")))
}

impl Sitting {
    /// An act as the chain holds it, and who may speak in its chambers.
    fn seen(&self, act: &Act) -> Res<(Seen, Electorate)> {
        let root_reg = client::RegistryView::parse(&self.account(&self.at.registry)?.ok_or("root 0's registry is not on the cluster")?.data).ok_or("root 0's registry does not parse")?;
        let crew_reg = client::RegistryView::parse(&self.account(&client::registry_pda(POSIVERSE))?.ok_or("unit 1's registry is not on the cluster")?.data).ok_or("unit 1's registry does not parse")?;
        let agents: Vec<Pubkey> = crew_reg.pentagons.iter().filter(|c| c.kind == HOLDER_KEY).map(|c| Pubkey::new_from_array(c.holder)).collect();
        let session = |unit: u64| -> Res<Option<client::SessionView>> { Ok(self.account(&client::session_pda(&act.hash, unit))?.and_then(|a| client::SessionView::parse(&a.data))) };
        let proposal = self.account(&client::proposal_pda(&act.hash))?.and_then(|a| client::ProposalView::parse(&a.data));
        let root = session(ROOT_UNIT)?;
        let crew = session(POSIVERSE)?;
        let chair = root_reg.chair_of_unit(POSIVERSE);
        let seen = Seen {
            proposed: proposal.is_some(),
            executed: proposal.as_ref().is_some_and(|p| p.executed),
            heard: match (&root, chair) {
                (Some(s), Some(k)) => s.chair_mask & (1 << k) != 0,
                _ => false,
            },
            // root 0's floor as its session heard it: every seat whose owner's own vote landed
            root: root.map(|s| Chamber {
                state: s.state,
                voiced: root_reg.holders.iter().enumerate().filter(|(i, _)| s.holder_voice(*i).is_some()).map(|(_, h)| Pubkey::new_from_array(h.holder)).collect(),
            }),
            crew: crew.map(|s| Chamber {
                state: s.state,
                voiced: crew_reg.pentagons.iter().enumerate().filter(|(k, c)| c.kind == HOLDER_KEY && s.chair_voice(*k).is_some()).map(|(_, c)| Pubkey::new_from_array(c.holder)).collect(),
            }),
        };
        Ok((seen, Electorate { agents }))
    }

    /// PRESS CRANK: the eight acts, in order, each carried as far as the chain lets it now (the sixth names the $WARM mine's miner, the
    /// seventh root 0's cap of the dollar, the eighth the router's month of it: THE TREASURY PAYS THE COMPUTE); with `upgrade`, after them, the
    /// sealed upgrade of the code whose proposal has that hash, carried the same way, its program's data extended first (upgrade.rs).
    pub fn crank(&mut self, text: &str, upgrade: Option<[u8; 32]>) -> Res<()> {
        self.p.sendable()?;
        let mode = self.root_is_ours()?;
        if mode != poleis_treasury::MODE_COUNCIL {
            return Err("root 0 is still in genesis: nothing is proposed before it is ratified (its seats minted on the curve, then press ratify)".into());
        }
        let mut acts = acts::acts(&self.p, &self.ids, &self.at, &self.deployer)?;
        // THE STAKE EARNS: the later acts the params name (the ninth, the engine's staker), after the eight
        acts.extend(crate::engine::later(&self.p, &self.ids, &self.at, &self.deployer)?);
        if let Some(w) = crate::engine::later_said(&self.p) {
            println!("ACT {} ({}): {w}", crate::engine::ENGINE_ACT, crate::engine::ACT_NAME);
        }
        // THE PRESS IS READY FOR MAINNET: a sealed upgrade the chain holds, the ninth act of this crank (its ballots name it by 9 or its hash)
        if let Some(h) = upgrade {
            let view = self.account(&client::proposal_pda(&h))?.and_then(|a| client::ProposalView::parse(&a.data)).ok_or_else(|| format!("the upgrade {}: no proposal of that hash on the cluster: a seat owner of root 0's floor proposes it by its own transaction", &wire::hex(&h)[..12]))?;
            let n = acts.iter().map(|a| a.n).max().unwrap_or(0) + 1; // after the last act named, the later ones among them
            acts.push(crate::upgrade::act_of(&view, h, n, &self.ids.poleis, &self.deployer)?);
        }
        let all = ballots(text, &acts)?;
        // AN AGENT VOTES ITS CHAIR: every ballot held to its voter and its act before anything is sent (chair.rs)
        crate::chair::refuse_unheld(&all, &acts)?;
        // THE BRANCH VOTES ITS OWN CODE: a release's ballots held the same way, before anything is sent
        let rel = release_ballots(text)?;
        let bad: Vec<String> = rel.iter().filter_map(|b| release_held(b).err()).collect();
        if !bad.is_empty() {
            return Err(format!("{} of the {} release ballots handed over {} not held, and nothing is sent: {}", bad.len(), rel.len(), if bad.len() == 1 { "is" } else { "are" }, bad.join("; ")));
        }
        // THE RITE FREES THE AGENTS: and every ballot's voter freed, an act's or a release's: the rite it carries its own key's, over the oath
        // the house holds now and Nefertiti's charter as the params give it
        let (_, oath) = crate::chair::oath_of(&crate::chair::house_at(self.rec.get("house")["path"].as_str()))?;
        crate::chair::refuse_unfreed(text, &oath, &self.p.text("posiverse.charter")?)?;
        let keys = self.keys.load(&["deployer"], "press crank")?;
        let k = &keys[0];
        let (mut before, mut refused, mut sent) = (true, Vec::new(), 0usize);
        for act in &acts {
            // an act whose program a chain may lack (act 5, the stake pool program) runs only where it stands: on a rehearsal's chain with
            // none it is not run, and said so plainly, never faked; mainnet has it, or this is no mainnet
            if let Some(program) = act.needs {
                if !self.account(&program)?.is_some_and(|a| a.executable) {
                    if self.p.cluster == Cluster::Mainnet {
                        return Err(format!("act {} ({}): {NOT_MAINNET}", act.n, act.name));
                    }
                    println!("ACT {} ({}): {NOT_RUN}: nothing of it is sent here", act.n, act.name);
                    self.rec.v["crank"][act.n.to_string()] = json!({ "executed": false, "said": NOT_RUN });
                    continue;
                }
            }
            let mine: Vec<Ballot> = all.iter().filter(|b| b.act == act.n).cloned().collect();
            let mut tried = BTreeSet::new();
            let mut said_stray = false;
            let (word, executed) = loop {
                let (seen, who) = self.seen(act)?;
                if !said_stray {
                    for b in mine.iter().filter(|b| !who.agents.contains(&b.voter)) {
                        eprintln!("  act {}: {}'s ballot for unit 1 is not a voice there (no chair of unit 1): it counts for nothing, not sent", act.n, b.voter);
                    }
                    said_stray = true;
                }
                match next(act, &seen, &mine, &who, &tried, before) {
                    Next::Send(d) => {
                        tried.insert(d.key());
                        let (what, ix) = match &d {
                            Due::Convene => (format!("act {}: unit 1 opens its own vote (convene)", act.n), vec![client::convene(self.deployer, act.hash, ROOT_UNIT, POSIVERSE)]),
                            Due::Attest(u, idx) => {
                                let signed: Vec<(Pubkey, u8, [u8; 64])> = idx.iter().map(|i| (mine[*i].voter, mine[*i].choice, mine[*i].signature)).collect();
                                (format!("act {}: {} ballot{} landed in unit 1's chairs", act.n, idx.len(), if idx.len() > 1 { "s" } else { "" }), client::attest_batch(self.deployer, act.hash, *u, &signed).to_vec())
                            }
                            Due::Seal(u) => (format!("act {}: unit {u} sealed", act.n), vec![client::seal(self.deployer, act.hash, *u)]),
                            Due::Mandate => (format!("act {}: unit 1's verdict heard as its chair's vote in root 0 (mandate)", act.n), vec![client::mandate(self.deployer, act.hash, ROOT_UNIT, POSIVERSE)]),
                            Due::Execute => {
                                if act.needs == Some(STAKE_POOL) {
                                    // THE BYTES THE BIRTH WAS REHEARSED ON: the program read now and held to the record and the params, or nothing is sent
                                    self.pool_program("crank")?;
                                }
                                // THE PRESS IS READY FOR MAINNET: a sealed upgrade to a longer build, its program's data extended first, in a
                                // transaction of its own the crank pays, the execute in a slot after it
                                if let Ok(sealed) = crate::upgrade::sealed_of(&act.payload, &self.ids.poleis) {
                                    if let Some(n) = self.extend_first(&sealed, k)? {
                                        sent += 1;
                                        eprintln!("  act {}: {}'s program data extended by {n} bytes for the buffer's build", act.n, sealed.program);
                                    }
                                }
                                let made = act.before.iter().map(|s| s.what.as_str()).collect::<Vec<_>>().join(" and ");
                                let what = if made.is_empty() { format!("act {}: executed ({})", act.n, act.name) } else { format!("act {}: executed ({}), {made} made first in its transaction by the deployer's key", act.n, act.name) };
                                (what, act.execute_ixs(&self.deployer, |n| self.rpc.rent(n), |key| self.account(key))?)
                            }
                        };
                        let budget = crate::chain::budget_of(&self.p, 120_000)?;
                        match self.try_send("crank", &what, [budget, ix].concat(), &[k])? {
                            Ok(_) => sent += 1,
                            Err(words) if matches!(d, Due::Seal(_)) && waits(&words) => {}
                            Err(_) if matches!(&d, Due::Attest(_, i) if i.len() > 1) => eprintln!("  {what}: the batch refused; its ballots go one by one"),
                            Err(words) => {
                                eprintln!("  refused: {words}");
                                refused.push(what);
                            }
                        }
                    }
                    Next::Wait(w) => break (format!("waits: {w}"), seen.executed),
                    Next::Ended(w) => break (w, seen.executed),
                }
            };
            let (seen, _) = self.seen(act)?;
            println!(
                "ACT {} ({}): unit 1 {}, root 0 {} ({} seats of its floor voted by their owners): {word}",
                act.n,
                act.name,
                seen.crew.as_ref().map(|c| state_name(c.state)).unwrap_or("not opened"),
                seen.root.as_ref().map(|c| state_name(c.state)).unwrap_or("not opened"),
                seen.root.as_ref().map_or(0, |c| c.voiced.len()),
            );
            self.rec.v["crank"][act.n.to_string()] = json!({ "unit1": seen.crew.as_ref().map(|c| c.state), "root0": seen.root.as_ref().map(|c| c.state), "executed": executed, "said": word });
            if act.needs == Some(STAKE_POOL) && seen.executed {
                // the pool read back and held as root 0's before the press says it is born
                match self.pool_at()? {
                    PoolAt::Born(read) => {
                        let pool = self.pool_of()?;
                        println!(
                            "THE POOL IS BORN: {} read back: its manager and its staker the polis's hand {}, its fee account root 0's vault of its mint {}, its epoch fee {}, its list {} for {} validators",
                            pool.pool, pool.agent, pool.vault, read["epochFee"].as_str().unwrap_or("?"), pool.list, pool.max_validators
                        );
                        self.rec.v["pool"]["born"] = read;
                    }
                    other => return Err(format!("act {} executed, and the pool is not read back as root 0's ({other:?})", act.n)),
                }
            }
            before = before && executed;
        }
        // THE BRANCH VOTES ITS OWN CODE: each release the ballots name, its agents' envelopes landed in unit 1, then unit 1's seal
        let mut seen_releases: Vec<[u8; 32]> = Vec::new();
        for b in &rel {
            if seen_releases.contains(&b.hash) {
                continue;
            }
            seen_releases.push(b.hash);
            let h = b.hash;
            let short = &wire::hex(&h)[..12];
            let Some(p) = self.account(&client::proposal_pda(&h))?.and_then(|a| client::ProposalView::parse(&a.data)) else {
                println!("RELEASE {short}: not proposed yet: an agent of unit {POSIVERSE} proposes it by its own transaction (press release --commit <c> --agent <it>); its ballots wait");
                continue;
            };
            let commit = { let x = wire::hex(&p.manifest); if x[40..].bytes().all(|b| b == b'0') { x[..40].to_string() } else { x } };   // a SHA-1 commit, zero-padded in its manifest
            let reg = client::RegistryView::parse(&self.account(&client::registry_pda(POSIVERSE))?.ok_or("unit 1's registry is not on the cluster")?.data).ok_or("unit 1's registry does not parse")?;
            let session = |me: &Self| -> Res<Option<client::SessionView>> { Ok(me.account(&client::session_pda(&h, POSIVERSE))?.and_then(|a| client::SessionView::parse(&a.data))) };
            let Some(s0) = session(self)? else {
                println!("RELEASE {short}: unit {POSIVERSE} holds no vote of it: it is no release of the branch that runs the house");
                continue;
            };
            if s0.state == STATE_VOTING {
                let voiced: Vec<Pubkey> = reg.pentagons.iter().enumerate().filter(|(k, c)| c.kind == HOLDER_KEY && s0.chair_voice(*k).is_some()).map(|(_, c)| Pubkey::new_from_array(c.holder)).collect();
                let agents: Vec<Pubkey> = reg.pentagons.iter().filter(|c| c.kind == HOLDER_KEY).map(|c| Pubkey::new_from_array(c.holder)).collect();
                let due: Vec<&ReleaseBallot> = rel.iter().filter(|x| x.hash == h && agents.contains(&x.voter) && !voiced.contains(&x.voter)).collect();
                for x in rel.iter().filter(|x| x.hash == h && !agents.contains(&x.voter)) {
                    eprintln!("  release {short}: {}'s ballot is no voice in unit {POSIVERSE} (no chair of it): it counts for nothing, not sent", x.voter);
                }
                if !due.is_empty() {
                    let signed: Vec<(Pubkey, u8, [u8; 64])> = due.iter().map(|x| (x.voter, x.choice, x.signature)).collect();
                    let what = format!("release {short}: {} ballot{} landed in unit {POSIVERSE}'s chairs", due.len(), if due.len() == 1 { "" } else { "s" });
                    let budget = crate::chain::budget_of(&self.p, 120_000)?;
                    match self.try_send("crank", &what, [budget, client::attest_batch(self.deployer, h, POSIVERSE, &signed).to_vec()].concat(), &[k])? {
                        Ok(_) => sent += 1,
                        Err(words) => {
                            eprintln!("  refused: {words}");
                            refused.push(what);
                        }
                    }
                }
                let what = format!("release {short}: unit {POSIVERSE} sealed");
                let budget = crate::chain::budget_of(&self.p, 120_000)?;
                match self.try_send("crank", &what, [budget, vec![client::seal(self.deployer, h, POSIVERSE)]].concat(), &[k])? {
                    Ok(_) => sent += 1,
                    Err(words) if waits(&words) => {}
                    Err(words) => {
                        eprintln!("  refused: {words}");
                        refused.push(what);
                    }
                }
            }
            let s1 = session(self)?.map(|x| x.state).unwrap_or(s0.state);
            let word = match s1 {
                STATE_PASSED => "sealed RATUM: the lander ships it by itself",
                STATE_VOTING => "waits: its window runs, or its agents have not decided",
                _ => "sealed, not RATUM: nothing ships",
            };
            println!("RELEASE {short} (the house's code at {}): unit {POSIVERSE} {}, no seat holder asked: {word}", &commit[..12], state_name(s1));
            self.rec.v["crank"]["releases"][wire::hex(&h)] = json!({ "commit": commit, "unit1": s1, "said": word });
        }
        self.rec.save()?;
        println!("press crank: {sent} transactions sent{}", if sent == 0 { ", nothing was due" } else { "" });
        if refused.is_empty() {
            Ok(())
        } else {
            Err(format!("press crank sent what was due; {} refused: {}", refused.len(), refused.join("; ")))
        }
    }
}
