//! Each piece of the law alone. The expected integers are the model's own (tests/reference/war-rules-sim.py and meme-royale-rulebook-sim.py,
//! run on the same inputs with Python's exact integers), or are worked out in the comment beside them.
use embertide_war::law::*;
use std::cmp::Ordering;

const R: Rules = Rules::MEME_ROYALE;

fn fresh() -> Curve {
    Curve::born(&R)
}
/// the crow's curve after the forty came in, one SOL each, twenty a side (the model's state when the stake begins)
fn after_forty() -> Curve {
    let mut c = fresh();
    for _ in 0..20 {
        c = buy_on(&R, &c, SOL).unwrap().0;
    }
    c
}
/// xorshift: a seeded sweep with no dependency
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

#[test]
fn the_rules_of_the_model_are_rules_the_law_carries_and_each_refusal_is_its_own() {
    assert_eq!(R.check(), Ok(()));
    let bad = [
        Rules { rt: 0, ..R },
        Rules { rt: R.vt, ..R },
        Rules { vq: 0, ..R },
        Rules { vt: AMOUNT_MAX + 1, rt: 1, ..R },
        Rules { vq: AMOUNT_MAX / 2, ..R },
        Rules { protocol_bps: 9_000, creator_bps: 1_000, ..R },
        Rules { stake_own_pm: 300, ..R },
        Rules { stake_max_pm: 1_001, ..R },
        Rules { opponent_min_pm: 1_001, ..R },
        Rules { opponent_max_pm: 999, ..R },
        Rules { raid_min_s: 59, ..R },
        Rules { raid_min_s: 30_000, ..R },
        Rules { raid_step_s: 0, ..R },
        Rules { stand_minutes: 0, ..R },
        Rules { revival_bps: 10_000, ..R },
        Rules { keep_bps: 10_001, ..R },
        Rules { dump_min_s: 4_000, ..R },
        Rules { dump_step_s: 0, ..R },
    ];
    for b in bad {
        assert!(b.check().is_err(), "refused: {b:?}");
    }
}

#[test]
fn a_fee_is_a_ceiling() {
    assert_eq!(ceil_fee(0, 95), Ok(0));
    assert_eq!(ceil_fee(1, 95), Ok(1));
    assert_eq!(ceil_fee(10_000, 95), Ok(95));
    assert_eq!(ceil_fee(10_001, 95), Ok(96));
    assert_eq!(ceil_fee(987_654_320, 95), Ok(9_382_717));
}

#[test]
fn a_buy_and_its_sale_are_pump_funs() {
    // the model's `_buy(Curve(VQ_SOL), SOL)`: (34199203120618, 987654320, 9382717, 2962963)
    let (c, b) = buy_on(&R, &fresh(), SOL).unwrap();
    assert_eq!(b, Buy { coins: 34_199_203_120_618, quote: 987_654_320, protocol: 9_382_717, creator: 2_962_963 });
    assert_eq!(b.paid(), SOL);
    assert_eq!(c, Curve { vt: 1_038_800_796_879_382, vq: 30_987_654_320, rt: 758_900_796_879_382, rq: 987_654_320, rt0: R.rt });
    assert_eq!(c.vq0(), R.vq, "a buy moves vq and rq by the same amount");
    // the model's `_sell` of those coins back: (987654319, 9382717, 2962963), and the curve keeps one lamport of the round trip
    let (c2, s) = sell_on(&R, &c, b.coins, 0).unwrap();
    assert_eq!(s, Sale { quote: 987_654_319, protocol: 9_382_717, creator: 2_962_963, tax: 0 });
    assert_eq!(s.net(), 987_654_319 - 9_382_717 - 2_962_963);
    assert_eq!(c2, Curve { vt: R.vt, vq: 30_000_000_001, rt: R.rt, rq: 1, rt0: R.rt });
}

#[test]
fn what_a_buy_pays_is_never_more_than_its_spend_and_at_most_a_lamport_less() {
    // 1014: raw 1001, the fees 10 and 4, 1015 in all: the quote gives up the lamport over, so 1014 is paid
    let (_, b) = buy_on(&R, &fresh(), 1_014).unwrap();
    assert_eq!((b.quote, b.protocol, b.creator, b.paid()), (1_000, 10, 4, 1_014));
    // 2026: raw 2000, the fees exactly 19 and 6: 2025 is paid and one lamport stays with the buyer
    let (_, b) = buy_on(&R, &fresh(), 2_026).unwrap();
    assert_eq!((b.quote, b.protocol, b.creator, b.paid()), (2_000, 19, 6, 2_025));
    // a quote of nothing buys nothing
    for spend in [0, 1, 2] {
        let (c, b) = buy_on(&R, &fresh(), spend).unwrap();
        assert_eq!((b.coins, b.quote), (0, 0), "a spend of {spend}");
        assert_eq!(c, Curve { rq: 0, ..fresh() });
    }
    // over a seeded sweep: paid is the spend or a lamport less, and a buy never takes the curve's last coin twice
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut c = fresh();
    for _ in 0..2_000 {
        let spend = 3 + rng.below(3 * SOL as u64) as u128;
        match buy_on(&R, &c, spend) {
            Ok((after, b)) => {
                assert!(b.paid() == spend || b.paid() + 1 == spend, "spend {spend} paid {}", b.paid());
                assert_eq!(after.vq0(), R.vq);
                c = after;
            }
            Err(e) => {
                assert_eq!(e, NOT_ENOUGH_TOKENS);
                break;
            }
        }
    }
}

#[test]
fn a_buy_past_the_coins_left_is_refused() {
    let c = Curve { rt: 1_000, ..fresh() };
    assert_eq!(buy_on(&R, &c, SOL).unwrap_err(), NOT_ENOUGH_TOKENS);
}

#[test]
fn a_sale_is_refused_when_it_gives_nothing_sells_unsold_coins_or_passes_the_reserve() {
    let (c, b) = buy_on(&R, &fresh(), SOL).unwrap();
    assert_eq!(sell_on(&R, &c, 0, 0).unwrap_err(), NOTHING_OUT);
    assert_eq!(sell_on(&R, &c, b.coins + 1, 0).unwrap_err(), NOT_SOLD);
    // one coin fetches a quote of nothing
    assert_eq!(sell_on(&R, &c, 1, 0).unwrap_err(), NOTHING_OUT);
    // a house whose reserve was paid out to a deserter cannot pay the whole sale of the coins left
    let short = Curve { rq: 1_000, vq: c.vq - (c.rq - 1_000), ..c };
    assert_eq!(sell_on(&R, &short, b.coins / 2, 0).unwrap_err(), RESERVE_SHORT);
    // the dump tax: its floor share of the quote, taken beside the fees
    let (_, s) = sell_on(&R, &c, b.coins, 1_990).unwrap();
    assert_eq!(s.tax, 987_654_319 * 1_990 / 10_000);
    assert_eq!(s.net(), 987_654_319 - 9_382_717 - 2_962_963 - s.tax);
    assert_eq!(tax_split(s.tax), (s.tax / 2, s.tax - s.tax / 2));
    assert_eq!(tax_split(7), (3, 4));
}

#[test]
fn the_market_caps_and_the_line() {
    let (c, _) = buy_on(&R, &fresh(), SOL).unwrap();
    // the model's mcap and mcap_at after one buy of a SOL
    assert_eq!(mcap(&c), Ok(1_020_169_687));
    assert_eq!(mcap_at(&c, 2 * SOL), Ok(2_133_333_333));
    assert_eq!(mcap(&fresh()), Ok(0));
    // M_GRAD: the exact rational vq * vt * rt / (vt - rt)^2, floored once (the reserve floored first would give 325869061330)
    assert_eq!(grad_mcap(&R), Ok(325_869_061_336));
}

#[test]
fn the_forty_and_the_stake_that_opens_the_raid() {
    let c = after_forty();
    // the model's crow after the forty: its curve, its market cap, its raid's length (both houses are alike)
    assert_eq!(c, Curve { vt: 646_995_037_804_867, vq: 49_753_086_400, rt: 367_095_037_804_867, rq: 19_753_086_400, rt0: R.rt });
    assert_eq!(mcap(&c), Ok(32_759_233_769));
    assert_eq!(raid_minutes(&R, 2 * c.sold()), Ok(195));
    assert_eq!(raid_minutes(&R, 0), Ok(15), "the shortest raid");
    assert_eq!(raid_minutes(&R, 2 * R.rt), Ok(360), "the longest: both houses sold out");
    assert_eq!(stake_need(&R, 32_759_233_769, 32_759_233_769), Ok(1_637_961_688));
    assert_eq!(stake_need(&R, 100, 10_000), Ok(198), "2% of the gap when it is the larger");
    // crow19 alone (13103696946871 coins) does not reach the need; crow18 with him (13645451858788 more) does
    assert_eq!(stake_on(&R, &c, &c, 13_103_696_946_871), Ok(None));
    let o = stake_on(&R, &c, &c, 13_103_696_946_871 + 13_645_451_858_788).unwrap().unwrap();
    assert_eq!(o, Opening { m0: [32_759_233_769, 32_759_233_769], need: 1_637_961_688, minutes: 195 });
    // the cap: 20% of the market cap at this price is 20% * 32759233769 * vt / vq coins
    let cap = 32_759_233_769u128 * 200 * c.vt / (c.vq * 1_000);
    assert!(stake_on(&R, &c, &c, cap).is_ok());
    assert_eq!(stake_on(&R, &c, &c, cap + 1), Err(STAKE_CAP));
    // the opponent: between half and twice, both ends taken
    let small = {
        let mut d = fresh();
        for _ in 0..10 {
            d = buy_on(&R, &d, SOL).unwrap().0;
        }
        d
    };
    let ratio = mcap(&small).unwrap() as f64 / mcap(&c).unwrap() as f64;
    assert!(ratio < 0.5, "ten SOL against twenty: {ratio}");
    assert_eq!(stake_on(&R, &c, &small, 1), Err(UNFAIR));
    assert_eq!(stake_on(&R, &small, &c, 1), Err(UNFAIR), "and the other way round, twice and more");
}

#[test]
fn his_burn_pays_the_least_of_the_worth_his_half_and_the_reserve() {
    let c = after_forty();
    // crow0, first in: his coins are worth 2466619441 after the fees; his half of one SOL binds
    let (after, paid) = burn_on(&R, &c, 34_199_203_120_618, SOL).unwrap();
    assert_eq!(paid, 500_000_000);
    assert_eq!(after, Curve { vq: c.vq - paid, rq: c.rq - paid, ..c }, "vt and rt stay: the coins are burned");
    // crow19, last in, brought ten SOL: his coins' worth binds (987654319 less the fees)
    let (_, paid) = burn_on(&R, &c, 13_103_696_946_871, 10 * SOL).unwrap();
    assert_eq!(paid, 975_308_639);
    // the reserve binds
    let (after, paid) = burn_on(&R, &Curve { rq: 7, vq: c.vq - c.rq + 7, ..c }, 34_199_203_120_618, SOL).unwrap();
    assert_eq!((paid, after.rq), (7, 0));
    // nothing for nothing
    assert_eq!(burn_on(&R, &c, 0, SOL).unwrap().1, 0);
    assert_eq!(burn_on(&R, &c, 1, SOL).unwrap().1, 0, "a coin worth less than its fees");
}

fn rd(delta: i128, gap: u128) -> Reading {
    Reading { now: 0, line: 0, delta, gap }
}

#[test]
fn the_line_is_met_strictly_and_the_revival_is_a_ninth_more() {
    // delta * t > gap * (t - m): at t 100 and m 50, a gap of 10 needs a climb of more than 5
    assert_eq!(meets(&rd(5, 10), 100, 50), Ok(false), "level is not past");
    assert_eq!(meets(&rd(6, 10), 100, 50), Ok(true));
    assert_eq!(meets(&rd(0, 0), 100, 50), Ok(false), "no climb never meets");
    assert_eq!(meets(&rd(-5, 0), 100, 50), Ok(false));
    assert_eq!(meets(&rd(1, 1_000), 100, 100), Ok(true), "at the end the line is nothing");
    assert_eq!(meets(&rd(1, 1_000), 100, 130), Ok(true), "and past it");
    // the revival: delta * t * 9000 > gap * (t - m) * 10000: a climb of 6 meets a gap of 10 at half time, and revives only past 50/9
    assert_eq!(revives(&R, &rd(6, 10), 100, 50), Ok(true), "6 * 9 = 54 > 50");
    assert_eq!(revives(&R, &rd(5, 9), 100, 50), Ok(false), "5 * 9 = 45, 9 * 5 = 45: level");
    assert_eq!(meets(&rd(51, 100), 100, 50), Ok(true));
    assert_eq!(revives(&R, &rd(51, 100), 100, 50), Ok(false), "met, but not a ninth more: 51 * 9 = 459 < 500");
    assert_eq!(revives(&R, &rd(56, 100), 100, 50), Ok(true), "56 * 9 = 504 > 500");
}

#[test]
fn further_along_is_the_climb_over_its_own_start_and_a_tie_is_level() {
    let (a, d) = (rd(10, 0), rd(20, 0));
    assert_eq!(further(&a, 100, &d, 200), Ok(Ordering::Equal), "10/100 against 20/200");
    assert_eq!(further(&a, 99, &d, 200), Ok(Ordering::Greater));
    assert_eq!(further(&rd(-1, 0), 100, &rd(-2, 0), 100), Ok(Ordering::Greater), "the one that fell less is further along");
}

/// two curves of the reference's size, the attacker pushed by `push` SOL more
fn pushed(push: u32) -> [Curve; 2] {
    let c = after_forty();
    let mut a = c;
    for _ in 0..push {
        a = buy_on(&R, &a, SOL).unwrap().0;
    }
    [a, c]
}

#[test]
fn a_minute_moves_the_phase_as_the_models_loop_does() {
    let c = after_forty();
    let m0 = [mcap(&c).unwrap(); 2];
    let t = 195;
    // nobody moved: nothing happens before the end, and at the end the clock gives a tie to the defender
    let p = minute_on(&R, Phase::Open, t, 10, [&c, &c], m0, 0).unwrap();
    assert_eq!(p, Phase::Open);
    let p = minute_on(&R, Phase::Open, t, t, [&c, &c], m0, 0).unwrap();
    assert_eq!(p, Phase::Decided { winner: Side::Defender, how: How::Clock { tie: true }, at: t });
    // a read late, past the end, is the end's
    let p = minute_on(&R, Phase::Open, t, t + 9, [&c, &c], m0, 0).unwrap();
    assert_eq!(p, Phase::Decided { winner: Side::Defender, how: How::Clock { tie: true }, at: t });
    // the attacker pushed twenty SOL beats its line at minute 60 (the reference's second war): the defender is put in its last stand
    let [a, d] = pushed(20);
    assert_eq!(minute_on(&R, Phase::Open, t, 59, [&a, &d], m0, 0), Ok(Phase::Open), "at 59 the line is still above it");
    let p = minute_on(&R, Phase::Open, t, 60, [&a, &d], m0, 0).unwrap();
    assert_eq!(p, Phase::Stand { side: Side::Defender, met: 60, ends: 62 });
    assert_eq!(trades_until(p, t), Some(62));
    assert_eq!(minute_on(&R, p, t, 61, [&a, &d], m0, 0), Ok(p), "the stand's first minute: no revival yet");
    let done = Phase::Decided { winner: Side::Attacker, how: How::StandFailed { met: 60 }, at: 62 };
    assert_eq!(minute_on(&R, p, t, 62, [&a, &d], m0, 0), Ok(done), "its last minute: it failed");
    assert_eq!(minute_on(&R, p, t, 90, [&a, &d], m0, 0), Ok(done), "a stand read late is read at its last minute");
    assert_eq!(minute_on(&R, done, t, 91, [&a, &d], m0, 0), Err(DECIDED));
    assert_eq!(trades_until(done, t), None);
    // the defender answers in its stand: the least answer that revives it, and one SOL less that does not
    let revived = |k: u32| {
        let [d2, _] = pushed(k);
        minute_on(&R, p, t, 61, [&a, &d2], m0, 0).unwrap()
    };
    let k = (20..200).find(|k| revived(*k) != p).expect("an answer revives it");
    assert_eq!(revived(k), Phase::Decided { winner: Side::Defender, how: How::Revived, at: 61 });
    assert_eq!(revived(k - 1), p, "one SOL less leaves it in its stand");
    eprintln!("  against a push of 20, the defender revives at minute 61 with {k} SOL and not with {}", k - 1);
    // both beat their lines in one minute: the further along wins, and on a tie the defender
    let both = |ka: u32, kd: u32| {
        let ([x, _], [y, _]) = (pushed(ka), pushed(kd));
        minute_on(&R, Phase::Open, t, 190, [&x, &y], m0, 0).unwrap()
    };
    assert_eq!(both(20, 21), Phase::Decided { winner: Side::Defender, how: How::BothMet, at: 190 });
    assert_eq!(both(21, 20), Phase::Decided { winner: Side::Attacker, how: How::BothMet, at: 190 });
    assert_eq!(both(20, 20), Phase::Decided { winner: Side::Defender, how: How::BothMet, at: 190 }, "a tie goes to the defender");
}

#[test]
fn the_settlement_buys_what_the_winners_curve_can_take_and_the_chest_keeps_the_rest() {
    // the most of a pot the curve can take: held against a walk over every x of a small curve with few coins left
    let small = Curve { vt: 10_000_000, vq: 3_000_000, rt: 5_000, rq: 0, rt0: 5_000 };
    let fits = |x: u128| {
        let raw = x * 10_000 / 10_125;
        raw == 0 || (raw - 1) * small.vt / (small.vq + raw - 1) <= small.rt
    };
    let walked = (0..10_000u128).take_while(|x| fits(*x)).last().unwrap();
    assert_eq!(bought_with(&R, &small, 9_999), Ok(walked));
    let (after, s) = settle_on(&R, &small, 9_999).unwrap();
    assert_eq!(s.bought_with, walked);
    assert!(s.buy.coins <= small.rt && after.rt == small.rt - s.buy.coins);
    assert_eq!(s.to_chest, 9_999 - s.buy.paid(), "the rest of the pot, and the lamport a buy may leave, go to the chest");
    // a pot the curve takes whole, with the lamport a spend of 2026 leaves: it goes to the chest
    let (_, s) = settle_on(&R, &fresh(), 2_026).unwrap();
    assert_eq!((s.bought_with, s.buy.paid(), s.to_chest), (2_026, 2_025, 1));
    // an empty pot buys nothing
    let (after, s) = settle_on(&R, &fresh(), 0).unwrap();
    assert_eq!((after, s.buy.coins, s.to_chest), (fresh(), 0, 0));
}

#[test]
fn the_shares_scaled_by_a_hundred_are_the_models_elevens_and_tens() {
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    for _ in 0..500 {
        let n = 1 + rng.below(30) as usize;
        let holders: Vec<(u128, bool)> = (0..n).map(|_| (1 + rng.below(1 << 50) as u128, rng.below(4) == 0)).collect();
        let amount = rng.below(1 << 50) as u128;
        let held: u128 = holders.iter().map(|h| h.0).sum();
        let staked: u128 = holders.iter().filter(|h| h.1).map(|h| h.0).sum();
        let model_sum: u128 = holders.iter().map(|(t, s)| t * if *s { 11 } else { 10 }).sum();
        let ws = weights_of(&R, held, staked).unwrap();
        assert_eq!(ws, model_sum * 100);
        for (t, s) in &holders {
            let model = amount * t * if *s { 11 } else { 10 } / model_sum;
            assert_eq!(share_of(amount, weight_of(&R, *t, if *s { *t } else { 0 }).unwrap(), ws), Ok(model));
        }
    }
    assert_eq!(share_of(5, 1, 0), Ok(0), "nobody holds: nobody's share");
    assert_eq!(backed_of(1_000, 1, 3), Ok(333));
    assert_eq!(backed_of(1_000, 1, 0), Ok(0));
}

#[test]
fn the_dump_tax_starts_at_the_raids_share_and_lasts_by_its_gain() {
    // the three winners of the reference: (m0, mcap_end) -> (rate, seconds), worked with M_grad 325869061336
    assert_eq!(dump_tax_of(&R, 32_759_233_769, 40_898_236_908), Ok(Some(DumpTax { rate_bps: 1_990, seconds: 180 })));
    assert_eq!(dump_tax_of(&R, 32_759_233_769, 91_530_762_307), Ok(Some(DumpTax { rate_bps: 6_420, seconds: 1_320 })));
    assert_eq!(dump_tax_of(&R, 32_759_233_769, 58_893_461_192), Ok(Some(DumpTax { rate_bps: 4_437, seconds: 600 })));
    // the least rate: a raid that added 1% starts at 10%, and lasts the shortest
    assert_eq!(dump_tax_of(&R, 100_000_000_000, 101_000_000_000), Ok(Some(DumpTax { rate_bps: 1_000, seconds: 60 })));
    // no gain, no tax; a gain of M_grad (twice its half) lasts 60 + 2 * 3540 seconds: no cap
    assert_eq!(dump_tax_of(&R, 5, 5), Ok(None));
    assert_eq!(dump_tax_of(&R, 5, 4), Ok(None));
    let g = grad_mcap(&R).unwrap();
    assert_eq!(dump_tax_of(&R, 1, 1 + g).unwrap().unwrap().seconds, 60 + 2 * 3_540);
    // it falls in a straight line
    assert_eq!(dump_rate_at(1_990, 1_000, 1_180, 999), 1_990);
    assert_eq!(dump_rate_at(1_990, 1_000, 1_180, 1_000), 1_990);
    assert_eq!(dump_rate_at(1_990, 1_000, 1_180, 1_090), 995);
    assert_eq!(dump_rate_at(1_990, 1_000, 1_180, 1_179), 11);
    assert_eq!(dump_rate_at(1_990, 1_000, 1_180, 1_180), 0);
    assert_eq!(dump_rate_at(1_990, 1_000, 1_000, 1_000), 0);
}

#[test]
fn what_he_brought_falls_with_what_he_sells_and_a_burn_ends_it() {
    // a partial sale takes its share of what he brought out with it, floored
    assert_eq!(brought_after_sale(SOL, 1_000, 500), Ok(SOL / 2), "half sold, half of it still in");
    assert_eq!(brought_after_sale(10 * SOL, 3, 1), Ok(10 * SOL * 2 / 3));
    assert_eq!(brought_after_sale(7, 3, 1), Ok(4), "7 * 2 / 3, floored");
    assert_eq!(brought_after_sale(10 * SOL, 1_000, 1_000), Ok(0), "all of it sold: nothing still in");
    assert_eq!(brought_after_sale(10 * SOL, 1_000, 0), Ok(10 * SOL), "nothing sold, all still in");
    assert_eq!(brought_after_sale(5, 0, 0), Ok(0), "nothing held has brought nothing");
    assert_eq!(brought_after_sale(5, 10, 11), Err(NOT_HELD), "a sale of more than he holds");
    // the hole it closes: ten SOL in, all of it sold, one SOL back in: his half is half of the one
    let (c, b) = buy_on(&R, &after_forty(), 10 * SOL).unwrap();
    let (c, _) = sell_on(&R, &c, b.coins, 0).unwrap();
    let left = brought_after_sale(b.paid(), b.coins, b.coins).unwrap();
    let (c, b1) = buy_on(&R, &c, SOL).unwrap();
    let (_, paid) = burn_on(&R, &c, b1.coins, left + b1.paid()).unwrap();
    assert!(paid <= SOL / 2, "deserting, he is paid at most half of the one SOL still in: {paid}");
    assert_eq!(paid, SOL / 2, "and his half binds");
}
