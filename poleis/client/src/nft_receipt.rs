//! Pure validation of supplied mint account snapshots; no RPC or signing.
//!
//! The caller must obtain these accounts from a trusted RPC on the expected
//! genesis, at one coherent commitment/context, and authenticate the deployed
//! program separately. Matching bytes are NOT a chain proof, transaction
//! confirmation, eligibility attestation, signer authority or live membership.
//! A permanent issuance receipt survives resale, expulsion and burn. An
//! optional token account observes the CURRENT owner, never the first buyer's
//! cached/derived token account after resale.

use crate::{nft, PROGRAM_ID};
use poleis_treasury::{
    nft_seat_state::{NftSeat, SEAT_SEED},
    nft_wire::{self, Metadata},
    seat_nft::{self, RawAccount, SeatNftError},
};
use solana_pubkey::Pubkey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiptError {
    InvalidExpectation,
    WrongReceiptOwner,
    InvalidReceipt,
    WrongIdentity,
    PriceExceeded,
    MetadataMismatch,
    InvalidMint(SeatNftError),
    InvalidToken(SeatNftError),
}

/// An immutable intent prepared from trusted completion and sale data. Payment
/// can differ from an earlier quote, but must fit the buyer-approved ceiling.
pub struct ReceiptExpectation<'a> {
    pub unit: u64,
    pub completion: [u8; 32],
    pub first_buyer: Pubkey,
    pub max_price_lamports: u64,
    pub metadata: Metadata<'a>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnershipObservation {
    /// Issuance is observed, but no token account was requested/checked.
    NotObserved,
    /// One live token is held here. This is not evidence its owner signed.
    HeldBy(Pubkey),
    /// The immutable mint has supply zero. No current ownership is claimed.
    Burned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptSnapshot {
    pub receipt: NftSeat,
    pub mint: Pubkey,
    pub seat: Pubkey,
    pub ownership: OwnershipObservation,
}

/// Validate immutable issuance independently of whether the first buyer still
/// owns the NFT. Supplying a live token account additionally observes its owner.
/// For burned mints an optional surviving token account must be empty; it does
/// not identify a current owner. Membership requires a separate registry check.
pub fn validate_receipt(
    receipt_account: RawAccount<'_>,
    mint_account: RawAccount<'_>,
    token_account: Option<RawAccount<'_>>,
    expected: &ReceiptExpectation<'_>,
) -> Result<ReceiptSnapshot, ReceiptError> {
    if expected.completion == [0; 32]
        || expected.first_buyer == Pubkey::default()
        || expected.max_price_lamports == 0
        || expected.metadata.validate().is_err()
    {
        return Err(ReceiptError::InvalidExpectation);
    }
    if receipt_account.program_owner != PROGRAM_ID.as_array() {
        return Err(ReceiptError::WrongReceiptOwner);
    }
    let receipt =
        NftSeat::decode(receipt_account.data).map_err(|_| ReceiptError::InvalidReceipt)?;
    let mint = nft::mint_pda(expected.unit, &expected.completion);
    let (seat, bump) = Pubkey::find_program_address(&[SEAT_SEED, mint.as_ref()], &PROGRAM_ID);
    if receipt_account.address != seat.as_array()
        || receipt.bump != bump
        || receipt.unit != expected.unit
        || receipt.mint != mint.to_bytes()
        || receipt.completion != expected.completion
        || receipt.first_buyer != expected.first_buyer.to_bytes()
    {
        return Err(ReceiptError::WrongIdentity);
    }
    if receipt.paid_lamports > expected.max_price_lamports {
        return Err(ReceiptError::PriceExceeded);
    }
    let burned = match seat_nft::validate_mint(mint_account, mint.as_array(), PROGRAM_ID.as_array())
    {
        Ok(()) => false,
        Err(SeatNftError::InvalidSupply) => {
            seat_nft::validate_burned_mint(mint_account, mint.as_array(), PROGRAM_ID.as_array())
                .map_err(ReceiptError::InvalidMint)?;
            true
        }
        Err(error) => return Err(ReceiptError::InvalidMint(error)),
    };
    // The shared validator already authenticated all TLV framing, paired
    // metadata/pointer, UTF-8 bounds and immutable authorities. Compare only
    // the canonical metadata string bytes; do not implement a second parser.
    let mut encoded = [0; nft_wire::MAX_METADATA_INITIALIZE_LEN];
    let encoded_len = nft_wire::initialize_metadata(expected.metadata, &mut encoded)
        .map_err(|_| ReceiptError::InvalidExpectation)?;
    let mut remaining = &mint_account.data[seat_nft::TLV_OFFSET..];
    let mut matches_metadata = false;
    while !remaining.is_empty() && remaining.iter().any(|b| *b != 0) {
        let kind = u16::from_le_bytes(remaining[..2].try_into().unwrap());
        let len = u16::from_le_bytes(remaining[2..4].try_into().unwrap()) as usize;
        let value = &remaining[4..4 + len];
        if kind == seat_nft::TOKEN_METADATA {
            matches_metadata = value[64..value.len() - 4] == encoded[8..encoded_len];
        }
        remaining = &remaining[4 + len..];
    }
    if !matches_metadata {
        return Err(ReceiptError::MetadataMismatch);
    }
    let ownership = match token_account {
        None if burned => OwnershipObservation::Burned,
        None => OwnershipObservation::NotObserved,
        Some(account) => {
            let token = seat_nft::parse_token(account, mint.as_array())
                .map_err(ReceiptError::InvalidToken)?;
            if token.transferring() {
                return Err(ReceiptError::InvalidToken(SeatNftError::UnexpectedTransfer));
            }
            if token.amount() != if burned { 0 } else { 1 } {
                return Err(ReceiptError::InvalidToken(SeatNftError::InvalidAmount));
            }
            if burned {
                OwnershipObservation::Burned
            } else {
                OwnershipObservation::HeldBy(Pubkey::new_from_array(*token.owner()))
            }
        }
    };
    Ok(ReceiptSnapshot {
        receipt,
        mint,
        seat,
        ownership,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const META: Metadata<'static> = Metadata {
        name: "Test Seat",
        symbol: "TEST",
        uri: "ipfs://fixture",
    };
    const UNIT: u64 = 42;
    const COMPLETION: [u8; 32] = [9; 32];
    const BUYER: Pubkey = Pubkey::new_from_array([7; 32]);

    fn expectation() -> ReceiptExpectation<'static> {
        ReceiptExpectation {
            unit: UNIT,
            completion: COMPLETION,
            first_buyer: BUYER,
            max_price_lamports: 30,
            metadata: META,
        }
    }
    fn extension(data: &mut Vec<u8>, kind: u16, value: &[u8]) {
        data.extend_from_slice(&kind.to_le_bytes());
        data.extend_from_slice(&(value.len() as u16).to_le_bytes());
        data.extend_from_slice(value);
    }
    struct Fixture {
        mint_key: Pubkey,
        seat_key: Pubkey,
        token_key: Pubkey,
        receipt: Vec<u8>,
        mint: Vec<u8>,
        token: Vec<u8>,
        receipt_owner: [u8; 32],
        mint_owner: [u8; 32],
        token_owner: [u8; 32],
    }
    impl Fixture {
        fn new() -> Self {
            let mint_key = nft::mint_pda(UNIT, &COMPLETION);
            let (seat_key, bump) =
                Pubkey::find_program_address(&[SEAT_SEED, mint_key.as_ref()], &PROGRAM_ID);
            let receipt = NftSeat {
                bump,
                unit: UNIT,
                mint: mint_key.to_bytes(),
                completion: COMPLETION,
                serial: 7,
                paid_lamports: 23,
                first_buyer: BUYER.to_bytes(),
            }
            .encode()
            .unwrap()
            .to_vec();
            let mut mint = vec![0; seat_nft::TLV_OFFSET];
            mint[36..44].copy_from_slice(&1u64.to_le_bytes());
            mint[45] = 1;
            mint[165] = 1;
            let mut hook = vec![0; 32];
            hook.extend_from_slice(PROGRAM_ID.as_ref());
            extension(&mut mint, seat_nft::TRANSFER_HOOK, &hook);
            let mut pointer = vec![0; 32];
            pointer.extend_from_slice(mint_key.as_ref());
            extension(&mut mint, seat_nft::METADATA_POINTER, &pointer);
            let mut metadata = vec![0; 32];
            metadata.extend_from_slice(mint_key.as_ref());
            let mut encoded = [0; nft_wire::MAX_METADATA_INITIALIZE_LEN];
            let length = nft_wire::initialize_metadata(META, &mut encoded).unwrap();
            metadata.extend_from_slice(&encoded[8..length]);
            metadata.extend_from_slice(&[0; 4]);
            extension(&mut mint, seat_nft::TOKEN_METADATA, &metadata);
            let mut token = vec![0; seat_nft::TLV_OFFSET];
            token[..32].copy_from_slice(mint_key.as_ref());
            token[32..64].copy_from_slice(BUYER.as_ref());
            token[64..72].copy_from_slice(&1u64.to_le_bytes());
            token[108] = 1;
            token[165] = 2;
            extension(&mut token, seat_nft::IMMUTABLE_OWNER, &[]);
            extension(&mut token, seat_nft::TRANSFER_HOOK_ACCOUNT, &[0]);
            Self {
                mint_key,
                seat_key,
                token_key: nft::initial_token_pda(&mint_key, &BUYER),
                receipt,
                mint,
                token,
                receipt_owner: PROGRAM_ID.to_bytes(),
                mint_owner: seat_nft::TOKEN_2022_OWNER,
                token_owner: seat_nft::TOKEN_2022_OWNER,
            }
        }
        fn validate(
            &self,
            expected: &ReceiptExpectation<'_>,
            with_token: bool,
        ) -> Result<ReceiptSnapshot, ReceiptError> {
            validate_receipt(
                RawAccount {
                    address: self.seat_key.as_array(),
                    program_owner: &self.receipt_owner,
                    data: &self.receipt,
                },
                RawAccount {
                    address: self.mint_key.as_array(),
                    program_owner: &self.mint_owner,
                    data: &self.mint,
                },
                with_token.then_some(RawAccount {
                    address: self.token_key.as_array(),
                    program_owner: &self.token_owner,
                    data: &self.token,
                }),
                expected,
            )
        }
    }

    #[test]
    fn issuance_and_current_owner_are_separate_observations() {
        let mut fixture = Fixture::new();
        let original = fixture.validate(&expectation(), true).unwrap();
        assert_eq!(original.ownership, OwnershipObservation::HeldBy(BUYER));
        assert_eq!(original.receipt.paid_lamports, 23);
        assert_eq!(original.receipt.serial, 7);
        assert_eq!(
            fixture.validate(&expectation(), false).unwrap().ownership,
            OwnershipObservation::NotObserved
        );
        let new_owner = Pubkey::new_from_array([8; 32]);
        fixture.token_key = Pubkey::new_from_array([11; 32]); // resale token account need not be the initial PDA
        fixture.token[32..64].copy_from_slice(new_owner.as_ref());
        let resale = fixture.validate(&expectation(), true).unwrap();
        assert_eq!(resale.receipt.first_buyer, BUYER.to_bytes());
        assert_eq!(resale.ownership, OwnershipObservation::HeldBy(new_owner));
    }

    #[test]
    fn burned_receipt_still_proves_issuance_but_never_live_ownership() {
        let mut fixture = Fixture::new();
        fixture.mint[36..44].fill(0);
        assert_eq!(
            fixture.validate(&expectation(), false).unwrap().ownership,
            OwnershipObservation::Burned
        );
        assert_eq!(
            fixture.validate(&expectation(), true),
            Err(ReceiptError::InvalidToken(SeatNftError::InvalidAmount))
        );
        fixture.token[64..72].fill(0);
        assert_eq!(
            fixture.validate(&expectation(), true).unwrap().ownership,
            OwnershipObservation::Burned
        );
        fixture.mint[36..44].copy_from_slice(&2u64.to_le_bytes());
        assert_eq!(
            fixture.validate(&expectation(), false),
            Err(ReceiptError::InvalidMint(SeatNftError::InvalidSupply))
        );
    }

    #[test]
    fn receipt_runtime_owner_pda_bump_and_immutable_fields_must_match() {
        for variant in 0..9 {
            let mut fixture = Fixture::new();
            match variant {
                0 => fixture.receipt_owner = [1; 32],
                1 => fixture.seat_key = Pubkey::new_from_array([1; 32]),
                2 => fixture.receipt[2] ^= 1,
                3 => fixture.receipt[3] ^= 1,
                4 => fixture.receipt[11] ^= 1,
                5 => fixture.receipt[43] ^= 1,
                6 => fixture.receipt[91] ^= 1,
                7 => {
                    fixture.receipt.pop();
                }
                _ => fixture.receipt[1] ^= 1,
            }
            assert!(
                fixture.validate(&expectation(), false).is_err(),
                "variant {variant}"
            );
        }
    }

    #[test]
    fn expected_identity_and_buyer_price_ceiling_are_bound() {
        let fixture = Fixture::new();
        let mut expected = expectation();
        expected.max_price_lamports = 22;
        assert_eq!(
            fixture.validate(&expected, false),
            Err(ReceiptError::PriceExceeded)
        );
        expected.max_price_lamports = 23;
        assert!(fixture.validate(&expected, false).is_ok());
        expected.first_buyer = Pubkey::new_from_array([8; 32]);
        assert_eq!(
            fixture.validate(&expected, false),
            Err(ReceiptError::WrongIdentity)
        );
        expected = expectation();
        expected.completion[0] ^= 1;
        assert_eq!(
            fixture.validate(&expected, false),
            Err(ReceiptError::WrongIdentity)
        );
        expected = expectation();
        expected.unit += 1;
        assert_eq!(
            fixture.validate(&expected, false),
            Err(ReceiptError::WrongIdentity)
        );
        expected = expectation();
        expected.max_price_lamports = 0;
        assert_eq!(
            fixture.validate(&expected, false),
            Err(ReceiptError::InvalidExpectation)
        );
    }

    #[test]
    fn metadata_must_be_present_exact_and_immutable() {
        let fixture = Fixture::new();
        for metadata in [
            Metadata {
                name: "Other",
                ..META
            },
            Metadata {
                symbol: "ALT",
                ..META
            },
            Metadata {
                uri: "ipfs://other",
                ..META
            },
        ] {
            let expected = ReceiptExpectation {
                metadata,
                ..expectation()
            };
            assert_eq!(
                fixture.validate(&expected, false),
                Err(ReceiptError::MetadataMismatch)
            );
        }
        let mut fixture = Fixture::new();
        fixture.mint[306] = 1; // TokenMetadata update authority, after fixed302 + TLV header4
        assert_eq!(
            fixture.validate(&expectation(), false),
            Err(ReceiptError::InvalidMint(SeatNftError::MutableAuthority))
        );
        fixture = Fixture::new();
        fixture.mint.truncate(seat_nft::MINT_PROFILE_LEN);
        assert_eq!(
            fixture.validate(&expectation(), false),
            Err(ReceiptError::MetadataMismatch)
        );
    }

    #[test]
    fn raw_token_profiles_cannot_forge_current_ownership() {
        for variant in 0..7 {
            let mut fixture = Fixture::new();
            match variant {
                0 => fixture.mint_owner = [1; 32],
                1 => fixture.token_owner = [1; 32],
                2 => fixture.token[0] ^= 1,
                3 => fixture.token[64..72].fill(0),
                4 => fixture.token[108] = 2,
                5 => *fixture.token.last_mut().unwrap() = 1,
                _ => fixture.mint_key = Pubkey::new_from_array([1; 32]),
            }
            assert!(
                fixture.validate(&expectation(), true).is_err(),
                "variant {variant}"
            );
        }
    }
}
