use pinocchio::error::ProgramError;

/// Every refusal the program can utter, one name each. The numbers are the program's public face (`ProgramError::Custom(n)`): a new
/// refusal takes a new number at the end, and none is ever renumbered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum WarmError {
    /// A signer who is not the program's upgrade authority (at `init`), its authority, its miner, or the flask an account belongs to.
    Unauthorized = 0,
    /// An account that is not the PDA its seeds and stored bump say.
    InvalidPda = 1,
    /// An account of another owner, size, discriminator, mint or holder, not writable where it must be, or not the one the mine names.
    InvalidAccount = 2,
    /// An account of this program written under another layout version.
    LayoutVersion = 3,
    /// An amount of nothing: a spend, a pay-back or a prior deposit of zero.
    InvalidAmount = 4,
    /// A sum that would leave its integer (a $WARM amount past what the token program counts among them).
    MathOverflow = 5,
    /// `init` on a mine that stands: it is opened once.
    MineStands = 6,
    /// A deposit under the flask minimum, 10 USDC (his, Nov 19 2026 12:33).
    UnderMinimum = 7,
    /// A spend whose `seq` is not above every `seq` mined before it.
    SeqNotRising = 8,
    /// A spend stamped earlier than the last spend mined.
    Backwards = 9,
    /// A spend stamped later than the chain's clock.
    Future = 10,
    /// THE BOUND: a spend that would take a flask's `spent` past its `deposited`. No key mines a dollar no flask put in.
    PastDeposits = 11,
    /// A prior deposit whose transaction signature is counted already, for any flask.
    PriorCounted = 12,
    /// A treasury whose mint is not of six decimals, as a dollar is, or not the dollar the mine was opened with.
    NotADollar = 13,
}

impl From<WarmError> for ProgramError {
    fn from(e: WarmError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
