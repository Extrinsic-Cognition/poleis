//! THE DOOR SIGNS THE SEAT: `press issuer` makes the mint door's permit key beside the sitting's other keys, never over one, its address kept beside
//! it, the door's secret given only after his finger; `press seat` writes one buyer's own mint from the door's answer, held to root 0's curve, the
//! buyer, the issuer's signature and the slot: one transaction its buyer alone signs, its fee payer, its blockhash blank. The door's answer for the
//! client's vector (poleis/client/tests/permit-vector.json, the bytes the door's own test holds it to) is minted as any other. Fixed test keys only.
use poleis_client::nft::{self, Metadata, MintCurve, PERMIT_LEN};
use poleis_client::{PROGRAM_ID, ROOT_UNIT};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::seat::{crediter_is_the_kept, crediter_key, door_law, door_vars, issuer_is_the_kept, issuer_key, issuer_secret, kept_address, kept_keys_held, mine_vars, miner_is_the_kept, miner_key, miner_secret, seat_set};
use poleis_press::wire::{self, Key};
use poleis_treasury::completion_permit;
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

const FACE: Metadata<'static> = Metadata { name: "Lyssajudron", symbol: "LYSSA", uri: "https://optimism.fit/app/lyssajudron.json" };
const SLOT: u64 = 5_000;

fn params() -> Params {
    let s = |w: &str| k(w).to_string();
    let v = json!({
        "cluster": "localnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "V", "desc": "", "defaultMint": s("seat mint"), "planCap": 1, "planPeriod": 6480000, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("seat issuer") },
        "posiverse": { "name": "F", "chair": 2, "agents": [s("seat agent 1"), s("seat agent 2"), s("seat agent 3")], "charter": "33".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "u", "dollar": s("seat dollar"), "opening": 0, "miner": s("seat miner") },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1, "poolQuote": 1, "judgeSlots": 5000, "crediter": s("c"), "split": ["treasury"], "plan": ["treasury"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "pool": { "maxValidators": 10, "tag": "lp1" }
    });
    Params::from_text(&v.to_string()).unwrap()
}

/// Root 0's curve as the chain would hold it: three seats minted, the window open at SLOT, its permits the issuer's.
fn curve(issuer: &Key) -> MintCurve {
    MintCurve { bump: 0, unit: ROOT_UNIT, start_lamports: 1_000_000, step_lamports: 100_000, minted_count: 3, capacity: 20, opens_slot: 4_000, closes_slot: 9_000, completion_issuer: issuer.address.to_bytes(), genesis_hash: [7; 32] }
}

/// The mint door's answer as space-router-seat.mjs gives it: the permit and the signature in base64, the face it names.
fn answer(c: &MintCurve, signer: &Key, buyer: &Pubkey, expiry: u64) -> Value {
    let permit = nft::permit_message(c, *buyer, [9; 32], FACE, c.start_lamports + c.step_lamports * (c.capacity - 1), expiry).unwrap();
    json!({ "permit": wire::b64(&permit), "signature": wire::b64(&signer.sign(&permit)), "metadata": { "name": FACE.name, "symbol": FACE.symbol, "uri": FACE.uri } })
}

fn permit_of(door: &Value) -> ([u8; PERMIT_LEN], [u8; 64]) {
    let b = |k: &str| wire::unb64(door[k].as_str().unwrap()).unwrap();
    (b("permit").try_into().unwrap(), b("signature").try_into().unwrap())
}

#[test]
fn a_buyers_seat_is_its_own_mint_and_nothing_else() {
    let (p, issuer) = (params(), Key::from_seed(&[5; 32]));
    let (c, buyer) = (curve(&issuer), k("a buyer of root 0's floor"));
    let door = answer(&c, &issuer, &buyer, SLOT + 9_000);
    let set = seat_set(&p, &c, &door, &buyer, SLOT, None, "g", &BTreeMap::new()).unwrap_or_else(|e| panic!("{e}"));
    let txs = set["transactions"].as_array().unwrap();
    assert_eq!(txs.len(), 1, "one transaction");
    let m = wire::unb64(txs[0]["message"].as_str().unwrap()).unwrap();
    let (signers, keys, at) = wire::layout(&m).unwrap();
    assert_eq!((signers, keys[0]), (1, buyer), "the buyer alone signs it, its fee payer");
    assert!(m[at..at + 32].iter().all(|b| *b == 0), "its blockhash blank");
    assert_eq!(txs[0]["messageSha256"].as_str().unwrap(), wire::hex(&wire::sha256(&m)));
    assert!(txs[0]["bytes"].as_u64().unwrap() as usize <= wire::PACKET);
    // exactly the client's own pair for this permit, the buyer's ceiling the curve's price now (three seats minted before it)
    let price = 1_000_000 + 3 * 100_000;
    let (permit, signature) = permit_of(&door);
    let pair = nft::mint_instructions(&c, &permit, signature, FACE, price).unwrap();
    assert_eq!(m, wire::message(&pair, &buyer, &[0; 32]).0, "the precompile's instruction, then mint_nft, and nothing else");
    completion_permit::check_ed25519_instruction(&pair[0].program_id.to_bytes(), &pair[0].data, &c.completion_issuer, &permit).unwrap(); // classical: Solana's precompile verifies only Ed25519
    assert_eq!((set["permit"]["priceLamports"].clone(), set["permit"]["mint"].clone()), (json!(price), json!(nft::mint_pda(ROOT_UNIT, &[9; 32]).to_string())));
    assert_eq!((set["signer"]["address"].as_str(), set["signable"].as_bool()), (Some(buyer.to_string().as_str()), Some(true)));
    // --max above the price is the buyer's own ceiling
    let more = seat_set(&p, &c, &door, &buyer, SLOT, Some(price + 5), "g", &BTreeMap::new()).unwrap();
    let m = wire::unb64(more["transactions"][0]["message"].as_str().unwrap()).unwrap();
    assert_eq!(m, wire::message(&nft::mint_instructions(&c, &permit, signature, FACE, price + 5).unwrap(), &buyer, &[0; 32]).0);
    // before the window opens it is written all the same, and says when to send it
    let early = seat_set(&p, &c, &door, &buyer, 3_000, None, "g", &BTreeMap::new()).unwrap();
    assert!(early["transactions"][0]["does"][0].as_str().unwrap().contains("once the curve's window opens at slot 4000"));
}

#[test]
fn a_permit_the_curve_would_refuse_is_refused_before_anything_is_written() {
    let (p, issuer) = (params(), Key::from_seed(&[5; 32]));
    let (c, buyer) = (curve(&issuer), k("a buyer of root 0's floor"));
    let door = answer(&c, &issuer, &buyer, 20_000);
    let refused = |c: &MintCurve, door: &Value, buyer: &Pubkey, slot: u64, max: Option<u64>| seat_set(&p, c, door, buyer, slot, max, "g", &BTreeMap::new()).unwrap_err();
    assert!(refused(&c, &door, &k("someone else"), SLOT, None).contains("a permit is its own buyer's alone"));
    assert!(refused(&c, &answer(&c, &Key::from_seed(&[6; 32]), &buyer, 20_000), &buyer, SLOT, None).contains("is not signed by root 0's issuer"));
    assert!(refused(&c, &answer(&c, &issuer, &buyer, SLOT + 10), &buyer, SLOT + 11, None).contains("ran out at slot"));
    assert!(refused(&c, &door, &buyer, 9_000, None).contains("window closed at slot 9000"));
    assert!(refused(&MintCurve { minted_count: 20, ..c }, &door, &buyer, SLOT, None).contains("every seat of the curve is minted"));
    assert!(refused(&c, &door, &buyer, SLOT, Some(1_299_999)).contains("is under the curve's price now"));
    // another chain's curve, the same issuer: the client's own mint refuses it
    assert!(refused(&MintCurve { genesis_hash: [8; 32], ..c }, &door, &buyer, SLOT, None).contains("WrongChain"));
    // a face that is not the one the permit's digest names
    let mut other = door.clone();
    other["metadata"]["uri"] = json!("https://optimism.fit/app/another.json");
    assert!(refused(&c, &other, &buyer, SLOT, None).contains("WrongMetadata"));
    // an answer that is not the door's
    for (k, v) in [("permit", Value::Null), ("permit", json!(wire::b64(&[1; PERMIT_LEN - 1]))), ("signature", json!("not base64!"))] {
        let mut bad = door.clone();
        bad[k] = v;
        assert!(seat_set(&p, &c, &bad, &buyer, SLOT, None, "g", &BTreeMap::new()).is_err(), "{k}");
    }
}

#[test]
fn the_doors_answer_for_the_clients_vector_is_minted_as_any() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../client/tests/permit-vector.json");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let n = |x: &Value| x.as_str().unwrap().parse::<u64>().unwrap();
    let issuer = Pubkey::from_str(v["key"]["issuer"].as_str().unwrap()).unwrap();
    let c = MintCurve {
        bump: 0, unit: ROOT_UNIT, start_lamports: n(&v["curve"]["start"]), step_lamports: n(&v["curve"]["step"]), minted_count: 0, capacity: n(&v["curve"]["capacity"]),
        opens_slot: n(&v["curve"]["opens"]), closes_slot: n(&v["curve"]["closes"]), completion_issuer: issuer.to_bytes(), genesis_hash: Pubkey::from_str(v["fields"]["genesis"].as_str().unwrap()).unwrap().to_bytes(),
    };
    let hex = |h: &str| (0..h.len()).step_by(2).map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap()).collect::<Vec<u8>>();
    let door = json!({ "permit": wire::b64(&hex(v["message"].as_str().unwrap())), "signature": wire::b64(&hex(v["signature"].as_str().unwrap())), "metadata": v["face"] });
    let buyer = Pubkey::from_str(v["fields"]["buyer"].as_str().unwrap()).unwrap();
    let set = seat_set(&params(), &c, &door, &buyer, n(&v["slot"]), None, "g", &BTreeMap::new()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(set["permit"]["completion"], v["fields"]["completion"]);
    assert_eq!(set["permit"]["mint"], json!(nft::mint_pda(ROOT_UNIT, &hex(v["fields"]["completion"].as_str().unwrap()).try_into().unwrap()).to_string()));
    assert_eq!(v["fields"]["program"], json!(PROGRAM_ID.to_string()), "the vector is this tree's Poleis");
}

struct Dir(PathBuf);
impl Dir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!("press-seat-{name}-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A stand-in asker: a shell script that says `says` and exits `code` (tests/finger.rs's own).
fn an_asker(dir: &Path, name: &str, says: &str, code: u8) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, format!("#!/bin/sh\nprintf '%s\\n' '{says}'\nexit {code}\n")).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p
}

#[test]
fn the_issuers_key_is_made_once_beside_the_others_and_read_only_after_his_finger() {
    let d = Dir::new("issuer");
    let fixed = Key::from_seed(&[5; 32]).address;
    let (a, made) = issuer_key(&d.0, || Ok(Key::from_seed(&[5; 32]))).unwrap();
    assert_eq!((a, made), (fixed, true));
    let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!((mode(&d.0), mode(&d.0.join("keys")), mode(&d.0.join("keys/issuer.json"))), (0o700, 0o700, 0o600), "his user's alone");
    assert_eq!(std::fs::read_to_string(d.0.join("keys/issuer.address")).unwrap(), format!("{fixed}\n"), "its address beside it, no key in it");
    // asked again: the key that stands, and no other is made
    assert_eq!(issuer_key(&d.0, || panic!("a second key is never made")).unwrap(), (fixed, false));
    // the door's secret: only after his finger, the seed and the address in base58
    let (yes, no) = (an_asker(&d.0, "yes", "ok", 0), an_asker(&d.0, "no", "no", 1));
    let secret: [u8; 64] = wire::unbase58(&issuer_secret(&d.0, &yes).unwrap()).unwrap();
    assert_eq!((Key::from_seed(&secret[..32].try_into().unwrap()).address, &secret[32..]), (fixed, fixed.as_ref()));
    assert!(issuer_secret(&d.0, &no).unwrap_err().contains("his finger did not say yes"));
    // a key whose address beside it is gone is never written over
    std::fs::remove_file(d.0.join("keys/issuer.address")).unwrap();
    assert!(issuer_key(&d.0, || panic!("never over a key")).unwrap_err().contains("the press never writes over a key"));
}

#[test]
fn the_miners_key_is_made_once_beside_the_issuers_and_read_only_after_his_finger() {
    // THE SWARM NAMES ITS MINER: the door's crank key, made as the issuer's is, a key of its own beside it
    let d = Dir::new("miner");
    let (issuer, miner) = (Key::from_seed(&[5; 32]).address, Key::from_seed(&[6; 32]).address);
    issuer_key(&d.0, || Ok(Key::from_seed(&[5; 32]))).unwrap();
    assert_eq!(miner_key(&d.0, || Ok(Key::from_seed(&[6; 32]))).unwrap(), (miner, true));
    let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!((mode(&d.0.join("keys")), mode(&d.0.join("keys/miner.json"))), (0o700, 0o600), "his user's alone");
    assert_eq!((kept_address(&d.0, "miner").unwrap(), kept_address(&d.0, "issuer").unwrap()), (Some(miner), Some(issuer)), "each address beside its own key, the two keys apart");
    assert_eq!(miner_key(&d.0, || panic!("a second key is never made")).unwrap(), (miner, false));
    // the door's secret: only after his finger, the seed and the address in base58, the miner's and not the issuer's
    let (yes, no) = (an_asker(&d.0, "yes", "ok", 0), an_asker(&d.0, "no", "no", 1));
    let secret: [u8; 64] = wire::unbase58(&miner_secret(&d.0, &yes).unwrap()).unwrap();
    assert_eq!((Key::from_seed(&secret[..32].try_into().unwrap()).address, &secret[32..]), (miner, miner.as_ref()));
    assert!(miner_secret(&d.0, &no).unwrap_err().contains("his finger did not say yes"));
    // the sixth act names the key it made: the params naming another are refused, a placeholder or the same address pass
    let mut p = params();
    p.raw["warm"]["miner"] = json!(miner.to_string());
    miner_is_the_kept(&d.0, &p).unwrap();
    p.raw["warm"]["miner"] = json!(issuer.to_string());
    let e = miner_is_the_kept(&d.0, &p).unwrap_err();
    assert!(e.contains("is not the key press miner made") && e.contains(&miner.to_string()), "{e}");
    p.raw["warm"]["miner"] = json!("PLACEHOLDER: the mint door's crank key");
    miner_is_the_kept(&d.0, &p).unwrap();
    // where no miner was made here, any address of the params is the sitting's word
    let none = Dir::new("no-miner");
    assert_eq!(kept_address(&none.0, "miner").unwrap(), None);
    p.raw["warm"]["miner"] = json!(issuer.to_string());
    miner_is_the_kept(&none.0, &p).unwrap();
    // a key whose address beside it is gone is never written over
    std::fs::remove_file(d.0.join("keys/miner.address")).unwrap();
    assert!(miner_key(&d.0, || panic!("never over a key")).unwrap_err().contains("the press never writes over a key"));
}

#[test]
fn the_crediters_key_is_made_once_beside_the_others_on_their_pattern() {
    // THE PRESS IS READY FOR MAINNET: the board's crediter, made as the issuer's and the miner's are, a key of its own beside them
    let d = Dir::new("crediter");
    let (issuer, miner, crediter) = (Key::from_seed(&[5; 32]).address, Key::from_seed(&[6; 32]).address, Key::from_seed(&[7; 32]).address);
    issuer_key(&d.0, || Ok(Key::from_seed(&[5; 32]))).unwrap();
    miner_key(&d.0, || Ok(Key::from_seed(&[6; 32]))).unwrap();
    assert_eq!(crediter_key(&d.0, || Ok(Key::from_seed(&[7; 32]))).unwrap(), (crediter, true));
    let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!((mode(&d.0), mode(&d.0.join("keys")), mode(&d.0.join("keys/crediter.json"))), (0o700, 0o700, 0o600), "his user's alone");
    assert_eq!(std::fs::read_to_string(d.0.join("keys/crediter.address")).unwrap(), format!("{crediter}\n"), "its address beside it, no key in it");
    let text = std::fs::read_to_string(d.0.join("keys/crediter.json")).unwrap();
    assert_eq!(Key::from_cli_text(&text).unwrap().address, crediter, "the Solana CLI's file shape, the key it names");
    assert_eq!((kept_address(&d.0, "crediter").unwrap(), kept_address(&d.0, "miner").unwrap(), kept_address(&d.0, "issuer").unwrap()), (Some(crediter), Some(miner), Some(issuer)), "each address beside its own key");
    // asked again: the key that stands, and no other is made
    assert_eq!(crediter_key(&d.0, || panic!("a second key is never made")).unwrap(), (crediter, false));
    // made from the system's randomness by the step: two folders, two keys
    let (e, f) = (Dir::new("crediter-fresh-1"), Dir::new("crediter-fresh-2"));
    let (a, _) = crediter_key(&e.0, Key::fresh).unwrap();
    let (b, _) = crediter_key(&f.0, Key::fresh).unwrap();
    assert_ne!(a, b, "a fresh key each sitting");
    // the second act names the key it made: the params naming another are refused, a placeholder or the same address pass
    let mut p = params();
    p.raw["market"]["crediter"] = json!(crediter.to_string());
    crediter_is_the_kept(&d.0, &p).unwrap();
    p.raw["market"]["crediter"] = json!(miner.to_string());
    let e = crediter_is_the_kept(&d.0, &p).unwrap_err();
    assert!(e.contains("market.crediter") && e.contains("is not the key press crediter made") && e.contains(&crediter.to_string()), "{e}");
    p.raw["market"]["crediter"] = json!("PLACEHOLDER: the board's crediter");
    crediter_is_the_kept(&d.0, &p).unwrap();
    // a key whose address beside it is gone is never written over
    std::fs::remove_file(d.0.join("keys/crediter.address")).unwrap();
    assert!(crediter_key(&d.0, || panic!("never over a key")).unwrap_err().contains("the press never writes over a key"));
}

#[test]
fn a_door_key_is_never_made_inside_a_git_repository() {
    let d = Dir::new("in-a-repository");
    assert!(std::process::Command::new("git").arg("-C").arg(&d.0).args(["init", "-q"]).status().unwrap().success());
    for (name, made) in [("issuer", issuer_key(&d.0, || panic!("no key is made in a repository"))), ("miner", miner_key(&d.0, || panic!("no key is made in a repository"))), ("crediter", crediter_key(&d.0, || panic!("no key is made in a repository")))] {
        assert!(made.unwrap_err().contains("is inside a git repository"), "{name}");
    }
    assert!(!d.0.join("keys").exists(), "no keys folder left behind");
}

#[test]
fn the_curves_issuer_is_held_to_the_key_press_issuer_made() {
    // THE PRESS IS READY FOR MAINNET: THE SWARM NAMES ITS MINER said floor.issuer was not held to D/keys/issuer.address; now it is, beside the
    // miner and the crediter, and every one that differs is named
    let d = Dir::new("held");
    let (issuer, miner, crediter, other) = (Key::from_seed(&[5; 32]).address, Key::from_seed(&[6; 32]).address, Key::from_seed(&[7; 32]).address, k("another key"));
    issuer_key(&d.0, || Ok(Key::from_seed(&[5; 32]))).unwrap();
    let mut p = params();
    p.raw["floor"]["issuer"] = json!(issuer.to_string());
    issuer_is_the_kept(&d.0, &p).unwrap();
    kept_keys_held(&d.0, &p).unwrap();
    p.raw["floor"]["issuer"] = json!(other.to_string());
    let e = issuer_is_the_kept(&d.0, &p).unwrap_err();
    assert!(e.contains("floor.issuer") && e.contains("is not the key press issuer made") && e.contains(&issuer.to_string()) && e.contains("no buyer could mint a seat"), "{e}");
    assert_eq!(kept_keys_held(&d.0, &p).unwrap_err(), e, "the sitting's hold names it");
    p.raw["floor"]["issuer"] = json!("PLACEHOLDER: the mint door's permit key");
    issuer_is_the_kept(&d.0, &p).unwrap();
    // all three made, all three named another: the hold names each
    miner_key(&d.0, || Ok(Key::from_seed(&[6; 32]))).unwrap();
    crediter_key(&d.0, || Ok(Key::from_seed(&[7; 32]))).unwrap();
    p.raw["floor"]["issuer"] = json!(other.to_string());
    p.raw["warm"]["miner"] = json!(other.to_string());
    p.raw["market"]["crediter"] = json!(other.to_string());
    let e = kept_keys_held(&d.0, &p).unwrap_err();
    for (path, kept) in [("floor.issuer", issuer), ("warm.miner", miner), ("market.crediter", crediter)] {
        assert!(e.contains(path) && e.contains(&kept.to_string()), "{path}: {e}");
    }
    p.raw["floor"]["issuer"] = json!(issuer.to_string());
    p.raw["warm"]["miner"] = json!(miner.to_string());
    p.raw["market"]["crediter"] = json!(crediter.to_string());
    kept_keys_held(&d.0, &p).unwrap();
    // where no key was made here, the params' word stands
    let none = Dir::new("held-none");
    p.raw["floor"]["issuer"] = json!(other.to_string());
    kept_keys_held(&none.0, &p).unwrap();
    // and the sitting asks it where the curve's issuer is fixed, and where the acts are written
    let chain = include_str!("../src/chain.rs");
    assert!(chain.contains("params_checked(&self.p, &self.ids, &self.at)?;\n        crate::seat::kept_keys_held(&self.dir, &self.p)"), "the deploy's check before anything starts");
    assert!(chain.contains("crate::seat::kept_keys_held(&self.dir, &self.p)?;\n        if self.account(&self.at.config)?.is_some()"), "press found, before the founding");
    assert!(include_str!("../src/main.rs").contains("seat::kept_keys_held(&s.dir, &s.p).and_then(|_| s.stake_pool_stands())"), "press acts");
}

#[test]
fn the_doors_vars_name_root_0s_curve_under_the_sittings_program() {
    let v = door_vars(&params(), &PROGRAM_ID, "the genesis");
    assert_eq!(v, json!({ "POLEIS_PROGRAM": PROGRAM_ID.to_string(), "POLEIS_GENESIS": "the genesis", "POLEIS_CURVE": nft::curve_pda(ROOT_UNIT).to_string(), "POLEIS_START": "1000000", "POLEIS_STEP": "100000", "POLEIS_CAPACITY": poleis_client::ladder_hexagons(1).to_string() }));
    // mainnet's params, the sheet's picks his since Nov 23 2026 19:37:50 UTC: the curve's first price, its step, rung 5's 150 seats
    let m = Params::read(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.mainnet.json"))).unwrap();
    let v = door_vars(&m, &PROGRAM_ID, poleis_press::rpc::MAINNET_GENESIS);
    assert_eq!((v["POLEIS_START"].as_str(), v["POLEIS_STEP"].as_str(), v["POLEIS_CAPACITY"].as_str()), (Some("1000000"), Some("100000"), Some("150")));
    assert_eq!(poleis_client::ladder_hexagons(5), 150);
    // a value not his yet says so
    let mut m = m;
    m.raw["floor"]["stepLamports"] = json!("PLACEHOLDER: what each seat after the first adds");
    assert!(door_vars(&m, &PROGRAM_ID, poleis_press::rpc::MAINNET_GENESIS)["POLEIS_STEP"].as_str().unwrap().starts_with("PLACEHOLDER"));
}

#[test]
fn the_doors_vars_of_the_mine_name_root_0s_vault_and_the_mines_opening() {
    // THE MINE IS LIVE: the mine under the sitting's $WARM program, the treasury root 0's own vault of the dollar under the sitting's Poleis (the
    // address `chain::treasury_of` derives), the opening as the door reads an instant
    let warm = Pubkey::new_from_array(warm_mine::ID.to_bytes());
    let mut p = params();
    let dollar = Pubkey::new_from_array([9; 32]);
    p.raw["warm"]["dollar"] = json!(dollar.to_string());
    let v = mine_vars(&p, &PROGRAM_ID, &warm, Some(1_779_558_634_005));
    assert_eq!(v, json!({ "SPACE_ROUTER_MINE": poleis_seat::mine::mine_at().to_string(), "SPACE_ROUTER_TREASURY": poleis_client::vault_pda(ROOT_UNIT, &dollar).to_string(), "SPACE_ROUTER_OPENED_AT": "2026-05-23T17:50:34.005Z" }));
    assert_eq!(v["SPACE_ROUTER_TREASURY"], json!(poleis_press::chain::treasury_of(&p).unwrap().to_string()), "the door's treasury is the one the press opens the mine with");
    // before the mine stands, and with a dollar that is not his yet, the vars say so
    let v = mine_vars(&Params::read(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json"))).unwrap(), &PROGRAM_ID, &warm, None);
    assert!(v["SPACE_ROUTER_OPENED_AT"].as_str().unwrap().contains("once press warm has opened it") && v["SPACE_ROUTER_TREASURY"].as_str().unwrap().starts_with("STAND-IN"), "{v}");
}

#[test]
fn the_doors_seat_law_is_named_beside_its_vars() {
    let said = door_law(&json!({ "bytes": 260000, "sha256": "ab".repeat(32), "path": "/the/copy/poleis/seat-wasm/target/poleis-seat.wasm" }));
    assert!(said.contains("worker/poleis-seat.wasm") && said.contains("/the/copy/poleis/seat-wasm/target/poleis-seat.wasm") && said.contains(&"ab".repeat(32)) && said.contains("260000 bytes"), "{said}");
    assert!(door_law(&Value::Null).contains("press build makes it"), "before the build, the road to it");
}
