//! The hexagon seat sale: a uniform-price sealed sale of N identical seats.
//! Buy as many as you like, one bid per seat, one nonce each; every bid
//! posts the same deposit. The top N revealed prices win and all pay the
//! (N+1)th price, floored at the reserve; proceeds go to the unit's vault.
//! An unclaimed seat is unsworn: it does not count until claimed.

use crate::error::PoleisError;
use crate::state::{Registry, SaleHead, SeatBid, SeatSale, Treasury};
use crate::util::{
    close_to, create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda,
    move_lamports, sha256v, Data,
};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};

/// Create and initialize a seat sale account at its PDA.
pub fn open_sale(payer: &AccountView, sale: &mut AccountView, mut head: SaleHead) -> Result<(), ProgramError> {
    if head.seat_count == 0 || head.seat_count > crate::MAX_SEATS_PER_SALE {
        return Err(PoleisError::InvalidAmount.into());
    }
    if head.reserve == 0 || head.deposit < head.reserve {
        return Err(PoleisError::InvalidAmount.into());
    }
    let unit_le = head.unit_id.to_le_bytes();
    let sale_le = head.sale_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"sale", &unit_le, &sale_le];
    let bump = find_and_expect_pda(sale, seeds)?;
    create_pda(payer, sale, SeatSale::size(head.seat_count), seeds, bump)?;
    head.bump = bump;
    SeatSale::init(sale, &head)
}

pub fn check_windows(now: u64, commit_end: u64, reveal_end: u64) -> Result<(), ProgramError> {
    if commit_end <= now || reveal_end <= commit_end {
        return Err(PoleisError::InvalidWindow.into());
    }
    if commit_end - now > crate::MAX_WINDOW_SLOTS || reveal_end - commit_end > crate::MAX_WINDOW_SLOTS {
        return Err(PoleisError::WindowTooLong.into());
    }
    Ok(())
}

fn read_sale(sale: &AccountView, unit_id: u64, sale_id: u64) -> Result<SaleHead, ProgramError> {
    let s = SeatSale::head(sale)?;
    if s.unit_id != unit_id || s.sale_id != sale_id {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(sale, &[b"sale", &unit_id.to_le_bytes(), &sale_id.to_le_bytes()], s.bump)?;
    Ok(s)
}

fn read_bid(bid: &AccountView, unit_id: u64, sale_id: u64, bidder: &Address, nonce: u64) -> Result<SeatBid, ProgramError> {
    let b = SeatBid::read(bid)?;
    if b.unit_id != unit_id || b.sale_id != sale_id || &b.bidder != bidder || b.nonce != nonce {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(
        bid,
        &[b"sbid", &unit_id.to_le_bytes(), &sale_id.to_le_bytes(), bidder.as_ref(), &nonce.to_le_bytes()],
        b.bump,
    )?;
    Ok(b)
}

/// data: [unit:8][sale:8][nonce:8][commitment:32]
/// accounts: [bidder (s,w), sale, bid (w), system]
pub fn commit_seat_bid(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [bidder, sale, bid, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(56)?;
    let unit_id = d.u64(0)?;
    let sale_id = d.u64(8)?;
    let nonce = d.u64(16)?;
    let commitment = d.h32(24)?;
    expect_signer(bidder)?;

    let s = read_sale(sale, unit_id, sale_id)?;
    if Clock::get()?.slot >= s.commit_end_slot {
        return Err(PoleisError::CommitWindowClosed.into());
    }

    let bidder_key = *bidder.address();
    let unit_le = unit_id.to_le_bytes();
    let sale_le = sale_id.to_le_bytes();
    let nonce_le = nonce.to_le_bytes();
    let seeds: &[&[u8]] = &[b"sbid", &unit_le, &sale_le, bidder_key.as_ref(), &nonce_le];
    let bump = find_and_expect_pda(bid, seeds)?;
    create_pda(bidder, bid, SeatBid::SIZE, seeds, bump)?;
    SeatBid {
        unit_id,
        sale_id,
        bidder: bidder_key,
        nonce,
        commitment,
        deposit: s.deposit,
        price: 0,
        revealed: false,
        done: false,
        bump,
    }
    .write(bid)?;

    pinocchio_system::instructions::Transfer { from: bidder, to: bid, lamports: s.deposit }.invoke()
}

/// data: [unit:8][sale:8][nonce:8][price:8][salt:32]
/// accounts: [bidder (s), sale (w), bid (w)]
/// commitment = sha256(price ‖ nonce ‖ salt ‖ bidder).
pub fn reveal_seat_bid(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [bidder, sale, bid] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(64)?;
    let unit_id = d.u64(0)?;
    let sale_id = d.u64(8)?;
    let nonce = d.u64(16)?;
    let price = d.u64(24)?;
    let salt = d.h32(32)?;
    expect_signer(bidder)?;
    expect_writable(sale)?;
    expect_writable(bid)?;

    let s = read_sale(sale, unit_id, sale_id)?;
    let now = Clock::get()?.slot;
    if now < s.commit_end_slot {
        return Err(PoleisError::RevealWindowNotOpen.into());
    }
    if now >= s.reveal_end_slot {
        return Err(PoleisError::RevealWindowClosed.into());
    }

    let bidder_key = *bidder.address();
    let mut b = read_bid(bid, unit_id, sale_id, &bidder_key, nonce)?;
    if b.revealed {
        return Err(PoleisError::AlreadyRevealed.into());
    }
    let expected = sha256v(&[&price.to_le_bytes(), &nonce.to_le_bytes(), &salt, bidder_key.as_ref()]);
    if expected != b.commitment {
        return Err(PoleisError::BadReveal.into());
    }
    if price == 0 || price > b.deposit {
        return Err(PoleisError::BidExceedsDeposit.into());
    }
    if price < s.reserve {
        return Err(PoleisError::InvalidAmount.into());
    }

    SeatSale::insert_reveal(sale, price, bidder_key.as_array(), nonce)?;
    b.revealed = true;
    b.price = price;
    b.write(bid)
}

/// data: [unit:8][sale:8] · accounts: [crank (s), sale (w), registry (w)]
/// Permissionless. The uniform price is the highest losing bid when the
/// board filled, the reserve otherwise; the registry remembers it as the
/// seat's last clearing price, which is what a jetted holder is paid.
pub fn settle_seats(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, sale, registry] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(16)?;
    let unit_id = d.u64(0)?;
    let sale_id = d.u64(8)?;
    expect_signer(crank)?;
    expect_writable(sale)?;
    expect_writable(registry)?;

    let s = read_sale(sale, unit_id, sale_id)?;
    if Clock::get()?.slot < s.reveal_end_slot {
        return Err(PoleisError::RevealWindowNotOpen.into());
    }
    if s.settled {
        return Err(PoleisError::AlreadySettled.into());
    }
    let clearing = if s.filled == s.seat_count { s.overflow_price.max(s.reserve) } else { s.reserve };
    SeatSale::set_settled(sale, clearing)?;

    let mut head = super::read_registry(registry, unit_id)?;
    head.last_clearing = clearing;
    Registry::set_head(registry, &head)
}

/// data: [unit:8][sale:8][nonce:8]
/// accounts: [bidder (s,w), sale, bid (w), registry (w), treasury (w), system, fund (w), tally (w)]
/// A winner takes the seat: pays the uniform price into the unit's vault,
/// less the fund's share, takes the rest of the escrow back, and is written
/// into the roll active.
pub fn claim_seat(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [bidder, sale, bid, registry, treasury, _system, fund, tally] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(24)?;
    let unit_id = d.u64(0)?;
    let sale_id = d.u64(8)?;
    let nonce = d.u64(16)?;
    expect_signer(bidder)?;
    expect_writable(bidder)?;
    expect_writable(bid)?;
    expect_writable(registry)?;
    expect_writable(treasury)?;

    let s = read_sale(sale, unit_id, sale_id)?;
    if !s.settled {
        return Err(PoleisError::SaleNotSettled.into());
    }
    let bidder_key = *bidder.address();
    let mut b = read_bid(bid, unit_id, sale_id, &bidder_key, nonce)?;
    if b.done {
        return Err(PoleisError::AlreadyClaimed.into());
    }
    if !b.revealed || !SeatSale::is_winner(sale, bidder_key.as_array(), nonce)? {
        return Err(PoleisError::NotWinner.into());
    }

    let t = Treasury::read(treasury)?;
    if t.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    expect_pda(treasury, &[b"treasury", &unit_id.to_le_bytes()], t.bump)?;
    // a unit dissolved while its sale ran takes no new holder and no new
    // lamports: the winners reclaim their whole escrow instead
    let head = super::read_registry(registry, unit_id)?;
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }

    // the seat first: a new holder may grow the registry by a row, which is
    // a system transfer from the bidder, and a CPI must precede any lamports
    // this program moves by hand in the same instruction
    let now = Clock::get()?.slot;
    Registry::add_seats(registry, bidder, crate::HOLDER_KEY, bidder_key.as_array(), 1, now)?;

    let price = s.clearing;
    // the fund's share, then the rest to the treasury (the fee's own CPI, if any, precedes the hand moves)
    let fee = super::fund::charge_fee(fund, tally, bidder, unit_id, head.root, price)?;
    if fee > 0 {
        move_lamports(bid, fund, fee)?;
    }
    move_lamports(bid, treasury, price - fee)?;
    Treasury::add_in(treasury, price - fee)?;
    let refund = b.deposit.saturating_sub(price);
    if refund > 0 {
        move_lamports(bid, bidder, refund)?;
    }
    b.done = true;
    b.write(bid)
}

/// data: [unit:8][sale:8][nonce:8] · accounts: [bidder (s,w), sale, bid (w), registry?]
/// Losers take their escrow back after settlement; unrevealed bids may walk
/// once the window is dead. Winners do not reclaim, winners claim: unless
/// the unit was dissolved while the sale ran, which the winner proves by
/// passing the frozen registry, and then the whole escrow walks too.
pub fn reclaim_seat_bid(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() < 3 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (core, extra) = accounts.split_at_mut(3);
    let [bidder, sale, bid] = core else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(24)?;
    let unit_id = d.u64(0)?;
    let sale_id = d.u64(8)?;
    let nonce = d.u64(16)?;
    expect_signer(bidder)?;
    expect_writable(bidder)?;
    expect_writable(bid)?;

    let s = read_sale(sale, unit_id, sale_id)?;
    let bidder_key = *bidder.address();
    let mut b = read_bid(bid, unit_id, sale_id, &bidder_key, nonce)?;
    if b.done {
        return Err(PoleisError::NothingToReclaim.into());
    }
    if s.settled {
        if b.revealed && SeatSale::is_winner(sale, bidder_key.as_array(), nonce)? {
            let dissolved = match extra.first() {
                Some(registry) => super::read_registry(registry, unit_id)?.frozen,
                None => false,
            };
            if !dissolved {
                return Err(PoleisError::WinnerCannotReclaim.into());
            }
        }
    } else if Clock::get()?.slot < s.reveal_end_slot || b.revealed {
        return Err(PoleisError::NotSettled.into());
    }

    move_lamports(bid, bidder, b.deposit)?;
    b.done = true;
    b.write(bid)
}

/// data: [unit:8][sale:8][nonce:8] · accounts: [bid (w), bidder (w)]
/// Permissionless rent recovery: a spent escrow (claimed or reclaimed)
/// closes and its rent goes back to the bidder.
pub fn close_seat_bid(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [bid, bidder] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(24)?;
    let unit_id = d.u64(0)?;
    let sale_id = d.u64(8)?;
    let nonce = d.u64(16)?;
    expect_writable(bid)?;
    expect_writable(bidder)?;
    let bidder_key = *bidder.address();
    let b = read_bid(bid, unit_id, sale_id, &bidder_key, nonce)?;
    if !b.done {
        return Err(PoleisError::NotClosable.into());
    }
    close_to(bid, bidder)
}
