//! THE FLASK TAKES ITS SEAT (Nov 22 2026, lane/the-flask-takes-its-seat; his 17:07:40 UTC "Yes, build it"). The seat's offer as this crate writes it,
//! held beside it (seat-vector.json): the inputs as the mint door gives them (root 0's curve as the chain would hold it, three seats minted; the door's
//! own permit for the client's permit vector's deposit and a fixed test flask, signed by that vector's fixed test issuer key; a fixed test fee key; the
//! rent, a blockhash, the door's floor price, the slot), the doors' ask and answer byte for byte, the message, and the wire the door must send (the fee
//! key's signature first, the flask's). The door's own test (worker/space-router-seat.test.mjs) asks the Worker's handler, which asks
//! worker/poleis-seat.wasm, on a node that answers this chain, and holds what it offers and sends to this vector: the WASM is this crate. No key of
//! anyone's. Written by `POLEIS_WRITE_VECTOR=1 cargo test --offline --test seat_vector`; at every other run held to what the crate writes now.
use ed25519_dalek::{Signer as _, SigningKey}; // classical: Solana's precompile verifies only Ed25519, and a Solana transaction is signed with it
use poleis_client::nft::{self, Metadata, MintCurve};
use poleis_client::PROGRAM_ID;
use poleis_seat::{answer, ask_bytes, ask_of, mint_of, offer, rent_of, Ask, Refusal, ANSWER_HEAD, ASK_FIXED, COMPUTE_BUDGET, DEFAULT_UNITS, PACKET, SIGNATURES, SIGNATURE_LAMPORTS, VOTES};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;
use std::str::FromStr as _;

/// The face every seat of root 0's floor is minted with (the door's FACE, the permit vector's).
const FACE: Metadata<'static> = Metadata { name: "Lyssajudron", symbol: "LYSSA", uri: "https://optimism.fit/app/lyssajudron.json" };
/// The rent of an account of 0 and of 1 byte today: 6,960 lamports a byte.
const RENT: [u64; 2] = [890_880, 897_840];
/// The door's floor price (app/space-router-deposit.mjs PRICE_FLOOR): what it pays a unit when its node knows no fee.
const PRICE: u64 = 1_000_000;

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

/// The keys, the curve and the deposit: the permit vector's chain, its deposit and its issuer, a flask and a fee key of this vector's own.
struct Case {
    pv: Value,
    issuer: SigningKey,
    flask: SigningKey,
    fee: SigningKey,
    curve: MintCurve,
    permit: [u8; nft::PERMIT_LEN],
    signature: [u8; 64],
}

fn case() -> Case {
    let pv: Value = serde_json::from_str(include_str!("../../client/tests/permit-vector.json")).unwrap();
    let n = |v: &Value| v.as_str().unwrap().parse::<u64>().unwrap();
    let seed: [u8; 32] = (0..32).map(|i| u8::from_str_radix(&pv["key"]["seed"].as_str().unwrap()[2 * i..2 * i + 2], 16).unwrap()).collect::<Vec<_>>().try_into().unwrap();
    let issuer = SigningKey::from_bytes(&seed);
    let flask = SigningKey::from_bytes(&sha(&[b"poleis seat vector: a flask, a test key nobody holds"]));
    let fee = SigningKey::from_bytes(&sha(&[b"poleis seat vector: the house's fee key, a test key nobody holds"]));
    let curve = MintCurve {
        bump: Pubkey::find_program_address(&[poleis_treasury::nft_seat_state::CURVE_SEED, &0u64.to_le_bytes()], &PROGRAM_ID).1,
        unit: 0,
        start_lamports: n(&pv["curve"]["start"]),
        step_lamports: n(&pv["curve"]["step"]),
        minted_count: 3,
        capacity: n(&pv["curve"]["capacity"]),
        opens_slot: n(&pv["curve"]["opens"]),
        closes_slot: n(&pv["curve"]["closes"]),
        completion_issuer: issuer.verifying_key().to_bytes(),
        genesis_hash: Pubkey::from_str(pv["fields"]["genesis"].as_str().unwrap()).unwrap().to_bytes(),
    };
    assert_eq!((pv["face"]["name"].as_str(), pv["face"]["symbol"].as_str(), pv["face"]["uri"].as_str()), (Some(FACE.name), Some(FACE.symbol), Some(FACE.uri)), "the face is the door's");
    // the door's permit for this deposit and this flask: its completion the deposit's, the curve's highest price, an expiry PERMIT_SLOTS past the slot
    let completion = sha(&[pv["completionDomain"].as_str().unwrap().as_bytes(), pv["deposit"].as_str().unwrap().as_bytes()]);
    let buyer = Pubkey::new_from_array(flask.verifying_key().to_bytes());
    let permit = nft::permit_message(&curve, buyer, completion, FACE, n(&pv["fields"]["maxPriceLamports"]), n(&pv["slot"]) + n(&pv["permitSlots"])).unwrap();
    let signature = issuer.sign(&permit).to_bytes();
    Case { pv, issuer, flask, fee, curve, permit, signature }
}

impl Case {
    fn ask<'a>(&'a self, curve: &'a [u8]) -> Ask<'a> {
        Ask {
            curve,
            permit: self.permit,
            signature: self.signature,
            face: FACE,
            buyer: Pubkey::new_from_array(self.flask.verifying_key().to_bytes()),
            fee: Pubkey::new_from_array(self.fee.verifying_key().to_bytes()),
            rent: RENT,
            blockhash: sha(&[b"poleis seat vector: a blockhash"]),
            price: PRICE,
            slot: self.pv["slot"].as_str().unwrap().parse().unwrap(),
        }
    }
}

/// The vector as the crate writes it today.
fn vector() -> Value {
    let c = case();
    let curve = c.curve.encode().unwrap();
    let a = c.ask(&curve);
    let o = offer(&a).unwrap();
    // the seat's lamports: the curve's price after three seats; the rents the rehearsal's buyers paid (9,716,160 at 6,960 a byte: the seat 1,746,960,
    // the hook's validation 1,489,440, the mint 3,974,160, the token 2,108,880, the registry's row 396,720); the allowance, an empty account and ten
    // votes' fees; the fee besides: three signatures and the priority on the runtime's default units
    assert_eq!((o.price, o.rents, o.allowance), (1_300_000, 9_716_160, 940_880));
    assert_eq!(o.allowance, rent_of(RENT, 0).unwrap() + VOTES * SIGNATURE_LAMPORTS);
    assert_eq!(o.lamports, o.price + o.rents + o.allowance);
    assert_eq!(o.fee, SIGNATURES * SIGNATURE_LAMPORTS + DEFAULT_UNITS * PRICE / 1_000_000);
    // the message: two signers, the fee key first and the flask second, nothing signed read-only; 1,228 bytes on the wire with both signatures
    assert_eq!(&o.message[..2], &[2, 0]);
    assert_eq!((&o.message[4..36], &o.message[36..68]), (a.fee.as_ref(), a.buyer.as_ref()));
    assert_eq!(1 + 2 * 64 + o.message.len(), 1_228);
    assert_eq!(o.mint, nft::mint_pda(0, &sha(&[c.pv["completionDomain"].as_str().unwrap().as_bytes(), c.pv["deposit"].as_str().unwrap().as_bytes()])));
    // the doors' layouts: the ask read back offers the same; the answer is the numbers, the mint and the message
    let (ask, out) = (ask_bytes(&a), answer(&o));
    assert_eq!(ask.len(), ASK_FIXED + 12 + FACE.name.len() + FACE.symbol.len() + FACE.uri.len());
    assert_eq!(offer(&ask_of(&ask).unwrap()), Ok(o.clone()));
    assert_eq!((out.len(), &out[ANSWER_HEAD..]), (ANSWER_HEAD + o.message.len(), &o.message[..]));
    // the wire the door sends: the fee key's signature, then the flask's, then the message
    let (paid, signed) = (c.fee.sign(&o.message).to_bytes(), c.flask.sign(&o.message).to_bytes());
    let key = |k: &SigningKey| json!({ "seed": hex(&k.to_bytes()), "address": Pubkey::new_from_array(k.verifying_key().to_bytes()).to_string() });
    json!({
        "what": "THE FLASK TAKES ITS SEAT: one seat's offer of the mint door, as poleis/seat-wasm writes it (tests/seat_vector.rs) and the door's WASM must (worker/space-router-seat.test.mjs): fixed test keys, root 0's curve, the door's permit, the ask and the answer of the doors, the message and the wire. No key of anyone's.",
        "program": PROGRAM_ID.to_string(),
        "issuer": Pubkey::new_from_array(c.issuer.verifying_key().to_bytes()).to_string(),
        "flask": key(&c.flask),
        "fee": key(&c.fee),
        "deposit": c.pv["deposit"],
        "slot": c.pv["slot"],
        "rent": [RENT[0].to_string(), RENT[1].to_string()],
        "price": PRICE.to_string(),
        "blockhash": Pubkey::new_from_array(a.blockhash).to_string(),
        "curve": hex(&curve),
        "permit": hex(&c.permit),
        "signature": hex(&c.signature),
        "ask": hex(&ask),
        "answer": hex(&out),
        "message": hex(&o.message),
        "lamports": { "lamports": o.lamports.to_string(), "price": o.price.to_string(), "rents": o.rents.to_string(), "allowance": o.allowance.to_string(), "fee": o.fee.to_string() },
        "mint": o.mint.to_string(),
        "wire": hex(&[&[2u8][..], &paid, &signed, &o.message].concat()),
    })
}

#[test]
fn the_doors_seat_is_this_crates_offer() {
    let v = vector();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/seat-vector.json");
    if std::env::var_os("POLEIS_WRITE_VECTOR").is_some() {
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
        return;
    }
    let held: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e} (POLEIS_WRITE_VECTOR=1 writes it)"))).unwrap();
    assert_eq!(held, v, "the vector is what the crate writes today: written again only by POLEIS_WRITE_VECTOR=1, and the door's test with it");
}

#[test]
fn a_curve_shut_full_or_not_open_a_permit_not_the_flasks_and_another_poleiss_are_refused() {
    let c = case();
    let curve = c.curve.encode().unwrap();
    let base = c.ask(&curve);
    let full = MintCurve { minted_count: c.curve.capacity, ..c.curve }.encode().unwrap();
    assert_eq!(offer(&Ask { slot: c.curve.opens_slot - 1, ..base }), Err(Refusal::NotYet));
    assert_eq!(offer(&Ask { slot: c.curve.closes_slot, ..base }), Err(Refusal::Shut));
    assert_eq!(offer(&Ask { curve: &full, ..base }), Err(Refusal::Full));
    assert_eq!(offer(&Ask { curve: &curve[..100], ..base }), Err(Refusal::Curve));
    // the permit's buyer is the flask's, the fee key is not the flask, the face is the permit's, the permit is this Poleis's
    assert_eq!(offer(&Ask { buyer: base.fee, ..base }), Err(Refusal::Permit));
    assert_eq!(offer(&Ask { fee: base.buyer, ..base }), Err(Refusal::Permit));
    assert_eq!(offer(&Ask { face: Metadata { uri: "https://optimism.fit/app/another.json", ..FACE }, ..base }), Err(Refusal::Permit));
    let mut another = c.permit;
    another[30] ^= 1; // the first byte of the permit's program, after its domain
    assert_eq!(offer(&Ask { permit: another, ..base }), Err(Refusal::Permit));
    assert_eq!(offer(&Ask { rent: [1, u64::MAX], ..base }), Err(Refusal::Rent));
    // an ask the doors cannot read
    let ask = ask_bytes(&base);
    assert!(ask_of(&ask[..ask.len() - 1]).is_none() && ask_of(&[ask.clone(), vec![0]].concat()).is_none() && ask_of(&ask[..ASK_FIXED - 1]).is_none());
    assert_eq!(mint_of(&[7; 32]), nft::mint_pda(0, &[7; 32]));
}

#[test]
fn with_its_unit_limit_the_seats_one_message_is_past_solanas_packet() {
    // the design's first message carried SetComputeUnitLimit: 1,236 bytes with its two signatures, four past the packet (the lane's first pass, measured)
    let c = case();
    let curve = c.curve.encode().unwrap();
    let a = c.ask(&curve);
    let o = offer(&a).unwrap();
    let pair = nft::mint_instructions(&c.curve, &c.permit, c.signature, FACE, o.price).unwrap();
    let budget = |tag: u8, v: &[u8]| Instruction { program_id: COMPUTE_BUDGET, accounts: vec![], data: [&[tag][..], v].concat() };
    let transfer = Instruction { program_id: Pubkey::default(), accounts: vec![AccountMeta::new(a.fee, true), AccountMeta::new(a.buyer, false)], data: [&[2u8, 0, 0, 0][..], &o.lamports.to_le_bytes()].concat() };
    let with = |ixs: &[Instruction]| 1 + 2 * 64 + Message::new_with_blockhash(ixs, Some(&a.fee), &Hash::new_from_array(a.blockhash)).serialize().len();
    let [p, m] = pair;
    assert_eq!(with(&[budget(2, &120_000u32.to_le_bytes()), budget(3, &PRICE.to_le_bytes()), transfer.clone(), p.clone(), m.clone()]), 1_236);
    assert!(1_236 > PACKET);
    assert_eq!(with(&[budget(3, &PRICE.to_le_bytes()), transfer, p, m]), 1 + 2 * 64 + o.message.len(), "the offer is the design without its unit limit");
}
