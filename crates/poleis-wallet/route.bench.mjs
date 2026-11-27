// route.bench.mjs: what her wallet's door costs a birth (Oct 14 2026), for LANDING.md. The door (tools/embertide-mesh/lib/lyssajudron-wallet.mjs)
// behind a loopback server, the REAL Nest (nest.mjs's Nest and serve on a lent port, a test key in a temp keys folder: never his key, never the live
// Nest) and the real poleis-wallet on a temp folder: throwaway keys, the whole temp folder removed at the end, a failure included. Each round
// rings a new birth on the Nest's paint (as the Mac page's unboxing does), then times, at the client:
//   a new birth's POST (her key made, four fsyncs, the ref written signed), the same birth's POST again (two GETs, no process), a GET;
// and counts the bytes each way, the ref's entry on the WAL, and her folder on disk. Prints one JSON line; nothing of any key.
// Run: node poleis-wallet/route.bench.mjs [rounds=20]   (needs target/release/poleis-wallet: cargo build --release --offline)
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import { fileURLToPath } from 'node:url';

const ROUNDS = Number(process.argv[2] || 20);
const TMP = fs.mkdtempSync(path.join(os.tmpdir(), 'her-wallet-bench-'));
for (const k of ['NEST_UPSTREAM', 'NEST_CEILING', 'NEST_AUTHORITY', 'NEST_POLICY', 'NEST_PIN_KEYS', 'NEST_REQUIRE_SIG', 'NEST_FOREIGN_PORT', 'NEST_DISK_WORKER', 'LYSSAJUDRON_WALLET_BIN']) delete process.env[k];
process.env.NEST_KEYS = path.join(TMP, 'keys'); process.env.NEST_DIR = path.join(TMP, 'nest-dir'); process.env.NEST_MODALITIES = '0';
const BIN = fileURLToPath(new URL('./target/release/poleis-wallet', import.meta.url));
const ms = (t0) => Number(process.hrtime.bigint() - t0) / 1e6;
const median = (a) => { const s = [...a].sort((x, y) => x - y); return +s[Math.floor(s.length / 2)].toFixed(2); };
const max = (a) => +Math.max(...a).toFixed(2);
let out;
try {
  if (!fs.existsSync(BIN)) throw new Error('poleis-wallet is not built here: cargo build --release --offline');
  const { Nest, serve, keygen } = await import('../tools/embertide-mesh/nest.mjs');
  keygen('mac');
  const { NestClient } = await import('../tools/embertide-mesh/nest-client.mjs');
  const { createLyssajudronWallet, PAINT_REF, WALLET_REF } = await import('../tools/embertide-mesh/lib/lyssajudron-wallet.mjs');
  const nest = new Nest(fs.mkdtempSync(path.join(TMP, 'nest-')));
  const nestServer = serve(nest, { port: 0, bind: '127.0.0.1', upstream: '' });
  await new Promise((ok) => nestServer.once('listening', ok));
  const nestURL = `http://127.0.0.1:${nestServer.address().port}`, hand = new NestClient(nestURL, 'mac');
  const dir = path.join(TMP, 'home', '.embertide', 'lyssajudron', 'wallets');
  const door = createLyssajudronWallet({ nestURL, dir, bin: BIN });
  const server = http.createServer((req, res) => door.handle(req, res));
  await new Promise((ok) => server.listen(0, '127.0.0.1', ok));
  const url = `http://127.0.0.1:${server.address().port}/api/lyssajudron/wallet`;
  const t = { made: [], again: [], get: [] }, bytes = {};
  let paintRev = 0;
  for (let i = 0; i < ROUNDS; i++) {
    const birth = `bench${i.toString(36)}${Date.now().toString(36)}`;
    paintRev = (await hand.cas(PAINT_REF, { schema: 'embertide.golem-paint/1', golem: 'lyssajudron', parts: {}, painted: [], birth }, paintRev)).rev;
    const body = JSON.stringify({ birth }), post = async () => { const t0 = process.hrtime.bigint(); const r = await fetch(url, { method: 'POST', headers: { 'content-type': 'application/json' }, body }); const text = await r.text(); return { took: ms(t0), status: r.status, text }; };
    const a = await post(); if (a.status !== 200 || JSON.parse(a.text).made !== true) throw new Error(`round ${i}: ${a.status} ${a.text.slice(0, 120)}`);
    const b = await post(); if (b.status !== 200 || JSON.parse(b.text).made !== false) throw new Error(`round ${i}: again ${b.status}`);
    const t0 = process.hrtime.bigint(); const g = await fetch(url); const gt = await g.text(); t.get.push(ms(t0));
    t.made.push(a.took); t.again.push(b.took);
    bytes.ask = Buffer.byteLength(body); bytes.answer = Buffer.byteLength(a.text); bytes.get = Buffer.byteLength(gt);
  }
  const entry = nest.recent.filter((e) => e.type === 'ref' && e.payload.name === WALLET_REF).at(-1);
  const folder = path.join(dir, fs.readdirSync(dir).filter((n) => !n.startsWith('.'))[0]);
  const onDisk = Object.fromEntries(fs.readdirSync(folder).map((f) => [f, fs.statSync(path.join(folder, f)).size]));
  out = { bench: 'her-wallet-route', rounds: ROUNDS, load: os.loadavg().map((x) => +x.toFixed(1)), cpu: os.cpus()[0].model,
    post_new_birth_ms: { median: median(t.made), max: max(t.made), first: +t.made[0].toFixed(2) }, post_made_birth_ms: { median: median(t.again), max: max(t.again) },
    get_ms: { median: median(t.get), max: max(t.get) }, bytes: { ...bytes, ref_value: Buffer.byteLength(JSON.stringify(entry.payload.value)), ref_entry_on_wal: Buffer.byteLength(JSON.stringify(entry)) },
    entry_signed: entry.signed === true, her_folder_bytes: onDisk, wallets_made: fs.readdirSync(dir).filter((n) => !n.startsWith('.')).length };
  await new Promise((ok) => { server.closeAllConnections?.(); server.close(ok); });
  await new Promise((ok) => { nestServer.closeAllConnections?.(); nestServer.close(() => { try { if (nest.fd != null) fs.closeSync(nest.fd); } catch {} ok(); }); });
} finally {
  fs.rmSync(TMP, { recursive: true, force: true });
}
console.log(JSON.stringify(out));
