// A RELEASE IS A VOTE: the law of the seal, on fixtures and on the vector pull.py is held to (tests/vector.json: the practice chain's own
// transactions of one sealed release, read back at `finalized`).
use poleis_seal::*;
use serde_json::{json, Value};

const PROGRAM: &str = "AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab";

fn vector() -> Value {
    serde_json::from_str(include_str!("vector.json")).unwrap()
}
fn road(v: &Value) -> Road {
    Road::parse(&v["road"]).unwrap()
}

#[test]
fn the_name_is_the_clients_hash_of_the_release() {
    let v = vector();
    for n in v["names"].as_array().unwrap() {
        let c = commit_bytes(n["commit"].as_str().unwrap()).unwrap();
        assert_eq!(hex(&proposal_of(&c, n["polis"].as_u64().unwrap(), n["nonce"].as_u64().unwrap())), n["proposal"].as_str().unwrap());
    }
    assert!(commit_bytes("ABCDEF0123456789abcdef0123456789abcdef01").is_err(), "upper case is no commit git names");
    assert!(commit_bytes("abc").is_err());
    let c = commit_bytes(&"ab".repeat(20)).unwrap();
    assert_eq!(payload_of(&c)[..2], [1, 0x52]);
    assert_eq!(&manifest_of(&c)[..20], &c[..]);
    assert_eq!(manifest_of(&c)[20..], [0u8; 12]);
    assert_ne!(proposal_of(&c, 0, 0), proposal_of(&c, 0, 1));
    assert_ne!(proposal_of(&c, 0, 0), proposal_of(&c, 1, 0), "another polis's release is another proposal");
}

#[test]
fn a_receipt_counts_only_in_the_programs_own_frame() {
    use base64::Engine as _;
    let h = [7u8; 32];
    let mut r = poleis_treasury::RECEIPT_DOMAIN.to_vec();
    r.push(poleis_treasury::RCPT_SEAL);
    r.extend_from_slice(&0u64.to_le_bytes());
    r.extend_from_slice(&h);
    r.extend_from_slice(&[0u8; 72]);
    r.push(1);
    let b = base64::engine::general_purpose::STANDARD.encode(&r);
    let at = |frame: &str| vec![format!("Program {frame} invoke [1]"), format!("Program data: {b}"), format!("Program {frame} success")];
    assert_eq!(seal_outcome(&at(PROGRAM), PROGRAM, &h, 0), Some(1));
    assert_eq!(seal_outcome(&at("Other1111111111111111111111111111111111111"), PROGRAM, &h, 0), None, "another program's data line is not the program's word");
    let nested = vec![format!("Program {PROGRAM} invoke [1]"), "Program Other111 invoke [2]".into(), format!("Program data: {b}"), "Program Other111 success".into(), format!("Program {PROGRAM} success")];
    assert_eq!(seal_outcome(&nested, PROGRAM, &h, 0), None, "a program the seal calls speaks in its own frame");
    assert_eq!(seal_outcome(&[format!("Program log: Program data: {b}")], PROGRAM, &h, 0), None, "sol_log cannot write a data line");
    assert_eq!(seal_outcome(&at(PROGRAM), PROGRAM, &h, 1), None, "another polis's seal");
    assert_eq!(seal_outcome(&at(PROGRAM), PROGRAM, &[8u8; 32], 0), None, "another proposal's seal");
}

#[test]
fn the_accounts_word_names_why_not() {
    let r = Road { rpc: "http://127.0.0.1:1".into(), program: PROGRAM.into(), polis: 0 };
    let c = "ab".repeat(20);
    let w = accounts_word(&c, &r, &vec![(None, None); 8]).unwrap();
    assert_eq!(w, Word::Not(format!("no proposal of polis 0 names {} (nonces 0 to 7)", &c[..12])));
    let mut squat = vec![(None, None); 8];
    squat[0] = (Some(vec![0u8; 40]), None);
    assert!(matches!(accounts_word(&c, &r, &squat).unwrap(), Word::Not(w) if w.contains("holds no proposal")));
}

#[test]
fn the_practice_chains_seal_verifies_and_nothing_else_does() {
    let v = vector();
    let r = road(&v);
    for case in v["seals"].as_array().unwrap() {
        let seal = Seal::parse(&case["seal"]).unwrap();
        let got = verify(&seal, case["commit"].as_str().unwrap(), &r, &case["seal_tx"], &case["propose_tx"]);
        match case["refused"].as_str() {
            None => assert_eq!(got, Ok(()), "{}", case["what"]),
            Some(why) => assert!(matches!(&got, Err(e) if e.contains(why)), "{}: {got:?}", case["what"]),
        }
    }
}

/// ANY MEMBER PROPOSES, THE SWARM DECIDES (Nov 23 2026, his "the swarm votes on updates"): the road names its node, its program and its
/// polis, and nobody who may propose; a road that names proposers, even none, is refused with words that say why.
#[test]
fn the_road_names_no_proposer() {
    let ok = json!({ "rpc": "http://127.0.0.1:1", "program": PROGRAM, "polis": 3 });
    assert_eq!(Road::parse(&ok).unwrap().polis, 3);
    for named in [json!(["k"]), json!([])] {
        assert_eq!(Road::parse(&json!({ "rpc": "x", "program": PROGRAM, "polis": 0, "proposers": named })).unwrap_err(), PROPOSERS_NAMED);
    }
    assert!(PROPOSERS_NAMED.contains("any member of the polis proposes and the swarm decides"));
    assert!(Road::parse(&json!({ "rpc": "x", "program": "Other", "polis": 0 })).unwrap_err().contains("is no address"));
}

/// THE LAUNCH NAMES THE SEAL'S ROAD: the launch's Poleis is the sitting's own id, so the road the press writes names it, and the reader
/// derives a proposal's and a session's addresses under it by the program's own seeds; under the practice chain's program they are the
/// client's own.
#[test]
fn the_road_names_its_program_and_its_addresses_are_that_programs() {
    let h = [7u8; 32];
    let practice: solana_pubkey::Pubkey = PROGRAM.parse().unwrap();
    assert_eq!((proposal_at(&practice, &h), session_at(&practice, &h, 0)), (poleis_client::proposal_pda(&h), poleis_client::session_pda(&h, 0)), "under the client's own program: the client's addresses");
    // a road as the launch press writes it, of a sitting's own Poleis: its three fields, read
    let sitting = solana_pubkey::Pubkey::new_from_array([3; 32]);
    let road = Road::parse(&json!({ "rpc": "http://127.0.0.1:23410", "program": sitting.to_string(), "polis": 0 })).unwrap();
    assert_eq!(road.id().unwrap(), sitting);
    assert_eq!(proposal_at(&sitting, &h), solana_pubkey::Pubkey::find_program_address(&[b"proposal", &h], &sitting).0);
    assert_ne!(proposal_at(&sitting, &h), poleis_client::proposal_pda(&h), "another program's proposal is at another address");
    assert_ne!(session_at(&sitting, &h, 0), session_at(&sitting, &h, 1), "each unit's session its own");
}
