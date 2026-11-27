// Account layouts computed from an IDL at load time, so no offset in this tool is typed by hand.
//
// A type is laid out here only when every field has a fixed size: an Anchor zero-copy account (the IDL marks it
// `serialization: "bytemuck"`; bytemuck's Pod forbids implicit padding, so a field's offset is the sum of the sizes before it) or
// a struct this tool marks `"packed"` itself (Streamflow's Contract: Borsh over fixed-size fields, which is the same sum).
// Anything else (a string, a vec, an option) throws: a variable layout is read by lib/borsh.mjs field by field instead.
// A plain Borsh struct (an event) may be laid out with { borsh: true }: Borsh adds nothing between fixed-size fields, and one
// field of variable size still throws.
import { b58, keyBytes, pda } from './web3.mjs';

const PRIMITIVES = { u8: 1, i8: 1, bool: 1, u16: 2, i16: 2, u32: 4, i32: 4, f32: 4, u64: 8, i64: 8, u128: 16, i128: 16, pubkey: 32 };
const FIXED = new Set(['bytemuck', 'bytemuckunsafe', 'packed']);

function typeMap(idl) { return new Map((idl.types || []).map(t => [t.name, t])); }

function walk(types, type, path, offset, out, trail, borsh) {
  if (typeof type === 'string') {
    const size = PRIMITIVES[type];
    if (!size) throw new Error(`layout: ${path} is a ${type}, which has no fixed size`);
    out.push({ path, offset, size, type });
    return size;
  }
  if (type.array) {
    const [inner, count] = type.array;
    if (inner === 'u8') { out.push({ path, offset, size: count, type: 'bytes' }); return count; }
    let size = 0;
    for (let i = 0; i < count; i++) size += walk(types, inner, `${path}.${i}`, offset + size, out, trail, borsh);
    return size;
  }
  if (type.defined) {
    const name = typeof type.defined === 'string' ? type.defined : type.defined.name;
    const def = types.get(name);
    if (!def) throw new Error(`layout: ${path} names the type ${name}, which the IDL does not define`);
    if (trail.includes(name)) throw new Error(`layout: ${name} contains itself`);
    if (def.type.kind !== 'struct') throw new Error(`layout: ${path} is the ${def.type.kind} ${name}, not a struct`);
    if (!borsh && !FIXED.has(def.serialization)) throw new Error(`layout: ${name} is not marked fixed-size (serialization: ${def.serialization ?? 'borsh'})`);
    let size = 0;
    for (const f of def.type.fields) size += walk(types, f.type, path ? `${path}.${f.name}` : f.name, offset + size, out, [...trail, name], borsh);
    return size;
  }
  throw new Error(`layout: ${path} has a type with no fixed size: ${JSON.stringify(type)}`);
}

const cache = new WeakMap();

/** { name, size, fields: [{ path, offset, size, type }], at(path) } for a fixed-size struct of the IDL. */
export function layoutOf(idl, name, { borsh = false } = {}) {
  let perIdl = cache.get(idl);
  if (!perIdl) cache.set(idl, perIdl = new Map());
  if (perIdl.has(name)) return perIdl.get(name);
  const fields = [];
  const size = walk(typeMap(idl), { defined: { name } }, '', 0, fields, [], borsh);
  const byPath = new Map(fields.map(f => [f.path, f]));
  const layout = {
    name, size, fields,
    at(path) { const f = byPath.get(path); if (!f) throw new Error(`layout: ${name} has no field ${path}`); return f; },
  };
  perIdl.set(name, layout);
  return layout;
}

function readField(buf, base, f) {
  const o = base + f.offset;
  if (o + f.size > buf.length) throw new Error(`layout: ${f.path} at ${o}+${f.size} is past the end of ${buf.length} bytes`);
  switch (f.type) {
    case 'u8': return buf.readUInt8(o);
    case 'i8': return buf.readInt8(o);
    case 'bool': return buf.readUInt8(o) !== 0;
    case 'u16': return buf.readUInt16LE(o);
    case 'i16': return buf.readInt16LE(o);
    case 'u32': return buf.readUInt32LE(o);
    case 'i32': return buf.readInt32LE(o);
    case 'f32': return buf.readFloatLE(o);
    case 'u64': return buf.readBigUInt64LE(o);
    case 'i64': return buf.readBigInt64LE(o);
    case 'u128': return buf.readBigUInt64LE(o) | (buf.readBigUInt64LE(o + 8) << 64n);
    case 'i128': return BigInt.asIntN(128, buf.readBigUInt64LE(o) | (buf.readBigUInt64LE(o + 8) << 64n));
    case 'pubkey': return b58(buf.subarray(o, o + 32));
    case 'bytes': return Buffer.from(buf.subarray(o, o + f.size));
    default: throw new Error(`layout: cannot read a ${f.type}`);
  }
}

/** A reader over `buf` starting at `base`: get('a.b.0.c') reads one field by its IDL path. */
export function view(layout, buf, base = 0) {
  if (base + layout.size > buf.length) throw new Error(`layout: ${layout.name} needs ${layout.size} bytes at ${base}, the data holds ${buf.length}`);
  return { layout, get: path => readField(buf, base, layout.at(path)) };
}

/** The 8-byte Anchor discriminator the IDL gives for an account, an instruction or an event. */
export function discriminator(idl, kind, name) {
  const hit = (idl[kind] || []).find(x => x.name === name);
  if (!hit || !Array.isArray(hit.discriminator)) throw new Error(`the IDL has no ${kind} discriminator for ${name}`);
  return Buffer.from(hit.discriminator);
}

/** The position of each named account in an instruction's account list, from the IDL. */
export function accountIndexes(idl, instruction) {
  const ix = (idl.instructions || []).find(x => x.name === instruction);
  if (!ix) throw new Error(`the IDL has no instruction ${instruction}`);
  return Object.fromEntries(ix.accounts.map((a, i) => [a.name, i]));
}

/**
 * The address the IDL derives for one account of an instruction: its seeds are read from the IDL (constants and other
 * accounts of the same instruction, given in `known`), never typed here.
 */
export function pdaOf(idl, instruction, account, known = {}) {
  const ix = (idl.instructions || []).find(x => x.name === instruction);
  const acc = ix?.accounts.find(a => a.name === account);
  if (!acc?.pda) throw new Error(`the IDL derives no address for ${instruction}.${account}`);
  if (acc.pda.program) throw new Error(`${instruction}.${account} is derived under another program`);
  const seeds = acc.pda.seeds.map(seed => {
    if (seed.kind === 'const') return Buffer.from(seed.value);
    if (seed.kind === 'account' && known[seed.path]) return keyBytes(known[seed.path]);
    throw new Error(`${instruction}.${account} needs the seed ${seed.path ?? seed.kind}`);
  });
  return pda(seeds, idl.address);
}
