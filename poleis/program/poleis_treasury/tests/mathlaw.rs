//! Host-side property sweep of the dividend share law. No SVM, no binary:
//! this mirrors the exact on-chain formula and hammers it across the whole
//! u64 domain, hundreds of thousands of random partitions per run.
//!
//! The claim being defended: because every claimant's weight is a term of
//! the total weight they divide by,
//!
//!   share_i = ⌊amount · w_i / total⌋   with   Σ w_i = total
//!
//! satisfies share_i ≤ amount (so the u128 quotient always fits u64 and the
//! cast can never wrap) and Σ share_i ≤ amount (a distribution can never be
//! over-dissipated). A holder carrying ALL the weight takes exactly the
//! amount, nothing lost. The same algebra prices a jet (seats × clearing)
//! and a unit's share of a dissolution.

const ROUNDS: usize = 500_000;
const SEED: u64 = 0x0715_1204_2026_0901;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// The exact formula from claim_dividend, including the checked cast belt.
fn share(amount: u64, weight: u64, total: u64) -> u64 {
    let wide = (amount as u128) * (weight as u128) / (total as u128);
    u64::try_from(wide).expect("belt violated: quotient above u64 with weight <= total")
}

#[test]
fn share_law_holds_across_the_domain() {
    let mut rng = Rng(SEED);
    for round in 0..ROUNDS {
        // extreme-biased sampling: full-range values 1 time in 4, else small
        let amount = if rng.next().is_multiple_of(4) { rng.next() } else { rng.next() % 1_000_000_000 };
        let total = (if rng.next().is_multiple_of(4) { rng.next() } else { rng.next() % 100_000 }).max(1);

        // partition total into 1..=8 random parts (zeros allowed and skipped:
        // a zero-weight holder is unexpressible on-chain anyway)
        let k = (rng.next() % 8 + 1) as usize;
        let mut cuts: Vec<u64> = (0..k - 1).map(|_| rng.next() % total).collect();
        cuts.sort_unstable();
        cuts.push(total);
        let mut prev = 0u64;
        let mut paid: u128 = 0;
        for cut in cuts {
            let h = cut - prev;
            prev = cut;
            if h == 0 {
                continue;
            }
            let s = share(amount, h, total);
            assert!(
                s <= amount,
                "round {round}: share {s} above amount {amount} (w={h}, total={total})"
            );
            if h == total {
                assert_eq!(s, amount, "round {round}: the sole holder must take exactly the amount");
            }
            paid += s as u128;
        }
        assert!(
            paid <= amount as u128,
            "round {round}: partition over-dissipated {paid} of {amount} (total={total})"
        );
    }
}

#[test]
fn share_law_edge_cases_exact() {
    // the corners, by hand
    assert_eq!(share(u64::MAX, u64::MAX, u64::MAX), u64::MAX); // everything to one
    assert_eq!(share(u64::MAX, 1, u64::MAX), 1); // one weight of everything: one lamport
    assert_eq!(share(1u64 << 63, 5, 5), 1u64 << 63); // the old F1 shape, exact
    assert_eq!(share(0, 1, 1), 0); // zero distribution pays zero (refused earlier on-chain)
    assert_eq!(share(400, 1, 1000), 0); // the dust law
    assert_eq!(share(400, 999, 1000), 399); // dust stays home
}

// ---- the SUM: a split settled by the two halves (his Mar 22 2026 rule) ----
// Unlike the share law above, this calls the program's own const fn: the
// law is pure, so the host runs the very code the chain runs.

use poleis_treasury::{chair_quorum, sum_passes};

/// The AND of the two chambers, as `seal` judges a session with a floor.
fn and_passes(pent_pro: u8, lit: u8, hex_pro: u64, active: u64) -> bool {
    pent_pro >= chair_quorum(lit) && (hex_pro as u128) * 2 > active as u128
}

#[test]
fn the_sum_passes_his_two_examples() {
    // "30% pentagons say yes, 21% hexagons say yes": six of ten chairs with 42 % of the floor
    assert!(sum_passes(6, 10, 42, 100));
    // "5/12 pentagons vote yes but 80% hexagons voted yes, it passes"
    assert!(sum_passes(5, 12, 80, 100));
    // and neither passes the AND: the SUM is what carries them
    assert!(!and_passes(6, 10, 42, 100));
    assert!(!and_passes(5, 12, 80, 100));
}

#[test]
fn whatever_passes_the_and_passes_the_sum_and_no_majority_never_does() {
    for lit in 1..=12u8 {
        for pent_pro in 0..=lit {
            for active in 1..=60u64 {
                for hex_pro in 0..=active {
                    let sum = sum_passes(pent_pro, lit, hex_pro, active);
                    if and_passes(pent_pro, lit, hex_pro, active) {
                        assert!(sum, "the AND passes but the SUM does not: {pent_pro}/{lit} chairs, {hex_pro}/{active} floor");
                    }
                    let pent_major = pent_pro >= chair_quorum(lit);
                    let hex_major = hex_pro * 2 > active;
                    if !pent_major && !hex_major {
                        assert!(!sum, "neither chamber has its majority but the SUM passes: {pent_pro}/{lit}, {hex_pro}/{active}");
                    }
                }
            }
        }
    }
}

#[test]
fn a_unanimous_chamber_cannot_carry_a_chamber_under_a_third() {
    assert!(!sum_passes(12, 12, 33, 100), "every chair with a third of the floor less one");
    assert!(!sum_passes(3, 12, 100, 100), "the whole floor with a quarter of the chairs");
    // at a third exactly, the chamber no longer blocks alone
    assert!(sum_passes(12, 12, 34, 100));
    assert!(sum_passes(4, 12, 100, 100));
}

#[test]
fn one_point_under_fifty_one_does_not_pass() {
    // six of ten chairs: 100 × (6 × 100 + h × 10) ≥ 102 × 10 × 100 asks h ≥ 42
    assert!(sum_passes(6, 10, 42, 100));
    assert!(!sum_passes(6, 10, 41, 100));
    // seven of twelve: 100 × (7 × 100 + h × 12) ≥ 102 × 12 × 100 asks h ≥ 44
    assert!(sum_passes(7, 12, 44, 100));
    assert!(!sum_passes(7, 12, 43, 100));
}

#[test]
fn the_sum_is_monotone_in_either_chamber() {
    for lit in 1..=12u8 {
        for pent_pro in 0..=lit {
            for active in 1..=60u64 {
                for hex_pro in 0..=active {
                    if !sum_passes(pent_pro, lit, hex_pro, active) {
                        continue;
                    }
                    if pent_pro < lit {
                        assert!(sum_passes(pent_pro + 1, lit, hex_pro, active), "one more chair broke a pass: {pent_pro}/{lit}, {hex_pro}/{active}");
                    }
                    if hex_pro < active {
                        assert!(sum_passes(pent_pro, lit, hex_pro + 1, active), "more floor broke a pass: {pent_pro}/{lit}, {hex_pro}/{active}");
                    }
                }
            }
        }
    }
}

#[test]
fn no_chairs_or_no_floor_never_passes_the_sum() {
    assert!(!sum_passes(0, 0, 100, 100));
    assert!(!sum_passes(12, 12, 0, 0));
    assert!(!sum_passes(0, 0, 0, 0));
    assert!(!sum_passes(5, 0, u64::MAX, 1));
    // the arithmetic is wide: the largest weights never overflow
    assert!(sum_passes(12, 12, u64::MAX, u64::MAX));
}

#[test]
fn only_four_kinds_can_ever_take_the_sum() {
    use poleis_treasury::{
        sum_bit, CLASS_ACT, CLASS_CODEX, CLASS_SEAT, CLASS_SIGNAL, CLASS_TREASURY, SIGNAL_OPEN_AUCTION, SIGNAL_RULE, SUM_BUDGET,
        SUM_KINDS, SUM_OPEN_AUCTION, SUM_RULE, SUM_TRANSFER, TREASURY_BUDGET, TREASURY_TRANSFER,
    };
    let granted = [
        (CLASS_SIGNAL, SIGNAL_OPEN_AUCTION, SUM_OPEN_AUCTION),
        (CLASS_SIGNAL, SIGNAL_RULE, SUM_RULE),
        (CLASS_TREASURY, TREASURY_TRANSFER, SUM_TRANSFER),
        (CLASS_TREASURY, TREASURY_BUDGET, SUM_BUDGET),
    ];
    let mut all = 0u8;
    for class in 0..=u8::MAX {
        for kind in 0..=u8::MAX {
            let bit = sum_bit(class, kind);
            match granted.iter().find(|g| g.0 == class && g.1 == kind) {
                Some(g) => {
                    assert_eq!(bit, g.2);
                    all |= bit;
                }
                None => assert_eq!(bit, 0, "class {class} kind {kind} can never take the SUM"),
            }
        }
    }
    assert_eq!(all, SUM_KINDS, "the four bits are the whole mask");
    // the law, a seat and the hands never: whatever their kind
    for kind in 0..=u8::MAX {
        assert_eq!(sum_bit(CLASS_CODEX, kind) | sum_bit(CLASS_SEAT, kind) | sum_bit(CLASS_ACT, kind), 0);
    }
}

// ================= a verdict reaches the seal: the market's half of the ruling =================

#[test]
fn only_three_classes_may_ask_the_market_and_two_may_be_plans() {
    use poleis_treasury::{
        market_bit, CLASS_ACT, CLASS_CODEX, CLASS_SEAT, CLASS_SIGNAL, CLASS_TREASURY, MARKET_ACT, MARKET_CLASSES, MARKET_SEAT,
        MARKET_TREASURY, PLAN_CLASSES,
    };
    let mut all = 0u8;
    for class in 0..=u8::MAX {
        let bit = market_bit(class);
        let want = match class {
            CLASS_TREASURY => MARKET_TREASURY,
            CLASS_SEAT => MARKET_SEAT,
            CLASS_ACT => MARKET_ACT,
            _ => 0,
        };
        assert_eq!(bit, want, "class {class}");
        assert!(bit.count_ones() <= 1);
        all |= bit;
    }
    assert_eq!(all, MARKET_CLASSES, "three classes, three bits, the whole mask");
    assert_eq!((market_bit(CLASS_CODEX), market_bit(CLASS_SIGNAL)), (0, 0), "the law and the signals stay under the AND for ever");
    assert_eq!(PLAN_CLASSES, MARKET_TREASURY | MARKET_ACT, "a plan is money or the hands");
    assert_eq!(market_bit(CLASS_SEAT) & PLAN_CLASSES, 0, "a seat is never decided with no ballot of the floor");
}

#[test]
fn the_four_bits_never_collide_and_every_reader_strips_them() {
    use poleis_treasury::{class_of, state::Proposal, CLASS_ACT, CLASS_BITS, CLASS_CONSENT, CLASS_MARKET, CLASS_PLAN, CLASS_SUM, MAX_IMPACTED};
    let bits = [CLASS_CONSENT, CLASS_SUM, CLASS_MARKET, CLASS_PLAN];
    for (i, a) in bits.iter().enumerate() {
        assert_eq!(a.count_ones(), 1);
        assert!(*a > CLASS_ACT && a & 0x0F == 0, "a bit is above every class");
        for b in &bits[i + 1..] {
            assert_eq!(a & b, 0, "{a:#x} and {b:#x} collide");
        }
    }
    assert_eq!(CLASS_BITS, 0xF0);
    for class in 0..=CLASS_ACT {
        for set in 0u8..16 {
            let mut byte = class;
            for (k, b) in bits.iter().enumerate() {
                if set & (1 << k) != 0 {
                    byte |= b;
                }
            }
            assert_eq!(class_of(byte), class);
            let p = Proposal {
                hash: [1u8; 32],
                class: byte,
                origin: 0,
                manifest: [0u8; 32],
                payload_hash: [0u8; 32],
                opened: 0,
                expires: 0,
                executed: false,
                count: 1,
                bump: 0,
                impacted: [0u64; MAX_IMPACTED],
                payer: [0u8; 32],
                payload_len: 2,
            };
            assert_eq!(p.class_of(), class, "byte {byte:#x}");
            assert_eq!((p.consent(), p.sum(), p.market(), p.plan()), (set & 1 != 0, set & 2 != 0, set & 4 != 0, set & 8 != 0));
        }
    }
}

#[test]
fn the_bar_is_a_share_of_the_board_at_the_basis_point() {
    use poleis_treasury::stood_enough;
    // a tenth of a board of ten thousand: a thousand counts, one less does not
    assert!(stood_enough(1_000, 10_000, 1_000));
    assert!(!stood_enough(999, 10_000, 1_000));
    // no bar: any road, even one nobody stood in
    assert!(stood_enough(0, u64::MAX, 0));
    // the whole board, and the widest numbers do not overflow
    assert!(stood_enough(u64::MAX, u64::MAX, 10_000));
    assert!(!stood_enough(u64::MAX - 1, u64::MAX, 10_000));
    assert!(stood_enough(1, 10_000, 1));
    assert!(!stood_enough(0, 10_000, 1));
}

/// The roads program's source, read as text: no dependency, no lock change.
const ROADS_STATE: &str = include_str!("../../poleis_roads/src/state.rs");

/// The right-hand side of `pub const <name>: ... = <value>;` in the roads program's state, the first after `anchor` (the impl block
/// it belongs to, or the file's head), and inside that block.
fn roads_const(anchor: &str, name: &str) -> String {
    let start = if anchor.is_empty() { 0 } else { ROADS_STATE.find(anchor).unwrap_or_else(|| panic!("the roads program has no `{anchor}`")) };
    let key = format!("pub const {name}:");
    let at = start + ROADS_STATE[start..].find(&key).unwrap_or_else(|| panic!("the roads program has no `{name}` after `{anchor}`"));
    assert!(!ROADS_STATE[start + anchor.len()..at].contains("\nimpl "), "`{name}` is not in `{anchor}`");
    let line = &ROADS_STATE[at..at + ROADS_STATE[at..].find(';').expect("a const ends")];
    line.split_once('=').expect("a const has a value").1.trim().to_string()
}

#[test]
fn the_pin_every_offset_poleis_reads_is_the_roads_programs_own() {
    use poleis_treasury::roads as r;
    let numbers: [(&str, &str, &str, usize); 20] = [
        ("LAYOUT_VERSION", "", "LAYOUT_VERSION", r::LAYOUT_VERSION as usize),
        ("OFF_DISC", "", "OFF_DISC", r::OFF_DISC),
        ("OFF_VERSION", "", "OFF_VERSION", r::OFF_VERSION),
        ("DISC_ROAD", "", "DISC_ROAD", r::DISC_ROAD as usize),
        ("VERDICT_OPEN", "", "VERDICT_OPEN", r::VERDICT_OPEN as usize),
        ("VERDICT_PASSED", "", "VERDICT_PASSED", r::VERDICT_PASSED as usize),
        ("VERDICT_FAILED", "", "VERDICT_FAILED", r::VERDICT_FAILED as usize),
        ("ROAD_OFF_BOARD", "impl Road {", "OFF_BOARD", r::ROAD_OFF_BOARD),
        ("ROAD_OFF_ROAD_ID", "impl Road {", "OFF_ROAD_ID", r::ROAD_OFF_ROAD_ID),
        ("ROAD_OFF_VERDICT", "impl Road {", "OFF_VERDICT", r::ROAD_OFF_VERDICT),
        ("ROAD_OFF_OPENS", "impl Road {", "OFF_OPENS", r::ROAD_OFF_OPENS),
        ("ROAD_OFF_CLOSES", "impl Road {", "OFF_CLOSES", r::ROAD_OFF_CLOSES),
        ("ROAD_OFF_DEPOSITS", "impl Road {", "OFF_DEPOSITS", r::ROAD_OFF_DEPOSITS),
        // a road's id is 32 bytes and its verdict follows it: the face never moves
        ("ROAD_OFF_CREDITED_AT_LAY", "impl Road {", "OFF_CREDITED_AT_LAY", r::ROAD_OFF_CREDITED_AT_LAY),
        ("ROAD_OFF_ROAD_ID + 32", "impl Road {", "OFF_VERDICT", r::ROAD_OFF_ROAD_ID + 32),
        ("DISC_BOARD", "", "DISC_BOARD", r::DISC_BOARD as usize),
        ("BOARD_OFF_BOARD", "impl Board {", "OFF_BOARD", r::BOARD_OFF_BOARD),
        ("BOARD_OFF_AUTHORITY", "impl Board {", "OFF_AUTHORITY", r::BOARD_OFF_AUTHORITY),
        ("BOARD_OFF_POLIS", "impl Board {", "OFF_POLIS", r::BOARD_OFF_POLIS),
        ("BOARD_OFF_UNIT", "impl Board {", "OFF_UNIT", r::BOARD_OFF_UNIT),
    ];
    for (ours, anchor, theirs, value) in numbers {
        let text = roads_const(anchor, theirs);
        let read: usize = text.parse().unwrap_or_else(|_| panic!("`{theirs}` is `{text}`, not a number"));
        assert_eq!(value, read, "Poleis's roads::{ours} is {value}, the roads program's {anchor} {theirs} is {read}");
    }
    for (ours, theirs, value) in [("SEED_ROAD", "SEED_ROAD", r::SEED_ROAD), ("SEED_BOARD", "SEED_BOARD", r::SEED_BOARD)] {
        let text = roads_const("", theirs);
        let seed = text.strip_prefix("b\"").and_then(|t| t.strip_suffix('"')).unwrap_or_else(|| panic!("`{theirs}` is `{text}`"));
        assert_eq!(value, seed.as_bytes(), "Poleis's roads::{ours} is not the roads program's {theirs}");
    }
    // the fields read are u64s inside the account the roads program lays
    let road_size: usize = roads_const("impl Road {", "SIZE").parse().unwrap();
    let board_size: usize = roads_const("impl Board {", "SIZE").parse().unwrap();
    assert!(r::BOARD_OFF_UNIT + 8 <= board_size && r::BOARD_OFF_POLIS + 32 == r::BOARD_OFF_UNIT);
    assert!(r::ROAD_OFF_CREDITED_AT_LAY + 8 <= road_size && r::ROAD_OFF_OPENS + 8 == r::ROAD_OFF_CLOSES);
    let deposits_after: usize = roads_const("impl Road {", "OFF_POSITIONS").parse().unwrap();
    assert_eq!(r::ROAD_OFF_DEPOSITS, deposits_after + 4, "deposits stand after the u32 of positions");
}
