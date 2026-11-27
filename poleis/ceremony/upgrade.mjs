#!/usr/bin/env node
// upgrade.mjs · THE HANDS WIDEN, the road (Oct 29 2026, lane/the-hands-widen): an upgrade of a ceremony's Poleis program by its own seal, on the cluster
// the ceremony founded it on, from the house's source as it stands. His words, UTC: 20:34 "you gotta go back to the design of our governance dude... the
// pentagon and hexagon voting"; 21:1x "Yes, widen them".
//   node poleis/ceremony/upgrade.mjs prepare --label <name> [--rpc URL] [--work DIR]
//       The house's source copied and patched to the ceremony's id (the declared id, FUND_ADDRESS and FUND_BUMP, as rehearse.mjs does), built for
//       sBPF v3; the program extended first if the build is larger (anyone pays, the seal still holds the code); the build written to a buffer whose
//       authority is the program's upgrade address. Nothing changes the code yet.
//   node poleis/ceremony/upgrade.mjs seal --label <name> --voters chairA,chairB,bidder [--rpc URL] [--work DIR]
//       A chair proposes CODEX Upgrade, the named keys say PRO (eight of twelve chairs and the floor), the seal, the execute; the code read back.
// The keys, the adapter and the ids are the ceremony's (WORK/keys, WORK/state.json), read, never made; each upgrade keeps its own state in
// WORK/upgrades/<label>.json and leaves its certificate beside this file. On devnet the chairs are stand-ins for his three Keystones (rehearse.mjs):
// whose keys seal is his call. Mainnet is refused: that is his genesis.
import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { b58decode, checkKeypair, freshKeypair, publicOf, signMessage, signWith } from './wire.mjs';

const [cmd] = process.argv.slice(2);
const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
const RPC = arg('rpc', 'https://api.devnet.solana.com');
const LABEL = arg('label', '');
if (!['prepare', 'seal'].includes(cmd) || !/^[a-z0-9-]{1,40}$/.test(LABEL)) {
  console.error('upgrade: prepare|seal --label <name> [--voters chairA,chairB,bidder] [--rpc URL] [--work DIR]');
  process.exit(2);
}
if (/mainnet/i.test(RPC)) { console.error('upgrade: not mainnet; that is his genesis'); process.exit(2); }
const LOCAL = /127\.0\.0\.1|localhost/.test(RPC);
const WORK = resolve(arg('work', `${homedir()}/.embertide/poleis-rehearsal/${LOCAL ? 'localnet' : new URL(RPC).hostname}`));
const HOUSE = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const HERE = dirname(fileURLToPath(import.meta.url));
const CEREMONY = JSON.parse(readFileSync(`${WORK}/state.json`, 'utf8'));
if (CEREMONY.rpc !== RPC) throw new Error(`${WORK} belongs to ${CEREMONY.rpc}, not ${RPC}`);
const IDS = CEREMONY.vars.ids, FUND = CEREMONY.vars.fund, PROGRAM = IDS.program;

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
// the 27.0 SDK's linker refuses the host build scripts; the 26.5 SDK links them (the house's standing workaround, rehearse.mjs)
const SDK = '/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk';
const BUILD_ENV = { ...process.env, ...(process.env.SDKROOT || !existsSync(SDK) ? {} : { SDKROOT: SDK }) };

const say = (s) => console.log(s);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const sha256 = (b) => createHash('sha256').update(b).digest('hex');
const hex = (b) => Buffer.from(b).toString('hex');

mkdirSync(`${WORK}/upgrades`, { recursive: true });
const STATE = `${WORK}/upgrades/${LABEL}.json`;
const state = existsSync(STATE) ? JSON.parse(readFileSync(STATE, 'utf8')) : { label: LABEL, rpc: RPC, program: PROGRAM, done: {}, txs: [] };
const save = () => writeFileSync(STATE, JSON.stringify(state, null, 1));
const keyPath = (name) => `${WORK}/keys/${name}.json`;
function key(name) {
  if (!existsSync(keyPath(name))) throw new Error(`no key ${name} in ${WORK}/keys: upgrade.mjs reads the ceremony's keys, never makes them`);
  return checkKeypair(Uint8Array.from(JSON.parse(readFileSync(keyPath(name), 'utf8'))));
}
const pub = (name) => publicOf(key(name));
async function once(step, fn) {
  if (state.done[step]) { say(`  ${step.padEnd(52)} (done before)`); return state.done[step]; }
  const v = (await fn()) ?? true;
  state.done[step] = v;
  save();
  return v;
}

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
const blockhash = async () => (await rpc('getLatestBlockhash', [{ commitment: 'confirmed' }])).value.blockhash;
function cli(args, { json = false } = {}) {
  const r = spawnSync(`${BIN}/solana`, [...args, '--url', RPC, '--commitment', 'confirmed', ...(json ? ['--output', 'json'] : [])], { encoding: 'utf8' });
  if (r.status !== 0) throw new Error(`solana ${args.slice(0, 2).join(' ')}: ${(r.stderr || r.stdout).trim()}`);
  return json ? JSON.parse(r.stdout) : r.stdout.trim();
}
const WRITE_FLAGS = LOCAL ? ['--use-rpc'] : ['--max-sign-attempts', '30'];
function cliWrites(args, opts) {
  for (let i = 1; ; i++) {
    try { return cli([...args, ...WRITE_FLAGS], opts); } catch (e) {
      if (i >= 3 || !/retries|timed out|blockhash|429|Too many requests/i.test(e.message)) throw e;
      say(`  the upload was cut short (${e.message.split('\n')[0].slice(0, 90)}); resuming from its buffer, try ${i + 1} of 3`);
    }
  }
}
function adapter(req) {
  const r = spawnSync(CEREMONY.vars.adapter, { input: JSON.stringify(req), encoding: 'utf8' });
  const out = JSON.parse(r.stdout || '{}');
  if (!out.ok) throw new Error(`adapter ${req.method}: ${out.error || r.stderr}`);
  return out;
}
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
    say(`  ${step.padEnd(52)} slot ${row.slot}  ${row.bytes} B  ${row.cu} CU  ${signature.slice(0, 12)}…`);
    return { ...row, ...(out.hash ? { hash: out.hash, envelope: out.envelope } : {}) };
  });
}

const DEPLOYER = pub('deployer');
const SRC = `${WORK}/src-${LABEL}`;
const BUFFER_NAME = `buffer-${LABEL}`;
say(`THE HANDS WIDEN: ${cmd} ${LABEL} on ${RPC}\n  program ${PROGRAM}, upgrade address ${IDS.upgrade}\n  work ${WORK}`);

if (cmd === 'prepare') {
  // the house's source as it stands, under the ceremony's id: the same three constants rehearse.mjs moves
  const BUILD = await once('build: the patched copy, for sBPF v3', async () => {
    mkdirSync(SRC, { recursive: true });
    for (const part of ['program', 'client']) {                       // the program's tests reach the client by path, as in rehearse.mjs
      execFileSync('rsync', ['-a', '--delete', '--exclude', 'target', `${HOUSE}/poleis/${part}/`, `${SRC}/${part}/`]);
    }
    const lib = `${SRC}/program/poleis_treasury/src/lib.rs`;
    let text = readFileSync(lib, 'utf8');
    const swap = (re, to, what) => {
      const n = (text.match(new RegExp(re.source, 'g')) || []).length;
      if (n !== 1) throw new Error(`${what}: ${n} matches in lib.rs, expected 1`);
      text = text.replace(re, to);
    };
    const fundBytes = [...b58decode(FUND.address)].map((b) => `0x${b.toString(16).padStart(2, '0')}`);
    swap(/declare_id!\("[1-9A-HJ-NP-Za-km-z]{32,44}"\)/, `declare_id!("${PROGRAM}")`, 'the declared id');
    swap(/pub const FUND_ADDRESS: Address = Address::new_from_array\(\[[^\]]*\]\);/,
      `pub const FUND_ADDRESS: Address = Address::new_from_array([\n    ${fundBytes.slice(0, 16).join(', ')},\n    ${fundBytes.slice(16).join(', ')},\n]);`, 'FUND_ADDRESS');
    swap(/pub const FUND_BUMP: u8 = \d+;/, `pub const FUND_BUMP: u8 = ${FUND.bump};`, 'FUND_BUMP');
    writeFileSync(lib, text);
    execFileSync(`${BIN}/cargo-build-sbf`, ['--arch', 'v3', '--manifest-path', `${SRC}/program/poleis_treasury/Cargo.toml`, '--sbf-out-dir', `${WORK}/build/${LABEL}`],
      { env: { ...BUILD_ENV, CARGO_TARGET_DIR: `${WORK}/target-sbf-${LABEL}` }, stdio: 'inherit' });
    const so = readFileSync(`${WORK}/build/${LABEL}/poleis_treasury.so`);
    say(`  ${LABEL}: ${so.length} bytes, sha256 ${sha256(so)}`);
    return { path: `${WORK}/build/${LABEL}/poleis_treasury.so`, bytes: so.length, sha256: sha256(so) };
  });
  await once('extend the program if the build is larger: anyone pays, the seal still holds the code', async () => {
    const shown = cli(['program', 'show', PROGRAM], { json: true });
    if (shown.authority !== IDS.upgrade) throw new Error(`the code's authority is ${shown.authority}, not the upgrade address`);
    if (BUILD.bytes <= shown.dataLen) { say(`  no extension: ${BUILD.bytes} B fits the ${shown.dataLen} B the program holds`); return { extended: 0, dataLen: shown.dataLen }; }
    const more = Math.max(BUILD.bytes - shown.dataLen, 10_240);       // the loader refuses an extension under 10,240 bytes (Agave 4.2, Oct 29)
    cli(['program', 'extend', PROGRAM, String(more), '--payer', keyPath('deployer'), '--keypair', keyPath('deployer')]);
    const after = cli(['program', 'show', PROGRAM], { json: true });
    if (after.authority !== IDS.upgrade) throw new Error(`extending moved the authority to ${after.authority}`);
    say(`  extended by ${more} B to ${after.dataLen} B, paid by the deployer; the authority is still the upgrade address`);
    return { extended: more, dataLen: after.dataLen };
  });
  await once('write the build to a buffer the upgrade address owns', async () => {
    if (!existsSync(keyPath(BUFFER_NAME))) writeFileSync(keyPath(BUFFER_NAME), JSON.stringify([...freshKeypair()]), { mode: 0o600 });
    const buffer = pub(BUFFER_NAME);
    cliWrites(['program', 'write-buffer', BUILD.path, '--buffer', keyPath(BUFFER_NAME), '--keypair', keyPath('deployer')]);
    cli(['program', 'set-buffer-authority', buffer, '--new-buffer-authority', IDS.upgrade, '--buffer-authority', keyPath('deployer'), '--keypair', keyPath('deployer')]);
    const shown = cli(['program', 'show', buffer], { json: true });
    if (shown.authority !== IDS.upgrade) throw new Error(`the buffer's authority is ${shown.authority}`);
    say(`  buffer ${buffer}: ${shown.dataLen} bytes, authority the upgrade address`);
    return { buffer, dataLen: shown.dataLen };
  });
  say(`  ready: "node poleis/ceremony/upgrade.mjs seal --label ${LABEL} --voters <keys>" puts it to the seal`);
} else {
  const BUILD = state.done['build: the patched copy, for sBPF v3'], BUF = state.done['write the build to a buffer the upgrade address owns'];
  if (!BUILD || !BUF) throw new Error(`run prepare --label ${LABEL} first`);
  const voters = (arg('voters', '') || '').split(',').filter(Boolean);
  if (!voters.length) throw new Error('--voters <keys> is needed: whose keys seal is his call');
  const up = { buffer: BUF.buffer, spill: DEPLOYER };
  const nonce = String(parseInt(sha256(`the hands widen: ${LABEL}`).slice(0, 8), 16));
  const proposal = await send('propose: CODEX Upgrade, by a chair', { method: 'proposeUpgrade', proposer: pub(voters[0]), manifest: sha256(`the hands widen: ${LABEL}`), nonce, ...up }, [voters[0]]);
  const envelope = Buffer.from(proposal.envelope, 'hex');
  const votes = voters.map((n) => ({ voter: pub(n), signature: hex(signWith(key(n), envelope)) }));
  await send('attest: the named keys, PRO', { method: 'attest', crank: DEPLOYER, hash: proposal.hash, votes, computeUnits: 600000 }, ['deployer']);
  await send('seal', { method: 'seal', crank: DEPLOYER, hash: proposal.hash }, ['deployer']);
  await send('execute: the loader replaces the code', { method: 'executeUpgrade', crank: DEPLOYER, hash: proposal.hash, computeUnits: 600000, ...up }, ['deployer']);
  const read = await once('read back: the code on chain is the build, byte for byte', async () => {
    const shown = cli(['program', 'show', PROGRAM], { json: true });
    const dump = `${WORK}/build/${LABEL}/onchain.so`;
    cli(['program', 'dump', PROGRAM, dump]);
    const onchain = readFileSync(dump).subarray(0, BUILD.bytes);
    if (sha256(onchain) !== BUILD.sha256) throw new Error(`the code on chain is not the build: ${sha256(onchain)} against ${BUILD.sha256}`);
    if (shown.authority !== IDS.upgrade) throw new Error(`the authority moved to ${shown.authority}`);
    return { authority: shown.authority, lastDeploySlot: shown.lastDeploySlot, dataLen: shown.dataLen, sha256: BUILD.sha256 };
  });
  const cert = { what: `THE HANDS WIDEN: the Poleis program upgraded by its own seal (${LABEL})`, at: new Date().toISOString(), cluster: RPC, program: PROGRAM,
    upgradeAddress: IDS.upgrade, build: BUILD, buffer: BUF, voters: voters.map(pub), proposal: proposal.hash, read, transactions: state.txs };
  writeFileSync(`${HERE}/upgrade-certificate-${new URL(RPC).hostname.split('.')[1] || 'local'}-${LABEL}.json`, JSON.stringify(cert, null, 1) + '\n');
  say(`  upgraded: ${PROGRAM} runs ${BUILD.sha256.slice(0, 12)}…, the authority still the upgrade address`);
}
