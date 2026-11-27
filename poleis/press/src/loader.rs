//! The instructions of Solana's own programs the press sends, written by hand as small byte layouts (the system program's, the upgradeable
//! loader's, the compute budget's: bincode as their interface crates write it, a u32 tag then the fields), and the loader's account states
//! the press reads back. Each is held, byte for byte and account by account, to the programs' own interface crates by tests/wire.rs
//! (solana-loader-v3-interface 7.0.0, solana-system-interface 3), which are the press's dev-dependencies only.

use crate::Res;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

pub const SYSTEM: Pubkey = Pubkey::new_from_array([0; 32]);
pub const LOADER_V3: Pubkey = solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111");
pub const RENT_SYSVAR: Pubkey = solana_pubkey::pubkey!("SysvarRent111111111111111111111111111111111");
pub const CLOCK_SYSVAR: Pubkey = solana_pubkey::pubkey!("SysvarC1ock11111111111111111111111111111111");
pub const COMPUTE_BUDGET: Pubkey = solana_pubkey::pubkey!("ComputeBudget111111111111111111111111111111");

/// A buffer's header: [1:4][an authority: 1 and 32]; its bytes after it.
pub const BUFFER_HEADER: usize = 37;
/// A program account: [2:4][its program data:32].
pub const PROGRAM_LEN: usize = 36;
/// A program data's header: [3:4][slot:8][an authority: 1 and 32]; the program's bytes after it.
pub const PROGRAMDATA_HEADER: usize = 45;
/// A seed of the system program's `with_seed` addresses: at most 32 bytes.
pub const MAX_SEED: usize = 32;

fn tag(t: u32) -> Vec<u8> {
    t.to_le_bytes().to_vec()
}

// ---- the system program ----
/// CreateAccount (0): [lamports:8][space:8][owner:32] · [from (s, w), to (s, w)]
pub fn create_account(from: &Pubkey, to: &Pubkey, lamports: u64, space: u64, owner: &Pubkey) -> Instruction {
    let mut d = tag(0);
    d.extend_from_slice(&lamports.to_le_bytes());
    d.extend_from_slice(&space.to_le_bytes());
    d.extend_from_slice(owner.as_ref());
    Instruction::new_with_bytes(SYSTEM, &d, vec![AccountMeta::new(*from, true), AccountMeta::new(*to, true)])
}
/// Assign (1): [owner:32] · [account (s, w)]
pub fn assign(account: &Pubkey, owner: &Pubkey) -> Instruction {
    let mut d = tag(1);
    d.extend_from_slice(owner.as_ref());
    Instruction::new_with_bytes(SYSTEM, &d, vec![AccountMeta::new(*account, true)])
}
/// Transfer (2): [lamports:8] · [from (s, w), to (w)]
pub fn transfer(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let mut d = tag(2);
    d.extend_from_slice(&lamports.to_le_bytes());
    Instruction::new_with_bytes(SYSTEM, &d, vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false)])
}
/// CreateAccountWithSeed (3): [base:32][seed: a string, its u64 length and its bytes][lamports:8][space:8][owner:32] · [from (s, w), to
/// (w), base (s)]
pub fn create_account_with_seed(from: &Pubkey, to: &Pubkey, base: &Pubkey, seed: &str, lamports: u64, space: u64, owner: &Pubkey) -> Instruction {
    let mut d = tag(3);
    d.extend_from_slice(base.as_ref());
    d.extend_from_slice(&(seed.len() as u64).to_le_bytes());
    d.extend_from_slice(seed.as_bytes());
    d.extend_from_slice(&lamports.to_le_bytes());
    d.extend_from_slice(&space.to_le_bytes());
    d.extend_from_slice(owner.as_ref());
    Instruction::new_with_bytes(SYSTEM, &d, vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false), AccountMeta::new_readonly(*base, true)])
}
/// Allocate (8): [space:8] · [account (s, w)]
pub fn allocate(account: &Pubkey, space: u64) -> Instruction {
    let mut d = tag(8);
    d.extend_from_slice(&space.to_le_bytes());
    Instruction::new_with_bytes(SYSTEM, &d, vec![AccountMeta::new(*account, true)])
}
/// AllocateWithSeed (9): [base:32][seed: a string, its u64 length and its bytes][space:8][owner:32] · [account (w), base (s)]: an account at
/// a `with_seed` address given its space and its owner by its base's signature (where a stranger's lamports stand, which bar a create)
pub fn allocate_with_seed(account: &Pubkey, base: &Pubkey, seed: &str, space: u64, owner: &Pubkey) -> Instruction {
    let mut d = tag(9);
    d.extend_from_slice(base.as_ref());
    d.extend_from_slice(&(seed.len() as u64).to_le_bytes());
    d.extend_from_slice(seed.as_bytes());
    d.extend_from_slice(&space.to_le_bytes());
    d.extend_from_slice(owner.as_ref());
    Instruction::new_with_bytes(SYSTEM, &d, vec![AccountMeta::new(*account, false), AccountMeta::new_readonly(*base, true)])
}
/// The address `create_account_with_seed` makes: sha256 of the base, the seed and the owner (the system program's own definition).
pub fn address_with_seed(base: &Pubkey, seed: &str, owner: &Pubkey) -> Res<Pubkey> {
    if seed.len() > MAX_SEED {
        return Err(format!("a seed of {} bytes: at most {MAX_SEED}", seed.len()));
    }
    let mut all = base.as_ref().to_vec();
    all.extend_from_slice(seed.as_bytes());
    all.extend_from_slice(owner.as_ref());
    Ok(Pubkey::new_from_array(crate::wire::sha256(&all)))
}

// ---- the upgradeable loader ----
/// InitializeBuffer (0) · [buffer (w), authority]
pub fn initialize_buffer(buffer: &Pubkey, authority: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(LOADER_V3, &tag(0), vec![AccountMeta::new(*buffer, false), AccountMeta::new_readonly(*authority, false)])
}
/// Write (1): [offset:4][bytes: their u64 length and the bytes] · [buffer (w), authority (s)]
pub fn write(buffer: &Pubkey, authority: &Pubkey, offset: u32, bytes: &[u8]) -> Instruction {
    let mut d = tag(1);
    d.extend_from_slice(&offset.to_le_bytes());
    d.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    d.extend_from_slice(bytes);
    Instruction::new_with_bytes(LOADER_V3, &d, vec![AccountMeta::new(*buffer, false), AccountMeta::new_readonly(*authority, true)])
}
/// The program data of a program: the loader's PDA of its id.
pub fn programdata_of(program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[program.as_ref()], &LOADER_V3).0
}
/// DeployWithMaxDataLen (2): [max_data_len:8] · [payer (s, w), program data (w), program (w), buffer (w), rent, clock, system, authority (s)]
pub fn deploy_with_max_data_len(payer: &Pubkey, program: &Pubkey, buffer: &Pubkey, authority: &Pubkey, max_data_len: u64) -> Instruction {
    let mut d = tag(2);
    d.extend_from_slice(&max_data_len.to_le_bytes());
    Instruction::new_with_bytes(
        LOADER_V3,
        &d,
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(programdata_of(program), false),
            AccountMeta::new(*program, false),
            AccountMeta::new(*buffer, false),
            AccountMeta::new_readonly(RENT_SYSVAR, false),
            AccountMeta::new_readonly(CLOCK_SYSVAR, false),
            AccountMeta::new_readonly(SYSTEM, false),
            AccountMeta::new_readonly(*authority, true),
        ],
    )
}
/// SetAuthority (4) · [program data or buffer (w), current authority (s), the new authority (none: never again)]. The new one need not sign:
/// it is how a program is handed to a PDA, which signs only inside its own program.
pub fn set_authority(account: &Pubkey, current: &Pubkey, new: Option<&Pubkey>) -> Instruction {
    let mut metas = vec![AccountMeta::new(*account, false), AccountMeta::new_readonly(*current, true)];
    if let Some(n) = new {
        metas.push(AccountMeta::new_readonly(*n, false));
    }
    Instruction::new_with_bytes(LOADER_V3, &tag(4), metas)
}
/// Close (5) · [buffer (w), recipient (w), authority (s)]: a buffer of ours that is not this binary's, its rent home.
pub fn close_buffer(buffer: &Pubkey, recipient: &Pubkey, authority: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        LOADER_V3,
        &tag(5),
        vec![AccountMeta::new(*buffer, false), AccountMeta::new(*recipient, false), AccountMeta::new_readonly(*authority, true)],
    )
}

/// ExtendProgram (6): [additional_bytes:4] · [program data (w), program (w), system, payer (s, w)]: the program's data grown by that many bytes,
/// the payer paying the rent of the new length. No authority signs it (Agave 4.2.2's loader: `common_extend_program(.., false)`); the loader
/// takes at least 10,240 bytes at a time (SIMD-0431, or all that is left under 10 MiB) and re-deploys the program at the extend's slot, so an
/// upgrade of it waits for the slot after (THE PRESS IS READY FOR MAINNET: upgrade.rs).
pub fn extend_program(program: &Pubkey, payer: &Pubkey, additional_bytes: u32) -> Instruction {
    let mut d = tag(6);
    d.extend_from_slice(&additional_bytes.to_le_bytes());
    Instruction::new_with_bytes(
        LOADER_V3,
        &d,
        vec![AccountMeta::new(programdata_of(program), false), AccountMeta::new(*program, false), AccountMeta::new_readonly(SYSTEM, false), AccountMeta::new(*payer, true)],
    )
}

// ---- the compute budget ----
/// SetComputeUnitLimit (2): [units:4]
pub fn compute_limit(units: u32) -> Instruction {
    let mut d = vec![2u8];
    d.extend_from_slice(&units.to_le_bytes());
    Instruction::new_with_bytes(COMPUTE_BUDGET, &d, vec![])
}
/// SetComputeUnitPrice (3): [micro-lamports a unit:8]
pub fn compute_price(micro_lamports: u64) -> Instruction {
    let mut d = vec![3u8];
    d.extend_from_slice(&micro_lamports.to_le_bytes());
    Instruction::new_with_bytes(COMPUTE_BUDGET, &d, vec![])
}

/// A loader account as it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Buffer { authority: Option<Pubkey> },
    Program { programdata: Pubkey },
    ProgramData { slot: u64, authority: Option<Pubkey> },
}

fn option_key(d: &[u8], at: usize) -> Res<Option<Pubkey>> {
    match d.get(at) {
        Some(0) => Ok(None),
        Some(1) => Ok(Some(Pubkey::new_from_array(d.get(at + 1..at + 33).ok_or("an authority cut short")?.try_into().unwrap()))),
        _ => Err("an authority's option that is neither 0 nor 1".into()),
    }
}

/// A loader account's state, by its tag.
pub fn state(d: &[u8]) -> Res<State> {
    let t = u32::from_le_bytes(d.get(..4).ok_or("a loader account of no state")?.try_into().unwrap());
    match t {
        1 => Ok(State::Buffer { authority: option_key(d, 4)? }),
        2 => Ok(State::Program { programdata: Pubkey::new_from_array(d.get(4..36).ok_or("a program cut short")?.try_into().unwrap()) }),
        3 => Ok(State::ProgramData { slot: u64::from_le_bytes(d.get(4..12).ok_or("a program data cut short")?.try_into().unwrap()), authority: option_key(d, 12)? }),
        _ => Err(format!("a loader account of state {t}")),
    }
}
