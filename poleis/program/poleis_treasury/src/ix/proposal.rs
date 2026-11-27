//! The double seal. A proposal names what it touches; every touched unit
//! holds a session, opened with the proposal or by the unit itself
//! (`convene`); votes signed off-chain land in batches through the Ed25519
//! precompile; a unit bound in a parent's chair speaks through its own seal;
//! the seal computes both chambers; execution cashes the payload only when
//! the tree passed, at most once, permissionlessly.
//!
//! It is not enough that the chairs say so. It is not enough that the floor
//! says so. Both must converge.

use super::auction::{open_lot, LotParams};
use super::council::{found_unit, parse_text, FoundParams};
use super::economy::{
    allow_mint, open_allowance, open_contract, open_distribution, read_distribution,
    revoke_allowance, rule_contract, set_dividend_policy, ZERO,
};
use super::sale::{check_windows, open_sale};
use super::vault::{expect_escrow, expect_vault, token_account_owner, token_balance, transfer_from_vault};
use crate::error::PoleisError;
use crate::state::{holder_unit, unit_holder, Market, Pentagon, Proposal, Registry, Rule, SaleHead, Session, Treasury, DISC_PROPOSAL, OFF_DISC};
use crate::util::{
    close_to, create_pda, envelope, expect_pda, expect_signer, expect_writable,
    find_and_expect_pda, move_lamports, receipt, sha256v, Data, ENVELOPE_LEN,
};
use pinocchio::{
    cpi::{invoke_signed_with_bounds, Seed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    sysvars::{
        clock::Clock,
        instructions::{Instructions, INSTRUCTIONS_ID},
        rent::Rent,
        Sysvar,
    },
    AccountView, Address, ProgramResult, Resize,
};

pub(super) fn read_session(session: &AccountView, hash: &[u8; 32], unit_id: u64) -> Result<Session, ProgramError> {
    let s = Session::read(session)?;
    if s.hash != *hash || s.unit_id != unit_id {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(session, &[b"session", hash, &unit_id.to_le_bytes()], s.bump)?;
    Ok(s)
}

fn read_proposal(proposal: &AccountView, hash: &[u8; 32]) -> Result<Proposal, ProgramError> {
    let p = Proposal::read(proposal)?;
    if p.hash != *hash {
        return Err(PoleisError::InvalidAccount.into());
    }
    expect_pda(proposal, &[b"proposal", hash], p.bump)?;
    Ok(p)
}

/// data: [hash:32][class:1][origin:8][manifest:32][count:1][impacted: count × 8]
///       [payload_len:2][payload: class:1, kind:1, fields...]
/// The class byte's high bit (`CLASS_CONSENT`) asks for consent: every
/// chamber must seal PASSED before the act executes, and each seal carries
/// its crews' AND (`seal`), so the origin's seal says it for the whole tree.
/// Without it a crew's verdict is its chair's vote and the origin's seal
/// alone decides. The second bit (`CLASS_SUM`) asks that a split of the
/// origin's two chambers be settled by the sum of its halves (`seal`); a
/// proposal that asks carries the origin's rule account last, and it is
/// granted only as the origin's own law says (`CODEX_RESOLUTION`): a kind
/// its rule turned on, a treasury sum at or under its small cap, never with
/// consent. The third and fourth bits (`CLASS_MARKET`, `CLASS_PLAN`) ask
/// for the market (`seal`); a proposal that asks carries the origin's
/// market account last, where the SUM carries its rule, and it is granted
/// only for a class that can ever ask (`market_bit`; a plan only for money
/// or the hands), alone of the four bits, and named by the origin's own law
/// (`CODEX_MARKET`) for that mode. The floor reads the bits with the
/// payload; the byte is stored as sent.
/// accounts: [proposer (s,w), config, proposal (w), system,
///            then count registries (impacted units, same order),
///            then count sessions (w, same order),
///            then, under the SUM, the origin's rule,
///            or, under the market or a plan, the origin's market]
/// The proposer is a member of the origin unit. The impacted set is the
/// origin and any of its descendants, each with its parent in the set: a
/// session opens in each, at most twelve with the proposal. Every other
/// crew seated in a chair of the decision opens its own (`convene`), in its
/// own transaction, so no tree is too wide to decide. No decision is ever
/// sealed over the head of a crew: every seal waits for the crews in its lit
/// chairs, the leaves speak first. The payload is stored in clear: what the
/// floor reads is what will execute.
pub fn propose(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    propose_with_nft(accounts, data, None)
}

/// Same proposal rules and wallet payer; prefix accounts [seat, mint, token]
/// prove membership through the NFT instead of a separately owned legacy seat.
pub fn propose_nft(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() < 3 { return Err(ProgramError::NotEnoughAccountKeys); }
    let (proof, rest) = accounts.split_at_mut(3);
    propose_with_nft(rest, data, Some(proof))
}

fn propose_with_nft(accounts: &mut [AccountView], data: &[u8], nft: Option<&[AccountView]>) -> ProgramResult {
    if data.len() < 86 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let d = Data(data);
    let hash = d.h32(0)?;
    let class_byte = data[32];
    let class = crate::class_of(class_byte);
    let sum = class_byte & crate::CLASS_SUM != 0;
    let plan = class_byte & crate::CLASS_PLAN != 0;
    let market = plan || class_byte & crate::CLASS_MARKET != 0;
    let origin = d.u64(33)?;
    let manifest = d.h32(41)?;
    let count = data[73] as usize;
    if count == 0 || count > crate::MAX_IMPACTED {
        return Err(ProgramError::InvalidInstructionData);
    }
    let off_len = 74 + count * 8;
    if data.len() < off_len + 2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let payload_len = u16::from_le_bytes([data[off_len], data[off_len + 1]]) as usize;
    if !(2..=crate::MAX_PAYLOAD).contains(&payload_len) || data.len() != off_len + 2 + payload_len {
        return Err(ProgramError::InvalidInstructionData);
    }
    let payload = &data[off_len + 2..];
    if class > crate::CLASS_ACT || payload[0] != class {
        return Err(PoleisError::ClassMismatch.into());
    }
    // the market only for a class that can ever ask it (a plan only for money or the hands), and alone of the four bits
    let mbit = crate::market_bit(class);
    if market
        && (mbit == 0
            || (plan && mbit & crate::PLAN_CLASSES == 0)
            || (class_byte & crate::CLASS_BITS).count_ones() != 1)
    {
        return Err(PoleisError::MarketNotAllowed.into());
    }
    // the SUM only for a kind that can ever take it, and never with consent
    let bit = crate::sum_bit(class, payload[1]);
    if sum && (bit == 0 || class_byte & crate::CLASS_CONSENT != 0) {
        return Err(PoleisError::SumNotAllowed.into());
    }
    if hash == [0u8; 32] {
        return Err(PoleisError::InvalidAccount.into());
    }
    let mut impacted = [0u64; crate::MAX_IMPACTED];
    for (i, slot) in impacted.iter_mut().enumerate().take(count) {
        *slot = d.u64(74 + i * 8)?;
    }
    for i in 0..count {
        for j in 0..i {
            if impacted[i] == impacted[j] {
                return Err(ProgramError::InvalidInstructionData);
            }
        }
    }
    if !impacted[..count].contains(&origin) {
        return Err(PoleisError::NotImpacted.into());
    }

    if accounts.len() != 4 + 2 * count + ((sum || market) as usize) {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, tail) = accounts.split_at_mut(4);
    let [proposer, config, proposal, _system] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let (registries, tail) = tail.split_at_mut(count);
    let (sessions, rule) = tail.split_at_mut(count);
    expect_signer(proposer)?;
    let now = Clock::get()?.slot;

    // the subtree law and the member law; a dissolved unit may still pass
    // one thing, a token dividend, so nothing is stranded in its vaults;
    // one decision lives in one tree, under that tree's council
    let sweep = sweeps(class, payload[1]);
    let mut expires = now;
    let mut root = None;
    for i in 0..count {
        let r = super::read_registry(&registries[i], impacted[i])?;
        if r.frozen && !sweep {
            return Err(PoleisError::Frozen.into());
        }
        if impacted[i] == origin {
            if let Some([seat, mint, token]) = nft {
                super::nft::current_owner(proposer, seat, mint, token, &registries[i])?;
            } else {
                super::expect_member(&registries[i], proposer, now)?;
            }
        } else if !impacted[..count].contains(&r.parent_unit) {
            return Err(PoleisError::NotDescendant.into());
        }
        match root {
            None => root = Some(r.root),
            Some(x) if x != r.root => return Err(PoleisError::NotDescendant.into()),
            _ => {}
        }
        // a society buys fast and rewrites itself slowly
        let window = if class == crate::CLASS_CODEX { r.codex_window } else { r.window };
        let deadline = now.checked_add(window).ok_or(PoleisError::MathOverflow)?;
        expires = expires.max(deadline);
    }
    super::expect_council(config, root.ok_or(PoleisError::NotImpacted)?)?;

    // the market is granted as the origin's own law says: its market names
    // the class for that mode (`split` for the market, `plan` for a plan)
    if let ([law], true) = (&*rule, market) {
        find_and_expect_pda(law, &[b"market", &origin.to_le_bytes()]).map_err(|_| PoleisError::MarketNotAllowed)?;
        let m = Market::read(law).map_err(|_| PoleisError::MarketNotAllowed)?;
        let mask = if plan { m.plan } else { m.split };
        if m.unit_id != origin || mask & mbit == 0 {
            return Err(PoleisError::MarketNotAllowed.into());
        }
        // THE MARKET NEVER DECIDES ITS OWN LAW: an act into the market's own program, or one that lists the origin's own board (which also
        // stops a relay through a third program for every instruction that reads the board), is the market's law by another door; it stays
        // proposable without the bit, under the AND. An act that cannot be parsed that far is refused the same way. Said, not built: a
        // judge that is still the polis's own hand could have its receipts sent by a market-passed act through a relay program (`receipts`
        // reads no board); receipts move settlements, never a verdict.
        if class == crate::CLASS_ACT && payload[1] == crate::ACT_RAW && touches_market(&payload[2..], &m) {
            return Err(PoleisError::MarketNotAllowed.into());
        }
    } else if let [rule] = rule {
        // the SUM is granted as the origin's own law says: its rule turned the
        // kind on, and a treasury sum stands at or under the origin's small cap
        find_and_expect_pda(rule, &[b"rule", &origin.to_le_bytes()])?;
        let law = Rule::read(rule).map_err(|_| PoleisError::SumNotAllowed)?;
        if law.unit_id != origin || law.sum_kinds & bit == 0 {
            return Err(PoleisError::SumNotAllowed.into());
        }
        if class == crate::CLASS_TREASURY {
            // [dest_unit:8][amount:8] for a transfer, [address:32][amount:8] for a budget
            let at = if payload[1] == crate::TREASURY_TRANSFER { 10 } else { 34 };
            let amount = Data(payload).u64(at).map_err(|_| PoleisError::BadPayload)?;
            if amount > law.small_cap {
                return Err(PoleisError::SumNotAllowed.into());
            }
        }
    }

    let seeds: &[&[u8]] = &[b"proposal", &hash];
    let bump = find_and_expect_pda(proposal, seeds)?;
    create_pda(proposer, proposal, Proposal::SIZE, seeds, bump)?;
    Proposal {
        hash,
        class: class_byte,
        origin,
        manifest,
        payload_hash: sha256v(&[payload]),
        opened: now,
        expires,
        executed: false,
        count: count as u8,
        bump,
        impacted,
        payer: *proposer.address().as_array(),
        payload_len: payload_len as u16,
    }
    .write(proposal)?;
    Proposal::write_payload(proposal, payload)?;

    for i in 0..count {
        let r = Registry::head(&registries[i])?;
        let window = if class == crate::CLASS_CODEX { r.codex_window } else { r.window };
        open_session(proposer, &mut sessions[i], &registries[i], &hash, impacted[i], now + window, now, 0)?;
    }
    Ok(())
}

/// Whether a raw act (`[program:32][n:1][n × (key:32, flags:1)]...`) calls the market's own program or lists the origin's own board; an act
/// too short to say is taken to (refused).
fn touches_market(fields: &[u8], m: &Market) -> bool {
    if fields.len() < 33 {
        return true;
    }
    let n = fields[32] as usize;
    if fields.len() < 33 + n * 33 {
        return true;
    }
    if fields[..32] == m.program {
        return true;
    }
    let board = crate::roads::board_address(&Address::new_from_array(m.program), m.board);
    (0..n).any(|i| fields[33 + i * 33..33 + i * 33 + 32] == board.as_ref()[..])
}

/// A unit's session of a decision, opened empty: its ballot box at
/// `["session", hash, unit]`, paid by `payer`, its window to `deadline`,
/// and its roster written now: the holders with their weights, the chairs
/// lit and the crews among them, and this slot. Who is counted is who stood
/// there when the vote opened: every vote, the chairs' bar, the wait for
/// crews and the consent AND are judged against it, never against the
/// registry as it moves under the vote. `tail` is room left at its end for
/// the convener.
#[allow(clippy::too_many_arguments)]
fn open_session(
    payer: &AccountView,
    session: &mut AccountView,
    registry: &AccountView,
    hash: &[u8; 32],
    unit: u64,
    deadline: u64,
    now: u64,
    tail: usize,
) -> ProgramResult {
    let unit_le = unit.to_le_bytes();
    let seeds: &[&[u8]] = &[b"session", hash, &unit_le];
    let bump = find_and_expect_pda(session, seeds)?;
    create_pda(payer, session, Session::size_for_registry(registry)? + tail, seeds, bump)?;
    let active_weight = Session::init_electorate(session, registry, now, tail)?;
    Session {
        hash: *hash,
        unit_id: unit,
        deadline,
        hex_pro: 0,
        hex_contra: 0,
        hex_abst: 0,
        pent_pro: 0,
        pent_contra: 0,
        pent_abst: 0,
        chair_mask: 0,
        state: crate::STATE_VOTING,
        bump,
        counted: [0u8; 24],
        votes: 0,
        active_weight,
        chair_voice: [0u8; Session::CHAIR_VOICE_BYTES],
        holder_voice: [0u8; Session::HOLDER_VOICE_BYTES],
    }
    .write(session)
}

/// The one thing a dissolved unit may still pass, so nothing is stranded in
/// its vaults: a dividend, in lamports or in a token.
fn sweeps(class: u8, kind: u8) -> bool {
    class == crate::CLASS_TREASURY && (kind == crate::TREASURY_TOKEN_DIVIDEND || kind == crate::TREASURY_DIVIDEND)
}

/// data: [hash:32][parent:8][unit:8]
/// accounts: [crank (s,w), proposal, parent registry, parent session,
///            registry, session (w), system]
/// Each polis opens its own vote. A unit seated in a lit chair of a unit
/// whose session of this decision is open opens its own session of it, in
/// its own transaction, permissionlessly, while that session's window runs;
/// the crank pays the rent and it comes home to them (`adjourn`). Its window
/// runs from the proposal's opening, as if it had opened with it, so a late
/// convening extends nothing. A decision costs one transaction per polis it
/// touches, and a tree has no bound but its depth. When the parent's
/// session carries its roster, the unit must hold a chair that roster lit
/// and nobody has touched since: a crew seated after the parent's vote
/// opened is not on it, and opens nothing for it.
pub fn convene(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, proposal, parent_registry, parent_session, registry, session, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(48)?;
    let hash = d.h32(0)?;
    let parent = d.u64(32)?;
    let unit = d.u64(40)?;
    expect_signer(crank)?;
    let p = read_proposal(proposal, &hash)?;
    if p.executed {
        return Err(PoleisError::AlreadyExecuted.into());
    }
    let ps = read_session(parent_session, &hash, parent)?;
    let now = Clock::get()?.slot;
    if ps.terminal() || now > ps.deadline {
        return Err(PoleisError::SessionClosed.into());
    }
    let above = super::read_registry(parent_registry, parent)?;
    // a crew seated in its chair after the parent's vote opened waits for
    // the next decision: it is not on that vote's roster
    let chairs = Registry::chairs_of_unit(parent_registry, unit, now)? & speakers(parent_session, parent_registry)?;
    if chairs == 0 {
        return Err(PoleisError::InvalidHolder.into());
    }
    if chairs & !ps.chair_mask == 0 {
        return Err(PoleisError::AlreadyMandated.into());
    }
    // one decision lives in one tree; a dissolved unit votes only on a sweep
    let r = super::read_registry(registry, unit)?;
    if r.root != above.root {
        return Err(PoleisError::NotDescendant.into());
    }
    if r.frozen && !sweeps(p.class_of(), Proposal::payload_kind(proposal)?) {
        return Err(PoleisError::Frozen.into());
    }
    let window = if p.class_of() == crate::CLASS_CODEX { r.codex_window } else { r.window };
    let deadline = p.opened.checked_add(window).ok_or(PoleisError::MathOverflow)?;
    open_session(crank, session, registry, &hash, unit, deadline, now, Session::CONVENER_TAIL)?;
    Session::write_convener(session, crank.address().as_array())
}

/// The Ed25519 precompile's instruction data, checked against the batch:
/// [n:1][pad:1] then n × 14-byte offsets, then the keys, signatures and
/// messages. Every reference must point inside this same instruction
/// (index u16::MAX), the key must be the voter, and the message must be
/// exactly the envelope we recompute. The runtime already verified the
/// signatures before this instruction ran; we verify what they signed.
fn check_precompile(
    ix_data: &[u8],
    hash: &[u8; 32],
    unit_id: u64,
    votes: &[u8],
    n: usize,
) -> Result<(), ProgramError> {
    if ix_data.len() < 2 || ix_data[0] as usize != n {
        return Err(PoleisError::BadPrecompile.into());
    }
    for i in 0..n {
        let o = 2 + i * 14;
        if ix_data.len() < o + 14 {
            return Err(PoleisError::BadPrecompile.into());
        }
        let rd = |k: usize| u16::from_le_bytes([ix_data[o + k], ix_data[o + k + 1]]) as usize;
        let (sig_off, sig_ix) = (rd(0), rd(2));
        let (pk_off, pk_ix) = (rd(4), rd(6));
        let (msg_off, msg_len, msg_ix) = (rd(8), rd(10), rd(12));
        if sig_ix != u16::MAX as usize || pk_ix != u16::MAX as usize || msg_ix != u16::MAX as usize {
            return Err(PoleisError::BadPrecompile.into());
        }
        if sig_off + 64 > ix_data.len() || pk_off + 32 > ix_data.len() || msg_off + msg_len > ix_data.len() {
            return Err(PoleisError::BadPrecompile.into());
        }
        let voter: [u8; 32] = votes[i * 33..i * 33 + 32].try_into().unwrap();
        let choice = votes[i * 33 + 32];
        if ix_data[pk_off..pk_off + 32] != voter {
            return Err(PoleisError::BadPrecompile.into());
        }
        let expected = envelope(hash, unit_id, choice);
        if msg_len != ENVELOPE_LEN || ix_data[msg_off..msg_off + msg_len] != expected {
            return Err(PoleisError::BadPrecompile.into());
        }
    }
    Ok(())
}

/// The chairs that may speak in a session: for one that carries its roster,
/// the chairs lit at its opening that nobody has seated, reseated or emptied
/// since; for one opened before the roster, every chair, judged at the
/// moment as it always was. Read once per instruction.
pub(super) fn speakers(session: &AccountView, registry: &AccountView) -> Result<u16, ProgramError> {
    match Session::roster(&session.try_borrow()?)? {
        Some((lit, _, opened)) => Ok(lit & Registry::unchanged_since(registry, opened)?),
        None => Ok(u16::MAX),
    }
}

/// One voice landing in a session, the one law for an envelope the
/// precompile verified and for a key inside a BLS aggregate. Who is counted
/// is who stood there when the vote opened: the hexagon seats of `voter`'s
/// row, if the session's roster holds that row with that identity and it is
/// not yet counted, at the seats × weight the roster recorded (a holder who
/// arrived or received seats after the opening adds nothing in this
/// session); and once each the chairs `voter` holds now, unexpired now,
/// that the roster lit and nobody has touched since (`speakers`), and that
/// have not spoken. A session opened before the roster counts as it did.
/// True when anything landed.
pub(super) fn land(
    s: &mut Session,
    session: &mut AccountView,
    registry: &mut AccountView,
    voter: &[u8; 32],
    choice: u8,
    now: u64,
    speakers: u16,
) -> Result<bool, ProgramError> {
    let mut landed = false;
    if let Some(idx) = Registry::find_holder(registry, crate::HOLDER_KEY, voter)? {
        if !s.counted(idx) {
            let mut h = Registry::holder(registry, idx)?;
            if let Some(w) = s.admit_vote(session, registry, idx)? {
                match choice {
                    crate::VOTE_PRO => s.hex_pro = s.hex_pro.checked_add(w).ok_or(PoleisError::MathOverflow)?,
                    crate::VOTE_CONTRA => s.hex_contra = s.hex_contra.checked_add(w).ok_or(PoleisError::MathOverflow)?,
                    _ => s.hex_abst = s.hex_abst.checked_add(w).ok_or(PoleisError::MathOverflow)?,
                }
                s.mark(idx);
                s.voice_holder(idx, choice);
                h.last_active = now;
                Registry::set_holder(registry, idx, &h)?;
                landed = true;
            }
        }
    }
    let chairs = Registry::chairs_of(registry, voter, now)? & speakers & !s.chair_mask;
    if chairs != 0 {
        let c = chairs.count_ones() as u8;
        match choice {
            crate::VOTE_PRO => s.pent_pro += c,
            crate::VOTE_CONTRA => s.pent_contra += c,
            _ => s.pent_abst += c,
        }
        s.chair_mask |= chairs;
        for k in 0..crate::PENTAGONS {
            if chairs & (1 << k) != 0 {
                s.voice_chair(k, choice);
            }
        }
        landed = true;
    }
    if landed {
        s.votes += 1;
    }
    Ok(landed)
}

/// data: [hash:32][unit:8][n:1][n × (voter:32, choice:1)]
/// accounts: [crank (s,w), registry (w), session (w), instructions sysvar, treasury (w)]
/// A batch of votes lands. The instruction before this one in the same
/// transaction must be the Ed25519 precompile verifying every envelope.
/// Chamber is derived from the registry, never chosen: seats count in the
/// hexagon chamber at seats × weight, live chairs count once each in the
/// pentagon chamber, a key holding both counts once in each, and a holder
/// already counted is skipped.
pub fn attest(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, registry, session, sysvar, treasury] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if data.len() < 41 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let d = Data(data);
    let hash = d.h32(0)?;
    let unit_id = d.u64(32)?;
    let n = data[40] as usize;
    if n == 0 || n > 16 || data.len() != 41 + n * 33 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let votes = &data[41..];
    expect_signer(crank)?;
    expect_writable(registry)?;
    expect_writable(session)?;

    let mut s = read_session(session, &hash, unit_id)?;
    if s.terminal() {
        return Err(PoleisError::SessionClosed.into());
    }
    let now = Clock::get()?.slot;
    if now > s.deadline {
        return Err(PoleisError::SessionClosed.into());
    }
    super::read_registry(registry, unit_id)?;

    if sysvar.address() != &INSTRUCTIONS_ID {
        return Err(ProgramError::UnsupportedSysvar);
    }
    let instructions = Instructions::try_from(&*sysvar)?;
    let prev = instructions.get_instruction_relative(-1)?;
    if prev.get_program_id() != &crate::ED25519_PROGRAM_ID {
        return Err(PoleisError::BadPrecompile.into());
    }
    check_precompile(prev.get_instruction_data(), &hash, unit_id, votes, n)?;

    let speakers = speakers(session, registry)?;
    let mut any_landed = false;
    for i in 0..n {
        let voter: [u8; 32] = votes[i * 33..i * 33 + 32].try_into().unwrap();
        let choice = votes[i * 33 + 32];
        if choice > crate::VOTE_CONTRA {
            return Err(ProgramError::InvalidInstructionData);
        }
        if land(&mut s, session, registry, &voter, choice, now, speakers)? {
            any_landed = true;
        }
    }
    s.write(session)?;
    // the crank's tip, for a batch that landed at least one voice
    if any_landed {
        super::tip(registry, treasury, crank, unit_id, &hash, now)?;
    }
    Ok(())
}

/// A wallet-signed NFT vote uses the same session and persistent holder bit as
/// a legacy vote. Reselling the NFT never creates a fresh vote in this session.
/// data [hash:32][unit:8][choice:1]; accounts [owner(s), seat, mint, token,
/// registry(w), session(w)]. The signer is the live owner, never a delegate.
pub fn attest_nft(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [owner, seat, mint, token, registry, session] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(41)?;
    let hash = d.h32(0)?;
    let unit = d.u64(32)?;
    let choice = data[40];
    if choice > crate::VOTE_CONTRA { return Err(ProgramError::InvalidInstructionData); }
    expect_writable(registry)?;
    expect_writable(session)?;
    let (receipt, index) = super::nft::current_owner(owner, seat, mint, token, registry)?;
    if receipt.unit != unit { return Err(PoleisError::UnitMismatch.into()); }
    let mut s = read_session(session, &hash, unit)?;
    let now = Clock::get()?.slot;
    if s.terminal() || now > s.deadline { return Err(PoleisError::SessionClosed.into()); }
    if s.counted(index) { return Ok(()); }
    let mut h = Registry::holder(registry, index)?;
    let weight = s.admit_vote(session, registry, index)?.ok_or(PoleisError::NotHolder)?;
    match choice {
        crate::VOTE_PRO => s.hex_pro = s.hex_pro.checked_add(weight).ok_or(PoleisError::MathOverflow)?,
        crate::VOTE_CONTRA => s.hex_contra = s.hex_contra.checked_add(weight).ok_or(PoleisError::MathOverflow)?,
        _ => s.hex_abst = s.hex_abst.checked_add(weight).ok_or(PoleisError::MathOverflow)?,
    }
    s.mark(index);
    s.voice_holder(index, choice);
    s.votes = s.votes.checked_add(1).ok_or(PoleisError::MathOverflow)?;
    h.last_active = now;
    Registry::set_holder(registry, index, &h)?;
    s.write(session)
}

/// data: [hash:32][parent:8][child:8]
/// accounts: [crank (s), parent registry, parent session (w), child session]
/// A unit bound in a chair speaks through its own seal, once in each lit
/// chair it holds: PASSED casts PRO, REJECTED casts CONTRA, DEADLOCK casts
/// ABSTENTIO. Who is counted is who stood there when the vote opened: in a
/// parent session that carries its roster, the child speaks only in the
/// chairs it holds now that the roster lit and nobody has touched since;
/// a crew seated, reseated or lapsed after the opening speaks in none
/// (`InvalidHolder`) and waits for the next decision. An EXPIRED child never spoke: its chairs are marked heard
/// from, so the parent may seal, and count as absent, so a tree nobody
/// voted in expires whole. A child that never convened is heard the same
/// way once its parent's window has closed: it had that whole window to open
/// its vote. Permissionless.
pub fn mandate(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crank, registry, parent_session, child_session] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(48)?;
    let hash = d.h32(0)?;
    let parent = d.u64(32)?;
    let child = d.u64(40)?;
    expect_signer(crank)?;
    expect_writable(parent_session)?;

    let mut ps = read_session(parent_session, &hash, parent)?;
    if ps.terminal() {
        return Err(PoleisError::SessionClosed.into());
    }
    super::read_registry(registry, parent)?;
    let now = Clock::get()?.slot;
    let chairs = Registry::chairs_of_unit(registry, child, now)? & speakers(parent_session, registry)?;
    if chairs == 0 {
        return Err(PoleisError::InvalidHolder.into());
    }
    let unheard = chairs & !ps.chair_mask;
    if unheard == 0 {
        return Err(PoleisError::AlreadyMandated.into());
    }
    let verdict = if child_session.owned_by(&crate::ID) {
        let cs = read_session(child_session, &hash, child)?;
        if !cs.terminal() {
            return Err(PoleisError::ChildMustSpeak.into());
        }
        cs.state
    } else {
        // never convened: the child's own ballot box is empty, and the window
        // it had to open it has closed
        find_and_expect_pda(child_session, &[b"session", &hash, &child.to_le_bytes()])?;
        if child_session.data_len() != 0 {
            return Err(PoleisError::InvalidAccount.into());
        }
        if now <= ps.deadline {
            return Err(PoleisError::ChildMustSpeak.into());
        }
        crate::STATE_EXPIRED
    };
    let n = unheard.count_ones() as u8;
    let choice = match verdict {
        crate::STATE_PASSED => {
            ps.pent_pro += n;
            ps.votes += 1;
            crate::VOTE_PRO
        }
        crate::STATE_REJECTED => {
            ps.pent_contra += n;
            ps.votes += 1;
            crate::VOTE_CONTRA
        }
        crate::STATE_DEADLOCK => {
            ps.pent_abst += n;
            ps.votes += 1;
            crate::VOTE_ABSTENTIO
        }
        _ => crate::VOTE_ABSTENTIO, // EXPIRED or never convened: heard from, absent
    };
    ps.chair_mask |= unheard;
    for k in 0..crate::PENTAGONS {
        if unheard & (1 << k) != 0 {
            ps.voice_chair(k, choice);
        }
    }
    ps.write(parent_session)
}

/// data: [hash:32][unit:8] · accounts: [crank (s,w), registry (w), session (w), proposal, treasury (w)]
/// After the deadline, or before it once the outcome is decided. Who is
/// counted is who stood there when the vote opened, read from the session's
/// roster. Refuses while any crew in a lit chair that the roster lit and
/// nobody has touched since has not spoken (`mandate`), whatever opened its
/// session: the leaves seal first, the root last; a crew recalled, replaced
/// or lapsed since the opening is absent, not waited for. Absent chairs
/// and absent hexagons are not PRO. Both seals: more than half of the chairs
/// lit at the opening (seven when all twelve were; twelve is the budget, never
/// the roster), so a term that ends inside the window lowers no bar; and PRO
/// weight above half of the opening's active weight (with every holder of the
/// roster who woke to vote). Both is PASSED, one is DEADLOCK, neither is
/// REJECTED; a session nobody voted in is EXPIRED. Where the proposal asked
/// for the SUM, the origin's own split is settled by the sum of its two
/// halves (`sum_passes`): PASSED when they reach 51 % with neither under a
/// third, else it stands as DEADLOCK; a crew's session seals by the AND. The
/// wait for crews, the early seal and the deadline are the AND's. The grant
/// is the one `propose` read in the origin's law: a law changed while the
/// vote is open changes nothing in it. Under consent a seal
/// carries the AND of the crews of the opening: both seals over a crew of
/// the opening that did not speak PRO, or never spoke, is DEADLOCK, the tree
/// split, whatever became of its chair since. A session opened before the
/// roster reads the chairs and the floor at the seal, as it always did.
/// Where the proposal asked for the market (`CLASS_MARKET`) or is a plan
/// (`CLASS_PLAN`), the origin's own session reads two accounts more: the
/// origin's market (its law) and the proposal's road, at the address the
/// law and the hash name under the law's roads program. Under
/// the market, a split where the chairs passed and the floor did not is
/// PASSED when the road passed; a split where the chairs refused stands.
/// Under a plan the floor's ballot is not counted: PASSED when the chairs
/// passed and the road passed, DEADLOCK when one of the two did, REJECTED
/// when neither, and the floor's weight never seals it early. THE ROAD
/// PASSED: the law is the opening's (written before the opening's slot), the
/// road is laid by the law's program, its window lies inside the vote, it
/// was decided PASSED, and enough of its board as it stood at the lay stood
/// in it (`stood_enough`). A road still open where its verdict could change
/// the seal holds it (`MarketOpen`); anything that cannot be read says
/// nothing. A crew's own session seals by the AND, with five accounts.
/// The unit's chain remembers the seal.
/// accounts: [crank (s,w), registry (w), session (w), proposal, treasury (w),
///            then, at the origin of a proposal that asks the market, its market and its road]
pub fn seal(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() < 5 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, word) = accounts.split_at_mut(5);
    let [crank, registry, session, proposal, treasury] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(40)?;
    let hash = d.h32(0)?;
    let unit_id = d.u64(32)?;
    expect_signer(crank)?;
    expect_writable(registry)?;
    expect_writable(session)?;

    let mut s = read_session(session, &hash, unit_id)?;
    if s.terminal() {
        return Err(PoleisError::SessionClosed.into());
    }
    let now = Clock::get()?.slot;
    let p = read_proposal(proposal, &hash)?;
    // the market speaks only in the origin's own session; every other seal is the AND's, five accounts
    let asks = (p.market() || p.plan()) && unit_id == p.origin;
    let plan = asks && p.plan();
    if word.len() != if asks { 2 } else { 0 } {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let mut head = super::read_registry(registry, unit_id)?;

    // who is counted is who stood there when the vote opened. A session
    // that carries its roster waits for the crews in lit chairs now that the
    // roster lit and nobody touched since (a crew recalled, replaced or
    // lapsed since is absent, not waited for), sets the chairs' bar by the
    // chairs lit at the opening, and under consent ANDs the crews of the
    // opening, whatever became of their chairs. A session opened before
    // the roster reads the chairs at the seal, as it always did.
    let (waited, lit, crews) = match Session::roster(&session.try_borrow()?)? {
        Some((lit, crews, opened)) => {
            let stood = lit & Registry::unchanged_since(registry, opened)?;
            (Registry::crew_chairs(registry, now)? & stood, lit.count_ones() as u8, crews)
        }
        None => {
            let crews = Registry::crew_chairs(registry, now)?;
            (crews, Registry::lit_chairs(registry, now)?, crews)
        }
    };
    // every crew in a lit chair speaks first, through its own seal
    if waited & !s.chair_mask != 0 {
        return Err(PoleisError::ChildMustSpeak.into());
    }

    let pent_pass = s.pent_pro >= crate::chair_quorum(lit);
    // the fast path: once the chairs' majority said PRO and the PRO weight is more
    // than half of the whole floor, no vote still to come can change the
    // outcome, so the seal need not wait for the window. Votes cannot be
    // withdrawn and a withheld envelope could only have added to PRO.
    // A session with a body (the roster, or an older NFT session's electorate)
    // retains its original electorate through moves, burns, resale and
    // later admissions. The fast path uses ALL original weight, including
    // dormant members who could still wake before the deadline.
    let electorate = Session::electorate_weights(&session.try_borrow()?)?;
    let (total, active, empty_floor) = if let Some((total, active)) = electorate {
        (total, active, total == 0)
    } else {
        (Registry::total_weight(registry)?, Registry::active_weight(registry, now)?, head.total_seats == 0)
    };
    // a plan has no ballot of the floor to decide by: its floor's weight never seals it early
    let decided = pent_pass && (empty_floor || (!plan && (s.hex_pro as u128) * 2 > total as u128));
    if now <= s.deadline && !decided {
        return Err(PoleisError::SessionOpen.into());
    }

    // A floor present in the snapshot cannot become chair-only by burning
    // every live NFT. Bare sessions keep the existing activity-window rule.
    let hex_pass = decided || empty_floor || (active > 0 && (s.hex_pro as u128) * 2 > active as u128);
    s.state = if s.votes == 0 {
        crate::STATE_EXPIRED
    } else if empty_floor {
        // camera hexagona nulla: the chairs' verdict is the verdict, and a
        // chamber with no floor can never be split against itself
        if pent_pass {
            crate::STATE_PASSED
        } else {
            crate::STATE_REJECTED
        }
    } else if pent_pass && hex_pass {
        crate::STATE_PASSED
    } else if pent_pass || hex_pass {
        crate::STATE_DEADLOCK
    } else {
        crate::STATE_REJECTED
    };
    // a split of the origin's chambers, where the proposal asked for the SUM
    // and the origin's law granted it at the opening, is settled by the sum
    // of the two halves; a crew's own session seals by the AND
    if s.state == crate::STATE_DEADLOCK && p.sum() && unit_id == p.origin && !empty_floor && crate::sum_passes(s.pent_pro, lit, s.hex_pro, active) {
        s.state = crate::STATE_PASSED;
    }
    // the market's word, at the origin of a proposal that asks it. Where it
    // could change this seal an open road holds it: under a plan with a floor
    // that voted, always; under the market, a split the chairs passed
    if asks {
        let split = s.state == crate::STATE_DEADLOCK && pent_pass && !empty_floor;
        let weighs = !empty_floor && s.votes != 0;
        let road_passed = road_passed(word, &p, &hash, s.deadline, now, if plan { weighs } else { split })?;
        if plan && weighs {
            s.state = match (pent_pass, road_passed) {
                (true, true) => crate::STATE_PASSED,
                (false, false) => crate::STATE_REJECTED,
                _ => crate::STATE_DEADLOCK,
            };
        } else if !plan && split && road_passed {
            // the chairs keep their veto: only a split they passed goes to the market
            s.state = crate::STATE_PASSED;
        }
    }
    // under consent the seal carries its crews' AND: a crew that did not pass
    // leaves this chamber split whatever it said itself, so the origin's seal
    // alone says whether the whole tree passed
    let crews_passed = (0..crate::PENTAGONS).all(|k| crews & (1 << k) == 0 || s.chair_voice_of(k) == crate::VOTE_PRO + 1);
    if s.state == crate::STATE_PASSED && p.consent() && !crews_passed {
        s.state = crate::STATE_DEADLOCK;
    }
    s.active_weight = active;
    s.write(session)?;

    head.chain_sig = sha256v(&[&head.chain_sig, &hash, &[s.state]]);
    Registry::set_head(registry, &head)?;
    receipt(crate::RCPT_SEAL, unit_id, &hash, &ZERO, &ZERO, 0, s.state);
    super::tip(registry, treasury, crank, unit_id, &hash, now)
}

/// THE ROAD PASSED, for the origin's seal of a proposal that asks the market. `word`: [market, road]. The market must be the origin's
/// own (`InvalidPda`); a law that cannot be read, is another unit's, was written in or after the proposal's opening slot, or no longer
/// names its class for its mode says nothing (the grant is the opening's). The road must be the address the law's program, its board
/// and the proposal's hash name (`InvalidPda`: a crank cannot hide a road). A road never laid, not readable as the roads program's
/// version, or whose window is not inside the vote (`deadline`: the origin session's) says nothing; an open one holds the seal
/// (`MarketOpen`) only where `matters`. Since a road that counts closes by the session's deadline, `MarketOpen` can hold a seal only
/// between that deadline and the road's `decide`, which anyone may send. A supporter must have the road laid, inside the vote's window,
/// for the market to speak: after the deadline anyone may seal, and a road not laid by then says nothing for good.
/// A ROAD THAT CANNOT COUNT NEVER HOLDS A SEAL, AND A ROAD NOBODY STOOD IN SAYS NOTHING: a decided road counts only with deposits in it
/// beside the bar (nobody traded a road nobody deposited in, whatever its verdict and whatever the bar); an open road holds the seal where
/// it matters unless it can no longer count: the clock is at or past its close (the roads program takes no deposit at or after the close:
/// `deposit`'s `RoadClosed`, held at the close's own slot by its `tests/refusals.rs` `deposit_refusals`) and what stood in it is nothing or
/// under the bar. Then it says nothing.
fn road_passed(word: &[AccountView], p: &Proposal, hash: &[u8; 32], deadline: u64, now: u64, matters: bool) -> Result<bool, ProgramError> {
    use crate::roads::{road_address, road_word, RoadWord};
    let [market, road] = word else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    find_and_expect_pda(market, &[b"market", &p.origin.to_le_bytes()])?;
    let Ok(law) = Market::read(market) else {
        return Ok(false);
    };
    let mask = if p.plan() { law.plan } else { law.split };
    // a law written in the opening's own slot, after the proposal, is not the law it was opened under
    if law.unit_id != p.origin || law.since >= p.opened || mask & crate::market_bit(p.class_of()) == 0 {
        return Ok(false);
    }
    let program = Address::new_from_array(law.program);
    if road.address() != &road_address(&program, law.board, hash) {
        return Err(PoleisError::InvalidPda.into());
    }
    Ok(match road_word(road, &program, law.board, hash, p.opened, deadline)? {
        RoadWord::Silent => false,
        RoadWord::Open { deposits, credited_at_lay, closes } => {
            let can_count = now < closes || (deposits > 0 && crate::stood_enough(deposits, credited_at_lay, law.min_stood_bp));
            if matters && can_count {
                return Err(PoleisError::MarketOpen.into());
            }
            false
        }
        RoadWord::Decided { passed, deposits, credited_at_lay } => passed && deposits > 0 && crate::stood_enough(deposits, credited_at_lay, law.min_stood_bp),
    })
}

/// data: [hash:32]
/// accounts: [crank (s,w), config, proposal (w), then count sessions (impacted
///            order), then for CODEX count registries (impacted order), then
///            the payload's own accounts]
/// Permissionless. The payload executed is the one the proposal stores,
/// never one the crank supplies. Every impacted session must be PASSED;
/// CODEX must be closed under bound children; every treasury outflow is
/// charged to the paying unit's plan.
pub fn execute(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    Data(data).expect_len(32)?;
    let hash: [u8; 32] = data[..32].try_into().unwrap();

    if accounts.len() < 3 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, tail) = accounts.split_at_mut(3);
    let [crank, config, proposal] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(crank)?;
    expect_writable(proposal)?;
    super::expect_any_council(config)?;
    let mut p = read_proposal(proposal, &hash)?;
    if p.executed {
        return Err(PoleisError::AlreadyExecuted.into());
    }
    let mut buf = [0u8; crate::MAX_PAYLOAD];
    let plen = p.payload_len as usize;
    if !(2..=crate::MAX_PAYLOAD).contains(&plen) {
        return Err(PoleisError::BadPayload.into());
    }
    Proposal::payload(proposal, plen, &mut buf)?;
    let payload = &buf[..plen];
    let class = payload[0];
    let kind = payload[1];
    let fields = &payload[2..];
    if p.class_of() != class {
        return Err(PoleisError::ClassMismatch.into());
    }
    let count = p.count as usize;
    if tail.len() < count {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (sessions, rest) = tail.split_at_mut(count);
    // the origin's seal is the tree's verdict: a crew spoke through its chair.
    // Under consent every chamber's seal must be PASSED as well, and the
    // origin's seal carried the AND of every crew that convened its own.
    let mut heard = [0u16; crate::MAX_IMPACTED];
    for (i, (session, unit)) in sessions.iter().zip(p.impacted.iter()).enumerate() {
        let s = read_session(session, &hash, *unit)?;
        if s.state != crate::STATE_PASSED && (*unit == p.origin || p.consent()) {
            return Err(PoleisError::NotPassed.into());
        }
        heard[i] = s.chair_mask;
    }
    let now = Clock::get()?.slot;
    let rest = if class == crate::CLASS_CODEX {
        // closure: every crew in a lit chair of an impacted unit is impacted,
        // or spoke in that chair: heard there, and seated there before the
        // question was asked, so the voice the chair carries is its own. Every
        // seal waited for its crews already; this is the belt for the one act
        // that rewrites the code, against a crew seated mid-session
        if rest.len() < count {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let (registries, rest) = rest.split_at_mut(count);
        for (i, (registry, unit)) in registries.iter().zip(p.impacted.iter()).enumerate() {
            super::read_registry(registry, *unit)?;
            let crews = Registry::crew_chairs(registry, now)?;
            for k in 0..crate::PENTAGONS {
                if crews & (1 << k) == 0 {
                    continue;
                }
                let pent = Registry::pentagon(registry, k)?;
                let spoke = heard[i] & (1 << k) != 0 && pent.seated_slot <= p.opened;
                if !spoke && !p.touches(holder_unit(&pent.holder)) {
                    return Err(PoleisError::ClosureBroken.into());
                }
            }
        }
        rest
    } else {
        rest
    };

    let origin = p.origin;
    let f = Data(fields);
    match (class, kind) {
        (crate::CLASS_TREASURY, crate::TREASURY_TRANSFER) => {
            f.expect_len(16)?;
            let dest_unit = f.u64(0)?;
            let amount = f.u64(8)?;
            let [registry, source, dest, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if amount == 0 || dest_unit == origin {
                return Err(PoleisError::InvalidAmount.into());
            }
            super::read_registry(registry, origin)?;
            treasury_of(source, origin)?;
            treasury_of(dest, dest_unit)?;
            super::charge_plan(registry, amount, now)?;
            move_lamports(source, dest, amount)?;
            Treasury::add_out(source, amount)?;
            Treasury::add_in(dest, amount)?;
            receipt(crate::RCPT_PAY, origin, &hash, &unit_holder(dest_unit), &ZERO, amount, crate::HOLDER_UNIT);
        }
        (crate::CLASS_TREASURY, crate::TREASURY_BUDGET) => {
            f.expect_len(40)?;
            let address = f.h32(0)?;
            let amount = f.u64(32)?;
            let [registry, source, dest, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if amount == 0 || dest.address().as_array() != &address {
                return Err(PoleisError::InvalidAccount.into());
            }
            super::read_registry(registry, origin)?;
            treasury_of(source, origin)?;
            super::charge_plan(registry, amount, now)?;
            move_lamports(source, dest, amount)?;
            Treasury::add_out(source, amount)?;
            receipt(crate::RCPT_PAY, origin, &hash, &address, &ZERO, amount, crate::HOLDER_KEY);
        }
        (crate::CLASS_TREASURY, crate::TREASURY_DIVIDEND) | (crate::CLASS_TREASURY, crate::TREASURY_DISSOLVE) => {
            let dissolve = kind == crate::TREASURY_DISSOLVE;
            let amount = if dissolve {
                f.expect_len(0)?;
                0
            } else {
                f.expect_len(8)?;
                f.u64(0)?
            };
            let [registry, treasury, distribution, _system, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let rh = super::read_registry(registry, origin)?;
            let floor = Rent::get()?.try_minimum_balance(treasury.data_len())?;
            // a dissolved unit sweeps again what came in after its dissolution
            // (a refund, a late contract), outside the plan; zero means all of it
            let sweep = dissolve || rh.frozen;
            let amount = if dissolve || (rh.frozen && amount == 0) { treasury.lamports().saturating_sub(floor) } else { amount };
            if amount == 0 {
                return Err(PoleisError::InvalidAmount.into());
            }
            if !sweep {
                super::charge_plan(registry, amount, now)?;
            }
            open_distribution(crank, registry, treasury, distribution, &hash, origin, amount, now, None)?;
            if dissolve {
                let mut h = Registry::head(registry)?;
                h.frozen = true;
                Registry::set_head(registry, &h)?;
            }
        }
        (crate::CLASS_TREASURY, crate::TREASURY_TOKEN_DIVIDEND) => {
            f.expect_len(40)?;
            let mint = f.h32(0)?;
            let amount = f.u64(32)?;
            let [registry, treasury, vault, distribution, mint_acc, escrow, token_program, _system, token_plan, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let h = super::read_registry(registry, origin)?;
            if mint_acc.address().as_array() != &mint {
                return Err(PoleisError::InvalidAccount.into());
            }
            expect_vault(vault, origin, &mint)?;
            // a dissolved unit sweeps a vault whole, outside the plan, so
            // nothing is stranded; a living one pays a dividend under it
            let amount = if h.frozen {
                if amount == 0 {
                    token_balance(vault)?
                } else {
                    amount
                }
            } else {
                super::charge_token_plan(registry, &mint, amount, now, Some(token_plan))?;
                amount
            };
            open_distribution(crank, registry, treasury, distribution, &hash, origin, amount, now, Some((vault, escrow, mint_acc, token_program)))?;
        }
        (crate::CLASS_TREASURY, crate::TREASURY_ALLOWANCE) => {
            let [registry, allowance, _system, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            open_allowance(crank, registry, allowance, &hash, origin, fields, now)?;
        }
        (crate::CLASS_TREASURY, crate::TREASURY_REVOKE) => {
            let [registry, allowance, payer, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            revoke_allowance(registry, allowance, payer, origin, fields)?;
        }
        (crate::CLASS_TREASURY, crate::TREASURY_CONTRACT) => {
            open_contract(crank, rest, &hash, origin, fields, now)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_DIVIDEND_POLICY) => {
            let [registry, treasury, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            set_dividend_policy(registry, treasury, origin, fields, now)?;
        }
        (crate::CLASS_SIGNAL, crate::SIGNAL_RULE) => {
            rule_contract(rest, origin, fields)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_UPGRADE | crate::CODEX_UPGRADE_PROGRAM) => {
            // only the root, over the whole tree (closure), changes the code: Poleis's own by kind 6, [buffer:32][spill:32]; any program the
            // seal holds by kind 14, which names it, [buffer:32][spill:32][program:32] (THE SEAL UPGRADES EVERY PROGRAM). Whoever carries it
            // out reaches the named program, its own data, the named buffer and spill, and nothing else.
            let named = kind == crate::CODEX_UPGRADE_PROGRAM;
            f.expect_len(if named { 96 } else { 64 })?;
            let buffer_key = f.h32(0)?;
            let spill_key = f.h32(32)?;
            let program_key = if named { f.h32(64)? } else { *crate::ID.as_array() };
            if origin != crate::ROOT_UNIT {
                return Err(PoleisError::Unauthorized.into());
            }
            let [registry, programdata, program, buffer, spill, rent, clock, authority, loader, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            if program.address().as_array() != &program_key || buffer.address().as_array() != &buffer_key || spill.address().as_array() != &spill_key {
                return Err(PoleisError::InvalidAccount.into());
            }
            super::upgrade::expect_programdata_of(programdata, &program_key)?;
            super::upgrade::upgrade(programdata, program, buffer, spill, rent, clock, authority, loader)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_UPGRADE_AUTHORITY) => {
            // THE SEAL NAMES ITS PROGRAM: [new:32][program:32]. Every program handed to the seal answers to the one upgrade PDA, so the
            // act names the program it is for and its hash covers the name: the only program data it reaches is that program's own,
            // whoever carries it out. The shape before it, [new:32] alone, is no act.
            f.expect_len(64)?;
            let new_key = f.h32(0)?;
            let program_key = f.h32(32)?;
            if origin != crate::ROOT_UNIT {
                return Err(PoleisError::Unauthorized.into());
            }
            let [registry, programdata, authority, loader, more @ ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            super::upgrade::expect_programdata_of(programdata, &program_key)?;
            let new_acc = if new_key == ZERO {
                None
            } else {
                let n = more.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
                if n.address().as_array() != &new_key {
                    return Err(PoleisError::InvalidAccount.into());
                }
                Some(n)
            };
            super::upgrade::set_authority(programdata, authority, loader, new_acc)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_CLOSE_BUFFER) => {
            // THE SEAL CLOSES A BUFFER: [buffer:32][recipient:32]. An upgrade's buffer is the upgrade PDA's before its vote; one never
            // carried out is closed here, by the root over the whole tree, its lamports to the recipient the act names. The account must
            // be the named one and a buffer (`close_buffer`): no vote closes a program through this.
            f.expect_len(64)?;
            let buffer_key = f.h32(0)?;
            let recipient_key = f.h32(32)?;
            if origin != crate::ROOT_UNIT {
                return Err(PoleisError::Unauthorized.into());
            }
            let [registry, buffer, recipient, authority, loader, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            if buffer.address().as_array() != &buffer_key || recipient.address().as_array() != &recipient_key {
                return Err(PoleisError::InvalidAccount.into());
            }
            super::upgrade::close_buffer(buffer, recipient, authority, loader)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_ALLOW_MINT) => {
            let [registry, token_plan, payer, _system, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            allow_mint(crank, registry, token_plan, payer, origin, fields, now)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_CRANK_TIP) => {
            f.expect_len(8)?;
            let [registry, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let mut h = super::read_registry(registry, origin)?;
            h.crank_tip = f.u64(0)?;
            Registry::set_head(registry, &h)?;
        }
        (crate::CLASS_SEAT, crate::SEAT_ATTACH) => {
            f.expect_len(50)?;
            let chair = fields[0] as usize;
            let holder = f.h32(1)?;
            let hkind = fields[33];
            let term = f.u64(34)?;
            let retainer = f.u64(42)?;
            if chair >= crate::PENTAGONS || holder == [0u8; 32] {
                return Err(PoleisError::InvalidHolder.into());
            }
            if hkind != crate::HOLDER_KEY && hkind != crate::HOLDER_UNIT {
                return Err(PoleisError::InvalidHolder.into());
            }
            // a unit is its id and nothing more: a holder no unit answers to is
            // a crew no seal can ever hear, and its polis would wait forever
            if hkind == crate::HOLDER_UNIT && holder != unit_holder(holder_unit(&holder)) {
                return Err(PoleisError::InvalidHolder.into());
            }
            if term > crate::MAX_TERM_SLOTS {
                return Err(PoleisError::InvalidWindow.into());
            }
            let (registry, more) = rest.split_first_mut().ok_or(ProgramError::NotEnoughAccountKeys)?;
            let head = super::read_registry(registry, origin)?;
            if Registry::pentagon(registry, chair)?.kind != crate::HOLDER_EMPTY {
                return Err(PoleisError::ChairTaken.into());
            }
            // a unit sits only below the polis that seats it, in its own tree:
            // its registry exists, names the same root and stands deeper. Levels
            // never change and rise along every seat, so no seat closes a cycle
            // (two seals each waiting for the other's, forever), and every chain
            // of crews ends within MAX_DEPTH
            let more = if hkind == crate::HOLDER_UNIT {
                let (crew, more) = more.split_first_mut().ok_or(ProgramError::NotEnoughAccountKeys)?;
                let c = super::read_registry(crew, holder_unit(&holder))?;
                if c.root != head.root || c.level <= head.level {
                    return Err(PoleisError::NotDescendant.into());
                }
                more
            } else {
                more
            };
            let term_end = if term == 0 { 0 } else { now.checked_add(term).ok_or(PoleisError::MathOverflow)? };
            Registry::set_pentagon(registry, chair, &Pentagon { holder, kind: hkind, seated_slot: now, term_end })?;
            if retainer > 0 {
                if hkind != crate::HOLDER_UNIT {
                    return Err(PoleisError::InvalidHolder.into());
                }
                let [source, dest, ..] = more else {
                    return Err(ProgramError::NotEnoughAccountKeys);
                };
                treasury_of(source, origin)?;
                treasury_of(dest, holder_unit(&holder))?;
                super::charge_plan(registry, retainer, now)?;
                move_lamports(source, dest, retainer)?;
                Treasury::add_out(source, retainer)?;
                Treasury::add_in(dest, retainer)?;
                receipt(crate::RCPT_PAY, origin, &hash, &holder, &ZERO, retainer, crate::HOLDER_UNIT);
            }
        }
        (crate::CLASS_SEAT, crate::SEAT_RECALL) => {
            f.expect_len(1)?;
            let chair = fields[0] as usize;
            let [registry, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            if chair >= crate::PENTAGONS || Registry::pentagon(registry, chair)?.kind == crate::HOLDER_EMPTY {
                return Err(PoleisError::ChairEmpty.into());
            }
            Registry::set_pentagon(
                registry,
                chair,
                &Pentagon { holder: [0u8; 32], kind: crate::HOLDER_EMPTY, seated_slot: now, term_end: 0 },
            )?;
        }
        (crate::CLASS_SEAT, crate::SEAT_JET) => {
            f.expect_len(33)?;
            let holder = f.h32(0)?;
            let hkind = fields[32];
            if ![crate::HOLDER_KEY, crate::HOLDER_UNIT, crate::HOLDER_NFT].contains(&hkind) {
                return Err(PoleisError::InvalidHolder.into());
            }
            let [registry, treasury, payee, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let head = super::read_registry(registry, origin)?;
            treasury_of(treasury, origin)?;
            let i = Registry::find_holder(registry, hkind, &holder)?.ok_or(PoleisError::NotHolder)?;
            let h = Registry::holder(registry, i)?;
            if head.last_clearing == 0 {
                return Err(PoleisError::InvalidAmount.into());
            }
            let price = h.seats.checked_mul(head.last_clearing).ok_or(PoleisError::MathOverflow)?;
            if hkind == crate::HOLDER_KEY || hkind == crate::HOLDER_NFT {
                if payee.address().as_array() != &holder {
                    return Err(PoleisError::InvalidAccount.into());
                }
            } else {
                treasury_of(payee, holder_unit(&holder))?;
            }
            if hkind == crate::HOLDER_NFT {
                let nft = crate::nft_seat_state::NftSeat::read(payee)?;
                if nft.unit != origin || h.seats != 1 {
                    return Err(PoleisError::InvalidHolder.into());
                }
            }
            super::charge_plan(registry, price, now)?;
            move_lamports(treasury, payee, price)?;
            Treasury::add_out(treasury, price)?;
            if hkind == crate::HOLDER_UNIT {
                Treasury::add_in(payee, price)?;
            }
            Registry::remove_holder(registry, i)?;
            receipt(crate::RCPT_PAY, origin, &hash, &holder, &ZERO, price, hkind);
        }
        (crate::CLASS_CODEX, crate::CODEX_PLAN) => {
            f.expect_len(24)?;
            let [registry, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let mut h = super::read_registry(registry, origin)?;
            h.plan_cap = f.u64(0)?;
            h.plan_period = f.u64(8)?;
            h.token_plan_cap = f.u64(16)?;
            h.plan_spent = 0;
            h.token_plan_spent = 0;
            h.plan_start = now;
            Registry::set_head(registry, &h)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_NAME) => {
            let (name, used) = parse_text(fields, crate::MAX_NAME)?;
            let (desc, used2) = parse_text(&fields[used..], crate::MAX_DESC)?;
            if fields.len() != used + used2 {
                return Err(PoleisError::BadPayload.into());
            }
            let [registry, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            Registry::set_name(registry, name, desc)?;
        }
        (crate::CLASS_TREASURY, crate::TREASURY_TOKEN_TRANSFER) => {
            f.expect_len(48)?;
            let mint = f.h32(0)?;
            let dest_unit = f.u64(32)?;
            let amount = f.u64(40)?;
            let [registry, treasury, source, dest, token_program, token_plan, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if dest_unit == origin {
                return Err(PoleisError::InvalidAmount.into());
            }
            super::read_registry(registry, origin)?;
            let t = treasury_of(treasury, origin)?;
            expect_vault(source, origin, &mint)?;
            expect_vault(dest, dest_unit, &mint)?;
            super::charge_token_plan(registry, &mint, amount, now, Some(token_plan))?;
            transfer_from_vault(origin, treasury, t.bump, source, dest, token_program, amount)?;
            receipt(crate::RCPT_PAY, origin, &hash, &unit_holder(dest_unit), &mint, amount, crate::HOLDER_UNIT);
        }
        (crate::CLASS_TREASURY, crate::TREASURY_TOKEN_BUDGET) => {
            f.expect_len(72)?;
            let mint = f.h32(0)?;
            let address = f.h32(32)?;
            let amount = f.u64(64)?;
            let [registry, treasury, source, dest, token_program, token_plan, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if dest.address().as_array() != &address {
                return Err(PoleisError::InvalidAccount.into());
            }
            super::read_registry(registry, origin)?;
            let t = treasury_of(treasury, origin)?;
            expect_vault(source, origin, &mint)?;
            super::charge_token_plan(registry, &mint, amount, now, Some(token_plan))?;
            transfer_from_vault(origin, treasury, t.bump, source, dest, token_program, amount)?;
            receipt(crate::RCPT_PAY, origin, &hash, &address, &mint, amount, crate::HOLDER_KEY);
        }
        (crate::CLASS_ACT, crate::ACT_RAW) => {
            // [program:32][n:1][n × (key:32, flags:1)][data_len:2][data]
            if fields.len() < 35 {
                return Err(PoleisError::BadPayload.into());
            }
            let program = f.h32(0)?;
            let n = fields[32] as usize;
            if n > crate::MAX_ACT_ACCOUNTS || fields.len() < 33 + n * 33 + 2 {
                return Err(PoleisError::BadPayload.into());
            }
            let off_len = 33 + n * 33;
            let data_len = u16::from_le_bytes([fields[off_len], fields[off_len + 1]]) as usize;
            if fields.len() != off_len + 2 + data_len {
                return Err(PoleisError::BadPayload.into());
            }
            let act_data = &fields[off_len + 2..];
            if program == *crate::ID.as_array() {
                return Err(PoleisError::ActForbidden.into());
            }
            let (registry, more) = rest.split_first_mut().ok_or(ProgramError::NotEnoughAccountKeys)?;
            super::read_registry(registry, origin)?;
            let (target, targets) = more.split_first_mut().ok_or(ProgramError::NotEnoughAccountKeys)?;
            if target.address().as_array() != &program || targets.len() < n {
                return Err(ProgramError::NotEnoughAccountKeys);
            }
            // the hands: the unit's agent key, never its vault or any account of ours
            let unit_le = origin.to_le_bytes();
            let (agent, agent_bump) = Address::find_program_address(&[b"agent", &unit_le], &crate::ID);
            let mut metas: [InstructionAccount; crate::MAX_ACT_ACCOUNTS] = core::array::from_fn(|_| InstructionAccount::readonly(&agent));
            for i in 0..n {
                let o = 33 + i * 33;
                let flags = fields[o + 32];
                let acc = &targets[i];
                if acc.address().as_array() != &fields[o..o + 32] || acc.owned_by(&crate::ID) {
                    return Err(PoleisError::ActForbidden.into());
                }
                let signer = flags & crate::ACT_SIGNER != 0;
                // the agent's signature is the program's to give; any other signer the sealed payload names must have signed this
                // transaction itself: a named co-signer, so a key can hand a unit what it holds (a stake pool's manager) in one act
                if signer && acc.address() != &agent && !acc.is_signer() {
                    return Err(PoleisError::ActForbidden.into());
                }
                metas[i] = InstructionAccount::new(acc.address(), flags & crate::ACT_WRITABLE != 0, signer);
            }
            let ix = InstructionView { program_id: target.address(), accounts: &metas[..n], data: act_data };
            let bump_ref = [agent_bump];
            let seeds = [Seed::from(&b"agent"[..]), Seed::from(&unit_le[..]), Seed::from(&bump_ref[..])];
            let views: &[AccountView] = &targets[..n];
            invoke_signed_with_bounds::<{ crate::MAX_ACT_ACCOUNTS }, AccountView>(&ix, views, &[Signer::from(&seeds[..])])?;
        }
        (crate::CLASS_CODEX, crate::CODEX_WINDOW) => {
            f.expect_len(16)?;
            let window = f.u64(0)?;
            let codex_window = f.u64(8)?;
            if !(crate::MIN_WINDOW_SLOTS..=crate::MAX_WINDOW_SLOTS).contains(&window)
                || !(window..=crate::MAX_WINDOW_SLOTS).contains(&codex_window)
            {
                return Err(PoleisError::InvalidWindow.into());
            }
            let [registry, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let mut h = super::read_registry(registry, origin)?;
            h.window = window;
            h.codex_window = codex_window;
            Registry::set_head(registry, &h)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_LADDER) => {
            // 32 bytes is a sale; a 33rd byte names the door
            if fields.len() != 32 && fields.len() != 33 {
                return Err(PoleisError::BadPayload.into());
            }
            let mode = if fields.len() == 33 { fields[32] } else { crate::LADDER_SALE };
            let reserve = f.u64(0)?;
            let deposit = f.u64(8)?;
            let commit_len = f.u64(16)?;
            let reveal_len = f.u64(24)?;
            let [registry, sale, _system, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let mut h = super::read_registry(registry, origin)?;
            let next = h.rung as usize + 1;
            if next >= crate::LADDER.len() {
                return Err(PoleisError::NotOnLadder.into());
            }
            let new_seats = crate::ladder_hexagons(next) - crate::ladder_hexagons(h.rung as usize);
            if mode == crate::LADDER_DOOR {
                // the increment goes to the pool, at the price named; `sale` is the pool here
                h.rung = next as u8;
                h.seat_reserve = reserve;
                Registry::set_head(registry, &h)?;
                super::door::open_door(crank, sale, origin, new_seats, reserve)?;
                receipt(crate::RCPT_DOOR, origin, &hash, &ZERO, &ZERO, new_seats, 1);
                p.executed = true;
                return p.write(proposal);
            }
            if mode != crate::LADDER_SALE {
                return Err(PoleisError::BadPayload.into());
            }
            let commit_end = now.checked_add(commit_len).ok_or(PoleisError::MathOverflow)?;
            let reveal_end = commit_end.checked_add(reveal_len).ok_or(PoleisError::MathOverflow)?;
            check_windows(now, commit_end, reveal_end)?;
            let sale_id = h.next_sale_id;
            h.next_sale_id += 1;
            h.rung = next as u8;
            h.seat_reserve = reserve;
            h.seat_deposit = deposit;
            Registry::set_head(registry, &h)?;
            open_sale(
                crank,
                sale,
                SaleHead {
                    unit_id: origin,
                    sale_id,
                    seat_count: new_seats,
                    reserve,
                    deposit,
                    commit_end_slot: commit_end,
                    reveal_end_slot: reveal_end,
                    clearing: 0,
                    overflow_price: 0,
                    settled: false,
                    filled: 0,
                    bump: 0,
                },
            )?;
        }
        (crate::CLASS_CODEX, crate::CODEX_FOUND) => {
            let fp = FoundParams::parse(fields)?;
            let [parent_registry, child_registry, child_treasury, _system, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            found_unit(crank, parent_registry, child_registry, child_treasury, origin, &fp, now)?;
        }
        (crate::CLASS_SIGNAL, crate::SIGNAL_SALE_DEFAULTS) => {
            f.expect_len(16)?;
            let [registry, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            let mut h = super::read_registry(registry, origin)?;
            h.seat_reserve = f.u64(0)?;
            h.seat_deposit = f.u64(8)?;
            Registry::set_head(registry, &h)?;
        }
        (crate::CLASS_SIGNAL, crate::SIGNAL_WEIGHT) => {
            f.expect_len(41)?;
            let holder = f.h32(0)?;
            let hkind = fields[32];
            let weight = f.u64(33)?;
            // past the ceiling, seats × weight could overflow and every later
            // seal of the unit would fail on its weight sum, for ever
            if weight == 0 || weight > crate::MAX_WEIGHT {
                return Err(PoleisError::InvalidAmount.into());
            }
            let [registry, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            let i = Registry::find_holder(registry, hkind, &holder)?.ok_or(PoleisError::NotHolder)?;
            let mut h = Registry::holder(registry, i)?;
            h.weight = weight;
            Registry::set_holder(registry, i, &h)?;
        }
        (crate::CLASS_SIGNAL, crate::SIGNAL_OPEN_AUCTION) => {
            f.expect_len(LotParams::LEN)?;
            let lp = LotParams::parse(&f)?;
            if lp.unit_id != origin {
                return Err(PoleisError::UnitMismatch.into());
            }
            let [treasury, auction, _system, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            open_lot(crank, treasury, auction, &lp, now)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_FUND) => {
            super::fund::codex_fund(crank, rest, origin, fields, now)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_RESOLUTION) => {
            // [sum_kinds:1][small_cap:8]: the unit's own law on how a split is
            // settled. A later act replaces it; an act that names no kind
            // turns the SUM off
            f.expect_len(9)?;
            let sum_kinds = fields[0];
            let small_cap = f.u64(1)?;
            if sum_kinds & !crate::SUM_KINDS != 0 {
                return Err(PoleisError::BadPayload.into());
            }
            let [registry, rule, _system, ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            expect_writable(rule)?;
            let unit_le = origin.to_le_bytes();
            let seeds: &[&[u8]] = &[b"rule", &unit_le];
            let bump = find_and_expect_pda(rule, seeds)?;
            if Rule::read(rule).is_err() {
                create_pda(crank, rule, Rule::SIZE, seeds, bump)?;
            }
            Rule { unit_id: origin, sum_kinds, small_cap, bump }.write(rule)?;
        }
        (crate::CLASS_CODEX, crate::CODEX_MARKET) => {
            // [program:32][board:8][split:1][plan:1][min_stood_bp:2]: the
            // unit's own law on its market. A later act replaces it; both
            // masks 0 turn the market off. `since` is the slot of the act.
            // An act that turns the market on takes the board one account
            // more, and a polis names only a board that is its own: its
            // authority the unit's agent key, its law naming this program
            // and this unit (`roads::board_is_ours`); one that turns it off
            // names none
            f.expect_len(44)?;
            let program = f.h32(0)?;
            let board = f.u64(32)?;
            let split = fields[40];
            let plan = fields[41];
            let min_stood_bp = u16::from_le_bytes([fields[42], fields[43]]);
            if split & !crate::MARKET_CLASSES != 0 || plan & !crate::PLAN_CLASSES != 0 || min_stood_bp as u64 > crate::BPS {
                return Err(PoleisError::BadPayload.into());
            }
            let [registry, market, _system, more @ ..] = rest else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            super::read_registry(registry, origin)?;
            expect_writable(market)?;
            let unit_le = origin.to_le_bytes();
            if split | plan != 0 {
                let board_acc = more.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
                let (agent, _) = Address::find_program_address(&[b"agent", &unit_le], &crate::ID);
                let ours = crate::roads::board_is_ours(board_acc, &Address::new_from_array(program), board, &agent, &crate::ID, origin)?;
                if !ours {
                    return Err(PoleisError::MarketBoardNotOurs.into());
                }
            }
            let seeds: &[&[u8]] = &[b"market", &unit_le];
            let bump = find_and_expect_pda(market, seeds)?;
            if Market::read(market).is_err() {
                create_pda(crank, market, Market::SIZE, seeds, bump)?;
            }
            Market { unit_id: origin, program, board, split, plan, min_stood_bp, since: now, bump }.write(market)?;
        }
        (crate::CLASS_SEAT, crate::SEAT_ADMIT) => {
            super::door::admit(crank, rest, origin, fields, &hash, now)?;
        }
        (crate::CLASS_SEAT, crate::SEAT_REFUSE) => {
            super::door::refuse(rest, origin, fields, &hash)?;
        }
        (crate::CLASS_SIGNAL, crate::SIGNAL_VET_OPTION) => {
            super::fund::vet_option(rest, origin, fields)?;
        }
        (crate::CLASS_SIGNAL, crate::SIGNAL_BACK_OPTION) => {
            super::fund::back_option(rest, origin, fields, now)?;
        }
        _ => return Err(PoleisError::BadPayload.into()),
    }

    p.executed = true;
    p.write(proposal)
}

/// data: [hash:32] · accounts: [proposal (w), payer (w), then count sessions (w, impacted order)]
/// Permissionless rent recovery. Once every session is terminal and the
/// proposal is spent (executed), dead (it can never execute: the origin's
/// session did not pass, or it asks for consent and some session did not
/// pass) or abandoned (a full ceiling past its expiry), the sessions close and their
/// rent goes back to whoever proposed, and the proposal shrinks to a
/// tombstone (discriminator, version, hash) that keeps its hash taken
/// forever while the rent above it goes home too. The receipts in the log
/// are the record of what it did.
pub fn close_proposal(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    Data(data).expect_len(32)?;
    let hash: [u8; 32] = data[..32].try_into().unwrap();
    if accounts.len() < 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, sessions) = accounts.split_at_mut(2);
    let [proposal, payer] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_writable(proposal)?;
    expect_writable(payer)?;
    let p = read_proposal(proposal, &hash)?;
    if payer.address().as_array() != &p.payer {
        return Err(PoleisError::InvalidAccount.into());
    }
    let count = p.count as usize;
    if sessions.len() != count {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let now = Clock::get()?.slot;
    // dead by execute's own rule: the origin's seal did not pass, or the
    // decision asks for consent and some chamber's seal did not pass. A
    // decision whose origin passed without consent can still execute, so
    // closing it would burn it
    let mut dead = false;
    for (session, unit) in sessions.iter().zip(p.impacted.iter()) {
        let s = read_session(session, &hash, *unit)?;
        if !s.terminal() {
            return Err(PoleisError::NotClosable.into());
        }
        dead |= s.state != crate::STATE_PASSED && (*unit == p.origin || p.consent());
    }
    let abandoned = now > p.expires.saturating_add(crate::MAX_WINDOW_SLOTS);
    if !(p.executed || dead || abandoned) {
        return Err(PoleisError::NotClosable.into());
    }
    for session in sessions.iter_mut() {
        close_to(session, payer)?;
    }
    proposal.resize(Proposal::TOMBSTONE_SIZE)?;
    let floor = Rent::get()?.try_minimum_balance(Proposal::TOMBSTONE_SIZE)?;
    let excess = proposal.lamports().saturating_sub(floor);
    if excess > 0 {
        move_lamports(proposal, payer, excess)?;
    }
    Ok(())
}

/// data: [hash:32][unit:8] · accounts: [session (w), convener (w), proposal]
/// Permissionless rent recovery for a session a polis convened: once the
/// decision is spent (executed) or closed to its tombstone, the session
/// closes and its rent goes home to whoever convened it. A session `propose`
/// opened goes home with its proposal (`close_proposal`).
pub fn adjourn(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [session, convener, proposal] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(40)?;
    let hash = d.h32(0)?;
    let unit = d.u64(32)?;
    expect_writable(session)?;
    expect_writable(convener)?;
    read_session(session, &hash, unit)?;
    let who = Session::convener(&session.try_borrow()?).ok_or(PoleisError::NotClosable)?;
    if convener.address().as_array() != &who {
        return Err(PoleisError::InvalidAccount.into());
    }
    let over = if proposal.data_len() == Proposal::TOMBSTONE_SIZE {
        find_and_expect_pda(proposal, &[b"proposal", &hash])?;
        let t = proposal.try_borrow()?;
        proposal.owned_by(&crate::ID) && t[OFF_DISC] == DISC_PROPOSAL && t[Proposal::OFF_HASH..Proposal::OFF_HASH + 32] == hash
    } else {
        read_proposal(proposal, &hash)?.executed
    };
    if !over {
        return Err(PoleisError::NotClosable.into());
    }
    close_to(session, convener)
}

fn treasury_of(acc: &AccountView, unit_id: u64) -> Result<Treasury, ProgramError> {
    let t = Treasury::read(acc)?;
    if t.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    expect_pda(acc, &[b"treasury", &unit_id.to_le_bytes()], t.bump)?;
    Ok(t)
}

/// data: [hash:32][holder:32][kind:1]
/// accounts, lamports: [claimer (s,w), registry (w), distribution (w), payee (w)]
/// accounts, tokens:   [claimer (s), registry (w), distribution (w), payee token account (w), escrow (w), treasury, token program]
/// A holder's share of a dividend, once: its weight over the weight the
/// distribution was cut against, the claim recorded as the holder's bit in
/// the distribution (no receipt account, no rent per claim). Who may claim
/// what is fixed at the cut: the distribution's roster names each row's
/// identity and its seats × weight as they stood then, and a claim is paid
/// by that row to that identity, whatever moved since. A key or a unit is
/// found in the roster, not in the registry: a holder who has left since
/// the cut still claims; a key that arrived after the cut, or sits in a
/// leaver's old row, is not in the roster and claims nothing; seats bought
/// after the cut add nothing. An NFT seat claims by its own row, which the
/// roster must name. A key claims for itself, into its own account or a
/// token account it owns; a unit's share is cranked by anyone into its own
/// treasury or vault; an NFT's goes to its current owner. A share never
/// claimed stays, and goes home when the distribution closes.
/// A distribution cut before the roster (the prefix alone) is claimed as it
/// always was: the holder found in the registry, priced at its seats ×
/// weight at the claim, and the claim rests its seats for a window.
pub fn claim_dividend(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    claim_dividend_with_nft(accounts, data, None)
}

/// Prefix the legacy dividend accounts with [seat, mint, token]. The claim is
/// charged to the stable seat's existing bit and paid to its CURRENT owner.
pub fn claim_dividend_nft(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    if accounts.len() < 3 { return Err(ProgramError::NotEnoughAccountKeys); }
    let (proof, rest) = accounts.split_at_mut(3);
    claim_dividend_with_nft(rest, data, Some(proof))
}

fn claim_dividend_with_nft(accounts: &mut [AccountView], data: &[u8], nft: Option<&[AccountView]>) -> ProgramResult {
    let d = Data(data);
    d.expect_len(65)?;
    let hash = d.h32(0)?;
    let holder = d.h32(32)?;
    let hkind = data[64];
    if nft.is_none() && hkind != crate::HOLDER_KEY && hkind != crate::HOLDER_UNIT {
        return Err(PoleisError::InvalidHolder.into());
    }
    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (head, tail) = accounts.split_at_mut(4);
    let [claimer, registry, distribution, payee] = head else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    expect_signer(claimer)?;
    expect_writable(distribution)?;
    expect_writable(payee)?;

    let mut dist = read_distribution(distribution, &hash)?;
    expect_writable(registry)?;
    super::read_registry(registry, dist.unit_id)?;
    // a distribution cut with its roster: who claims what was fixed at the cut
    let rostered = distribution.data_len() != crate::state::Distribution::SIZE;
    let i = if let Some([seat, mint, token]) = nft {
        if hkind != crate::HOLDER_NFT || seat.address().as_array() != &holder {
            return Err(PoleisError::InvalidHolder.into());
        }
        super::nft::current_owner(claimer, seat, mint, token, registry)?.1
    } else if rostered {
        crate::state::Distribution::roster_find(&distribution.try_borrow()?, hkind, &holder)?.ok_or(PoleisError::NotHolder)?.0
    } else {
        Registry::find_holder(registry, hkind, &holder)?.ok_or(PoleisError::NotHolder)?
    };
    // the row the roster holds at that index must be this very holder: an
    // NFT seat's own row, or the key or unit just found
    let cut_weight = if rostered {
        match crate::state::Distribution::roster_row(&distribution.try_borrow()?, i)? {
            Some((who, kind, weight)) if who == holder && kind == hkind => Some(weight),
            _ => return Err(PoleisError::NotHolder.into()),
        }
    } else {
        None
    };
    if dist.claimed(i) {
        return Err(PoleisError::AlreadyClaimed.into());
    }
    let w = match cut_weight {
        Some(weight) => weight as u128,
        None => {
            let mut h = Registry::holder(registry, i)?;
            // a claim rests the seats for a window, as a vote does: what was paid on
            // cannot be sold into a second claim in the same breath
            h.last_active = Clock::get()?.slot;
            Registry::set_holder(registry, i, &h)?;
            h.seats.checked_mul(h.weight).ok_or(PoleisError::MathOverflow)? as u128
        }
    };
    if hkind == crate::HOLDER_KEY || hkind == crate::HOLDER_NFT {
        let beneficiary = if nft.is_some() { *claimer.address().as_array() } else { holder };
        if claimer.address().as_array() != &beneficiary {
            return Err(PoleisError::Unauthorized.into());
        }
        let owner_ok = if dist.is_token() { token_account_owner(payee)? == beneficiary } else { payee.address().as_array() == &beneficiary };
        if !owner_ok {
            return Err(PoleisError::Unauthorized.into());
        }
    } else if dist.is_token() {
        expect_vault(payee, holder_unit(&holder), &dist.mint)?;
    } else {
        treasury_of(payee, holder_unit(&holder))?;
    }

    let share_wide = (dist.amount as u128) * w / (dist.total_weight as u128);
    let share: u64 = share_wide.try_into().map_err(|_| PoleisError::MathOverflow)?;
    if share == 0 {
        return Err(PoleisError::ShareIsZero.into());
    }
    let paid_after = dist.paid.checked_add(share).ok_or(PoleisError::MathOverflow)?;
    if paid_after > dist.amount {
        return Err(PoleisError::InvalidAmount.into());
    }

    // guard, then pay: the bit is set before the money moves
    dist.mark(i);
    dist.paid = paid_after;
    dist.claims = dist.claims.checked_add(1).ok_or(PoleisError::MathOverflow)?;
    dist.write(distribution)?;
    if dist.is_token() {
        let [escrow, treasury, token_program, ..] = tail else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        expect_escrow(escrow, &hash)?;
        let t = treasury_of(treasury, dist.unit_id)?;
        transfer_from_vault(dist.unit_id, treasury, t.bump, escrow, payee, token_program, share)?;
    } else {
        move_lamports(distribution, payee, share)?;
        if hkind == crate::HOLDER_UNIT {
            Treasury::add_in(payee, share)?;
        }
    }
    receipt(crate::RCPT_CLAIM, dist.unit_id, &hash, &holder, &dist.mint, share, hkind);
    Ok(())
}
