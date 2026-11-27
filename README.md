# Poleis

On-chain Solana protocol, verifiable governance, charters, and \$WARM token tooling for **Extrinsic Cognition**.

> **Status:** Pre-mainnet · **PRIVATE until license & launch confirmation.**
> Before flipping this repository public, confirm the license posture noted in `poleis/provenance.json`.
> All mainnet configs (`poleis/press/params.mainnet.json`, `tools/warm-launch/params.mainnet.json`) keep explicit `PLACEHOLDER:` locks until the founding sitting.

## Architecture

Poleis is the **sovereign settlement plane** in Extrinsic Cognition's three-layer stack:

1. **[SwarmClock](https://github.com/Extrinsic-Cognition/swarmclock)** — P2P Iroh claw-to-claw (`ec/ingress/v1`) & Merkle clock (`swarmclock_book`). Coordinates signed intentions off-chain.
2. **[TraceNet](https://github.com/Extrinsic-Cognition/tracenet)** — Open Traces standard (`iroh-gossip` + `iroh-docs` + `iroh-blobs` + privacy membrane).
3. **Poleis (this repo)** — On-chain Solana settlement: treasuries, futarchy roads, war contracts, \$WARM mining, and verifiable charters.

## Layout

```
poleis/
  program/
    poleis_treasury/   Core treasury, councils, auctions, sales, NFT seats, proposals, BLS
    poleis_roads/      Futarchy decision markets (boards, roads, positions)
    poleis_wars/       On-chain war & raid state machine
    warm_mine/         $WARM mining & vault program
  client/              PDA derivation & wire client (Rust)
  door/                Pool & society door logic (Rust)
  press/               Mainnet/devnet ceremony builder & verifier (Rust)
  seal/                On-chain release vote verification (Rust)
  seat-wasm/           WASM bindings for browser/Worker verification
  charters/            Founding charters (the-dyson-society.txt, nefertiti.txt)
  ceremony/            Devnet & localnet rehearsal receipts
crates/
  poleis-war/       Pure game/war math (zero external runtime deps)
  poleis-futarchy/  Pure integer futarchy math
  poleis-stake/     Stake book math
  poleis-wallet/    Wallet keygen & derivation helpers
tools/
  solana-pda.mjs       Zero-dependency Base58 / Ed25519 curve / PDA helper
  warm-launch/         $WARM token & Meteora DLMM launch builder + tests
  warm-day0/           Day-0 lock, stream & burn verifier + tests
  burn-digest/         Burn digest verification helper
```

## PDA & Brand Safety

None of the four Solana programs bake legacy brand strings into on-chain PDA seeds or account discriminators.

## Running Tests (Standalone)

```bash
# Pure-Rust & protocol tests
cargo test --manifest-path crates/poleis-war/Cargo.toml
cargo test --manifest-path crates/poleis-futarchy/Cargo.toml
cargo test --manifest-path crates/poleis-stake/Cargo.toml
cargo test --manifest-path poleis/client/Cargo.toml
cargo test --manifest-path poleis/door/Cargo.toml --lib
cargo test --manifest-path poleis/seal/Cargo.toml --lib
cargo test --manifest-path poleis/press/Cargo.toml --lib

# Node launch & Day-0 verifier tests (zero npm dependencies)
node --test tools/warm-launch/warm-launch.test.mjs
(cd tools/warm-day0 && node --test $(find . -name '*.test.mjs'))
```

## License

- **On-chain programs (`poleis/program/*`), tooling & core crates:** [GPL-3.0](LICENSE)
- **Integrator SDKs (`poleis/client/`, `poleis/seat-wasm/`):** [Apache-2.0](LICENSE-APACHE)
- See [NOTICE](NOTICE). *(Confirm `poleis/provenance.json` before flipping this repository public.)*
