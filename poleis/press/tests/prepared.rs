//! THE FLOOR'S VOTES ARE THE PAGE'S PREPARED SETS: what `press vote` writes holds the Keystone page's own rules for a prepared set
//! (poleis/ceremony/keystone/prepared.mjs `readSet`): each transaction's message with a blank blockhash, its sha256 stated, one signer, its
//! fee payer; a set with a placeholder says it may not be signed. THE WHOLE TREE DECIDES: a vote set is one seat owner's own transactions
//! of one act (its proposal, `propose_nft`, when it proposes; its PRO, `attest_nft`), and the acts' sets (`press acts`) hold the eight acts
//! in order (the fifth, the pool's birth, where the stake pool program stands, as on mainnet; the sixth, the mine's miner; the seventh and the eighth, the treasury's dollar cap and the router's month), each its payload and hash and unit 1's
//! agents' envelopes, and no transaction of anyone's: no key of his is in any of them. The vector beside this file (prepared-vector.json)
//! is one vote set, written by this test from fixed params and the tree's own ids, with what the page's own functions make of it (readSet,
//! toSign, signedWire), made on the new Mac by:
//!
//!   PRESS_WRITE_VECTOR=1 cargo test --offline --test prepared, then, at the house's root,
//!   node --input-type=module -e "import {readSet,toSign,signedWire} from './poleis/ceremony/keystone/prepared.mjs'; import {readFileSync,writeFileSync} from 'node:fs'; const p='poleis/press/tests/prepared-vector.json'; const v=JSON.parse(readFileSync(p,'utf8')); const s=await readSet(v.set); v.readSet={cluster:s.cluster,genesis:s.genesis,act:s.act,signable:s.signable,placeholders:s.placeholders,signers:s.txs.map(t=>t.signer),shown:s.txs.map(t=>t.shown.map(x=>({program:x.program,data:x.data})))}; v.toSign=s.txs.map(t=>Buffer.from(toSign(s,t,t.signer,v.blockhash)).toString('base64')); v.signedWire=s.txs.map(t=>Buffer.from(signedWire(toSign(s,t,t.signer,v.blockhash),new Uint8Array(64).fill(7))).toString('base64')); writeFileSync(p,JSON.stringify(v,null,1)+'\n');"
//!
//! The Rust side of the page's road (poleis_treasury/tests/council.rs, `the_prepared_road_is_the_keystone_pages_own`) is held to those
//! answers; no JavaScript runs in a gate.
use poleis_client::{nft, ROOT_UNIT};
use poleis_press::floor::{vote_set, Seat};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::{acts, build, chain, wire};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Fixed params of a mainnet shape: every value given, the addresses of keys nobody holds (sha256 of words).
fn params(placeholder: bool) -> Params {
    let v = json!({
        "cluster": "mainnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "V", "desc": "", "defaultMint": k("vector mint").to_string(), "planCap": 1, "planPeriod": 6480000, "dollarCap": 2, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": k("vector issuer").to_string() },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": if placeholder { json!("PLACEHOLDER: his") } else { json!(7) } },
        "posiverse": { "name": "V", "chair": 2, "agents": [k("vector agent 1").to_string(), k("vector agent 2").to_string(), k("vector agent 3").to_string()], "charter": "22".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "https://optimism.fit/app/warm-token.json", "dollar": k("vector dollar").to_string(), "opening": 0, "miner": k("vector miner").to_string() },
        "router": { "monthCap": 1 },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1000000000u64, "poolQuote": 1000000000u64, "judgeSlots": 5000,
                    "crediter": k("vector crediter").to_string(), "split": ["treasury"], "plan": ["treasury", "act"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "pool": { "maxValidators": 10, "tag": "lp1", "programSha256": "ab".repeat(32) }
    });
    Params::from_text(&v.to_string()).unwrap()
}

fn written(p: &Params) -> (tempdir::Dir, Vec<Value>) {
    let (ids, ..) = build::compiled().unwrap();
    let at = chain::addresses(&ids, 77);
    let dir = tempdir::Dir::new();
    // on mainnet the stake pool program stands, so the fifth act, the pool's birth, is written
    let files = acts::write(&dir.0, p, &ids, &at, poleis_press::rpc::MAINNET_GENESIS, &k("vector deployer"), true).unwrap();
    let sets = files.iter().filter(|f| !f.ends_with("index.json")).map(|f| serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap()).collect();
    (dir, sets)
}

/// One seat owner of root 0's floor, by words, and its one seat (its token the first buyer's own).
fn owner() -> (solana_pubkey::Pubkey, Seat) {
    let owner = k("vector seat owner");
    let mint = nft::mint_pda(ROOT_UNIT, &[0x5e; 32]);
    (owner, Seat { seat: nft::seat_pda(&mint), proof: nft::OwnerProof { owner, mint, token: nft::initial_token_pda(&mint, &owner), unit: ROOT_UNIT } })
}

/// The seat owner's vote set of act 1 with its proposal: the vector's.
fn the_vote(p: &Params) -> Value {
    let (ids, ..) = build::compiled().unwrap();
    let act = acts::acts(p, &ids, &chain::addresses(&ids, 77), &k("vector deployer")).unwrap().remove(0);
    let (o, seat) = owner();
    vote_set(p, &act, &o, &[seat], true, poleis_press::rpc::MAINNET_GENESIS, &BTreeMap::new()).unwrap()
}

mod tempdir {
    pub struct Dir(pub std::path::PathBuf);
    impl Dir {
        pub fn new() -> Self {
            // a folder a test: its threads share the process and may share the clock's nanosecond, never this count
            static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let p = std::env::temp_dir().join(format!("press-acts-{}-{n}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn the_acts_sets_hold_no_transaction_and_no_key_of_his() {
    let (_dir, sets) = written(&params(false));
    assert_eq!(sets.len(), 8, "the fund, the board's law, its roles, the market, the pool's birth, the $WARM mine's miner, the treasury's dollar cap, the router's month");
    let p = params(false);
    for (i, set) in sets.iter().enumerate() {
        assert_eq!(set["act"]["n"], json!(i + 1), "the acts in their order");
        assert_eq!((set["cluster"].as_str(), set["signable"].as_bool()), (Some("mainnet"), Some(true)));
        assert!(set.get("transactions").is_none() && set.get("signer").is_none(), "no transaction of anyone's, no signer named: the proposal and the floor's votes are the seat owners' (press vote)");
        assert!(set["proposal"].as_str().unwrap().contains("press vote") && set["floor"].as_str().unwrap().contains("press vote"));
        // a CODEX act closes over his posiverse; the others convene it
        let codex = set["act"]["class"] == json!(poleis_client::CLASS_CODEX);
        assert_eq!((set["act"]["impacted"].clone(), set["act"]["convened"].clone()), if codex { (json!([0, 1]), json!(false)) } else { (json!([0]), json!(true)) });
        let hash: [u8; 32] = wire::unhex(set["act"]["hash"].as_str().unwrap()).unwrap();
        let envelopes = set["envelopes"].as_array().unwrap();
        let want: Vec<(String, u64)> = p.agents().unwrap().iter().map(|a| (a.to_string(), 1)).collect();
        assert_eq!(envelopes.iter().map(|e| (e["signer"].as_str().unwrap().to_string(), e["unit"].as_u64().unwrap())).collect::<Vec<_>>(), want, "each agent's in unit 1, and nobody's on root 0's floor");
        for e in envelopes {
            let env = wire::unb64(e["envelope"].as_str().unwrap()).unwrap();
            assert_eq!(env, poleis_client::vote_envelope(&hash, 1, poleis_client::VOTE_PRO), "the PRO envelope of this act in unit 1, a message");
        }
        assert!(!set.to_string().to_lowercase().contains("keystone"), "act {}: no word of a Keystone in its set", i + 1);
    }
}

#[test]
fn a_seat_owners_vote_is_what_the_page_signs() {
    let set = the_vote(&params(false));
    assert_eq!((set["cluster"].as_str(), set["signable"].as_bool()), (Some("mainnet"), Some(true)));
    let (o, _) = owner();
    let txs = set["transactions"].as_array().unwrap();
    assert_eq!(txs.iter().map(|t| t["n"].as_u64().unwrap()).collect::<Vec<_>>(), vec![1, 2], "its proposal, then its PRO");
    for t in txs {
        let m = wire::unb64(t["message"].as_str().unwrap()).unwrap();
        let (signers, keys, at) = wire::layout(&m).unwrap();
        assert!(m[at..at + 32].iter().all(|b| *b == 0), "its blockhash blank");
        assert_eq!(t["messageSha256"].as_str().unwrap(), wire::hex(&wire::sha256(&m)), "its message the one it describes");
        assert_eq!((signers, keys[0].to_string(), t["signers"][0].as_str().unwrap().to_string()), (1, o.to_string(), o.to_string()), "one signer, its fee payer, the owner");
        assert!(t["bytes"].as_u64().unwrap() as usize <= wire::PACKET);
    }
}

#[test]
fn a_set_with_a_placeholder_says_it_may_not_be_signed() {
    let (_dir, sets) = written(&params(true));
    for set in sets.iter().chain([&the_vote(&params(true))]) {
        assert_eq!(set["signable"], json!(false));
        assert_eq!(set["placeholders"], json!(["fund.chairQuorum"]));
    }
}

#[test]
fn the_vector_is_the_press_s_own_set() {
    let set = the_vote(&params(false));
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/prepared-vector.json");
    if std::env::var_os("PRESS_WRITE_VECTOR").is_some() {
        let mut set = set;
        set["emitted"] = json!("fixed");
        let blockhash = wire::base58(&(1..=32u8).collect::<Vec<_>>());
        std::fs::write(path, serde_json::to_string_pretty(&json!({ "what": "one seat owner's vote set of the press's first act (its proposal and its PRO), and what the Keystone page's own functions make of it (tests/prepared.rs says how it was made)", "set": set, "blockhash": blockhash })).unwrap() + "\n").unwrap();
        return;
    }
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for k in ["transactions", "act", "cluster", "signable", "placeholders", "signer", "seats"] {
        assert_eq!(v["set"][k], set[k], "the vector's {k} is what the press writes today");
    }
    assert!(v["toSign"].as_array().is_some_and(|a| a.len() == 2), "the page's own answers stand in the vector");
    // A SEAT OWNER'S VOTE ON A LOCAL CHAIN: what the page says the Keystone shows of the PRO (the program's address, the data in base58)
    // is the client's own attest of this act, and the act the page names is the press's
    let (_, seat) = owner();
    let pro = seat.proof.attest(wire::unhex(set["act"]["hash"].as_str().unwrap()).unwrap(), poleis_client::VOTE_PRO).unwrap();
    assert_eq!(v["readSet"]["shown"][1], json!([{ "program": pro.program_id.to_string(), "data": wire::base58(&pro.data) }]), "the page shows the PRO's own bytes");
    assert_eq!((&v["readSet"]["act"]["hash"], &v["readSet"]["act"]["n"]), (&set["act"]["hash"], &set["act"]["n"]), "the page names the press's act");
}
