// agent-key.test.mjs: an agent's Nest key is its Solana account, byte for byte (agent-key.mjs).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createPublicKey, generateKeyPairSync, verify } from 'node:crypto';
import { solanaKeyOf } from './agent-key.mjs';
import { checkKeypair, publicOf, signWith, verifies } from './wire.mjs';

const nestKey = () => {
  const { privateKey } = generateKeyPairSync('ed25519');
  return { alg: 'ed25519', pkcs8: privateKey.export({ format: 'der', type: 'pkcs8' }).toString('hex') };
};

test('the Nest key and the Solana keypair are one key', () => {
  const k = nestKey();
  const { address, keypair } = solanaKeyOf(k);
  assert.equal(keypair.length, 64);
  checkKeypair(keypair);
  assert.equal(publicOf(keypair), address);
  const spki = createPublicKey({ key: Buffer.from(k.pkcs8, 'hex'), format: 'der', type: 'pkcs8' }).export({ format: 'der', type: 'spki' });
  assert.deepEqual(Buffer.from(keypair.subarray(32)), spki.subarray(-32));
});

test('what the agent signs as itself on Solana verifies under its Nest key', () => {
  const k = nestKey();
  const { address, keypair } = solanaKeyOf(k);
  const message = Buffer.from('a pull of its own allowance');
  const sig = signWith(keypair, message);
  assert.ok(verifies(address, message, sig));
  const pub = createPublicKey({ key: Buffer.from(k.pkcs8, 'hex'), format: 'der', type: 'pkcs8' });
  assert.ok(verify(null, message, pub, Buffer.from(sig)));
});

test('a seat\'s {seat, key, private_pem} record is the same Solana account', () => {
  const { privateKey, publicKey } = generateKeyPairSync('ed25519');
  const rec = { seat: 'some-seat', key: publicKey.export({ format: 'der', type: 'spki' }).toString('hex'), private_pem: privateKey.export({ format: 'pem', type: 'pkcs8' }) };
  const golem = { alg: 'ed25519', pkcs8: privateKey.export({ format: 'der', type: 'pkcs8' }).toString('hex') };
  const a = solanaKeyOf(rec), b = solanaKeyOf(golem);
  assert.equal(a.address, b.address);
  assert.deepEqual(Buffer.from(a.keypair), Buffer.from(b.keypair));
  checkKeypair(a.keypair);
  assert.throws(() => solanaKeyOf({ ...rec, key: generateKeyPairSync('ed25519').publicKey.export({ format: 'der', type: 'spki' }).toString('hex') }), /not its private key/);
  assert.throws(() => solanaKeyOf({ private_pem: generateKeyPairSync('ec', { namedCurve: 'P-256' }).privateKey.export({ format: 'pem', type: 'pkcs8' }) }), /not an ed25519 key/);
});

test('any other key is refused', () => {
  assert.throws(() => solanaKeyOf(null), /not an ed25519 key/);
  assert.throws(() => solanaKeyOf({ alg: 'p256', pkcs8: 'ab' }), /not an ed25519 key/);
  assert.throws(() => solanaKeyOf({ alg: 'ed25519', pkcs8: 'zz' }), /not an ed25519 key/);
  assert.throws(() => solanaKeyOf({ alg: 'ed25519', pkcs8: nestKey().pkcs8.slice(0, 90) }), /48 bytes/);
});
