//! The cost, printed and bounded: the compute units of every instruction on Mollusk (the mine's `init`, the authority's two words, a
//! deposit that makes a flask's record and one that finds it, a prior deposit, a spend that makes the flask's $WARM account, one that
//! finds it, one paid from the vault, a pay-back, a take-back); the largest `mine` batch a legacy transaction of 1,232 bytes carries, with every
//! flask's account to make and with every one standing, its bytes and its units against the 200,000 a transaction gets by default; each
//! account's size and rent; and the program's size and its rent on mainnet at 5,080 lamports a byte. Every line starts with `cost:`.
mod built;
mod common;
use common::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use warm_mine::state::{Flask, Mine, Prior};

/// A transaction a validator takes: an IPv6 packet's 1,280 bytes less its headers.
const PACKET: usize = 1_232;
/// What a transaction's instructions get without a compute budget instruction of their own: 200,000 units each.
const DEFAULT_UNITS: u64 = 200_000;
/// Mainnet's rent, as the brief names it: lamports a byte, the account's 128 bytes of overhead counted.
const MAINNET_LAMPORTS_A_BYTE: u64 = 5_080;
const OPENING: u64 = 1_794_700_800_000;

fn compact_len(n: usize) -> usize {
    if n < 0x80 {
        1
    } else if n < 0x4000 {
        2
    } else {
        3
    }
}
/// The bytes of a legacy transaction carrying `ix` alone, `payer` paying: its signatures, its header, every key once, the blockhash, and
/// the instruction by the index of its program and of its accounts.
fn tx_bytes(ix: &Instruction, payer: &Pubkey) -> usize {
    let mut keys: Vec<Pubkey> = vec![*payer];
    let mut signers: Vec<Pubkey> = vec![*payer];
    for m in &ix.accounts {
        if !keys.contains(&m.pubkey) {
            keys.push(m.pubkey);
        }
        if m.is_signer && !signers.contains(&m.pubkey) {
            signers.push(m.pubkey);
        }
    }
    if !keys.contains(&ix.program_id) {
        keys.push(ix.program_id);
    }
    compact_len(signers.len()) + 64 * signers.len() + 3 + compact_len(keys.len()) + 32 * keys.len() + 32 + 1 + 1 + compact_len(ix.accounts.len()) + ix.accounts.len() + compact_len(ix.data.len()) + ix.data.len()
}

#[test]
fn the_cost_of_every_instruction() {
    let polis = Pubkey::new_unique();
    let mut w = World::new(WORD_S);
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    let open = init(deployer, deployer, &polis, OPENING, &treasury, &dollar);
    let c_init = w.ok(&open, "init");
    let miner = w.wallet(100 * SOL);
    let c_miner = w.ok(&set_miner(polis, &miner), "set_miner");
    let second = w.lay_token(&dollar, &Pubkey::new_unique(), 0);
    let c_treasury = w.ok(&set_treasury(polis, &second), "set_treasury");
    w.ok(&set_treasury(polis, &treasury), "set_treasury back");
    let (f, source) = w.flask(100 * USD);
    let c_dep_new = w.ok(&deposit(f, f, &source, &treasury, &dollar, 50 * USD), "a deposit that makes the record");
    let c_dep = w.ok(&deposit(f, f, &source, &treasury, &dollar, 50 * USD), "a deposit to a record that stands");
    let g = Pubkey::new_unique();
    let c_prior_new = w.ok(&record_prior(polis, miner, &g, &[1u8; 64], 20 * USD), "a prior deposit that makes the record");
    let c_prior = w.ok(&record_prior(polis, miner, &g, &[2u8; 64], 20 * USD), "a prior deposit to a record that stands");
    let now = w.now_ms();
    let c_spend_new = w.ok(&mine(miner, &[Spend { flask: f, usd: 10 * USD, at: now, seq: 1 }]), "a spend that makes the flask's $WARM account");
    let c_spend = w.ok(&mine(miner, &[Spend { flask: f, usd: 10 * USD, at: now, seq: 2 }]), "a spend to an account that stands");
    let f_warm = ata(&f, &mint_pda());
    let c_pay = w.ok(&pay_back(f, f, &f_warm, 15 * WARM), "a pay-back");
    let c_vault = w.ok(&mine(miner, &[Spend { flask: f, usd: 10 * USD, at: now, seq: 3 }]), "a spend paid from the vault whole");
    let c_split = w.ok(&mine(miner, &[Spend { flask: f, usd: 10 * USD, at: now, seq: 4 }]), "a spend paid from the vault and minted");
    w.ok(&approve(&f, &mine_pda(), u64::MAX), "the flask's one approval, the mine its delegate");
    let c_take = w.ok(&take_back(miner, &[(f, 15 * WARM + WARM)]), "a take-back of what its tank drank");
    for (what, cu, ix) in [
        ("init (the mint, the vault and the mine made)", c_init, &open),
        ("set_miner", c_miner, &set_miner(polis, &miner)),
        ("set_treasury", c_treasury, &set_treasury(polis, &second)),
        ("deposit, the record made", c_dep_new, &deposit(f, f, &source, &treasury, &dollar, 50 * USD)),
        ("deposit, the record standing", c_dep, &deposit(f, f, &source, &treasury, &dollar, 50 * USD)),
        ("record_prior, the record made", c_prior_new, &record_prior(polis, miner, &g, &[1u8; 64], 20 * USD)),
        ("record_prior, the record standing", c_prior, &record_prior(polis, miner, &g, &[2u8; 64], 20 * USD)),
        ("mine one spend, its $WARM account made", c_spend_new, &mine(miner, &[Spend { flask: f, usd: 1, at: now, seq: 1 }])),
        ("mine one spend, minted", c_spend, &mine(miner, &[Spend { flask: f, usd: 1, at: now, seq: 1 }])),
        ("mine one spend, paid from the vault", c_vault, &mine(miner, &[Spend { flask: f, usd: 1, at: now, seq: 1 }])),
        ("mine one spend, from the vault and minted", c_split, &mine(miner, &[Spend { flask: f, usd: 1, at: now, seq: 1 }])),
        ("pay_back", c_pay, &pay_back(f, f, &f_warm, 1)),
        ("take_back, one flask", c_take, &take_back(miner, &[(f, 1)])),
    ] {
        let bytes = tx_bytes(ix, &ix.accounts.iter().find(|m| m.is_signer && m.is_writable).map(|m| m.pubkey).unwrap_or(deployer));
        eprintln!("cost: {what}: {cu} CU, {bytes} bytes a transaction");
        assert!(cu < DEFAULT_UNITS, "{what}: {cu} units, past a transaction's default");
        assert!(bytes <= PACKET);
    }
}

/// The largest batch: as many flasks as one legacy transaction of 1,232 bytes names (each spend names three accounts, two of them its
/// own), every one's $WARM account to make, then every one standing; and one flask's spends as many as fit.
#[test]
fn the_largest_batch_fits_one_transaction() {
    let mut most = 0;
    for k in 1..=40 {
        let spends: Vec<Spend> = (0..k).map(|i| Spend { flask: Pubkey::new_unique(), usd: 1, at: 1, seq: i as u64 + 1 }).collect();
        if tx_bytes(&mine(Pubkey::new_unique(), &spends), &Pubkey::new_unique()) <= PACKET {
            most = k;
        }
    }
    let (mut w, miner, flasks) = {
        let polis = Pubkey::new_unique();
        let mut w = World::opened(WORD_S, &polis, OPENING);
        let miner = w.wallet(100 * SOL);
        w.ok(&set_miner(polis, &miner), "the miner");
        let (treasury, dollar) = (w.treasury, w.dollar);
        let mut flasks = Vec::new();
        for _ in 0..most {
            let (f, source) = w.flask(10 * USD);
            w.ok(&deposit(f, f, &source, &treasury, &dollar, 10 * USD), "a deposit");
            flasks.push(f);
        }
        (w, miner, flasks)
    };
    let now = w.now_ms();
    let first: Vec<Spend> = flasks.iter().enumerate().map(|(i, f)| Spend { flask: *f, usd: USD, at: now, seq: i as u64 + 1 }).collect();
    let ix = mine(miner, &first);
    let bytes = tx_bytes(&ix, &miner);
    let cu_new = w.ok(&ix, "the largest batch, every account made");
    let again: Vec<Spend> = flasks.iter().enumerate().map(|(i, f)| Spend { flask: *f, usd: USD, at: now, seq: 100 + i as u64 }).collect();
    let cu = w.ok(&mine(miner, &again), "the largest batch, every account standing");
    eprintln!("cost: the largest batch of distinct flasks: {most} spends, {bytes} bytes of {PACKET}; {cu_new} CU with every $WARM account to make, {cu} CU with every one standing ({} CU a spend)", cu / most as u64);
    let mut one = 0;
    for k in 1..=60 {
        let spends: Vec<Spend> = (0..k).map(|i| Spend { flask: flasks[0], usd: 1, at: now, seq: 1_000 + i as u64 }).collect();
        if tx_bytes(&mine(miner, &spends), &miner) <= PACKET {
            one = k;
        }
    }
    let spends: Vec<Spend> = (0..one).map(|i| Spend { flask: flasks[0], usd: 1, at: now, seq: 1_000 + i as u64 }).collect();
    let cu_one = w.ok(&mine(miner, &spends), "one flask's spends, as many as fit");
    eprintln!("cost: one flask's spends in one transaction: {one} spends, {} bytes; {cu_one} CU", tx_bytes(&mine(miner, &spends), &miner));
    assert!(most >= 5, "a batch names at least five flasks");
    assert!(cu < DEFAULT_UNITS && cu_one < DEFAULT_UNITS, "a full batch whose accounts stand runs within a transaction's default units");
    if cu_new >= DEFAULT_UNITS {
        eprintln!("cost: a full batch whose accounts are all to make asks a compute budget of its own ({cu_new} CU), or a smaller batch");
    }
}

#[test]
fn the_accounts_and_the_program_cost_their_rent() {
    let w = World::new(WORD_S);
    let rent = |n: usize| w.m.sysvars.rent.minimum_balance(n);
    for (what, size) in [("the mine", Mine::SIZE), ("a flask's record", Flask::SIZE), ("a prior deposit", Prior::SIZE), ("the $WARM mint", 82), ("the vault, and each flask's $WARM account", 165)] {
        eprintln!("cost: {what}: {size} bytes, {} lamports of rent here, {} on mainnet at {MAINNET_LAMPORTS_A_BYTE} a byte", rent(size), (size as u64 + 128) * MAINNET_LAMPORTS_A_BYTE);
    }
    let elf = built::program_elf();
    let program_data = elf.len() as u64 + 45;
    let mainnet = (program_data + 128) * MAINNET_LAMPORTS_A_BYTE + (36 + 128) * MAINNET_LAMPORTS_A_BYTE;
    eprintln!(
        "cost: the program: {} bytes; its program data {program_data} bytes (the 45-byte header and the program), {} lamports on mainnet at {MAINNET_LAMPORTS_A_BYTE} a byte with the 128 of overhead, and its program account (36 bytes) {}: {} lamports, {}.{:09} SOL",
        elf.len(),
        (program_data + 128) * MAINNET_LAMPORTS_A_BYTE,
        (36 + 128) * MAINNET_LAMPORTS_A_BYTE,
        mainnet,
        mainnet / 1_000_000_000,
        mainnet % 1_000_000_000
    );
    assert!(elf.len() < 100_000, "the program stays small: {} bytes", elf.len());
}
