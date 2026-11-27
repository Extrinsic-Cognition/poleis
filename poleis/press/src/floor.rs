//! THE FLOOR IS THE MINTED LYSSAJUDRONS (THE WHOLE TREE DECIDES, Nov 22 2026; his 09:50:51 UTC "you gotta fucking stop treating the
//! keystones as what make the decisions porcoddio. the fucking entire tree perfect bicameralism gvs posiverse neghiverse molecularity", and
//! his Nov 19 line "each lyssajudron minted is a hex there"). Root 0's floor is never a list of his keys: it is the hexagons, each a
//! Lyssajudron minted on Poleis's curve by its own buyer, with a permit the mint door's issuer signed for that buyer, inside the curve's
//! window, at the curve's price. The GVS's rule (2.2 and 3.2): every impacted unit passes by its two chambers, a hexagon seat is one an
//! identity a unit, and the hexagon seats are the commons. His own Lyssajudron is one hexagon like anyone's.
//!
//! `press ratify` sends what anyone may send, the deployer its payer: `ratify_nft_genesis`, once the curve's window has closed or every
//! seat is minted, and at least one seat is (Poleis's own order); before, it says plainly what it waits for.
//!
//! `press vote --act <n> --owner <address> [--propose]` writes one seat owner's own transactions of one act, as the Keystone page's prepared
//! road reads a set: its proposal first when it proposes (`propose_nft`, its seat proved by its token), then its PRO (`attest_nft`) for each
//! seat it holds; unsigned, one signer (the owner, its fee payer), the blockhash blank, its seat, mint and token accounts read off the chain.
//! A vote of the floor is never a ballot the press lands (`press crank` sends everything anyone may send, and never this).


use crate::acts::{self, Act};
use crate::chain::Sitting;
use crate::params::Params;
use crate::pool::{NOT_MAINNET, NOT_RUN};
use crate::rpc::Cluster;
use crate::wire;
use crate::Res;
use poleis_client::{self as client, nft, ROOT_UNIT, VOTE_PRO};
use poleis_treasury::{HOLDER_NFT, MODE_COUNCIL, MODE_GENESIS};
use serde_json::{json, Value};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr as _;

/// What `press ratify` sends next, read off root 0's mode, its curve and its floor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RatifyDue {
    Ratify,
    Wait(String),
    Done,
}

/// THE RATIFICATION'S ORDER, Poleis's own (`ratify_nft_genesis`): root 0 is ratified once its curve's window has closed or every seat is
/// minted, with a seat minted and standing on its floor. Permissionless. `minted` and `capacity` are the curve's, `closes` its window's end,
/// `seats` root 0's floor as its registry counts it.
pub fn ratify_due(mode: u8, minted: u64, capacity: u64, closes: u64, slot: u64, seats: u64) -> RatifyDue {
    if mode == MODE_COUNCIL {
        return RatifyDue::Done;
    }
    if mode != MODE_GENESIS {
        return RatifyDue::Wait(format!("root 0 is in mode {mode}, not in genesis: nothing to ratify"));
    }
    let open = slot < closes && minted < capacity;
    if minted == 0 || seats == 0 {
        if open {
            return RatifyDue::Wait(format!("no seat of root 0's floor is minted yet, and its curve's window runs to slot {closes} (now {slot}): each seat is minted by its own buyer, with the issuer's permit for that buyer"));
        }
        return RatifyDue::Wait(format!("the curve's window closed at slot {closes} with no seat of root 0's floor standing: root 0 can never be ratified. The deployer still holds the code: root 0 is founded again under fresh program ids (his call)"));
    }
    if open {
        return RatifyDue::Wait(format!("the curve's window runs to slot {closes} (now {slot}), {minted} of its {capacity} seats minted: root 0 is ratified once the window has closed or every seat is minted"));
    }
    RatifyDue::Ratify
}

/// One seat of root 0's floor as the chain holds it: its receipt's address, and its owner's proof (the seat's mint and the owner's token
/// account holding it).
#[derive(Clone, Copy, Debug)]
pub struct Seat {
    pub seat: Pubkey,
    pub proof: nft::OwnerProof,
}

/// A seat owner's proof that names keys nobody holds (sha256 of words): the params' check of a proposal's size uses it.
pub fn placeholder_owner() -> nft::OwnerProof {
    let k = crate::params::placeholder_address;
    nft::OwnerProof { owner: k("a seat owner of root 0's floor"), mint: k("its seat's mint"), token: k("its token account"), unit: ROOT_UNIT }
}

/// An act proposed by a seat owner: `propose_nft`, the seat proved by `proof`, the act's own class, manifest, payload and impacted set.
pub fn proposal(act: &Act, proof: &nft::OwnerProof) -> Res<Instruction> {
    proof.propose_raw(ROOT_UNIT, act.hash, act.class, act.manifest, &act.payload, &act.impacted).map_err(|e| format!("act {}: its proposal by a seat owner: {e:?}", act.n))
}

/// The bytes of an act's proposal by a seat owner on the wire, with its compute budget when the params name a priority fee.
pub fn proposal_bytes(p: &Params, act: &Act, proof: &nft::OwnerProof) -> Res<usize> {
    let ixs = [crate::chain::budget_of(p, 80_000)?, vec![proposal(act, proof)?]].concat();
    let (m, signers) = wire::message(&ixs, &proof.owner, &[0; 32]);
    Ok(1 + 64 * signers.len() + m.len())
}

/// THE FLOOR'S OWN VOTE: one seat owner's set of one act, as the Keystone page's prepared road reads one: its proposal first when it
/// proposes (`propose_nft`, by its first seat), then its PRO (`attest_nft`) for each seat it holds; each transaction unsigned, one signer
/// (the owner, its fee payer), its blockhash blank.
pub fn vote_set(p: &Params, act: &Act, owner: &Pubkey, seats: &[Seat], propose: bool, genesis: &str, labels: &BTreeMap<Pubkey, String>) -> Res<Value> {
    if seats.is_empty() {
        return Err(format!("{owner} holds no seat of root 0's floor: a vote of the floor is a seat owner's (a seat is minted on the curve, with a permit of the mint door for its buyer)"));
    }
    if let Some(s) = seats.iter().find(|s| s.proof.owner != *owner || s.proof.unit != ROOT_UNIT) {
        return Err(format!("the seat {} is not proved for {owner} on root 0's floor", s.seat));
    }
    let placeholders = p.placeholders();
    let budget = |units: u32| crate::chain::budget_of(p, units);
    let mut transactions = Vec::new();
    if propose {
        let does = [act.does.clone(), vec![format!("the proposer is a member of root 0: its seat {} proved by its token, the proposal's rent and fee paid by its owner", seats[0].seat)]].concat();
        transactions.push(acts::entry(1, &format!("propose act {} ({})", act.n, act.name), &does, &[budget(80_000)?, vec![proposal(act, &seats[0].proof)?]].concat(), owner, labels, &placeholders)?);
    }
    for s in seats {
        let ix = s.proof.attest(act.hash, VOTE_PRO).map_err(|e| format!("act {}: the PRO of the seat {}: {e:?}", act.n, s.seat))?;
        let n = transactions.len() + 1;
        let does = [format!("the PRO of the seat {} in root 0's floor on act {}: one hexagon, one vote, its owner's own transaction; sign and send once the act is proposed, inside its vote's window", s.seat, act.n)];
        transactions.push(acts::entry(n, &format!("vote PRO on act {} by the seat {}", act.n, s.seat), &does, &[budget(30_000)?, vec![ix]].concat(), owner, labels, &placeholders)?);
    }
    Ok(json!({
        "what": format!("THE LAUNCH PRESS, act {}: {}. THE WHOLE TREE DECIDES: root 0's floor votes it by each seat owner's own transaction; here one owner's{}, each signed by that owner alone. Two of unit 1's three agents AND more than half of the floor's seats pass it; press crank sends everything else", act.n, act.name, if propose { ", its proposal first" } else { "" }),
        "cluster": p.cluster.name(),
        "genesisHash": genesis,
        "signable": placeholders.is_empty(),
        "placeholders": placeholders,
        "emitted": crate::chain::now_iso(),
        "signer": { "address": owner.to_string(), "role": format!("a seat owner of root 0's floor, {} seat{}: one hexagon a seat, like anyone's", seats.len(), if seats.len() == 1 { "" } else { "s" }) },
        "act": { "n": act.n, "name": act.name, "class": act.class, "hash": wire::hex(&act.hash), "payloadSha256": wire::hex(&act.payload_sha256), "nonce": act.nonce, "impacted": act.impacted },
        "seats": seats.iter().map(|s| json!({ "seat": s.seat.to_string(), "mint": s.proof.mint.to_string(), "token": s.proof.token.to_string() })).collect::<Vec<_>>(),
        "order": if propose { "1 the proposal; then each PRO once the proposal has landed, inside the vote's window" } else { "each PRO, once the act is proposed, inside its vote's window" },
        "transactions": transactions,
    }))
}

/// A Token-2022 account's base (its mint, its owner, its amount) holds exactly one of `mint` for `owner`.
fn holds(data: &[u8], mint: &Pubkey, owner: &Pubkey) -> bool {
    data.len() >= 72 && data[..32] == mint.to_bytes() && data[32..64] == owner.to_bytes() && data[64..72] == 1u64.to_le_bytes()
}

impl Sitting {
    /// PRESS RATIFY: root 0 ratified into its council once its curve's window has closed or every seat is minted, a seat standing; the
    /// deployer pays. Before, it says what it waits for and sends nothing.
    pub fn ratify(&mut self) -> Res<()> {
        self.p.sendable()?;
        self.root_is_ours()?;
        let mut keys = None;
        for _ in 0..2 {
            let mode = client::ConfigView::parse(&self.account(&self.at.config)?.ok_or("root 0's config is not on the cluster")?.data).ok_or("root 0's config does not parse")?.mode;
            let curve = nft::MintCurve::decode(&self.account(&self.at.curve)?.ok_or("root 0's curve is not on the cluster")?.data).map_err(|_| "root 0's curve does not parse")?;
            let seats = client::RegistryView::parse(&self.account(&self.at.registry)?.ok_or("root 0's registry is not on the cluster")?.data).ok_or("root 0's registry does not parse")?.total_seats;
            match ratify_due(mode, curve.minted_count, curve.capacity, curve.closes_slot, self.rpc.slot()?, seats) {
                RatifyDue::Done => {
                    println!("RATIFIED: root 0 is a council, its authority the PDA nobody holds; its floor {seats} seats, the minted Lyssajudrons ({} of {} minted)", curve.minted_count, curve.capacity);
                    return Ok(());
                }
                RatifyDue::Wait(w) => {
                    println!("press ratify waits: {w}");
                    return Ok(());
                }
                RatifyDue::Ratify => {}
            }
            if keys.is_none() {
                keys = Some(self.keys.load(&["deployer"], "press ratify")?);
            }
            let k = &keys.as_ref().unwrap()[0];
            let budget = crate::chain::budget_of(&self.p, 30_000)?;
            let l = self.send("ratify", "root 0 ratified into its council (ratify_nft_genesis, by anyone)", [budget, vec![nft::ratify_genesis(self.deployer, ROOT_UNIT)]].concat(), &[k])?;
            self.rec.v["ratify"] = json!({ "signature": l.signature, "slot": l.slot, "minted": curve.minted_count, "capacity": curve.capacity, "closes": curve.closes_slot });
            self.rec.save()?;
        }
        Ok(())
    }

    /// The seats of root 0's floor `owner` holds now, read off the chain: each NFT row of root 0's registry, its receipt, its mint, and the
    /// owner's token account holding it (the first buyer's own, or, after a resale, the account the owner holds it in).
    pub fn seats_of(&self, owner: &Pubkey) -> Res<Vec<Seat>> {
        let rv = client::RegistryView::parse(&self.account(&self.at.registry)?.ok_or("root 0's registry is not on the cluster")?.data).ok_or("root 0's registry does not parse")?;
        let mut theirs: Option<Vec<(Pubkey, Vec<u8>)>> = None;
        let mut out = Vec::new();
        for h in rv.holders.iter().filter(|h| h.kind == HOLDER_NFT && h.seats > 0) {
            let seat = Pubkey::new_from_array(h.holder);
            let Some(a) = self.account(&seat)? else { continue };
            let Ok(receipt) = nft::NftSeat::decode(&a.data) else { continue };
            let mint = Pubkey::new_from_array(receipt.mint);
            if a.owner != client::PROGRAM_ID.to_string() || receipt.unit != ROOT_UNIT || nft::seat_pda(&mint) != seat {
                continue;
            }
            let first = nft::initial_token_pda(&mint, owner);
            let token = if self.account(&first)?.is_some_and(|t| t.owner == client::TOKEN_2022_PROGRAM_ID.to_string() && holds(&t.data, &mint, owner)) {
                Some(first)
            } else {
                if theirs.is_none() {
                    let all = self.rpc.token_accounts(&owner.to_string(), &client::TOKEN_2022_PROGRAM_ID.to_string())?;
                    theirs = Some(all.into_iter().filter_map(|(k, d)| Pubkey::from_str(&k).ok().map(|k| (k, d))).collect());
                }
                theirs.as_ref().unwrap().iter().find(|(_, d)| holds(d, &mint, owner)).map(|(k, _)| *k)
            };
            if let Some(token) = token {
                out.push(Seat { seat, proof: nft::OwnerProof { owner: *owner, mint, token, unit: ROOT_UNIT } });
            }
        }
        Ok(out)
    }

    /// PRESS VOTE: one seat owner's own transactions of act `n` (its proposal first when `propose`), written for its owner to sign: the
    /// file's path.
    pub fn vote(&mut self, n: usize, owner: &Pubkey, propose: bool) -> Res<PathBuf> {
        if self.root_is_ours()? != MODE_COUNCIL {
            return Err("root 0 is still in genesis: nothing is proposed or voted before it is ratified (press ratify)".into());
        }
        let mut acts = acts::acts(&self.p, &self.ids, &self.at, &self.deployer)?;
        acts.extend(crate::engine::later(&self.p, &self.ids, &self.at, &self.deployer)?); // THE STAKE EARNS: the ninth
        if n == crate::engine::ENGINE_ACT && acts.len() < n {
            if let Some(w) = crate::engine::later_said(&self.p) {
                return Err(format!("act {n} ({}): {w}: no vote of it is written", crate::engine::ACT_NAME));
            }
        }
        let act = acts.iter().find(|a| a.n == n).ok_or_else(|| format!("--act {n}: the acts are 1 to {}", acts.len()))?;
        if let Some(program) = act.needs {
            if !self.account(&program)?.is_some_and(|a| a.executable) {
                if self.p.cluster == Cluster::Mainnet {
                    return Err(format!("act {n} ({}): {NOT_MAINNET}", act.name));
                }
                return Err(format!("act {n} ({}): {NOT_RUN}: no vote of it is written here", act.name));
            }
        }
        let proposed = self.account(&client::proposal_pda(&act.hash))?.is_some();
        if propose && proposed {
            return Err(format!("act {n} is proposed already: its proposal stands, and the chain refuses a second; write the owner's PRO without --propose"));
        }
        let seats = self.seats_of(owner)?;
        if !proposed && !propose {
            eprintln!("  act {n} is not proposed yet: this PRO lands once a seat owner's proposal of it has (press vote --act {n} --owner <a seat owner> --propose)");
        }
        let mut labels = acts::labels(&self.ids, &self.at, &self.p.agents()?);
        labels.insert(*owner, "a seat owner of root 0's floor: it signs these, and nothing else".into());
        for s in &seats {
            labels.insert(s.seat, "its seat's receipt (a hexagon of root 0's floor)".into());
            labels.insert(s.proof.mint, "its seat's mint (a Lyssajudron)".into());
            labels.insert(s.proof.token, "its token account, holding the seat".into());
        }
        let genesis = self.rec.get("genesisHash").as_str().unwrap_or_default().to_string();
        let set = vote_set(&self.p, act, owner, &seats, propose, &genesis, &labels)?;
        let dir = self.dir.join("votes");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join(format!("{n}-{owner}.json"));
        std::fs::write(&path, serde_json::to_string_pretty(&set).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
        let txs = set["transactions"].as_array().map_or(0, Vec::len);
        eprintln!(
            "  act {n} ({}): {txs} transaction{} for {owner} to sign, its {} seat{} of root 0's floor{}",
            act.name,
            if txs == 1 { "" } else { "s" },
            seats.len(),
            if seats.len() == 1 { "" } else { "s" },
            if propose { ", its proposal first" } else { "" }
        );
        Ok(path)
    }
}
