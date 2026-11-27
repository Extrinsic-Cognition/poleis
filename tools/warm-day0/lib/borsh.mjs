// A cursor for Borsh data whose layout is not fixed (Metaplex metadata, the Token-2022 metadata extension): the caller reads
// the fields in the order the program's own struct declares them. Every read is bounds-checked; a short buffer throws.
import { b58 } from './web3.mjs';

export function cursor(buf, start = 0) {
  let o = start;
  const need = (n, what) => { if (o + n > buf.length) throw new Error(`borsh: ${what} needs ${n} bytes at ${o}, the data holds ${buf.length}`); };
  const c = {
    get offset() { return o; },
    u8() { need(1, 'u8'); return buf.readUInt8(o++); },
    bool() { return c.u8() !== 0; },
    u16() { need(2, 'u16'); const v = buf.readUInt16LE(o); o += 2; return v; },
    u32() { need(4, 'u32'); const v = buf.readUInt32LE(o); o += 4; return v; },
    u64() { need(8, 'u64'); const v = buf.readBigUInt64LE(o); o += 8; return v; },
    pubkey() { need(32, 'pubkey'); const v = b58(buf.subarray(o, o + 32)); o += 32; return v; },
    bytes(n) { need(n, 'bytes'); const v = Buffer.from(buf.subarray(o, o + n)); o += n; return v; },
    string() { const n = c.u32(); need(n, 'string'); const v = buf.subarray(o, o + n).toString('utf8'); o += n; return v; },
    option(read) { return c.u8() === 0 ? null : read(); },
    vec(read) { const n = c.u32(); const out = []; for (let i = 0; i < n; i++) out.push(read()); return out; },
  };
  return c;
}

/** Metaplex pads name, symbol and uri with NUL bytes to their maximum length; the padding is not part of the text. */
export function unpad(s) { return s.replace(/\0+$/, ''); }
