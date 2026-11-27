//! THE LAUNCH NAMES THE SEAL'S ROAD (Nov 23 2026, lane/the-launch-names-the-seals-road; his "Wire it in", ~14:00 UTC, to "Wire the seal's
//! road into the launch press instead of using devnet?"). A RELEASE IS A VOTE (poleis/seal/A-RELEASE-IS-A-VOTE-LANDING.md) gave the house one
//! reader of the seal, `poleis-seal read <commit> --road R`, which the lander's ship asks once a road is named (`LANDER_SEAL_ROAD`); its owed
//! item 1 was the road. The press writes it once root 0 stands as this sitting's: `seal-road.json` beside the record, the three fields the
//! reader takes (`{rpc, program, polis}`), the same road on the record and on the birth certificate. The lander takes it by
//! `LANDER_SEAL_ROAD=<its path>` in its install: his hand, never the press's.
//!
//! THE POLIS IS THE BRANCH THAT RUNS THE HOUSE, UNIT 1 (THE BRANCH VOTES ITS OWN CODE, Nov 23 2026: his 20:13 UTC "what? do you not
//! understand this" over the figure "Vote the why, bet the where, vote the how", whose last row is "the vote: fast and small, each branch its
//! own; the agents act inside the budget", "one branch's own code" among its things; then his "Yes, fix it" to "Move code updates down to the
//! agents' fast vote?"). An update of the house's code is the how: his posiverse's agents, unit 1's chairs, propose it and vote it, two of
//! three, and no seat holder of root 0 votes code (the why, the slow vote of both chambers, is the law, who is let in, what the Fund pays
//! for). Until that word the road named root 0 and every seat holder voted every update.
//!
//! WHO MAY PROPOSE THERE, by the program: "The proposer is a member of the origin unit" (proposal.rs, `expect_member`), a member being "a key
//! holding hexagon seats, or a key in a live chair" (ix/mod.rs), and nothing is proposed before the tree is a council (`expect_council`).
//! Unit 1 is born with its agents' keys in its chairs and no floor, so its members are its agents: one proposes by its own `propose` (it pays
//! the proposal's rent, which comes home at close), and they vote by their envelopes, which `press crank` lands. `press release` writes the
//! agents' work order: the proposal for the one who proposes, and every agent's envelope.
//!
//! ANY MEMBER PROPOSES, THE SWARM DECIDES (Nov 23 2026, his 17:52 UTC "you keep getting it wrong, the swarm votes on updates, you dont have
//! it linear with me and keystones", then his "Yes, build it" to "Take your key and your press out of the update road?"). Until that day the
//! params named `seal.proposers`, "the keys his key names", and the reader refused every seal of anyone else's proposal: his key stood in
//! front of the swarm's vote. The road now names nobody who may propose: any member of root 0 (an owner of its floor's seats) proposes a
//! release, the whole tree seals it, and the house takes the seal whoever proposed it. A params file that still names `seal.proposers` is
//! refused by words that say why (PROPOSERS_NAMED), as one naming `keystones` is.
//!
//! THE NODE IS THE CLUSTER'S: on mainnet `seal.rpc`, an https node named by its host alone (the road is a file the lander reads, and a path,
//! a query or a user may carry a key; the sitting's own endpoint is never kept, for it may carry one); on a rehearsal's chain, the sitting's
//! own validator on this machine.

use crate::chain::Sitting;
use crate::params::Params;
use crate::rpc::{check_url, Cluster};
use crate::Res;
use poleis_client::ROOT_UNIT;
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::path::{Path, PathBuf};

/// The road's file, beside the sitting's record.
pub const FILE: &str = "seal-road.json";
/// The polis a release is sealed by: unit 1, the branch that runs the house, its agents (THE BRANCH VOTES ITS OWN CODE).
pub const POLIS: u64 = crate::chain::POSIVERSE;
/// What the press says of who proposes a release: the branch's agents, nobody named.
pub const ANY_MEMBER: &str = "the branch that runs the house votes its own code: unit 1's agents propose a release and vote it, two of three, fast; no seat holder votes code, no key names who proposes, and the house takes the seal whichever agent proposed it";
/// The words a params file that names `seal.proposers` is refused by.
pub const PROPOSERS_NAMED: &str = "params seal.proposers: no key names who proposes a release; the branch's agents propose and the swarm decides (his Nov 23 2026 \"the swarm votes on updates\"). Take the line out";

/// The node the road reads the chain at: on mainnet the params' `seal.rpc`, https and its host alone; on a rehearsal's chain the sitting's
/// own validator (`sitting_rpc`, on this machine).
pub fn node(p: &Params, sitting_rpc: &str) -> Res<String> {
    match p.cluster {
        Cluster::Localnet => {
            check_url(sitting_rpc, Cluster::Localnet)?;
            Ok(sitting_rpc.trim_end_matches('/').to_string())
        }
        Cluster::Mainnet => {
            let what = "params seal.rpc: the node the house's lander reads a release's seal at, https:// and its host alone";
            if p.is_placeholder("seal.rpc") {
                return Err(format!("params seal.rpc is not his yet: {}", p.at("seal.rpc")));
            }
            let url = p.at("seal.rpc").as_str().ok_or(what)?;
            check_url(url, Cluster::Mainnet).map_err(|e| format!("{what}: {e}"))?;
            let rest = &url["https://".len()..];
            let host = rest.strip_suffix('/').unwrap_or(rest);
            if host.is_empty() || host.contains(['/', '?', '#', '%']) {
                return Err(format!("{what}: {url:?} names more than a host (a path, a query or a fragment may carry a key, and the road is a file the lander reads)"));
            }
            Ok(format!("https://{host}"))
        }
    }
}

/// The road's params held before anything starts (`chain::params_checked`): mainnet's node, and on every chain no `seal.proposers`.
pub fn checked(p: &Params) -> Res<()> {
    if !p.at("seal.proposers").is_null() {
        return Err(PROPOSERS_NAMED.into());
    }
    if p.cluster == Cluster::Mainnet {
        node(p, "")?;
    }
    Ok(())
}

/// THE ROAD, as `poleis-seal read` takes it: its node, the sitting's own Poleis, unit 1; nothing else, and nobody who may propose.
pub fn road(p: &Params, poleis: &Pubkey, sitting_rpc: &str) -> Res<Value> {
    checked(p)?;
    Ok(json!({ "rpc": node(p, sitting_rpc)?, "program": poleis.to_string(), "polis": POLIS }))
}

/// What the press says of who proposes on a road: any member, the swarm decides (a road that names proposers, the reader refuses).
pub fn said(road: &Value) -> String {
    if road.get("proposers").is_some() {
        return "this road names proposers: poleis-seal refuses it (no key names who proposes a release); write it again with press found".into();
    }
    ANY_MEMBER.to_string()
}

/// How the lander takes the road: his hand, in its install.
pub fn lander_line(path: &Path) -> String {
    format!("LANDER_SEAL_ROAD={} in the lander's install (his hand: the press names it nowhere else)", path.display())
}

/// The road written into `dir`, a new file and then a rename (the same road is not written again): its path.
pub fn write(dir: &Path, road: &Value) -> Res<PathBuf> {
    let path = dir.join(FILE);
    let text = serde_json::to_string_pretty(road).map_err(|e| e.to_string())? + "\n";
    if std::fs::read_to_string(&path).ok().as_deref() == Some(text.as_str()) {
        return Ok(path);
    }
    let part = dir.join(format!("{FILE}.part"));
    std::fs::write(&part, &text).map_err(|e| format!("{}: {e}", part.display()))?;
    std::fs::rename(&part, &path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// The kind of a release, A RELEASE IS A VOTE's (poleis/seal): a SIGNAL kind the program does not execute; the house carries it out.
pub const SIGNAL_RELEASE: u8 = 0x52;
/// The nonces a release of one commit is read at, the reader's: a rejected or closed proposal is passed over, the next nonce taken.
pub const RELEASE_NONCES: u64 = 8;

/// The release of one commit in one polis at one nonce, named as the seal's reader names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub commit: String,
    pub polis: u64,
    pub nonce: u64,
    pub class: u8,
    pub manifest: [u8; 32],
    pub payload: Vec<u8>,
    pub hash: [u8; 32],
}

/// THE RELEASE'S NAME, the reader's: class SIGNAL|CONSENT, origin the polis, manifest the commit's bytes (20 for SHA-1, 32 for SHA-256)
/// zero-padded to 32, payload [SIGNAL, 0x52, the commit's bytes], impacted [the polis]; its hash the client's `proposal_hash` of those and the nonce.
pub fn release_in(commit: &str, polis: u64, nonce: u64) -> Res<Release> {
    use poleis_treasury::{CLASS_CONSENT, CLASS_SIGNAL};
    let c = commit.to_ascii_lowercase();
    if !(c.len() == 40 || c.len() == 64) || !c.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("--commit {commit}: a commit git names, 40 or 64 hex"));
    }
    let bytes: Vec<u8> = (0..c.len()).step_by(2).map(|i| u8::from_str_radix(&c[i..i + 2], 16).unwrap()).collect();
    let mut manifest = [0u8; 32];
    manifest[..bytes.len()].copy_from_slice(&bytes);
    let payload = [&[CLASS_SIGNAL, SIGNAL_RELEASE][..], &bytes[..]].concat();
    let class = CLASS_SIGNAL | CLASS_CONSENT;
    let hash = poleis_client::proposal_hash(class, polis, &manifest, &crate::wire::sha256(&payload), nonce);
    Ok(Release { commit: c, polis, nonce, class, manifest, payload, hash })
}

/// The release of `commit` in the house's branch (unit 1) at `nonce`.
pub fn release(commit: &str, nonce: u64) -> Res<Release> {
    release_in(commit, POLIS, nonce)
}

/// THE AGENTS' WORK ORDER for one release: every agent's envelope (its PRO in unit 1, a message it signs and hands to `press crank` as
/// `{release: <the proposal's hash>, unit: 1, voter, signature}`), and, when one proposes, its own `propose` transaction (one signer, the
/// agent, its fee payer and the rent's, the blockhash blank, in the prepared format). No seat holder is in it.
pub fn release_set(p: &Params, r: &Release, agents: &[Pubkey], proposer: Option<&Pubkey>, genesis: &str, labels: &std::collections::BTreeMap<Pubkey, String>) -> Res<Value> {
    use poleis_client::VOTE_PRO;
    if r.polis != POLIS {
        return Err(format!("the release of {} names unit {}: the house's code is unit {POLIS}'s, the branch that runs it", &r.commit[..12], r.polis));
    }
    if let Some(a) = proposer.filter(|a| !agents.contains(a)) {
        return Err(format!("{a} is no agent of unit {POLIS}: a release of the house's code is proposed by one of its agents (a key in its chairs)"));
    }
    let placeholders = p.placeholders();
    let c12 = &r.commit[..12];
    let envelopes: Vec<Value> = agents
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let e = poleis_client::vote_envelope(&r.hash, POLIS, VOTE_PRO);
            json!({ "n": i + 1, "signer": a.to_string(), "role": "agent", "unit": POLIS, "choice": "PRO", "envelope": crate::wire::b64(&e), "envelopeText": format!("poleis:vote:v1 ‖ {} ‖ unit {POLIS} ‖ PRO", crate::wire::hex(&r.hash)), "bytes": e.len(),
                    "does": format!("sign these {} bytes, a message and never a transaction, as its PRO on the release of {c12} in unit {POLIS}; hand the signature to press crank as {{release, unit, voter, signature}}: it lands it and seals unit {POLIS}", e.len()) })
        })
        .collect();
    let mut transactions = Vec::new();
    if let Some(a) = proposer {
        let ix = poleis_client::propose_raw_of(ROOT_UNIT, *a, r.hash, r.class, POLIS, r.manifest, &r.payload, &[POLIS]);
        let does = [
            format!("propose the house's code at commit {} as a release of unit {POLIS}, the branch that runs the house: SIGNAL under CONSENT, kind 0x52, unit {POLIS} alone impacted, at nonce {}", r.commit, r.nonce),
            format!("the proposer is an agent of unit {POLIS}, a key in its chairs; it pays the proposal's rent, which comes home at close"),
        ];
        transactions.push(crate::acts::entry(1, &format!("propose the release of {c12}"), &does, &[crate::chain::budget_of(p, 80_000)?, vec![ix]].concat(), a, labels, &placeholders)?);
    }
    let mut set = json!({
        "what": format!("THE BRANCH VOTES ITS OWN CODE: the release of the house's code at commit {}, the how of unit {POLIS}: its agents {}vote it by their envelopes, two of three, and press crank seals it; no seat holder votes code; the lander ships the seal by itself", r.commit, if proposer.is_some() { "propose it and " } else { "" }),
        "cluster": p.cluster.name(),
        "genesisHash": genesis,
        "signable": placeholders.is_empty(),
        "placeholders": placeholders,
        "emitted": crate::chain::now_iso(),
        "act": { "n": "release", "name": format!("the house's code at commit {c12}"), "class": r.class, "hash": crate::wire::hex(&r.hash), "payloadSha256": crate::wire::hex(&crate::wire::sha256(&r.payload)), "nonce": r.nonce, "impacted": [POLIS] },
        "release": { "commit": r.commit, "polis": POLIS, "nonce": r.nonce, "class": "SIGNAL", "consent": true, "kind": SIGNAL_RELEASE, "hash": crate::wire::hex(&r.hash) },
        "envelopes": envelopes,
        "envelopesRoad": "an envelope is a message its agent signs; press crank takes the signatures (a JSON list of {release, unit, voter, signature}, unit 1) and lands them, then seals unit 1",
    });
    if !transactions.is_empty() {
        set["signer"] = json!({ "address": proposer.unwrap().to_string(), "role": format!("an agent of unit {POLIS}: it proposes the release, its fee payer") });
        set["transactions"] = json!(transactions);
    }
    Ok(set)
}

impl Sitting {
    /// PRESS RELEASE: the agents' work order for the release of `commit` (THE BRANCH VOTES ITS OWN CODE): every agent's envelope of its open
    /// vote, or of the first nonce no proposal of it holds, with `proposer`'s own proposal when it proposes; a release sealed RATUM already is
    /// said and nothing written. The file's path (releases/<the commit's 12>.json).
    pub fn release(&mut self, commit: &str, proposer: Option<Pubkey>) -> Res<PathBuf> {
        use poleis_client::{self as client, HOLDER_KEY, STATE_PASSED, STATE_VOTING};
        if self.root_is_ours()? != poleis_treasury::MODE_COUNCIL {
            return Err("root 0 is still in genesis: nothing is proposed or voted before it is ratified (press ratify)".into());
        }
        let reg = client::RegistryView::parse(&self.account(&client::registry_pda(POLIS))?.ok_or("unit 1's registry is not on the cluster")?.data).ok_or("unit 1's registry does not parse")?;
        let agents: Vec<Pubkey> = reg.pentagons.iter().filter(|c| c.kind == HOLDER_KEY).map(|c| Pubkey::new_from_array(c.holder)).collect();
        let mut at = None;
        for nonce in 0..RELEASE_NONCES {
            let r = release(commit, nonce)?;
            let Some(a) = self.account(&client::proposal_pda(&r.hash))? else {
                at = Some((r, false));
                break;
            };
            if client::ProposalView::is_tombstone(&a.data) {
                continue;
            }
            match self.account(&client::session_pda(&r.hash, POLIS))?.and_then(|s| client::SessionView::parse(&s.data)).map(|s| s.state) {
                Some(STATE_VOTING) => {
                    at = Some((r, true));
                    break;
                }
                Some(STATE_PASSED) => return Err(format!("the release of {} is sealed RATUM already in unit {POLIS} (nonce {nonce}): the lander ships it; nothing is written", &r.commit[..12])),
                _ => continue,
            }
        }
        let (r, open) = at.ok_or_else(|| format!("the release of {commit}: its {RELEASE_NONCES} nonces are spent"))?;
        let proposer = match (proposer, open) {
            (Some(_), true) => {
                eprintln!("  the release of {} is proposed already (nonce {}, still voting): no second proposal is written, the envelopes are", &r.commit[..12], r.nonce);
                None
            }
            (p, _) => p,
        };
        if proposer.is_none() && !open {
            eprintln!("  the release of {} is not proposed yet: its envelopes count once an agent's proposal of it lands, at nonce {} (press release --commit {} --agent <an agent>)", &r.commit[..12], r.nonce, r.commit);
        }
        let mut labels = std::collections::BTreeMap::new();
        for a in &agents {
            labels.insert(*a, format!("an agent of unit {POLIS}, a key in its chairs"));
        }
        let genesis = self.rec.get("genesisHash").as_str().unwrap_or_default().to_string();
        let set = release_set(&self.p, &r, &agents, proposer.as_ref(), &genesis, &labels)?;
        let dir = self.dir.join("releases");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join(format!("{}.json", &r.commit[..12]));
        std::fs::write(&path, serde_json::to_string_pretty(&set).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!("  the release of {} (unit {POLIS}, nonce {}): {} envelope{} for its agents{}", &r.commit[..12], r.nonce, agents.len(), if agents.len() == 1 { "" } else { "s" }, if proposer.is_some() { ", and the proposal for one of them" } else { "" });
        Ok(path)
    }

    /// THE SEAL'S ROAD, for a root 0 that stands as this sitting's (its callers hold it so first): written beside the record, the same road
    /// on the record. Its path and the road.
    pub fn seal_road(&mut self) -> Res<(PathBuf, Value)> {
        let road = road(&self.p, &self.ids.poleis, &self.rpc.url)?;
        let path = write(&self.dir, &road)?;
        if self.rec.get("sealRoad")["road"] != road {
            self.rec.v["sealRoad"] = json!({ "file": path.display().to_string(), "road": road, "at": crate::chain::now_iso(), "proposes": said(&road), "lander": lander_line(&path) });
            self.rec.save()?;
        }
        Ok((path, road))
    }

    /// The road written where root 0 has just been found this sitting's, and said; a road that cannot be written stops nothing of the
    /// sitting (it is the lander's, never the chain's), and says why.
    pub fn road_said(&mut self) {
        match self.seal_road() {
            Ok((path, road)) => {
                eprintln!("  THE SEAL'S ROAD: {} {road}", path.display());
                eprintln!("    {}", said(&road));
                eprintln!("    the lander takes it by {}", lander_line(&path));
            }
            Err(e) => eprintln!("  THE SEAL'S ROAD: not written: {e}"),
        }
    }
}
