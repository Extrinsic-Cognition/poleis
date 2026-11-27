//! Seeded chaos under the double seal: a ratified society takes a
//! deterministic stream of random legal and hostile acts (proposals of every
//! class, signed votes landed through the Ed25519 precompile, crews opening
//! their own votes, mandates, seals, executions, dividends, seat sales and
//! transfers, time jumps) and
//! the registry, session and money invariants are machine-checked after
//! EVERY op:
//!
//!   S1  seat conservation per registry: total_seats == Σ holder seats,
//!       no duplicate holder, no holder over the cap, count within capacity
//!   S2  the root ledger: total_seats == seats claimed on root sales − seats
//!       jetted (seats never appear from nowhere)
//!   S3  seat sale sanity: board prices inside [reserve, deposit]; a settled
//!       clearing price inside [reserve, best]; filled ≤ seat_count
//!   S4  seat bid escrow exact: rent + deposit while live, rent once done
//!   S5  the constitution: mode stays council, authority stays the PDA
//!   S6  twelve chairs always; a chair bound to a unit names a registry that
//!       calls this unit its parent
//!   S7  the seal: PASSED only with a majority of the chairs lit at the
//!       session's opening (its roster; at the seal for a session with none)
//!       and of the active floor (or no floor at all); EXPIRED only with
//!       zero votes; REJECTED never with that majority of the chairs
//!   S8  no execution without the origin's session PASSED (no proposal in
//!       the season asks for consent; a crew's seal is its chair's vote)
//!   S10 a ballot box holds its rent and nothing more, and carries its
//!       roster; one a polis opened itself (a unit its proposal did not
//!       list) names who paid it, and goes home to them
//!   S9  the plan: never spent past its cap
//!   D1  a distribution never pays past its amount, and holds exactly what
//!       is still owed
//!   I1  treasury conservation (no donations this season: strict)
//!
//! Plus refusals asserted every single time they are attempted: the dead
//! payer key minting a vault, a seatless stranger proposing, a forged vote,
//! and an attest with no precompile in front of it.

mod built;
use ed25519_dalek::{Signer, SigningKey};
use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
use poleis_client::{
    attest, attest_batch, claim_dividend, claim_seat, close_proposal, close_seat_bid,
    commit_seat_bid, config_pda, council_pda, create_treasury, endow, execute, found_child,
    adjourn, convene, holder_unit, init_config, mandate, open_genesis, HOLDER_EMPTY,
    proposal_hash, proposal_pda, propose, ratify_genesis, reclaim_seat_bid, registry_pda, reveal_seat_bid,
    sale_pda, seal, seat_commitment, session_pda, settle_seats, sync_endowment, transfer_seats,
    unit_holder, vote_envelope, ConfigView, DistributionView,
    FoundSpec, GenesisSpec, Payload, PoleisError, ProposalView, RegistryView, SaleView, SeatBid,
    SeatBidView, Session, SessionView, Treasury, TreasuryView, CLASS_CODEX, HOLDER_KEY, HOLDER_UNIT,
    INSTRUCTIONS_SYSVAR_ID, MAX_HOLDERS, MAX_IMPACTED, MAX_SEATS_PER_HOLDER, MODE_COUNCIL,
    chair_quorum, PENTAGONS, PROGRAM_ID, ROOT_UNIT, STATE_EXPIRED, STATE_PASSED,
    STATE_REJECTED, VOTE_ABSTENTIO, VOTE_CONTRA, VOTE_PRO,
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use std::collections::HashMap;

const SOL: u64 = 1_000_000_000;
const OPS: usize = 700;
const SEED: u64 = 0x0715_1204_2026_0902;
const WINDOW: u64 = 800;
const MANIFEST: [u8; 32] = [9u8; 32];

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len() as u64) as usize]
    }
    fn salt(&mut self) -> [u8; 32] {
        let mut s = [0u8; 32];
        for c in s.chunks_mut(8) {
            c.copy_from_slice(&self.next().to_le_bytes());
        }
        s
    }
    /// A society that mostly agrees with itself, so seals actually close:
    /// three quarters PRO, one fifth CONTRA, the rest present and silent.
    fn choice(&mut self) -> u8 {
        match self.below(20) {
            0..=14 => VOTE_PRO,
            15..=18 => VOTE_CONTRA,
            _ => VOTE_ABSTENTIO,
        }
    }
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

struct World {
    mollusk: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
    accepted: usize,
    refused: usize,
    seed: u8,
    /// The chairs' quorum each session sealed under: its roster's chairs lit at the opening, or, for a session with no roster, the chairs lit when it is first seen sealed (the op that sealed it moved no chair).
    sealed_quorum: std::cell::RefCell<HashMap<Pubkey, u8>>,
}

impl World {
    fn new() -> Self {
        std::env::set_var(
            "SBF_OUT_DIR",
            built::program_dir(),
        );
        Self {
            mollusk: Mollusk::new(&PROGRAM_ID, "poleis_treasury"),
            accounts: vec![keyed_account_for_system_program()],
            accepted: 0,
            refused: 0,
            seed: 0,
            sealed_quorum: std::cell::RefCell::new(HashMap::new()),
        }
    }
    fn add_wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        key
    }
    fn voter(&mut self, lamports: u64) -> Voter {
        self.seed += 1;
        let mut seed = [0u8; 32];
        seed[0] = self.seed;
        seed[1] = 0xC0;
        let sk = SigningKey::from_bytes(&seed);
        let key = Pubkey::new_from_array(sk.verifying_key().to_bytes());
        self.accounts.push((key, Account::new(lamports, 0, &Pubkey::default())));
        Voter { sk, key }
    }
    fn touch(&mut self, key: Pubkey) {
        if key == INSTRUCTIONS_SYSVAR_ID {
            return;
        }
        if !self.accounts.iter().any(|(k, _)| *k == key) {
            self.accounts.push((key, Account::default()));
        }
    }
    fn data(&self, key: &Pubkey) -> Option<&[u8]> {
        self.accounts.iter().find(|(k, _)| k == key).map(|(_, a)| a.data.as_slice())
    }
    fn lamports(&self, key: &Pubkey) -> u64 {
        self.accounts.iter().find(|(k, _)| k == key).map(|(_, a)| a.lamports).unwrap_or(0)
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
    fn run_free(&mut self, ixn: Instruction) -> bool {
        for meta in &ixn.accounts {
            self.touch(meta.pubkey);
        }
        let result = self.mollusk.process_and_validate_instruction(&ixn, &self.accounts, &[]);
        if result.program_result.is_err() {
            self.refused += 1;
            return false;
        }
        let resulting = result.resulting_accounts.clone();
        self.merge(&resulting);
        self.accepted += 1;
        true
    }
    /// A whole transaction (the precompile plus attest), no expectations.
    fn run_tx_free(&mut self, ixs: &[Instruction], payer: &Pubkey) -> bool {
        for ixn in ixs {
            for meta in &ixn.accounts {
                self.touch(meta.pubkey);
            }
        }
        let result = self.mollusk.process_transaction_instructions(ixs, &self.accounts, Some(payer));
        if result.raw_result.is_err() {
            self.refused += 1;
            return false;
        }
        let resulting = result.resulting_accounts.clone();
        self.merge(&resulting);
        self.accepted += 1;
        true
    }
    fn must(&mut self, ixn: Instruction) {
        assert!(self.run_free(ixn), "bootstrap step refused");
    }
    fn floor(&self, size: usize) -> u64 {
        self.mollusk.sysvars.rent.minimum_balance(size)
    }
    fn registry(&self, unit: u64) -> Option<RegistryView> {
        self.data(&registry_pda(unit)).and_then(RegistryView::parse)
    }
    fn sale(&self, unit: u64, id: u64) -> Option<SaleView> {
        self.data(&sale_pda(unit, id)).and_then(SaleView::parse)
    }
    fn session(&self, hash: &[u8; 32], unit: u64) -> Option<SessionView> {
        self.data(&session_pda(hash, unit)).and_then(SessionView::parse)
    }
    /// The connected subtree rooted at `origin`, bound through chairs.
    fn subtree(&self, origin: u64) -> Vec<u64> {
        let mut out = vec![origin];
        let mut i = 0;
        while i < out.len() && out.len() < MAX_IMPACTED {
            if let Some(r) = self.registry(out[i]) {
                for p in &r.pentagons {
                    if p.kind == HOLDER_UNIT {
                        let child = holder_unit(&p.holder);
                        if !out.contains(&child) && out.len() < MAX_IMPACTED {
                            out.push(child);
                        }
                    }
                }
            }
            i += 1;
        }
        out
    }

    fn assert_invariants(&self, op: usize, claimed_root: u64, jetted_root: u64) {
        // S5: the constitution never regresses
        let cfg = ConfigView::parse(self.data(&config_pda()).unwrap()).unwrap();
        assert_eq!(cfg.mode, MODE_COUNCIL, "op {op}: mode regressed");
        assert_eq!(cfg.authority, council_pda(), "op {op}: authority regressed");

        for (key, a) in &self.accounts {
            if a.owner != PROGRAM_ID || a.data.is_empty() {
                continue;
            }
            let d = a.data.as_slice();
            if let Some(r) = RegistryView::parse(d) {
                // S1: seat conservation per registry
                let sum: u64 = r.holders.iter().map(|h| h.seats).sum();
                assert_eq!(r.total_seats, sum, "op {op}: registry {key} total != Σ holders");
                assert!(r.holders.len() <= MAX_HOLDERS, "op {op}: registry over capacity");
                // S6: a registry holds exactly the rows it has grown, never fewer than its holders
                assert!(r.holders.len() <= r.capacity && r.capacity <= MAX_HOLDERS, "op {op}: rows {} for {} holders", r.capacity, r.holders.len());
                for (i, h) in r.holders.iter().enumerate() {
                    // a leaver's row stays as a tombstone: empty, weightless, in place
                    if h.kind == HOLDER_EMPTY {
                        assert!(h.seats == 0 && h.weight == 0, "op {op}: a tombstone with seats or weight");
                        continue;
                    }
                    assert!(h.seats >= 1 && h.seats <= MAX_SEATS_PER_HOLDER, "op {op}: holder seat bounds");
                    assert!(h.weight >= 1, "op {op}: a holder with no weight");
                    assert!(
                        !r.holders[..i].iter().any(|g| g.kind == h.kind && g.holder == h.holder),
                        "op {op}: duplicate holder in registry {key}"
                    );
                }
                // S6: twelve chairs, bound children call this unit parent
                assert_eq!(r.pentagons.len(), PENTAGONS, "op {op}: not twelve chairs");
                for (k, p) in r.pentagons.iter().enumerate() {
                    if p.kind == HOLDER_UNIT {
                        let child = self.registry(holder_unit(&p.holder)).unwrap_or_else(|| panic!("op {op}: chair {k} of {} binds a unit with no registry", r.unit));
                        assert_eq!(child.parent, r.unit, "op {op}: bound child names another parent");
                        assert_eq!(child.chair_in_parent as usize, k, "op {op}: bound child names another chair");
                    }
                }
                // S9: the plan
                assert!(r.plan_spent <= r.plan_cap, "op {op}: plan breached in registry {key}");
                // S2: the root ledger
                if r.unit == ROOT_UNIT {
                    assert_eq!(
                        r.total_seats,
                        claimed_root - jetted_root,
                        "op {op}: root seats {} != claimed {claimed_root} - jetted {jetted_root}",
                        r.total_seats
                    );
                }
            } else if let Some(s) = SaleView::parse(d) {
                // S3: sale sanity
                assert!(s.board.len() as u64 <= s.seat_count, "op {op}: board overfilled");
                for (p, _, _) in &s.board {
                    assert!(*p >= s.reserve && *p <= s.deposit, "op {op}: board price outside law");
                }
                if s.settled {
                    let best = s.board.iter().map(|(p, _, _)| *p).max().unwrap_or(s.reserve);
                    assert!(
                        s.clearing >= s.reserve && s.clearing <= best.max(s.reserve),
                        "op {op}: sale {key} cleared at {} outside [{}, {best}]",
                        s.clearing,
                        s.reserve
                    );
                }
            } else if let Some(b) = SeatBidView::parse(d) {
                // S4: seat escrow exact
                let expect = if b.done { self.floor(SeatBid::SIZE) } else { self.floor(SeatBid::SIZE) + b.deposit };
                assert_eq!(a.lamports, expect, "op {op}: seat bid {key} escrow drift");
            } else if let Some(t) = TreasuryView::parse(d) {
                // I1: strict conservation, no donations this season
                assert_eq!(
                    a.lamports - self.floor(Treasury::SIZE),
                    t.total_in - t.total_out,
                    "op {op}: treasury {key} broke conservation"
                );
            } else if let Some(s) = SessionView::parse(d) {
                // S10: the ballot box's rent
                assert_eq!(a.lamports, self.floor(d.len()), "op {op}: session {key} holds more or less than its rent");
                let body = Session::body_len(d);
                assert!(Session::valid_data_len(d), "op {op}: session {key} of {} bytes is no shape", d.len());
                assert_eq!(s.convener.is_some(), body + Session::CONVENER_TAIL == d.len(), "op {op}: session {key} of {} bytes, convener {:?}", d.len(), s.convener);
                assert!(Session::roster(d).unwrap().is_some(), "op {op}: session {key} opened without its roster");
                if let Some(p) = self.data(&proposal_pda(&s.hash)).and_then(ProposalView::parse) {
                    assert_eq!(s.convener.is_some(), !p.impacted.contains(&s.unit), "op {op}: session {key}: convened iff its proposal did not list its unit");
                }
                // S7: the seal
                assert!(s.pent_pro as usize + s.pent_contra as usize + s.pent_abst as usize <= PENTAGONS, "op {op}: more chair voices than chairs");
                let quorum = if s.state == poleis_client::STATE_VOTING { 0 } else { *self.sealed_quorum.borrow_mut().entry(*key).or_insert_with(|| {
                    // who is counted is who stood there when the vote opened
                    if let Some((lit, _, _)) = Session::roster(d).unwrap() {
                        return chair_quorum(lit.count_ones() as u8);
                    }
                    let r = RegistryView::parse(self.data(&registry_pda(s.unit)).unwrap()).unwrap();
                    let now = self.mollusk.sysvars.clock.slot;
                    chair_quorum(r.pentagons.iter().filter(|c| c.kind != HOLDER_EMPTY && (c.term_end == 0 || now <= c.term_end)).count() as u8)
                }) };
                match s.state {
                    STATE_PASSED => {
                        assert!(s.pent_pro >= quorum, "op {op}: session {key} passed without a majority of the lit chairs");
                        assert!(
                            s.active_weight == 0 || (s.hex_pro as u128) * 2 > s.active_weight as u128,
                            "op {op}: session {key} passed against the floor"
                        );
                    }
                    STATE_EXPIRED => assert_eq!(s.votes, 0, "op {op}: session {key} expired with votes"),
                    STATE_REJECTED => assert!(s.pent_pro < quorum, "op {op}: session {key} rejected with a majority of the lit chairs"),
                    _ => {}
                }
            } else if let Some(p) = ProposalView::parse(d) {
                // S8: no execution without the origin's seal, nor without
                // every seal under consent (a closed proposal's sessions are
                // gone; the executed flag is the record)
                if p.executed {
                    for u in p.impacted.iter().filter(|u| **u == p.origin || p.consent) {
                        if let Some(s) = self.session(&p.hash, *u) {
                            assert_eq!(s.state, STATE_PASSED, "op {op}: executed proposal {key} with unit {u} not PASSED");
                        }
                    }
                }
            } else if let Some(x) = DistributionView::parse(d) {
                // D1: a distribution holds exactly what it still owes
                assert!(x.paid <= x.amount, "op {op}: distribution {key} overpaid");
                assert_eq!(a.lamports, self.floor(d.len()) + x.amount - x.paid, "op {op}: distribution {key} escrow drift");
            }
        }
    }
}

#[derive(Clone, Copy)]
struct SaleModel {
    unit: u64,
    id: u64,
    reserve: u64,
    deposit: u64,
}

#[derive(Clone, Copy)]
struct CommitModel {
    unit: u64,
    sale: u64,
    who: Pubkey,
    nonce: u64,
    price: u64,
    salt: [u8; 32],
}

struct ProposalModel {
    hash: [u8; 32],
    payload: Payload,
    impacted: Vec<u64>,
    /// The crews that opened their own vote of it, the crank paying.
    convened: Vec<u64>,
    founders: Vec<Voter>,
    proposer: Pubkey,
    executed: bool,
    closed: bool,
}

impl ProposalModel {
    /// Every unit holding a session of it: opened with it, or by itself.
    fn units(&self) -> Vec<u64> {
        self.impacted.iter().chain(self.convened.iter()).copied().collect()
    }
}

#[test]
fn seeded_council_season() {
    let mut rng = Rng(SEED);
    let mut convenings = 0usize;
    let mut silences = 0usize;
    let mut w = World::new();
    let payer = w.add_wallet(100 * SOL);
    let crank = w.add_wallet(100 * SOL);
    let stranger = w.voter(10 * SOL);
    // the hexagon: alice 3, bob 2, carol 1, dave none (a buyer on later sales)
    let actors: Vec<Voter> = (0..4).map(|_| w.voter(50 * SOL)).collect();

    // ---- genesis: eleven founders, chair 11 free; six seats sold at reserve ----
    w.must(init_config(payer, Pubkey::default()));
    let root_founders: Vec<Voter> = (0..11).map(|_| w.voter(SOL)).collect();
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
        founders: root_founders.iter().map(|f| f.key).collect(),
        name: b"THE SEASON".to_vec(),
        desc: b"a seeded society".to_vec(),
    };
    w.must(open_genesis(payer, &g));
    // a first child in genesis, eleven founders of its own
    let one_founders: Vec<Voter> = (0..11).map(|_| w.voter(SOL)).collect();
    let spec = FoundSpec {
        child: 1,
        chair: 11,
        charter: [1u8; 32],
        window: WINDOW,
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        founders: one_founders.iter().map(|f| f.key).collect(),
        name: b"UNIT ONE".to_vec(),
        desc: Vec::new(),
    };
    w.must(found_child(payer, ROOT_UNIT, &spec));
    let mut plans = Vec::new();
    for (i, a) in actors.iter().enumerate() {
        for nonce in 0..(3 - i.min(3)) as u64 {
            let price = 1_000_000 + rng.below(4_000_000);
            let salt = rng.salt();
            w.must(commit_seat_bid(a.key, ROOT_UNIT, 0, nonce, seat_commitment(price, nonce, &salt, &a.key)));
            plans.push(CommitModel { unit: ROOT_UNIT, sale: 0, who: a.key, nonce, price, salt });
        }
    }
    w.mollusk.warp_to_slot(100);
    for p in &plans {
        w.must(reveal_seat_bid(p.who, ROOT_UNIT, 0, p.nonce, p.price, p.salt));
    }
    w.mollusk.warp_to_slot(200);
    w.must(settle_seats(crank, ROOT_UNIT, 0));
    let mut claimed_root: u64 = 0;
    for p in &plans {
        w.must(claim_seat(p.who, ROOT_UNIT, 0, p.nonce, ROOT_UNIT));
        claimed_root += 1;
    }
    w.must(ratify_genesis(crank));
    let mut jetted_root: u64 = 0;
    w.must(endow(crank, ROOT_UNIT, 20 * SOL));

    let mut unit_voters: HashMap<u64, Vec<Voter>> = HashMap::new();
    unit_voters.insert(ROOT_UNIT, root_founders);
    unit_voters.insert(1, one_founders);
    let mut units: Vec<u64> = vec![ROOT_UNIT, 1];
    let mut next_unit: u64 = 2;
    w.assert_invariants(0, claimed_root, jetted_root);

    let mut slot: u64 = 200;
    let mut proposals: Vec<ProposalModel> = Vec::new();
    // proposals still able to execute: not spent, no session failed
    let mut live: Vec<usize> = Vec::new();
    let mut distributions: Vec<[u8; 32]> = Vec::new();
    let mut sales: Vec<SaleModel> = Vec::new();
    let mut commits: Vec<CommitModel> = Vec::new();

    for op in 1..=OPS {
        match rng.below(16) {
            0 => {
                let unit = *rng.pick(&units);
                w.run_free(endow(actors[rng.below(4) as usize].key, unit, rng.below(20_000_000) + 1));
            }
            1 => {
                // seats move by signature: to an actor, or to a unit
                let from = &actors[rng.below(4) as usize];
                let count = rng.below(3) + 1;
                if rng.below(3) == 0 {
                    let u = *rng.pick(&units[1..]);
                    w.run_free(transfer_seats(from.key, ROOT_UNIT, unit_holder(u), HOLDER_UNIT, count));
                } else {
                    let to = &actors[rng.below(4) as usize];
                    w.run_free(transfer_seats(from.key, ROOT_UNIT, *to.key.as_array(), HOLDER_KEY, count));
                }
            }
            2..=4 => {
                // a proposal of a random class from a random unit, its scope
                // from touch: CODEX takes the whole bound subtree
                let origin = *rng.pick(&units);
                let Some(r) = w.registry(origin) else { continue };
                let mut founders = Vec::new();
                let payload = match rng.below(12) {
                    0 => Payload::Transfer { origin, dest: *rng.pick(&units), amount: rng.below(3_000_000) + 1 },
                    1 => Payload::Budget { origin, address: actors[rng.below(4) as usize].key, amount: rng.below(3_000_000) + 1 },
                    2 => Payload::Dividend { origin, amount: rng.below(2_000_000) + 1 },
                    3 => {
                        let window = 500 + rng.below(1_500);
                        Payload::Window { origin, window, codex_window: window + rng.below(2_000) }
                    }
                    4 => Payload::Plan { origin, cap: 10 * SOL, period: 500_000 + rng.below(1_000_000), token_cap: rng.below(1_000_000) },
                    5 => Payload::SaleDefaults { origin, reserve: rng.below(2_000_000) + 1, deposit: 2_000_000 + rng.below(5_000_000) },
                    6 => Payload::Weight { origin, holder: *actors[rng.below(4) as usize].key.as_array(), kind: HOLDER_KEY, weight: rng.below(3) + 1 },
                    7 => Payload::Recall { origin, chair: rng.below(PENTAGONS as u64) as u8 },
                    8 => Payload::Attach {
                        origin,
                        chair: rng.below(PENTAGONS as u64) as u8,
                        holder: *actors[rng.below(4) as usize].key.as_array(),
                        kind: HOLDER_KEY,
                        term: 3_000 + rng.below(30_000),
                        retainer: if rng.below(2) == 0 { 0 } else { rng.below(500_000) + 1 },
                    },
                    9 => Payload::Jet { origin, holder: *actors[rng.below(4) as usize].key.as_array(), kind: HOLDER_KEY },
                    10 => Payload::Ladder { origin, reserve: 1_000_000, deposit: 5_000_000, commit_len: 300 + rng.below(700), reveal_len: 300 + rng.below(700), sale_id: r.next_sale_id },
                    _ => {
                        if next_unit > 3 {
                            continue;
                        }
                        let Some(chair) = r.pentagons.iter().position(|p| p.kind == poleis_client::HOLDER_EMPTY) else { continue };
                        founders = (0..11).map(|_| w.voter(SOL)).collect();
                        Payload::Found {
                            origin,
                            spec: FoundSpec {
                                child: next_unit,
                                chair: chair as u8,
                                charter: [next_unit as u8; 32],
                                window: WINDOW,
                                plan_cap: SOL,
                                plan_period: 1_000_000,
                                founders: founders.iter().map(|f| f.key).collect(),
                                name: format!("CREW {next_unit}").into_bytes(),
                                desc: Vec::new(),
                            },
                        }
                    }
                };
                // the sessions opened with it: the whole subtree under the
                // origin, or the origin alone and every crew opens its own
                let impacted = if rng.below(3) == 0 { vec![origin] } else { w.subtree(origin) };
                let hash = proposal_hash(payload.class(), origin, &MANIFEST, &payload.hash(), op as u64);
                let proposer = unit_voters[&origin][rng.below(unit_voters[&origin].len() as u64) as usize].key;
                if w.run_free(propose(proposer, hash, MANIFEST, &payload, &impacted)) {
                    proposals.push(ProposalModel { hash, payload, impacted, convened: Vec::new(), founders, proposer, executed: false, closed: false });
                    live.push(proposals.len() - 1);
                }
            }
            5..=7 => {
                // votes on one of the freshest live proposals: four fifths of
                // a touched unit's electorate speak, plus hostile envelopes
                // that must land nothing or be refused
                if live.is_empty() {
                    continue;
                }
                let i = live[live.len() - 1 - rng.below(3.min(live.len() as u64)) as usize];
                let hash = proposals[i].hash;
                let units = proposals[i].units();
                let unit = *rng.pick(&units);
                let before = w.session(&hash, unit).map(|s| s.votes);
                match rng.below(8) {
                    0 => {
                        // a signature over PRO presented as CONTRA
                        let (k, _, sig) = actors[0].vote(&hash, unit, VOTE_PRO);
                        let ixs = attest_batch(crank, hash, unit, &[(k, VOTE_CONTRA, sig)]);
                        assert!(!w.run_tx_free(&ixs, &crank), "op {op}: a forged vote landed");
                    }
                    1 => {
                        // a stranger's envelope lands nothing
                        let v = stranger.vote(&hash, unit, VOTE_PRO);
                        if w.run_tx_free(&attest_batch(crank, hash, unit, &[v]), &crank) {
                            assert_eq!(w.session(&hash, unit).map(|s| s.votes), before, "op {op}: a stranger's vote counted");
                        }
                    }
                    _ => {
                        let mut signed = Vec::new();
                        if let Some(chairs) = unit_voters.get(&unit) {
                            for v in chairs {
                                if rng.below(5) != 0 {
                                    let c = rng.choice();
                                    signed.push(v.vote(&hash, unit, c));
                                }
                            }
                        }
                        if unit == ROOT_UNIT {
                            for v in &actors {
                                if rng.below(5) != 0 {
                                    let c = rng.choice();
                                    signed.push(v.vote(&hash, unit, c));
                                }
                            }
                        }
                        for chunk in signed.chunks(4) {
                            w.run_tx_free(&attest_batch(crank, hash, unit, chunk), &crank);
                        }
                    }
                }
            }
            8 => {
                // every crew of the live set speaks to the polis that seats it:
                // it opens its own vote while that polis's is open, carries
                // its seal up once sealed, and is heard absent once the window
                // it had to open its vote has closed
                for &i in &live {
                    let hash = proposals[i].hash;
                    for u in proposals[i].units() {
                        let (Some(s), Some(r)) = (w.session(&hash, u), w.registry(u)) else { continue };
                        if s.state != poleis_client::STATE_VOTING {
                            continue;
                        }
                        for (k, p) in r.pentagons.iter().enumerate() {
                            if p.kind != HOLDER_UNIT || s.chair_mask & (1 << k) != 0 || (p.term_end != 0 && slot > p.term_end) {
                                continue;
                            }
                            let crew = holder_unit(&p.holder);
                            match w.session(&hash, crew) {
                                Some(c) if c.state != poleis_client::STATE_VOTING => {
                                    w.run_free(mandate(crank, hash, u, crew));
                                }
                                Some(_) => {}
                                None if slot <= s.deadline => {
                                    if w.run_free(convene(crank, hash, u, crew)) {
                                        proposals[i].convened.push(crew);
                                        convenings += 1;
                                    }
                                }
                                None => {
                                    if w.run_free(mandate(crank, hash, u, crew)) {
                                        silences += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            9 => {
                // the crank seals every session whose window has closed
                for &i in &live {
                    let hash = proposals[i].hash;
                    for unit in proposals[i].units() {
                        if let Some(s) = w.session(&hash, unit) {
                            if s.state == poleis_client::STATE_VOTING && slot > s.deadline {
                                w.run_free(seal(crank, hash, unit));
                            }
                        }
                    }
                }
            }
            10 => {
                // the crank tries every live proposal; a proposal whose origin's
                // seal failed is dead and leaves the live set (a crew's seal is
                // only its chair's vote)
                for i in std::mem::take(&mut live) {
                    let hash = proposals[i].hash;
                    let origin = proposals[i].payload.origin();
                    let dead = w.session(&hash, origin).map(|s| s.state != poleis_client::STATE_VOTING && s.state != STATE_PASSED).unwrap_or(false);
                    if dead {
                        continue;
                    }
                    let jet_before = match &proposals[i].payload {
                        Payload::Jet { origin, holder, kind } if *origin == ROOT_UNIT => w.registry(ROOT_UNIT).map(|r| r.seats_of(*kind, holder)).unwrap_or(0),
                        _ => 0,
                    };
                    let ixn = execute(crank, hash, &proposals[i].impacted, &proposals[i].payload);
                    if w.run_free(ixn) {
                        proposals[i].executed = true;
                        match &proposals[i].payload {
                            Payload::Jet { origin, .. } if *origin == ROOT_UNIT => jetted_root += jet_before,
                            Payload::Dividend { .. } => distributions.push(hash),
                            Payload::Ladder { origin, reserve, deposit, sale_id, .. } => {
                                sales.push(SaleModel { unit: *origin, id: *sale_id, reserve: *reserve, deposit: *deposit })
                            }
                            Payload::Found { spec, .. } => {
                                let child = spec.child;
                                let f = std::mem::take(&mut proposals[i].founders);
                                unit_voters.insert(child, f);
                                units.push(child);
                                next_unit += 1;
                            }
                            _ => {}
                        }
                    } else {
                        live.push(i);
                    }
                }
            }
            11 => {
                // a dividend claim by a random holder, or by a unit
                if distributions.is_empty() {
                    continue;
                }
                let h = *rng.pick(&distributions);
                if rng.below(4) == 0 {
                    let u = *rng.pick(&units[1..]);
                    w.run_free(claim_dividend(crank, h, ROOT_UNIT, unit_holder(u), HOLDER_UNIT));
                } else {
                    let a = &actors[rng.below(4) as usize];
                    w.run_free(claim_dividend(a.key, h, ROOT_UNIT, *a.key.as_array(), HOLDER_KEY));
                }
            }
            12 => {
                // the seat market on ladder sales: commit, reveal, settle,
                // claim, reclaim, sometimes below reserve or above deposit
                if sales.is_empty() {
                    continue;
                }
                let s = *rng.pick(&sales);
                match rng.below(5) {
                    0 | 1 => {
                        let a = &actors[rng.below(4) as usize];
                        let nonce = rng.below(3);
                        let price = match rng.below(8) {
                            0 => s.reserve / 2,
                            1 => s.deposit + 1,
                            _ => s.reserve + rng.below(s.deposit - s.reserve + 1),
                        };
                        let salt = rng.salt();
                        if w.run_free(commit_seat_bid(a.key, s.unit, s.id, nonce, seat_commitment(price, nonce, &salt, &a.key))) {
                            commits.push(CommitModel { unit: s.unit, sale: s.id, who: a.key, nonce, price, salt });
                        }
                    }
                    2 => {
                        if let Some(c) = commits.iter().copied().find(|c| c.sale == s.id && rng.below(2) == 0) {
                            let told = if rng.below(8) == 0 { c.price ^ 1 } else { c.price };
                            w.run_free(reveal_seat_bid(c.who, c.unit, c.sale, c.nonce, told, c.salt));
                        }
                    }
                    3 => {
                        w.run_free(settle_seats(crank, s.unit, s.id));
                    }
                    _ => {
                        if commits.is_empty() {
                            continue;
                        }
                        let c = *rng.pick(&commits);
                        match rng.below(3) {
                            0 => {
                                if w.sale(c.unit, c.sale).is_some() && w.run_free(claim_seat(c.who, c.unit, c.sale, c.nonce, ROOT_UNIT)) && c.unit == ROOT_UNIT {
                                    claimed_root += 1;
                                }
                            }
                            1 => {
                                w.run_free(reclaim_seat_bid(c.who, c.unit, c.sale, c.nonce));
                            }
                            _ => {
                                // a spent escrow gives its rent back; a live one refuses
                                w.run_free(close_seat_bid(c.who, c.unit, c.sale, c.nonce));
                            }
                        }
                    }
                }
            }
            13 => {
                // time passes: sessions close, terms end, plans roll
                slot += if rng.below(8) == 0 { rng.below(300_000) + 1 } else { rng.below(1_200) + 300 };
                w.mollusk.warp_to_slot(slot);
            }
            14 => {
                // rent recovery: every spent or dead proposal gives its
                // ballot boxes back to whoever proposed
                for (i, p) in proposals.iter_mut().enumerate() {
                    if p.closed {
                        continue;
                    }
                    if w.run_free(close_proposal(p.hash, p.proposer, &p.impacted)) {
                        p.closed = true;
                        live.retain(|&j| j != i);
                    }
                    // a vote a polis opened goes home to whoever opened it,
                    // once the decision is over
                    if p.closed || p.executed {
                        let hash = p.hash;
                        p.convened.retain(|u| !w.run_free(adjourn(hash, *u, crank)));
                    }
                }
            }
            _ => {
                // the things that must be refused EVERY time
                assert!(!w.run_free(create_treasury(payer, 40 + rng.below(50))), "op {op}: dead payer key opened a vault");
                let h = proposal_hash(CLASS_CODEX, ROOT_UNIT, &MANIFEST, &[op as u8; 32], u64::MAX - op as u64);
                let plain = Payload::Window { origin: ROOT_UNIT, window: 500, codex_window: 500 };
                assert!(
                    !w.run_free(propose(stranger.key, h, MANIFEST, &plain, &[ROOT_UNIT])),
                    "op {op}: a seatless stranger proposed"
                );
                if let Some(p) = proposals.first() {
                    let naked = attest(crank, p.hash, ROOT_UNIT, &[(actors[0].key, VOTE_PRO)]);
                    assert!(!w.run_tx_free(&[naked], &crank), "op {op}: attest landed with no precompile");
                }
                w.run_free(sync_endowment(ROOT_UNIT));
            }
        }
        w.assert_invariants(op, claimed_root, jetted_root);
    }

    let executed = proposals.iter().filter(|p| p.executed).count();
    let sealed = proposals
        .iter()
        .flat_map(|p| p.units().into_iter().map(move |u| (p.hash, u)))
        .filter(|(h, u)| w.session(h, *u).map(|s| s.state != poleis_client::STATE_VOTING).unwrap_or(false))
        .count();
    println!(
        "council season: {} accepted, {} refused, {} units, {} proposals, {} votes a polis opened itself, {} crews heard absent, {} sessions sealed, {} executed, {} sales, root seats {}",
        w.accepted,
        w.refused,
        units.len(),
        proposals.len(),
        convenings,
        silences,
        sealed,
        executed,
        sales.len(),
        w.registry(ROOT_UNIT).unwrap().total_seats
    );
    assert!(w.accepted > 150, "season too quiet: {} accepted", w.accepted);
    assert!(w.refused > 50, "season too polite: {} refused", w.refused);
    assert!(executed > 3, "season without consequence: {executed} executed");
    assert!(convenings > 0, "season where no polis opened its own vote");
    let _ = (PoleisError::Unauthorized, w.lamports(&crank));
}
