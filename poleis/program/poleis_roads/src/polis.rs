//! Reading the polis's program: the few bytes of a Proposal and of a Session that `lay_for` reads, by owner, discriminator, version and
//! offset, never by a dependency on the treasury crate (the mirror of Poleis's own `src/roads.rs`, which reads a road the same way). Every
//! constant here is a copy of Poleis's own (`poleis/program/poleis_treasury/src/state.rs` and `src/lib.rs`), its name there beside it, and a
//! test reads those files as text and holds each one equal to it (`tests/pin.rs`): if Poleis moves a field, it is red and names it.
//!
//! A reader checks the owner program first, then the discriminator and the VERSION, and only then trusts an offset of that version; a
//! length is checked "at least up to the field read", never as an exact size. A closed proposal shrinks to a tombstone of 34 bytes (its
//! discriminator, version and hash): too short to hold `opened`, it is not a live proposal.

use crate::error::RoadsError;
use pinocchio::{error::ProgramError, AccountView, Address};

/// `LAYOUT_VERSION` (Poleis's state.rs)
pub const LAYOUT_VERSION: u8 = 8;
/// `DISC_PROPOSAL`
pub const DISC_PROPOSAL: u8 = 8;
/// `DISC_SESSION`
pub const DISC_SESSION: u8 = 9;
/// `Proposal::OFF_HASH`
pub const PROPOSAL_OFF_HASH: usize = 2;
/// `Proposal::OFF_CLASS`
pub const PROPOSAL_OFF_CLASS: usize = 34;
/// `Proposal::OFF_ORIGIN`
pub const PROPOSAL_OFF_ORIGIN: usize = 35;
/// `Proposal::OFF_OPENED`
pub const PROPOSAL_OFF_OPENED: usize = 107;
/// `Session::OFF_HASH`
pub const SESSION_OFF_HASH: usize = 2;
/// `Session::OFF_UNIT`
pub const SESSION_OFF_UNIT: usize = 34;
/// `Session::OFF_DEADLINE`
pub const SESSION_OFF_DEADLINE: usize = 42;
/// `Session::OFF_STATE`
pub const SESSION_OFF_STATE: usize = 79;
/// `STATE_VOTING` (Poleis's lib.rs)
pub const STATE_VOTING: u8 = 0;
/// `CLASS_MARKET` (Poleis's lib.rs)
pub const CLASS_MARKET: u8 = 0x20;
/// `CLASS_PLAN` (Poleis's lib.rs)
pub const CLASS_PLAN: u8 = 0x10;

#[inline(always)]
fn rd_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

/// What `lay_for` needs of a proposal.
pub struct ProposalFace {
    pub hash: [u8; 32],
    pub class: u8,
    pub origin: u64,
    pub opened: u64,
}
/// What `lay_for` needs of a session.
pub struct SessionFace {
    pub hash: [u8; 32],
    pub unit: u64,
    pub deadline: u64,
    pub state: u8,
}

/// The owner, then the discriminator and the version, then the length up to `end`: an account of `polis` of that kind and this version.
fn face<'a>(acc: &'a AccountView, polis: &Address, disc: u8, end: usize) -> Result<pinocchio::account::Ref<'a, [u8]>, ProgramError> {
    if !acc.owned_by(polis) {
        return Err(RoadsError::InvalidAccount.into());
    }
    let d = acc.try_borrow()?;
    if d.len() <= 1 || d[0] != disc {
        return Err(RoadsError::InvalidAccount.into());
    }
    if d[1] != LAYOUT_VERSION {
        return Err(RoadsError::LayoutVersion.into());
    }
    if d.len() < end {
        return Err(RoadsError::InvalidAccount.into());
    }
    Ok(d)
}

/// A live proposal of `polis` (a tombstone is too short to hold `opened`).
pub fn proposal(acc: &AccountView, polis: &Address) -> Result<ProposalFace, ProgramError> {
    let d = face(acc, polis, DISC_PROPOSAL, PROPOSAL_OFF_OPENED + 8)?;
    Ok(ProposalFace {
        hash: d[PROPOSAL_OFF_HASH..PROPOSAL_OFF_HASH + 32].try_into().unwrap(),
        class: d[PROPOSAL_OFF_CLASS],
        origin: rd_u64(&d, PROPOSAL_OFF_ORIGIN),
        opened: rd_u64(&d, PROPOSAL_OFF_OPENED),
    })
}

/// A session of `polis`.
pub fn session(acc: &AccountView, polis: &Address) -> Result<SessionFace, ProgramError> {
    let d = face(acc, polis, DISC_SESSION, SESSION_OFF_STATE + 1)?;
    Ok(SessionFace {
        hash: d[SESSION_OFF_HASH..SESSION_OFF_HASH + 32].try_into().unwrap(),
        unit: rd_u64(&d, SESSION_OFF_UNIT),
        deadline: rd_u64(&d, SESSION_OFF_DEADLINE),
        state: d[SESSION_OFF_STATE],
    })
}
