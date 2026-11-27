// The checker's only road to a cluster. Read-only by construction: a method that is not on READ_ONLY_METHODS is refused before
// anything is sent, so sendTransaction, simulateTransaction and requestAirdrop cannot be reached from any row.
//
// A transport is `async (method, params) => result`. Three exist: http (live), fixture (replays a recording and throws on
// anything it does not hold, so a test run cannot touch the network), and recording (wraps http, keeps every pair).

export const READ_ONLY_METHODS = Object.freeze([
  'getAccountInfo', 'getMultipleAccounts', 'getProgramAccounts',
  'getTokenLargestAccounts', 'getTokenAccountsByOwner',
  'getSignaturesForAddress', 'getTransaction',
  'getSlot', 'getBlockTime', 'getGenesisHash',
]);
const ALLOWED = new Set(READ_ONLY_METHODS);

export const COMMITMENT = 'finalized';

export const CLUSTERS = Object.freeze({
  mainnet: { url: 'https://api.mainnet-beta.solana.com', genesis: '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d' },
  devnet: { url: 'https://api.devnet.solana.com', genesis: 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG' },
});

export class RpcError extends Error {
  constructor(message, { method, code } = {}) { super(message); this.name = 'RpcError'; this.method = method; this.code = code; }
}

function canon(v) {
  if (Array.isArray(v)) return v.map(canon);
  if (v && typeof v === 'object') return Object.fromEntries(Object.keys(v).sort().map(k => [k, canon(v[k])]));
  return v;
}
export function callKey(method, params) { return `${method} ${JSON.stringify(canon(params ?? []))}`; }

const sleep = ms => new Promise(r => setTimeout(r, ms));

/** Live JSON-RPC over HTTPS, one call at a time, spaced, backing off on a rate limit. */
export function httpTransport(url, { fetchImpl = globalThis.fetch, gapMs = 150, retries = 6, timeoutMs = 30_000 } = {}) {
  let last = 0, id = 0;
  return async (method, params) => {
    for (let attempt = 0; ; attempt++) {
      const wait = last + gapMs - Date.now();
      if (wait > 0) await sleep(wait);
      last = Date.now();
      let res;
      try {
        res = await fetchImpl(url, {
          method: 'POST', headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ jsonrpc: '2.0', id: ++id, method, params }), signal: AbortSignal.timeout(timeoutMs),
        });
      } catch (e) {
        if (attempt >= retries) throw new RpcError(`${method}: ${e.message}`, { method });
        await sleep(500 * 2 ** attempt); continue;
      }
      // An endpoint that does not serve a method at all answers 429 with a limit of 0 for it: waiting will not help.
      if (res.status === 429 && res.headers.get('x-ratelimit-method-limit') === '0') throw new RpcError(`${method}: this endpoint does not serve the method (its limit for it is 0)`, { method, code: 'METHOD_DISABLED' });
      if (res.status === 429 || res.status >= 500) {
        if (attempt >= retries) throw new RpcError(`${method}: HTTP ${res.status} after ${attempt + 1} tries`, { method, code: res.status });
        const after = Number(res.headers.get('retry-after')) * 1000;
        await sleep(Number.isFinite(after) && after > 0 ? after : 600 * 2 ** attempt); continue;
      }
      let body;
      try { body = await res.json(); } catch { throw new RpcError(`${method}: HTTP ${res.status}, body is not JSON`, { method, code: res.status }); }
      if (body.error) {
        if (body.error.code === 429 && attempt < retries) { await sleep(600 * 2 ** attempt); continue; }
        throw new RpcError(`${method}: ${body.error.message} (${body.error.code})`, { method, code: body.error.code });
      }
      return body.result;
    }
  };
}

/** Replays a recording ({ calls: { key: result } }). An error that was recorded is thrown again; a call that was not recorded throws. */
export function fixtureTransport(recording) {
  const calls = recording.calls || {};
  return async (method, params) => {
    const key = callKey(method, params);
    if (!Object.hasOwn(calls, key)) throw new RpcError(`the fixture holds no answer for: ${key}`, { method, code: 'NOT_RECORDED' });
    const hit = calls[key];
    if (hit && typeof hit === 'object' && hit.__rpcError) throw new RpcError(hit.__rpcError.message, { method, code: hit.__rpcError.code });
    return structuredClone(hit);
  };
}

/** Wraps a transport and writes every answer (or error) into `sink`, keyed as fixtureTransport reads them. */
export function recordingTransport(inner, sink) {
  return async (method, params) => {
    const key = callKey(method, params);
    try {
      const result = await inner(method, params);
      sink[key] = structuredClone(result);
      return result;
    } catch (e) {
      sink[key] = { __rpcError: { message: e.message, code: e.code ?? null } };
      throw e;
    }
  };
}

export function createRpc(transport) {
  const stats = { calls: 0, byMethod: {} };
  const call = async (method, params = []) => {
    if (!ALLOWED.has(method)) throw new RpcError(`refused: ${method} is not a read-only method this checker may call`, { method, code: 'REFUSED' });
    stats.calls++; stats.byMethod[method] = (stats.byMethod[method] || 0) + 1;
    return transport(method, params);
  };

  const unwrapAccount = (v) => v && ({ owner: v.owner, lamports: v.lamports, executable: v.executable, data: Buffer.from(v.data[0], 'base64') });

  return {
    call, stats,
    /** One account's raw bytes, or null when it does not exist. */
    async account(address) {
      const r = await call('getAccountInfo', [address, { encoding: 'base64', commitment: COMMITMENT }]);
      return unwrapAccount(r.value);
    },
    /** Many accounts in order (null for the missing), a hundred an ask. */
    async accounts(addresses) {
      const out = [];
      for (let i = 0; i < addresses.length; i += 100) {
        const r = await call('getMultipleAccounts', [addresses.slice(i, i + 100), { encoding: 'base64', commitment: COMMITMENT }]);
        out.push(...r.value.map(unwrapAccount));
      }
      return out;
    },
    /** The first `length` bytes of an account from `offset` (a program-data account is megabytes; its header is 45 bytes). */
    async slice(address, offset, length) {
      const r = await call('getAccountInfo', [address, { encoding: 'base64', commitment: COMMITMENT, dataSlice: { offset, length } }]);
      return unwrapAccount(r.value);
    },
    async largestAccounts(mint) {
      const r = await call('getTokenLargestAccounts', [mint, { commitment: COMMITMENT }]);
      return r.value;
    },
    async tokenAccountsByOwner(owner, mint) {
      const r = await call('getTokenAccountsByOwner', [owner, { mint }, { encoding: 'base64', commitment: COMMITMENT }]);
      return r.value.map(v => ({ address: v.pubkey, ...unwrapAccount(v.account) }));
    },
    async programAccounts(programId, filters, dataSlice) {
      const cfg = { encoding: 'base64', commitment: COMMITMENT, filters };
      if (dataSlice) cfg.dataSlice = dataSlice;
      const r = await call('getProgramAccounts', [programId, cfg]);
      return r.map(v => ({ address: v.pubkey, ...unwrapAccount(v.account) }));
    },
    async signatures(address, limit) {
      return call('getSignaturesForAddress', [address, { limit, commitment: COMMITMENT }]);
    },
    async transaction(signature) {
      return call('getTransaction', [signature, { encoding: 'jsonParsed', commitment: COMMITMENT, maxSupportedTransactionVersion: 0 }]);
    },
    /** The cluster's clock: its slot and that slot's block time. Rows compare chain times to this, never to the local clock. */
    async clock() {
      const slot = await call('getSlot', [{ commitment: COMMITMENT }]);
      let time = null;
      for (let back = 0; back < 8 && time === null; back++) {
        try { time = await call('getBlockTime', [slot - back]); } catch (e) { if (e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e; }
      }
      return { slot, time };
    },
    async genesis() { return call('getGenesisHash', []); },
  };
}
