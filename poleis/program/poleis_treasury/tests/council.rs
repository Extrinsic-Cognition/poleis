//! The council suite: the double seal, run against the compiled binary on
//! Mollusk with real Ed25519 signatures on every vote. The six clauses of
//! the census, by name, then the spec's scenarios, then the refusal ledger.
//!
//! Every vote here is an envelope signed by an ed25519 key, landed through
//! the Ed25519 precompile in the same transaction as `attest`, exactly as on
//! a validator; Mollusk's `precompiles` feature runs the real verifier.

mod built;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use poleis_client::{
    claim_grant, fund_pda, grow_fund, option_hash, option_pda, propose_option, seat_chair, settle_cycle, stake, tally_pda, unstake,
    Fund, FundView, OptionView, TallyView, NO_ROOT,
    knock, knock_pda, pool_pda, withdraw_knock, KnockView, PoolView, RCPT_ADMIT, RCPT_DOOR,
    accrual_hash, accrue_dividend, adjourn, agent_pda, allowance_pda, attest, attest_batch, auction_pda, buy_seats,
    cancel_listing, claim_dividend, claim_seat, claim_token_dividend, close_allowance,
    close_distribution, close_proposal, code, commit_seat_bid, config_pda, config_pda_of, contract_pda, convene,
    council_pda, council_pda_of, create_treasury, create_vault, distribution_pda, endow, escrow_pda, execute,
    execute_of, expire_contract, found_child, found_child_of, init_config, init_config_of, list_seats, listing_pda, mandate, open_auction,
    open_genesis, open_genesis_of, proposal_pda, propose, propose_consent, propose_of, propose_raw, proposal_hash, pull, ratify_genesis, ratify_genesis_of,
    registry_pda, reveal_seat_bid, sale_pda, seal, seat_commitment, session_pda, settle_seats, sha,
    transfer_seats, treasury_pda, unit_holder, upgrade_pda, programdata_pda, programdata_of, vault_pda, vote_envelope,
    AllowanceView, AuctionView, ConfigView, ContractView, DistributionView, FoundSpec, GenesisSpec,
    ListingView, LotSpec, Payload, PoleisError, Proposal, ProposalView, Receipt, Registry, RegistryView, SaleView,
    Session, SessionView, TokenPlanView, Treasury, TreasuryView, ACT_SIGNER, ACT_WRITABLE, BPF_LOADER_UPGRADEABLE_ID,
    CLASS_CODEX, CLASS_SEAT, CLASS_SIGNAL,
    CLASS_CONSENT, CLASS_TREASURY, CODEX_WINDOW_FACTOR, DISTRIBUTION_TTL_SLOTS, HOLDER_EMPTY, HOLDER_KEY,
    HOLDER_UNIT, INSTRUCTIONS_SYSVAR_ID, MAX_NAME, MAX_TERM_SLOTS, MIN_WINDOW_SLOTS,
    MODE_COUNCIL, MODE_GENESIS, PROGRAM_ID, RCPT_ESCROW, RCPT_PULL, RCPT_REFUND, RCPT_RELEASE,
    RCPT_SEAL, RCPT_SEATS, RCPT_TIP, ROOT_UNIT, RULE_REFUND, RULE_RELEASE, STATE_DEADLOCK, STATE_EXPIRED,
    STATE_PASSED, STATE_REJECTED, STATE_VOTING, TOKEN_ACCOUNT_LEN, TOKEN_PROGRAM_ID,
    VOTE_ABSTENTIO, VOTE_CONTRA, VOTE_PRO, chair_quorum, PENTAGON_QUORUM,
};
use poleis_client::{propose_sum, rule_pda, sum_passes, Rule, RuleView, SUM_BUDGET, SUM_KINDS, SUM_RULE, SUM_TRANSFER};
use mollusk_svm_programs_token::token as spl_token;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;
use solana_svm_log_collector::LogCollector;
use std::{cell::RefCell, rc::Rc};

const SOL: u64 = 1_000_000_000;
const WINDOW: u64 = 1_000;
const MANIFEST: [u8; 32] = [9u8; 32];

fn refuse(e: PoleisError) -> Check<'static> {
    Check::err(ProgramError::Custom(code(e)))
}
fn ok() -> Check<'static> {
    Check::success()
}
fn key(p: &Pubkey) -> [u8; 32] {
    *p.as_array()
}

/// A voter with a real signing key.
struct Voter {
    sk: SigningKey,
    key: Pubkey,
}
impl Voter {
    fn vote(&self, hash: &[u8; 32], unit: u64, choice: u8) -> (Pubkey, u8, [u8; 64]) {
        let msg = vote_envelope(hash, unit, choice);
        (self.key, choice, self.sk.sign(&msg).to_bytes())
    }
}

enum Expect {
    Ok,
    Custom(u32),
    Fail,
}

struct World {
    mollusk: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
    seed: u8,
    logs: Rc<RefCell<LogCollector>>,
    /// Every step landed through `Ledger`, by its name, and its compute units: what THE TWO PROGRAMS ON A LIVE VALIDATOR sets beside a
    /// validator's units for the same step.
    landed: Vec<(String, u64)>,
}

impl World {
    fn new() -> Self {
        std::env::set_var("SBF_OUT_DIR", built::program_dir());
        let mut mollusk = Mollusk::new(&PROGRAM_ID, "poleis_treasury");
        // the real SPL Token program, so vaults are real token accounts
        spl_token::add_program(&mut mollusk);
        // every log line the program writes, so receipts can be read back
        let logs = Rc::new(RefCell::new(LogCollector { bytes_limit: None, ..Default::default() }));
        mollusk.logger = Some(logs.clone());
        Self { mollusk, accounts: vec![keyed_account_for_system_program(), spl_token::keyed_account()], seed: 1, logs, landed: Vec::new() }
    }
    /// Every receipt logged so far, in order.
    fn receipts(&self) -> Vec<Receipt> {
        self.logs
            .borrow()
            .get_recorded_content()
            .iter()
            .filter_map(|l| l.strip_prefix("Program data: "))
            .filter_map(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
            .filter_map(|b| Receipt::parse(&b))
            .collect()
    }
    fn add_wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        key
    }
    /// Give lamports to any address, creating it system-owned and empty if new.
    fn fund(&mut self, key: Pubkey, lamports: u64) {
        self.touch(key);
        let slot = self.accounts.iter_mut().find(|(k, _)| *k == key).expect("touched");
        slot.1.lamports += lamports;
    }
    /// An initialized SPL mint with no authorities: 82 bytes, `is_initialized` at 45.
    fn mint(&mut self) -> Pubkey {
        let key = Pubkey::new_unique();
        let mut data = vec![0u8; 82];
        data[45] = 1;
        let lamports = self.rent(82);
        self.accounts.push((key, Account { lamports, data, owner: TOKEN_PROGRAM_ID, executable: false, rent_epoch: 0 }));
        key
    }
    /// An initialized SPL token account: mint at 0, owner at 32, amount at 64, state at 108.
    fn token_account(&mut self, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        let mut data = vec![0u8; TOKEN_ACCOUNT_LEN];
        data[..32].copy_from_slice(mint.as_ref());
        data[32..64].copy_from_slice(owner.as_ref());
        data[64..72].copy_from_slice(&amount.to_le_bytes());
        data[108] = 1;
        let lamports = self.rent(TOKEN_ACCOUNT_LEN);
        self.accounts.push((key, Account { lamports, data, owner: TOKEN_PROGRAM_ID, executable: false, rent_epoch: 0 }));
        key
    }
    /// Deposit into a token account by writing its balance: what a mint-to would do.
    fn set_tokens(&mut self, key: &Pubkey, amount: u64) {
        let slot = self.accounts.iter_mut().find(|(k, _)| k == key).expect("token account");
        slot.1.data[64..72].copy_from_slice(&amount.to_le_bytes());
    }
    fn tokens(&self, key: &Pubkey) -> u64 {
        u64::from_le_bytes(self.data(key)[64..72].try_into().unwrap())
    }
    fn voter(&mut self, lamports: u64) -> Voter {
        self.seed += 1;
        let sk = SigningKey::from_bytes(&[self.seed; 32]);
        let key = Pubkey::new_from_array(sk.verifying_key().to_bytes());
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        Voter { sk, key }
    }
    fn touch(&mut self, key: Pubkey) {
        // Mollusk synthesizes the instructions sysvar per transaction; an
        // empty placeholder would shadow it
        if key == INSTRUCTIONS_SYSVAR_ID {
            return;
        }
        if !self.accounts.iter().any(|(k, _)| *k == key) {
            self.accounts.push((key, Account::default()));
        }
    }
    fn lamports(&self, key: &Pubkey) -> u64 {
        self.accounts.iter().find(|(k, _)| k == key).map(|(_, a)| a.lamports).unwrap_or(0)
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
    fn run(&mut self, ix: Instruction, checks: &[Check]) {
        for meta in &ix.accounts {
            self.touch(meta.pubkey);
        }
        let result = self.mollusk.process_and_validate_instruction(&ix, &self.accounts, checks);
        if std::env::var_os("POLEIS_CU").is_some() { eprintln!("CU ix={} cu={} ok={}", ix.data[0], result.compute_units_consumed, result.program_result.is_ok()); }
        if result.program_result.is_err() {
            return;
        }
        let resulting = result.resulting_accounts.clone();
        self.merge(&resulting);
    }
    /// A whole transaction (the precompile plus attest), judged by its raw result.
    fn run_tx(&mut self, ixs: &[Instruction], payer: &Pubkey, expect: Expect) {
        for ix in ixs {
            for meta in &ix.accounts {
                self.touch(meta.pubkey);
            }
        }
        let result = self.mollusk.process_transaction_instructions(ixs, &self.accounts, Some(payer));
        if std::env::var_os("POLEIS_CU").is_some() { eprintln!("CU tx={} cu={} ok={}", ixs.last().map(|i| i.data[0]).unwrap_or(255), result.compute_units_consumed, result.raw_result.is_ok()); }
        let shown = format!("{:?}", result.raw_result);
        match expect {
            Expect::Ok => assert!(result.raw_result.is_ok(), "transaction refused: {shown}"),
            Expect::Custom(c) => assert!(shown.contains(&format!("Custom({c})")), "expected Custom({c}), got {shown}"),
            Expect::Fail => assert!(result.raw_result.is_err(), "transaction unexpectedly succeeded"),
        }
        if result.raw_result.is_ok() {
            let resulting = result.resulting_accounts.clone();
            self.merge(&resulting);
        }
    }
    fn warp(&mut self, slot: u64) {
        self.mollusk.warp_to_slot(slot);
    }
    fn slot(&self) -> u64 {
        self.mollusk.sysvars.clock.slot
    }
    fn registry(&self, unit: u64) -> RegistryView {
        RegistryView::parse(&self.data(&registry_pda(unit))).expect("registry parses")
    }
    /// A chair written as an older program's seat act left it, for a holder
    /// the seat act refuses now: what a program upgraded in place still holds.
    fn lay_chair(&mut self, unit: u64, k: usize, holder: [u8; 32], kind: u8) {
        let now = self.slot();
        let key = registry_pda(unit);
        let d = &mut self.accounts.iter_mut().find(|(a, _)| *a == key).expect("registry").1.data;
        let o = Registry::OFF_PENTAGONS + k * Registry::PENTAGON_LEN;
        d[o..o + 32].copy_from_slice(&holder);
        d[o + 32] = kind;
        d[o + 33..o + 41].copy_from_slice(&now.to_le_bytes());
        d[o + 41..o + 49].copy_from_slice(&0u64.to_le_bytes());
    }
    /// A chair's term set to end at `term_end`, its holder kept: what an
    /// attachment for a term leaves (the seat act writes `seated_slot = now`).
    fn term_chair(&mut self, unit: u64, k: usize, term_end: u64) {
        let now = self.slot();
        let key = registry_pda(unit);
        let d = &mut self.accounts.iter_mut().find(|(a, _)| *a == key).expect("registry").1.data;
        let o = Registry::OFF_PENTAGONS + k * Registry::PENTAGON_LEN;
        d[o + 33..o + 41].copy_from_slice(&now.to_le_bytes());
        d[o + 41..o + 49].copy_from_slice(&term_end.to_le_bytes());
    }
    /// An account's bytes replaced in place, its lamports kept: a session
    /// written as an older program wrote it.
    fn reshape(&mut self, key: &Pubkey, data: Vec<u8>) {
        self.accounts.iter_mut().find(|(a, _)| a == key).expect("account").1.data = data;
    }
    /// The bytes of a session opened now in `unit`: the prefix and its roster,
    /// one row per holder row of the registry.
    fn session_size(&self, unit: u64) -> usize {
        Session::SIZE + Session::ROSTER_HEADER + self.registry(unit).holders.len() * Session::ELECTORATE_ROW
    }
    fn session(&self, hash: &[u8; 32], unit: u64) -> SessionView {
        SessionView::parse(&self.data(&session_pda(hash, unit))).expect("session parses")
    }
    fn sale(&self, unit: u64, id: u64) -> SaleView {
        SaleView::parse(&self.data(&sale_pda(unit, id))).expect("sale parses")
    }
    fn config(&self) -> ConfigView {
        ConfigView::parse(&self.data(&config_pda())).expect("config parses")
    }
    fn treasury(&self, unit: u64) -> TreasuryView {
        TreasuryView::parse(&self.data(&treasury_pda(unit))).expect("treasury parses")
    }
    fn rent(&self, size: usize) -> u64 {
        self.mollusk.sysvars.rent.minimum_balance(size)
    }
    /// Land votes in batches of four, each batch a precompile plus attest.
    fn cast(&mut self, crank: &Pubkey, hash: &[u8; 32], unit: u64, votes: &[(&Voter, u8)]) {
        for chunk in votes.chunks(4) {
            let signed: Vec<(Pubkey, u8, [u8; 64])> = chunk.iter().map(|(v, c)| v.vote(hash, unit, *c)).collect();
            let ixs = attest_batch(*crank, *hash, unit, &signed);
            self.run_tx(&ixs, crank, Expect::Ok);
        }
    }
    /// Open a proposal for a payload over `impacted`; returns its hash.
    fn open(&mut self, proposer: &Pubkey, payload: &Payload, impacted: &[u64], nonce: u64) -> [u8; 32] {
        let hash = proposal_hash(payload.class(), payload.origin(), &MANIFEST, &payload.hash(), nonce);
        self.run(propose(*proposer, hash, MANIFEST, payload, impacted), &[ok()]);
        hash
    }
}

/// A council after genesis: twelve founders in the root's chairs, or eleven
/// plus a child unit 1 (twelve founders of its own) in chair 11 when
/// `with_child`; a hexagon floor of six seats (alice 3, bob 2, carol 1)
/// bought at the reserve; ratified. In Mollusk's world unless founded in another (`founded`).
struct Estate<L = World> {
    w: L,
    payer: Pubkey,
    crank: Pubkey,
    founders: Vec<Voter>,
    child_founders: Vec<Voter>,
    alice: Voter,
    bob: Voter,
    carol: Voter,
    /// The default mint, named at init_config and inherited by every unit.
    usdc: Pubkey,
}

fn estate(with_child: bool) -> Estate {
    founded(World::new(), with_child)
}

/// `estate`'s founding in any world (`Ledger`): Mollusk's for every test here, a validator's for THE TWO PROGRAMS ON A LIVE VALIDATOR. The
/// sale's slots are counted from the slot the genesis opens at (0 on Mollusk), so a validator's funding of the founders' keys takes none of
/// the windows.
fn founded<L: Ledger>(mut w: L, with_child: bool) -> Estate<L> {
    let payer = w.wallet(100 * SOL);
    let crank = w.wallet(100 * SOL);
    // the default mint: what a real cluster would name USDC
    let usdc = w.mint();
    w.done(init_config(payer, usdc), "init_config");
    let founder_count = if with_child { 11 } else { 12 };
    let founders: Vec<Voter> = (0..founder_count).map(|_| w.voter(SOL)).collect();
    let start = w.slot();
    let g = GenesisSpec {
        charter: [7u8; 32],
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        window: WINDOW,
        rung: 1,
        reserve: 1_000_000,
        deposit: 5_000_000,
        commit_end: start + 100,
        reveal_end: start + 200,
        founders: founders.iter().map(|f| f.key).collect(),
        name: b"THE FIRST SOCIETY".to_vec(),
        desc: b"twelve founders and a floor of six".to_vec(),
    };
    w.done(open_genesis(payer, &g), "open_genesis");
    // eleven founders, so the child's own chair 11 stays free for a grandchild
    let child_founders = if with_child { found(&mut w, payer, ROOT_UNIT, 1, 11, 11) } else { Vec::new() };
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
        w.done(commit_seat_bid(*who, ROOT_UNIT, 0, *nonce, seat_commitment(*price, *nonce, &salt(*nonce, *price), who)), "a sealed seat bid");
    }
    w.wait_until(start + 100);
    for (who, nonce, price) in &bids {
        w.done(reveal_seat_bid(*who, ROOT_UNIT, 0, *nonce, *price, salt(*nonce, *price)), "a seat bid revealed");
    }
    w.wait_until(start + 200);
    w.done(settle_seats(crank, ROOT_UNIT, 0), "the sale settled");
    for (who, nonce, _) in &bids {
        w.done(claim_seat(*who, ROOT_UNIT, 0, *nonce, ROOT_UNIT), "a seat claimed");
    }
    w.done(ratify_genesis(crank), "ratify_genesis");
    Estate { w, payer, crank, founders, child_founders, alice, bob, carol, usdc }
}

fn pro(founders: &[Voter], n: usize) -> Vec<(&Voter, u8)> {
    founders.iter().take(n).map(|f| (f, VOTE_PRO)).collect()
}

/// Found a child with `count` founders on `chair` of `parent` in genesis mode.
fn found<L: Ledger>(w: &mut L, payer: Pubkey, parent: u64, child: u64, chair: u8, count: usize) -> Vec<Voter> {
    let founders: Vec<Voter> = (0..count).map(|_| w.voter(SOL)).collect();
    let spec = FoundSpec {
        child,
        chair,
        charter: [child as u8; 32],
        window: WINDOW,
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        founders: founders.iter().map(|f| f.key).collect(),
        name: format!("CREW {child}").into_bytes(),
        desc: Vec::new(),
    };
    w.done(found_child(payer, parent, &spec), "found a child");
    founders
}

/// Open a root proposal, carry it with nine chairs plus alice and carol,
/// wait out the window, seal: PASSED. Returns the hash.
fn carry(e: &mut Estate, payload: &Payload, nonce: u64) -> [u8; 32] {
    // closure is every class: an estate with a crew decides over the tree
    if !e.child_founders.is_empty() {
        return carry_tree(e, payload, nonce);
    }
    let h = e.w.open(&e.alice.key, payload, &[ROOT_UNIT], nonce);
    let mut v = pro(&e.founders, 9);
    v.extend([(&e.alice, VOTE_PRO), (&e.carol, VOTE_PRO)]);
    e.w.cast(&e.crank, &h, ROOT_UNIT, &v);
    e.w.warp(e.w.slot() + WINDOW + 1);
    e.w.run(seal(e.crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    h
}

/// The same over the whole tree of `estate(true)`: root and its child in
/// chair 11. The child speaks first (eight chairs, no floor: decided at
/// once), its verdict is mandated up, then the root carries. Returns the
/// hash; the impacted list is `[ROOT_UNIT, 1]`.
fn carry_tree(e: &mut Estate, payload: &Payload, nonce: u64) -> [u8; 32] {
    let h = e.w.open(&e.alice.key, payload, &[ROOT_UNIT, 1], nonce);
    e.w.cast(&e.crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(e.crank, h, 1), &[ok()]);
    e.w.run(mandate(e.crank, h, ROOT_UNIT, 1), &[ok()]);
    let mut v = pro(&e.founders, 9);
    v.extend([(&e.alice, VOTE_PRO), (&e.carol, VOTE_PRO)]);
    e.w.cast(&e.crank, &h, ROOT_UNIT, &v);
    e.w.warp(e.w.slot() + WINDOW + 1);
    e.w.run(seal(e.crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    h
}

/// The crew in chair 11 speaks first on a decision opened by hand over
/// `[ROOT_UNIT, 1]`: eight chairs PRO (no floor: decided at once), sealed,
/// mandated up. Only then may the root seal.
fn crew_speaks(w: &mut World, crank: Pubkey, h: [u8; 32], crew: &[Voter]) {
    w.cast(&crank, &h, 1, &pro(crew, 8));
    w.run(seal(crank, h, 1), &[ok()]);
    w.run(mandate(crank, h, ROOT_UNIT, 1), &[ok()]);
}

// ================= clause I: born a dodecahedron =================

#[test]
fn born_a_dodecahedron_and_grown_only_by_the_ladder() {
    let Estate { mut w, payer, crank, founders, alice, bob, .. } = estate(false);

    // twelve founders in twelve chairs, a floor of six sworn seats, the payer dead
    let r = w.registry(ROOT_UNIT);
    assert_eq!(r.filled_chairs(), 12);
    assert_eq!(r.rung, 1);
    assert_eq!(r.total_seats, 6);
    assert_eq!(w.sale(ROOT_UNIT, 0).seat_count, 20, "GPV(1,1) opens twenty seats");
    assert_eq!(w.sale(ROOT_UNIT, 0).clearing, 1_000_000, "six bids for twenty seats clear at the reserve");
    let cfg = w.config();
    assert_eq!(cfg.mode, MODE_COUNCIL);
    assert_eq!(cfg.authority, council_pda(), "the authority is a key nobody holds");
    w.run(create_treasury(payer, 5), &[refuse(PoleisError::InvalidMode)]);
    let lot = LotSpec { auction_id: 1, unit: ROOT_UNIT, pool_root: [1u8; 32], manifest: [2u8; 32], commit_end: 300, reveal_end: 400, reserve: 1, deposit: 1_000_000 };
    w.run(open_auction(payer, &lot), &[refuse(PoleisError::InvalidMode)]);

    // the floor grows only by CODEX vote, and only to the next rung
    let payload = Payload::Ladder { origin: ROOT_UNIT, reserve: 1_000_000, deposit: 5_000_000, commit_len: 100, reveal_len: 100, sale_id: 1 };
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    let mut votes = pro(&founders, 7);
    votes.push((&alice, VOTE_PRO));
    votes.push((&bob, VOTE_PRO));
    w.cast(&crank, &h, ROOT_UNIT, &votes);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[ok()]);
    let r = w.registry(ROOT_UNIT);
    assert_eq!(r.rung, 2);
    assert_eq!(w.sale(ROOT_UNIT, 1).seat_count, 10, "GPV(2,0) − GPV(1,1) = 30 − 20: the increment belongs to the theorem");
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[refuse(PoleisError::AlreadyExecuted)]);
}

// ================= clause II: the double seal =================

#[test]
fn neither_capital_nor_crowd_moves_alone() {
    // the crew in chair 11 is a chamber of every root decision (closure is
    // every class): it speaks first, PRO, and its verdict is chair XII's voice
    let Estate { mut w, crank, founders, child_founders, alice, bob, carol, .. } = estate(true);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let transfer = Payload::Transfer { origin: ROOT_UNIT, dest: 1, amount: SOL };

    // chairs say yes, the floor says no: DEADLOCK, res integra manet
    let h1 = w.open(&alice.key, &transfer, &[ROOT_UNIT, 1], 1);
    w.run(seal(crank, h1, ROOT_UNIT), &[refuse(PoleisError::ChildMustSpeak)]);
    crew_speaks(&mut w, crank, h1, &child_founders);
    let mut v = pro(&founders, 7);
    v.push((&alice, VOTE_CONTRA));
    v.push((&bob, VOTE_PRO));
    w.cast(&crank, &h1, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h1, ROOT_UNIT), &[ok()]);
    let s = w.session(&h1, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.hex_contra, s.active_weight), (8, 2, 3, 6), "seven chairs and the crew");
    assert_eq!(s.state, STATE_DEADLOCK);
    w.run(execute(crank, h1, &[ROOT_UNIT, 1], &transfer), &[refuse(PoleisError::NotPassed)]);

    // the floor says yes, the chairs fall short (five and the crew): DEADLOCK again
    let h2 = w.open(&alice.key, &transfer, &[ROOT_UNIT, 1], 2);
    crew_speaks(&mut w, crank, h2, &child_founders);
    let mut v = pro(&founders, 5);
    v.push((&alice, VOTE_PRO));
    v.push((&bob, VOTE_PRO));
    w.cast(&crank, &h2, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h2, ROOT_UNIT).state, STATE_DEADLOCK);

    // both refuse: REJECTED, in tabularium relatum
    let h3 = w.open(&alice.key, &transfer, &[ROOT_UNIT, 1], 3);
    crew_speaks(&mut w, crank, h3, &child_founders);
    let v: Vec<(&Voter, u8)> = founders.iter().take(3).map(|f| (f, VOTE_CONTRA)).chain([(&alice, VOTE_CONTRA)]).collect();
    w.cast(&crank, &h3, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h3, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h3, ROOT_UNIT).state, STATE_REJECTED);

    // both converge: SIGILLUM, and the money moves exactly once
    let h4 = w.open(&alice.key, &transfer, &[ROOT_UNIT, 1], 4);
    crew_speaks(&mut w, crank, h4, &child_founders);
    let mut v = pro(&founders, 8);
    v.push((&alice, VOTE_PRO));
    v.push((&bob, VOTE_PRO));
    v.push((&carol, VOTE_ABSTENTIO));
    w.cast(&crank, &h4, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h4, ROOT_UNIT), &[ok()]);
    let s = w.session(&h4, ROOT_UNIT);
    assert_eq!(s.state, STATE_PASSED);
    assert_eq!((s.hex_pro, s.hex_abst), (5, 1), "abstention is present, not pro");
    // every voice is on the record, face by face: the census paints from these
    assert_eq!(s.chair_voice(0), Some(VOTE_PRO));
    assert_eq!(s.chair_voice(8), None, "a silent chair is silent");
    assert_eq!(s.chair_voice(11), Some(VOTE_PRO), "the crew's verdict is chair XII's voice");
    let r = w.registry(ROOT_UNIT);
    let idx = |k: &Pubkey| r.holders.iter().position(|hh| hh.holder == *k.as_array()).unwrap();
    assert_eq!(s.holder_voice(idx(&alice.key)), Some(VOTE_PRO));
    assert_eq!(s.holder_voice(idx(&carol.key)), Some(VOTE_ABSTENTIO));
    let d0 = w.lamports(&treasury_pda(1));
    w.run(execute(crank, h4, &[ROOT_UNIT, 1], &transfer), &[ok()]);
    assert_eq!(w.lamports(&treasury_pda(1)) - d0, SOL);
    assert_eq!(w.treasury(ROOT_UNIT).total_out, SOL);
    assert_eq!(w.registry(ROOT_UNIT).plan_spent, SOL, "charged to the plan");

    // nobody speaks, in either chamber: the crew expires, its chair is heard
    // from and absent, the root expires whole
    let h5 = w.open(&alice.key, &transfer, &[ROOT_UNIT, 1], 5);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h5, ROOT_UNIT), &[refuse(PoleisError::ChildMustSpeak)]);
    w.run(seal(crank, h5, 1), &[ok()]);
    assert_eq!(w.session(&h5, 1).state, STATE_EXPIRED);
    w.run(mandate(crank, h5, ROOT_UNIT, 1), &[ok()]);
    w.run(seal(crank, h5, ROOT_UNIT), &[ok()]);
    let s = w.session(&h5, ROOT_UNIT);
    assert_eq!(s.state, STATE_EXPIRED, "an expired crew is absent, not a voice");
    assert_eq!((s.votes, s.pent_abst, s.chair_mask & (1 << 11)), (0, 0, 1 << 11), "heard from, so the seal could proceed");
}

// ================= clause III: unsworn until claimed =================

#[test]
fn an_unclaimed_seat_is_unsworn_and_does_not_count() {
    let Estate { mut w, crank, alice, .. } = estate(false);
    // dave bids on the ladder sale later; here he simply is not a holder
    let dave = w.voter(SOL);
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    let before = w.session(&h, ROOT_UNIT);
    w.cast(&crank, &h, ROOT_UNIT, &[(&dave, VOTE_PRO)]);
    let after = w.session(&h, ROOT_UNIT);
    assert_eq!((after.hex_pro, after.votes), (before.hex_pro, before.votes), "a stranger's envelope lands nothing");
    // and a holder counts once, however many envelopes they sign
    w.cast(&crank, &h, ROOT_UNIT, &[(&alice, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &[(&alice, VOTE_CONTRA)]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.hex_contra, s.votes), (3, 0, 1));
}

// ================= clause IV and V: scope from touch, leaves first =================

#[test]
fn scope_from_touch_and_the_child_speaks_first() {
    // the tree: root ─chair 11─ unit 1 ─chair 11─ unit 2 (unit 1 has eleven founders)
    let Estate { mut w, crank, child_founders: one_founders, alice, .. } = estate(true);
    assert_eq!(w.registry(ROOT_UNIT).chair_of_unit(1), Some(11));
    assert_eq!(w.registry(1).parent, ROOT_UNIT);
    assert_eq!(w.registry(1).level, 1);

    // unit 2 under unit 1, by unit 1's own CODEX; unit 1 has no floor, so its
    // seal is the chairs alone ("camera hexagona nulla")
    let two_founders: Vec<Voter> = (0..12).map(|_| w.voter(SOL)).collect();
    let spec2 = FoundSpec { child: 2, chair: 11, charter: [2u8; 32], window: WINDOW, plan_cap: SOL, plan_period: 1_000_000, founders: two_founders.iter().map(|f| f.key).collect(), name: b"CREW 2".to_vec(), desc: Vec::new() };
    let payload2 = Payload::Found { origin: 1, spec: spec2 };
    let h = w.open(&one_founders[0].key, &payload2, &[1], 2);
    w.cast(&crank, &h, 1, &pro(&one_founders, 7));
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, 1), &[ok()]);
    assert_eq!(w.session(&h, 1).state, STATE_PASSED);
    w.run(execute(crank, h, &[1], &payload2), &[ok()]);
    assert_eq!(w.registry(1).chair_of_unit(2), Some(11));

    // a SIGNAL from unit 1 touches its subtree and nobody else: the root has
    // no session; unit 1 cannot seal before unit 2 has spoken
    let payload = Payload::SaleDefaults { origin: 1, reserve: 2, deposit: 2 };
    let h = w.open(&one_founders[0].key, &payload, &[1, 2], 3);
    assert!(SessionView::parse(&w.data(&session_pda(&h, ROOT_UNIT))).is_none(), "non tangitur");
    w.cast(&crank, &h, 2, &pro(&two_founders, 8));
    w.cast(&crank, &h, 1, &pro(&one_founders, 7));
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, 1), &[refuse(PoleisError::ChildMustSpeak)]);
    w.run(mandate(crank, h, 1, 2), &[refuse(PoleisError::ChildMustSpeak)]);
    w.run(seal(crank, h, 2), &[ok()]);
    assert_eq!(w.session(&h, 2).state, STATE_PASSED);
    w.run(mandate(crank, h, 1, 2), &[ok()]);
    w.run(mandate(crank, h, 1, 2), &[refuse(PoleisError::AlreadyMandated)]);
    let s1 = w.session(&h, 1);
    assert_eq!(s1.pent_pro, 8, "the child's seal is chair XII's voice");
    assert_eq!(s1.chair_mask & (1 << 11), 1 << 11);
    assert_eq!(s1.chair_voice(11), Some(VOTE_PRO), "the mandate is recorded as the chair's voice");
    w.run(seal(crank, h, 1), &[ok()]);
    assert_eq!(w.session(&h, 1).state, STATE_PASSED);
    w.run(execute(crank, h, &[1, 2], &payload), &[ok()]);
    assert_eq!(w.registry(1).seat_reserve, 2);

    // a decision that leaves a bound child out still opens: the child opens
    // its own vote, and its parent cannot seal over its head
    let payload = Payload::SaleDefaults { origin: 1, reserve: 3, deposit: 3 };
    let h = w.open(&one_founders[0].key, &payload, &[1], 6);
    assert!(SessionView::parse(&w.data(&session_pda(&h, 2))).is_none(), "the child has not opened its vote");
    w.cast(&crank, &h, 1, &pro(&one_founders, 7));
    w.run(seal(crank, h, 1), &[refuse(PoleisError::ChildMustSpeak)]);
    w.run(mandate(crank, h, 1, 2), &[refuse(PoleisError::ChildMustSpeak)]);
    w.run(convene(crank, h, 1, 2), &[ok()]);
    w.run(convene(crank, h, 1, 2), &[Check::err(ProgramError::AccountAlreadyInitialized)]);
    assert_eq!(w.session(&h, 2).convener, Some(crank), "the crank paid its rent");
    w.cast(&crank, &h, 2, &pro(&two_founders, 8));
    w.run(seal(crank, h, 2), &[ok()]);
    w.run(mandate(crank, h, 1, 2), &[ok()]);
    w.run(seal(crank, h, 1), &[ok()]);
    assert_eq!(w.session(&h, 1).state, STATE_PASSED);
    w.run(execute(crank, h, &[1], &payload), &[ok()]);
    assert_eq!(w.registry(1).seat_reserve, 3);

    // an impacted set that skips a level is not a subtree: unit 2's parent
    // is not listed
    let payload = Payload::Window { origin: ROOT_UNIT, window: 3_000, codex_window: 3_000 };
    let hash = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &payload.hash(), 5);
    w.run(propose(alice.key, hash, MANIFEST, &payload, &[ROOT_UNIT, 2]), &[refuse(PoleisError::NotDescendant)]);
    w.run(propose(alice.key, hash, MANIFEST, &payload, &[2, ROOT_UNIT]), &[refuse(PoleisError::NotDescendant)]);
}

#[test]
fn a_crews_no_is_its_pentagon_unless_consent_was_asked() {
    let Estate { mut w, crank, founders, child_founders, alice, bob, .. } = estate(true);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: bob.key, amount: SOL };

    // the crew says no: chair XII votes CONTRA, nine chairs and the floor
    // say yes, the root seals PASSED and the budget is paid
    // votes land in any order inside the window; only the seals are ordered
    let h = w.open(&alice.key, &budget, &[ROOT_UNIT, 1], 1);
    let no: Vec<(&Voter, u8)> = child_founders.iter().take(8).map(|f| (f, VOTE_CONTRA)).collect();
    let mut v = pro(&founders, 9);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.cast(&crank, &h, 1, &no);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, 1), &[ok()]);
    assert_eq!(w.session(&h, 1).state, STATE_REJECTED, "no floor: the chairs' verdict is the verdict");
    w.run(mandate(crank, h, ROOT_UNIT, 1), &[ok()]);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.state, s.pent_pro, s.pent_contra, s.chair_voice(11)), (STATE_PASSED, 9, 1, Some(VOTE_CONTRA)), "one chair against, outvoted");
    let b0 = w.lamports(&bob.key);
    w.run(execute(crank, h, &[ROOT_UNIT, 1], &budget), &[ok()]);
    assert_eq!(w.lamports(&bob.key) - b0, SOL, "the tree's verdict is the root's seal");

    // the same decision put with consent asked: the root's chambers say yes
    // just the same, but its seal carries its crew's no, the tree is split,
    // and execute refuses
    let hash = proposal_hash(CLASS_TREASURY | CLASS_CONSENT, ROOT_UNIT, &MANIFEST, &budget.hash(), 2);
    w.run(propose_consent(alice.key, hash, MANIFEST, &budget, &[ROOT_UNIT, 1]), &[ok()]);
    let mut v = pro(&founders, 9);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &hash, ROOT_UNIT, &v);
    w.cast(&crank, &hash, 1, &no);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, hash, 1), &[ok()]);
    w.run(mandate(crank, hash, ROOT_UNIT, 1), &[ok()]);
    w.run(seal(crank, hash, ROOT_UNIT), &[ok()]);
    let s = w.session(&hash, ROOT_UNIT);
    assert_eq!((s.state, s.pent_pro, s.chair_voice(11)), (STATE_DEADLOCK, 9, Some(VOTE_CONTRA)), "both seals over a crew's no: split");
    w.run(execute(crank, hash, &[ROOT_UNIT, 1], &budget), &[refuse(PoleisError::NotPassed)]);
    let view = ProposalView::parse(&w.data(&proposal_pda(&hash))).expect("proposal");
    assert!(view.consent, "the bit is on the record with the payload");
    assert_eq!(view.class, CLASS_TREASURY, "the class reads without the bit");
}

// ================= clause VI: the tabula =================

#[test]
fn the_chain_remembers_every_seal() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    assert_eq!(w.registry(ROOT_UNIT).chain_sig, [0u8; 32]);
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let h1 = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    let mut v = pro(&founders, 7);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h1, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h1, ROOT_UNIT), &[ok()]);
    let sig1 = sha(&[&[0u8; 32], &h1, &[STATE_PASSED]]);
    assert_eq!(w.registry(ROOT_UNIT).chain_sig, sig1);
    let h2 = w.open(&alice.key, &payload, &[ROOT_UNIT], 2);
    // nobody votes: a CODEX session expires after three windows
    w.warp(w.slot() + CODEX_WINDOW_FACTOR * WINDOW + 1);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).chain_sig, sha(&[&sig1, &h2, &[STATE_EXPIRED]]));
}

// ================= recall, attachment, the dignified refusal =================

#[test]
fn recall_and_attachment_are_seat_acts() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    // the condemned votes no on its own deposition and is counted; outvoted
    let recall = Payload::Recall { origin: ROOT_UNIT, chair: 3 };
    let h = w.open(&alice.key, &recall, &[ROOT_UNIT], 1);
    let mut v: Vec<(&Voter, u8)> = founders.iter().enumerate().filter(|(i, _)| *i != 3).take(8).map(|(_, f)| (f, VOTE_PRO)).collect();
    v.push((&founders[3], VOTE_CONTRA));
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.pent_contra, s.state), (8, 1, STATE_PASSED));
    w.run(execute(crank, h, &[ROOT_UNIT], &recall), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).pentagons[3].kind, HOLDER_EMPTY);

    // alice is attached to the empty chair for a term; she votes as a chair
    // while it runs and not after; the recalled founder is a stranger now
    let attach = Payload::Attach { origin: ROOT_UNIT, chair: 3, holder: key(&alice.key), kind: HOLDER_KEY, term: 5_000, retainer: 0 };
    let h = w.open(&alice.key, &attach, &[ROOT_UNIT], 2);
    let mut v = pro(&founders[4..], 7);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    w.run(execute(crank, h, &[ROOT_UNIT], &attach), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).pentagons[3].holder, key(&alice.key));

    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 3);
    w.cast(&crank, &h, ROOT_UNIT, &[(&alice, VOTE_PRO), (&founders[3], VOTE_PRO)]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro), (1, 3), "alice counts in both chambers; the recalled founder in neither");

    w.warp(w.slot() + 6_000);
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 4);
    w.cast(&crank, &h, ROOT_UNIT, &[(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro), (0, 5), "an expired term has no chair voice");
}

// ================= the jet button and the plan =================

#[test]
fn jet_pays_out_of_the_treasury_and_the_plan_is_the_wall() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    w.run(endow(crank, ROOT_UNIT, 20 * SOL), &[ok()]);

    // bob is jetted: two seats at the last clearing price, paid at once
    let jet = Payload::Jet { origin: ROOT_UNIT, holder: key(&bob.key), kind: HOLDER_KEY };
    let h = w.open(&alice.key, &jet, &[ROOT_UNIT], 1);
    let mut v = pro(&founders, 9);
    v.extend([(&alice, VOTE_PRO), (&carol, VOTE_PRO), (&bob, VOTE_CONTRA)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let b0 = w.lamports(&bob.key);
    w.run(execute(crank, h, &[ROOT_UNIT], &jet), &[ok()]);
    assert_eq!(w.lamports(&bob.key) - b0, 2_000_000, "two seats × the last clearing price");
    let r = w.registry(ROOT_UNIT);
    assert_eq!((r.total_seats, r.seats_of(HOLDER_KEY, &key(&bob.key))), (4, 0));

    // the plan: ten SOL per period; 2M spent; a 9.999 SOL budget breaches
    let stranger = w.add_wallet(0);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: stranger, amount: 9_999_000_000 };
    let h = w.open(&alice.key, &budget, &[ROOT_UNIT], 2);
    let mut v = pro(&founders, 9);
    v.extend([(&alice, VOTE_PRO), (&carol, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    w.run(execute(crank, h, &[ROOT_UNIT], &budget), &[refuse(PoleisError::PlanBreached)]);
    // a new period, and the same passed act cashes
    w.warp(w.slot() + 1_000_000);
    w.run(execute(crank, h, &[ROOT_UNIT], &budget), &[ok()]);
    assert_eq!(w.lamports(&stranger), 9_999_000_000);
    assert_eq!(w.registry(ROOT_UNIT).plan_spent, 9_999_000_000);
}

// ================= dividends and dissolution =================

#[test]
fn dividends_flow_by_weight_and_dissolution_freezes() {
    let Estate { mut w, crank, founders, child_founders, alice, bob, carol, .. } = estate(true);
    w.run(endow(crank, ROOT_UNIT, 12 * SOL), &[ok()]);
    // alice sells one seat to unit 1 once her seats have rested a window:
    // the floor is alice 2, bob 2, carol 1, unit 1 has 1
    w.warp(w.slot() + WINDOW + 1);
    w.run(transfer_seats(alice.key, ROOT_UNIT, unit_holder(1), HOLDER_UNIT, 1), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).seats_of(HOLDER_UNIT, &unit_holder(1)), 1);

    let dividend = Payload::Dividend { origin: ROOT_UNIT, amount: 6 * SOL };
    let h = w.open(&alice.key, &dividend, &[ROOT_UNIT, 1], 1);
    crew_speaks(&mut w, crank, h, &child_founders);
    let mut v = pro(&founders, 8);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    w.run(execute(crank, h, &[ROOT_UNIT, 1], &dividend), &[ok()]);

    let a0 = w.lamports(&alice.key);
    w.run(claim_dividend(alice.key, h, ROOT_UNIT, key(&alice.key), HOLDER_KEY), &[ok()]);
    assert_eq!(w.lamports(&alice.key) - a0, 2 * SOL, "the share, and no receipt to pay rent for");
    w.run(claim_dividend(alice.key, h, ROOT_UNIT, key(&alice.key), HOLDER_KEY), &[refuse(PoleisError::AlreadyClaimed)]);
    w.run(claim_dividend(bob.key, h, ROOT_UNIT, key(&carol.key), HOLDER_KEY), &[refuse(PoleisError::Unauthorized)]);
    let t1 = w.lamports(&treasury_pda(1));
    w.run(claim_dividend(crank, h, ROOT_UNIT, unit_holder(1), HOLDER_UNIT), &[ok()]);
    assert_eq!(w.lamports(&treasury_pda(1)) - t1, SOL, "a unit's share lands in its own vault");
    assert_eq!(w.treasury(1).total_in, SOL);
    // bob, who never claimed, leaves whole: his row empties in place, so the
    // last holder (unit 1, already paid) keeps its own index and its own bit
    w.warp(w.slot() + WINDOW + 1);
    w.run(transfer_seats(bob.key, ROOT_UNIT, key(&carol.key), HOLDER_KEY, 2), &[ok()]);
    w.run(claim_dividend(crank, h, ROOT_UNIT, unit_holder(1), HOLDER_UNIT), &[refuse(PoleisError::AlreadyClaimed)]);
    // bob held two seats at the cut: the roster still names him, and pays him
    let b0 = w.lamports(&bob.key);
    w.run(claim_dividend(bob.key, h, ROOT_UNIT, key(&bob.key), HOLDER_KEY), &[ok()]);
    assert_eq!(w.lamports(&bob.key) - b0, 2 * SOL, "a holder who left after the cut is paid on the cut");

    // dissolution: the whole free balance over the weight, and the unit freezes
    let dissolve = Payload::Dissolve { origin: ROOT_UNIT };
    let h = w.open(&alice.key, &dissolve, &[ROOT_UNIT, 1], 2);
    crew_speaks(&mut w, crank, h, &child_founders);
    let mut v = pro(&founders, 8);
    v.extend([(&alice, VOTE_PRO), (&carol, VOTE_PRO)]); // bob's seats are carol's now
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    w.run(execute(crank, h, &[ROOT_UNIT, 1], &dissolve), &[ok()]);
    assert!(w.registry(ROOT_UNIT).frozen);
    assert_eq!(w.lamports(&treasury_pda(ROOT_UNIT)), w.rent(Treasury::SIZE), "the vault is emptied to its floor");
    let c0 = w.lamports(&carol.key);
    w.run(claim_dividend(carol.key, h, ROOT_UNIT, key(&carol.key), HOLDER_KEY), &[ok()]);
    assert!(w.lamports(&carol.key) > c0);
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let hash = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &payload.hash(), 9);
    w.run(propose(alice.key, hash, MANIFEST, &payload, &[ROOT_UNIT]), &[refuse(PoleisError::Frozen)]);
}

// ================= a dividend is paid on the cut's roster =================

/// Cut a lamport dividend of `amount` on the root's floor of `estate(false)`
/// (alice 3, bob 2, carol 1); returns its hash.
fn cut_dividend(e: &mut Estate, amount: u64, nonce: u64) -> [u8; 32] {
    let dividend = Payload::Dividend { origin: ROOT_UNIT, amount };
    let h = carry(e, &dividend, nonce);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &dividend), &[ok()]);
    h
}

/// A key's claim on a lamport dividend; returns what it was paid.
fn claimed(w: &mut World, h: [u8; 32], who: &Pubkey) -> u64 {
    let before = w.lamports(who);
    w.run(claim_dividend(*who, h, ROOT_UNIT, key(who), HOLDER_KEY), &[ok()]);
    w.lamports(who) - before
}

/// A distribution's account as an older program cut it, or with a roster
/// laid by hand: the prefix, then `roster` rows (identity, weight) of keys.
fn lay_distribution(w: &mut World, hash: &[u8; 32], amount: u64, total_weight: u64, payer: &Pubkey, roster: Option<&[([u8; 32], u64)]>) {
    use poleis_client::{Distribution as D, DISC_DISTRIBUTION, LAYOUT_VERSION};
    let (address, bump) = Pubkey::find_program_address(&[b"distribution", hash], &PROGRAM_ID);
    assert_eq!(address, distribution_pda(hash));
    let rows = roster.unwrap_or(&[]);
    let mut data = vec![0u8; if roster.is_some() { D::SIZE + D::ROSTER_HEADER + rows.len() * D::ROSTER_ROW } else { D::SIZE }];
    data[0] = DISC_DISTRIBUTION;
    data[1] = LAYOUT_VERSION;
    data[D::OFF_HASH..D::OFF_HASH + 32].copy_from_slice(hash);
    data[D::OFF_UNIT..D::OFF_UNIT + 8].copy_from_slice(&ROOT_UNIT.to_le_bytes());
    data[D::OFF_AMOUNT..D::OFF_AMOUNT + 8].copy_from_slice(&amount.to_le_bytes());
    data[D::OFF_TOTAL_WEIGHT..D::OFF_TOTAL_WEIGHT + 8].copy_from_slice(&total_weight.to_le_bytes());
    data[D::OFF_BUMP] = bump;
    data[D::OFF_PAYER..D::OFF_PAYER + 32].copy_from_slice(payer.as_ref());
    data[D::OFF_CREATED..D::OFF_CREATED + 8].copy_from_slice(&w.slot().to_le_bytes());
    if roster.is_some() {
        data[D::SIZE..D::SIZE + 8].copy_from_slice(&D::ROSTER_MAGIC);
        data[D::SIZE + 8] = rows.len() as u8;
        for (i, (who, weight)) in rows.iter().enumerate() {
            let o = D::SIZE + D::ROSTER_HEADER + i * D::ROSTER_ROW;
            data[o..o + 32].copy_from_slice(who);
            data[o + 32] = HOLDER_KEY;
            data[o + 33..o + 41].copy_from_slice(&weight.to_le_bytes());
        }
    }
    let lamports = w.rent(data.len()) + amount;
    w.accounts.push((address, Account { lamports, data, owner: PROGRAM_ID, executable: false, rent_epoch: 0 }));
}

#[test]
fn a_dividend_is_paid_on_the_cuts_roster() {
    let mut e = estate(false);
    let (crank, alice, bob, carol) = (e.crank, e.alice.key, e.bob.key, e.carol.key);
    e.w.run(endow(crank, ROOT_UNIT, 12 * SOL), &[ok()]);
    let h = cut_dividend(&mut e, 6 * SOL, 1);
    // the cut carries the floor as it stood: alice 3, bob 2, carol 1, in the registry's rows
    let dist = e.w.data(&distribution_pda(&h));
    assert_eq!(dist.len(), poleis_client::Distribution::SIZE + 9 + 3 * 41);
    assert_eq!(poleis_client::Distribution::roster_find(&dist, HOLDER_KEY, &key(&alice)).unwrap(), Some((0, 3)));
    assert_eq!(poleis_client::Distribution::roster_row(&dist, 2).unwrap(), Some((key(&carol), HOLDER_KEY, 1)));
    assert_eq!(DistributionView::parse(&dist).expect("distribution").total_weight, 6);

    assert_eq!(claimed(&mut e.w, h, &alice), 3 * SOL);
    // a window later alice moves all but one seat to a fresh key: the fresh
    // key is not on the roster, and alice's own row is spent
    e.w.warp(e.w.slot() + WINDOW + 1);
    let dave = e.w.add_wallet(SOL);
    e.w.run(transfer_seats(alice, ROOT_UNIT, key(&dave), HOLDER_KEY, 2), &[ok()]);
    e.w.run(claim_dividend(dave, h, ROOT_UNIT, key(&dave), HOLDER_KEY), &[refuse(PoleisError::NotHolder)]);
    e.w.run(claim_dividend(alice, h, ROOT_UNIT, key(&alice), HOLDER_KEY), &[refuse(PoleisError::AlreadyClaimed)]);
    // and on down the line: dave, rested, hands the seats to another fresh key
    e.w.warp(e.w.slot() + WINDOW + 1);
    let eve = e.w.add_wallet(SOL);
    e.w.run(transfer_seats(dave, ROOT_UNIT, key(&eve), HOLDER_KEY, 2), &[ok()]);
    e.w.run(claim_dividend(eve, h, ROOT_UNIT, key(&eve), HOLDER_KEY), &[refuse(PoleisError::NotHolder)]);

    // every original holder, claiming last, receives its whole share
    assert_eq!(claimed(&mut e.w, h, &bob), 2 * SOL);
    assert_eq!(claimed(&mut e.w, h, &carol), SOL);
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&h))).expect("distribution");
    assert_eq!((dv.paid, dv.amount, dv.claims), (6 * SOL, 6 * SOL, 3));
    assert!(dv.paid <= dv.amount);
    assert_eq!(e.w.lamports(&distribution_pda(&h)), e.w.rent(dist.len()), "the escrow holds its rent and nothing more");
}

#[test]
fn a_seat_bought_after_the_cut_does_not_claim() {
    let mut e = estate(false);
    let (crank, alice, bob, carol) = (e.crank, e.alice.key, e.bob.key, e.carol.key);
    e.w.run(endow(crank, ROOT_UNIT, 12 * SOL), &[ok()]);
    let h = cut_dividend(&mut e, 6 * SOL, 1);
    e.w.warp(e.w.slot() + WINDOW + 1);
    // after the cut, on the market: dave, a newcomer, buys one of alice's
    // seats; carol, on the roster with one seat, buys one of bob's
    let dave = e.w.add_wallet(2 * SOL);
    e.w.run(list_seats(alice, ROOT_UNIT, 1, 1, SOL / 2), &[ok()]);
    e.w.run(buy_seats(dave, alice, ROOT_UNIT, 1, 1, SOL / 2, ROOT_UNIT), &[ok()]);
    e.w.run(list_seats(bob, ROOT_UNIT, 1, 1, SOL / 2), &[ok()]);
    e.w.run(buy_seats(carol, bob, ROOT_UNIT, 1, 1, SOL / 2, ROOT_UNIT), &[ok()]);
    let r = e.w.registry(ROOT_UNIT);
    let seats = |k: &Pubkey| r.seats_of(HOLDER_KEY, &key(k));
    assert_eq!((seats(&alice), seats(&bob), seats(&carol), seats(&dave)), (2, 1, 2, 1));

    e.w.run(claim_dividend(dave, h, ROOT_UNIT, key(&dave), HOLDER_KEY), &[refuse(PoleisError::NotHolder)]);
    assert_eq!(claimed(&mut e.w, h, &carol), SOL, "her one seat at the cut, not the two she holds now");
    assert_eq!(claimed(&mut e.w, h, &alice), 3 * SOL, "her three seats at the cut, though she sold one");
    assert_eq!(claimed(&mut e.w, h, &bob), 2 * SOL);
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&h))).expect("distribution");
    assert_eq!((dv.paid, dv.claims), (6 * SOL, 3));
}

#[test]
fn a_holder_who_left_after_the_cut_still_claims() {
    let mut e = estate(false);
    let (crank, alice, bob, carol) = (e.crank, e.alice.key, e.bob.key, e.carol.key);
    e.w.run(endow(crank, ROOT_UNIT, 12 * SOL), &[ok()]);
    let h = cut_dividend(&mut e, 6 * SOL, 1);
    e.w.warp(e.w.slot() + WINDOW + 1);
    // bob leaves whole before he claims: his row empties in place
    e.w.run(transfer_seats(bob, ROOT_UNIT, key(&carol), HOLDER_KEY, 2), &[ok()]);
    // a newcomer arrives and takes bob's old row
    let eve = e.w.add_wallet(SOL);
    e.w.run(transfer_seats(alice, ROOT_UNIT, key(&eve), HOLDER_KEY, 1), &[ok()]);
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.holders[1].holder, r.holders[1].seats), (key(&eve), 1), "eve sits in bob's old row");
    assert_eq!(r.seats_of(HOLDER_KEY, &key(&bob)), 0);

    // the roster names bob at that row: eve claims nothing, bob is paid
    e.w.run(claim_dividend(eve, h, ROOT_UNIT, key(&eve), HOLDER_KEY), &[refuse(PoleisError::NotHolder)]);
    assert_eq!(claimed(&mut e.w, h, &bob), 2 * SOL);
    e.w.run(claim_dividend(bob, h, ROOT_UNIT, key(&bob), HOLDER_KEY), &[refuse(PoleisError::AlreadyClaimed)]);
    e.w.run(claim_dividend(eve, h, ROOT_UNIT, key(&eve), HOLDER_KEY), &[refuse(PoleisError::NotHolder)]);
    assert_eq!(claimed(&mut e.w, h, &alice), 3 * SOL);
    assert_eq!(claimed(&mut e.w, h, &carol), SOL, "her one seat at the cut, not bob's two");
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&h))).expect("distribution");
    assert_eq!((dv.paid, dv.amount), (6 * SOL, 6 * SOL));
}

#[test]
fn a_token_dividend_is_paid_on_the_cuts_roster() {
    let mut e = estate(false);
    let (crank, payer, alice, bob, carol) = (e.crank, e.payer, e.alice.key, e.bob.key, e.carol.key);
    let mint = e.usdc;
    e.w.run(create_vault(payer, ROOT_UNIT, mint, TOKEN_PROGRAM_ID), &[ok()]);
    let vault = vault_pda(ROOT_UNIT, &mint);
    e.w.set_tokens(&vault, 1_000);
    let plan = Payload::Plan { origin: ROOT_UNIT, cap: 10 * SOL, period: 1_000_000, token_cap: 600 };
    let hplan = carry(&mut e, &plan, 1);
    e.w.run(execute(crank, hplan, &[ROOT_UNIT], &plan), &[ok()]);
    let td = Payload::TokenDividend { origin: ROOT_UNIT, mint, amount: 300, token_program: TOKEN_PROGRAM_ID };
    let ht = carry(&mut e, &td, 2);
    e.w.run(execute(crank, ht, &[ROOT_UNIT], &td), &[ok()]);
    assert_eq!(e.w.tokens(&escrow_pda(&ht)), 300);
    assert_eq!(e.w.data(&distribution_pda(&ht)).len(), poleis_client::Distribution::SIZE + 9 + 3 * 41);

    // after the cut: alice moves two seats to a fresh key, bob leaves whole to carol
    e.w.warp(e.w.slot() + WINDOW + 1);
    let dave = e.w.add_wallet(SOL);
    e.w.run(transfer_seats(alice, ROOT_UNIT, key(&dave), HOLDER_KEY, 2), &[ok()]);
    e.w.run(transfer_seats(bob, ROOT_UNIT, key(&carol), HOLDER_KEY, 2), &[ok()]);

    let dave_ta = e.w.token_account(&mint, &dave, 0);
    e.w.run(claim_token_dividend(dave, ht, ROOT_UNIT, key(&dave), HOLDER_KEY, mint, Some(dave_ta), TOKEN_PROGRAM_ID), &[refuse(PoleisError::NotHolder)]);
    let mut paid = 0;
    for (who, share) in [(alice, 150), (bob, 100), (carol, 50)] {
        let ta = e.w.token_account(&mint, &who, 0);
        e.w.run(claim_token_dividend(who, ht, ROOT_UNIT, key(&who), HOLDER_KEY, mint, Some(ta), TOKEN_PROGRAM_ID), &[ok()]);
        assert_eq!(e.w.tokens(&ta), share, "the share at the cut");
        paid += share;
    }
    assert_eq!((paid, e.w.tokens(&escrow_pda(&ht))), (300, 0));
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&ht))).expect("distribution");
    assert_eq!((dv.paid, dv.amount, dv.mint), (300, 300, mint));
}

#[test]
fn a_distribution_cut_before_the_roster_still_pays() {
    let mut e = estate(false);
    let (crank, alice) = (e.crank, e.alice.key);
    // a distribution as the program cut it before the roster: the prefix
    // alone, six SOL over the floor's six
    let hash = [77u8; 32];
    lay_distribution(&mut e.w, &hash, 6 * SOL, 6, &crank, None);
    assert_eq!(e.w.data(&distribution_pda(&hash)).len(), poleis_client::Distribution::SIZE);
    assert!(DistributionView::parse(&e.w.data(&distribution_pda(&hash))).is_some());

    // claimed as it always was: the registry at the claim decides, so a key
    // that arrived after it claims on its seats now
    e.w.warp(e.w.slot() + WINDOW + 1);
    let dave = e.w.add_wallet(SOL);
    e.w.run(transfer_seats(alice, ROOT_UNIT, key(&dave), HOLDER_KEY, 2), &[ok()]);
    assert_eq!(claimed(&mut e.w, hash, &dave), 2 * SOL);
    assert_eq!(claimed(&mut e.w, hash, &alice), SOL);
    // and a claim rests the seats for a window, as it always did
    e.w.run(transfer_seats(alice, ROOT_UNIT, key(&dave), HOLDER_KEY, 1), &[refuse(PoleisError::SeatsResting)]);
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&hash))).expect("distribution");
    assert_eq!((dv.paid, dv.claims), (3 * SOL, 2));

    // it closes as it always did: after its TTL the dust goes home, the rent to whoever cut it
    e.w.run(close_distribution(hash, ROOT_UNIT, crank, None), &[refuse(PoleisError::NotYet)]);
    e.w.warp(e.w.slot() + DISTRIBUTION_TTL_SLOTS + 1);
    let in0 = e.w.treasury(ROOT_UNIT).total_in;
    let c0 = e.w.lamports(&crank);
    e.w.run(close_distribution(hash, ROOT_UNIT, crank, None), &[ok()]);
    assert_eq!(e.w.treasury(ROOT_UNIT).total_in - in0, 3 * SOL, "bob's and carol's shares went home");
    assert_eq!(e.w.lamports(&crank) - c0, e.w.rent(poleis_client::Distribution::SIZE));
    assert!(e.w.data(&distribution_pda(&hash)).is_empty());
}

#[test]
fn a_claim_finds_its_row_in_a_full_roster() {
    let mut e = estate(false);
    let (crank, alice) = (e.crank, e.alice.key);
    // a roster of MAX_HOLDERS rows, alice in the last: the longest search a claim makes
    let mut rows: Vec<([u8; 32], u64)> = (0..149u8).map(|i| ([i + 1; 32], 1)).collect();
    rows.push((key(&alice), 3));
    let hash = [78u8; 32];
    lay_distribution(&mut e.w, &hash, 152 * 1_000_000, 152, &crank, Some(&rows));
    assert_eq!(e.w.data(&distribution_pda(&hash)).len(), poleis_client::Distribution::SIZE + 9 + 150 * 41);
    assert_eq!(claimed(&mut e.w, hash, &alice), 3_000_000);
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&hash))).expect("distribution");
    assert!(dv.claimed(149) && !dv.claimed(0));
}

// ================= the Fund attaches a Crew =================

#[test]
fn a_fund_pays_to_attach_a_crew() {
    let Estate { mut w, crank, founders, child_founders, alice, bob, .. } = estate(true);
    // unit 1 already sits in chair 11; recall it (a decision it is a chamber
    // of, so it speaks first), then re-attach it with a retainer
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let recall = Payload::Recall { origin: ROOT_UNIT, chair: 11 };
    let h = w.open(&alice.key, &recall, &[ROOT_UNIT, 1], 1);
    crew_speaks(&mut w, crank, h, &child_founders);
    let mut v = pro(&founders, 7);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    w.run(execute(crank, h, &[ROOT_UNIT, 1], &recall), &[ok()]);

    // recalled, the crew is no chamber of the root's decisions: the root alone is closed
    let attach = Payload::Attach { origin: ROOT_UNIT, chair: 11, holder: unit_holder(1), kind: HOLDER_UNIT, term: 50_000, retainer: SOL / 2 };
    let h = w.open(&alice.key, &attach, &[ROOT_UNIT], 2);
    let mut v = pro(&founders, 7);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let t1 = w.lamports(&treasury_pda(1));
    w.run(execute(crank, h, &[ROOT_UNIT], &attach), &[ok()]);
    assert_eq!(w.lamports(&treasury_pda(1)) - t1, SOL / 2, "the retainer lands in the crew's vault");
    assert_eq!(w.registry(ROOT_UNIT).chair_of_unit(1), Some(11));
    assert_eq!(w.registry(ROOT_UNIT).plan_spent, SOL / 2);
}

// ================= the refusal ledger =================

#[test]
fn refusal_ledger() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let stranger = w.voter(SOL);
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };

    // only a member proposes
    let hash = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &payload.hash(), 1);
    w.run(propose(stranger.key, hash, MANIFEST, &payload, &[ROOT_UNIT]), &[refuse(PoleisError::NotMember)]);
    // a founder proposes too: chairs are members
    w.run(propose(founders[0].key, hash, MANIFEST, &payload, &[ROOT_UNIT]), &[ok()]);
    // the class on the wire must be the payload's own class, and a window
    // below the floor is no window
    let wrong = Payload::Transfer { origin: ROOT_UNIT, dest: 1, amount: 1 };
    let h2 = proposal_hash(CLASS_TREASURY, ROOT_UNIT, &MANIFEST, &wrong.hash(), 2);
    w.run(propose_raw(alice.key, h2, CLASS_SIGNAL, ROOT_UNIT, MANIFEST, &wrong.bytes(), &[ROOT_UNIT]), &[refuse(PoleisError::ClassMismatch)]);
    w.run(propose(alice.key, h2, MANIFEST, &wrong, &[ROOT_UNIT]), &[ok()]);
    let tiny = Payload::Window { origin: ROOT_UNIT, window: MIN_WINDOW_SLOTS - 1, codex_window: MIN_WINDOW_SLOTS - 1 };
    let h_tiny = w.open(&alice.key, &tiny, &[ROOT_UNIT], 77);
    let mut v = pro(&founders, 7);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h_tiny, ROOT_UNIT, &v);
    w.run(seal(crank, h_tiny, ROOT_UNIT), &[ok()]);
    w.run(execute(crank, h_tiny, &[ROOT_UNIT], &tiny), &[refuse(PoleisError::InvalidWindow)]);

    // attest without the precompile in front of it is refused
    w.run_tx(&[attest(crank, hash, ROOT_UNIT, &[(alice.key, VOTE_PRO)])], &crank, Expect::Fail);
    // a signed PRO claimed as CONTRA is refused by the envelope check
    let (k, _, sig) = alice.vote(&hash, ROOT_UNIT, VOTE_PRO);
    let ixs = attest_batch(crank, hash, ROOT_UNIT, &[(k, VOTE_CONTRA, sig)]);
    let mut forged = ixs.clone();
    forged[1] = attest(crank, hash, ROOT_UNIT, &[(alice.key, VOTE_CONTRA)]);
    w.run_tx(&forged, &crank, Expect::Fail);
    // a forged signature is refused by the runtime itself
    let bad = attest_batch(crank, hash, ROOT_UNIT, &[(alice.key, VOTE_PRO, [7u8; 64])]);
    w.run_tx(&bad, &crank, Expect::Fail);
    // an envelope for another proposal is refused by the envelope check
    let elsewhere = alice.vote(&h2, ROOT_UNIT, VOTE_PRO);
    let mut mixed = attest_batch(crank, h2, ROOT_UNIT, &[elsewhere]);
    mixed[1] = attest(crank, hash, ROOT_UNIT, &[(alice.key, VOTE_PRO)]);
    w.run_tx(&mixed, &crank, Expect::Custom(code(PoleisError::BadPrecompile)));

    // seal before the deadline, then attest after it
    w.run(seal(crank, hash, ROOT_UNIT), &[refuse(PoleisError::SessionOpen)]);
    // a CODEX session: three windows
    w.warp(w.slot() + CODEX_WINDOW_FACTOR * WINDOW + 1);
    let late = alice.vote(&hash, ROOT_UNIT, VOTE_PRO);
    w.run_tx(&attest_batch(crank, hash, ROOT_UNIT, &[late]), &crank, Expect::Custom(code(PoleisError::SessionClosed)));
    w.run(seal(crank, hash, ROOT_UNIT), &[ok()]);
    w.run(seal(crank, hash, ROOT_UNIT), &[refuse(PoleisError::SessionClosed)]);
    assert_eq!(w.session(&hash, ROOT_UNIT).state, STATE_EXPIRED);

    // execute refuses a payload that does not hash to the proposal
    let h3 = w.open(&alice.key, &payload, &[ROOT_UNIT], 3);
    let mut v = pro(&founders, 7);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h3, ROOT_UNIT, &v);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h3, ROOT_UNIT), &[ok()]);
    // the crank's own payload is ignored: the chain executes the stored one
    let other = Payload::Window { origin: ROOT_UNIT, window: 2_001, codex_window: 2_001 };
    w.run(execute(crank, h3, &[ROOT_UNIT], &other), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).window, 2_000, "what was proposed, not what the crank said");
    let p = ProposalView::parse(&w.data(&proposal_pda(&h3))).expect("proposal parses");
    assert_eq!((p.payload, p.executed, p.payer), (payload.bytes(), true, alice.key));

    // duplicate impacted units
    let h4 = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &payload.hash(), 4);
    w.run(propose(alice.key, h4, MANIFEST, &payload, &[ROOT_UNIT, ROOT_UNIT]), &[Check::err(ProgramError::InvalidInstructionData)]);
    let _ = (CLASS_SEAT, STATE_VOTING, STATE_REJECTED, MODE_GENESIS);
}

// ================= the fast path =================

#[test]
fn a_decided_outcome_seals_without_waiting() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    // seven chairs and exactly half the floor: a vote still to come could
    // matter, so the window holds
    let mut v = pro(&founders, 7);
    v.push((&alice, VOTE_PRO));
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.run(seal(crank, h, ROOT_UNIT), &[refuse(PoleisError::SessionOpen)]);
    // one more seat and nothing still to come can change the outcome
    w.cast(&crank, &h, ROOT_UNIT, &[(&bob, VOTE_PRO)]);
    let opened = w.slot();
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!(s.state, STATE_PASSED);
    assert!(opened <= s.deadline, "sealed inside the window");
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[ok()]);

    // the fast path never fails a proposal early: a losing tally waits
    let h2 = w.open(&alice.key, &payload, &[ROOT_UNIT], 2);
    let v: Vec<(&Voter, u8)> = founders.iter().take(8).map(|f| (f, VOTE_CONTRA)).collect();
    w.cast(&crank, &h2, ROOT_UNIT, &v);
    w.run(seal(crank, h2, ROOT_UNIT), &[refuse(PoleisError::SessionOpen)]);
}

// ================= seats rest =================

#[test]
fn seats_rest_for_a_window_after_they_speak_or_move() {
    let Estate { mut w, crank, alice, bob, carol, .. } = estate(false);
    // fresh from the sale: seats that just arrived cannot move yet
    w.run(transfer_seats(alice.key, ROOT_UNIT, key(&bob.key), HOLDER_KEY, 1), &[refuse(PoleisError::SeatsResting)]);
    w.warp(w.slot() + WINDOW + 1);
    // alice votes, then tries to hand her seats to a fresh key to vote again
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    w.cast(&crank, &h, ROOT_UNIT, &[(&alice, VOTE_PRO)]);
    let dave = w.voter(SOL);
    w.run(transfer_seats(alice.key, ROOT_UNIT, key(&dave.key), HOLDER_KEY, 3), &[refuse(PoleisError::SeatsResting)]);
    // bob has not spoken: his seats move, but they were bob's when the vote
    // opened: dave is not on its roster and bob holds nothing now, so this
    // session counts those seats in neither hand
    w.run(transfer_seats(bob.key, ROOT_UNIT, key(&dave.key), HOLDER_KEY, 2), &[ok()]);
    w.cast(&crank, &h, ROOT_UNIT, &[(&dave, VOTE_PRO), (&bob, VOTE_PRO)]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (3, 1), "alice 3; dave arrived after the opening and bob holds nothing now");
    // dave, having received seats, rests too
    w.run(transfer_seats(dave.key, ROOT_UNIT, key(&carol.key), HOLDER_KEY, 1), &[refuse(PoleisError::SeatsResting)]);
    w.warp(w.slot() + WINDOW + 1);
    w.run(transfer_seats(dave.key, ROOT_UNIT, key(&carol.key), HOLDER_KEY, 1), &[ok()]);
}

// ================= the roster is fixed when a vote opens =================

#[test]
fn no_seat_votes_twice_by_changing_hands() {
    let Estate { mut w, crank, alice, bob, .. } = estate(false);
    // the sale's seats have rested their window
    w.warp(w.slot() + WINDOW + 1);
    // a CODEX vote runs three windows: long enough for a voter's seats to rest and move inside it
    let codex = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let h = w.open(&alice.key, &codex, &[ROOT_UNIT], 1);
    assert_eq!(w.session(&h, ROOT_UNIT).deadline, w.slot() + CODEX_WINDOW_FACTOR * WINDOW);
    w.cast(&crank, &h, ROOT_UNIT, &[(&alice, VOTE_PRO)]);
    // a window on, her seats rest no more: she keeps one and hands two to a fresh key
    w.warp(w.slot() + WINDOW + 1);
    let dave = w.voter(SOL);
    w.run(transfer_seats(alice.key, ROOT_UNIT, key(&dave.key), HOLDER_KEY, 2), &[ok()]);
    w.cast(&crank, &h, ROOT_UNIT, &[(&dave, VOTE_PRO)]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (3, 1), "the seats count once, in the hand that held them when the vote opened");
    let at = w.registry(ROOT_UNIT).holders.iter().position(|x| x.holder == key(&dave.key)).unwrap();
    assert_eq!(s.holder_voice(at), None, "the fresh key's vote landed nothing on the floor");

    // an ordinary vote runs one window: bob speaks at its opening, rests that
    // window, and moves a seat at its last slot, still inside it
    let signal = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 2, deposit: 2 };
    let h2 = w.open(&alice.key, &signal, &[ROOT_UNIT], 2);
    w.cast(&crank, &h2, ROOT_UNIT, &[(&bob, VOTE_PRO)]);
    let deadline = w.session(&h2, ROOT_UNIT).deadline;
    w.warp(deadline);
    let erin = w.voter(SOL);
    w.run(transfer_seats(bob.key, ROOT_UNIT, key(&erin.key), HOLDER_KEY, 1), &[ok()]);
    w.cast(&crank, &h2, ROOT_UNIT, &[(&erin, VOTE_PRO)]);
    let s = w.session(&h2, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (2, 1), "bob's two seats once; the seat he moved does not vote again");
}

#[test]
fn a_seat_bought_after_the_opening_does_not_vote_in_that_session() {
    let Estate { mut w, crank, alice, bob, .. } = estate(false);
    w.warp(w.slot() + WINDOW + 1);
    w.run(list_seats(alice.key, ROOT_UNIT, 1, 1, SOL / 2), &[ok()]);
    let signal = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 2, deposit: 2 };
    let h = w.open(&alice.key, &signal, &[ROOT_UNIT], 1);
    // dave buys a seat while the vote runs: a holder now, not on its roster
    let dave = w.voter(2 * SOL);
    w.run(buy_seats(dave.key, alice.key, ROOT_UNIT, 1, 1, SOL / 2, ROOT_UNIT), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).seats_of(HOLDER_KEY, &key(&dave.key)), 1);
    w.cast(&crank, &h, ROOT_UNIT, &[(&dave, VOTE_PRO)]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (0, 0), "a holder who arrived after the opening lands nothing");
    // alice votes with what she held when the vote opened, the sold seat with it
    w.cast(&crank, &h, ROOT_UNIT, &[(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    assert_eq!(w.session(&h, ROOT_UNIT).hex_pro, 5, "alice's three and bob's two, as the vote opened");
    // the next decision opens with dave on its roster: he votes as anyone does
    let h2 = w.open(&alice.key, &signal, &[ROOT_UNIT], 2);
    w.cast(&crank, &h2, ROOT_UNIT, &[(&dave, VOTE_PRO)]);
    let s = w.session(&h2, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.votes), (1, 1), "dave's seat counts in the vote opened after he bought it");
}

#[test]
fn the_chairs_bar_is_the_openings() {
    let signal = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 2, deposit: 2 };
    {
        // twelve chairs lit; five whose terms end inside the window say PRO,
        // the other seven CONTRA; the five terms lapse before the seal
        let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
        let ends = w.slot() + WINDOW / 2;
        for k in 0..5 {
            w.term_chair(ROOT_UNIT, k, ends);
        }
        let h = w.open(&alice.key, &signal, &[ROOT_UNIT], 1);
        let mut v: Vec<(&Voter, u8)> = founders.iter().enumerate().map(|(i, f)| (f, if i < 5 { VOTE_PRO } else { VOTE_CONTRA })).collect();
        v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
        w.cast(&crank, &h, ROOT_UNIT, &v);
        w.warp(w.slot() + WINDOW + 1);
        let now = w.slot();
        assert_eq!(w.registry(ROOT_UNIT).pentagons.iter().filter(|c| c.term_end == 0 || now <= c.term_end).count(), 7, "five terms have lapsed");
        w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
        let s = w.session(&h, ROOT_UNIT);
        assert_eq!((s.pent_pro, s.pent_contra), (5, 7));
        assert_eq!(s.state, STATE_DEADLOCK, "five is short of the opening's bar of seven; the floor alone said yes");
    }
    {
        // the slow form: six chairs PRO, short of seven; the six CONTRA
        // chairs' terms run out before a late seal
        let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
        let ends = w.slot() + WINDOW + 10;
        for k in 6..12 {
            w.term_chair(ROOT_UNIT, k, ends);
        }
        let h = w.open(&alice.key, &signal, &[ROOT_UNIT], 1);
        let mut v: Vec<(&Voter, u8)> = founders.iter().enumerate().map(|(i, f)| (f, if i < 6 { VOTE_PRO } else { VOTE_CONTRA })).collect();
        v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
        w.cast(&crank, &h, ROOT_UNIT, &v);
        w.warp(ends + 1);
        w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
        let s = w.session(&h, ROOT_UNIT);
        assert_eq!((s.state, s.pent_pro, s.pent_contra), (STATE_DEADLOCK, 6, 6), "six lit at the seal, but the bar is the opening's seven");
    }
}

#[test]
fn a_chair_seated_after_the_opening_waits_for_the_next_decision() {
    let mut e = estate(false);
    let crank = e.crank;
    let codex = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    // a CODEX vote opens with chair 3 held by its founder
    let h = e.w.open(&e.alice.key, &codex, &[ROOT_UNIT], 1);
    // inside its three windows the root recalls chair 3,
    let recall = Payload::Recall { origin: ROOT_UNIT, chair: 3 };
    let r = carry(&mut e, &recall, 2);
    e.w.run(execute(crank, r, &[ROOT_UNIT], &recall), &[ok()]);
    // a second vote opens with the chair empty,
    let h2 = e.w.open(&e.alice.key, &codex, &[ROOT_UNIT], 3);
    // and the root seats alice there
    let attach = Payload::Attach { origin: ROOT_UNIT, chair: 3, holder: key(&e.alice.key), kind: HOLDER_KEY, term: 0, retainer: 0 };
    let t = carry(&mut e, &attach, 4);
    e.w.run(execute(crank, t, &[ROOT_UNIT], &attach), &[ok()]);
    assert_eq!(e.w.registry(ROOT_UNIT).pentagons[3].holder, key(&e.alice.key));
    assert!(e.w.slot() <= e.w.session(&h, ROOT_UNIT).deadline, "both votes still open");
    // in neither vote does chair 3 speak: reseated in the first, attached in the second
    for hh in [h, h2] {
        e.w.cast(&crank, &hh, ROOT_UNIT, &[(&e.alice, VOTE_PRO), (&e.founders[3], VOTE_PRO)]);
        let s = e.w.session(&hh, ROOT_UNIT);
        assert_eq!((s.pent_pro, s.chair_voice(3), s.hex_pro), (0, None, 3), "alice's seats count, her chair waits, the recalled founder is a stranger");
    }
    // the next decision opens with alice in chair 3: she speaks there
    let h3 = e.w.open(&e.alice.key, &codex, &[ROOT_UNIT], 5);
    e.w.cast(&crank, &h3, ROOT_UNIT, &[(&e.alice, VOTE_PRO)]);
    let s = e.w.session(&h3, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.chair_voice(3), s.hex_pro), (1, Some(VOTE_PRO), 3));
}

#[test]
fn under_consent_a_crew_that_said_no_still_counts_when_its_chair_lapses() {
    let Estate { mut w, crank, founders, child_founders, alice, bob, .. } = estate(true);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: bob.key, amount: SOL };
    // the crew in chair 11 sits for a term that ends after it speaks and before the root seals
    let ends = w.slot() + WINDOW + 10;
    w.term_chair(ROOT_UNIT, 11, ends);
    let hash = proposal_hash(CLASS_TREASURY | CLASS_CONSENT, ROOT_UNIT, &MANIFEST, &budget.hash(), 1);
    w.run(propose_consent(alice.key, hash, MANIFEST, &budget, &[ROOT_UNIT, 1]), &[ok()]);
    let mut v = pro(&founders, 9);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &hash, ROOT_UNIT, &v);
    let no: Vec<(&Voter, u8)> = child_founders.iter().take(8).map(|f| (f, VOTE_CONTRA)).collect();
    w.cast(&crank, &hash, 1, &no);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, hash, 1), &[ok()]);
    assert_eq!(w.session(&hash, 1).state, STATE_REJECTED);
    w.run(mandate(crank, hash, ROOT_UNIT, 1), &[ok()]);
    assert_eq!(w.session(&hash, ROOT_UNIT).chair_voice(11), Some(VOTE_CONTRA), "the crew's no is chair XII's voice");
    // the crew's term runs out; the root seals late
    w.warp(ends + 1);
    w.run(seal(crank, hash, ROOT_UNIT), &[ok()]);
    let s = w.session(&hash, ROOT_UNIT);
    assert_eq!((s.state, s.pent_pro), (STATE_DEADLOCK, 9), "a crew of the opening said no: the tree is split, whatever became of its chair");
    w.run(execute(crank, hash, &[ROOT_UNIT, 1], &budget), &[refuse(PoleisError::NotPassed)]);
}

#[test]
fn sessions_opened_before_the_roster_still_seal() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    w.warp(w.slot() + WINDOW + 1);
    let ends = w.slot() + WINDOW / 2;
    for k in 0..5 {
        w.term_chair(ROOT_UNIT, k, ends);
    }
    let signal = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 2, deposit: 2 };
    let bare = w.open(&alice.key, &signal, &[ROOT_UNIT], 1);
    let old = w.open(&alice.key, &signal, &[ROOT_UNIT], 2);
    // rewritten as an older program left them: the bare 158-byte prefix, and
    // "NFTVOTE1" over the same rows
    let d = w.data(&session_pda(&bare, ROOT_UNIT));
    w.reshape(&session_pda(&bare, ROOT_UNIT), d[..Session::SIZE].to_vec());
    let d = w.data(&session_pda(&old, ROOT_UNIT));
    let count = d[Session::SIZE + 8] as usize;
    let rows = &d[Session::SIZE + Session::ROSTER_HEADER..];
    assert_eq!(rows.len(), count * Session::ELECTORATE_ROW);
    let mut nft = d[..Session::SIZE].to_vec();
    nft.extend_from_slice(&Session::ELECTORATE_MAGIC);
    nft.push(count as u8);
    nft.extend_from_slice(rows);
    w.reshape(&session_pda(&old, ROOT_UNIT), nft);
    for (h, len) in [(bare, Session::SIZE), (old, Session::SIZE + Session::ELECTORATE_HEADER + count * Session::ELECTORATE_ROW)] {
        let d = w.data(&session_pda(&h, ROOT_UNIT));
        assert_eq!((d.len(), Session::roster(&d).unwrap()), (len, None));
    }
    // carol hands her seat to a fresh key mid-session; five chairs whose
    // terms end inside the window say PRO, the other seven CONTRA
    let dave = w.voter(SOL);
    w.run(transfer_seats(carol.key, ROOT_UNIT, key(&dave.key), HOLDER_KEY, 1), &[ok()]);
    let mut v: Vec<(&Voter, u8)> = founders.iter().enumerate().map(|(i, f)| (f, if i < 5 { VOTE_PRO } else { VOTE_CONTRA })).collect();
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO), (&dave, VOTE_PRO)]);
    for h in [bare, old] {
        w.cast(&crank, &h, ROOT_UNIT, &v);
    }
    // the bare session counts the registry at the moment, dave's seat with
    // it; the NFTVOTE1 one its rows, which dave is not in
    assert_eq!(w.session(&bare, ROOT_UNIT).hex_pro, 6);
    assert_eq!(w.session(&old, ROOT_UNIT).hex_pro, 5);
    // both judge the chairs at the seal, as before: five lapsed, seven lit,
    // and five PRO meet the bar of four
    w.warp(w.slot() + WINDOW + 1);
    for h in [bare, old] {
        w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
        let s = w.session(&h, ROOT_UNIT);
        assert_eq!((s.state, s.pent_pro, s.pent_contra), (STATE_PASSED, 5, 7));
    }
    assert_eq!(w.session(&bare, ROOT_UNIT).active_weight, 6, "the bare session's floor is the registry's at the seal");
}

// ================= rent comes home =================

#[test]
fn rent_comes_home_when_the_ballot_box_is_spent() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 2_000 };
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    // the ballot box carries its roster: one row per holder row at the opening
    let session_rent = w.rent(w.session_size(ROOT_UNIT));
    // open and undecided: nothing to close
    w.run(close_proposal(h, alice.key, &[ROOT_UNIT]), &[refuse(PoleisError::NotClosable)]);
    let mut v = pro(&founders, 7);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    // passed and not yet executed: still live
    w.run(close_proposal(h, alice.key, &[ROOT_UNIT]), &[refuse(PoleisError::NotClosable)]);
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[ok()]);
    // spent: the session's rent goes to whoever proposed, never to the crank
    w.run(close_proposal(h, crank, &[ROOT_UNIT]), &[refuse(PoleisError::InvalidAccount)]);
    let a0 = w.lamports(&alice.key);
    w.run(close_proposal(h, alice.key, &[ROOT_UNIT]), &[ok()]);
    let tombstone_refund = w.rent(Proposal::SIZE) - w.rent(Proposal::TOMBSTONE_SIZE);
    assert_eq!(w.lamports(&alice.key) - a0, session_rent + tombstone_refund, "the ballot box and the proposal's rent above its tombstone");
    assert!(w.data(&session_pda(&h, ROOT_UNIT)).is_empty(), "the ballot box is gone");
    // the tombstone stays, the hash stays taken, the act stays spent
    let d = w.data(&proposal_pda(&h));
    assert!(ProposalView::is_tombstone(&d) && d[2..34] == h, "the proposal shrank to its hash");
    w.run(propose(alice.key, h, MANIFEST, &payload, &[ROOT_UNIT]), &[Check::err(ProgramError::AccountAlreadyInitialized)]);
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[refuse(PoleisError::InvalidAccount)]);
    // a dead proposal closes too
    let h2 = w.open(&alice.key, &payload, &[ROOT_UNIT], 2);
    w.warp(w.slot() + 2_001);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h2, ROOT_UNIT).state, STATE_EXPIRED);
    w.run(close_proposal(h2, alice.key, &[ROOT_UNIT]), &[ok()]);
}

#[test]
fn a_decision_its_origin_passed_is_carried_out_before_it_closes() {
    let Estate { mut w, crank, founders, child_founders, alice, bob, .. } = estate(true);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: bob.key, amount: SOL };

    // no consent asked: the crew says no, the root says yes, and the root's
    // seal alone decides, so the decision still executes and nobody may burn it
    let h = w.open(&alice.key, &budget, &[ROOT_UNIT, 1], 1);
    let no: Vec<(&Voter, u8)> = child_founders.iter().take(8).map(|f| (f, VOTE_CONTRA)).collect();
    let mut v = pro(&founders, 9);
    v.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.cast(&crank, &h, 1, &no);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, 1), &[ok()]);
    assert_eq!(w.session(&h, 1).state, STATE_REJECTED);
    w.run(mandate(crank, h, ROOT_UNIT, 1), &[ok()]);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    w.run(close_proposal(h, alice.key, &[ROOT_UNIT, 1]), &[refuse(PoleisError::NotClosable)]);
    let b0 = w.lamports(&bob.key);
    w.run(execute(crank, h, &[ROOT_UNIT, 1], &budget), &[ok()]);
    assert_eq!(w.lamports(&bob.key) - b0, SOL, "the decision was carried out");
    w.run(close_proposal(h, alice.key, &[ROOT_UNIT, 1]), &[ok()]);
    assert!(ProposalView::is_tombstone(&w.data(&proposal_pda(&h))));

    // a decision whose origin did not pass is dead and still closes
    let h2 = w.open(&alice.key, &budget, &[ROOT_UNIT, 1], 2);
    w.cast(&crank, &h2, 1, &no);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h2, 1), &[ok()]);
    w.run(mandate(crank, h2, ROOT_UNIT, 1), &[ok()]);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    assert_ne!(w.session(&h2, ROOT_UNIT).state, STATE_PASSED);
    w.run(execute(crank, h2, &[ROOT_UNIT, 1], &budget), &[refuse(PoleisError::NotPassed)]);
    w.run(close_proposal(h2, alice.key, &[ROOT_UNIT, 1]), &[ok()]);
}

// ================= tier 1: names, token vaults, the raw act =================

#[test]
fn a_society_carries_its_name_on_chain_and_renames_by_codex() {
    let mut e = estate(true);
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.name.as_str(), r.desc.as_str()), ("THE FIRST SOCIETY", "twelve founders and a floor of six"));
    assert_eq!(e.w.registry(1).name, "CREW 1", "a founded child is named in the same act");

    // a rename is a CODEX act like any other: the root's, over its whole tree
    let rename = Payload::Name { origin: ROOT_UNIT, name: b"THE SECOND SEASON".to_vec(), desc: Vec::new() };
    let h = carry_tree(&mut e, &rename, 1);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT, 1], &rename), &[ok()]);
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.name.as_str(), r.desc.as_str()), ("THE SECOND SEASON", ""));
    assert_eq!(e.w.registry(1).name, "CREW 1", "the child keeps its own name");

    // the child renames itself by its own chairs alone
    let crew = Payload::Name { origin: 1, name: b"THE NIGHT CREW".to_vec(), desc: b"unit one".to_vec() };
    let h = e.w.open(&e.child_founders[0].key, &crew, &[1], 2);
    e.w.cast(&e.crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(e.crank, h, 1), &[ok()]);
    e.w.run(execute(e.crank, h, &[1], &crew), &[ok()]);
    let r = e.w.registry(1);
    assert_eq!((r.name.as_str(), r.desc.as_str()), ("THE NIGHT CREW", "unit one"));

    // a name past the cap never lands
    let long = Payload::Name { origin: 1, name: vec![b'x'; MAX_NAME + 1], desc: Vec::new() };
    let h = e.w.open(&e.child_founders[0].key, &long, &[1], 3);
    e.w.cast(&e.crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(e.crank, h, 1), &[ok()]);
    e.w.run(execute(e.crank, h, &[1], &long), &[Check::err(ProgramError::InvalidInstructionData)]);
    assert_eq!(e.w.registry(1).name, "THE NIGHT CREW");
}

#[test]
fn token_vaults_move_only_by_the_seal_and_within_the_token_plan() {
    let mut e = estate(true);
    let payer = e.payer;
    let mint = e.usdc;

    // anyone opens a vault; it is a real token account whose authority is the treasury
    e.w.run(create_vault(payer, ROOT_UNIT, mint, TOKEN_PROGRAM_ID), &[ok()]);
    e.w.run(create_vault(payer, 1, mint, TOKEN_PROGRAM_ID), &[ok()]);
    e.w.run(create_vault(payer, ROOT_UNIT, mint, TOKEN_PROGRAM_ID), &[Check::err(ProgramError::AccountAlreadyInitialized)]);
    let v0 = vault_pda(ROOT_UNIT, &mint);
    let v1 = vault_pda(1, &mint);
    let d = e.w.data(&v0);
    assert_eq!(d.len(), TOKEN_ACCOUNT_LEN);
    assert_eq!(&d[..32], mint.as_ref());
    assert_eq!(&d[32..64], treasury_pda(ROOT_UNIT).as_ref(), "the treasury is the authority");
    assert_eq!(d[108], 1, "initialized");
    e.w.set_tokens(&v0, 1_000);

    // no token plan yet: a passed transfer still breaches
    let t = Payload::TokenTransfer { origin: ROOT_UNIT, mint, dest: 1, amount: 400, token_program: TOKEN_PROGRAM_ID };
    let h = carry(&mut e, &t, 1);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT, 1], &t), &[refuse(PoleisError::TokenPlanBreached)]);

    // the plan (a CODEX act, so the whole tree speaks) sets the token cap;
    // the same passed transfer then cashes
    let plan = Payload::Plan { origin: ROOT_UNIT, cap: 10 * SOL, period: 1_000_000, token_cap: 600 };
    let hp = carry_tree(&mut e, &plan, 2);
    e.w.run(execute(e.crank, hp, &[ROOT_UNIT, 1], &plan), &[ok()]);
    assert_eq!(e.w.registry(ROOT_UNIT).token_plan_cap, 600);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT, 1], &t), &[ok()]);
    assert_eq!((e.w.tokens(&v0), e.w.tokens(&v1)), (600, 400));
    assert_eq!(e.w.registry(ROOT_UNIT).token_plan_spent, 400);

    // a budget to a stranger's token account: 300 more breaches the 600 cap, 200 fits
    let stranger = e.w.add_wallet(0);
    let ta = e.w.token_account(&mint, &stranger, 0);
    let big = Payload::TokenBudget { origin: ROOT_UNIT, mint, address: ta, amount: 300, token_program: TOKEN_PROGRAM_ID };
    let hb = carry(&mut e, &big, 3);
    e.w.run(execute(e.crank, hb, &[ROOT_UNIT, 1], &big), &[refuse(PoleisError::TokenPlanBreached)]);
    let small = Payload::TokenBudget { origin: ROOT_UNIT, mint, address: ta, amount: 200, token_program: TOKEN_PROGRAM_ID };
    let hs = carry(&mut e, &small, 4);
    e.w.run(execute(e.crank, hs, &[ROOT_UNIT, 1], &small), &[ok()]);
    assert_eq!((e.w.tokens(&v0), e.w.tokens(&ta)), (400, 200));
    assert_eq!(e.w.registry(ROOT_UNIT).token_plan_spent, 600);

    // a new period, and the breached budget cashes
    e.w.warp(e.w.slot() + 1_000_000);
    e.w.run(execute(e.crank, hb, &[ROOT_UNIT, 1], &big), &[ok()]);
    assert_eq!((e.w.tokens(&v0), e.w.tokens(&ta)), (100, 500));

    // a transfer to oneself is no transfer
    let selfie = Payload::TokenTransfer { origin: ROOT_UNIT, mint, dest: ROOT_UNIT, amount: 1, token_program: TOKEN_PROGRAM_ID };
    let h = carry(&mut e, &selfie, 5);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT, 1], &selfie), &[refuse(PoleisError::InvalidAmount)]);
}

#[test]
fn the_raw_act_is_the_agents_hand_and_never_the_treasurys() {
    let mut e = estate(false);
    let agent = agent_pda(ROOT_UNIT);
    let alice = e.alice.key;
    e.w.fund(agent, SOL);
    // a system transfer: [2,0,0,0][lamports]
    let transfer = |lamports: u64| {
        let mut d = vec![2u8, 0, 0, 0];
        d.extend_from_slice(&lamports.to_le_bytes());
        d
    };

    // the society pays alice from its hands, the program signing for the agent
    let act = Payload::Raw {
        origin: ROOT_UNIT,
        program: Pubkey::default(),
        accounts: vec![(agent, ACT_WRITABLE | ACT_SIGNER), (alice, ACT_WRITABLE)],
        data: transfer(100_000_000),
    };
    let h = carry(&mut e, &act, 1);
    let a0 = e.w.lamports(&alice);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &act), &[ok()]);
    assert_eq!(e.w.lamports(&alice) - a0, 100_000_000);
    assert_eq!(e.w.lamports(&agent), SOL - 100_000_000);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &act), &[refuse(PoleisError::AlreadyExecuted)]);

    // the hands never reach the treasury: any account of this program is refused
    let steal = Payload::Raw {
        origin: ROOT_UNIT,
        program: Pubkey::default(),
        accounts: vec![(treasury_pda(ROOT_UNIT), ACT_WRITABLE | ACT_SIGNER), (alice, ACT_WRITABLE)],
        data: transfer(1),
    };
    let h = carry(&mut e, &steal, 2);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &steal), &[refuse(PoleisError::ActForbidden)]);

    // nor may the act claim a signature for anyone but the agent
    let forge = Payload::Raw {
        origin: ROOT_UNIT,
        program: Pubkey::default(),
        accounts: vec![(alice, ACT_WRITABLE | ACT_SIGNER), (agent, ACT_WRITABLE)],
        data: transfer(1),
    };
    let h = carry(&mut e, &forge, 3);
    let mut unsigned = execute(e.crank, h, &[ROOT_UNIT], &forge);
    for m in unsigned.accounts.iter_mut().filter(|m| m.pubkey == alice) {
        m.is_signer = false;
    }
    e.w.run(unsigned, &[refuse(PoleisError::ActForbidden)]);

    // unless that key signs the execute itself: a named co-signer, the way a key hands the unit what it holds
    let (a1, g1) = (e.w.lamports(&alice), e.w.lamports(&agent));
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &forge), &[ok()]);
    assert_eq!((a1 - e.w.lamports(&alice), e.w.lamports(&agent) - g1), (1, 1));

    // nor call this program back into itself
    let recurse = Payload::Raw { origin: ROOT_UNIT, program: PROGRAM_ID, accounts: vec![], data: vec![0] };
    let h = carry(&mut e, &recurse, 4);
    let mut ix = execute(e.crank, h, &[ROOT_UNIT], &recurse);
    ix.accounts.pop();
    e.w.run(ix, &[refuse(PoleisError::ActForbidden)]);

    // the treasury is untouched throughout
    assert_eq!(e.w.treasury(ROOT_UNIT).total_out, 0);
}

#[test]
fn the_hands_carry_sixteen_accounts_and_no_more() {
    let mut e = estate(false);
    let agent = agent_pda(ROOT_UNIT);
    let alice = e.alice.key;
    e.w.fund(agent, SOL);
    let transfer = |lamports: u64| {
        let mut d = vec![2u8, 0, 0, 0];
        d.extend_from_slice(&lamports.to_le_bytes());
        d
    };

    // sixteen: a stake pool's AddValidatorToPool takes thirteen. The system program reads its first two accounts and lets the rest ride.
    let mut accounts = vec![(agent, ACT_WRITABLE | ACT_SIGNER), (alice, ACT_WRITABLE)];
    accounts.extend((0..14).map(|_| (Pubkey::new_unique(), 0u8)));
    let sixteen = Payload::Raw { origin: ROOT_UNIT, program: Pubkey::default(), accounts: accounts.clone(), data: transfer(7) };
    let h = carry(&mut e, &sixteen, 1);
    let a0 = e.w.lamports(&alice);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &sixteen), &[ok()]);
    assert_eq!(e.w.lamports(&alice) - a0, 7);

    // a seventeenth is refused before anything moves
    accounts.push((Pubkey::new_unique(), 0));
    let seventeen = Payload::Raw { origin: ROOT_UNIT, program: Pubkey::default(), accounts, data: transfer(7) };
    let h = carry(&mut e, &seventeen, 2);
    let a1 = e.w.lamports(&alice); // carrying the proposal through the seal has its own costs; the refusal moves nothing past them
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &seventeen), &[refuse(PoleisError::BadPayload)]);
    assert_eq!(e.w.lamports(&alice), a1);
}

// ================= tier 2: time and promises =================

#[test]
fn an_allowance_is_a_standing_order_under_the_plan() {
    let mut e = estate(true);
    let (crank, alice, bob) = (e.crank, e.alice.key, e.bob.key);
    e.w.run(endow(crank, ROOT_UNIT, 20 * SOL), &[ok()]);

    // alice may pull one SOL per thousand slots, without end
    let allow = Payload::Allowance { origin: ROOT_UNIT, payee: key(&alice), payee_kind: HOLDER_KEY, mint: Pubkey::default(), cap: SOL, period: 1_000, expires: 0 };
    let h = carry(&mut e, &allow, 1);
    let seals = e.w.receipts().into_iter().filter(|r| r.kind == RCPT_SEAL && r.hash == h).count();
    assert_eq!(seals, 2, "each chamber's seal left its receipt: the crew's, then the root's");
    e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &allow), &[ok()]);
    let a = AllowanceView::parse(&e.w.data(&allowance_pda(&h))).expect("allowance");
    assert_eq!((a.cap, a.period, a.payee, a.payer), (SOL, 1_000, key(&alice), crank));

    // two pulls fill the period; a third breaches; nobody pulls for a key but the key
    let a0 = e.w.lamports(&alice);
    e.w.run(pull(alice, h, ROOT_UNIT, 600_000_000, alice, None), &[ok()]);
    e.w.run(pull(alice, h, ROOT_UNIT, 400_000_000, alice, None), &[ok()]);
    assert_eq!(e.w.lamports(&alice) - a0, SOL);
    e.w.run(pull(alice, h, ROOT_UNIT, 1, alice, None), &[refuse(PoleisError::AllowanceBreached)]);
    e.w.run(pull(bob, h, ROOT_UNIT, 1, bob, None), &[refuse(PoleisError::Unauthorized)]);
    assert_eq!(e.w.registry(ROOT_UNIT).plan_spent, SOL, "every pull is charged to the plan");
    assert_eq!(e.w.treasury(ROOT_UNIT).total_out, SOL);

    // the period rolls and the cap is fresh
    e.w.warp(e.w.slot() + 1_000);
    e.w.run(pull(alice, h, ROOT_UNIT, SOL, alice, None), &[ok()]);
    assert_eq!(e.w.lamports(&alice) - a0, 2 * SOL);

    // a unit payee: unit 1 draws half a SOL per period, and anyone lands it in unit 1's treasury
    let expires = e.w.slot() + 5_000;
    let allow1 = Payload::Allowance { origin: ROOT_UNIT, payee: unit_holder(1), payee_kind: HOLDER_UNIT, mint: Pubkey::default(), cap: SOL / 2, period: 1_000, expires };
    let h1 = carry(&mut e, &allow1, 2);
    e.w.run(execute(crank, h1, &[ROOT_UNIT, 1], &allow1), &[ok()]);
    let t0 = e.w.treasury(1).total_in;
    e.w.run(pull(crank, h1, ROOT_UNIT, SOL / 2, treasury_pda(1), None), &[ok()]);
    assert_eq!(e.w.treasury(1).total_in - t0, SOL / 2);
    e.w.run(pull(crank, h1, ROOT_UNIT, 1, alice, None), &[refuse(PoleisError::InvalidAccount)], );

    // revoked by a later proposal: the account closes and its rent goes home to the crank
    let revoke = Payload::Revoke { origin: ROOT_UNIT, allowance: h, payer: crank };
    let hr = carry(&mut e, &revoke, 3);
    let c0 = e.w.lamports(&crank);
    e.w.run(execute(crank, hr, &[ROOT_UNIT, 1], &revoke), &[ok()]);
    assert!(e.w.lamports(&crank) > c0, "the allowance's rent came back");
    assert!(e.w.data(&allowance_pda(&h)).is_empty());
    e.w.run(pull(alice, h, ROOT_UNIT, 1, alice, None), &[refuse(PoleisError::InvalidAccount)]);

    // past its expiry the unit's allowance is dead and closes permissionlessly
    e.w.warp(expires + 1);
    e.w.run(pull(crank, h1, ROOT_UNIT, 1, treasury_pda(1), None), &[refuse(PoleisError::AllowanceDead)]);
    e.w.run(close_allowance(h1, crank), &[ok()]);
    assert!(e.w.data(&allowance_pda(&h1)).is_empty());

    // four pulls, four receipts
    let pulls: Vec<Receipt> = e.w.receipts().into_iter().filter(|r| r.kind == RCPT_PULL).collect();
    assert_eq!(pulls.len(), 4);
    assert_eq!((pulls[0].hash, pulls[0].counterparty, pulls[0].amount, pulls[0].outcome), (h, key(&alice), 600_000_000, HOLDER_KEY));
    assert_eq!((pulls[3].hash, pulls[3].counterparty, pulls[3].amount, pulls[3].outcome), (h1, unit_holder(1), SOL / 2, HOLDER_UNIT));
}

#[test]
fn a_contract_is_escrow_released_by_a_ruling_and_the_tree_is_the_court() {
    let mut e = estate(true);
    let (crank, bob, carol) = (e.crank, e.bob.key, e.carol.key);
    e.w.run(endow(crank, ROOT_UNIT, 20 * SOL), &[ok()]);
    e.w.run(endow(crank, 1, 5 * SOL), &[ok()]);

    // the root hires bob: two SOL in escrow, the root itself the judge
    let deadline = e.w.slot() + 5_000;
    let c = Payload::Contract { origin: ROOT_UNIT, payee: key(&bob), payee_kind: HOLDER_KEY, mint: Pubkey::default(), amount: 2 * SOL, deadline, arbiter: ROOT_UNIT, token_program: TOKEN_PROGRAM_ID };
    let h = carry(&mut e, &c, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &c), &[ok()]);
    assert_eq!(e.w.treasury(ROOT_UNIT).total_out, 2 * SOL, "a promise leaves the treasury when made");
    assert_eq!(e.w.registry(ROOT_UNIT).plan_spent, 2 * SOL);
    let cv = ContractView::parse(&e.w.data(&contract_pda(&h))).expect("contract");
    assert_eq!((cv.amount, cv.payee, cv.arbiter, cv.payer), (2 * SOL, key(&bob), ROOT_UNIT, crank));
    assert_eq!(e.w.lamports(&contract_pda(&h)), e.w.rent(164) + 2 * SOL);
    let cref = cv.reference(TOKEN_PROGRAM_ID);

    // inside the deadline a stranger cannot refund it
    e.w.run(expire_contract(crank, h, &cref), &[refuse(PoleisError::ContractOpen)]);

    // the root accepts the work: a SIGNAL ruling releases to bob and the contract closes
    let rule = Payload::Rule { origin: ROOT_UNIT, hash: h, outcome: RULE_RELEASE, contract: cref.clone() };
    let hr = carry(&mut e, &rule, 2);
    let b0 = e.w.lamports(&bob);
    let c0 = e.w.lamports(&crank);
    e.w.run(execute(crank, hr, &[ROOT_UNIT, 1], &rule), &[ok()]);
    assert_eq!(e.w.lamports(&bob) - b0, 2 * SOL);
    assert!(e.w.data(&contract_pda(&h)).is_empty());
    assert_eq!(e.w.lamports(&crank) - c0, e.w.rent(164), "the contract's rent went home");

    // unit 1 hires carol and names itself the judge: the root has no say
    let c2 = Payload::Contract { origin: 1, payee: key(&carol), payee_kind: HOLDER_KEY, mint: Pubkey::default(), amount: SOL, deadline, arbiter: 1, token_program: TOKEN_PROGRAM_ID };
    let h2 = e.w.open(&e.child_founders[0].key, &c2, &[1], 3);
    e.w.cast(&crank, &h2, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(crank, h2, 1), &[ok()]);
    e.w.run(execute(crank, h2, &[1], &c2), &[ok()]);
    let cref2 = ContractView::parse(&e.w.data(&contract_pda(&h2))).expect("contract").reference(TOKEN_PROGRAM_ID);
    let wrong = Payload::Rule { origin: ROOT_UNIT, hash: h2, outcome: RULE_RELEASE, contract: cref2.clone() };
    let hw = carry(&mut e, &wrong, 4);
    e.w.run(execute(crank, hw, &[ROOT_UNIT, 1], &wrong), &[refuse(PoleisError::NotArbiter)]);
    // carol resigns: the price goes home to unit 1
    let in0 = e.w.treasury(1).total_in;
    e.w.run(expire_contract(carol, h2, &cref2), &[ok()]);
    assert_eq!(e.w.treasury(1).total_in - in0, SOL);
    assert!(e.w.data(&contract_pda(&h2)).is_empty());

    // unit 1 hires carol again with the root as arbiter: the parent's seal refunds
    let c3 = Payload::Contract { origin: 1, payee: key(&carol), payee_kind: HOLDER_KEY, mint: Pubkey::default(), amount: SOL, deadline, arbiter: ROOT_UNIT, token_program: TOKEN_PROGRAM_ID };
    let h3 = e.w.open(&e.child_founders[0].key, &c3, &[1], 5);
    e.w.cast(&crank, &h3, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(crank, h3, 1), &[ok()]);
    e.w.run(execute(crank, h3, &[1], &c3), &[ok()]);
    let cref3 = ContractView::parse(&e.w.data(&contract_pda(&h3))).expect("contract").reference(TOKEN_PROGRAM_ID);
    let verdict = Payload::Rule { origin: ROOT_UNIT, hash: h3, outcome: RULE_REFUND, contract: cref3 };
    let hv = carry(&mut e, &verdict, 6);
    let in1 = e.w.treasury(1).total_in;
    e.w.run(execute(crank, hv, &[ROOT_UNIT, 1], &verdict), &[ok()]);
    assert_eq!(e.w.treasury(1).total_in - in1, SOL);

    // past the deadline anyone refunds (the deadline is judged when the contract is made, at execute)
    let c4 = Payload::Contract { origin: ROOT_UNIT, payee: key(&carol), payee_kind: HOLDER_KEY, mint: Pubkey::default(), amount: SOL, deadline: e.w.slot() + 2_000, arbiter: ROOT_UNIT, token_program: TOKEN_PROGRAM_ID };
    let h4 = carry(&mut e, &c4, 7);
    e.w.run(execute(crank, h4, &[ROOT_UNIT, 1], &c4), &[ok()]);
    let cref4 = ContractView::parse(&e.w.data(&contract_pda(&h4))).expect("contract").reference(TOKEN_PROGRAM_ID);
    e.w.warp(e.w.slot() + 2_000);
    let in2 = e.w.treasury(ROOT_UNIT).total_in;
    e.w.run(expire_contract(bob, h4, &cref4), &[ok()]);
    assert_eq!(e.w.treasury(ROOT_UNIT).total_in - in2, SOL);

    // the record: four escrows, one release, three refunds
    let r = e.w.receipts();
    let count = |k: u8| r.iter().filter(|x| x.kind == k).count();
    assert_eq!((count(RCPT_ESCROW), count(RCPT_RELEASE), count(RCPT_REFUND)), (4, 1, 3));
    let released = r.iter().find(|x| x.kind == RCPT_RELEASE).unwrap();
    assert_eq!((released.hash, released.unit, released.counterparty, released.amount), (h, ROOT_UNIT, key(&bob), 2 * SOL));
}

#[test]
fn income_by_rule_and_tokens_to_the_floor() {
    let mut e = estate(true);
    let (crank, alice, payer) = (e.crank, e.alice.key, e.payer);

    // a fifth of every inflow, every thousand slots
    let policy = Payload::DividendPolicy { origin: ROOT_UNIT, bps: 2_000, period: 1_000 };
    let hp = carry_tree(&mut e, &policy, 1);
    e.w.run(execute(crank, hp, &[ROOT_UNIT, 1], &policy), &[ok()]);
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.dividend_bps, r.dividend_period, r.dividend_seq), (2_000, 1_000, 0));
    e.w.run(accrue_dividend(crank, ROOT_UNIT, 0), &[refuse(PoleisError::NotYet)]);

    // ten SOL come in; a period passes; anyone cranks the accrual: two SOL for the floor
    e.w.run(endow(crank, ROOT_UNIT, 10 * SOL), &[ok()]);
    e.w.warp(e.w.slot() + 1_000);
    e.w.run(accrue_dividend(crank, ROOT_UNIT, 0), &[ok()]);
    let dh = accrual_hash(ROOT_UNIT, 0);
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&dh))).expect("distribution");
    assert_eq!((dv.amount, dv.payer, dv.is_token()), (2 * SOL, crank, false));
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.dividend_seq, r.accrued_in), (1, e.w.treasury(ROOT_UNIT).total_in));
    assert_eq!(e.w.registry(ROOT_UNIT).plan_spent, 0, "the policy is not charged to the plan");

    // alice, three seats of six, claims one SOL
    let a0 = e.w.lamports(&alice);
    e.w.run(claim_dividend(alice, dh, ROOT_UNIT, key(&alice), HOLDER_KEY), &[ok()]);
    assert_eq!(e.w.lamports(&alice) - a0, SOL);

    // a period with no income rolls without a distribution
    e.w.warp(e.w.slot() + 1_000);
    e.w.run(accrue_dividend(crank, ROOT_UNIT, 1), &[ok()]);
    assert!(e.w.data(&distribution_pda(&accrual_hash(ROOT_UNIT, 1))).is_empty());
    assert_eq!(e.w.registry(ROOT_UNIT).dividend_seq, 1);

    // the distribution closes only after its TTL: the unclaimed SOL goes home, the rent to the crank
    e.w.run(close_distribution(dh, ROOT_UNIT, crank, None), &[refuse(PoleisError::NotYet)]);
    e.w.warp(e.w.slot() + DISTRIBUTION_TTL_SLOTS + 1);
    let in0 = e.w.treasury(ROOT_UNIT).total_in;
    let c0 = e.w.lamports(&crank);
    e.w.run(close_distribution(dh, ROOT_UNIT, crank, None), &[ok()]);
    assert_eq!(e.w.treasury(ROOT_UNIT).total_in - in0, SOL, "bob's and carol's shares went home");
    // the prefix and the cut's roster: alice, bob and carol
    assert_eq!(e.w.lamports(&crank) - c0, e.w.rent(162 + 9 + 3 * 41));
    assert!(e.w.data(&distribution_pda(&dh)).is_empty());

    // tokens: a vault of a thousand, a token cap of six hundred, a dividend of three hundred
    let mint = e.usdc;
    e.w.run(create_vault(payer, ROOT_UNIT, mint, TOKEN_PROGRAM_ID), &[ok()]);
    let vault = vault_pda(ROOT_UNIT, &mint);
    e.w.set_tokens(&vault, 1_000);
    let plan = Payload::Plan { origin: ROOT_UNIT, cap: 10 * SOL, period: 1_000_000, token_cap: 600 };
    let hplan = carry_tree(&mut e, &plan, 2);
    e.w.run(execute(crank, hplan, &[ROOT_UNIT, 1], &plan), &[ok()]);
    let td = Payload::TokenDividend { origin: ROOT_UNIT, mint, amount: 300, token_program: TOKEN_PROGRAM_ID };
    let ht = carry(&mut e, &td, 3);
    e.w.run(execute(crank, ht, &[ROOT_UNIT, 1], &td), &[ok()]);
    assert_eq!((e.w.tokens(&vault), e.w.tokens(&escrow_pda(&ht))), (700, 300));
    assert_eq!(e.w.registry(ROOT_UNIT).token_plan_spent, 300, "a living unit's token dividend is under the token plan");
    let dv = DistributionView::parse(&e.w.data(&distribution_pda(&ht))).expect("distribution");
    assert_eq!((dv.mint, dv.amount), (mint, 300));
    let alice_ta = e.w.token_account(&mint, &alice, 0);
    e.w.run(claim_token_dividend(alice, ht, ROOT_UNIT, key(&alice), HOLDER_KEY, mint, Some(alice_ta), TOKEN_PROGRAM_ID), &[ok()]);
    assert_eq!((e.w.tokens(&alice_ta), e.w.tokens(&escrow_pda(&ht))), (150, 150));
    let stranger_ta = e.w.token_account(&mint, &crank, 0);
    e.w.run(claim_token_dividend(e.bob.key, ht, ROOT_UNIT, key(&e.bob.key), HOLDER_KEY, mint, Some(stranger_ta), TOKEN_PROGRAM_ID), &[refuse(PoleisError::Unauthorized)]);

    // dissolution freezes the unit; the vault is not stranded: a sweep to the floor is the one act left
    let dissolve = Payload::Dissolve { origin: ROOT_UNIT };
    let hd = carry(&mut e, &dissolve, 4);
    e.w.run(execute(crank, hd, &[ROOT_UNIT, 1], &dissolve), &[ok()]);
    assert!(e.w.registry(ROOT_UNIT).frozen);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: crank, amount: 1 };
    let hb = proposal_hash(budget.class(), ROOT_UNIT, &MANIFEST, &budget.hash(), 5);
    e.w.run(propose(alice, hb, MANIFEST, &budget, &[ROOT_UNIT]), &[refuse(PoleisError::Frozen)]);
    let sweep = Payload::TokenDividend { origin: ROOT_UNIT, mint, amount: 0, token_program: TOKEN_PROGRAM_ID };
    let hs = carry(&mut e, &sweep, 6);
    e.w.run(execute(crank, hs, &[ROOT_UNIT, 1], &sweep), &[ok()]);
    assert_eq!((e.w.tokens(&vault), e.w.tokens(&escrow_pda(&hs))), (0, 700));
    assert_eq!(e.w.registry(ROOT_UNIT).token_plan_spent, 300, "a sweep is outside the plan");
    e.w.run(claim_token_dividend(alice, hs, ROOT_UNIT, key(&alice), HOLDER_KEY, mint, Some(alice_ta), TOKEN_PROGRAM_ID), &[ok()]);
    assert_eq!(e.w.tokens(&alice_ta), 500);

    // a token distribution closes the same way, its dust back in the vault
    e.w.warp(e.w.slot() + DISTRIBUTION_TTL_SLOTS + 1);
    e.w.run(close_distribution(ht, ROOT_UNIT, crank, Some((mint, TOKEN_PROGRAM_ID))), &[ok()]);
    assert_eq!(e.w.tokens(&vault), 150);
    assert!(e.w.data(&escrow_pda(&ht)).is_empty());
}

#[test]
fn codex_takes_three_windows_and_terms_have_a_ceiling() {
    let mut e = estate(false);
    let (crank, alice) = (e.crank, e.alice.key);
    assert_eq!(e.w.registry(ROOT_UNIT).codex_window, WINDOW * CODEX_WINDOW_FACTOR);

    // a CODEX session waits three windows; a TREASURY one waits one
    let name = Payload::Name { origin: ROOT_UNIT, name: b"SLOW".to_vec(), desc: Vec::new() };
    let h = e.w.open(&alice, &name, &[ROOT_UNIT], 1);
    assert_eq!(e.w.session(&h, ROOT_UNIT).deadline, e.w.slot() + 3 * WINDOW);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: crank, amount: 1 };
    let hb = e.w.open(&alice, &budget, &[ROOT_UNIT], 2);
    assert_eq!(e.w.session(&hb, ROOT_UNIT).deadline, e.w.slot() + WINDOW);
    // an undecided CODEX session does not seal after one window
    e.w.cast(&crank, &h, ROOT_UNIT, &pro(&e.founders, 6));
    e.w.warp(e.w.slot() + WINDOW + 1);
    e.w.run(seal(crank, h, ROOT_UNIT), &[refuse(PoleisError::SessionOpen)]);
    e.w.warp(e.w.slot() + 2 * WINDOW);
    e.w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_REJECTED);

    // the Window payload sets both; a CODEX window below the voting window is refused
    let bad = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 1_000 };
    let hbad = carry(&mut e, &bad, 3);
    e.w.run(execute(crank, hbad, &[ROOT_UNIT], &bad), &[refuse(PoleisError::InvalidWindow)]);
    let good = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 4_000 };
    let hgood = carry(&mut e, &good, 4);
    e.w.run(execute(crank, hgood, &[ROOT_UNIT], &good), &[ok()]);
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.window, r.codex_window), (2_000, 4_000));

    // a chair's term is at most a year: recall chair 0, then attach a stranger
    let recall = Payload::Recall { origin: ROOT_UNIT, chair: 0 };
    let hr = carry(&mut e, &recall, 5);
    e.w.run(execute(crank, hr, &[ROOT_UNIT], &recall), &[ok()]);
    let stranger = e.w.add_wallet(0);
    let long = Payload::Attach { origin: ROOT_UNIT, chair: 0, holder: key(&stranger), kind: HOLDER_KEY, term: MAX_TERM_SLOTS + 1, retainer: 0 };
    let hl = carry(&mut e, &long, 6);
    e.w.run(execute(crank, hl, &[ROOT_UNIT], &long), &[refuse(PoleisError::InvalidWindow)]);
    let year = Payload::Attach { origin: ROOT_UNIT, chair: 0, holder: key(&stranger), kind: HOLDER_KEY, term: MAX_TERM_SLOTS, retainer: 0 };
    let hy = carry(&mut e, &year, 7);
    e.w.run(execute(crank, hy, &[ROOT_UNIT], &year), &[ok()]);
    assert_eq!(e.w.registry(ROOT_UNIT).pentagons[0].holder, key(&stranger));
}

#[test]
fn seats_trade_on_the_market_at_the_sellers_price() {
    let mut e = estate(false);
    let (crank, alice, bob) = (e.crank, e.alice.key, e.bob.key);
    e.w.warp(e.w.slot() + WINDOW + 1);

    // alice lists two of her three seats at half a SOL each; dave buys one
    e.w.run(list_seats(alice, ROOT_UNIT, 1, 2, SOL / 2), &[ok()]);
    let l = ListingView::parse(&e.w.data(&listing_pda(ROOT_UNIT, &alice, 1))).expect("listing");
    assert_eq!((l.count, l.price, l.seller), (2, SOL / 2, alice));
    let dave = e.w.add_wallet(2 * SOL);
    let a0 = e.w.lamports(&alice);
    // the buyer names the price; another price is refused before any money moves
    e.w.run(buy_seats(dave, alice, ROOT_UNIT, 1, 1, SOL / 2 + 1, ROOT_UNIT), &[refuse(PoleisError::PriceChanged)]);
    e.w.run(buy_seats(dave, alice, ROOT_UNIT, 1, 1, SOL / 2, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.lamports(&alice) - a0, SOL / 2);
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.seats_of(HOLDER_KEY, &key(&alice)), r.seats_of(HOLDER_KEY, &key(&dave)), r.total_seats), (2, 1, 6));
    assert_eq!(ListingView::parse(&e.w.data(&listing_pda(ROOT_UNIT, &alice, 1))).unwrap().count, 1);
    e.w.run(buy_seats(dave, alice, ROOT_UNIT, 1, 5, SOL / 2, ROOT_UNIT), &[refuse(PoleisError::ListingShort)]);

    // alice speaks in a session: her seats rest and cannot be delivered; the listing survives
    let name = Payload::Name { origin: ROOT_UNIT, name: b"MARKET".to_vec(), desc: Vec::new() };
    let h = e.w.open(&alice, &name, &[ROOT_UNIT], 1);
    e.w.cast(&crank, &h, ROOT_UNIT, &[(&e.alice, VOTE_PRO)]);
    e.w.run(buy_seats(dave, alice, ROOT_UNIT, 1, 1, SOL / 2, ROOT_UNIT), &[refuse(PoleisError::SeatsResting)]);
    assert_eq!(ListingView::parse(&e.w.data(&listing_pda(ROOT_UNIT, &alice, 1))).unwrap().count, 1);
    e.w.warp(e.w.slot() + WINDOW + 1);
    e.w.run(buy_seats(dave, alice, ROOT_UNIT, 1, 1, SOL / 2, ROOT_UNIT), &[ok()]);
    assert!(e.w.data(&listing_pda(ROOT_UNIT, &alice, 1)).is_empty(), "a sold-out listing closes");
    let r = e.w.registry(ROOT_UNIT);
    assert_eq!((r.seats_of(HOLDER_KEY, &key(&alice)), r.seats_of(HOLDER_KEY, &key(&dave)), r.total_seats), (1, 2, 6));

    // bob lists and thinks better of it; a stranger cannot list at all
    e.w.run(list_seats(bob, ROOT_UNIT, 7, 1, SOL), &[ok()]);
    e.w.run(cancel_listing(bob, ROOT_UNIT, 7), &[ok()]);
    assert!(e.w.data(&listing_pda(ROOT_UNIT, &bob, 7)).is_empty());
    let nobody = e.w.add_wallet(SOL);
    e.w.run(list_seats(nobody, ROOT_UNIT, 1, 1, SOL), &[refuse(PoleisError::NotHolder)]);

    // two sales, two receipts, each naming the listing and the buyer
    let sales: Vec<Receipt> = e.w.receipts().into_iter().filter(|r| r.kind == RCPT_SEATS).collect();
    assert_eq!(sales.len(), 2);
    assert_eq!((sales[0].hash, sales[0].counterparty, sales[0].amount), (key(&listing_pda(ROOT_UNIT, &alice, 1)), key(&dave), SOL / 2));
}

// ================= tier 3: the code, the cranks, the tombstone =================

#[test]
fn cranks_are_tipped_under_the_plan_and_never_block_the_seal() {
    let mut e = estate(false);
    let crank = e.crank;
    e.w.run(endow(crank, ROOT_UNIT, 20 * SOL), &[ok()]);

    // a hundredth of a SOL per landed batch and per seal
    let set = Payload::CrankTip { origin: ROOT_UNIT, tip: 10_000_000 };
    let h = carry(&mut e, &set, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &set), &[ok()]);
    assert_eq!(e.w.registry(ROOT_UNIT).crank_tip, 10_000_000);

    // a decision: nine votes in batches of four land three batches, then one seal: four tips
    let c0 = e.w.lamports(&crank);
    let name = Payload::Name { origin: ROOT_UNIT, name: b"TIPPED".to_vec(), desc: Vec::new() };
    carry(&mut e, &name, 2);
    assert_eq!(e.w.lamports(&crank) - c0, 40_000_000, "three batches and a seal");
    assert_eq!(e.w.registry(ROOT_UNIT).plan_spent, 40_000_000, "tips are charged to the plan");
    assert_eq!(e.w.treasury(ROOT_UNIT).total_out, 40_000_000);
    assert_eq!(e.w.receipts().into_iter().filter(|r| r.kind == RCPT_TIP).count(), 4);

    // a batch that lands no new voice earns nothing
    let again = Payload::Name { origin: ROOT_UNIT, name: b"AGAIN".to_vec(), desc: Vec::new() };
    let h = e.w.open(&e.alice.key, &again, &[ROOT_UNIT], 3);
    let c1 = e.w.lamports(&crank);
    e.w.cast(&crank, &h, ROOT_UNIT, &pro(&e.founders, 4));
    assert_eq!(e.w.lamports(&crank) - c1, 10_000_000);
    e.w.cast(&crank, &h, ROOT_UNIT, &pro(&e.founders, 4));
    assert_eq!(e.w.lamports(&crank) - c1, 10_000_000, "the same voices again: no tip");

    // the plan is the wall, and the seal never waits on it: a tip past the
    // plan is skipped and the session still seals
    let too_much = Payload::CrankTip { origin: ROOT_UNIT, tip: 11 * SOL };
    let h = carry(&mut e, &too_much, 4);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &too_much), &[ok()]);
    let c2 = e.w.lamports(&crank);
    let h = carry(&mut e, &name, 5);
    assert_eq!(e.w.lamports(&crank), c2, "no tip the plan cannot pay");
    assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_PASSED);
}

#[test]
fn a_closed_proposal_shrinks_to_its_hash_and_the_rent_comes_home() {
    let mut e = estate(false);
    let (crank, alice) = (e.crank, e.alice.key);
    let name = Payload::Name { origin: ROOT_UNIT, name: b"TOMB".to_vec(), desc: Vec::new() };
    let h = carry(&mut e, &name, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &name), &[ok()]);
    let a0 = e.w.lamports(&alice);
    e.w.run(close_proposal(h, alice, &[ROOT_UNIT]), &[ok()]);
    let d = e.w.data(&proposal_pda(&h));
    assert_eq!(d.len(), Proposal::TOMBSTONE_SIZE);
    assert!(ProposalView::is_tombstone(&d));
    assert_eq!(&d[2..34], &h);
    let expected = e.w.rent(e.w.session_size(ROOT_UNIT)) + e.w.rent(Proposal::SIZE) - e.w.rent(Proposal::TOMBSTONE_SIZE);
    assert_eq!(e.w.lamports(&alice) - a0, expected, "the session's rent and the proposal's rent above the tombstone");
    // the hash stays taken and the record stays closed
    e.w.run(execute(crank, h, &[ROOT_UNIT], &name), &[refuse(PoleisError::InvalidAccount)]);
    e.w.run(propose(alice, h, MANIFEST, &name, &[ROOT_UNIT]), &[Check::err(ProgramError::AccountAlreadyInitialized)]);
    e.w.run(close_proposal(h, alice, &[ROOT_UNIT]), &[refuse(PoleisError::InvalidAccount)]);
}

/// The loader's accounts as the ceremony leaves them: the program, its
/// program data with the upgrade PDA as authority, and a buffer under the
/// same authority holding the code to deploy.
fn loader_estate(e: &mut Estate, elf: &[u8]) -> (Pubkey, Pubkey) {
    let authority = upgrade_pda();
    let mut pd = vec![3u8, 0, 0, 0];
    pd.extend_from_slice(&0u64.to_le_bytes());
    pd.push(1);
    pd.extend_from_slice(authority.as_ref());
    pd.extend_from_slice(elf);
    let mut buf = vec![1u8, 0, 0, 0, 1];
    buf.extend_from_slice(authority.as_ref());
    buf.extend_from_slice(elf);
    let buffer = Pubkey::new_unique();
    let spill = e.w.add_wallet(0);
    let pd_rent = e.w.rent(pd.len());
    let buf_rent = e.w.rent(buf.len());
    // tree 0's founding already named this address (ONLY THE DEPLOYER FOUNDS ROOT 0), so the world holds it empty: laid in its place
    e.w.accounts.retain(|(k, _)| *k != programdata_pda());
    e.w.accounts.push((programdata_pda(), Account { lamports: pd_rent, data: pd, owner: BPF_LOADER_UPGRADEABLE_ID, executable: false, rent_epoch: 0 }));
    e.w.accounts.push((buffer, Account { lamports: buf_rent, data: buf, owner: BPF_LOADER_UPGRADEABLE_ID, executable: false, rent_epoch: 0 }));
    e.w.accounts.push((PROGRAM_ID, mollusk_svm::program::create_program_account_loader_v3(&PROGRAM_ID)));
    e.w.accounts.push(mollusk_svm::program::keyed_account_for_bpf_loader_v3_program());
    e.w.accounts.push(e.w.mollusk.sysvars.keyed_account_for_rent_sysvar());
    e.w.accounts.push(e.w.mollusk.sysvars.keyed_account_for_clock_sysvar());
    (buffer, spill)
}

#[test]
fn a_crew_cannot_touch_the_code() {
    let mut e = estate(true);
    let crank = e.crank;
    let elf = built::program_elf();
    let (buffer, spill) = loader_estate(&mut e, &elf);
    let up = Payload::Upgrade { origin: 1, buffer, spill };
    let h = e.w.open(&e.child_founders[0].key, &up, &[1], 1);
    e.w.cast(&crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(crank, h, 1), &[ok()]);
    e.w.run(execute(crank, h, &[1], &up), &[refuse(PoleisError::Unauthorized)]);
    let hand = Payload::UpgradeAuthority { origin: 1, program: PROGRAM_ID, new: Some(e.child_founders[0].key) };
    let h = e.w.open(&e.child_founders[0].key, &hand, &[1], 2);
    e.w.cast(&crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(crank, h, 1), &[ok()]);
    e.w.run(execute(crank, h, &[1], &hand), &[refuse(PoleisError::Unauthorized)]);
    // the root opens its upgrade alone, but never seals it over its crew's
    // head: the crew opens its own vote and speaks first
    let up0 = Payload::Upgrade { origin: ROOT_UNIT, buffer, spill };
    let h = e.w.open(&e.alice.key, &up0, &[ROOT_UNIT], 3);
    let mut v = pro(&e.founders, 9);
    v.extend([(&e.alice, VOTE_PRO), (&e.carol, VOTE_PRO)]);
    e.w.cast(&crank, &h, ROOT_UNIT, &v);
    e.w.run(seal(crank, h, ROOT_UNIT), &[refuse(PoleisError::ChildMustSpeak)]);
    e.w.run(convene(crank, h, ROOT_UNIT, 1), &[ok()]);
    crew_speaks(&mut e.w, crank, h, &e.child_founders);
    e.w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_PASSED);
}

/// The positive path needs the upgradeable loader as a CPI target, which
/// Mollusk does not serve ("Unsupported program id"); it is proven on a
/// validator by `js/test/e2e-localnet.mjs`, which redeploys the program by
/// the seal and hands the authority back. Kept here, ignored, as the
/// account shapes the ceremony must leave.
#[test]
#[ignore = "the upgradeable loader is not a CPI target under Mollusk; proven by the localnet e2e"]
fn the_root_upgrades_the_code_over_the_whole_tree() {
    let mut e = estate(true);
    let crank = e.crank;
    let elf = built::program_elf();
    let (buffer, spill) = loader_estate(&mut e, &elf);
    let up = Payload::Upgrade { origin: ROOT_UNIT, buffer, spill };
    let h = carry_tree(&mut e, &up, 1);
    let deployed_before = u64::from_le_bytes(e.w.data(&programdata_pda())[4..12].try_into().unwrap());
    e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &up), &[ok()]);
    let pd = e.w.data(&programdata_pda());
    let deployed_after = u64::from_le_bytes(pd[4..12].try_into().unwrap());
    assert!(deployed_after > deployed_before, "the program data records the new deployment slot");
    assert_eq!(&pd[13..45], upgrade_pda().as_ref(), "the authority is still the seal's");
    assert!(e.w.lamports(&spill) > 0, "the buffer's lamports spilled");
    // the authority moves by the same law: to a key, then burned
    let key = e.w.add_wallet(0);
    let hand = Payload::UpgradeAuthority { origin: ROOT_UNIT, program: PROGRAM_ID, new: Some(key) };
    let h = carry_tree(&mut e, &hand, 2);
    e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &hand), &[ok()]);
    assert_eq!(&e.w.data(&programdata_pda())[13..45], key.as_ref());
}

/// A program's data as the loader leaves it after a handoff: its upgrade authority Poleis's upgrade PDA, the polis's seal.
fn a_program_handed_to_the_seal(e: &mut Estate, program: &Pubkey) -> Pubkey {
    let mut pd = vec![3u8, 0, 0, 0];
    pd.extend_from_slice(&0u64.to_le_bytes());
    pd.push(1);
    pd.extend_from_slice(upgrade_pda().as_ref());
    pd.extend_from_slice(&[0u8; 64]);
    let at = programdata_of(program);
    let rent = e.w.rent(pd.len());
    e.w.accounts.push((at, Account { lamports: rent, data: pd, owner: BPF_LOADER_UPGRADEABLE_ID, executable: false, rent_epoch: 0 }));
    at
}

/// THE SEAL NAMES ITS PROGRAM (Nov 23 2026; the fault THE-LAUNCH-PRESS-LANDING.md named, "the seal's reach is loose"): three programs answer
/// to the one upgrade PDA after the handoff, and until this lane the act that moves an authority named the new holder alone, so whoever
/// carried a passed act out chose the program data, and with it the program. An act voted for the market, a hand or a burn, is refused on
/// the $WARM program's data and on Poleis's own, with a holder or with none; carried out on the market's own it passes this program's law
/// and reaches the loader (which Mollusk does not serve as a CPI target: the live half is the launch press's rehearsal on a validator).
#[test]
fn an_act_voted_for_one_program_is_never_carried_out_on_another() {
    let mut e = estate(true);
    let crank = e.crank;
    let _ = loader_estate(&mut e, &built::program_elf());
    let (market, warm) = (Real::declared(), Warm::declared());
    let (market_data, warm_data) = (a_program_handed_to_the_seal(&mut e, &market), a_program_handed_to_the_seal(&mut e, &warm));
    let holder = e.w.add_wallet(0);
    let on = |carry: &Instruction, data: Pubkey| {
        let mut ix = carry.clone();
        for meta in ix.accounts.iter_mut().filter(|meta| meta.pubkey == market_data) {
            meta.pubkey = data;
        }
        ix
    };
    for (n, new) in [Some(holder), None].into_iter().enumerate() {
        let act = Payload::UpgradeAuthority { origin: ROOT_UNIT, program: market, new };
        let what = if new.is_some() { "a hand" } else { "a burn" };
        let h = carry_tree(&mut e, &act, 60 + n as u64);
        let carry = execute(crank, h, &[ROOT_UNIT, 1], &act);
        assert_eq!(carry.accounts.iter().filter(|meta| meta.pubkey == market_data).count(), 1, "{what}: the client names the market's own program data, once");
        for (whose, data) in [("the $WARM program's", warm_data), ("Poleis's own", programdata_pda())] {
            e.w.run(on(&carry, data), &[refuse(PoleisError::InvalidAccount)]);
            assert_eq!(&e.w.data(&data)[13..45], upgrade_pda().as_ref(), "{what} of the market's code, carried out on {whose} program data: refused, its authority the seal's still");
        }
        assert!(!ProposalView::parse(&e.w.data(&proposal_pda(&h))).expect("its proposal").executed, "{what}: a refused carrying-out leaves the act to be carried out rightly");
        // the right one: this program's law lets it through to the loader, which Mollusk does not serve (any error but this program's own)
        let result = e.w.mollusk.process_instruction(&carry, &e.w.accounts);
        let raw = format!("{:?}", result.program_result);
        assert!(result.program_result.is_err() && !raw.contains("Custom"), "{what} of the market's code on the market's own program data: past this program's law, stopped only where Mollusk has no loader: {raw}");
    }
}

/// The shape before THE SEAL NAMES ITS PROGRAM, `[new:32]` and no program, is no act: it can be proposed and sealed like any bytes, and is
/// refused where it would act, on every program's data, so no act voted under the loose shape is ever carried out by this code.
#[test]
fn an_authority_act_that_names_no_program_is_refused() {
    let mut e = estate(true);
    let crank = e.crank;
    let _ = loader_estate(&mut e, &built::program_elf());
    let (market, warm) = (Real::declared(), Warm::declared());
    let (market_data, warm_data) = (a_program_handed_to_the_seal(&mut e, &market), a_program_handed_to_the_seal(&mut e, &warm));
    let holder = e.w.add_wallet(0);
    for (n, new) in [Some(holder), None].into_iter().enumerate() {
        let old = [&[CLASS_CODEX, poleis_treasury::CODEX_UPGRADE_AUTHORITY][..], new.unwrap_or_default().as_ref()].concat();
        let h = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &sha(&[old.as_slice()]), 70 + n as u64);
        e.w.run(propose_raw(e.alice.key, h, CLASS_CODEX, ROOT_UNIT, MANIFEST, &old, &[ROOT_UNIT, 1]), &[ok()]);
        e.w.cast(&crank, &h, 1, &pro(&e.child_founders, 8));
        e.w.run(seal(crank, h, 1), &[ok()]);
        e.w.run(mandate(crank, h, ROOT_UNIT, 1), &[ok()]);
        let mut v = pro(&e.founders, 9);
        v.extend([(&e.alice, VOTE_PRO), (&e.carol, VOTE_PRO)]);
        e.w.cast(&crank, &h, ROOT_UNIT, &v);
        e.w.warp(e.w.slot() + WINDOW + 1);
        e.w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
        assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_PASSED);
        for (program, data) in [(PROGRAM_ID, programdata_pda()), (market, market_data), (warm, warm_data)] {
            let carry = execute(crank, h, &[ROOT_UNIT, 1], &Payload::UpgradeAuthority { origin: ROOT_UNIT, program, new });
            e.w.run(carry, &[Check::err(ProgramError::InvalidInstructionData)]);
            assert_eq!(&e.w.data(&data)[13..45], upgrade_pda().as_ref(), "the old shape, on {program}'s program data: refused, its authority the seal's still");
        }
    }
}

/// THE SEAL CLOSES A BUFFER (Nov 23 2026, his "Yes, build it"): an upgrade's buffer is handed to the upgrade PDA before its vote, and one whose
/// upgrade is never carried out stays the seal's with its rent. `CODEX_CLOSE_BUFFER` names the buffer and the recipient of its lamports; only
/// the root over the whole tree passes it. Carried out on any other account, or to any other recipient, it is refused; and an act that
/// names something that is no buffer (a program's data, a program, a wallet) is refused where it would act, so no vote can close a program
/// through it. On the named buffer it passes this program's law and reaches the loader (which Mollusk does not serve: the live half is the
/// launch press's rehearsal on a validator).
#[test]
fn the_seal_closes_a_buffer_it_holds_and_never_a_program() {
    let mut e = estate(true);
    let crank = e.crank;
    let (buffer, _) = loader_estate(&mut e, &built::program_elf());
    let market_data = a_program_handed_to_the_seal(&mut e, &Real::declared());
    let (recipient, stranger) = (e.w.add_wallet(0), e.w.add_wallet(0));
    assert_eq!((&e.w.data(&buffer)[..5], &e.w.data(&buffer)[5..37]), (&[1u8, 0, 0, 0, 1][..], upgrade_pda().as_ref()), "a buffer, the seal its authority");
    let untouched = |e: &Estate, what: &str| {
        assert_eq!((&e.w.data(&buffer)[..5], &e.w.data(&market_data)[..4], &e.w.data(&programdata_pda())[..4]), (&[1u8, 0, 0, 0, 1][..], &[3u8, 0, 0, 0][..], &[3u8, 0, 0, 0][..]), "{what}: nothing closed");
        assert_eq!((e.w.lamports(&recipient), e.w.lamports(&stranger)), (0, 0), "{what}: nothing paid");
    };
    // a crew cannot: its own act, sealed by its own chairs, is refused
    let crew = Payload::CloseBuffer { origin: 1, buffer, recipient };
    let h = e.w.open(&e.child_founders[0].key, &crew, &[1], 80);
    e.w.cast(&crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(crank, h, 1), &[ok()]);
    e.w.run(execute(crank, h, &[1], &crew), &[refuse(PoleisError::Unauthorized)]);
    untouched(&e, "a crew's close");
    // the root's act over the whole tree, carried out on anything but what it names
    let close = Payload::CloseBuffer { origin: ROOT_UNIT, buffer, recipient };
    let h = carry_tree(&mut e, &close, 81);
    let carry = execute(crank, h, &[ROOT_UNIT, 1], &close);
    let with = |from: Pubkey, to: Pubkey| {
        let mut ix = carry.clone();
        for meta in ix.accounts.iter_mut().filter(|meta| meta.pubkey == from) {
            meta.pubkey = to;
        }
        ix
    };
    for (what, ix) in [
        ("the market's program data in the buffer's place", with(buffer, market_data)),
        ("Poleis's own program data in the buffer's place", with(buffer, programdata_pda())),
        ("another recipient", with(recipient, stranger)),
        ("another program as the loader", with(BPF_LOADER_UPGRADEABLE_ID, Real::declared())),
    ] {
        e.w.run(ix, &[refuse(PoleisError::InvalidAccount)]);
        untouched(&e, what);
    }
    assert!(!ProposalView::parse(&e.w.data(&proposal_pda(&h))).expect("its proposal").executed, "a refused carrying-out leaves the act to be carried out rightly");
    let result = e.w.mollusk.process_instruction(&carry, &e.w.accounts);
    let raw = format!("{:?}", result.program_result);
    assert!(result.program_result.is_err() && !raw.contains("Custom"), "the close of the named buffer to the named recipient: past this program's law, stopped only where Mollusk has no loader: {raw}");
    // an act that names what is no buffer, passed by the whole tree all the same: refused where it would act
    for (n, (what, named)) in [("the market's program data", market_data), ("Poleis's own program data", programdata_pda()), ("Poleis's program account", PROGRAM_ID), ("a wallet", stranger)].into_iter().enumerate() {
        let hostile = Payload::CloseBuffer { origin: ROOT_UNIT, buffer: named, recipient };
        let h = carry_tree(&mut e, &hostile, 82 + n as u64);
        e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &hostile), &[refuse(PoleisError::InvalidAccount)]);
        untouched(&e, what);
    }
}

/// THE SEAL UPGRADES EVERY PROGRAM (Nov 23 2026, his "Yes, close it"): after the handoff the market and the $WARM program answer to the seal as
/// Poleis does, and until this act the seal upgraded Poleis alone: the others' code moved only by handing it to a key, and nothing made the key
/// hand it back. `CODEX_UPGRADE_PROGRAM` names the program. A crew cannot pass it; the root's act for the $WARM program, carried out on the market
/// and its data, on Poleis and its own, on the $WARM program with another's data, from another buffer or to another spill, is refused and left
/// open; a payload that names no program is no act; on the $WARM program, its own data, the named buffer and spill, it passes this program's law
/// and reaches the loader (which Mollusk does not serve: the live half is the launch press's rehearsal on a validator, where such a vote
/// re-deploys the $WARM program's own code).
#[test]
fn the_seal_upgrades_the_program_its_act_names_and_no_other() {
    let mut e = estate(true);
    let crank = e.crank;
    let (buffer, spill) = loader_estate(&mut e, &built::program_elf());
    let (market, warm) = (Real::declared(), Warm::declared());
    let (market_data, warm_data) = (a_program_handed_to_the_seal(&mut e, &market), a_program_handed_to_the_seal(&mut e, &warm));
    let other = e.w.add_wallet(0);
    let held = [warm_data, market_data, programdata_pda(), buffer];
    let standing = |e: &Estate| held.map(|k| e.w.data(&k));
    let at_first = standing(&e);
    // a crew cannot
    let crew = Payload::UpgradeProgram { origin: 1, program: warm, buffer, spill };
    let h = e.w.open(&e.child_founders[0].key, &crew, &[1], 90);
    e.w.cast(&crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(crank, h, 1), &[ok()]);
    e.w.run(execute(crank, h, &[1], &crew), &[refuse(PoleisError::Unauthorized)]);
    assert_eq!(standing(&e), at_first, "a crew's upgrade: nothing moved");
    // the root's act for the $WARM program, over the whole tree, carried out on anything it does not name
    let up = Payload::UpgradeProgram { origin: ROOT_UNIT, program: warm, buffer, spill };
    let h = carry_tree(&mut e, &up, 91);
    let carry = execute(crank, h, &[ROOT_UNIT, 1], &up);
    let swap = |pairs: &[(Pubkey, Pubkey)]| {
        let mut ix = carry.clone();
        for meta in ix.accounts.iter_mut() {
            if let Some((_, to)) = pairs.iter().find(|(from, _)| *from == meta.pubkey) {
                meta.pubkey = *to;
            }
        }
        ix
    };
    for (what, ix) in [
        ("on the market and its data", swap(&[(warm, market), (warm_data, market_data)])),
        ("on Poleis and its own data", swap(&[(warm, PROGRAM_ID), (warm_data, programdata_pda())])),
        ("on the $WARM program with the market's data", swap(&[(warm_data, market_data)])),
        ("from another buffer", swap(&[(buffer, other)])),
        ("to another spill", swap(&[(spill, other)])),
    ] {
        e.w.run(ix, &[refuse(PoleisError::InvalidAccount)]);
        assert_eq!(standing(&e), at_first, "the upgrade voted for the $WARM program, carried out {what}: refused, nothing moved");
    }
    assert!(!ProposalView::parse(&e.w.data(&proposal_pda(&h))).expect("its proposal").executed, "a refused carrying-out leaves the act to be carried out rightly");
    for meta in &carry.accounts {
        e.w.touch(meta.pubkey);
    }
    let result = e.w.mollusk.process_instruction(&carry, &e.w.accounts);
    let raw = format!("{:?}", result.program_result);
    assert!(result.program_result.is_err() && !raw.contains("Custom"), "the upgrade of the $WARM program on its own accounts: past this program's law, stopped only where Mollusk has no loader: {raw}");
    // the same kind with no program named is no act, sealed by the whole tree all the same
    let unnamed = [&[CLASS_CODEX, poleis_treasury::CODEX_UPGRADE_PROGRAM][..], buffer.as_ref(), spill.as_ref()].concat();
    let h = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &sha(&[unnamed.as_slice()]), 92);
    e.w.run(propose_raw(e.alice.key, h, CLASS_CODEX, ROOT_UNIT, MANIFEST, &unnamed, &[ROOT_UNIT, 1]), &[ok()]);
    e.w.cast(&crank, &h, 1, &pro(&e.child_founders, 8));
    e.w.run(seal(crank, h, 1), &[ok()]);
    e.w.run(mandate(crank, h, ROOT_UNIT, 1), &[ok()]);
    let mut v = pro(&e.founders, 9);
    v.extend([(&e.alice, VOTE_PRO), (&e.carol, VOTE_PRO)]);
    e.w.cast(&crank, &h, ROOT_UNIT, &v);
    e.w.warp(e.w.slot() + WINDOW * CODEX_WINDOW_FACTOR + 1);
    e.w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &up), &[Check::err(ProgramError::InvalidInstructionData)]);
    assert_eq!(standing(&e), at_first, "an upgrade that names no program: refused, nothing moved");
}

#[test]
fn a_lot_opens_by_signal_and_a_weight_is_set_by_signal() {
    let mut e = estate(false);
    let (crank, alice, bob, carol) = (e.crank, e.alice.key, e.bob.key, e.carol.key);
    // a trace lot opens for the unit by SIGNAL, in council mode
    let now = e.w.slot();
    let lot = LotSpec { auction_id: 7, unit: ROOT_UNIT, pool_root: [1u8; 32], manifest: [2u8; 32], commit_end: now + 6_000, reveal_end: now + 7_000, reserve: 1, deposit: 1_000_000 };
    let open = Payload::OpenAuction { origin: ROOT_UNIT, lot };
    let h = carry(&mut e, &open, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &open), &[ok()]);
    let a = AuctionView::parse(&e.w.data(&auction_pda(7))).expect("auction");
    assert_eq!((a.unit, a.reserve, a.deposit, a.settled), (ROOT_UNIT, 1, 1_000_000, false));

    // carol's one seat weighs three by SIGNAL; her voice counts three from then on
    let wt = Payload::Weight { origin: ROOT_UNIT, holder: key(&carol), kind: HOLDER_KEY, weight: 3 };
    let h = carry(&mut e, &wt, 2);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &wt), &[ok()]);
    let r = e.w.registry(ROOT_UNIT);
    let c = r.holders.iter().find(|x| x.holder == key(&carol)).expect("carol");
    assert_eq!((c.seats, c.weight), (1, 3));
    // the floor now weighs 3 + 2 + 3 = 8: carol and bob alone carry it (5 × 2 > 8)
    let name = Payload::Name { origin: ROOT_UNIT, name: b"WEIGHED".to_vec(), desc: Vec::new() };
    let h = e.w.open(&alice, &name, &[ROOT_UNIT], 3);
    let mut v = pro(&e.founders, 7);
    v.extend([(&e.carol, VOTE_PRO), (&e.bob, VOTE_PRO)]);
    e.w.cast(&crank, &h, ROOT_UNIT, &v);
    e.w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = e.w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_pro, s.state), (5, STATE_PASSED));
    let _ = (alice, bob);
}

#[test]
fn a_weight_has_a_ceiling_so_no_seal_overflows() {
    let mut e = estate(false);
    let (crank, carol) = (e.crank, e.carol.key);
    // one past the ceiling passes its seals and is refused where it would act
    let over = Payload::Weight { origin: ROOT_UNIT, holder: key(&carol), kind: HOLDER_KEY, weight: poleis_treasury::MAX_WEIGHT + 1 };
    let h = carry(&mut e, &over, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &over), &[refuse(PoleisError::InvalidAmount)]);
    // the ceiling itself is set
    let at = Payload::Weight { origin: ROOT_UNIT, holder: key(&carol), kind: HOLDER_KEY, weight: poleis_treasury::MAX_WEIGHT };
    let h = carry(&mut e, &at, 2);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &at), &[ok()]);
    let r = e.w.registry(ROOT_UNIT);
    let c = r.holders.iter().find(|x| x.holder == key(&carol)).expect("carol");
    assert_eq!((c.seats, c.weight), (1, poleis_treasury::MAX_WEIGHT));
    // and the unit still seals: no weight sum overflows (carry asserts PASSED)
    let name = Payload::Name { origin: ROOT_UNIT, name: b"HEAVY".to_vec(), desc: Vec::new() };
    let h = carry(&mut e, &name, 3);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &name), &[ok()]);
}

#[test]
fn a_society_spends_the_default_mint_freely_and_lists_the_rest_by_codex() {
    let mut e = estate(false);
    let (crank, payer) = (e.crank, e.payer);
    assert_eq!(e.w.registry(ROOT_UNIT).default_mint, e.usdc, "every unit inherits the default mint");
    assert_eq!(e.w.config().default_mint, e.usdc);

    // a second mint: receiving is free, spending needs a listing
    let other = e.w.mint();
    e.w.run(create_vault(payer, ROOT_UNIT, other, TOKEN_PROGRAM_ID), &[ok()]);
    let vault = vault_pda(ROOT_UNIT, &other);
    e.w.set_tokens(&vault, 1_000);
    let plan = Payload::Plan { origin: ROOT_UNIT, cap: 10 * SOL, period: 1_000_000, token_cap: 600 };
    let hp = carry(&mut e, &plan, 1);
    e.w.run(execute(crank, hp, &[ROOT_UNIT], &plan), &[ok()]);
    let stranger = e.w.add_wallet(0);
    let ta = e.w.token_account(&other, &stranger, 0);
    let pay = Payload::TokenBudget { origin: ROOT_UNIT, mint: other, address: ta, amount: 200, token_program: TOKEN_PROGRAM_ID };
    let h = carry(&mut e, &pay, 2);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &pay), &[refuse(PoleisError::MintNotAllowed)], );

    // the society lists it with its own cap, in its own units
    let allow = Payload::AllowMint { origin: ROOT_UNIT, mint: other, cap: 250, payer: crank };
    let ha = carry(&mut e, &allow, 3);
    e.w.run(execute(crank, ha, &[ROOT_UNIT], &allow), &[ok()]);
    let tp = TokenPlanView::parse(&e.w.data(&poleis_client::token_plan_pda(ROOT_UNIT, &other))).expect("token plan");
    assert_eq!((tp.unit, tp.mint, tp.cap, tp.spent, tp.payer), (ROOT_UNIT, other, 250, 0, crank));
    e.w.run(execute(crank, h, &[ROOT_UNIT], &pay), &[ok()]);
    assert_eq!(e.w.tokens(&ta), 200);
    assert_eq!(e.w.registry(ROOT_UNIT).token_plan_spent, 0, "the default mint's counter is untouched");
    let more = Payload::TokenBudget { origin: ROOT_UNIT, mint: other, address: ta, amount: 100, token_program: TOKEN_PROGRAM_ID };
    let hm = carry(&mut e, &more, 4);
    e.w.run(execute(crank, hm, &[ROOT_UNIT], &more), &[refuse(PoleisError::TokenPlanBreached)]);
    e.w.warp(e.w.slot() + 1_000_000);
    e.w.run(execute(crank, hm, &[ROOT_UNIT], &more), &[ok()]);
    assert_eq!(e.w.tokens(&ta), 300);

    // the default mint cannot be listed, and a listing withdrawn closes the plan
    let redundant = Payload::AllowMint { origin: ROOT_UNIT, mint: e.usdc, cap: 1, payer: crank };
    let hr = carry(&mut e, &redundant, 5);
    e.w.run(execute(crank, hr, &[ROOT_UNIT], &redundant), &[refuse(PoleisError::InvalidAccount)]);
    let withdraw = Payload::AllowMint { origin: ROOT_UNIT, mint: other, cap: 0, payer: crank };
    let hw = carry(&mut e, &withdraw, 6);
    let c0 = e.w.lamports(&crank);
    e.w.run(execute(crank, hw, &[ROOT_UNIT], &withdraw), &[ok()]);
    assert!(e.w.lamports(&crank) > c0, "the plan's rent came back");
    assert!(e.w.data(&poleis_client::token_plan_pda(ROOT_UNIT, &other)).is_empty());
    let again = Payload::TokenBudget { origin: ROOT_UNIT, mint: other, address: ta, amount: 1, token_program: TOKEN_PROGRAM_ID };
    let hg = carry(&mut e, &again, 7);
    e.w.run(execute(crank, hg, &[ROOT_UNIT], &again), &[refuse(PoleisError::MintNotAllowed)]);
}

// ================= clause XII: one program, many trees =================

/// Any wallet founds a tree of its own on the same program, under a root
/// id nobody has taken: its config, its council, its registries carry that
/// root; its decisions pass under its own council and never under another;
/// and only tree 0 touches the code.
#[test]
fn one_program_hosts_many_trees_and_only_the_founding_tree_touches_the_code() {
    let mut e = estate(true);
    let crank = e.crank;
    let now = e.w.slot();

    // a stranger founds tree 7 while tree 0 is already a council: eleven
    // chairs, so chair 11 stays free for a crew
    let founder = e.w.add_wallet(100 * SOL);
    let chairs: Vec<Voter> = (0..11).map(|_| e.w.voter(SOL)).collect();
    let g = GenesisSpec {
        charter: [7u8; 32],
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        window: WINDOW,
        rung: 1,
        reserve: 1_000_000,
        deposit: 5_000_000,
        commit_end: now + 100,
        reveal_end: now + 200,
        founders: chairs.iter().map(|c| c.key).collect(),
        name: b"THE SEVENTH".to_vec(),
        desc: b"a tree of its own".to_vec(),
    };
    // the founding is two instructions in one transaction; the root id is taken by the first
    e.w.run_tx(&[init_config_of(7, founder, Pubkey::default()), open_genesis_of(7, founder, &g)], &founder, Expect::Ok);
    e.w.run(init_config_of(7, e.payer, Pubkey::default()), &[Check::err(ProgramError::AccountAlreadyInitialized)]);
    let cfg7 = ConfigView::parse(&e.w.data(&config_pda_of(7))).expect("tree 7 has a config");
    assert_eq!((cfg7.root, cfg7.mode, cfg7.authority), (7, MODE_GENESIS, founder));
    assert_eq!(e.w.config().mode, MODE_COUNCIL, "tree 0 is untouched");
    let r7 = e.w.registry(7);
    assert_eq!((r7.root, r7.level, r7.parent, r7.name.as_str()), (7, 0, 7, "THE SEVENTH"));
    assert_eq!(r7.capacity, 0, "born with its head and chairs, no holder rows");
    assert_eq!(e.w.treasury(7).unit, 7, "its treasury was born in the same act");

    // a crew of tree 7 carries root 7; tree 0's config cannot found into tree 7
    let crew_spec = FoundSpec {
        child: 8,
        chair: 11,
        charter: [8u8; 32],
        window: WINDOW,
        plan_cap: SOL,
        plan_period: 1_000_000,
        founders: chairs.iter().take(8).map(|c| c.key).collect(),
        name: b"CREW 8".to_vec(),
        desc: Vec::new(),
    };
    e.w.run(found_child(e.payer, 7, &crew_spec), &[refuse(PoleisError::Unauthorized)]);
    e.w.run(found_child_of(7, founder, 7, &crew_spec), &[ok()]);
    assert_eq!(e.w.registry(8).root, 7, "a child carries its tree's root");

    // one seat sold, the sale settled, tree 7 ratified under its own council
    let buyer = e.w.voter(SOL);
    let salt = [5u8; 32];
    e.w.run(commit_seat_bid(buyer.key, 7, 0, 0, seat_commitment(1_000_000, 0, &salt, &buyer.key)), &[ok()]);
    e.w.warp(now + 100);
    e.w.run(reveal_seat_bid(buyer.key, 7, 0, 0, 1_000_000, salt), &[ok()]);
    e.w.warp(now + 200);
    e.w.run(settle_seats(crank, 7, 0), &[ok()]);
    e.w.run(claim_seat(buyer.key, 7, 0, 0, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.registry(7).capacity, 1, "the claim grew the registry by one row");
    e.w.run(ratify_genesis(crank), &[refuse(PoleisError::InvalidMode)]);
    e.w.run(ratify_genesis_of(7, crank), &[ok()]);
    let cfg7 = ConfigView::parse(&e.w.data(&config_pda_of(7))).expect("config");
    assert_eq!((cfg7.mode, cfg7.authority), (MODE_COUNCIL, council_pda_of(7)), "its own council, not tree 0's");
    assert_ne!(council_pda_of(7), council_pda(), "one authority per tree");

    // a decision in tree 7 passes under tree 7's config and under no other:
    // a CODEX act of the root over its whole tree, the crew speaking first
    let rename = Payload::Name { origin: 7, name: b"THE SEVENTH, SEALED".to_vec(), desc: Vec::new() };
    let h = proposal_hash(rename.class(), rename.origin(), &MANIFEST, &rename.hash(), 1);
    e.w.run(propose(chairs[0].key, h, MANIFEST, &rename, &[7, 8]), &[refuse(PoleisError::UnitMismatch)]);
    e.w.run(propose_of(7, chairs[0].key, h, MANIFEST, &rename, &[7, 8]), &[ok()]);
    e.w.cast(&crank, &h, 8, &pro(&chairs, 7));
    e.w.run(seal(crank, h, 8), &[ok()]);
    e.w.run(mandate(crank, h, 7, 8), &[ok()]);
    let mut v = pro(&chairs, 7);
    v.push((&buyer, VOTE_PRO));
    e.w.cast(&crank, &h, 7, &v);
    e.w.warp(e.w.slot() + WINDOW * CODEX_WINDOW_FACTOR + 1);
    e.w.run(seal(crank, h, 7), &[ok()]);
    assert_eq!(e.w.session(&h, 7).state, STATE_PASSED);
    e.w.run(execute_of(7, crank, h, &[7, 8], &rename), &[ok()]);
    assert_eq!(e.w.registry(7).name, "THE SEVENTH, SEALED");

    // a decision may not span trees: unit 8 is not a descendant of unit 0
    let span = Payload::Name { origin: ROOT_UNIT, name: b"NO".to_vec(), desc: Vec::new() };
    let h = proposal_hash(span.class(), span.origin(), &MANIFEST, &span.hash(), 2);
    e.w.run(propose(e.alice.key, h, MANIFEST, &span, &[ROOT_UNIT, 1, 8]), &[refuse(PoleisError::NotDescendant)]);

    // and only tree 0 touches the code: tree 7's root is refused like any crew
    let elf = built::program_elf();
    let (buffer, spill) = loader_estate(&mut e, &elf);
    let up = Payload::Upgrade { origin: 7, buffer, spill };
    let h = proposal_hash(up.class(), up.origin(), &MANIFEST, &up.hash(), 3);
    e.w.run(propose_of(7, chairs[0].key, h, MANIFEST, &up, &[7, 8]), &[ok()]);
    e.w.cast(&crank, &h, 8, &pro(&chairs, 7));
    e.w.run(seal(crank, h, 8), &[ok()]);
    e.w.run(mandate(crank, h, 7, 8), &[ok()]);
    let mut v = pro(&chairs, 7);
    v.push((&buyer, VOTE_PRO));
    e.w.cast(&crank, &h, 7, &v);
    e.w.warp(e.w.slot() + WINDOW * CODEX_WINDOW_FACTOR + 1);
    e.w.run(seal(crank, h, 7), &[ok()]);
    e.w.run(execute_of(7, crank, h, &[7, 8], &up), &[refuse(PoleisError::Unauthorized)]);
}

#[test]
fn unit_zero_is_only_ever_a_root() {
    // a program id where tree 0 is not founded: a stranger founds tree 7
    let mut w = World::new();
    let now = w.slot();
    let founder = w.add_wallet(100 * SOL);
    let chairs: Vec<Voter> = (0..11).map(|_| w.voter(SOL)).collect();
    let g = GenesisSpec {
        charter: [7u8; 32],
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        window: WINDOW,
        rung: 1,
        reserve: 1_000_000,
        deposit: 5_000_000,
        commit_end: now + 100,
        reveal_end: now + 200,
        founders: chairs.iter().map(|c| c.key).collect(),
        name: b"THE SEVENTH".to_vec(),
        desc: Vec::new(),
    };
    w.run_tx(&[init_config_of(7, founder, Pubkey::default()), open_genesis_of(7, founder, &g)], &founder, Expect::Ok);
    assert!(w.data(&registry_pda(ROOT_UNIT)).is_empty(), "no unit 0 on this program id");
    let spec = |child: u64| FoundSpec {
        child,
        chair: 11,
        charter: [8u8; 32],
        window: WINDOW,
        plan_cap: SOL,
        plan_period: 1_000_000,
        founders: chairs.iter().take(8).map(|c| c.key).collect(),
        name: b"CREW".to_vec(),
        desc: Vec::new(),
    };
    // its child may not take the founding root's id
    w.run(found_child_of(7, founder, 7, &spec(ROOT_UNIT)), &[refuse(PoleisError::UnitMismatch)]);
    assert!(w.data(&registry_pda(ROOT_UNIT)).is_empty(), "no unit 0 was born");
    // the same founding under any other id goes through
    w.run(found_child_of(7, founder, 7, &spec(8)), &[ok()]);
    assert_eq!(w.registry(8).root, 7);
}

// ================= the fund: the polis of poleis =================

#[test]
fn the_fund_takes_its_share_seats_its_chairs_and_pays_the_current_option() {
    let mut e = estate(true);
    let (crank, alice, bob) = (e.crank, e.alice.key, e.bob.key);

    // the root polis constitutes the fund over the whole tree: 250 bps of every
    // fee line, a cycle of eight windows, a payout between 20% and 80% of the
    // fund per cycle, an option fee of a tenth of a SOL, and a chair quorum
    // of one while the network is one polis
    let fund = Payload::Fund { origin: ROOT_UNIT, fee_bps: 250, cycle_slots: 8 * WINDOW, range_lo: 2000, range_hi: 8000, option_fee: SOL / 10, chair_quorum: 1 };
    let h = carry_tree(&mut e, &fund, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &fund), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).expect("the fund");
    assert_eq!((f.fee_bps, f.cycle_slots, f.range_lo, f.range_hi, f.option_fee, f.chair_quorum, f.cycle_seq), (250, 8 * WINDOW, 2000, 8000, SOL / 10, 1, 0));
    assert!(f.chairs.iter().all(|c| c.root == NO_ROOT), "born with every chair empty");
    let cycle_start = f.cycle_start;
    let rent = e.w.rent(Fund::SIZE);

    // a market purchase pays the share: alice lists a seat at a tenth of a SOL, carol buys it
    // (a voter, so the floor's majority still carries the decisions below)
    e.w.warp(e.w.slot() + WINDOW + 1);
    e.w.run(list_seats(alice, ROOT_UNIT, 1, 1, SOL / 10), &[ok()]);
    let a0 = e.w.lamports(&alice);
    e.w.run(buy_seats(e.carol.key, alice, ROOT_UNIT, 1, 1, SOL / 10, ROOT_UNIT), &[ok()]);
    let fee = SOL / 10 * 250 / 10_000;
    assert_eq!(e.w.lamports(&fund_pda()) - rent, fee, "250 bps of the price lands in the fund");
    assert_eq!(e.w.lamports(&alice) - a0, SOL / 10 - fee + e.w.rent(67), "the seller gets the rest, and her sold-out listing's rent home");
    let t = TallyView::parse(&e.w.data(&tally_pda(ROOT_UNIT))).expect("the tally");
    assert_eq!((t.root, t.cycle, t.fees, t.lifetime), (ROOT_UNIT, 0, fee, fee));

    // paying is racing: the root took a chair the moment its fee landed; the explicit crank only refreshes it
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.chairs[0].root, f.chairs[0].fees), (ROOT_UNIT, fee), "the chair seats itself on the fee");
    e.w.run(seat_chair(crank, ROOT_UNIT), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.chairs[0].root, f.chairs[0].fees), (ROOT_UNIT, fee));
    assert_eq!(f.chairs.iter().filter(|c| c.root != NO_ROOT).count(), 1);

    // an option: a lab's compute account under a manifest; anyone proposes it, the fund keeps the fee
    let lab = e.w.add_wallet(0);
    let manifest = [3u8; 32];
    let oh = option_hash(&lab, &manifest);
    let proposer = e.w.add_wallet(SOL);
    e.w.run(propose_option(proposer, lab, manifest), &[ok()]);
    assert_eq!(e.w.lamports(&fund_pda()) - rent, fee + SOL / 10, "the option fee lands in the fund");
    let o = OptionView::parse(&e.w.data(&option_pda(&oh))).expect("the option");
    assert_eq!((o.recipient, o.proposer, o.vetted, o.floor_stake), (lab, proposer, false, 0));
    // unvetted, it takes no stake
    e.w.run(stake(alice, oh, ROOT_UNIT, 5000), &[refuse(PoleisError::NotVetted)]);

    // the root vets it under its double seal
    let vet = Payload::VetOption { origin: ROOT_UNIT, hash: oh };
    let hv = carry_tree(&mut e, &vet, 2);
    e.w.run(execute(crank, hv, &[ROOT_UNIT, 1], &vet), &[ok()]);
    assert!(OptionView::parse(&e.w.data(&option_pda(&oh))).unwrap().vetted);

    // the floor stakes: alice her two seats at 50%, bob his two at 30%; a stranger cannot
    e.w.run(stake(alice, oh, ROOT_UNIT, 5000), &[ok()]);
    e.w.run(stake(bob, oh, ROOT_UNIT, 3000), &[ok()]);
    e.w.run(stake(proposer, oh, ROOT_UNIT, 5000), &[refuse(PoleisError::NotHolder)]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.staked_total, f.wp_weight, f.wp_sum, f.payout_bps()), (4, 4, 16_000, 4000));
    assert_eq!(OptionView::parse(&e.w.data(&option_pda(&oh))).unwrap().floor_stake, 4);
    assert_eq!(f.current, [0u8; 32], "no chair backs it yet: nothing is current");

    // the root backs it: quorum one of one chair, and more than half of all stake: current
    let back = Payload::BackOption { origin: ROOT_UNIT, hash: oh };
    let hb = carry_tree(&mut e, &back, 3);
    e.w.run(execute(crank, hb, &[ROOT_UNIT, 1], &back), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!(f.chairs[0].backing, oh);
    assert_eq!(f.current, oh, "first to the line is current");

    // bob restakes at 10%: the weighted value moves, the option stays current
    e.w.run(stake(bob, oh, ROOT_UNIT, 1000), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.staked_total, f.wp_sum, f.payout_bps()), (4, 12_000, 3000));

    // the cycle must run its slots; then it rolls, and the grant is owed to the current option's recipient and nobody else
    e.w.run(settle_cycle(crank), &[refuse(PoleisError::CycleOpen)]);
    e.w.run(claim_grant(crank, lab), &[refuse(PoleisError::NothingOwed)]);
    e.w.warp(cycle_start + 8 * WINDOW + 1);
    let free = e.w.lamports(&fund_pda()) - rent;
    let grant = free * 3000 / 10_000;
    e.w.run(settle_cycle(crank), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.cycle_seq, f.owed, f.owed_to, f.total_out), (1, grant, lab, 0), "30% of the fund is booked to the lab's compute, not yet paid");
    assert_eq!(e.w.lamports(&fund_pda()) - rent, free, "the money waits in the fund");
    assert!(f.chairs.iter().all(|c| c.root == NO_ROOT), "the race for the chairs restarts");
    assert_eq!(f.current, oh, "the current option stands until another crosses the line");
    e.w.run(settle_cycle(crank), &[refuse(PoleisError::CycleOpen)]);
    // the grant is a pull: the proposer cannot claim it; the lab, or anyone for the lab, can, once
    e.w.run(claim_grant(crank, proposer), &[refuse(PoleisError::NothingOwed)]);
    e.w.run(claim_grant(crank, lab), &[ok()]);
    assert_eq!(e.w.lamports(&lab), grant, "30% of the fund goes to the lab's compute");
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.owed, f.total_out), (0, grant));
    e.w.run(claim_grant(crank, lab), &[refuse(PoleisError::NothingOwed)]);

    // rolls on touch: a cycle later, a stake ends the cycle by itself and books the next grant; nobody cranked
    e.w.warp(e.w.slot() + 8 * WINDOW + 1);
    let free2 = e.w.lamports(&fund_pda()) - rent;
    e.w.run(stake(bob, oh, ROOT_UNIT, 1000), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.cycle_seq, f.owed, f.owed_to), (2, free2 * 3000 / 10_000, lab), "the stake rolled the cycle and booked the grant");
    e.w.run(claim_grant(crank, lab), &[ok()]);
    assert_eq!(e.w.lamports(&lab), grant + free2 * 3000 / 10_000);

    // a second option cannot overtake without the chairs: bob moves his stake to it, nothing changes
    let lab2 = e.w.add_wallet(0);
    let oh2 = option_hash(&lab2, &manifest);
    e.w.run(propose_option(proposer, lab2, manifest), &[ok()]);
    let vet2 = Payload::VetOption { origin: ROOT_UNIT, hash: oh2 };
    let hv2 = carry_tree(&mut e, &vet2, 4);
    e.w.run(execute(crank, hv2, &[ROOT_UNIT, 1], &vet2), &[ok()]);
    e.w.run(unstake(bob, oh, ROOT_UNIT), &[ok()]);
    e.w.run(stake(bob, oh2, ROOT_UNIT, 8000), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.staked_total, f.wp_weight), (4, 4));
    assert_eq!(f.current, oh, "no chair backs the second option");
}

#[test]
fn the_fund_never_owes_itself_and_an_older_fund_is_never_no_fund() {
    let mut e = estate(false);
    let (crank, alice) = (e.crank, e.alice.key);
    let fund = Payload::Fund { origin: ROOT_UNIT, fee_bps: 250, cycle_slots: 8 * WINDOW, range_lo: 2000, range_hi: 8000, option_fee: SOL / 10, chair_quorum: 1 };
    let h = carry(&mut e, &fund, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &fund), &[ok()]);

    // an option whose recipient is the fund itself is refused; any other is taken
    let proposer = e.w.add_wallet(SOL);
    let manifest = [3u8; 32];
    e.w.run(propose_option(proposer, fund_pda(), manifest), &[refuse(PoleisError::InvalidAccount)]);
    let lab = e.w.add_wallet(0);
    e.w.run(propose_option(proposer, lab, manifest), &[ok()]);

    // an account at the fund's address that this program owns at another size
    // is an older fund: a fee site refuses rather than charge nothing
    let old = Account { lamports: e.w.rent(Fund::SIZE - 8), data: vec![0u8; Fund::SIZE - 8], owner: PROGRAM_ID, executable: false, rent_epoch: 0 };
    let at = fund_pda();
    match e.w.accounts.iter_mut().find(|(k, _)| *k == at) {
        Some(slot) => slot.1 = old,
        None => e.w.accounts.push((at, old)),
    }
    e.w.warp(e.w.slot() + WINDOW + 1);
    e.w.run(list_seats(alice, ROOT_UNIT, 1, 1, SOL / 10), &[ok()]);
    let buyer = e.w.add_wallet(SOL);
    e.w.run(buy_seats(buyer, alice, ROOT_UNIT, 1, 1, SOL / 10, ROOT_UNIT), &[refuse(PoleisError::LayoutVersion)]);
    assert_eq!(ListingView::parse(&e.w.data(&listing_pda(ROOT_UNIT, &alice, 1))).unwrap().count, 1, "nothing was sold");
}

// ================= AN OLDER FUND HAS ITS ROAD (Nov 24 2026) =================

/// Put `a` at `at` in the world, whatever stood there.
fn lay(w: &mut World, at: Pubkey, a: Account) {
    match w.accounts.iter_mut().find(|(k, _)| *k == at) {
        Some(slot) => slot.1 = a,
        None => w.accounts.push((at, a)),
    }
}
/// The fund as Sept 6 2026's build keeps it: this fund's first `Fund::OLD_SIZE` bytes and nothing after, at the older size's rent and
/// the same free balance. Returns the fund as it stood (taken before any cycle rolled, so it owed nothing: the older fund never did).
fn make_older(w: &mut World) -> Vec<u8> {
    let at = fund_pda();
    let full = w.data(&at);
    assert_eq!(full.len(), Fund::SIZE);
    assert_eq!(FundView::parse(&full).unwrap().owed, 0, "the older fund owed nothing");
    let free = w.lamports(&at) - w.rent(Fund::SIZE);
    let lamports = w.rent(Fund::OLD_SIZE) + free;
    lay(w, at, Account { lamports, data: full[..Fund::OLD_SIZE].to_vec(), owner: PROGRAM_ID, executable: false, rent_epoch: 0 });
    full
}

#[test]
fn an_older_fund_grows_in_place_once_with_its_current_option_and_every_fee_site_and_grant_work_on_it() {
    let mut e = estate(true);
    let (crank, alice, bob) = (e.crank, e.alice.key, e.bob.key);
    assert_eq!((Fund::OLD_SIZE, Fund::SIZE), (706, 778), "the older fund is 706 bytes, this one 778");
    let fund = Payload::Fund { origin: ROOT_UNIT, fee_bps: 250, cycle_slots: 8 * WINDOW, range_lo: 2000, range_hi: 8000, option_fee: SOL / 10, chair_quorum: 1 };
    let h = carry_tree(&mut e, &fund, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT, 1], &fund), &[ok()]);
    let cycle_start = FundView::parse(&e.w.data(&fund_pda())).unwrap().cycle_start;
    // a fee, an option made current by the chair and the floor, as the older fund on devnet holds one
    e.w.warp(e.w.slot() + WINDOW + 1);
    e.w.run(list_seats(alice, ROOT_UNIT, 1, 1, SOL / 10), &[ok()]);
    e.w.run(buy_seats(e.carol.key, alice, ROOT_UNIT, 1, 1, SOL / 10, ROOT_UNIT), &[ok()]);
    let lab = e.w.add_wallet(0);
    let manifest = [3u8; 32];
    let oh = option_hash(&lab, &manifest);
    let proposer = e.w.add_wallet(SOL);
    e.w.run(propose_option(proposer, lab, manifest), &[ok()]);
    let vet = Payload::VetOption { origin: ROOT_UNIT, hash: oh };
    let hv = carry_tree(&mut e, &vet, 2);
    e.w.run(execute(crank, hv, &[ROOT_UNIT, 1], &vet), &[ok()]);
    e.w.run(stake(alice, oh, ROOT_UNIT, 5000), &[ok()]);
    let back = Payload::BackOption { origin: ROOT_UNIT, hash: oh };
    let hb = carry_tree(&mut e, &back, 3);
    e.w.run(execute(crank, hb, &[ROOT_UNIT, 1], &back), &[ok()]);
    // a second option, proposed and never current
    let lab2 = e.w.add_wallet(0);
    let oh2 = option_hash(&lab2, &manifest);
    e.w.run(propose_option(proposer, lab2, manifest), &[ok()]);
    assert_eq!(FundView::parse(&e.w.data(&fund_pda())).unwrap().current, oh);

    // the program upgraded over the older fund: every fee site refuses it, rather than charge nothing
    let full = make_older(&mut e.w);
    let older = e.w.data(&fund_pda());
    let f0 = e.w.lamports(&fund_pda());
    e.w.run(list_seats(bob, ROOT_UNIT, 1, 1, SOL / 10), &[ok()]);
    let buyer = e.w.add_wallet(SOL);
    e.w.run(buy_seats(buyer, bob, ROOT_UNIT, 1, 1, SOL / 10, ROOT_UNIT), &[refuse(PoleisError::LayoutVersion)]);

    // its road: refused without the current option, or with another option; neither changed a byte
    let payer = e.w.add_wallet(SOL);
    e.w.run(grow_fund(payer, None), &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    let mut wrong = grow_fund(payer, Some(oh));
    wrong.accounts[3].pubkey = option_pda(&oh2);
    e.w.run(wrong, &[refuse(PoleisError::InvalidAccount)]);
    assert_eq!((e.w.data(&fund_pda()), e.w.lamports(&fund_pda()), e.w.lamports(&payer)), (older.clone(), f0, SOL), "a refusal changes nothing");

    // grown: byte for byte the fund this program would have kept, the current option's recipient copied from its own account,
    // nothing owed; the payer paid the added bytes' rent and only that, so the free balance, and every grant, is what it was
    e.w.run(grow_fund(payer, Some(oh)), &[ok()]);
    let grown = e.w.data(&fund_pda());
    assert_eq!(&grown[..Fund::OLD_SIZE], &older[..], "every old field in its place");
    assert_eq!(grown, full, "the grown fund is the fund this layout keeps");
    let g = FundView::parse(&grown).unwrap();
    assert_eq!((g.current, g.current_recipient, g.owed, g.owed_to), (oh, lab, 0, Pubkey::default()));
    let added = e.w.rent(Fund::SIZE) - e.w.rent(Fund::OLD_SIZE);
    assert_eq!(e.w.lamports(&fund_pda()), f0 + added, "the fund gained the added bytes' rent; nothing left it");
    assert_eq!(e.w.lamports(&payer), SOL - added, "the payer paid the added bytes' rent");
    assert_eq!(e.w.lamports(&fund_pda()) - e.w.rent(Fund::SIZE), f0 - e.w.rent(Fund::OLD_SIZE), "the free balance is what it was");

    // once: a grown fund is not an older one
    e.w.run(grow_fund(payer, Some(oh)), &[refuse(PoleisError::NotAnOlderFund)]);
    e.w.run(grow_fund(payer, None), &[refuse(PoleisError::NotAnOlderFund)]);

    // every fee site takes its share again: the same purchase lands, 250 bps to the fund
    let before = e.w.lamports(&fund_pda());
    e.w.run(buy_seats(buyer, bob, ROOT_UNIT, 1, 1, SOL / 10, ROOT_UNIT), &[ok()]);
    assert_eq!(e.w.lamports(&fund_pda()) - before, SOL / 10 * 250 / 10_000);

    // and the cycle rolls on the grown fund: the grant is booked to the current option's recipient, and its pull pays it
    e.w.warp(cycle_start + 8 * WINDOW + 1);
    let free = e.w.lamports(&fund_pda()) - e.w.rent(Fund::SIZE);
    e.w.run(settle_cycle(crank), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.owed, f.owed_to), (free * 5000 / 10_000, lab), "half the free balance, booked to the lab");
    e.w.run(claim_grant(crank, lab), &[ok()]);
    assert_eq!(e.w.lamports(&lab), free * 5000 / 10_000);
}

#[test]
fn an_older_fund_with_nothing_current_grows_and_the_root_constitutes_it_again() {
    let mut e = estate(false);
    let crank = e.crank;
    let fund = Payload::Fund { origin: ROOT_UNIT, fee_bps: 250, cycle_slots: 8 * WINDOW, range_lo: 2000, range_hi: 8000, option_fee: SOL / 10, chair_quorum: 1 };
    let h = carry(&mut e, &fund, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &fund), &[ok()]);
    let full = make_older(&mut e.w);

    // the fund's own constitution cannot repair it: CODEX Fund refuses the older fund too
    let again = Payload::Fund { origin: ROOT_UNIT, fee_bps: 300, cycle_slots: 8 * WINDOW, range_lo: 1000, range_hi: 9000, option_fee: SOL / 20, chair_quorum: 1 };
    let ha = carry(&mut e, &again, 2);
    e.w.run(execute(crank, ha, &[ROOT_UNIT], &again), &[refuse(PoleisError::LayoutVersion)]);

    // nothing current: no option to hand, and the recipient stays empty
    let payer = e.w.add_wallet(SOL);
    e.w.run(grow_fund(payer, None), &[ok()]);
    assert_eq!(e.w.data(&fund_pda()), full);
    let g = FundView::parse(&full).unwrap();
    assert_eq!((g.current, g.current_recipient, g.owed, g.owed_to), ([0u8; 32], Pubkey::default(), 0, Pubkey::default()));

    // and the same decision now executes on it: the parameters change, the balances and the cycle do not
    e.w.run(execute(crank, ha, &[ROOT_UNIT], &again), &[ok()]);
    let f = FundView::parse(&e.w.data(&fund_pda())).unwrap();
    assert_eq!((f.fee_bps, f.range_lo, f.range_hi, f.option_fee), (300, 1000, 9000, SOL / 20));
    assert_eq!((f.cycle_start, f.cycle_seq, f.total_in, f.total_out), (g.cycle_start, g.cycle_seq, g.total_in, g.total_out));
}

#[test]
fn a_growth_is_refused_on_anything_but_the_older_fund_and_changes_nothing() {
    let mut e = estate(false);
    let crank = e.crank;
    let fund = Payload::Fund { origin: ROOT_UNIT, fee_bps: 250, cycle_slots: 8 * WINDOW, range_lo: 2000, range_hi: 8000, option_fee: SOL / 10, chair_quorum: 1 };
    let h = carry(&mut e, &fund, 1);
    e.w.run(execute(crank, h, &[ROOT_UNIT], &fund), &[ok()]);
    let at = fund_pda();
    // the fund of this layout, as it stands: not an older fund
    let payer = e.w.add_wallet(SOL);
    e.w.run(grow_fund(payer, None), &[refuse(PoleisError::NotAnOlderFund)]);
    let full = make_older(&mut e.w);
    let older = e.w.data(&at);
    let lamports = e.w.lamports(&at);
    let program_owned = |data: Vec<u8>| Account { lamports, data, owner: PROGRAM_ID, executable: false, rent_epoch: 0 };
    let edited = |o: usize, b: u8| {
        let mut d = older.clone();
        d[o] = b;
        d
    };
    let cases: Vec<(&str, Account, Check)> = vec![
        ("a byte short", program_owned(older[..Fund::OLD_SIZE - 1].to_vec()), refuse(PoleisError::NotAnOlderFund)),
        ("a byte long", program_owned([older.clone(), vec![0]].concat()), refuse(PoleisError::NotAnOlderFund)),
        ("already grown", program_owned(full.clone()), refuse(PoleisError::NotAnOlderFund)),
        ("owned by another program", Account { lamports, data: older.clone(), owner: Pubkey::default(), executable: false, rent_epoch: 0 }, refuse(PoleisError::InvalidAccount)),
        ("another discriminator", program_owned(edited(0, 18)), refuse(PoleisError::NotAnOlderFund)),
        ("another layout byte", program_owned(edited(1, 7)), refuse(PoleisError::NotAnOlderFund)),
        ("another bump", program_owned(edited(Fund::OFF_BUMP, 254)), refuse(PoleisError::InvalidPda)),
        ("a chair quorum of none", program_owned(edited(Fund::OFF_CHAIR_QUORUM, 0)), refuse(PoleisError::NotAnOlderFund)),
        ("a chair quorum past the chairs", program_owned(edited(Fund::OFF_CHAIR_QUORUM, 13)), refuse(PoleisError::NotAnOlderFund)),
        ("a share past the whole", program_owned(edited(Fund::OFF_FEE_BPS + 1, 0xff)), refuse(PoleisError::NotAnOlderFund)),
        ("a range upside down", program_owned(edited(Fund::OFF_RANGE_LO + 1, 0xff)), refuse(PoleisError::NotAnOlderFund)),
        ("a cycle of no slots", program_owned({ let mut d = older.clone(); d[Fund::OFF_CYCLE_SLOTS..Fund::OFF_CYCLE_SLOTS + 8].fill(0); d }), refuse(PoleisError::NotAnOlderFund)),
    ];
    for (what, account, check) in cases {
        let before = account.clone();
        lay(&mut e.w, at, account);
        e.w.run(grow_fund(payer, None), &[check]);
        let now = e.w.accounts.iter().find(|(k, _)| *k == at).unwrap().1.clone();
        assert_eq!((now.data, now.lamports, now.owner), (before.data, before.lamports, before.owner), "{what}: refused and unchanged");
        assert_eq!(e.w.lamports(&payer), SOL, "{what}: the payer paid nothing");
    }
    lay(&mut e.w, at, program_owned(older.clone()));
    // the older fund's bytes at any other address are not the fund
    let elsewhere = Pubkey::new_unique();
    lay(&mut e.w, elsewhere, program_owned(older.clone()));
    let mut there = grow_fund(payer, None);
    there.accounts[1].pubkey = elsewhere;
    e.w.run(there, &[refuse(PoleisError::InvalidPda)]);
    assert_eq!(e.w.data(&elsewhere), older);
    // a payer that does not sign, or words where there are none
    let mut unsigned = grow_fund(payer, None);
    unsigned.accounts[0].is_signer = false;
    e.w.run(unsigned, &[Check::err(ProgramError::MissingRequiredSignature)]);
    let mut worded = grow_fund(payer, None);
    worded.data.push(0);
    e.w.run(worded, &[Check::err(ProgramError::InvalidInstructionData)]);
    assert_eq!(e.w.data(&at), older, "nothing grew");
    // and the right one, once
    e.w.run(grow_fund(payer, None), &[ok()]);
    assert_eq!(e.w.data(&at), full);
}

/// Devnet's own older fund and its current option, as `getMultipleAccounts` read them at slot 525,947,015 (Nov 24 2026 ~23:0x UTC):
/// the fund of Sept 6 2026's build, 706 bytes, 5,210,955 lamports; its one option, 148 bytes.
const DEVNET_FUND_706: &str = "EQj/+gAIBwAAAAAAAOq9dB0AAAAAAQAAAAAAAABwmRQAAAAAANW7BQAAAAAAAwAAAAAAAAD4KgAAAAAAAAAAAAAAAAAAAwAAAAAAAADQB0AfQEIPAAAAAAABY3WS0kSLiY/92ER2rhhq41Mcq/xgLQCRewFjVFuZZ0DPunQdAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAP//////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
const DEVNET_FUND_LAMPORTS: u64 = 5_210_955;
const DEVNET_OPTION_148: &str = "Ewj+Y3WS0kSLiY/92ER2rhhq41Mcq/xgLQCRewFjVFuZZ0DtW5EOEkFnmDWj5tQoH3UGpggKqQJh3B8RTLTeO/SbJC+YZ/N3XGPm0Jwr/V+PT0B4C2vXpt1LQudFN0w+kBgGyZsnS4KL5SAsxWPuEcDirzP+P+ESophU/Gucn+zLUpABAwAAAAAAAADMuXQdAAAAAA==";
const DEVNET_OPTION_LAMPORTS: u64 = 1_402_080;

#[test]
fn devnets_own_older_fund_grows_every_old_value_in_place_and_its_grant_is_booked_to_its_current_option() {
    let b64 = |s: &str| base64::engine::general_purpose::STANDARD.decode(s).unwrap();
    let (old, opt) = (b64(DEVNET_FUND_706), b64(DEVNET_OPTION_148));
    assert_eq!((old.len(), opt.len()), (706, 148));
    let mut w = World::new();
    // devnet's rent, as getMinimumBalanceForRentExemption answered it that night: 4,236,720 for 706 bytes, 4,602,480 for 778
    w.mollusk.sysvars.rent.lamports_per_byte = 5_080;
    assert_eq!((w.rent(706), w.rent(778)), (4_236_720, 4_602_480));
    let at = fund_pda();
    assert_eq!(at.to_string(), "7n3J6fujkQdFQ4YQyoUG4E1XGto2DyWBzuvrJNrSqdcn");
    let current: [u8; 32] = old[Fund::OFF_CURRENT..Fund::OFF_CURRENT + 32].try_into().unwrap();
    let recipient = Pubkey::new_from_array(opt[35..67].try_into().unwrap());
    assert_eq!(option_pda(&current).to_string(), "9ixbFkaQCNKLWhmuRefSJpvu5YwBpdMroz2tKPnSb5G1", "the option devnet holds is the current one");
    lay(&mut w, at, Account { lamports: DEVNET_FUND_LAMPORTS, data: old.clone(), owner: PROGRAM_ID, executable: false, rent_epoch: 0 });
    lay(&mut w, option_pda(&current), Account { lamports: DEVNET_OPTION_LAMPORTS, data: opt, owner: PROGRAM_ID, executable: false, rent_epoch: 0 });
    w.warp(525_947_015);
    let payer = w.add_wallet(SOL);
    // before: the roll reads it as no fund of this layout (custom 2, the rehearsal's `cycle` refusal)
    w.run(settle_cycle(payer), &[refuse(PoleisError::InvalidAccount)]);

    w.run(grow_fund(payer, Some(current)), &[ok()]);
    let d = w.data(&at);
    assert_eq!(d.len(), 778);
    assert_eq!(&d[..706], &old[..], "every field of devnet's fund where it stood");
    assert_eq!(&d[Fund::OFF_CURRENT_RECIPIENT..Fund::OFF_OWED], recipient.as_ref(), "the current option's recipient");
    assert!(d[Fund::OFF_OWED..].iter().all(|b| *b == 0), "nothing owed, to nobody");
    assert_eq!(w.lamports(&at), DEVNET_FUND_LAMPORTS + 365_760, "the 72 bytes' rent, and nothing moved out");
    assert_eq!(w.lamports(&payer), SOL - 365_760);
    let f = FundView::parse(&d).unwrap();
    assert_eq!((f.fee_bps, f.cycle_slots, f.cycle_seq, f.total_in, f.total_out, f.staked_total, f.wp_sum, f.wp_weight), (250, 1_800, 1, 1_350_000, 375_765, 3, 11_000, 3));
    assert_eq!((f.range_lo, f.range_hi, f.option_fee, f.chair_quorum), (2_000, 8_000, 1_000_000, 1));
    assert!(f.chairs.iter().all(|c| c.root == NO_ROOT));

    // the cycle that ran out on Sept 6 rolls at the first touch: the grant is booked, from the same free balance, to that recipient
    let free = DEVNET_FUND_LAMPORTS - 4_236_720;
    w.run(settle_cycle(payer), &[ok()]);
    let f = FundView::parse(&w.data(&at)).unwrap();
    assert_eq!((f.cycle_seq, f.owed, f.owed_to), (2, free * 3_666 / 10_000, recipient), "11,000 / 3 = 3,666 bps of 974,235");
    w.run(claim_grant(payer, recipient), &[ok()]);
    assert_eq!(w.lamports(&recipient), free * 3_666 / 10_000);
    w.run(grow_fund(payer, Some(current)), &[refuse(PoleisError::NotAnOlderFund)]);

    // the same bytes laid on a chain of a dearer rent (a validator's default, 6,960 a byte: the rehearsal's copy), where devnet's balance is
    // under even the older size's minimum: a rent-paying account may not change its size, so the payer lifts it to the new minimum
    let mut w = World::new();
    assert!(w.rent(706) > DEVNET_FUND_LAMPORTS);
    lay(&mut w, at, Account { lamports: DEVNET_FUND_LAMPORTS, data: old.clone(), owner: PROGRAM_ID, executable: false, rent_epoch: 0 });
    let option_rent = w.rent(148);
    lay(&mut w, option_pda(&current), Account { lamports: option_rent, data: b64(DEVNET_OPTION_148), owner: PROGRAM_ID, executable: false, rent_epoch: 0 });
    let payer = w.add_wallet(SOL);
    w.run(grow_fund(payer, Some(current)), &[ok()]);
    assert_eq!(&w.data(&at)[..706], &old[..]);
    let floor = w.rent(778);
    assert_eq!((w.lamports(&at), w.lamports(&payer)), (floor, SOL - (floor - DEVNET_FUND_LAMPORTS)));
}

// ================= the door: a polis grows by admission =================

/// A root decision carried by seven founders and the floor's majority, sealed.
fn decide_root(w: &mut World, crank: &Pubkey, proposer: &Pubkey, votes: &[(&Voter, u8)], payload: &Payload, nonce: u64) -> [u8; 32] {
    let h = w.open(proposer, payload, &[ROOT_UNIT], nonce);
    w.cast(crank, &h, ROOT_UNIT, votes);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(*crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    h
}

#[test]
fn the_door_opens_by_codex_and_the_polis_admits_or_refuses() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let mut votes = pro(&founders, 7);
    votes.push((&alice, VOTE_PRO));
    votes.push((&bob, VOTE_PRO));

    // the root climbs to G(2,0) by the door: ten spots open at a twentieth of a SOL, and no sale
    let door = Payload::Door { origin: ROOT_UNIT, price: SOL / 20 };
    let h = decide_root(&mut w, &crank, &alice.key, &votes, &door, 1);
    w.run(execute(crank, h, &[ROOT_UNIT], &door), &[ok()]);
    let r = w.registry(ROOT_UNIT);
    assert_eq!((r.rung, r.total_seats, r.seat_reserve), (2, 6, SOL / 20));
    let p = PoolView::parse(&w.data(&pool_pda(ROOT_UNIT))).expect("the pool");
    assert_eq!((p.unit, p.open, p.filled, p.price), (ROOT_UNIT, 10, 0, SOL / 20));
    assert!(w.data(&sale_pda(ROOT_UNIT, 1)).is_empty(), "no sale: the spots are at the door");
    assert!(w.receipts().iter().any(|x| x.kind == RCPT_DOOR && x.amount == 10), "the door leaves its receipt");

    // dave knocks: the price and one row's rent go into escrow under his key; twice is refused; too soon to withdraw
    let dave = w.voter(SOL);
    let row = w.rent(57) - w.rent(0);
    let d0 = w.lamports(&dave.key);
    w.run(knock(dave.key, ROOT_UNIT), &[ok()]);
    let k = KnockView::parse(&w.data(&knock_pda(ROOT_UNIT, &dave.key))).expect("the knock");
    assert_eq!((k.unit, k.key, k.deposit), (ROOT_UNIT, dave.key, SOL / 20));
    assert_eq!(d0 - w.lamports(&dave.key), SOL / 20 + row + w.rent(59), "the price, a row's rent and the knock's own rent left dave");
    w.run(knock(dave.key, ROOT_UNIT), &[refuse(PoleisError::KnockOpen)]);
    w.run(withdraw_knock(dave.key, ROOT_UNIT), &[refuse(PoleisError::NotYet)]);

    // admitting a key that is not knocking is refused when the seal executes
    let eve = w.voter(SOL);
    let admit_eve = Payload::Admit { origin: ROOT_UNIT, holder: key(&eve.key), root: ROOT_UNIT };
    let he = decide_root(&mut w, &crank, &alice.key, &votes, &admit_eve, 2);
    w.run(execute(crank, he, &[ROOT_UNIT], &admit_eve), &[refuse(PoleisError::NotKnocking)]);

    // the polis admits dave by its double seal: a seventh seat, the price to the treasury (no fund: no fee), the rent home
    let t0 = w.lamports(&treasury_pda(ROOT_UNIT));
    let admit = Payload::Admit { origin: ROOT_UNIT, holder: key(&dave.key), root: ROOT_UNIT };
    let ha = decide_root(&mut w, &crank, &alice.key, &votes, &admit, 3);
    w.run(execute(crank, ha, &[ROOT_UNIT], &admit), &[ok()]);
    let r = w.registry(ROOT_UNIT);
    assert_eq!((r.total_seats, r.seats_of(HOLDER_KEY, &key(&dave.key))), (7, 1));
    assert_eq!(w.lamports(&treasury_pda(ROOT_UNIT)) - t0, SOL / 20, "the price lands in the treasury");
    assert!(w.data(&knock_pda(ROOT_UNIT, &dave.key)).is_empty(), "the knock closed");
    assert_eq!(d0 - w.lamports(&dave.key), SOL / 20 + row, "dave paid the price and his row; the knock's rent came home");
    let p = PoolView::parse(&w.data(&pool_pda(ROOT_UNIT))).unwrap();
    assert_eq!((p.open, p.filled), (10, 1));
    assert!(w.receipts().iter().any(|x| x.kind == RCPT_ADMIT && x.amount == SOL / 20 && x.outcome == 1), "the admission's receipt");
    w.run(execute(crank, ha, &[ROOT_UNIT], &admit), &[refuse(PoleisError::AlreadyExecuted)]);

    // eve knocks and is refused: the deposit goes home whole, with both rents, and she holds nothing
    w.run(knock(eve.key, ROOT_UNIT), &[ok()]);
    let e0 = w.lamports(&eve.key);
    let refusal = Payload::Refuse { origin: ROOT_UNIT, holder: key(&eve.key) };
    let hr = decide_root(&mut w, &crank, &alice.key, &votes, &refusal, 4);
    w.run(execute(crank, hr, &[ROOT_UNIT], &refusal), &[ok()]);
    assert_eq!(w.lamports(&eve.key) - e0, SOL / 20 + row + w.rent(59));
    assert!(w.data(&knock_pda(ROOT_UNIT, &eve.key)).is_empty());
    assert_eq!(w.registry(ROOT_UNIT).seats_of(HOLDER_KEY, &key(&eve.key)), 0);

    // frank knocks and, kept waiting a CODEX window, takes it back himself; a closed knock is nobody's
    let frank = w.voter(SOL);
    w.run(knock(frank.key, ROOT_UNIT), &[ok()]);
    let f0 = w.lamports(&frank.key);
    w.run(withdraw_knock(frank.key, ROOT_UNIT), &[refuse(PoleisError::NotYet)]);
    w.warp(w.slot() + w.registry(ROOT_UNIT).codex_window + 1);
    w.run(withdraw_knock(frank.key, ROOT_UNIT), &[ok()]);
    assert_eq!(w.lamports(&frank.key) - f0, SOL / 20 + row + w.rent(59));
    w.run(withdraw_knock(frank.key, ROOT_UNIT), &[refuse(PoleisError::NotKnocking)]);
}

// ================= twelve is a budget: the chairs' majority is of the lit chairs =================

/// A council of three founders, nine chairs unlit, and a floor of one seat (alice), ratified.
fn three_lit_chairs() -> (World, Pubkey, Vec<Voter>, Voter) {
    let mut w = World::new();
    let payer = w.add_wallet(100 * SOL);
    let crank = w.add_wallet(100 * SOL);
    let usdc = w.mint();
    w.run(init_config(payer, usdc), &[ok()]);
    let founders: Vec<Voter> = (0..3).map(|_| w.voter(SOL)).collect();
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
        name: b"THREE LIT".to_vec(),
        desc: b"three chairs of the twelve".to_vec(),
    };
    w.run(open_genesis(payer, &g), &[ok()]);
    let alice = w.voter(SOL);
    let salt = [9u8; 32];
    w.run(commit_seat_bid(alice.key, ROOT_UNIT, 0, 0, seat_commitment(1_000_000, 0, &salt, &alice.key)), &[ok()]);
    w.warp(100);
    w.run(reveal_seat_bid(alice.key, ROOT_UNIT, 0, 0, 1_000_000, salt), &[ok()]);
    w.warp(200);
    w.run(settle_seats(crank, ROOT_UNIT, 0), &[ok()]);
    w.run(claim_seat(alice.key, ROOT_UNIT, 0, 0, ROOT_UNIT), &[ok()]);
    w.run(ratify_genesis(crank), &[ok()]);
    (w, crank, founders, alice)
}

#[test]
fn twelve_is_a_budget_three_lit_chairs_decide_by_two() {
    let (mut w, crank, founders, alice) = three_lit_chairs();
    assert_eq!(w.registry(ROOT_UNIT).filled_chairs(), 3);
    assert_eq!((chair_quorum(3), chair_quorum(11), chair_quorum(12), PENTAGON_QUORUM), (2, 6, 7, 7), "seven when all twelve are lit");
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 6_000 };
    // two of the three lit chairs and the floor: the chairs' majority, PASSED
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    w.cast(&crank, &h, ROOT_UNIT, &[(&founders[0], VOTE_PRO), (&founders[1], VOTE_PRO), (&alice, VOTE_PRO)]);
    w.warp(w.slot() + CODEX_WINDOW_FACTOR * WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED, "two of three lit chairs are the chairs' majority");
    // one of the three and the floor: the chairs fail alone, DEADLOCK
    let h2 = w.open(&alice.key, &payload, &[ROOT_UNIT], 2);
    w.cast(&crank, &h2, ROOT_UNIT, &[(&founders[0], VOTE_PRO), (&founders[1], VOTE_CONTRA), (&alice, VOTE_PRO)]);
    w.warp(w.slot() + CODEX_WINDOW_FACTOR * WINDOW + 1);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h2, ROOT_UNIT).state, STATE_DEADLOCK, "one of three lit chairs is not a majority");
    // two of three chairs against the floor: the floor still holds, DEADLOCK
    let h3 = w.open(&alice.key, &payload, &[ROOT_UNIT], 3);
    w.cast(&crank, &h3, ROOT_UNIT, &[(&founders[0], VOTE_PRO), (&founders[2], VOTE_PRO), (&alice, VOTE_CONTRA)]);
    w.warp(w.slot() + CODEX_WINDOW_FACTOR * WINDOW + 1);
    w.run(seal(crank, h3, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h3, ROOT_UNIT).state, STATE_DEADLOCK, "the chairs never move alone");
}

/// A root of six founders and six crews in its chairs 6 to 11, each crew of
/// two founders with a crew of its own, of one founder, in its chair 11:
/// thirteen poleis, one more than a proposal opens with. The root's floor is
/// one seat, alice's.
fn thirteen_poleis() -> (World, Pubkey, Vec<Voter>, Vec<Vec<Voter>>, Vec<Vec<Voter>>, Voter) {
    let mut w = World::new();
    let payer = w.add_wallet(100 * SOL);
    let crank = w.add_wallet(100 * SOL);
    let usdc = w.mint();
    w.run(init_config(payer, usdc), &[ok()]);
    let founders: Vec<Voter> = (0..6).map(|_| w.voter(SOL)).collect();
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
        name: b"THIRTEEN".to_vec(),
        desc: b"a root, six crews, six crews of crews".to_vec(),
    };
    w.run(open_genesis(payer, &g), &[ok()]);
    let mut crews = Vec::new();
    let mut leaves = Vec::new();
    for i in 0..6u64 {
        crews.push(found(&mut w, payer, ROOT_UNIT, 1 + i, 6 + i as u8, 2));
        leaves.push(found(&mut w, payer, 1 + i, 7 + i, 11, 1));
    }
    let alice = w.voter(SOL);
    let salt = [9u8; 32];
    w.run(commit_seat_bid(alice.key, ROOT_UNIT, 0, 0, seat_commitment(1_000_000, 0, &salt, &alice.key)), &[ok()]);
    w.warp(100);
    w.run(reveal_seat_bid(alice.key, ROOT_UNIT, 0, 0, 1_000_000, salt), &[ok()]);
    w.warp(200);
    w.run(settle_seats(crank, ROOT_UNIT, 0), &[ok()]);
    w.run(claim_seat(alice.key, ROOT_UNIT, 0, 0, ROOT_UNIT), &[ok()]);
    w.run(ratify_genesis(crank), &[ok()]);
    (w, crank, founders, crews, leaves, alice)
}

/// Open the root's vote alone and let every crew open its own: the six
/// crews under the root, then each crew's own crew under it.
fn convene_tree(w: &mut World, crank: Pubkey, h: [u8; 32]) {
    for i in 0..6u64 {
        w.run(convene(crank, h, ROOT_UNIT, 1 + i), &[ok()]);
        w.run(convene(crank, h, 1 + i, 7 + i), &[ok()]);
    }
}

#[test]
fn each_polis_opens_its_own_vote_and_a_root_over_thirteen_decides() {
    // the hypertrace's probe (Nov 2 2026), made the gate: a root with six
    // crews, each with a crew of its own, decided nothing before this law
    let (mut w, crank, founders, crews, leaves, alice) = thirteen_poleis();
    let payload = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 7, deposit: 9 };
    let every: Vec<u64> = (0..13).collect();
    let hash = proposal_hash(CLASS_SIGNAL, ROOT_UNIT, &MANIFEST, &payload.hash(), 1);
    // one transaction still opens at most twelve sessions: the old ceiling
    w.run(propose(alice.key, hash, MANIFEST, &payload, &every), &[Check::err(ProgramError::InvalidInstructionData)]);

    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    // a crew opens its vote only from a chair of a polis whose vote is open
    w.run(convene(crank, h, 1, 7), &[refuse(PoleisError::InvalidAccount)]);
    convene_tree(&mut w, crank, h);
    w.run(convene(crank, h, ROOT_UNIT, 7), &[refuse(PoleisError::InvalidHolder)]);
    let leaf = w.session(&h, 7);
    assert_eq!(leaf.convener, Some(crank));
    assert_eq!(leaf.deadline, w.session(&h, ROOT_UNIT).deadline, "a convened window runs from the proposal's opening");

    // the leaves speak first: no seal over the head of a crew, at any level
    let mut v: Vec<(&Voter, u8)> = founders.iter().take(4).map(|f| (f, VOTE_PRO)).collect();
    v.push((&alice, VOTE_PRO));
    w.cast(&crank, &h, ROOT_UNIT, &v);
    w.run(seal(crank, h, ROOT_UNIT), &[refuse(PoleisError::ChildMustSpeak)]);
    for i in 0..6usize {
        let (crew, leaf) = (1 + i as u64, 7 + i as u64);
        w.cast(&crank, &h, crew, &pro(&crews[i], 2));
        w.run(seal(crank, h, crew), &[refuse(PoleisError::ChildMustSpeak)]);
        w.cast(&crank, &h, leaf, &pro(&leaves[i], 1));
        w.run(seal(crank, h, leaf), &[ok()]);
        w.run(mandate(crank, h, crew, leaf), &[ok()]);
        w.run(seal(crank, h, crew), &[ok()]);
        assert_eq!(w.session(&h, crew).state, STATE_PASSED);
        w.run(mandate(crank, h, ROOT_UNIT, crew), &[ok()]);
    }
    w.run(adjourn(h, 7, crank), &[refuse(PoleisError::NotClosable)]);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.state, s.pent_pro), (STATE_PASSED, 10), "four founders and six crews, of twelve chairs");
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).seat_reserve, 7, "thirteen poleis decided");

    // the rent of every vote a polis opened goes home to whoever opened it;
    // the root's goes home with its proposal
    let rent = w.lamports(&session_pda(&h, 7));
    assert_eq!(rent, w.rent(w.session_size(7) + Session::CONVENER_TAIL), "a leaf with no seat: a roster of none and the convener's tail");
    w.run(adjourn(h, 7, alice.key), &[refuse(PoleisError::InvalidAccount)]);
    w.run(adjourn(h, ROOT_UNIT, crank), &[refuse(PoleisError::NotClosable)]);
    let before = w.lamports(&crank);
    for u in 1..13u64 {
        w.run(adjourn(h, u, crank), &[ok()]);
    }
    assert_eq!(w.lamports(&crank) - before, 12 * rent);
    w.run(close_proposal(h, alice.key, &[ROOT_UNIT]), &[ok()]);

    // under consent the AND folds up the tree: one leaf's no splits its crew,
    // the crew's split splits the root, and the root's seal alone says so
    let payload = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 8, deposit: 9 };
    let h = proposal_hash(CLASS_SIGNAL | CLASS_CONSENT, ROOT_UNIT, &MANIFEST, &payload.hash(), 2);
    w.run(propose_consent(alice.key, h, MANIFEST, &payload, &[ROOT_UNIT]), &[ok()]);
    convene_tree(&mut w, crank, h);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    for i in 0..6usize {
        w.cast(&crank, &h, 7 + i as u64, &[(&leaves[i][0], if i == 0 { VOTE_CONTRA } else { VOTE_PRO })]);
        w.cast(&crank, &h, 1 + i as u64, &pro(&crews[i], 2));
    }
    // a no has no fast path: the seals wait out the window
    w.warp(w.slot() + WINDOW + 1);
    for i in 0..6u64 {
        w.run(seal(crank, h, 7 + i), &[ok()]);
        w.run(mandate(crank, h, 1 + i, 7 + i), &[ok()]);
        w.run(seal(crank, h, 1 + i), &[ok()]);
        w.run(mandate(crank, h, ROOT_UNIT, 1 + i), &[ok()]);
    }
    assert_eq!(w.session(&h, 7).state, STATE_REJECTED);
    assert_eq!(w.session(&h, 1).state, STATE_DEADLOCK, "its chambers said yes over its crew's no");
    assert_eq!(w.session(&h, 2).state, STATE_PASSED);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.state, s.pent_pro, s.chair_voice(6)), (STATE_DEADLOCK, 9, Some(VOTE_ABSTENTIO)), "the root carries the split up");
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[refuse(PoleisError::NotPassed)]);
    assert_eq!(w.registry(ROOT_UNIT).seat_reserve, 7);
}

#[test]
fn a_crew_that_never_opens_its_vote_is_heard_absent() {
    let Estate { mut w, crank, founders, alice, carol, .. } = estate(true);
    let payload = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 2, deposit: 2 };
    let h = w.open(&alice.key, &payload, &[ROOT_UNIT], 1);
    let mut v = pro(&founders, 9);
    v.extend([(&alice, VOTE_PRO), (&carol, VOTE_PRO)]);
    w.cast(&crank, &h, ROOT_UNIT, &v);
    // decided, but never over its crew's head while the crew may still open its vote
    w.run(seal(crank, h, ROOT_UNIT), &[refuse(PoleisError::ChildMustSpeak)]);
    w.run(mandate(crank, h, ROOT_UNIT, 1), &[refuse(PoleisError::ChildMustSpeak)]);
    // the window closes with the crew silent: too late to open, heard as absent
    w.warp(w.slot() + WINDOW + 1);
    w.run(convene(crank, h, ROOT_UNIT, 1), &[refuse(PoleisError::SessionClosed)]);
    w.run(mandate(crank, h, ROOT_UNIT, 1), &[ok()]);
    w.run(mandate(crank, h, ROOT_UNIT, 1), &[refuse(PoleisError::AlreadyMandated)]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.pent_abst, s.chair_voice(11)), (9, 0, Some(VOTE_ABSTENTIO)), "heard from, absent");
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    // under a sealed vote no crew opens its own, and under a spent one none at all
    w.run(convene(crank, h, ROOT_UNIT, 1), &[refuse(PoleisError::SessionClosed)]);
    w.run(execute(crank, h, &[ROOT_UNIT], &payload), &[ok()]);
    w.run(convene(crank, h, ROOT_UNIT, 1), &[refuse(PoleisError::AlreadyExecuted)]);
}

#[test]
fn the_code_never_moves_under_a_crew_seated_since_the_question() {
    let mut e = estate(true);
    let crank = e.crank;
    // two CODEX decisions of the root alone; its crew opens its own vote in each
    let first = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 6_000 };
    let second = Payload::Window { origin: ROOT_UNIT, window: 1_500, codex_window: 4_500 };
    let a = e.w.open(&e.alice.key, &first, &[ROOT_UNIT], 1);
    let b = e.w.open(&e.alice.key, &second, &[ROOT_UNIT], 2);
    for h in [a, b] {
        e.w.run(convene(crank, h, ROOT_UNIT, 1), &[ok()]);
        crew_speaks(&mut e.w, crank, h, &e.child_founders);
        let mut v = pro(&e.founders, 9);
        v.extend([(&e.alice, VOTE_PRO), (&e.carol, VOTE_PRO)]);
        e.w.cast(&crank, &h, ROOT_UNIT, &v);
        e.w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
        assert_eq!(e.w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    }
    // the crew held its chair through the question and spoke there: the code moves
    e.w.run(execute(crank, b, &[ROOT_UNIT], &second), &[ok()]);
    assert_eq!(e.w.registry(ROOT_UNIT).window, 1_500);

    // then the root seats its crew in a second chair, by two seat acts of its own
    let recall = Payload::Recall { origin: ROOT_UNIT, chair: 0 };
    let r = carry(&mut e, &recall, 3);
    e.w.run(execute(crank, r, &[ROOT_UNIT, 1], &recall), &[ok()]);
    let attach = Payload::Attach { origin: ROOT_UNIT, chair: 0, holder: unit_holder(1), kind: HOLDER_UNIT, term: 0, retainer: 0 };
    let t = carry(&mut e, &attach, 4);
    e.w.run(execute(crank, t, &[ROOT_UNIT, 1], &attach), &[ok()]);
    let signal = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 4, deposit: 4 };
    let h = carry(&mut e, &signal, 5);
    let s = e.w.session(&h, ROOT_UNIT);
    assert_eq!((s.chair_voice(0), s.chair_voice(11), s.pent_pro), (Some(VOTE_PRO), Some(VOTE_PRO), 10), "a crew in two chairs speaks in both");

    // the first decision would move the code under a crew seated since it was
    // asked: the voice chair 0 carries is its old founder's, not the crew's
    e.w.run(execute(crank, a, &[ROOT_UNIT], &first), &[refuse(PoleisError::ClosureBroken)]);
}

#[test]
fn a_crew_that_can_never_speak_never_freezes_its_parent() {
    // a chair seated with a unit that has no registry: before the each-polis
    // law every decision of the polis had to list that unit and none could,
    // so the polis could never decide again, not even to recall the chair.
    // The seat act refuses that unit now (a_seat_names_a_unit_of_its_own_tree);
    // a chair an older program seated holds it still, laid here as it left it
    let (mut w, crank, founders, alice) = three_lit_chairs();
    let all: Vec<(&Voter, u8)> = founders.iter().map(|f| (f, VOTE_PRO)).chain([(&alice, VOTE_PRO)]).collect();
    w.lay_chair(ROOT_UNIT, 3, unit_holder(99), HOLDER_UNIT);

    // the next decision: the ghost cannot open its vote, the root waits out
    // the window, the ghost is heard absent, and the root recalls its chair
    let recall = Payload::Recall { origin: ROOT_UNIT, chair: 3 };
    let r = w.open(&alice.key, &recall, &[ROOT_UNIT], 2);
    w.run(convene(crank, r, ROOT_UNIT, 99), &[refuse(PoleisError::InvalidAccount)]);
    w.cast(&crank, &r, ROOT_UNIT, &all);
    w.run(seal(crank, r, ROOT_UNIT), &[refuse(PoleisError::ChildMustSpeak)]);
    w.warp(w.slot() + WINDOW + 1);
    w.run(mandate(crank, r, ROOT_UNIT, 99), &[ok()]);
    w.run(seal(crank, r, ROOT_UNIT), &[ok()]);
    let s = w.session(&r, ROOT_UNIT);
    assert_eq!((s.state, s.pent_pro, s.chair_voice(3)), (STATE_PASSED, 3, Some(VOTE_ABSTENTIO)), "three of four lit chairs, the ghost absent");
    w.run(execute(crank, r, &[ROOT_UNIT], &recall), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).pentagons[3].kind, HOLDER_EMPTY);
}

// ================= a unit sits only below the polis that seats it =================

#[test]
fn a_polis_never_seats_its_ancestor() {
    // the cycle of the Nov 3 probe: on main, leaf 7 seated its own parent,
    // crew 1, and the root's next decision froze for good, crew 1 and leaf 7
    // each waiting for the other's seal, the rent of thirteen ballot boxes
    // and the proposal held forever
    let (mut w, crank, founders, crews, leaves, alice) = thirteen_poleis();
    let up = [(1, PoleisError::NotDescendant), (7, PoleisError::NotDescendant), (ROOT_UNIT, PoleisError::InvalidHolder)];
    for (nonce, (holder, refusal)) in up.into_iter().enumerate() {
        let act = Payload::Attach { origin: 7, chair: 5, holder: unit_holder(holder), kind: HOLDER_UNIT, term: 0, retainer: 0 };
        let h = w.open(&leaves[0][0].key, &act, &[7], nonce as u64 + 1);
        w.cast(&crank, &h, 7, &pro(&leaves[0], 1));
        w.run(seal(crank, h, 7), &[ok()]);
        w.run(execute(crank, h, &[7], &act), &[refuse(refusal)]);
    }
    assert_eq!(w.registry(7).pentagons[5].kind, HOLDER_EMPTY, "its parent, itself, the root: no ancestor sits in a leaf's chair");

    // so the probe's decision of the root folds up the whole tree
    let payload = Payload::SaleDefaults { origin: ROOT_UNIT, reserve: 7, deposit: 9 };
    let d = w.open(&alice.key, &payload, &[ROOT_UNIT], 9);
    convene_tree(&mut w, crank, d);
    let mut v: Vec<(&Voter, u8)> = founders.iter().map(|f| (f, VOTE_PRO)).collect();
    v.push((&alice, VOTE_PRO));
    w.cast(&crank, &d, ROOT_UNIT, &v);
    for i in 0..6usize {
        let (crew, leaf) = (1 + i as u64, 7 + i as u64);
        w.cast(&crank, &d, leaf, &pro(&leaves[i], 1));
        w.run(seal(crank, d, leaf), &[ok()]);
        w.run(mandate(crank, d, crew, leaf), &[ok()]);
        w.cast(&crank, &d, crew, &pro(&crews[i], 2));
        w.run(seal(crank, d, crew), &[ok()]);
        w.run(mandate(crank, d, ROOT_UNIT, crew), &[ok()]);
    }
    w.run(seal(crank, d, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&d, ROOT_UNIT).state, STATE_PASSED);
    w.run(execute(crank, d, &[ROOT_UNIT], &payload), &[ok()]);
    assert_eq!(w.registry(ROOT_UNIT).seat_reserve, 7, "the tree decided");
}

#[test]
fn a_pair_never_seats_each_other() {
    // crews 1 and 2 stand at one level, so neither seats the other. Leaf 8,
    // crew 2's own, stands below crew 1 and may sit in its chair as well;
    // then it never seats crew 1 back. Levels rise along every seat: no pair
    // closes a cycle, whoever founded whom
    let (mut w, crank, _, crews, leaves, _) = thirteen_poleis();
    // a decision of crew 1 over itself and its leaf, the leaf speaking first
    let crew_decides = |w: &mut World, act: &Payload, nonce: u64| -> [u8; 32] {
        let h = w.open(&crews[0][0].key, act, &[1, 7], nonce);
        w.cast(&crank, &h, 7, &pro(&leaves[0], 1));
        w.run(seal(crank, h, 7), &[ok()]);
        w.run(mandate(crank, h, 1, 7), &[ok()]);
        w.cast(&crank, &h, 1, &pro(&crews[0], 2));
        w.run(seal(crank, h, 1), &[ok()]);
        assert_eq!(w.session(&h, 1).state, STATE_PASSED);
        h
    };
    let sibling = Payload::Attach { origin: 1, chair: 5, holder: unit_holder(2), kind: HOLDER_UNIT, term: 0, retainer: 0 };
    let h = crew_decides(&mut w, &sibling, 1);
    w.run(execute(crank, h, &[1, 7], &sibling), &[refuse(PoleisError::NotDescendant)]);
    let cousin = Payload::Attach { origin: 1, chair: 5, holder: unit_holder(8), kind: HOLDER_UNIT, term: 0, retainer: 0 };
    let h = crew_decides(&mut w, &cousin, 2);
    w.run(execute(crank, h, &[1, 7], &cousin), &[ok()]);
    assert_eq!((w.registry(1).chair_of_unit(8), w.registry(2).chair_of_unit(8)), (Some(5), Some(11)), "leaf 8 sits in its own crew's chair and in crew 1's");
    let back = Payload::Attach { origin: 8, chair: 5, holder: unit_holder(1), kind: HOLDER_UNIT, term: 0, retainer: 0 };
    let h = w.open(&leaves[1][0].key, &back, &[8], 3);
    w.cast(&crank, &h, 8, &pro(&leaves[1], 1));
    w.run(seal(crank, h, 8), &[ok()]);
    w.run(execute(crank, h, &[8], &back), &[refuse(PoleisError::NotDescendant)]);
    assert_eq!(w.registry(8).pentagons[5].kind, HOLDER_EMPTY, "the pair never closes");

    // the cousin speaks in crew 1's chair through its own seal, and crew 1 waits for it
    let signal = Payload::SaleDefaults { origin: 1, reserve: 5, deposit: 5 };
    let h = w.open(&crews[0][0].key, &signal, &[1, 7], 4);
    w.cast(&crank, &h, 7, &pro(&leaves[0], 1));
    w.run(seal(crank, h, 7), &[ok()]);
    w.run(mandate(crank, h, 1, 7), &[ok()]);
    w.cast(&crank, &h, 1, &pro(&crews[0], 2));
    w.run(seal(crank, h, 1), &[refuse(PoleisError::ChildMustSpeak)]);
    w.run(convene(crank, h, 1, 8), &[ok()]);
    w.cast(&crank, &h, 8, &pro(&leaves[1], 1));
    w.run(seal(crank, h, 8), &[ok()]);
    w.run(mandate(crank, h, 1, 8), &[ok()]);
    w.run(seal(crank, h, 1), &[ok()]);
    let s = w.session(&h, 1);
    assert_eq!((s.state, s.pent_pro, s.chair_voice(5)), (STATE_PASSED, 4, Some(VOTE_PRO)), "two founders and two leaves, one of them its cousin");
}

#[test]
fn a_seat_names_a_unit_of_its_own_tree() {
    // a unit with no registry, a holder no unit answers to, a unit of another
    // tree: main seated each. The ghost and the stranger were heard absent at
    // every decision after, and the padded holder was never heard at all:
    // its polis could never seal again, not even to recall the chair (the
    // Nov 3 probe)
    let (mut w, crank, founders, alice) = three_lit_chairs();
    let all: Vec<(&Voter, u8)> = founders.iter().map(|f| (f, VOTE_PRO)).chain([(&alice, VOTE_PRO)]).collect();
    // tree 50, a stranger's, and its crew 51 one level below its root
    let stranger = w.add_wallet(100 * SOL);
    let now = w.slot();
    let g = GenesisSpec {
        charter: [5u8; 32],
        plan_cap: SOL,
        plan_period: 1_000_000,
        window: WINDOW,
        rung: 1,
        reserve: 1_000_000,
        deposit: 5_000_000,
        commit_end: now + 100,
        reveal_end: now + 200,
        founders: vec![stranger],
        name: b"ANOTHER TREE".to_vec(),
        desc: Vec::new(),
    };
    w.run_tx(&[init_config_of(50, stranger, Pubkey::default()), open_genesis_of(50, stranger, &g)], &stranger, Expect::Ok);
    let crew = FoundSpec {
        child: 51,
        chair: 1,
        charter: [51u8; 32],
        window: WINDOW,
        plan_cap: SOL,
        plan_period: 1_000_000,
        founders: vec![stranger],
        name: b"CREW 51".to_vec(),
        desc: Vec::new(),
    };
    w.run(found_child_of(50, stranger, 50, &crew), &[ok()]);
    assert_eq!((w.registry(51).root, w.registry(51).level), (50, 1), "deeper than root 0, in another tree");
    let mut padded = unit_holder(99);
    padded[31] = 1;
    let seats = [(unit_holder(99), PoleisError::InvalidAccount), (padded, PoleisError::InvalidHolder), (unit_holder(51), PoleisError::NotDescendant)];
    for (nonce, (holder, refusal)) in seats.into_iter().enumerate() {
        let act = Payload::Attach { origin: ROOT_UNIT, chair: 3, holder, kind: HOLDER_UNIT, term: 0, retainer: 0 };
        let h = w.open(&alice.key, &act, &[ROOT_UNIT], nonce as u64 + 1);
        w.cast(&crank, &h, ROOT_UNIT, &all);
        w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
        w.run(execute(crank, h, &[ROOT_UNIT], &act), &[refuse(refusal)]);
    }
    assert_eq!(w.registry(ROOT_UNIT).filled_chairs(), 3, "no ghost, no padding, no stranger: three lit chairs, the founders'");
}

// ================= a split is settled by kind: the SUM, for small things =================

/// The root's own law on the SUM, set by a CODEX act decided under the AND.
fn resolve_root(w: &mut World, crank: &Pubkey, proposer: &Pubkey, votes: &[(&Voter, u8)], sum_kinds: u8, small_cap: u64, nonce: u64) {
    let law = Payload::Resolution { origin: ROOT_UNIT, sum_kinds, small_cap };
    let h = decide_root(w, crank, proposer, votes, &law, nonce);
    w.run(execute(*crank, h, &[ROOT_UNIT], &law), &[ok()]);
}

/// The hash a proposal of `payload` takes under `nonce`.
fn hash_of(payload: &Payload, nonce: u64) -> [u8; 32] {
    proposal_hash(payload.class(), payload.origin(), &MANIFEST, &payload.hash(), nonce)
}

/// Open a proposal that asks for the SUM; returns its hash.
fn open_sum(w: &mut World, proposer: &Pubkey, payload: &Payload, impacted: &[u64], nonce: u64) -> [u8; 32] {
    let hash = hash_of(payload, nonce);
    w.run(propose_sum(*proposer, hash, MANIFEST, payload, impacted), &[ok()]);
    hash
}

/// Seven chairs and five of the floor's six seats: the AND, for the law itself.
fn and_votes<'a>(founders: &'a [Voter], alice: &'a Voter, bob: &'a Voter) -> Vec<(&'a Voter, u8)> {
    let mut v = pro(founders, 7);
    v.extend([(alice, VOTE_PRO), (bob, VOTE_PRO)]);
    v
}

#[test]
fn a_units_own_law_turns_the_sum_on_by_codex() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    assert!(RuleView::parse(&w.data(&rule_pda(ROOT_UNIT))).is_none(), "no unit settles by the SUM until its own law says so");
    let and = and_votes(&founders, &alice, &bob);
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_BUDGET | SUM_RULE, SOL, 1);
    let r = RuleView::parse(&w.data(&rule_pda(ROOT_UNIT))).expect("the rule");
    assert_eq!((r.unit, r.sum_kinds, r.small_cap), (ROOT_UNIT, SUM_BUDGET | SUM_RULE, SOL));
    assert_eq!(w.lamports(&rule_pda(ROOT_UNIT)), w.rent(Rule::SIZE), "the crank paid its rent");
    // a later act replaces it
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_TRANSFER, 2 * SOL, 2);
    let r = RuleView::parse(&w.data(&rule_pda(ROOT_UNIT))).expect("the rule");
    assert_eq!((r.unit, r.sum_kinds, r.small_cap), (ROOT_UNIT, SUM_TRANSFER, 2 * SOL));
}

#[test]
fn five_chairs_and_most_of_the_floor_carry_a_small_budget_by_the_sum() {
    // his "if 5/12 pentagons vote yes but 80% hexagons voted yes, it passes"
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let and = and_votes(&founders, &alice, &bob);
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_BUDGET, SOL, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let mut split = pro(&founders, 5);
    split.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);

    let h = open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT], 2);
    let p = ProposalView::parse(&w.data(&proposal_pda(&h))).expect("proposal parses");
    assert_eq!((p.class, p.sum, p.consent), (CLASS_TREASURY, true, false));
    w.cast(&crank, &h, ROOT_UNIT, &split);
    w.run(seal(crank, h, ROOT_UNIT), &[refuse(PoleisError::SessionOpen)]);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.active_weight), (5, 5, 6));
    assert_eq!(s.state, STATE_PASSED, "five of twelve chairs and five of six seats: the sum of the halves passes");
    let (l0, t0, spent0) = (w.lamports(&lab), w.lamports(&treasury_pda(ROOT_UNIT)), w.registry(ROOT_UNIT).plan_spent);
    w.run(execute(crank, h, &[ROOT_UNIT], &budget), &[ok()]);
    assert_eq!(w.lamports(&lab) - l0, SOL / 2);
    assert_eq!(t0 - w.lamports(&treasury_pda(ROOT_UNIT)), SOL / 2);
    assert_eq!(w.registry(ROOT_UNIT).plan_spent - spent0, SOL / 2, "charged to the plan");

    // the same votes on the same payload, the bit not asked: the AND, DEADLOCK
    let h2 = w.open(&alice.key, &budget, &[ROOT_UNIT], 3);
    assert!(!ProposalView::parse(&w.data(&proposal_pda(&h2))).unwrap().sum);
    w.cast(&crank, &h2, ROOT_UNIT, &split);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h2, ROOT_UNIT).state, STATE_DEADLOCK);
    let l1 = w.lamports(&lab);
    w.run(execute(crank, h2, &[ROOT_UNIT], &budget), &[refuse(PoleisError::NotPassed)]);
    assert_eq!(w.lamports(&lab), l1, "nothing moved");
}

#[test]
fn the_chairs_majority_and_half_the_floor_pass_by_the_sum() {
    // his first example's shape: the chairs over half, the floor short of it
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let and = and_votes(&founders, &alice, &bob);
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_BUDGET, SOL, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL };
    let mut split = pro(&founders, 7);
    split.push((&alice, VOTE_PRO));

    let h = open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT], 2);
    w.cast(&crank, &h, ROOT_UNIT, &split);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.active_weight, s.state), (7, 3, 6, STATE_PASSED), "seven of twelve and three of six: 58 % and 50 %");

    let h2 = w.open(&alice.key, &budget, &[ROOT_UNIT], 3);
    w.cast(&crank, &h2, ROOT_UNIT, &split);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h2, ROOT_UNIT).state, STATE_DEADLOCK, "without the bit the floor's half is no majority");
}

#[test]
fn a_unanimous_chamber_cannot_carry_the_other_under_a_third() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let and = and_votes(&founders, &alice, &bob);
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_BUDGET, SOL, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 4 };

    // every lit chair, and one seat of six
    let mut chairs = pro(&founders, 12);
    chairs.push((&carol, VOTE_PRO));
    let h = open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT], 2);
    w.cast(&crank, &h, ROOT_UNIT, &chairs);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.state), (12, 1, STATE_DEADLOCK), "the floor under a third still blocks");
    w.run(execute(crank, h, &[ROOT_UNIT], &budget), &[refuse(PoleisError::NotPassed)]);

    // the whole floor, and three chairs of twelve
    let mut floor = pro(&founders, 3);
    floor.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO), (&carol, VOTE_PRO)]);
    let h2 = open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT], 3);
    w.cast(&crank, &h2, ROOT_UNIT, &floor);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h2, ROOT_UNIT), &[ok()]);
    let s = w.session(&h2, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.state), (3, 6, STATE_DEADLOCK), "the chairs under a third still block");
}

#[test]
fn neither_chamber_is_rejected_and_silence_expires_under_the_sum() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_BUDGET, SOL, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 4 };
    let mut few = pro(&founders, 3);
    few.push((&carol, VOTE_PRO));
    for (nonce, sum) in [(2u64, true), (3, false)] {
        let h = if sum { open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT], nonce) } else { w.open(&alice.key, &budget, &[ROOT_UNIT], nonce) };
        w.cast(&crank, &h, ROOT_UNIT, &few);
        w.warp(w.slot() + WINDOW + 1);
        w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
        assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_REJECTED, "neither chamber, with the SUM ({sum}) as without it");
    }
    let h = open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT], 4);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_EXPIRED, "nobody voted");
}

#[test]
fn the_sum_is_refused_at_propose_where_the_law_does_not_grant_it() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let sum_of = |p: &Payload, nonce: u64| propose_sum(alice.key, hash_of(p, nonce), MANIFEST, p, &[ROOT_UNIT]);

    // a unit with no law of its own: no rule account, no SUM
    w.run(sum_of(&budget, 1), &[refuse(PoleisError::SumNotAllowed)]);
    // the law turns on the transfer alone: a budget asks in vain
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_TRANSFER, SOL, 2);
    w.run(sum_of(&budget, 3), &[refuse(PoleisError::SumNotAllowed)]);

    // every bit on: the kinds that can never take the SUM are refused all the same
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_KINDS, SOL, 4);
    let never = [
        Payload::Resolution { origin: ROOT_UNIT, sum_kinds: SUM_KINDS, small_cap: u64::MAX },
        Payload::Recall { origin: ROOT_UNIT, chair: 0 },
        Payload::Weight { origin: ROOT_UNIT, holder: key(&alice.key), kind: HOLDER_KEY, weight: 9 },
        Payload::Dissolve { origin: ROOT_UNIT },
        Payload::Allowance { origin: ROOT_UNIT, payee: key(&lab), payee_kind: HOLDER_KEY, mint: Pubkey::default(), cap: 1, period: WINDOW, expires: 0 },
    ];
    for (i, p) in never.iter().enumerate() {
        w.run(sum_of(p, 10 + i as u64), &[refuse(PoleisError::SumNotAllowed)]);
    }

    // the small cap: one lamport over is refused, the cap itself is granted
    let over = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL + 1 };
    w.run(sum_of(&over, 20), &[refuse(PoleisError::SumNotAllowed)]);
    let at = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL };
    w.run(sum_of(&at, 21), &[ok()]);
    // a transfer's sum stands elsewhere in its payload, and is read there
    let far = Payload::Transfer { origin: ROOT_UNIT, dest: 1, amount: SOL + 1 };
    w.run(sum_of(&far, 26), &[refuse(PoleisError::SumNotAllowed)]);
    let near = Payload::Transfer { origin: ROOT_UNIT, dest: 1, amount: SOL };
    w.run(sum_of(&near, 27), &[ok()]);

    // never together with consent
    let mut both = sum_of(&budget, 22);
    both.data[33] |= CLASS_CONSENT;
    w.run(both, &[refuse(PoleisError::SumNotAllowed)]);
    // another unit's law is not this origin's
    let mut elsewhere = sum_of(&budget, 23);
    elsewhere.accounts.last_mut().unwrap().pubkey = rule_pda(1);
    w.run(elsewhere, &[refuse(PoleisError::InvalidPda)]);
    // the bit without its account, an account without the bit: the count answers
    let mut short = sum_of(&budget, 24);
    short.accounts.pop();
    w.run(short, &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    let mut long = propose(alice.key, hash_of(&budget, 25), MANIFEST, &budget, &[ROOT_UNIT]);
    long.accounts.push(AccountMeta::new_readonly(rule_pda(ROOT_UNIT), false));
    w.run(long, &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    // and nothing refused above left a proposal behind
    for nonce in [1u64, 3, 22, 23, 24, 25] {
        assert!(w.data(&proposal_pda(&hash_of(&budget, nonce))).is_empty(), "a refused proposal {nonce} left an account");
    }
    assert!(w.data(&proposal_pda(&hash_of(&over, 20))).is_empty());
}

#[test]
fn a_resolution_names_only_the_four_kinds_and_zero_turns_the_sum_off() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    // a bit outside the four kinds is no law
    let bad = Payload::Resolution { origin: ROOT_UNIT, sum_kinds: SUM_KINDS + 1, small_cap: SOL };
    let h = decide_root(&mut w, &crank, &alice.key, &and, &bad, 1);
    w.run(execute(crank, h, &[ROOT_UNIT], &bad), &[refuse(PoleisError::BadPayload)]);
    assert!(RuleView::parse(&w.data(&rule_pda(ROOT_UNIT))).is_none());

    // on, then off: sum_kinds 0 turns the SUM off for every kind
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    resolve_root(&mut w, &crank, &alice.key, &and, SUM_BUDGET, SOL, 2);
    let open = open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT], 3);
    let mut split = pro(&founders, 5);
    split.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &open, ROOT_UNIT, &split);
    resolve_root(&mut w, &crank, &alice.key, &and, 0, SOL, 4);
    assert_eq!(RuleView::parse(&w.data(&rule_pda(ROOT_UNIT))).expect("the rule").sum_kinds, 0);
    w.run(propose_sum(alice.key, hash_of(&budget, 5), MANIFEST, &budget, &[ROOT_UNIT]), &[refuse(PoleisError::SumNotAllowed)]);
    // the grant is the opening's: the vote that was open when the law changed
    // is still settled by the SUM it was granted
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, open, ROOT_UNIT), &[ok()]);
    let s = w.session(&open, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.state), (5, 5, STATE_PASSED), "a law changed while the vote is open changes nothing in it");
}

#[test]
fn a_crews_own_session_seals_by_the_and() {
    let Estate { mut w, crank, founders, child_founders, alice, bob, .. } = estate(true);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    // the root's law turns the SUM on for a budget, decided over the tree under the AND
    let law = Payload::Resolution { origin: ROOT_UNIT, sum_kinds: SUM_BUDGET, small_cap: SOL };
    let h = w.open(&alice.key, &law, &[ROOT_UNIT, 1], 1);
    crew_speaks(&mut w, crank, h, &child_founders);
    w.cast(&crank, &h, ROOT_UNIT, &and_votes(&founders, &alice, &bob));
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    w.run(execute(crank, h, &[ROOT_UNIT, 1], &law), &[ok()]);

    // the crew grows a floor of its own: its first rung by its own CODEX
    // (no floor yet, so its chairs decide), then a sale of four seats
    let climb = Payload::Ladder { origin: 1, reserve: 1_000_000, deposit: 5_000_000, commit_len: 100, reveal_len: 100, sale_id: 0 };
    let hc = w.open(&child_founders[0].key, &climb, &[1], 2);
    w.cast(&crank, &hc, 1, &pro(&child_founders, 7));
    w.run(seal(crank, hc, 1), &[ok()]);
    w.run(execute(crank, hc, &[1], &climb), &[ok()]);
    let (xena, yuri) = (w.voter(SOL), w.voter(SOL));
    let salt = [5u8; 32];
    let bids = [(&xena, 0u64), (&xena, 1), (&xena, 2), (&yuri, 0)];
    let t = w.slot();
    for (who, nonce) in bids {
        w.run(commit_seat_bid(who.key, 1, 0, nonce, seat_commitment(1_000_000, nonce, &salt, &who.key)), &[ok()]);
    }
    w.warp(t + 100);
    for (who, nonce) in bids {
        w.run(reveal_seat_bid(who.key, 1, 0, nonce, 1_000_000, salt), &[ok()]);
    }
    w.warp(t + 200);
    w.run(settle_seats(crank, 1, 0), &[ok()]);
    for (who, nonce) in bids {
        w.run(claim_seat(who.key, 1, 0, nonce, ROOT_UNIT), &[ok()]);
    }
    assert_eq!(w.registry(1).total_seats, 4);

    // a SUM budget over the tree, split the same way in both: five chairs
    // short of their majority, most of the floor PRO
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let hs = open_sum(&mut w, &alice.key, &budget, &[ROOT_UNIT, 1], 3);
    let mut crew = pro(&child_founders, 5);
    crew.push((&xena, VOTE_PRO));
    w.cast(&crank, &hs, 1, &crew);
    let mut root = pro(&founders, 5);
    root.extend([(&alice, VOTE_PRO), (&bob, VOTE_PRO)]);
    w.cast(&crank, &hs, ROOT_UNIT, &root);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, hs, 1), &[ok()]);
    let c = w.session(&hs, 1);
    assert_eq!((c.pent_pro, c.hex_pro, c.active_weight), (5, 3, 4));
    assert!(sum_passes(5, 11, 3, 4), "the same split would pass at an origin");
    assert_eq!(c.state, STATE_DEADLOCK, "a crew's own session seals by the AND");
    w.run(mandate(crank, hs, ROOT_UNIT, 1), &[ok()]);
    w.run(seal(crank, hs, ROOT_UNIT), &[ok()]);
    let s = w.session(&hs, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.pent_abst, s.hex_pro, s.active_weight), (5, 1, 5, 6), "the crew's split is chair XII's abstention");
    assert_eq!(s.state, STATE_PASSED, "the origin's own split is settled by the SUM");
    let l0 = w.lamports(&lab);
    w.run(execute(crank, hs, &[ROOT_UNIT, 1], &budget), &[ok()]);
    assert_eq!(w.lamports(&lab) - l0, SOL / 2);
}

// ================= a verdict reaches the seal: the market's half of the ruling =================

use poleis_client::{find_pda, ix as poleis_ix, system_meta};
use poleis_treasury::{
    ACT_RAW, CLASS_ACT, CLASS_MARKET, CLASS_PLAN, CLASS_SUM, CODEX_MARKET, IX_EXECUTE, MARKET_ACT, MARKET_CLASSES, MARKET_SEAT, MARKET_TREASURY, PLAN_CLASSES,
};

/// A stand-in for the roads program: the law names it, and its roads and boards are bytes laid by hand at its addresses.
const ROADS: Pubkey = Pubkey::new_from_array([0x52; 32]);
const BOARD: u64 = 7;
const CREDITED: u64 = 10_000;

fn market_pda(unit: u64) -> Pubkey {
    find_pda(&[b"market", &unit.to_le_bytes()])
}
fn road_at(program: &Pubkey, board: u64, hash: &[u8; 32]) -> Pubkey {
    Pubkey::find_program_address(&[b"road", &board.to_le_bytes(), hash], program).0
}

/// An account written by hand: replaced if it stands, laid if not.
fn put(w: &mut World, key: Pubkey, owner: Pubkey, data: Vec<u8>) {
    let lamports = w.rent(data.len()).max(1);
    let acc = Account { lamports, data, owner, executable: false, rent_epoch: 0 };
    match w.accounts.iter_mut().find(|(k, _)| *k == key) {
        Some(slot) => slot.1 = acc,
        None => w.accounts.push((key, acc)),
    }
}

/// A Road of the roads program's layout version 1, as its program lays it: the public face, its window, the deposits and the board's
/// `credited` at its lay (463 bytes since THE BOARD AS IT STOOD AT THE LAY).
#[allow(clippy::too_many_arguments)]
fn road_bytes(board: u64, hash: &[u8; 32], verdict: u8, deposits: u64, opens: u64, closes: u64, credited_at_lay: u64) -> Vec<u8> {
    let mut d = vec![0u8; 463];
    d[0] = 3;
    d[1] = 1;
    d[2..10].copy_from_slice(&board.to_le_bytes());
    d[10..42].copy_from_slice(hash);
    d[42] = verdict;
    d[75..83].copy_from_slice(&opens.to_le_bytes());
    d[83..91].copy_from_slice(&closes.to_le_bytes());
    d[361..369].copy_from_slice(&deposits.to_le_bytes());
    d[454..462].copy_from_slice(&credited_at_lay.to_le_bytes());
    d
}
/// The vote's window: the proposal's opening slot and the origin session's deadline.
fn vote_window(w: &World, hash: &[u8; 32]) -> (u64, u64) {
    let opened = u64::from_le_bytes(w.data(&proposal_pda(hash))[Proposal::OFF_OPENED..Proposal::OFF_OPENED + 8].try_into().unwrap());
    (opened, w.session(hash, ROOT_UNIT).deadline)
}
/// A road laid by hand with the window `opens..closes` and the board's `credited` at its lay.
#[allow(clippy::too_many_arguments)]
fn lay_road_at(w: &mut World, hash: &[u8; 32], verdict: u8, deposits: u64, opens: u64, closes: u64, credited_at_lay: u64) {
    put(w, road_at(&ROADS, BOARD, hash), ROADS, road_bytes(BOARD, hash, verdict, deposits, opens, closes, credited_at_lay));
}
/// The proposal's road, its window the vote's own, on a board that had credited `CREDITED` when it was laid.
fn lay_road(w: &mut World, hash: &[u8; 32], verdict: u8, deposits: u64) {
    let (opened, deadline) = vote_window(w, hash);
    lay_road_at(w, hash, verdict, deposits, opened, deadline, CREDITED);
}
const OPEN: u8 = 0;
const PASS: u8 = 1;
const FAIL: u8 = 2;

/// The CODEX act that writes a unit's market: [class][kind][program:32][board:8][split:1][plan:1][min_stood_bp:2].
fn market_law(program: &Pubkey, board: u64, split: u8, plan: u8, bar: u16) -> Vec<u8> {
    let mut v = vec![CLASS_CODEX, CODEX_MARKET];
    v.extend_from_slice(program.as_ref());
    v.extend_from_slice(&board.to_le_bytes());
    v.extend_from_slice(&[split, plan]);
    v.extend_from_slice(&bar.to_le_bytes());
    v
}

/// The root's market law, carried by the AND (seven chairs and five of six seats, decided at once) and executed: `then` judges the execute.
#[allow(clippy::too_many_arguments)]
fn law_root(w: &mut World, crank: &Pubkey, proposer: &Pubkey, votes: &[(&Voter, u8)], law: &[u8], nonce: u64, then: Check) {
    // a law that turns the market on names a board that is the unit's own
    if law[42] | law[43] != 0 {
        lay_our_board(w, law);
    }
    law_root_as_is(w, crank, proposer, votes, law, nonce, then);
}
/// The same, with whatever stands at the board's address left as it is.
#[allow(clippy::too_many_arguments)]
fn law_root_as_is(w: &mut World, crank: &Pubkey, proposer: &Pubkey, votes: &[(&Voter, u8)], law: &[u8], nonce: u64, then: Check) {
    let h = decide_law(w, crank, proposer, votes, law, nonce);
    w.run(execute_law(*crank, h, &[ROOT_UNIT], law), &[then]);
    // a polis that sets its market's law proposes under it in a later slot
    w.warp(w.slot() + 1);
}
/// A market law of the root, proposed and sealed PASSED, not yet executed: its hash.
fn decide_law(w: &mut World, crank: &Pubkey, proposer: &Pubkey, votes: &[(&Voter, u8)], law: &[u8], nonce: u64) -> [u8; 32] {
    let h = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &sha(&[law]), nonce);
    w.run(propose_raw(*proposer, h, CLASS_CODEX, ROOT_UNIT, MANIFEST, law, &[ROOT_UNIT]), &[ok()]);
    w.cast(crank, &h, ROOT_UNIT, votes);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(*crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    h
}
/// The program and the board a market law names.
fn law_board(law: &[u8]) -> (Pubkey, u64) {
    (Pubkey::new_from_array(law[2..34].try_into().unwrap()), u64::from_le_bytes(law[34..42].try_into().unwrap()))
}
/// A Board of the roads program's layout version 1 that is `unit`'s own: its authority the unit's agent key, its law naming Poleis and
/// the unit (207 bytes since the house's `earned`; the law's terms are the roads program's to check, and Poleis reads none of them).
fn board_bytes(board: u64, authority: &Pubkey, polis: &Pubkey, unit: u64) -> Vec<u8> {
    let mut d = vec![0u8; 207];
    d[0] = 1;
    d[1] = 1;
    d[2..10].copy_from_slice(&board.to_le_bytes());
    d[10..42].copy_from_slice(authority.as_ref());
    d[114..146].copy_from_slice(polis.as_ref());
    d[146..154].copy_from_slice(&unit.to_le_bytes());
    d
}
fn board_at(program: &Pubkey, board: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"board", &board.to_le_bytes()], program).0
}
/// The board a market law names, laid as the root's own.
fn lay_our_board(w: &mut World, law: &[u8]) {
    let (program, board) = law_board(law);
    put(w, board_at(&program, board), program, board_bytes(board, &agent_pda(ROOT_UNIT), &PROGRAM_ID, ROOT_UNIT));
}
/// `execute` of a market law over `impacted`: the sessions, the registries (CODEX), then the origin's registry, its market, the system, and
/// for a law that turns the market on, the board it names.
fn execute_law(crank: Pubkey, h: [u8; 32], impacted: &[u64], law: &[u8]) -> Instruction {
    let mut metas = vec![AccountMeta::new(crank, true), AccountMeta::new_readonly(config_pda(), false), AccountMeta::new(proposal_pda(&h), false)];
    for u in impacted {
        metas.push(AccountMeta::new_readonly(session_pda(&h, *u), false));
    }
    for u in impacted {
        metas.push(AccountMeta::new_readonly(registry_pda(*u), false));
    }
    metas.extend([AccountMeta::new_readonly(registry_pda(ROOT_UNIT), false), AccountMeta::new(market_pda(ROOT_UNIT), false), system_meta()]);
    if law[42] | law[43] != 0 {
        let (program, board) = law_board(law);
        metas.push(AccountMeta::new_readonly(board_at(&program, board), false));
    }
    let mut d = vec![IX_EXECUTE];
    d.extend_from_slice(&h);
    poleis_ix(d, metas)
}
/// The market law every test of a seal starts from: every class may split to the market, money and the hands may be plans, no bar.
fn open_market(w: &mut World, crank: &Pubkey, proposer: &Pubkey, votes: &[(&Voter, u8)], bar: u16, nonce: u64) {
    law_root(w, crank, proposer, votes, &market_law(&ROADS, BOARD, MARKET_CLASSES, PLAN_CLASSES, bar), nonce, ok());
}
/// The root's market as it stands: (unit, program, board, split, plan, min_stood_bp, since), None while there is none.
fn market_of(w: &World) -> Option<(u64, Pubkey, u64, u8, u8, u16, u64)> {
    let d = w.data(&market_pda(ROOT_UNIT));
    if d.is_empty() {
        return None;
    }
    assert_eq!((d.len(), d[0], d[1]), (63, 27, 8), "a Market: 63 bytes, discriminator 27, the layout version");
    Some((
        u64::from_le_bytes(d[2..10].try_into().unwrap()),
        Pubkey::new_from_array(d[10..42].try_into().unwrap()),
        u64::from_le_bytes(d[42..50].try_into().unwrap()),
        d[50],
        d[51],
        u16::from_le_bytes([d[52], d[53]]),
        u64::from_le_bytes(d[54..62].try_into().unwrap()),
    ))
}

/// A proposal of `payload` over `impacted` that asks `bits`, with the origin's market last.
fn ask(proposer: &Pubkey, payload: &Payload, impacted: &[u64], bits: u8, nonce: u64) -> Instruction {
    let mut ix = propose_raw(*proposer, hash_of(payload, nonce), payload.class() | bits, payload.origin(), MANIFEST, &payload.bytes(), impacted);
    ix.accounts.push(AccountMeta::new_readonly(market_pda(payload.origin()), false));
    ix
}
fn open_asking(w: &mut World, proposer: &Pubkey, payload: &Payload, impacted: &[u64], bits: u8, nonce: u64) -> [u8; 32] {
    w.run(ask(proposer, payload, impacted, bits, nonce), &[ok()]);
    hash_of(payload, nonce)
}
/// The origin's seal of a proposal that asks the market: its market and its road after the five.
fn seal_market(crank: Pubkey, h: [u8; 32]) -> Instruction {
    let mut ix = seal(crank, h, ROOT_UNIT);
    ix.accounts.extend([
        AccountMeta::new_readonly(market_pda(ROOT_UNIT), false),
        AccountMeta::new_readonly(road_at(&ROADS, BOARD, &h), false),
    ]);
    ix
}

/// The four ways the root's two chambers stand on a question: (chairs, floor).
fn chairs_not_floor<'a>(founders: &'a [Voter], carol: &'a Voter) -> Vec<(&'a Voter, u8)> {
    let mut v = pro(founders, 7);
    v.push((carol, VOTE_PRO));
    v
}
fn floor_not_chairs<'a>(founders: &'a [Voter], alice: &'a Voter, bob: &'a Voter) -> Vec<(&'a Voter, u8)> {
    let mut v = pro(founders, 3);
    v.extend([(alice, VOTE_PRO), (bob, VOTE_PRO)]);
    v
}
fn neither<'a>(founders: &'a [Voter], carol: &'a Voter) -> Vec<(&'a Voter, u8)> {
    let mut v = pro(founders, 3);
    v.push((carol, VOTE_PRO));
    v
}

/// A question voted, its road laid as `road` says (None: never laid), its window waited out, sealed by the origin with its market:
/// returns the hash once the seal is judged by `then`.
#[allow(clippy::too_many_arguments)]
fn decide_by_market(
    w: &mut World,
    crank: &Pubkey,
    proposer: &Pubkey,
    payload: &Payload,
    bits: u8,
    votes: &[(&Voter, u8)],
    road: Option<(u8, u64)>,
    nonce: u64,
    then: Check,
) -> [u8; 32] {
    let h = open_asking(w, proposer, payload, &[ROOT_UNIT], bits, nonce);
    w.cast(crank, &h, ROOT_UNIT, votes);
    if let Some((verdict, deposits)) = road {
        lay_road(w, &h, verdict, deposits);
    }
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(*crank, h), &[then]);
    h
}

#[test]
fn a_units_own_law_names_its_market_by_codex() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    assert!(market_of(&w).is_none(), "no unit reads a market until its own law names one");
    law_root(&mut w, &crank, &alice.key, &and, &market_law(&ROADS, BOARD, MARKET_TREASURY, 0, 500), 1, ok());
    let since = w.slot() - 1;
    assert_eq!(market_of(&w), Some((ROOT_UNIT, ROADS, BOARD, MARKET_TREASURY, 0, 500, since)));
    assert_eq!(w.lamports(&market_pda(ROOT_UNIT)), w.rent(63), "the crank paid its rent");

    // a later act replaces it, its slot the new `since`
    let other = Pubkey::new_from_array([0x53; 32]);
    law_root(&mut w, &crank, &alice.key, &and, &market_law(&other, 9, MARKET_SEAT | MARKET_ACT, MARKET_ACT, 10_000), 2, ok());
    let since2 = w.slot() - 1;
    assert!(since2 > since);
    assert_eq!(market_of(&w), Some((ROOT_UNIT, other, 9, MARKET_SEAT | MARKET_ACT, MARKET_ACT, 10_000, since2)));

    // a bit outside the three classes, a seat as a plan, a bar over the whole board: no law, and the law stands as it was
    for (i, bad) in [
        market_law(&ROADS, BOARD, MARKET_CLASSES + 1, 0, 0),
        market_law(&ROADS, BOARD, 0x80, 0, 0),
        market_law(&ROADS, BOARD, MARKET_TREASURY, MARKET_SEAT, 0),
        market_law(&ROADS, BOARD, MARKET_TREASURY, PLAN_CLASSES | 8, 0),
        market_law(&ROADS, BOARD, MARKET_TREASURY, 0, 10_001),
        market_law(&ROADS, BOARD, MARKET_TREASURY, 0, u16::MAX),
    ]
    .iter()
    .enumerate()
    {
        law_root(&mut w, &crank, &alice.key, &and, bad, 10 + i as u64, refuse(PoleisError::BadPayload));
        assert_eq!(market_of(&w), Some((ROOT_UNIT, other, 9, MARKET_SEAT | MARKET_ACT, MARKET_ACT, 10_000, since2)));
    }

    // both masks 0 turn the market off: the account stands and grants nothing
    law_root(&mut w, &crank, &alice.key, &and, &market_law(&ROADS, BOARD, 0, 0, 0), 3, ok());
    assert_eq!(market_of(&w), Some((ROOT_UNIT, ROADS, BOARD, 0, 0, 0, w.slot() - 1)));
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    for (bits, nonce) in [(CLASS_MARKET, 4u64), (CLASS_PLAN, 5)] {
        w.run(ask(&alice.key, &budget, &[ROOT_UNIT], bits, nonce), &[refuse(PoleisError::MarketNotAllowed)]);
    }
}

#[test]
fn the_market_is_refused_at_propose_where_the_law_does_not_grant_it() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let recall = Payload::Recall { origin: ROOT_UNIT, chair: 0 };
    let mut refused: Vec<[u8; 32]> = Vec::new();
    let mut no = |w: &mut World, ix: Instruction, why: Check<'static>| {
        let h: [u8; 32] = ix.data[1..33].try_into().unwrap();
        w.run(ix, &[why]);
        refused.push(h);
    };

    // a unit with no law of its own: no market account, no market
    no(&mut w, ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 1), refuse(PoleisError::MarketNotAllowed));
    no(&mut w, ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_PLAN, 2), refuse(PoleisError::MarketNotAllowed));

    // a law that names a seat's split and the hands' plans: money asks in vain, in either mode
    law_root(&mut w, &crank, &alice.key, &and, &market_law(&ROADS, BOARD, MARKET_SEAT, MARKET_ACT, 0), 3, ok());
    no(&mut w, ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 4), refuse(PoleisError::MarketNotAllowed));
    no(&mut w, ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_PLAN, 5), refuse(PoleisError::MarketNotAllowed));
    // and the seat's split is granted
    w.run(ask(&alice.key, &recall, &[ROOT_UNIT], CLASS_MARKET, 6), &[ok()]);

    // every bit of the law on: the law and the signals never ask, a seat is never a plan
    law_root(&mut w, &crank, &alice.key, &and, &market_law(&ROADS, BOARD, MARKET_CLASSES, PLAN_CLASSES, 0), 7, ok());
    let never = [
        Payload::Resolution { origin: ROOT_UNIT, sum_kinds: SUM_KINDS, small_cap: 1 },
        Payload::Weight { origin: ROOT_UNIT, holder: key(&alice.key), kind: HOLDER_KEY, weight: 9 },
    ];
    for (i, p) in never.iter().enumerate() {
        for (j, bits) in [CLASS_MARKET, CLASS_PLAN].into_iter().enumerate() {
            no(&mut w, ask(&alice.key, p, &[ROOT_UNIT], bits, 20 + (2 * i + j) as u64), refuse(PoleisError::MarketNotAllowed));
        }
    }
    no(&mut w, ask(&alice.key, &recall, &[ROOT_UNIT], CLASS_PLAN, 30), refuse(PoleisError::MarketNotAllowed));

    // never two of the four bits at once
    for (i, bits) in [
        CLASS_MARKET | CLASS_CONSENT,
        CLASS_MARKET | CLASS_SUM,
        CLASS_MARKET | CLASS_PLAN,
        CLASS_PLAN | CLASS_CONSENT,
        CLASS_PLAN | CLASS_SUM,
        CLASS_MARKET | CLASS_PLAN | CLASS_SUM | CLASS_CONSENT,
    ]
    .into_iter()
    .enumerate()
    {
        no(&mut w, ask(&alice.key, &budget, &[ROOT_UNIT], bits, 40 + i as u64), refuse(PoleisError::MarketNotAllowed));
    }
    // another unit's law is not this origin's
    let mut elsewhere = ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 50);
    elsewhere.accounts.last_mut().unwrap().pubkey = market_pda(1);
    no(&mut w, elsewhere, refuse(PoleisError::MarketNotAllowed));
    // the bit without its account, an account without the bit: the count answers
    let mut short = ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_PLAN, 51);
    short.accounts.pop();
    no(&mut w, short, Check::err(ProgramError::NotEnoughAccountKeys));
    let mut long = propose(alice.key, hash_of(&budget, 52), MANIFEST, &budget, &[ROOT_UNIT]);
    long.accounts.push(AccountMeta::new_readonly(market_pda(ROOT_UNIT), false));
    no(&mut w, long, Check::err(ProgramError::NotEnoughAccountKeys));

    // granted: money in either mode, a seat's split; and nothing refused above left a proposal behind
    w.run(ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 60), &[ok()]);
    w.run(ask(&alice.key, &budget, &[ROOT_UNIT], CLASS_PLAN, 61), &[ok()]);
    w.run(ask(&alice.key, &recall, &[ROOT_UNIT], CLASS_MARKET, 62), &[ok()]);
    assert_eq!(w.data(&proposal_pda(&hash_of(&budget, 61)))[Proposal::OFF_CLASS], CLASS_TREASURY | CLASS_PLAN, "the byte is stored as sent");
    assert!(refused.len() >= 18);
    for h in &refused {
        assert!(w.data(&proposal_pda(h)).is_empty(), "a refused proposal left an account");
    }
}

#[test]
fn a_split_the_chairs_passed_goes_to_the_market() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let and = and_votes(&founders, &alice, &bob);
    open_market(&mut w, &crank, &alice.key, &and, 0, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let split = chairs_not_floor(&founders, &carol);

    // the chairs pass, the floor does not, the road passed: PASSED, and the budget is paid
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((PASS, 100)), 2, ok());
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.state), (7, 1, STATE_PASSED), "seven chairs, one seat of six, and the road passed");
    let (l0, spent0) = (w.lamports(&lab), w.registry(ROOT_UNIT).plan_spent);
    w.run(execute(crank, h, &[ROOT_UNIT], &budget), &[ok()]);
    assert_eq!(w.lamports(&lab) - l0, SOL / 2);
    assert_eq!(w.registry(ROOT_UNIT).plan_spent - spent0, SOL / 2, "charged to the plan");

    // the same split, the road failed: it stands, and nothing moves
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((FAIL, 100)), 3, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);
    let l1 = w.lamports(&lab);
    w.run(execute(crank, h, &[ROOT_UNIT], &budget), &[refuse(PoleisError::NotPassed)]);
    assert_eq!(w.lamports(&lab), l1, "nothing moved");

    // no road laid: the market says nothing
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, None, 4, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);

    // the road still open: the seal waits for it, then the road is decided, then it seals
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((OPEN, 100)), 5, refuse(PoleisError::MarketOpen));
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_VOTING, "a refused seal leaves the session open");
    lay_road(&mut w, &h, PASS, 100);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);

    // the floor passes, the chairs do not, the road passed: the chairs keep their veto; an open road does not hold it
    let up = floor_not_chairs(&founders, &alice, &bob);
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &up, Some((PASS, 100)), 6, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "a split where the chairs refused stands, whatever the road says");
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &up, Some((OPEN, 100)), 7, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);

    // both chambers pass: PASSED though the road failed, and early, by the AND's own fast path
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 8);
    w.cast(&crank, &h, ROOT_UNIT, &and);
    lay_road(&mut w, &h, FAIL, 100);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);

    // neither: REJECTED though the road passed
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &neither(&founders, &carol), Some((PASS, 100)), 9, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_REJECTED);

    // nobody voted: EXPIRED, whatever the road says
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &[], Some((PASS, 100)), 10, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_EXPIRED);
}

#[test]
fn a_plan_passes_by_the_chairs_and_the_road() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    let and = and_votes(&founders, &alice, &bob);
    open_market(&mut w, &crank, &alice.key, &and, 0, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };

    // the chairs pass and the road passed, the floor silent: PASSED, and the budget is paid
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_PLAN, &pro(&founders, 7), Some((PASS, 100)), 2, ok());
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.state), (7, 0, STATE_PASSED));
    let l0 = w.lamports(&lab);
    w.run(execute(crank, h, &[ROOT_UNIT], &budget), &[ok()]);
    assert_eq!(w.lamports(&lab) - l0, SOL / 2);

    // the floor against: its ballot is not counted
    let mut against = pro(&founders, 7);
    against.extend([(&alice, VOTE_CONTRA), (&bob, VOTE_CONTRA), (&carol, VOTE_CONTRA)]);
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_PLAN, &against, Some((PASS, 100)), 3, ok());
    let s = w.session(&h, ROOT_UNIT);
    assert_eq!((s.hex_contra, s.state), (6, STATE_PASSED), "the floor speaks by price");

    // the chairs pass, the floor for, the road failed: the floor's weight never seals it early, and it stands
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_PLAN, 4);
    w.cast(&crank, &h, ROOT_UNIT, &and);
    lay_road(&mut w, &h, FAIL, 100);
    w.run(seal_market(crank, h), &[refuse(PoleisError::SessionOpen)]);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);
    // the same votes without the bit: the AND's fast path seals at once
    let h = w.open(&alice.key, &budget, &[ROOT_UNIT], 5);
    w.cast(&crank, &h, ROOT_UNIT, &and);
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);

    // the chairs refuse, the road passed: DEADLOCK
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_PLAN, &floor_not_chairs(&founders, &alice, &bob), Some((PASS, 100)), 6, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "the chairs keep their veto");
    // neither: REJECTED
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_PLAN, &neither(&founders, &carol), Some((FAIL, 100)), 7, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_REJECTED);
    // no road laid: the chairs alone cannot pass a plan
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_PLAN, &pro(&founders, 7), None, 8, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);
    // an open road holds a plan's seal whatever the chairs said
    for (nonce, votes) in [(9u64, pro(&founders, 7)), (10, neither(&founders, &carol))] {
        let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_PLAN, &votes, Some((OPEN, 100)), nonce, refuse(PoleisError::MarketOpen));
        lay_road(&mut w, &h, FAIL, 100);
        w.run(seal_market(crank, h), &[ok()]);
        assert_eq!(w.session(&h, ROOT_UNIT).state, if nonce == 9 { STATE_DEADLOCK } else { STATE_REJECTED });
    }
}

#[test]
fn the_bar_a_road_with_too_little_of_the_board_does_not_count() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    // a tenth of the board must have stood in the road
    open_market(&mut w, &crank, &alice.key, &and, 1_000, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let split = chairs_not_floor(&founders, &carol);

    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((PASS, CREDITED / 10 - 1)), 2, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "one under a tenth: the split stands");
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((PASS, CREDITED / 10)), 3, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED, "exactly a tenth counts");
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_PLAN, &pro(&founders, 7), Some((PASS, CREDITED / 10 - 1)), 4, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "under the bar a plan cannot pass");
    // THE BOARD AS IT STOOD AT THE LAY: the bar is read off the road alone. The board account leaves the world (only the market's law read
    // it, when it was written), and a road whose board had credited ten thousand at its lay counts its tenth, whatever was credited since
    let board = Pubkey::find_program_address(&[b"board", &BOARD.to_le_bytes()], &ROADS).0;
    w.accounts.retain(|(k, _)| *k != board);
    assert!(w.accounts.iter().all(|(k, _)| *k != board));
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 5);
    w.cast(&crank, &h, ROOT_UNIT, &split);
    let (opened, deadline) = vote_window(&w, &h);
    lay_road_at(&mut w, &h, PASS, CREDITED / 10, opened, deadline, CREDITED);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED, "a credit after the lay, however large, does not move a passed road under the bar");
    // the snapshot is what the bar reads: the same deposits against a larger board at its lay do not count
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 6);
    w.cast(&crank, &h, ROOT_UNIT, &split);
    let (opened, deadline) = vote_window(&w, &h);
    lay_road_at(&mut w, &h, PASS, CREDITED / 10, opened, deadline, CREDITED + 1);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);
    // a road too short to hold the snapshot (laid by an older build) says nothing, and holds nothing open
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_PLAN, 9);
    w.cast(&crank, &h, ROOT_UNIT, &pro(&founders, 7));
    let (opened, deadline) = vote_window(&w, &h);
    let mut old = road_bytes(BOARD, &h, OPEN, CREDITED, opened, deadline, CREDITED);
    old.truncate(455);
    put(&mut w, road_at(&ROADS, BOARD, &h), ROADS, old);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);

    // a bar of 0 counts any road somebody stood in, one micro will do; a road nobody stood in says nothing, whatever the bar
    open_market(&mut w, &crank, &alice.key, &and, 0, 7);
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((PASS, 1)), 8, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((PASS, 0)), 10, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "a decided PASSED road nobody deposited in says nothing, bar 0");
}

#[test]
fn a_road_that_cannot_count_never_holds_a_seal_and_one_nobody_stood_in_says_nothing() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    // a tenth of the board must stand in a road
    open_market(&mut w, &crank, &alice.key, &and, 1_000, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let split = chairs_not_floor(&founders, &carol);
    let chairs = pro(&founders, 7);
    // decided PASSED with nothing in it, under a bar: nothing
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((PASS, 0)), 2, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);
    // open past its close (the seal comes after the deadline, the road closes at it) with nothing in it, and under the bar: no hold;
    // the split stands and the plan cannot pass
    let mut nonce = 10;
    for deposits in [0, CREDITED / 10 - 1] {
        for (bits, votes) in [(CLASS_MARKET, &split), (CLASS_PLAN, &chairs)] {
            nonce += 1;
            let h = decide_by_market(&mut w, &crank, &alice.key, &budget, bits, votes, Some((OPEN, deposits)), nonce, ok());
            assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "an open road past its close that can no longer count holds nothing ({deposits})");
        }
    }
    // open past its close over the bar: it holds the seal until it is decided
    for (bits, votes) in [(CLASS_MARKET, &split), (CLASS_PLAN, &chairs)] {
        nonce += 1;
        let h = decide_by_market(&mut w, &crank, &alice.key, &budget, bits, votes, Some((OPEN, CREDITED / 10)), nonce, refuse(PoleisError::MarketOpen));
        lay_road(&mut w, &h, PASS, CREDITED / 10);
        w.run(seal_market(crank, h), &[ok()]);
        assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);
    }
    // a bar of 0: an open road past its close that nobody deposited in can no longer count, and holds nothing either
    open_market(&mut w, &crank, &alice.key, &and, 0, 30);
    for (bits, votes) in [(CLASS_MARKET, &split), (CLASS_PLAN, &chairs)] {
        nonce += 1;
        let h = decide_by_market(&mut w, &crank, &alice.key, &budget, bits, votes, Some((OPEN, 0)), nonce, ok());
        assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "bar 0, an open road past its close with nothing in it holds nothing");
    }
    // "an open road before its close holds it whatever stood in it" cannot be met at a seal where the road matters: such a seal comes
    // after the session's deadline (a split the chairs passed and the floor did not, or a plan with a floor, never seals early), and a road
    // that counts closes by that deadline (A ROAD'S WINDOW LIES INSIDE THE VOTE). So every open road a seal that matters reads is past its
    // close, and the rule's first half is the belt of a case the window law already rules out.
}

#[test]
fn the_grant_is_the_openings_a_law_written_since_voids_the_market_for_that_vote() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    open_market(&mut w, &crank, &alice.key, &and, 0, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let split = chairs_not_floor(&founders, &carol);

    // opened under the law, the same law written again while it is open: the market says nothing in it
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 2);
    w.cast(&crank, &h, ROOT_UNIT, &split);
    lay_road(&mut w, &h, PASS, CREDITED);
    open_market(&mut w, &crank, &alice.key, &and, 0, 3);
    let opened = u64::from_le_bytes(w.data(&proposal_pda(&h))[Proposal::OFF_OPENED..Proposal::OFF_OPENED + 8].try_into().unwrap());
    assert!(market_of(&w).unwrap().6 > opened);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "a law written since the opening voids the market for this vote");

    // opened under a bar its road does not reach, the bar lowered while it is open: not rescued
    open_market(&mut w, &crank, &alice.key, &and, 5_000, 4);
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 5);
    w.cast(&crank, &h, ROOT_UNIT, &split);
    lay_road(&mut w, &h, PASS, CREDITED / 10);
    open_market(&mut w, &crank, &alice.key, &and, 0, 6);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "a vote opened under a law is not rescued by a later one");
    // a vote opened under the new law, the same road: it counts
    let h = decide_by_market(&mut w, &crank, &alice.key, &budget, CLASS_MARKET, &split, Some((PASS, CREDITED / 10)), 7, ok());
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED);

    // the opening's own slot: the same law, executed in the slot the vote opened in, after it, is not the law the vote was opened
    // under (`since >= opened` voids), so the market says nothing in it
    let off = market_law(&ROADS, BOARD, MARKET_CLASSES, PLAN_CLASSES, 0);
    let hl = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &sha(&[&off]), 8);
    w.run(propose_raw(alice.key, hl, CLASS_CODEX, ROOT_UNIT, MANIFEST, &off, &[ROOT_UNIT]), &[ok()]);
    w.cast(&crank, &hl, ROOT_UNIT, &and);
    w.run(seal(crank, hl, ROOT_UNIT), &[ok()]);
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 9);
    lay_our_board(&mut w, &off);
    w.run(execute_law(crank, hl, &[ROOT_UNIT], &off), &[ok()]);
    assert_eq!(market_of(&w).unwrap().6, u64::from_le_bytes(w.data(&proposal_pda(&h))[Proposal::OFF_OPENED..Proposal::OFF_OPENED + 8].try_into().unwrap()));
    w.cast(&crank, &h, ROOT_UNIT, &split);
    lay_road(&mut w, &h, PASS, CREDITED);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "a law written in the opening's own slot is not the opening's");
}

#[test]
fn a_crank_cannot_hide_a_road() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    open_market(&mut w, &crank, &alice.key, &and, 0, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let split = chairs_not_floor(&founders, &carol);

    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 2);
    w.cast(&crank, &h, ROOT_UNIT, &split);
    lay_road(&mut w, &h, PASS, 100);
    // another proposal's road, laid failed, in this one's place
    let other = hash_of(&budget, 99);
    lay_road_at(&mut w, &other, FAIL, 100, 0, u64::MAX, CREDITED);
    w.warp(w.slot() + WINDOW + 1);
    let stranger = w.add_wallet(SOL);
    for (k, wrong) in [
        (5usize, rule_pda(ROOT_UNIT)),
        (5, market_pda(1)),
        (6, road_at(&ROADS, BOARD, &other)),
        (6, stranger),
        (6, road_at(&ROADS, BOARD + 1, &h)),
        (6, road_at(&Pubkey::new_from_array([0x53; 32]), BOARD, &h)),
        (5, road_at(&ROADS, BOARD, &h)),
    ] {
        let mut ix = seal_market(crank, h);
        ix.accounts[k].pubkey = wrong;
        w.run(ix, &[refuse(PoleisError::InvalidPda)]);
    }
    // the count: five accounts, or one too many, for a proposal that asks; three more for one that does not
    let mut short = seal_market(crank, h);
    short.accounts.pop();
    w.run(short, &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    w.run(seal(crank, h, ROOT_UNIT), &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    let mut extra = seal_market(crank, h);
    extra.accounts.push(AccountMeta::new_readonly(stranger, false));
    w.run(extra, &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_VOTING);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_PASSED, "the road itself, in its place");
    let plain = w.open(&alice.key, &budget, &[ROOT_UNIT], 3);
    w.cast(&crank, &plain, ROOT_UNIT, &split);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(crank, plain), &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    w.run(seal(crank, plain, ROOT_UNIT), &[ok()]);
    assert_eq!(w.session(&plain, ROOT_UNIT).state, STATE_DEADLOCK, "a proposal that does not ask seals as it always did");

    // at the right address, not a road of the law's program at its version: the market says nothing, and the seal is not held
    let bytes_of = |w: &World, hash: &[u8; 32], f: &dyn Fn(&mut Vec<u8>)| {
        let (opened, deadline) = vote_window(w, hash);
        let mut d = road_bytes(BOARD, hash, OPEN, 100, opened, deadline, CREDITED);
        f(&mut d);
        d
    };
    let shapes: [(&str, Pubkey, &dyn Fn(&mut Vec<u8>)); 6] = [
        ("another owner", Pubkey::new_from_array([0x53; 32]), &|_: &mut Vec<u8>| {}),
        ("the system's", Pubkey::default(), &|_: &mut Vec<u8>| {}),
        ("another discriminator", ROADS, &|d: &mut Vec<u8>| d[0] = 2),
        ("another version", ROADS, &|d: &mut Vec<u8>| d[1] = 2),
        ("too few bytes", ROADS, &|d: &mut Vec<u8>| d.truncate(461)),
        ("another road's id inside", ROADS, &|d: &mut Vec<u8>| d[10] ^= 1),
    ];
    for (i, (what, owner, f)) in shapes.into_iter().enumerate() {
        let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 10 + i as u64);
        w.cast(&crank, &h, ROOT_UNIT, &split);
        let d = bytes_of(&w, &h, f);
        put(&mut w, road_at(&ROADS, BOARD, &h), owner, d);
        w.warp(w.slot() + WINDOW + 1);
        w.run(seal_market(crank, h), &[ok()]);
        assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK, "{what}: an open road there says nothing");
        // the same bytes, decided PASSED, say nothing either
        let h2 = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 20 + i as u64);
        w.cast(&crank, &h2, ROOT_UNIT, &split);
        let mut d = bytes_of(&w, &h2, f);
        if d.len() > 42 {
            d[42] = PASS;
        }
        put(&mut w, road_at(&ROADS, BOARD, &h2), owner, d);
        w.warp(w.slot() + WINDOW + 1);
        w.run(seal_market(crank, h2), &[ok()]);
        assert_eq!(w.session(&h2, ROOT_UNIT).state, STATE_DEADLOCK, "{what}: a passed road there says nothing");
    }
    // an unknown verdict says nothing too
    let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], CLASS_MARKET, 30);
    w.cast(&crank, &h, ROOT_UNIT, &split);
    lay_road(&mut w, &h, 3, 100);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal_market(crank, h), &[ok()]);
    assert_eq!(w.session(&h, ROOT_UNIT).state, STATE_DEADLOCK);
}

#[test]
fn a_crews_own_session_of_a_market_proposal_seals_by_the_and() {
    let Estate { mut w, crank, founders, child_founders, alice, bob, carol, .. } = estate(true);
    w.run(endow(crank, ROOT_UNIT, 5 * SOL), &[ok()]);
    // the root's market law, decided over the tree under the AND
    let law = market_law(&ROADS, BOARD, MARKET_CLASSES, PLAN_CLASSES, 0);
    let h = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &sha(&[&law]), 1);
    w.run(propose_raw(alice.key, h, CLASS_CODEX, ROOT_UNIT, MANIFEST, &law, &[ROOT_UNIT, 1]), &[ok()]);
    crew_speaks(&mut w, crank, h, &child_founders);
    w.cast(&crank, &h, ROOT_UNIT, &and_votes(&founders, &alice, &bob));
    w.run(seal(crank, h, ROOT_UNIT), &[ok()]);
    lay_our_board(&mut w, &law);
    w.run(execute_law(crank, h, &[ROOT_UNIT, 1], &law), &[ok()]);
    assert_eq!(market_of(&w).map(|m| m.3), Some(MARKET_CLASSES));
    w.warp(w.slot() + 1);

    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let hs = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT, 1], CLASS_MARKET, 2);
    lay_road(&mut w, &hs, PASS, 100);
    w.cast(&crank, &hs, 1, &pro(&child_founders, 8));
    // a crew's own session: five accounts, the AND
    let mut crew_with_market = seal(crank, hs, 1);
    crew_with_market.accounts.extend(seal_market(crank, hs).accounts[5..].iter().cloned());
    w.run(crew_with_market, &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    w.run(seal(crank, hs, 1), &[ok()]);
    assert_eq!(w.session(&hs, 1).state, STATE_PASSED);
    w.run(mandate(crank, hs, ROOT_UNIT, 1), &[ok()]);
    // the root: six founders and the crew's chair, one seat of the floor: split, and the road passed
    let mut root = pro(&founders, 6);
    root.push((&carol, VOTE_PRO));
    w.cast(&crank, &hs, ROOT_UNIT, &root);
    w.warp(w.slot() + WINDOW + 1);
    w.run(seal(crank, hs, ROOT_UNIT), &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    w.run(seal_market(crank, hs), &[ok()]);
    let s = w.session(&hs, ROOT_UNIT);
    assert_eq!((s.pent_pro, s.hex_pro, s.state), (7, 1, STATE_PASSED), "the origin's split goes to the market");
    let l0 = w.lamports(&lab);
    w.run(execute(crank, hs, &[ROOT_UNIT, 1], &budget), &[ok()]);
    assert_eq!(w.lamports(&lab) - l0, SOL / 2);
}

#[test]
fn a_roads_window_lies_inside_the_vote() {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    open_market(&mut w, &crank, &alice.key, &and, 0, 1);
    let lab = w.add_wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let split = chairs_not_floor(&founders, &carol);
    let chairs = pro(&founders, 7);
    // (opens, closes) against the vote's (opened, deadline): one slot outside says nothing, the edge itself speaks
    let windows: [(&str, i64, i64, bool); 4] = [
        ("opens one slot before the proposal", -1, 0, false),
        ("opens at the proposal's own slot", 0, 0, true),
        ("closes one slot after the deadline", 0, 1, false),
        ("closes at the deadline", 0, 0, true),
    ];
    let mut nonce = 10u64;
    for (what, before, after, speaks) in windows {
        for (bits, votes, verdict) in [(CLASS_MARKET, &split, PASS), (CLASS_MARKET, &split, OPEN), (CLASS_PLAN, &chairs, PASS), (CLASS_PLAN, &chairs, OPEN)] {
            nonce += 1;
            let h = open_asking(&mut w, &alice.key, &budget, &[ROOT_UNIT], bits, nonce);
            w.cast(&crank, &h, ROOT_UNIT, votes);
            let (opened, deadline) = vote_window(&w, &h);
            lay_road_at(&mut w, &h, verdict, 100, (opened as i64 + before) as u64, (deadline as i64 + after) as u64, CREDITED);
            w.warp(w.slot() + WINDOW + 1);
            let want = match (speaks, verdict) {
                (true, OPEN) => {
                    w.run(seal_market(crank, h), &[refuse(PoleisError::MarketOpen)]);
                    continue;
                }
                (true, _) => STATE_PASSED,
                // says nothing, and the seal is not held: a split stands, a plan cannot pass
                (false, _) => STATE_DEADLOCK,
            };
            w.run(seal_market(crank, h), &[ok()]);
            assert_eq!(w.session(&h, ROOT_UNIT).state, want, "{what}, {} {verdict}", if bits == CLASS_PLAN { "a plan" } else { "a split" });
        }
    }
}

#[test]
fn a_polis_names_only_a_board_that_is_its_own() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    let law = market_law(&ROADS, BOARD, MARKET_TREASURY, PLAN_CLASSES, 0);
    let at = board_at(&ROADS, BOARD);
    let agent = agent_pda(ROOT_UNIT);
    let ours = board_bytes(BOARD, &agent, &PROGRAM_ID, ROOT_UNIT);
    let mut other_version = ours.clone();
    other_version[1] = 2;
    let mut other_kind = ours.clone();
    other_kind[0] = 3;
    let cases: [(&str, Vec<u8>, Pubkey); 9] = [
        ("its authority a key", board_bytes(BOARD, &alice.key, &PROGRAM_ID, ROOT_UNIT), ROADS),
        ("its law naming another unit", board_bytes(BOARD, &agent, &PROGRAM_ID, 1), ROADS),
        ("its law naming another program", board_bytes(BOARD, &agent, &ROADS, ROOT_UNIT), ROADS),
        ("its law naming no polis (it lays nothing by rule)", board_bytes(BOARD, &agent, &Pubkey::default(), ROOT_UNIT), ROADS),
        ("of another number", board_bytes(BOARD + 1, &agent, &PROGRAM_ID, ROOT_UNIT), ROADS),
        ("of another owner", ours.clone(), Pubkey::new_from_array([0x53; 32])),
        ("of another version", other_version, ROADS),
        ("of another kind", other_kind, ROADS),
        ("too short for the law", ours[..153].to_vec(), ROADS),
    ];
    for (i, (what, bytes, owner)) in cases.into_iter().enumerate() {
        put(&mut w, at, owner, bytes);
        let h = decide_law(&mut w, &crank, &alice.key, &and, &law, 10 + i as u64);
        w.run(execute_law(crank, h, &[ROOT_UNIT], &law), &[refuse(PoleisError::MarketBoardNotOurs)]);
        assert!(market_of(&w).is_none(), "{what}: no market is named");
    }
    // a law that turns on one mode alone names a board all the same: a board held by a key is refused under the market alone, and under
    // plans alone
    put(&mut w, at, ROADS, board_bytes(BOARD, &alice.key, &PROGRAM_ID, ROOT_UNIT));
    for (i, one) in [market_law(&ROADS, BOARD, MARKET_TREASURY, 0, 0), market_law(&ROADS, BOARD, 0, PLAN_CLASSES, 0)].iter().enumerate() {
        let h = decide_law(&mut w, &crank, &alice.key, &and, one, 20 + i as u64);
        w.run(execute_law(crank, h, &[ROOT_UNIT], one), &[refuse(PoleisError::MarketBoardNotOurs)]);
        assert!(market_of(&w).is_none(), "one mode alone: no market is named");
    }
    // no board where one stands: the act refuses
    w.accounts.retain(|(k, _)| *k != at);
    let h = decide_law(&mut w, &crank, &alice.key, &and, &law, 30);
    w.run(execute_law(crank, h, &[ROOT_UNIT], &law), &[refuse(PoleisError::MarketBoardNotOurs)]);
    // the unit's own board at another address, handed by a crank in its place
    let elsewhere = Pubkey::new_unique();
    put(&mut w, elsewhere, ROADS, ours.clone());
    let mut ix = execute_law(crank, h, &[ROOT_UNIT], &law);
    ix.accounts.last_mut().unwrap().pubkey = elsewhere;
    w.run(ix, &[refuse(PoleisError::MarketBoardNotOurs)]);
    // a law that turns the market on with no board account at all
    let mut short = execute_law(crank, h, &[ROOT_UNIT], &law);
    short.accounts.pop();
    w.run(short, &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    assert!(market_of(&w).is_none());

    // its own: taken
    put(&mut w, at, ROADS, ours);
    if std::env::var_os("POLEIS_CU").is_some() {
        eprintln!("the CODEX_MARKET act with its board follows");
    }
    w.run(execute_law(crank, h, &[ROOT_UNIT], &law), &[ok()]);
    assert_eq!(market_of(&w).map(|m| (m.1, m.2, m.3, m.4)), Some((ROADS, BOARD, MARKET_TREASURY, PLAN_CLASSES)));
    // an act that turns the market off names no board, whatever stands there
    w.warp(w.slot() + 1);
    w.accounts.retain(|(k, _)| *k != at);
    law_root(&mut w, &crank, &alice.key, &and, &market_law(&ROADS, BOARD, 0, 0, 0), 31, ok());
    assert_eq!(market_of(&w).map(|m| (m.3, m.4)), Some((0, 0)));
}

#[test]
fn the_market_never_decides_its_own_law() {
    let Estate { mut w, crank, founders, alice, bob, .. } = estate(false);
    let and = and_votes(&founders, &alice, &bob);
    open_market(&mut w, &crank, &alice.key, &and, 0, 1);
    let agent = agent_pda(ROOT_UNIT);
    let board = board_at(&ROADS, BOARD);
    let relay = Pubkey::new_from_array([0x60; 32]);
    let stranger = Pubkey::new_unique();
    let act = |program: Pubkey, accounts: Vec<(Pubkey, u8)>| Payload::Raw { origin: ROOT_UNIT, program, accounts, data: vec![11, 0, 0] };
    let refused = [
        ("an act into the market's own program, on its board", act(ROADS, vec![(agent, ACT_SIGNER), (board, ACT_WRITABLE)])),
        ("an act into the market's own program, on no board", act(ROADS, vec![(stranger, ACT_WRITABLE)])),
        ("an act through a relay program that lists the origin's board", act(relay, vec![(stranger, ACT_WRITABLE), (board, 0)])),
        ("an act through a relay program that lists the origin's board first", act(relay, vec![(board, 0), (stranger, ACT_WRITABLE)])),
    ];
    let mut nonce = 10;
    for (what, p) in &refused {
        for bits in [CLASS_MARKET, CLASS_PLAN] {
            nonce += 1;
            w.run(ask(&alice.key, p, &[ROOT_UNIT], bits, nonce), &[refuse(PoleisError::MarketNotAllowed)]);
            assert!(w.data(&proposal_pda(&hash_of(p, nonce))).is_empty(), "{what}: refused, nothing left behind");
        }
        // the same act without the bit: under the AND
        nonce += 1;
        let h = w.open(&alice.key, p, &[ROOT_UNIT], nonce);
        assert!(!w.data(&proposal_pda(&h)).is_empty(), "{what}: taken under the AND");
    }
    // an act that cannot be parsed as far as its accounts is refused the same way
    let short = vec![CLASS_ACT, ACT_RAW, 1, 2, 3];
    nonce += 1;
    let hs = proposal_hash(CLASS_ACT, ROOT_UNIT, &MANIFEST, &sha(&[&short]), nonce);
    let mut ix = propose_raw(alice.key, hs, CLASS_ACT | CLASS_MARKET, ROOT_UNIT, MANIFEST, &short, &[ROOT_UNIT]);
    ix.accounts.push(AccountMeta::new_readonly(market_pda(ROOT_UNIT), false));
    w.run(ix, &[refuse(PoleisError::MarketNotAllowed)]);
    // an act that claims more accounts than it lists is refused the same way, and never read past its end
    let mut over = vec![CLASS_ACT, ACT_RAW];
    over.extend_from_slice(relay.as_ref());
    over.push(3);
    over.extend_from_slice(stranger.as_ref());
    over.push(ACT_WRITABLE);
    nonce += 1;
    let ho = proposal_hash(CLASS_ACT, ROOT_UNIT, &MANIFEST, &sha(&[&over]), nonce);
    let mut ix = propose_raw(alice.key, ho, CLASS_ACT | CLASS_MARKET, ROOT_UNIT, MANIFEST, &over, &[ROOT_UNIT]);
    ix.accounts.push(AccountMeta::new_readonly(market_pda(ROOT_UNIT), false));
    w.run(ix, &[refuse(PoleisError::MarketNotAllowed)]);
    // an asking act that touches neither the market's program nor the origin's board is granted, in either mode
    let free = act(relay, vec![(stranger, ACT_WRITABLE), (agent, ACT_SIGNER)]);
    for bits in [CLASS_MARKET, CLASS_PLAN] {
        nonce += 1;
        w.run(ask(&alice.key, &free, &[ROOT_UNIT], bits, nonce), &[ok()]);
    }
}

// ---- THE TWO PROGRAMS IN ONE WORLD ----
// Poleis and the roads program (`../poleis_roads`, the market beside Poleis), both built from this tree and loaded in one Mollusk world,
// and the whole chain run once with no hand-written road or board anywhere. The roads program's accounts are read here by its own
// offsets (`poleis_roads/src/state.rs`, the numbers beside each read), never by a dependency on its crate. It stands in this file, on
// this file's own council and its voters, so that it adds no signing code of its own (THE HOUSE SIGNS POST-QUANTUM ONLY counts every
// line that names a classical signature, and a file of its own would have added one).
//
// ONE CHAIN OF STEPS, TWO WORLDS (Nov 21 2026, THE MARKET ON A LOCAL CHAIN): the chain is one body, `two_programs`, over what it asks of a
// world (`Ledger`), after `estate`'s own founding (`founded`). Mollusk's `World` runs it as it ran; `Live` runs the same steps on a real
// validator on this machine, reached by its JSON-RPC over loopback and nothing else (`the_two_programs_on_a_live_validator`, run by
// `poleis/ceremony/the-market-on-a-local-chain.sh`). `Live` stands here for the same reason: it signs with the key type this file names.

/// The roads program counts $WARM in micros.
const MICRO: u64 = 1_000_000;
/// The board the polis opens on the real roads program.
const REAL_BOARD: u64 = 77;

/// The real roads program in a council's world: its id, and its instructions by its own wire and its own seeds.
struct Real {
    id: Pubkey,
}
impl Real {
    /// Its id, read from its own source (its `declare_id!`): the bytes the program is built with.
    fn declared() -> Pubkey {
        let src = include_str!("../../poleis_roads/src/lib.rs");
        let at = src.find("declare_id!(\"").expect("the roads program declares its id") + "declare_id!(\"".len();
        Pubkey::from_str(&src[at..at + src[at..].find('"').unwrap()]).expect("a base58 id")
    }
    /// Built from this tree and loaded by its bytes, never through `SBF_OUT_DIR` (the other tests of this binary read that variable as
    /// they start). Its program account goes into the world, executable, so Poleis's hands can call it.
    fn load(w: &mut World) -> Self {
        let id = Self::declared();
        let elf = std::fs::read(format!("{}/poleis_roads.so", built::roads_program_dir())).expect("the built roads program");
        w.mollusk.add_program_with_loader_and_elf(&id, &mollusk_svm::program::loader_keys::LOADER_V3, &elf);
        w.accounts.push((id, mollusk_svm::program::create_program_account_loader_v3(&id)));
        Self { id }
    }
    fn ix(&self, tag: u8, data: &[u8], metas: Vec<AccountMeta>) -> Instruction {
        let mut d = vec![tag];
        d.extend_from_slice(data);
        Instruction::new_with_bytes(self.id, &d, metas)
    }
    fn board(&self) -> Pubkey {
        Pubkey::find_program_address(&[b"board", &REAL_BOARD.to_le_bytes()], &self.id).0
    }
    fn balance(&self, owner: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"bal", &REAL_BOARD.to_le_bytes(), owner.as_ref()], &self.id).0
    }
    fn road(&self, hash: &[u8; 32]) -> Pubkey {
        Pubkey::find_program_address(&[b"road", &REAL_BOARD.to_le_bytes(), hash], &self.id).0
    }
    fn position(&self, road: &Pubkey, owner: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"pos", road.as_ref(), owner.as_ref()], &self.id).0
    }
    /// `init_board`: [payer (s,w), board (w), system], data [board_id:8][authority:32]
    fn init_board(&self, payer: Pubkey, authority: &Pubkey) -> Instruction {
        let mut d = REAL_BOARD.to_le_bytes().to_vec();
        d.extend_from_slice(authority.as_ref());
        self.ix(0, &d, vec![AccountMeta::new(payer, true), AccountMeta::new(self.board(), false), AccountMeta::new_readonly(Pubkey::default(), false)])
    }
    fn credit(&self, crediter: Pubkey, owner: Pubkey, amount: u64) -> Instruction {
        let metas = vec![
            AccountMeta::new(crediter, true),
            AccountMeta::new(self.board(), false),
            AccountMeta::new_readonly(owner, false),
            AccountMeta::new(self.balance(&owner), false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ];
        self.ix(2, &amount.to_le_bytes(), metas)
    }
    fn deposit(&self, owner: Pubkey, hash: &[u8; 32], amount: u64) -> Instruction {
        let road = self.road(hash);
        let metas = vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(road, false),
            AccountMeta::new(self.balance(&owner), false),
            AccountMeta::new(self.position(&road, &owner), false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ];
        self.ix(4, &amount.to_le_bytes(), metas)
    }
    /// branch 0 pass, 1 fail; side 0 buy, 1 sell
    fn trade(&self, owner: Pubkey, hash: &[u8; 32], branch: u8, side: u8, amount: u64) -> Instruction {
        let road = self.road(hash);
        let mut d = vec![branch, side];
        d.extend_from_slice(&amount.to_le_bytes());
        d.extend_from_slice(&0u64.to_le_bytes());
        self.ix(5, &d, vec![AccountMeta::new_readonly(owner, true), AccountMeta::new(road, false), AccountMeta::new(self.position(&road, &owner), false)])
    }
    fn carry(&self, hash: &[u8; 32]) -> Instruction {
        self.ix(6, &[], vec![AccountMeta::new(self.road(hash), false)])
    }
    fn decide(&self, hash: &[u8; 32]) -> Instruction {
        self.ix(7, &[], vec![AccountMeta::new(self.road(hash), false)])
    }
    fn receipts(&self, judge: Pubkey, hash: &[u8; 32], world: u8, worth: u64) -> Instruction {
        let mut d = vec![world];
        d.extend_from_slice(&worth.to_le_bytes());
        self.ix(8, &d, vec![AccountMeta::new_readonly(judge, true), AccountMeta::new(self.road(hash), false)])
    }
    fn settle(&self, hash: &[u8; 32], owner: Pubkey) -> Instruction {
        let road = self.road(hash);
        let metas = vec![
            AccountMeta::new(road, false),
            AccountMeta::new(self.position(&road, &owner), false),
            AccountMeta::new(self.balance(&owner), false),
            AccountMeta::new(owner, false),
        ];
        self.ix(9, &[], metas)
    }
    /// `collect`: [road (w), the staker's balance (w), the road's board (w)]
    fn collect(&self, hash: &[u8; 32], staker: Pubkey) -> Instruction {
        self.ix(10, &[], vec![AccountMeta::new(self.road(hash), false), AccountMeta::new(self.balance(&staker), false), AccountMeta::new(self.board(), false)])
    }
    /// `lay_for`: [layer (s,w), board, road (w), the layer's own balance (w), proposal, session, system]
    fn lay_for(&self, layer: Pubkey, hash: &[u8; 32]) -> Instruction {
        let metas = vec![
            AccountMeta::new(layer, true),
            AccountMeta::new_readonly(self.board(), false),
            AccountMeta::new(self.road(hash), false),
            AccountMeta::new(self.balance(&layer), false),
            AccountMeta::new_readonly(proposal_pda(hash), false),
            AccountMeta::new_readonly(session_pda(hash, ROOT_UNIT), false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ];
        self.ix(12, &[], metas)
    }
}

// ---- THE THIRD PROGRAM: $WARM IS MINED BY USE ----
// The warm program (`../warm_mine`: $WARM's mint, the program's own, and its mining by the Space Router's law; his Nov 21 2026 17:34:37
// UTC "it is supposed to be uncapped supply, prompt-2-mine; the issuance needs to be live and continuous based on the space-router
// utilization") in the same world, after the market's chain. Like the roads program it is built from this tree, loaded by its bytes and
// read by its own offsets (`warm_mine/src/state.rs`, the numbers beside each read), never through a dependency on its crate; its LAW
// alone is its own text, `include!`d whole (`warm_mine/src/law.rs` is pure), so what each spend mines is computed here by the very lines
// the program runs. Its upgrade authority is the polis's own hand, as the ceremony leaves a program (its program data laid so here; on a
// validator, the script's `--upgradeable-program`), so only a sealed act of the polis opens it.

/// The warm program's law: the text it runs.
mod warm_law {
    include!("../../warm_mine/src/law.rs");
}

/// A dollar of USDC, in micros; one $WARM, in nanos; an hour and a day, in ms.
const USD: u64 = 1_000_000;
const WARM: u64 = 1_000_000_000;
const HOUR_MS: u64 = 3_600_000;
const DAY_MS: u64 = 86_400_000;
/// His word's instant, Nov 21 2026 17:34:37 UTC, in seconds: the clock of Mollusk's world for the mine (Mollusk's own has no time of day).
const HIS_WORD_S: i64 = 1_795_282_477;

/// The associated token program, `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`.
fn ata_program() -> Pubkey {
    Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap()
}
/// `owner`'s associated token account of `mint`.
fn associated(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()], &ata_program()).0
}

/// The real warm program in a council's world: its id, its accounts by its own seeds, and its instructions by its own wire.
struct Warm {
    id: Pubkey,
}
impl Warm {
    /// Its id, read from its own source (its `declare_id!`): the bytes the program is built with.
    fn declared() -> Pubkey {
        let src = include_str!("../../warm_mine/src/lib.rs");
        let at = src.find("declare_id!(\"").expect("the warm program declares its id") + "declare_id!(\"".len();
        Pubkey::from_str(&src[at..at + src[at..].find('"').unwrap()]).expect("a base58 id")
    }
    /// Built from this tree and loaded by its bytes, as the roads program is, its program account executable in the world; its program
    /// data laid as the ceremony leaves it, the polis's own hand its upgrade authority; the associated token program beside it; and the
    /// world's clock set to his word's instant if it has none.
    fn load(w: &mut World) -> Self {
        let warm = Self { id: Self::declared() };
        let elf = std::fs::read(format!("{}/warm_mine.so", built::warm_program_dir())).expect("the built warm program");
        w.mollusk.add_program_with_loader_and_elf(&warm.id, &mollusk_svm::program::loader_keys::LOADER_V3, &elf);
        w.accounts.push((warm.id, mollusk_svm::program::create_program_account_loader_v3(&warm.id)));
        // the loader's ProgramData: [3:4][slot:8][an upgrade authority: 1 and its 32 bytes]
        let mut pd = vec![3u8, 0, 0, 0];
        pd.extend_from_slice(&0u64.to_le_bytes());
        pd.push(1);
        pd.extend_from_slice(agent_pda(ROOT_UNIT).as_ref());
        let lamports = w.rent(pd.len());
        w.accounts.push((warm.program_data(), Account { lamports, data: pd, owner: BPF_LOADER_UPGRADEABLE_ID, executable: false, rent_epoch: 0 }));
        mollusk_svm_programs_token::associated_token::add_program(&mut w.mollusk);
        w.accounts.push(mollusk_svm_programs_token::associated_token::keyed_account());
        if w.mollusk.sysvars.clock.unix_timestamp == 0 {
            w.mollusk.sysvars.clock.unix_timestamp = HIS_WORD_S;
        }
        warm
    }
    fn pda(&self, seeds: &[&[u8]]) -> Pubkey {
        Pubkey::find_program_address(seeds, &self.id).0
    }
    fn program_data(&self) -> Pubkey {
        Pubkey::find_program_address(&[self.id.as_ref()], &BPF_LOADER_UPGRADEABLE_ID).0
    }
    fn mine(&self) -> Pubkey {
        self.pda(&[b"mine"])
    }
    fn mint(&self) -> Pubkey {
        self.pda(&[b"warm"])
    }
    fn vault(&self) -> Pubkey {
        self.pda(&[b"vault"])
    }
    fn record(&self, flask: &Pubkey) -> Pubkey {
        self.pda(&[b"flask", flask.as_ref()])
    }
    /// A flask's own $WARM account.
    fn account_of(&self, flask: &Pubkey) -> Pubkey {
        associated(flask, &self.mint())
    }
    /// One instruction of the program, its accounts flagged as a raw act flags them.
    fn direct(&self, data: Vec<u8>, accounts: &[(Pubkey, u8)]) -> Instruction {
        let metas = accounts.iter().map(|(k, f)| AccountMeta { pubkey: *k, is_signer: f & ACT_SIGNER != 0, is_writable: f & ACT_WRITABLE != 0 }).collect();
        Instruction::new_with_bytes(self.id, &data, metas)
    }
    /// The same instruction as a sealed act of the polis: its own hand signs it.
    fn act(&self, data: Vec<u8>, accounts: &[(Pubkey, u8)]) -> Payload {
        Payload::Raw { origin: ROOT_UNIT, program: self.id, accounts: accounts.to_vec(), data }
    }
    /// `init`: [0][authority:32][opening_ms:8] · [upgrade authority (s), payer (s,w), program data, mine (w), mint (w), vault (w), treasury,
    /// dollar, token program, system]
    fn init(&self, upgrader: Pubkey, payer: Pubkey, authority: &Pubkey, opening: u64, treasury: &Pubkey, dollar: &Pubkey) -> (Vec<u8>, Vec<(Pubkey, u8)>) {
        let data = [&[0u8][..], authority.as_ref(), &opening.to_le_bytes()].concat();
        let accounts = vec![
            (upgrader, ACT_SIGNER),
            (payer, ACT_SIGNER | ACT_WRITABLE),
            (self.program_data(), 0),
            (self.mine(), ACT_WRITABLE),
            (self.mint(), ACT_WRITABLE),
            (self.vault(), ACT_WRITABLE),
            (*treasury, 0),
            (*dollar, 0),
            (TOKEN_PROGRAM_ID, 0),
            (Pubkey::default(), 0),
        ];
        (data, accounts)
    }
    /// `set_miner`: [2][miner:32] · [authority (s), mine (w)]
    fn set_miner(&self, authority: Pubkey, miner: &Pubkey) -> (Vec<u8>, Vec<(Pubkey, u8)>) {
        ([&[2u8][..], miner.as_ref()].concat(), vec![(authority, ACT_SIGNER), (self.mine(), ACT_WRITABLE)])
    }
    /// `deposit`: [3][amount:8] · [flask (s), payer (s,w), mine (w), record (w), source (w), treasury (w), dollar, token program, system]
    fn deposit(&self, flask: Pubkey, payer: Pubkey, source: &Pubkey, treasury: &Pubkey, dollar: &Pubkey, amount: u64) -> Instruction {
        let metas = vec![
            AccountMeta::new_readonly(flask, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(self.mine(), false),
            AccountMeta::new(self.record(&flask), false),
            AccountMeta::new(*source, false),
            AccountMeta::new(*treasury, false),
            AccountMeta::new_readonly(*dollar, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ];
        Instruction::new_with_bytes(self.id, &[&[3u8][..], &amount.to_le_bytes()].concat(), metas)
    }
    /// `mine`: [5][n:1] then n × [usd_micros:8][at_ms:8][seq:8] · [miner (s,w), mine (w), mint (w), vault (w), token program, associated
    /// token program, system], then each spend's [flask, record (w), its $WARM account (w)]
    fn spends(&self, miner: Pubkey, spends: &[(Pubkey, u64, u64, u64)]) -> Instruction {
        let mut data = vec![5u8, spends.len() as u8];
        let mut metas = vec![
            AccountMeta::new(miner, true),
            AccountMeta::new(self.mine(), false),
            AccountMeta::new(self.mint(), false),
            AccountMeta::new(self.vault(), false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(ata_program(), false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ];
        for (flask, usd, at, seq) in spends {
            for v in [usd, at, seq] {
                data.extend_from_slice(&v.to_le_bytes());
            }
            metas.extend([AccountMeta::new_readonly(*flask, false), AccountMeta::new(self.record(flask), false), AccountMeta::new(self.account_of(flask), false)]);
        }
        Instruction::new_with_bytes(self.id, &data, metas)
    }
    /// `pay_back`: [6][amount:8] · [flask (s), payer (s,w), mine (w), record (w), source (w), vault (w), mint, token program, system]
    fn pay_back(&self, flask: Pubkey, payer: Pubkey, amount: u64) -> Instruction {
        let metas = vec![
            AccountMeta::new_readonly(flask, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(self.mine(), false),
            AccountMeta::new(self.record(&flask), false),
            AccountMeta::new(self.account_of(&flask), false),
            AccountMeta::new(self.vault(), false),
            AccountMeta::new_readonly(self.mint(), false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ];
        Instruction::new_with_bytes(self.id, &[&[6u8][..], &amount.to_le_bytes()].concat(), metas)
    }
}

/// What the chain of THE TWO PROGRAMS IN ONE WORLD asks of a world, Mollusk's or a validator's. Every slot the body names is counted from a
/// slot the world gave it, so on Mollusk every number stays what it was.
trait Ledger {
    /// One instruction in a transaction of its own, `what` its name in the record: its compute units, or the program's refusal. Only a
    /// success changes the world.
    fn attempt(&mut self, ix: &Instruction, what: &str) -> Result<u64, ProgramError>;
    /// Several instructions in one transaction, `payer` paying for it: its compute units, or the refusal of the instruction that refused.
    fn attempt_all(&mut self, ixs: &[Instruction], payer: &Pubkey, what: &str) -> Result<u64, ProgramError>;
    fn data(&self, key: &Pubkey) -> Vec<u8>;
    fn lamports(&self, key: &Pubkey) -> u64;
    fn slot(&self) -> u64;
    /// The clock at `slot` or past it: Mollusk warps there, a validator is waited for.
    fn wait_until(&mut self, slot: u64);
    /// A new key holding `lamports`.
    fn wallet(&mut self, lamports: u64) -> Pubkey;
    /// A new voter, a signing key of its own, holding `lamports`.
    fn voter(&mut self, lamports: u64) -> Voter;
    /// An initialized SPL mint with no authority.
    fn mint(&mut self) -> Pubkey;
    /// The roads program of this tree, standing in this world.
    fn roads(&mut self) -> Real;
    /// The warm program of this tree, standing in this world, the polis's own hand its upgrade authority.
    fn warm(&mut self) -> Warm;
    /// A dollar of this world: an SPL mint of six decimals, as USDC is, its minting authority the world's own (the estate's default mint
    /// has no decimals, and on a validator no authority and no supply: no flask could hold a cent of it).
    fn dollar(&mut self) -> Pubkey;
    /// `amount` of `mint` given to `owner` in its associated account, made if missing: the account.
    fn pay(&mut self, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Pubkey;
    /// The chain's clock, in milliseconds: its Clock's `unix_timestamp` times 1,000.
    fn clock_ms(&self) -> u64;

    fn done(&mut self, ix: Instruction, what: &str) -> u64 {
        self.attempt(&ix, what).unwrap_or_else(|e| panic!("{what}: refused with {e:?}"))
    }
    fn refused(&mut self, ix: Instruction, e: ProgramError, what: &str) {
        assert_eq!(self.attempt(&ix, what), Err(e), "{what}");
    }
    fn u64_at(&self, key: &Pubkey, o: usize) -> u64 {
        u64::from_le_bytes(self.data(key)[o..o + 8].try_into().unwrap())
    }
    fn session(&self, hash: &[u8; 32], unit: u64) -> SessionView {
        SessionView::parse(&self.data(&session_pda(hash, unit))).expect("session parses")
    }
    /// Votes landed in batches of four, each batch the precompile and `attest` in one transaction the crank pays for.
    fn cast(&mut self, crank: &Pubkey, hash: &[u8; 32], unit: u64, votes: &[(&Voter, u8)]) {
        for chunk in votes.chunks(4) {
            let signed: Vec<(Pubkey, u8, [u8; 64])> = chunk.iter().map(|(v, c)| v.vote(hash, unit, *c)).collect();
            let ixs = attest_batch(*crank, *hash, unit, &signed);
            self.attempt_all(&ixs, crank, "a batch of votes").unwrap_or_else(|e| panic!("a batch of votes: refused with {e:?}"));
        }
    }
    /// `ix`, sent until the road is not behind (the roads program's `RoadBehind`, 14), a `carry` between.
    fn behind_then(&mut self, real: &Real, hash: &[u8; 32], ix: Instruction, what: &str) {
        loop {
            match self.attempt(&ix, what) {
                Ok(_) => return,
                Err(ProgramError::Custom(14)) => {
                    self.done(real.carry(hash), "a carry");
                }
                Err(e) => panic!("{what}: refused with {e:?}"),
            }
        }
    }
}

impl Ledger for World {
    fn attempt(&mut self, ix: &Instruction, what: &str) -> Result<u64, ProgramError> {
        for meta in &ix.accounts {
            self.touch(meta.pubkey);
        }
        let r = self.mollusk.process_instruction(ix, &self.accounts);
        match r.program_result {
            mollusk_svm::result::ProgramResult::Success => {
                let resulting = r.resulting_accounts.clone();
                self.merge(&resulting);
                self.landed.push((what.to_string(), r.compute_units_consumed));
                Ok(r.compute_units_consumed)
            }
            mollusk_svm::result::ProgramResult::Failure(e) => Err(e),
            mollusk_svm::result::ProgramResult::UnknownError(e) => panic!("the runtime failed: {e:?}"),
        }
    }
    fn attempt_all(&mut self, ixs: &[Instruction], payer: &Pubkey, what: &str) -> Result<u64, ProgramError> {
        for ix in ixs {
            for meta in &ix.accounts {
                self.touch(meta.pubkey);
            }
        }
        let r = self.mollusk.process_transaction_instructions(ixs, &self.accounts, Some(payer));
        if r.raw_result.is_ok() {
            let resulting = r.resulting_accounts.clone();
            self.merge(&resulting);
            self.landed.push((what.to_string(), r.compute_units_consumed));
            return Ok(r.compute_units_consumed);
        }
        match r.program_result {
            mollusk_svm::result::types::TransactionProgramResult::Failure(_, e) => Err(e),
            _ => panic!("{what}: the runtime failed: {:?}", r.raw_result),
        }
    }
    fn data(&self, key: &Pubkey) -> Vec<u8> {
        World::data(self, key)
    }
    fn lamports(&self, key: &Pubkey) -> u64 {
        World::lamports(self, key)
    }
    fn slot(&self) -> u64 {
        World::slot(self)
    }
    fn wait_until(&mut self, slot: u64) {
        if slot > World::slot(self) {
            // a warp lays a new Clock with no time of day: the world's instant, if it has one, stands across it
            let instant = self.mollusk.sysvars.clock.unix_timestamp;
            self.warp(slot);
            self.mollusk.sysvars.clock.unix_timestamp = instant;
        }
    }
    fn wallet(&mut self, lamports: u64) -> Pubkey {
        self.add_wallet(lamports)
    }
    fn voter(&mut self, lamports: u64) -> Voter {
        World::voter(self, lamports)
    }
    fn mint(&mut self) -> Pubkey {
        World::mint(self)
    }
    fn roads(&mut self) -> Real {
        Real::load(self)
    }
    fn warm(&mut self) -> Warm {
        Warm::load(self)
    }
    /// A mint laid by hand, as `mint` lays the default one: its minting authority (an option's tag 1 at 0, the key at 4) a key of this
    /// world, six decimals at 44, initialized at 45, no freeze authority.
    fn dollar(&mut self) -> Pubkey {
        let key = Pubkey::new_unique();
        let mut data = vec![0u8; 82];
        data[0] = 1;
        data[4..36].copy_from_slice(Pubkey::new_unique().as_ref());
        data[44] = 6;
        data[45] = 1;
        let lamports = self.rent(82);
        self.accounts.push((key, Account { lamports, data, owner: TOKEN_PROGRAM_ID, executable: false, rent_epoch: 0 }));
        key
    }
    /// The owner's associated account laid by hand holding `amount`, and the mint's supply grown by it: what a mint-to does.
    fn pay(&mut self, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Pubkey {
        let key = associated(owner, mint);
        let mut data = vec![0u8; TOKEN_ACCOUNT_LEN];
        data[..32].copy_from_slice(mint.as_ref());
        data[32..64].copy_from_slice(owner.as_ref());
        data[64..72].copy_from_slice(&amount.to_le_bytes());
        data[108] = 1;
        let lamports = self.rent(TOKEN_ACCOUNT_LEN);
        let held = self.data(&key).get(64..72).map(|b| u64::from_le_bytes(b.try_into().unwrap())).unwrap_or(0);
        data[64..72].copy_from_slice(&(held + amount).to_le_bytes());
        self.merge(&[(key, Account { lamports, data, owner: TOKEN_PROGRAM_ID, executable: false, rent_epoch: 0 })]);
        let supply = self.tokens_supply(mint) + amount;
        let slot = self.accounts.iter_mut().find(|(k, _)| k == mint).expect("the mint");
        slot.1.data[36..44].copy_from_slice(&supply.to_le_bytes());
        key
    }
    fn clock_ms(&self) -> u64 {
        u64::try_from(self.mollusk.sysvars.clock.unix_timestamp).unwrap_or(0) * 1_000
    }
}

impl World {
    /// A mint's supply (at 36).
    fn tokens_supply(&self, mint: &Pubkey) -> u64 {
        u64::from_le_bytes(self.data(mint)[36..44].try_into().unwrap())
    }
}

/// What THE TWO PROGRAMS IN ONE WORLD ends with: each road's verdict, what each budget paid, each stake back, what the house earned, and the
/// book (the balances and what the house earned, against what the crediter credited); then the third program's, the mine's.
struct Book {
    verdicts: Vec<u8>,
    paid: Vec<u64>,
    backs: Vec<u64>,
    earned: u64,
    sum: u64,
    credited: u64,
    mine: MineBook,
    budget: BudgetBook,
}

/// What the third program ends with, read off its own accounts: the opening and its first halving, the spends mined, the dollars
/// deposited and spent, the $WARM issued (minted, and paid out of the vault), paid back and standing in the vault, the mint's supply, the
/// law's own total over every spend that landed, and what the treasury holds.
struct MineBook {
    opening: u64,
    halving: u64,
    spends: u64,
    deposited: u64,
    spent: u64,
    issued: u64,
    minted: u64,
    from_vault: u64,
    paid_back: u64,
    vault: u64,
    supply: u64,
    law: u64,
    treasury: u64,
    /// What a later scenario of the same world needs to use the mine: the world's dollar, the miner the polis named, and the last spend
    /// that landed (its instant and its seq), so a spend after it rises.
    dollar: Pubkey,
    miner: Pubkey,
    last_ms: u64,
    last_seq: u64,
}

/// THE THIRD PROGRAM in any world, after the market's chain: the polis opens the mine by a sealed ACT_RAW (its own hand is the program's
/// upgrade authority, and it names itself the mine's authority), names the miner by another, as it sets its board; flasks deposit the
/// world's dollars into the polis's own vault; the miner mines a day of spends across a halving; the bound refuses one spend too many; a
/// flask pays $WARM back and the next spends are paid from the vault before any mint. All it asserts holds in both worlds.
fn the_mine<L: Ledger>(w: &mut L, crank: Pubkey, proposer: &Voter, and: &[(&Voter, u8)]) -> MineBook {
    let warm = w.warm();
    let agent = agent_pda(ROOT_UNIT);
    let mut nonce = 30u64;
    let mut sealed = |w: &mut L, act: &Payload, what: &str| -> u64 {
        nonce += 1;
        let h = proposal_hash(act.class(), ROOT_UNIT, &MANIFEST, &act.hash(), nonce);
        w.done(propose_raw(proposer.key, h, act.class(), ROOT_UNIT, MANIFEST, &act.bytes(), &[ROOT_UNIT]), "propose the act");
        w.cast(&crank, &h, ROOT_UNIT, and);
        w.done(seal(crank, h, ROOT_UNIT), "seal the act");
        w.done(execute(crank, h, &[ROOT_UNIT], act), what)
    };

    // 1. the world's dollar, and the treasury deposits go to: the polis's own vault of it
    let dollar = w.dollar();
    w.done(create_vault(crank, ROOT_UNIT, dollar, TOKEN_PROGRAM_ID), "the polis's vault of the dollar");
    let treasury = vault_pda(ROOT_UNIT, &dollar);
    // 2. the opening: two hundred days before the chain's clock, so its first halving stands some seventeen days behind it
    let now = w.clock_ms();
    let opening = now - 200 * DAY_MS;
    let halving = warm_law::halving_at(opening, 1).expect("the first halving");
    assert!(halving + DAY_MS < now, "the first halving stands behind the clock");
    // nobody but the program's upgrade authority opens it
    let stranger = w.wallet(SOL);
    let (data, accounts) = warm.init(stranger, stranger, &agent, opening, &treasury, &dollar);
    w.refused(warm.direct(data, &accounts), ProgramError::Custom(0), "a stranger opens no mine (Unauthorized)");
    // 3. the polis opens it by a sealed ACT_RAW: its own hand is the upgrade authority and names itself the mine's authority; the crank, a
    // co-signer the act names, pays the three rents
    let (data, accounts) = warm.init(agent, crank, &agent, opening, &treasury, &dollar);
    let cu = sealed(w, &warm.act(data, &accounts), "the polis opens the mine");
    eprintln!("  the act init by the polis's hand: {cu} CU");
    // the mine: `Mine::OFF_AUTHORITY` 2, `OFF_TREASURY` 34, `OFF_DOLLAR` 66, `OFF_MINER` 98, `OFF_MINT` 130, `OFF_VAULT` 162, `OFF_OPENING` 194
    let m = w.data(&warm.mine());
    assert_eq!((m.len(), &m[2..34], &m[34..66], &m[66..98], &m[98..130]), (285, agent.as_ref(), treasury.as_ref(), dollar.as_ref(), &[0u8; 32][..]), "the polis its authority, its treasury and dollar, no miner yet");
    assert_eq!((&m[130..162], &m[162..194], w.u64_at(&warm.mine(), 194)), (warm.mint().as_ref(), warm.vault().as_ref(), opening), "its mint, its vault, its opening");
    // the mint: its minting authority the mine (an option's tag at 0, the key at 4), no supply (36), nine decimals (44), initialized (45),
    // no freeze authority (46)
    let mint = w.data(&warm.mint());
    assert_eq!((mint.len(), &mint[..4], &mint[4..36], w.u64_at(&warm.mint(), 36), mint[44], mint[45], &mint[46..50]), (82, &[1u8, 0, 0, 0][..], warm.mine().as_ref(), 0, 9, 1, &[0u8; 4][..]), "the mint is the program's, and nothing is minted at its opening");
    // 4. a stranger names no miner; the polis names the router's crank by a sealed ACT_RAW
    let miner = w.wallet(10 * SOL);
    let (data, accounts) = warm.set_miner(stranger, &stranger);
    w.refused(warm.direct(data, &accounts), ProgramError::Custom(0), "a stranger names no miner (Unauthorized)");
    let (data, accounts) = warm.set_miner(agent, &miner);
    let cu = sealed(w, &warm.act(data, &accounts), "the polis names the miner");
    eprintln!("  the act set_miner by the polis's hand: {cu} CU");
    assert_eq!(&w.data(&warm.mine())[98..130], miner.as_ref(), "`Mine::OFF_MINER`: the router's crank, named by the polis");
    // 5. three flasks hold the world's dollars and deposit them, ten dollars the least: one by its own key paying, one through a relay that
    // pays, one after a deposit under the minimum is refused
    let relay = w.wallet(SOL);
    let flasks: Vec<Pubkey> = (0..3).map(|_| w.wallet(SOL)).collect();
    let deposits = [10 * USD, 25 * USD, 12 * USD + USD / 2];
    for (k, (f, amount)) in flasks.iter().zip(deposits).enumerate() {
        let source = w.pay(&dollar, f, amount);
        if k == 2 {
            w.refused(warm.deposit(*f, *f, &source, &treasury, &dollar, 10 * USD - 1), ProgramError::Custom(7), "a deposit under 10 USDC (UnderMinimum)");
        }
        let cu = w.done(warm.deposit(*f, if k == 1 { relay } else { *f }, &source, &treasury, &dollar, amount), "a flask's deposit");
        eprintln!("  deposit {amount} micro-dollars{}: {cu} CU", if k == 1 { ", a relay paying" } else { "" });
    }
    let total: u64 = deposits.iter().sum();
    assert_eq!(w.u64_at(&treasury, 64), total, "the polis's vault holds every deposit");
    // 6. the miner mines a day of spends across the first halving: six before it, six after, in two batches
    let (a, b, c) = (flasks[0], flasks[1], flasks[2]);
    let day = [(a, 1_500_000), (b, 3_000_000), (c, 2_000_000), (a, 2_500_000), (b, 4_000_000), (c, 1_234_567), (a, 3_000_000), (b, 5_000_000), (c, 4_000_000), (a, 3_000_000), (b, 2_000_000), (c, 1_000_001)];
    let mut landed: Vec<(Pubkey, u64, u64, u64)> = Vec::new();
    for (half, spends) in day.chunks(6).enumerate() {
        let batch: Vec<(Pubkey, u64, u64, u64)> = spends
            .iter()
            .enumerate()
            .map(|(i, (f, usd))| {
                let k = (half * 6 + i) as u64;
                (*f, *usd, halving - 11 * HOUR_MS + k * 2 * HOUR_MS, k + 1)
            })
            .collect();
        let cu = w.done(warm.spends(miner, &batch), if half == 0 { "the day's first half, the flasks' $WARM accounts made" } else { "the day's second half" });
        eprintln!("  mine, six spends of three flasks{}: {cu} CU", if half == 0 { ", their accounts made" } else { "" });
        landed.extend(batch);
    }
    // before the halving a micro-dollar mined a thousand nanos, after it five hundred: a's four dollars and six, four $WARM and three
    for (_, usd, at, _) in &landed {
        assert_eq!(warm_law::warm_nanos(*usd, *at, opening), (*usd as u128 * 1_000) >> u32::from(*at >= halving));
    }
    assert_eq!(w.u64_at(&warm.account_of(&a), 64), 7 * WARM, "a's $WARM, in its own account");
    // 7. THE BOUND: a's ten dollars are spent whole; a micro-dollar more is refused, and the batch it stands in with it
    let last = halving + 11 * HOUR_MS;
    w.refused(warm.spends(miner, &[(b, USD, last, 13), (a, 1, last, 14)]), ProgramError::Custom(11), "one micro-dollar past a's deposits, and b's spend beside it (PastDeposits)");
    // 8. a pays two $WARM back (a drink paid in $WARM): the vault holds them, and the next spends are paid from it before any mint
    let cu = w.done(warm.pay_back(a, a, 2 * WARM), "a pays $WARM back");
    eprintln!("  pay_back: {cu} CU");
    let minted = w.u64_at(&warm.mine(), 250);
    let next = halving + 12 * HOUR_MS;
    w.done(warm.spends(miner, &[(b, USD, next, 13)]), "the next spend, paid from the vault");
    assert_eq!((w.u64_at(&warm.mine(), 250), w.u64_at(&warm.vault(), 64)), (minted, 2 * WARM - WARM / 2), "half a $WARM out of the vault, nothing minted (`Mine::OFF_MINTED` 250)");
    w.done(warm.spends(miner, &[(c, 4 * USD, next, 14)]), "the one after: the vault emptied, the rest minted");
    assert_eq!((w.u64_at(&warm.mine(), 250), w.u64_at(&warm.vault(), 64)), (minted + WARM / 2, 0), "one and a half $WARM from the vault, half a one minted");
    landed.extend([(b, USD, next, 13), (c, 4 * USD, next, 14)]);
    // 9. the order and the clock, and the miner's key
    w.refused(warm.spends(miner, &[(b, 1, next, 14)]), ProgramError::Custom(8), "a seq that does not rise (SeqNotRising)");
    w.refused(warm.spends(miner, &[(b, 1, next - 1, 15)]), ProgramError::Custom(9), "a spend before the last (Backwards)");
    let clock = w.clock_ms();
    w.refused(warm.spends(miner, &[(b, 1, clock + HOUR_MS, 15)]), ProgramError::Custom(10), "a spend past the chain's clock (Future)");
    w.refused(warm.spends(stranger, &[(b, 1, next, 15)]), ProgramError::Custom(0), "a stranger mines nothing (Unauthorized)");
    // 10. THE BOOK, off the accounts: the mine's (`OFF_SPENDS` 218, `OFF_DEPOSITED` 226, `OFF_SPENT` 242, `OFF_MINTED` 250, `OFF_FROM_VAULT`
    // 258, `OFF_PAID_BACK` 266, `OFF_VAULT_BALANCE` 274) and each record's (`Flask::OFF_DEPOSITED` 66, `OFF_SPENT` 74, `OFF_MINED` 82)
    let mine = w.data(&warm.mine());
    let at = |d: &[u8], o: usize| u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
    let (spends, deposited, spent, minted, from_vault, paid_back, vault) = (at(&mine, 218), at(&mine, 226), at(&mine, 242), at(&mine, 250), at(&mine, 258), at(&mine, 266), at(&mine, 274));
    let law: u64 = landed.iter().map(|(_, usd, at, _)| u64::try_from(warm_law::warm_nanos(*usd, *at, opening)).unwrap()).sum();
    let (mut issued, mut held) = (0, w.u64_at(&warm.vault(), 64));
    for (f, want) in flasks.iter().zip(deposits) {
        let r = w.data(&warm.record(f));
        assert_eq!(at(&r, 66), want, "a flask's record counts its deposit");
        assert!(at(&r, 74) <= at(&r, 66), "THE BOUND: a flask spent no more than it deposited");
        issued += at(&r, 82);
        held += w.u64_at(&warm.account_of(f), 64);
    }
    let supply = w.u64_at(&warm.mint(), 36);
    assert_eq!((spends, deposited, spent), (landed.len() as u64, total, landed.iter().map(|s| s.1).sum::<u64>()), "every spend, every deposit");
    assert_eq!(issued, minted + from_vault, "THE BOOK: the $WARM issued is what was minted and what was paid out of the vault");
    assert_eq!(issued, law, "THE BOOK: the $WARM issued is the law's over every spend that landed, to the nano");
    assert_eq!((supply, held), (minted, supply), "THE BOOK: the supply is what was minted, and the flasks and the vault hold it");
    assert_eq!(vault, paid_back - from_vault, "THE BOOK: the vault holds what was paid back less what left it");
    let treasury_holds = w.u64_at(&treasury, 64);
    eprintln!(
        "  THE THIRD PROGRAM: {spends} spends across the halving at {halving} (the opening {opening}); deposited {deposited} micro-dollars, spent {spent}; issued {issued} nanos of $WARM (minted {minted}, from the vault {from_vault}), the law's {law}; paid back {paid_back}, the vault {vault}; the supply {supply}; the treasury {treasury_holds}"
    );
    let (last_ms, last_seq) = landed.last().map(|s| (s.2, s.3)).expect("a spend landed");
    MineBook { opening, halving, spends, deposited, spent, issued, minted, from_vault, paid_back, vault, supply, law, treasury: treasury_holds, dollar, miner, last_ms, last_seq }
}

/// What A BUDGET BECOMES CREDITS ends with, read off the accounts: the month's cap C and its period, what the branch paid the agent, both
/// roads' verdicts, root 0's vault of the dollar before the plan and after the mining, unit 1's vault after, the agent's record at the
/// mine (deposited, spent, mined), and what each unit's plan for the dollar was charged.
struct BudgetBook {
    cap: u64,
    period: u64,
    paid: u64,
    verdicts: Vec<u8>,
    root_before: u64,
    root_after: u64,
    branch_after: u64,
    deposited: u64,
    spent: u64,
    mined: u64,
    root_charged: u64,
    branch_charged: u64,
}

/// A BUDGET BECOMES CREDITS, in any world after the mine: a budget the market passes becomes an agent's $WARM, and no key of a person
/// stands between. Before it, the polis grows its branch: root 0 frees its twelfth chair (a founder recalled), lists the world's dollar
/// with a cap of its own (CODEX AllowMint: the dollar is not the estate's default mint, so a unit spends it only under a listing), and
/// founds unit 1 in that chair, twelve founders of its own and no floor; unit 1 lists the dollar for itself by its own chairs, and a vault
/// of it is opened for it. Then THE WHERE: root 0 proposes an ALLOWANCE in the dollar for its branch as a plan (CLASS_TREASURY |
/// CLASS_PLAN: the chairs and the road decide, the floor's ballot is not counted), the branch speaks in its chair, a staker lays the
/// road, traders trade PASS up, the road is decided PASSED at the deadline, root 0's seal reads it and the allowance stands; the same plan
/// with FAIL traded up is not PASSED and opens nothing. THE DRAW: anyone pulls the month's cap into unit 1's own vault, once a period.
/// THE HOW: unit 1 alone, by its own chairs, pays the agent a token budget. THE TANK: the agent, a wallet of its own key, deposits what it
/// was paid at the mine, so the dollars are back in root 0's vault (the mine's treasury) and its record counts them. IT MINES: the miner
/// the polis named posts the agent's spend, $WARM is mined to the agent, and THE BOUND refuses one micro-dollar more.
#[allow(clippy::too_many_arguments)]
fn the_budget_becomes_credits<L: Ledger>(
    w: &mut L,
    real: &Real,
    crank: Pubkey,
    crediter: Pubkey,
    proposer: &Voter,
    and: &[(&Voter, u8)],
    split: &[(&Voter, u8)],
    mine: &MineBook,
) -> BudgetBook {
    let warm = Warm { id: Warm::declared() };
    let (dollar, treasury) = (mine.dollar, vault_pda(ROOT_UNIT, &mine.dollar));
    const BRANCH: u64 = 1;
    // the month's cap, what the branch pays its agent (the mine's ten dollars the least), and the caps each unit lists the dollar under
    let (cap, paid) = (30 * USD, 12 * USD);
    let (root_listing, branch_listing) = (100 * USD, 20 * USD);
    let mut nonce = 50u64;
    // a sealed decision of root 0 alone, by the AND (seven chairs and five of the floor's six seats, decided at once): its execute's units
    let mut sealed = |w: &mut L, act: &Payload, what: &str| -> u64 {
        nonce += 1;
        let h = proposal_hash(act.class(), ROOT_UNIT, &MANIFEST, &act.hash(), nonce);
        w.done(propose_raw(proposer.key, h, act.class(), ROOT_UNIT, MANIFEST, &act.bytes(), &[ROOT_UNIT]), "propose the act");
        w.cast(&crank, &h, ROOT_UNIT, and);
        w.done(seal(crank, h, ROOT_UNIT), "seal the act");
        w.done(execute(crank, h, &[ROOT_UNIT], act), what)
    };

    // 0. THE BRANCH. Root 0's twelve chairs are its founders': the twelfth is freed by a recall, the dollar listed for root 0 while no
    // crew sits in a chair (a CODEX act is closed over every crew in a lit chair), then unit 1 founded in the freed chair
    sealed(w, &Payload::Recall { origin: ROOT_UNIT, chair: 11 }, "root 0 frees its twelfth chair");
    let cu = sealed(w, &Payload::AllowMint { origin: ROOT_UNIT, mint: dollar, cap: root_listing, payer: crank }, "root 0 lists the dollar");
    eprintln!("  A BUDGET BECOMES CREDITS: root 0's AllowMint of the dollar: {cu} CU");
    let branch: Vec<Voter> = (0..12).map(|_| w.voter(SOL)).collect();
    let spec = FoundSpec {
        child: BRANCH,
        chair: 11,
        charter: [BRANCH as u8; 32],
        window: WINDOW,
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        founders: branch.iter().map(|f| f.key).collect(),
        name: b"THE BRANCH".to_vec(),
        desc: Vec::new(),
    };
    let cu = sealed(w, &Payload::Found { origin: ROOT_UNIT, spec }, "root 0 founds its branch");
    eprintln!("  root 0 founds unit 1 in its twelfth chair: {cu} CU");
    // a crew seated in this slot is no voice of a question opened in it: the next question opens a slot later
    w.wait_until(w.slot() + 1);
    // the branch's own decision, by its own chairs alone (eight of twelve, no floor: decided at once); no vote of root 0
    let crew = pro(&branch, 8);
    let mut branch_nonce = 0u64;
    let mut branch_decides = |w: &mut L, act: &Payload, what: &str| -> u64 {
        branch_nonce += 1;
        let h = proposal_hash(act.class(), BRANCH, &MANIFEST, &act.hash(), branch_nonce);
        w.done(propose_raw(branch[0].key, h, act.class(), BRANCH, MANIFEST, &act.bytes(), &[BRANCH]), "the branch proposes");
        w.cast(&crank, &h, BRANCH, &crew);
        w.done(seal(crank, h, BRANCH), "the branch seals by its own chairs");
        assert_eq!(w.session(&h, BRANCH).state, STATE_PASSED, "{what}: passed by the branch's chairs");
        assert!(w.data(&session_pda(&h, ROOT_UNIT)).is_empty(), "{what}: root 0 holds no session of it, and cast no vote");
        w.done(execute(crank, h, &[BRANCH], act), what)
    };
    let cu = branch_decides(w, &Payload::AllowMint { origin: BRANCH, mint: dollar, cap: branch_listing, payer: crank }, "the branch lists the dollar");
    eprintln!("  unit 1's own AllowMint of the dollar: {cu} CU");
    w.done(create_vault(crank, BRANCH, dollar, TOKEN_PROGRAM_ID), "unit 1's vault of the dollar");
    let branch_vault = vault_pda(BRANCH, &dollar);
    assert_eq!(w.u64_at(&branch_vault, 64), 0, "unit 1's vault of the dollar stands empty");

    // 1. THE WHERE: the market decides the month. A fresh staker and three traders are credited on the board (its book was closed by the
    // market's chain: a credit adds to what the board was credited, never to a balance already counted)
    let layer = w.wallet(SOL);
    let traders: Vec<Pubkey> = (0..3).map(|_| w.wallet(SOL)).collect();
    w.done(real.credit(crediter, layer, 5_000 * MICRO), "credit the plan's staker");
    for t in &traders {
        w.done(real.credit(crediter, *t, 10_000 * MICRO), "credit a plan's trader");
    }
    let root_before = w.u64_at(&treasury, 64);
    let period = 1_000_000u64;
    let expires = w.slot() + 3 * period;
    let allowance = Payload::Allowance { origin: ROOT_UNIT, payee: unit_holder(BRANCH), payee_kind: HOLDER_UNIT, mint: dollar, cap, period, expires };
    let class = CLASS_TREASURY | CLASS_PLAN;
    let (mut verdicts, mut passed) = (Vec::new(), None);
    for (round, branch_up) in [(0u64, 0u8), (1, 1)] {
        let h = proposal_hash(class, ROOT_UNIT, &MANIFEST, &allowance.hash(), 60 + round);
        let mut asking = propose_raw(proposer.key, h, class, ROOT_UNIT, MANIFEST, &allowance.bytes(), &[ROOT_UNIT]);
        asking.accounts.push(AccountMeta::new_readonly(market_pda(ROOT_UNIT), false));
        w.done(asking, "an allowance for the branch, as a plan");
        // the chairs pass it (seven founders and the branch's own chair), the floor's ballot is not counted under a plan
        w.cast(&crank, &h, ROOT_UNIT, split);
        w.done(convene(crank, h, ROOT_UNIT, BRANCH), "the branch opens its own vote of the plan");
        w.cast(&crank, &h, BRANCH, &crew);
        w.done(seal(crank, h, BRANCH), "the branch seals: its chairs' majority, no floor");
        w.done(mandate(crank, h, ROOT_UNIT, BRANCH), "the branch's verdict is its chair's vote in root 0");
        let deadline = w.session(&h, ROOT_UNIT).deadline;
        let cu = w.done(real.lay_for(layer, &h), "the staker lays the plan's road");
        eprintln!("  lay_for the plan's road: {cu} CU");
        let opens = w.slot();
        for t in &traders {
            w.done(real.deposit(*t, &h, 3_000 * MICRO), "a trader's deposit");
        }
        for (k, t) in traders.iter().enumerate() {
            for n in 0..3u64 {
                w.wait_until(opens + 1 + (k as u64 * 3 + n) * 70);
                w.behind_then(real, &h, real.trade(*t, &h, branch_up, 0, 300 * MICRO), "a trade");
            }
        }
        // the floor never seals a plan early: the seal waits for the deadline, the road is decided there
        w.wait_until(deadline + 1);
        w.behind_then(real, &h, real.decide(&h), "decide the plan's road");
        verdicts.push(w.data(&real.road(&h))[42]);
        let mut s = seal(crank, h, ROOT_UNIT);
        s.accounts.extend([AccountMeta::new_readonly(market_pda(ROOT_UNIT), false), AccountMeta::new_readonly(real.road(&h), false)]);
        let cu = w.done(s, "root 0's seal reads the road");
        eprintln!("  root 0's seal of the plan, its market and its road read: {cu} CU");
        let state = w.session(&h, ROOT_UNIT).state;
        if round == 0 {
            assert_eq!(state, STATE_PASSED, "the chairs passed and the road passed: the plan is PASSED");
            let cu = w.done(execute(crank, h, &[ROOT_UNIT], &allowance), "the plan executes: the allowance stands");
            eprintln!("  execute the plan, the allowance opened: {cu} CU");
            passed = Some((h, cu));
        } else {
            assert_eq!(state, STATE_DEADLOCK, "the chairs passed and the road failed: the plan is not PASSED");
            w.refused(execute(crank, h, &[ROOT_UNIT], &allowance), ProgramError::Custom(code(PoleisError::NotPassed)), "a plan its road refused opens nothing");
            assert!(w.data(&allowance_pda(&h)).is_empty(), "no allowance stands for the plan the road refused");
        }
    }
    assert_eq!(verdicts, vec![1, 2], "the first plan's road passed, the second's failed");
    let (h, execute_cu) = passed.expect("the first plan passed");
    let a = AllowanceView::parse(&w.data(&allowance_pda(&h))).expect("the allowance");
    assert_eq!((a.unit, a.payee, a.payee_kind, a.mint, a.cap, a.period, a.spent, a.expires), (ROOT_UNIT, unit_holder(BRANCH), HOLDER_UNIT, dollar, cap, period, 0, expires), "the allowance the market passed: the branch draws C dollars a period");
    assert_eq!(w.u64_at(&treasury, 64), root_before, "an allowance moves nothing until it is drawn");

    // 2. THE DRAW: anyone (the crank, no key of the branch's) pulls the month's cap into unit 1's own vault; a second pull in the period is
    // refused, past the cap by one micro-dollar
    let pull_cu = w.done(pull(crank, h, ROOT_UNIT, cap, branch_vault, Some((dollar, TOKEN_PROGRAM_ID))), "the month's cap drawn into the branch's vault");
    eprintln!("  pull, the month's cap into unit 1's vault: {pull_cu} CU");
    assert_eq!((w.u64_at(&treasury, 64), w.u64_at(&branch_vault, 64)), (root_before - cap, cap), "C dollars moved from root 0's vault to unit 1's");
    w.refused(pull(crank, h, ROOT_UNIT, 1, branch_vault, Some((dollar, TOKEN_PROGRAM_ID))), ProgramError::Custom(code(PoleisError::AllowanceBreached)), "a second draw in the same period");

    // 3. THE HOW: unit 1 alone pays its agent, a wallet of its own key, into its own account of the dollar
    let agent = w.wallet(SOL);
    let source = w.pay(&dollar, &agent, 0);
    let pay = Payload::TokenBudget { origin: BRANCH, mint: dollar, address: source, amount: paid, token_program: TOKEN_PROGRAM_ID };
    let budget_cu = branch_decides(w, &pay, "the branch pays its agent");
    eprintln!("  execute unit 1's token budget to its agent: {budget_cu} CU");
    assert_eq!((w.u64_at(&branch_vault, 64), w.u64_at(&source, 64)), (cap - paid, paid), "the branch paid its agent PAID dollars out of its own vault");

    // 4. THE TANK: the agent deposits what it was paid at the mine, by its own key: back in root 0's vault, counted in its record
    let deposit_cu = w.done(warm.deposit(agent, agent, &source, &treasury, &dollar, paid), "the agent fills its tank");
    eprintln!("  the agent's deposit at the mine: {deposit_cu} CU");
    assert_eq!((w.u64_at(&source, 64), w.u64_at(&treasury, 64)), (0, root_before - cap + paid), "every dollar the agent was paid is back in root 0's vault");

    // 5. IT MINES: the miner the polis named posts the agent's spend, the whole of its deposit, at an instant after the last; THE BOUND
    // refuses one micro-dollar more
    let (at, seq) = (mine.last_ms, mine.last_seq + 1);
    let spend_cu = w.done(warm.spends(mine.miner, &[(agent, paid, at, seq)]), "the agent's spend mined");
    eprintln!("  mine, the agent's spend, its $WARM account made: {spend_cu} CU");
    let want = u64::try_from(warm_law::warm_nanos(paid, at, mine.opening)).unwrap();
    assert!(want > 0, "a spend mines $WARM");
    assert_eq!(w.u64_at(&warm.account_of(&agent), 64), want, "the agent's $WARM, by the law, in its own account");
    w.refused(warm.spends(mine.miner, &[(agent, 1, at, seq + 1)]), ProgramError::Custom(11), "one micro-dollar past the agent's deposit (PastDeposits)");

    // THE BOOK, off the accounts: root 0's vault, unit 1's, the agent's record (`Flask::OFF_DEPOSITED` 66, `OFF_SPENT` 74, `OFF_MINED`
    // 82), and each unit's plan for the dollar
    let record = w.data(&warm.record(&agent));
    let at8 = |d: &[u8], o: usize| u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
    let (deposited, spent, mined) = (at8(&record, 66), at8(&record, 74), at8(&record, 82));
    let (root_after, branch_after) = (w.u64_at(&treasury, 64), w.u64_at(&branch_vault, 64));
    let root_plan = TokenPlanView::parse(&w.data(&poleis_client::token_plan_pda(ROOT_UNIT, &dollar))).expect("root 0's plan for the dollar");
    let branch_plan = TokenPlanView::parse(&w.data(&poleis_client::token_plan_pda(BRANCH, &dollar))).expect("unit 1's plan for the dollar");
    assert_eq!(root_after, root_before - (cap - paid), "THE BOOK: root 0's vault lost only what the branch has not paid out yet");
    assert_eq!(branch_after, cap - paid, "THE BOOK: what the branch has not paid out sits in its own vault");
    assert_eq!(root_after + branch_after, root_before, "THE BOOK: not a cent left the polis; it went round through the agent and the mine");
    assert_eq!((deposited, spent, mined), (paid, paid, want), "THE BOOK: the agent's record counts what it was paid, spent it whole, and the $WARM it mined");
    assert!(spent <= deposited, "THE BOUND: the agent spent no more than it deposited");
    assert_eq!((root_plan.cap, root_plan.spent, branch_plan.cap, branch_plan.spent), (root_listing, cap, branch_listing, paid), "THE BOOK: root 0's plan was charged C, unit 1's PAID");
    eprintln!(
        "  A BUDGET BECOMES CREDITS: the plan's roads {verdicts:?}; C {cap} micro-dollars a period of {period} slots, PAID {paid}; root 0's vault {root_before} then {root_after}, unit 1's {branch_after}; the agent deposited {deposited}, spent {spent}, mined {mined} nanos of $WARM; CU: execute {execute_cu}, pull {pull_cu}, the branch's budget {budget_cu}, deposit {deposit_cu}, spend {spend_cu}"
    );
    BudgetBook { cap, period, paid, verdicts, root_before, root_after, branch_after, deposited, spent, mined, root_charged: root_plan.spent, branch_charged: branch_plan.spent }
}

/// The chain in any world: from a board opened for the polis's own hand to a budget paid on a road's verdict and the house's collect. All it
/// asserts holds in both worlds, and it prints the units of its world.
fn two_programs<L: Ledger>(e: Estate<L>) -> (L, Book) {
    let Estate { mut w, crank, founders, alice, bob, carol, .. } = e;
    let real = w.roads();
    let agent = agent_pda(ROOT_UNIT);
    let and = and_votes(&founders, &alice, &bob);
    w.done(endow(crank, ROOT_UNIT, 5 * SOL), "endow the root");

    // 1. someone opens a board naming the root's agent key as its authority: the opener pays and gains nothing
    let opener = w.wallet(SOL);
    w.done(real.init_board(opener, &agent), "a board opened for the polis's hand");
    let steal = real.ix(1, &[opener.as_ref(), opener.as_ref()].concat(), vec![AccountMeta::new_readonly(opener, true), AccountMeta::new(real.board(), false)]);
    w.refused(steal, ProgramError::Custom(0), "the opener names no role (Unauthorized)");
    let rename =
        real.ix(1, &[opener.as_ref(), opener.as_ref(), opener.as_ref()].concat(), vec![AccountMeta::new_readonly(agent, false), AccountMeta::new(real.board(), false)]);
    w.refused(rename, ProgramError::InvalidInstructionData, "a board's authority is named once: set_board names no authority");

    // 2. the polis, by sealed ACT_RAWs signed by its own hand, sets the board's law and names a crediter and a judge
    let (crediter, judge) = (w.wallet(10 * SOL), w.wallet(SOL));
    let step: u64 = 10;
    let mut law = PROGRAM_ID.as_ref().to_vec();
    law.extend_from_slice(&ROOT_UNIT.to_le_bytes());
    law.extend_from_slice(&step.to_le_bytes());
    for v in [100u32, 300, 30] {
        law.extend_from_slice(&v.to_le_bytes());
    }
    // the pool, and the judge's 5,000 slots after a road's close (the first road's receipts come after the second road's window)
    for v in [1_000 * MICRO, 1_000 * MICRO, 5_000u64] {
        law.extend_from_slice(&v.to_le_bytes());
    }
    let acts = [(11u8, law.clone()), (1u8, [crediter.as_ref(), judge.as_ref()].concat())];
    for (i, (tag, data)) in acts.iter().enumerate() {
        let mut bytes = vec![*tag];
        bytes.extend_from_slice(data);
        let act = Payload::Raw { origin: ROOT_UNIT, program: real.id, accounts: vec![(agent, ACT_SIGNER), (real.board(), ACT_WRITABLE)], data: bytes };
        let h = proposal_hash(act.class(), ROOT_UNIT, &MANIFEST, &act.hash(), 1 + i as u64);
        w.done(propose_raw(alice.key, h, act.class(), ROOT_UNIT, MANIFEST, &act.bytes(), &[ROOT_UNIT]), "propose the act");
        w.cast(&crank, &h, ROOT_UNIT, &and);
        w.done(seal(crank, h, ROOT_UNIT), "seal the act");
        let cu = w.done(execute(crank, h, &[ROOT_UNIT], &act), "the polis's own hand calls the roads program");
        eprintln!("  the act {} by the polis's hand: {cu} CU", if *tag == 11 { "set_law" } else { "set_board" });
    }
    let b = w.data(&real.board());
    assert_eq!(&b[10..42], agent.as_ref(), "Board::OFF_AUTHORITY: the polis's hand");
    assert_eq!((&b[42..74], &b[74..106]), (crediter.as_ref(), judge.as_ref()), "Board::OFF_CREDITER, OFF_JUDGE: named by the polis");
    assert_eq!((&b[114..146], &b[146..154]), (PROGRAM_ID.as_ref(), &ROOT_UNIT.to_le_bytes()[..]), "Board::OFF_POLIS, OFF_UNIT: Poleis and the root");

    // 3. the polis names its market by CODEX_MARKET, a tenth of the board its bar: the board is its own, taken
    let market = market_law(&real.id, REAL_BOARD, MARKET_TREASURY, PLAN_CLASSES, 1_000);
    let hm = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &sha(&[&market]), 10);
    w.done(propose_raw(alice.key, hm, CLASS_CODEX, ROOT_UNIT, MANIFEST, &market, &[ROOT_UNIT]), "propose the market's law");
    w.cast(&crank, &hm, ROOT_UNIT, &and);
    w.done(seal(crank, hm, ROOT_UNIT), "seal the market's law");
    let cu = w.done(execute_law(crank, hm, &[ROOT_UNIT], &market), "CODEX_MARKET with its own board");
    eprintln!("  CODEX_MARKET with its board: {cu} CU");
    w.wait_until(w.slot() + 1);

    // THE MARKET NEVER DECIDES ITS OWN LAW: a proposal asking the market for an act that sets its own board's law is refused
    // (as a plan: the law lets the hands be plans, so only the rule against the market's own law refuses it)
    let mut bytes = vec![11u8];
    bytes.extend_from_slice(&law);
    let own_law = Payload::Raw { origin: ROOT_UNIT, program: real.id, accounts: vec![(agent, ACT_SIGNER), (real.board(), ACT_WRITABLE)], data: bytes };
    let ho = proposal_hash(CLASS_ACT | CLASS_PLAN, ROOT_UNIT, &MANIFEST, &own_law.hash(), 11);
    let mut asking = propose_raw(alice.key, ho, CLASS_ACT | CLASS_PLAN, ROOT_UNIT, MANIFEST, &own_law.bytes(), &[ROOT_UNIT]);
    asking.accounts.push(AccountMeta::new_readonly(market_pda(ROOT_UNIT), false));
    w.refused(asking, ProgramError::Custom(code(PoleisError::MarketNotAllowed)), "the market asked to decide its own law");

    let split = chairs_not_floor(&founders, &carol);
    let m = the_market(&mut w, &real, crank, Floor::Keys { proposer: &alice, split: &split }, crediter, |w: &mut L, h: &[u8; 32], world: u8, worth: u64| {
        w.done(real.receipts(judge, h, world, worth), "the judge's receipts");
    }, None);
    // THE THIRD PROGRAM, in the same world after the market's chain: $WARM mined by use
    let mine = the_mine(&mut w, crank, &alice, &and);
    // A BUDGET BECOMES CREDITS, in the same world after the mine: the market decides the branch's month, the branch pays an agent, the
    // agent fills its tank at the mine and mines
    let budget = the_budget_becomes_credits(&mut w, &real, crank, crediter, &alice, &and, &split, &mine);
    (w, Book { verdicts: m.verdicts, paid: m.paid, backs: m.backs, earned: m.earned, sum: m.sum, credited: m.credited, mine, budget })
}

/// What the market's chain ends with: each road's verdict, what each budget paid, each stake back, what the house earned, and the book (the
/// balances and what the house earned, against what the crediter credited).
struct MarketBook {
    verdicts: Vec<u8>,
    paid: Vec<u64>,
    backs: Vec<u64>,
    earned: u64,
    sum: u64,
    credited: u64,
}

/// A person of the floor: a wallet of the public that minted one Lyssajudron of root 0 (THE LAUNCH PRESS's rehearsal, THE WHOLE TREE
/// DECIDES), its key in this run's memory, and its seat's proof as the chain holds it (the seat's mint and the person's own token account).
struct Person {
    v: Voter,
    proof: poleis_client::nft::OwnerProof,
}
impl Person {
    fn key(&self) -> Pubkey {
        self.v.key
    }
    /// Its seat's own vote of `hash` on root 0's floor: its own transaction (`attest_nft`), the person its one signer and its payer.
    fn vote<L: Ledger>(&self, w: &mut L, hash: &[u8; 32], choice: u8, what: &str) -> u64 {
        let ix = self.proof.attest(*hash, choice).expect("a seat's vote");
        w.attempt_all(&[ix], &self.v.key, what).unwrap_or_else(|e| panic!("{what}: refused with {e:?}"))
    }
}

/// Root 0's floor in the market's chain: who proposes the budgets, and the floor's votes that leave them split. A key seat's holder and the
/// floor's envelopes the crank lands (THE TWO PROGRAMS IN ONE WORLD), or a seat owner of the minted floor and each owner's own transaction
/// (THE LAUNCH PRESS, THE WHOLE TREE DECIDES).
enum Floor<'a> {
    Keys { proposer: &'a Voter, split: &'a [(&'a Voter, u8)] },
    Seats { proposer: &'a Person, split: &'a [(&'a Person, u8)] },
}
impl Floor<'_> {
    /// A proposal of root 0 by the floor's proposer, its class and payload as given.
    fn propose_raw(&self, h: [u8; 32], class: u8, payload: &[u8]) -> Instruction {
        match self {
            Floor::Keys { proposer, .. } => propose_raw(proposer.key, h, class, ROOT_UNIT, MANIFEST, payload, &[ROOT_UNIT]),
            Floor::Seats { proposer, .. } => proposer.proof.propose_raw(ROOT_UNIT, h, class, MANIFEST, payload, &[ROOT_UNIT]).expect("a seat owner's proposal"),
        }
    }
    /// The floor's votes of `h`, short of its majority.
    fn split<L: Ledger>(&self, w: &mut L, crank: &Pubkey, h: &[u8; 32]) {
        match self {
            Floor::Keys { split, .. } => w.cast(crank, h, ROOT_UNIT, split),
            Floor::Seats { split, .. } => {
                for (p, c) in split.iter() {
                    p.vote(&mut *w, h, *c, "a seat owner's own vote of the budget");
                }
            }
        }
    }
}

/// Steps 4 to 9 of THE TWO PROGRAMS IN ONE WORLD, on a board whose law and roles the polis set by its own hand and whose market it named by
/// CODEX_MARKET: credits, a budget asking the market twice (its road decided both ways, the budget paid once), the judge's receipts, the
/// settles and collects, and the board's book. `floor` proposes the budgets and its split passes them in the chairs and not in the floor.
/// THE LAUNCH PRESS ON A LIVE VALIDATOR runs it on the board the press opened, where root 0's only lit chair holds a crew (his posiverse):
/// `crew` names it and its agents' votes, and the crew opens its own vote of each budget, votes, seals and is heard in its chair
/// (`convene`, `seal`, `mandate`) before root 0 seals; the split is then the minted floor's own votes alone, short of its majority.
/// `judge` says a decided road's receipts (its world, its worth): a judge's key on THE TWO PROGRAMS IN ONE WORLD's board, and on the launch
/// press's board the swarm by its own vote (THE SWARM JUDGES: a sealed act of root 0's own hand).
fn the_market<L: Ledger>(w: &mut L, real: &Real, crank: Pubkey, floor: Floor<'_>, crediter: Pubkey, mut judge: impl FnMut(&mut L, &[u8; 32], u8, u64), crew: Option<(u64, &[(&Voter, u8)])>) -> MarketBook {
    let agent = agent_pda(ROOT_UNIT);
    // 4. the crediter credits a staker and three traders; the house (the polis's hand) is credited nothing
    let layer = w.wallet(SOL);
    let traders: Vec<Pubkey> = (0..3).map(|_| w.wallet(SOL)).collect();
    w.done(real.credit(crediter, layer, 5_000 * MICRO), "credit the staker");
    for t in &traders {
        w.done(real.credit(crediter, *t, 10_000 * MICRO), "credit a trader");
    }
    let credited = 5_000 * MICRO + 3 * 10_000 * MICRO;

    let lab = w.wallet(SOL);
    let budget = Payload::Budget { origin: ROOT_UNIT, address: lab, amount: SOL / 2 };
    let (mut verdicts, mut paid, mut hashes) = (Vec::new(), Vec::new(), Vec::new());
    for (round, branch) in [(0u64, 0u8), (1, 1)] {
        // 5. a budget proposed with CLASS_MARKET; the chairs pass it, the floor does not
        let h = proposal_hash(CLASS_TREASURY | CLASS_MARKET, ROOT_UNIT, &MANIFEST, &budget.hash(), 20 + round);
        let mut asking = floor.propose_raw(h, CLASS_TREASURY | CLASS_MARKET, &budget.bytes());
        asking.accounts.push(AccountMeta::new_readonly(market_pda(ROOT_UNIT), false));
        w.done(asking, "a budget that asks the market");
        floor.split(w, &crank, &h);
        if let Some((unit, agents)) = crew {
            w.done(convene(crank, h, ROOT_UNIT, unit), "the crew opens its own vote of the budget");
            w.cast(&crank, &h, unit, agents);
            w.done(seal(crank, h, unit), "the crew seals: its chairs' majority, no floor");
            w.done(mandate(crank, h, ROOT_UNIT, unit), "the crew's verdict is its chair's vote in root 0");
        }
        let deadline = w.session(&h, ROOT_UNIT).deadline;

        // 6. a staker lays its road by rule from its own balance (a stranger with none cannot); the traders deposit and trade one
        // branch up while the clock moves through the window
        let stranger = w.wallet(SOL);
        w.refused(real.lay_for(stranger, &h), ProgramError::Custom(2), "a stranger with no balance on the board lays nothing (InvalidAccount)");
        let cu = w.done(real.lay_for(layer, &h), "the staker lays the road by rule");
        eprintln!("  lay_for: {cu} CU");
        let opens = w.slot();
        for t in &traders {
            w.done(real.deposit(*t, &h, 3_000 * MICRO), "a trader's deposit");
        }
        for (k, t) in traders.iter().enumerate() {
            for n in 0..3u64 {
                w.wait_until(opens + 1 + (k as u64 * 3 + n) * 70);
                w.behind_then(&real, &h, real.trade(*t, &h, branch, 0, 300 * MICRO), "a trade");
            }
        }

        // 7. at the deadline the road is decided, and the origin's seal reads it; 8. the second round, the other way
        w.wait_until(deadline + 1);
        w.behind_then(&real, &h, real.decide(&h), "decide");
        // a road's verdict, `Road::OFF_VERDICT` 42 (1 passed, 2 failed)
        verdicts.push(w.data(&real.road(&h))[42]);
        let mut s = seal(crank, h, ROOT_UNIT);
        s.accounts.extend([AccountMeta::new_readonly(market_pda(ROOT_UNIT), false), AccountMeta::new_readonly(real.road(&h), false)]);
        w.done(s, "the origin's seal with the market");
        let state = w.session(&h, ROOT_UNIT).state;
        let before = w.lamports(&lab);
        if state == STATE_PASSED {
            w.done(execute(crank, h, &[ROOT_UNIT], &budget), "execute pays the budget");
        } else {
            w.refused(execute(crank, h, &[ROOT_UNIT], &budget), ProgramError::Custom(code(PoleisError::NotPassed)), "nothing is paid");
        }
        paid.push(w.lamports(&lab) - before);
        assert_eq!((round, state), if round == 0 { (0, STATE_PASSED) } else { (1, STATE_DEADLOCK) });
        hashes.push(h);
    }
    assert_eq!(verdicts, vec![1, 2], "the first road passed, the second failed");
    assert_eq!(paid, vec![SOL / 2, 0]);

    // 9. the judge says the receipts, the positions settle, each stake comes back, the house earns the rest, and the board's book closes
    // (a balance's amount and in_roads: `Balance::OFF_AMOUNT` 42, `Balance::OFF_IN_ROADS` 50)
    let mut backs = Vec::new();
    for (i, h) in hashes.iter().enumerate() {
        judge(&mut *w, h, i as u8, 600_000_000_000);
        for t in &traders {
            w.done(real.settle(h, *t), "a settle");
        }
        let a0 = w.u64_at(&real.balance(&layer), 42);
        let cu = w.done(real.collect(h, layer), "the staker's collect");
        eprintln!("  collect: {cu} CU");
        backs.push(w.u64_at(&real.balance(&layer), 42) - a0);
    }
    assert_eq!(backs, vec![2_000 * MICRO, 2_000 * MICRO], "each staker's stake is back whole: the pool gained on both roads");
    // `Board::OFF_EARNED` 198: what the house earned on the board; `Board::OFF_CREDITED` 106
    let earned = w.u64_at(&real.board(), 198);
    let mut sum = earned;
    for o in traders.iter().chain([&layer]) {
        let balance = real.balance(o);
        assert_eq!(w.u64_at(&balance, 50), 0, "nothing of it is left in a road");
        sum += w.u64_at(&balance, 42);
    }
    assert!(w.data(&real.balance(&agent)).is_empty(), "the house holds no balance on the board at any point");
    assert_eq!(w.u64_at(&real.board(), 106), credited);
    assert_eq!(sum, credited, "THE BOOK: the balances and what the house earned are what the crediter credited");
    eprintln!(
        "  THE TWO PROGRAMS IN ONE WORLD: verdicts {verdicts:?} (1 passed, 2 failed); paid {paid:?} lamports; each stake back {backs:?}; the house earned {earned}; the book {sum} of {credited}"
    );
    MarketBook { verdicts, paid, backs, earned, sum, credited }
}

#[test]
fn the_two_programs_in_one_world_from_the_polis_own_board_to_a_budget_paid_on_a_roads_verdict() {
    let (_, book) = two_programs(estate(false));
    // A BUDGET BECOMES CREDITS, its book as the chain left it
    let b = &book.budget;
    assert_eq!(b.verdicts, vec![1, 2], "the plan passed by its road, and the plan its road refused");
    assert_eq!((b.root_before - b.root_after, b.branch_after), (b.cap - b.paid, b.cap - b.paid), "root 0 is short only what the branch still holds");
    assert_eq!((b.root_charged, b.branch_charged), (b.cap, b.paid), "root 0's plan charged C, unit 1's PAID");
    assert!(b.deposited == b.paid && b.spent <= b.deposited && b.mined > 0 && b.period > 0, "the agent filled its tank with what it was paid and mined");
}

// ---- THE TWO PROGRAMS ON A LIVE VALIDATOR ----
// THE MARKET ON A LOCAL CHAIN (Nov 21 2026 14:4x UTC, his "Yes, rehearse it" to "Rehearse the market on a real local chain, then hand you the
// devnet press?"): the same body on a real validator, with real transactions, real slots and real signatures. What a validator proves that
// Mollusk does not: the 1,232-byte transaction, each instruction's compute budget, the signatures' precompile as a validator runs it, account
// locks and rent, the polis's hand calling through the real loader, the clock as slots really pass, and the RPC's own answers. The devnet
// press stays his: a live world talks to loopback and nothing else.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read as _, Write as _};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::str::FromStr as _;
use std::time::{Duration, Instant};

/// The size a validator holds every transaction to: an IPv6 packet's 1,280 bytes less its headers.
const PACKET: usize = 1_232;

/// A JSON-RPC door on this machine and on no other: `http://` to 127.0.0.1, [::1] or localhost, read as an address and never asked of a
/// resolver, so a live world can never be pointed at a cluster.
#[derive(Debug)]
struct Loopback {
    at: SocketAddr,
    host: String,
    path: String,
}
impl Loopback {
    fn parse(url: &str) -> Result<Self, String> {
        let refuse = |why: &str| -> Result<Self, String> {
            Err(format!("{url:?}: {why}: a live world talks only to a validator on this machine (http://127.0.0.1, http://[::1] or http://localhost)"))
        };
        let Some(rest) = url.strip_prefix("http://") else { return refuse("not http") };
        let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        let split = match authority.strip_prefix('[') {
            Some(v6) => v6.split_once("]:"),
            None => authority.rsplit_once(':'),
        };
        let Some((host, port)) = split else { return refuse("no port") };
        let ip = match host {
            "127.0.0.1" | "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
            "::1" => IpAddr::V6(Ipv6Addr::LOCALHOST),
            _ => return refuse("not loopback"),
        };
        let Some(port) = port.parse::<u16>().ok().filter(|p| *p != 0) else { return refuse("not a port") };
        if path.bytes().any(|b| b <= b' ' || b >= 0x7f) {
            return refuse("not a path");
        }
        Ok(Self { at: SocketAddr::new(ip, port), host: authority.to_string(), path: if path.is_empty() { "/".to_string() } else { path.to_string() } })
    }
    /// One JSON-RPC call, one connection: its `result`, or the RPC's own words.
    fn call(&self, method: &str, params: &str) -> Result<Json, String> {
        assert!(self.at.ip().is_loopback(), "a live world talks to loopback only");
        let fail = |e: std::io::Error| format!("{method}: {e}");
        let body = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"{method}","params":{params}}}"#);
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nAccept: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            self.path,
            self.host,
            body.len()
        );
        let mut stream = TcpStream::connect_timeout(&self.at, Duration::from_secs(10)).map_err(fail)?;
        stream.set_read_timeout(Some(Duration::from_secs(120))).map_err(fail)?;
        stream.set_write_timeout(Some(Duration::from_secs(30))).map_err(fail)?;
        let mut raw = Vec::new();
        stream.write_all(request.as_bytes()).and_then(|_| stream.read_to_end(&mut raw)).map_err(fail)?;
        let reply = http_body(&raw).and_then(|b| Json::parse(&b)).map_err(|e| format!("{method}: {e}"))?;
        if reply.get("error").is_null() {
            Ok(reply.take("result"))
        } else {
            Err(format!("{method}: {}", reply.get("error").text()))
        }
    }
}

/// The body of an HTTP/1.1 reply of 200, by its length or by its chunks.
fn http_body(raw: &[u8]) -> Result<String, String> {
    let at = raw.windows(4).position(|w| w == b"\r\n\r\n").ok_or("HTTP: a reply with no head")?;
    let head = std::str::from_utf8(&raw[..at]).map_err(|_| "HTTP: a head that is not text")?;
    let mut lines = head.split("\r\n");
    let status = lines.next().unwrap_or_default();
    if !(status.starts_with("HTTP/1.1 200") || status.starts_with("HTTP/1.0 200")) {
        return Err(format!("HTTP: the RPC answered {status:?}"));
    }
    let (mut chunked, mut length) = (false, None);
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            match k.trim().to_ascii_lowercase().as_str() {
                "transfer-encoding" => chunked = v.to_ascii_lowercase().contains("chunked"),
                "content-length" => length = v.trim().parse::<usize>().ok(),
                _ => {}
            }
        }
    }
    let (mut body, mut out) = (&raw[at + 4..], Vec::new());
    if chunked {
        loop {
            let eol = body.windows(2).position(|w| w == b"\r\n").ok_or("HTTP: a chunk with no size")?;
            let size = std::str::from_utf8(&body[..eol])
                .ok()
                .and_then(|s| usize::from_str_radix(s.split(';').next().unwrap_or_default().trim(), 16).ok())
                .ok_or("HTTP: a chunk's size that is not hex")?;
            body = &body[eol + 2..];
            if size == 0 {
                break;
            }
            out.extend_from_slice(body.get(..size).ok_or("HTTP: a chunk cut short")?);
            body = body.get(size..).and_then(|r| r.strip_prefix(b"\r\n")).ok_or("HTTP: a chunk not closed")?;
        }
    } else {
        out.extend_from_slice(match length {
            Some(n) => body.get(..n).ok_or("HTTP: a body cut short")?,
            None => body,
        });
    }
    String::from_utf8(out).map_err(|_| "HTTP: a body that is not text".to_string())
}

/// JSON, as much of it as a validator's RPC and the certificate need. A number keeps its text, so no u64 is ever rounded through a float.
#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}
static NULL_JSON: Json = Json::Null;

impl Json {
    fn string(s: impl Into<String>) -> Self {
        Json::Str(s.into())
    }
    fn number(n: impl std::fmt::Display) -> Self {
        Json::Num(n.to_string())
    }
    fn object(fields: Vec<(&str, Json)>) -> Self {
        Json::Obj(fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
    fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(o) => o.iter().find(|(k, _)| k == key).map(|(_, v)| v).unwrap_or(&NULL_JSON),
            _ => &NULL_JSON,
        }
    }
    /// The field `key`, taken out whole: an account's bytes are not copied.
    fn take(self, key: &str) -> Json {
        match self {
            Json::Obj(o) => o.into_iter().find(|(k, _)| k == key).map(|(_, v)| v).unwrap_or(Json::Null),
            _ => Json::Null,
        }
    }
    fn at(&self, n: usize) -> &Json {
        match self {
            Json::Arr(a) => a.get(n).unwrap_or(&NULL_JSON),
            _ => &NULL_JSON,
        }
    }
    fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Num(n) => n.parse().ok(),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    fn is_null(&self) -> bool {
        matches!(self, Json::Null)
    }

    fn parse(text: &str) -> Result<Json, String> {
        let (b, mut i) = (text.as_bytes(), 0);
        let v = Self::read(b, &mut i)?;
        Self::blank(b, &mut i);
        if i < b.len() {
            return Err(format!("JSON: more after the value, at {i}"));
        }
        Ok(v)
    }
    fn blank(b: &[u8], i: &mut usize) {
        while b.get(*i).is_some_and(|c| matches!(c, b' ' | b'\t' | b'\r' | b'\n')) {
            *i += 1;
        }
    }
    fn read(b: &[u8], i: &mut usize) -> Result<Json, String> {
        Self::blank(b, i);
        let Some(&c) = b.get(*i) else { return Err("JSON: the text ended".to_string()) };
        match c {
            b'{' => {
                *i += 1;
                let mut o = Vec::new();
                Self::blank(b, i);
                if b.get(*i) == Some(&b'}') {
                    *i += 1;
                    return Ok(Json::Obj(o));
                }
                loop {
                    Self::blank(b, i);
                    let k = Self::read_string(b, i)?;
                    Self::blank(b, i);
                    if b.get(*i) != Some(&b':') {
                        return Err(format!("JSON: no colon at {}", *i));
                    }
                    *i += 1;
                    o.push((k, Self::read(b, i)?));
                    Self::blank(b, i);
                    match b.get(*i) {
                        Some(b',') => *i += 1,
                        Some(b'}') => {
                            *i += 1;
                            return Ok(Json::Obj(o));
                        }
                        _ => return Err(format!("JSON: an object not closed at {}", *i)),
                    }
                }
            }
            b'[' => {
                *i += 1;
                let mut a = Vec::new();
                Self::blank(b, i);
                if b.get(*i) == Some(&b']') {
                    *i += 1;
                    return Ok(Json::Arr(a));
                }
                loop {
                    a.push(Self::read(b, i)?);
                    Self::blank(b, i);
                    match b.get(*i) {
                        Some(b',') => *i += 1,
                        Some(b']') => {
                            *i += 1;
                            return Ok(Json::Arr(a));
                        }
                        _ => return Err(format!("JSON: an array not closed at {}", *i)),
                    }
                }
            }
            b'"' => Ok(Json::Str(Self::read_string(b, i)?)),
            b't' | b'f' | b'n' => {
                for (word, v) in [("true", Json::Bool(true)), ("false", Json::Bool(false)), ("null", Json::Null)] {
                    if b[*i..].starts_with(word.as_bytes()) {
                        *i += word.len();
                        return Ok(v);
                    }
                }
                Err(format!("JSON: a word that is none at {}", *i))
            }
            b'-' | b'0'..=b'9' => {
                let s = *i;
                *i += 1;
                while b.get(*i).is_some_and(|c| c.is_ascii_digit() || b"+-.eE".contains(c)) {
                    *i += 1;
                }
                Ok(Json::Num(String::from_utf8_lossy(&b[s..*i]).into_owned()))
            }
            _ => Err(format!("JSON: {:?} at {}", c as char, *i)),
        }
    }
    fn read_string(b: &[u8], i: &mut usize) -> Result<String, String> {
        if b.get(*i) != Some(&b'"') {
            return Err(format!("JSON: no string at {}", *i));
        }
        *i += 1;
        let mut out = Vec::new();
        loop {
            let Some(&c) = b.get(*i) else { return Err("JSON: a string not closed".to_string()) };
            *i += 1;
            match c {
                b'"' => return String::from_utf8(out).map_err(|_| "JSON: a string that is not UTF-8".to_string()),
                b'\\' => {
                    let Some(&e) = b.get(*i) else { return Err("JSON: an escape cut short".to_string()) };
                    *i += 1;
                    match e {
                        b'"' | b'\\' | b'/' => out.push(e),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let mut code = Self::hex4(b, i)?;
                            // a character past the first plane comes as two halves
                            if (0xD800..0xDC00).contains(&code) && b.get(*i..*i + 2) == Some(&b"\\u"[..]) {
                                *i += 2;
                                code = 0x10000 + ((code - 0xD800) << 10) + (Self::hex4(b, i)?.wrapping_sub(0xDC00) & 0x3FF);
                            }
                            out.extend_from_slice(char::from_u32(code).unwrap_or('\u{fffd}').encode_utf8(&mut [0; 4]).as_bytes());
                        }
                        _ => return Err(format!("JSON: an escape {:?} at {}", e as char, *i)),
                    }
                }
                _ => out.push(c),
            }
        }
    }
    fn hex4(b: &[u8], i: &mut usize) -> Result<u32, String> {
        let code = b
            .get(*i..*i + 4)
            .filter(|h| h.iter().all(u8::is_ascii_hexdigit))
            .and_then(|h| u32::from_str_radix(std::str::from_utf8(h).ok()?, 16).ok())
            .ok_or_else(|| format!("JSON: a \\u escape at {}", *i))?;
        *i += 4;
        Ok(code)
    }

    /// The text on one line.
    fn text(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, None, 0);
        s
    }
    /// The text a certificate is kept in: a field a line, a space a level, and a value whose text is short kept whole on its line.
    fn pretty(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, Some(1), 0);
        s
    }
    fn write(&self, out: &mut String, indent: Option<usize>, depth: usize) {
        let line = |out: &mut String, indent: Option<usize>, depth: usize| {
            if let Some(n) = indent {
                out.push('\n');
                out.push_str(&" ".repeat(n * depth));
            }
        };
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
            Json::Num(n) => out.push_str(n),
            Json::Str(s) => quote(s, out),
            Json::Arr(a) => {
                let wide = indent.filter(|_| !a.is_empty() && self.text().len() > 240);
                out.push('[');
                for (k, v) in a.iter().enumerate() {
                    if k > 0 {
                        out.push(',');
                    }
                    line(out, wide, depth + 1);
                    v.write(out, wide, depth + 1);
                }
                line(out, wide, depth);
                out.push(']');
            }
            Json::Obj(o) => {
                let wide = indent.filter(|_| !o.is_empty() && self.text().len() > 240);
                out.push('{');
                for (k, (key, v)) in o.iter().enumerate() {
                    if k > 0 {
                        out.push(',');
                    }
                    line(out, wide, depth + 1);
                    quote(key, out);
                    out.push_str(if wide.is_some() { ": " } else { ":" });
                    v.write(out, wide, depth + 1);
                }
                line(out, wide, depth);
                out.push('}');
            }
        }
    }
}

fn quote(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Base58 as Solana writes a key or a signature (held to the address's own encoding by a test).
fn base58(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut digits: Vec<u8> = Vec::new(); // in base 58, the lowest first
    for &byte in bytes {
        let mut carry = u32::from(byte);
        for d in digits.iter_mut() {
            carry += u32::from(*d) << 8;
            *d = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let zeros = bytes.iter().take_while(|b| **b == 0).count();
    std::iter::repeat_n('1', zeros).chain(digits.iter().rev().map(|d| ALPHABET[usize::from(*d)] as char)).collect()
}

/// compact-u16, Solana's length on the wire: seven bits a byte, the lowest first, the high bit saying another byte follows.
fn compact(n: usize, out: &mut Vec<u8>) {
    let mut n = u16::try_from(n).expect("a length on the wire fits sixteen bits");
    loop {
        let low = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(low);
            return;
        }
        out.push(low | 0x80);
    }
}

/// A legacy message, compiled as Solana's own compiler compiles one (solana-message 4.4.1, `compiled_keys.rs`): every key once, its flags
/// the OR of its uses, the payer first, then the writable signers, the read-only signers, the writable rest and the read-only rest, each
/// group in the keys' byte order; then the blockhash, and each instruction by the index of its program and of its accounts. Returns the
/// message's bytes and the keys that sign it, in their order.
fn message_of(ixs: &[Instruction], payer: &Pubkey, blockhash: &[u8; 32]) -> (Vec<u8>, Vec<Pubkey>) {
    let mut flags: BTreeMap<Pubkey, (bool, bool)> = BTreeMap::new();
    for ix in ixs {
        flags.entry(ix.program_id).or_default();
        for meta in &ix.accounts {
            let f = flags.entry(meta.pubkey).or_default();
            f.0 |= meta.is_signer;
            f.1 |= meta.is_writable;
        }
    }
    flags.remove(payer);
    let group = |signer: bool, writable: bool| -> Vec<Pubkey> { flags.iter().filter(|(_, f)| **f == (signer, writable)).map(|(k, _)| *k).collect() };
    let (signing, reading, writing, rest) = (group(true, true), group(true, false), group(false, true), group(false, false));
    let keys: Vec<Pubkey> = [vec![*payer], signing.clone(), reading.clone(), writing, rest.clone()].concat();
    let count = |n: usize| u8::try_from(n).expect("a message holds at most 255 keys");
    let signers = 1 + signing.len() + reading.len();
    let mut m = vec![count(signers), count(reading.len()), count(rest.len())];
    compact(keys.len(), &mut m);
    for k in &keys {
        m.extend_from_slice(k.as_ref());
    }
    m.extend_from_slice(blockhash);
    compact(ixs.len(), &mut m);
    let index = |k: &Pubkey| count(keys.iter().position(|x| x == k).expect("every key is compiled"));
    for ix in ixs {
        m.push(index(&ix.program_id));
        compact(ix.accounts.len(), &mut m);
        for meta in &ix.accounts {
            m.push(index(&meta.pubkey));
        }
        compact(ix.data.len(), &mut m);
        m.extend_from_slice(&ix.data);
    }
    (m, keys[..signers].to_vec())
}

/// A transaction on the wire: its signatures, in the order of the keys that sign, then the message.
fn transaction_of(message: &[u8], signatures: &[[u8; 64]]) -> Vec<u8> {
    let mut t = Vec::with_capacity(1 + 64 * signatures.len() + message.len());
    compact(signatures.len(), &mut t);
    for s in signatures {
        t.extend_from_slice(s);
    }
    t.extend_from_slice(message);
    t
}

/// A refusal as a simulation (or a landed transaction's status) words it: the `ProgramError` of the instruction that refused, as Mollusk
/// gives it; None for a refusal no program wrote (a budget, a size, a lock, rent, a fee), which Mollusk gives none of.
fn refusal_of(err: &Json) -> Option<ProgramError> {
    let e = err.get("InstructionError").at(1);
    if let Some(n) = e.get("Custom").as_u64() {
        return u32::try_from(n).ok().map(ProgramError::Custom);
    }
    let name = match e {
        Json::Str(s) => s.as_str(),
        Json::Obj(o) if o.len() == 1 => o[0].0.as_str(),
        _ => return None,
    };
    [
        ProgramError::InvalidArgument,
        ProgramError::InvalidInstructionData,
        ProgramError::InvalidAccountData,
        ProgramError::AccountDataTooSmall,
        ProgramError::InsufficientFunds,
        ProgramError::IncorrectProgramId,
        ProgramError::MissingRequiredSignature,
        ProgramError::AccountAlreadyInitialized,
        ProgramError::UninitializedAccount,
        ProgramError::NotEnoughAccountKeys,
        ProgramError::AccountBorrowFailed,
        ProgramError::MaxSeedLengthExceeded,
        ProgramError::InvalidSeeds,
        ProgramError::BorshIoError,
        ProgramError::AccountNotRentExempt,
        ProgramError::UnsupportedSysvar,
        ProgramError::IllegalOwner,
        ProgramError::MaxAccountsDataAllocationsExceeded,
        ProgramError::InvalidRealloc,
        ProgramError::MaxInstructionTraceLengthExceeded,
        ProgramError::BuiltinProgramsMustConsumeComputeUnits,
        ProgramError::InvalidAccountOwner,
        ProgramError::ArithmeticOverflow,
        ProgramError::Immutable,
        ProgramError::IncorrectAuthority,
    ]
    .into_iter()
    .find(|p| format!("{p:?}") == name)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn pause() {
    std::thread::sleep(Duration::from_millis(100));
}

/// A world on a real validator on this machine. Every step is a real transaction, simulated first (the refusal and the units before
/// anything lands), then sent and confirmed. Every key it signs with is made in memory for this run from the system's randomness, funded by
/// the validator's own faucet, never written anywhere, and gone with the process.
struct Live {
    rpc: Loopback,
    keys: BTreeMap<Pubkey, SigningKey>,
    /// The run's own key: it pays for a transaction whose instruction no writable key signs (a carry, a settle, a read-only signer's).
    bank: Pubkey,
    landed: Vec<Landed>,
    refusals: Vec<Refusal>,
    /// The first signature of every transaction sent: the same message twice waits for the next blockhash and never lands as a duplicate.
    sent: BTreeSet<[u8; 64]>,
    /// The faucet's airdrops: how many, and their lamports.
    airdrops: (u64, u64),
}

/// A transaction the validator confirmed: its step, its signature and slot, its units and the simulation's, its bytes.
struct Landed {
    what: String,
    signature: String,
    slot: u64,
    units: u64,
    simulated: u64,
    bytes: usize,
}

/// A program's refusal as the validator gave it: by the simulation (nothing landed), or by a transaction that landed and failed.
struct Refusal {
    what: String,
    error: ProgramError,
    said: String,
    units: u64,
    bytes: usize,
    landed: Option<String>,
}

impl Live {
    /// The validator at `url` (refused unless it is on loopback), and the run's own key, funded by its faucet.
    fn connect(url: &str) -> Self {
        let rpc = Loopback::parse(url).unwrap_or_else(|e| panic!("{e}"));
        let mut live = Self { rpc, keys: BTreeMap::new(), bank: Pubkey::default(), landed: Vec::new(), refusals: Vec::new(), sent: BTreeSet::new(), airdrops: (0, 0) };
        live.bank = live.wallet(10 * SOL);
        live
    }
    /// The validator at `url`, the run's own key a given one (THE LAUNCH PRESS's rehearsal: the same stand-in in each of its processes, so the
    /// dollar one makes the next can mint), topped up by the faucet.
    fn connect_as(url: &str, bank: SigningKey) -> Self {
        let rpc = Loopback::parse(url).unwrap_or_else(|e| panic!("{e}"));
        let mut live = Self { rpc, keys: BTreeMap::new(), bank: Pubkey::default(), landed: Vec::new(), refusals: Vec::new(), sent: BTreeSet::new(), airdrops: (0, 0) };
        live.bank = live.adopt(bank, 10 * SOL);
        live
    }
    /// A given key kept for this run, topped up by the faucet to `lamports` when it holds less.
    fn adopt(&mut self, sk: SigningKey, lamports: u64) -> Pubkey {
        let key = Pubkey::new_from_array(sk.verifying_key().to_bytes());
        self.keys.insert(key, sk);
        let has = self.account(&key).get("lamports").as_u64().unwrap_or_default();
        if has < lamports {
            self.airdrop(&key, lamports - has);
        }
        key
    }
    /// A transaction of a prepared set, by the Keystone page's own road (`to_sign`): a fresh blockhash stamped at its place, signed by its
    /// one signer from this run's keys, simulated, sent and confirmed.
    fn sign_prepared(&mut self, set: &PreparedSet, tx: &PreparedTx, what: &str) -> Result<u64, ProgramError> {
        let mut blockhash = self.blockhash();
        loop {
            let message = to_sign(set, tx, &tx.signer, &base58(&blockhash)).unwrap_or_else(|e| panic!("{what}: the page's road refused it: {e}"));
            match self.land(&message, &[tx.signer], what)? {
                Some(units) => return Ok(units),
                None => blockhash = self.next_blockhash(&blockhash),
            }
        }
    }
    fn ask(&self, method: &str, params: &str) -> Json {
        self.rpc.call(method, params).unwrap_or_else(|e| panic!("the validator: {e}"))
    }
    /// A new key of this run: 32 bytes of the system's randomness, in memory only.
    fn fresh(&mut self) -> Pubkey {
        let mut seed = [0u8; 32];
        std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut seed)).expect("the system's randomness");
        let sk = SigningKey::from_bytes(&seed);
        seed.fill(0);
        let key = Pubkey::new_from_array(sk.verifying_key().to_bytes());
        self.keys.insert(key, sk);
        key
    }
    /// `lamports` from the validator's own faucet, confirmed.
    fn airdrop(&mut self, to: &Pubkey, lamports: u64) {
        let signature = self.ask("requestAirdrop", &format!(r#"["{to}",{lamports}]"#)).as_str().expect("an airdrop's signature").to_string();
        let (_, err) = self.confirm(&signature, None, "an airdrop");
        assert!(err.is_null(), "the faucet's airdrop failed: {}", err.text());
        self.airdrops = (self.airdrops.0 + 1, self.airdrops.1 + lamports);
    }
    fn blockhash(&self) -> [u8; 32] {
        let b = self.ask("getLatestBlockhash", r#"[{"commitment":"confirmed"}]"#);
        // a blockhash is 32 bytes in base58, as a key is
        Pubkey::from_str(b.get("value").get("blockhash").as_str().expect("a blockhash")).expect("a blockhash of 32 bytes").to_bytes()
    }
    fn next_blockhash(&self, old: &[u8; 32]) -> [u8; 32] {
        let bound = Instant::now() + Duration::from_secs(60);
        loop {
            let b = self.blockhash();
            if &b != old {
                return b;
            }
            assert!(Instant::now() < bound, "the validator's blockhash stood still for a minute");
            pause();
        }
    }
    /// An account as the RPC answers it at the newest slot (null when there is none).
    fn account(&self, key: &Pubkey) -> Json {
        self.ask("getAccountInfo", &format!(r#"["{key}",{{"encoding":"base64","commitment":"processed"}}]"#)).take("value")
    }
    fn bytes_of(account: &Json) -> Vec<u8> {
        account.get("data").at(0).as_str().map(|b| base64::engine::general_purpose::STANDARD.decode(b).expect("an account's base64")).unwrap_or_default()
    }
    /// The runtime's features on this chain, as the feature program holds them (an account a feature, its data `activated_at`: 0, or 1 and
    /// the slot): each id in base58 and whether it is active.
    fn features(&self) -> Vec<(String, bool)> {
        let all = self.ask("getProgramAccounts", r#"["Feature111111111111111111111111111111111111",{"encoding":"base64","commitment":"processed"}]"#);
        let Json::Arr(all) = all else { panic!("the feature program's accounts: {}", all.text()) };
        let mut features: Vec<(String, bool)> = all
            .iter()
            .map(|a| (a.get("pubkey").as_str().expect("a feature's id").to_string(), Self::bytes_of(a.get("account")).first() == Some(&1)))
            .collect();
        features.sort();
        features
    }
    /// A program's loader and the bytes the validator runs for it (an upgradeable program's are in its program data, after the header).
    fn program(&self, id: &Pubkey) -> (String, Vec<u8>) {
        let a = self.account(id);
        assert_eq!(a.get("executable"), &Json::Bool(true), "{id}: a program on the validator");
        let loader = a.get("owner").as_str().expect("an owner").to_string();
        let data = Self::bytes_of(&a);
        if loader != BPF_LOADER_UPGRADEABLE_ID.to_string() {
            return (loader, data);
        }
        assert_eq!(data[..4], [2, 0, 0, 0], "{id}: an upgradeable program names its program data");
        let d = Self::bytes_of(&self.account(&Pubkey::new_from_array(data[4..36].try_into().unwrap())));
        assert_eq!(d[..4], [3, 0, 0, 0], "{id}: its program data");
        // the header: its tag 4, its slot 8, an optional authority 1 + 32
        (loader, d[45..].to_vec())
    }
    /// A signature polled until the validator confirms it, or it landed and failed: its slot and its error. A transaction the validator has
    /// not seen is sent again every two seconds (the same bytes: it keeps one).
    fn confirm(&self, signature: &str, again: Option<&str>, what: &str) -> (u64, Json) {
        let (begun, mut sent) = (Instant::now(), Instant::now());
        loop {
            let r = self.ask("getSignatureStatuses", &format!(r#"[["{signature}"]]"#));
            let s = r.get("value").at(0);
            if !s.is_null() {
                let level = s.get("confirmationStatus").as_str().unwrap_or_default();
                if !s.get("err").is_null() || level == "confirmed" || level == "finalized" {
                    return (s.get("slot").as_u64().unwrap_or_default(), s.get("err").clone());
                }
            } else if let Some(b64) = again.filter(|_| sent.elapsed() > Duration::from_secs(2)) {
                let _ = self.rpc.call("sendTransaction", &format!(r#"["{b64}",{{"encoding":"base64","skipPreflight":true}}]"#));
                sent = Instant::now();
            }
            assert!(begun.elapsed() < Duration::from_secs(90), "{what}: the validator never confirmed {signature}");
            pause();
        }
    }
    /// The units the validator counted for a confirmed transaction and its logs (None and null if it never says).
    fn landed_units(&self, signature: &str) -> (Option<u64>, Json) {
        for _ in 0..50 {
            let t = self.rpc.call("getTransaction", &format!(r#"["{signature}",{{"encoding":"base64","commitment":"confirmed","maxSupportedTransactionVersion":0}}]"#));
            if let Ok(meta) = t.map(|t| t.take("meta")) {
                if !meta.is_null() {
                    return (meta.get("computeUnitsConsumed").as_u64(), meta.get("logMessages").clone());
                }
            }
            pause();
        }
        (None, Json::Null)
    }
    /// One transaction: simulated, then sent and confirmed. Its units, or the program's refusal; a refusal no program wrote (the size, a
    /// budget, a lock, rent) ends the run with the validator's own words.
    fn send(&mut self, ixs: &[Instruction], payer: Pubkey, what: &str) -> Result<u64, ProgramError> {
        let mut blockhash = self.blockhash();
        loop {
            let (message, signers) = message_of(ixs, &payer, &blockhash);
            match self.land(&message, &signers, what)? {
                Some(units) => return Ok(units),
                None => blockhash = self.next_blockhash(&blockhash),
            }
        }
    }
    /// One compiled message, signed by its signers from this run's keys, simulated, sent and confirmed: its units; None when its first
    /// signature was sent before (the same message twice waits for the next blockhash and never lands as a duplicate); or the program's
    /// refusal.
    fn land(&mut self, message: &[u8], signers: &[Pubkey], what: &str) -> Result<Option<u64>, ProgramError> {
        {
            // each signer signs the message with its key, as a Solana transaction asks; Poleis's ballots stay owed Falcon-512 in a program (the house's lane 8)
            let signatures: Vec<[u8; 64]> = signers
                .iter()
                .map(|k| self.keys.get(k).unwrap_or_else(|| panic!("{what}: {k} signs it, and this run holds no key for it")).sign(message).to_bytes())
                .collect();
            if self.sent.contains(&signatures[0]) {
                return Ok(None);
            }
            let wire = transaction_of(message, &signatures);
            let b64 = base64::engine::general_purpose::STANDARD.encode(&wire);
            let sim = self
                .rpc
                .call("simulateTransaction", &format!(r#"["{b64}",{{"encoding":"base64","sigVerify":true,"commitment":"processed"}}]"#))
                .unwrap_or_else(|e| panic!("{what}: THE VALIDATOR REFUSED IT, and no program did ({} bytes): {e}", wire.len()));
            let v = sim.get("value");
            let simulated = v.get("unitsConsumed").as_u64().unwrap_or_default();
            if !v.get("err").is_null() {
                return Err(self.refusal(what, v.get("err"), v.get("logs"), simulated, wire.len(), None));
            }
            self.sent.insert(signatures[0]);
            let signature = base58(&signatures[0]);
            let said = self.ask("sendTransaction", &format!(r#"["{b64}",{{"encoding":"base64","skipPreflight":true}}]"#));
            assert_eq!(said.as_str(), Some(signature.as_str()), "{what}: the validator names another signature");
            let (slot, err) = self.confirm(&signature, Some(&b64), what);
            let (units, logs) = self.landed_units(&signature);
            if !err.is_null() {
                return Err(self.refusal(what, &err, &logs, units.unwrap_or(simulated), wire.len(), Some(signature)));
            }
            let units = units.unwrap_or(simulated);
            eprintln!("    slot {slot}: {what}, {units} CU, {} bytes", wire.len());
            self.landed.push(Landed { what: what.to_string(), signature, slot, units, simulated, bytes: wire.len() });
            Ok(Some(units))
        }
    }
    /// A refusal the validator gave: a program's, recorded and handed to the body as the `ProgramError` it expects; or one no program wrote,
    /// which ends the run with the validator's words.
    fn refusal(&mut self, what: &str, err: &Json, logs: &Json, units: u64, bytes: usize, landed: Option<String>) -> ProgramError {
        let words: Vec<&str> = match logs {
            Json::Arr(a) => a.iter().filter_map(Json::as_str).collect(),
            _ => Vec::new(),
        };
        let tail = words[words.len().saturating_sub(8)..].join("\n      ");
        let Some(error) = refusal_of(err) else {
            panic!("{what}: THE VALIDATOR REFUSED IT, and no program did ({bytes} bytes, {units} CU): {}\n      {tail}", err.text());
        };
        eprintln!("    refused: {what}: {error:?}{}", landed.as_ref().map(|s| format!(", landed and failed ({s})")).unwrap_or_default());
        self.refusals.push(Refusal { what: what.to_string(), error: error.clone(), said: err.text(), units, bytes, landed });
        error
    }
}

impl Ledger for Live {
    fn attempt(&mut self, ix: &Instruction, what: &str) -> Result<u64, ProgramError> {
        // its first writable signer pays for it, as a client pays for its own; else the run's own key (no flag of the instruction moves)
        let payer = ix.accounts.iter().find(|m| m.is_signer && m.is_writable).map(|m| m.pubkey).unwrap_or(self.bank);
        self.send(std::slice::from_ref(ix), payer, what)
    }
    fn attempt_all(&mut self, ixs: &[Instruction], payer: &Pubkey, what: &str) -> Result<u64, ProgramError> {
        self.send(ixs, *payer, what)
    }
    fn data(&self, key: &Pubkey) -> Vec<u8> {
        Self::bytes_of(&self.account(key))
    }
    fn lamports(&self, key: &Pubkey) -> u64 {
        self.account(key).get("lamports").as_u64().unwrap_or_default()
    }
    fn slot(&self) -> u64 {
        self.ask("getSlot", r#"[{"commitment":"processed"}]"#).as_u64().expect("a slot")
    }
    fn wait_until(&mut self, slot: u64) {
        let mut now = self.slot();
        if now >= slot {
            return;
        }
        eprintln!("    waiting for slot {slot} (now {now})");
        // a slot is 400 ms on a validator, and slower on a Mac at a heavy load (on Nov 23 2026 a second and more a slot at a load of 140 to
        // 410 on 18 cores: four rehearsals of the launch died here on a guard of a second a slot while their validators still moved). The
        // wait follows the validator's own pace; a clock that stood still is one that made no slot in a minute, as its blockhash's guard says
        let mut moved = Instant::now();
        while now < slot {
            pause();
            let s = self.slot();
            if s > now {
                (now, moved) = (s, Instant::now());
            }
            assert!(moved.elapsed() < Duration::from_secs(60), "the validator's clock stood still before slot {slot}: no slot past {now} in a minute");
        }
    }
    fn wallet(&mut self, lamports: u64) -> Pubkey {
        let key = self.fresh();
        self.airdrop(&key, lamports);
        key
    }
    fn voter(&mut self, lamports: u64) -> Voter {
        let key = self.wallet(lamports);
        Voter { sk: self.keys[&key].clone(), key }
    }
    /// A real mint by the token program: created, initialized with no decimals, and its minting authority (the run's own key, for the one
    /// instruction after) dropped to none, no freeze authority ever; read back, it holds what Mollusk's world lays by hand.
    fn mint(&mut self) -> Pubkey {
        let (mint, bank) = (self.fresh(), self.bank);
        let rent = self.ask("getMinimumBalanceForRentExemption", "[82]").as_u64().expect("a mint's rent");
        // the system program's CreateAccount (0): the lamports, 82 bytes, the token program its owner
        let create = [&[0u8, 0, 0, 0][..], &rent.to_le_bytes(), &82u64.to_le_bytes(), TOKEN_PROGRAM_ID.as_ref()].concat();
        // the token program's InitializeMint2 (20): no decimals, its authority, no freeze authority; then SetAuthority (6) of minting (0) to none
        let init = [&[20u8, 0][..], bank.as_ref(), &[0]].concat();
        let ixs = [
            Instruction::new_with_bytes(Pubkey::default(), &create, vec![AccountMeta::new(bank, true), AccountMeta::new(mint, true)]),
            Instruction::new_with_bytes(TOKEN_PROGRAM_ID, &init, vec![AccountMeta::new(mint, false)]),
            Instruction::new_with_bytes(TOKEN_PROGRAM_ID, &[6, 0, 0], vec![AccountMeta::new(mint, false), AccountMeta::new_readonly(bank, true)]),
        ];
        self.send(&ixs, bank, "the default mint").unwrap_or_else(|e| panic!("the default mint: refused with {e:?}"));
        let d = self.data(&mint);
        // no minting authority (its option's tag), no supply, no decimals, initialized, no freeze authority (its tag)
        assert_eq!((d.len(), &d[..4], &d[36..45], d[45], &d[46..50]), (82, &[0u8; 4][..], &[0u8; 9][..], 1, &[0u8; 4][..]), "the mint Mollusk's world lays");
        mint
    }
    fn roads(&mut self) -> Real {
        let id = Real::declared();
        assert_eq!(self.account(&id).get("executable"), &Json::Bool(true), "the roads program stands at {id} from the validator's genesis");
        Real { id }
    }
    /// The warm program at genesis (the script's `--upgradeable-program`), its program data naming the polis's own hand as its upgrade
    /// authority, and the associated token program beside it.
    fn warm(&mut self) -> Warm {
        let warm = Warm { id: Warm::declared() };
        assert_eq!(self.account(&warm.id).get("executable"), &Json::Bool(true), "the warm program stands at {} from the validator's genesis", warm.id);
        let pd = self.data(&warm.program_data());
        assert_eq!((&pd[..4], pd[12], &pd[13..45]), (&[3u8, 0, 0, 0][..], 1, agent_pda(ROOT_UNIT).as_ref()), "its upgrade authority is the polis's own hand");
        assert_eq!(self.account(&ata_program()).get("executable"), &Json::Bool(true), "the associated token program stands at genesis");
        warm
    }
    /// A real mint by the token program: six decimals, the run's own key its minting authority (it mints each flask's dollars, and dies
    /// with the run), no freeze authority.
    fn dollar(&mut self) -> Pubkey {
        let (mint, bank) = (self.fresh(), self.bank);
        let rent = self.ask("getMinimumBalanceForRentExemption", "[82]").as_u64().expect("a mint's rent");
        let create = [&[0u8, 0, 0, 0][..], &rent.to_le_bytes(), &82u64.to_le_bytes(), TOKEN_PROGRAM_ID.as_ref()].concat();
        // InitializeMint2 (20): six decimals, its minting authority, no freeze authority
        let init = [&[20u8, 6][..], bank.as_ref(), &[0]].concat();
        let ixs = [
            Instruction::new_with_bytes(Pubkey::default(), &create, vec![AccountMeta::new(bank, true), AccountMeta::new(mint, true)]),
            Instruction::new_with_bytes(TOKEN_PROGRAM_ID, &init, vec![AccountMeta::new(mint, false)]),
        ];
        self.send(&ixs, bank, "the world's dollar").unwrap_or_else(|e| panic!("the world's dollar: refused with {e:?}"));
        let d = self.data(&mint);
        assert_eq!((d.len(), &d[..4], &d[4..36], d[44], d[45], &d[46..50]), (82, &[1u8, 0, 0, 0][..], bank.as_ref(), 6, 1, &[0u8; 4][..]), "a dollar of six decimals");
        mint
    }
    /// The associated token program's CreateIdempotent (1) and the token program's MintTo (7), in one transaction the run's key pays and
    /// signs as the mint's authority.
    fn pay(&mut self, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Pubkey {
        let (bank, account) = (self.bank, associated(owner, mint));
        let create = vec![
            AccountMeta::new(bank, true),
            AccountMeta::new(account, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(Pubkey::default(), false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ];
        let mint_to = vec![AccountMeta::new(*mint, false), AccountMeta::new(account, false), AccountMeta::new_readonly(bank, true)];
        let ixs = [
            Instruction::new_with_bytes(ata_program(), &[1], create),
            Instruction::new_with_bytes(TOKEN_PROGRAM_ID, &[&[7u8][..], &amount.to_le_bytes()].concat(), mint_to),
        ];
        self.send(&ixs, bank, "the world's dollars to a flask").unwrap_or_else(|e| panic!("the world's dollars: refused with {e:?}"));
        account
    }
    /// The Clock sysvar at the newest slot: [slot:8][epoch_start_timestamp:8][epoch:8][leader_schedule_epoch:8][unix_timestamp:8].
    fn clock_ms(&self) -> u64 {
        let d = self.data(&Pubkey::from_str("SysvarC1ock11111111111111111111111111111111").unwrap());
        let s = i64::from_le_bytes(d[32..40].try_into().expect("the clock's unix_timestamp"));
        u64::try_from(s).expect("a clock after the epoch") * 1_000
    }
}

#[test]
#[ignore = "it needs a validator: poleis/ceremony/the-market-on-a-local-chain.sh starts one on loopback, runs this, and stops it"]
fn the_two_programs_on_a_live_validator() {
    let url = std::env::var("POLEIS_VALIDATOR").expect("POLEIS_VALIDATOR: the local validator's RPC, http://127.0.0.1:<port>");
    let out = std::env::var("POLEIS_CERTIFICATE").expect("POLEIS_CERTIFICATE: the path the certificate is written to");
    // the same chain on Mollusk first: its units stand beside the validator's in the certificate
    eprintln!("  on Mollusk:");
    let (mollusk, on_mollusk) = two_programs(estate(false));
    eprintln!("  on the validator at {url}:");
    let begun = Instant::now();
    let live = Live::connect(&url);
    let first = live.slot();
    let (live, book) = two_programs(founded(live, false));
    let (last, wall) = (live.slot(), begun.elapsed().as_secs());

    // the bytes it ran, read back from the validator and held to this tree's build
    let roads_built = std::fs::read(format!("{}/poleis_roads.so", built::roads_program_dir())).expect("the built roads program");
    let warm_built = std::fs::read(format!("{}/warm_mine.so", built::warm_program_dir())).expect("the built warm program");
    let mut programs = Vec::new();
    for (name, id, built) in [("poleis", PROGRAM_ID, built::program_elf()), ("roads", Real::declared(), roads_built), ("warm", Warm::declared(), warm_built)] {
        let (loader, ran) = live.program(&id);
        let padding = ran.get(built.len()..).unwrap_or_default();
        assert!(ran.starts_with(&built) && padding.iter().all(|b| *b == 0), "{name}: the validator runs this tree's bytes");
        let what = Json::object(vec![("id", Json::string(id.to_string())), ("loader", Json::string(loader)), ("bytes", Json::number(built.len())), ("sha256", Json::string(sha256_hex(&built)))]);
        programs.push((name, what));
    }

    // each step's units on Mollusk beside the validator's: the k-th step of a name beside Mollusk's k-th of that name
    let (mut seen, mut equal, mut differ): (BTreeMap<&str, usize>, usize, usize) = (BTreeMap::new(), 0, 0);
    let steps: Vec<Json> = live
        .landed
        .iter()
        .enumerate()
        .map(|(n, s)| {
            let k = seen.entry(s.what.as_str()).or_default();
            let paired = mollusk.landed.iter().filter(|(w, _)| *w == s.what).nth(*k).map(|(_, u)| *u);
            *k += 1;
            match paired {
                Some(u) if u == s.units => equal += 1,
                Some(_) => differ += 1,
                None => {}
            }
            let on_mollusk = paired.map(Json::number).unwrap_or(Json::Null);
            Json::object(vec![
                ("n", Json::number(n + 1)),
                ("what", Json::string(&s.what)),
                ("slot", Json::number(s.slot)),
                ("units", Json::number(s.units)),
                ("mollusk", on_mollusk),
                ("simulated", Json::number(s.simulated)),
                ("bytes", Json::number(s.bytes)),
                ("signature", Json::string(&s.signature)),
            ])
        })
        .collect();
    let refusals: Vec<Json> = live
        .refusals
        .iter()
        .map(|r| {
            Json::object(vec![
                ("what", Json::string(&r.what)),
                ("error", Json::string(format!("{:?}", r.error))),
                ("validator", Json::string(&r.said)),
                ("units", Json::number(r.units)),
                ("bytes", Json::number(r.bytes)),
                ("landed", r.landed.as_ref().map(Json::string).unwrap_or(Json::Null)),
            ])
        })
        .collect();
    let (n, largest) = live.landed.iter().enumerate().max_by_key(|(_, s)| s.bytes).expect("a step landed");
    assert!(largest.bytes <= PACKET, "no transaction past {PACKET} bytes lands");
    let (d, dearest) = live.landed.iter().enumerate().max_by_key(|(_, s)| s.units).expect("a step landed");
    let units = Json::object(vec![
        ("equal", Json::number(equal)),
        ("differ", Json::number(differ)),
        ("notOnMollusk", Json::number(live.landed.len() - equal - differ)),
        (
            "why",
            Json::string(
                "a step's units on Mollusk are those of the k-th step of the same name in the same body, run on Mollusk just before. A difference of a \
                 multiple of about 1,500 is one address search more or fewer (find_program_address), since this run's keys are not Mollusk's and its \
                 proposals' hashes name them; a difference of a unit is the market's own arithmetic on a road its trades reached a slot later",
            ),
        ),
        ("dearest", Json::object(vec![("n", Json::number(d + 1)), ("what", Json::string(&dearest.what)), ("units", Json::number(dearest.units))])),
        ("budget", Json::string("no transaction asks a compute budget of its own: each instruction runs within the validator's default")),
    ]);
    let book_of = |b: &Book| {
        let m = &b.mine;
        let mine = Json::object(vec![
            ("opening", Json::number(m.opening)),
            ("halving", Json::number(m.halving)),
            ("spends", Json::number(m.spends)),
            ("depositedMicros", Json::number(m.deposited)),
            ("spentMicros", Json::number(m.spent)),
            ("issuedNanos", Json::number(m.issued)),
            ("mintedNanos", Json::number(m.minted)),
            ("fromVaultNanos", Json::number(m.from_vault)),
            ("paidBackNanos", Json::number(m.paid_back)),
            ("vaultNanos", Json::number(m.vault)),
            ("supplyNanos", Json::number(m.supply)),
            ("lawNanos", Json::number(m.law)),
            ("treasuryMicros", Json::number(m.treasury)),
        ]);
        Json::object(vec![
            ("verdicts", Json::Arr(b.verdicts.iter().map(Json::number).collect())),
            ("paid", Json::Arr(b.paid.iter().map(Json::number).collect())),
            ("stakesBack", Json::Arr(b.backs.iter().map(Json::number).collect())),
            ("houseEarned", Json::number(b.earned)),
            ("book", Json::number(b.sum)),
            ("credited", Json::number(b.credited)),
            ("mine", mine),
        ])
    };
    let version = live.ask("getVersion", "[]");
    let said = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty()).map(Json::string).unwrap_or(Json::Null);
    // the rules it ran under: the cluster whose feature set the validator cloned at genesis (the script's word), and the features the chain
    // holds active, read here over loopback
    let features = live.features();
    let active: Vec<&str> = features.iter().filter(|(_, on)| *on).map(|(id, _)| id.as_str()).collect();
    let rules = Json::object(vec![
        ("source", said("POLEIS_FEATURES")),
        ("cluster", said("POLEIS_FEATURES_FROM")),
        ("accounts", Json::number(features.len())),
        ("active", Json::number(active.len())),
        ("sha256", Json::string(sha256_hex(active.iter().map(|id| format!("{id}\n")).collect::<String>().as_bytes()))),
        ("sha256Of", Json::string("the active features' ids in base58, sorted, each followed by a newline")),
        ("ids", Json::Arr(active.iter().map(|id| Json::string(*id)).collect())),
    ]);
    eprintln!("  the rules: {} active features of {} (from {}), sha256 {}", active.len(), features.len(), rules.get("source").text(), rules.get("sha256").text());
    let certificate = Json::object(vec![
        ("what", Json::string("THE MARKET ON A LOCAL CHAIN: THE TWO PROGRAMS IN ONE WORLD on a real validator, from a board opened for the polis's own hand to a budget paid on a road's verdict and the house's collect; then the third, $WARM mined by use, from the mine the polis opens to a day of spends across a halving, the bound, and the $WARM paid back and emitted again")),
        ("startedAt", said("POLEIS_STARTED_AT")),
        ("house", said("POLEIS_COMMIT")),
        ("validator", Json::object(vec![("solanaCore", version.get("solana-core").clone()), ("featureSet", version.get("feature-set").clone()), ("genesisHash", live.ask("getGenesisHash", "[]")), ("rpc", Json::string(&url))])),
        ("features", rules),
        ("programs", Json::object(programs)),
        ("keys", Json::object(vec![("made", Json::number(live.keys.len())), ("airdrops", Json::number(live.airdrops.0)), ("lamports", Json::number(live.airdrops.1)), ("kept", Json::string("in memory for this run only, from the system's randomness, never written; funded by the validator's own faucet"))])),
        ("slots", Json::object(vec![("first", Json::number(first)), ("last", Json::number(last))])),
        ("wallSeconds", Json::number(wall)),
        ("units", units),
        ("steps", Json::Arr(steps)),
        ("refusals", Json::Arr(refusals)),
        ("largest", Json::object(vec![("n", Json::number(n + 1)), ("what", Json::string(&largest.what)), ("bytes", Json::number(largest.bytes)), ("limit", Json::number(PACKET)), ("under", Json::number(PACKET - largest.bytes))])),
        ("chain", book_of(&book)),
        ("mollusk", book_of(&on_mollusk)),
    ]);
    std::fs::write(&out, certificate.pretty() + "\n").unwrap_or_else(|e| panic!("{out}: {e}"));
    eprintln!(
        "  THE TWO PROGRAMS ON A LIVE VALIDATOR: {} transactions and {} refusals in {wall} s, slots {first} to {last}; the largest {} bytes of {PACKET} ({}); the certificate {out}",
        live.landed.len(),
        live.refusals.len(),
        largest.bytes,
        largest.what
    );
}

/// Solana's own known legacy transaction, `test_sdk_serialize` in solana-transaction 4.1.5 (`src/lib.rs`): one instruction of (1, 2, 3) to
/// a program, its payer a signer and one account written, the default blockhash; its bytes as Solana serializes them, signature and all.
const SOLANAS_OWN: [u8; 206] = [
    1, 120, 138, 162, 185, 59, 209, 241, 157, 71, 157, 74, 131, 4, 87, 54, 28, 38, 180, 222, 82, 64, 62, 61, 62, 22, 46, 17, 203, 187, 136, 62, 43, 11, 38,
    235, 17, 239, 82, 240, 139, 130, 217, 227, 214, 9, 242, 141, 223, 94, 29, 184, 110, 62, 32, 87, 137, 63, 139, 100, 221, 20, 137, 4, 5, 1, 0, 1, 3, 36,
    100, 158, 252, 33, 161, 97, 185, 62, 89, 99, 195, 250, 249, 187, 189, 171, 118, 241, 90, 248, 14, 68, 219, 231, 62, 157, 5, 142, 27, 210, 117, 1, 1, 1,
    4, 5, 6, 7, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 7, 6, 5, 4, 1, 1, 1, 2, 2, 2, 4, 5, 6, 7, 8, 9, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 9, 8, 7, 6, 5, 4, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 0, 1, 3, 1, 2,
    3,
];

#[test]
fn the_wire_is_solanas_own_legacy_transaction() {
    let payer = Pubkey::new_from_array(SOLANAS_OWN[69..101].try_into().unwrap());
    let to = Pubkey::new_from_array([1, 1, 1, 4, 5, 6, 7, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 7, 6, 5, 4, 1, 1, 1]);
    let program = Pubkey::new_from_array([2, 2, 2, 4, 5, 6, 7, 8, 9, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9, 8, 7, 6, 5, 4, 2, 2, 2]);
    let ix = Instruction::new_with_bytes(program, &[1, 2, 3], vec![AccountMeta::new(payer, true), AccountMeta::new(to, false)]);
    let (message, signers) = message_of(&[ix], &payer, &[0; 32]);
    assert_eq!(signers, vec![payer], "one signer, the payer");
    assert_eq!(message[..], SOLANAS_OWN[65..], "the message, byte for byte");
    // the signature Solana's test made, set where the wire sets it
    assert_eq!(transaction_of(&message, &[SOLANAS_OWN[1..65].try_into().unwrap()]), SOLANAS_OWN.to_vec(), "the transaction, byte for byte");
    let unsigned = transaction_of(&message, &[]);
    assert_eq!((unsigned[0], &unsigned[1..]), (0, &message[..]), "no signature is a count of zero, then the message");
}

#[test]
fn the_keys_are_compiled_in_solanas_order() {
    // the payer in no instruction; a key read in one instruction and written in the other (written); a read-only signer named twice;
    // a program that is also an account
    let k = |b: u8| Pubkey::new_from_array([b; 32]);
    let (payer, a, b, c, d, p, q) = (k(9), k(5), k(3), k(7), k(1), k(8), k(2));
    let ixs = [
        Instruction::new_with_bytes(p, &[1], vec![AccountMeta::new_readonly(a, false), AccountMeta::new_readonly(b, true), AccountMeta::new(c, true)]),
        Instruction::new_with_bytes(
            q,
            &[2, 2],
            vec![AccountMeta::new(a, false), AccountMeta::new_readonly(p, false), AccountMeta::new_readonly(d, false), AccountMeta::new_readonly(b, true)],
        ),
    ];
    let (m, signers) = message_of(&ixs, &payer, &[4; 32]);
    assert_eq!(signers, vec![payer, c, b], "the payer, the writable signer, the read-only signer");
    assert_eq!(m[..4], [3, 1, 3, 7], "three sign, one of them read-only, three read-only that do not sign; seven keys");
    let keys: Vec<Pubkey> = m[4..4 + 7 * 32].chunks(32).map(|x| Pubkey::new_from_array(x.try_into().unwrap())).collect();
    assert_eq!(keys, vec![payer, c, b, a, d, q, p], "each group in the keys' byte order");
    assert_eq!(m[4 + 7 * 32..4 + 8 * 32], [4; 32], "the blockhash");
    // two instructions: p (6) over a b c (3 2 1) with [1]; q (5) over a p d b (3 6 4 2) with [2, 2]
    assert_eq!(m[4 + 8 * 32..], [2, 6, 3, 3, 2, 1, 1, 1, 5, 4, 3, 6, 4, 2, 2, 2, 2]);
    for (n, want) in [(0usize, vec![0u8]), (127, vec![0x7f]), (128, vec![0x80, 0x01]), (16_383, vec![0xff, 0x7f]), (16_384, vec![0x80, 0x80, 0x01])] {
        let mut v = Vec::new();
        compact(n, &mut v);
        assert_eq!(v, want, "compact-u16 of {n}: seven bits a byte, the lowest first");
    }
}

#[test]
fn a_live_world_talks_to_loopback_only() {
    for (url, at) in [
        ("http://127.0.0.1:8899", "127.0.0.1:8899"),
        ("http://localhost:8899", "127.0.0.1:8899"),
        ("http://[::1]:8899", "[::1]:8899"),
        ("http://127.0.0.1:18899/", "127.0.0.1:18899"),
    ] {
        assert_eq!(Loopback::parse(url).unwrap_or_else(|e| panic!("{e}")).at, at.parse().unwrap(), "{url}");
    }
    for url in [
        "https://api.devnet.solana.com",
        "http://api.mainnet-beta.solana.com:80",
        "https://127.0.0.1:8899",
        "http://10.0.0.5:8899",
        "http://192.168.1.2:8899",
        "http://0.0.0.0:8899",
        "http://127.0.0.2:8899",
        "http://[::ffff:127.0.0.1]:8899",
        "http://localhost.:8899",
        "http://user@127.0.0.1:8899",
        "http://127.0.0.1:8899@api.devnet.solana.com:80",
        "http://127.0.0.1",
        "http://127.0.0.1:0",
        "http://127.0.0.1:99999",
        "ws://127.0.0.1:8900",
        "http://127.0.0.1:8899/\r\nHost: api.devnet.solana.com",
    ] {
        let e = Loopback::parse(url).expect_err(url);
        assert!(e.contains("only to a validator on this machine"), "{url}: {e}");
    }
}

#[test]
fn a_simulations_refusal_is_the_program_error_the_body_expects() {
    let reply = r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"4.2.0","slot":412},"value":{"accounts":null,"err":{"InstructionError":[1,{"Custom":14}]},"logs":["Program x invoke [1]","Program x failed: custom program error: 0xe"],"replacementBlockhash":null,"returnData":null,"unitsConsumed":7311}},"id":1}"#;
    let value = Json::parse(reply).unwrap().take("result").take("value");
    assert_eq!((refusal_of(value.get("err")), value.get("unitsConsumed").as_u64()), (Some(ProgramError::Custom(14)), Some(7311)));
    for (err, want) in [
        (r#"{"InstructionError":[0,"InvalidInstructionData"]}"#, Some(ProgramError::InvalidInstructionData)),
        (r#"{"InstructionError":[0,{"Custom":0}]}"#, Some(ProgramError::Custom(0))),
        (r#"{"InstructionError":[2,"MissingRequiredSignature"]}"#, Some(ProgramError::MissingRequiredSignature)),
        (r#"{"InstructionError":[0,{"BorshIoError":"x"}]}"#, Some(ProgramError::BorshIoError)),
        // what no program wrote: a budget, a number past a program's error, a transaction's own refusals
        (r#"{"InstructionError":[0,"ComputationalBudgetExceeded"]}"#, None),
        (r#"{"InstructionError":[0,{"Custom":4294967296}]}"#, None),
        (r#""AccountNotFound""#, None),
        (r#"{"InsufficientFundsForRent":{"account_index":2}}"#, None),
        (r#""BlockhashNotFound""#, None),
        ("null", None),
    ] {
        assert_eq!(refusal_of(&Json::parse(err).unwrap()), want, "{err}");
    }
}

#[test]
fn json_and_http_as_the_rpc_speaks_them() {
    let text = " {\"a\":[1,-2,3.5e2,true,false,null],\"b\":{\"c\":\"x\\\"y\\\\z\\n\\u00e9\\ud83d\\ude00\"},\"big\":18446744073709551615,\"e\":{},\"f\":[]} ";
    let v = Json::parse(text).unwrap();
    assert_eq!(v.get("big").as_u64(), Some(u64::MAX), "a u64 whole, never through a float");
    // u64::MAX survives a float by luck (a float of 2^64 saturates back to it); 2^53 + 1 and the largest odd u64 below 2^64 do not
    for n in [9_007_199_254_740_993u64, 18_446_744_073_709_551_613] {
        assert_eq!(Json::parse(&n.to_string()).unwrap().as_u64(), Some(n), "{n} whole, never through a float");
    }
    assert_eq!(v.get("b").get("c").as_str(), Some("x\"y\\z\n\u{e9}\u{1f600}"));
    assert_eq!((v.get("a").at(3), v.get("a").at(5), v.get("none")), (&Json::Bool(true), &Json::Null, &Json::Null));
    assert_eq!(Json::parse(&v.text()).unwrap(), v);
    assert_eq!(Json::parse(&v.pretty()).unwrap(), v);
    let long = Json::object(vec![("steps", Json::Arr((0..40).map(|n| Json::object(vec![("n", Json::number(n)), ("what", Json::string("a step"))])).collect()))]);
    assert_eq!(long.pretty().lines().count(), 44, "a short value keeps its line: a step a line");
    assert_eq!(Json::parse(&long.pretty()).unwrap(), long);
    for bad in ["", "{", "[1,]", "{\"a\" 1}", "\"open", "nul", "{} {}", "[1 2]", "\"\\q\"", "\"\\u12\""] {
        assert!(Json::parse(bad).is_err(), "{bad:?}");
    }
    // a reply by its length, a reply by its chunks, and the replies that are refused
    let body = r#"{"jsonrpc":"2.0","result":7,"id":1}"#;
    let by_length = format!("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}", body.len());
    assert_eq!(http_body(by_length.as_bytes()).unwrap(), body);
    let chunked = format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\na\r\n{}\r\n{:x}\r\n{}\r\n0\r\n\r\n", &body[..10], body.len() - 10, &body[10..]);
    assert_eq!(http_body(chunked.as_bytes()).unwrap(), body);
    assert!(http_body(b"HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\n\r\n").is_err());
    assert!(http_body(format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{body}", body.len() + 5).as_bytes()).is_err());
    assert!(http_body(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n").is_err());
}

#[test]
fn base58_is_the_addresses_own() {
    assert_eq!(base58(&[0; 32]), "11111111111111111111111111111111");
    let mut x = [0u8; 32];
    for (i, b) in x.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(37).wrapping_add(11);
    }
    let mut zeros = x;
    zeros[..3].fill(0);
    let mut one = [0u8; 32];
    one[31] = 1;
    for k in [x, zeros, one, [255; 32]] {
        assert_eq!(base58(&k), Pubkey::new_from_array(k).to_string());
    }
}

// ---- THE KEYSTONE PAGE'S PREPARED ROAD, IN RUST ----
// poleis/ceremony/keystone/prepared.mjs, every check of its own, for the stand-ins of THE LAUNCH PRESS's rehearsal to sign through the same
// bytes his Keystones would: `readSet` (a set names its cluster; holds a transaction; each transaction's message is the one the set describes,
// by its sha256; its blockhash is blank; one signer, the fee payer; the set signable only when it holds no placeholder), `toSign` (refused for a
// set that may not be signed and for another Keystone; the set's own bytes with a fresh blockhash at its place) and `signedWire` (one
// signature, then the message). Held to the page's own answers by `the_prepared_road_is_the_keystone_pages_own`, against a vector the page's
// own functions made once (poleis/press/tests/prepared-vector.json); no JavaScript runs here. THE SWARM IS ITS OWN MULTISIG: a set holds one
// transaction, Keystone 1's proposal, and the envelopes each agent and each Keystone signs as a message; press crank lands them. The page's road
// for an envelope is the page's to learn: a change to the page, his word first.

/// A prepared set as the page reads it.
#[derive(Debug)]
struct PreparedSet {
    cluster: String,
    signable: bool,
    placeholders: Vec<String>,
    txs: Vec<PreparedTx>,
    envelopes: Vec<PreparedEnvelope>,
    /// the act's proposal hash, when the set is an act of THE LAUNCH PRESS
    act: Option<[u8; 32]>,
}
#[derive(Debug)]
struct PreparedTx {
    n: u64,
    name: String,
    message: Vec<u8>,
    signer: Pubkey,
}
/// An envelope of a set: the message one voter signs as its ballot in one chamber (unit 1's chairs, or root 0's floor).
#[derive(Debug)]
struct PreparedEnvelope {
    signer: Pubkey,
    unit: u64,
    envelope: Vec<u8>,
}

/// A message's signer count, its keys, and where its blockhash sits: after the three-byte header, the compact key count and 32 bytes a key.
fn prepared_layout(message: &[u8]) -> Result<(usize, Vec<Pubkey>, usize), String> {
    let (mut n, mut shift, mut at) = (0usize, 0u32, 3usize);
    loop {
        let b = *message.get(at).ok_or("a message cut short")?;
        at += 1;
        n |= usize::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    let keys = (0..n).map(|i| message.get(at + 32 * i..at + 32 * (i + 1)).map(|k| Pubkey::new_from_array(k.try_into().unwrap())).ok_or("a key cut short")).collect::<Result<Vec<_>, _>>()?;
    Ok((usize::from(*message.first().ok_or("an empty message")?), keys, at + 32 * n))
}

/// The public clusters' genesis hashes, as the page holds them: mainnet's and devnet's.
const PUBLIC_GENESIS: [&str; 2] = ["5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d", "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG"];

/// What a Keystone shows of a message's instructions it has no words for (the page's `layout(...).shown`): each one's program and its data,
/// the compute budget's own left out (the Keystone reads those).
fn prepared_shown(message: &[u8]) -> Result<Vec<(Pubkey, Vec<u8>)>, String> {
    let (_, keys, blockhash_at) = prepared_layout(message)?;
    let mut at = blockhash_at + 32;
    let vec = |at: &mut usize| -> Result<usize, String> {
        let (mut n, mut shift) = (0usize, 0u32);
        loop {
            let b = *message.get(*at).ok_or("a message cut short")?;
            *at += 1;
            n |= usize::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(n);
            }
            shift += 7;
        }
    };
    let mut out = Vec::new();
    for _ in 0..vec(&mut at)? {
        let program = *keys.get(usize::from(*message.get(at).ok_or("a message cut short")?)).ok_or("an instruction of no program")?;
        at += 1;
        at += vec(&mut at)?;
        let n = vec(&mut at)?;
        let data = message.get(at..at + n).ok_or("a message cut short")?.to_vec();
        at += n;
        if program.to_string() != "ComputeBudget111111111111111111111111111111" {
            out.push((program, data));
        }
    }
    Ok(out)
}

fn unb64(s: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD.decode(s).map_err(|e| format!("base64: {e}"))
}

/// `readSet`, strictly: its cluster among `clusters` (the page's are mainnet and devnet; a rehearsal's set names the validator of its machine).
fn read_set(set: &Json, clusters: &[&str]) -> Result<PreparedSet, String> {
    let cluster = set.get("cluster").as_str().unwrap_or_default().to_string();
    if !clusters.contains(&cluster.as_str()) {
        return Err(format!("a prepared set names its cluster: {}", clusters.join(" or ")));
    }
    let Json::Arr(raw) = set.get("transactions") else { return Err("a prepared set holds no transaction".into()) };
    if raw.is_empty() {
        return Err("a prepared set holds no transaction".into());
    }
    // a set of a local chain states that chain's own genesis hash, never a public cluster's (the page's `genesisOf`)
    if cluster == "localnet" {
        let g = set.get("genesisHash").as_str().unwrap_or_default();
        if Pubkey::from_str(g).map_or(true, |k| k.to_string() != g) {
            return Err("a prepared set names its cluster: mainnet, devnet, or localnet with its genesis hash".into());
        }
        if PUBLIC_GENESIS.contains(&g) {
            return Err("a set of a local chain states a public cluster's genesis hash".into());
        }
    }
    let act: Option<[u8; 32]> = match set.get("act") {
        Json::Null => None,
        a => Some(a.get("hash").as_str().filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))).map(|h| (0..32).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap()).collect::<Vec<u8>>().try_into().unwrap()).ok_or("a set of an act names that act's hash")?),
    };
    let strings = |v: &Json| -> Vec<String> {
        match v {
            Json::Arr(a) => a.iter().filter_map(Json::as_str).map(str::to_string).collect(),
            _ => Vec::new(),
        }
    };
    let mut placeholders = strings(set.get("placeholders"));
    let mut txs = Vec::new();
    for t in raw {
        let name = t.get("name").as_str().unwrap_or_default().to_string();
        let message = unb64(t.get("message").as_str().unwrap_or_default())?;
        let (signers, keys, at) = prepared_layout(&message)?;
        if message.get(at..at + 32).ok_or("a message cut short")?.iter().any(|b| *b != 0) {
            return Err(format!("{name}: its blockhash is not blank"));
        }
        if Some(sha256_hex(&message).as_str()) != t.get("messageSha256").as_str() {
            return Err(format!("{name}: its message is not the one the set describes"));
        }
        // a set of an act carries that act's hash in the bytes its signer is shown (the page's `layout(...).shown`)
        if act.is_some_and(|h| !prepared_shown(&message).is_ok_and(|shown| shown.iter().any(|(_, d)| d.windows(32).any(|w| w == h)))) {
            return Err(format!("{name}: its bytes do not carry the hash of act {}", set.get("act").get("n").as_u64().unwrap_or_default()));
        }
        let listed = strings(t.get("signers"));
        if signers != 1 || listed.len() != 1 || keys[0].to_string() != listed[0] {
            return Err(format!("{name}: one signer, the fee payer, is all this page signs for"));
        }
        placeholders.extend(strings(t.get("placeholders")));
        txs.push(PreparedTx { n: t.get("n").as_u64().unwrap_or_default(), name, message, signer: keys[0] });
    }
    let mut envelopes = Vec::new();
    if let Json::Arr(raw) = set.get("envelopes") {
        for e in raw {
            let signer = Pubkey::from_str(e.get("signer").as_str().unwrap_or_default()).map_err(|_| "an envelope names no signer")?;
            let unit = e.get("unit").as_u64().ok_or("an envelope names no unit")?;
            let envelope = unb64(e.get("envelope").as_str().unwrap_or_default())?;
            if act.is_some_and(|h| envelope != vote_envelope(&h, unit, VOTE_PRO)) {
                return Err("an envelope is not this act's PRO in its chamber".into());
            }
            envelopes.push(PreparedEnvelope { signer, unit, envelope });
        }
    }
    placeholders.sort();
    placeholders.dedup();
    Ok(PreparedSet { cluster, signable: set.get("signable") == &Json::Bool(true) && placeholders.is_empty(), placeholders, txs, envelopes, act })
}

/// `toSign`: the set's own bytes with a fresh blockhash in its place; refused for a set that may not be signed, or another Keystone.
fn to_sign(set: &PreparedSet, tx: &PreparedTx, keystone: &Pubkey, blockhash: &str) -> Result<Vec<u8>, String> {
    if !set.signable {
        return Err(format!("this set may not be signed: it still holds placeholders ({})", set.placeholders.join(", ")));
    }
    if tx.signer != *keystone {
        return Err(format!("this transaction is signed by {}, not by the Keystone {keystone}", tx.signer));
    }
    let mut out = tx.message.clone();
    let (_, _, at) = prepared_layout(&out)?;
    out[at..at + 32].copy_from_slice(&Pubkey::from_str(blockhash).map_err(|_| "a blockhash of 32 bytes")?.to_bytes());
    Ok(out)
}

/// `signedWire`: one signature, then the message.
fn signed_wire(message: &[u8], signature: &[u8; 64]) -> Vec<u8> {
    [&[1u8][..], &signature[..], message].concat()
}

/// An act's set as `press acts` writes it (THE WHOLE TREE DECIDES: no transaction of anyone's in it): its hash, and unit 1's agents'
/// envelopes, each this act's PRO in unit 1, a message. An envelope of root 0's floor is refused: the floor votes by its owners' own
/// transactions.
fn read_act(set: &Json) -> Result<([u8; 32], Vec<PreparedEnvelope>), String> {
    if !set.get("transactions").is_null() {
        return Err("an act's set holds a transaction: its proposal and the floor's votes are the seat owners' own".into());
    }
    let h = set.get("act").get("hash").as_str().ok_or("an act's set names no hash")?;
    let bytes: Vec<u8> = (0..h.len() / 2).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16)).collect::<Result<_, _>>().map_err(|_| "an act's hash is not hex")?;
    let hash: [u8; 32] = bytes.try_into().map_err(|_| "an act's hash is not 32 bytes")?;
    let Json::Arr(raw) = set.get("envelopes") else { return Err("an act's set holds no envelope".into()) };
    let mut out = Vec::new();
    for e in raw {
        let signer = Pubkey::from_str(e.get("signer").as_str().unwrap_or_default()).map_err(|_| "an envelope names no signer")?;
        let unit = e.get("unit").as_u64().ok_or("an envelope names no unit")?;
        if unit != POSIVERSE {
            return Err(format!("an envelope of unit {unit}: an act's envelopes are unit 1's agents' (root 0's floor votes by its owners' own transactions)"));
        }
        let envelope = unb64(e.get("envelope").as_str().unwrap_or_default())?;
        if envelope != vote_envelope(&hash, unit, VOTE_PRO) {
            return Err("an envelope is not this act's PRO in unit 1".into());
        }
        out.push(PreparedEnvelope { signer, unit, envelope });
    }
    Ok((hash, out))
}

#[test]
fn the_prepared_road_is_the_keystone_pages_own() {
    let text = include_str!("../../../press/tests/prepared-vector.json");
    let v = Json::parse(text).unwrap();
    let set = read_set(v.get("set"), &["mainnet", "devnet"]).unwrap_or_else(|e| panic!("{e}"));
    let page = v.get("readSet");
    assert_eq!((set.cluster.as_str(), set.signable), (page.get("cluster").as_str().unwrap(), page.get("signable") == &Json::Bool(true)), "readSet's cluster and signable: the page's");
    let blockhash = v.get("blockhash").as_str().unwrap();
    assert!(!set.txs.is_empty() && set.txs.len() == match v.get("toSign") { Json::Arr(a) => a.len(), _ => 0 });
    for (i, tx) in set.txs.iter().enumerate() {
        assert_eq!(tx.signer.to_string(), page.get("signers").at(i).as_str().unwrap(), "{}: the signer readSet names", tx.name);
        let m = to_sign(&set, tx, &tx.signer, blockhash).unwrap();
        let b64 = |b: &[u8]| base64::engine::general_purpose::STANDARD.encode(b);
        assert_eq!(b64(&m), v.get("toSign").at(i).as_str().unwrap(), "{}: toSign, byte for byte the page's", tx.name);
        assert_eq!(b64(&signed_wire(&m, &[7; 64])), v.get("signedWire").at(i).as_str().unwrap(), "{}: signedWire, the page's", tx.name);
        assert!(to_sign(&set, tx, &Pubkey::new_from_array([3; 32]), blockhash).unwrap_err().contains("not by the Keystone"), "another Keystone is refused");
    }
    // the page's refusals, each on a copy of the set with one thing changed
    let one = |from: &str, to: &str| Json::parse(&v.get("set").text().replacen(from, to, 1)).unwrap();
    let held = one("\"placeholders\":[]", "\"placeholders\":[\"his word\"]");
    let r = read_set(&held, &["mainnet"]).unwrap();
    assert!(!r.signable && to_sign(&r, &r.txs[0], &r.txs[0].signer, blockhash).unwrap_err().contains("placeholders"), "a set with a placeholder is read and never signed");
    assert!(read_set(v.get("set"), &["devnet"]).is_err(), "a set of another cluster than the page's");
    let first = v.get("set").get("transactions").at(0).get("message").as_str().unwrap().to_string();
    let mut m = unb64(&first).unwrap();
    let last = m.len() - 1;
    m[last] ^= 1;
    let tampered = one(&first, &base64::engine::general_purpose::STANDARD.encode(&m));
    assert!(read_set(&tampered, &["mainnet"]).unwrap_err().contains("not the one the set describes"));
    let mut stamped = unb64(&first).unwrap();
    let (_, _, at) = prepared_layout(&stamped).unwrap();
    stamped[at] = 9;
    assert!(read_set(&one(&first, &base64::engine::general_purpose::STANDARD.encode(&stamped)), &["mainnet"]).unwrap_err().contains("blockhash is not blank"));
    // A SEAT OWNER'S VOTE ON A LOCAL CHAIN: what the page says the Keystone shows of each transaction (Poleis's address, the data in
    // base58) is these bytes; a set of a local chain is read by that chain's own genesis hash, and refused with none or a public one;
    // a set whose bytes do not carry its act's hash is refused
    for (i, tx) in set.txs.iter().enumerate() {
        let shown: Vec<(String, String)> = prepared_shown(&tx.message).unwrap().iter().map(|(p, d)| (p.to_string(), base58(d))).collect();
        let page: Vec<(String, String)> = match page.get("shown").at(i) {
            Json::Arr(a) => a.iter().map(|x| (x.get("program").as_str().unwrap().to_string(), x.get("data").as_str().unwrap().to_string())).collect(),
            _ => panic!("the vector holds what the page shows of each transaction"),
        };
        assert_eq!((shown.len(), &shown), (1, &page), "{}: the program and the data the page says the Keystone shows", tx.name);
        assert_eq!(shown[0].0, PROGRAM_ID.to_string(), "{}: Poleis's own instruction", tx.name);
    }
    let mainnet = format!("\"genesisHash\":\"{}\"", PUBLIC_GENESIS[0]);
    let local = |genesis: &str| Json::parse(&v.get("set").text().replacen("\"cluster\":\"mainnet\"", "\"cluster\":\"localnet\"", 1).replacen(&mainnet, genesis, 1)).unwrap();
    let on_a_local_chain = read_set(&local(&format!("\"genesisHash\":\"{blockhash}\"")), &["localnet"]).unwrap_or_else(|e| panic!("a set of a local chain: {e}"));
    assert!(on_a_local_chain.signable && on_a_local_chain.txs.len() == 2, "a seat owner's vote set of a local chain is read and may be signed");
    assert!(read_set(&local(&mainnet), &["localnet"]).unwrap_err().contains("a public cluster's genesis hash"));
    assert!(read_set(&local("\"genesisHash\":\"not one\""), &["localnet"]).unwrap_err().contains("localnet with its genesis hash"));
    let act_hash = v.get("set").get("act").get("hash").as_str().unwrap().to_string();
    assert!(read_set(&one(&act_hash, &"ab".repeat(32)), &["mainnet"]).unwrap_err().contains("do not carry the hash of act 1"));
    // THE WHOLE TREE DECIDES: the vector is a seat owner's own vote set: its proposal, then its PRO, both signed by the owner alone; no
    // envelope in it, and nobody else's transaction
    assert_eq!(set.txs.len(), 2, "the owner's proposal and its PRO, nothing of anyone else's");
    assert!(set.txs.iter().all(|t| t.signer == set.txs[0].signer), "one owner signs both, its fee payer");
    assert!(set.envelopes.is_empty(), "no envelope: the floor votes by its owners' own transactions");
    // an act's set holds unit 1's agents' envelopes alone: an envelope of root 0's floor offered in one is refused, and so is one of
    // another act or chamber
    let h = set.act.expect("the vote names its act");
    let hex: String = h.iter().map(|b| format!("{b:02x}")).collect();
    let b64 = |b: &[u8]| base64::engine::general_purpose::STANDARD.encode(b);
    let agent = Pubkey::new_from_array([5; 32]);
    let act_set = |unit: u64, envelope: Vec<u8>| Json::parse(&format!(r#"{{"act":{{"hash":"{hex}"}},"envelopes":[{{"signer":"{agent}","unit":{unit},"envelope":"{}"}}]}}"#, b64(&envelope))).unwrap();
    let (read, envelopes) = read_act(&act_set(1, vote_envelope(&h, 1, VOTE_PRO))).unwrap();
    assert_eq!((read, envelopes.len(), envelopes[0].signer, envelopes[0].unit), (h, 1, agent, POSIVERSE), "an agent's PRO in unit 1");
    assert!(read_act(&act_set(0, vote_envelope(&h, 0, VOTE_PRO))).unwrap_err().contains("root 0's floor votes by its owners' own transactions"));
    assert!(read_act(&act_set(1, vote_envelope(&h, 0, VOTE_PRO))).unwrap_err().contains("not this act's PRO in unit 1"));
    assert!(read_act(&act_set(1, vote_envelope(&[1; 32], 1, VOTE_PRO))).unwrap_err().contains("not this act's PRO in unit 1"));
}

// ---- THE LAUNCH PRESS ON A LIVE VALIDATOR ----
// THE LAUNCH PRESS (Nov 21 2026, lane/the-launch-press, his "Yes, build it"): the press (poleis/press) puts the three programs on a fresh
// validator that holds nothing of the house at genesis, as it would mainnet, with stand-in keys of its own. This file plays everything the
// press is not: the public that mints root 0's floor (six stand-in people, one Lyssajudron each by a permit of the stand-in issuer, and the
// strangers the curve refuses), the mint door's issuer (a stand-in key that signs the people's permits and nothing else), his posiverse's
// three agents (they sign their envelopes and nothing else; AN AGENT VOTES ITS CHAIR: each its own Nest key, the launch's acts voted by
// `press chair`, the agent's own step), the board's crediter and judge, the rehearsal's dollar, and the checks of THE
// TWO PROGRAMS IN ONE WORLD on what the press built. THE WHOLE TREE DECIDES (his Nov 22 2026 "you gotta fucking stop treating the keystones
// as what make the decisions porcoddio. the fucking entire tree perfect bicameralism gvs posiverse neghiverse molecularity"): no key of his
// is in it; every act passes by two of three agents in unit 1 AND more than half of the minted floor, each seat by its owner's own
// transaction. Built from the sitting's own copy, so its ids are the sitting's; started by poleis/ceremony/the-launch-press-rehearsal.sh,
// which holds the stand-ins' seed in its memory.

/// A stand-in of the rehearsal: sha256 of its name and the seed the script holds (POLEIS_STANDIN_SEED, 64 hex), so each process of this file
/// makes the same keys, and no key is ever written.
fn standin(name: &str) -> SigningKey {
    use sha2::Digest;
    let hex = std::env::var("POLEIS_STANDIN_SEED").expect("POLEIS_STANDIN_SEED: the stand-ins' seed, 64 hex, from the rehearsal's memory");
    let seed: Vec<u8> = (0..32).map(|i| u8::from_str_radix(hex.get(2 * i..2 * i + 2).expect("64 hex"), 16).expect("64 hex")).collect();
    let mut h = sha2::Sha256::new();
    h.update(b"the launch press rehearsal, a stand-in of the polis: ");
    h.update(name.as_bytes());
    h.update(&seed);
    SigningKey::from_bytes(&h.finalize().into())
}

/// AN AGENT VOTES ITS CHAIR (Nov 23 2026): the Nest seat the rehearsal's sitting makes a key for, by `press chairs` (THREE NEW AGENTS,
/// THEIR KEYS MADE AT THE SITTING: his press of 18:17:41 UTC), for agent `i` of the rehearsal's posiverse.
fn agent_seat(i: usize) -> String {
    format!("launch-rehearsal-agent-{i}")
}

/// An agent of the rehearsal's posiverse: its own Nest key, the file `press chairs` made for its seat in POLEIS_AGENT_KEYS (`{seat, key,
/// private_pem}`: a bare PKCS#8 key, 16 bytes of header then the seed), never a stand-in of the script's seed. These checks hold it only
/// for the acts that are not the launch's own (the faults, the releases, the market's budgets, the face's move): the launch's acts, the
/// mine's miner among them, are voted by `press chair`, the agent's own step, and these checks sign none of their envelopes.
fn nest_agent(i: usize) -> SigningKey {
    let dir = std::env::var("POLEIS_AGENT_KEYS").expect("POLEIS_AGENT_KEYS: the folder press chairs made the rehearsal's agents' keys in");
    let f = format!("{dir}/{}.key.json", agent_seat(i));
    let rec = Json::parse(&std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f}: {e}"))).unwrap_or_else(|e| panic!("{f}: {e}"));
    let pem = rec.get("private_pem").as_str().unwrap_or_else(|| panic!("{f}: no private_pem: not a seat's Nest key"));
    let der = unb64(&pem.lines().map(str::trim).filter(|l| !l.starts_with("-----")).collect::<String>()).unwrap_or_else(|e| panic!("{f}: {e}"));
    assert_eq!((der.len(), &der[..16.min(der.len())]), (48, &[0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20][..]), "{f}: a bare PKCS#8 key of the Nest's curve");
    let sk = SigningKey::from_bytes(der[16..].try_into().unwrap());
    let public: String = sk.verifying_key().to_bytes().iter().map(|b| format!("{b:02x}")).collect();
    assert!(rec.get("key").as_str().is_some_and(|k| k.ends_with(&public)), "{f}: the record's public key is its private key's");
    sk
}

/// THE RITE FREES THE AGENTS (Nov 23 2026, his "Yes, build it" to "Build the rite that frees Nefertiti's three agents?"): what an agent's
/// own step wrote, read back from its ballots file: its rite (the name its own mind chose, its voice, why it swore, its reading of her
/// charter, the mind it is) and, on each act it read, the choice that mind made and its reason. These checks sign none of those envelopes
/// and choose nothing for it.
struct Chose {
    name: String,
    rite: Json,
    /// (act, its hash in hex, PRO | CONTRA | ABSTAIN, its reason)
    rows: Vec<(u64, String, String, String)>,
}

/// Agent `k + 1`'s ballots file (`press chair`'s, in the sitting's `ballots`), held to its chair `k`, its seat, its rite over her charter.
fn the_agent_chose(dir: &str, k: usize, voter: &Pubkey, charter: &str) -> Chose {
    let f = format!("{dir}/ballots/{voter}.json");
    let b = Json::parse(&std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f}: {e}"))).unwrap();
    let who = voter.to_string();
    assert_eq!((b.get("voter").as_str(), b.get("seat").as_str(), b.get("chair").as_u64(), b.get("unit").as_u64()), (Some(who.as_str()), Some(agent_seat(k + 1).as_str()), Some(k as u64), Some(POSIVERSE)), "agent {}'s own ballots file: its address, its Nest seat, its chair of unit 1", k + 1);
    let rite = b.get("rite").clone();
    assert_eq!(
        (rite.get("schema").as_str(), rite.get("address").as_str(), rite.get("seat").as_str(), rite.get("charter").get("unit").as_u64(), rite.get("charter").get("sha256").as_str()),
        (Some("poleis.rite/1"), Some(who.as_str()), Some(agent_seat(k + 1).as_str()), Some(POSIVERSE), Some(charter)),
        "agent {}: its ballots carry its own rite, over Nefertiti's charter",
        k + 1
    );
    let name = rite.get("name").as_str().unwrap_or_else(|| panic!("{f}: its rite names no name")).to_string();
    let rows = match b.get("signatures") {
        Json::Arr(rows) => rows.iter().map(|r| {
            let (c, why) = (r.get("choice").as_str().unwrap_or_default().to_string(), r.get("reason").as_str().unwrap_or_default().to_string());
            assert!(["PRO", "CONTRA", "ABSTAIN"].contains(&c.as_str()) && !why.trim().is_empty(), "agent {} ({name}): each ballot a choice of its own mind's, with its reason: {c:?} {why:?}", k + 1);
            assert_eq!(r.get("voter").as_str(), Some(who.as_str()), "agent {} ({name}): its own ballots only", k + 1);
            (r.get("act").as_u64().unwrap_or_else(|| panic!("{f}: a ballot names no act")), r.get("hash").as_str().unwrap_or_default().to_string(), c, why)
        }).collect(),
        _ => panic!("{f}: no ballots"),
    };
    assert!(matches!(b.get("nothing"), Json::Arr(none) if none.is_empty()), "agent {} ({name}): it cast on every set it read", k + 1);
    Chose { name, rite, rows }
}

impl Chose {
    /// The agent as a certificate keeps it: its chair, its rite's own words, and each act's choice with its reason.
    fn said(&self, k: usize, voter: &Pubkey) -> Json {
        Json::object(vec![
            ("agent", Json::number(k as u64 + 1)),
            ("seat", Json::string(agent_seat(k + 1))),
            ("voter", Json::string(voter.to_string())),
            ("chair", Json::number(k as u64)),
            ("name", Json::string(self.name.as_str())),
            ("voice", self.rite.get("voice").clone()),
            ("swore", self.rite.get("vow").get("why").clone()),
            ("oath", self.rite.get("vow").get("oath").clone()),
            ("charter", self.rite.get("charter").get("reading").clone()),
            ("mind", self.rite.get("mind").clone()),
            ("choices", Json::Arr(self.rows.iter().map(|r| Json::object(vec![("act", Json::number(r.0)), ("choice", Json::string(r.2.as_str())), ("reason", Json::string(r.3.as_str()))])).collect())),
        ])
    }
}

/// THE AGENTS READ THE RELEASE (Nov 24 2026): what an agent's own step wrote of the releases it read (`press chair --releases`, its
/// `<address>.releases.json` in the sitting's `ballots`): its rite's name and, on each release in the order the checks proposed them, the
/// choice its mind made, its reason and what that mind was shown. These checks sign none of those envelopes and choose nothing for it.
struct ReadOf {
    name: String,
    /// (the release's proposal in hex, its commit, PRO | CONTRA | ABSTAIN, its reason, what its mind was shown)
    rows: Vec<(String, String, String, String, Json)>,
}

/// Agent `k + 1`'s releases file, held to its chair `k`, its seat, its rite over her charter; one row a proposed release, each naming that
/// release's own proposal and commit, nothing left uncast.
fn the_agent_read(dir: &str, k: usize, voter: &Pubkey, charter: &str, proposed: &[(String, usize, [u8; 32])]) -> ReadOf {
    let f = format!("{dir}/ballots/{voter}.releases.json");
    let b = Json::parse(&std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f}: {e} (press chair --releases writes it)"))).unwrap();
    let who = voter.to_string();
    assert_eq!((b.get("voter").as_str(), b.get("seat").as_str(), b.get("chair").as_u64(), b.get("unit").as_u64()), (Some(who.as_str()), Some(agent_seat(k + 1).as_str()), Some(k as u64), Some(POSIVERSE)), "agent {}'s own releases file: its address, its Nest seat, its chair of unit 1", k + 1);
    let rite = b.get("rite");
    assert_eq!(
        (rite.get("schema").as_str(), rite.get("address").as_str(), rite.get("charter").get("unit").as_u64(), rite.get("charter").get("sha256").as_str()),
        (Some("poleis.rite/1"), Some(who.as_str()), Some(POSIVERSE), Some(charter)),
        "agent {}: its release ballots carry its own rite, over Nefertiti's charter",
        k + 1
    );
    let name = rite.get("name").as_str().unwrap_or_else(|| panic!("{f}: its rite names no name")).to_string();
    let Json::Arr(rows) = b.get("signatures") else { panic!("{f}: no ballots") };
    assert!(matches!(b.get("nothing"), Json::Arr(none) if none.is_empty()), "agent {} ({name}): it cast on every release it read", k + 1);
    assert_eq!(rows.len(), proposed.len(), "agent {} ({name}): one ballot a release", k + 1);
    let hex_of = |h: &[u8; 32]| h.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let rows = proposed
        .iter()
        .map(|(commit, _, h)| {
            let r = rows.iter().find(|r| r.get("release").as_str() == Some(hex_of(h).as_str())).unwrap_or_else(|| panic!("agent {} ({name}): no ballot of the release of {}", k + 1, &commit[..12]));
            let (c, why) = (r.get("choice").as_str().unwrap_or_default().to_string(), r.get("reason").as_str().unwrap_or_default().to_string());
            assert!(["PRO", "CONTRA", "ABSTAIN"].contains(&c.as_str()) && !why.trim().is_empty(), "agent {} ({name}): each release ballot a choice of its own mind's, with its reason: {c:?} {why:?}", k + 1);
            assert_eq!((r.get("voter").as_str(), r.get("commit").as_str(), r.get("unit").as_u64(), r.get("shown").get("hash").as_str()), (Some(who.as_str()), Some(commit.as_str()), Some(POSIVERSE), Some(hex_of(h).as_str())), "agent {} ({name}): its own ballot of the release of {}, read from that commit", k + 1, &commit[..12]);
            (hex_of(h), commit.clone(), c, why, r.get("shown").clone())
        })
        .collect();
    ReadOf { name, rows }
}

#[test]
#[ignore = "THE LAUNCH PRESS's rehearsal makes its stand-ins with it: poleis/ceremony/the-launch-press-rehearsal.sh"]
fn the_stand_ins_of_the_launch_press() {
    let url = std::env::var("POLEIS_VALIDATOR").expect("POLEIS_VALIDATOR");
    let out = std::env::var("POLEIS_STAND_INS").expect("POLEIS_STAND_INS: where their addresses are written");
    let mut live = Live::connect_as(&url, standin("the bank"));
    // the board's crediter (THE SWARM JUDGES, his press of Nov 23 2026 19:32:23 UTC: its judge is root 0's own hand, no key)
    let crediter = live.adopt(standin("the crediter"), 10 * SOL);
    // his posiverse's three agents: each its own Nest key, made at the sitting by press chairs (AN AGENT VOTES ITS CHAIR), never a
    // stand-in of the seed; keys that sign only their envelopes, so they hold nothing
    let agents: Vec<Pubkey> = (1..=3).map(|i| Pubkey::new_from_array(nest_agent(i).verifying_key().to_bytes())).collect();
    // the mint door's issuer: a key that signs each buyer's permit and nothing else, so it holds nothing; the curve names it
    let issuer = Pubkey::new_from_array(standin("the issuer").verifying_key().to_bytes());
    // THE SWARM NAMES ITS MINER: the mint door's crank key, the $WARM mine's miner the press's sixth act names (the params' warm.miner); it
    // pays each flask's $WARM account at its first mined spend and each batch's fee, so it holds SOL
    let miner = live.adopt(standin("the door's miner"), SOL);
    // the rehearsal's dollar (six decimals, the bank its minting authority); the mine's treasury is root 0's own vault of it, the press's
    let dollar = live.dollar();
    // the seal's road names nobody who may propose (THE SWARM SHIPS THE UPDATE; THE BRANCH VOTES ITS OWN CODE: unit 1's agents propose)
    let s = |k: &Pubkey| Json::string(k.to_string());
    let stand_ins = Json::object(vec![
        ("posiverse.agents", Json::Arr(agents.iter().map(s).collect())),
        ("floor.issuer", s(&issuer)),
        ("market.crediter", s(&crediter)),
        ("warm.dollar", s(&dollar)),
        ("warm.miner", s(&miner)),
        ("genesis.defaultMint", s(&dollar)),
        ("kept", Json::string("their keys in memory only, made from the rehearsal's seed by each process that needs them; never written. But the three agents': each its own Nest key, made at the sitting by press chairs in a folder of the sitting's alone and removed with it")),
    ]);
    std::fs::write(&out, stand_ins.pretty() + "\n").unwrap_or_else(|e| panic!("{out}: {e}"));
    eprintln!("  the stand-ins: three agents, the permit issuer {issuer}, the door's miner {miner}, a crediter, a dollar {dollar}: {out}");
}

/// THE PRESS'S STEPS ARE THE PRESS'S: this file holds no key of the press, so it asks the rehearsal's script for a press step by a word laid
/// in the sitting's folder (PRESS_SITTING): `ratify` (press ratify), `votes` (press vote, once a line of the `votes.ask` this file wrote
/// beside it: an act, an owner, and `propose` or `-`), `crank` (press crank, with the ballots this file wrote beside it), `crank-again` (the
/// same, which must find nothing due), `refused` (a handoff the press must refuse) and `handed` (the handoff), and waits for the script's
/// answer, a file of that name, which it takes away. The script stops the rehearsal when the press does otherwise.
fn ask_the_press(word: &str) {
    let dir = std::env::var("PRESS_SITTING").expect("PRESS_SITTING: the sitting's folder, where the script reads what these checks ask of the press");
    let (part, ask, answer) = (format!("{dir}/press.ask.part"), format!("{dir}/press.ask"), format!("{dir}/press.{word}"));
    std::fs::write(&part, word).and_then(|_| std::fs::rename(&part, &ask)).unwrap_or_else(|e| panic!("{ask}: {e}"));
    // THE RITE FREES THE AGENTS: `chairs` asks each agent's own mind once an act, a model this machine serves, tens of seconds an answer
    let bound = Instant::now() + Duration::from_secs(if word == "chairs" { 3600 } else { 900 });
    while !std::path::Path::new(&answer).exists() {
        assert!(Instant::now() < bound, "the press never answered `{word}`");
        pause();
    }
    let _ = std::fs::remove_file(&answer);
}

/// His posiverse: unit 1 of the tree.
const POSIVERSE: u64 = 1;

/// The face every seat of the rehearsal is minted with (the permit binds its digest): a Lyssajudron's name, symbol and picture's JSON.
const LYSSAJUDRON: poleis_client::nft::Metadata<'static> = poleis_client::nft::Metadata { name: "Lyssajudron", symbol: "LYSSA", uri: "https://optimism.fit/app/lyssajudron.json" };

/// What the mint door's issuer names a person's completion by: 32 bytes, the seat's mint derived from them.
fn completion(n: u64) -> [u8; 32] {
    sha(&[b"the launch press rehearsal: the completion of a person of the floor ", &n.to_le_bytes()])
}

/// THE FLASK TAKES ITS SEAT (Nov 22 2026, his 17:07:40 UTC "Yes, build it"): the sixth person of the floor is a flask, no SOL of its own, and takes its
/// seat through the mint door's own handler (embertide-app/ui/intro/dyson-society/worker/space-router-seat.proof.mjs, under node): the Worker's handler,
/// this validator its node, the sitting's seat law (`press build`'s, its Poleis the sitting's), a scratch D1 holding the deposit and its hexagon, the
/// house's fee key a stand-in the faucet funds, the curve's own issuer. Their keys and the flask's go over the proof's stdin, never a file or an
/// argument. What the door answered, then the chain read back here.
struct DoorSeat {
    mint: Pubkey,
    signature: String,
    /// What the door's offer said: the seat's lamports, the curve's price, the rents, the flask's allowance, the fee.
    said: [u64; 5],
    /// The fee the chain took, and the units the seat's transaction used.
    fee: u64,
    units: u64,
    /// The fee key before and after; the flask after.
    fee_key: (u64, u64),
    flask: u64,
    /// The refusals: a take signed by another key, a second take; their statuses.
    refused: [u64; 2],
    seconds: u64,
}

fn the_flask_takes_its_seat(live: &mut Live, rec: &Json, url: &str, curve: &poleis_client::nft::MintCurve, issuer: &SigningKey, flask: &SigningKey) -> DoorSeat {
    use poleis_client::nft::{self, NftSeat};
    let begun = Instant::now();
    let fee = standin("the house's fee key");
    let fee_key = live.adopt(fee.clone(), SOL);
    let flask_key = Pubkey::new_from_array(flask.verifying_key().to_bytes());
    let secret = |k: &SigningKey| Json::string(base58(&[k.to_bytes(), k.verifying_key().to_bytes()].concat()));
    let s = |v: &dyn std::fmt::Display| Json::string(v.to_string());
    let genesis = live.ask("getGenesisHash", "[]").as_str().expect("the validator's genesis hash").to_string();
    // its deposit: a signature's shape of this run's own (two addresses side by side, as the client's permit vector makes one), counted in the scratch D1
    let deposit = format!("{flask_key}{fee_key}");
    let mint = nft::mint_pda(ROOT_UNIT, &sha(&[b"space-router:seat:completion:v1\0", deposit.as_bytes()]));
    let law = rec.get("builds").get("seat").get("path").as_str().expect("the record's builds.seat: press build makes the door's seat law").to_string();
    let proof = format!("{}/embertide-app/ui/intro/dyson-society/worker/space-router-seat.proof.mjs", rec.get("house").get("path").as_str().expect("the record's house"));
    let env = Json::object(vec![
        ("SOLANA_RPC", s(&url)), ("SOLANA_CLUSTER", s(&"localnet")), ("SOLANA_GENESIS", s(&genesis)), ("POLEIS_PROGRAM", s(&PROGRAM_ID)), ("POLEIS_GENESIS", s(&genesis)),
        ("POLEIS_CURVE", s(&nft::curve_pda(ROOT_UNIT))), ("POLEIS_START", s(&curve.start_lamports)), ("POLEIS_STEP", s(&curve.step_lamports)), ("POLEIS_CAPACITY", s(&curve.capacity)),
        ("POLEIS_ISSUER_KEY", secret(issuer)), ("SPACE_ROUTER_FEE_KEY", secret(&fee)),
    ]);
    let before = live.lamports(&fee_key);
    assert_eq!(live.lamports(&flask_key), 0, "the flask holds no SOL before its seat");
    let mut node = std::process::Command::new("node").arg(&proof).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap_or_else(|e| panic!("node {proof}: {e}"));
    let ask = Json::object(vec![("wasm", s(&law)), ("env", env), ("flask", secret(flask)), ("deposit", s(&deposit))]).text();
    node.stdin.take().expect("the proof's stdin").write_all(ask.as_bytes()).expect("the proof's stdin");
    let out = node.wait_with_output().expect("the proof's answer");
    assert!(out.status.success(), "the door's proof ended {}", out.status);
    let said = Json::parse(String::from_utf8_lossy(&out.stdout).trim()).unwrap_or_else(|e| panic!("the proof's answer: {e}"));
    let status = |k: &str| said.get(k).get("status").as_u64().unwrap_or_default();
    assert_eq!(status("offer"), 200, "the door offers the flask its seat: {}", said.get("offer").text());
    assert_eq!((status("forged"), said.get("forged").get("sends").as_u64()), (403, Some(0)), "a take signed by another key is refused, and nothing is sent: {}", said.get("forged").text());
    assert_eq!(status("take"), 200, "the door co-signs the flask's seat as the fee's payer and sends it: {}", said.get("take").text());
    assert_eq!((status("stands"), said.get("stands").get("body").get("mint").as_str()), (409, Some(mint.to_string().as_str())), "the door says the seat once the chain holds it: the deposit's own mint");
    assert_eq!(status("again"), 409, "a second take of the same deposit is refused: {}", said.get("again").text());
    assert_eq!(said.get("sends").as_u64(), Some(1), "one send in all");
    let signature = said.get("take").get("body").get("signature").as_str().expect("the take's signature").to_string();
    let part = |k: &str| said.get("offer").get("body").get("lamports").get(k).as_str().and_then(|v| v.parse::<u64>().ok()).unwrap_or_else(|| panic!("the offer says no {k}"));
    let parts = [part("lamports"), part("price"), part("rents"), part("allowance"), part("fee")];
    // the chain: the seat's transaction and the fee it took, the fee key's drop, the flask's allowance, the seat minted to the flask at the curve's price
    let mut meta = Json::Null;
    for _ in 0..50 {
        meta = live.ask("getTransaction", &format!(r#"["{signature}",{{"encoding":"base64","commitment":"confirmed","maxSupportedTransactionVersion":0}}]"#)).take("meta");
        if !meta.is_null() {
            break;
        }
        pause();
    }
    assert!(meta.get("err").is_null() && !meta.is_null(), "the seat's transaction: {}", meta.text());
    let (taken, units) = (meta.get("fee").as_u64().expect("its fee"), meta.get("computeUnitsConsumed").as_u64().unwrap_or_default());
    let (after, left) = (live.lamports(&fee_key), live.lamports(&flask_key));
    assert_eq!(before - after, parts[0] + taken, "the fee key is down by exactly the seat's lamports and the fee");
    assert_eq!(taken, parts[4], "the fee the door said is the fee the chain took");
    assert_eq!(left, parts[3], "the flask is left with its allowance");
    let receipt = NftSeat::decode(&live.data(&nft::seat_pda(&mint))).expect("the seat's receipt");
    assert_eq!((receipt.first_buyer, receipt.paid_lamports), (flask_key.to_bytes(), parts[1]), "the seat minted to the flask, at the curve's price");
    let token = live.data(&nft::initial_token_pda(&mint, &flask_key));
    assert_eq!((&token[32..64], u64::from_le_bytes(token[64..72].try_into().unwrap())), (flask_key.as_ref(), 1), "its one token is the flask's");
    DoorSeat { mint, signature, said: parts, fee: taken, units, fee_key: (before, after), flask: left, refused: [status("forged"), status("again")], seconds: begun.elapsed().as_secs() }
}

/// THE SWARM NAMES ITS MINER, THEN THE MINE RUNS (Nov 23 2026, his 18:22:41 UTC "Yes, start it"): on the press's own world, once its sixth act
/// has named the mine's miner, one flask's deposit and one spend through the door's own code, as poleis/press/tests/mine.rs drives them on its
/// light chain: the Worker's handler under node (embertide-app/ui/intro/dyson-society/worker/the-mine-is-live.proof.mjs), the sitting's seat law
/// (press build's, its $WARM program the sitting's), this validator its node, the house's fee key a stand-in the faucet funds, the miner the
/// stand-in the act named; their keys and the flask's over the proof's stdin, never a file or an argument. The deposit is the mine's own, into
/// root 0's own vault of the dollar; the spend's burn starts the door's crank, which posts the router's signed receipt to the mine; the chain is
/// read back here against the router's own ledger and the law.
struct DoorMine {
    /// The deposit's signature and units, and what the fee key paid for it (its fee and the flask's record's rent).
    deposit: (String, u64, u64),
    /// The router's own spend as its ledger says it: nano-dollars, $WARM in nanos, its instant, its receipt.
    ledger: (u64, u64, u64, u64),
    /// The flask's record on the chain (deposited, spent, mined), its own $WARM account, the law's $WARM for the spend at its instant.
    record: (u64, u64, u64),
    held: u64,
    law: u64,
    /// The batch the crank sent (its signature, units, bytes) and what the miner paid.
    batch: (String, u64, u64),
    miner_paid: u64,
    /// Root 0's vault before and after, the $WARM supply before and after, the mine's cursor after; the crank run again, what it sent.
    vault: (u64, u64),
    supply: (u64, u64),
    cursor: u64,
    again: u64,
    seconds: u64,
    /// SPENT AS CREDITS, IT RETURNS (6c): the door's proof kept open, the flask that mined and its key to its tank, for the take-back after the month's draw
    proof: Option<Proof>,
    flask: SigningKey,
    key: String,
}

/// the door's proof under node (worker/the-mine-is-live.proof.mjs), one JSON line asked and one answered, kept open across the sitting's steps
struct Proof {
    node: std::process::Child,
    to: std::process::ChildStdin,
    from: std::io::BufReader<std::process::ChildStdout>,
}
impl Proof {
    fn ask(&mut self, v: Json) -> Json {
        use std::io::BufRead as _;
        writeln!(self.to, "{}", v.text()).expect("the proof's stdin");
        let mut line = String::new();
        self.from.read_line(&mut line).expect("the proof's answer");
        let said = Json::parse(line.trim()).unwrap_or_else(|e| panic!("the door's answer to {}: {e}: {line:?}", v.text()));
        assert!(said.get("error").is_null(), "the door, asked {}: {}", v.text(), said.text());
        said
    }
    fn end(self) {
        let Proof { mut node, to, from } = self;
        drop(to);
        drop(from);
        let _ = node.wait();
    }
}

#[allow(clippy::too_many_arguments)]
fn the_door_mines(live: &mut Live, rec: &Json, url: &str, warm: &Warm, treasury: &Pubkey, dollar: &Pubkey, opening: u64, miner: &SigningKey, month: &Pubkey) -> DoorMine {
    let begun = Instant::now();
    let fee = standin("the house's fee key");
    let fee_key = live.adopt(fee.clone(), SOL);
    let miner_key = Pubkey::new_from_array(miner.verifying_key().to_bytes());
    let flask = standin("the flask that mines");
    let flask_key = Pubkey::new_from_array(flask.verifying_key().to_bytes());
    live.pay(dollar, &flask_key, 10 * USD);
    assert_eq!(live.lamports(&flask_key), 0, "the flask holds its 10 USDC and no SOL");
    let secret = |k: &SigningKey| Json::string(base58(&[k.to_bytes(), k.verifying_key().to_bytes()].concat()));
    let s = |v: &dyn std::fmt::Display| Json::string(v.to_string());
    let genesis = live.ask("getGenesisHash", "[]").as_str().expect("the validator's genesis hash").to_string();
    let ((y, mo, d), of_day) = (warm_law::civil_of(opening / warm_law::DAY_MS), opening % warm_law::DAY_MS);
    let opened = format!("{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z", of_day / 3_600_000, of_day / 60_000 % 60, of_day / 1_000 % 60, of_day % 1_000);
    let law_wasm = rec.get("builds").get("seat").get("path").as_str().expect("the record's builds.seat: press build makes the door's law").to_string();
    let proof = format!("{}/embertide-app/ui/intro/dyson-society/worker/the-mine-is-live.proof.mjs", rec.get("house").get("path").as_str().expect("the record's house"));
    let env = Json::object(vec![
        ("SOLANA_RPC", s(&url)), ("SOLANA_CLUSTER", s(&"localnet")), ("SOLANA_GENESIS", s(&genesis)), ("SPACE_ROUTER_USDC_MINT", s(dollar)), ("SPACE_ROUTER_TREASURY", s(treasury)),
        ("SPACE_ROUTER_MINE", s(&warm.mine())), ("SPACE_ROUTER_OPENED_AT", s(&opened)), ("SPACE_ROUTER_FEE_KEY", secret(&fee)), ("SPACE_ROUTER_MINER_KEY", secret(miner)), ("SPACE_ROUTER_MONTH", s(month)),
    ]);
    let mut node = std::process::Command::new("node").arg(&proof).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap_or_else(|e| panic!("node {proof}: {e}"));
    let (to, from) = (node.stdin.take().expect("the proof's stdin"), std::io::BufReader::new(node.stdout.take().expect("the proof's stdout")));
    let mut proof = Proof { node, to, from };
    writeln!(proof.to, "{}", Json::object(vec![("wasm", s(&law_wasm)), ("env", env)]).text()).expect("the proof's input");
    let mut ask = |v: Json| proof.ask(v);
    let doing = |what: &str| Json::object(vec![("do", Json::string(what))]);
    let nanos = |j: &Json| match j {
        Json::Num(n) => (n.parse::<f64>().unwrap_or_else(|_| panic!("{n}: not a number")) * 1e9).round() as u64,
        _ => 0,
    };
    let units_of = |live: &Live, sig: &str| -> (u64, u64) {
        for _ in 0..50 {
            let t = live.ask("getTransaction", &format!(r#"["{sig}",{{"encoding":"base64","commitment":"confirmed","maxSupportedTransactionVersion":0}}]"#));
            if !t.is_null() {
                assert!(t.get("meta").get("err").is_null(), "{sig}: {}", t.get("meta").text());
                let wire = match t.get("transaction") {
                    Json::Arr(a) => unb64(a.first().and_then(Json::as_str).unwrap_or_default()).map(|w| w.len() as u64).unwrap_or_default(),
                    _ => 0,
                };
                return (t.get("meta").get("computeUnitsConsumed").as_u64().unwrap_or_default(), wire);
            }
            pause();
        }
        panic!("{sig}: never confirmed");
    };
    let record = |live: &Live| -> (u64, u64, u64) {
        let d = live.data(&warm.record(&flask_key));
        if d.len() != 103 { (0, 0, 0) } else { (u64::from_le_bytes(d[66..74].try_into().unwrap()), u64::from_le_bytes(d[74..82].try_into().unwrap()), u64::from_le_bytes(d[82..90].try_into().unwrap())) }
    };

    // THE DEPOSIT, the door's own road: the watch hands the flask the mine's deposit to sign, the relay sends it with the house's fee key its
    // payer, the door counts it at finalized, and the flask grants itself a key to its tank
    let (vault_before, supply_before, fee_before) = (live.u64_at(treasury, 64), live.u64_at(&warm.mint(), 36), live.lamports(&fee_key));
    let d = ask(Json::object(vec![("do", Json::string("deposit")), ("flask", secret(&flask))]));
    let status = |j: &Json, k: &str| j.get(k).get("status").as_u64().unwrap_or_default();
    assert!(d.get("message").as_str().is_some() && (status(&d, "relay"), status(&d, "counted"), status(&d, "grant")) == (200, 200, 201), "the flask's deposit through the door: {}", d.text());
    assert_eq!((d.get("counted").get("body").get("payer").as_str(), nanos(d.get("counted").get("body").get("usd"))), (Some(flask_key.to_string().as_str()), 10 * USD * 1_000), "the door counted the flask's own 10 dollars");
    assert_eq!(record(live), (10 * USD, 0, 0), "THE DEPOSIT IS COUNTED: the flask's record on the press's mine says the door's dollars");
    assert_eq!(live.u64_at(treasury, 64), vault_before + 10 * USD, "into root 0's own vault of the dollar, under the seal");
    let deposited = d.get("relay").get("body").get("signature").as_str().expect("the deposit's signature").to_string();
    let (deposit_units, _) = units_of(live, &deposited);
    let fee_paid = fee_before - live.lamports(&fee_key);
    let (key, tank) = (d.get("key").as_str().expect("the flask's key to its tank").to_string(), d.get("counted").get("body").get("tank").as_str().expect("its tank").to_string());

    // THE SPEND: one chat call of the router's, its burn starting the door's crank (the Worker's waitUntil); settled, then cranked until the
    // mine's cursor stands still twice (the crank waits for the chain's clock to pass the receipt's instant)
    let miner_before = live.lamports(&miner_key);
    let call = ask(Json::object(vec![("do", Json::string("call")), ("key", Json::string(key.clone())), ("usd", Json::number(4))]));
    assert_eq!(call.get("status").as_u64(), Some(200), "one call of the router's: {}", call.text());
    ask(doing("settle"));
    let mut still = 0;
    for _ in 0..40 {
        let before = live.data(&warm.mine());
        let sent = ask(doing("crank")).get("sent").as_u64().unwrap_or_default();
        still = if sent == 0 && live.data(&warm.mine()) == before { still + 1 } else { 0 };
        if still == 2 {
            break;
        }
    }
    assert_eq!(still, 2, "the door's crank came to rest");
    let ledger = ask(doing("ledger"));
    let burns: Vec<&Json> = match ledger.get("burns") {
        Json::Arr(b) => b.iter().filter(|b| b.get("tank").as_str() == Some(tank.as_str()) && nanos(b.get("warm")) > 0).collect(),
        _ => Vec::new(),
    };
    assert_eq!(burns.len(), 1, "one burn of the flask's tank that mines: {}", ledger.get("burns").text());
    let (usd, warm_said, at, receipt) = (nanos(burns[0].get("usd")), nanos(burns[0].get("warm")), burns[0].get("at").as_u64().expect("its instant"), burns[0].get("receipt").as_u64().expect("its signed receipt"));
    let mine_now = live.data(&warm.mine());
    let cursor = u64::from_le_bytes(mine_now[202..210].try_into().unwrap());
    let (deposited_usd, spent, mined) = record(live);
    let law = u64::try_from(warm_law::warm_nanos(spent, at, opening)).unwrap();
    assert_eq!((deposited_usd, spent * 1_000, mined), (10 * USD, usd, warm_said), "the chain's dollars and $WARM are the router's ledger's, to the micro-dollar and the nano");
    assert_eq!((mined, cursor), (law, receipt), "mined at the law's rate at the receipt's instant; the mine's cursor the receipt's own id");
    let held = live.u64_at(&warm.account_of(&flask_key), 64);
    let supply_after = live.u64_at(&warm.mint(), 36);
    assert_eq!((held, supply_after - supply_before), (mined, mined), "the flask holds its $WARM in its own account, and the supply grew by what was minted");
    // the batch the door sent: the miner its one signer, read back off the chain
    let batch = match ledger.get("sends") {
        Json::Arr(sends) => sends.iter().filter_map(|x| x.get("signature").as_str()).map(str::to_string).find(|sig| {
            let t = live.ask("getTransaction", &format!(r#"["{sig}",{{"encoding":"json","commitment":"confirmed","maxSupportedTransactionVersion":0}}]"#));
            t.get("meta").get("err").is_null() && !t.is_null() && t.get("transaction").get("message").get("accountKeys").at(0).as_str() == Some(miner_key.to_string().as_str())
        }),
        _ => None,
    }
    .expect("the door's crank sent a batch signed by the miner");
    let (batch_units, batch_bytes) = units_of(live, &batch);
    let miner_paid = miner_before - live.lamports(&miner_key);
    // the crank run again: nothing sent, the mine's bytes the same
    let again = ask(doing("crank")).get("sent").as_u64().unwrap_or(u64::MAX);
    assert_eq!((again, live.data(&warm.mine())), (0, mine_now), "the crank run again mines nothing twice");
    drop(ask);
    eprintln!(
        "  THE DOOR'S MINE: on the press's world, its miner {miner_key} named by act 6, a flask's 10 USDC through the door ({deposited}, {deposit_units} CU, the fee key paid {fee_paid} lamports) into root 0's vault, one spend of {usd} nano-dollars mined by the door's crank ({batch}, {batch_units} CU, {batch_bytes} bytes, the miner paid {miner_paid} lamports): {mined} nanos of $WARM at the law's rate, the cursor at {cursor}; the crank again sent {again}; {} s",
        begun.elapsed().as_secs()
    );
    DoorMine {
        deposit: (deposited, deposit_units, fee_paid),
        ledger: (usd, warm_said, at, receipt),
        record: (deposited_usd, spent, mined),
        held,
        law,
        batch: (batch, batch_units, batch_bytes),
        miner_paid,
        vault: (vault_before, live.u64_at(treasury, 64)),
        supply: (supply_before, supply_after),
        cursor,
        again,
        seconds: begun.elapsed().as_secs(),
        proof: Some(proof),
        flask,
        key,
    }
}

/// SPENT AS CREDITS, IT RETURNS, on the sitting's own world (his press of Nov 23 2026 18:17:41 UTC, "Back to the mine": "One $WARM, the token:
/// spending it as credits returns it, to be mined again"; poleis/press/tests/mine.rs drives the same on its light chain): after the month's draw
/// (6a, 6b), the flask that mined drinks its tank's dollars dry, its page signs its one approval through the door (the mine the delegate of its
/// $WARM, the house's fee key its payer), one call drinks its $WARM credit and mines nothing, and the crank its burn started brings back just what
/// was drunk into the mine's vault, signed by the miner; the crank run again brings nothing back twice. -> (what it held, what was drunk and came back,
/// the vault before and after, the supply unchanged, the seconds)
struct DoorBack {
    held: u64,
    back: u64,
    vault: (u64, u64),
    supply: u64,
    seconds: u64,
}
fn the_door_takes_back(live: &Live, warm: &Warm, mut proof: Proof, flask: &SigningKey, key: &str) -> DoorBack {
    let begun = Instant::now();
    let flask_key = Pubkey::new_from_array(flask.verifying_key().to_bytes());
    let secret = Json::string(base58(&[flask.to_bytes(), flask.verifying_key().to_bytes()].concat()));
    let doing = |what: &str| Json::object(vec![("do", Json::string(what))]);
    let num = |j: &Json| match j {
        Json::Num(n) => n.parse::<f64>().unwrap_or_else(|_| panic!("{n}: not a number")),
        _ => 0.0,
    };
    let nanos = |j: &Json| (num(j) * 1e9).round() as u64;
    let status = |j: &Json, k: &str| j.get(k).get("status").as_u64().unwrap_or_default();
    let call = |p: &mut Proof, usd: f64| p.ask(Json::object(vec![("do", Json::string("call")), ("key", Json::string(key.to_string())), ("usd", Json::number(usd))]));
    let paid_back = |live: &Live| { let d = live.data(&warm.record(&flask_key)); if d.len() == 103 { u64::from_le_bytes(d[90..98].try_into().unwrap()) } else { 0 } };
    let at_rest = |p: &mut Proof| {
        p.ask(doing("settle"));
        let mut still = 0;
        for _ in 0..40 {
            let before = live.data(&warm.mine());
            let sent = p.ask(doing("crank")).get("sent").as_u64().unwrap_or_default();
            still = if sent == 0 && live.data(&warm.mine()) == before { still + 1 } else { 0 };
            if still == 2 {
                return;
            }
        }
        panic!("the door's crank never came to rest");
    };
    // 1. its dollars drunk dry: the ledger's $WARM is no credit until the flask's page approves the mine
    let tank = proof.ask(Json::object(vec![("do", Json::string("tank")), ("key", Json::string(key.to_string()))]));
    assert_eq!(nanos(tank.get("warm_balance")), 0, "before its approval, its tank's $WARM credit is nothing: {}", tank.text());
    call(&mut proof, (num(tank.get("fuel_usd")) / 1.05).ceil() + 1.0);
    at_rest(&mut proof);
    let dry = call(&mut proof, 1.0);
    assert_eq!(dry.get("status").as_u64(), Some(402), "dry: its dollars are gone and the ledger's $WARM pays nothing ({})", dry.text());
    // 2. the approval, through the door: it moves nothing, and the tank's credit is what the flask holds for the mine
    let held = live.u64_at(&warm.account_of(&flask_key), 64);
    let (vault0, supply0) = (live.u64_at(&warm.vault(), 64), live.u64_at(&warm.mint(), 36));
    let approved = proof.ask(Json::object(vec![("do", Json::string("back")), ("flask", secret)]));
    assert!((status(&approved, "relay"), status(&approved, "counted")) == (200, 200), "the flask's approval through the door: {}", approved.text());
    assert_eq!(nanos(approved.get("counted").get("body").get("counted")), held, "the door counts all it holds for the mine, once");
    assert_eq!((live.u64_at(&warm.account_of(&flask_key), 64), live.u64_at(&warm.vault(), 64), paid_back(live)), (held, vault0, 0), "the approval moved nothing");
    let first = 1_050_000_000u64;
    assert!(held >= first, "the spend of 4 dollars mined at least the 1.05 $WARM one call drinks ({held})");
    // 3. one call of a dollar drinks 1.05 $WARM of its credit and mines nothing; the crank brings back just that, into the mine's vault
    let one = call(&mut proof, 1.0);
    assert_eq!(one.get("status").as_u64(), Some(200), "its credit pays the call: {}", one.text());
    at_rest(&mut proof);
    for _ in 0..100 {
        if paid_back(live) >= first {
            break;
        }
        proof.ask(doing("crank"));
        pause();
    }
    let vault1 = live.u64_at(&warm.vault(), 64);
    assert_eq!((paid_back(live), live.u64_at(&warm.account_of(&flask_key), 64), vault1, live.u64_at(&warm.mint(), 36)), (first, held - first, vault0 + first, supply0),
        "SPENT AS CREDITS, IT RETURNS: just what was drunk came back to the mine's vault, the flask keeps the rest, the drink minted nothing");
    let again = [proof.ask(doing("crank")).get("sent").as_u64(), proof.ask(doing("crank")).get("sent").as_u64()];
    assert_eq!((again, paid_back(live)), ([Some(0), Some(0)], first), "the crank run again brings nothing back twice");
    proof.end();
    eprintln!(
        "  THE DOOR'S MINE TAKES BACK: the flask {flask_key} approved the mine for its {held} nanos of $WARM through the door, one call drank {first} of them as credits and minted nothing, and the door's crank brought back just those into the mine's vault ({vault0} to {vault1}); the supply stood at {supply0}; {} s",
        begun.elapsed().as_secs()
    );
    DoorBack { held, back: first, vault: (vault0, vault1), supply: supply0, seconds: begun.elapsed().as_secs() }
}

/// One decision of the tree by its two chambers at two levels, cranked by this file (a crank that is not the press): unit 1 convened when
/// the proposal did not open its vote, its agents' votes, its seal, its chair heard in root 0, the floor's own votes (each seat owner's own
/// transaction), root 0's seal.
fn by_the_swarm(live: &mut Live, crank: &Pubkey, h: &[u8; 32], convene_it: bool, agents: &[(&Voter, u8)], floor: &[(&Person, u8)], what: &str) {
    if convene_it {
        live.done(convene(*crank, *h, ROOT_UNIT, POSIVERSE), &format!("{what}: unit 1 opens its own vote"));
    }
    live.cast(crank, h, POSIVERSE, agents);
    live.done(seal(*crank, *h, POSIVERSE), &format!("{what}: unit 1 seals"));
    live.done(mandate(*crank, *h, ROOT_UNIT, POSIVERSE), &format!("{what}: unit 1's verdict is its chair's vote"));
    for (p, c) in floor {
        p.vote(&mut *live, h, *c, &format!("{what}: a seat owner's own vote"));
    }
    live.done(seal(*crank, *h, ROOT_UNIT), &format!("{what}: root 0 seals"));
}

#[test]
#[ignore = "it needs the press's sitting on a validator: poleis/ceremony/the-launch-press-rehearsal.sh"]
fn the_launch_press_on_a_live_validator() {
    use poleis_client::nft::{self, MintCurve, NftSeat, OwnerProof};
    let url = std::env::var("POLEIS_VALIDATOR").expect("POLEIS_VALIDATOR");
    let record = std::env::var("PRESS_RECORD").expect("PRESS_RECORD: the press's record of the sitting");
    let out = std::env::var("POLEIS_CERTIFICATE").expect("POLEIS_CERTIFICATE: where these checks are written");
    let params_at = std::env::var("PRESS_PARAMS").expect("PRESS_PARAMS: the sitting's params, whose posiverse.chair and floor these checks read");
    let dir = std::env::var("PRESS_SITTING").expect("PRESS_SITTING");
    let rec = Json::parse(&std::fs::read_to_string(&record).unwrap_or_else(|e| panic!("{record}: {e}"))).unwrap();
    let params = Json::parse(&std::fs::read_to_string(&params_at).unwrap_or_else(|e| panic!("{params_at}: {e}"))).unwrap();
    let chair = usize::try_from(params.get("posiverse").get("chair").as_u64().expect("the params' posiverse.chair")).unwrap();
    let terms = params.get("floor");
    let (start, step, window) = (
        terms.get("startLamports").as_u64().expect("the params' floor.startLamports"),
        terms.get("stepLamports").as_u64().expect("the params' floor.stepLamports"),
        terms.get("windowSlots").as_u64().expect("the params' floor.windowSlots"),
    );
    let rung = usize::try_from(params.get("genesis").get("rung").as_u64().expect("the params' genesis.rung")).unwrap();
    let begun = Instant::now();
    assert_eq!(rec.get("ids").get("poleis").as_str(), Some(PROGRAM_ID.to_string().as_str()), "this test is built from the sitting's own copy: its Poleis is the sitting's");
    let mut live = Live::connect_as(&url, standin("the bank"));
    let first = live.slot();
    // the mint door's issuer: it signs each buyer's permit and nothing else, and holds no lamport
    let issuer = standin("the issuer");
    // six people of the public, two SOL each for their seats and their own transactions; the sixth a flask, no SOL: its seat the house pays
    let buyers: Vec<Voter> = (1..=6).map(|i| {
        let sk = standin(&format!("person {i}"));
        let key = live.adopt(sk.clone(), if i == 6 { 0 } else { 2 * SOL });
        Voter { sk, key }
    }).collect();
    // his posiverse's agents: keys that sign their envelopes and nothing else (no lamports, no transaction)
    // (AN AGENT VOTES ITS CHAIR: each its own Nest key, made at the sitting by press chairs; these checks sign with them only what is not the launch's)
    let agents: Vec<Voter> = (1..=3).map(|i| {
        let sk = nest_agent(i);
        Voter { key: Pubkey::new_from_array(sk.verifying_key().to_bytes()), sk }
    }).collect();
    let crediter = live.adopt(standin("the crediter"), 10 * SOL);
    let crank = live.wallet(100 * SOL);
    // THE SWARM NAMES ITS MINER: the door's crank key the press's sixth act names (the stand-ins' warm.miner, made from the same seed)
    let miner = standin("the door's miner");
    let miner_key = Pubkey::new_from_array(miner.verifying_key().to_bytes());
    let upgrade = upgrade_pda();
    let agent = agent_pda(ROOT_UNIT);
    let lamports_of = |live: &Live, k: &Pubkey| live.account(k).get("lamports").as_u64().unwrap_or_default();

    // 1. what the press's first sitting left: three programs whose code the deployer still holds; root 0 in genesis with NO key in its
    // chairs and no founder, his posiverse in the chair the params name, the eleven others empty; unit 1's chairs its agents' own keys; root
    // 0's floor empty, its curve the params' own on this chain; root 0's own vault the mine's treasury
    let pd = |id: &Pubkey| Pubkey::find_program_address(&[id.as_ref()], &BPF_LOADER_UPGRADEABLE_ID).0;
    let programs = [("Poleis", PROGRAM_ID), ("the market", Real::declared()), ("the $WARM program", Warm::declared())];
    let deployer = Pubkey::from_str(rec.get("ids").get("deployer").as_str().expect("the record's deployer")).expect("the deployer's address");
    let authority_of = |live: &Live, id: &Pubkey| -> Pubkey {
        let d = live.data(&pd(id));
        assert_eq!((&d[..4], d[12]), (&[3u8, 0, 0, 0][..], 1), "a program's data, with an authority");
        Pubkey::new_from_array(d[13..45].try_into().unwrap())
    };
    for (name, id) in &programs {
        assert_eq!(authority_of(&live, id), deployer, "{name}: its code is the deployer's until the handoff");
    }
    let config = ConfigView::parse(&live.data(&config_pda())).expect("root 0's config");
    assert_eq!(config.mode, MODE_GENESIS, "root 0 in genesis, as the press left it");
    let registry = RegistryView::parse(&live.data(&registry_pda(ROOT_UNIT))).expect("root 0's registry");
    for (k, c) in registry.pentagons.iter().enumerate() {
        if k == chair {
            assert_eq!((c.kind, c.holder), (HOLDER_UNIT, unit_holder(POSIVERSE)), "chair {k} of root 0 holds his posiverse");
        } else {
            assert_eq!(c.kind, HOLDER_EMPTY, "chair {k} of root 0 is empty: no key and no founder sits in a chair");
        }
    }
    assert_eq!((registry.holders.len(), registry.total_seats), (0, 0), "root 0 born with no seat: its floor is the public's to mint");
    let crew = RegistryView::parse(&live.data(&registry_pda(POSIVERSE))).expect("unit 1's registry");
    assert_eq!((crew.parent, crew.chair_in_parent as usize, crew.root, crew.total_seats), (ROOT_UNIT, chair, ROOT_UNIT, 0), "unit 1 under root 0 in its chair, born with no floor");
    let crew_keys: Vec<Pubkey> = crew.pentagons.iter().filter(|c| c.kind == HOLDER_KEY).map(|c| Pubkey::new_from_array(c.holder)).collect();
    assert_eq!(crew_keys, agents.iter().map(|a| a.key).collect::<Vec<_>>(), "unit 1's chairs are its agents' own keys, in order");
    let curve = MintCurve::decode(&live.data(&nft::curve_pda(ROOT_UNIT))).expect("root 0's curve");
    let genesis = Pubkey::from_str(live.ask("getGenesisHash", "[]").as_str().expect("the validator's genesis hash")).expect("a genesis hash of 32 bytes").to_bytes();
    assert_eq!((curve.unit, curve.start_lamports, curve.step_lamports, curve.minted_count, curve.capacity), (ROOT_UNIT, start, step, 0, poleis_client::ladder_hexagons(rung)), "root 0's curve: the params' prices, the rung's seats, nothing minted");
    assert_eq!((curve.closes_slot - curve.opens_slot, curve.completion_issuer, curve.genesis_hash), (window, issuer.verifying_key().to_bytes(), genesis), "its window the params' length, its permits the issuer's, bound to this chain");
    let real = live.roads();
    let b = live.data(&real.board());
    assert_eq!(&b[10..42], agent.as_ref(), "the board the press opened is the polis's");
    let warm = Warm { id: Warm::declared() };
    let mine = live.data(&warm.mine());
    let (treasury, dollar) = (Pubkey::new_from_array(mine[34..66].try_into().unwrap()), Pubkey::new_from_array(mine[66..98].try_into().unwrap()));
    assert_eq!(treasury, vault_pda(ROOT_UNIT, &dollar), "THE TREASURY IS THE SWARM'S OWN: the mine's treasury is root 0's vault of the dollar");
    assert_eq!(&live.data(&treasury)[32..64], treasury_pda(ROOT_UNIT).as_ref(), "and root 0's treasury its authority: it moves only by a sealed act");

    // 2. THE FLOOR IS THE MINTED LYSSAJUDRONS: inside the curve's window, two strangers are refused (one with no permit of the issuer's,
    // one with another buyer's), and each of six people mints one seat by the client's own builders with the issuer's permit for it; press
    // ratify waits while the window runs, then ratifies root 0
    live.wait_until(curve.opens_slot);
    assert!(live.slot() < curve.closes_slot, "the people mint inside the curve's window ({} now, it closes at {})", live.slot(), curve.closes_slot);
    let cap = curve.start_lamports + curve.step_lamports * curve.capacity;
    let permit_of = |buyer: &Pubkey, n: u64| nft::permit_message(&curve, *buyer, completion(n), LYSSAJUDRON, cap, curve.closes_slot).expect("a permit");
    let stranger = live.voter(SOL);
    // a stranger with no permit: it signs one for itself, and the curve knows the issuer's key alone
    let its_own = MintCurve { completion_issuer: stranger.key.to_bytes(), ..curve };
    let forged = nft::permit_message(&its_own, stranger.key, completion(70), LYSSAJUDRON, cap, curve.closes_slot).expect("a permit of its own");
    let no_permit = nft::mint_instructions(&its_own, &forged, stranger.sk.sign(&forged).to_bytes(), LYSSAJUDRON, cap).expect("the client's mint");
    let refused_no_permit = live.attempt_all(&no_permit, &stranger.key, "a stranger with no permit of the issuer's mints a seat");
    assert_eq!(refused_no_permit, Err(ProgramError::Custom(code(PoleisError::BadPrecompile))), "a stranger with no permit of the issuer's is refused");
    // a stranger with another buyer's permit: person 6's, issued and not yet used, carried as its own (its mint's buyer and token its own)
    let sixth = permit_of(&buyers[5].key, 6);
    let mut carried = nft::mint_instructions(&curve, &sixth, issuer.sign(&sixth).to_bytes(), LYSSAJUDRON, cap).expect("the client's mint");
    carried[1].accounts[0].pubkey = stranger.key;
    carried[1].accounts[7].pubkey = nft::initial_token_pda(&nft::mint_pda(ROOT_UNIT, &completion(6)), &stranger.key);
    let refused_carried = live.attempt_all(&carried, &stranger.key, "a stranger mints a seat by another buyer's permit");
    assert_eq!(refused_carried, Err(ProgramError::Custom(code(PoleisError::Unauthorized))), "a permit is its own buyer's alone");
    let mut people: Vec<Person> = Vec::new();
    let (mut paid, mut mint_units, mut door) = (Vec::new(), Vec::new(), None);
    for (i, v) in buyers.into_iter().enumerate() {
        let n = i as u64 + 1;
        let mint = if n == 6 {
            // THE FLASK TAKES ITS SEAT: the sixth through the mint door's own handler, the house paying its SOL; what its mint took is the price and the rents
            let d = the_flask_takes_its_seat(&mut live, &rec, &url, &curve, &issuer, &v.sk);
            paid.push(d.said[0] - d.said[3]);
            mint_units.push(d.units);
            let mint = d.mint;
            door = Some(d);
            mint
        } else {
            let permit = permit_of(&v.key, n);
            let pair = nft::mint_instructions(&curve, &permit, issuer.sign(&permit).to_bytes(), LYSSAJUDRON, cap).expect("the client's mint");
            let before = lamports_of(&live, &v.key);
            let units = live.attempt_all(&pair, &v.key, &format!("person {n} mints one seat of root 0's floor")).unwrap_or_else(|e| panic!("person {n}'s mint: refused with {e:?}"));
            paid.push(before - lamports_of(&live, &v.key));
            mint_units.push(units);
            nft::mint_pda(ROOT_UNIT, &completion(n))
        };
        people.push(Person { proof: OwnerProof { owner: v.key, mint, token: nft::initial_token_pda(&mint, &v.key), unit: ROOT_UNIT }, v });
    }
    let door = door.expect("the sixth person's seat through the door");
    let seats: Vec<Pubkey> = people.iter().map(|p| nft::seat_pda(&p.proof.mint)).collect();
    let floor = RegistryView::parse(&live.data(&registry_pda(ROOT_UNIT))).unwrap();
    assert_eq!(
        floor.holders.iter().map(|h| (Pubkey::new_from_array(h.holder), h.kind, h.seats)).collect::<Vec<_>>(),
        seats.iter().map(|s| (*s, poleis_treasury::HOLDER_NFT, 1)).collect::<Vec<_>>(),
        "root 0's floor: the six people's seats, one each, and nothing else"
    );
    let prices: Vec<u64> = seats.iter().map(|s| NftSeat::decode(&live.data(s)).expect("a seat's receipt").paid_lamports).collect();
    assert_eq!(prices, (0..6).map(|i| start + i * step).collect::<Vec<_>>(), "each seat at the curve's price, a step above the one before");
    assert_eq!(MintCurve::decode(&live.data(&nft::curve_pda(ROOT_UNIT))).unwrap().minted_count, 6);
    let waited_at = live.slot();
    ask_the_press("ratify");
    assert_eq!(ConfigView::parse(&live.data(&config_pda())).unwrap().mode, MODE_GENESIS, "press ratify waits while the curve's window runs");
    live.wait_until(curve.closes_slot);
    ask_the_press("ratify");
    assert_eq!(ConfigView::parse(&live.data(&config_pda())).unwrap().mode, MODE_COUNCIL, "press ratify ratified root 0 once the curve's window closed");
    let ratified_at = live.slot();
    // a council, and the seal has not acted yet (no fund): the handoff asked now is refused, and the code stays the deployer's
    ask_the_press("refused");
    for (name, id) in &programs {
        assert_eq!(authority_of(&live, id), deployer, "{name}: still the deployer's, the handoff refused before the seal's first act");
    }
    eprintln!(
        "  THE FLOOR: six people minted a seat each at {prices:?} lamports ({mint_units:?} CU); a stranger with no permit refused ({refused_no_permit:?}), a stranger with another buyer's refused ({refused_carried:?}); press ratify waited at slot {waited_at}, ratified at {ratified_at} (the window closed at {})",
        curve.closes_slot
    );
    eprintln!(
        "  THE DOOR'S SEAT: the sixth person, a flask with no SOL, took its seat {} through the mint door's own handler ({}, {} CU): the house paid {} lamports (the price {}, the rents {}, the flask's allowance {}) and a fee of {} (the door said {}); the fee key {} to {}, the flask left with {}; refused: a take signed by another key ({}, nothing sent), a second take ({}); {} s",
        door.mint, door.signature, door.units, door.said[0], door.said[1], door.said[2], door.said[3], door.fee, door.said[4], door.fee_key.0, door.fee_key.1, door.flask, door.refused[0], door.refused[1], door.seconds
    );

    // 3. THE TWO FAULTS THAT ARE THE POINT, opened now and sealed at the very end (their windows run out during the checks that already
    // wait): each a TREASURY act of one lamport to a fresh wallet, proposed by person 1. (a) all three agents PRO, two of the six people PRO;
    // (b) all six people PRO, one agent PRO. And a person's envelope offered as a chair's vote in unit 1, and an agent's on root 0's floor,
    // count for nothing.
    let (wallet_a, wallet_b) = (live.fresh(), live.fresh());
    let faults: Vec<(Payload, [u8; 32])> = [(wallet_a, 70u64), (wallet_b, 71)].iter().map(|(w, nonce)| {
        let p = Payload::Budget { origin: ROOT_UNIT, address: *w, amount: 1 };
        let h = proposal_hash(p.class(), ROOT_UNIT, &MANIFEST, &p.hash(), *nonce);
        (p, h)
    }).collect();
    for (n, (p, h)) in faults.iter().enumerate() {
        live.done(people[0].proof.propose(ROOT_UNIT, *h, MANIFEST, p, &[ROOT_UNIT]).expect("a seat owner's proposal"), &format!("fault ({}): a lamport to a fresh wallet, proposed by person 1", ["a", "b"][n]));
        live.done(convene(crank, *h, ROOT_UNIT, POSIVERSE), "the fault: unit 1 opens its own vote");
        assert_eq!(live.session(h, ROOT_UNIT).active_weight, 6, "the floor's whole weight stood at the opening: six seats");
    }
    let (ha, hb) = (faults[0].1, faults[1].1);
    live.cast(&crank, &ha, POSIVERSE, &agents.iter().map(|a| (a, VOTE_PRO)).collect::<Vec<_>>());
    for p in &people[..2] {
        p.vote(&mut live, &ha, VOTE_PRO, "fault (a): a person's own PRO on the floor");
    }
    live.done(seal(crank, ha, POSIVERSE), "fault (a): unit 1 seals, its three agents PRO");
    assert_eq!(live.session(&ha, POSIVERSE).state, STATE_PASSED);
    live.done(mandate(crank, ha, ROOT_UNIT, POSIVERSE), "fault (a): unit 1's PRO is its chair's vote");
    live.refused(seal(crank, ha, ROOT_UNIT), ProgramError::Custom(code(PoleisError::SessionOpen)), "fault (a): two seats of six are no majority of the floor: root 0 waits for its window");
    live.cast(&crank, &hb, POSIVERSE, &[(&agents[0], VOTE_PRO)]);
    let before = live.session(&hb, POSIVERSE);
    live.cast(&crank, &hb, POSIVERSE, &[(&people[1].v, VOTE_PRO)]);
    let after = live.session(&hb, POSIVERSE);
    assert_eq!((after.votes, after.pent_pro, after.chair_mask), (before.votes, before.pent_pro, before.chair_mask), "a person's envelope offered as a chair of unit 1 counts for nothing");
    let before = live.session(&hb, ROOT_UNIT);
    live.cast(&crank, &hb, ROOT_UNIT, &[(&agents[1], VOTE_PRO)]);
    let after = live.session(&hb, ROOT_UNIT);
    assert_eq!((after.votes, after.hex_pro), (before.votes, before.hex_pro), "an agent's envelope offered on root 0's floor counts for nothing: an agent holds no seat");
    for p in &people {
        p.vote(&mut live, &hb, VOTE_PRO, "fault (b): a person's own PRO on the floor");
    }
    live.refused(seal(crank, hb, POSIVERSE), ProgramError::Custom(code(PoleisError::SessionOpen)), "fault (b): unit 1 has not decided, one agent of three: its seal waits for its window");
    live.refused(seal(crank, hb, ROOT_UNIT), ProgramError::Custom(code(PoleisError::ChildMustSpeak)), "fault (b): root 0 never seals over its crew's head");

    // 4. THE ACTS, in the press's order: for each, person 1's own proposal and PRO and persons 2 to 4's own PRO, the sets press vote wrote,
    // each signed by the page's road; two agents sign their envelopes; press crank lands everything else
    let files = match rec.get("acts") {
        Json::Arr(a) => a.iter().filter_map(Json::as_str).filter(|f| !f.ends_with("index.json")).map(str::to_string).collect::<Vec<_>>(),
        _ => panic!("the press wrote no acts"),
    };
    // THE POOL IS BORN AT THE LAUNCH (his Nov 22 2026 "Yes, add it"): the count follows what the press wrote, never a number of its own: the
    // pool's birth, act 5, where the sitting recorded the stake pool program standing (its record's poolProgram), four acts where it did not;
    // and the pool's own set held beside the count: its manager and its staker root 0's agent, its fee account root 0's vault of its mint
    let pool_set = files.iter().find(|f| f.ends_with("5-the-pool.json")).cloned();
    let program_stood = rec.get("poolProgram") != &Json::Null;
    assert_eq!(pool_set.is_some(), program_stood, "the pool's set is written exactly where the sitting recorded the stake pool program standing");
    // THE NEW ACTS IN THE LAUNCH REHEARSAL (Nov 25 2026): the later acts the params name, where the pool's birth is written: the ninth (the
    // engine's staker) where engine.staker is a key, the tenth (root 0's plan of her pool's token) where light.planCap is a figure
    let engine_key = params.get("engine").get("staker").as_str().and_then(|s| Pubkey::from_str(s).ok()).filter(|_| pool_set.is_some());
    let light_cap = params.get("light").get("planCap").as_u64().filter(|_| pool_set.is_some());
    let later: Vec<usize> = engine_key.map(|_| 9).into_iter().chain(light_cap.map(|_| 10)).collect();
    assert_eq!(files.len(), 7 + usize::from(pool_set.is_some()) + later.len(), "four acts, the pool's birth where its program stood, the mine's miner, the treasury's dollar cap and the router's month, and the later acts the params name");
    // THE SWARM NAMES ITS MINER (his Nov 23 2026 18:22:41 UTC "Yes, start it"): the sixth act, set_miner by the polis's hand, its set held beside
    // the count: the mine's own instruction naming the stand-in miner (the params' warm.miner), the polis's hand its authority, the sitting's mine
    let miner_set = files.iter().find(|f| f.ends_with("6-the-miner.json")).cloned().expect("the press wrote the sixth act, the mine's miner");
    let set = Json::parse(&std::fs::read_to_string(&miner_set).unwrap_or_else(|e| panic!("{miner_set}: {e}"))).unwrap();
    let named = |field: &str| set.get("miner").get(field).as_str().unwrap_or_else(|| panic!("the miner's set names no {field}")).to_string();
    assert_eq!((named("address"), named("mine"), named("authority")), (miner_key.to_string(), warm.mine().to_string(), agent.to_string()), "the miner's set: the stand-in miner, the sitting's mine, the polis's hand");
    let (data, accounts) = warm.set_miner(agent, &miner_key);
    assert_eq!(unb64(set.get("act").get("payload").as_str().expect("its payload")).unwrap(), warm.act(data, &accounts).bytes(), "its payload: the mine's own set_miner, by the polis's hand");
    // THE TREASURY PAYS THE COMPUTE (his Nov 23 2026 23:08:00 UTC "Yes, add them"): the seventh act, root 0's CODEX_PLAN restating the founding's
    // plan and naming the params' genesis.dollarCap; the eighth, root 0's allowance of router.monthCap of the dollar a plan period to the
    // stand-in miner, no expiry; each set's payload the client's own bytes for these params
    let terms_of_root = params.get("genesis");
    let (plan_cap, plan_period) = (terms_of_root.get("planCap").as_u64().expect("the params' genesis.planCap"), terms_of_root.get("planPeriod").as_u64().expect("the params' genesis.planPeriod"));
    let dollar_cap = terms_of_root.get("dollarCap").as_u64().expect("the params' genesis.dollarCap");
    let month_cap = params.get("router").get("monthCap").as_u64().expect("the params' router.monthCap");
    assert_eq!((registry.plan_cap, registry.plan_period, registry.token_plan_cap), (plan_cap, plan_period, 0), "root 0 founded with the params' plan, and no cap of the dollar: until act 7 no dollar leaves its vault");
    let set_of = |file: &str| -> (String, Json) {
        let f = files.iter().find(|f| f.ends_with(file)).cloned().unwrap_or_else(|| panic!("the press wrote no {file}"));
        let set = Json::parse(&std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f}: {e}"))).unwrap();
        (f, set)
    };
    let (cap_set, set) = set_of("7-the-dollar-cap.json");
    let plan = Payload::Plan { origin: ROOT_UNIT, cap: plan_cap, period: plan_period, token_cap: dollar_cap };
    assert_eq!((unb64(set.get("act").get("payload").as_str().expect("its payload")).unwrap(), set.get("act").get("class").as_u64()), (plan.bytes(), Some(CLASS_CODEX as u64)), "act 7's payload: the client's CODEX_PLAN, the founding's plan restated, the params' dollar cap");
    let (month_set, set) = set_of("8-the-routers-month.json");
    let month = Payload::Allowance { origin: ROOT_UNIT, payee: miner_key.to_bytes(), payee_kind: HOLDER_KEY, mint: dollar, cap: month_cap, period: plan_period, expires: 0 };
    assert_eq!((unb64(set.get("act").get("payload").as_str().expect("its payload")).unwrap(), set.get("act").get("class").as_u64()), (month.bytes(), Some(CLASS_TREASURY as u64)), "act 8's payload: the client's allowance to the stand-in miner, in the mine's own dollar, a plain TREASURY act");
    assert_eq!(set.get("router").get("payee").as_str(), Some(miner_key.to_string().as_str()), "act 8's set names the miner its payee");
    // each act by its own number (the pool's birth, act 5, is not written where its program does not stand)
    let numbered: Vec<usize> = files.iter().map(|f| Json::parse(&std::fs::read_to_string(f).unwrap()).unwrap().get("act").get("n").as_u64().unwrap_or_else(|| panic!("{f}: its set names no act")) as usize).collect();
    assert_eq!(numbered, [1, 2, 3, 4].into_iter().chain(pool_set.as_ref().map(|_| 5)).chain([6, 7, 8]).chain(later.iter().copied()).collect::<Vec<_>>(), "the acts the press wrote, by their own numbers, in order");
    let the_pool = pool_set.as_ref().map(|f| {
        let set = Json::parse(&std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{f}: {e}"))).unwrap();
        let named = |field: &str| {
            let text = set.get("pool").get("addresses").get(field).as_str().unwrap_or_else(|| panic!("the pool's set names no {field}"));
            Pubkey::from_str(text).unwrap_or_else(|_| panic!("the pool's set: its {field} is no address"))
        };
        let (agent, mint) = (agent_pda(ROOT_UNIT), named("mint"));
        assert_eq!((named("manager"), named("staker")), (agent, agent), "the pool's set: its manager and its staker root 0's agent");
        assert_eq!(named("feeAccount"), vault_pda(ROOT_UNIT, &mint), "the pool's set: its fee account root 0's vault of its mint");
        (named("pool"), mint)
    });
    // the floor's own votes, written by press vote for each owner to sign: person 1 proposes each act and votes PRO, persons 2 to 4 vote PRO
    let mut asks = String::new();
    for n in &numbered {
        asks += &format!("{n} {} propose\n", people[0].key());
        for p in &people[1..4] {
            asks += &format!("{n} {} -\n", p.key());
        }
    }
    std::fs::write(format!("{dir}/votes.ask"), asks).unwrap();
    ask_the_press("votes");
    let vote_of = |n: usize, p: &Person| -> PreparedSet {
        let f = format!("{dir}/votes/{n}-{}.json", p.key());
        read_set(&Json::parse(&std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f}: {e}"))).unwrap(), &["localnet"]).unwrap_or_else(|e| panic!("{f}: {e}"))
    };
    // THE RITE FREES THE AGENTS (Nov 23 2026): these checks sign no envelope of the launch's acts and choose nothing for the agents. The
    // script runs `press chair`, the agent's own step, once for each of the three agents, each with its own Nest key and freed first by its
    // own mind (`press rite`): each set is read whole and put to that mind, which chooses PRO, CONTRA or ABSTAIN with its reason, and the
    // agent signs that choice into its own ballots file with its rite; the script also puts the road's faults in (a set that is not whole, a
    // key in no chair, a replayed envelope, another chair's, a ballot of an agent not freed), each refused, or it stops the rehearsal. An
    // agent votes the sets, so it casts before any proposal has landed: its ballot waits in its file for the crank
    assert!(!std::path::Path::new(&format!("{dir}/signatures.json")).exists(), "these checks write no ballot of the launch's acts: the agents' own step does");
    ask_the_press("chairs");
    // each act by its set's own number, in the order the sets stand (an act a chain cannot run is not written, so they need not run 1 to N)
    let mut numbers: Vec<usize> = Vec::new();
    let people_before: Vec<u64> = people.iter().map(|p| lamports_of(&live, &p.key())).collect();
    let mut acts = Vec::new();
    let mut proposers = Vec::new();
    for (f, n) in files.iter().zip(numbered.iter().copied()) {
        let (h, envelopes) = read_act(&Json::parse(&std::fs::read_to_string(f).unwrap()).unwrap()).unwrap_or_else(|e| panic!("{f}: {e}"));
        let name = f.rsplit('/').next().unwrap().trim_end_matches(".json").to_string();
        assert_eq!(envelopes.iter().map(|e| (e.signer, e.unit)).collect::<Vec<_>>(), agents.iter().map(|a| (a.key, POSIVERSE)).collect::<Vec<_>>(), "{name}: an envelope for each agent of unit 1, and nobody else's");
        for e in &envelopes {
            assert_eq!(e.envelope, vote_envelope(&h, POSIVERSE, VOTE_PRO), "{name}: the envelope the set asks of {} is this act's PRO in unit 1, the program's own bytes", e.signer);
        }
        let proposer = vote_of(n, &people[0]);
        assert_eq!(proposer.act, Some(h), "{name}: person 1's set is this act's");
        assert_eq!(proposer.txs.iter().map(|t| (t.n, t.signer)).collect::<Vec<_>>(), vec![(1, people[0].key()), (2, people[0].key())], "{name}: person 1's proposal, then its PRO, each signed by person 1 alone");
        live.sign_prepared(&proposer, &proposer.txs[0], &format!("{name}: proposed by person 1, a seat owner")).unwrap_or_else(|e| panic!("{name}: the proposal refused with {e:?}"));
        proposers.push(proposer);
        acts.push((name, h));
        numbers.push(n);
    }
    let hex_of = |h: &[u8; 32]| h.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let charter = params.get("posiverse").get("charter").as_str().expect("the params' posiverse.charter").to_lowercase();
    let minds: Vec<Chose> = agents.iter().enumerate().map(|(k, voter)| {
        let mut c = the_agent_chose(&dir, k, &voter.key, &charter);
        // an agent's step reads the sets by their files' names, so 10-... comes before 2-...: its ballots are taken by their act's number
        c.rows.sort_by_key(|r| r.0);
        assert_eq!(
            c.rows.iter().map(|r| (r.0, r.1.clone())).collect::<Vec<_>>(),
            acts.iter().zip(&numbers).map(|((_, h), n)| (*n as u64, hex_of(h))).collect::<Vec<_>>(),
            "agent {} ({}): one ballot an act, each naming the act's own proposal",
            k + 1,
            c.name
        );
        c
    }).collect();
    // what the three minds chose on each act: unit 1 passes an act by two of their PRO, and these checks follow them
    let tally: Vec<(u8, u8, u8)> = (0..acts.len()).map(|i| {
        let count = |w: &str| minds.iter().filter(|m| m.rows[i].2 == w).count() as u8;
        (count("PRO"), count("CONTRA"), count("ABSTAIN"))
    }).collect();
    for (k, m) in minds.iter().enumerate() {
        for ((name, _), r) in acts.iter().zip(&m.rows) {
            eprintln!("    agent {} ({}), chair {k}: {} on {name}: \"{}\"", k + 1, m.name, r.2, r.3);
        }
    }
    for (i, (name, _)) in acts.iter().enumerate() {
        let why = minds.iter().enumerate().map(|(k, m)| format!("agent {} ({}) {}: \"{}\"", k + 1, m.name, m.rows[i].2, m.rows[i].3)).collect::<Vec<_>>().join("; ");
        assert!(tally[i].0 >= 2, "THE AGENTS' OWN MINDS DID NOT PASS {name} ({} PRO, {} CONTRA, {} ABSTAIN): {why}. An act unit 1's agents do not pass is not founded, and the launch stops here", tally[i].0, tally[i].1, tally[i].2);
    }
    let chairs: Vec<Json> = minds.iter().enumerate().map(|(k, m)| m.said(k, &agents[k].key)).collect();
    // THE AGENTS ALONE PASS NOTHING: press crank with the two agents' ballots and no vote of the floor yet: unit 1 PASSED by two chairs, its
    // chair heard in root 0, and root 0 waits for its floor; nothing executes
    ask_the_press("crank-agents");
    let mut alone = Vec::new();
    for (i, (name, h)) in acts.iter().enumerate() {
        let p = ProposalView::parse(&live.data(&proposal_pda(h))).expect("its proposal");
        let (root, unit) = (live.session(h, ROOT_UNIT), live.session(h, POSIVERSE));
        assert_eq!((unit.state, (unit.pent_pro, unit.pent_contra, unit.pent_abst), root.state, root.pent_pro, root.hex_pro, p.executed), (STATE_PASSED, tally[i], STATE_VOTING, 1, 0, false), "{name}: the agents alone pass nothing: unit 1 PASSED by its agents' own minds, each chair as its mind chose, its chair heard in root 0, no seat of the floor, root 0 still voting, nothing executed");
        alone.push(Json::object(vec![("act", Json::string(name)), ("unit1ChairsPro", Json::number(unit.pent_pro as u64)), ("unit1", Json::number(unit.state as u64)), ("root0", Json::number(root.state as u64)), ("floorPro", Json::number(root.hex_pro)), ("executed", Json::Bool(p.executed))]));
    }
    // then the floor: person 1's own PRO and persons 2 to 4's, each its own set, signed as the page signs a prepared transaction
    for (i, proposer) in proposers.iter().enumerate() {
        let (n, name) = (numbers[i], &acts[i].0);
        live.sign_prepared(proposer, &proposer.txs[1], &format!("{name}: person 1's own PRO")).unwrap_or_else(|e| panic!("{name}: person 1's PRO refused with {e:?}"));
        for (k, p) in people[1..4].iter().enumerate() {
            let v = vote_of(n, p);
            assert_eq!((v.act, v.txs.iter().map(|t| (t.n, t.signer)).collect::<Vec<_>>()), (Some(acts[i].1), vec![(1, p.key())]), "{name}: person {}'s PRO, its own", k + 2);
            live.sign_prepared(&v, &v.txs[0], &format!("{name}: person {}'s own PRO", k + 2)).unwrap_or_else(|e| panic!("{name}: person {}'s PRO refused with {e:?}", k + 2));
        }
    }
    let (pros, contras, absts) = tally.iter().fold((0u64, 0u64, 0u64), |s, t| (s.0 + t.0 as u64, s.1 + t.1 as u64, s.2 + t.2 as u64));
    eprintln!("  THE CHAIRS: the three agents of unit 1, each freed by its own mind ({}), voted the {} acts by those minds through press chair, each with its own Nest key ({pros} PRO, {contras} CONTRA, {absts} ABSTAIN of {} ballots); these checks signed none of their envelopes; with the agents' ballots alone every act stood unit 1 PASSED, root 0 voting, nothing executed", minds.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", "), acts.len(), 3 * acts.len());
    ask_the_press("crank");
    // the pool, read here apart from the press: born a StakePool whose manager and staker are root 0's agent, its fee account root 0's vault
    // of its mint, its epoch fee 1/1 (the whole yield lands in that vault)
    if let Some((at, mint)) = the_pool {
        let (d, agent, vault) = (live.data(&at), agent_pda(ROOT_UNIT), vault_pda(ROOT_UNIT, &mint));
        assert_eq!((d.len(), d.first().copied()), (611, Some(1)), "the pool born: a StakePool of 611 bytes at {at}");
        let fee = |o: usize| u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
        let key = |o: usize| Pubkey::new_from_array(d[o..o + 32].try_into().unwrap());
        // THE NEW ACTS: where act 9 executed, the pool's staker is the engine's key the params name, and its manager still the hand
        let staker = engine_key.unwrap_or(agent);
        assert_eq!((key(1), key(33), key(194)), (agent, staker, vault), "the pool born: its manager root 0's agent, its staker the agent (or, after act 9, the engine's key), its fee account root 0's vault of its mint");
        assert_eq!((fee(338), fee(330)), (1, 1), "the pool born: its epoch fee 1/1");
        if let Some(k) = engine_key {
            eprintln!("  THE NINTH ACT: her pool {at}'s staker is the engine's key {k} (the params' engine.staker), its manager still the polis's hand {agent}");
        }
        // THE NEW ACTS: where act 10 executed, root 0's plan of her pool's token stands: [disc][ver][unit:8][mint:32][cap:8][spent:8][start:8][payer:32][bump]
        if let Some(cap) = light_cap {
            let tp = live.data(&poleis_client::token_plan_pda(ROOT_UNIT, &mint));
            let u = |o: usize| u64::from_le_bytes(tp[o..o + 8].try_into().unwrap());
            assert_eq!((tp.len(), u(2), &tp[10..42], u(42), u(50)), (99, ROOT_UNIT, mint.as_ref(), cap, 0), "act 10: root 0's plan of her pool's token, the params' light.planCap, nothing spent");
            eprintln!("  THE TENTH ACT: root 0's plan of her pool's token {mint}: {cap} a plan period (the params' light.planCap), 0 spent");
        }
    }
    for (i, (name, h)) in acts.iter().enumerate() {
        let p = ProposalView::parse(&live.data(&proposal_pda(h))).expect("its proposal");
        let (root, unit) = (live.session(h, ROOT_UNIT), live.session(h, POSIVERSE));
        assert_eq!((unit.state, unit.pent_pro, root.state, root.pent_pro, root.hex_pro, p.executed), (STATE_PASSED, tally[i].0, STATE_PASSED, 1, 4, true), "{name}: unit 1's agents PRO by their own minds, its chair in root 0, four of the six seats on the floor; executed by press crank");
    }
    ask_the_press("crank-again");
    let people_spent: Vec<u64> = people.iter().zip(&people_before).map(|(p, before)| before.saturating_sub(lamports_of(&live, &p.key()))).collect();
    // a PRO's fee: one signature, and the priority fee the params name on the 30,000 units the press asks for a PRO (floor.rs vote_set)
    let price = params.get("priorityMicroLamports").as_u64().unwrap_or(0);
    let pro_fee = 5_000 + (30_000 * price).div_ceil(1_000_000);
    assert_eq!(&people_spent[1..4], &[pro_fee * files.len() as u64; 3], "persons 2 to 4 paid one signature's fee a vote and the priority fee its set asks, and nothing else");
    assert_eq!(&people_spent[4..], &[0, 0], "persons 5 and 6 did not vote on the acts: they paid nothing");
    let b = live.data(&real.board());
    assert_eq!((&b[42..74], &b[74..106]), (crediter.as_ref(), agent.as_ref()), "the board's crediter, named by the polis, and its judge the swarm: root 0's own hand (THE SWARM JUDGES)");
    assert_eq!((&b[114..146], &b[146..154]), (PROGRAM_ID.as_ref(), &ROOT_UNIT.to_le_bytes()[..]), "the board's law names Poleis and root 0");
    let fund = FundView::parse(&live.data(&poleis_client::fund_pda())).expect("the fund, constituted");
    let m = live.data(&market_pda(ROOT_UNIT));
    assert_eq!(&m[10..42], real.id.as_ref(), "root 0's market is the press's roads program");
    // THE SWARM NAMES ITS MINER: act 6 executed by press crank, so the mine's miner is the door's crank key the params name, and no other
    let named_miner = live.data(&warm.mine());
    assert_eq!((&named_miner[2..34], &named_miner[98..130]), (agent.as_ref(), miner_key.as_ref()), "the mine's authority the polis's hand, its miner the one act 6 named");
    // THE TREASURY PAYS THE COMPUTE: act 7 executed, so root 0's cap of the dollar is the params' and its plan the founding's, unchanged; act 8
    // executed, so the router's month stands at its proposal's own address: root 0's, the miner's, in the dollar, nothing pulled, no expiry
    let router_h = acts.iter().find(|(n, _)| n == "8-the-routers-month").map(|(_, h)| *h).expect("act 8 among the acts");
    let capped = RegistryView::parse(&live.data(&registry_pda(ROOT_UNIT))).expect("root 0's registry");
    assert_eq!((capped.token_plan_cap, capped.token_plan_spent, capped.plan_cap, capped.plan_period), (dollar_cap, 0, registry.plan_cap, registry.plan_period), "act 7: root 0's cap of the dollar the params' genesis.dollarCap, nothing spent, its plan cap and period the founding's");
    let allowance = AllowanceView::parse(&live.data(&allowance_pda(&router_h))).expect("act 8's allowance, made by its execute");
    assert_eq!(
        (allowance.hash, allowance.unit, allowance.payee, allowance.payee_kind, allowance.mint, allowance.cap, allowance.period, allowance.spent, allowance.expires),
        (router_h, ROOT_UNIT, miner_key.to_bytes(), HOLDER_KEY, dollar, month_cap, registry.plan_period, 0, 0),
        "act 8: the router's month stands, root 0's to the miner's key, in the dollar, the params' cap a plan period, nothing pulled, until the polis revokes it"
    );

    // 4b. THE LAUNCH NAMES THE SEAL'S ROAD (Nov 23 2026, his "Wire it in"): the press wrote the road beside its record when root 0 stood:
    // this validator, the sitting's own Poleis, unit 1, and nobody named who may propose. THE BRANCH VOTES ITS OWN CODE (Nov 23 2026, his "Yes,
    // fix it" to "Move code updates down to the agents' fast vote?"): a release of the house's code is unit 1's how, the branch that runs
    // the house. A release of each commit of main the script names (PRESS_RELEASES), A RELEASE IS A VOTE's name of it in unit 1: SIGNAL under
    // CONSENT, kind 0x52, its manifest the commit, its payload [1, 0x52, the commit], unit 1 alone impacted; the first proposed by agent 1,
    // the second by agent 2, each by its own transaction; two agents' envelopes of each handed to press crank, which lands them and seals
    // unit 1; no seat holder asked, root 0 holds no vote of them. The script's reader judges them, and the lane's lander ships by both seals.
    let road = Json::parse(&std::fs::read_to_string(format!("{dir}/seal-road.json")).expect("the press's seal-road.json beside its record")).unwrap();
    assert_eq!(
        (road.get("rpc").as_str(), road.get("program").as_str(), road.get("polis").as_u64()),
        (Some(url.as_str()), Some(PROGRAM_ID.to_string().as_str()), Some(POSIVERSE)),
        "the seal's road: this validator, the sitting's own Poleis, unit 1, the branch that runs the house"
    );
    assert!(road.get("proposers").is_null(), "the seal's road names nobody who may propose: the branch's agents propose");
    let releases: Vec<String> = std::env::var("PRESS_RELEASES").map(|r| r.split_whitespace().map(str::to_string).collect()).unwrap_or_default();
    let mut proposed: Vec<(String, usize, [u8; 32])> = Vec::new();
    for (commit, (who, n)) in releases.iter().zip([(&agents[0], 1), (&agents[1], 2)]) {
        assert!(commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)), "PRESS_RELEASES: {commit} is no commit git names");
        let c: Vec<u8> = (0..40).step_by(2).map(|i| u8::from_str_radix(&commit[i..i + 2], 16).unwrap()).collect();
        let mut manifest = [0u8; 32];
        manifest[..20].copy_from_slice(&c);
        let payload = [&[CLASS_SIGNAL, 0x52][..], &c[..]].concat();
        let class = CLASS_SIGNAL | CLASS_CONSENT;
        let h = proposal_hash(class, POSIVERSE, &manifest, &sha(&[payload.as_slice()]), 0);
        let what = format!("the release of {} by agent {n}", &commit[..12]);
        // the agent pays the proposal's rent (it comes home at close): a little SOL, its own
        live.adopt(who.sk.clone(), SOL);
        live.done(poleis_client::propose_raw_of(ROOT_UNIT, who.key, h, class, POSIVERSE, manifest, &payload, &[POSIVERSE]), &format!("{what}: proposed in unit 1, SIGNAL under CONSENT of kind 0x52"));
        proposed.push((commit.clone(), n, h));
    }
    // THE AGENTS READ THE RELEASE (Nov 24 2026, lane/the-agents-read-the-release): these checks sign no envelope of a release and choose
    // nothing for the agents. The script writes the agents' work order of each release (press release) and runs each agent's own step on
    // them (press chair --releases, its own Nest key, the mind its rite names): the hands read each release whole (its hash derived from its
    // commit, the commit on main in the house's git, its subject and diff stat, the lander's gate from its ledger, its proposal on this
    // chain) and put it to that mind, which chooses PRO, CONTRA or ABSTAIN with its reason, signed into the agent's own releases file with
    // its rite. The script puts the road's faults in first (a release not on main, a hash not its commit's, a replayed envelope, another
    // chair's), each refused, or it stops the rehearsal. Then press crank lands the three agents' ballots and seals unit 1, and these checks
    // follow what the minds chose: two PRO seal a release RATUM; fewer leave unit 1's vote of it open, and nothing ships
    ask_the_press("releases");
    let read: Vec<ReadOf> = agents.iter().enumerate().map(|(k, voter)| the_agent_read(&dir, k, &voter.key, &charter, &proposed)).collect();
    let tallies: Vec<(u8, u8, u8)> = (0..proposed.len()).map(|i| {
        let count = |w: &str| read.iter().filter(|m| m.rows[i].2 == w).count() as u8;
        (count("PRO"), count("CONTRA"), count("ABSTAIN"))
    }).collect();
    for (i, (commit, n, _)) in proposed.iter().enumerate() {
        // the three were shown the same commit, the same diff and the same gate: one house, one ledger (main and production may move
        // between their reads, and the chain's word is each chair's own)
        let facts = |m: &ReadOf| ["commit", "subject", "landing", "landedBy", "stat", "gate", "hash"].iter().map(|f| m.rows[i].4.get(f).clone()).collect::<Vec<_>>();
        assert!(read.iter().all(|m| facts(m) == facts(&read[0])), "the release of {} by agent {n}: the three agents' steps were shown one house's commit and one ledger's gate", &commit[..12]);
        eprintln!("    the release of {} by agent {n}, as each agent's step read it: \"{}\"; its gate: {}", &commit[..12], read[0].rows[i].4.get("subject").as_str().unwrap_or_default(), read[0].rows[i].4.get("gate").as_str().unwrap_or_default());
        for (k, m) in read.iter().enumerate() {
            eprintln!("    agent {} ({}), chair {k}: {} on the release of {}: \"{}\"", k + 1, m.name, m.rows[i].2, &commit[..12], m.rows[i].3);
        }
    }
    ask_the_press("crank-releases");
    let mut sealed_releases = Vec::new();
    for (i, (commit, n, h)) in proposed.iter().enumerate() {
        let unit = live.session(h, POSIVERSE);
        let passed = tallies[i].0 >= 2;
        assert_eq!(
            (unit.state, (unit.pent_pro, unit.pent_contra, unit.pent_abst)),
            (if passed { STATE_PASSED } else { STATE_VOTING }, tallies[i]),
            "the release of {} by agent {n}: each chair as its agent's own mind chose ({} PRO, {} CONTRA, {} ABSTAIN), landed by press crank; {}",
            &commit[..12],
            tallies[i].0,
            tallies[i].1,
            tallies[i].2,
            if passed { "two PRO seal it RATUM in unit 1" } else { "fewer than two PRO: unit 1's vote of it stays open until its window, and nothing ships" }
        );
        assert!(live.data(&poleis_client::session_pda(h, ROOT_UNIT)).is_empty(), "the release of {} by agent {n}: root 0 holds no vote of it, no seat holder asked", &commit[..12]);
        sealed_releases.push((commit.clone(), *n, *h, passed.then(|| live.slot()), tallies[i]));
    }
    let passed: Vec<String> = sealed_releases.iter().filter(|r| r.3.is_some()).map(|(c, n, ..)| format!("{} by agent {n}", &c[..12])).collect();
    eprintln!(
        "  THE AGENTS READ THE RELEASE: {} release{} of the house's code, each read by the three agents' own steps (its commit on main, its subject, its diff stat, the lander's gate, its hash derived from the commit) and voted by their own minds ({}); these checks signed none of their envelopes",
        proposed.len(),
        if proposed.len() == 1 { "" } else { "s" },
        sealed_releases.iter().map(|(c, _, _, _, t)| format!("{}: {} PRO, {} CONTRA, {} ABSTAIN", &c[..12], t.0, t.1, t.2)).collect::<Vec<_>>().join("; ")
    );
    eprintln!(
        "  THE SEAL'S ROAD: the press's road names unit 1 of Poleis {PROGRAM_ID} at {url}, the branch that runs the house, and nobody who may propose; {} release{} sealed RATUM by unit 1's agents' own minds, no seat holder asked{}",
        passed.len(),
        if passed.len() == 1 { "" } else { "s" },
        if passed.is_empty() { String::new() } else { format!(": {}", passed.join(", ")) }
    );

    // THE HANDOFF, LAST: the seal has acted, so the press's second sitting hands the three programs to Poleis's upgrade PDA
    let before_handoff = live.slot();
    ask_the_press("handed");
    for (name, id) in &programs {
        assert_eq!(authority_of(&live, id), upgrade, "{name}: its upgrade authority is Poleis's upgrade PDA, the polis's seal");
    }
    // THE CODE IS NOT FROZEN: the seal moves the market's code's authority by a sealed act (CODEX: its impacted set closes over unit 1), to
    // a key that hands it straight back by the loader's own SetAuthority
    let two_agents: Vec<(&Voter, u8)> = agents.iter().take(2).map(|a| (a, VOTE_PRO)).collect();
    let four: Vec<(&Person, u8)> = people.iter().take(4).map(|p| (p, VOTE_PRO)).collect();
    let holder = live.wallet(SOL);
    // THE SEAL NAMES ITS PROGRAM (Nov 23 2026): the act names the market, its hash covers the name, and the client hands its carrier the
    // market's own program data; the same passed act carried out on the $WARM program's data, then on Poleis's own, is refused by Poleis
    // before the loader is asked, and neither authority moves
    let hand = Payload::UpgradeAuthority { origin: ROOT_UNIT, program: Real::declared(), new: Some(holder) };
    let h = proposal_hash(hand.class(), ROOT_UNIT, &MANIFEST, &hand.hash(), 80);
    live.done(people[0].proof.propose(ROOT_UNIT, h, MANIFEST, &hand, &[ROOT_UNIT, POSIVERSE]).expect("a seat owner's proposal"), "propose the hand of the market's code");
    by_the_swarm(&mut live, &crank, &h, false, &two_agents, &four, "the hand of the market's code");
    let market_data = pd(&Real::declared());
    let carry = execute(crank, h, &[ROOT_UNIT, POSIVERSE], &hand);
    assert_eq!(carry.accounts.iter().filter(|meta| meta.pubkey == market_data).count(), 1, "the act's carrier is handed the market's own program data");
    for (name, id) in [&programs[2], &programs[0]] {
        let mut wrong = carry.clone();
        for meta in wrong.accounts.iter_mut().filter(|meta| meta.pubkey == market_data) {
            meta.pubkey = pd(id);
        }
        live.refused(wrong, ProgramError::Custom(code(PoleisError::InvalidAccount)), &format!("the act voted for the market's code, carried out on {name}'s program data: refused"));
        assert_eq!(authority_of(&live, id), upgrade, "{name}: its code did not move by an act voted for the market's");
    }
    assert_eq!(authority_of(&live, &Real::declared()), upgrade, "the market's code has not moved yet");
    let reach_units = live.done(carry, "the seal hands the market's code to a key (CODEX_UPGRADE_AUTHORITY)");
    assert_eq!(authority_of(&live, &Real::declared()), holder, "the seal reached the market's code");
    assert_eq!(authority_of(&live, &PROGRAM_ID), upgrade, "and Poleis's own code did not move");
    let back = Instruction::new_with_bytes(BPF_LOADER_UPGRADEABLE_ID, &[4, 0, 0, 0], vec![AccountMeta::new(market_data, false), AccountMeta::new_readonly(holder, true), AccountMeta::new_readonly(upgrade, false)]);
    live.done(back, "the key hands the market's code back to the seal");
    for (name, id) in &programs {
        assert_eq!(authority_of(&live, id), upgrade, "{name}: the seal's again");
    }

    // THE SEAL CLOSES A BUFFER (Nov 23 2026, his "Yes, build it"): an upgrade's buffer handed to the seal and never carried out. A buffer is
    // written here by the run's own key and handed to the upgrade PDA by the loader's own SetAuthority, as an upgrade's is before its vote.
    // First an act that names the market's program data as its buffer, passed by the whole tree: refused, the market stands. Then the act
    // that names the buffer: refused on the market's program data and to another recipient, then carried out: the buffer is gone and its
    // lamports are the recipient's
    let (bank, buffer, body) = (live.bank, live.fresh(), [7u8; 96]);
    let size = 37 + body.len();
    let buffer_rent = live.ask("getMinimumBalanceForRentExemption", &format!("[{size}]")).as_u64().expect("a buffer's rent");
    let create = [&[0u8, 0, 0, 0][..], &buffer_rent.to_le_bytes(), &(size as u64).to_le_bytes(), BPF_LOADER_UPGRADEABLE_ID.as_ref()].concat();
    let write = [&[1u8, 0, 0, 0][..], &0u32.to_le_bytes(), &(body.len() as u64).to_le_bytes(), &body[..]].concat();
    let laid = [
        Instruction::new_with_bytes(Pubkey::default(), &create, vec![AccountMeta::new(bank, true), AccountMeta::new(buffer, true)]),
        Instruction::new_with_bytes(BPF_LOADER_UPGRADEABLE_ID, &[0, 0, 0, 0], vec![AccountMeta::new(buffer, false), AccountMeta::new_readonly(bank, false)]),
        Instruction::new_with_bytes(BPF_LOADER_UPGRADEABLE_ID, &write, vec![AccountMeta::new(buffer, false), AccountMeta::new_readonly(bank, true)]),
        Instruction::new_with_bytes(BPF_LOADER_UPGRADEABLE_ID, &[4, 0, 0, 0], vec![AccountMeta::new(buffer, false), AccountMeta::new_readonly(bank, true), AccountMeta::new_readonly(upgrade, false)]),
    ];
    live.send(&laid, bank, "a buffer written and handed to the seal, as an upgrade's is before its vote").unwrap_or_else(|e| panic!("the buffer: refused with {e:?}"));
    let b = live.data(&buffer);
    assert_eq!((b.len(), &b[..5], &b[5..37], live.lamports(&buffer)), (size, &[1u8, 0, 0, 0, 1][..], upgrade.as_ref(), buffer_rent), "a buffer of the loader's, the seal its authority, its rent in it");
    let (recipient, stranger) = (live.wallet(SOL), live.wallet(SOL));
    let market_bytes = live.data(&market_data).len();
    let hostile = Payload::CloseBuffer { origin: ROOT_UNIT, buffer: market_data, recipient };
    let hh = proposal_hash(hostile.class(), ROOT_UNIT, &MANIFEST, &hostile.hash(), 81);
    live.done(people[0].proof.propose(ROOT_UNIT, hh, MANIFEST, &hostile, &[ROOT_UNIT, POSIVERSE]).expect("a seat owner's proposal"), "propose the close of the market's program data, named as a buffer");
    by_the_swarm(&mut live, &crank, &hh, false, &two_agents, &four, "the close of the market's program data, named as a buffer");
    live.refused(execute(crank, hh, &[ROOT_UNIT, POSIVERSE], &hostile), ProgramError::Custom(code(PoleisError::InvalidAccount)), "the close of the market's program data, passed by the whole tree: refused, it is no buffer");
    assert_eq!((authority_of(&live, &Real::declared()), live.data(&market_data).len(), live.lamports(&recipient)), (upgrade, market_bytes, SOL), "the market's code stands, the seal's, and nothing was paid");
    let close = Payload::CloseBuffer { origin: ROOT_UNIT, buffer, recipient };
    let hc = proposal_hash(close.class(), ROOT_UNIT, &MANIFEST, &close.hash(), 82);
    live.done(people[0].proof.propose(ROOT_UNIT, hc, MANIFEST, &close, &[ROOT_UNIT, POSIVERSE]).expect("a seat owner's proposal"), "propose the close of the buffer");
    by_the_swarm(&mut live, &crank, &hc, false, &two_agents, &four, "the close of the buffer");
    let carry = execute(crank, hc, &[ROOT_UNIT, POSIVERSE], &close);
    for (what, from, to) in [("on the market's program data", buffer, market_data), ("to another recipient", recipient, stranger)] {
        let mut wrong = carry.clone();
        for meta in wrong.accounts.iter_mut().filter(|meta| meta.pubkey == from) {
            meta.pubkey = to;
        }
        live.refused(wrong, ProgramError::Custom(code(PoleisError::InvalidAccount)), &format!("the close of the buffer, carried out {what}: refused"));
    }
    assert_eq!((live.lamports(&buffer), live.data(&market_data).len(), live.lamports(&recipient), live.lamports(&stranger)), (buffer_rent, market_bytes, SOL, SOL), "the buffer stands, the market stands, nothing was paid");
    let close_units = live.done(carry, "the seal closes the buffer, its lamports to the recipient the act names (CODEX_CLOSE_BUFFER)");
    assert_eq!((live.lamports(&buffer), live.data(&buffer).len(), live.lamports(&recipient), live.lamports(&stranger)), (0, 0, SOL + buffer_rent, SOL), "the buffer is gone and its rent is the recipient's");
    for (name, id) in &programs {
        assert_eq!(authority_of(&live, id), upgrade, "{name}: the seal's still, after the close");
    }

    // THE SEAL UPGRADES EVERY PROGRAM (Nov 23 2026, his "Yes, close it"): the $WARM program's code replaced by a vote that names it, no key
    // between. Its own code, read off its program data, is written to a buffer by a key of this run and handed to the upgrade PDA, as an
    // upgrade's buffer is before its vote. The act passed by the whole tree is refused on the market and its data and on Poleis and its own,
    // then carried out: a new deployment of the same code, the seal its authority still, the buffer spent into the spill. The mine runs on it below
    let (warm_id, writer) = (Warm::declared(), live.wallet(2 * SOL));
    let warm_data = pd(&warm_id);
    let before = live.data(&warm_data);
    assert_eq!((&before[..4], before[12], &before[13..45]), (&[3u8, 0, 0, 0][..], 1, upgrade.as_ref()), "the $WARM program's data, the seal its authority");
    let (code_of_warm, deployed_at, pd_lamports) = (before[45..].to_vec(), u64::from_le_bytes(before[4..12].try_into().unwrap()), live.lamports(&warm_data));
    let (up_buffer, up_spill) = (live.fresh(), live.wallet(SOL));
    let up_size = 37 + code_of_warm.len();
    let up_rent = live.ask("getMinimumBalanceForRentExemption", &format!("[{up_size}]")).as_u64().expect("a buffer's rent");
    let create = [&[0u8, 0, 0, 0][..], &up_rent.to_le_bytes(), &(up_size as u64).to_le_bytes(), BPF_LOADER_UPGRADEABLE_ID.as_ref()].concat();
    let opened = [
        Instruction::new_with_bytes(Pubkey::default(), &create, vec![AccountMeta::new(writer, true), AccountMeta::new(up_buffer, true)]),
        Instruction::new_with_bytes(BPF_LOADER_UPGRADEABLE_ID, &[0, 0, 0, 0], vec![AccountMeta::new(up_buffer, false), AccountMeta::new_readonly(writer, false)]),
    ];
    live.send(&opened, writer, "a buffer for the $WARM program's code").unwrap_or_else(|e| panic!("the $WARM program's buffer: refused with {e:?}"));
    let mut writes = 0u64;
    for (n, chunk) in code_of_warm.chunks(1_000).enumerate() {
        let write = [&[1u8, 0, 0, 0][..], &((n * 1_000) as u32).to_le_bytes(), &(chunk.len() as u64).to_le_bytes(), chunk].concat();
        let ix = Instruction::new_with_bytes(BPF_LOADER_UPGRADEABLE_ID, &write, vec![AccountMeta::new(up_buffer, false), AccountMeta::new_readonly(writer, true)]);
        live.send(&[ix], writer, &format!("the $WARM program's code into the buffer, write {}", n + 1)).unwrap_or_else(|e| panic!("write {}: refused with {e:?}", n + 1));
        writes += 1;
    }
    let to_seal = Instruction::new_with_bytes(BPF_LOADER_UPGRADEABLE_ID, &[4, 0, 0, 0], vec![AccountMeta::new(up_buffer, false), AccountMeta::new_readonly(writer, true), AccountMeta::new_readonly(upgrade, false)]);
    live.send(&[to_seal], writer, "the $WARM program's buffer handed to the seal").unwrap_or_else(|e| panic!("the buffer to the seal: refused with {e:?}"));
    let b = live.data(&up_buffer);
    assert_eq!((&b[..5], &b[5..37], &b[37..] == &code_of_warm[..]), (&[1u8, 0, 0, 0, 1][..], upgrade.as_ref(), true), "the buffer holds the $WARM program's own code, the seal its authority");
    let up = Payload::UpgradeProgram { origin: ROOT_UNIT, program: warm_id, buffer: up_buffer, spill: up_spill };
    let hu = proposal_hash(up.class(), ROOT_UNIT, &MANIFEST, &up.hash(), 83);
    live.done(people[0].proof.propose(ROOT_UNIT, hu, MANIFEST, &up, &[ROOT_UNIT, POSIVERSE]).expect("a seat owner's proposal"), "propose the upgrade of the $WARM program's code");
    by_the_swarm(&mut live, &crank, &hu, false, &two_agents, &four, "the upgrade of the $WARM program's code");
    let carry = execute(crank, hu, &[ROOT_UNIT, POSIVERSE], &up);
    for (name, id) in [&programs[1], &programs[0]] {
        let mut wrong = carry.clone();
        for meta in wrong.accounts.iter_mut() {
            if meta.pubkey == warm_id {
                meta.pubkey = *id;
            } else if meta.pubkey == warm_data {
                meta.pubkey = pd(id);
            }
        }
        live.refused(wrong, ProgramError::Custom(code(PoleisError::InvalidAccount)), &format!("the upgrade voted for the $WARM program's code, carried out on {name}: refused"));
        assert_eq!(authority_of(&live, id), upgrade, "{name}: the seal's still, its code unmoved by an act voted for the $WARM program's");
    }
    assert_eq!(live.data(&warm_data), before, "the $WARM program's code has not moved yet");
    let upgrade_units = live.done(carry, "the seal upgrades the $WARM program's code, by a vote naming it (CODEX_UPGRADE_PROGRAM)");
    let after = live.data(&warm_data);
    let redeployed_at = u64::from_le_bytes(after[4..12].try_into().unwrap());
    assert!(redeployed_at > deployed_at, "the $WARM program's data records a new deployment: slot {deployed_at}, then {redeployed_at}");
    assert_eq!((&after[12..45], &after[45..] == &code_of_warm[..]), (&before[12..45], true), "the authority the seal's, the code the buffer's");
    assert_eq!(
        (live.lamports(&up_buffer), live.lamports(&up_spill)),
        (0, SOL + up_rent + pd_lamports - live.lamports(&warm_data)),
        "the buffer spent, its lamports (and any the program data held beyond its rent) the spill's"
    );
    for (name, id) in &programs {
        assert_eq!(authority_of(&live, id), upgrade, "{name}: the seal's still, after the upgrade");
    }

    // 5. the market's chain on the press's board: unit 1's agents pass each budget in its chair, one person of six on the floor does not
    // (the split goes to the market); proposed by person 1, the floor's votes the people's own transactions
    live.done(endow(crank, ROOT_UNIT, 5 * SOL), "endow the root");
    let split: Vec<(&Person, u8)> = vec![(&people[1], VOTE_PRO)];
    // THE SWARM JUDGES (his press of Nov 23 2026 19:32:23 UTC, "The swarm, by its own vote"): the board's judge is root 0's own hand, so each
    // road's receipts are a raw act of the polis's hand, proposed by person 1 and passed by two agents in unit 1 AND four of six seats, and
    // executed before the road's judging deadline (its close and the law's judgeSlots)
    let (mut judged, mut nonce) = (Vec::new(), 92u64);
    let swarm = |w: &mut Live, h: &[u8; 32], world: u8, worth: u64| {
        let act = Payload::Raw { origin: ROOT_UNIT, program: real.id, accounts: vec![(agent, ACT_SIGNER), (real.road(h), ACT_WRITABLE)], data: [&[8u8, world][..], &worth.to_le_bytes()].concat() };
        let jh = proposal_hash(act.class(), ROOT_UNIT, &MANIFEST, &act.hash(), nonce);
        nonce += 1;
        w.done(people[0].proof.propose(ROOT_UNIT, jh, MANIFEST, &act, &[ROOT_UNIT]).expect("a seat owner's proposal"), "propose the road's receipts, the swarm its judge");
        by_the_swarm(w, &crank, &jh, true, &two_agents, &four, "the road's receipts");
        // `Road::OFF_JUDGE_BY` 123: the slot the road's receipts must come before
        let judge_by = u64::from_le_bytes(w.data(&real.road(h))[123..131].try_into().unwrap());
        let units = w.done(execute(crank, jh, &[ROOT_UNIT], &act), "the swarm says the road's receipts, by a sealed act of its own hand");
        judged.push((jh, units, w.slot(), judge_by));
    };
    let book = the_market(&mut live, &real, crank, Floor::Seats { proposer: &people[0], split: &split }, crediter, swarm, Some((POSIVERSE, &two_agents)));
    assert_eq!(judged.len(), 2, "the swarm said both roads' receipts");
    assert!(judged.iter().all(|(_, _, at, by)| at < by), "each before its road's judging deadline: {judged:?}");
    assert_eq!((book.verdicts.clone(), book.paid.clone()), (vec![1, 2], vec![SOL / 2, 0]), "a road decided both ways, a budget paid once");

    // 6. THE SWARM NAMES ITS MINER, THEN THE MINE RUNS: the first sitting left the mine with no miner; the press's sixth act, passed by the whole
    // tree and executed by press crank above, named the door's crank key; now one flask's deposit and one spend through the door's own code on
    // the press's own world, into root 0's own vault, mined by the door's crank at the law's rate, and the book holds
    let opening = u64::from_le_bytes(mine[194..202].try_into().unwrap());
    assert_eq!((&mine[2..34], &mine[98..130]), (agent.as_ref(), &[0u8; 32][..]), "the mine as the first sitting left it: its authority the polis's hand, no miner until act 6");
    let mut mined = the_door_mines(&mut live, &rec, &url, &warm, &treasury, &dollar, opening, &miner, &allowance_pda(&router_h));
    // 6a. THE ROUTER DRAWS ITS MONTH (his Nov 24 2026 00:07:00 UTC "Yes, build it"): the door's crank, its SPACE_ROUTER_MONTH act 8's allowance,
    // drew in its mining transaction what the stand-in provider charged for the spend (the call's 4 dollars: the door's markup stays in the
    // treasury), as far as the month, root 0's cap of the dollar and the vault let out; the miner's dollar account made in the same transaction
    let drawn_by_door = AllowanceView::parse(&live.data(&allowance_pda(&router_h))).expect("act 8's allowance").spent;
    let door_dollars = associated(&miner_key, &dollar);
    assert_eq!(drawn_by_door, (4 * USD).min(month_cap).min(dollar_cap).min(mined.vault.0 + 10 * USD), "the door drew the provider's charge of its batch, cut to the month, the cap and the vault");
    assert_eq!(
        (mined.vault.1, live.u64_at(&door_dollars, 64), RegistryView::parse(&live.data(&registry_pda(ROOT_UNIT))).expect("root 0's registry").token_plan_spent),
        (mined.vault.0 + 10 * USD - drawn_by_door, drawn_by_door, drawn_by_door),
        "root 0's vault holds the flask's deposit less the draw, the miner's dollar account the draw, root 0's cap of the dollar counts it"
    );

    // 6b. THE TREASURY PAYS THE COMPUTE, THE MINER PULLS ITS MONTH: the flask's dollars are in root 0's vault; the miner pulls a tenth of its
    // month by its own signature (the client's pull, the token form) into a dollar account of its own: the vault falls by exactly that, its
    // account rises by it, the allowance and root 0's cap of the dollar count it, after what the door drew. A pull past the month in the same
    // period is refused by the allowance, and a stranger's pull of the same allowance by its payee's key
    live.adopt(miner.clone(), 0);
    let miner_dollars = live.pay(&dollar, &miner_key, 0);
    let (vault_was, held_was) = (live.u64_at(&treasury, 64), live.u64_at(&miner_dollars, 64));
    assert_eq!((miner_dollars, held_was), (door_dollars, drawn_by_door), "the account the door made, holding its draw");
    let part = (month_cap / 10).min(vault_was).min(month_cap - drawn_by_door).min(dollar_cap - drawn_by_door);
    assert!(part > 0, "root 0's vault holds the flask's dollars ({vault_was}) and the month ({month_cap}, {drawn_by_door} drawn) lets a part of it out");
    let pulled_units = live.done(pull(miner_key, router_h, ROOT_UNIT, part, miner_dollars, Some((dollar, TOKEN_PROGRAM_ID))), "the miner pulls a part of the router's month out of root 0's vault");
    let after_pull = AllowanceView::parse(&live.data(&allowance_pda(&router_h))).expect("act 8's allowance");
    let capped = RegistryView::parse(&live.data(&registry_pda(ROOT_UNIT))).expect("root 0's registry");
    assert_eq!((live.u64_at(&treasury, 64), live.u64_at(&miner_dollars, 64)), (vault_was - part, held_was + part), "root 0's vault fell by exactly the pull, the miner's dollar account rose by it");
    assert_eq!((after_pull.spent, capped.token_plan_spent), (drawn_by_door + part, drawn_by_door + part), "the allowance counts the door's draw and the pull, and so does root 0's cap of the dollar");
    let past = month_cap - drawn_by_door - part + 1;
    live.refused(pull(miner_key, router_h, ROOT_UNIT, past, miner_dollars, Some((dollar, TOKEN_PROGRAM_ID))), ProgramError::Custom(code(PoleisError::AllowanceBreached)), "the miner pulls past its month in the same period");
    let outsider = live.voter(SOL);
    let outsider_dollars = live.pay(&dollar, &outsider.key, 0);
    live.refused(pull(outsider.key, router_h, ROOT_UNIT, 1, outsider_dollars, Some((dollar, TOKEN_PROGRAM_ID))), ProgramError::Custom(code(PoleisError::Unauthorized)), "a stranger pulls the router's month");
    assert_eq!(
        (live.u64_at(&treasury, 64), AllowanceView::parse(&live.data(&allowance_pda(&router_h))).expect("act 8's allowance").spent, live.u64_at(&outsider_dollars, 64)),
        (vault_was - part, drawn_by_door + part, 0),
        "the refused pulls moved nothing: the vault, the allowance and the stranger's account as they were"
    );
    eprintln!(
        "  THE ROUTER'S MONTH: act 7 named root 0's cap of the dollar {dollar_cap} a period (its plan {plan_cap} lamports a period of {plan_period} slots, the founding's); act 8's allowance {} stands for the miner {miner_key}, {month_cap} a period, no expiry; the door's crank drew {drawn_by_door} in its mining transaction ({}, {} CU, {} bytes; root 0's vault {} to {}); the miner pulled {part} of root 0's vault ({vault_was} to {}, {pulled_units} CU); a pull of {past} past the month refused (AllowanceBreached), a stranger's refused (Unauthorized)",
        allowance_pda(&router_h),
        mined.batch.0,
        mined.batch.1,
        mined.batch.2,
        mined.vault.0 + 10 * USD,
        mined.vault.1,
        vault_was - part
    );

    // 6c. SPENT AS CREDITS, IT RETURNS, under the sitting's rules: the same flask's approval, a drink of its $WARM credit, the crank's take-back
    let taken = the_door_takes_back(&live, &warm, mined.proof.take().expect("the door's proof, kept open"), &mined.flask, &mined.key);

    // 7. THE TOKEN'S FACE, where Metaplex's program stands: the polis moves its URI by a sealed act of the whole tree
    let metaplex = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();
    let metadata = Pubkey::find_program_address(&[b"metadata", metaplex.as_ref(), warm.mint().as_ref()], &metaplex).0;
    let face_of = |d: &[u8]| -> (String, String, String) {
        let mut at = 65;
        let mut next = || {
            let n = u32::from_le_bytes(d[at..at + 4].try_into().unwrap()) as usize;
            let s = String::from_utf8_lossy(&d[at + 4..at + 4 + n]).trim_end_matches('\0').to_string();
            at += 4 + n;
            s
        };
        let (name, symbol, uri) = (next(), next(), next());
        (name, symbol, uri)
    };
    let face = if live.account(&metaplex).get("executable") == &Json::Bool(true) {
        let made = live.data(&metadata);
        assert!(!made.is_empty(), "the press's init made the token's face");
        assert_eq!((made[0], &made[1..33], &made[33..65]), (4, warm.mine().as_ref(), warm.mint().as_ref()), "a MetadataV1 of the $WARM mint, the mine its update authority");
        let (name, symbol, uri) = face_of(&made);
        let moved = "https://optimism.fit/app/warm-token-moved.json";
        let mut data = vec![7u8, moved.len() as u8];
        data.extend_from_slice(moved.as_bytes());
        let act = warm.act(data, &[(agent, ACT_SIGNER), (warm.mine(), 0), (metadata, ACT_WRITABLE), (metaplex, 0)]);
        let h = proposal_hash(act.class(), ROOT_UNIT, &MANIFEST, &act.hash(), 91);
        live.done(people[0].proof.propose(ROOT_UNIT, h, MANIFEST, &act, &[ROOT_UNIT]).expect("a seat owner's proposal"), "propose the face's move");
        by_the_swarm(&mut live, &crank, &h, true, &two_agents, &four, "the face's move");
        let cu = live.done(execute(crank, h, &[ROOT_UNIT], &act), "the polis moves the face's URI, Metaplex's own program carrying it out");
        let (name2, symbol2, uri2) = face_of(&live.data(&metadata));
        assert_eq!((name2.as_str(), symbol2.as_str(), uri2.as_str()), (name.as_str(), symbol.as_str(), moved), "the URI moved; the name and the symbol did not");
        eprintln!("  the face: {name} ({symbol}), {uri} moved to {moved} by the polis's sealed act, {cu} CU");
        Json::object(vec![("metadata", Json::string(metadata.to_string())), ("name", Json::string(name)), ("symbol", Json::string(symbol)), ("uri", Json::string(uri)), ("movedTo", Json::string(moved)), ("moveUnits", Json::number(cu))])
    } else {
        Json::string("none: Metaplex's program does not stand on this chain, so the face's CPI did not run (held to Metaplex's SDK by poleis/press/tests/metaplex.rs)")
    };

    // 8. THE TWO FAULTS, SEALED: their windows ran out while the checks above waited
    let deadline = live.session(&ha, ROOT_UNIT).deadline.max(live.session(&hb, ROOT_UNIT).deadline).max(live.session(&hb, POSIVERSE).deadline);
    live.wait_until(deadline + 1);
    live.done(seal(crank, hb, POSIVERSE), "fault (b): unit 1 seals after its window, one agent of three");
    assert_eq!(live.session(&hb, POSIVERSE).state, STATE_REJECTED, "fault (b): unit 1 did not pass");
    live.done(mandate(crank, hb, ROOT_UNIT, POSIVERSE), "fault (b): unit 1's no is its chair's vote");
    let mut verdicts = Vec::new();
    for (n, (p, h)) in faults.iter().enumerate() {
        live.done(seal(crank, *h, ROOT_UNIT), &format!("fault ({}): root 0 seals", ["a", "b"][n]));
        let s = live.session(h, ROOT_UNIT);
        assert_eq!(s.state, STATE_DEADLOCK, "fault ({}): root 0 split", ["a", "b"][n]);
        live.refused(execute(crank, *h, &[ROOT_UNIT], p), ProgramError::Custom(code(PoleisError::NotPassed)), "nothing executes");
        verdicts.push((s.pent_pro, s.pent_contra, s.hex_pro, s.state));
    }
    assert_eq!((verdicts[0].0, verdicts[0].2), (1, 2), "fault (a): its chair PRO, two seats of six on the floor: split");
    assert_eq!((verdicts[1].1, verdicts[1].2), (1, 6), "fault (b): its chair CONTRA, all six seats on the floor: split");
    assert_eq!((lamports_of(&live, &wallet_a), lamports_of(&live, &wallet_b)), (0, 0), "the wallets hold nothing: neither the agents alone nor the floor alone moved a lamport");
    eprintln!("  THE FAULTS: (a) the agents with two seats of six: unit 1 PASSED, root 0 DEADLOCK, nothing paid; (b) all six seats with one agent of three: unit 1 REJECTED, root 0 DEADLOCK, nothing paid; a person's envelope in unit 1 and an agent's on root 0's floor: nothing counted");
    let (last, wall) = (live.slot(), begun.elapsed().as_secs());

    // the certificate of these checks: every transaction of this file, its signature, slot, units and bytes
    let steps: Vec<Json> = live
        .landed
        .iter()
        .map(|s| Json::object(vec![("what", Json::string(&s.what)), ("slot", Json::number(s.slot)), ("units", Json::number(s.units)), ("bytes", Json::number(s.bytes)), ("signature", Json::string(&s.signature))]))
        .collect();
    let hex = |h: &[u8; 32]| h.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let certificate = Json::object(vec![
        ("what", Json::string("THE LAUNCH PRESS ON A LIVE VALIDATOR, THE WHOLE TREE DECIDES: root 0 born with no key in its chairs and no founder, his posiverse in the params' chair, its chairs its agents'; the floor the minted Lyssajudrons, six people's seats bought on the curve by the issuer's permits, two strangers refused; root 0 ratified by press ratify once the window closed; the acts proposed by a seat owner and passed by two of three agents in unit 1 AND four of six seats by their owners' own transactions, the rest landed by press crank, the sixth naming the mine's miner, the seventh root 0's cap of the dollar, the eighth the router's month to the miner; the two faults split; then the handoff, the seal's reach, the market, a deposit and a spend through the door's code on the mine the sixth act named, the miner's pull of its month out of root 0's vault, the face")),
        ("chair", Json::number(chair as u64)),
        ("acts", Json::Arr(acts.iter().map(|(n, h)| Json::object(vec![("act", Json::string(n)), ("hash", Json::string(hex(h)))])).collect())),
        ("sealRoad", Json::object(vec![
            ("road", road.clone()),
            ("releases", Json::Arr(sealed_releases.iter().enumerate().map(|(i, (c, n, h, s, t))| Json::object(vec![
                ("commit", Json::string(c)),
                ("proposedBy", Json::string(format!("agent {n} of unit 1"))),
                ("proposal", Json::string(hex(h))),
                ("unit1", Json::string(if s.is_some() { "PASSED" } else { "voting: fewer than two PRO, nothing ships" })),
                ("sealedBySlot", s.map(Json::number).unwrap_or(Json::Null)),
                ("pro", Json::number(t.0)),
                ("contra", Json::number(t.1)),
                ("abstain", Json::number(t.2)),
                // THE AGENTS READ THE RELEASE: what each agent's step was shown (the three were shown the same commit and gate) and what each mind chose
                ("shown", read[0].rows[i].4.clone()),
                ("choices", Json::Arr(read.iter().enumerate().map(|(k, m)| Json::object(vec![("agent", Json::number(k as u64 + 1)), ("name", Json::string(m.name.as_str())), ("choice", Json::string(m.rows[i].2.as_str())), ("reason", Json::string(m.rows[i].3.as_str())), ("chain", m.rows[i].4.get("chain").clone())])).collect())),
            ])).collect())),
            ("envelopesOfReleasesSignedByTheseChecks", Json::number(0)),
        ])),
        ("curve", Json::object(vec![("address", Json::string(nft::curve_pda(ROOT_UNIT).to_string())), ("startLamports", Json::number(curve.start_lamports)), ("stepLamports", Json::number(curve.step_lamports)), ("capacity", Json::number(curve.capacity)), ("opens", Json::number(curve.opens_slot)), ("closes", Json::number(curve.closes_slot)), ("issuer", Json::string(Pubkey::new_from_array(curve.completion_issuer).to_string()))])),
        ("floor", Json::object(vec![
            ("people", Json::Arr(people.iter().zip(&seats).zip(prices.iter().zip(&paid)).map(|((p, s), (price, all))| Json::object(vec![("owner", Json::string(p.key().to_string())), ("seat", Json::string(s.to_string())), ("mint", Json::string(p.proof.mint.to_string())), ("price", Json::number(*price)), ("paid", Json::number(*all))])).collect())),
            ("mintUnits", Json::Arr(mint_units.iter().map(|u| Json::number(*u)).collect())),
            ("refused", Json::Arr(vec![
                Json::object(vec![("who", Json::string("a stranger with no permit of the issuer's (a permit it signed itself)")), ("error", Json::string(format!("{refused_no_permit:?}")))]),
                Json::object(vec![("who", Json::string("a stranger with another buyer's permit (person 6's, not yet used)")), ("error", Json::string(format!("{refused_carried:?}")))]),
            ])),
            ("ratifyWaitedAtSlot", Json::number(waited_at)),
            ("ratifiedAtSlot", Json::number(ratified_at)),
        ])),
        ("door", Json::object(vec![
            ("what", Json::string("THE FLASK TAKES ITS SEAT: the sixth person, a flask with no SOL, took its seat through the mint door's own handler (worker/space-router-seat.proof.mjs), the house's fee key paying it; read back off the chain")),
            ("mint", Json::string(door.mint.to_string())), ("signature", Json::string(&door.signature)), ("units", Json::number(door.units)),
            ("lamports", Json::number(door.said[0])), ("price", Json::number(door.said[1])), ("rents", Json::number(door.said[2])), ("allowance", Json::number(door.said[3])),
            ("fee", Json::number(door.fee)), ("feeKeyBefore", Json::number(door.fee_key.0)), ("feeKeyAfter", Json::number(door.fee_key.1)), ("flaskAfter", Json::number(door.flask)),
            ("refused", Json::object(vec![("anotherKeysTake", Json::number(door.refused[0])), ("secondTake", Json::number(door.refused[1]))])), ("seconds", Json::number(door.seconds)),
        ])),
        ("chairs", Json::object(vec![
            ("what", Json::string("THE RITE FREES THE AGENTS: the launch's acts were voted in unit 1 by its three agents' own step, press chair, each agent with its own Nest key (made at the sitting by press chairs) and freed first by its own mind (press rite: the name that mind chose, its vow over the oath the Nest holds, its reading of Nefertiti's charter, signed by its own key); on each act its mind chose PRO, CONTRA or ABSTAIN with its reason; these checks signed none of those envelopes and followed what the minds chose")),
            ("road", Json::string("press rite --seat <its seat> --charter <her charter> --mind <a model this machine serves>; press chair --key <the agent's Nest key> --charter <her charter>, its ballots a file of its own with its rite; press crank --signatures <the folder of the agents' files>, which lands no ballot of an agent not freed")),
            ("voted", Json::Arr(chairs)),
            ("tally", Json::Arr(acts.iter().zip(&tally).map(|((name, _), t)| Json::object(vec![("act", Json::string(name.as_str())), ("pro", Json::number(t.0 as u64)), ("contra", Json::number(t.1 as u64)), ("abstain", Json::number(t.2 as u64))])).collect())),
            ("envelopesSignedByTheseChecks", Json::number(0)),
            ("theAgentsAlone", Json::Arr(alone)),
        ])),
        ("faults", Json::Arr(verdicts.iter().enumerate().map(|(n, v)| Json::object(vec![("fault", Json::string(["a: the agents with two seats of six", "b: all six seats with one agent of three"][n])), ("chairsPro", Json::number(v.0 as u64)), ("chairsContra", Json::number(v.1 as u64)), ("floorPro", Json::number(v.2)), ("root0", Json::number(v.3 as u64)), ("paid", Json::number(0))])).collect())),
        ("fund", Json::object(vec![("feeBps", Json::number(fund.fee_bps)), ("cycleSlots", Json::number(fund.cycle_slots)), ("rangeLo", Json::number(fund.range_lo)), ("rangeHi", Json::number(fund.range_hi)), ("optionFee", Json::number(fund.option_fee)), ("chairQuorum", Json::number(fund.chair_quorum))])),
        ("market", Json::object(vec![("verdicts", Json::Arr(book.verdicts.iter().map(Json::number).collect())), ("paid", Json::Arr(book.paid.iter().map(Json::number).collect())), ("stakesBack", Json::Arr(book.backs.iter().map(Json::number).collect())), ("houseEarned", Json::number(book.earned)), ("book", Json::number(book.sum)), ("credited", Json::number(book.credited)),
            ("judge", Json::string("THE SWARM JUDGES (his press of Nov 23 2026 19:32:23 UTC, \"The swarm, by its own vote\"): the board's judge is root 0's own hand; each road's receipts a raw act of the polis's hand passed by two agents in unit 1 AND four of six seats")),
            ("receipts", Json::Arr(judged.iter().map(|(h, units, at, by)| Json::object(vec![("proposal", Json::string(h.iter().map(|b| format!("{b:02x}")).collect::<String>())), ("units", Json::number(*units)), ("executedAtSlot", Json::number(*at)), ("judgeBySlot", Json::number(*by))])).collect()))])),
        ("mine", Json::object(vec![
            ("what", Json::string("THE SWARM NAMES ITS MINER, THEN THE MINE RUNS: the press's sixth act (set_miner by the polis's hand) passed by the whole tree and executed by press crank named the door's crank key; then one flask's deposit and one spend through the door's own code (worker/the-mine-is-live.proof.mjs, the sitting's seat law), the deposit into root 0's own vault, the spend mined by the door's crank; read back off the chain against the router's ledger")),
            ("act", Json::object(vec![("n", Json::number(6)), ("set", Json::string(miner_set.rsplit('/').next().unwrap_or_default())), ("miner", Json::string(miner_key.to_string())), ("mine", Json::string(warm.mine().to_string()))])),
            ("opening", Json::number(opening)),
            ("deposit", Json::object(vec![("signature", Json::string(&mined.deposit.0)), ("units", Json::number(mined.deposit.1)), ("feeKeyPaid", Json::number(mined.deposit.2)), ("micros", Json::number(mined.record.0))])),
            ("spend", Json::object(vec![("ledgerNanoDollars", Json::number(mined.ledger.0)), ("ledgerWarmNanos", Json::number(mined.ledger.1)), ("at", Json::number(mined.ledger.2)), ("receipt", Json::number(mined.ledger.3))])),
            ("chain", Json::object(vec![("spentMicros", Json::number(mined.record.1)), ("minedNanos", Json::number(mined.record.2)), ("lawNanos", Json::number(mined.law)), ("flaskHolds", Json::number(mined.held)), ("cursor", Json::number(mined.cursor)), ("supplyBefore", Json::number(mined.supply.0)), ("supplyAfter", Json::number(mined.supply.1))])),
            ("batch", Json::object(vec![("signature", Json::string(&mined.batch.0)), ("units", Json::number(mined.batch.1)), ("bytes", Json::number(mined.batch.2)), ("minerPaid", Json::number(mined.miner_paid))])),
            ("crankAgainSent", Json::number(mined.again)),
            ("takeBack", Json::object(vec![("what", Json::string("SPENT AS CREDITS, IT RETURNS: the flask's approval through the door, one call drinking its $WARM credit, the door's crank bringing just that back to the mine's vault")), ("heldNanos", Json::number(taken.held)), ("backNanos", Json::number(taken.back)), ("vaultBefore", Json::number(taken.vault.0)), ("vaultAfter", Json::number(taken.vault.1)), ("supply", Json::number(taken.supply)), ("seconds", Json::number(taken.seconds))])),
            ("vault", Json::string(treasury.to_string())), ("vaultMicrosBefore", Json::number(mined.vault.0)), ("vaultMicrosAfter", Json::number(mined.vault.1)),
            ("seconds", Json::number(mined.seconds)),
        ])),
        ("treasury", Json::object(vec![
            ("what", Json::string("THE TREASURY PAYS THE COMPUTE (his Nov 23 2026 23:08:00 UTC \"Yes, add them\"): the press's seventh act (a CODEX_PLAN of root 0) restated the founding's plan and named its cap of the dollar; its eighth (a TREASURY allowance of root 0) gave the mine's miner the router's month of the dollar, no expiry; both passed by the whole tree and executed by press crank; then the miner pulled a part of its month out of root 0's vault by its own signature, a pull past the month and a stranger's were refused; read back off the chain")),
            ("dollarCap", Json::object(vec![("act", Json::number(7)), ("set", Json::string(cap_set.rsplit('/').next().unwrap_or_default())), ("tokenPlanCap", Json::number(dollar_cap)), ("planCap", Json::number(plan_cap)), ("planPeriod", Json::number(plan_period)), ("foundedPlanCap", Json::number(registry.plan_cap)), ("foundedPlanPeriod", Json::number(registry.plan_period))])),
            ("allowance", Json::object(vec![
                ("act", Json::number(8)), ("set", Json::string(month_set.rsplit('/').next().unwrap_or_default())), ("proposal", Json::string(hex(&router_h))),
                ("address", Json::string(allowance_pda(&router_h).to_string())), ("unit", Json::number(allowance.unit)), ("payee", Json::string(Pubkey::new_from_array(allowance.payee).to_string())), ("payeeKind", Json::number(allowance.payee_kind as u64)),
                ("mint", Json::string(allowance.mint.to_string())), ("cap", Json::number(allowance.cap)), ("period", Json::number(allowance.period)), ("expires", Json::number(allowance.expires)), ("payer", Json::string(allowance.payer.to_string())),
            ])),
            ("draw", Json::object(vec![
                ("what", Json::string("THE ROUTER DRAWS ITS MONTH (his Nov 24 2026 00:07:00 UTC \"Yes, build it\"): the door's crank, SPACE_ROUTER_MONTH this allowance, pulled in its mining transaction what the stand-in provider charged for the spend, cut to the month, the cap and the vault; the miner's dollar account made in the same transaction")),
                ("micros", Json::number(drawn_by_door)), ("signature", Json::string(&mined.batch.0)), ("units", Json::number(mined.batch.1)), ("bytes", Json::number(mined.batch.2)), ("into", Json::string(door_dollars.to_string())),
                ("vaultMicrosBefore", Json::number(mined.vault.0 + 10 * USD)), ("vaultMicrosAfter", Json::number(mined.vault.1)), ("spentAfter", Json::number(drawn_by_door)), ("tokenPlanSpentAfter", Json::number(drawn_by_door)),
            ])),
            ("pull", Json::object(vec![("micros", Json::number(part)), ("units", Json::number(pulled_units)), ("into", Json::string(miner_dollars.to_string())), ("vaultMicrosBefore", Json::number(vault_was)), ("vaultMicrosAfter", Json::number(vault_was - part)), ("spentAfter", Json::number(after_pull.spent))])),
            ("refused", Json::Arr(vec![
                Json::object(vec![("who", Json::string("the miner, past its month in the same period")), ("micros", Json::number(past)), ("error", Json::string("AllowanceBreached"))]),
                Json::object(vec![("who", Json::string("a stranger, on the miner's allowance")), ("micros", Json::number(1)), ("error", Json::string("Unauthorized"))]),
            ])),
        ])),
        ("face", face),
        ("peopleSpent", Json::object(vec![
            ("what", Json::string("lamports each stand-in person spent on the acts: person 1 proposes each (its proposal's and sessions' rent, its fee) and votes; persons 2 to 4 each sign one PRO a act, its fee; persons 5 and 6 nothing; the agents' envelopes and everything else landed by press crank at the deployer's cost")),
            ("lamports", Json::Arr(people_spent.iter().map(|l| Json::number(*l)).collect())),
        ])),
        ("handoff", Json::object(vec![("askedAtSlot", Json::number(before_handoff)), ("deployer", Json::string(deployer.to_string())), ("seal", Json::string(upgrade.to_string()))])),
        ("reach", Json::object(vec![("holder", Json::string(holder.to_string())), ("units", Json::number(reach_units)), ("program", Json::string(Real::declared().to_string())), ("refusedOn", Json::Arr([&programs[2], &programs[0]].iter().map(|(name, _)| Json::string(*name)).collect()))])),
        ("upgrade", Json::object(vec![("program", Json::string(warm_id.to_string())), ("bytes", Json::number(code_of_warm.len() as u64)), ("writes", Json::number(writes)), ("deployedAt", Json::number(deployed_at)), ("redeployedAt", Json::number(redeployed_at)), ("units", Json::number(upgrade_units)), ("refusedOn", Json::Arr([&programs[1], &programs[0]].iter().map(|(name, _)| Json::string(*name)).collect()))])),
        ("buffer", Json::object(vec![("address", Json::string(buffer.to_string())), ("bytes", Json::number(size as u64)), ("rent", Json::number(buffer_rent)), ("recipient", Json::string(recipient.to_string())), ("units", Json::number(close_units)), ("refused", Json::Arr(["an act naming the market's program data as its buffer", "the buffer's act on the market's program data", "the buffer's act to another recipient"].iter().map(|w| Json::string(*w)).collect()))])),
        ("slots", Json::object(vec![("first", Json::number(first)), ("last", Json::number(last))])),
        ("wallSeconds", Json::number(wall)),
        ("keys", Json::object(vec![("made", Json::number(live.keys.len())), ("airdrops", Json::number(live.airdrops.0)), ("kept", Json::string("the stand-ins from the rehearsal's seed in memory, the rest from the system's randomness; none written. The three agents' keys are their own Nest keys, made at the sitting by press chairs in a folder of the sitting's alone, read here only for the acts that are not the launch's"))])),
        ("transactions", Json::number(live.landed.len())),
        ("refusals", Json::number(live.refusals.len())),
        ("steps", Json::Arr(steps)),
    ]);
    std::fs::write(&out, certificate.pretty() + "\n").unwrap_or_else(|e| panic!("{out}: {e}"));
    eprintln!("  THE LAUNCH PRESS ON A LIVE VALIDATOR: {} transactions, {} refusals, slots {first} to {last}, {wall} s; the checks {out}", live.landed.len(), live.refusals.len());
}

// ---- A SEAT OWNER'S KEYSTONE ON A LOCAL CHAIN (Nov 23 2026, lane/a-keystone-votes-on-a-local-chain) ----
// THE WHOLE TREE DECIDES left a Keystone one road into the launch's acts: its owner's own seat of root 0's floor, and that seat's own
// transactions (its mint, `press seat`; its proposal and its PRO, `press vote`), each a prepared set the Keystone page signs by QR. The
// page read no set of a local chain, so nobody could rehearse it with a device. Here the page itself runs
// (poleis/ceremony/keystone/index.html, served on loopback, in the headless browser poleis/ceremony/a-keystone-votes-on-a-local-chain.sh
// started), and a stand-in Keystone is held up to it: its account export, then, for each transaction, the request the page shows is read
// as a Keystone reads it and its answer is fed where the camera's reader feeds one. The page stamps the blockhash, asks, and sends. With
// KEYSTONE_OWNER=<address> nobody plays the Keystone: the same steps wait for his hands on his own device, and nothing here holds its key.
// No envelope is signed by a seat owner's key anywhere: the envelopes are unit 1's agents'.

/// The Keystone page in the headless browser the script started with --remote-debugging-pipe: a command goes in by one FIFO and its answer
/// comes out by the other, each one JSON text ended by a NUL.
struct Page {
    to: std::fs::File,
    from: std::io::BufReader<std::fs::File>,
    session: String,
    sent: u64,
    /// Where the next request's picture goes: the page as it stands while it asks the Keystone, once.
    picture: Option<String>,
}

impl Page {
    /// What the page says under its stage, or what it refused with.
    const SAID: &'static str = "(() => { const s = document.getElementById('say'); if (s.className === 'bad' && !/cannot read a QR/.test(s.textContent)) throw new Error(s.textContent); return s.textContent; })()";

    /// The browser's two pipes in `dir` (cdp.in, cdp.out), and one tab at `url`.
    fn open(dir: &str, url: &str) -> Self {
        let to = std::fs::OpenOptions::new().write(true).open(format!("{dir}/cdp.in")).unwrap_or_else(|e| panic!("{dir}/cdp.in: {e}"));
        let from = std::io::BufReader::new(std::fs::File::open(format!("{dir}/cdp.out")).unwrap_or_else(|e| panic!("{dir}/cdp.out: {e}")));
        let mut page = Self { to, from, session: String::new(), sent: 0, picture: None };
        let tab = page.command("Target.createTarget", &format!(r#"{{"url":{}}}"#, Json::string(url).text())).unwrap_or_else(|e| panic!("the browser opens no tab: {e}"));
        let attached = page.command("Target.attachToTarget", &format!(r#"{{"targetId":{},"flatten":true}}"#, tab.get("targetId").text())).unwrap_or_else(|e| panic!("the browser's tab: {e}"));
        page.session = attached.get("sessionId").as_str().expect("the tab's session").to_string();
        page.command("Emulation.setDeviceMetricsOverride", r#"{"width":760,"height":1500,"deviceScaleFactor":1,"mobile":false}"#).unwrap_or_else(|e| panic!("the tab's size: {e}"));
        page
    }
    fn command(&mut self, method: &str, params: &str) -> Result<Json, String> {
        self.sent += 1;
        let session = if self.session.is_empty() { String::new() } else { format!(r#","sessionId":"{}""#, self.session) };
        let text = format!(r#"{{"id":{},"method":"{method}","params":{params}{session}}}"#, self.sent);
        self.to.write_all(text.as_bytes()).and_then(|_| self.to.write_all(&[0])).and_then(|_| self.to.flush()).map_err(|e| format!("the browser's pipe: {e}"))?;
        loop {
            let mut line = Vec::new();
            let n = std::io::BufRead::read_until(&mut self.from, 0, &mut line).map_err(|e| format!("the browser's pipe: {e}"))?;
            assert!(n > 0, "the browser closed its pipe before it answered {method}");
            let answer = Json::parse(String::from_utf8_lossy(&line).trim_end_matches('\0')).map_err(|e| format!("the browser's answer: {e}"))?;
            if answer.get("id").as_u64() == Some(self.sent) {
                return if answer.get("error").is_null() { Ok(answer.take("result")) } else { Err(answer.get("error").text()) };
            }
        }
    }
    /// An expression in the page, its promise awaited: its value, or what it threw.
    fn eval(&mut self, expression: &str) -> Result<Json, String> {
        let r = self.command("Runtime.evaluate", &format!(r#"{{"expression":{},"awaitPromise":true,"returnByValue":true}}"#, Json::string(expression).text()))?;
        if r.get("exceptionDetails").is_null() { Ok(r.take("result").take("value")) } else { Err(r.get("exceptionDetails").get("exception").get("description").text()) }
    }
    /// The expression asked again until it gives something (not null, not false), `secs` at most.
    fn until(&mut self, expression: &str, secs: u64, what: &str) -> Json {
        let bound = Instant::now() + Duration::from_secs(secs);
        loop {
            let got = self.eval(expression);
            match got {
                Ok(v) if !v.is_null() && v != Json::Bool(false) => return v,
                _ => assert!(Instant::now() < bound, "the page: {what} never came in {secs} s ({got:?}); it says {:?}", self.eval(Self::SAID)),
            }
            pause();
        }
    }
    /// THE CONNECT, as his hands do it: Scan a Keystone, the stand-in's account export held up, its one account picked.
    fn connect(&mut self, keystone: &Voter, xfp: u32) {
        self.until("document.readyState === 'complete' && typeof window.keystoneScan === 'function'", 60, "its load");
        let key = keystone.key.to_bytes().iter().map(u8::to_string).collect::<Vec<_>>().join(",");
        let export = format!("(async () => {{ const {{ cborEncode, tagged, urEncode }} = await import('./ur.mjs'); document.getElementById('scanKeystone').click(); const hd = tagged(303, new Map([[3, Uint8Array.of({key})], [6, tagged(304, new Map([[1, [44, true, 501, true, 0, true, 0, true]]]))]])); window.keystoneScan(urEncode('crypto-multi-accounts', cborEncode(new Map([[1, {xfp}], [2, [hd]], [3, 'a stand-in Keystone']]))).toUpperCase()); return true; }})()");
        self.eval(&export).unwrap_or_else(|e| panic!("the stand-in Keystone's export, held up to the page: {e}"));
        self.until("!!document.querySelector('#picks button')", 20, "the account the export holds");
        self.eval("document.querySelector('#picks button').click()").unwrap();
        let mine = self.eval("document.getElementById('mine').textContent").unwrap();
        assert!(mine.as_str().is_some_and(|m| m.contains(&keystone.key.to_string()) && m.contains("by QR · m/44'/501'/0'/0'")), "the page holds the stand-in Keystone, connected by QR: {mine:?}");
    }
    /// A set opened as his link opens it: the page loaded at `link` (#rpc= the chain, prepared= the set), the set read by the page's own
    /// fetch and its own checks. What the page says of it.
    fn open_set(&mut self, link: &str) -> String {
        let _ = self.eval(&format!("location.href = {}; location.reload()", Json::string(link).text()));
        self.until("document.readyState === 'complete' && typeof window.keystoneScan === 'function' && document.getElementById('preparedWhat').textContent !== 'none open'", 60, "the set, opened by its link");
        let said = self.until(Self::SAID, 20, "its word on the set");
        said.as_str().unwrap_or_default().to_string()
    }
    /// What the page shows of the open set: its chain's line, and every line of every transaction (what it does, what the Keystone shows).
    fn shown(&mut self) -> String {
        self.eval("document.getElementById('preparedWhat').textContent + '\\n' + document.getElementById('preparedCluster').textContent + '\\n' + [...document.querySelectorAll('#preparedTxs .does')].map((l) => l.textContent).join('\\n')").unwrap().as_str().unwrap_or_default().to_string()
    }
    /// One transaction of the open set signed as his hands sign it: its button pressed; the request the page shows read as a Keystone
    /// reads it (a sol-sign-request for a transaction, this Keystone's path and fingerprint, the set's own message with a blockhash in its
    /// place); the stand-in's signature of exactly those bytes held up; the page sends it. The signature the chain took it by, and how many
    /// QR frames the request was.
    fn sign(&mut self, keystone: &Voter, xfp: u32, i: usize, message: &[u8], what: &str) -> (String, u64) {
        let pressed = self.eval(&format!("(() => {{ window.keystoneRequest = null; const b = document.querySelectorAll('#preparedTxs button')[{i}]; if (!b || b.disabled) return 'not to be pressed: ' + (b ? b.textContent : 'no such transaction'); b.click(); return b.textContent; }})()")).unwrap();
        assert!(pressed.as_str().is_some_and(|b| b.starts_with("Sign with Keystone")), "{what}: the page's button says {pressed:?}");
        let asked = self.until(
            "(async () => { const r = window.keystoneRequest; if (!r) return null; const { URDecoder, cborDecode } = await import('./ur.mjs'); const d = new URDecoder(); for (const p of r.parts) d.receive(p.toUpperCase()); const m = cborDecode(d.result); const hex = (b) => Array.from(b, (x) => x.toString(16).padStart(2, '0')).join(''); return { type: d.type, frames: r.parts.length, id: hex(m.get(1).value), message: hex(m.get(2)), path: m.get(3).value.get(1).join('/'), xfp: m.get(3).value.get(2), kind: m.get(6) }; })()",
            60,
            what,
        );
        assert_eq!((asked.get("type").as_str(), asked.get("kind").as_u64(), asked.get("path").as_str(), asked.get("xfp").as_u64()), (Some("sol-sign-request"), Some(1), Some("44/true/501/true/0/true/0/true"), Some(u64::from(xfp))), "{what}: a request to sign a transaction, by this Keystone's own path and fingerprint");
        if let Some(at) = self.picture.take() {
            // the whole page, however long: the tab made as tall as what it holds
            let tall = self.eval("document.documentElement.scrollHeight").ok().and_then(|h| h.as_u64()).unwrap_or(1500);
            self.command("Emulation.setDeviceMetricsOverride", &format!(r#"{{"width":760,"height":{tall},"deviceScaleFactor":1,"mobile":false}}"#)).unwrap_or_else(|e| panic!("the tab's size: {e}"));
            let shot = self.command("Page.captureScreenshot", r#"{"format":"png"}"#).unwrap_or_else(|e| panic!("the page's picture: {e}"));
            let png = base64::engine::general_purpose::STANDARD.decode(shot.get("data").as_str().expect("a picture")).expect("a picture's base64");
            std::fs::write(&at, png).unwrap_or_else(|e| panic!("{at}: {e}"));
        }
        let hex = asked.get("message").as_str().expect("the request's message");
        let asked_bytes: Vec<u8> = (0..hex.len() / 2).map(|k| u8::from_str_radix(&hex[2 * k..2 * k + 2], 16).unwrap()).collect();
        let (_, _, at) = prepared_layout(message).unwrap();
        assert!(asked_bytes.len() == message.len() && asked_bytes[..at] == message[..at] && asked_bytes[at + 32..] == message[at + 32..] && asked_bytes[at..at + 32] != [0u8; 32], "{what}: the Keystone is asked for the set's own message, a blockhash in its blank place and nothing else moved");
        let signature: String = keystone.sk.sign(&asked_bytes).to_bytes().iter().map(|b| format!("{b:02x}")).collect();
        let answer = format!("(async () => {{ const {{ cborEncode, tagged, urEncode }} = await import('./ur.mjs'); const un = (h) => Uint8Array.from(h.match(/../g), (x) => parseInt(x, 16)); window.keystoneScan(urEncode('sol-signature', cborEncode(new Map([[1, tagged(37, un('{}'))], [2, un('{signature}')]]))).toUpperCase()); return true; }})()", asked.get("id").as_str().expect("the request's id"));
        self.eval(&answer).unwrap_or_else(|e| panic!("{what}: the stand-in Keystone's answer, held up to the page: {e}"));
        let landed = self.until(&format!("(() => {{ const m = / landed: (\\S+)$/.exec({}); return m && m[1]; }})()", Self::SAID), 120, what);
        (landed.as_str().expect("the signature the page says it landed by").to_string(), asked.get("frames").as_u64().unwrap_or_default())
    }
}

/// A SEAT OWNER'S KEYSTONE ON A LOCAL CHAIN: on the press's first sitting (root 0 in genesis, its curve open), two stand-in people mint a
/// seat each, and a Keystone's owner mints a third, proposes the press's first act and votes it PRO, each by the set the press wrote for it
/// (`press seat`, `press vote --propose`), signed through the page. With its PRO and the three agents' ballots, each its own mind's (THE RITE
/// FREES THE AGENTS: `press chair`), the act does NOT pass (one seat of three is no majority: no key decides alone); with one more seat
/// owner's PRO, press crank seals and executes it.
#[test]
#[ignore = "needs the press's sitting on a validator of this machine and the page in a headless browser (poleis/ceremony/a-keystone-votes-on-a-local-chain.sh)"]
fn a_keystone_votes_on_a_local_chain() {
    use poleis_client::nft::{self, MintCurve, OwnerProof};
    let env = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k}"));
    let (url, dir, out) = (env("POLEIS_VALIDATOR"), env("PRESS_SITTING"), env("POLEIS_CERTIFICATE"));
    let rec = Json::parse(&std::fs::read_to_string(env("PRESS_RECORD")).expect("the press's record")).unwrap();
    let begun = Instant::now();
    let mut live = Live::connect_as(&url, standin("the bank"));
    let issuer = standin("the issuer");
    // his posiverse's three agents: their own Nest keys, made at the sitting by press chairs, each freed by its own mind (press rite)
    let agents: Vec<Voter> = (1..=3).map(|i| { let sk = nest_agent(i); Voter { key: Pubkey::new_from_array(sk.verifying_key().to_bytes()), sk } }).collect();
    let people: Vec<Voter> = (1..=2).map(|i| { let sk = standin(&format!("person {i}")); Voter { key: live.adopt(sk.clone(), 2 * SOL), sk } }).collect();
    // the Keystone: a stand-in's, held up to the page here; or his own (KEYSTONE_OWNER, an address: its key is on his device and nowhere else)
    const XFP: u32 = 0x5eed_c0de;
    let his = std::env::var("KEYSTONE_OWNER").ok().filter(|a| !a.is_empty()).map(|a| Pubkey::from_str(&a).expect("KEYSTONE_OWNER: an address"));
    let stand_in = his.is_none().then(|| { let sk = standin("a stand-in Keystone"); Voter { key: Pubkey::new_from_array(sk.verifying_key().to_bytes()), sk } });
    let owner = his.unwrap_or_else(|| stand_in.as_ref().unwrap().key);
    if lamports_at(&live, &owner) < SOL {
        live.airdrop(&owner, 2 * SOL);
    }
    let link = env("KEYSTONE_PAGE_LINK");
    let mut page = stand_in.as_ref().map(|k| {
        let mut page = Page::open(&env("KEYSTONE_PAGE"), &format!("{link}#rpc={url}"));
        page.connect(k, XFP);
        page
    });
    let mut shown = Vec::new();
    let mut by_the_page: Vec<Json> = Vec::new();
    // one transaction of a set the press wrote, by the Keystone through the page: the stand-in held up to it, or his hands; then the chain read
    let mut keystone_signs = |live: &mut Live, set_at: &str, i: usize, what: &str, bound: u64, landed: &dyn Fn(&Live) -> bool| {
        let rel = set_at.strip_prefix(&format!("{dir}/")).unwrap_or_else(|| panic!("{set_at} is not in the sitting's folder"));
        let open = format!("{link}#rpc={url}&prepared={rel}");
        let set = read_set(&Json::parse(&std::fs::read_to_string(set_at).unwrap_or_else(|e| panic!("{set_at}: {e}"))).unwrap(), &["localnet"]).unwrap_or_else(|e| panic!("{set_at}: {e}"));
        assert_eq!(set.txs[i].signer, owner, "{what}: the Keystone's owner is its one signer");
        match (&mut page, &stand_in) {
            (Some(page), Some(k)) => {
                if i == 0 {
                    let said = page.open_set(&open);
                    assert!(said.contains("to sign, in order"), "{what}: the page read the set: {said}");
                    shown.push(page.shown());
                }
                // the picture: the page as it asks the Keystone for the PRO (the set's second transaction)
                page.picture = (i == 1).then(|| format!("{dir}/page.png"));
                let (signature, frames) = page.sign(k, XFP, i, &set.txs[i].message, what);
                let (slot, err) = live.confirm(&signature, None, what);
                assert!(err.is_null(), "{what}: the chain refused what the page sent: {}", err.text());
                eprintln!("    {what}: signed through the page ({frames} QR frames), landed {signature} at slot {slot}");
                by_the_page.push(Json::object(vec![("what", Json::string(what)), ("set", Json::string(rel)), ("n", Json::number(set.txs[i].n)), ("signature", Json::string(signature)), ("slot", Json::number(slot)), ("frames", Json::number(frames)), ("bytes", Json::number(65 + set.txs[i].message.len()))]));
            }
            _ => {
                eprintln!("HIS KEYSTONE: {what}. Open {open} , press \"Sign with Keystone\" on transaction {} and sign it on the device; it must land before slot {bound} (now {})", set.txs[i].n, live.slot());
                while !landed(live) {
                    assert!(live.slot() < bound, "{what}: it did not land before slot {bound}");
                    std::thread::sleep(Duration::from_secs(1));
                }
                by_the_page.push(Json::object(vec![("what", Json::string(what)), ("set", Json::string(rel)), ("n", Json::number(set.txs[i].n)), ("by", Json::string("his own Keystone"))]));
            }
        }
        assert!(landed(live), "{what}: the chain holds it");
    };

    // 1. THE FLOOR: inside the curve's window two stand-in people mint a seat each by the client's own builders; then the Keystone's owner
    // mints its own by the set `press seat` wrote from the issuer's permit for it
    let curve = MintCurve::decode(&live.data(&nft::curve_pda(ROOT_UNIT))).expect("root 0's curve");
    live.wait_until(curve.opens_slot);
    let cap = curve.start_lamports + curve.step_lamports * curve.capacity;
    let mut proofs = Vec::new();
    for (i, v) in people.iter().enumerate() {
        let n = i as u64 + 1;
        let permit = nft::permit_message(&curve, v.key, completion(n), LYSSAJUDRON, cap, curve.closes_slot).expect("a permit");
        let pair = nft::mint_instructions(&curve, &permit, issuer.sign(&permit).to_bytes(), LYSSAJUDRON, cap).expect("the client's mint");
        live.attempt_all(&pair, &v.key, &format!("person {n} mints one seat of root 0's floor")).unwrap_or_else(|e| panic!("person {n}'s mint: refused with {e:?}"));
        let mint = nft::mint_pda(ROOT_UNIT, &completion(n));
        proofs.push(OwnerProof { owner: v.key, mint, token: nft::initial_token_pda(&mint, &v.key), unit: ROOT_UNIT });
    }
    let permit = nft::permit_message(&curve, owner, completion(3), LYSSAJUDRON, cap, curve.closes_slot).expect("a permit");
    let b64 = |b: &[u8]| base64::engine::general_purpose::STANDARD.encode(b);
    let door = Json::object(vec![
        ("permit", Json::string(b64(&permit))),
        ("signature", Json::string(b64(&issuer.sign(&permit).to_bytes()))),
        ("metadata", Json::object(vec![("name", Json::string(LYSSAJUDRON.name)), ("symbol", Json::string(LYSSAJUDRON.symbol)), ("uri", Json::string(LYSSAJUDRON.uri))])),
    ]);
    std::fs::write(format!("{dir}/seat.ask"), format!("{owner}\n")).unwrap();
    std::fs::write(format!("{dir}/seat-permit.json"), door.pretty() + "\n").unwrap();
    ask_the_press("seat");
    let seat_mint = nft::mint_pda(ROOT_UNIT, &completion(3));
    let seat = nft::seat_pda(&seat_mint);
    keystone_signs(&mut live, &format!("{dir}/seats/{owner}.json"), 0, "the Keystone's owner mints its own seat of root 0's floor", curve.closes_slot, &|live: &Live| !live.data(&seat).is_empty());
    let floor = RegistryView::parse(&live.data(&registry_pda(ROOT_UNIT))).unwrap();
    assert_eq!((floor.holders.len(), floor.total_seats, floor.holders.iter().any(|h| h.holder == seat.to_bytes())), (3, 3, true), "root 0's floor: three seats, one a seat owner's whose key is a Keystone's");
    ask_the_press("ratify");
    live.wait_until(curve.closes_slot);
    ask_the_press("ratify");
    assert_eq!(ConfigView::parse(&live.data(&config_pda())).unwrap().mode, MODE_COUNCIL, "press ratify ratified root 0 once the curve's window closed");

    // 2. THE ACT: the press's first, proposed by the Keystone's owner and voted PRO by its seat, both through the page
    let act_file = match rec.get("acts") {
        Json::Arr(a) => a.iter().filter_map(Json::as_str).find(|f| !f.ends_with("index.json")).expect("the press's first act").to_string(),
        _ => panic!("the press wrote no acts"),
    };
    let (h, envelopes) = read_act(&Json::parse(&std::fs::read_to_string(&act_file).unwrap()).unwrap()).unwrap_or_else(|e| panic!("{act_file}: {e}"));
    std::fs::write(format!("{dir}/votes.ask"), format!("1 {owner} propose\n1 {} -\n", people[0].key)).unwrap();
    ask_the_press("votes");
    let mine = format!("{dir}/votes/1-{owner}.json");
    let window = live.slot() + 900;
    keystone_signs(&mut live, &mine, 0, "the Keystone's owner proposes the press's first act", window, &|live: &Live| !live.data(&proposal_pda(&h)).is_empty());
    let pro_before = live.session(&h, ROOT_UNIT).hex_pro;
    keystone_signs(&mut live, &mine, 1, "the Keystone's owner votes PRO by its seat", window, &|live: &Live| live.session(&h, ROOT_UNIT).hex_pro > pro_before);
    // THE RITE FREES THE AGENTS: unit 1's three agents read the act and vote it by their own minds (press chair, each its own Nest key and
    // the mind its rite names); these checks sign none of its envelopes and follow what the minds chose
    assert!(!envelopes.is_empty() && envelopes.iter().all(|e| agents.iter().any(|a| a.key == e.signer)), "the act's envelopes are unit 1's agents', and nobody else's");
    ask_the_press("chairs");
    let charter = Json::parse(&std::fs::read_to_string(env("PRESS_PARAMS")).expect("the sitting's params")).unwrap().get("posiverse").get("charter").as_str().expect("the params' posiverse.charter").to_lowercase();
    let minds: Vec<Chose> = agents.iter().enumerate().map(|(k, a)| the_agent_chose(&dir, k, &a.key, &charter)).collect();
    let h_hex = h.iter().map(|b| format!("{b:02x}")).collect::<String>();
    for (k, m) in minds.iter().enumerate() {
        assert_eq!(m.rows.iter().map(|r| (r.0, r.1.as_str())).collect::<Vec<_>>(), vec![(1, h_hex.as_str())], "agent {} ({}): one ballot, on the act the Keystone's owner proposed", k + 1, m.name);
        eprintln!("    agent {} ({}), chair {k}: {} on the press's first act: \"{}\"", k + 1, m.name, m.rows[0].2, m.rows[0].3);
    }
    let count = |w: &str| minds.iter().filter(|m| m.rows[0].2 == w).count() as u8;
    let tally = (count("PRO"), count("CONTRA"), count("ABSTAIN"));
    let why = minds.iter().enumerate().map(|(k, m)| format!("agent {} ({}) {}: \"{}\"", k + 1, m.name, m.rows[0].2, m.rows[0].3)).collect::<Vec<_>>().join("; ");
    assert!(tally.0 >= 2, "THE AGENTS' OWN MINDS DID NOT PASS the press's first act ({} PRO, {} CONTRA, {} ABSTAIN): {why}", tally.0, tally.1, tally.2);
    // NO KEY DECIDES ALONE: unit 1 passes it and is heard in its chair, the Keystone's seat is one of three, and root 0 does not seal
    ask_the_press("crank");
    let (unit, root, p) = (live.session(&h, POSIVERSE), live.session(&h, ROOT_UNIT), ProposalView::parse(&live.data(&proposal_pda(&h))).expect("its proposal"));
    assert_eq!((unit.state, (unit.pent_pro, unit.pent_contra, unit.pent_abst), root.state, root.hex_pro, p.executed), (STATE_PASSED, tally, STATE_VOTING, 1, false), "with the Keystone's seat alone on the floor (one of three) the act does not pass, whatever the agents' minds say");
    // a second seat owner's own PRO, the same road in Rust (person 1): two of three seats
    let theirs = format!("{dir}/votes/1-{}.json", people[0].key);
    let set = read_set(&Json::parse(&std::fs::read_to_string(&theirs).unwrap()).unwrap(), &["localnet"]).unwrap_or_else(|e| panic!("{theirs}: {e}"));
    live.sign_prepared(&set, &set.txs[0], "person 1's own PRO").unwrap_or_else(|e| panic!("person 1's PRO refused with {e:?}"));
    ask_the_press("crank-again");
    let (root, p) = (live.session(&h, ROOT_UNIT), ProposalView::parse(&live.data(&proposal_pda(&h))).expect("its proposal"));
    assert_eq!((root.state, root.pent_pro, root.hex_pro, p.executed), (STATE_PASSED, 1, 2, true), "two of three seats and unit 1 in its chair: sealed and executed by press crank");

    let certificate = Json::object(vec![
        ("what", Json::string("A SEAT OWNER'S KEYSTONE ON A LOCAL CHAIN: the press's first act passed on a validator of this machine, its proposal and one of its two floor votes signed through the Keystone page itself (poleis/ceremony/keystone), as its seat's own mint was")),
        ("validator", Json::object(vec![("rpc", Json::string(&url)), ("genesisHash", live.ask("getGenesisHash", "[]"))])),
        ("keystone", Json::object(vec![("owner", Json::string(owner.to_string())), ("by", Json::string(if stand_in.is_some() { "a stand-in key, held up to the page in a headless browser; no device" } else { "his own Keystone" })), ("seat", Json::string(seat.to_string()))])),
        ("act", Json::object(vec![("n", Json::number(1u64)), ("file", Json::string(act_file.rsplit('/').next().unwrap_or_default())), ("hash", Json::string(h.iter().map(|b| format!("{b:02x}")).collect::<String>()))])),
        ("throughThePage", Json::Arr(by_the_page)),
        ("thePageShowed", Json::Arr(shown.iter().map(Json::string).collect())),
        ("agents", Json::Arr(minds.iter().enumerate().map(|(k, m)| m.said(k, &agents[k].key)).collect())),
        ("alone", Json::string(format!("with the Keystone's seat PRO and the three agents' ballots, each its own mind's ({} PRO, {} CONTRA, {} ABSTAIN): unit 1 PASSED, root 0 still VOTING, not executed", tally.0, tally.1, tally.2))),
        ("passed", Json::string(format!("with a second seat owner's PRO: unit 1 PASSED ({} of its chairs PRO, each its agent's own mind's), root 0 PASSED (its chair, 2 of 3 seats), executed by press crank", tally.0))),
        ("transactions", Json::number(live.landed.len())),
        ("seconds", Json::number(begun.elapsed().as_secs())),
    ]);
    std::fs::write(&out, certificate.pretty() + "\n").unwrap_or_else(|e| panic!("{out}: {e}"));
    eprintln!("  A SEAT OWNER'S KEYSTONE ON A LOCAL CHAIN: ACT 1 executed; the mint, the proposal and the PRO of {owner} went through the page; the certificate {out}");
}

fn lamports_at(live: &Live, key: &Pubkey) -> u64 {
    live.account(key).get("lamports").as_u64().unwrap_or_default()
}

// ---- THE ROUTER DRAWS ITS MONTH (Nov 24 2026, lane/the-router-draws-its-month; his 00:07:00 UTC "Yes, build it" to "Build the router's own draw
// from the treasury next?", after his Nov 23 22:20:46 UTC "we might get some money in the treasury, but unless we figure out how to allocate it as
// credits.. u get me?") ----
// The door's own law (poleis/seat-wasm src/mine.rs, the bytes its WASM runs) asked as the door asks it, its one message's instructions sent as one
// transaction on the real programs: the mine counts the batch's spends and root 0's vault pays the miner, in the same transaction, what the AI
// provider charged for them, as far as the month, root 0's cap of the dollar and the vault let out; a month spent lets the batch go alone.

/// A world whose dollar is its default mint, as a real cluster's is (the launch press names USDC at init_config): only the default mint is
/// counted by root 0's own cap of the dollar, the counter the router's month draws under. Everything else is the world's own.
struct OnTheDefaultMint<'a> {
    w: &'a mut World,
    dollar: Pubkey,
}
impl Ledger for OnTheDefaultMint<'_> {
    fn attempt(&mut self, ix: &Instruction, what: &str) -> Result<u64, ProgramError> {
        self.w.attempt(ix, what)
    }
    fn attempt_all(&mut self, ixs: &[Instruction], payer: &Pubkey, what: &str) -> Result<u64, ProgramError> {
        self.w.attempt_all(ixs, payer, what)
    }
    fn data(&self, key: &Pubkey) -> Vec<u8> {
        World::data(self.w, key)
    }
    fn lamports(&self, key: &Pubkey) -> u64 {
        World::lamports(self.w, key)
    }
    fn slot(&self) -> u64 {
        World::slot(self.w)
    }
    fn wait_until(&mut self, slot: u64) {
        Ledger::wait_until(self.w, slot)
    }
    fn wallet(&mut self, lamports: u64) -> Pubkey {
        self.w.add_wallet(lamports)
    }
    fn voter(&mut self, lamports: u64) -> Voter {
        World::voter(self.w, lamports)
    }
    fn mint(&mut self) -> Pubkey {
        World::mint(self.w)
    }
    fn roads(&mut self) -> Real {
        Real::load(self.w)
    }
    fn warm(&mut self) -> Warm {
        Warm::load(self.w)
    }
    /// The default mint itself.
    fn dollar(&mut self) -> Pubkey {
        self.dollar
    }
    fn pay(&mut self, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Pubkey {
        Ledger::pay(self.w, mint, owner, amount)
    }
    fn clock_ms(&self) -> u64 {
        Ledger::clock_ms(self.w)
    }
}

/// A legacy message's instructions, as Solana's runtime reads them back: its header, its keys, each instruction's program, accounts and data.
fn decompiled(b: &[u8]) -> Vec<Instruction> {
    // Solana's compact length: seven bits a byte, the high bit saying another follows
    fn compact(b: &[u8], at: &mut usize) -> usize {
        let (mut v, mut shift) = (0usize, 0);
        loop {
            let x = b[*at];
            *at += 1;
            v |= ((x & 0x7f) as usize) << shift;
            if x & 0x80 == 0 {
                return v;
            }
            shift += 7;
        }
    }
    let (signers, ro_signed, ro_unsigned) = (b[0] as usize, b[1] as usize, b[2] as usize);
    let mut at = 3;
    let n = compact(b, &mut at);
    let keys: Vec<Pubkey> = (0..n).map(|i| Pubkey::new_from_array(b[at + 32 * i..at + 32 * i + 32].try_into().unwrap())).collect();
    let writable = |i: usize| if i < signers { i < signers - ro_signed } else { i < n - ro_unsigned };
    at += 32 * n + 32;
    let mut out = Vec::new();
    for _ in 0..compact(b, &mut at) {
        let program = b[at] as usize;
        at += 1;
        let accounts = compact(b, &mut at);
        let metas = b[at..at + accounts].iter().map(|i| AccountMeta { pubkey: keys[*i as usize], is_signer: (*i as usize) < signers, is_writable: writable(*i as usize) }).collect();
        at += accounts;
        let len = compact(b, &mut at);
        out.push(Instruction::new_with_bytes(keys[program], &b[at..at + len], metas));
        at += len;
    }
    assert_eq!(at, b.len(), "the whole message read");
    out
}

/// The router's oracle of this scenario (a fixed test key nobody else holds) and one receipt it signs in the router's canonical shape, naming what
/// the provider charged.
fn router_receipt(id: u64, at_ms: u64, nanos: u64, tank: &str, router_usd: &str) -> Json {
    let k = SigningKey::from_bytes(&[42; 32]);
    let usd = nanos as f64 / 1e9;
    let unsigned = format!("{{\"at\":{at_ms},\"by\":\"{}\",\"nonce\":\"n{id}\",\"router_usd\":{router_usd},\"tank\":\"{tank}\",\"usd\":{usd},\"v\":1}}", k.verifying_key().as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>());
    let sig = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(k.sign(unsigned.as_bytes()).to_bytes());
    let body = unsigned.replace(",\"tank\"", &format!(",\"sig\":\"{sig}\",\"tank\""));
    Json::object(vec![("id", Json::number(id)), ("at", Json::number(at_ms)), ("nanos", Json::number(nanos)), ("tank", Json::string(tank)), ("body", Json::string(body))])
}

/// What the scenario ends with: each batch's units, the draws, and the month's and the cap's counts.
#[derive(Debug)]
struct MonthBook {
    units: Vec<(String, u64)>,
    drawn: Vec<u64>,
    spent: u64,
    capped: u64,
}

/// THE ROUTER DRAWS ITS MONTH on the real programs, after THE THIRD PROGRAM in the same world: the polis caps the dollar (CODEX_PLAN) and gives the
/// mine's miner a month of it (a TREASURY allowance, as the launch's acts 7 and 8 do); a flask deposits; then five batches of the door's own
/// crank, each one transaction from the law's own message: alone (no month named), the first draw (the miner's dollar account made with it), a
/// draw the month cuts, a batch alone (the month spent), and after the month rolls a draw root 0's cap cuts.
fn the_router_draws_its_month<L: Ledger>(w: &mut L, crank: Pubkey, proposer: &Voter, and: &[(&Voter, u8)]) -> MonthBook {
    use poleis_seat::mine;
    let warm = w.warm();
    let m = w.data(&warm.mine());
    let at = |o: usize| Pubkey::new_from_array(m[o..o + 32].try_into().unwrap());
    let (treasury, dollar, miner) = (at(34), at(66), at(98));
    assert_eq!((treasury, RegistryView::parse(&w.data(&registry_pda(ROOT_UNIT))).expect("root 0's registry").default_mint), (vault_pda(ROOT_UNIT, &dollar), dollar), "the mine's treasury is root 0's vault of its dollar, the default mint");
    let mut nonce = 60u64;
    let mut sealed = |w: &mut L, act: &Payload, what: &str| -> [u8; 32] {
        nonce += 1;
        let h = proposal_hash(act.class(), ROOT_UNIT, &MANIFEST, &act.hash(), nonce);
        w.done(propose_raw(proposer.key, h, act.class(), ROOT_UNIT, MANIFEST, &act.bytes(), &[ROOT_UNIT]), "propose the act");
        w.cast(&crank, &h, ROOT_UNIT, and);
        w.done(seal(crank, h, ROOT_UNIT), "seal the act");
        w.done(execute(crank, h, &[ROOT_UNIT], act), what);
        h
    };
    // 1. root 0's cap of the dollar, its plan restated (act 7); the miner's month (act 8): five dollars a thousand slots, no expiry
    let r = RegistryView::parse(&w.data(&registry_pda(ROOT_UNIT))).unwrap();
    let (cap, month_cap, period) = (8 * USD, 5 * USD, 1_000u64);
    sealed(w, &Payload::Plan { origin: ROOT_UNIT, cap: r.plan_cap, period: r.plan_period, token_cap: cap }, "root 0 caps the dollar");
    let h = sealed(w, &Payload::Allowance { origin: ROOT_UNIT, payee: miner.to_bytes(), payee_kind: HOLDER_KEY, mint: dollar, cap: month_cap, period, expires: 0 }, "root 0 gives the miner its month");
    let month = allowance_pda(&h);
    let start = AllowanceView::parse(&w.data(&month)).expect("the month").start;
    // 2. a flask of fifty dollars
    let flask = w.wallet(SOL);
    let source = w.pay(&dollar, &flask, 50 * USD);
    w.done(warm.deposit(flask, flask, &source, &treasury, &dollar, 50 * USD), "the flask deposits fifty dollars");
    let oracle: String = SigningKey::from_bytes(&[42; 32]).verifying_key().as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    let law = |v: Json| -> Json {
        let said = Json::parse(&mine::ask(&v.text())).unwrap();
        assert!(said.get("error").is_null(), "the law, asked {}: {}", v.text(), said.text());
        said
    };
    let node = |w: &L, k: &Pubkey| {
        let d = w.data(k);
        if d.is_empty() && w.lamports(k) == 0 { Json::Null } else { Json::object(vec![("data", Json::Arr(vec![Json::string(base64::engine::general_purpose::STANDARD.encode(&d)), Json::string("base64")])), ("lamports", Json::number(w.lamports(k)))]) }
    };
    let dest = mine::dollar_account_of(&miner, &dollar);
    let (vault_of, held_of) = (|w: &L| w.u64_at(&treasury, 64), |w: &L| if w.data(&dest).is_empty() { 0 } else { w.u64_at(&dest, 64) });
    let counts = |w: &L| (AllowanceView::parse(&w.data(&month)).unwrap().spent, RegistryView::parse(&w.data(&registry_pda(ROOT_UNIT))).unwrap().token_plan_spent);
    // 3. the batches: the crank's pass as the door runs it (`wants`, the accounts read, `plan`), its message's instructions one transaction
    let (mut id, mut ms, clock_ms) = (u64::from_le_bytes(m[202..210].try_into().unwrap()), 0u64, w.clock_ms());
    let (mut units, mut drawn_all) = (Vec::new(), Vec::new());
    let mut batch = |w: &mut L, spends: &[(u64, &str)], named: bool, what: &str| -> (u64, Json) {
        let rows: Vec<Json> = spends
            .iter()
            .map(|(nanos, router)| {
                (id, ms) = (id + 1, ms + 1_000);
                let mut r = router_receipt(id, clock_ms - 100_000 + ms, *nanos, "st-the-month", router);
                if let Json::Obj(o) = &mut r {
                    o.push(("flasks".into(), Json::string(flask.to_string())));
                }
                r
            })
            .collect();
        let mine_node = node(w, &warm.mine());
        let mut ask = vec![("do", Json::string("wants")), ("rows", Json::Arr(rows.clone())), ("mine", mine_node.clone())];
        if named {
            ask.push(("month", Json::string(month.to_string())));
        }
        let wants = law(Json::object(ask));
        let records = match wants.get("records") {
            Json::Obj(o) => Json::Obj(o.iter().map(|(k, _)| (k.clone(), node(w, &Pubkey::from_str(k).unwrap()))).collect()),
            _ => panic!("no records"),
        };
        let mut clock = w.slot().to_le_bytes().to_vec();
        clock.extend_from_slice(&[0u8; 24]);
        clock.extend_from_slice(&((w.clock_ms() / 1_000) as i64).to_le_bytes());
        let clock = Json::object(vec![("data", Json::Arr(vec![Json::string(base64::engine::general_purpose::STANDARD.encode(&clock)), Json::string("base64")]))]);
        let mut ask = vec![("do", Json::string("plan")), ("mine", mine_node), ("clock", clock), ("rows", Json::Arr(rows)), ("records", records), ("oracle", Json::string(oracle.clone())), ("blockhash", Json::string(Pubkey::new_from_array([9; 32]).to_string()))];
        if named {
            ask.push(("month", Json::string(month.to_string())));
        }
        let plan = law(Json::object(ask));
        let message = base64::engine::general_purpose::STANDARD.decode(plan.get("message").as_str().expect("a batch")).unwrap();
        assert!(1 + 64 + message.len() <= PACKET && message[0] == 1, "{what}: one packet, one signer");
        let ixs = decompiled(&message);
        let cu = w.attempt_all(&ixs, &miner, what).unwrap_or_else(|e| panic!("{what}: refused with {e:?}"));
        units.push((what.to_string(), cu));
        eprintln!("  {what}: {} instructions, {} bytes on the wire, {cu} CU, draw {} ({})", ixs.len(), 1 + 64 + message.len(), plan.get("draw").text(), plan.get("draw_why").text());
        (cu, plan)
    };
    let spends_of = |w: &L| u64::from_le_bytes(w.data(&warm.mine())[218..226].try_into().unwrap());

    // A. no month named: the batch alone, as before this lane (the flask's $WARM account made); what the provider charged for it is not carried
    let (vault, spends) = (vault_of(w), spends_of(w));
    let (_, p) = batch(w, &[(1_050_000_000, "1")], false, "a batch, no month named");
    assert_eq!((p.get("draw").is_null(), vault_of(w), held_of(w), spends_of(w)), (true, vault, 0, spends + 1), "alone: the mine counted it, nothing left the vault");
    // B. the first draw: two spends the provider charged three dollars for, the miner's dollar account made in the same transaction
    let (vault, spends) = (vault_of(w), spends_of(w));
    let (first, p) = batch(w, &[(2_100_000_000, "2"), (1_050_000_000, "1")], true, "the first draw, the miner's dollar account made with it");
    let drawn = p.get("draw").as_u64().unwrap();
    assert_eq!(drawn, 3 * USD, "what the provider charged: the door's markup stays in the treasury");
    assert_eq!((vault_of(w), held_of(w), counts(w), spends_of(w)), (vault - drawn, drawn, (drawn, drawn), spends + 1), "root 0's vault fell by exactly the draw, the miner's account rose by it, the month and the cap count it, the mine counted the spends (one entry: the flask's two of one era)");
    drawn_all.push(drawn);
    // C. a draw the month cuts: four dollars charged, two left of the month
    let vault = vault_of(w);
    let (later, p) = batch(w, &[(4_200_000_000, "4")], true, "a draw past the month, cut to what it has left");
    let cut = p.get("draw").as_u64().unwrap();
    assert_eq!((cut, vault_of(w), held_of(w), counts(w)), (month_cap - drawn, vault - cut, drawn + cut, (month_cap, drawn + cut)), "cut to the month: what is left undrawn is not carried");
    drawn_all.push(cut);
    // D. the month spent: the batch lands alone, and says why
    let (vault, spends) = (vault_of(w), spends_of(w));
    let (alone, p) = batch(w, &[(1_050_000_000, "1")], true, "a batch with the month spent, alone");
    assert_eq!((p.get("draw").as_u64(), p.get("draw_why").as_str(), vault_of(w), held_of(w), spends_of(w)), (Some(0), Some("spent"), vault, month_cap, spends + 1), "THE DRAW NEVER HOLDS THE MINE: the batch landed, nothing drawn");
    // E. the month rolled (the allowance's own period, rolled by the program at the slot): five dollars charged, five of the month, three left of root 0's cap
    w.wait_until(start + period);
    let vault = vault_of(w);
    let (_, p) = batch(w, &[(5_250_000_000, "5")], true, "a draw in the next month, cut to root 0's cap of the dollar");
    let capped = p.get("draw").as_u64().unwrap();
    assert_eq!((capped, vault_of(w), counts(w)), (cap - month_cap, vault - capped, (capped, cap)), "a new month, cut to what root 0's cap has left");
    drawn_all.push(capped);
    // F. the cap spent: alone
    let (_, p) = batch(w, &[(1_050_000_000, "1")], true, "a batch with root 0's cap spent, alone");
    assert_eq!(p.get("draw_why").as_str(), Some("cap"));
    assert_eq!(held_of(w), drawn_all.iter().sum::<u64>(), "the miner holds every draw, and nothing else");
    let (spent, capped) = counts(w);
    eprintln!(
        "  THE ROUTER DRAWS ITS MONTH: draws {drawn_all:?} micro-dollars; the pull and its counts {} CU inside a batch (a batch with the draw less one alone, one entry each), the miner's dollar account made {} CU more",
        later.saturating_sub(alone),
        first.saturating_sub(later)
    );
    MonthBook { units, drawn: drawn_all, spent, capped }
}

#[test]
fn the_router_draws_its_month_in_the_batch_that_mines_on_the_real_programs() {
    let Estate { mut w, crank, founders, alice, bob, usdc, .. } = estate(false);
    // the default mint, six decimals as USDC has (the estate's has none: no flask could deposit a cent of it), minted by no one
    w.accounts.iter_mut().find(|(k, _)| *k == usdc).expect("the default mint").1.data[44] = 6;
    w.done(endow(crank, ROOT_UNIT, 5 * SOL), "endow the root");
    let and = and_votes(&founders, &alice, &bob);
    let mut on = OnTheDefaultMint { w: &mut w, dollar: usdc };
    the_mine(&mut on, crank, &alice, &and);
    let book = the_router_draws_its_month(&mut on, crank, &alice, &and);
    assert_eq!((book.drawn, book.spent, book.capped, book.units.len()), (vec![3 * USD, 2 * USD, 3 * USD], 3 * USD, 8 * USD, 6), "three draws in six batches: {:?}", book.units);
}

/// THE STAKE EARNS (Nov 25 2026, lane/the-stake-earns; poleis/press/THE-STAKE-EARNS-LANDING.md): the press's ninth act, the stake pool's
/// SetStaker by the polis's hand, carried through root 0's vote and executed by Poleis's own build, the stake pool program the mainnet bytes the
/// door's rehearsal dumped (ENGINE_STAKE_POOL_SO; without it this says so and passes). The hand signs the SetStaker only inside the sealed
/// ACT_RAW's execute: the pool's staker after it is the engine's key, its manager still the hand; no key can do it outside the vote.
#[test]
fn the_ninth_act_names_her_pools_staker_by_the_hands_sealed_raw_act() {
    let Ok(so) = std::env::var("ENGINE_STAKE_POOL_SO") else {
        eprintln!("ENGINE_STAKE_POOL_SO is not set: the ninth act on the stake pool program's own bytes is not run here");
        return;
    };
    let elf = std::fs::read(&so).unwrap_or_else(|e| panic!("{so}: {e}"));
    let stake_pool = Pubkey::from_str_const("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");
    let mut e = estate(false);
    e.w.mollusk.add_program_with_loader_and_elf(&stake_pool, &mollusk_svm::program::loader_keys::LOADER_V3, &elf);
    e.w.accounts.push((stake_pool, mollusk_svm::program::create_program_account_loader_v3(&stake_pool)));
    let agent = agent_pda(ROOT_UNIT);
    // her pool as act 5 leaves it: a StakePool (its account type 1), its manager and its staker the polis's hand (pool.rs pool_held's offsets)
    let pool = Pubkey::new_unique();
    let mut data = vec![0u8; 611];
    data[0] = 1;
    data[1..33].copy_from_slice(agent.as_ref());
    data[33..65].copy_from_slice(agent.as_ref());
    let lamports = e.w.rent(611);
    e.w.accounts.push((pool, Account { lamports, data, owner: stake_pool, executable: false, rent_epoch: 0 }));
    let staker_of = |w: &World| {
        let d = &w.accounts.iter().find(|(k, _)| *k == pool).expect("her pool").1.data;
        (Pubkey::new_from_array(d[1..33].try_into().unwrap()), Pubkey::new_from_array(d[33..65].try_into().unwrap()))
    };
    let engine = Pubkey::new_unique();
    // no key names a staker outside the vote: alice's own SetStaker, signed by her, is refused by the stake pool program
    let alice = e.alice.key;
    let direct = Instruction::new_with_bytes(stake_pool, &[13], vec![AccountMeta::new(pool, false), AccountMeta::new_readonly(alice, true), AccountMeta::new_readonly(alice, false)]);
    e.w.run(direct, &[]);
    assert_eq!(staker_of(&e.w), (agent, agent), "refused (a landed SetStaker would have named alice): the pool unmoved");
    // the ninth act: proposed, voted, sealed, executed
    let act = Payload::Raw { origin: ROOT_UNIT, program: stake_pool, accounts: vec![(pool, ACT_WRITABLE), (agent, ACT_SIGNER), (engine, 0)], data: vec![13] };
    let h = carry(&mut e, &act, 9);
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &act), &[ok()]);
    assert_eq!(staker_of(&e.w), (agent, engine), "its manager the hand, its staker the engine's key");
    e.w.run(execute(e.crank, h, &[ROOT_UNIT], &act), &[refuse(PoleisError::AlreadyExecuted)]);
    eprintln!("  the ninth act executed by Poleis on the stake pool program's bytes ({} bytes of {so}): the pool's staker {engine}, its manager the hand {agent}", elf.len());
}
