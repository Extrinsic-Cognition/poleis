# Mainnet Engine & Light Defaults (`lane/mainnet-engine-and-light-defaults`)

## His Word (Nov 26 2026 22:20 UTC)
Asked whether to lock the 3 remaining re-votable starting defaults in `poleis/press/params.mainnet.json` (`10%` reserve, top `0%`-commission mainnet validators, `100 SOL/month` yield cap):
> "i dont fucking know go if you think is besr"

## Changes (`JavaScript: 0`)
1. **`poleis/press/params.mainnet.json`**:
   - `engine.reserveBp`: `1000` (10% instant-unstake reserve; re-votable anytime).
   - `engine.validators`: 10 live, non-delinquent, `0%`-commission Solana mainnet validators queried via `getVoteAccounts` on `https://api.mainnet-beta.solana.com` (epoch 1096/1097, 1.38M–2.49M SOL activated stake each, equal weight `1`).
   - `light.planCap`: `100000000000` (100 SOL of pool yield tokens per 30-day plan period — covers ~20,000 SOL staked without throttling Snowtracer payouts; re-votable anytime).
   - All sitting-generated keys (`floor.issuer`, `posiverse.agents`, `warm.opening`, `warm.miner`, `market.crediter`, `engine.staker`) remain `PLACEHOLDER`s so `poleis/press` generates and verifies them at Sitting 1.
2. **`poleis/press/tests/{params,engine,light}.rs`**:
   - Updated mainnet-file assertions to reflect that `engine.reserveBp`, `engine.validators`, and `light.planCap` are now set while retaining explicit placeholder-refusal tests.

## Proofs
- `cargo test --manifest-path poleis/press/Cargo.toml` — PASS
- `node --test tools/embertide-mesh/tests/the-javascript-only-shrinks.test.mjs tools/embertide-mesh/tests/the-look-is-a-law.test.mjs` — PASS (0 JS lines changed)
