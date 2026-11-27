//! Canonical issuer-permit bytes and binding checks; no allocation or signing.
//!
//! This format is NOT evidence that NestOS was installed or that Harmony's
//! Lyssajudron was completed. A configured issuer must independently establish
//! its eligibility policy before signing. Local appearance fields are not an
//! issuer attestation. Parsing a permit, or matching precompile bytes, does not
//! verify a signature. Only the runtime-verified Ed25519 instruction does that.
//!
//! V1 message: DOMAIN (including its final NUL), program_id[32],
//! chain_genesis_hash[32], unit:u64LE, curve[32], buyer[32],
//! completion_digest[32], metadata_digest[32], max_price_lamports:u64LE,
//! expiry_slot:u64LE. Hashes are raw 32-byte values, not text/base58/hex strings.
//! No optional fields, padding, or trailing bytes. Root unit zero is valid.
//!
//! Future account-layer adapter must: authenticate the curve/config and issuer;
//! take buyer/signature status from runtime accounts, time from Clock, and the
//! chain domain from trusted deployment config (NOT the instruction's claim);
//! compare the metadata digest with the metadata actually issued; verify the
//! genuine Instructions sysvar and immediately preceding Ed25519 instruction;
//! reject CPI use or independently bind the current top-level instruction to
//! this mint entrypoint. The precompile cannot be invoked via CPI.
//!
//! It must also enforce both the permit and buyer price ceilings and atomically
//! consume one receipt per (program, chain, unit, curve, buyer, completion digest)
//! with payment and mint. Never deduplicate by signature or whole permit bytes:
//! changing expiry, price ceiling or metadata must not reuse a completion.
//!
//! Ed25519 wire source (checked 2026-10-13):
//! https://solana.com/docs/core/programs/precompiles
//! https://docs.rs/solana-ed25519-program/3.0.0/src/solana_ed25519_program/lib.rs.html
//! We accept only the standard single-signature, self-contained SDK layout;
//! batch signatures, cross-instruction references and alternate layouts refuse.

pub type Key = [u8; 32];
pub const DOMAIN: &[u8; 30] = b"poleis:harmony:mint-permit:v1\0";
pub const PROGRAM_OFFSET: usize = DOMAIN.len();
pub const CHAIN_OFFSET: usize = PROGRAM_OFFSET + 32;
pub const UNIT_OFFSET: usize = CHAIN_OFFSET + 32;
pub const CURVE_OFFSET: usize = UNIT_OFFSET + 8;
pub const BUYER_OFFSET: usize = CURVE_OFFSET + 32;
pub const COMPLETION_OFFSET: usize = BUYER_OFFSET + 32;
pub const METADATA_OFFSET: usize = COMPLETION_OFFSET + 32;
pub const MAX_PRICE_OFFSET: usize = METADATA_OFFSET + 32;
pub const EXPIRY_OFFSET: usize = MAX_PRICE_OFFSET + 8;
pub const PERMIT_LEN: usize = EXPIRY_OFFSET + 8;
pub const ED25519_DATA_LEN: usize = 112 + PERMIT_LEN;

/// Ed25519SigVerify111111111111111111111111111.
pub const ED25519_PROGRAM: Key = [
    0x03, 0x7d, 0x46, 0xd6, 0x7c, 0x93, 0xfb, 0xbe, 0x12, 0xf9, 0x42, 0x8f, 0x83, 0x8d, 0x40, 0xff,
    0x05, 0x70, 0x74, 0x49, 0x27, 0xf4, 0x8a, 0x64, 0xfc, 0xca, 0x70, 0x44, 0x80, 0x00, 0x00, 0x00,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermitError {
    InvalidLength,
    WrongDomain,
    ZeroField,
    WrongProgram,
    WrongChain,
    WrongUnit,
    WrongCurve,
    WrongBuyer,
    WrongMetadata,
    MissingBuyerSignature,
    Expired,
    PriceExceeded,
    WrongPrecompile,
    InvalidPrecompileLayout,
    WrongIssuer,
    WrongMessage,
    ZeroSignature,
}

/// Format fields only; constructing or parsing this is not an authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PermitFields {
    pub program_id: Key,
    pub chain_genesis_hash: Key,
    pub unit: u64,
    pub curve: Key,
    pub buyer: Key,
    pub completion_digest: Key,
    pub metadata_digest: Key,
    pub max_price_lamports: u64,
    pub expiry_slot: u64,
}

/// Expected values from authenticated runtime/configuration and actual metadata.
/// Passing the permit's own claimed values here would defeat the binding checks.
pub struct PermitContext {
    pub program_id: Key,
    pub chain_genesis_hash: Key,
    pub unit: u64,
    pub curve: Key,
    pub buyer: Key,
    pub buyer_is_signer: bool,
    pub metadata_digest: Key,
    pub current_slot: u64,
}

fn nonzero_fields(fields: &PermitFields) -> Result<(), PermitError> {
    for key in [
        &fields.program_id,
        &fields.chain_genesis_hash,
        &fields.curve,
        &fields.buyer,
        &fields.completion_digest,
        &fields.metadata_digest,
    ] {
        if key == &[0; 32] {
            return Err(PermitError::ZeroField);
        }
    }
    if fields.max_price_lamports == 0 || fields.expiry_slot == 0 {
        return Err(PermitError::ZeroField);
    }
    Ok(())
}

/// Exact message to be signed by the configured issuer; this function signs
/// nothing and does not select an issuer or sale terms.
pub fn encode(fields: &PermitFields) -> Result<[u8; PERMIT_LEN], PermitError> {
    nonzero_fields(fields)?;
    let mut data = [0; PERMIT_LEN];
    data[..DOMAIN.len()].copy_from_slice(DOMAIN);
    for (offset, key) in [
        (PROGRAM_OFFSET, &fields.program_id),
        (CHAIN_OFFSET, &fields.chain_genesis_hash),
        (CURVE_OFFSET, &fields.curve),
        (BUYER_OFFSET, &fields.buyer),
        (COMPLETION_OFFSET, &fields.completion_digest),
        (METADATA_OFFSET, &fields.metadata_digest),
    ] {
        data[offset..offset + 32].copy_from_slice(key);
    }
    for (offset, number) in [
        (UNIT_OFFSET, fields.unit),
        (MAX_PRICE_OFFSET, fields.max_price_lamports),
        (EXPIRY_OFFSET, fields.expiry_slot),
    ] {
        data[offset..offset + 8].copy_from_slice(&number.to_le_bytes());
    }
    Ok(data)
}

/// Validate the entire canonical byte representation. No cryptography occurs.
pub fn parse(data: &[u8]) -> Result<PermitFields, PermitError> {
    if data.len() != PERMIT_LEN {
        return Err(PermitError::InvalidLength);
    }
    if &data[..DOMAIN.len()] != DOMAIN {
        return Err(PermitError::WrongDomain);
    }
    // Fixed offsets are in bounds after the exact-length check.
    let key = |offset: usize| data[offset..offset + 32].try_into().unwrap();
    let number = |offset: usize| u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
    let fields = PermitFields {
        program_id: key(PROGRAM_OFFSET),
        chain_genesis_hash: key(CHAIN_OFFSET),
        unit: number(UNIT_OFFSET),
        curve: key(CURVE_OFFSET),
        buyer: key(BUYER_OFFSET),
        completion_digest: key(COMPLETION_OFFSET),
        metadata_digest: key(METADATA_OFFSET),
        max_price_lamports: number(MAX_PRICE_OFFSET),
        expiry_slot: number(EXPIRY_OFFSET),
    };
    nonzero_fields(&fields)?;
    Ok(fields)
}

/// Check a raw permit against the actual mint request. Expiry is inclusive:
/// current_slot == expiry_slot remains valid. This does not verify its issuer.
pub fn validate_for(data: &[u8], context: &PermitContext) -> Result<PermitFields, PermitError> {
    let fields = parse(data)?;
    if fields.program_id != context.program_id {
        return Err(PermitError::WrongProgram);
    }
    if fields.chain_genesis_hash != context.chain_genesis_hash {
        return Err(PermitError::WrongChain);
    }
    if fields.unit != context.unit {
        return Err(PermitError::WrongUnit);
    }
    if fields.curve != context.curve {
        return Err(PermitError::WrongCurve);
    }
    if fields.buyer != context.buyer {
        return Err(PermitError::WrongBuyer);
    }
    if !context.buyer_is_signer {
        return Err(PermitError::MissingBuyerSignature);
    }
    if fields.metadata_digest != context.metadata_digest {
        return Err(PermitError::WrongMetadata);
    }
    if context.current_slot > fields.expiry_slot {
        return Err(PermitError::Expired);
    }
    Ok(fields)
}

/// Price bounds only; call with the execution-time on-chain quote and the
/// buyer's separately signed ceiling. This does not charge or reserve funds.
pub fn check_price(
    fields: &PermitFields,
    buyer_max_lamports: u64,
    price_lamports: u64,
) -> Result<(), PermitError> {
    nonzero_fields(fields)?;
    if price_lamports == 0 || buyer_max_lamports == 0 {
        return Err(PermitError::ZeroField);
    }
    if price_lamports > fields.max_price_lamports || price_lamports > buyer_max_lamports {
        return Err(PermitError::PriceExceeded);
    }
    Ok(())
}

/// Match one raw Ed25519 instruction to the configured issuer and exact permit.
/// `program_id` and `data` MUST be read from the genuine Instructions sysvar's
/// immediately preceding instruction, after enforcing the adapter rules above.
/// This validates bindings only: arbitrary nonzero signature bytes can pass this
/// helper and still FAIL runtime Ed25519 verification. Never expose its success
/// as off-chain signature verification or as proof of installation/completion.
pub fn check_ed25519_instruction(
    program_id: &Key,
    data: &[u8],
    configured_issuer: &Key,
    permit: &[u8],
) -> Result<(), PermitError> {
    parse(permit)?;
    if configured_issuer == &[0; 32] {
        return Err(PermitError::ZeroField);
    }
    if program_id != &ED25519_PROGRAM {
        return Err(PermitError::WrongPrecompile);
    }
    if data.len() != ED25519_DATA_LEN {
        return Err(PermitError::InvalidLength);
    }
    // Solana SDK single-signature layout: 16-byte header, pubkey, signature,
    // message. All instruction indices must be u16::MAX (this instruction).
    let expected_header = [
        1,
        0,
        48,
        0,
        255,
        255,
        16,
        0,
        255,
        255,
        112,
        0,
        PERMIT_LEN as u8,
        (PERMIT_LEN >> 8) as u8,
        255,
        255,
    ];
    if data[..16] != expected_header {
        return Err(PermitError::InvalidPrecompileLayout);
    }
    if &data[16..48] != configured_issuer {
        return Err(PermitError::WrongIssuer);
    }
    if data[48..112].iter().all(|byte| *byte == 0) {
        return Err(PermitError::ZeroSignature);
    }
    if &data[112..] != permit {
        return Err(PermitError::WrongMessage);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields() -> PermitFields {
        PermitFields {
            program_id: [1; 32],
            chain_genesis_hash: [2; 32],
            unit: 0,
            curve: [3; 32],
            buyer: [4; 32],
            completion_digest: [5; 32],
            metadata_digest: [6; 32],
            max_price_lamports: 9_007_199_254_740_993,
            expiry_slot: 100,
        }
    }
    fn context() -> PermitContext {
        let f = fields();
        PermitContext {
            program_id: f.program_id,
            chain_genesis_hash: f.chain_genesis_hash,
            unit: f.unit,
            curve: f.curve,
            buyer: f.buyer,
            buyer_is_signer: true,
            metadata_digest: f.metadata_digest,
            current_slot: 99,
        }
    }
    // Byte-shape fixture only. [7;64] is NOT a cryptographically valid signature.
    fn precompile(permit: &[u8; PERMIT_LEN]) -> [u8; ED25519_DATA_LEN] {
        let mut data = [0; ED25519_DATA_LEN];
        data[..16].copy_from_slice(&[
            1, 0, 48, 0, 255, 255, 16, 0, 255, 255, 112, 0, 246, 0, 255, 255,
        ]);
        data[16..48].fill(8);
        data[48..112].fill(7);
        data[112..].copy_from_slice(permit);
        data
    }

    #[test]
    fn wire_format_is_fixed_domain_bound_and_exact_above_js_precision() {
        let mut f = fields();
        f.unit = 0x0807060504030201;
        let data = encode(&f).unwrap();
        assert_eq!(PERMIT_LEN, 246);
        assert_eq!(&data[..30], b"poleis:harmony:mint-permit:v1\0");
        assert_eq!(&data[30..62], &[1; 32]);
        assert_eq!(&data[62..94], &[2; 32]);
        assert_eq!(&data[94..102], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(&data[102..134], &[3; 32]);
        assert_eq!(&data[134..166], &[4; 32]);
        assert_eq!(&data[166..198], &[5; 32]);
        assert_eq!(&data[198..230], &[6; 32]);
        assert_eq!(&data[230..238], &[1, 0, 0, 0, 0, 0, 32, 0]);
        assert_eq!(&data[238..246], &[100, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(parse(&data), Ok(f));
        assert_eq!(encode(&parse(&data).unwrap()).unwrap(), data);
        assert!(
            validate_for(&encode(&fields()).unwrap(), &context()).is_ok(),
            "root unit zero is valid"
        );
    }

    #[test]
    fn all_truncations_trailing_data_and_domain_changes_refuse() {
        let data = encode(&fields()).unwrap();
        for end in 0..PERMIT_LEN {
            assert_eq!(parse(&data[..end]), Err(PermitError::InvalidLength));
        }
        let mut extra = [0; PERMIT_LEN + 1];
        extra[..PERMIT_LEN].copy_from_slice(&data);
        assert_eq!(parse(&extra), Err(PermitError::InvalidLength));
        for offset in 0..DOMAIN.len() {
            let mut changed = data;
            changed[offset] ^= 1;
            assert_eq!(parse(&changed), Err(PermitError::WrongDomain));
        }
    }

    #[test]
    fn empty_identity_digests_and_numeric_limits_refuse_on_parse_and_encode() {
        let data = encode(&fields()).unwrap();
        for offset in [
            PROGRAM_OFFSET,
            CHAIN_OFFSET,
            CURVE_OFFSET,
            BUYER_OFFSET,
            COMPLETION_OFFSET,
            METADATA_OFFSET,
        ] {
            let mut changed = data;
            changed[offset..offset + 32].fill(0);
            assert_eq!(parse(&changed), Err(PermitError::ZeroField));
        }
        for offset in [MAX_PRICE_OFFSET, EXPIRY_OFFSET] {
            let mut changed = data;
            changed[offset..offset + 8].fill(0);
            assert_eq!(parse(&changed), Err(PermitError::ZeroField));
        }
        let mut f = fields();
        f.completion_digest = [0; 32];
        assert_eq!(encode(&f), Err(PermitError::ZeroField));
        f = fields();
        f.max_price_lamports = 0;
        assert_eq!(encode(&f), Err(PermitError::ZeroField));
    }

    #[test]
    fn request_cannot_change_program_network_unit_curve_wallet_or_metadata() {
        let data = encode(&fields()).unwrap();
        let mut c = context();
        c.program_id[0] ^= 1;
        assert_eq!(validate_for(&data, &c), Err(PermitError::WrongProgram));
        c = context();
        c.chain_genesis_hash[0] ^= 1;
        assert_eq!(validate_for(&data, &c), Err(PermitError::WrongChain));
        c = context();
        c.unit = 1;
        assert_eq!(validate_for(&data, &c), Err(PermitError::WrongUnit));
        c = context();
        c.curve[0] ^= 1;
        assert_eq!(validate_for(&data, &c), Err(PermitError::WrongCurve));
        c = context();
        c.buyer[0] ^= 1;
        assert_eq!(validate_for(&data, &c), Err(PermitError::WrongBuyer));
        c = context();
        c.buyer_is_signer = false;
        assert_eq!(
            validate_for(&data, &c),
            Err(PermitError::MissingBuyerSignature)
        );
        c = context();
        c.metadata_digest[0] ^= 1;
        assert_eq!(validate_for(&data, &c), Err(PermitError::WrongMetadata));
    }

    #[test]
    fn expiry_is_inclusive_and_both_price_caps_apply_without_overflow() {
        let data = encode(&fields()).unwrap();
        let mut c = context();
        for slot in [99, 100] {
            c.current_slot = slot;
            assert!(validate_for(&data, &c).is_ok());
        }
        for slot in [101, u64::MAX] {
            c.current_slot = slot;
            assert_eq!(validate_for(&data, &c), Err(PermitError::Expired));
        }
        let f = fields();
        assert_eq!(
            check_price(&f, f.max_price_lamports, f.max_price_lamports),
            Ok(())
        );
        assert_eq!(
            check_price(&f, u64::MAX, f.max_price_lamports + 1),
            Err(PermitError::PriceExceeded)
        );
        assert_eq!(check_price(&f, 10, 11), Err(PermitError::PriceExceeded));
        assert_eq!(check_price(&f, 10, 10), Ok(()));
        assert_eq!(check_price(&f, 0, 1), Err(PermitError::ZeroField));
        assert_eq!(check_price(&f, 1, 0), Err(PermitError::ZeroField));
    }

    #[test]
    fn precompile_parser_checks_bindings_but_does_not_verify_signature() {
        let permit = encode(&fields()).unwrap();
        let mut ix = precompile(&permit);
        let check =
            |data: &[u8]| check_ed25519_instruction(&ED25519_PROGRAM, data, &[8; 32], &permit);
        assert_eq!(
            check(&ix),
            Ok(()),
            "nonzero fake signature passes only byte bindings, not cryptography"
        );
        ix[48] ^= 1;
        assert_eq!(
            check(&ix),
            Ok(()),
            "signature validation belongs to the runtime precompile"
        );
        ix[48..112].fill(0);
        assert_eq!(check(&ix), Err(PermitError::ZeroSignature));
        ix = precompile(&permit);
        ix[16] ^= 1;
        assert_eq!(check(&ix), Err(PermitError::WrongIssuer));
        assert_eq!(
            check_ed25519_instruction(&[0; 32], &ix, &[8; 32], &permit),
            Err(PermitError::WrongPrecompile)
        );
        assert_eq!(
            check_ed25519_instruction(&ED25519_PROGRAM, &ix, &[0; 32], &permit),
            Err(PermitError::ZeroField)
        );
    }

    #[test]
    fn signature_message_is_bound_to_every_permit_byte() {
        let permit = encode(&fields()).unwrap();
        let ix = precompile(&permit);
        for offset in [
            PROGRAM_OFFSET,
            CHAIN_OFFSET,
            UNIT_OFFSET,
            CURVE_OFFSET,
            BUYER_OFFSET,
            COMPLETION_OFFSET,
            METADATA_OFFSET,
            MAX_PRICE_OFFSET,
            EXPIRY_OFFSET,
        ] {
            let mut changed = permit;
            changed[offset] ^= 0x40;
            assert_eq!(
                check_ed25519_instruction(&ED25519_PROGRAM, &ix, &[8; 32], &changed),
                Err(PermitError::WrongMessage)
            );
        }
    }

    #[test]
    fn precompile_truncation_padding_batches_offsets_and_external_references_refuse() {
        let permit = encode(&fields()).unwrap();
        let ix = precompile(&permit);
        let check =
            |data: &[u8]| check_ed25519_instruction(&ED25519_PROGRAM, data, &[8; 32], &permit);
        for end in 0..ED25519_DATA_LEN {
            assert_eq!(check(&ix[..end]), Err(PermitError::InvalidLength));
        }
        let mut extra = [0; ED25519_DATA_LEN + 1];
        extra[..ED25519_DATA_LEN].copy_from_slice(&ix);
        assert_eq!(check(&extra), Err(PermitError::InvalidLength));
        // Includes count/padding, all offsets, all instruction references and size.
        for offset in 0..16 {
            let mut changed = ix;
            changed[offset] ^= 1;
            assert_eq!(
                check(&changed),
                Err(PermitError::InvalidPrecompileLayout),
                "header byte {offset}"
            );
        }
    }
}
