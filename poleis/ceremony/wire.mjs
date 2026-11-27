// The ceremony's wire, with no dependency: base58, Ed25519 keys in the Solana CLI's file shape, signatures by node:crypto,
// and the legacy transaction around a message the adapter compiled (poleis/client/examples/ceremony.rs).
import { createPrivateKey, createPublicKey, generateKeyPairSync, sign, verify } from 'node:crypto';

const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

export function b58encode(bytes) {
  let n = 0n;
  for (const b of bytes) n = n * 256n + BigInt(b);
  let s = '';
  while (n > 0n) { s = ALPHABET[Number(n % 58n)] + s; n /= 58n; }
  for (const b of bytes) { if (b !== 0) break; s = '1' + s; }
  return s;
}

export function b58decode(s) {
  let n = 0n;
  for (const c of s) {
    const i = ALPHABET.indexOf(c);
    if (i < 0) throw new Error(`not base58: ${c}`);
    n = n * 58n + BigInt(i);
  }
  const out = [];
  while (n > 0n) { out.unshift(Number(n % 256n)); n /= 256n; }
  for (const c of s) { if (c !== '1') break; out.unshift(0); }
  return Uint8Array.from(out);
}

// DER heads of an Ed25519 key: PKCS#8 before the 32-byte seed, SPKI before the 32-byte public key
const PKCS8 = Buffer.from('302e020100300506032b657004220420', 'hex');
const SPKI = Buffer.from('302a300506032b6570032100', 'hex');

const privateOf = (kp) => createPrivateKey({ key: Buffer.concat([PKCS8, Buffer.from(kp.subarray(0, 32))]), format: 'der', type: 'pkcs8' });

/** A fresh keypair as the CLI keeps it: 64 bytes, the seed then the public key. */
export function freshKeypair() {
  const { privateKey, publicKey } = generateKeyPairSync('ed25519');
  const seed = privateKey.export({ format: 'der', type: 'pkcs8' }).subarray(PKCS8.length);
  const pub = publicKey.export({ format: 'der', type: 'spki' }).subarray(SPKI.length);
  return checkKeypair(Uint8Array.from([...seed, ...pub]));
}

/** A keypair whose second half is not the public key of its first is refused. */
export function checkKeypair(kp) {
  if (!(kp instanceof Uint8Array) || kp.length !== 64) throw new Error('a keypair is 64 bytes');
  const pub = createPublicKey(privateOf(kp)).export({ format: 'der', type: 'spki' }).subarray(SPKI.length);
  if (!Buffer.from(pub).equals(Buffer.from(kp.subarray(32)))) throw new Error('a keypair whose public half does not match its seed');
  return kp;
}

export const publicOf = (kp) => b58encode(kp.subarray(32, 64));

export const signWith = (kp, message) => new Uint8Array(sign(null, Buffer.from(message), privateOf(kp)));

export function verifies(publicB58, message, signature) {
  const key = createPublicKey({ key: Buffer.concat([SPKI, Buffer.from(b58decode(publicB58))]), format: 'der', type: 'spki' });
  return verify(null, Buffer.from(message), key, Buffer.from(signature));
}

/** Solana's compact length: seven bits a byte, low first. */
export function shortvec(n) {
  const out = [];
  do { let b = n & 0x7f; n >>= 7; if (n) b |= 0x80; out.push(b); } while (n);
  return Uint8Array.from(out);
}

/** The signed legacy transaction: the signature count, the signatures in the message's signer order, the message. */
export function transaction(message, signatures) {
  return Uint8Array.from([...shortvec(signatures.length), ...signatures.flatMap((s) => [...s]), ...message]);
}

/** Sign a compiled message with the keypairs its signers name, in its order; a signer without a key is refused. */
export function signMessage(message, signers, keypairs) {
  const byKey = new Map(keypairs.map((kp) => [publicOf(kp), kp]));
  const sigs = signers.map((pk) => {
    const kp = byKey.get(pk);
    if (!kp) throw new Error(`no key for signer ${pk}`);
    return signWith(kp, message);
  });
  return { wire: transaction(message, sigs), signature: b58encode(sigs[0]) };
}
