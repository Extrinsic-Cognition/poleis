#!/usr/bin/env node
// rehearse.mjs · THE $WARM LAUNCH, rehearsed end to end on devnet (Oct 30 2026). DEVNET ONLY: every send asks the cluster's genesis hash first
// (rpc.mjs) and refuses anything that is not devnet's. The keys are throwaway devnet keys in a folder outside the house (--keys, the seat's scratchpad):
// the launch key stands in for his Keystone, three members and a create key for his Squads, one key for the dev wallet, one for a test buyer. The
// public faucet is closed on devnet, so test SOL comes from a funded devnet key (--bank), as the ceremony's rehearsals take it.
// The flow, in the order mainnet runs it, the 15 days compressed into minutes:
//   the treasury (a Squads 2-of-3 vault) · the mint and the 90/10 split, the mint authority revoked · the pool at the starting price · the 16 tranches,
//   each laid just above the price of its moment, a test buyer between them · the money after tranches 4, 8, 12 and 15 · the receipts.
// Usage: node tools/warm-launch/rehearse.mjs --keys DIR --bank KEYPAIR [--params FILE] [--rpc URL] [--tag NAME] [--out FILE]
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { base58Of, findPda } from '../solana-pda.mjs';
import { freshKeypair } from '../../poleis/ceremony/wire.mjs';
import * as SQ from '../../poleis/ceremony/squads.mjs';
import * as C from './chain.mjs';
import * as L from './launch.mjs';
import * as PL from './plan.mjs';
import { cluster, signerOf } from './rpc.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
const KEYS = arg('keys', null);
const BANK = arg('bank', null);
const RPC = arg('rpc', 'https://api.devnet.solana.com');
const TAG = arg('tag', 'r1');
const OUT = resolve(arg('out', `${HERE}/receipts/devnet-rehearsal-2026-10-30.json`));
const p = L.readParams(JSON.parse(readFileSync(resolve(arg('params', `${HERE}/params.devnet.json`)), 'utf8')));
if (!KEYS || !BANK) throw new Error('--keys (a folder outside the house) and --bank (a funded devnet keypair) are needed');
if (resolve(KEYS).startsWith(resolve(HERE, '..', '..'))) throw new Error('--keys must lie outside the house: no key file goes in git');

const SOL = 1_000_000_000n;
const sol = (l) => (Number(l) / 1e9).toFixed(6);
const tok = (b) => (Number(b) / 10 ** p.decimals).toLocaleString('en-US', { maximumFractionDigits: 3 });
const tx = (s) => `https://explorer.solana.com/tx/${s}?cluster=devnet`;
const at = (k) => `https://explorer.solana.com/address/${k}?cluster=devnet`;
const now = () => new Date().toISOString();

function keyAt(name) {
  const f = `${KEYS}/${name}.json`;
  if (!existsSync(f)) writeFileSync(f, JSON.stringify([...freshKeypair()]), { mode: 0o600 });
  return signerOf(JSON.parse(readFileSync(f, 'utf8')));
}
const k = { launch: keyAt('his-standin'), buyer: keyAt('buyer'), dev: keyAt('dev-standin'), m1: keyAt('squads-m1'), m2: keyAt('squads-m2'),
  m3: keyAt('squads-m3'), create: keyAt(`squads-create-${TAG}`) };
const bank = signerOf(JSON.parse(readFileSync(BANK, 'utf8')));
const c = cluster(RPC, { log: console.log });
await c.devnetOnly();

const STATE = `${KEYS}/rehearsal-${TAG}.json`;
const R = existsSync(STATE) ? JSON.parse(readFileSync(STATE, 'utf8'), (key, v) => (typeof v === 'string' && /^\d+n$/.test(v) ? BigInt(v.slice(0, -1)) : v))
  : { tag: TAG, rpc: RPC, started: now(), steps: {}, receipts: [], buys: [], tranches: [], money: [], withdrawn: 0n, carry: 0n };
const save = () => writeFileSync(STATE, JSON.stringify(R, (key, v) => (typeof v === 'bigint' ? `${v}n` : v), 1));
async function once(name, fn) {
  if (R.steps[name]) return R.steps[name];
  const out = await fn();
  R.steps[name] = out ?? true; save();
  return R.steps[name];
}
const receipt = (step, res, extra = {}) => { R.receipts.push({ step, at: now(), signature: res.signature, units: res.units, bytes: res.size, link: tx(res.signature), ...extra }); save(); };
const send = async (step, built, signers, extra) => { const res = await c.send(built.ixs, k.launch.address, signers, { label: step }); receipt(step, res, extra); return res; };

console.log(`THE $WARM LAUNCH, rehearsed on devnet (${RPC}), tag ${TAG}\n  launch key (his Keystone's stand-in) ${k.launch.address}\n  test buyer ${k.buyer.address}`);

// ---- test SOL from the bank: devnet only (the send asks the genesis hash) ----
await once('fund', async () => {
  for (const [who, s, want] of [['the launch key', k.launch, 3n * SOL], ['the test buyer', k.buyer, 3n * SOL]]) {
    const have = await c.balance(s.address);
    if (have >= want / 2n) { console.log(`  ${who} holds ${sol(have)} SOL`); continue; }
    const res = await c.send([C.transfer(bank.address, s.address, want - have)], bank.address, [bank], { label: `fund ${who}` });
    receipt(`fund ${who}`, res, { lamports: (want - have).toString() });
  }
});

// ---- the treasury: a Squads v4 2-of-3 vault, as his on mainnet (three stand-in members here) ----
const squads = await once('treasury', async () => {
  const pdaOf = (seeds) => base58Of(findPda(seeds, SQ.SQUADS)[0]);
  const programConfig = pdaOf([Buffer.from('multisig'), Buffer.from('program_config')]);
  const config = await c.account(programConfig);
  const multisig = pdaOf([Buffer.from('multisig'), Buffer.from('multisig'), C.key(k.create.address)]);
  const vault = pdaOf([Buffer.from('multisig'), C.key(multisig), Buffer.from('vault'), Buffer.of(0)]);
  if (!(await c.account(multisig))) {
    const sq = SQ.multisigCreateV2({ programConfig, treasury: base58Of(config.data.subarray(48, 80)), multisig, createKey: k.create.address,
      creator: k.launch.address, members: [k.m1.address, k.m2.address, k.m3.address], threshold: 2 });
    const ix = { program: sq.programId, keys: sq.accounts.map((m) => ({ key: m.pubkey, signer: m.isSigner, writable: m.isWritable })), data: Buffer.from(sq.data, 'hex') };
    const res = await c.send([ix], k.launch.address, [k.launch, k.create], { label: 'treasury: Squads 2-of-3' });
    receipt('treasury: a Squads v4 2-of-3 multisig (stand-in members)', res, { multisig, vault });
  }
  return { multisig, vault };
});

const a = PL.addresses({ launchKey: k.launch.address, seed: `swarm-${TAG}`, treasuryOwner: squads.vault, devWallet: k.dev.address });
console.log(`  mint ${a.mint}\n  pair ${a.pair.lbPair}\n  treasury vault ${a.treasuryOwner}\n  dev wallet ${a.devWallet}`);

// ---- 1. the mint and the split ----
await once('genesis', async () => {
  const g = PL.genesis({ p, a, mintRent: await c.rent(C.SIZES.mint) });
  await send('genesis', g, [k.launch], { supply: g.amounts.supply.toString(), treasury: g.amounts.treasury.toString(), sale: g.amounts.sale.toString() });
});

// ---- 2. the pool ----
await once('pool', async () => {
  const pl = PL.pool({ p, a, activationPoint: null });
  await send('pool', pl, [k.launch], { activeId: pl.activeId, startPrice: p.startPrice });
});

// ---- the chain's word on the pair and the position ----
async function pairState() { return C.readPair((await c.account(a.pair.lbPair)).data); }
async function positionsState(indexes) {
  const keys = indexes.flatMap((i) => [C.positionAddress(a.pair.lbPair, a.launchKey, i * C.BINS_PER_ARRAY), C.binArrayAddress(a.pair.lbPair, i)]);
  const accts = await c.accounts(keys);
  const out = [];
  indexes.forEach((index, j) => {
    const pos = accts[2 * j], arr = accts[2 * j + 1];
    if (pos && arr) { const ps = C.readPosition(pos.data); out.push({ index, shares: ps.shares, fees: ps.fees, bins: C.readBinArray(arr.data).bins }); }
  });
  return out;
}
const owned = () => [...new Set(R.tranches.flatMap((t) => t.indexes))].sort((x, y) => x - y);
async function snapshot() {
  const pair = await pairState();
  const held = L.positionSol(await positionsState(owned()), pair.activeId);
  return { activeId: pair.activeId, price: L.uiPriceOfBin(pair.activeId, p.binStep, p.decimals), solInPosition: held.total, swarmInPosition: held.swarm,
    reserveSol: await c.tokenAmount(a.pair.reserveY), reserveSwarm: await c.tokenAmount(a.pair.reserveX) };
}
const show = (s) => `bin ${s.activeId}, ${s.price.toExponential(4)} SOL per $WARM; the position holds ${sol(s.solInPosition)} SOL and ${tok(s.swarmInPosition)} $WARM`;

// ---- 3. and 4. the tranches, a test buyer between them, the money on its schedule ----
const width = L.windowBins(p.windowRatio, p.binStep);
const sched = L.schedule({ opening: p.opening, close: p.close, trancheHours: p.trancheHours, sale: L.split(p.supplyBase).sale });
const BUYS = { 1: 150n, 2: 100n, 3: 200n, 5: 150n, 7: 300n, 9: 200n, 11: 250n, 13: 300n, 15: 200n };   // millisol before each tranche
console.log(`  ${sched.length} tranches of ${tok(sched[0].amount)} $WARM, each over ${width} bins (${p.windowRatio}x) just above the price of its moment`);

async function buyNow(kk, millisol) {
  const before = await snapshot();
  const lamports = millisol * 1_000_000n;
  const first = C.binArrayIndex(before.activeId);
  const arrays = (await c.accounts([0, 1, 2, 3].map((d) => C.binArrayAddress(a.pair.lbPair, first + d)))).map((acc, d) => (acc ? C.binArrayAddress(a.pair.lbPair, first + d) : null)).filter(Boolean);
  const swarmBefore = await c.tokenAmount(C.ata(k.buyer.address, a.mint));
  const b = PL.buy({ a, buyer: k.buyer.address, lamports, arrays });
  const res = await c.send(b.ixs, k.buyer.address, [k.buyer], { label: `buy before tranche ${kk}` });
  const after = await snapshot();
  const got = (await c.tokenAmount(C.ata(k.buyer.address, a.mint))) - swarmBefore;
  const row = { beforeTranche: kk, lamports, swarmOut: got, avgPrice: Number(lamports) / Number(got) / 10 ** (9 - p.decimals), before, after, signature: res.signature };
  R.buys.push(row); receipt(`buy before tranche ${kk}`, res, { lamports: lamports.toString(), swarmOut: got.toString() });
  console.log(`    the buyer paid ${sol(lamports)} SOL for ${tok(got)} $WARM: ${show(before)} -> ${show(after)}`);
}

async function moneyNow(label) {
  const pair = await pairState();
  const positions = await positionsState(owned());
  const m = PL.money({ a, active: pair.activeId, positions, withdrawn: R.withdrawn, devShareBps: p.devShareBps });
  if (!m.ixs.length) { console.log(`    money ${label}: nothing owed (${sol(m.due.due)} SOL)`); return; }
  const devBefore = await c.tokenAmount(a.devWsol);
  const res = await send(`money ${label}`, m, [k.launch], { owed: m.due.due.toString(), fees: m.due.fees.toString(), bps: m.due.bps });
  const got = (await c.tokenAmount(a.devWsol)) - devBefore;
  R.withdrawn += got;
  R.money.push({ label, takenIn: m.due.takenIn, owed: m.due.owed, due: m.due.due, feesClaimed: m.due.fees, fromBins: m.due.fromBins, bps: m.due.bps, paid: got,
    withdrawnTotal: R.withdrawn, signature: res.signature });
  save();
  console.log(`    money ${label}: the position had taken in ${sol(m.due.takenIn)} SOL; its earned fees (${sol(m.due.fees)}) and ${m.due.bps} bps of its SOL bins paid ${sol(got)} SOL to the dev wallet (owed ${sol(m.due.due)}); ${sol(R.withdrawn)} SOL paid in all`);
}

for (const t of sched) {
  if (R.tranches.some((x) => x.k === t.k)) continue;
  if (BUYS[t.k] && !R.buys.some((b) => b.beforeTranche === t.k)) await buyNow(t.k, BUYS[t.k]);
  const pair = await pairState();
  const amount = t.amount + R.carry;
  const tr = PL.tranche({ a, active: pair.activeId, width, amount });
  const haveAccts = await c.accounts(tr.indexes.flatMap((i) => [C.binArrayAddress(a.pair.lbPair, i), C.positionAddress(a.pair.lbPair, a.launchKey, i * C.BINS_PER_ARRAY)]));
  const have = { arrays: new Set(tr.indexes.filter((_, j) => haveAccts[2 * j])), positions: new Set(tr.indexes.filter((_, j) => haveAccts[2 * j + 1])) };
  const prep = PL.prepare({ a, indexes: tr.indexes, have });
  for (let i = 0; i < prep.ixs.length; i += 2) await send(`prepare for tranche ${t.k}`, { ixs: prep.ixs.slice(i, i + 2) }, [k.launch]);
  const res = await send(`tranche ${t.k}`, tr, [k.launch], { scheduledFor: t.at, laid: tr.laid.toString(), activeId: pair.activeId, bins: `${pair.activeId + 1}..${pair.activeId + width}` });
  R.carry = tr.carry;
  R.tranches.push({ k: t.k, scheduledFor: t.at, rehearsedAt: now(), activeId: pair.activeId, from: pair.activeId + 1, to: pair.activeId + width, laid: tr.laid,
    carry: tr.carry, multiplier: tr.multiplier, indexes: tr.indexes, signature: res.signature });
  save();
  console.log(`    tranche ${t.k} (due ${t.at}): ${tok(tr.laid)} $WARM over bins ${pair.activeId + 1}..${pair.activeId + width}`);
  if (p.moneyAfterTranches.includes(t.k) && !R.money.some((m) => m.label === `after tranche ${t.k}`)) await moneyNow(`after tranche ${t.k}`);
}

if (!R.money.some((m) => m.label === 'at the close')) await moneyNow('at the close');

// ---- the receipts: what the chain says now ----
const mintAcct = C.readMint((await c.account(a.mint)).data);
const fin = await snapshot();
const finHeld = L.positionSol(await positionsState(owned()), fin.activeId);
const protocolFeeSol = (await c.account(a.pair.lbPair)).data.readBigUInt64LE(224);    // LbPair.protocol_fee.amount_y: Meteora's share of the swap fees
const buyerSwarm = await c.tokenAmount(C.ata(k.buyer.address, a.mint));
const treasury = await c.tokenAmount(a.treasurySwarm);
const launchLeft = await c.tokenAmount(a.launchSwarm);
const laid = R.tranches.reduce((s, t) => s + t.laid, 0n);
const paidIn = R.buys.reduce((s, b) => s + b.lamports, 0n);
const out = {
  what: 'THE $WARM LAUNCH rehearsed on devnet: the mint, the 90/10 split, the DLMM pool, 16 tranches, a test buyer, the money',
  cluster: 'devnet', rpc: RPC, tag: TAG, started: R.started, finished: now(),
  params: { ...p, supplyBase: p.supplyBase.toString() },
  addresses: { launchKey: a.launchKey, mint: a.mint, metadata: a.metadata, lbPair: a.pair.lbPair, reserveX: a.pair.reserveX, reserveY: a.pair.reserveY,
    treasuryMultisig: squads.multisig, treasuryVault: a.treasuryOwner, treasurySwarm: a.treasurySwarm, devWallet: a.devWallet, devWsol: a.devWsol, buyer: k.buyer.address },
  links: { mint: at(a.mint), pair: at(a.pair.lbPair), treasury: at(a.treasurySwarm), dev: at(a.devWsol) },
  mint: { supply: mintAcct.supply.toString(), decimals: mintAcct.decimals, mintAuthority: mintAcct.mintAuthority, freezeAuthority: mintAcct.freezeAuthority },
  holdings: { treasury: treasury.toString(), launchKeyLeft: launchLeft.toString(), pool: fin.reserveSwarm.toString(), buyer: buyerSwarm.toString(),
    sumsToSupply: (treasury + launchLeft + fin.reserveSwarm + buyerSwarm).toString() === mintAcct.supply.toString() },
  sale: { laid: laid.toString(), carriedAtEnd: R.carry.toString(), tranches: R.tranches.length },
  price: { startBin: R.tranches[0]?.activeId, endBin: fin.activeId, start: L.uiPriceOfBin(R.tranches[0]?.activeId, p.binStep, p.decimals), end: fin.price },
  money: { buyersPaid: paidIn.toString(), poolSolNow: fin.solInPosition.toString(), poolFeesUnclaimed: finHeld.feeY.toString(), meteoraProtocolFee: protocolFeeSol.toString(),
    reserveSolNow: fin.reserveSol.toString(), devPaid: R.withdrawn.toString(), takenIn: (fin.solInPosition + finHeld.feeY + R.withdrawn).toString(), rounds: R.money },
  buys: R.buys, tranches: R.tranches, receipts: R.receipts,
};
writeFileSync(OUT, JSON.stringify(out, (key, v) => (typeof v === 'bigint' ? v.toString() : v), 1) + '\n');
console.log(`\nTHE MINT ${a.mint}: supply ${tok(mintAcct.supply)}, mint authority ${mintAcct.mintAuthority ?? 'none'}, freeze authority ${mintAcct.freezeAuthority ?? 'none'}`);
console.log(`  treasury ${tok(treasury)} · pool ${tok(fin.reserveSwarm)} · buyer ${tok(buyerSwarm)} · launch key ${tok(launchLeft)} · sums to the supply: ${out.holdings.sumsToSupply}`);
console.log(`  price: bin ${out.price.startBin} -> ${fin.activeId}, ${out.price.start.toExponential(4)} -> ${fin.price.toExponential(4)} SOL per $WARM`);
console.log(`  money: buyers paid ${sol(paidIn)} SOL; Meteora kept ${sol(protocolFeeSol)} of the swap fees; the position took in ${sol(fin.solInPosition + finHeld.feeY + R.withdrawn)}: the dev wallet got ${sol(R.withdrawn)}, the pool keeps ${sol(fin.solInPosition)} in its bins and ${sol(finHeld.feeY)} of earned fees`);
console.log(`  receipts: ${OUT}`);
