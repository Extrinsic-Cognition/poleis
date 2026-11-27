//! The nine instructions: `init`, `set_treasury`, `set_miner`, `deposit`, `record_prior`, `mine`, `pay_back`, `set_face`, `take_back`.

use crate::error::WarmError;
use crate::face::{self, Face, METADATA_PROGRAM_ID};
use crate::law;
use crate::state::{Flask, Mine, Prior, SEED_FLASK, SEED_MINE, SEED_MINT, SEED_PRIOR, SEED_VAULT};
use crate::token::{self, expect_token_program, ACCOUNT_LEN, ATA_PROGRAM_ID, DOLLAR_DECIMALS, LOADER_V3_ID, MINT_LEN, TOKEN_PROGRAM_ID, WARM_DECIMALS};
use crate::util::{create_pda, expect_signer, expect_writable, find_and_expect_pda, now_ms, now_slot, stands, Data};
use crate::{MIN_DEPOSIT_MICROS, SPEND_ACCOUNTS, SPEND_LEN};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};

/// The program's upgrade authority signed: its program data (the upgradeable loader's PDA of this program's id) is the loader's, a
/// ProgramData ([3:4][slot:8][an authority: 1 and 32]) naming `upgrader`. A program whose upgrades were turned off names none, and opens
/// nothing.
fn expect_upgrade_authority(program_data: &AccountView, upgrader: &AccountView) -> ProgramResult {
    let (at, _) = Address::find_program_address(&[crate::ID.as_ref()], &LOADER_V3_ID);
    if program_data.address() != &at || !program_data.owned_by(&LOADER_V3_ID) {
        return Err(WarmError::InvalidAccount.into());
    }
    let d = program_data.try_borrow()?;
    if d.len() < 45 || d[..4] != [3, 0, 0, 0] {
        return Err(WarmError::InvalidAccount.into());
    }
    if d[12] != 1 || &d[13..45] != upgrader.address().as_ref() {
        return Err(WarmError::Unauthorized.into());
    }
    Ok(())
}

/// data: [authority:32][opening_ms:8], then the face or nothing: [name_len:1][name][symbol_len:1][symbol][uri_len:1][uri]
/// accounts: [upgrade authority (s), payer (s,w), program data, mine (w), $WARM mint (w), vault (w), treasury, dollar, token program, system],
/// then, with a face, [metadata (w), Metaplex's token metadata program]
/// Opens the mine, once: the $WARM mint (nine decimals, the mine its minting authority, no freeze authority, nothing minted), the vault
/// (a $WARM account the mine holds) and the mine itself, naming its AUTHORITY, its TREASURY (a USDC account, of a mint of six decimals:
/// the DOLLAR every deposit is counted in) and its OPENING. Signed by the program's upgrade authority; the payer pays the three rents.
/// THE TOKEN'S FACE: given a name, a symbol and a URI, it also makes the mint's Metaplex metadata by CPI, the mine signing as the mint's
/// authority and as the face's update authority, the payer paying its rent; with none, the mine has no face (a chain with no Metaplex).
pub fn init(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let (head, extra) = accounts.split_at_mut(accounts.len().min(10));
    let [upgrader, payer, program_data, mine, mint, vault, treasury, dollar, token_program, system] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let (fixed, rest) = data.split_at(data.len().min(40));
    let d = Data(fixed);
    d.expect_len(40)?;
    let face: Option<Face> = if rest.is_empty() { None } else { Some(face::parse_face(rest)?) };
    // the face's two accounts, exactly, or none
    let face_accounts = match (&face, extra) {
        (None, []) => None,
        (Some(_), [metadata, program]) => Some((metadata, program)),
        _ => return Err(ProgramError::NotEnoughAccountKeys),
    };
    let authority = Address::new_from_array(d.h32(0)?);
    let opening_ms = d.u64(32)?;
    expect_signer(upgrader)?;
    expect_signer(payer)?;
    expect_writable(payer)?;
    expect_upgrade_authority(program_data, upgrader)?;
    let bump = find_and_expect_pda(mine, &[SEED_MINE])?;
    if stands(mine) {
        return Err(WarmError::MineStands.into());
    }
    // the face's two accounts, checked before anything is made: Metaplex's program, and the mint's own metadata address
    if let Some((metadata, program)) = &face_accounts {
        expect_writable(metadata)?;
        if program.address() != &METADATA_PROGRAM_ID {
            return Err(ProgramError::IncorrectProgramId);
        }
        if metadata.address() != &face::metadata_address(mint.address()) {
            return Err(WarmError::InvalidPda.into());
        }
    }
    for acc in [&*mine, &*mint, &*vault] {
        expect_writable(acc)?;
    }
    if authority == Address::default() {
        return Err(WarmError::InvalidAccount.into());
    }
    expect_token_program(token_program)?;
    if token::mint_decimals(dollar)? != DOLLAR_DECIMALS {
        return Err(WarmError::NotADollar.into());
    }
    if &token::token_account(treasury)?.mint != dollar.address() {
        return Err(WarmError::NotADollar.into());
    }
    let mint_bump = find_and_expect_pda(mint, &[SEED_MINT])?;
    let vault_bump = find_and_expect_pda(vault, &[SEED_VAULT])?;
    create_pda(payer, mint, MINT_LEN, &TOKEN_PROGRAM_ID, &[SEED_MINT], mint_bump)?;
    token::initialize_mint2(mint, WARM_DECIMALS, mine.address())?;
    create_pda(payer, vault, ACCOUNT_LEN, &TOKEN_PROGRAM_ID, &[SEED_VAULT], vault_bump)?;
    token::initialize_account3(vault, mint, mine.address())?;
    create_pda(payer, mine, Mine::SIZE, &crate::ID, &[SEED_MINE], bump)?;
    Mine {
        authority,
        treasury: *treasury.address(),
        dollar: *dollar.address(),
        miner: Address::default(),
        mint: *mint.address(),
        vault: *vault.address(),
        opening_ms,
        last_seq: 0,
        last_at_ms: 0,
        spends: 0,
        deposited: 0,
        prior: 0,
        spent: 0,
        minted: 0,
        from_vault: 0,
        paid_back: 0,
        vault_balance: 0,
        bump,
        mint_bump,
        vault_bump,
    }
    .write(mine)?;
    if let (Some(f), Some((metadata, _))) = (face, face_accounts) {
        let bump_ref = [bump];
        let seeds = [Seed::from(SEED_MINE), Seed::from(&bump_ref[..])];
        face::create(metadata, mint, mine, payer, system, &f, &[Signer::from(&seeds[..])])?;
    }
    Ok(())
}

/// data: [uri_len:1][uri] · accounts: [authority (s), mine, metadata (w), Metaplex's token metadata program]
/// THE FACE MOVES BY THE AUTHORITY ALONE: the URI of the token's picture moves (a site deploy); its name and its symbol, read off the face
/// Metaplex holds, never do. The mine signs Metaplex's UpdateMetadataAccountV2 as the face's update authority.
pub fn set_face(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, mine, metadata, program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let uri = face::parse_uri(data)?;
    expect_signer(authority)?;
    let m = Mine::read_pda(mine)?;
    if authority.address() != &m.authority {
        return Err(WarmError::Unauthorized.into());
    }
    expect_writable(metadata)?;
    if program.address() != &METADATA_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !metadata.owned_by(&METADATA_PROGRAM_ID) || metadata.address() != &face::metadata_address(&m.mint) {
        return Err(WarmError::InvalidAccount.into());
    }
    // the name and the symbol as they stand, copied out of the account before the CPI borrows it
    let (mut name, mut symbol) = ([0u8; face::MAX_NAME], [0u8; face::MAX_SYMBOL]);
    let (nl, sl) = {
        let d = metadata.try_borrow()?;
        let (n, s) = face::read_face(&d, mine.address(), &m.mint)?;
        name[..n.len()].copy_from_slice(n);
        symbol[..s.len()].copy_from_slice(s);
        (n.len(), s.len())
    };
    let f = Face { name: &name[..nl], symbol: &symbol[..sl], uri };
    let bump_ref = [m.bump];
    let seeds = [Seed::from(SEED_MINE), Seed::from(&bump_ref[..])];
    face::update(metadata, mine, &f, &[Signer::from(&seeds[..])])
}

/// The mine, checked, and `authority` its authority and a signer.
fn by_authority(authority: &AccountView, mine: &AccountView) -> Result<Mine, ProgramError> {
    expect_signer(authority)?;
    expect_writable(mine)?;
    let m = Mine::read_pda(mine)?;
    if authority.address() != &m.authority {
        return Err(WarmError::Unauthorized.into());
    }
    Ok(m)
}

/// data: none · accounts: [authority (s), mine (w), treasury]
/// The authority moves the treasury to another USDC account of the same dollar. The authority and the opening never move.
pub fn set_treasury(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, mine, treasury] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    let mut m = by_authority(authority, mine)?;
    if token::token_account(treasury)?.mint != m.dollar {
        return Err(WarmError::NotADollar.into());
    }
    m.treasury = *treasury.address();
    m.write(mine)
}

/// data: [miner:32] · accounts: [authority (s), mine (w)]
/// The authority names the miner, the router's crank key, or renames it; all zero names none, and nothing is mined until it names one.
pub fn set_miner(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, mine] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(32)?;
    let mut m = by_authority(authority, mine)?;
    m.miner = Address::new_from_array(d.h32(0)?);
    m.write(mine)
}

/// A flask's record: the one that stands (its own flask's), or made now at its PDA, the payer paying its rent, with the flask's $WARM
/// account (its associated token account) found once.
fn record_of(payer: &AccountView, record: &AccountView, flask: &Address, m: &Mine) -> Result<Flask, ProgramError> {
    if stands(record) {
        let r = Flask::read_pda(record)?;
        if &r.flask != flask {
            return Err(WarmError::InvalidAccount.into());
        }
        return Ok(r);
    }
    expect_signer(payer)?;
    expect_writable(payer)?;
    let bump = find_and_expect_pda(record, &[SEED_FLASK, flask.as_ref()])?;
    create_pda(payer, record, Flask::SIZE, &crate::ID, &[SEED_FLASK, flask.as_ref()], bump)?;
    let (warm_account, _) = Address::find_program_address(&[flask.as_ref(), TOKEN_PROGRAM_ID.as_ref(), m.mint.as_ref()], &ATA_PROGRAM_ID);
    Ok(Flask { flask: *flask, warm_account, deposited: 0, spent: 0, mined: 0, paid_back: 0, priors: 0, bump })
}

/// data: [amount:8]
/// accounts: [flask (s), payer (s,w), mine (w), record (w), source (w), treasury (w), dollar, token program, system]
/// DEPOSITS ARE COUNTED ON CHAIN: the flask's key moves `amount` (10 USDC at least) of the dollar from its own USDC account to the
/// treasury's, and the flask's record counts it (made at its first deposit, the payer paying its rent: any fee payer, the house's relay
/// as today's door has it). The source is the flask's own: its holder is the flask.
pub fn deposit(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [flask, payer, mine, record, source, treasury, dollar, token_program, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let amount = d.u64(0)?;
    expect_signer(flask)?;
    for acc in [&*mine, &*record, &*source, &*treasury] {
        expect_writable(acc)?;
    }
    let mut m = Mine::read_pda(mine)?;
    if amount < MIN_DEPOSIT_MICROS {
        return Err(WarmError::UnderMinimum.into());
    }
    // a transfer from the treasury to itself moves nothing: it is never a deposit
    if treasury.address() != &m.treasury || dollar.address() != &m.dollar || source.address() == treasury.address() {
        return Err(WarmError::InvalidAccount.into());
    }
    expect_token_program(token_program)?;
    let s = token::token_account(source)?;
    if s.mint != m.dollar || &s.owner != flask.address() {
        return Err(WarmError::InvalidAccount.into());
    }
    let mut r = record_of(payer, record, flask.address(), &m)?;
    r.deposited = r.deposited.checked_add(amount).ok_or(WarmError::MathOverflow)?;
    m.deposited = m.deposited.checked_add(amount).ok_or(WarmError::MathOverflow)?;
    token::transfer_checked(source, dollar, treasury, flask, amount, DOLLAR_DECIMALS, &[])?;
    r.write(record)?;
    m.write(mine)
}

/// data: [signature:64][usd_micros:8]
/// accounts: [authority (s), payer (s,w), mine (w), flask, record (w), prior (w), system]
/// A deposit that reached the treasury before this program (his pre-deposit, or any made through today's door), added to the flask's
/// `deposited` by the authority (a sealed act of the polis), its transaction's signature kept: the prior is an account of its own at that
/// signature, so it is counted once, for one flask, ever. The payer pays the prior's rent, and the record's if it is new.
pub fn record_prior(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, payer, mine, flask, record, prior, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(72)?;
    let signature = d.h64(0)?;
    let usd = d.u64(64)?;
    let mut m = by_authority(authority, mine)?;
    expect_signer(payer)?;
    expect_writable(payer)?;
    expect_writable(record)?;
    expect_writable(prior)?;
    if usd == 0 {
        return Err(WarmError::InvalidAmount.into());
    }
    let seeds: [&[u8]; 3] = [SEED_PRIOR, &signature[..32], &signature[32..]];
    let bump = find_and_expect_pda(prior, &seeds)?;
    if stands(prior) {
        return Err(WarmError::PriorCounted.into());
    }
    let mut r = record_of(payer, record, flask.address(), &m)?;
    create_pda(payer, prior, Prior::SIZE, &crate::ID, &seeds, bump)?;
    Prior { signature, flask: *flask.address(), usd_micros: usd, slot: now_slot()?, bump }.write(prior)?;
    r.deposited = r.deposited.checked_add(usd).ok_or(WarmError::MathOverflow)?;
    r.priors = r.priors.checked_add(1).ok_or(WarmError::MathOverflow)?;
    m.deposited = m.deposited.checked_add(usd).ok_or(WarmError::MathOverflow)?;
    m.prior = m.prior.checked_add(usd).ok_or(WarmError::MathOverflow)?;
    r.write(record)?;
    m.write(mine)
}

/// data: [n:1] then n × [usd_micros:8][at_ms:8][seq:8]
/// accounts: [miner (s,w), mine (w), $WARM mint (w), vault (w), token program, associated token program, system], then n × [flask, its
/// record (w), its $WARM account (w)]
/// MINING IS LIVE, BY THE MINER, AND BOUNDED BY THE MONEY. Each spend, in order: its `seq` above every seq mined before it (one counter,
/// the mine's), its `at_ms` no earlier than the last spend mined and no later than the chain's clock, and the flask's `spent` with it no
/// more than its `deposited` (THE BOUND); then the law's $WARM at `at_ms` (law.rs) goes to the flask's own $WARM account (made if missing,
/// the miner paying its rent), from the vault first and minted only for the rest. One refusal refuses the whole batch: nothing of it lands.
pub fn mine(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() < 7 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, spends) = accounts.split_at_mut(7);
    let [miner, mine, mint, vault, token_program, ata_program, system] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    let n = d.u8(0)? as usize;
    if n == 0 || data.len() != 1 + n * SPEND_LEN {
        return Err(ProgramError::InvalidInstructionData);
    }
    if spends.len() != n * SPEND_ACCOUNTS {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    expect_signer(miner)?;
    for acc in [&*miner, &*mine, &*mint, &*vault] {
        expect_writable(acc)?;
    }
    let mut m = Mine::read_pda(mine)?;
    if miner.address() != &m.miner {
        return Err(WarmError::Unauthorized.into());
    }
    if mint.address() != &m.mint || vault.address() != &m.vault {
        return Err(WarmError::InvalidAccount.into());
    }
    expect_token_program(token_program)?;
    if ata_program.address() != &ATA_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let clock_ms = now_ms()?;
    let bump = [m.bump];
    let seeds = [Seed::from(SEED_MINE), Seed::from(&bump[..])];
    for (i, spend) in spends.chunks_exact_mut(SPEND_ACCOUNTS).enumerate() {
        let [flask, record, warm_account] = spend else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        let o = 1 + i * SPEND_LEN;
        let (usd, at, seq) = (d.u64(o)?, d.u64(o + 8)?, d.u64(o + 16)?);
        if seq <= m.last_seq {
            return Err(WarmError::SeqNotRising.into());
        }
        if at < m.last_at_ms {
            return Err(WarmError::Backwards.into());
        }
        if at > clock_ms {
            return Err(WarmError::Future.into());
        }
        if usd == 0 {
            return Err(WarmError::InvalidAmount.into());
        }
        expect_writable(record)?;
        expect_writable(warm_account)?;
        let mut r = Flask::read_pda(record)?;
        if flask.address() != &r.flask || warm_account.address() != &r.warm_account {
            return Err(WarmError::InvalidAccount.into());
        }
        let spent = r.spent.checked_add(usd).ok_or(WarmError::MathOverflow)?;
        if spent > r.deposited {
            return Err(WarmError::PastDeposits.into());
        }
        let warm = u64::try_from(law::warm_nanos(usd, at, m.opening_ms)).map_err(|_| WarmError::MathOverflow)?;
        if warm > 0 {
            if warm_account.data_len() == 0 {
                token::create_associated(miner, warm_account, flask, mint, system, token_program)?;
            }
            let from_vault = if m.vault_balance < warm { m.vault_balance } else { warm };
            let fresh = warm - from_vault;
            if from_vault > 0 {
                token::transfer_checked(vault, mint, warm_account, mine, from_vault, WARM_DECIMALS, &[Signer::from(&seeds[..])])?;
            }
            if fresh > 0 {
                token::mint_to(mint, warm_account, mine, fresh, &[Signer::from(&seeds[..])])?;
            }
            m.vault_balance -= from_vault;
            m.from_vault = m.from_vault.checked_add(from_vault).ok_or(WarmError::MathOverflow)?;
            m.minted = m.minted.checked_add(fresh).ok_or(WarmError::MathOverflow)?;
        }
        r.spent = spent;
        r.mined = r.mined.checked_add(warm).ok_or(WarmError::MathOverflow)?;
        r.write(record)?;
        m.spent = m.spent.checked_add(usd).ok_or(WarmError::MathOverflow)?;
        m.spends = m.spends.checked_add(1).ok_or(WarmError::MathOverflow)?;
        m.last_seq = seq;
        m.last_at_ms = at;
    }
    m.write(mine)
}

/// data: [amount:8]
/// accounts: [flask (s), payer (s,w), mine (w), record (w), source (w), vault (w), $WARM mint, token program, system]
/// WHAT RETURNS IS EMITTED AGAIN: the flask's key moves `amount` of its $WARM from its own account into the vault (how a drink paid in
/// $WARM reaches the treasury on chain), and its record and the mine count it; the next spends are paid from the vault before any mint.
pub fn pay_back(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [flask, payer, mine, record, source, vault, mint, token_program, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let amount = d.u64(0)?;
    expect_signer(flask)?;
    for acc in [&*mine, &*record, &*source, &*vault] {
        expect_writable(acc)?;
    }
    let mut m = Mine::read_pda(mine)?;
    if amount == 0 {
        return Err(WarmError::InvalidAmount.into());
    }
    if vault.address() != &m.vault || mint.address() != &m.mint {
        return Err(WarmError::InvalidAccount.into());
    }
    expect_token_program(token_program)?;
    let s = token::token_account(source)?;
    if s.mint != m.mint || &s.owner != flask.address() {
        return Err(WarmError::InvalidAccount.into());
    }
    let mut r = record_of(payer, record, flask.address(), &m)?;
    token::transfer_checked(source, mint, vault, flask, amount, WARM_DECIMALS, &[])?;
    r.paid_back = r.paid_back.checked_add(amount).ok_or(WarmError::MathOverflow)?;
    m.paid_back = m.paid_back.checked_add(amount).ok_or(WarmError::MathOverflow)?;
    m.vault_balance = m.vault_balance.checked_add(amount).ok_or(WarmError::MathOverflow)?;
    r.write(record)?;
    m.write(mine)
}

/// data: [n:1] then n × [target:8] · accounts: [miner (s), mine (w), vault (w), $WARM mint, token program], then n × [record (w), source (w)]
/// SPENT AS CREDITS, IT RETURNS (his press of Nov 23 2026 18:17:41 UTC, "Back to the mine": "One $WARM, the token: spending it as credits
/// returns it, to be mined again"). A flask that made the mine the delegate of its own $WARM account keeps its $WARM; what its tank drank as
/// credits comes back here, by the miner alone: each flask's record's `paid_back` becomes `target` (the $WARM its tank has drunk, as the
/// router's ledger says it), the difference moving from the flask's own $WARM account into the vault, the mine signing as the account's
/// delegate. A target, never an amount: a batch sent twice moves nothing twice, and a target the record has reached moves nothing. The token
/// program holds the rest: an account whose delegate is not the mine, or whose delegation or balance is short, refuses the whole batch. The
/// next spends are paid from the vault before anything is minted, as a pay-back's are.
pub fn take_back(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() < 5 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, flasks) = accounts.split_at_mut(5);
    let [miner, mine, vault, mint, token_program] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    let n = d.u8(0)? as usize;
    if n == 0 || data.len() != 1 + n * 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    if flasks.len() != n * 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    expect_signer(miner)?;
    expect_writable(mine)?;
    expect_writable(vault)?;
    let mut m = Mine::read_pda(mine)?;
    if miner.address() != &m.miner {
        return Err(WarmError::Unauthorized.into());
    }
    if vault.address() != &m.vault || mint.address() != &m.mint {
        return Err(WarmError::InvalidAccount.into());
    }
    expect_token_program(token_program)?;
    let bump = [m.bump];
    let seeds = [Seed::from(SEED_MINE), Seed::from(&bump[..])];
    for (i, pair) in flasks.chunks_exact_mut(2).enumerate() {
        let [record, source] = pair else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        let target = d.u64(1 + i * 8)?;
        expect_writable(record)?;
        expect_writable(source)?;
        let mut r = Flask::read_pda(record)?;
        if source.address() != &r.warm_account {
            return Err(WarmError::InvalidAccount.into());
        }
        if target <= r.paid_back {
            continue;
        }
        let amount = target - r.paid_back;
        token::transfer_checked(source, mint, vault, mine, amount, WARM_DECIMALS, &[Signer::from(&seeds[..])])?;
        r.paid_back = target;
        r.write(record)?;
        m.paid_back = m.paid_back.checked_add(amount).ok_or(WarmError::MathOverflow)?;
        m.vault_balance = m.vault_balance.checked_add(amount).ok_or(WarmError::MathOverflow)?;
    }
    m.write(mine)
}
