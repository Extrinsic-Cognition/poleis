//! THE PIN: every constant of `src/polis.rs` is a copy of Poleis's own, read here from Poleis's source as text (no dependency, no lock
//! change), by name. If Poleis moves a field of a proposal or a session, or renumbers a discriminator, a state or a bit, this test is red
//! and names it.
use poleis_roads::polis as p;

const STATE: &str = include_str!("../../poleis_treasury/src/state.rs");
const LIB: &str = include_str!("../../poleis_treasury/src/lib.rs");

/// The right-hand side of `pub const <name>: ... = <value>;` in `src`, the first after `anchor` (an impl block, or the file's head), and
/// inside that block.
fn poleis_const(src: &str, anchor: &str, name: &str) -> String {
    let start = if anchor.is_empty() { 0 } else { src.find(anchor).unwrap_or_else(|| panic!("Poleis has no `{anchor}`")) };
    let key = format!("pub const {name}:");
    let at = start + src[start..].find(&key).unwrap_or_else(|| panic!("Poleis has no `{name}` after `{anchor}`"));
    assert!(!src[start + anchor.len()..at].contains("\nimpl "), "`{name}` is not in `{anchor}`");
    let line = &src[at..at + src[at..].find(';').expect("a const ends")];
    line.split_once('=').expect("a const has a value").1.trim().to_string()
}
fn number(text: &str) -> u64 {
    match text.strip_prefix("0x") {
        Some(hex) => u64::from_str_radix(hex, 16).unwrap_or_else(|_| panic!("`{text}` is not a number")),
        None => text.parse().unwrap_or_else(|_| panic!("`{text}` is not a number")),
    }
}

#[test]
fn the_pin_every_offset_the_roads_program_reads_is_poleiss_own() {
    let table: [(&str, &str, &str, &str, u64); 14] = [
        ("LAYOUT_VERSION", STATE, "", "LAYOUT_VERSION", p::LAYOUT_VERSION as u64),
        ("DISC_PROPOSAL", STATE, "", "DISC_PROPOSAL", p::DISC_PROPOSAL as u64),
        ("DISC_SESSION", STATE, "", "DISC_SESSION", p::DISC_SESSION as u64),
        ("PROPOSAL_OFF_HASH", STATE, "impl Proposal {", "OFF_HASH", p::PROPOSAL_OFF_HASH as u64),
        ("PROPOSAL_OFF_CLASS", STATE, "impl Proposal {", "OFF_CLASS", p::PROPOSAL_OFF_CLASS as u64),
        ("PROPOSAL_OFF_ORIGIN", STATE, "impl Proposal {", "OFF_ORIGIN", p::PROPOSAL_OFF_ORIGIN as u64),
        ("PROPOSAL_OFF_OPENED", STATE, "impl Proposal {", "OFF_OPENED", p::PROPOSAL_OFF_OPENED as u64),
        ("SESSION_OFF_HASH", STATE, "impl Session {", "OFF_HASH", p::SESSION_OFF_HASH as u64),
        ("SESSION_OFF_UNIT", STATE, "impl Session {", "OFF_UNIT", p::SESSION_OFF_UNIT as u64),
        ("SESSION_OFF_DEADLINE", STATE, "impl Session {", "OFF_DEADLINE", p::SESSION_OFF_DEADLINE as u64),
        ("SESSION_OFF_STATE", STATE, "impl Session {", "OFF_STATE", p::SESSION_OFF_STATE as u64),
        ("STATE_VOTING", LIB, "", "STATE_VOTING", p::STATE_VOTING as u64),
        ("CLASS_MARKET", LIB, "", "CLASS_MARKET", p::CLASS_MARKET as u64),
        ("CLASS_PLAN", LIB, "", "CLASS_PLAN", p::CLASS_PLAN as u64),
    ];
    for (ours, src, anchor, theirs, value) in table {
        let read = number(&poleis_const(src, anchor, theirs));
        assert_eq!(value, read, "the roads program's polis::{ours} is {value}, Poleis's {anchor} {theirs} is {read}");
    }
    // the tombstone a closed proposal shrinks to is too short to hold `opened`: a reader that asks for it refuses a closed proposal
    let tomb = number(&poleis_const(STATE, "impl Proposal {", "TOMBSTONE_SIZE"));
    assert!(tomb < (p::PROPOSAL_OFF_OPENED + 8) as u64, "a tombstone of {tomb} bytes would read as a live proposal");
}
