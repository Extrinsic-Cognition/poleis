//! Token vaults. A unit holds tokens in one SPL token account per mint, at a
//! PDA of this program, whose authority is the unit's treasury account: the
//! vault moves tokens only when the treasury signs, and the treasury signs
//! only inside a passed TREASURY payload charged to the token plan. Anyone
//! may create a vault and anyone may deposit into it; the balance is the book.

use crate::error::PoleisError;
use crate::state::Treasury;
use crate::util::{expect_pda, expect_signer, expect_writable, find_and_expect_pda, Data};
use pinocchio::{
    cpi::{invoke_signed, Seed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

/// Either SPL Token program.
pub fn expect_token_program(acc: &AccountView) -> Result<(), ProgramError> {
    if acc.address() != &crate::TOKEN_PROGRAM_ID && acc.address() != &crate::TOKEN_2022_PROGRAM_ID {
        return Err(PoleisError::InvalidAccount.into());
    }
    Ok(())
}

/// The vault of `unit` for `mint`, verified as our PDA.
pub fn expect_vault(vault: &AccountView, unit_id: u64, mint: &[u8; 32]) -> Result<u8, ProgramError> {
    find_and_expect_pda(vault, &[b"vault", &unit_id.to_le_bytes(), mint])
}

/// data: [unit:8] · accounts: [payer (s,w), treasury, mint, vault (w), token program, system]
/// Permissionless. Creates the unit's token account for the mint with the
/// treasury as its authority. Init-once by construction.
pub fn create_vault(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [payer, treasury, mint, vault, token_program, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let unit_id = d.u64(0)?;
    expect_signer(payer)?;
    expect_writable(vault)?;
    expect_token_program(token_program)?;
    if !mint.owned_by(token_program.address()) {
        return Err(PoleisError::InvalidAccount.into());
    }
    let t = Treasury::read(treasury)?;
    if t.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    let unit_le = unit_id.to_le_bytes();
    expect_pda(treasury, &[b"treasury", &unit_le], t.bump)?;
    let mint_key: [u8; 32] = *mint.address().as_array();
    let bump = expect_vault(vault, unit_id, &mint_key)?;
    let bump_ref = [bump];
    let seeds = [Seed::from(&b"vault"[..]), Seed::from(&unit_le[..]), Seed::from(&mint_key[..]), Seed::from(&bump_ref[..])];
    create_token_account(payer, vault, mint, treasury, token_program, &seeds)
}

/// The escrow token account of a contract or a token distribution, keyed by
/// its hash, verified as our PDA.
pub fn expect_escrow(escrow: &AccountView, hash: &[u8; 32]) -> Result<u8, ProgramError> {
    find_and_expect_pda(escrow, &[b"escrow", hash])
}

/// Create a token account at one of our PDAs, owned by the token program,
/// with `owner` as its authority. Robust to a pre-funded address.
pub fn create_token_account(
    payer: &AccountView,
    account: &AccountView,
    mint: &AccountView,
    owner: &AccountView,
    token_program: &AccountView,
    seeds: &[Seed],
) -> ProgramResult {
    expect_token_program(token_program)?;
    if account.data_len() != 0 || !account.owned_by(&Address::default()) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let lamports = Rent::get()?.try_minimum_balance(crate::TOKEN_ACCOUNT_LEN)?;
    // a pre-funded address must not brick the account: top up, allocate, assign
    if account.lamports() == 0 {
        pinocchio_system::instructions::CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: crate::TOKEN_ACCOUNT_LEN as u64,
            owner: token_program.address(),
        }
        .invoke_signed(&[Signer::from(seeds)])?;
    } else {
        let top_up = lamports.saturating_sub(account.lamports());
        if top_up > 0 {
            pinocchio_system::instructions::Transfer { from: payer, to: account, lamports: top_up }.invoke()?;
        }
        pinocchio_system::instructions::Allocate { account, space: crate::TOKEN_ACCOUNT_LEN as u64 }
            .invoke_signed(&[Signer::from(seeds)])?;
        pinocchio_system::instructions::Assign { account, owner: token_program.address() }
            .invoke_signed(&[Signer::from(seeds)])?;
    }

    // InitializeAccount3: [18][owner:32], accounts [account (w), mint]
    let mut ix_data = [0u8; 33];
    ix_data[0] = 18;
    ix_data[1..].copy_from_slice(owner.address().as_ref());
    let metas = [InstructionAccount::writable(account.address()), InstructionAccount::readonly(mint.address())];
    let ix = InstructionView { program_id: token_program.address(), accounts: &metas, data: &ix_data };
    let views: [&AccountView; 2] = [account, mint];
    invoke_signed(&ix, &views, &[])
}

/// Close a token account whose authority is the unit's treasury, its rent
/// going to `dest`. The account must be empty: the token program refuses
/// otherwise.
pub fn close_token_account(
    unit_id: u64,
    treasury: &AccountView,
    treasury_bump: u8,
    account: &AccountView,
    dest: &AccountView,
    token_program: &AccountView,
) -> ProgramResult {
    expect_token_program(token_program)?;
    // CloseAccount: [9], accounts [account (w), dest (w), authority (s)]
    let ix_data = [9u8];
    let metas = [
        InstructionAccount::writable(account.address()),
        InstructionAccount::writable(dest.address()),
        InstructionAccount::readonly_signer(treasury.address()),
    ];
    let ix = InstructionView { program_id: token_program.address(), accounts: &metas, data: &ix_data };
    let unit_le = unit_id.to_le_bytes();
    let bump_ref = [treasury_bump];
    let seeds = [Seed::from(&b"treasury"[..]), Seed::from(&unit_le[..]), Seed::from(&bump_ref[..])];
    invoke_signed(&ix, &[account, dest, treasury], &[Signer::from(&seeds[..])])
}

/// The owner (authority) field of an SPL token account, bytes 32..64.
pub fn token_account_owner(acc: &AccountView) -> Result<[u8; 32], ProgramError> {
    let d = acc.try_borrow()?;
    if d.len() != crate::TOKEN_ACCOUNT_LEN {
        return Err(PoleisError::InvalidAccount.into());
    }
    Ok(d[32..64].try_into().unwrap())
}

/// The balance of an SPL token account, bytes 64..72.
pub fn token_balance(acc: &AccountView) -> Result<u64, ProgramError> {
    let d = acc.try_borrow()?;
    if d.len() != crate::TOKEN_ACCOUNT_LEN {
        return Err(PoleisError::InvalidAccount.into());
    }
    Ok(u64::from_le_bytes(d[64..72].try_into().unwrap()))
}

/// Move `amount` of a mint from the unit's vault to any token account, the
/// treasury signing as the vault's authority. Callers charge the plan first.
pub fn transfer_from_vault(
    unit_id: u64,
    treasury: &AccountView,
    treasury_bump: u8,
    source: &AccountView,
    dest: &AccountView,
    token_program: &AccountView,
    amount: u64,
) -> ProgramResult {
    expect_token_program(token_program)?;
    if amount == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    // Transfer: [3][amount:8], accounts [source (w), dest (w), authority (s)]
    let mut ix_data = [0u8; 9];
    ix_data[0] = 3;
    ix_data[1..].copy_from_slice(&amount.to_le_bytes());
    let metas = [
        InstructionAccount::writable(source.address()),
        InstructionAccount::writable(dest.address()),
        InstructionAccount::readonly_signer(treasury.address()),
    ];
    let ix = InstructionView { program_id: token_program.address(), accounts: &metas, data: &ix_data };
    let unit_le = unit_id.to_le_bytes();
    let bump_ref = [treasury_bump];
    let seeds = [Seed::from(&b"treasury"[..]), Seed::from(&unit_le[..]), Seed::from(&bump_ref[..])];
    invoke_signed(&ix, &[source, dest, treasury], &[Signer::from(&seeds[..])])
}
