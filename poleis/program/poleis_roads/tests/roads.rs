//! One road, from the first credit to the last settle, held against the fold on the same trades stamped at the clock's milliseconds.
mod built;
mod common;
use common::*;
use embertide_futarchy::law::{back_of, decide as fold_decide, market_with, pool_result, settle_of, world_of, Receipts, Which, PRICE, UNIT};
use poleis_roads::error::RoadsError;
use solana_pubkey::Pubkey;

/// the world's first slot
const S0: u64 = 1_000_000;
/// a minute in slots, at the chain's nominal 400 ms a slot
const MIN: u64 = 60_000 / poleis_roads::SLOT_MS;
const U: u64 = UNIT as u64;

#[test]
fn a_board_opens_once_for_the_authority_it_names_and_only_that_authority_names_the_roles() {
    let mut w = World::new(S0);
    let payer = w.wallet(10 * SOL);
    let (authority, crediter, judge) = (w.wallet(SOL), w.wallet(SOL), w.wallet(SOL));
    w.ok(&init_board(payer, 7, authority), "board 7, opened by a payer for an authority it names");
    let b = board_view(&w.data(&board_pda(7)));
    assert_eq!(b, BoardView { board_id: 7, authority, crediter: authority, judge: authority, credited: 0 }, "the crediter and the judge start as the authority");
    assert_eq!(law_view(&w.data(&board_pda(7))).polis, Pubkey::default(), "a board opens with no law: it lays nothing by rule");
    // the payer is nobody: it names no role and credits nothing
    w.refuse(&set_board(payer, 7, payer, payer), code(RoadsError::Unauthorized), "the payer names no role");
    w.refuse(&credit(payer, 7, payer, 1), code(RoadsError::Unauthorized), "the payer credits nothing");
    w.ok(&set_board(authority, 7, crediter, judge), "the authority names the crediter and the judge");
    assert_eq!(board_view(&w.data(&board_pda(7))), BoardView { board_id: 7, authority, crediter, judge, credited: 0 });
    w.refuse(&init_board(crediter, 7, crediter), code(RoadsError::BoardStands), "board 7 again");
}

#[test]
fn one_road_from_the_first_credit_to_the_last_settle() {
    // the same road and the same trades at a fee of 100 bp and at none: the fold and the program are held equal at each, the book closes at
    // each, and the house collects the fees with what is left
    let (left, pool, verdict, fees) = one_road(100);
    let (left0, pool0, verdict0, fees0) = one_road(0);
    eprintln!("  the house collected {left} at 100 bp against {left0} with no fee, {} more; the road's fees in the world that happened {fees} (none at no fee: {fees0}); the pool's result {pool} against {pool0}; the verdicts {verdict} and {verdict0}", left as i128 - left0 as i128);
    assert!(left > left0 && pool > pool0 && fees > 0 && fees0 == 0, "the fee is the house's at the collect");
}

/// one road from the first credit to the last settle at a fee of `fee_bp`: what the house collected, the fold's pool result, the verdict, and
/// the road's fees in the world that happened
fn one_road(fee_bp: u32) -> (u64, i128, u8, u128) {
    let mut w = World::new(S0);
    let authority = w.wallet(10 * SOL);
    let (crediter, judge) = (w.wallet(10 * SOL), w.wallet(SOL));
    let board = 3u64;
    w.open_board(authority, board, crediter, judge);

    // three owners credited
    let owners: Vec<Pubkey> = (0..3).map(|_| w.wallet(SOL)).collect();
    let credited = [10_000 * U, 4_000 * U, 7_500 * U];
    for (o, c) in owners.iter().zip(credited) {
        w.ok(&credit(crediter, board, *o, c - 1), "a credit that creates the balance");
        w.ok(&credit(crediter, board, *o, 1), "a credit to a balance that stands");
        assert_eq!(w.balance(board, o).amount, c);
    }

    // a road of four hours at a step a minute, opening in a minute; the house (its authority) is credited its stake first
    let opens_slot = S0 + MIN;
    let l = Lay { road_id: [9u8; 32], opens_slot, closes_slot: opens_slot + 240 * MIN, step_slots: MIN, max_step_bp: 100, threshold_bp: 300, fee_bp, pool_base: 1_000 * U, pool_quote: 1_000 * U, judge_by_slot: opens_slot + 28 * 60 * MIN };
    let t = l.terms();
    let house_credit = l.stake() + 7;
    w.ok(&credit(crediter, board, authority, house_credit), "the house's balance");
    w.ok(&lay(authority, board, &l), "the road");
    let road = road_pda(board, &l.road_id);
    let r = w.road(&road);
    assert_eq!((r.judge, r.staker, r.stake, r.collected), (judge, authority, l.stake(), 0));
    // THE BOARD AS IT STOOD AT THE LAY: the road keeps the board's `credited` of that moment, and a later credit does not move it
    let at_lay = credited.iter().sum::<u64>() + house_credit;
    assert_eq!(board_view(&w.data(&board_pda(board))).credited, at_lay);
    assert_eq!(r.credited_at_lay, at_lay, "the road keeps the board as it stood at its lay");
    let late = w.wallet(SOL);
    w.ok(&credit(crediter, board, late, 1_000_000 * U), "a credit after the lay, to an owner who never trades");
    assert_eq!(board_view(&w.data(&board_pda(board))).credited, at_lay + 1_000_000 * U);
    assert_eq!(w.road(&road).credited_at_lay, at_lay, "a later credit does not move it");
    let hb = w.balance(board, &authority);
    assert_eq!((hb.amount, hb.in_roads), (7, l.stake()), "the stake left the house's balance for its in_roads");

    // three deposits of different sizes
    let deposits = [500 * U, 1_200 * U, 3_000 * U];
    for (o, d) in owners.iter().zip(deposits) {
        w.ok(&deposit(*o, board, road, d), "a deposit");
        let p = w.position(&road, o);
        assert_eq!((p.deposit, p.pass, p.fail), (d, t.purse_of(d as u128), t.purse_of(d as u128)));
    }
    let r = w.road(&road);
    assert_eq!((r.positions, r.deposits), (3, deposits.iter().sum::<u64>()));

    // the trades: (minutes after the opening, owner, branch, side, amount)
    let script: [(u64, usize, u8, u8, u64); 10] = [
        (0, 0, 0, 0, 100 * U),
        (5, 1, 1, 1, 200 * U),
        (12, 2, 0, 0, 500 * U),
        (12, 0, 0, 1, 50 * U),
        (40, 1, 0, 0, 100 * U),
        (190, 2, 1, 0, 300 * U),
        (200, 0, 1, 0, 50 * U),
        (205, 1, 0, 1, 100 * U),
        (205, 2, 0, 1, 200 * U),
        (230, 0, 0, 0, 30 * U),
    ];
    let (mut entries, mut seq, mut carries, mut behind) = (Vec::new(), 0u64, 0u32, 0u32);
    for (min, who, branch, side, amount) in script {
        w.set_slot(opens_slot + min * MIN);
        let ix = trade(owners[who], road, branch, side, amount, 0);
        loop {
            match w.run(&ix) {
                Ok(_) => break,
                Err(e) if e == code(RoadsError::RoadBehind) => {
                    behind += 1;
                    w.ok(&carry(road), "a carry");
                    carries += 1;
                }
                Err(e) => panic!("trade at {min} min refused: {e:?}"),
            }
        }
        seq += 1;
        entries.push(entry(seq, w.now_ms(), &owners[who], law_trade(branch, side, amount, 0)));
        assert_eq!(w.road(&road).last_slot, w.slot());
    }
    // A QUIET STRETCH IS ONE STEP: the long gap (minutes 40 to 190) stands at rest once the observations have caught the spots the trades
    // before it moved, and every chase here is shorter than MOST boundaries, so no trade is refused behind and none needs a carry
    assert_eq!((behind, carries), (0, 0), "the long gap needed no carry: {behind} refused behind, {carries} carries");

    // the close, two hours after it
    w.set_slot(opens_slot + 360 * MIN);
    w.refuse(&receipts(judge, road, 0, 0), code(RoadsError::NotDecided), "receipts before the verdict");
    w.ok(&decide(road), "decide");
    let now = w.now_ms();
    let ros = roster(&owners.iter().copied().zip(deposits).collect::<Vec<_>>());
    let m = market_with(&t, &ros, &entries, now);
    assert_eq!(m.accepted, entries.len() as u64);
    let d = fold_decide(&t, &m, now);
    let world = world_of(&d).unwrap();
    let r = w.road(&road);
    assert_eq!((r.pass, r.fail), (m.pass, m.fail), "the road's branches at the verdict are the fold's");
    assert_eq!(r.verdict, if world == Which::Pass { 1 } else { 2 }, "the verdict is the fold's");

    // the judge's receipts
    let worth = 600_000_000_000u64;
    assert!((worth as u128) < PRICE);
    w.ok(&receipts(judge, road, if world == Which::Pass { 0 } else { 1 }, worth), "the receipts");
    let rc = Receipts { world, worth: worth as u128 };
    let fold_pool = pool_result(&t, &m, &d, &rc).unwrap();
    assert_eq!(w.road(&road).pool_result, fold_pool, "the road's pool result is the fold's");
    // the judge's word reads the two pools through the law's own decision and writes the road back: on a decided road that moves nothing
    let said = w.road(&road);
    assert_eq!((said.pass, said.fail, said.verdict, said.said), (m.pass, m.fail, r.verdict, 1), "the judge's word moved nothing of the road's two pools nor its verdict");
    let rows = settle_of(&t, &m, &d, &rc).unwrap();

    // three settles, by anyone
    let mut backs = 0i128;
    for (i, o) in owners.iter().enumerate() {
        let pos = position_pda(&road, o);
        let (rent, before) = (w.lamports(&pos), w.lamports(o));
        w.ok(&settle(road, *o, board), "a settle");
        let result = rows.iter().find(|s| s.key == o.to_string()).unwrap().result;
        let back = back_of(deposits[i] as u128, result) as u64;
        let b = w.balance(board, o);
        assert_eq!(b.amount, credited[i] - deposits[i] + back, "owner {i}: credited, less its deposit, plus exactly back_of of the fold's result");
        assert_eq!(b.in_roads, 0);
        assert_eq!(w.lamports(&pos), 0, "owner {i}'s position is closed");
        assert!(w.data(&pos).is_empty());
        assert_eq!(w.lamports(o), before + rent, "owner {i}: the position's lamports are with its owner");
        backs += back as i128;
        eprintln!("  owner {i}: deposit {} back {back} result {result}", deposits[i]);
    }
    let r = w.road(&road);
    let dep: i128 = deposits.iter().map(|d| *d as i128).sum();
    assert_eq!((r.positions, r.given_back as i128), (0, backs));
    assert!((backs + fold_pool - dep).abs() <= owners.len() as i128, "the backs and the pool's result are the deposits to one micro a trader: {backs} + {fold_pool} vs {dep}");
    w.refuse(&settle(road, owners[0], board), code(RoadsError::InvalidAccount), "a settle twice");

    // the house collects what is left, by a stranger's hand
    let left = r.left();
    assert_eq!(left as i128, l.stake() as i128 + dep - backs);
    // collect asks no signature: whoever sends it, what is left goes to the staker up to its stake, and the rest to the house
    w.ok(&collect(road, authority, board), "collect, sent by anyone");
    let hb = w.balance(board, &authority);
    let back = left.min(l.stake());
    assert_eq!((hb.amount, hb.in_roads), (7 + back, 0), "the staker's balance: back by what was left, up to its stake");
    assert_eq!(earned_of(&w.data(&board_pda(board))), left - back, "the rest is the house's: the board's earned");
    assert_eq!(w.road(&road).collected, 1);
    assert_eq!(book(&w, board), 0, "the book closes with nothing left in the road");
    let house_back = back_of(l.stake() as u128, fold_pool) as i128;
    assert!((left as i128 - house_back).abs() <= 3, "what the house collected ({left}) is back_of(stake, the fold's pool result) ({house_back}) to one micro a position");
    w.refuse(&collect(road, authority, board), code(RoadsError::AlreadyCollected), "collect twice");
    eprintln!("  the house: staked {}, collected {left}, back_of(stake, pool result) {house_back}", l.stake());
    eprintln!("  the road at {fee_bp} bp: verdict {}, {} trades, {behind} refused behind and {carries} carries, pool result {fold_pool}, backs {backs} of deposits {dep}", r.verdict, entries.len());
    let fees = if world == Which::Pass { r.pass.fees } else { r.fail.fees };
    (left, fold_pool, r.verdict, fees)
}
