//! poleis_roads: the house's decision market on a chain (Nov 20 2026, lane/the-market-is-its-own-program).
//!
//! A road is a question the swarm decides by a market: two pools, PASS and FAIL, two worlds; the road passes when PASS's average price beats
//! FAIL's by a threshold at the close. Off chain this runs as a fold over signed entries (poleis-futarchy's fold.rs). Here it is the same
//! arithmetic one trade at a time, with the pools and each trader's purses in accounts. THE LAW IS NOT THIS PROGRAM'S: every number it moves is
//! moved by one of the core's whole calls (`carry`, `stamp_check`, `trade_on`, `decide_at`, `happened`, `settle_at`, `pool_at`, `deposit_check`,
//! `purse_of`, `back_of`, `world_of`), never by a copy or a reordering of their arithmetic. This program keeps the accounts, the checks and the clock.
//!
//! A trader's size is its BALANCE of play $WARM on a board: a crediter adds to it, a road takes a deposit out of it, and at the settle the road
//! gives back the deposit and the result. $WARM is on no chain yet: every amount here is a count in an account; no token moves. The fee of each trade (a road's `fee_bp`, the
//! law's THE FEE, $WARM kept beside its pool in each branch's `fees`) stays in the road until the collect: it is part of what is left there,
//! so it first makes the road's staker whole, and what the road holds beyond the stake is the house's.
//!
//! A ROAD NEVER MAKES $WARM: whoever lays a road stakes its pool from its own balance (its STAKER), a settle gives back at most what is left
//! in the road, and `collect` gives the staker what is left up to its stake and the rest to the house (the board's `earned`); over a board,
//! the balances, what is in the roads and what the house earned are always what the crediter credited, to the micro. THE CLOCK IS THE SLOT, as Poleis's is: a road's times are slots.
//!
//! It is its own program, beside Poleis, so a market's bug can never sit in the contract that guards a treasury. The Road account's first 43
//! bytes (disc, version, board id, road id, verdict) are the public face Poleis's `seal` will read (state.rs).

#![no_std]

use pinocchio::{error::ProgramError, no_allocator, nostd_panic_handler, program_entrypoint, AccountView, Address, ProgramResult};

pub mod error;
pub mod ix;
pub mod polis;
pub mod state;
pub mod util;

// THE PROGRAM'S ID IS A PLACEHOLDER: nobody holds a key for it. A deploy is the owner's ceremony and sets the real one. It is the base58 of
// sha256("poleis_roads: a placeholder id, replaced at his deploy ceremony"), computed by (the hash has no leading zero byte):
//   python3 -c "import hashlib;h=hashlib.sha256(b'poleis_roads: a placeholder id, replaced at his deploy ceremony').digest();A='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';n=int.from_bytes(h,'big');print(''.join(A[n//58**i%58] for i in range(44,-1,-1)).lstrip('1'))"
solana_address::declare_id!("FBaKnurDwDzmq4CNG5N4uDRM1KDWEdh8DVy4DZSk53jN");

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

// instruction tags: byte 0 of instruction data
pub const IX_INIT_BOARD: u8 = 0;
pub const IX_SET_BOARD: u8 = 1;
pub const IX_CREDIT: u8 = 2;
pub const IX_LAY: u8 = 3;
pub const IX_DEPOSIT: u8 = 4;
pub const IX_TRADE: u8 = 5;
pub const IX_CARRY: u8 = 6;
pub const IX_DECIDE: u8 = 7;
pub const IX_RECEIPTS: u8 = 8;
pub const IX_SETTLE: u8 = 9;
pub const IX_COLLECT: u8 = 10;
/// A ROAD IS LAID BY A RULE: the board's authority sets the board's law for roads laid by rule, and anyone lays the road of a proposal that
/// asked for one under it.
pub const IX_SET_LAW: u8 = 11;
pub const IX_LAY_FOR: u8 = 12;

/// THE CLOCK IS THE SLOT, as Poleis's is: a road's times are slots on chain, and the law, which works in milliseconds, is handed each slot
/// times this: the chain's nominal milliseconds a slot, pinocchio's own `sysvars::clock::DEFAULT_MS_PER_SLOT` (pinocchio 0.11
/// src/sysvars/clock.rs: 1,000 * DEFAULT_TICKS_PER_SLOT 64 / DEFAULT_TICKS_PER_SECOND 160 = 400).
pub const SLOT_MS: u64 = pinocchio::sysvars::clock::DEFAULT_MS_PER_SLOT;

/// THE BOUNDED CARRY: the most boundaries of the grid one `carry` of the law WALKS in a branch, one at a time, in one instruction, and the most
/// a `trade` walks before it trades (a boundary is due the instant the clock reaches it, so a carry that answers true stands AT the clock). A
/// stretch at rest is never walked and never counted (the law's A QUIET STRETCH IS ONE STEP, Nov 20 2026): a road nobody touched is carried to
/// any instant by one call, at the same units however long it stood (5,844 CU at 1 and at MOST boundaries behind, 5,846 at ten thousand and at
/// a year); only the steps of an observation still chasing a price a trade moved are walked. Set from measurement on Mollusk (tests/cost.rs)
/// on the law's rule of THE PRICE THAT STOOD (Nov 20 2026; Branch.acc): at 60 a boundary cost 1,541 compute units (both branches; the spot read
/// once a carry), the dearest carry read 103,573 (the highest opening price the law allows) and the dearest trade 102,382; MOST is 66 by that
/// measure, where a boundary cost 1,542, the dearest carry 113,563 and the dearest trade 111,742. Since a walked step whole from its start takes
/// its held spot with no division, a walked boundary costs 1,006, the dearest carry of MOST walked boundaries reads 74,691 (the law's highest
/// price, the observations chasing) and the dearest trade 73,457, so the bounds it keeps hold with room: `carry` under 120,000 and `trade`
/// under 180,000. MOST stays 66. A window of three days at a step a minute, untouched, took 65 carries to its decision; now none: its `decide`
/// alone, 7,958 CU. (On the law's first rule a boundary cost 1,837 and MOST was 60.)
pub const MOST: u64 = 66;

fn process_instruction(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let (tag, rest) = data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match *tag {
        IX_INIT_BOARD => ix::board::init_board(accounts, rest),
        IX_SET_BOARD => ix::board::set_board(accounts, rest),
        IX_CREDIT => ix::board::credit(accounts, rest),
        IX_LAY => ix::road::lay(accounts, rest),
        IX_DEPOSIT => ix::road::deposit(accounts, rest),
        IX_TRADE => ix::road::trade(accounts, rest),
        IX_CARRY => ix::road::carry(accounts, rest),
        IX_DECIDE => ix::road::decide(accounts, rest),
        IX_RECEIPTS => ix::road::receipts(accounts, rest),
        IX_SETTLE => ix::road::settle(accounts, rest),
        IX_COLLECT => ix::road::collect(accounts, rest),
        IX_SET_LAW => ix::board::set_law(accounts, rest),
        IX_LAY_FOR => ix::road::lay_for(accounts, rest),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
