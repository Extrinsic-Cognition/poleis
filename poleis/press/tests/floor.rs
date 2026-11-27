//! THE FLOOR IS THE MINTED LYSSAJUDRONS: `press ratify` sends only what Poleis's order lets anyone send (`ratify_nft_genesis`, once the
//! curve's window has closed or every seat is minted, a seat standing), and says plainly what it waits for otherwise; `press vote` writes
//! one seat owner's own transactions of an act, as the Keystone page's prepared road reads a set: its proposal first when it proposes
//! (`propose_nft`, its seat proved by its token), then its PRO (`attest_nft`) for each seat it holds, each signed by that owner alone, its
//! fee payer, the blockhash blank. No key of his is in any of it, and a seat owner with no seat writes nothing.
use poleis_client::{nft, ROOT_UNIT};
use poleis_press::floor::{proposal, ratify_due, vote_set, RatifyDue, Seat};
use poleis_press::params::{placeholder_address as k, Params};
use poleis_press::{acts, build, chain, wire};
use poleis_treasury::{MODE_COUNCIL, MODE_GENESIS};
use serde_json::json;
use std::collections::BTreeMap;

fn params() -> Params {
    let s = |w: &str| k(w).to_string();
    let v = json!({
        "cluster": "localnet", "priorityMicroLamports": 0,
        "genesis": { "charter": "00".repeat(32), "name": "V", "desc": "", "defaultMint": s("floor mint"), "planCap": 1, "planPeriod": 6480000, "dollarCap": 2, "window": 1000, "rung": 1 },
        "floor": { "startLamports": 1000000, "stepLamports": 100000, "opensAfterSlots": 200, "windowSlots": 700, "issuer": s("floor issuer") },
        "posiverse": { "name": "F", "chair": 2, "agents": [s("floor agent 1"), s("floor agent 2"), s("floor agent 3")], "charter": "33".repeat(32), "window": 1000, "planCap": 1, "planPeriod": 1000000 },
        "fund": { "feeBps": 250, "cycleSlots": 1000, "rangeLo": 2000, "rangeHi": 8000, "optionFee": 1000000, "chairQuorum": 7 },
        "warm": { "name": "SWARM", "symbol": "$WARM", "uri": "u", "dollar": s("floor dollar"), "opening": 0, "miner": s("floor miner") },
        "router": { "monthCap": 1 },
        "market": { "board": 77, "stepSlots": 10, "maxStepBp": 100, "thresholdBp": 300, "feeBp": 30, "poolBase": 1, "poolQuote": 1, "judgeSlots": 5000, "crediter": s("c"), "split": ["treasury"], "plan": ["treasury"], "barBp": 1000 },
        "acts": { "nonce": 1 },
        "pool": { "maxValidators": 10, "tag": "lp1" }
    });
    Params::from_text(&v.to_string()).unwrap()
}

fn the_acts(p: &Params) -> Vec<acts::Act> {
    let (ids, ..) = build::compiled().unwrap();
    acts::acts(p, &ids, &chain::addresses(&ids, 77), &k("floor deployer")).unwrap()
}

/// A seat of root 0's floor `owner` holds, by words: its receipt at the seat's PDA of its mint, its token the first buyer's own.
fn seat(owner: &solana_pubkey::Pubkey, n: u8) -> Seat {
    let mint = nft::mint_pda(ROOT_UNIT, &[n; 32]);
    Seat { seat: nft::seat_pda(&mint), proof: nft::OwnerProof { owner: *owner, mint, token: nft::initial_token_pda(&mint, owner), unit: ROOT_UNIT } }
}

#[test]
fn the_ratification_follows_poleiss_order() {
    // nothing minted, the window open: it waits for the first seat
    assert!(matches!(ratify_due(MODE_GENESIS, 0, 20, 900, 899, 0), RatifyDue::Wait(w) if w.contains("no seat of root 0's floor is minted yet")));
    // nothing minted and the window closed: never, and said so plainly
    assert!(matches!(ratify_due(MODE_GENESIS, 0, 20, 900, 900, 0), RatifyDue::Wait(w) if w.contains("can never be ratified")));
    // six of twenty minted inside the window: it waits for the window's end
    assert!(matches!(ratify_due(MODE_GENESIS, 6, 20, 900, 899, 6), RatifyDue::Wait(w) if w.contains("runs to slot 900") && w.contains("6 of its 20 seats minted")));
    // the window closed, or every seat minted inside it: due, by anyone
    assert_eq!(ratify_due(MODE_GENESIS, 6, 20, 900, 900, 6), RatifyDue::Ratify);
    assert_eq!(ratify_due(MODE_GENESIS, 20, 20, 900, 10, 20), RatifyDue::Ratify);
    // seats minted and every one burned: Poleis refuses a floor of none, and the press waits for nothing it could send
    assert!(matches!(ratify_due(MODE_GENESIS, 6, 20, 900, 950, 0), RatifyDue::Wait(w) if w.contains("can never be ratified")));
    assert_eq!(ratify_due(MODE_COUNCIL, 6, 20, 900, 950, 6), RatifyDue::Done);
    assert!(matches!(ratify_due(0, 6, 20, 900, 950, 6), RatifyDue::Wait(w) if w.contains("not in genesis")));
}

#[test]
fn an_owners_vote_is_its_own_transactions_and_nothing_else() {
    let p = params();
    let acts = the_acts(&p);
    let owner = k("a seat owner");
    for act in &acts {
        let s = seat(&owner, 1);
        for propose in [true, false] {
            let set = vote_set(&p, act, &owner, &[s], propose, "g", &BTreeMap::new()).unwrap();
            let txs = set["transactions"].as_array().unwrap();
            for t in txs {
                let m = wire::unb64(t["message"].as_str().unwrap()).unwrap();
                let (signers, keys, at) = wire::layout(&m).unwrap();
                assert_eq!((signers, keys[0]), (1, owner), "act {}: the owner alone signs it, its fee payer", act.n);
                assert!(m[at..at + 32].iter().all(|b| *b == 0), "its blockhash blank");
                assert_eq!(t["messageSha256"].as_str().unwrap(), wire::hex(&wire::sha256(&m)));
                assert!(t["bytes"].as_u64().unwrap() as usize <= wire::PACKET, "act {}: {} bytes", act.n, t["bytes"]);
            }
            // exactly its proposal first when it proposes, then its seat's PRO: the messages Solana's own compiler makes of them
            let mut want = Vec::new();
            if propose {
                want.push(proposal(act, &s.proof).unwrap());
            }
            want.push(s.proof.attest(act.hash, poleis_client::VOTE_PRO).unwrap());
            assert_eq!(want.iter().map(|ix| ix.data[0]).collect::<Vec<_>>(), if propose { vec![nft::IX_PROPOSE_NFT, nft::IX_ATTEST_NFT] } else { vec![nft::IX_ATTEST_NFT] });
            let got: Vec<Vec<u8>> = txs.iter().map(|t| wire::unb64(t["message"].as_str().unwrap()).unwrap()).collect();
            let compiled: Vec<Vec<u8>> = want.into_iter().map(|ix| wire::message(&[ix], &owner, &[0; 32]).0).collect();
            assert_eq!(got, compiled, "act {}: its own transactions and nothing else", act.n);
            assert_eq!(set["signer"]["address"], json!(owner.to_string()));
            assert_eq!(set["act"]["hash"], json!(wire::hex(&act.hash)));
        }
    }
    // an owner of two seats votes with each, one hexagon a seat
    let two = vote_set(&p, &acts[0], &owner, &[seat(&owner, 1), seat(&owner, 2)], false, "g", &BTreeMap::new()).unwrap();
    assert_eq!(two["transactions"].as_array().unwrap().len(), 2);
    // no seat, no vote; and a seat proved for another owner is not this owner's
    assert!(vote_set(&p, &acts[0], &owner, &[], true, "g", &BTreeMap::new()).unwrap_err().contains("holds no seat of root 0's floor"));
    assert!(vote_set(&p, &acts[0], &owner, &[seat(&k("someone else"), 3)], false, "g", &BTreeMap::new()).unwrap_err().contains("is not proved for"));
}

#[test]
fn a_proposal_by_a_seat_owner_is_the_acts_own() {
    let p = params();
    let owner = k("a proposer");
    for act in the_acts(&p) {
        let s = seat(&owner, 9);
        let ix = proposal(&act, &s.proof).unwrap();
        assert_eq!(ix.data[0], poleis_client::nft::IX_PROPOSE_NFT);
        assert_eq!(&ix.data[1..33], &act.hash[..], "act {}: its hash", act.n);
        assert_eq!((ix.data[33], &ix.data[34..42]), (act.class, &ROOT_UNIT.to_le_bytes()[..]), "act {}: its class and its origin, root 0", act.n);
        // the seat, its mint, the owner's token, then the proposer (the owner, its signer and payer): nobody else signs
        assert_eq!(ix.accounts[..4].iter().map(|m| (m.pubkey, m.is_signer)).collect::<Vec<_>>(), vec![(s.seat, false), (s.proof.mint, false), (s.proof.token, false), (owner, true)]);
        assert_eq!(ix.accounts.iter().filter(|m| m.is_signer).count(), 1);
        // the impacted set: a CODEX act closes over unit 1
        let count = ix.data[74] as usize;
        let impacted: Vec<u64> = (0..count).map(|i| u64::from_le_bytes(ix.data[75 + 8 * i..83 + 8 * i].try_into().unwrap())).collect();
        assert_eq!(impacted, act.impacted, "act {}", act.n);
    }
}
