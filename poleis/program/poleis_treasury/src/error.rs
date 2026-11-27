use pinocchio::error::ProgramError;

/// Every refusal the program can utter, one name each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PoleisError {
    Unauthorized = 0,
    InvalidPda = 1,
    InvalidAccount = 2,
    InvalidAmount = 3,
    InvalidWindow = 4,
    CommitWindowClosed = 5,
    RevealWindowNotOpen = 6,
    RevealWindowClosed = 7,
    AlreadyRevealed = 8,
    BadReveal = 9,
    BidExceedsDeposit = 10,
    AlreadySettled = 11,
    NotSettled = 12,
    NothingToReclaim = 13,
    WinnerCannotReclaim = 14,
    TreasuryInsufficient = 15,
    UnitMismatch = 16,
    MissingWinningBid = 17,
    ShareIsZero = 18,
    MathOverflow = 19,
    WindowTooLong = 20,
    LayoutVersion = 21,
    InvalidMode = 22,
    RegistryFull = 23,
    NotHolder = 24,
    SeatLimit = 25,
    SaleNotSettled = 26,
    NotWinner = 27,
    AlreadyClaimed = 28,
    InvalidHolder = 29,
    ChairTaken = 30,
    ChairEmpty = 31,
    NotOnLadder = 32,
    NotMember = 33,
    NotImpacted = 34,
    NotDescendant = 35,
    SessionClosed = 36,
    SessionOpen = 37,
    ChildMustSpeak = 38,
    NotSealed = 39,
    NotPassed = 40,
    AlreadyExecuted = 41,
    ClassMismatch = 42,
    BadPayload = 43,
    ClosureBroken = 44,
    PlanBreached = 45,
    BadPrecompile = 46,
    Frozen = 47,
    AlreadyMandated = 48,
    TooMany = 49,
    SeatsResting = 50,
    NotClosable = 51,
    ActForbidden = 52,
    TokenPlanBreached = 53,
    /// A pull past the allowance's cap for this period.
    AllowanceBreached = 54,
    /// An allowance revoked or past its expiry.
    AllowanceDead = 55,
    /// A ruling on a contract by a unit that is neither its payer nor its arbiter.
    NotArbiter = 56,
    /// A contract already released or refunded.
    ContractSettled = 57,
    /// A contract still inside its deadline and not resigned by its payee.
    ContractOpen = 58,
    /// A purchase larger than what the listing offers.
    ListingShort = 59,
    /// An accrual or a closing asked for before its time.
    NotYet = 60,
    /// A spend in a mint the society has not allowed (the default needs no listing).
    MintNotAllowed = 61,
    /// A purchase at a price the listing no longer offers: the buyer names the
    /// price they agreed to, and a listing cancelled and relisted dearer under
    /// the same nonce cannot take it.
    PriceChanged = 62,
    /// A stake or a backing on an option the root polis has not vetted.
    NotVetted = 63,
    /// A root with no chair in the fund this cycle, or fees that win none.
    NoChair = 64,
    /// The fund's cycle has not run its slots yet.
    CycleOpen = 65,
    /// An option or a recipient that is not the fund's current one.
    NotCurrent = 66,
    /// A knock at a polis with no open spot, or an admission past the last one.
    NoSpots = 67,
    /// A key already knocking at this polis.
    KnockOpen = 68,
    /// An admission, a refusal or a withdrawal of a key that is not knocking.
    NotKnocking = 69,
    /// A claim of a grant the fund does not owe to that account.
    NothingOwed = 70,
    /// A BLS point the syscalls refuse: an x past the prime, the larger root, a point off the curve, a flagged encoding.
    BadBlsPoint = 71,
    /// A BLS aggregate or proof of possession the pairing refuses.
    BadBlsSignature = 72,
    /// The proposal asks for the SUM where it cannot have it: the kind can never take it, the origin's law has not turned it on for that kind, a treasury sum is over the origin's small cap, or consent was asked with it.
    SumNotAllowed = 73,
    /// The proposal asks for the market where it cannot have it: the class can never ask, more than one of the class byte's four bits,
    /// no law of the origin's own, or a class the law does not name for that mode (`split` for the market, `plan` for a plan).
    MarketNotAllowed = 74,
    /// The proposal's road is laid and its verdict is still open, where that verdict could change the seal: decide the road first.
    MarketOpen = 75,
    /// A `CODEX_MARKET` act that turns the market on names a board that is not the unit's own: not the named program's board of that
    /// number at its own address, its authority not the unit's agent key, or its law not naming this program and this unit.
    MarketBoardNotOurs = 76,
    /// A growth of the fund asked of an account that is not the older fund: another size (a fund already grown among them), another
    /// discriminator or layout byte, or old fields a constitution of the fund could never have written.
    NotAnOlderFund = 77,
}

impl From<PoleisError> for ProgramError {
    fn from(e: PoleisError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
