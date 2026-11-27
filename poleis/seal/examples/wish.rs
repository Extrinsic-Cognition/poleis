//! wish · A WISH, PLAYED on a practice chain: a validator of this machine holding this tree's own Poleis program, nothing past 127.0.0.1 (the way of
//! examples/practice.rs). His five steps of Nov 24 2026 15:26 UTC ("you start a wish * the 12 pentagons become fertile soil for SnowTracers to apply
//! (pentagons are necromancers...) * they get down working on the wish * the wish evolves into a neghiverse issuiing hexagons * hexagons aare rights to use
//! the product idk dude"), each played by the program's own acts and by nothing else, so that Swarmbook's own pages (polis.html?civ=<root>&program=..&rpc=..)
//! read every state off the chain: his 15:51 "it is not meant to be hardcoded". No stage is named on the chain: a step is only the acts it sends.
//! Stand-in keys only, made here and kept in the play's folder; none of his. Any wish: its words and its name are the caller's.
//!
//!   cargo run --example wish -- start --wish "<what should exist>" --name "<its name>" --rpc http://127.0.0.1:P --dir D
//!        a capital allocator starts it: the tree's config and its genesis in one transaction (the allocator alone in its chairs, the wish's own
//!        words its description, their hash its charter, a first sale of hexagons open); a first SnowTracer's sealed bid
//!   cargo run --example wish -- soil   --rpc .. --dir D    the sale settled, the first hexagon claimed, the genesis ratified; three more necromancers
//!        seated in three pentagons, one decision each; the door opened by a decision; three SnowTracers knock
//!   cargo run --example wish -- work   --rpc .. --dir D    two knocks admitted and one refused, a decision each; the allocator endows the treasury; a
//!        budget for a SnowTracer's work proposed and half voted (left open: a vote to be seen)
//!   cargo run --example wish -- grow   --rpc .. --dir D    the budget voted through, sealed and paid; two more knocks admitted; the door opened again, a rung up
//!   cargo run --example wish -- satellites --rpc .. --dir D    a pentagon of the wish becomes a swarm of its own (Build), a pentagon of that one
//!        another (Tools); then one question of the wish answered by the whole tree, leaves first, each seal its parent's pentagon's voice
//!   cargo run --example wish -- rights --rpc .. --dir D    one more decision proposed and left open, for a hexagon's holder to be seen with a voice;
//!        writes D/rider.json, that holder's stand-in key as the page's own seat store takes one
//!   cargo run --example wish -- said   --rpc .. --dir D    what the chain holds of it now, as JSON
//!
//! THE PUBLISHER STANDS ON DEVNET (Nov 24 2026 16:56:22 UTC, his "Yes, on devnet", to "Make it live on devnet: the business as a swarm, paid by one
//! war, up to 45 of your test SOL?"): the same birth on Solana's public test chain, where Swarmbook's own wall reads it. --cluster devnet reaches
//! https://api.devnet.solana.com alone, its genesis pinned and every call paced (the war's rehearsal's copy of the press's two modules; the press
//! itself never sends there); the cast's SOL comes by transfers a payer's keypair file signs (devnet's faucet is closed), only who acts at the
//! birth is funded, the first sale's windows are long enough for a public endpoint, and the folder's keys are kept (mode 600, never in git): they
//! hold the swarm's first pentagon and its first hexagon. Never mainnet.
//!   cargo run --example wish -- try    --wish .. --name .. --cluster devnet --rpc https://api.devnet.solana.com --payer <keypair.json>
//!        nothing sent: the birth's one transaction simulated against the chain's own program (would it land, its error, its units, its logs)
//!   cargo run --example wish -- start  --wish .. --name .. --cluster devnet --rpc .. --payer <keypair.json> --dir D
//!   cargo run --example wish -- seat   --cluster devnet --rpc .. --dir D    the sale settled, the first hexagon claimed, the genesis ratified, and no more
//!   cargo run --example wish -- books  --cluster devnet --rpc .. --dir D    what reached the treasury by no act of Poleis, written into its own books
//!   cargo run --example wish -- refund --cluster devnet --rpc .. --dir D --to <address> [--keep <lamports>]
//!        every key of the cast sends back what it holds beyond --keep (10,000,000 unless said)
//!
//! AN OLDER FUND HAS ITS ROAD (Nov 24 2026): a program upgraded over the fund of Sept 6 2026's build (706 bytes) refuses it at every fee site
//! until it is grown in place, once, by anyone (`grow_fund`, tag 57).
//!   cargo run --example wish -- fund   --rpc .. [--cluster devnet --payer <keypair.json>] [--grow]
//!        the fund as the chain holds it (its size, the older one or this tree's, its current option) and the growth's one transaction simulated;
//!        with --grow, sent when the simulation lands, and the fund read back. On a validator of this machine the payer is a stand-in the faucet pays

use poleis_client as p;
// the war's rehearsal's copy of the press's two modules: the same calls, and the one copy that reaches devnet (its genesis pinned, its calls paced)
use poleis_wars_rehearsal::rpc::{wait, Cluster, Rpc};
use poleis_wars_rehearsal::wire::{self, Key};
use poleis_treasury as pt;
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::time::Duration;

type Res<T> = Result<T, String>;
const SOL: u64 = 1_000_000_000;
/// the cast: who starts the wish, who carries the votes, three more necromancers, six SnowTracers
const NAMES: [&str; 11] = ["allocator", "crank", "n1", "n2", "n3", "s1", "s2", "s3", "s4", "s5", "s6"];
/// a seat at the door, and the budget for a SnowTracer's work
const DOOR_PRICE: u64 = SOL / 20;
const BUDGET: u64 = SOL / 20;

fn send(rpc: &Rpc, what: &str, ixs: Vec<Instruction>, keys: &[&Key]) -> Res<String> {
    let (w, first) = wire::signed(&ixs, &keys[0].address, &rpc.blockhash()?, keys)?;
    let sig = rpc.send(&w, true).map_err(|e| format!("{what}: {e}"))?;
    if sig != wire::base58(&first) {
        return Err(format!("{what}: another signature came back"));
    }
    wait(Duration::from_secs(60), || match rpc.statuses(&[sig.clone()])?.into_iter().next().flatten() {
        Some((_, e, c)) if e.is_null() && (c == "confirmed" || c == "finalized") => Ok(Some(())),
        Some((_, e, _)) if !e.is_null() => Err(format!("{what}: landed and failed: {e}")),
        _ => Ok(None),
    })?;
    println!("  {what:<58} {}…", &sig[..16]);
    Ok(sig)
}
fn until_slot(rpc: &Rpc, s: u64) -> Res<()> {
    wait(Duration::from_secs(1_800), || Ok((rpc.slot()? >= s).then_some(())))
}
/// a system transfer: [2u32 LE][lamports u64 LE], from (s,w) to (w)
fn transfer(from: Pubkey, to: Pubkey, lamports: u64) -> Instruction {
    let mut d = 2u32.to_le_bytes().to_vec();
    d.extend_from_slice(&lamports.to_le_bytes());
    Instruction::new_with_bytes(Pubkey::default(), &d, vec![AccountMeta::new(from, true), AccountMeta::new(to, false)])
}
/// SOL for a key of the cast: the practice chain's faucet, or on devnet a transfer the payer signs
fn give(rpc: &Rpc, payer: Option<&Key>, k: &Key, lamports: u64) -> Res<()> {
    match payer {
        Some(pk) => send(rpc, "  test SOL from the payer", vec![transfer(pk.address, k.address, lamports)], &[pk]).map(|_| ()),
        None => {
            let s = rpc.airdrop(&k.address.to_string(), lamports)?;
            wait(Duration::from_secs(60), || Ok(rpc.statuses(&[s.clone()])?.into_iter().next().flatten().map(|_| ())))
        }
    }
}
/// the tree's root is the wish's own: the first eight bytes of its words' hash (never 0, the meta-swarm's)
fn root_of(charter: &[u8; 32]) -> u64 {
    u64::from_le_bytes(charter[..8].try_into().unwrap()).max(1)
}
/// the birth's terms. The first sale's two windows and a decision's window, in slots: short on the practice chain; on devnet long enough for a
/// public endpoint's paced calls and for a person to see a vote
fn genesis_of(charter: [u8; 32], s0: u64, devnet: bool, allocator: Pubkey, name: &str, wish: &str) -> p::GenesisSpec {
    let (commit, reveal, window) = if devnet { (600, 1_200, 1_500) } else { (150, 200, pt::MIN_WINDOW_SLOTS) };
    // WISH_WINDOW_SLOTS: both windows of the first sale that long, so a page's own door can bid and reveal in them
    let (commit, reveal) = std::env::var("WISH_WINDOW_SLOTS").ok().and_then(|v| v.parse::<u64>().ok()).map_or((commit, reveal), |w| (w, 2 * w));
    p::GenesisSpec {
        charter, plan_cap: 10 * SOL, plan_period: 1_000_000, window, rung: 1, reserve: 1_000_000, deposit: 5_000_000,
        commit_end: s0 + commit, reveal_end: s0 + reveal, founders: vec![allocator], name: name.as_bytes().to_vec(), desc: wish.as_bytes().to_vec(),
    }
}
fn words_fit(wish: &str, name: &str) -> Res<()> {
    if wish.is_empty() || wish.len() > pt::MAX_DESC || name.is_empty() || name.len() > pt::MAX_NAME {
        return Err(format!("a wish is 1 to {} bytes and its name 1 to {}", pt::MAX_DESC, pt::MAX_NAME));
    }
    Ok(())
}
fn salt(n: u64, price: u64) -> [u8; 32] {
    let mut s = [0u8; 32];
    s[0] = n as u8 + 1;
    s[1..9].copy_from_slice(&price.to_le_bytes());
    s
}

/// the play's folder: its stand-in keys and what it remembers between steps (the tree's root, the next nonce, a decision left open)
struct Play {
    dir: String,
    keys: Vec<Key>,
    root: u64,
    nonce: u64,
    open: Option<String>,
}
impl Play {
    fn load(dir: &str) -> Res<Self> {
        let read = |f: &str| -> Res<Value> { serde_json::from_str(&std::fs::read_to_string(format!("{dir}/{f}")).map_err(|e| format!("{dir}/{f}: {e}"))?).map_err(|e| e.to_string()) };
        let (k, w) = (read("keys.json")?, read("wish.json")?);
        Ok(Self {
            dir: dir.into(),
            keys: NAMES.iter().map(|n| Key::from_cli_text(&k[*n].to_string())).collect::<Res<_>>()?,
            root: w["root"].as_str().and_then(|s| s.parse().ok()).ok_or("wish.json: no root")?,
            nonce: w["nonce"].as_u64().unwrap_or(1),
            open: w["open"].as_str().map(String::from),
        })
    }
    fn save(&self) -> Res<()> {
        std::fs::write(format!("{}/wish.json", self.dir), json!({ "root": self.root.to_string(), "nonce": self.nonce, "open": self.open }).to_string()).map_err(|e| e.to_string())
    }
    fn key(&self, name: &str) -> &Key {
        &self.keys[NAMES.iter().position(|n| *n == name).expect("a name of the cast")]
    }
    fn registry(&self, rpc: &Rpc) -> Res<p::RegistryView> {
        p::RegistryView::parse(&rpc.account(&p::registry_pda(self.root).to_string())?.ok_or("no registry: the wish is not started")?.data).ok_or("the registry does not read".into())
    }
    /// who has a voice now, read off the chain: every key of the cast that holds a pentagon or a hexagon
    fn voices(&self, rpc: &Rpc) -> Res<Vec<&Key>> {
        let r = self.registry(rpc)?;
        Ok(self.keys.iter().filter(|k| { let h = k.address.to_bytes(); r.pentagons.iter().any(|x| x.kind == pt::HOLDER_KEY && x.holder == h) || r.seats_of(pt::HOLDER_KEY, &h) > 0 }).collect())
    }
    /// a decision opened by the allocator: its hash
    fn propose(&mut self, rpc: &Rpc, what: &str, payload: &p::Payload) -> Res<[u8; 32]> {
        let manifest = poleis_seal::sha256(what.as_bytes());
        let h = p::proposal_hash(payload.class(), payload.origin(), &manifest, &payload.hash(), self.nonce);
        self.nonce += 1;
        self.save()?;
        let a = self.key("allocator");
        send(rpc, &format!("propose: {what}"), vec![p::propose_of(self.root, a.address, h, manifest, payload, &[self.root])], &[a])?;
        Ok(h)
    }
    fn vote(&self, rpc: &Rpc, h: &[u8; 32], voters: &[&Key]) -> Res<()> {
        let crank = self.key("crank");
        let signed: Vec<(Pubkey, u8, [u8; 64])> = voters.iter().map(|k| (k.address, pt::VOTE_PRO, k.sign(&p::vote_envelope(h, self.root, pt::VOTE_PRO)))).collect();
        for b in signed.chunks(4) {
            send(rpc, &format!("  {} voices say yes", b.len()), p::attest_batch(crank.address, *h, self.root, b).to_vec(), &[crank])?;
        }
        Ok(())
    }
    /// sealed (at once when no voice still to come can change it, else at the window's end) and carried out
    fn seal_and_execute(&self, rpc: &Rpc, h: &[u8; 32], payload: &p::Payload) -> Res<()> {
        let crank = self.key("crank");
        if send(rpc, "  seal", vec![p::seal(crank.address, *h, self.root)], &[crank]).is_err() {
            let s = p::SessionView::parse(&rpc.account(&p::session_pda(h, self.root).to_string())?.ok_or("no session")?.data).ok_or("no session reads")?;
            println!("  the seal waits for the window's end, slot {}", s.deadline);
            until_slot(rpc, s.deadline + 1)?;
            send(rpc, "  seal (the window closed)", vec![p::seal(crank.address, *h, self.root)], &[crank])?;
        }
        let s = p::SessionView::parse(&rpc.account(&p::session_pda(h, self.root).to_string())?.ok_or("no session")?.data).ok_or("no session reads")?;
        if s.state != pt::STATE_PASSED {
            return Err(format!("the swarm did not pass it: state {}", s.state));
        }
        send(rpc, "  execute", vec![p::execute_of(self.root, crank.address, *h, &[self.root], payload)], &[crank])?;
        Ok(())
    }
    /// who has a voice in one chamber of the tree now, read off the chain: every key of the cast that holds one of its pentagons or hexagons
    fn voices_of(&self, rpc: &Rpc, unit: u64) -> Res<Vec<&Key>> {
        let r = self.registry_of(rpc, unit)?;
        Ok(self.keys.iter().filter(|k| { let h = k.address.to_bytes(); r.pentagons.iter().any(|x| x.kind == pt::HOLDER_KEY && x.holder == h) || r.seats_of(pt::HOLDER_KEY, &h) > 0 }).collect())
    }
    fn registry_of(&self, rpc: &Rpc, unit: u64) -> Res<p::RegistryView> {
        p::RegistryView::parse(&rpc.account(&p::registry_pda(unit).to_string())?.ok_or("no such chamber")?.data).ok_or("the chamber's registry does not read".into())
    }
    /// the satellites of a chamber: the swarms seated in its pentagons ("pent can be polyehdrons recursively", his Aug 31 2026)
    fn crews_of(&self, rpc: &Rpc, unit: u64) -> Res<Vec<u64>> {
        Ok(self.registry_of(rpc, unit)?.pentagons.iter().filter(|x| x.kind == pt::HOLDER_UNIT).map(|x| pt::state::holder_unit(&x.holder)).collect())
    }
    /// A chamber's answer to a question, leaves first (his Sept 9 2026 "Recursive governance Two chambers. Every branch.", Sept 22 "the fractally
    /// recursive perfect bicameral direct democracy"): every swarm seated in one of its pentagons opens its own vote of the question, answers by
    /// this same rule and seals, and its seal is carried up as its pentagon's voice; then the chamber's own pentagons and hexagons vote and it
    /// seals, both halves or nothing. The program refuses any other order (`ChildMustSpeak`).
    fn answer(&self, rpc: &Rpc, h: &[u8; 32], unit: u64, parent: Option<u64>, depth: usize) -> Res<()> {
        let crank = self.key("crank");
        let pad = "  ".repeat(depth + 1);
        if let Some(up) = parent {
            if rpc.account(&p::session_pda(h, unit).to_string())?.is_none() {
                send(rpc, &format!("{pad}a satellite opens its own vote of it"), vec![p::convene(crank.address, *h, up, unit)], &[crank])?;
            }
        }
        for c in self.crews_of(rpc, unit)? {
            self.answer(rpc, h, c, Some(unit), depth + 1)?;
            send(rpc, &format!("{pad}its seal carried up as its pentagon's voice"), vec![p::mandate(crank.address, *h, unit, c)], &[crank])?;
        }
        let voters = self.voices_of(rpc, unit)?;
        let signed: Vec<(Pubkey, u8, [u8; 64])> = voters.iter().map(|k| (k.address, pt::VOTE_PRO, k.sign(&p::vote_envelope(h, unit, pt::VOTE_PRO)))).collect();
        for b in signed.chunks(4) {
            send(rpc, &format!("{pad}{} voices say yes", b.len()), p::attest_batch(crank.address, *h, unit, b).to_vec(), &[crank])?;
        }
        if send(rpc, &format!("{pad}seal"), vec![p::seal(crank.address, *h, unit)], &[crank]).is_err() {
            let s = p::SessionView::parse(&rpc.account(&p::session_pda(h, unit).to_string())?.ok_or("no session")?.data).ok_or("no session reads")?;
            until_slot(rpc, s.deadline + 1)?;
            send(rpc, &format!("{pad}seal (the window closed)"), vec![p::seal(crank.address, *h, unit)], &[crank])?;
        }
        let s = p::SessionView::parse(&rpc.account(&p::session_pda(h, unit).to_string())?.ok_or("no session")?.data).ok_or("no session reads")?;
        if s.state != pt::STATE_PASSED {
            return Err(format!("chamber {unit} did not pass it: state {}", s.state));
        }
        Ok(())
    }
    /// one whole decision of any chamber of the tree: proposed by one of its own, answered leaves first, carried out
    fn decide_in(&mut self, rpc: &Rpc, what: &str, payload: &p::Payload) -> Res<()> {
        let origin = payload.origin();
        let manifest = poleis_seal::sha256(what.as_bytes());
        let h = p::proposal_hash(payload.class(), origin, &manifest, &payload.hash(), self.nonce);
        self.nonce += 1;
        self.save()?;
        let proposer = *self.voices_of(rpc, origin)?.first().ok_or("the chamber has no voice of this cast")?;
        send(rpc, &format!("propose: {what}"), vec![p::propose_of(self.root, proposer.address, h, manifest, payload, &[origin])], &[proposer])?;
        self.answer(rpc, &h, origin, None, 0)?;
        let crank = self.key("crank");
        send(rpc, "  carried out", vec![p::execute_of(self.root, crank.address, h, &[origin], payload)], &[crank])?;
        Ok(())
    }
    /// the whole tree, read off the chain: every chamber, its place, its two halves
    fn tree(&self, rpc: &Rpc) -> Res<Vec<Value>> {
        let (mut out, mut todo) = (Vec::new(), vec![self.root]);
        while let Some(u) = todo.pop() {
            let r = self.registry_of(rpc, u)?;
            let crews = self.crews_of(rpc, u)?;
            out.push(json!({ "chamber": u.to_string(), "name": r.name, "level": r.level, "inside": if u == self.root { Value::Null } else { json!(r.parent.to_string()) },
                "pentagons_held": r.filled_chairs(), "of_them_swarms": crews.len(), "hexagons_held": r.total_seats,
                "treasury_lamports": rpc.balance(&p::treasury_pda(u).to_string())?.saturating_sub(rpc.rent(pt::state::Treasury::SIZE)?) }));
            todo.extend(crews);
        }
        Ok(out)
    }
    /// one whole decision of the swarm: proposed, every voice it has now saying yes, sealed, carried out
    fn decide(&mut self, rpc: &Rpc, what: &str, payload: &p::Payload) -> Res<()> {
        let h = self.propose(rpc, what, payload)?;
        let voters = self.voices(rpc)?;
        self.vote(rpc, &h, &voters)?;
        self.seal_and_execute(rpc, &h, payload)
    }
    /// ratified: the tree's authority is no longer its starter's key (ratify_genesis hands it to an address nobody holds)
    fn ratified(&self, rpc: &Rpc) -> Res<Option<bool>> {
        Ok(rpc.account(&p::config_pda_of(self.root).to_string())?.and_then(|a| p::ConfigView::parse(&a.data)).map(|c| c.authority != self.key("allocator").address))
    }
    /// what the chain holds of the wish now
    fn said(&self, rpc: &Rpc) -> Res<Value> {
        let r = self.registry(rpc)?;
        let pool = rpc.account(&p::pool_pda(self.root).to_string())?.and_then(|a| p::PoolView::parse(&a.data));
        let knocking: Vec<&str> = NAMES.iter().zip(&self.keys).filter(|(_, k)| rpc.account(&p::knock_pda(self.root, &k.address).to_string()).ok().flatten().is_some()).map(|(n, _)| *n).collect();
        let rent = rpc.rent(pt::state::Treasury::SIZE)?;
        let ratified = self.ratified(rpc)?;
        Ok(json!({
            "root": self.root.to_string(), "registry": p::registry_pda(self.root).to_string(), "treasury": p::treasury_pda(self.root).to_string(),
            "name": r.name, "words": r.desc, "ratified": ratified,
            "pentagons_held": r.filled_chairs(), "hexagons_held": r.total_seats, "hexagons_of_its_rung": pt::ladder_hexagons(r.rung as usize), "rung": r.rung,
            "at_the_door": pool.as_ref().map(|x| x.open - x.filled), "door_price": pool.as_ref().map(|x| x.price), "knocking": knocking,
            "treasury_lamports": rpc.balance(&p::treasury_pda(self.root).to_string())?.saturating_sub(rent), "decisions_opened": self.nonce - 1,
            "chambers": self.tree(rpc)?,
        }))
    }
}

/// nothing sent: the birth's one transaction as this chain's own program would take it, the payer standing in for the allocator
fn try_birth(rpc: &Rpc, payer: &Key, wish: &str, name: &str) -> Res<()> {
    words_fit(wish, name)?;
    let charter = poleis_seal::sha256(wish.as_bytes());
    let root = root_of(&charter);
    let g = genesis_of(charter, rpc.slot()?, true, payer.address, name, wish);
    let (w, _) = wire::signed(&[p::init_config_of(root, payer.address, Pubkey::default()), p::open_genesis_of(root, payer.address, &g)], &payer.address, &rpc.blockhash()?, &[payer])?;
    // with the registry as the transaction would leave it, read back by this tree's own client: the chain's program may be an older build
    // (devnet's is Sept 6 2026's), and a birth it takes must also be a birth this tree reads the same
    let v = rpc.call(
        "simulateTransaction",
        json!([wire::b64(&w), { "encoding": "base64", "sigVerify": true, "commitment": "confirmed", "accounts": { "encoding": "base64", "addresses": [p::registry_pda(root).to_string()] } }]),
    )?;
    let r = &v["value"];
    let written = r["accounts"][0]["data"][0]
        .as_str()
        .and_then(|b| base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b).ok())
        .and_then(|d| p::RegistryView::parse(&d))
        .map(|x| json!({ "name": x.name, "words": x.desc, "rung": x.rung, "pentagons_held": x.filled_chairs(), "hexagons_held": x.total_seats, "hexagons_of_its_rung": pt::ladder_hexagons(x.rung as usize) }));
    println!(
        "{}",
        json!({ "program": p::PROGRAM_ID.to_string(), "root": root.to_string(), "would_land": r["err"].is_null(), "error": r["err"], "units": r["unitsConsumed"], "logs": r["logs"],
            "the_registry_it_would_write_as_this_tree_reads_it": written })
    );
    Ok(())
}

fn start(rpc: &Rpc, payer: Option<&Key>, dir: &str, wish: &str, name: &str) -> Res<()> {
    words_fit(wish, name)?;
    // a wish is started once: its folder's keys hold what it starts with
    if std::path::Path::new(&format!("{dir}/keys.json")).exists() {
        return Err(format!("{dir} already holds a play's keys: a wish is started once"));
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let ks: Vec<Key> = NAMES.iter().map(|_| Key::fresh()).collect::<Res<_>>()?;
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let text = json!(NAMES.iter().zip(&ks).map(|(n, k)| (n.to_string(), serde_json::from_str::<Value>(&k.cli_text()).unwrap())).collect::<serde_json::Map<_, _>>()).to_string();
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(format!("{dir}/keys.json")).map_err(|e| format!("{dir}/keys.json: {e}"))?;
        f.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    }
    let charter = poleis_seal::sha256(wish.as_bytes());
    let root = root_of(&charter);
    let play = Play { dir: dir.into(), keys: ks, root, nonce: 1, open: None };
    play.save()?;
    let devnet = payer.is_some();
    println!("{} {}, program {}, genesis {}", if devnet { "devnet" } else { "practice chain" }, rpc.url, p::PROGRAM_ID, rpc.call("getGenesisHash", json!([]))?);
    // the practice chain's faucet fills the whole cast; on devnet only who acts at the birth, with what the birth takes and a margin that comes back
    let cast: &[(&str, u64)] = if devnet {
        &[("allocator", SOL / 2), ("crank", SOL / 20), ("s1", SOL / 10)]
    } else {
        &[("allocator", 20 * SOL), ("crank", 5 * SOL), ("s1", SOL), ("s2", SOL), ("s3", SOL), ("s4", SOL), ("s5", SOL), ("s6", SOL)]
    };
    for (n, l) in cast {
        give(rpc, payer, play.key(n), *l)?;
    }
    let (a, s1) = (play.key("allocator"), play.key("s1"));
    let g = genesis_of(charter, rpc.slot()?, devnet, a.address, name, wish);
    send(rpc, "the wish is started (its tree and its genesis)", vec![p::init_config_of(root, a.address, Pubkey::default()), p::open_genesis_of(root, a.address, &g)], &[a])?;
    send(rpc, "a first SnowTracer's sealed bid for a hexagon", vec![p::commit_seat_bid(s1.address, root, 0, 0, p::seat_commitment(2_000_000, 0, &salt(0, 2_000_000), &s1.address))], &[s1])?;
    println!("{}", play.said(rpc)?);
    Ok(())
}

/// the first sale closed and the birth made good: the bid revealed, the sale settled, the first hexagon claimed, the genesis ratified
fn seat(rpc: &Rpc, play: &Play) -> Res<()> {
    let root = play.root;
    let sale = p::SaleView::parse(&rpc.account(&p::sale_pda(root, 0).to_string())?.ok_or("no genesis sale")?.data).ok_or("the sale does not read")?;
    let (crank, s1) = (play.key("crank"), play.key("s1"));
    until_slot(rpc, sale.commit_end + 1)?;
    send(rpc, "the bid revealed", vec![p::reveal_seat_bid(s1.address, root, 0, 0, 2_000_000, salt(0, 2_000_000))], &[s1])?;
    until_slot(rpc, sale.reveal_end + 1)?;
    send(rpc, "the sale settled", vec![p::settle_seats(crank.address, root, 0)], &[crank])?;
    send(rpc, "the first hexagon claimed", vec![p::claim_seat(s1.address, root, 0, 0, root)], &[s1])?;
    send(rpc, "the genesis ratified: nobody holds its key", vec![p::ratify_genesis_of(root, crank.address)], &[crank])?;
    Ok(())
}

/// devnet: every key of the cast sends back what it holds beyond `keep` (and its own fee), each signing its own transfer
fn refund(rpc: &Rpc, dir: &str, to: Pubkey, keep: u64) -> Res<()> {
    const FEE: u64 = 5_000;
    let play = Play::load(dir)?;
    let mut back = 0u64;
    for (n, k) in NAMES.iter().zip(&play.keys) {
        let held = rpc.balance(&k.address.to_string())?;
        if held > keep + FEE {
            send(rpc, &format!("{n} sends back"), vec![transfer(k.address, to, held - keep - FEE)], &[k])?;
            back += held - keep - FEE;
        }
    }
    println!("{}", json!({ "sent_back_lamports": back, "to": to.to_string(), "kept_by_each": keep }));
    Ok(())
}

/// AN OLDER FUND HAS ITS ROAD: the fund read, the growth simulated, and with `grow` sent. Nothing is sent to an account that is not the older fund.
fn fund(rpc: &Rpc, payer: Option<&Key>, grow: bool) -> Res<()> {
    let at = p::fund_pda().to_string();
    let read = |rpc: &Rpc| -> Res<Value> {
        let a = rpc.account(&at)?.ok_or("no account at the fund's address")?;
        let d = &a.data;
        let current: [u8; 32] = d.get(p::Fund::OFF_CURRENT..p::Fund::OFF_CURRENT + 32).and_then(|x| x.try_into().ok()).unwrap_or([0u8; 32]);
        let tail = (d.len() == p::Fund::SIZE).then(|| p::FundView::parse(d)).flatten();
        Ok(json!({ "address": at, "owner": a.owner, "bytes": d.len(), "lamports": a.lamports,
            "older": a.owner == p::PROGRAM_ID.to_string() && d.len() == p::Fund::OLD_SIZE,
            "current": current.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "current_recipient": tail.as_ref().map(|f| f.current_recipient.to_string()), "owed": tail.as_ref().map(|f| f.owed) }))
    };
    let before = read(rpc)?;
    if before["older"] != true {
        println!("{}", json!({ "fund": before, "grow": "nothing to grow: not the older fund" }));
        return Ok(());
    }
    let current: [u8; 32] = (0..32).map(|i| u8::from_str_radix(&before["current"].as_str().unwrap()[2 * i..2 * i + 2], 16).unwrap()).collect::<Vec<_>>().try_into().unwrap();
    let stand_in;
    let k = match payer {
        Some(k) => k,
        None => {
            stand_in = Key::fresh()?;
            give(rpc, None, &stand_in, SOL / 10)?;
            // the faucet's transfer seen at the level the simulation reads, or the payer is an account not found
            wait(Duration::from_secs(60), || Ok((rpc.balance(&stand_in.address.to_string())? > 0).then_some(())))?;
            &stand_in
        }
    };
    let ix = p::grow_fund(k.address, (current != [0u8; 32]).then_some(current));
    let (w, _) = wire::signed(&[ix.clone()], &k.address, &rpc.blockhash()?, &[k])?;
    let (err, units, logs) = rpc.simulate(&w)?;
    let mut said = json!({ "fund": before, "payer": k.address.to_string(), "added_rent": rpc.rent(p::Fund::SIZE)? - rpc.rent(p::Fund::OLD_SIZE)?,
        "simulated": { "would_land": err.is_null(), "error": err, "units": units, "logs": logs } });
    if grow && err.is_null() {
        let sig = send(rpc, "the older fund grown", vec![ix], &[k])?;
        said["sent"] = json!({ "signature": sig, "units": rpc.transaction(&sig)?.and_then(|t| t.2) });
        said["after"] = read(rpc)?;
    }
    println!("{said}");
    Ok(())
}

fn soil(rpc: &Rpc, dir: &str) -> Res<()> {
    let mut play = Play::load(dir)?;
    let root = play.root;
    // the first sale is closed here unless `seat` already closed it
    if play.ratified(rpc)? != Some(true) {
        seat(rpc, &play)?;
    }
    // three more necromancers take three pentagons: the first free chairs, a decision of the swarm each
    for n in ["n1", "n2", "n3"] {
        let chair = play.registry(rpc)?.pentagons.iter().position(|x| x.kind == pt::HOLDER_EMPTY).ok_or("no pentagon is free")? as u8;
        let holder = play.key(n).address.to_bytes();
        play.decide(rpc, &format!("seat a necromancer in pentagon {chair}"), &p::Payload::Attach { origin: root, chair, holder, kind: pt::HOLDER_KEY, term: 0, retainer: 0 })?;
    }
    play.decide(rpc, "open the door to SnowTracers", &p::Payload::Door { origin: root, price: DOOR_PRICE })?;
    for n in ["s2", "s3", "s4"] {
        let k = play.key(n);
        send(rpc, &format!("{n} knocks"), vec![p::knock(k.address, root)], &[k])?;
    }
    println!("{}", play.said(rpc)?);
    Ok(())
}

fn work(rpc: &Rpc, dir: &str) -> Res<()> {
    let mut play = Play::load(dir)?;
    let root = play.root;
    for (n, admit) in [("s2", true), ("s3", true), ("s4", false)] {
        let holder = play.key(n).address.to_bytes();
        if admit {
            play.decide(rpc, &format!("admit {n}"), &p::Payload::Admit { origin: root, holder, root })?;
        } else {
            play.decide(rpc, &format!("refuse {n}"), &p::Payload::Refuse { origin: root, holder })?;
        }
    }
    {
        let a = play.key("allocator");
        send(rpc, "the allocator endows the wish's treasury", vec![p::endow(a.address, root, 2 * SOL)], &[a])?;
    }
    // a budget for a SnowTracer's work, proposed and half voted: left open for the page to show a vote
    let payee = play.key("s2").address;
    let h = play.propose(rpc, "a budget for a SnowTracer's work", &p::Payload::Budget { origin: root, address: payee, amount: BUDGET })?;
    let voters = play.voices(rpc)?;
    play.vote(rpc, &h, &voters[..voters.len() / 2])?;
    play.open = Some(h.iter().map(|b| format!("{b:02x}")).collect());
    play.save()?;
    println!("{}", play.said(rpc)?);
    Ok(())
}

fn hash_of(hex: &str) -> Res<[u8; 32]> {
    let v: Vec<u8> = (0..hex.len() / 2).map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).map_err(|e| e.to_string())).collect::<Res<_>>()?;
    v.try_into().map_err(|_| "a decision's hash is 32 bytes".into())
}

fn grow(rpc: &Rpc, dir: &str) -> Res<()> {
    let mut play = Play::load(dir)?;
    let root = play.root;
    let h = hash_of(&play.open.clone().ok_or("no decision was left open: run work first")?)?;
    let payee = play.key("s2").address;
    {
        let voters = play.voices(rpc)?;
        play.vote(rpc, &h, &voters[voters.len() / 2..])?;
        play.seal_and_execute(rpc, &h, &p::Payload::Budget { origin: root, address: payee, amount: BUDGET })?;
    }
    play.open = None;
    play.save()?;
    for n in ["s5", "s6"] {
        let k = play.key(n);
        send(rpc, &format!("{n} knocks"), vec![p::knock(k.address, root)], &[k])?;
    }
    for n in ["s5", "s6"] {
        let holder = play.key(n).address.to_bytes();
        play.decide(rpc, &format!("admit {n}"), &p::Payload::Admit { origin: root, holder, root })?;
    }
    play.decide(rpc, "open the door again, a rung up", &p::Payload::Door { origin: root, price: DOOR_PRICE })?;
    println!("{}", play.said(rpc)?);
    Ok(())
}

/// THE SATELLITES (his Nov 24 2026 17:5x UTC, over the flat play: "this is lacking when the satellite gets added? this is not yet recursive in this
/// example there is no fucking unit1 tree or unit2 etc etc"). A pentagon of the wish becomes a swarm of its own, and a pentagon of that one another:
/// the wish founds a branch in a free pentagon (a decision of both its halves), the branch founds one inside itself (its own decision, by its own
/// chairs: it has no floor yet), and then one question of the wish is answered by the whole tree, leaves first: the innermost seals, its seal is its
/// parent's pentagon's voice, that one seals, its seal is the wish's pentagon's voice, the wish's own pentagons and hexagons vote, it seals. The
/// branches' names are his own (his Sept 9 2026 edit of poleis.us: "The branches Swarm Counsel Research Build Systems Design Tools").
fn satellites(rpc: &Rpc, dir: &str) -> Res<()> {
    let mut play = Play::load(dir)?;
    let root = play.root;
    let (build, tools) = (root.wrapping_add(1), root.wrapping_add(2));
    let spec = |child: u64, chair: u8, name: &str, desc: &str, founders: Vec<Pubkey>| p::FoundSpec {
        child, chair, charter: poleis_seal::sha256(desc.as_bytes()), window: pt::MIN_WINDOW_SLOTS, plan_cap: SOL, plan_period: 1_000_000, founders,
        name: name.as_bytes().to_vec(), desc: desc.as_bytes().to_vec(),
    };
    let free = |play: &Play, unit: u64| -> Res<u8> { Ok(play.registry_of(rpc, unit)?.pentagons.iter().rposition(|x| x.kind == pt::HOLDER_EMPTY).ok_or("no pentagon is free")? as u8) };
    // unit 1: the SnowTracers who were let in form the branch that builds
    let chair = free(&play, root)?;
    let founders = ["s2", "s3", "s5"].iter().map(|n| play.key(n).address).collect();
    play.decide_in(rpc, "a pentagon of the wish becomes a swarm: Build", &p::Payload::Found { origin: root, spec: spec(build, chair, "Build", "The wish's branch that builds it.", founders) })?;
    // unit 2, inside unit 1, by unit 1's own chairs
    let chair = free(&play, build)?;
    let founders = ["s5", "s6"].iter().map(|n| play.key(n).address).collect();
    play.decide_in(rpc, "a pentagon of Build becomes a swarm: Tools", &p::Payload::Found { origin: build, spec: spec(tools, chair, "Tools", "Build's branch that makes its tools.", founders) })?;
    // one question of the wish, answered by the whole tree, leaves first
    play.decide_in(rpc, "the wish gives its branch a budget", &p::Payload::Transfer { origin: root, dest: build, amount: SOL / 2 })?;
    println!("{}", play.said(rpc)?);
    Ok(())
}

fn rights(rpc: &Rpc, dir: &str) -> Res<()> {
    let mut play = Play::load(dir)?;
    let root = play.root;
    let r = play.registry(rpc)?;
    let h = play.propose(rpc, "say the wish again, in its holders' words", &p::Payload::Name { origin: root, name: r.name.as_bytes().to_vec(), desc: r.desc.as_bytes().to_vec() })?;
    play.open = Some(h.iter().map(|b| format!("{b:02x}")).collect());
    play.save()?;
    // a hexagon's holder, for the page's own seat store: a stand-in key of this play, of a chain of this machine
    std::fs::write(format!("{dir}/rider.json"), play.key("s2").cli_text()).map_err(|e| e.to_string())?;
    println!("{}", play.said(rpc)?);
    Ok(())
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let o = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let r = (|| -> Res<()> {
        let cluster = Cluster::parse(&o("--cluster").unwrap_or_else(|| "localnet".into()))?;
        if cluster.name() == "mainnet" {
            return Err("a wish is played on a practice chain or founded on devnet: never on mainnet by this hand".into());
        }
        let devnet = cluster.name() == "devnet";
        let rpc = Rpc::new(&o("--rpc").ok_or("no --rpc")?, cluster)?;
        // held to its genesis before anything is read or sent
        rpc.check_cluster()?;
        let payer = match o("--payer") {
            Some(f) if devnet => Some(Key::from_cli_text(&std::fs::read_to_string(&f).map_err(|e| format!("--payer {f}: {e}"))?)?),
            _ => None,
        };
        let need_payer = || payer.as_ref().ok_or("--payer <keypair.json>: on devnet the cast's SOL comes from a payer's own keypair file".to_string());
        let dir = match a.first().map(String::as_str) {
            Some("try") | Some("fund") => String::new(),
            _ => o("--dir").ok_or("no --dir")?,
        };
        match a.first().map(String::as_str) {
            Some("try") => try_birth(&rpc, need_payer()?, &o("--wish").ok_or("no --wish")?, &o("--name").ok_or("no --name")?),
            Some("start") => start(&rpc, if devnet { Some(need_payer()?) } else { None }, &dir, &o("--wish").ok_or("no --wish")?, &o("--name").ok_or("no --name")?),
            Some("seat") => {
                let play = Play::load(&dir)?;
                seat(&rpc, &play)?;
                play.said(&rpc).map(|v| println!("{v}"))
            }
            Some("refund") => {
                let to: Pubkey = o("--to").ok_or("no --to")?.parse().map_err(|e| format!("--to: {e:?}"))?;
                refund(&rpc, &dir, to, o("--keep").map(|k| k.parse().map_err(|_| "--keep: not a number of lamports".to_string())).transpose()?.unwrap_or(10_000_000))
            }
            // what came into the treasury by no act of Poleis (a war's fees paid to it) written into Poleis's own books: anyone may ask it
            Some("books") => {
                let play = Play::load(&dir)?;
                send(&rpc, "the treasury's gifts booked", vec![p::sync_endowment(play.root)], &[play.key("crank")])?;
                play.said(&rpc).map(|v| println!("{v}"))
            }
            Some("soil") => soil(&rpc, &dir),
            Some("work") => work(&rpc, &dir),
            Some("grow") => grow(&rpc, &dir),
            Some("satellites") => satellites(&rpc, &dir),
            Some("rights") => rights(&rpc, &dir),
            Some("said") => Play::load(&dir)?.said(&rpc).map(|v| println!("{v}")),
            Some("fund") => {
                let payer = o("--payer").map(|f| Key::from_cli_text(&std::fs::read_to_string(&f).map_err(|e| format!("--payer {f}: {e}"))?)).transpose()?;
                if devnet && payer.is_none() {
                    return Err("--payer <keypair.json>: on devnet the growth's rent comes from a payer's own keypair file".into());
                }
                fund(&rpc, payer.as_ref(), a.iter().any(|x| x == "--grow"))
            }
            _ => Err("start --wish .. --name .. | soil | work | grow | satellites | rights | said | try | seat | books | refund | fund [--grow]".into()),
        }
    })();
    if let Err(e) = r {
        eprintln!("wish: {e}");
        std::process::exit(1);
    }
}
