//! The sealed second-price trace auction. The lot is bound at open
//! (pool_root + manifest_hash): an auction that sells nothing cannot be
//! opened. Commitments bind the bidder's key, so they are non-transferable.
//! Escrow lives in the bid account itself. Settlement is permissionless law:
//! anyone may crank it once the reveal window closes, so no operator absence
//! can strand a deposit. Under a council a lot auction is opened by a
//! SIGNAL proposal; in solo mode by the authority.

use crate::error::PoleisError;
use crate::state::{Auction, Bid, Treasury};
use crate::util::{
    close_to, create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda, sha256v,
    Data,
};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};

pub struct LotParams {
    pub auction_id: u64,
    pub unit_id: u64,
    pub pool_root: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub commit_end_slot: u64,
    pub reveal_end_slot: u64,
    pub reserve_price: u64,
    pub deposit: u64,
}

impl LotParams {
    /// [auction:8][unit:8][pool_root:32][manifest:32][commit_end:8][reveal_end:8][reserve:8][deposit:8]
    pub const LEN: usize = 112;
    pub fn parse(d: &Data) -> Result<Self, ProgramError> {
        Ok(Self {
            auction_id: d.u64(0)?,
            unit_id: d.u64(8)?,
            pool_root: d.h32(16)?,
            manifest_hash: d.h32(48)?,
            commit_end_slot: d.u64(80)?,
            reveal_end_slot: d.u64(88)?,
            reserve_price: d.u64(96)?,
            deposit: d.u64(104)?,
        })
    }
}

/// Open a lot auction: the law shared by the solo authority and the council.
pub fn open_lot(
    payer: &AccountView,
    treasury: &AccountView,
    auction: &mut AccountView,
    p: &LotParams,
    now: u64,
) -> ProgramResult {
    // the lot is the law: no empty sale
    if p.pool_root == [0u8; 32] || p.manifest_hash == [0u8; 32] {
        return Err(PoleisError::InvalidAccount.into());
    }
    // a reserve of zero would hand the lot to a lone bidder for nothing
    if p.reserve_price == 0 || p.deposit < p.reserve_price {
        return Err(PoleisError::InvalidAmount.into());
    }
    if p.commit_end_slot <= now || p.reveal_end_slot <= p.commit_end_slot {
        return Err(PoleisError::InvalidWindow.into());
    }
    if p.commit_end_slot - now > crate::MAX_WINDOW_SLOTS
        || p.reveal_end_slot - p.commit_end_slot > crate::MAX_WINDOW_SLOTS
    {
        return Err(PoleisError::WindowTooLong.into());
    }

    let t = Treasury::read(treasury)?;
    if t.unit_id != p.unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    let unit_le = p.unit_id.to_le_bytes();
    expect_pda(treasury, &[b"treasury", &unit_le], t.bump)?;

    let auction_le = p.auction_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"auction", &auction_le];
    let bump = find_and_expect_pda(auction, seeds)?;
    create_pda(payer, auction, Auction::SIZE, seeds, bump)?;
    Auction {
        auction_id: p.auction_id,
        unit_id: p.unit_id,
        pool_root: p.pool_root,
        manifest_hash: p.manifest_hash,
        commit_end_slot: p.commit_end_slot,
        reveal_end_slot: p.reveal_end_slot,
        reserve_price: p.reserve_price,
        best_bid: 0,
        second_bid: 0,
        clearing_price: 0,
        best_bidder: Address::new_from_array([0u8; 32]),
        settled: false,
        bump,
        deposit: p.deposit,
    }
    .write(auction)
}

/// data: LotParams · accounts: [authority (s,w), config, treasury, auction (w), system]
/// Solo mode only; a council opens lots by SIGNAL proposal.
pub fn open_auction(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, config, treasury, auction, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(LotParams::LEN)?;
    let p = LotParams::parse(&d)?;
    super::expect_solo_authority(config, authority)?;
    open_lot(authority, treasury, auction, &p, Clock::get()?.slot)
}

/// data: [auction:8][commitment:32]
/// accounts: [bidder (s,w), auction, bid (w), system]
/// The escrow amount is not the bidder's to choose: every bid posts the
/// auction's uniform deposit, so escrow size leaks nothing about the bid.
pub fn commit_bid(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [bidder, auction, bid, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(40)?;
    let auction_id = d.u64(0)?;
    let commitment = d.h32(8)?;
    expect_signer(bidder)?;

    let a = Auction::read(auction)?;
    let deposit = a.deposit;
    if a.auction_id != auction_id {
        return Err(PoleisError::InvalidAccount.into());
    }
    let auction_le = auction_id.to_le_bytes();
    expect_pda(auction, &[b"auction", &auction_le], a.bump)?;
    if Clock::get()?.slot >= a.commit_end_slot {
        return Err(PoleisError::CommitWindowClosed.into());
    }

    let bidder_key = *bidder.address();
    let seeds: &[&[u8]] = &[b"bid", &auction_le, bidder_key.as_ref()];
    let bump = find_and_expect_pda(bid, seeds)?;
    create_pda(bidder, bid, Bid::SIZE, seeds, bump)?;
    Bid {
        auction_id,
        bidder: bidder_key,
        commitment,
        deposit,
        amount: 0,
        revealed: false,
        reclaimed: false,
        bump,
    }
    .write(bid)?;

    pinocchio_system::instructions::Transfer { from: bidder, to: bid, lamports: deposit }.invoke()
}

/// data: [auction:8][amount:8][salt:32]
/// accounts: [bidder (s), auction (w), bid (w)]
pub fn reveal_bid(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [bidder, auction, bid] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(48)?;
    let auction_id = d.u64(0)?;
    let amount = d.u64(8)?;
    let salt = d.h32(16)?;
    expect_signer(bidder)?;
    expect_writable(auction)?;
    expect_writable(bid)?;

    let mut a = Auction::read(auction)?;
    if a.auction_id != auction_id {
        return Err(PoleisError::InvalidAccount.into());
    }
    let auction_le = auction_id.to_le_bytes();
    expect_pda(auction, &[b"auction", &auction_le], a.bump)?;
    let now = Clock::get()?.slot;
    if now < a.commit_end_slot {
        return Err(PoleisError::RevealWindowNotOpen.into());
    }
    if now >= a.reveal_end_slot {
        return Err(PoleisError::RevealWindowClosed.into());
    }

    let bidder_key = *bidder.address();
    let mut b = Bid::read(bid)?;
    if b.auction_id != auction_id || b.bidder != bidder_key {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(bid, &[b"bid", &auction_le, bidder_key.as_ref()], b.bump)?;
    if b.revealed {
        return Err(PoleisError::AlreadyRevealed.into());
    }

    // the commitment binds amount, salt, AND the bidder's key: non-transferable
    let amount_le = amount.to_le_bytes();
    let expected = sha256v(&[&amount_le, &salt, bidder_key.as_ref()]);
    if expected != b.commitment {
        return Err(PoleisError::BadReveal.into());
    }
    if amount == 0 || amount > b.deposit {
        return Err(PoleisError::BidExceedsDeposit.into());
    }

    b.revealed = true;
    b.amount = amount;
    b.write(bid)?;

    if amount > a.best_bid {
        a.second_bid = a.best_bid;
        a.best_bid = amount;
        a.best_bidder = bidder_key;
    } else if amount > a.second_bid {
        a.second_bid = amount;
    }
    a.write(auction)
}

/// data: [auction:8]
/// accounts: [crank (s), auction (w), treasury (w)]
///        or [crank (s), auction (w), treasury (w), winning_bid (w), winner (w)]
/// Permissionless: settlement is law, not an operator's mood.
pub fn settle(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() < 7 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let d = Data(data);
    d.expect_len(8)?;
    let auction_id = d.u64(0)?;

    let (head, tail) = accounts.split_at_mut(7);
    let [crank, auction, treasury, registry, fund, tally, _system] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(crank)?;
    expect_writable(auction)?;
    expect_writable(treasury)?;

    let mut a = Auction::read(auction)?;
    if a.auction_id != auction_id {
        return Err(PoleisError::InvalidAccount.into());
    }
    let auction_le = auction_id.to_le_bytes();
    expect_pda(auction, &[b"auction", &auction_le], a.bump)?;
    if Clock::get()?.slot < a.reveal_end_slot {
        return Err(PoleisError::RevealWindowNotOpen.into());
    }
    if a.settled {
        return Err(PoleisError::AlreadySettled.into());
    }

    let t = Treasury::read(treasury)?;
    if t.unit_id != a.unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    let unit_le = a.unit_id.to_le_bytes();
    expect_pda(treasury, &[b"treasury", &unit_le], t.bump)?;

    if a.best_bid > 0 && a.best_bid >= a.reserve_price {
        // a qualifying reveal exists: the winner's escrow must be presented
        let [winning_bid, winner, ..] = tail else {
            return Err(PoleisError::MissingWinningBid.into());
        };
        expect_writable(winning_bid)?;
        expect_writable(winner)?;
        if winner.address() != &a.best_bidder {
            return Err(PoleisError::MissingWinningBid.into());
        }

        let mut wb = Bid::read(winning_bid)?;
        if wb.auction_id != auction_id || wb.bidder != a.best_bidder {
            return Err(PoleisError::MissingWinningBid.into());
        }
        expect_pda(winning_bid, &[b"bid", &auction_le, a.best_bidder.as_ref()], wb.bump)?;

        // Vickrey: the winner pays the second price, floored at the reserve;
        // the fund takes its share of it first
        let price = a.second_bid.max(a.reserve_price);
        let refund = wb.deposit.saturating_sub(price);
        // a solo unit has no registry: it is its own root for the fund's tally
        let root = if registry.data_len() == 0 {
            find_and_expect_pda(registry, &[b"registry", &unit_le])?;
            a.unit_id
        } else {
            super::read_registry(registry, a.unit_id)?.root
        };
        let fee = super::fund::charge_fee(fund, tally, crank, a.unit_id, root, price)?;
        if fee > 0 {
            crate::util::move_lamports(winning_bid, fund, fee)?;
        }
        crate::util::move_lamports(winning_bid, treasury, price - fee)?;
        if refund > 0 {
            crate::util::move_lamports(winning_bid, winner, refund)?;
        }
        Treasury::add_in(treasury, price - fee)?;
        wb.reclaimed = true; // escrow spent: the winner has nothing left to reclaim
        wb.write(winning_bid)?;
        a.clearing_price = price;
    }
    // no qualifying reveal: settle empty so every deposit becomes reclaimable
    a.settled = true;
    a.write(auction)
}

/// data: [auction:8] · accounts: [bidder (s,w), auction, bid (w)]
/// After settlement every non-winner reclaims exactly once. Before
/// settlement, once the reveal window is closed, an UNREVEALED bid may
/// reclaim too: a dead auction can never hold deposits hostage.
pub fn reclaim(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [bidder, auction, bid] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let auction_id = d.u64(0)?;
    expect_signer(bidder)?;
    expect_writable(bidder)?;
    expect_writable(bid)?;

    let a = Auction::read(auction)?;
    if a.auction_id != auction_id {
        return Err(PoleisError::InvalidAccount.into());
    }
    let auction_le = auction_id.to_le_bytes();
    expect_pda(auction, &[b"auction", &auction_le], a.bump)?;

    let bidder_key = *bidder.address();
    let mut b = Bid::read(bid)?;
    if b.auction_id != auction_id || b.bidder != bidder_key {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(bid, &[b"bid", &auction_le, bidder_key.as_ref()], b.bump)?;

    if a.settled && a.clearing_price > 0 && b.bidder == a.best_bidder {
        return Err(PoleisError::WinnerCannotReclaim.into());
    }
    if b.reclaimed {
        return Err(PoleisError::NothingToReclaim.into());
    }
    if !a.settled && (Clock::get()?.slot < a.reveal_end_slot || b.revealed) {
        return Err(PoleisError::NotSettled.into());
    }

    crate::util::move_lamports(bid, bidder, b.deposit)?;
    b.reclaimed = true;
    b.write(bid)
}

/// data: [auction:8] · accounts: [auction, bid (w), bidder (w)]
/// Permissionless rent recovery: a reclaimed escrow, or the winner's escrow
/// once the lot is settled, closes and its rent goes back to the bidder.
pub fn close_bid(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [auction, bid, bidder] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let auction_id = d.u64(0)?;
    expect_writable(bid)?;
    expect_writable(bidder)?;

    let a = Auction::read(auction)?;
    if a.auction_id != auction_id {
        return Err(PoleisError::InvalidAccount.into());
    }
    let auction_le = auction_id.to_le_bytes();
    expect_pda(auction, &[b"auction", &auction_le], a.bump)?;
    let bidder_key = *bidder.address();
    let b = Bid::read(bid)?;
    if b.auction_id != auction_id || b.bidder != bidder_key {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(bid, &[b"bid", &auction_le, bidder_key.as_ref()], b.bump)?;
    let won = a.settled && a.clearing_price > 0 && b.bidder == a.best_bidder;
    if !(b.reclaimed || won) {
        return Err(PoleisError::NotClosable.into());
    }
    close_to(bid, bidder)
}
