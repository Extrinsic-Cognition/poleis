//! THE GAME IS ONE SWARM (Nov 24 2026, his "Yes, pay them", to "Pay each house's creator its 0.30% and the publisher its 0.95%, as pump.fun
//! does?"): what a house owes whoever launched it, and what a generation's fee account holds for the publisher, each pushed out by anyone,
//! never inside a trade (a wallet under rent would fail every trade that paid it).

use crate::error::WarsError;
use crate::state::{Config, Fees, House};
use crate::util::{expect_writable, move_lamports, rent_of, Data};
use pinocchio::{error::ProgramError, AccountView, ProgramResult};

/// data: none · accounts: [house (w), creator (w)]
/// Anyone: the whole of what the house owes its creator (`creator_owed`, the creator's fee of every trade into it) goes from the house to
/// its creator's address, which the account passed must be (`InvalidAccount` for another); on any house, a rekt one too. Refused when
/// nothing is owed (`NothingOwed`). A creator whose wallet stands empty is paid once what the house owes him reaches a wallet's rent, or
/// once he puts SOL in it (the chain refuses an account under its rent): the house keeps it for him till then, and no trade waits on it.
pub fn pay_creator(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [house, creator] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(house)?;
    expect_writable(creator)?;
    let mut h = House::read_pda(house)?;
    if creator.address() != &h.creator {
        return Err(WarsError::InvalidAccount.into());
    }
    if h.creator_owed == 0 {
        return Err(WarsError::NothingOwed.into());
    }
    move_lamports(house, creator, h.creator_owed)?;
    h.creator_owed = 0;
    h.write(house)
}

/// data: none · accounts: [config, fees (w), treasury (w)]
/// Anyone: everything the generation's fee account holds beyond its rent goes to the generation's treasury, the publisher's (on poleis.us the
/// game swarm's Poleis treasury, whose own `sync_endowment` then books it), which the account passed must be (`InvalidAccount` for another);
/// `paid` counts it. What came to the fee account's address since its birth by no trade (anyone's transfer, or a rent the cluster lowered) is
/// booked as `donated` first and paid on with the fees: anyone trues the book, as Poleis's own `sync_endowment` does, and no gift can stop
/// the payout. Refused when it holds nothing beyond its rent (`NothingOwed`), or less than its book says (`MathOverflow`: nothing is paid on
/// a book that does not hold; a transfer of the difference mends it).
pub fn pay_publisher(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [config, fees, treasury] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_writable(fees)?;
    expect_writable(treasury)?;
    let c = Config::read_pda(config)?;
    let mut f = Fees::read_named(fees, &c.fees)?;
    if treasury.address() != &c.treasury {
        return Err(WarsError::InvalidAccount.into());
    }
    let beyond = fees.lamports().checked_sub(rent_of(Fees::SIZE)?).ok_or(WarsError::MathOverflow)?;
    // what it holds beyond its book came by no trade: a gift, booked as donated (never under the book: that would be lamports gone)
    let gift = beyond.checked_sub(f.total()?).ok_or(WarsError::MathOverflow)?;
    f.donated = f.donated.checked_add(gift).ok_or(WarsError::MathOverflow)?;
    if beyond == 0 {
        return Err(WarsError::NothingOwed.into());
    }
    move_lamports(fees, treasury, beyond)?;
    f.paid = f.paid.checked_add(beyond).ok_or(WarsError::MathOverflow)?;
    f.write(fees)
}
