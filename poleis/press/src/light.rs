//! THE LIGHT PAYS A WISH (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider; his "Yes, build them" at 13:25 UTC to "Build the three missing
//! links now?", over the launch's road whose link 6 reads "Each staker's light reaches the wish they chose"). His words it stands on, UTC: Nov 22
//! 16:57:42 "THE META-SWARM IS GLORIOUS BECAUSE YOU STAKE ALL THERE IN 1 PLACE ... BUT!!!! you can delegate the yield of your stake or let the swarm
//! futarchy handle it for you"; Nov 25 13:11 "we gotta launch swarmbook and the stake sol to fund snowtracers".
//!
//! Two halves, the reader pure and the pour a crank:
//!
//! 1. THE READER (`lights_of`, `read`, `agree`). One counted epoch of her pool's book (`embertide-stake`, its own law: the book before the count
//!    and the book after it) gives each staker's light, the lamports of yield they gave up. Each staker's newest choice (the door's memo,
//!    `embertide.light/1 wish <unit>` or `futarchy`, signed by the staker before the epoch whose fee is poured began) says where it goes. A wish's
//!    share is its stakers' light in her pool's token, rounded down: `fee_tokens * light / fee_lamports`. No choice, the futarchy, root 0 itself, or
//!    a unit that is no swarm on the chain sends nothing: that share stays in root 0's vault, and the reader says how much (the rule for undirected
//!    light is not his yet; none is invented here). The shares never sum past what the epoch minted; rounding's dust stays in the vault. When the
//!    book's fee and the chain's mints disagree, nothing is poured and both numbers are said.
//!
//! 2. THE POUR (`plan`). Each wish's share moves from root 0's vault to that wish's own vault of the same mint, by a road Poleis already has: a
//!    standing TREASURY_ALLOWANCE of root 0 whose payee is the wish's unit (HOLDER_UNIT). For a unit payee Poleis checks the destination itself, the
//!    payee's own vault of the allowance's mint and nothing else (`ix::economy::check_dest`: `expect_vault(dest, unit, mint)`), and lets anyone pull
//!    for it; so the allowance is voted once by the whole tree for each wish, and the pour is a crank with no vote per epoch, every pull charged to
//!    root 0's plan of the mint (its CODEX AllowMint) and capped by the allowance per period. Each epoch of each wish leaves one record of the
//!    crank's, `embertide.pour/1 epoch <e> wish <unit> owed <s> tokens <n>`, a memo in the transaction of its pull (or alone, when nothing is
//!    due): that memo is the record (on the chain, in a transaction the crank's key paid and signed), and a second run that reads it pours nothing
//!    for that epoch and wish.
//!
//! What the allowance does not hold (said, his to rule): anyone may pull a unit's allowance into that unit's vault, up to its cap a period, whatever
//! its light was. The token never leaves the tree for a key by that road, but a well can be filled past its light. So the crank reads every pull on
//! each wish's road (Poleis's own RCPT_PULL receipts) against its own records (`Ledger`): a pull it did not send is an OVER-PULL, said by name (the
//! wish, the amount, the puller, the transaction); each wish is poured only what its light is still owed after everything it took, never a
//! negative pour, and a wish over its light is said to be over and gets nothing until its next epochs' light evens it. Root 0's vault never pays
//! twice for the same light.

use crate::door::{self, Keys, Light, MEMO};
use crate::rpc::Rpc;
use crate::{wire, Res};
use embertide_stake::{self as stake, Book};
use poleis_client::{self as client, HOLDER_UNIT, ROOT_UNIT};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

/// The pour's word, at the head of its memo.
pub const POUR: &str = "embertide.pour/1";
/// The token program her pool mints with (Keys::of_account holds it so).
pub const TOKEN: Pubkey = poleis_door::TOKEN;

// ---------------------------------------------------------------- 1. THE READER (pure)

/// One counted epoch of the book: its span, its fee, and each staker's light in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lights {
    /// the epoch of the read that counted it
    pub epoch: u64,
    /// the epoch of the read before; the fee is what was minted in the epochs after it, up to `epoch`
    pub from: u64,
    pub fee_tokens: u64,
    pub fee_lamports: u64,
    /// lamports of the fee no staker's share took (the book's own rounding, or the whole fee when nobody held across)
    pub dust_lamports: u64,
    pub of: BTreeMap<String, u128>,
}

/// THE EPOCH'S LIGHTS: the book after one count less the book before it. Refused unless `after` is `before` with exactly one more epoch counted
/// (another pool's, a book that went back, a count skipped), and unless the lights and the dust sum to the fee, as the book's own law keeps them.
pub fn lights_of(before: Option<&Book>, after: &Book) -> Res<Lights> {
    let row = after.row.as_ref().ok_or("the book after has counted no epoch: nothing to pour")?;
    let counted_before = before.map_or(0, |b| b.counted);
    if after.counted != counted_before + 1 {
        return Err(format!("the book after has counted {} epochs and the book before {counted_before}: one count between them, or nothing is read", after.counted));
    }
    if let Some(b) = before {
        if b.pool != after.pool {
            return Err(format!("the two books are of two pools ({} and {})", b.pool, after.pool));
        }
        match &b.last {
            Some(last) if last.epoch == row.from => {}
            _ => return Err(format!("the count after reads from epoch {}, and the book before was not last read there", row.from)),
        }
    }
    let mut of = BTreeMap::new();
    let mut sum: u128 = 0;
    for (owner, s) in &after.stakers {
        let was = before.and_then(|b| b.stakers.get(owner)).map_or(0, |s| s.light);
        let l = s.light.checked_sub(was).ok_or_else(|| format!("{owner}'s light went down between the two books"))?;
        if l > 0 {
            sum += l;
            of.insert(owner.clone(), l);
        }
    }
    if sum + row.dust as u128 != row.fee_lamports as u128 {
        return Err(format!("the lights ({sum}) and the dust ({}) do not sum to the fee ({} lamports)", row.dust, row.fee_lamports));
    }
    Ok(Lights { epoch: row.epoch, from: row.from, fee_tokens: row.fee_tokens, fee_lamports: row.fee_lamports, dust_lamports: row.dust, of })
}

/// Where one staker's light goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goes {
    Wish(u64),
    /// no choice signed: undirected
    NoChoice,
    /// the futarchy: undirected until a rule for it is his
    Futarchy,
    /// a unit that is no swarm on the chain, or root 0 itself: undirected
    NotASwarm(u64),
}

impl Goes {
    pub fn words(&self) -> String {
        match self {
            Goes::Wish(u) => format!("wish {u}"),
            Goes::NoChoice => "no choice: stays in root 0's vault".into(),
            Goes::Futarchy => "the futarchy: stays in root 0's vault".into(),
            Goes::NotASwarm(u) => format!("unit {u}, no swarm on the chain: stays in root 0's vault"),
        }
    }
}

/// One wish's share of an epoch.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Share {
    pub light_lamports: u128,
    pub tokens: u64,
    pub stakers: Vec<String>,
}

/// THE READING of one epoch: what each wish is owed in her pool's token, what stays undirected, the dust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub epoch: u64,
    pub from: u64,
    pub fee_tokens: u64,
    pub fee_lamports: u64,
    pub wishes: BTreeMap<u64, Share>,
    pub undirected_lamports: u128,
    pub undirected_tokens: u64,
    /// tokens no share took: the book's dust and every share's rounding
    pub dust_tokens: u64,
    pub stakers: Vec<(String, u128, Goes)>,
}

fn tokens_of(light: u128, fee_tokens: u64, fee_lamports: u64) -> u64 {
    if fee_lamports == 0 {
        return 0;
    }
    (light * fee_tokens as u128 / fee_lamports as u128) as u64
}

/// THE READER: each staker's light sent where their choice says, in her pool's token. `choices` holds each staker's newest choice (None: none
/// signed); `swarms` the units that stand as swarms on the chain. Pure.
pub fn read(l: &Lights, choices: &BTreeMap<String, Option<Light>>, swarms: &BTreeSet<u64>) -> Reading {
    let mut wishes: BTreeMap<u64, Share> = BTreeMap::new();
    let (mut undirected, mut stakers) = (0u128, Vec::new());
    for (owner, light) in &l.of {
        let goes = match choices.get(owner).copied().flatten() {
            None => Goes::NoChoice,
            Some(Light::Futarchy) => Goes::Futarchy,
            Some(Light::Wish(u)) if u != ROOT_UNIT && swarms.contains(&u) => Goes::Wish(u),
            Some(Light::Wish(u)) => Goes::NotASwarm(u),
        };
        match goes {
            Goes::Wish(u) => {
                let s = wishes.entry(u).or_default();
                s.light_lamports += light;
                s.stakers.push(owner.clone());
            }
            _ => undirected += light,
        }
        stakers.push((owner.clone(), *light, goes));
    }
    for s in wishes.values_mut() {
        s.tokens = tokens_of(s.light_lamports, l.fee_tokens, l.fee_lamports);
    }
    let undirected_tokens = tokens_of(undirected, l.fee_tokens, l.fee_lamports);
    let shared: u64 = wishes.values().map(|s| s.tokens).sum::<u64>() + undirected_tokens;
    Reading {
        epoch: l.epoch,
        from: l.from,
        fee_tokens: l.fee_tokens,
        fee_lamports: l.fee_lamports,
        wishes,
        undirected_lamports: undirected,
        undirected_tokens,
        dust_tokens: l.fee_tokens - shared,
        stakers,
    }
}

impl Reading {
    /// Every wish's tokens together: what the pour would move out of root 0's vault.
    pub fn to_wishes(&self) -> u64 {
        self.wishes.values().map(|s| s.tokens).sum()
    }
    pub fn json(&self) -> Value {
        json!({
            "epoch": self.epoch, "from": self.from, "fee_tokens": self.fee_tokens.to_string(), "fee_lamports": self.fee_lamports.to_string(),
            "wishes": self.wishes.iter().map(|(u, s)| (u.to_string(), json!({ "light_lamports": s.light_lamports.to_string(), "tokens": s.tokens.to_string(), "stakers": s.stakers }))).collect::<serde_json::Map<_, _>>(),
            "undirected": { "light_lamports": self.undirected_lamports.to_string(), "tokens": self.undirected_tokens.to_string(), "stays": "in root 0's vault: the rule for undirected light is not his yet" },
            "dust_tokens": self.dust_tokens.to_string(),
            "stakers": self.stakers.iter().map(|(o, l, g)| json!({ "staker": o, "light_lamports": l.to_string(), "goes": g.words() })).collect::<Vec<_>>(),
        })
    }
}

/// The book and the chain agree: the fee the book counted for the epoch's span is the fee the pool's program minted into root 0's vault in it.
/// Otherwise nothing is poured, and both numbers are said.
pub fn agree(r: &Reading, chain_fee_tokens: u64) -> Res<()> {
    if r.fee_tokens != chain_fee_tokens {
        return Err(format!(
            "the book counts {} tokens of fee for epochs {} to {}, and the chain's mints into root 0's vault in them are {chain_fee_tokens}: nothing is poured",
            r.fee_tokens,
            r.from + 1,
            r.epoch
        ));
    }
    if r.to_wishes() + r.undirected_tokens + r.dust_tokens != r.fee_tokens {
        return Err("the shares do not sum to the fee: nothing is poured".into());
    }
    Ok(())
}

// ---------------------------------------------------------------- 2. THE POUR (pure)

/// A standing allowance of root 0 to one wish's unit in her pool's token, as the chain holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allowed {
    pub hash: [u8; 32],
    pub cap: u64,
    pub period: u64,
    pub start: u64,
    pub spent: u64,
    pub expires: u64,
}

impl Allowed {
    /// What it lets out now: the cap less what its period spent (the whole cap once the period has turned), nothing past its expiry.
    pub fn room(&self, now: u64) -> u64 {
        if self.expires != 0 && now >= self.expires {
            return 0;
        }
        if self.period > 0 && now >= self.start.saturating_add(self.period) {
            return self.cap;
        }
        self.cap.saturating_sub(self.spent)
    }
}

/// The crank's record of one epoch of one wish: the light it owed the wish that epoch (`owed`) and what the crank pulled for it (`tokens`, which is
/// less when the wish had already taken some of it by a pull the crank did not send, and 0 when it had taken all of it).
pub fn pour_memo(epoch: u64, unit: u64, owed: u64, tokens: u64) -> String {
    format!("{POUR} epoch {epoch} wish {unit} owed {owed} tokens {tokens}")
}

/// A pour's record read back: (epoch, wish, owed, tokens), or None for any other memo.
pub fn pour_of(memo: &str) -> Option<(u64, u64, u64, u64)> {
    let w: Vec<&str> = memo.trim().split(' ').collect();
    match w.as_slice() {
        [POUR, "epoch", e, "wish", u, "owed", o, "tokens", n] => Some((e.parse().ok()?, u.parse().ok()?, o.parse().ok()?, n.parse().ok()?)),
        _ => None,
    }
}

/// One pull on a wish's road as the chain recorded it: Poleis's own receipt (RCPT_PULL, its amount), the pull's signer, its transaction, and the
/// crank's pour record when the crank sent it (None: a pull the crank did not send, an over-pull).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pull {
    pub sig: String,
    pub slot: u64,
    pub puller: Pubkey,
    pub tokens: u64,
    pub pour: Option<u64>,
}

/// A wish's books on its road: every light the crank recorded it owed, every token pulled on the road (the crank's pours and everyone else's), and
/// the pulls the crank did not send.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Ledger {
    pub owed: u128,
    pub taken: u128,
    /// the slot of the crank's last record for this wish (0: none yet)
    pub last_record: u64,
    pub over_pulls: Vec<Pull>,
}

impl Ledger {
    /// From the crank's records of the wish (owed, slot) and every pull on its road.
    pub fn of(records: &[(u64, u64)], pulls: &[Pull]) -> Ledger {
        Ledger {
            owed: records.iter().map(|r| r.0 as u128).sum(),
            taken: pulls.iter().map(|p| p.tokens as u128).sum(),
            last_record: records.iter().map(|r| r.1).max().unwrap_or(0),
            over_pulls: pulls.iter().filter(|p| p.pour.is_none()).cloned().collect(),
        }
    }
    /// What an epoch's share leaves owed after everything the wish already took: (the pour, never negative; the wish's over-take past its light,
    /// carried against its next epochs until even).
    pub fn due(&self, share: u64) -> (u64, u64) {
        let owed = self.owed + share as u128;
        if self.taken >= owed {
            (0, (self.taken - owed) as u64)
        } else {
            ((owed - self.taken) as u64, 0)
        }
    }
    /// The over-pulls since the crank's last record for this wish.
    pub fn new_over_pulls(&self) -> Vec<&Pull> {
        self.over_pulls.iter().filter(|p| p.slot > self.last_record).collect()
    }
}

/// One wish's pour: the epoch's light owed it, what is pulled after the netting, what the wish is over by, and its transaction (its vault made if it
/// does not stand, the pull when there is one, the crank's record), the crank its payer and signer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pour {
    pub unit: u64,
    pub owed: u64,
    pub tokens: u64,
    pub over: u64,
    pub ixs: Vec<Instruction>,
}

/// Why a wish's share is not poured now (it stays in root 0's vault).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Held {
    /// the crank's record for this epoch and wish stands on the chain
    Poured,
    /// root 0 has voted this wish no allowance of her pool's token: its well has no road yet
    NoWell,
    /// the allowance lets out less than what is due this period
    NoRoom { room: u64 },
    /// a share of nothing
    Nothing,
}

/// THE POUR'S PLAN: for each wish of the reading, its transaction, or why it is held. Each wish is poured what its light is still owed after
/// everything it already took on its road (`ledgers`): never a negative pour, and a wish over its light gets a record and no pull until even. Refused
/// whole, nothing poured, when the pours would take more than the epoch's fee or than root 0's vault holds. Pure.
#[allow(clippy::too_many_arguments)]
pub fn plan(
    crank: &Pubkey,
    mint: &Pubkey,
    r: &Reading,
    wells: &BTreeMap<u64, Allowed>,
    vaults: &BTreeSet<u64>,
    done: &BTreeSet<(u64, u64)>,
    ledgers: &BTreeMap<u64, Ledger>,
    vault_balance: u64,
    now: u64,
) -> Res<(Vec<Pour>, Vec<(u64, Held)>)> {
    let (mut pours, mut held) = (Vec::new(), Vec::new());
    let none = Ledger::default();
    for (unit, s) in &r.wishes {
        let (due, over) = ledgers.get(unit).unwrap_or(&none).due(s.tokens);
        let why = if s.tokens == 0 {
            Some(Held::Nothing)
        } else if done.contains(&(r.epoch, *unit)) {
            Some(Held::Poured)
        } else {
            match wells.get(unit) {
                None => Some(Held::NoWell),
                Some(a) if a.room(now) < due => Some(Held::NoRoom { room: a.room(now) }),
                Some(_) => None,
            }
        };
        if let Some(w) = why {
            held.push((*unit, w));
            continue;
        }
        let mut ixs = Vec::new();
        if due > 0 {
            if !vaults.contains(unit) {
                ixs.push(client::create_vault(*crank, *unit, *mint, TOKEN));
            }
            ixs.push(client::pull(*crank, wells[unit].hash, ROOT_UNIT, due, client::vault_pda(*unit, mint), Some((*mint, TOKEN))));
        }
        ixs.push(Instruction::new_with_bytes(MEMO, pour_memo(r.epoch, *unit, s.tokens, due).as_bytes(), vec![AccountMeta::new_readonly(*crank, true)]));
        pours.push(Pour { unit: *unit, owed: s.tokens, tokens: due, over, ixs });
    }
    let total: u64 = pours.iter().map(|p| p.tokens).sum();
    if total > r.fee_tokens {
        return Err(format!("the pours would take {total} tokens, past the epoch's fee of {}: nothing is poured", r.fee_tokens));
    }
    if total > vault_balance {
        return Err(format!("the pours would take {total} tokens and root 0's vault holds {vault_balance}: nothing is poured"));
    }
    Ok((pours, held))
}

pub fn held_words(h: &Held) -> String {
    match h {
        Held::Poured => "poured already: the crank's record for this epoch stands on the chain".into(),
        Held::NoWell => "root 0 has voted this wish no allowance of her pool's token: its share stays in root 0's vault".into(),
        Held::NoRoom { room } => format!("its allowance lets out {room} tokens now, less than is due: nothing poured, the share stays in root 0's vault"),
        Held::Nothing => "a share of nothing".into(),
    }
}

// ---------------------------------------------------------------- 3. THE CHAIN, READ

/// The cluster's epochs, as the book's law names them.
pub fn schedule(rpc: &Rpc) -> Res<stake::Schedule> {
    let s = rpc.call("getEpochSchedule", json!([]))?;
    let n = |k: &str| s[k].as_u64().ok_or_else(|| format!("getEpochSchedule: no {k}"));
    Ok(stake::Schedule { slots_per_epoch: n("slotsPerEpoch")?, first_normal_epoch: n("firstNormalEpoch")?, first_normal_slot: n("firstNormalSlot")? })
}

/// Every signature of an address, newest first, back to the slot `back_to` (the memo field as the chain lists it, the slot, whether it failed).
fn signatures(rpc: &Rpc, address: &str, back_to: u64) -> Res<Vec<(String, u64, bool, Option<String>)>> {
    let (mut out, mut before): (Vec<_>, Option<String>) = (Vec::new(), None);
    loop {
        let mut opts = json!({ "limit": 1000, "commitment": "confirmed" });
        if let Some(b) = &before {
            opts["before"] = json!(b);
        }
        let page = rpc.call("getSignaturesForAddress", json!([address, opts]))?;
        let page = page.as_array().cloned().unwrap_or_default();
        let last = page.len() < 1000;
        for s in &page {
            let slot = s["slot"].as_u64().unwrap_or(0);
            let sig = s["signature"].as_str().unwrap_or_default().to_string();
            before = Some(sig.clone());
            if slot < back_to {
                return Ok(out);
            }
            out.push((sig, slot, !s["err"].is_null(), s["memo"].as_str().map(str::to_string)));
        }
        if last {
            return Ok(out);
        }
    }
}

/// A landed transaction (getTransaction, `base64`): its wire bytes read by the door's law (`poleis_door::landed_of`), its bytes, its slot and its logs;
/// None while the node does not have it, or when it failed.
struct Tx {
    landed: poleis_door::Landed,
    bytes: Vec<u8>,
    logs: Vec<String>,
}

fn landed(rpc: &Rpc, sig: &str) -> Res<Option<Tx>> {
    let t = rpc.call("getTransaction", json!([sig, { "encoding": "base64", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }]))?;
    if t.is_null() || !t["meta"]["err"].is_null() {
        return Ok(None);
    }
    let bytes = wire::unb64(t["transaction"][0].as_str().unwrap_or_default())?;
    let Some(landed) = poleis_door::landed_of(&bytes) else { return Ok(None) };
    let logs = t["meta"]["logMessages"].as_array().cloned().unwrap_or_default().iter().filter_map(|l| l.as_str().map(str::to_string)).collect();
    Ok(Some(Tx { landed, bytes, logs }))
}

/// THE STAKER'S CHOICE: the newest light the staker signed in a slot before `before_slot` (the first slot of the epoch whose fee is poured: the choice
/// that stood when its yield was minted), None when they signed none. One law with the page's reader (`poleis_door::light_signed`): a memo counts only
/// in a transaction the staker signed where the memo program names them; a memo a stranger attached to a transfer to the staker is not their choice.
pub fn choice_of(rpc: &Rpc, staker: &str, before_slot: u64) -> Res<Option<Light>> {
    let who = Pubkey::from_str(staker).map_err(|_| format!("{staker}: not an address"))?;
    for (sig, slot, failed, memo) in signatures(rpc, staker, 0)? {
        if failed || slot >= before_slot || memo.as_deref().and_then(door::light_of_listed).is_none() {
            continue;
        }
        if let Some(choice) = landed(rpc, &sig)?.and_then(|t| poleis_door::light_signed(&t.bytes, &who)) {
            return Ok(Some(choice));
        }
    }
    Ok(None)
}

/// The units among `units` that stand as swarms on the chain: a registry of Poleis at the unit's address that reads as one, and never root 0.
pub fn swarms_of(rpc: &Rpc, units: impl IntoIterator<Item = u64>) -> Res<BTreeSet<u64>> {
    let mut out = BTreeSet::new();
    for u in units {
        if u == ROOT_UNIT {
            continue;
        }
        if let Some(a) = rpc.account(&client::registry_pda(u).to_string())? {
            if a.owner == client::PROGRAM_ID.to_string() && client::RegistryView::parse(&a.data).is_some_and(|r| r.unit == u) {
                out.insert(u);
            }
        }
    }
    Ok(out)
}

/// THE CHAIN'S FEE: the pool tokens her pool's program minted into root 0's vault inside its own UpdateStakePoolBalance in the epochs `first` to
/// `last`, by the book's own law of the fee's mints (`embertide_stake::fee_mints`).
pub fn chain_fee(rpc: &Rpc, k: &Keys, sched: &stake::Schedule, first: u64, last: u64) -> Res<u64> {
    let (from, _) = sched.slots_of(first).map_err(|e| e.to_string())?;
    let (_, to) = sched.slots_of(last).map_err(|e| e.to_string())?;
    let mut tokens = 0u64;
    for (sig, slot, failed, _) in signatures(rpc, &k.vault.to_string(), from)? {
        if failed || slot > to {
            continue;
        }
        let tx = rpc.call("getTransaction", json!([sig, { "encoding": "jsonParsed", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }]))?;
        for m in stake::fee_mints(&tx, &k.program.to_string(), &k.pool.to_string(), &k.vault.to_string()) {
            tokens = tokens.saturating_add(m.tokens);
        }
    }
    Ok(tokens)
}

/// THE WELLS: every standing allowance of root 0 whose payee is a unit and whose mint is her pool's, by the unit it pays (the first by its hash
/// when a unit has several).
pub fn wells(rpc: &Rpc, mint: &Pubkey) -> Res<BTreeMap<u64, Allowed>> {
    use poleis_treasury::state::Allowance;
    let filters = json!([
        { "dataSize": Allowance::SIZE },
        { "memcmp": { "offset": 0, "bytes": wire::base58(&[client::DISC_ALLOWANCE]) } },
        { "memcmp": { "offset": Allowance::OFF_UNIT, "bytes": wire::base58(&ROOT_UNIT.to_le_bytes()) } },
        { "memcmp": { "offset": Allowance::OFF_MINT, "bytes": mint.to_string() } },
    ]); // four filters at most (the RPC's limit): the payee's kind is held after the read
    let v = rpc.call("getProgramAccounts", json!([client::PROGRAM_ID.to_string(), { "encoding": "base64", "commitment": "confirmed", "filters": filters }]))?;
    let mut out = BTreeMap::new();
    let mut all: Vec<(u64, Allowed)> = Vec::new();
    for a in v.as_array().cloned().unwrap_or_default() {
        let d = wire::unb64(a["account"]["data"][0].as_str().unwrap_or_default())?;
        let Some(x) = client::AllowanceView::parse(&d) else { continue };
        if x.unit != ROOT_UNIT || x.payee_kind != HOLDER_UNIT || x.mint != *mint {
            continue;
        }
        let unit = u64::from_le_bytes(x.payee[..8].try_into().unwrap());
        all.push((unit, Allowed { hash: x.hash, cap: x.cap, period: x.period, start: x.start, spent: x.spent, expires: x.expires }));
    }
    all.sort_by_key(|(u, a)| (*u, a.hash));
    for (u, a) in all {
        out.entry(u).or_insert(a);
    }
    Ok(out)
}

/// THE CRANK'S RECORDS ON THE CHAIN: each (epoch, wish) a pour record of the crank's names, (owed, tokens, slot), in a transaction that landed, paid
/// and signed by the crank, the memo the crank's own.
pub fn poured(rpc: &Rpc, crank: &Pubkey) -> Res<BTreeMap<(u64, u64), (u64, u64, u64)>> {
    let me = crank.to_string();
    let mut out = BTreeMap::new();
    for (sig, slot, failed, memo) in signatures(rpc, &me, 0)? {
        if failed || !memo.as_deref().is_some_and(|m| m.contains(POUR)) {
            continue;
        }
        let Some(t) = landed(rpc, &sig)? else { continue };
        if t.landed.keys.first() != Some(crank) {
            continue;
        }
        for (e, u, owed, tokens) in t.landed.memos_signed_by(crank).iter().filter_map(|m| pour_of(m)) {
            out.insert((e, u), (owed, tokens, slot));
        }
    }
    Ok(out)
}

/// EVERY PULL ON A WISH'S ROAD: each transaction that touched the road's allowance and landed, its pulls read from Poleis's own receipts (RCPT_PULL on
/// this allowance, their amounts) and its pull instruction (who signed it); a pull is the crank's pour when the crank paid and signed the transaction
/// and its own pour record of the wish rides in it, and an over-pull otherwise.
pub fn pulls_on(rpc: &Rpc, road: &[u8; 32], unit: u64, crank: &Pubkey) -> Res<Vec<Pull>> {
    use base64::Engine as _;
    let mut out = Vec::new();
    for (sig, slot, failed, _) in signatures(rpc, &client::allowance_pda(road).to_string(), 0)? {
        if failed {
            continue;
        }
        let Some(t) = landed(rpc, &sig)? else { continue };
        let tokens: u64 = t
            .logs
            .iter()
            .filter_map(|l| l.strip_prefix("Program data: "))
            .filter_map(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
            .filter_map(|b| client::Receipt::parse(&b))
            .filter(|r| r.kind == client::RCPT_PULL && r.hash == *road)
            .map(|r| r.amount)
            .sum();
        if tokens == 0 {
            continue;
        }
        let puller = t
            .landed
            .instructions
            .iter()
            .find(|(p, _, d)| *p == Some(client::PROGRAM_ID) && d.first() == Some(&client::tag::IX_PULL) && d.get(1..33) == Some(&road[..]))
            .and_then(|(_, a, _)| a.first().and_then(|i| t.landed.keys.get(*i)).copied())
            .unwrap_or_default();
        let pour = (t.landed.keys.first() == Some(crank))
            .then(|| t.landed.memos_signed_by(crank).iter().filter_map(|m| pour_of(m)).find(|r| r.1 == unit).map(|r| r.0))
            .flatten();
        out.push(Pull { sig, slot, puller, tokens, pour });
    }
    Ok(out)
}

/// An SPL token account's amount on the chain, 0 when none stands.
pub fn balance(rpc: &Rpc, account: &Pubkey) -> Res<u64> {
    rpc.account(&account.to_string())?.map_or(Ok(0), |a| door::amount_of(&a.data))
}

// ---------------------------------------------------------------- 4. THE CRANK

/// Everything the crank reads before it pours, and what it would send.
pub struct Crank {
    pub keys: Keys,
    pub reading: Reading,
    pub chain_fee: u64,
    pub agreed: Res<()>,
    pub pours: Vec<Pour>,
    pub held: Vec<(u64, Held)>,
    pub vault_before: u64,
    pub poured_before: BTreeMap<(u64, u64), (u64, u64, u64)>,
    /// each wish's books on its road: what it was owed, what it took, the pulls the crank did not send
    pub ledgers: BTreeMap<u64, Ledger>,
}

impl Crank {
    /// The tokens pulled on the wishes' roads by pulls the crank did not send, since each road's last record of the crank's.
    pub fn over_pulled(&self) -> u64 {
        self.ledgers.values().flat_map(|l| l.new_over_pulls()).map(|p| p.tokens).sum()
    }
}

/// THE READ AND THE PLAN, nothing sent: her pool read from its account (hers: the stake pool program's, its fee the whole yield), the two books,
/// each staker's choice, the swarms, the chain's fee for the span, the wells, the vaults, the crank's records and every pull on each wish's road.
pub fn crank(rpc: &Rpc, pool: &Pubkey, before: Option<&Book>, after: &Book, crank: &Pubkey) -> Res<Crank> {
    let a = rpc.account(&pool.to_string())?.ok_or_else(|| format!("no account at {pool}"))?;
    let keys = Keys::of_account(pool, &Pubkey::from_str(&a.owner).map_err(|_| "the pool's owner is not an address")?, &a.data)?;
    if after.pool != pool.to_string() {
        return Err(format!("the book after is of {}, not of {pool}", after.pool));
    }
    let lights = lights_of(before, after)?;
    let sched = schedule(rpc)?;
    let (epoch_first, _) = sched.slots_of(lights.epoch).map_err(|e| e.to_string())?;
    let mut choices = BTreeMap::new();
    for owner in lights.of.keys() {
        choices.insert(owner.clone(), choice_of(rpc, owner, epoch_first)?);
    }
    let named: BTreeSet<u64> = choices.values().filter_map(|c| match c {
        Some(Light::Wish(u)) => Some(*u),
        _ => None,
    }).collect();
    let swarms = swarms_of(rpc, named)?;
    let reading = read(&lights, &choices, &swarms);
    let chain_fee = chain_fee(rpc, &keys, &sched, reading.from + 1, reading.epoch)?;
    let agreed = agree(&reading, chain_fee);
    let wells = wells(rpc, &keys.mint)?;
    let mut vaults = BTreeSet::new();
    for u in reading.wishes.keys() {
        if rpc.account(&client::vault_pda(*u, &keys.mint).to_string())?.is_some() {
            vaults.insert(*u);
        }
    }
    let poured_before = poured(rpc, crank)?;
    let done: BTreeSet<(u64, u64)> = poured_before.keys().copied().collect();
    let mut ledgers = BTreeMap::new();
    for (u, road) in &wells {
        let records: Vec<(u64, u64)> = poured_before.iter().filter(|((_, w), _)| w == u).map(|(_, (owed, _, slot))| (*owed, *slot)).collect();
        ledgers.insert(*u, Ledger::of(&records, &pulls_on(rpc, &road.hash, *u, crank)?));
    }
    let vault_before = balance(rpc, &keys.vault)?;
    let (pours, held) = match &agreed {
        Ok(()) => plan(crank, &keys.mint, &reading, &wells, &vaults, &done, &ledgers, vault_before, rpc.slot()?)?,
        Err(_) => (Vec::new(), reading.wishes.keys().map(|u| (*u, Held::Nothing)).collect()),
    };
    Ok(Crank { keys, reading, chain_fee, agreed, pours, held, vault_before, poured_before, ledgers })
}

impl Crank {
    pub fn json(&self) -> Value {
        let pull = |p: &Pull| json!({ "tokens": p.tokens.to_string(), "puller": p.puller.to_string(), "transaction": p.sig, "slot": p.slot });
        json!({
            "pool": self.keys.pool.to_string(), "mint": self.keys.mint.to_string(), "root0_vault": self.keys.vault.to_string(),
            "reading": self.reading.json(), "chain_fee_tokens": self.chain_fee.to_string(),
            "agree": match &self.agreed { Ok(()) => json!(true), Err(e) => json!(e) },
            "root0_vault_tokens": self.vault_before.to_string(),
            "over_pulled_tokens": self.over_pulled().to_string(),
            "over_pulls": self.ledgers.iter().flat_map(|(u, l)| l.new_over_pulls().into_iter().map(move |p| (u, p))).map(|(u, p)| {
                let mut v = pull(p);
                v["wish"] = json!(u.to_string());
                v["is"] = json!("an over-pull: tokens that reached the wish's well by a pull the crank did not send, netted against its light");
                v
            }).collect::<Vec<_>>(),
            "roads": self.ledgers.iter().map(|(u, l)| (u.to_string(), json!({ "owed_before": l.owed.to_string(), "taken": l.taken.to_string(), "over_pulls_ever": l.over_pulls.iter().map(pull).collect::<Vec<_>>() }))).collect::<serde_json::Map<_, _>>(),
            "pours": self.pours.iter().map(|p| json!({ "wish": p.unit.to_string(), "owed": p.owed.to_string(), "tokens": p.tokens.to_string(), "over": p.over.to_string(), "into": client::vault_pda(p.unit, &self.keys.mint).to_string(), "memo": pour_memo(self.reading.epoch, p.unit, p.owed, p.tokens) })).collect::<Vec<_>>(),
            "held": self.held.iter().map(|(u, h)| json!({ "wish": u.to_string(), "why": held_words(h) })).collect::<Vec<_>>(),
            "poured_on_the_chain": self.poured_before.iter().map(|((e, u), (o, n, _))| json!({ "epoch": e, "wish": u.to_string(), "owed": o.to_string(), "tokens": n.to_string() })).collect::<Vec<_>>(),
        })
    }
}

/// Each pour landed: its transaction signed by the crank alone, sent with its preflight, confirmed. Their signatures.
pub fn send(rpc: &Rpc, c: &Crank, key: &wire::Key) -> Res<Vec<String>> {
    c.agreed.clone()?;
    let mut sigs = Vec::new();
    for p in &c.pours {
        // THE EPOCH TURNS THE CRANKS: sent again until confirmed, built again past its blockhash's last valid height (epoch.rs land_surely)
        let (sig, _) = crate::epoch::land_surely(rpc, &p.ixs, &[key]).map_err(|e| format!("the pour into wish {}: {e}", p.unit))?;
        sigs.push(sig);
    }
    Ok(sigs)
}

/// `press light --rpc URL [--cluster localnet|mainnet] --pool P --after G [--before F] [--crank K.json] [--send]`: the epoch's reading and its
/// pour's plan, said; with --send, the crank's key file signs and sends each pour. The books are `embertide-stake count`'s files.
pub fn step(opts: &BTreeMap<String, String>, send_it: bool) -> Res<Value> {
    let get = |k: &str| opts.get(k).map(String::as_str).ok_or_else(|| format!("{k} is needed"));
    let cluster = crate::rpc::Cluster::parse(opts.get("--cluster").map_or("localnet", String::as_str))?;
    let rpc = Rpc::new(get("--rpc")?, cluster)?;
    rpc.check_cluster()?;
    let pool = Pubkey::from_str(get("--pool")?).map_err(|_| "--pool: not an address")?;
    let book = |f: &str| -> Res<Book> {
        let t = std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?;
        stake::book_of(&serde_json::from_str(&t).map_err(|e| format!("{f}: not JSON ({e})"))?, &pool.to_string()).map_err(|e| format!("{f}: {e}"))
    };
    let before = opts.get("--before").map(|f| book(f)).transpose()?;
    let after = book(get("--after")?)?;
    let key = opts.get("--crank").map(|f| std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}")).and_then(|t| wire::Key::from_cli_text(&t))).transpose()?;
    if send_it && key.is_none() {
        return Err("--send: the crank's key file (--crank) signs each pour".into());
    }
    // without a key the plan is said for an address that holds none: the pours' records are read for no one
    let crank_at = key.as_ref().map_or(Pubkey::default(), |k| k.address);
    let c = crank(&rpc, &pool, before.as_ref(), &after, &crank_at)?;
    let mut out = c.json();
    if send_it {
        out["sent"] = json!(send(&rpc, &c, key.as_ref().unwrap())?);
    }
    Ok(out)
}

// ---------------------------------------------------------------- 5. THE LIGHT'S PLAN, A LATER ACT

/// The tenth act's set, as `press acts` writes each of the eight (and `engine::write_set` the ninth): the act, its payload and hash, unit 1's
/// agents' envelopes, and the plan it makes. Written into `dir` beside the others; refused where the params name no cap.
pub fn write_set(dir: &std::path::Path, p: &crate::params::Params, ids: &crate::build::Ids, at: &crate::chain::Addresses, genesis: &str, deployer: &Pubkey) -> Res<std::path::PathBuf> {
    let a = later(p, ids, deployer)?.into_iter().next().ok_or("params light.planCap: root 0's plan of her pool's token names no figure, so no tenth act is written")?;
    let cap = plan_cap(p)?.unwrap_or_default();
    let pool = crate::pool::Pool::of(p, deployer, &ids.poleis)?;
    let agents = p.agents()?;
    let labels = crate::acts::labels(ids, at, &agents);
    let envelopes: Vec<Value> = agents.iter().enumerate().map(|(i, v)| crate::acts::envelope(1 + i, v, &a.hash, &labels)).collect();
    let placeholders = p.placeholders();
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
        "plan": { "unit": ROOT_UNIT, "mint": pool.mint.to_string(), "cap": cap, "params": p.at("light.planCap"), "tokenPlan": client::token_plan_pda(ROOT_UNIT, &pool.mint).to_string(), "vault": pool.vault.to_string() },
    });
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(a.file);
    std::fs::write(&path, serde_json::to_string_pretty(&set).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// THE LIGHT'S PLAN, the tenth act (a later act, voted after the launch's eight and the engine's ninth, on the road `engine::later` lays): root 0's
/// CODEX AllowMint of her pool's token, `light.planCap` of it a plan period. Poleis charges every pull of a mint that is not root 0's default (the
/// dollar) to that plan, so until it executes no pour leaves root 0's vault.
pub const PLAN_ACT: usize = 10;
pub const PLAN_ACT_NAME: &str = "root 0's plan of her pool's token (CODEX AllowMint)";
pub const PLAN_ACT_FILE: &str = "10-the-lights-plan.json";

/// The params' `light.planCap`: a figure, or none yet (absent, a blank of his, or the rehearsal's word, which a sitting on mainnet refuses).
pub fn plan_cap(p: &crate::params::Params) -> Res<Option<u64>> {
    let v = p.at("light.planCap");
    if v.is_null() || !p.later_placeholders("light").is_empty() {
        return Ok(None);
    }
    if v.as_str().is_some_and(|s| s.starts_with(crate::engine::THIS_REHEARSAL)) {
        if p.cluster != crate::rpc::Cluster::Localnet {
            return Err(format!("params light.planCap: {} is a rehearsal's word; mainnet's is his figure", crate::engine::THIS_REHEARSAL));
        }
        return Ok(None);
    }
    p.u64("light.planCap").map(Some)
}

/// The tenth act where the params name its cap: written, proposed, voted and executed as the launch's acts are (its nonce `acts.nonce + 9`), its
/// execute's crank the deployer, who pays the plan's account.
pub fn later(p: &crate::params::Params, ids: &crate::build::Ids, deployer: &Pubkey) -> Res<Vec<crate::acts::Act>> {
    let Some(cap) = plan_cap(p)? else { return Ok(Vec::new()) };
    let pool = crate::pool::Pool::of(p, deployer, &ids.poleis)?;
    let act = client::Payload::AllowMint { origin: ROOT_UNIT, mint: pool.mint, cap, payer: *deployer };
    let manifest = wire::sha256(format!("THE LAUNCH PRESS: {PLAN_ACT_NAME}").as_bytes());
    let nonce = p.u64_or("acts.nonce", 1)? + PLAN_ACT as u64 - 1;
    let h = client::proposal_hash(act.class(), ROOT_UNIT, &manifest, &act.hash(), nonce);
    let impacted = crate::acts::impacted_of(act.class());
    Ok(vec![crate::acts::Act {
        n: PLAN_ACT,
        name: PLAN_ACT_NAME,
        file: PLAN_ACT_FILE,
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
                "let root 0 spend up to {cap} of her pool's token {} a plan period (the params' light.planCap): the plan every pour into a wish's well is charged to; until it executes no pour leaves root 0's vault {}",
                pool.mint, pool.vault
            ),
            format!("its execute makes the plan's account at {}, the deployer {deployer} its payer", client::token_plan_pda(ROOT_UNIT, &pool.mint)),
        ],
    }])
}
