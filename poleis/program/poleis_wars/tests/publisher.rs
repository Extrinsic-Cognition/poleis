//! THE GAME IS ONE SWARM (Nov 24 2026): the publisher's share goes to the game swarm's Poleis treasury. Mollusk loads Poleis's own
//! poleis_treasury.so beside the war (as poleis_treasury's own tests load it), founds the game swarm's tree and its treasury
//! (["treasury", unit] of Poleis), founds a generation of the war naming that treasury, and plays trades into two houses launched by two
//! players: each house's creator is paid exactly the creator fees of every trade into it, and `pay_publisher` pays the treasury exactly the
//! protocol's, which Poleis's own `sync_endowment` then books in the treasury's total_in, to the lamport.
mod built;
mod common;
use common::*;
use embertide_war::law::{buy_on, Curve, Rules, SOL as LAW_SOL};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

const T0: i64 = 1_800_000_000;
/// Poleis's id, as its source declares it (held to it below)
const POLEIS: &str = "AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab";
/// Poleis's own layout of a treasury (poleis_treasury/src/state.rs Treasury): 27 bytes, total_in at 10, total_out at 18
const TREASURY_SIZE: usize = 27;
const TREASURY_OFF_TOTAL_IN: usize = 10;
const TREASURY_OFF_TOTAL_OUT: usize = 18;

fn poleis_id() -> Pubkey {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../poleis_treasury/src/lib.rs")).expect("poleis_treasury's source");
    assert!(src.contains(&format!("declare_id!(\"{POLEIS}\")")), "Poleis declares {POLEIS}");
    POLEIS.parse().unwrap()
}
/// Poleis's own instructions, by its tags and data (poleis_treasury/src/lib.rs, src/ix/treasury.rs)
fn poleis_ix(tag: u8, data: &[u8], metas: Vec<AccountMeta>) -> Instruction {
    let mut d = vec![tag];
    d.extend_from_slice(data);
    Instruction::new_with_bytes(poleis_id(), &d, metas)
}

#[test]
fn the_publishers_share_reaches_the_game_swarms_poleis_treasury_and_is_booked() {
    let mut w = World::new(T0);
    std::env::set_var("SBF_OUT_DIR", built::poleis_dir());
    let poleis = poleis_id();
    w.m.add_program(&poleis, "poleis_treasury");
    // the game swarm: a tree of Poleis (root 7, its solo authority standing in for its seal) and its unit's treasury
    let swarm = w.wallet(10 * SOL);
    let (root, unit) = (7u64, 7u64);
    let tree = Pubkey::find_program_address(&[b"config", &root.to_le_bytes()], &poleis).0;
    let treasury = Pubkey::find_program_address(&[b"treasury", &unit.to_le_bytes()], &poleis).0;
    let mut d = root.to_le_bytes().to_vec();
    d.extend_from_slice(&[0u8; 32]);
    w.ok(&poleis_ix(0, &d, vec![AccountMeta::new(swarm, true), AccountMeta::new(tree, false), AccountMeta::new_readonly(system_id(), false)]), "Poleis: the game swarm's tree");
    w.ok(
        &poleis_ix(1, &unit.to_le_bytes(), vec![AccountMeta::new(swarm, true), AccountMeta::new_readonly(tree, false), AccountMeta::new(treasury, false), AccountMeta::new_readonly(system_id(), false)]),
        "Poleis: its treasury",
    );
    let t = w.account(&treasury);
    assert_eq!((t.owner, t.data.len()), (poleis, TREASURY_SIZE), "a treasury of Poleis");
    let total_in = |w: &World| u64_at(&w.data(&treasury), TREASURY_OFF_TOTAL_IN);
    assert_eq!((total_in(&w), u64_at(&t.data, TREASURY_OFF_TOTAL_OUT)), (0, 0));
    // the war's generation 0, founded by the swarm's seal (a key standing in for it, signing only), naming that treasury
    let seal = w.wallet(SOL);
    w.ok(&init_config_of(swarm, seal, 0, treasury, &rules_bytes(&Rules::MEME_ROYALE, 0, 0, 60)), "the war's generation, naming the swarm's treasury");
    let config = config_pda_of(&seal, 0);
    // two houses launched by two players; trades into each
    let (p1, p2) = (w.wallet(SOL), w.wallet(SOL));
    w.ok(&init_house(p1, config, 0), "p1 launches the crow");
    w.ok(&init_house(p2, config, 1), "p2 launches the wolf");
    let (crow, wolf) = (house_pda(&config, 0), house_pda(&config, 1));
    let r = Rules::MEME_ROYALE;
    let (mut c0, mut c1) = (Curve::born(&r), Curve::born(&r));
    let (mut creator0, mut creator1, mut protocol) = (0u128, 0u128, 0u128);
    for (i, spend) in [3u64, 1, 2, 5, 1].iter().enumerate() {
        let k = w.wallet(10 * SOL);
        let (house, curve, creator) = if i % 2 == 0 { (crow, &mut c0, &mut creator0) } else { (wolf, &mut c1, &mut creator1) };
        let (after, b) = buy_on(&r, curve, *spend as u128 * LAW_SOL).unwrap();
        *curve = after;
        *creator += b.creator;
        protocol += b.protocol;
        w.ok(&buy(k, config, house, spend * SOL, 0, None), "a buy");
    }
    // each creator paid exactly the creator fees of the trades into his house
    for (house, player, owed) in [(crow, p1, creator0), (wolf, p2, creator1)] {
        assert_eq!(w.house(&house).creator_owed as u128, owed);
        let before = w.lamports(&player);
        w.ok(&pay_creator(house, player), "the creator paid");
        assert_eq!((w.lamports(&player) - before) as u128, owed, "exactly his house's creator fees");
    }
    // the publisher: everything the fee account holds beyond its rent into the swarm's Poleis treasury, then Poleis books it
    let before = w.lamports(&treasury);
    w.ok(&pay_publisher(config, treasury), "the publisher paid into the swarm's treasury");
    let paid = w.lamports(&treasury) - before;
    assert_eq!(paid as u128, protocol, "exactly the protocol's fees");
    assert_eq!(total_in(&w), 0, "wired straight in, the treasury's books have not seen it yet");
    w.ok(&poleis_ix(3, &unit.to_le_bytes(), vec![AccountMeta::new(treasury, false)]), "Poleis: sync_endowment, anyone trues the books");
    assert_eq!(total_in(&w), paid, "Poleis's own books now hold the publisher's share, to the lamport");
    eprintln!("  the swarm's treasury booked {paid} lamports from the war's publisher share; the crow's creator got {creator0}, the wolf's {creator1}");
    // a generation's fees go to its own treasury and no other, and once paid there is nothing left to pay
    w.refuse(&pay_publisher(config, Pubkey::new_unique()), code(poleis_wars::error::WarsError::InvalidAccount), "another treasury");
    w.refuse(&pay_publisher(config, treasury), code(poleis_wars::error::WarsError::NothingOwed), "nothing left to pay");
}
