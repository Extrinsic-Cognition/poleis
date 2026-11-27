//! The ceremony's host adapter. One JSON request on stdin, one JSON line out.
//! Every step of a founding and of an upgrade by the seal is encoded by this
//! crate's own builders, and the legacy message is compiled by Solana's own
//! compiler. No RPC, no keys, no signing: the host
//! (`poleis/ceremony/rehearse.mjs`) signs the message it gets back and sends it.
//!
//! Build: `cargo build --release --example ceremony --manifest-path poleis/client/Cargo.toml`.
//! The program id is the program crate's own; the rehearsal builds this
//! adapter from the same patched copy it deploys, so the two never differ.
//!
//! Every reply is `{ "ok": true, ... }` or `{ "ok": false, "error": "..." }`.
//! A step's reply carries `messageBase64` and `signers` (the message's signer
//! keys in order, the fee payer first).

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use poleis_client::{self as client, GenesisSpec, Payload, PoleisError, HOLDER_KEY, ROOT_UNIT, VOTE_PRO};
use serde_json::{json, Value};
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;
use std::{
    io::{Read, Write},
    str::FromStr,
};

const MAX_INPUT: usize = 64 * 1024;
type Result<T> = std::result::Result<T, String>;

/// The compute budget program's SetComputeUnitLimit, for steps that ask for one.
const COMPUTE_BUDGET: Pubkey = solana_pubkey::pubkey!("ComputeBudget111111111111111111111111111111");

fn field<'a>(r: &'a Value, name: &str) -> Result<&'a Value> {
    r.get(name).ok_or_else(|| format!("missing-{name}"))
}
fn key(r: &Value, name: &str) -> Result<Pubkey> {
    let s = field(r, name)?.as_str().ok_or_else(|| format!("invalid-{name}"))?;
    Pubkey::from_str(s).map_err(|_| format!("invalid-{name}"))
}
fn keys(r: &Value, name: &str) -> Result<Vec<Pubkey>> {
    let a = field(r, name)?.as_array().ok_or_else(|| format!("invalid-{name}"))?;
    a.iter()
        .map(|v| v.as_str().and_then(|s| Pubkey::from_str(s).ok()).ok_or_else(|| format!("invalid-{name}")))
        .collect()
}
/// A u64 as a decimal string (or a small JSON number).
fn int(r: &Value, name: &str) -> Result<u64> {
    let v = field(r, name)?;
    v.as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| v.as_u64())
        .ok_or_else(|| format!("invalid-{name}"))
}
fn bytes<const N: usize>(r: &Value, name: &str) -> Result<[u8; N]> {
    let s = field(r, name)?.as_str().ok_or_else(|| format!("invalid-{name}"))?;
    if s.len() != 2 * N {
        return Err(format!("invalid-{name}"));
    }
    let mut out = [0u8; N];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_| format!("invalid-{name}"))?;
    }
    Ok(out)
}
fn text(r: &Value, name: &str) -> Vec<u8> {
    r.get(name).and_then(Value::as_str).unwrap_or("").as_bytes().to_vec()
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// A step that asks for a compute limit gets the budget instruction first.
fn with_budget(r: &Value, mut ixs: Vec<Instruction>) -> Vec<Instruction> {
    if let Some(units) = r.get("computeUnits").and_then(Value::as_u64) {
        let mut data = vec![2u8];
        data.extend_from_slice(&(units as u32).to_le_bytes());
        ixs.insert(0, Instruction { program_id: COMPUTE_BUDGET, accounts: vec![], data });
    }
    ixs
}

/// Compile the step's instructions into one legacy message, the fee payer first.
fn message(r: &Value, payer: Pubkey, ixs: Vec<Instruction>) -> Result<Value> {
    let ixs = with_budget(r, ixs);
    let blockhash = key(r, "blockhash")?;
    let m = Message::new_with_blockhash(&ixs, Some(&payer), &Hash::new_from_array(blockhash.to_bytes()));
    let signers: Vec<String> =
        m.account_keys[..m.header.num_required_signatures as usize].iter().map(|k| k.to_string()).collect();
    Ok(json!({ "messageBase64": B64.encode(m.serialize()), "signers": signers }))
}

fn bytes_hex(r: &Value, name: &str) -> Result<Vec<u8>> {
    let s = field(r, name)?.as_str().ok_or_else(|| format!("invalid-{name}"))?;
    if s.len() % 2 != 0 {
        return Err(format!("invalid-{name}"));
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| format!("invalid-{name}"))).collect()
}

/// An instruction as JSON, for a host that wraps it inside another program's message (a Squads vault transaction).
fn instruction_json(ix: &Instruction) -> Value {
    json!({
        "programId": ix.program_id.to_string(),
        "accounts": ix.accounts.iter().map(|m| json!({ "pubkey": m.pubkey.to_string(), "isSigner": m.is_signer, "isWritable": m.is_writable })).collect::<Vec<_>>(),
        "data": hex(&ix.data),
    })
}

/// A TREASURY Allowance of unit 0 to a key payee, in SOL.
fn allowance_payload(r: &Value) -> Result<Payload> {
    Ok(Payload::Allowance {
        origin: ROOT_UNIT,
        payee: key(r, "payee")?.to_bytes(),
        payee_kind: HOLDER_KEY,
        mint: Pubkey::default(),
        cap: int(r, "cap")?,
        period: int(r, "period")?,
        expires: int(r, "expires")?,
    })
}

fn upgrade_payload(r: &Value) -> Result<Payload> {
    Ok(Payload::Upgrade { origin: ROOT_UNIT, buffer: key(r, "buffer")?, spill: key(r, "spill")? })
}

/// A raw act of unit 0, the society's hands: { program, accounts: [{ key, flags }], data (hex) }.
fn raw_payload(r: &Value) -> Result<Payload> {
    let mut accounts = Vec::new();
    for a in field(r, "accounts")?.as_array().ok_or("invalid-accounts")? {
        accounts.push((key(a, "key")?, u8::try_from(int(a, "flags")?).map_err(|_| "invalid-flags")?));
    }
    Ok(Payload::Raw { origin: ROOT_UNIT, program: key(r, "program")?, accounts, data: bytes_hex(r, "data")? })
}

fn step(r: &Value) -> Result<Value> {
    let method = field(r, "method")?.as_str().ok_or("invalid-method")?;
    let root = ROOT_UNIT;
    match method {
        // the addresses the ceremony reads and checks
        "ids" => Ok(json!({
            "program": client::PROGRAM_ID.to_string(),
            "upgrade": client::upgrade_pda().to_string(),
            "programdata": client::programdata_pda().to_string(),
            "config": client::config_pda_of(root).to_string(),
            "council": client::council_pda_of(root).to_string(),
            "treasury": client::treasury_pda(root).to_string(),
            "registry": client::registry_pda(root).to_string(),
            "sale": client::sale_pda(root, 0).to_string(),
            "allowanceBreached": client::code(PoleisError::AllowanceBreached),
            "fund": client::fund_pda().to_string(),
            "fundBump": Pubkey::find_program_address(&[b"fund"], &client::PROGRAM_ID).1,
        })),
        // the founding of tree 0: init_config and open_genesis in one transaction
        "found" => {
            let payer = key(r, "payer")?;
            let spec = GenesisSpec {
                charter: bytes(r, "charter")?,
                plan_cap: int(r, "planCap")?,
                plan_period: int(r, "planPeriod")?,
                window: int(r, "window")?,
                rung: int(r, "rung")? as u8,
                reserve: int(r, "reserve")?,
                deposit: int(r, "deposit")?,
                commit_end: int(r, "commitEnd")?,
                reveal_end: int(r, "revealEnd")?,
                founders: keys(r, "founders")?,
                name: text(r, "name"),
                desc: text(r, "desc"),
            };
            let ixs = vec![
                client::init_config_of(root, payer, key(r, "defaultMint")?),
                client::open_genesis_of(root, payer, &spec),
            ];
            message(r, payer, ixs)
        }
        "commitSeat" => {
            let bidder = key(r, "bidder")?;
            let nonce = int(r, "nonce")?;
            let c = client::seat_commitment(int(r, "price")?, nonce, &bytes(r, "salt")?, &bidder);
            message(r, bidder, vec![client::commit_seat_bid(bidder, root, 0, nonce, c)])
        }
        "revealSeat" => {
            let bidder = key(r, "bidder")?;
            let ix = client::reveal_seat_bid(bidder, root, 0, int(r, "nonce")?, int(r, "price")?, bytes(r, "salt")?);
            message(r, bidder, vec![ix])
        }
        "settleSeats" => {
            let crank = key(r, "crank")?;
            message(r, crank, vec![client::settle_seats(crank, root, 0)])
        }
        "claimSeat" => {
            let bidder = key(r, "bidder")?;
            message(r, bidder, vec![client::claim_seat(bidder, root, 0, int(r, "nonce")?, root)])
        }
        "ratify" => {
            let crank = key(r, "crank")?;
            message(r, crank, vec![client::ratify_genesis_of(root, crank)])
        }
        // CODEX Upgrade by the root over the whole tree: the proposal, its hash,
        // and the envelope every PRO voter signs
        "proposeUpgrade" => {
            let proposer = key(r, "proposer")?;
            let payload = upgrade_payload(r)?;
            let manifest: [u8; 32] = bytes(r, "manifest")?;
            let hash = client::proposal_hash(payload.class(), root, &manifest, &payload.hash(), int(r, "nonce")?);
            let mut out = message(r, proposer, vec![client::propose(proposer, hash, manifest, &payload, &[root])])?;
            out["hash"] = json!(hex(&hash));
            out["envelope"] = json!(hex(&client::vote_envelope(&hash, root, VOTE_PRO)));
            Ok(out)
        }
        // PRO votes signed off-chain, landed with the Ed25519 precompile
        "attest" => {
            let crank = key(r, "crank")?;
            let hash: [u8; 32] = bytes(r, "hash")?;
            let votes = field(r, "votes")?.as_array().ok_or("invalid-votes")?;
            let mut signed = Vec::new();
            for v in votes {
                signed.push((key(v, "voter")?, VOTE_PRO, bytes::<64>(v, "signature")?));
            }
            message(r, crank, client::attest_batch(crank, hash, root, &signed).to_vec())
        }
        "seal" => {
            let crank = key(r, "crank")?;
            message(r, crank, vec![client::seal(crank, bytes(r, "hash")?, root)])
        }
        // any program's address by its seeds (hex), by Solana's own derivation
        "pda" => {
            let program = key(r, "program")?;
            let seeds: Vec<Vec<u8>> = field(r, "seeds")?
                .as_array()
                .ok_or("invalid-seeds")?
                .iter()
                .map(|v| bytes_hex(&json!({ "s": v }), "s"))
                .collect::<Result<_>>()?;
            let refs: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
            let (address, bump) = Pubkey::find_program_address(&refs, &program);
            Ok(json!({ "address": address.to_string(), "bump": bump }))
        }
        // another program's instructions, built by the host, compiled by Solana's own compiler
        "compile" => {
            let payer = key(r, "payer")?;
            let mut ixs = Vec::new();
            for i in field(r, "instructions")?.as_array().ok_or("invalid-instructions")? {
                let mut metas = Vec::new();
                for a in field(i, "accounts")?.as_array().ok_or("invalid-accounts")? {
                    let k = key(a, "pubkey")?;
                    let signer = a.get("isSigner").and_then(Value::as_bool).unwrap_or(false);
                    metas.push(if a.get("isWritable").and_then(Value::as_bool).unwrap_or(false) { AccountMeta::new(k, signer) } else { AccountMeta::new_readonly(k, signer) });
                }
                ixs.push(Instruction { program_id: key(i, "programId")?, accounts: metas, data: bytes_hex(i, "data")? });
            }
            message(r, payer, ixs)
        }
        "syncEndowment" => {
            let crank = key(r, "crank")?;
            message(r, crank, vec![client::sync_endowment(root)])
        }
        // TREASURY Allowance: proposed by a member, sealed like any decision, executed into an account
        "proposeAllowance" => {
            let proposer = key(r, "proposer")?;
            let payload = allowance_payload(r)?;
            let manifest: [u8; 32] = bytes(r, "manifest")?;
            let hash = client::proposal_hash(payload.class(), root, &manifest, &payload.hash(), int(r, "nonce")?);
            let mut out = message(r, proposer, vec![client::propose(proposer, hash, manifest, &payload, &[root])])?;
            out["hash"] = json!(hex(&hash));
            out["envelope"] = json!(hex(&client::vote_envelope(&hash, root, VOTE_PRO)));
            out["allowance"] = json!(client::allowance_pda(&hash).to_string());
            Ok(out)
        }
        "executeAllowance" => {
            let crank = key(r, "crank")?;
            let payload = allowance_payload(r)?;
            message(r, crank, vec![client::execute(crank, bytes(r, "hash")?, &[root], &payload)])
        }
        // the pull, as an instruction for the payee's own program to sign (a Squads vault)
        "pullInstruction" => {
            let ix = client::pull(key(r, "puller")?, bytes(r, "hash")?, root, int(r, "amount")?, key(r, "dest")?, None);
            Ok(json!({ "instruction": instruction_json(&ix) }))
        }
        // ACT Raw: the society's hands, proposed by a member and sealed like any decision
        "proposeRaw" => {
            let proposer = key(r, "proposer")?;
            let payload = raw_payload(r)?;
            let manifest: [u8; 32] = bytes(r, "manifest")?;
            let hash = client::proposal_hash(payload.class(), root, &manifest, &payload.hash(), int(r, "nonce")?);
            let mut out = message(r, proposer, vec![client::propose(proposer, hash, manifest, &payload, &[root])])?;
            out["hash"] = json!(hex(&hash));
            out["envelope"] = json!(hex(&client::vote_envelope(&hash, root, VOTE_PRO)));
            Ok(out)
        }
        "executeRaw" => {
            let crank = key(r, "crank")?;
            let payload = raw_payload(r)?;
            message(r, crank, vec![client::execute(crank, bytes(r, "hash")?, &[root], &payload)])
        }
        // permissionless: unit 0's token account for a mint, its treasury the authority
        "createVault" => {
            let payer = key(r, "payer")?;
            message(r, payer, vec![client::create_vault(payer, root, key(r, "mint")?, key(r, "tokenProgram")?)])
        }
        "executeUpgrade" => {
            let crank = key(r, "crank")?;
            let payload = upgrade_payload(r)?;
            message(r, crank, vec![client::execute(crank, bytes(r, "hash")?, &[root], &payload)])
        }
        _ => Err("unknown-method".into()),
    }
}

fn main() {
    let mut input = Vec::new();
    let reply = match std::io::stdin().lock().take((MAX_INPUT + 1) as u64).read_to_end(&mut input) {
        Ok(n) if n <= MAX_INPUT => match serde_json::from_slice::<Value>(&input) {
            Ok(r) => match step(&r) {
                Ok(mut v) => {
                    v["ok"] = json!(true);
                    v
                }
                Err(e) => json!({ "ok": false, "error": e }),
            },
            Err(_) => json!({ "ok": false, "error": "invalid-json" }),
        },
        _ => json!({ "ok": false, "error": "input-unavailable" }),
    };
    let mut out = serde_json::to_vec(&reply).unwrap_or_else(|_| br#"{"ok":false,"error":"encoding-failed"}"#.to_vec());
    out.push(b'\n');
    let _ = std::io::stdout().lock().write_all(&out);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(v: Value) -> Value {
        step(&v).unwrap_or_else(|e| json!({ "error": e }))
    }
    fn b58(b: [u8; 32]) -> String {
        Pubkey::new_from_array(b).to_string()
    }

    #[test]
    fn the_founding_is_one_message_and_one_signer() {
        let payer = Pubkey::new_from_array([1; 32]);
        let founders: Vec<String> = (0..12).map(|i| b58([10 + (i / 4) as u8; 32])).collect();
        let r = ask(json!({
            "method": "found", "payer": payer.to_string(), "blockhash": b58([2; 32]),
            "defaultMint": Pubkey::default().to_string(), "charter": hex(&[7; 32]), "planCap": "1000", "planPeriod": "6480000",
            "window": "100", "rung": 1, "reserve": "10", "deposit": "50", "commitEnd": "200", "revealEnd": "300",
            "founders": founders, "name": "rehearsal", "desc": ""
        }));
        assert_eq!(r["signers"], json!([payer.to_string()]));
        let m = B64.decode(r["messageBase64"].as_str().unwrap()).unwrap();
        // the legacy header: one signature, the payer; the founding's two
        // instructions share it, so nobody else signs
        assert_eq!(m[0], 1);
    }

    #[test]
    fn the_upgrade_proposal_hash_is_the_one_its_votes_sign() {
        let req = json!({
            "method": "proposeUpgrade", "proposer": b58([3; 32]), "blockhash": b58([2; 32]),
            "buffer": b58([4; 32]), "spill": b58([5; 32]), "manifest": hex(&[0; 32]), "nonce": "1"
        });
        let r = ask(req.clone());
        let hash: [u8; 32] = bytes(&r, "hash").unwrap();
        assert_eq!(r["envelope"], json!(hex(&client::vote_envelope(&hash, ROOT_UNIT, VOTE_PRO))));
        assert_eq!(ask(req)["hash"], r["hash"]);
    }

    #[test]
    fn the_allowance_pull_is_signed_by_its_payee_and_lands_at_it() {
        let vault = b58([8; 32]);
        let r = ask(json!({ "method": "pullInstruction", "puller": vault, "dest": vault, "hash": hex(&[9; 32]), "amount": "300000000" }));
        let accts = r["instruction"]["accounts"].as_array().unwrap();
        assert_eq!(accts[0]["pubkey"], json!(vault));
        assert_eq!(accts[0]["isSigner"], json!(true));
        assert_eq!(accts[4]["pubkey"], json!(vault));
        assert_eq!(accts[4]["isWritable"], json!(true));
        assert_eq!(r["instruction"]["programId"], json!(client::PROGRAM_ID.to_string()));
    }

    #[test]
    fn a_pda_is_solanas_own_derivation() {
        let program = Pubkey::new_from_array([7; 32]);
        let r = ask(json!({ "method": "pda", "program": program.to_string(), "seeds": [hex(b"multisig"), hex(b"program_config")] }));
        let (a, b) = Pubkey::find_program_address(&[b"multisig", b"program_config"], &program);
        assert_eq!(r["address"], json!(a.to_string()));
        assert_eq!(r["bump"], json!(b));
    }

    #[test]
    fn a_bad_key_is_refused_by_name() {
        assert_eq!(ask(json!({ "method": "ratify", "crank": "nope", "blockhash": "x" }))["error"], json!("invalid-crank"));
        assert_eq!(ask(json!({ "method": "what" }))["error"], json!("unknown-method"));
    }

    #[test]
    fn a_compute_limit_rides_first() {
        let ixs = with_budget(&json!({ "computeUnits": 400000 }), vec![client::ratify_genesis_of(ROOT_UNIT, Pubkey::new_from_array([6; 32]))]);
        assert_eq!(ixs.len(), 2);
        assert_eq!(ixs[0].program_id, COMPUTE_BUDGET);
        assert_eq!(ixs[0].data, [2, 0x80, 0x1a, 0x06, 0x00]);
        assert!(with_budget(&json!({}), vec![]).is_empty());
    }
}
