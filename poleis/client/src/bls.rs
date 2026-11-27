//! The BLS side of the client: keys, the message point, signatures and
//! their sum, the proof of possession, the two instructions, and the key
//! account's view. The law is the program's (`poleis_treasury::bls`): the
//! same counter rule for the message point, the same cofactor, the same
//! envelope `attest` verifies through the precompile. Keys in G2,
//! signatures in G1, big-endian Zcash bytes throughout.

use crate::{data, find_pda, ix, registry_pda, session_pda, system_meta, tag, treasury_pda, vote_envelope, LAYOUT_VERSION, OFF_DISC, OFF_VERSION};
use blstrs::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use ff::Field;
use group::{prime::PrimeCurveAffine, Curve, Group};
use poleis_treasury::bls::{FQ_LEN, G1_LEN, G2_LEN, H_EFF, POP_DST, VOTE_DST};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

pub use poleis_treasury::bls::{G2_GENERATOR, P};
pub use poleis_treasury::state::{BlsKey, DISC_BLS_KEY};

/// A BLS keypair: the secret a scalar of the subgroup order, the public
/// key `sk · G2`.
pub struct BlsKeypair {
    sk: Scalar,
    pub pk: G2Affine,
}

impl BlsKeypair {
    /// Derived from a seed: sha256(seed ‖ i) for the first `i` whose bytes
    /// read as a non-zero scalar below the group order.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        for i in 0u8..=255 {
            let bytes = crate::sha(&[seed, &[i]]);
            if let Some(sk) = Scalar::from_bytes_be(&bytes).into_option() {
                if !bool::from(sk.is_zero()) {
                    return Self { sk, pk: (G2Projective::generator() * sk).to_affine() };
                }
            }
        }
        unreachable!("a scalar below the order within 256 tries")
    }
    /// The key as the program stores it: uncompressed, big-endian, 192 bytes.
    pub fn pk_bytes(&self) -> [u8; G2_LEN] {
        self.pk.to_uncompressed()
    }
    /// The signature on a message point: `sk · H`.
    pub fn sign(&self, h: &G1Affine) -> G1Affine {
        (G1Projective::from(*h) * self.sk).to_affine()
    }
}

/// A message point named by its counter and root, cleared into G1.
#[derive(Clone, Copy, Debug)]
pub struct HashPoint {
    pub ctr: u32,
    pub y: [u8; FQ_LEN],
    pub point: G1Affine,
}

/// The `x` candidate of a counter, as the program computes it.
pub fn x_of(dst: &[u8], msg: &[&[u8]], ctr: u32) -> [u8; FQ_LEN] {
    let ctr_le = ctr.to_le_bytes();
    let mut parts: Vec<&[u8]> = vec![dst];
    parts.extend_from_slice(msg);
    parts.push(&ctr_le);
    let h = crate::sha(&parts);
    let a = crate::sha(&[&h, &[0u8]]);
    let b = crate::sha(&[&h, &[1u8]]);
    let mut x = [0u8; FQ_LEN];
    x[..32].copy_from_slice(&a);
    x[32..].copy_from_slice(&b[..16]);
    x[0] &= 0x1f;
    x
}

#[cfg(test)]
fn le(a: &[u8; FQ_LEN], b: &[u8; FQ_LEN]) -> bool {
    a <= b
}

/// The raw curve point of a counter, when its `x` is below the prime and
/// `x³ + 4` has a root: `x ‖ y` with the smaller root, not yet in G1. The
/// root is found by decoding `x` as a compressed point with the sign flag
/// clear, which the Zcash encoding defines as the lexicographically
/// smaller root: the library's field check, curve check and square root,
/// and no field arithmetic of this crate's own.
pub fn raw_point(dst: &[u8], msg: &[&[u8]], ctr: u32) -> Option<[u8; G1_LEN]> {
    let mut c = x_of(dst, msg, ctr);
    c[0] |= 0x80;
    let p0 = G1Affine::from_compressed_unchecked(&c).into_option()?;
    Some(p0.to_uncompressed())
}

/// The first counter whose `x` is on the curve, its smaller root, and
/// `h_eff` times the point: what every signer of a message signs.
pub fn hash_point(dst: &[u8], msg: &[&[u8]]) -> HashPoint {
    for ctr in 0u32.. {
        let Some(raw) = raw_point(dst, msg, ctr) else { continue };
        let p0 = G1Affine::from_uncompressed_unchecked(&raw).into_option().expect("on the curve by construction");
        let point = (G1Projective::from(p0) * Scalar::from(H_EFF)).to_affine();
        let mut y = [0u8; FQ_LEN];
        y.copy_from_slice(&raw[FQ_LEN..]);
        return HashPoint { ctr, y, point };
    }
    unreachable!()
}

/// The point every voter of (proposal, unit, choice) signs.
pub fn vote_point(hash: &[u8; 32], unit: u64, choice: u8) -> HashPoint {
    hash_point(VOTE_DST, &[&vote_envelope(hash, unit, choice)])
}

/// The point a holder signs to prove possession of its key in a unit.
pub fn pop_point(unit: u64, holder: &Pubkey, pk: &[u8; G2_LEN]) -> HashPoint {
    hash_point(POP_DST, &[&unit.to_le_bytes(), holder.as_ref(), pk])
}

/// The sum of signatures over one point: one signature for the side.
pub fn aggregate(sigs: &[G1Affine]) -> G1Affine {
    sigs.iter().fold(G1Projective::identity(), |a, s| a + s).to_affine()
}

/// The sum of the signers' keys, as the program adds them up.
pub fn aggregate_keys(pks: &[G2Affine]) -> G2Affine {
    pks.iter().fold(G2Projective::identity(), |a, k| a + k).to_affine()
}

/// e(h, pk) == e(sig, G2): the host's own check, by the library.
pub fn verify(h: &G1Affine, pk: &G2Affine, sig: &G1Affine) -> bool {
    blstrs::pairing(h, pk) == blstrs::pairing(sig, &G2Affine::generator())
}

/// A signature or a message point as the wire carries it: 96 bytes.
pub fn point_bytes(p: &G1Affine) -> [u8; G1_LEN] {
    p.to_uncompressed()
}

/// A holder's key account in a unit: `["blskey", unit, holder]`.
pub fn bls_key_pda(unit: u64, holder: &Pubkey) -> Pubkey {
    find_pda(&[b"blskey", &unit.to_le_bytes(), holder.as_ref()])
}

/// accounts: [holder (s,w), registry, key (w), system]
pub fn register_bls_key(holder: Pubkey, unit: u64, pk: &[u8; G2_LEN], pop: &HashPoint, pop_sig: &[u8; G1_LEN]) -> Instruction {
    ix(
        data(tag::IX_REGISTER_BLS_KEY, &[&unit.to_le_bytes(), pk, &pop.ctr.to_le_bytes(), &pop.y, pop_sig]),
        vec![
            AccountMeta::new(holder, true),
            AccountMeta::new_readonly(registry_pda(unit), false),
            AccountMeta::new(bls_key_pda(unit, &holder), false),
            system_meta(),
        ],
    )
}

/// The registration of a key, proof and all.
pub fn registration(holder: Pubkey, unit: u64, key: &BlsKeypair) -> Instruction {
    let pk = key.pk_bytes();
    let pop = pop_point(unit, &holder, &pk);
    let sig = key.sign(&pop.point);
    register_bls_key(holder, unit, &pk, &pop, &point_bytes(&sig))
}

/// accounts: [crank (s,w), registry (w), session (w), treasury (w), the signers' key accounts]
pub fn attest_bls(crank: Pubkey, hash: [u8; 32], unit: u64, choice: u8, point: &HashPoint, sig: &[u8; G1_LEN], signers: &[Pubkey]) -> Instruction {
    let mut metas = vec![
        AccountMeta::new(crank, true),
        AccountMeta::new(registry_pda(unit), false),
        AccountMeta::new(session_pda(&hash, unit), false),
        AccountMeta::new(treasury_pda(unit), false),
    ];
    for s in signers {
        metas.push(AccountMeta::new_readonly(bls_key_pda(unit, s), false));
    }
    ix(data(tag::IX_ATTEST_BLS, &[&hash, &unit.to_le_bytes(), &[choice], &point.ctr.to_le_bytes(), &point.y, sig]), metas)
}

/// The whole road for one side: every signer signs the one point, the
/// signatures add up, one instruction lands them all.
pub fn attest_bls_side(crank: Pubkey, hash: [u8; 32], unit: u64, choice: u8, signers: &[(Pubkey, &BlsKeypair)]) -> Instruction {
    let point = vote_point(&hash, unit, choice);
    let sigs: Vec<G1Affine> = signers.iter().map(|(_, k)| k.sign(&point.point)).collect();
    let keys: Vec<Pubkey> = signers.iter().map(|(p, _)| *p).collect();
    attest_bls(crank, hash, unit, choice, &point, &point_bytes(&aggregate(&sigs)), &keys)
}

/// A key account, read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlsKeyView {
    pub unit: u64,
    pub holder: Pubkey,
    pub pk: [u8; G2_LEN],
    pub bump: u8,
}
impl BlsKeyView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        if d.len() != BlsKey::SIZE || d[OFF_DISC] != DISC_BLS_KEY || d[OFF_VERSION] != LAYOUT_VERSION {
            return None;
        }
        Some(Self {
            unit: u64::from_le_bytes(d[BlsKey::OFF_UNIT..BlsKey::OFF_UNIT + 8].try_into().unwrap()),
            holder: Pubkey::new_from_array(d[BlsKey::OFF_HOLDER..BlsKey::OFF_HOLDER + 32].try_into().unwrap()),
            pk: d[BlsKey::OFF_PK..BlsKey::OFF_PK + G2_LEN].try_into().unwrap(),
            bump: d[BlsKey::OFF_BUMP],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poleis_treasury::bls::{clear_cofactor_by, negate, GT_LEN, P_MINUS_1_HALF};
    use solana_bls12_381_syscall::{
        bls12_381_g1_addition_unchecked, bls12_381_g2_addition_unchecked, bls12_381_pairing_map, Endianness, PodG1Point, PodG2Point,
        Version,
    };

    fn g1_add(a: &[u8; G1_LEN], b: &[u8; G1_LEN]) -> Option<[u8; G1_LEN]> {
        bls12_381_g1_addition_unchecked(Version::V0, &PodG1Point(*a), &PodG1Point(*b), Endianness::BE).map(|p| p.0)
    }

    #[test]
    fn the_g2_generator_constant_is_the_generator() {
        assert_eq!(G2_GENERATOR, G2Affine::generator().to_uncompressed());
        assert_eq!(P_MINUS_1_HALF[0], 0x0d);
    }

    #[test]
    fn the_message_point_is_the_smaller_root_cleared_into_the_subgroup() {
        for (i, msg) in [b"one".as_ref(), b"two", b"three", b"the envelope of a vote"].iter().enumerate() {
            let h = hash_point(VOTE_DST, &[msg]);
            assert!(bool::from(h.point.is_torsion_free()), "message {i}: in G1");
            assert!(bool::from(h.point.is_on_curve()));
            assert!(le(&h.y, &P_MINUS_1_HALF), "the smaller root");
            let raw = raw_point(VOTE_DST, &[msg], h.ctr).expect("the named counter is on the curve");
            assert_eq!(raw[FQ_LEN..], h.y);
            assert!(raw[..FQ_LEN] < P[..], "x below the prime");
            assert_eq!(raw[..FQ_LEN], x_of(VOTE_DST, &[msg], h.ctr)[..]);
            for c in 0..h.ctr {
                assert!(raw_point(VOTE_DST, &[msg], c).is_none(), "an earlier counter was off the curve");
            }
        }
    }

    /// The chain the program runs, step for step through the runtime's own
    /// implementation of the ADD syscall: the same bytes as the library's
    /// scalar product.
    #[test]
    fn the_chain_of_additions_is_the_cofactor_product() {
        for msg in [b"alpha".as_ref(), b"beta", b"gamma"] {
            let h = hash_point(VOTE_DST, &[msg]);
            let p0 = raw_point(VOTE_DST, &[msg], h.ctr).unwrap();
            let cleared = clear_cofactor_by(&p0, g1_add).expect("every addition lands");
            assert_eq!(cleared, h.point.to_uncompressed());
            // the raw point is not in G1; the chain's first step still takes it
            let raw = G1Affine::from_uncompressed_unchecked(&p0).unwrap();
            assert!(!bool::from(raw.is_torsion_free()));
            assert!(g1_add(&p0, &p0).is_some());
            // a point off the curve is refused at the first step
            let mut off = p0;
            off[G1_LEN - 1] ^= 1;
            assert!(g1_add(&off, &off).is_none());
            assert!(clear_cofactor_by(&off, g1_add).is_none());
        }
    }

    /// The pairing check as the program issues it, through the runtime's
    /// own pairing: identity bytes for a good aggregate, something else for
    /// a wrong one.
    #[test]
    fn the_aggregate_verifies_against_the_summed_keys_by_the_runtime_pairing() {
        let keys: Vec<BlsKeypair> = (1u8..=5).map(|i| BlsKeypair::from_seed(&[i; 32])).collect();
        let hash = [3u8; 32];
        let point = vote_point(&hash, 7, 1);
        let sigs: Vec<G1Affine> = keys.iter().map(|k| k.sign(&point.point)).collect();
        let agg = aggregate(&sigs);
        let pk = aggregate_keys(&keys.iter().map(|k| k.pk).collect::<Vec<_>>());
        assert!(verify(&point.point, &pk, &agg), "the library agrees");
        // the keys add up through the G2 ADD syscall, unchecked
        let mut sum = keys[0].pk_bytes();
        for k in &keys[1..] {
            sum = bls12_381_g2_addition_unchecked(Version::V0, &PodG2Point(sum), &PodG2Point(k.pk_bytes()), Endianness::BE).unwrap().0;
        }
        assert_eq!(sum, pk.to_uncompressed());
        let check = |h: &[u8; G1_LEN], pk: &[u8; G2_LEN], sig: &[u8; G1_LEN]| -> bool {
            let mut neg = *sig;
            let mut y = [0u8; FQ_LEN];
            y.copy_from_slice(&neg[FQ_LEN..]);
            negate(&mut y);
            neg[FQ_LEN..].copy_from_slice(&y);
            let g1 = [PodG1Point(*h), PodG1Point(neg)];
            let g2 = [PodG2Point(*pk), PodG2Point(G2_GENERATOR)];
            let gt = bls12_381_pairing_map(Version::V0, &g1, &g2, Endianness::BE).expect("valid points").0;
            gt[..GT_LEN - 1].iter().all(|b| *b == 0) && gt[GT_LEN - 1] == 1
        };
        assert!(check(&point.point.to_uncompressed(), &sum, &agg.to_uncompressed()));
        // one signer missing from the sum: refused
        let four = aggregate_keys(&keys[..4].iter().map(|k| k.pk).collect::<Vec<_>>());
        assert!(!check(&point.point.to_uncompressed(), &four.to_uncompressed(), &agg.to_uncompressed()));
        // the same signers over another envelope: refused
        let other = vote_point(&hash, 7, 2);
        assert!(!check(&other.point.to_uncompressed(), &sum, &agg.to_uncompressed()));
        // the identity of the target group is 575 zero bytes then one
        let id = bls12_381_pairing_map(Version::V0, &[], &[], Endianness::BE).unwrap().0;
        assert!(id[..GT_LEN - 1].iter().all(|b| *b == 0) && id[GT_LEN - 1] == 1);
    }

    #[test]
    fn the_proof_of_possession_binds_the_holder_and_the_unit() {
        let k = BlsKeypair::from_seed(&[9u8; 32]);
        let holder = Pubkey::new_from_array([4u8; 32]);
        let pk = k.pk_bytes();
        let pop = pop_point(5, &holder, &pk);
        let sig = k.sign(&pop.point);
        assert!(verify(&pop.point, &k.pk, &sig));
        let elsewhere = pop_point(6, &holder, &pk);
        assert!(!verify(&elsewhere.point, &k.pk, &sig), "another unit, another point");
        let other = BlsKeypair::from_seed(&[10u8; 32]);
        assert!(!verify(&pop.point, &other.pk, &sig), "another key");
    }

    #[test]
    fn the_instruction_bytes() {
        let k = BlsKeypair::from_seed(&[1u8; 32]);
        let holder = Pubkey::new_from_array([2u8; 32]);
        let reg = registration(holder, 0, &k);
        assert_eq!(reg.data.len(), 1 + 8 + G2_LEN + 4 + FQ_LEN + G1_LEN);
        assert_eq!(reg.data[0], tag::IX_REGISTER_BLS_KEY);
        assert_eq!(reg.accounts.len(), 4);
        assert_eq!(reg.accounts[2].pubkey, bls_key_pda(0, &holder));
        let crank = Pubkey::new_from_array([3u8; 32]);
        let att = attest_bls_side(crank, [7u8; 32], 0, 1, &[(holder, &k)]);
        assert_eq!(att.data.len(), 1 + 32 + 8 + 1 + 4 + FQ_LEN + G1_LEN);
        assert_eq!(att.data[0], tag::IX_ATTEST_BLS);
        assert_eq!(att.accounts.len(), 5);
        assert!(!att.accounts[4].is_writable);
    }
}
