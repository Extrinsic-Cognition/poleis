// Uniform Resources, the QR language of air-gapped wallets (Blockchain Commons BCR-2020-005, -012, -006), with no dependency: the same
// module runs in the page (Brave) and in Node's tests. SHA-256, CRC-32, Bytewords (minimal), a CBOR subset, Xoshiro256**, and the
// fountain code that animated QRs carry. Ported line by line from the reference (ngraveio/bc-ur, Blockchain Commons' bc-ur) and checked
// against its published vectors (ur.test.mjs), so a Keystone and this page read each other.

// ---- SHA-256 (synchronous: the fountain seeds one per part) ----
const K256 = Uint32Array.from([
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
  0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
  0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
  0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
]);
export function sha256(data) {
  const bytes = data instanceof Uint8Array ? data : new TextEncoder().encode(data);
  const bitLen = bytes.length * 8;
  const padded = new Uint8Array(Math.ceil((bytes.length + 9) / 64) * 64);
  padded.set(bytes);
  padded[bytes.length] = 0x80;
  const dv = new DataView(padded.buffer);
  dv.setUint32(padded.length - 8, Math.floor(bitLen / 2 ** 32));
  dv.setUint32(padded.length - 4, bitLen >>> 0);
  const h = Uint32Array.from([0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]);
  const w = new Uint32Array(64);
  const rotr = (x, n) => (x >>> n) | (x << (32 - n));
  for (let off = 0; off < padded.length; off += 64) {
    for (let i = 0; i < 16; i++) w[i] = dv.getUint32(off + 4 * i);
    for (let i = 16; i < 64; i++) {
      const s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >>> 3);
      const s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >>> 10);
      w[i] = (w[i - 16] + s0 + w[i - 7] + s1) >>> 0;
    }
    let [a, b, c, d, e, f, g, hh] = h;
    for (let i = 0; i < 64; i++) {
      const t1 = (hh + (rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25)) + ((e & f) ^ (~e & g)) + K256[i] + w[i]) >>> 0;
      const t2 = ((rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22)) + ((a & b) ^ (a & c) ^ (b & c))) >>> 0;
      hh = g; g = f; f = e; e = (d + t1) >>> 0; d = c; c = b; b = a; a = (t1 + t2) >>> 0;
    }
    h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
  }
  const out = new Uint8Array(32);
  const ov = new DataView(out.buffer);
  for (let i = 0; i < 8; i++) ov.setUint32(4 * i, h[i]);
  return out;
}

// ---- CRC-32 (IEEE) ----
const CRC_TABLE = Uint32Array.from({ length: 256 }, (_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; return c >>> 0; });
export function crc32(bytes) {
  let c = 0xffffffff;
  for (const b of bytes) c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
const u32be = (n) => Uint8Array.of(n >>> 24, (n >>> 16) & 255, (n >>> 8) & 255, n & 255);
const concat = (parts) => { const out = new Uint8Array(parts.reduce((s, p) => s + p.length, 0)); let o = 0; for (const p of parts) { out.set(p, o); o += p.length; } return out; };

// ---- Bytewords, minimal style: each byte its word's first and last letter, a CRC-32 of the body appended ----
const WORDS = 'ableacidalsoapexaquaarchatomauntawayaxisbackbaldbarnbeltbetabiasbluebodybragbrewbulbbuzzcalmcashcatschefcityclawcodecolacookcostcruxcurlcuspcyandarkdatadaysdelidicedietdoordowndrawdropdrumdulldutyeacheasyechoedgeepicevenexamexiteyesfactfairfernfigsfilmfishfizzflapflewfluxfoxyfreefrogfuelfundgalagamegeargemsgiftgirlglowgoodgraygrimgurugushgyrohalfhanghardhawkheathelphighhillholyhopehornhutsicedideaidleinchinkyintoirisironitemjadejazzjoinjoltjowljudojugsjumpjunkjurykeepkenokeptkeyskickkilnkingkitekiwiknoblamblavalazyleaflegsliarlimplionlistlogoloudloveluaulucklungmainmanymathmazememomenumeowmildmintmissmonknailnavyneednewsnextnoonnotenumbobeyoboeomitonyxopenovalowlspaidpartpeckplaypluspoempoolposepuffpumapurrquadquizraceramprealredorichroadrockroofrubyruinrunsrustsafesagascarsetssilkskewslotsoapsolosongstubsurfswantacotasktaxitenttiedtimetinytoiltombtoystriptunatwinuglyundouniturgeuservastveryvetovialvibeviewvisavoidvowswallwandwarmwaspwavewaxywebswhatwhenwhizwolfworkyankyawnyellyogayurtzapszerozestzinczonezoom';
const word = (i) => WORDS.slice(4 * i, 4 * i + 4);
const MINIMAL = new Map(Array.from({ length: 256 }, (_, i) => [word(i)[0] + word(i)[3], i]));
export function bytewordsEncode(body) {
  const all = concat([body, u32be(crc32(body))]);
  return Array.from(all, (b) => word(b)[0] + word(b)[3]).join('');
}
export function bytewordsDecode(text) {
  const s = text.toLowerCase();
  if (s.length % 2 || s.length < 10) throw new Error('bytewords: bad length');
  const all = new Uint8Array(s.length / 2);
  for (let i = 0; i < all.length; i++) {
    const v = MINIMAL.get(s.slice(2 * i, 2 * i + 2));
    if (v === undefined) throw new Error(`bytewords: not a word at ${2 * i}`);
    all[i] = v;
  }
  const body = all.subarray(0, -4);
  const sum = all.subarray(-4);
  const want = u32be(crc32(body));
  if (!sum.every((b, i) => b === want[i])) throw new Error('bytewords: checksum mismatch');
  return body;
}

// ---- CBOR: unsigned and negative integers, byte and text strings, arrays, maps (a Map), tags ({ tag, value }), true/false/null ----
export const tagged = (tag, value) => ({ tag, value });
function head(major, n) {
  const m = major << 5;
  if (n < 24) return Uint8Array.of(m | n);
  if (n < 0x100) return Uint8Array.of(m | 24, n);
  if (n < 0x10000) return Uint8Array.of(m | 25, n >> 8, n & 255);
  if (n < 0x100000000) return concat([Uint8Array.of(m | 26), u32be(n)]);
  const hi = Math.floor(n / 0x100000000);
  return concat([Uint8Array.of(m | 27), u32be(hi), u32be(n >>> 0)]);
}
export function cborEncode(v) {
  if (typeof v === 'number') { if (!Number.isInteger(v) || v < 0) throw new Error('cbor: only unsigned integers'); return head(0, v); }
  if (typeof v === 'boolean') return Uint8Array.of(v ? 0xf5 : 0xf4);
  if (v === null) return Uint8Array.of(0xf6);
  if (v instanceof Uint8Array) return concat([head(2, v.length), v]);
  if (typeof v === 'string') { const b = new TextEncoder().encode(v); return concat([head(3, b.length), b]); }
  if (Array.isArray(v)) return concat([head(4, v.length), ...v.map(cborEncode)]);
  if (v instanceof Map) {
    // keys in ascending order: the canonical map, whatever order a reader expects
    const entries = [...v.entries()].sort(([a], [b]) => a - b);
    return concat([head(5, entries.length), ...entries.flatMap(([k, x]) => [cborEncode(k), cborEncode(x)])]);
  }
  if (v && typeof v === 'object' && 'tag' in v) return concat([head(6, v.tag), cborEncode(v.value)]);
  throw new Error(`cbor: cannot encode ${typeof v}`);
}
export function cborDecode(bytes) {
  let at = 0;
  const uint = (info) => {
    if (info < 24) return info;
    const n = { 24: 1, 25: 2, 26: 4, 27: 8 }[info];
    if (!n) throw new Error('cbor: indefinite or reserved length');
    let v = 0;
    for (let i = 0; i < n; i++) v = v * 256 + bytes[at++];
    return v;
  };
  const item = () => {
    if (at >= bytes.length) throw new Error('cbor: truncated');
    const b = bytes[at++];
    const major = b >> 5, info = b & 31;
    if (major === 7) {
      if (info === 20) return false;
      if (info === 21) return true;
      if (info === 22 || info === 23) return null;
      throw new Error('cbor: floats are not read here');
    }
    const n = uint(info);
    switch (major) {
      case 0: return n;
      case 1: return -1 - n;
      case 2: { const v = bytes.slice(at, at + n); at += n; return v; }
      case 3: { const v = new TextDecoder().decode(bytes.subarray(at, at + n)); at += n; return v; }
      case 4: return Array.from({ length: n }, item);
      case 5: { const m = new Map(); for (let i = 0; i < n; i++) { const k = item(); m.set(k, item()); } return m; }
      case 6: return tagged(n, item());
    }
    throw new Error('cbor: unknown major type');
  };
  const v = item();
  if (at !== bytes.length) throw new Error('cbor: trailing bytes');
  return v;
}

// ---- Xoshiro256**, seeded by the SHA-256 of its seed, as the reference ----
const M64 = (1n << 64n) - 1n;
const rotl = (x, k) => ((x << BigInt(k)) | (x >> BigInt(64 - k))) & M64;
export class Xoshiro {
  constructor(seed) {
    const d = sha256(seed);
    this.s = [0, 1, 2, 3].map((i) => d.subarray(8 * i, 8 * i + 8).reduce((v, b) => (v << 8n) | BigInt(b), 0n));
  }
  next() {
    const s = this.s;
    const result = (rotl((s[1] * 5n) & M64, 7) * 9n) & M64;
    const t = (s[1] << 17n) & M64;
    s[2] ^= s[0]; s[3] ^= s[1]; s[1] ^= s[2]; s[0] ^= s[3];
    s[2] ^= t; s[3] = rotl(s[3], 45);
    return result;
  }
  nextDouble() { return Number(this.next()) / Number(M64 + 1n); }
  nextInt(low, high) { return Math.floor(this.nextDouble() * (high - low + 1) + low); }
}

// ---- the fountain: which fragments a part carries ----
function aliasTable(p) {
  const n = p.length, sum = p.reduce((a, b) => a + b, 0);
  const scaled = p.map((x) => (x * n) / sum);
  const prob = new Array(n), alias = new Array(n), small = [], large = [];
  for (let i = n - 1; i >= 0; i--) (scaled[i] < 1 ? small : large).push(i);
  while (small.length && large.length) {
    const less = small.pop(), more = large.pop();
    prob[less] = scaled[less]; alias[less] = more;
    scaled[more] = scaled[more] + scaled[less] - 1;
    (scaled[more] < 1 ? small : large).push(more);
  }
  while (large.length) prob[large.pop()] = 1;
  while (small.length) prob[small.pop()] = 1;
  return { prob, alias };
}
export function chooseDegree(seqLen, rng) {
  const { prob, alias } = aliasTable(Array.from({ length: seqLen }, (_, i) => 1 / (i + 1)));
  const c = Math.floor(rng.nextDouble() * prob.length);
  return (rng.nextDouble() < prob[c] ? c : alias[c]) + 1;
}
export function shuffle(items, rng, degree = items.length) {
  const remaining = [...items], result = [];
  while (remaining.length && result.length < degree) result.push(remaining.splice(rng.nextInt(0, remaining.length - 1), 1)[0]);
  return result;
}
export function chooseFragments(seqNum, seqLen, checksum) {
  if (seqNum <= seqLen) return [seqNum - 1];
  const rng = new Xoshiro(concat([u32be(seqNum), u32be(checksum)]));
  const degree = chooseDegree(seqLen, rng);
  return shuffle(Array.from({ length: seqLen }, (_, i) => i), rng, degree);
}

// ---- URs: one part, or a fountain's parts ----
export function urEncode(type, cbor) { return `ur:${type}/${bytewordsEncode(cbor)}`; }

/** A message too long for one QR as its pure parts, which a reader may see in any order and any number of times. */
export function urParts(type, cbor, maxFragment = 200) {
  if (cbor.length <= maxFragment) return [urEncode(type, cbor)];
  let len = cbor.length;
  for (let count = 1; count <= Math.ceil(cbor.length / 10); count++) { len = Math.ceil(cbor.length / count); if (len <= maxFragment) break; }
  const n = Math.ceil(cbor.length / len), checksum = crc32(cbor);
  return Array.from({ length: n }, (_, i) => {
    const frag = new Uint8Array(len); frag.set(cbor.subarray(i * len, (i + 1) * len));
    return `ur:${type}/${i + 1}-${n}/${bytewordsEncode(cborEncode([i + 1, n, cbor.length, checksum, frag]))}`;
  });
}

const xor = (a, b) => a.map((x, i) => x ^ b[i]);
/** Reads a UR from its QR texts, one part or many, in any order; mixed parts are reduced as the reference does. */
export class URDecoder {
  constructor() { this.type = null; this.result = null; this.expected = null; this.simple = new Map(); this.mixed = []; this.queue = []; }
  get done() { return this.result !== null; }
  get progress() { return this.done ? 1 : this.expected ? this.simple.size / this.expected.seqLen : 0; }
  receive(text) {
    if (this.done) return true;
    const m = /^ur:([a-z0-9-]+)\/(?:(\d+)-(\d+)\/)?([a-z]+)$/i.exec(text.trim());
    if (!m) throw new Error('not a UR');
    const type = m[1].toLowerCase();
    if (this.type && this.type !== type) throw new Error(`a ${type} UR while reading a ${this.type}`);
    this.type = type;
    const body = bytewordsDecode(m[4]);
    if (!m[2]) { this.result = body; return true; }
    const [seqNum, seqLen, messageLen, checksum, fragment] = cborDecode(body);
    if (!this.expected) this.expected = { seqLen, messageLen, checksum, fragmentLen: fragment.length };
    const e = this.expected;
    if (seqLen !== e.seqLen || messageLen !== e.messageLen || checksum !== e.checksum || fragment.length !== e.fragmentLen) return false;
    this.queue.push({ indexes: chooseFragments(seqNum, seqLen, checksum).sort((a, b) => a - b), fragment });
    while (!this.done && this.queue.length) {
      let block = this.queue.shift();
      if (block.indexes.length === 1) { this.addSimple(block); continue; }
      if (this.mixed.some((b) => same(b.indexes, block.indexes))) continue;
      for (const [i, f] of this.simple) block = reduce(block, { indexes: [i], fragment: f });
      for (const b of this.mixed) block = reduce(block, b);
      if (block.indexes.length === 1) { this.queue.push(block); continue; }
      this.reduceMixedBy(block);
      this.mixed.push(block);
    }
    return this.done;
  }
  addSimple(block) {
    const i = block.indexes[0];
    if (this.simple.has(i)) return;
    this.simple.set(i, block.fragment);
    if (this.simple.size === this.expected.seqLen) {
      const message = concat([...this.simple.entries()].sort(([a], [b]) => a - b).map(([, f]) => f)).subarray(0, this.expected.messageLen);
      if (crc32(message) !== this.expected.checksum) throw new Error('UR: the message fails its checksum');
      this.result = message;
    } else this.reduceMixedBy(block);
  }
  reduceMixedBy(block) {
    const kept = [];
    for (const b of this.mixed.map((x) => reduce(x, block))) (b.indexes.length === 1 ? this.queue : kept).push(b);
    this.mixed = kept;
  }
}
const same = (a, b) => a.length === b.length && a.every((x) => b.includes(x));
function reduce(a, b) {
  if (!b.indexes.every((i) => a.indexes.includes(i))) return a;
  return { indexes: a.indexes.filter((i) => !b.indexes.includes(i)), fragment: xor(a.fragment, b.fragment) };
}
