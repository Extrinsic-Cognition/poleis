// tools/solana-pda.mjs · Pure zero-dependency Solana Base58 + Ed25519 curve + PDA derivation.
import { createHash } from 'node:crypto';

const B58 = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

export function base58Bytes(s) {
  if (typeof s !== 'string' || !s) return null;
  let n = 0n;
  for (const ch of s) { const d = B58.indexOf(ch); if (d < 0) return null; n = n * 58n + BigInt(d); }
  const out = [];
  while (n > 0n) { out.unshift(Number(n & 255n)); n >>= 8n; }
  for (const ch of s) { if (ch !== '1') break; out.unshift(0); }
  return Uint8Array.from(out);
}

export function base58Of(bytes) {
  let n = 0n;
  for (const b of bytes) n = (n << 8n) | BigInt(b);
  let s = '';
  while (n > 0n) { s = B58[Number(n % 58n)] + s; n /= 58n; }
  for (const b of bytes) { if (b !== 0) break; s = '1' + s; }
  return s;
}

const bytesOf = (a) => {
  const b = typeof a === 'string' ? base58Bytes(a) : a;
  if (!b || b.length !== 32) throw new Error(`not an address: ${typeof a === 'string' ? a : '(bytes)'}`);
  return Buffer.from(b);
};

const P = 2n ** 255n - 19n;
const md = (a) => ((a % P) + P) % P;
const pw = (b, e) => { let r = 1n; b = md(b); while (e > 0n) { if (e & 1n) r = md(r * b); b = md(b * b); e >>= 1n; } return r; };
const D = md(-121665n * pw(121666n, P - 2n));

export function onCurve(bytes) {
  let y = 0n;
  for (let i = 31; i >= 0; i--) y = (y << 8n) | BigInt(i === 31 ? bytes[i] & 0x7f : bytes[i]);
  y = md(y);
  const y2 = md(y * y), u = md(y2 - 1n), v = md(D * y2 + 1n);
  const x = md(u * pw(v, 3n) * pw(md(u * pw(v, 7n)), (P - 5n) / 8n));
  const vx2 = md(v * x * x);
  return vx2 === u || vx2 === md(-u);
}

const sha = (...parts) => createHash('sha256').update(Buffer.concat(parts.map((p) => Buffer.from(p)))).digest();
export const createWithSeed = (base, seed, owner) => sha(bytesOf(base), Buffer.from(seed, 'utf8'), bytesOf(owner));

export function findPda(seeds, program) {
  for (let bump = 255; bump >= 0; bump--) {
    const h = sha(...seeds, Uint8Array.of(bump), bytesOf(program), Buffer.from('ProgramDerivedAddress'));
    if (!onCurve(h)) return [h, bump];
  }
  throw new Error('no program address');
}
