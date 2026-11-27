//! THE CRANK SENDS WHAT IS DUE, IN POLEIS'S ORDER, AND NOTHING ELSE. `crank::next` is the planner `press crank` runs on what it reads off
//! the chain: the leaves speak first (unit 1 convened when its proposal did not open its vote, its agents' ballots, its seal, its chair heard
//! in root 0 by `mandate`), then root 0's seal, once the floor's own votes (each seat owner's transaction, never the crank's) or the window's
//! end let the chain seal it, then the execute, an act only after the acts before it. Here it is walked over an act's whole life, and the
//! faults the crank must never commit are put in one at a time: a stranger's ballot offered as a chair of unit 1, a ballot for root 0's
//! floor (refused as the file is read: the floor votes by its owners' own transactions), a ballot landed twice, a root sealed before its
//! crew spoke, an act executed out of order or after a split. And the ballots file: what it names is held to the acts.
use poleis_client::{ROOT_UNIT, VOTE_CONTRA, VOTE_PRO, STATE_DEADLOCK, STATE_PASSED, STATE_REJECTED, STATE_VOTING};
use poleis_press::crank::{ballots, next, waits, Ballot, Chamber, Due, Electorate, Next, Seen, FLOOR_BALLOT};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::{acts, build, chain, wire};
use serde_json::json;
use solana_pubkey::Pubkey;
use std::collections::BTreeSet;

fn the_acts() -> Vec<acts::Act> {
    let s = |w: &str| k(w).to_string();
    let v = json!({
        "cluster": "localnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "V", "desc": "", "defaultMint": s("crank mint"), "planCap": 1, "planPeriod": 6480000, "dollarCap": 2, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("crank issuer") },
        "posiverse": { "name": "C", "chair": 2, "agents": [s("crank agent 1"), s("crank agent 2"), s("crank agent 3")], "charter": "33".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "u", "dollar": s("crank dollar"), "opening": 0, "miner": s("crank miner") },
        "router": { "monthCap": 1 },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1, "poolQuote": 1, "judgeSlots": 5000, "crediter": s("c"), "split": ["treasury"], "plan": ["treasury"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "pool": { "maxValidators": 10, "tag": "lp1" }
    });
    let p = Params::from_text(&v.to_string()).unwrap();
    let (ids, ..) = build::compiled().unwrap();
    acts::acts(&p, &ids, &chain::addresses(&ids, 77), &k("crank deployer")).unwrap()
}

fn agent(i: u8) -> Pubkey {
    k(&format!("crank agent {i}"))
}
/// A seat of root 0's floor as its session's roster names it (the seat's receipt): heard only by its owner's own transaction.
fn seat(i: u8) -> Pubkey {
    k(&format!("crank seat {i}"))
}
fn who() -> Electorate {
    Electorate { agents: (1..=3).map(agent).collect() }
}
fn ballot(act: usize, unit: u64, voter: Pubkey) -> Ballot {
    Ballot { act, unit, voter, choice: VOTE_PRO, signature: [7; 64] }
}
fn chamber(state: u8, voiced: &[Pubkey]) -> Option<Chamber> {
    Some(Chamber { state, voiced: voiced.to_vec() })
}

/// The crank's walk of one act, as `press crank` loops: each step sent is taken to land as Poleis would land it.
#[test]
fn an_act_is_carried_leaves_first_to_its_execute() {
    let acts = the_acts();
    // act 2, the board's law: an ACT_RAW, its impacted set root 0 alone, so unit 1 is convened by the crank
    let act = &acts[1];
    assert!(act.convene());
    let b = vec![ballot(2, 1, agent(1)), ballot(2, 1, agent(2))];
    let none = BTreeSet::new();
    let mut seen = Seen { proposed: false, ..Seen::default() };
    assert!(matches!(next(act, &seen, &b, &who(), &none, true), Next::Wait(w) if w.contains("a seat owner of root 0's floor proposes it by its own transaction")));
    // the floor's own votes land as its owners send them, before or after anything the crank does: the crank never sends one
    seen = Seen { proposed: true, root: chamber(STATE_VOTING, &[seat(1), seat(2), seat(3), seat(4)]), ..Seen::default() };
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Send(Due::Convene));
    seen.crew = chamber(STATE_VOTING, &[]);
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Send(Due::Attest(1, vec![0, 1])), "the agents' ballots, one batch");
    seen.crew = chamber(STATE_VOTING, &[agent(1), agent(2)]);
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Send(Due::Seal(1)), "unit 1 seals first: no ballot of the floor is ever the crank's to land");
    seen.crew = chamber(STATE_PASSED, &[agent(1), agent(2)]);
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Send(Due::Mandate), "its verdict becomes its chair's vote before root 0 seals");
    seen.heard = true;
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Send(Due::Seal(0)));
    // root 0 refused to seal (its floor has not decided, its window runs): the crank waits for the floor's own votes, and says so
    let tried: BTreeSet<String> = [Due::Seal(0).key()].into();
    assert!(matches!(next(act, &seen, &b, &who(), &tried, true), Next::Wait(w) if w.contains("waits for its floor") && w.contains("press vote")));
    seen.root = chamber(STATE_PASSED, &[seat(1), seat(2), seat(3), seat(4)]);
    assert!(matches!(next(act, &seen, &b, &who(), &none, false), Next::Wait(w) if w.contains("once the acts before it")), "never out of order");
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Send(Due::Execute));
    seen.executed = true;
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Ended("executed".into()), "run again, it sends nothing");
}

#[test]
fn a_codex_act_opens_unit_1_with_its_proposal_and_is_never_convened() {
    let acts = the_acts();
    let fund = &acts[0];
    assert_eq!((fund.impacted.clone(), fund.convene()), (vec![ROOT_UNIT, 1], false), "CODEX closes over the bound child");
    let seen = Seen { proposed: true, root: chamber(STATE_VOTING, &[]), ..Seen::default() };
    assert!(matches!(next(fund, &seen, &[], &who(), &BTreeSet::new(), true), Next::Wait(_)), "unit 1's session missing for a CODEX act: nothing to convene");
}

#[test]
fn the_crank_never_counts_a_voice_out_of_its_chamber_nor_seals_over_a_crew() {
    let acts = the_acts();
    let act = &acts[1];
    let none = BTreeSet::new();
    // a seat owner's ballot offered as a chair of unit 1, a stranger's: none is sent (a ballot for root 0's floor is refused as the file is
    // read: tests the ballots file below)
    let stray = vec![ballot(2, 1, k("a seat owner")), ballot(2, 1, k("a stranger"))];
    let seen = Seen { proposed: true, root: chamber(STATE_VOTING, &[]), crew: chamber(STATE_VOTING, &[]), ..Seen::default() };
    assert_eq!(next(act, &seen, &stray, &who(), &none, true), Next::Send(Due::Seal(1)), "no stray ballot is due: the crank goes straight to unit 1's seal");
    // a voice the session holds is not landed again
    let b = vec![ballot(2, 1, agent(1))];
    let seen = Seen { proposed: true, root: chamber(STATE_VOTING, &[]), crew: chamber(STATE_VOTING, &[agent(1)]), ..Seen::default() };
    assert_eq!(next(act, &seen, &b, &who(), &none, true), Next::Send(Due::Seal(1)));
    // unit 1 refused to seal (its window runs): the crank waits and never seals root 0 over its head
    let tried: BTreeSet<String> = [Due::Seal(1).key()].into();
    assert!(matches!(next(act, &seen, &b, &who(), &tried, true), Next::Wait(w) if w.contains("unit 1 has not decided")));
    // a refused batch goes one ballot at a time
    let b = vec![ballot(2, 1, agent(1)), ballot(2, 1, agent(2))];
    let seen = Seen { proposed: true, root: chamber(STATE_VOTING, &[]), crew: chamber(STATE_VOTING, &[]), ..Seen::default() };
    let tried: BTreeSet<String> = [Due::Attest(1, vec![0, 1]).key()].into();
    assert_eq!(next(act, &seen, &b, &who(), &tried, true), Next::Send(Due::Attest(1, vec![0])));
    // a split or a refusal ends the act: nothing executes
    for state in [STATE_DEADLOCK, STATE_REJECTED] {
        let seen = Seen { proposed: true, heard: true, root: chamber(state, &[]), crew: chamber(STATE_PASSED, &[]), ..Seen::default() };
        assert!(matches!(next(act, &seen, &[], &who(), &none, true), Next::Ended(w) if w.contains("nothing executes")));
    }
}

#[test]
fn the_ballots_file_is_held_to_the_acts() {
    let acts = the_acts();
    let sig = wire::base58(&[5u8; 64]);
    let good = json!([{ "act": 1, "unit": 1, "voter": agent(1).to_string(), "signature": sig }, { "act": wire::hex(&acts[3].hash), "unit": 1, "voter": agent(2).to_string(), "signature": "05".repeat(64), "choice": "CONTRA" }]);
    let b = ballots(&good.to_string(), &acts).unwrap();
    assert_eq!((b[0].act, b[0].unit, b[0].choice, b[0].signature), (1, 1, VOTE_PRO, [5; 64]));
    assert_eq!((b[1].act, b[1].unit, b[1].choice, b[1].signature), (4, 1, VOTE_CONTRA, [5; 64]), "an act by its hash, a signature in hex, a CONTRA");
    // THE WHOLE TREE DECIDES: a ballot for root 0's floor is no envelope the crank lands, whoever signed it
    let floor = json!([{ "act": 1, "unit": 0, "voter": seat(1).to_string(), "signature": sig }]);
    assert!(ballots(&floor.to_string(), &acts).unwrap_err().contains(FLOOR_BALLOT));
    for (bad, says) in [
        // THE TREASURY PAYS THE COMPUTE: the seventh act is the dollar cap and the eighth the router's month, so the act past the acts is the ninth
        (json!([{ "act": 9, "unit": 1, "voter": agent(1).to_string(), "signature": sig }]), "its act"),
        (json!([{ "act": 1, "unit": 2, "voter": agent(1).to_string(), "signature": sig }]), "unit 2"),
        (json!([{ "act": 1, "unit": 1, "voter": "nobody", "signature": sig }]), "not an address"),
        (json!([{ "act": 1, "unit": 1, "voter": agent(1).to_string(), "signature": "abc" }]), "64 bytes"),
        (json!([{ "act": 1, "unit": 1, "voter": agent(1).to_string(), "signature": sig, "choice": "MAYBE" }]), "PRO, CONTRA or ABSTAIN"),
        (json!([{ "act": 1, "unit": 1, "voter": agent(1).to_string(), "signature": sig }, { "act": 1, "unit": 1, "voter": agent(1).to_string(), "signature": sig }]), "a second ballot"),
    ] {
        assert!(ballots(&bad.to_string(), &acts).unwrap_err().contains(says), "{says}");
    }
}

#[test]
fn a_seal_whose_window_runs_waits_and_any_other_refusal_is_said() {
    let code = |e| poleis_client::code(e);
    assert!(waits(&format!("act 1: unit 1 sealed: refused in simulation: {{\"InstructionError\":[0,{{\"Custom\":{}}}]}}", code(poleis_client::PoleisError::SessionOpen))));
    assert!(waits(&format!("{{\"InstructionError\":[0,{{\"Custom\":{}}}]}}", code(poleis_client::PoleisError::ChildMustSpeak))));
    assert!(!waits(&format!("{{\"InstructionError\":[0,{{\"Custom\":{}}}]}}", code(poleis_client::PoleisError::NotPassed))));
}
