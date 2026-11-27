//! THE TEXT IS THE WITNESS, ONE STEP LATER: tools/embertide-mesh/futarchy/futarchy-law.mjs (Oct 30 2026, floats) answered 24 markets whose trades
//! stand on the grid; its answers are kept as data (tests/witness.txt). Off the grid the two differ on purpose (law.rs THE GRID; tests/law.rs the
//! spam). On it they differ by one step, on purpose too (law.rs THE PRICE THAT STOOD): a trade on a boundary counted, in the text, for the step that
//! ended as it came, and here counts from its own stamp. With no trade at the opening and a window of whole steps, this law's observation over a
//! step is exactly the text's over the step before, so the sum the text kept is `cum - open * step + acc` at the close: the first step, at the
//! opening price, leaves, and the step the next boundary would have taken, which `acc` holds whole at the close, enters. Of those two sums: the
//! text's averages within a millionth, its verdict by the threshold, and its scores within a hundredth of a $WARM; the spots and the counts as they
//! stood.
#![cfg(feature = "fold")]
use embertide_futarchy::law::*;

fn near(mine: u128, text: u128, what: &str, road: &str) { let gap = mine.abs_diff(text); assert!(gap * 1_000_000 <= text.max(1), "{road}: {what} {mine} against the text's {text}"); }

#[test]
fn on_the_grid_the_law_answers_as_the_text_did() {
    let text = include_str!("witness.txt");
    let (mut roads, mut passed, mut moved) = (0, 0, 0);
    let mut lines = text.lines().filter(|l| !l.starts_with('#')).peekable();
    while let Some(l) = lines.next() {
        let w: Vec<&str> = l.split_whitespace().collect();
        assert_eq!(w[0], "road", "a road opens each block");
        let n = |i: usize| w[i].parse::<u128>().unwrap();
        let t = Terms { opens_ms: n(2) as u64, closes_ms: n(3) as u64, step_ms: n(4) as u64, max_step_bp: n(5) as u32, threshold_bp: n(6) as u32, fee_bp: 0, pool_base: n(7), pool_quote: n(8), budget: n(9), both_ways: false };
        t.check().unwrap();
        let mut es = Vec::new();
        while let Some(e) = lines.next_if(|l| l.starts_with("e ")) {
            let v: Vec<&str> = e.split_whitespace().collect();
            es.push(Entry { seq: v[1].parse().unwrap(), ts: v[2].parse().unwrap(), key: v[3].into(), seat: v[3].into(), ok: true,
                trade: Some(Trade { branch: if v[4] == "pass" { Which::Pass } else { Which::Fail }, side: if v[5] == "buy" { Side::Buy } else { Side::Sell }, amount: v[6].parse().unwrap(), min_out: 0 }) });
        }
        let want: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
        assert_eq!(want[0], "want");
        assert!(es.iter().all(|e| e.ts > t.opens_ms && (e.ts - t.opens_ms) % t.step_ms == 0) && (t.closes_ms - t.opens_ms) % t.step_ms == 0, "{}: on the grid, none at the opening, whole steps", w[1]);
        let m = market_of(&t, &es, t.closes_ms);
        let mine = decide(&t, &m, t.closes_ms);
        // the text's two sums, from this law's: one step later
        let open = spot(t.pool_base, t.pool_quote).unwrap();
        let (span, step) = ((t.closes_ms - t.opens_ms) as u128, t.step_ms as u128);
        let text_sum = |b: &Branch| b.cum - open * step + b.acc;
        let (sp, sf) = (text_sum(&m.pass), text_sum(&m.fail));
        let state = if sp * BP > sf * (BP + t.threshold_bp as u128) { State::Passed } else { State::Failed };
        let d = Decision { state, twap_pass: Some(sp / span), twap_fail: Some(sf / span), needs: Some(sf / span * (BP + t.threshold_bp as u128) / BP), ..mine };
        assert_eq!(match d.state { State::Passed => "passed", State::Failed => "failed", State::Open => "open" }, want[1], "{}: the verdict", w[1]);
        if mine.state != d.state { moved += 1; }
        let p = |i: usize| want[i].parse::<u128>().unwrap();
        near(d.twap_pass.unwrap(), p(2), "PASS's average", w[1]); near(d.twap_fail.unwrap(), p(3), "FAIL's average", w[1]);
        near(d.spot_pass, p(4), "PASS's spot", w[1]); near(d.spot_fail, p(5), "FAIL's spot", w[1]);
        assert_eq!((m.accepted as u128, m.refused.len() as u128), (p(6), p(7)), "{}: the entries taken and refused", w[1]);
        let scores = score_of(&t, &m, &d);
        let mut seen = 0;
        while let Some(s) = lines.next_if(|l| l.starts_with("score ")) {
            let v: Vec<&str> = s.split_whitespace().collect();
            let mine = scores.iter().find(|x| x.key == v[1]).unwrap_or_else(|| panic!("{}: the text scored {} and the law did not", w[1], v[1]));
            assert!((mine.result - v[2].parse::<i128>().unwrap()).abs() <= 10_000, "{}: {}'s result {} against the text's {}", w[1], v[1], mine.result, v[2]);
            seen += 1;
        }
        assert_eq!(seen, scores.len(), "{}: every trader is scored once", w[1]);
        roads += 1; if d.state == State::Passed { passed += 1; }
    }
    eprintln!("  24 roads of the text, one step later; this law's own verdict, a step later than the text's, differs from it on {moved}");
    assert_eq!((roads, passed), (24, 11), "24 roads of the text, 11 of them passed");
}
