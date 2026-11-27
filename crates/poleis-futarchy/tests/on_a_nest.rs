//! THE MARKET RUNS ON A NEST: the house's own nest.mjs as a scratch Nest over a scratch folder (ended and removed after), invented seats with keys
//! made here. Two roads; three wardens trade them by signed causes; one trade comes unsigned; a whale spikes road 2's PASS ten seconds before the
//! close and follows it with 150 dust trades. At the close, by the Nest's clock, both are settled: road 1 passes and grants its allowance, road 2
//! fails, the spike and its spam notwithstanding. Then a third road, both ways, with a judge and a roster of the three wardens' own deposits: two
//! sell, one buys, the whale off the roster is refused; the judge's word is refused
//! before the close and when it names the world that did not happen, a word of that world put on the log all the same counts for nothing, and the
//! judge's signed receipts of the world that happened settle it, counted once. It takes the windows (about 55 s) and needs node, so it is asked for
//! by name:
//!   cargo test --test on_a_nest -- --ignored --nocapture
#![cfg(feature = "nest")]
use poleis_futarchy::{law::*, nest::*};
use serde_json::{json, Value};
use std::io::Read;
use std::process::{Child, Command, Stdio};

struct Scratch { child: Child, dir: std::path::PathBuf }
impl Drop for Scratch { fn drop(&mut self) { let _ = self.child.kill(); let _ = self.child.wait(); let _ = std::fs::remove_dir_all(&self.dir); } }

const SERVE: &str = r#"
import fs from 'node:fs'; import { generateKeyPairSync } from 'node:crypto';
const nestFile = process.env.FUTARCHY_NEST_FILE, dir = process.env.NEST_DIR, port = process.env.NEST_PORT;   /* never an argument: nest.mjs named as argv[1] runs as the house's own Nest */
const keys = process.env.NEST_KEYS; fs.mkdirSync(keys, { recursive: true });
const { privateKey, publicKey } = generateKeyPairSync('ed25519');
fs.writeFileSync(keys + '/proposer-n.key.json', JSON.stringify({ seat: 'proposer-n', key: publicKey.export({ format: 'der', type: 'spki' }).toString('hex'), private_pem: privateKey.export({ format: 'pem', type: 'pkcs8' }) }));
const { Nest, serve } = await import(nestFile);
serve(new Nest(dir), { port: Number(port), bind: '127.0.0.1', upstream: '' });
"#;

fn seed() -> [u8; 32] { let mut b = [0u8; 32]; std::fs::File::open("/dev/urandom").unwrap().read_exact(&mut b).unwrap(); b }
fn wait_until(nest: &Nest, ms: u64) { while nest.read("futarchy/none").map(|r| r.2).unwrap_or(0) < ms { std::thread::sleep(std::time::Duration::from_millis(40)); } }

#[test]
#[ignore = "45 s on a scratch Nest; needs node: cargo test --test on_a_nest -- --ignored --nocapture"]
fn two_roads_are_traded_and_settled_on_a_scratch_nest() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let dir = std::env::temp_dir().join(format!("futarchy-on-a-nest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    // the Nest's file, its folder and its port go by the environment, and NEST_DIR and NEST_PORT are the scratch ones: whatever nest.mjs does at its load,
    // it does it there (a first cut named the file as an argument, and nest.mjs then ran as a house Nest over this Mac's default folder)
    let child = Command::new("node").args(["--input-type=module", "-e", SERVE]).env("FUTARCHY_NEST_FILE", root.join("tools/embertide-mesh/nest.mjs")).env("NEST_DIR", dir.join("nest")).env("NEST_PORT", port.to_string())
        .env("NEST_KEYS", dir.join("keys")).env_remove("NEST_UPSTREAM").env_remove("NEST_PEERS").stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("node starts the scratch Nest");
    let _scratch = Scratch { child, dir: dir.clone() };
    let nest = Nest::new(&format!("http://127.0.0.1:{port}"));
    let mut up = false;
    for _ in 0..1200 { if nest.seq().is_ok() { up = true; break; } std::thread::sleep(std::time::Duration::from_millis(50)); }
    assert!(up, "the scratch Nest answers");

    // the proposer's key is a file the Nest's own keygen shape wrote; the wardens' keys are made here
    let proposer = Key::load(&dir.join("keys"), "proposer-n").expect("a key file as nest.mjs keygen writes it");
    let (a, b, c, whale) = (Key::from_seed("warden-a", seed()), Key::from_seed("warden-b", seed()), Key::from_seed("warden-c", seed()), Key::from_seed("warden-whale", seed()));
    let now = nest.read("futarchy/none").unwrap().2;
    let (opens, closes) = (now + 1_500, now + 41_500);
    let terms = json!({ "schema": SCHEMA, "opens_ms": opens, "closes_ms": closes, "step_ms": 1_000, "max_step_bp": 100, "threshold_bp": 300 });
    let mut r1 = terms.clone(); r1["id"] = json!("road-agent-x-2026-12"); r1["kind"] = json!("allowances"); r1["month"] = json!("2026-12");
    r1["grants"] = json!([{ "seat": "agent-x", "address": "AgentAddrXaaaa1111111111111111111111111111", "cap": 20_000_000 }]);
    let mut r2 = terms.clone(); r2["id"] = json!("the-spiked-road"); r2["kind"] = json!("road"); r2["what"] = json!("A road the wardens doubt and a whale wants");
    assert!(propose(&nest, &proposer, r1.clone()).unwrap(), "road 1 is laid");
    assert!(propose(&nest, &proposer, r2).unwrap(), "road 2 is laid");
    assert!(!propose(&nest, &a, r1).unwrap(), "a road is laid once: a second hand's does not replace it");

    wait_until(&nest, opens + 100);
    let buy = |k: &Key, id: &str, branch: Which, units: u128| { trade(&nest, k, id, &Trade { branch, side: Side::Buy, amount: units, min_out: 0 }).unwrap(); };
    buy(&a, "road-agent-x-2026-12", Which::Pass, 300 * UNIT); buy(&b, "road-agent-x-2026-12", Which::Pass, 150 * UNIT); buy(&c, "road-agent-x-2026-12", Which::Fail, 60 * UNIT);
    buy(&a, "the-spiked-road", Which::Fail, 300 * UNIT); buy(&b, "the-spiked-road", Which::Fail, 120 * UNIT); buy(&c, "the-spiked-road", Which::Pass, 40 * UNIT);
    nest.append(None, "warden-u", TRADE, trade_value("the-spiked-road", &Trade { branch: Which::Pass, side: Side::Buy, amount: 900 * UNIT, min_out: 0 })).expect("the Nest takes an unsigned cause; the market does not");
    assert!(settle(&nest, &proposer, "the-spiked-road").unwrap().is_none(), "a settle before the close writes nothing");

    wait_until(&nest, closes - 10_000);
    buy(&whale, "the-spiked-road", Which::Pass, 900 * UNIT);
    let spam = std::time::Instant::now();
    for _ in 0..150 { buy(&whale, "the-spiked-road", Which::Pass, 20); }   // the same trade 150 times: each is its own entry by its author's instant
    let spam_ms = spam.elapsed().as_millis();
    let mid = read_market(&nest, "the-spiked-road").unwrap().unwrap();
    assert_eq!(mid.decision.state, State::Open);
    eprintln!("  the whale's spike and 150 dust trades in {spam_ms} ms: PASS's spot {} and its average so far {}", price_text(mid.decision.spot_pass), price_text(mid.decision.twap_pass.unwrap()));

    wait_until(&nest, closes + 200);
    let d1 = settle(&nest, &proposer, "road-agent-x-2026-12").unwrap().expect("after the close the road is decided");
    let d2 = settle(&nest, &proposer, "the-spiked-road").unwrap().unwrap();
    eprintln!("  road 1: {} (PASS {} against FAIL {}, needs {})", d1["state"], d1["twap"]["pass"], d1["twap"]["fail"], d1["needs"]);
    eprintln!("  road 2: {} (PASS {} against FAIL {}, needs {}); {} trades taken, {} refused", d2["state"], d2["twap"]["pass"], d2["twap"]["fail"], d2["needs"], d2["accepted"], d2["refused"]);
    assert_eq!(d1["state"], json!("passed"), "the road the wardens backed passes");
    assert_eq!(d2["state"], json!("failed"), "the road they doubted fails, the whale's spike and its spam notwithstanding");
    assert_eq!((d2["accepted"].as_u64(), d2["refused"].as_u64()), (Some(154), Some(1)), "every signed trade is taken (3 wardens, the spike, 150 dust); the unsigned one is refused");
    let read = std::time::Instant::now();
    let why = read_market(&nest, "the-spiked-road").unwrap().unwrap().market.refused[0].why;
    eprintln!("  cost: one reading of road 2 (two asks of the Nest, 155 signatures checked, the fold) took {:.1} ms", read.elapsed().as_secs_f64() * 1000.0);
    assert_eq!(why, UNSIGNED);

    let (allow, _, _) = nest.read("futarchy/allowances/2026-12").unwrap();
    let allow = allow.expect("the road that passed granted its month's allowances");
    assert_eq!((allow["grants"][0]["seat"].as_str(), allow["grants"][0]["cap"].as_u64(), allow["grants"][0]["proposal"].as_str()), (Some("agent-x"), Some(20_000_000), Some("road-agent-x-2026-12")));
    // a second settle, by another hand, writes nothing: the decision stands as the first wrote it
    let rev = |name: &str| nest.read(name).unwrap().1;
    let before = (rev("futarchy/decisions/the-spiked-road"), rev("futarchy/decisions/road-agent-x-2026-12"), rev("futarchy/allowances/2026-12"));
    let again: Value = settle(&nest, &a, "road-agent-x-2026-12").unwrap().unwrap();
    settle(&nest, &a, "the-spiked-road").unwrap();
    assert_eq!(again, d1);
    assert_eq!(before, (rev("futarchy/decisions/the-spiked-road"), rev("futarchy/decisions/road-agent-x-2026-12"), rev("futarchy/allowances/2026-12")), "a second settle moved no ref");
    assert_eq!(before, (1, 1, 1));

    // a third road, both ways (every purse opens with the pool's own mix) and with a judge whose signed word after the close settles it
    let judge = Key::from_seed("judge-j", seed());
    let now = nest.read("futarchy/none").unwrap().2;
    let (opens, closes) = (now + 1_000, now + 9_000);
    let mut r3 = json!({ "schema": SCHEMA, "id": "the-judged-road", "kind": "road", "what": "A road its judge will read the receipts of", "opens_ms": opens, "closes_ms": closes, "step_ms": 1_000,
        "max_step_bp": 100, "threshold_bp": 300, "both_ways": true, "judge": judge.key, "deposits": {} });
    // its roster: the three wardens, each with its own deposit; the whale is not on it
    for (k, units) in [(&a, 300u64), (&b, 600), (&c, 1500)] { r3["deposits"][&k.key] = json!(units * UNIT as u64); }
    assert!(propose(&nest, &proposer, r3).unwrap(), "road 3 is laid");
    wait_until(&nest, opens + 100);
    let sell = |k: &Key, units: u128| { trade(&nest, k, "the-judged-road", &Trade { branch: Which::Pass, side: Side::Sell, amount: units, min_out: 0 }).unwrap(); };
    sell(&a, 30 * UNIT); sell(&b, 20 * UNIT); buy(&c, "the-judged-road", Which::Fail, 50 * UNIT);
    buy(&whale, "the-judged-road", Which::Pass, 100 * UNIT);   // signed, so the Nest takes it; the market refuses it: the whale is not on the roster
    assert!(count(&nest, &proposer, "the-judged-road").unwrap().is_none(), "no count while the road is open");
    assert_eq!(judge_says(&nest, &judge, "the-judged-road", &Receipts { world: Which::Fail, worth: PRICE / 4 }), Err(STILL_OPEN.to_string()), "the judge's word is not taken before the close");
    wait_until(&nest, closes + 200);
    assert!(count(&nest, &proposer, "the-judged-road").unwrap().is_none(), "no count before the judge's receipts");
    let read = read_market(&nest, "the-judged-road").unwrap().unwrap();
    assert_eq!((read.market.accepted, read.market.refused.len(), read.road.roster.len()), (3, 1, 3), "both sells and the buy were taken; the whale's trade was refused");
    assert_eq!((read.market.refused[0].seat.as_str(), read.market.refused[0].why), ("warden-whale", NOT_ON_THE_ROSTER));
    assert_eq!(read.road.terms.purse_of(read.road.terms.budget), Purse { quote: 500 * UNIT, base: 500 * UNIT }, "a deposit of the default 1000 would open with 500 and 500");
    // a and b traded PASS, c traded FAIL: each one's other branch is still what its own deposit opened
    for (k, units, traded_pass) in [(&a, 300, true), (&b, 600, true), (&c, 1500, false)] {
        let h = read.market.traders.iter().find(|h| h.key == k.key).unwrap();
        assert_eq!((h.deposit, if traded_pass { h.fail } else { h.pass }), (units * UNIT, read.road.terms.purse_of(units * UNIT)), "{} opened with its own deposit", k.seat);
    }
    let world = world_of(&read.decision).expect("the road has closed");
    let other = if world == Which::Pass { Which::Fail } else { Which::Pass };
    assert!(judge_says(&nest, &a, "the-judged-road", &Receipts { world, worth: PRICE / 4 }).is_err(), "only the road's judge says its receipts");
    assert_eq!(judge_says(&nest, &judge, "the-judged-road", &Receipts { world: other, worth: PRICE / 2 }), Err(OTHER_WORLD.to_string()), "nor a word of the world that did not happen");
    // such a word put on the log past the judge's door counts for nothing: it does not stand in the way of the receipts
    nest.append(Some(&judge), &judge.seat, RECEIPTS, receipts_value("the-judged-road", &Receipts { world: other, worth: PRICE / 2 })).unwrap();
    assert!(count(&nest, &proposer, "the-judged-road").unwrap().is_none(), "a word of the other world is no receipts");
    judge_says(&nest, &judge, "the-judged-road", &Receipts { world, worth: PRICE / 4 }).unwrap();
    let c1 = count(&nest, &proposer, "the-judged-road").unwrap().expect("after the close and the receipts the road is counted");
    let rev3 = rev("futarchy/counts/the-judged-road");
    let c2 = count(&nest, &a, "the-judged-road").unwrap().unwrap();
    assert_eq!((c2, rev("futarchy/counts/the-judged-road")), (c1.clone(), rev3), "a second count wrote nothing");
    assert_eq!(rev3, 1);
    let counts = c1["counts"].as_array().unwrap();
    let sum: i64 = counts.iter().map(|x| x["result"].as_i64().unwrap()).sum::<i64>() + c1["pool"].as_i64().unwrap();
    eprintln!("  road 3: {} on receipts of {} by its judge; {} traders, their results and the pool's sum to {sum} micro", c1["world"], c1["worth"], counts.len());
    assert_eq!((c1["world"].as_str(), c1["worth"].as_str(), c1["judge"].as_str()), (Some(if world == Which::Pass { "pass" } else { "fail" }), Some("0.250000000000"), Some(judge.key.as_str())));
    assert_eq!(counts.len(), 3);
    assert!(sum.abs() <= counts.len() as i64, "what the traders won the pool lost, to the rounding");
    // what the road gives back: each deposit and its result, never negative, and with the pool's result the deposits to the rounding
    for x in counts {
        let (deposit, result, back) = (x["deposit"].as_u64().unwrap() as i64, x["result"].as_i64().unwrap(), x["back"].as_i64().expect("a row says what comes back, never negative"));
        assert_eq!(back, deposit + result, "{}: back is the deposit and the result", x["seat"]);
        assert!(back >= 0);
    }
    let backs: i64 = counts.iter().map(|x| x["back"].as_i64().unwrap()).sum();
    let deposits: i64 = counts.iter().map(|x| x["deposit"].as_i64().unwrap()).sum();
    eprintln!("  road 3: deposits {deposits}, given back {backs}, the pool {}", c1["pool"]);
    assert_eq!(deposits, 2400 * UNIT as i64);
    assert!((backs + c1["pool"].as_i64().unwrap() - deposits).abs() <= counts.len() as i64, "what comes back and the pool's result are the deposits, to one micro a trader");
}
