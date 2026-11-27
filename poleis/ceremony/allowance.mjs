#!/usr/bin/env node
// THE ALLOWANCE REACHES THE VAULT (Oct 28 2026): his money road's middle, proven on chain. His pick of 15:12: the mint pays the treasury,
// a monthly allowance goes to his Keystone 2-of-3. The road plan's finding 7 called this unproven: a key payee must sign its own pull,
// and a Squads vault signs only from inside a vault transaction its members approve.
//   1. A Squads v4 multisig of three keys (stand-ins for his three Keystones, the same three that hold the twelve chairs), two to approve.
//   2. A deposit into unit 0's treasury by a plain transfer: the mint's own path.
//   3. A TREASURY Allowance to the vault, a cap per 6,480,000 slots (about 30 days), sealed by 8 of 12 chairs and the floor.
//   4. The month's pull through the vault: refused with one approval; with two, the cap lands in the vault.
//   5. A second pull in the same period: refused by the program, the cap holds.
// It runs on the tree 0 the rehearsal founded (rehearse.mjs): same work folder, same keys, its adapter rebuilt with the allowance.
//
// Usage: node poleis/ceremony/allowance.mjs [--rpc URL] [--work DIR] --bank KEYPAIR
// Resumable from WORK/allowance-state.json.
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cluster } from './cluster.mjs';
import { signWith } from './wire.mjs';
import * as S from './squads.mjs';

const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
const RPC = arg('rpc', 'https://api.devnet.solana.com');
const WORK = resolve(arg('work', `${homedir()}/.embertide/poleis-rehearsal/${/127\.0\.0\.1|localhost/.test(RPC) ? 'localnet' : new URL(RPC).hostname}`));
const BANK = arg('bank', null);
const HOUSE = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const SDK = '/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk';
const SOL = 1_000_000_000n;
const CAP = (3n * SOL) / 10n; // the month's allowance in this proof
const PERIOD = 6_480_000n; // about 30 days of 400 ms slots
const sha256 = (s) => createHash('sha256').update(s).digest('hex');
const hex = (b) => Buffer.from(b).toString('hex');
const sol = (l) => (Number(l) / 1e9).toFixed(9);

const R = JSON.parse(readFileSync(`${WORK}/state.json`, 'utf8')); // the rehearsal's own record
const IDS = R.vars.ids;
const ADAPTER = `${WORK}/target-host/release/examples/ceremony`;
const C = cluster({ rpcUrl: RPC, work: WORK, stateFile: 'allowance-state.json', adapterPath: ADAPTER });
const { once, pub, keyPath, adapter, send, refused, cli, account, balance } = C;
console.log(`THE ALLOWANCE REACHES THE VAULT on ${RPC}\n  program ${IDS.program}, tree 0's treasury ${IDS.treasury}`);

// ---- the adapter, rebuilt from the same patched copy with the allowance's methods ----
await once('adapter: the allowance methods', async () => {
  execFileSync('rsync', ['-a', '--delete', '--exclude', 'target', `${HOUSE}/poleis/client/`, `${WORK}/src/client/`]);
  execFileSync('cargo', ['build', '--release', '--example', 'ceremony', '--manifest-path', `${WORK}/src/client/Cargo.toml`],
    { env: { ...process.env, ...(process.env.SDKROOT || !existsSync(SDK) ? {} : { SDKROOT: SDK }), CARGO_TARGET_DIR: `${WORK}/target-host` }, stdio: 'inherit' });
  const ids = adapter({ method: 'ids' });
  if (ids.program !== IDS.program || ids.allowanceBreached === undefined) throw new Error(`the adapter speaks to ${ids.program}`);
  return { allowanceBreached: ids.allowanceBreached };
});
const BREACHED = `0x${C.state.done['adapter: the allowance methods'].allowanceBreached.toString(16)}`;

// ---- funding: devnet SOL, from the bank ----
await once('fund the keys from the bank', async () => {
  if (!BANK) throw new Error('--bank is needed: the faucet refuses fresh keys');
  const plan = [['deployer', SOL / 10n], ['chairA', SOL / 4n], ['chairB', SOL / 50n], ['buyer', SOL + SOL / 10n]];
  for (const [name, l] of plan) {
    cli(['transfer', '--keypair', BANK, '--fee-payer', BANK, '--allow-unfunded-recipient', pub(name), sol(l)]);
    console.log(`  funded ${name.padEnd(9)} ${sol(l)} SOL from the bank`);
  }
});

// ---- 1. the multisig: three keys, two to approve ----
const pda = (seeds) => adapter({ method: 'pda', program: S.SQUADS, seeds }).address;
const MEMBERS = ['chairA', 'chairB', 'chairC'].map(pub);
const MS = await once('the Squads addresses', async () => {
  const multisig = pda(S.seeds.multisig(pub('squads-create')));
  const programConfig = pda(S.seeds.programConfig());
  const config = await account(programConfig);
  if (!config || config.owner !== S.SQUADS) throw new Error(`no Squads program config at ${programConfig}`);
  const { b58encode } = await import('./wire.mjs');
  return { multisig, vault: pda(S.seeds.vault(multisig)), programConfig, treasury: b58encode(S.programConfigTreasury(config.data)) };
});
const compile = (payer, ixs, extra = {}) => ({ method: 'compile', payer: pub(payer), instructions: ixs, ...extra });
await send('create the 2-of-3 multisig',
  compile('chairA', [S.multisigCreateV2({ programConfig: MS.programConfig, treasury: MS.treasury, multisig: MS.multisig, createKey: pub('squads-create'), creator: pub('chairA'), members: MEMBERS, threshold: 2 })]),
  ['chairA', 'squads-create']);
console.log(`  multisig ${MS.multisig}, its vault ${MS.vault}`);

// ---- 2. the deposit: the mint's path, a plain transfer into the treasury ----
const DEPOSIT = await once('deposit 1 SOL into the treasury, a plain transfer', async () => {
  const before = await balance(IDS.treasury);
  const out = cli(['transfer', '--keypair', keyPath('buyer'), '--fee-payer', keyPath('buyer'), IDS.treasury, '1']);
  const signature = (out.match(/Signature: (\w+)/) || [])[1];
  if (!signature) throw new Error(`no signature in: ${out}`);
  const after = await balance(IDS.treasury);
  console.log(`  treasury ${sol(before)} -> ${sol(after)} SOL (${signature.slice(0, 12)}…)`);
  return { signature, before: String(before), after: String(after) };
});
await send('book it: sync_endowment, anyone may', { method: 'syncEndowment', crank: pub('deployer') }, ['deployer']);

// ---- 3. the allowance, by the double seal ----
const allowanceTerms = { payee: MS.vault, cap: String(CAP), period: String(PERIOD), expires: '0' };
const PROPOSAL = await send('propose: TREASURY Allowance to the vault',
  { method: 'proposeAllowance', proposer: pub('chairA'), manifest: sha256('the monthly allowance to his three Keystones'), nonce: '2', ...allowanceTerms }, ['chairA']);
const envelope = Buffer.from(PROPOSAL.envelope, 'hex');
const votes = ['chairA', 'chairB', 'bidder'].map((n) => ({ voter: pub(n), signature: hex(signWith(C.key(n), envelope)) }));
await send('attest: 8 of 12 chairs and the floor, PRO', { method: 'attest', crank: pub('deployer'), hash: PROPOSAL.hash, votes, computeUnits: 600000 }, ['deployer']);
await send('seal', { method: 'seal', crank: pub('deployer'), hash: PROPOSAL.hash }, ['deployer']);
await send('execute: the allowance account is born', { method: 'executeAllowance', crank: pub('deployer'), hash: PROPOSAL.hash, computeUnits: 400000, ...allowanceTerms }, ['deployer']);
const allowanceAccount = await account(PROPOSAL.allowance);
if (!allowanceAccount || allowanceAccount.owner !== IDS.program) throw new Error(`no allowance at ${PROPOSAL.allowance}`);
console.log(`  allowance ${PROPOSAL.allowance}: ${sol(CAP)} SOL per ${PERIOD} slots to the vault`);

// ---- 4 and 5. pulls through the vault ----
async function vaultPull(n, amount, { approvals, expect }) {
  const idx = await once(`pull ${n}: its transaction index`, async () => String(S.multisigTransactionIndex((await account(MS.multisig)).data) + 1n));
  const transaction = pda(S.seeds.transaction(MS.multisig, idx));
  const proposal = pda(S.seeds.proposal(MS.multisig, idx));
  const inner = adapter({ method: 'pullInstruction', puller: MS.vault, dest: MS.vault, hash: PROPOSAL.hash, amount: String(amount) }).instruction;
  const msg = S.transactionMessage(MS.vault, inner);
  await send(`pull ${n}: the vault transaction (${sol(amount)} SOL)`,
    compile('chairA', [S.vaultTransactionCreate({ multisig: MS.multisig, transaction, creator: pub('chairA'), rentPayer: pub('chairA'), message: msg.bytes }),
      S.proposalCreate({ multisig: MS.multisig, proposal, creator: pub('chairA'), rentPayer: pub('chairA'), index: idx })]), ['chairA']);
  const execute = compile('chairA', [S.vaultTransactionExecute({ multisig: MS.multisig, proposal, transaction, member: pub('chairA'), keys: msg.keys })], { computeUnits: 400000 });
  for (const [i, member] of approvals.entries()) {
    await send(`pull ${n}: approved by ${member}`, compile(member, [S.proposalApprove({ multisig: MS.multisig, member: pub(member), proposal })]), [member]);
    if (i === 0 && approvals.length > 1 && expect === 'lands') {
      await refused(`pull ${n}: executed with one approval of three`, execute, ['chairA'], /InvalidProposalStatus|Error Code/);
    }
  }
  if (expect === 'lands') {
    const before = { vault: await balance(MS.vault), treasury: await balance(IDS.treasury) };
    const row = await send(`pull ${n}: executed with two of three`, execute, ['chairA']);
    const after = { vault: await balance(MS.vault), treasury: await balance(IDS.treasury) };
    console.log(`  vault ${sol(before.vault)} -> ${sol(after.vault)} SOL; treasury ${sol(before.treasury)} -> ${sol(after.treasury)} SOL`);
    return { ...row, vault: [String(before.vault), String(after.vault)], treasury: [String(before.treasury), String(after.treasury)] };
  }
  return refused(`pull ${n}: past the month's cap`, execute, ['chairA'], new RegExp(`custom program error: ${BREACHED}`));
}
const PULL1 = await vaultPull(1, CAP, { approvals: ['chairA', 'chairB'], expect: 'lands' });
const PULL2 = await vaultPull(2, SOL / 100n, { approvals: ['chairA', 'chairB'], expect: 'refused' });

const certificate = {
  what: 'THE ALLOWANCE REACHES THE VAULT: a deposit into tree 0, a sealed monthly allowance to a Squads 2-of-3 vault, one pull, the cap held',
  cluster: RPC, genesisHash: await C.rpc('getGenesisHash'), at: new Date().toISOString(),
  program: IDS.program, treasury: IDS.treasury, squads: { program: S.SQUADS, ...MS, members: MEMBERS, threshold: 2 },
  deposit: DEPOSIT, allowance: { address: PROPOSAL.allowance, proposal: PROPOSAL.hash, ...allowanceTerms, votes: votes.map((v) => v.voter) },
  pulls: { first: PULL1, second: PULL2, refusedWithOneApproval: C.state.done['pull 1: executed with one approval of three'] },
  transactions: C.state.txs,
};
writeFileSync(`${WORK}/allowance-certificate.json`, JSON.stringify(certificate, null, 1));
console.log(`\n  THE ALLOWANCE REACHED THE VAULT: ${sol(CAP)} SOL pulled by two of three, a second pull refused by the cap.\n  certificate: ${WORK}/allowance-certificate.json`);
