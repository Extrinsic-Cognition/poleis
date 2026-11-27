#!/usr/bin/env node
// The proof of the aggregate vote, for the lander's gate and for any seat: the program built for SBF
// (sbpf v3), the Poleis suites on Mollusk with the runtime's own BLS12-381 syscalls (`tests/bls.rs`
// and every other suite, for the refactor of `attest`), and the client's host-side proofs of the law
// (`cargo test --lib bls`). Exit 0 only when every step passes.
//
// Run from anywhere, with any PATH (the gate's has no cargo): node poleis/attest-bls.proof.mjs
// The toolchains are the new Mac's: cargo-build-sbf from the Agave release under ~/.local/share/solana
// (platform-tools v1.54 cached under ~/.cache/solana), cargo's native builds against the 26.5 SDK with
// the 1.99.0 toolchain (the 27.0 CommandLineTools SDK's tapi files break rustc 1.99.0). The build
// cache lives in /tmp/poleis-target-{host,client}, outside the tree, kept between runs.
import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const PROGRAM = path.join(HERE, 'program', 'poleis_treasury');
const CLIENT = path.join(HERE, 'client');
const HOME = process.env.HOME || homedir();

function sbfBin() {
  const releases = path.join(HOME, '.local', 'share', 'solana', 'install', 'releases');
  const preferred = path.join(releases, 'stable-515465dd278366f3785dc75e5cce3165edbe316f', 'solana-release', 'bin');
  if (existsSync(path.join(preferred, 'cargo-build-sbf'))) return preferred;
  if (existsSync(releases)) {
    for (const d of readdirSync(releases).sort().reverse()) {
      const bin = path.join(releases, d, 'solana-release', 'bin');
      if (existsSync(path.join(bin, 'cargo-build-sbf'))) return bin;
    }
  }
  return null;
}

const env = { ...process.env, HOME };
// The lander's gate runs every proof with launchd's PATH (lander.mjs GATE_PATH: brew, /usr/local, the system's bins): rustup's cargo
// lives in ~/.cargo/bin and is on no such PATH, so the first hand of this lane (88e17a092) fell at the gate's first step,
// `cargo metadata: No such file or directory`. The script carries its own road to cargo; a seat's shell needs nothing of it.
const cargoBin = path.join(HOME, '.cargo', 'bin');
if (existsSync(path.join(cargoBin, 'cargo'))) env.PATH = `${cargoBin}:${env.PATH || '/usr/bin:/bin'}`;
const sdk = '/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk';
if (existsSync(sdk)) env.SDKROOT = sdk;
const toolchains = spawnSync('rustup', ['toolchain', 'list'], { encoding: 'utf8', env }).stdout || '';
if (/^1\.99\.0-aarch64-apple-darwin/m.test(toolchains)) env.RUSTUP_TOOLCHAIN = '1.99.0-aarch64-apple-darwin';
const bin = sbfBin();
if (bin) env.PATH = `${bin}:${env.PATH || '/usr/bin:/bin'}`;

function step(name, cmd, args, cwd, extra = {}) {
  console.log(`--- ${name}: ${cmd} ${args.join(' ')} (in ${path.relative(process.cwd(), cwd) || '.'})`);
  const r = spawnSync(cmd, args, { cwd, env: { ...env, ...extra }, stdio: 'inherit', timeout: 14 * 60_000 });
  if (r.error) throw new Error(`${name}: ${r.error.message}`);
  if (r.status !== 0) throw new Error(`${name}: exit ${r.status}${r.signal ? ` (${r.signal})` : ''}`);
}

try {
  if (!bin) throw new Error('cargo-build-sbf not found under ~/.local/share/solana/install/releases');
  step('the program for SBF', 'cargo-build-sbf', ['--arch', 'v3'], PROGRAM);
  step('the aggregate vote on Mollusk', 'cargo', ['test', '--test', 'bls'], PROGRAM, { CARGO_TARGET_DIR: '/tmp/poleis-target-host' });
  step('every suite of the program', 'cargo', ['test'], PROGRAM, { CARGO_TARGET_DIR: '/tmp/poleis-target-host' });
  step("the client's proofs of the law", 'cargo', ['test', '--lib', 'bls'], CLIENT, { CARGO_TARGET_DIR: '/tmp/poleis-target-client' });
  console.log('attest-bls proof: every step passed');
} catch (e) {
  console.error(`attest-bls proof FAILED: ${e.message}`);
  process.exit(1);
}
