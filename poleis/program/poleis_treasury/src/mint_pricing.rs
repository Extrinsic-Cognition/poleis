//! Pure, allocation-free increasing mint-price arithmetic; no sale parameters
//! or transaction entrypoint are defined here. `minted_count` counts successful
//! mints BEFORE the proposed mint, so the first mint pays the starting price.
//!
//! The caller must authenticate the current count and pricing configuration,
//! enforce the buyer's maximum, and keep payment, NFT issuance and the count
//! increment atomic. A quote neither reserves a price nor records a successful
//! mint. On any failure, the caller must leave all of those effects unchanged.

/// Local errors keep this arithmetic independent of a chain runtime. A future
/// instruction can map these to the identically named `PoleisError` variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MintPricingError {
    InvalidAmount,
    MathOverflow,
    PriceChanged,
}

/// Proposed values only: write `minted_count_after` only when payment and NFT
/// issuance succeed atomically using the authoritative count quoted here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MintQuote {
    pub price_lamports: u64,
    pub minted_count_after: u64,
}

/// `start_lamports + minted_count * step_lamports`, with a positive start and
/// step. The count and price are exact `u64`s; overflow is never saturated.
pub fn quote_mint_price_lamports(
    start_lamports: u64,
    step_lamports: u64,
    minted_count: u64,
) -> Result<u64, MintPricingError> {
    if start_lamports == 0 || step_lamports == 0 {
        return Err(MintPricingError::InvalidAmount);
    }
    let increase = minted_count
        .checked_mul(step_lamports)
        .ok_or(MintPricingError::MathOverflow)?;
    start_lamports
        .checked_add(increase)
        .ok_or(MintPricingError::MathOverflow)
}

/// Quote one mint against the buyer's inclusive maximum and ensure the next
/// successful count fits. Charge the returned price, never the maximum.
/// Recompute at execution against the current count; this is not a reservation.
pub fn quote_mint(
    start_lamports: u64,
    step_lamports: u64,
    minted_count: u64,
    max_price_lamports: u64,
) -> Result<MintQuote, MintPricingError> {
    let price_lamports = quote_mint_price_lamports(start_lamports, step_lamports, minted_count)?;
    if price_lamports > max_price_lamports {
        return Err(MintPricingError::PriceChanged);
    }
    let minted_count_after = minted_count
        .checked_add(1)
        .ok_or(MintPricingError::MathOverflow)?;
    Ok(MintQuote {
        price_lamports,
        minted_count_after,
    })
}

#[cfg(test)]
mod tests {
    use super::{quote_mint, quote_mint_price_lamports as price, MintPricingError::*, MintQuote};

    // Synthetic vectors shared with tools/harmony-mint-pricing.test.mjs;
    // these are not live sale parameters.
    #[test]
    fn count_means_successful_mints_before_this_mint_and_quotes_do_not_advance_it() {
        assert_eq!(price(7, 3, 0), Ok(7));
        assert_eq!(price(7, 3, 1), Ok(10));
        assert_eq!(price(7, 3, 2), Ok(13));
        let first = quote_mint(7, 3, 0, 7).unwrap();
        assert_eq!(
            first,
            MintQuote {
                price_lamports: 7,
                minted_count_after: 1
            }
        );
        assert_eq!(quote_mint(7, 3, 0, 7), Ok(first));
        assert_eq!(
            quote_mint(7, 3, first.minted_count_after, 10),
            Ok(MintQuote {
                price_lamports: 10,
                minted_count_after: 2
            })
        );
    }

    #[test]
    fn lamports_beyond_javascript_number_precision_remain_exact() {
        assert_eq!(
            price(9_007_199_254_740_993, 2, 3),
            Ok(9_007_199_254_740_999)
        );
    }

    #[test]
    fn start_and_step_must_be_positive_but_count_can_be_zero() {
        for (start, step) in [(0, 3), (7, 0), (0, 0)] {
            assert_eq!(price(start, step, 0), Err(InvalidAmount));
            assert_eq!(quote_mint(start, step, 0, u64::MAX), Err(InvalidAmount));
        }
        assert_eq!(price(1, 1, 0), Ok(1));
    }

    #[test]
    fn buyer_maximum_is_inclusive_and_rejects_a_stale_quote_without_an_increment() {
        assert_eq!(quote_mint(7, 3, 0, 6), Err(PriceChanged));
        assert_eq!(quote_mint(7, 3, 0, 0), Err(PriceChanged));
        let accepted = MintQuote {
            price_lamports: 7,
            minted_count_after: 1,
        };
        assert_eq!(quote_mint(7, 3, 0, 7), Ok(accepted));
        assert_eq!(quote_mint(7, 3, 0, 99), Ok(accepted));
        assert_eq!(quote_mint(7, 3, 1, 7), Err(PriceChanged));
        assert_eq!(
            quote_mint(7, 3, 1, 10),
            Ok(MintQuote {
                price_lamports: 10,
                minted_count_after: 2
            })
        );
    }

    #[test]
    fn price_and_successful_count_never_wrap_at_u64_boundaries() {
        assert_eq!(price(u64::MAX, u64::MAX, 0), Ok(u64::MAX));
        assert_eq!(price(1, 1, u64::MAX - 1), Ok(u64::MAX));
        assert_eq!(price(u64::MAX, 1, 1), Err(MathOverflow));
        assert_eq!(price(1, 1, u64::MAX), Err(MathOverflow));
        assert_eq!(price(1, u64::MAX, 2), Err(MathOverflow));
        assert_eq!(
            quote_mint(1, 1, u64::MAX - 1, u64::MAX),
            Ok(MintQuote {
                price_lamports: u64::MAX,
                minted_count_after: u64::MAX
            })
        );
        assert_eq!(quote_mint(1, 1, u64::MAX, u64::MAX), Err(MathOverflow));
    }

    #[test]
    fn non_unit_step_accepts_the_last_representable_price_and_rejects_the_next() {
        assert_eq!(price(2, 3, 6_148_914_691_236_517_204), Ok(u64::MAX - 1));
        assert_eq!(price(2, 3, 6_148_914_691_236_517_205), Err(MathOverflow));
    }
}
