//! THE DOOR SIGNS THE SEAT (Nov 22 2026, lane/the-door-signs-the-seat; his 15:20:34 UTC "Yes, build it" to "Next: the mint door signs each finished
//! Lyssajudron's permit, so the floor can fill?"). The mint door (embertide-app/ui/intro/dyson-society/worker/space-router-seat.mjs) is the curve's
//! permit issuer, and its permit is Poleis's own bytes: this test writes the vector beside it (permit-vector.json), a fixed test key nobody issues
//! with, a permit's fields as the door fills them (the completion: sha256 of the door's domain and a deposit's signature; the fixed face, no person's;
//! the curve's highest price; an expiry PERMIT_SLOTS past a slot), the message `permit_message` makes of them and the key's signature over it. The
//! door's own test (worker/space-router-seat.test.mjs) builds the same bytes with the door's functions, signs them with the same key by WebCrypto and
//! holds both to this vector. Written by `POLEIS_WRITE_VECTOR=1 cargo test --offline --test permit_vector`; at every other run it is held to what the
//! client makes now, and the client's own mint takes it (the precompile's instruction and `mint_nft`, the signature good for the curve's issuer).
use ed25519_dalek::{Signer as _, SigningKey, Verifier as _}; // classical: Solana's precompile verifies only Ed25519
use poleis_client::nft::{self, Metadata, MintCurve};
use poleis_client::{ladder_hexagons, PROGRAM_ID};
use poleis_treasury::completion_permit::{self, PermitFields};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use solana_pubkey::Pubkey;
use std::str::FromStr as _;

/// The door's domain for a deposit's completion (space-router-seat.mjs COMPLETION): the seat's mint derives from it, so a deposit seats once.
const COMPLETION: &[u8] = b"space-router:seat:completion:v1\0";
/// The face every seat of root 0's floor is minted with (space-router-seat.mjs FACE): a Lyssajudron's, no person's.
const FACE: Metadata<'static> = Metadata { name: "Lyssajudron", symbol: "LYSSA", uri: "https://optimism.fit/app/lyssajudron.json" };
/// Mainnet-beta's genesis hash: the chain the door's permits name.
const MAINNET: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
/// The slot the door's node read, and the slots a permit lasts past it (space-router-seat.mjs PERMIT_SLOTS).
const SLOT: u64 = 400_000_000;
const PERMIT_SLOTS: u64 = 9_000;
/// The curve's terms: the rehearsal's prices, the first rung's seats.
const START: u64 = 1_000_000;
const STEP: u64 = 100_000;

fn sha(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The vector as the client makes it today.
fn vector() -> Value {
    // a fixed test key: the issuer of this vector and of nothing else
    let seed = sha(&[b"poleis permit vector: a test key, never an issuer's"]);
    let issuer = SigningKey::from_bytes(&seed);
    let capacity = ladder_hexagons(1);
    let curve = MintCurve {
        bump: 0,
        unit: 0,
        start_lamports: START,
        step_lamports: STEP,
        minted_count: 0,
        capacity,
        opens_slot: SLOT,
        closes_slot: SLOT + 700,
        completion_issuer: issuer.verifying_key().to_bytes(),
        genesis_hash: Pubkey::from_str(MAINNET).unwrap().to_bytes(),
    };
    let buyer = Pubkey::new_from_array(sha(&[b"poleis permit vector: a buyer nobody holds"]));
    // a deposit's signature as the door's pattern takes it: two addresses' base58 side by side, 88 characters
    let deposit = format!("{}{}", Pubkey::new_from_array([0x11; 32]), Pubkey::new_from_array([0x22; 32]));
    assert!((64..=88).contains(&deposit.len()), "a deposit's signature is 64 to 88 base58 characters: {}", deposit.len());
    let completion = sha(&[COMPLETION, deposit.as_bytes()]);
    let max = START + STEP * (capacity - 1);
    let expiry = SLOT + PERMIT_SLOTS;
    let message = nft::permit_message(&curve, buyer, completion, FACE, max, expiry).unwrap();
    let signature = issuer.sign(&message).to_bytes();

    // the bytes are Poleis's: they parse back to the fields the door fills, root 0's curve on this program, the face's digest
    let fields = completion_permit::parse(&message).unwrap();
    let face = nft::metadata_digest(FACE).unwrap();
    assert_eq!(
        fields,
        PermitFields {
            program_id: PROGRAM_ID.to_bytes(),
            chain_genesis_hash: curve.genesis_hash,
            unit: 0,
            curve: nft::curve_pda(0).to_bytes(),
            buyer: buyer.to_bytes(),
            completion_digest: completion,
            metadata_digest: face,
            max_price_lamports: max,
            expiry_slot: expiry,
        }
    );
    // the issuer's signature holds, and the client's own mint takes the permit: the precompile's instruction first, bound to the curve's issuer
    issuer.verifying_key().verify(&message, &ed25519_dalek::Signature::from_bytes(&signature)).unwrap(); // classical: Solana's precompile verifies only Ed25519
    let pair = nft::mint_instructions(&curve, &message, signature, FACE, max).unwrap();
    completion_permit::check_ed25519_instruction(&pair[0].program_id.to_bytes(), &pair[0].data, &curve.completion_issuer, &message).unwrap(); // classical: Solana's precompile verifies only Ed25519
    assert_eq!(&pair[0].data[48..112], &signature[..], "the signature rides in the precompile's instruction");
    assert_eq!(pair[1].accounts[6].pubkey, nft::mint_pda(0, &completion), "the seat's mint is the deposit's: one deposit, one mint");

    json!({
        "what": "THE DOOR SIGNS THE SEAT: one permit of the mint door, as Poleis's client makes it (poleis/client/tests/permit_vector.rs) and the door must (worker/space-router-seat.test.mjs): a fixed test key, the fields, the message, the signature. No key of anyone's.",
        "key": { "seed": hex(&seed), "issuer": Pubkey::new_from_array(issuer.verifying_key().to_bytes()).to_string() },
        "deposit": deposit,
        "completionDomain": String::from_utf8_lossy(COMPLETION),
        "face": { "name": FACE.name, "symbol": FACE.symbol, "uri": FACE.uri },
        "curve": { "start": START.to_string(), "step": STEP.to_string(), "capacity": capacity.to_string(), "opens": curve.opens_slot.to_string(), "closes": curve.closes_slot.to_string() },
        "slot": SLOT.to_string(),
        "permitSlots": PERMIT_SLOTS.to_string(),
        "fields": {
            "program": PROGRAM_ID.to_string(),
            "genesis": MAINNET,
            "unit": 0,
            "curve": nft::curve_pda(0).to_string(),
            "buyer": buyer.to_string(),
            "completion": hex(&completion),
            "metadataDigest": hex(&face),
            "maxPriceLamports": max.to_string(),
            "expirySlot": expiry.to_string(),
        },
        "message": hex(&message),
        "signature": hex(&signature),
    })
}

#[test]
fn the_doors_permit_is_poleiss_own_bytes() {
    let v = vector();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/permit-vector.json");
    if std::env::var_os("POLEIS_WRITE_VECTOR").is_some() {
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
        return;
    }
    let held: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e} (POLEIS_WRITE_VECTOR=1 writes it)"))).unwrap();
    assert_eq!(held, v, "the vector is what the client makes today: written again only by POLEIS_WRITE_VECTOR=1, and the door's test with it");
    assert_eq!(v["message"].as_str().unwrap().len(), 2 * completion_permit::PERMIT_LEN);
}
