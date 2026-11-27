//! poleis_treasury: the settlement program of Poleis Protocol.
//!
//! The boundary law: the chain is touched when money moves or a seal lands.
//! Deliberation and voting live in the swarm's own channel; this program
//! holds treasuries, sells seats and trace lots at sealed sales, and moves
//! money and structure only when a proposal has passed the double seal in
//! every unit it touches.
//!
//! The double seal: a unit changes only when two incompatible legitimacies
//! sign the same hash. The hexagons, whose weight
//! is what they put in, and the pentagons, twelve heads with one voice each.
//! More than half of the lit chairs (seven of twelve) AND more than half the
//! active weight, because you cannot bribe Euler. A polis in a chair speaks
//! through its own seal, its session opened by itself, so a decision folds
//! up a tree of any width, one transaction per polis.

// The program is `no_std`: no heap, no allocator, no formatting machinery.
// Every buffer is on the stack and every size is a constant; a stray
// allocation would panic instead of silently costing compute and bytes.
#![no_std]

use pinocchio::{
    error::ProgramError, no_allocator, nostd_panic_handler, program_entrypoint, AccountView, Address,
    ProgramResult,
};

pub mod error;
pub mod assembly;
pub mod bls;
pub mod ix;
pub mod mint_pricing;
pub mod nft_seat_state;
pub mod nft_wire;
pub mod completion_permit;
pub mod roads;
pub mod seat_nft;
pub mod state;
pub mod util;

solana_address::declare_id!("AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab");

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

// instruction tags: byte 0 of instruction data
pub const IX_INIT_CONFIG: u8 = 0;
pub const IX_CREATE_TREASURY: u8 = 1;
pub const IX_ENDOW: u8 = 2;
pub const IX_SYNC_ENDOWMENT: u8 = 3;
// the trace lot auction
pub const IX_OPEN_AUCTION: u8 = 4;
pub const IX_COMMIT_BID: u8 = 5;
pub const IX_REVEAL_BID: u8 = 6;
pub const IX_SETTLE: u8 = 7;
pub const IX_RECLAIM: u8 = 8;
// the hexagon seat sale
pub const IX_COMMIT_SEAT_BID: u8 = 9;
pub const IX_REVEAL_SEAT_BID: u8 = 10;
pub const IX_SETTLE_SEATS: u8 = 11;
pub const IX_CLAIM_SEAT: u8 = 12;
pub const IX_RECLAIM_SEAT_BID: u8 = 13;
pub const IX_TRANSFER_SEATS: u8 = 14;
// genesis
pub const IX_OPEN_GENESIS: u8 = 15;
pub const IX_FOUND_CHILD: u8 = 16;
pub const IX_RATIFY_GENESIS: u8 = 17;
// the double seal
pub const IX_PROPOSE: u8 = 18;
pub const IX_ATTEST: u8 = 19;
pub const IX_MANDATE: u8 = 20;
pub const IX_SEAL: u8 = 21;
pub const IX_EXECUTE: u8 = 22;
pub const IX_CLAIM_DIVIDEND: u8 = 23;
// rent recovery: a decided proposal and its sessions, a spent escrow
pub const IX_CLOSE_PROPOSAL: u8 = 24;
pub const IX_CLOSE_SEAT_BID: u8 = 25;
pub const IX_CLOSE_BID: u8 = 26;
/// The aggregate vote: one BLS signature over one envelope for every key
/// passed, verified by one pairing (the BLS12-381 syscalls, live on mainnet
/// since epoch 986, June 12 2026). Reserved here since September 2026.
pub const IX_ATTEST_BLS: u8 = 27;
// token vaults: a unit's token account per mint, its authority the vault
pub const IX_CREATE_VAULT: u8 = 28;
// the economy: standing authorizations, conditional payments, income by rule
pub const IX_PULL: u8 = 29;
pub const IX_CLOSE_ALLOWANCE: u8 = 30;
pub const IX_EXPIRE_CONTRACT: u8 = 31;
pub const IX_ACCRUE_DIVIDEND: u8 = 32;
pub const IX_CLOSE_DISTRIBUTION: u8 = 33;
// the seat market
pub const IX_LIST_SEATS: u8 = 34;
pub const IX_BUY_SEATS: u8 = 35;
pub const IX_CANCEL_LISTING: u8 = 36;
/// Refresh a public floor index from an authenticated current or legacy registry.
pub const IX_SYNC_ASSEMBLY_FLOOR: u8 = 37;
// the fund: the polis of poleis
pub const IX_SEAT_CHAIR: u8 = 38;
pub const IX_PROPOSE_OPTION: u8 = 39;
pub const IX_STAKE: u8 = 40;
pub const IX_UNSTAKE: u8 = 41;
pub const IX_SETTLE_CYCLE: u8 = 42;
/// The door: a knock at a polis whose spots are open by admission, and the
/// knocker's withdrawal once the polis has kept it waiting a CODEX window.
pub const IX_KNOCK: u8 = 43;
pub const IX_WITHDRAW_KNOCK: u8 = 44;
/// The fund's grant, booked when a cycle rolled on touch, paid to its recipient.
pub const IX_CLAIM_GRANT: u8 = 45;
// Harmony's completion-gated NFT genesis and stable seat ownership.
pub const IX_OPEN_NFT_GENESIS: u8 = 46;
pub const IX_MINT_NFT: u8 = 47;
pub const IX_RATIFY_NFT_GENESIS: u8 = 48;
pub const IX_ATTEST_NFT: u8 = 49;
pub const IX_PROPOSE_NFT: u8 = 50;
pub const IX_CLAIM_DIVIDEND_NFT: u8 = 51;
pub const IX_RETIRE_BURNED_NFT: u8 = 52;
pub const IX_WITHDRAW_RETIRED_NFT: u8 = 53;
/// A holder's BLS key for the aggregate vote, registered with its proof of possession.
pub const IX_REGISTER_BLS_KEY: u8 = 54;
/// Each polis opens its own vote: a crew seated in a chair of an open session
/// opens its own session of the decision, in its own transaction.
pub const IX_CONVENE: u8 = 55;
/// A convened session's rent goes home to whoever convened it, once the decision is over.
pub const IX_ADJOURN: u8 = 56;
/// AN OLDER FUND HAS ITS ROAD (Nov 24 2026): the fund of before Sept 9 2026 (706 bytes, no owed slot) grown in place to this layout, once,
/// by anyone who pays the rent of the added bytes (`ix::fund::grow_fund`).
pub const IX_GROW_FUND: u8 = 57;

// Law ceilings. Slots run ~400ms, so 1_296_000 slots is roughly six days.
pub const MAX_WINDOW_SLOTS: u64 = 1_296_000;
/// The shortest voting window a unit may set: room for a voter to notice a
/// dropped envelope and land it alone.
pub const MIN_WINDOW_SLOTS: u64 = 100;
/// A payload lives in its proposal account in clear, so voters read what
/// will execute and anyone can execute it.
pub const MAX_PAYLOAD: usize = 768;
/// A society's name and description, on chain, in the registry.
pub const MAX_NAME: usize = 64;
pub const MAX_DESC: usize = 128;
/// A raw act passes at most this many accounts to the program it calls: sixteen, room for an SPL stake pool's AddValidatorToPool (13) and its
/// Initialize (10), so a unit can hold a stake pool by its own hands (his Oct 29 2026 "you gotta go back to the design of our governance dude...
/// the pentagon and hexagon voting", and "Yes, widen them"). Sixteen keys with their flags are 528 bytes, inside MAX_PAYLOAD.
pub const MAX_ACT_ACCOUNTS: usize = 16;
/// The longest term a chair may be attached for: about a year of slots.
pub const MAX_TERM_SLOTS: u64 = 78_840_000;
/// A CODEX window is this many voting windows by default: a society buys
/// fast and rewrites itself slowly.
pub const CODEX_WINDOW_FACTOR: u64 = 3;
/// A distribution may be closed, its dust swept home, this long after it
/// was cut (about 60 days): every holder had two activity windows to claim.
pub const DISTRIBUTION_TTL_SLOTS: u64 = 12_960_000;
/// Basis points: the dividend policy's unit.
pub const BPS: u64 = 10_000;
/// The BPF upgradeable loader: `BPFLoaderUpgradeab1e11111111111111111111111`.
pub const BPF_LOADER_UPGRADEABLE_ID: Address = Address::new_from_array([
    0x02, 0xa8, 0xf6, 0x91, 0x4e, 0x88, 0xa1, 0xb0, 0xe2, 0x10, 0x15, 0x3e, 0xf7, 0x63, 0xae, 0x2b,
    0x00, 0xc2, 0xb9, 0x3d, 0x16, 0xc1, 0x24, 0xd2, 0xc0, 0x53, 0x7a, 0x10, 0x04, 0x80, 0x00, 0x00,
]);

/// The SPL Token programs a vault may live under.
pub const TOKEN_PROGRAM_ID: Address = Address::new_from_array([
    0x06, 0xdd, 0xf6, 0xe1, 0xd7, 0x65, 0xa1, 0x93, 0xd9, 0xcb, 0xe1, 0x46, 0xce, 0xeb, 0x79, 0xac,
    0x1c, 0xb4, 0x85, 0xed, 0x5f, 0x5b, 0x37, 0x91, 0x3a, 0x8c, 0xf5, 0x85, 0x7e, 0xff, 0x00, 0xa9,
]);
pub const TOKEN_2022_PROGRAM_ID: Address = Address::new_from_array([
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
]);
/// An SPL token account, classic layout.
pub const TOKEN_ACCOUNT_LEN: usize = 165;
pub const MAX_SEATS_PER_SALE: u64 = 150;
pub const MAX_SEATS_PER_HOLDER: u64 = 150;
/// Hexagons silent for this long leave the quorum denominator (~30 days).
pub const ACTIVE_WINDOW_SLOTS: u64 = 6_480_000;

// The geometry. Twelve pentagons always; the hexagon floor grows only along
// the admissible ladder, H = 10(T − 1), T = m² + mn + n². The rung index
// is what a registry stores; the counts are what the ladder permits.
pub const PENTAGONS: usize = 12;
/// Twelve is the budget, never the roster (his Nov 1 2026 "12 pentagons/
/// companions is a budget it is not a requirement"): the chairs' chamber
/// passes on more than half of the chairs lit, seven when all twelve are.
pub const fn chair_quorum(lit: u8) -> u8 {
    lit / 2 + 1
}
pub const PENTAGON_QUORUM: u8 = chair_quorum(PENTAGONS as u8);
pub const LADDER: [(u8, u8); 6] = [(1, 0), (1, 1), (2, 0), (3, 0), (2, 2), (4, 0)];
pub const fn ladder_t(m: u8, n: u8) -> u64 {
    let (m, n) = (m as u64, n as u64);
    m * m + m * n + n * n
}
pub const fn ladder_hexagons(rung: usize) -> u64 {
    let (m, n) = LADDER[rung];
    10 * (ladder_t(m, n) - 1)
}
pub const MAX_HOLDERS: usize = 150;
/// The ceiling on a holder's weight: with at most MAX_SEATS_PER_HOLDER seats
/// a holder and MAX_HOLDERS holders, no sum of seats × weight can overflow a
/// u64, so no seal of the unit can fail on arithmetic.
pub const MAX_WEIGHT: u64 = u64::MAX / (MAX_SEATS_PER_HOLDER * MAX_HOLDERS as u64);
/// The sessions `propose` opens with the proposal, the origin's among them.
/// Every other crew of the decision opens its own (`IX_CONVENE`), so this
/// bounds one transaction, never a tree.
pub const MAX_IMPACTED: usize = 12;
/// The high bit of a proposal's class byte: the proposer asks for consent,
/// every chamber of the decision must seal PASSED before it executes. Off,
/// a crew's verdict is its chair's vote and the origin's seal alone decides.
pub const CLASS_CONSENT: u8 = 0x80;
/// A second bit of a proposal's class byte: the proposer asks that a split
/// of the origin's two chambers be settled by the SUM (below) instead of
/// standing as DEADLOCK. Granted only for the kinds the origin's own law
/// turned it on for (`CODEX_RESOLUTION`), never together with consent.
pub const CLASS_SUM: u8 = 0x40;
/// A third bit: the proposer asks that a split of the origin's two chambers, the chairs for and the floor against, go to the market:
/// PASSED when the proposal's road passed. The chairs keep their veto: a split where they refused stands, whatever the road says.
/// Granted only for the classes the origin's own law names (`CODEX_MARKET`, its `split`).
pub const CLASS_MARKET: u8 = 0x20;
/// A fourth bit: a plan. The origin's floor speaks by price, not by ballot: PASSED when the chairs passed AND the proposal's road
/// passed, DEADLOCK when only one of the two did, REJECTED when neither. Granted only for the classes the origin's law names (its `plan`).
pub const CLASS_PLAN: u8 = 0x10;
/// Every bit a class byte may carry above its class. A proposal carries at most one of them.
pub const CLASS_BITS: u8 = CLASS_CONSENT | CLASS_SUM | CLASS_MARKET | CLASS_PLAN;
/// The class of a class byte, without its bits: the one way every reader strips them.
pub const fn class_of(class_byte: u8) -> u8 {
    class_byte & !CLASS_BITS
}
/// The only classes whose decision may ever go to a market: money, seats, the hands. The law (CODEX) and the signals (the code, the
/// weights, the engine's own rulings) stay under the AND for ever. A plan may be money or the hands only: a seat is never decided with
/// no ballot of the floor.
pub const MARKET_TREASURY: u8 = 1 << 0;
pub const MARKET_SEAT: u8 = 1 << 1;
pub const MARKET_ACT: u8 = 1 << 2;
pub const MARKET_CLASSES: u8 = MARKET_TREASURY | MARKET_SEAT | MARKET_ACT;
pub const PLAN_CLASSES: u8 = MARKET_TREASURY | MARKET_ACT;
/// The market bit of a class (the class without its bits), 0 when the class can never ask the market.
pub const fn market_bit(class: u8) -> u8 {
    match class {
        CLASS_TREASURY => MARKET_TREASURY,
        CLASS_SEAT => MARKET_SEAT,
        CLASS_ACT => MARKET_ACT,
        _ => 0,
    }
}
/// THE BAR: a road counts when the law asks no share (`min_stood_bp == 0`), or when what stood in it as deposits is at least that
/// share of what was in hands on its board when the road was laid (`credited_at_lay`, which the road keeps: the board's `credited` less
/// what its house had `earned`, which is in nobody's balance and can stand in no road; a later credit cannot move it). A road nobody
/// deposited in never counts, whatever the bar (`seal`). A road nobody else trades is decided by whoever trades it, for nothing (`poleis-futarchy`, WHAT THE LAW CANNOT DO
/// ALONE): this is how much of the board must have stood in it to speak for the floor. It measures how much stood in a road, not how
/// many stood there: one depositor of the required share passes it alone, and a crediter can fund that depositor. What it refuses
/// is the small trader alone in a road.
pub const fn stood_enough(deposits: u64, credited_at_lay: u64, min_stood_bp: u16) -> bool {
    min_stood_bp == 0 || (deposits as u128) * 10_000 >= (min_stood_bp as u128) * (credited_at_lay as u128)
}
/// The only kinds that can ever take the SUM: a lot opened, a ruling on a contract, and the two plain lamport payments (at or under
/// the unit's small cap). None of them changes the law, nor who votes, nor with what weight, so a small thing may pass on the sum of
/// the two halves while everything that rewrites the society stays under the AND for ever.
pub const SUM_OPEN_AUCTION: u8 = 1 << 0;
pub const SUM_RULE: u8 = 1 << 1;
pub const SUM_TRANSFER: u8 = 1 << 2;
pub const SUM_BUDGET: u8 = 1 << 3;
pub const SUM_KINDS: u8 = 0x0F;
/// The SUM bit of a (class, kind), 0 when the kind can never take the SUM. `class` is the class without its bits.
pub const fn sum_bit(class: u8, kind: u8) -> u8 {
    match (class, kind) {
        (CLASS_SIGNAL, SIGNAL_OPEN_AUCTION) => SUM_OPEN_AUCTION,
        (CLASS_SIGNAL, SIGNAL_RULE) => SUM_RULE,
        (CLASS_TREASURY, TREASURY_TRANSFER) => SUM_TRANSFER,
        (CLASS_TREASURY, TREASURY_BUDGET) => SUM_BUDGET,
        _ => 0,
    }
}
/// The SUM, his rule of Mar 22 2026: "the votes are divided by 50% hexagons and 50% pentagons... to pass anything they just need the
/// 51% compounded (for example, 30% pentagons say yes, 21% hexagons say yes, it passes)" and "if 5/12 pentagons vote yes but 80%
/// hexagons voted yes, it passes". Each chamber is half the vote, the two halves must reach 51 %, and no chamber may be under a
/// third, so a chamber still blocks, but only with two thirds of itself. `pent_pro` of `lit` chairs, `hex_pro` of `active` weight.
pub const fn sum_passes(pent_pro: u8, lit: u8, hex_pro: u64, active: u64) -> bool {
    if lit == 0 || active == 0 {
        return false;
    }
    let (p, l, h, a) = (pent_pro as u128, lit as u128, hex_pro as u128, active as u128);
    3 * p >= l && 3 * h >= a && 100 * (p * a + h * l) >= 102 * l * a
}
pub const MAX_DEPTH: u8 = 5;
/// The root of the founding tree: the one polis, per program, whose CODEX
/// governs the code itself. Every other tree is seeded by its own root id.
pub const ROOT_UNIT: u64 = 0;
pub const NO_CHAIR: u8 = 0xFF;

// Config modes: a sole-operator estate, an estate whose genesis sale is open
// (frozen until ratified), and a council under the double seal.
pub const MODE_SOLO: u8 = 0;
pub const MODE_GENESIS: u8 = 1;
pub const MODE_COUNCIL: u8 = 2;

// Decision classes (the Mar 31 spec). Every class is double sealed, and every
// seal waits for the crews in its lit chairs; CODEX checks that closure again
// at execute, against a crew seated after its unit sealed. ACT is the
// society's hands: an instruction to any other program, signed by the unit's
// agent key, never by its vault.
pub const CLASS_CODEX: u8 = 0;
pub const CLASS_SIGNAL: u8 = 1;
pub const CLASS_TREASURY: u8 = 2;
pub const CLASS_SEAT: u8 = 3;
pub const CLASS_ACT: u8 = 4;

// Payload kinds, per class.
pub const CODEX_PLAN: u8 = 0;
pub const CODEX_LADDER: u8 = 1;
/// The ladder's two doors, the payload's 33rd byte (absent: a sale): a sealed
/// sale of the new seats, or admission, where the spots open to knocks and
/// the polis seats whom it admits, one decision each, at the price the climb named.
pub const LADDER_SALE: u8 = 0;
pub const LADDER_DOOR: u8 = 1;
pub const CODEX_FOUND: u8 = 2;
pub const CODEX_WINDOW: u8 = 3;
pub const CODEX_NAME: u8 = 4;
/// Income by rule: a fraction of every inflow escrowed for the floor per period.
pub const CODEX_DIVIDEND_POLICY: u8 = 5;
/// The code itself: the root, over the whole tree, upgrades the program
/// from a buffer, hands its upgrade authority on, or burns it. Kind 6 is
/// Poleis's own code; `CODEX_UPGRADE_PROGRAM` is any program the seal holds.
pub const CODEX_UPGRADE: u8 = 6;
/// The upgrade authority of a program the seal holds, handed on or burned: `[new:32][program:32]`, the program named by the act itself.
pub const CODEX_UPGRADE_AUTHORITY: u8 = 7;
/// What a crank is paid per landed batch and per seal, from the treasury, under the plan.
pub const CODEX_CRANK_TIP: u8 = 8;
/// A mint the society recognizes beyond the default, with its own cap per
/// period in that mint's units; cap zero withdraws it.
pub const CODEX_ALLOW_MINT: u8 = 9;
/// The constitution of the fund, by the root polis alone: the fee share,
/// the cycle, the payout range, the option fee, the chair quorum.
pub const CODEX_FUND: u8 = 10;
/// The unit's own law on how a split is settled: which kinds may ask for the SUM, and the small cap of a treasury kind that asks.
pub const CODEX_RESOLUTION: u8 = 11;
/// The unit's own law on its market: the roads program and the board whose roads it reads, the classes whose split goes to the
/// market, the classes that may be plans, and the least share of the board that must stand in a road for its verdict to count.
pub const CODEX_MARKET: u8 = 12;
/// A buffer of the loader's that the seal holds (an upgrade's, handed to the upgrade PDA before its vote and never carried out), closed:
/// `[buffer:32][recipient:32]`, its lamports to the recipient the act names. A buffer and nothing else: never a program's data.
pub const CODEX_CLOSE_BUFFER: u8 = 13;
/// The code of a program the seal holds, by name: `[buffer:32][spill:32][program:32]`, the named program's code replaced from the buffer, as
/// `CODEX_UPGRADE` replaces Poleis's own. Every program handed to the seal (the market, the $WARM program) is upgraded by a vote, no key between.
pub const CODEX_UPGRADE_PROGRAM: u8 = 14;
pub const SIGNAL_SALE_DEFAULTS: u8 = 0;
pub const SIGNAL_WEIGHT: u8 = 1;
pub const SIGNAL_OPEN_AUCTION: u8 = 2;
/// A ruling on a contract: release to the payee or refund, by the paying
/// unit itself or by the arbiter unit the contract named.
pub const SIGNAL_RULE: u8 = 3;
/// The root polis vets an option of the fund; a chair backs one.
pub const SIGNAL_VET_OPTION: u8 = 4;
pub const SIGNAL_BACK_OPTION: u8 = 5;
pub const TREASURY_TRANSFER: u8 = 0;
pub const TREASURY_BUDGET: u8 = 1;
pub const TREASURY_DIVIDEND: u8 = 2;
pub const TREASURY_DISSOLVE: u8 = 3;
pub const TREASURY_TOKEN_TRANSFER: u8 = 4;
pub const TREASURY_TOKEN_BUDGET: u8 = 5;
/// A dividend in a token; the one payload a dissolved unit may still pass.
pub const TREASURY_TOKEN_DIVIDEND: u8 = 6;
/// A standing authorization: the payee pulls up to a cap per period.
pub const TREASURY_ALLOWANCE: u8 = 7;
pub const TREASURY_REVOKE: u8 = 8;
/// Escrow released by a ruling, refunded at the deadline.
pub const TREASURY_CONTRACT: u8 = 9;
/// Rulings on a contract.
pub const RULE_RELEASE: u8 = 1;
pub const RULE_REFUND: u8 = 2;
/// Receipt kinds: the structured log every money move and seal leaves.
pub const RECEIPT_DOMAIN: &[u8] = b"poleis:rcpt1";
pub const RCPT_SEAL: u8 = 1;
pub const RCPT_PAY: u8 = 2;
pub const RCPT_PULL: u8 = 3;
pub const RCPT_ESCROW: u8 = 4;
pub const RCPT_RELEASE: u8 = 5;
pub const RCPT_REFUND: u8 = 6;
pub const RCPT_DIVIDEND: u8 = 7;
pub const RCPT_CLAIM: u8 = 8;
pub const RCPT_SEATS: u8 = 9;
pub const RCPT_TIP: u8 = 10;
/// The fund: a fee taken, a grant paid, a stake placed (outcome 1) or lifted (0).
pub const RCPT_FEE: u8 = 11;
pub const RCPT_GRANT: u8 = 12;
pub const RCPT_STAKE: u8 = 13;
/// The door: a knocker admitted (outcome 1) or refused (0); spots opened by a climb.
pub const RCPT_ADMIT: u8 = 14;
pub const RCPT_DOOR: u8 = 15;
/// An option is named by its recipient and its manifest under this domain.
pub const OPTION_DOMAIN: &[u8] = b"poleis:option:v1";
/// A fund cycle: between a voting window's floor and about a year.
pub const MIN_CYCLE_SLOTS: u64 = MIN_WINDOW_SLOTS;
pub const MAX_CYCLE_SLOTS: u64 = MAX_TERM_SLOTS;
/// The one address of the fund: `["fund"]` under this program, derived once
/// and fixed here so a fee site verifies it by comparison, not by search.
pub const FUND_ADDRESS: Address = Address::new_from_array([
    0x64, 0xaf, 0xda, 0x4a, 0x15, 0x0c, 0xf7, 0xd3, 0x62, 0xfb, 0xb3, 0x16, 0xed, 0x05, 0xaa, 0x9d,
    0xce, 0x98, 0x2d, 0x0a, 0x53, 0xf1, 0x2d, 0x0f, 0x5a, 0x9e, 0x78, 0x47, 0xb5, 0xfd, 0x73, 0x9b,
]);
pub const FUND_BUMP: u8 = 255;
pub const SEAT_ATTACH: u8 = 0;
pub const SEAT_RECALL: u8 = 1;
pub const SEAT_JET: u8 = 2;
/// The door: a knocker admitted to one seat at the price it deposited, or refused.
pub const SEAT_ADMIT: u8 = 3;
pub const SEAT_REFUSE: u8 = 4;
pub const ACT_RAW: u8 = 0;
/// Raw act account flags.
pub const ACT_WRITABLE: u8 = 1;
pub const ACT_SIGNER: u8 = 2;

// Votes and session states.
pub const VOTE_ABSTENTIO: u8 = 0;
pub const VOTE_PRO: u8 = 1;
pub const VOTE_CONTRA: u8 = 2;
pub const STATE_VOTING: u8 = 0;
pub const STATE_PASSED: u8 = 1;
pub const STATE_REJECTED: u8 = 2;
pub const STATE_DEADLOCK: u8 = 3;
pub const STATE_EXPIRED: u8 = 4;

// Holder kinds: a key that signs, or a unit that speaks through its own seal.
pub const HOLDER_EMPTY: u8 = 0;
pub const HOLDER_KEY: u8 = 1;
pub const HOLDER_UNIT: u8 = 2;
/// A seat controlled only by its current NFT owner, never by legacy transfers.
pub const HOLDER_NFT: u8 = 3;
/// A permanent row reservation after burn/expulsion; historical bits cannot
/// be inherited by another holder while sessions/distributions remain alive.
pub const HOLDER_RETIRED_NFT: u8 = 4;

/// The Ed25519 precompile: `Ed25519SigVerify111111111111111111111111111`.
pub const ED25519_PROGRAM_ID: Address = Address::new_from_array([
    0x03, 0x7d, 0x46, 0xd6, 0x7c, 0x93, 0xfb, 0xbe, 0x12, 0xf9, 0x42, 0x8f, 0x83, 0x8d, 0x40, 0xff,
    0x05, 0x70, 0x74, 0x49, 0x27, 0xf4, 0x8a, 0x64, 0xfc, 0xca, 0x70, 0x44, 0x80, 0x00, 0x00, 0x00,
]);

fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if data.starts_with(&seat_nft::EXECUTE_DISCRIMINATOR) {
        return ix::nft::transfer_hook(accounts, data);
    }
    let (tag, rest) = data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    match *tag {
        IX_INIT_CONFIG => ix::treasury::init_config(accounts, rest),
        IX_CREATE_TREASURY => ix::treasury::create_treasury(accounts, rest),
        IX_ENDOW => ix::treasury::endow(accounts, rest),
        IX_SYNC_ENDOWMENT => ix::treasury::sync_endowment(accounts, rest),
        IX_OPEN_AUCTION => ix::auction::open_auction(accounts, rest),
        IX_COMMIT_BID => ix::auction::commit_bid(accounts, rest),
        IX_REVEAL_BID => ix::auction::reveal_bid(accounts, rest),
        IX_SETTLE => ix::auction::settle(accounts, rest),
        IX_RECLAIM => ix::auction::reclaim(accounts, rest),
        IX_COMMIT_SEAT_BID => ix::sale::commit_seat_bid(accounts, rest),
        IX_REVEAL_SEAT_BID => ix::sale::reveal_seat_bid(accounts, rest),
        IX_SETTLE_SEATS => ix::sale::settle_seats(accounts, rest),
        IX_CLAIM_SEAT => ix::sale::claim_seat(accounts, rest),
        IX_RECLAIM_SEAT_BID => ix::sale::reclaim_seat_bid(accounts, rest),
        IX_TRANSFER_SEATS => ix::council::transfer_seats(accounts, rest),
        IX_OPEN_GENESIS => ix::council::open_genesis(accounts, rest),
        IX_FOUND_CHILD => ix::council::found_child(accounts, rest),
        IX_RATIFY_GENESIS => ix::council::ratify_genesis(accounts, rest),
        IX_PROPOSE => ix::proposal::propose(accounts, rest),
        IX_ATTEST => ix::proposal::attest(accounts, rest),
        IX_MANDATE => ix::proposal::mandate(accounts, rest),
        IX_SEAL => ix::proposal::seal(accounts, rest),
        IX_EXECUTE => ix::proposal::execute(accounts, rest),
        IX_CLAIM_DIVIDEND => ix::proposal::claim_dividend(accounts, rest),
        IX_CLOSE_PROPOSAL => ix::proposal::close_proposal(accounts, rest),
        IX_CLOSE_SEAT_BID => ix::sale::close_seat_bid(accounts, rest),
        IX_CLOSE_BID => ix::auction::close_bid(accounts, rest),
        IX_CREATE_VAULT => ix::vault::create_vault(accounts, rest),
        IX_PULL => ix::economy::pull(accounts, rest),
        IX_CLOSE_ALLOWANCE => ix::economy::close_allowance(accounts, rest),
        IX_EXPIRE_CONTRACT => ix::economy::expire_contract(accounts, rest),
        IX_ACCRUE_DIVIDEND => ix::economy::accrue_dividend(accounts, rest),
        IX_CLOSE_DISTRIBUTION => ix::economy::close_distribution(accounts, rest),
        IX_LIST_SEATS => ix::market::list_seats(accounts, rest),
        IX_BUY_SEATS => ix::market::buy_seats(accounts, rest),
        IX_CANCEL_LISTING => ix::market::cancel_listing(accounts, rest),
        IX_SYNC_ASSEMBLY_FLOOR => assembly::sync_assembly_floor(accounts, rest),
        IX_SEAT_CHAIR => ix::fund::seat_chair(accounts, rest),
        IX_PROPOSE_OPTION => ix::fund::propose_option(accounts, rest),
        IX_STAKE => ix::fund::stake(accounts, rest),
        IX_UNSTAKE => ix::fund::unstake(accounts, rest),
        IX_SETTLE_CYCLE => ix::fund::settle_cycle(accounts, rest),
        IX_CLAIM_GRANT => ix::fund::claim_grant(accounts, rest),
        IX_OPEN_NFT_GENESIS => ix::nft_sale::open_nft_genesis(accounts, rest),
        IX_MINT_NFT => ix::nft_sale::mint_nft(accounts, rest),
        IX_RATIFY_NFT_GENESIS => ix::nft_sale::ratify_nft_genesis(accounts, rest),
        IX_ATTEST_NFT => ix::proposal::attest_nft(accounts, rest),
        IX_PROPOSE_NFT => ix::proposal::propose_nft(accounts, rest),
        IX_CLAIM_DIVIDEND_NFT => ix::proposal::claim_dividend_nft(accounts, rest),
        IX_RETIRE_BURNED_NFT => ix::nft::retire_burned(accounts, rest),
        IX_WITHDRAW_RETIRED_NFT => ix::nft::withdraw_retired(accounts, rest),
        IX_ATTEST_BLS => ix::bls::attest_bls(accounts, rest),
        IX_REGISTER_BLS_KEY => ix::bls::register_bls_key(accounts, rest),
        IX_CONVENE => ix::proposal::convene(accounts, rest),
        IX_ADJOURN => ix::proposal::adjourn(accounts, rest),
        IX_KNOCK => ix::door::knock(accounts, rest),
        IX_WITHDRAW_KNOCK => ix::door::withdraw_knock(accounts, rest),
        IX_GROW_FUND => ix::fund::grow_fund(accounts, rest),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
