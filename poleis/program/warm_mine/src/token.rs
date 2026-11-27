//! The SPL Token and Associated Token instructions this program calls, and the two account layouts it reads, written by hand as small
//! byte layouts (no token crate is in the house's cache: `pinocchio-token` never came in, and no crate is fetched for this lane). Each
//! one is held to the programs' own published encoding by `tests/wire.rs` (spl-token-interface 2 and spl-associated-token-account-interface
//! 2, the crates those programs' clients build their instructions with): the tag and the bytes of its data, and each account's place, its
//! write and its signature.

use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

use crate::error::WarmError;

/// The SPL Token program, `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`: USDC's, and $WARM's.
pub const TOKEN_PROGRAM_ID: Address = solana_address::address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
/// The Associated Token Account program, `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`.
pub const ATA_PROGRAM_ID: Address = solana_address::address!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
/// The upgradeable loader, `BPFLoaderUpgradeab1e11111111111111111111111`: its program data names who may upgrade this program.
pub const LOADER_V3_ID: Address = solana_address::address!("BPFLoaderUpgradeab1e11111111111111111111111");

/// USDC's decimals: a deposit counts micros of a dollar.
pub const DOLLAR_DECIMALS: u8 = 6;
/// $WARM's decimals: it is mined in nanos.
pub const WARM_DECIMALS: u8 = 9;

// ---- the two layouts (spl-token-interface's `state::Mint` and `state::Account`, packed) ----
/// A mint: [mint_authority: an option, 4 + 32][supply:8][decimals:1][is_initialized:1][freeze_authority: an option, 4 + 32].
pub const MINT_LEN: usize = 82;
pub const MINT_OFF_AUTHORITY: usize = 0;
pub const MINT_OFF_SUPPLY: usize = 36;
pub const MINT_OFF_DECIMALS: usize = 44;
pub const MINT_OFF_INITIALIZED: usize = 45;
pub const MINT_OFF_FREEZE: usize = 46;
/// A token account: [mint:32][owner:32][amount:8][delegate: an option, 4 + 32][state:1] and the rest.
pub const ACCOUNT_LEN: usize = 165;
pub const ACCOUNT_OFF_MINT: usize = 0;
pub const ACCOUNT_OFF_OWNER: usize = 32;
pub const ACCOUNT_OFF_AMOUNT: usize = 64;
pub const ACCOUNT_OFF_STATE: usize = 108;
/// A token account's state: initialized (0 is not, 2 is frozen).
pub const STATE_INITIALIZED: u8 = 1;

// ---- the instructions: each one's tag, its data, and its accounts as (writable, signer) in their order ----
/// `InitializeMint2`: [20][decimals:1][mint_authority:32][freeze_authority: 0 none | 1 and 32]; accounts [mint (w)].
pub const IX_INITIALIZE_MINT2: u8 = 20;
pub const INITIALIZE_MINT2_ACCOUNTS: [(bool, bool); 1] = [(true, false)];
pub fn initialize_mint2_data(decimals: u8, authority: &Address) -> [u8; 35] {
    let mut d = [0u8; 35];
    d[0] = IX_INITIALIZE_MINT2;
    d[1] = decimals;
    d[2..34].copy_from_slice(authority.as_ref());
    // d[34] = 0: no freeze authority, ever
    d
}
/// `InitializeAccount3`: [18][owner:32]; accounts [account (w), mint].
pub const IX_INITIALIZE_ACCOUNT3: u8 = 18;
pub const INITIALIZE_ACCOUNT3_ACCOUNTS: [(bool, bool); 2] = [(true, false), (false, false)];
pub fn initialize_account3_data(owner: &Address) -> [u8; 33] {
    let mut d = [0u8; 33];
    d[0] = IX_INITIALIZE_ACCOUNT3;
    d[1..33].copy_from_slice(owner.as_ref());
    d
}
/// `MintTo`: [7][amount:8]; accounts [mint (w), account (w), authority (s)].
pub const IX_MINT_TO: u8 = 7;
pub const MINT_TO_ACCOUNTS: [(bool, bool); 3] = [(true, false), (true, false), (false, true)];
pub fn mint_to_data(amount: u64) -> [u8; 9] {
    let mut d = [0u8; 9];
    d[0] = IX_MINT_TO;
    d[1..9].copy_from_slice(&amount.to_le_bytes());
    d
}
/// `TransferChecked`: [12][amount:8][decimals:1]; accounts [source (w), mint, destination (w), authority (s)].
pub const IX_TRANSFER_CHECKED: u8 = 12;
pub const TRANSFER_CHECKED_ACCOUNTS: [(bool, bool); 4] = [(true, false), (false, false), (true, false), (false, true)];
pub fn transfer_checked_data(amount: u64, decimals: u8) -> [u8; 10] {
    let mut d = [0u8; 10];
    d[0] = IX_TRANSFER_CHECKED;
    d[1..9].copy_from_slice(&amount.to_le_bytes());
    d[9] = decimals;
    d
}
/// `ApproveChecked` (13): accounts [source (w), mint, delegate, owner (s)]; data [13][amount:8][decimals:1]. Never called by this program: it
/// is the one approval a flask signs in its page to make the mine the delegate of its $WARM (SPENT AS CREDITS, IT RETURNS), written here so its
/// bytes are held to the token program's own (tests/wire.rs) and the door's WASM writes them from this.
pub const IX_APPROVE_CHECKED: u8 = 13;
pub const APPROVE_CHECKED_ACCOUNTS: [(bool, bool); 4] = [(true, false), (false, false), (false, false), (false, true)];
pub fn approve_checked_data(amount: u64, decimals: u8) -> [u8; 10] {
    let mut d = transfer_checked_data(amount, decimals);
    d[0] = IX_APPROVE_CHECKED;
    d
}
/// A token account's delegate (an option: [tag:4][key:32] at 72) and what it may still move (at 121).
pub const ACCOUNT_OFF_DELEGATE: usize = 72;
pub const ACCOUNT_OFF_DELEGATED: usize = 121;
/// The Associated Token program's `Create` (0): accounts [funding (s, w), the account (w), its wallet, the mint, the system program, the
/// token program].
pub const ATA_CREATE: [u8; 1] = [0];
pub const ATA_CREATE_ACCOUNTS: [(bool, bool); 6] = [(true, true), (true, false), (false, false), (false, false), (false, false), (false, false)];

/// One instruction of `program` over `accounts`, each flagged by `flags`, signed for by `signers`.
fn call<const N: usize>(program: &Address, flags: &[(bool, bool); N], accounts: [&AccountView; N], data: &[u8], signers: &[Signer]) -> ProgramResult {
    let metas: [InstructionAccount; N] = core::array::from_fn(|i| InstructionAccount::new(accounts[i].address(), flags[i].0, flags[i].1));
    let ix = InstructionView { program_id: program, accounts: &metas, data };
    invoke_signed::<N, &AccountView>(&ix, &accounts, signers)
}

pub fn initialize_mint2(mint: &AccountView, decimals: u8, authority: &Address) -> ProgramResult {
    call(&TOKEN_PROGRAM_ID, &INITIALIZE_MINT2_ACCOUNTS, [mint], &initialize_mint2_data(decimals, authority), &[])
}
pub fn initialize_account3(account: &AccountView, mint: &AccountView, owner: &Address) -> ProgramResult {
    call(&TOKEN_PROGRAM_ID, &INITIALIZE_ACCOUNT3_ACCOUNTS, [account, mint], &initialize_account3_data(owner), &[])
}
pub fn mint_to(mint: &AccountView, account: &AccountView, authority: &AccountView, amount: u64, signers: &[Signer]) -> ProgramResult {
    call(&TOKEN_PROGRAM_ID, &MINT_TO_ACCOUNTS, [mint, account, authority], &mint_to_data(amount), signers)
}
#[allow(clippy::too_many_arguments)]
pub fn transfer_checked(
    source: &AccountView,
    mint: &AccountView,
    destination: &AccountView,
    authority: &AccountView,
    amount: u64,
    decimals: u8,
    signers: &[Signer],
) -> ProgramResult {
    call(&TOKEN_PROGRAM_ID, &TRANSFER_CHECKED_ACCOUNTS, [source, mint, destination, authority], &transfer_checked_data(amount, decimals), signers)
}
pub fn create_associated(payer: &AccountView, account: &AccountView, wallet: &AccountView, mint: &AccountView, system: &AccountView, token_program: &AccountView) -> ProgramResult {
    call(&ATA_PROGRAM_ID, &ATA_CREATE_ACCOUNTS, [payer, account, wallet, mint, system, token_program], &ATA_CREATE, &[])
}

/// A token program's account given where the token program must be.
pub fn expect_token_program(acc: &AccountView) -> Result<(), ProgramError> {
    if acc.address() != &TOKEN_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

/// What this program reads of a token account.
pub struct TokenAccount {
    pub mint: Address,
    pub owner: Address,
    pub amount: u64,
}
/// An initialized token account of the token program (a frozen one moves nothing, and is refused here first).
pub fn token_account(acc: &AccountView) -> Result<TokenAccount, ProgramError> {
    if !acc.owned_by(&TOKEN_PROGRAM_ID) {
        return Err(WarmError::InvalidAccount.into());
    }
    let d = acc.try_borrow()?;
    if d.len() != ACCOUNT_LEN || d[ACCOUNT_OFF_STATE] != STATE_INITIALIZED {
        return Err(WarmError::InvalidAccount.into());
    }
    Ok(TokenAccount {
        mint: Address::new_from_array(d[ACCOUNT_OFF_MINT..ACCOUNT_OFF_MINT + 32].try_into().unwrap()),
        owner: Address::new_from_array(d[ACCOUNT_OFF_OWNER..ACCOUNT_OFF_OWNER + 32].try_into().unwrap()),
        amount: u64::from_le_bytes(d[ACCOUNT_OFF_AMOUNT..ACCOUNT_OFF_AMOUNT + 8].try_into().unwrap()),
    })
}
/// An initialized mint of the token program: its decimals.
pub fn mint_decimals(acc: &AccountView) -> Result<u8, ProgramError> {
    if !acc.owned_by(&TOKEN_PROGRAM_ID) {
        return Err(WarmError::InvalidAccount.into());
    }
    let d = acc.try_borrow()?;
    if d.len() != MINT_LEN || d[MINT_OFF_INITIALIZED] != 1 {
        return Err(WarmError::InvalidAccount.into());
    }
    Ok(d[MINT_OFF_DECIMALS])
}
