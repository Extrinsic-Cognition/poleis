//! poleis-futarchy: a hand's or an agent's words to the market on the Nest.
//!   poleis-futarchy state <id>                                        the market and the decision so far, read only
//!   poleis-futarchy propose <road.json> --seat S                      lay a road (once)
//!   poleis-futarchy trade <id> <pass|fail> <buy|sell> <amount> --seat S   one trade, in $WARM to buy and in base to sell (12.5 is 12.5)
//!   poleis-futarchy settle <id> --seat S                              after the close: the decision, and the allowances when it passed
//!   poleis-futarchy receipts <id> <pass|fail> <worth> --seat S        the judge's word after the close: what a claim was worth in the world
//!                                                                        that happened, 0 to 1 $WARM with at most twelve decimals (0.25 is 0.25)
//!   poleis-futarchy count <id> --seat S                               after the receipts: each trader's result settled on them, and the pool's
//! --nest URL (else NEST, else http://127.0.0.1:8904); a seat's key is its file in NEST_KEYS (else ~/.embertide/nest), as `nest.mjs keygen` wrote it.
use poleis_futarchy::{law::*, nest::*};

fn micro(s: &str) -> Option<u128> {
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    if frac.len() > 6 || whole.is_empty() || !whole.bytes().chain(frac.bytes()).all(|c| c.is_ascii_digit()) { return None; }
    Some(whole.parse::<u128>().ok()?.checked_mul(UNIT)? + format!("{frac:0<6}").parse::<u128>().ok()?)
}

/// a worth of 0 to 1 $WARM, with at most twelve decimals, in PRICE units: exact, in integers
fn worth(s: &str) -> Option<u128> {
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    if frac.len() > 12 || whole.is_empty() || !whole.bytes().chain(frac.bytes()).all(|c| c.is_ascii_digit()) { return None; }
    Some(whole.parse::<u128>().ok()?.checked_mul(PRICE)?.checked_add(format!("{frac:0<12}").parse::<u128>().ok()?)?).filter(|w| *w <= PRICE)
}

fn run() -> Result<String, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opt = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let nest = Nest::new(&opt("--nest").or_else(|| std::env::var("NEST").ok()).unwrap_or_else(|| "http://127.0.0.1:8904".into()));
    let key = || -> Result<Key, String> {
        let seat = opt("--seat").ok_or("--seat <seat>: the trader or proposer, whose key signs")?;
        let dir = std::env::var("NEST_KEYS").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".embertide/nest"));
        Key::load(&dir, &seat)
    };
    let word = |i: usize| args.get(i).map(String::as_str).ok_or("usage: poleis-futarchy state <id> | propose <road.json> --seat S | trade <id> <pass|fail> <buy|sell> <amount> --seat S | settle <id> --seat S | receipts <id> <pass|fail> <worth> --seat S | count <id> --seat S");
    match word(0)? {
        "state" => Ok(read_market(&nest, word(1)?)?.map(|r| reading_value(&r).to_string()).unwrap_or_else(|| "null".into())),
        "propose" => {
            let v = serde_json::from_str(&std::fs::read_to_string(word(1)?).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            Ok(if propose(&nest, &key()?, v)? { "laid".into() } else { "a road of that id already stands".into() })
        }
        "trade" => {
            let t = Trade { branch: match word(2)? { "pass" => Which::Pass, "fail" => Which::Fail, _ => return Err("the branch is pass or fail".into()) }, side: match word(3)? { "buy" => Side::Buy, "sell" => Side::Sell, _ => return Err("the side is buy or sell".into()) },
                amount: micro(word(4)?).filter(|a| *a > 0).ok_or("the amount is a number with at most six decimals")?, min_out: opt("--min-out").and_then(|m| micro(&m)).unwrap_or(0) };
            Ok(trade(&nest, &key()?, word(1)?, &t)?.to_string())
        }
        "settle" => Ok(settle(&nest, &key()?, word(1)?)?.map(|v| v.to_string()).unwrap_or_else(|| "open: the road has not closed on the Nest's clock".into())),
        "receipts" => {
            let r = Receipts { world: match word(2)? { "pass" => Which::Pass, "fail" => Which::Fail, _ => return Err("the world is pass or fail".into()) }, worth: worth(word(3)?).ok_or("the worth is 0 to 1 $WARM with at most twelve decimals")? };
            Ok(judge_says(&nest, &key()?, word(1)?, &r)?.to_string())
        }
        "count" => match count(&nest, &key()?, word(1)?)? {
            Some(v) => Ok(v.to_string()),
            None if read_market(&nest, word(1)?)?.map(|r| r.decision.state == State::Open).unwrap_or(true) => Ok("open: the road has not closed on the Nest's clock".into()),
            None => Ok("no receipts: the road's judge has not said what the world that happened was worth".into()),
        },
        _ => Err(word(99).unwrap_err().into()),
    }
}

fn main() { match run() { Ok(s) => println!("{s}"), Err(e) => { eprintln!("futarchy: {e}"); std::process::exit(1); } } }
