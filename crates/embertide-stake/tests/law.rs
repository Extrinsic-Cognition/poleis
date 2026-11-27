//! The law of embertide-stake, held to mainnet's own bytes where a fixture exists (tests/fixtures: read Nov 20 2026, epoch 1092) and to invented
//! holders everywhere else (no stranger's wallet is kept in this tree: the fixtures hold pools' and programs' own accounts only).

use std::collections::BTreeMap;

use embertide_stake::*;
use serde_json::{json, Value};

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap().to_string()
}

// ---------------------------------------------------------------- the pool, read

#[test]
fn every_fixture_pool_reads_as_the_chain_holds_it() {
    let f = fixture("pools.json");
    let pools = f["pools"].as_array().unwrap();
    assert_eq!(pools.len(), 4, "the SPL program's two, and one of each of Sanctum's forks");
    let mut programs = std::collections::BTreeSet::new();
    for p in pools {
        let bytes = base64_of(p["data"].as_str().unwrap()).unwrap();
        let pool = pool_of(&bytes).unwrap_or_else(|e| panic!("{}: {e}", p["name"]));
        let e = &p["expect"];
        assert_eq!(address(&pool.manager), s(e, "manager"));
        assert_eq!(address(&pool.staker), s(e, "staker"));
        assert_eq!(address(&pool.validator_list), s(e, "validator_list"));
        assert_eq!(address(&pool.reserve_stake), s(e, "reserve_stake"));
        assert_eq!(address(&pool.pool_mint), s(e, "pool_mint"));
        assert_eq!(address(&pool.manager_fee_account), s(e, "manager_fee_account"));
        assert_eq!(address(&pool.token_program), s(e, "token_program"));
        assert_eq!(pool.total_lamports.to_string(), s(e, "total_lamports"));
        assert_eq!(pool.pool_token_supply.to_string(), s(e, "pool_token_supply"));
        assert_eq!(pool.last_update_epoch, e["last_update_epoch"].as_u64().unwrap());
        assert_eq!(pool.epoch_fee.numerator.to_string(), e["epoch_fee"][0].as_str().unwrap());
        assert_eq!(pool.epoch_fee.denominator.to_string(), e["epoch_fee"][1].as_str().unwrap());
        // the two fields at the end of the walk are the epoch before's totals: within a twentieth of today's, or the walk went wrong
        for (was, is) in [(pool.last_epoch_total_lamports, pool.total_lamports), (pool.last_epoch_pool_token_supply, pool.pool_token_supply)] {
            assert!(was > is / 20 * 19 && was < is / 20 * 21, "{}: {was} beside {is}", p["name"]);
        }
        programs.insert(s(p, "program"));
    }
    assert_eq!(programs.len(), 3, "one layout over three programs");
}

#[test]
fn a_personal_pool_takes_the_whole_yield_and_jito_does_not() {
    let f = fixture("pools.json");
    let read = |i: usize| pool_of(&base64_of(f["pools"][i]["data"].as_str().unwrap()).unwrap()).unwrap();
    assert!(read(0).epoch_fee.is_whole(), "fpSOL: 100/100");
    assert!(read(1).epoch_fee.is_whole());
    assert!(read(2).epoch_fee.is_whole());
    let jito = read(3).epoch_fee;
    assert_eq!((jito.numerator, jito.denominator), (4, 100));
    assert!(!jito.is_whole());
    assert!(!Fee { numerator: 0, denominator: 0 }.is_whole(), "a fee over nothing is not the whole");
}

#[test]
fn bytes_that_are_not_a_pool_are_refused() {
    let f = fixture("pools.json");
    let good = base64_of(f["pools"][0]["data"].as_str().unwrap()).unwrap();
    assert!(matches!(pool_of(&[]), Err(Refusal::NotAPool(_))));
    assert!(matches!(pool_of(&good[..345]), Err(Refusal::NotAPool(_))), "cut inside the epoch fee");
    assert!(matches!(pool_of(&good[..400]), Err(Refusal::NotAPool(_))), "cut before the last two fields");
    let mut other = good.clone();
    other[0] = 2; // a validator list's first byte
    assert!(matches!(pool_of(&other), Err(Refusal::NotAPool(_))));
    let mut tag = good.clone();
    tag[346] = 9; // the next epoch fee's tag
    assert!(matches!(pool_of(&tag), Err(Refusal::NotAPool(_))));
    let mut opt = good.clone();
    opt[347] = 7; // the preferred deposit validator's tag
    assert!(matches!(pool_of(&opt), Err(Refusal::NotAPool(_))));
}

#[test]
fn a_future_fee_and_a_preferred_validator_move_the_walk() {
    // the same pool with a fee set for next epoch (tag 1, sixteen bytes) and a preferred deposit validator (tag 1, a key): the last two fields
    // are found 48 bytes later, and are the same two
    let f = fixture("pools.json");
    let good = base64_of(f["pools"][0]["data"].as_str().unwrap()).unwrap();
    let was = pool_of(&good).unwrap();
    let mut moved = good[..346].to_vec();
    moved.push(1);
    moved.extend_from_slice(&[7u8; 16]);
    moved.push(1);
    moved.extend_from_slice(&[9u8; 32]);
    moved.extend_from_slice(&good[348..]);
    assert_eq!(pool_of(&moved).unwrap(), was);
}

#[test]
fn the_fee_account_reads_as_a_token_account() {
    let f = fixture("fpsol-fee.json");
    let (mint, owner, amount) = token_of(&base64_of(f["fee_account_data"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(address(&mint), s(&f["fee_account_expect"], "mint"));
    assert_eq!(address(&owner), s(&f["fee_account_expect"], "owner"));
    assert_eq!(amount.to_string(), s(&f["fee_account_expect"], "amount"));
    assert_eq!(token_of(&[0u8; 164]), Err(Refusal::NotATokenAccount));
    // and its mint is the pool's own
    let pools = fixture("pools.json");
    let pool = pool_of(&base64_of(pools["pools"][0]["data"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(mint, pool.pool_mint);
    assert_eq!(address(&pool.manager_fee_account), s(&f, "fee_account"));
}

#[test]
fn tokens_are_worth_lamports_at_the_pools_rate_rounded_down() {
    assert_eq!(lamports_of(10, 3, 2), 15);
    assert_eq!(lamports_of(1, 3, 2), 1, "1.5 rounds down");
    assert_eq!(lamports_of(5, 100, 0), 0, "a pool with no tokens prices nothing");
    assert_eq!(lamports_of(u64::MAX, u64::MAX, u64::MAX), u64::MAX, "no overflow on the way");
}

// ---------------------------------------------------------------- the epoch's yield

fn fee_fixture() -> (Value, String, String, String, Schedule) {
    let f = fixture("fpsol-fee.json");
    let schedule = Schedule {
        slots_per_epoch: f["schedule"]["slotsPerEpoch"].as_u64().unwrap(),
        first_normal_epoch: f["schedule"]["firstNormalEpoch"].as_u64().unwrap(),
        first_normal_slot: f["schedule"]["firstNormalSlot"].as_u64().unwrap(),
    };
    let (program, pool, fee) = (s(&f, "program"), s(&f, "pool"), s(&f, "fee_account"));
    (f, program, pool, fee, schedule)
}

fn all_mints(f: &Value, program: &str, pool: &str, fee: &str) -> Vec<FeeMint> {
    f["transactions"].as_array().unwrap().iter().flat_map(|t| fee_mints(t, program, pool, fee)).collect()
}

#[test]
fn the_yield_of_two_real_epochs_is_the_sum_of_the_programs_own_mints() {
    let (f, program, pool, fee, schedule) = fee_fixture();
    let mints = all_mints(&f, &program, &pool, &fee);
    assert_eq!(mints.len(), 4, "two updates an epoch, two epochs; the owner's two sweeps mint nothing");
    assert_eq!(epoch_fee(&mints, &schedule, 1092).unwrap().to_string(), s(&f["expect"], "1092"));
    assert_eq!(epoch_fee(&mints, &schedule, 1091).unwrap().to_string(), s(&f["expect"], "1091"));
    assert_eq!(epoch_fee(&mints, &schedule, 1090).unwrap(), 0, "nothing of 1090 is in the fixture");
    assert_eq!(epoch_fee(&mints, &schedule, 1093).unwrap(), 0);
}

#[test]
fn a_balance_read_would_have_said_the_wrong_yield() {
    // the reason the law reads the mints: across epoch 1092 the fee account's balance went 1,750,162,466 -> 1,749,103,570 (it FELL, by the owner's
    // sweep), while the program minted 25,854,151 into it
    let (f, program, pool, fee, schedule) = fee_fixture();
    let minted = epoch_fee(&all_mints(&f, &program, &pool, &fee), &schedule, 1092).unwrap();
    let (before, after): (i64, i64) = (1_750_162_466, 1_749_103_570);
    assert!(after - before < 0);
    assert_eq!(minted, 25_854_151);
}

#[test]
fn only_the_pools_own_update_counts() {
    let (f, program, pool, fee, _) = fee_fixture();
    let txs = f["transactions"].as_array().unwrap();
    let update = txs.iter().find(|t| t["slot"] == 471745493u64).unwrap().clone();
    assert_eq!(fee_mints(&update, &program, &pool, &fee), vec![FeeMint { slot: 471745493, tokens: 25_198_739 }]);

    // the owner's sweep of the fee account: no instruction of the pool's program
    let sweep = txs.iter().find(|t| t["slot"] == 471762910u64).unwrap();
    assert!(fee_mints(sweep, &program, &pool, &fee).is_empty());

    // a failed transaction minted nothing
    let mut failed = update.clone();
    failed["meta"]["err"] = json!({ "InstructionError": [1, "Custom"] });
    assert!(fee_mints(&failed, &program, &pool, &fee).is_empty());

    // another program, another pool, another fee account
    assert!(fee_mints(&update, "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy", &pool, &fee).is_empty());
    assert!(fee_mints(&update, &program, "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb", &fee).is_empty());
    assert!(fee_mints(&update, &program, &pool, &pool).is_empty());

    // a transfer into the fee account (a withdrawal's fee) is not the yield
    let mut transfer = update.clone();
    transfer["meta"]["innerInstructions"][0]["instructions"][0]["parsed"]["type"] = json!("transfer");
    assert!(fee_mints(&transfer, &program, &pool, &fee).is_empty());

    // a mint under another instruction of the same program (a deposit's fee: DepositSol is 14) is not the yield
    let mut deposit = update.clone();
    let at = deposit["meta"]["innerInstructions"][0]["index"].as_u64().unwrap() as usize;
    deposit["transaction"]["message"]["instructions"][at]["data"] = json!(bs58::encode([14u8, 1, 2, 3]).into_string());
    assert!(fee_mints(&deposit, &program, &pool, &fee).is_empty());

    // an inner group that points at no outer instruction, and a transaction with no slot
    let mut astray = update.clone();
    astray["meta"]["innerInstructions"][0]["index"] = json!(99);
    assert!(fee_mints(&astray, &program, &pool, &fee).is_empty());
    let mut slotless = update.clone();
    slotless.as_object_mut().unwrap().remove("slot");
    assert!(fee_mints(&slotless, &program, &pool, &fee).is_empty());
    assert!(fee_mints(&json!(null), &program, &pool, &fee).is_empty());
}

#[test]
fn the_update_is_the_eighth_instruction() {
    // every real update in the fixture carries the tag the law names
    let (f, program, _, _, _) = fee_fixture();
    let mut seen = 0;
    for t in f["transactions"].as_array().unwrap() {
        for g in t["meta"]["innerInstructions"].as_array().unwrap() {
            let ix = &t["transaction"]["message"]["instructions"][g["index"].as_u64().unwrap() as usize];
            assert_eq!(ix["programId"].as_str().unwrap(), program);
            assert_eq!(bs58::decode(ix["data"].as_str().unwrap()).into_vec().unwrap()[0], UPDATE_POOL_BALANCE);
            seen += 1;
        }
    }
    assert_eq!(seen, 4);
}

#[test]
fn an_epoch_has_its_slots() {
    let main = Schedule { slots_per_epoch: 432_000, first_normal_epoch: 0, first_normal_slot: 0 };
    assert_eq!(main.slots_of(1092).unwrap(), (471_744_000, 472_175_999));
    assert_eq!(main.epoch_of(471_744_000), Some(1092));
    assert_eq!(main.epoch_of(471_743_999), Some(1091));
    // a cluster that warmed up (devnet's shape): epochs before the first normal one had other lengths and are refused, never guessed
    let warm = Schedule { slots_per_epoch: 432_000, first_normal_epoch: 14, first_normal_slot: 524_256 };
    assert_eq!(warm.slots_of(14).unwrap(), (524_256, 956_255));
    assert_eq!(warm.slots_of(15).unwrap().0, 956_256);
    assert_eq!(warm.slots_of(13), Err(Refusal::BeforeNormalEpochs));
    assert_eq!(warm.epoch_of(524_255), None);
    assert_eq!(warm.epoch_of(956_256), Some(15));
    assert_eq!(epoch_fee(&[], &warm, 3), Err(Refusal::BeforeNormalEpochs));
}

// ---------------------------------------------------------------- the book

fn snap(epoch: u64, holders: &[(&str, u64)]) -> Snapshot {
    let total: u64 = holders.iter().map(|h| h.1).sum();
    // a rate of exactly one: a token a lamport, as a pool whose whole yield is its fee keeps it
    Snapshot { epoch, total_lamports: total.max(1), pool_token_supply: total.max(1), holders: holders.iter().map(|(k, v)| (k.to_string(), *v)).collect() }
}

#[test]
fn the_first_read_counts_nothing() {
    let b = count(&Book::new("pool"), &snap(10, &[("ada", 100)]), 999).unwrap();
    assert!(b.row.is_none() && b.counted == 0 && b.stakers.is_empty(), "nobody has held from one read to the next yet");
    assert_eq!(b.last.unwrap().epoch, 10);
}

#[test]
fn held_is_the_lesser_of_two_reads() {
    let b = count(&Book::new("pool"), &snap(10, &[("ada", 100), ("bo", 300), ("cy", 50)]), 0).unwrap();
    // ada adds, bo takes half out, cy leaves, dee arrives
    let b = count(&b, &snap(11, &[("ada", 500), ("bo", 150), ("dee", 1000)]), 1000).unwrap();
    assert_eq!(b.stakers["ada"].stake_time, 100, "what stood at both reads, never what was added between");
    assert_eq!(b.stakers["bo"].stake_time, 150);
    assert!(!b.stakers.contains_key("cy"), "left before this read");
    assert!(!b.stakers.contains_key("dee"), "came after the last read: arriving before a read earns nothing");
    let row = b.row.clone().unwrap();
    assert_eq!((row.epoch, row.from, row.holders, row.held_lamports), (11, 10, 2, 250));
    // the fee by what was held: 1000 * 100/250 and 1000 * 150/250
    assert_eq!((b.stakers["ada"].light, b.stakers["bo"].light), (400, 600));
    assert_eq!(row.dust, 0);
    // dee is counted from the next epoch on
    let b = count(&b, &snap(12, &[("ada", 500), ("bo", 150), ("dee", 1000)]), 0).unwrap();
    assert_eq!(b.stakers["dee"].stake_time, 1000);
    assert_eq!(b.stakers["ada"].stake_time, 600);
    assert_eq!(b.stakers["ada"].epochs, 2);
}

#[test]
fn the_lights_and_the_dust_are_the_fee() {
    let b = count(&Book::new("pool"), &snap(1, &[("a", 1), ("b", 1), ("c", 1)]), 0).unwrap();
    let b = count(&b, &snap(2, &[("a", 1), ("b", 1), ("c", 1)]), 100).unwrap();
    assert_eq!(b.stakers.values().map(|s| s.light).sum::<u128>(), 99);
    assert_eq!((b.row.as_ref().unwrap().dust, b.dust, b.fee_lamports), (1, 1, 100), "what rounding leaves is said, never given to one of them");
    // nobody held across: the whole fee is dust
    let b2 = count(&Book::new("pool"), &snap(1, &[("a", 5)]), 0).unwrap();
    let b2 = count(&b2, &snap(2, &[("z", 5)]), 100).unwrap();
    assert!(b2.stakers.is_empty());
    let r = b2.row.as_ref().unwrap();
    assert_eq!((r.dust, r.fee_lamports, r.holders), (100, 100, 0));
}

#[test]
fn an_epoch_is_counted_once_and_the_book_given_is_not_changed() {
    let first = count(&Book::new("pool"), &snap(10, &[("ada", 100)]), 0).unwrap();
    let second = count(&first, &snap(11, &[("ada", 100)]), 7).unwrap();
    assert_eq!(count(&second, &snap(11, &[("ada", 100)]), 7), Err(Refusal::AlreadyCounted { epoch: 11 }));
    assert_eq!(count(&second, &snap(9, &[("ada", 100)]), 7), Err(Refusal::AlreadyCounted { epoch: 9 }));
    assert!(first.row.is_none() && first.counted == 0, "count returns a new book");
    assert_eq!(second.counted, 1);
}

#[test]
fn a_gap_is_counted_as_one_epoch() {
    let b = count(&Book::new("pool"), &snap(10, &[("ada", 100)]), 0).unwrap();
    let b = count(&b, &snap(15, &[("ada", 100)]), 50).unwrap();
    assert_eq!(b.stakers["ada"].stake_time, 100, "five epochs passed unread: what stood between is not known, so one is counted");
    let r = b.row.as_ref().unwrap();
    assert_eq!((r.from, r.epoch, b.counted), (10, 15, 1));
    assert_eq!(b.stakers["ada"].light, 50, "the yield of the whole gap, which the caller summed, is still theirs");
}

#[test]
fn held_and_fee_are_valued_at_the_reads_rate() {
    // a pool whose token is worth 1.5 lamports: 100 tokens held are 150 lamports of stake-time, and a fee of 10 tokens is 15 lamports of light
    let at = |epoch| Snapshot { epoch, total_lamports: 1500, pool_token_supply: 1000, holders: BTreeMap::from([("ada".to_string(), 100u64)]) };
    let b = count(&count(&Book::new("pool"), &at(1), 0).unwrap(), &at(2), 10).unwrap();
    assert_eq!(b.stakers["ada"].stake_time, 150);
    let r = b.row.as_ref().unwrap();
    assert_eq!((r.fee_tokens, r.fee_lamports), (10, 15));
    assert_eq!(b.stakers["ada"].light, 15);
}

#[test]
fn over_many_invented_books_nothing_is_made_or_lost() {
    // a small generator of its own (no dependency): 300 books of 2 to 6 reads over up to 12 holders
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = move |n: u64| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x % n
    };
    for _ in 0..300 {
        let mut book = Book::new("pool");
        let mut epoch = 100;
        let mut fees: u128 = 0;
        let mut before: Option<Snapshot> = None;
        for _ in 0..(2 + next(5)) {
            let mut holders = BTreeMap::new();
            for h in 0..12 {
                if next(3) > 0 {
                    holders.insert(format!("h{h}"), 1 + next(1_000_000_000));
                }
            }
            let supply = holders.values().sum::<u64>().max(1);
            let s = Snapshot { epoch, total_lamports: supply + next(supply), pool_token_supply: supply, holders };
            let fee = next(50_000_000);
            let was = book.clone();
            book = count(&book, &s, fee).unwrap();
            if let Some(b) = &before {
                let row = book.row.clone().unwrap();
                fees += row.fee_lamports as u128;
                let lights: u128 = book.stakers.iter().map(|(k, v)| v.light - was.stakers.get(k).map_or(0, |w| w.light)).sum();
                assert_eq!(lights + row.dust as u128, row.fee_lamports as u128);
                for (k, v) in &book.stakers {
                    let grew = v.stake_time - was.stakers.get(k).map_or(0, |w| w.stake_time);
                    let both = b.holders.get(k).copied().unwrap_or(0).min(s.holders.get(k).copied().unwrap_or(0));
                    assert_eq!(grew, lamports_of(both, s.total_lamports, s.pool_token_supply) as u128, "only what stood at both reads");
                }
            }
            before = Some(s);
            epoch += 1 + next(3);
        }
        let lights: u128 = book.stakers.values().map(|s| s.light).sum();
        assert_eq!(book.fee_lamports, fees);
        assert_eq!(lights + book.dust, fees, "every lamport of every fee is in a light or in the dust");
    }
}

#[test]
fn the_holders_of_a_read() {
    let accounts = vec![
        ("acct1".to_string(), "ada".to_string(), 100u64),
        ("acct2".to_string(), "ada".to_string(), 50),
        ("acct3".to_string(), "bo".to_string(), 0),
        ("fee".to_string(), "house".to_string(), 999),
        ("acct4".to_string(), "house".to_string(), 30),
        ("acct5".to_string(), "cy".to_string(), 7),
    ];
    let h = holders_of(&accounts, "fee", &[]);
    assert_eq!(h, BTreeMap::from([("ada".to_string(), 150), ("cy".to_string(), 7), ("house".to_string(), 30)]), "an owner's accounts are summed; the fee account is the yield, not a stake; an empty account holds nothing");
    let h = holders_of(&accounts, "fee", &["house".to_string()]);
    assert!(!h.contains_key("house"), "an owner the caller excepts is left out");
}

#[test]
fn the_book_goes_to_json_and_back() {
    let mut b = count(&Book::new("GutG5bcmEZw15WmPHNVMWHU77c6t8CEinUEdPLYz3doa"), &snap(10, &[("ada", 100), ("bo", 300)]), 0).unwrap();
    b = count(&b, &snap(11, &[("ada", 100), ("bo", 300)]), 1001).unwrap();
    // a stake-time past what a JSON number keeps: it is written as digits
    b.stakers.get_mut("ada").unwrap().stake_time = u64::MAX as u128 * 1000 + 7;
    let v = book_json(&b);
    assert_eq!(v["schema"], BOOK_SCHEMA);
    assert_eq!(v["stakers"]["ada"]["stake_time"], "18446744073709551615007");
    let text = serde_json::to_string_pretty(&v).unwrap();
    let back = book_of(&serde_json::from_str(&text).unwrap(), "GutG5bcmEZw15WmPHNVMWHU77c6t8CEinUEdPLYz3doa").unwrap();
    assert_eq!(back, b);
    assert!(v.get("epochs").is_none(), "the book is state: it keeps the last row and the sums, never a list of epochs");
    assert_eq!((back.counted, back.fee_lamports, back.row.as_ref().map(|r| r.epoch)), (1, 1001, Some(11)));
    // an empty book too
    let empty = Book::new("p");
    assert_eq!(book_of(&book_json(&empty), "p").unwrap(), empty);
}

#[test]
fn a_book_of_another_pool_or_schema_is_refused() {
    let v = book_json(&Book::new("one"));
    assert!(matches!(book_of(&v, "two"), Err(Refusal::NotThisBook(_))));
    let mut other = v.clone();
    other["schema"] = json!("embertide.stake-book/0");
    assert!(matches!(book_of(&other, "one"), Err(Refusal::NotThisBook(_))));
    let mut bad = book_json(&count(&Book::new("one"), &snap(1, &[("a", 1)]), 0).unwrap());
    bad["last"]["holders"]["a"] = json!("-3");
    assert!(matches!(book_of(&bad, "one"), Err(Refusal::NotANumber(_))));
    assert!(matches!(book_of(&json!({}), "one"), Err(Refusal::NotThisBook(_))));
}

// ---------------------------------------------------------------- the refill's row

#[test]
fn an_epochs_yield_is_one_row_the_ledger_takes() {
    let pool = "GutG5bcmEZw15WmPHNVMWHU77c6t8CEinUEdPLYz3doa";
    let row = refill_row(1092, pool, "nefertiti", 25_854_151, 71.6).unwrap();
    assert_eq!(row["key"], format!("pool:1092:{pool}"));
    let why = row["why"].as_str().unwrap();
    // the Oct 29 crank's ledger finds a row by the first word of its why (pool-crank.mjs writtenKeys: KEY_RX = pool:<digits>:<an address>)
    let first = why.split(' ').next().unwrap();
    assert_eq!(first, row["key"].as_str().unwrap());
    let parts: Vec<&str> = first.split(':').collect();
    assert_eq!(parts.len(), 3);
    assert!(parts[0] == "pool" && parts[1].bytes().all(|b| b.is_ascii_digit()) && (32..=44).contains(&parts[2].len()) && key_of(parts[2]).is_some());
    // dollars as that crank rounds them: nine places
    assert_eq!(row["usd"].as_f64().unwrap(), 1.851157212);
    assert_eq!(row["lamports"], 25_854_151u64);
    assert_eq!(row["tank"], "nefertiti");
    assert!(why.ends_with("25854151 lamports at $71.6/SOL"));
    assert!(refill_row(1, pool, "her", 5, 100.0).unwrap()["why"].as_str().unwrap().ends_with("at $100/SOL"), "a whole price is said as the crank says it");
}

#[test]
fn a_row_is_refused_for_a_bad_tank_price_or_pool() {
    let pool = "GutG5bcmEZw15WmPHNVMWHU77c6t8CEinUEdPLYz3doa";
    for bad in ["", "Nefertiti", "9lives", "a_b", "a b", &"a".repeat(33)] {
        assert_eq!(refill_row(1, pool, bad, 1, 1.0), Err(Refusal::NotATank), "{bad}");
    }
    assert!(is_tank("snowtrace-riders") && is_tank("st-0123456789ab") && is_tank("a"));
    for price in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(refill_row(1, pool, "her", 1, price), Err(Refusal::NotANumber(_))));
    }
    assert!(matches!(refill_row(1, "not-an-address", "her", 1, 1.0), Err(Refusal::NotAPool(_))));
}

// ---------------------------------------------------------------- the wire's bytes

#[test]
fn base64_reads_as_the_standard_says() {
    assert_eq!(base64_of("").unwrap(), b"");
    assert_eq!(base64_of("Zg==").unwrap(), b"f");
    assert_eq!(base64_of("Zm8=").unwrap(), b"fo");
    assert_eq!(base64_of("Zm9v").unwrap(), b"foo");
    assert_eq!(base64_of("Zm9vYmFy").unwrap(), b"foobar");
    assert_eq!(base64_of("Zm9vYmE").unwrap(), b"fooba", "unpadded");
    assert_eq!(base64_of("+/+/").unwrap(), [0xfb, 0xff, 0xbf]);
    assert_eq!(base64_of("Zm9v!"), None);
    let key = key_of("GutG5bcmEZw15WmPHNVMWHU77c6t8CEinUEdPLYz3doa").unwrap();
    assert_eq!(address(&key), "GutG5bcmEZw15WmPHNVMWHU77c6t8CEinUEdPLYz3doa");
    assert_eq!(key_of("short"), None);
}
