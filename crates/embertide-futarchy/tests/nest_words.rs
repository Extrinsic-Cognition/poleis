//! THE NEST'S WORDS: a road, a trade, the signature. The signed entry in tests/signed-entry.json was made by the Nest's own author file
//! (tools/embertide-mesh/lib/nest-author.mjs signEntry) with a key made for this and thrown away: it is the witness that canonical(), body_of() and
//! verified() read an entry as the Nest writes it.
#![cfg(feature = "nest")]
use embertide_futarchy::{law::*, nest::*};
use serde_json::{json, Value};

#[test]
fn an_entry_the_nests_own_author_signed_verifies_and_a_changed_one_does_not() {
    let e: Value = serde_json::from_str(include_str!("signed-entry.json")).unwrap();
    assert!(verified(&e), "the Nest's own signature holds over this crate's body of the entry");
    assert!(body_of(&e).starts_with(r#"{"blobs":[],"cause":[41,7],"key":"302a300506032b6570032100"#), "the body's keys are sorted: {}", body_of(&e));
    assert!(body_of(&e).contains(r#""note":{"a":"café →","z":[3,{"a":"q\"uote\n","b":1}]}"#), "nested keys sorted, arrays in their order, strings as JSON writes them");
    let mut more = e.clone(); more["payload"]["amount"] = json!(12500001);
    assert!(!verified(&more), "one micro more and the signature does not hold");
    let mut other = e.clone(); other["seat"] = json!("another-seat");
    assert!(!verified(&other), "the seat is signed");
    let mut bare = e.clone(); bare.as_object_mut().unwrap().remove("sig");
    assert!(!verified(&bare), "an entry with no signature is not verified");
    let entry = entry_of(&e, "road-1");
    assert_eq!((entry.seq, entry.ts, entry.ok, entry.seat.as_str()), (9, 1794360000200, true, "witness-seat"), "the law reads the Nest's seq and the Nest's stamp, never the author's t");
    assert_eq!(entry.trade, Some(Trade { branch: Which::Pass, side: Side::Buy, amount: 12_500_000, min_out: 0 }));
    assert_eq!(entry_of(&e, "road-2").trade, None, "a trade of another road is no trade of this one");
}

#[test]
fn a_key_signs_what_the_law_verifies() {
    let key = Key::from_seed("warden-a", [7u8; 32]);
    let e = key.sign(json!({ "type": TRADE, "payload": trade_value("road-1", &Trade { branch: Which::Fail, side: Side::Sell, amount: 3, min_out: 1 }), "cause": [], "blobs": [], "t": null }));
    assert!(verified(&e));
    assert_eq!(e["key"].as_str().unwrap().len(), 88, "the key is the Ed25519 SPKI in hex, as the Nest keeps keys");
    assert_eq!(trade_of(&e["payload"], "road-1"), Some(Trade { branch: Which::Fail, side: Side::Sell, amount: 3, min_out: 1 }));
}

fn road() -> Value {
    json!({ "schema": SCHEMA, "id": "road-agent-x-2026-12", "kind": "allowances", "month": "2026-12", "grants": [{ "seat": "agent-x", "address": "AgentAddrXaaaa1111111111111111111111111111", "cap": 20_000_000 }], "opens_ms": 1_000, "closes_ms": 61_000 })
}

#[test]
fn a_road_takes_the_laws_defaults_and_only_what_the_law_can_carry() {
    let r = road_of(&road()).unwrap();
    assert_eq!(r.terms, Terms { opens_ms: 1_000, closes_ms: 61_000, step_ms: 60_000, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 1000 * UNIT, pool_quote: 1000 * UNIT, budget: 1000 * UNIT, both_ways: false });
    assert_eq!(road_of(&road_value(&r)), Some(r.clone()), "a road reads back from its own ref");
    let with = |k: &str, v: Value| { let mut x = road(); x[k] = v; road_of(&x) };
    assert!(with("schema", json!("embertide.futarchy/1")).is_none(), "the text's roads (floats) are not this law's");
    assert!(with("closes_ms", json!(1_000)).is_none());
    assert!(with("step_ms", json!(100)).is_none(), "a step under a second");
    assert!(with("month", json!("2026-13")).is_none());
    assert!(with("id", json!("Road 1")).is_none());
    assert!(with("budget", json!(12.5)).is_none() || with("budget", json!(12.5)).unwrap().terms.budget == 1000 * UNIT, "an amount that is no integer is not taken");
    assert!(with("grants", json!([])).is_none());
    assert!(with("pool", json!({ "base": 1, "quote": 2_000_000_000_000u64 })).is_none(), "an opening price outside the law's range");
    let words = json!({ "schema": SCHEMA, "id": "the-mint-stays-open", "kind": "road", "what": "Keep the mint open a week more", "opens_ms": 1_000, "closes_ms": 61_000, "step_ms": 1_000 });
    assert_eq!(road_of(&words).unwrap().kind, Kind::Road { what: "Keep the mint open a week more".into() });
}

#[test]
fn the_decision_and_the_allowances_as_their_refs_hold_them() {
    assert_eq!(price_text(1_393_200_000_000), "1.393200000000");
    assert_eq!(price_text(PRICE / 1_000_000), "0.000001000000");
    let r = road_of(&road()).unwrap();
    let t = r.terms;
    let es = [Entry { seq: 1, ts: 2_000, key: "k".into(), seat: "bull".into(), ok: true, trade: Some(Trade { branch: Which::Pass, side: Side::Buy, amount: 500 * UNIT, min_out: 0 }) }];
    let m = market_of(&t, &es, t.closes_ms);
    let d = decide(&t, &m, t.closes_ms);
    let v = decision_value(&r, &m, &d, &score_of(&t, &m, &d));
    assert_eq!((v["state"].as_str(), v["accepted"].as_u64(), v["twap"]["fail"].as_str()), (Some("failed"), Some(1), Some("1.000000000000")), "one step of a minute in a window of a minute: the average never moved");
    let Kind::Allowances { month, grants } = &r.kind else { panic!() };
    let first = merge_allowances(None, month, grants, &r.id, 5);
    assert_eq!(first["grants"][0]["proposal"], json!(r.id));
    let later = merge_allowances(Some(&first), month, &[Grant { seat: "agent-x".into(), address: grants[0].address.clone(), cap: 9 }, Grant { seat: "agent-a".into(), address: grants[0].address.clone(), cap: 1 }], "road-2", 6);
    assert_eq!((later["grants"][0]["seat"].as_str(), later["grants"][1]["cap"].as_u64(), later["grants"][1]["proposal"].as_str()), (Some("agent-a"), Some(9), Some("road-2")), "sorted by seat; a seat granted twice keeps the later road's cap");
}

#[test]
fn a_road_both_ways_with_a_judge_reads_back_and_one_without_writes_what_it_wrote_before() {
    // a road with neither writes, byte for byte, the text road_value wrote before the receipts
    let bare = road_of(&road()).unwrap();
    assert_eq!((bare.terms.both_ways, bare.terms.purse_of(bare.terms.budget), bare.judge.as_deref(), bare.roster.len()), (false, Purse { quote: 1000 * UNIT, base: 0 }, None, 0));
    assert_eq!(canonical(&road_value(&bare)), r#"{"budget":1000000000,"by":null,"closes_ms":61000,"grants":[{"address":"AgentAddrXaaaa1111111111111111111111111111","cap":20000000,"seat":"agent-x"}],"id":"road-agent-x-2026-12","kind":"allowances","max_step_bp":100,"month":"2026-12","opens_ms":1000,"opens_seq":0,"pool":{"base":1000000000,"quote":1000000000},"schema":"embertide.futarchy/2","step_ms":60000,"threshold_bp":300}"#);
    let judge = Key::from_seed("judge-j", [9u8; 32]);
    let mut v = road(); v["both_ways"] = json!(true); v["judge"] = json!(judge.key); v["pool"] = json!({ "base": 2000 * UNIT as u64, "quote": 1000 * UNIT as u64 });
    let r = road_of(&v).unwrap();
    assert_eq!((r.terms.both_ways, r.terms.purse_of(r.terms.budget), r.judge.as_deref()), (true, Purse { quote: 333_333_333, base: 666_666_667 }, Some(judge.key.as_str())), "claims at half a $WARM: a deposit of 1000 opens with a third as $WARM and two thirds as claims");
    assert_eq!(road_of(&road_value(&r)), Some(r.clone()), "a road both ways with a judge reads back from its own ref");
    assert_eq!((road_value(&r)["both_ways"].as_bool(), road_value(&r)["judge"].as_str(), road_value(&r).get("claim")), (Some(true), Some(judge.key.as_str()), None), "the ref says both ways and names no amount: the law derives it");
    let mut one = v.clone(); one["both_ways"] = json!(false);
    assert_eq!(road_of(&one).map(|r| (r.terms.both_ways, road_value(&r).get("both_ways").cloned())), Some((false, None)), "false is a road one way, and its ref does not name it");
    for not in [json!(1), json!("true"), json!(2000 * UNIT as u64)] { let mut no = v.clone(); no["both_ways"] = not; assert!(road_of(&no).is_none(), "both_ways is true or false: anything else is no road"); }
    let mut no = v.clone(); no["judge"] = json!("302a300506032b6570032100abc");
    assert!(road_of(&no).is_none(), "a judge that is not a key is no road");
    let mut no = v.clone(); no["judge"] = json!(judge.key.replace("302a3005", "402a3005"));
    assert!(road_of(&no).is_none(), "a judge that is not an Ed25519 SPKI is no road");
    let mut no = v.clone(); no["budget"] = json!(1);
    assert!(road_of(&no).is_none(), "a budget of one micro cannot hold both $WARM and claims: no road both ways");
    no["both_ways"] = json!(false);
    assert!(road_of(&no).is_some(), "and is a road one way");
}

#[test]
fn a_road_with_deposits_reads_back_with_its_roster_and_one_the_law_cannot_carry_is_no_road() {
    let (a, b, c) = (Key::from_seed("warden-a", [1u8; 32]), Key::from_seed("warden-b", [2u8; 32]), Key::from_seed("warden-c", [3u8; 32]));
    let mut v = road(); v["deposits"] = json!({});
    for (k, units) in [(&b, 600), (&a, 300), (&c, 1500)] { v["deposits"][&k.key] = json!(units * UNIT as u64); }
    let r = road_of(&v).unwrap();
    let mut want = vec![Deposit { key: a.key.clone(), amount: 300 * UNIT }, Deposit { key: b.key.clone(), amount: 600 * UNIT }, Deposit { key: c.key.clone(), amount: 1500 * UNIT }];
    want.sort_by(|x, y| x.key.cmp(&y.key));
    assert_eq!(r.roster, want, "the roster sorted by key");
    assert_eq!(road_of(&road_value(&r)), Some(r.clone()), "a road with deposits reads back from its own ref");
    assert_eq!(road_value(&r)["deposits"], v["deposits"], "its ref names the deposits as they were laid");
    let with = |key: &str, amount: Value| { let mut x = v.clone(); x["deposits"][key] = amount; road_of(&x) };
    assert!(with("302a300506032b6570032100abc", json!(UNIT as u64)).is_none(), "a key that is not a key is no road");
    assert!(with("warden-d", json!(UNIT as u64)).is_none(), "a seat is not a key");
    for not in [json!(12.5), json!(-1), json!("5"), json!(null), json!(1e6)] { assert!(with(&a.key, not.clone()).is_none(), "an amount of {not} is not an unsigned integer: no road"); }
    assert!(with(&a.key, json!(0)).is_none(), "a deposit of 0 is no road");
    assert!(with(&a.key, json!(AMOUNT_MAX as u64 + 1)).is_none(), "a deposit over a billion is no road");
    let mut both = v.clone(); both["both_ways"] = json!(true); both["deposits"][&a.key] = json!(1);
    assert!(road_of(&both).is_none(), "a roster the law refuses (a deposit of one micro both ways) is no road");
    both["deposits"][&a.key] = json!(300 * UNIT as u64);
    assert_eq!(road_of(&both).map(|r| r.roster.len()), Some(3), "and with a deposit that holds both it is a road");
    let mut arr = v.clone(); arr["deposits"] = json!([a.key, 300]);
    assert!(road_of(&arr).is_none(), "deposits are an object from a key to an amount");
    let mut null = v.clone(); null["deposits"] = Value::Null;
    let bare = road_of(&null).unwrap();
    assert_eq!((bare.roster.len(), road_value(&bare).get("deposits")), (0, None), "deposits of null are no roster, and the ref does not name them");
    let mut empty = v.clone(); empty["deposits"] = json!({});
    assert_eq!(road_of(&empty).map(|r| (r.roster.len(), road_value(&r).get("deposits").cloned())), Some((0, None)), "an empty object is no roster");
}

#[test]
fn the_receipts_are_the_judges_signed_word_after_the_close_and_nothing_else() {
    let judge = Key::from_seed("judge-j", [9u8; 32]);
    let mut v = road(); v["judge"] = json!(judge.key);
    let r = road_of(&v).unwrap();
    let close = r.terms.closes_ms;
    let said = |k: &Key, payload: Value, ts: u64| { let mut e = k.sign(json!({ "type": RECEIPTS, "payload": payload, "cause": [], "blobs": [], "t": null })); e["ts"] = json!(ts); e };
    let word = receipts_value(&r.id, &Receipts { world: Which::Fail, worth: PRICE / 4 });
    assert_eq!(word, json!({ "proposal": r.id, "world": "fail", "worth": 250_000_000_000u64 }));
    assert_eq!(receipts_of(&said(&judge, word.clone(), close), &r), Some(Receipts { world: Which::Fail, worth: PRICE / 4 }), "the judge's signed word at the close");
    assert_eq!(receipts_of(&said(&judge, receipts_value(&r.id, &Receipts { world: Which::Pass, worth: PRICE }), close + 5), &r), Some(Receipts { world: Which::Pass, worth: PRICE }));
    assert_eq!(receipts_of(&said(&Key::from_seed("judge-k", [8u8; 32]), word.clone(), close), &r), None, "another key's word");
    let mut bare = said(&judge, word.clone(), close); bare.as_object_mut().unwrap().remove("sig");
    assert_eq!(receipts_of(&bare, &r), None, "an unsigned word");
    assert_eq!(receipts_of(&said(&judge, json!({ "proposal": "road-2", "world": "fail", "worth": 1 }), close), &r), None, "another road's");
    assert_eq!(receipts_of(&said(&judge, json!({ "proposal": r.id, "world": "fail", "worth": (PRICE + 1) as u64 }), close), &r), None, "a worth over one $WARM");
    assert_eq!(receipts_of(&said(&judge, json!({ "proposal": r.id, "world": "maybe", "worth": 1 }), close), &r), None, "a world that is neither");
    assert_eq!(receipts_of(&said(&judge, word.clone(), close - 1), &r), None, "a word stamped before the close");
    let mut trade = said(&judge, word.clone(), close); trade["type"] = json!(TRADE);
    assert_eq!(receipts_of(&judge.sign(trade), &r), None, "a trade is no receipts");
    let unjudged = road_of(&road()).unwrap();
    assert_eq!(receipts_of(&said(&judge, word, close), &unjudged), None, "a road that names no judge has no receipts");
}

#[test]
fn a_word_of_the_other_world_counts_for_nothing_and_the_first_word_of_this_one_stands() {
    let judge = Key::from_seed("judge-j", [9u8; 32]);
    let mut v = road(); v["judge"] = json!(judge.key);
    let r = road_of(&v).unwrap();
    let close = r.terms.closes_ms;
    let said = |k: &Key, world: Which, worth: u128, ts: u64| { let mut e = k.sign(json!({ "type": RECEIPTS, "payload": receipts_value(&r.id, &Receipts { world, worth }), "cause": [], "blobs": [], "t": null })); e["ts"] = json!(ts); e };
    // the Nest's order: the judge's word of PASS first, a word of FAIL before the close, another key's word of FAIL, then the judge's two words of FAIL
    let log = [said(&judge, Which::Pass, PRICE / 2, close), said(&judge, Which::Fail, PRICE / 8, close - 1), said(&Key::from_seed("judge-k", [8u8; 32]), Which::Fail, PRICE / 9, close + 1),
        said(&judge, Which::Fail, PRICE / 4, close + 2), said(&judge, Which::Fail, PRICE / 5, close + 3)];
    assert_eq!(receipts_among(&log, &r, Which::Fail), Some(Receipts { world: Which::Fail, worth: PRICE / 4 }), "the road failed: the word of PASS before it does not stand in its way, and the first word of FAIL after the close stands");
    assert_eq!(receipts_among(&log, &r, Which::Pass), Some(Receipts { world: Which::Pass, worth: PRICE / 2 }), "had it passed, the word of PASS would be its receipts");
    assert_eq!(receipts_among(&log[..3], &r, Which::Fail), None, "until the judge says the world that happened there are no receipts");
    assert_eq!(receipts_among(&log[..0], &r, Which::Pass), None);
}

#[test]
fn a_road_with_a_fee_is_read_folded_with_it_and_shown() {
    let mut v = road(); v["fee_bp"] = json!(100);
    let r = road_of(&v).unwrap();
    assert_eq!(r.terms.fee_bp, 100, "the ref names the fee");
    assert_eq!((road_value(&r)["fee_bp"].as_u64(), road_of(&road_value(&r)).map(|x| x.terms)), (Some(100), Some(r.terms)), "a road with a fee reads back from its own ref");
    assert_eq!((road_of(&road()).unwrap().terms.fee_bp, road_value(&road_of(&road()).unwrap()).get("fee_bp")), (0, None), "a road that names none keeps none, and its ref does not name it");
    for not in [json!(10_000), json!(12.5), json!(-1), json!("100")] { let mut no = road(); no["fee_bp"] = not.clone(); assert!(road_of(&no).is_none(), "a fee of {not} is no road"); }
    let mut most = road(); most["fee_bp"] = json!(9_999);
    assert_eq!(road_of(&most).map(|r| r.terms.fee_bp), Some(9_999));
    // folded with it: the buy pays 5 of its 500 beside the pool, the net of 495 buys, and the pool's $WARM grows by the net alone
    let es = [Entry { seq: 1, ts: 2_000, key: "k".into(), seat: "bull".into(), ok: true, trade: Some(Trade { branch: Which::Pass, side: Side::Buy, amount: 500 * UNIT, min_out: 0 }) }];
    let t = r.terms;
    let with = market_of(&t, &es, t.closes_ms);
    assert_eq!(with.traders[0].pass.base, swap_out(t.pool_base, t.pool_quote, Side::Buy, 495 * UNIT).unwrap(), "the net of 495 buys");
    assert_eq!((with.pass.quote, with.pass.fees, with.fail.fees), (t.pool_quote + 495 * UNIT, 5 * UNIT, 0));
    let d = decide(&t, &with, t.closes_ms);
    let reading = Reading { road: r.clone(), scores: score_of(&t, &with, &d), market: with, decision: d, now: t.closes_ms };
    let shown = reading_value(&reading);
    assert_eq!((shown["fee_bp"].as_u64(), shown["fees"]["pass"].as_u64(), shown["fees"]["fail"].as_u64()), (Some(100), Some(5 * UNIT as u64), Some(0)), "the reading shows the fee and each branch's fees");
    // a bare road's reading names neither
    let bare = road_of(&road()).unwrap();
    let m = market_of(&bare.terms, &es, bare.terms.closes_ms);
    let d = decide(&bare.terms, &m, bare.terms.closes_ms);
    let shown = reading_value(&Reading { road: bare.clone(), scores: score_of(&bare.terms, &m, &d), market: m, decision: d, now: bare.terms.closes_ms });
    assert_eq!((shown.get("fee_bp"), shown.get("fees")), (None, None));
}
