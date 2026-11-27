//! THE WELL PAYS ITS RIDER, the pure half (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider): the wish's act as Poleis reads it (the launch's
//! act 8's shape on the wish's own unit), root 0's road into a wish's well, the plan of the mint, the press's sets, and the door's draw held to
//! Poleis's own client byte for byte. The live half is tests/light.rs under poleis/ceremony/the-light-rehearsal.sh.

use poleis_client::{self as client, Payload, CLASS_CODEX, CLASS_TREASURY, CODEX_ALLOW_MINT, HOLDER_KEY, HOLDER_UNIT, ROOT_UNIT, TREASURY_ALLOWANCE};
use poleis_door::{self as door, Keys};
use poleis_press::rider;
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;

fn k(b: u8) -> Pubkey {
    Pubkey::new_from_array([b; 32])
}

fn her() -> Keys {
    Keys { program: door::STAKE_POOL, pool: k(3), withdraw: k(4), list: k(5), reserve: k(6), mint: k(7), vault: client::vault_pda(ROOT_UNIT, &k(7)) }
}

#[test]
fn the_riders_act_is_the_routers_shape_on_the_wishs_unit() {
    let (rider, mint) = (k(9), k(7));
    let p = rider::rider_act(42, &rider, &mint, 5_000, 1_000);
    assert_eq!(p.origin(), 42, "the wish's own unit, never root 0");
    assert_eq!(p.class(), CLASS_TREASURY);
    let b = p.bytes();
    assert_eq!(&b[..2], &[CLASS_TREASURY, TREASURY_ALLOWANCE]);
    // Poleis's open_allowance reads [payee:32][payee_kind:1][mint:32][cap:8][period:8][expires:8]
    let f = &b[2..];
    assert_eq!(f.len(), 89);
    assert_eq!(&f[..32], rider.as_ref());
    assert_eq!(f[32], HOLDER_KEY, "a key: the rider pulls by its own signature");
    assert_eq!(&f[33..65], mint.as_ref());
    assert_eq!(u64::from_le_bytes(f[65..73].try_into().unwrap()), 5_000);
    assert_eq!(u64::from_le_bytes(f[73..81].try_into().unwrap()), 1_000);
    assert_eq!(u64::from_le_bytes(f[81..89].try_into().unwrap()), 0, "no expiry: it stands until the wish revokes it");
    // the launch's act 8 is the same payload on root 0
    let router = Payload::Allowance { origin: ROOT_UNIT, payee: rider.to_bytes(), payee_kind: HOLDER_KEY, mint, cap: 5_000, period: 1_000, expires: 0 };
    assert_eq!(router.bytes(), b, "one shape, two units: the payload bytes never name the unit");
}

#[test]
fn the_wells_road_is_root_0s_allowance_to_a_unit() {
    let mint = k(7);
    let p = rider::well_act(42, &mint, 9_000, 32);
    assert_eq!(p.origin(), ROOT_UNIT);
    let f = &p.bytes()[2..];
    assert_eq!(&f[..32], &client::unit_holder(42), "the payee is the wish's unit");
    assert_eq!(f[32], HOLDER_UNIT, "a unit: Poleis lets anyone pull for it, into its own vault of the mint and nowhere else");
    let plan = rider::plan_act(42, &mint, 77, &k(1));
    assert_eq!(plan.class(), CLASS_CODEX);
    assert_eq!(&plan.bytes()[..2], &[CLASS_CODEX, CODEX_ALLOW_MINT]);
}

#[test]
fn the_door_draws_as_poleis_pulls() {
    let (p, rider, hash) = (her(), k(9), [0x5a; 32]);
    let ixs = door::draw(&p, &client::PROGRAM_ID, 42, &hash, &rider, 1_234);
    let want = client::pull(rider, hash, 42, 1_234, door::receipts_of(&rider, &p.mint), Some((p.mint, door::TOKEN)));
    assert_eq!(ixs[1], want, "the door's pull is Poleis's own client's, account for account and byte for byte");
    assert_eq!(ixs[0], door::receipts_made(&rider, &rider, &p.mint));
    let flow = door::draw_to_sol(&p, &client::PROGRAM_ID, 42, &hash, &rider, 1_234, Some(&[]));
    assert_eq!(flow.len(), 5, "the pool brought to the epoch, the receipts account, the pull, the way out");
    assert_eq!(flow[3], want);
    assert_eq!(flow[4], door::unstake(&p, &rider, 1_234, None)[0]);
}

#[test]
fn the_press_writes_the_sets_and_refuses_what_poleis_would() {
    let mut o = BTreeMap::new();
    for (a, b) in [("--unit", "42"), ("--key", &k(9).to_string()), ("--mint", &k(7).to_string()), ("--cap", "5000"), ("--period", "1000"), ("--plan-cap", "100000"), ("--crank", &k(1).to_string()), ("--proposer", &k(2).to_string())] {
        o.insert(a.to_string(), b.to_string());
    }
    let v = rider::step("rider", &o).unwrap();
    let sets = v["sets"].as_array().unwrap();
    assert_eq!(sets.len(), 2, "the wish's plan of the mint, then the rider's allowance");
    let d = rider::decision(sets[1]["what"].as_str().unwrap(), rider::rider_act(42, &k(9), &k(7), 5_000, 1_000), 1);
    assert_eq!(sets[1]["hash"], poleis_press::wire::hex(&d.hash));
    assert_eq!(sets[1]["allowance"], client::allowance_pda(&d.hash).to_string());
    assert_eq!(sets[1]["propose"]["accounts"][0]["address"], k(2).to_string(), "a member of the wish proposes it");
    assert_eq!(sets[1]["root"], "42", "a wish is its own root");
    for (key, bad) in [("--unit", "0"), ("--cap", "0"), ("--period", "0"), ("--mint", "nobody")] {
        let mut b = o.clone();
        b.insert(key.into(), bad.into());
        assert!(rider::step("rider", &b).is_err(), "{key} {bad}");
    }
    let w = rider::step("well", &o).unwrap();
    assert_eq!(w["sets"][1]["unit"], "0", "the well's road is root 0's decision");
}
