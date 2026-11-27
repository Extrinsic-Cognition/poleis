//! THE BRANCH VOTES ITS OWN CODE (Nov 23 2026, his 20:13 UTC "what? do you not understand this" over "Vote the why, bet the where, vote the
//! how", then "Yes, fix it" to "Move code updates down to the agents' fast vote?"): a release of the house's code is unit 1's own decision,
//! the branch that runs the house, its agents proposing and voting it; no seat holder votes code. The release's name is the seal's reader's
//! own (poleis/seal/tests/vector.json: the names Python made and Rust holds, of any polis), `press release` writes the agents' work order
//! (every agent's envelope, the proposer's own `propose`), and `press crank` takes a release's ballots for unit 1 alone.
use poleis_client::{ROOT_UNIT, VOTE_PRO};
use poleis_press::crank::{ballots, release_ballots, release_held};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::seal::{release, release_in, release_set, POLIS, RELEASE_NONCES, SIGNAL_RELEASE};
use poleis_press::wire::{self, Key};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn params(placeholder: bool) -> Params {
    let s = |w: &str| k(w).to_string();
    let v = json!({
        "cluster": "localnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "R", "desc": "", "defaultMint": s("release mint"), "planCap": 1, "planPeriod": 6480000, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("release issuer") },
        "posiverse": { "name": "R", "chair": 2, "agents": [s("release agent 1"), s("release agent 2"), s("release agent 3")], "charter": "33".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": if placeholder { json!("PLACEHOLDER: his") } else { json!(7) } },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "u", "dollar": s("release dollar"), "opening": 0, "miner": s("release miner") },
        "acts": { "nonce": 1 },
        "pool": { "maxValidators": 10, "tag": "lp1" }
    });
    Params::from_text(&v.to_string()).unwrap()
}

#[test]
fn the_releases_name_is_the_seal_readers_own_in_the_branch_that_runs_the_house() {
    let v: Value = serde_json::from_str(include_str!("../../seal/tests/vector.json")).unwrap();
    for n in v["names"].as_array().unwrap() {
        let r = release_in(n["commit"].as_str().unwrap(), n["polis"].as_u64().unwrap(), n["nonce"].as_u64().unwrap()).unwrap();
        assert_eq!(wire::hex(&r.hash), n["proposal"].as_str().unwrap(), "{} in polis {}: the press's name is the reader's", n["commit"], n["polis"]);
    }
    assert_eq!(POLIS, 1, "the house's code is unit 1's, the branch that runs it: never root 0's floor");
    assert_ne!(POLIS, ROOT_UNIT);
    let c = "1ec7da52de041916f032eb8a29886d431e701664";
    let r = release(c, 0).unwrap();
    assert_eq!((r.polis, r.class, r.payload[..2].to_vec()), (1, poleis_treasury::CLASS_SIGNAL | poleis_treasury::CLASS_CONSENT, vec![poleis_treasury::CLASS_SIGNAL, SIGNAL_RELEASE]));
    assert_eq!((&r.manifest[..20], &r.manifest[20..]), (&wire::unhex::<20>(c).unwrap()[..], &[0u8; 12][..]));
    assert_ne!(release(c, 0).unwrap().hash, release(c, 1).unwrap().hash, "each nonce its own proposal");
    assert_ne!(release(c, 0).unwrap().hash, release_in(c, ROOT_UNIT, 0).unwrap().hash, "the branch's release is another proposal than root 0's");
    for bad in ["1ec7da52de04", "zz".repeat(20).as_str(), ""] {
        assert!(release(bad, 0).unwrap_err().contains("a commit git names"), "{bad}");
    }
    assert_eq!(RELEASE_NONCES, 8, "the reader reads eight nonces of a commit");
}

#[test]
fn the_agents_work_order_is_their_envelopes_and_one_agents_proposal() {
    let p = params(false);
    let agents = [k("release agent 1"), k("release agent 2"), k("release agent 3")];
    let r = release("1ec7da52de041916f032eb8a29886d431e701664", 0).unwrap();
    let set = release_set(&p, &r, &agents, Some(&agents[0]), "the chain's genesis", &BTreeMap::new()).unwrap();
    assert_eq!((set["cluster"].as_str(), set["signable"].as_bool(), set["release"]["polis"].as_u64(), set["release"]["hash"].as_str()), (Some("localnet"), Some(true), Some(1), Some(wire::hex(&r.hash).as_str())));
    // every agent's envelope: its PRO of this release in unit 1, a message and never a transaction
    let envelopes = set["envelopes"].as_array().unwrap();
    assert_eq!(envelopes.iter().map(|e| e["signer"].as_str().unwrap().to_string()).collect::<Vec<_>>(), agents.iter().map(|a| a.to_string()).collect::<Vec<_>>());
    for e in envelopes {
        assert_eq!(wire::unb64(e["envelope"].as_str().unwrap()).unwrap(), poleis_client::vote_envelope(&r.hash, 1, VOTE_PRO), "the PRO envelope of the release in unit 1");
        assert_eq!(e["unit"], json!(1));
    }
    // the proposer's own transaction: the client's propose of the release in unit 1, one signer (the agent), the blockhash blank
    let txs = set["transactions"].as_array().unwrap();
    assert_eq!(txs.len(), 1);
    let m = wire::unb64(txs[0]["message"].as_str().unwrap()).unwrap();
    let (signers, keys, at) = wire::layout(&m).unwrap();
    assert_eq!((signers, keys[0]), (1, agents[0]), "one signer, the agent, its fee payer");
    assert!(m[at..at + 32].iter().all(|b| *b == 0), "its blockhash blank");
    let want = poleis_client::propose_raw_of(ROOT_UNIT, agents[0], r.hash, r.class, 1, r.manifest, &r.payload, &[1]);
    assert!(m.windows(want.data.len()).any(|w| w == want.data.as_slice()), "the client's own propose of the release in unit 1, byte for byte");
    assert!(m.windows(32).any(|x| x == r.hash), "its bytes carry the release's hash");
    // envelopes alone when nobody proposes; a proposer that is no agent is refused; a release of another polis is refused; no seat holder anywhere
    let only = release_set(&p, &r, &agents, None, "g", &BTreeMap::new()).unwrap();
    assert!(only.get("transactions").is_none() && only["envelopes"].as_array().unwrap().len() == 3);
    assert!(release_set(&p, &r, &agents, Some(&k("a seat holder")), "g", &BTreeMap::new()).unwrap_err().contains("is no agent of unit 1"));
    assert!(release_set(&p, &release_in("ab".repeat(20).as_str(), ROOT_UNIT, 0).unwrap(), &agents, None, "g", &BTreeMap::new()).unwrap_err().contains("the house's code is unit 1's"));
    let held = release_set(&params(true), &r, &agents, Some(&agents[0]), "g", &BTreeMap::new()).unwrap();
    assert_eq!((held["signable"].as_bool(), held["placeholders"].clone()), (Some(false), json!(["fund.chairQuorum"])));
    assert!(set.get("seats").is_none() && set["what"].as_str().unwrap().contains("no seat holder votes code"), "no seat holder in a release of the house's code");
}

#[test]
fn the_crank_takes_a_releases_ballots_for_unit_1_alone() {
    let h = wire::hex(&release("1ec7da52de041916f032eb8a29886d431e701664", 0).unwrap().hash);
    let sig = "11".repeat(64);
    let text = json!([
        { "release": h, "unit": 1, "voter": k("release agent 1").to_string(), "signature": sig },
        { "release": h, "unit": 1, "voter": k("release agent 2").to_string(), "signature": sig, "choice": "CONTRA" },
    ])
    .to_string();
    let rel = release_ballots(&text).unwrap();
    assert_eq!(rel.iter().map(|b| (b.voter, b.choice)).collect::<Vec<_>>(), vec![(k("release agent 1"), VOTE_PRO), (k("release agent 2"), poleis_client::VOTE_CONTRA)]);
    assert!(ballots(&text, &[]).unwrap().is_empty(), "a release's rows are no act's ballots");
    // root 0's floor never votes code; a second ballot of a voter on the same release; a release that is no hash
    let floor = json!([{ "release": h, "unit": 0, "voter": k("a seat holder").to_string(), "signature": sig }]).to_string();
    assert!(release_ballots(&floor).unwrap_err().contains("no seat holder votes code"));
    let twice = json!([{ "release": h, "unit": 1, "voter": k("a").to_string(), "signature": sig }, { "release": h, "unit": 1, "voter": k("a").to_string(), "signature": sig }]).to_string();
    assert!(release_ballots(&twice).unwrap_err().contains("a second ballot"));
    assert!(release_ballots(&json!([{ "release": "1ec7da52de04", "unit": 1, "voter": k("a").to_string(), "signature": sig }]).to_string()).unwrap_err().contains("64 hex"));
    // THE HOLD (chair.rs's, for a release): a stand-in agent's own signature over the release's envelope in unit 1 is held; another
    // release's replayed, another choice's and a forged one are refused before the crank sends anything
    let agent = Key::from_seed(&[7; 32]);
    let (r0, r1) = (release("1ec7da52de041916f032eb8a29886d431e701664", 0).unwrap(), release("1ec7da52de041916f032eb8a29886d431e701664", 1).unwrap());
    let signed = agent.sign(&poleis_client::vote_envelope(&r0.hash, 1, VOTE_PRO));
    let one = |h: &[u8; 32], choice: &str, s: &[u8; 64]| {
        let row = json!([{ "release": wire::hex(h), "unit": 1, "voter": agent.address.to_string(), "signature": wire::hex(s), "choice": choice }]);
        release_ballots(&row.to_string()).unwrap().remove(0)
    };
    release_held(&one(&r0.hash, "PRO", &signed)).unwrap();
    for (b, why) in [(one(&r1.hash, "PRO", &signed), "another release's replayed"), (one(&r0.hash, "CONTRA", &signed), "another choice's"), (one(&r0.hash, "PRO", &[0x11; 64]), "forged")] {
        assert!(release_held(&b).unwrap_err().contains("not that key's over this release's envelope in unit 1"), "{why}");
    }
}
