//! THE TOKEN'S FACE on Mollusk: what `init` and `set_face` ask of Metaplex by CPI, recorded by a stand-in at Metaplex's id (a builtin that
//! keeps every instruction it is asked, its accounts with their write and signature as the runtime passed them, and answers yes: Metaplex's
//! own program is not on Mollusk, and no ELF of it is fetched), held to the program's own encoders (`warm_mine::face`). The press's wire
//! test (poleis/press/tests/metaplex.rs) holds those encoders, and the Metadata layout `set_face` reads, to Metaplex's SDK; together the two
//! hold what the program sends to what Metaplex reads, with no Metaplex on any chain of the house.
mod built;
mod common;
use common::*;
use solana_account::Account;
use solana_program_error::ProgramError;
use solana_program_runtime::solana_sbpf::program::BuiltinFunctionDefinition as _;
use solana_pubkey::Pubkey;
use std::cell::RefCell;
use warm_mine::error::WarmError;
use warm_mine::face::{self, Face, DATA_MAX};

/// One instruction the stand-in was asked: its data, and each account's key, write and signature.
type Asked = (Vec<u8>, Vec<([u8; 32], bool, bool)>);
thread_local! {
    static ASKED: RefCell<Vec<Asked>> = const { RefCell::new(Vec::new()) };
}

// one unit: the runtime refuses a builtin that consumes none ("Builtin programs must consume compute units"); Metaplex's own units are not counted
solana_program_runtime::declare_process_instruction!(MetaplexStandIn, 1, |invoke_context| {
    let ic = invoke_context.transaction_context.get_current_instruction_context()?;
    let mut accounts = Vec::new();
    for i in 0..ic.get_number_of_instruction_accounts() {
        accounts.push((ic.get_key_of_instruction_account(i)?.to_bytes(), ic.is_instruction_account_writable(i)?, ic.is_instruction_account_signer(i)?));
    }
    let data = ic.get_instruction_data().to_vec();
    ASKED.with(|a| a.borrow_mut().push((data, accounts)));
    Ok(())
});

fn asked() -> Vec<Asked> {
    ASKED.with(|a| std::mem::take(&mut *a.borrow_mut()))
}

/// A world whose Metaplex is the stand-in.
fn world() -> World {
    let mut w = World::new(WORD_S);
    w.m.program_cache.add_builtin(mollusk_svm::program::Builtin { program_id: METAPLEX, name: "metaplex_stand_in", register_fn: MetaplexStandIn::register });
    w.put(METAPLEX, mollusk_svm::program::create_keyed_account_for_builtin_program(&METAPLEX, "metaplex_stand_in").1);
    asked();
    w
}

const OPENING: u64 = 1_794_700_800_000;
const NAME: &[u8] = b"SWARM";
const SYMBOL: &[u8] = b"$WARM";
const URI: &[u8] = b"https://optimism.fit/app/warm-token.json";

fn data_of(encode: fn(&Face, &mut [u8; DATA_MAX]) -> usize, f: &Face) -> Vec<u8> {
    let mut out = [0u8; DATA_MAX];
    let n = encode(f, &mut out);
    out[..n].to_vec()
}

#[test]
fn init_with_a_face_asks_metaplex_for_it_the_mine_signing_twice() {
    let mut w = world();
    let polis = Pubkey::new_unique();
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    let cu = w.ok(&init_face(deployer, deployer, &polis, OPENING, &treasury, &dollar, &face_bytes(NAME, SYMBOL, URI)), "the deployer opens the mine with its face");
    let a = asked();
    assert_eq!(a.len(), 1, "one CPI to Metaplex");
    let (data, accounts) = &a[0];
    assert_eq!(*data, data_of(face::create_data, &Face { name: NAME, symbol: SYMBOL, uri: URI }), "CreateMetadataAccountV3, as the encoder writes it");
    let keys: Vec<[u8; 32]> = [metadata_pda(), mint_pda(), mine_pda(), deployer, mine_pda(), system_id()].iter().map(|k| k.to_bytes()).collect();
    let flags: Vec<(bool, bool)> = face::CREATE_ACCOUNTS.to_vec();
    assert_eq!(accounts.iter().map(|(k, _, _)| *k).collect::<Vec<_>>(), keys, "metadata, mint, the mine as mint authority, the payer, the mine as update authority, system");
    assert_eq!(accounts.iter().map(|(_, wr, s)| (*wr, *s)).collect::<Vec<_>>(), flags, "each account's write and signature as Metaplex's builder flags them");
    // the mine opened as without a face, and nothing minted
    let m = w.mine();
    assert_eq!((m.authority, m.treasury, m.opening_ms, m.minted, m.miner), (polis, treasury, OPENING, 0, Pubkey::default()));
    assert_eq!(mint_view(&w.data(&mint_pda())).authority, Some(mine_pda()), "the mine is the mint's authority, so it can sign for the face");
    eprintln!("  init with a face: {cu} CU, Metaplex's own units not counted (a stand-in)");
}

#[test]
fn init_without_a_face_asks_nothing_and_opens_the_mine_as_before() {
    let mut w = world();
    let polis = Pubkey::new_unique();
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    let cu = w.ok(&init(deployer, deployer, &polis, OPENING, &treasury, &dollar), "the deployer opens the mine with no face");
    assert!(asked().is_empty(), "no face, no CPI: a chain with no Metaplex opens it all the same");
    assert_eq!(w.mine().authority, polis);
    eprintln!("  init with no face: {cu} CU");
}

#[test]
fn a_face_out_of_metaplexs_bounds_or_its_accounts_wrong_opens_nothing() {
    let mut w = world();
    let polis = Pubkey::new_unique();
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    let good = face_bytes(NAME, SYMBOL, URI);
    for (bad, what) in [
        (face_bytes(&[b'n'; 33], SYMBOL, URI), "a name of 33 bytes"),
        (face_bytes(NAME, b"$WARMWARMWA", URI), "a symbol of 11 bytes"),
        (face_bytes(NAME, SYMBOL, &[b'u'; 201]), "a URI of 201 bytes"),
        (face_bytes(b"", SYMBOL, URI), "no name"),
        (face_bytes(NAME, b"", URI), "no symbol"),
        ([good.clone(), vec![0]].concat(), "a byte after the URI"),
        (good[..good.len() - 1].to_vec(), "a URI cut short"),
    ] {
        w.refuse(&init_face(deployer, deployer, &polis, OPENING, &treasury, &dollar, &bad), ProgramError::InvalidInstructionData, what);
    }
    // a face with no accounts for it, accounts with no face
    let mut short = init_face(deployer, deployer, &polis, OPENING, &treasury, &dollar, &good);
    short.accounts.pop();
    w.refuse(&short, ProgramError::NotEnoughAccountKeys, "a face with only one of its two accounts");
    let mut extra = init(deployer, deployer, &polis, OPENING, &treasury, &dollar);
    extra.accounts.extend([solana_instruction::AccountMeta::new(metadata_pda(), false), solana_instruction::AccountMeta::new_readonly(METAPLEX, false)]);
    w.refuse(&extra, ProgramError::NotEnoughAccountKeys, "the face's accounts with no face");
    // another program for Metaplex, another address for the metadata
    let mut other = init_face(deployer, deployer, &polis, OPENING, &treasury, &dollar, &good);
    other.accounts[11].pubkey = Pubkey::new_unique();
    w.refuse(&other, ProgramError::IncorrectProgramId, "a program that is not Metaplex's");
    let mut elsewhere = init_face(deployer, deployer, &polis, OPENING, &treasury, &dollar, &good);
    elsewhere.accounts[10].pubkey = Pubkey::new_unique();
    w.refuse(&elsewhere, WarmError::InvalidPda.into(), "a metadata account that is not the mint's own");
    assert!(asked().is_empty(), "nothing refused asked Metaplex anything");
    // and the stranger still opens nothing, face or none
    let stranger = w.wallet(SOL);
    w.refuse(&init_face(stranger, stranger, &polis, OPENING, &treasury, &dollar, &good), code(WarmError::Unauthorized), "a stranger's init with a face");
}

/// A world with the mine opened, its face standing as Metaplex keeps one.
fn faced() -> (World, Pubkey) {
    let mut w = world();
    let polis = Pubkey::new_unique();
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    w.ok(&init_face(deployer, deployer, &polis, OPENING, &treasury, &dollar, &face_bytes(NAME, SYMBOL, URI)), "the mine opened with its face");
    asked();
    // what Metaplex would have made of that ask: the stand-in made nothing, so it is laid here as Metaplex lays it
    w.put_owned(metadata_pda(), METAPLEX, metadata_bytes(&mine_pda(), &mint_pda(), NAME, SYMBOL, URI));
    (w, polis)
}

#[test]
fn set_face_by_the_authority_moves_the_uri_and_keeps_the_name() {
    let (mut w, polis) = faced();
    let moved = b"https://optimism.fit/app/warm-token-2.json";
    let cu = w.ok(&set_face(polis, moved), "the authority moves the face's URI");
    let a = asked();
    assert_eq!(a.len(), 1, "one CPI to Metaplex");
    let (data, accounts) = &a[0];
    // the name and the symbol read off the face Metaplex padded, without its padding
    assert_eq!(*data, data_of(face::update_data, &Face { name: NAME, symbol: SYMBOL, uri: moved }), "UpdateMetadataAccountV2: the URI moved, the name and symbol kept");
    assert_eq!(accounts.iter().map(|(k, _, _)| *k).collect::<Vec<_>>(), vec![metadata_pda().to_bytes(), mine_pda().to_bytes()]);
    assert_eq!(accounts.iter().map(|(_, wr, s)| (*wr, *s)).collect::<Vec<_>>(), face::UPDATE_ACCOUNTS.to_vec(), "the metadata written, the mine signing");
    eprintln!("  set_face: {cu} CU, Metaplex's own units not counted (a stand-in)");
}

#[test]
fn nobody_but_the_authority_moves_the_face_and_only_the_mines_own() {
    let (mut w, polis) = faced();
    let stranger = w.wallet(SOL);
    w.refuse(&set_face(stranger, URI), code(WarmError::Unauthorized), "a stranger moves no face");
    let mut unsigned = set_face(polis, URI);
    unsigned.accounts[0].is_signer = false;
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "the authority unsigned");
    w.refuse(&set_face(polis, &[b'u'; 201]), ProgramError::InvalidInstructionData, "a URI of 201 bytes");
    let mut other = set_face(polis, URI);
    other.accounts[3].pubkey = Pubkey::new_unique();
    w.refuse(&other, ProgramError::IncorrectProgramId, "a program that is not Metaplex's");
    // a face Metaplex does not hold, a face of another mint, a face the mine is not the update authority of
    let laid = w.account(&metadata_pda());
    for (account, what) in [
        (Account { owner: Pubkey::new_unique(), ..laid.clone() }, "a metadata account Metaplex does not own"),
        (Account { data: metadata_bytes(&mine_pda(), &Pubkey::new_unique(), NAME, SYMBOL, URI), ..laid.clone() }, "the face of another mint"),
        (Account { data: metadata_bytes(&Pubkey::new_unique(), &mint_pda(), NAME, SYMBOL, URI), ..laid.clone() }, "a face whose update authority is not the mine"),
        (Account { data: [&[5u8][..], &laid.data[1..]].concat(), ..laid.clone() }, "an account that is not a Metadata (its key)"),
    ] {
        w.put(metadata_pda(), account);
        w.refuse(&set_face(polis, URI), WarmError::InvalidAccount.into(), what);
    }
    w.put(metadata_pda(), laid);
    let mut elsewhere = set_face(polis, URI);
    elsewhere.accounts[2].pubkey = Pubkey::new_unique();
    w.put_owned(elsewhere.accounts[2].pubkey, METAPLEX, metadata_bytes(&mine_pda(), &mint_pda(), NAME, SYMBOL, URI));
    w.refuse(&elsewhere, WarmError::InvalidAccount.into(), "a Metadata at another address than the mint's own");
    assert!(asked().is_empty(), "nothing refused asked Metaplex anything");
}

#[test]
fn a_mine_with_no_face_has_none_to_move() {
    let mut w = world();
    let polis = Pubkey::new_unique();
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    w.ok(&init(deployer, deployer, &polis, OPENING, &treasury, &dollar), "the mine opened with no face");
    w.refuse(&set_face(polis, URI), WarmError::InvalidAccount.into(), "set_face moves a face, it makes none");
}

#[test]
fn the_encoders_write_borsh_as_metaplex_reads_it() {
    let f = Face { name: NAME, symbol: SYMBOL, uri: URI };
    let c = data_of(face::create_data, &f);
    let mut want = vec![33u8];
    for s in [NAME, SYMBOL, URI] {
        want.extend_from_slice(&(s.len() as u32).to_le_bytes());
        want.extend_from_slice(s);
    }
    want.extend_from_slice(&[0, 0, 0, 0, 0]);
    assert_eq!(c, [&want[..], &[1, 0]].concat(), "[33] DataV2 (no royalty, creators, collection, uses), mutable, no collection details");
    let u = data_of(face::update_data, &f);
    assert_eq!(u, [&[15u8, 1][..], &want[1..], &[0, 0, 0]].concat(), "[15] Some(DataV2), and no new authority, sale flag or mutability");
    // the longest face fits the buffer
    let long = Face { name: &[b'n'; 32], symbol: &[b's'; 10], uri: &[b'u'; 200] };
    assert_eq!(data_of(face::update_data, &long).len(), DATA_MAX);
    assert_eq!(data_of(face::create_data, &long).len(), DATA_MAX - 2);
    // what set_face reads back of a face Metaplex padded
    let d = metadata_bytes(&mine_pda(), &mint_pda(), NAME, SYMBOL, URI);
    let (name, symbol) = face::read_face(&d, &solana_address::Address::new_from_array(mine_pda().to_bytes()), &solana_address::Address::new_from_array(mint_pda().to_bytes())).unwrap();
    assert_eq!((name, symbol), (NAME, SYMBOL), "the padding is Metaplex's, never the face's");
}
