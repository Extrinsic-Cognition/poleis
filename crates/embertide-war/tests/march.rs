use embertide_war::{law::*, war::War};

fn war() -> War {
    let mut w = War::new(Rules {
        march_s: 10,
        ..Rules::MEME_ROYALE
    });
    for (who, side, amount) in [
        ("a", Side::Attacker, 4 * SOL),
        ("b", Side::Attacker, SOL / 2),
        ("c", Side::Defender, 4 * SOL),
        ("d", Side::Defender, SOL / 2),
    ] {
        w.enter(who, side, amount).unwrap();
    }
    assert!(w.stake("b").unwrap());
    w
}

#[test]
fn visibility_escrow_landing_price_fifo_and_no_early_landing() {
    let mut w = war();
    w.fund("first", SOL).unwrap();
    w.fund("later", 2 * SOL).unwrap();
    let before = w.curves[0];
    w.enter_at("first", Side::Attacker, SOL, 0, 1).unwrap();
    w.enter_at("later", Side::Attacker, 2 * SOL, 0, 2).unwrap();
    assert_eq!(w.wallets["first"], 0);
    assert_eq!(w.curves[0], before);
    assert_eq!(
        (w.orders[0].who.as_str(), w.orders[0].order.accepted_at, w.orders[0].order.lands_at),
        ("first", 1, 11)
    );
    // His burn is immediate, before either arrival: the acceptance quote is obsolete.
    w.desert_at("a", 5).unwrap();
    assert!(w.land(10).unwrap().is_empty());
    let at_landing = w.curves[0];
    let (after_first, first) = buy_on(&w.rules, &at_landing, SOL).unwrap();
    let (after_later, later) = buy_on(&w.rules, &after_first, 2 * SOL).unwrap();
    assert_ne!(first.coins, buy_on(&w.rules, &before, SOL).unwrap().1.coins);
    assert_eq!(w.land(20).unwrap(), vec![Landing::Buy(first), Landing::Buy(later)]);
    assert_eq!(w.curves[0], after_later);
    assert_eq!(w.coins[&("first".into(), Side::Attacker)], first.coins);
    assert!(w.orders.is_empty());
}

#[test]
fn sales_reserve_coins_once_and_land_after_earlier_buys() {
    let mut w = war();
    let key = ("a".into(), Side::Attacker);
    let held = w.coins[&key];
    let basis = w.brought[&key];
    let coins = held / 2;
    w.fund("buyer", SOL).unwrap();
    w.enter_at("buyer", Side::Attacker, SOL, 0, 0).unwrap();
    w.sell_at("a", Side::Attacker, coins, 0, 0).unwrap();
    assert_eq!(w.coins[&key], held - coins);
    assert_eq!(w.brought[&key] + w.orders[1].brought, basis);
    assert!(w.sell_at("a", Side::Attacker, held, 0, 0).is_err());
    let (c, b) = buy_on(&w.rules, &w.curves[0], SOL).unwrap();
    let (c, s) = sell_on(&w.rules, &c, coins, 0).unwrap();
    assert_eq!(w.land(10).unwrap(), vec![Landing::Buy(b), Landing::Sale(s)]);
    assert_eq!(w.curves[0], c);
    assert_eq!(w.wallets["a"], s.net());
}

#[test]
fn decision_refunds_marching_inputs_before_the_settlement() {
    let mut w = war();
    let t = w.opening.unwrap().minutes;
    let at = t as i64 * 60;
    let key = ("a".into(), Side::Attacker);
    let (coins, brought) = (w.coins[&key], w.brought[&key]);
    w.fund("late", SOL).unwrap();
    w.enter_at("late", Side::Attacker, SOL, 0, at - 1).unwrap();
    w.sell_at("a", Side::Attacker, coins / 2, 0, at - 1).unwrap();
    assert!(matches!(w.minute(t).unwrap(), Phase::Decided { .. }));
    assert!(w.orders.is_empty());
    assert_eq!((w.coins[&key], w.brought[&key]), (coins, brought));
    assert_eq!(w.wallets["late"], SOL);
    w.settle().unwrap();
}

#[test]
fn price_failure_refunds_and_does_not_block_the_next_order() {
    let mut w = war();
    w.fund("limit", SOL).unwrap();
    w.fund("next", SOL).unwrap();
    w.enter_at("limit", Side::Attacker, SOL, u128::MAX, 0).unwrap();
    w.enter_at("next", Side::Attacker, SOL, 0, 0).unwrap();
    let expected = buy_on(&w.rules, &w.curves[0], SOL).unwrap().1;
    assert_eq!(w.land(10).unwrap(), vec![Landing::Refund, Landing::Buy(expected)]);
    assert_eq!(w.wallets["limit"], SOL);
}

#[test]
fn acceptance_is_checked_and_zero_is_the_default() {
    assert_eq!(Rules::default().march_s, 0);
    let r = Rules {
        march_s: 10,
        ..Rules::MEME_ROYALE
    };
    assert_eq!(Order::accept(&r, OrderKind::Buy, SOL, 0, i64::MAX, None), Err(OVERFLOW));
    assert_eq!(Order::accept(&r, OrderKind::Buy, SOL, 0, 1, Some(2)), Err(ORDER_TIME));
    let order = Order::accept(&r, OrderKind::Buy, SOL, 0, 0, None).unwrap();
    assert_eq!(land_on(&r, &Curve::born(&r), &order, 9, false, 0), Err(ORDER_EARLY));
    let mut w = war();
    w.fund("many", 9 * SOL).unwrap();
    for _ in 0..MARCH_CAPACITY {
        w.enter_at("many", Side::Attacker, SOL, 0, 0).unwrap();
    }
    assert_eq!(w.enter_at("many", Side::Attacker, SOL, 0, 0), Err(MARCH_FULL));
    assert_eq!(w.wallets["many"], SOL);
}

#[test]
fn exact_instant_reads_cannot_rewind_and_the_winner_trades_after_settlement() {
    let mut w = war();
    w.fund("buyer", SOL).unwrap();
    w.enter_at("buyer", Side::Attacker, SOL, 0, 1).unwrap();
    w.minute_at(11).unwrap();
    assert!(w.orders.is_empty());
    assert_eq!(w.land(10), Err(ORDER_TIME));
    assert_eq!(w.enter_at("buyer", Side::Attacker, 1, 0, 10), Err(ORDER_TIME));
    let end = w.opening.unwrap().minutes;
    w.minute(end).unwrap();
    if !matches!(w.phase, Phase::Decided { .. }) {
        w.minute(end + 2).unwrap();
    }
    let d = w.settle().unwrap();
    let before = w.curves[d.winner.index()];
    w.enter("after", d.winner, SOL).unwrap();
    assert_ne!(w.curves[d.winner.index()], before);
}

#[test]
fn timed_entry_points_keep_their_limits_when_a_trade_is_immediate() {
    let mut w = War::new(Rules::default());
    let curve = w.curves[0];
    assert!(w.enter_at("a", Side::Attacker, SOL, u128::MAX, 0).is_err());
    assert_eq!(w.curves[0], curve);
    w.enter_at("a", Side::Attacker, SOL, 0, 0).unwrap();
    let curve = w.curves[0];
    let coins = w.coins[&("a".into(), Side::Attacker)];
    assert!(w.sell_at("a", Side::Attacker, coins, u128::MAX, 0).is_err());
    assert_eq!(w.curves[0], curve);
    assert_eq!(w.coins[&("a".into(), Side::Attacker)], coins);
}

#[test]
fn march_limits_apply_across_sides_and_kinds_until_landing() {
    let mut w = war();
    assert_eq!(w.rules.march_min_order, 0);
    assert_eq!(w.rules.march_max_per_owner, 0);
    w.rules.march_min_order = SOL as u64;
    w.rules.march_max_per_owner = 1;
    w.fund("c", 2 * SOL).unwrap();
    let before = w.curves;
    assert_eq!(w.enter_at("c", Side::Attacker, 1, u128::MAX, 0), Err(ORDER_TOO_SMALL));
    assert_eq!(w.wallets["c"], 2 * SOL);
    assert!(w.orders.is_empty());
    w.enter_at("c", Side::Attacker, SOL, u128::MAX, 0).unwrap();
    assert_eq!(w.enter_at("c", Side::Defender, SOL, 0, 0), Err(OWNER_MARCH_FULL));
    assert_eq!(w.sell_at("c", Side::Defender, SOL, 0, 0), Err(OWNER_MARCH_FULL));
    assert_eq!(w.cross_at("c", Side::Defender, 0), Err(OWNER_MARCH_FULL));
    assert_eq!(w.curves, before);
    assert_eq!(w.sell_at("a", Side::Attacker, SOL - 1, 0, 0), Err(ORDER_TOO_SMALL));
    w.sell_at("a", Side::Attacker, SOL, 0, 0).unwrap();
    assert!(matches!(w.land(10).unwrap().as_slice(), [Landing::Refund, Landing::Sale(_)]));
    w.enter_at("c", Side::Defender, SOL, 0, 10).unwrap();
    assert_eq!(w.orders.len(), 1);
}
