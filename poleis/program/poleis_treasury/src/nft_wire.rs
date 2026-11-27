//! Bounded, allocation-free Token-2022 seat-NFT instruction data.
//!
//! These builders only encode bytes. Runtime callers must authenticate all
//! accounts, derive canonical PDAs, invoke the real Token-2022 program and make
//! admission, payment, issuance and authority revocation one atomic instruction.
//! All authorities below are single authority accounts, never multisigs.
//!
//! Issuance order (all in one transaction; no intermediate mutable NFT escapes):
//! 1. Create the mint with `MINT_INITIAL_LEN` bytes, owned by Token-2022, but
//!    fund rent for `metadata_mint_len(metadata)` before metadata reallocates.
//! 2. Initialize immutable TransferHook and self-pointing MetadataPointer.
//! 3. InitializeMint2 (zero decimals, the seat PDA as mint authority, no freeze).
//! 4. Initialize metadata with that PDA as temporary update authority; then
//!    revoke metadata authority. Token-2022 resizes the mint in metadata init.
//! 5. Create the token account with `TOKEN_ACCOUNT_LEN` bytes, initialize its
//!    ImmutableOwner before InitializeAccount3, mint exactly one, then revoke
//!    mint authority. Check the resulting raw profile before seating the NFT.
//! 6. Create the hook's immutable validation PDA using `extra_account_metas`.
//!
//! Token-2022 InitializeMint2 requires the INITIAL allocation to match its
//! initialized fixed extensions. Preallocating variable metadata space breaks
//! that check; fund the final rent while allocating only the initial 302 bytes.
//! Costs: zero heap; fixed encoders <= 86 bytes; metadata <= 262 encoded bytes,
//! <= 628 final mint bytes. Work is linear only in the bounded metadata strings.
//!
//! Official wire sources checked 2026-10-13 (no new crate dependencies):
//! - https://github.com/solana-program/token-2022/tree/6a72b1658e0d61732012aae8544cebcce8774597
//!   interface/src/instruction.rs, extension/{transfer_hook,metadata_pointer}/
//!   instruction.rs; program/src/extension/token_metadata/processor.rs.
//! - https://github.com/solana-program/token-metadata/tree/9f54923e51b0c3e3d408dac70f45d36d07d2150a
//!   interface/src/{instruction,state}.rs (Borsh strings and nullable authority).
//! - https://github.com/solana-program/transfer-hook/tree/3ca0dbcbb418d06563a491f705fb115d1bd87acf
//!   interface/src/{instruction,lib}.rs (Execute discriminator and PDA seeds).
//! - https://github.com/solana-program/libraries/tree/4331de4a9bb6c7aea6e4759348534d69abd8d88f
//!   tlv-account-resolution/src/{account,state}.rs, type-length-value/src/state.rs
//!   (35-byte literal account metas, TLV u32 lengths and u32 list count).

pub use crate::seat_nft::{MAX_NAME_BYTES, MAX_SYMBOL_BYTES, MAX_URI_BYTES};

pub type Key = [u8; 32];
pub const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";
pub const MINT_INITIAL_LEN: usize = 166 + (4 + 64) * 2;
pub const TOKEN_ACCOUNT_LEN: usize = 166 + 4 + (4 + 1);
pub const EXTRA_ACCOUNT_METAS_LEN: usize = 8 + 4 + 4 + 35 * 2;
pub const MAX_METADATA_INITIALIZE_LEN: usize =
    8 + 12 + MAX_NAME_BYTES + MAX_SYMBOL_BYTES + MAX_URI_BYTES;
pub const METADATA_INITIALIZE_DISCRIMINATOR: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];
pub const METADATA_UPDATE_AUTHORITY_DISCRIMINATOR: [u8; 8] = [215, 228, 166, 228, 84, 100, 86, 123];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireError {
    ZeroAddress,
    InvalidMetadata,
    BufferTooSmall,
}

/// Application limits are shared with the strict raw-account validator.
/// Lengths are UTF-8 BYTE lengths, as required by Borsh, not character counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata<'a> {
    pub name: &'a str,
    pub symbol: &'a str,
    pub uri: &'a str,
}

impl Metadata<'_> {
    pub fn validate(&self) -> Result<(), WireError> {
        for (value, max) in [
            (self.name, MAX_NAME_BYTES),
            (self.symbol, MAX_SYMBOL_BYTES),
            (self.uri, MAX_URI_BYTES),
        ] {
            if value.is_empty() || value.len() > max {
                return Err(WireError::InvalidMetadata);
            }
        }
        Ok(())
    }

    fn strings_len(&self) -> Result<usize, WireError> {
        self.validate()?;
        // The checked per-field bounds keep this sum below 2^16.
        Ok(self.name.len() + self.symbol.len() + self.uri.len())
    }
}

fn nonzero(address: &Key) -> Result<(), WireError> {
    if address == &[0; 32] {
        Err(WireError::ZeroAddress)
    } else {
        Ok(())
    }
}

/// CPI accounts: [mint writable]. Hook authority starts revoked (32 zero bytes).
pub fn initialize_transfer_hook(hook_program: &Key) -> Result<[u8; 66], WireError> {
    nonzero(hook_program)?;
    let mut data = [0; 66];
    data[0] = 36;
    // Subinstruction 0 (Initialize), authority None at 2..34.
    data[34..66].copy_from_slice(hook_program);
    Ok(data)
}

/// CPI accounts: [mint writable]. Pointer authority is revoked immediately.
pub fn initialize_metadata_pointer(mint: &Key) -> Result<[u8; 66], WireError> {
    nonzero(mint)?;
    let mut data = [0; 66];
    data[0] = 39;
    data[34..66].copy_from_slice(mint);
    Ok(data)
}

/// CPI accounts: [mint writable]. Decimals = 0, freeze authority = None.
/// Instruction COption uses ONE tag byte, unlike the stored base-mint layout.
pub fn initialize_mint2(mint_authority: &Key) -> Result<[u8; 35], WireError> {
    nonzero(mint_authority)?;
    let mut data = [0; 35];
    data[0] = 20;
    data[2..34].copy_from_slice(mint_authority);
    Ok(data)
}

/// CPI accounts: [token account writable]. Must precede InitializeAccount3.
pub const fn initialize_immutable_owner() -> [u8; 1] {
    [22]
}

/// CPI accounts: [token account writable, mint readonly]. InitializeAccount3
/// also initializes the mint's required TransferHookAccount extension.
pub fn initialize_account3(owner: &Key) -> Result<[u8; 33], WireError> {
    nonzero(owner)?;
    let mut data = [0; 33];
    data[0] = 18;
    data[1..].copy_from_slice(owner);
    Ok(data)
}

/// CPI accounts: [mint writable, destination writable, mint authority signer].
/// MintToChecked fixes both amount = 1 and decimals = 0.
pub const fn mint_to_one_checked() -> [u8; 10] {
    [14, 1, 0, 0, 0, 0, 0, 0, 0, 0]
}

/// CPI accounts: [mint writable, current mint authority signer].
/// SetAuthority(MintTokens, None), after the single MintToChecked succeeds.
pub const fn revoke_mint_authority() -> [u8; 3] {
    [6, 0, 0]
}

/// CPI accounts: [mint writable, temporary update authority readonly,
/// same mint readonly, current mint authority signer]. Use the SAME PDA as both
/// authorities, then revoke metadata authority before leaving the instruction.
/// This is the token-metadata interface's discriminator, with NO TokenInstruction
/// tag prefix. Errors leave the output buffer untouched; only the returned
/// prefix is instruction data, never the buffer's unused tail.
pub fn initialize_metadata(metadata: Metadata<'_>, output: &mut [u8]) -> Result<usize, WireError> {
    let len = 8 + 12 + metadata.strings_len()?;
    if output.len() < len {
        return Err(WireError::BufferTooSmall);
    }
    output[..8].copy_from_slice(&METADATA_INITIALIZE_DISCRIMINATOR);
    let mut cursor = 8;
    for value in [metadata.name, metadata.symbol, metadata.uri] {
        let bytes = value.as_bytes();
        output[cursor..cursor + 4].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
        cursor += 4;
        output[cursor..cursor + bytes.len()].copy_from_slice(bytes);
        cursor += bytes.len();
    }
    Ok(len)
}

/// Final mint allocation/rent requirement: 302-byte fixed profile, 4-byte
/// Token-2022 TLV header, two pubkeys, three Borsh strings and empty vec count.
/// The TokenMetadata interface's separate 8-byte TLV discriminator is NOT
/// embedded inside Token-2022's 2-byte-type / 2-byte-length extension container.
pub fn metadata_mint_len(metadata: Metadata<'_>) -> Result<usize, WireError> {
    Ok(MINT_INITIAL_LEN + 4 + 32 + 32 + 12 + 4 + metadata.strings_len()?)
}

/// CPI accounts: [mint writable, current metadata update authority signer].
/// The nullable authority is 32 zero bytes; there is no Option tag byte.
pub const fn revoke_metadata_authority() -> [u8; 40] {
    let mut data = [0; 40];
    let mut i = 0;
    while i < 8 {
        data[i] = METADATA_UPDATE_AUTHORITY_DISCRIMINATOR[i];
        i += 1;
    }
    data
}

/// Entire data for the program-owned PDA [b"extra-account-metas", mint].
/// Contains exactly Execute's TLV entry: u32 byte length = 74, u32 count = 2,
/// then literal seat (readonly, nonsigner) and registry (writable, nonsigner).
/// Execute's complete account order is source, mint, destination, authority,
/// validation PDA, seat, registry. The Token-2022 program downgrades the first
/// four to readonly/nonsigner in the hook. The caller must authenticate both
/// canonical addresses and keep this validation account immutable.
pub fn extra_account_metas(seat: &Key, registry: &Key) -> [u8; EXTRA_ACCOUNT_METAS_LEN] {
    let mut data = [0; EXTRA_ACCOUNT_METAS_LEN];
    data[..8].copy_from_slice(&crate::seat_nft::EXECUTE_DISCRIMINATOR);
    data[8..12].copy_from_slice(&74u32.to_le_bytes());
    data[12..16].copy_from_slice(&2u32.to_le_bytes());
    // Each item: discriminator0 + literalpubkey32 + signer0 + writable0/1.
    data[17..49].copy_from_slice(seat);
    data[52..84].copy_from_slice(registry);
    data[85] = 1;
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    const AUTHORITY: Key = [3; 32];
    const MINT: Key = [5; 32];
    const REGISTRY: Key = [7; 32];

    #[test]
    fn fixed_token_instruction_vectors_match_official_pack_layout() {
        let mint = initialize_mint2(&AUTHORITY).unwrap();
        assert_eq!(&mint[..2], &[20, 0]);
        assert_eq!(&mint[2..34], &AUTHORITY);
        assert_eq!(mint[34], 0);
        let account = initialize_account3(&AUTHORITY).unwrap();
        assert_eq!(account[0], 18);
        assert_eq!(&account[1..], &AUTHORITY);
        assert_eq!(initialize_immutable_owner(), [22]);
        assert_eq!(mint_to_one_checked(), [14, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(revoke_mint_authority(), [6, 0, 0]);
    }

    #[test]
    fn extension_authorities_start_revoked_and_addresses_are_nonzero() {
        for (tag, bytes) in [
            (36, initialize_transfer_hook(&AUTHORITY).unwrap()),
            (39, initialize_metadata_pointer(&AUTHORITY).unwrap()),
        ] {
            assert_eq!(bytes[0], tag);
            assert_eq!(&bytes[1..34], &[0; 33]);
            assert_eq!(&bytes[34..66], &AUTHORITY);
        }
        assert_eq!(
            initialize_transfer_hook(&[0; 32]),
            Err(WireError::ZeroAddress)
        );
        assert_eq!(
            initialize_metadata_pointer(&[0; 32]),
            Err(WireError::ZeroAddress)
        );
        assert_eq!(initialize_mint2(&[0; 32]), Err(WireError::ZeroAddress));
        assert_eq!(initialize_account3(&[0; 32]), Err(WireError::ZeroAddress));
        let revoke = revoke_metadata_authority();
        assert_eq!(&revoke[..8], &[215, 228, 166, 228, 84, 100, 86, 123]);
        assert_eq!(&revoke[8..], &[0; 32]);
    }

    #[test]
    fn metadata_uses_utf8_byte_counts_and_exact_borsh_prefix() {
        let metadata = Metadata {
            name: "é",
            symbol: "S",
            uri: "u",
        };
        let mut bytes = [0xa5; MAX_METADATA_INITIALIZE_LEN];
        let len = initialize_metadata(metadata, &mut bytes).unwrap();
        assert_eq!(len, 24);
        assert_eq!(&bytes[..8], &[210, 225, 30, 162, 88, 184, 77, 141]);
        assert_eq!(
            &bytes[8..24],
            &[2, 0, 0, 0, 0xc3, 0xa9, 1, 0, 0, 0, b'S', 1, 0, 0, 0, b'u']
        );
        assert!(bytes[len..].iter().all(|b| *b == 0xa5));
        assert_eq!(metadata_mint_len(metadata), Ok(390));
    }

    #[test]
    fn bad_metadata_or_short_output_never_partially_overwrites() {
        let mut output = [0xa5; MAX_METADATA_INITIALIZE_LEN];
        for metadata in [
            Metadata {
                name: "",
                symbol: "S",
                uri: "u",
            },
            Metadata {
                name: "n",
                symbol: "",
                uri: "u",
            },
            Metadata {
                name: "n",
                symbol: "S",
                uri: "",
            },
            Metadata {
                name: "12345678901234567890123456789012345",
                symbol: "S",
                uri: "u",
            },
            Metadata {
                name: "n",
                symbol: "12345678901",
                uri: "u",
            },
        ] {
            assert_eq!(
                initialize_metadata(metadata, &mut output),
                Err(WireError::InvalidMetadata)
            );
            assert_eq!(metadata_mint_len(metadata), Err(WireError::InvalidMetadata));
            assert!(output.iter().all(|b| *b == 0xa5));
        }
        let metadata = Metadata {
            name: "n",
            symbol: "S",
            uri: "u",
        };
        for len in 0..23 {
            assert_eq!(
                initialize_metadata(metadata, &mut output[..len]),
                Err(WireError::BufferTooSmall)
            );
            assert!(output.iter().all(|b| *b == 0xa5));
        }
    }

    #[test]
    fn maximum_metadata_and_rent_lengths_are_bounded() {
        let name = [b'n'; MAX_NAME_BYTES];
        let symbol = [b'S'; MAX_SYMBOL_BYTES];
        let uri = [b'u'; MAX_URI_BYTES + 1];
        let metadata = Metadata {
            name: core::str::from_utf8(&name).unwrap(),
            symbol: core::str::from_utf8(&symbol).unwrap(),
            uri: core::str::from_utf8(&uri[..MAX_URI_BYTES]).unwrap(),
        };
        let mut bytes = [0; MAX_METADATA_INITIALIZE_LEN];
        assert_eq!(initialize_metadata(metadata, &mut bytes), Ok(262));
        assert_eq!(metadata_mint_len(metadata), Ok(628));
        assert_eq!(MINT_INITIAL_LEN, 302);
        assert_eq!(TOKEN_ACCOUNT_LEN, 175);
        assert_eq!(
            metadata_mint_len(Metadata {
                uri: core::str::from_utf8(&uri).unwrap(),
                ..metadata
            }),
            Err(WireError::InvalidMetadata)
        );
    }

    #[test]
    fn extra_metas_are_an_exact_static_two_account_tlv() {
        let bytes = extra_account_metas(&MINT, &REGISTRY);
        assert_eq!(bytes.len(), 86);
        assert_eq!(
            &bytes[..16],
            &[105, 37, 101, 197, 75, 251, 102, 26, 74, 0, 0, 0, 2, 0, 0, 0]
        );
        assert_eq!(bytes[16], 0);
        assert_eq!(&bytes[17..49], &MINT);
        assert_eq!(&bytes[49..52], &[0, 0, 0]);
        assert_eq!(&bytes[52..84], &REGISTRY);
        assert_eq!(&bytes[84..], &[0, 1]);
    }
}
