//! THE LAW ON CHAIN IS THE ROUTER'S: the program's `halving_at`, `era_at` and `warm_nanos` (src/law.rs) held row by row to the router's
//! own `halvingAt`, `eraAt` and `warmMined` (embertide-app/ui/intro/dyson-society/app/space-router-law.mjs) by a table those functions
//! made, kept here as data (`law-parity.json`), and to the very text it was made from (its sha256 is in the table, and this test hashes
//! the law's file as it stands: a change of the router's law is red here until the table is made again).
//!
//! The table was made ONCE, on the new Mac (Nov 21 2026), at the house's root, by node and the law's own functions; to make it again run
//! there `node --input-type=module -e '<the lines below>' > poleis/program/warm_mine/tests/law-parity.json` (no JavaScript file is kept):
//!
//!   import { halvingAt, eraAt, warmMined, HALVING_MONTHS, WARM_PER_USD } from "./embertide-app/ui/intro/dyson-society/app/space-router-law.mjs";
//!   import { createHash } from "node:crypto";
//!   import { readFileSync } from "node:fs";
//!   const LAW = "embertide-app/ui/intro/dyson-society/app/space-router-law.mjs", D = 86400000, H = 3600000;
//!   const opens = ["1970-01-01T00:00:00.000Z", "2024-02-29T00:00:00.001Z", "2025-08-31T23:59:59.999Z", "2026-08-31T12:34:56.789Z", "2026-10-31T12:00:00.000Z", "2026-11-15T00:00:00.000Z", "2026-12-31T23:59:59.999Z", "2027-03-31T06:00:00.000Z", "2099-08-31T00:00:00.000Z", "2399-08-30T08:08:08.008Z"].map(Date.parse);
//!   const halvings = [], eras = [], mined = [];
//!   let s = 20261121;
//!   const rnd = () => (s = (s * 48271) % 2147483647) / 2147483647;
//!   for (const o of opens) {
//!     for (let n = 0; n <= 48; n++) halvings.push([o, n, halvingAt(o, n)]);
//!     for (const n of [97, 200, 401, 999]) halvings.push([o, n, halvingAt(o, n)]);
//!     for (const at of [o - 1, o, o + 1]) if (at >= 0) eras.push([o, at, eraAt(at, o)]);
//!     for (let n = 1; n <= 48; n++) { const h = halvingAt(o, n); for (const at of [h - 1, h, h + 1, h + D]) eras.push([o, at, eraAt(at, o)]); }
//!     for (let i = 0; i < 64; i++) { const at = o + Math.floor(rnd() * 300 * 365.2425 * D); eras.push([o, at, eraAt(at, o)]); }
//!   }
//!   const nanos = (usd, at, o) => Number(warmMined(usd, at, o).toFixed(9).replace(".", ""));
//!   const usd = [1, 7, 15, 16, 17, 999, 1000, 999999, 1000000, 1234567, 10000000, 123456789, 1000000000000];
//!   const o = Date.parse("2026-08-31T12:34:56.789Z");
//!   for (let n = 0; n <= 44; n++) { const at = halvingAt(o, n) + H; for (const u of usd) mined.push([u, at, o, nanos(u / 1e6, at, o)]); }
//!   for (const u of usd) mined.push([u, o - D, o, nanos(u / 1e6, o - D, o)]);
//!   const law_sha256 = createHash("sha256").update(readFileSync(LAW)).digest("hex");
//!   process.stdout.write(JSON.stringify({ law: LAW, law_sha256, WARM_PER_USD, HALVING_MONTHS, halvings, eras, mined }).replace(/\],\[/g, "],\n[") + "\n");
//!
//! The openings cross month ends (Aug 31, Oct 31, Dec 31, Mar 31: a halving lands on Feb 28 or 29, Apr 30, Jun 30, Sep 30), leap days
//! (2024-02-29, and 2100 that is no leap year, 2400 that is), the epoch itself and the last millisecond of a day; each one's first 48
//! halvings and four far ones; every halving's last millisecond before it, its own, the one after and a day after; and 64 instants a
//! seeded walk picks over 300 years. Every value is an integer of milliseconds below 2^53, exact in JavaScript's numbers.
use sha2::Digest;
use warm_mine::law::{civil_of, days_of, era_at, halving_at, month_days, warm_nanos, DAY_MS, HALVING_MONTHS, NANOS_PER_MICRO, WARM_PER_USD};

const TABLE: &str = include_str!("law-parity.json");
const ROUTERS_LAW: &[u8] = include_bytes!("../../../../embertide-app/ui/intro/dyson-society/app/space-router-law.mjs");

/// The rows of `key` in the table: arrays of unsigned integers, read with no float and no dependency.
fn rows(key: &str) -> Vec<Vec<u64>> {
    // past `"key":[`, the array's own bracket: then each row is one `[...]`
    let at = TABLE.find(&format!("\"{key}\":[")).unwrap_or_else(|| panic!("the table has no {key}")) + key.len() + 4;
    let b = TABLE.as_bytes();
    let (mut i, mut out) = (at, Vec::new());
    loop {
        while b[i].is_ascii_whitespace() || b[i] == b',' {
            i += 1;
        }
        match b[i] {
            b']' => return out,
            b'[' => {
                i += 1;
                let mut row = Vec::new();
                loop {
                    let s = i;
                    while b[i].is_ascii_digit() {
                        i += 1;
                    }
                    row.push(TABLE[s..i].parse::<u64>().unwrap_or_else(|_| panic!("{key}: not an unsigned integer at {s}")));
                    match b[i] {
                        b',' => i += 1,
                        b']' => {
                            i += 1;
                            break;
                        }
                        c => panic!("{key}: {:?} at {i}", c as char),
                    }
                }
                out.push(row);
            }
            c => panic!("{key}: {:?} at {i}", c as char),
        }
    }
}
/// A string or a number field of the table, as its text.
fn field(key: &str) -> String {
    let at = TABLE.find(&format!("\"{key}\":")).unwrap_or_else(|| panic!("the table has no {key}")) + key.len() + 3;
    let rest = &TABLE[at..];
    let end = rest.find([',', '}']).unwrap();
    rest[..end].trim_matches('"').to_string()
}

#[test]
fn the_table_is_of_this_law_and_its_constants() {
    let hex: String = sha2::Sha256::digest(ROUTERS_LAW).iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(field("law"), "embertide-app/ui/intro/dyson-society/app/space-router-law.mjs");
    assert_eq!(hex, field("law_sha256"), "the router's law changed since the table was made: make the table again (the command at the head of this file)");
    assert_eq!(field("WARM_PER_USD").parse::<u64>().unwrap(), WARM_PER_USD);
    assert_eq!(field("HALVING_MONTHS").parse::<u64>().unwrap(), HALVING_MONTHS);
}

#[test]
fn every_halving_is_the_routers_to_the_millisecond() {
    let table = rows("halvings");
    assert!(table.len() >= 500, "{} halvings", table.len());
    for r in &table {
        let (o, n, want) = (r[0], r[1], r[2]);
        assert_eq!(halving_at(o, n), Some(want), "halvingAt({o}, {n})");
    }
    eprintln!("law: {} halvings, each the router's to the millisecond", table.len());
}

#[test]
fn every_era_is_the_routers() {
    let table = rows("eras");
    assert!(table.len() >= 2_500, "{} eras", table.len());
    let most = table.iter().map(|r| r[2]).max().unwrap();
    for r in &table {
        let (o, at, want) = (r[0], r[1], r[2]);
        assert_eq!(era_at(at, o), want, "eraAt({at}, {o})");
    }
    assert!(most >= 500, "the table reaches far eras (its farthest: {most})");
    eprintln!("law: {} eras, each the router's (up to era {most})", table.len());
}

/// What a spend mines, to the nano. The chain's is `usd_micros × 1,000 >> era`, the brief's formula, which floors; the router's
/// `warmMined` rounds its float to the nearest nano. Where the law's exact amount is a whole number of nanos the two are one number; where
/// it holds a fraction of a nano (from the fourth halving on, an amount not a multiple of the halving's power of two) the chain mines the
/// whole nanos and the router may say one more. Never anything else.
#[test]
fn what_a_spend_mines_is_the_routers_to_the_nano() {
    let table = rows("mined");
    let (mut same, mut one_less) = (0, 0);
    for r in &table {
        let (usd, at, o, router) = (r[0], r[1], r[2], r[3] as u128);
        let era = era_at(at, o);
        let chain = warm_nanos(usd, at, o);
        let whole = usd as u128 * NANOS_PER_MICRO as u128;
        assert_eq!(chain, whole >> era, "the chain's formula at era {era}");
        if chain == router {
            same += 1;
        } else {
            assert_eq!(chain + 1, router, "{usd} micros at {at} (era {era}): the chain {chain}, the router {router}");
            assert!(era >= 4 && whole % (1u128 << era) != 0, "only a fraction of a nano rounds: {usd} at era {era}");
            one_less += 1;
        }
    }
    assert!(table.len() >= 500);
    eprintln!("law: {} spends: {same} the router's to the nano, {one_less} one nano under its rounding (a fraction of a nano, from the fourth halving)", table.len());
}

#[test]
fn a_dollar_mines_one_warm_at_the_opening_and_before_it_and_halves_by_date() {
    // Aug 31 2026 12:34:56.789 UTC: its first halving is Feb 28 2027, its second Aug 31 2027, its fourth Aug 31 2028, its third Feb 29 2028
    let o = 1_788_179_696_789u64;
    assert_eq!(civil_of(o / DAY_MS), (2026, 8, 31));
    assert_eq!(warm_nanos(1_000_000, o, o), 1_000_000_000, "a dollar at the opening mines one $WARM");
    assert_eq!(warm_nanos(1_000_000, o - 365 * DAY_MS, o), 1_000_000_000, "before the opening, the opening's rate");
    let h1 = halving_at(o, 1).unwrap();
    assert_eq!((civil_of(h1 / DAY_MS), h1 % DAY_MS), ((2027, 2, 28), o % DAY_MS), "Aug 31 + 6 months is Feb 28, at the opening's time of day");
    assert_eq!(civil_of(halving_at(o, 3).unwrap() / DAY_MS), (2028, 2, 29), "a leap year's February ends on the 29th");
    assert_eq!(civil_of(halving_at(o, 2).unwrap() / DAY_MS), (2027, 8, 31), "always counted from the opening, never from the halving before");
    assert_eq!((warm_nanos(1_000_000, h1 - 1, o), warm_nanos(1_000_000, h1, o)), (1_000_000_000, 500_000_000), "the halving's own millisecond is the next era's");
    for n in 0..80u64 {
        let h = halving_at(o, n).unwrap();
        assert!(halving_at(o, n + 1).unwrap() > h, "the halvings rise");
        assert_eq!(era_at(h, o), n);
        assert_eq!(era_at(h - 1, o), n.saturating_sub(1));
        assert_eq!(warm_nanos(1_000_000, h, o), 1_000_000_000u128 >> n, "era {n}: a dollar mines 1 / 2^{n} $WARM");
    }
    // uncapped, forever: the law never refuses a spend; past the 33rd halving a dollar's $WARM is under a nano, and a large spend still mines
    assert_eq!(warm_nanos(u64::MAX, halving_at(o, 127).unwrap(), o), (u64::MAX as u128 * 1_000) >> 127);
    assert_eq!(warm_nanos(u64::MAX, halving_at(o, 128).unwrap(), o), 0, "a shift past the integer mines nothing and never wraps");
}

#[test]
fn the_calendar_is_the_gregorian_one() {
    // every day from 1970 to 2500 goes to its date and back, and the month lengths are the calendar's
    let mut prev = (1969, 12, 31);
    for days in 0..days_of(2500, 1, 1).unwrap() {
        let (y, m, d) = civil_of(days);
        assert_eq!(days_of(y, m, d), Some(days));
        assert!(d >= 1 && d <= month_days(y, m) && (1..=12).contains(&m));
        let next_of_prev = if prev.2 < month_days(prev.0, prev.1) { (prev.0, prev.1, prev.2 + 1) } else if prev.1 < 12 { (prev.0, prev.1 + 1, 1) } else { (prev.0 + 1, 1, 1) };
        assert_eq!((y, m, d), next_of_prev, "the day after {prev:?}");
        prev = (y, m, d);
    }
    assert_eq!((month_days(2024, 2), month_days(2100, 2), month_days(2400, 2), month_days(2026, 2)), (29, 28, 29, 28));
    // past what a u64 counts in milliseconds, there is no halving (an instant no chain reaches), and the era stops counting
    assert_eq!(halving_at(u64::MAX - 1, 1), None);
    assert_eq!(halving_at(0, u64::MAX), None);
}
