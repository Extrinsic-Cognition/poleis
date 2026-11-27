// keygen-cross.proof.mjs: HER WORDS MAKE THE OFFICIAL KEYGEN'S ADDRESS, both ways (Oct 14 2026). Throwaway keys only; no word, no key and no
// address of any round is printed; the scratch folder goes at the end, a failure included.
//   1. the official keygen draws (`solana-keygen new --no-outfile --no-bip39-passphrase --word-count 24 --derivation-path`, an empty path being
//      m/44'/501'/0'/0'): its words go to this crate's `derive` on stdin, and the address must be the keygen's pubkey;
//   2. this crate's `new` draws (in a 0700 scratch folder): its words go to `solana-keygen pubkey prompt://?key=0/0` through a pty made by
//      expect(1) (keygen-pty.exp, the words on its stdin: the keygen reads a seed phrase from a terminal only, and script(1) wants a terminal on
//      its own stdin), and the keygen's pubkey must be the address `new` gave and public.json holds;
//   then `check` passes on every folder `new` made.
// It also times this crate's `new` (spawn to exit, files fsynced) and `derive`, for LANDING.md.
// Run: node poleis-wallet/keygen-cross.proof.mjs [rounds=10]   needs target/release/poleis-wallet (cargo build --release --offline) and
// solana-keygen (Agave) on PATH. COST: ~4 processes a round; no network; nothing left behind.
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const BIN = process.env.EMBERTIDE_WALLET_BIN || path.join(HERE, 'target/release/poleis-wallet');
const ROUNDS = Number(process.argv[2] || 10);
const B58 = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'keygen-cross-'));
fs.chmodSync(scratch, 0o700);
const wallets = path.join(scratch, 'wallets');
const ms = (t0) => Number(process.hrtime.bigint() - t0) / 1e6;
const median = (a) => { const s = [...a].sort((x, y) => x - y); return s[Math.floor(s.length / 2)]; };
const run = (cmd, args, input) => spawnSync(cmd, args, { input, encoding: 'utf8', env: { PATH: process.env.PATH, HOME: scratch }, timeout: 20000 });
let failures = 0;
const fail = (why) => { failures++; console.log(`FAIL ${why}`); };
const times = { new: [], derive: [] };
try {
  const version = run('solana-keygen', ['--version']).stdout.trim();
  for (let i = 0; i < ROUNDS; i++) {
    // 1. the keygen's words -> our derive
    const drawn = run('solana-keygen', ['new', '--no-outfile', '--no-bip39-passphrase', '--word-count', '24', '--derivation-path']);
    const pub = /pubkey: ([1-9A-HJ-NP-Za-km-z]{32,44})/.exec(drawn.stdout)?.[1];
    const lines = drawn.stdout.split('\n'), at = lines.findIndex((l) => /^Save this seed phrase/.test(l));
    const words = at >= 0 ? lines[at + 1].trim() : '';
    if (drawn.status !== 0 || !pub || words.split(' ').length !== 24) { fail(`round ${i}: the keygen did not draw (exit ${drawn.status})`); continue; }
    let t0 = process.hrtime.bigint();
    const ours = run(BIN, ['derive'], words + '\n'); times.derive.push(ms(t0));
    const derived = ours.status === 0 ? JSON.parse(ours.stdout).address : null;
    if (derived !== pub) fail(`round ${i}: derive from the keygen's words is not the keygen's pubkey`);
    // 2. our words -> the keygen's pubkey
    t0 = process.hrtime.bigint();
    const made = run(BIN, ['new', wallets, '--birth', `cross${i}`]); times.new.push(ms(t0));
    const line = made.status === 0 ? JSON.parse(made.stdout) : null;
    if (!line || !B58.test(line.address)) { fail(`round ${i}: new made nothing (exit ${made.status}: ${made.stderr.trim()})`); continue; }
    const home = path.join(wallets, line.address);
    const ourWords = fs.readFileSync(path.join(home, 'words'), 'utf8').trim();
    const told = run('expect', ['-f', path.join(HERE, 'keygen-pty.exp')], ourWords + '\n');
    const theirs = (told.stdout.match(/[1-9A-HJ-NP-Za-km-z]{32,44}/g) || []).pop();
    if (theirs !== line.address) fail(`round ${i}: the keygen's pubkey from our words is not our address`);
    const public_ = JSON.parse(fs.readFileSync(path.join(home, 'public.json'), 'utf8'));
    if (public_.address !== line.address || public_.path !== "m/44'/501'/0'/0'" || public_.birth !== `cross${i}`) fail(`round ${i}: public.json is not the answer`);
    const checked = run(BIN, ['check', home]);
    if (checked.status !== 0 || JSON.parse(checked.stdout).address !== line.address) fail(`round ${i}: check refused (${checked.stderr.trim()})`);
    if (made.stdout.includes(ourWords.split(' ').slice(0, 2).join(' ')) || made.stderr.length) fail(`round ${i}: new said more than its public line`);
  }
  console.log(JSON.stringify({ proof: 'keygen-cross', keygen: version, rounds: ROUNDS, failures,
    new_ms: { median: +median(times.new).toFixed(2), max: +Math.max(...times.new).toFixed(2) },
    derive_ms: { median: +median(times.derive).toFixed(2), max: +Math.max(...times.derive).toFixed(2) },
    binary_bytes: fs.statSync(BIN).size }));
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}
process.exitCode = failures ? 1 : 0;
