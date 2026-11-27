//! THE FEE'S BUDGET IS ASKED ONCE: with a priority fee every transaction the press sends asks one compute budget, never two. A step that
//! names its own cap (the vault, the crank's votes and seals, the ratify, the pool's birth) keeps it, and the press adds none of its own
//! there; a transaction with a second is refused ("Transaction contains a duplicate instruction"), as the rehearsal of mainnet's priority
//! fee found at `press vault` (Nov 23 2026). Mainnet's own params, and the rehearsal's with no fee.
use poleis_press::chain::{asks_budget, budget_of, priority_lamports, sized_limit};
use poleis_press::loader::{self, COMPUTE_BUDGET};
use poleis_press::params::Params;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use std::path::Path;

fn params(file: &str) -> Params {
    Params::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).unwrap()
}

fn an_instruction() -> Instruction {
    Instruction::new_with_bytes(Pubkey::new_from_array([7; 32]), &[1, 2, 3], vec![])
}

#[test]
fn a_step_with_its_own_cap_asks_one_budget_and_the_press_adds_none() {
    let m = params("params.mainnet.json");
    assert_eq!(m.u64_or("priorityMicroLamports", 0).unwrap(), 100_000, "mainnet's priority fee, his picks of Nov 23 2026");
    let capped = [budget_of(&m, 30_000).unwrap(), vec![an_instruction()]].concat();
    assert!(asks_budget(&capped), "a step's own cap is a compute budget the press sees");
    assert_eq!(capped.iter().filter(|ix| ix.program_id == COMPUTE_BUDGET).count(), 2, "one budget: its unit limit and its price, once each");
    assert!(!asks_budget(&[an_instruction()]), "a transaction with no cap of its own gets the press's, sized by its simulation");
    assert!(asks_budget(&[loader::compute_limit(1_400_000), an_instruction()]), "a probe's limit counts too");
}

#[test]
fn a_buffer_write_costs_its_signature_and_its_budgets_priority() {
    // a write asks 10,000 units, sized to 13,000; at mainnet's 100,000 micro-lamports a unit the cluster takes 1,300 lamports more than
    // its 5,000 signature, what the record now writes for it (it wrote 5,000 for every write until the rehearsal of Nov 23 2026)
    assert_eq!(sized_limit(10_000), 13_000);
    assert_eq!(priority_lamports(100_000, sized_limit(10_000)), 1_300);
    assert_eq!(priority_lamports(0, sized_limit(10_000)), 0, "no fee, nothing more");
    assert_eq!(priority_lamports(100_000, 30_000), 3_000, "a PRO's 30,000 units: the council checks' 8,000 lamports a PRO");
    assert_eq!(priority_lamports(1, 1), 1, "a part of a lamport is a lamport");
    assert_eq!(sized_limit(1_300_000), 1_400_000, "never past the cluster's limit");
}

#[test]
fn with_no_fee_nothing_is_asked() {
    let r = params("params.rehearsal.json");
    assert_eq!(r.u64_or("priorityMicroLamports", 0).unwrap(), 0);
    assert!(budget_of(&r, 30_000).unwrap().is_empty(), "no fee, no budget");
    assert!(!asks_budget(&[budget_of(&r, 30_000).unwrap(), vec![an_instruction()]].concat()));
}
