//! A permissionless public index of authenticated source floors.
//! This account grants no authority, moves no source funds, and is not a ballot.
//! Rows describe current holdings; row order and ordinals are not permanent NFTs.

use crate::error::PoleisError;
use crate::state::{Config, Registry, DISC_CONFIG, DISC_REGISTRY, LAYOUT_VERSION};
use crate::util::{
    create_pda, expect_signer, expect_writable, find_and_expect_pda, grow_account, sha256v, Data,
};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult, Resize,
};

pub const LEGACY_PROGRAM_ID: Address =
    solana_address::address!("AJNrRxyvysf7iokrDLBHeoJQUqitiBva3nRUnCj7VEkJ");
pub const DISC_ASSEMBLY_FLOOR: u8 = 16;

/// Fixed header followed by exactly `holder_count` original 57-byte rows.
pub struct AssemblyFloor;
impl AssemblyFloor {
    pub const HEADER: usize = 104;
    pub const OFF_BUMP: usize = 2;
    pub const OFF_SOURCE_LAYOUT: usize = 3;
    pub const OFF_SOURCE_PROGRAM: usize = 4;
    pub const OFF_UNIT: usize = 36;
    pub const OFF_ROOT: usize = 44;
    pub const OFF_REGISTRY_HASH: usize = 52;
    pub const OFF_SYNCED_SLOT: usize = 84;
    pub const OFF_TOTAL_SEATS: usize = 92;
    pub const OFF_RUNG: usize = 100;
    pub const OFF_FROZEN: usize = 101;
    pub const OFF_MODE: usize = 102;
    pub const OFF_HOLDER_COUNT: usize = 103;
    pub const ROW_LEN: usize = Registry::HOLDER_LEN;
    pub const MAX_SIZE: usize = Self::HEADER + crate::MAX_HOLDERS * Self::ROW_LEN;
    pub const fn size_for(rows: usize) -> usize {
        Self::HEADER + rows * Self::ROW_LEN
    }
}

fn u64_at(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

fn expect_source_pda(
    account: &AccountView,
    seeds: &[&[u8]],
    program: &Address,
    bump: u8,
) -> ProgramResult {
    let (expected, canonical_bump) = Address::find_program_address(seeds, program);
    if account.address() != &expected || bump != canonical_bump {
        return Err(PoleisError::InvalidPda.into());
    }
    Ok(())
}

// Sort only a small index, never the source rows. This keeps duplicate checking
// bounded by n log n even when every purchased seat belongs to a different key.
#[inline(never)]
fn check_unique_holders(rows: &[u8], count: usize) -> ProgramResult {
    let mut order = [0u8; crate::MAX_HOLDERS];
    for (i, value) in order[..count].iter_mut().enumerate() {
        *value = i as u8;
    }
    order[..count].sort_unstable_by(|a, b| {
        let a = *a as usize * Registry::HOLDER_LEN;
        let b = *b as usize * Registry::HOLDER_LEN;
        rows[a..a + 33].cmp(&rows[b..b + 33])
    });
    for pair in order[..count].windows(2) {
        let a = pair[0] as usize * Registry::HOLDER_LEN;
        let b = pair[1] as usize * Registry::HOLDER_LEN;
        if rows[a..a + 33] == rows[b..b + 33] {
            return Err(PoleisError::InvalidHolder.into());
        }
    }
    Ok(())
}

/// data: [source_program:32][unit:8]
/// accounts: [payer (s,w), source_registry, source_config, floor_index (w), system]
/// The registry hash covers its full data, including unused allocated rows.
/// An unchanged registry hash and config mode leave the index and payer untouched.
pub fn sync_assembly_floor(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [payer, source_registry, source_config, floor, system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let input = Data(data);
    input.expect_len(40)?;
    let source_program = Address::new_from_array(input.h32(0)?);
    let unit = input.u64(32)?;
    let source_layout = if source_program == crate::ID {
        8
    } else if source_program == LEGACY_PROGRAM_ID {
        7
    } else {
        return Err(PoleisError::InvalidAccount.into());
    };
    expect_signer(payer)?;
    expect_writable(payer)?;
    expect_writable(floor)?;
    if system.address() != &Address::new_from_array([0; 32])
        || !source_registry.owned_by(&source_program)
        || !source_config.owned_by(&source_program)
    {
        return Err(PoleisError::InvalidAccount.into());
    }

    let source = source_registry.try_borrow()?;
    let config = source_config.try_borrow()?;
    // Layout 7 has one fixed-size registry and one config for its root zero.
    let row_offset = if source_layout == 8 {
        Registry::OFF_HOLDERS
    } else {
        Registry::OFF_HOLDERS - 8
    };
    let config_size = if source_layout == 8 {
        Config::SIZE
    } else {
        Config::SIZE - 8
    };
    let valid_size = if source_layout == 8 {
        source.len() >= row_offset
            && source.len() <= Registry::MAX_SIZE
            && (source.len() - row_offset).is_multiple_of(Registry::HOLDER_LEN)
    } else {
        source.len() == row_offset + crate::MAX_HOLDERS * Registry::HOLDER_LEN
    };
    if !valid_size
        || config.len() != config_size
        || source[0] != DISC_REGISTRY
        || config[0] != DISC_CONFIG
    {
        return Err(PoleisError::InvalidAccount.into());
    }
    if source[1] != source_layout || config[1] != source_layout {
        return Err(PoleisError::LayoutVersion.into());
    }
    if u64_at(&source, Registry::OFF_UNIT) != unit {
        return Err(PoleisError::UnitMismatch.into());
    }
    let root = if source_layout == 8 {
        u64_at(&source, Registry::OFF_ROOT)
    } else {
        0
    };
    if source_layout == 8 && u64_at(&config, Config::OFF_ROOT) != root {
        return Err(PoleisError::UnitMismatch.into());
    }
    let mode = config[Config::OFF_MODE];
    if mode > crate::MODE_COUNCIL {
        return Err(PoleisError::InvalidMode.into());
    }
    let unit_le = unit.to_le_bytes();
    let root_le = root.to_le_bytes();
    expect_source_pda(
        source_registry,
        &[b"registry", &unit_le],
        &source_program,
        source[Registry::OFF_BUMP],
    )?;
    let config_seeds: &[&[u8]] = if source_layout == 8 {
        &[b"config", &root_le]
    } else {
        &[b"config"]
    };
    expect_source_pda(
        source_config,
        config_seeds,
        &source_program,
        config[Config::OFF_BUMP],
    )?;

    let count = source[Registry::OFF_HOLDER_COUNT] as usize;
    let rung = source[Registry::OFF_RUNG] as usize;
    let frozen = source[Registry::OFF_FROZEN];
    if count > crate::MAX_HOLDERS || row_offset + count * Registry::HOLDER_LEN > source.len() {
        return Err(PoleisError::RegistryFull.into());
    }
    if rung >= crate::LADDER.len() {
        return Err(PoleisError::NotOnLadder.into());
    }
    let name_offset = if source_layout == 8 {
        Registry::OFF_NAME_LEN
    } else {
        Registry::OFF_NAME_LEN - 8
    };
    let desc_offset = name_offset + 1 + crate::MAX_NAME;
    if frozen > 1
        || source[name_offset] as usize > crate::MAX_NAME
        || source[desc_offset] as usize > crate::MAX_DESC
    {
        return Err(PoleisError::InvalidAccount.into());
    }
    let level = source[Registry::OFF_LEVEL];
    let parent = u64_at(&source, Registry::OFF_PARENT);
    let chair = source[Registry::OFF_CHAIR];
    if level > crate::MAX_DEPTH
        || (unit == root && (level != 0 || parent != root || chair != crate::NO_CHAIR))
        || (unit != root && (level == 0 || parent == unit || chair as usize >= crate::PENTAGONS))
    {
        return Err(PoleisError::NotDescendant.into());
    }
    let rows = &source[row_offset..row_offset + count * Registry::HOLDER_LEN];
    let mut total = 0u64;
    for i in 0..count {
        let row = &rows[i * Registry::HOLDER_LEN..(i + 1) * Registry::HOLDER_LEN];
        let kind = row[32];
        let seats = u64_at(row, 33);
        let weight = u64_at(row, 41);
        // a leaver's emptied row stays in place (rows never move under live sessions)
        if kind == crate::HOLDER_EMPTY && seats == 0 {
            continue;
        }
        // NFT identities only exist in the current program/layout. Retired
        // rows retain their nonzero identity forever, but have no live weight.
        if source_layout == 8 && kind == crate::HOLDER_RETIRED_NFT {
            if row[..32] == [0; 32] || seats != 0 || weight != 0 || u64_at(row, 49) != 0 {
                return Err(PoleisError::InvalidHolder.into());
            }
            continue;
        }
        let nft = source_layout == 8 && kind == crate::HOLDER_NFT;
        if kind != crate::HOLDER_KEY && kind != crate::HOLDER_UNIT && !nft {
            return Err(PoleisError::InvalidHolder.into());
        }
        if nft && (row[..32] == [0; 32] || seats != 1) {
            return Err(PoleisError::InvalidHolder.into());
        }
        if seats == 0 || seats > crate::MAX_SEATS_PER_HOLDER || weight == 0 {
            return Err(PoleisError::SeatLimit.into());
        }
        total = total.checked_add(seats).ok_or(PoleisError::MathOverflow)?;
    }
    check_unique_holders(rows, count)?;
    if total != u64_at(&source, Registry::OFF_TOTAL_SEATS) || total > crate::ladder_hexagons(rung) {
        return Err(PoleisError::SeatLimit.into());
    }
    let hash = sha256v(&[&source]);
    let seeds: &[&[u8]] = &[b"assembly-floor", source_program.as_ref(), &unit_le];
    let bump = find_and_expect_pda(floor, seeds)?;
    let new_size = AssemblyFloor::size_for(count);
    if floor.data_len() == 0 {
        create_pda(payer, floor, new_size, seeds, bump)?;
    } else {
        if !floor.owned_by(&crate::ID) {
            return Err(PoleisError::InvalidAccount.into());
        }
        {
            let old = floor.try_borrow()?;
            if old.len() < AssemblyFloor::HEADER
                || old.len() > AssemblyFloor::MAX_SIZE
                || old.len()
                    != AssemblyFloor::size_for(old[AssemblyFloor::OFF_HOLDER_COUNT] as usize)
                || old[0] != DISC_ASSEMBLY_FLOOR
                || old[1] != LAYOUT_VERSION
                || old[AssemblyFloor::OFF_BUMP] != bump
                || old[AssemblyFloor::OFF_SOURCE_LAYOUT] != source_layout
                || old[4..36] != source_program.to_bytes()
                || u64_at(&old, AssemblyFloor::OFF_UNIT) != unit
                || u64_at(&old, AssemblyFloor::OFF_ROOT) != root
            {
                return Err(PoleisError::InvalidAccount.into());
            }
            if old[52..84] == hash && old[AssemblyFloor::OFF_MODE] == mode {
                return Ok(());
            }
        }
        if new_size > floor.data_len() {
            grow_account(payer, floor, new_size)?;
        } else if new_size < floor.data_len() {
            floor.resize(new_size)?;
        }
    }
    let slot = Clock::get()?.slot;
    let mut out = floor.try_borrow_mut()?;
    out[0] = DISC_ASSEMBLY_FLOOR;
    out[1] = LAYOUT_VERSION;
    out[2] = bump;
    out[3] = source_layout;
    out[4..36].copy_from_slice(source_program.as_ref());
    out[36..44].copy_from_slice(&unit_le);
    out[44..52].copy_from_slice(&root_le);
    out[52..84].copy_from_slice(&hash);
    out[84..92].copy_from_slice(&slot.to_le_bytes());
    out[92..100].copy_from_slice(&total.to_le_bytes());
    out[100] = rung as u8;
    out[101] = frozen;
    out[102] = mode;
    out[103] = count as u8;
    out[AssemblyFloor::HEADER..].copy_from_slice(rows);
    Ok(())
}
