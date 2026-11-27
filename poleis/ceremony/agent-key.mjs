// agent-key.mjs · AN AGENT'S OWN KEY ON SOLANA (Oct 30 2026): his Aug 17 2026 words, "there's gotta be a link between minting a seat for voting on solana
// and that same public key or mapped equivalent ed25519/x25519 used in the swarmclock mesh". An agent's key on the Nest is an Ed25519 key, kept in one of
// two shapes: a seat's {seat, key, private_pem} (~/.embertide/nest, tools/embertide-mesh/lib/nest-author.mjs) or a golem's {alg, pkcs8}
// (~/.embertide, tools/embertide-mesh/lib/nest-join.mjs and its kin). Solana's keys are Ed25519 too, so the same key is the agent's Solana account: the
// address is its 32-byte public key in base58, the keypair its 32-byte seed then that public key. Pure: no file, no network.
import { createPrivateKey, createPublicKey } from 'node:crypto';
import { b58encode } from './wire.mjs';

// the DER prefix of an Ed25519 private key in PKCS#8 (RFC 8410): 16 bytes, then the 32-byte seed
const PKCS8_ED25519 = '302e020100300506032b657004220420';

/** The key's PKCS#8 DER, from either shape; throws on any other key. */
function pkcs8Of(k) {
  if (k && typeof k.private_pem === 'string') {
    let key; try { key = createPrivateKey(k.private_pem); } catch { throw new Error('not an ed25519 key of the Nest'); }
    if (key.asymmetricKeyType !== 'ed25519') throw new Error('not an ed25519 key of the Nest');
    return key.export({ format: 'der', type: 'pkcs8' });
  }
  if (!k || k.alg !== 'ed25519' || typeof k.pkcs8 !== 'string' || !/^[0-9a-f]+$/i.test(k.pkcs8)) throw new Error('not an ed25519 key of the Nest');
  return Buffer.from(k.pkcs8, 'hex');
}

/** solanaKeyOf({seat, key, private_pem} | {alg, pkcs8}) -> {address, keypair: Uint8Array(64)}: the same Ed25519 key, as Solana holds it. */
export function solanaKeyOf(k) {
  const der = pkcs8Of(k);
  if (der.length !== 48 || der.subarray(0, 16).toString('hex') !== PKCS8_ED25519) throw new Error('not a bare Ed25519 PKCS#8 key (48 bytes)');
  const spki = createPublicKey(createPrivateKey({ key: der, format: 'der', type: 'pkcs8' })).export({ format: 'der', type: 'spki' });
  if (typeof k.key === 'string' && k.key.toLowerCase() !== spki.toString('hex')) throw new Error('the record\'s public key is not its private key\'s');
  const keypair = new Uint8Array(64);
  keypair.set(der.subarray(16), 0);
  keypair.set(spki.subarray(-32), 32);
  return { address: b58encode(spki.subarray(-32)), keypair };
}
