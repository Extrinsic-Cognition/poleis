//! THE MINE IS LIVE (Nov 23 2026, lane/the-mine-is-live). His words, UTC: Nov 21 2026 17:34:37 "it is supposed to be uncapped supply, prompt-2-mine;
//! the issuance needs to be live and continuous based on the space-router utilization"; Oct 28 13:50 "we should give $WARM only when money is spent in
//! the SPACE-ROUTER". The $WARM program (`poleis/program/warm_mine`) stood with nothing reaching it: the door sent a deposit straight to the treasury,
//! and nothing posted a spend. The two halves, both the door's, written here once and compiled into the door's own WASM (worker/poleis-seat.wasm):
//!
//! - THE DEPOSIT IS COUNTED (`deposit_message`): a flask's deposit is the mine's own `deposit`, so its dollars go to the mine's treasury (root 0's own
//!   vault of the dollar, as the launch press derives it) and stand in the flask's record, where `mine` reads them. One legacy message by Solana's own
//!   compiler: the compute budget (when a price is named), the terms' memo the flask signs, the mine's `deposit`; the house's fee key its payer when one
//!   is named (it pays the fee and the record's rent, once a flask), the flask signing second as the owner of its USDC, as today's relayed deposit has it.
//! - THE CRANK (`plan`): the router's signed receipts, in the order the door wrote them, become batches of the mine's `mine`. THE CURSOR IS THE CHAIN'S:
//!   a spend's `seq` is its receipt's id, so the mine's own `last_seq` says where the crank stopped; nothing is kept anywhere else, a crank run twice
//!   finds the same cursor and the chain refuses a seq it holds (two cranks at once land one batch). Each receipt is held to the oracle's key before it
//!   counts (its signature over its own canonical bytes, its tank, its instant, its dollars no less than the fuel the ledger says it burned); one that
//!   does not hold mines nothing. A batch gathers a flask's spends of one era into one entry (their nano-dollars summed, then floored to the chain's
//!   micro-dollar: under a micro-dollar of an entry is not mined), stamps it with its last receipt's instant (never earlier than the spend before it:
//!   the chain's rule) and never takes a flask past what it deposited (THE BOUND, read from the flask's record on the chain: the part of a spend past it
//!   mines nothing, as the program would refuse it). A receipt whose instant the chain's clock has not passed waits, and everything after it.
//!   WHAT CANNOT MINE NEVER HOLDS THE LINE: the plan says how far its receipts, from the first, mined nothing at all (`passed`: past their flasks'
//!   deposits, or not the oracle's), and the door notes it and reads from there the next time; the note is a place to read from, never what was mined.
//! - SPENT $WARM GOES BACK TO THE MINE (`pay_back_message`, `paid_back_of`; his press of Nov 23 2026 18:17:41 UTC, "Back to the mine": "One $WARM, the
//!   token: spending it as credits returns it, to be mined again"). Where the mine is named a tank's $WARM credit is what its flask paid back on the
//!   chain, never what the ledger says it mined. A call cannot sign for a flask, so the flask pays back ahead, in its page: one legacy message, the
//!   mine's own `pay_back` of its $WARM from its own $WARM account into the vault, the house's fee key its payer, the flask signing second as the owner
//!   of its $WARM; the door counts it off the flask's record at finalized (`paid_back`, what the mine counted moving into its vault: a plain transfer
//!   into the vault is never re-emitted, so it is no pay-back), and the vault pays the next spends before anything is minted.
//! - SPENT AS CREDITS, IT RETURNS (`allow_message`, `takes`, `take_back_message`): a flask keeps its $WARM. Its page signs one approval, the mine the
//!   delegate of its own $WARM account; a tank's $WARM credit is what its flasks paid back and what they hold for the mine; and after a drink of
//!   that credit the crank brings back just what the tank drank past what its flasks' records say they paid back, by the mine's `take_back`
//!   (a target a record, never an amount: a batch sent twice moves nothing twice).
//! - THE ROUTER DRAWS ITS MONTH (`month_of`, `draw_ixs`, `draw_of`; his Nov 24 2026 00:07:00 UTC "Yes, build it"): where the door names root 0's month
//!   (SPACE_ROUTER_MONTH), the batch that posts spends pulls from root 0's vault, in the same transaction, what the AI provider charged for them.
//!
//! Pure: no clock, no I/O. `ask` is the one door (JSON in, JSON out) the Worker's glue calls through `mine_ask` (doors.rs).
use ed25519_dalek::{Signature, VerifyingKey}; // classical: the router's oracle signs its receipts with Ed25519 (app/space-router-oracle.mjs ALG)
use serde_json::{json, Value};
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::str::FromStr as _;
use warm_mine::state::{Flask, Mine, DISC_FLASK, DISC_MINE, SEED_FLASK, SEED_MINE, SEED_MINT, SEED_VAULT};

/// The $WARM program this build speaks: the tree's for the tests, a sitting's by `press build`.
pub const WARM: Pubkey = Pubkey::new_from_array(warm_mine::ID.to_bytes());
pub const TOKEN: Pubkey = Pubkey::new_from_array(warm_mine::token::TOKEN_PROGRAM_ID.to_bytes());
pub const ATA: Pubkey = Pubkey::new_from_array(warm_mine::token::ATA_PROGRAM_ID.to_bytes());
pub const MEMO: Pubkey = solana_pubkey::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
pub const CLOCK: Pubkey = solana_pubkey::pubkey!("SysvarC1ock11111111111111111111111111111111");
/// The terms a deposit's memo carries (app/space-router-deposit.mjs DEPOSIT_TERMS).
pub const DEPOSIT_TERMS: &str = "space-router:deposit:v1";
/// The units a deposit through the mine asks: the memo (23,218 measured on mainnet, 46,218 when it names a key), the mine's `deposit` (8,128 with the
/// flask's record made), the compute budget's own two instructions.
pub const DEPOSIT_UNITS: u32 = 45_000;
pub const DEPOSIT_UNITS_KEYED: u32 = 75_000;
/// The units a pay-back asks: the mine's `pay_back` (the token's transfer_checked and the two counts) and the compute budget's own two instructions,
/// 2,812 measured on a local validator (poleis/press/tests/mine.rs says them), the flask's record standing; five times that covers a token program
/// whose transfer costs more and a record made when none stands, and the priority price is paid on no more.
pub const PAY_BACK_UNITS: u32 = 15_000;
/// The units an approval asks: the token program's ApproveChecked and the compute budget's own two, the same bound as a pay-back's.
pub const ALLOW_UNITS: u32 = 15_000;
/// Nano-dollars (the router's grain) in a micro-dollar (the chain's).
pub const NANOS: u64 = 1_000;

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &WARM).0
}
/// The mine, its $WARM mint and its vault: the program's own addresses.
pub fn mine_at() -> Pubkey {
    pda(&[SEED_MINE])
}
pub fn mint_at() -> Pubkey {
    pda(&[SEED_MINT])
}
pub fn vault_at() -> Pubkey {
    pda(&[SEED_VAULT])
}
/// A flask's record, and its own $WARM account (its associated token account of the mine's mint).
pub fn record_of(flask: &Pubkey) -> Pubkey {
    pda(&[SEED_FLASK, flask.as_ref()])
}
pub fn warm_account_of(flask: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[flask.as_ref(), TOKEN.as_ref(), mint_at().as_ref()], &ATA).0
}

/// A flask's deposit through the mine, as the door is asked for it.
#[derive(Clone, Debug)]
pub struct Deposit {
    /// The house's fee key, when it pays: the fee and the record's rent. None: the flask pays its own.
    pub fee: Option<Pubkey>,
    pub flask: Pubkey,
    /// The flask's own account of the dollar.
    pub source: Pubkey,
    /// The mine's treasury and its dollar, as the mine names them.
    pub treasury: Pubkey,
    pub dollar: Pubkey,
    /// The dollar's base units.
    pub amount: u64,
    pub blockhash: [u8; 32],
    /// The priority price, micro-lamports a unit; None asks no compute budget.
    pub price: Option<u64>,
    /// The key commitment the memo names (64 lowercase hex), or none.
    pub commitment: Option<String>,
}

/// The mine's `deposit` (ix.rs): [flask (s), payer (s,w), mine (w), record (w), source (w), treasury (w), dollar, token program, system].
pub fn deposit_ix(d: &Deposit) -> Instruction {
    let payer = d.fee.unwrap_or(d.flask);
    Instruction {
        program_id: WARM,
        accounts: vec![
            AccountMeta::new_readonly(d.flask, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(mine_at(), false),
            AccountMeta::new(record_of(&d.flask), false),
            AccountMeta::new(d.source, false),
            AccountMeta::new(d.treasury, false),
            AccountMeta::new_readonly(d.dollar, false),
            AccountMeta::new_readonly(TOKEN, false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ],
        data: [&[warm_mine::IX_DEPOSIT][..], &d.amount.to_le_bytes()].concat(),
    }
}

/// THE DEPOSIT IS COUNTED: the one message a flask signs to deposit through the mine. Its signers: the fee key first when one pays, the flask.
pub fn deposit_message(d: &Deposit) -> Result<Vec<u8>, &'static str> {
    if d.amount < warm_mine::MIN_DEPOSIT_MICROS {
        return Err("a flask is made from 10 USDC");
    }
    let keys = [Some(d.flask), d.fee, Some(d.source), Some(d.treasury), Some(d.dollar)];
    let keys: Vec<Pubkey> = keys.into_iter().flatten().collect();
    if keys.iter().enumerate().any(|(i, k)| keys[..i].contains(k) || [WARM, TOKEN, MEMO, crate::COMPUTE_BUDGET, Pubkey::default(), mine_at()].contains(k)) {
        return Err("the flask, its account, the treasury, the dollar and the fee key are different addresses, and none a program's");
    }
    let memo = match &d.commitment {
        None => DEPOSIT_TERMS.to_string(),
        Some(c) if c.len() == 64 && c.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) => format!("{DEPOSIT_TERMS}:{c}"),
        Some(_) => return Err("the key commitment is 64 lowercase hex"),
    };
    let mut ixs = Vec::new();
    if let Some(price) = d.price {
        let units = if d.commitment.is_some() { DEPOSIT_UNITS_KEYED } else { DEPOSIT_UNITS };
        ixs.push(Instruction { program_id: crate::COMPUTE_BUDGET, accounts: vec![], data: [&[2u8][..], &units.to_le_bytes()].concat() });
        ixs.push(Instruction { program_id: crate::COMPUTE_BUDGET, accounts: vec![], data: [&[3u8][..], &price.to_le_bytes()].concat() });
    }
    ixs.push(Instruction { program_id: MEMO, accounts: vec![AccountMeta::new_readonly(d.flask, true)], data: memo.into_bytes() });
    ixs.push(deposit_ix(d));
    let payer = d.fee.unwrap_or(d.flask);
    let m = Message::new_with_blockhash(&ixs, Some(&payer), &Hash::new_from_array(d.blockhash));
    let signers = if d.fee.is_some() { vec![payer, d.flask] } else { vec![payer] };
    if m.header.num_required_signatures as usize != signers.len() || m.account_keys[..signers.len()] != signers[..] {
        return Err("the message's signers are not the fee key and the flask");
    }
    let message = m.serialize();
    if 1 + 64 * signers.len() + message.len() > crate::PACKET {
        return Err("past Solana's packet");
    }
    Ok(message)
}

/// A flask's pay-back through the mine, as the door is asked for it: `amount` nanos of its $WARM, from its own $WARM account into the vault.
#[derive(Clone, Debug)]
pub struct PayBack {
    /// The house's fee key, when it pays the fee (and the record's rent, when none stands). None: the flask pays its own.
    pub fee: Option<Pubkey>,
    pub flask: Pubkey,
    pub amount: u64,
    pub blockhash: [u8; 32],
    /// The priority price, micro-lamports a unit; None asks no compute budget.
    pub price: Option<u64>,
}

/// The mine's `pay_back` (ix.rs): [flask (s), payer (s,w), mine (w), record (w), source (w), vault (w), $WARM mint, token program, system]; the
/// source is the flask's own $WARM account, the one `mine` pays.
pub fn pay_back_ix(b: &PayBack) -> Instruction {
    let payer = b.fee.unwrap_or(b.flask);
    Instruction {
        program_id: WARM,
        accounts: vec![
            AccountMeta::new_readonly(b.flask, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(mine_at(), false),
            AccountMeta::new(record_of(&b.flask), false),
            AccountMeta::new(warm_account_of(&b.flask), false),
            AccountMeta::new(vault_at(), false),
            AccountMeta::new_readonly(mint_at(), false),
            AccountMeta::new_readonly(TOKEN, false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ],
        data: [&[warm_mine::IX_PAY_BACK][..], &b.amount.to_le_bytes()].concat(),
    }
}

/// One message a flask signs, the house's fee key its payer when one is named, the compute budget asked when a price is: its signers the fee key first
/// and the flask, and never past Solana's packet.
fn by_flask(fee: Option<Pubkey>, flask: Pubkey, ix: Instruction, units: u32, price: Option<u64>, blockhash: &[u8; 32]) -> Result<Vec<u8>, &'static str> {
    if fee == Some(flask) || [WARM, TOKEN, MEMO, crate::COMPUTE_BUDGET, Pubkey::default(), mine_at(), mint_at(), vault_at()].iter().any(|k| *k == flask || Some(*k) == fee) {
        return Err("the flask and the fee key are different addresses, and neither a program's nor the mine's");
    }
    let mut ixs = Vec::new();
    if let Some(price) = price {
        ixs.push(Instruction { program_id: crate::COMPUTE_BUDGET, accounts: vec![], data: [&[2u8][..], &units.to_le_bytes()].concat() });
        ixs.push(Instruction { program_id: crate::COMPUTE_BUDGET, accounts: vec![], data: [&[3u8][..], &price.to_le_bytes()].concat() });
    }
    ixs.push(ix);
    let payer = fee.unwrap_or(flask);
    let m = Message::new_with_blockhash(&ixs, Some(&payer), &Hash::new_from_array(*blockhash));
    let signers = if fee.is_some() { vec![payer, flask] } else { vec![payer] };
    if m.header.num_required_signatures as usize != signers.len() || m.account_keys[..signers.len()] != signers[..] {
        return Err("the message's signers are not the fee key and the flask");
    }
    let message = m.serialize();
    if 1 + 64 * signers.len() + message.len() > crate::PACKET {
        return Err("past Solana's packet");
    }
    Ok(message)
}

/// SPENT $WARM GOES BACK TO THE MINE: the one message a flask signs to pay its $WARM back. Its signers: the fee key first when one pays, the flask.
pub fn pay_back_message(b: &PayBack) -> Result<Vec<u8>, &'static str> {
    if b.amount == 0 {
        return Err("nothing to pay back");
    }
    by_flask(b.fee, b.flask, pay_back_ix(b), PAY_BACK_UNITS, b.price, &b.blockhash)
}

/// A flask's one approval (SPENT AS CREDITS, IT RETURNS), as the door is asked for it.
#[derive(Clone, Debug)]
pub struct Allow {
    pub fee: Option<Pubkey>,
    pub flask: Pubkey,
    pub blockhash: [u8; 32],
    pub price: Option<u64>,
}
/// The token program's ApproveChecked (warm_mine::token, held to the token program's own bytes): [the flask's $WARM account (w), the mint, the
/// delegate (the mine), the flask (s)], for all of it, ever.
pub fn allow_ix(a: &Allow) -> Instruction {
    Instruction {
        program_id: TOKEN,
        accounts: vec![AccountMeta::new(warm_account_of(&a.flask), false), AccountMeta::new_readonly(mint_at(), false), AccountMeta::new_readonly(mine_at(), false), AccountMeta::new_readonly(a.flask, true)],
        data: warm_mine::token::approve_checked_data(u64::MAX, warm_mine::token::WARM_DECIMALS).to_vec(),
    }
}
/// The one message a flask signs to make the mine the delegate of its $WARM: it moves nothing; its $WARM stays the flask's.
pub fn allow_message(a: &Allow) -> Result<Vec<u8>, &'static str> {
    by_flask(a.fee, a.flask, allow_ix(a), ALLOW_UNITS, a.price, &a.blockhash)
}

/// What a flask paid back, nanos of $WARM, as its record on the chain holds it: the mine's own count of what moved from the flask into its vault.
/// None when the account is not this flask's record of this layout.
pub fn paid_back_of(d: &[u8], flask: &Pubkey) -> Option<u64> {
    if d.len() != Flask::SIZE || d[0] != DISC_FLASK || d[1] != warm_mine::state::LAYOUT_VERSION || &key_at(d, Flask::OFF_FLASK) != flask {
        return None;
    }
    Some(u64_at(d, Flask::OFF_PAID_BACK))
}
/// What a flask's own $WARM account holds for the mine, nanos: as much of its balance as the mine, its delegate, may still move; 0 for an account
/// of another mint or holder, or whose delegate is not the mine.
pub fn held_for_mine(d: &[u8], flask: &Pubkey) -> u64 {
    use warm_mine::token::{ACCOUNT_LEN, ACCOUNT_OFF_AMOUNT, ACCOUNT_OFF_DELEGATE, ACCOUNT_OFF_DELEGATED};
    if d.len() != ACCOUNT_LEN || key_at(d, 0) != mint_at() || &key_at(d, 32) != flask || d[ACCOUNT_OFF_DELEGATE..ACCOUNT_OFF_DELEGATE + 4] != [1, 0, 0, 0] || key_at(d, ACCOUNT_OFF_DELEGATE + 4) != mine_at() {
        return 0;
    }
    u64_at(d, ACCOUNT_OFF_AMOUNT).min(u64_at(d, ACCOUNT_OFF_DELEGATED))
}

/// The miner's `take_back` (ix.rs): [miner (s,w), mine (w), vault (w), $WARM mint, token program], then each flask's [record (w), its $WARM
/// account (w)]; data [n] then n × [target]: each record's `paid_back` brought to its target.
pub fn take_back_ix(miner: &Pubkey, takes: &[(Pubkey, u64)]) -> Instruction {
    let mut accounts = vec![AccountMeta::new(*miner, true), AccountMeta::new(mine_at(), false), AccountMeta::new(vault_at(), false), AccountMeta::new_readonly(mint_at(), false), AccountMeta::new_readonly(TOKEN, false)];
    let mut data = vec![warm_mine::IX_TAKE_BACK, takes.len() as u8];
    for (flask, target) in takes {
        accounts.extend([AccountMeta::new(record_of(flask), false), AccountMeta::new(warm_account_of(flask), false)]);
        data.extend_from_slice(&target.to_le_bytes());
    }
    Instruction { program_id: WARM, accounts, data }
}
/// The miner's one-signer message of a take-back.
pub fn take_back_message(miner: &Pubkey, takes: &[(Pubkey, u64)], blockhash: &[u8; 32]) -> Vec<u8> {
    Message::new_with_blockhash(&[take_back_ix(miner, takes)], Some(miner), &Hash::new_from_array(*blockhash)).serialize()
}

/// A tank that drank $WARM as credits where the mine is named: what it drank in all, nanos, and its flasks in the door's order.
#[derive(Clone, Debug)]
pub struct Owing {
    pub drunk: u64,
    pub flasks: Vec<Pubkey>,
}
/// THE TAKE-BACK'S PLAN: each tank's $WARM drunk past what its flasks' records say they paid back comes back from its flasks, in their order, as
/// far as each holds for the mine; one batch, as many flasks as Solana's packet carries, each flask once. `chain` holds each flask's (paid back,
/// held for the mine) as the chain holds them.
pub fn takes(owing: &[Owing], chain: &BTreeMap<Pubkey, (u64, u64)>, miner: &Pubkey) -> Vec<(Pubkey, u64)> {
    let mut out: Vec<(Pubkey, u64)> = Vec::new();
    for o in owing {
        let paid: u64 = o.flasks.iter().map(|f| chain.get(f).map_or(0, |c| c.0)).sum();
        let mut owed = o.drunk.saturating_sub(paid);
        for f in &o.flasks {
            let (p, h) = chain.get(f).copied().unwrap_or((0, 0));
            let give = owed.min(h);
            if give == 0 || out.iter().any(|t| t.0 == *f) {
                continue;
            }
            let next = [&out[..], &[(*f, p + give)]].concat();
            if next.len() > u8::MAX as usize || 1 + 64 + take_back_message(miner, &next, &[0; 32]).len() > crate::PACKET {
                return out;
            }
            (out, owed) = (next, owed - give);
        }
    }
    out
}

/// The mine as the chain holds it, the fields the crank reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MineState {
    pub miner: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub treasury: Pubkey,
    pub dollar: Pubkey,
    pub opening_ms: u64,
    pub last_seq: u64,
    pub last_at_ms: u64,
}
fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap_or([0; 8]))
}
fn key_at(d: &[u8], o: usize) -> Pubkey {
    Pubkey::new_from_array(d[o..o + 32].try_into().unwrap_or([0; 32]))
}
/// The mine's account read by the program's own offsets; None when it is not a mine of this layout.
pub fn mine_state(d: &[u8]) -> Option<MineState> {
    if d.len() != Mine::SIZE || d[0] != DISC_MINE {
        return None;
    }
    Some(MineState {
        miner: key_at(d, Mine::OFF_MINER),
        mint: key_at(d, Mine::OFF_MINT),
        vault: key_at(d, Mine::OFF_VAULT),
        treasury: key_at(d, Mine::OFF_TREASURY),
        dollar: key_at(d, Mine::OFF_DOLLAR),
        opening_ms: u64_at(d, Mine::OFF_OPENING),
        last_seq: u64_at(d, Mine::OFF_LAST_SEQ),
        last_at_ms: u64_at(d, Mine::OFF_LAST_AT),
    })
}
/// The chain's clock in the mine's milliseconds, from the Clock sysvar's account (its unix timestamp, the fifth of its five numbers), as the
/// program reads it (`util::now_ms`).
pub fn clock_ms(d: &[u8]) -> Option<u64> {
    let s = i64::from_le_bytes(d.get(32..40)?.try_into().ok()?);
    Some(if s <= 0 { 0 } else { (s as u64).saturating_mul(1_000) })
}
/// A flask's record: what it deposited and what it spent, micro-dollars; None when the account is not this flask's record.
pub fn room_of(d: &[u8], flask: &Pubkey) -> Option<(u64, u64)> {
    if d.len() != Flask::SIZE || d[0] != DISC_FLASK || &key_at(d, Flask::OFF_FLASK) != flask {
        return None;
    }
    Some((u64_at(d, Flask::OFF_DEPOSITED), u64_at(d, Flask::OFF_SPENT)))
}

/// One burn the router's ledger says mined, with its signed receipt: the receipt's id (the door's own counter), its instant, the fuel dollars it
/// burned in nano-dollars (its dollars less what its tank paid in $WARM), its tank, the tank's flasks in the order of their first deposit, and the
/// receipt as the door kept it (the canonical signed text).
#[derive(Clone, Debug)]
pub struct Spend {
    pub id: u64,
    pub at_ms: u64,
    pub nanos: u64,
    pub tank: String,
    pub flasks: Vec<Pubkey>,
    pub body: String,
}

/// A receipt held to the oracle: the text the door kept is canonical, so what was signed is that text without its `sig` member; its signature holds
/// under `oracle`, it names that key, this tank and this instant, and its dollars are no less than the fuel said of it.
pub fn receipt_holds(s: &Spend, oracle: &[u8; 32]) -> bool {
    let Ok(v) = serde_json::from_str::<Value>(&s.body) else { return false };
    let (Some(sig), Some(by), Some(tank), Some(at), Some(usd)) = (v["sig"].as_str(), v["by"].as_str(), v["tank"].as_str(), v["at"].as_u64(), v["usd"].as_f64()) else {
        return false;
    };
    let member = format!(",\"sig\":\"{sig}\"");
    if by != hex(oracle) || tank != s.tank || at != s.at_ms || s.body.matches(&member).count() != 1 || !(usd >= 0.0) || (usd * 1e9).round() < s.nanos as f64 {
        return false;
    }
    let signed = s.body.replacen(&member, "", 1);
    let (Some(sig), Ok(key)) = (unb64url(sig).and_then(|b| <[u8; 64]>::try_from(b).ok()), VerifyingKey::from_bytes(oracle)) else { return false };
    key.verify_strict(signed.as_bytes(), &Signature::from_bytes(&sig)).is_ok()
}

/// One entry of a `mine` batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub flask: Pubkey,
    pub usd_micros: u64,
    pub at_ms: u64,
    pub seq: u64,
}

/// What the crank does with the receipts it was given.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// The batch, in the chain's order; empty when nothing is due.
    pub entries: Vec<Entry>,
    /// The last receipt's id this plan read past (the mine's `last_seq` when it read none): the next page starts after it.
    pub through: u64,
    /// The last id of the receipts that, from the first one read, mined nothing at all (past their flasks' deposits, or not the oracle's); 0 when the
    /// first one read took a place in a batch. The door may read from after it the next time.
    pub passed: u64,
    /// The receipts read, and of them: those whose receipt did not hold, the nano-dollars past their flasks' deposits, and the nano-dollars under a
    /// micro-dollar of an entry.
    pub read: usize,
    pub unsigned: usize,
    pub held_nanos: u64,
    pub dust_nanos: u64,
    /// The receipts after `through` wait: the chain's clock has not passed the next one's instant, or the batch is full.
    pub waits: bool,
    /// THE ROUTER DRAWS ITS MONTH: of the entries that stand, what the AI provider charged for the spends in them, nano-dollars: each receipt's
    /// share the least of what it gives its entry and its `router_usd` (`router_nanos_of`).
    pub router_nanos: u64,
}

/// The `mine` instruction of a batch (ix.rs): [miner (s,w), mine (w), $WARM mint (w), vault (w), token program, associated token program, system],
/// then each entry's [flask, its record (w), its $WARM account (w)]; data [n] then n × [usd_micros][at_ms][seq].
pub fn mine_ix(miner: &Pubkey, entries: &[Entry]) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*miner, true),
        AccountMeta::new(mine_at(), false),
        AccountMeta::new(mint_at(), false),
        AccountMeta::new(vault_at(), false),
        AccountMeta::new_readonly(TOKEN, false),
        AccountMeta::new_readonly(ATA, false),
        AccountMeta::new_readonly(Pubkey::default(), false),
    ];
    let mut data = vec![warm_mine::IX_MINE, entries.len() as u8];
    for e in entries {
        accounts.extend([AccountMeta::new_readonly(e.flask, false), AccountMeta::new(record_of(&e.flask), false), AccountMeta::new(warm_account_of(&e.flask), false)]);
        for n in [e.usd_micros, e.at_ms, e.seq] {
            data.extend_from_slice(&n.to_le_bytes());
        }
    }
    Instruction { program_id: WARM, accounts, data }
}
/// The miner's one-signer message of a batch.
pub fn mine_message(miner: &Pubkey, entries: &[Entry], blockhash: &[u8; 32]) -> Vec<u8> {
    Message::new_with_blockhash(&[mine_ix(miner, entries)], Some(miner), &Hash::new_from_array(*blockhash)).serialize()
}
/// A batch's message within Solana's packet with its one signature and `reserve` bytes more (the draw's, `draw_room`).
fn fits(miner: &Pubkey, entries: &[Entry], reserve: usize) -> bool {
    entries.len() <= u8::MAX as usize && 1 + 64 + mine_message(miner, entries, &[0; 32]).len() + reserve <= crate::PACKET
}

/// THE CRANK'S PLAN: one batch from the receipts after the mine's cursor, in their order. `rooms` holds each flask's record as the chain holds it
/// (deposited, spent); a flask with none has no room.
pub fn plan(m: &MineState, clock_ms: u64, rows: &[Spend], rooms: &BTreeMap<Pubkey, (u64, u64)>, oracle: &[u8; 32]) -> Plan {
    plan_within(m, clock_ms, rows, rooms, oracle, 0)
}
/// The plan, its batch packed to leave `reserve` bytes of the packet free: the draw's instructions ride the same message (THE ROUTER DRAWS ITS MONTH).
pub fn plan_within(m: &MineState, clock_ms: u64, rows: &[Spend], rooms: &BTreeMap<Pubkey, (u64, u64)>, oracle: &[u8; 32], reserve: usize) -> Plan {
    let mut p = Plan { through: m.last_seq, ..Plan::default() };
    // a group: one flask's spends of one era; its nano-dollars, its last receipt's id and (stamped) instant, and what the provider charged for them
    let mut groups: Vec<(Pubkey, u64, u64, u64, u64, u64)> = Vec::new();
    let mut taken: BTreeMap<Pubkey, u64> = BTreeMap::new();
    let mut prev_at = m.last_at_ms;
    let mut placed = false;
    let shape = |groups: &[(Pubkey, u64, u64, u64, u64, u64)]| -> Vec<Entry> {
        let mut e: Vec<Entry> = groups.iter().map(|g| Entry { flask: g.0, usd_micros: g.2 / NANOS, at_ms: g.4, seq: g.3 }).collect();
        e.sort_by_key(|e| e.seq);
        e
    };
    for s in rows {
        if s.id <= p.through {
            continue;
        }
        if s.at_ms > clock_ms {
            p.waits = true;
            break;
        }
        if !receipt_holds(s, oracle) {
            (p.read, p.unsigned, p.through) = (p.read + 1, p.unsigned + 1, s.id);
            if !placed {
                p.passed = s.id;
            }
            continue;
        }
        // THE BOUND: the first of the tank's flasks with room left takes the spend, as far as its room goes
        let allowed = |f: &Pubkey| rooms.get(f).map_or(0, |(d, sp)| d.saturating_sub(*sp).saturating_mul(NANOS).saturating_sub(taken.get(f).copied().unwrap_or(0)));
        let Some((flask, room)) = s.flasks.iter().map(|f| (*f, allowed(f))).find(|(_, room)| *room > 0) else {
            (p.read, p.held_nanos, p.through) = (p.read + 1, p.held_nanos + s.nanos, s.id);
            if !placed {
                p.passed = s.id;
            }
            continue;
        };
        let give = s.nanos.min(room);
        // its share of the draw: what the provider charged for it, never more than it gives its entry (a spend cut by its flask's room draws as far
        // as it mines)
        let share = give.min(router_nanos_of(&s.body));
        let at = s.at_ms.max(prev_at);
        let era = warm_mine::law::era_at(at, m.opening_ms);
        let mut next = groups.clone();
        match next.iter_mut().find(|g| g.0 == flask && g.1 == era) {
            Some(g) => (g.2, g.3, g.4, g.5) = (g.2 + give, s.id, at, g.5 + share),
            None => next.push((flask, era, give, s.id, at, share)),
        }
        if next.len() > groups.len() && !fits(&m.miner, &shape(&next), reserve) {
            p.waits = true;
            break;
        }
        groups = next;
        placed = true;
        *taken.entry(flask).or_default() += give;
        prev_at = at;
        (p.read, p.held_nanos, p.through) = (p.read + 1, p.held_nanos + (s.nanos - give), s.id);
    }
    p.dust_nanos = groups.iter().map(|g| g.2 % NANOS).sum();
    // an entry under a micro-dollar is not mined, so its spends draw nothing either
    p.router_nanos = groups.iter().filter(|g| g.2 / NANOS > 0).map(|g| g.5).sum();
    p.entries = shape(&groups).into_iter().filter(|e| e.usd_micros > 0).collect();
    p
}

/// THE ROUTER DRAWS ITS MONTH (Nov 24 2026, lane/the-router-draws-its-month; his 00:07:00 UTC "Yes, build it" to "Build the router's own draw from
/// the treasury next?", after his Nov 23 22:20:46 UTC "we might get some money in the treasury, but unless we figure out how to allocate it as
/// credits.. u get me?"). Root 0's eighth act gave the mine's miner a month of the dollar (a TREASURY allowance, ROUTER_ACT in poleis/press):
/// here the miner draws it. No timer, no cursor, no store: the draw exists only inside a batch's own transaction, [the mine's `mine`, the associated
/// token program's CreateIdempotent of the miner's dollar account when it does not stand (the miner paying), Poleis's `pull`], one signer, the
/// miner. A batch that does not land drew nothing; one that lands drew with it. What it draws is what the AI provider charged for the batch's spends
/// (each receipt's `router_usd`, never more than it gives its entry), floored to the micro-dollar, never more than the micro-dollars the batch's
/// entries post (THE DRAW IS NEVER MORE THAN WHAT THE BATCH MINES), and cut to what the month, root 0's cap of the dollar and its vault let out at
/// the chain's slot; the door's markup (`usd` less `router_usd`) stays in the treasury, and what a cut leaves
/// undrawn is not carried: the cursor is the mine's, and a spend's share is drawn only with the batch that mines it, or never.
/// THE DRAW NEVER HOLDS THE MINE: the pull rides a batch only when every refusal of the program's `pull` (poleis_treasury src/ix/economy.rs) is
/// ruled out by the accounts read; otherwise the batch goes alone, as before this lane, and the plan says why in a word.
///
/// How many slots after the clock it read a batch may still land, so an allowance expiring within them draws nothing: a message's blockhash
/// lives 150 slots and the clock is read before it, so twice that. An allowance with no expiry (the launch's act 8) never comes near it.
pub const DRAW_MARGIN_SLOTS: u64 = 300;
/// The rent of a token account (165 bytes) the miner pays when the batch makes its dollar account.
pub const DOLLAR_ACCOUNT_RENT: u64 = 2_039_280;
/// THE MINER'S RENT: the lamports the miner must hold for a batch to make its dollar account: five times that rent, the create's own and room for
/// the batch's other makings the miner pays too (a flask's $WARM account and its record) and the fee. A miner that cannot pay would fail the
/// whole batch, so under it the month goes alone ("rent"); once the account stands its lamports are not asked.
pub const MINER_FLOOR_LAMPORTS: u64 = 5 * DOLLAR_ACCOUNT_RENT;

/// The chain's slot, the first of the Clock sysvar's five numbers (`clock_ms` reads its fifth from the same account).
pub fn clock_slot(d: &[u8]) -> Option<u64> {
    Some(u64::from_le_bytes(d.get(0..8)?.try_into().ok()?))
}
/// `owner`'s associated account of the dollar (the classic token program's, as the mine's dollar is).
pub fn dollar_account_of(owner: &Pubkey, dollar: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN.as_ref(), dollar.as_ref()], &ATA).0
}
/// What the provider charged for a spend, nano-dollars, as its signed receipt says (`router_usd`); 0 when it says null, or not a finite number of
/// dollars no less than 0.
pub fn router_nanos_of(body: &str) -> u64 {
    match serde_json::from_str::<Value>(body).ok().and_then(|v| v["router_usd"].as_f64()) {
        Some(usd) if usd.is_finite() && usd >= 0.0 => (usd * 1e9).round() as u64,
        _ => 0,
    }
}
/// A token account of the classic program, initialized and not frozen: its mint, its owner, its amount.
fn token_of(d: &[u8]) -> Option<(Pubkey, Pubkey, u64)> {
    use warm_mine::token::{ACCOUNT_LEN, ACCOUNT_OFF_AMOUNT, ACCOUNT_OFF_MINT, ACCOUNT_OFF_OWNER, ACCOUNT_OFF_STATE, STATE_INITIALIZED};
    (d.len() == ACCOUNT_LEN && d[ACCOUNT_OFF_STATE] == STATE_INITIALIZED).then(|| (key_at(d, ACCOUNT_OFF_MINT), key_at(d, ACCOUNT_OFF_OWNER), u64_at(d, ACCOUNT_OFF_AMOUNT)))
}

/// The accounts of a month as the chain holds them (None: none stands): the allowance, its unit's registry, the mine's treasury (the unit's
/// vault of the dollar), the miner's dollar account, and the miner's lamports.
#[derive(Clone, Copy, Debug, Default)]
pub struct MonthAccounts<'a> {
    pub allowance: Option<&'a [u8]>,
    pub registry: Option<&'a [u8]>,
    pub vault: Option<&'a [u8]>,
    pub dest: Option<&'a [u8]>,
    /// The miner's own lamports, as the node answered them (None: not read).
    pub miner_lamports: Option<u64>,
}
/// A month the program's `pull` would let the miner draw from now: its proposal's hash and unit, the dollar, the miner's dollar account and whether
/// the batch makes it, and what the allowance (its period rolled), the unit's cap of the dollar (its period rolled) and the vault let out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Month {
    pub hash: [u8; 32],
    pub unit: u64,
    pub dollar: Pubkey,
    pub dest: Pubkey,
    pub create: bool,
    pub month_left: u64,
    pub cap_left: u64,
    pub vault_holds: u64,
}
impl Month {
    /// The most a pull may take now.
    pub fn left(&self) -> u64 {
        self.month_left.min(self.cap_left).min(self.vault_holds)
    }
}

/// The month at `month` read as the program's `pull` reads it at `slot`, every refusal in its order: none of them, a Month; any, the word for it.
/// The signer is the miner (the batch's one); the amount is cut to `left` and drawn only above 0 (`draw_of`).
pub fn month_of(m: &MineState, slot: u64, month: &Pubkey, at: &MonthAccounts) -> Result<Month, &'static str> {
    use poleis_client::{allowance_pda, treasury_pda, vault_pda, AllowanceView, RegistryView, HOLDER_KEY};
    // the allowance: this one, at its own address (`read_allowance`)
    let a = at.allowance.and_then(AllowanceView::parse).filter(|a| allowance_pda(&a.hash) == *month && a.period > 0).ok_or("allowance")?;
    // not expired at any slot the batch may land at (`AllowanceDead`)
    if a.expires != 0 && slot.saturating_add(DRAW_MARGIN_SLOTS) > a.expires {
        return Err("expired");
    }
    // its unit's registry (`read_registry`: its unit, at its own address, which `wants` and `plan` read it at), not frozen (`Frozen`)
    let r = at.registry.and_then(RegistryView::parse).filter(|r| r.unit == a.unit).ok_or("registry")?;
    if r.frozen {
        return Err("frozen");
    }
    // a key payee, the miner (`Unauthorized`)
    if a.payee_kind != HOLDER_KEY || a.payee != m.miner.to_bytes() {
        return Err("payee");
    }
    // the mine's dollar, the registry's default mint: the plan's own counter of it (`charge_token_plan`'s first branch), never a TokenPlan's
    if a.mint != m.dollar || r.default_mint != m.dollar || m.dollar == Pubkey::default() {
        return Err("mint");
    }
    // the miner's own dollar account, or none: the batch makes it (`check_dest`: a token account its payee owns)
    let dest = dollar_account_of(&m.miner, &m.dollar);
    let create = match at.dest {
        None | Some([]) => true,
        Some(d) => match token_of(d) {
            Some((mint, owner, _)) if mint == m.dollar && owner == m.miner => false,
            _ => return Err("account"),
        },
    };
    // THE MINER'S RENT: a batch that makes the account is paid by the miner, or it fails whole
    if create && at.miner_lamports.is_none_or(|l| l < MINER_FLOOR_LAMPORTS) {
        return Err("rent");
    }
    // the vault is the mine's treasury, the unit's vault of the dollar, its authority the unit's treasury (`expect_vault`, the transfer)
    let vault_holds = match at.vault.and_then(token_of) {
        Some((mint, owner, amount)) if m.treasury == vault_pda(a.unit, &m.dollar) && mint == m.dollar && owner == treasury_pda(a.unit) => amount,
        _ => return Err("vault"),
    };
    // the allowance's own period, rolled at the slot as `pull` rolls it: a later slot only rolls it further, never to less room
    let month_left = if slot >= a.start.saturating_add(a.period) { a.cap } else { a.cap.saturating_sub(a.spent) };
    // the unit's cap of the dollar, its period rolled as `charge_token_plan` rolls it
    let rolled = r.plan_period > 0 && slot >= r.plan_start.saturating_add(r.plan_period);
    let cap_left = r.token_plan_cap.saturating_sub(if rolled { 0 } else { r.token_plan_spent });
    match (month_left, cap_left, vault_holds) {
        (0, _, _) => Err("spent"),
        (_, 0, _) => Err("cap"),
        (_, _, 0) => Err("empty"),
        _ => Ok(Month { hash: a.hash, unit: a.unit, dollar: m.dollar, dest, create, month_left, cap_left, vault_holds }),
    }
}

/// The draw's instructions, after the batch's `mine`: the miner's dollar account made when it does not stand (CreateIdempotent: [payer (s,w), the
/// account (w), its owner, the mint, system, the token program], data [1]), then Poleis's `pull` of `amount`, the client's own builder.
pub fn draw_ixs(miner: &Pubkey, month: &Month, amount: u64) -> Vec<Instruction> {
    let mut ixs = Vec::new();
    if month.create {
        ixs.push(Instruction {
            program_id: ATA,
            accounts: vec![
                AccountMeta::new(*miner, true),
                AccountMeta::new(month.dest, false),
                AccountMeta::new_readonly(*miner, false),
                AccountMeta::new_readonly(month.dollar, false),
                AccountMeta::new_readonly(Pubkey::default(), false),
                AccountMeta::new_readonly(TOKEN, false),
            ],
            data: vec![1],
        });
    }
    ixs.push(poleis_client::pull(*miner, month.hash, month.unit, amount, month.dest, Some((month.dollar, TOKEN))));
    ixs
}
/// The miner's one-signer message of a batch, with its draw when one rides it.
pub fn draw_message(miner: &Pubkey, entries: &[Entry], draw: Option<(&Month, u64)>, blockhash: &[u8; 32]) -> Vec<u8> {
    let mut ixs = vec![mine_ix(miner, entries)];
    if let Some((month, amount)) = draw {
        ixs.extend(draw_ixs(miner, month, amount));
    }
    Message::new_with_blockhash(&ixs, Some(miner), &Hash::new_from_array(*blockhash)).serialize()
}
/// The bytes the draw adds to a batch's message, measured: an empty batch's message with the draw less the one without. The draw's accounts are
/// none of a batch's (its flasks, their records and $WARM accounts), so it adds as much to any batch.
pub fn draw_room(miner: &Pubkey, month: &Month) -> usize {
    draw_message(miner, &[], Some((month, 0)), &[0; 32]).len() - mine_message(miner, &[], &[0; 32]).len()
}
/// What the batch of `p` draws from `month`: the provider's charge of its spends floored to the micro-dollar, never more than the micro-dollars
/// its entries post (their shares are summed before the floor, so their sum could pass the entries' own floors by under a micro-dollar an entry:
/// THE DRAW IS NEVER MORE THAN WHAT THE BATCH MINES), cut to what the month, the cap and the vault let out; "nothing" when that is 0 or the batch
/// is empty.
pub fn draw_of(p: &Plan, month: &Month) -> Result<u64, &'static str> {
    let mined: u64 = p.entries.iter().map(|e| e.usd_micros).sum();
    match (p.router_nanos / NANOS).min(mined).min(month.left()) {
        _ if p.entries.is_empty() => Err("nothing"),
        0 => Err("nothing"),
        amount => Ok(amount),
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn unhex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 || !s.is_ascii() {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub fn b64(bytes: &[u8]) -> String {
    let mut out = String::new();
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (c.get(1).copied().unwrap_or(0) as u32) << 8 | c.get(2).copied().unwrap_or(0) as u32;
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            out.push(if i <= c.len() { B64[(n >> shift & 63) as usize] as char } else { '=' });
        }
    }
    out
}
/// base64, standard or URL-safe, padded or not; None on any other byte.
pub fn unb64(s: &str) -> Option<Vec<u8>> {
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0u32);
    for b in s.bytes().filter(|b| *b != b'=') {
        let v = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return None,
        };
        (acc, bits) = (acc << 6 | v as u32, bits + 6);
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}
fn unb64url(s: &str) -> Option<Vec<u8>> {
    unb64(s)
}

fn key(v: &Value) -> Result<Pubkey, String> {
    v.as_str().and_then(|s| Pubkey::from_str(s).ok()).ok_or_else(|| "not an address".to_string())
}
fn number(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}
/// An account's bytes as a node answers it (`["<base64>", "base64"]`), or the base64 itself.
fn account(v: &Value) -> Option<Vec<u8>> {
    unb64(v.as_str().or_else(|| v.get("data").unwrap_or(v).get(0).and_then(Value::as_str))?)
}
/// An account's bytes when the program `by` owns it (a node's answer that names no owner is taken as it is); an empty account of the system
/// program's as empty (an address the batch may still make); any other owner's as no account of the kind asked (one byte, which nothing parses).
fn owned(v: &Value, by: &Pubkey) -> Option<Vec<u8>> {
    let d = account(v)?;
    Some(match v.get("owner").and_then(Value::as_str) {
        None => d,
        Some(o) if o == by.to_string() => d,
        Some(o) if d.is_empty() && o == Pubkey::default().to_string() => d,
        Some(_) => vec![0xff],
    })
}
/// The accounts a month's draw reads, beside the flasks' records: the allowance, its unit's registry, the mine's treasury, the miner's dollar account,
/// and the miner itself (its lamports, THE MINER'S RENT).
fn month_reads(m: &MineState, month: &Pubkey, unit: u64) -> [(Pubkey, &'static str, Pubkey); 5] {
    let (poleis, registry) = (crate::PROGRAM, poleis_client::registry_pda(unit));
    [(*month, "month", poleis), (registry, "registry", poleis), (m.treasury, "vault", TOKEN), (dollar_account_of(&m.miner, &m.dollar), "the miner's dollar", TOKEN), (m.miner, "the miner", Pubkey::default())]
}
fn flasks_of(v: &Value) -> Vec<Pubkey> {
    v.as_str().unwrap_or("").split(',').filter_map(|s| Pubkey::from_str(s).ok()).collect()
}
fn rows_of(v: &Value) -> Vec<Spend> {
    let row = |r: &Value| Some(Spend { id: number(&r["id"])?, at_ms: number(&r["at"])?, nanos: number(&r["nanos"])?, tank: r["tank"].as_str()?.to_string(), flasks: flasks_of(&r["flasks"]), body: r["body"].as_str()?.to_string() });
    v.as_array().map(|a| a.iter().filter_map(row).collect()).unwrap_or_default()
}

/// Every flask the owing tanks name, each once, in their order: the accounts `takes` reads are this list's records and $WARM accounts.
fn flasks_once(rows: &[Owing]) -> Vec<Pubkey> {
    let mut all: Vec<Pubkey> = Vec::new();
    for f in rows.iter().flat_map(|o| o.flasks.iter()) {
        if !all.contains(f) {
            all.push(*f);
        }
    }
    all
}
fn owing_of(v: &Value) -> Vec<Owing> {
    let row = |r: &Value| Some(Owing { drunk: number(&r["drunk"])?, flasks: flasks_of(&r["flasks"]) });
    v.as_array().map(|a| a.iter().filter_map(row).collect()).unwrap_or_default()
}

/// THE ONE DOOR, JSON in and JSON out; `do` names what is asked:
/// - `terms`: -> {program, mine, mint, vault, clock}: the addresses this build speaks.
/// - `deposit` {flask, source, treasury, dollar, amount, blockhash, fee?, price?, commitment?}: -> {message (base64), record}.
/// - `cursor` {mine, clock} (the two accounts as a node answers them): -> {seq, at, clock_ms, miner, treasury, dollar, opening}.
/// - `wants` {rows, mine?, month?}: -> {records: {<record's address>: <its flask>}}: the records the plan reads; with the mine's account and a month
///   (its allowance's address), the draw's five accounts beside them (the miner's own among them, for its lamports) (THE ROUTER DRAWS ITS MONTH).
/// - `plan` {mine, clock, rows, records: {<address>: <account>}, oracle (hex), blockhash, month?}: -> {message (base64, or null when nothing is due),
///   spends, usd_micros, through, passed, read, unsigned, held_nanos, dust_nanos, waits}; with a month, and {draw (micro-dollars the message pulls,
///   0 when none), draw_why (null when it draws, else the word the batch goes alone by)}.
/// - `back` {flask, amount, blockhash, fee?, price?}: -> {message (base64), amount, source, record}: the pay-back of `amount` nanos of the flask's $WARM.
/// - `paid` {flask, account? (its record as a node answers it), token? (its $WARM account)}: -> {record, source, paid?, held?}: the record's and the
///   $WARM account's addresses; with the record read, what the flask paid back (0 while none stands); with its $WARM account, what it holds for the mine.
/// - `allow` {flask, blockhash, fee?, price?}: -> {message, source, record}: the flask's one approval, the mine the delegate of its $WARM.
/// - `owing` {rows: [{drunk (nanos), flasks}]}: -> {read}: the accounts the take-back reads, each flask's record and its $WARM account.
/// - `takes` {rows, accounts (as `read` named them), miner, blockhash}: -> {message (or null), takes, nanos, flasks: [{flask, paid, held}]}.
/// A refusal is {error}.
pub fn ask(text: &str) -> String {
    let out = (|| -> Result<Value, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let hash = |v: &Value| key(v).map(|k| k.to_bytes());
        match v["do"].as_str().unwrap_or("") {
            "terms" => Ok(json!({ "program": WARM.to_string(), "mine": mine_at().to_string(), "mint": mint_at().to_string(), "vault": vault_at().to_string(), "clock": CLOCK.to_string() })),
            "deposit" => {
                let d = Deposit {
                    fee: if v["fee"].is_null() { None } else { Some(key(&v["fee"])?) },
                    flask: key(&v["flask"])?,
                    source: key(&v["source"])?,
                    treasury: key(&v["treasury"])?,
                    dollar: key(&v["dollar"])?,
                    amount: number(&v["amount"]).ok_or("not an amount")?,
                    blockhash: hash(&v["blockhash"])?,
                    price: if v["price"].is_null() { None } else { Some(number(&v["price"]).ok_or("not a price")?) },
                    commitment: v["commitment"].as_str().map(str::to_string),
                };
                Ok(json!({ "message": b64(&deposit_message(&d)?), "record": record_of(&d.flask).to_string() }))
            }
            "cursor" => {
                let m = account(&v["mine"]).as_deref().and_then(mine_state).ok_or("the mine does not stand")?;
                let clock = account(&v["clock"]).as_deref().and_then(clock_ms).ok_or("no clock")?;
                Ok(json!({ "seq": m.last_seq, "at": m.last_at_ms, "clock_ms": clock, "miner": m.miner.to_string(), "treasury": m.treasury.to_string(), "dollar": m.dollar.to_string(), "opening": m.opening_ms }))
            }
            "wants" => {
                let mut records = serde_json::Map::new();
                for f in rows_of(&v["rows"]).iter().flat_map(|r| r.flasks.clone()) {
                    records.insert(record_of(&f).to_string(), json!(f.to_string()));
                }
                // THE ROUTER DRAWS ITS MONTH: with the mine and a month named, the accounts the draw reads (the allowance's unit read at plan time;
                // here root 0's, the launch's act 8)
                if let (Some(m), Ok(month)) = (account(&v["mine"]).as_deref().and_then(mine_state), key(&v["month"])) {
                    for (k, what, _) in month_reads(&m, &month, poleis_client::ROOT_UNIT) {
                        records.insert(k.to_string(), json!(what));
                    }
                }
                Ok(json!({ "records": records }))
            }
            "plan" => {
                let m = account(&v["mine"]).as_deref().and_then(mine_state).ok_or("the mine does not stand")?;
                let clock = account(&v["clock"]).as_deref().and_then(clock_ms).ok_or("no clock")?;
                let oracle = v["oracle"].as_str().and_then(unhex32).ok_or("the oracle's key is 64 hex")?;
                let rows = rows_of(&v["rows"]);
                let mut rooms = BTreeMap::new();
                for f in rows.iter().flat_map(|r| r.flasks.iter()) {
                    if let Some(room) = account(&v["records"][record_of(f).to_string()]).and_then(|d| room_of(&d, f)) {
                        rooms.insert(*f, room);
                    }
                }
                // THE ROUTER DRAWS ITS MONTH: a month named is read as the program's pull reads it, and stands or says the word it does not by
                let month = (!v["month"].is_null()).then(|| {
                    let at = key(&v["month"]).map_err(|_| "month")?;
                    let slot = account(&v["clock"]).as_deref().and_then(clock_slot).ok_or("clock")?;
                    let allowance = owned(&v["records"][at.to_string()], &crate::PROGRAM);
                    let unit = allowance.as_deref().and_then(poleis_client::AllowanceView::parse).map_or(poleis_client::ROOT_UNIT, |a| a.unit);
                    let [_, (registry, _, by_r), (vault, _, by_v), (dest, _, by_d), (miner, _, _)] = month_reads(&m, &at, unit);
                    let (registry, vault, dest) = (owned(&v["records"][registry.to_string()], &by_r), owned(&v["records"][vault.to_string()], &by_v), owned(&v["records"][dest.to_string()], &by_d));
                    let miner_lamports = number(&v["records"][miner.to_string()]["lamports"]);
                    month_of(&m, slot, &at, &MonthAccounts { allowance: allowance.as_deref(), registry: registry.as_deref(), vault: vault.as_deref(), dest: dest.as_deref(), miner_lamports })
                });
                let reserve = match &month {
                    Some(Ok(mo)) => draw_room(&m.miner, mo),
                    _ => 0,
                };
                let p = plan_within(&m, clock, &rows, &rooms, &oracle, reserve);
                let mut draw = month.map(|mo| mo.and_then(|mo| draw_of(&p, &mo).map(|amount| (mo, amount))));
                let message = if p.entries.is_empty() {
                    Value::Null
                } else {
                    let blockhash = hash(&v["blockhash"])?;
                    let with = draw.as_ref().and_then(|d| d.as_ref().ok()).map(|(mo, amount)| draw_message(&m.miner, &p.entries, Some((mo, *amount)), &blockhash));
                    match with {
                        Some(bytes) if 1 + 64 + bytes.len() <= crate::PACKET => json!(b64(&bytes)),
                        _ => {
                            // never past the packet: the reserve is measured, so this is the law's own bug, and the batch still goes alone
                            if with.is_some() {
                                draw = Some(Err("room"));
                            }
                            json!(b64(&mine_message(&m.miner, &p.entries, &blockhash)))
                        }
                    }
                };
                let mut out = json!({ "message": message, "spends": p.entries.len(), "usd_micros": p.entries.iter().map(|e| e.usd_micros).sum::<u64>(), "through": p.through, "passed": p.passed, "read": p.read,
                    "unsigned": p.unsigned, "held_nanos": p.held_nanos, "dust_nanos": p.dust_nanos, "waits": p.waits });
                if let Some(d) = draw {
                    (out["draw"], out["draw_why"]) = match d {
                        Ok((_, amount)) => (json!(amount), Value::Null),
                        Err(why) => (json!(0), json!(why)),
                    };
                }
                Ok(out)
            }
            "back" => {
                let (flask, amount) = (key(&v["flask"])?, number(&v["amount"]).ok_or("not an amount")?);
                let (source, record) = (warm_account_of(&flask).to_string(), record_of(&flask).to_string());
                let b = PayBack {
                    fee: if v["fee"].is_null() { None } else { Some(key(&v["fee"])?) },
                    flask,
                    amount,
                    blockhash: hash(&v["blockhash"])?,
                    price: if v["price"].is_null() { None } else { Some(number(&v["price"]).ok_or("not a price")?) },
                };
                Ok(json!({ "message": b64(&pay_back_message(&b)?), "amount": amount, "source": source, "record": record }))
            }
            "allow" => {
                let flask = key(&v["flask"])?;
                let a = Allow {
                    fee: if v["fee"].is_null() { None } else { Some(key(&v["fee"])?) },
                    flask,
                    blockhash: hash(&v["blockhash"])?,
                    price: if v["price"].is_null() { None } else { Some(number(&v["price"]).ok_or("not a price")?) },
                };
                Ok(json!({ "message": b64(&allow_message(&a)?), "source": warm_account_of(&flask).to_string(), "record": record_of(&flask).to_string() }))
            }
            "owing" => Ok(json!({ "read": flasks_once(&owing_of(&v["rows"])).iter().flat_map(|f| [record_of(f).to_string(), warm_account_of(f).to_string()]).collect::<Vec<_>>() })),
            "takes" => {
                let (rows, miner) = (owing_of(&v["rows"]), key(&v["miner"])?);
                let flasks = flasks_once(&rows);
                let read = v["accounts"].as_array().ok_or("no accounts")?;
                let mut chain = BTreeMap::new();
                for (i, f) in flasks.iter().enumerate() {
                    let paid = account(read.get(2 * i).unwrap_or(&Value::Null)).and_then(|d| paid_back_of(&d, f)).unwrap_or(0);
                    let held = account(read.get(2 * i + 1).unwrap_or(&Value::Null)).map_or(0, |d| held_for_mine(&d, f));
                    chain.insert(*f, (paid, held));
                }
                let t = takes(&rows, &chain, &miner);
                let message = if t.is_empty() { Value::Null } else { json!(b64(&take_back_message(&miner, &t, &hash(&v["blockhash"])?))) };
                let nanos: u64 = t.iter().map(|(f, target)| target - chain[f].0).sum();
                Ok(json!({ "message": message, "takes": t.len(), "nanos": nanos, "flasks": flasks.iter().map(|f| json!({ "flask": f.to_string(), "paid": chain[f].0, "held": chain[f].1 })).collect::<Vec<_>>() }))
            }
            "paid" => {
                let flask = key(&v["flask"])?;
                let mut out = json!({ "record": record_of(&flask).to_string(), "source": warm_account_of(&flask).to_string() });
                if let Some(t) = v.get("token") {
                    out["held"] = json!(account(t).map_or(0, |d| held_for_mine(&d, &flask)));
                }
                if let Some(a) = v.get("account") {
                    // none stands: nothing was paid back; one that stands is the mine's own record of this flask, or a refusal
                    let paid = match account(a) {
                        None => 0,
                        Some(_) if a.get("owner").and_then(Value::as_str).is_some_and(|o| o != WARM.to_string()) => return Err("not the mine's record".into()),
                        Some(d) => paid_back_of(&d, &flask).ok_or("not this flask's record")?,
                    };
                    out["paid"] = json!(paid);
                }
                Ok(out)
            }
            other => Err(format!("no such ask: {other:?}")),
        }
    })();
    out.unwrap_or_else(|e| json!({ "error": e })).to_string()
}
