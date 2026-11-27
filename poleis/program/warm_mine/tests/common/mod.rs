//! What the tests share: the world on Mollusk, with the real SPL Token and Associated Token programs (their mainnet ELFs, Mollusk's own
//! copies), the instruction builders and the account readers. The readers read the program's own offsets (warm_mine::state), so a test
//! reads the bytes the program writes; nothing here is in the program. THE BOOK is checked after every instruction that lands.
#![allow(dead_code)]

use crate::built;
use mollusk_svm::{program::keyed_account_for_system_program, result::ProgramResult, Mollusk};
use mollusk_svm_programs_token::{associated_token, token};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;
use warm_mine::error::WarmError;
use warm_mine::law;
use warm_mine::state::{Flask, Mine, Prior, SEED_FLASK, SEED_MINE, SEED_MINT, SEED_PRIOR, SEED_VAULT};
use warm_mine::token::{ACCOUNT_LEN, MINT_LEN};

pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(warm_mine::ID.to_bytes());
pub const TOKEN: Pubkey = Pubkey::new_from_array(warm_mine::token::TOKEN_PROGRAM_ID.to_bytes());
pub const ATA: Pubkey = Pubkey::new_from_array(warm_mine::token::ATA_PROGRAM_ID.to_bytes());
pub const LOADER_V3: Pubkey = Pubkey::new_from_array(warm_mine::token::LOADER_V3_ID.to_bytes());
pub const SOL: u64 = 1_000_000_000;
/// A dollar of USDC, in micros.
pub const USD: u64 = 1_000_000;
/// One $WARM, in nanos.
pub const WARM: u64 = 1_000_000_000;
pub const SECOND: u64 = 1_000;
pub const HOUR: u64 = 3_600_000;
pub const DAY: u64 = 86_400_000;
/// The world's clock: his word's instant, Nov 21 2026 17:34:37 UTC, in seconds.
pub const WORD_S: i64 = 1_795_282_477;
pub const WORD_MS: u64 = WORD_S as u64 * 1_000;

pub fn system_id() -> Pubkey {
    keyed_account_for_system_program().0
}
pub fn code(e: WarmError) -> ProgramError {
    ProgramError::Custom(e as u32)
}

// ---- PDAs ----
pub fn mine_pda() -> Pubkey {
    Pubkey::find_program_address(&[SEED_MINE], &PROGRAM_ID).0
}
pub fn mint_pda() -> Pubkey {
    Pubkey::find_program_address(&[SEED_MINT], &PROGRAM_ID).0
}
pub fn vault_pda() -> Pubkey {
    Pubkey::find_program_address(&[SEED_VAULT], &PROGRAM_ID).0
}
pub fn record_pda(flask: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[SEED_FLASK, flask.as_ref()], &PROGRAM_ID).0
}
pub fn prior_pda(signature: &[u8; 64]) -> Pubkey {
    Pubkey::find_program_address(&[SEED_PRIOR, &signature[..32], &signature[32..]], &PROGRAM_ID).0
}
/// The program's program data: the upgradeable loader's PDA of its id.
pub fn program_data() -> Pubkey {
    Pubkey::find_program_address(&[PROGRAM_ID.as_ref()], &LOADER_V3).0
}
/// `owner`'s associated token account of `mint`.
pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN.as_ref(), mint.as_ref()], &ATA).0
}

// ---- builders ----
fn ix(tag: u8, data: &[u8], metas: Vec<AccountMeta>) -> Instruction {
    let mut d = vec![tag];
    d.extend_from_slice(data);
    Instruction::new_with_bytes(PROGRAM_ID, &d, metas)
}
/// accounts: [upgrade authority (s), payer (s,w), program data, mine (w), mint (w), vault (w), treasury, dollar, token program, system]
pub fn init(upgrader: Pubkey, payer: Pubkey, authority: &Pubkey, opening_ms: u64, treasury: &Pubkey, dollar: &Pubkey) -> Instruction {
    let mut d = authority.as_ref().to_vec();
    d.extend_from_slice(&opening_ms.to_le_bytes());
    ix(
        warm_mine::IX_INIT,
        &d,
        vec![
            AccountMeta::new_readonly(upgrader, true),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(program_data(), false),
            AccountMeta::new(mine_pda(), false),
            AccountMeta::new(mint_pda(), false),
            AccountMeta::new(vault_pda(), false),
            AccountMeta::new_readonly(*treasury, false),
            AccountMeta::new_readonly(*dollar, false),
            AccountMeta::new_readonly(TOKEN, false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// Metaplex's token metadata program, by the program's own constant.
pub const METAPLEX: Pubkey = Pubkey::new_from_array(warm_mine::face::METADATA_PROGRAM_ID.to_bytes());
/// The $WARM mint's metadata address, by Metaplex's seeds.
pub fn metadata_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"metadata", METAPLEX.as_ref(), mint_pda().as_ref()], &METAPLEX).0
}
/// A face as `init` takes it after its 40 bytes: [name_len:1][name][symbol_len:1][symbol][uri_len:1][uri].
pub fn face_bytes(name: &[u8], symbol: &[u8], uri: &[u8]) -> Vec<u8> {
    let mut d = Vec::new();
    for f in [name, symbol, uri] {
        d.push(f.len() as u8);
        d.extend_from_slice(f);
    }
    d
}
/// `init` with a face: the ten accounts, then [metadata (w), Metaplex's program].
#[allow(clippy::too_many_arguments)]
pub fn init_face(upgrader: Pubkey, payer: Pubkey, authority: &Pubkey, opening_ms: u64, treasury: &Pubkey, dollar: &Pubkey, face: &[u8]) -> Instruction {
    let mut ix = init(upgrader, payer, authority, opening_ms, treasury, dollar);
    ix.data.extend_from_slice(face);
    ix.accounts.extend([AccountMeta::new(metadata_pda(), false), AccountMeta::new_readonly(METAPLEX, false)]);
    ix
}
/// accounts: [authority (s), mine, metadata (w), Metaplex's program] · data [uri_len:1][uri]
pub fn set_face(authority: Pubkey, uri: &[u8]) -> Instruction {
    let mut d = vec![uri.len() as u8];
    d.extend_from_slice(uri);
    ix(
        warm_mine::IX_SET_FACE,
        &d,
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new_readonly(mine_pda(), false),
            AccountMeta::new(metadata_pda(), false),
            AccountMeta::new_readonly(METAPLEX, false),
        ],
    )
}
/// A Metadata account as Metaplex lays one (its `Metadata` struct, Borsh, the name, symbol and URI padded with zeros to their bounds as
/// Metaplex stores them): [key 4][update authority][mint][name][symbol][uri][seller fee 0][no creators][primary sale no][mutable][no edition
/// nonce, standard, collection, uses, collection details, programmable config].
pub fn metadata_bytes(update_authority: &Pubkey, mint: &Pubkey, name: &[u8], symbol: &[u8], uri: &[u8]) -> Vec<u8> {
    let mut d = vec![warm_mine::face::KEY_METADATA_V1];
    d.extend_from_slice(update_authority.as_ref());
    d.extend_from_slice(mint.as_ref());
    for (f, bound) in [(name, 32usize), (symbol, 10), (uri, 200)] {
        d.extend_from_slice(&(bound as u32).to_le_bytes());
        d.extend_from_slice(f);
        d.extend(std::iter::repeat_n(0u8, bound - f.len()));
    }
    d.extend_from_slice(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
    d
}

/// accounts: [authority (s), mine (w), treasury]
pub fn set_treasury(authority: Pubkey, treasury: &Pubkey) -> Instruction {
    ix(warm_mine::IX_SET_TREASURY, &[], vec![AccountMeta::new_readonly(authority, true), AccountMeta::new(mine_pda(), false), AccountMeta::new_readonly(*treasury, false)])
}
/// accounts: [authority (s), mine (w)] · data [miner:32]
pub fn set_miner(authority: Pubkey, miner: &Pubkey) -> Instruction {
    ix(warm_mine::IX_SET_MINER, miner.as_ref(), vec![AccountMeta::new_readonly(authority, true), AccountMeta::new(mine_pda(), false)])
}
/// accounts: [flask (s), payer (s,w), mine (w), record (w), source (w), treasury (w), dollar, token program, system]
pub fn deposit(flask: Pubkey, payer: Pubkey, source: &Pubkey, treasury: &Pubkey, dollar: &Pubkey, amount: u64) -> Instruction {
    ix(
        warm_mine::IX_DEPOSIT,
        &amount.to_le_bytes(),
        vec![
            AccountMeta::new_readonly(flask, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(mine_pda(), false),
            AccountMeta::new(record_pda(&flask), false),
            AccountMeta::new(*source, false),
            AccountMeta::new(*treasury, false),
            AccountMeta::new_readonly(*dollar, false),
            AccountMeta::new_readonly(TOKEN, false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// accounts: [authority (s), payer (s,w), mine (w), flask, record (w), prior (w), system] · data [signature:64][usd_micros:8]
pub fn record_prior(authority: Pubkey, payer: Pubkey, flask: &Pubkey, signature: &[u8; 64], usd: u64) -> Instruction {
    let mut d = signature.to_vec();
    d.extend_from_slice(&usd.to_le_bytes());
    ix(
        warm_mine::IX_RECORD_PRIOR,
        &d,
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(mine_pda(), false),
            AccountMeta::new_readonly(*flask, false),
            AccountMeta::new(record_pda(flask), false),
            AccountMeta::new(prior_pda(signature), false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}
/// One spend as the router's ledger has it: whose flask, how many micro-dollars, when, and its place in the ledger.
#[derive(Clone, Copy, Debug)]
pub struct Spend {
    pub flask: Pubkey,
    pub usd: u64,
    pub at: u64,
    pub seq: u64,
}
/// accounts: [miner (s,w), mine (w), mint (w), vault (w), token program, associated token program, system], then each spend's
/// [flask, record (w), $WARM account (w)] · data [n:1] then n × [usd:8][at_ms:8][seq:8]
pub fn mine(miner: Pubkey, spends: &[Spend]) -> Instruction {
    let mut d = vec![spends.len() as u8];
    let mut metas = vec![
        AccountMeta::new(miner, true),
        AccountMeta::new(mine_pda(), false),
        AccountMeta::new(mint_pda(), false),
        AccountMeta::new(vault_pda(), false),
        AccountMeta::new_readonly(TOKEN, false),
        AccountMeta::new_readonly(ATA, false),
        AccountMeta::new_readonly(system_id(), false),
    ];
    for s in spends {
        d.extend_from_slice(&s.usd.to_le_bytes());
        d.extend_from_slice(&s.at.to_le_bytes());
        d.extend_from_slice(&s.seq.to_le_bytes());
        metas.push(AccountMeta::new_readonly(s.flask, false));
        metas.push(AccountMeta::new(record_pda(&s.flask), false));
        metas.push(AccountMeta::new(ata(&s.flask, &mint_pda()), false));
    }
    ix(warm_mine::IX_MINE, &d, metas)
}
/// accounts: [flask (s), payer (s,w), mine (w), record (w), source (w), vault (w), mint, token program, system]
pub fn pay_back(flask: Pubkey, payer: Pubkey, source: &Pubkey, amount: u64) -> Instruction {
    ix(
        warm_mine::IX_PAY_BACK,
        &amount.to_le_bytes(),
        vec![
            AccountMeta::new_readonly(flask, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(mine_pda(), false),
            AccountMeta::new(record_pda(&flask), false),
            AccountMeta::new(*source, false),
            AccountMeta::new(vault_pda(), false),
            AccountMeta::new_readonly(mint_pda(), false),
            AccountMeta::new_readonly(TOKEN, false),
            AccountMeta::new_readonly(system_id(), false),
        ],
    )
}

/// accounts: [miner (s), mine (w), vault (w), mint, token program], then each flask's [record (w), its $WARM account (w)] · data [n:1] then n ×
/// [target:8]: SPENT AS CREDITS, IT RETURNS, each flask's `paid_back` brought to its target from its own $WARM, the mine its delegate
pub fn take_back(miner: Pubkey, targets: &[(Pubkey, u64)]) -> Instruction {
    let mut d = vec![targets.len() as u8];
    let mut metas = vec![
        AccountMeta::new(miner, true),
        AccountMeta::new(mine_pda(), false),
        AccountMeta::new(vault_pda(), false),
        AccountMeta::new_readonly(mint_pda(), false),
        AccountMeta::new_readonly(TOKEN, false),
    ];
    for (flask, target) in targets {
        d.extend_from_slice(&target.to_le_bytes());
        metas.push(AccountMeta::new(record_pda(flask), false));
        metas.push(AccountMeta::new(ata(flask, &mint_pda()), false));
    }
    ix(warm_mine::IX_TAKE_BACK, &d, metas)
}
/// The flask's one approval, by the token program alone: the mine the delegate of its $WARM account, for `amount`
pub fn approve(flask: &Pubkey, delegate: &Pubkey, amount: u64) -> Instruction {
    spl_token_interface::instruction::approve_checked(&TOKEN, &ata(flask, &mint_pda()), &mint_pda(), delegate, flask, &[], amount, warm_mine::token::WARM_DECIMALS).unwrap()
}

// ---- readers: the program's own offsets ----
fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
pub fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
fn key_at(d: &[u8], o: usize) -> Pubkey {
    Pubkey::new_from_array(d[o..o + 32].try_into().unwrap())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MineView {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub dollar: Pubkey,
    pub miner: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub opening_ms: u64,
    pub last_seq: u64,
    pub last_at_ms: u64,
    pub spends: u64,
    pub deposited: u64,
    pub prior: u64,
    pub spent: u64,
    pub minted: u64,
    pub from_vault: u64,
    pub paid_back: u64,
    pub vault_balance: u64,
}
pub fn mine_view(d: &[u8]) -> MineView {
    assert_eq!((d.len(), d[0], d[1]), (Mine::SIZE, warm_mine::state::DISC_MINE, warm_mine::state::LAYOUT_VERSION));
    MineView {
        authority: key_at(d, Mine::OFF_AUTHORITY),
        treasury: key_at(d, Mine::OFF_TREASURY),
        dollar: key_at(d, Mine::OFF_DOLLAR),
        miner: key_at(d, Mine::OFF_MINER),
        mint: key_at(d, Mine::OFF_MINT),
        vault: key_at(d, Mine::OFF_VAULT),
        opening_ms: u64_at(d, Mine::OFF_OPENING),
        last_seq: u64_at(d, Mine::OFF_LAST_SEQ),
        last_at_ms: u64_at(d, Mine::OFF_LAST_AT),
        spends: u64_at(d, Mine::OFF_SPENDS),
        deposited: u64_at(d, Mine::OFF_DEPOSITED),
        prior: u64_at(d, Mine::OFF_PRIOR),
        spent: u64_at(d, Mine::OFF_SPENT),
        minted: u64_at(d, Mine::OFF_MINTED),
        from_vault: u64_at(d, Mine::OFF_FROM_VAULT),
        paid_back: u64_at(d, Mine::OFF_PAID_BACK),
        vault_balance: u64_at(d, Mine::OFF_VAULT_BALANCE),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlaskView {
    pub flask: Pubkey,
    pub warm_account: Pubkey,
    pub deposited: u64,
    pub spent: u64,
    pub mined: u64,
    pub paid_back: u64,
    pub priors: u32,
}
pub fn flask_view(d: &[u8]) -> FlaskView {
    assert_eq!((d.len(), d[0], d[1]), (Flask::SIZE, warm_mine::state::DISC_FLASK, warm_mine::state::LAYOUT_VERSION));
    FlaskView {
        flask: key_at(d, Flask::OFF_FLASK),
        warm_account: key_at(d, Flask::OFF_WARM_ACCOUNT),
        deposited: u64_at(d, Flask::OFF_DEPOSITED),
        spent: u64_at(d, Flask::OFF_SPENT),
        mined: u64_at(d, Flask::OFF_MINED),
        paid_back: u64_at(d, Flask::OFF_PAID_BACK),
        priors: u32_at(d, Flask::OFF_PRIORS),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriorView {
    pub signature: [u8; 64],
    pub flask: Pubkey,
    pub usd: u64,
    pub slot: u64,
}
pub fn prior_view(d: &[u8]) -> PriorView {
    assert_eq!((d.len(), d[0], d[1]), (Prior::SIZE, warm_mine::state::DISC_PRIOR, warm_mine::state::LAYOUT_VERSION));
    PriorView {
        signature: d[Prior::OFF_SIGNATURE..Prior::OFF_SIGNATURE + 64].try_into().unwrap(),
        flask: key_at(d, Prior::OFF_FLASK),
        usd: u64_at(d, Prior::OFF_USD),
        slot: u64_at(d, Prior::OFF_SLOT),
    }
}
/// A mint as the token program lays it: its minting authority, supply, decimals, whether initialized, its freeze authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MintView {
    pub authority: Option<Pubkey>,
    pub supply: u64,
    pub decimals: u8,
    pub initialized: bool,
    pub freeze: Option<Pubkey>,
}
fn option_key(d: &[u8], o: usize) -> Option<Pubkey> {
    match u32_at(d, o) {
        0 => None,
        1 => Some(key_at(d, o + 4)),
        t => panic!("an option's tag {t}"),
    }
}
pub fn mint_view(d: &[u8]) -> MintView {
    assert_eq!(d.len(), MINT_LEN);
    MintView { authority: option_key(d, 0), supply: u64_at(d, 36), decimals: d[44], initialized: d[45] == 1, freeze: option_key(d, 46) }
}
/// A token account: its mint, holder, amount and state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenView {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub state: u8,
}
pub fn token_view(d: &[u8]) -> TokenView {
    assert_eq!(d.len(), ACCOUNT_LEN);
    TokenView { mint: key_at(d, 0), owner: key_at(d, 32), amount: u64_at(d, 64), state: d[108] }
}

/// A mint laid by hand, as the token program lays one: its minting authority, its supply, its decimals, initialized, no freeze authority.
pub fn mint_bytes(authority: Option<&Pubkey>, supply: u64, decimals: u8) -> Vec<u8> {
    let mut d = vec![0u8; MINT_LEN];
    if let Some(a) = authority {
        d[0] = 1;
        d[4..36].copy_from_slice(a.as_ref());
    }
    d[36..44].copy_from_slice(&supply.to_le_bytes());
    d[44] = decimals;
    d[45] = 1;
    d
}
/// A token account laid by hand: its mint, its holder, its amount, initialized.
pub fn token_bytes(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut d = vec![0u8; ACCOUNT_LEN];
    d[..32].copy_from_slice(mint.as_ref());
    d[32..64].copy_from_slice(owner.as_ref());
    d[64..72].copy_from_slice(&amount.to_le_bytes());
    d[108] = 1;
    d
}
/// The program data a deploy leaves: [3:4][slot:8][an upgrade authority: 1 and 32, or 0], the ELF after it (not needed here: Mollusk
/// runs the program from its own cache, and the program reads the header alone).
pub fn program_data_bytes(authority: Option<&Pubkey>) -> Vec<u8> {
    let mut d = vec![3u8, 0, 0, 0];
    d.extend_from_slice(&0u64.to_le_bytes());
    match authority {
        Some(a) => {
            d.push(1);
            d.extend_from_slice(a.as_ref());
        }
        None => d.extend_from_slice(&[0u8; 33]),
    }
    d
}

/// xorshift: a seeded sweep with no dependency (the roads' tests' own)
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

// ---- the world ----
pub struct World {
    pub m: Mollusk,
    pub accounts: Vec<(Pubkey, Account)>,
    /// the program's upgrade authority, as its deploy left it
    pub deployer: Pubkey,
    /// the dollar: a mint of six decimals, as USDC is
    pub dollar: Pubkey,
    /// its minting authority (Mollusk signs for any key: the tests lay balances by hand instead)
    pub dollar_authority: Pubkey,
    /// the treasury's USDC account
    pub treasury: Pubkey,
    /// every spend that landed, by the world's tally: (flask, usd, at, what the law says it mines)
    pub spent: Vec<(Pubkey, u64, u64, u64)>,
    /// every micro-dollar deposited through the program, by the world's tally
    pub deposits: u64,
    /// every treasury the mine has named, and what it held when named
    pub treasuries: Vec<(Pubkey, u64)>,
    /// whether THE BOOK is checked after every instruction that lands
    pub keep_book: bool,
}

impl World {
    /// A world whose clock stands at `now_s` (unix seconds), with the program's data naming a deployer, a dollar and a treasury, and no mine.
    pub fn new(now_s: i64) -> Self {
        std::env::set_var("SBF_OUT_DIR", built::program_dir());
        let mut m = Mollusk::new(&PROGRAM_ID, "warm_mine");
        token::add_program(&mut m);
        associated_token::add_program(&mut m);
        m.sysvars.clock.unix_timestamp = now_s;
        let mut w = Self {
            m,
            accounts: vec![keyed_account_for_system_program(), token::keyed_account(), associated_token::keyed_account()],
            deployer: Pubkey::default(),
            dollar: Pubkey::default(),
            dollar_authority: Pubkey::new_unique(),
            treasury: Pubkey::default(),
            spent: Vec::new(),
            deposits: 0,
            treasuries: Vec::new(),
            keep_book: true,
        };
        w.deployer = w.wallet(10 * SOL);
        let pd = program_data_bytes(Some(&w.deployer));
        w.put_owned(program_data(), LOADER_V3, pd);
        let auth = w.dollar_authority;
        w.dollar = w.lay_mint(Some(&auth), 6);
        let holder = Pubkey::new_unique();
        let dollar = w.dollar;
        w.treasury = w.lay_token(&dollar, &holder, 1_000 * USD);
        w
    }
    /// The world with the mine opened by its deployer: `authority` its authority, the opening `opening_ms`.
    pub fn opened(now_s: i64, authority: &Pubkey, opening_ms: u64) -> Self {
        let mut w = Self::new(now_s);
        let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
        w.ok(&init(deployer, deployer, authority, opening_ms, &treasury, &dollar), "init");
        w
    }
    pub fn now_ms(&self) -> u64 {
        self.m.sysvars.clock.unix_timestamp as u64 * 1_000
    }
    pub fn set_clock_s(&mut self, s: i64) {
        self.m.sysvars.clock.unix_timestamp = s;
    }
    pub fn wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        key
    }
    pub fn put(&mut self, key: Pubkey, account: Account) {
        match self.accounts.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = account,
            None => self.accounts.push((key, account)),
        }
    }
    /// An account of `owner` holding `data`, rent-exempt.
    pub fn put_owned(&mut self, key: Pubkey, owner: Pubkey, data: Vec<u8>) {
        let lamports = self.m.sysvars.rent.minimum_balance(data.len());
        self.put(key, Account { lamports, data, owner, executable: false, rent_epoch: 0 });
    }
    /// A mint of `decimals`, laid by hand.
    pub fn lay_mint(&mut self, authority: Option<&Pubkey>, decimals: u8) -> Pubkey {
        let key = Pubkey::new_unique();
        self.put_owned(key, TOKEN, mint_bytes(authority, 0, decimals));
        key
    }
    /// A token account of `mint` for `owner` holding `amount`, laid by hand at `owner`'s associated address (the mint's supply grows by
    /// it, as a mint-to's would).
    pub fn lay_token(&mut self, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Pubkey {
        let key = ata(owner, mint);
        self.put_owned(key, TOKEN, token_bytes(mint, owner, amount));
        let mut md = self.data(mint);
        let supply = u64_at(&md, 36) + amount;
        md[36..44].copy_from_slice(&supply.to_le_bytes());
        let lamports = self.lamports(mint);
        self.put(*mint, Account { lamports, data: md, owner: TOKEN, executable: false, rent_epoch: 0 });
        key
    }
    /// A flask: a key holding SOL and `usd` micro-dollars in its own USDC account.
    pub fn flask(&mut self, usd: u64) -> (Pubkey, Pubkey) {
        let f = self.wallet(SOL);
        let dollar = self.dollar;
        let source = self.lay_token(&dollar, &f, usd);
        (f, source)
    }
    pub fn account(&self, key: &Pubkey) -> Account {
        self.accounts.iter().find(|(k, _)| k == key).map(|(_, a)| a.clone()).unwrap_or_default()
    }
    pub fn data(&self, key: &Pubkey) -> Vec<u8> {
        self.account(key).data
    }
    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.account(key).lamports
    }
    pub fn mine(&self) -> MineView {
        mine_view(&self.data(&mine_pda()))
    }
    pub fn record(&self, flask: &Pubkey) -> FlaskView {
        flask_view(&self.data(&record_pda(flask)))
    }
    pub fn tokens(&self, key: &Pubkey) -> u64 {
        token_view(&self.data(key)).amount
    }
    /// $WARM in a flask's own account (none: 0)
    pub fn warm_of(&self, flask: &Pubkey) -> u64 {
        let d = self.data(&ata(flask, &mint_pda()));
        if d.is_empty() {
            0
        } else {
            token_view(&d).amount
        }
    }
    /// Run one instruction against the world: its compute units, or the program's refusal. Only a success changes the world (a failed
    /// instruction leaves nothing, as on a chain).
    pub fn run(&mut self, ix: &Instruction) -> Result<u64, ProgramError> {
        for meta in &ix.accounts {
            if !self.accounts.iter().any(|(k, _)| *k == meta.pubkey) {
                self.accounts.push((meta.pubkey, Account::default()));
            }
        }
        let r = self.m.process_instruction(ix, &self.accounts);
        match r.program_result {
            ProgramResult::Success => {
                for (key, account) in r.resulting_accounts {
                    self.put(key, account);
                }
                self.tally(ix);
                if self.keep_book {
                    book(self);
                }
                Ok(r.compute_units_consumed)
            }
            ProgramResult::Failure(e) => Err(e),
            ProgramResult::UnknownError(e) => panic!("the runtime failed: {e:?}"),
        }
    }
    /// What a landed instruction adds to the world's own tally: each spend mined, by the law, and each deposit.
    fn tally(&mut self, ix: &Instruction) {
        if ix.program_id != PROGRAM_ID {
            return;
        }
        match ix.data[0] {
            warm_mine::IX_MINE => {
                let opening = self.mine().opening_ms;
                let n = ix.data[1] as usize;
                for i in 0..n {
                    let o = 2 + i * 24;
                    let (usd, at) = (u64_at(&ix.data, o), u64_at(&ix.data, o + 8));
                    let flask = ix.accounts[7 + i * 3].pubkey;
                    let warm = u64::try_from(law::warm_nanos(usd, at, opening)).unwrap();
                    self.spent.push((flask, usd, at, warm));
                }
            }
            warm_mine::IX_DEPOSIT => self.deposits += u64_at(&ix.data, 1),
            warm_mine::IX_INIT | warm_mine::IX_SET_TREASURY => {
                let t = self.mine().treasury;
                if !self.treasuries.iter().any(|(k, _)| *k == t) {
                    let held = self.tokens(&t);
                    self.treasuries.push((t, held));
                }
            }
            _ => {}
        }
    }
    /// Run an instruction that must succeed: its compute units.
    pub fn ok(&mut self, ix: &Instruction, what: &str) -> u64 {
        match self.run(ix) {
            Ok(cu) => cu,
            Err(e) => panic!("{what}: refused with {e:?}"),
        }
    }
    /// Run an instruction that must be refused with exactly `e`, and check that no account of the world changed.
    pub fn refuse(&mut self, ix: &Instruction, e: ProgramError, what: &str) {
        let before = self.accounts.clone();
        let got = self.run(ix);
        assert_eq!(got, Err(e), "{what}");
        for (k, a) in &self.accounts {
            if !before.iter().any(|(b, _)| b == k) {
                assert_eq!(a, &Account::default(), "{what}: account {k} was created");
            }
        }
        for (k, a) in &before {
            assert_eq!(&self.account(k), a, "{what}: account {k} changed");
        }
    }
}

/// THE BOOK of the mine, read off the accounts (a world with no mine has none): for every flask, `spent` never past `deposited`; the
/// records' deposits, spends, $WARM issued and paid back sum to the mine's; the $WARM every spend issued is what was minted and what was
/// paid out of the vault, and it is the law's over every spend that landed, to the nano; the vault holds what was paid back less what was
/// paid out of it; the mint's supply is what was minted, and every $WARM account of the mint together holds it; the treasury holds what it
/// held and every deposit through the program.
pub fn book(w: &World) {
    let md = w.data(&mine_pda());
    if md.is_empty() {
        return;
    }
    let m = mine_view(&md);
    let (mut deposited, mut spent, mut mined, mut paid_back) = (0u128, 0u128, 0u128, 0u128);
    let (mut held, mut supply_seen) = (0u128, false);
    for (k, a) in &w.accounts {
        if a.owner == PROGRAM_ID && a.data.len() == Flask::SIZE && a.data[0] == warm_mine::state::DISC_FLASK {
            let r = flask_view(&a.data);
            assert_eq!(*k, record_pda(&r.flask), "a record stands at its flask's own address");
            assert!(r.spent <= r.deposited, "THE BOUND: flask {} spent {} of {} deposited", r.flask, r.spent, r.deposited);
            deposited += r.deposited as u128;
            spent += r.spent as u128;
            mined += r.mined as u128;
            paid_back += r.paid_back as u128;
        }
        if a.owner == TOKEN && a.data.len() == ACCOUNT_LEN && &a.data[..32] == m.mint.as_ref() {
            held += token_view(&a.data).amount as u128;
        }
        if *k == m.mint {
            supply_seen = true;
        }
    }
    assert!(supply_seen, "the mint stands");
    let law: u128 = w.spent.iter().map(|s| s.3 as u128).sum();
    let mint = mint_view(&w.data(&m.mint));
    assert_eq!((deposited, spent), (m.deposited as u128, m.spent as u128), "THE BOOK: the records' deposits and spends are the mine's");
    assert_eq!(paid_back, m.paid_back as u128, "THE BOOK: the records' pay-backs are the mine's");
    assert_eq!(mined, m.minted as u128 + m.from_vault as u128, "THE BOOK: the $WARM issued is what was minted and what was paid out of the vault");
    assert_eq!(mined, law, "THE BOOK: the $WARM issued is the law's over every spend that landed, to the nano");
    assert_eq!(m.vault_balance as u128 + m.from_vault as u128, m.paid_back as u128, "THE BOOK: the vault holds what was paid back less what left it");
    assert_eq!(w.tokens(&m.vault), m.vault_balance, "THE BOOK: the vault's account holds what the mine says");
    assert_eq!(mint.supply, m.minted, "THE BOOK: the supply is what was minted");
    assert_eq!(held, mint.supply as u128, "THE BOOK: every $WARM account together holds the supply");
    let (now, then): (u128, u128) = w.treasuries.iter().fold((0, 0), |(n, t), (k, held)| (n + w.tokens(k) as u128, t + *held as u128));
    assert_eq!(now, then + w.deposits as u128, "THE BOOK: the treasuries the mine named hold what they held and every deposit");
}
