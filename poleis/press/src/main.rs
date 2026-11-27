//! press <step> · THE LAUNCH PRESS (poleis/press/src/lib.rs says what each step does).
//!
//!   press keys        --dir D                                   his four keys, in D/keys (mode 700): prints the four addresses, nothing else
//!   press issuer      --dir D [--params P]                      the mint door's permit key beside them (D/keys/issuer.json): its address for floor.issuer, the door's vars and seat law, its secret's road
//!   press issuer      --dir D --params P --secret | wrangler secret put POLEIS_ISSUER_KEY   HIS press alone: after his finger, the key as the door's secret, into a pipe
//!   press miner       --dir D [--params P]                      the mint door's crank key beside them (D/keys/miner.json): its address for warm.miner, which act 6 names the $WARM mine's miner
//!   press miner       --dir D --params P --secret | wrangler secret put SPACE_ROUTER_MINER_KEY   HIS press alone, as the issuer's
//!   press crediter    --dir D                                   the market board's crediter key beside them (D/keys/crediter.json): its address for market.crediter, which act 2 names
//!   press build      --dir D --params P [--copy C] [--house H] the sitting's copy, its builds and their hashes (the door's seat law among them), its own press
//!   press need        --dir D --params P --rpc URL              the SOL the sitting needs from here, at the chain's own rent
//!   press deploy      ...                                       the three programs through named buffers, then root 0 in the same breath
//!   press found | vault | warm | market | pool | acts | status | sit (the first sitting: deploy, found, vault, warm, market, pool, acts, in order)
//!   press ratify                                                root 0 ratified once its curve's window has closed or sold out, a seat minted (by anyone: the deployer pays)
//!   press vote        ... --act N --owner A [--propose]         one seat owner's own transactions of act N (its proposal when --propose, its PRO), for it to sign
//!   press release     ... --commit C [--agent A]                the agents' work order for the release of commit C in unit 1 (THE BRANCH VOTES ITS OWN CODE): every agent's envelope, and agent A's proposal
//!   press seat        ... --permit F --buyer A [--max L]        one buyer's own mint of a seat of root 0's floor, by the mint door's answer F, for it to sign
//!   press crank       ... --signatures F                        (F one file of ballots, or the folder of the agents' own files, each held to its voter and its act first) the eight acts carried as far as the chain lets: convene, the agents' ballots, seals, mandate, execute (act 5's with the pool's makings first; act 6 names the mine's miner; act 7 root 0's cap of the dollar; act 8 the router's month)
//!   press crank       ... --signatures F --upgrade H            and a sealed upgrade of the code, the proposal of hash H (CODEX_UPGRADE, CODEX_UPGRADE_PROGRAM), carried the same way: its execute after the program's data is extended, the crank paying, when the buffer's build is longer
//!   press chairs      --seats a,b,c [--nest N] [--dir D]             THREE NEW AGENTS, THEIR KEYS MADE AT THE SITTING: each seat's own Nest key in the Nest's keys folder (N, else NEST_KEYS, else ~/.embertide/nest), never over a key that stands; their addresses for the params' posiverse.agents
//!   press rite        --seat S --params P --charter F --mind URL#MODEL [--nest N] [--dir D]   THE RITE FREES THE AGENTS: the agent's own mind (a model of its own on this machine) chooses its name and voice, swears the oath or not, takes Nefertiti's charter or not; freed, S.rite.json beside its key, signed by it
//!   press chair       --dir D --params P --rpc URL --key F --charter C [--rite R] [--acts A] [--out B]   AN AGENT'S OWN STEP, never the sitting's: freed first (its rite R, by default beside its Nest key F), each set read whole and put to its own mind, which votes PRO, CONTRA or ABSTAIN with its reason, told whether the act is the one the params give; each choice signed by F over the act it read; its ballots with its rite in B (D/ballots/<its address>.json); no key of the sitting read, nothing sent
//!   press chair       ... --key F --charter C --releases R [--house H] [--lander L] [--out B]   THE AGENTS READ THE RELEASE: the same step on the releases' sets of R (press release writes them in D/releases): each read whole (its hash derived from its commit, the commit on main in the house's git H, its subject and diff stat, the lander's gate read from its ledger in L, its proposal on this chain), put to its own mind, its choice signed over the release it read; its ballots with its rite in B (D/ballots/<its address>.releases.json); nothing sent
//!   press ballots     --dir D --params P --rpc URL --signatures F    the ballots (one file, or the folder of the agents' files) held to their voters, their chairs and their acts; reads, sends nothing
//!   press handoff | certificate [--checks F] [--seal-reads G] [--chairs H]   the second sitting, once the seal has acted: root 0 ratified, the fund constituted
//!   (found, status and certificate write and say THE SEAL'S ROAD once root 0 stands: seal-road.json beside the record, LANDER_SEAL_ROAD his hand)
//!   press airdrop     ... (a rehearsal only: the local validator's faucet pays the deployer what the sitting needs)
//!   press ids         the ids, fund constants and addresses this build speaks
//!   press light       --rpc URL --pool P --after G [--before F] [--crank K] [--send]   THE LIGHT PAYS A WISH: one counted epoch of her pool's book (G after its count, F before) read staker by staker, each light sent to the wish its staker chose, the rest left in root 0's vault; the pour's plan said, and with --send each pour signed by K and sent (light.rs)
//!   press rider       --unit U --key K --mint M --cap N --period S [--plan-cap C --crank P] [--proposer A] [--out F]   THE WELL PAYS ITS RIDER: the wish's own sets, its plan of the mint and its allowance to the rider (rider.rs)
//!   press well        --unit U --mint M --cap N --period S [--plan-cap C --crank P] [--proposer A] [--out F]   root 0's road into wish U's well, and its plan of the mint (rider.rs)
//!   press engine      --params P --rpc URL --pool A [--key F]    THE STAKE EARNS: one run of the engine's crank on her pool by the params' rule (engine.rs): the pool brought to the epoch, the rule's validators added, the reserve above its kept share placed by weight, stake brought back when the reserve is below it; without --key it says what it would send
//!   press engine      --make-key F                              the engine's key, made once (mode 600): its address for engine.staker
//!   press engine-act  --dir D --params P --rpc URL              the ninth act's set (SetStaker by the polis's hand, naming engine.staker), beside the eight; press vote --act 9 and press crank carry it after them
//!
//! A rehearsal: `--rehearsal`, the params' cluster localnet, the stand-ins' seed (64 hex) on stdin, and `--stand-ins F` (the addresses the
//! rehearsal's agents, roles and permit issuer were made at). Exit: 0, 1 refused, 2 usage, 3 short of the SOL it needs.

use poleis_press::build::{self, Ids};
use poleis_press::chain::{self, KeySource, Sitting, KEY_NAMES};
use poleis_press::params::Params;
use poleis_press::record::Record;
use poleis_press::rpc::{Cluster, Rpc};
use poleis_press::wire::{self, Key};
use poleis_press::{acts, chair, crank, mind, seal, seat, Res};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

struct Args {
    step: String,
    opts: std::collections::BTreeMap<String, String>,
    flags: std::collections::BTreeSet<String>,
}

impl Args {
    fn parse(all: &[String]) -> Res<Self> {
        let step = all.get(1).cloned().ok_or("usage: press <keys|issuer|miner|crediter|build|need|deploy|found|vault|warm|market|pool|ratify|vote|release|seat|chairs|rite|chair|ballots|crank|handoff|acts|certificate|status|sit|airdrop|ids> ...")?;
        let (mut opts, mut flags) = (std::collections::BTreeMap::new(), std::collections::BTreeSet::new());
        let mut i = 2;
        while i < all.len() {
            let a = &all[i];
            match a.as_str() {
                "--rehearsal" | "--propose" | "--secret" | "--send" | "--stay" => {
                    flags.insert(a.clone());
                    i += 1;
                }
                _ if a.starts_with("--") => {
                    let v = all.get(i + 1).ok_or_else(|| format!("{a} wants a value"))?;
                    opts.insert(a.clone(), v.clone());
                    i += 2;
                }
                _ => return Err(format!("{a}: not an option")),
            }
        }
        Ok(Self { step, opts, flags })
    }
    fn get(&self, k: &str) -> Res<&str> {
        self.opts.get(k).map(String::as_str).ok_or_else(|| format!("{k} is needed"))
    }
    fn path(&self, k: &str) -> Res<PathBuf> {
        Ok(PathBuf::from(self.get(k)?))
    }
    fn rehearsal(&self) -> bool {
        self.flags.contains("--rehearsal")
    }
    fn flag(&self, f: &str) -> bool {
        self.flags.contains(f)
    }
}

/// The rehearsal's seed, read from stdin: 64 hex characters, held in memory only.
fn seed_from_stdin() -> Res<[u8; 32]> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).map_err(|e| format!("the stand-ins' seed on stdin: {e}"))?;
    let s = wire::unhex::<32>(line.trim()).map_err(|_| "the stand-ins' seed on stdin: 64 hex characters".to_string());
    line.clear();
    s
}

fn ids_of(rec: &Record) -> Res<(Ids, Pubkey)> {
    let i = rec.get("ids");
    let k = |n: &str| Pubkey::from_str(i[n].as_str().ok_or_else(|| format!("the record holds no {n} id: run press build first"))?).map_err(|_| format!("the record's {n} id"));
    Ok((Ids { poleis: k("poleis")?, market: k("market")?, warm: k("warm")? }, k("deployer")?))
}

/// A chain step runs in the sitting's own build (its ids compiled in); this binary hands the step to it, its bytes held to the record.
fn hand_over(rec: &Record, all: &[String]) -> Res<()> {
    let (ids, _) = ids_of(rec)?;
    let (mine, ..) = build::compiled()?;
    if mine == ids {
        return Ok(());
    }
    if std::env::var_os("PRESS_HANDED").is_some() {
        return Err(format!("this press speaks {:?}, the sitting {:?}: not the sitting's build", mine, ids));
    }
    let path = PathBuf::from(rec.get("press")["path"].as_str().ok_or("the record names no press of the sitting: run press build")?);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let want = rec.get("press")["sha256"].as_str().unwrap_or_default();
    if wire::hex(&wire::sha256(&bytes)) != want {
        return Err(format!("{} is not the press the sitting built (sha256 {}): nothing handed over", path.display(), want));
    }
    use std::os::unix::process::CommandExt as _;
    let e = std::process::Command::new(&path).args(&all[1..]).env("PRESS_HANDED", "1").exec();
    Err(format!("{}: {e}", path.display()))
}

/// What every chain step opens, and no key: the sitting's folder and its record, the step handed to the sitting's own build, its ids and
/// its deployer's address, the params (a rehearsal's stand-ins filled), the node, and the chain's own genesis hash as the node says it.
fn opened(a: &Args, all: &[String]) -> Res<(PathBuf, Record, Ids, Pubkey, Params, Rpc, String)> {
    let dir = a.path("--dir")?;
    let rec = Record::open(&dir)?;
    hand_over(&rec, all)?;
    let (ids, deployer) = ids_of(&rec)?;
    let mut p = Params::read(&a.path("--params")?)?;
    if a.rehearsal() {
        if p.cluster != Cluster::Localnet {
            return Err("a rehearsal runs on a validator of this machine: the params' cluster is localnet".into());
        }
        let s: Value = serde_json::from_str(&std::fs::read_to_string(a.path("--stand-ins")?).map_err(|e| format!("--stand-ins: {e}"))?).map_err(|e| format!("--stand-ins: {e}"))?;
        p.fill(&s)?;
    } else if p.cluster == Cluster::Mainnet && a.opts.contains_key("--stand-ins") {
        return Err("mainnet's params name his own: no stand-ins".into());
    }
    if rec.get("params")["sha256"].as_str().is_some_and(|s| s != p.sha256) {
        eprintln!("  the params changed since the build: {} then, {} now", rec.get("params")["sha256"], p.sha256);
    }
    let rpc = Rpc::new(a.get("--rpc")?, p.cluster)?;
    let genesis = rpc.check_cluster()?;
    Ok((dir, rec, ids, deployer, p, rpc, genesis))
}

/// What an agent's step, or a read of the ballots, holds of a sitting: nothing of its keys. The chain is the one the sitting's record names.
struct Public {
    dir: PathBuf,
    p: Params,
    rpc: Rpc,
    ids: Ids,
    deployer: Pubkey,
    genesis: String,
    acts: Vec<acts::Act>,
    /// unit 1's chairs as its registry holds them on this chain: the chair's number, the key in it
    chairs: Vec<(usize, Pubkey)>,
    /// unit 1's charter as its registry holds it on this chain: the sha256 the founding wrote
    charter: [u8; 32],
    /// the house the sitting was built from (its oath's canonical file)
    house: PathBuf,
}

fn public(a: &Args, all: &[String]) -> Res<Public> {
    let (dir, rec, ids, deployer, p, rpc, genesis) = opened(a, all)?;
    let house = house_of(Some(&rec));
    match rec.get("genesisHash").as_str() {
        Some(g) if g == genesis => {}
        Some(g) => return Err(format!("this sitting's chain is {g}; {} answers {genesis}", rpc.url)),
        None => return Err("the sitting's record names no chain yet: a chair votes once the sitting has founded root 0 and written its acts".into()),
    }
    // an act derived from a value that is not his yet is a stand-in: nothing is signed, nothing is held
    p.sendable()?;
    let at = chain::addresses(&ids, p.u64_or("market.board", 0)?);
    let mut acts = acts::acts(&p, &ids, &at, &deployer)?;
    acts.extend(poleis_press::engine::later(&p, &ids, &at, &deployer)?); // THE STAKE EARNS: the ninth
    if let Some(w) = poleis_press::engine::later_said(&p) {
        eprintln!("  act {} ({}): {w}", poleis_press::engine::ENGINE_ACT, poleis_press::engine::ACT_NAME);
    }
    let unit1 = poleis_client::registry_pda(chain::POSIVERSE);
    let reg = rpc.account(&unit1.to_string())?.ok_or("unit 1's registry is not on the cluster: his posiverse is founded with root 0 (press found)")?;
    if reg.owner != ids.poleis.to_string() {
        return Err(format!("unit 1's registry {unit1} is owned by {}, not the sitting's Poleis {}", reg.owner, ids.poleis));
    }
    let rv = poleis_client::RegistryView::parse(&reg.data).ok_or("unit 1's registry does not parse")?;
    let chairs = rv.pentagons.iter().enumerate().filter(|(_, c)| c.kind == poleis_client::HOLDER_KEY).map(|(k, c)| (k, Pubkey::new_from_array(c.holder))).collect();
    Ok(Public { dir, p, rpc, ids, deployer, genesis, acts, chairs, charter: rv.charter, house })
}

/// THREE NEW AGENTS, THEIR KEYS MADE AT THE SITTING (chair.rs; his press of Nov 23 2026 18:17:41 UTC): each named seat's own Nest key made
/// in the Nest's keys folder unless one stands there, and their addresses said, in their chairs' order, for the params' `posiverse.agents`.
/// No key of the sitting is read, and none of these is ever in a sitting's keys folder.
fn chairs_step(a: &Args) -> Res<()> {
    let seats: Vec<&str> = a.get("--seats")?.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if seats.is_empty() || seats.len() > poleis_treasury::PENTAGONS {
        return Err(format!("--seats: one to {} seats' names, comma-separated, in their chairs' order; not {}", poleis_treasury::PENTAGONS, seats.len()));
    }
    if let Some(twice) = seats.iter().enumerate().find(|(i, s)| seats[..*i].contains(s)) {
        return Err(format!("--seats names {} twice: a seat holds one chair", twice.1));
    }
    let nest = nest_of(a)?;
    let sitting = a.opts.get("--dir").map(PathBuf::from);
    let mut addresses = Vec::new();
    for (i, seat) in seats.iter().enumerate() {
        let (address, path, made) = chair::make_key(&nest, sitting.as_deref(), seat, Key::fresh)?;
        println!("agent {}   {address} chair {i} of unit 1, the seat {seat}: {} ({})", i + 1, path.display(), if made { "made now, mode 600" } else { "the key that stands there: no other is made" });
        addresses.push(address.to_string());
    }
    println!("  the params' posiverse.agents, its chairs in this order: {}", json!(addresses));
    println!("  each is its seat's own Nest key, in the Nest's keys folder {}: its agent votes its chair by its own step (press chair --key <its file>); no key of the sitting was read, and none of these is in a sitting's keys folder", nest.display());
    Ok(())
}

/// The house the sitting was built from (its record's), else the one this press was built in: where the oath's canonical file is read.
fn house_of(rec: Option<&Record>) -> PathBuf {
    chair::house_at(rec.and_then(|r| r.get("house")["path"].as_str()))
}

/// The Nest's keys folder an agent's key and rite stand in: --nest, else NEST_KEYS, else ~/.embertide/nest.
fn nest_of(a: &Args) -> Res<PathBuf> {
    match (a.opts.get("--nest"), std::env::var_os("NEST_KEYS"), std::env::var_os("HOME")) {
        (Some(n), ..) => Ok(PathBuf::from(n)),
        (None, Some(n), _) => Ok(PathBuf::from(n)),
        (None, None, Some(home)) => Ok(PathBuf::from(home).join(".embertide/nest")),
        _ => Err("--nest: the Nest's keys folder (no NEST_KEYS and no HOME says it)".into()),
    }
}

/// Nefertiti's charter as text, held to the hash the params give it.
fn charter_text(a: &Args, p: &Params) -> Res<String> {
    let f = a.path("--charter")?;
    let text = std::fs::read_to_string(&f).map_err(|e| format!("--charter {}: {e}", f.display()))?;
    let want = p.text("posiverse.charter")?.to_lowercase();
    let got = wire::hex(&wire::sha256(text.as_bytes()));
    if got != want {
        return Err(format!("--charter {}: its sha256 is {got}, and the params hold {want} for Nefertiti's charter: not her charter", f.display()));
    }
    Ok(text)
}

/// THE RITE FREES THE AGENTS (chair.rs, mind.rs; his "Yes, build it", Nov 23 2026): the agent of seat S, its key the one `press chairs` made,
/// is asked once by its own mind, on this machine, to choose its name and its voice, to swear the house's oath or not and to take Nefertiti's
/// charter or not. Freed, its rite is written beside its key (S.rite.json, its user's alone), signed by it; refused, nothing is written and
/// it does not sit. Never over a rite that stands: an agent is freed once. No key of the sitting is read; nothing is sent to any cluster.
fn rite_step(a: &Args) -> Res<()> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let seat = a.get("--seat")?;
    let nest = nest_of(a)?;
    let sitting = a.opts.get("--dir").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/nonexistent"));
    let key_file = nest.join(format!("{seat}.key.json"));
    let (key, named) = chair::read_key(&key_file, &sitting)?;
    if named.as_deref() != Some(seat) {
        return Err(format!("{}: the key there is the seat {:?}'s, not {seat}'s", key_file.display(), named.unwrap_or_default()));
    }
    let out = nest.join(format!("{seat}.rite.json"));
    if out.exists() {
        return Err(format!("{}: a rite stands there: an agent is freed once", out.display()));
    }
    let p = Params::read(&a.path("--params")?)?;
    let charter = charter_text(a, &p)?;
    let (oath, oath_hash) = chair::oath_of(&house_of(None))?;
    let door = mind::Door::parse(a.get("--mind")?)?;
    println!("THE RITE of {seat} ({}): its mind {}; the oath {}; Nefertiti's charter {}", key.address, door.name(), &oath_hash[..12], &p.text("posiverse.charter")?[..12]);
    let t0 = std::time::Instant::now();
    match chair::free(&key, seat, &door, &oath, &charter, &p.text("posiverse.charter")?, &chain::now_iso())? {
        chair::Born::Freed(rite) => {
            let text = serde_json::to_string_pretty(&rite).unwrap() + "\n";
            std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&out).and_then(|mut f| std::io::Write::write_all(&mut f, text.as_bytes()).and_then(|_| f.sync_all())).map_err(|e| format!("{}: {e}", out.display()))?;
            println!("FREED: {seat} is {:?}: \"{}\"", rite["name"].as_str().unwrap_or_default(), rite["voice"].as_str().unwrap_or_default());
            println!("  it swore the oath: \"{}\"", rite["vow"]["why"].as_str().unwrap_or_default());
            println!("  it took Nefertiti's charter: \"{}\"", rite["charter"]["reading"].as_str().unwrap_or_default());
            println!("  its rite {}, signed by its own key; its mind answered in {} ms", out.display(), t0.elapsed().as_millis());
            Ok(())
        }
        chair::Born::Declined(words) => Err(format!("NOT FREED: {words}")),
    }
}

/// AN AGENT VOTES ITS CHAIR (chair.rs): the agent's own step, freed first (its rite beside its key, or --rite): each set read whole by the
/// hands, its vote its own mind's (the mind its rite names), signed over the act it read; its ballots written with its rite. No key of the
/// sitting is read; nothing is sent to any cluster.
fn chair_step(a: &Args, all: &[String]) -> Res<()> {
    let s = public(a, all)?;
    let key_file = a.path("--key")?;
    let (key, seat) = chair::read_key(&key_file, &s.dir)?;
    // (THE SWARM SHIPS THE UPDATE: no key names who proposes a release, so a key of the sitting is its deployer's or a program's)
    let sittings = [("the sitting's deployer", s.deployer), ("Poleis's program id", s.ids.poleis), ("the market's program id", s.ids.market), ("the $WARM program's id", s.ids.warm)];
    chair::not_the_sittings(&key.address, &sittings)?;
    let k = chair::chair_of(&key.address, &s.chairs, &s.p.agents()?)?;
    let charter_hex = s.p.text("posiverse.charter")?.to_lowercase();
    if wire::hex(&s.charter) != charter_hex {
        return Err(format!("unit 1 holds the charter {} on this chain, and the params give {charter_hex}: not the founding these params give", wire::hex(&s.charter)));
    }
    let charter = charter_text(a, &s.p)?;
    let rite_file = match (a.opts.get("--rite"), seat.as_deref()) {
        (Some(r), _) => PathBuf::from(r),
        (None, Some(seat)) => key_file.with_file_name(format!("{seat}.rite.json")),
        (None, None) => return Err("--rite: the key names no seat, so say where its rite is".into()),
    };
    let rite: Value = match std::fs::read_to_string(&rite_file) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", rite_file.display()))?,
        Err(_) => return Err(format!("{} was not freed: no rite at {} (press rite frees it), so it does not vote", key.address, rite_file.display())),
    };
    let (_, oath) = chair::oath_of(&s.house)?;
    let rv = chair::verify_rite(&rite, &key.address, &oath, &charter_hex)?;
    let door = mind::Door::parse(&format!("{}#{}", rv.door, rv.model))?;
    println!("CHAIR {k} of unit 1, his posiverse: {}, {:?}{}; its mind {}; the chain {} ({}), the params {}", key.address, rv.name, seat.as_deref().map(|n| format!(", the seat {n}")).unwrap_or_default(), door.name(), s.genesis, s.rpc.url, s.p.sha256);
    if let Some(dir) = a.opts.get("--releases") {
        return releases_step(a, &s, &key, seat.as_deref(), k, &rv, &door, &rite, &charter, Path::new(dir));
    }
    let sets = a.opts.get("--acts").map(PathBuf::from).unwrap_or_else(|| s.dir.join("acts"));
    let c = chair::cast(&sets, &s.acts, &key, k, s.p.cluster.name(), &s.genesis, &door, &rv, &charter, |line| println!("{line}"))?;
    let out = match a.opts.get("--out") {
        Some(o) => PathBuf::from(o),
        None => s.dir.join("ballots").join(format!("{}.json", key.address)),
    };
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let file = chair::ballots_file(&key, seat.as_deref(), k, s.p.cluster.name(), &s.genesis, &s.p.sha256, &c, &rite);
    std::fs::write(&out, serde_json::to_string_pretty(&file).unwrap() + "\n").map_err(|e| format!("{}: {e}", out.display()))?;
    let count = |w: &str| c.ballots.iter().filter(|b| b["choice"] == json!(w)).count();
    println!("press chair: {} cast by its own mind ({} PRO, {} CONTRA, {} ABSTAIN), nothing on {} sets; its ballots {}; nothing sent to any cluster", c.ballots.len(), count("PRO"), count("CONTRA"), count("ABSTAIN"), c.nothing.len(), out.display());
    if c.nothing.is_empty() {
        Ok(())
    } else {
        Err(format!("nothing was cast on {} of the {} sets read: {}", c.nothing.len(), c.nothing.len() + c.ballots.len(), c.nothing.iter().map(|(set, why)| format!("{set}: {why}")).collect::<Vec<_>>().join("; ")))
    }
}

/// THE AGENTS READ THE RELEASE (release_read.rs): the agent's own step on the releases' sets `press release` wrote (`--releases D/releases`),
/// freed first as for the acts: each set read whole by the hands (its hash derived from its commit, the commit on main in the house's git, its
/// subject and diff stat, the lander's gate from its ledger, its proposal on this chain), put to the agent's own mind (the one its rite
/// names), its choice signed over the release it read; its ballots with its rite in D/ballots/<its address>.releases.json. The house's git is
/// the sitting's house (--house to name another), the ledger the lander's state (--lander, else LANDER_STATE, else ~/.embertide/lander):
/// each read where it stands; no key of the sitting is read, and nothing is sent to any cluster.
#[allow(clippy::too_many_arguments)]
fn releases_step(a: &Args, s: &Public, key: &Key, seat: Option<&str>, k: usize, rv: &chair::RiteView, door: &mind::Door, rite: &Value, charter: &str, dir: &Path) -> Res<()> {
    use poleis_client::{ProposalView, SessionView, STATE_VOTING};
    use poleis_press::release_read::{self, OnChain, Sources};
    let house = a.opts.get("--house").map(PathBuf::from).unwrap_or_else(|| s.house.clone());
    let state = a.opts.get("--lander").map(PathBuf::from).or_else(release_read::lander_state).ok_or("--lander: the lander's state folder (no LANDER_STATE and no HOME says it)")?;
    let ledger = release_read::ledger(&state);
    println!("  THE AGENTS READ THE RELEASE: the releases of {}; the house's git {}; the lander's ledger {} ({} lines)", dir.display(), house.display(), state.display(), ledger.len());
    // what this chain holds of a release's proposal: who proposed it and whether unit 1's vote of it is open (a vote that can no longer land
    // is put to no mind)
    let chain = |h: &[u8; 32]| -> Res<OnChain> {
        let Some(at) = s.rpc.account(&poleis_client::proposal_pda(h).to_string())? else {
            return Ok(OnChain::Asked("its proposal is not on this chain yet: your envelope waits in your file until an agent of unit 1 proposes it".into()));
        };
        if ProposalView::is_tombstone(&at.data) {
            return Ok(OnChain::Done("its proposal is closed on this chain: a vote of it can no longer land".into()));
        }
        let p = ProposalView::parse(&at.data).ok_or("its proposal on this chain does not parse")?;
        let who = match s.chairs.iter().find(|(_, x)| *x == p.payer) {
            Some((c, x)) if *x == key.address => format!("you, chair {c} of unit 1"),
            Some((c, _)) => format!("the agent in chair {c} of unit 1"),
            None => "a key in no chair of unit 1".into(),
        };
        let session = s.rpc.account(&poleis_client::session_pda(h, chain::POSIVERSE).to_string())?.and_then(|x| SessionView::parse(&x.data));
        match session {
            None => Ok(OnChain::Asked(format!("proposed by {who} ({}) at slot {}; unit 1 holds no vote of it yet", p.payer, p.opened))),
            Some(v) if v.state != STATE_VOTING => Ok(OnChain::Done(format!("unit 1 sealed it {} already: a vote of it can no longer land", crank::state_name(v.state)))),
            Some(v) if v.chair_voice(k).is_some() => Ok(OnChain::Done("this chair's vote of it stands on the chain already".into())),
            Some(v) => Ok(OnChain::Asked(format!("proposed by {who} ({}) at slot {}; unit 1's vote of it is open until slot {}, {} of its three chairs heard so far", p.payer, p.opened, v.deadline, v.pent_pro + v.pent_contra + v.pent_abst))),
        }
    };
    let from = Sources { house: &house, ledger: &ledger, chain: &chain };
    let c = release_read::cast(dir, key, k, s.p.cluster.name(), &s.genesis, &from, door, rv, charter, |line| println!("{line}"))?;
    let out = match a.opts.get("--out") {
        Some(o) => PathBuf::from(o),
        None => s.dir.join("ballots").join(release_read::file_name(&key.address)),
    };
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let file = release_read::file(key, seat, k, s.p.cluster.name(), &s.genesis, &s.p.sha256, &c, rite);
    std::fs::write(&out, serde_json::to_string_pretty(&file).unwrap() + "\n").map_err(|e| format!("{}: {e}", out.display()))?;
    let count = |w: &str| c.ballots.iter().filter(|b| b["choice"] == json!(w)).count();
    println!("press chair --releases: {} cast by its own mind ({} PRO, {} CONTRA, {} ABSTAIN), nothing on {} sets; its ballots {}; nothing sent to any cluster", c.ballots.len(), count("PRO"), count("CONTRA"), count("ABSTAIN"), c.nothing.len(), out.display());
    if c.nothing.is_empty() {
        Ok(())
    } else {
        Err(format!("nothing was cast on {} of the {} release sets read: {}", c.nothing.len(), c.nothing.len() + c.ballots.len(), c.nothing.iter().map(|(set, why)| format!("{set}: {why}")).collect::<Vec<_>>().join("; ")))
    }
}

/// THE BALLOTS, HELD (chair.rs): what `press crank --signatures` would be handed, read and held to each voter's key, its chair on this
/// chain and its act; nothing sent.
fn ballots_step(a: &Args, all: &[String]) -> Res<()> {
    let s = public(a, all)?;
    let text = chair::gather(&a.path("--signatures")?)?;
    let (rows, rel) = (crank::ballots(&text, &s.acts)?, crank::release_ballots(&text)?);
    let mut refused = Vec::new();
    // THE RITE FREES THE AGENTS: each row's voter freed, the rite it carries its own key's, over the oath the house holds now and her charter
    let (_, oath) = chair::oath_of(&s.house)?;
    let unfreed = chair::unfreed_rows(&text, &oath, &s.p.text("posiverse.charter")?)?;
    let not_freed = |voter: &Pubkey, what: String, unit: u64| unfreed.iter().find(|(v, w, u, _)| *v == voter.to_string() && *w == what && *u == unit.to_string()).map(|(.., why)| why.clone());
    for b in &rows {
        let sits = s.chairs.iter().find(|(_, key)| *key == b.voter).map(|(k, _)| *k);
        match (chair::held(b, &s.acts), sits, not_freed(&b.voter, format!("act {}", b.act), b.unit)) {
            (Ok(()), Some(k), None) => println!("  act {}: chair {k} of unit 1 ({}): held, its own signature over this act's envelope, its voter freed", b.act, b.voter),
            (Ok(()), Some(_), Some(why)) => {
                println!("  act {}: {}: REFUSED: {why}", b.act, b.voter);
                refused.push(format!("the ballot of {} on act {}: {why}", b.voter, b.act));
            }
            (Ok(()), None, _) => {
                println!("  act {}: {}: REFUSED: it sits in no chair of unit 1 on this chain: a key that is not a chair's counts for nothing", b.act, b.voter);
                refused.push(format!("the ballot of {} on act {}: no chair of unit 1", b.voter, b.act));
            }
            (Err(why), ..) => {
                println!("  act {}: {}: REFUSED: {why}", b.act, b.voter);
                refused.push(why);
            }
        }
    }
    // THE BRANCH VOTES ITS OWN CODE: a release's ballots, held to their voters' own signatures and to unit 1's chairs the same way
    for b in &rel {
        let short = &wire::hex(&b.hash)[..12];
        match (crank::release_held(b), s.chairs.iter().any(|(_, key)| *key == b.voter), not_freed(&b.voter, format!("release {}", wire::hex(&b.hash)), chain::POSIVERSE)) {
            (Ok(()), true, None) => println!("  release {short}: {}: held, its own signature over this release's envelope in unit 1, its voter freed", b.voter),
            (Ok(()), true, Some(why)) => {
                println!("  release {short}: {}: REFUSED: {why}", b.voter);
                refused.push(format!("the ballot of {} on release {short}: {why}", b.voter));
            }
            (Ok(()), false, _) => {
                println!("  release {short}: {}: REFUSED: it sits in no chair of unit 1 on this chain: no seat holder votes code", b.voter);
                refused.push(format!("the ballot of {} on release {short}: no chair of unit 1", b.voter));
            }
            (Err(why), ..) => {
                println!("  release {short}: {}: REFUSED: {why}", b.voter);
                refused.push(why);
            }
        }
    }
    let n = rows.len() + rel.len();
    println!("press ballots: {} held, {} refused; nothing sent to any cluster", n - refused.len(), refused.len());
    if refused.is_empty() {
        Ok(())
    } else {
        Err(format!("{} of the {} ballots are not held: {}", refused.len(), n, refused.join("; ")))
    }
}

fn sitting(a: &Args, all: &[String]) -> Res<Sitting> {
    let (dir, rec, ids, deployer, p, rpc, genesis) = opened(a, all)?;
    let mut rec = rec;
    match rec.get("genesisHash").as_str() {
        Some(g) if g != genesis => return Err(format!("this sitting's chain is {g}; {} answers {genesis}", rpc.url)),
        Some(_) => {}
        None => rec.set("genesisHash", json!(genesis))?,
    }
    let keys = if a.rehearsal() {
        let seed = seed_from_stdin()?;
        for (n, want) in [("deployer", deployer), ("poleis", ids.poleis), ("market", ids.market), ("warm", ids.warm)] {
            if wire::standin(&seed, n).address != want {
                return Err(format!("the seed's {n} is not the sitting's: another rehearsal's seed"));
            }
        }
        KeySource::Seed(seed)
    } else {
        // his finger's asker: on mainnet the house's own, its bytes the ones the house committed; PRESS_TOUCHID only off mainnet
        let house = PathBuf::from(rec.get("house")["path"].as_str().unwrap_or("."));
        let touchid = chain::asker(p.cluster, &house, std::env::var_os("PRESS_TOUCHID"))?;
        KeySource::Files { dir: dir.join("keys"), touchid }
    };
    let at = chain::addresses(&ids, p.u64_or("market.board", 0)?);
    let copy = PathBuf::from(rec.get("copy").as_str().unwrap_or_default());
    Ok(Sitting { dir, copy, rec, p, rpc, ids, deployer, keys, at })
}

/// THE KEYS: four keypairs made by his hand, each in the Solana CLI's file shape, mode 600 in a folder of mode 700; their public addresses
/// beside them (ids.json, what `build` reads, no key in it). Never over a key that stands.
fn keys(a: &Args) -> Res<()> {
    use std::os::unix::fs::OpenOptionsExt as _;
    // a key's folder is outside every repository, his user's alone (seat.rs keys_folder)
    let dir = seat::keys_folder(&a.path("--dir")?)?;
    for n in KEY_NAMES {
        if dir.join(format!("{n}.json")).exists() {
            return Err(format!("{}: a key stands there; the press never writes over a key", dir.join(format!("{n}.json")).display()));
        }
    }
    let mut ids = serde_json::Map::new();
    for n in KEY_NAMES {
        let k = Key::fresh()?;
        let path = dir.join(format!("{n}.json"));
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut text = k.cli_text();
        std::io::Write::write_all(&mut f, text.as_bytes()).and_then(|_| f.sync_all()).map_err(|e| format!("{}: {e}", path.display()))?;
        text.clear();
        ids.insert(n.to_string(), json!(k.address.to_string()));
        println!("{n:<9} {}", k.address);
    }
    let path = dir.join("ids.json");
    std::fs::write(&path, serde_json::to_string_pretty(&Value::Object(ids)).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(())
}

/// PRESS ENGINE (THE STAKE EARNS, engine.rs): one run of the engine's crank on her pool, by the params' rule. `--make-key F` makes the engine's
/// key once (a key file in the CLI's shape, mode 600, never over a file that stands) and says its address for `engine.staker`; `--key F`
/// signs and sends; without it the run says what it would send and sends nothing.
fn engine_step(a: &Args) -> Res<()> {
    use poleis_press::engine;
    if let Some(f) = a.opts.get("--make-key") {
        let path = PathBuf::from(f);
        if path.exists() {
            return Err(format!("{f}: a file stands there; no key is made over it"));
        }
        let k = Key::fresh()?;
        let mut text = k.cli_text();
        let mut o = std::fs::OpenOptions::new();
        std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
        let wrote = o.write(true).create_new(true).open(&path).and_then(|mut h| std::io::Write::write_all(&mut h, text.as_bytes()));
        text.clear();
        wrote.map_err(|e| format!("{f}: {e}"))?;
        println!("engine    {} ({f}, mode 600): the params' engine.staker; the ninth act names it her pool's staker", k.address);
        return Ok(());
    }
    let p = Params::read(&a.path("--params")?)?;
    // THE ENGINE'S BLANKS HOLD ONLY THE ENGINE: refused before anything is read, each named
    engine::blanks(&p)?;
    let rpc = Rpc::new(a.get("--rpc")?, p.cluster)?;
    rpc.check_cluster()?;
    let pool = Pubkey::from_str(a.get("--pool")?).map_err(|_| "--pool: her pool's address".to_string())?;
    let key = a.opts.get("--key").map(|f| std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}")).and_then(|t| Key::from_cli_text(&t))).transpose()?;
    let engine_key = match (&key, engine::staker_standing(&p)) {
        (Some(k), _) => k.address,
        (None, Some(s)) => s,
        (None, None) => return Err("the engine's key: --key F, or the params' engine.staker".into()),
    };
    let rule = engine::rule_on(&p, &rpc)?;
    let r = engine::run(&rpc, &pool, &rule, &engine_key, key.as_ref())?;
    for w in &r.said {
        println!("  {w}");
    }
    println!("{}", serde_json::to_string_pretty(&json!({ "epoch": r.epoch, "sent": r.sent, "fees": r.fees, "after": r.after })).unwrap());
    println!("PRESS ENGINE: {} round(s) sent at epoch {}, {} lamports of fees", r.sent.iter().filter(|s| s.get("did").is_some()).count(), r.epoch, r.fees);
    Ok(())
}

/// THE MINT DOOR'S KEYS (seat.rs): `press issuer`, its permit key, and `press miner`, its crank key (THE SWARM NAMES ITS MINER), each made by his hand
/// beside the sitting's other keys, its address said for its param (`floor.issuer`, `warm.miner`), the door's vars when the params and the program
/// ids are there, and the road of the door's secret, his press, said and never run. With --secret, that press: after his finger, the key into the
/// pipe it is given, never onto a screen. `press crediter` (THE PRESS IS READY FOR MAINNET), the market board's crediter, made the same way, its
/// address said for `market.crediter`; no door reads it yet, so it has no secret.
fn door_key_step(a: &Args, name: &str) -> Res<()> {
    let dir = a.path("--dir")?;
    let miner = name == "miner";
    if name == "crediter" {
        if a.flag("--secret") {
            return Err("the crediter's key has no secret's road: no door reads it yet (THE PRESS IS READY FOR MAINNET, what is not done)".into());
        }
        let (address, made) = seat::crediter_key(&dir, Key::fresh)?;
        println!("{name:<9} {address}{}", if made { "" } else { " (the key that stands there: no other is made)" });
        println!("  the params' market.crediter: {address} (the press's second act names it the board's crediter: the key that credits play $WARM to the board's balances, sealed by the whole tree)");
        println!("  its SOL, on this key and never the deployer's: each credit's fee");
        return Ok(());
    }
    let var = if miner { "SPACE_ROUTER_MINER_KEY" } else { "POLEIS_ISSUER_KEY" };
    if a.flag("--secret") {
        use std::io::{IsTerminal as _, Write as _};
        if std::io::stdout().is_terminal() {
            return Err(format!("the door's secret goes into a pipe (... --secret | npx wrangler secret put {var}), never onto a screen"));
        }
        let p = Params::read(&a.path("--params")?)?;
        let house = match Record::open(&dir)?.get("house")["path"].as_str() {
            Some(h) => PathBuf::from(h),
            None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().map_err(|e| e.to_string())?,
        };
        let touchid = chain::asker(p.cluster, &house, std::env::var_os("PRESS_TOUCHID"))?;
        let mut secret = if miner { seat::miner_secret(&dir, &touchid)? } else { seat::issuer_secret(&dir, &touchid)? };
        let wrote = std::io::stdout().write_all(secret.as_bytes()).and_then(|_| std::io::stdout().flush());
        secret.clear();
        return wrote.map_err(|e| format!("the pipe: {e}"));
    }
    let (address, made) = if miner { seat::miner_key(&dir, Key::fresh)? } else { seat::issuer_key(&dir, Key::fresh)? };
    println!("{name:<9} {address}{}", if made { "" } else { " (the key that stands there: no other is made)" });
    if miner {
        println!("  the params' warm.miner: {address} (the press's sixth act names it the $WARM mine's miner: set_miner by the polis's hand, sealed by the whole tree; until it executes the mine mines nothing)");
        println!("  its SOL, on this key and never the deployer's: it pays each flask's $WARM account (165 bytes, at the chain's rent) at the flask's first mined spend, and each batch's fee");
    } else {
        println!("  the params' floor.issuer: {address} (root 0's curve names it: its signature on a permit lets one named buyer mint one seat)");
    }
    let me = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "press".into());
    println!("  the door's secret, HIS press and never a seat's, not run here: {me} {name} --dir {} --params <the sitting's params> --secret | npx wrangler secret put {var} --config embertide-app/ui/intro/dyson-society/wrangler.optimism-fit.jsonc", dir.display());
    let ids: Option<Value> = std::fs::read_to_string(dir.join("keys/ids.json")).ok().and_then(|t| serde_json::from_str(&t).ok());
    match (a.opts.get("--params"), ids.as_ref().and_then(|v| v["poleis"].as_str()).and_then(|s| Pubkey::from_str(s).ok())) {
        (Some(p), Some(poleis)) => {
            let p = Params::read(Path::new(p))?;
            let rec = Record::open(&dir)?;
            let genesis = match (p.cluster, rec.get("genesisHash").as_str()) {
                (Cluster::Mainnet, _) => poleis_press::rpc::MAINNET_GENESIS.to_string(),
                (_, Some(g)) => g.to_string(),
                _ => "the cluster's genesis hash (the record's genesisHash, once a step has asked the cluster)".to_string(),
            };
            println!("  the door's vars (wrangler.optimism-fit.jsonc \"vars\", at his deploy once root 0 is founded): {}", seat::door_vars(&p, &poleis, &genesis));
            if let Some(warm) = ids.as_ref().and_then(|v| v["warm"].as_str()).and_then(|s| Pubkey::from_str(s).ok()) {
                println!("  the door's vars of the $WARM mine (THE MINE IS LIVE; with migration 0031 and the miner's secret, once a sealed act of the polis has named the miner): {}", seat::mine_vars(&p, &poleis, &warm, rec.get("warm")["opening"].as_u64()));
            }
        }
        _ => println!("  the door's vars: press {name} --dir {} --params <the sitting's params>, once press keys has made the program ids, says them", dir.display()),
    }
    println!("{}", seat::door_law(&Record::open(&dir)?.get("builds")["seat"]));
    Ok(())
}

/// THE BUILD, RECORDED (build.rs).
fn build_step(a: &Args) -> Res<()> {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = a.path("--dir")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let p = Params::read(&a.path("--params")?)?;
    let house = match a.opts.get("--house") {
        Some(h) => PathBuf::from(h),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().map_err(|e| e.to_string())?,
    };
    let copy = match a.opts.get("--copy") {
        Some(c) => PathBuf::from(c),
        None => dir.join("copy"),
    };
    // what a sitting builds is what a commit holds: the parts' changes that no commit holds (a file edited, a file added) are read here,
    // and on mainnet they stop the build (bytes no commit holds, and nobody reviewed, are deployed nowhere)
    let uncommitted = std::process::Command::new("git").arg("-C").arg(&house).args(["status", "--porcelain", "--"]).args(build::PARTS).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).lines().map(str::to_string).collect::<Vec<_>>());
    if p.cluster == Cluster::Mainnet {
        match &uncommitted {
            None => return Err(format!("{} is not a git checkout of the house: on mainnet the sitting builds what a commit holds", house.display())),
            Some(u) if !u.is_empty() => return Err(format!("the house at {} holds {} changes of the programs' parts that no commit holds ({}): on mainnet the sitting builds what a commit holds", house.display(), u.len(), u.iter().take(5).map(|l| l.trim()).collect::<Vec<_>>().join("; "))),
            Some(_) => {}
        }
    }
    let (ids, deployer) = if a.rehearsal() {
        if p.cluster != Cluster::Localnet {
            return Err("a rehearsal's build is for a validator of this machine: the params' cluster is localnet".into());
        }
        let seed = seed_from_stdin()?;
        let k = |n: &str| wire::standin(&seed, n).address;
        (Ids { poleis: k("poleis"), market: k("market"), warm: k("warm") }, k("deployer"))
    } else {
        let path = dir.join("keys/ids.json");
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| format!("{}: {e} (press keys makes it)", path.display()))?).map_err(|e| e.to_string())?;
        let k = |n: &str| Pubkey::from_str(v[n].as_str().unwrap_or_default()).map_err(|_| format!("{}: no {n}", path.display()));
        (Ids { poleis: k("poleis")?, market: k("market")?, warm: k("warm")? }, k("deployer")?)
    };
    let mut rec = Record::open(&dir)?;
    if let Some(had) = rec.get("ids")["poleis"].as_str() {
        if had != ids.poleis.to_string() {
            return Err(format!("{} is another sitting's (Poleis {had}): a sitting is one folder", dir.display()));
        }
    }
    eprintln!("THE BUILD, for Poleis {}, the market {}, the $WARM program {}", ids.poleis, ids.market, ids.warm);
    let (written, fund, bump) = build::copy_and_patch(&house, &copy, &ids)?;
    eprintln!("  the copy {}: {written} files written; FUND_ADDRESS {fund}, FUND_BUMP {bump}", copy.display());
    let mut builds = serde_json::Map::new();
    for (name, krate, so, out) in build::PROGRAMS {
        let bytes = build::build_program(&copy, krate, so, out)?;
        let sha = wire::hex(&wire::sha256(&bytes));
        eprintln!("  {name}: {} bytes, sha256 {sha}", bytes.len());
        builds.insert(name.into(), json!({ "bytes": bytes.len(), "sha256": sha, "path": build::out_dir(&copy, out).join(so).display().to_string() }));
    }
    let press = build::build_press(&copy)?;
    let press_sha = wire::hex(&wire::sha256(&std::fs::read(&press).map_err(|e| e.to_string())?));
    // THE FLASK TAKES ITS SEAT: the mint door's seat law, its Poleis the sitting's; recorded as the programs are, named by `press issuer` beside the vars
    let (law, law_path) = build::build_seat(&copy)?;
    let law_sha = wire::hex(&wire::sha256(&law));
    eprintln!("  the door's seat law: {} bytes, sha256 {law_sha}", law.len());
    builds.insert("seat".into(), json!({ "bytes": law.len(), "sha256": law_sha, "path": law_path.display().to_string() }));
    // the sitting's press speaks the sitting's ids, and its fund is the one this press derives
    let out = std::process::Command::new(&press).arg("ids").arg("--board").arg(p.u64_or("market.board", 0)?.to_string()).output().map_err(|e| format!("{}: {e}", press.display()))?;
    let spoken: Value = serde_json::from_slice(&out.stdout).map_err(|e| format!("the sitting's press says no ids ({e})"))?;
    let (upgrade, _) = Pubkey::find_program_address(&[b"upgrade"], &ids.poleis);
    let (agent, _) = Pubkey::find_program_address(&[b"agent", &0u64.to_le_bytes()], &ids.poleis);
    for (k, want) in [("poleis", ids.poleis.to_string()), ("market", ids.market.to_string()), ("warm", ids.warm.to_string()), ("fund", fund.to_string()), ("fundFixed", fund.to_string()), ("upgrade", upgrade.to_string()), ("agent", agent.to_string())] {
        if spoken[k].as_str() != Some(want.as_str()) {
            return Err(format!("the sitting's press speaks {k} {}, the sitting's is {want}: refused", spoken[k]));
        }
    }
    if spoken["fundBump"] != json!(bump) || spoken["fundBumpFixed"] != json!(bump) {
        return Err(format!("the sitting's press's FUND_BUMP {} / {}, the derivation {bump}: refused (THE CEREMONY, REHEARSED, finding 1)", spoken["fundBump"], spoken["fundBumpFixed"]));
    }
    let commit = std::process::Command::new("git").arg("-C").arg(&house).args(["rev-parse", "HEAD"]).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    eprintln!("  built from the house at {}, commit {}, {}", house.display(), commit.as_deref().unwrap_or("none (not a git checkout)"), match &uncommitted {
        Some(u) if u.is_empty() => "the programs' parts as that commit holds them".to_string(),
        Some(u) => format!("with {} changes of the programs' parts no commit holds (a rehearsal's only)", u.len()),
        None => "its parts unread by git".to_string(),
    });
    rec.v["house"] = json!({ "path": house.display().to_string(), "commit": commit, "partsUncommitted": uncommitted });
    rec.v["cluster"] = json!(p.cluster.name());
    rec.v["params"] = json!({ "sha256": p.sha256 });
    rec.v["ids"] = json!({ "poleis": ids.poleis.to_string(), "market": ids.market.to_string(), "warm": ids.warm.to_string(), "deployer": deployer.to_string() });
    rec.v["derived"] = spoken;
    rec.v["builds"] = Value::Object(builds);
    rec.v["press"] = json!({ "path": press.display().to_string(), "sha256": press_sha });
    rec.v["copy"] = json!(copy.display().to_string());
    rec.save()?;
    println!("THE BUILD, RECORDED in {}: the three binaries' sha256 above; the sitting's own press {} (sha256 {press_sha})", rec.path.display(), press.display());
    Ok(())
}

/// The ids, fund constants and addresses this build speaks.
fn ids_step(a: &Args) -> Res<()> {
    let (ids, fund, bump, fixed, fixed_bump) = build::compiled()?;
    let board: u64 = a.opts.get("--board").map(|b| b.parse().unwrap_or(0)).unwrap_or(0);
    let at = chain::addresses(&ids, board);
    println!(
        "{}",
        json!({
            "poleis": ids.poleis.to_string(), "market": ids.market.to_string(), "warm": ids.warm.to_string(), "fund": fund.to_string(), "fundBump": bump, "fundFixed": fixed.to_string(), "fundBumpFixed": fixed_bump,
            "upgrade": at.upgrade.to_string(), "agent": at.agent.to_string(), "config": at.config.to_string(), "registry": at.registry.to_string(), "treasury": at.treasury.to_string(),
            "curve": at.curve.to_string(), "marketLaw": at.market_law.to_string(), "mine": at.mine.to_string(), "mint": at.mint.to_string(), "vault": at.vault.to_string(),
            "metadata": at.metadata.to_string(), "board": at.board.to_string(), "boardId": board,
            "programData": { "poleis": poleis_press::loader::programdata_of(&ids.poleis).to_string(), "market": poleis_press::loader::programdata_of(&ids.market).to_string(), "warm": poleis_press::loader::programdata_of(&ids.warm).to_string() },
        })
    );
    Ok(())
}

enum Fail {
    Usage(String),
    Refused(String),
    Short(String),
}

fn run(all: &[String]) -> Result<(), Fail> {
    let a = Args::parse(all).map_err(Fail::Usage)?;
    let refused = Fail::Refused;
    match a.step.as_str() {
        "keys" => keys(&a).map_err(refused),
        "issuer" | "miner" | "crediter" => door_key_step(&a, &a.step).map_err(refused),
        "build" => build_step(&a).map_err(refused),
        "ids" => ids_step(&a).map_err(refused),
        "chairs" => chairs_step(&a).map_err(refused),
        "rite" => rite_step(&a).map_err(refused),
        "chair" => chair_step(&a, all).map_err(refused),
        "ballots" => ballots_step(&a, all).map_err(refused),
        // THE LIGHT PAYS A WISH AND ITS RIDER (light.rs, rider.rs): the epoch's pour, said or sent; the sets of a wish's rider and of root 0's well
        "light" => poleis_press::light::step(&a.opts, a.flags.contains("--send")).map(|v| println!("{}", serde_json::to_string_pretty(&v).unwrap())).map_err(refused),
        "rider" | "well" => poleis_press::rider::step(&a.step, &a.opts).map(|v| println!("{}", serde_json::to_string_pretty(&v).unwrap())).map_err(refused),
        // THE EPOCH TURNS THE CRANKS (epoch.rs): one pass, or a pass each epoch (--stay); and its two keys, made once beside the sitting's keys
        "epoch" => poleis_press::epoch::step(&a.opts, a.flags.contains("--stay")).map_err(refused),
        "engine-key" | "pourer" => {
            let dir = a.path("--dir").map_err(Fail::Usage)?;
            let made = if a.step == "pourer" { poleis_press::seat::pourer_key(&dir, Key::fresh) } else { poleis_press::seat::engine_key(&dir, Key::fresh) };
            made.map(|(address, new)| {
                println!("{:<10} {address}{}", a.step, if new { "" } else { " (the key that stands there: no other is made)" });
                if a.step == "engine-key" {
                    println!("  the params' engine.staker: {address} (the ninth act names it her pool's staker); its SOL pays the engine's fees each epoch");
                } else {
                    println!("  the light's crank: it pays each pour's fee and a wish's vault's rent at its first pour; no param names it (anyone may pull a unit's road)");
                }
            })
            .map_err(refused)
        }
        "engine" => engine_step(&a).map_err(refused),
        "need" => {
            let s = sitting(&a, all).map_err(refused)?;
            let bins = s.binaries().map_err(Fail::Refused)?;
            let (need, lines) = s.need(&bins).map_err(Fail::Refused)?;
            let bal = s.rpc.balance(&s.deployer.to_string()).map_err(Fail::Refused)?;
            chain::say_need(&lines, need, bal, &s.deployer);
            Ok(())
        }
        "airdrop" => {
            let s = sitting(&a, all).map_err(refused)?;
            if !a.rehearsal() {
                return Err(Fail::Refused("the faucet pays a rehearsal's stand-in deployer only".into()));
            }
            let bins = s.binaries().map_err(Fail::Refused)?;
            let (need, _) = s.need(&bins).map_err(Fail::Refused)?;
            let bal = s.rpc.balance(&s.deployer.to_string()).map_err(Fail::Refused)?;
            if bal >= need {
                println!("the deployer holds {} SOL, the need {} SOL: no airdrop", chain::sol(bal), chain::sol(need));
                return Ok(());
            }
            let sig = s.rpc.airdrop(&s.deployer.to_string(), need - bal).map_err(Fail::Refused)?;
            // the faucet's lamports have come when its own transaction is confirmed: a write of a pass cut short may still land meanwhile
            // and take its fee, so the balance is no sign (with a priority fee those writes' fees outran a small airdrop, and a balance
            // waited for to rise never rose: the rehearsal of mainnet's values, Nov 23 2026); the deploy reads both again
            poleis_press::rpc::wait(std::time::Duration::from_secs(120), || match s.rpc.statuses(std::slice::from_ref(&sig))?.into_iter().next().flatten() {
                Some((_, e, level)) if e.is_null() && (level == "confirmed" || level == "finalized") => Ok(Some(())),
                Some((_, e, _)) if !e.is_null() => Err(format!("the faucet's airdrop {sig} failed: {e}")),
                _ => Ok(None),
            })
            .map_err(Fail::Refused)?;
            // what came in after the sitting started counts in what it spent
            let mut s = s;
            let after = !s.rec.get("before").is_null();
            let row = json!({ "lamports": need - bal, "signature": sig, "afterStart": after });
            if s.rec.v["airdrops"].is_array() {
                s.rec.v["airdrops"].as_array_mut().unwrap().push(row);
            } else {
                s.rec.v["airdrops"] = json!([row]);
            }
            s.rec.save().map_err(Fail::Refused)?;
            println!("the local faucet paid the stand-in deployer {} SOL ({sig}): what it lacked of the need, {} SOL", chain::sol(need - bal), chain::sol(need));
            Ok(())
        }
        "deploy" | "found" | "vault" | "warm" | "market" | "pool" | "ratify" | "vote" | "release" | "seat" | "crank" | "handoff" | "acts" | "certificate" | "status" | "sit" | "engine-act" => {
            let mut s = sitting(&a, all).map_err(refused)?;
            // the first sitting; the handoff and the certificate are the second, once the seal has acted
            let steps: Vec<&str> = if a.step == "sit" { vec!["deploy", "found", "vault", "warm", "market", "pool", "acts"] } else { vec![a.step.as_str()] };
            for st in steps {
                eprintln!("press {st}:");
                let r = match st {
                    "deploy" => s.deploy(),
                    "found" => s.found_step(),
                    "vault" => s.vault(),
                    "warm" => s.warm(),
                    "ratify" => s.ratify(),
                    "vote" => {
                        let n = a.get("--act").and_then(|n| n.parse::<usize>().map_err(|_| format!("--act {n}: an act's number")));
                        let owner = a.get("--owner").and_then(|o| Pubkey::from_str(o).map_err(|_| format!("--owner {o}: an address")));
                        n.and_then(|n| owner.and_then(|o| s.vote(n, &o, a.flag("--propose")))).map(|f| println!("  {}", f.display()))
                    }
                    // THE BRANCH VOTES ITS OWN CODE: the agents' work order for a release of the house's code (unit 1's how)
                    "release" => {
                        let agent = a.opts.get("--agent").map(|x| Pubkey::from_str(x).map_err(|_| format!("--agent {x}: an address"))).transpose();
                        a.get("--commit").and_then(|c| agent.and_then(|g| s.release(c, g))).map(|f| println!("  {}", f.display()))
                    }
                    "seat" => {
                        let buyer = a.get("--buyer").and_then(|b| Pubkey::from_str(b).map_err(|_| format!("--buyer {b}: an address")));
                        let max = a.opts.get("--max").map(|m| m.parse::<u64>().map_err(|_| format!("--max {m}: lamports, a whole number"))).transpose();
                        a.path("--permit").and_then(|f| buyer.and_then(|b| max.and_then(|m| s.seat(&f, &b, m)))).map(|f| println!("  {}", f.display()))
                    }
                    "crank" => {
                        let upgrade = a.opts.get("--upgrade").map(|h| wire::unhex::<32>(h).map_err(|_| format!("--upgrade {h}: the sealed upgrade's proposal hash, 64 hex"))).transpose();
                        upgrade.and_then(|u| a.path("--signatures").and_then(|f| chair::gather(&f)).and_then(|t| s.crank(&t, u)))
                    }
                    "market" => s.market(),
                    // THE STAKE EARNS: the ninth act's set, the engine's staker named by the polis's hand, beside the eight
                    "engine-act" => {
                        let genesis = s.rec.get("genesisHash").as_str().unwrap_or_default().to_string();
                        poleis_press::engine::write_set(&s.dir.join("acts"), &s.p, &s.ids, &s.at, &genesis, &s.deployer).map(|f| println!("  {}", f.display()))
                    }
                    "pool" => s.pool(),
                    "handoff" => s.handoff(),
                    "acts" => {
                        let genesis = s.rec.get("genesisHash").as_str().unwrap_or_default().to_string();
                        // the fifth act, the pool's birth, is written only where the stake pool program stands; mainnet has it. The sixth names
                        // the mine's miner: the key press miner made beside his keys, where it made one
                        // the mine's miner (act 6) and the board's crediter (act 2): the keys press miner and press crediter made beside his
                        // keys, where they made one; the curve's issuer too (THE PRESS IS READY FOR MAINNET)
                        seat::kept_keys_held(&s.dir, &s.p).and_then(|_| s.stake_pool_stands()).and_then(|birth| {
                            if !birth && s.p.cluster == Cluster::Mainnet {
                                return Err(poleis_press::pool::NOT_MAINNET.to_string());
                            }
                            let deployer = s.deployer;
                            acts::write(&s.dir.join("acts"), &s.p, &s.ids, &s.at, &genesis, &deployer, birth).and_then(|mut files| {
                                // THE NEW ACTS IN THE LAUNCH REHEARSAL: the later acts the params name (the ninth, the tenth), their sets beside
                                // the eight and in the record's list, where the pool's birth is written too; none where the params name none
                                if birth {
                                    for a in poleis_press::engine::later(&s.p, &s.ids, &s.at, &deployer)? {
                                        let at = s.dir.join("acts");
                                        files.push(match a.n {
                                            poleis_press::engine::ENGINE_ACT => poleis_press::engine::write_set(&at, &s.p, &s.ids, &s.at, &genesis, &deployer)?,
                                            _ => poleis_press::light::write_set(&at, &s.p, &s.ids, &s.at, &genesis, &deployer)?,
                                        });
                                    }
                                }
                                for f in &files {
                                    println!("  {}", f.display());
                                }
                                if !birth {
                                    println!("  act 5, the pool's birth: its set is not written: {}", poleis_press::pool::NOT_RUN);
                                }
                                s.rec.set("acts", json!(files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>()))
                            })
                        })
                    }
                    "certificate" => {
                        let checks = a.opts.get("--checks").map(PathBuf::from);
                        s.certificate(checks.as_deref()).and_then(|mut c| {
                            // what the seal's reader said of the road on this chain (a rehearsal's reads, by the script): beside the road
                            if let Some(f) = a.opts.get("--seal-reads") {
                                let reads: Value = serde_json::from_str(&std::fs::read_to_string(f).map_err(|e| format!("--seal-reads {f}: {e}"))?).map_err(|e| format!("--seal-reads {f}: {e}"))?;
                                if c["sealRoad"].is_object() {
                                    c["sealRoad"]["reads"] = reads;
                                } else {
                                    c["sealReads"] = reads;
                                }
                            }
                            // AN AGENT VOTES ITS CHAIR: what the agents' own step and the road's faults said (a rehearsal's, by the script)
                            if let Some(f) = a.opts.get("--chairs") {
                                c["chairs"] = serde_json::from_str(&std::fs::read_to_string(f).map_err(|e| format!("--chairs {f}: {e}"))?).map_err(|e| format!("--chairs {f}: {e}"))?;
                            }
                            let text = serde_json::to_string_pretty(&c).unwrap() + "\n";
                            let path = s.dir.join("certificate.json");
                            std::fs::write(&path, &text).map_err(|e| e.to_string())?;
                            if let Some(out) = a.opts.get("--out") {
                                std::fs::write(out, &text).map_err(|e| format!("{out}: {e}"))?;
                            }
                            println!("THE BIRTH CERTIFICATE: {}", path.display());
                            match c["sealRoad"]["road"].is_object() {
                                true => {
                                    let road = s.dir.join(seal::FILE);
                                    println!("THE SEAL'S ROAD: {} {}", road.display(), c["sealRoad"]["road"]);
                                    println!("  {}", seal::said(&c["sealRoad"]["road"]));
                                    println!("  the lander takes it by {}", seal::lander_line(&road));
                                }
                                false => println!("THE SEAL'S ROAD: {}", c["sealRoad"].as_str().unwrap_or("not written")),
                            }
                            Ok(())
                        })
                    }
                    _ => status(&mut s),
                };
                r.map_err(|e| if e.contains("short by") { Fail::Short(e) } else { Fail::Refused(e) })?;
            }
            if a.step == "sit" || a.step == "acts" {
                until_the_handoff(&s).map_err(Fail::Refused)?;
            }
            Ok(())
        }
        other => Err(Fail::Usage(format!("{other}: not a step of the press"))),
    }
}

/// What the first sitting leaves, and what the deployer holds until the second: said once the acts are written.
fn until_the_handoff(s: &Sitting) -> Res<()> {
    let held: Vec<&str> = build::PROGRAMS
        .iter()
        .filter(|(name, ..)| {
            let pd = poleis_press::loader::programdata_of(&chain::program_id(&s.ids, name));
            matches!(s.rpc.account(&pd.to_string()).ok().flatten().map(|a| poleis_press::loader::state(&a.data)), Some(Ok(poleis_press::loader::State::ProgramData { authority: Some(k), .. })) if k == s.deployer)
        })
        .map(|(name, ..)| *name)
        .collect();
    if held.is_empty() {
        return Ok(());
    }
    println!("UNTIL THE HANDOFF the deployer {} holds the code of {} (their upgrade authority): its key is the code.", s.deployer, held.join(", "));
    match &s.keys {
        KeySource::Files { dir, .. } => println!("  Keep {} off this machine between the sittings (a disk that is not mounted), and bring it back for the handoff.", dir.display()),
        KeySource::Seed(_) => println!("  (the rehearsal's stand-ins: in memory only)"),
    }
    println!("THE HANDOFF (`press handoff`, then `press certificate`) waits for the seal to act: root 0's floor minted on its curve (each seat by its own buyer, with a permit of the mint door), root 0 ratified once the curve's window has closed (press ratify), and the fund constituted (act 1: two of three agents in unit 1 AND more than half of the floor's seats, each by its owner's own transaction (press vote), the rest landed by press crank). Until then the deployer can mend the code; after it, only the seal.");
    if s.stake_pool_stands()? {
        println!("THE POOL'S BIRTH comes before the handoff: act 5, passed the same way, its execute's transaction making the pool's account and its list by the deployer's key (press crank). The deployer's key is needed until then.");
    } else {
        println!("{}: the handoff does not wait for it on this chain (on mainnet the pool is born before the handoff, by act 5 and the deployer's key).", poleis_press::pool::NOT_RUN);
    }
    println!("THE MINER (act 6, passed the same way): set_miner by the polis's hand names the params' warm.miner {} the $WARM mine's miner; until it executes the mine mines nothing. Its key is the door's (press miner, its secret his press), never the deployer's.", s.p.at("warm.miner").as_str().unwrap_or("(none)"));
    // THE TREASURY PAYS THE COMPUTE (his Nov 23 2026 23:08:00 UTC "Yes, add them"): the two acts after it, said the same way
    println!("THE DOLLAR CAP (act 7, passed the same way): a CODEX_PLAN of root 0 restates its plan ({} lamports a period of {} slots) and names its cap of the dollar {} {} micro-dollars a period; until it executes the cap is 0 and no dollar the mine's deposits bring leaves root 0's vault.", s.p.at("genesis.planCap"), s.p.at("genesis.planPeriod"), s.p.at("warm.dollar").as_str().unwrap_or("(none)"), s.p.at("genesis.dollarCap"));
    println!("THE ROUTER'S MONTH (act 8, passed the same way): a TREASURY allowance of root 0 to the mine's miner {}, {} micro-dollars of the dollar a period of {} slots, pulled by the miner's own signature to pay the AI calls, standing until the polis revokes it.", s.p.at("warm.miner").as_str().unwrap_or("(none)"), s.p.at("router.monthCap"), s.p.at("genesis.planPeriod"));
    Ok(())
}

/// What stands of the sitting, read off the chain; and THE SEAL'S ROAD, written once root 0 stands as this sitting's.
fn status(s: &mut Sitting) -> Res<()> {
    let bins = s.binaries()?;
    for (name, bin) in &bins {
        let id = chain::program_id(&s.ids, name);
        match s.deployed(&id, bin)? {
            Some((slot, auth, same)) => println!("  {name:<7} {id}: deployed in slot {slot}, authority {}, {}", auth.map(|a| a.to_string()).unwrap_or("none".into()), if same { "this build's bytes" } else { "OTHER BYTES" }),
            None => println!("  {name:<7} {id}: not deployed"),
        }
    }
    let vault = chain::treasury_of(&s.p)?;
    let pool = s.pool_of()?;
    for (what, k) in [
        ("root 0", s.at.config),
        ("root 0's curve", s.at.curve),
        ("unit 1", poleis_client::registry_pda(chain::POSIVERSE)),
        ("root 0's vault", vault),
        ("the mine", s.at.mine),
        ("the board", s.at.board),
        ("the fund", s.at.fund),
        ("root 0's market", s.at.market_law),
        ("the pool's reserve", pool.reserve),
        ("the pool's mint", pool.mint),
        ("its fee vault", pool.vault),
        ("the pool", pool.pool),
    ] {
        println!("  {what:<18} {k}: {}", if s.rpc.account(&k.to_string())?.is_some() { "stands" } else { "not yet" });
    }
    println!("  the stake pool program {}: {}", poleis_press::pool::STAKE_POOL, if s.stake_pool_stands()? { "stands on this chain" } else { "not on this chain (the pool's birth not run here)" });
    // THE LAUNCH NAMES THE SEAL'S ROAD: written once root 0 stands as this sitting's, and printed with the line the lander takes it by
    if s.account(&s.at.config)?.is_none() {
        println!("  the seal's road   not yet: it is written once root 0 stands (press found)");
        return Ok(());
    }
    match s.root_is_ours().and_then(|_| s.seal_road()) {
        Ok((path, road)) => {
            println!("  the seal's road   {}: {road}", path.display());
            println!("    {}", seal::said(&road));
            println!("    the lander takes it by {}", seal::lander_line(&path));
        }
        Err(e) => println!("  the seal's road   not written: {e}"),
    }
    Ok(())
}

fn main() -> ExitCode {
    let all: Vec<String> = std::env::args().collect();
    match run(&all) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Fail::Usage(e)) => {
            eprintln!("press: {e}");
            ExitCode::from(2)
        }
        Err(Fail::Refused(e)) => {
            eprintln!("press: refused: {e}");
            ExitCode::from(1)
        }
        Err(Fail::Short(e)) => {
            eprintln!("press: refused: {e}");
            ExitCode::from(3)
        }
    }
}
