// The Keystone's side, played in Node: a device that exports accounts and signs a request, read back by sol.mjs.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createPrivateKey, createPublicKey, generateKeyPairSync, sign, verify } from 'node:crypto';
import { cborDecode, cborEncode, tagged, urEncode, URDecoder } from './ur.mjs';
import { readAccounts, readSignature, signRequest, toHex } from './sol.mjs';
import { b58encode } from './b58.mjs';

const fresh = () => {
  const { privateKey, publicKey } = generateKeyPairSync('ed25519');
  return { privateKey, pub: publicKey.export({ format: 'der', type: 'spki' }).subarray(12) };
};
const keypath = (comps, xfp) => tagged(304, new Map([[1, comps], ...(xfp === undefined ? [] : [[2, xfp]])]));

test('a Keystone account export reads as Solana addresses with their paths', () => {
  const a = fresh(), b = fresh();
  const hd = (pub, comps) => tagged(303, new Map([[3, pub], [6, keypath(comps)]]));
  const exportCbor = cborEncode(new Map([[1, 0xdeadbeef], [2, [hd(a.pub, [44, true, 501, true, 0, true, 0, true]), hd(Uint8Array.of(0, ...b.pub), [44, true, 501, true, 1, true, 0, true]), hd(new Uint8Array(65), [44, true, 60, true])]], [3, 'Keystone 3 Pro']]));
  // as the Keystone shows it: animated when long
  const d = new URDecoder();
  d.receive(urEncode('crypto-multi-accounts', exportCbor).toUpperCase());
  const r = readAccounts(d.result);
  assert.equal(r.xfp, 'deadbeef');
  assert.equal(r.device, 'Keystone 3 Pro');
  assert.deepEqual(r.accounts.map((x) => [x.address, x.path]), [[b58encode(a.pub), "m/44'/501'/0'/0'"], [b58encode(b.pub), "m/44'/501'/1'/0'"]]);
});

test('a sign request carries the message, the path and the fingerprint; the answer signs exactly that message', () => {
  const k = fresh();
  const message = Uint8Array.from({ length: 230 }, (_, i) => (i * 31) & 255);
  const requestId = Uint8Array.from({ length: 16 }, (_, i) => i + 1);
  const { parts } = signRequest({ message, path: "m/44'/501'/0'/0'", xfp: 'deadbeef', requestId });
  assert.equal(parts.length, 2);
  // the device reads the request
  const d = new URDecoder();
  for (const p of parts) d.receive(p.toUpperCase());
  assert.equal(d.type, 'sol-sign-request');
  const req = cborDecode(d.result);
  assert.deepEqual([...req.keys()], [1, 2, 3, 5, 6]);
  assert.equal(req.get(1).tag, 37);
  assert.deepEqual(req.get(2), message);
  assert.equal(req.get(3).tag, 304);
  assert.deepEqual(req.get(3).value.get(1), [44, true, 501, true, 0, true, 0, true]);
  assert.equal(req.get(3).value.get(2), 0xdeadbeef);
  assert.equal(req.get(6), 1);
  // and answers
  const signature = new Uint8Array(sign(null, Buffer.from(req.get(2)), k.privateKey));
  const answer = urEncode('sol-signature', cborEncode(new Map([[1, tagged(37, req.get(1).value)], [2, signature]])));
  const back = new URDecoder();
  back.receive(answer.toUpperCase());
  const r = readSignature(back.result);
  assert.equal(r.requestId, toHex(requestId));
  assert.ok(verify(null, Buffer.from(message), createPublicKey({ key: Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), k.pub]), format: 'der', type: 'spki' }), Buffer.from(r.signature)));
});

test('an answer without a 64-byte signature is refused', () => {
  assert.throws(() => readSignature(cborEncode(new Map([[1, tagged(37, new Uint8Array(16))], [2, new Uint8Array(10)]]))), /64-byte/);
});

test('a single-key export reads too, its fingerprint from its own path', () => {
  const a = fresh();
  const hdkey = cborEncode(new Map([[3, a.pub], [6, keypath([44, true, 501, true, 2, true, 0, true], 0x01020304)], [9, 'Keystone']]));
  const r = readAccounts(hdkey, 'crypto-hdkey');
  assert.equal(r.xfp, '01020304');
  assert.deepEqual(r.accounts.map((x) => [x.address, x.path]), [[b58encode(a.pub), "m/44'/501'/2'/0'"]]);
});
