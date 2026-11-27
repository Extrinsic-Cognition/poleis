//! THE FACE IS METAPLEX'S OWN: the $WARM program's two Metaplex instructions, written by hand in the program (warm_mine/src/face.rs), held
//! byte for byte and account by account to Metaplex's own SDK (mpl-token-metadata 5.1.1, its generated builders), and the Metadata layout
//! `set_face` reads held to Metaplex's own account struct. Metaplex's program is on no chain of the house (the rehearsal's validator has
//! none, and none is fetched), so this is where the face's CPI meets Metaplex: warm_mine/tests/face.rs holds what the program sends to
//! these encoders, and this holds the encoders to Metaplex.
use borsh::BorshSerialize;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs, UpdateMetadataAccountV2, UpdateMetadataAccountV2InstructionArgs};
use mpl_token_metadata::types::{DataV2, Key};
use solana_program::pubkey::Pubkey as Theirs;
use warm_mine::face::{self, Face, DATA_MAX};

fn k(b: u8) -> Theirs {
    Theirs::new_from_array([b; 32])
}
fn data_v2(f: &Face) -> DataV2 {
    DataV2 {
        name: String::from_utf8(f.name.to_vec()).unwrap(),
        symbol: String::from_utf8(f.symbol.to_vec()).unwrap(),
        uri: String::from_utf8(f.uri.to_vec()).unwrap(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    }
}
fn ours(encode: fn(&Face, &mut [u8; DATA_MAX]) -> usize, f: &Face) -> Vec<u8> {
    let mut out = [0u8; DATA_MAX];
    let n = encode(f, &mut out);
    out[..n].to_vec()
}

const FACES: [Face<'static>; 3] = [
    Face { name: b"SWARM", symbol: b"$WARM", uri: b"https://optimism.fit/app/warm-token.json" },
    Face { name: &[b'n'; 32], symbol: &[b's'; 10], uri: &[b'u'; 200] },
    Face { name: b"a", symbol: b"b", uri: b"" },
];

#[test]
fn metaplexs_program_and_bounds_are_the_programs_own() {
    assert_eq!(face::METADATA_PROGRAM_ID.to_bytes(), mpl_token_metadata::ID.to_bytes());
    assert_eq!((face::MAX_NAME, face::MAX_SYMBOL, face::MAX_URI), (mpl_token_metadata::MAX_NAME_LENGTH, mpl_token_metadata::MAX_SYMBOL_LENGTH, mpl_token_metadata::MAX_URI_LENGTH));
    // the metadata address, by Metaplex's seeds
    let mint = solana_address::Address::new_from_array([5; 32]);
    assert_eq!(face::metadata_address(&mint).to_bytes(), Metadata::find_pda(&k(5)).0.to_bytes());
}

#[test]
fn create_metadata_account_v3_is_metaplexs_own() {
    let (metadata, mint, mine, payer, system) = (k(1), k(2), k(3), k(4), Theirs::default());
    for f in &FACES {
        let theirs = CreateMetadataAccountV3 { metadata, mint, mint_authority: mine, payer, update_authority: (mine, true), system_program: system, rent: None }
            .instruction(CreateMetadataAccountV3InstructionArgs { data: data_v2(f), is_mutable: true, collection_details: None });
        assert_eq!(theirs.program_id.to_bytes(), face::METADATA_PROGRAM_ID.to_bytes());
        assert_eq!(theirs.data, ours(face::create_data, f), "{f:?}: CreateMetadataAccountV3's data");
        let flags: Vec<(bool, bool)> = theirs.accounts.iter().map(|m| (m.is_writable, m.is_signer)).collect();
        assert_eq!(flags, face::CREATE_ACCOUNTS.to_vec(), "each account's write and signature");
        let keys: Vec<Theirs> = theirs.accounts.iter().map(|m| m.pubkey).collect();
        assert_eq!(keys, vec![metadata, mint, mine, payer, mine, system], "metadata, mint, the mine twice (mint authority, update authority), payer, system");
    }
}

#[test]
fn update_metadata_account_v2_is_metaplexs_own() {
    let (metadata, mine) = (k(1), k(3));
    for f in &FACES {
        let theirs = UpdateMetadataAccountV2 { metadata, update_authority: mine }
            .instruction(UpdateMetadataAccountV2InstructionArgs { data: Some(data_v2(f)), new_update_authority: None, primary_sale_happened: None, is_mutable: None });
        assert_eq!(theirs.data, ours(face::update_data, f), "{f:?}: UpdateMetadataAccountV2's data");
        let flags: Vec<(bool, bool)> = theirs.accounts.iter().map(|m| (m.is_writable, m.is_signer)).collect();
        assert_eq!(flags, face::UPDATE_ACCOUNTS.to_vec());
        assert_eq!(theirs.accounts.iter().map(|m| m.pubkey).collect::<Vec<_>>(), vec![metadata, mine]);
    }
}

/// A Metadata account as warm_mine's tests lay it (tests/common/mod.rs `metadata_bytes`): each string padded with zeros to its bound.
fn laid(update_authority: &[u8; 32], mint: &[u8; 32], f: &Face) -> Vec<u8> {
    let mut d = vec![face::KEY_METADATA_V1];
    d.extend_from_slice(update_authority);
    d.extend_from_slice(mint);
    for (s, bound) in [(f.name, 32usize), (f.symbol, 10), (f.uri, 200)] {
        d.extend_from_slice(&(bound as u32).to_le_bytes());
        d.extend_from_slice(s);
        d.extend(std::iter::repeat_n(0u8, bound - s.len()));
    }
    d.extend_from_slice(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
    d
}

#[test]
fn the_face_set_face_reads_is_metaplexs_metadata_layout() {
    let pad = |s: &[u8], n: usize| String::from_utf8([s, &vec![0u8; n - s.len()]].concat()).unwrap();
    for f in &FACES {
        let m = Metadata {
            key: Key::MetadataV1,
            update_authority: k(3),
            mint: k(2),
            name: pad(f.name, 32),
            symbol: pad(f.symbol, 10),
            uri: pad(f.uri, 200),
            seller_fee_basis_points: 0,
            creators: None,
            primary_sale_happened: false,
            is_mutable: true,
            edition_nonce: None,
            token_standard: None,
            collection: None,
            uses: None,
            collection_details: None,
            programmable_config: None,
        };
        let bytes = m.try_to_vec().unwrap();
        assert_eq!(bytes, laid(&[3; 32], &[2; 32], f), "{f:?}: the layout warm_mine's tests lay is Metaplex's Metadata, padded as Metaplex stores it");
        let (mine, mint) = (solana_address::Address::new_from_array([3; 32]), solana_address::Address::new_from_array([2; 32]));
        let (name, symbol) = face::read_face(&bytes, &mine, &mint).unwrap();
        assert_eq!((name, symbol), (f.name, f.symbol), "set_face reads the name and the symbol, the padding off");
        assert!(face::read_face(&bytes, &mint, &mint).is_err(), "a face whose update authority is not the mine");
        // and Metaplex reads back what it stored
        assert_eq!(Metadata::from_bytes(&bytes).unwrap().name, pad(f.name, 32));
    }
}
