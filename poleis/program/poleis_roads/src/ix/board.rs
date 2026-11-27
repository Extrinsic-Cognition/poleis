//! A board, a polis's own market place, and the balances of play $WARM on it.

use crate::error::RoadsError;
use crate::state::{terms_of, Balance, Board, Law, SEED_BALANCE, SEED_BOARD};
use crate::util::{create_pda, expect_signer, expect_writable, find_and_expect_pda, ms_of, stands, Data};
use poleis_futarchy::law::WINDOW_MAX_MS;
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// data: [board_id:8][authority:32] · accounts: [payer (s,w), board (w), system]
/// Opens board N for the authority it names; the payer only pays. The crediter and the judge start as the authority, so nobody but the
/// authority ever names a role (`set_board`): whoever opens a board for a polis's agent key gains nothing by it, since every later change
/// needs that key's signature, which only a sealed act of the polis gives. Nothing is global: board N is the first opener's to name, and a
/// number opened for another authority can never become the polis's, which then names another.
/// Refuses a board that stands.
pub fn init_board(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [payer, board, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(40)?;
    let board_id = d.u64(0)?;
    let authority = Address::new_from_array(d.h32(8)?);
    expect_signer(payer)?;
    expect_writable(payer)?;
    expect_writable(board)?;
    let id = board_id.to_le_bytes();
    let seeds: &[&[u8]] = &[SEED_BOARD, &id];
    let bump = find_and_expect_pda(board, seeds)?;
    if stands(board) {
        return Err(RoadsError::BoardStands.into());
    }
    create_pda(payer, board, Board::SIZE, seeds, bump)?;
    Board { board_id, authority, crediter: authority, judge: authority, credited: 0, law: Law::default(), earned: 0, bump }.write(board)
}

/// data: [polis:32][unit:8][step_slots:8][max_step_bp:4][threshold_bp:4][fee_bp:4][pool_base:8][pool_quote:8][judge_slots:8]
/// accounts: [authority (s), board (w)]
/// A ROAD IS LAID BY A RULE: the board's own law for the roads `lay_for` lays, the authority's signature. Its terms must be terms the law
/// takes (`Terms::check` on a window of two steps) and `judge_slots` (how long the judge has after a road's close) more than 0 and at most the
/// law's longest window: else `InvalidTerms`. A `polis` of all zero turns the rule off and keeps no terms. A road's stretches at rest cost
/// nothing to carry, however small the step (the law's A QUIET STRETCH IS ONE STEP); only the steps in which a price is still moving toward
/// a trade's spot are walked, `MOST` (66) a call, and this law's step and lag limit set how many those are: a smaller step or a narrower
/// lag limit walks more of them after a trade that moved a price far. A road keeps the terms it was laid with: a later law changes only the
/// roads laid after it.
pub fn set_law(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, board] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(84)?;
    expect_signer(authority)?;
    expect_writable(board)?;
    let mut b = Board::read_pda(board)?;
    if authority.address() != &b.authority {
        return Err(RoadsError::Unauthorized.into());
    }
    let law = Law {
        polis: d.h32(0)?,
        unit: d.u64(32)?,
        step_slots: d.u64(40)?,
        max_step_bp: d.u32(48)?,
        threshold_bp: d.u32(52)?,
        fee_bp: d.u32(56)?,
        pool_base: d.u64(60)?,
        pool_quote: d.u64(68)?,
        judge_slots: d.u64(76)?,
    };
    b.law = if law.stands() {
        let window = law.step_slots.checked_mul(2).ok_or(RoadsError::InvalidTerms)?;
        let t = terms_of(0, window, law.step_slots, law.max_step_bp, law.threshold_bp, law.fee_bp, law.pool_base, law.pool_quote).map_err(|_| RoadsError::InvalidTerms)?;
        t.check().map_err(|_| RoadsError::InvalidTerms)?;
        let judge_ms = ms_of(law.judge_slots).map_err(|_| RoadsError::InvalidTerms)?;
        if law.judge_slots == 0 || judge_ms > WINDOW_MAX_MS {
            return Err(RoadsError::InvalidTerms.into());
        }
        law
    } else {
        Law::default()
    };
    b.write(board)
}

/// data: [crediter:32][judge:32] · accounts: [authority (s), board (w)]
/// The authority names the crediter and the judge (the same to keep one). A BOARD'S AUTHORITY IS NAMED ONCE: nothing changes it after
/// `init_board`, so a board whose authority is a polis's agent key was always the polis's, its law, its roles and every credit on it. A road
/// already laid keeps the judge it was laid with.
pub fn set_board(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [authority, board] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(64)?;
    expect_signer(authority)?;
    expect_writable(board)?;
    let mut b = Board::read_pda(board)?;
    if authority.address() != &b.authority {
        return Err(RoadsError::Unauthorized.into());
    }
    b.crediter = Address::new_from_array(d.h32(0)?);
    b.judge = Address::new_from_array(d.h32(32)?);
    b.write(board)
}

/// data: [amount:8] · accounts: [crediter (s,w), board (w), owner, balance (w), system]
/// Adds `amount` (at least 1) to the owner's balance on this board, creating it if it is not there (the crediter pays its rent), and to the
/// board's `credited`, the book's total (checked). Only the board's crediter. The owner signs nothing: it is only the address the balance
/// belongs to.
pub fn credit(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [crediter, board, owner, balance, _system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let d = Data(data);
    d.expect_len(8)?;
    let amount = d.u64(0)?;
    expect_signer(crediter)?;
    expect_writable(crediter)?;
    expect_writable(balance)?;
    let mut b = Board::read_pda(board)?;
    if crediter.address() != &b.crediter {
        return Err(RoadsError::Unauthorized.into());
    }
    if amount == 0 {
        return Err(RoadsError::InvalidAmount.into());
    }
    expect_writable(board)?;
    b.credited = b.credited.checked_add(amount).ok_or(RoadsError::MathOverflow)?;
    if stands(balance) {
        let mut bal = Balance::read_pda(balance)?;
        if bal.board_id != b.board_id || &bal.owner != owner.address() {
            return Err(RoadsError::InvalidAccount.into());
        }
        bal.amount = bal.amount.checked_add(amount).ok_or(RoadsError::MathOverflow)?;
        bal.write(balance)?;
    } else {
        let id = b.board_id.to_le_bytes();
        let seeds: &[&[u8]] = &[SEED_BALANCE, &id, owner.address().as_ref()];
        let bump = find_and_expect_pda(balance, seeds)?;
        create_pda(crediter, balance, Balance::SIZE, seeds, bump)?;
        Balance { board_id: b.board_id, owner: *owner.address(), amount, in_roads: 0, bump }.write(balance)?;
    }
    // the board's book total, written after the balance's creation (a system CPI that does not carry the board)
    b.write(board)
}
