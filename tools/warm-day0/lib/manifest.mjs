// The manifest: what the house says is where. Every address in it is public; it holds no key and no secret.
// A manifest is read strictly: an unknown field or a malformed address stops the run, because a typo here would turn a row
// into "not checkable" without anyone noticing.
import { isAddress } from './web3.mjs';
import { UsageError } from './rows.mjs';

const TOP = new Set(['about', 'mint', 'accounts', 'pool', 'locks', 'fund', 'houseWallets', 'schedule', 'burnDigest']);

function fail(what) { throw new UsageError(`manifest: ${what}`); }
function address(v, where) { if (!isAddress(v)) fail(`${where} is not a Solana address (${JSON.stringify(v)})`); return v; }
function label(v, where) { if (typeof v !== 'string' || !v.trim()) fail(`${where} needs a label`); return v.trim(); }
function only(obj, keys, where) { for (const k of Object.keys(obj)) if (!keys.includes(k)) fail(`${where} has an unknown field "${k}"`); }
function list(v, where) { if (v === undefined) return []; if (!Array.isArray(v)) fail(`${where} must be a list`); return v; }
function item(v, where) { if (!v || typeof v !== 'object' || Array.isArray(v)) fail(`${where} must be an object`); return v; }
function time(v, where) { if (v === undefined || v === null) return null; if (!Number.isInteger(v) || v < 0) fail(`${where} must be a unix time in seconds`); return v; }

/** Check a manifest's shape against the mint being run and return it normalized. Throws UsageError. */
export function parseManifest(raw, mintAddress) {
  const m = item(raw, 'the file');
  for (const k of Object.keys(m)) if (!TOP.has(k)) fail(`unknown field "${k}" (known: ${[...TOP].join(', ')})`);
  if (m.mint !== undefined && address(m.mint, 'mint') !== mintAddress) fail(`it is for the mint ${m.mint}, not ${mintAddress}`);
  const out = { accounts: [], locks: [], houseWallets: [], schedule: [], pool: null, fund: null, burnDigest: null };

  out.accounts = list(m.accounts, 'accounts').map((a, i) => { const w = `accounts[${i}]`; only(item(a, w), ['label', 'address'], w); return { label: label(a.label, w), address: address(a.address, `${w}.address`) }; });
  out.locks = list(m.locks, 'locks').map((a, i) => { const w = `locks[${i}]`; only(item(a, w), ['label', 'address'], w); return { label: label(a.label, w), address: address(a.address, `${w}.address`) }; });
  out.houseWallets = list(m.houseWallets, 'houseWallets').map((a, i) => { const w = `houseWallets[${i}]`; only(item(a, w), ['label', 'address'], w); return { label: label(a.label, w), address: address(a.address, `${w}.address`) }; });
  if (m.pool !== undefined) {
    only(item(m.pool, 'pool'), ['address', 'positions'], 'pool');
    out.pool = { address: address(m.pool.address, 'pool.address'), positions: list(m.pool.positions, 'pool.positions').map((p, i) => address(p, `pool.positions[${i}]`)) };
  }
  if (m.fund !== undefined) { only(item(m.fund, 'fund'), ['address'], 'fund'); out.fund = { address: address(m.fund.address, 'fund.address') }; }
  if (m.burnDigest !== undefined) { only(item(m.burnDigest, 'burnDigest'), ['address'], 'burnDigest'); out.burnDigest = { address: address(m.burnDigest.address, 'burnDigest.address') }; }
  const wallets = new Set(out.houseWallets.map(w => w.address));
  out.schedule = list(m.schedule, 'schedule').map((s, i) => {
    const w = `schedule[${i}]`;
    only(item(s, w), ['label', 'from', 'to', 'amount', 'notBefore', 'notAfter'], w);
    if (!wallets.has(address(s.from, `${w}.from`))) fail(`${w}.from is not one of the house wallets`);
    if (s.to !== 'burn') address(s.to, `${w}.to`);
    if (typeof s.amount !== 'string' || !/^[1-9]\d*$/.test(s.amount)) fail(`${w}.amount must be a whole number of base units, written as a string`);
    return { label: typeof s.label === 'string' ? s.label : '', from: s.from, to: s.to, amount: s.amount, notBefore: time(s.notBefore, `${w}.notBefore`), notAfter: time(s.notAfter, `${w}.notAfter`) };
  });
  for (const [name, entries] of [['accounts', out.accounts], ['locks', out.locks], ['houseWallets', out.houseWallets]]) {
    const seen = new Set();
    for (const e of entries) { if (seen.has(e.address)) fail(`${name} names ${e.address} twice`); seen.add(e.address); }
  }
  return out;
}

/** burn-digest's record (state/digests.jsonl): one JSON line a day. Returns Map day -> { rows, cost_usd, digest }. */
export function parseDigests(text) {
  const out = new Map();
  for (const line of text.split('\n')) {
    if (!line.trim()) continue;
    let row;
    try { row = JSON.parse(line); } catch { throw new UsageError('digests: a line is not JSON'); }
    if (typeof row.day !== 'string' || typeof row.digest !== 'string') throw new UsageError('digests: a line has no day or no digest');
    out.set(row.day, { rows: row.rows, cost_usd: row.cost_usd, digest: row.digest });
  }
  return out;
}
