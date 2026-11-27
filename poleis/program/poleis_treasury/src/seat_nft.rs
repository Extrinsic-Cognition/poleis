//! Allocation-free validation of the deliberately narrow seat-NFT token profile.
//!
//! Reads raw Token-2022 bytes; no caller-supplied decoded token fields are trusted.
//! `RawAccount` metadata and signer status MUST come from runtime account views,
//! never from instruction data or an RPC response. The caller must authenticate
//! the canonical mint against its seat PDA and the expected hook against this
//! program, then validate the live registry seat and apply its governance rules.
//! These functions do not establish admission, execute a hook, or mutate state.
//!
//! Wire references, checked 2026-10-13 against solana-program/token-2022 commit
//! 6a72b1658e0d61732012aae8544cebcce8774597:
//! - interface/src/state.rs: Mint (82 bytes), Account (165), COption tags.
//! - interface/src/extension/mod.rs: type byte at 165, TLV from 166, extension IDs.
//! - interface/src/extension/{transfer_hook/mod.rs,immutable_owner.rs}: layouts.
//! - interface/src/extension/{metadata_pointer,token_metadata}/mod.rs: metadata
//!   is an optional pair, with a 64-byte pointer and Borsh metadata payload.
//! - program/src/processor.rs: balances change BEFORE both transferring flags
//!   are set and the hook executes; ImmutableOwner rejects owner replacement.
//! https://github.com/solana-program/token-2022/tree/6a72b1658e0d61732012aae8544cebcce8774597
//! Metadata payload: solana-program/token-metadata commit
//! 9f54923e51b0c3e3d408dac70f45d36d07d2150a, interface/src/state.rs:
//! https://github.com/solana-program/token-metadata/blob/9f54923e51b0c3e3d408dac70f45d36d07d2150a/interface/src/state.rs
//!
//! Accepted profile: TransferHook on the mint, optionally paired with immutable
//! self-pointing MetadataPointer and TokenMetadata; only ImmutableOwner and
//! TransferHookAccount on token accounts. Unknown, duplicate, fee,
//! permanent-delegate, confidential, close-authority and other extensions are
//! rejected, not skipped. Metadata strings have application byte limits and no
//! additional map entries; validating a URI never fetches or authenticates its
//! external contents.
//! Mint/freeze/hook-update authorities must be revoked. An ordinary delegate
//! may spend the token, but never qualifies as its governance owner.
//! Burn reconciliation, registry membership, extra-account-meta/PDA validation,
//! instruction dispatch and transaction-level atomicity remain caller work.

pub type Key = [u8; 32];

/// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb (not classic SPL Token).
pub const TOKEN_2022_OWNER: Key = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];
pub const MINT_BASE_LEN: usize = 82;
pub const TOKEN_BASE_LEN: usize = 165;
pub const TYPE_OFFSET: usize = TOKEN_BASE_LEN;
pub const TLV_OFFSET: usize = 166;
pub const IMMUTABLE_OWNER: u16 = 7;
pub const TRANSFER_HOOK: u16 = 14;
pub const TRANSFER_HOOK_ACCOUNT: u16 = 15;
pub const METADATA_POINTER: u16 = 18;
pub const TOKEN_METADATA: u16 = 19;
/// Application profile limits in UTF-8 bytes, shared with the mint builder.
/// These fields must be nonempty; no URI scheme is imposed by the wire parser.
pub const MAX_NAME_BYTES: usize = 32;
pub const MAX_SYMBOL_BYTES: usize = 10;
pub const MAX_URI_BYTES: usize = 200;
pub const MINT_PROFILE_LEN: usize = TLV_OFFSET + 4 + 64;
pub const TOKEN_PROFILE_LEN: usize = TLV_OFFSET + 4 + 4 + 1;
/// First eight SHA-256 bytes of `spl-transfer-hook-interface:execute`.
pub const EXECUTE_DISCRIMINATOR: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

/// Borrowed raw account data and runtime metadata; this is not an owner proof.
#[derive(Clone, Copy)]
pub struct RawAccount<'a> {
    pub address: &'a Key,
    pub program_owner: &'a Key,
    pub data: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeatNftError {
    WrongProgram,
    WrongMint,
    InvalidLength,
    InvalidAccountType,
    InvalidPadding,
    InvalidOption,
    NotInitialized,
    Frozen,
    InvalidSupply,
    InvalidDecimals,
    MutableAuthority,
    WrongHook,
    UnsupportedExtension,
    DuplicateExtension,
    MissingExtension,
    InvalidExtension,
    UnsupportedAuthority,
    MissingSignature,
    NotOwner,
    InvalidAmount,
    UnexpectedTransfer,
    NotTransferring,
    SameAccount,
    NotLiveSeat,
    SeatsResting,
    MathOverflow,
    InvalidInstruction,
}

/// Produced only after parsing and checking the current raw token state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerProof {
    mint: Key,
    owner: Key,
}

impl OwnerProof {
    pub fn mint(&self) -> &Key {
        &self.mint
    }
    pub fn owner(&self) -> &Key {
        &self.owner
    }
}

/// Observed token owners, not the possibly delegated transfer authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferOwners {
    from: Key,
    to: Key,
}

impl TransferOwners {
    pub fn from(&self) -> &Key {
        &self.from
    }
    pub fn to(&self) -> &Key {
        &self.to
    }
}

/// Strict hook ABI: discriminator followed by exactly one little-endian u64.
/// Decoding authenticates no accounts; always pair it with the raw hook proof.
pub fn parse_execute_amount(data: &[u8]) -> Result<u64, SeatNftError> {
    if data.len() != 16 || data[..8] != EXECUTE_DISCRIMINATOR {
        return Err(SeatNftError::InvalidInstruction);
    }
    Ok(u64_at(data, 8))
}

/// Verify ownership outside a token transfer. Invoke immediately before using
/// the seat; never persist this proof or reuse it after a CPI/ownership change.
pub fn verify_owner(
    mint: RawAccount<'_>,
    token: RawAccount<'_>,
    canonical_mint: &Key,
    expected_hook: &Key,
    signer: &Key,
    signer_is_signer: bool,
) -> Result<OwnerProof, SeatNftError> {
    validate_mint(mint, canonical_mint, expected_hook)?;
    let token = parse_token(token, canonical_mint)?;
    if token.transferring {
        return Err(SeatNftError::UnexpectedTransfer);
    }
    if token.amount != 1 {
        return Err(SeatNftError::InvalidAmount);
    }
    if !signer_is_signer {
        return Err(SeatNftError::MissingSignature);
    }
    if &token.owner != signer {
        return Err(SeatNftError::NotOwner);
    }
    Ok(OwnerProof {
        mint: *canonical_mint,
        owner: token.owner,
    })
}

/// Validate the raw POST-transfer state visible inside Token-2022's hook.
/// Token-2022 has already authenticated the transfer authority, which may be a
/// delegate; its signer privilege is not forwarded to this hook. Do not require
/// it here or treat it as the seat owner. Both flags prevent direct-call forgery
/// when these bytes and owners are taken from the actual runtime accounts.
pub fn verify_transfer_hook(
    mint: RawAccount<'_>,
    source: RawAccount<'_>,
    destination: RawAccount<'_>,
    canonical_mint: &Key,
    expected_hook: &Key,
    amount: u64,
) -> Result<TransferOwners, SeatNftError> {
    validate_mint(mint, canonical_mint, expected_hook)?;
    if source.address == destination.address {
        return Err(SeatNftError::SameAccount);
    }
    let source = parse_token(source, canonical_mint)?;
    let destination = parse_token(destination, canonical_mint)?;
    if !source.transferring || !destination.transferring {
        return Err(SeatNftError::NotTransferring);
    }
    if amount != 1 || source.amount != 0 || destination.amount != 1 {
        return Err(SeatNftError::InvalidAmount);
    }
    Ok(TransferOwners {
        from: source.owner,
        to: destination.owner,
    })
}

/// Pure guard for a single live seat's existing freeze/rest policy. Inputs must
/// come from the authenticated registry and Clock. This does not update activity
/// or weight, and ownership validation alone must never bypass this guard.
pub fn check_transfer_policy(
    frozen: bool,
    seats: u64,
    last_active: u64,
    window: u64,
    now: u64,
) -> Result<(), SeatNftError> {
    if frozen {
        return Err(SeatNftError::Frozen);
    }
    if seats != 1 {
        return Err(SeatNftError::NotLiveSeat);
    }
    // Fail closed on overflow rather than granting movement at u64::MAX.
    let unlock = last_active
        .checked_add(window)
        .ok_or(SeatNftError::MathOverflow)?;
    if now < unlock {
        return Err(SeatNftError::SeatsResting);
    }
    Ok(())
}

fn require_program(account: RawAccount<'_>) -> Result<(), SeatNftError> {
    if account.program_owner != &TOKEN_2022_OWNER {
        return Err(SeatNftError::WrongProgram);
    }
    Ok(())
}

fn base(data: &[u8], account_type: u8) -> Result<(), SeatNftError> {
    // Token-2022 explicitly excludes the legacy multisig length (355).
    if data.len() < TLV_OFFSET || data.len() == 355 {
        return Err(SeatNftError::InvalidLength);
    }
    if data[TYPE_OFFSET] != account_type {
        return Err(SeatNftError::InvalidAccountType);
    }
    Ok(())
}

fn u64_at(data: &[u8], offset: usize) -> u64 {
    // Called only at fixed base-layout offsets after the minimum-length check.
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

fn key_at(data: &[u8], offset: usize) -> Key {
    data[offset..offset + 32].try_into().unwrap()
}

/// COption uses a four-byte tag. None deliberately ignores the old payload:
/// Token-2022's packer only overwrites the tag when an authority is revoked.
fn option_present(data: &[u8], offset: usize) -> Result<bool, SeatNftError> {
    match &data[offset..offset + 4] {
        [0, 0, 0, 0] => Ok(false),
        [1, 0, 0, 0] => Ok(true),
        _ => Err(SeatNftError::InvalidOption),
    }
}

/// Validate the canonical mint's entire supported profile, including runtime
/// program ownership and its immutable hook. This does not prove token ownership.
pub fn validate_mint(
    mint: RawAccount<'_>,
    canonical_mint: &Key,
    expected_hook: &Key,
) -> Result<(), SeatNftError> {
    validate_mint_supply(mint, canonical_mint, expected_hook, 1)
}

/// Validate an irreversibly burned mint: the exact same immutable profile as
/// `validate_mint`, but with supply exactly zero. This does not prove who burned
/// it, authenticate its seat receipt, or remove any registry membership.
pub fn validate_burned_mint(
    mint: RawAccount<'_>,
    canonical_mint: &Key,
    expected_hook: &Key,
) -> Result<(), SeatNftError> {
    validate_mint_supply(mint, canonical_mint, expected_hook, 0)
}

fn validate_mint_supply(
    mint: RawAccount<'_>,
    canonical_mint: &Key,
    expected_hook: &Key,
    supply: u64,
) -> Result<(), SeatNftError> {
    require_program(mint)?;
    if mint.address != canonical_mint || canonical_mint == &[0; 32] {
        return Err(SeatNftError::WrongMint);
    }
    let d = mint.data;
    base(d, 1)?;
    if d[MINT_BASE_LEN..TYPE_OFFSET].iter().any(|b| *b != 0) {
        return Err(SeatNftError::InvalidPadding);
    }
    if option_present(d, 0)? || option_present(d, 46)? {
        return Err(SeatNftError::MutableAuthority);
    }
    if d[45] != 1 {
        return Err(SeatNftError::NotInitialized);
    }
    if d[44] != 0 {
        return Err(SeatNftError::InvalidDecimals);
    }
    if u64_at(d, 36) != supply {
        return Err(SeatNftError::InvalidSupply);
    }
    let mut hook = false;
    let mut pointer = false;
    let mut metadata = false;
    for_each_extension(d, |kind, value| {
        match kind {
            TRANSFER_HOOK => {
                if hook {
                    return Err(SeatNftError::DuplicateExtension);
                }
                hook = true;
                if value.len() != 64 {
                    return Err(SeatNftError::InvalidExtension);
                }
                // Nullable pubkeys use zero bytes, not the base COption tags.
                if value[..32].iter().any(|b| *b != 0) {
                    return Err(SeatNftError::MutableAuthority);
                }
                if expected_hook == &[0; 32] || &value[32..] != expected_hook {
                    return Err(SeatNftError::WrongHook);
                }
            }
            METADATA_POINTER => {
                if pointer {
                    return Err(SeatNftError::DuplicateExtension);
                }
                pointer = true;
                if value.len() != 64 {
                    return Err(SeatNftError::InvalidExtension);
                }
                if value[..32].iter().any(|b| *b != 0) {
                    return Err(SeatNftError::MutableAuthority);
                }
                if &value[32..] != canonical_mint {
                    return Err(SeatNftError::WrongMint);
                }
            }
            TOKEN_METADATA => {
                if metadata {
                    return Err(SeatNftError::DuplicateExtension);
                }
                metadata = true;
                validate_metadata(value, canonical_mint)?;
            }
            _ => return Err(SeatNftError::UnsupportedExtension),
        }
        Ok(())
    })?;
    if !hook || pointer != metadata {
        return Err(SeatNftError::MissingExtension);
    }
    Ok(())
}

fn validate_metadata(value: &[u8], canonical_mint: &Key) -> Result<(), SeatNftError> {
    if value.len() < 64 {
        return Err(SeatNftError::InvalidExtension);
    }
    if value[..32].iter().any(|b| *b != 0) {
        return Err(SeatNftError::MutableAuthority);
    }
    if &value[32..64] != canonical_mint {
        return Err(SeatNftError::WrongMint);
    }
    // Borsh strings have a u32 byte length, followed by exactly that UTF-8 data.
    // Bound before slicing or decoding; no allocation depends on raw lengths.
    let mut rest = &value[64..];
    for max in [MAX_NAME_BYTES, MAX_SYMBOL_BYTES, MAX_URI_BYTES] {
        let prefix = rest.get(..4).ok_or(SeatNftError::InvalidExtension)?;
        let len = u32::from_le_bytes(prefix.try_into().unwrap()) as usize;
        rest = &rest[4..];
        if len == 0 || len > max {
            return Err(SeatNftError::InvalidExtension);
        }
        let text = rest.get(..len).ok_or(SeatNftError::InvalidExtension)?;
        core::str::from_utf8(text).map_err(|_| SeatNftError::InvalidExtension)?;
        rest = &rest[len..];
    }
    // Exactly one empty Borsh Vec<(String, String)> count, with no extra bytes.
    if rest != [0, 0, 0, 0] {
        return Err(SeatNftError::InvalidExtension);
    }
    Ok(())
}

/// Decoded only from the complete checked token profile. Does not by itself
/// authenticate its mint, a signer, live seat membership or transfer policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParsedTokenAccount {
    owner: Key,
    amount: u64,
    transferring: bool,
}

impl ParsedTokenAccount {
    pub fn owner(&self) -> &Key {
        &self.owner
    }
    pub fn amount(&self) -> u64 {
        self.amount
    }
    pub fn transferring(&self) -> bool {
        self.transferring
    }
}

pub fn parse_token(
    token: RawAccount<'_>,
    canonical_mint: &Key,
) -> Result<ParsedTokenAccount, SeatNftError> {
    require_program(token)?;
    let d = token.data;
    base(d, 2)?;
    if &d[..32] != canonical_mint {
        return Err(SeatNftError::WrongMint);
    }
    let owner = key_at(d, 32);
    if owner == [0; 32] {
        return Err(SeatNftError::UnsupportedAuthority);
    }
    match d[108] {
        1 => {}
        2 => return Err(SeatNftError::Frozen),
        _ => return Err(SeatNftError::NotInitialized),
    }
    let delegated = option_present(d, 72)?;
    let delegated_amount = u64_at(d, 121);
    if (delegated && key_at(d, 76) == [0; 32]) || (!delegated && delegated_amount != 0) {
        return Err(SeatNftError::UnsupportedAuthority);
    }
    if option_present(d, 109)? || option_present(d, 129)? {
        return Err(SeatNftError::UnsupportedAuthority);
    }
    let mut immutable = false;
    let mut hook = None;
    for_each_extension(d, |kind, value| {
        match kind {
            IMMUTABLE_OWNER => {
                if immutable {
                    return Err(SeatNftError::DuplicateExtension);
                }
                if !value.is_empty() {
                    return Err(SeatNftError::InvalidExtension);
                }
                immutable = true;
            }
            TRANSFER_HOOK_ACCOUNT => {
                if hook.is_some() {
                    return Err(SeatNftError::DuplicateExtension);
                }
                hook = Some(match value {
                    [0] => false,
                    [1] => true,
                    _ => return Err(SeatNftError::InvalidExtension),
                });
            }
            _ => return Err(SeatNftError::UnsupportedExtension),
        }
        Ok(())
    })?;
    if !immutable {
        return Err(SeatNftError::MissingExtension);
    }
    Ok(ParsedTokenAccount {
        owner,
        amount: u64_at(d, 64),
        transferring: hook.ok_or(SeatNftError::MissingExtension)?,
    })
}

fn for_each_extension(
    data: &[u8],
    mut visit: impl FnMut(u16, &[u8]) -> Result<(), SeatNftError>,
) -> Result<(), SeatNftError> {
    let mut rest = &data[TLV_OFFSET..];
    while !rest.is_empty() {
        // Empty allocation/padding is allowed only when the entire tail is zero;
        // never hide an extension or junk after an Uninitialized type marker.
        if rest.len() < 2 || rest[..2] == [0, 0] {
            return if rest.iter().all(|b| *b == 0) {
                Ok(())
            } else {
                Err(SeatNftError::InvalidPadding)
            };
        }
        if rest.len() < 4 {
            return Err(SeatNftError::InvalidLength);
        }
        let kind = u16::from_le_bytes([rest[0], rest[1]]);
        let len = u16::from_le_bytes([rest[2], rest[3]]) as usize;
        let end = 4 + len; // u16 length cannot overflow usize on Solana or host.
        if end > rest.len() {
            return Err(SeatNftError::InvalidLength);
        }
        visit(kind, &rest[4..end])?;
        rest = &rest[end..];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const MINT: Key = [11; 32];
    const HOOK: Key = [12; 32];
    const ALICE: Key = [13; 32];
    const BOB: Key = [14; 32];
    const SOURCE: Key = [15; 32];
    const DEST: Key = [16; 32];

    fn raw<'a>(address: &'a Key, data: &'a [u8]) -> RawAccount<'a> {
        RawAccount {
            address,
            program_owner: &TOKEN_2022_OWNER,
            data,
        }
    }
    fn mint() -> [u8; 234] {
        let mut d = [0; 234];
        d[36..44].copy_from_slice(&1u64.to_le_bytes());
        d[45] = 1;
        d[165] = 1;
        d[166..170].copy_from_slice(&[14, 0, 64, 0]);
        d[202..234].copy_from_slice(&HOOK);
        d
    }
    fn mint_with_metadata(fields: [&[u8]; 3]) -> ([u8; 1024], usize) {
        let mut d = [0; 1024];
        d[..234].copy_from_slice(&mint());
        d[234..238].copy_from_slice(&[18, 0, 64, 0]);
        d[270..302].copy_from_slice(&MINT);
        d[302..304].copy_from_slice(&TOKEN_METADATA.to_le_bytes());
        d[338..370].copy_from_slice(&MINT);
        let mut at = 370;
        for text in fields {
            d[at..at + 4].copy_from_slice(&(text.len() as u32).to_le_bytes());
            at += 4;
            d[at..at + text.len()].copy_from_slice(text);
            at += text.len();
        }
        at += 4; // empty additional_metadata Vec length
        d[304..306].copy_from_slice(&((at - 306) as u16).to_le_bytes());
        (d, at)
    }
    fn token(owner: &Key, amount: u64, transferring: bool) -> [u8; 175] {
        let mut d = [0; 175];
        d[..32].copy_from_slice(&MINT);
        d[32..64].copy_from_slice(owner);
        d[64..72].copy_from_slice(&amount.to_le_bytes());
        d[108] = 1;
        d[165] = 2;
        d[166..175].copy_from_slice(&[7, 0, 0, 0, 15, 0, 1, 0, transferring as u8]);
        d
    }
    fn proof(m: &[u8], t: &[u8], signer: &Key) -> Result<OwnerProof, SeatNftError> {
        verify_owner(raw(&MINT, m), raw(&SOURCE, t), &MINT, &HOOK, signer, true)
    }

    #[test]
    fn immutable_self_metadata_accepts_utf8_byte_limits_and_either_tlv_order() {
        assert_eq!(validate_mint(raw(&MINT, &mint()), &MINT, &HOOK), Ok(()));
        let (d, len) = mint_with_metadata(["Dyson ✦".as_bytes(), b"SEAT", b"ipfs://example"]);
        assert_eq!(validate_mint(raw(&MINT, &d[..len]), &MINT, &HOOK), Ok(()));
        let mut reordered = [0; 1024];
        reordered[..166].copy_from_slice(&d[..166]);
        let metadata_len = len - 302;
        reordered[166..166 + metadata_len].copy_from_slice(&d[302..len]);
        reordered[166 + metadata_len..234 + metadata_len].copy_from_slice(&d[234..302]);
        reordered[234 + metadata_len..len].copy_from_slice(&d[166..234]);
        assert_eq!(
            validate_mint(raw(&MINT, &reordered[..len]), &MINT, &HOOK),
            Ok(())
        );
        let (bounded, len) = mint_with_metadata([
            &[b'n'; MAX_NAME_BYTES],
            &[b's'; MAX_SYMBOL_BYTES],
            &[b'u'; MAX_URI_BYTES],
        ]);
        assert_eq!(
            validate_mint(raw(&MINT, &bounded[..len]), &MINT, &HOOK),
            Ok(())
        );
    }

    #[test]
    fn metadata_rejects_mutable_authorities_external_pointers_and_mismatched_mints() {
        let (d, len) = mint_with_metadata([b"Name", b"SYM", b"uri"]);
        for offset in [238, 306] {
            let mut mutable = d;
            mutable[offset] = 1;
            assert_eq!(
                validate_mint(raw(&MINT, &mutable[..len]), &MINT, &HOOK),
                Err(SeatNftError::MutableAuthority)
            );
        }
        for offset in [270, 338] {
            for address in [[0; 32], BOB] {
                let mut wrong = d;
                wrong[offset..offset + 32].copy_from_slice(&address);
                assert_eq!(
                    validate_mint(raw(&MINT, &wrong[..len]), &MINT, &HOOK),
                    Err(SeatNftError::WrongMint)
                );
            }
        }
    }

    #[test]
    fn metadata_pair_and_hook_are_required_once_each_when_metadata_is_present() {
        let (d, len) = mint_with_metadata([b"Name", b"SYM", b"uri"]);
        assert_eq!(
            validate_mint(raw(&MINT, &d[..302]), &MINT, &HOOK),
            Err(SeatNftError::MissingExtension)
        );
        let mut no_pointer = d;
        no_pointer[234..len - 68].copy_from_slice(&d[302..len]);
        assert_eq!(
            validate_mint(raw(&MINT, &no_pointer[..len - 68]), &MINT, &HOOK),
            Err(SeatNftError::MissingExtension)
        );
        let mut no_hook = d;
        no_hook[166..len - 68].copy_from_slice(&d[234..len]);
        assert_eq!(
            validate_mint(raw(&MINT, &no_hook[..len - 68]), &MINT, &HOOK),
            Err(SeatNftError::MissingExtension)
        );
        for extension in [&d[234..302], &d[302..len]] {
            let mut duplicate = d;
            duplicate[len..len + extension.len()].copy_from_slice(extension);
            assert_eq!(
                validate_mint(
                    raw(&MINT, &duplicate[..len + extension.len()]),
                    &MINT,
                    &HOOK
                ),
                Err(SeatNftError::DuplicateExtension)
            );
        }
        let mut wrong_size = d;
        wrong_size[236] = 63;
        assert_eq!(
            validate_mint(raw(&MINT, &wrong_size[..len]), &MINT, &HOOK),
            Err(SeatNftError::InvalidExtension)
        );
    }

    #[test]
    fn metadata_rejects_malformed_strings_maps_lengths_and_every_partial_record() {
        let maxes = [MAX_NAME_BYTES, MAX_SYMBOL_BYTES, MAX_URI_BYTES];
        let oversized = [b'x'; MAX_URI_BYTES + 1];
        for field in 0..3 {
            for invalid in [&[][..], &[0xff][..], &oversized[..maxes[field] + 1]] {
                let mut fields: [&[u8]; 3] = [b"N", b"S", b"u"];
                fields[field] = invalid;
                let (d, len) = mint_with_metadata(fields);
                assert_eq!(
                    validate_mint(raw(&MINT, &d[..len]), &MINT, &HOOK),
                    Err(SeatNftError::InvalidExtension)
                );
            }
        }
        let (d, len) = mint_with_metadata([b"Name", b"SYM", b"uri"]);
        for end in 0..len {
            // A complete hook-only profile remains intentionally supported.
            if end != MINT_PROFILE_LEN {
                assert!(
                    validate_mint(raw(&MINT, &d[..end]), &MINT, &HOOK).is_err(),
                    "prefix {end}"
                );
            }
        }
        let mut wrong_length = d;
        wrong_length[370..374].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            validate_mint(raw(&MINT, &wrong_length[..len]), &MINT, &HOOK),
            Err(SeatNftError::InvalidExtension)
        );
        let mut map = d;
        map[len - 4..len].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(
            validate_mint(raw(&MINT, &map[..len]), &MINT, &HOOK),
            Err(SeatNftError::InvalidExtension)
        );
        let mut trailing = d;
        trailing[304..306].copy_from_slice(&((len - 306 + 1) as u16).to_le_bytes());
        assert_eq!(
            validate_mint(raw(&MINT, &trailing[..len + 1]), &MINT, &HOOK),
            Err(SeatNftError::InvalidExtension)
        );
    }

    #[test]
    fn metadata_preserves_current_owner_and_real_post_transfer_checks() {
        let (d, len) = mint_with_metadata([b"Name", b"SYM", b"uri"]);
        assert_eq!(
            proof(&d[..len], &token(&ALICE, 1, false), &ALICE)
                .unwrap()
                .owner(),
            &ALICE
        );
        let source = token(&ALICE, 0, true);
        let destination = token(&BOB, 1, true);
        let owners = verify_transfer_hook(
            raw(&MINT, &d[..len]),
            raw(&SOURCE, &source),
            raw(&DEST, &destination),
            &MINT,
            &HOOK,
            1,
        )
        .unwrap();
        assert_eq!(owners.from(), &ALICE);
        assert_eq!(owners.to(), &BOB);
    }

    #[test]
    fn burned_mint_requires_exact_zero_supply_and_the_same_immutable_profile() {
        let (mut d, len) = mint_with_metadata([b"Name", b"SYM", b"uri"]);
        for end in [MINT_PROFILE_LEN, len] {
            d[36..44].copy_from_slice(&1u64.to_le_bytes());
            assert_eq!(
                validate_burned_mint(raw(&MINT, &d[..end]), &MINT, &HOOK),
                Err(SeatNftError::InvalidSupply)
            );
            d[36..44].fill(0);
            assert_eq!(
                validate_burned_mint(raw(&MINT, &d[..end]), &MINT, &HOOK),
                Ok(())
            );
            assert_eq!(
                validate_mint(raw(&MINT, &d[..end]), &MINT, &HOOK),
                Err(SeatNftError::InvalidSupply)
            );
            for (offset, value, error) in [
                (0, 1, SeatNftError::MutableAuthority),
                (45, 0, SeatNftError::NotInitialized),
                (170, 1, SeatNftError::MutableAuthority),
                (202, 1, SeatNftError::WrongHook),
                (36, 2, SeatNftError::InvalidSupply),
            ] {
                let mut invalid = d;
                invalid[offset] = value;
                assert_eq!(
                    validate_burned_mint(raw(&MINT, &invalid[..end]), &MINT, &HOOK),
                    Err(error)
                );
            }
        }
        d[306] = 1;
        assert_eq!(
            validate_burned_mint(raw(&MINT, &d[..len]), &MINT, &HOOK),
            Err(SeatNftError::MutableAuthority)
        );
        d[306] = 0;
        let wrong_program = RawAccount {
            program_owner: &BOB,
            ..raw(&MINT, &d[..len])
        };
        assert_eq!(
            validate_burned_mint(wrong_program, &MINT, &HOOK),
            Err(SeatNftError::WrongProgram)
        );
        assert_eq!(
            validate_burned_mint(raw(&BOB, &d[..len]), &MINT, &HOOK),
            Err(SeatNftError::WrongMint)
        );
    }

    #[test]
    fn execute_abi_is_exact_and_decoder_does_not_authorize_an_amount() {
        let mut ix = [0; 17];
        ix[..8].copy_from_slice(&EXECUTE_DISCRIMINATOR);
        ix[8..16].copy_from_slice(&1u64.to_le_bytes());
        assert_eq!(parse_execute_amount(&ix[..16]), Ok(1));
        for len in 0..16 {
            assert_eq!(
                parse_execute_amount(&ix[..len]),
                Err(SeatNftError::InvalidInstruction)
            );
        }
        assert_eq!(
            parse_execute_amount(&ix),
            Err(SeatNftError::InvalidInstruction)
        );
        ix[0] ^= 1;
        assert_eq!(
            parse_execute_amount(&ix[..16]),
            Err(SeatNftError::InvalidInstruction)
        );
        ix[0] ^= 1;
        ix[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(parse_execute_amount(&ix[..16]), Ok(u64::MAX));
        // verify_transfer_hook separately rejects every amount except one.
        assert_eq!(MINT_PROFILE_LEN, mint().len());
        assert_eq!(TOKEN_PROFILE_LEN, token(&ALICE, 1, false).len());
    }

    #[test]
    fn token_extensions_cannot_be_omitted_substituted_or_duplicated() {
        let m = mint();
        let t = token(&ALICE, 1, false);
        let mut other = t;
        other[170] = 11; // CPI guard must not be silently treated as a hook flag.
        assert_eq!(
            proof(&m, &other, &ALICE),
            Err(SeatNftError::UnsupportedExtension)
        );
        other[170..].fill(0);
        assert_eq!(
            proof(&m, &other, &ALICE),
            Err(SeatNftError::MissingExtension)
        );
        let mut duplicate = [0; 180];
        duplicate[..175].copy_from_slice(&t);
        duplicate[175..].copy_from_slice(&[15, 0, 1, 0, 0]);
        assert_eq!(
            proof(&m, &duplicate, &ALICE),
            Err(SeatNftError::DuplicateExtension)
        );
        other = t;
        other[168] = 1; // ImmutableOwner is zero-sized.
        assert_eq!(
            proof(&m, &other, &ALICE),
            Err(SeatNftError::InvalidExtension)
        );
    }

    #[test]
    fn current_owner_follows_the_nft_and_old_proofs_are_not_accepted() {
        let m = mint();
        let before = token(&ALICE, 1, false);
        assert_eq!(proof(&m, &before, &ALICE).unwrap().owner(), &ALICE);
        assert_eq!(proof(&m, &before, &ALICE).unwrap().mint(), &MINT);
        let empty = token(&ALICE, 0, false);
        assert_eq!(proof(&m, &empty, &ALICE), Err(SeatNftError::InvalidAmount));
        let after = token(&BOB, 1, false);
        assert_eq!(proof(&m, &after, &ALICE), Err(SeatNftError::NotOwner));
        assert_eq!(proof(&m, &after, &BOB).unwrap().owner(), &BOB);
        assert_eq!(
            verify_owner(
                raw(&MINT, &m),
                raw(&SOURCE, &before),
                &MINT,
                &HOOK,
                &ALICE,
                false
            ),
            Err(SeatNftError::MissingSignature)
        );
    }

    #[test]
    fn delegate_is_never_the_governance_owner() {
        let m = mint();
        let mut t = token(&ALICE, 1, false);
        t[72] = 1;
        t[76..108].copy_from_slice(&BOB);
        t[121..129].copy_from_slice(&1u64.to_le_bytes());
        assert!(proof(&m, &t, &ALICE).is_ok());
        assert_eq!(proof(&m, &t, &BOB), Err(SeatNftError::NotOwner));
        t[72] = 2;
        assert_eq!(proof(&m, &t, &ALICE), Err(SeatNftError::InvalidOption));
        t[72] = 0;
        assert_eq!(
            proof(&m, &t, &ALICE),
            Err(SeatNftError::UnsupportedAuthority)
        );
    }

    #[test]
    fn runtime_owners_and_canonical_mint_are_mandatory() {
        let m = mint();
        let t = token(&ALICE, 1, false);
        let wrong_program = RawAccount {
            program_owner: &BOB,
            ..raw(&SOURCE, &t)
        };
        assert_eq!(
            verify_owner(raw(&MINT, &m), wrong_program, &MINT, &HOOK, &ALICE, true),
            Err(SeatNftError::WrongProgram)
        );
        let wrong_mint_owner = RawAccount {
            program_owner: &BOB,
            ..raw(&MINT, &m)
        };
        assert_eq!(
            verify_owner(
                wrong_mint_owner,
                raw(&SOURCE, &t),
                &MINT,
                &HOOK,
                &ALICE,
                true
            ),
            Err(SeatNftError::WrongProgram)
        );
        assert_eq!(
            verify_owner(raw(&BOB, &m), raw(&SOURCE, &t), &MINT, &HOOK, &ALICE, true),
            Err(SeatNftError::WrongMint)
        );
        let mut other = t;
        other[..32].copy_from_slice(&BOB);
        assert_eq!(proof(&m, &other, &ALICE), Err(SeatNftError::WrongMint));
    }

    #[test]
    fn only_irreversible_single_supply_initialized_mint_is_accepted() {
        let t = token(&ALICE, 1, false);
        for (offset, value, error) in [
            (0, 1, SeatNftError::MutableAuthority),
            (0, 2, SeatNftError::InvalidOption),
            (46, 1, SeatNftError::MutableAuthority),
            (45, 0, SeatNftError::NotInitialized),
            (45, 2, SeatNftError::NotInitialized),
            (44, 1, SeatNftError::InvalidDecimals),
            (36, 0, SeatNftError::InvalidSupply),
            (36, 2, SeatNftError::InvalidSupply),
            (82, 1, SeatNftError::InvalidPadding),
            (165, 2, SeatNftError::InvalidAccountType),
            (170, 1, SeatNftError::MutableAuthority),
            (202, 1, SeatNftError::WrongHook),
        ] {
            let mut m = mint();
            m[offset] = value;
            assert_eq!(proof(&m, &t, &ALICE), Err(error), "offset {offset}");
        }
        let mut revoked = mint();
        revoked[4..36].fill(91);
        revoked[50..82].fill(92);
        assert!(
            proof(&revoked, &t, &ALICE).is_ok(),
            "None may retain a revoked authority's bytes"
        );
    }

    #[test]
    fn token_state_amount_and_unsupported_authorities_fail_closed() {
        let m = mint();
        for (offset, value, error) in [
            (108, 0, SeatNftError::NotInitialized),
            (108, 2, SeatNftError::Frozen),
            (108, 3, SeatNftError::NotInitialized),
            (109, 1, SeatNftError::UnsupportedAuthority),
            (129, 1, SeatNftError::UnsupportedAuthority),
            (129, 2, SeatNftError::InvalidOption),
            (64, 2, SeatNftError::InvalidAmount),
            (165, 1, SeatNftError::InvalidAccountType),
            (174, 1, SeatNftError::UnexpectedTransfer),
            (174, 2, SeatNftError::InvalidExtension),
        ] {
            let mut t = token(&ALICE, 1, false);
            t[offset] = value;
            assert_eq!(proof(&m, &t, &ALICE), Err(error), "offset {offset}");
        }
    }

    #[test]
    fn every_truncated_prefix_and_multisig_length_is_rejected_without_panicking() {
        let m = mint();
        let t = token(&ALICE, 1, false);
        for len in 0..m.len() {
            assert!(proof(&m[..len], &t, &ALICE).is_err(), "mint prefix {len}");
        }
        for len in 0..t.len() {
            assert!(proof(&m, &t[..len], &ALICE).is_err(), "token prefix {len}");
        }
        let mut multisig = [0; 355];
        multisig[..m.len()].copy_from_slice(&m);
        assert_eq!(
            proof(&multisig, &t, &ALICE),
            Err(SeatNftError::InvalidLength)
        );
    }

    #[test]
    fn extension_ids_sizes_and_duplicate_records_are_checked() {
        let m = mint();
        let t = token(&ALICE, 1, false);
        for id in [3u8, 12, 17, 255] {
            let mut other = m;
            other[166] = id;
            assert_eq!(
                proof(&other, &t, &ALICE),
                Err(SeatNftError::UnsupportedExtension)
            );
        }
        let mut short = m;
        short[168] = 63;
        assert_eq!(
            proof(&short, &t, &ALICE),
            Err(SeatNftError::InvalidExtension)
        );
        short[168] = 65;
        assert_eq!(proof(&short, &t, &ALICE), Err(SeatNftError::InvalidLength));
        let mut duplicate = [0; 302];
        duplicate[..234].copy_from_slice(&m);
        duplicate[234..].copy_from_slice(&m[166..]);
        assert_eq!(
            proof(&duplicate, &t, &ALICE),
            Err(SeatNftError::DuplicateExtension)
        );
        let mut duplicate_token = [0; 179];
        duplicate_token[..175].copy_from_slice(&t);
        duplicate_token[175..].copy_from_slice(&[7, 0, 0, 0]);
        assert_eq!(
            proof(&m, &duplicate_token, &ALICE),
            Err(SeatNftError::DuplicateExtension)
        );
        let mut missing = t;
        missing[166..].fill(0);
        assert_eq!(
            proof(&m, &missing, &ALICE),
            Err(SeatNftError::MissingExtension)
        );
    }

    #[test]
    fn only_all_zero_padding_is_accepted_and_tlv_order_is_not_assumed() {
        let m = mint();
        let t = token(&ALICE, 1, false);
        let mut padded = [0; 178];
        padded[..175].copy_from_slice(&t);
        assert!(proof(&m, &padded, &ALICE).is_ok());
        padded[177] = 1;
        assert_eq!(
            proof(&m, &padded, &ALICE),
            Err(SeatNftError::InvalidPadding)
        );
        let mut reordered = t;
        reordered[166..].copy_from_slice(&[15, 0, 1, 0, 0, 7, 0, 0, 0]);
        assert!(proof(&m, &reordered, &ALICE).is_ok());
        let mut hidden = [0; 179];
        hidden[..166].copy_from_slice(&t[..166]);
        hidden[170..].copy_from_slice(&t[166..]);
        assert_eq!(
            proof(&m, &hidden, &ALICE),
            Err(SeatNftError::InvalidPadding)
        );
    }

    #[test]
    fn hook_requires_both_real_transfer_flags_and_post_transfer_balances() {
        let m = mint();
        let mut source = token(&ALICE, 0, true);
        let mut destination = token(&BOB, 1, true);
        let check = |s: &[u8], d: &[u8], amount| {
            verify_transfer_hook(
                raw(&MINT, &m),
                raw(&SOURCE, s),
                raw(&DEST, d),
                &MINT,
                &HOOK,
                amount,
            )
        };
        let owners = check(&source, &destination, 1).unwrap();
        assert_eq!(owners.from(), &ALICE);
        assert_eq!(owners.to(), &BOB);
        source[174] = 0;
        assert_eq!(
            check(&source, &destination, 1),
            Err(SeatNftError::NotTransferring)
        );
        source[174] = 1;
        destination[174] = 0;
        assert_eq!(
            check(&source, &destination, 1),
            Err(SeatNftError::NotTransferring)
        );
        destination[174] = 1;
        for amount in [0, 2, u64::MAX] {
            assert_eq!(
                check(&source, &destination, amount),
                Err(SeatNftError::InvalidAmount)
            );
        }
        source[64] = 1;
        assert_eq!(
            check(&source, &destination, 1),
            Err(SeatNftError::InvalidAmount)
        );
        source[64] = 0;
        destination[64] = 0;
        assert_eq!(
            check(&source, &destination, 1),
            Err(SeatNftError::InvalidAmount)
        );
    }

    #[test]
    fn hook_requires_immutable_owner_on_both_sides_and_distinct_accounts() {
        let m = mint();
        let source = token(&ALICE, 0, true);
        let destination = token(&BOB, 1, true);
        assert_eq!(
            verify_transfer_hook(
                raw(&MINT, &m),
                raw(&SOURCE, &source),
                raw(&SOURCE, &destination),
                &MINT,
                &HOOK,
                1
            ),
            Err(SeatNftError::SameAccount)
        );
        let mut missing = [0; 171];
        missing[..166].copy_from_slice(&source[..166]);
        missing[166..].copy_from_slice(&[15, 0, 1, 0, 1]);
        assert_eq!(
            verify_transfer_hook(
                raw(&MINT, &m),
                raw(&SOURCE, &missing),
                raw(&DEST, &destination),
                &MINT,
                &HOOK,
                1
            ),
            Err(SeatNftError::MissingExtension)
        );
        missing[..166].copy_from_slice(&destination[..166]);
        assert_eq!(
            verify_transfer_hook(
                raw(&MINT, &m),
                raw(&SOURCE, &source),
                raw(&DEST, &missing),
                &MINT,
                &HOOK,
                1
            ),
            Err(SeatNftError::MissingExtension)
        );
    }

    #[test]
    fn transfer_policy_preserves_frozen_live_seat_and_cooldown_boundaries() {
        assert_eq!(
            check_transfer_policy(true, 1, 10, 5, 15),
            Err(SeatNftError::Frozen)
        );
        for seats in [0, 2, u64::MAX] {
            assert_eq!(
                check_transfer_policy(false, seats, 10, 5, 15),
                Err(SeatNftError::NotLiveSeat)
            );
        }
        assert_eq!(
            check_transfer_policy(false, 1, 10, 5, 14),
            Err(SeatNftError::SeatsResting)
        );
        assert_eq!(check_transfer_policy(false, 1, 10, 5, 15), Ok(()));
        assert_eq!(check_transfer_policy(false, 1, 10, 5, 16), Ok(()));
        assert_eq!(
            check_transfer_policy(false, 1, u64::MAX, 1, u64::MAX),
            Err(SeatNftError::MathOverflow)
        );
    }
}
