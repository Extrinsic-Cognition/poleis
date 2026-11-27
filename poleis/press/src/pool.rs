//! THE POOL IS BORN AT THE LAUNCH (Nov 22 2026, lane/the-pool-is-born-at-the-launch; his "Yes, add it" at 01:18:14 UTC to "Add the staking
//! pool's birth to your mainnet launch?"). A society's page on Swarmbook promises "Keep your SOL. Burn its light.": the SPL stake pool behind
//! that promise is born at the launch by root 0's own sealed raw act, act 5 of the press's acts, as it was born on devnet on Oct 29 2026
//! (poleis/ceremony/birth-pool.mjs, its certificate pool-birth-certificate-devnet-u0a.json): its manager and its staker the polis's hand
//! (`agent_pda(0)`), its fee account root 0's vault of the pool's mint (`vault_pda(0, mint)`, answering to root 0's treasury), its epoch fee
//! 1/1, so the whole yield lands under the seal and leaves only by a sealed TREASURY act. No key is its manager.
//!
//! THE HOLE AND ITS CURE. The stake pool's Initialize asks only the manager to sign, never the pool's own account: an empty pool account and
//! list standing on the chain, owned by the program and not initialized, are anyone's to initialize with a manager and a fee account of their
//! own. On devnet the crank made them 40 slots before the sealed execute; on mainnet the act waits for the council for hours. So:
//! - AHEAD, in the first sitting (`press pool`), the deployer paying: the reserve (a stake account whose staker and withdrawer are both the
//!   pool's withdraw authority, a PDA of the pool's address, holding exactly its rent), the mint (nine decimals, its mint authority that PDA,
//!   no freeze authority) and root 0's vault of the mint (Poleis's permissionless, init-once `create_vault`). Useless to anyone else: they
//!   answer to a pool address only the deployer can make.
//! - IN THE EXECUTE'S OWN TRANSACTION, before the execute: the pool's account and its validator list, by `create_account_with_seed` whose
//!   base is the key that executes (the deployer, as press crank). Only the base makes an account at its `with_seed` address, so a stranger
//!   who carries out the passed act makes its accounts at its own addresses: the payload's pool and list stay empty, and its execute fails
//!   whole. Lamports a stranger sends to those addresses do not bar them: such an address is topped up and given its space and owner by the
//!   base instead (`allocate_with_seed`), as the deploy makes a program account a stranger sent dust to.
//!
//! THE BYTES IT WAS REHEARSED ON. The stake pool program is upgradeable (on mainnet its last deploy was by a multisig, read at slot
//! 469,669,929 on Nov 22 2026). A rehearsal proves the birth on the program's bytes as they stood when copied, so the sitting records the
//! program's last deploy slot and the sha256 of its executable bytes the first time it reads them, and holds every later read to that record
//! and to the params' `pool.programSha256` (on mainnet the hash a rehearsal's certificate names, his to fill): other bytes, and nothing of the
//! pool is sent.

use crate::chain::{vault_held, Sitting, TOKEN};
use crate::loader::{self, State, LOADER_V3, RENT_SYSVAR, SYSTEM};
use crate::params::Params;
use crate::rpc::{Account, Cluster};
use crate::wire;
use crate::Res;
use poleis_client::{self as client, Payload, ACT_SIGNER, ACT_WRITABLE, ROOT_UNIT};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::str::FromStr as _;

/// The SPL stake pool program on mainnet (devnet's birth of Oct 29 ran another deployment of it, DPoo15wW…).
pub const STAKE_POOL: Pubkey = solana_pubkey::pubkey!("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");
/// The stake program: the reserve is one of its accounts.
pub const STAKE: Pubkey = solana_pubkey::pubkey!("Stake11111111111111111111111111111111111111");
/// The pool's account (a `StakePool`, as devnet's birth sized it), the reserve's stake account, the mint.
pub const POOL_LEN: usize = 611;
pub const RESERVE_LEN: usize = 200;
pub const MINT_LEN: usize = warm_mine::token::MINT_LEN;
/// The validator list: [account type:1][max validators:4][the vector's length:4], then 73 bytes a validator.
pub const LIST_HEADER: usize = 9;
pub const VALIDATOR_LEN: usize = 73;
/// The largest account Solana allocates (its MAX_PERMITTED_DATA_LENGTH, 10 MiB): the list's bound.
pub const MAX_DATA: usize = 10 * 1024 * 1024;
/// The most validators a list in one account holds; the stake pool's Initialize refuses none.
pub const MAX_VALIDATORS: u64 = ((MAX_DATA - LIST_HEADER) / VALIDATOR_LEN) as u64;
/// The mint's decimals: SOL's own nine.
pub const DECIMALS: u8 = 9;
/// The account types the stake pool program writes in an account's first byte.
pub const TYPE_POOL: u8 = 1;
pub const TYPE_LIST: u8 = 2;
/// The seeds of the pool's two authorities, PDAs of its address under the stake pool program.
pub const WITHDRAW: &[u8] = b"withdraw";
pub const DEPOSIT: &[u8] = b"deposit";
/// What the press says where the pool cannot be born, and where that cannot be.
pub const NOT_RUN: &str = "the pool's birth not run: no stake pool program on this chain";
pub const NOT_MAINNET: &str = "the stake pool program does not stand on this cluster: mainnet has it; this is no mainnet";

/// The list's bytes for `n` validators.
pub fn list_len(n: u32) -> usize {
    LIST_HEADER + VALIDATOR_LEN * n as usize
}

/// One of the pool's authorities: the PDA of its address and the kind's seed under the stake pool program.
pub fn authority_of(pool: &Pubkey, program: &Pubkey, kind: &[u8]) -> Pubkey {
    Pubkey::find_program_address(&[pool.as_ref(), kind], program).0
}

/// An account of the system program with no data: nothing, or the lamports a stranger sent to an address.
pub fn is_dust(a: &Account) -> bool {
    a.owner == SYSTEM.to_string() && a.data.is_empty()
}

/// An account a transaction makes at the `with_seed` address of its base, the key that pays and signs: what it is, its seed, its size, its
/// owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seeded {
    pub what: String,
    pub seed: String,
    pub space: usize,
    pub owner: Pubkey,
}

impl Seeded {
    /// Its address for a base: sha256 of the base, the seed and the owner.
    pub fn at(&self, base: &Pubkey) -> Res<Pubkey> {
        loader::address_with_seed(base, &self.seed, &self.owner)
    }
    /// Its making by `base`, which pays and signs: create_account_with_seed while nothing stands at its address. A system account with no data
    /// there (lamports a stranger sent, over which a create is refused) is topped up to the rent and given its space and its owner by
    /// allocate_with_seed instead, so dust does not bar it. Anything else standing there is no account a sitting made.
    pub fn making(&self, base: &Pubkey, rent: u64, standing: Option<&Account>) -> Res<Vec<Instruction>> {
        let at = self.at(base)?;
        match standing {
            None => Ok(vec![loader::create_account_with_seed(base, &at, base, &self.seed, rent, self.space as u64, &self.owner)]),
            Some(a) if is_dust(a) => {
                let mut v = Vec::new();
                if a.lamports < rent {
                    v.push(loader::transfer(base, &at, rent - a.lamports));
                }
                v.push(loader::allocate_with_seed(&at, base, &self.seed, self.space as u64, &self.owner));
                Ok(v)
            }
            Some(a) => Err(format!("{}: an account stands at its address {at} ({} lamports, owner {}, {} bytes): none this sitting made", self.what, a.lamports, a.owner, a.data.len())),
        }
    }
}

/// A tag: 1 to 8 of a-z and 0-9 (birth-pool.mjs's rule), so each seed (the tag and one letter) is far inside a seed's 32 bytes.
pub fn tag_held(tag: &str) -> Res<()> {
    if tag.is_empty() || tag.len() > 8 || !tag.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()) {
        return Err(format!("params pool.tag: {tag:?}; a tag is 1 to 8 of a-z and 0-9 (a new tag is a new pool)"));
    }
    Ok(())
}

/// The pool, every address of it derived: from its base (the deployer, which makes its seeded accounts), the tag, the stake pool program and
/// Poleis (the polis's hand, root 0's treasury and its vault of the pool's mint).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pool {
    pub tag: String,
    pub max_validators: u32,
    pub base: Pubkey,
    pub program: Pubkey,
    pub pool: Pubkey,
    pub list: Pubkey,
    pub reserve: Pubkey,
    pub mint: Pubkey,
    /// the pool's withdraw authority: the reserve's staker and withdrawer and the mint's authority
    pub withdraw: Pubkey,
    /// the program's default deposit authority: anyone may stake
    pub deposit: Pubkey,
    /// the polis's hand: the pool's manager and its staker
    pub agent: Pubkey,
    /// root 0's vault of the pool's mint, its fee account; root 0's treasury, that vault's authority
    pub vault: Pubkey,
    pub treasury: Pubkey,
    /// THE STAKE EARNS: the engine's key the params name, the staker the pool stands under once the engine's act (the ninth) has executed
    pub engine: Option<Pubkey>,
}

impl Pool {
    pub fn derive(base: &Pubkey, tag: &str, max_validators: u64, program: &Pubkey, poleis: &Pubkey) -> Res<Self> {
        tag_held(tag)?;
        if max_validators == 0 || max_validators > MAX_VALIDATORS {
            return Err(format!("params pool.maxValidators: {max_validators}; the stake pool takes 1 to {MAX_VALIDATORS} (a list of more is past the {MAX_DATA} bytes an account holds)"));
        }
        let seeded = |c: char, owner: &Pubkey| loader::address_with_seed(base, &format!("{tag}{c}"), owner);
        let pool = seeded('p', program)?;
        let mint = seeded('m', &TOKEN)?;
        let pda = |seeds: &[&[u8]]| Pubkey::find_program_address(seeds, poleis).0;
        Ok(Self {
            tag: tag.to_string(),
            max_validators: max_validators as u32,
            base: *base,
            program: *program,
            pool,
            list: seeded('l', program)?,
            reserve: seeded('r', &STAKE)?,
            mint,
            withdraw: authority_of(&pool, program, WITHDRAW),
            deposit: authority_of(&pool, program, DEPOSIT),
            agent: pda(&[b"agent", &ROOT_UNIT.to_le_bytes()]),
            vault: pda(&[b"vault", &ROOT_UNIT.to_le_bytes(), mint.as_ref()]),
            treasury: pda(&[b"treasury", &ROOT_UNIT.to_le_bytes()]),
            engine: None,
        })
    }

    /// The sitting's pool: the params' tag and list length (a placeholder read as a stand-in, as a set written for reading reads one), the
    /// deployer its base, mainnet's stake pool program.
    pub fn of(p: &Params, deployer: &Pubkey, poleis: &Pubkey) -> Res<Self> {
        let tag = if p.is_placeholder("pool.tag") { "standin".to_string() } else { p.text("pool.tag")? };
        let mut pool = Self::derive(deployer, &tag, p.u64_or("pool.maxValidators", 10)?, &STAKE_POOL, poleis)?;
        pool.engine = crate::engine::staker_standing(p);
        Ok(pool)
    }

    pub fn seed(&self, c: char) -> String {
        format!("{}{c}", self.tag)
    }

    /// The two made in the execute's own transaction, before it, by the key that executes: the pool's account and its list.
    pub fn in_the_execute(&self) -> Vec<Seeded> {
        vec![
            Seeded { what: "the pool's account".into(), seed: self.seed('p'), space: POOL_LEN, owner: self.program },
            Seeded { what: format!("its validator list for {} validators", self.max_validators), seed: self.seed('l'), space: list_len(self.max_validators), owner: self.program },
        ]
    }
    /// The two made ahead, in the first sitting, by the deployer.
    pub fn reserve_seeded(&self) -> Seeded {
        Seeded { what: "the pool's reserve".into(), seed: self.seed('r'), space: RESERVE_LEN, owner: STAKE }
    }
    pub fn mint_seeded(&self) -> Seeded {
        Seeded { what: "the pool's mint".into(), seed: self.seed('m'), space: MINT_LEN, owner: TOKEN }
    }

    /// StakePoolInstruction::Initialize as devnet's birth sent it: [0][the epoch fee: denominator 1, numerator 1][the withdrawal fee 0/0]
    /// [the deposit fee 0/0][the referral fee 0][max validators: u32].
    pub fn initialize_data(&self) -> Vec<u8> {
        let fee = |numerator: u64, denominator: u64| [denominator.to_le_bytes(), numerator.to_le_bytes()].concat();
        let mut d = vec![0u8];
        d.extend(fee(1, 1));
        d.extend(fee(0, 0));
        d.extend(fee(0, 0));
        d.push(0);
        d.extend_from_slice(&self.max_validators.to_le_bytes());
        d
    }
    /// Its nine accounts with the raw act's flags, as devnet's: the pool (w), the manager (the polis's hand, signing), the staker (the same),
    /// the withdraw authority, the list (w), the reserve, the mint (w), the fee account (w), the token program. No deposit authority is
    /// named: anyone may stake.
    pub fn initialize_accounts(&self) -> Vec<(Pubkey, u8)> {
        vec![
            (self.pool, ACT_WRITABLE),
            (self.agent, ACT_SIGNER),
            (self.agent, 0),
            (self.withdraw, 0),
            (self.list, ACT_WRITABLE),
            (self.reserve, 0),
            (self.mint, ACT_WRITABLE),
            (self.vault, ACT_WRITABLE),
            (TOKEN, 0),
        ]
    }
    /// The raw act of root 0 whose execute is the pool's birth.
    pub fn payload(&self) -> Payload {
        Payload::Raw { origin: ROOT_UNIT, program: self.program, accounts: self.initialize_accounts(), data: self.initialize_data() }
    }

    /// The stake program's Initialize of the reserve (its tag 0): its staker and its withdrawer the pool's withdraw authority, no lockup ·
    /// [the reserve (w), the rent sysvar].
    pub fn reserve_init(&self) -> Instruction {
        let mut d = 0u32.to_le_bytes().to_vec();
        d.extend_from_slice(self.withdraw.as_ref());
        d.extend_from_slice(self.withdraw.as_ref());
        d.extend_from_slice(&[0u8; 48]);
        Instruction::new_with_bytes(STAKE, &d, vec![AccountMeta::new(self.reserve, false), AccountMeta::new_readonly(RENT_SYSVAR, false)])
    }
    /// SPL Token's InitializeMint2 of the mint: nine decimals, the withdraw authority its mint authority, no freeze authority · [the mint (w)].
    pub fn mint_init(&self) -> Instruction {
        let mut d = vec![warm_mine::token::IX_INITIALIZE_MINT2, DECIMALS];
        d.extend_from_slice(self.withdraw.as_ref());
        d.push(0);
        Instruction::new_with_bytes(TOKEN, &d, vec![AccountMeta::new(self.mint, false)])
    }
    /// THE FIRST SITTING'S PART: what it sends for the three made ahead (the reserve, the mint, root 0's vault of the mint), of those that do
    /// not stand as the pool's, the base paying, and their words. `standing` is what the chain holds at their three addresses; one standing
    /// that is not the pool's (another authority, another owner) stops the step with its words. Lamports a stranger sent to an address are no
    /// account: made over them. The pool's account and its list are never made here.
    pub fn ahead(&self, reserve_rent: u64, mint_rent: u64, standing: [Option<&Account>; 3]) -> Res<(Vec<&'static str>, Vec<Instruction>)> {
        let (mut made, mut ixs) = (Vec::new(), Vec::new());
        let [r, m, v] = standing;
        match r {
            Some(a) if !is_dust(a) => reserve_held(&a.owner, &a.data, &self.withdraw)?,
            _ => {
                ixs.extend(self.reserve_seeded().making(&self.base, reserve_rent, r)?);
                ixs.push(self.reserve_init());
                made.push("its reserve");
            }
        }
        match m {
            Some(a) if !is_dust(a) => mint_held(&a.owner, &a.data, &self.withdraw)?,
            _ => {
                ixs.extend(self.mint_seeded().making(&self.base, mint_rent, m)?);
                ixs.push(self.mint_init());
                made.push("its mint");
            }
        }
        match v {
            Some(a) if !is_dust(a) => vault_held(&a.owner, &a.data, &self.mint)?,
            // Poleis's create_vault tops up a stranger's lamports itself
            _ => {
                ixs.push(client::create_vault(self.base, ROOT_UNIT, self.mint, TOKEN));
                made.push("root 0's vault of its mint");
            }
        }
        Ok((made, ixs))
    }

    pub fn seeds_json(&self) -> Value {
        json!({ "pool": self.seed('p'), "list": self.seed('l'), "reserve": self.seed('r'), "mint": self.seed('m') })
    }
    pub fn addresses_json(&self) -> Value {
        json!({
            "program": self.program.to_string(), "base": self.base.to_string(), "pool": self.pool.to_string(), "list": self.list.to_string(),
            "reserve": self.reserve.to_string(), "mint": self.mint.to_string(), "withdrawAuthority": self.withdraw.to_string(),
            "depositAuthority": self.deposit.to_string(), "manager": self.agent.to_string(), "staker": self.agent.to_string(),
            "feeAccount": self.vault.to_string(), "feeAccountAuthority": self.treasury.to_string(),
        })
    }
}

fn key_at(d: &[u8], o: usize) -> Pubkey {
    Pubkey::new_from_array(d[o..o + 32].try_into().unwrap())
}
fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

/// THE RESERVE, held: the stake program's account, 200 bytes, initialized (its state 1, never delegated), its staker and its withdrawer the
/// pool's withdraw authority, no lockup ([state:4][rent reserve:8][staker:32][withdrawer:32][lockup: its time 8, its epoch 8, its custodian
/// 32]).
pub fn reserve_held(owner: &str, data: &[u8], withdraw: &Pubkey) -> Res<()> {
    if owner != STAKE.to_string() || data.len() != RESERVE_LEN {
        return Err(format!("the account at the pool's reserve is not a stake account ({owner}, {} bytes)", data.len()));
    }
    if data[..4] != [1, 0, 0, 0] {
        return Err("the pool's reserve is not an initialized stake account".into());
    }
    if key_at(data, 12) != *withdraw || key_at(data, 44) != *withdraw {
        return Err(format!("the pool's reserve answers to {} (its staker) and {} (its withdrawer), not to the pool's withdraw authority {withdraw}: no reserve of this pool", key_at(data, 12), key_at(data, 44)));
    }
    if data[76..124].iter().any(|b| *b != 0) {
        return Err("the pool's reserve carries a lockup: the pool takes none".into());
    }
    Ok(())
}

/// THE MINT, held: the token program's, initialized, nine decimals, the pool's withdraw authority its mint authority, no freeze authority
/// ([its mint authority: an option 4 and 32][supply:8][decimals:1][initialized:1][its freeze authority: an option 4 and 32]).
pub fn mint_held(owner: &str, data: &[u8], withdraw: &Pubkey) -> Res<()> {
    use warm_mine::token::{MINT_OFF_DECIMALS, MINT_OFF_FREEZE, MINT_OFF_INITIALIZED};
    if owner != TOKEN.to_string() || data.len() != MINT_LEN || data[MINT_OFF_INITIALIZED] != 1 {
        return Err(format!("the account at the pool's mint is not an initialized mint of SPL Token ({owner}, {} bytes)", data.len()));
    }
    if data[..4] != [1, 0, 0, 0] || key_at(data, 4) != *withdraw {
        return Err(format!("the pool's mint is minted by another authority than the pool's withdraw authority {withdraw}: no mint of this pool"));
    }
    if data[MINT_OFF_DECIMALS] != DECIMALS {
        return Err(format!("the pool's mint has {} decimals, not {DECIMALS}", data[MINT_OFF_DECIMALS]));
    }
    if data[MINT_OFF_FREEZE..MINT_OFF_FREEZE + 4] != [0, 0, 0, 0] {
        return Err("the pool's mint has a freeze authority: the pool takes none".into());
    }
    Ok(())
}

/// THE POOL READ BACK: the stake pool program's account, 611 bytes, of account type 1 (a StakePool); its manager and its staker the polis's
/// hand, its deposit authority the program's default, its list, its reserve and its mint the sitting's, its fee account root 0's vault of
/// the mint, its token program SPL Token, its epoch fee 1/1 ([type:1][manager:32][staker:32][deposit authority:32][withdraw bump:1][list:32]
/// [reserve:32][mint:32][fee account:32][token program:32][total lamports:8][pool tokens:8][last update epoch:8][lockup:48][the epoch fee:
/// its denominator 8, its numerator 8]: the offsets birth-pool.mjs read devnet's by). Held whole, or the press does not say it stands.
pub fn pool_held(owner: &str, data: &[u8], pool: &Pool) -> Res<Value> {
    if owner != pool.program.to_string() || data.len() != POOL_LEN {
        return Err(format!("the account at the pool's address is not the stake pool program's pool ({owner}, {} bytes)", data.len()));
    }
    let read = json!({
        "accountType": data[0], "manager": key_at(data, 1).to_string(), "staker": key_at(data, 33).to_string(), "depositAuthority": key_at(data, 65).to_string(),
        "list": key_at(data, 98).to_string(), "reserve": key_at(data, 130).to_string(), "mint": key_at(data, 162).to_string(), "feeAccount": key_at(data, 194).to_string(),
        "tokenProgram": key_at(data, 226).to_string(), "totalLamports": u64_at(data, 258), "poolTokens": u64_at(data, 266),
        "epochFee": format!("{}/{}", u64_at(data, 338), u64_at(data, 330)),
    });
    let want: [(&str, Value); 10] = [
        ("accountType", json!(TYPE_POOL)),
        ("manager", json!(pool.agent.to_string())),
        ("staker", json!(pool.agent.to_string())),
        ("depositAuthority", json!(pool.deposit.to_string())),
        ("list", json!(pool.list.to_string())),
        ("reserve", json!(pool.reserve.to_string())),
        ("mint", json!(pool.mint.to_string())),
        ("feeAccount", json!(pool.vault.to_string())),
        ("tokenProgram", json!(TOKEN.to_string())),
        ("epochFee", json!("1/1")),
    ];
    for (field, v) in want {
        // THE STAKE EARNS: after the engine's act the staker is the engine's key the params name, the manager still the hand
        if field == "staker" && pool.engine.is_some_and(|e| read[field] == json!(e.to_string())) {
            continue;
        }
        if read[field] != v {
            return Err(format!("the pool's {field} is {}, not {v}: it is not the pool the polis's act makes", read[field]));
        }
    }
    Ok(read)
}

/// THE LIST READ BACK: the stake pool program's account of the params' length, of account type 2 (a ValidatorList), its max validators the
/// params'.
pub fn list_held(owner: &str, data: &[u8], pool: &Pool) -> Res<()> {
    let n = pool.max_validators;
    if owner != pool.program.to_string() || data.len() != list_len(n) || data[0] != TYPE_LIST || data[1..5] != n.to_le_bytes() {
        return Err(format!("the account at the pool's list is not its validator list for {n} validators ({owner}, {} bytes)", data.len()));
    }
    Ok(())
}

/// The stake pool program as a chain holds it: its program data's address, its last deploy's slot, its upgrade authority, and its executable
/// bytes (those after the program data's 45-byte header, the zeros of its room trimmed from their end, the way a verified build's hash is
/// read): their length and their sha256.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub id: Pubkey,
    pub programdata: Pubkey,
    pub slot: u64,
    pub authority: Option<Pubkey>,
    pub bytes: usize,
    pub sha256: [u8; 32],
}

impl Program {
    /// The program `id` from its program data's account bytes.
    pub fn of(id: &Pubkey, programdata: &[u8]) -> Res<Self> {
        let State::ProgramData { slot, authority } = loader::state(programdata)? else { return Err(format!("{id}: its program data is not one")) };
        let code = &programdata[loader::PROGRAMDATA_HEADER.min(programdata.len())..];
        let end = code.iter().rposition(|b| *b != 0).map(|i| i + 1).unwrap_or(0);
        Ok(Self { id: *id, programdata: loader::programdata_of(id), slot, authority, bytes: end, sha256: wire::sha256(&code[..end]) })
    }
    pub fn json(&self) -> Value {
        json!({
            "program": self.id.to_string(), "programData": self.programdata.to_string(), "deploySlot": self.slot,
            "authority": self.authority.map(|a| a.to_string()), "bytes": self.bytes, "sha256": wire::hex(&self.sha256),
        })
    }
    /// What `json` wrote (the record's `poolProgram`), read back; None where nothing was recorded.
    pub fn from_json(v: &Value) -> Res<Option<Self>> {
        if v.is_null() {
            return Ok(None);
        }
        let bad = |f: &str| format!("the record's poolProgram: its {f}");
        let key = |f: &str| v[f].as_str().and_then(|s| Pubkey::from_str(s).ok()).ok_or_else(|| bad(f));
        Ok(Some(Self {
            id: key("program")?,
            programdata: key("programData")?,
            slot: v["deploySlot"].as_u64().ok_or_else(|| bad("deploySlot"))?,
            authority: match &v["authority"] {
                Value::Null => None,
                _ => Some(key("authority")?),
            },
            bytes: v["bytes"].as_u64().ok_or_else(|| bad("bytes"))? as usize,
            sha256: wire::unhex(v["sha256"].as_str().ok_or_else(|| bad("sha256"))?).map_err(|_| bad("sha256"))?,
        }))
    }
}

/// THE BYTES THE BIRTH WAS REHEARSED ON: the program standing now, held to the sitting's first read of it (`recorded`) and to the hash the
/// params name (`named`; on mainnet the one a rehearsal's certificate names, his to fill). Only the bytes are held: the same bytes deployed
/// at another slot are the same program (a rehearsal's copy stands at the slot its validator gave it).
pub fn program_held(now: &Program, recorded: Option<&Program>, named: Option<[u8; 32]>) -> Res<()> {
    if let Some(r) = recorded {
        if r.sha256 != now.sha256 {
            return Err(format!(
                "the stake pool program {} runs other bytes than the sitting recorded: sha256 {} ({} bytes, deployed at slot {}), recorded {} ({} bytes, slot {}). It was upgraded since the sitting first read it: nothing of the pool is sent. A rehearsal of the new bytes comes first, then his word on them (the params' pool.programSha256)",
                now.id, wire::hex(&now.sha256), now.bytes, now.slot, wire::hex(&r.sha256), r.bytes, r.slot
            ));
        }
    }
    if let Some(h) = named {
        if h != now.sha256 {
            return Err(format!(
                "the stake pool program {} runs bytes whose sha256 is {} ({} bytes, deployed at slot {}), not the {} the params name (pool.programSha256, the bytes the birth was rehearsed on): nothing of the pool is sent",
                now.id, wire::hex(&now.sha256), now.bytes, now.slot, wire::hex(&h)
            ));
        }
    }
    Ok(())
}

/// The hash the params name: none (a rehearsal records what it reads), or 64 hex. On mainnet it is named: the pool is born on the bytes a
/// rehearsal proved, or not at all. A placeholder reads as none (a set written for reading; nothing is sent while one stands).
pub fn named_sha(p: &Params) -> Res<Option<[u8; 32]>> {
    let v = p.at("pool.programSha256");
    if v.is_null() {
        if p.cluster == Cluster::Mainnet {
            return Err("params pool.programSha256: on mainnet the pool is born on the bytes a rehearsal proved, so the params name their sha256 (a rehearsal's certificate: pool.programRecorded.sha256)".into());
        }
        return Ok(None);
    }
    if p.is_placeholder("pool.programSha256") {
        return Ok(None);
    }
    let s = v.as_str().ok_or("params pool.programSha256: the sha256 of the program's executable bytes, 64 hex")?;
    wire::unhex::<32>(s).map(Some).map_err(|e| format!("params pool.programSha256: {e}"))
}

/// Where the pool stands, for the handoff.
#[derive(Clone, Debug, PartialEq)]
pub enum PoolAt {
    /// born, and read back as root 0's: what it reads
    Born(Value),
    /// not born yet (nothing at its address, or a stranger's lamports)
    Unborn,
    /// not to be born on this chain: no stake pool program stands here
    NoProgram,
}

impl Sitting {
    pub fn pool_of(&self) -> Res<Pool> {
        Pool::of(&self.p, &self.deployer, &self.ids.poleis)
    }

    /// Whether an executable stake pool program stands on this chain (the need's question, as `metaplex_stands` is the face's).
    pub fn stake_pool_stands(&self) -> Res<bool> {
        Ok(self.account(&STAKE_POOL)?.is_some_and(|a| a.executable))
    }

    /// The stake pool program as it stands on this chain, its bytes read: None while no executable program stands at its id.
    pub fn stake_pool_program(&self) -> Res<Option<Program>> {
        let Some(a) = self.account(&STAKE_POOL)? else { return Ok(None) };
        if !a.executable {
            return Ok(None);
        }
        if a.owner != LOADER_V3.to_string() {
            return Err(format!("the stake pool program {STAKE_POOL} is not a program of the upgradeable loader on this chain (its owner {}): its bytes are not read as mainnet's are", a.owner));
        }
        let State::Program { programdata } = loader::state(&a.data)? else { return Err(format!("{STAKE_POOL}: not a program account")) };
        let pd = self.account(&programdata)?.ok_or("the stake pool program's program data is not on this chain")?;
        Program::of(&STAKE_POOL, &pd.data).map(Some)
    }

    /// THE PROGRAM, RECORDED AND HELD: read; recorded the first time (`poolProgram`: its last deploy's slot and the sha256 of its executable
    /// bytes, the step that read it); held after to that record and to the params' hash, or the step stops with a plain sentence.
    pub fn pool_program(&mut self, step: &str) -> Res<Option<Program>> {
        let named = named_sha(&self.p)?;
        let recorded = Program::from_json(self.rec.get("poolProgram"))?;
        let Some(now) = self.stake_pool_program()? else {
            if let Some(r) = &recorded {
                return Err(format!("the stake pool program the sitting recorded ({}, sha256 {}) does not stand on this chain now: nothing of the pool is sent", r.id, wire::hex(&r.sha256)));
            }
            return Ok(None);
        };
        program_held(&now, recorded.as_ref(), named)?;
        if recorded.is_none() {
            let mut v = now.json();
            v["readBy"] = json!(step);
            v["readAt"] = json!(crate::chain::now_iso());
            self.rec.v["poolProgram"] = v;
            self.rec.save()?;
            eprintln!("  the stake pool program {STAKE_POOL} recorded: {} executable bytes, sha256 {}, last deployed at slot {}", now.bytes, wire::hex(&now.sha256), now.slot);
        }
        Ok(Some(now))
    }

    /// PRESS POOL, the first sitting's part of the pool: its reserve, its mint and root 0's vault of the mint made ahead by the deployer (one
    /// transaction), read back and held; what stands is skipped. The pool itself is born by act 5 (press acts writes its set, press crank
    /// carries it out, its account and its list made in the execute's own transaction).
    pub fn pool(&mut self) -> Res<()> {
        self.p.sendable()?;
        let pool = self.pool_of()?;
        let program = self.pool_program("pool")?;
        if program.is_none() && self.p.cluster == Cluster::Mainnet {
            return Err(NOT_MAINNET.into());
        }
        self.account(&pool.treasury)?.ok_or("root 0's treasury is not on the cluster: the founding comes first")?;
        let standing = [self.account(&pool.reserve)?, self.account(&pool.mint)?, self.account(&pool.vault)?];
        let (made, ixs) = pool.ahead(self.rpc.rent(RESERVE_LEN)?, self.rpc.rent(MINT_LEN)?, [standing[0].as_ref(), standing[1].as_ref(), standing[2].as_ref()])?;
        if ixs.is_empty() {
            if self.rec.get("pool")["ahead"].is_null() {
                self.rec.resumed(format!("the pool's reserve {}, its mint {} and root 0's vault of it {} stood already", pool.reserve, pool.mint, pool.vault))?;
            }
        } else {
            let keys = self.keys.load(&["deployer"], "the pool's reserve, mint and vault")?;
            let what = format!("the pool made ahead, the deployer paying: {} (the pool's account and its list are made only in act 5's execute)", made.join(", "));
            let l = self.send("pool", &what, [crate::chain::budget_of(&self.p, 60_000)?, ixs].concat(), &[&keys[0]])?;
            self.rec.v["pool"]["ahead"] = json!({ "made": made, "signature": l.signature, "slot": l.slot, "bytes": l.bytes, "units": l.units });
        }
        let r = self.account(&pool.reserve)?.ok_or("the pool's reserve is not on the cluster after its making")?;
        reserve_held(&r.owner, &r.data, &pool.withdraw)?;
        let m = self.account(&pool.mint)?.ok_or("the pool's mint is not on the cluster after its making")?;
        mint_held(&m.owner, &m.data, &pool.withdraw)?;
        let v = self.account(&pool.vault)?.ok_or("root 0's vault of the pool's mint is not on the cluster after its making")?;
        vault_held(&v.owner, &v.data, &pool.mint)?;
        eprintln!(
            "  the pool's reserve {} stands ({} lamports, its staker and withdrawer the pool's withdraw authority {}), its mint {} (nine decimals, no freeze authority) and root 0's vault of it {} (its authority root 0's treasury)",
            pool.reserve, r.lamports, pool.withdraw, pool.mint, pool.vault
        );
        self.rec.v["pool"]["tag"] = json!(pool.tag);
        self.rec.v["pool"]["maxValidators"] = json!(pool.max_validators);
        self.rec.v["pool"]["seeds"] = pool.seeds_json();
        self.rec.v["pool"]["addresses"] = pool.addresses_json();
        self.rec.v["pool"]["reserveLamports"] = json!(r.lamports);
        self.rec.save()?;
        match program {
            Some(p) => println!(
                "THE POOL, AHEAD: its reserve, its mint and root 0's vault of it stand; its birth is act 5, the polis's raw act (press acts writes its set; press crank's execute makes the pool {} and its list {} in its own transaction, the deployer its base), on the stake pool program's bytes of sha256 {}",
                pool.pool, pool.list, wire::hex(&p.sha256)
            ),
            None => println!("{NOT_RUN}: its reserve, its mint and root 0's vault of it stand; the pool and its list are made only in act 5's execute, on a chain that has the program"),
        }
        Ok(())
    }

    /// Where the pool stands: born and held as root 0's (its list too); not born (nothing, or a stranger's lamports, at its address); or not
    /// to be born on this chain (no stake pool program stands here).
    pub fn pool_at(&self) -> Res<PoolAt> {
        let pool = self.pool_of()?;
        match self.account(&pool.pool)? {
            Some(a) if !is_dust(&a) => {
                let read = pool_held(&a.owner, &a.data, &pool)?;
                let l = self.account(&pool.list)?.ok_or("the pool stands, and its validator list does not")?;
                list_held(&l.owner, &l.data, &pool)?;
                Ok(PoolAt::Born(read))
            }
            _ if !self.stake_pool_stands()? => Ok(PoolAt::NoProgram),
            _ => Ok(PoolAt::Unborn),
        }
    }
}
