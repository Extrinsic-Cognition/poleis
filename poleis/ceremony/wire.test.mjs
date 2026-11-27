import test from 'node:test';
import assert from 'node:assert/strict';
import { b58decode, b58encode, checkKeypair, freshKeypair, publicOf, shortvec, signMessage, signWith, transaction, verifies } from './wire.mjs';

test('base58 keeps leading zeros and round-trips', () => {
  assert.equal(b58encode(new Uint8Array(32)), '11111111111111111111111111111111');
  assert.deepEqual(b58decode('11111111111111111111111111111111'), new Uint8Array(32));
  const k = Uint8Array.from({ length: 32 }, (_, i) => (i * 37 + 11) & 255);
  assert.deepEqual(b58decode(b58encode(k)), k);
});

test('the fund address of the devnet program decodes to the bytes lib.rs fixes', () => {
  // poleis/program/poleis_treasury/src/lib.rs FUND_ADDRESS, the ["fund"] PDA of AJds…Tyab
  assert.equal(b58encode(Uint8Array.from([
    0x64, 0xaf, 0xda, 0x4a, 0x15, 0x0c, 0xf7, 0xd3, 0x62, 0xfb, 0xb3, 0x16, 0xed, 0x05, 0xaa, 0x9d,
    0xce, 0x98, 0x2d, 0x0a, 0x53, 0xf1, 0x2d, 0x0f, 0x5a, 0x9e, 0x78, 0x47, 0xb5, 0xfd, 0x73, 0x9b,
  ])), '7n3J6fujkQdFQ4YQyoUG4E1XGto2DyWBzuvrJNrSqdcn');
});

test('a fresh keypair signs what its public half verifies, and a forged half is refused', () => {
  const kp = freshKeypair();
  const msg = new TextEncoder().encode('the seal');
  assert.ok(verifies(publicOf(kp), msg, signWith(kp, msg)));
  const forged = Uint8Array.from(kp); forged[40] ^= 1;
  assert.throws(() => checkKeypair(forged), /does not match/);
});

test('shortvec is seven bits a byte, low first', () => {
  assert.deepEqual(shortvec(0), Uint8Array.of(0));
  assert.deepEqual(shortvec(127), Uint8Array.of(0x7f));
  assert.deepEqual(shortvec(128), Uint8Array.of(0x80, 0x01));
  assert.deepEqual(shortvec(16384), Uint8Array.of(0x80, 0x80, 0x01));
});

test('a transaction is the count, the signatures in the signer order, then the message', () => {
  const a = freshKeypair(), b = freshKeypair();
  const msg = Uint8Array.of(2, 0, 1, 9, 9);
  const { wire, signature } = signMessage(msg, [publicOf(b), publicOf(a)], [a, b]);
  assert.equal(wire.length, 1 + 2 * 64 + msg.length);
  assert.equal(wire[0], 2);
  assert.equal(signature, b58encode(signWith(b, msg)));
  assert.deepEqual(wire.subarray(1 + 128), msg);
  assert.deepEqual(transaction(msg, []), Uint8Array.of(0, ...msg));
  assert.throws(() => signMessage(msg, [publicOf(freshKeypair())], [a]), /no key for signer/);
});
