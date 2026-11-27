// The Keystone's Solana words, from its open registry (KeystoneHQ/ur-registry and keystone-sdk-base, ur-registry-sol):
//   crypto-multi-accounts (1103): { 1: master fingerprint, 2: [crypto-hdkey (303) { 3: key, 6: crypto-keypath (304) }], 3: device }
//   sol-sign-request (1101): { 1: uuid (37), 2: the message to sign, 3: crypto-keypath (304) { 1: [index, hardened, ...], 2: fingerprint },
//                             5: origin, 6: 1 for a transaction }
//   sol-signature (1102): { 1: uuid (37), 2: the 64-byte signature }
import { cborDecode, cborEncode, tagged, urParts } from './ur.mjs';
import { b58encode } from './b58.mjs';

const untag = (v, tag) => (v && typeof v === 'object' && 'tag' in v ? (tag === undefined || v.tag === tag ? v.value : null) : v);
const hex = (b) => Array.from(b, (x) => x.toString(16).padStart(2, '0')).join('');

function pathOf(keypath) {
  const comps = keypath.get(1) || [];
  const parts = [];
  for (let i = 0; i < comps.length; i += 2) parts.push(`${comps[i]}${comps[i + 1] ? "'" : ''}`);
  return `m/${parts.join('/')}`;
}

/** The accounts a Keystone shows when connecting a software wallet: each its Solana address, its path, the device's fingerprint.
 *  A crypto-multi-accounts export carries many; a crypto-hdkey export carries one, its fingerprint in its own key path. */
export function readAccounts(cbor, type = 'crypto-multi-accounts') {
  const m = cborDecode(cbor);
  if (type === 'crypto-hdkey') {
    const origin = untag(m.get(6), 304);
    const fp = origin instanceof Map ? origin.get(2) >>> 0 : 0;
    return readAccounts(cborEncode(new Map([[1, fp], [2, [tagged(303, m)]], ...(m.get(9) ? [[3, m.get(9)]] : [])])));
  }
  const xfp = m.get(1) >>> 0;
  const accounts = [];
  for (const k of m.get(2) || []) {
    const hd = untag(k, 303);
    if (!(hd instanceof Map)) continue;
    let key = hd.get(3);
    if (!(key instanceof Uint8Array)) continue;
    if (key.length === 33 && key[0] === 0) key = key.subarray(1); // an ed25519 key some writers prefix with a zero byte
    if (key.length !== 32) continue; // not a Solana key
    const origin = untag(hd.get(6), 304);
    accounts.push({ address: b58encode(key), path: origin instanceof Map ? pathOf(origin) : null, name: hd.get(9) || null });
  }
  return { xfp: xfp.toString(16).padStart(8, '0'), device: m.get(3) || null, accounts };
}

/** A sign request for one Solana message: its UR parts (one QR, or an animated few) and its request id. */
export function signRequest({ message, path, xfp, requestId, origin = 'Poleis' }) {
  const comps = path.replace(/^m\//i, '').split('/').flatMap((p) => [parseInt(p, 10), p.endsWith("'")]);
  const keypath = new Map([[1, comps], [2, parseInt(xfp, 16) >>> 0]]);
  const map = new Map([[1, tagged(37, requestId)], [2, message], [3, tagged(304, keypath)], [5, origin], [6, 1]]);
  return { requestId, parts: urParts('sol-sign-request', cborEncode(map), 200) };
}

/** The Keystone's answer: the request it signed and the signature. */
export function readSignature(cbor) {
  const m = cborDecode(cbor);
  const requestId = untag(m.get(1), 37);
  const signature = m.get(2);
  if (!(signature instanceof Uint8Array) || signature.length !== 64) throw new Error('the Keystone answer carries no 64-byte signature');
  return { requestId: requestId instanceof Uint8Array ? hex(requestId) : null, signature };
}

export { hex as toHex };
