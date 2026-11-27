//! THE STAKE EARNS (Nov 25 2026, lane/the-stake-earns; his "Yes, build them" to "Build the three missing links now?", link 3). Her pool's
//! stake placed on validators so it earns, and the yield arriving in root 0's vault as the pool's token (the epoch fee 1/1 the launch's act
//! 5 gave it). His words it stands on, UTC: Nov 20 20:18 "we are supposed to delegate automatically like marinade does but with our own
//! rules"; the staker role is "a Poleis address" (Oct 14, by the Nov 20 doc's reading).
//!
//! THE SHAPE, and the fact that decided it. Poleis signs as the polis's hand (`agent_pda`) in one place only: the execute of a sealed
//! ACT_RAW (program/poleis_treasury/src/ix/proposal.rs, `invoke_signed` with the seeds `agent`, the unit). So while the hand is the pool's
//! staker, every placement is an act of the whole tree: a proposal by a seat owner, unit 1's agents' envelopes, the floor's own votes, two
//! seals, a mandate and the execute, at least seven transactions and a vote window (a day on mainnet) for each move, and one transient move
//! a validator an epoch. No instruction of the program lets the hand sign without a sealed act, so no third way keeps the staker a PDA and
//! runs each epoch without a vote. The engine is therefore one act of the swarm (`ENGINE_ACT`, the ninth, a later act voted after the
//! launch's eight): the stake pool's SetStaker, signed by the manager (the hand, by Poleis, once sealed), naming the params' `engine.staker`,
//! the engine's crank key, the way act 6 names the $WARM mine's miner. The stake pool program bounds a staker: it adds and removes
//! validators and moves stake between the reserve and them, and never withdraws a lamport; the manager (still the hand) renames it by
//! another act.
//!
//! THE RULE IS DATA (`Rule`, the params' `engine`): the validators (vote accounts) and each one's weight, and the share of the pool kept in
//! the reserve (`reserveBp`) so a staker can still leave at once. On mainnet they are his to name (PLACEHOLDER); the rehearsal names its
//! validator's own vote account by a word (`THIS VALIDATOR`), resolved only on a validator of this machine. No address of a validator is in
//! this code.
//!
//! THE CRANK (`next`, pure; `run`, on a chain): the pool brought to the epoch (every validator's pair named, five a transaction), each
//! validator of the rule the list does not hold added (AddValidatorToPool, the reserve funding its stake account), the reserve above its kept
//! share moved onto the validators by their weights (IncreaseValidatorStake), and, when the reserve is below its kept share, stake brought back
//! (DecreaseValidatorStakeWithReserve, the variant the program's current version keeps) so the way out stays open. One transient move a
//! validator an epoch: a validator whose transient stands is not moved. A second run in the same epoch finds nothing due and says so.

use crate::acts::{self, Act};
use crate::build::Ids;
use crate::chain::Addresses;
use crate::params::{Params, PLACEHOLDER, STAND_IN};
use crate::pool::{self, Pool, STAKE};
use crate::rpc::{Cluster, Rpc};
use crate::wire::{self, Key};
use crate::Res;
use poleis_client::{self as client, Payload, ACT_SIGNER, ACT_WRITABLE, ROOT_UNIT};
use poleis_door::{Keys, CLOCK_SYSVAR, STAKE_HISTORY};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::str::FromStr as _;


/// The engine's act: the ninth, voted after the launch's eight.
pub const ENGINE_ACT: usize = 9;

/// The stake pool program's instructions the engine sends, by their index (its v2 enum; held to the program's own words by the rehearsal).
pub const ADD_VALIDATOR: u8 = 1;
pub const INCREASE: u8 = 4;
pub const SET_STAKER: u8 = 13;
pub const DECREASE_WITH_RESERVE: u8 = 21;
pub const UPDATE_LIST: u8 = poleis_door::UPDATE_LIST;
pub const UPDATE_POOL: u8 = poleis_door::UPDATE_POOL;
/// The stake program's GetMinimumDelegation (its tag 13): what a delegation must hold at least, as its return data.
pub const GET_MINIMUM_DELEGATION: u32 = 13;

pub const RENT_SYSVAR: Pubkey = crate::loader::RENT_SYSVAR;
pub const SYSTEM: Pubkey = crate::loader::SYSTEM;
/// The stake config account the program still names (deprecated by the stake program, read by none).
pub const STAKE_CONFIG: Pubkey = solana_pubkey::pubkey!("StakeConfig11111111111111111111111111111111");
/// The pool program's own floor of a validator's active stake (its MINIMUM_ACTIVE_STAKE): a delegation holds the larger of it and the stake
/// program's minimum.
pub const MINIMUM_ACTIVE_STAKE: u64 = 1_000_000;
/// Validators a transaction of the list's update names (each with its transient: two accounts a validator).
pub const UPDATE_CHUNK: usize = poleis_door::PAIRS_AN_INSTRUCTION;
pub const BPS: u64 = 10_000;

/// The pool account: its staker after its manager.
pub const STAKER: usize = 33;
/// A validator's entry in the list: [active:8][transient:8][last update epoch:8][transient seed:8][unused:4][validator seed:4][status:1][vote:32].
pub const ENTRY_LEN: usize = pool::VALIDATOR_LEN;

/// The rehearsal's word for its validator's own vote account: resolved only on a validator of this machine.
pub const THIS_VALIDATOR: &str = "THIS VALIDATOR";
/// The rehearsal's word for the engine's key: the engine rehearsal makes its own, so no later act is written from these params.
pub const THIS_REHEARSAL: &str = "THIS REHEARSAL";

/// One validator of the rule: its vote account and its weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Validator {
    pub vote: Pubkey,
    pub weight: u64,
}

/// THE RULE: the validators and their weights, and the share of the pool's lamports kept in the reserve, in basis points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub validators: Vec<Validator>,
    pub reserve_bp: u64,
}

impl Rule {
    /// The params' `engine`, held: a list of one or more validators, each a vote account and a weight above 0, none twice; a reserve share of
    /// 0 to 10,000 bp. A placeholder is no rule yet (his to name), and the engine sends nothing. `own` is the validator's own vote account the
    /// rehearsal's word names, read off a validator of this machine.
    pub fn of(p: &Params, own: Option<&Pubkey>) -> Res<Self> {
        for path in ["engine.validators", "engine.reserveBp"] {
            if p.at(path).is_null() {
                return Err(format!("params {path}: the engine's rule names it"));
            }
            if p.is_placeholder(path) {
                return Err(format!("params {path} is not his yet ({}): the engine places nothing", p.at(path)));
            }
        }
        let reserve_bp = p.u64("engine.reserveBp")?;
        if reserve_bp > BPS {
            return Err(format!("params engine.reserveBp: {reserve_bp}; the share kept in the reserve is 0 to {BPS} basis points"));
        }
        let list = p.at("engine.validators").as_array().ok_or("params engine.validators: a list of {vote, weight}")?;
        if list.is_empty() {
            return Err("params engine.validators: the rule names at least one validator".into());
        }
        let mut validators: Vec<Validator> = Vec::new();
        for (i, v) in list.iter().enumerate() {
            let at = format!("engine.validators[{i}]");
            let words = v["vote"].as_str().ok_or_else(|| format!("params {at}.vote: a vote account's address"))?;
            if words.starts_with(PLACEHOLDER) || words.starts_with(STAND_IN) {
                return Err(format!("params {at}.vote is not his yet ({words}): the engine places nothing"));
            }
            let vote = if words.starts_with(THIS_VALIDATOR) {
                if p.cluster != Cluster::Localnet {
                    return Err(format!("params {at}.vote: {THIS_VALIDATOR} is a rehearsal's word, on a validator of this machine; mainnet's rule names its validators"));
                }
                *own.ok_or_else(|| format!("params {at}.vote: {THIS_VALIDATOR}, and this validator's own vote account was not read"))?
            } else {
                Pubkey::from_str(words).map_err(|_| format!("params {at}.vote: {words:?} is not an address"))?
            };
            let weight = v["weight"].as_u64().ok_or_else(|| format!("params {at}.weight: a whole number above 0"))?;
            if weight == 0 {
                return Err(format!("params {at}.weight: 0; a validator of the rule weighs more than nothing (take it out of the list instead)"));
            }
            if validators.iter().any(|x| x.vote == vote) {
                return Err(format!("params {at}.vote: {vote} twice; a validator stands once in the rule"));
            }
            validators.push(Validator { vote, weight });
        }
        Ok(Self { validators, reserve_bp })
    }
    pub fn weight(&self) -> u64 {
        self.validators.iter().map(|v| v.weight).sum()
    }
    /// The lamports the reserve keeps of the pool's `total`: its share, rounded up.
    pub fn kept(&self, total: u64) -> u64 {
        ((total as u128 * self.reserve_bp as u128).div_ceil(BPS as u128)) as u64
    }
    /// What validator `i` of the rule holds by its weight of what is placed (`total` less the kept share).
    pub fn target(&self, i: usize, total: u64) -> u64 {
        let placed = total - self.kept(total);
        (placed as u128 * self.validators[i].weight as u128 / self.weight() as u128) as u64
    }
}

/// The engine's key the params name: none (no `engine.staker`, or the rehearsal's word), a placeholder's address (a set written for reading),
/// or its address.
pub fn staker_named(p: &Params) -> Res<Option<Pubkey>> {
    let v = p.at("engine.staker");
    if v.is_null() {
        return Ok(None);
    }
    let s = v.as_str().ok_or("params engine.staker: the engine's crank key's address")?;
    if s.starts_with(THIS_REHEARSAL) {
        if p.cluster != Cluster::Localnet {
            return Err(format!("params engine.staker: {THIS_REHEARSAL} is a rehearsal's word; mainnet's names the key press makes at the sitting"));
        }
        return Ok(None);
    }
    p.key_or_standin("engine.staker").map(Some)
}

/// The staker a pool may stand under once the engine's act has executed: the params' engine key, where it is an address (not a placeholder).
pub fn staker_standing(p: &Params) -> Option<Pubkey> {
    if p.is_placeholder("engine.staker") {
        return None;
    }
    staker_named(p).ok().flatten()
}

/// The stake pool's SetStaker: [13] · [the pool (w), its manager or its staker (s), the new staker].
pub fn set_staker(pool: &Pubkey, program: &Pubkey, authority: &Pubkey, new: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(*program, &[SET_STAKER], vec![AccountMeta::new(*pool, false), AccountMeta::new_readonly(*authority, true), AccountMeta::new_readonly(*new, false)])
}

/// The engine's act's payload: root 0's raw act on the stake pool program, its manager the polis's hand, signing by Poleis once sealed.
pub fn payload(pool: &Pool, staker: &Pubkey) -> Payload {
    Payload::Raw { origin: ROOT_UNIT, program: pool.program, accounts: vec![(pool.pool, ACT_WRITABLE), (pool.agent, ACT_SIGNER), (*staker, 0)], data: vec![SET_STAKER] }
}

pub const ACT_NAME: &str = "the engine's staker (SetStaker, by the polis's hand)";
pub const ACT_FILE: &str = "9-the-engine.json";

/// THE ENGINE'S ACT, the ninth: written, proposed, voted and executed as the launch's acts are, after them (its nonce `acts.nonce + 8`).
pub fn act(p: &Params, ids: &Ids, at: &Addresses, deployer: &Pubkey, staker: &Pubkey) -> Res<Act> {
    let pool = Pool::of(p, deployer, &ids.poleis)?;
    let act = payload(&pool, staker);
    let manifest = wire::sha256(format!("THE LAUNCH PRESS: {ACT_NAME}").as_bytes());
    let nonce = p.u64_or("acts.nonce", 1)? + ENGINE_ACT as u64 - 1;
    let h = client::proposal_hash(act.class(), ROOT_UNIT, &manifest, &act.hash(), nonce);
    let impacted = acts::impacted_of(act.class());
    Ok(Act {
        n: ENGINE_ACT,
        name: ACT_NAME,
        file: ACT_FILE,
        class: act.class(),
        payload: act.bytes(),
        payload_sha256: act.hash(),
        manifest,
        nonce,
        hash: h,
        execute: client::execute_of(ROOT_UNIT, *deployer, h, &impacted, &act),
        impacted,
        before: Vec::new(),
        needs: Some(pool.program),
        does: vec![
            format!(
                "name the staker of her pool {}: {staker} (the params' engine.staker {}), the engine's crank key, which adds the rule's validators and moves her stake between the reserve and them each epoch, by the rule the params' engine names; the stake pool program lets a staker never withdraw a lamport",
                pool.pool, p.at("engine.staker")
            ),
            format!("the polis's own hand {}, the pool's manager since its birth, signs it, by Poleis, once the act is sealed; it stays the manager, and renames the staker by another act", at.agent),
        ],
    })
}

/// What the ninth act and the engine's steps say while the params' `engine` holds a blank.
pub const NOT_NAMED: &str = "the engine's rule is not named yet";

/// THE ENGINE'S BLANKS HOLD ONLY THE ENGINE (params.rs LATER): its steps refuse while one stands, naming each; the launch's do not see them.
pub fn blanks(p: &Params) -> Res<()> {
    let left = p.later_placeholders("engine");
    if left.is_empty() {
        return Ok(());
    }
    Err(format!("{NOT_NAMED} ({}): nothing of the engine or its act is sent", left.join(", ")))
}

/// The later acts the params name: the engine's, where `engine.staker` names a key and no blank of the engine's stands; none otherwise
/// (`later_said` says why, where the engine's rule waits on a blank); then THE LIGHT'S PLAN, the tenth (light.rs), where `light.planCap` names a figure.
pub fn later(p: &Params, ids: &Ids, at: &Addresses, deployer: &Pubkey) -> Res<Vec<Act>> {
    let mut out = Vec::new();
    if blanks(p).is_ok() {
        if let Some(s) = staker_named(p)? {
            out.push(act(p, ids, at, deployer, &s)?);
        }
    }
    out.extend(crate::light::later(p, ids, deployer)?);
    Ok(out)
}

/// Why the ninth act is not among the acts, where the params' engine holds a blank: what the crank, a vote and a chair say of it.
pub fn later_said(p: &Params) -> Option<String> {
    blanks(p).err()
}

/// The engine's act's set, as `press acts` writes each of the eight: the act, its payload and hash, unit 1's agents' envelopes, and the
/// staker it names. Written into `dir` beside the others.
pub fn write_set(dir: &std::path::Path, p: &Params, ids: &Ids, at: &Addresses, genesis: &str, deployer: &Pubkey) -> Res<std::path::PathBuf> {
    blanks(p)?;
    let staker = staker_named(p)?.ok_or("params engine.staker: the engine's crank key (press engine --make-key makes it), or no act names it")?;
    let a = act(p, ids, at, deployer, &staker)?;
    let pool = Pool::of(p, deployer, &ids.poleis)?;
    let agents = p.agents()?;
    let placeholders = p.placeholders();
    let labels = acts::labels(ids, at, &agents);
    let envelopes: Vec<Value> = agents.iter().enumerate().map(|(i, v)| acts::envelope(1 + i, v, &a.hash, &labels)).collect();
    let set = json!({
        "what": format!("THE LAUNCH PRESS, a later act, {}: {}. THE WHOLE TREE DECIDES, as for the launch's eight: a seat owner of root 0's floor proposes it (press vote --act {} --propose); unit 1's agents sign their envelopes; the floor votes by its owners' own transactions; press crank sends the rest, after the eight have executed", a.n, a.name, a.n),
        "cluster": p.cluster.name(),
        "genesisHash": genesis,
        "signable": placeholders.is_empty(),
        "placeholders": placeholders,
        "emitted": crate::chain::now_iso(),
        "act": { "n": a.n, "name": a.name, "class": a.class, "hash": wire::hex(&a.hash), "payload": wire::b64(&a.payload), "payloadSha256": wire::hex(&a.payload_sha256), "manifest": wire::hex(&a.manifest), "nonce": a.nonce, "impacted": a.impacted, "convened": a.convene() },
        "does": a.does,
        "envelopes": envelopes,
        "engine": {
            "staker": staker.to_string(), "params": p.at("engine.staker"), "pool": pool.pool.to_string(), "manager": pool.agent.to_string(),
            "rule": { "validators": p.at("engine.validators"), "reserveBp": p.at("engine.reserveBp") },
            "crank": "press engine --params P --rpc URL --pool <the pool> --key <the engine's key file>: each epoch, the pool brought to it, the rule's validators added, the reserve above its kept share placed by weight, stake brought back when the reserve is below it",
        },
    });
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(a.file);
    std::fs::write(&path, serde_json::to_string_pretty(&set).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

// ---- the list, the pool's head, the addresses ----

/// One validator of the pool's list, as the program writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub active: u64,
    pub transient: u64,
    pub last_update_epoch: u64,
    pub transient_seed: u64,
    pub validator_seed: u32,
    pub status: u8,
    pub vote: Pubkey,
}

fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

/// The list's entries and its room: [type 2][max:4][len:4], then 73 bytes an entry.
pub fn list_of(data: &[u8]) -> Res<(u32, Vec<Entry>)> {
    if data.len() < pool::LIST_HEADER || data[0] != pool::TYPE_LIST {
        return Err(format!("not a validator list ({} bytes)", data.len()));
    }
    let max = u32::from_le_bytes(data[1..5].try_into().unwrap());
    let n = u32::from_le_bytes(data[5..9].try_into().unwrap()) as usize;
    if data.len() < pool::LIST_HEADER + n * ENTRY_LEN {
        return Err(format!("the validator list says {n} entries and holds {} bytes", data.len()));
    }
    let entries = (0..n)
        .map(|i| {
            let e = &data[pool::LIST_HEADER + i * ENTRY_LEN..pool::LIST_HEADER + (i + 1) * ENTRY_LEN];
            Entry {
                active: u64_at(e, 0),
                transient: u64_at(e, 8),
                last_update_epoch: u64_at(e, 16),
                transient_seed: u64_at(e, 24),
                validator_seed: u32::from_le_bytes(e[36..40].try_into().unwrap()),
                status: e[40],
                vote: Pubkey::new_from_array(e[41..73].try_into().unwrap()),
            }
        })
        .collect();
    Ok((max, entries))
}

/// A validator's stake account of the pool: [vote, pool] and its seed when it has one.
pub fn validator_stake(program: &Pubkey, vote: &Pubkey, pool: &Pubkey, seed: u32) -> Pubkey {
    let s = seed.to_le_bytes();
    let mut seeds: Vec<&[u8]> = vec![vote.as_ref(), pool.as_ref()];
    if seed != 0 {
        seeds.push(&s);
    }
    Pubkey::find_program_address(&seeds, program).0
}

/// Its transient stake account: ["transient", vote, pool, seed].
pub fn transient_stake(program: &Pubkey, vote: &Pubkey, pool: &Pubkey, seed: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"transient", vote.as_ref(), pool.as_ref(), &seed.to_le_bytes()], program).0
}

// ---- the instructions ----

/// The pool brought to the epoch with every validator named: the door's own update (`poleis_door::update`, five validators a list
/// instruction), one list instruction a transaction, the pool's own totals in the last. Each inner list is one transaction.
pub fn update(k: &Keys, entries: &[Entry]) -> Vec<Vec<Instruction>> {
    let pairs: Vec<poleis_door::Pair> = entries
        .iter()
        .map(|e| (validator_stake(&k.program, &e.vote, &k.pool, e.validator_seed), transient_stake(&k.program, &e.vote, &k.pool, e.transient_seed)))
        .collect();
    let mut ixs = poleis_door::update(k, &pairs);
    let totals = ixs.pop().unwrap();
    let mut out: Vec<Vec<Instruction>> = ixs.into_iter().map(|i| vec![i]).collect();
    out.last_mut().unwrap().push(totals);
    out
}

/// AddValidatorToPool: [1][its seed: u32, 0 for none] · [the pool (w), the staker (s), the reserve (w, it funds the stake account), the
/// withdraw authority, the list (w), the validator's stake account (w), its vote account, rent, clock, stake history, stake config, system,
/// stake].
pub fn add(k: &Keys, staker: &Pubkey, vote: &Pubkey) -> Instruction {
    let mut d = vec![ADD_VALIDATOR];
    d.extend_from_slice(&0u32.to_le_bytes());
    Instruction::new_with_bytes(
        k.program,
        &d,
        vec![
            AccountMeta::new(k.pool, false),
            AccountMeta::new_readonly(*staker, true),
            AccountMeta::new(k.reserve, false),
            AccountMeta::new_readonly(k.withdraw, false),
            AccountMeta::new(k.list, false),
            AccountMeta::new(validator_stake(&k.program, vote, &k.pool, 0), false),
            AccountMeta::new_readonly(*vote, false),
            AccountMeta::new_readonly(RENT_SYSVAR, false),
            AccountMeta::new_readonly(CLOCK_SYSVAR, false),
            AccountMeta::new_readonly(STAKE_HISTORY, false),
            AccountMeta::new_readonly(STAKE_CONFIG, false),
            AccountMeta::new_readonly(SYSTEM, false),
            AccountMeta::new_readonly(STAKE, false),
        ],
    )
}

/// IncreaseValidatorStake: [4][lamports:8][transient seed:8] · [the pool, the staker (s), the withdraw authority, the list (w), the reserve
/// (w), the transient (w), the validator's stake account, its vote account, clock, rent, stake history, stake config, system, stake].
pub fn increase(k: &Keys, staker: &Pubkey, e: &Entry, lamports: u64) -> Instruction {
    let mut d = vec![INCREASE];
    d.extend_from_slice(&lamports.to_le_bytes());
    d.extend_from_slice(&e.transient_seed.to_le_bytes());
    Instruction::new_with_bytes(
        k.program,
        &d,
        vec![
            AccountMeta::new_readonly(k.pool, false),
            AccountMeta::new_readonly(*staker, true),
            AccountMeta::new_readonly(k.withdraw, false),
            AccountMeta::new(k.list, false),
            AccountMeta::new(k.reserve, false),
            AccountMeta::new(transient_stake(&k.program, &e.vote, &k.pool, e.transient_seed), false),
            AccountMeta::new_readonly(validator_stake(&k.program, &e.vote, &k.pool, e.validator_seed), false),
            AccountMeta::new_readonly(e.vote, false),
            AccountMeta::new_readonly(CLOCK_SYSVAR, false),
            AccountMeta::new_readonly(RENT_SYSVAR, false),
            AccountMeta::new_readonly(STAKE_HISTORY, false),
            AccountMeta::new_readonly(STAKE_CONFIG, false),
            AccountMeta::new_readonly(SYSTEM, false),
            AccountMeta::new_readonly(STAKE, false),
        ],
    )
}

/// DecreaseValidatorStakeWithReserve: [21][lamports:8][transient seed:8] · [the pool, the staker (s), the withdraw authority, the list (w),
/// the reserve (w, it funds the transient's rent), the validator's stake account (w), the transient (w), clock, stake history, system, stake].
pub fn decrease(k: &Keys, staker: &Pubkey, e: &Entry, lamports: u64) -> Instruction {
    let mut d = vec![DECREASE_WITH_RESERVE];
    d.extend_from_slice(&lamports.to_le_bytes());
    d.extend_from_slice(&e.transient_seed.to_le_bytes());
    Instruction::new_with_bytes(
        k.program,
        &d,
        vec![
            AccountMeta::new_readonly(k.pool, false),
            AccountMeta::new_readonly(*staker, true),
            AccountMeta::new_readonly(k.withdraw, false),
            AccountMeta::new(k.list, false),
            AccountMeta::new(k.reserve, false),
            AccountMeta::new(validator_stake(&k.program, &e.vote, &k.pool, e.validator_seed), false),
            AccountMeta::new(transient_stake(&k.program, &e.vote, &k.pool, e.transient_seed), false),
            AccountMeta::new_readonly(CLOCK_SYSVAR, false),
            AccountMeta::new_readonly(STAKE_HISTORY, false),
            AccountMeta::new_readonly(SYSTEM, false),
            AccountMeta::new_readonly(STAKE, false),
        ],
    )
}

// ---- the planner ----

/// What the engine reads of the chain before it moves anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    pub epoch: u64,
    /// the pool's last update epoch, its staker, its total lamports (the reserve's above its rent and every validator's)
    pub pool_epoch: u64,
    pub staker: Pubkey,
    pub total: u64,
    pub max_validators: u32,
    pub entries: Vec<Entry>,
    /// the reserve's lamports, its rent among them
    pub reserve: u64,
    /// a stake account's rent (the reserve's, a validator's, a transient's: 200 bytes each)
    pub stake_rent: u64,
    /// the least a delegation holds: the larger of the stake program's minimum and the pool's own
    pub min_delegation: u64,
    /// the epoch's rewards still being paid: nothing moves, and the pool cannot be brought to the epoch
    pub rewards_paying: bool,
}

impl Chain {
    /// The reserve's lamports above its rent.
    pub fn free(&self) -> u64 {
        self.reserve.saturating_sub(self.stake_rent)
    }
    pub fn behind(&self) -> bool {
        self.pool_epoch < self.epoch
    }
}

/// One move of the engine's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Move {
    Add { vote: Pubkey },
    Increase { vote: Pubkey, lamports: u64 },
    Decrease { vote: Pubkey, lamports: u64 },
    /// the transients' rents sent into a reserve a staker emptied to its own rent, by the engine's key
    Fund { lamports: u64 },
}

/// What the engine does next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// the pool behind the epoch: brought to it first
    Update,
    /// these moves, in this order (the adds first, in a transaction of their own)
    Moves(Vec<Move>, Vec<String>),
    /// nothing is due now, and why
    Nothing(Vec<String>),
    /// nothing can move now (the epoch's rewards being paid), and why
    Wait(String),
}

/// THE ENGINE'S ORDER, pure: the pool brought to the epoch; the rule's validators the list does not hold added, while it has room and the
/// reserve funds their stake accounts; then the reserve above its kept share placed by weight on the validators whose transient does not
/// stand, each move at least a delegation's minimum and paid with a transient's rent; or, with the reserve below its kept share, stake
/// brought back from the validators that hold the most above their floor. `engine` is the key the crank signs with: the pool's staker, or
/// nothing is sent.
pub fn next(rule: &Rule, c: &Chain, engine: &Pubkey) -> Res<Step> {
    if c.staker != *engine {
        return Err(format!(
            "the pool's staker is {}, not the engine's key {engine}: the engine's act (the ninth, SetStaker by the polis's hand) has not executed, or names another key",
            c.staker
        ));
    }
    if rule.validators.len() > c.max_validators as usize {
        return Err(format!("the rule names {} validators; her pool's list holds {} at most (fixed at its birth)", rule.validators.len(), c.max_validators));
    }
    if c.behind() {
        if c.rewards_paying {
            return Ok(Step::Wait(format!("the pool was last brought to epoch {} and the chain is at {}: the epoch's rewards are being paid, so nothing can bring it there yet", c.pool_epoch, c.epoch)));
        }
        return Ok(Step::Update);
    }
    let mut words = Vec::new();
    for e in &c.entries {
        if !rule.validators.iter().any(|v| v.vote == e.vote) {
            words.push(format!("{} stands in her list and not in the rule: its removal is owed (RemoveValidatorFromPool is not built), so its stake stays where it is", e.vote));
        }
    }
    // the adds: each stake account funded by the reserve with its rent and a delegation's minimum
    let absent: Vec<&Validator> = rule.validators.iter().filter(|v| !c.entries.iter().any(|e| e.vote == v.vote)).collect();
    if !absent.is_empty() {
        let each = c.stake_rent + c.min_delegation;
        let mut room = c.max_validators as usize - c.entries.len();
        let mut free = c.free();
        let mut moves = Vec::new();
        for v in absent {
            if room == 0 {
                words.push(format!("{}: her list is full ({} validators): not added", v.vote, c.max_validators));
                continue;
            }
            if free < each {
                words.push(format!("{}: the reserve holds {} lamports above its rent, and a validator's stake account takes {each}: not added yet", v.vote, free));
                continue;
            }
            free -= each;
            room -= 1;
            moves.push(Move::Add { vote: v.vote });
        }
        if !moves.is_empty() {
            return Ok(Step::Moves(moves, words));
        }
    }
    let kept = rule.kept(c.total);
    let free = c.free();
    let mut moves = Vec::new();
    if free >= kept {
        // the reserve above its kept share, placed by weight: each move is a transient funded with its rent besides the lamports placed
        let mut spare = free - kept;
        for (i, v) in rule.validators.iter().enumerate() {
            let Some(e) = c.entries.iter().find(|e| e.vote == v.vote) else { continue };
            if e.transient > 0 {
                words.push(format!("{}: a move of {} lamports is in flight this epoch: one transient a validator an epoch", v.vote, e.transient));
                continue;
            }
            if e.status != 0 {
                words.push(format!("{}: its status is {} (not active): not moved", v.vote, e.status));
                continue;
            }
            let want = rule.target(i, c.total).saturating_sub(e.active);
            let lamports = want.min(spare.saturating_sub(c.stake_rent));
            if lamports < c.min_delegation {
                continue;
            }
            spare -= lamports + c.stake_rent;
            moves.push(Move::Increase { vote: v.vote, lamports });
        }
    } else {
        // the way out kept open: stake brought back until the reserve holds its share again. The program funds each transient's rent from
        // the reserve; a reserve a staker emptied to its own rent cannot, so the engine's key sends that rent in first (a gift to the pool,
        // counted as yield at the next update and minted to the fee account like any)
        let mut short = kept - free;
        let floor = c.stake_rent + c.min_delegation;
        let mut by_spare: Vec<&Entry> = c.entries.iter().filter(|e| e.transient == 0 && e.status == 0 && e.active > floor).collect();
        by_spare.sort_by(|a, b| b.active.cmp(&a.active));
        let mut back = Vec::new();
        for e in by_spare {
            if short == 0 {
                break;
            }
            let lamports = short.max(c.min_delegation).min(e.active - floor);
            if lamports < c.min_delegation {
                continue;
            }
            short = short.saturating_sub(lamports);
            back.push(Move::Decrease { vote: e.vote, lamports });
        }
        let rents = c.stake_rent * back.len() as u64;
        if !back.is_empty() && free < rents {
            words.push(format!("the reserve holds {free} lamports above its rent and the transients' rents are {rents}: the engine's key sends the {} it lacks first", rents - free));
            moves.push(Move::Fund { lamports: rents - free });
        }
        moves.extend(back);
        if short > 0 && moves.is_empty() {
            words.push(format!("the reserve is {} lamports below its kept share and no validator can give stake back this epoch (a move in flight, or no stake above its floor)", kept - free));
        }
    }
    if moves.is_empty() {
        words.push(format!(
            "nothing is due at epoch {}: the pool holds {} lamports, the reserve {} above its rent against a kept share of {kept}",
            c.epoch, c.total, free
        ));
        return Ok(Step::Nothing(words));
    }
    Ok(Step::Moves(moves, words))
}

/// A move's instruction.
pub fn instruction(k: &Keys, engine: &Pubkey, m: &Move, entries: &[Entry]) -> Res<Instruction> {
    let entry = |vote: &Pubkey| entries.iter().find(|e| e.vote == *vote).ok_or_else(|| format!("{vote} is not in her list"));
    Ok(match m {
        Move::Add { vote } => add(k, engine, vote),
        Move::Increase { vote, lamports } => increase(k, engine, entry(vote)?, *lamports),
        Move::Decrease { vote, lamports } => decrease(k, engine, entry(vote)?, *lamports),
        Move::Fund { lamports } => crate::loader::transfer(engine, &k.reserve, *lamports),
    })
}

// ---- on a chain ----

/// The stake program's minimum delegation on this chain, by simulating its GetMinimumDelegation (its return data), paid for by `payer`.
pub fn stake_minimum(rpc: &Rpc, payer: &Pubkey) -> Res<u64> {
    let ix = Instruction::new_with_bytes(STAKE, &GET_MINIMUM_DELEGATION.to_le_bytes(), vec![]);
    let (m, _) = wire::message(&[ix], payer, &rpc.blockhash()?);
    let t = wire::transaction(&m, &[[0u8; 64]]);
    let v = rpc.call("simulateTransaction", json!([wire::b64(&t), { "encoding": "base64", "sigVerify": false, "replaceRecentBlockhash": true, "commitment": "confirmed" }]))?;
    let r = &v["value"];
    if !r["err"].is_null() {
        return Err(format!("the stake program's minimum delegation could not be read: {}", r["err"]));
    }
    let d = wire::unb64(r["returnData"]["data"][0].as_str().ok_or("the stake program's minimum delegation: no return data")?)?;
    Ok(u64::from_le_bytes(d.get(..8).ok_or("the stake program's minimum delegation: short return data")?.try_into().unwrap()))
}

/// The validator's own vote account on a chain of one validator (the rehearsal's): the only one `getVoteAccounts` names.
pub fn own_vote(rpc: &Rpc) -> Res<Pubkey> {
    if rpc.cluster != Cluster::Localnet {
        return Err(format!("{THIS_VALIDATOR}: a rehearsal's word; mainnet's rule names its validators"));
    }
    let v = rpc.call("getVoteAccounts", json!([{ "commitment": "confirmed" }]))?;
    let all: Vec<&str> = ["current", "delinquent"].iter().flat_map(|k| v[*k].as_array().into_iter().flatten().filter_map(|x| x["votePubkey"].as_str())).collect();
    match all.as_slice() {
        [one] => Pubkey::from_str(one).map_err(|_| format!("getVoteAccounts: {one}")),
        _ => Err(format!("{THIS_VALIDATOR}: this chain has {} vote accounts, and the word names one", all.len())),
    }
}

/// Everything the planner reads, off the chain.
pub fn read(rpc: &Rpc, pool: &Pubkey, payer: &Pubkey) -> Res<(Keys, Chain)> {
    let a = rpc.account(&pool.to_string())?.ok_or_else(|| format!("{pool}: no account on this chain"))?;
    let owner = Pubkey::from_str(&a.owner).map_err(|_| "the pool's owner".to_string())?;
    let k = Keys::of_account(pool, &owner, &a.data)?;
    let now = poleis_door::pool_now(&a.data)?;
    let staker = Pubkey::new_from_array(a.data[STAKER..STAKER + 32].try_into().unwrap());
    let l = rpc.account(&k.list.to_string())?.ok_or("her pool's validator list is not on this chain")?;
    let (max_validators, entries) = list_of(&l.data)?;
    let reserve = rpc.account(&k.reserve.to_string())?.ok_or("her pool's reserve is not on this chain")?.lamports;
    let epoch = rpc.call("getEpochInfo", json!([{ "commitment": "confirmed" }]))?["epoch"].as_u64().ok_or("getEpochInfo: no epoch")?;
    let rewards = rpc.account(&poleis_door::EPOCH_REWARDS.to_string())?.map(|a| a.data).unwrap_or_default();
    Ok((
        k,
        Chain {
            epoch,
            pool_epoch: now.last_update_epoch,
            staker,
            total: now.total_lamports,
            max_validators,
            entries,
            reserve,
            stake_rent: rpc.rent(pool::RESERVE_LEN)?,
            min_delegation: stake_minimum(rpc, payer)?.max(MINIMUM_ACTIVE_STAKE),
            rewards_paying: poleis_door::rewards_paying(&rewards),
        },
    ))
}

/// One transaction landed, signed by the engine's key (its fee payer and the pool's staker): its signature and its fee, or the chain's words.
/// THE EPOCH TURNS THE CRANKS: the engine sends at an epoch's boundary, so every send is the one that survives it (epoch.rs `land_surely`: sent
/// again until confirmed, built again past its blockhash's last valid height, a bounded number of times).
pub fn land(rpc: &Rpc, ixs: &[Instruction], key: &Key) -> Res<(String, u64)> {
    crate::epoch::land_surely(rpc, ixs, &[key])
}

/// What one run of the crank did, in numbers.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub epoch: u64,
    pub sent: Vec<Value>,
    pub said: Vec<String>,
    pub fees: u64,
    /// after the run: each validator's active and transient lamports, the reserve's lamports above its rent, the pool's total, its kept share
    pub after: Value,
}

/// PRESS ENGINE: the crank, one run. Reads the pool, then sends what `next` says until nothing is due (at most four rounds: the update, the
/// adds, the moves); with no key (`key` None) it says what it would send and sends nothing.
pub fn run(rpc: &Rpc, pool: &Pubkey, rule: &Rule, engine: &Pubkey, key: Option<&Key>) -> Res<Report> {
    if let Some(k) = key {
        if k.address != *engine {
            return Err(format!("the key given is {}, the engine's {engine}", k.address));
        }
    }
    let mut r = Report::default();
    for _ in 0..4 {
        let (k, c) = read(rpc, pool, engine)?;
        r.epoch = c.epoch;
        let (what, txs): (Vec<Value>, Vec<Vec<Instruction>>) = match next(rule, &c, engine)? {
            Step::Update => (vec![json!({ "update": { "from": c.pool_epoch, "to": c.epoch, "validators": c.entries.len() } })], update(&k, &c.entries)),
            Step::Moves(moves, words) => {
                r.said.extend(words);
                let adds: Vec<Instruction> = moves.iter().filter(|m| matches!(m, Move::Add { .. })).map(|m| instruction(&k, engine, m, &c.entries)).collect::<Res<_>>()?;
                let rest: Vec<Instruction> = moves.iter().filter(|m| !matches!(m, Move::Add { .. })).map(|m| instruction(&k, engine, m, &c.entries)).collect::<Res<_>>()?;
                let what = moves
                    .iter()
                    .map(|m| match m {
                        Move::Add { vote } => json!({ "add": vote.to_string(), "lamports": c.stake_rent + c.min_delegation }),
                        Move::Increase { vote, lamports } => json!({ "increase": vote.to_string(), "lamports": lamports }),
                        Move::Decrease { vote, lamports } => json!({ "decrease": vote.to_string(), "lamports": lamports }),
                        Move::Fund { lamports } => json!({ "fund the reserve": lamports }),
                    })
                    .collect();
                // one transaction a move: each names its own transient and stays far inside a packet
                (what, adds.into_iter().chain(rest).map(|i| vec![i]).collect())
            }
            Step::Nothing(words) => {
                r.said.extend(words);
                break;
            }
            Step::Wait(w) => {
                r.said.push(w);
                break;
            }
        };
        let Some(key) = key else {
            r.said.push(format!("no key given: {} transaction(s) would be sent, and none is", txs.len()));
            r.sent.extend(what.into_iter().map(|w| json!({ "would": w })));
            break;
        };
        let mut sigs = Vec::new();
        for ixs in &txs {
            let (sig, fee) = land(rpc, ixs, key)?;
            r.fees += fee;
            sigs.push(sig);
        }
        r.sent.push(json!({ "did": what, "signatures": sigs }));
    }
    let (_, c) = read(rpc, pool, engine)?;
    r.after = json!({
        "epoch": c.epoch, "poolEpoch": c.pool_epoch, "total": c.total, "reserveFree": c.free(), "kept": rule.kept(c.total),
        "validators": c.entries.iter().map(|e| json!({ "vote": e.vote.to_string(), "active": e.active, "transient": e.transient, "status": e.status })).collect::<Vec<_>>(),
    });
    Ok(r)
}

/// The params' rule as the crank reads it on this chain: the rehearsal's word for its validator resolved where it stands.
pub fn rule_on(p: &Params, rpc: &Rpc) -> Res<Rule> {
    let needs_own = p.at("engine.validators").as_array().is_some_and(|a| a.iter().any(|v| v["vote"].as_str().is_some_and(|s| s.starts_with(THIS_VALIDATOR))));
    let own = if needs_own { Some(own_vote(rpc)?) } else { None };
    Rule::of(p, own.as_ref())
}
