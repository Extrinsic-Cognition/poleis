//! THE SWARM NAMES ITS MINER (Nov 23 2026, lane/the-swarm-names-its-miner; his 18:22:41 UTC "Yes, start it" to "Start the sixth act, the swarm
//! naming $WARM's miner?"). Until an act of the polis names the $WARM mine's miner, the mine mines nothing: only its authority, the polis's agent
//! PDA, may call `set_miner`. The press's sixth act is that call, an ACT_RAW of the polis's hand decided by the whole tree like the others: its
//! payload the mine's own instruction naming the params' `warm.miner` (the door's crank key `press miner` makes), its execute the program's accounts
//! as ix.rs names them, its set saying who it names and where the same key stands on the door, and a ballot of it read by its number and its hash.
//! The live half (the act passed by the tree and executed by press crank, then a deposit and a spend through the door's code on the press's own
//! world) is the launch rehearsal's: poleis/ceremony/the-launch-press-rehearsal.sh.
use poleis_client::{self as client, Payload, ACT_SIGNER, ACT_WRITABLE, CLASS_ACT, ROOT_UNIT};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::{acts, build, chain, crank, wire};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;

fn params(miner: Value) -> Params {
    let s = |w: &str| k(w).to_string();
    let v = json!({
        "cluster": "mainnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "M", "desc": "", "defaultMint": s("miner mint"), "planCap": 1, "planPeriod": 6480000, "dollarCap": 2, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("miner issuer") },
        "posiverse": { "name": "M", "chair": 2, "agents": [s("miner agent 1"), s("miner agent 2"), s("miner agent 3")], "charter": "33".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "u", "dollar": s("miner dollar"), "opening": 0, "miner": miner },
        "router": { "monthCap": 1 },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1, "poolQuote": 1, "judgeSlots": 5000, "crediter": s("c"), "split": ["treasury"], "plan": ["treasury"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "seal": { "rpc": "https://api.mainnet-beta.solana.com", "proposers": [] },
        "pool": { "maxValidators": 10, "tag": "lp1", "programSha256": "ab".repeat(32) }
    });
    Params::from_text(&v.to_string()).unwrap()
}

fn the_miner() -> Pubkey {
    k("the door's crank key")
}

#[test]
fn the_sixth_act_is_the_mines_own_set_miner_by_the_polis_hand() {
    let p = params(json!(the_miner().to_string()));
    let (ids, ..) = build::compiled().unwrap();
    let at = chain::addresses(&ids, 77);
    let all = acts::acts(&p, &ids, &at, &k("miner deployer")).unwrap();
    assert_eq!(all.iter().map(|a| a.n).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6, 7, 8], "eight acts, in order (THE TREASURY PAYS THE COMPUTE: the dollar cap and the router's month after the miner)");
    let six = &all[5];
    assert_eq!((six.n, six.file, six.class, six.impacted.clone(), six.convene(), six.nonce, six.needs, six.before.len()), (acts::MINER_ACT, "6-the-miner.json", CLASS_ACT, vec![ROOT_UNIT], true, 6, None, 0), "an ACT_RAW of root 0, unit 1 convened by the crank, the sixth nonce, nothing made first");
    // its payload: the $WARM program's own set_miner naming the params' miner, the polis's hand its authority, the sitting's mine
    let want = Payload::Raw { origin: ROOT_UNIT, program: ids.warm, accounts: vec![(at.agent, ACT_SIGNER), (at.mine, ACT_WRITABLE)], data: [&[warm_mine::IX_SET_MINER][..], the_miner().as_ref()].concat() };
    assert_eq!((six.payload.clone(), six.payload_sha256), (want.bytes(), want.hash()));
    assert_eq!(six.hash, client::proposal_hash(CLASS_ACT, ROOT_UNIT, &six.manifest, &want.hash(), 6), "the proposal's hash covers the miner it names");
    assert_eq!(acts::set_miner_data(&the_miner()), [vec![2u8], the_miner().to_bytes().to_vec()].concat());
    assert!(include_str!("../../program/warm_mine/src/ix.rs").contains("/// data: [miner:32] · accounts: [authority (s), mine (w)]"), "the program's own words of the instruction");
    assert_eq!((at.mine, at.agent), (Pubkey::find_program_address(&[warm_mine::state::SEED_MINE], &ids.warm).0, client::agent_pda(ROOT_UNIT)), "the sitting's mine, and root 0's own hand: the authority press warm opened it with");
    // its execute: Poleis's execute of the proposal, carried to the $WARM program with the polis's hand (Poleis signs for it, never the outer
    // transaction) and the mine, writable
    let e = six.execute_by(&k("a crank"));
    let n = e.accounts.len();
    assert_eq!((e.program_id, e.accounts[0].pubkey, e.accounts[0].is_signer), (client::PROGRAM_ID, k("a crank"), true));
    assert_eq!(e.accounts[n - 3..].iter().map(|m| (m.pubkey, m.is_signer, m.is_writable)).collect::<Vec<_>>(), vec![(ids.warm, false, false), (at.agent, false, false), (at.mine, false, true)]);
    let (execute, proposed) = (acts::execute_bytes(&p, six, &k("a crank"), false).unwrap(), poleis_press::floor::proposal_bytes(&p, six, &poleis_press::floor::placeholder_owner()).unwrap());
    assert!(execute <= wire::PACKET && proposed <= wire::PACKET, "its execute and a seat owner's proposal of it: one transaction each");
    eprintln!("  act 6: its payload {} bytes, its execute's transaction {execute} bytes, a seat owner's proposal of it {proposed} bytes (a packet holds {})", six.payload.len(), wire::PACKET);
    // another miner is another act: its hash moves
    let other = acts::acts(&params(json!(k("another key").to_string())), &ids, &at, &k("miner deployer")).unwrap();
    assert_ne!(other[5].hash, six.hash);
    assert_eq!(other[..5].iter().map(|a| a.hash).collect::<Vec<_>>(), all[..5].iter().map(|a| a.hash).collect::<Vec<_>>(), "and the five before it do not");
}

mod tempdir {
    pub struct Dir(pub std::path::PathBuf);
    impl Dir {
        pub fn new(what: &str) -> Self {
            static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let p = std::env::temp_dir().join(format!("press-miner-{what}-{}-{n}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
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

fn the_set(p: &Params) -> (Value, Value) {
    let (ids, ..) = build::compiled().unwrap();
    let d = tempdir::Dir::new("set");
    // where no stake pool program stands the pool's set is not written; the miner's is, everywhere
    acts::write(&d.0, p, &ids, &chain::addresses(&ids, 77), poleis_press::rpc::MAINNET_GENESIS, &k("miner deployer"), false).unwrap();
    let read = |f: &str| serde_json::from_str::<Value>(&std::fs::read_to_string(d.0.join(f)).unwrap()).unwrap();
    (read("6-the-miner.json"), read("index.json"))
}

#[test]
fn its_set_names_the_miner_and_where_the_door_holds_it() {
    let (ids, ..) = build::compiled().unwrap();
    let at = chain::addresses(&ids, 77);
    let (set, index) = the_set(&params(json!(the_miner().to_string())));
    assert_eq!(index["acts"].as_array().unwrap().iter().map(|a| a["n"].as_u64().unwrap()).collect::<Vec<_>>(), vec![1, 2, 3, 4, 6, 7, 8], "the pool's birth not written where its program does not stand, the miner's written (and the two after it)");
    assert_eq!((set["act"]["n"].clone(), set["signable"].clone(), set["act"]["convened"].clone()), (json!(6), json!(true), json!(true)));
    assert_eq!((set["miner"]["address"].as_str(), set["miner"]["mine"].as_str(), set["miner"]["authority"].as_str()), (Some(the_miner().to_string().as_str()), Some(at.mine.to_string().as_str()), Some(at.agent.to_string().as_str())));
    let does = set["does"].as_array().unwrap().iter().map(|d| d.as_str().unwrap().to_string()).collect::<Vec<_>>().join(" ");
    assert!(does.contains(&the_miner().to_string()) && does.contains("SPACE_ROUTER_MINER_KEY") && does.contains("mines nothing"), "{does}");
    assert!(set.get("transactions").is_none(), "no transaction of anyone's: its proposal and the floor's votes are the seat owners' own (press vote)");
    assert_eq!(set["envelopes"].as_array().unwrap().len(), 3, "an envelope for each of unit 1's agents");
    // the miner not named yet: the set is for reading, and the address it names is one nobody holds a key for
    let words = "PLACEHOLDER: the mint door's crank key";
    let (set, _) = the_set(&params(json!(words)));
    assert_eq!((set["signable"].clone(), set["placeholders"].clone()), (json!(false), json!(["warm.miner"])));
    assert_eq!(set["miner"]["address"].as_str(), Some(k(words).to_string().as_str()));
}

#[test]
fn a_ballot_of_the_sixth_act_is_read_by_its_number_and_its_hash() {
    let p = params(json!(the_miner().to_string()));
    let (ids, ..) = build::compiled().unwrap();
    let all = acts::acts(&p, &ids, &chain::addresses(&ids, 77), &k("miner deployer")).unwrap();
    let row = |act: Value| json!([{ "act": act, "unit": 1, "voter": k("miner agent 1").to_string(), "signature": wire::base58(&[7; 64]) }]).to_string();
    for act in [json!(6), json!(wire::hex(&all[5].hash))] {
        let b = crank::ballots(&row(act.clone()), &all).unwrap_or_else(|e| panic!("{act}: {e}"));
        assert_eq!((b[0].act, b[0].unit), (6, 1), "{act}");
    }
    assert!(crank::ballots(&row(json!(9)), &all).unwrap_err().contains("its act is a number of the acts"), "there is no ninth");
}
