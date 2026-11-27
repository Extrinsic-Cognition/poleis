// The prepared road, played in Node: a set from tools/warm-launch/mainnet.mjs, a Keystone (a test key) that signs what this page asks, the checks
// that refuse a placeholder, another Keystone and a message that is not the one the set describes.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash, createPublicKey, generateKeyPairSync, sign, verify } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { cborDecode, cborEncode, tagged, urEncode, URDecoder } from './ur.mjs';
import { readSignature, signRequest } from './sol.mjs';
import { b58encode } from './b58.mjs';
import { layout, readSet, signedWire, toSign } from './prepared.mjs';
import { emit } from '../../../tools/warm-launch/mainnet.mjs';

const sha = async (u8) => new Uint8Array(createHash('sha256').update(u8).digest());
const keystone = () => {
  const { privateKey, publicKey } = generateKeyPairSync('ed25519');
  return { privateKey, address: b58encode(publicKey.export({ format: 'der', type: 'spki' }).subarray(12)) };
};
const params = (launchKey) => ({
  cluster: 'mainnet', launchKey, treasuryOwner: 'SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf',
  devWallet: 'Stake11111111111111111111111111111111111111', seed: 'swarm', name: 'SWARM', symbol: '$WARM', uri: 'https://example.org/swarm.json',
  metadataMutable: true, decimals: 9, supply: '1000000000', treasuryBps: 9000, saleBps: 1000, devShareBps: 5000, startPrice: '0.00000003', windowRatio: '2',
  binStep: 100, feeBps: 100, opening: '2026-10-30T20:00:00Z', close: '2026-11-15T00:00:00Z', trancheHours: 24, moneyAfterTranches: [4, 8, 12, 15], priorityMicroLamports: 10000,
});

test('a set with his numbers is signed by its Keystone by QR, the blockhash stamped fresh, and the signature verifies over what was asked', async () => {
  const k = keystone();
  const set = await readSet(JSON.stringify(emit(params(k.address))), sha);
  assert.equal(set.signable, true);
  assert.equal(set.txs.length, 5);
  const blockhash = b58encode(Uint8Array.from({ length: 32 }, (_, i) => i + 7));
  for (const tx of set.txs) {
    const message = toSign(set, tx, k.address, blockhash);
    const { keys, blockhashAt } = layout(message);
    assert.equal(keys[0], k.address);
    assert.equal(b58encode(message.subarray(blockhashAt, blockhashAt + 32)), blockhash);
    // the page asks; the Keystone reads the request, signs exactly its message, answers
    const requestId = Uint8Array.from({ length: 16 }, (_, i) => i);
    const d = new URDecoder();
    for (const p of signRequest({ message, path: "m/44'/501'/0'/0'", xfp: 'deadbeef', requestId }).parts) d.receive(p.toUpperCase());
    const req = cborDecode(d.result);
    const signature = new Uint8Array(sign(null, Buffer.from(req.get(2)), k.privateKey));
    const back = new URDecoder();
    back.receive(urEncode('sol-signature', cborEncode(new Map([[1, tagged(37, requestId)], [2, signature]]))).toUpperCase());
    const wire = signedWire(message, readSignature(back.result).signature);
    assert.equal(wire[0], 1);
    assert.ok(wire.length <= 1232, `${tx.name}: ${wire.length} bytes`);
    assert.ok(verify(null, Buffer.from(wire.subarray(65)), createPublicKey({ key: Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), Buffer.from(createPublicKey(k.privateKey).export({ format: 'der', type: 'spki' }).subarray(12))]), format: 'der', type: 'spki' }), Buffer.from(wire.subarray(1, 65))));
  }
});

test('a set that still holds placeholders is read and shown, and refused for signing', async () => {
  const raw = params('PLACEHOLDER: his Keystone');
  raw.supply = 'PLACEHOLDER: S';
  const set = await readSet(emit(raw), sha);
  assert.equal(set.signable, false);
  assert.deepEqual(set.placeholders.sort(), ['launchKey', 'supply']);
  assert.throws(() => toSign(set, set.txs[0], set.txs[0].signer, '11111111111111111111111111111111'), /placeholders/);
});

test('another Keystone, a message changed after it was described, or a filled blockhash are refused', async () => {
  const k = keystone(), other = keystone();
  const json = emit(params(k.address));
  const set = await readSet(json, sha);
  assert.throws(() => toSign(set, set.txs[0], other.address, '11111111111111111111111111111111'), /not by the Keystone/);
  const tampered = structuredClone(json);
  const m = Buffer.from(tampered.transactions[0].message, 'base64'); m[m.length - 1] ^= 1; tampered.transactions[0].message = m.toString('base64');
  await assert.rejects(readSet(tampered, sha), /not the one the set describes/);
  const stamped = structuredClone(json);
  const s = Buffer.from(stamped.transactions[1].message, 'base64'); s[layout(s).blockhashAt] = 9; stamped.transactions[1].message = s.toString('base64');
  await assert.rejects(readSet(stamped, sha), /blockhash is not blank/);
});

test('the page asks a Keystone by the path and fingerprint it read from that Keystone, never by the set', () => {
  const page = readFileSync(new URL('./keystone.mjs', import.meta.url), 'utf8');
  assert.ok(page.includes('signRequest({ message, path: k.path, xfp: k.xfp, requestId })'));
  assert.doesNotMatch(page, /signer\.(path|xfp)/);
});

test("a seat owner's vote set of a local chain is read by that chain's own genesis, and a set of an act carries the act's hash in what the Keystone shows", async () => {
  const { set } = JSON.parse(readFileSync(new URL('../../press/tests/prepared-vector.json', import.meta.url), 'utf8'));
  const local = { ...set, cluster: 'localnet', genesisHash: b58encode(Uint8Array.from({ length: 32 }, (_, i) => i + 1)) };
  const read = await readSet(local, sha);
  assert.deepEqual([read.genesis, (await readSet(set, sha)).genesis], [local.genesisHash, '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d']);
  assert.ok(read.txs.every((t) => t.shown.length === 1 && t.shown[0].program === read.txs[0].shown[0].program && t.shown[0].data.length > 40));
  await assert.rejects(readSet({ ...local, genesisHash: undefined }, sha), /localnet with its genesis hash/);
  await assert.rejects(readSet({ ...local, genesisHash: set.genesisHash }, sha), /public cluster/);
  await assert.rejects(readSet({ ...set, act: { ...set.act, hash: 'ab'.repeat(32) } }, sha), /do not carry the hash of act 1/);
});
