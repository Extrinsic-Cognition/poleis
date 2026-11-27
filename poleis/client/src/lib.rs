//! The one place that knows how to talk to poleis_treasury from the host:
//! program id, PDA derivations, instruction encoding, the hashes, the vote
//! envelope and the Ed25519 batch, error codes, and read-only views over
//! account bytes. Every offset, size, tag and code comes from the program
//! crate itself, so the client cannot drift from the law it speaks to.

use sha2::{Digest, Sha256};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

#[cfg(feature = "bls")]
pub mod bls;
pub mod nft;
pub mod nft_receipt;

pub use poleis_treasury::error::PoleisError;
pub use poleis_treasury::assembly::{AssemblyFloor, DISC_ASSEMBLY_FLOOR};
pub use poleis_treasury::state::{
    holder_unit, unit_holder, Allowance, Auction, Bid, Candidate, Chair, Config, Contract, Distribution, FeeTally, Fund, Listing,
    Proposal, Registry, Rule, SeatBid, SeatSale, Session, Stake, TokenPlan, Treasury, DISC_ALLOWANCE,
    DISC_AUCTION, DISC_BID, DISC_CONFIG, DISC_CONTRACT, DISC_DISTRIBUTION, DISC_FEE_TALLY, DISC_FUND, DISC_LISTING,
    DISC_OPTION, DISC_PROPOSAL, DISC_REGISTRY, DISC_RULE, DISC_SEAT_BID, DISC_SEAT_SALE, DISC_SESSION, DISC_STAKE, DISC_TOKEN_PLAN,
    DISC_TREASURY, LAYOUT_VERSION, NO_ROOT, OFF_DISC, OFF_VERSION, Knock, Pool, DISC_KNOCK, DISC_POOL,
};
pub use poleis_treasury::util::{ENVELOPE_LEN, RECEIPT_LEN, VOTE_DOMAIN};
pub use poleis_treasury::state::{BlsKey, DISC_BLS_KEY};
pub use poleis_treasury::{
    chair_quorum, ladder_hexagons, ladder_t, ACTIVE_WINDOW_SLOTS, ACT_RAW, ACT_SIGNER, ACT_WRITABLE, BPS,
    CLASS_ACT, CLASS_CODEX, CLASS_SEAT, CLASS_SIGNAL, CLASS_CONSENT, CLASS_TREASURY, CODEX_ALLOW_MINT, CODEX_CRANK_TIP, CODEX_FUND,
    CLASS_SUM, CLASS_MARKET, CLASS_PLAN, CLASS_BITS, class_of, CODEX_RESOLUTION, SUM_BUDGET, SUM_KINDS, SUM_OPEN_AUCTION, SUM_RULE, SUM_TRANSFER, sum_bit, sum_passes,
    FUND_ADDRESS, FUND_BUMP, MAX_CYCLE_SLOTS, MIN_CYCLE_SLOTS, OPTION_DOMAIN, RCPT_FEE, RCPT_GRANT, RCPT_STAKE, SIGNAL_BACK_OPTION, SIGNAL_VET_OPTION,
    LADDER_DOOR, LADDER_SALE, RCPT_ADMIT, RCPT_DOOR, SEAT_ADMIT, SEAT_REFUSE,
    CODEX_DIVIDEND_POLICY, CODEX_FOUND, CODEX_LADDER, CODEX_NAME, CODEX_PLAN, CODEX_UPGRADE,
    CODEX_CLOSE_BUFFER, CODEX_UPGRADE_AUTHORITY, CODEX_UPGRADE_PROGRAM, CODEX_WINDOW, CODEX_WINDOW_FACTOR,
    DISTRIBUTION_TTL_SLOTS, HOLDER_EMPTY, HOLDER_KEY, HOLDER_UNIT, LADDER, MAX_ACT_ACCOUNTS,
    MAX_DEPTH, MAX_DESC, MAX_HOLDERS, MAX_IMPACTED, MAX_NAME, MAX_PAYLOAD, MAX_SEATS_PER_HOLDER,
    MAX_SEATS_PER_SALE, MAX_TERM_SLOTS, MAX_WINDOW_SLOTS, MIN_WINDOW_SLOTS, MODE_COUNCIL,
    MODE_GENESIS, MODE_SOLO, NO_CHAIR, PENTAGONS, PENTAGON_QUORUM, RCPT_CLAIM, RCPT_DIVIDEND,
    RCPT_ESCROW, RCPT_PAY, RCPT_PULL, RCPT_REFUND, RCPT_RELEASE, RCPT_SEAL, RCPT_SEATS, RCPT_TIP,
    RECEIPT_DOMAIN, ROOT_UNIT, RULE_REFUND, RULE_RELEASE, SEAT_ATTACH, SEAT_JET, SEAT_RECALL,
    SIGNAL_OPEN_AUCTION, SIGNAL_RULE, SIGNAL_SALE_DEFAULTS, SIGNAL_WEIGHT, STATE_DEADLOCK,
    STATE_EXPIRED, STATE_PASSED, STATE_REJECTED, STATE_VOTING, TOKEN_ACCOUNT_LEN,
    TREASURY_ALLOWANCE, TREASURY_BUDGET, TREASURY_CONTRACT, TREASURY_DISSOLVE, TREASURY_DIVIDEND,
    TREASURY_REVOKE, TREASURY_TOKEN_BUDGET, TREASURY_TOKEN_DIVIDEND, TREASURY_TOKEN_TRANSFER,
    TREASURY_TRANSFER, VOTE_ABSTENTIO, VOTE_CONTRA, VOTE_PRO,
};

/// The program id, taken from the program crate, never retyped.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(poleis_treasury::ID.to_bytes());
/// The two SPL Token programs a vault may live under, from the program crate.
pub const TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array(poleis_treasury::TOKEN_PROGRAM_ID.to_bytes());
pub const TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::new_from_array(poleis_treasury::TOKEN_2022_PROGRAM_ID.to_bytes());
/// The BPF upgradeable loader and the two sysvars an upgrade needs.
pub const BPF_LOADER_UPGRADEABLE_ID: Pubkey = Pubkey::new_from_array(poleis_treasury::BPF_LOADER_UPGRADEABLE_ID.to_bytes());
pub const RENT_SYSVAR_ID: Pubkey = solana_pubkey::pubkey!("SysvarRent111111111111111111111111111111111");
pub const CLOCK_SYSVAR_ID: Pubkey = solana_pubkey::pubkey!("SysvarC1ock11111111111111111111111111111111");
/// The Ed25519 precompile and the instructions sysvar, from the program crate.
pub const ED25519_PROGRAM_ID: Pubkey = Pubkey::new_from_array(poleis_treasury::ED25519_PROGRAM_ID.to_bytes());
pub const INSTRUCTIONS_SYSVAR_ID: Pubkey =
    Pubkey::new_from_array(pinocchio_sysvar_instructions_id());
const fn pinocchio_sysvar_instructions_id() -> [u8; 32] {
    [
        0x06, 0xa7, 0xd5, 0x17, 0x18, 0x7b, 0xd1, 0x66, 0x35, 0xda, 0xd4, 0x04, 0x55, 0xfd, 0xc2,
        0xc0, 0xc1, 0x24, 0xc6, 0x8f, 0x21, 0x56, 0x75, 0xa5, 0xdb, 0xba, 0xcb, 0x5f, 0x08, 0x00,
        0x00, 0x00,
    ]
}

/// Instruction tags, re-exported from the program.
pub mod tag {
    pub use poleis_treasury::{
        IX_ACCRUE_DIVIDEND, IX_ATTEST, IX_BUY_SEATS, IX_CANCEL_LISTING, IX_CLAIM_DIVIDEND,
        IX_CLAIM_SEAT, IX_CLOSE_ALLOWANCE, IX_CLOSE_BID, IX_CLOSE_DISTRIBUTION, IX_CLOSE_PROPOSAL,
        IX_CLOSE_SEAT_BID, IX_COMMIT_BID, IX_COMMIT_SEAT_BID, IX_CREATE_TREASURY, IX_CREATE_VAULT,
        IX_ENDOW, IX_EXECUTE, IX_EXPIRE_CONTRACT, IX_FOUND_CHILD, IX_INIT_CONFIG, IX_LIST_SEATS,
        IX_MANDATE, IX_OPEN_AUCTION, IX_OPEN_GENESIS, IX_PROPOSE, IX_PULL, IX_RATIFY_GENESIS,
        IX_RECLAIM, IX_RECLAIM_SEAT_BID, IX_REVEAL_BID, IX_REVEAL_SEAT_BID, IX_SEAL, IX_SETTLE,
        IX_SETTLE_SEATS, IX_SYNC_ENDOWMENT, IX_TRANSFER_SEATS, IX_SYNC_ASSEMBLY_FLOOR,
        IX_SEAT_CHAIR, IX_PROPOSE_OPTION, IX_STAKE, IX_UNSTAKE, IX_SETTLE_CYCLE, IX_KNOCK, IX_WITHDRAW_KNOCK, IX_CLAIM_GRANT,
        IX_ATTEST_BLS, IX_REGISTER_BLS_KEY, IX_CONVENE, IX_ADJOURN, IX_GROW_FUND,
    };
}

/// The numeric code a refusal carries on the wire (`ProgramError::Custom`).
pub const fn code(e: PoleisError) -> u32 {
    e as u32
}

// ---- raw helpers ----

pub fn ix(data: Vec<u8>, metas: Vec<AccountMeta>) -> Instruction {
    Instruction::new_with_bytes(PROGRAM_ID, &data, metas)
}
pub fn system_meta() -> AccountMeta {
    AccountMeta::new_readonly(Pubkey::default(), false)
}
pub fn find_pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &PROGRAM_ID).0
}
/// The public index is namespaced by its source program as well as its unit.
pub fn assembly_floor_pda(source_program: &Pubkey, unit: u64) -> Pubkey {
    find_pda(&[b"assembly-floor", source_program.as_ref(), &unit.to_le_bytes()])
}
/// Copy authenticated current source facts. This confers no voting or spending authority.
pub fn sync_assembly_floor(payer: Pubkey, source_program: Pubkey, unit: u64, source_registry: Pubkey, source_config: Pubkey) -> Instruction {
    ix(data(tag::IX_SYNC_ASSEMBLY_FLOOR, &[source_program.as_ref(), &unit.to_le_bytes()]), vec![
        AccountMeta::new(payer, true), AccountMeta::new_readonly(source_registry, false),
        AccountMeta::new_readonly(source_config, false), AccountMeta::new(assembly_floor_pda(&source_program, unit), false), system_meta(),
    ])
}
fn data(tag: u8, parts: &[&[u8]]) -> Vec<u8> {
    let mut v = vec![tag];
    for p in parts {
        v.extend_from_slice(p);
    }
    v
}
/// A length-prefixed text field: [len:1][bytes].
fn push_text(v: &mut Vec<u8>, t: &[u8]) {
    v.push(t.len() as u8);
    v.extend_from_slice(t);
}
pub fn sha(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

// ---- PDAs ----

/// The founding tree's config: tree 0, whose CODEX governs the code.
pub fn config_pda() -> Pubkey {
    config_pda_of(ROOT_UNIT)
}
/// One config per tree, seeded by its root unit's id.
pub fn config_pda_of(root: u64) -> Pubkey {
    find_pda(&[b"config", &root.to_le_bytes()])
}
pub fn council_pda() -> Pubkey {
    council_pda_of(ROOT_UNIT)
}
/// The authority nobody holds, one per tree.
pub fn council_pda_of(root: u64) -> Pubkey {
    find_pda(&[b"council", &root.to_le_bytes()])
}
pub fn treasury_pda(unit: u64) -> Pubkey {
    find_pda(&[b"treasury", &unit.to_le_bytes()])
}
pub fn registry_pda(unit: u64) -> Pubkey {
    find_pda(&[b"registry", &unit.to_le_bytes()])
}
pub fn auction_pda(auction: u64) -> Pubkey {
    find_pda(&[b"auction", &auction.to_le_bytes()])
}
pub fn bid_pda(auction: u64, bidder: &Pubkey) -> Pubkey {
    find_pda(&[b"bid", &auction.to_le_bytes(), bidder.as_ref()])
}
pub fn sale_pda(unit: u64, sale: u64) -> Pubkey {
    find_pda(&[b"sale", &unit.to_le_bytes(), &sale.to_le_bytes()])
}
pub fn seat_bid_pda(unit: u64, sale: u64, bidder: &Pubkey, nonce: u64) -> Pubkey {
    find_pda(&[b"sbid", &unit.to_le_bytes(), &sale.to_le_bytes(), bidder.as_ref(), &nonce.to_le_bytes()])
}
pub fn proposal_pda(hash: &[u8; 32]) -> Pubkey {
    find_pda(&[b"proposal", hash])
}
pub fn session_pda(hash: &[u8; 32], unit: u64) -> Pubkey {
    find_pda(&[b"session", hash, &unit.to_le_bytes()])
}
pub fn distribution_pda(hash: &[u8; 32]) -> Pubkey {
    find_pda(&[b"distribution", hash])
}
/// The unit's token account for a mint; its authority is the unit's treasury.
pub fn vault_pda(unit: u64, mint: &Pubkey) -> Pubkey {
    find_pda(&[b"vault", &unit.to_le_bytes(), mint.as_ref()])
}
/// The unit's hands: the address that signs raw acts. Never holds the treasury.
pub fn agent_pda(unit: u64) -> Pubkey {
    find_pda(&[b"agent", &unit.to_le_bytes()])
}
pub fn allowance_pda(hash: &[u8; 32]) -> Pubkey {
    find_pda(&[b"allowance", hash])
}
pub fn contract_pda(hash: &[u8; 32]) -> Pubkey {
    find_pda(&[b"contract", hash])
}
/// The escrow token account of a token contract or token distribution.
pub fn escrow_pda(hash: &[u8; 32]) -> Pubkey {
    find_pda(&[b"escrow", hash])
}
pub fn listing_pda(unit: u64, seller: &Pubkey, nonce: u64) -> Pubkey {
    find_pda(&[b"listing", &unit.to_le_bytes(), seller.as_ref(), &nonce.to_le_bytes()])
}
/// A unit's plan for a mint beyond the default, made by CODEX AllowMint.
pub fn token_plan_pda(unit: u64, mint: &Pubkey) -> Pubkey {
    find_pda(&[b"tokenplan", &unit.to_le_bytes(), mint.as_ref()])
}
/// The program's upgrade authority after the ceremony: never created, signed for only by a root CODEX act.
pub fn upgrade_pda() -> Pubkey {
    find_pda(&[b"upgrade"])
}
/// The loader's program-data account of this program.
pub fn programdata_pda() -> Pubkey {
    programdata_of(&PROGRAM_ID)
}
/// The loader's program-data account of any upgradeable program: the one address `CODEX_UPGRADE_AUTHORITY` reaches for the program its payload names.
pub fn programdata_of(program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[program.as_ref()], &BPF_LOADER_UPGRADEABLE_ID).0
}
/// The hash a dividend-policy accrual is keyed by: sha256("poleis:accrual" ‖ unit ‖ seq).
pub fn accrual_hash(unit: u64, seq: u64) -> [u8; 32] {
    sha(&[b"poleis:accrual", &unit.to_le_bytes(), &seq.to_le_bytes()])
}

// ---- the fund ----
/// The one fund of the program.
pub fn fund_pda() -> Pubkey {
    find_pda(&[b"fund"])
}
/// The fees a root generated, this cycle and for life.
pub fn tally_pda(root: u64) -> Pubkey {
    find_pda(&[b"fees", &root.to_le_bytes()])
}
/// An option for where the fund goes, named by its hash.
pub fn option_pda(hash: &[u8; 32]) -> Pubkey {
    find_pda(&[b"option", hash])
}
/// A holder's stake from its seats in `unit` on an option.
pub fn stake_pda(hash: &[u8; 32], unit: u64, holder: &Pubkey) -> Pubkey {
    find_pda(&[b"stake", hash, &unit.to_le_bytes(), holder.as_ref()])
}
// ---- the door ----
/// The spots of a polis open by admission.
pub fn pool_pda(unit: u64) -> Pubkey {
    find_pda(&[b"pool", &unit.to_le_bytes()])
}
/// One key waiting at the door of one polis.
pub fn knock_pda(unit: u64, key: &Pubkey) -> Pubkey {
    find_pda(&[b"knock", &unit.to_le_bytes(), key.as_ref()])
}
/// A unit's own law on how a split is settled: which kinds may ask for the SUM, and its small cap.
pub fn rule_pda(unit: u64) -> Pubkey {
    find_pda(&[b"rule", &unit.to_le_bytes()])
}
/// sha256(OPTION_DOMAIN ‖ recipient ‖ manifest): the same proposal is the same account.
pub fn option_hash(recipient: &Pubkey, manifest: &[u8; 32]) -> [u8; 32] {
    sha(&[OPTION_DOMAIN, recipient.as_ref(), manifest])
}

// ---- hashes and envelopes ----

/// A sealed lot bid: sha256(amount_le ‖ salt ‖ bidder).
pub fn commitment(amount: u64, salt: &[u8; 32], bidder: &Pubkey) -> [u8; 32] {
    sha(&[&amount.to_le_bytes(), salt, bidder.as_ref()])
}
/// A sealed seat bid: sha256(price_le ‖ nonce_le ‖ salt ‖ bidder).
pub fn seat_commitment(price: u64, nonce: u64, salt: &[u8; 32], bidder: &Pubkey) -> [u8; 32] {
    sha(&[&price.to_le_bytes(), &nonce.to_le_bytes(), salt, bidder.as_ref()])
}
/// The 55-byte envelope a holder signs off-chain (`poleis_treasury::util::ENVELOPE_LEN`): one message per
/// (proposal, unit, choice), the same bytes for every voter of that choice.
pub fn vote_envelope(hash: &[u8; 32], unit: u64, choice: u8) -> Vec<u8> {
    poleis_treasury::util::envelope(hash, unit, choice).to_vec()
}
/// A proposal's fingerprint, as the forum computes it: any 32 bytes but zero;
/// this helper derives one from its content and a nonce.
pub fn proposal_hash(class: u8, origin: u64, manifest: &[u8; 32], payload_hash: &[u8; 32], nonce: u64) -> [u8; 32] {
    sha(&[b"poleis:proposal:v1", &[class], &origin.to_le_bytes(), manifest, payload_hash, &nonce.to_le_bytes()])
}

/// One entry of an Ed25519 precompile batch.
pub struct SignedEnvelope {
    pub pubkey: [u8; 32],
    pub signature: [u8; 64],
    pub message: Vec<u8>,
}

/// The Ed25519 precompile instruction for a batch, every reference pointing
/// inside itself (index u16::MAX), laid out exactly as the runtime parses
/// it: [n][pad][n × 14-byte offsets][keys ‖ signatures ‖ distinct messages].
/// Entries that signed the same bytes share one copy of the message, so a
/// batch of one choice carries the envelope once.
pub fn ed25519_instruction(entries: &[SignedEnvelope]) -> Instruction {
    let n = entries.len();
    let mut d = vec![n as u8, 0u8];
    let base = 2 + n * 14;
    let mut body = Vec::new();
    let mut messages: Vec<(Vec<u8>, usize)> = Vec::new();
    let mut refs = Vec::with_capacity(n);
    for e in entries {
        let pk_off = base + body.len();
        body.extend_from_slice(&e.pubkey);
        let sig_off = base + body.len();
        body.extend_from_slice(&e.signature);
        refs.push((sig_off, pk_off));
    }
    let mut msg_offs = Vec::with_capacity(n);
    for e in entries {
        let off = match messages.iter().find(|(m, _)| *m == e.message) {
            Some((_, off)) => *off,
            None => {
                let off = base + body.len();
                body.extend_from_slice(&e.message);
                messages.push((e.message.clone(), off));
                off
            }
        };
        msg_offs.push(off);
    }
    for (i, e) in entries.iter().enumerate() {
        let (sig_off, pk_off) = refs[i];
        for v in [sig_off as u16, u16::MAX, pk_off as u16, u16::MAX, msg_offs[i] as u16, e.message.len() as u16, u16::MAX] {
            d.extend_from_slice(&v.to_le_bytes());
        }
    }
    d.extend_from_slice(&body);
    Instruction { program_id: ED25519_PROGRAM_ID, accounts: vec![], data: d }
}

// ---- money and sales ----

/// `default_mint`: the mint every unit spends under its own token counter
/// without a listing (USDC on a real cluster); `Pubkey::default()` for none.
/// Tree 0; `init_config_of` founds any other tree.
pub fn init_config(payer: Pubkey, default_mint: Pubkey) -> Instruction {
    init_config_of(ROOT_UNIT, payer, default_mint)
}
/// The first instruction of a founding: the config of a new tree, seeded by
/// the id its root unit will carry. Permissionless, but for tree 0, which
/// only Poleis's own upgrade authority founds (ONLY THE DEPLOYER FOUNDS
/// ROOT 0): its program data rides last. The payer is the tree's authority
/// until ratification.
pub fn init_config_of(root: u64, payer: Pubkey, default_mint: Pubkey) -> Instruction {
    let mut accounts = vec![AccountMeta::new(payer, true), AccountMeta::new(config_pda_of(root), false), system_meta()];
    if root == ROOT_UNIT {
        accounts.push(AccountMeta::new_readonly(programdata_pda(), false));
    }
    ix(data(tag::IX_INIT_CONFIG, &[&root.to_le_bytes(), default_mint.as_ref()]), accounts)
}
pub fn create_treasury(authority: Pubkey, unit: u64) -> Instruction {
    create_treasury_of(ROOT_UNIT, authority, unit)
}
pub fn create_treasury_of(root: u64, authority: Pubkey, unit: u64) -> Instruction {
    ix(
        data(tag::IX_CREATE_TREASURY, &[&unit.to_le_bytes()]),
        vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(config_pda_of(root), false),
            AccountMeta::new(treasury_pda(unit), false),
            system_meta(),
        ],
    )
}
pub fn endow(donor: Pubkey, unit: u64, amount: u64) -> Instruction {
    ix(
        data(tag::IX_ENDOW, &[&unit.to_le_bytes(), &amount.to_le_bytes()]),
        vec![AccountMeta::new(donor, true), AccountMeta::new(treasury_pda(unit), false), system_meta()],
    )
}
pub fn sync_endowment(unit: u64) -> Instruction {
    ix(data(tag::IX_SYNC_ENDOWMENT, &[&unit.to_le_bytes()]), vec![AccountMeta::new(treasury_pda(unit), false)])
}
/// accounts: [payer (s,w), treasury, mint, vault (w), token program, system]
/// Permissionless: anyone opens the unit's token account for a mint; the
/// treasury is its authority, so only a passed TREASURY payload moves it.
pub fn create_vault(payer: Pubkey, unit: u64, mint: Pubkey, token_program: Pubkey) -> Instruction {
    ix(
        data(tag::IX_CREATE_VAULT, &[&unit.to_le_bytes()]),
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(treasury_pda(unit), false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(vault_pda(unit, &mint), false),
            AccountMeta::new_readonly(token_program, false),
            system_meta(),
        ],
    )
}

#[derive(Clone, Copy, Debug)]
pub struct LotSpec {
    pub auction_id: u64,
    pub unit: u64,
    pub pool_root: [u8; 32],
    pub manifest: [u8; 32],
    pub commit_end: u64,
    pub reveal_end: u64,
    pub reserve: u64,
    pub deposit: u64,
}
impl LotSpec {
    pub fn bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(112);
        v.extend_from_slice(&self.auction_id.to_le_bytes());
        v.extend_from_slice(&self.unit.to_le_bytes());
        v.extend_from_slice(&self.pool_root);
        v.extend_from_slice(&self.manifest);
        v.extend_from_slice(&self.commit_end.to_le_bytes());
        v.extend_from_slice(&self.reveal_end.to_le_bytes());
        v.extend_from_slice(&self.reserve.to_le_bytes());
        v.extend_from_slice(&self.deposit.to_le_bytes());
        v
    }
}

/// Solo mode only; a council opens lots by SIGNAL proposal.
pub fn open_auction(authority: Pubkey, lot: &LotSpec) -> Instruction {
    open_auction_of(ROOT_UNIT, authority, lot)
}
pub fn open_auction_of(root: u64, authority: Pubkey, lot: &LotSpec) -> Instruction {
    ix(
        data(tag::IX_OPEN_AUCTION, &[&lot.bytes()]),
        vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(config_pda_of(root), false),
            AccountMeta::new_readonly(treasury_pda(lot.unit), false),
            AccountMeta::new(auction_pda(lot.auction_id), false),
            system_meta(),
        ],
    )
}
pub fn commit_bid(bidder: Pubkey, auction: u64, c: [u8; 32]) -> Instruction {
    ix(
        data(tag::IX_COMMIT_BID, &[&auction.to_le_bytes(), &c]),
        vec![
            AccountMeta::new(bidder, true),
            AccountMeta::new_readonly(auction_pda(auction), false),
            AccountMeta::new(bid_pda(auction, &bidder), false),
            system_meta(),
        ],
    )
}
pub fn reveal_bid(bidder: Pubkey, auction: u64, amount: u64, salt: [u8; 32]) -> Instruction {
    ix(
        data(tag::IX_REVEAL_BID, &[&auction.to_le_bytes(), &amount.to_le_bytes(), &salt]),
        vec![
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new(auction_pda(auction), false),
            AccountMeta::new(bid_pda(auction, &bidder), false),
        ],
    )
}
/// `root` is the unit's tree root, or the unit itself for a solo unit with no registry.
pub fn settle(crank: Pubkey, auction: u64, unit: u64, winner: Option<Pubkey>, root: u64) -> Instruction {
    let mut metas = vec![
        AccountMeta::new(crank, true),
        AccountMeta::new(auction_pda(auction), false),
        AccountMeta::new(treasury_pda(unit), false),
        AccountMeta::new_readonly(registry_pda(unit), false),
        AccountMeta::new(fund_pda(), false),
        AccountMeta::new(tally_pda(root), false),
        system_meta(),
    ];
    if let Some(w) = winner {
        metas.push(AccountMeta::new(bid_pda(auction, &w), false));
        metas.push(AccountMeta::new(w, false));
    }
    ix(data(tag::IX_SETTLE, &[&auction.to_le_bytes()]), metas)
}
pub fn reclaim(bidder: Pubkey, auction: u64) -> Instruction {
    ix(
        data(tag::IX_RECLAIM, &[&auction.to_le_bytes()]),
        vec![
            AccountMeta::new(bidder, true),
            AccountMeta::new_readonly(auction_pda(auction), false),
            AccountMeta::new(bid_pda(auction, &bidder), false),
        ],
    )
}

pub fn commit_seat_bid(bidder: Pubkey, unit: u64, sale: u64, nonce: u64, c: [u8; 32]) -> Instruction {
    ix(
        data(tag::IX_COMMIT_SEAT_BID, &[&unit.to_le_bytes(), &sale.to_le_bytes(), &nonce.to_le_bytes(), &c]),
        vec![
            AccountMeta::new(bidder, true),
            AccountMeta::new_readonly(sale_pda(unit, sale), false),
            AccountMeta::new(seat_bid_pda(unit, sale, &bidder, nonce), false),
            system_meta(),
        ],
    )
}
pub fn reveal_seat_bid(bidder: Pubkey, unit: u64, sale: u64, nonce: u64, price: u64, salt: [u8; 32]) -> Instruction {
    ix(
        data(
            tag::IX_REVEAL_SEAT_BID,
            &[&unit.to_le_bytes(), &sale.to_le_bytes(), &nonce.to_le_bytes(), &price.to_le_bytes(), &salt],
        ),
        vec![
            AccountMeta::new_readonly(bidder, true),
            AccountMeta::new(sale_pda(unit, sale), false),
            AccountMeta::new(seat_bid_pda(unit, sale, &bidder, nonce), false),
        ],
    )
}
pub fn settle_seats(crank: Pubkey, unit: u64, sale: u64) -> Instruction {
    ix(
        data(tag::IX_SETTLE_SEATS, &[&unit.to_le_bytes(), &sale.to_le_bytes()]),
        vec![
            AccountMeta::new_readonly(crank, true),
            AccountMeta::new(sale_pda(unit, sale), false),
            AccountMeta::new(registry_pda(unit), false),
        ],
    )
}
/// `root` is the unit's tree root: the fund's tally of fees is kept per root.
pub fn claim_seat(bidder: Pubkey, unit: u64, sale: u64, nonce: u64, root: u64) -> Instruction {
    ix(
        data(tag::IX_CLAIM_SEAT, &[&unit.to_le_bytes(), &sale.to_le_bytes(), &nonce.to_le_bytes()]),
        vec![
            AccountMeta::new(bidder, true),
            AccountMeta::new_readonly(sale_pda(unit, sale), false),
            AccountMeta::new(seat_bid_pda(unit, sale, &bidder, nonce), false),
            AccountMeta::new(registry_pda(unit), false),
            AccountMeta::new(treasury_pda(unit), false),
            system_meta(),
            AccountMeta::new(fund_pda(), false),
            AccountMeta::new(tally_pda(root), false),
        ],
    )
}
/// The registry rides along read-only: a winner of a dissolved unit's sale reclaims with it.
pub fn reclaim_seat_bid(bidder: Pubkey, unit: u64, sale: u64, nonce: u64) -> Instruction {
    ix(
        data(tag::IX_RECLAIM_SEAT_BID, &[&unit.to_le_bytes(), &sale.to_le_bytes(), &nonce.to_le_bytes()]),
        vec![
            AccountMeta::new(bidder, true),
            AccountMeta::new_readonly(sale_pda(unit, sale), false),
            AccountMeta::new(seat_bid_pda(unit, sale, &bidder, nonce), false),
            AccountMeta::new_readonly(registry_pda(unit), false),
        ],
    )
}
pub fn transfer_seats(holder: Pubkey, unit: u64, to: [u8; 32], to_kind: u8, count: u64) -> Instruction {
    ix(
        data(tag::IX_TRANSFER_SEATS, &[&unit.to_le_bytes(), &to, &[to_kind], &count.to_le_bytes()]),
        vec![AccountMeta::new(holder, true), AccountMeta::new(registry_pda(unit), false), system_meta()],
    )
}

// ---- genesis ----

#[derive(Clone, Debug)]
pub struct GenesisSpec {
    pub charter: [u8; 32],
    pub plan_cap: u64,
    pub plan_period: u64,
    pub window: u64,
    pub rung: u8,
    pub reserve: u64,
    pub deposit: u64,
    pub commit_end: u64,
    pub reveal_end: u64,
    pub founders: Vec<Pubkey>,
    /// Up to MAX_NAME bytes, shown as the unit's name; may be empty.
    pub name: Vec<u8>,
    /// Up to MAX_DESC bytes; may be empty.
    pub desc: Vec<u8>,
}

pub fn open_genesis(payer: Pubkey, g: &GenesisSpec) -> Instruction {
    open_genesis_of(ROOT_UNIT, payer, g)
}
/// The second instruction of a founding: the root registry and treasury,
/// the founders in the chairs, the first seat sale. With init_config_of in
/// the same transaction, a polis is founded in one.
pub fn open_genesis_of(root: u64, payer: Pubkey, g: &GenesisSpec) -> Instruction {
    let mut d = data(
        tag::IX_OPEN_GENESIS,
        &[
            &g.charter,
            &g.plan_cap.to_le_bytes(),
            &g.plan_period.to_le_bytes(),
            &g.window.to_le_bytes(),
            &[g.rung],
            &g.reserve.to_le_bytes(),
            &g.deposit.to_le_bytes(),
            &g.commit_end.to_le_bytes(),
            &g.reveal_end.to_le_bytes(),
            &[g.founders.len() as u8],
        ],
    );
    for f in &g.founders {
        d.extend_from_slice(f.as_ref());
    }
    push_text(&mut d, &g.name);
    push_text(&mut d, &g.desc);
    ix(
        d,
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(config_pda_of(root), false),
            AccountMeta::new(treasury_pda(root), false),
            AccountMeta::new(registry_pda(root), false),
            AccountMeta::new(sale_pda(root, 0), false),
            system_meta(),
        ],
    )
}

#[derive(Clone, Debug)]
pub struct FoundSpec {
    pub child: u64,
    pub chair: u8,
    pub charter: [u8; 32],
    pub window: u64,
    pub plan_cap: u64,
    pub plan_period: u64,
    pub founders: Vec<Pubkey>,
    /// Up to MAX_NAME bytes; may be empty.
    pub name: Vec<u8>,
    /// Up to MAX_DESC bytes; may be empty.
    pub desc: Vec<u8>,
}
impl FoundSpec {
    pub fn bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.child.to_le_bytes());
        v.push(self.chair);
        v.extend_from_slice(&self.charter);
        v.extend_from_slice(&self.window.to_le_bytes());
        v.extend_from_slice(&self.plan_cap.to_le_bytes());
        v.extend_from_slice(&self.plan_period.to_le_bytes());
        v.push(self.founders.len() as u8);
        for f in &self.founders {
            v.extend_from_slice(f.as_ref());
        }
        push_text(&mut v, &self.name);
        push_text(&mut v, &self.desc);
        v
    }
}

pub fn found_child(payer: Pubkey, parent: u64, f: &FoundSpec) -> Instruction {
    found_child_of(ROOT_UNIT, payer, parent, f)
}
pub fn found_child_of(root: u64, payer: Pubkey, parent: u64, f: &FoundSpec) -> Instruction {
    ix(
        data(tag::IX_FOUND_CHILD, &[&f.bytes()]),
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(config_pda_of(root), false),
            AccountMeta::new(registry_pda(parent), false),
            AccountMeta::new(registry_pda(f.child), false),
            AccountMeta::new(treasury_pda(f.child), false),
            system_meta(),
        ],
    )
}
pub fn ratify_genesis(crank: Pubkey) -> Instruction {
    ratify_genesis_of(ROOT_UNIT, crank)
}
pub fn ratify_genesis_of(root: u64, crank: Pubkey) -> Instruction {
    ix(
        data(tag::IX_RATIFY_GENESIS, &[]),
        vec![
            AccountMeta::new_readonly(crank, true),
            AccountMeta::new(config_pda_of(root), false),
            AccountMeta::new_readonly(registry_pda(root), false),
            AccountMeta::new_readonly(sale_pda(root, 0), false),
        ],
    )
}

// ---- the double seal ----

/// What a proposal cashes. Bytes are what gets hashed at propose and
/// presented at execute; accounts are what execute needs after the sessions.
#[derive(Clone, Debug)]
pub enum Payload {
    Transfer { origin: u64, dest: u64, amount: u64 },
    Budget { origin: u64, address: Pubkey, amount: u64 },
    Dividend { origin: u64, amount: u64 },
    Dissolve { origin: u64 },
    Attach { origin: u64, chair: u8, holder: [u8; 32], kind: u8, term: u64, retainer: u64 },
    Recall { origin: u64, chair: u8 },
    Jet { origin: u64, holder: [u8; 32], kind: u8 },
    /// `token_cap` bounds token moves per period the way `cap` bounds lamports.
    Plan { origin: u64, cap: u64, period: u64, token_cap: u64 },
    /// The voting window and the CODEX window (at least the voting window).
    Window { origin: u64, window: u64, codex_window: u64 },
    Ladder { origin: u64, reserve: u64, deposit: u64, commit_len: u64, reveal_len: u64, sale_id: u64 },
    /// The ladder in door mode: the next rung's spots open to knocks at
    /// `price` a seat, no sale. The same CODEX kind, with a 33rd byte.
    Door { origin: u64, price: u64 },
    /// The polis admits a knocker to one seat at the price it deposited.
    /// `root` is off-wire: it names the fund's tally for the fee.
    Admit { origin: u64, holder: [u8; 32], root: u64 },
    /// The polis refuses a knocker: the deposit goes home whole.
    Refuse { origin: u64, holder: [u8; 32] },
    Found { origin: u64, spec: FoundSpec },
    /// Rename the unit: up to MAX_NAME and MAX_DESC bytes, either may be empty.
    Name { origin: u64, name: Vec<u8>, desc: Vec<u8> },
    /// Income by rule: `bps` of every lamport of income escrowed for the
    /// floor each `period`; bps 0 turns the policy off.
    DividendPolicy { origin: u64, bps: u16, period: u64 },
    /// The code itself: the root, over the whole tree, has the loader replace
    /// the program from `buffer`, the excess rent going to `spill`.
    Upgrade { origin: u64, buffer: Pubkey, spill: Pubkey },
    /// The code of `program`, a program the seal holds, replaced from `buffer`, the excess rent going to `spill`: `[buffer:32][spill:32][program:32]`.
    /// The program refuses any program, program data, buffer or spill but the named ones (THE SEAL UPGRADES EVERY PROGRAM).
    UpgradeProgram { origin: u64, program: Pubkey, buffer: Pubkey, spill: Pubkey },
    /// Hand `program`'s upgrade authority to `new`, or burn it (None). The act names its program: the payload is `[new:32][program:32]`,
    /// the act's hash covers both, and the program refuses any program data but that program's own (THE SEAL NAMES ITS PROGRAM).
    UpgradeAuthority { origin: u64, program: Pubkey, new: Option<Pubkey> },
    /// Close `buffer`, a buffer of the loader's that the seal holds (an upgrade's that was never carried out), its lamports to `recipient`:
    /// `[buffer:32][recipient:32]`. The program refuses any account but the named one, and any account that is not a buffer (THE SEAL CLOSES A BUFFER).
    CloseBuffer { origin: u64, buffer: Pubkey, recipient: Pubkey },
    /// What a crank is paid per landed batch and per seal, under the plan.
    CrankTip { origin: u64, tip: u64 },
    /// A mint beyond the default the society may spend, `cap` per period in
    /// its own units; cap 0 withdraws it and `payer` (whoever executed it
    /// into being) gets the plan account's rent back.
    AllowMint { origin: u64, mint: Pubkey, cap: u64, payer: Pubkey },
    SaleDefaults { origin: u64, reserve: u64, deposit: u64 },
    Weight { origin: u64, holder: [u8; 32], kind: u8, weight: u64 },
    OpenAuction { origin: u64, lot: LotSpec },
    /// A ruling on a contract by the paying unit or the arbiter it named.
    /// `contract` describes the contract being ruled on (off-wire: it only
    /// builds the account list).
    Rule { origin: u64, hash: [u8; 32], outcome: u8, contract: ContractRef },
    /// The constitution of the fund, by the root polis.
    Fund { origin: u64, fee_bps: u16, cycle_slots: u64, range_lo: u16, range_hi: u16, option_fee: u64, chair_quorum: u8 },
    /// The unit's own law on how a split is settled: the kinds (`SUM_*` bits)
    /// that may ask for the SUM, and the small cap a treasury kind that asks
    /// stays at or under. A later act replaces it; `sum_kinds` 0 turns it off.
    Resolution { origin: u64, sum_kinds: u8, small_cap: u64 },
    /// The root polis vets an option of the fund.
    VetOption { origin: u64, hash: [u8; 32] },
    /// A chair of the fund backs an option.
    BackOption { origin: u64, hash: [u8; 32] },
    /// Tokens from the origin's vault to another unit's vault of the same
    /// mint. `token_program` is not on the wire: it is the account execute
    /// hands to the CPI, and the vault's owner decides which one is right.
    TokenTransfer { origin: u64, mint: Pubkey, dest: u64, amount: u64, token_program: Pubkey },
    /// Tokens from the origin's vault to any token account.
    TokenBudget { origin: u64, mint: Pubkey, address: Pubkey, amount: u64, token_program: Pubkey },
    /// A dividend in a token, out of the unit's vault. In a dissolved unit
    /// amount 0 sweeps the whole vault, outside the plan.
    TokenDividend { origin: u64, mint: Pubkey, amount: u64, token_program: Pubkey },
    /// A standing authorization: `payee` (a key, or a unit) pulls up to
    /// `cap` per `period` slots of SOL (mint zero) or of `mint`, until
    /// `expires` (0: never) or a Revoke. Each pull is charged to the plan.
    Allowance { origin: u64, payee: [u8; 32], payee_kind: u8, mint: Pubkey, cap: u64, period: u64, expires: u64 },
    /// Close an allowance; `payer` is whoever executed it into being.
    Revoke { origin: u64, allowance: [u8; 32], payer: Pubkey },
    /// Escrow `amount` for `payee` until `deadline`, released or refunded by
    /// a Rule of this unit or of `arbiter`, refunded by anyone past the
    /// deadline. Charged to the plan now.
    Contract { origin: u64, payee: [u8; 32], payee_kind: u8, mint: Pubkey, amount: u64, deadline: u64, arbiter: u64, token_program: Pubkey },
    /// The society's hands: any instruction to any other program, signed by
    /// the unit's agent PDA. `accounts` carry ACT_WRITABLE / ACT_SIGNER flags,
    /// at most MAX_ACT_ACCOUNTS of them; ACT_SIGNER on any key but the agent
    /// names a co-signer, who must sign the executing transaction itself; and
    /// no account of this program may appear at all.
    Raw { origin: u64, program: Pubkey, accounts: Vec<(Pubkey, u8)>, data: Vec<u8> },
}

/// What a Rule needs to know about the contract it settles: the account
/// list depends on these, the wire does not.
#[derive(Clone, Debug)]
pub struct ContractRef {
    pub unit: u64,
    pub payee: [u8; 32],
    pub payee_kind: u8,
    pub mint: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
}
impl ContractRef {
    /// `dest` for a release: the payee's own account or token account, or
    /// the payee unit's treasury or vault. `payee_token_account` is the key
    /// payee's token account when the contract is in a token.
    pub fn release_dest(&self, payee_token_account: Option<Pubkey>) -> Pubkey {
        let token = self.mint != Pubkey::default();
        if self.payee_kind == HOLDER_KEY {
            if token {
                payee_token_account.expect("a token contract's key payee needs a token account")
            } else {
                Pubkey::new_from_array(self.payee)
            }
        } else if token {
            vault_pda(holder_unit(&self.payee), &self.mint)
        } else {
            treasury_pda(holder_unit(&self.payee))
        }
    }
    /// `dest` for a refund: the paying unit's treasury, or its vault.
    pub fn refund_dest(&self) -> Pubkey {
        if self.mint != Pubkey::default() { vault_pda(self.unit, &self.mint) } else { treasury_pda(self.unit) }
    }
    /// The settle accounts: [contract (w), payer (w), treasury (w), dest (w), then escrow (w) and token program for tokens].
    pub fn settle_accounts(&self, hash: &[u8; 32], dest: Pubkey) -> Vec<AccountMeta> {
        let mut m = vec![
            AccountMeta::new(contract_pda(hash), false),
            AccountMeta::new(self.payer, false),
            AccountMeta::new(treasury_pda(self.unit), false),
            AccountMeta::new(dest, false),
        ];
        if self.mint != Pubkey::default() {
            m.push(AccountMeta::new(escrow_pda(hash), false));
            m.push(AccountMeta::new_readonly(self.token_program, false));
        }
        m
    }
}

impl Payload {
    pub fn class(&self) -> u8 {
        match self {
            Payload::Transfer { .. }
            | Payload::Budget { .. }
            | Payload::Dividend { .. }
            | Payload::Dissolve { .. }
            | Payload::TokenTransfer { .. }
            | Payload::TokenBudget { .. }
            | Payload::TokenDividend { .. }
            | Payload::Allowance { .. }
            | Payload::Revoke { .. }
            | Payload::Contract { .. } => CLASS_TREASURY,
            Payload::Attach { .. } | Payload::Recall { .. } | Payload::Jet { .. } | Payload::Admit { .. } | Payload::Refuse { .. } => CLASS_SEAT,
            Payload::Plan { .. }
            | Payload::Window { .. }
            | Payload::Ladder { .. }
            | Payload::Door { .. }
            | Payload::Found { .. }
            | Payload::Name { .. }
            | Payload::DividendPolicy { .. }
            | Payload::Upgrade { .. }
            | Payload::UpgradeAuthority { .. }
            | Payload::CrankTip { .. }
            | Payload::AllowMint { .. } => CLASS_CODEX,
            Payload::SaleDefaults { .. } | Payload::Weight { .. } | Payload::OpenAuction { .. } | Payload::Rule { .. } => CLASS_SIGNAL,
            Payload::VetOption { .. } | Payload::BackOption { .. } => CLASS_SIGNAL,
            Payload::Fund { .. } | Payload::Resolution { .. } | Payload::CloseBuffer { .. } | Payload::UpgradeProgram { .. } => CLASS_CODEX,
            Payload::Raw { .. } => CLASS_ACT,
        }
    }
    pub fn origin(&self) -> u64 {
        match self {
            Payload::Transfer { origin, .. }
            | Payload::Budget { origin, .. }
            | Payload::Dividend { origin, .. }
            | Payload::Dissolve { origin }
            | Payload::Attach { origin, .. }
            | Payload::Recall { origin, .. }
            | Payload::Jet { origin, .. }
            | Payload::Plan { origin, .. }
            | Payload::Window { origin, .. }
            | Payload::Ladder { origin, .. }
            | Payload::Door { origin, .. }
            | Payload::Admit { origin, .. }
            | Payload::Refuse { origin, .. }
            | Payload::Found { origin, .. }
            | Payload::Name { origin, .. }
            | Payload::DividendPolicy { origin, .. }
            | Payload::Upgrade { origin, .. }
            | Payload::UpgradeAuthority { origin, .. }
            | Payload::CloseBuffer { origin, .. }
            | Payload::UpgradeProgram { origin, .. }
            | Payload::CrankTip { origin, .. }
            | Payload::AllowMint { origin, .. }
            | Payload::SaleDefaults { origin, .. }
            | Payload::Weight { origin, .. }
            | Payload::OpenAuction { origin, .. }
            | Payload::Rule { origin, .. }
            | Payload::Fund { origin, .. }
            | Payload::Resolution { origin, .. }
            | Payload::VetOption { origin, .. }
            | Payload::BackOption { origin, .. }
            | Payload::TokenTransfer { origin, .. }
            | Payload::TokenBudget { origin, .. }
            | Payload::TokenDividend { origin, .. }
            | Payload::Allowance { origin, .. }
            | Payload::Revoke { origin, .. }
            | Payload::Contract { origin, .. }
            | Payload::Raw { origin, .. } => *origin,
        }
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut v = vec![self.class()];
        match self {
            Payload::Transfer { dest, amount, .. } => {
                v.push(TREASURY_TRANSFER);
                v.extend_from_slice(&dest.to_le_bytes());
                v.extend_from_slice(&amount.to_le_bytes());
            }
            Payload::Budget { address, amount, .. } => {
                v.push(TREASURY_BUDGET);
                v.extend_from_slice(address.as_ref());
                v.extend_from_slice(&amount.to_le_bytes());
            }
            Payload::Dividend { amount, .. } => {
                v.push(TREASURY_DIVIDEND);
                v.extend_from_slice(&amount.to_le_bytes());
            }
            Payload::Dissolve { .. } => v.push(TREASURY_DISSOLVE),
            Payload::Attach { chair, holder, kind, term, retainer, .. } => {
                v.push(SEAT_ATTACH);
                v.push(*chair);
                v.extend_from_slice(holder);
                v.push(*kind);
                v.extend_from_slice(&term.to_le_bytes());
                v.extend_from_slice(&retainer.to_le_bytes());
            }
            Payload::Recall { chair, .. } => {
                v.push(SEAT_RECALL);
                v.push(*chair);
            }
            Payload::Jet { holder, kind, .. } => {
                v.push(SEAT_JET);
                v.extend_from_slice(holder);
                v.push(*kind);
            }
            Payload::Plan { cap, period, token_cap, .. } => {
                v.push(CODEX_PLAN);
                v.extend_from_slice(&cap.to_le_bytes());
                v.extend_from_slice(&period.to_le_bytes());
                v.extend_from_slice(&token_cap.to_le_bytes());
            }
            Payload::Name { name, desc, .. } => {
                v.push(CODEX_NAME);
                push_text(&mut v, name);
                push_text(&mut v, desc);
            }
            Payload::TokenTransfer { mint, dest, amount, .. } => {
                v.push(TREASURY_TOKEN_TRANSFER);
                v.extend_from_slice(mint.as_ref());
                v.extend_from_slice(&dest.to_le_bytes());
                v.extend_from_slice(&amount.to_le_bytes());
            }
            Payload::TokenBudget { mint, address, amount, .. } => {
                v.push(TREASURY_TOKEN_BUDGET);
                v.extend_from_slice(mint.as_ref());
                v.extend_from_slice(address.as_ref());
                v.extend_from_slice(&amount.to_le_bytes());
            }
            Payload::Raw { program, accounts, data, .. } => {
                v.push(ACT_RAW);
                v.extend_from_slice(program.as_ref());
                v.push(accounts.len() as u8);
                for (key, flags) in accounts {
                    v.extend_from_slice(key.as_ref());
                    v.push(*flags);
                }
                v.extend_from_slice(&(data.len() as u16).to_le_bytes());
                v.extend_from_slice(data);
            }
            Payload::Window { window, codex_window, .. } => {
                v.push(CODEX_WINDOW);
                v.extend_from_slice(&window.to_le_bytes());
                v.extend_from_slice(&codex_window.to_le_bytes());
            }
            Payload::DividendPolicy { bps, period, .. } => {
                v.push(CODEX_DIVIDEND_POLICY);
                v.extend_from_slice(&bps.to_le_bytes());
                v.extend_from_slice(&period.to_le_bytes());
            }
            Payload::Upgrade { buffer, spill, .. } => {
                v.push(CODEX_UPGRADE);
                v.extend_from_slice(buffer.as_ref());
                v.extend_from_slice(spill.as_ref());
            }
            Payload::UpgradeAuthority { program, new, .. } => {
                v.push(CODEX_UPGRADE_AUTHORITY);
                v.extend_from_slice(new.unwrap_or_default().as_ref());
                v.extend_from_slice(program.as_ref());
            }
            Payload::UpgradeProgram { program, buffer, spill, .. } => {
                v.push(CODEX_UPGRADE_PROGRAM);
                v.extend_from_slice(buffer.as_ref());
                v.extend_from_slice(spill.as_ref());
                v.extend_from_slice(program.as_ref());
            }
            Payload::CloseBuffer { buffer, recipient, .. } => {
                v.push(CODEX_CLOSE_BUFFER);
                v.extend_from_slice(buffer.as_ref());
                v.extend_from_slice(recipient.as_ref());
            }
            Payload::CrankTip { tip, .. } => {
                v.push(CODEX_CRANK_TIP);
                v.extend_from_slice(&tip.to_le_bytes());
            }
            Payload::AllowMint { mint, cap, .. } => {
                v.push(CODEX_ALLOW_MINT);
                v.extend_from_slice(mint.as_ref());
                v.extend_from_slice(&cap.to_le_bytes());
            }
            Payload::Fund { fee_bps, cycle_slots, range_lo, range_hi, option_fee, chair_quorum, .. } => {
                v.push(CODEX_FUND);
                v.extend_from_slice(&fee_bps.to_le_bytes());
                v.extend_from_slice(&cycle_slots.to_le_bytes());
                v.extend_from_slice(&range_lo.to_le_bytes());
                v.extend_from_slice(&range_hi.to_le_bytes());
                v.extend_from_slice(&option_fee.to_le_bytes());
                v.push(*chair_quorum);
            }
            Payload::Resolution { sum_kinds, small_cap, .. } => {
                v.push(CODEX_RESOLUTION);
                v.push(*sum_kinds);
                v.extend_from_slice(&small_cap.to_le_bytes());
            }
            Payload::VetOption { hash, .. } => {
                v.push(SIGNAL_VET_OPTION);
                v.extend_from_slice(hash);
            }
            Payload::BackOption { hash, .. } => {
                v.push(SIGNAL_BACK_OPTION);
                v.extend_from_slice(hash);
            }
            Payload::Rule { hash, outcome, .. } => {
                v.push(SIGNAL_RULE);
                v.extend_from_slice(hash);
                v.push(*outcome);
            }
            Payload::TokenDividend { mint, amount, .. } => {
                v.push(TREASURY_TOKEN_DIVIDEND);
                v.extend_from_slice(mint.as_ref());
                v.extend_from_slice(&amount.to_le_bytes());
            }
            Payload::Allowance { payee, payee_kind, mint, cap, period, expires, .. } => {
                v.push(TREASURY_ALLOWANCE);
                v.extend_from_slice(payee);
                v.push(*payee_kind);
                v.extend_from_slice(mint.as_ref());
                v.extend_from_slice(&cap.to_le_bytes());
                v.extend_from_slice(&period.to_le_bytes());
                v.extend_from_slice(&expires.to_le_bytes());
            }
            Payload::Revoke { allowance, .. } => {
                v.push(TREASURY_REVOKE);
                v.extend_from_slice(allowance);
            }
            Payload::Contract { payee, payee_kind, mint, amount, deadline, arbiter, .. } => {
                v.push(TREASURY_CONTRACT);
                v.extend_from_slice(payee);
                v.push(*payee_kind);
                v.extend_from_slice(mint.as_ref());
                v.extend_from_slice(&amount.to_le_bytes());
                v.extend_from_slice(&deadline.to_le_bytes());
                v.extend_from_slice(&arbiter.to_le_bytes());
            }
            Payload::Ladder { reserve, deposit, commit_len, reveal_len, .. } => {
                v.push(CODEX_LADDER);
                v.extend_from_slice(&reserve.to_le_bytes());
                v.extend_from_slice(&deposit.to_le_bytes());
                v.extend_from_slice(&commit_len.to_le_bytes());
                v.extend_from_slice(&reveal_len.to_le_bytes());
            }
            Payload::Door { price, .. } => {
                v.push(CODEX_LADDER);
                v.extend_from_slice(&price.to_le_bytes());
                v.extend_from_slice(&[0u8; 24]);
                v.push(LADDER_DOOR);
            }
            Payload::Admit { holder, .. } => {
                v.push(SEAT_ADMIT);
                v.extend_from_slice(holder);
            }
            Payload::Refuse { holder, .. } => {
                v.push(SEAT_REFUSE);
                v.extend_from_slice(holder);
            }
            Payload::Found { spec, .. } => {
                v.push(CODEX_FOUND);
                v.extend_from_slice(&spec.bytes());
            }
            Payload::SaleDefaults { reserve, deposit, .. } => {
                v.push(SIGNAL_SALE_DEFAULTS);
                v.extend_from_slice(&reserve.to_le_bytes());
                v.extend_from_slice(&deposit.to_le_bytes());
            }
            Payload::Weight { holder, kind, weight, .. } => {
                v.push(SIGNAL_WEIGHT);
                v.extend_from_slice(holder);
                v.push(*kind);
                v.extend_from_slice(&weight.to_le_bytes());
            }
            Payload::OpenAuction { lot, .. } => {
                v.push(SIGNAL_OPEN_AUCTION);
                v.extend_from_slice(&lot.bytes());
            }
        }
        v
    }
    pub fn hash(&self) -> [u8; 32] {
        sha(&[&self.bytes()])
    }
    /// The accounts `execute` needs after the sessions (and, for CODEX, the registries).
    pub fn accounts(&self, hash: &[u8; 32]) -> Vec<AccountMeta> {
        let o = self.origin();
        match self {
            Payload::Fund { .. } => vec![AccountMeta::new_readonly(registry_pda(o), false), AccountMeta::new(fund_pda(), false), system_meta()],
            Payload::Resolution { .. } => vec![AccountMeta::new_readonly(registry_pda(o), false), AccountMeta::new(rule_pda(o), false), system_meta()],
            Payload::VetOption { hash: h, .. } => vec![AccountMeta::new_readonly(registry_pda(o), false), AccountMeta::new(option_pda(h), false)],
            Payload::BackOption { hash: h, .. } => vec![AccountMeta::new_readonly(registry_pda(o), false), AccountMeta::new(fund_pda(), false), AccountMeta::new(option_pda(h), false)],
            Payload::Transfer { dest, .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new(treasury_pda(o), false),
                AccountMeta::new(treasury_pda(*dest), false),
            ],
            Payload::Budget { address, .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new(treasury_pda(o), false),
                AccountMeta::new(*address, false),
            ],
            Payload::Dividend { .. } | Payload::Dissolve { .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new(treasury_pda(o), false),
                AccountMeta::new(distribution_pda(hash), false),
                system_meta(),
            ],
            Payload::Attach { holder, kind, retainer, .. } => {
                // a unit's own registry follows the polis's: the seat reads its root and level
                let mut m = vec![AccountMeta::new(registry_pda(o), false)];
                if *kind == HOLDER_UNIT {
                    m.push(AccountMeta::new_readonly(registry_pda(holder_unit(holder)), false));
                    if *retainer > 0 {
                        m.push(AccountMeta::new(treasury_pda(o), false));
                        m.push(AccountMeta::new(treasury_pda(holder_unit(holder)), false));
                    }
                }
                m
            }
            Payload::Recall { .. }
            | Payload::Plan { .. }
            | Payload::Window { .. }
            | Payload::Name { .. }
            | Payload::SaleDefaults { .. }
            | Payload::Weight { .. } => vec![AccountMeta::new(registry_pda(o), false)],
            Payload::DividendPolicy { .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new_readonly(treasury_pda(o), false),
            ],
            Payload::CrankTip { .. } => vec![AccountMeta::new(registry_pda(o), false)],
            Payload::AllowMint { mint, payer, .. } => vec![
                AccountMeta::new_readonly(registry_pda(o), false),
                AccountMeta::new(token_plan_pda(o, mint), false),
                AccountMeta::new(*payer, false),
                system_meta(),
            ],
            Payload::Upgrade { buffer, spill, .. } => vec![
                AccountMeta::new_readonly(registry_pda(o), false),
                AccountMeta::new(programdata_pda(), false),
                AccountMeta::new(PROGRAM_ID, false),
                AccountMeta::new(*buffer, false),
                AccountMeta::new(*spill, false),
                AccountMeta::new_readonly(RENT_SYSVAR_ID, false),
                AccountMeta::new_readonly(CLOCK_SYSVAR_ID, false),
                AccountMeta::new_readonly(upgrade_pda(), false),
                AccountMeta::new_readonly(BPF_LOADER_UPGRADEABLE_ID, false),
            ],
            Payload::UpgradeAuthority { program, new, .. } => {
                let mut m = vec![
                    AccountMeta::new_readonly(registry_pda(o), false),
                    AccountMeta::new(programdata_of(program), false),
                    AccountMeta::new_readonly(upgrade_pda(), false),
                    AccountMeta::new_readonly(BPF_LOADER_UPGRADEABLE_ID, false),
                ];
                if let Some(n) = new {
                    m.push(AccountMeta::new_readonly(*n, false));
                }
                m
            }
            Payload::UpgradeProgram { program, buffer, spill, .. } => vec![
                AccountMeta::new_readonly(registry_pda(o), false),
                AccountMeta::new(programdata_of(program), false),
                AccountMeta::new(*program, false),
                AccountMeta::new(*buffer, false),
                AccountMeta::new(*spill, false),
                AccountMeta::new_readonly(RENT_SYSVAR_ID, false),
                AccountMeta::new_readonly(CLOCK_SYSVAR_ID, false),
                AccountMeta::new_readonly(upgrade_pda(), false),
                AccountMeta::new_readonly(BPF_LOADER_UPGRADEABLE_ID, false),
            ],
            Payload::CloseBuffer { buffer, recipient, .. } => vec![
                AccountMeta::new_readonly(registry_pda(o), false),
                AccountMeta::new(*buffer, false),
                AccountMeta::new(*recipient, false),
                AccountMeta::new_readonly(upgrade_pda(), false),
                AccountMeta::new_readonly(BPF_LOADER_UPGRADEABLE_ID, false),
            ],
            Payload::Rule { hash: chash, outcome, contract, .. } => {
                let dest = if *outcome == RULE_RELEASE { contract.release_dest(None) } else { contract.refund_dest() };
                let mut m = vec![AccountMeta::new_readonly(registry_pda(o), false)];
                m.extend(contract.settle_accounts(chash, dest));
                m
            }
            Payload::TokenDividend { mint, token_program, .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new(treasury_pda(o), false),
                AccountMeta::new(vault_pda(o, mint), false),
                AccountMeta::new(distribution_pda(hash), false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new(escrow_pda(hash), false),
                AccountMeta::new_readonly(*token_program, false),
                system_meta(),
                AccountMeta::new(token_plan_pda(o, mint), false),
            ],
            Payload::Allowance { .. } => vec![
                AccountMeta::new_readonly(registry_pda(o), false),
                AccountMeta::new(allowance_pda(hash), false),
                system_meta(),
            ],
            Payload::Revoke { allowance, payer, .. } => vec![
                AccountMeta::new_readonly(registry_pda(o), false),
                AccountMeta::new(allowance_pda(allowance), false),
                AccountMeta::new(*payer, false),
            ],
            Payload::Contract { mint, token_program, .. } => {
                let mut m = vec![
                    AccountMeta::new(registry_pda(o), false),
                    AccountMeta::new(treasury_pda(o), false),
                    AccountMeta::new(contract_pda(hash), false),
                    system_meta(),
                ];
                if *mint != Pubkey::default() {
                    m.push(AccountMeta::new_readonly(*mint, false));
                    m.push(AccountMeta::new(vault_pda(o, mint), false));
                    m.push(AccountMeta::new(escrow_pda(hash), false));
                    m.push(AccountMeta::new_readonly(*token_program, false));
                    m.push(AccountMeta::new(token_plan_pda(o, mint), false));
                }
                m
            }
            Payload::TokenTransfer { mint, dest, token_program, .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new_readonly(treasury_pda(o), false),
                AccountMeta::new(vault_pda(o, mint), false),
                AccountMeta::new(vault_pda(*dest, mint), false),
                AccountMeta::new_readonly(*token_program, false),
                AccountMeta::new(token_plan_pda(o, mint), false),
            ],
            Payload::TokenBudget { mint, address, token_program, .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new_readonly(treasury_pda(o), false),
                AccountMeta::new(vault_pda(o, mint), false),
                AccountMeta::new(*address, false),
                AccountMeta::new_readonly(*token_program, false),
                AccountMeta::new(token_plan_pda(o, mint), false),
            ],
            Payload::Raw { program, accounts, .. } => {
                let mut m = vec![AccountMeta::new(registry_pda(o), false), AccountMeta::new_readonly(*program, false)];
                // the outer transaction never signs for the agent: the program does, by CPI. A named co-signer signs it itself.
                let agent = agent_pda(o);
                for (key, flags) in accounts {
                    m.push(AccountMeta { pubkey: *key, is_signer: flags & ACT_SIGNER != 0 && *key != agent, is_writable: flags & ACT_WRITABLE != 0 });
                }
                m
            }
            Payload::Jet { holder, kind, .. } => {
                let payee = if *kind == HOLDER_KEY { Pubkey::new_from_array(*holder) } else { treasury_pda(holder_unit(holder)) };
                vec![
                    AccountMeta::new(registry_pda(o), false),
                    AccountMeta::new(treasury_pda(o), false),
                    AccountMeta::new(payee, false),
                ]
            }
            Payload::Ladder { sale_id, .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new(sale_pda(o, *sale_id), false),
                system_meta(),
            ],
            Payload::Door { .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new(pool_pda(o), false),
                system_meta(),
            ],
            Payload::Admit { holder, root, .. } => {
                let key = Pubkey::new_from_array(*holder);
                vec![
                    AccountMeta::new(registry_pda(o), false),
                    AccountMeta::new(treasury_pda(o), false),
                    AccountMeta::new(pool_pda(o), false),
                    AccountMeta::new(knock_pda(o, &key), false),
                    AccountMeta::new(key, false),
                    AccountMeta::new(fund_pda(), false),
                    AccountMeta::new(tally_pda(*root), false),
                    system_meta(),
                ]
            }
            Payload::Refuse { holder, .. } => {
                let key = Pubkey::new_from_array(*holder);
                vec![
                    AccountMeta::new_readonly(registry_pda(o), false),
                    AccountMeta::new(knock_pda(o, &key), false),
                    AccountMeta::new(key, false),
                ]
            }
            Payload::Found { spec, .. } => vec![
                AccountMeta::new(registry_pda(o), false),
                AccountMeta::new(registry_pda(spec.child), false),
                AccountMeta::new(treasury_pda(spec.child), false),
                system_meta(),
            ],
            Payload::OpenAuction { lot, .. } => vec![
                AccountMeta::new_readonly(treasury_pda(o), false),
                AccountMeta::new(auction_pda(lot.auction_id), false),
                system_meta(),
            ],
        }
    }
}

/// accounts: [proposer (s,w), config, proposal (w), system, registries..., sessions...]
/// The payload rides in clear and is stored in the proposal.
pub fn propose(proposer: Pubkey, hash: [u8; 32], manifest: [u8; 32], payload: &Payload, impacted: &[u64]) -> Instruction {
    propose_raw(proposer, hash, payload.class(), payload.origin(), manifest, &payload.bytes(), impacted)
}
/// The same, asking for consent: every chamber must seal PASSED before the act executes.
pub fn propose_consent(proposer: Pubkey, hash: [u8; 32], manifest: [u8; 32], payload: &Payload, impacted: &[u64]) -> Instruction {
    propose_raw(proposer, hash, payload.class() | CLASS_CONSENT, payload.origin(), manifest, &payload.bytes(), impacted)
}
/// The same, asking that a split of the origin's chambers be settled by the
/// SUM: the class byte carries `CLASS_SUM`, and the origin's rule account
/// rides last, read-only, for the program to check the kind and the cap.
pub fn propose_sum(proposer: Pubkey, hash: [u8; 32], manifest: [u8; 32], payload: &Payload, impacted: &[u64]) -> Instruction {
    let mut ix = propose_raw(proposer, hash, payload.class() | CLASS_SUM, payload.origin(), manifest, &payload.bytes(), impacted);
    ix.accounts.push(AccountMeta::new_readonly(rule_pda(payload.origin()), false));
    ix
}
/// A proposal in some other tree: its council's config is the one checked.
pub fn propose_of(root: u64, proposer: Pubkey, hash: [u8; 32], manifest: [u8; 32], payload: &Payload, impacted: &[u64]) -> Instruction {
    propose_raw_of(root, proposer, hash, payload.class(), payload.origin(), manifest, &payload.bytes(), impacted)
}

/// The same wire with every field chosen by the caller, for hostile shapes.
pub fn propose_raw(proposer: Pubkey, hash: [u8; 32], class: u8, origin: u64, manifest: [u8; 32], payload: &[u8], impacted: &[u64]) -> Instruction {
    propose_raw_of(ROOT_UNIT, proposer, hash, class, origin, manifest, payload, impacted)
}
#[allow(clippy::too_many_arguments)]
pub fn propose_raw_of(root: u64, proposer: Pubkey, hash: [u8; 32], class: u8, origin: u64, manifest: [u8; 32], payload: &[u8], impacted: &[u64]) -> Instruction {
    let mut d = data(tag::IX_PROPOSE, &[&hash, &[class], &origin.to_le_bytes(), &manifest, &[impacted.len() as u8]]);
    for u in impacted {
        d.extend_from_slice(&u.to_le_bytes());
    }
    d.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    d.extend_from_slice(payload);
    let mut metas = vec![
        AccountMeta::new(proposer, true),
        AccountMeta::new_readonly(config_pda_of(root), false),
        AccountMeta::new(proposal_pda(&hash), false),
        system_meta(),
    ];
    for u in impacted {
        metas.push(AccountMeta::new_readonly(registry_pda(*u), false));
    }
    for u in impacted {
        metas.push(AccountMeta::new(session_pda(&hash, *u), false));
    }
    ix(d, metas)
}

/// accounts: [crank (s,w), registry (w), session (w), instructions sysvar, treasury (w)]
pub fn attest(crank: Pubkey, hash: [u8; 32], unit: u64, votes: &[(Pubkey, u8)]) -> Instruction {
    let mut d = data(tag::IX_ATTEST, &[&hash, &unit.to_le_bytes(), &[votes.len() as u8]]);
    for (v, c) in votes {
        d.extend_from_slice(v.as_ref());
        d.push(*c);
    }
    ix(
        d,
        vec![
            AccountMeta::new(crank, true),
            AccountMeta::new(registry_pda(unit), false),
            AccountMeta::new(session_pda(&hash, unit), false),
            AccountMeta::new_readonly(INSTRUCTIONS_SYSVAR_ID, false),
            AccountMeta::new(treasury_pda(unit), false),
        ],
    )
}

/// The two instructions a batch of signed votes lands with: the precompile
/// first, then `attest`, in one transaction.
pub fn attest_batch(crank: Pubkey, hash: [u8; 32], unit: u64, signed: &[(Pubkey, u8, [u8; 64])]) -> [Instruction; 2] {
    let entries: Vec<SignedEnvelope> = signed
        .iter()
        .map(|(v, c, s)| SignedEnvelope { pubkey: *v.as_array(), signature: *s, message: vote_envelope(&hash, unit, *c) })
        .collect();
    let votes: Vec<(Pubkey, u8)> = signed.iter().map(|(v, c, _)| (*v, *c)).collect();
    [ed25519_instruction(&entries), attest(crank, hash, unit, &votes)]
}

/// accounts: [crank (s), parent registry, parent session (w), child session]
pub fn mandate(crank: Pubkey, hash: [u8; 32], parent: u64, child: u64) -> Instruction {
    ix(
        data(tag::IX_MANDATE, &[&hash, &parent.to_le_bytes(), &child.to_le_bytes()]),
        vec![
            AccountMeta::new_readonly(crank, true),
            AccountMeta::new_readonly(registry_pda(parent), false),
            AccountMeta::new(session_pda(&hash, parent), false),
            AccountMeta::new_readonly(session_pda(&hash, child), false),
        ],
    )
}

/// accounts: [crank (s,w), proposal, parent registry, parent session, registry, session (w), system]
/// Each polis opens its own vote: `unit`, seated in a chair of `parent`,
/// opens its session of the decision while `parent`'s is open.
pub fn convene(crank: Pubkey, hash: [u8; 32], parent: u64, unit: u64) -> Instruction {
    ix(
        data(tag::IX_CONVENE, &[&hash, &parent.to_le_bytes(), &unit.to_le_bytes()]),
        vec![
            AccountMeta::new(crank, true),
            AccountMeta::new_readonly(proposal_pda(&hash), false),
            AccountMeta::new_readonly(registry_pda(parent), false),
            AccountMeta::new_readonly(session_pda(&hash, parent), false),
            AccountMeta::new_readonly(registry_pda(unit), false),
            AccountMeta::new(session_pda(&hash, unit), false),
            system_meta(),
        ],
    )
}

/// accounts: [crank (s,w), registry (w), session (w), proposal, treasury (w)]
pub fn seal(crank: Pubkey, hash: [u8; 32], unit: u64) -> Instruction {
    ix(
        data(tag::IX_SEAL, &[&hash, &unit.to_le_bytes()]),
        vec![
            AccountMeta::new(crank, true),
            AccountMeta::new(registry_pda(unit), false),
            AccountMeta::new(session_pda(&hash, unit), false),
            AccountMeta::new_readonly(proposal_pda(&hash), false),
            AccountMeta::new(treasury_pda(unit), false),
        ],
    )
}

/// accounts: [crank (s,w), config, proposal (w), sessions..., (CODEX registries...), payload accounts]
pub fn execute(crank: Pubkey, hash: [u8; 32], impacted: &[u64], payload: &Payload) -> Instruction {
    execute_of(ROOT_UNIT, crank, hash, impacted, payload)
}
pub fn execute_of(root: u64, crank: Pubkey, hash: [u8; 32], impacted: &[u64], payload: &Payload) -> Instruction {
    let mut metas = vec![
        AccountMeta::new(crank, true),
        AccountMeta::new_readonly(config_pda_of(root), false),
        AccountMeta::new(proposal_pda(&hash), false),
    ];
    for u in impacted {
        metas.push(AccountMeta::new_readonly(session_pda(&hash, *u), false));
    }
    if payload.class() == CLASS_CODEX {
        for u in impacted {
            metas.push(AccountMeta::new_readonly(registry_pda(*u), false));
        }
    }
    metas.extend(payload.accounts(&hash));
    // the payload on the wire is only for the account list: the chain
    // executes the bytes the proposal stores
    ix(data(tag::IX_EXECUTE, &[&hash]), metas)
}

/// accounts: [claimer (s,w), registry (w), distribution (w), payee (w)]
pub fn claim_dividend(claimer: Pubkey, hash: [u8; 32], unit: u64, holder: [u8; 32], kind: u8) -> Instruction {
    let payee = if kind == HOLDER_KEY { Pubkey::new_from_array(holder) } else { treasury_pda(holder_unit(&holder)) };
    ix(
        data(tag::IX_CLAIM_DIVIDEND, &[&hash, &holder, &[kind]]),
        vec![
            AccountMeta::new(claimer, true),
            AccountMeta::new(registry_pda(unit), false),
            AccountMeta::new(distribution_pda(&hash), false),
            AccountMeta::new(payee, false),
        ],
    )
}

/// A claim on a token distribution. `payee_token_account` is the key
/// payee's token account; a unit payee is paid into its vault of the mint.
/// accounts: [claimer (s), registry (w), distribution (w), payee token account (w), escrow (w), treasury, token program]
pub fn claim_token_dividend(
    claimer: Pubkey,
    hash: [u8; 32],
    unit: u64,
    holder: [u8; 32],
    kind: u8,
    mint: Pubkey,
    payee_token_account: Option<Pubkey>,
    token_program: Pubkey,
) -> Instruction {
    let payee = if kind == HOLDER_KEY {
        payee_token_account.expect("a key payee needs a token account")
    } else {
        vault_pda(holder_unit(&holder), &mint)
    };
    ix(
        data(tag::IX_CLAIM_DIVIDEND, &[&hash, &holder, &[kind]]),
        vec![
            AccountMeta::new_readonly(claimer, true),
            AccountMeta::new(registry_pda(unit), false),
            AccountMeta::new(distribution_pda(&hash), false),
            AccountMeta::new(payee, false),
            AccountMeta::new(escrow_pda(&hash), false),
            AccountMeta::new_readonly(treasury_pda(unit), false),
            AccountMeta::new_readonly(token_program, false),
        ],
    )
}

// ---- the economy ----

/// A pull on an allowance. `dest`: the key payee's account, or its token
/// account; a unit payee's treasury or vault (anyone may pull for a unit).
/// accounts, lamports: [puller (s), registry (w), treasury (w), allowance (w), dest (w)]
/// accounts, tokens:   [puller (s), registry (w), treasury, allowance (w), dest (w), vault (w), token program]
pub fn pull(puller: Pubkey, hash: [u8; 32], unit: u64, amount: u64, dest: Pubkey, mint: Option<(Pubkey, Pubkey)>) -> Instruction {
    let mut m = vec![
        AccountMeta::new_readonly(puller, true),
        AccountMeta::new(registry_pda(unit), false),
        AccountMeta::new(treasury_pda(unit), false),
        AccountMeta::new(allowance_pda(&hash), false),
        AccountMeta::new(dest, false),
    ];
    if let Some((mint, token_program)) = mint {
        m.push(AccountMeta::new(vault_pda(unit, &mint), false));
        m.push(AccountMeta::new_readonly(token_program, false));
        m.push(AccountMeta::new(token_plan_pda(unit, &mint), false));
    }
    ix(data(tag::IX_PULL, &[&hash, &amount.to_le_bytes()]), m)
}

/// accounts: [allowance (w), payer (w)]: an expired allowance's rent goes home.
pub fn close_allowance(hash: [u8; 32], payer: Pubkey) -> Instruction {
    ix(
        data(tag::IX_CLOSE_ALLOWANCE, &[&hash]),
        vec![AccountMeta::new(allowance_pda(&hash), false), AccountMeta::new(payer, false)],
    )
}

/// Refund a contract past its deadline (anyone) or resign it (the payee key).
/// accounts: [signer (s), then the settle accounts with the refund destination]
pub fn expire_contract(signer: Pubkey, hash: [u8; 32], contract: &ContractRef) -> Instruction {
    let mut m = vec![AccountMeta::new_readonly(signer, true)];
    m.extend(contract.settle_accounts(&hash, contract.refund_dest()));
    ix(data(tag::IX_EXPIRE_CONTRACT, &[&hash]), m)
}

/// accounts: [crank (s,w), registry (w), treasury (w), distribution (w), system]
/// The distribution is keyed by `accrual_hash(unit, seq)` for the registry's current `dividend_seq`.
pub fn accrue_dividend(crank: Pubkey, unit: u64, seq: u64) -> Instruction {
    ix(
        data(tag::IX_ACCRUE_DIVIDEND, &[&unit.to_le_bytes()]),
        vec![
            AccountMeta::new(crank, true),
            AccountMeta::new(registry_pda(unit), false),
            AccountMeta::new(treasury_pda(unit), false),
            AccountMeta::new(distribution_pda(&accrual_hash(unit, seq)), false),
            system_meta(),
        ],
    )
}

/// accounts, lamports: [distribution (w), treasury (w), payer (w)]
/// accounts, tokens:   [distribution (w), treasury, payer (w), vault (w), escrow (w), token program]
pub fn close_distribution(hash: [u8; 32], unit: u64, payer: Pubkey, mint: Option<(Pubkey, Pubkey)>) -> Instruction {
    let mut m = vec![
        AccountMeta::new(distribution_pda(&hash), false),
        AccountMeta::new(treasury_pda(unit), false),
        AccountMeta::new(payer, false),
    ];
    if let Some((mint, token_program)) = mint {
        m.push(AccountMeta::new(vault_pda(unit, &mint), false));
        m.push(AccountMeta::new(escrow_pda(&hash), false));
        m.push(AccountMeta::new_readonly(token_program, false));
    }
    ix(data(tag::IX_CLOSE_DISTRIBUTION, &[&hash]), m)
}

// ---- the seat market ----

/// accounts: [seller (s,w), registry, listing (w), system]
pub fn list_seats(seller: Pubkey, unit: u64, nonce: u64, count: u64, price: u64) -> Instruction {
    ix(
        data(tag::IX_LIST_SEATS, &[&unit.to_le_bytes(), &nonce.to_le_bytes(), &count.to_le_bytes(), &price.to_le_bytes()]),
        vec![
            AccountMeta::new(seller, true),
            AccountMeta::new_readonly(registry_pda(unit), false),
            AccountMeta::new(listing_pda(unit, &seller, nonce), false),
            system_meta(),
        ],
    )
}
/// accounts: [buyer (s,w), seller (w), registry (w), listing (w), system]
/// `price` is the price per seat the buyer agreed to; the program refuses a listing that offers another.
pub fn buy_seats(buyer: Pubkey, seller: Pubkey, unit: u64, nonce: u64, count: u64, price: u64, root: u64) -> Instruction {
    ix(
        data(tag::IX_BUY_SEATS, &[&unit.to_le_bytes(), &nonce.to_le_bytes(), &count.to_le_bytes(), &price.to_le_bytes()]),
        vec![
            AccountMeta::new(buyer, true),
            AccountMeta::new(seller, false),
            AccountMeta::new(registry_pda(unit), false),
            AccountMeta::new(listing_pda(unit, &seller, nonce), false),
            system_meta(),
            AccountMeta::new(fund_pda(), false),
            AccountMeta::new(tally_pda(root), false),
        ],
    )
}
/// accounts: [seller (s,w), listing (w)]
pub fn cancel_listing(seller: Pubkey, unit: u64, nonce: u64) -> Instruction {
    ix(
        data(tag::IX_CANCEL_LISTING, &[&unit.to_le_bytes(), &nonce.to_le_bytes()]),
        vec![AccountMeta::new(seller, true), AccountMeta::new(listing_pda(unit, &seller, nonce), false)],
    )
}

// ---- the fund ----

/// accounts: [crank (s), fund (w), tally]
pub fn seat_chair(crank: Pubkey, root: u64) -> Instruction {
    ix(
        data(tag::IX_SEAT_CHAIR, &[&root.to_le_bytes()]),
        vec![AccountMeta::new_readonly(crank, true), AccountMeta::new(fund_pda(), false), AccountMeta::new_readonly(tally_pda(root), false)],
    )
}
/// accounts: [proposer (s,w), fund (w), option (w), system]
pub fn propose_option(proposer: Pubkey, recipient: Pubkey, manifest: [u8; 32]) -> Instruction {
    let hash = option_hash(&recipient, &manifest);
    ix(
        data(tag::IX_PROPOSE_OPTION, &[recipient.as_ref(), &manifest]),
        vec![AccountMeta::new(proposer, true), AccountMeta::new(fund_pda(), false), AccountMeta::new(option_pda(&hash), false), system_meta()],
    )
}
/// accounts: [voter (s,w), fund (w), option (w), registry, stake (w), system]
pub fn stake(voter: Pubkey, hash: [u8; 32], unit: u64, value_bps: u16) -> Instruction {
    ix(
        data(tag::IX_STAKE, &[&hash, &unit.to_le_bytes(), &value_bps.to_le_bytes()]),
        vec![
            AccountMeta::new(voter, true),
            AccountMeta::new(fund_pda(), false),
            AccountMeta::new(option_pda(&hash), false),
            AccountMeta::new_readonly(registry_pda(unit), false),
            AccountMeta::new(stake_pda(&hash, unit, &voter), false),
            system_meta(),
        ],
    )
}
/// accounts: [voter (s,w), fund (w), option (w), stake (w)]
pub fn unstake(voter: Pubkey, hash: [u8; 32], unit: u64) -> Instruction {
    ix(
        data(tag::IX_UNSTAKE, &[&hash, &unit.to_le_bytes()]),
        vec![
            AccountMeta::new(voter, true),
            AccountMeta::new(fund_pda(), false),
            AccountMeta::new(option_pda(&hash), false),
            AccountMeta::new(stake_pda(&hash, unit, &voter), false),
        ],
    )
}
// ---- the door ----
/// accounts: [knocker (s,w), registry, pool, knock (w), system]
pub fn knock(knocker: Pubkey, unit: u64) -> Instruction {
    ix(
        data(tag::IX_KNOCK, &[&unit.to_le_bytes()]),
        vec![
            AccountMeta::new(knocker, true),
            AccountMeta::new_readonly(registry_pda(unit), false),
            AccountMeta::new_readonly(pool_pda(unit), false),
            AccountMeta::new(knock_pda(unit, &knocker), false),
            system_meta(),
        ],
    )
}
/// accounts: [knocker (s,w), registry, knock (w)]
pub fn withdraw_knock(knocker: Pubkey, unit: u64) -> Instruction {
    ix(
        data(tag::IX_WITHDRAW_KNOCK, &[&unit.to_le_bytes()]),
        vec![
            AccountMeta::new(knocker, true),
            AccountMeta::new_readonly(registry_pda(unit), false),
            AccountMeta::new(knock_pda(unit, &knocker), false),
        ],
    )
}
/// accounts: [crank (s), fund (w)]: rolls a cycle that has run its slots; the grant is booked, not paid
pub fn settle_cycle(crank: Pubkey) -> Instruction {
    ix(data(tag::IX_SETTLE_CYCLE, &[]), vec![AccountMeta::new_readonly(crank, true), AccountMeta::new(fund_pda(), false)])
}
/// accounts: [crank (s), fund (w), recipient (w)]: pays the recipient what the fund owes it
pub fn claim_grant(crank: Pubkey, recipient: Pubkey) -> Instruction {
    ix(
        data(tag::IX_CLAIM_GRANT, &[]),
        vec![AccountMeta::new_readonly(crank, true), AccountMeta::new(fund_pda(), false), AccountMeta::new(recipient, false)],
    )
}

/// accounts: [payer (s,w), fund (w), system, option (when the older fund has a current option)]: the older fund (706 bytes, before Sept 9
/// 2026) grown in place to this layout, the payer paying the added bytes' rent; `current` is the older fund's own current option, if any
pub fn grow_fund(payer: Pubkey, current: Option<[u8; 32]>) -> Instruction {
    let mut metas = vec![AccountMeta::new(payer, true), AccountMeta::new(fund_pda(), false), system_meta()];
    if let Some(h) = current {
        metas.push(AccountMeta::new_readonly(option_pda(&h), false));
    }
    ix(data(tag::IX_GROW_FUND, &[]), metas)
}

// ---- rent recovery ----

/// accounts: [proposal (w), payer (w), sessions (w)...]
/// Anyone closes a decided proposal: rent goes back to whoever proposed.
pub fn close_proposal(hash: [u8; 32], payer: Pubkey, impacted: &[u64]) -> Instruction {
    let mut metas = vec![AccountMeta::new(proposal_pda(&hash), false), AccountMeta::new(payer, false)];
    for u in impacted {
        metas.push(AccountMeta::new(session_pda(&hash, *u), false));
    }
    ix(data(tag::IX_CLOSE_PROPOSAL, &[&hash]), metas)
}

/// accounts: [session (w), convener (w), proposal]
/// Anyone adjourns a convened session once the decision is over: its rent
/// goes back to whoever convened it.
pub fn adjourn(hash: [u8; 32], unit: u64, convener: Pubkey) -> Instruction {
    ix(
        data(tag::IX_ADJOURN, &[&hash, &unit.to_le_bytes()]),
        vec![
            AccountMeta::new(session_pda(&hash, unit), false),
            AccountMeta::new(convener, false),
            AccountMeta::new_readonly(proposal_pda(&hash), false),
        ],
    )
}

/// accounts: [seat bid (w), bidder (w)]
pub fn close_seat_bid(bidder: Pubkey, unit: u64, sale: u64, nonce: u64) -> Instruction {
    ix(
        data(tag::IX_CLOSE_SEAT_BID, &[&unit.to_le_bytes(), &sale.to_le_bytes(), &nonce.to_le_bytes()]),
        vec![AccountMeta::new(seat_bid_pda(unit, sale, &bidder, nonce), false), AccountMeta::new(bidder, false)],
    )
}

/// accounts: [auction, bid (w), bidder (w)]
pub fn close_bid(bidder: Pubkey, auction: u64) -> Instruction {
    ix(
        data(tag::IX_CLOSE_BID, &[&auction.to_le_bytes()]),
        vec![
            AccountMeta::new_readonly(auction_pda(auction), false),
            AccountMeta::new(bid_pda(auction, &bidder), false),
            AccountMeta::new(bidder, false),
        ],
    )
}

// ---- account views ----

fn header_ok(d: &[u8], disc: u8, size: usize) -> bool {
    d.len() == size && d[OFF_DISC] == disc && d[OFF_VERSION] == LAYOUT_VERSION
}
fn u16_at(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(d[o..o + 2].try_into().unwrap())
}
fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
fn key_at(d: &[u8], o: usize) -> Pubkey {
    Pubkey::new_from_array(d[o..o + 32].try_into().unwrap())
}
fn h32_at(d: &[u8], o: usize) -> [u8; 32] {
    d[o..o + 32].try_into().unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigView {
    pub authority: Pubkey,
    pub mode: u8,
    pub default_mint: Pubkey,
    pub root: u64,
}
impl ConfigView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_CONFIG, Config::SIZE).then(|| Self {
            authority: key_at(d, Config::OFF_AUTHORITY),
            mode: d[Config::OFF_MODE],
            default_mint: key_at(d, Config::OFF_DEFAULT_MINT),
            root: u64_at(d, Config::OFF_ROOT),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenPlanView {
    pub unit: u64,
    pub mint: Pubkey,
    pub cap: u64,
    pub spent: u64,
    pub start: u64,
    pub payer: Pubkey,
}
impl TokenPlanView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_TOKEN_PLAN, TokenPlan::SIZE).then(|| Self {
            unit: u64_at(d, TokenPlan::OFF_UNIT),
            mint: key_at(d, TokenPlan::OFF_MINT),
            cap: u64_at(d, TokenPlan::OFF_CAP),
            spent: u64_at(d, TokenPlan::OFF_SPENT),
            start: u64_at(d, TokenPlan::OFF_START),
            payer: key_at(d, TokenPlan::OFF_PAYER),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreasuryView {
    pub unit: u64,
    pub total_in: u64,
    pub total_out: u64,
}
impl TreasuryView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_TREASURY, Treasury::SIZE).then(|| Self {
            unit: u64_at(d, Treasury::OFF_UNIT),
            total_in: u64_at(d, Treasury::OFF_TOTAL_IN),
            total_out: u64_at(d, Treasury::OFF_TOTAL_OUT),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuctionView {
    pub auction_id: u64,
    pub unit: u64,
    pub commit_end: u64,
    pub reveal_end: u64,
    pub reserve: u64,
    pub best_bid: u64,
    pub second_bid: u64,
    pub clearing: u64,
    pub best_bidder: Pubkey,
    pub settled: bool,
    pub deposit: u64,
}
impl AuctionView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_AUCTION, Auction::SIZE).then(|| Self {
            auction_id: u64_at(d, Auction::OFF_AUCTION_ID),
            unit: u64_at(d, Auction::OFF_UNIT),
            commit_end: u64_at(d, Auction::OFF_COMMIT_END),
            reveal_end: u64_at(d, Auction::OFF_REVEAL_END),
            reserve: u64_at(d, Auction::OFF_RESERVE),
            best_bid: u64_at(d, Auction::OFF_BEST_BID),
            second_bid: u64_at(d, Auction::OFF_SECOND_BID),
            clearing: u64_at(d, Auction::OFF_CLEARING),
            best_bidder: key_at(d, Auction::OFF_BEST_BIDDER),
            settled: d[Auction::OFF_SETTLED] != 0,
            deposit: u64_at(d, Auction::OFF_DEPOSIT),
        })
    }
    pub fn winner(&self) -> Option<Pubkey> {
        (self.best_bid > 0 && self.best_bid >= self.reserve).then_some(self.best_bidder)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BidView {
    pub auction_id: u64,
    pub bidder: Pubkey,
    pub deposit: u64,
    pub amount: u64,
    pub revealed: bool,
    pub reclaimed: bool,
}
impl BidView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_BID, Bid::SIZE).then(|| Self {
            auction_id: u64_at(d, Bid::OFF_AUCTION_ID),
            bidder: key_at(d, Bid::OFF_BIDDER),
            deposit: u64_at(d, Bid::OFF_DEPOSIT),
            amount: u64_at(d, Bid::OFF_AMOUNT),
            revealed: d[Bid::OFF_REVEALED] != 0,
            reclaimed: d[Bid::OFF_RECLAIMED] != 0,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeatBidView {
    pub unit: u64,
    pub sale_id: u64,
    pub bidder: Pubkey,
    pub nonce: u64,
    pub deposit: u64,
    pub price: u64,
    pub revealed: bool,
    pub done: bool,
}
impl SeatBidView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_SEAT_BID, SeatBid::SIZE).then(|| Self {
            unit: u64_at(d, SeatBid::OFF_UNIT),
            sale_id: u64_at(d, SeatBid::OFF_SALE),
            bidder: key_at(d, SeatBid::OFF_BIDDER),
            nonce: u64_at(d, SeatBid::OFF_NONCE),
            deposit: u64_at(d, SeatBid::OFF_DEPOSIT),
            price: u64_at(d, SeatBid::OFF_PRICE),
            revealed: d[SeatBid::OFF_REVEALED] != 0,
            done: d[SeatBid::OFF_DONE] != 0,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PentagonView {
    pub holder: [u8; 32],
    pub kind: u8,
    pub seated_slot: u64,
    pub term_end: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HolderView {
    pub holder: [u8; 32],
    pub kind: u8,
    pub seats: u64,
    pub weight: u64,
    pub last_active: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryView {
    pub unit: u64,
    pub parent: u64,
    pub chair_in_parent: u8,
    pub level: u8,
    pub plan_cap: u64,
    pub plan_period: u64,
    pub plan_spent: u64,
    pub plan_start: u64,
    pub rung: u8,
    pub frozen: bool,
    pub total_seats: u64,
    pub charter: [u8; 32],
    pub chain_sig: [u8; 32],
    pub window: u64,
    pub next_sale_id: u64,
    pub seat_reserve: u64,
    pub seat_deposit: u64,
    pub last_clearing: u64,
    pub token_plan_cap: u64,
    pub token_plan_spent: u64,
    pub codex_window: u64,
    pub dividend_bps: u16,
    pub dividend_period: u64,
    pub dividend_start: u64,
    pub accrued_in: u64,
    pub dividend_seq: u64,
    pub crank_tip: u64,
    pub default_mint: Pubkey,
    pub name: String,
    pub desc: String,
    pub pentagons: Vec<PentagonView>,
    pub holders: Vec<HolderView>,
    /// The root unit of the tree: the config's key.
    pub root: u64,
    /// Holder rows the account has room for; it grows one at a time.
    pub capacity: usize,
}
impl RegistryView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        // a registry holds a whole number of rows, from none up to the cap
        let rows_ok = (Registry::MIN_SIZE..=Registry::MAX_SIZE).contains(&d.len()) && (d.len() - Registry::MIN_SIZE).is_multiple_of(Registry::HOLDER_LEN);
        if !rows_ok || d[OFF_DISC] != DISC_REGISTRY || d[OFF_VERSION] != LAYOUT_VERSION {
            return None;
        }
        let name_len = (d[Registry::OFF_NAME_LEN] as usize).min(MAX_NAME);
        let desc_len = (d[Registry::OFF_DESC_LEN] as usize).min(MAX_DESC);
        let name = String::from_utf8_lossy(&d[Registry::OFF_NAME..Registry::OFF_NAME + name_len]).into_owned();
        let desc = String::from_utf8_lossy(&d[Registry::OFF_DESC..Registry::OFF_DESC + desc_len]).into_owned();
        let pentagons = (0..PENTAGONS)
            .map(|k| {
                let o = Registry::OFF_PENTAGONS + k * Registry::PENTAGON_LEN;
                PentagonView { holder: h32_at(d, o), kind: d[o + 32], seated_slot: u64_at(d, o + 33), term_end: u64_at(d, o + 41) }
            })
            .collect();
        let n = d[Registry::OFF_HOLDER_COUNT] as usize;
        let holders = (0..n)
            .map(|i| {
                let o = Registry::OFF_HOLDERS + i * Registry::HOLDER_LEN;
                HolderView {
                    holder: h32_at(d, o),
                    kind: d[o + 32],
                    seats: u64_at(d, o + 33),
                    weight: u64_at(d, o + 41),
                    last_active: u64_at(d, o + 49),
                }
            })
            .collect();
        Some(Self {
            unit: u64_at(d, Registry::OFF_UNIT),
            parent: u64_at(d, Registry::OFF_PARENT),
            chair_in_parent: d[Registry::OFF_CHAIR],
            level: d[Registry::OFF_LEVEL],
            plan_cap: u64_at(d, Registry::OFF_PLAN_CAP),
            plan_period: u64_at(d, Registry::OFF_PLAN_PERIOD),
            plan_spent: u64_at(d, Registry::OFF_PLAN_SPENT),
            plan_start: u64_at(d, Registry::OFF_PLAN_START),
            rung: d[Registry::OFF_RUNG],
            frozen: d[Registry::OFF_FROZEN] != 0,
            total_seats: u64_at(d, Registry::OFF_TOTAL_SEATS),
            charter: h32_at(d, Registry::OFF_CHARTER),
            chain_sig: h32_at(d, Registry::OFF_CHAIN_SIG),
            window: u64_at(d, Registry::OFF_WINDOW),
            next_sale_id: u64_at(d, Registry::OFF_NEXT_SALE),
            seat_reserve: u64_at(d, Registry::OFF_SEAT_RESERVE),
            seat_deposit: u64_at(d, Registry::OFF_SEAT_DEPOSIT),
            last_clearing: u64_at(d, Registry::OFF_LAST_CLEARING),
            token_plan_cap: u64_at(d, Registry::OFF_TOKEN_PLAN_CAP),
            token_plan_spent: u64_at(d, Registry::OFF_TOKEN_PLAN_SPENT),
            codex_window: u64_at(d, Registry::OFF_CODEX_WINDOW),
            dividend_bps: u16_at(d, Registry::OFF_DIVIDEND_BPS),
            dividend_period: u64_at(d, Registry::OFF_DIVIDEND_PERIOD),
            dividend_start: u64_at(d, Registry::OFF_DIVIDEND_START),
            accrued_in: u64_at(d, Registry::OFF_ACCRUED_IN),
            dividend_seq: u64_at(d, Registry::OFF_DIVIDEND_SEQ),
            crank_tip: u64_at(d, Registry::OFF_CRANK_TIP),
            default_mint: key_at(d, Registry::OFF_DEFAULT_MINT),
            root: u64_at(d, Registry::OFF_ROOT),
            capacity: (d.len() - Registry::MIN_SIZE) / Registry::HOLDER_LEN,
            name,
            desc,
            pentagons,
            holders,
        })
    }
    pub fn seats_of(&self, kind: u8, holder: &[u8; 32]) -> u64 {
        self.holders.iter().find(|h| h.kind == kind && &h.holder == holder).map(|h| h.seats).unwrap_or(0)
    }
    pub fn chair_of_unit(&self, unit: u64) -> Option<usize> {
        let h = unit_holder(unit);
        self.pentagons.iter().position(|p| p.kind == HOLDER_UNIT && p.holder == h)
    }
    pub fn filled_chairs(&self) -> usize {
        self.pentagons.iter().filter(|p| p.kind != HOLDER_EMPTY).count()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaleView {
    pub unit: u64,
    pub sale_id: u64,
    pub seat_count: u64,
    pub reserve: u64,
    pub deposit: u64,
    pub commit_end: u64,
    pub reveal_end: u64,
    pub clearing: u64,
    pub overflow_price: u64,
    pub settled: bool,
    pub board: Vec<(u64, Pubkey, u64)>,
}
impl SaleView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        if d.len() < SeatSale::HEAD || d[OFF_DISC] != DISC_SEAT_SALE || d[OFF_VERSION] != LAYOUT_VERSION {
            return None;
        }
        let seat_count = u64_at(d, SeatSale::OFF_SEAT_COUNT);
        if d.len() != SeatSale::size(seat_count) {
            return None;
        }
        let filled = u64_at(d, SeatSale::OFF_FILLED) as usize;
        let board = (0..filled)
            .map(|i| {
                let o = SeatSale::HEAD + i * SeatSale::ENTRY;
                (u64_at(d, o), key_at(d, o + 8), u64_at(d, o + 40))
            })
            .collect();
        Some(Self {
            unit: u64_at(d, SeatSale::OFF_UNIT),
            sale_id: u64_at(d, SeatSale::OFF_SALE),
            seat_count,
            reserve: u64_at(d, SeatSale::OFF_RESERVE),
            deposit: u64_at(d, SeatSale::OFF_DEPOSIT),
            commit_end: u64_at(d, SeatSale::OFF_COMMIT_END),
            reveal_end: u64_at(d, SeatSale::OFF_REVEAL_END),
            clearing: u64_at(d, SeatSale::OFF_CLEARING),
            overflow_price: u64_at(d, SeatSale::OFF_OVERFLOW),
            settled: d[SeatSale::OFF_SETTLED] != 0,
            board,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposalView {
    pub hash: [u8; 32],
    /// The class without its bits.
    pub class: u8,
    /// The proposer asked that every chamber seal PASSED before execution.
    pub consent: bool,
    /// The proposer asked that a split of the origin's chambers be settled by the SUM.
    pub sum: bool,
    /// The proposer asked that a split of the origin's chambers, the chairs for, go to the market.
    pub market: bool,
    /// A plan: the origin's floor speaks by price.
    pub plan: bool,
    pub origin: u64,
    pub manifest: [u8; 32],
    pub payload_hash: [u8; 32],
    pub opened: u64,
    pub expires: u64,
    pub executed: bool,
    pub impacted: Vec<u64>,
    pub payer: Pubkey,
    pub payload: Vec<u8>,
}
impl ProposalView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        if !header_ok(d, DISC_PROPOSAL, Proposal::SIZE) {
            return None;
        }
        let n = d[Proposal::OFF_COUNT] as usize;
        let plen = u16_at(d, Proposal::OFF_PAYLOAD_LEN) as usize;
        Some(Self {
            hash: h32_at(d, Proposal::OFF_HASH),
            class: class_of(d[Proposal::OFF_CLASS]),
            consent: d[Proposal::OFF_CLASS] & CLASS_CONSENT != 0,
            sum: d[Proposal::OFF_CLASS] & CLASS_SUM != 0,
            market: d[Proposal::OFF_CLASS] & CLASS_MARKET != 0,
            plan: d[Proposal::OFF_CLASS] & CLASS_PLAN != 0,
            origin: u64_at(d, Proposal::OFF_ORIGIN),
            manifest: h32_at(d, Proposal::OFF_MANIFEST),
            payload_hash: h32_at(d, Proposal::OFF_PAYLOAD_HASH),
            opened: u64_at(d, Proposal::OFF_OPENED),
            expires: u64_at(d, Proposal::OFF_EXPIRES),
            executed: d[Proposal::OFF_EXECUTED] != 0,
            impacted: (0..n).map(|i| u64_at(d, Proposal::OFF_IMPACTED + i * 8)).collect(),
            payer: key_at(d, Proposal::OFF_PAYER),
            payload: d[Proposal::OFF_PAYLOAD..Proposal::OFF_PAYLOAD + plen.min(MAX_PAYLOAD)].to_vec(),
        })
    }
    /// A closed proposal: the tombstone that keeps the hash taken.
    pub fn is_tombstone(d: &[u8]) -> bool {
        d.len() == Proposal::TOMBSTONE_SIZE && d[OFF_DISC] == DISC_PROPOSAL && d[OFF_VERSION] == LAYOUT_VERSION
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionView {
    pub hash: [u8; 32],
    pub unit: u64,
    pub deadline: u64,
    pub hex_pro: u64,
    pub hex_contra: u64,
    pub hex_abst: u64,
    pub pent_pro: u8,
    pub pent_contra: u8,
    pub pent_abst: u8,
    pub chair_mask: u16,
    pub state: u8,
    pub votes: u32,
    pub active_weight: u64,
    pub chair_voice: [u8; Session::CHAIR_VOICE_BYTES],
    pub holder_voice: [u8; Session::HOLDER_VOICE_BYTES],
    /// Whoever convened the session, when its polis opened it itself (`convene`).
    pub convener: Option<Pubkey>,
}
impl SessionView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        if d.len() < Session::SIZE || d[OFF_DISC] != DISC_SESSION || d[OFF_VERSION] != LAYOUT_VERSION {
            return None;
        }
        if let Some((_, active)) = Session::electorate_weights(d).ok()? {
            if active != u64_at(d, Session::OFF_ACTIVE_WEIGHT) { return None; }
        }
        Some(Self {
            hash: h32_at(d, Session::OFF_HASH),
            unit: u64_at(d, Session::OFF_UNIT),
            deadline: u64_at(d, Session::OFF_DEADLINE),
            hex_pro: u64_at(d, Session::OFF_HEX_PRO),
            hex_contra: u64_at(d, Session::OFF_HEX_CONTRA),
            hex_abst: u64_at(d, Session::OFF_HEX_ABST),
            pent_pro: d[Session::OFF_PENT_PRO],
            pent_contra: d[Session::OFF_PENT_CONTRA],
            pent_abst: d[Session::OFF_PENT_ABST],
            chair_mask: u16_at(d, Session::OFF_CHAIR_MASK),
            state: d[Session::OFF_STATE],
            votes: u32_at(d, Session::OFF_VOTES),
            active_weight: u64_at(d, Session::OFF_ACTIVE_WEIGHT),
            chair_voice: d[Session::OFF_CHAIR_VOICE..Session::OFF_CHAIR_VOICE + Session::CHAIR_VOICE_BYTES].try_into().unwrap(),
            holder_voice: d[Session::OFF_HOLDER_VOICE..Session::OFF_HOLDER_VOICE + Session::HOLDER_VOICE_BYTES].try_into().unwrap(),
            convener: Session::convener(d).map(Pubkey::new_from_array),
        })
    }
    fn voice(bits: &[u8], i: usize) -> Option<u8> {
        let v = (bits[i / 4] >> ((i % 4) * 2)) & 0b11;
        (v != 0).then(|| v - 1)
    }
    /// Chair `k`'s voice: None while silent, else the vote code.
    pub fn chair_voice(&self, k: usize) -> Option<u8> {
        Self::voice(&self.chair_voice, k)
    }
    /// Holder `i`'s voice (registry index): None while silent, else the vote code.
    pub fn holder_voice(&self, i: usize) -> Option<u8> {
        Self::voice(&self.holder_voice, i)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DistributionView {
    pub hash: [u8; 32],
    pub unit: u64,
    pub amount: u64,
    pub total_weight: u64,
    pub paid: u64,
    pub claims: u32,
    /// Zero for lamports; the mint of a token distribution otherwise.
    pub mint: Pubkey,
    pub payer: Pubkey,
    pub created: u64,
    pub claimed: [u8; Distribution::CLAIM_BYTES],
}
impl DistributionView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        // the prefix alone (cut before the roster) or the prefix and the cut's roster
        (d.len() >= Distribution::SIZE && Distribution::valid_data_len(d) && header_ok(d, DISC_DISTRIBUTION, d.len())).then(|| Self {
            hash: h32_at(d, Distribution::OFF_HASH),
            unit: u64_at(d, Distribution::OFF_UNIT),
            amount: u64_at(d, Distribution::OFF_AMOUNT),
            total_weight: u64_at(d, Distribution::OFF_TOTAL_WEIGHT),
            paid: u64_at(d, Distribution::OFF_PAID),
            claims: u32_at(d, Distribution::OFF_CLAIMS),
            mint: key_at(d, Distribution::OFF_MINT),
            payer: key_at(d, Distribution::OFF_PAYER),
            created: u64_at(d, Distribution::OFF_CREATED),
            claimed: d[Distribution::OFF_CLAIMED..Distribution::OFF_CLAIMED + Distribution::CLAIM_BYTES].try_into().unwrap(),
        })
    }
    /// Whether the holder at registry index `i` has claimed.
    pub fn claimed(&self, i: usize) -> bool {
        self.claimed[i / 8] & (1 << (i % 8)) != 0
    }
    pub fn is_token(&self) -> bool {
        self.mint != Pubkey::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllowanceView {
    pub hash: [u8; 32],
    pub unit: u64,
    pub payee: [u8; 32],
    pub payee_kind: u8,
    pub mint: Pubkey,
    pub cap: u64,
    pub period: u64,
    pub start: u64,
    pub spent: u64,
    pub expires: u64,
    pub payer: Pubkey,
}
impl AllowanceView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_ALLOWANCE, Allowance::SIZE).then(|| Self {
            hash: h32_at(d, Allowance::OFF_HASH),
            unit: u64_at(d, Allowance::OFF_UNIT),
            payee: h32_at(d, Allowance::OFF_PAYEE),
            payee_kind: d[Allowance::OFF_PAYEE_KIND],
            mint: key_at(d, Allowance::OFF_MINT),
            cap: u64_at(d, Allowance::OFF_CAP),
            period: u64_at(d, Allowance::OFF_PERIOD),
            start: u64_at(d, Allowance::OFF_START),
            spent: u64_at(d, Allowance::OFF_SPENT),
            expires: u64_at(d, Allowance::OFF_EXPIRES),
            payer: key_at(d, Allowance::OFF_PAYER),
        })
    }
    /// What may still be pulled at `now`, the period rolled forward.
    pub fn available(&self, now: u64) -> u64 {
        if self.expires != 0 && now > self.expires {
            return 0;
        }
        if now >= self.start.saturating_add(self.period) {
            self.cap
        } else {
            self.cap.saturating_sub(self.spent)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContractView {
    pub hash: [u8; 32],
    pub unit: u64,
    pub payee: [u8; 32],
    pub payee_kind: u8,
    pub mint: Pubkey,
    pub amount: u64,
    pub deadline: u64,
    pub arbiter: u64,
    pub payer: Pubkey,
}
impl ContractView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_CONTRACT, Contract::SIZE).then(|| Self {
            hash: h32_at(d, Contract::OFF_HASH),
            unit: u64_at(d, Contract::OFF_UNIT),
            payee: h32_at(d, Contract::OFF_PAYEE),
            payee_kind: d[Contract::OFF_PAYEE_KIND],
            mint: key_at(d, Contract::OFF_MINT),
            amount: u64_at(d, Contract::OFF_AMOUNT),
            deadline: u64_at(d, Contract::OFF_DEADLINE),
            arbiter: u64_at(d, Contract::OFF_ARBITER),
            payer: key_at(d, Contract::OFF_PAYER),
        })
    }
    /// The reference a Rule or an expiry needs.
    pub fn reference(&self, token_program: Pubkey) -> ContractRef {
        ContractRef {
            unit: self.unit,
            payee: self.payee,
            payee_kind: self.payee_kind,
            mint: self.mint,
            payer: self.payer,
            token_program,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListingView {
    pub unit: u64,
    pub seller: Pubkey,
    pub nonce: u64,
    pub count: u64,
    pub price: u64,
}
impl ListingView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_LISTING, Listing::SIZE).then(|| Self {
            unit: u64_at(d, Listing::OFF_UNIT),
            seller: key_at(d, Listing::OFF_SELLER),
            nonce: u64_at(d, Listing::OFF_NONCE),
            count: u64_at(d, Listing::OFF_COUNT),
            price: u64_at(d, Listing::OFF_PRICE),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChairView {
    pub root: u64,
    pub fees: u64,
    pub backing: [u8; 32],
}
/// The fund: the polis of poleis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundView {
    pub fee_bps: u16,
    pub cycle_slots: u64,
    pub cycle_start: u64,
    pub cycle_seq: u64,
    pub total_in: u64,
    pub total_out: u64,
    pub staked_total: u64,
    pub wp_sum: u128,
    pub wp_weight: u64,
    pub range_lo: u16,
    pub range_hi: u16,
    pub option_fee: u64,
    pub chair_quorum: u8,
    pub current: [u8; 32],
    pub current_since: u64,
    pub chairs: Vec<ChairView>,
    pub current_recipient: Pubkey,
    pub owed: u64,
    pub owed_to: Pubkey,
}
impl FundView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_FUND, Fund::SIZE).then(|| Self {
            fee_bps: u16_at(d, Fund::OFF_FEE_BPS),
            cycle_slots: u64_at(d, Fund::OFF_CYCLE_SLOTS),
            cycle_start: u64_at(d, Fund::OFF_CYCLE_START),
            cycle_seq: u64_at(d, Fund::OFF_CYCLE_SEQ),
            total_in: u64_at(d, Fund::OFF_TOTAL_IN),
            total_out: u64_at(d, Fund::OFF_TOTAL_OUT),
            staked_total: u64_at(d, Fund::OFF_STAKED),
            wp_sum: u128::from_le_bytes(d[Fund::OFF_WP_SUM..Fund::OFF_WP_SUM + 16].try_into().unwrap()),
            wp_weight: u64_at(d, Fund::OFF_WP_WEIGHT),
            range_lo: u16_at(d, Fund::OFF_RANGE_LO),
            range_hi: u16_at(d, Fund::OFF_RANGE_HI),
            option_fee: u64_at(d, Fund::OFF_OPTION_FEE),
            chair_quorum: d[Fund::OFF_CHAIR_QUORUM],
            current: d[Fund::OFF_CURRENT..Fund::OFF_CURRENT + 32].try_into().unwrap(),
            current_since: u64_at(d, Fund::OFF_CURRENT_SINCE),
            chairs: (0..PENTAGONS)
                .map(|k| {
                    let o = Fund::OFF_CHAIRS + k * Fund::CHAIR_LEN;
                    ChairView { root: u64_at(d, o), fees: u64_at(d, o + 8), backing: d[o + 16..o + 48].try_into().unwrap() }
                })
                .collect(),
            current_recipient: Pubkey::new_from_array(d[Fund::OFF_CURRENT_RECIPIENT..Fund::OFF_CURRENT_RECIPIENT + 32].try_into().unwrap()),
            owed: u64_at(d, Fund::OFF_OWED),
            owed_to: Pubkey::new_from_array(d[Fund::OFF_OWED_TO..Fund::OFF_OWED_TO + 32].try_into().unwrap()),
        })
    }
    /// The payout share the floor named, clamped to the range: what the next settle pays.
    pub fn payout_bps(&self) -> u16 {
        if self.wp_weight == 0 {
            self.range_lo
        } else {
            (self.wp_sum / self.wp_weight as u128).clamp(self.range_lo as u128, self.range_hi as u128) as u16
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TallyView {
    pub root: u64,
    pub cycle: u64,
    pub fees: u64,
    pub lifetime: u64,
}
impl TallyView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_FEE_TALLY, FeeTally::SIZE).then(|| Self {
            root: u64_at(d, FeeTally::OFF_ROOT),
            cycle: u64_at(d, FeeTally::OFF_CYCLE),
            fees: u64_at(d, FeeTally::OFF_FEES),
            lifetime: u64_at(d, FeeTally::OFF_LIFETIME),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionView {
    pub hash: [u8; 32],
    pub recipient: Pubkey,
    pub manifest: [u8; 32],
    pub proposer: Pubkey,
    pub vetted: bool,
    pub floor_stake: u64,
    pub created: u64,
}
impl OptionView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_OPTION, Candidate::SIZE).then(|| Self {
            hash: d[Candidate::OFF_HASH..Candidate::OFF_HASH + 32].try_into().unwrap(),
            recipient: key_at(d, Candidate::OFF_RECIPIENT),
            manifest: d[Candidate::OFF_MANIFEST..Candidate::OFF_MANIFEST + 32].try_into().unwrap(),
            proposer: key_at(d, Candidate::OFF_PROPOSER),
            vetted: d[Candidate::OFF_VETTED] != 0,
            floor_stake: u64_at(d, Candidate::OFF_FLOOR_STAKE),
            created: u64_at(d, Candidate::OFF_CREATED),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StakeView {
    pub hash: [u8; 32],
    pub unit: u64,
    pub holder: Pubkey,
    pub weight: u64,
    pub value_bps: u16,
}
impl StakeView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_STAKE, Stake::SIZE).then(|| Self {
            hash: d[Stake::OFF_HASH..Stake::OFF_HASH + 32].try_into().unwrap(),
            unit: u64_at(d, Stake::OFF_UNIT),
            holder: key_at(d, Stake::OFF_HOLDER),
            weight: u64_at(d, Stake::OFF_WEIGHT),
            value_bps: u16_at(d, Stake::OFF_VALUE),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolView {
    pub unit: u64,
    pub open: u64,
    pub filled: u64,
    pub price: u64,
}
impl PoolView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_POOL, Pool::SIZE).then(|| Self {
            unit: u64_at(d, Pool::OFF_UNIT),
            open: u64_at(d, Pool::OFF_OPEN),
            filled: u64_at(d, Pool::OFF_FILLED),
            price: u64_at(d, Pool::OFF_PRICE),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnockView {
    pub unit: u64,
    pub key: Pubkey,
    pub deposit: u64,
    pub slot: u64,
}
impl KnockView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_KNOCK, Knock::SIZE).then(|| Self {
            unit: u64_at(d, Knock::OFF_UNIT),
            key: key_at(d, Knock::OFF_KEY),
            deposit: u64_at(d, Knock::OFF_DEPOSIT),
            slot: u64_at(d, Knock::OFF_SLOT),
        })
    }
}

/// A unit's own law on how a split is settled (`CODEX_RESOLUTION`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleView {
    pub unit: u64,
    pub sum_kinds: u8,
    pub small_cap: u64,
}
impl RuleView {
    pub fn parse(d: &[u8]) -> Option<Self> {
        header_ok(d, DISC_RULE, Rule::SIZE).then(|| Self {
            unit: u64_at(d, Rule::OFF_UNIT),
            sum_kinds: d[Rule::OFF_SUM_KINDS],
            small_cap: u64_at(d, Rule::OFF_SMALL_CAP),
        })
    }
}

/// A receipt as the program logs it (`Program data: <base64>` in the
/// transaction log): the structured record of a seal, a payment, a pull, an
/// escrow, a release or refund, a dividend, a claim, a seat sale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub kind: u8,
    pub unit: u64,
    pub hash: [u8; 32],
    pub counterparty: [u8; 32],
    pub mint: Pubkey,
    pub amount: u64,
    pub outcome: u8,
}
impl Receipt {
    /// Parse the decoded bytes of one log data entry; None if it is not a receipt.
    pub fn parse(b: &[u8]) -> Option<Self> {
        if b.len() != RECEIPT_LEN || &b[..RECEIPT_DOMAIN.len()] != RECEIPT_DOMAIN {
            return None;
        }
        let o = RECEIPT_DOMAIN.len();
        Some(Self {
            kind: b[o],
            unit: u64_at(b, o + 1),
            hash: h32_at(b, o + 9),
            counterparty: h32_at(b, o + 41),
            mint: key_at(b, o + 73),
            amount: u64_at(b, o + 105),
            outcome: b[o + 113],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_proposal_reads_its_class_without_its_four_bits() {
        assert_eq!(CLASS_BITS, CLASS_CONSENT | CLASS_SUM | CLASS_MARKET | CLASS_PLAN);
        for class in [CLASS_CODEX, CLASS_SIGNAL, CLASS_TREASURY, CLASS_SEAT, CLASS_ACT] {
            for bits in 0u8..16 {
                let byte = class | (bits << 4);
                let mut d = vec![0u8; Proposal::SIZE];
                d[OFF_DISC] = DISC_PROPOSAL;
                d[OFF_VERSION] = LAYOUT_VERSION;
                d[Proposal::OFF_CLASS] = byte;
                d[Proposal::OFF_COUNT] = 1;
                d[Proposal::OFF_PAYLOAD_LEN..Proposal::OFF_PAYLOAD_LEN + 2].copy_from_slice(&2u16.to_le_bytes());
                let p = ProposalView::parse(&d).expect("a proposal");
                assert_eq!(p.class, class, "byte {byte:#x}");
                assert_eq!((p.consent, p.sum, p.market, p.plan), (byte & CLASS_CONSENT != 0, byte & CLASS_SUM != 0, byte & CLASS_MARKET != 0, byte & CLASS_PLAN != 0));
            }
        }
    }

    /// THE SEAL NAMES ITS PROGRAM: the act that moves a program's upgrade authority carries the program in its payload, so the act's hash
    /// covers it, and the only program data its carrier is handed is that program's own.
    #[test]
    fn an_authority_act_names_its_program_in_its_bytes_its_hash_and_its_accounts() {
        let (market, warm, holder) = (Pubkey::new_from_array([7u8; 32]), Pubkey::new_from_array([8u8; 32]), Pubkey::new_from_array([9u8; 32]));
        let act = |program: Pubkey, new: Option<Pubkey>| Payload::UpgradeAuthority { origin: ROOT_UNIT, program, new };
        let hand = act(market, Some(holder));
        assert_eq!(hand.bytes(), [&[CLASS_CODEX, CODEX_UPGRADE_AUTHORITY][..], holder.as_ref(), market.as_ref()].concat(), "[class][kind][new:32][program:32]");
        assert_eq!(act(market, None).bytes(), [&[CLASS_CODEX, CODEX_UPGRADE_AUTHORITY][..], &[0u8; 32][..], market.as_ref()].concat(), "a burn: no new holder, its program still named");
        assert_ne!(hand.hash(), act(warm, Some(holder)).hash(), "the same hand of another program is another act");
        assert_ne!(act(market, None).hash(), act(warm, None).hash(), "the same burn of another program is another act");
        let metas = hand.accounts(&[0u8; 32]);
        assert_eq!(
            metas.iter().map(|m| (m.pubkey, m.is_writable)).collect::<Vec<_>>(),
            vec![(registry_pda(ROOT_UNIT), false), (programdata_of(&market), true), (upgrade_pda(), false), (BPF_LOADER_UPGRADEABLE_ID, false), (holder, false)],
            "the registry, the named program's own program data, the seal's PDA, the loader, the new holder"
        );
        assert_eq!(act(warm, None).accounts(&[0u8; 32])[1].pubkey, programdata_of(&warm), "a burn reaches its own program's data, and no fifth account");
        assert_eq!(act(warm, None).accounts(&[0u8; 32]).len(), 4);
        assert_eq!(programdata_pda(), programdata_of(&PROGRAM_ID));
        assert_ne!(programdata_of(&market), programdata_of(&warm));
    }

    /// THE SEAL CLOSES A BUFFER: the act names the buffer and where its lamports go, its hash covers both, and its carrier is handed those two
    /// accounts to write and no other of the loader's.
    #[test]
    fn a_close_buffer_act_names_its_buffer_and_its_recipient() {
        let (buffer, other, recipient) = (Pubkey::new_from_array([4u8; 32]), Pubkey::new_from_array([5u8; 32]), Pubkey::new_from_array([6u8; 32]));
        let act = |buffer: Pubkey, recipient: Pubkey| Payload::CloseBuffer { origin: ROOT_UNIT, buffer, recipient };
        let close = act(buffer, recipient);
        assert_eq!((close.class(), close.origin()), (CLASS_CODEX, ROOT_UNIT));
        assert_eq!(close.bytes(), [&[CLASS_CODEX, CODEX_CLOSE_BUFFER][..], buffer.as_ref(), recipient.as_ref()].concat(), "[class][kind][buffer:32][recipient:32]");
        assert_ne!(close.hash(), act(other, recipient).hash(), "the close of another buffer is another act");
        assert_ne!(close.hash(), act(buffer, other).hash(), "the same close to another recipient is another act");
        assert_eq!(
            close.accounts(&[0u8; 32]).iter().map(|m| (m.pubkey, m.is_writable, m.is_signer)).collect::<Vec<_>>(),
            vec![(registry_pda(ROOT_UNIT), false, false), (buffer, true, false), (recipient, true, false), (upgrade_pda(), false, false), (BPF_LOADER_UPGRADEABLE_ID, false, false)],
            "the registry, the named buffer, the named recipient, the seal's PDA, the loader: never a fourth account of the loader's (a program)"
        );
    }

    /// THE SEAL UPGRADES EVERY PROGRAM: the act names the program whose code it replaces, its hash covers the name, and its carrier is handed that
    /// program, that program's data, the named buffer and spill, and nothing else to write.
    #[test]
    fn an_upgrade_names_its_program_in_its_bytes_its_hash_and_its_accounts() {
        let (market, warm, buffer, spill) = (Pubkey::new_from_array([7u8; 32]), Pubkey::new_from_array([8u8; 32]), Pubkey::new_from_array([4u8; 32]), Pubkey::new_from_array([6u8; 32]));
        let act = |program: Pubkey| Payload::UpgradeProgram { origin: ROOT_UNIT, program, buffer, spill };
        let up = act(warm);
        assert_eq!((up.class(), up.origin()), (CLASS_CODEX, ROOT_UNIT));
        assert_eq!(up.bytes(), [&[CLASS_CODEX, CODEX_UPGRADE_PROGRAM][..], buffer.as_ref(), spill.as_ref(), warm.as_ref()].concat(), "[class][kind][buffer:32][spill:32][program:32]");
        assert_ne!(up.hash(), act(market).hash(), "the same upgrade of another program is another act");
        assert_eq!(
            up.accounts(&[0u8; 32]).iter().map(|m| (m.pubkey, m.is_writable)).collect::<Vec<_>>(),
            vec![
                (registry_pda(ROOT_UNIT), false), (programdata_of(&warm), true), (warm, true), (buffer, true), (spill, true),
                (RENT_SYSVAR_ID, false), (CLOCK_SYSVAR_ID, false), (upgrade_pda(), false), (BPF_LOADER_UPGRADEABLE_ID, false),
            ],
            "the registry, the named program's data and the program, the buffer, the spill, the rent, the clock, the seal's PDA, the loader"
        );
        assert!(up.accounts(&[0u8; 32]).iter().all(|m| !m.is_signer), "nobody signs for the code: the seal's PDA signs inside the program");
    }

    #[test]
    fn program_id_matches_the_program() {
        assert_eq!(PROGRAM_ID.to_string(), "AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab");
        assert_eq!(ED25519_PROGRAM_ID.to_string(), "Ed25519SigVerify111111111111111111111111111");
        assert_eq!(INSTRUCTIONS_SYSVAR_ID.to_string(), "Sysvar1nstructions1111111111111111111111111");
    }

    #[test]
    fn ed25519_layout_points_inside_itself() {
        let e = SignedEnvelope { pubkey: [1u8; 32], signature: [2u8; 64], message: vec![3u8; ENVELOPE_LEN] };
        let ixn = ed25519_instruction(&[e]);
        let d = &ixn.data;
        assert_eq!(d[0], 1);
        let rd = |o: usize| u16::from_le_bytes([d[o], d[o + 1]]) as usize;
        let (sig_off, pk_off, msg_off, msg_len) = (rd(2), rd(6), rd(10), rd(12));
        assert_eq!(rd(4), u16::MAX as usize);
        assert_eq!(&d[pk_off..pk_off + 32], &[1u8; 32]);
        assert_eq!(&d[sig_off..sig_off + 64], &[2u8; 64]);
        assert_eq!(msg_len, ENVELOPE_LEN);
        assert_eq!(&d[msg_off..msg_off + msg_len], &[3u8; ENVELOPE_LEN][..]);
    }

    #[test]
    fn payload_bytes_hash_stably() {
        let p = Payload::Transfer { origin: 0, dest: 1, amount: 5 };
        assert_eq!(p.bytes()[0], CLASS_TREASURY);
        assert_eq!(p.bytes()[1], TREASURY_TRANSFER);
        assert_eq!(p.hash(), sha(&[&p.bytes()]));
    }
}
