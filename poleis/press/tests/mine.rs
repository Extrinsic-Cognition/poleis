//! THE MINE IS LIVE, proven on a local chain (Nov 23 2026, lane/the-mine-is-live; his Nov 21 2026 17:34:37 UTC "it is supposed to be uncapped
//! supply, prompt-2-mine; the issuance needs to be live and continuous based on the space-router utilization"). A validator of this machine
//! holding the $WARM program (MINE_VALIDATOR, which poleis/ceremony/the-mine-is-live.sh sets), stand-in keys made from the run's own seed
//! (MINE_SEED) and never written, and the door's own code under node (embertide-app/ui/intro/dyson-society/worker/the-mine-is-live.proof.mjs:
//! the Worker's handler, the door's WASM, a scratch D1). In order:
//! - a stand-in dollar, a treasury of it (a stand-in of root 0's vault: no Poleis stands on this chain; that the launch's treasury is that
//!   vault is the press's own check, `chain::treasury_of`), four flasks holding USDC and no SOL, the mine opened six calendar months less a few
//!   minutes ago so its halving falls inside the run, the miner named by the mine's authority (a stand-in of the polis's hand);
//! - THE DEPOSITS, through the door: each flask signs the door's message, the house's fee key pays, the door counts it at finalized; the
//!   chain's record of each flask says what the door's ledger says, and the treasury holds every dollar;
//! - THE SPENDS, through the door, and the crank each burn starts: before the halving with the miner's key on the door, then with it off (the
//!   crank stopped), across the halving still off, then on again: the crank resumes from the chain's cursor and posts what waited, each era's
//!   dollars at their own instants;
//! - THE BOOK: each flask's dollars and $WARM on the chain against the router's ledger, to the micro-dollar and the nano; the flask whose
//!   calls cost nano-dollars, floored an entry; the flask poured past its deposits, stopped at them;
//! - the refusals: a spend past a flask's deposits, a seq the mine holds, a stranger's batch; a burn past its deposits read past and nothing
//!   posted; the crank run twice more, nothing sent and the mine's bytes the same.
//! Without MINE_VALIDATOR the live half says so and passes. The pure half (the law's own tests) is poleis/seat-wasm/tests/mine.rs.

use poleis_press::loader;
use poleis_press::rpc::{self, Cluster, Rpc};
use poleis_press::wire::{self, Key};
use poleis_seat::mine;
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use warm_mine::state::{Flask, Mine};
use warm_mine::error::WarmError;
use warm_mine::law;

const SOL: u64 = 1_000_000_000;
const USD: u64 = 1_000_000;

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
fn u64_at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
/// The router's dollars (a float of nine decimals at most) in nano-dollars; its $WARM in nanos.
fn nanos(v: &Value) -> u64 {
    (v.as_f64().unwrap_or(0.0) * 1e9).round() as u64
}

/// One transaction landed: signed by its keys, sent with its preflight, confirmed. Its signature, or the chain's words.
fn land(rpc: &Rpc, ixs: &[Instruction], payer: &Key, keys: &[&Key]) -> Result<String, String> {
    let blockhash = rpc.blockhash()?;
    let (w, _) = wire::signed(ixs, &payer.address, &blockhash, keys)?;
    let sig = rpc.send(&w, true)?;
    rpc::wait(Duration::from_secs(60), || match rpc.statuses(&[sig.clone()])?.pop().flatten() {
        Some((_, err, _)) if !err.is_null() => Err(format!("{sig}: {err}")),
        Some((_, _, s)) if s == "confirmed" || s == "finalized" => Ok(Some(())),
        _ => Ok(None),
    })?;
    Ok(sig)
}
/// A transaction the chain must refuse, by simulation with its signatures verified: the program's own refusal number.
fn refused(rpc: &Rpc, ixs: &[Instruction], payer: &Key, keys: &[&Key]) -> Option<u64> {
    let (w, _) = wire::signed(ixs, &payer.address, &rpc.blockhash().unwrap(), keys).unwrap();
    let (err, _, _) = rpc.simulate(&w).unwrap();
    err["InstructionError"][1]["Custom"].as_u64()
}
fn data_of(rpc: &Rpc, key: &Pubkey) -> Vec<u8> {
    rpc.account(&key.to_string()).unwrap().map(|a| a.data).unwrap_or_default()
}
/// A token account's amount; a mint's supply.
fn held(rpc: &Rpc, account: &Pubkey) -> u64 {
    let d = data_of(rpc, account);
    if d.len() < 72 { 0 } else { u64_at(&d, warm_mine::token::ACCOUNT_OFF_AMOUNT) }
}
fn clock(rpc: &Rpc) -> u64 {
    mine::clock_ms(&data_of(rpc, &mine::CLOCK)).expect("the chain's clock")
}
/// A flask's record on the chain: (deposited, spent, mined), zeros while none stands.
fn record(rpc: &Rpc, flask: &Pubkey) -> (u64, u64, u64) {
    let d = data_of(rpc, &mine::record_of(flask));
    if d.len() != Flask::SIZE { (0, 0, 0) } else { (u64_at(&d, Flask::OFF_DEPOSITED), u64_at(&d, Flask::OFF_SPENT), u64_at(&d, Flask::OFF_MINED)) }
}

// ---- the token program's instructions the stand-in world needs ----
fn token() -> Pubkey {
    mine::TOKEN
}
fn initialize_mint2(mint: &Pubkey, decimals: u8, authority: &Pubkey) -> Instruction {
    let mut d = vec![warm_mine::token::IX_INITIALIZE_MINT2, decimals];
    d.extend_from_slice(authority.as_ref());
    d.push(0);
    Instruction::new_with_bytes(token(), &d, vec![AccountMeta::new(*mint, false)])
}
fn initialize_account3(account: &Pubkey, mint: &Pubkey, owner: &Pubkey) -> Instruction {
    let mut d = vec![warm_mine::token::IX_INITIALIZE_ACCOUNT3];
    d.extend_from_slice(owner.as_ref());
    Instruction::new_with_bytes(token(), &d, vec![AccountMeta::new(*account, false), AccountMeta::new_readonly(*mint, false)])
}
fn mint_to(mint: &Pubkey, account: &Pubkey, authority: &Pubkey, amount: u64) -> Instruction {
    let d = [&[warm_mine::token::IX_MINT_TO][..], &amount.to_le_bytes()].concat();
    Instruction::new_with_bytes(token(), &d, vec![AccountMeta::new(*mint, false), AccountMeta::new(*account, false), AccountMeta::new_readonly(*authority, true)])
}
fn ata_of(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref(), token().as_ref(), mint.as_ref()], &mine::ATA).0
}
fn ata_made(payer: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(ata_of(owner, mint), false),
        AccountMeta::new_readonly(*owner, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(loader::SYSTEM, false),
        AccountMeta::new_readonly(token(), false),
    ];
    Instruction::new_with_bytes(mine::ATA, &[1], accounts)
}

// ---- the mine's own instructions the door does not send ----
/// `init` (ix.rs): [upgrade authority (s), payer (s,w), program data, mine (w), $WARM mint (w), vault (w), treasury, dollar, token program, system].
fn init(upgrader: &Pubkey, authority: &Pubkey, opening_ms: u64, treasury: &Pubkey, dollar: &Pubkey) -> Instruction {
    let d = [&[warm_mine::IX_INIT][..], authority.as_ref(), &opening_ms.to_le_bytes()].concat();
    let accounts = vec![
        AccountMeta::new_readonly(*upgrader, true),
        AccountMeta::new(*upgrader, true),
        AccountMeta::new_readonly(loader::programdata_of(&mine::WARM), false),
        AccountMeta::new(mine::mine_at(), false),
        AccountMeta::new(mine::mint_at(), false),
        AccountMeta::new(mine::vault_at(), false),
        AccountMeta::new_readonly(*treasury, false),
        AccountMeta::new_readonly(*dollar, false),
        AccountMeta::new_readonly(token(), false),
        AccountMeta::new_readonly(loader::SYSTEM, false),
    ];
    Instruction::new_with_bytes(mine::WARM, &d, accounts)
}
fn set_miner(authority: &Pubkey, miner: &Pubkey) -> Instruction {
    let d = [&[warm_mine::IX_SET_MINER][..], miner.as_ref()].concat();
    Instruction::new_with_bytes(mine::WARM, &d, vec![AccountMeta::new_readonly(*authority, true), AccountMeta::new(mine::mine_at(), false)])
}

/// The opening whose k-th halving is `at`: `at` moved back 6k calendar months, for the first k (1 to 3) whose month holds its day.
fn opening_for(at: u64) -> (u64, u64) {
    let (days, of_day) = (at / law::DAY_MS, at % law::DAY_MS);
    let (y, m, d) = law::civil_of(days);
    for k in 1..=3u64 {
        let months = y * 12 + (m - 1) - 6 * k;
        let (yo, mo) = (months / 12, months % 12 + 1);
        if d <= law::month_days(yo, mo) {
            let opening = law::days_of(yo, mo, d).unwrap() * law::DAY_MS + of_day;
            assert_eq!(law::halving_at(opening, k), Some(at));
            return (opening, k);
        }
    }
    panic!("no opening within eighteen months whose halving is this instant");
}

/// The take-backs the door sent: their signatures, read back from the bytes (one signer the miner, its one instruction the mine's `take_back`).
fn ledger_takes(ledger: &Value, miner: &Pubkey) -> Vec<String> {
    let mut out = Vec::new();
    for s in ledger["sends"].as_array().unwrap() {
        let (Some(sig), Ok(w)) = (s["signature"].as_str(), wire::unb64(s["wire"].as_str().unwrap_or(""))) else { continue };
        let message = &w[1 + 64 * w[0] as usize..];
        let keys = message[3] as usize;
        let at = 4 + 32 * keys + 32;
        if w[0] == 1 && message[4..36] == miner.to_bytes() && message[at] == 1 {
            let accounts = message[at + 2] as usize;
            let d = at + 3 + accounts;
            if message[d + 1] == warm_mine::IX_TAKE_BACK {
                out.push(sig.to_string());
            }
        }
    }
    out
}

/// The door under node: one JSON ask a line, one answer a line.
struct Door {
    child: Child,
    to: ChildStdin,
    from: BufReader<ChildStdout>,
}
impl Door {
    fn start(proof: &str, input: &Value) -> Self {
        let mut child = Command::new("node").arg(proof).arg(input.to_string()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap_or_else(|e| panic!("node {proof}: {e}"));
        let (to, from) = (child.stdin.take().unwrap(), BufReader::new(child.stdout.take().unwrap()));
        Self { child, to, from }
    }
    fn ask(&mut self, v: Value) -> Value {
        writeln!(self.to, "{v}").unwrap();
        let mut line = String::new();
        self.from.read_line(&mut line).unwrap();
        let said: Value = serde_json::from_str(&line).unwrap_or_else(|e| panic!("the door's answer to {v} is not JSON ({e}): {line:?}"));
        assert!(said["error"].is_null(), "the door, asked {v}: {said}");
        said
    }
    /// One chat call that costs the router `cost` dollars: its burn's dollars and $WARM as the door's headers say them.
    fn call(&mut self, key: &str, cost: f64) -> (f64, f64) {
        let r = self.ask(json!({ "do": "call", "key": key, "usd": cost }));
        assert_eq!(r["status"], 200, "a call: {r}");
        (r["usd"].as_str().unwrap().parse().unwrap(), r["warm"].as_str().unwrap().parse().unwrap())
    }
}
impl Drop for Door {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn the_upgrader_of_a_seed() {
    // the script asks the stand-in deployer's address before the validator starts: the $WARM program's upgrade authority at genesis
    let Ok(seed) = std::env::var("MINE_SEED") else { return };
    println!("upgrader {}", wire::standin(&wire::unhex(&seed).unwrap(), "the mine's deployer").address);
}

#[test]
fn the_mine_is_live_on_a_local_chain() {
    let (Ok(url), Ok(seed)) = (std::env::var("MINE_VALIDATOR"), std::env::var("MINE_SEED")) else {
        eprintln!("MINE_VALIDATOR is not set: the live half runs under poleis/ceremony/the-mine-is-live.sh");
        return;
    };
    let began = std::time::Instant::now();
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let genesis = rpc.check_cluster().unwrap();
    let tree = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let worker = tree.join("embertide-app/ui/intro/dyson-society/worker");
    let wasm = std::fs::read(worker.join("poleis-seat.wasm")).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };
    let units = |sig: &str| rpc.transaction(sig).unwrap().and_then(|t| t.2);

    // the stand-ins, from the run's seed
    let seed: [u8; 32] = wire::unhex(&seed).unwrap();
    let key = |name: &str| wire::standin(&seed, name);
    let (deployer, authority, miner, fee) = (key("the mine's deployer"), key("the polis's hand"), key("the door's miner"), key("the door's fee key"));
    let (dollar, dollar_hand, treasury, polis, stranger) = (key("the dollar"), key("the dollar's mint authority"), key("the treasury's account"), key("the polis"), key("a stranger"));
    let flasks: Vec<Key> = ["a", "b", "c", "d"].iter().map(|n| key(&format!("flask {n}"))).collect();
    let deposits = [20 * USD, 10 * USD, 10 * USD, 10 * USD];
    for k in [&deployer, &authority, &miner, &fee, &stranger] {
        rpc.airdrop(&k.address.to_string(), 10 * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || Ok([&deployer, &authority, &miner, &fee, &stranger].iter().all(|k| rpc.balance(&k.address.to_string()).unwrap_or(0) >= 10 * SOL).then_some(()))).unwrap();
    let program = rpc.account(&loader::programdata_of(&mine::WARM).to_string()).unwrap().expect("the $WARM program's data on this validator");
    say("the chain", json!({ "validator": url, "genesis": genesis, "the $WARM program": mine::WARM.to_string(), "its bytes' sha256": wire::hex(&wire::sha256(program.data[45..].iter().rev().skip_while(|b| **b == 0).collect::<Vec<_>>().iter().rev().map(|b| **b).collect::<Vec<u8>>().as_slice())),
        "the door's WASM": { "bytes": wasm.len(), "sha256": wire::hex(&wire::sha256(&wasm)) } }));

    // the dollar, the treasury (a stand-in of root 0's vault of it), the flasks' USDC: no flask holds SOL
    let token_rent = |len: usize| rpc.rent(len).unwrap();
    land(&rpc, &[loader::create_account(&deployer.address, &dollar.address, token_rent(warm_mine::token::MINT_LEN), warm_mine::token::MINT_LEN as u64, &token()), initialize_mint2(&dollar.address, 6, &dollar_hand.address)], &deployer, &[&deployer, &dollar]).unwrap();
    land(&rpc, &[loader::create_account(&deployer.address, &treasury.address, token_rent(warm_mine::token::ACCOUNT_LEN), warm_mine::token::ACCOUNT_LEN as u64, &token()), initialize_account3(&treasury.address, &dollar.address, &polis.address)], &deployer, &[&deployer, &treasury]).unwrap();
    for (f, usd) in flasks.iter().zip(deposits) {
        land(&rpc, &[ata_made(&deployer.address, &f.address, &dollar.address), mint_to(&dollar.address, &ata_of(&f.address, &dollar.address), &dollar_hand.address, usd)], &deployer, &[&deployer, &dollar_hand]).unwrap();
        assert_eq!(rpc.balance(&f.address.to_string()).unwrap(), 0, "a flask holds USDC and no SOL");
    }

    // the mine opened by the program's upgrade authority, its halving HALVING_AFTER seconds on; a stranger's init and miner refused first
    let after: u64 = std::env::var("MINE_HALVING_AFTER").ok().and_then(|s| s.parse().ok()).unwrap_or(150);
    let halving = (now_ms() / 1_000 + after) * 1_000;
    let (opening, k) = opening_for(halving);
    assert_eq!(refused(&rpc, &[init(&stranger.address, &authority.address, opening, &treasury.address, &dollar.address)], &stranger, &[&stranger]), Some(WarmError::Unauthorized as u64), "a stranger opens nothing");
    let opened = land(&rpc, &[init(&deployer.address, &authority.address, opening, &treasury.address, &dollar.address)], &deployer, &[&deployer]).unwrap();
    assert_eq!(refused(&rpc, &[set_miner(&stranger.address, &miner.address)], &stranger, &[&stranger]), Some(WarmError::Unauthorized as u64), "a stranger names no miner");
    let named = land(&rpc, &[set_miner(&authority.address, &miner.address)], &authority, &[&authority]).unwrap();
    let m = mine::mine_state(&data_of(&rpc, &mine::mine_at())).expect("the mine");
    assert_eq!((m.miner, m.treasury, m.dollar, m.opening_ms, m.last_seq), (miner.address, treasury.address, dollar.address, opening, 0));
    assert_eq!(held(&rpc, &mine::vault_at()), 0, "nothing premined");
    say("the mine opened", json!({ "mine": mine::mine_at().to_string(), "mint": mine::mint_at().to_string(), "treasury": treasury.address.to_string(), "opening_ms": opening, "halving_ms": halving, "the halving": k,
        "init": { "signature": opened, "units": units(&opened) }, "set_miner": { "signature": named, "units": units(&named) } }));

    // the door, under node: its node this validator, its mine this one, its opening the mine's
    let iso = |ms: u64| {
        let (y, mo, d) = law::civil_of(ms / law::DAY_MS);
        let s = ms % law::DAY_MS / 1_000;
        format!("{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z", s / 3_600, s / 60 % 60, s % 60, ms % 1_000)
    };
    let door_env = json!({ "SOLANA_RPC": url, "SOLANA_CLUSTER": "localnet", "SOLANA_GENESIS": genesis, "SPACE_ROUTER_USDC_MINT": dollar.address.to_string(), "SPACE_ROUTER_TREASURY": treasury.address.to_string(),
        "SPACE_ROUTER_MINE": mine::mine_at().to_string(), "SPACE_ROUTER_OPENED_AT": iso(opening), "SPACE_ROUTER_FEE_KEY": fee.secret_base58(), "SPACE_ROUTER_MINER_KEY": miner.secret_base58() });
    let proof = worker.join("the-mine-is-live.proof.mjs");
    let start = |env: &Value| Door::start(proof.to_str().unwrap(), &json!({ "wasm": worker.join("poleis-seat.wasm"), "env": env }));
    let mut door = start(&door_env);
    let (fee_before, miner_before) = (rpc.balance(&fee.address.to_string()).unwrap(), rpc.balance(&miner.address.to_string()).unwrap());

    // a validator seconds old has finalized nothing yet, and the door's watch hands a finalized blockhash: wait for the chain a living cluster is
    rpc::wait(Duration::from_secs(120), || Ok((rpc.call("getSlot", json!([{ "commitment": "finalized" }]))?.as_u64().unwrap_or(0) >= 2).then_some(()))).unwrap();
    // THE DEPOSITS: each flask's whole USDC through the door's own road, at once
    let asked = door.ask(json!({ "do": "deposits", "flasks": flasks.iter().map(Key::secret_base58).collect::<Vec<_>>() }));
    let mut keys = Vec::new();
    let mut tanks = Vec::new();
    let mut deposit_units = Vec::new();
    for ((f, usd), d) in flasks.iter().zip(deposits).zip(asked.as_array().unwrap()) {
        assert!(d["message"].is_string() && d["relay"]["status"] == 200 && d["counted"]["status"] == 200 && d["grant"]["status"] == 201, "flask {}'s deposit through the door: {d}", f.address);
        assert_eq!((d["counted"]["body"]["payer"].as_str(), nanos(&d["counted"]["body"]["usd"])), (Some(f.address.to_string().as_str()), usd * 1_000), "the door counted the flask's own dollars");
        assert_eq!(record(&rpc, &f.address), (usd, 0, 0), "THE DEPOSIT IS COUNTED: the flask's record on the chain says what the door's ledger says");
        assert_eq!(held(&rpc, &ata_of(&f.address, &dollar.address)), 0);
        deposit_units.push(units(d["relay"]["body"]["signature"].as_str().unwrap()));
        keys.push(d["key"].as_str().unwrap().to_string());
        tanks.push(d["counted"]["body"]["tank"].as_str().unwrap().to_string());
    }
    let all: u64 = deposits.iter().sum();
    assert_eq!((held(&rpc, &treasury.address), u64_at(&data_of(&rpc, &mine::mine_at()), Mine::OFF_DEPOSITED)), (all, all), "the treasury holds every deposit, and the mine counts it");
    let fee_paid = fee_before - rpc.balance(&fee.address.to_string()).unwrap();
    say("the deposits, through the door", json!({ "flasks": flasks.len(), "micro-dollars": all, "units": deposit_units, "the fee key paid (lamports)": fee_paid, "a flask (lamports)": fee_paid / flasks.len() as u64,
        "a flask under 10 USDC": "the door's watch hands it no message to sign (a flask is made from 10 USDC); the program's own refusal is UnderMinimum" }));

    // what the chain must say of a flask: its dollars and its $WARM
    let book = |rpc: &Rpc| -> Vec<(u64, u64, u64)> { flasks.iter().map(|f| record(rpc, &f.address)).collect() };
    // the crank, asked until the mine's cursor stands still twice (the door's own cranks may still be waiting for the chain's clock)
    let flush = |door: &mut Door, rpc: &Rpc| {
        door.ask(json!({ "do": "settle" }));
        let mut still = 0;
        for _ in 0..40 {
            let before = data_of(rpc, &mine::mine_at());
            let sent = door.ask(json!({ "do": "crank" }))["sent"].as_u64().unwrap();
            still = if sent == 0 && data_of(rpc, &mine::mine_at()) == before { still + 1 } else { 0 };
            if still == 2 {
                return;
            }
        }
        panic!("the crank never came to rest");
    };
    // the router's own books, a tank: its fuel dollars that mined, in nano-dollars, and its $WARM, in nanos
    let ledger_of = |ledger: &Value, tank: &str| -> (u64, u64) {
        let burns: Vec<&Value> = ledger["burns"].as_array().unwrap().iter().filter(|b| b["tank"] == tank && b["warm"].as_f64().unwrap_or(0.0) > 0.0).collect();
        (burns.iter().map(|b| nanos(&b["usd"])).sum(), burns.iter().map(|b| nanos(&b["warm"])).sum())
    };

    // THE SPENDS, the first half, before the halving, the miner's key on the door: every burn starts the crank
    let (a, b, c, d) = (0usize, 1usize, 2usize, 3usize);
    for (who, cost) in [(a, 2.0), (b, 1.0), (c, 0.0123456789), (d, 4.0), (a, 0.1), (c, 0.000987654321), (b, 0.4), (a, 0.02), (c, 0.31415926535)] {
        door.call(&keys[who], cost);
    }
    flush(&mut door, &rpc);
    let ledger = door.ask(json!({ "do": "ledger" }));
    for who in [a, b, d] {
        let (usd, warm) = ledger_of(&ledger, &tanks[who]);
        let (_, spent, mined) = book(&rpc)[who];
        assert_eq!((spent * 1_000, mined), (usd, warm), "before the halving, flask {who}: the chain's dollars and $WARM are the ledger's, to the micro-dollar and the nano");
    }
    let first_cursor = mine::mine_state(&data_of(&rpc, &mine::mine_at())).unwrap().last_seq;
    say("the first half, the crank at every burn", json!({ "burns": ledger["burns"].as_array().unwrap().len(), "the mine's cursor": first_cursor, "the book": book(&rpc), "transactions sent by the door": ledger["sends"].as_array().unwrap().len() }));

    // THE CRANK STOPS: the door loses the miner's key (a Worker without its secret); the burns go on, before the halving and across it
    door.ask(json!({ "do": "miner", "on": false }));
    for (who, cost) in [(a, 0.1), (c, 0.00271828182), (b, 0.2)] {
        door.call(&keys[who], cost);
    }
    assert_eq!(door.ask(json!({ "do": "settle" }))["cranked"], 0, "with no miner's key the door sends nothing");
    let ledger = door.ask(json!({ "do": "ledger" }));
    let last_before = ledger["burns"].as_array().unwrap().iter().map(|b| b["at"].as_u64().unwrap()).max().unwrap();
    assert!(last_before < halving, "the first half's burns ended {} ms after the halving: this Mac was slow, raise MINE_HALVING_AFTER", last_before.saturating_sub(halving));
    while now_ms() < halving + 1_500 {
        std::thread::sleep(Duration::from_millis(500));
    }
    rpc::wait(Duration::from_secs(120), || Ok((clock(&rpc) >= halving).then_some(()))).unwrap();
    for (who, cost) in [(a, 1.0), (b, 2.0), (c, 0.0314159265), (d, 4.0)] {
        door.call(&keys[who], cost);
    }
    // flask d's tank is poured 5 dollars that are no deposit (the tank tool's refill): the ledger lets it spend them, the chain holds 10
    door.ask(json!({ "do": "pour", "tank": tanks[d], "usd": 5 }));
    door.ask(json!({ "do": "settle" }));
    assert_eq!(mine::mine_state(&data_of(&rpc, &mine::mine_at())).unwrap().last_seq, first_cursor, "the crank stopped: the mine's cursor stands where it was");
    let waiting = door.ask(json!({ "do": "ledger" }))["burns"].as_array().unwrap().iter().filter(|b| b["receipt"].as_u64().unwrap_or(0) > first_cursor).count();

    // THE CRANK RESUMES from the chain's cursor: the key is back, the next burns start it, and everything that waited is posted in its order
    door.ask(json!({ "do": "miner", "on": true }));
    for (who, cost) in [(d, 2.0), (d, 1.0), (a, 0.02), (c, 0.00161803398)] {
        door.call(&keys[who], cost);
    }
    // and two more cranks at once, from doors that remember nothing, beside the one the burns started: the chain takes each seq once
    let raced = door.ask(json!({ "do": "race" }))["sent"].clone();
    flush(&mut door, &rpc);
    let ledger = door.ask(json!({ "do": "ledger" }));
    let burns = ledger["burns"].as_array().unwrap().clone();
    say("the crank stopped, and resumed", json!({ "three cranks at once, two of them said they sent": raced, "burns that waited": waiting, "of them before the halving": burns.iter().filter(|b| b["receipt"].as_u64().unwrap_or(0) > first_cursor && b["at"].as_u64().unwrap() < halving).count(),
        "the mine's cursor before": first_cursor, "after": mine::mine_state(&data_of(&rpc, &mine::mine_at())).unwrap().last_seq, "the last receipt": burns.iter().filter_map(|b| b["receipt"].as_u64()).max() }));

    // every batch the door sent, read back from the bytes it sent: [1][signature][message], the message's one instruction the mine's `mine`
    let mut entries: Vec<(Pubkey, u64, u64, u64)> = Vec::new();
    let mut batches = Vec::new();
    for s in ledger["sends"].as_array().unwrap() {
        let w = wire::unb64(s["wire"].as_str().unwrap()).unwrap();
        let (sigs, message) = (w[0] as usize, &w[1 + 64 * w[0] as usize..]);
        // a legacy message: three bytes of header, its keys' count (under 128: one byte), 32 bytes a key, the blockhash, its instructions
        let all: Vec<Pubkey> = (0..message[3] as usize).map(|i| Pubkey::new_from_array(message[4 + 32 * i..36 + 32 * i].try_into().unwrap())).collect();
        if sigs != 1 || all[0] != miner.address {
            continue; // a flask's deposit, the fee key first
        }
        let mut at = 4 + 32 * all.len() + 32;
        assert_eq!(message[at], 1, "a batch is one instruction");
        let accounts = message[at + 2] as usize;
        let idx = &message[at + 3..at + 3 + accounts];
        at += 3 + accounts;
        let (len, used) = if message[at] < 0x80 { (message[at] as usize, 1) } else { ((message[at] & 0x7f) as usize | (message[at + 1] as usize) << 7, 2) };
        let data = &message[at + used..at + used + len];
        assert_eq!((data[0], data.len()), (warm_mine::IX_MINE, 2 + 24 * data[1] as usize));
        let Some(sig) = s["signature"].as_str() else { continue }; // a second crank's same batch, refused by the node before it was sent
        if rpc.statuses(&[sig.to_string()]).unwrap().pop().flatten().is_none_or(|(_, err, _)| !err.is_null()) || batches.iter().any(|b: &Value| b["signature"] == sig) {
            continue;
        }
        for i in 0..data[1] as usize {
            let o = 2 + 24 * i;
            entries.push((all[idx[7 + 3 * i] as usize], u64_at(data, o), u64_at(data, o + 8), u64_at(data, o + 16)));
        }
        batches.push(json!({ "signature": sig, "spends": data[1], "bytes": w.len(), "units": units(sig) }));
    }
    // THE CHAIN'S ORDER: every seq above the one before, no instant before the one before
    assert!(entries.windows(2).all(|w| w[1].3 > w[0].3 && w[1].2 >= w[0].2), "the batches' spends rise by seq and never go back in time: {entries:?}");
    // THE CRANK'S LAW, held to the router's ledger: an entry is its flask's mining burns since its entry before (their receipts' ids up to its
    // seq), their nano-dollars summed and floored to the micro-dollar, all of one era, stamped inside it
    let mut dust = [0u64; 4];
    for who in [a, b, c] {
        let mut from = 0;
        for e in entries.iter().filter(|e| e.0 == flasks[who].address) {
            let mine_burns: Vec<&Value> = burns.iter().filter(|x| x["tank"] == tanks[who].as_str() && x["warm"].as_f64().unwrap_or(0.0) > 0.0 && (from + 1..=e.3).contains(&x["receipt"].as_u64().unwrap_or(0))).collect();
            let sum: u64 = mine_burns.iter().map(|x| nanos(&x["usd"])).sum();
            assert_eq!(e.1, sum / 1_000, "flask {who}'s entry at seq {}: its burns' nano-dollars floored to the micro-dollar", e.3);
            let era = law::era_at(e.2, opening);
            assert!(mine_burns.iter().all(|x| law::era_at(x["at"].as_u64().unwrap(), opening) == era), "flask {who}'s entry at seq {} gathers one era's burns", e.3);
            dust[who] += sum % 1_000;
            from = e.3;
        }
    }
    assert!(entries.iter().any(|e| e.2 < halving) && entries.iter().any(|e| e.2 >= halving), "spends on both sides of the halving");
    let late = entries.iter().filter(|e| e.3 > first_cursor && e.2 < halving).count();
    assert!(late > 0, "spends from before the halving posted after it, stamped before it");

    // THE BOOK
    let state = data_of(&rpc, &mine::mine_at());
    let (minted, from_vault, spent_all) = (u64_at(&state, Mine::OFF_MINTED), u64_at(&state, Mine::OFF_FROM_VAULT), u64_at(&state, Mine::OFF_SPENT));
    let issued_by_law: u128 = entries.iter().map(|e| law::warm_nanos(e.1, e.2, opening)).sum();
    let records = book(&rpc);
    let supply = u64_at(&data_of(&rpc, &mine::mint_at()), warm_mine::token::MINT_OFF_SUPPLY);
    assert_eq!((minted as u128 + from_vault as u128, supply, records.iter().map(|r| r.2).sum::<u64>(), records.iter().map(|r| r.1).sum::<u64>()), (issued_by_law, minted, minted + from_vault, spent_all),
        "THE BOOK: the $WARM issued is the law's over every spend that landed, the supply is what was minted, and the flasks' records sum to the mine's");
    let mut flask_rows = Vec::new();
    for (who, name) in ["a", "b", "c", "d"].iter().enumerate() {
        let (usd, warm) = ledger_of(&ledger, &tanks[who]);
        let (deposited, spent, mined) = records[who];
        assert_eq!(held(&rpc, &mine::warm_account_of(&flasks[who].address)), mined, "flask {name} holds its $WARM in its own account");
        match who {
            // whole micro-dollars: the chain is the ledger, to the micro-dollar and the nano, across the halving and the crank's stop
            0 | 1 => assert_eq!((spent * 1_000, mined), (usd, warm), "flask {name}: the chain's dollars and $WARM are the router's ledger's, to the unit"),
            // nano-dollars: each entry floored to the chain's micro-dollar, never more than the ledger
            2 => assert!(spent * 1_000 + dust[who] == usd && dust[who] < 1_000 * entries.iter().filter(|e| e.0 == flasks[who].address).count() as u64 && mined <= warm, "flask c: the chain holds the ledger's dollars less under a micro-dollar an entry"),
            // THE BOUND: the ledger let it spend its pour, the chain stopped at its deposits
            _ => {
                assert_eq!((deposited, spent, mined), (10 * USD, 10 * USD, 7_100_000_000), "flask d: mined up to its deposits and no further");
                assert_eq!((usd, warm), (11_550_000_000, 7_875_000_000), "flask d's ledger: every dollar it burned, its pour's too");
            }
        }
        flask_rows.push(json!({ "flask": name, "deposited (micro-dollars)": deposited, "the chain: spent (micro-dollars)": spent, "the ledger: spent (nano-dollars)": usd, "the chain: $WARM (nanos)": mined, "the ledger: $WARM (nanos)": warm }));
    }
    let last = mine::mine_state(&state).unwrap();
    say("THE BOOK", json!({ "flasks": flask_rows, "spends posted": entries.len(), "batches": batches, "before the halving": entries.iter().filter(|e| e.2 < halving).count(), "of them posted after it": late,
        "issued (nanos of $WARM)": minted + from_vault, "the law's over every spend": issued_by_law as u64, "supply": supply, "the ledger's owed (nanos)": nanos(&ledger["owed"]),
        "transactions the door sent": ledger["sends"].as_array().unwrap().len(), "of them refused by the node, or the same transaction again": ledger["sends"].as_array().unwrap().len() - flasks.len() - batches.len(),
        "the treasury (micro-dollars)": held(&rpc, &treasury.address), "the mine's cursor": last.last_seq, "the miner paid (lamports)": miner_before - rpc.balance(&miner.address.to_string()).unwrap() }));

    // THE REFUSALS, by the chain: one micro-dollar past a flask's deposits, a seq the mine holds, a stranger's batch, an instant before the last, one after the clock
    let one = |flask: usize, at: u64, seq: u64| [mine::Entry { flask: flasks[flask].address, usd_micros: 1, at_ms: at, seq }];
    let no = |by: &Key, e: &[mine::Entry]| refused(&rpc, &[mine::mine_ix(&by.address, e)], by, &[by]);
    let refusals = [
        ("one micro-dollar past flask d's deposits", no(&miner, &one(d, last.last_at_ms, last.last_seq + 1)), WarmError::PastDeposits),
        ("a seq the mine holds (the last batch again)", no(&miner, &one(a, last.last_at_ms, last.last_seq)), WarmError::SeqNotRising),
        ("a stranger's batch", no(&stranger, &one(a, last.last_at_ms, last.last_seq + 1)), WarmError::Unauthorized),
        ("a spend before the last", no(&miner, &one(a, last.last_at_ms - 1, last.last_seq + 1)), WarmError::Backwards),
        ("a spend after the chain's clock", no(&miner, &one(a, clock(&rpc) + 600_000, last.last_seq + 1)), WarmError::Future),
    ];
    for (what, got, want) in &refusals {
        assert_eq!(*got, Some(*want as u64), "{what}");
    }
    assert_eq!(refused(&rpc, &[mine::mine_ix(&miner.address, &one(a, last.last_at_ms, last.last_seq + 1))], &miner, &[&miner]), None, "and the same batch inside flask a's deposits would land: the refusals are the program's, not the test's");

    // WHAT CANNOT MINE NEVER HOLDS THE LINE: flask d burns once more, past its deposits and alone; nothing is posted (the mine's bytes are the
    // same), and the door notes it has read past that receipt, so it will not stand before the next spend that can mine
    door.call(&keys[d], 1.0);
    flush(&mut door, &rpc);
    let ledger = door.ask(json!({ "do": "ledger" }));
    let last_receipt = ledger["burns"].as_array().unwrap().iter().filter_map(|b| b["receipt"].as_u64()).max().unwrap();
    assert_eq!((data_of(&rpc, &mine::mine_at()), ledger["passed"].as_u64(), last_receipt), (state.clone(), Some(last_receipt), last.last_seq + 1), "a spend past its deposits is read past, and posts nothing");

    // THE CRANK RUN TWICE MORE: nothing sent, the mine's bytes and the supply the same
    let sends_before = ledger["sends"].as_array().unwrap().len();
    let twice = [door.ask(json!({ "do": "crank" }))["sent"].clone(), door.ask(json!({ "do": "crank" }))["sent"].clone()];
    let sends_after = door.ask(json!({ "do": "ledger" }))["sends"].as_array().unwrap().len();
    assert_eq!((twice[0].as_u64(), twice[1].as_u64(), sends_after, data_of(&rpc, &mine::mine_at()), u64_at(&data_of(&rpc, &mine::mint_at()), warm_mine::token::MINT_OFF_SUPPLY)), (Some(0), Some(0), sends_before, state, supply),
        "the crank run twice mints nothing twice");
    say("the refusals, and the crank run twice", json!({ "refused by the chain": refusals.iter().map(|(what, got, _)| json!({ "what": what, "the program's refusal": got })).collect::<Vec<_>>(), "a spend past its deposits, alone": { "posted": 0, "the door read past receipt": last_receipt, "the mine's cursor": last.last_seq },
        "the crank again, twice": twice, "transactions sent": sends_after - sends_before }));

    // SPENT AS CREDITS, IT RETURNS (his press of Nov 23 2026 18:17:41 UTC, "Back to the mine": "One $WARM, the token: spending it as credits returns
    // it, to be mined again"). Where the mine is named a tank's $WARM credit is what its flasks paid back and what they hold for the mine, never what the
    // ledger says it mined; a flask keeps its $WARM, its page signs one approval (the mine the delegate of its $WARM), and after a drink of that credit
    // the crank brings back just what was drunk; the drink mines nothing; the vault pays the next spends first.
    let mine_now = |rpc: &Rpc| {
        let s = data_of(rpc, &mine::mine_at());
        (u64_at(&s, Mine::OFF_MINTED), u64_at(&s, Mine::OFF_FROM_VAULT), u64_at(&s, Mine::OFF_PAID_BACK), u64_at(&s, Mine::OFF_VAULT_BALANCE))
    };
    let supply_now = |rpc: &Rpc| u64_at(&data_of(rpc, &mine::mint_at()), warm_mine::token::MINT_OFF_SUPPLY);
    let paid_of = |rpc: &Rpc, f: &Key| { let r = data_of(rpc, &mine::record_of(&f.address)); if r.len() == Flask::SIZE { u64_at(&r, Flask::OFF_PAID_BACK) } else { 0 } };
    let warm_of = |rpc: &Rpc, f: &Key| held(rpc, &mine::warm_account_of(&f.address));
    let held_of = |rpc: &Rpc, f: &Key| mine::held_for_mine(&data_of(rpc, &mine::warm_account_of(&f.address)), &f.address);
    let tank_of = |door: &mut Door, who: usize| door.ask(json!({ "do": "tank", "key": keys[who] }));
    let credit = |t: &Value| nanos(&t["warm_balance"]);
    let count_of = |v: &Value| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())).unwrap_or(0);
    let until_paid = |rpc: &Rpc, f: &Key, want: u64| rpc::wait(Duration::from_secs(300), || Ok((paid_of(rpc, f) >= want).then_some(()))).unwrap();

    // 1. THE LEDGER'S $WARM IS NO CREDIT: flask d's tank drinks its dollars past zero and is dry, though the ledger says it mined, and its flask holds
    let before_d = tank_of(&mut door, d);
    assert!(nanos(&before_d["warm_mined"]) > 0 && warm_of(&rpc, &flasks[d]) > 0, "flask d mined: on the ledger and in its own $WARM account ({before_d})");
    assert_eq!(credit(&before_d), 0, "where the mine is named its tank's $WARM credit is what it paid back or holds for the mine: nothing yet");
    let drain = (before_d["fuel_usd"].as_f64().unwrap() / 1.05).ceil() + 1.0;
    door.call(&keys[d], drain);
    let dry = door.ask(json!({ "do": "call", "key": keys[d], "usd": 1.0 }));
    let dry_said = dry["body"]["error"]["message"].as_str().unwrap_or("").to_string();
    assert!(dry["status"] == 402 && dry_said.contains("0.000000 $WARM"), "dry: its dollars are gone and the ledger's $WARM pays nothing ({dry})");

    // 2. THE FLASK KEEPS ITS $WARM: flask d's page signs its one approval through the door; flask b pays one $WARM back ahead (the pay-back's road)
    let (minted0, from_vault0, paid0, vault0) = mine_now(&rpc);
    let (supply0, held_d, held_b) = (supply_now(&rpc), warm_of(&rpc, &flasks[d]), warm_of(&rpc, &flasks[b]));
    let some = 1_000_000_000u64;
    let backs = [door.ask(json!({ "do": "back", "flask": flasks[d].secret_base58() })), door.ask(json!({ "do": "back", "flask": flasks[b].secret_base58(), "amount": some }))];
    for (bk, (who, amount)) in backs.iter().zip([(d, held_d), (b, some)]) {
        assert!(bk["relay"]["status"] == 200 && bk["counted"]["status"] == 200, "flask {who} through the door: {bk}");
        assert_eq!((nanos(&bk["counted"]["body"]["counted"]), nanos(&bk["again"]["body"]["counted"]), bk["counted"]["body"]["tank"].as_str()), (amount, 0, Some(tanks[who].as_str())), "the door counts it once, for its own tank");
    }
    assert_eq!(count_of(&backs[0]["held"]), held_d, "the watch saw all of flask d's $WARM, and the approval is for all of it");
    let units_of: Vec<Option<u64>> = backs.iter().map(|bk| units(bk["relay"]["body"]["signature"].as_str().unwrap())).collect();
    assert_eq!((warm_of(&rpc, &flasks[d]), held_of(&rpc, &flasks[d]), paid_of(&rpc, &flasks[d]), warm_of(&rpc, &flasks[b]), paid_of(&rpc, &flasks[b])), (held_d, held_d, 0, held_b - some, some),
        "flask d keeps its $WARM, all of it held for the mine; flask b's one $WARM left for the vault");
    let (minted1, from_vault1, paid1, vault1) = mine_now(&rpc);
    assert_eq!((minted1, from_vault1, paid1, vault1, held(&rpc, &mine::vault_at()), supply_now(&rpc)), (minted0, from_vault0, paid0 + some, vault0 + some, vault0 + some, supply0), "the approval moved nothing; the pay-back moved its one $WARM");
    let (credit_d, credit_b) = (credit(&tank_of(&mut door, d)), credit(&tank_of(&mut door, b)));
    assert_eq!((credit_d, credit_b), (held_d, some), "each tank's credit: what its flask holds for the mine, what it paid back");

    // 3. SPENT AS CREDITS, IT RETURNS: one call of 1.05 dollars drinks flask d's credit and mines nothing; the crank its burn started brings back just the 1.05
    let d_ledger = |door: &mut Door| -> Vec<Value> { door.ask(json!({ "do": "ledger" }))["burns"].as_array().unwrap().iter().filter(|x| x["tank"] == tanks[d].as_str()).cloned().collect() };
    let seen = d_ledger(&mut door).len();
    let first = 1_050_000_000u64;
    let (usd_one, warm_one) = door.call(&keys[d], 1.0);
    door.ask(json!({ "do": "settle" }));
    until_paid(&rpc, &flasks[d], first);
    assert_eq!((usd_one, warm_one, paid_of(&rpc, &flasks[d]), warm_of(&rpc, &flasks[d]), held(&rpc, &mine::vault_at())), (1.05, 0.0, first, held_d - first, vault1 + first),
        "the drink mined nothing; just what it drank came back to the vault; flask d keeps the rest");
    // two calls at once, held at the router until both split their price from the same tank: 4.2 and the 1.85 left, the rest in dollars past zero
    let at_once = door.ask(json!({ "do": "calls", "key": keys[d], "usd": 4.0, "n": 2 }));
    assert!(at_once.as_array().unwrap().iter().all(|c| c["status"] == 200), "both let in: {at_once}");
    door.ask(json!({ "do": "settle" }));
    until_paid(&rpc, &flasks[d], held_d);
    let drinks: Vec<Value> = d_ledger(&mut door).split_off(seen);
    let warm_spent: u64 = drinks.iter().map(|x| nanos(&x["warm_spent"])).sum();
    let after_d = tank_of(&mut door, d);
    assert_eq!((drinks.len(), warm_spent, credit(&after_d), paid_of(&rpc, &flasks[d]), warm_of(&rpc, &flasks[d])), (3, held_d, 0, held_d, 0), "the drinks took exactly what flask d held for the mine, the credit stands at nothing, never below, and all of it came back");
    for x in &drinks {
        let (fuel, per_dollar) = (nanos(&x["usd"]) - nanos(&x["warm_spent"]), law::warm_nanos(USD, x["at"].as_u64().unwrap(), opening) as u64);
        assert!(nanos(&x["warm"]).abs_diff((fuel as u128 * per_dollar as u128 / 1_000_000_000) as u64) <= 1, "a drink's $WARM part mines nothing: {x}");
    }
    let dry_again = door.ask(json!({ "do": "call", "key": keys[d], "usd": 1.0 }));
    assert_eq!(dry_again["status"], 402, "its credit drunk, the tank is dry again");
    flush(&mut door, &rpc);
    assert_eq!(record(&rpc, &flasks[d].address).1, 10 * USD, "nothing of the credit's drinks is posted as a spend: flask d's record is where its deposits stopped it");
    // the crank run again brings nothing back twice; and the chain refuses a stranger's take-back and one past what a flask holds
    let quiet = [door.ask(json!({ "do": "crank" }))["sent"].clone(), door.ask(json!({ "do": "crank" }))["sent"].clone()];
    assert_eq!((quiet[0].as_u64(), quiet[1].as_u64(), paid_of(&rpc, &flasks[d])), (Some(0), Some(0), held_d), "a take-back sent twice moves nothing twice");
    let take = |by: &Key, target: u64| refused(&rpc, &[mine::take_back_ix(&by.address, &[(flasks[d].address, target)])], by, &[by]);
    assert_eq!((take(&stranger, held_d + 1), take(&miner, held_d + 1)), (Some(WarmError::Unauthorized as u64), Some(1)), "a stranger takes nothing back; nothing past what a flask holds (the token program's own refusal)");
    let take_units = ledger_takes(&door.ask(json!({ "do": "ledger" })), &miner.address).into_iter().filter_map(|s| units(&s)).collect::<Vec<_>>();

    // 4. THE VAULT PAYS FIRST: flask a's three calls and flask b's one mine $WARM (flask b's credit untouched: its dollars go first); the crank posts them,
    // and the mine pays them from the vault before it mints
    let (minted1, from_vault1, _, vault1) = mine_now(&rpc);
    let (supply1, mined_before) = (supply_now(&rpc), [record(&rpc, &flasks[a].address).2, record(&rpc, &flasks[b].address).2]);
    for (who, cost) in [(a, 4.0), (a, 6.0), (a, 5.0), (b, 2.0)] {
        door.call(&keys[who], cost);
    }
    flush(&mut door, &rpc);
    let (minted2, from_vault2, paid2, vault2) = mine_now(&rpc);
    let issued = [record(&rpc, &flasks[a].address).2 - mined_before[0], record(&rpc, &flasks[b].address).2 - mined_before[1]];
    let all_issued = issued[0] + issued[1];
    assert_eq!(all_issued, law::warm_nanos(17_850_000, now_ms(), opening) as u64, "the law's $WARM over the new spends: 17.85 dollars (4.2, 6.3, 5.25 and 2.1, the markup in), after the halving");
    assert_eq!((from_vault2 - from_vault1, minted2 - minted1), (all_issued.min(vault1), all_issued - all_issued.min(vault1)), "the vault paid first, and only the rest was minted");
    assert_eq!((supply_now(&rpc) - supply1, vault2, held(&rpc, &mine::vault_at())), (minted2 - minted1, vault1 - (from_vault2 - from_vault1), vault2), "the supply grew only by what was minted; the vault holds what the mine counts");
    assert_eq!(credit(&tank_of(&mut door, b)), some, "flask b's credit stands: its dollars paid its call");

    // 5. ONE $WARM: every token is in a flask's own account or the vault; the vault holds what came back less what it paid out; the supply is what was
    // minted; and, a tank, its $WARM credit drunk and standing is what its flask paid back and holds for the mine, never more
    let st = data_of(&rpc, &mine::mine_at());
    let in_flasks: u64 = flasks.iter().map(|f| warm_of(&rpc, f)).sum();
    assert_eq!((in_flasks + held(&rpc, &mine::vault_at()), u64_at(&st, Mine::OFF_VAULT_BALANCE), supply_now(&rpc)), (supply_now(&rpc), paid2 - u64_at(&st, Mine::OFF_FROM_VAULT), u64_at(&st, Mine::OFF_MINTED)), "one $WARM");
    let ledger = door.ask(json!({ "do": "ledger" }));
    let mut tank_rows = Vec::new();
    for (who, name) in ["a", "b", "c", "d"].iter().enumerate() {
        let counted: i128 = ledger["backs"].as_array().unwrap().iter().filter(|x| x["tank"] == tanks[who].as_str()).map(|x| (x["warm"].as_f64().unwrap_or(0.0) * 1e9).round() as i128).sum();
        let drunk: u64 = ledger["burns"].as_array().unwrap().iter().filter(|x| x["tank"] == tanks[who].as_str()).map(|x| nanos(&x["warm_spent"])).sum();
        let (standing, paid, holds) = (credit(&tank_of(&mut door, who)), paid_of(&rpc, &flasks[who]), held_of(&rpc, &flasks[who]));
        assert_eq!(((drunk + standing) as i128, counted), ((paid + holds) as i128, (paid + holds) as i128), "flask {name}'s tank: its $WARM credit drunk and standing is what its flask paid back and holds for the mine, no more");
        tank_rows.push(json!({ "flask": name, "paid back on the chain (nanos)": paid, "held for the mine (nanos)": holds, "drunk as credits (nanos)": drunk, "credit standing (nanos)": standing, "its flask holds (nanos)": warm_of(&rpc, &flasks[who]) }));
    }
    say("SPENT AS CREDITS, IT RETURNS", json!({
        "the ledger's $WARM is no credit": { "flask d's tank, the ledger's mined (nanos)": nanos(&before_d["warm_mined"]), "its flask held (nanos)": held_d, "its credit before its approval": 0, "dry, the door said": dry_said },
        "the flask keeps its $WARM": { "flask d's approval, all of it held for the mine (nanos)": held_d, "flask d's flask after it (nanos)": held_d, "flask b's pay-back ahead (nanos)": some, "units (the approval, the pay-back)": units_of, "counted again": [nanos(&backs[0]["again"]["body"]["counted"]), nanos(&backs[1]["again"]["body"]["counted"])], "moved by the approval": 0 },
        "spent as credits, it returns": { "one call (dollars)": usd_one, "mined by it": warm_one, "brought back after it (nanos)": first, "flask d kept (nanos)": held_d - first, "then two calls at once (dollars)": drinks[1..].iter().map(|x| x["usd"].clone()).collect::<Vec<_>>(),
            "of them in $WARM": drinks[1..].iter().map(|x| x["warm_spent"].clone()).collect::<Vec<_>>(), "mined on the ledger": drinks.iter().map(|x| x["warm"].clone()).collect::<Vec<_>>(), "brought back in all (nanos)": held_d, "the take-backs' units": take_units, "the crank again, twice": quiet, "the next call": dry_again["status"] },
        "the vault pays first": { "issued to flasks a and b (nanos)": issued, "from the vault": from_vault2 - from_vault1, "minted": minted2 - minted1, "the vault before": vault1, "after": vault2, "supply before": supply1, "after": supply_now(&rpc) },
        "one $WARM": { "in the flasks (nanos)": in_flasks, "in the vault": held(&rpc, &mine::vault_at()), "supply": supply_now(&rpc), "minted": u64_at(&st, Mine::OFF_MINTED), "paid back": paid2, "re-emitted": u64_at(&st, Mine::OFF_FROM_VAULT) },
        "the tanks": tank_rows }));

    if let Ok(path) = std::env::var("MINE_CERTIFICATE") {
        if !path.is_empty() {
            let cert = json!({ "schema": "embertide.the-mine-is-live/1", "validator": "solana-test-validator, local, stand-in keys of the run's own seed", "commit": std::env::var("MINE_COMMIT").ok(), "started_at": std::env::var("MINE_STARTED_AT").ok(),
                "seconds": began.elapsed().as_secs(), "said": said });
            std::fs::write(&path, serde_json::to_string_pretty(&cert).unwrap()).unwrap();
        }
    }
}
