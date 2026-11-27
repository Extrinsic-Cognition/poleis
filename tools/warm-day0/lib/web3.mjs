// @solana/web3.js, loaded the way the house's one other Node tool loads it (embertide-app/ui/intro/dyson-society/tools/
// open-whitelist-sale.mjs): borrowed from the Poleis client's node_modules through createRequire, POLEIS_JS overriding the path.
// A folder of this tool's own (`npm install` here, node_modules is ignored by the house) is the second road, for a clone that has
// no ~/code-ec/poleis-protocol.
//
// Only PublicKey leaves this file: base58, and program-derived addresses. Keypair, Transaction and Connection are never
// imported by the checker, so it cannot hold a key, build a transaction or reach a cluster except through lib/rpc.mjs.
import { createRequire } from 'node:module';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const roads = [
  process.env.POLEIS_JS || join(homedir(), 'code-ec/poleis-protocol/js'),
  join(here, '..'),
];

function load() {
  const tried = [];
  for (const road of roads) {
    try {
      const need = createRequire(join(road, 'package.json'));
      const web3 = need('@solana/web3.js');
      let version = 'unknown';
      try { version = need('@solana/web3.js/package.json').version; } catch { /* the package hides its package.json */ }
      return { PublicKey: web3.PublicKey, version, from: road };
    } catch (e) {
      tried.push(`${road}: ${e.code || e.message}`);
    }
  }
  throw new Error(`@solana/web3.js not found. Tried:\n  ${tried.join('\n  ')}\nSet POLEIS_JS to a folder whose node_modules holds it, or run npm install in ${join(here, '..')}.`);
}

const loaded = load();
export const PublicKey = loaded.PublicKey;
export const WEB3_VERSION = loaded.version;

export const DEFAULT_PUBKEY = '11111111111111111111111111111111';   // 32 zero bytes: Pubkey::default(), also the System Program

export function isAddress(s) {
  if (typeof s !== 'string' || s.length < 32 || s.length > 44) return false;
  try { return new PublicKey(s).toBase58() === s; } catch { return false; }
}

export function b58(bytes) { return new PublicKey(bytes).toBase58(); }

export function keyBytes(address) { return new PublicKey(address).toBuffer(); }

const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
/** Base58 of any length (instruction data in a parsed transaction). PublicKey only decodes 32 bytes. */
export function base58Decode(s) {
  let n = 0n;
  for (const ch of s) {
    const i = ALPHABET.indexOf(ch);
    if (i < 0) throw new Error(`not base58: ${s.slice(0, 16)}`);
    n = n * 58n + BigInt(i);
  }
  const body = [];
  for (; n > 0n; n >>= 8n) body.push(Number(n & 0xffn));
  let zeros = 0;
  while (zeros < s.length && s[zeros] === '1') zeros++;
  return Buffer.from([...new Array(zeros).fill(0), ...body.reverse()]);
}

export function pda(seeds, programId) {
  return PublicKey.findProgramAddressSync(seeds.map(s => (typeof s === 'string' ? Buffer.from(s) : s)), new PublicKey(programId))[0].toBase58();
}
