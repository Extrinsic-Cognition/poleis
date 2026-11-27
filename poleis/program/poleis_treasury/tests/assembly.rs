//! The public floor index against the compiled program, including legacy input.
mod built;
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use poleis_client::{
    assembly_floor_pda, sync_assembly_floor, transfer_seats, AssemblyFloor, HOLDER_KEY,
    HOLDER_UNIT, MODE_COUNCIL, MODE_GENESIS, MODE_SOLO, PROGRAM_ID,
};
use poleis_treasury::{assembly::LEGACY_PROGRAM_ID, error::PoleisError, state::Registry};
use sha2::{Digest, Sha256};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

const SOL: u64 = 1_000_000_000;
fn put64(data: &mut [u8], offset: usize, value: u64) {
    data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn get64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}
fn row(holder: Pubkey, kind: u8, seats: u64) -> [u8; 57] {
    let mut data = [0; 57];
    data[..32].copy_from_slice(holder.as_ref());
    data[32] = kind;
    put64(&mut data, 33, seats);
    put64(&mut data, 41, 1);
    data
}

struct World {
    svm: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
    payer: Pubkey,
    program: Pubkey,
    unit: u64,
    registry: Pubkey,
    config: Pubkey,
    floor: Pubkey,
    row_offset: usize,
}
impl World {
    fn new(layout: u8, unit: u64, root: u64, rows: &[[u8; 57]]) -> Self {
        std::env::set_var(
            "SBF_OUT_DIR",
            built::program_dir(),
        );
        let mut svm = Mollusk::new(&PROGRAM_ID, "poleis_treasury");
        svm.sysvars.clock.slot = 1000;
        let program = if layout == 8 {
            PROGRAM_ID
        } else {
            Pubkey::new_from_array(LEGACY_PROGRAM_ID.to_bytes())
        };
        let payer = Pubkey::new_unique();
        let (registry, registry_bump) =
            Pubkey::find_program_address(&[b"registry", &unit.to_le_bytes()], &program);
        let (config, config_bump) = if layout == 8 {
            Pubkey::find_program_address(&[b"config", &root.to_le_bytes()], &program)
        } else {
            Pubkey::find_program_address(&[b"config"], &program)
        };
        let row_offset = if layout == 8 {
            Registry::OFF_HOLDERS
        } else {
            Registry::OFF_HOLDERS - 8
        };
        let mut registry_data = vec![
            0;
            row_offset
                + if layout == 8 {
                    rows.len() * 57
                } else {
                    150 * 57
                }
        ];
        registry_data[0] = 5;
        registry_data[1] = layout;
        put64(&mut registry_data, Registry::OFF_UNIT, unit);
        put64(&mut registry_data, Registry::OFF_PARENT, root);
        registry_data[Registry::OFF_CHAIR] = if unit == root { 255 } else { 2 };
        registry_data[Registry::OFF_LEVEL] = u8::from(unit != root);
        registry_data[Registry::OFF_RUNG] = if rows.is_empty() { 0 } else { 5 };
        registry_data[Registry::OFF_HOLDER_COUNT] = rows.len() as u8;
        registry_data[Registry::OFF_BUMP] = registry_bump;
        put64(&mut registry_data, Registry::OFF_WINDOW, 100);
        put64(
            &mut registry_data,
            Registry::OFF_TOTAL_SEATS,
            rows.iter().map(|r| get64(r, 33)).sum(),
        );
        if layout == 8 {
            put64(&mut registry_data, Registry::OFF_ROOT, root);
        }
        for (i, data) in rows.iter().enumerate() {
            registry_data[row_offset + i * 57..row_offset + (i + 1) * 57].copy_from_slice(data);
        }
        let mut config_data = vec![0; if layout == 8 { 76 } else { 68 }];
        config_data[0] = 1;
        config_data[1] = layout;
        config_data[34] = config_bump;
        config_data[35] = MODE_COUNCIL;
        if layout == 8 {
            put64(&mut config_data, 68, root);
        }
        let mut accounts = vec![
            keyed_account_for_system_program(),
            (payer, Account::new(20 * SOL, 0, &Pubkey::default())),
        ];
        accounts.push((
            registry,
            Account {
                data: registry_data,
                ..Account::new(SOL, 0, &program)
            },
        ));
        accounts.push((
            config,
            Account {
                data: config_data,
                ..Account::new(SOL, 0, &program)
            },
        ));
        let floor = assembly_floor_pda(&program, unit);
        Self {
            svm,
            accounts,
            payer,
            program,
            unit,
            registry,
            config,
            floor,
            row_offset,
        }
    }
    fn account(&self, key: Pubkey) -> &Account {
        &self.accounts.iter().find(|(k, _)| *k == key).unwrap().1
    }
    fn account_mut(&mut self, key: Pubkey) -> &mut Account {
        &mut self.accounts.iter_mut().find(|(k, _)| *k == key).unwrap().1
    }
    fn instruction(&self) -> Instruction {
        sync_assembly_floor(
            self.payer,
            self.program,
            self.unit,
            self.registry,
            self.config,
        )
    }
    fn run(&mut self, instruction: Instruction, error: Option<ProgramError>) -> u64 {
        for meta in &instruction.accounts {
            if !self.accounts.iter().any(|(key, _)| *key == meta.pubkey) {
                self.accounts.push((meta.pubkey, Account::default()));
            }
        }
        let check = if let Some(error) = error {
            Check::err(error)
        } else {
            Check::success()
        };
        let result =
            self.svm
                .process_and_validate_instruction(&instruction, &self.accounts, &[check]);
        if result.program_result.is_ok() {
            for (key, account) in result.resulting_accounts {
                *self.account_mut(key) = account;
            }
        }
        result.compute_units_consumed
    }
    fn sync(&mut self) -> u64 {
        self.run(self.instruction(), None)
    }
    fn refuse(&mut self, error: PoleisError) {
        self.run(self.instruction(), Some(ProgramError::Custom(error as u32)));
    }
}

#[test]
fn current_and_legacy_sources_index_exact_rows_and_keep_source_accounts_unchanged() {
    for layout in [7, 8] {
        let mut unit_holder = [0; 32];
        unit_holder[..8].copy_from_slice(&12u64.to_le_bytes());
        let rows = [
            row(Pubkey::new_unique(), HOLDER_KEY, 3),
            row(Pubkey::new_from_array(unit_holder), HOLDER_UNIT, 2),
        ];
        let mut w = World::new(layout, 41, if layout == 8 { 9 } else { 0 }, &rows);
        let registry_before = w.account(w.registry).clone();
        let config_before = w.account(w.config).clone();
        let ix = w.instruction();
        assert_eq!(ix.data.len(), 41);
        assert_eq!(ix.data[0], 37);
        assert!(!ix.accounts[1].is_writable && !ix.accounts[2].is_writable);
        w.sync();
        let indexed = &w.account(w.floor).data;
        assert_eq!(indexed.len(), 104 + 2 * 57);
        assert_eq!(&indexed[..2], &[16, 8]);
        assert_eq!(indexed[3], layout);
        assert_eq!(&indexed[4..36], w.program.as_ref());
        assert_eq!(get64(indexed, 36), 41);
        assert_eq!(get64(indexed, 44), if layout == 8 { 9 } else { 0 });
        assert_eq!(get64(indexed, 84), 1000);
        assert_eq!(get64(indexed, 92), 5);
        assert_eq!(&indexed[104..], rows.concat());
        assert_eq!(
            &indexed[52..84],
            Sha256::digest(&registry_before.data).as_slice()
        );
        assert_eq!(*w.account(w.registry), registry_before);
        assert_eq!(*w.account(w.config), config_before);
        assert_eq!(w.account(w.floor).owner, PROGRAM_ID);
    }
    assert_ne!(
        assembly_floor_pda(&PROGRAM_ID, 41),
        assembly_floor_pda(&Pubkey::new_from_array(LEGACY_PROGRAM_ID.to_bytes()), 41)
    );
}

#[test]
fn prefunded_creation_noop_and_config_only_change_are_safe() {
    let mut w = World::new(8, 0, 0, &[row(Pubkey::new_unique(), HOLDER_KEY, 1)]);
    w.accounts.push((
        w.floor,
        Account {
            lamports: 777,
            ..Account::default()
        },
    ));
    w.sync();
    let original = w.account(w.floor).clone();
    let payer = w.account(w.payer).clone();
    w.svm.sysvars.clock.slot = 1200;
    w.sync();
    assert_eq!(
        *w.account(w.floor),
        original,
        "unchanged facts must not rewrite the observation slot"
    );
    assert_eq!(
        *w.account(w.payer),
        payer,
        "a no-op must not require more rent"
    );
    w.account_mut(w.config).data[35] = MODE_GENESIS;
    w.sync();
    assert_eq!(w.account(w.floor).data[102], MODE_GENESIS);
    assert_eq!(get64(&w.account(w.floor).data, 84), 1200);
    assert_eq!(&w.account(w.floor).data[52..84], &original.data[52..84]);
}

#[test]
fn zero_floor_genesis_and_dissolved_facts_are_indexed_without_inventing_seats() {
    for layout in [7, 8] {
        for mode in [MODE_SOLO, MODE_GENESIS, MODE_COUNCIL] {
            let mut w = World::new(layout, 0, 0, &[]);
            w.account_mut(w.config).data[35] = mode;
            if mode == MODE_GENESIS {
                w.account_mut(w.registry).data[52] = 1;
            }
            w.sync();
            assert_eq!(w.account(w.floor).data.len(), 104);
            assert_eq!(get64(&w.account(w.floor).data, 92), 0);
            assert_eq!(w.account(w.floor).data[102], mode);
        }
        let mut w = World::new(layout, 0, 0, &[row(Pubkey::new_unique(), HOLDER_KEY, 2)]);
        w.account_mut(w.registry).data[53] = 1;
        w.sync();
        assert_eq!(w.account(w.floor).data[101], 1);
        assert_eq!(get64(&w.account(w.floor).data, 92), 2);
    }
}

#[test]
fn a_leaver_empties_its_row_in_place_and_the_index_follows_without_changing_the_source() {
    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();
    let carol = Pubkey::new_unique();
    let mut w = World::new(
        8,
        0,
        0,
        &[
            row(alice, HOLDER_KEY, 2),
            row(bob, HOLDER_KEY, 3),
            row(carol, HOLDER_KEY, 1),
        ],
    );
    w.accounts
        .push((alice, Account::new(SOL, 0, &Pubkey::default())));
    w.sync();
    let index_before = w.account(w.floor).clone();
    w.run(
        transfer_seats(alice, 0, bob.to_bytes(), HOLDER_KEY, 2),
        None,
    );
    let source = w.account(w.registry).clone();
    // rows never move: alice's row is a tombstone in place, bob and carol keep their indexes
    assert_eq!(source.data[126], 3);
    assert_eq!(&source.data[w.row_offset..w.row_offset + 32], &[0u8; 32]);
    assert_eq!(source.data[w.row_offset + 32], 0, "an emptied row has no holder kind");
    assert_eq!(&source.data[w.row_offset + 57..w.row_offset + 57 + 32], bob.as_ref());
    assert_eq!(&source.data[w.row_offset + 2 * 57..w.row_offset + 2 * 57 + 32], carol.as_ref());
    w.svm.sysvars.clock.slot += 1;
    w.sync();
    let indexed = w.account(w.floor);
    assert_eq!(indexed.data.len(), 104 + 3 * 57);
    assert_eq!(indexed.lamports, index_before.lamports);
    assert_eq!(get64(&indexed.data, 92), 6, "the tombstone adds no seats to the floor");
    assert_eq!(
        &indexed.data[104..],
        &source.data[w.row_offset..w.row_offset + 3 * 57]
    );
    assert_ne!(&indexed.data[52..84], &index_before.data[52..84]);
    assert_eq!(*w.account(w.registry), source);
}

#[test]
fn growth_funds_only_needed_rent_and_maximum_floor_remains_bounded() {
    let mut w = World::new(8, 0, 0, &[]);
    w.sync();
    let before = w.account(w.floor).lamports;
    let rows: Vec<_> = (0..150)
        .map(|_| row(Pubkey::new_unique(), HOLDER_KEY, 1))
        .collect();
    let source = w.account_mut(w.registry);
    source.data.resize(Registry::MAX_SIZE, 0);
    source.data[126] = 150;
    source.data[52] = 5;
    put64(&mut source.data, 54, 150);
    source.data[Registry::OFF_HOLDERS..].copy_from_slice(&rows.concat());
    let payer_before = w.account(w.payer).lamports;
    let units = w.sync();
    assert_eq!(w.account(w.floor).data.len(), AssemblyFloor::MAX_SIZE);
    let growth = w.account(w.floor).lamports - before;
    assert!(growth > 0);
    assert_eq!(payer_before - w.account(w.payer).lamports, growth);
    assert!(
        units < 200_000,
        "maximum source must fit an ordinary instruction budget: {units}"
    );
    eprintln!("maximum floor sync: {units} compute units");
}

#[test]
fn rejects_foreign_owner_program_version_shape_and_noncanonical_source_addresses() {
    for variant in 0..10 {
        let mut w = World::new(8, 0, 0, &[row(Pubkey::new_unique(), HOLDER_KEY, 1)]);
        let mut ix = w.instruction();
        let error = match variant {
            0 => {
                w.account_mut(w.registry).owner = Pubkey::new_unique();
                PoleisError::InvalidAccount
            }
            1 => {
                w.account_mut(w.config).owner = Pubkey::new_unique();
                PoleisError::InvalidAccount
            }
            2 => {
                ix.data[1..33].copy_from_slice(Pubkey::new_unique().as_ref());
                PoleisError::InvalidAccount
            }
            3 => {
                w.account_mut(w.registry).data[1] = 7;
                PoleisError::LayoutVersion
            }
            4 => {
                w.account_mut(w.config).data[1] = 7;
                PoleisError::LayoutVersion
            }
            5 => {
                w.account_mut(w.registry).data.push(0);
                PoleisError::InvalidAccount
            }
            6 => {
                w.account_mut(w.config).data.push(0);
                PoleisError::InvalidAccount
            }
            7 => {
                let clone = Pubkey::new_unique();
                w.accounts.push((clone, w.account(w.registry).clone()));
                ix.accounts[1].pubkey = clone;
                PoleisError::InvalidPda
            }
            8 => {
                let clone = Pubkey::new_unique();
                w.accounts.push((clone, w.account(w.config).clone()));
                ix.accounts[2].pubkey = clone;
                PoleisError::InvalidPda
            }
            _ => {
                w.account_mut(w.registry).data[127] ^= 1;
                PoleisError::InvalidPda
            }
        };
        w.run(ix, Some(ProgramError::Custom(error as u32)));
        assert!(w.account(w.floor).data.is_empty());
    }
}

#[test]
fn rejects_wrong_config_root_unit_mode_and_source_tree_header() {
    for variant in 0..6 {
        let mut w = World::new(8, 11, 9, &[row(Pubkey::new_unique(), HOLDER_KEY, 1)]);
        let error = match variant {
            0 => {
                put64(&mut w.account_mut(w.config).data, 68, 10);
                PoleisError::UnitMismatch
            }
            1 => {
                put64(&mut w.account_mut(w.registry).data, 2, 12);
                PoleisError::UnitMismatch
            }
            2 => {
                w.account_mut(w.config).data[35] = 3;
                PoleisError::InvalidMode
            }
            3 => {
                w.account_mut(w.registry).data[19] = 0;
                PoleisError::NotDescendant
            }
            4 => {
                w.account_mut(w.registry).data[19] = 6;
                PoleisError::NotDescendant
            }
            _ => {
                w.account_mut(w.registry).data[18] = 12;
                PoleisError::NotDescendant
            }
        };
        w.refuse(error);
    }
}

#[test]
fn rejects_inconsistent_or_phantom_holdings_and_malformed_index_accounts() {
    for variant in 0..10 {
        let mut w = World::new(8, 0, 0, &[row(Pubkey::new_unique(), HOLDER_KEY, 1)]);
        let off = w.row_offset;
        let error = match variant {
            0 => {
                w.account_mut(w.registry).data[126] = 2;
                PoleisError::RegistryFull
            }
            1 => {
                w.account_mut(w.registry).data[52] = 6;
                PoleisError::NotOnLadder
            }
            2 => {
                put64(&mut w.account_mut(w.registry).data, 54, 2);
                PoleisError::SeatLimit
            }
            3 => {
                w.account_mut(w.registry).data[52] = 0;
                PoleisError::SeatLimit
            }
            4 => {
                w.account_mut(w.registry).data[off + 32] = 0;
                PoleisError::InvalidHolder
            }
            5 => {
                put64(&mut w.account_mut(w.registry).data, off + 33, 0);
                PoleisError::SeatLimit
            }
            6 => {
                put64(&mut w.account_mut(w.registry).data, off + 41, 0);
                PoleisError::SeatLimit
            }
            7 => {
                w.account_mut(w.registry).data[53] = 2;
                PoleisError::InvalidAccount
            }
            8 => {
                w.account_mut(w.registry).data[off + 32] = 5; // 3/4 are live/retired NFTs.
                PoleisError::InvalidHolder
            }
            _ => {
                w.accounts
                    .push((w.floor, Account::new(SOL, 104, &Pubkey::new_unique())));
                PoleisError::InvalidAccount
            }
        };
        w.refuse(error);
    }
    let same = row(Pubkey::new_unique(), HOLDER_KEY, 1);
    let mut w = World::new(8, 0, 0, &[same, same]);
    w.refuse(PoleisError::InvalidHolder);
}

#[test]
fn instruction_authority_and_shape_checks_precede_any_index_write() {
    let mut w = World::new(8, 0, 0, &[]);
    let mut ix = w.instruction();
    ix.accounts[0].is_signer = false;
    w.run(ix, Some(ProgramError::MissingRequiredSignature));
    let mut ix = w.instruction();
    ix.accounts[3].is_writable = false;
    w.run(
        ix,
        Some(ProgramError::Custom(PoleisError::InvalidAccount as u32)),
    );
    let mut ix = w.instruction();
    ix.data.push(0);
    w.run(ix, Some(ProgramError::InvalidInstructionData));
    let mut ix = w.instruction();
    ix.accounts[3].pubkey = Pubkey::new_unique();
    w.run(
        ix,
        Some(ProgramError::Custom(PoleisError::InvalidPda as u32)),
    );
    assert!(w.account(w.floor).data.is_empty());
}

#[test]
fn every_legal_depth_and_a_unit_zero_holder_keep_their_source_identity() {
    for depth in 0..=5 {
        let unit = if depth == 0 { 42 } else { 42 + depth as u64 };
        let mut w = World::new(8, unit, 42, &[row(Pubkey::default(), HOLDER_UNIT, 1)]);
        w.account_mut(w.registry).data[19] = depth;
        if depth > 0 {
            put64(&mut w.account_mut(w.registry).data, 10, unit - 1);
        }
        w.sync();
        assert_eq!(get64(&w.account(w.floor).data, 36), unit);
        assert_eq!(get64(&w.account(w.floor).data, 44), 42);
        assert_eq!(&w.account(w.floor).data[104..136], &[0; 32]);
        assert_eq!(w.account(w.floor).data[136], HOLDER_UNIT);
    }
}

#[test]
fn legacy_shape_is_fixed_and_the_hash_includes_unused_source_allocation() {
    let mut w = World::new(7, 0, 0, &[row(Pubkey::new_unique(), HOLDER_KEY, 1)]);
    w.sync();
    let first = w.account(w.floor).data.clone();
    w.svm.sysvars.clock.slot += 1;
    *w.account_mut(w.registry).data.last_mut().unwrap() ^= 1;
    w.sync();
    assert_ne!(&w.account(w.floor).data[52..84], &first[52..84]);
    assert_eq!(&w.account(w.floor).data[104..], &first[104..]);
    let mut truncated = World::new(7, 0, 0, &[]);
    truncated
        .account_mut(truncated.registry)
        .data
        .truncate(1048);
    truncated.refuse(PoleisError::InvalidAccount);
    let mut bad_count = World::new(7, 0, 0, &[]);
    bad_count.account_mut(bad_count.registry).data[126] = 151;
    bad_count.refuse(PoleisError::RegistryFull);
    let mut wrong_config = World::new(7, 0, 0, &[]);
    let with_root_seed =
        Pubkey::find_program_address(&[b"config", &0u64.to_le_bytes()], &wrong_config.program).0;
    wrong_config.accounts.push((
        with_root_seed,
        wrong_config.account(wrong_config.config).clone(),
    ));
    let mut ix = wrong_config.instruction();
    ix.accounts[2].pubkey = with_root_seed;
    wrong_config.run(
        ix,
        Some(ProgramError::Custom(PoleisError::InvalidPda as u32)),
    );
}

#[test]
fn raw_unit_holder_identity_created_by_existing_transfer_is_preserved() {
    let alice = Pubkey::new_unique();
    let mut w = World::new(8, 0, 0, &[row(alice, HOLDER_KEY, 1)]);
    w.accounts
        .push((alice, Account::new(SOL, 0, &Pubkey::default())));
    let mut recipient = [7u8; 32];
    recipient[..8].copy_from_slice(&42u64.to_le_bytes());
    w.run(transfer_seats(alice, 0, recipient, HOLDER_UNIT, 1), None);
    w.sync();
    assert_eq!(&w.account(w.floor).data[104..136], &recipient);
    assert_eq!(w.account(w.floor).data[136], HOLDER_UNIT);
    assert_eq!(get64(&w.account(w.floor).data, 92), 1);
}
