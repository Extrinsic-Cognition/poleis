//! THE WIRE IS SOLANA'S OWN: every instruction of Solana's programs the press writes by hand (the upgradeable loader's, the system
//! program's, the compute budget's) held, byte for byte and account by account, to the interface crates those programs' clients build them
//! with (solana-loader-v3-interface 7.0.0, solana-system-interface 3), the loader's account states read as its interface writes them, the
//! `with_seed` address held to Solana's own derivation, and the legacy message held to Solana's own known transaction.
use poleis_press::loader::*;
use poleis_press::wire::{self, Key};
use solana_instruction::Instruction;
use solana_loader_v3_interface::{instruction as l3, state::UpgradeableLoaderState};
use solana_pubkey::Pubkey;
use solana_system_interface::instruction as sys;

fn k(b: u8) -> Pubkey {
    Pubkey::new_from_array([b; 32])
}
fn same(ours: &Instruction, theirs: &Instruction, what: &str) {
    assert_eq!(ours.program_id, theirs.program_id, "{what}: the program");
    assert_eq!(ours.data, theirs.data, "{what}: the data");
    assert_eq!(ours.accounts, theirs.accounts, "{what}: the accounts, their writes and signatures");
}

/// THE PRESS IS READY FOR MAINNET: the loader's ExtendProgram, written by hand, is its interface's own, the crank its payer.
#[test]
fn the_extend_is_the_loaders_own_extend_program() {
    for (program, payer, n) in [(k(3), k(1), 10_240u32), (k(7), k(2), 98_744), (poleis_client::PROGRAM_ID, k(9), 437_656)] {
        same(&extend_program(&program, &payer, n), &l3::extend_program(&program, Some(&payer), n), "ExtendProgram");
    }
    assert_eq!(poleis_press::upgrade::MIN_EXTEND, l3::MINIMUM_EXTEND_PROGRAM_BYTES, "the loader's least extension");
}

/// THE SEAL NAMES ITS PROGRAM: after the handoff the three programs answer to the one upgrade PDA, and the act that moves an authority
/// names its program. What the client hands the act's carrier to write is that program's data by Solana's own derivation, and nothing else
/// of the loader's: the address Poleis derives for the name and refuses every other by.
#[test]
fn the_act_that_moves_a_handed_programs_authority_reaches_solanas_own_program_data_of_it() {
    use poleis_client::{execute, proposal_pda, Payload, BPF_LOADER_UPGRADEABLE_ID, ROOT_UNIT};
    assert_eq!(LOADER_V3, BPF_LOADER_UPGRADEABLE_ID, "the press's loader is the client's");
    let (crank, hash) = (k(1), [9u8; 32]);
    for program in [k(3), k(7), poleis_client::PROGRAM_ID] {
        let theirs = solana_loader_v3_interface::get_program_data_address(&program);
        assert_eq!((programdata_of(&program), poleis_client::programdata_of(&program)), (theirs, theirs), "the program data of {program}");
        for new in [Some(k(5)), None] {
            let act = Payload::UpgradeAuthority { origin: ROOT_UNIT, program, new };
            let carry = execute(crank, hash, &[ROOT_UNIT], &act);
            let written: Vec<Pubkey> = carry.accounts.iter().filter(|m| m.is_writable).map(|m| m.pubkey).collect();
            assert_eq!(written, vec![crank, proposal_pda(&hash), theirs], "the carrier writes its own fee, the proposal and {program}'s program data alone");
            assert_eq!(&act.bytes()[34..66], program.as_ref(), "the act's bytes end in its program");
        }
    }
}

/// THE SEAL UPGRADES EVERY PROGRAM: the act that replaces a handed program's code hands Poleis, for its call into the loader, the accounts the
/// loader's own `Upgrade` names, in its order (the program's data, the program, the buffer, the spill, the rent, the clock, the authority), then
/// the loader. The authority is the seal's PDA, which signs inside Poleis and never on the transaction that carries the act out.
#[test]
fn the_upgrade_of_a_handed_program_carries_the_loaders_own_upgrade_accounts() {
    use poleis_client::{execute, upgrade_pda, Payload, BPF_LOADER_UPGRADEABLE_ID, ROOT_UNIT};
    let (crank, hash, buffer, spill) = (k(1), [9u8; 32], k(2), k(4));
    for program in [k(3), k(7), poleis_client::PROGRAM_ID] {
        let carry = execute(crank, hash, &[ROOT_UNIT], &Payload::UpgradeProgram { origin: ROOT_UNIT, program, buffer, spill });
        let theirs = l3::upgrade(&program, &buffer, &upgrade_pda(), &spill);
        let (len, n) = (carry.accounts.len(), theirs.accounts.len());
        let tail = &carry.accounts[len - n - 1..len - 1];
        assert_eq!(
            tail.iter().map(|m| (m.pubkey, m.is_writable)).collect::<Vec<_>>(),
            theirs.accounts.iter().map(|m| (m.pubkey, m.is_writable)).collect::<Vec<_>>(),
            "{program}: the loader's own Upgrade accounts, in its order"
        );
        assert!(theirs.accounts[n - 1].is_signer && !tail[n - 1].is_signer, "{program}: the authority signs in the loader's call, by the seal's PDA, never outside");
        assert_eq!(carry.accounts[len - 1].pubkey, BPF_LOADER_UPGRADEABLE_ID, "{program}: the loader last");
    }
}

#[test]
fn the_loaders_instructions_are_its_interfaces_own() {
    let (payer, buffer, program, authority, seal) = (k(1), k(2), k(3), k(4), k(5));
    assert_eq!(LOADER_V3, l3::write(&buffer, &authority, 0, vec![]).program_id, "the upgradeable loader");
    let theirs = l3::create_buffer(&payer, &buffer, &authority, 777, 101_968).unwrap();
    same(&create_account(&payer, &buffer, 777, (BUFFER_HEADER + 101_968) as u64, &LOADER_V3), &theirs[0], "the buffer's account");
    same(&initialize_buffer(&buffer, &authority), &theirs[1], "InitializeBuffer");
    for (offset, bytes) in [(0u32, vec![1u8, 2, 3]), (123_456, vec![9; 1_012]), (u32::MAX, vec![])] {
        same(&write(&buffer, &authority, offset, &bytes), &l3::write(&buffer, &authority, offset, bytes.clone()), "Write");
    }
    let theirs = l3::deploy_with_max_program_len(&payer, &program, &buffer, &authority, 999, 456_800).unwrap();
    same(&create_account(&payer, &program, 999, PROGRAM_LEN as u64, &LOADER_V3), &theirs[0], "the program's account");
    same(&deploy_with_max_data_len(&payer, &program, &buffer, &authority, 456_800), &theirs[1], "DeployWithMaxDataLen");
    assert_eq!(programdata_of(&program), solana_loader_v3_interface::get_program_data_address(&program));
    same(&set_authority(&programdata_of(&program), &authority, Some(&seal)), &l3::set_upgrade_authority(&program, &authority, Some(&seal)), "SetAuthority, to a PDA that does not sign");
    same(&set_authority(&programdata_of(&program), &authority, None), &l3::set_upgrade_authority(&program, &authority, None), "SetAuthority, to none");
    same(&close_buffer(&buffer, &payer, &authority), &l3::close(&buffer, &payer, &authority), "Close, a buffer");
    assert_eq!((BUFFER_HEADER, PROGRAM_LEN, PROGRAMDATA_HEADER), (UpgradeableLoaderState::size_of_buffer(0), UpgradeableLoaderState::size_of_program(), UpgradeableLoaderState::size_of_programdata(0)));
    assert_eq!((RENT_SYSVAR, CLOCK_SYSVAR), (theirs[1].accounts[4].pubkey, theirs[1].accounts[5].pubkey), "the rent and the clock");
}

#[test]
fn the_loaders_states_read_as_its_interface_writes_them() {
    for (state, want) in [
        (UpgradeableLoaderState::Buffer { authority_address: Some(k(7)) }, State::Buffer { authority: Some(k(7)) }),
        (UpgradeableLoaderState::Buffer { authority_address: None }, State::Buffer { authority: None }),
        (UpgradeableLoaderState::Program { programdata_address: k(8) }, State::Program { programdata: k(8) }),
        (UpgradeableLoaderState::ProgramData { slot: 616, upgrade_authority_address: Some(k(9)) }, State::ProgramData { slot: 616, authority: Some(k(9)) }),
        (UpgradeableLoaderState::ProgramData { slot: u64::MAX, upgrade_authority_address: None }, State::ProgramData { slot: u64::MAX, authority: None }),
    ] {
        let bytes = bincode::serialize(&state).unwrap();
        assert_eq!(poleis_press::loader::state(&bytes).unwrap(), want, "{state:?}");
    }
    assert!(poleis_press::loader::state(&[0, 0, 0, 0]).is_err(), "an uninitialized account is no state the press reads");
}

#[test]
fn the_system_programs_instructions_are_its_interfaces_own() {
    let (from, to, base, owner) = (k(1), k(2), k(3), LOADER_V3);
    same(&create_account(&from, &to, 1_461_600, 82, &owner), &sys::create_account(&from, &to, 1_461_600, 82, &owner), "CreateAccount");
    let seed = "launch-press:poleis:0";
    same(&create_account_with_seed(&from, &to, &base, seed, 5, 37, &owner), &sys::create_account_with_seed(&from, &to, &base, seed, 5, 37, &owner), "CreateAccountWithSeed");
    same(&transfer(&from, &to, 42), &sys::transfer(&from, &to, 42), "Transfer");
    same(&allocate(&to, 36), &sys::allocate(&to, 36), "Allocate");
    same(&assign(&to, &owner), &sys::assign(&to, &owner), "Assign");
    // the named buffer's address: Solana's own derivation, and refused past the seed's 32 bytes
    for seed in ["launch-press:poleis:0", "launch-press:market:7", ""] {
        assert_eq!(address_with_seed(&base, seed, &owner).unwrap(), Pubkey::create_with_seed(&base, seed, &owner).unwrap(), "{seed:?}");
    }
    assert!(address_with_seed(&base, &"x".repeat(33), &owner).is_err());
}

#[test]
fn the_compute_budgets_two_instructions_are_its_own() {
    // ComputeBudgetInstruction (Borsh, one byte a variant): SetComputeUnitLimit is 2, a u32; SetComputeUnitPrice is 3, a u64
    assert_eq!(compute_limit(200_000).data, [&[2u8][..], &200_000u32.to_le_bytes()].concat());
    assert_eq!(compute_price(10_000).data, [&[3u8][..], &10_000u64.to_le_bytes()].concat());
    assert!(compute_limit(1).accounts.is_empty() && compute_price(1).accounts.is_empty());
    assert_eq!(COMPUTE_BUDGET.to_string(), "ComputeBudget111111111111111111111111111111");
}

/// Solana's own known legacy transaction, `test_sdk_serialize` in solana-transaction 4.1.5: one instruction of (1, 2, 3) to a program,
/// its payer a signer and one account written, the default blockhash (held here as THE TWO PROGRAMS ON A LIVE VALIDATOR holds it).
const SOLANAS_OWN: [u8; 206] = [
    1, 120, 138, 162, 185, 59, 209, 241, 157, 71, 157, 74, 131, 4, 87, 54, 28, 38, 180, 222, 82, 64, 62, 61, 62, 22, 46, 17, 203, 187, 136, 62, 43, 11, 38,
    235, 17, 239, 82, 240, 139, 130, 217, 227, 214, 9, 242, 141, 223, 94, 29, 184, 110, 62, 32, 87, 137, 63, 139, 100, 221, 20, 137, 4, 5, 1, 0, 1, 3, 36,
    100, 158, 252, 33, 161, 97, 185, 62, 89, 99, 195, 250, 249, 187, 189, 171, 118, 241, 90, 248, 14, 68, 219, 231, 62, 157, 5, 142, 27, 210, 117, 1, 1, 1,
    4, 5, 6, 7, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 7, 6, 5, 4, 1, 1, 1, 2, 2, 2, 4, 5, 6, 7, 8, 9, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 9, 8, 7, 6, 5, 4, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 0, 1, 3, 1, 2,
    3,
];

#[test]
fn the_message_and_the_transaction_are_solanas_own() {
    let payer = Pubkey::new_from_array(SOLANAS_OWN[69..101].try_into().unwrap());
    let to = Pubkey::new_from_array([1, 1, 1, 4, 5, 6, 7, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 7, 6, 5, 4, 1, 1, 1]);
    let program = Pubkey::new_from_array([2, 2, 2, 4, 5, 6, 7, 8, 9, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9, 8, 7, 6, 5, 4, 2, 2, 2]);
    let ix = Instruction::new_with_bytes(program, &[1, 2, 3], vec![solana_instruction::AccountMeta::new(payer, true), solana_instruction::AccountMeta::new(to, false)]);
    let (m, signers) = wire::message(&[ix], &payer, &[0; 32]);
    assert_eq!((signers, &m[..]), (vec![payer], &SOLANAS_OWN[65..]), "the message, byte for byte");
    assert_eq!(wire::transaction(&m, &[SOLANAS_OWN[1..65].try_into().unwrap()]), SOLANAS_OWN.to_vec(), "the transaction, byte for byte");
    let (n, keys, blockhash_at) = wire::layout(&m).unwrap();
    assert_eq!((n, keys, blockhash_at), (1, vec![payer, to, program], 3 + 1 + 3 * 32), "the header, the keys, and where the blockhash sits");
}

#[test]
fn base58_and_a_keys_file_are_solanas_own() {
    for b in [[0u8; 32], [255; 32], { let mut x = [0u8; 32]; x[31] = 1; x }, { let mut x = [7u8; 32]; x[..3].fill(0); x }] {
        let s = wire::base58(&b);
        assert_eq!(s, Pubkey::new_from_array(b).to_string());
        assert_eq!(wire::unbase58::<32>(&s).unwrap(), b);
    }
    let sig = [42u8; 64];
    assert_eq!(wire::unbase58::<64>(&wire::base58(&sig)).unwrap(), sig);
    assert!(wire::unbase58::<32>("0OIl").is_err());
    // the CLI's file shape: 64 bytes, the seed then the address, the address held to the seed's
    let k = Key::from_seed(&[7; 32]);
    let back = Key::from_cli_text(&k.cli_text()).unwrap();
    assert_eq!(back.address, k.address);
    let mut v: Vec<u8> = serde_json::from_str(&k.cli_text()).unwrap();
    v[63] ^= 1;
    assert!(Key::from_cli_text(&serde_json::to_string(&v).unwrap()).is_err(), "a file whose address is not its seed's");
    // a rehearsal's stand-ins: one seed, one key a name, and another seed none of them
    let (a, b) = (wire::standin(&[1; 32], "deployer"), wire::standin(&[1; 32], "poleis"));
    assert_ne!(a.address, b.address);
    assert_eq!(a.address, wire::standin(&[1; 32], "deployer").address);
    assert_ne!(a.address, wire::standin(&[2; 32], "deployer").address);
}

#[test]
fn the_rpc_talks_to_mainnet_by_https_and_to_this_machine_only_otherwise() {
    use poleis_press::rpc::{check_url, Cluster};
    for ok in ["http://127.0.0.1:8899", "http://127.0.0.1:18899/", "http://[::1]:8899"] {
        assert!(check_url(ok, Cluster::Localnet).is_ok(), "{ok}");
    }
    for bad in [
        "https://api.devnet.solana.com", "http://localhost:8899", "http://10.0.0.5:8899", "http://127.0.0.2:8899", "http://0.0.0.0:8899", "http://127.0.0.1",
        "http://127.0.0.1:0", "http://127.0.0.1:99999", "http://127.0.0.1:08899", "http://127.0.0.1:8899/rpc", "http://user@127.0.0.1:8899", "http://127.0.0.1:8899 x",
        "https://127.0.0.1:8899", "ws://127.0.0.1:8900",
    ] {
        assert!(check_url(bad, Cluster::Localnet).is_err(), "{bad}");
    }
    assert!(check_url("https://api.mainnet-beta.solana.com", Cluster::Mainnet).is_ok());
    for bad in ["http://api.mainnet-beta.solana.com", "http://127.0.0.1:8899", "https://", "https://a@b.com"] {
        assert!(check_url(bad, Cluster::Mainnet).is_err(), "{bad}");
    }
    assert!(Cluster::parse("devnet").is_err(), "devnet is no step of his");
}
