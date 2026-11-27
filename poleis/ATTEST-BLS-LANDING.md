# ATTEST_BLS: the aggregate vote, now that the syscalls are live

His order, Sept 1 2026 23:39 WEST: "keep bls stuff in backlog mentioning that when it's switched we'll switch";
Sept 3 01:17: "do the optimizations we can and keep as a priority to go into BLS as soon as Solana has it".
Solana has it: the BLS12-381 syscalls (SIMD-0388, feature `enable_bls12_381_syscall`,
`b1sgUiJ3qu7hYm3tNDyyqZNQd6gLGJmJppnLNa93PCQ`) activated on mainnet at slot 425,952,004, epoch 986,
2026-06-12 09:14:45Z, and on devnet at slot 457,488,000 (read from both RPCs on Oct 22 2026). The house's
September docs waited for "Agave 4.3, October": a stale read of a Foundation page; the wait ended in June.
Alpenglow, the BLS votes in consensus, switched at epoch 1064 on Oct 21 2026; it is not what this lane needs.

## Cost (the line this lane collapses)

| | before (Ed25519 precompile) | after (`attest_bls`) |
|---|---|---|
| base fee of a batch of n votes | (n + 1) × 5,000 lamports: the runtime charges every envelope the precompile verifies as a signature (`poleis-protocol/docs/localnet-rehearsal.md`, measured) | 5,000 lamports: one signature, the crank's, whatever n |
| base fee per vote | 5,000 lamports | 5,000 / n lamports |
| a million votes a day | about 5 SOL a day | about 5 / n SOL a day (n the voices per batch) |
| compute of the program, predicted | about 9,000 CU for five votes (the rehearsal) | 8,832 (the cofactor chain: 63 + 6 G1 additions at 128 CU) + 38,468 (one pairing of two pairs: 25,445 + 13,023) + 203 × (n − 1) (the keys' sum) + the landing of n voices |
| compute, measured on Mollusk (the runtime's own syscalls, Agave 4.2.2) | | see the table below |
| bytes per batch | 2 + 14n + 32n + 64n + 55 (the precompile) + 41 + 33n (attest) | 189 + 32n (one instruction; the keys as accounts) |

The fee is the number he feels: a vote cost one signature fee whatever the batch, and now a side of any
size costs one. The compute is larger per instruction and smaller per vote past a handful of voices; it is
paid in compute budget, not in signatures.

| instruction | keys | compute units, measured (Mollusk, the runtime's syscalls, Agave 4.2.2) |
|---|---|---|
| `register_bls_key` | 1 | 60,995 to 68,161 (the search for the key account's bump varies by holder) |
| `attest_bls` | 1 | 62,609 |
| `attest_bls` | 2 | 63,713 to 64,170 |
| `attest_bls` | 3 | 64,110 to 65,718 |
| `attest_bls` | 15 (the twelve chairs and the floor of six, one instruction) | 79,819 |
| each further key | | about 1,200 (203 the G2 addition, the rest the landing of its voice) |

Read against the default budget of 200,000 compute units per instruction: about 110 keys before a
compute-budget instruction is needed. The side of fifteen above cost 16 × 5,000 = 80,000 lamports in four
transactions on the precompile road (five envelopes fit a legacy transaction); it costs 5,000 lamports in
one transaction here. The whole suite that proves it: `tests/bls.rs`, 10 of 10; the program's other
suites unchanged (unit 45, adversarial 18, assembly 12, council 30, fuzz 1, fuzz_council 1, mathlaw 2,
nft 22, nft_governance 6, poleis 4); the client's 19.


## What landed

- `poleis/program/poleis_treasury/src/bls.rs`: the law. Keys in G2, signatures and the message point in
  G1. The message point by try-and-increment, the one hash-to-curve the syscalls can carry (no map-to-curve,
  inversion or square root runs in BPF): `x` from sha256 of (dst ‖ msg ‖ ctr), the signer finds the first
  counter whose `x` is below `p` and on the curve, takes the smaller root `y`, and clears the cofactor by
  `h_eff = 1 − z`; on chain the crank names `ctr` and `y`, the program recomputes `x`, checks the two bounds
  and rebuilds `h_eff · (x, y)` by 69 G1 additions (the ADD syscall validates field and curve and skips the
  subgroup, which is exactly what lets a raw curve point through; MUL would refuse it). One pairing of two
  pairs, e(H, pk) · e(−S, G2) == 1, read against the identity (575 zero bytes, then one).
- `src/ix/bls.rs`: `register_bls_key` (tag 54: a member's key with its proof of possession over
  unit ‖ holder ‖ key, verified by the same pairing, which also validates the key: field, curve, subgroup;
  rotation under the same account) and `attest_bls` (tag 27, reserved since September: the keys passed add
  up, one pairing verifies the sum, every key's holder lands as a voice of `attest` lands).
- `src/ix/proposal.rs`: the landing of one voice (a row once at seats × weight, the live chairs once each)
  is now `land()`, the one law for both roads; `attest` calls it for every envelope the precompile verified,
  `attest_bls` for every key in the sum. The Ed25519 road is byte for byte what it was.
- `src/state.rs`: `BlsKey` (disc 25; `["blskey", unit, holder]`, 235 bytes: unit, holder, the key
  uncompressed, bump). `src/error.rs`: `BadBlsPoint` 71, `BadBlsSignature` 72.
- `poleis/client/src/bls.rs`: keys from a seed, the message point, signatures and their sum, the proof,
  `register_bls_key` / `registration`, `attest_bls` / `attest_bls_side`, the key account's view;
  `examples/bls_vectors.rs` prints the reference vectors for other language clients.

## The proofs (named in the commit)

1. `cargo test --lib bls` in `poleis/client`: the chain of 69 additions through the runtime's own
   implementation of the ADD syscall (`solana-bls12-381-syscall`) equals the library's scalar product;
   the message point is in G1, the smaller root, the first counter; the aggregate verifies against the
   summed keys through the runtime's own pairing, a missing signer or another envelope does not; the proof
   of possession binds the holder and the unit; the G2 generator and the identity bytes.
2. `cargo build-sbf --arch v3` then `cargo test --test bls` in `poleis/program/poleis_treasury`
   (Mollusk, the real syscalls): registration with its proof and a rotation; a proof signed by another key
   (the rogue-key road), over another unit, or a misnamed point refused; a stranger refused; one aggregate
   lands every signer once and again counts nothing; a second aggregate counts only the new voices; an
   aggregate over another envelope, a point taken from the crank, a missing signer refused; a key that is
   not a key refused; the twelve chairs once each through a fifteen-key aggregate; the two roads count one
   holder once; a closed session takes no aggregate.
3. `cargo test` in `poleis/program/poleis_treasury`, the whole suite, for the refactor of `attest`.
4. All three in one script for the lander's gate and for any seat: `node poleis/attest-bls.proof.mjs`
   (the toolchains of the new Mac; the build cache in `/tmp/poleis-target-{host,client}`, outside the tree).
   The lander refused the first hand (88e17a092) at 2026-10-22T20:41Z at this script's first step: the gate runs every
   proof with launchd's PATH (`lander.mjs` GATE_PATH), which has no `~/.cargo/bin`, and cargo-build-sbf fell at
   `cargo metadata: No such file or directory`. The script now puts `~/.cargo/bin` on its own PATH; the proof of the fix
   is the same script run under the gate's PATH on a fresh SBF build directory: every step passed (Oct 22 2026 21:47 WEST).

## What Mollusk could not prove, and his line

- The fee charged by a validator (Mollusk meters compute, not fees). It is the runtime's arithmetic,
  `signatures × lamports_per_signature`, and `attest_bls` carries one signature. The measurement on a
  validator is one run of the localnet rehearsal with a BLS side added to its client: the JS client of
  `poleis-protocol/js` has no BLS signer yet (a port of `client/src/bls.rs`, owed).
- The transaction's size: 189 bytes of data plus 32 per key as an account; a legacy transaction carries
  about 23 keys, a v0 transaction with a lookup table carries a society.

## Owed

- The JS client's BLS side (noble-curves carries BLS12-381 and the field arithmetic the counter rule needs).
- The devnet deployment of this program (the deploy ceremony is his hand; the program id is unchanged).
