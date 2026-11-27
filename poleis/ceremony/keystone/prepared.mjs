// A prepared set: transactions another tool built unsigned (tools/warm-launch/mainnet.mjs; the launch press's `press vote` and `press seat`), for a
// Keystone to sign one by one by QR on this page. Each carries its legacy message with a zero blockhash, its signer and the sha256 of that message.
// Before the QR is shown the page checks that the message is exactly the one the set describes, that a set of an act carries that act's hash in its
// bytes, and that the Keystone it is about to ask is its signer, then stamps a fresh blockhash (a blockhash lives about a minute). A set that still
// holds a placeholder is read and shown, never signed. A set of a local chain (a rehearsal) states that chain's own genesis hash, never a public one.
import { b58decode32, b58encode } from './b58.mjs';

const unb64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));
const b64 = (u8) => btoa(String.fromCharCode(...u8));
const hex = (b) => Array.from(b, (x) => x.toString(16).padStart(2, '0')).join('');
const shortvec = (n) => { const o = []; do { let b = n & 0x7f; n >>= 7; if (n) b |= 0x80; o.push(b); } while (n); return o; };

const PUBLIC = { mainnet: '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d', devnet: 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG' };
const BUDGET = 'ComputeBudget111111111111111111111111111111';

/** The message's account keys, where its blockhash sits (after the 3-byte header, the compact key count and 32 bytes a key), and its instructions
 *  as the Keystone shows one it has no words for: the program's address and the data in base58. */
export function layout(message) {
  let at = 3;
  const vec = () => { let n = 0, shift = 0; for (;;) { const b = message[at++]; n |= (b & 0x7f) << shift; if (!(b & 0x80)) return n; shift += 7; } };
  const keys = Array.from({ length: vec() }, () => b58encode(message.subarray(at, at += 32)));
  const blockhashAt = at;
  at += 32;
  const shown = Array.from({ length: vec() }, () => { const program = keys[message[at++]]; const accounts = vec(); at += accounts; const n = vec(); return { program, bytes: message.subarray(at, at += n) }; });
  return { signers: message[0], keys, blockhashAt, shown: shown.filter((x) => x.program !== BUDGET).map((x) => ({ ...x, data: b58encode(x.bytes) })) };
}

/** The genesis hash a set's chain answers with: a public cluster's own, or the one a local chain's set states for itself. */
function genesisOf(set) {
  if (Object.hasOwn(PUBLIC, set.cluster)) return PUBLIC[set.cluster];
  const g = set.genesisHash;
  if (set.cluster !== 'localnet' || typeof g !== 'string' || b58encode(b58decode32(g)) !== g) throw new Error('a prepared set names its cluster: mainnet, devnet, or localnet with its genesis hash');
  if (Object.values(PUBLIC).includes(g)) throw new Error("a set of a local chain states a public cluster's genesis hash");
  return g;
}

/** A set read strictly: its cluster, whether it may be signed, and each transaction's message checked against the hash the set states for it. */
export async function readSet(json, sha256 = async (u8) => new Uint8Array(await crypto.subtle.digest('SHA-256', u8))) {
  const set = typeof json === 'string' ? JSON.parse(json) : json;
  const genesis = genesisOf(set);
  const act = set.act ? { n: set.act.n, name: set.act.name, hash: String(set.act.hash) } : null;
  if (act && !/^[0-9a-f]{64}$/.test(act.hash)) throw new Error("a set of an act names that act's hash");
  if (!Array.isArray(set.transactions) || !set.transactions.length) throw new Error('a prepared set holds no transaction');
  const placeholders = [...(set.placeholders || [])];
  const txs = [];
  for (const t of set.transactions) {
    const message = unb64(t.message);
    const { signers, keys, blockhashAt, shown } = layout(message);
    if (act && !shown.some((x) => hex(x.bytes).includes(act.hash))) throw new Error(`${t.name}: its bytes do not carry the hash of act ${act.n}`);
    if (message.subarray(blockhashAt, blockhashAt + 32).some((b) => b !== 0)) throw new Error(`${t.name}: its blockhash is not blank`);
    if (hex(await sha256(message)) !== t.messageSha256) throw new Error(`${t.name}: its message is not the one the set describes`);
    if (signers !== 1 || t.signers?.length !== 1 || keys[0] !== t.signers[0]) throw new Error(`${t.name}: one signer, the fee payer, is all this page signs for`);
    placeholders.push(...(t.placeholders || []));
    txs.push({ ...t, message, signer: keys[0], shown });
  }
  return { cluster: set.cluster, genesis, act, what: set.what || '', signable: set.signable === true && placeholders.length === 0, placeholders: [...new Set(placeholders)], signer: set.signer || null, txs };
}

/** The message to sign: the set's own bytes with a fresh blockhash in its place. Refused for a set that may not be signed, or another Keystone. */
export function toSign(set, tx, keystoneAddress, blockhash) {
  if (!set.signable) throw new Error(`this set may not be signed: it still holds placeholders (${set.placeholders.join(', ')})`);
  if (tx.signer !== keystoneAddress) throw new Error(`this transaction is signed by ${tx.signer}, not by the Keystone ${keystoneAddress}`);
  const out = Uint8Array.from(tx.message);
  out.set(b58decode32(blockhash), layout(out).blockhashAt);
  return out;
}

/** The signed transaction, as the cluster takes it: one signature, then the message. */
export const signedWire = (message, signature) => Uint8Array.from([...shortvec(1), ...signature, ...message]);
export { b64, unb64 };
