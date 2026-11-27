//! Council-governed upgrades. The program's upgrade authority is a PDA of
//! the program itself (`"upgrade"`, never created, never signed for except
//! here), so no key outranks the seal: only a CODEX act of the root over
//! the whole tree makes the loader replace the code from a buffer, hand the
//! authority to another key, or burn it. The same PDA holds every program
//! handed to the seal (the market, the $WARM program), so every act that
//! reaches a program (its code, its authority) names it, and only that
//! program and its own data are reached.
//! A buffer the seal holds (an upgrade's that was never carried out) is
//! closed by an act of its own, which names the buffer and reaches no program.

use crate::error::PoleisError;
use crate::util::find_and_expect_pda;
use pinocchio::{
    cpi::{invoke_signed_with_bounds, Seed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    sysvars::{clock::CLOCK_ID, rent::RENT_ID},
    AccountView, Address, ProgramResult,
};

fn expect_loader(acc: &AccountView) -> Result<(), ProgramError> {
    if acc.address() != &crate::BPF_LOADER_UPGRADEABLE_ID {
        return Err(PoleisError::InvalidAccount.into());
    }
    Ok(())
}

/// The upgrade authority the deploy ceremony hands the program to.
pub fn expect_upgrade_authority(acc: &AccountView) -> Result<u8, ProgramError> {
    find_and_expect_pda(acc, &[b"upgrade"])
}

/// The loader's program data of `program` and of no other: the address the loader itself derives for it (`[program]` under the
/// upgradeable loader). An act that moves an upgrade authority names its program; this is where the name binds what its carrier hands in.
pub fn expect_programdata_of(programdata: &AccountView, program: &[u8; 32]) -> ProgramResult {
    let (derived, _) = Address::find_program_address(&[program], &crate::BPF_LOADER_UPGRADEABLE_ID);
    if programdata.address() != &derived {
        return Err(PoleisError::InvalidAccount.into());
    }
    Ok(())
}

/// ONLY THE DEPLOYER FOUNDS ROOT 0: Poleis's own program data ([3:4][slot:8][1 and the authority:33]) names `founder` as its upgrade
/// authority. A program data whose upgrades were burned names none, and founds nothing. Nothing at that address means no upgradeable
/// loader deployed this Poleis (a test's), and asks nothing: on a cluster the loader lays it in the deploy itself, and no one else can.
pub fn expect_founder(program_data: &AccountView, founder: &AccountView) -> ProgramResult {
    expect_programdata_of(program_data, crate::ID.as_array())?;
    if !program_data.owned_by(&crate::BPF_LOADER_UPGRADEABLE_ID) {
        if program_data.lamports() == 0 && program_data.data_len() == 0 {
            return Ok(());
        }
        return Err(PoleisError::InvalidAccount.into());
    }
    let d = program_data.try_borrow()?;
    if d.len() < 45 || d[..4] != [3, 0, 0, 0] {
        return Err(PoleisError::InvalidAccount.into());
    }
    if d[12] != 1 || &d[13..45] != founder.address().as_ref() {
        return Err(PoleisError::Unauthorized.into());
    }
    Ok(())
}

/// A buffer of the loader's and nothing else: owned by the loader, its state `Buffer` (tag 1; a program is 2, a program's data 3).
fn expect_buffer(acc: &AccountView) -> ProgramResult {
    if !acc.owned_by(&crate::BPF_LOADER_UPGRADEABLE_ID) {
        return Err(PoleisError::InvalidAccount.into());
    }
    let d = acc.try_borrow()?;
    if d.len() < 4 || d[..4] != [1, 0, 0, 0] {
        return Err(PoleisError::InvalidAccount.into());
    }
    Ok(())
}

/// Loader `Close`: [5,0,0,0]; accounts [buffer (w), recipient (w), authority (s)], and never the fourth (a program) by which the loader
/// closes a program's data: what is closed here is a buffer the seal holds, its lamports to the recipient.
pub fn close_buffer(buffer: &AccountView, recipient: &AccountView, authority: &AccountView, loader: &AccountView) -> ProgramResult {
    expect_loader(loader)?;
    expect_buffer(buffer)?;
    let bump = expect_upgrade_authority(authority)?;
    let data = [5u8, 0, 0, 0];
    let metas = [
        InstructionAccount::writable(buffer.address()),
        InstructionAccount::writable(recipient.address()),
        InstructionAccount::readonly_signer(authority.address()),
    ];
    let ix = InstructionView { program_id: loader.address(), accounts: &metas, data: &data };
    let bump_ref = [bump];
    let seeds = [Seed::from(&b"upgrade"[..]), Seed::from(&bump_ref[..])];
    let views: [&AccountView; 3] = [buffer, recipient, authority];
    invoke_signed_with_bounds::<4, &AccountView>(&ix, &views, &[Signer::from(&seeds[..])])
}

/// Loader `Upgrade`: [3,0,0,0]; accounts [programdata (w), program (w),
/// buffer (w), spill (w), rent, clock, authority (s)].
#[allow(clippy::too_many_arguments)]
pub fn upgrade(
    programdata: &AccountView,
    program: &AccountView,
    buffer: &AccountView,
    spill: &AccountView,
    rent: &AccountView,
    clock: &AccountView,
    authority: &AccountView,
    loader: &AccountView,
) -> ProgramResult {
    expect_loader(loader)?;
    if rent.address() != &RENT_ID || clock.address() != &CLOCK_ID {
        return Err(PoleisError::InvalidAccount.into());
    }
    let bump = expect_upgrade_authority(authority)?;
    let data = [3u8, 0, 0, 0];
    let metas = [
        InstructionAccount::writable(programdata.address()),
        InstructionAccount::writable(program.address()),
        InstructionAccount::writable(buffer.address()),
        InstructionAccount::writable(spill.address()),
        InstructionAccount::readonly(rent.address()),
        InstructionAccount::readonly(clock.address()),
        InstructionAccount::readonly_signer(authority.address()),
    ];
    let ix = InstructionView { program_id: loader.address(), accounts: &metas, data: &data };
    let bump_ref = [bump];
    let seeds = [Seed::from(&b"upgrade"[..]), Seed::from(&bump_ref[..])];
    let views: [&AccountView; 7] = [programdata, program, buffer, spill, rent, clock, authority];
    invoke_signed_with_bounds::<8, &AccountView>(&ix, &views, &[Signer::from(&seeds[..])])
}

/// Loader `SetAuthority`: [4,0,0,0]; accounts [programdata (w), authority (s),
/// new authority]; no third account burns the authority for good.
pub fn set_authority(
    programdata: &AccountView,
    authority: &AccountView,
    loader: &AccountView,
    new: Option<&AccountView>,
) -> ProgramResult {
    expect_loader(loader)?;
    let bump = expect_upgrade_authority(authority)?;
    let data = [4u8, 0, 0, 0];
    let bump_ref = [bump];
    let seeds = [Seed::from(&b"upgrade"[..]), Seed::from(&bump_ref[..])];
    match new {
        Some(n) => {
            let metas = [
                InstructionAccount::writable(programdata.address()),
                InstructionAccount::readonly_signer(authority.address()),
                InstructionAccount::readonly(n.address()),
            ];
            let ix = InstructionView { program_id: loader.address(), accounts: &metas, data: &data };
            let views: [&AccountView; 3] = [programdata, authority, n];
            invoke_signed_with_bounds::<4, &AccountView>(&ix, &views, &[Signer::from(&seeds[..])])
        }
        None => {
            let metas = [
                InstructionAccount::writable(programdata.address()),
                InstructionAccount::readonly_signer(authority.address()),
            ];
            let ix = InstructionView { program_id: loader.address(), accounts: &metas, data: &data };
            let views: [&AccountView; 2] = [programdata, authority];
            invoke_signed_with_bounds::<4, &AccountView>(&ix, &views, &[Signer::from(&seeds[..])])
        }
    }
}
