//! THE ACTS, DECIDED BY THE WHOLE TREE. The press writes the sets of the polis's first acts: the fund's constitution (`CODEX_FUND`), the
//! market's law and roles (`set_law`, `set_board`, each an `ACT_RAW` of the polis's own hand), the market itself (`CODEX_MARKET`), THE
//! POOL IS BORN AT THE LAUNCH (his Nov 22 2026 "Yes, add it"), the stake pool's birth (its Initialize, an `ACT_RAW` of the polis's hand,
//! whose execute's transaction makes the pool's account and its list first by the key that executes: pool.rs), and THE SWARM NAMES ITS
//! MINER (his Nov 23 2026 18:22:41 UTC "Yes, start it" to "Start the sixth act, the swarm naming $WARM's miner?"), the $WARM mine's
//! `set_miner` naming the params' `warm.miner`, the door's crank key `press miner` makes (an `ACT_RAW` of the polis's hand, the mine's
//! authority since its opening: until it executes the mine names no miner and mines nothing), and THE TREASURY PAYS THE COMPUTE (his Nov 23
//! 2026 23:08:00 UTC "Yes, add them" to "Add two first acts to the launch so the treasury pays for the AI calls?"): root 0's cap of the
//! dollar (`CODEX_PLAN`, the founding's plan cap and period restated, the params' `genesis.dollarCap` named: until it executes the cap is 0
//! and no dollar the mine's deposits brought leaves root 0's vault), then the router's month (`TREASURY_ALLOWANCE`, `router.monthCap` of the
//! dollar a plan period to the mine's miner, the door's crank key, standing until the polis revokes it), in that order. THE WHOLE
//! TREE DECIDES (his Nov 22 2026 09:50:51 UTC "you gotta fucking stop treating the keystones as what make the decisions porcoddio. the
//! fucking entire tree perfect bicameralism gvs posiverse neghiverse molecularity"): every act passes by two chambers at two levels, the AND
//! of each, and no key of his is in either. A member of root 0, a seat owner of its floor, proposes it by its own transaction
//! (`propose_nft`, written by `press vote --propose`); unit 1, his posiverse, holds its own vote of it (opened with the proposal for a CODEX
//! act, whose impacted set closes over the bound child, [root 0, unit 1]; convened by the crank for the others); its agents vote there by
//! their envelopes, more than half of its chairs passing it; unit 1's seal is its chair's vote in root 0 (`mandate`); root 0's floor, the
//! minted Lyssajudrons, votes by each seat owner's own transaction (`attest_nft`, written by `press vote`), more than half of its seats'
//! weight passing it; root 0's seal is the tree's verdict; the act executes.
//!
//! Each act's set holds the act (its payload, its hash) and the ENVELOPES of unit 1's agents, each a message its agent signs as its ballot
//! (`poleis:vote:v1` ‖ the proposal's hash ‖ unit 1 ‖ the choice), and nothing for a key of his. Everything anyone may send (the convene,
//! the attest that carries the agents' signed envelopes, both seals, the mandate, the execute) is `press crank`'s, paid by the deployer;
//! the proposal and the floor's votes are the seat owners' own, paid by them.

use crate::build::Ids;
use crate::chain::Addresses;
use crate::params::Params;
use crate::pool::{self, Seeded};
use crate::rpc::Account;
use crate::wire;
use crate::Res;
use crate::chain::POSIVERSE;
use poleis_client::{self as client, Payload, ACT_SIGNER, ACT_WRITABLE, CLASS_CODEX, HOLDER_KEY, ROOT_UNIT, VOTE_PRO};
use poleis_treasury::{CODEX_MARKET, MARKET_ACT, MARKET_SEAT, MARKET_TREASURY};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;

/// One act of the polis: its name, the proposal's class and payload, its hash, and its execute as the crank sends it. Its proposal is a
/// seat owner's (`floor::proposal`), never built here.
pub struct Act {
    pub n: usize,
    pub name: &'static str,
    pub file: &'static str,
    pub class: u8,
    pub payload: Vec<u8>,
    pub payload_sha256: [u8; 32],
    pub manifest: [u8; 32],
    pub nonce: u64,
    pub hash: [u8; 32],
    /// the units whose sessions the proposal opens: [root 0, unit 1] for a CODEX act (closed over the bound child), [root 0] otherwise
    pub impacted: Vec<u64>,
    /// the execute, its crank the deployer as built; `execute_by` names the crank that sends it
    pub execute: Instruction,
    /// what the execute's own transaction makes first, at the `with_seed` addresses of the key that executes (act 5: the pool's account and
    /// its list); nothing for the others
    pub before: Vec<Seeded>,
    /// a program the act calls that a chain may lack (act 5: the stake pool program): where none stands, the act is not run, and that is said
    pub needs: Option<Pubkey>,
    pub does: Vec<String>,
}

impl Act {
    /// Unit 1's vote of this act is convened by the crank (the proposal did not open it).
    pub fn convene(&self) -> bool {
        !self.impacted.contains(&POSIVERSE)
    }
    /// The execute with `crank` as its first account, its signer and payer (it pays an account the act makes: the fund, root 0's market).
    pub fn execute_by(&self, crank: &Pubkey) -> Instruction {
        let mut ix = self.execute.clone();
        ix.accounts[0].pubkey = *crank;
        ix
    }
    /// The execute's transaction as press crank sends it, built for the key that executes: each account of `before` made at that key's own
    /// `with_seed` address (at the chain's rent, `rent`, read against what stands there, `standing`), then the execute, that key its crank.
    /// Another key's transaction makes them at its own addresses, never at the ones the sealed payload names.
    pub fn execute_ixs(&self, executor: &Pubkey, rent: impl Fn(usize) -> Res<u64>, standing: impl Fn(&Pubkey) -> Res<Option<Account>>) -> Res<Vec<Instruction>> {
        let mut out = Vec::new();
        for s in &self.before {
            out.extend(s.making(executor, rent(s.space)?, standing(&s.at(executor)?)?.as_ref())?);
        }
        out.push(self.execute_by(executor));
        Ok(out)
    }
}

/// The execute's transaction of `act` on the wire, its makings first and its compute budget when the params name a priority fee: its bytes,
/// with nothing standing at the makings' addresses, or (`dust`) a stranger's lamports at each (the larger form).
pub fn execute_bytes(p: &Params, act: &Act, executor: &Pubkey, dust: bool) -> Res<usize> {
    let standing = dust.then(|| Account { lamports: 1, owner: crate::loader::SYSTEM.to_string(), executable: false, data: Vec::new() });
    let ixs = [crate::chain::budget_of(p, 120_000)?, act.execute_ixs(executor, |_| Ok(1_000_000), |_| Ok(standing.clone()))?].concat();
    let (m, signers) = wire::message(&ixs, executor, &[0; 32]);
    Ok(1 + 64 * signers.len() + m.len())
}

/// The impacted set of an act of root 0: a CODEX act closes over the bound child, his posiverse in its chair.
pub fn impacted_of(class: u8) -> Vec<u64> {
    if class == CLASS_CODEX {
        vec![ROOT_UNIT, POSIVERSE]
    } else {
        vec![ROOT_UNIT]
    }
}

/// The market's law as CODEX_MARKET names it: [class][kind][the roads program:32][board:8][split:1][plan:1][bar:2].
pub fn market_law(program: &Pubkey, board: u64, split: u8, plan: u8, bar: u16) -> Vec<u8> {
    let mut v = vec![CLASS_CODEX, CODEX_MARKET];
    v.extend_from_slice(program.as_ref());
    v.extend_from_slice(&board.to_le_bytes());
    v.extend_from_slice(&[split, plan]);
    v.extend_from_slice(&bar.to_le_bytes());
    v
}

/// `execute` of a market law: each impacted unit's session, then each one's registry (a CODEX act's closure), root 0's registry again as
/// the origin's, its market, the system, and the board the law names when it turns the market on.
pub fn execute_market(crank: &Pubkey, hash: &[u8; 32], law: &[u8], board: &Pubkey, impacted: &[u64]) -> Instruction {
    let mut metas = vec![AccountMeta::new(*crank, true), AccountMeta::new_readonly(client::config_pda(), false), AccountMeta::new(client::proposal_pda(hash), false)];
    metas.extend(impacted.iter().map(|u| AccountMeta::new_readonly(client::session_pda(hash, *u), false)));
    metas.extend(impacted.iter().map(|u| AccountMeta::new_readonly(client::registry_pda(*u), false)));
    metas.extend([
        AccountMeta::new_readonly(client::registry_pda(ROOT_UNIT), false),
        AccountMeta::new(client::find_pda(&[b"market", &ROOT_UNIT.to_le_bytes()]), false),
        client::system_meta(),
    ]);
    if law[42] | law[43] != 0 {
        metas.push(AccountMeta::new_readonly(*board, false));
    }
    let mut d = vec![poleis_client::tag::IX_EXECUTE];
    d.extend_from_slice(hash);
    client::ix(d, metas)
}

/// A class's market bit by its name in the params.
fn classes(p: &Params, path: &str) -> Res<u8> {
    if p.is_placeholder(path) {
        return Ok(0);
    }
    let a = p.at(path).as_array().ok_or_else(|| format!("params {path}: a list of classes (treasury, seat, act)"))?;
    let mut bits = 0u8;
    for c in a {
        bits |= match c.as_str() {
            Some("treasury") => MARKET_TREASURY,
            Some("seat") => MARKET_SEAT,
            Some("act") => MARKET_ACT,
            _ => return Err(format!("params {path}: {c} is not treasury, seat or act")),
        };
    }
    Ok(bits)
}

/// The eight acts, from the params (a placeholder read as a stand-in that is marked: such a set is for reading). `deployer` is the key that
/// carries them out as press crank: the base of the pool's accounts its fifth act's execute makes, and the crank each execute is built for.
pub fn acts(p: &Params, ids: &Ids, at: &Addresses, deployer: &Pubkey) -> Res<Vec<Act>> {
    let crank = *deployer;
    let base = p.u64_or("acts.nonce", 1)?;
    let manifest_of = |name: &str| wire::sha256(format!("THE LAUNCH PRESS: {name}").as_bytes());
    let mut out = Vec::new();

    // 1. the fund's constitution
    let fund = Payload::Fund {
        origin: ROOT_UNIT,
        fee_bps: u16::try_from(p.u64_or("fund.feeBps", 0)?).map_err(|_| "fund.feeBps: basis points")?,
        cycle_slots: p.u64_or("fund.cycleSlots", 0)?,
        range_lo: u16::try_from(p.u64_or("fund.rangeLo", 0)?).map_err(|_| "fund.rangeLo: basis points")?,
        range_hi: u16::try_from(p.u64_or("fund.rangeHi", 0)?).map_err(|_| "fund.rangeHi: basis points")?,
        option_fee: p.u64_or("fund.optionFee", 0)?,
        chair_quorum: u8::try_from(p.u64_or("fund.chairQuorum", 0)?).map_err(|_| "fund.chairQuorum: a count of chairs")?,
    };
    let name = "the fund's constitution (CODEX_FUND)";
    let manifest = manifest_of(name);
    let h = client::proposal_hash(fund.class(), ROOT_UNIT, &manifest, &fund.hash(), base);
    let impacted = impacted_of(fund.class());
    out.push(Act {
        n: 1,
        name,
        file: "1-the-fund.json",
        class: fund.class(),
        payload: fund.bytes(),
        payload_sha256: fund.hash(),
        manifest,
        nonce: base,
        hash: h,
        execute: client::execute_of(ROOT_UNIT, crank, h, &impacted, &fund),
        impacted,
        before: Vec::new(),
        needs: None,
        does: vec![format!(
            "constitute the fund of every swarm: a fee of {} bps on every sale, a cycle of {} slots, a payout range of {} to {} bps, an option fee of {} lamports, a chair quorum of {}",
            p.at("fund.feeBps"), p.at("fund.cycleSlots"), p.at("fund.rangeLo"), p.at("fund.rangeHi"), p.at("fund.optionFee"), p.at("fund.chairQuorum")
        )],
    });

    // 2. and 3. the market's law and roles, by the polis's own hand (ACT_RAW)
    let mut law = vec![poleis_roads_tag::SET_LAW];
    law.extend_from_slice(ids.poleis.as_ref());
    law.extend_from_slice(&ROOT_UNIT.to_le_bytes());
    law.extend_from_slice(&p.u64_or("market.stepSlots", 0)?.to_le_bytes());
    for f in ["market.maxStepBp", "market.thresholdBp", "market.feeBp"] {
        law.extend_from_slice(&u32::try_from(p.u64_or(f, 0)?).map_err(|_| format!("{f}: basis points"))?.to_le_bytes());
    }
    for f in ["market.poolBase", "market.poolQuote", "market.judgeSlots"] {
        law.extend_from_slice(&p.u64_or(f, 0)?.to_le_bytes());
    }
    // THE SWARM JUDGES (his press of Nov 23 2026 19:32:23 UTC, "The swarm, by its own vote"): the board's judge is root 0's own hand, so each
    // road's receipts are said by a sealed act of the whole tree, never by a key
    let judge = crate::chain::judge_of(p, at)?;
    let mut roles = vec![poleis_roads_tag::SET_BOARD];
    roles.extend_from_slice(p.key_or_standin("market.crediter")?.as_ref());
    roles.extend_from_slice(judge.as_ref());
    for (n, name, file, data, does) in [
        (2, "the market's law (set_law, by the polis's hand)", "2-the-boards-law.json", law, format!(
            "set the law of the roads laid by rule on board {}: a step of {} slots, a lag limit of {} bp, a threshold of {} bp, a fee of {} bp, pools of {} and {}, the judge {} slots after a close; the polis Poleis {}, root 0",
            p.at("market.board"), p.at("market.stepSlots"), p.at("market.maxStepBp"), p.at("market.thresholdBp"), p.at("market.feeBp"), p.at("market.poolBase"), p.at("market.poolQuote"), p.at("market.judgeSlots"), ids.poleis
        )),
        (3, "the market's roles (set_board, by the polis's hand)", "3-the-boards-roles.json", roles, format!(
            "name the board's crediter {} and its judge, the swarm by its own vote: root 0's own hand {judge}, so a road's receipts (what a passed road was worth) are said only by a sealed act of the whole tree, within {} slots of the road's close",
            p.at("market.crediter"), p.at("market.judgeSlots")
        )),
    ] {
        let act = Payload::Raw { origin: ROOT_UNIT, program: ids.market, accounts: vec![(at.agent, ACT_SIGNER), (at.board, ACT_WRITABLE)], data };
        let manifest = manifest_of(name);
        let nonce = base + n as u64 - 1;
        let h = client::proposal_hash(act.class(), ROOT_UNIT, &manifest, &act.hash(), nonce);
        let impacted = impacted_of(act.class());
        out.push(Act {
            n,
            name,
            file,
            class: act.class(),
            payload: act.bytes(),
            payload_sha256: act.hash(),
            manifest,
            nonce,
            hash: h,
            execute: client::execute_of(ROOT_UNIT, crank, h, &impacted, &act),
            impacted,
            before: Vec::new(),
            needs: None,
            does: vec![does, format!("the polis's own hand {} signs it, by Poleis, once the act is sealed", at.agent)],
        });
    }

    // 4. the market itself
    let board_id = p.u64_or("market.board", 0)?;
    let bar = u16::try_from(p.u64_or("market.barBp", 0)?).map_err(|_| "market.barBp: basis points")?;
    let law = market_law(&ids.market, board_id, classes(p, "market.split")?, classes(p, "market.plan")?, bar);
    let name = "the market (CODEX_MARKET)";
    let manifest = manifest_of(name);
    let nonce = base + 3;
    let payload_sha256 = client::sha(&[&law]);
    let h = client::proposal_hash(CLASS_CODEX, ROOT_UNIT, &manifest, &payload_sha256, nonce);
    let impacted = impacted_of(CLASS_CODEX);
    out.push(Act {
        n: 4,
        name,
        file: "4-the-market.json",
        class: CLASS_CODEX,
        execute: execute_market(&crank, &h, &law, &at.board, &impacted),
        impacted,
        payload: law,
        payload_sha256,
        manifest,
        nonce,
        hash: h,
        before: Vec::new(),
        needs: None,
        does: vec![format!(
            "name the polis's market: the roads program {} and its board {}, the classes {} split to the market and {} that may be plans, a bar of {} bp of the board",
            ids.market, p.at("market.board"), p.at("market.split"), p.at("market.plan"), p.at("market.barBp")
        )],
    });

    // 5. THE POOL IS BORN AT THE LAUNCH: the stake pool's Initialize by the polis's own hand (ACT_RAW), its execute's transaction making the
    // pool's account and its list first by the key that executes; its reserve, its mint and its fee account made ahead (press pool)
    let pool = pool::Pool::of(p, deployer, &ids.poleis)?;
    let act = pool.payload();
    let name = "the stake pool's birth (its Initialize, by the polis's hand)";
    let manifest = manifest_of(name);
    let nonce = base + 4;
    let h = client::proposal_hash(act.class(), ROOT_UNIT, &manifest, &act.hash(), nonce);
    let impacted = impacted_of(act.class());
    out.push(Act {
        n: 5,
        name,
        file: "5-the-pool.json",
        class: act.class(),
        payload: act.bytes(),
        payload_sha256: act.hash(),
        manifest,
        nonce,
        hash: h,
        execute: client::execute_of(ROOT_UNIT, crank, h, &impacted, &act),
        impacted,
        before: pool.in_the_execute(),
        needs: Some(pool.program),
        does: vec![
            format!(
                "give birth to the polis's stake pool: the SPL stake pool program {}'s Initialize of the pool {}, its validator list {} for {} validators, its reserve {} and its mint {} (nine decimals, minted only by the pool's withdraw authority {}); its manager and its staker the polis's own hand {}; its fee account root 0's vault of the mint {} (its authority root 0's treasury); its epoch fee 1/1, the whole yield into that vault, out only by a sealed TREASURY act; no withdrawal, deposit or referral fee; deposits open to anyone",
                pool.program, pool.pool, pool.list, pool.max_validators, pool.reserve, pool.mint, pool.withdraw, pool.agent, pool.vault
            ),
            format!(
                "its execute's transaction makes the pool's account ({} bytes) and its list ({} bytes) first, by create_account_with_seed whose base is the key that executes, the deployer {deployer} as press crank: no other key makes them, so another key's execute fails whole",
                pool::POOL_LEN,
                pool::list_len(pool.max_validators)
            ),
        ],
    });

    // 6. THE SWARM NAMES ITS MINER: the $WARM mine's `set_miner` by the polis's own hand (ACT_RAW), naming the door's crank key
    let miner = p.key_or_standin("warm.miner")?;
    let act = Payload::Raw { origin: ROOT_UNIT, program: ids.warm, accounts: vec![(at.agent, ACT_SIGNER), (at.mine, ACT_WRITABLE)], data: set_miner_data(&miner) };
    let name = "the $WARM mine's miner (set_miner, by the polis's hand)";
    let manifest = manifest_of(name);
    let nonce = base + MINER_ACT as u64 - 1;
    let h = client::proposal_hash(act.class(), ROOT_UNIT, &manifest, &act.hash(), nonce);
    let impacted = impacted_of(act.class());
    out.push(Act {
        n: MINER_ACT,
        name,
        file: "6-the-miner.json",
        class: act.class(),
        payload: act.bytes(),
        payload_sha256: act.hash(),
        manifest,
        nonce,
        hash: h,
        execute: client::execute_of(ROOT_UNIT, crank, h, &impacted, &act),
        impacted,
        before: Vec::new(),
        needs: None,
        does: vec![
            format!(
                "name the miner of the $WARM mine {}: {} (the params' warm.miner {}), the key whose signature alone posts the router's signed spends to the mine, the mint door's crank (its Worker's secret SPACE_ROUTER_MINER_KEY, by his press: press miner); until it is named the mine names no miner and mines nothing",
                at.mine, miner, p.at("warm.miner").as_str().unwrap_or("(none)")
            ),
            format!("the polis's own hand {}, the mine's authority since its opening, signs it, by Poleis, once the act is sealed", at.agent),
        ],
    });

    // 7. THE TREASURY PAYS THE COMPUTE: root 0's cap of the dollar, a CODEX_PLAN restating the founding's plan cap and period exactly (its
    // execute overwrites all three and zeroes what was spent) and naming the params' genesis.dollarCap; until it executes the cap is 0 and no
    // dollar leaves root 0's vault
    let (plan_cap, plan_period, dollar_cap) = (p.u64_or("genesis.planCap", 0)?, p.u64_or("genesis.planPeriod", 0)?, p.u64_or("genesis.dollarCap", 0)?);
    let act = Payload::Plan { origin: ROOT_UNIT, cap: plan_cap, period: plan_period, token_cap: dollar_cap };
    let name = "the treasury's dollar cap (CODEX_PLAN)";
    let manifest = manifest_of(name);
    let nonce = base + DOLLAR_CAP_ACT as u64 - 1;
    let h = client::proposal_hash(act.class(), ROOT_UNIT, &manifest, &act.hash(), nonce);
    let impacted = impacted_of(act.class());
    let dollar = p.key_or_standin("warm.dollar")?;
    out.push(Act {
        n: DOLLAR_CAP_ACT,
        name,
        file: "7-the-dollar-cap.json",
        class: act.class(),
        payload: act.bytes(),
        payload_sha256: act.hash(),
        manifest,
        nonce,
        hash: h,
        execute: client::execute_of(ROOT_UNIT, crank, h, &impacted, &act),
        impacted,
        before: Vec::new(),
        needs: None,
        does: vec![
            format!(
                "set root 0's cap of its default mint, the dollar {dollar} (the params' genesis.dollarCap {}): {dollar_cap} micro-dollars a plan period may leave root 0's own vault of it {}, the mine's treasury, where people's deposits land; until it executes the cap is 0 and no dollar leaves",
                p.at("genesis.dollarCap"),
                client::vault_pda(ROOT_UNIT, &dollar)
            ),
            format!(
                "it restates the founding's plan as it stands: a cap of {plan_cap} lamports and a period of {plan_period} slots (the params' genesis.planCap and genesis.planPeriod), since the execute writes all three and zeroes what the period spent"
            ),
        ],
    });

    // 8. THE TREASURY PAYS THE COMPUTE: the router's month, a TREASURY allowance of root 0 to the mine's miner (the door's crank key the sixth
    // act names): router.monthCap of the dollar a plan period, pulled by the miner's own signature, until the polis revokes it
    let month_cap = p.u64_or("router.monthCap", 0)?;
    let act = Payload::Allowance { origin: ROOT_UNIT, payee: miner.to_bytes(), payee_kind: HOLDER_KEY, mint: dollar, cap: month_cap, period: plan_period, expires: 0 };
    let name = "the router's month (TREASURY_ALLOWANCE)";
    let manifest = manifest_of(name);
    let nonce = base + ROUTER_ACT as u64 - 1;
    let h = client::proposal_hash(act.class(), ROOT_UNIT, &manifest, &act.hash(), nonce);
    let impacted = impacted_of(act.class());
    out.push(Act {
        n: ROUTER_ACT,
        name,
        file: "8-the-routers-month.json",
        class: act.class(),
        payload: act.bytes(),
        payload_sha256: act.hash(),
        manifest,
        nonce,
        hash: h,
        execute: client::execute_of(ROOT_UNIT, crank, h, &impacted, &act),
        impacted,
        before: Vec::new(),
        needs: None,
        does: vec![
            format!(
                "give the mine's miner {miner} (the params' warm.miner, the door's crank key) a standing allowance of root 0's treasury: up to {month_cap} micro-dollars of the dollar {dollar} (the params' router.monthCap {}) a period of {plan_period} slots, pulled out of root 0's vault by the miner's own signature, each pull charged to the dollar cap act {DOLLAR_CAP_ACT} names; it pays the AI calls the router buys; no expiry: it stands until the polis revokes it (TREASURY_REVOKE)",
                p.at("router.monthCap")
            ),
            format!("its execute makes the allowance's account at {}, the crank its payer", client::allowance_pda(&h)),
        ],
    });
    Ok(out)
}

/// The act that names the $WARM mine's miner: the sixth.
pub const MINER_ACT: usize = 6;
/// THE TREASURY PAYS THE COMPUTE: the act that names root 0's cap of the dollar (a CODEX_PLAN), the seventh.
pub const DOLLAR_CAP_ACT: usize = 7;
/// THE TREASURY PAYS THE COMPUTE: the act that gives the mine's miner the router's month of the dollar (a TREASURY allowance), the eighth.
pub const ROUTER_ACT: usize = 8;

/// The $WARM mine's `set_miner` (warm_mine ix.rs): `[IX_SET_MINER][miner:32]`, its accounts `[authority (s), mine (w)]`.
pub fn set_miner_data(miner: &Pubkey) -> Vec<u8> {
    [&[warm_mine::IX_SET_MINER][..], miner.as_ref()].concat()
}

/// The market program's tags this press sends (poleis_roads `IX_SET_BOARD` 1, `IX_SET_LAW` 11; held to its source by a test).
pub mod poleis_roads_tag {
    pub const INIT_BOARD: u8 = 0;
    pub const SET_BOARD: u8 = 1;
    pub const SET_LAW: u8 = 11;
}

/// One transaction as the page reads it: its signer, its message with a blank blockhash, the message's sha256, its accounts named.
pub fn entry(n: usize, name: &str, does: &[String], ixs: &[Instruction], payer: &Pubkey, labels: &BTreeMap<Pubkey, String>, placeholders: &[String]) -> Res<Value> {
    let (m, signers) = wire::message(ixs, payer, &[0; 32]);
    if signers.len() != 1 || signers[0] != *payer {
        return Err(format!("{name}: {} signers; the page signs for one, its fee payer", signers.len()));
    }
    let (sig_count, keys, _) = wire::layout(&m)?;
    let (ro_signed, ro_unsigned) = (m[1] as usize, m[2] as usize);
    let accounts: Vec<Value> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let signer = i < sig_count;
            let writable = if signer { i < sig_count - ro_signed } else { i < keys.len() - ro_unsigned };
            json!({ "address": k.to_string(), "signer": signer, "writable": writable, "is": labels.get(k) })
        })
        .collect();
    let bytes = 1 + 64 + m.len();
    if bytes > wire::PACKET {
        return Err(format!("{name}: {bytes} bytes, past {}", wire::PACKET));
    }
    Ok(json!({
        "n": n, "name": name, "does": does, "signers": [payer.to_string()], "signable": placeholders.is_empty(), "placeholders": placeholders,
        "accounts": accounts, "message": wire::b64(&m), "messageSha256": wire::hex(&wire::sha256(&m)), "bytes": bytes,
    }))
}

/// A ballot: the envelope one agent of unit 1 signs as its chair's vote of an act, a message and never a transaction.
pub fn envelope(n: usize, agent: &Pubkey, hash: &[u8; 32], labels: &BTreeMap<Pubkey, String>) -> Value {
    let e = client::vote_envelope(hash, POSIVERSE, VOTE_PRO);
    let chamber = "unit 1's chairs: his posiverse's own vote, its seal its chair's voice in root 0";
    json!({
        "n": n, "signer": agent.to_string(), "role": "agent", "unit": POSIVERSE, "chamber": chamber, "choice": "PRO",
        "envelope": wire::b64(&e), "envelopeText": format!("poleis:vote:v1 ‖ {} ‖ unit {POSIVERSE} ‖ PRO", wire::hex(hash)), "bytes": e.len(),
        "does": format!("sign these {} bytes, a message and never a transaction, as its PRO in {chamber}; hand the signature to press crank as {{act, unit, voter, signature}}: it lands it", e.len()),
        "is": labels.get(agent),
    })
}

/// The names the page shows beside an account.
pub fn labels(ids: &Ids, at: &Addresses, agents: &[Pubkey]) -> BTreeMap<Pubkey, String> {
    let mut m = BTreeMap::new();
    for (i, a) in agents.iter().enumerate() {
        m.insert(*a, format!("agent {} of his posiverse (unit 1, chair {i})", i + 1));
    }
    for (k, w) in [
        (ids.poleis, "Poleis"),
        (ids.market, "the market's program"),
        (ids.warm, "the $WARM program"),
        (at.config, "root 0's config"),
        (at.registry, "root 0's registry"),
        (at.treasury, "root 0's treasury"),
        (at.curve, "root 0's curve (the floor's mint terms)"),
        (at.agent, "the polis's own hand (its agent PDA)"),
        (at.board, "the market's board"),
        (at.fund, "the fund"),
        (at.market_law, "root 0's market"),
        (at.mine, "the $WARM mine"),
        (client::registry_pda(POSIVERSE), "unit 1's registry (his posiverse)"),
        (client::treasury_pda(POSIVERSE), "unit 1's treasury"),
        (Pubkey::default(), "the system program"),
        (crate::loader::COMPUTE_BUDGET, "the compute budget"),
    ] {
        m.insert(k, w.to_string());
    }
    m
}

/// The sets, one an act, and their index: written into `dir`, for unit 1's agents and the crank. `birth`: the stake pool program stands on
/// this chain, so the fifth act, the pool's birth, can run here; where it does not, its set is not written (the press says so). No set holds
/// a transaction of anyone's: the proposal and the floor's votes are each seat owner's own (press vote), the rest is press crank's.
pub fn write(dir: &std::path::Path, p: &Params, ids: &Ids, at: &Addresses, genesis: &str, deployer: &Pubkey, birth: bool) -> Res<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let agents = p.agents()?;
    let placeholders = p.placeholders();
    let labels = labels(ids, at, &agents);
    let mut written = Vec::new();
    let mut index = Vec::new();
    let all = acts(p, ids, at, deployer)?;
    let total = all.len();
    for a in all {
        if a.needs.is_some() && !birth {
            continue;
        }
        let envelopes: Vec<Value> = agents.iter().enumerate().map(|(i, v)| envelope(1 + i, v, &a.hash, &labels)).collect();
        let crew_pass = client::chair_quorum(agents.len() as u8);
        let mut crank = Vec::new();
        if a.convene() {
            crank.push("convene: unit 1 opens its own vote of the act in its own session".to_string());
        }
        crank.extend([
            format!("attest: the agents' signed envelopes landed in unit 1's session ({crew_pass} of its {} chairs pass it)", agents.len()),
            "seal unit 1: its chairs' majority is its verdict while it has no floor".to_string(),
            "mandate: unit 1's verdict becomes its chair's vote in root 0".to_string(),
            "seal root 0: unit 1's chair AND the floor's own votes; both PASSED, or nothing executes (it waits for the floor's votes, or its window's end)".to_string(),
            "execute: what the proposal stores is what executes, once".to_string(),
        ]);
        if !a.before.is_empty() {
            crank.push(format!(
                "its execute's transaction makes first, by the key that executes (the deployer {deployer}, its base and its payer): {}; another key's makings stand at its own addresses, never at the ones this payload names",
                a.before.iter().map(|s| format!("{} ({} bytes, seed {:?})", s.what, s.space, s.seed)).collect::<Vec<_>>().join(", ")
            ));
        }
        let mut set = json!({
            "what": format!("THE LAUNCH PRESS, act {} of {total}: {}. THE WHOLE TREE DECIDES: a seat owner of root 0's floor proposes it by its own transaction (press vote --propose); each of unit 1's agents signs only its envelope, a message; root 0's floor votes by each seat owner's own transaction (press vote); two of the three agents in unit 1 AND more than half of the floor's seats pass it; press crank sends everything else, paid by the deployer. No key of his is in it", a.n, a.name),
            "cluster": p.cluster.name(),
            "genesisHash": genesis,
            "signable": placeholders.is_empty(),
            "placeholders": placeholders,
            "emitted": crate::chain::now_iso(),
            "act": { "n": a.n, "name": a.name, "class": a.class, "hash": wire::hex(&a.hash), "payload": wire::b64(&a.payload), "payloadSha256": wire::hex(&a.payload_sha256), "manifest": wire::hex(&a.manifest), "nonce": a.nonce, "impacted": a.impacted, "convened": a.convene() },
            "does": a.does,
            "proposal": format!("a seat owner's own transaction, propose_nft (its seat proved by its token): press vote --act {} --owner <its address> --propose writes it for its owner to sign", a.n),
            "floor": format!("each seat owner's own PRO, attest_nft: press vote --act {} --owner <its address> writes it for its owner to sign; the press never lands one", a.n),
            "order": "1 the proposal (a seat owner's); then the agents' envelopes and the floor's votes, any order, inside the vote's window; then press crank",
            "envelopes": envelopes,
            "crank": crank,
            "envelopesRoad": "an envelope is a message its agent signs; press crank takes the signatures (a JSON list of {act, unit, voter, signature}, unit 1) and lands them",
        });
        if a.needs.is_some() {
            // the pool's birth: every address of the pool, and what the execute's transaction makes first and where
            let pool = pool::Pool::of(p, deployer, &ids.poleis)?;
            let mut before = Vec::new();
            for s in &a.before {
                before.push(json!({ "what": s.what, "seed": s.seed, "bytes": s.space, "owner": s.owner.to_string(), "base": deployer.to_string(), "at": s.at(deployer)?.to_string() }));
            }
            set["pool"] = json!({ "tag": pool.tag, "maxValidators": pool.max_validators, "seeds": pool.seeds_json(), "addresses": pool.addresses_json(), "madeAhead": "its reserve, its mint and root 0's vault of the mint: press pool, in the first sitting" });
            set["before"] = json!(before);
        }
        if a.n == MINER_ACT {
            // THE SWARM NAMES ITS MINER: who the act names, and where the same key stands on the door
            set["miner"] = json!({
                "address": p.key_or_standin("warm.miner")?.to_string(), "params": p.at("warm.miner"), "mine": at.mine.to_string(), "authority": at.agent.to_string(),
                "door": "the mint door's crank signs each batch of the router's spends with this key: its Worker's secret SPACE_ROUTER_MINER_KEY, by his press (press miner --secret), SOL on it for each flask's $WARM account and each batch's fee",
            });
        }
        if a.n == DOLLAR_CAP_ACT {
            // THE TREASURY PAYS THE COMPUTE: the cap it names and the plan it restates, as the params give them
            set["plan"] = json!({
                "unit": ROOT_UNIT, "dollar": p.key_or_standin("warm.dollar")?.to_string(), "vault": client::vault_pda(ROOT_UNIT, &p.key_or_standin("warm.dollar")?).to_string(),
                "dollarCap": p.u64_or("genesis.dollarCap", 0)?, "planCap": p.u64_or("genesis.planCap", 0)?, "planPeriod": p.u64_or("genesis.planPeriod", 0)?,
                "restates": "the founding's plan cap and period, exactly: the execute writes the three and zeroes what the period spent",
            });
        }
        if a.n == ROUTER_ACT {
            // THE TREASURY PAYS THE COMPUTE: the allowance the act makes, its payee the mine's miner
            set["router"] = json!({
                "allowance": client::allowance_pda(&a.hash).to_string(), "unit": ROOT_UNIT, "payee": p.key_or_standin("warm.miner")?.to_string(), "payeeKind": "a key (HOLDER_KEY): the mine's miner, the door's crank key",
                "mint": p.key_or_standin("warm.dollar")?.to_string(), "cap": p.u64_or("router.monthCap", 0)?, "period": p.u64_or("genesis.planPeriod", 0)?, "expires": 0,
                "pull": "the miner pulls by its own signature into a dollar account it owns, up to the cap a period, each pull charged to root 0's dollar cap (act 7); a revoke is a TREASURY_REVOKE of the polis",
            });
        }
        let path = dir.join(a.file);
        std::fs::write(&path, serde_json::to_string_pretty(&set).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
        index.push(json!({ "n": a.n, "act": a.name, "file": a.file, "hash": wire::hex(&a.hash) }));
        written.push(path);
    }
    let idx = dir.join("index.json");
    std::fs::write(&idx, serde_json::to_string_pretty(&json!({ "what": "THE LAUNCH PRESS: the polis's first acts, in order", "cluster": p.cluster.name(), "acts": index })).unwrap() + "\n").map_err(|e| e.to_string())?;
    written.push(idx);
    Ok(written)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_market_programs_tags_are_its_own() {
        let src = include_str!("../../program/poleis_roads/src/lib.rs");
        for (name, tag) in [("IX_INIT_BOARD", super::poleis_roads_tag::INIT_BOARD), ("IX_SET_BOARD", super::poleis_roads_tag::SET_BOARD), ("IX_SET_LAW", super::poleis_roads_tag::SET_LAW)] {
            assert!(src.contains(&format!("pub const {name}: u8 = {tag};")), "{name} is {tag} in the roads program's source");
        }
    }
}
