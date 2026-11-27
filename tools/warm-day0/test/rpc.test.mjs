// The road to a cluster: read-only by construction, and silent in a test run.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { createRpc, httpTransport, fixtureTransport, recordingTransport, callKey, READ_ONLY_METHODS, RpcError } from '../lib/rpc.mjs';
import { noNetwork, root, fixture } from './helpers.mjs';

noNetwork();

test('a method that could write, sign or simulate is refused before any transport is asked', async () => {
  let asked = 0;
  const rpc = createRpc(async () => { asked++; return null; });
  for (const method of ['sendTransaction', 'simulateTransaction', 'requestAirdrop', 'sendRawTransaction', 'getLatestBlockhash', 'getBalance', 'getFeeForMessage']) {
    await assert.rejects(rpc.call(method, []), e => e instanceof RpcError && e.code === 'REFUSED', method);
  }
  assert.equal(asked, 0);
  assert.equal(rpc.stats.calls, 0);
  assert.deepEqual([...READ_ONLY_METHODS].sort(), ['getAccountInfo', 'getBlockTime', 'getGenesisHash', 'getMultipleAccounts', 'getProgramAccounts', 'getSignaturesForAddress', 'getSlot', 'getTokenAccountsByOwner', 'getTokenLargestAccounts', 'getTransaction']);
  assert.ok(READ_ONLY_METHODS.every(m => m.startsWith('get')));
});

test('a fixture answers only what it recorded, and says so loudly otherwise', async () => {
  const sink = {};
  const live = async (method, params) => { if (method === 'getSlot') return 42; throw new RpcError('nope', { method, code: -32000 }); };
  const rec = createRpc(recordingTransport(live, sink));
  assert.equal(await rec.call('getSlot', [{ commitment: 'finalized' }]), 42);
  await assert.rejects(rec.call('getBlockTime', [42]), /nope/);
  const replay = createRpc(fixtureTransport({ calls: sink }));
  assert.equal(await replay.call('getSlot', [{ commitment: 'finalized' }]), 42);
  await assert.rejects(replay.call('getBlockTime', [42]), e => e.code === -32000, 'a recorded error is thrown again');
  await assert.rejects(replay.call('getSlot', [{ commitment: 'confirmed' }]), e => e.code === 'NOT_RECORDED');
  assert.equal(callKey('getSlot', [{ b: 1, a: [{ d: 1, c: 2 }] }]), callKey('getSlot', [{ a: [{ c: 2, d: 1 }], b: 1 }]), 'the key does not depend on the order of an object\'s members');
});

test('the clock is the cluster\'s: a skipped slot is stepped over, a missing recording is not', async () => {
  const calls = { [callKey('getSlot', [{ commitment: 'finalized' }])]: 100, [callKey('getBlockTime', [100])]: { __rpcError: { message: 'slot skipped', code: -32009 } }, [callKey('getBlockTime', [99])]: 1791865000 };
  assert.deepEqual(await createRpc(fixtureTransport({ calls })).clock(), { slot: 100, time: 1791865000 });
  delete calls[callKey('getBlockTime', [99])];
  await assert.rejects(createRpc(fixtureTransport({ calls })).clock(), e => e.code === 'NOT_RECORDED');
});

test('the live transport waits out a rate limit, and does not wait for a method the endpoint does not serve', async () => {
  const seen = [];
  const answers = [{ status: 429, headers: {} }, { status: 200, headers: {}, body: { jsonrpc: '2.0', id: 2, result: 'ok' } }];
  const fakeFetch = async (url, init) => { const a = answers.shift(); seen.push(JSON.parse(init.body).method); return { status: a.status, headers: { get: k => a.headers[k] ?? null }, json: async () => a.body }; };
  const quick = { fetchImpl: fakeFetch, gapMs: 0, retries: 2 };
  // 429 with a retry-after of a millisecond, then the answer
  answers[0].headers['retry-after'] = '0.001';
  assert.equal(await httpTransport('https://example.invalid', quick)('getSlot', []), 'ok');
  assert.deepEqual(seen, ['getSlot', 'getSlot']);
  // limit 0 for the method: one ask, no retry
  seen.length = 0; answers.push({ status: 429, headers: { 'x-ratelimit-method-limit': '0' } });
  await assert.rejects(httpTransport('https://example.invalid', quick)('getTokenLargestAccounts', []), e => e.code === 'METHOD_DISABLED');
  assert.equal(seen.length, 1);
  // a JSON-RPC error is an RpcError carrying its code
  answers.push({ status: 200, headers: {}, body: { jsonrpc: '2.0', id: 1, error: { code: -32602, message: 'bad params' } } });
  await assert.rejects(httpTransport('https://example.invalid', quick)('getSlot', []), e => e instanceof RpcError && e.code === -32602);
});

test('the public endpoint really does not serve getTokenLargestAccounts: the recording holds its refusal and the fallback\'s answer', () => {
  const calls = fixture('sra').calls;
  const refused = Object.entries(calls).find(([k]) => k.startsWith('getTokenLargestAccounts '));
  assert.equal(refused[1].__rpcError.code, 'METHOD_DISABLED');
  assert.ok(Object.keys(calls).some(k => k.startsWith('getProgramAccounts ["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"')));
});

// The tool's own source, comments removed: what it must never contain, and the one place it may write a file.
const sources = ['check.mjs', ...readdirSync(join(root, 'lib')).map(f => join('lib', f))].filter(f => f.endsWith('.mjs'));
const code = file => readFileSync(join(root, file), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:'"`\\])\/\/.*$/gm, '$1');

test('no source file can hold a key, build a transaction or write anywhere but --record', () => {
  assert.ok(sources.length >= 12);
  const forbidden = /\b(Keypair|fromSecretKey|fromSeed|secretKey|sendAndConfirm\w*|sendTransaction|sendRawTransaction|simulateTransaction|requestAirdrop|signTransaction|signAllTransactions|VersionedTransaction|TransactionInstruction|TransactionMessage|SystemProgram|nacl|child_process|execSync|spawn)\b|new Transaction\b|\.sign\(/;
  for (const file of sources) { const hit = code(file).match(forbidden); assert.equal(hit, null, `${file} contains ${hit && hit[0]}`); }
  const writers = sources.filter(f => /\b(writeFile\w*|appendFile\w*|createWriteStream|mkdir\w*|rm\w*Sync|unlink\w*|rename\w*)\b/.test(code(f)));
  assert.deepEqual(writers, ['check.mjs'], 'only the CLI writes, and only its --record file');
  assert.equal((code('check.mjs').match(/writeFileSync\(/g) || []).length, 1);
  assert.match(code('check.mjs'), /writeFileSync\(opts\.record,/);
  const fetchers = sources.filter(f => /\bfetch\b/.test(code(f)));
  assert.deepEqual(fetchers, [join('lib', 'rpc.mjs')], 'only the RPC layer reaches a network');
  const web3 = sources.filter(f => /@solana\/web3\.js/.test(code(f)));
  assert.deepEqual(web3, [join('lib', 'web3.mjs')], 'web3.js enters through one file');
  assert.match(code(join('lib', 'web3.mjs')), /PublicKey: web3\.PublicKey/);
  assert.doesNotMatch(code(join('lib', 'web3.mjs')), /web3\.(Connection|Keypair|Transaction)/);
});
