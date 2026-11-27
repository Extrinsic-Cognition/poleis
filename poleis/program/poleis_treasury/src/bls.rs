//! BLS12-381 for the aggregate vote, through the runtime's curve syscalls
//! (SIMD-0388, live on mainnet since epoch 986, June 12 2026). Keys live
//! in G2, signatures and the message point in G1 (the "min-sig" shape):
//! the message point is the one thing hashed on chain, and G1 is where
//! that is cheap.
//!
//! The message point is found by try-and-increment, the one hash-to-curve
//! the syscalls can carry: no map-to-curve, no field inversion, no square
//! root runs in BPF. For a counter `ctr`, `x` is the first 48 bytes of
//! sha256(h ‖ 0) ‖ sha256(h ‖ 1), h = sha256(dst ‖ msg ‖ ctr_le), its top
//! three bits cleared. The signer takes the first counter whose `x` is
//! below `p` and has a root `y` for `x³ + 4`, the smaller root
//! (`y ≤ (p − 1) / 2`), and clears the cofactor by
//! `h_eff = 1 − z = 0xd201000000010001`. On chain the crank hands over
//! `ctr` and `y`: the program recomputes `x`, checks the two bounds, and
//! rebuilds `h_eff · (x, y)` by a double-and-add chain of G1 additions,
//! whose first step refuses a point off the curve (the ADD syscall checks
//! field and curve, never the subgroup, which is what lets a raw curve
//! point through). Any counter whose `x` lies on the curve names a random
//! point of the curve, so the verifier accepts whichever the crank names:
//! every signer of a side signs the same one, and the smallest wins.
//!
//! Verification is one pairing of two pairs, e(H, pk) · e(−S, G2) == 1,
//! with `pk` the sum of the signers' keys (G2 additions, no subgroup check:
//! every registered key passed its own pairing at registration, against a
//! proof of possession, so no rogue key is in the sum) and `−S` the
//! aggregate signature with `y ↦ p − y`.
//!
//! Costs (the runtime's table, Agave 4.2): a G1 addition 128 CU, a G2
//! addition 203 CU, a pairing of two pairs 25,445 + 13,023 CU. The chain is
//! 63 doublings and 6 additions, 8,832 CU; n keys add up in 203 · (n − 1).

use crate::error::PoleisError;
use pinocchio::error::ProgramError;

pub const FQ_LEN: usize = 48;
pub const G1_LEN: usize = 96;
pub const G2_LEN: usize = 192;
pub const GT_LEN: usize = 576;
/// The curve ids of SIMD-0388, big-endian (the Zcash encoding).
pub const CURVE_PAIRING_BE: u64 = 4 | 0x80;
pub const CURVE_G1_BE: u64 = 5 | 0x80;
pub const CURVE_G2_BE: u64 = 6 | 0x80;
pub const OP_ADD: u64 = 0;
/// The effective cofactor of G1: 1 − z.
pub const H_EFF: u64 = 0xd201_0000_0001_0001;
/// Domain separation: the vote envelope, and the proof of possession.
pub const VOTE_DST: &[u8] = b"poleis:bls:vote:v1";
pub const POP_DST: &[u8] = b"poleis:bls:pop:v1";
/// The flags a compressed or infinite encoding would carry in its first
/// byte; an uncompressed finite point carries none.
pub const FLAGS: u8 = 0xe0;

/// The base field prime, big-endian.
pub const P: [u8; FQ_LEN] = [
    0x1a, 0x01, 0x11, 0xea, 0x39, 0x7f, 0xe6, 0x9a, 0x4b, 0x1b, 0xa7, 0xb6,
    0x43, 0x4b, 0xac, 0xd7, 0x64, 0x77, 0x4b, 0x84, 0xf3, 0x85, 0x12, 0xbf,
    0x67, 0x30, 0xd2, 0xa0, 0xf6, 0xb0, 0xf6, 0x24, 0x1e, 0xab, 0xff, 0xfe,
    0xb1, 0x53, 0xff, 0xff, 0xb9, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xaa, 0xab,
];
/// (p − 1) / 2: of a pair of roots, the larger one exceeds it.
pub const P_MINUS_1_HALF: [u8; FQ_LEN] = [
    0x0d, 0x00, 0x88, 0xf5, 0x1c, 0xbf, 0xf3, 0x4d, 0x25, 0x8d, 0xd3, 0xdb,
    0x21, 0xa5, 0xd6, 0x6b, 0xb2, 0x3b, 0xa5, 0xc2, 0x79, 0xc2, 0x89, 0x5f,
    0xb3, 0x98, 0x69, 0x50, 0x7b, 0x58, 0x7b, 0x12, 0x0f, 0x55, 0xff, 0xff,
    0x58, 0xa9, 0xff, 0xff, 0xdc, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xd5, 0x55,
];
/// The G2 generator, uncompressed big-endian (x₁ ‖ x₀ ‖ y₁ ‖ y₀).
pub const G2_GENERATOR: [u8; G2_LEN] = [
    0x13, 0xe0, 0x2b, 0x60, 0x52, 0x71, 0x9f, 0x60, 0x7d, 0xac, 0xd3, 0xa0,
    0x88, 0x27, 0x4f, 0x65, 0x59, 0x6b, 0xd0, 0xd0, 0x99, 0x20, 0xb6, 0x1a,
    0xb5, 0xda, 0x61, 0xbb, 0xdc, 0x7f, 0x50, 0x49, 0x33, 0x4c, 0xf1, 0x12,
    0x13, 0x94, 0x5d, 0x57, 0xe5, 0xac, 0x7d, 0x05, 0x5d, 0x04, 0x2b, 0x7e,
    0x02, 0x4a, 0xa2, 0xb2, 0xf0, 0x8f, 0x0a, 0x91, 0x26, 0x08, 0x05, 0x27,
    0x2d, 0xc5, 0x10, 0x51, 0xc6, 0xe4, 0x7a, 0xd4, 0xfa, 0x40, 0x3b, 0x02,
    0xb4, 0x51, 0x0b, 0x64, 0x7a, 0xe3, 0xd1, 0x77, 0x0b, 0xac, 0x03, 0x26,
    0xa8, 0x05, 0xbb, 0xef, 0xd4, 0x80, 0x56, 0xc8, 0xc1, 0x21, 0xbd, 0xb8,
    0x06, 0x06, 0xc4, 0xa0, 0x2e, 0xa7, 0x34, 0xcc, 0x32, 0xac, 0xd2, 0xb0,
    0x2b, 0xc2, 0x8b, 0x99, 0xcb, 0x3e, 0x28, 0x7e, 0x85, 0xa7, 0x63, 0xaf,
    0x26, 0x74, 0x92, 0xab, 0x57, 0x2e, 0x99, 0xab, 0x3f, 0x37, 0x0d, 0x27,
    0x5c, 0xec, 0x1d, 0xa1, 0xaa, 0xa9, 0x07, 0x5f, 0xf0, 0x5f, 0x79, 0xbe,
    0x0c, 0xe5, 0xd5, 0x27, 0x72, 0x7d, 0x6e, 0x11, 0x8c, 0xc9, 0xcd, 0xc6,
    0xda, 0x2e, 0x35, 0x1a, 0xad, 0xfd, 0x9b, 0xaa, 0x8c, 0xbd, 0xd3, 0xa7,
    0x6d, 0x42, 0x9a, 0x69, 0x51, 0x60, 0xd1, 0x2c, 0x92, 0x3a, 0xc9, 0xcc,
    0x3b, 0xac, 0xa2, 0x89, 0xe1, 0x93, 0x54, 0x86, 0x08, 0xb8, 0x28, 0x01,
];

/// `a < b`, both big-endian.
pub fn lt(a: &[u8; FQ_LEN], b: &[u8; FQ_LEN]) -> bool {
    for i in 0..FQ_LEN {
        if a[i] != b[i] {
            return a[i] < b[i];
        }
    }
    false
}

/// `y ↦ p − y`, big-endian, for `0 < y < p`; anything else leaves a value
/// at or past `p`, which the syscall then refuses as a field element.
pub fn negate(y: &mut [u8; FQ_LEN]) {
    let mut borrow = 0i32;
    for i in (0..FQ_LEN).rev() {
        let d = P[i] as i32 - y[i] as i32 - borrow;
        if d < 0 {
            y[i] = (d + 256) as u8;
            borrow = 1;
        } else {
            y[i] = d as u8;
            borrow = 0;
        }
    }
}

/// The `x` candidate of (dst, msg, ctr).
pub fn x_of(dst: &[u8], msg: &[&[u8]], ctr: u32) -> Result<[u8; FQ_LEN], ProgramError> {
    let ctr_le = ctr.to_le_bytes();
    let mut parts: [&[u8]; 8] = [&[]; 8];
    if msg.len() + 2 > parts.len() {
        return Err(ProgramError::InvalidInstructionData);
    }
    parts[0] = dst;
    for (i, m) in msg.iter().enumerate() {
        parts[i + 1] = *m;
    }
    parts[msg.len() + 1] = &ctr_le;
    let h = crate::util::sha256v(&parts[..msg.len() + 2]);
    let a = crate::util::sha256v(&[&h, &[0u8]]);
    let b = crate::util::sha256v(&[&h, &[1u8]]);
    let mut x = [0u8; FQ_LEN];
    x[..32].copy_from_slice(&a);
    x[32..].copy_from_slice(&b[..16]);
    x[0] &= 0x1f;
    Ok(x)
}

/// The message point `h_eff · (x, y)` for the counter and root the crank
/// names: refused unless `x < p`, `y ≤ (p − 1) / 2` and `(x, y)` is on the
/// curve.
pub fn hash_point(dst: &[u8], msg: &[&[u8]], ctr: u32, y: &[u8; FQ_LEN]) -> Result<[u8; G1_LEN], ProgramError> {
    let x = x_of(dst, msg, ctr)?;
    if !lt(&x, &P) || lt(&P_MINUS_1_HALF, y) {
        return Err(PoleisError::BadBlsPoint.into());
    }
    let mut p0 = [0u8; G1_LEN];
    p0[..FQ_LEN].copy_from_slice(&x);
    p0[FQ_LEN..].copy_from_slice(y);
    clear_cofactor(&p0).ok_or_else(|| PoleisError::BadBlsPoint.into())
}

/// `h_eff · p0` by double-and-add over G1 additions, the leading bit of
/// `h_eff` first: 63 doublings and one addition per further set bit.
pub fn clear_cofactor(p0: &[u8; G1_LEN]) -> Option<[u8; G1_LEN]> {
    clear_cofactor_by(p0, g1_add)
}

/// The same chain over any addition: the syscall on chain, the runtime's
/// own implementation in the host tests.
pub fn clear_cofactor_by(p0: &[u8; G1_LEN], add: fn(&[u8; G1_LEN], &[u8; G1_LEN]) -> Option<[u8; G1_LEN]>) -> Option<[u8; G1_LEN]> {
    let mut acc = *p0;
    let mut bit = 62i32;
    while bit >= 0 {
        acc = add(&acc, &acc)?;
        if (H_EFF >> bit) & 1 == 1 {
            acc = add(&acc, p0)?;
        }
        bit -= 1;
    }
    Some(acc)
}

/// e(h, pk) · e(−sig, G2) == 1: one syscall of two pairs, the result read
/// against the identity of the target group (575 zero bytes, then one).
pub fn verify(h: &[u8; G1_LEN], pk: &[u8; G2_LEN], sig: &[u8; G1_LEN]) -> Result<(), ProgramError> {
    if sig[0] & FLAGS != 0 || pk[0] & FLAGS != 0 {
        return Err(PoleisError::BadBlsPoint.into());
    }
    let mut neg = *sig;
    let mut y = [0u8; FQ_LEN];
    y.copy_from_slice(&neg[FQ_LEN..]);
    negate(&mut y);
    neg[FQ_LEN..].copy_from_slice(&y);
    let mut g1 = [0u8; 2 * G1_LEN];
    g1[..G1_LEN].copy_from_slice(h);
    g1[G1_LEN..].copy_from_slice(&neg);
    let mut g2 = [0u8; 2 * G2_LEN];
    g2[..G2_LEN].copy_from_slice(pk);
    g2[G2_LEN..].copy_from_slice(&G2_GENERATOR);
    let gt = pairing(&g1, &g2, 2).ok_or(PoleisError::BadBlsSignature)?;
    if gt[..GT_LEN - 1].iter().any(|b| *b != 0) || gt[GT_LEN - 1] != 1 {
        return Err(PoleisError::BadBlsSignature.into());
    }
    Ok(())
}

/// `a + b` in G1: the syscall checks field and curve, not the subgroup.
pub fn g1_add(a: &[u8; G1_LEN], b: &[u8; G1_LEN]) -> Option<[u8; G1_LEN]> {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    {
        let mut out = [0u8; G1_LEN];
        let r = unsafe {
            pinocchio::syscalls::sol_curve_group_op(CURVE_G1_BE, OP_ADD, a.as_ptr(), b.as_ptr(), out.as_mut_ptr())
        };
        if r == 0 {
            Some(out)
        } else {
            None
        }
    }
    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    {
        let _ = (a, b);
        unimplemented!("the curve syscalls run on chain only")
    }
}

/// `a + b` in G2: the syscall checks field and curve, not the subgroup.
pub fn g2_add(a: &[u8; G2_LEN], b: &[u8; G2_LEN]) -> Option<[u8; G2_LEN]> {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    {
        let mut out = [0u8; G2_LEN];
        let r = unsafe {
            pinocchio::syscalls::sol_curve_group_op(CURVE_G2_BE, OP_ADD, a.as_ptr(), b.as_ptr(), out.as_mut_ptr())
        };
        if r == 0 {
            Some(out)
        } else {
            None
        }
    }
    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    {
        let _ = (a, b);
        unimplemented!("the curve syscalls run on chain only")
    }
}

/// The product of `pairs` pairings, every point fully validated by the
/// syscall (field, curve, subgroup).
fn pairing(g1: &[u8], g2: &[u8], pairs: u64) -> Option<[u8; GT_LEN]> {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    {
        let mut out = [0u8; GT_LEN];
        let r = unsafe {
            pinocchio::syscalls::sol_curve_pairing_map(CURVE_PAIRING_BE, pairs, g1.as_ptr(), g2.as_ptr(), out.as_mut_ptr())
        };
        if r == 0 {
            Some(out)
        } else {
            None
        }
    }
    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    {
        let _ = (g1, g2, pairs);
        unimplemented!("the curve syscalls run on chain only")
    }
}
