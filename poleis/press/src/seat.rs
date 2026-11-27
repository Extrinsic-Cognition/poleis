//! THE DOOR SIGNS THE SEAT (Nov 22 2026, lane/the-door-signs-the-seat; his 15:20:34 UTC "Yes, build it" to "Next: the mint door signs each finished
//! Lyssajudron's permit, so the floor can fill?", over THE WHOLE TREE DECIDES). Root 0's floor is whoever mints a seat on Poleis's curve, and Poleis
//! seats a buyer only by a permit the curve's issuer signed for that buyer; the mint door (embertide-app/ui/intro/dyson-society/worker/space-router-seat.mjs)
//! is that issuer, one seat a finished Lyssajudron's deposit. Two steps of the press stand beside it:
//!
//! - `press issuer --dir D [--params P]`: the issuer's key, made by his hand beside the sitting's other keys (D/keys/issuer.json, mode 600 in a folder
//!   of mode 700 outside every repository, never over a key that stands), its address said for the params' `floor.issuer` and kept beside it
//!   (D/keys/issuer.address, no key in it); with the params and the program ids `press keys` made, the door's Worker vars. `--secret` is HIS press,
//!   never a seat's: after his finger, the key in the Worker secret's shape, into a pipe to `wrangler secret put POLEIS_ISSUER_KEY`, never onto a
//!   screen.
//! - `press miner --dir D [--params P]` (THE SWARM NAMES ITS MINER, Nov 23 2026, his 18:22:41 UTC "Yes, start it"): the door's crank key made the same
//!   way (D/keys/miner.json, its address in D/keys/miner.address), said for the params' `warm.miner`, which the press's sixth act names the $WARM
//!   mine's miner; `--secret` is HIS press, into a pipe to `wrangler secret put SPACE_ROUTER_MINER_KEY`, never a seat's and never onto a screen.
//! - `press crediter --dir D [--params P]` (THE PRESS IS READY FOR MAINNET, Nov 24 2026): the market board's crediter key made the same way
//!   (D/keys/crediter.json, its address in D/keys/crediter.address), said for the params' `market.crediter`, which act 2 names the board's
//!   crediter (the key that credits play $WARM to a board's balances). No door reads it yet, so it has no `--secret`: its road to whoever
//!   credits is owed. A sitting whose params name another key than one of these three steps made is refused (`kept_keys_held`).
//! - `press seat --permit F --buyer A [--max L]`, a sitting's step (root 0's curve and the slot read off the chain): the door's answer F held to the
//!   curve (Poleis's program, this chain, root 0, its curve, the face), to the buyer named, to the issuer's signature and to the slot, then written as
//!   one transaction for the buyer alone to sign, its fee payer, its blockhash blank, in the Keystone page's prepared format: the precompile's
//!   instruction (the issuer's signature over the permit, checked by Solana itself) and `mint_nft`, at the curve's price now unless `--max` says more.

use crate::acts;
use crate::chain::{self, KeySource, Sitting};
use crate::params::Params;
use crate::wire::{self, Key};
use crate::Res;
use ed25519_dalek::{Signature, VerifyingKey}; // classical: Solana's precompile verifies only Ed25519
use poleis_client::nft::{self, Metadata, MintCurve, PERMIT_LEN};
use poleis_client::ROOT_UNIT;
use poleis_treasury::completion_permit;
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

/// The units a seat's mint asks when the params name a priority fee: a mint ran 57,331 to 73,832 in the rehearsals.
pub const MINT_UNITS: u32 = 120_000;

/// A sitting's key folder, D/keys: outside every git repository, his user's alone (700, and D too), as `press keys` makes it.
pub fn keys_folder(dir: &Path) -> Res<PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;
    let keys = dir.join("keys");
    // a key's folder is outside every repository: git is asked where the folder would stand
    let mut probe = keys.clone();
    while !probe.exists() {
        if !probe.pop() {
            break;
        }
    }
    let inside = std::process::Command::new("git").arg("-C").arg(&probe).args(["rev-parse", "--is-inside-work-tree"]).output().ok().is_some_and(|o| String::from_utf8_lossy(&o.stdout).trim() == "true");
    if inside {
        return Err(format!("{} is inside a git repository: his keys live outside every repository", keys.display()));
    }
    std::fs::create_dir_all(&keys).map_err(|e| format!("{}: {e}", keys.display()))?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    std::fs::set_permissions(&keys, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    Ok(keys)
}

/// THE ISSUER'S KEY (`press issuer`): made by `make` (his press: a fresh key; a test: a fixed one) into D/keys/issuer.json, mode 600, unless one stands;
/// its address kept beside it (D/keys/issuer.address, read later without opening the key). The address, and whether it was made now.
pub fn issuer_key(dir: &Path, make: impl FnOnce() -> Res<Key>) -> Res<(Pubkey, bool)> {
    door_key(dir, "issuer", make)
}

/// THE MINER'S KEY (`press miner`; THE SWARM NAMES ITS MINER, Nov 23 2026): the mint door's crank key, made as the issuer's is into D/keys/miner.json
/// beside the sitting's other keys, never over one, its address kept beside it (D/keys/miner.address) for the params' `warm.miner`, which the sixth act
/// names the mine's miner. The address, and whether it was made now.
pub fn miner_key(dir: &Path, make: impl FnOnce() -> Res<Key>) -> Res<(Pubkey, bool)> {
    door_key(dir, "miner", make)
}

/// THE CREDITER'S KEY (`press crediter`; THE PRESS IS READY FOR MAINNET, Nov 24 2026): the market board's crediter, made as the issuer's and
/// the miner's are into D/keys/crediter.json beside the sitting's other keys, never over one, its address kept beside it (D/keys/crediter.address)
/// for the params' `market.crediter`, which act 2 names the board's crediter. The address, and whether it was made now.
pub fn crediter_key(dir: &Path, make: impl FnOnce() -> Res<Key>) -> Res<(Pubkey, bool)> {
    door_key(dir, "crediter", make)
}

/// THE EPOCH TURNS THE CRANKS (Nov 25 2026): the engine's key (`press engine-key`), her pool's staker once the ninth act names it, and the pourer's
/// (`press pourer`), the light's crank, each made as the miner's is into D/keys/<name>.json beside the sitting's other keys, never over one, its
/// address kept beside it; `press epoch` reads them from there and nowhere else. The address, and whether it was made now.
pub fn engine_key(dir: &Path, make: impl FnOnce() -> Res<Key>) -> Res<(Pubkey, bool)> {
    door_key(dir, "engine", make)
}
pub fn pourer_key(dir: &Path, make: impl FnOnce() -> Res<Key>) -> Res<(Pubkey, bool)> {
    door_key(dir, "pourer", make)
}

/// A key a step of the press makes beside the sitting's other keys (`name`: issuer, miner, crediter, engine or pourer), and the address kept beside it.
fn door_key(dir: &Path, name: &str, make: impl FnOnce() -> Res<Key>) -> Res<(Pubkey, bool)> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let keys = keys_folder(dir)?;
    let (path, kept) = (keys.join(format!("{name}.json")), keys.join(format!("{name}.address")));
    if path.exists() {
        let text = std::fs::read_to_string(&kept).map_err(|_| format!("{}: a key stands there and its address beside it ({}) is gone; the press never writes over a key", path.display(), kept.display()))?;
        return Pubkey::from_str(text.trim()).map(|a| (a, false)).map_err(|_| format!("{}: not an address", kept.display()));
    }
    let k = make()?;
    let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut text = k.cli_text();
    let wrote = std::io::Write::write_all(&mut f, text.as_bytes()).and_then(|_| f.sync_all());
    text.clear();
    wrote.map_err(|e| format!("{}: {e}", path.display()))?;
    std::fs::write(&kept, format!("{}\n", k.address)).map_err(|e| format!("{}: {e}", kept.display()))?;
    Ok((k.address, true))
}

/// THE DOOR'S SECRET (`press issuer --secret`, his press alone): after his finger and the folder's and the file's modes, the issuer's key in the Worker
/// secret's shape (POLEIS_ISSUER_KEY: the seed and the address, 64 bytes in base58). main.rs writes it into a pipe and never onto a screen.
pub fn issuer_secret(dir: &Path, touchid: &Path) -> Res<String> {
    let k = KeySource::Files { dir: dir.join("keys"), touchid: touchid.to_path_buf() }.load(&["issuer"], "the mint door's permit key, for its Worker secret")?;
    Ok(k[0].secret_base58())
}

/// THE CRANK'S SECRET (`press miner --secret`, his press alone): after his finger and the modes, the miner's key in the Worker secret's shape
/// (SPACE_ROUTER_MINER_KEY: the seed and the address, 64 bytes in base58, as worker/space-router-deposit.mjs reads it), for a pipe, never a screen.
pub fn miner_secret(dir: &Path, touchid: &Path) -> Res<String> {
    let k = KeySource::Files { dir: dir.join("keys"), touchid: touchid.to_path_buf() }.load(&["miner"], "the mint door's crank key (the $WARM mine's miner), for its Worker secret")?;
    Ok(k[0].secret_base58())
}

/// The address `press issuer` or `press miner` kept beside its key in D/keys (`name`.address), read without opening the key: none where it made none.
pub fn kept_address(dir: &Path, name: &str) -> Res<Option<Pubkey>> {
    let kept = dir.join("keys").join(format!("{name}.address"));
    match std::fs::read_to_string(&kept) {
        Ok(text) => Pubkey::from_str(text.trim()).map(Some).map_err(|_| format!("{}: not an address", kept.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", kept.display())),
    }
}

/// The keys a step of the press makes beside his keys, each with the param that names it and what that param does on the chain.
pub const KEPT: [(&str, &str, &str); 3] = [
    ("issuer", "floor.issuer", "root 0's curve would name a permit issuer the door holds no key of: no buyer could mint a seat, and the curve's issuer is fixed at the founding"),
    ("miner", "warm.miner", "the sixth act would name a miner the door holds no key of: the door's crank reads the mine naming another key and sends nothing"),
    ("crediter", "market.crediter", "the second act would name a crediter nobody of this sitting holds a key of: the board's balances could never be credited"),
];

/// A param held to the key a step of the press made beside his keys (`name`, KEPT), where it made one: refused when the params name another.
/// A placeholder, or no key made in this sitting's folder, leaves the params' word standing.
fn held_to_kept(dir: &Path, p: &Params, name: &str) -> Res<()> {
    let (_, path, why) = KEPT.iter().find(|(n, ..)| *n == name).ok_or_else(|| format!("{name}: no key a step of the press makes"))?;
    match (kept_address(dir, name)?, p.is_placeholder(path)) {
        (Some(kept), false) if p.key(path)? != kept => Err(format!(
            "the params' {path} {} is not the key press {name} made beside his keys ({kept}, {}): {why}; write {kept} in the params",
            p.at(path),
            dir.join(format!("keys/{name}.address")).display()
        )),
        _ => Ok(()),
    }
}

/// THE SWARM NAMES ITS MINER: the miner the sixth act names is the key `press miner` made beside his keys, where it made one.
pub fn miner_is_the_kept(dir: &Path, p: &Params) -> Res<()> {
    held_to_kept(dir, p, "miner")
}

/// THE PRESS IS READY FOR MAINNET: the permit issuer root 0's curve names at the founding is the key `press issuer` made beside his keys, where it
/// made one (THE SWARM NAMES ITS MINER said it was not held so).
pub fn issuer_is_the_kept(dir: &Path, p: &Params) -> Res<()> {
    held_to_kept(dir, p, "issuer")
}

/// THE PRESS IS READY FOR MAINNET: the board's crediter the second act names is the key `press crediter` made beside his keys, where it made one.
pub fn crediter_is_the_kept(dir: &Path, p: &Params) -> Res<()> {
    held_to_kept(dir, p, "crediter")
}

/// Every param a step of the press made a key for, held to that key (KEPT): the deploy, the founding and the acts are refused when one differs,
/// and every one that differs is named.
pub fn kept_keys_held(dir: &Path, p: &Params) -> Res<()> {
    let bad: Vec<String> = KEPT.iter().filter_map(|(name, ..)| held_to_kept(dir, p, name).err()).collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(bad.join("; "))
    }
}

/// THE DOOR'S VARS (wrangler.optimism-fit.jsonc "vars", at his deploy once root 0 is founded): Poleis's program id, the chain's genesis hash, root 0's
/// curve under that program, the curve's first price, its step and its seats (the params' rung). A value that is not his yet says so.
// (the miner's address is no var: the door reads it off SPACE_ROUTER_MINER_KEY, and the chain holds the one the sixth act named)
pub fn door_vars(p: &Params, poleis: &Pubkey, genesis: &str) -> Value {
    let curve = Pubkey::find_program_address(&[poleis_treasury::nft_seat_state::CURVE_SEED, &ROOT_UNIT.to_le_bytes()], poleis).0;
    let number = |path: &str| if p.is_placeholder(path) { p.at(path).as_str().unwrap_or_default().to_string() } else { p.u64(path).map(|n| n.to_string()).unwrap_or_else(|e| e) };
    let capacity = p.u64("genesis.rung").ok().and_then(chain::rung_hexagons).map(|c| c.to_string()).unwrap_or_else(|| number("genesis.rung"));
    json!({ "POLEIS_PROGRAM": poleis.to_string(), "POLEIS_GENESIS": genesis, "POLEIS_CURVE": curve.to_string(), "POLEIS_START": number("floor.startLamports"), "POLEIS_STEP": number("floor.stepLamports"), "POLEIS_CAPACITY": capacity })
}

/// THE MINE IS LIVE (Nov 23 2026): the door's vars of the $WARM mine, beside Poleis's, at the same deploy. SPACE_ROUTER_MINE: the mine's address under
/// the sitting's $WARM program (the door's WASM must speak the same one, or the door stays as it was); SPACE_ROUTER_TREASURY: root 0's own vault of the
/// dollar under the sitting's Poleis, where the mine sends every deposit; SPACE_ROUTER_OPENED_AT: the mine's opening, so the router's ledger halves when
/// the chain does (said once the mine stands). The miner's key is the door's secret (SPACE_ROUTER_MINER_KEY), its address named by a sealed act of the polis.
pub fn mine_vars(p: &Params, poleis: &Pubkey, warm: &Pubkey, opening_ms: Option<u64>) -> Value {
    let treasury = match p.key("warm.dollar") {
        Ok(dollar) => Pubkey::find_program_address(&[b"vault", &ROOT_UNIT.to_le_bytes(), dollar.as_ref()], poleis).0.to_string(),
        Err(_) => p.at("warm.dollar").as_str().unwrap_or("the params' warm.dollar").to_string(),
    };
    let opened = match opening_ms {
        Some(ms) => {
            let ((y, mo, d), s) = (warm_mine::law::civil_of(ms / warm_mine::law::DAY_MS), ms % warm_mine::law::DAY_MS / 1_000);
            format!("{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z", s / 3_600, s / 60 % 60, s % 60, ms % 1_000)
        }
        None => "the mine's opening (the record's warm.opening, once press warm has opened it)".to_string(),
    };
    json!({ "SPACE_ROUTER_MINE": Pubkey::find_program_address(&[warm_mine::state::SEED_MINE], warm).0.to_string(), "SPACE_ROUTER_TREASURY": treasury, "SPACE_ROUTER_OPENED_AT": opened })
}

/// THE FLASK TAKES ITS SEAT (Nov 22 2026): the mint door's seat law as `press build` recorded it (its `builds.seat`), said beside the door's vars: the
/// file a lane after his sitting lands as embertide-app/ui/intro/dyson-society/worker/poleis-seat.wasm with them, its Poleis the sitting's.
pub fn door_law(seat: &Value) -> String {
    match (seat["path"].as_str(), seat["sha256"].as_str()) {
        (Some(path), Some(sha)) => format!("  the door's seat law (worker/poleis-seat.wasm, landed with the vars): {path} ({} bytes, sha256 {sha})", seat["bytes"]),
        _ => "  the door's seat law: press build makes it from the sitting's copy (worker/poleis-seat.wasm, landed with the vars)".to_string(),
    }
}

/// The price of the curve's next seat: its start and a step for each seat minted before it.
fn price_now(curve: &MintCurve) -> Res<u64> {
    curve.step_lamports.checked_mul(curve.minted_count).and_then(|s| s.checked_add(curve.start_lamports)).ok_or_else(|| "the curve's price is past a u64".to_string())
}

/// A SEAT'S MINT, THE BUYER'S OWN: the mint door's answer (`door`: its permit and the issuer's signature in base64, the face it names) held to root
/// 0's curve as the chain holds it at `slot`, then written as one transaction its buyer alone signs, its fee payer, the blockhash blank: the
/// precompile's instruction and `mint_nft`, the buyer's ceiling the curve's price now (or `max`, never under it).
#[allow(clippy::too_many_arguments)]
pub fn seat_set(p: &Params, curve: &MintCurve, door: &Value, buyer: &Pubkey, slot: u64, max: Option<u64>, genesis: &str, labels: &BTreeMap<Pubkey, String>) -> Res<Value> {
    let bytes = |k: &str| door[k].as_str().ok_or_else(|| format!("the door's answer holds no {k}")).and_then(|t| wire::unb64(t).map_err(|e| format!("the door's {k}: {e}")));
    let permit: [u8; PERMIT_LEN] = bytes("permit")?.try_into().map_err(|_| format!("the door's permit is not {PERMIT_LEN} bytes"))?;
    let signature: [u8; 64] = bytes("signature")?.try_into().map_err(|_| "the door's signature is not 64 bytes".to_string())?;
    let text = |k: &str| door["metadata"][k].as_str().ok_or_else(|| format!("the door's answer names no metadata.{k}"));
    let face = Metadata { name: text("name")?, symbol: text("symbol")?, uri: text("uri")? };
    let fields = completion_permit::parse(&permit).map_err(|e| format!("the door's permit does not parse ({e:?})"))?;
    if fields.buyer != buyer.to_bytes() {
        return Err(format!("the permit seats {}, not {buyer}: a permit is its own buyer's alone", Pubkey::new_from_array(fields.buyer)));
    }
    if slot > fields.expiry_slot {
        return Err(format!("the permit ran out at slot {} (now {slot}): the door signs the same deposit's seat anew for the same buyer when asked again", fields.expiry_slot));
    }
    let issuer = Pubkey::new_from_array(curve.completion_issuer);
    VerifyingKey::from_bytes(&curve.completion_issuer).and_then(|k| k.verify_strict(&permit, &Signature::from_bytes(&signature))).map_err(|_| format!("the permit is not signed by root 0's issuer {issuer}: the curve seats no buyer by it"))?;
    if slot >= curve.closes_slot {
        return Err(format!("the curve's window closed at slot {} (now {slot}): no seat is minted on it now", curve.closes_slot));
    }
    if curve.minted_count >= curve.capacity {
        return Err(format!("every seat of the curve is minted ({} of {})", curve.minted_count, curve.capacity));
    }
    let price = price_now(curve)?;
    if price > fields.max_price_lamports {
        return Err(format!("the curve's price now, {price} lamports, is past the permit's ceiling, {}", fields.max_price_lamports));
    }
    let ceiling = max.unwrap_or(price);
    if ceiling < price {
        return Err(format!("--max {ceiling} is under the curve's price now, {price} lamports: the mint would be refused"));
    }
    let pair = nft::mint_instructions(curve, &permit, signature, face, ceiling).map_err(|e| format!("the client's mint refuses this permit for root 0's curve on this chain: {e:?}"))?;
    let ixs = [chain::budget_of(p, MINT_UNITS)?, pair.to_vec()].concat();
    let (mint, placeholders) = (nft::mint_pda(ROOT_UNIT, &fields.completion_digest), p.placeholders());
    let mut labels = labels.clone();
    for (k, w) in [
        (*buyer, "the permit's buyer: it signs this, pays the seat's price and the fee, and holds the seat"),
        (mint, "its seat's mint (a Lyssajudron), the deposit's own: one deposit, one mint"),
        (nft::seat_pda(&mint), "its seat's receipt (a hexagon of root 0's floor)"),
        (nft::initial_token_pda(&mint, buyer), "its token account, the buyer's"),
        (pair[0].program_id, "Solana's signature check (the precompile): the issuer's signature over the permit"),
    ] {
        labels.insert(k, w.to_string());
    }
    let opens = if slot < curve.opens_slot { format!("; sign and send once the curve's window opens at slot {} (now {slot})", curve.opens_slot) } else { String::new() };
    let does = [format!("mint one seat of root 0's floor at the curve's price now, {price} lamports ({ceiling} at most), by the permit root 0's issuer {issuer} signed for {buyer}: Solana checks its signature, then mint_nft seats {buyer}; the permit lasts to slot {}{opens}", fields.expiry_slot)];
    let tx = acts::entry(1, "mint a seat of root 0's floor", &does, &ixs, buyer, &labels, &placeholders)?;
    Ok(json!({
        "what": format!("THE DOOR SIGNS THE SEAT: {buyer}'s own mint of one seat of root 0's floor, by the permit the mint door signed for it; signed by {buyer} alone, its fee payer"),
        "cluster": p.cluster.name(),
        "genesisHash": genesis,
        "signable": placeholders.is_empty(),
        "placeholders": placeholders,
        "emitted": chain::now_iso(),
        "signer": { "address": buyer.to_string(), "role": "the permit's buyer: one hexagon a seat, like anyone's" },
        "permit": { "issuer": issuer.to_string(), "completion": wire::hex(&fields.completion_digest), "mint": mint.to_string(), "seat": nft::seat_pda(&mint).to_string(),
                    "maxPriceLamports": fields.max_price_lamports, "expirySlot": fields.expiry_slot, "priceLamports": price, "buyerMaxLamports": ceiling, "opens": curve.opens_slot, "closes": curve.closes_slot },
        "transactions": [tx],
    }))
}

impl Sitting {
    /// PRESS SEAT: the buyer's own mint of one seat of root 0's floor by the mint door's answer in `door`, held to root 0's curve and the slot as the
    /// chain holds them now: the set's path (D/seats/<buyer>.json).
    pub fn seat(&mut self, door: &Path, buyer: &Pubkey, max: Option<u64>) -> Res<PathBuf> {
        let answer: Value = serde_json::from_str(&std::fs::read_to_string(door).map_err(|e| format!("{}: {e}", door.display()))?).map_err(|e| format!("{}: not the door's answer ({e})", door.display()))?;
        let curve = MintCurve::decode(&self.account(&self.at.curve)?.ok_or("root 0's curve is not on the cluster: root 0 is not founded here")?.data).map_err(|_| "root 0's curve does not parse")?;
        let genesis = self.rec.get("genesisHash").as_str().unwrap_or_default().to_string();
        let set = seat_set(&self.p, &curve, &answer, buyer, self.rpc.slot()?, max, &genesis, &acts::labels(&self.ids, &self.at, &[]))?;
        let dir = self.dir.join("seats");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join(format!("{buyer}.json"));
        std::fs::write(&path, serde_json::to_string_pretty(&set).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!("  {buyer}'s mint of one seat of root 0's floor: one transaction for it to sign, at {} lamports, its permit good to slot {}", set["permit"]["priceLamports"], set["permit"]["expirySlot"]);
        Ok(path)
    }
}
