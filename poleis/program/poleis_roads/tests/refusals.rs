//! Every refusal, one test a family: for each instruction a missing signature, a stranger's signature, a wrong PDA, an account of another owner
//! and a wrong discriminator where the instruction reads one; and each rule of the brief's section 3. Each asserts the exact error and that no
//! account of the world changed (World::refuse).
mod built;
mod common;
use common::*;
use poleis_futarchy::law::{PRICE, UNIT};
use poleis_roads::error::RoadsError::{self, *};
use poleis_roads::state::{Balance, Board, Position, Road};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

/// the world's first slot
const S0: u64 = 1_000_000;
/// a minute in slots
const MIN: u64 = 60_000 / poleis_roads::SLOT_MS;
const U: u64 = UNIT as u64;
const BOARD: u64 = 5;

struct Fx {
    w: World,
    auth: Pubkey,
    crediter: Pubkey,
    judge: Pubkey,
    a: Pubkey,
    b: Pubkey,
    l: Lay,
    road: Pubkey,
}

/// a board, two owners credited 5,000 each, a road of two hours opening a minute from now, a's deposit of 1,000 in it
fn fx() -> Fx {
    let mut w = World::new(S0);
    let auth = w.wallet(10 * SOL);
    let (crediter, judge) = (w.wallet(10 * SOL), w.wallet(SOL));
    let (a, b) = (w.wallet(SOL), w.wallet(SOL));
    w.open_board(auth, BOARD, crediter, judge);
    w.ok(&credit(crediter, BOARD, a, 5_000 * U), "credit a");
    w.ok(&credit(crediter, BOARD, b, 5_000 * U), "credit b");
    let opens_slot = S0 + MIN;
    let closes_slot = opens_slot + 120 * MIN;
    let l = Lay { road_id: [1u8; 32], opens_slot, closes_slot, step_slots: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp: 0, pool_base: 1_000 * U, pool_quote: 1_000 * U, judge_by_slot: closes_slot + 60 * MIN };
    // the house: credited ten stakes, so a test can lay more roads
    w.ok(&credit(crediter, BOARD, auth, 10 * l.stake()), "credit the house");
    w.ok(&lay(auth, BOARD, &l), "lay");
    let road = road_pda(BOARD, &l.road_id);
    w.ok(&deposit(a, BOARD, road, 1_000 * U), "a's deposit");
    Fx { w, auth, crediter, judge, a, b, l, road }
}

fn e(x: RoadsError) -> ProgramError {
    code(x)
}
/// the instruction with its account `i` unsigned
fn unsigned(mut ix: Instruction, i: usize) -> Instruction {
    ix.accounts[i].is_signer = false;
    ix
}
/// the instruction with its account `i` replaced by `key`
fn swap(mut ix: Instruction, i: usize, key: Pubkey) -> Instruction {
    ix.accounts[i].pubkey = key;
    ix
}
/// an account at a fresh address holding `data`, owned by `owner`
fn fake(w: &mut World, data: Vec<u8>, owner: Pubkey) -> Pubkey {
    let key = Pubkey::new_unique();
    w.put(key, Account { lamports: SOL, data, owner, executable: false, rent_epoch: 0 });
    key
}
/// the same bytes as `of`, at a fresh address this program owns (a wrong PDA), owned by the system (another owner), with another
/// discriminator, and with another layout version
fn fakes(w: &mut World, of: &Pubkey) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let d = w.data(of);
    let not_pda = fake(w, d.clone(), PROGRAM_ID);
    let other_owner = fake(w, d.clone(), Pubkey::default());
    let mut dd = d.clone();
    dd[0] = dd[0] % 4 + 1;
    let other_disc = fake(w, dd, PROGRAM_ID);
    let mut dv = d;
    dv[1] = dv[1].wrapping_add(1);
    let other_version = fake(w, dv, PROGRAM_ID);
    (not_pda, other_owner, other_disc, other_version)
}
/// carry a road until it stands at the clock
fn carry_up(w: &mut World, road: Pubkey) {
    for _ in 0..200 {
        w.ok(&carry(road), "carry");
    }
}

#[test]
fn init_board_refusals() {
    let mut f = fx();
    let w = &mut f.w;
    let s = w.wallet(SOL);
    w.refuse(&unsigned(init_board(s, 9, s), 0), ProgramError::MissingRequiredSignature, "init_board unsigned");
    w.refuse(&swap(init_board(s, 9, s), 1, board_pda(10)), e(InvalidPda), "init_board on another board's address");
    w.refuse(&init_board(s, BOARD, s), e(BoardStands), "a board twice, by a stranger");
    w.refuse(&init_board(f.auth, BOARD, s), e(BoardStands), "a board twice, by its authority");
    w.refuse(&Instruction::new_with_bytes(PROGRAM_ID, &[0u8; 10], init_board(s, 9, s).accounts), ProgramError::InvalidInstructionData, "init_board with short data");
}

#[test]
fn set_board_refusals() {
    let mut f = fx();
    let (auth, w) = (f.auth, &mut f.w);
    let s = w.wallet(SOL);
    let board = board_pda(BOARD);
    w.refuse(&unsigned(set_board(auth, BOARD, s, s), 0), ProgramError::MissingRequiredSignature, "set_board unsigned");
    w.refuse(&set_board(s, BOARD, s, s), e(Unauthorized), "set_board by a stranger");
    let (not_pda, other_owner, other_disc, other_version) = fakes(w, &board);
    w.refuse(&swap(set_board(auth, BOARD, s, s), 1, not_pda), e(InvalidPda), "set_board on a copy of the board at another address");
    w.refuse(&swap(set_board(auth, BOARD, s, s), 1, other_owner), e(InvalidAccount), "set_board on a board of another owner");
    w.refuse(&swap(set_board(auth, BOARD, s, s), 1, other_disc), e(InvalidAccount), "set_board on a wrong discriminator");
    w.refuse(&swap(set_board(auth, BOARD, s, s), 1, other_version), e(LayoutVersion), "set_board on another layout version");
    w.refuse(&swap(set_board(auth, BOARD, s, s), 1, balance_pda(BOARD, &f.a)), e(InvalidAccount), "set_board on a balance");
    // A BOARD'S AUTHORITY IS NAMED ONCE: the old 96-byte data (an authority, then the two roles) is no longer the wire
    let mut old = vec![poleis_roads::IX_SET_BOARD];
    old.extend_from_slice(&[s.as_ref(), s.as_ref(), s.as_ref()].concat());
    w.refuse(&Instruction::new_with_bytes(PROGRAM_ID, &old, set_board(auth, BOARD, s, s).accounts), ProgramError::InvalidInstructionData, "set_board naming an authority");
    // the authority names the crediter and the judge; it stays the authority; the road laid keeps the judge it was laid with
    w.ok(&set_board(auth, BOARD, s, s), "set_board");
    assert_eq!(board_view(&w.data(&board)), BoardView { board_id: BOARD, authority: auth, crediter: s, judge: s, credited: w.credited[&BOARD] as u64 });
    assert_eq!(w.road(&f.road).judge, f.judge);
    w.refuse(&credit(f.crediter, BOARD, f.a, 1), e(Unauthorized), "the old crediter credits no more");
    let _ = Board::SIZE;
}

#[test]
fn credit_refusals() {
    let mut f = fx();
    let (cr, a, w) = (f.crediter, f.a, &mut f.w);
    let s = w.wallet(SOL);
    w.refuse(&unsigned(credit(cr, BOARD, a, 1), 0), ProgramError::MissingRequiredSignature, "credit unsigned");
    w.refuse(&credit(s, BOARD, a, 1), e(Unauthorized), "a credit by a stranger");
    w.refuse(&credit(cr, BOARD, a, 0), e(InvalidAmount), "a credit of 0");
    w.refuse(&credit(cr, BOARD, s, 0), e(InvalidAmount), "a credit of 0 to a new balance");
    w.refuse(&swap(credit(cr, BOARD, s, 1), 3, Pubkey::new_unique()), e(InvalidPda), "a new balance at an address that is not its PDA");
    w.refuse(&swap(credit(cr, BOARD, s, 1), 3, balance_pda(BOARD, &a)), e(InvalidAccount), "a credit to s into a's balance");
    let (not_pda, other_owner, other_disc, _) = fakes(w, &balance_pda(BOARD, &a));
    w.refuse(&swap(credit(cr, BOARD, a, 1), 3, not_pda), e(InvalidPda), "a credit to a copy of a's balance at another address");
    w.refuse(&swap(credit(cr, BOARD, a, 1), 3, other_owner), e(InvalidAccount), "a credit to a balance of another owner");
    w.refuse(&swap(credit(cr, BOARD, a, 1), 3, other_disc), e(InvalidAccount), "a credit to a wrong discriminator");
    let (bnp, boo, bod, _) = fakes(w, &board_pda(BOARD));
    w.refuse(&swap(credit(cr, BOARD, a, 1), 1, bnp), e(InvalidPda), "a credit under a board that is not its PDA");
    w.refuse(&swap(credit(cr, BOARD, a, 1), 1, boo), e(InvalidAccount), "a credit under a board of another owner");
    w.refuse(&swap(credit(cr, BOARD, a, 1), 1, bod), e(InvalidAccount), "a credit under a wrong discriminator");
    w.refuse(&swap(credit(cr, BOARD, a, 1), 1, f.road), e(InvalidAccount), "a credit under a road");
    w.refuse(&credit(cr, BOARD, a, u64::MAX), e(MathOverflow), "a credit past u64");
    let _ = Balance::SIZE;
}

#[test]
fn lay_refusals() {
    let mut f = fx();
    let (auth, l0, w) = (f.auth, f.l, &mut f.w);
    let s = w.wallet(10 * SOL);
    let l = Lay { road_id: [2u8; 32], ..l0 };
    w.refuse(&unsigned(lay(auth, BOARD, &l), 0), ProgramError::MissingRequiredSignature, "lay unsigned");
    w.refuse(&lay(s, BOARD, &l), e(Unauthorized), "a road laid by a stranger");
    w.refuse(&swap(lay(auth, BOARD, &l), 2, road_pda(BOARD, &[3u8; 32])), e(InvalidPda), "a road at another road's address");
    w.refuse(&lay(auth, BOARD, &l0), e(RoadStands), "a road twice");
    let (bnp, boo, bod, _) = fakes(w, &board_pda(BOARD));
    w.refuse(&swap(lay(auth, BOARD, &l), 1, bnp), e(InvalidPda), "a road under a board that is not its PDA");
    w.refuse(&swap(lay(auth, BOARD, &l), 1, boo), e(InvalidAccount), "a road under a board of another owner");
    w.refuse(&swap(lay(auth, BOARD, &l), 1, bod), e(InvalidAccount), "a road under a wrong discriminator");
    let past = Lay { opens_slot: w.slot() - 1, ..l };
    w.refuse(&lay(auth, BOARD, &past), e(InvalidTerms), "a road in the past");
    w.refuse(&lay(auth, BOARD, &Lay { step_slots: 2, ..l }), e(InvalidTerms), "a step of 2 slots, 800 ms, shorter than a second (the law refuses)");
    w.refuse(&lay(auth, BOARD, &Lay { closes_slot: l.opens_slot, ..l }), e(InvalidTerms), "an empty window (the law refuses)");
    w.refuse(&lay(auth, BOARD, &Lay { max_step_bp: 0, ..l }), e(InvalidTerms), "a lag limit of 0 (the law refuses)");
    w.refuse(&lay(auth, BOARD, &Lay { fee_bp: 10_000, ..l }), e(InvalidTerms), "a fee of 10000 bp (the law refuses)");
    w.refuse(&lay(auth, BOARD, &Lay { pool_base: 0, ..l }), e(InvalidTerms), "an empty pool (the law refuses)");
    w.refuse(&lay(auth, BOARD, &Lay { pool_base: 1, pool_quote: u64::MAX / 2, ..l }), e(InvalidTerms), "an opening price out of range (the law refuses)");
    w.refuse(&lay(auth, BOARD, &Lay { judge_by_slot: l.closes_slot, ..l }), e(InvalidTerms), "judge_by_slot not after the close");
    // a road opening at the clock itself is laid
    w.ok(&lay(auth, BOARD, &Lay { opens_slot: w.slot(), ..l }), "a road opening now");

    // no span longer than the law's longest window: refused a slot past the bound, taken at it
    let span = poleis_futarchy::law::WINDOW_MAX_MS / poleis_roads::SLOT_MS;
    let far = |opens: u64, road_id: u8| Lay { road_id: [road_id; 32], opens_slot: opens, closes_slot: opens + 120 * MIN, judge_by_slot: opens + 180 * MIN, ..l };
    w.refuse(&lay(auth, BOARD, &far(w.slot() + span + 1, 40)), e(InvalidTerms), "a road opening a slot past the law's longest window from now");
    w.ok(&lay(auth, BOARD, &far(w.slot() + span, 40)), "a road opening the law's longest window from now");
    let late = |judge_by: u64| Lay { road_id: [41; 32], judge_by_slot: judge_by, ..l };
    w.refuse(&lay(auth, BOARD, &late(l.closes_slot + span + 1)), e(InvalidTerms), "a judge's deadline a slot past the law's longest window after the close");
    w.ok(&lay(auth, BOARD, &late(l.closes_slot + span)), "a judge's deadline the law's longest window after the close");
    w.refuse(&lay(auth, BOARD, &Lay { road_id: [42; 32], judge_by_slot: u64::MAX, ..l }), e(InvalidTerms), "a judge's deadline of u64::MAX");

    // THE HOUSE STAKES THE POOL: from its own balance on this board
    let l = Lay { road_id: [43u8; 32], ..l0 };
    w.refuse(&swap(lay(auth, BOARD, &l), 3, balance_pda(BOARD, &s)), e(InvalidAccount), "a lay with no balance");
    w.refuse(&swap(lay(auth, BOARD, &l), 3, balance_pda(BOARD, &f.a)), e(InvalidAccount), "a lay staked from another owner's balance");
    let auth2 = w.wallet(10 * SOL);
    w.ok(&init_board(auth2, BOARD + 1, auth2), "board 6");
    w.ok(&credit(auth2, BOARD + 1, auth, 10 * l.stake()), "the authority on board 6");
    w.refuse(&swap(lay(auth, BOARD, &l), 3, balance_pda(BOARD + 1, &auth)), e(InvalidAccount), "a lay staked from a balance of another board");
    let (bnp, boo, _, _) = fakes(w, &balance_pda(BOARD, &auth));
    w.refuse(&swap(lay(auth, BOARD, &l), 3, bnp), e(InvalidPda), "a lay staked from a balance that is not its PDA");
    w.refuse(&swap(lay(auth, BOARD, &l), 3, boo), e(InvalidAccount), "a lay staked from a balance of another program");
    let mut ix = lay(auth, BOARD, &l);
    ix.accounts[3].is_writable = false;
    w.refuse(&ix, e(InvalidAccount), "a lay with the balance not writable");
    let have = w.balance(BOARD, &auth).amount;
    let big = Lay { pool_base: have / 2 + 1, pool_quote: have / 2 + 1, ..l };
    w.refuse(&lay(auth, BOARD, &big), e(NotEnoughBalance), "a pool over the house's balance");
    let exact = Lay { pool_base: have / 2, pool_quote: have - have / 2, ..l };
    w.ok(&lay(auth, BOARD, &exact), "a pool of exactly the house's balance");
    let hb = w.balance(BOARD, &auth);
    assert_eq!(hb.amount, 0, "the whole balance staked");
}

#[test]
fn deposit_refusals() {
    let mut f = fx();
    let (a, b, road, w) = (f.a, f.b, f.road, &mut f.w);
    let s = w.wallet(SOL);
    w.refuse(&unsigned(deposit(b, BOARD, road, 100 * U), 0), ProgramError::MissingRequiredSignature, "deposit unsigned");
    w.refuse(&swap(swap(deposit(s, BOARD, road, 100 * U), 2, balance_pda(BOARD, &b)), 3, position_pda(&road, &s)), e(Unauthorized), "a stranger deposits b's balance");
    w.refuse(&deposit(s, BOARD, road, 100 * U), e(InvalidAccount), "a deposit with no balance");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 3, position_pda(&road, &a)), e(InvalidPda), "b's deposit into a's position");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 3, Pubkey::new_unique()), e(InvalidPda), "a position that is not its PDA");
    let (rnp, roo, rod, rov) = fakes(w, &road);
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 1, rnp), e(InvalidPda), "a deposit on a road that is not its PDA");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 1, roo), e(InvalidAccount), "a deposit on a road of another owner");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 1, rod), e(InvalidAccount), "a deposit on a wrong discriminator");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 1, rov), e(LayoutVersion), "a deposit on another layout version");
    let (bnp, boo, bod, _) = fakes(w, &balance_pda(BOARD, &b));
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 2, bnp), e(InvalidPda), "a balance that is not its PDA");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 2, boo), e(InvalidAccount), "a balance of another owner");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 2, bod), e(InvalidAccount), "a balance's wrong discriminator");
    // a balance on another board
    let auth2 = w.wallet(10 * SOL);
    w.ok(&init_board(auth2, BOARD + 1, auth2), "board 6");
    w.ok(&credit(auth2, BOARD + 1, b, 100 * U), "b on board 6");
    w.refuse(&swap(deposit(b, BOARD, road, 100 * U), 2, balance_pda(BOARD + 1, &b)), e(InvalidAccount), "a deposit from a balance on another board");
    // the rules
    w.refuse(&deposit(a, BOARD, road, 100 * U), e(AlreadyDeposited), "a deposit twice");
    w.refuse(&deposit(b, BOARD, road, 5_000 * U + 1), e(NotEnoughBalance), "a deposit over the balance");
    w.refuse(&deposit(b, BOARD, road, 0), e(InvalidAmount), "a deposit of nothing");
    w.refuse(&deposit(b, BOARD, road, 1), e(InvalidAmount), "a deposit too small to hold both $WARM and claims");
    w.ok(&deposit(b, BOARD, road, 2), "two micro hold one of each");
    let s2 = w.wallet(SOL);
    w.ok(&credit(f.crediter, BOARD, s2, 100 * U), "s2");
    w.set_slot(f.l.closes_slot);
    w.refuse(&deposit(s2, BOARD, road, 100 * U), e(RoadClosed), "a deposit at the close");
    w.set_slot(f.l.closes_slot - 1);
    w.ok(&deposit(s2, BOARD, road, 100 * U), "a deposit a slot before the close");
}

#[test]
fn trade_refusals() {
    let mut f = fx();
    let (a, b, road, l, w) = (f.a, f.b, f.road, f.l, &mut f.w);
    let s = w.wallet(SOL);
    w.ok(&deposit(b, BOARD, road, 1_000 * U), "b's deposit");
    w.refuse(&trade(a, road, 0, 0, 10 * U, 0), e(RoadNotOpen), "a trade before the open");
    w.set_slot(l.opens_slot);
    w.refuse(&unsigned(trade(a, road, 0, 0, 10 * U, 0), 0), ProgramError::MissingRequiredSignature, "trade unsigned");
    w.refuse(&swap(trade(s, road, 0, 0, 10 * U, 0), 2, position_pda(&road, &a)), e(Unauthorized), "a stranger trades a's position");
    w.refuse(&trade(s, road, 0, 0, 10 * U, 0), e(InvalidAccount), "a trade with no position");
    let (pnp, poo, pod, _) = fakes(w, &position_pda(&road, &a));
    w.refuse(&swap(trade(a, road, 0, 0, 10 * U, 0), 2, pnp), e(InvalidPda), "a position that is not its PDA");
    w.refuse(&swap(trade(a, road, 0, 0, 10 * U, 0), 2, poo), e(InvalidAccount), "a position of another owner");
    w.refuse(&swap(trade(a, road, 0, 0, 10 * U, 0), 2, pod), e(InvalidAccount), "a position's wrong discriminator");
    let (rnp, roo, rod, _) = fakes(w, &road);
    w.refuse(&swap(trade(a, road, 0, 0, 10 * U, 0), 1, rnp), e(InvalidPda), "a road that is not its PDA");
    w.refuse(&swap(trade(a, road, 0, 0, 10 * U, 0), 1, roo), e(InvalidAccount), "a road of another owner");
    w.refuse(&swap(trade(a, road, 0, 0, 10 * U, 0), 1, rod), e(InvalidAccount), "a road's wrong discriminator");
    // a position on another road
    let l2 = Lay { road_id: [7u8; 32], opens_slot: w.slot(), ..l };
    w.ok(&lay(f.auth, BOARD, &l2), "a second road");
    let road2 = road_pda(BOARD, &l2.road_id);
    w.refuse(&swap(trade(a, road2, 0, 0, 10 * U, 0), 2, position_pda(&road, &a)), e(InvalidAccount), "a's position of road 1 on road 2");
    w.refuse(&Instruction::new_with_bytes(PROGRAM_ID, &[5, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], trade(a, road, 0, 0, 1, 0).accounts), ProgramError::InvalidInstructionData, "a branch that is neither");
    // the law's four refusals: a's purse is 500 $WARM and 500 claims in each world
    w.refuse(&trade(a, road, 0, 0, 500 * U + 1, 0), e(NotEnough), "a buy over the purse's $WARM");
    w.refuse(&trade(a, road, 1, 1, 500 * U + 1, 0), e(NotEnough), "a sell over the purse's claims");
    w.refuse(&trade(a, road, 0, 0, 1, 0), e(NothingOut), "a buy too small to take anything out");
    w.refuse(&trade(a, road, 0, 0, 10 * U, u64::MAX), e(PastMin), "a buy past its min_out");
    w.refuse(&trade(a, road, 0, 0, 0, 0), e(OutOfRange), "a trade of nothing");
    w.ok(&trade(a, road, 0, 0, 10 * U, 0), "a buy at the open");
    // a road behind the clock: a price moved far and then left. a's buy of the rest of its $WARM lifts PASS's spot 2.25-fold, and its
    // observation chases it 1 % a step: some 82 boundaries to walk, more than MOST. Refused until anyone carries it, then taken (the trade
    // walks the rest). A stretch at rest costs nothing to carry (FAIL's here): only the chase is walked.
    w.ok(&trade(a, road, 0, 0, 490 * U, 0), "a buy of a's whole $WARM in PASS");
    w.set_slot(l.opens_slot + 100 * MIN);
    w.refuse(&trade(b, road, 1, 0, 10 * U, 0), e(RoadBehind), "a trade on a road with more than MOST boundaries of chase to walk");
    w.ok(&carry(road), "a carry by anyone");
    let r = w.road(&road);
    assert_eq!((r.pass.steps, r.fail.steps), (poleis_roads::MOST, 100), "the carry walked MOST boundaries of PASS's chase and carried FAIL's quiet 100 at once");
    w.ok(&trade(b, road, 1, 0, 10 * U, 0), "the trade, after enough carry");
    assert_eq!(w.road(&road).last_slot, w.slot());
    assert_eq!(w.position(&road, &b).trades, 1);
    // at the close
    w.set_slot(l.closes_slot);
    carry_up(w, road);
    w.refuse(&trade(a, road, 0, 0, 10 * U, 0), e(RoadNotOpen), "a trade at the close (the law's OUTSIDE)");
    w.ok(&decide(road), "decide");
    w.refuse(&trade(a, road, 0, 0, 10 * U, 0), e(RoadClosed), "a trade on a decided road");
}

#[test]
fn carry_refusals() {
    let mut f = fx();
    let (road, l, w) = (f.road, f.l, &mut f.w);
    w.ok(&carry(road), "a carry with nothing to carry (before the open)");
    w.ok(&carry(road), "and again");
    let (rnp, roo, rod, rov) = fakes(w, &road);
    w.refuse(&carry(rnp), e(InvalidPda), "a carry of a road that is not its PDA");
    w.refuse(&carry(roo), e(InvalidAccount), "a carry of a road of another owner");
    w.refuse(&carry(rod), e(InvalidAccount), "a carry of a wrong discriminator");
    w.refuse(&carry(rov), e(LayoutVersion), "a carry of another layout version");
    w.refuse(&carry(balance_pda(BOARD, &f.a)), e(InvalidAccount), "a carry of a balance");
    let mut ix = carry(road);
    ix.accounts[0].is_writable = false;
    w.refuse(&ix, e(InvalidAccount), "a carry of a road not writable");
    // a road nobody touched is carried to the clock by one carry, however far: its 30 boundaries at rest are one step
    w.set_slot(l.opens_slot + 30 * MIN);
    w.ok(&carry(road), "a carry of a quiet road");
    assert_eq!((w.road(&road).pass.steps, w.road(&road).fail.steps), (30, 30), "one carry carries a quiet road all 30 boundaries due (a boundary is due the instant the clock reaches it)");
    // a price moved far and then left: one carry walks MOST boundaries of its chase, and carries the quiet branch whole
    w.ok(&trade(f.a, road, 1, 0, 500 * U, 0), "a buy of a's whole $WARM in FAIL");
    w.set_slot(l.opens_slot + 115 * MIN);
    w.ok(&carry(road), "a carry that walks");
    assert_eq!(w.road(&road).fail.steps, 30 + poleis_roads::MOST, "one carry walks MOST boundaries of FAIL's chase (of some 82 to walk)");
    assert_eq!(w.road(&road).pass.steps, 115, "and carries PASS, at rest, to the clock");
    let _ = Road::SIZE;
}

#[test]
fn decide_refusals() {
    let mut f = fx();
    let (road, l, w) = (f.road, f.l, &mut f.w);
    w.refuse(&decide(road), e(NotClosed), "decide before the open");
    // a price moved far and then left: a's buy of its whole $WARM in FAIL, 90 steps before the close, leaves some 82 boundaries of chase to
    // walk at the close, more than MOST (the 30 quiet steps before it cost the trade nothing)
    w.set_slot(l.closes_slot - 90 * MIN);
    w.ok(&trade(f.a, road, 1, 0, 500 * U, 0), "a buy of a's whole $WARM in FAIL");
    w.set_slot(l.closes_slot - 1);
    w.refuse(&decide(road), e(NotClosed), "decide a slot before the close");
    w.set_slot(l.closes_slot);
    w.refuse(&decide(road), e(RoadBehind), "decide on a road with more than MOST boundaries of chase to walk");
    w.ok(&carry(road), "a carry");
    w.ok(&decide(road), "decide, after enough carry");
    assert_eq!(w.road(&road).verdict, 2, "FAIL was pushed up: PASS's average does not beat FAIL's by the threshold");
    w.refuse(&decide(road), e(AlreadyDecided), "decide twice");
    let (rnp, roo, _, _) = fakes(w, &road);
    w.refuse(&decide(rnp), e(InvalidPda), "decide a road that is not its PDA");
    w.refuse(&decide(roo), e(InvalidAccount), "decide a road of another owner");
}

#[test]
fn receipts_refusals() {
    let mut f = fx();
    let (judge, road, l, w) = (f.judge, f.road, f.l, &mut f.w);
    let s = w.wallet(SOL);
    w.set_slot(l.closes_slot);
    carry_up(w, road);
    w.refuse(&receipts(judge, road, 1, 0), e(NotDecided), "receipts before the verdict");
    w.ok(&decide(road), "decide");
    let world = w.road(&road).verdict - 1;
    w.refuse(&unsigned(receipts(judge, road, world, 0), 0), ProgramError::MissingRequiredSignature, "receipts unsigned");
    w.refuse(&receipts(s, road, world, 0), e(Unauthorized), "receipts by a stranger");
    w.refuse(&receipts(f.auth, road, world, 0), e(Unauthorized), "receipts by the board's authority");
    let (rnp, roo, rod, _) = fakes(w, &road);
    w.refuse(&swap(receipts(judge, road, world, 0), 1, rnp), e(InvalidPda), "receipts on a road that is not its PDA");
    w.refuse(&swap(receipts(judge, road, world, 0), 1, roo), e(InvalidAccount), "receipts on a road of another owner");
    w.refuse(&swap(receipts(judge, road, world, 0), 1, rod), e(InvalidAccount), "receipts on a wrong discriminator");
    w.refuse(&receipts(judge, road, 1 - world, 0), e(OtherWorld), "receipts of the other world");
    w.refuse(&receipts(judge, road, world, PRICE as u64 + 1), e(WorthRange), "a worth over one");
    w.set_slot(l.judge_by_slot);
    w.refuse(&receipts(judge, road, world, PRICE as u64), e(TooLate), "receipts at judge_by_slot");
    w.set_slot(l.judge_by_slot - 1);
    w.ok(&receipts(judge, road, world, PRICE as u64), "receipts a slot before judge_by_slot, a worth of exactly one");
    w.refuse(&receipts(judge, road, world, 0), e(AlreadySaid), "receipts twice");
}

#[test]
fn settle_refusals() {
    let mut f = fx();
    let (a, b, judge, road, l, w) = (f.a, f.b, f.judge, f.road, f.l, &mut f.w);
    w.ok(&deposit(b, BOARD, road, 2_000 * U), "b's deposit");
    w.refuse(&settle(road, a, BOARD), e(NotSettleable), "a settle before the close");
    w.set_slot(l.opens_slot + 10 * MIN);
    w.ok(&trade(a, road, 0, 0, 200 * U, 0), "a buys pass");
    w.set_slot(l.closes_slot);
    carry_up(w, road);
    w.ok(&decide(road), "decide");
    w.refuse(&settle(road, a, BOARD), e(NotSettleable), "a settle before the receipts");
    let world = w.road(&road).verdict - 1;
    w.ok(&receipts(judge, road, world, 400_000_000_000), "receipts");
    w.refuse(&swap(settle(road, a, BOARD), 2, balance_pda(BOARD, &b)), e(InvalidAccount), "a's settle into b's balance");
    w.refuse(&swap(settle(road, a, BOARD), 3, b), e(InvalidAccount), "a's position's lamports to b");
    w.refuse(&swap(settle(road, a, BOARD), 1, position_pda(&road, &b)), e(InvalidAccount), "b's position settled as a's");
    let (pnp, poo, pod, _) = fakes(w, &position_pda(&road, &a));
    w.refuse(&swap(settle(road, a, BOARD), 1, pnp), e(InvalidPda), "a position that is not its PDA");
    w.refuse(&swap(settle(road, a, BOARD), 1, poo), e(InvalidAccount), "a position of another owner");
    w.refuse(&swap(settle(road, a, BOARD), 1, pod), e(InvalidAccount), "a position's wrong discriminator");
    let (bnp, _, _, _) = fakes(w, &balance_pda(BOARD, &a));
    w.refuse(&swap(settle(road, a, BOARD), 2, bnp), e(InvalidPda), "a balance that is not its PDA");
    w.ok(&settle(road, a, BOARD), "a's settle");
    w.refuse(&settle(road, a, BOARD), e(InvalidAccount), "a settle twice");
    let before = w.balance(BOARD, &b).amount;
    w.ok(&settle(road, b, BOARD), "b's settle, who never traded");
    assert_eq!(w.balance(BOARD, &b).amount, before + 2_000 * U, "a position that never traded gets back exactly its deposit");
    assert_eq!(w.road(&road).positions, 0);
    let _ = Position::SIZE;
}

#[test]
fn a_void_road_gives_back_exactly_each_deposit() {
    let mut f = fx();
    let (a, b, judge, road, l, w) = (f.a, f.b, f.judge, f.road, f.l, &mut f.w);
    w.ok(&deposit(b, BOARD, road, 3_000 * U), "b's deposit");
    w.set_slot(l.opens_slot + 10 * MIN);
    w.ok(&trade(a, road, 0, 0, 300 * U, 0), "a buys pass");
    w.ok(&trade(b, road, 0, 1, 900 * U, 0), "b sells pass");
    w.set_slot(l.closes_slot + MIN);
    carry_up(w, road);
    w.ok(&decide(road), "decide");
    w.set_slot(l.judge_by_slot - 1);
    w.refuse(&settle(road, a, BOARD), e(NotSettleable), "a slot before the judge's deadline, no word: not yet void");
    w.set_slot(l.judge_by_slot);
    let world = w.road(&road).verdict - 1;
    w.refuse(&receipts(judge, road, world, 0), e(TooLate), "the judge's word at its deadline");
    for (o, d) in [(a, 1_000 * U), (b, 3_000 * U)] {
        let before = w.balance(BOARD, &o);
        w.ok(&settle(road, o, BOARD), "a void settle");
        let after = w.balance(BOARD, &o);
        assert_eq!((after.amount, after.in_roads), (before.amount + d, before.in_roads - d), "the void road gives back exactly the deposit");
    }
    let r = w.road(&road);
    assert_eq!((r.positions, r.given_back, r.said), (0, 4_000 * U, 0));
    let before = w.balance(BOARD, &f.auth);
    w.ok(&collect(road, f.auth, BOARD), "collect the void road");
    let after = w.balance(BOARD, &f.auth);
    assert_eq!((after.amount, after.in_roads), (before.amount + l.stake(), before.in_roads - l.stake()), "a void road is collected for exactly its stake");
}

#[test]
fn a_judged_road_settles_by_its_receipts_also_after_the_judges_deadline() {
    // the judge spoke in time. A position settled long after the judge's deadline still settles by the receipts, never as a void road's:
    // the deadline is the judge's, not the positions'
    let mut f = fx();
    let (a, b, judge, road, l, w) = (f.a, f.b, f.judge, f.road, f.l, &mut f.w);
    w.ok(&deposit(b, BOARD, road, 3_000 * U), "b's deposit");
    w.set_slot(l.opens_slot + 10 * MIN);
    // PASS is sold down and FAIL bid up, so the FAIL world happens, and both traded in it: a bought its claim dear, b sold it dear
    w.ok(&trade(a, road, 0, 0, 300 * U, 0), "a buys pass");
    w.ok(&trade(b, road, 0, 1, 900 * U, 0), "b sells pass");
    w.ok(&trade(a, road, 1, 0, 200 * U, 0), "a buys fail");
    w.ok(&trade(b, road, 1, 1, 100 * U, 0), "b sells fail");
    w.set_slot(l.closes_slot + MIN);
    carry_up(w, road);
    w.ok(&decide(road), "decide");
    assert_eq!(w.road(&road).verdict, 2, "the road fails: the FAIL world happens");
    w.ok(&receipts(judge, road, 1, 250_000_000_000), "the judge's word, in time: a claim was worth a quarter");
    let before = w.balance(BOARD, &a).amount;
    w.ok(&settle(road, a, BOARD), "a's settle, before the deadline");
    let back_a = w.balance(BOARD, &a).amount - before;
    w.set_slot(l.judge_by_slot + 1_000 * MIN);
    let before = w.balance(BOARD, &b).amount;
    w.ok(&settle(road, b, BOARD), "b's settle, long after the judge's deadline");
    let back_b = w.balance(BOARD, &b).amount - before;
    assert!(back_a < 1_000 * U, "a bought a claim worth a quarter for more than one: it gets back {back_a}, less than its deposit");
    assert!(back_b > 3_000 * U, "b sold that claim dear: after the deadline it still gets back {back_b}, by the receipts, not the deposit a void road gives");
    w.ok(&collect(road, f.auth, BOARD), "collect");
}

#[test]
fn collect_refusals() {
    let mut f = fx();
    let (auth, a, road, l, w) = (f.auth, f.a, f.road, f.l, &mut f.w);
    // a second road nobody deposits in
    let l2 = Lay { road_id: [8u8; 32], ..l };
    w.ok(&lay(auth, BOARD, &l2), "a road nobody deposits in");
    let road2 = road_pda(BOARD, &l2.road_id);
    w.refuse(&collect(road2, auth, BOARD), e(NotCollectable), "collect before the close");
    w.set_slot(l.closes_slot - 1);
    w.refuse(&collect(road2, auth, BOARD), e(NotCollectable), "collect a slot before the close");
    w.set_slot(l.closes_slot);
    w.refuse(&collect(road, auth, BOARD), e(NotCollectable), "collect after the close with a position standing");
    w.refuse(&collect(road2, a, BOARD), e(InvalidAccount), "collect into another owner's balance");
    let auth2 = w.wallet(10 * SOL);
    w.ok(&init_board(auth2, BOARD + 1, auth2), "board 6");
    w.ok(&credit(auth2, BOARD + 1, auth, 1), "the house on board 6");
    w.refuse(&swap(collect(road2, auth, BOARD), 1, balance_pda(BOARD + 1, &auth)), e(InvalidAccount), "collect into a balance of another board");
    w.refuse(&swap(collect(road2, auth, BOARD), 2, board_pda(BOARD + 1)), e(InvalidAccount), "collect with a board that is not the road's");
    let (gnp, goo, _, _) = fakes(w, &board_pda(BOARD));
    w.refuse(&swap(collect(road2, auth, BOARD), 2, gnp), e(InvalidPda), "collect with a copy of the board at another address");
    w.refuse(&swap(collect(road2, auth, BOARD), 2, goo), e(InvalidAccount), "collect with a board of another owner");
    let mut ro = collect(road2, auth, BOARD);
    ro.accounts[2].is_writable = false;
    w.refuse(&ro, e(InvalidAccount), "collect with the board not writable");
    let (rnp, roo, rod, _) = fakes(w, &road2);
    w.refuse(&swap(collect(road2, auth, BOARD), 0, rnp), e(InvalidPda), "collect a road that is not its PDA");
    w.refuse(&swap(collect(road2, auth, BOARD), 0, roo), e(InvalidAccount), "collect a road of another owner");
    w.refuse(&swap(collect(road2, auth, BOARD), 0, rod), e(InvalidAccount), "collect a wrong discriminator as a road");
    let (bnp, boo, bod, _) = fakes(w, &balance_pda(BOARD, &auth));
    w.refuse(&swap(collect(road2, auth, BOARD), 1, bnp), e(InvalidPda), "collect into a balance that is not its PDA");
    w.refuse(&swap(collect(road2, auth, BOARD), 1, boo), e(InvalidAccount), "collect into a balance of another program");
    w.refuse(&swap(collect(road2, auth, BOARD), 1, bod), e(InvalidAccount), "collect into a wrong discriminator as a balance");
    let mut ix = collect(road2, auth, BOARD);
    ix.accounts[1].is_writable = false;
    w.refuse(&ix, e(InvalidAccount), "collect into a balance not writable");
    let before = w.balance(BOARD, &auth);
    w.ok(&collect(road2, auth, BOARD), "collect the road nobody deposited in, right at its close");
    let after = w.balance(BOARD, &auth);
    assert_eq!((after.amount, after.in_roads), (before.amount + l2.stake(), before.in_roads - l2.stake()), "exactly its stake");
    w.refuse(&collect(road2, auth, BOARD), e(AlreadyCollected), "collect twice");
    let _ = Balance::SIZE;
}

#[test]
fn the_cap_no_road_gives_back_more_than_entered_it() {
    let mut f = fx();
    let (auth, a, b, road, l, w) = (f.auth, f.a, f.b, f.road, f.l, &mut f.w);
    w.ok(&deposit(b, BOARD, road, 1_000 * U), "b's deposit");
    let entered = w.road(&road).left();
    assert_eq!(entered, l.stake() + 2_000 * U);
    // the road's bytes written by hand: given_back raised until less is left than a's deposit (what the void road owes it)
    w.keep_book = false;
    let mut acc = w.account(&road);
    let phantom = entered - 500 * U;
    acc.data[Road::OFF_GIVEN_BACK..Road::OFF_GIVEN_BACK + 8].copy_from_slice(&phantom.to_le_bytes());
    w.put(road, acc);
    assert_eq!(w.road(&road).left(), 500 * U);
    w.set_slot(l.judge_by_slot);
    let (a0, b0, h0) = (w.balance(BOARD, &a).amount, w.balance(BOARD, &b).amount, w.balance(BOARD, &auth).amount);
    w.ok(&settle(road, a, BOARD), "a's void settle, owed 1,000 with 500 left");
    assert_eq!(w.balance(BOARD, &a).amount, a0 + 500 * U, "a got exactly what was left");
    w.ok(&settle(road, b, BOARD), "b's void settle, with nothing left");
    assert_eq!(w.balance(BOARD, &b).amount, b0, "b got nothing: nothing was left");
    assert_eq!(w.road(&road).left(), 0);
    w.ok(&collect(road, auth, BOARD), "collect");
    assert_eq!(w.balance(BOARD, &auth).amount, h0, "the house got 0");
    let out = (w.balance(BOARD, &a).amount - a0) + (w.balance(BOARD, &b).amount - b0) + (w.balance(BOARD, &auth).amount - h0);
    assert!(out <= entered, "the road gave back {out}, no more than the {entered} that entered it");
}

#[test]
fn a_road_never_decided_is_void_after_its_judges_deadline() {
    let mut f = fx();
    let (a, road, l, w) = (f.a, f.road, f.l, &mut f.w);
    w.set_slot(l.judge_by_slot);
    let before = w.balance(BOARD, &a).amount;
    w.ok(&settle(road, a, BOARD), "a settle on a road nobody decided");
    assert_eq!(w.balance(BOARD, &a).amount, before + 1_000 * U);
}
