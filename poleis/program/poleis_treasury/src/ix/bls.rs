//! The aggregate vote. His standing order (Sept 1 2026 23:39 WEST, "keep bls
//! stuff in backlog mentioning that when it's switched we'll switch"; Sept 3
//! 01:17, "keep as a priority to go into BLS as soon as Solana has it"): the
//! BLS12-381 syscalls switched on at mainnet epoch 986 (June 12 2026), and
//! this is the switch. A holder registers one BLS key per unit with a proof
//! of possession; every voter of a side signs the same envelope `attest`
//! already verifies through the Ed25519 precompile; the signatures add up
//! to one, the keys add up to one, and one pairing verifies the whole side.
//! A batch of n voices then costs one transaction signature, not n + 1:
//! the base fee of a vote falls from 5,000 lamports to 5,000 / n.

use crate::bls::{self, FLAGS, FQ_LEN, G1_LEN, G2_LEN};
use crate::error::PoleisError;
use crate::state::BlsKey;
use crate::util::{create_pda, envelope, expect_pda, expect_signer, expect_writable, find_and_expect_pda, Data};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, ProgramResult,
};

/// data: [unit:8][pk:192][ctr:4][y:48][pop:96]
/// accounts: [holder (s,w), registry, key (w), system]
/// A member of the unit (a hexagon row or a live chair) registers its
/// BLS key, or rotates it: `pk` an uncompressed big-endian G2 point, `pop`
/// its signature over the point of (unit ‖ holder ‖ pk), named by `ctr`
/// and `y` as every message point is. The pairing that checks the proof
/// also validates the key (field, curve, subgroup), so a key in the sum
/// of a later aggregate was never unchecked.
pub fn register_bls_key(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [holder, registry, key, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8 + G2_LEN + 4 + FQ_LEN + G1_LEN)?;
    let unit_id = d.u64(0)?;
    let pk: [u8; G2_LEN] = data[8..8 + G2_LEN].try_into().unwrap();
    let o = 8 + G2_LEN;
    let ctr = u32::from_le_bytes(data[o..o + 4].try_into().unwrap());
    let y: [u8; FQ_LEN] = data[o + 4..o + 4 + FQ_LEN].try_into().unwrap();
    let pop: [u8; G1_LEN] = data[o + 4 + FQ_LEN..].try_into().unwrap();
    expect_signer(holder)?;
    expect_writable(key)?;
    super::read_registry(registry, unit_id)?;
    let now = Clock::get()?.slot;
    super::expect_member(registry, holder, now)?;
    if pk[0] & FLAGS != 0 || pop[0] & FLAGS != 0 {
        return Err(PoleisError::BadBlsPoint.into());
    }
    let who: [u8; 32] = *holder.address().as_array();
    let unit_le = unit_id.to_le_bytes();
    let h = bls::hash_point(bls::POP_DST, &[&unit_le, &who, &pk], ctr, &y)?;
    bls::verify(&h, &pk, &pop)?;
    let seeds: [&[u8]; 3] = [b"blskey", &unit_le, &who];
    let bump = if key.data_len() == 0 {
        let bump = find_and_expect_pda(key, &seeds)?;
        create_pda(holder, key, BlsKey::SIZE, &seeds, bump)?;
        bump
    } else {
        let k = BlsKey::read(key)?;
        if k.unit_id != unit_id || k.holder != who {
            return Err(PoleisError::InvalidAccount.into());
        }
        expect_pda(key, &seeds, k.bump)?;
        k.bump
    };
    BlsKey { unit_id, holder: who, pk, bump }.write(key)
}

/// data: [hash:32][unit:8][choice:1][ctr:4][y:48][sig:96]
/// accounts: [crank (s,w), registry (w), session (w), treasury (w), key₁ … keyₙ]
/// One aggregate signature over one envelope for every key passed: the
/// keys add up, one pairing verifies the sum against the message point
/// the crank names, then each key's holder lands exactly as a voice of
/// `attest` lands (its row once, its chairs once); a holder already
/// counted adds nothing, and the crank's tip follows a batch that landed.
pub fn attest_bls(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, registry, session, treasury, keys @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(32 + 8 + 1 + 4 + FQ_LEN + G1_LEN)?;
    let hash = d.h32(0)?;
    let unit_id = d.u64(32)?;
    let choice = data[40];
    if choice > crate::VOTE_CONTRA || keys.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let ctr = u32::from_le_bytes(data[41..45].try_into().unwrap());
    let y: [u8; FQ_LEN] = data[45..45 + FQ_LEN].try_into().unwrap();
    let sig: [u8; G1_LEN] = data[45 + FQ_LEN..].try_into().unwrap();
    expect_signer(crank)?;
    expect_writable(registry)?;
    expect_writable(session)?;

    let mut s = super::proposal::read_session(session, &hash, unit_id)?;
    if s.terminal() {
        return Err(PoleisError::SessionClosed.into());
    }
    let now = Clock::get()?.slot;
    if now > s.deadline {
        return Err(PoleisError::SessionClosed.into());
    }
    super::read_registry(registry, unit_id)?;

    // the keys add up
    let mut agg = [0u8; G2_LEN];
    for (i, key) in keys.iter().enumerate() {
        let k = BlsKey::read(key)?;
        if k.unit_id != unit_id {
            return Err(PoleisError::UnitMismatch.into());
        }
        agg = if i == 0 { k.pk } else { bls::g2_add(&agg, &k.pk).ok_or(PoleisError::BadBlsPoint)? };
    }
    // one message, one pairing
    let env = envelope(&hash, unit_id, choice);
    let h = bls::hash_point(bls::VOTE_DST, &[&env], ctr, &y)?;
    bls::verify(&h, &agg, &sig)?;

    // every signer lands as a voice of attest would
    let speakers = super::proposal::speakers(session, registry)?;
    let mut any_landed = false;
    for key in keys.iter() {
        let k = BlsKey::read(key)?;
        if super::proposal::land(&mut s, session, registry, &k.holder, choice, now, speakers)? {
            any_landed = true;
        }
    }
    s.write(session)?;
    if any_landed {
        super::tip(registry, treasury, crank, unit_id, &hash, now)?;
    }
    Ok(())
}
