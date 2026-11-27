//! THE FOUR REFERENCE WARS, replayed by the law and held to the owner's model to the unit. The reference is tests/reference/reference-in-sol.json,
//! byte for byte the file the brief of lane/the-war-is-its-own-program carried (claude-54dd2237, Nov 24 2026): the model's own run,
//! `python3 -I tests/reference/meme-royale-rulebook-sim.py <out.json> <reference.json>`, which regenerates it (the second test below runs it
//! when a python3 is there). For each war: the winner; how it was decided, in the model's words, and the minute; the raid's length; every
//! integer of the settlement; and every fighter's `holds`, `backed_lamports` and `took_lamports`. The one number the model keeps as a float, the
//! stake's need (`0.05 * m0`), is held to its floor.
use poleis_war::law::{Rules, Side, BPS};
use poleis_war::war::{fight, how_text, the_four, HOUSE};
use serde_json::Value;

fn reference() -> Vec<Value> {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/reference/reference-in-sol.json")).expect("the reference");
    serde_json::from_str::<Value>(&text).expect("the reference is JSON").as_array().expect("a list of wars").clone()
}
fn int(v: &Value) -> u128 {
    v.as_u64().unwrap_or_else(|| panic!("an integer: {v}")) as u128
}

#[test]
fn the_four_reference_wars_to_the_unit() {
    let refs = reference();
    assert_eq!(refs.len(), 4);
    for (s, r) in the_four().iter().zip(&refs) {
        let name = s.name;
        assert_eq!(r["name"].as_str().unwrap(), name, "the scenarios are the reference's, in its order");
        let (w, d) = fight(&Rules::MEME_ROYALE, s).unwrap_or_else(|e| panic!("{name}: {e}"));
        let st = &r["settled"];
        // the raid's length, its opening, its stakers
        assert_eq!(d.opening.minutes as u128, int(&r["minutes"]), "{name}: minutes");
        assert_eq!(d.opening.minutes as u128, int(&st["minutes"]), "{name}: settled minutes");
        assert_eq!(d.opening.m0[0], int(&st["m0"]["crow"]), "{name}: m0 crow");
        assert_eq!(d.opening.m0[1], int(&st["m0"]["wolf"]), "{name}: m0 wolf");
        let stakers: Vec<String> = st["stakers"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        assert_eq!(d.stakers, stakers, "{name}: the stakers, the attacker's last ones in, downward");
        assert_eq!(d.stakers.len() as u128, int(&r["stakers"]), "{name}: how many staked");
        let need_float = st["stake_need"].as_f64().unwrap();
        assert_eq!(d.opening.need, need_float.floor() as u128, "{name}: the stake's need is the floor of the model's float {need_float}");
        assert_eq!(w.refused.len() as u128, int(&r["refused"]), "{name}: refused");
        // who won and how
        assert_eq!(HOUSE[d.winner.index()], r["winner"].as_str().unwrap(), "{name}: winner");
        assert_eq!(how_text(d.how, d.at), r["how"].as_str().unwrap(), "{name}: how");
        // the settlement
        assert_eq!(d.settlement.pot_in, int(&st["pot_in"]), "{name}: pot_in");
        assert_eq!(d.settlement.bought_with, int(&st["bought_with"]), "{name}: bought_with");
        assert_eq!(d.settlement.buy.coins, int(&st["coins_bought"]), "{name}: coins_bought");
        assert_eq!(d.winner_reserve, int(&st["winner_reserve"]), "{name}: winner_reserve");
        assert_eq!(d.winner_chest, int(&st["winner_chest"]), "{name}: winner_chest");
        assert_eq!(d.all_held, int(&st["all_held"]), "{name}: all_held");
        assert_eq!(d.mcap_end[0], int(&st["mcap_end"]["crow"]), "{name}: mcap_end crow");
        assert_eq!(d.mcap_end[1], int(&st["mcap_end"]["wolf"]), "{name}: mcap_end wolf");
        // the lamport the model drops: no reference war's settlement buy leaves one
        assert_eq!(d.settlement.buy.paid(), d.settlement.bought_with, "{name}: the settlement's buy pays all it was given");
        // every fighter, to the unit
        let pf = r["per_fighter"].as_object().unwrap();
        assert_eq!(pf.len(), d.fighters.len(), "{name}: the same fighters");
        for (who, f) in &d.fighters {
            let x = pf.get(who).unwrap_or_else(|| panic!("{name}: {who} is not in the reference"));
            assert_eq!(f.holds, int(&x["holds"]), "{name}: {who} holds");
            assert_eq!(f.backed, int(&x["backed_lamports"]), "{name}: {who} backed_lamports");
            assert_eq!(f.took, int(&x["took_lamports"]), "{name}: {who} took_lamports");
        }
        // the dump tax at once: the model's float, the law's basis points (its floor, at least 10%)
        let at_once = r["dump_tax_at_once"].as_f64().unwrap();
        let rate = d.dump.map(|t| t.rate_bps).unwrap_or(0);
        assert!(((rate as f64) / BPS as f64 - at_once).abs() < 1e-4, "{name}: dump tax {rate} bp against the model's {at_once}");
        assert_eq!(rate as f64, (at_once * BPS as f64).floor(), "{name}: the dump tax's rate is the floor of the model's");
        eprintln!(
            "  {name}: {} wins, {} | T {} min | pot {} bought {} coins | the dump tax {} bp for {} s",
            HOUSE[d.winner.index()],
            how_text(d.how, d.at),
            d.opening.minutes,
            d.settlement.pot_in,
            d.settlement.buy.coins,
            rate,
            d.dump.map(|t| t.seconds).unwrap_or(0)
        );
        let _ = Side::Attacker;
    }
}

/// the reference is the model's: when a python3 is on this machine, the copied model regenerates the reference byte for byte
#[test]
fn the_copied_model_regenerates_the_reference() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/reference");
    let out = std::env::temp_dir().join(format!("poleis-war-model-{}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let (all, mine) = (out.join("all.json"), out.join("reference.json"));
    let run = std::process::Command::new("python3")
        .args(["-I", "-B", &format!("{dir}/meme-royale-rulebook-sim.py"), all.to_str().unwrap(), mine.to_str().unwrap()])
        .output();
    let Ok(run) = run else {
        eprintln!("  no python3 here: the reference is read as it stands");
        return;
    };
    assert!(run.status.success(), "the model ran: {}", String::from_utf8_lossy(&run.stderr));
    let a = std::fs::read(&mine).unwrap();
    let b = std::fs::read(format!("{dir}/reference-in-sol.json")).unwrap();
    let _ = std::fs::remove_dir_all(&out);
    assert!(a == b, "the model's run differs from the reference kept here");
}
