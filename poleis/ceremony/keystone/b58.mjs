// Base58, as Solana writes addresses and blockhashes. No dependency; the page and the tests share it.
const A = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
export function b58encode(bytes) {
  let n = 0n; for (const b of bytes) n = n * 256n + BigInt(b);
  let s = ''; while (n > 0n) { s = A[Number(n % 58n)] + s; n /= 58n; }
  for (const b of bytes) { if (b !== 0) break; s = '1' + s; }
  return s;
}
/** A 32-byte key or hash; anything else is refused. */
export function b58decode32(s) {
  let n = 0n; for (const c of s) { const i = A.indexOf(c); if (i < 0) throw new Error(`not base58: ${s}`); n = n * 58n + BigInt(i); }
  const out = new Uint8Array(32);
  for (let i = 31; i >= 0; i--) { out[i] = Number(n & 255n); n >>= 8n; }
  if (n !== 0n) throw new Error(`not 32 bytes: ${s}`);
  return out;
}
