//! ROOT 0 IS THIS SITTING'S, OR THE SITTING STOPS, and THE HANDOFF IS LAST. Poleis lets the first wallet found root 0, so a root that
//! stands is held to the sitting whole (`chain::founding_held`): its authority the deployer, every value of its founding the params'.
//! A stranger's root is put here one fault at a time (another payer, other chairs, the same chairs under another plan, another curve) and
//! each is refused by its name. THE SWARM IS ITS OWN MULTISIG: a root with a key in a chair, unit 1 in another chair, or a crew of other
//! agents is not this sitting's. THE WHOLE TREE DECIDES: its floor is the minted Lyssajudrons by the params' curve (its prices, its seats,
//! its permit issuer, the chain its permits name, its window), and a root born by the sale's genesis, or by a curve of a stranger's, is not
//! this sitting's either. And the code is handed to the seal only once the seal has acted (`chain::handoff_ready`): root 0 a council, the
//! fund constituted, the mine and the board standing.
use poleis_client::nft::MintCurve;
use poleis_press::chain::{crew_chairs, curve_of, founding_held, founding_ixs, handoff_ready, root_chairs, Crew, Founded, STRANGER};
use poleis_press::params::Params;
use poleis_treasury::{MODE_COUNCIL, MODE_GENESIS, MODE_SOLO};
use serde_json::json;
use solana_pubkey::Pubkey;

fn k(w: &str) -> Pubkey {
    poleis_press::params::placeholder_address(w)
}

/// The chain the sitting reads its genesis hash from, as 32 bytes.
const CHAIN: [u8; 32] = [7; 32];

fn params() -> Params {
    let v = json!({
        "cluster": "localnet", "priorityMicroLamports": 0,
        "posiverse": { "name": "THE FIRST POSIVERSE", "desc": "", "chair": 2, "agents": [k("root agent 1").to_string(), k("root agent 2").to_string(), k("root agent 3").to_string()], "charter": "cd".repeat(32), "window": 500, "planCap": 1_000_000_000u64, "planPeriod": 1_000_000 },
        "genesis": { "charter": "ab".repeat(32), "name": "THE DYSON SOCIETY", "desc": "a rehearsal", "defaultMint": k("root mint").to_string(), "planCap": 10_000_000_000u64, "planPeriod": 1_000_000, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1_000_000, "stepLamports": 100_000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": k("root issuer").to_string() },
    });
    Params::from_text(&v.to_string()).unwrap()
}

/// Root 0 as the press's own founding leaves it: sent at slot 1,000 with the params above, its curve's window 1,200 to 1,900.
fn ours(deployer: &Pubkey) -> Founded {
    let chairs = root_chairs(2);
    let posiverse = Some(Crew {
        parent: 0,
        chair: 2,
        root: 0,
        chairs: crew_chairs(&[k("root agent 1"), k("root agent 2"), k("root agent 3")]),
        charter: [0xcd; 32],
        window: 500,
        plan_cap: 1_000_000_000,
        plan_period: 1_000_000,
        name: b"THE FIRST POSIVERSE".to_vec(),
        desc: Vec::new(),
        seats: 0,
    });
    Founded {
        authority: *deployer,
        default_mint: k("root mint"),
        charter: [0xab; 32],
        plan_cap: 10_000_000_000,
        plan_period: 1_000_000,
        window: 1000,
        rung: 1,
        name: b"THE DYSON SOCIETY".to_vec(),
        desc: b"a rehearsal".to_vec(),
        chairs,
        curve: Some(MintCurve {
            bump: 254,
            unit: 0,
            start_lamports: 1_000_000,
            step_lamports: 100_000,
            minted_count: 0,
            capacity: poleis_client::ladder_hexagons(1),
            opens_slot: 1_200,
            closes_slot: 1_900,
            completion_issuer: k("root issuer").to_bytes(),
            genesis_hash: CHAIN,
        }),
        posiverse,
    }
}

#[test]
fn the_sittings_own_founding_is_held() {
    let (p, deployer) = (params(), k("the deployer"));
    founding_held(&ours(&deployer), &p, &deployer, &CHAIN, Some((1_200, 1_900))).unwrap();
    // with no founding on record (a sitting cut before it was saved), the window's length is the params'
    founding_held(&ours(&deployer), &p, &deployer, &CHAIN, None).unwrap();
    // what the seats' mints move since is the polis's own: a floor of six minted is still this sitting's root
    let mut minted = ours(&deployer);
    minted.curve.as_mut().unwrap().minted_count = 6;
    founding_held(&minted, &p, &deployer, &CHAIN, Some((1_200, 1_900))).unwrap();
}

#[test]
fn a_strangers_root_0_is_refused_whatever_it_copies() {
    let (p, deployer) = (params(), k("the deployer"));
    // a fault: what the stranger did, the root it leaves, and the words it is refused by
    fn fault(what: &'static str, put: fn(&mut Founded), says: &'static str) -> (&'static str, fn(&mut Founded), &'static str) {
        (what, put, says)
    }
    fn curve(f: &mut Founded) -> &mut MintCurve {
        f.curve.as_mut().unwrap()
    }
    let faults = [
        fault("another payer founded it", |f| f.authority = k("a stranger"), "its authority is"),
        fault("his Keystone in a chair (the launch press's old founding)", |f| f.chairs[0] = (k("root keystone 1").to_bytes(), poleis_client::HOLDER_KEY), "a key in a chair of root 0"),
        fault("a founder seated in a chair", |f| f.chairs[0] = (k("a founder").to_bytes(), poleis_client::HOLDER_KEY), "a key in a chair of root 0"),
        fault("a stranger in an empty chair", |f| f.chairs[11] = (k("a stranger").to_bytes(), poleis_client::HOLDER_KEY), "a key in a chair of root 0"),
        fault("unit 1 in another chair", |f| f.chairs.swap(2, 0), "its chairs are not his posiverse in chair 2"),
        fault("another unit in unit 1's chair", |f| f.chairs[2] = (poleis_client::unit_holder(7), poleis_client::HOLDER_UNIT), "chair 2 holds unit 7"),
        fault("eleven chairs", |f| f.chairs.truncate(11), "its chairs are not"),
        fault("no unit 1", |f| f.posiverse = None, "unit 1 has no registry"),
        fault("a crew of other agents", |f| f.posiverse.as_mut().unwrap().chairs[1] = (k("a stranger's agent").to_bytes(), poleis_client::HOLDER_KEY), "unit 1's chairs are not its agents' own"),
        fault("his agents in another order", |f| f.posiverse.as_mut().unwrap().chairs.swap(0, 1), "unit 1's chairs are not its agents' own"),
        fault("a fourth agent in unit 1", |f| f.posiverse.as_mut().unwrap().chairs[3] = (k("a fourth").to_bytes(), poleis_client::HOLDER_KEY), "unit 1's chairs are not its agents' own"),
        fault("unit 1 under another parent", |f| f.posiverse.as_mut().unwrap().parent = 5, "unit 1's parent"),
        fault("unit 1 says another chair", |f| f.posiverse.as_mut().unwrap().chair = 0, "unit 1's chair in root 0"),
        fault("unit 1 under another charter", |f| f.posiverse.as_mut().unwrap().charter[0] ^= 1, "unit 1's charter"),
        fault("unit 1 with another window", |f| f.posiverse.as_mut().unwrap().window = 100, "unit 1's window"),
        fault("unit 1 with no cap", |f| f.posiverse.as_mut().unwrap().plan_cap = u64::MAX, "unit 1's plan's cap"),
        fault("unit 1 under another name", |f| f.posiverse.as_mut().unwrap().name = b"ANOTHER".to_vec(), "unit 1's name"),
        fault("unit 1 born with a floor", |f| f.posiverse.as_mut().unwrap().seats = 1, "unit 1's floor"),
        fault("his chairs, another mint", |f| f.default_mint = k("a stranger's mint"), "its default mint"),
        fault("his chairs, another charter", |f| f.charter[0] ^= 1, "its charter"),
        fault("his chairs, a plan with no cap", |f| f.plan_cap = u64::MAX, "its plan's cap"),
        fault("his chairs, another period", |f| f.plan_period += 1, "its plan's period"),
        fault("his chairs, another window", |f| f.window = 100, "its window"),
        fault("his chairs, another rung", |f| f.rung = 2, "its rung"),
        fault("his chairs, another name", |f| f.name = b"THE DYSON S0CIETY".to_vec(), "its name"),
        fault("his chairs, another description", |f| f.desc.clear(), "its description"),
        fault("his chairs, born by the sale's genesis (no curve)", |f| f.curve = None, "it has no curve"),
        fault("a curve of another unit", |f| curve(f).unit = 1, "its curve's unit"),
        fault("seats sold for nothing", |f| curve(f).start_lamports = 1, "its curve's first price"),
        fault("a curve that barely rises", |f| curve(f).step_lamports = 1, "its curve's step"),
        fault("more seats than the rung's", |f| curve(f).capacity = 150, "its curve's seats"),
        fault("a stranger's permits (another issuer)", |f| curve(f).completion_issuer = k("a stranger").to_bytes(), "its curve's permit issuer"),
        fault("permits of another chain", |f| curve(f).genesis_hash = [8; 32], "its curve's chain"),
        fault("a window that opens at once", |f| curve(f).opens_slot = 1_001, "its curve's window"),
        fault("a window open for ever", |f| curve(f).closes_slot = 1_000_000, "its curve's window"),
    ];
    for (what, fault, says) in faults {
        let mut f = ours(&deployer);
        fault(&mut f);
        let e = founding_held(&f, &p, &deployer, &CHAIN, Some((1_200, 1_900))).expect_err(what);
        eprintln!("  {what}: refused, {}", &e[..e.find(": a stranger").unwrap_or(e.len())]);
        assert!(e.contains(says) && e.contains(STRANGER), "{what}: refused, by other words than its own: {e}");
    }
    // with no founding on record, a window of another length is still refused
    let mut f = ours(&deployer);
    f.curve.as_mut().unwrap().closes_slot += 1;
    assert!(founding_held(&f, &p, &deployer, &CHAIN, None).unwrap_err().contains("its curve's window is not the params' length"));
    // and another deployer's sitting, or another chain's, does not take this root for its own
    assert!(founding_held(&ours(&deployer), &p, &k("another sitting's deployer"), &CHAIN, Some((1_200, 1_900))).unwrap_err().contains("its authority is"));
    assert!(founding_held(&ours(&deployer), &p, &deployer, &[8; 32], Some((1_200, 1_900))).unwrap_err().contains("its curve's chain"));
}

/// THE FOUNDING IS ONE TRANSACTION, the NFT genesis with no founder: init_config, Poleis's open_nft_genesis (its curve the params', its
/// window counted from the founding's slot, its permits bound to the chain's genesis hash) and found_child of unit 1, in that order, the
/// deployer its one signer, inside a packet.
#[test]
fn the_founding_is_the_nft_genesis_with_no_founder() {
    let (p, deployer) = (params(), k("the deployer"));
    let c = curve_of(&p, 1_000, &CHAIN).unwrap();
    assert_eq!((c.unit, c.start_lamports, c.step_lamports, c.minted_count, c.capacity), (0, 1_000_000, 100_000, 0, poleis_client::ladder_hexagons(1)));
    assert_eq!((c.opens_slot, c.closes_slot, c.completion_issuer, c.genesis_hash), (1_200, 1_900, k("root issuer").to_bytes(), CHAIN));
    let (ixs, window) = founding_ixs(&p, &deployer, 1_000, &CHAIN).unwrap();
    assert_eq!(window, (1_200, 1_900));
    assert_eq!(ixs.len(), 3, "init_config, the NFT genesis, found_child: no priority fee asked");
    assert!(ixs.iter().all(|ix| ix.program_id == poleis_client::PROGRAM_ID));
    let tags: Vec<u8> = ixs.iter().map(|ix| ix.data[0]).collect();
    assert_eq!(tags[1], poleis_client::nft::IX_OPEN_NFT_GENESIS, "root 0 opened by the NFT genesis, never the sale's");
    let open = &ixs[1].data[1..];
    assert_eq!(open[153], 0, "no founder: nobody is seated in a chair of root 0 by the founding");
    assert_eq!((&open[89..121], &open[121..153]), (&k("root issuer").to_bytes()[..], &CHAIN[..]), "the issuer and the chain the permits name");
    assert_eq!(ixs[1].accounts[4].pubkey, poleis_client::nft::curve_pda(0), "root 0's curve");
    let (m, signers) = poleis_press::wire::message(&ixs, &deployer, &[0; 32]);
    let bytes = 1 + 64 * signers.len() + m.len();
    assert_eq!(signers, vec![deployer], "the deployer alone signs the founding");
    assert!(bytes <= poleis_press::wire::PACKET, "{bytes} bytes");
    eprintln!("  the founding with the NFT genesis: {bytes} bytes of {}", poleis_press::wire::PACKET);
    // a curve the chain cannot hold whole is refused before any byte is signed: a rung that is none
    let mut v: serde_json::Value = serde_json::from_str(&p.raw.to_string()).unwrap();
    v["genesis"]["rung"] = json!(6);
    assert!(curve_of(&Params::from_text(&v.to_string()).unwrap(), 1_000, &CHAIN).unwrap_err().contains("genesis.rung"));
}

#[test]
fn the_handoff_is_last() {
    // the pool born (its own half of the handoff's order: tests/pool.rs), mainnet's rules
    let (born, rules) = (&poleis_press::pool::PoolAt::Born(json!({})), poleis_press::rpc::Cluster::Mainnet);
    // the first sitting's end: root 0 in genesis, frozen until its first seat is claimed and it is ratified
    assert!(handoff_ready(MODE_GENESIS, false, true, true, born, rules).unwrap_err().contains("the handoff waits: root 0 is still in genesis"));
    // even with a fund laid by no vote of this polis, a root in genesis is handed no code
    assert!(handoff_ready(MODE_GENESIS, true, true, true, born, rules).unwrap_err().contains("still in genesis"));
    assert!(handoff_ready(MODE_SOLO, true, true, true, born, rules).is_err());
    // ratified, and the seal has not acted yet
    assert!(handoff_ready(MODE_COUNCIL, false, true, true, born, rules).unwrap_err().contains("the handoff waits for the seal's first act"));
    // the mine's init needs the deployer as the program's upgrade authority: never handed over before it
    assert!(handoff_ready(MODE_COUNCIL, true, false, true, born, rules).unwrap_err().contains("the $WARM mine"));
    assert!(handoff_ready(MODE_COUNCIL, true, true, false, born, rules).unwrap_err().contains("the market's board"));
    handoff_ready(MODE_COUNCIL, true, true, true, born, rules).unwrap();
}

/// THE TREASURY IS THE SWARM'S OWN: the account at root 0's vault is the token program's, of the dollar, and answers to root 0's
/// treasury alone; anything else is not the vault the mine may name.
#[test]
fn a_vault_that_answers_to_another_is_not_root_0s() {
    use poleis_press::chain::{vault_held, TOKEN};
    let dollar = k("root mint");
    let mut d = vec![0u8; warm_mine::token::ACCOUNT_LEN];
    d[..32].copy_from_slice(dollar.as_ref());
    d[32..64].copy_from_slice(poleis_client::treasury_pda(0).as_ref());
    let token = TOKEN.to_string();
    vault_held(&token, &d, &dollar).unwrap();
    // a wallet of his as its authority (the treasury the params named before this lane): refused
    let mut other = d.clone();
    other[32..64].copy_from_slice(k("a wallet of his").as_ref());
    assert!(vault_held(&token, &other, &dollar).unwrap_err().contains("not to root 0's treasury"));
    // another mint, another program, another length
    assert!(vault_held(&token, &d, &k("another mint")).unwrap_err().contains("is not a token account of"));
    assert!(vault_held(&k("another program").to_string(), &d, &dollar).is_err());
    assert!(vault_held(&token, &d[..164], &dollar).is_err());
}

/// The chairs this sitting founds, said here in full and not by the functions that are held to them: root 0's one lit chair is the
/// params', holding unit 1, the eleven others empty; unit 1's chairs are its agents' keys in their order and nothing after them.
#[test]
fn the_chairs_the_sitting_founds_are_said_in_full() {
    use poleis_client::{HOLDER_EMPTY, HOLDER_KEY, HOLDER_UNIT};
    for chair in [0u8, 2, 11] {
        let rc = root_chairs(chair);
        assert_eq!(rc.len(), 12);
        for (i, c) in rc.iter().enumerate() {
            if i == chair as usize {
                assert_eq!(*c, (poleis_client::unit_holder(1), HOLDER_UNIT), "chair {i} holds unit 1");
            } else {
                assert_eq!(*c, ([0u8; 32], HOLDER_EMPTY), "chair {i} is empty: no key sits in a chair of root 0");
            }
        }
    }
    let agents = [k("root agent 1"), k("root agent 2"), k("root agent 3")];
    let cc = crew_chairs(&agents);
    assert_eq!(cc.len(), 12);
    for (i, c) in cc.iter().enumerate() {
        match agents.get(i) {
            Some(a) => assert_eq!(*c, (a.to_bytes(), HOLDER_KEY), "unit 1's chair {i} is its agent's own key"),
            None => assert_eq!(*c, ([0u8; 32], HOLDER_EMPTY), "unit 1's chair {i} is empty"),
        }
    }
}
