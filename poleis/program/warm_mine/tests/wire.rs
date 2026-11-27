//! THE WIRE IS THE TOKEN PROGRAMS' OWN: every SPL Token and Associated Token instruction the program calls (src/token.rs, written by
//! hand as small byte layouts: no token crate is in the house's cache) held to the encoding those programs' own interface crates build
//! (spl-token-interface 2, spl-associated-token-account-interface 2), byte for byte and account by account, each account's write and
//! signature; the two account layouts it reads held to theirs; and the upgradeable loader's program data held to Mollusk's own.
use solana_instruction::Instruction;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use spl_associated_token_account_interface::{address::get_associated_token_address_with_program_id, instruction::create_associated_token_account};
use spl_token_interface::instruction as spl;
use warm_mine::token::*;

fn flags(ix: &Instruction) -> Vec<(bool, bool)> {
    ix.accounts.iter().map(|m| (m.is_writable, m.is_signer)).collect()
}
fn token_id() -> Pubkey {
    spl_token_interface::id()
}

#[test]
fn the_programs_are_theirs_by_their_ids() {
    assert_eq!(TOKEN_PROGRAM_ID, token_id(), "the token program");
    assert_eq!(ATA_PROGRAM_ID, spl_associated_token_account_interface::program::id(), "the associated token program");
    assert_eq!(LOADER_V3_ID, mollusk_svm::program::loader_keys::LOADER_V3, "the upgradeable loader");
}

#[test]
fn initialize_mint2_is_the_token_programs_own() {
    let (mint, authority) = (Pubkey::new_unique(), Pubkey::new_unique());
    let theirs = spl::initialize_mint2(&token_id(), &mint, &authority, None, WARM_DECIMALS).unwrap();
    assert_eq!(theirs.program_id, TOKEN_PROGRAM_ID);
    assert_eq!(theirs.data, initialize_mint2_data(WARM_DECIMALS, &authority).to_vec(), "its data: no freeze authority");
    assert_eq!((theirs.data[0], flags(&theirs)), (IX_INITIALIZE_MINT2, INITIALIZE_MINT2_ACCOUNTS.to_vec()));
    assert_eq!(theirs.accounts[0].pubkey, mint);
}

#[test]
fn initialize_account3_is_the_token_programs_own() {
    let (account, mint, owner) = (Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique());
    let theirs = spl::initialize_account3(&token_id(), &account, &mint, &owner).unwrap();
    assert_eq!(theirs.data, initialize_account3_data(&owner).to_vec());
    assert_eq!((theirs.data[0], flags(&theirs)), (IX_INITIALIZE_ACCOUNT3, INITIALIZE_ACCOUNT3_ACCOUNTS.to_vec()));
    assert_eq!((theirs.accounts[0].pubkey, theirs.accounts[1].pubkey), (account, mint));
}

#[test]
fn mint_to_is_the_token_programs_own() {
    let (mint, account, authority) = (Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique());
    for amount in [0, 1, 500_000_000, u64::MAX] {
        let theirs = spl::mint_to(&token_id(), &mint, &account, &authority, &[], amount).unwrap();
        assert_eq!(theirs.data, mint_to_data(amount).to_vec(), "{amount}");
        assert_eq!((theirs.data[0], flags(&theirs)), (IX_MINT_TO, MINT_TO_ACCOUNTS.to_vec()));
        assert_eq!([theirs.accounts[0].pubkey, theirs.accounts[1].pubkey, theirs.accounts[2].pubkey], [mint, account, authority]);
    }
}

#[test]
fn transfer_checked_is_the_token_programs_own() {
    let (source, mint, destination, authority) = (Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique());
    for (amount, decimals) in [(10_000_000, DOLLAR_DECIMALS), (1, WARM_DECIMALS), (u64::MAX, WARM_DECIMALS)] {
        let theirs = spl::transfer_checked(&token_id(), &source, &mint, &destination, &authority, &[], amount, decimals).unwrap();
        assert_eq!(theirs.data, transfer_checked_data(amount, decimals).to_vec(), "{amount} at {decimals}");
        assert_eq!((theirs.data[0], flags(&theirs)), (IX_TRANSFER_CHECKED, TRANSFER_CHECKED_ACCOUNTS.to_vec()));
        let keys: Vec<Pubkey> = theirs.accounts.iter().map(|m| m.pubkey).collect();
        assert_eq!(keys, vec![source, mint, destination, authority]);
    }
}

#[test]
fn approve_checked_is_the_token_programs_own() {
    let (source, mint, delegate, owner) = (Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique());
    for amount in [1, WARM_DECIMALS as u64, u64::MAX] {
        let theirs = spl::approve_checked(&token_id(), &source, &mint, &delegate, &owner, &[], amount, WARM_DECIMALS).unwrap();
        assert_eq!(theirs.data, approve_checked_data(amount, WARM_DECIMALS).to_vec(), "{amount}");
        assert_eq!((theirs.data[0], flags(&theirs)), (IX_APPROVE_CHECKED, APPROVE_CHECKED_ACCOUNTS.to_vec()));
        let keys: Vec<Pubkey> = theirs.accounts.iter().map(|m| m.pubkey).collect();
        assert_eq!(keys, vec![source, mint, delegate, owner]);
    }
    // the delegate and what it may still move, where the token program keeps them
    let mut account = spl_token_interface::state::Account::default();
    (account.delegate, account.delegated_amount) = (Some(delegate).into(), 7);
    let mut d = vec![0u8; ACCOUNT_LEN];
    spl_token_interface::state::Account::pack(account, &mut d).unwrap();
    assert_eq!((&d[ACCOUNT_OFF_DELEGATE..ACCOUNT_OFF_DELEGATE + 4], &d[ACCOUNT_OFF_DELEGATE + 4..ACCOUNT_OFF_DELEGATE + 36], &d[ACCOUNT_OFF_DELEGATED..ACCOUNT_OFF_DELEGATED + 8]), (&[1u8, 0, 0, 0][..], delegate.as_ref(), &7u64.to_le_bytes()[..]));
}

#[test]
fn the_associated_account_and_its_create_are_the_associated_programs_own() {
    let (payer, wallet, mint) = (Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique());
    let theirs = create_associated_token_account(&payer, &wallet, &mint, &token_id());
    let address = get_associated_token_address_with_program_id(&wallet, &mint, &token_id());
    assert_eq!(theirs.program_id, ATA_PROGRAM_ID);
    assert_eq!(theirs.data, ATA_CREATE.to_vec(), "Create, not CreateIdempotent");
    assert_eq!(flags(&theirs), ATA_CREATE_ACCOUNTS.to_vec());
    let keys: Vec<Pubkey> = theirs.accounts.iter().map(|m| m.pubkey).collect();
    assert_eq!(keys, vec![payer, address, wallet, mint, Pubkey::default(), token_id()], "funding, the account, its wallet, the mint, system, token");
    // the address a record keeps for its flask's $WARM (ix.rs record_of): the same seeds, the same program
    let ours = Pubkey::find_program_address(&[wallet.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()], &ATA_PROGRAM_ID).0;
    assert_eq!(ours, address);
}

#[test]
fn the_mint_and_the_account_are_read_where_the_token_program_writes_them() {
    let (authority, mint, owner) = (Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique());
    let mut m = vec![0u8; MINT_LEN];
    m[MINT_OFF_AUTHORITY..MINT_OFF_AUTHORITY + 4].copy_from_slice(&1u32.to_le_bytes());
    m[MINT_OFF_AUTHORITY + 4..MINT_OFF_AUTHORITY + 36].copy_from_slice(authority.as_ref());
    m[MINT_OFF_SUPPLY..MINT_OFF_SUPPLY + 8].copy_from_slice(&123_456_789u64.to_le_bytes());
    m[MINT_OFF_DECIMALS] = DOLLAR_DECIMALS;
    m[MINT_OFF_INITIALIZED] = 1;
    let theirs = spl_token_interface::state::Mint::unpack(&m).unwrap();
    let (a, f): (Option<Pubkey>, Option<Pubkey>) = (theirs.mint_authority.into(), theirs.freeze_authority.into());
    assert_eq!((a, theirs.supply, theirs.decimals, theirs.is_initialized, f), (Some(authority), 123_456_789, DOLLAR_DECIMALS, true, None));
    assert_eq!((spl_token_interface::state::Mint::LEN, MINT_OFF_FREEZE + 36), (MINT_LEN, MINT_LEN));

    let mut t = vec![0u8; ACCOUNT_LEN];
    t[ACCOUNT_OFF_MINT..ACCOUNT_OFF_MINT + 32].copy_from_slice(mint.as_ref());
    t[ACCOUNT_OFF_OWNER..ACCOUNT_OFF_OWNER + 32].copy_from_slice(owner.as_ref());
    t[ACCOUNT_OFF_AMOUNT..ACCOUNT_OFF_AMOUNT + 8].copy_from_slice(&42u64.to_le_bytes());
    t[ACCOUNT_OFF_STATE] = STATE_INITIALIZED;
    let theirs = spl_token_interface::state::Account::unpack(&t).unwrap();
    assert_eq!((theirs.mint, theirs.owner, theirs.amount), (mint, owner, 42));
    assert_eq!(theirs.state, spl_token_interface::state::AccountState::Initialized);
    assert_eq!(spl_token_interface::state::Account::LEN, ACCOUNT_LEN);
    t[ACCOUNT_OFF_STATE] = 2;
    assert_eq!(spl_token_interface::state::Account::unpack(&t).unwrap().state, spl_token_interface::state::AccountState::Frozen, "2 is frozen, refused by the program");
}

#[test]
fn the_program_data_is_the_loaders_own() {
    // Mollusk lays a program and its data as the upgradeable loader does: the program names its data at the loader's PDA of its id, and
    // the data's header is [3:4][slot:8][an option of the authority: 0, or 1 and 32]
    let id = Pubkey::new_from_array(warm_mine::ID.to_bytes());
    let program = mollusk_svm::program::create_program_account_loader_v3(&id);
    let at = Pubkey::find_program_address(&[id.as_ref()], &LOADER_V3_ID).0;
    assert_eq!((&program.data[..4], &program.data[4..36]), (&[2u8, 0, 0, 0][..], at.as_ref()));
    let data = mollusk_svm::program::create_program_data_account_loader_v3(&[7u8; 16]);
    assert_eq!((&data.data[..4], &data.data[4..12], data.data[12], &data.data[45..]), (&[3u8, 0, 0, 0][..], &[0u8; 8][..], 0, &[7u8; 16][..]), "no authority: none opens it");
    assert_eq!(data.owner, LOADER_V3_ID);
}
