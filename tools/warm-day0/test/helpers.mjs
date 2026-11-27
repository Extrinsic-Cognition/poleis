// Shared by the tests: where the fixtures are, and a wall against the network.
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const root = join(dirname(fileURLToPath(import.meta.url)), '..');
export const fixturePath = name => join(root, 'fixtures', `${name}.json`);
export const fixture = name => JSON.parse(readFileSync(fixturePath(name), 'utf8'));

/** No test reaches a cluster: fetch throws for the whole process, and WARM_DAY0_RPC cannot point anywhere. */
export function noNetwork() {
  globalThis.fetch = () => { throw new Error('a test tried to reach the network'); };
  delete process.env.WARM_DAY0_RPC;
}

/** An account's bytes as a fixture recorded them, whichever call fetched it whole. */
export function accountOf(fx, address) {
  for (const [key, answer] of Object.entries(fx.calls)) {
    const params = JSON.parse(key.slice(key.indexOf(' ') + 1));
    if (params[1]?.dataSlice || params[1]?.encoding !== 'base64') continue;
    let value = null;
    if (key.startsWith('getAccountInfo ') && params[0] === address) value = answer.value;
    if (key.startsWith('getMultipleAccounts ') && params[0].includes(address)) value = answer.value[params[0].indexOf(address)];
    if (value) return { owner: value.owner, lamports: value.lamports, data: Buffer.from(value.data[0], 'base64') };
  }
  throw new Error(`the fixture holds no whole account ${address}`);
}

export const big = (key, value) => (typeof value === 'bigint' ? value.toString() : value);
