// What no recording could show, shown with small made-up inputs: every one is marked as made up. The rows that read the
// future (a schedule nobody has published, a digest nobody has anchored) are held to their rules here.
import test from 'node:test';
import assert from 'node:assert/strict';
import { runChecks, mintMoves, outflowsIn, burnLines, tokens, share, when, PASS, FAIL, UNCHECKED } from '../lib/rows.mjs';
import { parseManifest, parseDigests } from '../lib/manifest.mjs';
import { createRpc, fixtureTransport, callKey } from '../lib/rpc.mjs';
import { exitOf } from '../lib/render.mjs';
import { memoOf, TAG } from '../../burn-digest/digest.mjs';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { noNetwork, fixture, root } from './helpers.mjs';

noNetwork();
const MINT = 'BoY2UJq7VJmyYzgszK9bzo6MFqQB4u5NEMhRupiDwJp4';
// Addresses below are real public keys used as stand-ins; nothing is claimed about them.
const HOUSE = 'BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs', OTHER = 'Boy7pkvoq3ipJ3qfxBzCFeA76x1segrZHEANHBX4XAoe', THIRD = '5M5ggeV4NET5NVENqTdCVauWGLuH2qgeoDYCmn1XkhSU';
const A1 = 'AjtoZGtMWx7AZZBUFsuzhhYZL2MhF7xmVxH3MJVsuGj5', A2 = '35zT1xgDyVjk48DAEt5sN4C2HggckhPPn3oTy2RjhPXn', A3 = '61G7eZ5JZ9Q8BxiM1B7vC1TMAHBuYZBP7pPjUF5qj9B4';

/** A made-up transaction in the shape getTransaction (jsonParsed) returns: balances of MINT before and after. */
function tx(before, after) {
  const keys = [...new Set([...before, ...after].map(b => b[0]))];
  const side = rows => rows.map(([account, owner, amount]) => ({ accountIndex: keys.indexOf(account), mint: MINT, owner, uiTokenAmount: { amount: String(amount) } }));
  return { slot: 1, blockTime: 1000, transaction: { message: { accountKeys: keys.map(pubkey => ({ pubkey })), instructions: [] } }, meta: { err: null, preTokenBalances: side(before), postTokenBalances: side(after), innerInstructions: [] } };
}

test('row 11, made-up transactions: what counts as a transfer out of a house wallet', () => {
  // a plain transfer to a stranger
  assert.deepEqual(outflowsIn(tx([[A1, HOUSE, 100], [A2, OTHER, 5]], [[A1, HOUSE, 60], [A2, OTHER, 45]]), MINT, [HOUSE]), [{ wallet: HOUSE, amount: 40n, to: [{ account: A2, owner: OTHER, amount: 40n }] }]);
  // moving between two accounts of the same wallet is not a transfer out
  assert.deepEqual(outflowsIn(tx([[A1, HOUSE, 100], [A2, HOUSE, 0]], [[A1, HOUSE, 60], [A2, HOUSE, 40]]), MINT, [HOUSE]), []);
  // receiving is not a transfer out, and a transaction between strangers is nothing
  assert.deepEqual(outflowsIn(tx([[A1, HOUSE, 1], [A2, OTHER, 9]], [[A1, HOUSE, 10], [A2, OTHER, 0]]), MINT, [HOUSE]), []);
  assert.deepEqual(outflowsIn(tx([[A2, OTHER, 9], [A3, THIRD, 0]], [[A2, OTHER, 0], [A3, THIRD, 9]]), MINT, [HOUSE]), []);
  // a burn has no destination
  assert.deepEqual(outflowsIn(tx([[A1, HOUSE, 100]], [[A1, HOUSE, 70]]), MINT, [HOUSE]), [{ wallet: HOUSE, amount: 30n, to: [{ account: null, owner: null, amount: 30n, burned: true }] }]);
  // handing the token account itself to another owner moves everything in it, with no balance change
  assert.deepEqual(outflowsIn(tx([[A1, HOUSE, 100]], [[A1, OTHER, 100]]), MINT, [HOUSE]), [{ wallet: HOUSE, amount: 100n, to: [{ account: A1, owner: OTHER, amount: 100n }] }]);
  // an account created and filled in the transaction, and one emptied and closed in it
  assert.deepEqual(outflowsIn(tx([[A1, HOUSE, 100]], [[A2, OTHER, 100]]), MINT, [HOUSE]), [{ wallet: HOUSE, amount: 100n, to: [{ account: A2, owner: OTHER, amount: 100n }] }]);
  // two house wallets in one transaction are each judged
  assert.equal(outflowsIn(tx([[A1, HOUSE, 50], [A2, OTHER, 50], [A3, THIRD, 0]], [[A1, HOUSE, 0], [A2, OTHER, 0], [A3, THIRD, 100]]), MINT, [HOUSE, OTHER]).length, 2);
  assert.deepEqual(mintMoves(tx([[A1, HOUSE, 7]], [[A1, HOUSE, 7]]), MINT), [{ account: A1, ownerBefore: HOUSE, ownerAfter: HOUSE, before: 7n, after: 7n }]);
});

/** The SRA recording with its manifest swapped: a real pool, real wallets, a made-up schedule or fund. */
async function sra(change, opts = {}) {
  const fx = fixture('sra');
  if (opts.patch) opts.patch(fx);
  const manifest = parseManifest(change({ ...fx.manifest }), fx.meta.mint);
  return runChecks({ rpc: createRpc(fixtureTransport(fx)), mint: fx.meta.mint, manifest, history: fx.meta.history });
}

test('row 11, a made-up schedule over real transfers: a transfer the schedule covers is not flagged, one it does not is', async () => {
  const flagged = (await sra(m => m)).rows[10].data.flagged;
  const lines = flagged.flatMap(f => f.to.map(d => ({ label: 'made up', from: f.wallet, to: d.account, amount: d.amount.toString() })));
  const covered = (await sra(m => ({ ...m, schedule: lines }))).rows[10];
  assert.equal(covered.data.flagged.length, 0);
  assert.equal(covered.data.onSchedule, 5);
  // the wallet has more transactions than were read (history 6), so a clean reading is still not a pass
  assert.equal(covered.verdict, UNCHECKED);
  assert.match(covered.detail.at(-1), /more than 6 transactions; only the newest 6 were read/);
  assert.match(covered.notes.join(' '), /16 of 16 schedule lines matched a transfer/);
  // one base unit short on one line: that transfer is off the schedule by one unit
  const short = lines.map((l, i) => (i === 0 ? { ...l, amount: (BigInt(l.amount) - 1n).toString() } : l));
  const off = (await sra(m => ({ ...m, schedule: short }))).rows[10];
  assert.equal(off.verdict, FAIL);
  assert.deepEqual(off.data.flagged.map(f => f.unscheduled.map(d => d.amount)), [[1n]]);
  // the schedule may name the receiving wallet instead of its token account; a window that has not opened covers nothing
  const byOwner = flagged.flatMap(f => f.to.map(d => ({ from: f.wallet, to: d.owner, amount: d.amount.toString() })));
  assert.equal((await sra(m => ({ ...m, schedule: byOwner }))).rows[10].data.flagged.length, 0);
  assert.equal((await sra(m => ({ ...m, schedule: byOwner.map(l => ({ ...l, notBefore: 1778641101 })) }))).rows[10].data.flagged.length, 5);
  assert.equal((await sra(m => ({ ...m, schedule: byOwner.map(l => ({ ...l, notBefore: 1778641100, notAfter: 1778641100 })) }))).rows[10].data.flagged.length, 0);
});

test('row 12, one field of a real claim changed: the same claim emptied to the fund itself passes; any other fund fails', async () => {
  // made up: the closeAccount of the real claim transaction sends its balance to the holder instead of another wallet
  const toFund = fx => {
    for (const [key, answer] of Object.entries(fx.calls)) {
      if (!key.startsWith('getTransaction ["3Ttac7qk')) continue;
      for (const ix of [...answer.transaction.message.instructions, ...answer.meta.innerInstructions.flatMap(g => g.instructions)]) if (ix.parsed?.type === 'closeAccount' && ix.parsed.info.account === 'A8DxcpM8PbVPqq1xTDtkRaGN1LooLSrc8tKwvRCnZnME') ix.parsed.info.destination = HOUSE;
    }
  };
  const pass = (await sra(m => m, { patch: toFund })).rows[11];
  assert.equal(pass.verdict, PASS);
  assert.match(pass.detail[0], /Every named position is held by the fund, BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs, so only the fund can claim, and all 1 claim so far were paid into accounts the fund owns/);
  assert.match(pass.notes.at(-1), /cannot read the next one/);
  // as recorded, the balance went to another wallet
  assert.equal((await sra(m => m)).rows[11].verdict, FAIL);
  // a fund that does not hold the position
  const other = (await sra(m => ({ ...m, fund: { address: OTHER } }))).rows[11];
  assert.equal(other.verdict, FAIL);
  assert.match(other.detail[0], /is held by BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs, not the fund/);
  // no fund named, no position named
  assert.equal((await sra(m => { const { fund, ...rest } = m; return rest; })).rows[11].verdict, UNCHECKED);
  assert.equal((await sra(m => ({ ...m, pool: { address: m.pool.address } }))).rows[11].verdict, UNCHECKED);
});

test('row 9 without a named position: all locked passes on the pool alone', async () => {
  const nine = (await sra(m => ({ ...m, pool: { address: m.pool.address } }))).rows[8];
  assert.equal(nine.verdict, PASS);
  assert.match(nine.detail[0], /All of the pool's liquidity is permanently locked/);
});

test('row 5: a table that names every holder passes; a labelled account of another mint, or one named twice, does not', async () => {
  const fx = fixture('sra');
  const [, holders] = Object.entries(fx.calls).find(([k]) => k.startsWith('getProgramAccounts ["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"'));
  // made up: a recording in which the cluster returns exactly the two labelled accounts, which hold the whole supply
  const two = holders.filter(h => Buffer.from(h.account.data[0], 'base64').readBigUInt64LE(0) > 0n).slice(0, 2);
  const whole = (owner, mint, amount) => { const d = Buffer.alloc(165); d.set(Buffer.from(fx.calls[callKey('getMultipleAccounts', [[A1], { encoding: 'base64', commitment: 'finalized' }])].value[0].data[0], 'base64')); d.writeBigUInt64LE(amount, 64); return { data: [d.toString('base64'), 'base64'], executable: false, lamports: 2039280, owner: 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', rentEpoch: 0, space: 165 }; };
  const supply = 999911027513532n, first = 969446620166100n;
  const addresses = [A1, two.find(h => h.pubkey !== A1).pubkey];
  fx.calls[callKey('getMultipleAccounts', [addresses, { encoding: 'base64', commitment: 'finalized' }])] = { context: { slot: 1 }, value: [whole(null, MINT, first), whole(null, MINT, supply - first)] };
  const manifest = parseManifest({ mint: MINT, accounts: [{ label: 'pool vault', address: addresses[0] }, { label: 'fund lock', address: addresses[1] }] }, MINT);
  const five = (await runChecks({ rpc: createRpc(fixtureTransport(fx)), mint: MINT, manifest })).rows[4];
  assert.equal(five.data.gap, 0n);
  assert.equal(five.verdict, PASS);
  // as recorded: the vault alone is 96.95% of the supply
  assert.equal((await sra(m => ({ mint: m.mint, accounts: m.accounts }))).rows[4].verdict, FAIL);
  assert.throws(() => parseManifest({ accounts: [{ label: 'a', address: A1 }, { label: 'b', address: A1 }] }, MINT), /names AjtoZG.* twice/);
});

test('row 1, a made-up resolver: a TXT record that names the mint leaves the row for a person; one that does not, or none, fails it', async () => {
  const fx = fixture('wif'), mint = fx.meta.mint;
  const run = resolveTxt => runChecks({ rpc: createRpc(fixtureTransport(fx)), mint, domain: 'example.com', resolveTxt });
  const named = (await run(async () => [['v=spf1 -all'], ['warm-mint=', mint]])).rows[0];
  assert.equal(named.verdict, UNCHECKED);
  assert.match(named.detail[0], /names this mint: "warm-mint=EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm"\. That is one place of four/);
  const another = (await run(async () => [['mint=BoY2UJq7VJmyYzgszK9bzo6MFqQB4u5NEMhRupiDwJp4']])).rows[0];
  assert.equal(another.verdict, FAIL);
  assert.match(another.detail[0], /Named there instead: BoY2UJq7VJmyYzgszK9bzo6MFqQB4u5NEMhRupiDwJp4/);
  // an address that only contains the mint's characters inside a longer one is not the mint
  assert.equal((await run(async () => [[`x${mint}y`]])).rows[0].verdict, FAIL);
  assert.equal((await run(async () => { throw Object.assign(new Error('no data'), { code: 'ENODATA' }); })).rows[0].verdict, FAIL);
  assert.equal((await run(async () => { throw Object.assign(new Error('timeout'), { code: 'ETIMEOUT' }); })).rows[0].verdict, UNCHECKED);
  assert.equal((await runChecks({ rpc: createRpc(fixtureTransport(fx)), mint })).rows[0].verdict, UNCHECKED);
});

// Row 13: nothing is anchored on any chain yet, so every signature list below is made up. The lines are the ones
// house/tools/burn-digest prints (its memoOf), so a change of its format breaks this test before it breaks the row.
const ANCHOR = 'Boy7pkvoq3ipJ3qfxBzCFeA76x1segrZHEANHBX4XAoe';
const digestOf = day => ({ day, rows: 2, cost_usd: '0.300000', digest: (day.replaceAll('-', '') + 'ab').padEnd(64, '0') });
async function row13(days, { clockTime = Date.parse('2026-10-13T08:00:00Z') / 1000, history = 200, digests = null, memos = null, signer = () => ANCHOR } = {}) {
  const fx = fixture('wif');
  const list = (memos || days.map(d => `[92] ${memoOf(digestOf(d))}`)).map((memo, i) => ({ signature: `made-up-${i}`, slot: 1000 + i, err: null, memo, blockTime: clockTime - i * 60 }));
  fx.calls[callKey('getSignaturesForAddress', [ANCHOR, { limit: history, commitment: 'finalized' }])] = list;
  // each made-up transaction names the anchoring address; `signer` says who signed it
  list.forEach((s, i) => { fx.calls[callKey('getTransaction', [s.signature, { encoding: 'jsonParsed', commitment: 'finalized', maxSupportedTransactionVersion: 0 }])] = { slot: s.slot, blockTime: s.blockTime, meta: { err: null }, transaction: { message: { accountKeys: [...new Set([signer(i), ANCHOR])].map(pubkey => ({ pubkey, signer: pubkey === signer(i), writable: true })), instructions: [] } } }; });
  const slotKey = callKey('getSlot', [{ commitment: 'finalized' }]);
  fx.calls[callKey('getBlockTime', [fx.calls[slotKey]])] = clockTime;
  const manifest = parseManifest({ burnDigest: { address: ANCHOR } }, fx.meta.mint);
  return (await runChecks({ rpc: createRpc(fixtureTransport(fx)), mint: fx.meta.mint, manifest, history, digests })).rows[12];
}

test('row 13, made-up memos: one line for every closed day passes; a missing day, two hashes for a day, or none at all fail', async () => {
  assert.equal(TAG, 'burn-digest/1');
  assert.deepEqual(burnLines(`[92] ${memoOf(digestOf('2026-10-12'))}`), [{ day: '2026-10-12', rows: 2, costUsd: '0.300000', sha256: digestOf('2026-10-12').digest }]);
  assert.deepEqual(burnLines('[5] hello; [4] gm'), []); assert.deepEqual(burnLines(null), []);
  const ok = await row13(['2026-10-12', '2026-10-11', '2026-10-10']);
  assert.equal(ok.verdict, PASS);
  assert.match(ok.detail[0], /One line for every closed day from 2026-10-10 to 2026-10-12, none missing, none contradicted, each signed by Boy7pk/);
  assert.match(ok.notes[0], /recomputes that day's hash from its receipts in the burn book/);
  const gap = await row13(['2026-10-12', '2026-10-10']);
  assert.equal(gap.verdict, FAIL); assert.deepEqual(gap.data.missing, ['2026-10-11']);
  // yesterday closed ten minutes after midnight and has no line yet
  assert.deepEqual((await row13(['2026-10-11', '2026-10-10'])).data.missing, ['2026-10-12']);
  // nine minutes after midnight yesterday has not closed: nothing is missing
  assert.equal((await row13(['2026-10-11', '2026-10-10'], { clockTime: Date.parse('2026-10-13T00:09:00Z') / 1000 })).verdict, PASS);
  const twice = await row13([], { memos: [`[92] ${memoOf(digestOf('2026-10-12'))}`, `[92] ${memoOf({ ...digestOf('2026-10-12'), digest: 'f'.repeat(64) })}`] });
  assert.equal(twice.verdict, FAIL); assert.deepEqual(twice.data.conflicting, ['2026-10-12']);
  const none = await row13([], { memos: ['[2] gm', null] });
  assert.equal(none.verdict, FAIL); assert.match(none.detail[0], /has anchored no burn-digest line in its 2 newest transactions\. The spend is not on chain/);
  // an address busier than the history read: nothing found is not proof of nothing
  assert.equal((await row13([], { memos: ['[2] gm', '[2] gn'], history: 2 })).verdict, UNCHECKED);
  assert.equal((await row13(['2026-10-12', '2026-10-11'], { history: 2 })).verdict, UNCHECKED);
});

test('row 13, made up: a look-alike line in a transaction the address did not sign is not counted', async () => {
  // a stranger names the address beside a line for the missing day: the day stays missing
  const days = ['2026-10-12', '2026-10-11', '2026-10-10'];
  const forged = await row13(days, { signer: i => (i === 1 ? HOUSE : ANCHOR) });
  assert.equal(forged.verdict, FAIL);
  assert.deepEqual(forged.data.missing, ['2026-10-11']);
  assert.equal(forged.data.notSigned, 1);
  assert.match(forged.notes.join(' '), /1 line in transactions Boy7pk\S+ did not sign was not counted/);
  // a stranger's second hash for a day the address did anchor contradicts nothing
  const memos = [...days.map(d => `[92] ${memoOf(digestOf(d))}`), `[92] ${memoOf({ ...digestOf('2026-10-12'), digest: 'f'.repeat(64) })}`];
  const noise = await row13([], { memos, signer: i => (i === 3 ? HOUSE : ANCHOR) });
  assert.equal(noise.verdict, PASS); assert.deepEqual(noise.data.conflicting, []);
  // only strangers' lines: nothing is anchored
  assert.equal((await row13(days, { signer: () => HOUSE })).verdict, FAIL);
});

test('row 13 against burn-digest\'s own record: a line on chain that is not the record\'s fails', async () => {
  const record = days => parseDigests(days.map(d => JSON.stringify({ ...digestOf(d), unpriced: 0, prev: '0'.repeat(64) })).join('\n') + '\n');
  const same = await row13(['2026-10-12', '2026-10-11'], { digests: record(['2026-10-11', '2026-10-12']) });
  assert.equal(same.verdict, PASS); assert.equal(same.data.compared, 2);
  const changed = new Map(record(['2026-10-11', '2026-10-12'])); changed.set('2026-10-12', { rows: 3, cost_usd: '0.300000', digest: digestOf('2026-10-12').digest });
  const off = await row13(['2026-10-12', '2026-10-11'], { digests: changed });
  assert.equal(off.verdict, FAIL); assert.match(off.detail[0], /2026-10-12: the line on chain .* is not the record's/);
  assert.equal((await row13(['2026-10-12', '2026-10-11'], { digests: record(['2026-10-11']) })).verdict, FAIL);
  assert.throws(() => parseDigests('not json\n'), /not JSON/);
});

test('the manifest is read strictly: an unknown field, a bad address or a schedule from a stranger stops the run', () => {
  const ok = parseManifest({ about: 'x', mint: MINT, accounts: [{ label: 'vault', address: A1 }], pool: { address: A2, positions: [A3] }, locks: [{ label: 'l', address: A3 }], fund: { address: HOUSE }, houseWallets: [{ label: 'deployer', address: HOUSE }], schedule: [{ from: HOUSE, to: 'burn', amount: '5' }, { label: 's', from: HOUSE, to: OTHER, amount: '7', notBefore: 1, notAfter: 2 }], burnDigest: { address: OTHER } }, MINT);
  assert.deepEqual(ok.schedule[1], { label: 's', from: HOUSE, to: OTHER, amount: '7', notBefore: 1, notAfter: 2 });
  assert.deepEqual(ok.pool, { address: A2, positions: [A3] });
  assert.deepEqual(parseManifest({}, MINT), { accounts: [], locks: [], houseWallets: [], schedule: [], pool: null, fund: null, burnDigest: null });
  for (const [bad, why] of [
    [{ houseWallet: [] }, /unknown field "houseWallet"/], [{ mint: A1 }, /it is for the mint/], [{ accounts: [{ label: 'x', address: 'nope' }] }, /not a Solana address/],
    [{ accounts: [{ address: A1 }] }, /needs a label/], [{ accounts: [{ label: 'x', address: A1, amount: 1 }] }, /unknown field "amount"/], [{ pool: { address: A1, position: A2 } }, /unknown field "position"/],
    [{ schedule: [{ from: HOUSE, to: OTHER, amount: '5' }] }, /not one of the house wallets/], [{ houseWallets: [{ label: 'd', address: HOUSE }], schedule: [{ from: HOUSE, to: OTHER, amount: 5 }] }, /written as a string/],
    [{ houseWallets: [{ label: 'd', address: HOUSE }], schedule: [{ from: HOUSE, to: OTHER, amount: '0' }] }, /whole number/], [{ houseWallets: [{ label: 'd', address: HOUSE }], schedule: [{ from: HOUSE, to: OTHER, amount: '5', notBefore: -1 }] }, /unix time/],
    [{ locks: 'x' }, /must be a list/], [[], /must be an object/], [{ fund: HOUSE }, /must be an object/],
  ]) assert.throws(() => parseManifest(bad, MINT), why);
});

test('the example manifest is a manifest', () => {
  const example = JSON.parse(readFileSync(join(root, 'manifest.example.json'), 'utf8'));
  const m = parseManifest(example, example.mint);
  assert.deepEqual([m.accounts.length, m.locks.length, m.houseWallets.length, m.schedule.length, m.pool.positions.length], [2, 1, 2, 2, 1]);
  assert.equal(m.schedule[1].to, 'burn');
});

test('amounts, shares, times and the exit rule are written one way', () => {
  assert.equal(tokens(999911027513532n, 6), '999,911,027.513532'); assert.equal(tokens(1000000n, 6), '1'); assert.equal(tokens(0n, 6), '0'); assert.equal(tokens(5n, 0), '5'); assert.equal(tokens(-1500000n, 6), '-1.5');
  assert.equal(share(969446620166100n, 999911027513532n), '96.95%'); assert.equal(share(1n, 3n), '33.33%'); assert.equal(share(5n, 0n), 'n/a'); assert.equal(share(3n, 3n), '100.00%');
  assert.equal(when(1778641075n), '2026-05-13T02:57:55Z'); assert.equal(when(null), 'an unknown time'); assert.equal(when(2n ** 62n), `time ${2n ** 62n}`);
  const rows = v => ({ rows: Array.from({ length: 14 }, (_, i) => ({ n: i + 1, verdict: v[i + 1] || UNCHECKED })) });
  assert.deepEqual(exitOf(rows({})), { code: 0, why: 'no decidable row failed' });
  assert.deepEqual(exitOf(rows({ 3: FAIL, 9: FAIL })), { code: 1, why: 'rows 3, 9 failed' });
  assert.deepEqual(exitOf(rows({ 2: PASS }), [2]), { code: 0, why: 'no row failed and rows 2 passed' });
  assert.deepEqual(exitOf(rows({ 2: PASS, 3: FAIL }), [2, 3, 4]), { code: 1, why: 'row 3 failed; required and not passed: 4' });
});
