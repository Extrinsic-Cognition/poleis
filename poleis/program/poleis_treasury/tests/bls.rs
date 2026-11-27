//! The aggregate vote on the compiled binary: BLS12-381 keys registered
//! with their proofs of possession, one signature per side verified by the
//! runtime's pairing syscall on Mollusk, every voice landing once, beside
//! the Ed25519 road that stays as it was. The estate is the council
//! suite's: twelve founders in the chairs, a floor of six (alice 3, bob 2,
//! carol 1), ratified.

mod built;
use ed25519_dalek::SigningKey;
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use mollusk_svm_programs_token::token as spl_token;
use poleis_client::bls::{aggregate, attest_bls, attest_bls_side, bls_key_pda, point_bytes, pop_point, register_bls_key, registration, vote_point, BlsKeyView, BlsKeypair};
use poleis_client::{
    attest_batch, claim_seat, code, commit_seat_bid, init_config, open_genesis, propose, proposal_hash, ratify_genesis, registry_pda,
    reveal_seat_bid, seat_commitment, session_pda, settle_seats, vote_envelope, GenesisSpec, Payload, PoleisError, SessionView,
    INSTRUCTIONS_SYSVAR_ID, PROGRAM_ID, ROOT_UNIT, TOKEN_PROGRAM_ID, VOTE_CONTRA, VOTE_PRO,
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

const SOL: u64 = 1_000_000_000;
const WINDOW: u64 = 1_000;
const MANIFEST: [u8; 32] = [9u8; 32];

fn refuse(e: PoleisError) -> Check<'static> {
    Check::err(ProgramError::Custom(code(e)))
}
fn ok() -> Check<'static> {
    Check::success()
}

/// A member with an Ed25519 key (its address) and a BLS key beside it.
struct Voter {
    sk: SigningKey,
    key: Pubkey,
    bls: BlsKeypair,
}
impl Voter {
    fn vote(&self, hash: &[u8; 32], unit: u64, choice: u8) -> (Pubkey, u8, [u8; 64]) {
        use ed25519_dalek::Signer;
        let msg = vote_envelope(hash, unit, choice);
        (self.key, choice, self.sk.sign(&msg).to_bytes())
    }
}

enum Expect {
    Ok,
}

struct World {
    mollusk: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
    seed: u8,
}

impl World {
    fn new() -> Self {
        std::env::set_var("SBF_OUT_DIR", built::program_dir());
        let mut mollusk = Mollusk::new(&PROGRAM_ID, "poleis_treasury");
        spl_token::add_program(&mut mollusk);
        Self { mollusk, accounts: vec![keyed_account_for_system_program(), spl_token::keyed_account()], seed: 1 }
    }
    fn add_wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        key
    }
    fn mint(&mut self) -> Pubkey {
        let key = Pubkey::new_unique();
        let mut data = vec![0u8; 82];
        data[45] = 1;
        let lamports = self.mollusk.sysvars.rent.minimum_balance(82);
        self.accounts.push((key, Account { lamports, data, owner: TOKEN_PROGRAM_ID, executable: false, rent_epoch: 0 }));
        key
    }
    fn voter(&mut self, lamports: u64) -> Voter {
        self.seed += 1;
        let sk = SigningKey::from_bytes(&[self.seed; 32]);
        let key = Pubkey::new_from_array(sk.verifying_key().to_bytes());
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        Voter { sk, key, bls: BlsKeypair::from_seed(&[self.seed; 32]) }
    }
    fn touch(&mut self, key: Pubkey) {
        if key == INSTRUCTIONS_SYSVAR_ID {
            return;
        }
        if !self.accounts.iter().any(|(k, _)| *k == key) {
            self.accounts.push((key, Account::default()));
        }
    }
    fn data(&self, key: &Pubkey) -> Vec<u8> {
        self.accounts.iter().find(|(k, _)| k == key).map(|(_, a)| a.data.clone()).unwrap_or_default()
    }
    fn merge(&mut self, resulting: &[(Pubkey, Account)]) {
        for (key, account) in resulting {
            if let Some(slot) = self.accounts.iter_mut().find(|(k, _)| k == key) {
                slot.1 = account.clone();
            } else {
                self.accounts.push((*key, account.clone()));
            }
        }
    }
    /// One instruction, judged by `checks`; its compute units returned.
    fn run(&mut self, ix: Instruction, checks: &[Check]) -> u64 {
        for meta in &ix.accounts {
            self.touch(meta.pubkey);
        }
        let result = self.mollusk.process_and_validate_instruction(&ix, &self.accounts, checks);
        if std::env::var_os("POLEIS_CU").is_some() {
            eprintln!("CU ix={} accounts={} cu={} ok={}", ix.data[0], ix.accounts.len(), result.compute_units_consumed, result.program_result.is_ok());
        }
        if result.program_result.is_ok() {
            let resulting = result.resulting_accounts.clone();
            self.merge(&resulting);
        }
        result.compute_units_consumed
    }
    fn run_tx(&mut self, ixs: &[Instruction], payer: &Pubkey, expect: Expect) {
        for ix in ixs {
            for meta in &ix.accounts {
                self.touch(meta.pubkey);
            }
        }
        let result = self.mollusk.process_transaction_instructions(ixs, &self.accounts, Some(payer));
        let shown = format!("{:?}", result.raw_result);
        match expect {
            Expect::Ok => assert!(result.raw_result.is_ok(), "transaction refused: {shown}"),
        }
        if result.raw_result.is_ok() {
            let resulting = result.resulting_accounts.clone();
            self.merge(&resulting);
        }
    }
    fn warp(&mut self, slot: u64) {
        self.mollusk.warp_to_slot(slot);
    }
    fn session(&self, hash: &[u8; 32], unit: u64) -> SessionView {
        SessionView::parse(&self.data(&session_pda(hash, unit))).expect("session parses")
    }
    fn key(&self, unit: u64, holder: &Pubkey) -> Option<BlsKeyView> {
        BlsKeyView::parse(&self.data(&bls_key_pda(unit, holder)))
    }
    /// A root proposal over `[ROOT_UNIT]`, opened by `proposer`; its hash.
    fn open(&mut self, proposer: &Pubkey, nonce: u64) -> [u8; 32] {
        let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 6_000 };
        let hash = proposal_hash(payload.class(), payload.origin(), &MANIFEST, &payload.hash(), nonce);
        self.run(propose(*proposer, hash, MANIFEST, &payload, &[ROOT_UNIT]), &[ok()]);
        hash
    }
}

struct Estate {
    w: World,
    crank: Pubkey,
    founders: Vec<Voter>,
    alice: Voter,
    bob: Voter,
    carol: Voter,
}

/// The council suite's estate: twelve founders in the chairs, a floor of
/// six (alice 3, bob 2, carol 1) bought at the reserve, ratified.
fn estate() -> Estate {
    let mut w = World::new();
    let payer = w.add_wallet(100 * SOL);
    let crank = w.add_wallet(100 * SOL);
    let usdc = w.mint();
    w.run(init_config(payer, usdc), &[ok()]);
    let founders: Vec<Voter> = (0..12).map(|_| w.voter(SOL)).collect();
    let g = GenesisSpec {
        charter: [7u8; 32],
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        window: WINDOW,
        rung: 1,
        reserve: 1_000_000,
        deposit: 5_000_000,
        commit_end: 100,
        reveal_end: 200,
        founders: founders.iter().map(|f| f.key).collect(),
        name: b"THE FIRST SOCIETY".to_vec(),
        desc: b"twelve founders and a floor of six".to_vec(),
    };
    w.run(open_genesis(payer, &g), &[ok()]);
    let alice = w.voter(SOL);
    let bob = w.voter(SOL);
    let carol = w.voter(SOL);
    let bids: Vec<(Pubkey, u64, u64)> = vec![
        (alice.key, 0, 3_000_000),
        (alice.key, 1, 3_000_000),
        (alice.key, 2, 3_000_000),
        (bob.key, 0, 2_000_000),
        (bob.key, 1, 2_000_000),
        (carol.key, 0, 1_500_000),
    ];
    let salt = |nonce: u64, price: u64| {
        let mut s = [0u8; 32];
        s[0] = nonce as u8 + 1;
        s[1..9].copy_from_slice(&price.to_le_bytes());
        s
    };
    for (who, nonce, price) in &bids {
        w.run(commit_seat_bid(*who, ROOT_UNIT, 0, *nonce, seat_commitment(*price, *nonce, &salt(*nonce, *price), who)), &[ok()]);
    }
    w.warp(100);
    for (who, nonce, price) in &bids {
        w.run(reveal_seat_bid(*who, ROOT_UNIT, 0, *nonce, *price, salt(*nonce, *price)), &[ok()]);
    }
    w.warp(200);
    w.run(settle_seats(crank, ROOT_UNIT, 0), &[ok()]);
    for (who, nonce, _) in &bids {
        w.run(claim_seat(*who, ROOT_UNIT, 0, *nonce, ROOT_UNIT), &[ok()]);
    }
    w.run(ratify_genesis(crank), &[ok()]);
    Estate { w, crank, founders, alice, bob, carol }
}

fn register(w: &mut World, v: &Voter) -> u64 {
    w.run(registration(v.key, ROOT_UNIT, &v.bls), &[ok()])
}

fn side<'a>(voters: &'a [&'a Voter]) -> Vec<(Pubkey, &'a BlsKeypair)> {
    voters.iter().map(|v| (v.key, &v.bls)).collect()
}

// ================= the key and its proof =================

#[test]
fn a_member_registers_its_key_with_a_proof_of_possession_and_may_rotate_it() {
    let Estate { mut w, alice, founders, .. } = estate();
    let cu = register(&mut w, &alice);
    let k = w.key(ROOT_UNIT, &alice.key).expect("the key account");
    assert_eq!(k.unit, ROOT_UNIT);
    assert_eq!(k.holder, alice.key);
    assert_eq!(k.pk, alice.bls.pk_bytes());
    assert!(cu < 80_000, "registration under the budget it predicts: {cu} CU");
    // a chair holder is a member too
    register(&mut w, &founders[0]);
    assert!(w.key(ROOT_UNIT, &founders[0].key).is_some());
    // rotation: a new key under the same account, with its own proof
    let fresh = BlsKeypair::from_seed(&[77u8; 32]);
    w.run(registration(alice.key, ROOT_UNIT, &fresh), &[ok()]);
    assert_eq!(w.key(ROOT_UNIT, &alice.key).unwrap().pk, fresh.pk_bytes());
}

#[test]
fn a_key_without_a_proof_of_possession_is_refused() {
    let Estate { mut w, alice, bob, .. } = estate();
    // the proof signed by another key: the rogue-key road, closed
    let pk = alice.bls.pk_bytes();
    let pop = pop_point(ROOT_UNIT, &alice.key, &pk);
    let wrong = bob.bls.sign(&pop.point);
    w.run(register_bls_key(alice.key, ROOT_UNIT, &pk, &pop, &point_bytes(&wrong)), &[refuse(PoleisError::BadBlsSignature)]);
    // the proof over another unit's bytes: its counter and root name no point of this unit's message
    let elsewhere = pop_point(1, &alice.key, &pk);
    let sig = alice.bls.sign(&elsewhere.point);
    w.run(register_bls_key(alice.key, ROOT_UNIT, &pk, &elsewhere, &point_bytes(&sig)), &[refuse(PoleisError::BadBlsPoint)]);
    // the right point of this unit, signed as if it were the other unit's: the pairing refuses
    w.run(register_bls_key(alice.key, ROOT_UNIT, &pk, &pop, &point_bytes(&sig)), &[refuse(PoleisError::BadBlsSignature)]);
    // a point the crank misnames: the larger root
    let good = alice.bls.sign(&pop.point);
    let mut y_flip = pop;
    y_flip.y[0] |= 0x10;
    w.run(register_bls_key(alice.key, ROOT_UNIT, &pk, &y_flip, &point_bytes(&good)), &[refuse(PoleisError::BadBlsPoint)]);
    assert!(w.key(ROOT_UNIT, &alice.key).is_none());
    // the honest proof lands
    w.run(register_bls_key(alice.key, ROOT_UNIT, &pk, &pop, &point_bytes(&good)), &[ok()]);
}

#[test]
fn a_stranger_cannot_register() {
    let Estate { mut w, .. } = estate();
    let stranger = w.voter(SOL);
    w.run(registration(stranger.key, ROOT_UNIT, &stranger.bls), &[refuse(PoleisError::NotMember)]);
}

// ================= the aggregate =================

#[test]
fn one_aggregate_lands_every_signer_once() {
    let Estate { mut w, crank, alice, bob, carol, .. } = estate();
    for v in [&alice, &bob, &carol] {
        register(&mut w, v);
    }
    let h = w.open(&alice.key, 1);
    let ix = attest_bls_side(crank, h, ROOT_UNIT, VOTE_PRO, &side(&[&alice, &bob, &carol]));
    let cu = w.run(ix.clone(), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!(s.hex_pro, 6, "alice 3, bob 2, carol 1");
    assert_eq!(s.votes, 3);
    assert_eq!(s.pent_pro, 0);
    assert!(cu < 120_000, "three signers under the budget it predicts: {cu} CU");
    // the same aggregate again: verified, nothing new counted
    w.run(ix, &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (6, 3));
}

#[test]
fn a_second_aggregate_counts_only_the_new_voices() {
    let Estate { mut w, crank, alice, bob, .. } = estate();
    register(&mut w, &alice);
    register(&mut w, &bob);
    let h = w.open(&alice.key, 1);
    w.run(attest_bls_side(crank, h, ROOT_UNIT, VOTE_PRO, &side(&[&alice])), &[ok()]);
    assert_eq!((w.session(&h, ROOT_UNIT).hex_pro, w.session(&h, ROOT_UNIT).votes), (3, 1));
    w.run(attest_bls_side(crank, h, ROOT_UNIT, VOTE_PRO, &side(&[&alice, &bob])), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (5, 2), "alice once, bob the new voice");
}

#[test]
fn an_aggregate_over_another_envelope_is_refused() {
    let Estate { mut w, crank, alice, bob, .. } = estate();
    register(&mut w, &alice);
    register(&mut w, &bob);
    let h = w.open(&alice.key, 1);
    // both signed CONTRA; the crank claims PRO
    let contra = vote_point(&h, ROOT_UNIT, VOTE_CONTRA);
    let agg = aggregate(&[alice.bls.sign(&contra.point), bob.bls.sign(&contra.point)]);
    let pro = vote_point(&h, ROOT_UNIT, VOTE_PRO);
    w.run(attest_bls(crank, h, ROOT_UNIT, VOTE_PRO, &pro, &point_bytes(&agg), &[alice.key, bob.key]), &[refuse(PoleisError::BadBlsSignature)]);
    // the CONTRA point with the PRO claim: the point is recomputed from the claim, not taken from the crank
    w.run(attest_bls(crank, h, ROOT_UNIT, VOTE_PRO, &contra, &point_bytes(&agg), &[alice.key, bob.key]), &[refuse(PoleisError::BadBlsPoint)]);
    // a signer missing from the keys passed
    w.run(attest_bls(crank, h, ROOT_UNIT, VOTE_CONTRA, &contra, &point_bytes(&agg), &[alice.key]), &[refuse(PoleisError::BadBlsSignature)]);
    assert_eq!(w.session(&h, ROOT_UNIT).votes, 0);
    // as signed, it lands
    w.run(attest_bls(crank, h, ROOT_UNIT, VOTE_CONTRA, &contra, &point_bytes(&agg), &[alice.key, bob.key]), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).hex_contra, 5);
}

#[test]
fn a_key_that_is_not_a_key_is_refused() {
    let Estate { mut w, crank, alice, .. } = estate();
    register(&mut w, &alice);
    let h = w.open(&alice.key, 1);
    let pro = vote_point(&h, ROOT_UNIT, VOTE_PRO);
    let sig = point_bytes(&alice.bls.sign(&pro.point));
    // an unregistered holder: an empty account where its key would be
    let mut ix = attest_bls(crank, h, ROOT_UNIT, VOTE_PRO, &pro, &sig, &[alice.key]);
    ix.accounts[4].pubkey = bls_key_pda(ROOT_UNIT, &crank);
    w.run(ix, &[refuse(PoleisError::InvalidAccount)]);
    // the registry in a key's place
    let mut ix = attest_bls(crank, h, ROOT_UNIT, VOTE_PRO, &pro, &sig, &[alice.key]);
    ix.accounts[4].pubkey = registry_pda(ROOT_UNIT);
    w.run(ix, &[refuse(PoleisError::InvalidAccount)]);
    // no key at all
    let mut ix = attest_bls(crank, h, ROOT_UNIT, VOTE_PRO, &pro, &sig, &[alice.key]);
    ix.accounts.pop();
    w.run(ix, &[Check::err(ProgramError::InvalidInstructionData)]);
}

#[test]
fn the_chairs_count_once_each_through_the_aggregate() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate();
    let all: Vec<&Voter> = founders.iter().chain([&alice, &bob, &carol]).collect();
    for v in &all {
        register(&mut w, v);
    }
    let h = w.open(&alice.key, 1);
    let cu = w.run(attest_bls_side(crank, h, ROOT_UNIT, VOTE_PRO, &side(&all)), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!(s.pent_pro, 12, "twelve chairs, once each");
    assert_eq!(s.hex_pro, 6);
    assert_eq!(s.votes, 15);
    assert!(cu < 200_000, "fifteen signers under the budget it predicts: {cu} CU");
}

// ================= beside the Ed25519 road =================

#[test]
fn the_two_roads_count_one_holder_once() {
    let Estate { mut w, crank, alice, bob, .. } = estate();
    register(&mut w, &alice);
    register(&mut w, &bob);
    let h = w.open(&alice.key, 1);
    // alice through the precompile, then alice and bob through the aggregate
    let signed = vec![alice.vote(&h, ROOT_UNIT, VOTE_PRO)];
    w.run_tx(&attest_batch(crank, h, ROOT_UNIT, &signed), &crank, Expect::Ok);
    assert_eq!((w.session(&h, ROOT_UNIT).hex_pro, w.session(&h, ROOT_UNIT).votes), (3, 1));
    w.run(attest_bls_side(crank, h, ROOT_UNIT, VOTE_PRO, &side(&[&alice, &bob])), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (5, 2), "alice not twice");
    // bob through the precompile after the aggregate: not twice either
    let signed = vec![bob.vote(&h, ROOT_UNIT, VOTE_CONTRA)];
    w.run_tx(&attest_batch(crank, h, ROOT_UNIT, &signed), &crank, Expect::Ok);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.hex_contra, s.votes), (5, 0, 2));
}

#[test]
fn a_closed_session_takes_no_aggregate() {
    let Estate { mut w, crank, alice, .. } = estate();
    register(&mut w, &alice);
    let h = w.open(&alice.key, 1);
    let deadline = w.session(&h, ROOT_UNIT).deadline;
    w.warp(deadline + 1);
    w.run(attest_bls_side(crank, h, ROOT_UNIT, VOTE_PRO, &side(&[&alice])), &[refuse(PoleisError::SessionClosed)]);
}
