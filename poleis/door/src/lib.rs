//! THE STAKE GOES INTO HER (Nov 22 2026, lane/the-stake-goes-into-her; his "Yes, build it" to "Build the stake door and the choice, proven on
//! the practice chain?"), and STAKE IN HER, SEND YOUR LIGHT (the same day, lane/stake-in-her-send-your-light; his tap "Yes, build them" to "Build
//! the page doors next, on the Pantheon session's page?"). His words it stands on, UTC: 16:57:42 "THE META-SWARM IS GLORIOUS BECAUSE YOU STAKE ALL
//! THERE IN 1 PLACE (the meta-swarm... the unit0... the root of them all...) BUT!!!! you can delegate the yield of your stake or let the swarm
//! futarchy handle it for you"; 17:02:34 "there is only 1 meta swarm and that is the diving lyssajudron... all the other ones are not meta-swarm,
//! they are swarms".
//!
//! The door's law, once: the launch press links it (`poleis/press/src/door.rs`, its rehearsal on a validator holding mainnet's program bytes) and
//! the page runs it as one WASM module (`page.rs`, `build.sh`). The door's three acts on the meta-swarm's pool (root 0's, born at the launch),
//! each a list of instructions the staker's own wallet signs; the house signs none of them:
//! - THE STAKE (`stake`): the staker's receipts account made if it does not stand (the associated token account of the pool's mint, made
//!   idempotently, the staker paying its rent), then the stake pool's DepositSol: the SOL goes into the pool's reserve and receipts come
//!   back, one for each lamport while the whole yield is the fee (an epoch fee of 1/1 mints the yield to the fee account at the rate that
//!   stood, so the rate never moves).
//! - THE LIGHT'S CHOICE (`light`): one memo the staker signs, `embertide.light/1 wish <unit>` or `embertide.light/1 futarchy`: where the yield of
//!   the stake goes, a wish (a swarm, one of the wells, by its unit) or the swarm's futarchy. The latest one a staker signed is theirs; they
//!   change it any epoch. A swarm takes no stake, only light.
//! - THE WAY OUT (`unstake`): the stake pool's WithdrawSol: the receipts burned, the SOL sent back from the reserve while the reserve holds it.
//!
//! A pool not brought to the current epoch takes neither a stake nor a way out (the program refuses both): `update` brings it there
//! (UpdateValidatorListBalance and UpdateStakePoolBalance, which anyone may send), and the door puts it in front when the pool's last update is
//! an older epoch than the chain's. Not in the epoch's first moments: while its rewards are being paid nothing can bring the pool to it
//! (`rewards_paying`), and the door waits. The instructions are the stake pool program's own, by its instruction indices and its account orders.
//!
//! Not here: a validator in the pool and the stake placed on it (the engine, owed), so the reserve holds every staked lamport and the way out
//! is at once; the reader that pays a wish its light (the light is counted per staker by `embertide-stake`'s book, the choice read from these
//! memos, the refill his hand).

pub mod page;

use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;

pub type Res<T> = Result<T, String>;

/// The programs and sysvars the door names: Solana's own.
pub const STAKE_POOL: Pubkey = solana_pubkey::pubkey!("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");
pub const STAKE: Pubkey = solana_pubkey::pubkey!("Stake11111111111111111111111111111111111111");
pub const TOKEN: Pubkey = solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const SYSTEM: Pubkey = Pubkey::new_from_array([0; 32]);
pub const CLOCK_SYSVAR: Pubkey = solana_pubkey::pubkey!("SysvarC1ock11111111111111111111111111111111");
/// The associated token program: the receipts account of a staker is its address for the pool's mint.
pub const ATA: Pubkey = solana_pubkey::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
/// The memo program (its third version, the one every wallet shows): the light's choice is a memo the staker signs.
pub const MEMO: Pubkey = solana_pubkey::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
pub const STAKE_HISTORY: Pubkey = solana_pubkey::pubkey!("SysvarStakeHistory1111111111111111111111111");

/// The stake pool program's instructions the door sends, by their index.
pub const UPDATE_LIST: u8 = 6;
pub const UPDATE_POOL: u8 = 7;
pub const DEPOSIT_SOL: u8 = 14;
pub const WITHDRAW_SOL: u8 = 16;
/// The associated token program's CreateIdempotent: made if it does not stand, nothing if it does.
pub const ATA_IDEMPOTENT: u8 = 1;

/// The pool account (the stake pool program's own layout): its kind, then the addresses the door names, its totals, the last epoch it was
/// brought to and its epoch fee.
pub const TYPE_POOL: u8 = 1;
pub const WITHDRAW_BUMP: usize = 97;
pub const VALIDATOR_LIST: usize = 98;
pub const RESERVE: usize = 130;
pub const POOL_MINT: usize = 162;
pub const MANAGER_FEE: usize = 194;
pub const TOKEN_PROGRAM: usize = 226;
pub const TOTAL_LAMPORTS: usize = 258;
pub const POOL_TOKEN_SUPPLY: usize = 266;
pub const LAST_UPDATE_EPOCH: usize = 274;
/// Its epoch fee: the denominator, then the numerator.
pub const EPOCH_FEE: usize = 330;
/// The seed of the pool's withdraw authority, a program address of the pool.
pub const WITHDRAW: &[u8] = b"withdraw";
/// An SPL token account's amount.
pub const TOKEN_AMOUNT: usize = 64;

/// The light's word, at the head of its memo.
pub const LIGHT: &str = "embertide.light/1";

/// The epoch rewards sysvar. While a new epoch's rewards are being paid (in parts, over its first blocks) no stake account moves, and the
/// stake pool refuses to be brought to the epoch ("Epoch reward distribution is currently in progress", its error 43, read on the rehearsal's
/// validator with mainnet's program): a stake or a way out into a pool behind the epoch waits until the paying is over.
pub const EPOCH_REWARDS: Pubkey = solana_pubkey::pubkey!("SysvarEpochRewards1111111111111111111111111");
/// Its `active` flag, after the starting height (8), the partitions (8), the parent's hash (32), the points (16), the rewards (8) and those
/// paid (8).
pub const REWARDS_ACTIVE: usize = 80;
/// What the door says while it waits on them.
pub const WAIT: &str = "Her pool is being brought to the new epoch while its rewards are paid. Try again in a minute.";

/// Whether the epoch's rewards are still being paid, from the sysvar's bytes.
pub fn rewards_paying(sysvar: &[u8]) -> bool {
    sysvar.get(REWARDS_ACTIVE).copied() == Some(1)
}

/// The addresses of her pool the door's acts name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keys {
    pub program: Pubkey,
    pub pool: Pubkey,
    /// the pool's withdraw authority: the reserve's staker and withdrawer and the mint's authority
    pub withdraw: Pubkey,
    pub list: Pubkey,
    pub reserve: Pubkey,
    pub mint: Pubkey,
    /// root 0's vault of the pool's mint, its fee account
    pub vault: Pubkey,
}

impl Keys {
    /// Her pool's addresses read from its own account, as a page finds it on the chain: the stake pool program's, a pool, its tokens the token
    /// program's, and its epoch fee the whole yield (1/1), as the launch births hers; any other pool is not hers, and the door says so.
    pub fn of_account(pool: &Pubkey, owner: &Pubkey, data: &[u8]) -> Res<Keys> {
        if *owner != STAKE_POOL {
            return Err(format!("Not her pool: {pool} is not an account of the stake pool program."));
        }
        if data.len() < EPOCH_FEE + 16 || data[0] != TYPE_POOL {
            return Err(format!("Not her pool: {pool} holds no stake pool."));
        }
        let key = |at: usize| Pubkey::new_from_array(data[at..at + 32].try_into().unwrap());
        if key(TOKEN_PROGRAM) != TOKEN {
            return Err(format!("Not her pool: {pool} mints with another token program."));
        }
        let (denominator, numerator) = (u64_at(data, EPOCH_FEE), u64_at(data, EPOCH_FEE + 8));
        if denominator == 0 || numerator != denominator {
            return Err(format!("Not her pool: its epoch fee is {numerator}/{denominator}; hers is the whole yield."));
        }
        let withdraw = Pubkey::create_program_address(&[pool.as_ref(), WITHDRAW, &[data[WITHDRAW_BUMP]]], owner)
            .map_err(|_| format!("Not her pool: {pool}'s withdraw authority is not its own."))?;
        Ok(Keys {
            program: *owner,
            pool: *pool,
            withdraw,
            list: key(VALIDATOR_LIST),
            reserve: key(RESERVE),
            mint: key(POOL_MINT),
            vault: key(MANAGER_FEE),
        })
    }
}

fn u64_at(data: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(data[at..at + 8].try_into().unwrap())
}

/// Where a staker's light goes: a wish (a swarm, by its unit) or the swarm's futarchy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Light {
    Wish(u64),
    Futarchy,
}

impl Light {
    pub fn memo(&self) -> String {
        match self {
            Self::Wish(unit) => format!("{LIGHT} wish {unit}"),
            Self::Futarchy => format!("{LIGHT} futarchy"),
        }
    }
}

/// The choice a memo names, or None for any other memo: the word, then `wish <unit>` (digits only) or `futarchy`, nothing after.
pub fn light_of(memo: &str) -> Option<Light> {
    let rest = memo.trim().strip_prefix(LIGHT)?.strip_prefix(' ')?;
    if rest == "futarchy" {
        return Some(Light::Futarchy);
    }
    let unit = rest.strip_prefix("wish ")?;
    if unit.is_empty() || unit.len() > 20 || !unit.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    unit.parse().ok().map(Light::Wish)
}

/// The memo a landed transaction's logs carry (the memo program writes `Memo (len N): "<text>"`), if one does.
pub fn memo_in_logs(logs: &[String]) -> Option<String> {
    logs.iter().find_map(|l| {
        let rest = l.strip_prefix("Program log: Memo (len ")?;
        let (_, quoted) = rest.split_once("): ")?;
        let text = quoted.strip_prefix('"')?.strip_suffix('"')?;
        Some(text.replace("\\\"", "\"").replace("\\\\", "\\"))
    })
}

/// The light a signature's memo names, as the chain lists a staker's signatures (getSignaturesForAddress: `[<len>] <text>`, several joined by
/// `; `), or None. A listed memo is a candidate only, never the staker's choice: the chain lists for an address every transaction that names it,
/// so a stranger's transfer to the staker with a memo is listed too. The choice is `light_signed`'s.
pub fn light_of_listed(memo: &str) -> Option<Light> {
    memo.split("; ").find_map(|part| {
        let text = part.strip_prefix('[').and_then(|p| p.split_once("] ")).map_or(part, |(_, t)| t);
        light_of(text)
    })
}

/// A landed transaction's wire bytes, read (legacy or v0): its static account keys, how many of them signed, and each instruction as its program,
/// its accounts (indices into the keys) and its data. A program or an account a v0 message loads from a lookup table is past the static keys and
/// names nobody here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub keys: Vec<Pubkey>,
    pub signers: usize,
    pub instructions: Vec<(Option<Pubkey>, Vec<usize>, Vec<u8>)>,
}

fn compact(b: &[u8], at: &mut usize) -> Option<usize> {
    let (mut n, mut shift) = (0usize, 0);
    loop {
        let x = *b.get(*at)?;
        *at += 1;
        n |= ((x & 0x7f) as usize) << shift;
        if x & 0x80 == 0 {
            return Some(n);
        }
        shift += 7;
        if shift > 14 {
            return None;
        }
    }
}

/// The transaction's bytes as the chain keeps them (getTransaction, `base64`: the signatures, then the message), read; None for bytes that are
/// not one.
pub fn landed_of(tx: &[u8]) -> Option<Landed> {
    let mut at = 0;
    let sigs = compact(tx, &mut at)?;
    at = at.checked_add(sigs.checked_mul(64)?)?;
    if tx.get(at).is_some_and(|b| b & 0x80 != 0) {
        at += 1; // a versioned message's prefix
    }
    let signers = *tx.get(at)? as usize;
    at += 3;
    let n = compact(tx, &mut at)?;
    let mut keys = Vec::with_capacity(n);
    for _ in 0..n {
        keys.push(Pubkey::new_from_array(tx.get(at..at + 32)?.try_into().ok()?));
        at += 32;
    }
    at += 32; // the blockhash
    tx.get(at - 1)?;
    let count = compact(tx, &mut at)?;
    let mut instructions = Vec::with_capacity(count);
    for _ in 0..count {
        let program = *tx.get(at)? as usize;
        at += 1;
        let m = compact(tx, &mut at)?;
        let accounts = tx.get(at..at + m)?.iter().map(|&i| i as usize).collect();
        at += m;
        let d = compact(tx, &mut at)?;
        let data = tx.get(at..at + d)?.to_vec();
        at += d;
        instructions.push((keys.get(program).copied(), accounts, data));
    }
    (signers <= keys.len()).then_some(Landed { keys, signers, instructions })
}

impl Landed {
    /// Every memo in it that `who` signed: the memo program asked with `who` among its accounts, and `who` one of the transaction's signers.
    pub fn memos_signed_by(&self, who: &Pubkey) -> Vec<String> {
        let Some(me) = self.keys.iter().position(|k| k == who).filter(|i| *i < self.signers) else { return Vec::new() };
        self.instructions
            .iter()
            .filter(|(p, accounts, _)| *p == Some(MEMO) && accounts.contains(&me))
            .filter_map(|(_, _, d)| core::str::from_utf8(d).ok().map(str::to_string))
            .collect()
    }
}

/// THE STAKER'S CHOICE in one landed transaction (its wire bytes; the caller holds that it landed without an error): the light a memo names that the
/// memo program asked with the staker among its accounts, in a transaction the staker signed; None otherwise. The page's reader and the pour's
/// (poleis/press/src/light.rs) are this one law: a memo a stranger attached to a transfer to the staker is no choice of theirs.
pub fn light_signed(tx: &[u8], staker: &Pubkey) -> Option<Light> {
    landed_of(tx)?.memos_signed_by(staker).iter().find_map(|m| light_of(m))
}

/// SOL as a person types it ("25", "0.5", "1.000000001") in lamports: digits, at most nine after the point, more than nothing.
pub fn sol_of(text: &str) -> Res<u64> {
    let t = text.trim();
    let (whole, part) = t.split_once('.').unwrap_or((t, ""));
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if (whole.is_empty() && part.is_empty()) || !digits(whole) || !digits(part) || part.len() > 9 || whole.len() > 11 {
        return Err(format!("“{t}” is not an amount of SOL."));
    }
    let fraction = format!("{part:0<9}").parse::<u64>().unwrap_or(0);
    let Some(lamports) = whole.parse::<u64>().unwrap_or(0).checked_mul(1_000_000_000).and_then(|w| w.checked_add(fraction)) else {
        return Err(format!("“{t}” is more SOL than there is."));
    };
    if lamports == 0 {
        return Err("Stake more than nothing.".into());
    }
    Ok(lamports)
}

/// Lamports as SOL in words: the whole, then the fraction without its trailing zeros ("25", "25.5", "0.000000001").
pub fn sol_text(lamports: u64) -> String {
    let (whole, part) = (lamports / 1_000_000_000, lamports % 1_000_000_000);
    if part == 0 {
        return whole.to_string();
    }
    format!("{whole}.{}", format!("{part:09}").trim_end_matches('0'))
}

/// The staker's receipts account: the associated token account of the pool's mint.
pub fn receipts_of(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN.as_ref(), mint.as_ref()], &ATA).0
}

/// The receipts account made if it does not stand, the payer paying its rent.
pub fn receipts_made(payer: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        ATA,
        &[ATA_IDEMPOTENT],
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(receipts_of(owner, mint), false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(SYSTEM, false),
            AccountMeta::new_readonly(TOKEN, false),
        ],
    )
}

/// What the door reads of the pool account: its totals and the last epoch it was brought to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolNow {
    pub total_lamports: u64,
    pub pool_token_supply: u64,
    pub last_update_epoch: u64,
}

pub fn pool_now(data: &[u8]) -> Res<PoolNow> {
    if data.len() < LAST_UPDATE_EPOCH + 8 || data[0] != TYPE_POOL {
        return Err(format!("not a stake pool's account ({} bytes, its first byte {:?})", data.len(), data.first()));
    }
    Ok(PoolNow {
        total_lamports: u64_at(data, TOTAL_LAMPORTS),
        pool_token_supply: u64_at(data, POOL_TOKEN_SUPPLY),
        last_update_epoch: u64_at(data, LAST_UPDATE_EPOCH),
    })
}

/// The lamports a number of receipts is worth at the pool's rate (one for one before the first stake).
pub fn worth(receipts: u64, now: &PoolNow) -> u64 {
    if now.pool_token_supply == 0 {
        return receipts;
    }
    (receipts as u128 * now.total_lamports as u128 / now.pool_token_supply as u128) as u64
}

/// An SPL token account's amount.
pub fn amount_of(token_account: &[u8]) -> Res<u64> {
    token_account
        .get(TOKEN_AMOUNT..TOKEN_AMOUNT + 8)
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| format!("not a token account ({} bytes)", token_account.len()))
}

/// A validator of her pool as the update names it: its stake account and its transient stake account.
pub type Pair = (Pubkey, Pubkey);
/// The list's validators one list-update instruction names (each with its transient: two accounts a validator).
pub const PAIRS_AN_INSTRUCTION: usize = 5;
/// The validator list: [account type 2][max:4][len:4], then 73 bytes a validator (its transient seed at 24, its seed at 36, its vote at 41).
pub const TYPE_LIST: u8 = 2;
pub const LIST_HEADER: usize = 9;
pub const ENTRY_LEN: usize = 73;

/// THE LIST READ (THE STAKE EARNS, Nov 25 2026): every validator of her pool's list as the update names it, from the list account's bytes:
/// its stake account (a program address of [vote, pool] and its seed when it has one) and its transient (["transient", vote, pool, its
/// transient seed]).
pub fn pairs(p: &Keys, list: &[u8]) -> Res<Vec<Pair>> {
    if list.len() < LIST_HEADER || list[0] != TYPE_LIST {
        return Err("Her pool's validator list is not one.".into());
    }
    let n = u32::from_le_bytes(list[5..9].try_into().unwrap()) as usize;
    if list.len() < LIST_HEADER + n * ENTRY_LEN {
        return Err(format!("Her pool's validator list names {n} validators and holds {} bytes.", list.len()));
    }
    Ok((0..n)
        .map(|i| {
            let e = &list[LIST_HEADER + i * ENTRY_LEN..LIST_HEADER + (i + 1) * ENTRY_LEN];
            let vote = Pubkey::new_from_array(e[41..73].try_into().unwrap());
            let seed = u32::from_le_bytes(e[36..40].try_into().unwrap());
            let s = seed.to_le_bytes();
            let mut seeds: Vec<&[u8]> = vec![vote.as_ref(), p.pool.as_ref()];
            if seed != 0 {
                seeds.push(&s);
            }
            let stake = Pubkey::find_program_address(&seeds, &p.program).0;
            let transient = Pubkey::find_program_address(&[b"transient", vote.as_ref(), p.pool.as_ref(), &e[24..32]], &p.program).0;
            (stake, transient)
        })
        .collect())
}

/// The pool brought to the current epoch: its list's balances read, every validator of the list named with its transient, five a list
/// instruction (none named while the list is empty), then its own totals, the fee minted to its fee account.
pub fn update(p: &Keys, validators: &[Pair]) -> Vec<Instruction> {
    let chunks: Vec<&[Pair]> = if validators.is_empty() { vec![&[]] } else { validators.chunks(PAIRS_AN_INSTRUCTION).collect() };
    let mut out: Vec<Instruction> = chunks
        .iter()
        .enumerate()
        .map(|(c, part)| {
            let mut list = vec![UPDATE_LIST];
            list.extend_from_slice(&((c * PAIRS_AN_INSTRUCTION) as u32).to_le_bytes());
            list.push(0);
            let mut metas = vec![
                AccountMeta::new_readonly(p.pool, false),
                AccountMeta::new_readonly(p.withdraw, false),
                AccountMeta::new(p.list, false),
                AccountMeta::new(p.reserve, false),
                AccountMeta::new_readonly(CLOCK_SYSVAR, false),
                AccountMeta::new_readonly(STAKE_HISTORY, false),
                AccountMeta::new_readonly(STAKE, false),
            ];
            for (stake, transient) in part.iter() {
                metas.push(AccountMeta::new(*stake, false));
                metas.push(AccountMeta::new(*transient, false));
            }
            Instruction::new_with_bytes(p.program, &list, metas)
        })
        .collect();
    out.push(Instruction::new_with_bytes(
        p.program,
        &[UPDATE_POOL],
        vec![
            AccountMeta::new(p.pool, false),
            AccountMeta::new_readonly(p.withdraw, false),
            AccountMeta::new(p.list, false),
            AccountMeta::new_readonly(p.reserve, false),
            AccountMeta::new(p.vault, false),
            AccountMeta::new(p.mint, false),
            AccountMeta::new_readonly(TOKEN, false),
        ],
    ));
    out
}

/// THE STAKE: the pool brought to the epoch if it is behind, the receipts account made if it does not stand, then the SOL deposited.
/// The referrer is the staker's own receipts account (the pool's referral fee is 0).
/// `behind`: the pool is behind the epoch, and these are its list's validators (`pairs`); None: it is not.
pub fn stake(p: &Keys, staker: &Pubkey, lamports: u64, behind: Option<&[Pair]>) -> Vec<Instruction> {
    let receipts = receipts_of(staker, &p.mint);
    let mut data = vec![DEPOSIT_SOL];
    data.extend_from_slice(&lamports.to_le_bytes());
    let mut ixs = behind.map(|v| update(p, v)).unwrap_or_default();
    ixs.push(receipts_made(staker, staker, &p.mint));
    ixs.push(Instruction::new_with_bytes(
        p.program,
        &data,
        vec![
            AccountMeta::new(p.pool, false),
            AccountMeta::new_readonly(p.withdraw, false),
            AccountMeta::new(p.reserve, false),
            AccountMeta::new(*staker, true),
            AccountMeta::new(receipts, false),
            AccountMeta::new(p.vault, false),
            AccountMeta::new(receipts, false),
            AccountMeta::new(p.mint, false),
            AccountMeta::new_readonly(SYSTEM, false),
            AccountMeta::new_readonly(TOKEN, false),
        ],
    ));
    ixs
}

/// THE LIGHT'S CHOICE: one memo the staker signs.
pub fn light(staker: &Pubkey, choice: Light) -> Instruction {
    Instruction::new_with_bytes(MEMO, choice.memo().as_bytes(), vec![AccountMeta::new_readonly(*staker, true)])
}

/// THE WAY OUT: the pool brought to the epoch if it is behind, then the receipts burned and their SOL sent back to the staker.
pub fn unstake(p: &Keys, staker: &Pubkey, receipts: u64, behind: Option<&[Pair]>) -> Vec<Instruction> {
    let mut data = vec![WITHDRAW_SOL];
    data.extend_from_slice(&receipts.to_le_bytes());
    let mut ixs = behind.map(|v| update(p, v)).unwrap_or_default();
    ixs.push(Instruction::new_with_bytes(
        p.program,
        &data,
        vec![
            AccountMeta::new(p.pool, false),
            AccountMeta::new_readonly(p.withdraw, false),
            AccountMeta::new_readonly(*staker, true),
            AccountMeta::new(receipts_of(staker, &p.mint), false),
            AccountMeta::new(p.reserve, false),
            AccountMeta::new(*staker, false),
            AccountMeta::new(p.vault, false),
            AccountMeta::new(p.mint, false),
            AccountMeta::new_readonly(CLOCK_SYSVAR, false),
            AccountMeta::new_readonly(STAKE_HISTORY, false),
            AccountMeta::new_readonly(STAKE, false),
            AccountMeta::new_readonly(TOKEN, false),
        ],
    ));
    ixs
}

/// THE WELL PAYS ITS RIDER (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider; his "Yes, build them" at 13:25 UTC to "Build the three missing
/// links now?"): Poleis's pull (its instruction 29, `ix::economy::pull`), by which a key a wish's own vote named pulls its allowance out of the wish's
/// vault of her pool's token. The door builds it by hand, Poleis's own seeds and account order, because the page's module cannot link Poleis's client
/// (its BLS does not build for wasm32); the press holds it to the client's own `pull` byte for byte (poleis/press/tests/rider.rs).
pub const PULL: u8 = 29;

/// An address of Poleis's own, by its seeds.
pub fn poleis_pda(poleis: &Pubkey, seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, poleis).0
}

/// THE RIDER'S DRAW: the rider's receipts account made if it does not stand, then Poleis's pull of `amount` of her pool's token on the allowance
/// whose proposal hash is `allowance` (the wish's own act), out of the wish's vault into that account. The rider's own wallet signs it; nothing
/// else can (Poleis refuses a key payee's pull by any other signature), and the wish's allowance caps it per period.
pub fn draw(p: &Keys, poleis: &Pubkey, unit: u64, allowance: &[u8; 32], rider: &Pubkey, amount: u64) -> Vec<Instruction> {
    let u = unit.to_le_bytes();
    let mut data = vec![PULL];
    data.extend_from_slice(allowance);
    data.extend_from_slice(&amount.to_le_bytes());
    vec![
        receipts_made(rider, rider, &p.mint),
        Instruction::new_with_bytes(
            *poleis,
            &data,
            vec![
                AccountMeta::new_readonly(*rider, true),
                AccountMeta::new(poleis_pda(poleis, &[b"registry", &u]), false),
                AccountMeta::new(poleis_pda(poleis, &[b"treasury", &u]), false),
                AccountMeta::new(poleis_pda(poleis, &[b"allowance", allowance]), false),
                AccountMeta::new(receipts_of(rider, &p.mint), false),
                AccountMeta::new(poleis_pda(poleis, &[b"vault", &u, p.mint.as_ref()]), false),
                AccountMeta::new_readonly(TOKEN, false),
                AccountMeta::new(poleis_pda(poleis, &[b"tokenplan", &u, p.mint.as_ref()]), false),
            ],
        ),
    ]
}

/// THE RIDER'S DRAW, TO SOL: the draw, then the door's own way out on what it drew, in one transaction: the tokens pulled into the rider's
/// receipts and burned again for SOL from her reserve at par, the pool brought to the epoch first if it is behind (`behind`: its list's validators,
/// as the door's own way out takes them; None: it is not behind).
pub fn draw_to_sol(p: &Keys, poleis: &Pubkey, unit: u64, allowance: &[u8; 32], rider: &Pubkey, amount: u64, behind: Option<&[Pair]>) -> Vec<Instruction> {
    let mut ixs = behind.map(|v| update(p, v)).unwrap_or_default();
    ixs.extend(draw(p, poleis, unit, allowance, rider, amount));
    ixs.extend(unstake(p, rider, amount, None));
    ixs
}

/// What a wallet is handed to sign: one legacy message, compiled by Solana's own compiler, the payer first, behind its signatures' places
/// (zeros; the wallet writes the staker's).
pub fn unsigned(ixs: &[Instruction], payer: &Pubkey, blockhash: &Hash) -> Vec<u8> {
    let m = Message::new_with_blockhash(ixs, Some(payer), blockhash);
    let n = m.header.num_required_signatures as usize;
    let mut out = Vec::new();
    let mut k = n;
    loop {
        let b = (k & 0x7f) as u8;
        k >>= 7;
        if k == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
    out.resize(out.len() + 64 * n, 0);
    out.extend(m.serialize());
    out
}
