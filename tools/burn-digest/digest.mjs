#!/usr/bin/env node
// THE BURN BOOK'S DAILY DIGEST (Oct 13 2026). Row 13 of the day 0 page, "the spend is read too"
// (docs/open-transparent-markets-from-day-0-2026-10-13.md, §3 hop 3 and §6): the house's inference spend lives off chain in the burn
// book, ~/.embertide/minirouter-burn.jsonl, one receipt per call, appended by the Aiolos door (minirouter-openai-shim.mjs), the
// Station's /rite/ask and the app's inference lane. This file turns the book into one SHA-256 per UTC day, each chained to the day
// before, so that one published line pins every receipt up to its day: the book cannot be rewritten afterwards without the chain
// naming the day. README.md beside this file is the specification of the exact bytes.
//
//   node digest.mjs build                      record every closed UTC day that has no digest yet in state/digests.jsonl
//   node digest.mjs verify                     recompute everything from the book and name the first day that differs
//   node digest.mjs memo [--day YYYY-MM-DD]    print the one line that would be anchored for a day (default: the latest closed day)
//   --book <file> | MINIROUTER_BURN_FILE       --state <dir> | BURN_DIGEST_STATE
//
// The limits, held in code: the book is opened read-only and nothing here writes, renames or removes it; the digests file may not live
// in the book's folder; rows are hashed, never copied; no socket, no transaction, no key. Anchoring a line on chain is his hand.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';

export const TAG = 'burn-digest/1';
export const GENESIS = '0'.repeat(64);
// A day closes ten minutes after its UTC midnight. Every writer stamps `at` and appends in the same tick, so the margin only has to
// outlast a clock step; and a line once anchored cannot be taken back, so a day is digested late rather than early.
export const SETTLE_MS = 10 * 60 * 1000;
const DAY_MS = 86400000;
const HERE = path.dirname(fileURLToPath(import.meta.url));
const USAGE = 'usage: node digest.mjs build | verify | memo [--day YYYY-MM-DD]   [--book <file>] [--state <dir>]';

class Stop extends Error { constructor(message, code = 2) { super(message); this.code = code; } }

const dayOf = (ms) => new Date(ms).toISOString().slice(0, 10);
const nextDay = (day) => dayOf(Date.parse(day + 'T00:00:00Z') + DAY_MS);
const isDay = (s) => /^\d{4}-\d{2}-\d{2}$/.test(s) && !Number.isNaN(Date.parse(s + 'T00:00:00Z')) && dayOf(Date.parse(s + 'T00:00:00Z')) === s;
export const lastClosedDay = (now) => dayOf(now.getTime() - SETTLE_MS - DAY_MS);

// Money is summed in whole micro-dollars (every writer rounds costUsd to six decimals), never in floats.
const usd = (micro) => { const a = micro < 0n ? -micro : micro; return `${micro < 0n ? '-' : ''}${a / 1000000n}.${String(a % 1000000n).padStart(6, '0')}`; };

// THE CANONICAL FORM is RFC 8785 (JCS): no whitespace, members sorted by their names' UTF-16 code units, numbers and strings as
// ECMAScript's JSON.stringify writes them. A string that is not well-formed Unicode has no canonical form there, and none here.
export function canonical(v) {
  if (typeof v === 'string') { if (!v.isWellFormed()) throw new Error('an unpaired surrogate has no canonical form'); return JSON.stringify(v); }
  if (v === null || typeof v !== 'object') return JSON.stringify(v);
  if (Array.isArray(v)) return '[' + v.map(canonical).join(',') + ']';
  return '{' + Object.keys(v).sort().map((k) => canonical(k) + ':' + canonical(v[k])).join(',') + '}';
}

// A RECEIPT is a line of the book that is valid UTF-8, parses as a JSON object and carries `at` as a UTC instant the way
// Date.toISOString writes it (YYYY-MM-DDTHH:MM:SS[.fff]Z), which is what every writer of the book stamps. Its day is the date in `at`:
// the same UTC day the door's own tank counts by. Any other line is not a receipt: it is counted, reported, and under no digest.
const AT = /^(\d{4}-\d{2}-\d{2})T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/;
const utf8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
function receiptOf(bytes) {
  let r; try { r = JSON.parse(utf8.decode(bytes)); } catch { return null; }
  if (!r || typeof r !== 'object' || Array.isArray(r) || typeof r.at !== 'string') return null;
  const m = AT.exec(r.at), t = m ? Date.parse(r.at) : NaN;
  if (Number.isNaN(t) || dayOf(t) !== m[1]) return null;
  const micro = typeof r.costUsd === 'number' ? Math.round(r.costUsd * 1e6) : NaN;
  try { return { day: m[1], line: canonical(r), micro: Number.isFinite(micro) ? BigInt(micro) : null }; } catch { return null; }
}

// The one place the book is touched: opened read-only, read once, closed.
function readBook(file) {
  let fd;
  try { fd = fs.openSync(file, fs.constants.O_RDONLY); } catch (e) { throw new Stop(e.code === 'ENOENT' ? `no burn book at ${file}` : `cannot open the burn book ${file}: ${e.code}`); }
  try { return fs.readFileSync(fd); } catch (e) { throw new Stop(`cannot read the burn book ${file}: ${e.code}`); } finally { fs.closeSync(fd); }
}

// scan(bytes): the book read the way the purse feed reads it (split on LF, blank lines are nothing), each receipt filed under its day
// in book order as its canonical line. The canonical lines live in memory for the length of the run and are written nowhere.
export function scan(buf) {
  const days = new Map(), stray = [];
  let lines = 0, torn = false;
  for (let start = 0; start < buf.length;) {
    let end = buf.indexOf(0x0a, start);
    const last = end < 0; if (last) end = buf.length;
    const bytes = buf.subarray(start, end); start = end + 1; lines++;
    if (bytes.every((b) => b === 0x20 || b === 0x09 || b === 0x0d)) continue;
    const r = receiptOf(bytes);
    if (!r) { stray.push(lines); torn = last; continue; }
    let d = days.get(r.day); if (!d) days.set(r.day, d = { lines: [], micro: 0n, unpriced: 0 });
    d.lines.push(r.line);
    if (r.micro === null) d.unpriced++; else d.micro += r.micro;
  }
  return { days, stray, torn, lines };
}

// chain(days, first, last): one digest for every UTC day from first to last, a day with no receipt included (rows 0), so that no day
// can be dropped or slipped in. digest = SHA-256 of the UTF-8 bytes of: the tag, LF, the previous day's digest (64 zeros before the
// first day), LF, the day, LF, then each of the day's receipts in book order as its canonical line and LF.
const EMPTY = { lines: [], micro: 0n, unpriced: 0 };
export function chain(days, first, last) {
  const out = [];
  let prev = GENESIS;
  for (let day = first; day <= last; day = nextDay(day)) {
    const d = days.get(day) || EMPTY;
    const h = createHash('sha256').update(`${TAG}\n${prev}\n${day}\n`);
    for (const line of d.lines) h.update(line + '\n');
    const digest = h.digest('hex');
    out.push({ day, rows: d.lines.length, unpriced: d.unpriced, cost_usd: usd(d.micro), prev, digest, micro: d.micro });
    prev = digest;
  }
  return out;
}

// THE DIGESTS FILE: one line per day, these six fields in this order, nothing of a row. It only ever grows.
const FIELDS = ['day', 'rows', 'unpriced', 'cost_usd', 'prev', 'digest'];
const lineOf = (d) => JSON.stringify(Object.fromEntries(FIELDS.map((k) => [k, d[k]])));
export const memoOf = (d) => `${TAG} ${d.day} rows=${d.rows} cost_usd=${d.cost_usd} sha256=${d.digest}`;
const count = (v) => Number.isSafeInteger(v) && v >= 0, hex = (v) => typeof v === 'string' && /^[0-9a-f]{64}$/.test(v);
function readState(file) {
  let text; try { text = fs.readFileSync(file, 'utf8'); } catch (e) { if (e.code === 'ENOENT') return []; throw new Stop(`cannot read the digests file ${file}: ${e.code}`); }
  const lines = text.split('\n');
  if (lines.pop() !== '') throw new Stop(`the digests file ${file} does not end on a full line: it is damaged, and nothing was done`);
  return lines.map((l, i) => {
    let d = null; try { d = JSON.parse(l); } catch {}
    const ok = d && typeof d === 'object' && typeof d.day === 'string' && isDay(d.day) && count(d.rows) && count(d.unpriced)
      && typeof d.cost_usd === 'string' && /^-?\d+\.\d{6}$/.test(d.cost_usd) && hex(d.prev) && hex(d.digest) && lineOf(d) === l;
    if (!ok) throw new Stop(`the digests file ${file} is damaged at line ${i + 1}, and nothing was done`);
    return d;
  });
}
function appendState(file, days) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  let old = Buffer.alloc(0); try { old = fs.readFileSync(file); } catch (e) { if (e.code !== 'ENOENT') throw e; }
  const tmp = `${file}.tmp-${process.pid}`, fd = fs.openSync(tmp, 'w', 0o600);
  try { fs.writeFileSync(fd, Buffer.concat([old, Buffer.from(days.map((d) => lineOf(d) + '\n').join(''))])); fs.fsyncSync(fd); } finally { fs.closeSync(fd); }
  fs.renameSync(tmp, file);
}

// NEVER BESIDE THE BOOK: the digests folder may be neither the book's folder nor anything under it, links resolved; when the book's
// path is itself a link, that holds for the folder the link sits in and for the folder of the file it points to.
function real(p) {
  const rest = [];
  for (let cur = path.resolve(p); ;) {
    try { return path.join(fs.realpathSync.native(cur), ...rest); } catch (e) { if (e.code !== 'ENOENT') throw e; }
    const up = path.dirname(cur); if (up === cur) return path.resolve(p);
    rest.unshift(path.basename(cur)); cur = up;
  }
}
function fence(book, dir) {
  const at = real(dir);
  for (const home of new Set([path.dirname(real(book)), real(path.dirname(path.resolve(book)))]))
    if (at === home || at.startsWith(home.endsWith(path.sep) ? home : home + path.sep)) throw new Stop(`the digests may not live in or under the burn book's folder (${home}): give --state another place`);
}

// Everything is recomputed from the book on every run (one pass, milliseconds): from the earliest day either the book or the record
// names, through the last closed day, or the last recorded day if the clock says less.
function load(a, env, now) {
  const book = path.resolve(a.book || env.MINIROUTER_BURN_FILE || path.join(os.homedir(), '.embertide/minirouter-burn.jsonl'));
  const dir = path.resolve(a.state || env.BURN_DIGEST_STATE || path.join(HERE, 'state'));
  fence(book, dir);
  const file = path.join(dir, 'digests.jsonl'), t0 = performance.now();
  const s = scan(readBook(book)), recorded = readState(file), closed = lastClosedDay(now);
  const bookFirst = [...s.days.keys()].sort()[0], recFirst = recorded[0]?.day, recLast = recorded.at(-1)?.day;
  const first = bookFirst && recFirst ? (bookFirst < recFirst ? bookFirst : recFirst) : bookFirst || recFirst;
  const last = recLast && recLast > closed ? recLast : closed;
  const computed = first && first <= last ? chain(s.days, first, last) : [];
  const bad = recorded.findIndex((d, i) => !computed[i] || lineOf(d) !== lineOf(computed[i]));
  let waiting = 0; for (const [day, d] of s.days) if (day > last) waiting += d.lines.length;
  return { ...s, book, file, recorded, computed, bad, closed, last, waiting, ms: Math.round(performance.now() - t0) };
}

function why(rec, now) {
  if (!now) return 'the book no longer reaches this day';
  if (now.day !== rec.day) return `the book has ${now.rows} receipt(s) dated ${now.day}, before the first recorded day ${rec.day}`;
  if (now.rows !== rec.rows) return `recorded ${rec.rows} rows, the book now has ${now.rows || 'none: the day is gone'}`;
  if (now.cost_usd !== rec.cost_usd) return `same row count, the total moved from ${rec.cost_usd} to ${now.cost_usd} USD`;
  if (now.prev !== rec.prev) return 'the recorded line does not chain to the line before it: the digests file itself was edited';
  return 'same row count and total, another digest: a row was edited or the rows stand in another order (or the recorded line was edited)';
}
function mismatch(cmd, L, say) {
  const rec = L.recorded[L.bad], now = L.computed[L.bad];
  say(`burn-digest ${cmd}: MISMATCH, first at ${now && now.day < rec.day ? now.day : rec.day}`);
  say(`  recorded  ${memoOf(rec)}`);
  if (now) say(`  the book  ${memoOf(now)}`);
  say(`  ${why(rec, now)}`);
  say(`  ${L.bad} of ${L.recorded.length} recorded days stand before it; every day after it differs through the chain`);
  return 1;
}
function span(days) {
  const rows = days.reduce((n, d) => n + d.rows, 0), unpriced = days.reduce((n, d) => n + d.unpriced, 0);
  return `${days[0].day} to ${days.at(-1).day}, ${rows} receipts, ${usd(days.reduce((n, d) => n + d.micro, 0n))} USD${unpriced ? `, ${unpriced} with no cost` : ''}`;
}
function notes(L, out) {
  if (L.stray.length) out(`  ${L.stray.length} line(s) of the book are not receipts and are under no digest: line ${L.stray.slice(0, 8).join(', ')}${L.stray.length > 8 ? ', ...' : ''}${L.torn ? ' (the last line is not whole: an append may be in progress)' : ''}`);
  if (L.waiting) out(`  ${L.waiting} receipt(s) dated after ${L.last} wait for their day to close`);
}

export function run(argv, { now = new Date(), env = process.env, out = console.log, err = console.error } = {}) {
  try {
    const [cmd, ...rest] = argv, a = {};
    for (let i = 0; i < rest.length; i += 2) {
      if (!['--book', '--state', '--day'].includes(rest[i]) || rest[i + 1] === undefined) throw new Stop(USAGE);
      a[rest[i].slice(2)] = rest[i + 1];
    }
    if (!['build', 'verify', 'memo'].includes(cmd) || (a.day !== undefined && (cmd !== 'memo' || !isDay(a.day)))) throw new Stop(USAGE);
    const L = load(a, env, now), head = L.computed.slice(0, L.recorded.length);

    if (cmd === 'memo') {
      const day = a.day || L.closed, i = L.recorded.findIndex((d) => d.day === day);
      if (day > L.closed) throw new Stop(`${day} is still open (the last closed UTC day is ${L.closed}): it has no digest yet`);
      if (L.bad >= 0 && (i < 0 || L.bad <= i)) return mismatch('memo', L, err);
      if (i < 0) throw new Stop(`no digest is recorded for ${day}${L.recorded.length ? ` (the record runs ${L.recorded[0].day} to ${L.recorded.at(-1).day})` : ''}: run build first`);
      out(memoOf(L.recorded[i]));
      return 0;
    }
    if (L.bad >= 0) { const code = mismatch(cmd, L, out); if (cmd === 'build') out('  nothing was written: build never rewrites a recorded day'); notes(L, out); return code; }
    const fresh = L.computed.slice(L.recorded.length);
    if (cmd === 'verify') {
      if (!L.recorded.length) throw new Stop(`no digests are recorded yet at ${L.file}: run build first`);
      out(`burn-digest verify: all ${head.length} recorded days match the book (${span(head)}) in ${L.ms} ms`);
      if (fresh.length) out(`  ${fresh.length} closed day(s) have no digest yet (${fresh[0].day} to ${fresh.at(-1).day}): run build`);
    } else {
      if (fresh.length) appendState(L.file, fresh);
      if (!L.computed.length) out('burn-digest build: nothing to digest yet, the book has no receipt on a closed day');
      else {
        out(`burn-digest build: ${fresh.length} day(s) added, ${L.computed.length} recorded (${span(L.computed)}) in ${L.ms} ms`);
        out(`  head  ${memoOf(L.computed.at(-1))}`);
        out(`  digests ${L.file}   book, read only, ${L.lines} lines: ${L.book}`);
      }
    }
    notes(L, out);
    return 0;
  } catch (e) {
    err(`burn-digest: ${e instanceof Stop ? e.message : e.stack || e}`);   // 1 is kept for a mismatch: anything else that stops a run is 2
    return e instanceof Stop ? e.code : 2;
  }
}

const invoked = (() => { try { return fs.realpathSync(process.argv[1]) === fs.realpathSync(fileURLToPath(import.meta.url)); } catch { return false; } })();
if (invoked) process.exitCode = run(process.argv.slice(2));
