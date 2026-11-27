//! The seat market. A holder lists seats at a price; anyone buys some or
//! all; the lamports go straight to the seller and the seats move exactly
//! as a transfer would, resting rule included. The listing escrows nothing:
//! it is an offer, checked against the registry at the moment of purchase.

use super::council::move_seats;
use crate::error::PoleisError;
use crate::state::{Listing, Registry};
use crate::util::{close_to, create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda, receipt, Data};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, ProgramResult,
};

fn read_listing(acc: &AccountView, unit: u64, seller: &[u8; 32], nonce: u64) -> Result<Listing, ProgramError> {
    let l = Listing::read(acc)?;
    if l.unit_id != unit || l.seller != *seller || l.nonce != nonce {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(acc, &[b"listing", &unit.to_le_bytes(), seller, &nonce.to_le_bytes()], l.bump)?;
    Ok(l)
}

/// data: [unit:8][nonce:8][count:8][price:8]
/// accounts: [seller (s,w), registry, listing (w), system]
pub fn list_seats(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [seller, registry, listing, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(32)?;
    let unit = d.u64(0)?;
    let nonce = d.u64(8)?;
    let count = d.u64(16)?;
    let price = d.u64(24)?;
    expect_signer(seller)?;
    if count == 0 || price == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    let head = super::read_registry(registry, unit)?;
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }
    let key: [u8; 32] = *seller.address().as_array();
    let i = Registry::find_holder(registry, crate::HOLDER_KEY, &key)?.ok_or(PoleisError::NotHolder)?;
    if Registry::holder(registry, i)?.seats < count {
        return Err(PoleisError::SeatLimit.into());
    }
    let unit_le = unit.to_le_bytes();
    let nonce_le = nonce.to_le_bytes();
    let seeds: &[&[u8]] = &[b"listing", &unit_le, &key, &nonce_le];
    let bump = find_and_expect_pda(listing, seeds)?;
    create_pda(seller, listing, Listing::SIZE, seeds, bump)?;
    Listing { unit_id: unit, seller: key, nonce, count, price, bump }.write(listing)
}

/// data: [unit:8][nonce:8][count:8]
/// accounts: [buyer (s,w), seller (w), registry (w), listing (w), system]
/// data: [unit:8][nonce:8][count:8][price:8] · accounts: [buyer (s,w), seller (w), registry (w), listing (w), system, fund (w), tally (w)]
/// The buyer pays count × price to the seller and receives the seats. The
/// buyer names the price: a listing cancelled and relisted dearer under the
/// same nonce, landed ahead of the purchase, refuses instead of charging. A
/// seller who no longer holds them, or whose seats are resting, cannot
/// sell; the listing survives the refusal.
pub fn buy_seats(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [buyer, seller, registry, listing, _system, fund, tally] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(32)?;
    let unit = d.u64(0)?;
    let nonce = d.u64(8)?;
    let count = d.u64(16)?;
    let price = d.u64(24)?;
    expect_signer(buyer)?;
    expect_writable(seller)?;
    expect_writable(registry)?;
    expect_writable(listing)?;
    if count == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    let seller_key: [u8; 32] = *seller.address().as_array();
    let mut l = read_listing(listing, unit, &seller_key, nonce)?;
    if count > l.count {
        return Err(PoleisError::ListingShort.into());
    }
    if l.price != price {
        return Err(PoleisError::PriceChanged.into());
    }
    let head = super::read_registry(registry, unit)?;
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }
    let total = count.checked_mul(l.price).ok_or(PoleisError::MathOverflow)?;
    let now = Clock::get()?.slot;
    let buyer_key: [u8; 32] = *buyer.address().as_array();
    let fee = super::fund::charge_fee(fund, tally, buyer, unit, head.root, total)?;
    move_seats(registry, buyer, &seller_key, &buyer_key, crate::HOLDER_KEY, count, head.window, now)?;
    pinocchio_system::instructions::Transfer { from: buyer, to: seller, lamports: total - fee }.invoke()?;
    if fee > 0 {
        pinocchio_system::instructions::Transfer { from: buyer, to: fund, lamports: fee }.invoke()?;
    }
    let listing_key: [u8; 32] = *listing.address().as_array();
    receipt(crate::RCPT_SEATS, unit, &listing_key, &buyer_key, &[0u8; 32], total, 0);
    l.count -= count;
    if l.count == 0 {
        close_to(listing, seller)
    } else {
        l.write(listing)
    }
}

/// data: [unit:8][nonce:8] · accounts: [seller (s,w), listing (w)]
pub fn cancel_listing(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [seller, listing] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(16)?;
    let unit = d.u64(0)?;
    let nonce = d.u64(8)?;
    expect_signer(seller)?;
    expect_writable(seller)?;
    expect_writable(listing)?;
    let key: [u8; 32] = *seller.address().as_array();
    read_listing(listing, unit, &key, nonce)?;
    close_to(listing, seller)
}
