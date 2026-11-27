// A cluster session for the ceremony's scripts: keys by name, steps done once (resumable), JSON-RPC with backoff, the Solana CLI,
// and transactions compiled by the adapter (poleis/client/examples/ceremony.rs), signed here, sent and read back.
// rehearse.mjs keeps its own copy of these; this module is what the scripts after it share.
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { checkKeypair, freshKeypair, publicOf, signMessage } from './wire.mjs';

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** The Solana CLI: SOLANA_BIN, else the installer's active release, else the newest release that holds one. */
export function solanaBin() {
  if (process.env.SOLANA_BIN) return process.env.SOLANA_BIN;
  const root = `${homedir()}/.local/share/solana/install`;
  const found = [`${root}/active_release/bin`,
    ...(existsSync(`${root}/releases`) ? readdirSync(`${root}/releases`).map((r) => `${root}/releases/${r}/solana-release/bin`) : [])]
    .filter((d) => existsSync(`${d}/solana`));
  if (!found.length) throw new Error('no Solana CLI: set SOLANA_BIN');
  return found.sort((x, y) => statSync(`${y}/solana`).mtimeMs - statSync(`${x}/solana`).mtimeMs)[0];
}

export function cluster({ rpcUrl, work, stateFile, adapterPath, say = console.log }) {
  const LOCAL = /127\.0\.0\.1|localhost/.test(rpcUrl);
  const BIN = solanaBin();
  mkdirSync(`${work}/keys`, { recursive: true, mode: 0o700 });
  const statePath = `${work}/${stateFile}`;
  const state = existsSync(statePath) ? JSON.parse(readFileSync(statePath, 'utf8')) : { rpc: rpcUrl, vars: {}, done: {}, txs: [] };
  if (state.rpc !== rpcUrl) throw new Error(`${statePath} belongs to ${state.rpc}`);
  const save = () => writeFileSync(statePath, JSON.stringify(state, null, 1));

  const keyPath = (name) => `${work}/keys/${name}.json`;
  function key(name) {
    const p = keyPath(name);
    if (!existsSync(p)) writeFileSync(p, JSON.stringify([...freshKeypair()]), { mode: 0o600 });
    return checkKeypair(Uint8Array.from(JSON.parse(readFileSync(p, 'utf8'))));
  }
  const pub = (name) => publicOf(key(name));

  async function once(step, fn) {
    if (state.done[step]) { say(`  ${step.padEnd(52)} (done before)`); return state.done[step]; }
    const v = (await fn()) ?? true;
    state.done[step] = v;
    save();
    return v;
  }

  let id = 0;
  async function rpc(method, params = []) {
    for (let i = 0, wait = 500; ; i++, wait = Math.min(wait * 2, 15_000)) {
      try {
        const r = await fetch(rpcUrl, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', id: ++id, method, params }) });
        if (r.status === 429 && i < 8) { await sleep(wait); continue; }
        const j = await r.json();
        if (j.error) { const e = new Error(`${method}: ${j.error.message}`); e.logs = j.error.data?.logs; throw e; }
        return j.result;
      } catch (e) {
        if (e.logs || i >= 8 || !/fetch failed|ECONN|socket/i.test(String(e.message || e))) throw e;
        await sleep(wait);
      }
    }
  }
  const slot = async () => BigInt(await rpc('getSlot', [{ commitment: 'confirmed' }]));
  const blockhash = async () => (await rpc('getLatestBlockhash', [{ commitment: 'confirmed' }])).value.blockhash;
  const balance = async (pk) => BigInt((await rpc('getBalance', [pk, { commitment: 'confirmed' }])).value);
  async function account(pk) {
    const v = (await rpc('getAccountInfo', [pk, { encoding: 'base64', commitment: 'confirmed' }])).value;
    return v ? { owner: v.owner, lamports: BigInt(v.lamports), data: Buffer.from(v.data[0], 'base64') } : null;
  }

  function cli(args, { json = false } = {}) {
    const r = spawnSync(`${BIN}/solana`, [...args, '--url', rpcUrl, '--commitment', 'confirmed', ...(json ? ['--output', 'json'] : [])], { encoding: 'utf8' });
    if (r.status !== 0) throw new Error(`solana ${args.slice(0, 2).join(' ')}: ${(r.stderr || r.stdout).trim()}`);
    return json ? JSON.parse(r.stdout) : r.stdout.trim();
  }

  function adapter(req) {
    const r = spawnSync(adapterPath, { input: JSON.stringify(req), encoding: 'utf8' });
    const out = JSON.parse(r.stdout || '{}');
    if (!out.ok) throw new Error(`adapter ${req.method}: ${out.error || r.stderr}`);
    return out;
  }

  /** Compile (by the adapter), sign with the named keys, send; throws with the program logs when the cluster refuses. */
  async function submit(step, req, signerNames) {
    const out = adapter({ ...req, blockhash: await blockhash() });
    const { wire, signature } = signMessage(Buffer.from(out.messageBase64, 'base64'), out.signers, signerNames.map(key));
    try {
      await rpc('sendTransaction', [Buffer.from(wire).toString('base64'), { encoding: 'base64', preflightCommitment: 'confirmed' }]);
    } catch (e) {
      const err = new Error(`${step}: ${e.message}\n    ${(e.logs || []).join('\n    ')}`);
      err.logs = e.logs || [];
      throw err;
    }
    for (let t0 = Date.now(); ; await sleep(LOCAL ? 300 : 1500)) {
      const st = (await rpc('getSignatureStatuses', [[signature]])).value[0];
      if (st?.err) throw new Error(`${step}: ${signature} failed: ${JSON.stringify(st.err)}`);
      if (st && (st.confirmationStatus === 'confirmed' || st.confirmationStatus === 'finalized')) break;
      if (Date.now() - t0 > 90_000) throw new Error(`${step}: ${signature} never confirmed`);
    }
    let got = null;
    for (let i = 0; i < 10 && !got; i++) {
      got = await rpc('getTransaction', [signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0, encoding: 'json' }]);
      if (!got) await sleep(1000);
    }
    const row = { step, signature, slot: got?.slot ?? null, fee: got?.meta?.fee ?? null, cu: got?.meta?.computeUnitsConsumed ?? null, bytes: wire.length };
    state.txs.push(row);
    say(`  ${step.padEnd(52)} slot ${row.slot}  ${row.bytes} B  ${row.cu} CU  ${signature.slice(0, 12)}…`);
    return { ...row, out };
  }

  /** Once per step: a transaction that must land. */
  const send = (step, req, signerNames) => once(step, async () => {
    const { out, ...row } = await submit(step, req, signerNames);
    return { ...row, ...(out.hash ? { hash: out.hash, envelope: out.envelope } : {}), ...(out.allowance ? { allowance: out.allowance } : {}) };
  });

  /** Once per step: a transaction the cluster must refuse, with this pattern in its logs; nothing lands. */
  const refused = (step, req, signerNames, pattern) => once(step, async () => {
    try { await submit(step, req, signerNames); } catch (e) {
      const line = (e.logs || []).find((l) => pattern.test(l));
      if (!line) throw new Error(`${step}: refused, but not by ${pattern}:\n${e.message}`);
      say(`  ${step.padEnd(52)} refused: ${line.trim().slice(0, 110)}`);
      return { refused: line.trim() };
    }
    throw new Error(`${step}: it landed; it had to be refused`);
  });

  return { LOCAL, BIN, state, save, once, key, keyPath, pub, rpc, slot, blockhash, balance, account, cli, adapter, send, refused };
}
