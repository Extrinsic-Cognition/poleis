//! THE POOL IS BORN AT THE LAUNCH (his Nov 22 2026 01:18:14 UTC "Yes, add it"): the SPL stake pool born by root 0's own sealed raw act, act 5
//! of the press's acts, as devnet's was born on Oct 29 2026 (poleis/ceremony/birth-pool.mjs). Held here:
//! - its Initialize, byte for byte and account by account, to the payload devnet's seal passed: the press's act built for devnet's
//!   addresses hashes to devnet's own proposal (38cafee3…), which executed there; every address of it as devnet's certificate names them and
//!   by Solana's own derivations;
//! - the makings its execute's transaction carries, built for the key that executes, held to Solana's system interface (a stranger's
//!   lamports at an address topped up and allocated, never a create they would bar);
//! - THE HOLE closed: a stranger's execute makes its accounts at its own addresses and only the base makes the pool's; the first sitting
//!   makes neither the pool nor its list, so until the act's own transaction nothing stands there for a stranger's Initialize; the reserve
//!   and the mint answer to this pool alone;
//! - the pool read back as root 0's, fault by fault; the need, at the chain's rent, read off an RPC of the test's own; the handoff, which
//!   waits for the pool on mainnet's rules; the bytes the birth was rehearsed on; the params' bounds; and the execute's size on the wire.
use poleis_client::{self as client, ACT_SIGNER, ACT_WRITABLE, CLASS_ACT, ROOT_UNIT};
use poleis_press::chain::{self, handoff_ready, KeySource, Sitting, TOKEN};
use poleis_press::loader::{self, LOADER_V3, SYSTEM};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::pool::*;
use poleis_press::record::Record;
use poleis_press::rpc::{Account, Cluster, Rpc};
use poleis_press::wire::{self, Key};
use poleis_press::{acts, build};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_loader_v3_interface::state::UpgradeableLoaderState;
use solana_pubkey::Pubkey;
use solana_system_interface::instruction as sys;
use std::collections::BTreeSet;
use std::str::FromStr as _;

/// Devnet's birth of Oct 29 2026: the ceremony's deployer (birth-certificate-devnet-2026-10-28.json), the stake pool program and the Poleis
/// it ran on, and its certificate.
const DEVNET_DEPLOYER: &str = "6t9gzYpssjVuNfRjazmDA5aBRssYgamS2BPRJ54G5qin";
const DEVNET_STAKE_POOL: &str = "DPoo15wWDqpPJJtS2MUZ49aRxqz5ZaaJCJP4z8bLuib";
const DEVNET_POLEIS: &str = "4627ahFCh78nC9NJ7Psb7B1NSjDkmnP8SPfvbt5MQz84";
const DEVNET_CERTIFICATE: &str = include_str!("../../ceremony/pool-birth-certificate-devnet-u0a.json");

fn key(s: &str) -> Pubkey {
    Pubkey::from_str(s).unwrap()
}
fn devnet() -> Pool {
    Pool::derive(&key(DEVNET_DEPLOYER), "u0a", 10, &key(DEVNET_STAKE_POOL), &key(DEVNET_POLEIS)).unwrap()
}
fn certificate() -> Value {
    serde_json::from_str(DEVNET_CERTIFICATE).unwrap()
}

/// Params of a launch's shape, every value given, the addresses of keys nobody holds (sha256 of words).
fn params(cluster: &str) -> Value {
    let s = |w: &str| k(w).to_string();
    json!({
        "cluster": cluster, "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "V", "desc": "", "defaultMint": s("pool dollar"), "planCap": 1, "planPeriod": 6480000, "dollarCap": 2, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("pool issuer") },
        "posiverse": { "name": "P", "desc": "", "chair": 2, "agents": [s("pool agent 1"), s("pool agent 2"), s("pool agent 3")], "charter": "11".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "https://optimism.fit/app/warm-token.json", "dollar": s("pool dollar"), "opening": "the founding", "miner": s("pool miner") },
        "router": { "monthCap": 1 },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1000000000u64, "poolQuote": 1000000000u64, "judgeSlots": 5000,
                    "crediter": s("pool crediter"), "split": ["treasury"], "plan": ["treasury", "act"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "seal": { "rpc": "https://api.mainnet-beta.solana.com" },
        "pool": { "maxValidators": 10, "tag": "lp1", "programSha256": "ab".repeat(32) }
    })
}
fn read(v: &Value) -> Params {
    Params::from_text(&v.to_string()).unwrap()
}
fn deployer() -> Pubkey {
    k("pool deployer")
}
fn the_pool(p: &Params) -> Pool {
    let (ids, ..) = build::compiled().unwrap();
    Pool::of(p, &deployer(), &ids.poleis).unwrap()
}
fn act_5(p: &Params) -> acts::Act {
    let (ids, ..) = build::compiled().unwrap();
    acts::acts(p, &ids, &chain::addresses(&ids, 77), &deployer()).unwrap().into_iter().find(|a| a.n == 5).expect("a fifth act")
}
/// Rent at mainnet's 5,080 lamports a byte, an account's 128 bytes of overhead counted (devnet's reserve held 1,666,240 for its 200).
fn rent(n: usize) -> Result<u64, String> {
    Ok((128 + n as u64) * 5_080)
}
fn same(ours: &Instruction, theirs: &Instruction, what: &str) {
    assert_eq!(ours.program_id, theirs.program_id, "{what}: the program");
    assert_eq!(ours.data, theirs.data, "{what}: the data");
    assert_eq!(ours.accounts, theirs.accounts, "{what}: the accounts, their writes and signatures");
}
/// Solana's system interface's create_account_with_seed as the press makes it: where the base pays, the interface leaves the base out of the
/// accounts and the press's own builder names it again, a signer (the deploy's named buffers are made in that form on every rehearsal's
/// chain); the data and every other account are the interface's.
fn seeded(from: &Pubkey, to: &Pubkey, base: &Pubkey, seed: &str, lamports: u64, space: u64, owner: &Pubkey) -> Instruction {
    let mut ix = sys::create_account_with_seed(from, to, base, seed, lamports, space, owner);
    if from == base {
        ix.accounts.push(AccountMeta::new_readonly(*base, true));
    }
    ix
}
fn acc(owner: &Pubkey, data: &[u8]) -> Account {
    Account { lamports: 1, owner: owner.to_string(), executable: false, data: data.to_vec() }
}
/// A reserve as the stake program writes it once initialized: [1:4][its rent:8][staker][withdrawer][no lockup], in its 200 bytes.
fn laid_reserve(withdraw: &Pubkey) -> Vec<u8> {
    let mut d = vec![0u8; RESERVE_LEN];
    d[..4].copy_from_slice(&1u32.to_le_bytes());
    d[4..12].copy_from_slice(&1_666_240u64.to_le_bytes());
    d[12..44].copy_from_slice(withdraw.as_ref());
    d[44..76].copy_from_slice(withdraw.as_ref());
    d
}
/// A mint as SPL Token writes it: [Some:4][its authority][supply 0][9 decimals][initialized][no freeze authority].
fn laid_mint(withdraw: &Pubkey) -> Vec<u8> {
    let mut d = vec![0u8; MINT_LEN];
    d[..4].copy_from_slice(&1u32.to_le_bytes());
    d[4..36].copy_from_slice(withdraw.as_ref());
    d[44] = 9;
    d[45] = 1;
    d
}
/// Root 0's vault of a mint as Poleis's create_vault leaves it: a token account of the mint whose authority is root 0's treasury.
fn laid_vault(mint: &Pubkey) -> Vec<u8> {
    let mut d = vec![0u8; warm_mine::token::ACCOUNT_LEN];
    d[..32].copy_from_slice(mint.as_ref());
    d[32..64].copy_from_slice(client::treasury_pda(ROOT_UNIT).as_ref());
    d[108] = 1;
    d
}
/// A pool as the stake pool program's Initialize leaves it, laid at the offsets birth-pool.mjs read devnet's by.
fn laid_pool(p: &Pool) -> Vec<u8> {
    let mut d = vec![0u8; POOL_LEN];
    d[0] = TYPE_POOL;
    for (o, at) in [(1, p.agent), (33, p.agent), (65, p.deposit), (98, p.list), (130, p.reserve), (162, p.mint), (194, p.vault), (226, TOKEN)] {
        d[o..o + 32].copy_from_slice(at.as_ref());
    }
    d[330..338].copy_from_slice(&1u64.to_le_bytes());
    d[338..346].copy_from_slice(&1u64.to_le_bytes());
    d
}

#[test]
fn every_address_of_the_pool_is_devnets_and_solanas_own_derivation() {
    let p = devnet();
    let cert = certificate();
    for (name, got) in [("pool", p.pool), ("list", p.list), ("reserve", p.reserve), ("mint", p.mint), ("withdraw", p.withdraw), ("agent", p.agent), ("vault", p.vault)] {
        assert_eq!(got.to_string(), cert["addresses"][name].as_str().unwrap(), "{name}: as devnet's certificate names it");
    }
    assert_eq!(p.seeds_json(), cert["seeds"], "the seeds: the tag and one letter");
    // the program's default deposit authority, as devnet read it back off the pool of Oct 29 20:29 UTC (THE-POOL-WAITS-ON-HIS-KEY-LANDING.md)
    assert_eq!(authority_of(&key("AtkWUCFBsv8jM9M8diDyzgyvben7RfzK9AMYUFY2pu8j"), &key(DEVNET_STAKE_POOL), DEPOSIT), key("2sQ4WMgScycvo7b6QWAooCPUWnVEKVXCuaHwkZorThaG"));
    // the sitting's: the four seeded by Solana's own create_with_seed from the deployer, the authorities its own PDAs, the rest Poleis's
    let (ids, ..) = build::compiled().unwrap();
    let s = the_pool(&read(&params("mainnet")));
    for (got, seed, owner) in [(s.pool, "lp1p", STAKE_POOL), (s.list, "lp1l", STAKE_POOL), (s.reserve, "lp1r", STAKE), (s.mint, "lp1m", TOKEN)] {
        assert_eq!(got, Pubkey::create_with_seed(&deployer(), seed, &owner).unwrap(), "{seed}");
    }
    assert_eq!((s.withdraw, s.deposit), (Pubkey::find_program_address(&[s.pool.as_ref(), b"withdraw"], &STAKE_POOL).0, Pubkey::find_program_address(&[s.pool.as_ref(), b"deposit"], &STAKE_POOL).0));
    assert_eq!((s.agent, s.vault, s.treasury), (client::agent_pda(ROOT_UNIT), client::vault_pda(ROOT_UNIT, &s.mint), client::treasury_pda(ROOT_UNIT)), "the polis's hand, root 0's vault of the mint, root 0's treasury");
    assert_eq!(STAKE_POOL.to_string(), "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy", "mainnet's stake pool program");
    // another base, another pool: the pool is its base's
    let other = Pool::of(&read(&params("mainnet")), &k("another deployer"), &ids.poleis).unwrap();
    for (a, b) in [(other.pool, s.pool), (other.list, s.list), (other.reserve, s.reserve), (other.mint, s.mint), (other.vault, s.vault), (other.withdraw, s.withdraw)] {
        assert_ne!(a, b);
    }
}

#[test]
fn the_initialize_is_the_payload_devnets_seal_passed_and_executed() {
    let p = devnet();
    let cert = certificate();
    assert_eq!(
        wire::hex(&p.initialize_data()),
        "00010000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000",
        "[0][the epoch fee 1/1][the withdrawal fee 0/0][the deposit fee 0/0][the referral fee 0][10 validators]: birth-pool.mjs's"
    );
    assert_eq!(
        p.initialize_accounts(),
        vec![(p.pool, ACT_WRITABLE), (p.agent, ACT_SIGNER), (p.agent, 0), (p.withdraw, 0), (p.list, ACT_WRITABLE), (p.reserve, 0), (p.mint, ACT_WRITABLE), (p.vault, ACT_WRITABLE), (TOKEN, 0)],
        "the nine accounts and their flags, birth-pool.mjs's"
    );
    let payload = p.payload();
    assert_eq!((payload.class(), payload.bytes().len()), (CLASS_ACT, 388), "an ACT_RAW of 388 bytes");
    let manifest = wire::sha256(b"unit 0 gives birth to its stake pool: u0a");
    let nonce = u64::from(u32::from_be_bytes(wire::sha256(b"birth-pool u0a")[..4].try_into().unwrap()));
    assert_eq!(nonce, 70_550_906, "birth-pool.mjs's nonce: the first eight hex of a sha256");
    assert_eq!(
        wire::hex(&client::proposal_hash(CLASS_ACT, ROOT_UNIT, &manifest, &payload.hash(), nonce)),
        cert["proposal"].as_str().unwrap(),
        "the proposal devnet's seal passed and executed (its execute 21,746 CU, 673 bytes)"
    );
}

#[test]
fn act_5_is_the_pools_birth_by_the_polis_hand() {
    let p = read(&params("mainnet"));
    let (ids, ..) = build::compiled().unwrap();
    let all = acts::acts(&p, &ids, &chain::addresses(&ids, 77), &deployer()).unwrap();
    assert_eq!(all.iter().map(|a| a.n).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6, 7, 8], "the pool's birth the fifth, the mine's miner after it, then the treasury's dollar cap and the router's month");
    let (five, pool) = (&all[4], the_pool(&p));
    assert_eq!(five.payload, pool.payload().bytes(), "its payload: the pool's Initialize, a raw act of root 0");
    assert_eq!((five.class, five.impacted.clone(), five.convene(), five.nonce), (CLASS_ACT, vec![ROOT_UNIT], true, 5), "an ACT_RAW of root 0, unit 1 convened by the crank, the fifth nonce");
    assert_eq!((five.needs, five.before.clone()), (Some(STAKE_POOL), pool.in_the_execute()));
    assert_eq!(five.before.iter().map(|s| (s.seed.as_str(), s.space, s.owner)).collect::<Vec<_>>(), vec![("lp1p", 611, STAKE_POOL), ("lp1l", 9 + 73 * 10, STAKE_POOL)], "the pool's account and its list for ten validators");
    assert!(five.payload.len() <= poleis_client::MAX_PAYLOAD);
    assert!(all[..4].iter().all(|a| a.before.is_empty() && a.needs.is_none()), "the four before it make nothing before their executes");
}

#[test]
fn its_execute_makes_the_pool_and_its_list_first_by_the_key_that_executes() {
    let p = read(&params("mainnet"));
    let (five, pool, d) = (act_5(&p), the_pool(&p), deployer());
    let ixs = five.execute_ixs(&d, rent, |_| Ok(None)).unwrap();
    assert_eq!(ixs.len(), 3);
    same(&ixs[0], &seeded(&d, &pool.pool, &d, "lp1p", 739 * 5_080, 611, &STAKE_POOL), "the pool's account, the deployer its base and payer");
    same(&ixs[1], &seeded(&d, &pool.list, &d, "lp1l", 867 * 5_080, 739, &STAKE_POOL), "its list");
    same(&ixs[2], &five.execute_by(&d), "then the execute, the deployer its crank");
    assert_eq!(ixs[2].accounts[0].pubkey, d);
    // a stranger's lamports at the pool's address: topped up to its rent and allocated by the base, never a create the dust would bar
    let dust = acc(&SYSTEM, &[]);
    let ixs = five.execute_ixs(&d, rent, |a| Ok((*a == pool.pool).then(|| dust.clone()))).unwrap();
    same(&ixs[0], &sys::transfer(&d, &pool.pool, 739 * 5_080 - 1), "the rent's shortfall");
    same(&ixs[1], &sys::allocate_with_seed(&pool.pool, &d, "lp1p", 611, &STAKE_POOL), "its space and its owner, by the base's signature");
    same(&ixs[2], &seeded(&d, &pool.list, &d, "lp1l", 867 * 5_080, 739, &STAKE_POOL), "its list, made");
    // more than its rent there already: no transfer
    let rich = Account { lamports: 10_000_000, ..dust.clone() };
    let ixs = five.execute_ixs(&d, rent, |a| Ok((*a == pool.list).then(|| rich.clone()))).unwrap();
    assert_eq!(ixs.len(), 3);
    same(&ixs[1], &sys::allocate_with_seed(&pool.list, &d, "lp1l", 739, &STAKE_POOL), "its list, allocated over the lamports");
    // anything else at its address is no account the sitting made: refused, nothing built
    let other = acc(&STAKE_POOL, &[0; 611]);
    assert!(five.execute_ixs(&d, rent, |a| Ok((*a == pool.pool).then(|| other.clone()))).unwrap_err().contains("none this sitting made"));
}

/// THE HOLE: the stake pool's Initialize asks only its manager to sign. A pool account that stood empty on the chain would be anyone's, so
/// it never stands empty: it is made in the transaction that initializes it, at an address only its base makes.
#[test]
fn a_strangers_execute_makes_its_accounts_at_its_own_addresses_and_fails_whole() {
    let p = read(&params("mainnet"));
    let (five, pool, d) = (act_5(&p), the_pool(&p), deployer());
    let stranger = Key::from_seed(&[9; 32]);
    let theirs = five.execute_ixs(&stranger.address, rent, |_| Ok(None)).unwrap();
    let made: Vec<Pubkey> = theirs.iter().filter(|i| i.program_id == SYSTEM).map(|i| i.accounts[1].pubkey).collect();
    assert_eq!(made, vec![Pubkey::create_with_seed(&stranger.address, "lp1p", &STAKE_POOL).unwrap(), Pubkey::create_with_seed(&stranger.address, "lp1l", &STAKE_POOL).unwrap()], "the stranger's makings stand at the stranger's own addresses");
    assert!(!made.contains(&pool.pool) && !made.contains(&pool.list), "never at the payload's");
    // whoever carries it out, what executes is the sealed payload: the Initialize of the deployer's pool and list, and none of the stranger's
    let exec = theirs.last().unwrap();
    assert!(exec.accounts.iter().any(|m| m.pubkey == pool.pool && m.is_writable) && exec.accounts.iter().any(|m| m.pubkey == pool.list && m.is_writable));
    assert!(!exec.accounts.iter().any(|m| made.contains(&m.pubkey)), "the stranger's own accounts are no account of the act");
    // only the base makes an account at its with_seed address: the deployer's pool made in a stranger's transaction asks the deployer's
    // signature, the stranger paying or not
    for forged in [
        [pool.in_the_execute()[0].making(&d, 1, None).unwrap(), vec![five.execute_by(&stranger.address)]].concat(),
        vec![loader::create_account_with_seed(&stranger.address, &pool.pool, &d, "lp1p", 1, 611, &STAKE_POOL), five.execute_by(&stranger.address)],
        vec![loader::allocate_with_seed(&pool.pool, &d, "lp1p", 611, &STAKE_POOL), five.execute_by(&stranger.address)],
    ] {
        let e = wire::signed(&forged, &stranger.address, &[7; 32], &[&stranger]).unwrap_err();
        assert!(e.contains(&d.to_string()) && e.contains("signs this transaction"), "{e}");
    }
    // the stranger's own transaction is whole and its own to sign; its Initialize meets no account at the payload's pool, since nothing the
    // stranger made is there, and a transaction lands whole or not at all (Solana's own atomicity): the execute fails, nothing of it stands
    assert!(wire::signed(&theirs, &stranger.address, &[7; 32], &[&stranger]).is_ok());
}

#[test]
fn the_first_sitting_makes_no_pool_account_so_a_strangers_initialize_finds_none() {
    let p = read(&params("mainnet"));
    let (pool, d) = (the_pool(&p), deployer());
    let (ids, ..) = build::compiled().unwrap();
    let (made, ixs) = pool.ahead(rent(RESERVE_LEN).unwrap(), rent(MINT_LEN).unwrap(), [None, None, None]).unwrap();
    assert_eq!(made, vec!["its reserve", "its mint", "root 0's vault of its mint"]);
    assert_eq!(ixs.len(), 5);
    same(&ixs[0], &seeded(&d, &pool.reserve, &d, "lp1r", 328 * 5_080, 200, &STAKE), "the reserve, its rent exactly");
    // the stake program's Initialize as devnet's birth sent it: [0:4][staker][withdrawer][no lockup: 48 zeros], the reserve and the rent sysvar
    assert_eq!((ixs[1].program_id, ixs[1].data.clone()), (STAKE, [&[0u8; 4][..], &pool.withdraw.to_bytes()[..], &pool.withdraw.to_bytes()[..], &[0u8; 48][..]].concat()));
    assert_eq!(ixs[1].accounts, vec![AccountMeta::new(pool.reserve, false), AccountMeta::new_readonly(loader::RENT_SYSVAR, false)]);
    same(&ixs[2], &seeded(&d, &pool.mint, &d, "lp1m", 210 * 5_080, 82, &TOKEN), "the mint");
    // InitializeMint2 as devnet's: [20][9][the withdraw authority][no freeze authority], the mint
    assert_eq!((ixs[3].program_id, ixs[3].data.clone(), ixs[3].accounts.clone()), (TOKEN, [&[20u8, 9][..], &pool.withdraw.to_bytes()[..], &[0u8][..]].concat(), vec![AccountMeta::new(pool.mint, false)]));
    assert_eq!(ixs[4], client::create_vault(d, ROOT_UNIT, pool.mint, TOKEN), "root 0's vault of the mint, by Poleis's create_vault");
    // no instruction of the first sitting names the pool's address or its list's: none makes them, and until act 5's own transaction makes
    // and initializes them at once, nothing stands there for a stranger's Initialize to take
    assert!(!ixs.iter().any(|i| i.accounts.iter().any(|m| m.pubkey == pool.pool || m.pubkey == pool.list)));
    let written: BTreeSet<Pubkey> = ixs.iter().flat_map(|i| i.accounts.iter().filter(|m| m.is_writable).map(|m| m.pubkey)).collect();
    assert_eq!(written, BTreeSet::from([d, pool.reserve, pool.mint, pool.vault]), "it writes the deployer, the reserve, the mint and the vault, and nothing else");
    // the reserve and the mint answer to this pool alone: a stranger's pool (its own address, so its own withdraw authority) is refused them
    let (reserve, mint, vault) = (laid_reserve(&pool.withdraw), laid_mint(&pool.withdraw), laid_vault(&pool.mint));
    reserve_held(&STAKE.to_string(), &reserve, &pool.withdraw).unwrap();
    mint_held(&TOKEN.to_string(), &mint, &pool.withdraw).unwrap();
    let theirs = Pool::derive(&k("a stranger"), "lp1", 10, &STAKE_POOL, &ids.poleis).unwrap();
    assert_ne!(theirs.withdraw, pool.withdraw);
    assert!(reserve_held(&STAKE.to_string(), &reserve, &theirs.withdraw).unwrap_err().contains("no reserve of this pool"));
    assert!(mint_held(&TOKEN.to_string(), &mint, &theirs.withdraw).unwrap_err().contains("no mint of this pool"));
    // what stands as the pool's is skipped: nothing is sent
    let (made, ixs) = pool.ahead(1, 1, [Some(&acc(&STAKE, &reserve)), Some(&acc(&TOKEN, &mint)), Some(&acc(&TOKEN, &vault))]).unwrap();
    assert!(made.is_empty() && ixs.is_empty());
    // a stranger's lamports at the reserve's address: made over them, topped up and allocated by the base
    let (made, ixs) = pool.ahead(rent(RESERVE_LEN).unwrap(), 1, [Some(&acc(&SYSTEM, &[])), Some(&acc(&TOKEN, &mint)), Some(&acc(&TOKEN, &vault))]).unwrap();
    assert_eq!(made, vec!["its reserve"]);
    same(&ixs[0], &sys::transfer(&d, &pool.reserve, 328 * 5_080 - 1), "the reserve's rent, less the stranger's lamport");
    same(&ixs[1], &sys::allocate_with_seed(&pool.reserve, &d, "lp1r", 200, &STAKE), "its space and owner, by the base");
    // one standing that answers to another authority stops the step, by its words
    assert!(pool.ahead(1, 1, [Some(&acc(&STAKE, &laid_reserve(&theirs.withdraw))), None, None]).unwrap_err().contains("no reserve of this pool"));
    assert!(pool.ahead(1, 1, [None, Some(&acc(&TOKEN, &laid_mint(&theirs.withdraw))), None]).unwrap_err().contains("no mint of this pool"));
    assert!(pool.ahead(1, 1, [None, None, Some(&acc(&TOKEN, &laid_vault(&theirs.mint)))]).is_err(), "a vault of another mint");
}

#[test]
fn the_pool_is_held_as_root_0s_before_the_press_says_it_stands() {
    let p = devnet();
    let cert = certificate();
    let read_back = pool_held(DEVNET_STAKE_POOL, &laid_pool(&p), &p).unwrap();
    for f in ["accountType", "manager", "staker", "mint", "feeAccount", "epochFee"] {
        assert_eq!(read_back[f], cert["read"][f], "{f}: as devnet's was read back");
    }
    let stranger = k("a stranger").to_bytes().to_vec();
    let faults: Vec<(&str, usize, Vec<u8>, &str)> = vec![
        ("another manager", 1, stranger.clone(), "manager"),
        ("another staker", 33, stranger.clone(), "staker"),
        ("a deposit authority of its own (deposits closed to all but it)", 65, stranger.clone(), "depositAuthority"),
        ("another list", 98, stranger.clone(), "list"),
        ("another reserve", 130, stranger.clone(), "reserve"),
        ("another mint", 162, stranger.clone(), "mint"),
        ("another fee account (the yield to a key)", 194, stranger.clone(), "feeAccount"),
        ("another token program", 226, stranger.clone(), "tokenProgram"),
        ("an epoch fee of 0/1 (no yield to the seal)", 338, 0u64.to_le_bytes().to_vec(), "epochFee"),
        ("an epoch fee of 1/2", 330, 2u64.to_le_bytes().to_vec(), "epochFee"),
        ("no pool yet (uninitialized)", 0, vec![0], "accountType"),
    ];
    for (what, at, bytes, says) in faults {
        let mut d = laid_pool(&p);
        d[at..at + bytes.len()].copy_from_slice(&bytes);
        let e = pool_held(DEVNET_STAKE_POOL, &d, &p).expect_err(what);
        assert!(e.contains(says), "{what}: refused, by other words than its own: {e}");
    }
    assert!(pool_held(&STAKE.to_string(), &laid_pool(&p), &p).is_err(), "another program's account");
    assert!(pool_held(DEVNET_STAKE_POOL, &laid_pool(&p)[..610], &p).is_err(), "another length");
    // the list: the program's, of type 2, its max validators the params'
    let mut list = vec![0u8; list_len(10)];
    list[0] = TYPE_LIST;
    list[1..5].copy_from_slice(&10u32.to_le_bytes());
    list_held(DEVNET_STAKE_POOL, &list, &p).unwrap();
    let mut other = list.clone();
    other[1..5].copy_from_slice(&9u32.to_le_bytes());
    assert!(list_held(DEVNET_STAKE_POOL, &other, &p).is_err(), "another max validators");
    assert!(list_held(DEVNET_STAKE_POOL, &list[..list.len() - 73], &p).is_err(), "a list of nine");
    // the reserve's faults and the mint's
    let w = p.withdraw;
    for (what, at, bytes) in [("not initialized", 0usize, vec![0u8]), ("another staker", 12, stranger.clone()), ("another withdrawer", 44, stranger.clone()), ("a lockup", 84, vec![1u8])] {
        let mut d = laid_reserve(&w);
        d[at..at + bytes.len()].copy_from_slice(&bytes);
        assert!(reserve_held(&STAKE.to_string(), &d, &w).is_err(), "a reserve with {what}");
    }
    assert!(reserve_held(&TOKEN.to_string(), &laid_reserve(&w), &w).is_err(), "not the stake program's");
    for (what, at, bytes) in [("no mint authority", 0usize, vec![0u8]), ("another mint authority", 4, stranger.clone()), ("six decimals", 44, vec![6u8]), ("not initialized", 45, vec![0u8]), ("a freeze authority", 46, vec![1u8])] {
        let mut d = laid_mint(&w);
        d[at..at + bytes.len()].copy_from_slice(&bytes);
        assert!(mint_held(&TOKEN.to_string(), &d, &w).is_err(), "a mint with {what}");
    }
}

/// THE HANDOFF IS LAST, and the pool is born before it: its making needs the deployer's key, which the handoff's last words tell him to
/// delete. Only a rehearsal's chain with no stake pool program does not wait for it; on mainnet's rules nothing passes without it.
#[test]
fn the_handoff_waits_for_the_pool_on_mainnets_rules() {
    use poleis_treasury::{MODE_COUNCIL, MODE_GENESIS};
    let born = PoolAt::Born(json!({ "epochFee": "1/1" }));
    handoff_ready(MODE_COUNCIL, true, true, true, &born, Cluster::Mainnet).unwrap();
    handoff_ready(MODE_COUNCIL, true, true, true, &born, Cluster::Localnet).unwrap();
    for rules in [Cluster::Mainnet, Cluster::Localnet] {
        assert!(handoff_ready(MODE_COUNCIL, true, true, true, &PoolAt::Unborn, rules).unwrap_err().contains("the handoff waits for the pool's birth"), "{rules:?}: a chain that bears the pool waits for it");
    }
    assert!(handoff_ready(MODE_COUNCIL, true, true, true, &PoolAt::NoProgram, Cluster::Mainnet).unwrap_err().contains(NOT_MAINNET), "on mainnet the program stands, or this is no mainnet");
    handoff_ready(MODE_COUNCIL, true, true, true, &PoolAt::NoProgram, Cluster::Localnet).unwrap();
    // and the order before it stands: genesis, the fund, the mine, the board are said first
    assert!(handoff_ready(MODE_GENESIS, true, true, true, &PoolAt::Unborn, Cluster::Mainnet).unwrap_err().contains("still in genesis"));
    assert!(handoff_ready(MODE_COUNCIL, false, true, true, &PoolAt::Unborn, Cluster::Mainnet).unwrap_err().contains("the seal's first act"));
}

/// THE BYTES THE BIRTH WAS REHEARSED ON: the stake pool program is upgradeable (on mainnet by a multisig); the sitting records its executable
/// bytes at its first read and holds every later read to them and to the params' hash.
#[test]
fn the_birth_runs_on_the_bytes_it_was_rehearsed_on() {
    let elf: Vec<u8> = (0..5_000u32).map(|i| (i % 251) as u8 | 1).collect();
    let laid = |slot: u64, code: &[u8], room: usize| -> Vec<u8> {
        let mut d = bincode::serialize(&UpgradeableLoaderState::ProgramData { slot, upgrade_authority_address: Some(k("a multisig")) }).unwrap();
        d.extend_from_slice(code);
        d.resize(loader::PROGRAMDATA_HEADER + room, 0);
        d
    };
    let now = Program::of(&STAKE_POOL, &laid(469_669_929, &elf, elf.len() + 10_240)).unwrap();
    assert_eq!((now.slot, now.bytes, now.sha256, now.authority), (469_669_929, elf.len(), wire::sha256(&elf), Some(k("a multisig"))), "its executable bytes: those after the header, the zeros of its room trimmed");
    assert_eq!(now.programdata, solana_loader_v3_interface::get_program_data_address(&STAKE_POOL));
    program_held(&now, None, None).unwrap();
    program_held(&now, Some(&now), Some(now.sha256)).unwrap();
    // the same bytes copied to a rehearsal's chain at its genesis (another slot, another room): the same program
    let copied = Program::of(&STAKE_POOL, &laid(0, &elf, elf.len())).unwrap();
    program_held(&copied, Some(&now), Some(now.sha256)).unwrap();
    // upgraded since: one byte moved, at a later slot: refused, both hashes said
    let mut moved = elf.clone();
    moved[100] ^= 0x40;
    let upgraded = Program::of(&STAKE_POOL, &laid(469_700_000, &moved, moved.len())).unwrap();
    let e = program_held(&upgraded, Some(&now), None).unwrap_err();
    assert!(e.contains("runs other bytes than the sitting recorded") && e.contains(&wire::hex(&now.sha256)) && e.contains(&wire::hex(&upgraded.sha256)), "{e}");
    // a byte added at its end: other bytes too
    let longer = Program::of(&STAKE_POOL, &laid(469_669_929, &[&elf[..], &[7u8][..]].concat(), elf.len() + 10)).unwrap();
    assert!(program_held(&longer, Some(&now), None).is_err());
    // the params name the rehearsed hash: other bytes are refused even at the sitting's first read
    let e = program_held(&upgraded, None, Some(now.sha256)).unwrap_err();
    assert!(e.contains("not the") && e.contains("pool.programSha256"), "{e}");
    // what the record keeps reads back as it was
    assert_eq!(Program::from_json(&now.json()).unwrap(), Some(now.clone()));
    assert_eq!(Program::from_json(&Value::Null).unwrap(), None);
}

#[test]
fn the_pools_params_are_held_before_anything_starts() {
    let (ids, ..) = build::compiled().unwrap();
    let checked = |v: &Value| chain::params_checked(&read(v), &ids, &chain::addresses(&ids, 77));
    checked(&params("mainnet")).unwrap();
    let with = |field: &str, value: Value| {
        let mut v = params("mainnet");
        v["pool"][field] = value;
        v
    };
    assert_eq!(MAX_VALIDATORS, 143_640, "a list of 9 + 73 × 143,640 bytes is the largest account Solana makes");
    for (what, v) in [("a tag of one", with("tag", json!("a"))), ("a tag of eight", with("tag", json!("abcd1234"))), ("one validator", with("maxValidators", json!(1))), ("the most an account holds", with("maxValidators", json!(143_640)))] {
        checked(&v).unwrap_or_else(|e| panic!("{what}: {e}"));
    }
    for (bad, says) in [
        (with("tag", json!("")), "pool.tag"),
        (with("tag", json!("Swarm")), "pool.tag"),
        (with("tag", json!("ninechars")), "pool.tag"),
        (with("tag", json!("lp-1")), "pool.tag"),
        (with("maxValidators", json!(0)), "pool.maxValidators"),
        (with("maxValidators", json!(143_641)), "pool.maxValidators"),
        (with("maxValidators", json!("ten")), "pool.maxValidators"),
        (with("programSha256", Value::Null), "pool.programSha256"),
        (with("programSha256", json!("00")), "pool.programSha256"),
        (with("programSha256", json!("PLACEHOLDER: his")), "not his yet"),
    ] {
        let e = checked(&bad).expect_err(says);
        assert!(e.contains(says), "{says}: refused, by other words than its own: {e}");
    }
    // a launch's params with no pool are no launch's: the pool is born at it
    let mut v = params("mainnet");
    v.as_object_mut().unwrap().remove("pool");
    assert!(checked(&v).unwrap_err().contains("pool."));
    // on a rehearsal's chain the hash may be left out: the sitting records what it reads
    let mut v = params("localnet");
    v["pool"].as_object_mut().unwrap().remove("programSha256");
    checked(&v).unwrap();
}

#[test]
fn the_execute_and_its_two_makings_are_one_transaction() {
    let p = read(&params("mainnet"));
    let five = act_5(&p);
    let (clean, dust) = (acts::execute_bytes(&p, &five, &deployer(), false).unwrap(), acts::execute_bytes(&p, &five, &deployer(), true).unwrap());
    let mut priced = params("mainnet");
    priced["priorityMicroLamports"] = json!(10_000);
    let priced = read(&priced);
    let five_priced = act_5(&priced);
    let (priced_clean, priced_dust) = (acts::execute_bytes(&priced, &five_priced, &deployer(), false).unwrap(), acts::execute_bytes(&priced, &five_priced, &deployer(), true).unwrap());
    let proposed = poleis_press::floor::proposal_bytes(&p, &five, &poleis_press::floor::placeholder_owner()).unwrap();
    eprintln!(
        "  act 5's execute: {clean} bytes with its two makings; {dust} with a stranger's lamports at both addresses; {priced_clean} and {priced_dust} with a priority fee's two compute budget instructions; a seat owner's proposal of it {proposed} bytes; of {}",
        wire::PACKET
    );
    for n in [clean, dust, priced_clean, priced_dust, proposed] {
        assert!(n <= wire::PACKET, "{n} bytes");
    }
    assert!(dust > clean && priced_clean > clean);
}

mod dir {
    pub struct Dir(pub std::path::PathBuf);
    impl Dir {
        pub fn new(what: &str) -> Self {
            static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let p = std::env::temp_dir().join(format!("press-pool-{what}-{}-{n}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn act_5s_set_is_written_only_where_the_pool_can_be_born() {
    let p = read(&params("mainnet"));
    let (ids, ..) = build::compiled().unwrap();
    let pool = the_pool(&p);
    for birth in [false, true] {
        let d = dir::Dir::new("sets");
        let files = acts::write(&d.0, &p, &ids, &chain::addresses(&ids, 77), poleis_press::rpc::MAINNET_GENESIS, &deployer(), birth).unwrap();
        let sets: Vec<Value> = files.iter().filter(|f| !f.ends_with("index.json")).map(|f| serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap()).collect();
        assert_eq!(sets.len(), if birth { 8 } else { 7 }, "the fifth set only where the stake pool program stands (the sixth, the mine's miner, the seventh and the eighth, the treasury's dollar cap and the router's month, everywhere)");
        if birth {
            let five = &sets[4];
            assert_eq!((five["act"]["n"].clone(), five["pool"]["addresses"]["pool"].clone()), (json!(5), json!(pool.pool.to_string())));
            assert_eq!(five["before"].as_array().unwrap().iter().map(|b| b["at"].as_str().unwrap().to_string()).collect::<Vec<_>>(), vec![pool.pool.to_string(), pool.list.to_string()], "the makings, at the deployer's own addresses");
            assert!(five["crank"].as_array().unwrap().iter().any(|c| c.as_str().unwrap().contains("its execute's transaction makes first")));
            assert!(five.get("transactions").is_none(), "no transaction of anyone's in an act's set: its proposal and the floor's votes are the seat owners' own (press vote)");
        }
    }
}

/// A JSON-RPC of the test's own on 127.0.0.1 (the press's RPC speaks to this machine in a rehearsal): rent at mainnet's 5,080 lamports a byte,
/// an account's 128 bytes of overhead counted, and the accounts `standing` names; every other account none, every other method refused.
fn an_rpc(standing: Vec<(Pubkey, Account)>) -> String {
    use std::io::{Read as _, Write as _};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { continue };
            let (mut buf, mut part) = (Vec::new(), [0u8; 4096]);
            let body = loop {
                match s.read(&mut part) {
                    Ok(0) | Err(_) => break None,
                    Ok(n) => buf.extend_from_slice(&part[..n]),
                }
                if let Some(at) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buf[..at]).to_ascii_lowercase();
                    let len: usize = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse().ok()).unwrap_or(0);
                    while buf.len() < at + 4 + len {
                        match s.read(&mut part) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => buf.extend_from_slice(&part[..n]),
                        }
                    }
                    break Some(buf[at + 4..].to_vec());
                }
            };
            let Some(body) = body else { continue };
            let req: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            let mut answer = match req["method"].as_str() {
                Some("getMinimumBalanceForRentExemption") => json!({ "result": rent(req["params"][0].as_u64().unwrap_or(0) as usize).unwrap() }),
                Some("getAccountInfo") => {
                    let at = req["params"][0].as_str().unwrap_or_default();
                    let value = standing.iter().find(|(key, _)| key.to_string() == at).map(|(_, a)| json!({ "lamports": a.lamports, "owner": a.owner, "executable": a.executable, "data": [wire::b64(&a.data), "base64"], "rentEpoch": 0, "space": a.data.len() }));
                    json!({ "result": { "context": { "slot": 1 }, "value": value } })
                }
                m => json!({ "error": { "code": -32601, "message": format!("the test's RPC answers no {m:?}") } }),
            };
            answer["jsonrpc"] = json!("2.0");
            answer["id"] = json!(1);
            let text = answer.to_string();
            let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}", text.len());
        }
    });
    url
}

/// A sitting on that RPC: a rehearsal's params, nothing deployed, nothing founded.
fn a_sitting(url: &str, at: &std::path::Path) -> Sitting {
    let mut v = params("localnet");
    v["pool"] = json!({ "maxValidators": 10, "tag": "lp1" });
    let (ids, ..) = build::compiled().unwrap();
    Sitting {
        dir: at.to_path_buf(),
        copy: std::path::PathBuf::new(),
        rec: Record::open(at).unwrap(),
        p: read(&v),
        rpc: Rpc::new(url, Cluster::Localnet).unwrap(),
        ids,
        deployer: deployer(),
        keys: KeySource::Seed([0; 32]),
        at: chain::addresses(&ids, 77),
    }
}

/// THE NEED COUNTS THE POOL, at the chain's own rent, as `press need` and `press deploy` read it: the three made ahead always (the reserve,
/// the mint, root 0's vault of the mint); the pool's account, its list, unit 1's convened session of act 5 and the act's crank where the
/// stake pool program stands; nothing more once they stand.
#[test]
fn the_need_counts_the_pool_at_the_chains_rent() {
    let d = dir::Dir::new("need");
    let bins: Vec<(&'static str, Vec<u8>)> = vec![("market", vec![1; 1_000]), ("warm", vec![2; 1_000]), ("poleis", vec![3; 1_000])];
    let r = |n: usize| rent(n).unwrap();
    let line = |lines: &[(String, u64)], starts: &str| lines.iter().find(|(w, _)| w.starts_with(starts)).map(|l| l.1);
    let mut v = params("localnet");
    v["pool"] = json!({ "maxValidators": 10, "tag": "lp1" });
    let pool = the_pool(&read(&v));
    // 1. a chain with no stake pool program (the default rehearsal's): the three made ahead counted; the birth not run, and not counted
    let (none, lines) = a_sitting(&an_rpc(vec![]), &d.0).need(&bins).unwrap();
    assert_eq!(line(&lines, "the pool, made ahead"), Some(r(200) + r(82) + r(165)));
    assert_eq!(r(200) + r(82) + r(165), 4_221_480, "the reserve, the mint and the vault at 5,080 a byte");
    assert_eq!(line(&lines, NOT_RUN), Some(0));
    // 2. where the program stands: the birth counted, its pool and its list and unit 1's session of the act, and the act's crank (six
    // transactions and the two agents' votes' signatures; the proposal and the floor's votes are the seat owners' own)
    let program = Account { lamports: 1, owner: LOADER_V3.to_string(), executable: true, data: [&2u32.to_le_bytes()[..], &[0u8; 32][..]].concat() };
    let (with, lines) = a_sitting(&an_rpc(vec![(STAKE_POOL, program.clone())]), &d.0).need(&bins).unwrap();
    let session = client::Session::SIZE + client::Session::ROSTER_HEADER + client::Session::CONVENER_TAIL;
    let birth = r(611) + r(9 + 73 * 10) + r(session);
    assert_eq!(line(&lines, "the pool's birth (act 5)"), Some(birth));
    assert_eq!(r(611) + r(739), 8_158_480, "the pool's account and its list for ten validators at 5,080 a byte");
    assert_eq!(r(611) + r(739) + r(200) + r(82) + r(165), 12_379_960, "the five accounts, as devnet's of Oct 29 cost at the same rate");
    assert_eq!(with - none, birth + 6 * 5_000 + 2 * 5_000, "and the act's crank's fees");
    eprintln!("  the need: {none} lamports with no stake pool program, {with} where it stands; the birth's line {birth} (unit 1's session {} of it)", r(session));
    // 3. everything of the pool standing: it costs nothing more
    let standing = vec![
        (STAKE_POOL, program),
        (pool.reserve, acc(&STAKE, &laid_reserve(&pool.withdraw))),
        (pool.mint, acc(&TOKEN, &laid_mint(&pool.withdraw))),
        (pool.vault, acc(&TOKEN, &laid_vault(&pool.mint))),
        (pool.pool, acc(&STAKE_POOL, &laid_pool(&pool))),
    ];
    let (born, lines) = a_sitting(&an_rpc(standing), &d.0).need(&bins).unwrap();
    assert_eq!((line(&lines, "the pool, made ahead"), line(&lines, "the pool's birth"), line(&lines, NOT_RUN)), (None, None, None), "{lines:?}");
    assert_eq!(none - born, r(200) + r(82) + r(165) + 5_000, "the three made ahead and their one transaction's fee");
}
