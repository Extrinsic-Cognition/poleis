//! One immutable, increasing-price NFT genesis sale in the existing tree.
//! Payment, issuer-permit consumption, NFT creation and registry admission are
//! atomic. Local Harmony appearance fields are never eligibility evidence here.

use crate::{
    completion_permit::{self, PermitContext, PermitError, PermitFields},
    error::PoleisError,
    nft_seat_state::{MintCurve, NftSeat, CURVE_SEED, MINT_SEED, SEAT_SEED},
    nft_wire::{self, Metadata, WireError},
    seat_nft::{self, RawAccount},
    state::{Holder, Registry, Treasury},
    util::{
        create_pda, expect_signer, expect_writable, find_and_expect_pda, grow_account, receipt,
        sha256v, Data,
    },
};
use pinocchio::{
    cpi::{invoke_signed, Seed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    sysvars::{
        clock::Clock,
        instructions::{Instructions, INSTRUCTIONS_ID},
        rent::Rent,
        Sysvar,
    },
    AccountView, Address, ProgramResult,
};

pub const TOKEN_SEED: &[u8] = b"seat-token";

fn wire_error(_: WireError) -> ProgramError {
    ProgramError::InvalidInstructionData
}

fn permit_error(error: PermitError) -> ProgramError {
    match error {
        PermitError::MissingBuyerSignature => ProgramError::MissingRequiredSignature,
        PermitError::PriceExceeded => PoleisError::PriceChanged.into(),
        PermitError::WrongPrecompile
        | PermitError::InvalidPrecompileLayout
        | PermitError::WrongIssuer
        | PermitError::WrongMessage
        | PermitError::ZeroSignature => PoleisError::BadPrecompile.into(),
        _ => PoleisError::Unauthorized.into(),
    }
}

fn system_program(system: &AccountView) -> ProgramResult {
    if system.address() != &Address::default() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

fn fresh(account: &AccountView) -> ProgramResult {
    expect_writable(account)?;
    if account.data_len() != 0 || !account.owned_by(&Address::default()) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

/// Tag 46 data: charter32, plan cap8, plan period8, governance window8,
/// rung1, start8, step8, opens8, closes8, issuer32, chain genesis32,
/// founder count1, founders[count*32], name/description (legacy u8 lengths).
/// Accounts: payer(s,w), config(w), treasury(w), registry(w), curve(w), system.
pub fn open_nft_genesis(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [payer, config, treasury, registry, curve, system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if data.len() < 156 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let d = Data(data);
    let n = data[153] as usize;
    let end = 154 + n * 32;
    if n > crate::PENTAGONS || data.len() < end + 2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let (name, used) = super::council::parse_text(&data[end..], crate::MAX_NAME)?;
    let (desc, used2) = super::council::parse_text(&data[end + used..], crate::MAX_DESC)?;
    if data.len() != end + used + used2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut cfg = super::expect_solo_authority(config, payer)?;
    expect_writable(payer)?;
    expect_writable(config)?;
    system_program(system)?;
    for account in [&*treasury, &*registry, &*curve] {
        fresh(account)?;
    }
    let rung = data[56] as usize;
    if rung == 0 || rung >= crate::LADDER.len() {
        return Err(PoleisError::NotOnLadder.into());
    }
    let window = d.u64(48)?;
    if !(crate::MIN_WINDOW_SLOTS..=crate::MAX_WINDOW_SLOTS).contains(&window) {
        return Err(PoleisError::InvalidWindow.into());
    }
    let now = Clock::get()?.slot;
    let root = cfg.root;
    let root_le = root.to_le_bytes();
    let curve_seeds: &[&[u8]] = &[CURVE_SEED, &root_le];
    let terms = MintCurve {
        bump: find_and_expect_pda(curve, curve_seeds)?,
        unit: root,
        start_lamports: d.u64(57)?,
        step_lamports: d.u64(65)?,
        minted_count: 0,
        capacity: crate::ladder_hexagons(rung),
        opens_slot: d.u64(73)?,
        closes_slot: d.u64(81)?,
        completion_issuer: d.h32(89)?,
        genesis_hash: d.h32(121)?,
    };
    let encoded_terms = terms.encode()?;
    if terms.opens_slot < now {
        return Err(PoleisError::InvalidWindow.into());
    }
    let treasury_seeds: &[&[u8]] = &[b"treasury", &root_le];
    let treasury_bump = find_and_expect_pda(treasury, treasury_seeds)?;
    let registry_seeds: &[&[u8]] = &[b"registry", &root_le];
    let registry_bump = find_and_expect_pda(registry, registry_seeds)?;
    create_pda(
        payer,
        treasury,
        Treasury::SIZE,
        treasury_seeds,
        treasury_bump,
    )?;
    Treasury::init(treasury, root, treasury_bump)?;
    create_pda(
        payer,
        registry,
        Registry::MIN_SIZE,
        registry_seeds,
        registry_bump,
    )?;
    Registry::init(
        registry,
        root,
        root,
        crate::NO_CHAIR,
        0,
        &d.h32(0)?,
        window,
        d.u64(32)?,
        d.u64(40)?,
        now,
        registry_bump,
        &cfg.default_mint,
        root,
    )?;
    let mut head = Registry::head(registry)?;
    head.rung = rung as u8;
    head.next_sale_id = 1; // Sale zero remains reserved; no legacy auction is created.
    head.seat_reserve = terms.start_lamports;
    head.seat_deposit = 0;
    Registry::set_head(registry, &head)?;
    super::council::seat_founders(registry, &data[154..end], now)?;
    Registry::set_name(registry, name, desc)?;
    create_pda(payer, curve, MintCurve::SIZE, curve_seeds, terms.bump)?;
    curve.try_borrow_mut()?.copy_from_slice(&encoded_terms);
    cfg.mode = crate::MODE_GENESIS;
    cfg.write(config)
}

fn borsh_text<'a>(data: &'a [u8], cursor: &mut usize, max: usize) -> Result<&'a str, ProgramError> {
    let length_end = cursor
        .checked_add(4)
        .ok_or(ProgramError::InvalidInstructionData)?;
    let length_bytes = data
        .get(*cursor..length_end)
        .ok_or(ProgramError::InvalidInstructionData)?;
    let length = u32::from_le_bytes(
        length_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;
    if length == 0 || length > max {
        return Err(ProgramError::InvalidInstructionData);
    }
    let end = length_end
        .checked_add(length)
        .ok_or(ProgramError::InvalidInstructionData)?;
    let text = core::str::from_utf8(
        data.get(length_end..end)
            .ok_or(ProgramError::InvalidInstructionData)?,
    )
    .map_err(|_| ProgramError::InvalidInstructionData)?;
    *cursor = end;
    Ok(text)
}

fn mint_data(data: &[u8]) -> Result<(u64, Metadata<'_>), ProgramError> {
    let max_price = Data(data).u64(0)?;
    let mut cursor = 8;
    let metadata = Metadata {
        name: borsh_text(data, &mut cursor, nft_wire::MAX_NAME_BYTES)?,
        symbol: borsh_text(data, &mut cursor, nft_wire::MAX_SYMBOL_BYTES)?,
        uri: borsh_text(data, &mut cursor, nft_wire::MAX_URI_BYTES)?,
    };
    if cursor != data.len() {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok((max_price, metadata))
}

/// The genuine Instructions sysvar describes top-level transaction
/// instructions. Binding its current instruction to this exact entrypoint and
/// payload refuses wrapper CPI / generic ACT entry. The runtime, not this byte
/// parser, verifies the preceding Ed25519 signature before this handler runs.
fn verified_permit(
    sysvar: &AccountView,
    data: &[u8],
    issuer: &[u8; 32],
    context: &PermitContext,
) -> Result<PermitFields, ProgramError> {
    if sysvar.address() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }
    let instructions = Instructions::try_from(sysvar)?;
    let current = instructions.get_instruction_relative(0)?;
    let current_data = current.get_instruction_data();
    if current.get_program_id() != &crate::ID
        || current_data.first() != Some(&crate::IX_MINT_NFT)
        || current_data.get(1..) != Some(data)
    {
        return Err(PoleisError::BadPrecompile.into());
    }
    let previous = instructions.get_instruction_relative(-1)?;
    let bytes = previous.get_instruction_data();
    if bytes.len() != completion_permit::ED25519_DATA_LEN {
        return Err(PoleisError::BadPrecompile.into());
    }
    let permit = &bytes[112..];
    completion_permit::check_ed25519_instruction(
        previous.get_program_id().as_array(),
        bytes,
        issuer,
        permit,
    )
    .map_err(permit_error)?;
    completion_permit::validate_for(permit, context).map_err(permit_error)
}

/// Allocate a Token-2022-owned PDA even if somebody pre-funded its address.
/// Initial allocation and rent allocation deliberately differ for metadata.
fn create_token_pda(
    payer: &AccountView,
    account: &AccountView,
    initial: usize,
    final_len: usize,
    seeds: &[&[u8]],
    bump: u8,
) -> ProgramResult {
    fresh(account)?;
    let rent = Rent::get()?.try_minimum_balance(final_len)?;
    let bump_bytes = [bump];
    let mut seed_array: [Seed; 8] = core::array::from_fn(|_| Seed::from(&[] as &[u8]));
    if seeds.len() + 1 > seed_array.len() {
        return Err(PoleisError::InvalidPda.into());
    }
    for (i, seed) in seeds.iter().enumerate() {
        seed_array[i] = Seed::from(*seed);
    }
    seed_array[seeds.len()] = Seed::from(&bump_bytes[..]);
    let signers = [Signer::from(&seed_array[..seeds.len() + 1])];
    if account.lamports() == 0 {
        pinocchio_system::instructions::CreateAccount {
            from: payer,
            to: account,
            lamports: rent,
            space: initial as u64,
            owner: &crate::TOKEN_2022_PROGRAM_ID,
        }
        .invoke_signed(&signers)?;
    } else {
        let top_up = rent.saturating_sub(account.lamports());
        if top_up > 0 {
            pinocchio_system::instructions::Transfer {
                from: payer,
                to: account,
                lamports: top_up,
            }
            .invoke()?;
        }
        pinocchio_system::instructions::Allocate {
            account,
            space: initial as u64,
        }
        .invoke_signed(&signers)?;
        pinocchio_system::instructions::Assign {
            account,
            owner: &crate::TOKEN_2022_PROGRAM_ID,
        }
        .invoke_signed(&signers)?;
    }
    Ok(())
}

fn token_cpi<const N: usize>(
    program: &AccountView,
    accounts: [&AccountView; N],
    flags: [(bool, bool); N],
    data: &[u8],
    signers: &[Signer<'_, '_>],
) -> ProgramResult {
    let metas: [InstructionAccount; N] = core::array::from_fn(|i| {
        InstructionAccount::new(accounts[i].address(), flags[i].0, flags[i].1)
    });
    invoke_signed(
        &InstructionView {
            program_id: program.address(),
            accounts: &metas,
            data,
        },
        &accounts,
        signers,
    )
}

/// Separate frame keeps bounded metadata/CPI buffers off the sale frame.
#[allow(clippy::too_many_arguments)]
fn issue_token(
    payer: &AccountView,
    mint: &AccountView,
    token: &AccountView,
    seat: &AccountView,
    program: &AccountView,
    metadata: Metadata<'_>,
    metadata_bytes: &[u8],
    unit_le: &[u8; 8],
    completion: &[u8; 32],
    seat_bump: u8,
) -> ProgramResult {
    let mint_key = *mint.address().as_array();
    let mint_seeds: &[&[u8]] = &[MINT_SEED, unit_le, completion];
    let mint_bump = find_and_expect_pda(mint, mint_seeds)?;
    let token_seeds: &[&[u8]] = &[TOKEN_SEED, &mint_key, payer.address().as_array()];
    let token_bump = find_and_expect_pda(token, token_seeds)?;
    create_token_pda(
        payer,
        mint,
        nft_wire::MINT_INITIAL_LEN,
        nft_wire::metadata_mint_len(metadata).map_err(wire_error)?,
        mint_seeds,
        mint_bump,
    )?;
    token_cpi(
        program,
        [mint],
        [(true, false)],
        &nft_wire::initialize_transfer_hook(crate::ID.as_array()).map_err(wire_error)?,
        &[],
    )?;
    token_cpi(
        program,
        [mint],
        [(true, false)],
        &nft_wire::initialize_metadata_pointer(&mint_key).map_err(wire_error)?,
        &[],
    )?;
    token_cpi(
        program,
        [mint],
        [(true, false)],
        &nft_wire::initialize_mint2(seat.address().as_array()).map_err(wire_error)?,
        &[],
    )?;
    let bump_bytes = [seat_bump];
    let seeds = [
        Seed::from(SEAT_SEED),
        Seed::from(&mint_key[..]),
        Seed::from(&bump_bytes[..]),
    ];
    let signers = [Signer::from(&seeds[..])];
    // Repeated mint/authority metas and corresponding account views are required
    // by the SPL metadata interface, in this exact order.
    token_cpi(
        program,
        [mint, seat, mint, seat],
        [(true, false), (false, false), (false, false), (false, true)],
        metadata_bytes,
        &signers,
    )?;
    token_cpi(
        program,
        [mint, seat],
        [(true, false), (false, true)],
        &nft_wire::revoke_metadata_authority(),
        &signers,
    )?;
    create_token_pda(
        payer,
        token,
        nft_wire::TOKEN_ACCOUNT_LEN,
        nft_wire::TOKEN_ACCOUNT_LEN,
        token_seeds,
        token_bump,
    )?;
    token_cpi(
        program,
        [token],
        [(true, false)],
        &nft_wire::initialize_immutable_owner(),
        &[],
    )?;
    token_cpi(
        program,
        [token, mint],
        [(true, false), (false, false)],
        &nft_wire::initialize_account3(payer.address().as_array()).map_err(wire_error)?,
        &[],
    )?;
    token_cpi(
        program,
        [mint, token, seat],
        [(true, false), (true, false), (false, true)],
        &nft_wire::mint_to_one_checked(),
        &signers,
    )?;
    token_cpi(
        program,
        [mint, seat],
        [(true, false), (false, true)],
        &nft_wire::revoke_mint_authority(),
        &signers,
    )?;
    if !mint.owned_by(&crate::TOKEN_2022_PROGRAM_ID)
        || !token.owned_by(&crate::TOKEN_2022_PROGRAM_ID)
    {
        return Err(ProgramError::IncorrectProgramId);
    }
    seat_nft::verify_owner(
        RawAccount {
            address: &mint_key,
            program_owner: &seat_nft::TOKEN_2022_OWNER,
            data: &mint.try_borrow()?,
        },
        RawAccount {
            address: token.address().as_array(),
            program_owner: &seat_nft::TOKEN_2022_OWNER,
            data: &token.try_borrow()?,
        },
        &mint_key,
        crate::ID.as_array(),
        payer.address().as_array(),
        payer.is_signer(),
    )
    .map_err(super::nft::token_error)?;
    Ok(())
}

/// Tag 47 data: buyer max price u64LE, then name/symbol/URI as bounded UTF-8
/// Borsh strings (u32LE byte lengths), with no trailing bytes. The permit occurs
/// ONLY in the immediately preceding Ed25519 instruction, avoiding duplication.
/// Accounts: buyer(s,w), config, curve(w), registry(w), treasury(w), seat(w),
/// mint(w), token(w), validation(w), fund(w), tally(w), instructions, token2022, system.
pub fn mint_nft(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [buyer, config, curve_account, registry, treasury, seat, mint, token, validation, fund, tally, instructions, token_program, system] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(buyer)?;
    for account in [
        &*buyer,
        &*curve_account,
        &*registry,
        &*treasury,
        &*fund,
        &*tally,
    ] {
        expect_writable(account)?;
    }
    system_program(system)?;
    if token_program.address() != &crate::TOKEN_2022_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if buyer.data_len() != 0 || !buyer.owned_by(&Address::default()) {
        return Err(PoleisError::InvalidAccount.into());
    }
    for account in [&*seat, &*mint, &*token, &*validation] {
        fresh(account)?;
    }
    let cfg = super::read_config(config)?;
    if cfg.mode != crate::MODE_GENESIS && cfg.mode != crate::MODE_COUNCIL {
        return Err(PoleisError::InvalidMode.into());
    }
    let mut curve = MintCurve::read(curve_account)?;
    if curve.unit != cfg.root {
        return Err(PoleisError::UnitMismatch.into());
    }
    let mut head = super::read_registry(registry, curve.unit)?;
    if head.root != cfg.root {
        return Err(PoleisError::UnitMismatch.into());
    }
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }
    super::read_treasury(treasury, curve.unit)?;
    let now = Clock::get()?.slot;
    let (buyer_max, metadata) = mint_data(data)?;
    let quote = curve.quote(now, buyer_max)?;
    let mut metadata_buffer = [0; nft_wire::MAX_METADATA_INITIALIZE_LEN];
    let metadata_len =
        nft_wire::initialize_metadata(metadata, &mut metadata_buffer).map_err(wire_error)?;
    let metadata_bytes = &metadata_buffer[..metadata_len];
    let permit = verified_permit(
        instructions,
        data,
        &curve.completion_issuer,
        &PermitContext {
            program_id: *crate::ID.as_array(),
            chain_genesis_hash: curve.genesis_hash,
            unit: curve.unit,
            curve: *curve_account.address().as_array(),
            buyer: *buyer.address().as_array(),
            buyer_is_signer: buyer.is_signer(),
            metadata_digest: sha256v(&[metadata_bytes]),
            current_slot: now,
        },
    )?;
    completion_permit::check_price(&permit, buyer_max, quote.price_lamports)
        .map_err(permit_error)?;
    let unit_le = curve.unit.to_le_bytes();
    find_and_expect_pda(mint, &[MINT_SEED, &unit_le, &permit.completion_digest])?;
    let mint_key = *mint.address().as_array();
    let seat_seeds: &[&[u8]] = &[SEAT_SEED, &mint_key];
    let seat_bump = find_and_expect_pda(seat, seat_seeds)?;
    find_and_expect_pda(token, &[TOKEN_SEED, &mint_key, buyer.address().as_array()])?;
    let validation_seeds: &[&[u8]] = &[nft_wire::EXTRA_ACCOUNT_METAS_SEED, &mint_key];
    let validation_bump = find_and_expect_pda(validation, validation_seeds)?;
    let row = head.holder_count as usize;
    if row >= crate::MAX_HOLDERS {
        return Err(PoleisError::RegistryFull.into());
    }
    let total_after = head
        .total_seats
        .checked_add(1)
        .ok_or(PoleisError::MathOverflow)?;
    if head.rung as usize >= crate::LADDER.len() {
        return Err(PoleisError::NotOnLadder.into());
    }
    if total_after > crate::ladder_hexagons(head.rung as usize) {
        return Err(PoleisError::SeatLimit.into());
    }
    let seat_receipt = NftSeat {
        bump: seat_bump,
        unit: curve.unit,
        mint: mint_key,
        completion: permit.completion_digest,
        serial: curve.minted_count,
        paid_lamports: quote.price_lamports,
        first_buyer: *buyer.address().as_array(),
    };
    let encoded_receipt = seat_receipt.encode()?;
    // No historical holder row is reused, even after burning or expulsion.
    grow_account(buyer, registry, Registry::size_for(row + 1))?;
    create_pda(buyer, seat, NftSeat::SIZE, seat_seeds, seat_bump)?;
    create_pda(
        buyer,
        validation,
        nft_wire::EXTRA_ACCOUNT_METAS_LEN,
        validation_seeds,
        validation_bump,
    )?;
    validation
        .try_borrow_mut()?
        .copy_from_slice(&nft_wire::extra_account_metas(
            seat.address().as_array(),
            registry.address().as_array(),
        ));
    issue_token(
        buyer,
        mint,
        token,
        seat,
        token_program,
        metadata,
        metadata_bytes,
        &unit_le,
        &permit.completion_digest,
        seat_bump,
    )?;
    // charge_fee may create a tally via CPI. No lamports have been moved by
    // hand, and every account-data borrow above has already been released.
    let fee = super::fund::charge_fee(
        fund,
        tally,
        buyer,
        curve.unit,
        cfg.root,
        quote.price_lamports,
    )?;
    let net = quote
        .price_lamports
        .checked_sub(fee)
        .ok_or(PoleisError::MathOverflow)?;
    if net > 0 {
        pinocchio_system::instructions::Transfer {
            from: buyer,
            to: treasury,
            lamports: net,
        }
        .invoke()?;
    }
    if fee > 0 {
        pinocchio_system::instructions::Transfer {
            from: buyer,
            to: fund,
            lamports: fee,
        }
        .invoke()?;
    }
    Treasury::add_in(treasury, net)?;
    Registry::set_holder(
        registry,
        row,
        &Holder {
            holder: *seat.address().as_array(),
            kind: crate::HOLDER_NFT,
            seats: 1,
            weight: 1,
            last_active: now,
        },
    )?;
    head.holder_count = (row + 1) as u8;
    head.total_seats = total_after;
    head.last_clearing = quote.price_lamports;
    Registry::set_head(registry, &head)?;
    seat.try_borrow_mut()?.copy_from_slice(&encoded_receipt);
    curve.minted_count = quote.minted_count_after;
    curve_account
        .try_borrow_mut()?
        .copy_from_slice(&curve.encode()?);
    receipt(
        crate::RCPT_SEATS,
        curve.unit,
        &permit.completion_digest,
        buyer.address().as_array(),
        &mint_key,
        quote.price_lamports,
        1,
    );
    Ok(())
}

/// Tag 48 data empty. Accounts: crank(s), config(w), registry, curve.
/// The existing council authority takes over once this sale closes or sells
/// out and a live seat exists. No legacy sale account is fabricated or changed.
pub fn ratify_nft_genesis(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, config, registry, curve_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(crank)?;
    expect_writable(config)?;
    let mut cfg = super::read_config(config)?;
    if cfg.mode != crate::MODE_GENESIS {
        return Err(PoleisError::InvalidMode.into());
    }
    let curve = MintCurve::read(curve_account)?;
    if curve.unit != cfg.root {
        return Err(PoleisError::UnitMismatch.into());
    }
    let head = super::read_registry(registry, cfg.root)?;
    if head.root != cfg.root {
        return Err(PoleisError::UnitMismatch.into());
    }
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }
    if curve.minted_count == 0 || head.total_seats == 0 {
        return Err(PoleisError::NotHolder.into());
    }
    if Clock::get()?.slot < curve.closes_slot && curve.minted_count < curve.capacity {
        return Err(PoleisError::NotYet.into());
    }
    let (council, _) =
        Address::find_program_address(&[b"council", &cfg.root.to_le_bytes()], &crate::ID);
    cfg.authority = council;
    cfg.mode = crate::MODE_COUNCIL;
    cfg.write(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata_data() -> [u8; 38] {
        let mut bytes = [0; 38];
        bytes[..8].copy_from_slice(&10u64.to_le_bytes());
        let mut cursor = 8;
        for value in ["Seat", "SOL", "ipfs://test"] {
            bytes[cursor..cursor + 4].copy_from_slice(&(value.len() as u32).to_le_bytes());
            cursor += 4;
            bytes[cursor..cursor + value.len()].copy_from_slice(value.as_bytes());
            cursor += value.len();
        }
        bytes
    }

    #[test]
    fn compact_mint_data_has_exact_bounded_utf8_fields() {
        let data = metadata_data();
        let (price, metadata) = mint_data(&data).unwrap();
        assert_eq!(price, 10);
        assert_eq!(
            metadata,
            Metadata {
                name: "Seat",
                symbol: "SOL",
                uri: "ipfs://test"
            }
        );
        for length in 0..data.len() {
            assert!(mint_data(&data[..length]).is_err());
        }
        let mut trailing = [0; 39];
        trailing[..38].copy_from_slice(&data);
        assert!(mint_data(&trailing).is_err());
        let mut empty = data.clone();
        empty[8..12].copy_from_slice(&0u32.to_le_bytes());
        assert!(mint_data(&empty).is_err());
        let mut huge = data.clone();
        huge[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(mint_data(&huge).is_err());
        let mut invalid_utf8 = data;
        invalid_utf8[12] = 255;
        assert!(mint_data(&invalid_utf8).is_err());
    }
}
