//! The hostile suite: one assertion per attack, run against the compiled
//! binary on Mollusk. Where poleis.rs is the refusal ledger of the happy
//! law, this file is the audit of everything an adversary would try on the
//! money layer (treasuries, lots, seat sales, naked donations), plus the
//! finding tests from docs/threat-matrix.md that survive the double seal
//! (F5, D1, D2, D3). The council's own refusals live in council.rs.
//!
//! Honest instruction shapes come from poleis_client; the hostile variants
//! (wrong units, impostor escrows, mismatched winners) are built here on
//! the client's PDA helpers, so even the attacks share the law's encoding.

mod built;
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use poleis_client::{
    auction_pda, bid_pda, claim_seat, close_bid, close_seat_bid, code, commit_bid, fund_pda, registry_pda, tally_pda,
    commit_seat_bid, commitment, config_pda, create_treasury, init_config, init_config_of, ix, open_auction, programdata_pda,
    BPF_LOADER_UPGRADEABLE_ID,
    open_genesis, ratify_genesis, reclaim, reclaim_seat_bid, reveal_bid, reveal_seat_bid,
    sale_pda, seat_commitment, settle, settle_seats, sync_endowment, system_meta, tag,
    transfer_seats, treasury_pda, Bid, GenesisSpec, LotSpec, PoleisError, RegistryView, SaleView,
    SeatBid, Treasury, TreasuryView, HOLDER_KEY, MODE_COUNCIL, MODE_GENESIS, PROGRAM_ID,
    ROOT_UNIT,
};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

// refusal codes, sourced from the program's own enum: never retyped
const E_UNAUTHORIZED: u32 = code(PoleisError::Unauthorized);
const E_INVALID_ACCOUNT: u32 = code(PoleisError::InvalidAccount);
const E_INVALID_AMOUNT: u32 = code(PoleisError::InvalidAmount);
const E_INVALID_WINDOW: u32 = code(PoleisError::InvalidWindow);
const E_COMMIT_CLOSED: u32 = code(PoleisError::CommitWindowClosed);
const E_REVEAL_NOT_OPEN: u32 = code(PoleisError::RevealWindowNotOpen);
const E_REVEAL_CLOSED: u32 = code(PoleisError::RevealWindowClosed);
const E_ALREADY_REVEALED: u32 = code(PoleisError::AlreadyRevealed);
const E_BAD_REVEAL: u32 = code(PoleisError::BadReveal);
const E_BID_EXCEEDS_DEPOSIT: u32 = code(PoleisError::BidExceedsDeposit);
const E_ALREADY_SETTLED: u32 = code(PoleisError::AlreadySettled);
const E_NOT_SETTLED: u32 = code(PoleisError::NotSettled);
const E_NOTHING_TO_RECLAIM: u32 = code(PoleisError::NothingToReclaim);
const E_WINNER_CANNOT_RECLAIM: u32 = code(PoleisError::WinnerCannotReclaim);
const E_UNIT_MISMATCH: u32 = code(PoleisError::UnitMismatch);
const E_MISSING_WINNING_BID: u32 = code(PoleisError::MissingWinningBid);
const E_WINDOW_TOO_LONG: u32 = code(PoleisError::WindowTooLong);
const E_INVALID_MODE: u32 = code(PoleisError::InvalidMode);
const E_SALE_NOT_SETTLED: u32 = code(PoleisError::SaleNotSettled);
const E_NOT_WINNER: u32 = code(PoleisError::NotWinner);
const E_ALREADY_CLAIMED: u32 = code(PoleisError::AlreadyClaimed);
const E_NOT_HOLDER: u32 = code(PoleisError::NotHolder);
const E_SEAT_LIMIT: u32 = code(PoleisError::SeatLimit);
const E_INVALID_HOLDER: u32 = code(PoleisError::InvalidHolder);
const E_NOT_CLOSABLE: u32 = code(PoleisError::NotClosable);

const ALREADY_INIT: ProgramError = ProgramError::AccountAlreadyInitialized;
const MISSING_SIG: ProgramError = ProgramError::MissingRequiredSignature;

const SOL: u64 = 1_000_000_000;

fn custom(code: u32) -> Check<'static> {
    Check::err(ProgramError::Custom(code))
}

struct World {
    mollusk: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
}

impl World {
    fn new() -> Self {
        std::env::set_var(
            "SBF_OUT_DIR",
            built::program_dir(),
        );
        let mollusk = Mollusk::new(&PROGRAM_ID, "poleis_treasury");
        let accounts = vec![keyed_account_for_system_program()];
        Self { mollusk, accounts }
    }

    fn add_wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts
            .push((key, Account::new(lamports, 0, &Pubkey::default())));
        key
    }

    fn touch(&mut self, key: Pubkey) {
        if !self.accounts.iter().any(|(k, _)| *k == key) {
            self.accounts.push((key, Account::default()));
        }
    }

    fn lamports(&self, key: &Pubkey) -> u64 {
        self.accounts
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, a)| a.lamports)
            .unwrap_or(0)
    }

    fn account(&self, key: &Pubkey) -> Account {
        self.accounts
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, a)| a.clone())
            .unwrap_or_default()
    }

    fn run(&mut self, ix: Instruction, checks: &[Check]) {
        for meta in &ix.accounts {
            self.touch(meta.pubkey);
        }
        let result = self
            .mollusk
            .process_and_validate_instruction(&ix, &self.accounts, checks);
        if std::env::var_os("POLEIS_CU").is_some() { eprintln!("CU ix={} cu={} ok={}", ix.data[0], result.compute_units_consumed, result.program_result.is_ok()); }
        if result.program_result.is_err() {
            return; // refused: state unchanged
        }
        for (key, account) in &result.resulting_accounts {
            if let Some(slot) = self.accounts.iter_mut().find(|(k, _)| k == key) {
                slot.1 = account.clone();
            } else {
                self.accounts.push((*key, account.clone()));
            }
        }
    }

    /// (total_in, total_out) telemetry parsed from a treasury account.
    fn treasury_totals(&self, unit: u64) -> (u64, u64) {
        let a = self.account(&treasury_pda(unit));
        let t = TreasuryView::parse(&a.data).expect("treasury account parses");
        (t.total_in, t.total_out)
    }

    fn rent_floor(&self, size: usize) -> u64 {
        self.mollusk.sysvars.rent.minimum_balance(size)
    }

    /// Model a naked lamport donation to ANY address: mainnet reality:
    /// anyone can transfer to any PDA address, before or after its creation,
    /// and no program can prevent it.
    fn prefund(&mut self, key: Pubkey, lamports: u64) {
        if let Some(slot) = self.accounts.iter_mut().find(|(k, _)| *k == key) {
            slot.1.lamports += lamports;
        } else {
            self.accounts
                .push((key, Account::new(lamports, 0, &Pubkey::default())));
        }
    }
}

// ---- hostile variants: the honest shapes live in poleis_client ----

/// Endow naming one unit while presenting another unit's treasury PDA.
fn endow(donor: Pubkey, unit_arg: u64, pda_unit: u64, amount: u64) -> Instruction {
    let mut d = vec![tag::IX_ENDOW];
    d.extend(unit_arg.to_le_bytes());
    d.extend(amount.to_le_bytes());
    ix(
        d,
        vec![
            AccountMeta::new(donor, true),
            AccountMeta::new(treasury_pda(pda_unit), false),
            system_meta(),
        ],
    )
}

/// Settle with an explicitly chosen (winning_bid owner, winner account) pair
/// so tests can pass a mismatched winner.
fn settle_as(
    crank: Pubkey,
    auction: u64,
    treasury_unit: u64,
    winner: Option<(Pubkey, Pubkey)>,
) -> Instruction {
    let mut metas = vec![
        AccountMeta::new(crank, true),
        AccountMeta::new(auction_pda(auction), false),
        AccountMeta::new(treasury_pda(treasury_unit), false),
        AccountMeta::new_readonly(registry_pda(treasury_unit), false),
        AccountMeta::new(fund_pda(), false),
        AccountMeta::new(tally_pda(treasury_unit), false),
        system_meta(),
    ];
    if let Some((bid_owner, winner_acc)) = winner {
        metas.push(AccountMeta::new(bid_pda(auction, &bid_owner), false));
        metas.push(AccountMeta::new(winner_acc, false));
    }
    let mut d = vec![tag::IX_SETTLE];
    d.extend(auction.to_le_bytes());
    ix(d, metas)
}

/// Reclaim presenting another bidder's escrow (impostor test).
fn reclaim_as(signer: Pubkey, victim: Pubkey, auction: u64) -> Instruction {
    let mut d = vec![tag::IX_RECLAIM];
    d.extend(auction.to_le_bytes());
    ix(
        d,
        vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(auction_pda(auction), false),
            AccountMeta::new(bid_pda(auction, &victim), false),
        ],
    )
}

/// A lot on unit 7 with the season's windows and prices.
fn lot(id: u64, commit_end: u64, reveal_end: u64, reserve: u64, deposit: u64) -> LotSpec {
    LotSpec {
        auction_id: id,
        unit: 7,
        pool_root: [1u8; 32],
        manifest: [2u8; 32],
        commit_end,
        reveal_end,
        reserve,
        deposit,
    }
}

/// Bootstrapped world: config + one treasury for unit 7, endowed with `amt`.
fn bootstrapped(amt: u64) -> (World, Pubkey) {
    let mut w = World::new();
    let operator = w.add_wallet(100 * SOL);
    w.run(init_config(operator, Pubkey::default()), &[Check::success()]);
    w.run(create_treasury(operator, 7), &[Check::success()]);
    if amt > 0 {
        w.run(endow(operator, 7, 7, amt), &[Check::success()]);
    }
    (w, operator)
}

/// A society in genesis: root treasury, twelve founders in the chairs, the
/// rung-1 sale (twenty seats) open at reserve 1M, deposit 5M, windows 100/200.
fn genesis() -> (World, Pubkey, Vec<Pubkey>) {
    let mut w = World::new();
    let payer = w.add_wallet(100 * SOL);
    w.run(init_config(payer, Pubkey::default()), &[Check::success()]);
    let founders: Vec<Pubkey> = (0..12).map(|_| w.add_wallet(SOL)).collect();
    let spec = GenesisSpec {
        charter: [7u8; 32],
        plan_cap: 10 * SOL,
        plan_period: 1_000_000,
        window: 1_000,
        rung: 1,
        reserve: 1_000_000,
        deposit: 5_000_000,
        commit_end: 100,
        reveal_end: 200,
        founders: founders.clone(),
        name: b"HOSTILE".to_vec(),
        desc: Vec::new(),
    };
    w.run(open_genesis(payer, &spec), &[Check::success()]);
    (w, payer, founders)
}

// ================= authority & identity =================

#[test]
fn authority_gate_refusals() {
    let (mut w, _operator) = bootstrapped(0);
    let mallory = w.add_wallet(10 * SOL);

    // every authority-gated instruction refuses a stranger, by name
    w.run(create_treasury(mallory, 9), &[custom(E_UNAUTHORIZED)]);
    w.run(open_auction(mallory, &lot(1, 100, 200, 1, 1_000_000)), &[custom(E_UNAUTHORIZED)]);
    let spec = GenesisSpec {
        charter: [7u8; 32],
        plan_cap: SOL,
        plan_period: 1_000,
        window: 1_000,
        rung: 1,
        reserve: 1,
        deposit: 1,
        commit_end: 100,
        reveal_end: 200,
        founders: vec![mallory],
        name: Vec::new(),
        desc: Vec::new(),
    };
    w.run(open_genesis(mallory, &spec), &[custom(E_UNAUTHORIZED)]);
}

#[test]
fn init_config_is_init_once() {
    let mut w = World::new();
    let first = w.add_wallet(10 * SOL);
    let second = w.add_wallet(10 * SOL);
    w.run(init_config(first, Pubkey::default()), &[Check::success()]);
    // a second caller cannot rotate the authority to themselves
    w.run(init_config(second, Pubkey::default()), &[Check::err(ALREADY_INIT)]);
}

// ================= ONLY THE DEPLOYER FOUNDS ROOT 0 =================
// Tree 0's root passes the code's upgrade and the fund's constitution. Between the deploy and the press's founding (two slots on a real
// cluster) a stranger who read the deploy could found tree 0 first; the press would refuse to go on, and the launch would need Poleis at a
// new id. So tree 0 is founded only by the payer Poleis's own program data names as its upgrade authority: the deployer, at the launch.

/// Poleis's own program data as the upgradeable loader lays it ([3:4][slot:8][1 and the authority:33], or [0] when burned).
fn lay_poleis_program_data(w: &mut World, authority: Option<Pubkey>) {
    let mut data = vec![3u8, 0, 0, 0];
    data.extend_from_slice(&7u64.to_le_bytes());
    match authority {
        Some(a) => {
            data.push(1);
            data.extend_from_slice(a.as_ref());
        }
        None => data.extend_from_slice(&[0u8; 33]),
    }
    w.accounts.push((programdata_pda(), Account { lamports: SOL, data, owner: BPF_LOADER_UPGRADEABLE_ID, executable: false, rent_epoch: 0 }));
}

#[test]
fn only_the_deployer_founds_root_0() {
    let mut w = World::new();
    let deployer = w.add_wallet(10 * SOL);
    let stranger = w.add_wallet(10 * SOL);
    let elsewhere = w.add_wallet(SOL);
    lay_poleis_program_data(&mut w, Some(deployer));
    // the stranger who sends the founding first takes nothing
    w.run(init_config(stranger, Pubkey::default()), &[custom(E_UNAUTHORIZED)]);
    // nor with Poleis's program data left out, or another account in its place
    let mut short = init_config(stranger, Pubkey::default());
    assert_eq!(short.accounts.pop().map(|m| m.pubkey), Some(programdata_pda()), "tree 0's founding names Poleis's program data last");
    w.run(short, &[Check::err(ProgramError::NotEnoughAccountKeys)]);
    let mut swapped = init_config(stranger, Pubkey::default());
    swapped.accounts[3].pubkey = elsewhere;
    w.run(swapped, &[custom(E_INVALID_ACCOUNT)]);
    assert_eq!(w.lamports(&config_pda()), 0, "a stranger laid tree 0's config");
    // the deployer founds it
    w.run(init_config(deployer, Pubkey::default()), &[Check::success()]);
    // every other tree is still anyone's: one program hosts every polis
    let other = init_config_of(7, stranger, Pubkey::default());
    assert_eq!(other.accounts.len(), 3, "another tree's founding asks no program data");
    w.run(other, &[Check::success()]);
}

#[test]
fn a_poleis_whose_upgrades_were_burned_founds_no_root_0() {
    let mut w = World::new();
    let deployer = w.add_wallet(10 * SOL);
    lay_poleis_program_data(&mut w, None);
    w.run(init_config(deployer, Pubkey::default()), &[custom(E_UNAUTHORIZED)]);
}

#[test]
fn unsigned_donor_cannot_endow() {
    let (mut w, operator) = bootstrapped(0);
    let mut ixn = endow(operator, 7, 7, SOL);
    ixn.accounts[0].is_signer = false; // forged: claims the donor without the signature
    w.run(ixn, &[Check::err(MISSING_SIG)]);
}

// ================= unit confusion =================

#[test]
fn unit_confusion_refusals() {
    let (mut w, operator) = bootstrapped(SOL);
    w.run(create_treasury(operator, 8), &[Check::success()]);

    // endow naming unit 8 but passing unit 7's treasury PDA
    w.run(endow(operator, 8, 7, 100_000), &[custom(E_UNIT_MISMATCH)]);

    // duplicate treasury for an existing unit
    w.run(create_treasury(operator, 7), &[Check::err(ALREADY_INIT)]);

    // a lot naming unit 8 against unit 7's vault
    let mut wrong = lot(1, 100, 200, 1, 1_000_000);
    wrong.unit = 8;
    let mut ixn = open_auction(operator, &wrong);
    ixn.accounts[2] = AccountMeta::new_readonly(treasury_pda(7), false);
    w.run(ixn, &[custom(E_UNIT_MISMATCH)]);
}

// ================= auction: windows, commits, reveals =================

#[test]
fn auction_window_refusals() {
    let (mut w, operator) = bootstrapped(0);

    // commit window already dead at open
    w.run(open_auction(operator, &lot(10, 0, 200, 1, 1_000_000)), &[custom(E_INVALID_WINDOW)]);
    // reveal window not after commit window
    w.run(open_auction(operator, &lot(10, 100, 100, 1, 1_000_000)), &[custom(E_INVALID_WINDOW)]);
    w.run(open_auction(operator, &lot(10, 100, 200, 1, 1_000_000)), &[Check::success()]);
    // duplicate auction id
    w.run(open_auction(operator, &lot(10, 100, 200, 1, 1_000_000)), &[Check::err(ALREADY_INIT)]);
}

#[test]
fn commit_refusals() {
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);

    // a reserve of zero would hand the lot to a lone bidder for nothing
    w.run(open_auction(operator, &lot(11, 100, 200, 0, 5_000_000)), &[custom(E_INVALID_AMOUNT)]);
    // the escrow is fixed at open, not chosen per bid: a zero deposit, or a
    // deposit no qualifying bid could ever fit under, is no auction at all
    w.run(open_auction(operator, &lot(11, 100, 200, 1, 0)), &[custom(E_INVALID_AMOUNT)]);
    w.run(open_auction(operator, &lot(11, 100, 200, 2_000_000, 1_000_000)), &[custom(E_INVALID_AMOUNT)]);
    w.run(open_auction(operator, &lot(11, 100, 200, 1, 5_000_000)), &[Check::success()]);

    w.run(
        commit_bid(alice, 11, commitment(5_000_000, &[1u8; 32], &alice)),
        &[Check::success()],
    );
    // one bidder, one bid
    w.run(
        commit_bid(alice, 11, commitment(6_000_000, &[2u8; 32], &alice)),
        &[Check::err(ALREADY_INIT)],
    );
    // after the commit window, no entry
    w.mollusk.warp_to_slot(150);
    let carol = w.add_wallet(SOL);
    w.run(
        commit_bid(carol, 11, commitment(1_000_000, &[3u8; 32], &carol)),
        &[custom(E_COMMIT_CLOSED)],
    );
}

#[test]
fn reveal_hostile_ledger() {
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);
    let mallory = w.add_wallet(SOL);
    let dave = w.add_wallet(SOL);
    w.run(open_auction(operator, &lot(12, 100, 200, 1, 5_000_000)), &[Check::success()]);

    let salt_a = [7u8; 32];
    let c_alice = commitment(5_000_000, &salt_a, &alice);
    w.run(commit_bid(alice, 12, c_alice), &[Check::success()]);

    // mallory copies alice's commitment verbatim: non-transferability test
    // (the uniform deposit means her escrow is indistinguishable too)
    w.run(commit_bid(mallory, 12, c_alice), &[Check::success()]);

    w.mollusk.warp_to_slot(100);

    // the copied commitment cannot be revealed under mallory's key
    w.run(reveal_bid(mallory, 12, 5_000_000, salt_a), &[custom(E_BAD_REVEAL)]);

    let salt_m = [8u8; 32];
    w.run(reveal_bid(alice, 12, 5_000_001, salt_a), &[custom(E_BAD_REVEAL)]); // wrong amount vs commitment
    w.run(reveal_bid(alice, 12, 5_000_000, salt_a), &[Check::success()]);
    w.run(reveal_bid(alice, 12, 5_000_000, salt_a), &[custom(E_ALREADY_REVEALED)]);

    // fresh auction with a small uniform deposit: a zero-amount reveal and a
    // reveal above the escrow are both refused, even with matching commitments
    w.run(open_auction(operator, &lot(13, 150, 250, 1, 1_000_000)), &[Check::success()]);
    w.mollusk.warp_to_slot(120);
    w.run(commit_bid(mallory, 13, commitment(0, &salt_m, &mallory)), &[Check::success()]);
    let salt_d = [9u8; 32];
    w.run(commit_bid(dave, 13, commitment(2_000_000, &salt_d, &dave)), &[Check::success()]);
    w.mollusk.warp_to_slot(160);
    w.run(reveal_bid(mallory, 13, 0, salt_m), &[custom(E_BID_EXCEEDS_DEPOSIT)]);
    w.run(reveal_bid(dave, 13, 2_000_000, salt_d), &[custom(E_BID_EXCEEDS_DEPOSIT)]);

    // after the reveal window, no reveal
    w.mollusk.warp_to_slot(300);
    w.run(reveal_bid(mallory, 12, 1_000_000, salt_m), &[custom(E_REVEAL_CLOSED)]);
}

#[test]
fn revealed_best_bidder_cannot_flee() {
    // the escape hatch must NOT open for a revealed bid before settlement,
    // otherwise the best bidder walks and the sale dies unpaid
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);
    w.run(open_auction(operator, &lot(14, 100, 200, 1_000_000, 5_000_000)), &[Check::success()]);
    let salt = [1u8; 32];
    w.run(commit_bid(alice, 14, commitment(5_000_000, &salt, &alice)), &[Check::success()]);
    w.mollusk.warp_to_slot(100);
    w.run(reveal_bid(alice, 14, 5_000_000, salt), &[Check::success()]);

    w.mollusk.warp_to_slot(250); // reveal window closed, nobody settled yet

    // unrevealed bids can walk after the window; a REVEALED bid cannot
    w.run(reclaim(alice, 14), &[custom(E_NOT_SETTLED)]);

    // the crank settles the sale anyway: permissionless law
    w.run(settle(operator, 14, 7, Some(alice), 7), &[Check::success()]);
}

#[test]
fn impostor_reclaim_and_double_reclaim() {
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);
    let mallory = w.add_wallet(SOL);
    w.run(open_auction(operator, &lot(15, 100, 200, 1, 1_000_000)), &[Check::success()]);
    w.run(
        commit_bid(alice, 15, commitment(1_000_000, &[1u8; 32], &alice)),
        &[Check::success()],
    );
    w.mollusk.warp_to_slot(300); // dead season, no reveals

    // mallory presents alice's bid account: the bid's bidder field refuses her
    w.run(reclaim_as(mallory, alice, 15), &[custom(E_INVALID_ACCOUNT)]);
    // a live escrow cannot be closed, by anyone
    w.run(close_bid(alice, 15), &[custom(E_NOT_CLOSABLE)]);

    // alice reclaims; a second reclaim speaks by name
    w.run(reclaim(alice, 15), &[Check::success()]);
    w.run(reclaim(alice, 15), &[custom(E_NOTHING_TO_RECLAIM)]);

    // a spent escrow closes: its rent comes home to the bidder, never to
    // whoever presents it
    let mut hostile = close_bid(alice, 15);
    hostile.accounts[2] = AccountMeta::new(mallory, false);
    w.run(hostile, &[custom(E_INVALID_ACCOUNT)]);
    let a0 = w.lamports(&alice);
    w.run(close_bid(alice, 15), &[Check::success()]);
    assert_eq!(w.lamports(&alice) - a0, w.rent_floor(Bid::SIZE));
    assert!(w.account(&bid_pda(15, &alice)).data.is_empty());
}

// ================= auction: settlement =================

#[test]
fn settle_refusal_ledger() {
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);
    let bob = w.add_wallet(SOL);
    w.run(create_treasury(operator, 8), &[Check::success()]);
    w.run(open_auction(operator, &lot(16, 100, 200, 1, 5_000_000)), &[Check::success()]);
    w.run(commit_bid(alice, 16, commitment(5_000_000, &[1u8; 32], &alice)), &[Check::success()]);
    w.run(commit_bid(bob, 16, commitment(3_000_000, &[2u8; 32], &bob)), &[Check::success()]);
    w.mollusk.warp_to_slot(100);
    w.run(reveal_bid(alice, 16, 5_000_000, [1u8; 32]), &[Check::success()]);
    w.run(reveal_bid(bob, 16, 3_000_000, [2u8; 32]), &[Check::success()]);

    // early settle
    w.run(settle(operator, 16, 7, Some(alice), 7), &[custom(E_REVEAL_NOT_OPEN)]);
    w.mollusk.warp_to_slot(200);

    // wrong winner account alongside the true winning bid
    w.run(
        settle_as(operator, 16, 7, Some((alice, bob))),
        &[custom(E_MISSING_WINNING_BID)],
    );
    // right winner, wrong-unit treasury
    w.run(
        settle_as(operator, 16, 8, Some((alice, alice))),
        &[custom(E_UNIT_MISMATCH)],
    );
    // lawful settle, then double settle speaks by name
    w.run(settle(operator, 16, 7, Some(alice), 7), &[Check::success()]);
    w.run(settle(operator, 16, 7, Some(alice), 7), &[custom(E_ALREADY_SETTLED)]);
}

#[test]
fn tie_bids_pay_full_price() {
    // two equal top bids: second price equals the winner's own bid
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);
    let bob = w.add_wallet(SOL);
    w.run(open_auction(operator, &lot(17, 100, 200, 1, 5_000_000)), &[Check::success()]);
    w.run(commit_bid(alice, 17, commitment(5_000_000, &[1u8; 32], &alice)), &[Check::success()]);
    w.run(commit_bid(bob, 17, commitment(5_000_000, &[2u8; 32], &bob)), &[Check::success()]);
    w.mollusk.warp_to_slot(100);
    w.run(reveal_bid(alice, 17, 5_000_000, [1u8; 32]), &[Check::success()]);
    w.run(reveal_bid(bob, 17, 5_000_000, [2u8; 32]), &[Check::success()]);
    w.mollusk.warp_to_slot(200);

    let treasury = treasury_pda(7);
    let t_before = w.lamports(&treasury);
    let a_before = w.lamports(&alice);
    w.run(settle(operator, 17, 7, Some(alice), 7), &[Check::success()]);
    assert_eq!(w.lamports(&treasury) - t_before, 5_000_000); // full price, no discount
    assert_eq!(w.lamports(&alice), a_before); // no refund on a tie
}

#[test]
fn reserve_miss_settles_empty_and_best_bidder_reclaims() {
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);
    w.run(open_auction(operator, &lot(18, 100, 200, 10_000_000, 10_000_000)), &[Check::success()]);
    w.run(commit_bid(alice, 18, commitment(5_000_000, &[1u8; 32], &alice)), &[Check::success()]);
    w.mollusk.warp_to_slot(100);
    w.run(reveal_bid(alice, 18, 5_000_000, [1u8; 32]), &[Check::success()]);
    w.mollusk.warp_to_slot(200);

    // best bid 5M < reserve 10M: the season settles empty, no winner exists,
    // and the whole uniform deposit walks home
    w.run(settle(operator, 18, 7, None, 7), &[Check::success()]);
    let before = w.lamports(&alice);
    w.run(reclaim(alice, 18), &[Check::success()]);
    assert_eq!(w.lamports(&alice) - before, 10_000_000);
}

// ================= F5: window ceilings =================

#[test]
fn f5_windows_are_capped() {
    // FINDING F5 closed: no phase may exceed MAX_WINDOW_SLOTS, so no auction
    // can hold deposits behind a horizon nobody lives to see: and once the
    // capped window passes, the escape hatch opens as designed.
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);

    // an eternal reveal window is refused at open
    w.run(open_auction(operator, &lot(20, 100, u64::MAX, 1, 2_000_000)), &[custom(E_WINDOW_TOO_LONG)]);
    // so is an eternal commit window
    w.run(open_auction(operator, &lot(20, 2_000_000, 2_000_100, 1, 2_000_000)), &[custom(E_WINDOW_TOO_LONG)]);
    // the longest lawful season opens fine
    w.run(open_auction(operator, &lot(20, 1_000_000, 2_000_000, 1, 2_000_000)), &[Check::success()]);
    w.run(commit_bid(alice, 20, commitment(2_000_000, &[1u8; 32], &alice)), &[Check::success()]);

    // the window ends within a human horizon; the unrevealed deposit walks free
    w.mollusk.warp_to_slot(2_000_001);
    let before = w.lamports(&alice);
    w.run(reclaim(alice, 20), &[Check::success()]);
    assert_eq!(w.lamports(&alice) - before, 2_000_000);
}

// ================= naked donations & pre-funded PDAs =================

#[test]
fn prefunded_pdas_cannot_brick_creation() {
    // an attacker donating dust to a future PDA address must not brick the
    // account it was about to become: creation switches to top-up +
    // allocate + assign and the law proceeds untouched
    let mut w = World::new();
    let operator = w.add_wallet(100 * SOL);

    // the boot grief: dust the config and the vault before the deploy ceremony
    w.prefund(config_pda(), 1);
    w.prefund(treasury_pda(7), 5_000);
    w.run(init_config(operator, Pubkey::default()), &[Check::success()]);
    w.run(create_treasury(operator, 7), &[Check::success()]);
    w.run(endow(operator, 7, 7, SOL), &[Check::success()]);

    // dust the lot and the bid before they exist: the season runs untouched
    let alice = w.add_wallet(SOL);
    w.prefund(auction_pda(21), 3);
    w.prefund(bid_pda(21, &alice), 12_345);
    w.run(open_auction(operator, &lot(21, 100, 200, 1, 2_000_000)), &[Check::success()]);
    w.run(commit_bid(alice, 21, commitment(2_000_000, &[1u8; 32], &alice)), &[Check::success()]);
    w.mollusk.warp_to_slot(100);
    w.run(reveal_bid(alice, 21, 2_000_000, [1u8; 32]), &[Check::success()]);
    w.mollusk.warp_to_slot(200);
    let t_before = w.lamports(&treasury_pda(7));
    w.run(settle(operator, 21, 7, Some(alice), 7), &[Check::success()]);
    assert_eq!(w.lamports(&treasury_pda(7)) - t_before, 1, "a lone bid pays the reserve");
}

#[test]
fn overfunded_bid_pda_keeps_escrow_exact() {
    // donate MORE than rent to a future bid address: creation takes the
    // no-top-up path, the uniform deposit still rides on top, and reclaim
    // returns exactly the deposit: the donation stays stuck, never skimmed
    let (mut w, operator) = bootstrapped(0);
    let alice = w.add_wallet(SOL);
    w.run(open_auction(operator, &lot(40, 100, 200, 1, 2_000_000)), &[Check::success()]);
    let bid = bid_pda(40, &alice);
    let donation = w.rent_floor(Bid::SIZE) + 777_777;
    w.prefund(bid, donation);
    w.run(commit_bid(alice, 40, commitment(1_000_000, &[1u8; 32], &alice)), &[Check::success()]);
    assert_eq!(w.lamports(&bid), donation + 2_000_000);

    w.mollusk.warp_to_slot(250); // dead season: unrevealed escape hatch
    let before = w.lamports(&alice);
    w.run(reclaim(alice, 40), &[Check::success()]);
    assert_eq!(w.lamports(&alice) - before, 2_000_000, "deposit exact, donation not skimmed");
}

#[test]
fn naked_donations_are_unreceipted_until_synced() {
    // money wired straight to the treasury address is spendable but the
    // books never saw it; sync_endowment lets anyone true the books:
    // moving nothing, needing no signer, idempotent
    let (mut w, operator) = bootstrapped(SOL);
    let _ = operator;
    let treasury = treasury_pda(7);
    w.prefund(treasury, 5_000_000);

    let (tin, tout) = w.treasury_totals(7);
    assert_eq!(tin - tout, SOL, "books blind to the donation");

    let before = w.lamports(&treasury);
    w.run(sync_endowment(7), &[Check::success()]);
    assert_eq!(w.lamports(&treasury), before, "sync moves nothing");
    let (tin2, _) = w.treasury_totals(7);
    assert_eq!(tin2, SOL + 5_000_000, "books now match reality");

    // idempotent
    w.run(sync_endowment(7), &[Check::success()]);
    let (tin3, _) = w.treasury_totals(7);
    assert_eq!(tin3, SOL + 5_000_000);

    // naming the wrong unit against this PDA refuses
    let mut wrong = sync_endowment(9);
    wrong.accounts[0] = AccountMeta::new(treasury, false);
    w.run(wrong, &[custom(E_UNIT_MISMATCH)]);
}

// ================= conservation invariant =================

#[test]
fn treasury_conservation_across_mixed_flows() {
    // invariant: lamports − rent_floor == total_in − total_out, after a flow
    // mixing every money-moving instruction of the solo layer
    let (mut w, operator) = bootstrapped(2 * SOL);
    w.run(create_treasury(operator, 8), &[Check::success()]);
    let alice = w.add_wallet(SOL);
    let bob = w.add_wallet(SOL);
    let crank = w.add_wallet(SOL);

    // auction proceeds into unit 7, a donation into unit 8
    w.run(open_auction(operator, &lot(30, 100, 200, 1, 5_000_000)), &[Check::success()]);
    w.run(commit_bid(alice, 30, commitment(5_000_000, &[1u8; 32], &alice)), &[Check::success()]);
    w.run(commit_bid(bob, 30, commitment(3_000_000, &[2u8; 32], &bob)), &[Check::success()]);
    w.mollusk.warp_to_slot(100);
    w.run(reveal_bid(alice, 30, 5_000_000, [1u8; 32]), &[Check::success()]);
    w.run(reveal_bid(bob, 30, 3_000_000, [2u8; 32]), &[Check::success()]);
    w.mollusk.warp_to_slot(200);
    w.run(settle(crank, 30, 7, Some(alice), 7), &[Check::success()]);
    w.run(reclaim(bob, 30), &[Check::success()]);
    w.run(endow(bob, 8, 8, 3_000_000), &[Check::success()]);

    for unit in [7u64, 8] {
        let key = treasury_pda(unit);
        let lamports = w.lamports(&key);
        let (tin, tout) = w.treasury_totals(unit);
        let floor = w.rent_floor(Treasury::SIZE);
        assert_eq!(
            lamports - floor,
            tin - tout,
            "unit {unit}: conservation violated (lamports={lamports}, in={tin}, out={tout})"
        );
    }
}

// ================= the seat sale, hostile =================

#[test]
fn seat_sale_hostile_ledger() {
    let (mut w, payer, _founders) = genesis();
    let crank = w.add_wallet(SOL);
    let alice = w.add_wallet(SOL);
    let bob = w.add_wallet(SOL);
    let carol = w.add_wallet(SOL);
    let dave = w.add_wallet(SOL);
    let salt = [3u8; 32];

    assert_eq!(w.account(&sale_pda(ROOT_UNIT, 0)).data.len(), poleis_client::SeatSale::size(20));

    // genesis is open once
    let again = GenesisSpec {
        charter: [7u8; 32],
        plan_cap: SOL,
        plan_period: 1_000,
        window: 1_000,
        rung: 1,
        reserve: 1,
        deposit: 1,
        commit_end: 100,
        reveal_end: 200,
        founders: vec![payer],
        name: Vec::new(),
        desc: Vec::new(),
    };
    w.run(open_genesis(payer, &again), &[custom(E_INVALID_MODE)]);

    w.run(commit_seat_bid(alice, ROOT_UNIT, 0, 0, seat_commitment(3_000_000, 0, &salt, &alice)), &[Check::success()]);
    // one bidder, one commitment per nonce
    w.run(commit_seat_bid(alice, ROOT_UNIT, 0, 0, seat_commitment(4_000_000, 0, &salt, &alice)), &[Check::err(ALREADY_INIT)]);
    w.run(commit_seat_bid(bob, ROOT_UNIT, 0, 0, seat_commitment(500_000, 0, &salt, &bob)), &[Check::success()]);
    w.run(commit_seat_bid(carol, ROOT_UNIT, 0, 0, seat_commitment(6_000_000, 0, &salt, &carol)), &[Check::success()]);
    w.run(commit_seat_bid(dave, ROOT_UNIT, 0, 0, seat_commitment(2_000_000, 0, &salt, &dave)), &[Check::success()]);
    // reveal before the window
    w.run(reveal_seat_bid(alice, ROOT_UNIT, 0, 0, 3_000_000, salt), &[custom(E_REVEAL_NOT_OPEN)]);
    // ratify before anything is settled
    w.run(ratify_genesis(crank), &[custom(E_SALE_NOT_SETTLED)]);

    w.mollusk.warp_to_slot(100);
    // late commit
    w.run(commit_seat_bid(crank, ROOT_UNIT, 0, 0, seat_commitment(2_000_000, 0, &salt, &crank)), &[custom(E_COMMIT_CLOSED)]);
    // below the reserve, above the deposit, a lie, then the truth once
    w.run(reveal_seat_bid(bob, ROOT_UNIT, 0, 0, 500_000, salt), &[custom(E_INVALID_AMOUNT)]);
    w.run(reveal_seat_bid(carol, ROOT_UNIT, 0, 0, 6_000_000, salt), &[custom(E_BID_EXCEEDS_DEPOSIT)]);
    w.run(reveal_seat_bid(alice, ROOT_UNIT, 0, 0, 3_000_001, salt), &[custom(E_BAD_REVEAL)]);
    w.run(reveal_seat_bid(alice, ROOT_UNIT, 0, 0, 3_000_000, salt), &[Check::success()]);
    w.run(reveal_seat_bid(alice, ROOT_UNIT, 0, 0, 3_000_000, salt), &[custom(E_ALREADY_REVEALED)]);
    w.run(reveal_seat_bid(dave, ROOT_UNIT, 0, 0, 2_000_000, salt), &[Check::success()]);
    // claim and settle before the reveal window closes
    w.run(claim_seat(alice, ROOT_UNIT, 0, 0, ROOT_UNIT), &[custom(E_SALE_NOT_SETTLED)]);
    w.run(settle_seats(crank, ROOT_UNIT, 0), &[custom(E_REVEAL_NOT_OPEN)]);

    w.mollusk.warp_to_slot(200);
    w.run(reveal_seat_bid(bob, ROOT_UNIT, 0, 0, 500_000, salt), &[custom(E_REVEAL_CLOSED)]);
    w.run(settle_seats(crank, ROOT_UNIT, 0), &[Check::success()]);
    w.run(settle_seats(crank, ROOT_UNIT, 0), &[custom(E_ALREADY_SETTLED)]);
    let s = SaleView::parse(&w.account(&sale_pda(ROOT_UNIT, 0)).data).expect("sale parses");
    assert_eq!((s.clearing, s.board.len()), (1_000_000, 2), "two bids for twenty seats clear at the reserve");

    // a winner's escrow is the price, never a refund
    w.run(reclaim_seat_bid(alice, ROOT_UNIT, 0, 0), &[custom(E_WINNER_CANNOT_RECLAIM)]);

    // only revealed winners claim, once each, paying the clearing price
    w.run(claim_seat(bob, ROOT_UNIT, 0, 0, ROOT_UNIT), &[custom(E_NOT_WINNER)]);
    let t0 = w.lamports(&treasury_pda(ROOT_UNIT));
    let a0 = w.lamports(&alice);
    // a registry is born with no holder rows; each first claim grows it by one row, paid by the claimer
    let sysvar_rent = w.mollusk.sysvars.rent.clone();
    let rent = move |n: usize| sysvar_rent.minimum_balance(poleis_client::Registry::size_for(n));
    let row = rent(1) - rent(0);
    assert_eq!(w.account(&registry_pda_of(ROOT_UNIT)).data.len(), poleis_client::Registry::size_for(0), "born without rows");
    w.run(claim_seat(alice, ROOT_UNIT, 0, 0, ROOT_UNIT), &[Check::success()]);
    w.run(claim_seat(alice, ROOT_UNIT, 0, 0, ROOT_UNIT), &[custom(E_ALREADY_CLAIMED)]);
    w.run(claim_seat(dave, ROOT_UNIT, 0, 0, ROOT_UNIT), &[Check::success()]);
    assert_eq!(w.account(&registry_pda_of(ROOT_UNIT)).data.len(), poleis_client::Registry::size_for(2), "one row per holder");
    assert_eq!(w.lamports(&registry_pda_of(ROOT_UNIT)), rent(2), "exactly the rent its rows need");
    assert_eq!(w.lamports(&treasury_pda(ROOT_UNIT)) - t0, 2_000_000);
    assert_eq!(w.lamports(&alice) - a0, 5_000_000 - 1_000_000 - row, "the deposit minus the price comes home, less the row the claimer paid for");
    assert_eq!(w.treasury_totals(ROOT_UNIT).0, 2_000_000);

    // the escrow hatch: a spent escrow has nothing left, losers get exactly
    // the deposit, once
    w.run(reclaim_seat_bid(alice, ROOT_UNIT, 0, 0), &[custom(E_NOTHING_TO_RECLAIM)]);
    w.run(close_seat_bid(bob, ROOT_UNIT, 0, 0), &[custom(E_NOT_CLOSABLE)]);
    let b0 = w.lamports(&bob);
    w.run(reclaim_seat_bid(bob, ROOT_UNIT, 0, 0), &[Check::success()]);
    assert_eq!(w.lamports(&bob) - b0, 5_000_000);
    w.run(reclaim_seat_bid(bob, ROOT_UNIT, 0, 0), &[custom(E_NOTHING_TO_RECLAIM)]);
    w.run(reclaim_seat_bid(carol, ROOT_UNIT, 0, 0), &[Check::success()]);
    // spent escrows give their rent back, once
    let b1 = w.lamports(&bob);
    w.run(close_seat_bid(bob, ROOT_UNIT, 0, 0), &[Check::success()]);
    assert_eq!(w.lamports(&bob) - b1, w.rent_floor(SeatBid::SIZE));
    w.run(close_seat_bid(bob, ROOT_UNIT, 0, 0), &[custom(E_INVALID_ACCOUNT)]);
    w.run(close_seat_bid(alice, ROOT_UNIT, 0, 0), &[Check::success()]);

    // ratification seats the council and kills the payer key
    let r = RegistryView::parse(&w.account(&registry_pda_of(ROOT_UNIT)).data).expect("registry parses");
    assert_eq!(r.total_seats, 2);
    w.run(ratify_genesis(crank), &[Check::success()]);
    w.run(ratify_genesis(crank), &[custom(E_INVALID_MODE)]);
    w.run(create_treasury(payer, 5), &[custom(E_INVALID_MODE)]);
    w.run(open_auction(payer, &lot(1, 300, 400, 1, 1_000_000)), &[custom(E_INVALID_MODE)]);
    let _ = (MODE_GENESIS, MODE_COUNCIL);

    // seats move by signature, inside the law, once they have rested a
    // window: never zero, never more than held, never to yourself, never
    // from a stranger
    w.run(transfer_seats(alice, ROOT_UNIT, *dave.as_array(), HOLDER_KEY, 1), &[custom(code(PoleisError::SeatsResting))]);
    w.mollusk.warp_to_slot(1_300);
    w.run(transfer_seats(alice, ROOT_UNIT, *dave.as_array(), HOLDER_KEY, 0), &[custom(E_INVALID_AMOUNT)]);
    w.run(transfer_seats(alice, ROOT_UNIT, *dave.as_array(), HOLDER_KEY, 2), &[custom(E_SEAT_LIMIT)]);
    w.run(transfer_seats(alice, ROOT_UNIT, *alice.as_array(), HOLDER_KEY, 1), &[custom(E_INVALID_HOLDER)]);
    w.run(transfer_seats(bob, ROOT_UNIT, *dave.as_array(), HOLDER_KEY, 1), &[custom(E_NOT_HOLDER)]);
    w.run(transfer_seats(alice, ROOT_UNIT, *dave.as_array(), HOLDER_KEY, 1), &[Check::success()]);
    let r = RegistryView::parse(&w.account(&registry_pda_of(ROOT_UNIT)).data).expect("registry parses");
    assert_eq!((r.total_seats, r.seats_of(HOLDER_KEY, alice.as_array()), r.seats_of(HOLDER_KEY, dave.as_array())), (2, 0, 2));
}

fn registry_pda_of(unit: u64) -> Pubkey {
    poleis_client::registry_pda(unit)
}
