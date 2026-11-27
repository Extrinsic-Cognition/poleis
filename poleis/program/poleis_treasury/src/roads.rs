//! Reading the market's program: the few bytes of a Road that Poleis's `seal` reads, and of a Board that its `CODEX_MARKET` act reads, by owner, discriminator, version
//! and offset, never by a dependency on the roads crate (a market's bug must never sit in the contract that guards a treasury). Every
//! constant here is a copy of the roads program's own (`poleis/program/poleis_roads/src/state.rs`), its name there beside it, and a test
//! reads that file as text and holds each one equal to it (`tests/mathlaw.rs`, the pin): if the roads program moves a field, it is red
//! and names it.
//!
//! A reader checks the owner program, the discriminator and the VERSION first, and only then trusts an offset of that version (the
//! roads program's THE PUBLIC FACE). A length is checked "at least up to the field read", never as an exact size: a field may be added
//! at a Road's end without a new version. Whatever cannot be read says nothing: it never blocks a seal.
//!
//! The seal reads the road alone, never its board: the bar is measured against what was in hands on the board when the road was laid
//! (`credited_at_lay`, which the road keeps: the board's `credited` less what its house had earned back), so a credit after the lay
//! cannot make a passed road stop counting. A road laid by an older build of the roads program, too short to hold that field, says
//! nothing.
//!
//! A ROAD IS LAID BY A RULE. A polis can only name a board that is its own (`board_is_ours`, read by the `CODEX_MARKET` act that turns
//! the market on): the board's authority is the unit's own agent key, and a board's authority is named once, so its law, its crediter,
//! its judge and every credit on it came only by sealed acts of the polis; and its law names this program and this unit, so any holder
//! with the stake free on that board lays the road of a proposal that asked (the roads program's `lay_for`), staking its pool from its
//! own balance, with the board's terms and the vote's own window. The program a polis's law names is checked only as the owner of that
//! board: a program that lays its boards' roads by other rules is the polis's own choice, made under the AND.

use pinocchio::{error::ProgramError, AccountView, Address};

/// `LAYOUT_VERSION`
pub const LAYOUT_VERSION: u8 = 1;
/// `OFF_DISC`
pub const OFF_DISC: usize = 0;
/// `OFF_VERSION`
pub const OFF_VERSION: usize = 1;
/// `DISC_BOARD`
pub const DISC_BOARD: u8 = 1;
/// `DISC_ROAD`
pub const DISC_ROAD: u8 = 3;
/// `SEED_BOARD`
pub const SEED_BOARD: &[u8] = b"board";
/// `Board::OFF_BOARD`
pub const BOARD_OFF_BOARD: usize = 2;
/// `Board::OFF_AUTHORITY`
pub const BOARD_OFF_AUTHORITY: usize = 10;
/// `Board::OFF_POLIS`
pub const BOARD_OFF_POLIS: usize = 114;
/// `Board::OFF_UNIT`
pub const BOARD_OFF_UNIT: usize = 146;
/// `SEED_ROAD`
pub const SEED_ROAD: &[u8] = b"road";
/// `VERDICT_OPEN`
pub const VERDICT_OPEN: u8 = 0;
/// `VERDICT_PASSED`
pub const VERDICT_PASSED: u8 = 1;
/// `VERDICT_FAILED`
pub const VERDICT_FAILED: u8 = 2;
/// `Road::OFF_BOARD`
pub const ROAD_OFF_BOARD: usize = 2;
/// `Road::OFF_ROAD_ID`
pub const ROAD_OFF_ROAD_ID: usize = 10;
/// `Road::OFF_VERDICT`
pub const ROAD_OFF_VERDICT: usize = 42;
/// `Road::OFF_OPENS`
pub const ROAD_OFF_OPENS: usize = 75;
/// `Road::OFF_CLOSES`
pub const ROAD_OFF_CLOSES: usize = 83;
/// `Road::OFF_DEPOSITS`
pub const ROAD_OFF_DEPOSITS: usize = 361;
/// `Road::OFF_CREDITED_AT_LAY`
pub const ROAD_OFF_CREDITED_AT_LAY: usize = 454;

/// What a road says to a seal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoadWord {
    /// Never laid (no data, another owner), not readable as this version's road, or its window not inside the vote: the market says
    /// nothing.
    Silent,
    /// Laid, its verdict still open: what stood in it so far, what was in hands on its board at its lay, and its close.
    Open { deposits: u64, credited_at_lay: u64, closes: u64 },
    /// Decided: passed or failed, the deposits that stood in it, and what was in hands on its board when it was laid.
    Decided { passed: bool, deposits: u64, credited_at_lay: u64 },
}

#[inline(always)]
fn rd_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

/// The road of `hash` on `board`, under `program`: `["road", board (u64 LE), hash]`.
pub fn road_address(program: &Address, board: u64, hash: &[u8; 32]) -> Address {
    Address::find_program_address(&[SEED_ROAD, &board.to_le_bytes(), hash], program).0
}

/// The word of the road at `acc`, whose address the caller has checked. The owner first, then the discriminator and the version, then
/// the length up to the last field read; the road must name the board and the id its address does. A ROAD'S WINDOW LIES INSIDE THE
/// VOTE: a road that opens before the proposal opened (`opened`: it was laid, and may have been decided, before the question was
/// public) or closes after the origin's session ends (`deadline`: it could hold the seal for as long as its window) says nothing. Both
/// programs count slots.
pub fn road_word(acc: &AccountView, program: &Address, board: u64, hash: &[u8; 32], opened: u64, deadline: u64) -> Result<RoadWord, ProgramError> {
    if !acc.owned_by(program) {
        return Ok(RoadWord::Silent);
    }
    let d = acc.try_borrow()?;
    if d.len() <= OFF_VERSION || d[OFF_DISC] != DISC_ROAD || d[OFF_VERSION] != LAYOUT_VERSION {
        return Ok(RoadWord::Silent);
    }
    if d.len() < ROAD_OFF_CREDITED_AT_LAY + 8 || rd_u64(&d, ROAD_OFF_BOARD) != board || d[ROAD_OFF_ROAD_ID..ROAD_OFF_ROAD_ID + 32] != hash[..] {
        return Ok(RoadWord::Silent);
    }
    if rd_u64(&d, ROAD_OFF_OPENS) < opened || rd_u64(&d, ROAD_OFF_CLOSES) > deadline {
        return Ok(RoadWord::Silent);
    }
    let (deposits, credited_at_lay) = (rd_u64(&d, ROAD_OFF_DEPOSITS), rd_u64(&d, ROAD_OFF_CREDITED_AT_LAY));
    Ok(match d[ROAD_OFF_VERDICT] {
        VERDICT_OPEN => RoadWord::Open { deposits, credited_at_lay, closes: rd_u64(&d, ROAD_OFF_CLOSES) },
        VERDICT_PASSED => RoadWord::Decided { passed: true, deposits, credited_at_lay },
        VERDICT_FAILED => RoadWord::Decided { passed: false, deposits, credited_at_lay },
        _ => RoadWord::Silent,
    })
}

/// The board `board` under `program`: `["board", board (u64 LE)]`.
pub fn board_address(program: &Address, board: u64) -> Address {
    Address::find_program_address(&[SEED_BOARD, &board.to_le_bytes()], program).0
}

/// Whether the account at `acc` is `program`'s board number `board` (at its own address, of this version), its authority `authority`
/// (the unit's own agent key), its law naming `polis` (this program) and `unit`. The owner first, then the discriminator and the version,
/// then the length up to the last field read.
pub fn board_is_ours(acc: &AccountView, program: &Address, board: u64, authority: &Address, polis: &Address, unit: u64) -> Result<bool, ProgramError> {
    if !acc.owned_by(program) || acc.address() != &board_address(program, board) {
        return Ok(false);
    }
    let d = acc.try_borrow()?;
    if d.len() <= OFF_VERSION || d[OFF_DISC] != DISC_BOARD || d[OFF_VERSION] != LAYOUT_VERSION || d.len() < BOARD_OFF_UNIT + 8 {
        return Ok(false);
    }
    Ok(rd_u64(&d, BOARD_OFF_BOARD) == board
        && d[BOARD_OFF_AUTHORITY..BOARD_OFF_AUTHORITY + 32] == authority.as_ref()[..]
        && d[BOARD_OFF_POLIS..BOARD_OFF_POLIS + 32] == polis.as_ref()[..]
        && rd_u64(&d, BOARD_OFF_UNIT) == unit)
}
