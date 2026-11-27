//! Genesis and seats. A republic is born a dodecahedron: up to twelve
//! founders in the chairs and no hexagon floor; the floor opens with the
//! genesis sale on the ladder and grows only by CODEX vote. Ratification is
//! permissionless and one-way: the authority becomes a PDA nobody holds.
//! Seats are property: they move by the holder's signature.

use super::sale::{check_windows, open_sale};
use crate::error::PoleisError;
use crate::state::{unit_holder, Pentagon, Registry, SaleHead, Treasury};
use crate::util::{create_pda, expect_pda, expect_signer, expect_writable, find_and_expect_pda, Data};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};

/// The founding of one unit, shared by genesis and the CODEX FOUND payload:
/// [child_unit:8][chair:1][charter:32][window:8][plan_cap:8][plan_period:8]
/// [n:1][founders: n × 32][name_len:1][name][desc_len:1][desc]
pub struct FoundParams<'a> {
    pub child_unit: u64,
    pub chair: u8,
    pub charter: [u8; 32],
    pub window: u64,
    pub plan_cap: u64,
    pub plan_period: u64,
    pub founders: &'a [u8],
    pub name: &'a [u8],
    pub desc: &'a [u8],
}

/// [len:1][bytes] with a ceiling; returns the text and the bytes consumed.
pub fn parse_text(data: &[u8], max: usize) -> Result<(&[u8], usize), ProgramError> {
    let n = *data.first().ok_or(ProgramError::InvalidInstructionData)? as usize;
    if n > max || data.len() < 1 + n {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok((&data[1..1 + n], 1 + n))
}

impl<'a> FoundParams<'a> {
    pub const HEAD: usize = 66;
    pub fn parse(data: &'a [u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::HEAD {
            return Err(ProgramError::InvalidInstructionData);
        }
        let d = Data(data);
        let n = data[65] as usize;
        if n > crate::PENTAGONS || data.len() < Self::HEAD + n * 32 + 2 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let end = Self::HEAD + n * 32;
        let (name, used) = parse_text(&data[end..], crate::MAX_NAME)?;
        let (desc, used2) = parse_text(&data[end + used..], crate::MAX_DESC)?;
        if data.len() != end + used + used2 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            child_unit: d.u64(0)?,
            chair: data[8],
            charter: d.h32(9)?,
            window: d.u64(41)?,
            plan_cap: d.u64(49)?,
            plan_period: d.u64(57)?,
            founders: &data[Self::HEAD..end],
            name,
            desc,
        })
    }
}

pub(crate) fn seat_founders(registry: &mut AccountView, founders: &[u8], now: u64) -> Result<(), ProgramError> {
    for (k, f) in founders.chunks(32).enumerate() {
        let holder: [u8; 32] = f.try_into().unwrap();
        if holder == [0u8; 32] {
            return Err(PoleisError::InvalidHolder.into());
        }
        Registry::set_pentagon(
            registry,
            k,
            &Pentagon { holder, kind: crate::HOLDER_KEY, seated_slot: now, term_end: 0 },
        )?;
    }
    Ok(())
}

/// Create a child's registry and treasury, seat its founders, and bind it
/// into the parent's chair. The child inherits nothing but its parent's
/// name; its plan, window and charter are its own.
pub fn found_unit(
    payer: &AccountView,
    parent_registry: &mut AccountView,
    child_registry: &mut AccountView,
    child_treasury: &mut AccountView,
    parent_unit: u64,
    p: &FoundParams,
    now: u64,
) -> ProgramResult {
    let parent = super::read_registry(parent_registry, parent_unit)?;
    if parent.frozen {
        return Err(PoleisError::Frozen.into());
    }
    if p.child_unit == parent_unit || p.child_unit == parent.root {
        return Err(PoleisError::UnitMismatch.into());
    }
    // unit 0 is only ever a root: the gates that ask `origin == ROOT_UNIT`
    // (the code's upgrade, the fund's constitution and vetting) take it for
    // the root of the founding tree, and on a program id where tree 0 is not
    // founded yet a stranger's tree could otherwise found a child 0 and pass them
    if p.child_unit == crate::ROOT_UNIT {
        return Err(PoleisError::UnitMismatch.into());
    }
    if p.chair as usize >= crate::PENTAGONS {
        return Err(PoleisError::InvalidHolder.into());
    }
    if Registry::pentagon(parent_registry, p.chair as usize)?.kind != crate::HOLDER_EMPTY {
        return Err(PoleisError::ChairTaken.into());
    }
    if parent.level >= crate::MAX_DEPTH {
        return Err(PoleisError::NotDescendant.into());
    }
    if !(crate::MIN_WINDOW_SLOTS..=crate::MAX_WINDOW_SLOTS).contains(&p.window) {
        return Err(PoleisError::InvalidWindow.into());
    }

    let child_le = p.child_unit.to_le_bytes();
    let t_seeds: &[&[u8]] = &[b"treasury", &child_le];
    let t_bump = find_and_expect_pda(child_treasury, t_seeds)?;
    create_pda(payer, child_treasury, Treasury::SIZE, t_seeds, t_bump)?;
    Treasury::init(child_treasury, p.child_unit, t_bump)?;

    let r_seeds: &[&[u8]] = &[b"registry", &child_le];
    let r_bump = find_and_expect_pda(child_registry, r_seeds)?;
    create_pda(payer, child_registry, Registry::MIN_SIZE, r_seeds, r_bump)?;
    Registry::init(
        child_registry,
        p.child_unit,
        parent_unit,
        p.chair,
        parent.level + 1,
        &p.charter,
        p.window,
        p.plan_cap,
        p.plan_period,
        now,
        r_bump,
        &parent.default_mint,
        parent.root,
    )?;
    seat_founders(child_registry, p.founders, now)?;
    Registry::set_name(child_registry, p.name, p.desc)?;

    Registry::set_pentagon(
        parent_registry,
        p.chair as usize,
        &Pentagon { holder: unit_holder(p.child_unit), kind: crate::HOLDER_UNIT, seated_slot: now, term_end: 0 },
    )
}

/// data: [charter:32][plan_cap:8][plan_period:8][window:8][rung:1][reserve:8]
///       [deposit:8][commit_end:8][reveal_end:8][n:1][founders: n × 32]
///       [name_len:1][name][desc_len:1][desc]
/// accounts: [payer (s,w), config (w), root treasury (w), root registry (w), sale (w), system]
/// Phase one. The payer writes the charter and the bylaws, seats the founders
/// in the chairs, and opens the first hexagon sale at a rung of the ladder.
/// The root's treasury is born here, so founding is init_config and this,
/// in one transaction. The tree is frozen from this block until ratification.
pub fn open_genesis(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [payer, config, treasury, registry, sale, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if data.len() < 92 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let d = Data(data);
    let charter = d.h32(0)?;
    let plan_cap = d.u64(32)?;
    let plan_period = d.u64(40)?;
    let window = d.u64(48)?;
    let rung = data[56] as usize;
    let reserve = d.u64(57)?;
    let deposit = d.u64(65)?;
    let commit_end = d.u64(73)?;
    let reveal_end = d.u64(81)?;
    let n = data[89] as usize;
    if n > crate::PENTAGONS || data.len() < 90 + n * 32 + 2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let end = 90 + n * 32;
    let founders = &data[90..end];
    let (name, used) = parse_text(&data[end..], crate::MAX_NAME)?;
    let (desc, used2) = parse_text(&data[end + used..], crate::MAX_DESC)?;
    if data.len() != end + used + used2 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut cfg = super::expect_solo_authority(config, payer)?;
    expect_writable(config)?;
    if rung == 0 || rung >= crate::LADDER.len() {
        return Err(PoleisError::NotOnLadder.into());
    }
    let seats = crate::ladder_hexagons(rung);
    if !(crate::MIN_WINDOW_SLOTS..=crate::MAX_WINDOW_SLOTS).contains(&window) {
        return Err(PoleisError::InvalidWindow.into());
    }
    let now = Clock::get()?.slot;
    check_windows(now, commit_end, reveal_end)?;

    // the root is the id the config was seeded with; its treasury is born
    // here, so founding is two instructions in one transaction
    let root = cfg.root;
    let root_le = root.to_le_bytes();
    let t_seeds: &[&[u8]] = &[b"treasury", &root_le];
    let t_bump = find_and_expect_pda(treasury, t_seeds)?;
    create_pda(payer, treasury, Treasury::SIZE, t_seeds, t_bump)?;
    Treasury::init(treasury, root, t_bump)?;

    let seeds: &[&[u8]] = &[b"registry", &root_le];
    let bump = find_and_expect_pda(registry, seeds)?;
    create_pda(payer, registry, Registry::MIN_SIZE, seeds, bump)?;
    Registry::init(
        registry,
        root,
        root,
        crate::NO_CHAIR,
        0,
        &charter,
        window,
        plan_cap,
        plan_period,
        now,
        bump,
        &cfg.default_mint,
        root,
    )?;
    let mut head = Registry::head(registry)?;
    head.rung = rung as u8;
    head.next_sale_id = 1;
    head.seat_reserve = reserve;
    head.seat_deposit = deposit;
    Registry::set_head(registry, &head)?;
    seat_founders(registry, founders, now)?;
    Registry::set_name(registry, name, desc)?;

    open_sale(
        payer,
        sale,
        SaleHead {
            unit_id: root,
            sale_id: 0,
            seat_count: seats,
            reserve,
            deposit,
            commit_end_slot: commit_end,
            reveal_end_slot: reveal_end,
            clearing: 0,
            overflow_price: 0,
            settled: false,
            filled: 0,
            bump: 0,
        },
    )?;

    cfg.mode = crate::MODE_GENESIS;
    cfg.write(config)
}

/// data: FoundParams
/// accounts: [payer (s,w), config, parent registry (w), child registry (w),
///            child treasury (w), system]
/// The founding tree: before ratification the payer may hang children on
/// chairs with the exact payload a council later uses through CODEX.
pub fn found_child(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [payer, config, parent_registry, child_registry, child_treasury, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let p = FoundParams::parse(data)?;
    expect_signer(payer)?;
    let cfg = super::read_config(config)?;
    if cfg.mode != crate::MODE_GENESIS || payer.address() != &cfg.authority {
        return Err(PoleisError::Unauthorized.into());
    }
    let parent = Registry::head(parent_registry)?;
    if parent.root != cfg.root {
        return Err(PoleisError::UnitMismatch.into());
    }
    found_unit(payer, parent_registry, child_registry, child_treasury, parent.unit_id, &p, Clock::get()?.slot)
}

/// data: [] · accounts: [crank (s), config (w), root registry, genesis sale]
/// The constitutional moment, permissionless so the payer cannot withhold
/// it: once the genesis sale is settled and at least one seat is claimed,
/// the authority becomes a PDA nobody holds. One-way by construction.
pub fn ratify_genesis(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, config, registry, sale] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Data(data).expect_len(0)?;
    expect_signer(crank)?;
    expect_writable(config)?;

    let mut cfg = super::read_config(config)?;
    if cfg.mode != crate::MODE_GENESIS {
        return Err(PoleisError::InvalidMode.into());
    }
    let root = cfg.root;
    let root_le = root.to_le_bytes();
    let head = super::read_registry(registry, root)?;
    let s = crate::state::SeatSale::head(sale)?;
    if s.unit_id != root || s.sale_id != 0 {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(sale, &[b"sale", &root_le, &0u64.to_le_bytes()], s.bump)?;
    if !s.settled {
        return Err(PoleisError::SaleNotSettled.into());
    }
    if head.total_seats == 0 {
        return Err(PoleisError::NotHolder.into());
    }

    // the authority nobody holds, one per tree
    let (council, _) = Address::find_program_address(&[b"council", &root_le], &crate::ID);
    cfg.authority = council;
    cfg.mode = crate::MODE_COUNCIL;
    cfg.write(config)
}

/// data: [unit:8][to:32][to_kind:1][count:8] · accounts: [holder (s,w), registry (w), system]
/// Property you cannot sell is not property: a key holder moves seats to
/// any key or unit. The recipient becomes active now and weighs one.
/// Seats rest: a holder heard from within the last window (a vote, a
/// claim, a receipt of seats) cannot move seats until the window has
/// passed, so no seat votes twice by changing hands mid-session. The roster
/// a session counts is, in effect, frozen for one window.
pub fn transfer_seats(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [holder, registry, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(49)?;
    let unit_id = d.u64(0)?;
    let to = d.h32(8)?;
    let to_kind = data[40];
    let count = d.u64(41)?;
    expect_signer(holder)?;
    expect_writable(registry)?;
    let head = super::read_registry(registry, unit_id)?;
    if head.frozen {
        return Err(PoleisError::Frozen.into());
    }
    if count == 0 {
        return Err(PoleisError::InvalidAmount.into());
    }
    if to_kind != crate::HOLDER_KEY && to_kind != crate::HOLDER_UNIT {
        return Err(PoleisError::InvalidHolder.into());
    }
    let from: [u8; 32] = *holder.address().as_array();
    let now = Clock::get()?.slot;
    move_seats(registry, holder, &from, &to, to_kind, count, head.window, now)
}

/// Seats leave a key holder for any key or unit, by transfer or by sale.
/// The mover must hold them and must not be resting; the recipient becomes
/// active now and weighs one if new, its row's rent paid by `payer`.
#[allow(clippy::too_many_arguments)]
pub fn move_seats(
    registry: &mut AccountView,
    payer: &AccountView,
    from: &[u8; 32],
    to: &[u8; 32],
    to_kind: u8,
    count: u64,
    window: u64,
    now: u64,
) -> ProgramResult {
    if to_kind != crate::HOLDER_KEY && to_kind != crate::HOLDER_UNIT {
        return Err(PoleisError::InvalidHolder.into());
    }
    if to_kind == crate::HOLDER_KEY && to == from {
        return Err(PoleisError::InvalidHolder.into());
    }
    if to_kind == crate::HOLDER_UNIT && crate::state::holder_unit(to) == Registry::head(registry)?.unit_id {
        return Err(PoleisError::InvalidHolder.into());
    }
    let i = Registry::find_holder(registry, crate::HOLDER_KEY, from)?.ok_or(PoleisError::NotHolder)?;
    let mut h = Registry::holder(registry, i)?;
    if h.seats < count {
        return Err(PoleisError::SeatLimit.into());
    }
    if now < h.last_active.saturating_add(window) {
        return Err(PoleisError::SeatsResting.into());
    }
    h.seats -= count;
    Registry::set_holder(registry, i, &h)?;
    let mut head = Registry::head(registry)?;
    head.total_seats = head.total_seats.checked_sub(count).ok_or(PoleisError::MathOverflow)?;
    Registry::set_head(registry, &head)?;
    if h.seats == 0 {
        Registry::remove_holder(registry, i)?;
    }
    Registry::add_seats(registry, payer, to_kind, to, count, now)
}
