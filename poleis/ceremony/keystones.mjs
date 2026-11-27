#!/usr/bin/env node
// HIS KEYSTONES APPROVE (Oct 28 2026, his go by the box: "set up the devnet 2-of-3 with my three Keystones as members and rerun the
// allowance proof with my approvals"). The allowance proof of allowance.mjs, with his own devices as the vault's voters:
//   1. A Squads v4 multisig: his three Keystones vote (two of three), and a house key may only initiate and execute, never vote.
//   2. A TREASURY Allowance of tree 0 to that vault, sealed by the chairs (still stand-ins: his Keystones in the chairs are next).
//   3. The month's pull, prepared by the house key, waits for his approvals on the page (poleis/ceremony/keystone/), which Solflare
//      signs with his Keystones by QR.
//   4. After one approval the house tries to execute and must be refused; after two it executes, and the vault gets the month's cap.
// Usage: node poleis/ceremony/keystones.mjs --members K1,K2,K3 --bank KEYPAIR [--page URL] [--rpc URL] [--work DIR] [--label NAME] [--cap SOL]
// --label keeps a dress rehearsal (the page driven by test keys) apart from his own run: its state, its multisig, its certificate.
// Resumable from WORK/keystones-state.json; the watch for his approvals lasts an hour at most, and a rerun picks it up.
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { resolve } from 'node:path';
import { cluster } from './cluster.mjs';
import { b58decode, b58encode, signWith } from './wire.mjs';
import * as S from './squads.mjs';

const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
const RPC = arg('rpc', 'https://api.devnet.solana.com');
const WORK = resolve(arg('work', `${homedir()}/.embertide/poleis-rehearsal/${new URL(RPC).hostname}`));
const BANK = arg('bank', null);
const PAGE = arg('page', 'http://127.0.0.1:8813/');
const LABEL = arg('label', 'keystones');
const SOL = 1_000_000_000n;
// the month in this proof: 0.3 SOL unless --cap names another (in SOL); the plan's cap per period bounds every allowance together
const CAP = BigInt(Math.round(Number(arg('cap', '0.3')) * 1e9));
const PERIOD = 6_480_000n;
const sol = (l) => (Number(l) / 1e9).toFixed(9);
const sha256 = (s) => createHash('sha256').update(s).digest('hex');
const hex = (b) => Buffer.from(b).toString('hex');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const KEYSTONES = (arg('members', '') || '').split(',').map((s) => s.trim()).filter(Boolean);
if (KEYSTONES.length !== 3 || new Set(KEYSTONES).size !== 3 || !KEYSTONES.every((k) => b58decode(k).length === 32)) {
  throw new Error('--members needs his three Keystone addresses, distinct, comma-separated');
}
const R = JSON.parse(readFileSync(`${WORK}/state.json`, 'utf8'));
const IDS = R.vars.ids;
const C = cluster({ rpcUrl: RPC, work: WORK, stateFile: `${LABEL}-state.json`, adapterPath: `${WORK}/target-host/release/examples/ceremony` });
const { once, pub, adapter, send, refused, cli, account, balance } = C;
console.log(`HIS KEYSTONES APPROVE on ${RPC}\n  tree 0's treasury ${IDS.treasury}\n  his Keystones ${KEYSTONES.join(', ')}`);

await once('fund the fees from the bank', async () => {
  if (!BANK) throw new Error('--bank is needed');
  const plan = [...KEYSTONES.map((k, i) => [`Keystone ${i + 1}`, k, SOL / 100n]), ['the house key', pub('deployer'), SOL / 20n], ['chairA', pub('chairA'), SOL / 20n]];
  for (const [name, k, l] of plan) {
    cli(['transfer', '--keypair', BANK, '--fee-payer', BANK, '--allow-unfunded-recipient', k, sol(l)]);
    console.log(`  funded ${name.padEnd(13)} ${sol(l)} SOL`);
  }
});

// ---- 1. his vault: his Keystones vote, the house key only initiates and executes ----
const pda = (seeds) => adapter({ method: 'pda', program: S.SQUADS, seeds }).address;
const MS = await once('the Squads addresses', async () => {
  const multisig = pda(S.seeds.multisig(pub(`squads-create-${LABEL}`)));
  const programConfig = pda(S.seeds.programConfig());
  const config = await account(programConfig);
  return { multisig, vault: pda(S.seeds.vault(multisig)), programConfig, treasury: b58encode(S.programConfigTreasury(config.data)) };
});
const compile = (payer, ixs, extra = {}) => ({ method: 'compile', payer: pub(payer), instructions: ixs, ...extra });
const MEMBERS = [...KEYSTONES, { key: pub('deployer'), mask: S.PERMISSIONS.initiate | S.PERMISSIONS.execute }];
await send('create his 2-of-3: three Keystones vote, the house key cannot',
  compile('deployer', [S.multisigCreateV2({ programConfig: MS.programConfig, treasury: MS.treasury, multisig: MS.multisig, createKey: pub(`squads-create-${LABEL}`), creator: pub('deployer'), members: MEMBERS, threshold: 2 })]),
  ['deployer', `squads-create-${LABEL}`]);
console.log(`  his multisig ${MS.multisig}, his vault ${MS.vault}`);

// ---- 2. the allowance to his vault, by the seal ----
const terms = { payee: MS.vault, cap: String(CAP), period: String(PERIOD), expires: '0' };
const PROPOSAL = await send('propose: TREASURY Allowance to his vault',
  { method: 'proposeAllowance', proposer: pub('chairA'), manifest: sha256('the monthly allowance to his own three Keystones'), nonce: LABEL === 'keystones' ? '3' : '4', ...terms }, ['chairA']);
const envelope = Buffer.from(PROPOSAL.envelope, 'hex');
const votes = ['chairA', 'chairB', 'bidder'].map((n) => ({ voter: pub(n), signature: hex(signWith(C.key(n), envelope)) }));
await send('attest: 8 of 12 chairs and the floor, PRO', { method: 'attest', crank: pub('deployer'), hash: PROPOSAL.hash, votes, computeUnits: 600000 }, ['deployer']);
await send('seal', { method: 'seal', crank: pub('deployer'), hash: PROPOSAL.hash }, ['deployer']);
await send('execute: his allowance is born', { method: 'executeAllowance', crank: pub('deployer'), hash: PROPOSAL.hash, computeUnits: 400000, ...terms }, ['deployer']);

// ---- 3. the month's pull, prepared by the house key, waiting for him ----
const IDX = await once('the pull: its transaction index', async () => String(S.multisigTransactionIndex((await account(MS.multisig)).data) + 1n));
const transaction = pda(S.seeds.transaction(MS.multisig, IDX));
const proposal = pda(S.seeds.proposal(MS.multisig, IDX));
const inner = adapter({ method: 'pullInstruction', puller: MS.vault, dest: MS.vault, hash: PROPOSAL.hash, amount: String(CAP) }).instruction;
const msg = S.transactionMessage(MS.vault, inner);
await send(`the pull: prepared by the house key (${sol(CAP)} SOL)`,
  compile('deployer', [S.vaultTransactionCreate({ multisig: MS.multisig, transaction, creator: pub('deployer'), rentPayer: pub('deployer'), message: msg.bytes }),
    S.proposalCreate({ multisig: MS.multisig, proposal, creator: pub('deployer'), rentPayer: pub('deployer'), index: IDX })]), ['deployer']);
const link = `${PAGE}#${new URLSearchParams({ rpc: RPC, squads: S.SQUADS, multisig: MS.multisig, proposal, vault: MS.vault, treasury: IDS.treasury, amount: String(CAP), members: KEYSTONES.join(','), threshold: '2' })}`;
writeFileSync(`${WORK}/${LABEL}-link.txt`, `${link}\n`);
console.log(`\n  THE PAGE FOR HIS APPROVALS:\n  ${link}\n`);

// ---- 4. his approvals: one is not enough, two pull the month ----
const execute = compile('deployer', [S.vaultTransactionExecute({ multisig: MS.multisig, proposal, transaction, member: pub('deployer'), keys: msg.keys })], { computeUnits: 400000 });
const state = async () => S.proposalState((await account(proposal)).data);
let seen = -1;
for (let t0 = Date.now(); Date.now() - t0 < 3_600_000; await sleep(5000)) {
  const p = await state();
  if (p.approved.length !== seen) {
    seen = p.approved.length;
    console.log(`  his approvals: ${seen} of 2 (${p.approved.map((a) => b58encode(a).slice(0, 6)).join(', ') || 'none yet'}), ${p.status}`);
  }
  if (p.status === 'Active' && seen === 1) await refused('one Keystone is not enough: the house key cannot execute', execute, ['deployer'], /InvalidProposalStatus/);
  if (p.status === 'Approved') break;
  if (p.status === 'Executed') break;
}
const final = await state();
if (final.status !== 'Approved' && final.status !== 'Executed') { console.log('  no second approval within the hour: rerun to keep waiting'); process.exit(2); }
const before = await once('balances before the pull', async () => ({ vault: String(await balance(MS.vault)), treasury: String(await balance(IDS.treasury)) }));
await send('two Keystones approved: the house key executes the pull', execute, ['deployer']);
const after = { vault: String(await balance(MS.vault)), treasury: String(await balance(IDS.treasury)) };
console.log(`  his vault ${sol(before.vault)} -> ${sol(after.vault)} SOL; treasury ${sol(before.treasury)} -> ${sol(after.treasury)} SOL`);

const certificate = {
  what: 'HIS KEYSTONES APPROVE: his three Keystones vote on the vault the allowance pays; two approved, the month landed',
  cluster: RPC, genesisHash: await C.rpc('getGenesisHash'), at: new Date().toISOString(),
  program: IDS.program, treasury: IDS.treasury, keystones: KEYSTONES, houseKey: { key: pub('deployer'), may: 'initiate and execute, never vote' },
  squads: { ...MS, threshold: 2, transactionIndex: IDX, proposal, transaction },
  allowance: { address: PROPOSAL.allowance, proposal: PROPOSAL.hash, ...terms },
  approvals: final.approved.map((a) => b58encode(a)), oneWasNotEnough: C.state.done['one Keystone is not enough: the house key cannot execute'] || 'his two approvals landed before the house tried',
  pull: { before, after }, transactions: C.state.txs,
};
writeFileSync(`${WORK}/${LABEL}-certificate.json`, JSON.stringify(certificate, null, 1));
console.log(`\n  HIS KEYSTONES MOVED THE MONTH: ${sol(CAP)} SOL into his vault by two of his three approvals.\n  certificate: ${WORK}/${LABEL}-certificate.json`);
