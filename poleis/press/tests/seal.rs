//! THE LAUNCH NAMES THE SEAL'S ROAD (Nov 23 2026, his "Wire it in"): the road the press writes once root 0 stands, the three fields
//! `poleis-seal read` takes and nothing else (`{rpc, program, polis}`): the node the params name for the cluster, the sitting's own Poleis,
//! root 0. ANY MEMBER PROPOSES, THE SWARM DECIDES (Nov 23 2026, his "the swarm votes on updates" and his "Yes, build it"): the road names
//! nobody who may propose, and a params file that names `seal.proposers`, even none, is refused; mainnet's node is named by its host alone,
//! and the sitting's own endpoint is never kept.
use poleis_press::params::{placeholder_address, Params};
use poleis_press::seal::{self, ANY_MEMBER, FILE, POLIS, PROPOSERS_NAMED};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;

fn k(w: &str) -> Pubkey {
    placeholder_address(w)
}

/// The rehearsal's params as the press ships them, their stand-ins filled as the rehearsal's own process makes them.
fn rehearsal() -> Params {
    let mut p = Params::read(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.rehearsal.json"))).unwrap();
    let s = |w: &str| json!(k(w).to_string());
    p.fill(&json!({
        "posiverse.agents": [s("agent 1"), s("agent 2"), s("agent 3")],
        "floor.issuer": s("the issuer"),
        "market.crediter": s("the crediter"),
        "warm.dollar": s("the dollar"),
        "warm.miner": s("the miner"),
        "genesis.defaultMint": s("the dollar"),
    }))
    .unwrap();
    p
}
fn mainnet() -> Params {
    Params::read(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/params.mainnet.json"))).unwrap()
}
/// The params' text with `seal` set to `v`.
fn with_seal(cluster: &str, v: Value) -> Params {
    Params::from_text(&json!({ "cluster": cluster, "seal": v }).to_string()).unwrap()
}

#[test]
fn the_road_is_the_branch_that_runs_the_house_of_the_sittings_own_poleis_and_nothing_else() {
    let (p, poleis) = (rehearsal(), k("the sitting's Poleis"));
    let road = seal::road(&p, &poleis, "http://127.0.0.1:23410").unwrap();
    assert_eq!(road, json!({ "rpc": "http://127.0.0.1:23410", "program": poleis.to_string(), "polis": 1 }));
    assert_eq!(POLIS, 1, "THE BRANCH VOTES ITS OWN CODE: the polis is unit 1, the branch that runs the house, its agents; never root 0's floor");
    // the shape poleis-seal's Road::parse takes: three fields, an address its program, nobody who may propose
    let mut fields: Vec<&str> = road.as_object().unwrap().keys().map(String::as_str).collect();
    fields.sort_unstable();
    assert_eq!(fields, vec!["polis", "program", "rpc"], "the road holds the reader's three fields and nothing else");
    assert!(road["program"].as_str().unwrap().parse::<Pubkey>().is_ok() && road["polis"].is_u64());
    // written beside the record, read back whole; no part left; the same road twice is one file
    let dir = std::env::temp_dir().join(format!("the-seals-road-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = seal::write(&dir, &road).unwrap();
    assert_eq!(path, dir.join(FILE));
    assert_eq!(serde_json::from_str::<Value>(&std::fs::read_to_string(&path).unwrap()).unwrap(), road);
    assert!(!dir.join(format!("{FILE}.part")).exists(), "the file is renamed whole, never left half");
    let before = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(seal::write(&dir, &road).unwrap(), path);
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), before, "the same road is not written again");
    // what the press says of it, and the line the lander takes it by: his hand
    assert_eq!(seal::said(&road), ANY_MEMBER);
    assert!(ANY_MEMBER.contains("unit 1's agents propose a release") && ANY_MEMBER.contains("no seat holder votes code") && ANY_MEMBER.contains("no key names who proposes"));
    let line = seal::lander_line(&path);
    assert!(line.starts_with(&format!("LANDER_SEAL_ROAD={}", path.display())) && line.contains("his hand"), "{line}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// ANY MEMBER PROPOSES, THE SWARM DECIDES (Nov 23 2026, his 17:52 UTC "the swarm votes on updates, you dont have it linear with me and
/// keystones", then his "Yes, build it"): no params file names who proposes; one that does, even naming none, is refused before anything
/// starts, with words that say why, on every chain; mainnet's params as shipped name nobody, and their road says any member proposes.
#[test]
fn no_params_name_who_proposes_a_release() {
    let p = mainnet();
    seal::checked(&p).unwrap();
    let road = seal::road(&p, &k("mainnet's Poleis"), "https://his-endpoint.example/?api-key=not-kept").unwrap();
    assert!(road.get("proposers").is_none());
    assert_eq!(seal::said(&road), ANY_MEMBER);
    for cluster in ["mainnet", "localnet"] {
        for named in [json!([k("p").to_string()]), json!([]), json!("nobody")] {
            let p = with_seal(cluster, json!({ "rpc": "https://api.mainnet-beta.solana.com", "proposers": named }));
            assert_eq!(seal::checked(&p).unwrap_err(), PROPOSERS_NAMED, "{cluster}: {named}");
            assert_eq!(seal::road(&p, &k("a Poleis"), "http://127.0.0.1:23410").unwrap_err(), PROPOSERS_NAMED);
        }
    }
    assert!(PROPOSERS_NAMED.contains("the branch's agents propose and the swarm decides") && PROPOSERS_NAMED.contains("Take the line out"));
    // a road written before, with proposers: the press says the reader refuses it
    assert!(seal::said(&json!({ "rpc": "x", "program": "y", "polis": 0, "proposers": [] })).contains("refuses it"));
}

#[test]
fn the_roads_node_is_the_clusters() {
    // mainnet: the params' node, by its host alone, never the sitting's own endpoint (it may carry a key, and the road is a file)
    let p = mainnet();
    assert_eq!(seal::node(&p, "https://his-endpoint.example/?api-key=not-kept").unwrap(), "https://api.mainnet-beta.solana.com");
    let node = |url: &str| seal::node(&with_seal("mainnet", json!({ "rpc": url })), "");
    assert_eq!(node("https://his-node.example:8899/").unwrap(), "https://his-node.example:8899", "one trailing slash is no path");
    for (bad, says) in [
        ("http://api.mainnet-beta.solana.com", "https:// only"),
        ("https://mainnet.example/?api-key=x", "names more than a host"),
        ("https://mainnet.example/key-in-the-path", "names more than a host"),
        ("https://mainnet.example#frag", "names more than a host"),
        ("https://user@mainnet.example", "a user"),
        ("https://", "no host"),
        ("PLACEHOLDER: his node", "not his yet"),
    ] {
        let e = node(bad).expect_err(bad);
        assert!(e.contains(says), "{bad}: {e}");
    }
    assert!(seal::node(&with_seal("mainnet", json!({})), "").unwrap_err().contains("params seal.rpc"), "mainnet names its node");
    // a rehearsal's chain: the sitting's own validator, on this machine and nothing else
    let r = rehearsal();
    assert_eq!(seal::node(&r, "http://127.0.0.1:23410").unwrap(), "http://127.0.0.1:23410");
    for bad in ["http://localhost:8899", "https://api.mainnet-beta.solana.com", "http://127.0.0.1:23410/path"] {
        assert!(seal::node(&r, bad).is_err(), "{bad}");
    }
    // the road's params are held before anything starts: mainnet's node, and on every chain no proposers
    seal::checked(&p).unwrap();
    seal::checked(&r).unwrap();
    assert!(seal::checked(&with_seal("mainnet", json!({ "rpc": "https://mainnet.example/?api-key=x" }))).is_err());
    seal::checked(&with_seal("localnet", json!({}))).unwrap();
}
