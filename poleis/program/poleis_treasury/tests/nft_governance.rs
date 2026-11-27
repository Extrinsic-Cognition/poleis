//! Current NFT floor rows and retired identities through the compiled SBF.
//! Fixtures are synthetic program-owned state; no network or signing keys.
mod built;
use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
use poleis_client::{assembly_floor_pda, sync_assembly_floor, PROGRAM_ID};
use poleis_treasury::{
    assembly::LEGACY_PROGRAM_ID,
    state::{Config, Registry},
    HOLDER_NFT, HOLDER_RETIRED_NFT,
};
use solana_account::Account;
use solana_pubkey::Pubkey;

fn put64(data: &mut [u8], offset: usize, value: u64) {
    data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn row(kind: u8, seats: u64, weight: u64, activity: u64) -> [u8; 57] {
    let mut data = [0; 57];
    data[..32].copy_from_slice(Pubkey::new_unique().as_ref());
    data[32] = kind;
    put64(&mut data, 33, seats);
    put64(&mut data, 41, weight);
    put64(&mut data, 49, activity);
    data
}

fn check_floor(layout: u8, rows: &[[u8; 57]], success: bool) {
    std::env::set_var(
        "SBF_OUT_DIR",
        built::program_dir(),
    );
    let svm = Mollusk::new(&PROGRAM_ID, "poleis_treasury");
    let program = if layout == 8 {
        PROGRAM_ID
    } else {
        Pubkey::new_from_array(LEGACY_PROGRAM_ID.to_bytes())
    };
    let payer = Pubkey::new_unique();
    let unit = 0u64;
    let (registry, registry_bump) =
        Pubkey::find_program_address(&[b"registry", &unit.to_le_bytes()], &program);
    let (config, config_bump) = if layout == 8 {
        Pubkey::find_program_address(&[b"config", &unit.to_le_bytes()], &program)
    } else {
        Pubkey::find_program_address(&[b"config"], &program)
    };
    let row_offset = Registry::OFF_HOLDERS - if layout == 8 { 0 } else { 8 };
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
    registry_data[Registry::OFF_CHAIR] = 255;
    registry_data[Registry::OFF_RUNG] = 5;
    registry_data[Registry::OFF_HOLDER_COUNT] = rows.len() as u8;
    registry_data[Registry::OFF_BUMP] = registry_bump;
    let total: u64 = rows
        .iter()
        .map(|r| u64::from_le_bytes(r[33..41].try_into().unwrap()))
        .sum();
    put64(&mut registry_data, Registry::OFF_TOTAL_SEATS, total);
    put64(&mut registry_data, Registry::OFF_WINDOW, 100);
    for (i, r) in rows.iter().enumerate() {
        registry_data[row_offset + i * 57..row_offset + (i + 1) * 57].copy_from_slice(r);
    }
    let mut config_data = vec![0; Config::SIZE - if layout == 8 { 0 } else { 8 }];
    config_data[0] = 1;
    config_data[1] = layout;
    config_data[Config::OFF_BUMP] = config_bump;
    config_data[Config::OFF_MODE] = poleis_treasury::MODE_COUNCIL;
    let floor = assembly_floor_pda(&program, unit);
    let accounts = vec![
        keyed_account_for_system_program(),
        (payer, Account::new(20_000_000_000, 0, &Pubkey::default())),
        (
            registry,
            Account {
                data: registry_data.clone(),
                ..Account::new(1_000_000_000, 0, &program)
            },
        ),
        (
            config,
            Account {
                data: config_data.clone(),
                ..Account::new(1_000_000_000, 0, &program)
            },
        ),
        (floor, Account::default()),
    ];
    let ix = sync_assembly_floor(payer, program, unit, registry, config);
    let result = svm.process_instruction(&ix, &accounts);
    assert_eq!(
        result.raw_result.is_ok(),
        success,
        "{:?}",
        result.raw_result
    );
    if success {
        let index = &result
            .resulting_accounts
            .iter()
            .find(|(key, _)| *key == floor)
            .unwrap()
            .1
            .data;
        assert_eq!(&index[104..], rows.concat());
        assert_eq!(index[3], layout);
        assert_eq!(&index[92..100], &total.to_le_bytes());
        assert_eq!(
            result
                .resulting_accounts
                .iter()
                .find(|(key, _)| *key == registry)
                .unwrap()
                .1
                .data,
            registry_data
        );
        assert_eq!(
            result
                .resulting_accounts
                .iter()
                .find(|(key, _)| *key == config)
                .unwrap()
                .1
                .data,
            config_data
        );
    }
}

#[test]
fn current_nft_and_retired_rows_index_without_reordering_or_source_mutation() {
    check_floor(
        8,
        &[row(HOLDER_NFT, 1, 3, 100), row(HOLDER_RETIRED_NFT, 0, 0, 0)],
        true,
    );
    check_floor(8, &[row(HOLDER_RETIRED_NFT, 0, 0, 0)], true);
}

#[test]
fn nft_kinds_are_never_accepted_from_legacy_layout() {
    check_floor(7, &[row(HOLDER_NFT, 1, 1, 100)], false);
    check_floor(7, &[row(HOLDER_RETIRED_NFT, 0, 0, 0)], false);
}

#[test]
fn live_nft_rows_require_one_seat_positive_weight_and_identity() {
    for invalid in [
        row(HOLDER_NFT, 0, 1, 100),
        row(HOLDER_NFT, 2, 1, 100),
        row(HOLDER_NFT, 1, 0, 100),
    ] {
        check_floor(8, &[invalid], false);
    }
    let mut zero_key = row(HOLDER_NFT, 1, 1, 100);
    zero_key[..32].fill(0);
    check_floor(8, &[zero_key], false);
}

#[test]
fn retired_rows_require_retained_identity_and_zero_economic_state() {
    for invalid in [
        row(HOLDER_RETIRED_NFT, 1, 0, 0),
        row(HOLDER_RETIRED_NFT, 0, 1, 0),
        row(HOLDER_RETIRED_NFT, 0, 0, 1),
    ] {
        check_floor(8, &[invalid], false);
    }
    let mut zero_key = row(HOLDER_RETIRED_NFT, 0, 0, 0);
    zero_key[..32].fill(0);
    check_floor(8, &[zero_key], false);
}

#[test]
fn electorate_extension_refuses_truncation_unknown_versions_flags_and_overflow() {
    use poleis_treasury::state::Session;
    let mut data =
        vec![0; Session::SIZE + Session::ELECTORATE_HEADER + 2 * Session::ELECTORATE_ROW];
    data[Session::SIZE..Session::SIZE + 8].copy_from_slice(&Session::ELECTORATE_MAGIC);
    data[Session::SIZE + 8] = 2;
    let a = Session::SIZE + Session::ELECTORATE_HEADER;
    let b = a + Session::ELECTORATE_ROW;
    put64(&mut data, a + 33, 7);
    put64(&mut data, b + 33, 3);
    data[a + 41] = 1;
    assert_eq!(Session::electorate_weights(&data).unwrap(), Some((10, 7)));
    for length in Session::SIZE + 1..data.len() {
        assert!(Session::electorate_weights(&data[..length]).is_err());
    }
    let mut bad = data.clone();
    bad[Session::SIZE + 7] = b'2';
    assert!(Session::electorate_weights(&bad).is_err());
    bad = data.clone();
    bad[Session::SIZE + 8] = 151;
    assert!(Session::electorate_weights(&bad).is_err());
    bad = data.clone();
    bad[b + 41] = 2;
    assert!(Session::electorate_weights(&bad).is_err());
    bad = data.clone();
    put64(&mut bad, a + 33, u64::MAX);
    assert!(Session::electorate_weights(&bad).is_err());
    bad = data;
    put64(&mut bad, a + 33, 0);
    assert!(
        Session::electorate_weights(&bad).is_err(),
        "zero-weight row cannot be admitted"
    );
}

#[test]
fn electorate_storage_is_bounded_without_changing_legacy_session_prefix() {
    use poleis_treasury::state::Session;
    let legacy = vec![0; Session::SIZE];
    assert_eq!(Session::SIZE, 158);
    assert_eq!(Session::electorate_weights(&legacy).unwrap(), None);
    let extra = Session::ELECTORATE_HEADER + Session::ELECTORATE_ROW * poleis_treasury::MAX_HOLDERS;
    assert_eq!(extra, 6309);
    let mut maximum = vec![0; Session::SIZE + extra];
    maximum[Session::SIZE..Session::SIZE + 8].copy_from_slice(&Session::ELECTORATE_MAGIC);
    maximum[Session::SIZE + 8] = poleis_treasury::MAX_HOLDERS as u8;
    assert_eq!(Session::electorate_weights(&maximum).unwrap(), Some((0, 0)));
    maximum.push(0);
    assert!(Session::electorate_weights(&maximum).is_err());
}
