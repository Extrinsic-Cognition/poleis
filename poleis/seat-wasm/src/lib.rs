//! poleis-seat-wasm · THE FLASK TAKES ITS SEAT (Nov 22 2026, lane/the-flask-takes-its-seat; his 17:07:40 UTC "Yes, build it" to "Next: the page lets
//! a finished Lyssajudron take its seat with one tap of the buyer's wallet?", over THE DOOR SIGNS THE SEAT and his Nov 19 12:37 "each lyssajudron minted
//! is a hex there"). A finished Lyssajudron's seat of root 0's floor, bought by the person's own flask, its SOL the house's: the mint door
//! (embertide-app/ui/intro/dyson-society/worker/space-router-seat.mjs) asks this crate, compiled once to WASM (worker/poleis-seat.wasm, by build.sh), for
//! the seat's one transaction; the flask signs it in the browser, the door co-signs it as the fee's payer and sends it.
//!
//! - `offer`: from root 0's curve account as the chain holds it, the door's permit and its issuer's signature, the face, the buyer (the flask the permit
//!   names), the house's fee key, the rent of 0 and of 1 byte, the blockhash, the priority price and the slot, ONE legacy message, compiled by Solana's
//!   own compiler: SetComputeUnitPrice; a system transfer from the fee key to the flask of exactly the seat's lamports; the precompile's instruction and
//!   `mint_nft`, the client's own builders (no second copy of the law), the buyer's ceiling the curve's price now. The fee key pays the fee and signs
//!   first, the flask second. No SetComputeUnitLimit: with it the message is 1,236 bytes, over Solana's 1,232 (measured on the lane's first pass), so
//!   the priority price is paid on the runtime's default limit (DEFAULT_UNITS).
//! - The seat's lamports: the curve's price now, the rents of what `mint_nft` makes and its buyer pays for (the seat's receipt, the hook's validation,
//!   the mint at its metadata's length, the first token account, one more row of root 0's registry), and the flask's allowance (a rent-exempt empty
//!   account and VOTES votes' fees), all from the rent the door reads. The fee besides is the fee key's: SIGNATURES signatures and the priority.
//! - The Poleis it speaks is the one it was built against (the client's PROGRAM_ID): the tree's for the tests and the vector, a sitting's by `press
//!   build` (from the sitting's copy, its sha256 recorded); the door reads it (`seat_program`) and answers 503 when it is not POLEIS_PROGRAM.
//! - `ask_of` and `answer` are the doors' flat layouts (src/doors.rs, wasm32 only: `#[no_mangle] extern "C"`, no bindgen, as embertide-mint's).
use pinocchio::error::ProgramError;
use poleis_client::nft::{self, BuildError, Metadata, MintCurve, PERMIT_LEN};
use poleis_client::{PoleisError, Registry, ROOT_UNIT};
use poleis_treasury::{completion_permit, nft_seat_state::NftSeat, nft_wire};
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;

#[cfg(target_arch = "wasm32")]
mod doors;
pub mod mine;

/// The Poleis this build speaks: the client's, baked in.
pub const PROGRAM: Pubkey = poleis_client::PROGRAM_ID;
/// The flask's first votes whose fees its allowance carries: a vote is one transaction the flask signs and pays, one signature.
pub const VOTES: u64 = 10;
/// Solana's fee a signature, lamports.
pub const SIGNATURE_LAMPORTS: u64 = 5_000;
/// The signatures Solana charges the seat's transaction: the fee key's, the flask's, and the issuer's inside the precompile.
pub const SIGNATURES: u64 = 3;
/// The units the runtime gives a transaction that asks none (SIMD-0170): 3,000 an instruction of a builtin (the compute budget's, the transfer, the
/// precompile) and 200,000 an instruction of a program (`mint_nft`); the priority price is paid on them.
pub const DEFAULT_UNITS: u64 = 3 * 3_000 + 200_000;
/// Solana's packet: a transaction's whole wire, its signatures with it.
pub const PACKET: usize = 1_232;
/// The compute budget's program.
pub const COMPUTE_BUDGET: Pubkey = solana_pubkey::pubkey!("ComputeBudget111111111111111111111111111111");
/// An ask's bytes before its face: the curve's account, the permit, the issuer's signature, the buyer, the fee key, the blockhash and four numbers.
pub const ASK_FIXED: usize = MintCurve::SIZE + PERMIT_LEN + 64 + 3 * 32 + 4 * 8;
/// An answer's bytes before its message: five numbers and the mint.
pub const ANSWER_HEAD: usize = 5 * 8 + 32;

/// What the door gives, as the doors' ask carries it (`ask_of`).
#[derive(Clone, Copy, Debug)]
pub struct Ask<'a> {
    /// Root 0's curve account's bytes as the chain holds them.
    pub curve: &'a [u8],
    /// The door's permit and its issuer's signature over it.
    pub permit: [u8; PERMIT_LEN],
    pub signature: [u8; 64],
    /// The face the permit binds by its digest.
    pub face: Metadata<'a>,
    /// The flask: the permit's buyer, the seat's.
    pub buyer: Pubkey,
    /// The house's fee key: it pays the fee and the seat's lamports.
    pub fee: Pubkey,
    /// The rent of an account of 0 and of 1 byte.
    pub rent: [u64; 2],
    pub blockhash: [u8; 32],
    /// The priority price, micro-lamports a unit.
    pub price: u64,
    /// The slot the door's node read: the curve's window is held to it.
    pub slot: u64,
}

/// The seat's one transaction, for the flask to sign: its message; the lamports the fee key sends the flask and their parts; the fee besides; the mint
/// root 0 names the seat by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub message: Vec<u8>,
    pub lamports: u64,
    pub price: u64,
    pub rents: u64,
    pub allowance: u64,
    pub fee: u64,
    pub mint: Pubkey,
}

/// Why no offer; the doors give its number negated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Root 0's curve does not read as one.
    Curve = 1,
    /// Its window has not opened.
    NotYet = 2,
    /// Its window is shut.
    Shut = 3,
    /// Every seat of it is minted.
    Full = 4,
    /// A permit, a face, a buyer or a fee key the law refuses (another Poleis's permit among them), or an ask that does not read.
    Permit = 5,
    /// A rent or a sum past a u64.
    Rent = 6,
    /// Past Solana's packet.
    Size = 7,
}

/// The rent of an account of `len` bytes, from the rent of 0 and of 1 byte: Solana's rent is linear in an account's length.
pub fn rent_of(rent: [u64; 2], len: usize) -> Option<u64> {
    rent[1].checked_sub(rent[0])?.checked_mul(u64::try_from(len).ok()?)?.checked_add(rent[0])
}

/// The mint root 0 names the seat of a deposit's completion by.
pub fn mint_of(completion: &[u8; 32]) -> Pubkey {
    nft::mint_pda(ROOT_UNIT, completion)
}

/// The seat's one transaction for the flask to sign, by the law.
pub fn offer(a: &Ask<'_>) -> Result<Offer, Refusal> {
    let curve = MintCurve::decode(a.curve).map_err(|_| Refusal::Curve)?;
    let code = |e: PoleisError| ProgramError::Custom(e as u32);
    // the curve's own quote at the slot the door read: its window, its seats, its price now
    let price = match curve.quote(a.slot, u64::MAX) {
        Ok(q) => q.price_lamports,
        Err(e) if e == code(PoleisError::NotYet) => return Err(Refusal::NotYet),
        Err(e) if e == code(PoleisError::CommitWindowClosed) => return Err(Refusal::Shut),
        Err(e) if e == code(PoleisError::NoSpots) => return Err(Refusal::Full),
        Err(_) => return Err(Refusal::Curve),
    };
    let fields = completion_permit::parse(&a.permit).map_err(|_| Refusal::Permit)?;
    if fields.buyer != a.buyer.to_bytes() || a.fee == a.buyer {
        return Err(Refusal::Permit);
    }
    let [precompile, mint_nft] = nft::mint_instructions(&curve, &a.permit, a.signature, a.face, price).map_err(|e| match e {
        BuildError::TransactionTooLarge(_) => Refusal::Size,
        _ => Refusal::Permit,
    })?;
    // what `mint_nft` makes and its buyer pays for: the seat's receipt, the hook's validation, the mint at its metadata's length, the first token
    // account, and one more row of root 0's registry (its growth at the rent of a byte)
    let mint_len = nft_wire::metadata_mint_len(a.face).map_err(|_| Refusal::Permit)?;
    let row = a.rent[1].checked_sub(a.rent[0]).and_then(|b| b.checked_mul(Registry::HOLDER_LEN as u64)).ok_or(Refusal::Rent)?;
    let rents = [NftSeat::SIZE, nft_wire::EXTRA_ACCOUNT_METAS_LEN, mint_len, nft_wire::TOKEN_ACCOUNT_LEN]
        .into_iter()
        .try_fold(row, |sum, len| sum.checked_add(rent_of(a.rent, len)?))
        .ok_or(Refusal::Rent)?;
    // the flask keeps a rent-exempt empty account and its first votes' fees
    let allowance = rent_of(a.rent, 0).and_then(|r| r.checked_add(VOTES * SIGNATURE_LAMPORTS)).ok_or(Refusal::Rent)?;
    let lamports = price.checked_add(rents).and_then(|l| l.checked_add(allowance)).ok_or(Refusal::Rent)?;
    let priority = u64::try_from((u128::from(DEFAULT_UNITS) * u128::from(a.price)).div_ceil(1_000_000)).map_err(|_| Refusal::Rent)?;
    let fee = priority.checked_add(SIGNATURES * SIGNATURE_LAMPORTS).ok_or(Refusal::Rent)?;
    let ixs = [
        Instruction { program_id: COMPUTE_BUDGET, accounts: vec![], data: [&[3u8][..], &a.price.to_le_bytes()].concat() },
        Instruction { program_id: Pubkey::default(), accounts: vec![AccountMeta::new(a.fee, true), AccountMeta::new(a.buyer, false)], data: [&[2u8, 0, 0, 0][..], &lamports.to_le_bytes()].concat() },
        precompile,
        mint_nft,
    ];
    let m = Message::new_with_blockhash(&ixs, Some(&a.fee), &Hash::new_from_array(a.blockhash));
    if m.header.num_required_signatures != 2 || m.account_keys[..2] != [a.fee, a.buyer] {
        return Err(Refusal::Permit);
    }
    let message = m.serialize();
    if 1 + 2 * 64 + message.len() > PACKET {
        return Err(Refusal::Size);
    }
    Ok(Offer { message, lamports, price, rents, allowance, fee, mint: mint_of(&fields.completion_digest) })
}

/// The doors' ask: ASK_FIXED bytes (the curve's account, the permit, the issuer's signature, the buyer, the fee key, the blockhash; the rent of 0 and
/// of 1 byte, the price and the slot, u64 LE each), then the face's name, symbol and URI, each a u32 LE length and its UTF-8, and nothing after.
pub fn ask_of(b: &[u8]) -> Option<Ask<'_>> {
    let (fixed, mut face) = (b.get(..ASK_FIXED)?, b.get(ASK_FIXED..)?);
    let (curve, rest) = fixed.split_at(MintCurve::SIZE);
    let (permit, rest) = rest.split_at(PERMIT_LEN);
    let (signature, rest) = rest.split_at(64);
    let key = |i: usize| Pubkey::new_from_array(rest[32 * i..32 * i + 32].try_into().unwrap_or([0; 32]));
    let n = |i: usize| u64::from_le_bytes(rest[96 + 8 * i..104 + 8 * i].try_into().unwrap_or([0; 8]));
    let (name, symbol, uri) = (text(&mut face)?, text(&mut face)?, text(&mut face)?);
    if !face.is_empty() {
        return None;
    }
    Some(Ask {
        curve,
        permit: permit.try_into().ok()?,
        signature: signature.try_into().ok()?,
        face: Metadata { name, symbol, uri },
        buyer: key(0),
        fee: key(1),
        blockhash: key(2).to_bytes(),
        rent: [n(0), n(1)],
        price: n(2),
        slot: n(3),
    })
}

/// One Borsh string of the face: a u32 LE length and its UTF-8.
fn text<'a>(b: &mut &'a [u8]) -> Option<&'a str> {
    let n = usize::try_from(u32::from_le_bytes(b.get(..4)?.try_into().ok()?)).ok()?;
    let s = core::str::from_utf8(b.get(4..4usize.checked_add(n)?)?).ok()?;
    *b = &b[4 + n..];
    Some(s)
}

/// An ask's bytes, as the door lays them (`ask_of` reads them back).
pub fn ask_bytes(a: &Ask<'_>) -> Vec<u8> {
    let mut b = [a.curve, &a.permit, &a.signature, a.buyer.as_ref(), a.fee.as_ref(), &a.blockhash].concat();
    for n in [a.rent[0], a.rent[1], a.price, a.slot] {
        b.extend_from_slice(&n.to_le_bytes());
    }
    for s in [a.face.name, a.face.symbol, a.face.uri] {
        b.extend_from_slice(&(s.len() as u32).to_le_bytes());
        b.extend_from_slice(s.as_bytes());
    }
    b
}

/// The doors' answer: the lamports, the price, the rents, the allowance and the fee (u64 LE each), the mint, then the message.
pub fn answer(o: &Offer) -> Vec<u8> {
    [&o.lamports.to_le_bytes()[..], &o.price.to_le_bytes(), &o.rents.to_le_bytes(), &o.allowance.to_le_bytes(), &o.fee.to_le_bytes(), o.mint.as_ref(), &o.message].concat()
}
