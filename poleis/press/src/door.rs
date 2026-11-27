//! THE STAKE GOES INTO HER (Nov 22 2026, lane/the-stake-goes-into-her; his "Yes, build it" to "Build the stake door and the choice, proven on
//! the practice chain?"). The door's three acts on the meta-swarm's pool (the one `pool.rs` births at the launch, root 0's): the stake, the
//! light's choice, the way out, each a list of instructions the staker's own wallet signs.
//!
//! Their law lives once, in `poleis/door` (STAKE IN HER, SEND YOUR LIGHT, Nov 22 2026, his tap "Yes, build them"): the press links it, and the
//! page runs the same code as one WASM module. Here the press names it on its own `Pool`; the proof is `tests/door.rs` on a validator holding
//! mainnet's program bytes (`poleis/ceremony/the-door-rehearsal.sh`).

use crate::pool::Pool;
pub use poleis_door::{
    amount_of, light, light_of, light_of_listed, memo_in_logs, pool_now, receipts_made, receipts_of, rewards_paying, sol_of, sol_text, unsigned, worth,
    Keys, Light, Pair, PoolNow, ATA, ATA_IDEMPOTENT, DEPOSIT_SOL, EPOCH_REWARDS, LAST_UPDATE_EPOCH, LIGHT, MEMO, POOL_TOKEN_SUPPLY, REWARDS_ACTIVE,
    STAKE_HISTORY, TOKEN_AMOUNT, TOTAL_LAMPORTS, UPDATE_LIST, UPDATE_POOL, WAIT, WITHDRAW_SOL,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

/// Her pool's addresses as the door's acts name them.
pub fn keys(p: &Pool) -> Keys {
    Keys { program: p.program, pool: p.pool, withdraw: p.withdraw, list: p.list, reserve: p.reserve, mint: p.mint, vault: p.vault }
}

/// THE LIST READ: her list's validators as the update names them, from the list account's bytes.
pub fn pairs(p: &Pool, list: &[u8]) -> crate::Res<Vec<Pair>> {
    poleis_door::pairs(&keys(p), list)
}

/// The pool brought to the current epoch, its list's validators named.
pub fn update(p: &Pool, validators: &[Pair]) -> Vec<Instruction> {
    poleis_door::update(&keys(p), validators)
}

/// THE STAKE: the pool brought to the epoch if it is behind (`behind`: its list's validators), the receipts account made if it does not
/// stand, then the SOL deposited.
pub fn stake(p: &Pool, staker: &Pubkey, lamports: u64, behind: Option<&[Pair]>) -> Vec<Instruction> {
    poleis_door::stake(&keys(p), staker, lamports, behind)
}

/// THE WAY OUT: the pool brought to the epoch if it is behind, then the receipts burned and their SOL sent back to the staker.
pub fn unstake(p: &Pool, staker: &Pubkey, receipts: u64, behind: Option<&[Pair]>) -> Vec<Instruction> {
    poleis_door::unstake(&keys(p), staker, receipts, behind)
}
