//! THE PRESS IS READY FOR MAINNET: a sealed upgrade to a longer build extends its program's data first (Nov 24 2026,
//! lane/the-press-is-ready-for-mainnet). The press deploys each program at its own length, and the loader's `Upgrade` (the end of Poleis's
//! CODEX_UPGRADE and CODEX_UPGRADE_PROGRAM alike) refuses a build longer than the program's data holds; its `ExtendProgram` asks no authority,
//! takes at least 10,240 bytes at a time, and re-deploys the program at its own slot, so the upgrade waits for the slot after.
//!
//! The pure half: the bytes to extend by, the sealed payloads read back, the act the crank carries, every transaction one packet. The live half
//! (UPGRADE_VALIDATOR, which poleis/ceremony/the-upgrade-extends-first.sh sets): a validator of this machine holding the $WARM program's build
//! at genesis, upgradeable, its authority a stand-in of the run's seed; a buffer of a longer build (the market's) refused by the loader's own
//! `Upgrade`; an extend of 168 bytes refused; the extend and the upgrade in one transaction refused; then the press's own extend
//! (`upgrade::extend_on`, the one `press crank` sends before a sealed upgrade's execute) signed and paid by a crank that holds no authority, and
//! the upgrade in a slot after it lands, the longer build read back byte for byte. Without UPGRADE_VALIDATOR the live half says so and passes.
use poleis_client::{self as client, Payload, CLASS_CODEX, ROOT_UNIT};
use poleis_press::chain::POSIVERSE;
use poleis_press::loader::{self, BUFFER_HEADER, LOADER_V3, PROGRAMDATA_HEADER};
use poleis_press::rpc::{self, Cluster, Rpc};
use poleis_press::upgrade::{act_of, extend_by, extend_for, extend_on, sealed_of, Sealed, MAX_DATA, MIN_EXTEND};
use poleis_press::wire::{self, Key, PACKET};
use poleis_treasury::{CODEX_UPGRADE, CODEX_UPGRADE_PROGRAM};
use serde_json::{json, Value};
use solana_instruction::Instruction;
use solana_loader_v3_interface::instruction as l3;
use solana_pubkey::Pubkey;
use std::time::Duration;

fn k(b: u8) -> Pubkey {
    Pubkey::new_from_array([b; 32])
}

#[test]
fn the_bytes_to_extend_are_what_the_build_lacks_and_never_under_the_loaders_least() {
    let room = |build: usize| PROGRAMDATA_HEADER + build;
    // a program data that holds the build, to the byte or with room to spare: nothing to extend
    assert_eq!(extend_by(room(56_544), 56_544).unwrap(), None);
    assert_eq!(extend_by(room(98_512), 56_544).unwrap(), None);
    // one byte short: the loader's least, 10,240
    assert_eq!(extend_by(room(56_544), 56_545).unwrap(), Some(MIN_EXTEND));
    // devnet's Poleis of Nov 24 2026: 340,000 bytes of data (its account 340,045), this tree's build 438,744, then 445,744 with the older fund's road
    assert_eq!(extend_by(340_045, 438_744).unwrap(), Some(98_744), "the 98,744 the evening's rehearsal extended devnet's Poleis by");
    assert_eq!(extend_by(340_045, 445_744).unwrap(), Some(105_744), "the 105,744 AN OLDER FUND HAS ITS ROAD names");
    // the launch's own: the press deploys at the build's length, so the market to the size of Poleis's build is the difference
    assert_eq!(extend_by(room(98_512), 437_656).unwrap(), Some(437_656 - 98_512));
    // near the 10 MiB an account may hold: what is left, even under the least (the loader's own exception); past it, refused
    assert_eq!(extend_by(MAX_DATA - 100, MAX_DATA - PROGRAMDATA_HEADER).unwrap(), Some(100));
    assert_eq!(extend_by(MAX_DATA - 20_000, MAX_DATA - PROGRAMDATA_HEADER - 19_990).unwrap(), Some(MIN_EXTEND));
    assert!(extend_by(room(1), MAX_DATA).unwrap_err().contains("past the"));
}

fn programdata(len: usize) -> Vec<u8> {
    let mut d = vec![0u8; len];
    d[..4].copy_from_slice(&3u32.to_le_bytes());
    d[12] = 1;
    d
}
fn buffer(build: usize) -> Vec<u8> {
    let mut d = vec![0u8; BUFFER_HEADER + build];
    d[..4].copy_from_slice(&1u32.to_le_bytes());
    d[4] = 1;
    d
}

#[test]
fn the_extend_is_the_loaders_own_and_the_crank_pays_it() {
    let (program, crank) = (k(7), k(1));
    let (n, ix) = extend_for(&program, &programdata(PROGRAMDATA_HEADER + 56_544), &buffer(98_512), &crank).unwrap().unwrap();
    assert_eq!(n, 98_512 - 56_544);
    let theirs = l3::extend_program(&program, Some(&crank), n);
    assert_eq!((ix.program_id, ix.data.clone(), ix.accounts.clone()), (theirs.program_id, theirs.data, theirs.accounts), "the loader interface's own ExtendProgram, the crank its payer");
    assert!(ix.accounts.iter().filter(|m| m.is_signer).map(|m| m.pubkey).eq([crank]), "the crank alone signs: no authority");
    // the build fits: nothing
    assert_eq!(extend_for(&program, &programdata(PROGRAMDATA_HEADER + 98_512), &buffer(98_512), &crank).unwrap(), None);
    // a buffer that is not a buffer, a program data that is not one: refused
    assert!(extend_for(&program, &buffer(10), &buffer(98_512), &crank).unwrap_err().contains("not a program data"));
    assert!(extend_for(&program, &programdata(1_000), &programdata(98_512), &crank).unwrap_err().contains("not a buffer"));
    // its transaction, alone, one packet: the crank its one signer
    let (m, signers) = wire::message(&[ix], &crank, &[0; 32]);
    assert!(1 + 64 * signers.len() + m.len() <= PACKET && signers == vec![crank]);
}

fn view_of(payload: &Payload, hash: [u8; 32]) -> client::ProposalView {
    client::ProposalView {
        hash,
        class: CLASS_CODEX,
        consent: false,
        sum: false,
        market: false,
        plan: false,
        origin: payload.origin(),
        manifest: [4; 32],
        payload_hash: payload.hash(),
        opened: 1,
        expires: 2,
        executed: false,
        impacted: vec![ROOT_UNIT, POSIVERSE],
        payer: k(9),
        payload: payload.bytes(),
    }
}

#[test]
fn a_sealed_upgrade_is_read_back_from_its_proposal_and_carried_as_the_crank_carries_an_act() {
    let (poleis, crank, buffer, spill, warm) = (client::PROGRAM_ID, k(1), k(2), k(3), k(8));
    let own = Payload::Upgrade { origin: ROOT_UNIT, buffer, spill };
    let named = Payload::UpgradeProgram { origin: ROOT_UNIT, program: warm, buffer, spill };
    assert_eq!(sealed_of(&own.bytes(), &poleis).unwrap(), Sealed { kind: CODEX_UPGRADE, program: poleis, buffer, spill }, "kind 6: Poleis's own code");
    assert_eq!(sealed_of(&named.bytes(), &poleis).unwrap(), Sealed { kind: CODEX_UPGRADE_PROGRAM, program: warm, buffer, spill }, "kind 14: the program it names");
    for (p, s) in [(&own, sealed_of(&own.bytes(), &poleis).unwrap()), (&named, sealed_of(&named.bytes(), &poleis).unwrap())] {
        assert_eq!(s.payload(ROOT_UNIT).bytes(), p.bytes(), "read back as the payload it was proposed as");
    }
    // nothing else is an upgrade: an authority's move, a buffer's close, a cut payload, a launch act
    for not in [
        Payload::UpgradeAuthority { origin: ROOT_UNIT, program: warm, new: None }.bytes(),
        Payload::CloseBuffer { origin: ROOT_UNIT, buffer, recipient: spill }.bytes(),
        own.bytes()[..65].to_vec(),
        [named.bytes(), vec![0]].concat(),
    ] {
        assert!(sealed_of(&not, &poleis).is_err(), "{not:?}");
    }
    // the act the crank carries: the ninth, its execute Poleis's for the proposal, the loader's Upgrade accounts at its end
    let hash = [5u8; 32];
    for p in [&own, &named] {
        let act = act_of(&view_of(p, hash), hash, 9, &poleis, &k(6)).unwrap();
        assert_eq!((act.n, act.hash, act.class, act.convene(), act.before.len()), (9, hash, CLASS_CODEX, false, 0), "a CODEX act: unit 1's vote opened with the proposal");
        let e = act.execute_by(&crank);
        let theirs = client::execute(crank, hash, &[ROOT_UNIT, POSIVERSE], p);
        assert_eq!((e.program_id, e.data.clone(), e.accounts.clone()), (theirs.program_id, theirs.data, theirs.accounts), "the client's own execute of it, the crank its payer");
        let (m, signers) = wire::message(&[e], &crank, &[0; 32]);
        assert!(1 + 64 * signers.len() + m.len() <= PACKET, "its execute one packet");
    }
    // refused: another unit's, a payload the proposal's hash does not cover, another hash at the address, no upgrade at all
    let mut other = view_of(&named, hash);
    other.origin = POSIVERSE;
    assert!(act_of(&other, hash, 9, &poleis, &crank).err().unwrap().contains("only root 0 upgrades the code"));
    let mut forged = view_of(&named, hash);
    forged.payload_hash = [0; 32];
    assert!(act_of(&forged, hash, 9, &poleis, &crank).err().unwrap().contains("does not read back"));
    assert!(act_of(&view_of(&named, hash), [6; 32], 9, &poleis, &crank).err().unwrap().contains("another hash"));
    assert!(act_of(&view_of(&Payload::CloseBuffer { origin: ROOT_UNIT, buffer, recipient: spill }, hash), hash, 9, &poleis, &crank).err().unwrap().contains("no sealed upgrade"));
}

#[test]
fn the_crank_extends_before_the_execute_and_never_in_its_transaction() {
    // in press crank, the extend is sent (and its slot waited past) before the execute's instructions are built and sent
    let src = include_str!("../src/crank.rs");
    let at = |w: &str| src.find(w).unwrap_or_else(|| panic!("the crank holds {w}"));
    let (extend, execute) = (at("self.extend_first(&sealed, k)?"), at("act.execute_ixs(&self.deployer"));
    assert!(extend < execute, "the extend first, then the execute");
    let up = include_str!("../src/upgrade.rs");
    assert!(up.contains("let l = self.send(\"crank\", &what, vec![ix], &[crank])?;") && up.contains("self.rpc.slot()? > l.slot"), "a transaction of its own, the crank its signer and payer, the slot after it waited for");
}

// ---------------------------------------------------------------- the live half

fn land(rpc: &Rpc, ixs: &[Instruction], payer: &Key, keys: &[&Key]) -> Result<(String, u64), String> {
    let (w, _) = wire::signed(ixs, &payer.address, &rpc.blockhash()?, keys)?;
    let sig = rpc.send(&w, true)?;
    let slot = rpc::wait(Duration::from_secs(60), || match rpc.statuses(&[sig.clone()])?.pop().flatten() {
        Some((_, err, _)) if !err.is_null() => Err(format!("{sig}: {err}")),
        Some((slot, _, s)) if s == "confirmed" || s == "finalized" => Ok(Some(slot)),
        _ => Ok(None),
    })?;
    Ok((sig, slot))
}
/// The chain's words for a transaction it would refuse (simulated, signatures verified): its error and the loader's last line.
fn refusal(rpc: &Rpc, ixs: &[Instruction], payer: &Key, keys: &[&Key]) -> Option<String> {
    let (w, _) = wire::signed(ixs, &payer.address, &rpc.blockhash().unwrap(), keys).unwrap();
    let (err, _, logs) = rpc.simulate(&w).unwrap();
    if err.is_null() {
        return None;
    }
    let said: Vec<String> = logs.as_array().cloned().unwrap_or_default().iter().filter_map(|l| l.as_str().map(str::to_string)).filter(|l| !l.contains("invoke [") && !l.contains("consumed")).collect();
    Some(format!("{err} ({})", said.join(" / ")))
}

#[test]
fn the_authority_of_a_seed() {
    // the script asks the stand-in authority's address before the validator starts: the program's upgrade authority at genesis
    let Ok(seed) = std::env::var("UPGRADE_SEED") else { return };
    println!("authority {}", wire::standin(&wire::unhex(&seed).unwrap(), "the upgrade's authority").address);
}

#[test]
fn the_extend_comes_first_on_a_local_chain() {
    let (Ok(url), Ok(seed), Ok(program), Ok(shorter), Ok(longer)) =
        (std::env::var("UPGRADE_VALIDATOR"), std::env::var("UPGRADE_SEED"), std::env::var("UPGRADE_PROGRAM"), std::env::var("UPGRADE_SHORTER"), std::env::var("UPGRADE_LONGER"))
    else {
        eprintln!("UPGRADE_VALIDATOR is not set: the live half runs under poleis/ceremony/the-upgrade-extends-first.sh");
        return;
    };
    let began = std::time::Instant::now();
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    rpc.check_cluster().unwrap();
    let program: Pubkey = program.parse().unwrap();
    let (shorter, longer) = (std::fs::read(&shorter).unwrap(), std::fs::read(&longer).unwrap());
    assert!(longer.len() > shorter.len(), "the longer build is longer");
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };
    let seed: [u8; 32] = wire::unhex(&seed).unwrap();
    let (authority, crank, buf, spill) = (wire::standin(&seed, "the upgrade's authority"), wire::standin(&seed, "a crank"), wire::standin(&seed, "the buffer"), wire::standin(&seed, "the spill"));
    for key in [&authority, &crank] {
        rpc.airdrop(&key.address.to_string(), 10_000_000_000).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || Ok([&authority, &crank].iter().all(|x| rpc.balance(&x.address.to_string()).unwrap_or(0) >= 10_000_000_000).then_some(()))).unwrap();
    let pd = loader::programdata_of(&program);
    let read = |key: &Pubkey| rpc.account(&key.to_string()).unwrap();
    let before = read(&pd).expect("the program's data on this validator");
    assert_eq!(before.data.len(), PROGRAMDATA_HEADER + shorter.len(), "the program loaded at its build's own length, as the press deploys it");
    assert_eq!(&before.data[PROGRAMDATA_HEADER..], &shorter[..]);
    say("the program", json!({ "id": program.to_string(), "its data": before.data.len(), "its build": shorter.len(), "the longer build": longer.len() }));

    // the longer build in a buffer whose authority is the program's
    let space = BUFFER_HEADER + longer.len();
    land(&rpc, &[loader::create_account(&authority.address, &buf.address, rpc.rent(space).unwrap(), space as u64, &LOADER_V3), loader::initialize_buffer(&buf.address, &authority.address)], &authority, &[&authority, &buf]).unwrap();
    let mut writes = 0;
    for (i, chunk) in longer.chunks(900).enumerate() {
        land(&rpc, &[loader::write(&buf.address, &authority.address, (i * 900) as u32, chunk)], &authority, &[&authority]).unwrap();
        writes += 1;
    }
    assert_eq!(&read(&buf.address).unwrap().data[BUFFER_HEADER..], &longer[..], "the buffer holds the longer build");
    say("the buffer", json!({ "address": buf.address.to_string(), "writes": writes }));

    let up = l3::upgrade(&program, &buf.address, &authority.address, &spill.address);
    // 1. the upgrade alone: the loader refuses a build the program's data cannot hold
    let refused = refusal(&rpc, &[up.clone()], &authority, &[&authority]).expect("the upgrade of a longer build, unextended, is refused");
    assert!(refused.contains("not large enough") || refused.contains("AccountDataTooSmall"), "{refused}");
    say("the upgrade, unextended", json!(refused));
    // 2. an extend under the loader's least is refused (SIMD-0431), where the chain holds that feature
    let small = refusal(&rpc, &[loader::extend_program(&program, &crank.address, 168)], &crank, &[&crank]);
    say("an extend of 168 bytes", json!(small.clone().unwrap_or_else(|| "taken: this chain does not hold SIMD-0431".into())));
    // 3. the press's own extend, as press crank reads it off the chain
    let (n, ix) = extend_on(&rpc, &program, &buf.address, &crank.address).unwrap().expect("the longer build needs an extend");
    assert_eq!(Some(n), extend_by(before.data.len(), longer.len()).unwrap());
    // the extend and the upgrade in one transaction: the loader re-deploys at the extend's slot and refuses an upgrade in it
    let together = refusal(&rpc, &[ix.clone(), up.clone()], &crank, &[&crank, &authority]).expect("the extend and the upgrade in one transaction are refused");
    assert!(together.contains("deployed in this block already") || together.contains("InvalidArgument"), "{together}");
    say("the extend and the upgrade in one transaction", json!(together));
    // the extend alone, the crank its one signer and payer: no authority signs it
    let held = rpc.balance(&crank.address.to_string()).unwrap();
    let (sig, slot) = land(&rpc, &[ix], &crank, &[&crank]).unwrap();
    let (fee, units) = rpc::wait(Duration::from_secs(30), || Ok(rpc.transaction(&sig)?.map(|t| (t.1, t.2)))).unwrap();
    let after = read(&pd).unwrap();
    assert_eq!(after.data.len(), before.data.len() + n as usize, "extended by the bytes the press asked");
    let paid = held - rpc.balance(&crank.address.to_string()).unwrap();
    assert_eq!(paid, fee + (after.lamports - before.lamports), "the crank paid the rent of the new length and the fee, nothing else");
    assert_eq!(loader::state(&after.data).unwrap(), loader::State::ProgramData { slot, authority: Some(authority.address) }, "re-deployed at the extend's slot, its authority untouched");
    say("the press's extend", json!({ "bytes": n, "slot": slot, "units": units, "fee": fee, "rent": after.lamports - before.lamports, "the program data": [before.data.len(), after.data.len()] }));
    assert_eq!(extend_on(&rpc, &program, &buf.address, &crank.address).unwrap().map(|x| x.0), None, "asked again: nothing to extend");
    // 4. the upgrade in a slot after it lands
    rpc::wait(Duration::from_secs(30), || Ok((rpc.slot()? > slot).then_some(()))).unwrap();
    let (usig, uslot) = land(&rpc, &[up], &authority, &[&authority]).unwrap();
    let done = read(&pd).unwrap();
    assert_eq!(done.data.len(), PROGRAMDATA_HEADER + longer.len());
    assert_eq!(&done.data[PROGRAMDATA_HEADER..], &longer[..], "the longer build, byte for byte");
    assert_eq!(loader::state(&done.data).unwrap(), loader::State::ProgramData { slot: uslot, authority: Some(authority.address) });
    assert!(read(&buf.address).is_none(), "the buffer spent");
    let uunits = rpc::wait(Duration::from_secs(30), || Ok(rpc.transaction(&usig)?.map(|t| t.2))).unwrap();
    say("the upgrade, a slot after", json!({ "slot": uslot, "units": uunits, "the program data": done.data.len(), "the spill": rpc.balance(&spill.address.to_string()).unwrap() }));
    if let Ok(path) = std::env::var("UPGRADE_CERTIFICATE") {
        let cert = json!({ "schema": "embertide.the-upgrade-extends-first/1", "validator": "solana-test-validator, local, stand-in keys of the run's own seed",
            "commit": std::env::var("UPGRADE_COMMIT").ok(), "started_at": std::env::var("UPGRADE_STARTED_AT").ok(), "seconds": began.elapsed().as_secs(), "said": said });
        std::fs::write(&path, serde_json::to_string_pretty(&cert).unwrap() + "\n").unwrap();
    }
}
