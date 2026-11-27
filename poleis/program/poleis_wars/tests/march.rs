mod built;
mod common;
use common::*;
use poleis_war::law::{buy_on, sell_on, Rules};
use poleis_wars::{error::WarsError, state::*, IX_CLAIM_MARCH_QUOTE, IX_CLOSE_POSITION, IX_INIT_MARCH_CONFIG, IX_LAND_ORDERS};
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

const T0: i64 = 1_800_000_000;
fn queue(raid: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[SEED_MARCH, raid.as_ref()], &PROGRAM_ID).0
}
fn queued(mut ix: Instruction, raid: Pubkey) -> Instruction {
    ix.accounts.push(AccountMeta::new(queue(raid), false));
    ix
}
fn sale(owner: Pubkey, config: Pubkey, house: Pubkey, raid: Pubkey, amount: u64, min: u64) -> Instruction {
    let mut ix = sell(owner, config, house, amount, min);
    ix.accounts.push(AccountMeta::new_readonly(system_id(), false));
    ix.accounts.push(AccountMeta::new_readonly(raid, false));
    queued(ix, raid)
}
struct Fx {
    w: World,
    config: Pubkey,
    a: Pubkey,
    b: Pubkey,
    c: Pubkey,
    d: Pubkey,
    houses: [Pubkey; 2],
    raid: Pubkey,
}
fn fx(delay: u32) -> Fx {
    fx_with_limits(delay, None)
}

fn fx_with_limits(delay: u32, limits: Option<(u64, u8)>) -> Fx {
    let mut w = World::new(T0);
    let auth = w.wallet(100 * SOL);
    let mut ix = init_config(auth, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60));
    ix.data[0] = IX_INIT_MARCH_CONFIG;
    ix.data.extend_from_slice(&delay.to_le_bytes());
    if let Some((least, cap)) = limits {
        ix.data.extend_from_slice(&least.to_le_bytes());
        ix.data.push(cap);
    }
    let cu = w.ok(&ix, "marching config");
    println!("march cost: init_march_config {cu} CU");
    let config = config_pda(&auth);
    let config_size = match (delay, limits) {
        (0, _) => Config::SIZE,
        (_, None | Some((0, 0))) => Config::MARCH_SIZE,
        _ => Config::MARCH_LIMITS_SIZE,
    };
    assert_eq!(w.data(&config).len(), config_size);
    for id in 0..2 {
        w.ok(&init_house(auth, config, id), "house");
    }
    let houses = [house_pda(&config, 0), house_pda(&config, 1)];
    let (a, b, c, d) = (w.wallet(100 * SOL), w.wallet(100 * SOL), w.wallet(100 * SOL), w.wallet(100 * SOL));
    for (owner, side, spend) in [(a, 0, 4 * SOL), (b, 0, SOL / 2), (c, 1, 4 * SOL), (d, 1, SOL / 2)] {
        w.ok(&buy(owner, config, houses[side], spend, 0, None), "before the raid");
    }
    let coins = w.position(&houses[0], &b).unwrap().coins;
    w.ok(&stake(b, config, houses[0], houses[1], coins), "open");
    let raid = raid_pda(&houses[0], &houses[1]);
    assert_eq!(w.raid(&raid).status, RAID_ACTIVE);
    Fx {
        w,
        config,
        a,
        b,
        c,
        d,
        houses,
        raid,
    }
}
fn land(f: &Fx, orders: &[(Pubkey, usize)]) -> Instruction {
    let mut metas = vec![
        AccountMeta::new_readonly(f.config, false),
        AccountMeta::new_readonly(f.raid, false),
        AccountMeta::new(f.houses[0], false),
        AccountMeta::new(f.houses[1], false),
        AccountMeta::new(fees_pda(&f.config), false),
        AccountMeta::new(queue(f.raid), false),
    ];
    for (owner, side) in orders {
        metas.push(AccountMeta::new_readonly(*owner, false));
        metas.push(AccountMeta::new(position_pda(&f.houses[*side], owner), false));
    }
    Instruction::new_with_bytes(PROGRAM_ID, &[IX_LAND_ORDERS], metas)
}
fn pending(f: &Fx) -> usize {
    f.w.data(&queue(f.raid))[March::OFF_COUNT] as usize
}
fn credit(f: &Fx, owner: Pubkey, side: usize) -> u64 {
    f.w.lamports(&position_pda(&f.houses[side], &owner)) - f.w.rent(Position::SIZE)
}
fn claim(f: &Fx, owner: Pubkey, side: usize, recipient: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        PROGRAM_ID,
        &[IX_CLAIM_MARCH_QUOTE],
        vec![
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new(position_pda(&f.houses[side], &owner), false),
            AccountMeta::new(recipient, false),
        ],
    )
}
fn accept_buy(f: &mut Fx, who: Pubkey, side: usize, spend: u64, minimum: u64) -> u64 {
    f.w.ok(&queued(buy(who, f.config, f.houses[side], spend, minimum, Some(f.raid)), f.raid), "accept buy")
}

#[test]
fn public_escrow_fifo_landing_price_and_no_overtaking() {
    let mut f = fx(10);
    let (c, d) = (f.c, f.d);
    let curve = f.w.house(&f.houses[0]).curve;
    let balance = f.w.lamports(&c);
    let cu = accept_buy(&mut f, c, 0, SOL, 0);
    println!("march cost: buy first queue and position {cu} CU");
    assert_eq!(balance - f.w.lamports(&c), SOL + f.w.rent(March::SIZE) + f.w.rent(Position::SIZE));
    assert_eq!(f.w.lamports(&queue(f.raid)), f.w.rent(March::SIZE) + SOL);
    assert_eq!(f.w.house(&f.houses[0]).curve, curve);
    assert_eq!(f.w.position(&f.houses[0], &c).unwrap().coins, 0);
    let q = f.w.data(&queue(f.raid));
    let o = March::OFF_ORDERS;
    assert_eq!(key_at(&q, o), c);
    assert_eq!(
        (q[o + 32], q[o + 33], u64_at(&q, o + 34), i64_at(&q, o + 50), i64_at(&q, o + 58)),
        (0, 0, SOL, T0, T0 + 10)
    );
    let cu = accept_buy(&mut f, d, 0, 2 * SOL, 0);
    println!("march cost: buy queue standing position born {cu} CU");
    assert_eq!(u64_at(&f.w.data(&queue(f.raid)), o + March::ORDER_SIZE + 74), 1);
    f.w.set_now(T0 + 5);
    let early = land(&f, &[(c, 0), (d, 0)]);
    f.w.refuse(&early, ProgramError::NotEnoughAccountKeys, "cannot land early");
    let empty = land(&f, &[]);
    let cu = f.w.ok(&empty, "early no-op");
    println!("march cost: land none due {cu} CU");
    // An immediate burn before arrival moves the price, proving no acceptance quote is used.
    let cu = f.w.ok(&queued(desert(f.a, f.config, f.houses[0], f.raid), f.raid), "burn before arrival");
    println!("march cost: desert guarded {cu} CU");
    let at_landing = f.w.house(&f.houses[0]).curve;
    let (after1, b1) = buy_on(&Rules::MEME_ROYALE, &at_landing, SOL as u128).unwrap();
    let (after2, b2) = buy_on(&Rules::MEME_ROYALE, &after1, 2 * SOL as u128).unwrap();
    assert_ne!(b1.coins, buy_on(&Rules::MEME_ROYALE, &curve, SOL as u128).unwrap().1.coins);
    f.w.set_now(T0 + 15); // keeper late; due orders precede every later curve change
    let cu = accept_buy(&mut f, c, 0, 3 * SOL, 0);
    println!("march cost: buy queue and position standing {cu} CU");
    f.w.refuse(
        &queued(crank(f.config, f.houses[0], f.houses[1]), f.raid),
        code(WarsError::OrdersDue),
        "decision cannot overtake",
    );
    f.w.refuse(
        &queued(desert(f.c, f.config, f.houses[1], f.raid), f.raid),
        code(WarsError::OrdersDue),
        "burn cannot overtake",
    );
    f.w.refuse(
        &queued(change_sides(f.c, f.config, f.raid, f.houses[1], f.houses[0], 0), f.raid),
        code(WarsError::OrdersDue),
        "change cannot overtake",
    );
    let reversed = land(&f, &[(d, 0), (c, 0)]);
    f.w.refuse(&reversed, code(WarsError::Unauthorized), "caller cannot reorder");
    let omitted = land(&f, &[(c, 0)]);
    f.w.refuse(&omitted, ProgramError::NotEnoughAccountKeys, "caller cannot omit a due order");
    let ix = land(&f, &[(c, 0), (d, 0)]);
    let cu = f.w.ok(&ix, "both due, in FIFO order");
    println!("march cost: land two buys {cu} CU");
    assert_eq!(f.w.house(&f.houses[0]).curve, after2);
    assert_eq!(f.w.position(&f.houses[0], &c).unwrap().coins as u128, b1.coins);
    assert_eq!(f.w.position(&f.houses[0], &d).unwrap().coins as u128, b2.coins);
    assert_eq!(pending(&f), 1);
    assert_eq!(f.w.lamports(&queue(f.raid)), f.w.rent(March::SIZE) + 3 * SOL);
    f.w.set_now(T0 + 25);
    let ix = land(&f, &[(c, 0)]);
    let cu = f.w.ok(&ix, "last arrival");
    println!("march cost: land one buy {cu} CU");
    assert_eq!(pending(&f), 0);
    assert_eq!(f.w.lamports(&queue(f.raid)), f.w.rent(March::SIZE));
}

#[test]
fn sale_is_escrowed_and_prices_after_the_earlier_buy() {
    let mut f = fx(10);
    let a = f.a;
    let old = f.w.position(&f.houses[0], &a).unwrap();
    let curve = f.w.house(&f.houses[0]).curve;
    accept_buy(&mut f, a, 0, SOL, 0);
    let coins = old.coins / 2;
    let ix = sale(a, f.config, f.houses[0], f.raid, coins, 0);
    let cu = f.w.ok(&ix, "sale accepted");
    println!("march cost: accept sale {cu} CU");
    assert_eq!(f.w.position(&f.houses[0], &a).unwrap().coins, old.coins - coins);
    assert_eq!(f.w.house(&f.houses[0]).curve, curve);
    f.w.refuse(
        &sale(a, f.config, f.houses[0], f.raid, old.coins, 0),
        code(WarsError::NotEnoughCoins),
        "escrow cannot be sold twice",
    );
    f.w.refuse(
        &sale(f.b, f.config, f.houses[0], f.raid, 1, 0),
        code(WarsError::NotEnoughCoins),
        "stake cannot march",
    );
    let (after_buy, buy_result) = buy_on(&Rules::MEME_ROYALE, &curve, SOL as u128).unwrap();
    let (after_sale, s) = sell_on(&Rules::MEME_ROYALE, &after_buy, coins as u128, 0).unwrap();
    let wallet = f.w.lamports(&a);
    f.w.set_now(T0 + 10);
    let ix = land(&f, &[(a, 0), (a, 0)]);
    let cu = f.w.ok(&ix, "buy then sale, duplicate destination");
    println!("march cost: land buy and sale {cu} CU");
    assert_eq!(f.w.house(&f.houses[0]).curve, after_sale);
    assert_eq!(f.w.lamports(&a), wallet);
    assert_eq!(credit(&f, a, 0), s.net() as u64);
    let ix = claim(&f, a, 0, a);
    let cu = f.w.ok(&ix, "claim sale proceeds");
    println!("march cost: claim quote {cu} CU");
    assert_eq!(f.w.lamports(&a) - wallet, s.net() as u64);
    assert_eq!(f.w.position(&f.houses[0], &a).unwrap().coins, old.coins - coins + buy_result.coins as u64);
}

#[test]
fn decision_refunds_both_inputs_and_settlement_waits_for_them() {
    let mut f = fx(10);
    let a = f.a;
    let end = f.w.raid(&f.raid).minutes as i64 * 60;
    f.w.set_now(T0 + end - 1);
    let before = f.w.position(&f.houses[0], &a).unwrap();
    accept_buy(&mut f, a, 0, SOL, 0);
    f.w.ok(&sale(a, f.config, f.houses[0], f.raid, before.coins / 2, 0), "late sale");
    let wallet = f.w.lamports(&a);
    f.w.set_now(T0 + end);
    let cu = f.w.ok(&queued(crank(f.config, f.houses[0], f.houses[1]), f.raid), "decision before arrival");
    println!("march cost: crank guarded {cu} CU");
    assert_eq!(f.w.raid(&f.raid).status, RAID_DECIDED);
    f.w.refuse(
        &queued(settle(f.config, f.houses[0], f.houses[1]), f.raid),
        code(WarsError::OrdersDue),
        "settlement cannot consume escrow",
    );
    let ix = land(&f, &[(a, 0), (a, 0)]);
    let cu = f.w.ok(&ix, "refund at decision");
    println!("march cost: refund buy and sale {cu} CU");
    assert_eq!(f.w.lamports(&a), wallet);
    assert_eq!(credit(&f, a, 0), SOL);
    assert_eq!(f.w.position(&f.houses[0], &a).unwrap(), before);
    let cu = f.w.ok(&queued(settle(f.config, f.houses[0], f.houses[1]), f.raid), "settle after refunds");
    println!("march cost: settle guarded {cu} CU");
}

#[test]
fn failed_limits_refund_and_queue_capacity_and_destinations_are_checked() {
    let mut f = fx(10);
    let a = f.a;
    let curve = f.w.house(&f.houses[0]).curve;
    let balance = f.w.lamports(&a);
    accept_buy(&mut f, a, 0, SOL, u64::MAX);
    let mut bad = queued(buy(a, f.config, f.houses[0], SOL, 0, Some(f.raid)), f.raid);
    bad.accounts[7].pubkey = queue(Pubkey::new_unique());
    f.w.refuse(&bad, code(WarsError::InvalidPda), "wrong queue");
    let mut unsigned = queued(buy(a, f.config, f.houses[0], SOL, 0, Some(f.raid)), f.raid);
    unsigned.accounts[0].is_signer = false;
    f.w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "acceptance must sign");
    for _ in 1..MARCH_CAPACITY {
        accept_buy(&mut f, a, 0, SOL, u64::MAX);
    }
    f.w.refuse(
        &queued(buy(a, f.config, f.houses[0], SOL, 0, Some(f.raid)), f.raid),
        code(WarsError::MarchFull),
        "full queue changes nothing",
    );
    f.w.set_now(T0 + 10);
    let ix = land(&f, &vec![(a, 0); MARCH_CAPACITY]);
    let cu = f.w.ok(&ix, "all refunds, no poisoned head");
    println!("march cost: refund eight buys {cu} CU");
    assert_eq!(f.w.house(&f.houses[0]).curve, curve);
    assert_eq!(f.w.lamports(&a) + credit(&f, a, 0), balance - f.w.rent(March::SIZE));
    assert_eq!(pending(&f), 0);
    accept_buy(&mut f, a, 0, SOL, 0);
    f.w.set_now(T0 + 20);
    let ix = land(&f, &[(a, 0)]);
    f.w.ok(&ix, "queue reused");
}

#[test]
fn change_sides_burns_now_and_its_buy_marches() {
    let mut f = fx(10);
    let src = f.w.house(&f.houses[1]);
    let dest = f.w.house(&f.houses[0]);
    let cu = f.w.ok(
        &queued(change_sides(f.c, f.config, f.raid, f.houses[1], f.houses[0], 0), f.raid),
        "change sides",
    );
    println!("march cost: change_sides burn and accept buy {cu} CU");
    assert!(f.w.house(&f.houses[1]).curve.rq < src.curve.rq);
    assert_eq!(f.w.house(&f.houses[0]).curve, dest.curve);
    assert_eq!(f.w.position(&f.houses[1], &f.c).unwrap().coins, 0);
    assert_eq!(pending(&f), 1);
    f.w.set_now(T0 + 10);
    let ix = land(&f, &[(f.c, 0)]);
    f.w.ok(&ix, "change arrives");
    assert!(f.w.position(&f.houses[0], &f.c).unwrap().coins > 0);
}

#[test]
fn zero_march_config_uses_legacy_layout_and_instructions() {
    let mut f = fx(0);
    let before = f.w.house(&f.houses[0]).curve;
    f.w.ok(&buy(f.a, f.config, f.houses[0], SOL, 0, Some(f.raid)), "immediate buy");
    assert_ne!(f.w.house(&f.houses[0]).curve, before);
    f.w.refuse(&sell(f.a, f.config, f.houses[0], 1, 0), code(WarsError::InRaid), "legacy sale refusal");
    f.w.refuse(
        &queued(buy(f.a, f.config, f.houses[0], SOL, 0, Some(f.raid)), f.raid),
        code(WarsError::MarchDisabled),
        "queue disabled",
    );
}

#[test]
fn legacy_buy_cannot_bypass_an_active_march() {
    let mut f = fx(10);
    let ix = buy(f.a, f.config, f.houses[0], SOL, 0, Some(f.raid));
    assert_eq!(ix.accounts.len(), 7);
    // World::refuse compares all account bytes and lamports, including both curves and the fees.
    f.w.refuse(&ix, ProgramError::NotEnoughAccountKeys, "the active raid requires a queue");
}

#[test]
fn all_eight_due_orders_land_below_the_instruction_budget() {
    let mut f = fx(10);
    let a = f.a;
    for _ in 0..MARCH_CAPACITY {
        accept_buy(&mut f, a, 0, SOL, 0);
    }
    f.w.set_now(T0 + 10);
    let ix = land(&f, &vec![(a, 0); MARCH_CAPACITY]);
    let cu = f.w.ok(&ix, "full queue");
    println!(
        "march cost: land eight buys {cu} CU; queue {} bytes, rent {}",
        March::SIZE,
        f.w.rent(March::SIZE)
    );
    assert!(cu < 200_000);
    assert_eq!(pending(&f), 0);
}

#[test]
fn a_drained_wallet_cannot_poison_a_landing_and_only_its_owner_claims() {
    let mut f = fx(10);
    let a = f.a;
    // 2026 pays 2025: one lamport of change cannot create a rent-exempt wallet.
    accept_buy(&mut f, a, 0, 2026, 0);
    let mut drained = f.w.account(&a);
    drained.lamports = 0;
    f.w.put(a, drained);
    f.w.set_now(T0 + 10);
    let ix = land(&f, &[(a, 0)]);
    f.w.ok(&ix, "landing needs no funded wallet");
    assert_eq!(credit(&f, a, 0), 1);
    assert_eq!(f.w.lamports(&a), 0);
    let mut wrong = claim(&f, a, 0, f.c);
    wrong.accounts[0].pubkey = f.c;
    f.w.refuse(&wrong, code(WarsError::Unauthorized), "cannot steal a credit");
    let mut unsigned = claim(&f, a, 0, f.c);
    unsigned.accounts[0].is_signer = false;
    f.w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "claim requires its owner");
    let before = f.w.lamports(&f.c);
    let ix = claim(&f, a, 0, f.c);
    f.w.ok(&ix, "owner sends the change to a funded recipient");
    assert_eq!(f.w.lamports(&f.c), before + 1);
    assert_eq!(credit(&f, a, 0), 0);
    f.w.refuse(&ix, code(WarsError::NothingOwed), "cannot claim twice");
}

#[test]
fn sale_limit_refunds_basis_and_orders_past_the_raid_window_never_trade() {
    let mut f = fx(10);
    let a = f.a;
    let before = f.w.position(&f.houses[0], &a).unwrap();
    f.w.ok(&sale(a, f.config, f.houses[0], f.raid, before.coins / 2, u64::MAX), "sale with limit");
    f.w.set_now(T0 + 10);
    let ix = land(&f, &[(a, 0)]);
    f.w.ok(&ix, "failed sale restores its basis");
    assert_eq!(f.w.position(&f.houses[0], &a).unwrap(), before);
    let last = f.w.raid(&f.raid).minutes as i64 * 60 + 59;
    f.w.set_now(T0 + last);
    let curve = f.w.house(&f.houses[0]).curve;
    accept_buy(&mut f, a, 0, SOL, 0);
    f.w.set_now(T0 + last + 10);
    let ix = land(&f, &[(a, 0)]);
    f.w.ok(&ix, "arrival outside the trade window refunds without trading");
    assert_eq!(f.w.house(&f.houses[0]).curve, curve);
    assert_eq!(credit(&f, a, 0), SOL);
    f.w.refuse(
        &buy(a, f.config, f.houses[0], SOL, 0, Some(f.raid)),
        code(WarsError::RaidOver),
        "old buy does not bypass the clock",
    );
    f.w.ok(&queued(crank(f.config, f.houses[0], f.houses[1]), f.raid), "late decision");
}

#[test]
fn march_limits_refuse_spam_and_admit_other_owners() {
    let mut f = fx_with_limits(10, Some((SOL, 1)));
    let config = f.w.data(&f.config);
    assert_eq!(u64_at(&config, Config::OFF_MARCH_MIN_ORDER), SOL);
    assert_eq!(config[Config::OFF_MARCH_MAX_PER_OWNER], 1);
    let spam = queued(buy(f.c, f.config, f.houses[0], 1, u64::MAX, Some(f.raid)), f.raid);
    for _ in 0..MARCH_CAPACITY {
        f.w.refuse(&spam, code(WarsError::OrderTooSmall), "one lamport cannot reserve a slot");
    }
    assert_eq!(f.w.lamports(&queue(f.raid)), 0);
    f.w.refuse(
        &sale(f.a, f.config, f.houses[0], f.raid, SOL - 1, 0),
        code(WarsError::OrderTooSmall),
        "a sale must reach the floor in base coin units",
    );
    let (a, c, d) = (f.a, f.c, f.d);
    let cu = accept_buy(&mut f, c, 0, SOL, u64::MAX);
    println!("march limits cost: first queued buy {cu} CU");
    f.w.refuse(
        &queued(buy(c, f.config, f.houses[1], SOL, u64::MAX, Some(f.raid)), f.raid),
        code(WarsError::OwnerMarchFull),
        "the cap covers both sides",
    );
    f.w.refuse(
        &sale(c, f.config, f.houses[1], f.raid, SOL, 0),
        code(WarsError::OwnerMarchFull),
        "changing order kind does not free a slot",
    );
    f.w.refuse(
        &queued(change_sides(c, f.config, f.raid, f.houses[1], f.houses[0], 0), f.raid),
        code(WarsError::OwnerMarchFull),
        "changing sides cannot evade the cap or burn on refusal",
    );
    let cu = accept_buy(&mut f, d, 0, SOL, 0);
    println!("march limits cost: second owner's buy {cu} CU");
    let cu = f.w.ok(&sale(a, f.config, f.houses[0], f.raid, SOL, 0), "another owner may sell at the floor");
    println!("march limits cost: sale {cu} CU");
    assert_eq!(pending(&f), 3);
    f.w.set_now(T0 + 10);
    let ix = land(&f, &[(c, 0), (d, 0), (a, 0)]);
    f.w.ok(&ix, "the refund and honest orders land");
    assert_eq!(credit(&f, c, 0), SOL);
    assert!(f.w.position(&f.houses[0], &d).unwrap().coins > 0);
    let cu = f.w.ok(
        &queued(change_sides(c, f.config, f.raid, f.houses[1], f.houses[0], 0), f.raid),
        "landing frees the owner's slot",
    );
    println!("march limits cost: change sides {cu} CU");
    assert_eq!(pending(&f), 1);
}

#[test]
fn extended_zero_limits_preserve_legacy_bytes_and_eight_lamport_capacity() {
    for delay in [0u32, 10] {
        let mut w = World::new(T0);
        let auth = w.wallet(100 * SOL);
        let legacy = init_config(auth, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60));
        let mut extended = legacy.clone();
        extended.data[0] = IX_INIT_MARCH_CONFIG;
        extended.data.extend_from_slice(&delay.to_le_bytes());
        let mut original = legacy;
        if delay > 0 {
            original = extended.clone();
        }
        extended.data.extend_from_slice(&0u64.to_le_bytes());
        extended.data.push(0);
        let before = w.accounts.clone();
        w.ok(&original, "original config wire");
        let expected = std::mem::replace(&mut w.accounts, before);
        let cu = w.ok(&extended, "extended wire with zero limits");
        println!("march limits cost: init with delay {delay}, explicit zero limits {cu} CU");
        assert_eq!(w.accounts, expected);
    }

    let mut f = fx_with_limits(10, Some((0, 0)));
    let a = f.a;
    let wallet = f.w.lamports(&a);
    for _ in 0..MARCH_CAPACITY {
        accept_buy(&mut f, a, 0, 1, u64::MAX);
    }
    assert_eq!(pending(&f), MARCH_CAPACITY);
    assert_eq!(f.w.lamports(&queue(f.raid)), f.w.rent(March::SIZE) + MARCH_CAPACITY as u64);
    let ix = queued(buy(f.d, f.config, f.houses[0], SOL, 0, Some(f.raid)), f.raid);
    f.w.refuse(&ix, code(WarsError::MarchFull), "zero limits retain the old shared cap");
    f.w.set_now(T0 + 10);
    let ix = land(&f, &vec![(a, 0); MARCH_CAPACITY]);
    f.w.ok(&ix, "all eight one-lamport inputs refund");
    assert_eq!(f.w.lamports(&a) + credit(&f, a, 0), wallet - f.w.rent(March::SIZE));

    let mut f = fx(0);
    f.w.ok(&desert(f.a, f.config, f.houses[0], f.raid), "empty a legacy position");
    assert_eq!(f.w.position(&f.houses[0], &f.a).unwrap().coins, 0);
    let ix = close(&f, f.a, 0);
    f.w.refuse(&ix, code(WarsError::MarchDisabled), "a zero-delay generation retains the old close rules");
}

fn close(f: &Fx, owner: Pubkey, side: usize) -> Instruction {
    let house = f.houses[side];
    Instruction::new_with_bytes(
        PROGRAM_ID,
        &[IX_CLOSE_POSITION],
        vec![
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(f.config, false),
            AccountMeta::new_readonly(house, false),
            AccountMeta::new(position_pda(&house, &owner), false),
            AccountMeta::new_readonly(f.raid, false),
            AccountMeta::new_readonly(queue(f.raid), false),
        ],
    )
}

#[test]
fn owner_recovers_position_rent_after_refund_and_claim() {
    let mut f = fx(10);
    let (a, b, c) = (f.a, f.b, f.w.wallet(100 * SOL));
    let ix = close(&f, c, 0);
    let coins = close(&f, a, 0);
    f.w.refuse(&coins, code(WarsError::PositionHasCoins), "coins still held");
    let stake = close(&f, b, 0);
    f.w.refuse(&stake, code(WarsError::Staked), "coins still staked");
    let wallet = f.w.lamports(&c);
    accept_buy(&mut f, c, 0, SOL, u64::MAX);
    let mut other_owner = ix.clone();
    other_owner.accounts[0].pubkey = a;
    f.w.refuse(&other_owner, code(WarsError::Unauthorized), "another signer cannot close the position");
    let mut unsigned = ix.clone();
    unsigned.accounts[0].is_signer = false;
    f.w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "the owner must sign");
    f.w.refuse(&ix, code(WarsError::PositionOrderPending), "an order is still marching");
    let mut omitted = ix.clone();
    omitted.accounts.truncate(4);
    f.w.refuse(&omitted, ProgramError::NotEnoughAccountKeys, "the current queue cannot be omitted");
    let mut wrong_queue = ix.clone();
    wrong_queue.accounts[5].pubkey = queue(Pubkey::new_unique());
    f.w.refuse(&wrong_queue, code(WarsError::InvalidPda), "an empty substitute queue cannot be used");
    f.w.set_now(T0 + 10);
    let arrival = land(&f, &[(c, 0)]);
    f.w.ok(&arrival, "failed buy refunds");
    assert_eq!(credit(&f, c, 0), SOL);
    f.w.refuse(&ix, code(WarsError::PositionHasQuote), "claim the refund first");
    let refund = claim(&f, c, 0, c);
    f.w.ok(&refund, "claim the refund");
    accept_buy(&mut f, c, 1, SOL, u64::MAX);
    let cu = f.w.ok(&ix, "the same owner's other position may have an order");
    println!("march limits cost: close during a raid {cu} CU");
    assert!(f.w.position(&f.houses[0], &c).is_none());
    assert_eq!(f.w.lamports(&position_pda(&f.houses[0], &c)), 0);
    assert_eq!(f.w.lamports(&c), wallet - f.w.rent(March::SIZE) - f.w.rent(Position::SIZE) - SOL);
    assert_eq!(pending(&f), 1);
    f.w.set_now(T0 + 20);
    let arrival = land(&f, &[(c, 1)]);
    f.w.ok(&arrival, "the other position still receives its refund");
    let refund = claim(&f, c, 1, c);
    f.w.ok(&refund, "claim the other refund");
    let end = f.w.raid(&f.raid).minutes as i64 * 60;
    f.w.set_now(T0 + end);
    f.w.ok(&queued(crank(f.config, f.houses[0], f.houses[1]), f.raid), "decide");
    f.w.ok(&queued(settle(f.config, f.houses[0], f.houses[1]), f.raid), "settle");
    let mut ix = close(&f, c, 1);
    ix.accounts.truncate(4);
    let cu = f.w.ok(&ix, "close after settlement");
    println!("march limits cost: close after settlement {cu} CU");
    assert!(f.w.position(&f.houses[1], &c).is_none());
    assert_eq!(f.w.lamports(&c), wallet - f.w.rent(March::SIZE));
}
