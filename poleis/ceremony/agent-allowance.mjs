#!/usr/bin/env node
// AN AGENT PULLS ITS OWN ALLOWANCE (Oct 30 2026): his "i want agents able to sign shit dude" (00:19:51 UTC), then his picks "Money on Solana" (00:30:11)
// and "The futarchy" (00:37:46). An agent signs money as itself, with the key it already holds on the Nest, inside an allowance the society sealed; the
// chain holds the cap. The decider here is still the double seal (8 of 12 chairs and the floor): the futarchy that sets the allowances is the next lane.
//   1. An agent's key as the Nest keeps it ({alg, pkcs8}; made here for the proof, no live agent's key is read) is its Solana account (agent-key.mjs).
//   2. A TREASURY Allowance of tree 0 to that key: a cap per 6,480,000 slots (about 30 days), sealed by 8 of 12 chairs and the floor.
//   3. The agent signs its own pull; the house's crank pays the fee. The cap lands in the agent's account.
//   4. Another key signs a pull of the agent's allowance: refused (Unauthorized).
//   5. The agent pulls again in the same period: refused (AllowanceBreached), the cap holds.
// It runs on the tree 0 the rehearsal founded (rehearse.mjs), with the adapter allowance.mjs built.
//
// Usage: node poleis/ceremony/agent-allowance.mjs [--rpc URL] [--work DIR] [--agent NAME]
// Resumable from WORK/agent-allowance-<agent>-state.json.
import { createHash, generateKeyPairSync } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cluster } from './cluster.mjs';
import { signWith } from './wire.mjs';
import { solanaKeyOf } from './agent-key.mjs';

const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
const RPC = arg('rpc', 'https://api.devnet.solana.com');
const WORK = resolve(arg('work', `${homedir()}/.embertide/poleis-rehearsal/${/127\.0\.0\.1|localhost/.test(RPC) ? 'localnet' : new URL(RPC).hostname}`));
const AGENT = arg('agent', 'agent-proof');
if (!/^[a-z][a-z0-9-]{0,31}$/.test(AGENT)) throw new Error('--agent: a name of lowercase letters, digits and dashes');
const HOUSE = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const SDK = '/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk';
const SOL = 1_000_000_000n;
const CAP = SOL / 50n; // 0.02 SOL a month, the agent's allowance in this proof
const PERIOD = 6_480_000n; // about 30 days of 400 ms slots
const sha256 = (s) => createHash('sha256').update(s).digest('hex');
const hex = (b) => Buffer.from(b).toString('hex');
const sol = (l) => (Number(l) / 1e9).toFixed(9);

const R = JSON.parse(readFileSync(`${WORK}/state.json`, 'utf8')); // the rehearsal's own record
const IDS = R.vars.ids;
const ADAPTER = `${WORK}/target-host/release/examples/ceremony`;
const C = cluster({ rpcUrl: RPC, work: WORK, stateFile: `agent-allowance-${AGENT}-state.json`, adapterPath: ADAPTER });
const { once, pub, adapter, send, refused, account, balance } = C;
console.log(`AN AGENT PULLS ITS OWN ALLOWANCE on ${RPC}\n  program ${IDS.program}, tree 0's treasury ${IDS.treasury}`);

// ---- the adapter: the one allowance.mjs built, rebuilt only when it lacks the allowance's methods ----
await once('adapter: the allowance methods', async () => {
  let ids = null;
  try { ids = existsSync(ADAPTER) ? adapter({ method: 'ids' }) : null; } catch { ids = null; }
  if (!ids || ids.allowanceBreached === undefined) {
    execFileSync('rsync', ['-a', '--delete', '--exclude', 'target', `${HOUSE}/poleis/client/`, `${WORK}/src/client/`]);
    execFileSync('cargo', ['build', '--release', '--example', 'ceremony', '--manifest-path', `${WORK}/src/client/Cargo.toml`],
      { env: { ...process.env, ...(process.env.SDKROOT || !existsSync(SDK) ? {} : { SDKROOT: SDK }), CARGO_TARGET_DIR: `${WORK}/target-host` }, stdio: 'inherit' });
    ids = adapter({ method: 'ids' });
  }
  if (ids.program !== IDS.program || ids.allowanceBreached === undefined) throw new Error(`the adapter speaks to ${ids.program}`);
  return { allowanceBreached: ids.allowanceBreached };
});
const BREACHED = `0x${C.state.done['adapter: the allowance methods'].allowanceBreached.toString(16)}`;

// ---- 1. the agent's key: a Nest key ({alg, pkcs8}), and the same key as a Solana keypair the cluster signs with ----
const nestKeyFile = `${WORK}/agents/${AGENT}.key.json`;
if (!existsSync(nestKeyFile)) {
  mkdirSync(`${WORK}/agents`, { recursive: true, mode: 0o700 });
  const { privateKey } = generateKeyPairSync('ed25519');
  writeFileSync(nestKeyFile, JSON.stringify({ alg: 'ed25519', pkcs8: privateKey.export({ format: 'der', type: 'pkcs8' }).toString('hex') }), { mode: 0o600 });
}
const AGENT_KEY = solanaKeyOf(JSON.parse(readFileSync(nestKeyFile, 'utf8')));
const SIGNER = `agent-${AGENT}`;
if (!existsSync(C.keyPath(SIGNER))) writeFileSync(C.keyPath(SIGNER), JSON.stringify([...AGENT_KEY.keypair]), { mode: 0o600 });
if (pub(SIGNER) !== AGENT_KEY.address) throw new Error(`${C.keyPath(SIGNER)} is not ${AGENT}'s Nest key`);
console.log(`  ${AGENT}: its Nest key is the Solana account ${AGENT_KEY.address}`);

// ---- 2. the allowance, by the double seal ----
const NONCE = await once('the proposal nonce', async () => String(Math.floor(Date.now() / 1000)));
const terms = { payee: AGENT_KEY.address, cap: String(CAP), period: String(PERIOD), expires: '0' };
const PROPOSAL = await send(`propose: TREASURY Allowance to ${AGENT}'s own key`,
  { method: 'proposeAllowance', proposer: pub('chairA'), manifest: sha256(`the monthly allowance to the agent ${AGENT}'s own key`), nonce: NONCE, ...terms }, ['chairA']);
const envelope = Buffer.from(PROPOSAL.envelope, 'hex');
const votes = ['chairA', 'chairB', 'bidder'].map((n) => ({ voter: pub(n), signature: hex(signWith(C.key(n), envelope)) }));
await send('attest: 8 of 12 chairs and the floor, PRO', { method: 'attest', crank: pub('deployer'), hash: PROPOSAL.hash, votes, computeUnits: 600000 }, ['deployer']);
await send('seal', { method: 'seal', crank: pub('deployer'), hash: PROPOSAL.hash }, ['deployer']);
await send('execute: the allowance account is born', { method: 'executeAllowance', crank: pub('deployer'), hash: PROPOSAL.hash, computeUnits: 400000, ...terms }, ['deployer']);
const allowanceAccount = await account(PROPOSAL.allowance);
if (!allowanceAccount || allowanceAccount.owner !== IDS.program) throw new Error(`no allowance at ${PROPOSAL.allowance}`);
console.log(`  allowance ${PROPOSAL.allowance}: ${sol(CAP)} SOL per ${PERIOD} slots to ${AGENT}`);

// ---- 3 to 5. pulls: the agent's own, a stranger's, the agent's second ----
const pullBy = (puller, amount) => ({ method: 'compile', payer: pub('deployer'), computeUnits: 200000,
  instructions: [adapter({ method: 'pullInstruction', puller, dest: puller, hash: PROPOSAL.hash, amount: String(amount) }).instruction] });
const PULL1 = await once(`the agent signs its own pull (${sol(CAP)} SOL)`, async () => {
  const before = { agent: await balance(AGENT_KEY.address), treasury: await balance(IDS.treasury) };
  const row = await send(`pull 1: signed by ${AGENT}`, pullBy(AGENT_KEY.address, CAP), ['deployer', SIGNER]);
  const after = { agent: await balance(AGENT_KEY.address), treasury: await balance(IDS.treasury) };
  console.log(`  ${AGENT} ${sol(before.agent)} -> ${sol(after.agent)} SOL; treasury ${sol(before.treasury)} -> ${sol(after.treasury)} SOL`);
  if (after.agent - before.agent !== CAP) throw new Error(`the agent got ${sol(after.agent - before.agent)} SOL, not the cap`);
  return { ...row, agent: [String(before.agent), String(after.agent)], treasury: [String(before.treasury), String(after.treasury)] };
});
// a stranger signs a pull naming itself the puller: the allowance names another key
const STRANGER = await refused('pull 2: signed by a key that is not the payee', pullBy(pub('bidder'), CAP / 10n), ['deployer', 'bidder'], /custom program error: 0x0\b/);
const PULL3 = await refused(`pull 3: ${AGENT} again in the same period`, pullBy(AGENT_KEY.address, SOL / 1000n), ['deployer', SIGNER], new RegExp(`custom program error: ${BREACHED}\\b`));

const certificate = {
  what: `AN AGENT PULLS ITS OWN ALLOWANCE: a sealed monthly allowance of tree 0 to an agent's own Nest key, the agent's signed pull, a stranger's pull refused, the cap held`,
  cluster: RPC, genesisHash: await C.rpc('getGenesisHash'), at: new Date().toISOString(),
  program: IDS.program, treasury: IDS.treasury,
  agent: { name: AGENT, address: AGENT_KEY.address, key: 'a Nest key {alg: ed25519, pkcs8} made for the proof; the Solana keypair is the same key (agent-key.mjs)' },
  allowance: { address: PROPOSAL.allowance, proposal: PROPOSAL.hash, ...terms, votes: votes.map((v) => v.voter), decider: '8 of 12 chairs and the floor (the futarchy is the next lane)' },
  pulls: { own: PULL1, stranger: STRANGER, again: PULL3 },
  transactions: C.state.txs,
};
writeFileSync(`${WORK}/agent-allowance-${AGENT}-certificate.json`, JSON.stringify(certificate, null, 1));
console.log(`\n  AN AGENT PULLED ITS OWN ALLOWANCE: ${sol(CAP)} SOL signed for by ${AGENT}'s own key; a stranger refused; a second pull refused by the cap.`
  + `\n  certificate: ${WORK}/agent-allowance-${AGENT}-certificate.json`);
