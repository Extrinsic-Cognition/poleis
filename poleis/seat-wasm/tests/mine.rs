//! THE MINE IS LIVE (Nov 23 2026, lane/the-mine-is-live), the law's own tests (src/mine.rs): the receipts the router signs, held as the crank holds
//! them; the deposit a flask signs; the crank's plan. The receipts of tests/mine-vector.json are the router's own (app/space-router-oracle.mjs
//! receiptOf, signReceipt and canonical, under a fixed test key nobody else holds), made once, from embertide-app/ui/intro/dyson-society, by a
//! `node --input-type=module -e` of those three functions over five calls (a tank's two, a rider's of nano-dollars, one of a tenth of a micro-dollar
//! whose number JSON writes with an exponent, one that walked a trail) and kept as data; no JavaScript runs here. The plans' other receipts are
//! signed here in the same canonical shape. The live half is poleis/press/tests/mine.rs under poleis/ceremony/the-mine-is-live.sh.
use ed25519_dalek::{Signer as _, SigningKey}; // classical: the router's oracle signs with Ed25519, and so does a Solana transaction
use poleis_seat::mine::{self, Deposit, Entry, MineState, Spend};
use poleis_seat::PACKET;
use serde_json::{json, Value};
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::str::FromStr as _;
use warm_mine::state::{Flask, Mine, DISC_FLASK, DISC_MINE};

const USD: u64 = 1_000_000;
/// A day the tests' mine opened on, and its first halving six calendar months on: 2026-05-23 and 2026-11-23, 00:00 UTC.
const OPENING: u64 = 1_779_494_400_000;
const HALVING: u64 = 1_795_392_000_000;

fn vector() -> Value {
    serde_json::from_str(include_str!("mine-vector.json")).unwrap()
}
fn oracle_of(v: &Value) -> [u8; 32] {
    let h = v["oracle"].as_str().unwrap();
    core::array::from_fn(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap())
}
fn key(n: u8) -> Pubkey {
    Pubkey::new_from_array([n; 32])
}
/// The tests' own oracle, and a receipt it signs in the router's canonical shape (its members in their order, `sig` between `nonce` and `tank`).
fn oracle() -> SigningKey {
    SigningKey::from_bytes(&[42; 32])
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn b64url(b: &[u8]) -> String {
    mine::b64(b).replace('+', "-").replace('/', "_").trim_end_matches('=').to_string()
}
fn spend(id: u64, at_ms: u64, nanos: u64, flask: u8) -> Spend {
    let (k, tank) = (oracle(), format!("st-{flask:012x}"));
    let usd = nanos as f64 / 1e9;
    let unsigned = format!("{{\"at\":{at_ms},\"by\":\"{}\",\"nonce\":\"n{id}\",\"tank\":\"{tank}\",\"usd\":{usd},\"v\":1}}", hex(k.verifying_key().as_bytes()));
    let sig = b64url(&k.sign(unsigned.as_bytes()).to_bytes());
    let body = unsigned.replace(",\"tank\"", &format!(",\"sig\":\"{sig}\",\"tank\""));
    Spend { id, at_ms, nanos, tank, flasks: vec![key(flask)], body }
}
fn state(last_seq: u64, last_at_ms: u64) -> MineState {
    MineState { miner: key(200), mint: mine::mint_at(), vault: mine::vault_at(), treasury: key(201), dollar: key(202), opening_ms: OPENING, last_seq, last_at_ms }
}
fn rooms(of: &[(u8, u64, u64)]) -> BTreeMap<Pubkey, (u64, u64)> {
    of.iter().map(|(f, deposited, spent)| (key(*f), (*deposited, *spent))).collect()
}
fn by() -> [u8; 32] {
    *oracle().verifying_key().as_bytes()
}

#[test]
fn a_receipt_is_held_to_the_oracle_as_the_router_signed_it() {
    let v = vector();
    let oracle = oracle_of(&v);
    let rows: Vec<Spend> = v["receipts"].as_array().unwrap().iter().enumerate().map(|(i, r)| Spend { id: i as u64 + 1, at_ms: r["at"].as_u64().unwrap(), nanos: r["nanos"].as_u64().unwrap(), tank: r["tank"].as_str().unwrap().into(), flasks: vec![key(1)], body: r["body"].as_str().unwrap().into() }).collect();
    assert_eq!(rows.len(), 5);
    for r in &rows {
        assert!(mine::receipt_holds(r, &oracle), "the router's own receipt holds: {}", r.body);
        // fuel under the receipt's dollars holds too (a part its tank paid in $WARM); a nano over does not
        assert!(mine::receipt_holds(&Spend { nanos: r.nanos - 1, ..r.clone() }, &oracle));
        assert!(!mine::receipt_holds(&Spend { nanos: r.nanos + 1, ..r.clone() }, &oracle), "more fuel than the receipt's dollars");
        assert!(!mine::receipt_holds(&Spend { at_ms: r.at_ms + 1, ..r.clone() }, &oracle), "another instant");
        assert!(!mine::receipt_holds(&Spend { tank: "st-000000000000".into(), ..r.clone() }, &oracle), "another tank");
        assert!(!mine::receipt_holds(r, &by()), "another oracle");
        // one byte of what was signed changed: the tank's name, the same length
        let forged = r.body.replace(&format!("\"tank\":\"{}\"", r.tank), &format!("\"tank\":\"{}x\"", &r.tank[..r.tank.len() - 1]));
        assert!(!mine::receipt_holds(&Spend { tank: format!("{}x", &r.tank[..r.tank.len() - 1]), body: forged, ..r.clone() }, &oracle), "a receipt whose words were changed");
        assert!(!mine::receipt_holds(&Spend { body: r.body.replace("\"sig\":\"", "\"sig\":\"A"), ..r.clone() }, &oracle), "a signature that is not the oracle's");
    }
    // the same dollars under a `usd` raised in the text: the signature no longer covers it
    let raised = rows[0].body.replace("\"usd\":0.021", "\"usd\":21");
    assert!(!mine::receipt_holds(&Spend { nanos: 21_000_000_000, body: raised, ..rows[0].clone() }, &oracle));
    assert!(!mine::receipt_holds(&Spend { body: "not a receipt".into(), ..rows[0].clone() }, &oracle));
    // the tests' own receipts are of the same shape
    assert!(mine::receipt_holds(&spend(1, HALVING, 21_000_000, 1), &by()));
}

#[test]
fn a_deposit_through_the_mine_is_one_message_the_flask_signs() {
    let (flask, fee, source, treasury, dollar) = (key(1), key(2), key(3), key(4), key(5));
    let d = Deposit { fee: Some(fee), flask, source, treasury, dollar, amount: 10 * USD, blockhash: [9; 32], price: Some(1_000_000), commitment: None };
    let bytes = mine::deposit_message(&d).unwrap();
    assert!(1 + 2 * 64 + bytes.len() <= PACKET);
    // read back by Solana's own layout: the fee key signs first and pays, the flask second and only as a signer
    let (h, keys) = (&bytes[..3], &bytes[4..4 + 32 * bytes[3] as usize]);
    let at = |k: &Pubkey| keys.chunks(32).position(|c| c == k.as_ref()).unwrap_or_else(|| panic!("{k} is not in the message"));
    assert_eq!((h[0], h[1], at(&fee), at(&flask)), (2, 1, 0, 1), "two signers, the flask read-only");
    for k in [mine::mine_at(), mine::record_of(&flask), source, treasury, dollar, mine::WARM, mine::TOKEN, mine::MEMO] {
        at(&k);
    }
    let ix = mine::deposit_ix(&d);
    assert_eq!(ix.data, [&[warm_mine::IX_DEPOSIT][..], &(10 * USD).to_le_bytes()].concat());
    let metas: Vec<(Pubkey, bool, bool)> = ix.accounts.iter().map(|a| (a.pubkey, a.is_signer, a.is_writable)).collect();
    assert_eq!(
        metas,
        [(flask, true, false), (fee, true, true), (mine::mine_at(), false, true), (mine::record_of(&flask), false, true), (source, false, true), (treasury, false, true), (dollar, false, false), (mine::TOKEN, false, false), (Pubkey::default(), false, false)],
        "the mine's deposit, as ix.rs reads it: the flask, the payer, the mine, the record, the source, the treasury, the dollar, the token program, the system program"
    );
    // the terms' memo rides it, and the compute budget only with a price
    let text = |b: &[u8]| b.windows(mine::DEPOSIT_TERMS.len()).any(|w| w == mine::DEPOSIT_TERMS.as_bytes());
    assert!(text(&bytes));
    let plain = mine::deposit_message(&Deposit { price: None, ..d.clone() }).unwrap();
    assert!(plain.len() < bytes.len() && !plain.chunks(32).any(|c| c == poleis_seat::COMPUTE_BUDGET.as_ref()));
    let keyed = mine::deposit_message(&Deposit { commitment: Some("ab".repeat(32)), ..d.clone() }).unwrap();
    assert!(keyed.windows(64).any(|w| w == "ab".repeat(32).as_bytes()));
    // a flask that pays its own fee signs alone
    let own = mine::deposit_message(&Deposit { fee: None, ..d.clone() }).unwrap();
    assert_eq!((own[0], own[1], &own[4..36]), (1, 0, flask.as_ref()));
    // the same ask, the same bytes: the door writes them for the page to sign and again to check what it signed
    assert_eq!(mine::deposit_message(&d).unwrap(), bytes);
    // the refusals
    assert!(mine::deposit_message(&Deposit { amount: 10 * USD - 1, ..d.clone() }).is_err(), "under 10 USDC");
    assert!(mine::deposit_message(&Deposit { fee: Some(flask), ..d.clone() }).is_err(), "the fee key is not the flask");
    assert!(mine::deposit_message(&Deposit { source: treasury, ..d.clone() }).is_err(), "the treasury to itself");
    assert!(mine::deposit_message(&Deposit { treasury: mine::mine_at(), ..d.clone() }).is_err());
    assert!(mine::deposit_message(&Deposit { commitment: Some("AB".repeat(32)), ..d.clone() }).is_err());
    assert!(Message::new_with_blockhash(&[mine::deposit_ix(&d)], Some(&fee), &solana_hash::Hash::new_from_array([9; 32])).serialize().len() < bytes.len());
}

#[test]
fn the_plan_posts_each_spend_once_in_its_era_and_never_past_the_deposits() {
    let now = HALVING + 3_600_000;
    // two flasks across the halving: one entry a flask an era, by seq, each stamped by its last receipt
    let rows = vec![
        spend(1, HALVING - 5_000, 2_100_000_000, 1),
        spend(2, HALVING - 4_000, 1_050_000_000, 2),
        spend(3, HALVING - 3_000, 105_000_000, 1),
        spend(4, HALVING + 1_000, 1_050_000_000, 1),
        spend(5, HALVING + 2_000, 2_100_000_000, 2),
        spend(6, HALVING + 3_000, 21_000_000, 1),
    ];
    let both = rooms(&[(1, 20 * USD, 0), (2, 10 * USD, 0)]);
    let p = mine::plan(&state(0, 0), now, &rows, &both, &by());
    assert_eq!(
        p.entries,
        [
            Entry { flask: key(2), usd_micros: 1_050_000, at_ms: HALVING - 4_000, seq: 2 },
            Entry { flask: key(1), usd_micros: 2_205_000, at_ms: HALVING - 3_000, seq: 3 },
            Entry { flask: key(2), usd_micros: 2_100_000, at_ms: HALVING + 2_000, seq: 5 },
            Entry { flask: key(1), usd_micros: 1_071_000, at_ms: HALVING + 3_000, seq: 6 },
        ]
    );
    assert_eq!((p.through, p.passed, p.read, p.unsigned, p.held_nanos, p.dust_nanos, p.waits), (6, 0, 6, 0, 0, 0, false));
    // the law's $WARM of the batch is the router's of its burns: a dollar a $WARM before the halving, half after
    let warm: u128 = p.entries.iter().map(|e| warm_mine::law::warm_nanos(e.usd_micros, e.at_ms, OPENING)).sum();
    assert_eq!(warm, 3_255_000_000 + 3_171_000_000 / 2);
    // THE CURSOR IS THE CHAIN'S: the same rows after the batch landed (the mine's cursor at its last seq, the records moved) plan nothing
    let after = rooms(&[(1, 20 * USD, 3_276_000), (2, 10 * USD, 3_150_000)]);
    let again = mine::plan(&state(6, HALVING + 3_000), now, &rows, &after, &by());
    assert_eq!((again.entries.len(), again.read, again.through), (0, 0, 6), "a crank run twice finds nothing to post");
    // cut after its third receipt, it resumes at the fourth
    let resumed = mine::plan(&state(3, HALVING - 3_000), now, &rows, &rooms(&[(1, 20 * USD, 2_205_000), (2, 10 * USD, 1_050_000)]), &by());
    assert_eq!(resumed.entries, p.entries[2..]);

    // THE CLOCK: a receipt the chain's clock has not passed waits, and everything after it
    let early = mine::plan(&state(0, 0), HALVING - 3_500, &rows, &both, &by());
    assert_eq!((early.entries.len(), early.through, early.waits), (2, 2, true));
    assert_eq!(early.entries[1], Entry { flask: key(2), usd_micros: 1_050_000, at_ms: HALVING - 4_000, seq: 2 });
    // a spend is never stamped before the last one mined: an earlier instant takes the mine's last, and its era
    let late = mine::plan(&state(0, HALVING + 500), now, &rows[..1], &both, &by());
    assert_eq!(late.entries, [Entry { flask: key(1), usd_micros: 2_100_000, at_ms: HALVING + 500, seq: 1 }]);

    // THE BOUND: a spend is mined as far as its flask's deposits go; the part past them, and a flask with no record, mine nothing
    let tight = rooms(&[(1, 10 * USD, 8 * USD)]);
    let p = mine::plan(&state(0, 0), now, &rows, &tight, &by());
    assert_eq!(p.entries, [Entry { flask: key(1), usd_micros: 2_000_000, at_ms: HALVING - 5_000, seq: 1 }], "flask 1 had two dollars of room; flask 2 has no record");
    assert_eq!((p.through, p.read, p.held_nanos), (6, 6, 100_000_000 + 105_000_000 + 1_050_000_000 + 21_000_000 + 1_050_000_000 + 2_100_000_000));
    // a tank's second flask takes what its first has no room for
    let mut shared = spend(7, HALVING + 4_000, 500_000_000, 1);
    shared.flasks = vec![key(1), key(3)];
    let p = mine::plan(&state(6, HALVING + 3_000), now, &[shared], &rooms(&[(1, 10 * USD, 10 * USD), (3, 10 * USD, 0)]), &by());
    assert_eq!(p.entries, [Entry { flask: key(3), usd_micros: 500_000, at_ms: HALVING + 4_000, seq: 7 }]);
    // WHAT CANNOT MINE NEVER HOLDS THE LINE: the receipts that, from the first, mined nothing are said (`passed`), up to the first that took a place
    let p = mine::plan(&state(0, 0), now, &rows, &rooms(&[(1, 20 * USD, 0)]), &by());
    assert_eq!((p.passed, p.through, p.entries.len()), (0, 6, 2), "flask 1 takes the first receipt: nothing is passed, though flask 2's mine nothing");
    let p = mine::plan(&state(0, 0), now, &rows, &rooms(&[(2, 10 * USD, 10 * USD)]), &by());
    assert_eq!((p.passed, p.through, p.entries.len(), p.held_nanos), (6, 6, 0, 6_426_000_000), "nothing here can mine: all of it is passed");
    let p = mine::plan(&state(0, 0), now, &rows[1..], &rooms(&[(1, 20 * USD, 0)]), &by());
    assert_eq!((p.passed, p.entries.first().map(|e| e.seq)), (2, Some(3)), "flask 2's first receipt is passed, then flask 1 takes its place");

    // nano-dollars: an entry floors its sum to the micro-dollar; under a micro-dollar posts nothing, and the cursor does not move past it by a send
    let fine = vec![spend(1, HALVING + 1_000, 12_962_963, 1), spend(2, HALVING + 2_000, 1_037_037, 1), spend(3, HALVING + 3_000, 999, 1)];
    let p = mine::plan(&state(0, 0), now, &fine, &both, &by());
    assert_eq!((p.entries.clone(), p.dust_nanos), (vec![Entry { flask: key(1), usd_micros: 14_000, at_ms: HALVING + 3_000, seq: 3 }], 999));
    let p = mine::plan(&state(0, 0), now, &fine[2..], &both, &by());
    assert_eq!((p.entries.len(), p.through, p.dust_nanos), (0, 3, 999));

    // a receipt that does not hold mines nothing and is read past
    let mut forged = spend(2, HALVING + 2_000, 5_000_000_000, 1);
    forged.body = forged.body.replace("\"usd\":5", "\"usd\":50");
    forged.nanos = 50_000_000_000;
    let p = mine::plan(&state(0, 0), now, &[spend(1, HALVING + 1_000, 1_000_000_000, 1), forged, spend(3, HALVING + 3_000, 1_000_000_000, 1)], &both, &by());
    assert_eq!((p.entries.clone(), p.unsigned, p.through, p.passed), (vec![Entry { flask: key(1), usd_micros: 2_000_000, at_ms: HALVING + 3_000, seq: 3 }], 1, 3, 0));
}

#[test]
fn a_batch_fits_solanas_packet_and_the_rest_waits() {
    let now = HALVING + 3_600_000;
    let rows: Vec<Spend> = (1..=40u8).map(|i| spend(i as u64, HALVING + i as u64 * 1_000, 1_000_000_000, i)).collect();
    let all = rooms(&(1..=40u8).map(|i| (i, 10 * USD, 0)).collect::<Vec<_>>());
    let p = mine::plan(&state(0, 0), now, &rows, &all, &by());
    let wire = 1 + 64 + mine::mine_message(&key(200), &p.entries, &[0; 32]).len();
    assert!(p.waits && wire <= PACKET && !p.entries.is_empty(), "{} spends in {wire} bytes", p.entries.len());
    assert_eq!((p.through, p.read), (p.entries.len() as u64, p.entries.len()), "the batch ends at the last receipt it took");
    assert!(1 + 64 + mine::mine_message(&key(200), &[&p.entries[..], &[Entry { flask: key(99), usd_micros: 1, at_ms: now, seq: 99 }]].concat(), &[0; 32]).len() > PACKET, "one more flask would not fit");
    // one flask's forty spends are one entry
    let one: Vec<Spend> = (1..=40u64).map(|i| spend(i, HALVING + i * 1_000, 250_000_000, 1)).collect();
    let p = mine::plan(&state(0, 0), now, &one, &all, &by());
    assert_eq!((p.entries.clone(), p.waits), (vec![Entry { flask: key(1), usd_micros: 10 * USD, at_ms: HALVING + 40_000, seq: 40 }], false));
    // the `mine` instruction as ix.rs reads it
    let ix = mine::mine_ix(&key(200), &p.entries);
    assert_eq!(ix.data, [&[warm_mine::IX_MINE, 1][..], &(10 * USD).to_le_bytes(), &(HALVING + 40_000).to_le_bytes(), &40u64.to_le_bytes()].concat());
    let metas: Vec<(Pubkey, bool, bool)> = ix.accounts.iter().map(|a| (a.pubkey, a.is_signer, a.is_writable)).collect();
    assert_eq!(metas[..7], [(key(200), true, true), (mine::mine_at(), false, true), (mine::mint_at(), false, true), (mine::vault_at(), false, true), (mine::TOKEN, false, false), (mine::ATA, false, false), (Pubkey::default(), false, false)]);
    assert_eq!(metas[7..], [(key(1), false, false), (mine::record_of(&key(1)), false, true), (mine::warm_account_of(&key(1)), false, true)]);
}

#[test]
fn the_one_door_speaks_json_as_the_workers_glue_does() {
    let ask = |v: Value| -> Value { serde_json::from_str(&mine::ask(&v.to_string())).unwrap() };
    let terms = ask(json!({ "do": "terms" }));
    assert_eq!((terms["program"].as_str(), terms["mine"].as_str(), terms["clock"].as_str()), (Some(mine::WARM.to_string().as_str()), Some(mine::mine_at().to_string().as_str()), Some("SysvarC1ock11111111111111111111111111111111")));
    // the deposit, its fields as the door names them
    let d = ask(json!({ "do": "deposit", "flask": key(1).to_string(), "fee": key(2).to_string(), "source": key(3).to_string(), "treasury": key(4).to_string(), "dollar": key(5).to_string(), "amount": "10000000", "blockhash": key(9).to_string(), "price": 1_000_000 }));
    let want = mine::deposit_message(&Deposit { fee: Some(key(2)), flask: key(1), source: key(3), treasury: key(4), dollar: key(5), amount: 10 * USD, blockhash: [9; 32], price: Some(1_000_000), commitment: None }).unwrap();
    assert_eq!((mine::unb64(d["message"].as_str().unwrap()).unwrap(), d["record"].as_str()), (want, Some(mine::record_of(&key(1)).to_string().as_str())));
    assert!(ask(json!({ "do": "deposit", "flask": key(1).to_string(), "source": key(3).to_string(), "treasury": key(4).to_string(), "dollar": key(5).to_string(), "amount": "9999999", "blockhash": key(9).to_string() }))["error"].is_string());
    assert!(ask(json!({ "do": "nothing" }))["error"].is_string() && serde_json::from_str::<Value>(&mine::ask("not json")).unwrap()["error"].is_string());

    // the mine and the clock as a node answers them, a flask's record, a page of rows: the cursor, the records wanted, the plan
    let mut m = vec![0u8; Mine::SIZE];
    (m[0], m[1]) = (DISC_MINE, 1);
    m[Mine::OFF_MINER..Mine::OFF_MINER + 32].copy_from_slice(key(200).as_ref());
    m[Mine::OFF_OPENING..Mine::OFF_OPENING + 8].copy_from_slice(&OPENING.to_le_bytes());
    m[Mine::OFF_LAST_SEQ..Mine::OFF_LAST_SEQ + 8].copy_from_slice(&7u64.to_le_bytes());
    m[Mine::OFF_LAST_AT..Mine::OFF_LAST_AT + 8].copy_from_slice(&(HALVING - 1).to_le_bytes());
    let mut clock = vec![0u8; 40];
    clock[32..].copy_from_slice(&((HALVING / 1_000 + 60) as i64).to_le_bytes());
    let node = |d: &[u8]| json!({ "data": [mine::b64(d), "base64"], "lamports": 1, "owner": mine::WARM.to_string() });
    let cur = ask(json!({ "do": "cursor", "mine": node(&m), "clock": node(&clock) }));
    assert_eq!((cur["seq"].as_u64(), cur["at"].as_u64(), cur["clock_ms"].as_u64(), cur["miner"].as_str(), cur["opening"].as_u64()), (Some(7), Some(HALVING - 1), Some(HALVING + 60_000), Some(key(200).to_string().as_str()), Some(OPENING)));
    assert!(ask(json!({ "do": "cursor", "mine": Value::Null, "clock": node(&clock) }))["error"].is_string(), "no mine, no cursor");
    let s = spend(8, HALVING + 1_000, 2_100_000_000, 1);
    let rows = json!([{ "id": 8, "at": s.at_ms, "nanos": s.nanos, "tank": s.tank, "flasks": format!("{},{}", key(1), key(3)), "body": s.body }]);
    let wants = ask(json!({ "do": "wants", "rows": rows }));
    assert_eq!(wants["records"], json!({ mine::record_of(&key(1)).to_string(): key(1).to_string(), mine::record_of(&key(3)).to_string(): key(3).to_string() }));
    let mut r = vec![0u8; Flask::SIZE];
    (r[0], r[1]) = (DISC_FLASK, 1);
    r[Flask::OFF_FLASK..Flask::OFF_FLASK + 32].copy_from_slice(key(1).as_ref());
    r[Flask::OFF_DEPOSITED..Flask::OFF_DEPOSITED + 8].copy_from_slice(&(10 * USD).to_le_bytes());
    let plan = ask(json!({ "do": "plan", "mine": node(&m), "clock": node(&clock), "rows": rows, "records": { mine::record_of(&key(1)).to_string(): node(&r), mine::record_of(&key(3)).to_string(): Value::Null },
        "oracle": hex(&by()), "blockhash": key(9).to_string() }));
    let entry = [Entry { flask: key(1), usd_micros: 2_100_000, at_ms: HALVING + 1_000, seq: 8 }];
    assert_eq!((mine::unb64(plan["message"].as_str().unwrap()).unwrap(), plan["spends"].as_u64(), plan["usd_micros"].as_u64(), plan["through"].as_u64(), plan["waits"].as_bool()), (mine::mine_message(&key(200), &entry, &[9; 32]), Some(1), Some(2_100_000), Some(8), Some(false)));
    // another flask's record under this flask's address is no room; another oracle's receipt is no spend
    let mut other = r.clone();
    other[Flask::OFF_FLASK..Flask::OFF_FLASK + 32].copy_from_slice(key(3).as_ref());
    let none = ask(json!({ "do": "plan", "mine": node(&m), "clock": node(&clock), "rows": rows, "records": { mine::record_of(&key(1)).to_string(): node(&other) }, "oracle": hex(&by()), "blockhash": key(9).to_string() }));
    assert_eq!((none["message"].is_null(), none["held_nanos"].as_u64(), none["passed"].as_u64()), (true, Some(2_100_000_000), Some(8)));
    let unsigned = ask(json!({ "do": "plan", "mine": node(&m), "clock": node(&clock), "rows": rows, "records": { mine::record_of(&key(1)).to_string(): node(&r) }, "oracle": "00".repeat(32), "blockhash": key(9).to_string() }));
    assert_eq!((unsigned["message"].is_null(), unsigned["unsigned"].as_u64()), (true, Some(1)));
    assert!(Pubkey::from_str(terms["mint"].as_str().unwrap()).is_ok());
}

#[test]
fn spent_warm_goes_back_to_the_mine_by_one_message_the_flask_signs() {
    // his press of Nov 23 2026 18:17:41 UTC, "Back to the mine": the flask pays its $WARM back ahead, the house's fee key paying
    let (flask, fee) = (key(1), key(2));
    let b = mine::PayBack { fee: Some(fee), flask, amount: 2_000_000_000, blockhash: [9; 32], price: Some(1_000_000) };
    let bytes = mine::pay_back_message(&b).unwrap();
    assert!(1 + 2 * 64 + bytes.len() <= PACKET);
    let (h, keys) = (&bytes[..3], &bytes[4..4 + 32 * bytes[3] as usize]);
    let at = |k: &Pubkey| keys.chunks(32).position(|c| c == k.as_ref()).unwrap_or_else(|| panic!("{k} is not in the message"));
    assert_eq!((h[0], h[1], at(&fee), at(&flask)), (2, 1, 0, 1), "two signers, the fee key first and paying, the flask read-only");
    for k in [mine::mine_at(), mine::record_of(&flask), mine::warm_account_of(&flask), mine::vault_at(), mine::mint_at(), mine::WARM, mine::TOKEN] {
        at(&k);
    }
    let ix = mine::pay_back_ix(&b);
    assert_eq!(ix.data, [&[warm_mine::IX_PAY_BACK][..], &2_000_000_000u64.to_le_bytes()].concat());
    let metas: Vec<(Pubkey, bool, bool)> = ix.accounts.iter().map(|a| (a.pubkey, a.is_signer, a.is_writable)).collect();
    assert_eq!(
        metas,
        [(flask, true, false), (fee, true, true), (mine::mine_at(), false, true), (mine::record_of(&flask), false, true), (mine::warm_account_of(&flask), false, true), (mine::vault_at(), false, true), (mine::mint_at(), false, false), (mine::TOKEN, false, false), (Pubkey::default(), false, false)],
        "the mine's pay_back, as ix.rs reads it: the flask, the payer, the mine, the record, the flask's own $WARM, the vault, the mint, the token program, the system program"
    );
    // no memo (the count is the record's, not a text's), the compute budget only with a price; the same ask, the same bytes
    assert!(!keys.chunks(32).any(|c| c == mine::MEMO.as_ref()));
    let plain = mine::pay_back_message(&mine::PayBack { price: None, ..b.clone() }).unwrap();
    assert!(plain.len() < bytes.len() && !plain.chunks(32).any(|c| c == poleis_seat::COMPUTE_BUDGET.as_ref()));
    assert_eq!(mine::pay_back_message(&b).unwrap(), bytes);
    let own = mine::pay_back_message(&mine::PayBack { fee: None, ..b.clone() }).unwrap();
    assert_eq!((own[0], own[1], &own[4..36]), (1, 0, flask.as_ref()), "a flask that pays its own fee signs alone");
    assert!(mine::pay_back_message(&mine::PayBack { amount: 0, ..b.clone() }).is_err(), "nothing to pay back");
    assert!(mine::pay_back_message(&mine::PayBack { fee: Some(flask), ..b.clone() }).is_err(), "the fee key is not the flask");
    assert!(mine::pay_back_message(&mine::PayBack { flask: mine::vault_at(), ..b.clone() }).is_err(), "the vault pays nothing back to itself");

    // the record's count: what the flask paid back, its own record only
    let mut r = vec![0u8; Flask::SIZE];
    (r[0], r[1]) = (DISC_FLASK, warm_mine::state::LAYOUT_VERSION);
    r[Flask::OFF_FLASK..Flask::OFF_FLASK + 32].copy_from_slice(flask.as_ref());
    r[Flask::OFF_PAID_BACK..Flask::OFF_PAID_BACK + 8].copy_from_slice(&2_000_000_000u64.to_le_bytes());
    assert_eq!((mine::paid_back_of(&r, &flask), mine::paid_back_of(&r, &key(3)), mine::paid_back_of(&r[..Flask::SIZE - 1], &flask)), (Some(2_000_000_000), None, None));
    // the one door: the pay-back's ask (the watch writes it, the relay writes it again from the fields the flask signed over), the count's (the
    // record's and the $WARM account's addresses, then what the record says the flask paid back)
    let ask = |v: Value| -> Value { serde_json::from_str(&mine::ask(&v.to_string())).unwrap() };
    let node = |d: &[u8], owner: &Pubkey| json!({ "data": [mine::b64(d), "base64"], "lamports": 1, "owner": owner.to_string() });
    let back = ask(json!({ "do": "back", "flask": flask.to_string(), "fee": fee.to_string(), "amount": "2000000000", "blockhash": key(9).to_string(), "price": 1_000_000 }));
    assert_eq!((mine::unb64(back["message"].as_str().unwrap()).unwrap(), back["amount"].as_u64(), back["source"].as_str(), back["record"].as_str()), (bytes, Some(2_000_000_000), Some(mine::warm_account_of(&flask).to_string().as_str()), Some(mine::record_of(&flask).to_string().as_str())));
    assert!(ask(json!({ "do": "back", "flask": flask.to_string(), "fee": fee.to_string(), "amount": "0", "blockhash": key(9).to_string() }))["error"].is_string(), "nothing to pay back");
    assert!(ask(json!({ "do": "back", "flask": flask.to_string(), "blockhash": key(9).to_string() }))["error"].is_string(), "no amount");
    let paid = ask(json!({ "do": "paid", "flask": flask.to_string() }));
    assert_eq!((paid["record"].as_str(), paid["source"].as_str(), paid["paid"].is_null()), (Some(mine::record_of(&flask).to_string().as_str()), Some(mine::warm_account_of(&flask).to_string().as_str()), true));
    assert_eq!(ask(json!({ "do": "paid", "flask": flask.to_string(), "account": node(&r, &mine::WARM) }))["paid"].as_u64(), Some(2_000_000_000));
    assert_eq!(ask(json!({ "do": "paid", "flask": flask.to_string(), "account": Value::Null }))["paid"].as_u64(), Some(0), "no record: nothing paid back");
    assert!(ask(json!({ "do": "paid", "flask": flask.to_string(), "account": node(&r, &mine::TOKEN) }))["error"].is_string(), "a record no program of the mine's owns");
    assert!(ask(json!({ "do": "paid", "flask": key(3).to_string(), "account": node(&r, &mine::WARM) }))["error"].is_string(), "another flask's record");
}

#[test]
fn spent_as_credits_it_returns_by_one_approval_and_the_cranks_take_back() {
    let (flask, fee, miner) = (key(1), key(2), key(200));
    // the approval: the token program's ApproveChecked of the flask's own $WARM account to the mine, for all of it; it moves nothing
    let a = mine::Allow { fee: Some(fee), flask, blockhash: [9; 32], price: Some(1_000_000) };
    let bytes = mine::allow_message(&a).unwrap();
    assert!(1 + 2 * 64 + bytes.len() <= PACKET);
    let (h, keys) = (&bytes[..3], &bytes[4..4 + 32 * bytes[3] as usize]);
    let at = |k: &Pubkey| keys.chunks(32).position(|c| c == k.as_ref()).unwrap_or_else(|| panic!("{k} is not in the message"));
    assert_eq!((h[0], at(&fee), at(&flask)), (2, 0, 1), "the fee key pays, the flask signs as the owner of its $WARM");
    let ix = mine::allow_ix(&a);
    let metas: Vec<(Pubkey, bool, bool)> = ix.accounts.iter().map(|m| (m.pubkey, m.is_signer, m.is_writable)).collect();
    assert_eq!((ix.program_id, ix.data.clone()), (mine::TOKEN, warm_mine::token::approve_checked_data(u64::MAX, 9).to_vec()));
    assert_eq!(metas, [(mine::warm_account_of(&flask), false, true), (mine::mint_at(), false, false), (mine::mine_at(), false, false), (flask, true, false)]);
    assert!(mine::allow_message(&mine::Allow { fee: Some(flask), ..a.clone() }).is_err(), "the fee key is not the flask");
    assert_eq!(mine::allow_message(&a).unwrap(), bytes, "the same ask, the same bytes");

    // what a flask holds for the mine: of the mine's mint, its own, the mine its delegate; as far as its balance and its delegation go
    let token = |owner: &Pubkey, amount: u64, delegate: Option<Pubkey>, delegated: u64| {
        let mut t = vec![0u8; 165];
        t[..32].copy_from_slice(mine::mint_at().as_ref());
        t[32..64].copy_from_slice(owner.as_ref());
        t[64..72].copy_from_slice(&amount.to_le_bytes());
        if let Some(d) = delegate {
            t[72] = 1;
            t[76..108].copy_from_slice(d.as_ref());
        }
        t[108] = 1;
        t[121..129].copy_from_slice(&delegated.to_le_bytes());
        t
    };
    assert_eq!(mine::held_for_mine(&token(&flask, 7_100_000_000, Some(mine::mine_at()), u64::MAX), &flask), 7_100_000_000, "all of it, approved for ever");
    assert_eq!(mine::held_for_mine(&token(&flask, 7_100_000_000, Some(mine::mine_at()), 500_000_000), &flask), 500_000_000, "no more than its delegation");
    assert_eq!(mine::held_for_mine(&token(&flask, 7_100_000_000, Some(key(7)), u64::MAX), &flask), 0, "another delegate: nothing for the mine");
    assert_eq!(mine::held_for_mine(&token(&flask, 7_100_000_000, None, 0), &flask), 0, "no approval: nothing");
    assert_eq!(mine::held_for_mine(&token(&key(3), 7_100_000_000, Some(mine::mine_at()), u64::MAX), &flask), 0, "another holder's account");

    // the take-back: a target a record, the miner its payer
    let ix = mine::take_back_ix(&miner, &[(flask, 4_200_000_000)]);
    assert_eq!(ix.data, [&[warm_mine::IX_TAKE_BACK, 1][..], &4_200_000_000u64.to_le_bytes()].concat());
    let metas: Vec<(Pubkey, bool, bool)> = ix.accounts.iter().map(|m| (m.pubkey, m.is_signer, m.is_writable)).collect();
    assert_eq!(metas, [(miner, true, true), (mine::mine_at(), false, true), (mine::vault_at(), false, true), (mine::mint_at(), false, false), (mine::TOKEN, false, false), (mine::record_of(&flask), false, true), (mine::warm_account_of(&flask), false, true)]);

    // the plan: what a tank drank past what its flasks paid back, from its flasks in order, each as far as it holds, each flask once
    let owing = |drunk: u64, flasks: &[u8]| mine::Owing { drunk, flasks: flasks.iter().map(|n| key(*n)).collect() };
    let chain: BTreeMap<Pubkey, (u64, u64)> = [(key(1), (1_000_000_000, 2_000_000_000)), (key(3), (0, 5_000_000_000)), (key(4), (3_000_000_000, 9_000_000_000))].into_iter().collect();
    assert_eq!(mine::takes(&[owing(4_200_000_000, &[1, 3])], &chain, &miner), vec![(key(1), 3_000_000_000), (key(3), 1_200_000_000)], "3.2 owed: 2 from the first flask, 1.2 from the next");
    assert_eq!(mine::takes(&[owing(2_500_000_000, &[4])], &chain, &miner), vec![], "paid back past what it drank: nothing owed");
    assert_eq!(mine::takes(&[owing(9_999_000_000_000, &[3]), owing(4_200_000_000, &[3, 1])], &chain, &miner), vec![(key(3), 5_000_000_000), (key(1), 3_000_000_000)], "never past what a flask holds, and a flask once a batch");
    let many: Vec<mine::Owing> = (10..60u8).map(|n| owing(WARM_OF, &[n])).collect();
    let rich: BTreeMap<Pubkey, (u64, u64)> = (10..60u8).map(|n| (key(n), (0, WARM_OF))).collect();
    let full = mine::takes(&many, &rich, &miner);
    assert!(full.len() > 8 && full.len() < 50 && 1 + 64 + mine::take_back_message(&miner, &full, &[0; 32]).len() <= PACKET, "a batch is one packet: {} flasks", full.len());

    // the one door
    let ask = |v: Value| -> Value { serde_json::from_str(&mine::ask(&v.to_string())).unwrap() };
    let node = |d: &[u8], owner: &Pubkey| json!({ "data": [mine::b64(d), "base64"], "lamports": 1, "owner": owner.to_string() });
    let allow = ask(json!({ "do": "allow", "flask": flask.to_string(), "fee": fee.to_string(), "blockhash": key(9).to_string(), "price": 1_000_000 }));
    assert_eq!((mine::unb64(allow["message"].as_str().unwrap()).unwrap(), allow["source"].as_str()), (bytes, Some(mine::warm_account_of(&flask).to_string().as_str())));
    let held = ask(json!({ "do": "paid", "flask": flask.to_string(), "account": Value::Null, "token": node(&token(&flask, 7_100_000_000, Some(mine::mine_at()), u64::MAX), &mine::TOKEN) }));
    assert_eq!((held["paid"].as_u64(), held["held"].as_u64()), (Some(0), Some(7_100_000_000)));
    let rows = json!([{ "tank": "st-000000000001", "drunk": 4_200_000_000u64, "flasks": format!("{},{}", key(1), key(3)) }]);
    let read = ask(json!({ "do": "owing", "rows": rows }));
    assert_eq!(read["read"], json!([mine::record_of(&key(1)).to_string(), mine::warm_account_of(&key(1)).to_string(), mine::record_of(&key(3)).to_string(), mine::warm_account_of(&key(3)).to_string()]));
    let mut r1 = vec![0u8; Flask::SIZE];
    (r1[0], r1[1]) = (DISC_FLASK, warm_mine::state::LAYOUT_VERSION);
    r1[Flask::OFF_FLASK..Flask::OFF_FLASK + 32].copy_from_slice(key(1).as_ref());
    r1[Flask::OFF_PAID_BACK..Flask::OFF_PAID_BACK + 8].copy_from_slice(&1_000_000_000u64.to_le_bytes());
    let accounts = json!([node(&r1, &mine::WARM), node(&token(&key(1), 2_000_000_000, Some(mine::mine_at()), u64::MAX), &mine::TOKEN), Value::Null, node(&token(&key(3), 5_000_000_000, Some(mine::mine_at()), u64::MAX), &mine::TOKEN)]);
    let plan = ask(json!({ "do": "takes", "rows": rows, "accounts": accounts, "miner": miner.to_string(), "blockhash": key(9).to_string() }));
    let want = [(key(1), 3_000_000_000), (key(3), 1_200_000_000)];
    assert_eq!((mine::unb64(plan["message"].as_str().unwrap()).unwrap(), plan["takes"].as_u64(), plan["nanos"].as_u64()), (mine::take_back_message(&miner, &want, &[9; 32]), Some(2), Some(3_200_000_000)));
    assert_eq!(plan["flasks"], json!([{ "flask": key(1).to_string(), "paid": 1_000_000_000u64, "held": 2_000_000_000u64 }, { "flask": key(3).to_string(), "paid": 0, "held": 5_000_000_000u64 }]));
    let none = ask(json!({ "do": "takes", "rows": json!([{ "tank": "t", "drunk": 0, "flasks": key(1).to_string() }]), "accounts": json!([node(&r1, &mine::WARM), Value::Null]), "miner": miner.to_string(), "blockhash": key(9).to_string() }));
    assert_eq!((none["message"].is_null(), none["takes"].as_u64()), (true, Some(0)), "nothing drunk, nothing taken");
}
/// A $WARM, in nanos.
const WARM_OF: u64 = 1_000_000_000;

#[test]
fn the_doors_wasm_is_this_crate() {
    // the door's file (worker/poleis-seat.wasm, written by build.sh) exports the mine's three doors beside the seat's: read from its export
    // section's names, so a WASM built before this lane is told here and not at his deploy
    let wasm = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../embertide-app/ui/intro/dyson-society/worker/poleis-seat.wasm")).unwrap();
    for name in ["mine_in", "mine_ask", "mine_out", "seat_offer", "seat_program"] {
        assert!(wasm.windows(name.len()).any(|w| w == name.as_bytes()), "the door's WASM exports {name}");
    }
}

// ---- THE ROUTER DRAWS ITS MONTH (Nov 24 2026; his 00:07:00 UTC "Yes, build it" to "Build the router's own draw from the treasury next?", after his
// Nov 23 22:20:46 UTC "we might get some money in the treasury, but unless we figure out how to allocate it as credits.. u get me?") ----
use poleis_treasury::state::{Allowance, Registry, DISC_ALLOWANCE, DISC_REGISTRY, LAYOUT_VERSION};

/// The slot the month tests read the chain at.
const SLOT: u64 = 50_000;
/// A receipt of the router's canonical shape that names what the provider charged (`router_usd`, its JSON as given: a number, null, a string).
fn charged(id: u64, at_ms: u64, nanos: u64, flask: u8, router: &str) -> Spend {
    let (k, tank) = (oracle(), format!("st-{flask:012x}"));
    let usd = nanos as f64 / 1e9;
    let unsigned = format!("{{\"at\":{at_ms},\"by\":\"{}\",\"nonce\":\"n{id}\",\"router_usd\":{router},\"tank\":\"{tank}\",\"usd\":{usd},\"v\":1}}", hex(k.verifying_key().as_bytes()));
    let sig = b64url(&k.sign(unsigned.as_bytes()).to_bytes());
    let body = unsigned.replace(",\"tank\"", &format!(",\"sig\":\"{sig}\",\"tank\""));
    Spend { id, at_ms, nanos, tank, flasks: vec![key(flask)], body }
}
/// The mine of the month tests: its treasury root 0's own vault of its dollar, as the launch press opens it.
fn drawing(last_seq: u64, last_at_ms: u64) -> MineState {
    MineState { treasury: poleis_client::vault_pda(0, &key(202)), ..state(last_seq, last_at_ms) }
}
/// The month's four accounts as the chain would hold them, laid by the program's own offsets, and what each test moves in them.
#[derive(Clone)]
struct Chain {
    hash: [u8; 32],
    unit: u64,
    payee: Pubkey,
    payee_kind: u8,
    mint: Pubkey,
    cap: u64,
    period: u64,
    start: u64,
    spent: u64,
    expires: u64,
    frozen: bool,
    default_mint: Pubkey,
    token_cap: u64,
    token_spent: u64,
    plan_start: u64,
    plan_period: u64,
    vault: Option<(Pubkey, Pubkey, u64)>,
    dest: Option<(Pubkey, Pubkey)>,
    lamports: Option<u64>,
}
fn chain() -> Chain {
    Chain {
        hash: [8; 32],
        unit: 0,
        payee: key(200),
        payee_kind: poleis_client::HOLDER_KEY,
        mint: key(202),
        cap: 100 * USD,
        period: 10_000,
        start: SLOT - 1_000,
        spent: 0,
        expires: 0,
        frozen: false,
        default_mint: key(202),
        token_cap: 1_000 * USD,
        token_spent: 0,
        plan_start: SLOT - 1_000,
        plan_period: 10_000,
        vault: Some((key(202), poleis_client::treasury_pda(0), 500 * USD)),
        dest: None,
        lamports: Some(mine::MINER_FLOOR_LAMPORTS),
    }
}
fn token(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut t = vec![0u8; 165];
    t[..32].copy_from_slice(mint.as_ref());
    t[32..64].copy_from_slice(owner.as_ref());
    t[64..72].copy_from_slice(&amount.to_le_bytes());
    t[108] = 1;
    t
}
impl Chain {
    fn month(&self) -> Pubkey {
        poleis_client::allowance_pda(&self.hash)
    }
    fn allowance(&self) -> Vec<u8> {
        let mut d = vec![0u8; Allowance::SIZE];
        (d[0], d[1]) = (DISC_ALLOWANCE, LAYOUT_VERSION);
        d[Allowance::OFF_HASH..Allowance::OFF_HASH + 32].copy_from_slice(&self.hash);
        d[Allowance::OFF_UNIT..Allowance::OFF_UNIT + 8].copy_from_slice(&self.unit.to_le_bytes());
        d[Allowance::OFF_PAYEE..Allowance::OFF_PAYEE + 32].copy_from_slice(self.payee.as_ref());
        d[Allowance::OFF_PAYEE_KIND] = self.payee_kind;
        d[Allowance::OFF_MINT..Allowance::OFF_MINT + 32].copy_from_slice(self.mint.as_ref());
        for (o, n) in [(Allowance::OFF_CAP, self.cap), (Allowance::OFF_PERIOD, self.period), (Allowance::OFF_START, self.start), (Allowance::OFF_SPENT, self.spent), (Allowance::OFF_EXPIRES, self.expires)] {
            d[o..o + 8].copy_from_slice(&n.to_le_bytes());
        }
        d
    }
    fn registry(&self) -> Vec<u8> {
        let mut d = vec![0u8; Registry::MIN_SIZE];
        (d[0], d[1]) = (DISC_REGISTRY, LAYOUT_VERSION);
        d[Registry::OFF_FROZEN] = self.frozen as u8;
        d[Registry::OFF_DEFAULT_MINT..Registry::OFF_DEFAULT_MINT + 32].copy_from_slice(self.default_mint.as_ref());
        for (o, n) in [(Registry::OFF_UNIT, self.unit), (Registry::OFF_TOKEN_PLAN_CAP, self.token_cap), (Registry::OFF_TOKEN_PLAN_SPENT, self.token_spent), (Registry::OFF_PLAN_START, self.plan_start), (Registry::OFF_PLAN_PERIOD, self.plan_period)] {
            d[o..o + 8].copy_from_slice(&n.to_le_bytes());
        }
        d
    }
    fn of(&self, m: &MineState, slot: u64) -> Result<mine::Month, &'static str> {
        let (a, r, v, t) = (self.allowance(), self.registry(), self.vault.map(|(mint, owner, n)| token(&mint, &owner, n)), self.dest.map(|(mint, owner)| token(&mint, &owner, 0)));
        mine::month_of(m, slot, &self.month(), &mine::MonthAccounts { allowance: Some(&a), registry: Some(&r), vault: v.as_deref(), dest: t.as_deref(), miner_lamports: self.lamports })
    }
}
/// The instructions of a legacy message read back by Solana's own layout (compact lengths under 128, as a batch's are): its signers, and each
/// instruction's program and data.
fn instructions(b: &[u8]) -> (u8, Vec<(Pubkey, Vec<u8>)>) {
    let n = b[3] as usize;
    let keys: Vec<Pubkey> = (0..n).map(|i| Pubkey::new_from_array(b[4 + 32 * i..36 + 32 * i].try_into().unwrap())).collect();
    let mut at = 4 + 32 * n + 32;
    let mut ixs = Vec::new();
    let count = b[at];
    at += 1;
    for _ in 0..count {
        let (program, accounts) = (b[at] as usize, b[at + 1] as usize);
        let len = b[at + 2 + accounts] as usize;
        ixs.push((keys[program], b[at + 3 + accounts..at + 3 + accounts + len].to_vec()));
        at += 3 + accounts + len;
    }
    assert_eq!(at, b.len(), "the whole message read");
    (b[0], ixs)
}

#[test]
fn the_draw_is_what_the_provider_charged_for_the_spends_the_batch_mines() {
    let now = HALVING + 3_600_000;
    let roomy = rooms(&[(1, 100 * USD, 0), (2, 100 * USD, 0)]);
    // the provider's charge below what the tank paid (the door's markup stays), equal to it, above it (never more than the spend gives), none
    let rows = vec![
        charged(1, HALVING + 1_000, 2_100_000_000, 1, "2"),
        charged(2, HALVING + 2_000, 1_000_000_000, 1, "1"),
        charged(3, HALVING + 3_000, 500_000_000, 2, "0.9"),
        charged(4, HALVING + 4_000, 700_000_000, 2, "null"),
    ];
    let p = mine::plan(&drawing(0, 0), now, &rows, &roomy, &by());
    assert_eq!(p.router_nanos, 2_000_000_000 + 1_000_000_000 + 500_000_000, "below, equal, above (cut to the spend's 0.5), null (nothing)");
    // a number that is not one of dollars no less than 0 draws nothing; the receipt still mines
    for bad in ["-1", "\"2\"", "true", "{}"] {
        let p = mine::plan(&drawing(0, 0), now, &[charged(1, HALVING + 1_000, 1_000_000_000, 1, bad)], &roomy, &by());
        assert_eq!((p.entries.len(), p.router_nanos), (1, 0), "router_usd {bad}");
    }
    assert_eq!((mine::router_nanos_of(&rows[0].body), mine::router_nanos_of("{}"), mine::router_nanos_of("not json")), (2_000_000_000, 0, 0));
    // a spend cut by its flask's room draws as far as it mines: flask 1 has 1.5 dollars of room for a spend of 2.1 the provider charged 2 for
    let p = mine::plan(&drawing(0, 0), now, &rows[..1], &rooms(&[(1, 10 * USD, 8_500_000)]), &by());
    assert_eq!((p.entries[0].usd_micros, p.router_nanos), (1_500_000, 1_500_000_000));
    // a spend no flask has room for, and one under a micro-dollar of an entry, draw nothing
    let p = mine::plan(&drawing(0, 0), now, &[charged(1, HALVING + 1_000, 1_000_000_000, 3, "1"), charged(2, HALVING + 2_000, 999, 1, "0.000000999")], &roomy, &by());
    assert_eq!((p.entries.len(), p.router_nanos), (0, 0));
    // the sum floored to the micro-dollar: 0.0123456789 and 0.0000005 make 12,346 micro-dollars
    let fine = [charged(1, HALVING + 1_000, 12_962_963, 1, "0.0123456789"), charged(2, HALVING + 2_000, 1_037_037, 2, "0.0000005")];
    let p = mine::plan(&drawing(0, 0), now, &fine, &roomy, &by());
    let month = chain().of(&drawing(0, 0), SLOT).unwrap();
    assert_eq!((p.router_nanos, mine::draw_of(&p, &month)), (12_345_679 + 500, Ok(12_346)));
    // a plan with no entries sends no draw
    assert_eq!(mine::draw_of(&mine::plan(&drawing(0, 0), now, &[], &roomy, &by()), &month), Err("nothing"));
    let p = mine::plan(&drawing(0, 0), now, &rows[3..], &roomy, &by());
    assert_eq!((p.entries.len(), mine::draw_of(&p, &month)), (1, Err("nothing")), "spends the provider named no charge for draw nothing");
}

#[test]
fn the_draw_is_cut_to_the_month_the_cap_and_the_vault_and_each_rolls_its_own_period() {
    let m = drawing(0, 0);
    let now = HALVING + 3_600_000;
    let p = mine::plan(&m, now, &[charged(1, HALVING + 1_000, 9_000_000_000, 1, "8")], &rooms(&[(1, 100 * USD, 0)]), &by());
    let draw = |c: &Chain, slot: u64| c.of(&m, slot).and_then(|mo| mine::draw_of(&p, &mo));
    assert_eq!(draw(&chain(), SLOT), Ok(8 * USD), "all of it: the month, the cap and the vault have room");
    // each cut: what the month has left, what root 0's cap of the dollar has left, what its vault holds
    assert_eq!(draw(&Chain { spent: 97 * USD, ..chain() }, SLOT), Ok(3 * USD), "the month's three dollars left");
    assert_eq!(draw(&Chain { token_spent: 995 * USD, ..chain() }, SLOT), Ok(5 * USD), "the cap's five");
    assert_eq!(draw(&Chain { vault: Some((key(202), poleis_client::treasury_pda(0), 2 * USD)), ..chain() }, SLOT), Ok(2 * USD), "the vault's two");
    // the allowance's own period: spent whole in this one, whole again once the slot passes its start and its period (as `pull` rolls it)
    let spent = Chain { spent: 100 * USD, ..chain() };
    assert_eq!(draw(&spent, SLOT), Err("spent"));
    assert_eq!(draw(&spent, spent.start + spent.period - 1), Err("spent"));
    assert_eq!(draw(&spent, spent.start + spent.period), Ok(8 * USD), "a new month");
    assert_eq!(draw(&spent, spent.start + 7 * spent.period + 3), Ok(8 * USD), "some months on");
    // root 0's cap of the dollar: its own period, the registry's plan's (`charge_token_plan`); a plan of no period never rolls
    let capped = Chain { token_spent: 1_000 * USD, ..chain() };
    assert_eq!(draw(&capped, SLOT), Err("cap"));
    assert_eq!(draw(&capped, capped.plan_start + capped.plan_period - 1), Err("cap"));
    assert_eq!(draw(&capped, capped.plan_start + capped.plan_period), Ok(8 * USD), "a new period of the plan");
    assert_eq!(draw(&Chain { plan_period: 0, ..capped.clone() }, SLOT + 10_000_000), Err("cap"), "a plan of no period never rolls");
    assert_eq!(draw(&Chain { token_cap: 4 * USD, token_spent: 9 * USD, ..chain() }, SLOT), Err("cap"), "spent past its cap (a cap lowered under what was spent)");
    assert_eq!(draw(&Chain { vault: Some((key(202), poleis_client::treasury_pda(0), 0)), ..chain() }, SLOT), Err("empty"));
    let month = chain().of(&m, SLOT).unwrap();
    assert_eq!((month.month_left, month.cap_left, month.vault_holds, month.left()), (100 * USD, 1_000 * USD, 500 * USD, 100 * USD));
}

#[test]
fn the_draw_is_never_more_than_what_the_batch_mines() {
    // two flasks' entries of 1,500 nanos each the provider charged in full: each entry posts one micro-dollar, their shares sum to three
    let now = HALVING + 3_600_000;
    let p = mine::plan(&drawing(0, 0), now, &[charged(1, HALVING + 1_000, 1_500, 1, "0.0000015"), charged(2, HALVING + 2_000, 1_500, 2, "0.0000015")], &rooms(&[(1, 10 * USD, 0), (2, 10 * USD, 0)]), &by());
    assert_eq!((p.entries.iter().map(|e| e.usd_micros).collect::<Vec<_>>(), p.router_nanos), (vec![1, 1], 3_000));
    assert_eq!(mine::draw_of(&p, &chain().of(&drawing(0, 0), SLOT).unwrap()), Ok(2), "the two micro-dollars the chain counts, not the shares' three");
}

#[test]
fn the_miners_rent_a_batch_that_makes_its_dollar_account_needs_the_floor() {
    let m = drawing(0, 0);
    assert_eq!((mine::DOLLAR_ACCOUNT_RENT, mine::MINER_FLOOR_LAMPORTS), (2_039_280, 10_196_400));
    let at = |lamports: Option<u64>, dest: Option<(Pubkey, Pubkey)>| Chain { lamports, dest, ..chain() }.of(&m, SLOT).err();
    // the account to be made: the floor's edge, and lamports not read
    assert_eq!(at(Some(mine::MINER_FLOOR_LAMPORTS), None), None, "at the floor the batch makes the account and draws");
    assert_eq!(at(Some(mine::MINER_FLOOR_LAMPORTS - 1), None), Some("rent"), "a lamport under it: the month goes alone");
    assert_eq!(at(None, None), Some("rent"), "lamports not read: alone");
    // the account standing: the lamports are not asked
    let own = Some((key(202), key(200)));
    assert_eq!((at(None, own), at(Some(0), own)), (None, None));
}

#[test]
fn the_draw_never_holds_the_mine_a_month_the_pull_would_refuse_goes_alone_and_says_why() {
    let m = drawing(0, 0);
    let why = |c: Chain| c.of(&m, SLOT).err();
    assert_eq!(why(chain()), None, "the month stands");
    assert_eq!(why(Chain { period: 0, ..chain() }), Some("allowance"), "no period: no allowance the program opens");
    // the allowance read is this month's: another address's bytes are not it
    let (a, r, v) = (chain().allowance(), chain().registry(), token(&key(202), &poleis_client::treasury_pda(0), USD));
    let at = |allowance: Option<&[u8]>, registry: Option<&[u8]>, vault: Option<&[u8]>, dest: Option<&[u8]>, month: &Pubkey| mine::month_of(&m, SLOT, month, &mine::MonthAccounts { allowance, registry, vault, dest, miner_lamports: Some(mine::MINER_FLOOR_LAMPORTS) }).err();
    assert_eq!(at(Some(&a), Some(&r), Some(&v), None, &key(9)), Some("allowance"), "another month's bytes");
    assert_eq!(at(None, Some(&r), Some(&v), None, &chain().month()), Some("allowance"), "none stands");
    assert_eq!(at(Some(&a[..Allowance::SIZE - 1]), Some(&r), Some(&v), None, &chain().month()), Some("allowance"), "not of the layout");
    // expired at any slot the batch may land at (the program: past `expires`)
    assert_eq!(why(Chain { expires: SLOT + mine::DRAW_MARGIN_SLOTS - 1, ..chain() }), Some("expired"));
    assert_eq!(why(Chain { expires: SLOT + mine::DRAW_MARGIN_SLOTS, ..chain() }), None, "an expiry past the margin stands");
    assert_eq!(at(Some(&a), None, Some(&v), None, &chain().month()), Some("registry"), "no registry");
    let mut other = chain().registry();
    other[Registry::OFF_UNIT..Registry::OFF_UNIT + 8].copy_from_slice(&1u64.to_le_bytes());
    assert_eq!(at(Some(&a), Some(&other), Some(&v), None, &chain().month()), Some("registry"), "another unit's registry");
    assert_eq!(why(Chain { frozen: true, ..chain() }), Some("frozen"));
    assert_eq!(why(Chain { payee: key(7), ..chain() }), Some("payee"), "another key's month (Unauthorized)");
    assert_eq!(why(Chain { payee_kind: poleis_client::HOLDER_UNIT, ..chain() }), Some("payee"), "a unit's month");
    assert_eq!(why(Chain { mint: key(7), ..chain() }), Some("mint"), "a month of another mint than the mine's dollar");
    assert_eq!(why(Chain { default_mint: key(7), ..chain() }), Some("mint"), "the dollar not the registry's default mint: its cap is a TokenPlan's");
    assert_eq!(why(Chain { dest: Some((key(7), key(200))), ..chain() }), Some("account"), "an account at the miner's address of another mint");
    assert_eq!(why(Chain { dest: Some((key(202), key(7))), ..chain() }), Some("account"), "another owner's");
    assert_eq!(why(Chain { dest: Some((key(202), key(200))), ..chain() }), None, "the miner's own dollar account stands");
    assert_eq!(at(Some(&a), Some(&r), Some(&v), Some(&[]), &chain().month()), None, "an empty address: the batch makes the account");
    assert_eq!(why(Chain { vault: None, ..chain() }), Some("vault"), "no vault");
    assert_eq!(why(Chain { vault: Some((key(202), key(7), USD)), ..chain() }), Some("vault"), "a vault not root 0's treasury's");
    assert_eq!(why(Chain { vault: Some((key(7), poleis_client::treasury_pda(0), USD)), ..chain() }), Some("vault"), "a vault of another mint");
    assert_eq!(chain().of(&MineState { treasury: key(201), ..m }, SLOT).err(), Some("vault"), "a mine whose treasury is not root 0's vault of its dollar");
    let mut frozen = v.clone();
    frozen[108] = 2;
    assert_eq!(at(Some(&a), Some(&r), Some(&frozen), None, &chain().month()), Some("vault"), "a frozen vault");
}

#[test]
fn the_draw_rides_the_batch_one_signer_the_first_making_the_miners_account_and_fits_the_packet() {
    let now = HALVING + 3_600_000;
    let m = drawing(0, 0);
    let miner = m.miner;
    let first = chain().of(&m, SLOT).unwrap();
    let later = Chain { dest: Some((key(202), key(200))), ..chain() }.of(&m, SLOT).unwrap();
    assert!(first.create && !later.create);
    let entries = [Entry { flask: key(1), usd_micros: 2_100_000, at_ms: HALVING + 1_000, seq: 1 }];
    let pull = [&[poleis_client::tag::IX_PULL][..], &chain().hash, &(2 * USD).to_le_bytes()].concat();
    assert_eq!(
        instructions(&mine::draw_message(&miner, &entries, Some((&first, 2 * USD)), &[9; 32])),
        (1, vec![(mine::WARM, mine::mine_ix(&miner, &entries).data), (mine::ATA, vec![1]), (poleis_seat::PROGRAM, pull.clone())]),
        "the batch, then the miner's dollar account made, then the pull: the miner the one signer"
    );
    assert_eq!(instructions(&mine::draw_message(&miner, &entries, Some((&later, 2 * USD)), &[9; 32])), (1, vec![(mine::WARM, mine::mine_ix(&miner, &entries).data), (poleis_seat::PROGRAM, pull)]), "once the account stands, no create");
    // the pull is the client's own, into the miner's own dollar account, from root 0's vault, its cap's counter passed
    let p = mine::draw_ixs(&miner, &later, 5)[0].clone();
    let metas: Vec<(Pubkey, bool, bool)> = p.accounts.iter().map(|a| (a.pubkey, a.is_signer, a.is_writable)).collect();
    assert_eq!(metas[..5], [(miner, true, false), (poleis_client::registry_pda(0), false, true), (poleis_client::treasury_pda(0), false, true), (chain().month(), false, true), (mine::dollar_account_of(&miner, &key(202)), false, true)]);
    assert_eq!(metas[5..], [(m.treasury, false, true), (mine::TOKEN, false, false), (poleis_client::token_plan_pda(0, &key(202)), false, true)]);
    // the bytes it adds to a batch, measured, with the create and without, the same for any batch
    let (with, without) = (mine::draw_room(&miner, &first), mine::draw_room(&miner, &later));
    for n in [0u8, 1, 5] {
        let e: Vec<Entry> = (0..n).map(|i| Entry { flask: key(i + 1), usd_micros: 1, at_ms: HALVING, seq: i as u64 + 1 }).collect();
        let plain = mine::mine_message(&miner, &e, &[0; 32]).len();
        assert_eq!((mine::draw_message(&miner, &e, Some((&first, 1)), &[0; 32]).len() - plain, mine::draw_message(&miner, &e, Some((&later, 1)), &[0; 32]).len() - plain), (with, without), "{n} entries");
    }
    // a full batch with the draw is one packet with its one signature; without a month the batch packs exactly as before
    let rows: Vec<Spend> = (1..=40u8).map(|i| charged(i as u64, HALVING + i as u64 * 1_000, 1_000_000_000, i, "0.9")).collect();
    let all = rooms(&(1..=40u8).map(|i| (i, 10 * USD, 0)).collect::<Vec<_>>());
    let alone = mine::plan(&m, now, &rows, &all, &by());
    assert_eq!(mine::plan_within(&m, now, &rows, &all, &by(), 0), alone, "no reserve: the plan as it was");
    for month in [first, later] {
        let p = mine::plan_within(&m, now, &rows, &all, &by(), mine::draw_room(&miner, &month));
        let amount = mine::draw_of(&p, &month).unwrap();
        let wire = 1 + 64 + mine::draw_message(&miner, &p.entries, Some((&month, amount)), &[0; 32]).len();
        assert!(p.waits && wire <= PACKET && p.entries.len() < alone.entries.len(), "{} spends and the draw in {wire} bytes", p.entries.len());
        let one_more = [&p.entries[..], &[Entry { flask: key(99), usd_micros: 1, at_ms: now, seq: 99 }]].concat();
        assert!(1 + 64 + mine::draw_message(&miner, &one_more, Some((&month, amount)), &[0; 32]).len() > PACKET, "one more flask would not fit beside the draw");
        assert_eq!(amount, p.entries.len() as u64 * 900_000, "0.9 of each dollar mined");
        eprintln!("THE ROUTER DRAWS ITS MONTH: a full batch holds {} spends with the draw{} ({wire} bytes on the wire), {} alone; the draw adds {} bytes", p.entries.len(), if month.create { " and the create" } else { "" }, alone.entries.len(), mine::draw_room(&miner, &month));
    }
    eprintln!("THE ROUTER DRAWS ITS MONTH: the draw adds {with} bytes to a batch's message with the create, {without} without");
}

#[test]
fn the_one_door_reads_the_month_beside_the_records_and_with_none_named_answers_as_before() {
    let ask = |v: Value| -> Value { serde_json::from_str(&mine::ask(&v.to_string())).unwrap() };
    let node = |d: &[u8], owner: &Pubkey| json!({ "data": [mine::b64(d), "base64"], "lamports": 1, "owner": owner.to_string() });
    let m = drawing(7, HALVING - 1);
    let mut mine_bytes = vec![0u8; Mine::SIZE];
    (mine_bytes[0], mine_bytes[1]) = (DISC_MINE, 1);
    for (o, k) in [(Mine::OFF_MINER, m.miner), (Mine::OFF_TREASURY, m.treasury), (Mine::OFF_DOLLAR, m.dollar)] {
        mine_bytes[o..o + 32].copy_from_slice(k.as_ref());
    }
    for (o, n) in [(Mine::OFF_OPENING, OPENING), (Mine::OFF_LAST_SEQ, 7), (Mine::OFF_LAST_AT, HALVING - 1)] {
        mine_bytes[o..o + 8].copy_from_slice(&n.to_le_bytes());
    }
    let mut clock = vec![0u8; 40];
    clock[..8].copy_from_slice(&SLOT.to_le_bytes());
    clock[32..].copy_from_slice(&((HALVING / 1_000 + 60) as i64).to_le_bytes());
    assert_eq!(mine::clock_slot(&clock), Some(SLOT));
    let s = charged(8, HALVING + 1_000, 2_100_000_000, 1, "2");
    let rows = json!([{ "id": 8, "at": s.at_ms, "nanos": s.nanos, "tank": s.tank, "flasks": key(1).to_string(), "body": s.body }]);
    // wants: with no month, as before, the mine given or not
    let before = mine::ask(&json!({ "do": "wants", "rows": rows }).to_string());
    assert_eq!(mine::ask(&json!({ "do": "wants", "rows": rows, "mine": node(&mine_bytes, &mine::WARM) }).to_string()), before, "no month: the same answer, byte for byte");
    assert_eq!(mine::ask(&json!({ "do": "wants", "rows": rows, "mine": node(&mine_bytes, &mine::WARM), "month": "not an address" }).to_string()), before, "a month that is no address reads nothing more");
    let c = chain();
    let wants = ask(json!({ "do": "wants", "rows": rows, "mine": node(&mine_bytes, &mine::WARM), "month": c.month().to_string() }));
    let dest = mine::dollar_account_of(&m.miner, &m.dollar);
    let mut keys: Vec<String> = wants["records"].as_object().unwrap().keys().cloned().collect();
    keys.sort();
    let mut want = vec![mine::record_of(&key(1)).to_string(), c.month().to_string(), poleis_client::registry_pda(0).to_string(), m.treasury.to_string(), dest.to_string(), m.miner.to_string()];
    want.sort();
    assert_eq!(keys, want, "the flask's record, and the month's allowance, root 0's registry, the mine's treasury, the miner's dollar account, and the miner (its lamports): five");
    // plan: the records as a node answers them, the miner's dollar account not yet standing
    let mut r = vec![0u8; Flask::SIZE];
    (r[0], r[1]) = (DISC_FLASK, 1);
    r[Flask::OFF_FLASK..Flask::OFF_FLASK + 32].copy_from_slice(key(1).as_ref());
    r[Flask::OFF_DEPOSITED..Flask::OFF_DEPOSITED + 8].copy_from_slice(&(10 * USD).to_le_bytes());
    let vault = token(&m.dollar, &poleis_client::treasury_pda(0), 500 * USD);
    let records = |allowance_owner: &Pubkey| {
        json!({ mine::record_of(&key(1)).to_string(): node(&r, &mine::WARM), c.month().to_string(): node(&c.allowance(), allowance_owner), poleis_client::registry_pda(0).to_string(): node(&c.registry(), &poleis_seat::PROGRAM),
            m.treasury.to_string(): node(&vault, &mine::TOKEN), dest.to_string(): Value::Null, m.miner.to_string(): json!({ "data": ["", "base64"], "lamports": mine::MINER_FLOOR_LAMPORTS, "owner": Pubkey::default().to_string() }) })
    };
    let base = json!({ "do": "plan", "mine": node(&mine_bytes, &mine::WARM), "clock": node(&clock, &Pubkey::default()), "rows": rows, "records": records(&poleis_seat::PROGRAM), "oracle": hex(&by()), "blockhash": key(9).to_string() });
    let mut asked = base.clone();
    asked["month"] = json!(c.month().to_string());
    let plan = ask(asked.clone());
    let entry = [Entry { flask: key(1), usd_micros: 2_100_000, at_ms: HALVING + 1_000, seq: 8 }];
    let month = c.of(&m, SLOT).unwrap();
    assert_eq!((plan["draw"].as_u64(), plan["draw_why"].is_null()), (Some(2 * USD), true), "{plan}");
    assert_eq!(mine::unb64(plan["message"].as_str().unwrap()).unwrap(), mine::draw_message(&m.miner, &entry, Some((&month, 2 * USD)), &[9; 32]), "the batch and its draw, the create with it");
    // with no month: no draw in the answer, the batch's message as before this lane
    let v = ask(base.clone());
    assert_eq!((v.get("draw"), v.get("draw_why"), mine::unb64(v["message"].as_str().unwrap()).unwrap()), (None, None, mine::mine_message(&m.miner, &entry, &[9; 32])));
    // a month the pull would refuse: the batch alone, and the word; an allowance some other program owns is none
    let mut foreign = asked.clone();
    foreign["records"] = records(&mine::TOKEN);
    let f = ask(foreign);
    assert_eq!((f["draw"].as_u64(), f["draw_why"].as_str(), mine::unb64(f["message"].as_str().unwrap()).unwrap()), (Some(0), Some("allowance"), mine::mine_message(&m.miner, &entry, &[9; 32])));
    let mut odd = asked.clone();
    odd["month"] = json!("not an address");
    assert_eq!(ask(odd)["draw_why"].as_str(), Some("month"));
    // THE MINER'S RENT through the door: the miner's lamports are read off its record; a lamport under the floor, or none read, and the batch
    // (which would make the miner's dollar account) goes alone
    for lamports in [json!(mine::MINER_FLOOR_LAMPORTS - 1), Value::Null] {
        let mut poor = asked.clone();
        poor["records"][m.miner.to_string()] = if lamports.is_null() { Value::Null } else { json!({ "data": ["", "base64"], "lamports": lamports }) };
        let p = ask(poor);
        assert_eq!((p["draw"].as_u64(), p["draw_why"].as_str(), mine::unb64(p["message"].as_str().unwrap()).unwrap()), (Some(0), Some("rent"), mine::mine_message(&m.miner, &entry, &[9; 32])), "{lamports}");
    }
    // nothing due: no message, no draw
    let mut done = asked.clone();
    done["rows"] = json!([]);
    let d = ask(done);
    assert_eq!((d["message"].is_null(), d["draw"].as_u64(), d["draw_why"].as_str()), (true, Some(0), Some("nothing")));
}
