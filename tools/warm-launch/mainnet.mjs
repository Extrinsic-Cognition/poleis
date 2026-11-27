#!/usr/bin/env node
// mainnet.mjs · THE $WARM LAUNCH on mainnet, prepared and never signed here (Oct 30 2026). It emits every transaction unsigned for his Keystone to sign
// by QR on the house's Keystone page (poleis/ceremony/keystone/, its prepared road). It reads no key, signs nothing and sends nothing: `due` reads the
// cluster (getAccountInfo, getSignaturesForAddress, getTransaction) and nothing else.
//   emit [--params FILE] [--out FILE]      the genesis set: the mint and the split, the pool (trading opens at the opening), the first positions, the
//                                          first tranche; and the schedule of the rest. Every PLACEHOLDER of the params stays one in the set, as an
//                                          address or a number no one holds, and the set says it may not be signed. From the house's own mainnet
//                                          params (no --params) it also writes the page's module of the pool (PAGE_DOOR, THE DOOR OPENS WITH THE POOL):
//                                          the page's door to buy $WARM reads the pair's address there, and none while a placeholder stands.
//   due --k N | --money [--params FILE] [--rpc URL] [--out FILE]
//                                          one scheduled transaction built at its time from the live pair: tranche N laid above the price of the
//                                          moment, or the money owed now. Run it when the schedule says; his Keystone signs what it writes.
// The blockhash is left as zeros: the page stamps a fresh one just before it shows the QR (a blockhash lives about a minute).
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { base58Of } from '../solana-pda.mjs';
import * as C from './chain.mjs';
import * as L from './launch.mjs';
import * as PL from './plan.mjs';
import * as GS from './genesis-share.mjs';
import { cluster } from './rpc.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
// What each transaction used on devnet (receipts/devnet-rehearsal-2026-10-30.json), with room: the mainnet limits and the priority fee ride on them.
export const UNITS = Object.freeze({ genesis: 120_000, pool: 150_000, prepare: 320_000, tranche: 450_000, money: 600_000 });
export const RENT = Object.freeze({ mint: 1_461_600n });                 // 82 bytes at mainnet's rate, the same as devnet's

/** A placeholder address: sha256 of its words, so it is 32 bytes no one holds a key for, and the set names it as what it stands for. */
export const placeholderAddress = (field) => base58Of(createHash('sha256').update(`warm-launch placeholder: ${field}`).digest());
const STANDINS = Object.freeze({ supply: '1000000000', startPrice: '0.00000003', windowRatio: '2', uri: '' });

/** The params with every PLACEHOLDER replaced by a stand-in that is marked as one: addresses by placeholderAddress, numbers by the rehearsal's. */
export function withStandins(raw) {
  const placeholders = L.placeholdersIn(raw);
  const p = { ...raw };
  for (const f of placeholders) p[f] = f in STANDINS ? STANDINS[f] : placeholderAddress(f);
  return { p: L.readParams(p), placeholders };
}

const budget = (p, units) => [C.computeLimit(units), ...(p.priorityMicroLamports ? [C.computePrice(p.priorityMicroLamports)] : [])];
const KNOWN = new Map(Object.entries(C.P).map(([k, v]) => [v, k]));

/** One transaction as the page reads it: what it does, who signs, its message with a zero blockhash, the message's hash, its size. */
export function entry({ n, name, when, built, p, payer, units, placeholders, labels, does }) {
  const ixs = [...budget(p, units), ...built.ixs.filter((x) => x.program !== C.P.compute)];
  const c = C.compile(ixs, payer);
  if (c.size > C.MAX_TX) throw new Error(`${name}: ${c.size} bytes, over ${C.MAX_TX}`);
  return {
    n, name, when, does, signers: c.signers, signable: placeholders.length === 0, placeholders,
    accounts: c.accounts.map((o) => ({ address: o.key, signer: o.signer, writable: o.writable, is: labels.get(o.key) || KNOWN.get(o.key) || null })),
    message: c.message.toString('base64'), messageSha256: createHash('sha256').update(c.message).digest('hex'), bytes: c.size, computeUnits: units,
  };
}

function labelsOf(a, placeholders) {
  const vault = a.devWallet === a.treasuryOwner;
  const m = new Map([[a.launchKey, 'the launch key (his Keystone)'], [a.mint, 'the $WARM mint'], [a.metadata, 'its name (Metaplex)'], [a.pair.lbPair, 'the SWARM/SOL pair'],
    [a.pair.reserveX, "the pair's SWARM reserve"], [a.pair.reserveY, "the pair's SOL reserve"], [a.pair.oracle, "the pair's oracle"], [a.launchSwarm, "the launch key's SWARM"],
    [a.launchWsol, "the launch key's SOL account"], [a.devWallet, 'the dev wallet'], [a.devWsol, vault ? "his Squads vault's SOL account (the proceeds)" : "the dev wallet's SOL account"],
    [a.treasuryOwner, vault ? 'his Squads vault (the 90% and the proceeds)' : 'the treasury (his Squads vault)'], [a.treasurySwarm, "the treasury's SWARM"],
    [a.pair.eventAuthority, "DLMM's event authority"]]);
  for (const f of placeholders) m.set(placeholderAddress(f), `PLACEHOLDER ${f}`);
  return m;
}

export function emit(raw) {
  const { p, placeholders } = withStandins(raw);
  const a = PL.addresses({ launchKey: p.launchKey, seed: p.seed, treasuryOwner: p.treasuryOwner, devWallet: p.devWallet });
  const labels = labelsOf(a, placeholders);
  const opening = Math.floor(Date.parse(p.opening) / 1000);
  const sale = L.split(p.supplyBase).sale;
  const sched = L.schedule({ opening: p.opening, close: p.close, trancheHours: p.trancheHours, sale });
  const width = L.windowBins(p.windowRatio, p.binStep);
  const g = PL.genesis({ p, a, mintRent: RENT.mint });
  const pl = PL.pool({ p, a, activationPoint: opening });
  const t0 = PL.tranche({ a, active: pl.activeId, width, amount: sched[0].amount });
  const prep = PL.prepare({ a, indexes: t0.indexes });
  const before = `before ${p.opening} (trading opens then, by the pair's activation point)`;
  const common = { p, payer: a.launchKey, placeholders, labels };
  const txs = [
    entry({ ...common, n: 1, name: 'genesis', when: before, built: g, units: UNITS.genesis,
      does: [`make the mint from his address and the seed "${p.seed}": ${p.decimals} decimals, no freeze authority`, `mint ${g.amounts.treasury} base units (90%) to the treasury`,
        `mint ${g.amounts.sale} base units (10%) to the launch key, for the pool`, `name it ${p.name} (${p.symbol})`, 'revoke the mint authority: the supply is fixed'] }),
    entry({ ...common, n: 2, name: 'pool', when: before, built: pl, units: UNITS.pool,
      does: [`wrap ${PL.PROOF_LAMPORTS} lamports into the launch key's own SOL account (the program's proof of the creator; it stays his)`,
        `open the SWARM/SOL pair: bin ${pl.activeId} (${L.uiPriceOfBin(pl.activeId, p.binStep, p.decimals).toExponential(4)} SOL per ${p.symbol}), bin step ${p.binStep}, base fee ${p.feeBps} bps, trading from ${p.opening}`] }),
    ...chunk(prep.ixs, 2).map((ixs, i) => entry({ ...common, n: 3 + i, name: `prepare ${i + 1}`, when: before, built: { ixs }, units: UNITS.prepare,
      does: ixs.map((x) => (x.data[0] === 35 ? 'make a bin array of the pair (70 bins), his key pays its rent' : "make a position over it, his key its owner"))})),
  ];
  txs.push(entry({ ...common, n: txs.length + 1, name: 'tranche 0', when: before, built: t0, units: UNITS.tranche,
    does: [`lay ${t0.laid} base units of SWARM alone over bins ${pl.activeId + 1}..${pl.activeId + width} (${p.windowRatio}x above the opening price), equal in every bin`] }));
  return {
    what: `THE $WARM LAUNCH on ${raw.cluster === 'devnet' ? 'devnet (a rehearsal of the mainnet set)' : 'mainnet'}: every transaction unsigned, for his Keystone to sign by QR on the house's Keystone page`,
    cluster: raw.cluster === 'devnet' ? 'devnet' : 'mainnet', signable: placeholders.length === 0, placeholders, emitted: new Date().toISOString(),
    // The page asks the Keystone by the path and fingerprint it read from it when he connected it, so the set names only the address.
    signer: { address: a.launchKey, role: 'his Keystone: it signs every transaction of the launch, and no other key does' },
    params: Object.fromEntries(Object.entries(raw).filter(([k]) => k !== 'note')),
    addresses: { mint: a.mint, metadata: a.metadata, lbPair: a.pair.lbPair, reserveX: a.pair.reserveX, reserveY: a.pair.reserveY, oracle: a.pair.oracle,
      launchSwarm: a.launchSwarm, launchWsol: a.launchWsol, treasuryOwner: a.treasuryOwner, treasurySwarm: a.treasurySwarm, devWallet: a.devWallet, devWsol: a.devWsol },
    transactions: txs,
    schedule: sched.map((t) => ({ k: t.k, at: t.at, amount: t.amount.toString(), built: t.k === 0 ? 'in this set' : `at its time: node tools/warm-launch/mainnet.mjs due --k ${t.k}` })),
    money: p.moneyAfterTranches.map((k) => ({ afterTranche: k, at: sched[k]?.at, built: 'at its time: node tools/warm-launch/mainnet.mjs due --money' }))
      .concat([{ afterTranche: 'the close', at: p.close, built: 'at its time: node tools/warm-launch/mainnet.mjs due --money' }]),
  };
}
const chunk = (xs, n) => Array.from({ length: Math.ceil(xs.length / n) }, (_, i) => xs.slice(i * n, i * n + n));

// THE GENESIS SHARES THE TENTH (Nov 19 2026, his "we gotta mint $WARM and we gotta get going with our ecosystem", then "build as if we were launching"):
// the tenth is the pre-depositors' (genesis-share.mjs; the line: ec-workspace docs/warm-the-fuel-the-rebate-and-what-the-arrow-lets-go-hypertrace-2026-11-13.md).
// genesisSet is the mint alone (the 90% to his Squads, the tenth to the launch key, the authority revoked); shareSet, after it, sends the tenth to every
// pre-depositor in proportion, each deposit the door counted held to its own transaction. Both unsigned, for his Keystone; the sale's pool is another road.
const genesisDoes = (p, g) => [`make the mint from his address and the seed "${p.seed}": ${p.decimals} decimals, no freeze authority`, `mint ${g.amounts.treasury} base units (90%) to the treasury`,
  `mint ${g.amounts.sale} base units (10%) to the launch key: the pre-depositors' tenth, sent by \`mainnet.mjs share\``, `name it ${p.name} (${p.symbol})`, 'revoke the mint authority: the supply is fixed'];
const ownOf = (raw) => { const w = withStandins(raw); return { ...w, placeholders: w.placeholders.filter((f) => f !== 'startPrice' && f !== 'windowRatio') }; };   // the pool's numbers: neither set reads them
export function genesisSet(raw) {
  const { p, placeholders } = ownOf(raw), a = PL.addresses({ launchKey: p.launchKey, seed: p.seed, treasuryOwner: p.treasuryOwner, devWallet: p.devWallet }), g = PL.genesis({ p, a, mintRent: RENT.mint });
  const t = entry({ p, payer: a.launchKey, placeholders, labels: labelsOf(a, placeholders), n: 1, name: 'genesis', when: 'at the close, before the share', built: g, units: UNITS.genesis, does: genesisDoes(p, g) });
  return { what: `THE $WARM GENESIS on ${raw.cluster === 'devnet' ? 'devnet' : 'mainnet'}: the mint alone, unsigned, for his Keystone`, cluster: raw.cluster === 'devnet' ? 'devnet' : 'mainnet', signable: placeholders.length === 0,
    placeholders, emitted: new Date().toISOString(), signer: { address: a.launchKey, role: 'his Keystone' }, addresses: { mint: a.mint, metadata: a.metadata, launchSwarm: a.launchSwarm, treasuryOwner: a.treasuryOwner, treasurySwarm: a.treasurySwarm }, transactions: [t] };
}
/** The share: the door's rows ({ signature, payer }), each read off its own transaction (read(signature): getTransaction), held by the door's terms
 *  ({ mint, treasury, fee }, its GET), paid before the genesis instant (ISO), and the tenth split among them. */
export async function shareSet(raw, { rows, before, read, terms }) {
  const { p, placeholders } = ownOf(raw), a = PL.addresses({ launchKey: p.launchKey, seed: p.seed, treasuryOwner: p.treasuryOwner, devWallet: p.devWallet }), seen = new Map();
  for (const r of rows) seen.set(r.signature, await read(r.signature));
  const { counted, refused } = GS.countedOf(rows, seen, { mint: terms.mint, treasury: terms.treasury, fee: terms.fee, before: Date.parse(before) });
  const shares = GS.sharesOf(counted, L.split(p.supplyBase).sale), labels = labelsOf(a, placeholders), txs = GS.distribution({ a, shares, decimals: p.decimals, budget: budget(p, 400_000) });
  return { what: `THE $WARM SHARE on ${raw.cluster === 'devnet' ? 'devnet' : 'mainnet'}: the tenth to every pre-depositor before ${before}, unsigned, for his Keystone`, cluster: raw.cluster === 'devnet' ? 'devnet' : 'mainnet',
    signable: placeholders.length === 0 && shares.length > 0, placeholders, before, terms, refused, deposits: counted.map((d) => ({ signature: d.signature, payer: d.payer, amount: d.amount.toString(), paidAt: new Date(d.paidAt).toISOString() })),
    shares: shares.map((x) => ({ payer: x.payer, put: x.put.toString(), share: x.share.toString() })), transactions: txs.map((t, i) => entry({ p, payer: a.launchKey, placeholders, labels, n: i + 1, name: `share ${i + 1}`,
      when: 'after the genesis', built: { ixs: t.ixs }, units: 40_000 * t.rows.length + 20_000, does: t.rows.map((r) => `send ${r.share} base units of ${p.symbol} to ${r.payer}, who put in ${r.put} base units of USDC`) })) };
}

// THE DOOR OPENS WITH THE POOL (lane/the-door-opens-with-the-pool, Oct 30 2026): the page's door to buy $WARM (dyson-society app/warm-door.mjs) reads
// the pool's address from this module, written by emit, so the page and the set never name two pairs.
export const PAGE_DOOR = resolve(HERE, '../../embertide-app/ui/intro/dyson-society/app/warm-pair.mjs');

/** What the page's door reads of a set: the pool's address and its window. A set that still holds a placeholder names no address (an address no
 *  one holds is never a door), and a devnet set names none either: the page sells on mainnet only. */
export function pageDoor(set) {
  const named = set.cluster === 'mainnet' && set.signable === true;
  return { cluster: set.cluster, mint: named ? set.addresses.mint : null, pair: named ? set.addresses.lbPair : null, opening: set.params.opening, close: set.params.close };
}

/** The page's module for a door, as emit writes it. */
export const pageModule = (d) => [
  "// warm-pair.mjs · $WARM's pool as the mainnet set names it (THE DOOR OPENS WITH THE POOL, lane/the-door-opens-with-the-pool). Written by",
  '// `node tools/warm-launch/mainnet.mjs emit` from tools/warm-launch/params.mainnet.json, never by hand: warm-launch.test.mjs holds it to the emit.',
  "// While the set holds a placeholder it names no address, and the page's door to buy $WARM (warm-door.mjs) stays shut and asks nothing.",
  `export const WARM_PAIR = Object.freeze(${JSON.stringify(d)});`, ''].join('\n');

/** What the dev wallet has been paid by the money rounds, read from the chain alone: the SOL its account gained in every transaction that also moved
 *  the pair's SOL reserve. */
export async function paidToDev(c, a) {
  let paid = 0n, before;
  for (;;) {
    const sigs = await c.call('getSignaturesForAddress', [a.devWsol, { limit: 1000, ...(before ? { before } : {}) }]);
    for (const s of sigs) {
      if (s.err) continue;
      const t = await c.call('getTransaction', [s.signature, { encoding: 'jsonParsed', maxSupportedTransactionVersion: 0, commitment: 'confirmed' }]);
      const keys = t.transaction.message.accountKeys.map((k) => k.pubkey);
      // only the launch key's own rounds out of the pool: the vault's own trades, or anyone's swap paid into its SOL account, are not proceeds paid
      if (keys[0] !== a.launchKey || !keys.includes(a.pair.reserveY)) continue;
      const i = keys.indexOf(a.devWsol);
      const amt = (list) => BigInt(list.find((b) => b.accountIndex === i)?.uiTokenAmount.amount ?? '0');
      const d = amt(t.meta.postTokenBalances) - amt(t.meta.preTokenBalances);
      if (d > 0n) paid += d;
    }
    if (sigs.length < 1000) return paid;
    before = sigs.at(-1).signature;
  }
}

export async function due(raw, { k = null, money = false, rpc }) {
  const placeholders = L.placeholdersIn(raw);
  if (placeholders.length) throw new Error(`due: the params still hold placeholders (${placeholders.join(', ')}); nothing scheduled can be built until his numbers are in`);
  const p = L.readParams(raw);
  const c = cluster(rpc);
  const a = PL.addresses({ launchKey: p.launchKey, seed: p.seed, treasuryOwner: p.treasuryOwner, devWallet: p.devWallet });
  const pairAcct = await c.account(a.pair.lbPair);
  if (!pairAcct) throw new Error('due: the pair is not on chain yet (the genesis set comes first)');
  const pair = C.readPair(pairAcct.data);
  const labels = labelsOf(a, []);
  const common = { p, payer: a.launchKey, placeholders: [], labels };
  const sched = L.schedule({ opening: p.opening, close: p.close, trancheHours: p.trancheHours, sale: L.split(p.supplyBase).sale });
  const out = [];
  if (k !== null) {
    const t = sched[k];
    if (!t) throw new Error(`due: there is no tranche ${k}`);
    // what tranche k lays: all the launch key still holds, less what the later tranches are scheduled to lay (so a carry, or SWARM fees, ride along)
    const left = await c.tokenAmount(a.launchSwarm);
    const later = sched.filter((x) => x.k > k).reduce((s, x) => s + x.amount, 0n);
    const amount = left > later ? left - later : 0n;
    const tr = PL.tranche({ a, active: pair.activeId, width: L.windowBins(p.windowRatio, p.binStep), amount });
    if (tr.laid === 0n) return { what: `tranche ${k}: nothing to lay (the launch key holds ${left} base units, the later tranches are owed ${later})`, cluster: null, signable: false, placeholders: [], transactions: [] };
    const have = await c.accounts(tr.indexes.flatMap((i) => [C.binArrayAddress(a.pair.lbPair, i), C.positionAddress(a.pair.lbPair, a.launchKey, i * C.BINS_PER_ARRAY)]));
    const prep = PL.prepare({ a, indexes: tr.indexes, have: { arrays: new Set(tr.indexes.filter((_, j) => have[2 * j])), positions: new Set(tr.indexes.filter((_, j) => have[2 * j + 1])) } });
    chunk(prep.ixs, 2).forEach((ixs, i) => out.push(entry({ ...common, n: out.length + 1, name: `prepare for tranche ${k} (${i + 1})`, when: 'now, before the tranche', built: { ixs }, units: UNITS.prepare,
      does: ['make the bin array and position the window needs'] })));
    out.push(entry({ ...common, n: out.length + 1, name: `tranche ${k}`, when: `${t.at} (built ${new Date().toISOString()} at bin ${pair.activeId})`, built: tr, units: UNITS.tranche,
      does: [`lay ${tr.laid} base units of SWARM alone over bins ${pair.activeId + 1}..${pair.activeId + L.windowBins(p.windowRatio, p.binStep)}`] }));
  }
  if (money) {
    const indexes = [];
    for (let i = C.binArrayIndex(L.binOfPrice(p.startPrice, p.binStep, p.decimals)); i <= C.binArrayIndex(pair.activeId) + 2; i++) indexes.push(i);
    const accts = await c.accounts(indexes.flatMap((i) => [C.positionAddress(a.pair.lbPair, a.launchKey, i * C.BINS_PER_ARRAY), C.binArrayAddress(a.pair.lbPair, i)]));
    const positions = indexes.map((index, j) => (accts[2 * j] && accts[2 * j + 1] ? { index, ...C.readPosition(accts[2 * j].data), bins: C.readBinArray(accts[2 * j + 1].data).bins } : null)).filter(Boolean);
    const withdrawn = await paidToDev(c, a);
    const m = PL.money({ a, active: pair.activeId, positions, withdrawn, devShareBps: p.devShareBps });
    if (m.ixs.length) {
      out.push(entry({ ...common, n: out.length + 1, name: 'money', when: `now (built ${new Date().toISOString()} at bin ${pair.activeId})`, built: m, units: UNITS.money,
        does: [`the position has taken in ${m.due.takenIn} lamports; the dev wallet was paid ${withdrawn}; it is owed ${m.due.due}`,
          `claim ${m.due.fees} lamports of earned fees and take ${m.due.bps} bps of every SOL bin, into the dev wallet's SOL account`] }));
    }
  }
  return { what: 'THE $WARM LAUNCH: a scheduled transaction, unsigned, built now from the live pair', cluster: (await c.which()) === '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d' ? 'mainnet' : 'devnet',
    signable: true, placeholders: [], signer: { address: a.launchKey }, transactions: out };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const arg = (name, dflt) => { const i = process.argv.indexOf(`--${name}`); return i > 0 ? process.argv[i + 1] : dflt; };
  const raw = JSON.parse(readFileSync(resolve(arg('params', `${HERE}/params.mainnet.json`)), 'utf8'));
  const cmd = process.argv[2];
  const write = (set, file) => { mkdirSync(dirname(file), { recursive: true }); writeFileSync(file, JSON.stringify(set, null, 1) + '\n'); return file; };
  if (cmd === 'emit') {
    const set = emit(raw);
    const file = write(set, resolve(arg('out', `${HERE}/mainnet/unsigned-genesis.json`)));
    console.log(`${set.transactions.length} transactions, unsigned, ${set.signable ? 'signable' : `NOT signable: ${set.placeholders.length} placeholders (${set.placeholders.join(', ')})`}`);
    for (const t of set.transactions) console.log(`  ${String(t.n).padStart(2)}. ${t.name.padEnd(12)} ${t.bytes} B, ${t.computeUnits} CU, sha256 ${t.messageSha256.slice(0, 16)}…`);
    console.log(`  mint ${set.addresses.mint}\n  pair ${set.addresses.lbPair}\n  ${file}`);
    if (set.cluster === 'mainnet' && !process.argv.includes('--params')) {
      const d = pageDoor(set);
      writeFileSync(PAGE_DOOR, pageModule(d));
      console.log(`  the page's door: ${d.pair ? `the pair ${d.pair}` : 'no address while a placeholder stands'}: ${PAGE_DOOR}`);
    }
  } else if (cmd === 'genesis') {
    const set = genesisSet(raw), file = write(set, resolve(arg('out', `${HERE}/mainnet/unsigned-genesis-only.json`)));
    console.log(`the genesis, unsigned, ${set.signable ? 'signable' : `NOT signable: ${set.placeholders.length} placeholders (${set.placeholders.join(', ')})`}\n  mint ${set.addresses.mint}\n  ${file}`);
  } else if (cmd === 'share') {   // --deposits: the door's rows, as `wrangler d1 execute ... --json` answers SELECT signature, payer FROM sr_deposits
    const before = arg('before', null), terms = await (await fetch(arg('terms', 'https://optimism.fit/space-router/v1/deposit'))).json(), c = cluster(arg('rpc', 'https://api.mainnet-beta.solana.com'));
    if (!before || !Number.isFinite(Date.parse(before))) throw new Error('share: --before <ISO>, the genesis instant, is needed');
    const j = JSON.parse(readFileSync(arg('deposits', ''), 'utf8')), rows = Array.isArray(j) ? (j[0]?.results ?? j) : j.results ?? j.rows ?? [];
    const set = await shareSet(raw, { rows, before, terms, read: (sig) => c.call('getTransaction', [sig, { encoding: 'jsonParsed', maxSupportedTransactionVersion: 0, commitment: 'finalized' }]) });
    console.log(`${set.deposits.length} pre-deposits counted, ${set.refused.length} refused, ${set.shares.length} payers, ${set.transactions.length} transactions, unsigned: ${write(set, resolve(arg('out', `${process.env.HOME}/.embertide/warm-launch/share-${Date.now()}.json`)))}`);
  } else if (cmd === 'due') {
    const k = arg('k', null);
    const set = await due(raw, { k: k === null ? null : Number(k), money: process.argv.includes('--money'), rpc: arg('rpc', 'https://api.mainnet-beta.solana.com') });
    const file = write(set, resolve(arg('out', `${process.env.HOME}/.embertide/warm-launch/due-${Date.now()}.json`)));
    console.log(`${set.transactions.length} transactions, unsigned, on ${set.cluster}: ${file}`);
  } else {
    console.log('usage: mainnet.mjs emit [--params FILE] [--out FILE] | genesis [--params FILE] [--out FILE] | share --deposits FILE --before ISO [--terms URL] [--rpc URL] [--out FILE] | due --k N | --money [--params FILE] [--rpc URL] [--out FILE]');
    process.exitCode = 2;
  }
}
