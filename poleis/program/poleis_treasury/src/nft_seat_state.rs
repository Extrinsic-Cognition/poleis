//! Account definitions for the Harmony NFT sale. These are not entrypoints.
//! No legacy seat, auction, admission or ratification rule is changed here.
//! A seat's identity is a PDA derived from its immutable mint, never a holder
//! row index or a cached owner. The token's current owner proves control.

use crate::{
    error::PoleisError,
    mint_pricing::{quote_mint, quote_mint_price_lamports, MintPricingError, MintQuote},
    state::LAYOUT_VERSION,
    util::{expect_pda, Data},
};
use pinocchio::{error::ProgramError, AccountView};

pub const SEAT_SEED: &[u8] = b"nft-seat";
pub const MINT_SEED: &[u8] = b"seat-mint";
pub const CURVE_SEED: &[u8] = b"mint-curve";
pub const DISC_NFT_SEAT: u8 = 23;
pub const DISC_MINT_CURVE: u8 = 24;

fn price_error(error: MintPricingError) -> ProgramError {
    match error {
        MintPricingError::InvalidAmount => PoleisError::InvalidAmount.into(),
        MintPricingError::MathOverflow => PoleisError::MathOverflow.into(),
        MintPricingError::PriceChanged => PoleisError::PriceChanged.into(),
    }
}

fn header(bytes: &[u8], discriminator: u8, size: usize) -> Result<(), ProgramError> {
    if bytes.len() != size || bytes[0] != discriminator {
        return Err(PoleisError::InvalidAccount.into());
    }
    if bytes[1] != LAYOUT_VERSION {
        return Err(PoleisError::LayoutVersion.into());
    }
    Ok(())
}

/// Immutable issuance receipt. No owner is cached: resale must use the live
/// Token-2022 owner, and a retired registry seat grants no membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NftSeat {
    pub bump: u8,
    pub unit: u64,
    pub mint: [u8; 32],
    /// Issuer-authenticated completion identity; no private traces or rooms.
    pub completion: [u8; 32],
    /// Successful-mint count immediately BEFORE this issuance.
    pub serial: u64,
    pub paid_lamports: u64,
    pub first_buyer: [u8; 32],
}

impl NftSeat {
    pub const SIZE: usize = 123;

    pub fn decode(bytes: &[u8]) -> Result<Self, ProgramError> {
        header(bytes, DISC_NFT_SEAT, Self::SIZE)?;
        let d = Data(bytes);
        let value = Self {
            bump: bytes[2],
            unit: d.u64(3)?,
            mint: d.h32(11)?,
            completion: d.h32(43)?,
            serial: d.u64(75)?,
            paid_lamports: d.u64(83)?,
            first_buyer: d.h32(91)?,
        };
        if value.mint == [0; 32]
            || value.completion == [0; 32]
            || value.first_buyer == [0; 32]
            || value.paid_lamports == 0
        {
            return Err(PoleisError::InvalidAccount.into());
        }
        Ok(value)
    }

    pub fn read(account: &AccountView) -> Result<Self, ProgramError> {
        if !account.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        let value = Self::decode(&account.try_borrow()?)?;
        expect_pda(account, &[SEAT_SEED, &value.mint], value.bump)?;
        Ok(value)
    }

    /// Encode only; creation/authentication belongs to the future atomic mint
    /// instruction. This function does not grant permission to issue an NFT.
    pub fn encode(&self) -> Result<[u8; Self::SIZE], ProgramError> {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = DISC_NFT_SEAT;
        bytes[1] = LAYOUT_VERSION;
        bytes[2] = self.bump;
        bytes[3..11].copy_from_slice(&self.unit.to_le_bytes());
        bytes[11..43].copy_from_slice(&self.mint);
        bytes[43..75].copy_from_slice(&self.completion);
        bytes[75..83].copy_from_slice(&self.serial.to_le_bytes());
        bytes[83..91].copy_from_slice(&self.paid_lamports.to_le_bytes());
        bytes[91..123].copy_from_slice(&self.first_buyer);
        Self::decode(&bytes)?;
        Ok(bytes)
    }
}

/// Public sale terms plus the successful issuance count. Opening this account
/// requires constitutional authorization; no initializer is exposed yet.
/// A positive increment is immutable within a sale. The execution instruction
/// must charge the exact quote, mint once, seat once and advance once atomically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MintCurve {
    pub bump: u8,
    pub unit: u64,
    pub start_lamports: u64,
    pub step_lamports: u64,
    pub minted_count: u64,
    pub capacity: u64,
    pub opens_slot: u64,
    pub closes_slot: u64,
    pub completion_issuer: [u8; 32],
    /// Immutable raw genesis hash for the deployment's network domain. The
    /// opening authority must verify it against the intended public RPC.
    pub genesis_hash: [u8; 32],
}

impl MintCurve {
    pub const SIZE: usize = 123;

    fn validate(&self) -> Result<(), ProgramError> {
        if self.capacity == 0
            || self.capacity > crate::MAX_SEATS_PER_SALE
            || self.minted_count > self.capacity
            || self.completion_issuer == [0; 32]
            || self.genesis_hash == [0; 32]
        {
            return Err(PoleisError::InvalidAccount.into());
        }
        if self.closes_slot <= self.opens_slot
            || self.closes_slot - self.opens_slot > crate::MAX_WINDOW_SLOTS
        {
            return Err(PoleisError::InvalidWindow.into());
        }
        // Every advertised seat must have a representable price, even before
        // the sale starts. Sold out is valid state and never quotes another.
        quote_mint_price_lamports(self.start_lamports, self.step_lamports, self.capacity - 1)
            .map_err(price_error)?;
        Ok(())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProgramError> {
        header(bytes, DISC_MINT_CURVE, Self::SIZE)?;
        let d = Data(bytes);
        let value = Self {
            bump: bytes[2],
            unit: d.u64(3)?,
            start_lamports: d.u64(11)?,
            step_lamports: d.u64(19)?,
            minted_count: d.u64(27)?,
            capacity: d.u64(35)?,
            opens_slot: d.u64(43)?,
            closes_slot: d.u64(51)?,
            completion_issuer: d.h32(59)?,
            genesis_hash: d.h32(91)?,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn read(account: &AccountView) -> Result<Self, ProgramError> {
        if !account.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        let value = Self::decode(&account.try_borrow()?)?;
        expect_pda(
            account,
            &[CURVE_SEED, &value.unit.to_le_bytes()],
            value.bump,
        )?;
        Ok(value)
    }

    pub fn encode(&self) -> Result<[u8; Self::SIZE], ProgramError> {
        self.validate()?;
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = DISC_MINT_CURVE;
        bytes[1] = LAYOUT_VERSION;
        bytes[2] = self.bump;
        for (offset, value) in [
            (3, self.unit),
            (11, self.start_lamports),
            (19, self.step_lamports),
            (27, self.minted_count),
            (35, self.capacity),
            (43, self.opens_slot),
            (51, self.closes_slot),
        ] {
            bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        }
        bytes[59..91].copy_from_slice(&self.completion_issuer);
        bytes[91..123].copy_from_slice(&self.genesis_hash);
        Ok(bytes)
    }

    /// No mutation: a rejected/stale quote cannot consume a seat or change a
    /// price. The successful instruction stores count-after only after all
    /// authorization, payment and issuance operations have succeeded.
    pub fn quote(&self, slot: u64, max_price_lamports: u64) -> Result<MintQuote, ProgramError> {
        self.validate()?;
        if slot < self.opens_slot {
            return Err(PoleisError::NotYet.into());
        }
        if slot >= self.closes_slot {
            return Err(PoleisError::CommitWindowClosed.into());
        }
        if self.minted_count >= self.capacity {
            return Err(PoleisError::NoSpots.into());
        }
        quote_mint(
            self.start_lamports,
            self.step_lamports,
            self.minted_count,
            max_price_lamports,
        )
        .map_err(price_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn curve() -> MintCurve {
        MintCurve {
            bump: 254,
            unit: 0,
            start_lamports: 7,
            step_lamports: 3,
            minted_count: 0,
            capacity: 2,
            opens_slot: 100,
            closes_slot: 200,
            completion_issuer: [5; 32],
            genesis_hash: [6; 32],
        }
    }
    #[test]
    fn bounds_and_stale_approvals_do_not_consume_seats() {
        let c = curve();
        assert!(c.quote(99, 7).is_err());
        assert!(c.quote(100, 6).is_err());
        assert_eq!(c.quote(100, 7).unwrap().price_lamports, 7);
        assert_eq!(c.quote(199, 7).unwrap().minted_count_after, 1);
        assert!(c.quote(200, 7).is_err());
        assert_eq!(c.minted_count, 0);
        let next = MintCurve {
            minted_count: 1,
            ..c
        };
        assert!(next.quote(100, 7).is_err());
        assert_eq!(next.quote(100, 10).unwrap().price_lamports, 10);
        assert!(MintCurve {
            minted_count: 2,
            ..c
        }
        .quote(100, u64::MAX)
        .is_err());
    }
    #[test]
    fn invalid_capacity_issuer_or_unpriceable_supply_is_refused() {
        for c in [
            MintCurve {
                capacity: 0,
                ..curve()
            },
            MintCurve {
                capacity: 151,
                ..curve()
            },
            MintCurve {
                minted_count: 3,
                ..curve()
            },
            MintCurve {
                completion_issuer: [0; 32],
                ..curve()
            },
            MintCurve {
                genesis_hash: [0; 32],
                ..curve()
            },
            MintCurve {
                start_lamports: u64::MAX,
                ..curve()
            },
            MintCurve {
                step_lamports: 0,
                ..curve()
            },
            MintCurve {
                closes_slot: 100,
                ..curve()
            },
        ] {
            assert!(c.encode().is_err());
        }
    }
    #[test]
    fn decoding_refuses_truncation_wrong_kind_version_and_trailing_data() {
        let bytes = curve().encode().unwrap();
        for n in 0..bytes.len() {
            assert!(MintCurve::decode(&bytes[..n]).is_err());
        }
        let mut bad = bytes;
        bad[0] = DISC_NFT_SEAT;
        assert!(MintCurve::decode(&bad).is_err());
        bad = bytes;
        bad[1] = LAYOUT_VERSION + 1;
        assert!(MintCurve::decode(&bad).is_err());
        let mut extra = [0; MintCurve::SIZE + 1];
        extra[..MintCurve::SIZE].copy_from_slice(&bytes);
        assert!(MintCurve::decode(&extra).is_err());
        assert_eq!(MintCurve::decode(&bytes).unwrap(), curve());
        assert_eq!(MintCurve::SIZE, 123);
        assert_eq!(&bytes[91..123], &[6; 32]);
        let mut wrong_chain = bytes;
        wrong_chain[91..123].fill(0);
        assert!(MintCurve::decode(&wrong_chain).is_err());
        assert!(
            MintCurve::decode(&bytes[..91]).is_err(),
            "old curve layout has no network binding"
        );
    }
    #[test]
    fn receipt_has_immutable_identity_and_no_cached_resale_owner() {
        let receipt = NftSeat {
            bump: 253,
            unit: 0,
            mint: [2; 32],
            completion: [3; 32],
            serial: 0,
            paid_lamports: 7,
            first_buyer: [4; 32],
        };
        let bytes = receipt.encode().unwrap();
        assert_eq!(&bytes[0..3], &[23, 8, 253]);
        assert_eq!(&bytes[83..91], &[7, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(NftSeat::decode(&bytes).unwrap(), receipt);
        assert!(NftSeat {
            mint: [0; 32],
            ..receipt
        }
        .encode()
        .is_err());
        assert!(NftSeat {
            paid_lamports: 0,
            ..receipt
        }
        .encode()
        .is_err());
        for n in 0..bytes.len() {
            assert!(NftSeat::decode(&bytes[..n]).is_err());
        }
    }
}
