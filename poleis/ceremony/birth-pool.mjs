#!/usr/bin/env node
// birth-pool.mjs · THE POOL IS BORN BY THE SEAL (Oct 29 2026, lane/the-hands-widen): an SPL stake pool born by unit 0's own hands, on the tree 0 the
// ceremony founded. His words, UTC: 20:34 "you gotta go back to the design of our governance dude... the pentagon and hexagon voting"; 21:11 "Yes,
// widen them"; 21:23 "Stand-ins now"; 21:3x "Yes, birth it".
//   1. The pool's empty accounts, paid by the crank, since anyone may: the pool and its validator list (the stake pool program's), the reserve stake
//      (both authorities the pool's withdraw authority) and the mint (9 decimals, minted only by that authority), each with a seed from the crank.
//   2. Unit 0's vault for the pool's mint (create_vault, anyone may): the pool's fee account. The whole yield lands in tree 0's treasury, and only a
//      sealed TREASURY act moves it.
//   3. The birth, the one step that needs a signature: the pool's Initialize as a raw act of unit 0. The agent signs as the manager and is the staker;
//      the epoch fee is 1/1. A chair proposes, the named keys say PRO, the seal, the execute.
//   4. Read back from the chain: manager and staker unit 0's agent, the fee account unit 0's vault, the epoch fee 1/1.
// No key holds the pool: what it does next (its validators, its fees) is a sealed act of unit 0.
//
// Usage: node poleis/ceremony/birth-pool.mjs --voters chairA,chairB,bidder [--rpc URL] [--work DIR] [--tag u0a] [--label the-hands-widen]
// Resumable from WORK/birth-pool-<tag>.json. Mainnet is refused.
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cluster } from './cluster.mjs';
import { b58decode, b58encode, signWith } from './wire.mjs';

const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
const RPC = arg('rpc', 'https://api.devnet.solana.com');
if (/mainnet/i.test(RPC)) { console.error('birth-pool: not mainnet; that is his genesis'); process.exit(2); }
const LOCAL = /127\.0\.0\.1|localhost/.test(RPC);
const WORK = resolve(arg('work', `${homedir()}/.embertide/poleis-rehearsal/${LOCAL ? 'localnet' : new URL(RPC).hostname}`));
const TAG = arg('tag', 'u0a');
const LABEL = arg('label', 'the-hands-widen');          // the patched copy upgrade.mjs made and deployed
const VOTERS = (arg('voters', '') || '').split(',').filter(Boolean);
if (!/^[a-z0-9]{1,8}$/.test(TAG) || !VOTERS.length) { console.error('birth-pool: --voters <keys> [--tag <a-z0-9, 8 at most>]'); process.exit(2); }
const HOUSE = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const HERE = dirname(fileURLToPath(import.meta.url));
const SDK = '/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk';

const POOL_PROGRAM = LOCAL ? 'SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy' : 'DPoo15wWDqpPJJtS2MUZ49aRxqz5ZaaJCJP4z8bLuib';
const SYSTEM = '11111111111111111111111111111111', STAKE = 'Stake11111111111111111111111111111111111111';
const TOKEN = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', RENT = 'SysvarRent111111111111111111111111111111111';
const ACT_WRITABLE = 1, ACT_SIGNER = 2, MAX_VALIDATORS = 10;
const SIZES = { pool: 611, list: 9 + 73 * MAX_VALIDATORS, reserve: 200, mint: 82 };

const sha256 = (b) => createHash('sha256').update(b).digest();
const hex = (b) => Buffer.from(b).toString('hex');
const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
const u64 = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const withSeed = (base, seed, owner) => b58encode(sha256(Buffer.concat([b58decode(base), Buffer.from(seed), b58decode(owner)])));

const R = JSON.parse(readFileSync(`${WORK}/state.json`, 'utf8'));   // the rehearsal's own record: the ids of tree 0
const IDS = R.vars.ids;
const ADAPTER = `${WORK}/target-host-${LABEL}/release/examples/ceremony`;
const C = cluster({ rpcUrl: RPC, work: WORK, stateFile: `birth-pool-${TAG}.json`, adapterPath: ADAPTER });
const { once, pub, adapter, send, rpc, account } = C;
const CRANK = pub('deployer');
console.log(`THE POOL IS BORN BY THE SEAL on ${RPC}\n  Poleis ${IDS.program}, tree 0's treasury ${IDS.treasury}\n  the stake pool program ${POOL_PROGRAM}`);

// ---- the adapter, built from the deployed patched copy with the raw act's and the vault's methods ----
await once('adapter: the raw act and the vault', async () => {
  const src = `${WORK}/src-${LABEL}`;
  if (!existsSync(`${src}/program`)) throw new Error(`no patched copy at ${src}: run upgrade.mjs prepare --label ${LABEL} first`);
  execFileSync('rsync', ['-a', '--delete', '--exclude', 'target', `${HOUSE}/poleis/client/`, `${src}/client/`]);
  execFileSync('cargo', ['build', '--release', '--example', 'ceremony', '--manifest-path', `${src}/client/Cargo.toml`],
    { env: { ...process.env, ...(process.env.SDKROOT || !existsSync(SDK) ? {} : { SDKROOT: SDK }), CARGO_TARGET_DIR: `${WORK}/target-host-${LABEL}` }, stdio: 'inherit' });
  const ids = adapter({ method: 'ids' });
  if (ids.program !== IDS.program) throw new Error(`the adapter speaks to ${ids.program}, not ${IDS.program}`);
  return true;
});

const pda = (program, seeds) => adapter({ method: 'pda', program, seeds: seeds.map((s) => hex(s)) }).address;
const seeds = { pool: `${TAG}p`, list: `${TAG}l`, reserve: `${TAG}r`, mint: `${TAG}m` };
const A = {
  pool: withSeed(CRANK, seeds.pool, POOL_PROGRAM), list: withSeed(CRANK, seeds.list, POOL_PROGRAM),
  reserve: withSeed(CRANK, seeds.reserve, STAKE), mint: withSeed(CRANK, seeds.mint, TOKEN),
};
A.withdraw = pda(POOL_PROGRAM, [b58decode(A.pool), Buffer.from('withdraw')]);
A.agent = pda(IDS.program, [Buffer.from('agent'), u64(0)]);
A.vault = pda(IDS.program, [Buffer.from('vault'), u64(0), b58decode(A.mint)]);
console.log(`  the pool ${A.pool}\n  unit 0's agent ${A.agent}, its vault for the pool's mint ${A.vault}`);

// ---- 1. the empty accounts, paid by the crank ----
const meta = (pubkey, isSigner, isWritable) => ({ pubkey, isSigner, isWritable });
const rent = async (n) => BigInt(await rpc('getMinimumBalanceForRentExemption', [n]));
const createIx = async (name, owner) => ({ programId: SYSTEM, accounts: [meta(CRANK, true, true), meta(A[name], false, true)],
  data: hex(Buffer.concat([u32(3), b58decode(CRANK), u64(seeds[name].length), Buffer.from(seeds[name]), u64(await rent(SIZES[name])), u64(SIZES[name]), b58decode(owner)])) });
await send('the pool\'s empty accounts, paid by the crank', { method: 'compile', payer: CRANK, instructions: [
  await createIx('pool', POOL_PROGRAM), await createIx('list', POOL_PROGRAM), await createIx('reserve', STAKE),
  { programId: STAKE, accounts: [meta(A.reserve, false, true), meta(RENT, false, false)],
    data: hex(Buffer.concat([u32(0), b58decode(A.withdraw), b58decode(A.withdraw), Buffer.alloc(48)])) },
  await createIx('mint', TOKEN),
  { programId: TOKEN, accounts: [meta(A.mint, false, true)], data: hex(Buffer.concat([Buffer.of(20, 9), b58decode(A.withdraw), Buffer.of(0)])) },
] }, ['deployer']);

// ---- 2. unit 0's vault for the pool's mint: the fee account ----
await send('unit 0\'s vault for the pool\'s mint, the fee account', { method: 'createVault', payer: CRANK, mint: A.mint, tokenProgram: TOKEN }, ['deployer']);

// ---- 3. the birth: Initialize as unit 0's raw act ----
// StakePoolInstruction::Initialize: Fee { denominator, numerator } for the epoch (1/1), the withdrawal and the deposit (none), the referral (0), the list
const fee = (n, d) => Buffer.concat([u64(d), u64(n)]);
const act = {
  program: POOL_PROGRAM,
  accounts: [
    { key: A.pool, flags: ACT_WRITABLE }, { key: A.agent, flags: ACT_SIGNER }, { key: A.agent, flags: 0 }, { key: A.withdraw, flags: 0 },
    { key: A.list, flags: ACT_WRITABLE }, { key: A.reserve, flags: 0 }, { key: A.mint, flags: ACT_WRITABLE }, { key: A.vault, flags: ACT_WRITABLE },
    { key: TOKEN, flags: 0 },
  ],
  data: hex(Buffer.concat([Buffer.of(0), fee(1, 1), fee(0, 0), fee(0, 0), Buffer.of(0), u32(MAX_VALIDATORS)])),
};
const manifest = hex(sha256(`unit 0 gives birth to its stake pool: ${TAG}`));
const nonce = String(parseInt(hex(sha256(`birth-pool ${TAG}`)).slice(0, 8), 16));
const P = await send('propose: ACT Raw, the pool\'s Initialize, by a chair', { method: 'proposeRaw', proposer: pub(VOTERS[0]), manifest, nonce, ...act }, [VOTERS[0]]);
const envelope = Buffer.from(P.envelope, 'hex');
const votes = VOTERS.map((n) => ({ voter: pub(n), signature: hex(signWith(C.key(n), envelope)) }));
await send('attest: the named keys, PRO', { method: 'attest', crank: CRANK, hash: P.hash, votes, computeUnits: 600000 }, ['deployer']);
await send('seal', { method: 'seal', crank: CRANK, hash: P.hash }, ['deployer']);
await send('execute: unit 0\'s hands initialize the pool', { method: 'executeRaw', crank: CRANK, hash: P.hash, computeUnits: 600000, ...act }, ['deployer']);

// ---- 4. read back ----
const read = await once('read back: the pool is unit 0\'s', async () => {
  const d = (await account(A.pool)).data;
  const k = (o) => b58encode(d.subarray(o, o + 32));
  const got = { accountType: d[0], manager: k(1), staker: k(33), mint: k(162), feeAccount: k(194),
    epochFee: `${d.readBigUInt64LE(338)}/${d.readBigUInt64LE(330)}` };
  const want = { accountType: 1, manager: A.agent, staker: A.agent, mint: A.mint, feeAccount: A.vault, epochFee: '1/1' };
  for (const [f, v] of Object.entries(want)) if (got[f] !== v) throw new Error(`the pool's ${f} is ${got[f]}, not ${v}`);
  return got;
});
const cert = { what: 'THE POOL IS BORN BY THE SEAL: an SPL stake pool initialized by unit 0\'s raw act, no key its manager', at: new Date().toISOString(),
  cluster: RPC, poleis: IDS.program, stakePoolProgram: POOL_PROGRAM, addresses: A, seeds, maxValidators: MAX_VALIDATORS, voters: VOTERS.map(pub),
  proposal: P.hash, read, transactions: C.state.txs };
writeFileSync(`${HERE}/pool-birth-certificate-${LOCAL ? 'localnet' : new URL(RPC).hostname.split('.')[1]}-${TAG}.json`, JSON.stringify(cert, null, 1) + '\n');
console.log(`  born: the pool ${A.pool}, its manager and staker unit 0's agent, its fee account unit 0's vault, the epoch fee 1/1`);
