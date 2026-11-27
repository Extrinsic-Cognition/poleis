//! THE TOKEN'S FACE (Nov 21 2026, lane/the-launch-press): $WARM's name, symbol and picture as Metaplex's Token Metadata program holds them,
//! the face every wallet and explorer shows. `init` makes it by CPI when it is given one (the mine its mint authority and its update
//! authority, so no key ever renames the token), and the AUTHORITY's `set_face` moves its URI later (the picture moves with a site deploy;
//! the name and the symbol never move). A mine opened with no face (a chain with no Metaplex, as the local rehearsal's) has none, ever:
//! `set_face` moves a face, it makes none.
//!
//! The two instructions are written here by hand as small byte layouts, on the stack (no Metaplex crate is in the program, and none is
//! fetched): Borsh as Metaplex's SDK writes it, a string as its u32 length and its bytes, an option as 0 or 1 and its value. The press's
//! wire test holds both, byte for byte and account by account, to Metaplex's own SDK (mpl-token-metadata 5.1.1's generated builders:
//! poleis/press/tests/metaplex.rs), and `tests/face.rs` holds what the program sends by CPI to these encoders, on Mollusk, with a stand-in
//! at Metaplex's id that records what it is asked.

use crate::error::WarmError;
use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

/// Metaplex's Token Metadata program, `metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`.
pub const METADATA_PROGRAM_ID: Address = solana_address::address!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
/// A mint's metadata: the PDA of `[b"metadata", METADATA_PROGRAM_ID, mint]` under Metaplex's program.
pub const SEED_METADATA: &[u8] = b"metadata";
/// Metaplex's own bounds on a face (mpl-token-metadata's MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH).
pub const MAX_NAME: usize = 32;
pub const MAX_SYMBOL: usize = 10;
pub const MAX_URI: usize = 200;
/// `CreateMetadataAccountV3` and `UpdateMetadataAccountV2`, by their one-byte discriminators.
pub const IX_CREATE_METADATA_V3: u8 = 33;
pub const IX_UPDATE_METADATA_V2: u8 = 15;
/// CreateMetadataAccountV3's accounts as (writable, signer): metadata (w), mint, mint authority (s), payer (s, w), update authority (s),
/// the system program (no rent account: Metaplex's SDK leaves it out when none is given).
pub const CREATE_ACCOUNTS: [(bool, bool); 6] = [(true, false), (false, false), (false, true), (true, true), (false, true), (false, false)];
/// UpdateMetadataAccountV2's accounts: metadata (w), update authority (s).
pub const UPDATE_ACCOUNTS: [(bool, bool); 2] = [(true, false), (false, true)];
/// The longest instruction data either writes: a tag, an option's byte, DataV2 at its bounds (three strings with their lengths, the fee,
/// three empty options) and three bytes after it.
pub const DATA_MAX: usize = 2 + (4 + MAX_NAME) + (4 + MAX_SYMBOL) + (4 + MAX_URI) + 2 + 3 + 3;
/// A Metadata account's first byte: `Key::MetadataV1`.
pub const KEY_METADATA_V1: u8 = 4;
/// Where a Metadata account's name begins: after its key (1), its update authority (32) and its mint (32).
pub const OFF_NAME: usize = 65;

/// A face: the token's name, its symbol and the URI of its picture's JSON, each within Metaplex's bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face<'a> {
    pub name: &'a [u8],
    pub symbol: &'a [u8],
    pub uri: &'a [u8],
}

/// One length-prefixed field: [len:1][bytes].
fn field<'a>(d: &'a [u8], at: &mut usize, most: usize) -> Result<&'a [u8], ProgramError> {
    let n = *d.get(*at).ok_or(ProgramError::InvalidInstructionData)? as usize;
    if n > most {
        return Err(ProgramError::InvalidInstructionData);
    }
    let s = d.get(*at + 1..*at + 1 + n).ok_or(ProgramError::InvalidInstructionData)?;
    *at += 1 + n;
    Ok(s)
}

/// The face as `init` takes it after its 40 bytes: [name_len:1][name][symbol_len:1][symbol][uri_len:1][uri], and nothing after. A face
/// with no name or no symbol is no face.
pub fn parse_face(d: &[u8]) -> Result<Face<'_>, ProgramError> {
    let mut at = 0;
    let name = field(d, &mut at, MAX_NAME)?;
    let symbol = field(d, &mut at, MAX_SYMBOL)?;
    let uri = field(d, &mut at, MAX_URI)?;
    if at != d.len() || name.is_empty() || symbol.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(Face { name, symbol, uri })
}

/// The URI as `set_face` takes it: [uri_len:1][uri], and nothing after.
pub fn parse_uri(d: &[u8]) -> Result<&[u8], ProgramError> {
    let mut at = 0;
    let uri = field(d, &mut at, MAX_URI)?;
    if at != d.len() {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(uri)
}

/// A Borsh string: its u32 length, little-endian, then its bytes.
fn put_str(out: &mut [u8; DATA_MAX], at: usize, s: &[u8]) -> usize {
    out[at..at + 4].copy_from_slice(&(s.len() as u32).to_le_bytes());
    out[at + 4..at + 4 + s.len()].copy_from_slice(s);
    at + 4 + s.len()
}

/// DataV2 { name, symbol, uri, seller_fee_basis_points: 0, creators: None, collection: None, uses: None }: no royalty, no creator, no
/// collection, no uses, ever.
fn put_data_v2(out: &mut [u8; DATA_MAX], at: usize, f: &Face) -> usize {
    let at = put_str(out, at, f.name);
    let at = put_str(out, at, f.symbol);
    let at = put_str(out, at, f.uri);
    // seller_fee_basis_points (u16) 0, then the three options' None
    out[at..at + 5].fill(0);
    at + 5
}

/// `CreateMetadataAccountV3`'s data: [33] DataV2, is_mutable true (so the authority's `set_face` can move the URI), collection_details
/// None. Returns its length in `out`.
pub fn create_data(f: &Face, out: &mut [u8; DATA_MAX]) -> usize {
    out[0] = IX_CREATE_METADATA_V3;
    let at = put_data_v2(out, 1, f);
    out[at] = 1;
    out[at + 1] = 0;
    at + 2
}

/// `UpdateMetadataAccountV2`'s data: [15] Some(DataV2), new_update_authority None, primary_sale_happened None, is_mutable None. Returns
/// its length in `out`.
pub fn update_data(f: &Face, out: &mut [u8; DATA_MAX]) -> usize {
    out[0] = IX_UPDATE_METADATA_V2;
    out[1] = 1;
    let at = put_data_v2(out, 2, f);
    out[at..at + 3].fill(0);
    at + 3
}

/// A mint's metadata address, by its seeds under Metaplex's program.
pub fn metadata_address(mint: &Address) -> Address {
    Address::find_program_address(&[SEED_METADATA, METADATA_PROGRAM_ID.as_ref(), mint.as_ref()], &METADATA_PROGRAM_ID).0
}

/// A Borsh string read at `at`: its bytes with the zeros Metaplex pads a stored name and symbol with taken off the end.
fn read_str(d: &[u8], at: usize, most: usize) -> Result<(&[u8], usize), ProgramError> {
    let n = d.get(at..at + 4).and_then(|s| s.try_into().ok()).map(u32::from_le_bytes).ok_or(WarmError::InvalidAccount)? as usize;
    if n > most {
        return Err(WarmError::InvalidAccount.into());
    }
    let s = d.get(at + 4..at + 4 + n).ok_or(WarmError::InvalidAccount)?;
    let mut end = s.len();
    while end > 0 && s[end - 1] == 0 {
        end -= 1;
    }
    Ok((&s[..end], at + 4 + n))
}

/// What `set_face` reads of a Metadata account Metaplex wrote: [key:1 = 4][update_authority:32][mint:32][name][symbol], and checks: its
/// key, its update authority (the mine) and its mint (the mine's). Returns the name and the symbol, which never move.
pub fn read_face<'a>(d: &'a [u8], mine: &Address, mint: &Address) -> Result<(&'a [u8], &'a [u8]), ProgramError> {
    if d.len() < OFF_NAME || d[0] != KEY_METADATA_V1 || &d[1..33] != mine.as_ref() || &d[33..65] != mint.as_ref() {
        return Err(WarmError::InvalidAccount.into());
    }
    let (name, at) = read_str(d, OFF_NAME, MAX_NAME)?;
    let (symbol, _) = read_str(d, at, MAX_SYMBOL)?;
    Ok((name, symbol))
}

/// The face made by CPI: Metaplex's CreateMetadataAccountV3, the mine signing as the mint's authority and as the face's update authority,
/// the payer paying its rent.
pub fn create(metadata: &AccountView, mint: &AccountView, mine: &AccountView, payer: &AccountView, system: &AccountView, f: &Face, signers: &[Signer]) -> ProgramResult {
    let mut data = [0u8; DATA_MAX];
    let n = create_data(f, &mut data);
    let views: [&AccountView; 6] = [metadata, mint, mine, payer, mine, system];
    let metas: [InstructionAccount; 6] = core::array::from_fn(|i| InstructionAccount::new(views[i].address(), CREATE_ACCOUNTS[i].0, CREATE_ACCOUNTS[i].1));
    let ix = InstructionView { program_id: &METADATA_PROGRAM_ID, accounts: &metas, data: &data[..n] };
    invoke_signed::<6, &AccountView>(&ix, &views, signers)
}

/// The face's URI moved by CPI: Metaplex's UpdateMetadataAccountV2, the mine signing as its update authority.
pub fn update(metadata: &AccountView, mine: &AccountView, f: &Face, signers: &[Signer]) -> ProgramResult {
    let mut data = [0u8; DATA_MAX];
    let n = update_data(f, &mut data);
    let views: [&AccountView; 2] = [metadata, mine];
    let metas: [InstructionAccount; 2] = core::array::from_fn(|i| InstructionAccount::new(views[i].address(), UPDATE_ACCOUNTS[i].0, UPDATE_ACCOUNTS[i].1));
    let ix = InstructionView { program_id: &METADATA_PROGRAM_ID, accounts: &metas, data: &data[..n] };
    invoke_signed::<2, &AccountView>(&ix, &views, signers)
}
