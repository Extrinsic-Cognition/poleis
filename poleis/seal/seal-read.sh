#!/bin/sh
# seal-read.sh · A RELEASE IS A VOTE (Nov 23 2026), the lander's child: is <commit> sealed RATUM by the house's polis? The read is Rust, poleis-seal's
# `read` (src/main.rs); this only builds it once in the served house (niced, two jobs, again only when its source or the client's is newer than the
# binary: a stale one would judge by an old law) and runs it. Exit 0 sealed, 1 not, 2 not read; the last line of stdout is the JSON whatever the exit.
# COST: one getMultipleAccounts for the eight nonces' proposals and sessions; a seal found, one getSignaturesForAddress and two getTransaction; no
# port, nothing at rest.
# THE SWARM SHIPS THE UPDATE (Nov 23 2026): `newest <file> --road R` is poleis-seal's own newest, the newest of the file's commits (one a line,
# newest first) the polis sealed: six commits' accounts a call, from the newest down.
#   sh poleis/seal/seal-read.sh <commit> --road <road.json> [--json]
#   sh poleis/seal/seal-read.sh newest <commits> --road <road.json> [--json]
set -eu
PATH="$HOME/.cargo/bin:$PATH"   # the lander runs under launchd with no cargo on its PATH
here=$(cd "$(dirname "$0")" && pwd); bin="$here/target/release/poleis-seal"
if [ ! -x "$bin" ] || [ -n "$(find "$here/src" "$here/../client/src" "$here/Cargo.toml" -newer "$bin" 2>/dev/null | head -1)" ]; then
  (cd "$here" && SDKROOT="$(xcrun --show-sdk-path 2>/dev/null || true)" nice -n 19 cargo build --release --locked --quiet -j 2 --bin poleis-seal) >&2 \
    || { echo "{\"sealed\":false,\"read\":false,\"why\":\"the reader did not build (cd poleis/seal && cargo build --release --locked)\"}"; exit 2; }
fi
[ "${1:-}" = newest ] && exec "$bin" "$@"
exec "$bin" read "$@"
