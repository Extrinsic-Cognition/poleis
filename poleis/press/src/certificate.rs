//! THE BIRTH CERTIFICATE: the sitting read back from the chain and set beside its record. Every binary's hash and the bytes each program
//! runs held to it; the ids; every transaction the press sent, its signature, slot, units, fee and bytes, and every transaction of the acts
//! (the seat owners' proposals and votes, the crank's), read off the chain by their proposals; root 0's curve and the seats minted on it;
//! the final authority of each program; the rent each account holds; the SOL the sitting asked before it started and what it spent; the
//! params whole; and the seal's road the house's lander takes (seal.rs), its file's place said in words.

use crate::acts;
use crate::build::PROGRAMS;
use crate::chain::{program_id, Sitting, METAPLEX};
use crate::loader::{self, State};
use crate::wire;
use crate::Res;
use poleis_client::{self as client, nft, ROOT_UNIT};
use serde_json::{json, Value};
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;

impl Sitting {
    pub fn certificate(&mut self, checks: Option<&std::path::Path>) -> Res<Value> {
        let mut rent: Vec<Value> = Vec::new();
        let mut rent_total = 0u64;
        let mut hold = |what: &str, key: &Pubkey, a: &Option<crate::rpc::Account>| {
            if let Some(a) = a {
                rent_total += a.lamports;
                rent.push(json!({ "what": what, "address": key.to_string(), "bytes": a.data.len(), "lamports": a.lamports }));
            }
        };

        // the programs: their bytes held to the builds, their final authority
        let mut programs = serde_json::Map::new();
        for (name, _, _, _) in PROGRAMS {
            let id = program_id(&self.ids, name);
            let pa = self.rpc.account(&id.to_string())?;
            hold(&format!("{name}: its program account"), &id, &pa);
            let pd_key = loader::programdata_of(&id);
            let pd = self.rpc.account(&pd_key.to_string())?;
            hold(&format!("{name}: its program data"), &pd_key, &pd);
            // the build as the record holds it, less its path on this machine (a birth certificate is published: it names no folder)
            let mut built = self.rec.get("builds")[name].clone();
            if let Some(o) = built.as_object_mut() {
                o.remove("path");
            }
            let (slot, authority, onchain) = match &pd {
                Some(a) => match loader::state(&a.data)? {
                    State::ProgramData { slot, authority } => {
                        let n = built["bytes"].as_u64().unwrap_or(0) as usize;
                        let code = &a.data[loader::PROGRAMDATA_HEADER..];
                        let head = &code[..n.min(code.len())];
                        (json!(slot), json!(authority.map(|k| k.to_string())), json!({ "sha256OfBuildLength": wire::hex(&wire::sha256(head)), "rest": if code[head.len()..].iter().all(|b| *b == 0) { "zeros" } else { "NOT ZEROS" }, "dataLen": code.len() }))
                    }
                    _ => (Value::Null, Value::Null, Value::Null),
                },
                None => (Value::Null, Value::Null, json!("not deployed")),
            };
            let matches = onchain["sha256OfBuildLength"].as_str() == built["sha256"].as_str() && onchain["rest"] == "zeros";
            let final_authority = authority.clone();
            programs.insert(
                name.to_string(),
                json!({
                    "id": id.to_string(), "programData": pd_key.to_string(), "deploySlot": slot, "build": &built, "onChain": onchain, "runsTheBuild": matches,
                    "authority": final_authority, "authorityIs": if authority.as_str() == Some(&self.at.upgrade.to_string()) { "Poleis's upgrade PDA: the polis's seal" } else if authority.as_str() == Some(&self.deployer.to_string()) { "the deployer (not handed off yet)" } else { "another" },
                }),
            );
        }

        // root 0
        let config = self.rpc.account(&self.at.config.to_string())?;
        hold("root 0: its config", &self.at.config, &config);
        let registry = self.rpc.account(&self.at.registry.to_string())?;
        hold("root 0: its registry", &self.at.registry, &registry);
        let treasury = self.rpc.account(&self.at.treasury.to_string())?;
        hold("root 0: its treasury (rent and what the seats' mints paid)", &self.at.treasury, &treasury);
        let curve = self.rpc.account(&self.at.curve.to_string())?;
        hold("root 0: its curve (the floor's mint terms)", &self.at.curve, &curve);
        let root = match (&config, &registry) {
            (Some(c), Some(r)) => {
                let cv = client::ConfigView::parse(&c.data).ok_or("root 0's config does not parse")?;
                let rv = client::RegistryView::parse(&r.data).ok_or("root 0's registry does not parse")?;
                // THE FLOOR IS THE MINTED LYSSAJUDRONS: each row its seat's receipt, and for a minted seat its mint, its first buyer, its price
                let mut floor = Vec::new();
                for h in &rv.holders {
                    let mut row = json!({ "holder": Pubkey::new_from_array(h.holder).to_string(), "kind": h.kind, "seats": h.seats, "weight": h.weight });
                    if h.kind == poleis_treasury::HOLDER_NFT {
                        if let Some(r) = self.rpc.account(&Pubkey::new_from_array(h.holder).to_string())?.and_then(|a| nft::NftSeat::decode(&a.data).ok()) {
                            row["seat"] = json!({ "mint": Pubkey::new_from_array(r.mint).to_string(), "firstBuyer": Pubkey::new_from_array(r.first_buyer).to_string(), "serial": r.serial, "paidLamports": r.paid_lamports });
                        }
                    }
                    floor.push(row);
                }
                let terms = curve.as_ref().and_then(|a| nft::MintCurve::decode(&a.data).ok()).map(|c| {
                    json!({ "address": self.at.curve.to_string(), "startLamports": c.start_lamports, "stepLamports": c.step_lamports, "minted": c.minted_count, "capacity": c.capacity, "opens": c.opens_slot, "closes": c.closes_slot, "issuer": Pubkey::new_from_array(c.completion_issuer).to_string(), "genesisHash": wire::base58(&c.genesis_hash) })
                });
                json!({
                    "mode": match cv.mode { 0 => "solo", 1 => "genesis", 2 => "council", _ => "?" }, "defaultMint": cv.default_mint.to_string(),
                    "chairs": chairs_of(&rv),
                    "curve": terms,
                    "floor": floor,
                    "seats": rv.total_seats, "holders": rv.holders.len(), "charter": wire::hex(&rv.charter), "window": rv.window, "codexWindow": rv.codex_window, "planCap": rv.plan_cap, "planPeriod": rv.plan_period,
                    // THE TREASURY PAYS THE COMPUTE: root 0's cap of its default mint, the dollar, as the chain holds it (act 7 names it)
                    "dollarCap": rv.token_plan_cap, "dollarSpent": rv.token_plan_spent,
                    "dollarCapIs": match self.p.u64("genesis.dollarCap") {
                        _ if rv.token_plan_cap == 0 => "none: no dollar leaves root 0's vault until act 7 executes".to_string(),
                        Ok(c) if c == rv.token_plan_cap => format!("the params' genesis.dollarCap, named by act {} (a CODEX_PLAN of root 0, sealed by the whole tree)", acts::DOLLAR_CAP_ACT),
                        _ => "ANOTHER than the params' genesis.dollarCap".to_string(),
                    },
                    "name": rv.name, "founding": self.rec.get("founding"),
                })
            }
            _ => json!("not founded"),
        };

        // his posiverse, unit 1: its chairs, its agents' own keys
        let crew_reg = self.rpc.account(&client::registry_pda(crate::chain::POSIVERSE).to_string())?;
        hold("unit 1 (his posiverse): its registry", &client::registry_pda(crate::chain::POSIVERSE), &crew_reg);
        let crew_t = self.rpc.account(&client::treasury_pda(crate::chain::POSIVERSE).to_string())?;
        hold("unit 1: its treasury", &client::treasury_pda(crate::chain::POSIVERSE), &crew_t);
        let posiverse = match crew_reg.as_ref().and_then(|r| client::RegistryView::parse(&r.data)) {
            Some(v) => json!({ "unit": v.unit, "parent": v.parent, "chairInParent": v.chair_in_parent, "chairs": chairs_of(&v), "seats": v.total_seats, "charter": wire::hex(&v.charter), "window": v.window, "planCap": v.plan_cap, "planPeriod": v.plan_period, "name": v.name }),
            None => json!("not founded"),
        };
        // root 0's vault of the dollar: the mine's treasury
        let vault_at = crate::chain::treasury_of(&self.p)?;
        let vault_a = self.rpc.account(&vault_at.to_string())?;
        hold("root 0's vault of the dollar (the mine's treasury)", &vault_at, &vault_a);
        let root_vault = match &vault_a {
            Some(a) if a.data.len() >= 72 => json!({ "address": vault_at.to_string(), "mint": Pubkey::new_from_array(a.data[..32].try_into().unwrap()).to_string(), "authority": Pubkey::new_from_array(a.data[32..64].try_into().unwrap()).to_string(), "amount": u64::from_le_bytes(a.data[64..72].try_into().unwrap()), "authorityIs": if a.data[32..64] == client::treasury_pda(ROOT_UNIT).to_bytes() { "root 0's treasury: tokens leave only by a sealed TREASURY act" } else { "ANOTHER" } }),
            _ => json!("not made"),
        };

        // the $WARM mine and its face
        let mine = self.rpc.account(&self.at.mine.to_string())?;
        hold("the $WARM mine", &self.at.mine, &mine);
        let mint = self.rpc.account(&self.at.mint.to_string())?;
        hold("the $WARM mint", &self.at.mint, &mint);
        let vault = self.rpc.account(&self.at.vault.to_string())?;
        hold("the $WARM vault", &self.at.vault, &vault);
        let metadata = self.rpc.account(&self.at.metadata.to_string())?;
        hold("the $WARM face (Metaplex's metadata)", &self.at.metadata, &metadata);
        let warm = match (&mine, &mint) {
            (Some(m), Some(t)) => {
                use warm_mine::state::Mine;
                let at = |o: usize| Pubkey::new_from_array(m.data[o..o + 32].try_into().unwrap()).to_string();
                let u = |d: &[u8], o: usize| u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
                // THE SWARM NAMES ITS MINER: who the mine's miner is, and whether it is the one the sixth act names
                let miner = at(Mine::OFF_MINER);
                let miner_is = match self.p.key("warm.miner") {
                    _ if miner == Pubkey::default().to_string() => "none: the mine mines nothing until act 6 executes".to_string(),
                    Ok(k) if k.to_string() == miner => format!("the params' warm.miner, named by act {} (set_miner by the polis's hand, sealed by the whole tree): the door's crank key", acts::MINER_ACT),
                    _ => "ANOTHER than the params' warm.miner".to_string(),
                };
                json!({
                    "mine": self.at.mine.to_string(), "authority": at(Mine::OFF_AUTHORITY), "treasury": at(Mine::OFF_TREASURY), "dollar": at(Mine::OFF_DOLLAR), "miner": miner, "minerIs": miner_is,
                    "opening": u(&m.data, Mine::OFF_OPENING), "minted": u(&m.data, Mine::OFF_MINTED), "deposited": u(&m.data, Mine::OFF_DEPOSITED),
                    "mint": { "address": self.at.mint.to_string(), "authority": if t.data[..4] == [1, 0, 0, 0] { json!(Pubkey::new_from_array(t.data[4..36].try_into().unwrap()).to_string()) } else { Value::Null }, "supply": u(&t.data, 36), "decimals": t.data[44], "freezeAuthority": if t.data[46..50] == [0, 0, 0, 0] { Value::Null } else { json!("SOME") } },
                    "face": match &metadata {
                        Some(md) if md.owner == METAPLEX.to_string() => {
                            let mine_addr = solana_address::Address::new_from_array(self.at.mine.to_bytes());
                            let mint_addr = solana_address::Address::new_from_array(self.at.mint.to_bytes());
                            match warm_mine::face::read_face(&md.data, &mine_addr, &mint_addr) {
                                Ok((n, s)) => json!({ "metadata": self.at.metadata.to_string(), "name": String::from_utf8_lossy(n), "symbol": String::from_utf8_lossy(s), "updateAuthority": "the mine" }),
                                Err(_) => json!("a metadata account that is not the mine's face"),
                            }
                        }
                        _ => json!("none: Metaplex's program does not stand on this chain, so the face's CPI did not run (held to Metaplex's SDK by the press's wire test)"),
                    },
                    "opened": self.rec.get("warm"),
                })
            }
            _ => json!("not opened"),
        };

        // the market's board
        let board = self.rpc.account(&self.at.board.to_string())?;
        hold("the market's board", &self.at.board, &board);
        let board_v = match &board {
            Some(b) if b.data.len() == crate::chain::BOARD_SIZE => {
                let k = |o: usize| Pubkey::new_from_array(b.data[o..o + 32].try_into().unwrap()).to_string();
                let u = |o: usize| u64::from_le_bytes(b.data[o..o + 8].try_into().unwrap());
                let w = |o: usize| u32::from_le_bytes(b.data[o..o + 4].try_into().unwrap());
                json!({
                    "address": self.at.board.to_string(), "id": u(2), "authority": k(10), "crediter": k(42), "judge": k(74),
                    "judgeIs": if k(74) == self.at.agent.to_string() { "the swarm, by its own vote: root 0's own hand (a road's receipts a sealed act of the whole tree)" } else { "ANOTHER" }, "credited": u(106), "polis": k(114), "unit": u(146),
                    "law": { "stepSlots": u(154), "maxStepBp": w(162), "thresholdBp": w(166), "feeBp": w(170), "poolBase": u(174), "poolQuote": u(182), "judgeSlots": u(190) }, "earned": u(198),
                })
            }
            _ => json!("not opened"),
        };

        // the acts: each proposal as it stands, and every transaction that touched it, read off the chain
        let mut act_rows = Vec::new();
        // THE STAKE EARNS: and the ninth, where the params name it
        for a in acts::acts(&self.p, &self.ids, &self.at, &self.deployer)?.into_iter().chain(crate::engine::later(&self.p, &self.ids, &self.at, &self.deployer)?) {
            let prop = client::proposal_pda(&a.hash);
            let session = client::session_pda(&a.hash, ROOT_UNIT);
            let pv = self.rpc.account(&prop.to_string())?.and_then(|p| client::ProposalView::parse(&p.data));
            let sv = self.rpc.account(&session.to_string())?.and_then(|s| client::SessionView::parse(&s.data));
            let cv = self.rpc.account(&client::session_pda(&a.hash, crate::chain::POSIVERSE).to_string())?.and_then(|s| client::SessionView::parse(&s.data));
            let mut txs = Vec::new();
            for (sig, slot, ok) in self.rpc.signatures_for(&prop.to_string(), 50)?.into_iter().rev() {
                let (fee, units) = match self.rpc.transaction(&sig)? {
                    Some((_, fee, units, _)) => (json!(fee), json!(units)),
                    None => (Value::Null, Value::Null),
                };
                txs.push(json!({ "signature": sig, "slot": slot, "landed": ok, "fee": fee, "units": units }));
            }
            act_rows.push(json!({
                "n": a.n, "act": a.name, "hash": wire::hex(&a.hash), "proposal": prop.to_string(),
                "state": match (&pv, &sv) {
                    (Some(p), Some(s)) => json!({ "executed": p.executed, "impacted": a.impacted, "session": crate::crank::state_name(s.state), "chairsPro": s.pent_pro, "floorPro": s.hex_pro, "votes": s.votes, "unit1": cv.map(|c| json!({ "session": crate::crank::state_name(c.state), "chairsPro": c.pent_pro, "votes": c.votes })) }),
                    _ => json!("not proposed"),
                },
                "transactions": txs,
            }));
        }
        // THE TREASURY PAYS THE COMPUTE: the router's month as the chain holds it, the allowance act 8 made at its proposal's own address
        let router = match acts::acts(&self.p, &self.ids, &self.at, &self.deployer)?.into_iter().find(|a| a.n == acts::ROUTER_ACT) {
            Some(a) => {
                let at = client::allowance_pda(&a.hash);
                let acc = self.rpc.account(&at.to_string())?;
                hold("the router's month (act 8's allowance)", &at, &acc);
                match acc.as_ref().and_then(|x| client::AllowanceView::parse(&x.data)) {
                    Some(v) => {
                        let payee = Pubkey::new_from_array(v.payee);
                        json!({
                            "address": at.to_string(), "unit": v.unit, "payee": payee.to_string(), "payeeKind": v.payee_kind, "mint": v.mint.to_string(), "cap": v.cap, "period": v.period, "start": v.start, "spent": v.spent, "expires": v.expires, "payer": v.payer.to_string(),
                            "payeeIs": if self.p.key("warm.miner").is_ok_and(|m| m == payee) && v.payee_kind == client::HOLDER_KEY { "the params' warm.miner, the mine's miner (the door's crank key): it pulls by its own signature" } else { "ANOTHER than the params' warm.miner" },
                            "expiresIs": if v.expires == 0 { "never: it stands until the polis revokes it (TREASURY_REVOKE)" } else { "at a slot" },
                        })
                    }
                    None => json!({ "address": at.to_string(), "state": "not made: act 8 waits for the seal, then press crank" }),
                }
            }
            None => json!("no act 8 in these params"),
        };
        let fund = self.rpc.account(&self.at.fund.to_string())?.and_then(|f| client::FundView::parse(&f.data)).map(|f| {
            json!({ "feeBps": f.fee_bps, "cycleSlots": f.cycle_slots, "rangeLo": f.range_lo, "rangeHi": f.range_hi, "optionFee": f.option_fee, "chairQuorum": f.chair_quorum })
        });
        let market_law = self.rpc.account(&self.at.market_law.to_string())?.map(|m| {
            let d = &m.data;
            if d.len() >= 54 {
                json!({ "program": Pubkey::new_from_array(d[10..42].try_into().unwrap()).to_string(), "board": u64::from_le_bytes(d[42..50].try_into().unwrap()), "split": d[50], "plan": d[51], "barBp": u16::from_le_bytes([d[52], d[53]]) })
            } else {
                json!("a market account of another shape")
            }
        });

        // the press's own transactions, by step
        let txs = self.rec.transactions();
        // every buffer's whole history, read off the chain: a pass cut short (a kill) can land writes its record never saved; the chain keeps
        // them all, and the certificate lists the ones the record lacks
        let recorded: std::collections::BTreeSet<&str> = txs.iter().filter_map(|t| t["signature"].as_str()).collect();
        let mut buffers = serde_json::Map::new();
        if let Some(bs) = self.rec.get("buffers").as_object() {
            for (name, b) in bs {
                let Some(addr) = b["address"].as_str() else { continue };
                let history = self.rpc.signatures_for(addr, 1000)?;
                let missing: Vec<Value> = history.iter().filter(|(sig, _, _)| !recorded.contains(sig.as_str())).map(|(sig, slot, ok)| json!({ "signature": sig, "slot": slot, "landed": ok })).collect();
                buffers.insert(name.clone(), json!({ "address": addr, "seed": b["seed"], "onChain": history.len(), "inTheRecord": history.len() - missing.len(), "notInTheRecord": missing }));
            }
        }
        let mut steps: BTreeMap<String, (u64, u64, u64, u64, u64)> = BTreeMap::new();
        for t in &txs {
            let e = steps.entry(t["step"].as_str().unwrap_or("?").to_string()).or_default();
            e.0 += 1;
            e.1 += t["signatures"].as_u64().unwrap_or(1);
            e.2 += t["units"].as_u64().unwrap_or(0);
            e.3 += t["bytes"].as_u64().unwrap_or(0);
            e.4 += t["fee"].as_u64().unwrap_or(0);
        }
        let by_step: Vec<Value> = steps.iter().map(|(s, (n, sigs, cu, bytes, fee))| json!({ "step": s, "transactions": n, "signatures": sigs, "unitsRead": cu, "bytes": bytes, "fees": fee })).collect();

        // THE POOL IS BORN AT THE LAUNCH: its five accounts' rent, its addresses and seeds, the stake pool program it was born on (as the
        // sitting recorded it and as it stands), what was made ahead, its birth read back, and the transactions that made it
        let pool = self.pool_of()?;
        for (what, key) in [
            ("the pool's account", pool.pool),
            ("the pool's validator list", pool.list),
            ("the pool's reserve", pool.reserve),
            ("the pool's mint", pool.mint),
            ("root 0's vault of the pool's mint (its fee account)", pool.vault),
        ] {
            let a = self.rpc.account(&key.to_string())?;
            hold(what, &key, &a);
        }
        let born = match self.pool_at()? {
            crate::pool::PoolAt::Born(read) => json!({ "read": read }),
            crate::pool::PoolAt::Unborn => json!("not born yet: act 5 waits for the seal, then press crank"),
            crate::pool::PoolAt::NoProgram => json!(crate::pool::NOT_RUN),
        };
        let pool_txs: Vec<Value> = txs.iter().filter(|t| t["step"] == json!("pool") || t["what"].as_str().is_some_and(|w| w.starts_with("act 5: executed"))).cloned().collect();
        let pool_v = json!({
            "what": "THE POOL IS BORN AT THE LAUNCH: the SPL stake pool born by root 0's sealed raw act (act 5), its manager and its staker the polis's hand, its fee account root 0's vault of its mint, its epoch fee 1/1; its account and its list made in the execute's own transaction by the deployer's key, its reserve, mint and vault made ahead",
            "program": crate::pool::STAKE_POOL.to_string(),
            "programRecorded": self.rec.get("poolProgram"),
            "programNow": self.stake_pool_program()?.map(|p| p.json()),
            "programNamed": self.p.at("pool.programSha256"),
            "tag": pool.tag, "maxValidators": pool.max_validators, "seeds": pool.seeds_json(), "addresses": pool.addresses_json(),
            "ahead": self.rec.get("pool")["ahead"],
            "born": born,
            "transactions": pool_txs,
        });
        let fees: u64 = steps.values().map(|v| v.4).sum();
        let (before, now) = (self.rec.get("before")["deployer"].as_u64(), self.rpc.balance(&self.deployer.to_string())?);
        // what came to the deployer after the sitting started (a rehearsal's faucet after a cut), which it spent too
        let came: u64 = self.rec.get("airdrops").as_array().map(|a| a.iter().filter(|r| r["afterStart"] == json!(true)).filter_map(|r| r["lamports"].as_u64()).sum()).unwrap_or(0);
        let checks_v = match checks {
            Some(p) => serde_json::from_str::<Value>(&std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?).map_err(|e| format!("{}: {e}", p.display()))?,
            None => Value::Null,
        };
        // THE LAUNCH NAMES THE SEAL'S ROAD: the road the house's lander reads a release's seal by, as the sitting wrote it beside its record;
        // the file's place said in words (a birth certificate is published: it names no folder)
        let seal_road = match &config {
            Some(_) => match self.root_is_ours().and_then(|_| self.seal_road()) {
                Ok((_, road)) => json!({
                    "what": "THE LAUNCH NAMES THE SEAL'S ROAD: the road poleis-seal read takes, unit 1 of the sitting's own Poleis (THE BRANCH VOTES ITS OWN CODE: the branch that runs the house, its agents), read at the node the params name for the cluster; it names nobody who may propose",
                    "road": road,
                    "proposes": crate::seal::said(&road),
                    "file": format!("{} beside the sitting's record", crate::seal::FILE),
                    "lander": format!("LANDER_SEAL_ROAD=<the sitting's folder>/{} in the lander's install: his hand, never the press's", crate::seal::FILE),
                }),
                Err(e) => json!(format!("not written: {e}")),
            },
            None => json!("not written: root 0 is not founded"),
        };
        let version = self.rpc.call("getVersion", json!([]))?;
        // the rules it ran under: the word of whoever started the validator (the rehearsal's script says it), the features the chain holds
        // active, read here, and whether Metaplex's program stands
        let metaplex = self.rpc.account(&METAPLEX.to_string())?.is_some_and(|a| a.executable);
        let rules = match self.rpc.features() {
            Ok(features) => {
                let active: Vec<&str> = features.iter().filter(|(_, on)| *on).map(|(id, _)| id.as_str()).collect();
                json!({
                    "said": std::env::var("PRESS_RULES").ok(),
                    "featureAccounts": features.len(),
                    "activeFeatures": active.len(),
                    "activeSha256": wire::hex(&wire::sha256(active.iter().map(|id| format!("{id}\n")).collect::<String>().as_bytes())),
                    "activeSha256Of": "the active features' ids in base58, sorted, each followed by a newline",
                    "metaplex": metaplex,
                })
            }
            // an endpoint that will not list the feature program's accounts (a public one may refuse it) stops no certificate
            Err(e) => json!({ "said": std::env::var("PRESS_RULES").ok(), "featuresUnread": e, "metaplex": metaplex }),
        };
        let cert = json!({
            "what": "THE LAUNCH PRESS: the birth certificate of one sitting, read back from the chain",
            "cluster": self.p.cluster.name(),
            "genesisHash": self.rpc.check_cluster()?,
            "rpc": if self.p.cluster == crate::rpc::Cluster::Localnet { json!(self.rpc.url) } else { json!("his mainnet endpoint (not kept: it may carry a key)") },
            "validator": { "solanaCore": version["solana-core"], "featureSet": version["feature-set"] },
            "rules": rules,
            "at": crate::chain::now_iso(),
            "house": json!({ "commit": self.rec.get("house")["commit"], "partsUncommitted": self.rec.get("house")["partsUncommitted"] }),
            "params": { "sha256": self.p.sha256, "values": self.p.raw },
            "ids": self.rec.get("ids"),
            "derived": self.rec.get("derived"),
            "programs": programs,
            "root0": root,
            "sealRoad": seal_road,
            "posiverse": posiverse,
            "vault": root_vault,
            "warm": warm,
            "board": board_v,
            "acts": act_rows,
            "fund": fund,
            "marketLaw": market_law,
            "pool": pool_v,
            "router": router,
            "press": {
                "steps": by_step,
                "transactions": txs,
                "buffers": buffers,
                "resumes": self.rec.get("resumes"),
                "need": self.rec.get("need"),
                "deployer": { "address": self.deployer.to_string(), "before": before, "cameAfter": came, "now": now, "spent": before.map(|b| (b + came).saturating_sub(now)), "fees": fees },
                "airdrops": self.rec.get("airdrops"),
                "rpcCalls": *self.rpc.calls.borrow(),
            },
            "rent": { "accounts": rent, "lamports": rent_total },
            "checks": checks_v,
        });
        Ok(cert)
    }
}

/// A unit's twelve chairs: each one's index, holder and kind.
fn chairs_of(v: &client::RegistryView) -> Vec<Value> {
    v.pentagons
        .iter()
        .enumerate()
        .map(|(k, c)| match c.kind {
            client::HOLDER_EMPTY => json!({ "chair": k, "holds": "nobody" }),
            client::HOLDER_UNIT => json!({ "chair": k, "holds": format!("unit {}", client::holder_unit(&c.holder)) }),
            _ => json!({ "chair": k, "holds": Pubkey::new_from_array(c.holder).to_string(), "kind": c.kind }),
        })
        .collect()
}
