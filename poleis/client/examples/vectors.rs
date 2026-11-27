//! Reference vectors for other language clients: PDAs, hashes, the
//! envelope and instruction bytes for fixed inputs, printed as JSON. The
//! JavaScript conformance test recomputes every one and must match.
//!
//! Run: cargo run -q --example vectors

use poleis_client::{
    accrual_hash, agent_pda, allowance_pda, attest_batch, buy_seats, config_pda, contract_pda, knock, knock_pda, pool_pda, withdraw_knock,
    council_pda, create_vault, distribution_pda, escrow_pda, execute, list_seats, listing_pda,
    open_genesis, propose, proposal_hash, proposal_pda, pull, registry_pda, sale_pda,
    seat_bid_pda, seat_commitment, session_pda, treasury_pda, vault_pda, vote_envelope,
    programdata_pda, upgrade_pda, ContractRef, GenesisSpec, Payload, ACT_SIGNER, ACT_WRITABLE,
    HOLDER_KEY, HOLDER_UNIT, ROOT_UNIT, RULE_RELEASE, TOKEN_PROGRAM_ID, VOTE_PRO,
};
use solana_pubkey::Pubkey;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn keys(ix: &solana_instruction::Instruction) -> String {
    ix.accounts.iter().map(|m| format!("\"{}\"", m.pubkey)).collect::<Vec<_>>().join(", ")
}

fn main() {
    // a fixed voter: the same seed the JS test derives its key from
    let seed = [7u8; 32];
    let sk = ed25519_dalek::SigningKey::from_bytes(&seed);
    let voter = Pubkey::new_from_array(sk.verifying_key().to_bytes());
    let crank = Pubkey::new_from_array([9u8; 32]);
    let manifest = [3u8; 32];
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 6_000 };
    let hash = proposal_hash(payload.class(), ROOT_UNIT, &manifest, &payload.hash(), 42);
    let envelope = vote_envelope(&hash, ROOT_UNIT, VOTE_PRO);
    use ed25519_dalek::Signer;
    let sig = sk.sign(&envelope).to_bytes();
    let [pre, att] = attest_batch(crank, hash, ROOT_UNIT, &[(voter, VOTE_PRO, sig)]);
    let prop = propose(voter, hash, manifest, &payload, &[ROOT_UNIT]);
    let exec = execute(crank, hash, &[ROOT_UNIT], &payload);
    let salt = [5u8; 32];
    // tier 1: names, token vaults, the raw act
    let mint = Pubkey::new_from_array([11u8; 32]);
    let genesis = GenesisSpec {
        charter: [1u8; 32],
        plan_cap: 10_000_000_000,
        plan_period: 1_000_000,
        window: 1_000,
        rung: 1,
        reserve: 1_000_000,
        deposit: 5_000_000,
        commit_end: 100,
        reveal_end: 200,
        founders: vec![voter, crank],
        name: b"THE FIRST SOCIETY".to_vec(),
        desc: b"two founders".to_vec(),
    };
    let genesis_ix = open_genesis(voter, &genesis);
    let name = Payload::Name { origin: ROOT_UNIT, name: b"RENAMED".to_vec(), desc: Vec::new() };
    let token = Payload::TokenTransfer { origin: ROOT_UNIT, mint, dest: 1, amount: 500, token_program: TOKEN_PROGRAM_ID };
    let token_hash = proposal_hash(token.class(), ROOT_UNIT, &manifest, &token.hash(), 43);
    let token_exec = execute(crank, token_hash, &[ROOT_UNIT], &token);
    let raw = Payload::Raw {
        origin: ROOT_UNIT,
        program: Pubkey::default(),
        accounts: vec![(agent_pda(ROOT_UNIT), ACT_WRITABLE | ACT_SIGNER), (voter, ACT_WRITABLE)],
        data: vec![2, 0, 0, 0, 0xe8, 3, 0, 0, 0, 0, 0, 0],
    };
    let raw_hash = proposal_hash(raw.class(), ROOT_UNIT, &manifest, &raw.hash(), 44);
    let raw_exec = execute(crank, raw_hash, &[ROOT_UNIT], &raw);
    let vault_ix = create_vault(voter, ROOT_UNIT, mint, TOKEN_PROGRAM_ID);
    // tier 2: the economy
    let allowance = Payload::Allowance { origin: ROOT_UNIT, payee: *voter.as_array(), payee_kind: HOLDER_KEY, mint: Pubkey::default(), cap: 1_000_000, period: 1_000, expires: 0 };
    let allowance_hash = proposal_hash(allowance.class(), ROOT_UNIT, &manifest, &allowance.hash(), 45);
    let allowance_exec = execute(crank, allowance_hash, &[ROOT_UNIT], &allowance);
    let pull_ix = pull(voter, allowance_hash, ROOT_UNIT, 250, voter, None);
    let token_pull = pull(voter, allowance_hash, ROOT_UNIT, 250, crank, Some((mint, TOKEN_PROGRAM_ID)));
    let contract = Payload::Contract { origin: ROOT_UNIT, payee: poleis_client::unit_holder(1), payee_kind: HOLDER_UNIT, mint, amount: 500, deadline: 9_000, arbiter: 0, token_program: TOKEN_PROGRAM_ID };
    let contract_hash = proposal_hash(contract.class(), ROOT_UNIT, &manifest, &contract.hash(), 46);
    let contract_exec = execute(crank, contract_hash, &[ROOT_UNIT], &contract);
    let cref = ContractRef { unit: ROOT_UNIT, payee: poleis_client::unit_holder(1), payee_kind: HOLDER_UNIT, mint, payer: crank, token_program: TOKEN_PROGRAM_ID };
    let rule = Payload::Rule { origin: ROOT_UNIT, hash: contract_hash, outcome: RULE_RELEASE, contract: cref };
    let rule_hash = proposal_hash(rule.class(), ROOT_UNIT, &manifest, &rule.hash(), 47);
    let rule_exec = execute(crank, rule_hash, &[ROOT_UNIT], &rule);
    let policy = Payload::DividendPolicy { origin: ROOT_UNIT, bps: 2_000, period: 1_000 };
    let token_dividend = Payload::TokenDividend { origin: ROOT_UNIT, mint, amount: 300, token_program: TOKEN_PROGRAM_ID };
    let td_hash = proposal_hash(token_dividend.class(), ROOT_UNIT, &manifest, &token_dividend.hash(), 48);
    let td_exec = execute(crank, td_hash, &[ROOT_UNIT], &token_dividend);
    let upgrade = Payload::Upgrade { origin: ROOT_UNIT, buffer: Pubkey::new_from_array([13u8; 32]), spill: crank };
    let upgrade_hash = proposal_hash(upgrade.class(), ROOT_UNIT, &manifest, &upgrade.hash(), 49);
    let upgrade_exec = execute(crank, upgrade_hash, &[ROOT_UNIT], &upgrade);
    let hand = Payload::UpgradeAuthority { origin: ROOT_UNIT, program: poleis_client::PROGRAM_ID, new: Some(voter) };
    let burn = Payload::UpgradeAuthority { origin: ROOT_UNIT, program: poleis_client::PROGRAM_ID, new: None };
    let tip = Payload::CrankTip { origin: ROOT_UNIT, tip: 1_000_000 };
    let allow = Payload::AllowMint { origin: ROOT_UNIT, mint, cap: 900, payer: crank };
    let allow_hash = proposal_hash(allow.class(), ROOT_UNIT, &manifest, &allow.hash(), 50);
    let allow_exec = execute(crank, allow_hash, &[ROOT_UNIT], &allow);
    let init_ix = poleis_client::init_config(voter, mint);
    let list_ix = list_seats(voter, ROOT_UNIT, 3, 2, 500_000);
    let buy_ix = buy_seats(crank, voter, ROOT_UNIT, 3, 1, 500_000, ROOT_UNIT);
    // the door
    let door = Payload::Door { origin: ROOT_UNIT, price: 50_000_000 };
    let door_hash = proposal_hash(door.class(), ROOT_UNIT, &manifest, &door.hash(), 51);
    let door_exec = execute(crank, door_hash, &[ROOT_UNIT], &door);
    let admit = Payload::Admit { origin: ROOT_UNIT, holder: *voter.as_array(), root: ROOT_UNIT };
    let admit_hash = proposal_hash(admit.class(), ROOT_UNIT, &manifest, &admit.hash(), 52);
    let admit_exec = execute(crank, admit_hash, &[ROOT_UNIT], &admit);
    let refusal = Payload::Refuse { origin: ROOT_UNIT, holder: *voter.as_array() };
    let refusal_hash = proposal_hash(refusal.class(), ROOT_UNIT, &manifest, &refusal.hash(), 53);
    let refusal_exec = execute(crank, refusal_hash, &[ROOT_UNIT], &refusal);
    let knock_ix = knock(voter, ROOT_UNIT);
    let withdraw_ix = withdraw_knock(voter, ROOT_UNIT);
    println!("{{");
    println!("  \"allowance\": \"{}\",", allowance_pda(&allowance_hash));
    println!("  \"contract\": \"{}\",", contract_pda(&contract_hash));
    println!("  \"escrow\": \"{}\",", escrow_pda(&contract_hash));
    println!("  \"listing\": \"{}\",", listing_pda(ROOT_UNIT, &voter, 3));
    println!("  \"accrual\": \"{}\",", hex(&accrual_hash(ROOT_UNIT, 2)));
    println!("  \"allowanceBytes\": \"{}\",", hex(&allowance.bytes()));
    println!("  \"allowanceExecuteKeys\": [{}],", keys(&allowance_exec));
    println!("  \"pullData\": \"{}\",", hex(&pull_ix.data));
    println!("  \"pullKeys\": [{}],", keys(&pull_ix));
    println!("  \"tokenPullKeys\": [{}],", keys(&token_pull));
    println!("  \"contractBytes\": \"{}\",", hex(&contract.bytes()));
    println!("  \"contractExecuteKeys\": [{}],", keys(&contract_exec));
    println!("  \"ruleBytes\": \"{}\",", hex(&rule.bytes()));
    println!("  \"ruleExecuteKeys\": [{}],", keys(&rule_exec));
    println!("  \"policyBytes\": \"{}\",", hex(&policy.bytes()));
    println!("  \"tokenDividendBytes\": \"{}\",", hex(&token_dividend.bytes()));
    println!("  \"tokenDividendExecuteKeys\": [{}],", keys(&td_exec));
    println!("  \"upgradePda\": \"{}\",", upgrade_pda());
    println!("  \"programdata\": \"{}\",", programdata_pda());
    println!("  \"upgradeBytes\": \"{}\",", hex(&upgrade.bytes()));
    println!("  \"upgradeExecuteKeys\": [{}],", keys(&upgrade_exec));
    println!("  \"handBytes\": \"{}\",", hex(&hand.bytes()));
    println!("  \"burnBytes\": \"{}\",", hex(&burn.bytes()));
    println!("  \"tipBytes\": \"{}\",", hex(&tip.bytes()));
    println!("  \"tokenPlan0\": \"{}\",", poleis_client::token_plan_pda(ROOT_UNIT, &mint));
    println!("  \"allowBytes\": \"{}\",", hex(&allow.bytes()));
    println!("  \"allowExecuteKeys\": [{}],", keys(&allow_exec));
    println!("  \"initConfigData\": \"{}\",", hex(&init_ix.data));
    println!("  \"listData\": \"{}\",", hex(&list_ix.data));
    println!("  \"listKeys\": [{}],", keys(&list_ix));
    println!("  \"buyData\": \"{}\",", hex(&buy_ix.data));
    println!("  \"buyKeys\": [{}],", keys(&buy_ix));
    println!("  \"doorBytes\": \"{}\",", hex(&door.bytes()));
    println!("  \"doorExecuteKeys\": [{}],", keys(&door_exec));
    println!("  \"admitBytes\": \"{}\",", hex(&admit.bytes()));
    println!("  \"admitExecuteKeys\": [{}],", keys(&admit_exec));
    println!("  \"refuseBytes\": \"{}\",", hex(&refusal.bytes()));
    println!("  \"refuseExecuteKeys\": [{}],", keys(&refusal_exec));
    println!("  \"knockData\": \"{}\",", hex(&knock_ix.data));
    println!("  \"knockKeys\": [{}],", keys(&knock_ix));
    println!("  \"withdrawKnockKeys\": [{}],", keys(&withdraw_ix));
    println!("  \"pool0\": \"{}\",", pool_pda(ROOT_UNIT));
    println!("  \"knock0\": \"{}\",", knock_pda(ROOT_UNIT, &voter));
    println!("  \"vault0\": \"{}\",", vault_pda(ROOT_UNIT, &mint));
    println!("  \"agent0\": \"{}\",", agent_pda(ROOT_UNIT));
    println!("  \"genesisData\": \"{}\",", hex(&genesis_ix.data));
    println!("  \"nameBytes\": \"{}\",", hex(&name.bytes()));
    println!("  \"tokenBytes\": \"{}\",", hex(&token.bytes()));
    println!("  \"tokenExecuteKeys\": [{}],", keys(&token_exec));
    println!("  \"rawBytes\": \"{}\",", hex(&raw.bytes()));
    println!("  \"rawExecuteKeys\": [{}],", keys(&raw_exec));
    println!("  \"rawExecuteWritable\": [{}],", raw_exec.accounts.iter().map(|m| if m.is_writable { "1" } else { "0" }).collect::<Vec<_>>().join(", "));
    println!("  \"createVaultData\": \"{}\",", hex(&vault_ix.data));
    println!("  \"createVaultKeys\": [{}],", keys(&vault_ix));
    println!("  \"voter\": \"{voter}\",");
    println!("  \"config\": \"{}\",", config_pda());
    println!("  \"council\": \"{}\",", council_pda());
    println!("  \"treasury0\": \"{}\",", treasury_pda(0));
    println!("  \"registry0\": \"{}\",", registry_pda(0));
    println!("  \"sale00\": \"{}\",", sale_pda(0, 0));
    println!("  \"seatBid\": \"{}\",", seat_bid_pda(0, 0, &voter, 2));
    println!("  \"proposal\": \"{}\",", proposal_pda(&hash));
    println!("  \"session0\": \"{}\",", session_pda(&hash, 0));
    println!("  \"distribution\": \"{}\",", distribution_pda(&hash));
    println!("  \"payloadBytes\": \"{}\",", hex(&payload.bytes()));
    println!("  \"payloadHash\": \"{}\",", hex(&payload.hash()));
    println!("  \"proposalHash\": \"{}\",", hex(&hash));
    println!("  \"envelope\": \"{}\",", hex(&envelope));
    println!("  \"signature\": \"{}\",", hex(&sig));
    println!("  \"seatCommitment\": \"{}\",", hex(&seat_commitment(3_000_000, 2, &salt, &voter)));
    println!("  \"precompileData\": \"{}\",", hex(&pre.data));
    println!("  \"attestData\": \"{}\",", hex(&att.data));
    println!("  \"attestKeys\": [{}],", att.accounts.iter().map(|m| format!("\"{}\"", m.pubkey)).collect::<Vec<_>>().join(", "));
    println!("  \"proposeData\": \"{}\",", hex(&prop.data));
    println!("  \"proposeKeys\": [{}],", prop.accounts.iter().map(|m| format!("\"{}\"", m.pubkey)).collect::<Vec<_>>().join(", "));
    println!("  \"executeData\": \"{}\",", hex(&exec.data));
    println!("  \"executeKeys\": [{}]", exec.accounts.iter().map(|m| format!("\"{}\"", m.pubkey)).collect::<Vec<_>>().join(", "));
    println!("}}");
}
