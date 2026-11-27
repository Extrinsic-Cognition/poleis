#!/usr/bin/env node
// THE CEREMONY, REHEARSED (his go, Oct 28 2026): the mainnet founding of the Poleis program, run end to end on a test cluster
// under a fresh program id, in the order of plans/the-road-to-mainnet-2026-10-28.md and the Sept 9 launch plan's section 6.
//   1. Fresh keys, none of them a key that lived before.
//   2. The program built for sBPF v3 under the fresh id, and a second build for the upgrade: the same source compiled as one
//      codegen unit, a different binary of the same law (an exported marker would make the linker stamp the ELF as GNU, which the
//      loader refuses). A fresh id moves three constants of lib.rs, not one: the declared id, FUND_ADDRESS and FUND_BUMP (the
//      fund's ["fund"] PDA is fixed there, not searched).
//   3. Deploy, then tree 0 founded in the same breath: init_config and open_genesis in one transaction, so nobody takes root 0.
//   4. The genesis sale: one seat sealed, revealed, settled and claimed; the council ratified.
//   5. The code's authority handed to the program's own upgrade address.
//   6. An upgrade by the seal: a chair proposes, eight of the twelve chairs and the floor say PRO, the seal, the execute.
//   7. The birth certificate: every address, hash, signature and slot, read back from the chain.
//
// Usage: node poleis/ceremony/rehearse.mjs [--rpc URL] [--work DIR] [--bank KEYPAIR]
//   --rpc   the cluster (default a local test validator at http://127.0.0.1:8899)
//   --work  the keys, the builds and the state, outside git (default ~/.embertide/poleis-rehearsal/<cluster>)
//   --bank  a funded keypair that pays every fresh key by transfer; without it the faucet pays (localnet)
// Resumable: a run that stops continues from WORK/state.json; delete WORK to start over.
// The encodings are the program's own client (poleis/client/examples/ceremony.rs), built from the same patched copy that is deployed.
import { createHash, randomBytes } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { chmodSync, existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { b58decode, checkKeypair, freshKeypair, publicOf, signMessage, signWith } from './wire.mjs';

const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
const RPC = arg('rpc', 'http://127.0.0.1:8899');
const LOCAL = /127\.0\.0\.1|localhost/.test(RPC);
const WORK = resolve(arg('work', `${homedir()}/.embertide/poleis-rehearsal/${LOCAL ? 'localnet' : new URL(RPC).hostname}`));
const BANK = arg('bank', null);
const HOUSE = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
// the Solana CLI: SOLANA_BIN, else the installer's active release, else the newest release that holds one
function solanaBin() {
  if (process.env.SOLANA_BIN) return process.env.SOLANA_BIN;
  const root = `${homedir()}/.local/share/solana/install`;
  const found = [`${root}/active_release/bin`,
    ...(existsSync(`${root}/releases`) ? readdirSync(`${root}/releases`).map((r) => `${root}/releases/${r}/solana-release/bin`) : [])]
    .filter((d) => existsSync(`${d}/solana`) && existsSync(`${d}/cargo-build-sbf`));
  if (!found.length) throw new Error('no Solana CLI: set SOLANA_BIN');
  return found.sort((x, y) => statSync(`${y}/solana`).mtimeMs - statSync(`${x}/solana`).mtimeMs)[0];
}
const BIN = solanaBin();
// the 27.0 SDK's linker refuses the host build scripts; the 26.5 SDK links them (the house's standing workaround)
const SDK = '/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk';
const BUILD_ENV = { ...process.env, ...(process.env.SDKROOT || !existsSync(SDK) ? {} : { SDKROOT: SDK }) };

const SOL = 1_000_000_000n;
const SLOTS = LOCAL ? 40n : 150n; // the sale's commit and reveal windows: a few seconds on localnet, a minute on a public cluster
const WINDOW = LOCAL ? 100n : 300n; // the decision window; 100 slots is the program's floor
const PRICE = SOL / 50n, RESERVE = SOL / 100n, DEPOSIT = SOL / 20n;
const CHAIRS = ['chairA', 'chairB', 'chairC']; // stand-ins for his three Keystones, four chairs each

const say = (s) => console.log(s);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const sha256 = (b) => createHash('sha256').update(b).digest('hex');
const hex = (b) => Buffer.from(b).toString('hex');

// ---- the state: keys by name, values and steps done, so a run resumes ----
mkdirSync(`${WORK}/keys`, { recursive: true, mode: 0o700 });
chmodSync(`${WORK}/keys`, 0o700);
const STATE = `${WORK}/state.json`;
const state = existsSync(STATE) ? JSON.parse(readFileSync(STATE, 'utf8')) : { rpc: RPC, vars: {}, done: {}, txs: [] };
if (state.rpc !== RPC) throw new Error(`${STATE} belongs to ${state.rpc}; use another --work for ${RPC}`);
const save = () => writeFileSync(STATE, JSON.stringify(state, null, 1));
const keyPath = (name) => `${WORK}/keys/${name}.json`;
function key(name) {
  const p = keyPath(name);
  if (!existsSync(p)) writeFileSync(p, JSON.stringify([...freshKeypair()]), { mode: 0o600 });
  return checkKeypair(Uint8Array.from(JSON.parse(readFileSync(p, 'utf8'))));
}
const pub = (name) => publicOf(key(name));
async function once(step, fn) {
  if (state.done[step]) { say(`  ${step.padEnd(46)} (done before)`); return state.done[step]; }
  const v = (await fn()) ?? true;
  state.done[step] = v;
  save();
  return v;
}

// ---- the cluster ----
let rpcId = 0;
async function rpc(method, params = []) {
  for (let i = 0, wait = 500; ; i++, wait = Math.min(wait * 2, 15_000)) {
    try {
      const r = await fetch(RPC, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', id: ++rpcId, method, params }) });
      if (r.status === 429 && i < 8) { await sleep(wait); continue; }
      const j = await r.json();
      if (j.error) { const e = new Error(`${method}: ${j.error.message}`); e.logs = j.error.data?.logs; throw e; }
      return j.result;
    } catch (e) {
      if (e.logs || i >= 8 || !/fetch failed|ECONN|socket/i.test(String(e.message || e))) throw e;
      await sleep(wait);
    }
  }
}
const slot = async () => BigInt(await rpc('getSlot', [{ commitment: 'confirmed' }]));
async function waitSlot(target) { while ((await slot()) <= target) await sleep(LOCAL ? 300 : 1500); }
const blockhash = async () => (await rpc('getLatestBlockhash', [{ commitment: 'confirmed' }])).value.blockhash;
const rent = async (size) => BigInt(await rpc('getMinimumBalanceForRentExemption', [size]));

/** The Solana CLI against this cluster; its JSON parsed when asked. */
function cli(args, { json = false } = {}) {
  const r = spawnSync(`${BIN}/solana`, [...args, '--url', RPC, '--commitment', 'confirmed', ...(json ? ['--output', 'json'] : [])], { encoding: 'utf8' });
  if (r.status !== 0) throw new Error(`solana ${args.slice(0, 2).join(' ')}: ${(r.stderr || r.stdout).trim()}`);
  return json ? JSON.parse(r.stdout) : r.stdout.trim();
}

/** A program's bytes go up in hundreds of writes. A public RPC rate-limits them, so a public cluster sends them straight to the
 *  validators with more signing attempts, and a cut-short upload resumes from its named buffer, at most three times. */
const WRITE_FLAGS = LOCAL ? ['--use-rpc'] : ['--max-sign-attempts', '30'];
function cliWrites(args, opts) {
  for (let i = 1; ; i++) {
    try { return cli([...args, ...WRITE_FLAGS], opts); } catch (e) {
      if (i >= 3 || !/retries|timed out|blockhash|429|Too many requests/i.test(e.message)) throw e;
      say(`  the upload was cut short (${e.message.split('\n')[0].slice(0, 90)}); resuming from its buffer, try ${i + 1} of 3`);
    }
  }
}

/** One request to the adapter: the program's own encodings, compiled to a message. */
function adapter(req) {
  const r = spawnSync(state.vars.adapter, { input: JSON.stringify(req), encoding: 'utf8' });
  const out = JSON.parse(r.stdout || '{}');
  if (!out.ok) throw new Error(`adapter ${req.method}: ${out.error || r.stderr}`);
  return out;
}

/** Compile by the adapter, sign with the named keys, send, confirm, read back slot, fee and compute. */
async function send(step, req, signerNames) {
  return once(step, async () => {
    const out = adapter({ ...req, blockhash: await blockhash() });
    const { wire, signature } = signMessage(Buffer.from(out.messageBase64, 'base64'), out.signers, signerNames.map(key));
    try {
      await rpc('sendTransaction', [Buffer.from(wire).toString('base64'), { encoding: 'base64', preflightCommitment: 'confirmed' }]);
    } catch (e) {
      throw new Error(`${step}: ${e.message}\n    ${(e.logs || []).join('\n    ')}`);
    }
    for (let t0 = Date.now(); ; await sleep(LOCAL ? 300 : 1500)) {
      const st = (await rpc('getSignatureStatuses', [[signature]])).value[0];
      if (st?.err) throw new Error(`${step}: ${signature} failed: ${JSON.stringify(st.err)}`);
      if (st && (st.confirmationStatus === 'confirmed' || st.confirmationStatus === 'finalized')) break;
      if (Date.now() - t0 > 90_000) throw new Error(`${step}: ${signature} never confirmed`);
    }
    let got = null;
    for (let i = 0; i < 10 && !got; i++) {
      got = await rpc('getTransaction', [signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0, encoding: 'json' }]);
      if (!got) await sleep(1000);
    }
    const row = { step, signature, slot: got?.slot ?? null, fee: got?.meta?.fee ?? null, cu: got?.meta?.computeUnitsConsumed ?? null, bytes: wire.length };
    state.txs.push(row);
    say(`  ${step.padEnd(46)} slot ${row.slot}  ${row.bytes} B  ${row.cu} CU  ${signature.slice(0, 12)}…`);
    return { ...row, ...(out.hash ? { hash: out.hash, envelope: out.envelope } : {}) };
  });
}

// ---- 1. fresh keys ----
say(`THE CEREMONY, REHEARSED on ${RPC}\n  work ${WORK}`);
for (const n of ['program', 'deployer', ...CHAIRS, 'bidder', 'buffer', 'deploy-buffer']) key(n);
const PROGRAM = pub('program');
const DEPLOYER = pub('deployer');
say(`  program id ${PROGRAM} (fresh)\n  deployer   ${DEPLOYER} (fresh)`);

// ---- 2. the builds, under the fresh id ----
const SRC = `${WORK}/src`;
await once('build: the patched copy and the adapter', async () => {
  mkdirSync(SRC, { recursive: true });
  for (const part of ['program', 'client']) {
    execFileSync('rsync', ['-a', '--delete', '--exclude', 'target', `${HOUSE}/poleis/${part}/`, `${SRC}/${part}/`]);
  }
  const lib = `${SRC}/program/poleis_treasury/src/lib.rs`;
  let text = readFileSync(lib, 'utf8');
  const swap = (re, to, what) => {
    const n = (text.match(new RegExp(re.source, 'g')) || []).length;
    if (n !== 1) throw new Error(`${what}: ${n} matches in lib.rs, expected 1`);
    text = text.replace(re, to);
  };
  swap(/declare_id!\("[1-9A-HJ-NP-Za-km-z]{32,44}"\)/, `declare_id!("${PROGRAM}")`, 'the declared id');
  const fund = cli(['find-program-derived-address', PROGRAM, 'string:fund'], { json: true });
  const fundBytes = [...b58decode(fund.address)].map((b) => `0x${b.toString(16).padStart(2, '0')}`);
  const bump = fund.bumpSeed; // the CLI's JSON: { address, seeds, bumpSeed }
  if (!Number.isInteger(bump) || bump < 0 || bump > 255) throw new Error(`no bump for the fund PDA: ${JSON.stringify(fund)}`);
  swap(/pub const FUND_ADDRESS: Address = Address::new_from_array\(\[[^\]]*\]\);/,
    `pub const FUND_ADDRESS: Address = Address::new_from_array([\n    ${fundBytes.slice(0, 16).join(', ')},\n    ${fundBytes.slice(16).join(', ')},\n]);`, 'FUND_ADDRESS');
  swap(/pub const FUND_BUMP: u8 = \d+;/, `pub const FUND_BUMP: u8 = ${bump};`, 'FUND_BUMP');
  writeFileSync(lib, text);
  state.vars.fund = { address: fund.address, bump };
  execFileSync('cargo', ['build', '--release', '--example', 'ceremony', '--manifest-path', `${SRC}/client/Cargo.toml`],
    { env: { ...BUILD_ENV, CARGO_TARGET_DIR: `${WORK}/target-host` }, stdio: 'inherit' });
  state.vars.adapter = `${WORK}/target-host/release/examples/ceremony`;
  const ids = adapter({ method: 'ids' });
  if (ids.program !== PROGRAM) throw new Error(`the adapter speaks to ${ids.program}, not ${PROGRAM}`);
  // the patched constants against the client's own derivation, from the same patched copy
  if (ids.fund !== fund.address || ids.fundBump !== bump) throw new Error(`FUND_ADDRESS/FUND_BUMP ${fund.address}/${bump}, the client derives ${ids.fund}/${ids.fundBump}`);
  state.vars.ids = ids;
  say(`  patched: the id, FUND_ADDRESS ${fund.address}, FUND_BUMP ${bump}; adapter built for ${ids.program}`);
});
const IDS = state.vars.ids;

async function buildSo(name, oneUnit) {
  return once(`build: ${name}`, async () => {
    execFileSync(`${BIN}/cargo-build-sbf`, ['--arch', 'v3', '--manifest-path', `${SRC}/program/poleis_treasury/Cargo.toml`, '--sbf-out-dir', `${WORK}/build/${name}`],
      { env: { ...BUILD_ENV, CARGO_TARGET_DIR: `${WORK}/target-sbf-${name}`, ...(oneUnit ? { CARGO_PROFILE_RELEASE_CODEGEN_UNITS: '1' } : {}) }, stdio: 'inherit' });
    const so = readFileSync(`${WORK}/build/${name}/poleis_treasury.so`);
    say(`  ${name}: ${so.length} bytes, sha256 ${sha256(so)}`);
    return { path: `${WORK}/build/${name}/poleis_treasury.so`, bytes: so.length, sha256: sha256(so) };
  });
}
const V1 = await buildSo('v1', false);
const V2 = await buildSo('v2', true);
if (V1.sha256 === V2.sha256) throw new Error('the two builds are the same binary: the upgrade would prove nothing');
// deployed at its own length, as the Bronze handoff does; a larger upgrade first extends the program (anyone may pay for that)
const MAX_LEN = V1.bytes;

// ---- funding ----
await once('fund the fresh keys', async () => {
  const need = (await rent(45 + Math.max(V1.bytes, V2.bytes) + 10240)) + (await rent(37 + V2.bytes)) + SOL / 2n;
  const plan = [['deployer', need], ['bidder', SOL / 5n], ['chairA', SOL / 10n]];
  for (const [name, lamports] of plan) {
    const sol = (Number(lamports) / 1e9).toFixed(9);
    if (BANK) cli(['transfer', '--keypair', BANK, '--fee-payer', BANK, '--allow-unfunded-recipient', pub(name), sol]);
    else cli(['airdrop', sol, pub(name)]);
    say(`  funded ${name.padEnd(9)} ${sol} SOL ${BANK ? 'from the bank' : 'from the faucet'}`);
  }
  return plan.map(([n, l]) => ({ key: n, lamports: String(l) }));
});

// ---- 3. deploy, then tree 0 in the same breath ----
const DEPLOY = await once('deploy v1 under the fresh id', async () => {
  // a named buffer: a deploy that a public RPC cuts short resumes on the next run instead of stranding its rent
  const out = cliWrites(['program', 'deploy', V1.path, '--program-id', keyPath('program'), '--keypair', keyPath('deployer'), '--fee-payer', keyPath('deployer'),
    '--upgrade-authority', keyPath('deployer'), '--buffer', keyPath('deploy-buffer'), '--max-len', String(MAX_LEN)], { json: true });
  const shown = cli(['program', 'show', PROGRAM], { json: true });
  say(`  deployed ${out.programId}: authority ${shown.authority}, last deployed in slot ${shown.lastDeploySlot}`);
  return { signature: out.signature ?? null, slot: shown.lastDeploySlot, authority: shown.authority };
});

// a program is callable from the slot after its deploy: the founding goes in that next slot
await waitSlot(BigInt(DEPLOY.slot));
const founders = CHAIRS.flatMap((n) => Array(4).fill(pub(n)));
const GENESIS = await once('genesis parameters', async () => {
  const now = await slot();
  return {
    charter: sha256('THE CEREMONY, REHEARSED: the Dyson Society, tree 0'),
    planCap: String(SOL), planPeriod: '6480000', window: String(WINDOW), rung: 1,
    reserve: String(RESERVE), deposit: String(DEPOSIT),
    commitEnd: String(now + SLOTS + 20n), revealEnd: String(now + 2n * SLOTS + 20n),
    founders, name: 'The Dyson Society (rehearsal)', desc: '', defaultMint: '11111111111111111111111111111111',
  };
});
const FOUND = await send('found tree 0: init_config + open_genesis, one tx', { method: 'found', payer: DEPLOYER, ...GENESIS }, ['deployer']);
if (FOUND.slot !== null) say(`  the same breath: deployed in slot ${DEPLOY.slot}, tree 0 founded in slot ${FOUND.slot} (${FOUND.slot - DEPLOY.slot} slots)`);

// ---- 4. the genesis sale, then the council ----
const SALT = await once('the bid salt', async () => hex(randomBytes(32)));
const bid = { bidder: pub('bidder'), nonce: '1', price: String(PRICE), salt: SALT };
await send('commit one sealed seat bid', { method: 'commitSeat', ...bid }, ['bidder']);
await once('wait out the commit window', async () => { await waitSlot(BigInt(GENESIS.commitEnd)); });
await send('reveal it', { method: 'revealSeat', ...bid }, ['bidder']);
await once('wait out the reveal window', async () => { await waitSlot(BigInt(GENESIS.revealEnd)); });
await send('settle the sale', { method: 'settleSeats', crank: DEPLOYER }, ['deployer']);
await send('claim the seat', { method: 'claimSeat', bidder: bid.bidder, nonce: bid.nonce }, ['bidder']);
await send('ratify: the council takes tree 0', { method: 'ratify', crank: DEPLOYER }, ['deployer']);

// ---- 5. the code's authority to the program's own upgrade address ----
await once('hand the code to the upgrade address', async () => {
  cli(['program', 'set-upgrade-authority', PROGRAM, '--new-upgrade-authority', IDS.upgrade, '--skip-new-upgrade-authority-signer-check',
    '--upgrade-authority', keyPath('deployer'), '--keypair', keyPath('deployer')]);
  const shown = cli(['program', 'show', PROGRAM], { json: true });
  if (shown.authority !== IDS.upgrade) throw new Error(`the authority is ${shown.authority}, not ${IDS.upgrade}`);
  say(`  authority ${shown.authority}: the program's own upgrade address; no key holds the code`);
  return { authority: shown.authority };
});

// ---- 6. an upgrade by the seal ----
const BUFFER = pub('buffer');
await once('extend the program if v2 is larger: anyone pays, the seal still holds the code', async () => {
  const shown = cli(['program', 'show', PROGRAM], { json: true });
  if (V2.bytes <= shown.dataLen) { say(`  no extension: v2 (${V2.bytes} B) fits the ${shown.dataLen} B the program holds`); return { extended: 0 }; }
  const more = Math.max(10240, V2.bytes - shown.dataLen); // the loader's smallest extension is 10,240 bytes
  cli(['program', 'extend', PROGRAM, String(more), '--payer', keyPath('deployer'), '--keypair', keyPath('deployer')]);
  const after = cli(['program', 'show', PROGRAM], { json: true });
  if (after.authority !== IDS.upgrade) throw new Error(`extending moved the authority to ${after.authority}`);
  say(`  extended by ${more} B to ${after.dataLen} B, paid by the deployer; the authority is still the upgrade address`);
  return { extended: more, dataLen: after.dataLen };
});
await once('write v2 to a buffer the upgrade address owns', async () => {
  cliWrites(['program', 'write-buffer', V2.path, '--buffer', keyPath('buffer'), '--keypair', keyPath('deployer')]);
  cli(['program', 'set-buffer-authority', BUFFER, '--new-buffer-authority', IDS.upgrade, '--buffer-authority', keyPath('deployer'), '--keypair', keyPath('deployer')]);
  const shown = cli(['program', 'show', BUFFER], { json: true });
  if (shown.authority !== IDS.upgrade) throw new Error(`the buffer's authority is ${shown.authority}`);
  say(`  buffer ${BUFFER}: ${shown.dataLen} bytes, authority the upgrade address`);
});
const up = { buffer: BUFFER, spill: DEPLOYER };
const PROPOSAL = await send('propose: CODEX Upgrade, by a chair', { method: 'proposeUpgrade', proposer: pub('chairA'), manifest: sha256('upgrade by the seal'), nonce: '1', ...up }, ['chairA']);
const envelope = Buffer.from(PROPOSAL.envelope, 'hex');
const votes = ['chairA', 'chairB', 'bidder'].map((n) => ({ voter: pub(n), signature: hex(signWith(key(n), envelope)) }));
await send('attest: 8 of 12 chairs and the floor, PRO', { method: 'attest', crank: DEPLOYER, hash: PROPOSAL.hash, votes, computeUnits: 600000 }, ['deployer']);
await send('seal', { method: 'seal', crank: DEPLOYER, hash: PROPOSAL.hash }, ['deployer']);
const BEFORE = cli(['program', 'show', PROGRAM], { json: true }).lastDeploySlot;
await send('execute: the loader replaces the code', { method: 'executeUpgrade', crank: DEPLOYER, hash: PROPOSAL.hash, computeUnits: 600000, ...up }, ['deployer']);

// ---- 7. read it back, and the birth certificate ----
const VERIFY = await once('read the upgraded program back', async () => {
  const shown = cli(['program', 'show', PROGRAM], { json: true });
  cli(['program', 'dump', PROGRAM, `${WORK}/build/onchain.so`]);
  const onchain = readFileSync(`${WORK}/build/onchain.so`);
  const head = onchain.subarray(0, V2.bytes);
  const tailZero = onchain.subarray(V2.bytes).every((b) => b === 0);
  const v = { authority: shown.authority, lastDeploySlot: shown.lastDeploySlot, deployedBefore: DEPLOY.slot,
    onchainSha256OfV2Length: sha256(head), matchesV2: sha256(head) === V2.sha256 && tailZero, differsFromV1: V1.sha256 !== V2.sha256 };
  if (!v.matchesV2) throw new Error(`the code on chain is not v2: ${JSON.stringify(v)}`);
  if (v.authority !== IDS.upgrade) throw new Error(`the authority moved: ${v.authority}`);
  if (!(Number(v.lastDeploySlot) > Number(BEFORE ?? DEPLOY.slot))) throw new Error(`the code was not redeployed: ${v.lastDeploySlot}`);
  return v;
});
const certificate = {
  what: 'THE CEREMONY, REHEARSED: the Poleis program founded and upgraded by its own seal',
  cluster: RPC, genesisHash: await rpc('getGenesisHash'), at: new Date().toISOString(),
  program: PROGRAM, upgradeAddress: IDS.upgrade, programdata: IDS.programdata, tree0: { config: IDS.config, council: IDS.council, registry: IDS.registry, treasury: IDS.treasury },
  fund: state.vars.fund, source: { house: execFileSync('git', ['-C', HOUSE, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), patched: ['declare_id', 'FUND_ADDRESS', 'FUND_BUMP'] },
  binaries: { v1: { bytes: V1.bytes, sha256: V1.sha256 }, v2: { bytes: V2.bytes, sha256: V2.sha256, build: 'the same source, one codegen unit' }, maxLen: MAX_LEN,
    extension: state.done['extend the program if v2 is larger: anyone pays, the seal still holds the code'] },
  keys: { deployer: DEPLOYER, chairs: Object.fromEntries(CHAIRS.map((n) => [n, pub(n)])), floor: pub('bidder'), buffer: BUFFER },
  founders, genesis: GENESIS, deploy: DEPLOY, founding: FOUND, upgrade: { proposal: PROPOSAL.hash, votes: votes.map((v) => v.voter), ...VERIFY },
  transactions: state.txs,
};
writeFileSync(`${WORK}/birth-certificate.json`, JSON.stringify(certificate, null, 1));
say(`\n  THE CODE MOVED ONLY BY THE SEAL: ${PROGRAM} now runs v2 (sha256 ${V2.sha256.slice(0, 16)}…), last deployed in slot ${VERIFY.lastDeploySlot},`);
say(`  its authority the upgrade address ${VERIFY.authority}.\n  birth certificate: ${WORK}/birth-certificate.json`);
