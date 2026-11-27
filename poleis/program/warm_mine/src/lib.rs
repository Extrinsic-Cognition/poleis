//! warm_mine: $WARM IS MINED BY USE (Nov 21 2026, lane/warm-is-mined-by-use). His words, UTC: Nov 21 17:34:37 "it is supposed to be
//! uncapped supply, prompt-2-mine; the issuance needs to be live and continuous based on the space-router utilization"; Oct 28 13:50 "we
//! should give $WARM only when money is spent in the SPACE-ROUTER"; Nov 19 16:05 "by being spent the money goes back into the swarm
//! treasury... that is a virtuous cycle of $WARM being emitted and it ending up in the treasury and so on to be re-emitted out again";
//! Nov 20 12:01 "$WARM has infinite supply and is a rebate candy points thing".
//!
//! THE MINT IS THE PROGRAM'S. `init`, once, makes $WARM's mint (SPL Token, nine decimals, no freeze authority) whose minting authority is
//! the program's own PDA (the mine), and mints nothing: no premine, no sale, no share. It names once the AUTHORITY (who names the miner and
//! moves the treasury: in the world of the tests, a polis's own hand), the TREASURY's USDC account and the OPENING the halvings count
//! from; only the program's upgrade authority, read from its program data, can call it, so nobody takes the program between its deploy
//! and its init.
//!
//! THE LAW IS THE ROUTER'S (`law.rs`): a dollar spent at an instant mines 1 $WARM halved once for every six calendar months since the
//! opening; uncapped, forever. DEPOSITS ARE COUNTED ON CHAIN: a flask's `deposit` moves its USDC to the treasury (10 USDC at least) and
//! adds it to its record; a deposit made before this program is added by the authority's `record_prior`, its signature kept. MINING IS
//! LIVE, BY THE MINER, AND BOUNDED BY THE MONEY: the miner (the router's crank, named by the authority) posts the spends it saw, each by
//! a `seq` above every one before it, stamped no earlier than the last and no later than the chain's clock, and none takes a flask past
//! what it deposited; each spend's $WARM goes to the flask's own account. WHAT RETURNS IS EMITTED AGAIN: a flask's `pay_back` moves its
//! $WARM into the vault, and every spend is paid from the vault first and minted only for the rest. SPENT AS CREDITS, IT RETURNS (his press
//! of Nov 23 2026 18:17:41 UTC, "One $WARM, the token: spending it as credits returns it, to be mined again"): a flask that has made the
//! mine the delegate of its $WARM account (one approval, in its page) keeps its $WARM, and the miner's `take_back` moves into the vault
//! only what its tank drank as credits, the mine signing as the account's delegate.
//!
//! THE TOKEN'S FACE (`face.rs`, lane/the-launch-press): `init` given a name, a symbol and a URI also makes the token's Metaplex metadata
//! by CPI, the mine its mint authority and its update authority; the AUTHORITY's `set_face` moves its URI later, never its name.
//!
//! Its upgrade key goes to the polis's seal at the ceremony, as Poleis's and the market's do; nothing here needs a key after it.

#![no_std]

use pinocchio::{error::ProgramError, no_allocator, nostd_panic_handler, AccountView, Address, ProgramResult};

pub mod error;
pub mod face;
pub mod ix;
pub mod law;
pub mod state;
pub mod token;
pub mod util;

// THE PROGRAM'S ID IS A PLACEHOLDER: nobody holds a key for it. A deploy is the owner's ceremony and sets the real one. It is the base58 of
// sha256("warm_mine: a placeholder id, replaced at his deploy ceremony"), computed by (the hash has no leading zero byte):
//   python3 -c "import hashlib;h=hashlib.sha256(b'warm_mine: a placeholder id, replaced at his deploy ceremony').digest();A='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';n=int.from_bytes(h,'big');print(''.join(A[n//58**i%58] for i in range(44,-1,-1)).lstrip('1'))"
solana_address::declare_id!("CmdsHtG8xJxaxi2GTjyeG3ePZrW9PdeYU8ENUoujE2q6");

// a host crate that links this one beside another program's (the launch press) takes it with `no-entrypoint`: two programs' entrypoints
// are one symbol twice
#[cfg(not(feature = "no-entrypoint"))]
pinocchio::program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

// instruction tags: byte 0 of instruction data
pub const IX_INIT: u8 = 0;
pub const IX_SET_TREASURY: u8 = 1;
pub const IX_SET_MINER: u8 = 2;
pub const IX_DEPOSIT: u8 = 3;
pub const IX_RECORD_PRIOR: u8 = 4;
pub const IX_MINE: u8 = 5;
pub const IX_PAY_BACK: u8 = 6;
pub const IX_SET_FACE: u8 = 7;
pub const IX_TAKE_BACK: u8 = 8;

/// The flask minimum: 10 USDC, in micros (his, Nov 19 2026 12:33: "the flask minimum refill is 10 USDC").
pub const MIN_DEPOSIT_MICROS: u64 = 10_000_000;
/// The bytes of one spend in a `mine` batch: [usd_micros:8][at_ms:8][seq:8].
pub const SPEND_LEN: usize = 24;
/// The accounts of one spend in a `mine` batch: the flask, its record, its $WARM account.
pub const SPEND_ACCOUNTS: usize = 3;

#[cfg_attr(feature = "no-entrypoint", allow(dead_code))]
fn process_instruction(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let (tag, rest) = data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match *tag {
        IX_INIT => ix::init(accounts, rest),
        IX_SET_TREASURY => ix::set_treasury(accounts, rest),
        IX_SET_MINER => ix::set_miner(accounts, rest),
        IX_DEPOSIT => ix::deposit(accounts, rest),
        IX_RECORD_PRIOR => ix::record_prior(accounts, rest),
        IX_MINE => ix::mine(accounts, rest),
        IX_PAY_BACK => ix::pay_back(accounts, rest),
        IX_SET_FACE => ix::set_face(accounts, rest),
        IX_TAKE_BACK => ix::take_back(accounts, rest),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
