//! poleis_wars: the price wars on a chain (Nov 24 2026, lane/the-war-is-its-own-program).
//!
//! His words, UTC, Nov 24 2026: "you should follow always the meme royale formula... if you desert the fight, you deserve to lose" (01:08:33);
//! "Half" (01:14:54); "Yes, burn them"; and "Yes, build it" to "Build the war's program first, on a local validator?".
//!
//! A house is a coin on pump.fun's curve, priced in SOL. Its holders raid another house by staking their coins; once the stake reaches the
//! need the raid is open, minute by minute, and anyone cranks it: each side must beat a line that falls to nothing by the raid's end, the side
//! that did not gets a last stand, and the one further along wins at the end. Whoever deserts during a raid, or changes sides, is paid at most
//! half of what he brought into his house out of its reserve, and his coins are burned. At the settlement the loser's pot buys the winner's
//! coin on the winner's curve, and each holder of the loser converts his coins into his share of it, the raid's stakers a tenth better; the
//! winner's stakers then unlock their coins, and the winners who sell pay a dump tax that falls to nothing. A stake in a raid that never
//! opened is its staker's to take back (`unstake`).
//!
//! THE LAW IS NOT THIS PROGRAM'S: every number it moves is moved by one whole call of embertide-war's core (`buy_on`, `sell_on`, `burn_on`,
//! `stake_on`, `minute_on`, `trades_until`, `settle_on`, `weights_of`, `weight_of`, `share_of`, `dump_tax_of`, `dump_rate_at`, `tax_split`,
//! `mcap`, `Rules::check`), never by a copy of its arithmetic. This program keeps the accounts, the lamports, the checks and the clock.
//!
//! THE GAME IS ONE SWARM (Nov 24 2026, lane/the-game-pays-its-swarm; his "we are building a game publisher firm... the whole mechanism of
//! meme royale is to use the blockchain as multiplayer like pump.fun", his "Yes, pay them" to "Pay each house's creator its 0.30% and the
//! publisher its 0.95%, as pump.fun does?", his "poleis is meta-swarm... this gaming shit is 1 swarm"). The game publisher, this war and every
//! game after it, is one swarm brought to life through poleis.us, and its money is its Poleis treasury. A config is one generation of the
//! war's rules, founded by the swarm's seal and naming that treasury; the houses are coins players and agents launch inside it, as pump.fun's
//! are, and each house's creator share of every trade goes to whoever launched it. No number is chosen here: the shares are the generation's.
//!
//! THE LAMPORTS ARE THE BOOK. A house holds exactly its rent, its reserve, its chest and what it owes its creator; the fee account exactly
//! its rent and every protocol fee and tax it was paid (and what it held before its birth), less what it paid its publisher. Every lamport a
//! buy, a sale, a burn, a change of sides, a settlement or a payment moves goes from one of these accounts or a signer's wallet to another,
//! by the law's numbers, and the tests hold the book after every instruction.
//!
//! THE CLOCK IS THE CHAIN'S `unix_timestamp`, and a raid's minute is `(now - start) / minute_seconds`, the Config's (60 by default, the law's
//! minute; the rehearsal on a local validator, whose clock cannot be warped, runs a raid at 4: rehearse-on-a-local-validator.sh). The dump tax's
//! length is in real seconds, as Meme Royale's.

#![no_std]

use pinocchio::{error::ProgramError, no_allocator, nostd_panic_handler, program_entrypoint, AccountView, Address, ProgramResult};

pub mod error;
pub mod ix;
pub mod state;
pub mod util;

// THE PROGRAM'S ID IS ITS DEVNET ADDRESS (his "Yes, devnet", Nov 24 2026 12:24:57 UTC, to "Put the war on devnet next, Solana's free test
// chain, so screens and phones can read a live war?"): deployed there from a throwaway key made for it on the new Mac (never in git), its
// upgrade authority another throwaway devnet key beside it (~/.embertide/poleis-wars-devnet/). Before it the id was a placeholder nobody held a
// key for (E8BEmciBLa7tfCWXpdgguRD7DSsbik6q3uJzPdVrhXba). Mainnet's id and authority are the swarm's ceremony's, never a seat's.
solana_address::declare_id!("6ZA4DVYtEEDZF4nTjcaxzeWLxWUs7Mrve9EANzjRs4Ef");

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

// instruction tags: byte 0 of instruction data
pub const IX_INIT_CONFIG: u8 = 0;
pub const IX_INIT_HOUSE: u8 = 1;
pub const IX_BUY: u8 = 2;
pub const IX_SELL: u8 = 3;
pub const IX_STAKE: u8 = 4;
pub const IX_DESERT: u8 = 5;
pub const IX_CHANGE_SIDES: u8 = 6;
pub const IX_CRANK: u8 = 7;
pub const IX_SETTLE: u8 = 8;
pub const IX_CONVERT: u8 = 9;
pub const IX_UNLOCK: u8 = 10;
/// a staker takes his stake back from a raid still a proposal (Meme Royale's withdrawProposalStake)
pub const IX_UNSTAKE: u8 = 11;
/// THE GAME IS ONE SWARM: anyone pushes what a house owes its creator to its creator, and what a generation's fee account holds to the
/// generation's treasury, the publisher's
pub const IX_PAY_CREATOR: u8 = 12;
pub const IX_PAY_PUBLISHER: u8 = 13;
pub const IX_INIT_MARCH_CONFIG: u8 = 14;
pub const IX_LAND_ORDERS: u8 = 15;
pub const IX_CLAIM_MARCH_QUOTE: u8 = 16;
pub const IX_CLOSE_POSITION: u8 = 17;

fn process_instruction(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let (tag, rest) = data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match *tag {
        IX_INIT_CONFIG => ix::config::init_config(accounts, rest),
        IX_INIT_HOUSE => ix::config::init_house(accounts, rest),
        IX_BUY => ix::trade::buy(accounts, rest),
        IX_SELL => ix::trade::sell(accounts, rest),
        IX_STAKE => ix::raid::stake(accounts, rest),
        IX_DESERT => ix::raid::desert(accounts, rest),
        IX_CHANGE_SIDES => ix::raid::change_sides(accounts, rest),
        IX_CRANK => ix::raid::crank(accounts, rest),
        IX_SETTLE => ix::settle::settle(accounts, rest),
        IX_CONVERT => ix::settle::convert(accounts, rest),
        IX_UNLOCK => ix::settle::unlock(accounts, rest),
        IX_UNSTAKE => ix::raid::unstake(accounts, rest),
        IX_PAY_CREATOR => ix::pay::pay_creator(accounts, rest),
        IX_PAY_PUBLISHER => ix::pay::pay_publisher(accounts, rest),
        IX_INIT_MARCH_CONFIG => ix::config::init_march_config(accounts, rest),
        IX_LAND_ORDERS => ix::march::land(accounts, rest),
        IX_CLAIM_MARCH_QUOTE => ix::march::claim(accounts, rest),
        IX_CLOSE_POSITION => ix::march::close_position(accounts, rest),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
