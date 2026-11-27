//! THE PARAMS ARE HELD BEFORE ANYTHING STARTS: a value Poleis would refuse at the founding, an address that is none, a face past
//! Metaplex's bounds or a value still waiting on his word stops the sitting before a byte is deployed (`chain::params_checked`, the first
//! thing `press deploy` does). A founding the program refused the slot Poleis became callable would leave root 0 open to the first wallet
//! while the params were mended; these are the faults, put in one at a time, and each is refused by its name. THE WHOLE TREE DECIDES: a
//! params file that names his Keystones, or the first sale's terms, is refused before anything is read.
use poleis_press::params::Params;
use poleis_press::{build, chain};
use serde_json::{json, Value};

/// Params of a mainnet shape, every value given, the addresses of keys nobody holds (sha256 of words).
fn good() -> Value {
    let k = |w: &str| poleis_press::params::placeholder_address(w).to_string();
    json!({
        "cluster": "mainnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "V", "desc": "", "defaultMint": k("params dollar"), "planCap": 1, "planPeriod": 6480000, "dollarCap": 60000000, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": k("params issuer") },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
                "posiverse": { "name": "P", "desc": "", "chair": 2, "agents": [k("params agent 1"), k("params agent 2"), k("params agent 3")], "charter": "11".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "https://optimism.fit/app/warm-token.json", "dollar": k("params dollar"), "opening": "the founding", "miner": k("params miner") },
        "router": { "monthCap": 40000000 },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1000000000u64, "poolQuote": 1000000000u64, "judgeSlots": 1296001u64,
                    "crediter": k("params crediter"), "split": ["treasury"], "plan": ["treasury", "act"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "seal": { "rpc": "https://api.mainnet-beta.solana.com" },
        "pool": { "maxValidators": 10, "tag": "lp1", "programSha256": "ab".repeat(32) }
    })
}

fn checked(v: &Value) -> Result<(), String> {
    let p = Params::from_text(&v.to_string()).unwrap();
    let (ids, ..) = build::compiled().unwrap();
    chain::params_checked(&p, &ids, &chain::addresses(&ids, 77))
}

fn with(path: &str, value: Value) -> Value {
    let mut v = good();
    let (parents, leaf) = path.rsplit_once('.').unwrap();
    let mut at = &mut v;
    for part in parents.split('.') {
        at = at.get_mut(part).unwrap_or_else(|| panic!("the params have no {path}"));
    }
    // a leaf the good params leave out (genesis.bid) is put in
    at[leaf] = value;
    v
}

#[test]
fn params_whole_and_within_the_programs_bounds_pass() {
    checked(&good()).unwrap();
    // the bounds themselves pass: Poleis's shortest and longest window, the ladder's first and last rung, the curve's earliest opening and
    // its shortest and longest window, its cheapest step
    for (path, edge) in [
        ("genesis.window", json!(100)),
        ("genesis.window", json!(1_296_000u64)),
        ("genesis.rung", json!(5)),
        ("floor.opensAfterSlots", json!(151)),
        ("floor.windowSlots", json!(1)),
        ("floor.windowSlots", json!(1_296_000u64)),
        ("floor.stepLamports", json!(1)),
        ("warm.opening", json!("2026-11-21T17:34:37Z")),
    ] {
        checked(&with(path, edge.clone())).unwrap_or_else(|e| panic!("{path} = {edge}: {e}"));
    }
}

#[test]
fn a_value_the_sitting_cannot_send_stops_it_before_anything_starts() {
    let faults: Vec<(&str, Value, &str)> = vec![
        ("genesis.window", json!(99), "genesis.window"),
        ("genesis.window", json!(1_296_001u64), "genesis.window"),
        ("genesis.rung", json!(0), "genesis.rung"),
        ("genesis.rung", json!(6), "genesis.rung"),
        ("floor.startLamports", json!(0), "floor.startLamports"),
        ("floor.stepLamports", json!(0), "floor.stepLamports"),
        ("floor.startLamports", json!(u64::MAX), "the price of the curve's last seat"),
        ("floor.opensAfterSlots", json!(150), "floor.opensAfterSlots"),
        ("floor.opensAfterSlots", json!(1_296_001u64), "floor.opensAfterSlots"),
        ("floor.windowSlots", json!(0), "floor.windowSlots"),
        ("floor.windowSlots", json!(1_296_001u64), "floor.windowSlots"),
        ("floor.issuer", json!("not an address"), "floor.issuer"),
        ("floor.issuer", json!("PLACEHOLDER: the mint door's permit key"), "not his yet"),
        ("genesis.charter", json!("00"), "genesis.charter"),
        ("genesis.planCap", json!("many"), "genesis.planCap"),
        ("genesis.name", json!("n".repeat(65)), "genesis.name"),
        ("genesis.desc", json!("d".repeat(129)), "genesis.desc"),
        ("genesis.defaultMint", json!("not an address"), "genesis.defaultMint"),
        ("posiverse.chair", json!(12), "posiverse.chair"),
        ("posiverse.window", json!(99), "posiverse.window"),
        ("posiverse.charter", json!("11"), "posiverse.charter"),
        ("posiverse.name", json!("n".repeat(65)), "posiverse.name"),
        ("posiverse.agents", json!([]), "one to 12 agents"),
        ("warm.dollar", json!(""), "warm.dollar"),
        ("warm.opening", json!("yesterday"), "warm.opening"),
        ("warm.name", json!("n".repeat(33)), "the face"),
        ("warm.symbol", json!(""), "the face"),
        ("warm.uri", json!("u".repeat(201)), "the face"),
        // THE SWARM NAMES ITS MINER: the sixth act names the door's crank key, so it is an address of his before anything starts
        ("warm.miner", json!("not an address"), "warm.miner"),
        ("warm.miner", json!("PLACEHOLDER: the mint door's crank key"), "not his yet"),
        ("warm.miner", Value::Null, "warm.miner"),
        // THE TREASURY PAYS THE COMPUTE: the seventh act names root 0's cap of the dollar, the eighth the router's month of it; each more than
        // nothing and his, the month within the cap, the dollar the tree's default mint
        ("genesis.dollarCap", json!(0), "genesis.dollarCap"),
        ("genesis.dollarCap", json!("PLACEHOLDER: the swarm's dollar a period"), "genesis.dollarCap"),
        ("genesis.dollarCap", Value::Null, "genesis.dollarCap"),
        ("router.monthCap", json!(0), "router.monthCap"),
        ("router.monthCap", json!("PLACEHOLDER: the router's month"), "router.monthCap"),
        ("router.monthCap", json!(60000001), "could never be drawn whole"),
        ("warm.dollar", json!(poleis_press::params::placeholder_address("another dollar").to_string()), "CODEX_ALLOW_MINT"),
        // THE SWARM JUDGES: a road's receipts are a sealed act, so its judging window is longer than a vote of root 0 and of unit 1
        ("market.judgeSlots", json!(1000), "the swarm judges a road by its own vote"),
        ("market.judgeSlots", json!(0), "the swarm judges a road by its own vote"),
        ("market.split", json!(["nothing"]), "market.split"),
        ("market.barBp", json!(70_000), "market.barBp"),
        ("fund.feeBps", json!(70_000), "fund.feeBps"),
        ("fund.chairQuorum", json!("PLACEHOLDER: his"), "not his yet"),
        // THE LAUNCH NAMES THE SEAL'S ROAD: the founding writes it, so its node is held before anything starts; ANY MEMBER PROPOSES, THE
        // SWARM DECIDES: a seal.proposers line, whatever it holds, stops the sitting by its own words
        ("seal.rpc", json!("http://api.mainnet-beta.solana.com"), "seal.rpc"),
        ("seal.rpc", json!("https://mainnet.example/?api-key=x"), "names more than a host"),
        ("seal.rpc", Value::Null, "seal.rpc"),
        ("seal.proposers", json!("nobody"), "no key names who proposes a release"),
        ("seal.proposers", json!(["not an address"]), "no key names who proposes a release"),
        ("seal.proposers", json!([]), "no key names who proposes a release"),
    ];
    for (path, bad, says) in faults {
        let e = checked(&with(path, bad.clone())).expect_err(&format!("{path} = {bad} must stop the sitting"));
        assert!(e.contains(says), "{path} = {bad}: refused, by other words than its own: {e}");
    }
    // the same agent twice is no two chairs
    let mut twice = good();
    twice["posiverse"]["agents"][1] = twice["posiverse"]["agents"][0].clone();
    assert!(checked(&twice).unwrap_err().contains("the same address twice"));
}

/// THE WHOLE TREE DECIDES: no key of his is a param. A params file that names his Keystones (the floor this lane took out: three seats of
/// his, half of every decision) is refused as it is read, whatever the values, with the words that say why; so is one that still names the
/// first sale's terms, which the minted floor replaced.
#[test]
fn a_params_file_that_names_his_keystones_or_the_first_sale_is_refused() {
    let k = |w: &str| poleis_press::params::placeholder_address(w).to_string();
    let mut v = good();
    v["keystones"] = json!([k("his Keystone 1"), k("his Keystone 2"), k("his Keystone 3")]);
    assert_eq!(Params::from_text(&v.to_string()).err().unwrap(), poleis_press::params::KEYSTONES_NAMED);
    for path in poleis_press::params::FIRST_SALE {
        let e = Params::from_text(&with(path, json!(1)).to_string()).err().unwrap();
        assert!(e.contains(path) && e.contains(poleis_press::params::FIRST_SALE_NAMED), "{path}: {e}");
    }
    // both files the press ships are read: neither names his Keystones nor the first sale
    for file in ["params.mainnet.json", "params.rehearsal.json"] {
        let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/").to_string() + file).unwrap();
        Params::from_text(&text).unwrap_or_else(|e| panic!("{file}: {e}"));
    }
}

#[test]
fn the_treasury_is_derived_and_a_params_file_that_names_one_is_refused() {
    // THE TREASURY IS THE SWARM'S OWN: the mine's treasury is root 0's vault of the dollar, never a param
    let mut v = good();
    v["warm"]["treasury"] = json!(poleis_press::params::placeholder_address("his vault").to_string());
    assert_eq!(checked(&v).unwrap_err(), chain::TREASURY_NAMED);
    let p = Params::from_text(&good().to_string()).unwrap();
    assert_eq!(chain::treasury_of(&p).unwrap(), poleis_client::vault_pda(0, &p.key("warm.dollar").unwrap()), "root 0's own vault of the dollar");
}

/// MAINNET'S VALUES ARE HELD TOO (Nov 23 2026: his four answers of 19:32:23 UTC and the sheet's picks of 19:37:50): the params the sitting
/// will read, with what only the sitting can give put in (the keys it makes, the opening it reads off the live Worker), pass every check
/// before anything starts: Poleis's bounds, rung 5's founding in one packet, the priority fee's budget on every transaction the press
/// writes, act 5's execute with a list of 100 validators, the judging window holding a vote.
#[test]
fn mainnets_named_values_pass_every_check_before_anything_starts() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/params.mainnet.json")).unwrap();
    let mut v: Value = serde_json::from_str(&text).unwrap();
    let k = |w: &str| json!(poleis_press::params::placeholder_address(w).to_string());
    v["floor"]["issuer"] = k("the sitting's issuer");
    v["warm"]["miner"] = k("the sitting's miner");
    v["market"]["crediter"] = k("the sitting's crediter");
    v["posiverse"]["agents"] = json!([k("agent 1"), k("agent 2"), k("agent 3")]);
    v["warm"]["opening"] = json!("2026-11-15T00:00:00Z");
    // THE STAKE EARNS: engine.reserveBp, engine.validators and light.planCap are now named (Nov 26 2026); only engine.staker waits on Sitting 1
    let p = Params::from_text(&v.to_string()).unwrap();
    assert_eq!(p.placeholders(), Vec::<String>::new(), "only what the sitting gives was left");
    assert!(p.sendable().is_ok(), "the sitting sends with only the engine's sitting key left");
    assert_eq!(p.later_placeholders("engine"), ["engine.staker"]);
    assert_eq!(p.later_placeholders("light"), Vec::<String>::new());
    checked(&v).unwrap_or_else(|e| panic!("mainnet's params, the sitting's own values put in: {e}"));
    assert_eq!((p.u64("genesis.window").unwrap(), p.u64("posiverse.window").unwrap(), p.u64("market.judgeSlots").unwrap(), p.u64("fund.feeBps").unwrap(), p.u64("market.stepSlots").unwrap()), (216_000, 216_000, 432_000, 250, 150), "his four answers");
    assert!(p.at("market").get("judge").is_none(), "no judge key: the swarm judges");
    // and as the file stands, the seven the sitting gives are all that is left of the launch's
    let left = Params::from_text(&text).unwrap().placeholders();
    assert_eq!(left, ["floor.issuer", "market.crediter", "posiverse.agents[0]", "posiverse.agents[1]", "posiverse.agents[2]", "warm.miner", "warm.opening"]);
}

#[test]
fn the_judge_is_the_swarm_and_a_params_file_that_names_one_is_refused() {
    // THE SWARM JUDGES (his press of Nov 23 2026 19:32:23 UTC, "The swarm, by its own vote"): the board's judge is root 0's own hand, never a param
    let mut v = good();
    v["market"]["judge"] = json!(poleis_press::params::placeholder_address("a judge's key").to_string());
    assert_eq!(checked(&v).unwrap_err(), chain::JUDGE_NAMED);
    let (ids, ..) = build::compiled().unwrap();
    let at = chain::addresses(&ids, 77);
    let p = Params::from_text(&good().to_string()).unwrap();
    assert_eq!(chain::judge_of(&p, &at).unwrap(), at.agent, "root 0's own hand");
    assert_eq!(at.agent, poleis_client::agent_pda(0));
    // the judging window holds a whole vote: one slot past the longer of the two windows passes, the same length does not
    let mut v = good();
    v["genesis"]["window"] = json!(2000);
    v["market"]["judgeSlots"] = json!(2001);
    checked(&v).unwrap();
    v["market"]["judgeSlots"] = json!(2000);
    assert!(checked(&v).unwrap_err().contains("which can last 2000 slots"));
    v["genesis"]["window"] = json!(1000);
    v["posiverse"]["window"] = json!(3000);
    assert!(checked(&v).unwrap_err().contains("which can last 3000 slots"), "unit 1's vote counts too");
}

#[test]
fn the_founding_fits_one_transaction() {
    let mut v = good();
    v["posiverse"]["agents"][1] = v["posiverse"]["agents"][0].clone();
    assert!(checked(&v).unwrap_err().contains("the same address twice"));
    // twelve agents is the budget, and still one transaction; thirteen is past it
    let k = |i: usize| poleis_press::params::placeholder_address(&format!("agent {i}")).to_string();
    let mut v = good();
    v["posiverse"]["agents"] = json!((0..12).map(k).collect::<Vec<_>>());
    checked(&v).unwrap();
    v["posiverse"]["agents"] = json!((0..13).map(k).collect::<Vec<_>>());
    assert!(checked(&v).unwrap_err().contains("one to 12 agents"));
    // the longest names Poleis takes with twelve agents: the founding is past a packet, and the sitting stops before anything starts
    v["posiverse"]["agents"] = json!((0..12).map(k).collect::<Vec<_>>());
    v["posiverse"]["name"] = json!("n".repeat(64));
    v["posiverse"]["desc"] = json!("d".repeat(128));
    v["genesis"]["name"] = json!("n".repeat(64));
    v["genesis"]["desc"] = json!("d".repeat(128));
    assert!(checked(&v).unwrap_err().contains("past the 1232 one transaction holds"));
}

/// THE ENGINE'S BLANKS HOLD ONLY THE ENGINE (THE STAKE EARNS, Nov 25 2026): mainnet's params with the launch's values named and only the
/// engine's three blanks left pass the sitting's check; the engine's own steps refuse on them, each named, and the ninth act is no act yet
/// (the crank, a vote and a chair say "the engine's rule is not named yet").
#[test]
fn the_engines_blanks_hold_only_the_engine() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/params.mainnet.json")).unwrap();
    let mut v: Value = serde_json::from_str(&text).unwrap();
    let k = |w: &str| json!(poleis_press::params::placeholder_address(w).to_string());
    for (a, b) in [("floor", "issuer"), ("warm", "miner"), ("market", "crediter")] {
        v[a][b] = k(b);
    }
    v["posiverse"]["agents"] = json!([k("agent 1"), k("agent 2"), k("agent 3")]);
    v["warm"]["opening"] = json!("2026-11-15T00:00:00Z");
    let p = Params::from_text(&v.to_string()).unwrap();
    p.sendable().expect("the sitting is held only by the launch's values");
    checked(&v).expect("and every check of the launch passes");
    let e = poleis_press::engine::blanks(&p).unwrap_err();
    assert!(e.starts_with(poleis_press::engine::NOT_NAMED), "{e}");
    assert!(e.contains("engine.staker"), "the refusal names engine.staker: {e}");
    let (ids, ..) = build::compiled().unwrap();
    let at = chain::addresses(&ids, 77);
    let d = poleis_press::params::placeholder_address("blanks deployer");
    assert_eq!(poleis_press::engine::later(&p, &ids, &at, &d).unwrap().iter().map(|a| a.n).collect::<Vec<_>>(), [10], "act 10 stands once light.planCap is named; act 9 waits on engine.staker");
    assert!(poleis_press::engine::later_said(&p).unwrap().contains(poleis_press::engine::NOT_NAMED));
    let dir = std::env::temp_dir().join(format!("engine-blanks-{}", std::process::id()));
    assert!(poleis_press::engine::write_set(&dir, &p, &ids, &at, poleis_press::rpc::MAINNET_GENESIS, &d).unwrap_err().contains("engine.staker"), "press engine-act refuses");
    assert!(!dir.exists(), "and writes nothing");
    // blank reserveBp and validators too: all three named in refusal
    let mut vb = v.clone();
    vb["engine"]["reserveBp"] = json!("PLACEHOLDER: his");
    vb["engine"]["validators"] = json!("PLACEHOLDER: his");
    let pb = Params::from_text(&vb.to_string()).unwrap();
    let eb = poleis_press::engine::blanks(&pb).unwrap_err();
    for path in ["engine.reserveBp", "engine.staker", "engine.validators"] {
        assert!(eb.contains(path), "the refusal names {path}: {eb}");
    }
    // sitting engine key named: both acts 9 and 10 stand after the eight
    v["engine"]["staker"] = k("the sitting's engine key");
    let p = Params::from_text(&v.to_string()).unwrap();
    poleis_press::engine::blanks(&p).unwrap();
    assert_eq!(poleis_press::engine::later(&p, &ids, &at, &d).unwrap().iter().map(|a| a.n).collect::<Vec<_>>(), [9, 10]);
}
