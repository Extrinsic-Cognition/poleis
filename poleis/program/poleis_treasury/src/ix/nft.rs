//! Token ownership is the authority for one stable registry seat. No cached
//! wallet owner or generic PDA-signing proxy exists on this path.

use crate::{
    error::PoleisError,
    nft_seat_state::{NftSeat, MINT_SEED},
    seat_nft::{self, RawAccount, SeatNftError},
    state::Registry,
    util::{expect_writable, find_and_expect_pda},
};
use pinocchio::{error::ProgramError, sysvars::{clock::Clock, Sysvar}, AccountView, ProgramResult};

pub(crate) fn token_error(e: SeatNftError) -> ProgramError {
    match e {
        SeatNftError::MissingSignature => ProgramError::MissingRequiredSignature,
        SeatNftError::NotOwner => PoleisError::Unauthorized.into(),
        SeatNftError::Frozen => PoleisError::Frozen.into(),
        SeatNftError::SeatsResting => PoleisError::SeatsResting.into(),
        SeatNftError::MathOverflow => PoleisError::MathOverflow.into(),
        SeatNftError::NotLiveSeat => PoleisError::NotHolder.into(),
        SeatNftError::InvalidInstruction => ProgramError::InvalidInstructionData,
        _ => PoleisError::InvalidAccount.into(),
    }
}

fn token_program(account: &AccountView) -> ProgramResult {
    if !account.owned_by(&crate::TOKEN_2022_PROGRAM_ID) {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

/// The immutable receipt, its canonical mint, and the one live registry row
/// must agree. An expelled or burned seat never acquires a replacement row.
pub(crate) fn live_seat(
    seat: &AccountView,
    mint: &AccountView,
    registry: &AccountView,
) -> Result<(NftSeat, usize), ProgramError> {
    let receipt = NftSeat::read(seat)?;
    if mint.address().as_array() != &receipt.mint {
        return Err(PoleisError::InvalidAccount.into());
    }
    find_and_expect_pda(mint, &[MINT_SEED, &receipt.unit.to_le_bytes(), &receipt.completion])?;
    super::read_registry(registry, receipt.unit)?;
    let index = Registry::find_holder(registry, crate::HOLDER_NFT, seat.address().as_array())?
        .ok_or(PoleisError::NotHolder)?;
    if Registry::holder(registry, index)?.seats != 1 {
        return Err(PoleisError::InvalidHolder.into());
    }
    Ok((receipt, index))
}

/// Each owner action rechecks the actual token state. All borrows are released
/// before returning, and callers must not perform another ownership-changing
/// CPI between this proof and the action it authorizes.
pub(crate) fn current_owner(
    owner: &AccountView,
    seat: &AccountView,
    mint: &AccountView,
    token: &AccountView,
    registry: &AccountView,
) -> Result<(NftSeat, usize), ProgramError> {
    let live = live_seat(seat, mint, registry)?;
    token_program(mint)?;
    token_program(token)?;
    seat_nft::verify_owner(
        RawAccount { address: mint.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &mint.try_borrow()? },
        RawAccount { address: token.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &token.try_borrow()? },
        &live.0.mint, crate::ID.as_array(), owner.address().as_array(), owner.is_signer(),
    ).map_err(token_error)?;
    Ok(live)
}

/// SPL Transfer Hook Execute ABI, called by Token-2022 after balance changes.
/// Accounts: source, mint, destination, transfer authority, extra-account-metas,
/// immutable seat receipt, writable registry. The transfer authority may be a
/// delegate; only the two token owners determine whether the seat changed hands.
pub fn transfer_hook(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [source, mint, destination, _authority, validation, seat, registry] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let amount = seat_nft::parse_execute_amount(data).map_err(token_error)?;
    let (receipt, index) = live_seat(seat, mint, registry)?;
    expect_writable(registry)?;
    if !validation.owned_by(&crate::ID) {
        return Err(PoleisError::InvalidAccount.into());
    }
    find_and_expect_pda(validation, &[b"extra-account-metas", &receipt.mint])?;
    let expected = crate::nft_wire::extra_account_metas(seat.address().as_array(), registry.address().as_array());
    if validation.try_borrow()?.as_ref() != expected.as_slice() {
        return Err(PoleisError::InvalidAccount.into());
    }
    for account in [&*mint, &*source, &*destination] { token_program(account)?; }
    let owners = seat_nft::verify_transfer_hook(
        RawAccount { address: mint.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &mint.try_borrow()? },
        RawAccount { address: source.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &source.try_borrow()? },
        RawAccount { address: destination.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &destination.try_borrow()? },
        &receipt.mint, crate::ID.as_array(), amount,
    ).map_err(token_error)?;
    let head = Registry::head(registry)?;
    let mut holder = Registry::holder(registry, index)?;
    let now = Clock::get()?.slot;
    seat_nft::check_transfer_policy(head.frozen, holder.seats, holder.last_active, head.window, now)
        .map_err(token_error)?;
    if owners.from() != owners.to() {
        // The row and its historical session/distribution bits stay put.
        holder.weight = 1;
        holder.last_active = now;
        Registry::set_holder(registry, index, &holder)?;
    }
    Ok(())
}

/// Permissionless reconciliation after Token-2022 burn. Burn is not a transfer
/// and therefore does not invoke the hook. The immutable mint must actually
/// have supply zero; the original issuance receipt remains to prevent remint.
/// Accounts [seat, mint, registry(w)]; data [].
pub fn retire_burned(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [seat, mint, registry] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    crate::util::Data(data).expect_len(0)?;
    let (receipt, index) = live_seat(seat, mint, registry)?;
    expect_writable(registry)?;
    token_program(mint)?;
    seat_nft::validate_burned_mint(
        RawAccount { address: mint.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &mint.try_borrow()? },
        &receipt.mint, crate::ID.as_array(),
    ).map_err(token_error)?;
    Registry::remove_holder(registry, index)?;
    Ok(())
}

/// A constitutional expulsion may pay compensation to the stable seat PDA.
/// Its current NFT owner can withdraw that balance above rent after the seat
/// has left the registry. The immutable receipt is never closed or reassigned.
/// Accounts [owner(s,w), seat(w), mint, token, registry]; data [].
pub fn withdraw_retired(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, seat, mint, token, registry] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    crate::util::Data(data).expect_len(0)?;
    let receipt = NftSeat::read(seat)?;
    super::read_registry(registry, receipt.unit)?;
    if mint.address().as_array() != &receipt.mint {
        return Err(PoleisError::InvalidAccount.into());
    }
    find_and_expect_pda(mint, &[MINT_SEED, &receipt.unit.to_le_bytes(), &receipt.completion])?;
    if Registry::find_holder(registry, crate::HOLDER_NFT, seat.address().as_array())?.is_some() {
        return Err(PoleisError::NotClosable.into());
    }
    expect_writable(owner)?;
    expect_writable(seat)?;
    token_program(mint)?;
    token_program(token)?;
    seat_nft::verify_owner(
        RawAccount { address: mint.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &mint.try_borrow()? },
        RawAccount { address: token.address().as_array(), program_owner: &seat_nft::TOKEN_2022_OWNER, data: &token.try_borrow()? },
        &receipt.mint, crate::ID.as_array(), owner.address().as_array(), owner.is_signer(),
    ).map_err(token_error)?;
    let floor = pinocchio::sysvars::rent::Rent::get()?.try_minimum_balance(NftSeat::SIZE)?;
    let amount = seat.lamports().checked_sub(floor).ok_or(PoleisError::TreasuryInsufficient)?;
    if amount == 0 { return Err(PoleisError::NothingToReclaim.into()); }
    crate::util::move_lamports(seat, owner, amount)
}
