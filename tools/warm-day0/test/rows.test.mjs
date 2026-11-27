// The fourteen rows, end to end, over answers recorded once from mainnet. Nothing here reaches a network.
import test from 'node:test';
import assert from 'node:assert/strict';
import { main, parseArgs } from '../check.mjs';
import { runChecks, PASS, FAIL, UNCHECKED, CLAIMS } from '../lib/rows.mjs';
import { parseManifest } from '../lib/manifest.mjs';
import { createRpc, fixtureTransport, RpcError } from '../lib/rpc.mjs';
import { metadataAddress } from '../lib/metaplex.mjs';
import { noNetwork, fixture, fixturePath } from './helpers.mjs';

noNetwork();
const noDns = async () => { throw new Error('a test tried to reach DNS'); };
const replay = async (name, extra = []) => { const fx = fixture(name); return main([fx.meta.mint, '--replay', fixturePath(name), ...extra], { resolveTxt: noDns }); };
const verdicts = result => Object.fromEntries(result.rows.map(r => [r.n, r.verdict]));
const only = (result, expected) => { const all = Object.fromEntries(Array.from({ length: 14 }, (_, i) => [i + 1, UNCHECKED])); assert.deepEqual(verdicts(result), { ...all, ...expected }); };

test('a token with both authorities gone and an immutable name: rows 2, 3 and 4 pass, exit 0', async () => {
  const { code, stdout, stderr, result } = await replay('wif');
  assert.equal(stderr, ''); assert.equal(code, 0);
  only(result, { 2: PASS, 3: PASS, 4: PASS });
  assert.deepEqual(result.summary, { pass: 3, fail: 0, notCheckable: 11 });
  assert.deepEqual(result.rows.map(r => r.n), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
  assert.deepEqual(result.rows.map(r => r.claim), Object.values(CLAIMS));
  assert.deepEqual(result.rows[1].source, ['EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm']);
  assert.match(result.rows[1].detail[0], /Mint authority: none/);
  assert.match(result.rows[3].detail.join(' '), /immutable.*dogwifhat/);
  assert.equal(result.cost.rpcCalls, 7);
  // the table: a header, fourteen numbered rows, each with one of the three verdicts and where it was read
  const table = stdout.split('\n').filter(l => /^\s?\d{1,2}  (pass|fail|not checkable)\s/.test(l));
  assert.equal(table.length, 14);
  assert.match(table[1], /^ 2  pass\s+Fixed supply\s+EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm$/);
  assert.match(table[5], /^ 6  not checkable\s+No hidden tranche\s+-$/);
  assert.match(stdout, /Exit 0: no decidable row failed\.$/);
  for (const n of [1, 6, 13, 14]) assert.ok(result.rows[n - 1].detail[0].length > 40, `row ${n} says what a person looks at`);
});

test('--json prints the same result, every amount a string', async () => {
  const { code, stdout } = await replay('sra', ['--json']);
  const json = JSON.parse(stdout);
  assert.equal(code, 1); assert.equal(json.exit, 1);
  assert.equal(json.rows.length, 14);
  assert.deepEqual(json.rows.map(r => r.verdict).filter(v => ![PASS, FAIL, UNCHECKED].includes(v)), []);
  assert.equal(json.supply, '999911027513532');
  assert.equal(json.rows[4].data.gap, '30464407347432');
  assert.equal(json.tool, 'warm-day0'); assert.equal(json.cluster, 'mainnet'); assert.equal(typeof json.slot, 'number');
});

test('a live mint authority and a live freeze authority fail rows 2 and 3 by name; a stream its sender can cancel fails row 7', async () => {
  const { code, result } = await replay('usdc-stream');
  assert.equal(code, 1);
  only(result, { 2: FAIL, 3: FAIL, 4: FAIL, 7: FAIL });
  assert.match(result.rows[1].detail[0], /BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG/);
  assert.match(result.rows[2].detail[0], /7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar/);
  assert.match(result.rows[3].detail[0], /mutable/);
  const lock = result.rows[6].data.locks[0];
  assert.deepEqual([lock.kind, lock.ok, lock.cancelAuthority, lock.recipientAuthority], ['streamflow', false, 'the sender', 'the sender']);
  assert.match(result.rows[6].detail.join(' '), /the sender can cancel it; the sender can change its recipient/);
  assert.equal(result.exitWhy, 'rows 2, 3, 4, 7 failed');
});

test('Token-2022: a permanent delegate fails row 3, metadata an authority can rewrite fails row 4, a domain that does not name the mint fails row 1', async () => {
  const { code, result } = await replay('pyusd');
  assert.equal(code, 1);
  only(result, { 1: FAIL, 2: FAIL, 3: FAIL, 4: FAIL });
  assert.match(result.rows[0].detail[0], /1 TXT record on optimism\.fit, none names this mint/);
  assert.deepEqual(result.rows[0].source, ['dns:TXT optimism.fit']);
  const three = result.rows[2].detail.join(' ');
  assert.match(three, /Freeze authority: 2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk/);
  assert.match(three, /PermanentDelegate: 2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk can move or burn/);
  assert.match(three, /TransferHook: 2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk can set a program/);
  assert.match(result.rows[2].notes.join(' '), /TransferFeeConfig: 0 bps.*not read: ConfidentialTransferMint, ConfidentialTransferFeeConfig/);
  assert.match(result.rows[3].detail.join(' '), /Token-2022 metadata in the mint.*MetadataPointer.*PayPal USD/);
  assert.equal(result.tokenProgram, 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb');
});

test('locks: Jupiter Lock with no authority passes, with a cancel authority fails, two sound Streamflow streams pass', async () => {
  const cndl = (await replay('cndl-lock')).result;
  only(cndl, { 2: PASS, 3: PASS, 4: FAIL, 7: FAIL });
  assert.match(cndl.rows[6].detail[1], /Fails because the creator or the recipient can cancel it; the creator or the recipient can change its recipient/);
  assert.match(cndl.rows[6].detail[1], /Starts 2026-03-01T12:00:00Z, cliff 2026-03-01T12:00:00Z, ends 2027-03-01T12:00:00Z/);
  const nest = (await replay('nest-streams')).result;
  only(nest, { 2: PASS, 3: PASS, 4: FAIL, 7: PASS });
  assert.equal(nest.rows[6].detail[0], '2 locks named, 2 sound.');
  assert.deepEqual(nest.rows[6].data.locks.map(l => [l.kind, l.ok, l.total, l.held]), [['streamflow', true, 55000000000000000n, 55000000000000000n], ['streamflow', true, 70000000000000000n, 70000000000000000n]]);
  assert.match(nest.rows[6].detail[1], /Starts 2027-10-30T21:34:07Z.*ends 2031-10-29T21:34:07Z.*Nothing has unlocked yet.*No cancel authority, no recipient-change authority/);
  assert.deepEqual(nest.programs.map(p => [p.key, p.moved]), [['spl-token', false], ['metaplex-token-metadata', false], ['streamflow', false]]);
});

test('a pool locked for good only in part fails row 9 for the named position; its linear fee schedule passes row 10; its holder is not the fund', async () => {
  const { result } = await replay('arena');
  only(result, { 2: PASS, 3: PASS, 4: FAIL, 7: PASS, 8: PASS, 9: FAIL, 10: PASS, 12: FAIL });
  assert.match(result.rows[6].detail[1], /ok, Jupiter Lock\. 900,000,000 \(90\.09%\).*150,000,000 every 2628000 s \(30\.4 days\), 6 times.*holds 900,000,000/);
  assert.match(result.rows[8].detail[0], /can withdraw 64\.52% of its liquidity today/);
  assert.match(result.rows[8].detail[1], /35\.47% is permanently locked/);
  assert.match(result.rows[9].detail[0], /time scheduler, linear\. Starts at 90% when the pool opens \(2026-09-17T14:31:28Z\) and falls to 0\.25% over 24 steps of 300 s \(5 minutes\), reaching it at 2026-09-17T16:31:28Z\. Now: 0\.25%/);
  assert.deepEqual([result.rows[9].data.start, result.rows[9].data.end, result.rows[9].data.periods], [900000000n, 2500008n, 24]);
  assert.match(result.rows[11].detail[0], /is held by 5M5ggeV4NET5NVENqTdCVauWGLuH2qgeoDYCmn1XkhSU, not the fund/);
  assert.match(result.rows[11].detail.at(-1), /1 claim in its 4 newest transactions, 0 to the fund/);
  assert.match(result.rows[7].notes.join(' '), /SetPoolStatus.*5 other DAMM v2 pools hold this mint/s);
  assert.match(result.rows[9].notes.join(' '), /UpdatePoolFees/);
});

test('Meteora\'s reference pool, emptied since: open, no liquidity left, and the schedule Meteora\'s own test states', async () => {
  const { result } = await replay('cudis-pool');
  only(result, { 2: PASS, 3: PASS, 4: FAIL, 8: PASS, 9: FAIL, 10: PASS });
  assert.equal(result.rows[8].detail[0], 'The pool holds no liquidity at all.');
  assert.deepEqual([result.rows[9].data.start, result.rows[9].data.periods, result.rows[9].data.frequency, result.rows[9].data.curve], [500000000n, 120, 60n, 'exponential']);
  assert.match(result.rows[9].detail[0], /Starts at 50%.*falls to 1\.9919% over 120 steps of 60 s,/);
  assert.match(result.rows[7].detail[0], /creator not recorded/);
  assert.match(result.rows[11].detail[0], /names no fund address/);
});

test('the full run: a pool locked for good, a decaying fee, and three real failures a stranger can verify', async () => {
  const { code, result } = await replay('sra');
  assert.equal(code, 1);
  only(result, { 2: PASS, 3: PASS, 4: FAIL, 5: FAIL, 8: PASS, 9: PASS, 10: PASS, 11: FAIL, 12: FAIL });
  // row 5: one labelled account, 146 others found by reading every holder, because the endpoint serves no largest-accounts
  const five = result.rows[4];
  assert.deepEqual([five.data.supply, five.data.labelled, five.data.gap, five.data.holdersRead], [999911027513532n, 969446620166100n, 30464407347432n, 'getProgramAccounts']);
  assert.match(five.detail.join(' '), /among all 147 accounts that hold the token: 146 unlabelled holders with 30,464,407\.347432 \(3\.04%\)/);
  assert.match(five.detail.join(' '), /35zT1xgDyVjk48DAEt5sN4C2HggckhPPn3oTy2RjhPXn 11,171,751\.406708 \(1\.11%\)/);
  assert.match(five.notes[0], /does not serve getTokenLargestAccounts/);
  // rows 8 to 10
  assert.match(result.rows[7].detail[0], /customizable, this mint against So11111111111111111111111111111111111111112, made by BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs\. Enabled: yes\. Open since 2026-05-13T02:57:55Z\. Whitelisted vault: none/);
  assert.match(result.rows[8].detail.join(' '), /Every named position is locked for good.*100\.00% is permanently locked/);
  assert.deepEqual([result.rows[9].data.start, result.rows[9].data.end, result.rows[9].data.now], [800000000n, 10014870n, 10014870n]);
  // row 11: the creator handed tokens to a dozen wallets 25 seconds after the pool opened
  const eleven = result.rows[10];
  assert.equal(eleven.data.flagged.length, 5);
  assert.ok(eleven.data.flagged.every(f => f.time === 1778641100 && f.wallet === 'BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs'));
  assert.equal(eleven.data.flagged.reduce((s, f) => s + f.to.length, 0), 16);
  assert.match(eleven.detail[0], /5 moved this token out of a house wallet: 0 on the published schedule, 5 off it/);
  assert.match(eleven.notes.join(' '), /no schedule, so every transfer out of a house wallet is flagged/);
  // row 12: the fund holds the position, and the one claim was emptied to another wallet in the same transaction
  const twelve = result.rows[11];
  assert.match(twelve.detail[0], /paid 467728913657 base units of token B into the fund's A8DxcpM8PbVPqq1xTDtkRaGN1LooLSrc8tKwvRCnZnME, closed in the same transaction with its balance sent to Boy7pkvoq3ipJ3qfxBzCFeA76x1segrZHEANHBX4XAoe/);
  assert.equal(twelve.data.positions[0].holderIsFund, true);
  assert.deepEqual(result.programs.map(p => p.key), ['spl-token', 'metaplex-token-metadata', 'meteora-damm-v2']);
  assert.equal(result.programs[2].upgradeAuthority, 'JADaUV8kvDpDbJr55wxXJHVaBS3VCj8thZZHjfeuCVLd');
  assert.equal(result.cost.rpcCalls, 35);
});

test('the exit code: 1 when a row fails, 1 when a required row did not pass, 2 when the run cannot be made', async () => {
  assert.equal((await replay('wif', ['--require', '2,3,4'])).code, 0);
  const strict = await replay('wif', ['--require', '2,5']);
  assert.equal(strict.code, 1);
  assert.equal(strict.result.exitWhy, 'required and not passed: row 5');
  assert.equal((await replay('sra', ['--require', '9,10'])).result.exitWhy, 'rows 4, 5, 11, 12 failed');
  for (const argv of [[], ['a', 'b'], ['EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm', '--cluster', 'testnet'], ['EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm', '--history', '0'], ['EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm', '--frobnicate'], ['EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm', '--domain', 'http://x/'], ['EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm', '--require', '15']]) {
    const r = await main(argv, { resolveTxt: noDns });
    assert.equal(r.code, 2, argv.join(' ')); assert.match(r.stderr, /usage: check\.mjs/);
  }
  const wrong = await main(['BoY2UJq7VJmyYzgszK9bzo6MFqQB4u5NEMhRupiDwJp4', '--replay', fixturePath('wif')], { resolveTxt: noDns });
  assert.equal(wrong.code, 2); assert.match(wrong.stderr, /the fixture is for EKpQ/);
  const notMint = await main(['4KWvNUCc5tuszhTGgb4YHnezbqhYe763dyGYNuZPEsqd', '--replay', fixturePath('wif')], { resolveTxt: noDns });
  assert.equal(notMint.code, 2);
  assert.equal((await main(['--help'])).code, 0);
  assert.deepEqual(parseArgs(['M', '--json', '--history', '9', '--require', '2,3']).require, [2, 3]);
});

test('a wrong cluster is refused by its genesis hash; an account that is not a mint stops the run', async () => {
  const fx = fixture('wif');
  const devnet = await main([fx.meta.mint, '--cluster', 'devnet'], { transport: fixtureTransport(fx), resolveTxt: noDns });
  assert.equal(devnet.code, 2); assert.match(devnet.stderr, /is not devnet: its genesis hash is 5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d/);
  // the metadata account is a real account and not a mint
  const sra = fixture('sra');
  await assert.rejects(runChecks({ rpc: createRpc(fixtureTransport(sra)), mint: 'AjtoZGtMWx7AZZBUFsuzhhYZL2MhF7xmVxH3MJVsuGj5' }), e => e.code === 'NOT_RECORDED' || e.name === 'UsageError');
});

test('a row never turns quiet when the recording has no answer: the run throws instead', async () => {
  const fx = fixture('sra');
  const manifest = parseManifest({ ...fx.manifest, locks: [{ label: 'not recorded', address: '77vFUtcMWqAtqDtojH3GABx2ZPHUpsKe9KkzFq45ASrW' }] }, fx.meta.mint);
  await assert.rejects(runChecks({ rpc: createRpc(fixtureTransport(fx)), mint: fx.meta.mint, manifest, history: fx.meta.history }), e => e.code === 'NOT_RECORDED');
});

test('a cluster that stops answering makes a row not checkable, never a pass', async () => {
  const fx = fixture('wif');
  const inner = fixtureTransport(fx), metadata = metadataAddress(fx.meta.mint);
  const flaky = async (method, params) => { if (method === 'getAccountInfo' && params[0] === metadata) throw new RpcError('getAccountInfo: HTTP 503 after 7 tries', { method, code: 503 }); return inner(method, params); };
  const result = await runChecks({ rpc: createRpc(flaky), mint: fx.meta.mint });
  assert.equal(result.rows[3].verdict, UNCHECKED);
  assert.match(result.rows[3].detail[0], /The cluster did not answer: getAccountInfo: HTTP 503/);
  assert.deepEqual(result.rows[3].source, [metadata]);
  assert.equal(result.rows[1].verdict, PASS);
});
