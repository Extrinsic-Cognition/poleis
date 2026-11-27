use pinocchio::error::ProgramError;

/// Every refusal the program can utter, one name each. The numbers are the program's public face (`ProgramError::Custom(n)`): a new refusal
/// takes a new number at the end, and none is ever renumbered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum WarsError {
    /// A signer who is not the owner a position names, or not the config's authority.
    Unauthorized = 0,
    /// An account that is not the PDA its seeds and stored bump say.
    InvalidPda = 1,
    /// An account of another owner, size or discriminator, not writable where it must be, or not the one another account names.
    InvalidAccount = 2,
    /// An account of this program written under another layout version.
    LayoutVersion = 3,
    /// An amount of nothing.
    InvalidAmount = 4,
    /// Rules the law refuses (`Rules::check`), or a minute of no second.
    InvalidRules = 5,
    /// A sum that would leave its integer.
    MathOverflow = 6,
    /// `init_config` on a config that stands.
    ConfigStands = 7,
    /// `init_house` on a house that stands.
    HouseStands = 8,
    /// The law's NOT_ENOUGH_TOKENS: a buy past the coins the curve has left.
    NotEnoughTokens = 9,
    /// The law's NOTHING_OUT, or a buy that buys no coin.
    NothingOut = 10,
    /// The price moved past the trade's least.
    PastMin = 11,
    /// A sale or a stake of more coins than the position holds free of a stake.
    NotEnoughCoins = 12,
    /// A sale or a stake on a house in a raid: a raid's ways out are `desert` and `change_sides`, and one battle at a time.
    InRaid = 13,
    /// A house that lost its raid: nothing trades it any more.
    Rekt = 14,
    /// A raid on a house still under its shield (younger than the config's shield age, or worth less than its shield market cap).
    Shielded = 15,
    /// The law's UNFAIR: the opponent is not between half and twice the attacker's market cap.
    Unfair = 16,
    /// The law's STAKE_CAP: the stake is past the most that may be staked.
    StakeCap = 17,
    /// A position whose stake is in another raid, or an attacker that proposes another raid.
    OtherRaid = 18,
    /// A stake, or a stake taken back, on a raid that is no longer a proposal (open, in a stand, decided or settled).
    RaidOpen = 19,
    /// A desert, a change of sides or a buy that names a raid the house is not in.
    NotInRaid = 20,
    /// A trade of a raid after its last minute or after its decision: crank it.
    RaidOver = 21,
    /// A staker deserting or changing sides: his coins are locked in the raid.
    Staked = 22,
    /// A crank of a minute already read.
    Cranked = 23,
    /// A crank of a raid still a proposal.
    NotOpen = 24,
    /// A crank of a raid already decided (the law's DECIDED).
    Decided = 25,
    /// A settle before the raid is decided.
    NotDecided = 26,
    /// A settle a second time.
    AlreadySettled = 27,
    /// A convert or an unlock before the settlement.
    NotSettled = 28,
    /// A convert of a position that is not of the house that lost.
    NotTheLoser = 29,
    /// An unlock when the raid's attacker lost: its stakers convert.
    NotTheWinner = 30,
    /// A desert, a change of sides or a convert of a position that holds no coin.
    NothingHeld = 31,
    /// An unlock, or a stake taken back, of a position that staked nothing in this raid.
    NotStaked = 32,
    /// The law's RESERVE_SHORT: the house's reserve cannot pay that sale.
    ReserveShort = 33,
    /// A raid of a house on itself, or a change of sides into the same house.
    SameHouse = 34,
    /// A refusal of the law this program does not name (none is known: a new one of the law's lands here until it is named).
    LawRefused = 35,
    /// The law's NOT_SOLD: more coins than the curve ever sold.
    NotSold = 36,
    /// Houses of two configs.
    OtherConfig = 37,
    /// A `pay_creator` of a house that owes its creator nothing, or a `pay_publisher` of a fee account that holds nothing beyond its rent.
    NothingOwed = 38,
    /// A generation founded naming no treasury: its publisher's fees would have nowhere to go.
    NoTreasury = 39,
    OrdersDue = 40,
    MarchFull = 41,
    MarchDisabled = 42,
    OrderTooSmall = 43,
    OwnerMarchFull = 44,
    PositionHasCoins = 45,
    PositionHasQuote = 46,
    PositionOrderPending = 47,
}

impl From<WarsError> for ProgramError {
    fn from(e: WarsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// The law's refusal as this program's error: compared with the law's own constants, never with a copy of their text.
pub fn of_law(why: &'static str) -> WarsError {
    use embertide_war::law::{
        DECIDED, NOTHING_OUT, NOT_ENOUGH_TOKENS, NOT_HELD, NOT_SOLD, ORDER_TOO_SMALL, OVERFLOW, OWNER_MARCH_FULL,
        RESERVE_SHORT, STAKE_CAP, UNFAIR,
    };
    if why == NOT_ENOUGH_TOKENS {
        WarsError::NotEnoughTokens
    } else if why == NOTHING_OUT {
        WarsError::NothingOut
    } else if why == RESERVE_SHORT {
        WarsError::ReserveShort
    } else if why == NOT_SOLD {
        WarsError::NotSold
    } else if why == UNFAIR {
        WarsError::Unfair
    } else if why == STAKE_CAP {
        WarsError::StakeCap
    } else if why == DECIDED {
        WarsError::Decided
    } else if why == OVERFLOW {
        WarsError::MathOverflow
    } else if why == NOT_HELD {
        WarsError::NotEnoughCoins
    } else if why == ORDER_TOO_SMALL {
        WarsError::OrderTooSmall
    } else if why == OWNER_MARCH_FULL {
        WarsError::OwnerMarchFull
    } else {
        WarsError::LawRefused
    }
}
