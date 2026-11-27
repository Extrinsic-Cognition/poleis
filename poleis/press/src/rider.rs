//! THE WELL PAYS ITS RIDER (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider; his "Yes, build them" at 13:25 UTC to "Build the three missing
//! links now?", over the launch's road whose link 7 reads "The well pays its Snowtracer"). His words it stands on, UTC: Nov 25 13:11 "we gotta launch
//! swarmbook and the stake sol to fund snowtracers... i need some salary and that i can only think would come from the snowtracing staking sol there";
//! Nov 20 12:13 "a snowtracer is a top tier claw rider".
//!
//! The decisions a swarm votes, each written as a set any of its members proposes and its voices vote the way they vote anything:
//!
//! - THE RIDER (`rider_act`): a wish's own TREASURY_ALLOWANCE (the shape of the launch's act 8, the router's month, on the wish's unit instead of
//!   root 0's) giving one rider's key up to `cap` of her pool's token a `period` of slots out of the wish's vault, pulled by the rider's own signature
//!   (`poleis_door::draw`), standing until the wish revokes it. Its execute admits the rider: nothing else names a Snowtracer. The cap and the period
//!   are the caller's; no figure and no person is written here.
//! - THE PLAN (`plan_act`): a unit's CODEX AllowMint of her pool's token, its cap a plan period. Poleis charges every pull of a mint that is not the
//!   unit's default to that plan, so a well pays nothing in her token, and root 0 pours nothing into a well, until its own plan of the mint stands.
//! - THE WELL'S ROAD (`well_act`): root 0's TREASURY_ALLOWANCE whose payee is a wish's unit (HOLDER_UNIT), in her pool's token: the standing road the
//!   light's pour runs on (light.rs), voted once for each wish by the whole tree.

use crate::{wire, Res};
use poleis_client::{self as client, Payload, HOLDER_KEY, HOLDER_UNIT, ROOT_UNIT, VOTE_PRO};
use serde_json::{json, Value};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::str::FromStr;

/// One decision of a unit: its words (their sha256 the manifest), its payload, its nonce and the proposal's hash.
#[derive(Clone, Debug)]
pub struct Decision {
    pub what: String,
    pub payload: Payload,
    pub manifest: [u8; 32],
    pub nonce: u64,
    pub hash: [u8; 32],
}

pub fn decision(what: &str, payload: Payload, nonce: u64) -> Decision {
    let manifest = wire::sha256(what.as_bytes());
    let hash = client::proposal_hash(payload.class(), payload.origin(), &manifest, &payload.hash(), nonce);
    Decision { what: what.to_string(), payload, manifest, nonce, hash }
}

/// THE RIDER: the wish `unit` gives `rider` up to `cap` of `mint` a `period` of slots, pulled by the rider's own signature, no expiry.
pub fn rider_act(unit: u64, rider: &Pubkey, mint: &Pubkey, cap: u64, period: u64) -> Payload {
    Payload::Allowance { origin: unit, payee: rider.to_bytes(), payee_kind: HOLDER_KEY, mint: *mint, cap, period, expires: 0 }
}

/// THE PLAN: `unit` may spend up to `cap` of `mint` a plan period; `payer` pays the plan's account at the execute and is its rent's home.
pub fn plan_act(unit: u64, mint: &Pubkey, cap: u64, payer: &Pubkey) -> Payload {
    Payload::AllowMint { origin: unit, mint: *mint, cap, payer: *payer }
}

/// THE WELL'S ROAD: root 0 lets up to `cap` of `mint` a `period` of slots be pulled into the wish `unit`'s own vault, by anyone, no expiry.
pub fn well_act(unit: u64, mint: &Pubkey, cap: u64, period: u64) -> Payload {
    Payload::Allowance { origin: ROOT_UNIT, payee: client::unit_holder(unit), payee_kind: HOLDER_UNIT, mint: *mint, cap, period, expires: 0 }
}

fn ix_json(ix: &Instruction) -> Value {
    json!({
        "program": ix.program_id.to_string(),
        "accounts": ix.accounts.iter().map(|a| json!({ "address": a.pubkey.to_string(), "signer": a.is_signer, "writable": a.is_writable })).collect::<Vec<_>>(),
        "data": wire::b64(&ix.data),
    })
}

/// A decision's set: the act, the proposal `proposer` signs, the envelope each voice of the unit signs (a message, never a transaction), and the
/// execute anyone sends once it is sealed. `root` is the unit's tree (a wish is its own root). Nothing here is signed or sent.
pub fn set_json(d: &Decision, root: u64, proposer: Option<&Pubkey>, crank: &Pubkey) -> Value {
    let unit = d.payload.origin();
    let envelope = client::vote_envelope(&d.hash, unit, VOTE_PRO);
    let mut v = json!({
        "what": d.what, "unit": unit.to_string(), "root": root.to_string(), "class": d.payload.class(), "nonce": d.nonce,
        "manifest": wire::hex(&d.manifest), "payload": wire::b64(&d.payload.bytes()), "payloadSha256": wire::hex(&d.payload.hash()), "hash": wire::hex(&d.hash),
        "envelope": { "bytes": wire::b64(&envelope), "text": format!("poleis:vote:v1 ‖ {} ‖ unit {unit} ‖ PRO", wire::hex(&d.hash)), "does": "each voice of the unit (a pentagon's or a hexagon's key) signs these bytes as its PRO; the crank lands the signatures (attest), seals, executes" },
        "execute": ix_json(&client::execute_of(root, *crank, d.hash, &[unit], &d.payload)),
    });
    if let Some(p) = proposer {
        v["propose"] = ix_json(&client::propose_of(root, *p, d.hash, d.manifest, &d.payload, &[unit]));
    }
    if matches!(d.payload, Payload::Allowance { .. }) {
        v["allowance"] = json!(client::allowance_pda(&d.hash).to_string());
    }
    v
}

/// `press rider --unit U --key K --mint M --cap N --period S [--plan-cap C] [--root R] [--nonce n] [--proposer P] [--crank C] [--out F]` and
/// `press well --unit U --mint M --cap N --period S [--plan-cap C] [--nonce n] [--proposer P] [--crank C] [--out F]`: the sets, written to F or said.
/// With --plan-cap the unit's plan of the mint comes first (the wish's for a rider, root 0's for a well). The figures are the caller's.
pub fn step(kind: &str, opts: &BTreeMap<String, String>) -> Res<Value> {
    let get = |k: &str| opts.get(k).map(String::as_str).ok_or_else(|| format!("{k} is needed"));
    let num = |k: &str| get(k)?.parse::<u64>().map_err(|_| format!("{k}: a whole number"));
    let key = |k: &str| Pubkey::from_str(get(k)?).map_err(|_| format!("{k}: not an address"));
    let unit = num("--unit")?;
    if unit == ROOT_UNIT {
        return Err("--unit: a wish's unit, never root 0".into());
    }
    let mint = key("--mint")?;
    let (cap, period) = (num("--cap")?, num("--period")?);
    if cap == 0 || period == 0 {
        return Err("--cap and --period: more than nothing (Poleis refuses an allowance of 0 or a period of 0)".into());
    }
    let nonce = opts.get("--nonce").map(|_| num("--nonce")).transpose()?.unwrap_or(1);
    let proposer = opts.get("--proposer").map(|_| key("--proposer")).transpose()?;
    let crank = opts.get("--crank").map(|_| key("--crank")).transpose()?.unwrap_or_default();
    let (origin, root, act) = match kind {
        "rider" => {
            let rider = key("--key")?;
            let root = opts.get("--root").map(|_| num("--root")).transpose()?.unwrap_or(unit);
            (unit, root, decision(&format!("unit {unit} gives the rider {rider} up to {cap} of {mint} a period of {period} slots"), rider_act(unit, &rider, &mint, cap, period), nonce))
        }
        "well" => (ROOT_UNIT, ROOT_UNIT, decision(&format!("root 0 lets up to {cap} of {mint} a period of {period} slots be pulled into unit {unit}'s vault"), well_act(unit, &mint, cap, period), nonce)),
        other => return Err(format!("{other}: rider or well")),
    };
    let mut sets = Vec::new();
    if let Some(c) = opts.get("--plan-cap") {
        let c = c.parse::<u64>().map_err(|_| "--plan-cap: a whole number")?;
        let payer = opts.get("--crank").map(|_| crank).ok_or("--plan-cap: --crank, who pays the plan's account")?;
        let d = decision(&format!("unit {origin} may spend up to {c} of {mint} a plan period"), plan_act(origin, &mint, c, &payer), nonce + 1);
        sets.push(set_json(&d, root, proposer.as_ref(), &crank));
    }
    sets.push(set_json(&act, root, proposer.as_ref(), &crank));
    let out = json!({ "what": if kind == "rider" { "THE WELL PAYS ITS RIDER: the wish's own decisions" } else { "THE LIGHT PAYS A WISH: root 0's road into a wish's well" }, "sets": sets });
    if let Some(f) = opts.get("--out") {
        std::fs::write(f, serde_json::to_string_pretty(&out).unwrap() + "\n").map_err(|e| format!("{f}: {e}"))?;
    }
    Ok(out)
}
