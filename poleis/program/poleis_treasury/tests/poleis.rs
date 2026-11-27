//! The birth suite: the money layer's happy law and its refusal ledger in
//! solo mode, run against the compiled binary on Mollusk. Every instruction
//! shape comes from poleis_client, so the tests speak exactly the encoding
//! the program reads. The council law has its own suite (council.rs).

mod built;
use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
use poleis_client::{
    code, commit_bid, commitment, create_treasury, endow, init_config, open_auction, reclaim,
    reveal_bid, settle, treasury_pda, LotSpec, PoleisError, TreasuryView, PROGRAM_ID,
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

fn mollusk() -> Mollusk {
    std::env::set_var(
        "SBF_OUT_DIR",
        built::program_dir(),
    );
    Mollusk::new(&PROGRAM_ID, "poleis_treasury")
}

struct World {
    mollusk: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
}

impl World {
    fn new() -> Self {
        let mollusk = mollusk();
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

    fn data(&self, key: &Pubkey) -> Vec<u8> {
        self.accounts
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, a)| a.data.clone())
            .unwrap_or_default()
    }

    /// Run one instruction against current state with the given checks.
    /// On success, the resulting accounts replace the world state.
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
}

const SOL: u64 = 1_000_000_000;

fn refuse(e: PoleisError) -> Check<'static> {
    Check::err(ProgramError::Custom(code(e)))
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

// ---- the suites ----

#[test]
fn bootstrap_and_unauthorized_refusal() {
    let mut w = World::new();
    let operator = w.add_wallet(10 * SOL);
    let mallory = w.add_wallet(10 * SOL);

    w.run(init_config(operator, Pubkey::default()), &[Check::success()]);
    w.run(create_treasury(operator, 7), &[Check::success()]);
    w.run(endow(operator, 7, SOL), &[Check::success()]);

    // a stranger can neither mint a treasury nor open a lot
    w.run(create_treasury(mallory, 8), &[refuse(PoleisError::Unauthorized)]);
    w.run(open_auction(mallory, &lot(1, 100, 200, 1, 1_000_000)), &[refuse(PoleisError::Unauthorized)]);

    assert!(w.lamports(&treasury_pda(7)) > SOL, "endowment landed");
}

#[test]
fn endowment_is_open_to_anyone_and_receipted() {
    // endow is not an authority act: any wallet may fund a vault, and the
    // books record every lamport that arrives through the law
    let mut w = World::new();
    let operator = w.add_wallet(10 * SOL);
    let donor = w.add_wallet(10 * SOL);
    w.run(init_config(operator, Pubkey::default()), &[Check::success()]);
    w.run(create_treasury(operator, 7), &[Check::success()]);
    w.run(endow(donor, 7, 3 * SOL), &[Check::success()]);
    w.run(endow(donor, 7, 0), &[refuse(PoleisError::InvalidAmount)]);
    let t = TreasuryView::parse(&w.data(&treasury_pda(7))).expect("treasury parses");
    assert_eq!((t.total_in, t.total_out), (3 * SOL, 0));
}

#[test]
fn auction_season_with_refusal_ledger() {
    let mut w = World::new();
    let operator = w.add_wallet(10 * SOL);
    let alice = w.add_wallet(SOL);
    let bob = w.add_wallet(SOL);
    let carol = w.add_wallet(SOL);

    w.run(init_config(operator, Pubkey::default()), &[Check::success()]);
    w.run(create_treasury(operator, 7), &[Check::success()]);

    let salt_a = [7u8; 32];
    let salt_b = [9u8; 32];
    let bid_a = 5_000_000u64;
    let bid_b = 3_000_000u64;

    // an auction with no lot cannot be opened
    let mut empty = lot(1, 100, 200, 1_000_000, 5_000_000);
    empty.pool_root = [0u8; 32];
    w.run(open_auction(operator, &empty), &[refuse(PoleisError::InvalidAccount)]);
    w.run(open_auction(operator, &lot(1, 100, 200, 1_000_000, 5_000_000)), &[Check::success()]);

    w.run(
        commit_bid(alice, 1, commitment(bid_a, &salt_a, &alice)),
        &[Check::success()],
    );
    w.run(
        commit_bid(bob, 1, commitment(bid_b, &salt_b, &bob)),
        &[Check::success()],
    );

    // reveal before the window is refused
    w.run(reveal_bid(alice, 1, bid_a, salt_a), &[refuse(PoleisError::RevealWindowNotOpen)]);

    w.mollusk.warp_to_slot(100);

    // late commit is refused
    w.run(
        commit_bid(carol, 1, commitment(1, &[1u8; 32], &carol)),
        &[refuse(PoleisError::CommitWindowClosed)],
    );
    // a lying reveal is refused
    w.run(reveal_bid(bob, 1, bid_b + 1, salt_b), &[refuse(PoleisError::BadReveal)]);

    w.run(reveal_bid(alice, 1, bid_a, salt_a), &[Check::success()]);
    w.run(reveal_bid(bob, 1, bid_b, salt_b), &[Check::success()]);

    w.mollusk.warp_to_slot(200);

    // settling a qualified season without the winner's escrow is refused
    w.run(settle(operator, 1, 7, None, 7), &[refuse(PoleisError::MissingWinningBid)]);

    let treasury = treasury_pda(7);
    let treasury_before = w.lamports(&treasury);
    let alice_before = w.lamports(&alice);

    w.run(settle(operator, 1, 7, Some(alice), 7), &[Check::success()]);

    // Vickrey: alice wins, pays bob's price (above the reserve)
    assert_eq!(w.lamports(&treasury) - treasury_before, bid_b);
    assert_eq!(w.lamports(&alice) - alice_before, bid_a - bid_b);

    // winner does not reclaim, winner pays
    w.run(reclaim(alice, 1), &[refuse(PoleisError::WinnerCannotReclaim)]);

    let bob_before = w.lamports(&bob);
    w.run(reclaim(bob, 1), &[Check::success()]);
    // bob gets back the uniform deposit, not his bid
    assert_eq!(w.lamports(&bob) - bob_before, 5_000_000);
}

#[test]
fn zero_reveal_auction_frees_every_deposit() {
    let mut w = World::new();
    let operator = w.add_wallet(10 * SOL);
    let alice = w.add_wallet(SOL);

    w.run(init_config(operator, Pubkey::default()), &[Check::success()]);
    w.run(create_treasury(operator, 7), &[Check::success()]);
    w.run(open_auction(operator, &lot(2, 100, 200, 1_000_000, 2_000_000)), &[Check::success()]);
    w.run(
        commit_bid(alice, 2, commitment(2_000_000, &[3u8; 32], &alice)),
        &[Check::success()],
    );

    // nobody reveals; the window dies; the unrevealed deposit walks free
    w.mollusk.warp_to_slot(300);
    let alice_before = w.lamports(&alice);
    w.run(reclaim(alice, 2), &[Check::success()]);
    assert_eq!(w.lamports(&alice) - alice_before, 2_000_000);

    // and the empty season still settles cleanly
    w.run(settle(operator, 2, 7, None, 7), &[Check::success()]);
}
