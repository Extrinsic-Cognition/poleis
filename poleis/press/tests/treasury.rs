//! THE TREASURY PAYS THE COMPUTE (Nov 23 2026, lane/the-treasury-pays-the-compute; his 23:08:00 UTC "Yes, add them" to "Add two first acts
//! to the launch so the treasury pays for the AI calls?"). Once the $WARM mine is named, people's deposits of the dollar land in root 0's own
//! vault of it; root 0's cap for its default mint (`token_plan_cap`) starts at 0 and only a CODEX_PLAN moves it, so with the six acts no
//! dollar could ever leave that vault, and the router pays the AI calls from outside the treasury. The press's seventh act is that
//! CODEX_PLAN: it restates the founding's plan cap and period exactly (the execute overwrites all three and zeroes what was spent) and names
//! the params' `genesis.dollarCap`. Its eighth is a TREASURY allowance of root 0 to the mine's miner, the door's crank key the sixth act
//! names: `router.monthCap` of the dollar a plan period, standing until the polis revokes it. Each is decided by the whole tree like the
//! others; here their payloads are held to the client's own bytes, their sets to what an agent votes, and their params to what the press
//! refuses. The live half (both executed by press crank, then the miner's pull out of root 0's vault, a pull past the cap and a stranger's
//! refused) is the launch rehearsal's: poleis/ceremony/the-launch-press-rehearsal.sh.
use poleis_client::{self as client, Payload, CLASS_CODEX, CLASS_TREASURY, HOLDER_KEY, ROOT_UNIT};
use poleis_press::chair::{self, Reading};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::{acts, build, chain, wire};
use serde_json::{json, Value};

const GENESIS: &str = "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY";

/// Params of a mainnet shape, every value given, the dollar the tree's default mint; `dollar_cap` and `month_cap` the two values these
/// tests move.
fn params(dollar_cap: Value, month_cap: Value) -> Value {
    let s = |w: &str| k(w).to_string();
    json!({
        "cluster": "mainnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "T", "desc": "", "defaultMint": s("treasury dollar"), "planCap": 1000000000u64, "planPeriod": 6480000, "dollarCap": dollar_cap, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("treasury issuer") },
        "posiverse": { "name": "T", "desc": "", "chair": 2, "agents": [s("treasury agent 1"), s("treasury agent 2"), s("treasury agent 3")], "charter": "33".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "https://optimism.fit/app/warm-token.json", "dollar": s("treasury dollar"), "opening": "the founding", "miner": s("treasury miner") },
        "router": { "monthCap": month_cap },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1000000000u64, "poolQuote": 1000000000u64, "judgeSlots": 1296001u64,
                    "crediter": s("treasury crediter"), "split": ["treasury"], "plan": ["treasury", "act"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "seal": { "rpc": "https://api.mainnet-beta.solana.com" },
        "pool": { "maxValidators": 10, "tag": "lp1", "programSha256": "ab".repeat(32) }
    })
}

fn read(v: &Value) -> Params {
    Params::from_text(&v.to_string()).unwrap()
}

fn good() -> Params {
    read(&params(json!(3_000_000_000u64), json!(1_500_000_000u64)))
}

fn the_acts(p: &Params) -> Vec<acts::Act> {
    let (ids, ..) = build::compiled().unwrap();
    acts::acts(p, &ids, &chain::addresses(&ids, 77), &k("treasury deployer")).unwrap()
}

fn checked(v: &Value) -> Result<(), String> {
    let (ids, ..) = build::compiled().unwrap();
    chain::params_checked(&read(v), &ids, &chain::addresses(&ids, 77))
}

#[test]
fn the_seventh_act_restates_the_plan_and_names_the_dollar_cap() {
    let p = good();
    let all = the_acts(&p);
    assert_eq!(all.iter().map(|a| a.n).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6, 7, 8], "eight acts, in order");
    let seven = &all[6];
    let want = Payload::Plan { origin: ROOT_UNIT, cap: 1_000_000_000, period: 6_480_000, token_cap: 3_000_000_000 };
    assert_eq!((seven.n, seven.file, seven.class, seven.impacted.clone(), seven.convene(), seven.nonce, seven.needs, seven.before.len()), (acts::DOLLAR_CAP_ACT, "7-the-dollar-cap.json", CLASS_CODEX, vec![ROOT_UNIT, 1], false, 7, None, 0), "a CODEX act of root 0, closed over unit 1 (its proposal opens unit 1's vote), the seventh nonce, nothing made first");
    assert_eq!((seven.payload.clone(), seven.payload_sha256), (want.bytes(), want.hash()), "the client's own CODEX_PLAN: the founding's plan cap and period restated, the params' dollar cap");
    assert_eq!(seven.payload.len(), 26, "[class][kind][cap:8][period:8][token cap:8]");
    assert_eq!(seven.hash, client::proposal_hash(CLASS_CODEX, ROOT_UNIT, &seven.manifest, &want.hash(), 7), "the proposal's hash covers the cap it names");
    let e = seven.execute_by(&k("a crank"));
    assert_eq!(e.accounts, client::execute_of(ROOT_UNIT, k("a crank"), seven.hash, &seven.impacted, &want).accounts, "its execute: the client's own, root 0's registry writable last");
    assert_eq!(e.accounts.last().map(|m| (m.pubkey, m.is_writable)), Some((client::registry_pda(ROOT_UNIT), true)));
    // another dollar cap is another act; the six before it do not move
    let other = the_acts(&read(&params(json!(3_000_000_001u64), json!(1_500_000_000u64))));
    assert_ne!(other[6].hash, seven.hash);
    assert_eq!(other[..6].iter().map(|a| a.hash).collect::<Vec<_>>(), all[..6].iter().map(|a| a.hash).collect::<Vec<_>>(), "and the six before it do not");
}

#[test]
fn the_eighth_act_is_the_routers_month_for_the_mines_miner() {
    let p = good();
    let all = the_acts(&p);
    let eight = &all[7];
    let (miner, dollar) = (k("treasury miner"), k("treasury dollar"));
    let want = Payload::Allowance { origin: ROOT_UNIT, payee: miner.to_bytes(), payee_kind: HOLDER_KEY, mint: dollar, cap: 1_500_000_000, period: 6_480_000, expires: 0 };
    assert_eq!((eight.n, eight.file, eight.class, eight.impacted.clone(), eight.convene(), eight.nonce, eight.needs, eight.before.len()), (acts::ROUTER_ACT, "8-the-routers-month.json", CLASS_TREASURY, vec![ROOT_UNIT], true, 8, None, 0), "a plain TREASURY act of root 0, unit 1 convened by the crank, the eighth nonce, nothing made first");
    assert_eq!(eight.class & (client::CLASS_MARKET | client::CLASS_PLAN | client::CLASS_CONSENT), 0, "no market, plan or consent bit");
    assert_eq!((eight.payload.clone(), eight.payload_sha256), (want.bytes(), want.hash()), "the client's own allowance: the miner's key, the dollar, the month's cap, the plan's period, no expiry");
    assert_eq!(eight.payload.len(), 91, "[class][kind][payee:32][kind:1][mint:32][cap:8][period:8][expires:8]");
    assert_eq!(eight.hash, client::proposal_hash(CLASS_TREASURY, ROOT_UNIT, &eight.manifest, &want.hash(), 8));
    let e = eight.execute_by(&k("a crank"));
    assert_eq!(e.accounts, client::execute_of(ROOT_UNIT, k("a crank"), eight.hash, &eight.impacted, &want).accounts, "its execute: the client's own");
    assert!(e.accounts.iter().any(|m| m.pubkey == client::allowance_pda(&eight.hash) && m.is_writable), "the allowance made at its proposal's own address, the crank its payer");
    let (execute, proposed) = (acts::execute_bytes(&p, eight, &k("a crank"), false).unwrap(), poleis_press::floor::proposal_bytes(&p, eight, &poleis_press::floor::placeholder_owner()).unwrap());
    assert!(execute <= wire::PACKET && proposed <= wire::PACKET, "its execute and a seat owner's proposal of it: one transaction each");
    eprintln!("  act 7: payload {} bytes; act 8: payload {} bytes, its execute's transaction {execute} bytes, a seat owner's proposal of it {proposed} bytes", all[6].payload.len(), eight.payload.len());
    // with mainnet's priority fee each carries its compute budget too (the crank's 120,000 units on an execute, 80,000 on a proposal): still one
    // transaction each, the seventh's execute with it
    let mut v = params(json!(3_000_000_000u64), json!(1_500_000_000u64));
    v["priorityMicroLamports"] = json!(100_000);
    let priced = read(&v);
    let priced_all = the_acts(&priced);
    assert_eq!(priced_all.iter().map(|a| a.hash).collect::<Vec<_>>(), all.iter().map(|a| a.hash).collect::<Vec<_>>(), "a priority fee moves no act");
    let sized = |n: usize| (acts::execute_bytes(&priced, &priced_all[n], &k("a crank"), false).unwrap(), poleis_press::floor::proposal_bytes(&priced, &priced_all[n], &poleis_press::floor::placeholder_owner()).unwrap());
    let ((seven_execute, seven_proposed), (priced_execute, priced_proposed)) = (sized(6), sized(7));
    assert!([seven_execute, seven_proposed, priced_execute, priced_proposed].iter().all(|b| *b <= wire::PACKET), "with a priority fee's compute budget, each execute and each proposal of the two: one transaction");
    assert!(priced_execute > execute && priced_proposed > proposed, "the budget is counted in both");
    eprintln!("  with a priority fee's compute budget: act 7's execute {seven_execute} bytes, its proposal {seven_proposed}; act 8's execute {priced_execute} bytes, its proposal {priced_proposed}; of {}", wire::PACKET);
    // another miner moves the eighth and not the seventh
    let mut v = params(json!(3_000_000_000u64), json!(1_500_000_000u64));
    v["warm"]["miner"] = json!(k("another miner").to_string());
    let other = the_acts(&read(&v));
    assert_ne!(other[7].hash, eight.hash);
    assert_eq!(other[6].hash, all[6].hash);
}

#[test]
fn the_press_refuses_a_dollar_cap_or_a_month_it_could_not_send() {
    checked(&params(json!(3_000_000_000u64), json!(1_500_000_000u64))).unwrap();
    // the month may be the whole cap
    checked(&params(json!(1_500_000_000u64), json!(1_500_000_000u64))).unwrap();
    for (dollar_cap, month_cap, says) in [
        (json!(0), json!(1), "genesis.dollarCap"),
        (json!("PLACEHOLDER: his dollar cap"), json!(1), "genesis.dollarCap"),
        (Value::Null, json!(1), "genesis.dollarCap"),
        (json!(10), json!(0), "router.monthCap"),
        (json!(10), json!("PLACEHOLDER: the router's month"), "router.monthCap"),
        (json!(10), Value::Null, "router.monthCap"),
        (json!(10), json!(11), "could never be drawn whole"),
    ] {
        let e = checked(&params(dollar_cap.clone(), month_cap.clone())).expect_err(&format!("dollarCap {dollar_cap}, monthCap {month_cap} must stop the sitting"));
        assert!(e.contains(says), "dollarCap {dollar_cap}, monthCap {month_cap}: refused by other words than its own: {e}");
    }
    // the dollar the mine counts in is the tree's default mint, or the allowance would need a CODEX_ALLOW_MINT listing first
    let mut v = params(json!(10), json!(5));
    v["warm"]["dollar"] = json!(k("another dollar").to_string());
    let e = checked(&v).unwrap_err();
    assert!(e.contains("warm.dollar") && e.contains("genesis.defaultMint") && e.contains("CODEX_ALLOW_MINT"), "{e}");
}

#[test]
fn the_agents_vote_both_as_acts_the_params_give_and_the_sets_hold_both() {
    let p = good();
    let all = the_acts(&p);
    let (ids, ..) = build::compiled().unwrap();
    let dir = std::env::temp_dir().join(format!("poleis-press-treasury-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // where no stake pool program stands the pool's set is not written; the seventh's and the eighth's are, everywhere
    acts::write(&dir, &p, &ids, &chain::addresses(&ids, 77), GENESIS, &k("treasury deployer"), false).unwrap();
    let read_set = |f: &str| serde_json::from_str::<Value>(&std::fs::read_to_string(dir.join(f)).unwrap()).unwrap();
    let index = read_set("index.json");
    assert_eq!(index["acts"].as_array().unwrap().iter().map(|a| (a["n"].as_u64().unwrap(), a["file"].as_str().unwrap().to_string())).collect::<Vec<_>>()[4..], [(6, "6-the-miner.json".to_string()), (7, "7-the-dollar-cap.json".to_string()), (8, "8-the-routers-month.json".to_string())], "the index: the miner, then the dollar cap, then the router's month");
    let (seven, eight) = (read_set("7-the-dollar-cap.json"), read_set("8-the-routers-month.json"));
    assert_eq!((seven["act"]["n"].clone(), seven["act"]["convened"].clone(), eight["act"]["n"].clone(), eight["act"]["convened"].clone()), (json!(7), json!(false), json!(8), json!(true)));
    assert_eq!((seven["plan"]["dollarCap"].clone(), seven["plan"]["planCap"].clone(), seven["plan"]["planPeriod"].clone()), (json!(3_000_000_000u64), json!(1_000_000_000u64), json!(6_480_000)), "the seventh's set says the cap and the plan it restates");
    assert_eq!((eight["router"]["payee"].as_str(), eight["router"]["mint"].as_str(), eight["router"]["cap"].clone(), eight["router"]["expires"].clone()), (Some(k("treasury miner").to_string().as_str()), Some(k("treasury dollar").to_string().as_str()), json!(1_500_000_000u64), json!(0)), "the eighth's set names its payee, the dollar, the month, no expiry");
    assert_eq!(eight["router"]["allowance"].as_str(), Some(client::allowance_pda(&all[7].hash).to_string().as_str()));
    // THE READING: an agent's hands put a set to its mind as the act the params give only when it is, and both are (its mind then chooses)
    let me = k("treasury agent 2");
    for (set, n) in [(&seven, 7), (&eight, 8)] {
        assert!(matches!(chair::read_set(set, &all, &me, "mainnet", GENESIS), Reading::Act(ref a) if a.n == n && a.differs.is_none()), "act {n}: read whole, the act the params give");
    }
    // a month the params do not give is never read as the launch's
    let mut moved = eight.clone();
    let other = Payload::Allowance { origin: ROOT_UNIT, payee: k("treasury miner").to_bytes(), payee_kind: HOLDER_KEY, mint: k("treasury dollar"), cap: 1_500_000_001, period: 6_480_000, expires: 0 };
    moved["act"]["payload"] = json!(wire::b64(&other.bytes()));
    assert!(!matches!(chair::read_set(&moved, &all, &me, "mainnet", GENESIS), Reading::Act(ref a) if a.differs.is_none()));
    let _ = std::fs::remove_dir_all(&dir);
}
