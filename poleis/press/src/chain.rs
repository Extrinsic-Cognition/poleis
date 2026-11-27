//! The sitting on a chain: the keys (his, behind his finger, or a rehearsal's stand-ins from a seed held in memory), the one road every
//! transaction takes (simulated with its signatures verified, sent, confirmed, read back), and the steps: the deploy through named
//! buffers with root 0 founded the slot Poleis becomes callable, the $WARM mine opened with its face, the market's board, the handoff of
//! the three upgrade authorities to Poleis's `upgrade` PDA. Every step reads the chain first: what stands is read back and skipped, so a
//! sitting cut short resumes where it stopped, and the record says where it resumed.

use crate::build::{self, Ids, PROGRAMS};
use crate::loader::{self, State, LOADER_V3, SYSTEM};
use crate::params::Params;
use crate::record::{Landed, Record};
use crate::rpc::{Cluster, Rpc};
use crate::wire::{self, Key, PACKET};
use crate::Res;
use poleis_client::{self as client, nft, FoundSpec, HOLDER_EMPTY, HOLDER_KEY, HOLDER_UNIT, ROOT_UNIT};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// SPL Token, the associated token program, Metaplex's token metadata program: Solana's, and Metaplex's, never the house's.
pub const TOKEN: Pubkey = solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const METAPLEX: Pubkey = Pubkey::new_from_array(warm_mine::face::METADATA_PROGRAM_ID.to_bytes());
/// The fee of one signature, lamports (the cluster's base fee).
pub const SIGNATURE_FEE: u64 = 5_000;
/// Metaplex's fee for making a token's metadata, as its program charged it when last read by the house (unread on mainnet in this lane):
/// in the need as an upper bound, never in a transaction.
pub const METAPLEX_FEE: u64 = 10_000_000;
/// What the need keeps beyond what it counts.
pub const MARGIN: u64 = 20_000_000;
/// The market's Board (poleis_roads `state::Board::SIZE`; held to its source by a test).
pub const BOARD_SIZE: usize = 207;
/// Metaplex's Metadata account at its largest (mpl-token-metadata's MAX_METADATA_LEN): the need's bound for the face's rent.
pub const METADATA_MAX: usize = 679;
/// The writes sent before their confirmations are read.
pub const BATCH: usize = 64;
/// The passes over a buffer, one after the other, that may land no write before the deploy stops.
pub const IDLE_PASSES: u32 = 3;
/// The four keys of a sitting, by the names of their files.
pub const KEY_NAMES: [&str; 4] = ["deployer", "poleis", "market", "warm"];
/// His posiverse: unit 1 of the tree, root 0's first pentagon (his Nov 19 2026 "my posiverse is the first pentagon added there"), in the
/// chair the params name (`posiverse.chair`). Its chairs are its agents' own keys; the eleven other chairs of root 0 stay empty (twelve
/// is a budget, lit bottom up).
pub const POSIVERSE: u64 = 1;

/// The chair of root 0 his posiverse takes: `posiverse.chair`, 0 to 11, required.
pub fn posiverse_chair(p: &Params) -> Res<u8> {
    let c = p.u64("posiverse.chair")?;
    if c as usize >= poleis_treasury::PENTAGONS {
        return Err(format!("params posiverse.chair: {c}; root 0's chairs are 0 to {}", poleis_treasury::PENTAGONS - 1));
    }
    Ok(c as u8)
}
/// The words a params file that still names the mine's treasury is refused by.
pub const TREASURY_NAMED: &str = "params warm.treasury: the $WARM mine's treasury is root 0's own vault of the dollar, derived (vault_pda(0, the dollar)), so a flask's deposit lands under the swarm's seal and leaves only by a sealed act; it is never a param (it named a vault of his, a key of his holding the swarm's money): take the line out";

/// THE SWARM JUDGES (his press of Nov 23 2026 19:32:23 UTC, "The swarm, by its own vote", to "Mainnet: who reports what a passed market
/// road was worth, so its bets pay out?"): the words a params file that still names the board's judge is refused by.
pub const JUDGE_NAMED: &str = "params market.judge: the market's judge is the swarm, by its own vote (his press of Nov 23 2026 19:32:23 UTC): root 0's own hand, derived (agent_pda(0)), so a road's receipts are said only by a sealed act of the whole tree; it is never a param (no key decides): take the line out";

/// Where a sitting's keys come from.
pub enum KeySource {
    /// His: four files in a folder only his user reads, each read only after his finger said yes (mainnet).
    Files { dir: PathBuf, touchid: PathBuf },
    /// A rehearsal's: stand-ins made from a seed held in memory, never on a disk.
    Seed([u8; 32]),
}

impl KeySource {
    /// The named keys. His files: his finger first, then the folder's and each file's mode, then the keys.
    pub fn load(&self, names: &[&str], step: &str) -> Res<Vec<Key>> {
        match self {
            KeySource::Seed(seed) => Ok(names.iter().map(|n| wire::standin(seed, n)).collect()),
            KeySource::Files { dir, touchid } => {
                finger(touchid, &format!("THE LAUNCH PRESS, {step}: read the {} key{}", names.join(", "), if names.len() > 1 { "s" } else { "" }))?;
                check_private(dir)?;
                names
                    .iter()
                    .map(|n| {
                        let p = dir.join(format!("{n}.json"));
                        check_private(&p)?;
                        Key::from_cli_text(&std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?).map_err(|e| format!("{}: {e}", p.display()))
                    })
                    .collect()
            }
        }
    }
}

/// His finger, by the house's Touch ID asker (tools/embertide-mesh/touchid): it answers "ok" and exits 0 only when he said yes.
pub fn finger(touchid: &Path, reason: &str) -> Res<()> {
    let out = Command::new(touchid).arg(reason).output().map_err(|e| format!("his finger: the asker {}: {e}", touchid.display()))?;
    let said = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() && said == "ok" {
        Ok(())
    } else {
        Err(format!("his finger did not say yes ({said}): nothing read, nothing sent"))
    }
}

/// A key's folder is his user's alone (700), a key file his user's alone (600).
pub fn check_private(p: &Path) -> Res<()> {
    use std::os::unix::fs::MetadataExt as _;
    let m = std::fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mode = m.mode() & 0o777;
    let want = if m.is_dir() { 0o700 } else { 0o600 };
    if mode != want {
        return Err(format!("{} is mode {mode:o}, not {want:o}: a key's place is read by his user alone", p.display()));
    }
    if m.uid() != own_uid() {
        return Err(format!("{} is not his user's", p.display()));
    }
    Ok(())
}
/// The process's own user, read off its home folder's owner (no libc call).
fn own_uid() -> u32 {
    std::env::var_os("HOME").and_then(|h| std::fs::metadata(h).ok()).map(|m| std::os::unix::fs::MetadataExt::uid(&m)).unwrap_or(u32::MAX)
}

/// Every address of the sitting, derived from its ids by the programs' own seeds.
#[derive(Clone, Debug)]
pub struct Addresses {
    pub config: Pubkey,
    pub registry: Pubkey,
    pub treasury: Pubkey,
    /// root 0's curve: the floor's mint terms (its NFT genesis)
    pub curve: Pubkey,
    pub agent: Pubkey,
    pub upgrade: Pubkey,
    pub fund: Pubkey,
    pub market_law: Pubkey,
    pub mine: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub metadata: Pubkey,
    pub board: Pubkey,
}

pub fn addresses(ids: &Ids, board_id: u64) -> Addresses {
    let warm = |seed: &[u8]| Pubkey::find_program_address(&[seed], &ids.warm).0;
    let mint = warm(warm_mine::state::SEED_MINT);
    Addresses {
        config: client::config_pda_of(ROOT_UNIT),
        registry: client::registry_pda(ROOT_UNIT),
        treasury: client::treasury_pda(ROOT_UNIT),
        curve: nft::curve_pda(ROOT_UNIT),
        agent: client::agent_pda(ROOT_UNIT),
        upgrade: client::upgrade_pda(),
        fund: client::fund_pda(),
        market_law: client::find_pda(&[b"market", &ROOT_UNIT.to_le_bytes()]),
        mine: warm(warm_mine::state::SEED_MINE),
        mint,
        vault: warm(warm_mine::state::SEED_VAULT),
        metadata: Pubkey::find_program_address(&[b"metadata", METAPLEX.as_ref(), mint.as_ref()], &METAPLEX).0,
        board: Pubkey::find_program_address(&[b"board", &board_id.to_le_bytes()], &ids.market).0,
    }
}

pub struct Sitting {
    pub dir: PathBuf,
    pub copy: PathBuf,
    pub rec: Record,
    pub p: Params,
    pub rpc: Rpc,
    pub ids: Ids,
    pub deployer: Pubkey,
    pub keys: KeySource,
    pub at: Addresses,
}

/// A program of the sitting by its name.
pub fn program_id(ids: &Ids, name: &str) -> Pubkey {
    match name {
        "poleis" => ids.poleis,
        "market" => ids.market,
        _ => ids.warm,
    }
}

impl Sitting {
    pub fn board_id(&self) -> Res<u64> {
        self.p.u64("market.board")
    }
    /// The compute budget a transaction asks when the params name a priority fee: its units as simulated, and some.
    fn budget(&self, units: u64) -> Res<Vec<Instruction>> {
        let price = self.p.u64_or("priorityMicroLamports", 0)?;
        if price == 0 {
            return Ok(Vec::new());
        }
        Ok(vec![loader::compute_limit(sized_limit(units) as u32), loader::compute_price(price)])
    }

    /// One transaction: simulated with its signatures verified (a refusal stops the step with the cluster's words), sent, confirmed and
    /// read back; kept in the record.
    pub fn send(&mut self, step: &str, what: &str, ixs: Vec<Instruction>, keys: &[&Key]) -> Res<Landed> {
        let (wire, first) = self.simulated(what, ixs, keys)?;
        self.send_wire(step, what, &wire, &first, keys.len())
    }

    /// A transaction signed and simulated with its signatures verified (a refusal stops the step with the cluster's words), its compute budget
    /// asked when the params name a priority fee.
    pub(crate) fn simulated(&self, what: &str, ixs: Vec<Instruction>, keys: &[&Key]) -> Res<(Vec<u8>, [u8; 64])> {
        let payer = keys[0].address;
        let mut ixs = ixs;
        // a step that names its own cap (the vault, the crank's, the ratify, the pool's birth) keeps it: a second compute budget is refused
        // ("a duplicate instruction"), as the rehearsal of mainnet's priority fee found
        if self.p.u64_or("priorityMicroLamports", 0)? > 0 && !asks_budget(&ixs) {
            let probe = [vec![loader::compute_limit(1_400_000)], ixs.clone()].concat();
            let (wire, _) = wire::signed(&probe, &payer, &self.rpc.blockhash()?, keys)?;
            let (_, units, _) = self.rpc.simulate(&wire)?;
            ixs = [self.budget(units)?, ixs].concat();
        }
        let (wire, first) = wire::signed(&ixs, &payer, &self.rpc.blockhash()?, keys)?;
        if wire.len() > PACKET {
            return Err(format!("{what}: {} bytes, past the {PACKET} a transaction may hold", wire.len()));
        }
        let (err, _, logs) = self.rpc.simulate(&wire)?;
        if !err.is_null() {
            return Err(format!("{what}: refused in simulation: {err}\n      {}", tail(&logs)));
        }
        Ok((wire, first))
    }

    /// A transaction sent only when the cluster takes it now: Ok(Ok(landed)), or Ok(Err(the cluster's words)) when its simulation
    /// refused it and nothing was sent (a seal before its window closes, a vote already counted): the crank's way of asking what is due.
    pub fn try_send(&mut self, step: &str, what: &str, ixs: Vec<Instruction>, keys: &[&Key]) -> Res<Result<Landed, String>> {
        match self.simulated(what, ixs, keys) {
            Ok((wire, first)) => self.send_wire(step, what, &wire, &first, keys.len()).map(Ok),
            Err(e) if e.contains("refused in simulation") => Ok(Err(e)),
            Err(e) => Err(e),
        }
    }

    /// A transaction sent, and its slot read at the processed level, the moment the cluster has it (THE SAME BREATH reads Poleis's deploy
    /// so): confirmed after, by `settle`.
    fn fire(&mut self, what: &str, ixs: Vec<Instruction>, keys: &[&Key]) -> Res<(Vec<u8>, [u8; 64], u64)> {
        let (wire, first) = self.simulated(what, ixs, keys)?;
        let sig = wire::base58(&first);
        self.rpc.send(&wire, false)?;
        let slot = crate::rpc::wait(Duration::from_secs(90), || match self.rpc.statuses(&[sig.clone()])?.into_iter().next().flatten() {
            Some((s, e, _)) if e.is_null() => Ok(Some(s)),
            Some((_, e, _)) => Err(format!("{what}: {sig} landed and failed: {e}")),
            None => Ok(None),
        })?;
        Ok((wire, first, slot))
    }

    /// A transaction already signed, sent without a simulation (the founding, which no simulation can run before Poleis is callable),
    /// confirmed and kept.
    pub fn send_wire(&mut self, step: &str, what: &str, wire: &[u8], first: &[u8; 64], signatures: usize) -> Res<Landed> {
        let sig = wire::base58(first);
        let said = self.rpc.send(wire, false)?;
        if said != sig {
            return Err(format!("{what}: the cluster names another signature ({said})"));
        }
        self.settle(step, what, wire, first, signatures)
    }

    /// A transaction the cluster has: confirmed (sent again, the same bytes, while unseen), its fee and units read, kept in the record.
    fn settle(&mut self, step: &str, what: &str, wire: &[u8], first: &[u8; 64], signatures: usize) -> Res<Landed> {
        let sig = wire::base58(first);
        let slot = self.confirm(&sig, wire, what)?;
        let (fee, units) = self.fee_and_units(&sig)?;
        let l = Landed { step: step.into(), what: what.into(), signature: sig, slot, fee, units, bytes: wire.len(), signatures };
        self.rec.landed(&l)?;
        eprintln!("  {what}: slot {slot}, {} CU, {} bytes, fee {fee}", units.map(|u| u.to_string()).unwrap_or("?".into()), wire.len());
        Ok(l)
    }

    /// Polled until the cluster confirms it, or it landed and failed; sent again, the same bytes, while the cluster has not seen it.
    fn confirm(&self, sig: &str, wire: &[u8], what: &str) -> Res<u64> {
        let (begun, mut sent) = (Instant::now(), Instant::now());
        loop {
            match self.rpc.statuses(&[sig.to_string()])?.into_iter().next().flatten() {
                Some((_, err, _)) if !err.is_null() => return Err(format!("{what}: {sig} landed and failed: {err}")),
                Some((slot, _, level)) if level == "confirmed" || level == "finalized" => return Ok(slot),
                Some(_) => {}
                None if sent.elapsed() > Duration::from_secs(2) => {
                    let _ = self.rpc.send(wire, false);
                    sent = Instant::now();
                }
                None => {}
            }
            if begun.elapsed() > Duration::from_secs(90) {
                return Err(format!("{what}: the cluster never confirmed {sig}"));
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    /// A confirmed transaction's fee and units, for the record. An endpoint that will not give them (a fault, a rate) stops nothing: the
    /// transaction landed, and it is kept in the record without them.
    fn fee_and_units(&self, sig: &str) -> Res<(u64, Option<u64>)> {
        for _ in 0..40 {
            if let Ok(Some((_, fee, units, _))) = self.rpc.transaction(sig) {
                return Ok((fee, units));
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        Ok((0, None))
    }

    /// Each program's built bytes, read from the copy and held to the hash `press build` recorded: the deploy refuses any other.
    pub fn binaries(&self) -> Res<Vec<(&'static str, Vec<u8>)>> {
        let mut out = Vec::new();
        for (name, _, so, dir) in PROGRAMS {
            let path = build::out_dir(&self.copy, dir).join(so);
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let want = self.rec.get("builds")[name]["sha256"].as_str().ok_or_else(|| format!("the record holds no build of {name}: run press build first"))?;
            let got = wire::hex(&wire::sha256(&bytes));
            if got != want {
                return Err(format!("{name}: {} is not the binary press build recorded (sha256 {got}, recorded {want}): the deploy sends only recorded bytes", path.display()));
            }
            out.push((name, bytes));
        }
        Ok(out)
    }

    /// The bytes a buffer's write carries, at most: the largest chunk whose write transaction still fits a packet.
    pub fn chunk(&self) -> Res<usize> {
        let payer = self.deployer;
        let buffer = Pubkey::new_from_array([1; 32]);
        let budget = self.budget(10_000)?;
        let mut size = PACKET;
        loop {
            let ixs = [budget.clone(), vec![loader::write(&buffer, &payer, 0, &vec![0u8; size])]].concat();
            let (m, signers) = wire::message(&ixs, &payer, &[0; 32]);
            let tx = 1 + 64 * signers.len() + m.len();
            if tx <= PACKET {
                return Ok(size);
            }
            size -= tx - PACKET;
        }
    }

    /// The account of `key` as the cluster holds it.
    pub fn account(&self, key: &Pubkey) -> Res<Option<crate::rpc::Account>> {
        self.rpc.account(&key.to_string())
    }

    /// A program as it stands: None while nothing is deployed at its id; else the slot of its deploy, its authority, and whether the bytes
    /// it runs are this binary's.
    pub fn deployed(&self, program: &Pubkey, bin: &[u8]) -> Res<Option<(u64, Option<Pubkey>, bool)>> {
        let Some(a) = self.account(program)? else { return Ok(None) };
        if a.owner != LOADER_V3.to_string() {
            return Ok(None);
        }
        if !a.executable {
            return Err(format!("{program}: a program account of the loader that is not executable"));
        }
        let State::Program { programdata } = loader::state(&a.data)? else { return Err(format!("{program}: not a program account")) };
        let pd = self.account(&programdata)?.ok_or_else(|| format!("{program}: its program data is not on the cluster"))?;
        let State::ProgramData { slot, authority } = loader::state(&pd.data)? else { return Err(format!("{program}: its program data is not one")) };
        let code = &pd.data[loader::PROGRAMDATA_HEADER..];
        let same = code.len() >= bin.len() && &code[..bin.len()] == bin && code[bin.len()..].iter().all(|b| *b == 0);
        Ok(Some((slot, authority, same)))
    }

    // ---------------------------------------------------------------- the need
    /// The SOL the sitting needs from here, by the chain's own rent: what each step still to come locks for good, a buffer's rent on hand
    /// until its deploy gives it back, every signature's fee, and a margin; each line said. What already stands costs nothing more.
    pub fn need(&self, bins: &[(&'static str, Vec<u8>)]) -> Res<(u64, Vec<(String, u64)>)> {
        let chunk = self.chunk()? as u64;
        let price = self.p.u64_or("priorityMicroLamports", 0)?;
        let tx_fee = |signatures: u64, units: u64| signatures * SIGNATURE_FEE + price * units / 1_000_000;
        let mut lines: Vec<(String, u64)> = Vec::new();
        let (mut running, mut peak, mut fees): (u64, u64, u64) = (0, 0, 0);
        let rent36 = self.rpc.rent(loader::PROGRAM_LEN)?;
        for (name, bin) in bins {
            let program = program_id(&self.ids, name);
            if self.deployed(&program, bin)?.is_some() {
                lines.push((format!("{name}: deployed already, nothing more"), 0));
                continue;
            }
            let len = bin.len();
            let buffer_rent = self.rpc.rent(loader::BUFFER_HEADER + len)?;
            let pd_rent = self.rpc.rent(loader::PROGRAMDATA_HEADER + len)?;
            let (buffer, _) = self.buffer_standing(name, len)?;
            let mut writes = (len as u64).div_ceil(chunk);
            // what the buffer holds when its deploy drains it to the deployer: its rent, made now or by a pass before
            let funds = match buffer {
                None => {
                    running += buffer_rent;
                    fees += tx_fee(1, 10_000);
                    lines.push((format!("{name}: its named buffer, {} bytes (rent on hand until its deploy gives it back)", loader::BUFFER_HEADER + len), buffer_rent));
                    buffer_rent
                }
                Some(b) => {
                    let standing = self.account(&b)?;
                    let held = standing.as_ref().map(|a| a.lamports).unwrap_or(0);
                    // what a pass before wrote is paid: only the chunks that still differ are counted, so a sitting cut mid-upload
                    // needs from here exactly what is left (counted whole again, it came back short by the fees it had already paid)
                    if let Some(a) = &standing {
                        let (body, c) = (&a.data[loader::BUFFER_HEADER.min(a.data.len())..], chunk as usize);
                        writes = (0..len.div_ceil(c)).filter(|i| body.get(i * c..((i + 1) * c).min(len)) != Some(&bin[i * c..((i + 1) * c).min(len)])).count() as u64;
                    }
                    lines.push((format!("{name}: its buffer stands already, its rent paid; its {} SOL come back at the deploy", sol(held)), 0));
                    held
                }
            };
            fees += writes * tx_fee(1, 10_000);
            peak = peak.max(running);
            // the deploy, in one transaction: the program account made first; then the loader drains the buffer to the deployer before it
            // makes the program data from it (the loader's own order), so at that instant the deployer holds the program account's rent
            // and what the program data's rent passes the buffer's
            peak = peak.max(running + rent36 + pd_rent.saturating_sub(funds));
            running = (running + rent36 + pd_rent).saturating_sub(funds);
            fees += tx_fee(2, 50_000);
            peak = peak.max(running);
            lines.push((format!("{name}: its program data, {} bytes, for good (paid from its buffer's rent, drained to the deployer first)", loader::PROGRAMDATA_HEADER + len), pd_rent));
            lines.push((format!("{name}: its program account, {} bytes, for good", loader::PROGRAM_LEN), rent36));
            lines.push((format!("{name}: {writes} writes of {chunk} bytes at most, and its deploy"), writes * tx_fee(1, 10_000) + tx_fee(2, 50_000)));
        }
        if self.account(&self.at.config)?.is_none() {
            let rents = [
                self.rpc.rent(client::Config::SIZE)?,
                self.rpc.rent(client::Treasury::SIZE)?,
                self.rpc.rent(client::Registry::MIN_SIZE)?,
                self.rpc.rent(nft::MintCurve::SIZE)?,
                self.rpc.rent(client::Registry::MIN_SIZE)?,
                self.rpc.rent(client::Treasury::SIZE)?,
            ];
            let r: u64 = rents.iter().sum();
            running += r;
            fees += tx_fee(1, 90_000);
            lines.push(("root 0: its config, treasury, registry and curve (the floor's mint terms), and his posiverse unit 1's registry and treasury, in the founding (root 0's registry grows a row a seat minted: each buyer pays its own)".into(), r));
        }
        if self.account(&treasury_of(&self.p)?)?.is_none() {
            let r = self.rpc.rent(warm_mine::token::ACCOUNT_LEN)?;
            running += r;
            fees += tx_fee(1, 30_000);
            lines.push(("root 0's vault of the dollar, the $WARM mine's treasury".into(), r));
        }
        if self.account(&self.at.fund)?.is_none() {
            // THE CRANK: everything anyone may send for the seven acts, the deployer its payer (the four, THE SWARM NAMES ITS MINER's sixth, and
            // THE TREASURY PAYS THE COMPUTE's seventh and eighth): unit 1's sessions of the four acts it convenes (their rent home at adjourn),
            // the fund's, root 0's market's and the router's allowance's accounts made at their execute, and every signature's fee: per act a
            // convene, one attest of unit 1's agents' envelopes (two votes), two seals, a mandate and an execute; and the ratification. THE
            // WHOLE TREE DECIDES: each proposal and each vote of root 0's floor is its seat owner's own transaction, paid by that owner, never by
            // the press
            let session = client::Session::SIZE + client::Session::ROSTER_HEADER + client::Session::CONVENER_TAIL;
            let r = 4 * self.rpc.rent(session)? + self.rpc.rent(client::Fund::SIZE)? + self.rpc.rent(poleis_treasury::state::Market::SIZE)? + self.rpc.rent(client::Allowance::SIZE)?;
            running += r;
            fees += 7 * (6 * tx_fee(1, 40_000) + 2 * SIGNATURE_FEE) + tx_fee(1, 10_000);
            lines.push(("the crank of the four acts, the mine's miner, the treasury's dollar cap, the router's month and the ratification (press crank, press ratify): unit 1's four convened sessions, the fund's, root 0's market's and the router's allowance's accounts (the proposals and the floor's votes are the seat owners' own)".into(), r));
        }
        // THE STAKE EARNS: the ninth act where the params name it (its rule whole), until its proposal stands: unit 1's convened session of it,
        // and its convene, attest, two seals, mandate and execute
        for nine in crate::engine::later(&self.p, &self.ids, &self.at, &self.deployer)? {
            if self.account(&client::proposal_pda(&nine.hash))?.is_none() {
                let mut r = self.rpc.rent(client::Session::SIZE + client::Session::ROSTER_HEADER + client::Session::CONVENER_TAIL)?;
                // THE LIGHT'S PLAN: act 10's execute makes root 0's plan of her token, the crank its payer
                if nine.n == crate::light::PLAN_ACT {
                    r += self.rpc.rent(poleis_treasury::state::TokenPlan::SIZE)?;
                }
                running += r;
                fees += 6 * tx_fee(1, 40_000) + 2 * SIGNATURE_FEE;
                lines.push((format!("the crank of act {} ({}): unit 1's convened session of it (its rent home at adjourn)", nine.n, nine.name), r));
            }
        }
        if self.account(&self.at.mine)?.is_none() {
            let mut r = self.rpc.rent(warm_mine::token::MINT_LEN)? + self.rpc.rent(warm_mine::token::ACCOUNT_LEN)? + self.rpc.rent(warm_mine::state::Mine::SIZE)?;
            let face = self.metaplex_stands()?;
            if face {
                r += self.rpc.rent(METADATA_MAX)? + METAPLEX_FEE;
            }
            running += r;
            fees += tx_fee(1, 120_000);
            lines.push((
                format!("the $WARM mine: its mint, vault and mine{}", if face { ", and its face (Metaplex's metadata at its largest, and Metaplex's fee as last read, an upper bound)" } else { " (no face: no Metaplex on this cluster)" }),
                r,
            ));
        }
        if self.account(&self.at.board)?.is_none() {
            let r = self.rpc.rent(BOARD_SIZE)?;
            running += r;
            fees += tx_fee(1, 20_000);
            lines.push(("the market's board".into(), r));
        }
        // THE POOL IS BORN AT THE LAUNCH: its reserve, its mint and root 0's vault of the mint, made ahead (press pool); its account and its
        // list, made in act 5's execute by the deployer as its crank, with the act's crank (unit 1's convened session of it, home at adjourn,
        // and a convene, one attest of unit 1's two agents' envelopes, two seals, a mandate and the execute), where the stake pool program
        // stands (a rehearsal's chain may have none: the birth is not run there, and not counted)
        let pool = crate::pool::Pool::of(&self.p, &self.deployer, &self.ids.poleis)?;
        let standing = |k: &Pubkey| -> Res<bool> { Ok(self.account(k)?.is_some_and(|a| !crate::pool::is_dust(&a))) };
        let mut ahead = 0u64;
        for (key, size) in [(pool.reserve, crate::pool::RESERVE_LEN), (pool.mint, crate::pool::MINT_LEN), (pool.vault, warm_mine::token::ACCOUNT_LEN)] {
            if !standing(&key)? {
                ahead += self.rpc.rent(size)?;
            }
        }
        if ahead > 0 {
            running += ahead;
            fees += tx_fee(1, 60_000);
            lines.push(("the pool, made ahead (press pool): its reserve, its mint and root 0's vault of the mint".into(), ahead));
        }
        if !standing(&pool.pool)? {
            if self.stake_pool_stands()? {
                let session = client::Session::SIZE + client::Session::ROSTER_HEADER + client::Session::CONVENER_TAIL;
                let list = crate::pool::list_len(pool.max_validators);
                let r = self.rpc.rent(crate::pool::POOL_LEN)? + self.rpc.rent(list)? + self.rpc.rent(session)?;
                running += r;
                fees += 6 * tx_fee(1, 40_000) + 2 * SIGNATURE_FEE;
                lines.push((
                    format!("the pool's birth (act 5): its account, {} bytes, and its validator list for {} validators, {list} bytes, made in its execute's transaction; unit 1's convened session of it; the act's crank", crate::pool::POOL_LEN, pool.max_validators),
                    r,
                ));
            } else {
                lines.push((format!("{}: not counted", crate::pool::NOT_RUN), 0));
            }
        }
        fees += 3 * tx_fee(1, 10_000);
        peak = peak.max(running);
        lines.push(("every signature's fee".into(), fees));
        lines.push(("a margin".into(), MARGIN));
        Ok((peak + fees + MARGIN, lines))
    }

    fn metaplex_stands(&self) -> Res<bool> {
        Ok(self.account(&METAPLEX)?.is_some_and(|a| a.executable))
    }

    // ---------------------------------------------------------------- the deploy
    /// A program's named buffer, if one of ours stands for a binary of this length: its address and its seed. A buffer is named by the
    /// deployer and a seed (the system program's `with_seed` address), so a sitting cut short finds it again with no key of its own.
    fn buffer_standing(&self, name: &str, len: usize) -> Res<(Option<Pubkey>, String)> {
        for n in 0..8 {
            let seed = format!("launch-press:{name}:{n}");
            let at = loader::address_with_seed(&self.deployer, &seed, &LOADER_V3)?;
            match self.account(&at)? {
                None => return Ok((None, seed)),
                Some(a) if a.owner == LOADER_V3.to_string() => {
                    if loader::state(&a.data)? == (State::Buffer { authority: Some(self.deployer) }) && a.data.len() == loader::BUFFER_HEADER + len {
                        return Ok((Some(at), seed));
                    }
                    if loader::state(&a.data)? == (State::Buffer { authority: Some(self.deployer) }) {
                        // ours, for another length: closed by the deploy, its rent home
                        return Ok((None, seed));
                    }
                }
                // a buffer of no one's, or lamports someone sent to this address: the next seed
                Some(_) => {}
            }
        }
        Err(format!("{name}: no free seed for its buffer"))
    }

    /// Its buffer: found standing (and closed first if it holds another length), or made: created by seed and initialized, the deployer
    /// its authority.
    fn buffer(&mut self, name: &str, len: usize, deployer: &Key) -> Res<Pubkey> {
        let (standing, seed) = self.buffer_standing(name, len)?;
        let at = loader::address_with_seed(&self.deployer, &seed, &LOADER_V3)?;
        if let Some(b) = standing {
            self.rec.resumed(format!("{name}: its named buffer {b} (seed {seed}) stood already"))?;
            self.rec.v["buffers"][name] = json!({ "address": b.to_string(), "seed": seed });
            self.rec.save()?;
            return Ok(b);
        }
        if let Some(a) = self.account(&at)? {
            if a.owner == LOADER_V3.to_string() {
                self.send("deploy", &format!("{name}: a buffer of ours for another binary closed, its rent home"), vec![loader::close_buffer(&at, &self.deployer, &self.deployer)], &[deployer])?;
            }
        }
        let size = loader::BUFFER_HEADER + len;
        let rent = self.rpc.rent(size)?;
        let ixs = vec![loader::create_account_with_seed(&self.deployer, &at, &self.deployer, &seed, rent, size as u64, &LOADER_V3), loader::initialize_buffer(&at, &self.deployer)];
        self.send("deploy", &format!("{name}: its named buffer made, {size} bytes (seed {seed})"), ixs, &[deployer])?;
        self.rec.v["buffers"][name] = json!({ "address": at.to_string(), "seed": seed, "rent": rent });
        self.rec.save()?;
        Ok(at)
    }

    /// The buffer written until it holds the binary: each pass reads it back and writes the chunks that differ, BATCH at a time, so a pass
    /// cut short anywhere is finished by the next.
    fn write_buffer(&mut self, name: &str, buffer: &Pubkey, bin: &[u8], deployer: &Key) -> Res<()> {
        let chunk = self.chunk()?;
        let total = bin.len().div_ceil(chunk);
        let budget = self.budget(10_000)?;
        // what the cluster takes for a write: its signature and the priority fee its budget asks (the record wrote 5,000 for every
        // write until the rehearsal of mainnet's priority fee counted 1,300 more on each)
        let write_fee = SIGNATURE_FEE + priority_lamports(self.p.u64_or("priorityMicroLamports", 0)?, sized_limit(10_000));
        let mut first_pass = true;
        let mut written = 0usize;
        // the passes, one after the other, that left the buffer no fuller: after three the step stops and says what is left (a write that
        // can never land is not sent for ever, each try a fee), and a pass after an idle one waits first (an endpoint under load is not
        // pressed harder)
        let (mut idle, mut left) = (0u32, usize::MAX);
        loop {
            let data = self.account(buffer)?.ok_or("the buffer is gone")?.data;
            let body = &data[loader::BUFFER_HEADER..];
            let pending: Vec<usize> = (0..total).filter(|i| {
                let (a, b) = (i * chunk, ((i + 1) * chunk).min(bin.len()));
                body[a..b] != bin[a..b]
            }).collect();
            if pending.is_empty() {
                eprintln!("  {name}: the buffer holds the binary, {} bytes in {total} chunks", bin.len());
                return Ok(());
            }
            if first_pass && pending.len() < total {
                self.rec.resumed(format!("{name}: {} of its {total} chunks stood in the buffer already; {} left to write", total - pending.len(), pending.len()))?;
            }
            first_pass = false;
            idle = if pending.len() < left { 0 } else { idle + 1 };
            left = pending.len();
            if idle >= IDLE_PASSES {
                return Err(format!("{name}: {IDLE_PASSES} passes over its buffer landed no write, {left} of its {total} chunks still to write: stopped. Nothing is lost: the buffer keeps what landed, and `press deploy` resumes from it"));
            }
            if idle > 0 {
                std::thread::sleep(Duration::from_secs(2 * u64::from(idle)));
            }
            for batch in pending.chunks(BATCH) {
                let bh = self.rpc.blockhash()?;
                let mut sent = Vec::new();
                for &i in batch {
                    let (a, b) = (i * chunk, ((i + 1) * chunk).min(bin.len()));
                    let ixs = [budget.clone(), vec![loader::write(buffer, &self.deployer, a as u32, &bin[a..b])]].concat();
                    let (w, first) = wire::signed(&ixs, &self.deployer, &bh, &[deployer])?;
                    let sig = wire::base58(&first);
                    // a write the cluster would not take now is written by the next pass, never a stop
                    match self.rpc.send(&w, false) {
                        Ok(_) => sent.push((sig, w, a)),
                        Err(e) => eprintln!("  {name}: a write at {a} not taken now ({e}); the next pass writes it"),
                    }
                }
                // their confirmations, the unseen sent again, thirty seconds at most: what did not land is written by the next pass
                let begun = Instant::now();
                let mut landed: Vec<Landed> = Vec::new();
                let mut open: Vec<(String, Vec<u8>, usize)> = sent;
                let mut resent = Instant::now();
                while !open.is_empty() && begun.elapsed() < Duration::from_secs(30) {
                    let st = self.rpc.statuses(&open.iter().map(|o| o.0.clone()).collect::<Vec<_>>())?;
                    let mut still = Vec::new();
                    for (o, s) in open.into_iter().zip(st) {
                        match s {
                            Some((slot, err, level)) if err.is_null() && (level == "confirmed" || level == "finalized") => {
                                written += 1;
                                eprintln!("  {name}: write {written} at {} of {}, slot {slot}", o.2, bin.len());
                                landed.push(Landed { step: "deploy".into(), what: format!("{name}: write at {}", o.2), signature: o.0, slot, fee: write_fee, units: None, bytes: o.1.len(), signatures: 1 });
                            }
                            Some((_, err, _)) if !err.is_null() => eprintln!("  {name}: a write at {} failed: {err}", o.2),
                            _ => still.push(o),
                        }
                    }
                    open = still;
                    if !open.is_empty() && resent.elapsed() > Duration::from_secs(2) {
                        for o in &open {
                            let _ = self.rpc.send(&o.1, false);
                        }
                        resent = Instant::now();
                    }
                    std::thread::sleep(Duration::from_millis(300));
                }
                self.rec.landed_all(&landed)?;
            }
        }
    }

    /// The program account and the deploy, in one transaction the deployer pays and the program's key signs (its one signature, ever).
    /// A program address someone sent lamports to is made by top-up, allocate and assign instead of create, so a stranger's dust does not
    /// bar it.
    fn deploy_ixs(&self, program: &Key, buffer: &Pubkey, len: usize) -> Res<Vec<Instruction>> {
        let rent36 = self.rpc.rent(loader::PROGRAM_LEN)?;
        let mut ixs = Vec::new();
        match self.account(&program.address)? {
            None => ixs.push(loader::create_account(&self.deployer, &program.address, rent36, loader::PROGRAM_LEN as u64, &LOADER_V3)),
            Some(a) if a.owner == SYSTEM.to_string() && a.data.is_empty() => {
                if a.lamports < rent36 {
                    ixs.push(loader::transfer(&self.deployer, &program.address, rent36 - a.lamports));
                }
                ixs.push(loader::allocate(&program.address, loader::PROGRAM_LEN as u64));
                ixs.push(loader::assign(&program.address, &LOADER_V3));
            }
            Some(a) => return Err(format!("{}: an account stands at the program's id ({} lamports, owner {}): not a program of this sitting", program.address, a.lamports, a.owner)),
        }
        ixs.push(loader::deploy_with_max_data_len(&self.deployer, &program.address, buffer, &self.deployer, len as u64));
        Ok(ixs)
    }

    /// THE DEPLOY: the market, then the $WARM program, then Poleis, each through its named buffer, deployed at its own length and read
    /// back, the bytes held to the recorded hash; then root 0, the slot Poleis becomes callable. Short of the SOL it needs, it starts
    /// nothing.
    pub fn deploy(&mut self) -> Res<()> {
        self.p.sendable()?;
        self.checked()?;
        // THE POOL IS BORN AT THE LAUNCH, on the stake pool program's bytes a rehearsal proved: read at the sitting's first step and recorded
        // (or held to the record and the params' hash, or nothing starts); on mainnet it stands, or this is no mainnet
        if self.pool_program("deploy")?.is_none() && self.p.cluster == Cluster::Mainnet {
            return Err(crate::pool::NOT_MAINNET.into());
        }
        let bins = self.binaries()?;
        let (need, lines) = self.need(&bins)?;
        let balance = self.rpc.balance(&self.deployer.to_string())?;
        say_need(&lines, need, balance, &self.deployer);
        self.rec.v["need"] = json!({ "lamports": need, "balance": balance, "at": now_iso(), "lines": lines.iter().map(|(w, l)| json!({ "what": w, "lamports": l })).collect::<Vec<_>>() });
        self.rec.save()?;
        if balance < need {
            return Err(format!("the deployer {} holds {} SOL, the sitting needs {} SOL from here: short by {} SOL, nothing started", self.deployer, sol(balance), sol(need), sol(need - balance)));
        }
        if self.rec.get("before").is_null() {
            // what the deployer held when the sitting started: what it spent is measured from here
            self.rec.v["before"] = json!({ "deployer": balance, "need": need, "at": now_iso() });
            self.rec.save()?;
        }
        let keys = self.keys.load(&KEY_NAMES, "the deploy")?;
        let (deployer, by_name) = (&keys[0], |n: &str| KEY_NAMES.iter().position(|k| *k == n).unwrap());
        for (name, bin) in &bins {
            let program = &keys[by_name(name)];
            if program.address != program_id(&self.ids, name) {
                return Err(format!("{name}: its key is {}, the sitting's id {}", program.address, program_id(&self.ids, name)));
            }
            if let Some((slot, authority, same)) = self.deployed(&program.address, bin)? {
                if !same {
                    return Err(format!("{name}: {} runs other bytes than the recorded build", program.address));
                }
                self.rec.resumed(format!("{name}: deployed already at {} (slot {slot}, authority {})", program.address, authority.map(|a| a.to_string()).unwrap_or("none".into())))?;
                self.rec.v["deploys"][name] = json!({ "program": program.address.to_string(), "slot": slot, "bytes": bin.len(), "sha256": wire::hex(&wire::sha256(bin)) });
                self.rec.save()?;
                if *name == "poleis" {
                    self.found(deployer, Some(slot))?;
                }
                continue;
            }
            let buffer = self.buffer(name, bin.len(), deployer)?;
            self.write_buffer(name, &buffer, bin, deployer)?;
            // the founding signed before the deploy is sent, so it leaves the slot Poleis becomes callable
            let founding = if *name == "poleis" { Some(self.founding(deployer)?) } else { None };
            let ixs = self.deploy_ixs(program, &buffer, bin.len())?;
            let what = format!("{name}: deployed at {} at its own length, {} bytes", program.address, bin.len());
            let l = match founding {
                // THE SAME BREATH: the deploy's slot read the moment the cluster has it, the founding sent the slot after it, nothing read
                // between; both confirmed after, and the program read back last
                Some((fw, ff, window)) => {
                    let (dw, df, slot) = self.fire(&what, ixs, &[deployer, program])?;
                    self.same_breath(&fw, &ff, window, slot, deployer)?;
                    self.settle("deploy", &what, &dw, &df, 2)?
                }
                None => self.send("deploy", &what, ixs, &[deployer, program])?,
            };
            let Some((slot, authority, same)) = self.deployed(&program.address, bin)? else { return Err(format!("{name}: deployed and not found")) };
            if !same || authority != Some(self.deployer) {
                return Err(format!("{name}: read back other bytes or another authority ({authority:?})"));
            }
            self.rec.v["deploys"][name] = json!({ "program": program.address.to_string(), "slot": slot, "signature": l.signature, "bytes": bin.len(), "sha256": wire::hex(&wire::sha256(bin)), "authority": self.deployer.to_string() });
            self.rec.save()?;
        }
        Ok(())
    }

    /// Every value of the params the sitting sends, held to the programs' own bounds BEFORE anything starts (`params_checked`), and every key
    /// a step of the press made beside his keys held to the param that names it (THE PRESS IS READY FOR MAINNET: `seat::kept_keys_held`).
    pub fn checked(&self) -> Res<()> {
        params_checked(&self.p, &self.ids, &self.at)?;
        crate::seat::kept_keys_held(&self.dir, &self.p)
    }

    /// Root 0's founding: init_config, Poleis's NFT genesis with no founder (the floor's curve, its window counted from now and its permits
    /// bound to this chain's genesis hash) and found_child of his posiverse in its chair, in ONE transaction the deployer pays, signed now.
    fn founding(&self, deployer: &Key) -> Res<(Vec<u8>, [u8; 64], (u64, u64))> {
        let (ixs, window) = founding_ixs(&self.p, &self.deployer, self.rpc.slot()?, &self.chain_genesis()?)?;
        let (w, first) = wire::signed(&ixs, &self.deployer, &self.rpc.blockhash()?, &[deployer])?;
        if w.len() > PACKET {
            return Err(format!("the founding is {} bytes, past {PACKET}", w.len()));
        }
        Ok((w, first, window))
    }

    /// The chain's own genesis hash, as the sitting read it from the cluster (the record's): the domain every permit of the floor's curve names.
    pub fn chain_genesis(&self) -> Res<[u8; 32]> {
        match self.rec.get("genesisHash").as_str() {
            Some(g) => wire::unbase58(g).map_err(|e| format!("the record's genesis hash: {e}")),
            None => wire::unbase58(&self.rpc.check_cluster()?),
        }
    }

    /// THE SAME BREATH: the founding sent the slot after Poleis's deploy (a program is callable from the slot after it: THE CEREMONY,
    /// REHEARSED, finding 2), and again with a fresh blockhash while the cluster says the program is not callable yet.
    fn same_breath(&mut self, wire0: &[u8], first0: &[u8; 64], window: (u64, u64), deployed: u64, deployer: &Key) -> Res<()> {
        crate::rpc::wait(Duration::from_secs(60), || Ok((self.rpc.slot()? > deployed).then_some(())))?;
        let (mut w, mut first, mut window) = (wire0.to_vec(), *first0, window);
        for attempt in 1..=5 {
            match self.send_wire("found", "root 0 founded with no key in its chairs and no founder, its floor the minted Lyssajudrons, his posiverse unit 1 in the params' chair: init_config, the NFT genesis and found_child in one transaction, the slot Poleis became callable", &w, &first, 1) {
                Ok(l) => {
                    eprintln!("  THE FOUNDING: one transaction of {} bytes, {} CU, {}", l.bytes, l.units.map(|u| u.to_string()).unwrap_or("?".into()), l.signature);
                    self.rec.v["founding"] = json!({
                        "deploySlot": deployed, "foundSlot": l.slot, "slotsBetween": l.slot - deployed, "signature": l.signature, "attempts": attempt,
                        "bytes": l.bytes, "units": l.units,
                        "opens": window.0, "closes": window.1, "founders": [],
                        "posiverse": { "unit": POSIVERSE, "chair": posiverse_chair(&self.p)?, "agents": self.p.agents()?.iter().map(|a| a.to_string()).collect::<Vec<_>>() },
                        "charter": wire::hex(&wire::unhex::<32>(&self.p.text("genesis.charter")?)?), "blockTime": self.rpc.block_time(l.slot).ok().flatten(),
                    });
                    self.rec.save()?;
                    self.root_is_ours()?;
                    // THE LAUNCH NAMES THE SEAL'S ROAD: root 0 stands as this sitting's, so the road to its seal is written beside the record
                    self.road_said();
                    return Ok(());
                }
                Err(e) if attempt < 5 && (e.contains("InvalidProgramForExecution") || e.contains("ProgramAccountNotFound") || e.contains("Unsupported") || e.contains("never confirmed")) => {
                    eprintln!("  the founding, try {attempt}: {e}; again the next slot");
                    let s = self.rpc.slot()?;
                    crate::rpc::wait(Duration::from_secs(30), || Ok((self.rpc.slot()? > s).then_some(())))?;
                    let (w2, f2, s2) = self.founding(deployer)?;
                    (w, first, window) = (w2, f2, s2);
                }
                Err(e) => return Err(format!("root 0: {e}: if a stranger founded root 0 first, the program is no sitting's: the deployer still holds its code, closes it and deploys under a fresh id (his call)")),
            }
        }
        Err("root 0: never founded".into())
    }

    /// ROOT 0 IS THIS SITTING'S, or the sitting stops: its mode. Poleis lets the first wallet found root 0, so a root that stands is held
    /// to this sitting whole, never by its chairs alone (a stranger who saw the founding fly could seat the same chairs under a curve and a
    /// plan of its own).
    /// In genesis (until its curve's window has closed, or every seat is minted, with a seat minted, and it is ratified): its authority is
    /// the deployer, the payer of its founding by Poleis's own rule, and every value of the founding is the params': the default mint, the
    /// charter, the plan, the window, the rung, the name; its curve the params' (its first price and step, its seats the rung's hexagons, its
    /// permit issuer, the chain its permits name, its window the one the record holds); its chairs his posiverse, unit 1, in the params' chair
    /// and the eleven others empty (no founder), and unit 1 the params' whole (its parent, its chair, its agents in order, its charter, window,
    /// plan, name, no floor). Held so, the record says the founding is verified.
    /// Once a council: its authority is the PDA nobody holds, and the record's own verified founding says it is this sitting's (one
    /// founding of root 0 ever lands on a program); its chairs and its numbers are then the polis's own to move.
    pub fn root_is_ours(&mut self) -> Res<u8> {
        let c = self.account(&self.at.config)?.ok_or("root 0's config is not on the cluster")?;
        let cv = client::ConfigView::parse(&c.data).ok_or("root 0's config does not parse")?;
        let r = self.account(&self.at.registry)?.ok_or("root 0's registry is not on the cluster")?;
        let rv = client::RegistryView::parse(&r.data).ok_or("root 0's registry does not parse")?;
        if cv.root != ROOT_UNIT || rv.unit != ROOT_UNIT {
            return Err("root 0's config or registry is another unit's".into());
        }
        let verified = self.rec.get("founding")["verified"] == json!(true);
        if cv.mode == poleis_treasury::MODE_COUNCIL {
            if cv.authority != client::council_pda_of(ROOT_UNIT) {
                return Err(format!("root 0 is a council whose authority is {}, not the PDA nobody holds", cv.authority));
            }
            if !verified {
                return Err("root 0 is a council, and this folder's record holds no founding it verified: this is not the sitting that founded it".into());
            }
            eprintln!("  root 0 stands, ratified: a council, founded by this sitting (its record's own verified founding)");
            return Ok(cv.mode);
        }
        if cv.mode != poleis_treasury::MODE_GENESIS {
            return Err(format!("root 0 is in mode {}, neither genesis nor a council", cv.mode));
        }
        if cv.authority == self.deployer && verified {
            // held whole when it was founded; what a mint moves since (its seats, its count) is the polis's own
            eprintln!("  root 0 stands: in genesis, its authority the deployer, its founding verified by this sitting");
            return Ok(cv.mode);
        }
        if rv.default_mint != cv.default_mint {
            return Err("root 0's registry's mint is not its config's".into());
        }
        let curve = self.account(&nft::curve_pda(ROOT_UNIT))?.and_then(|a| nft::MintCurve::decode(&a.data).ok());
        let as_found = Founded {
            authority: cv.authority,
            default_mint: cv.default_mint,
            charter: rv.charter,
            plan_cap: rv.plan_cap,
            plan_period: rv.plan_period,
            window: rv.window,
            rung: u64::from(rv.rung),
            name: rv.name.clone().into_bytes(),
            desc: rv.desc.clone().into_bytes(),
            chairs: rv.pentagons.iter().map(|x| (x.holder, x.kind)).collect(),
            curve,
            posiverse: self.crew()?,
        };
        let f = self.rec.get("founding").clone();
        let sent = f["opens"].as_u64().zip(f["closes"].as_u64());
        founding_held(&as_found, &self.p, &self.deployer, &self.chain_genesis()?, sent)?;
        let p = &self.p;
        let stranger = |what: String| format!("root 0 stands, and {what}: {STRANGER}");
        match (sent, as_found.curve) {
            (Some(_), _) | (None, None) => {}
            // a sitting cut between the founding's landing and its record: the founding is read off the chain (the config's first
            // transaction), held above to the deployer and the params, its window to the params' length
            (None, Some(curve)) => {
                let after = p.u64("floor.opensAfterSlots")?;
                // the founding: the oldest transaction of the config that landed well, ran Poleis, and whose slot the curve's window counts
                // from (signed at most a blockhash's life, 150 slots, before it landed)
                let history = self.rpc.signatures_for(&self.at.config.to_string(), 1000)?;
                let ran = format!("Program {} invoke [1]", self.ids.poleis);
                let mut found = None;
                for (signature, slot, _) in history.iter().rev().filter(|h| h.2) {
                    let left = curve.opens_slot.saturating_sub(*slot);
                    if *slot <= curve.opens_slot && left <= after && after - left <= 150 {
                        let logs = self.rpc.transaction(signature)?.map(|t| t.3).unwrap_or(Value::Null);
                        if logs.as_array().is_some_and(|a| a.iter().any(|l| l.as_str() == Some(ran.as_str()))) {
                            found = Some((signature.clone(), *slot));
                            break;
                        }
                    }
                }
                let Some((signature, slot)) = found else {
                    return Err(stranger("its founding cannot be read back off the chain as this sitting's (no transaction of its config that ran Poleis where its curve's window counts from)".into()));
                };
                self.rec.resumed(format!("the founding was not in the record: read off the chain, slot {slot}, {signature}"))?;
                self.rec.v["founding"] = json!({
                    "foundSlot": slot, "signature": signature, "recovered": true, "opens": curve.opens_slot, "closes": curve.closes_slot,
                    "founders": [], "posiverse": { "unit": POSIVERSE, "chair": as_found.posiverse.as_ref().map(|c| c.chair), "agents": as_found.posiverse.as_ref().map(|c| c.chairs.iter().filter(|x| x.1 == HOLDER_KEY).map(|x| Pubkey::new_from_array(x.0).to_string()).collect::<Vec<_>>()) }, "charter": wire::hex(&as_found.charter), "blockTime": self.rpc.block_time(slot).ok().flatten(),
                });
            }
        }
        self.rec.v["founding"]["verified"] = json!(true);
        self.rec.save()?;
        eprintln!("  root 0 stands: in genesis, its authority the deployer, its founding the params' whole: no key in its chairs and no founder, its floor the params' curve, his posiverse unit 1 in the params' chair, its chairs its agents' own keys");
        Ok(cv.mode)
    }

    /// Unit 1 as the cluster holds it, if it stands.
    pub fn crew(&self) -> Res<Option<Crew>> {
        let Some(a) = self.account(&client::registry_pda(POSIVERSE))? else { return Ok(None) };
        let v = client::RegistryView::parse(&a.data).ok_or("unit 1's registry does not parse")?;
        if v.unit != POSIVERSE {
            return Err("unit 1's registry is another unit's".into());
        }
        Ok(Some(Crew {
            parent: v.parent,
            chair: v.chair_in_parent,
            root: v.root,
            chairs: v.pentagons.iter().map(|x| (x.holder, x.kind)).collect(),
            charter: v.charter,
            window: v.window,
            plan_cap: v.plan_cap,
            plan_period: v.plan_period,
            name: v.name.into_bytes(),
            desc: v.desc.into_bytes(),
            seats: v.total_seats,
        }))
    }

    /// ROOT 0: read back when it stands (founded by this sitting's deploy); founded now when Poleis stands without it (a sitting cut
    /// between the deploy and the founding: said in the record).
    pub fn found(&mut self, deployer: &Key, deployed: Option<u64>) -> Res<()> {
        if self.account(&self.at.config)?.is_some() {
            self.root_is_ours()?;
            self.road_said();
            return Ok(());
        }
        let slot = match deployed {
            Some(s) => s,
            None => self.rec.get("deploys")["poleis"]["slot"].as_u64().ok_or("Poleis is not deployed by this sitting")?,
        };
        let now = self.rpc.slot()?;
        self.rec.resumed(format!("root 0 was not founded in the deploy's breath: founded now, {} slots after Poleis's deploy", now.saturating_sub(slot)))?;
        let (w, first, window) = self.founding(deployer)?;
        self.same_breath(&w, &first, window, slot, deployer)
    }

    /// `press found` alone.
    pub fn found_step(&mut self) -> Res<()> {
        self.p.sendable()?;
        // the curve's permit issuer is fixed at the founding: the key press issuer made, where it made one
        crate::seat::kept_keys_held(&self.dir, &self.p)?;
        if self.account(&self.at.config)?.is_some() {
            self.root_is_ours()?;
            self.road_said();
            return Ok(());
        }
        let keys = self.keys.load(&["deployer"], "root 0")?;
        self.found(&keys[0], None)
    }

    // ---------------------------------------------------------------- root 0's vault
    /// THE TREASURY IS THE SWARM'S OWN: root 0's vault of the dollar, made by Poleis's permissionless `create_vault` (a token account at
    /// Poleis's PDA whose authority is root 0's treasury, so tokens leave it only by a sealed TREASURY act), in the first sitting before
    /// the mine, which names it its treasury.
    pub fn vault(&mut self) -> Res<()> {
        self.p.sendable()?;
        let (vault, dollar) = (treasury_of(&self.p)?, self.p.key("warm.dollar")?);
        if let Some(a) = self.account(&vault)? {
            vault_held(&a.owner, &a.data, &dollar)?;
            eprintln!("  root 0's vault of the dollar stands: {vault}, its authority root 0's treasury");
            return Ok(());
        }
        self.account(&self.at.treasury)?.ok_or("root 0's treasury is not on the cluster: the founding comes first")?;
        let d = self.account(&dollar)?.ok_or_else(|| format!("the dollar {dollar} is not on the cluster"))?;
        if d.owner != TOKEN.to_string() || d.data.get(warm_mine::token::MINT_OFF_DECIMALS) != Some(&warm_mine::token::DOLLAR_DECIMALS) {
            return Err(format!("the dollar {dollar} is not a mint of six decimals"));
        }
        let keys = self.keys.load(&["deployer"], "root 0's vault")?;
        let l = self.send("vault", &format!("root 0's vault of the dollar made at {vault}, its authority root 0's treasury"), [self.budget(30_000)?, vec![client::create_vault(self.deployer, ROOT_UNIT, dollar, TOKEN)]].concat(), &[&keys[0]])?;
        let a = self.account(&vault)?.ok_or("root 0's vault is not on the cluster after its making")?;
        vault_held(&a.owner, &a.data, &dollar)?;
        self.rec.v["vault"] = json!({ "address": vault.to_string(), "dollar": dollar.to_string(), "authority": self.at.treasury.to_string(), "signature": l.signature, "slot": l.slot });
        self.rec.save()
    }

    // ---------------------------------------------------------------- the $WARM program
    /// THE $WARM PROGRAM OPENED: `init` signed by the deployer as the program's upgrade authority, naming the polis's agent PDA its AUTHORITY,
    /// root 0's own vault of the dollar its treasury (derived, never a param), the opening; with its face when Metaplex's program stands.
    pub fn warm(&mut self) -> Res<()> {
        self.p.sendable()?;
        let (treasury, dollar) = (treasury_of(&self.p)?, self.p.key("warm.dollar")?);
        let opening = self.opening()?;
        if let Some(m) = self.account(&self.at.mine)? {
            return self.check_mine(&m.data, &treasury, &dollar, opening);
        }
        // the treasury: root 0's vault of the dollar, six decimals
        let t = self.account(&treasury)?.ok_or_else(|| format!("root 0's vault {treasury} is not on the cluster: press vault makes it first"))?;
        let d = self.account(&dollar)?.ok_or_else(|| format!("the dollar {dollar} is not on the cluster"))?;
        vault_held(&t.owner, &t.data, &dollar)?;
        if d.owner != TOKEN.to_string() || d.data.get(warm_mine::token::MINT_OFF_DECIMALS) != Some(&warm_mine::token::DOLLAR_DECIMALS) {
            return Err(format!("the dollar {dollar} is not a mint of six decimals"));
        }
        let pd = loader::programdata_of(&self.ids.warm);
        let face = self.metaplex_stands()?;
        if !face && self.p.cluster == Cluster::Mainnet {
            return Err("Metaplex's program does not stand on this cluster: mainnet has it; this is no mainnet".into());
        }
        let mut data = vec![warm_mine::IX_INIT];
        data.extend_from_slice(self.at.agent.as_ref());
        data.extend_from_slice(&opening.to_le_bytes());
        let mut metas = vec![
            AccountMeta::new_readonly(self.deployer, true),
            AccountMeta::new(self.deployer, true),
            AccountMeta::new_readonly(pd, false),
            AccountMeta::new(self.at.mine, false),
            AccountMeta::new(self.at.mint, false),
            AccountMeta::new(self.at.vault, false),
            AccountMeta::new_readonly(treasury, false),
            AccountMeta::new_readonly(dollar, false),
            AccountMeta::new_readonly(TOKEN, false),
            AccountMeta::new_readonly(SYSTEM, false),
        ];
        let (name, symbol, uri) = (self.p.text("warm.name")?, self.p.text("warm.symbol")?, self.p.text("warm.uri")?);
        if face {
            for f in [name.as_bytes(), symbol.as_bytes(), uri.as_bytes()] {
                data.push(u8::try_from(f.len()).map_err(|_| "the face: a field past 255 bytes")?);
                data.extend_from_slice(f);
            }
            warm_mine::face::parse_face(&data[41..]).map_err(|_| "the face is out of Metaplex's bounds (name 32, symbol 10, URI 200 bytes)")?;
            metas.extend([AccountMeta::new(self.at.metadata, false), AccountMeta::new_readonly(METAPLEX, false)]);
        }
        let keys = self.keys.load(&["deployer"], "the $WARM program")?;
        let what = if face { "the $WARM mine opened with its face (Metaplex's metadata by CPI)" } else { "the $WARM mine opened, no face: Metaplex's program does not stand on this chain" };
        let l = self.send("warm", what, vec![Instruction { program_id: self.ids.warm, accounts: metas, data }], &[&keys[0]])?;
        let m = self.account(&self.at.mine)?.ok_or("the mine is not on the cluster after its init")?;
        self.check_mine(&m.data, &treasury, &dollar, opening)?;
        self.rec.v["warm"] = json!({
            "signature": l.signature, "slot": l.slot, "authority": self.at.agent.to_string(), "treasury": treasury.to_string(), "dollar": dollar.to_string(), "opening": opening,
            "face": if face { json!({ "name": name, "symbol": symbol, "uri": uri, "metadata": self.at.metadata.to_string() }) } else { json!("not made: Metaplex's program does not stand on this chain; its CPI is held to Metaplex's SDK by poleis/press/tests/metaplex.rs and to the program by warm_mine/tests/face.rs") },
        });
        self.rec.save()
    }

    /// The opening the mine counts from: the params' instant, or `the founding`, root 0's founding block time.
    fn opening(&self) -> Res<u64> {
        let v = self.p.at("warm.opening");
        if v.as_str() == Some("the founding") {
            let f = self.rec.get("founding");
            let t = match f["blockTime"].as_i64() {
                Some(t) => t,
                None => {
                    let slot = f["foundSlot"].as_u64().ok_or("the opening is the founding's instant, and the record holds no founding")?;
                    self.rpc.block_time(slot)?.ok_or("the founding's block has no time on this cluster")?
                }
            };
            return u64::try_from(t).map(|t| t * 1_000).map_err(|_| "a founding before the epoch".into());
        }
        crate::params::ms_of(v)
    }

    fn check_mine(&self, m: &[u8], treasury: &Pubkey, dollar: &Pubkey, opening: u64) -> Res<()> {
        use warm_mine::state::Mine;
        let at = |o: usize| Pubkey::new_from_array(m[o..o + 32].try_into().unwrap());
        let u64_at = |o: usize| u64::from_le_bytes(m[o..o + 8].try_into().unwrap());
        if m.len() != Mine::SIZE || m[0] != warm_mine::state::DISC_MINE {
            return Err("the mine's account is not a mine".into());
        }
        if at(Mine::OFF_AUTHORITY) != self.at.agent || at(Mine::OFF_TREASURY) != *treasury || at(Mine::OFF_DOLLAR) != *dollar || u64_at(Mine::OFF_OPENING) != opening {
            return Err(format!("the mine stands with another authority, treasury, dollar or opening: {} {} {} {}", at(Mine::OFF_AUTHORITY), at(Mine::OFF_TREASURY), at(Mine::OFF_DOLLAR), u64_at(Mine::OFF_OPENING)));
        }
        // THE MINE IS LIVE: only the mine's authority names its miner (`set_miner`), and that authority is the polis's hand, held to it above: a
        // miner that stands is the polis's own sealed act, the door's crank key (`seat::mine_vars` says the door's side; THE SWARM NAMES ITS MINER:
        // the press's sixth act names the params' warm.miner)
        let miner = at(Mine::OFF_MINER);
        let named = if miner == Pubkey::default() { "no miner yet (act 6, a sealed act of the polis, names the door's)".to_string() } else { format!("its miner {miner}, named by the polis's hand") };
        eprintln!("  the $WARM mine stands: its authority the polis's hand {}, its treasury {treasury}, its opening {opening}, {named}", self.at.agent);
        Ok(())
    }

    // ---------------------------------------------------------------- the market's board
    /// THE MARKET'S BOARD, opened for the polis's agent key (permissionless: whoever opens it gains nothing; every change after needs the
    /// agent's signature, a sealed act). A board of this number opened for another authority is no board of the polis: the sitting stops.
    pub fn market(&mut self) -> Res<()> {
        self.p.sendable()?;
        let id = self.board_id()?;
        if let Some(b) = self.account(&self.at.board)? {
            return self.check_board(&b.data, id);
        }
        let mut data = vec![0u8];
        data.extend_from_slice(&id.to_le_bytes());
        data.extend_from_slice(self.at.agent.as_ref());
        let ix = Instruction::new_with_bytes(self.ids.market, &data, vec![AccountMeta::new(self.deployer, true), AccountMeta::new(self.at.board, false), AccountMeta::new_readonly(SYSTEM, false)]);
        let keys = self.keys.load(&["deployer"], "the market's board")?;
        let l = self.send("market", &format!("the market's board {id} opened for the polis's hand"), vec![ix], &[&keys[0]])?;
        let b = self.account(&self.at.board)?.ok_or("the board is not on the cluster after its init")?;
        self.check_board(&b.data, id)?;
        self.rec.v["board"] = json!({ "id": id, "address": self.at.board.to_string(), "authority": self.at.agent.to_string(), "signature": l.signature, "slot": l.slot });
        self.rec.save()
    }

    fn check_board(&self, b: &[u8], id: u64) -> Res<()> {
        // Board: [disc:1][ver:1][board:8][authority:32] (poleis_roads state.rs, OFF_BOARD 2, OFF_AUTHORITY 10)
        if b.len() != BOARD_SIZE || u64::from_le_bytes(b[2..10].try_into().unwrap()) != id {
            return Err(format!("board {id}: the account at its address is not its board"));
        }
        if b[10..42] != self.at.agent.to_bytes() {
            return Err(format!("board {id} stands for another authority ({}): it is not the polis's, and the params must name another number", Pubkey::new_from_array(b[10..42].try_into().unwrap())));
        }
        eprintln!("  the market's board {id} stands, its authority the polis's hand");
        Ok(())
    }

    // ---------------------------------------------------------------- the handoff
    /// THE HANDOFF, LAST: each program's upgrade authority set to Poleis's `upgrade` PDA and read back. From here the deployer holds
    /// nothing, and the code moves only by a sealed CODEX act of root 0, so the code is handed to a seal that has acted, never to one
    /// that cannot yet (his launch plan's ceremony, Sept 9 2026: constitute the fund; ratify once the root's floor has a seat; then hand
    /// the upgrade authority to the `upgrade` PDA). Poleis's own rule: root 0 stays in genesis until its curve's window has closed (or
    /// every seat is minted) with a seat minted and it is ratified, and nothing is proposed before; a curve that mints nobody never
    /// ratifies. Handed over before that, the three programs would answer to a seal that might never act: their code, the mine and the
    /// board frozen for good. So the handoff waits for root 0 to be a council and for the seal's first act to stand (the fund, constituted
    /// by unit 1's chairs AND the minted floor); until then the deployer holds the code, and can mend it.
    pub fn handoff(&mut self) -> Res<()> {
        self.p.sendable()?;
        let mode = self.root_is_ours().map_err(|e| format!("the handoff waits for root 0: {e}"))?;
        let fund = self.account(&self.at.fund)?.and_then(|f| client::FundView::parse(&f.data)).is_some();
        let pool = self.pool_at()?;
        handoff_ready(mode, fund, self.account(&self.at.mine)?.is_some(), self.account(&self.at.board)?.is_some(), &pool, self.p.cluster)?;
        match &pool {
            crate::pool::PoolAt::Born(_) => eprintln!("  the pool is born and read back as root 0's: the handoff does not leave it unborn"),
            _ => eprintln!("  {}: a rehearsal's chain, so the handoff does not wait for a pool it cannot bear (on mainnet it waits)", crate::pool::NOT_RUN),
        }
        let mut keys: Option<Vec<Key>> = None;
        for (name, _, _, _) in PROGRAMS {
            let program = program_id(&self.ids, name);
            let pd = loader::programdata_of(&program);
            let a = self.account(&pd)?.ok_or_else(|| format!("{name}: no program data"))?;
            let State::ProgramData { authority, .. } = loader::state(&a.data)? else { return Err(format!("{name}: not a program data")) };
            if authority == Some(self.at.upgrade) {
                self.rec.resumed(format!("{name}: its upgrade authority is the polis's seal already"))?;
                continue;
            }
            if authority != Some(self.deployer) {
                return Err(format!("{name}: its upgrade authority is {authority:?}, neither the deployer nor the seal"));
            }
            if keys.is_none() {
                keys = Some(self.keys.load(&["deployer"], "the handoff")?);
            }
            let k = &keys.as_ref().unwrap()[0];
            let l = self.send("handoff", &format!("{name}: its upgrade authority handed to Poleis's upgrade PDA {}", self.at.upgrade), vec![loader::set_authority(&pd, &self.deployer, Some(&self.at.upgrade))], &[k])?;
            let a = self.account(&pd)?.ok_or_else(|| format!("{name}: no program data"))?;
            if !matches!(loader::state(&a.data)?, State::ProgramData { authority: Some(x), .. } if x == self.at.upgrade) {
                return Err(format!("{name}: read back another authority after the handoff"));
            }
            self.rec.v["handoff"][name] = json!({ "signature": l.signature, "slot": l.slot, "authority": self.at.upgrade.to_string() });
            self.rec.save()?;
        }
        let left = self.rpc.balance(&self.deployer.to_string())?;
        self.rec.v["after"] = json!({ "deployer": left });
        self.rec.save()?;
        println!("THE HANDOFF: the market, the $WARM program and Poleis answer to Poleis's upgrade PDA {} alone; only a sealed CODEX act of root 0 moves them (the code of any of the three by CODEX_UPGRADE_PROGRAM, Poleis's also by CODEX_UPGRADE; the authority of any of them by CODEX_UPGRADE_AUTHORITY; each act names its program, and an act voted for one is refused on another's).", self.at.upgrade);
        println!("The deployer {} holds no authority over anything: root 0 is a council, its authority the PDA nobody holds. It holds {} SOL: move what you keep, then delete its key file yourself:", self.deployer, sol(left));
        match &self.keys {
            KeySource::Files { dir, .. } => {
                println!("  {}", dir.join("deployer.json").display());
                println!("The three program keys fixed the ids and are needed no more (offline): {}", ["poleis", "market", "warm"].iter().map(|n| dir.join(format!("{n}.json")).display().to_string()).collect::<Vec<_>>().join(", "));
            }
            KeySource::Seed(_) => println!("  (the rehearsal's stand-ins: in memory only, gone with this process)"),
        }
        Ok(())
    }
}

/// What a root 0 that is not this sitting's means, said after what differs.
pub const STRANGER: &str = "a stranger founded it, or the params moved since the founding. The deployer still holds Poleis's code: nothing is handed over; close the program and sit again under fresh ids (his call)";

/// Root 0 as the cluster holds it while in genesis: exactly what a sitting holds it to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Founded {
    pub authority: Pubkey,
    pub default_mint: Pubkey,
    pub charter: [u8; 32],
    pub plan_cap: u64,
    pub plan_period: u64,
    pub window: u64,
    pub rung: u64,
    pub name: Vec<u8>,
    pub desc: Vec<u8>,
    /// its twelve chairs: each holder and its kind (empty, a key, a unit)
    pub chairs: Vec<([u8; 32], u8)>,
    /// the floor's curve, as root 0's NFT genesis wrote it (None when root 0 was born by another genesis)
    pub curve: Option<nft::MintCurve>,
    /// unit 1, his posiverse, as its registry stands (None when there is none)
    pub posiverse: Option<Crew>,
}

/// A unit seated in a chair of root 0 as its registry stands: its parent, its chair there, its tree, its twelve chairs, its charter,
/// window and plan, its name, and the seats of its floor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crew {
    pub parent: u64,
    pub chair: u8,
    pub root: u64,
    pub chairs: Vec<([u8; 32], u8)>,
    pub charter: [u8; 32],
    pub window: u64,
    pub plan_cap: u64,
    pub plan_period: u64,
    pub name: Vec<u8>,
    pub desc: Vec<u8>,
    pub seats: u64,
}

/// Root 0's twelve chairs as this sitting founds them: his posiverse, unit 1, in chair `chair`, the eleven others empty.
pub fn root_chairs(chair: u8) -> Vec<([u8; 32], u8)> {
    (0..poleis_treasury::PENTAGONS).map(|k| if k == chair as usize { (client::unit_holder(POSIVERSE), HOLDER_UNIT) } else { ([0; 32], HOLDER_EMPTY) }).collect()
}

/// Unit 1's twelve chairs as the params name them: its agents' own keys in their order, the rest empty.
pub fn crew_chairs(agents: &[Pubkey]) -> Vec<([u8; 32], u8)> {
    (0..poleis_treasury::PENTAGONS).map(|k| agents.get(k).map(|a| (a.to_bytes(), HOLDER_KEY)).unwrap_or(([0; 32], HOLDER_EMPTY))).collect()
}

/// What sits in a chair, in words.
fn sits(c: &([u8; 32], u8)) -> String {
    match c.1 {
        HOLDER_EMPTY => "nobody".into(),
        HOLDER_KEY => format!("the key {}", Pubkey::new_from_array(c.0)),
        HOLDER_UNIT => format!("unit {}", poleis_client::holder_unit(&c.0)),
        k => format!("a holder of kind {k}"),
    }
}

/// A ROOT 0 IN GENESIS IS THIS SITTING'S only when its authority is the deployer (Poleis makes the payer of a founding the tree's
/// authority until it is ratified) and every value of its founding is the params': the default mint, the charter, the plan, the window,
/// the rung, the name. THE SWARM IS ITS OWN MULTISIG: no key sits in a chair of root 0 (no founder); the chair `posiverse.chair` names
/// holds his posiverse, unit 1, and the eleven others are empty; unit 1 is the params' whole: root 0 its parent and its tree, that chair
/// its chair, its chairs its agents' own keys in their order and nothing else, its charter, window, plan and name, and no floor. THE
/// WHOLE TREE DECIDES: its floor is the minted Lyssajudrons, by the params' curve: its unit root 0, its first price and its step, its
/// seats the rung's hexagons, its permit issuer, the chain its permits name (`genesis`, the sitting's own genesis hash), and its window
/// (`sent`: where the founding this sitting sent put its opening and its close; with none on record, the params' length).
pub fn founding_held(f: &Founded, p: &Params, deployer: &Pubkey, genesis: &[u8; 32], sent: Option<(u64, u64)>) -> Res<()> {
    let stranger = |what: String| format!("root 0 stands, and {what}: {STRANGER}");
    if f.authority != *deployer {
        return Err(stranger(format!("its authority is {}, not the deployer {deployer}", f.authority)));
    }
    let chair = posiverse_chair(p)?;
    let want = root_chairs(chair);
    if f.chairs != want {
        let k = (0..want.len()).find(|k| f.chairs.get(*k) != want.get(*k)).unwrap_or(0);
        let held = f.chairs.get(k).map(sits).unwrap_or("no chair".into());
        return Err(stranger(format!("its chairs are not his posiverse in chair {chair} and eleven empty: chair {k} holds {held}{}", if f.chairs.get(k).is_some_and(|c| c.1 == HOLDER_KEY) { " (a key in a chair of root 0: the reverse of the law)" } else { "" })));
    }
    let Some(c) = &f.posiverse else { return Err(stranger("its chair names unit 1, and unit 1 has no registry".into())) };
    let agents = p.agents()?;
    if c.chairs != crew_chairs(&agents) {
        let k = (0..PENTAGONS_LEN).find(|k| c.chairs.get(*k) != crew_chairs(&agents).get(*k)).unwrap_or(0);
        return Err(stranger(format!("unit 1's chairs are not its agents' own in their order: chair {k} holds {}", c.chairs.get(k).map(sits).unwrap_or("no chair".into()))));
    }
    let spec = posiverse_spec(p)?;
    let crew: [(&str, bool); 9] = [
        ("unit 1's parent", c.parent == ROOT_UNIT && c.root == ROOT_UNIT),
        ("unit 1's chair in root 0", c.chair == chair),
        ("unit 1's charter", c.charter == spec.charter),
        ("unit 1's window", c.window == spec.window),
        ("unit 1's plan's cap", c.plan_cap == spec.plan_cap),
        ("unit 1's plan's period", c.plan_period == spec.plan_period),
        ("unit 1's name", c.name == spec.name),
        ("unit 1's description", c.desc == spec.desc),
        ("unit 1's floor (it is born with none)", c.seats == 0),
    ];
    if let Some((what, _)) = crew.iter().find(|(_, same)| !same) {
        return Err(stranger(format!("{what} is not the params'")));
    }
    let charter: [u8; 32] = wire::unhex(&p.text("genesis.charter")?)?;
    let rung = p.u64("genesis.rung")?;
    let held: [(&str, bool); 8] = [
        ("its default mint", f.default_mint == p.key("genesis.defaultMint")?),
        ("its charter", f.charter == charter),
        ("its plan's cap", f.plan_cap == p.u64("genesis.planCap")?),
        ("its plan's period", f.plan_period == p.u64("genesis.planPeriod")?),
        ("its window", f.window == p.u64("genesis.window")?),
        ("its rung", f.rung == rung),
        ("its name", f.name == p.text("genesis.name")?.into_bytes()),
        ("its description", f.desc == p.text("genesis.desc")?.into_bytes()),
    ];
    if let Some((what, _)) = held.iter().find(|(_, same)| !same) {
        return Err(stranger(format!("{what} is not the params'")));
    }
    let Some(curve) = &f.curve else { return Err(stranger("it has no curve: its floor is not the minted Lyssajudrons (a root born by another genesis)".into())) };
    let terms: [(&str, bool); 6] = [
        ("its curve's unit", curve.unit == ROOT_UNIT),
        ("its curve's first price", curve.start_lamports == p.u64("floor.startLamports")?),
        ("its curve's step", curve.step_lamports == p.u64("floor.stepLamports")?),
        ("its curve's seats (the rung's hexagons)", Some(curve.capacity) == rung_hexagons(rung)),
        ("its curve's permit issuer", curve.completion_issuer == p.key("floor.issuer")?.to_bytes()),
        ("its curve's chain (the genesis hash its permits name)", curve.genesis_hash == *genesis),
    ];
    if let Some((what, _)) = terms.iter().find(|(_, same)| !same) {
        return Err(stranger(format!("{what} is not the params'")));
    }
    match sent {
        Some((o, c)) if (o, c) != (curve.opens_slot, curve.closes_slot) => Err(stranger(format!("its curve's window opens at {} and closes at {}, the founding this sitting sent at {o} and {c}", curve.opens_slot, curve.closes_slot))),
        None if curve.closes_slot.saturating_sub(curve.opens_slot) != p.u64("floor.windowSlots")? => Err(stranger("its curve's window is not the params' length".into())),
        _ => Ok(()),
    }
}

/// The hexagons of a rung of the ladder, None for a rung that is none.
pub fn rung_hexagons(rung: u64) -> Option<u64> {
    (rung > 0 && (rung as usize) < poleis_treasury::LADDER.len()).then(|| client::ladder_hexagons(rung as usize))
}
const PENTAGONS_LEN: usize = poleis_treasury::PENTAGONS;

/// His posiverse as the params name it: unit 1 in the chair `posiverse.chair` of root 0, its chairs its agents, its charter, window and plan its own.
pub fn posiverse_spec(p: &Params) -> Res<FoundSpec> {
    Ok(FoundSpec {
        child: POSIVERSE,
        chair: posiverse_chair(p)?,
        charter: wire::unhex(&p.text("posiverse.charter")?).map_err(|e| format!("params posiverse.charter: {e}"))?,
        window: p.u64("posiverse.window")?,
        plan_cap: p.u64("posiverse.planCap")?,
        plan_period: p.u64("posiverse.planPeriod")?,
        founders: p.agents()?,
        name: p.text("posiverse.name")?.into_bytes(),
        desc: p.at("posiverse.desc").as_str().unwrap_or_default().as_bytes().to_vec(),
    })
}

/// THE FLOOR IS THE MINTED LYSSAJUDRONS: the curve the params name, its window counted from `now` (the founding's slot) and its permits
/// bound to `genesis`, the chain's own genesis hash; its seats the rung's hexagons, nothing minted yet.
pub fn curve_of(p: &Params, now: u64, genesis: &[u8; 32]) -> Res<nft::MintCurve> {
    let rung = p.u64("genesis.rung")?;
    let capacity = rung_hexagons(rung).ok_or_else(|| format!("params genesis.rung: {rung}; the ladder's rungs are 1 to {}", poleis_treasury::LADDER.len() - 1))?;
    let opens = now.checked_add(p.u64("floor.opensAfterSlots")?).ok_or("params floor.opensAfterSlots: past the clock's end")?;
    let closes = opens.checked_add(p.u64("floor.windowSlots")?).ok_or("params floor.windowSlots: past the clock's end")?;
    Ok(nft::MintCurve {
        bump: 0,
        unit: ROOT_UNIT,
        start_lamports: p.u64("floor.startLamports")?,
        step_lamports: p.u64("floor.stepLamports")?,
        minted_count: 0,
        capacity,
        opens_slot: opens,
        closes_slot: closes,
        completion_issuer: p.key("floor.issuer")?.to_bytes(),
        genesis_hash: *genesis,
    })
}

/// Whether a transaction's instructions already ask a compute budget (a cap of its step's own): then no other is added, for a
/// transaction with two is refused.
pub fn asks_budget(ixs: &[Instruction]) -> bool {
    ixs.iter().any(|ix| ix.program_id == loader::COMPUTE_BUDGET)
}

/// The compute budget a transaction asks when the params name a priority fee.
pub fn budget_of(p: &Params, units: u32) -> Res<Vec<Instruction>> {
    let price = p.u64_or("priorityMicroLamports", 0)?;
    Ok(if price == 0 { Vec::new() } else { vec![loader::compute_limit(units), loader::compute_price(price)] })
}

/// The unit limit the press asks for a transaction of `units`: a fifth more and a thousand, never past the cluster's 1,400,000.
pub fn sized_limit(units: u64) -> u64 {
    (units + units / 5 + 1_000).min(1_400_000)
}

/// The priority fee, in lamports, the cluster takes for a unit limit at a price in micro-lamports a unit.
pub fn priority_lamports(price: u64, limit: u64) -> u64 {
    (price * limit).div_ceil(1_000_000)
}

/// THE FOUNDING, in one transaction the deployer pays: init_config, Poleis's NFT genesis with no founder (root 0 is born with no key in
/// its chairs, and its floor is the minted Lyssajudrons: the params' curve, its window counted from `now`, its permits bound to `genesis`),
/// and found_child of his posiverse, unit 1, in its chair (in genesis the founding's payer may hang a unit on an empty chair). The curve's
/// window opens and closes where the second value says.
pub fn founding_ixs(p: &Params, deployer: &Pubkey, now: u64, genesis: &[u8; 32]) -> Res<(Vec<Instruction>, (u64, u64))> {
    let c = curve_of(p, now, genesis)?;
    let (name, desc) = (p.text("genesis.name")?, p.text("genesis.desc")?);
    let open = nft::open_genesis(
        *deployer,
        &nft::Genesis {
            root: ROOT_UNIT,
            charter: wire::unhex(&p.text("genesis.charter")?)?,
            plan_cap: p.u64("genesis.planCap")?,
            plan_period: p.u64("genesis.planPeriod")?,
            governance_window: p.u64("genesis.window")?,
            rung: u8::try_from(p.u64("genesis.rung")?).map_err(|_| "params genesis.rung: a rung of the ladder")?,
            start_lamports: c.start_lamports,
            step_lamports: c.step_lamports,
            opens_slot: c.opens_slot,
            closes_slot: c.closes_slot,
            completion_issuer: Pubkey::new_from_array(c.completion_issuer),
            chain_genesis_hash: c.genesis_hash,
            founders: &[],
            name: &name,
            description: &desc,
        },
    )
    .map_err(|e| format!("the founding's NFT genesis is out of Poleis's bounds ({e:?}): its rung, window, names or curve"))?;
    let ixs = [
        budget_of(p, 90_000)?,
        vec![client::init_config_of(ROOT_UNIT, *deployer, p.key("genesis.defaultMint")?), open, client::found_child_of(ROOT_UNIT, *deployer, ROOT_UNIT, &posiverse_spec(p)?)],
    ]
    .concat();
    Ok((ixs, (c.opens_slot, c.closes_slot)))
}

/// THE TREASURY IS THE SWARM'S OWN: the $WARM mine's treasury is root 0's vault of the dollar, derived, never a param; a params file
/// that still names one is refused.
pub fn treasury_of(p: &Params) -> Res<Pubkey> {
    if !p.at("warm.treasury").is_null() {
        return Err(TREASURY_NAMED.into());
    }
    Ok(client::vault_pda(ROOT_UNIT, &p.key("warm.dollar")?))
}

/// THE SWARM JUDGES: the market's judge is root 0's own hand, derived, never a param; a params file that still names one is refused.
pub fn judge_of(p: &Params, at: &Addresses) -> Res<Pubkey> {
    if !p.at("market.judge").is_null() {
        return Err(JUDGE_NAMED.into());
    }
    Ok(at.agent)
}

/// A token account standing at root 0's vault is root 0's vault of the dollar: the token program's, of that mint, its authority root 0's
/// treasury (only a sealed TREASURY act moves it).
pub fn vault_held(owner: &str, data: &[u8], dollar: &Pubkey) -> Res<()> {
    if owner != TOKEN.to_string() || data.len() != warm_mine::token::ACCOUNT_LEN || data[..32] != dollar.to_bytes() {
        return Err(format!("the account at root 0's vault is not a token account of {dollar}"));
    }
    if data[32..64] != client::treasury_pda(ROOT_UNIT).to_bytes() {
        return Err(format!("root 0's vault answers to {}, not to root 0's treasury", Pubkey::new_from_array(data[32..64].try_into().unwrap())));
    }
    Ok(())
}

/// THE HANDOFF IS LAST: the code goes to the seal once the seal has acted. Root 0 a council (Poleis ratifies it only once a seat of its
/// curve is minted and the window has closed or sold out, and nothing is proposed before), the fund constituted (the seal's first act,
/// voted by unit 1's chairs and the minted floor), the mine opened (its `init` needs the deployer as the program's upgrade authority), the board, and THE POOL BORN (act 5: its
/// account and its list are made in its execute's own transaction by the key that executes, the deployer, whose key file the handoff's last
/// words tell him to delete; a pool not born before then is born never). Only on a rehearsal's chain with no stake pool program is the pool
/// not waited for (it cannot be born there); on mainnet the program stands and the pool is born first, with no way around it.
pub fn handoff_ready(mode: u8, fund: bool, mine: bool, board: bool, pool: &crate::pool::PoolAt, cluster: Cluster) -> Res<()> {
    use crate::pool::PoolAt;
    if mode != poleis_treasury::MODE_COUNCIL {
        return Err("the handoff waits: root 0 is still in genesis until its curve's window has closed (or every seat is minted) with a seat minted, and it is ratified (ratify_nft_genesis, by anyone: press ratify). A seal that cannot act yet is handed no code; the deployer holds it until then".into());
    }
    if !fund {
        return Err("the handoff waits for the seal's first act: the fund is not constituted (act 1 of the press's acts), so no act of unit 1's chairs AND the minted floor has passed on this chain yet. A seal that has not acted is handed no code; the deployer holds it until then".into());
    }
    if !mine {
        return Err("the handoff waits for the $WARM mine: its init needs the deployer as the program's upgrade authority".into());
    }
    if !board {
        return Err("the handoff waits for the market's board".into());
    }
    match pool {
        PoolAt::Born(_) => Ok(()),
        PoolAt::Unborn => Err("the handoff waits for the pool's birth (act 5 of the press's acts, passed by the seal and carried out by press crank): its account and its list are made in its execute's own transaction by the deployer's key, which the handoff's last words tell him to delete, so the pool is born before it; the deployer holds the code until then".into()),
        PoolAt::NoProgram if cluster == Cluster::Mainnet => Err(format!("the handoff waits for the pool's birth: {}", crate::pool::NOT_MAINNET)),
        PoolAt::NoProgram => Ok(()),
    }
}

/// HIS FINGER'S ASKER, for a sitting that reads key files. On mainnet it is the house's own and no other: the file at
/// tools/embertide-mesh/touchid/touchid of the house the sitting was built from, its bytes the ones that house's commit holds (a
/// swapped asker answers nothing, and an asker named by the environment is not heard). Off mainnet, PRESS_TOUCHID may name another.
pub fn asker(cluster: Cluster, house: &Path, named: Option<std::ffi::OsString>) -> Res<PathBuf> {
    let own = house.join("tools/embertide-mesh/touchid/touchid");
    if cluster != Cluster::Mainnet {
        return Ok(named.map(PathBuf::from).unwrap_or(own));
    }
    if !own.is_file() {
        return Err(format!("his finger's asker is not at {}: on mainnet no key is read without it", own.display()));
    }
    let git = |args: &[&str]| Command::new("git").arg("-C").arg(house).args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let committed = git(&["rev-parse", "HEAD:tools/embertide-mesh/touchid/touchid"]);
    let standing = git(&["hash-object", &own.display().to_string()]);
    if committed.is_none() || committed != standing {
        return Err(format!("his finger's asker at {} is not the house's committed one: nothing read", own.display()));
    }
    Ok(own)
}

/// Every value of the params a sitting sends, read and held to the programs' own bounds, with no chain asked. A founding the program
/// refused the slot Poleis became callable would leave root 0 open to the first wallet while the params were mended, so a value Poleis
/// would refuse stops the sitting before anything is deployed. The fund's and the market's numbers are read for their shape (the acts
/// are built whole); their own bounds are the programs' to judge when an act executes, and a refused act is proposed again.
pub fn params_checked(p: &Params, ids: &Ids, at: &Addresses) -> Res<()> {
    use poleis_treasury::{MAX_DESC, MAX_NAME, MAX_WINDOW_SLOTS, MIN_WINDOW_SLOTS};
    p.sendable()?;
    // his posiverse's agents: one to twelve, each distinct (no key of his is a param: Params::from_text refuses `keystones`)
    p.agents()?;
    wire::unhex::<32>(&p.text("genesis.charter")?).map_err(|e| format!("params genesis.charter: {e}"))?;
    for n in ["genesis.planCap", "genesis.planPeriod"] {
        p.u64(n)?;
    }
    let window = p.u64("genesis.window")?;
    if !(MIN_WINDOW_SLOTS..=MAX_WINDOW_SLOTS).contains(&window) {
        return Err(format!("params genesis.window: {window} slots; Poleis takes {MIN_WINDOW_SLOTS} to {MAX_WINDOW_SLOTS}"));
    }
    let rung = p.u64("genesis.rung")?;
    let seats = rung_hexagons(rung).ok_or_else(|| format!("params genesis.rung: {rung}; the ladder's rungs are 1 to {}", poleis_treasury::LADDER.len() - 1))?;
    // THE FLOOR IS THE MINTED LYSSAJUDRONS: the curve's prices, its window and its issuer within Poleis's bounds
    let (start, step) = (p.u64("floor.startLamports")?, p.u64("floor.stepLamports")?);
    if start == 0 || step == 0 {
        return Err(format!("params floor.startLamports {start} and floor.stepLamports {step}: each more than nothing (Poleis's curve prices the first seat at its start and rises a step a seat)"));
    }
    if (seats - 1).checked_mul(step).and_then(|x| x.checked_add(start)).is_none() {
        return Err(format!("params floor: the price of the curve's last seat ({start} + {} x {step}) is past what a lamport count holds", seats - 1));
    }
    let (after, length) = (p.u64("floor.opensAfterSlots")?, p.u64("floor.windowSlots")?);
    // the founding is signed before Poleis's deploy and lands within a blockhash's life (150 slots); Poleis refuses a curve that opens
    // before the founding lands, so the window opens after it
    if after <= 150 || after > MAX_WINDOW_SLOTS || length == 0 || length > MAX_WINDOW_SLOTS {
        return Err(format!("params floor.opensAfterSlots {after} and floor.windowSlots {length}: the window opens 151 to {MAX_WINDOW_SLOTS} slots after the founding's slot (the founding lands within a blockhash's life, 150 slots, and Poleis refuses a curve that opens before it lands) and runs 1 to {MAX_WINDOW_SLOTS} slots"));
    }
    p.key("floor.issuer")?;
    let (name, desc) = (p.text("genesis.name")?, p.text("genesis.desc")?);
    if name.len() > MAX_NAME || desc.len() > MAX_DESC {
        return Err(format!("params genesis.name is {} bytes (at most {MAX_NAME}), genesis.desc {} (at most {MAX_DESC})", name.len(), desc.len()));
    }
    p.key("genesis.defaultMint")?;
    // his posiverse: its charter, window, plan and name within Poleis's bounds
    let crew = posiverse_spec(p)?;
    if !(MIN_WINDOW_SLOTS..=MAX_WINDOW_SLOTS).contains(&crew.window) {
        return Err(format!("params posiverse.window: {} slots; Poleis takes {MIN_WINDOW_SLOTS} to {MAX_WINDOW_SLOTS}", crew.window));
    }
    if crew.name.len() > MAX_NAME || crew.desc.len() > MAX_DESC {
        return Err(format!("params posiverse.name is {} bytes (at most {MAX_NAME}), posiverse.desc {} (at most {MAX_DESC})", crew.name.len(), crew.desc.len()));
    }
    // the founding is one transaction: it fits a packet with these params (its size does not hang on the keys' values, the slot or the
    // genesis hash: every one of them is fixed width)
    let (ixs, _) = founding_ixs(p, &crate::params::placeholder_address("the founding's payer"), 0, &[1; 32])?;
    let (m, signers) = wire::message(&ixs, &crate::params::placeholder_address("the founding's payer"), &[0; 32]);
    let bytes = 1 + 64 * signers.len() + m.len();
    if bytes > PACKET {
        return Err(format!("the founding with these params is {bytes} bytes, past the {PACKET} one transaction holds: fewer agents or shorter names"));
    }
    // the mine: its treasury root 0's own vault (derived), its dollar, its opening, its face within Metaplex's bounds; its miner, the door's crank
    // key the sixth act names (THE SWARM NAMES ITS MINER: press miner makes it)
    treasury_of(p)?;
    p.key("warm.dollar")?;
    p.key("warm.miner")?;
    // THE TREASURY PAYS THE COMPUTE: the seventh act names root 0's cap of the dollar, the eighth the router's month of it to the miner; both
    // more than nothing, the month within the cap, and the dollar the tree's default mint (the one mint whose cap is the registry's own; any
    // other would need a CODEX_ALLOW_MINT listing first, which the launch does not write)
    let (dollar_cap, month_cap) = (p.u64("genesis.dollarCap")?, p.u64("router.monthCap")?);
    if dollar_cap == 0 || month_cap == 0 {
        return Err(format!("params genesis.dollarCap {dollar_cap} and router.monthCap {month_cap}: each more than nothing (micro-dollars a period: a cap of 0 lets no dollar leave root 0's vault, and an allowance of 0 the chain refuses)"));
    }
    if month_cap > dollar_cap {
        return Err(format!("params router.monthCap {month_cap} is past genesis.dollarCap {dollar_cap}: the router's month could never be drawn whole, every pull is charged to root 0's cap of the dollar"));
    }
    if p.key("warm.dollar")? != p.key("genesis.defaultMint")? {
        return Err(format!(
            "params warm.dollar {} is not genesis.defaultMint {}: the router's month is paid in the dollar the mine counts deposits in, and only the tree's default mint has its cap in root 0's registry (another mint needs a CODEX_ALLOW_MINT listing, which the launch does not write): name the same mint in both",
            p.at("warm.dollar"),
            p.at("genesis.defaultMint")
        ));
    }
    if p.at("warm.opening").as_str() != Some("the founding") {
        crate::params::ms_of(p.at("warm.opening")).map_err(|e| format!("params warm.opening: {e}"))?;
    }
    let (name, symbol, uri) = (p.text("warm.name")?, p.text("warm.symbol")?, p.text("warm.uri")?);
    if name.is_empty() || name.len() > warm_mine::face::MAX_NAME || symbol.is_empty() || symbol.len() > warm_mine::face::MAX_SYMBOL || uri.len() > warm_mine::face::MAX_URI {
        return Err("params warm: the face is out of Metaplex's bounds (a name of 1 to 32 bytes, a symbol of 1 to 10, a URI of at most 200)".into());
    }
    // THE LAUNCH NAMES THE SEAL'S ROAD: its node (mainnet's, by its host alone), and no seal.proposers (any member proposes), held before anything
    // starts, so the road the founding writes never stops on a value the params could have said
    crate::seal::checked(p)?;
    // the board, its roles' addresses, and the eight acts built whole
    p.u64("market.board")?;
    p.key("market.crediter")?;
    // THE SWARM JUDGES: the judge is root 0's own hand, so a road's receipts are a sealed act of the whole tree, and its judging window holds
    // a whole vote of root 0 and of unit 1 and its execution: a window no longer than a vote may close before the swarm has spoken
    judge_of(p, at)?;
    let (judging, vote) = (p.u64("market.judgeSlots")?, window.max(crew.window));
    if judging <= vote {
        return Err(format!("params market.judgeSlots {judging}: the swarm judges a road by its own vote, which can last {vote} slots (genesis.window, posiverse.window), so a road's judging window is longer than a vote, room for it and its execution"));
    }
    // the pool: its tag and its list's length within the stake pool's bounds, its program's rehearsed hash (named on mainnet), and act 5's
    // execute with its two makings one transaction, with nothing at their addresses and with a stranger's lamports there (the larger form;
    // its size does not hang on the keys' values)
    let deployer = crate::params::placeholder_address("the deployer");
    crate::pool::Pool::of(p, &deployer, &ids.poleis)?;
    crate::pool::named_sha(p)?;
    let acts = crate::acts::acts(p, ids, at, &deployer)?;
    for act in acts.iter().filter(|a| !a.before.is_empty()) {
        for dust in [false, true] {
            let bytes = crate::acts::execute_bytes(p, act, &deployer, dust)?;
            if bytes > PACKET {
                return Err(format!("act {}'s execute with its makings is {bytes} bytes with these params, past the {PACKET} one transaction holds", act.n));
            }
        }
    }
    // THE WHOLE TREE DECIDES: each act is proposed by a seat owner's own transaction (propose_nft, its seat proved by its token): it fits
    // one transaction, the owner its one signer
    let owner = crate::floor::placeholder_owner();
    for act in &acts {
        let bytes = crate::floor::proposal_bytes(p, act, &owner)?;
        if bytes > PACKET {
            return Err(format!("act {}'s proposal by a seat owner is {bytes} bytes with these params, past the {PACKET} one transaction holds", act.n));
        }
    }
    Ok(())
}

/// The last lines of a transaction's logs.
pub fn tail(logs: &Value) -> String {
    let words: Vec<&str> = logs.as_array().map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    words[words.len().saturating_sub(8)..].join("\n      ")
}

pub fn sol(lamports: u64) -> String {
    format!("{}.{:09}", lamports / 1_000_000_000, lamports % 1_000_000_000)
}

pub fn say_need(lines: &[(String, u64)], need: u64, balance: u64, deployer: &Pubkey) {
    println!("THE NEED, at the chain's own rent:");
    for (what, l) in lines {
        println!("  {:>16} SOL  {what}", sol(*l));
    }
    println!("  {:>16} SOL  the sitting needs from here, at its peak", sol(need));
    println!("  {:>16} SOL  the deployer {deployer} holds", sol(balance));
}

pub fn now_iso() -> String {
    let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (s.div_euclid(86_400), s.rem_euclid(86_400));
    // civil_from_days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3_600, rem % 3_600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_boards_size_is_the_roads_own() {
        let src = include_str!("../../program/poleis_roads/src/state.rs");
        let at = src.find("pub struct Board {").expect("the roads' Board");
        let size = &src[at..][src[at..].find("pub const SIZE: usize = ").unwrap() + "pub const SIZE: usize = ".len()..];
        assert_eq!(size[..size.find(';').unwrap()].parse::<usize>().unwrap(), super::BOARD_SIZE);
        // and its authority where check_board reads it
        assert!(src.contains("pub const OFF_BOARD: usize = 2;") && src.contains("pub const OFF_AUTHORITY: usize = 10;"));
    }
}
