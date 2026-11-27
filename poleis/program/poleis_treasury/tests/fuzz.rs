//! Seeded chaos on the money layer: a deterministic stream of random legal
//! and hostile instructions against the compiled binary in solo mode, with
//! the money invariants machine-checked after EVERY op. No expectations on
//! individual ops: any op may be refused; what may never happen is an
//! accepted op that breaks an invariant:
//!
//!   I1  treasury conservation, donation-aware: ledger ≤ free ≤ ledger + donated
//!       (equality with the ledger whenever nothing was donated or after sync)
//!   I3  bid escrow exact up to donations: base ≤ lamports ≤ base + donated,
//!       base = rent (+ uniform deposit while live)
//!   I4  auction sanity: best ≤ deposit; clearing inside [reserve, best]
//!
//! The season also throws naked lamport donations at every PDA class,
//! mainnet reality no program can prevent, and asserts the law shrugs.
//! The seed is fixed: every run replays the same season, so a failure here
//! is a reproducible counterexample, not a flake. The council layer has its
//! own season (fuzz_council.rs).

mod built;
use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
use poleis_client::{
    auction_pda, bid_pda, close_bid, commit_bid, commitment, config_pda, create_treasury, endow,
    init_config, open_auction, reclaim, reveal_bid, settle, sync_endowment, treasury_pda,
    AuctionView, Bid, BidView, LotSpec, Treasury, TreasuryView, PROGRAM_ID,
};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::collections::HashMap;

const SOL: u64 = 1_000_000_000;
const OPS: usize = 1_200;
const SEED: u64 = 0x0715_1204_2026_0901;

// ---- deterministic rng ----
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
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
}

struct World {
    mollusk: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
    accepted: usize,
    refused: usize,
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
        }
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

    fn data_of(&self, key: &Pubkey) -> Option<&[u8]> {
        self.accounts
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, a)| a.data.as_slice())
    }

    /// Run one op with no expectations; merge state only on acceptance.
    fn run_free(&mut self, ixn: Instruction) -> bool {
        for meta in &ixn.accounts {
            self.touch(meta.pubkey);
        }
        let result = self
            .mollusk
            .process_and_validate_instruction(&ixn, &self.accounts, &[]);
        if result.program_result.is_err() {
            self.refused += 1;
            return false;
        }
        for (key, account) in &result.resulting_accounts {
            if let Some(slot) = self.accounts.iter_mut().find(|(k, _)| k == key) {
                slot.1 = account.clone();
            } else {
                self.accounts.push((*key, account.clone()));
            }
        }
        self.accepted += 1;
        true
    }

    fn floor(&self, size: usize) -> u64 {
        self.mollusk.sysvars.rent.minimum_balance(size)
    }

    /// A naked lamport donation to any address: no instruction, no program
    /// consent, exactly as mainnet allows.
    fn donate(&mut self, key: Pubkey, lamports: u64) {
        if let Some(slot) = self.accounts.iter_mut().find(|(k, _)| *k == key) {
            slot.1.lamports += lamports;
        } else {
            self.accounts
                .push((key, Account::new(lamports, 0, &Pubkey::default())));
        }
    }

    /// The law of the estate, checked over every program account. Donations
    /// widen the exact equalities into tight ranges: lamports may exceed the
    /// books by at most what was donated, and never fall below them.
    fn assert_invariants(&self, op_index: usize, donated: &HashMap<Pubkey, u64>) {
        for (key, a) in &self.accounts {
            if a.owner != PROGRAM_ID || a.data.is_empty() {
                continue;
            }
            let d = a.data.as_slice();
            let don = *donated.get(key).unwrap_or(&0) as i128;
            if let Some(t) = TreasuryView::parse(d) {
                // I1: Treasury conservation, donation-aware: free lamports
                // can exceed the ledger by at most the donations
                let ledger = t.total_in as i128 - t.total_out as i128;
                let free = (a.lamports - self.floor(Treasury::SIZE)) as i128;
                assert!(
                    free >= ledger && free <= ledger + don,
                    "op {op_index}: treasury {key} broke conservation \
                     (free={free} ledger={ledger} donated={don})"
                );
            } else if let Some(b) = BidView::parse(d) {
                // I3: Bid escrow exact up to donations: spent/reclaimed
                // bids hold rent, live bids rent + the uniform deposit;
                // donated dust may sit on top, never be skimmed below
                let base = if b.reclaimed {
                    self.floor(Bid::SIZE)
                } else {
                    self.floor(Bid::SIZE) + b.deposit
                } as i128;
                let lamports = a.lamports as i128;
                assert!(
                    lamports >= base && lamports <= base + don,
                    "op {op_index}: bid {key} escrow drift \
                     (lamports={lamports} base={base} donated={don} reclaimed={})",
                    b.reclaimed
                );
            } else if let Some(x) = AuctionView::parse(d) {
                // I4: Auction prices inside the law
                assert!(
                    x.best_bid <= x.deposit,
                    "op {op_index}: auction {key} best bid above uniform deposit"
                );
                assert!(x.reserve > 0, "op {op_index}: auction {key} opened with reserve 0");
                if x.clearing > 0 {
                    assert!(
                        x.clearing <= x.best_bid && x.clearing >= x.reserve,
                        "op {op_index}: auction {key} cleared at {} outside [{}, {}]",
                        x.clearing,
                        x.reserve,
                        x.best_bid
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct AuctionModel {
    id: u64,
    unit: u64,
    deposit: u64,
}

#[test]
fn seeded_chaos_season() {
    let mut rng = Rng(SEED);
    let mut w = World::new();
    let operator = w.add_wallet(1_000 * SOL);
    let actors: Vec<Pubkey> = (0..4).map(|_| w.add_wallet(1_000 * SOL)).collect();

    // bootstrap: config + two treasuries
    assert!(w.run_free(init_config(operator, Pubkey::default())));
    for unit in [7u64, 8] {
        assert!(w.run_free(create_treasury(operator, unit)));
    }

    let mut slot: u64 = 1;
    let mut next_auction: u64 = 100;
    // remembered honest commits: (auction_id, actor, amount, salt)
    let mut commits: Vec<(u64, Pubkey, u64, [u8; 32])> = Vec::new();
    let mut auctions: Vec<AuctionModel> = Vec::new();
    let mut donated: HashMap<Pubkey, u64> = HashMap::new();

    for op in 0..OPS {
        let unit = *rng.pick(&[7u64, 8]);
        let actor = *rng.pick(&actors);
        match rng.below(12) {
            11 => {
                // rent recovery on a random remembered escrow: refused while
                // live, refunded once reclaimed or won and settled
                if commits.is_empty() {
                    continue;
                }
                let (aid, bidder, _, _) = *rng.pick(&commits);
                w.run_free(close_bid(bidder, aid));
            }
            0 => {
                // endow; 1 time in 8 name a mismatched unit against this PDA
                let amount = rng.below(20_000_000) + 1;
                if rng.below(8) == 0 {
                    let mut wrong = endow(actor, 9, amount);
                    wrong.accounts[1] = AccountMeta::new(treasury_pda(unit), false);
                    w.run_free(wrong);
                } else {
                    w.run_free(endow(actor, unit, amount));
                }
            }
            1 | 2 => {
                // open an auction; reserve, deposit and windows sometimes
                // hostile; 1 time in 6 a stranger tries the authority's act
                let id = next_auction;
                let deposit = rng.below(10_000_000) + 1;
                let reserve = match rng.below(10) {
                    0 => 0,                                  // hostile: no reserve
                    1 => deposit + rng.below(1_000_000) + 1, // hostile: reserve above deposit
                    _ => rng.below(deposit) + 1,             // lawful: 1..=deposit
                };
                let commit_end = if rng.below(10) == 0 {
                    slot + 2_000_000 // hostile: phase beyond the ceiling
                } else {
                    slot + rng.below(5_000) + 2
                };
                let reveal_end = commit_end + rng.below(5_000) + 2;
                let spec = LotSpec {
                    auction_id: id,
                    unit,
                    pool_root: [1u8; 32],
                    manifest: [2u8; 32],
                    commit_end,
                    reveal_end,
                    reserve,
                    deposit,
                };
                if rng.below(6) == 0 {
                    assert!(!w.run_free(open_auction(actor, &spec)), "op {op}: a stranger opened a lot");
                } else if w.run_free(open_auction(operator, &spec)) {
                    auctions.push(AuctionModel { id, unit, deposit });
                    next_auction += 1;
                }
            }
            3 | 4 => {
                // commit a bid on a random known auction; amount sometimes
                // hostile (above the uniform deposit: reveal must refuse it)
                if auctions.is_empty() {
                    continue;
                }
                let am = *rng.pick(&auctions);
                let amount = if rng.below(6) == 0 {
                    am.deposit + rng.below(1_000_000) + 1
                } else {
                    rng.below(am.deposit) + 1
                };
                let salt = rng.salt();
                if w.run_free(commit_bid(actor, am.id, commitment(amount, &salt, &actor))) {
                    commits.push((am.id, actor, amount, salt));
                }
            }
            5 => {
                // reveal a remembered commit honestly (or lie 1 time in 8)
                if commits.is_empty() {
                    continue;
                }
                let (aid, bidder, amount, salt) = *rng.pick(&commits);
                let told = if rng.below(8) == 0 { amount ^ 1 } else { amount };
                w.run_free(reveal_bid(bidder, aid, told, salt));
            }
            6 => {
                // settle a random auction, presenting the winner the chain names
                if auctions.is_empty() {
                    continue;
                }
                let am = *rng.pick(&auctions);
                let winner = w
                    .data_of(&auction_pda(am.id))
                    .and_then(AuctionView::parse)
                    .and_then(|a| a.winner());
                w.run_free(settle(actor, am.id, am.unit, winner, am.unit));
            }
            7 => {
                // reclaim: any actor tries to pull a deposit back
                if auctions.is_empty() {
                    continue;
                }
                let am = *rng.pick(&auctions);
                w.run_free(reclaim(actor, am.id));
            }
            8 => {
                // time passes
                slot += rng.below(3_000) + 1;
                w.mollusk.warp_to_slot(slot);
            }
            9 => {
                // naked donation to a random PDA class, created or not yet:
                // no program consents to this on mainnet, so the law must hold
                let amount = rng.below(2_000_000) + 1;
                let aid = auctions.last().map(|a| a.id).unwrap_or(next_auction);
                let target = match rng.below(4) {
                    0 => treasury_pda(unit),
                    1 => config_pda(),
                    2 => auction_pda(aid),
                    _ => bid_pda(aid, &actor),
                };
                w.donate(target, amount);
                *donated.entry(target).or_insert(0) += amount;
            }
            _ => {
                // anyone trues the books on a random treasury
                w.run_free(sync_endowment(unit));
            }
        }
        w.assert_invariants(op, &donated);
    }

    // the season must have been a real fight, not a wall of refusals
    println!(
        "chaos season: {} accepted, {} refused, {} auctions",
        w.accepted,
        w.refused,
        auctions.len(),
    );
    assert!(w.accepted > 200, "season too quiet: {} accepted", w.accepted);
    assert!(w.refused > 50, "season too polite: {} refused", w.refused);
}
