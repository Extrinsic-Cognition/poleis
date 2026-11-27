//! THE EPOCH TURNS THE CRANKS, the pure half (Nov 25 2026, lane/the-epoch-turns-the-cranks): what a pass finds due from the chain's epoch, the
//! pool's last update, the epoch's rewards and the book's last read and row; the sleep's arithmetic; the bound on resends; the keys the runner
//! reads and refuses. The live half is tests/light.rs the_epoch_turns_the_cranks_on_a_validator under poleis/ceremony/the-epoch-rehearsal.sh.

use poleis_press::epoch::{self, Due, Next};
use poleis_press::params::Params;
use poleis_press::rpc::{Cluster, Rpc};
use poleis_press::wire::Key;
use std::time::Duration;

#[test]
fn what_is_due_follows_the_chain_and_the_book() {
    // a new epoch, her pool behind, the book read last epoch and counted then: bring, count, pour
    assert_eq!(epoch::due(7, 6, false, Some(6), Some(6)), Due { wait: false, bring: true, count: true, pour: true });
    // the epoch's rewards being paid while her pool is behind: wait
    assert!(epoch::due(7, 6, true, Some(6), Some(6)).wait);
    // the rewards paid but her pool already at the epoch: nothing waits
    assert!(!epoch::due(7, 7, true, Some(6), None).wait);
    // everything done this epoch: the pool at it, the book read and counted at it: only the pour's record, which the chain holds, is asked
    assert_eq!(epoch::due(7, 7, false, Some(7), Some(7)), Due { wait: false, bring: false, count: false, pour: true });
    // the book's first read (no book): a read, and no pour comes of it (a first read counts nothing)
    let first = epoch::due(3, 3, false, None, None);
    assert!(first.count && !first.bring);
    // read this epoch, counted an earlier one (a first read after a gap in which nothing was counted): no pour is asked
    assert_eq!(epoch::due(9, 9, false, Some(9), Some(8)), Due { wait: false, bring: false, count: false, pour: false });
}

#[test]
fn the_sleep_ends_at_the_next_epochs_first_slot_and_never_spins() {
    // slot 10 of 32 at 400 ms, a margin of 2: 24 slots
    assert_eq!(epoch::sleep_for(10, 32, 400, 2), Duration::from_millis(24 * 400));
    // mainnet's epoch, slot 0 of 432,000 at 400 ms: two days
    assert_eq!(epoch::sleep_for(0, 432_000, 400, 0), Duration::from_millis(432_000 * 400));
    // woken past the epoch's end with no margin: one slot, never nothing
    assert_eq!(epoch::sleep_for(40, 32, 400, 0), Duration::from_millis(400));
    assert_eq!(epoch::sleep_for(32, 32, 0, 0), Duration::from_millis(1));
}

#[test]
fn a_send_is_sent_again_built_again_and_bounded() {
    let s = Duration::from_secs;
    assert_eq!(epoch::next(true, false, 10, 100, s(0), 1, 5), Next::Done);
    assert_eq!(epoch::next(false, true, 10, 100, s(9), 1, 5), Next::Failed, "a landed failure is the chain's word, never sent again");
    assert_eq!(epoch::next(false, false, 10, 100, Duration::from_millis(500), 1, 5), Next::Wait);
    assert_eq!(epoch::next(false, false, 10, 100, s(2), 1, 5), Next::Resend, "the same bytes again every two seconds");
    assert_eq!(epoch::next(false, false, 101, 100, s(0), 1, 5), Next::Rebuild, "past its blockhash's last valid height: built again");
    assert_eq!(epoch::next(false, false, 101, 100, s(0), 5, 5), Next::GiveUp, "five builds and never confirmed: the step fails and says so");
}

#[test]
fn the_runner_refuses_without_its_rules_or_its_keys() {
    let rpc = Rpc::new("http://127.0.0.1:1", Cluster::Localnet).unwrap();
    let dir = std::env::temp_dir().join(format!("epoch-keys-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // the rehearsal's params: the engine's staker and the light's cap are the rehearsal's words, no keys stand
    let p = Params::read(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json"))).unwrap();
    let e = epoch::held(&dir, &p, &rpc).err().expect("refused");
    for what in ["engine.staker", "light.planCap", "no engine key", "no pourer key"] {
        assert!(e.contains(what), "the refusal names {what}: {e}");
    }
    // the keys made once, beside the sitting's keys; a second making makes none
    let k = Key::from_seed(&[7; 32]);
    let (a, made) = poleis_press::seat::engine_key(&dir, || Ok(Key::from_seed(&[7; 32]))).unwrap();
    assert!(made && a == k.address);
    assert!(!poleis_press::seat::engine_key(&dir, || Ok(Key::from_seed(&[8; 32]))).unwrap().1, "never over a key that stands");
    poleis_press::seat::pourer_key(&dir, || Ok(Key::from_seed(&[9; 32]))).unwrap();
    // the params naming another key as the pool's staker: refused, never another key
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json")).unwrap()).unwrap();
    v["engine"]["staker"] = serde_json::json!(Key::from_seed(&[8; 32]).address.to_string());
    v["light"]["planCap"] = serde_json::json!(1_000_000u64);
    let p = Params::from_text(&v.to_string()).unwrap();
    let e = epoch::held(&dir, &p, &rpc).err().expect("refused");
    assert!(e.contains("is not the params' engine.staker"), "{e}");
    let _ = std::fs::remove_dir_all(&dir);
}

fn snap(epoch: u64, holders: &[(&str, u64)]) -> embertide_stake::Snapshot {
    embertide_stake::Snapshot { epoch, total_lamports: 100, pool_token_supply: 100, holders: holders.iter().map(|(o, t)| (o.to_string(), *t)).collect() }
}

/// A MISSED EPOCH'S LIGHT IS POURED LATER, NEVER DROPPED: an epoch counted whose pour failed stays owed, and the next pass pours it before the newer
/// one, oldest first; each is removed only once the chain's record of its pour is read (settled).
#[test]
fn a_counted_epoch_stays_owed_until_its_pour_is_recorded() {
    use embertide_stake::{count, Book};
    use poleis_press::door::Light;
    use poleis_press::epoch::{owed_list, remove_owed, settled, write_owed, Owed};
    use poleis_press::light::{self, Held};
    let pool = solana_pubkey::Pubkey::new_from_array([3; 32]);
    let dir = std::env::temp_dir().join(format!("epoch-owed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let h = [("a", 60u64), ("b", 40)];
    let b6 = count(&Book::new(&pool.to_string()), &snap(6, &h), 0).unwrap();
    let b7 = count(&b6, &snap(7, &h), 10).unwrap();
    let b8 = count(&b7, &snap(8, &h), 10).unwrap();
    // epoch 7 counted, its pour failed: still owed; epoch 8 counted: both owed, the older first
    write_owed(&dir, &Owed { epoch: 7, before: b6.clone(), after: b7.clone() }).unwrap();
    write_owed(&dir, &Owed { epoch: 8, before: b7.clone(), after: b8.clone() }).unwrap();
    let owed = owed_list(&dir, &pool).unwrap();
    assert_eq!(owed.iter().map(|o| o.epoch).collect::<Vec<_>>(), [7, 8], "every owed epoch, oldest first, before the newest");
    // each reads back as its own count: the reader's lights of epoch 7, then of epoch 8
    for o in &owed {
        let l = light::lights_of(Some(&o.before), &o.after).unwrap();
        assert_eq!((l.epoch, l.fee_lamports), (o.epoch, 10));
    }
    // settled only once the chain holds its pour for every wish it owes
    let l = light::lights_of(Some(&owed[0].before), &owed[0].after).unwrap();
    let r = light::read(&l, &[("a".to_string(), Some(Light::Wish(5))), ("b".to_string(), None)].into_iter().collect(), &[5].into_iter().collect());
    assert!(!settled(&r, &[], 1), "a pour still to send: owed");
    assert!(!settled(&r, &[(5, Held::NoRoom { room: 0 })], 0), "its road has no room yet: owed");
    assert!(settled(&r, &[(5, Held::Poured)], 0), "the chain's record read back: settled");
    assert!(settled(&r, &[(5, Held::NoWell)], 0), "a wish with no road: settled, its light said undirected");
    remove_owed(&dir, 7).unwrap();
    assert_eq!(owed_list(&dir, &pool).unwrap().iter().map(|o| o.epoch).collect::<Vec<_>>(), [8]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// THE RUNNER STOPPED FOR TWO EPOCHS: the book's own law counts the span from its last read to now as one epoch, by what stood at both reads; its
/// whole fee is attributed (a staker who came in the gap takes nothing; the others take its part); the pass names the span; where nobody held across,
/// the whole fee stays in root 0's vault, named with its epochs and its amount.
#[test]
fn a_gap_is_the_books_one_count_and_is_said_by_name() {
    use embertide_stake::{count, Book};
    use poleis_press::epoch::gap_words;
    let b7 = count(&Book::new("p"), &snap(7, &[("a", 60), ("b", 40)]), 0).unwrap();
    // read at 7, the runner down through 8 and 9, read again at 10: one count of epochs 8 to 10, the fee of all three
    let b10 = count(&b7, &snap(10, &[("a", 60), ("b", 40), ("c", 50)]), 300).unwrap();
    let row = b10.row.clone().unwrap();
    assert_eq!((row.from, row.epoch, row.fee_lamports, row.holders), (7, 10, 300, 2), "one count, a and b only: c came in the gap");
    assert_eq!((b10.stakers["a"].light, b10.stakers["b"].light, row.dust), (180, 120, 0), "the whole fee attributed by what stood at both reads");
    let w = gap_words(&row).expect("a gap is said");
    assert!(w.contains("epochs 8 to 10 counted as one") && w.contains("300 lamports"), "{w}");
    // nobody held across the gap: the whole fee stays in root 0's vault, said with its epochs and its amount
    let gone = count(&b7, &snap(10, &[("c", 50)]), 300).unwrap();
    let w = gap_words(gone.row.as_ref().unwrap()).unwrap();
    assert!(w.contains("300 lamports of epochs 8 to 10's fee no holder's share took") && w.contains("stays in root 0's vault"), "{w}");
    // one epoch, rounding only: nothing to say
    let one = count(&b7, &snap(8, &[("a", 60), ("b", 40)]), 10).unwrap();
    assert!(gap_words(one.row.as_ref().unwrap()).is_none());
    assert_eq!(poleis_press::epoch::OWED_BOUND, 64);
}
