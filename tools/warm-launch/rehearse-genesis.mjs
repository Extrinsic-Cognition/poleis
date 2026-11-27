#!/usr/bin/env node
// rehearse-genesis.mjs · THE GENESIS SHARES THE TENTH, rehearsed on devnet (Nov 19 2026). The same sets as `mainnet.mjs genesis` and `share`, signed by
// the Keystone page's own road (poleis/ceremony/keystone/prepared.mjs: readSet, toSign with a fresh blockhash, signedWire) with a throwaway key in the
// Keystone's place, sent on devnet only (rpc.mjs's genesis check). Before the genesis, pre-deposits into a stand-in treasury by the deposit door's own
// messages (depositMessage): a flask whose fee the house pays (twice), a depositor who pays its own, the house's USDC for a flask; after it one deposit
// more, which must not count. Then every balance is read back. Keys: throwaway, in a folder outside the house (--keys); test SOL from --bank.
// Usage: node tools/warm-launch/rehearse-genesis.mjs --keys DIR --bank KEYPAIR [--rpc URL] [--tag NAME] [--out FILE]
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { freshKeypair } from '../../poleis/ceremony/wire.mjs';
import { readSet, toSign, signedWire } from '../../poleis/ceremony/keystone/prepared.mjs';
import { depositMessage, relayedDeposit, signedDeposit } from '../../embertide-app/ui/intro/dyson-society/app/space-router-deposit.mjs';
import * as C from './chain.mjs';
import * as M from './mainnet.mjs';
import { cluster, signerOf } from './rpc.mjs';

const HERE = dirname(fileURLToPath(import.meta.url)), arg = (k, d) => { const i = process.argv.indexOf(`--${k}`); return i > 0 ? process.argv[i + 1] : d; };
const KEYS = arg('keys'), BANK = arg('bank'), TAG = arg('tag', `g${Date.now().toString(36)}`), sleep = (ms) => new Promise((ok) => setTimeout(ok, ms));
if (!KEYS || !BANK) throw new Error('--keys (a folder outside the house) and --bank (a funded devnet keypair) are needed');
if (resolve(KEYS).startsWith(resolve(HERE, '..', '..'))) throw new Error('--keys must lie outside the house: no key file goes in git');
const c = cluster(arg('rpc', 'https://api.devnet.solana.com'), { log: console.log }); await c.devnetOnly();
mkdirSync(KEYS, { recursive: true, mode: 0o700 });
const key = (name) => { const f = join(KEYS, `${TAG}-${name}.json`); if (!existsSync(f)) writeFileSync(f, JSON.stringify([...freshKeypair()]), { mode: 0o600 }); return signerOf(JSON.parse(readFileSync(f, 'utf8'))); };
const bank = signerOf(JSON.parse(readFileSync(BANK, 'utf8'))), K = Object.fromEntries(['launch', 'vault', 'usdc', 'fee', 'till', 'flask', 'own', 'poured'].map((n) => [n, key(n)]));
const R = { what: 'THE GENESIS SHARES THE TENTH, rehearsed on devnet', tag: TAG, at: new Date().toISOString(), keys: Object.fromEntries(Object.entries(K).map(([n, k]) => [n, k.address])), steps: [] };
const sendRaw = async (wire) => { await c.devnetOnly(); return c.call('sendTransaction', [wire, { encoding: 'base64', preflightCommitment: 'confirmed' }]); };   // every send after the genesis check
const read = async (sig) => { for (let i = 0; i < 10; i++) { const tx = await c.call('getTransaction', [sig, { encoding: 'jsonParsed', maxSupportedTransactionVersion: 0, commitment: 'finalized' }]); if (tx) return tx; await sleep(1500); } return null; };
async function final(signature, label) {
  for (let i = 0; i < 90; i++) {
    await sleep(1000);
    const st = (await c.call('getSignatureStatuses', [[signature]])).value[0];
    if (st?.err) throw new Error(`${label}: devnet refused ${signature}: ${JSON.stringify(st.err)}`);
    if (st?.confirmationStatus !== 'finalized') continue;
    const tx = await read(signature), step = { label, signature, slot: tx.slot, blockTime: tx.blockTime, units: tx.meta.computeUnitsConsumed };
    console.log(`  ${label.padEnd(46)} ${signature}  ${step.units} CU`); R.steps.push(step); return step;
  }
  throw new Error(`${label}: ${signature} not finalized in 90 s`);
}

// 1. the stand-ins: SOL from the bank, a USDC of 6 decimals, the treasury the account of `till` (the door's is a USDC account too)
await c.send([[K.launch, 30_000_000n], [K.usdc, 20_000_000n], [K.fee, 5_000_000n], [K.own, 5_000_000n]].map(([k, l]) => C.transfer(bank.address, k.address, l)), bank.address, [bank], { label: 'fund the stand-ins' });
const usdcSeed = `usdc-${TAG}`, usdc = C.withSeed(K.usdc.address, usdcSeed, C.P.token), at = (owner) => C.ata(owner, usdc), treasury = at(K.till.address);
await c.send([C.createAccountWithSeed(K.usdc.address, usdc, usdcSeed, await c.rent(C.SIZES.mint), C.SIZES.mint, C.P.token), C.initializeMint2(usdc, 6, K.usdc.address),
  ...[K.flask, K.own, K.fee, K.till].map((k) => C.createAtaIdempotent(K.usdc.address, k.address, usdc))], K.usdc.address, [K.usdc], { label: 'a stand-in USDC and its accounts' });
await c.send([[K.flask, 25_000_000n], [K.own, 42_500_000n], [K.fee, 600_412_645n]].map(([k, n]) => C.mintTo(usdc, at(k.address), K.usdc.address, n)), K.usdc.address, [K.usdc], { label: 'their USDC' });

// 2. the pre-deposits, as the door's pages send them; each row as the door counts it (the house's pour counted for the flask that asked)
async function deposit(label, { from, amount, fee = null, row = from.address }) {
  const { value } = await c.call('getLatestBlockhash', [{ commitment: 'finalized' }]);
  const message = depositMessage({ payer: from.address, source: at(from.address), treasury, mint: usdc, amount: String(amount), blockhash: value.blockhash, fee: fee?.address ?? null });
  const wire = fee ? relayedDeposit(message, fee.sign(message), from.sign(message)) : signedDeposit(message, from.sign(message));
  return { signature: (await final(await sendRaw(wire), label)).signature, payer: row };
}
const rows = await Promise.all([deposit('a flask, the house paying its fee', { from: K.flask, amount: 10_000_000n, fee: K.fee }), deposit('the same flask again', { from: K.flask, amount: 15_000_000n, fee: K.fee }),
  deposit('a depositor paying its own fee', { from: K.own, amount: 37_500_000n }), deposit("the house's USDC for a flask", { from: K.fee, amount: 600_412_645n, row: K.poured.address })]);

// 3. the genesis, by the Keystone page's road; its block time is the instant the pre-deposits close at
const raw = { ...JSON.parse(readFileSync(join(HERE, 'params.devnet.json'), 'utf8')), launchKey: K.launch.address, treasuryOwner: K.vault.address, seed: `swarm-${TAG}`, uri: 'https://optimism.fit/app/warm-token.json' };
async function keystone(set, label) {
  const s = await readSet(JSON.parse(JSON.stringify(set))), out = [];
  if (!s.signable) throw new Error(`${label}: the page would not sign it (${s.placeholders.join(', ')})`);
  for (const t of s.txs) {
    const { value } = await c.call('getLatestBlockhash', [{ commitment: 'finalized' }]), m = toSign(s, t, K.launch.address, value.blockhash);
    out.push(await final(await sendRaw(Buffer.from(signedWire(m, K.launch.sign(m))).toString('base64')), `${label}: ${t.name}`));
  }
  return out;
}
const genesis = M.genesisSet(raw), [g] = await keystone(genesis, 'the genesis'), before = new Date(g.blockTime * 1000).toISOString();
rows.push(await deposit('one deposit after the genesis', { from: K.own, amount: 5_000_000n }));

// 4. the share, read off the chain by the door's law, then signed and sent the same way; every balance read back
const share = await M.shareSet(raw, { rows, before, read, terms: { mint: usdc, treasury, fee: K.fee.address } });
const sent = await keystone(share, 'the share'), mint = genesis.addresses.mint, own = (o) => c.tokenAmount(C.ata(o, mint));
const m = C.readMint((await c.account(mint)).data), supply = 10n ** 18n, tenth = supply / 10n, got = {};
for (const s of share.shares) got[s.payer] = { share: s.share, holds: (await own(s.payer)).toString() };
const checks = {
  'the supply is 1 billion at 9 decimals': m.supply === supply, 'no one can mint more': m.mintAuthority === null, 'no one can freeze': m.freezeAuthority === null,
  'the 90% is the vault\'s': (await own(K.vault.address)) === supply - tenth, 'the launch key keeps nothing of the tenth': (await own(K.launch.address)) === 0n,
  'three payers, four deposits counted': share.shares.length === 3 && share.deposits.length === 4, 'the house\'s pour is the flask\'s': got[K.poured.address]?.share !== undefined,
  'the deposit after the genesis did not count': share.refused.length === 1 && /after the genesis/.test(share.refused[0].why),
  'every payer holds their share': share.shares.every((s) => got[s.payer].holds === s.share), 'the shares are the tenth': share.shares.reduce((t, s) => t + BigInt(s.share), 0n) === tenth,
};
Object.assign(R, { cluster: 'devnet', mint, usdc, treasury, before, rows, shares: share.shares, refused: share.refused, balances: got, transactions: { genesis: genesis.transactions.length, share: share.transactions.length }, sent: sent.length, checks });
const out = resolve(arg('out', join(HERE, 'receipts', `devnet-genesis-${R.at.slice(0, 10)}.json`)));
writeFileSync(out, JSON.stringify(R, (k, v) => (typeof v === 'bigint' ? v.toString() : v), 2) + '\n');
for (const [what, ok] of Object.entries(checks)) console.log(`  ${ok ? 'yes' : 'NO '}  ${what}`);
console.log(`${Object.values(checks).every(Boolean) ? 'THE GENESIS SHARES THE TENTH, on devnet' : 'NOT WHOLE'}: ${out}`);
process.exit(Object.values(checks).every(Boolean) ? 0 : 1);
