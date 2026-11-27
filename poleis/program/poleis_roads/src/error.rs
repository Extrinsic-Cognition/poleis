use pinocchio::error::ProgramError;

/// Every refusal the program can utter, one name each. The numbers are the
/// program's public face (`ProgramError::Custom(n)`): a new refusal takes a
/// new number at the end, and none is ever renumbered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RoadsError {
    /// A signer who is not the board's authority, crediter or the road's judge, or not the owner a position or balance names.
    Unauthorized = 0,
    /// An account that is not the PDA its seeds and stored bump say.
    InvalidPda = 1,
    /// An account of another owner, size or discriminator, not writable where it must be, or not the one another account names.
    InvalidAccount = 2,
    /// An account of this program written under another layout version.
    LayoutVersion = 3,
    /// An amount of nothing, or a deposit the law cannot carry (`deposit_check`).
    InvalidAmount = 4,
    /// Terms the law refuses (`Terms::check`), a road opening before the clock or more than the law's longest window after it, or a judge's
    /// deadline not after the close or more than the law's longest window after it.
    InvalidTerms = 5,
    /// A sum that would leave its integer.
    MathOverflow = 6,
    /// `init_board` on a board that stands.
    BoardStands = 7,
    /// `lay` on a road that stands.
    RoadStands = 8,
    /// A second position of one owner on one road.
    AlreadyDeposited = 9,
    /// A deposit, or a staker's stake, over the balance's free amount.
    NotEnoughBalance = 10,
    /// A trade outside the road's window (the law's OUTSIDE).
    RoadNotOpen = 11,
    /// A trade stamped before the last trade the road took (the law's BACKWARDS).
    Backwards = 12,
    /// A deposit at or after the close, or a trade on a decided road.
    RoadClosed = 13,
    /// The road has more than `MOST` boundaries still to walk to the clock (an observation chasing a price that moved far and was then left; a
    /// stretch at rest is never walked): send `carry` first.
    RoadBehind = 14,
    /// `decide` before the close.
    NotClosed = 15,
    /// `decide` on a road already decided.
    AlreadyDecided = 16,
    /// Receipts on a road with no verdict yet.
    NotDecided = 17,
    /// Receipts a second time.
    AlreadySaid = 18,
    /// Receipts at or after the road's `judge_by_ms`.
    TooLate = 19,
    /// Receipts of the world that did not happen (the law's OTHER_WORLD).
    OtherWorld = 20,
    /// A worth over one $WARM a claim (the law's WORTH_RANGE).
    WorthRange = 21,
    /// A settle before the judge spoke and before the road is void.
    NotSettleable = 22,
    /// The law's NOT_ENOUGH: the purse holds less than the trade's amount.
    NotEnough = 23,
    /// The law's NOTHING_OUT: too small to take anything out.
    NothingOut = 24,
    /// The law's PAST_MIN: the price moved past `min_out`.
    PastMin = 25,
    /// The law's OUT_OF_RANGE: the trade would leave the law's range.
    OutOfRange = 26,
    /// A refusal of the law this program does not name (none is known: a new one of the law's lands here until it is named).
    LawRefused = 27,
    /// `collect` before the close, or while a position stands in the road.
    NotCollectable = 28,
    /// `collect` a second time.
    AlreadyCollected = 29,
    /// `lay_for` on a board whose law names no polis: it lays nothing by rule.
    NoLaw = 30,
    /// `lay_for` for a proposal whose class byte asks neither the market nor a plan.
    NotAsked = 31,
    /// `lay_for` for a proposal, or with a session, of another unit than the board's law names.
    OtherUnit = 32,
    /// `lay_for` with a session no longer voting: its vote is sealed.
    VoteClosed = 33,
}

impl From<RoadsError> for ProgramError {
    fn from(e: RoadsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// The law's refusal of a trade or a time, as this program's error: compared with the law's own constants, never with a copy of their text.
pub fn of_law(why: &'static str) -> RoadsError {
    use poleis_futarchy::law::{BACKWARDS, NOTHING_OUT, NOT_ENOUGH, OTHER_WORLD, OUTSIDE, OUT_OF_RANGE, PAST_MIN, STILL_OPEN, WORTH_RANGE};
    if why == NOT_ENOUGH {
        RoadsError::NotEnough
    } else if why == NOTHING_OUT {
        RoadsError::NothingOut
    } else if why == PAST_MIN {
        RoadsError::PastMin
    } else if why == OUT_OF_RANGE {
        RoadsError::OutOfRange
    } else if why == OUTSIDE {
        RoadsError::RoadNotOpen
    } else if why == BACKWARDS {
        RoadsError::Backwards
    } else if why == OTHER_WORLD {
        RoadsError::OtherWorld
    } else if why == WORTH_RANGE {
        RoadsError::WorthRange
    } else if why == STILL_OPEN {
        RoadsError::NotDecided
    } else {
        RoadsError::LawRefused
    }
}
