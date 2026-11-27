#!/usr/bin/env node
// warm-day0: which of the fourteen day 0 claims hold for a Solana mint (docs/open-transparent-markets-from-day-0-2026-10-13.md, section 6).
//
// Read-only. It asks a cluster for accounts and transactions through the methods lib/rpc.mjs allows and nothing else: it
// makes and loads no key, builds, simulates and sends no transaction, and writes no file unless --record names one.
//
//   node check.mjs <mint> [--cluster mainnet|devnet] [--manifest file.json] [--domain example.com] [--json]
//                         [--rpc url] [--history n] [--require 2,3,4] [--digests file] [--record file] [--replay file]
//
// Exit 0: no decidable row failed (and every --require row passed). 1: a row failed. 2: the run could not be made.
import { readFileSync, writeFileSync } from 'node:fs';
import { resolveTxt } from 'node:dns/promises';
import { fileURLToPath } from 'node:url';
import { createRpc, httpTransport, fixtureTransport, recordingTransport, CLUSTERS } from './lib/rpc.mjs';
import { runChecks, UsageError } from './lib/rows.mjs';
import { parseManifest, parseDigests } from './lib/manifest.mjs';
import { renderTable, renderJson, exitOf } from './lib/render.mjs';
import { WEB3_VERSION } from './lib/web3.mjs';

const USAGE = `usage: check.mjs <mint> [--cluster mainnet|devnet] [--manifest file.json] [--domain example.com] [--json]

  <mint>              the token's mint address
  --cluster           mainnet (default) or devnet
  --manifest file     the labelled token accounts, the pool, the locks, the fund address and the house wallets (manifest.example.json)
  --domain name       read this name's DNS TXT records for the mint (row 1)
  --json              print the result as JSON instead of the table

  --rpc url           another endpoint for the same cluster (or WARM_DAY0_RPC); its genesis hash is checked
  --history n         how many transactions back to read for each address in rows 11, 12 and 13 (default 200)
  --require 2,3,4     exit 1 unless these rows pass (a row that is "not checkable" does not fail the run by itself)
  --digests file      burn-digest's record (state/digests.jsonl): compare the lines on chain with it (row 13)
  --record file       also write every answer the cluster gave to this file (a fixture)
  --replay file       answer from a recorded file instead of a cluster: no network at all`;

export function parseArgs(argv) {
  const opts = { cluster: 'mainnet', history: 200, json: false, require: [] };
  const rest = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const value = () => { if (i + 1 >= argv.length) throw new UsageError(`${a} needs a value`); return argv[++i]; };
    if (a === '--json') opts.json = true;
    else if (a === '--help' || a === '-h') opts.help = true;
    else if (a === '--cluster') opts.cluster = value();
    else if (a === '--manifest') opts.manifest = value();
    else if (a === '--domain') opts.domain = value();
    else if (a === '--rpc') opts.rpc = value();
    else if (a === '--history') opts.history = Number(value());
    else if (a === '--require') opts.require = value().split(',').map(Number);
    else if (a === '--digests') opts.digests = value();
    else if (a === '--record') opts.record = value();
    else if (a === '--replay') opts.replay = value();
    else if (a.startsWith('-')) throw new UsageError(`unknown option ${a}`);
    else rest.push(a);
  }
  if (opts.help) return opts;
  if (rest.length !== 1) throw new UsageError(rest.length ? 'one mint address at a time' : 'which mint?');
  opts.mint = rest[0];
  if (!Object.hasOwn(CLUSTERS, opts.cluster)) throw new UsageError(`--cluster is mainnet or devnet, not ${opts.cluster}`);
  if (!Number.isInteger(opts.history) || opts.history < 1 || opts.history > 1000) throw new UsageError('--history is a whole number from 1 to 1000');
  if (opts.require.some(n => !Number.isInteger(n) || n < 1 || n > 14)) throw new UsageError('--require is a list of row numbers, 1 to 14');
  if (opts.record && opts.replay) throw new UsageError('--record and --replay do not go together');
  if (opts.domain && !/^[a-z0-9_]([a-z0-9_-]*[a-z0-9])?(\.[a-z0-9_]([a-z0-9_-]*[a-z0-9])?)+$/i.test(opts.domain)) throw new UsageError(`--domain is a host name, not ${opts.domain}`);
  return opts;
}

const readJson = (path, what) => { try { return JSON.parse(readFileSync(path, 'utf8')); } catch (e) { throw new UsageError(`${what} ${path}: ${e.message}`); } };

/** One run. `io` lets a test stand in for the cluster, DNS and the clock. Returns { code, stdout, stderr, result }. */
export async function main(argv, io = {}) {
  let opts;
  try { opts = parseArgs(argv); } catch (e) { return { code: 2, stdout: '', stderr: `${e.message}\n${USAGE}` }; }
  if (opts.help) return { code: 0, stdout: USAGE, stderr: '' };
  try {
    const replay = opts.replay ? readJson(opts.replay, 'fixture') : null;
    if (replay && replay.meta?.mint !== opts.mint) throw new UsageError(`the fixture is for ${replay.meta?.mint}, not ${opts.mint}`);
    if (replay) { opts.cluster = replay.meta.cluster; if (!argv.includes('--history')) opts.history = replay.meta.history; if (!opts.domain) opts.domain = replay.meta.domain ?? undefined; }
    const rawManifest = opts.manifest ? readJson(opts.manifest, 'manifest') : replay?.manifest ?? null;
    const manifest = rawManifest ? parseManifest(rawManifest, opts.mint) : null;
    const digests = opts.digests ? parseDigests(readFileSync(opts.digests, 'utf8')) : null;

    const url = opts.rpc || process.env.WARM_DAY0_RPC || CLUSTERS[opts.cluster].url;
    const recorded = {}, dnsSeen = {};
    let transport = io.transport || (replay ? fixtureTransport(replay) : httpTransport(url));
    if (opts.record) transport = recordingTransport(transport, recorded);
    const rpc = createRpc(transport);

    const genesis = await rpc.genesis();
    if (genesis !== CLUSTERS[opts.cluster].genesis) throw new UsageError(`${replay ? 'the fixture' : url} is not ${opts.cluster}: its genesis hash is ${genesis}`);

    const lookup = io.resolveTxt || resolveTxt;
    const dns = async (name) => {
      if (replay) { const hit = replay.dns?.[name]; if (!hit) throw Object.assign(new Error('not in the fixture'), { code: 'ENOTRECORDED' }); if (hit.error) throw Object.assign(new Error(hit.error), { code: hit.error }); return hit.records; }
      try { const records = await lookup(name); dnsSeen[name] = { records }; return records; } catch (e) { dnsSeen[name] = { error: e.code || 'EFAIL' }; throw e; }
    };

    const result = await runChecks({ rpc, mint: opts.mint, cluster: opts.cluster, manifest, domain: opts.domain || null, resolveTxt: dns, history: opts.history, digests });
    const exit = exitOf(result, opts.require);
    result.exit = exit.code; result.exitWhy = exit.why; result.web3 = WEB3_VERSION;
    if (opts.record) {
      const meta = { tool: 'warm-day0', recordedAt: new Date((io.now || Date.now)()).toISOString(), cluster: opts.cluster, endpoint: new URL(url).host, mint: opts.mint, history: opts.history, domain: opts.domain ?? null, slot: result.slot, calls: Object.keys(recorded).length };
      // One recorded answer a line: nobody edits a fixture by hand, and a re-recording then differs line by line.
      const calls = Object.entries(recorded).map(([key, answer]) => `${JSON.stringify(key)}: ${JSON.stringify(answer)}`);
      writeFileSync(opts.record, `{\n"meta": ${JSON.stringify(meta)},\n"manifest": ${JSON.stringify(rawManifest)},\n"dns": ${JSON.stringify(dnsSeen)},\n"calls": {\n${calls.join(',\n')}\n}\n}\n`);
    }
    return { code: exit.code, stdout: opts.json ? renderJson(result) : renderTable(result), stderr: '', result };
  } catch (e) {
    if (e instanceof UsageError) return { code: 2, stdout: '', stderr: e.message };
    if (e.name === 'RpcError') return { code: 2, stdout: '', stderr: `the cluster did not answer: ${e.message}` };
    throw e;
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const { code, stdout, stderr } = await main(process.argv.slice(2));
  if (stdout) console.log(stdout);
  if (stderr) console.error(stderr);
  process.exitCode = code;
}
