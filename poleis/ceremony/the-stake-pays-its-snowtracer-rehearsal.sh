#!/bin/sh
# THE STAKE PAYS ITS SNOWTRACER, rehearsed (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider joined with lane/the-stake-earns; his "Yes, build
# them" at 13:25 UTC to "Build the three missing links now?"). One local validator holding this tree's Poleis (upgradeable, a stand-in deployer its
# upgrade authority, which founds root 0) and mainnet's stake pool program, its epochs 32 slots long, then the joint half of poleis/press/tests/light.rs:
# root 0 and her pool born by root 0's own act (act 5); act 9, the engine's staker, and act 10, root 0's plan of her token, voted and executed through
# Poleis; a wish and root 0's road into its well; two stakers (one light to the wish, one none); the engine places her stake on this validator's own vote
# account (the rehearsal params' rule); epochs turn until the validator's reward is minted as fee into root 0's vault (no transfer: the yield is the
# chain's own); her book read twice by its own hand; the reader and the pour; the wish's rider; the rider's draw to SOL through a pool that has a validator.
#   sh poleis/ceremony/the-stake-pays-its-snowtracer-rehearsal.sh <the stake pool program's .so> [the certificate's path]
# The .so is `solana program dump SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy <file> --url mainnet`, written once and held to the launch's fingerprint;
# other bytes, and nothing starts. Nothing is read from or sent to any chain but this machine's validator. Stand-in keys only. Everything at nice 19; the
# validator ends with the script, and its ledger, the books, the deployer's key and the build of Poleis are removed. CARGO_TARGET_DIR is taken when set.
set -eu
say() { printf '%s %s\n' "$(date -u '+%H:%M:%S')" "$*"; }
so=${1:-}
[ -n "$so" ] || { say "usage: sh poleis/ceremony/the-stake-pays-its-snowtracer-rehearsal.sh <the stake pool program's .so> [the certificate's path]"; exit 2; }
cert=${2:-}
case "$cert" in ""|/*) ;; *) cert="$(pwd)/$cert" ;; esac
want=${LIGHT_PROGRAM_SHA256:-d18a2cdc0e0a52f55cb363cf517a4e72e9fca4fd6950c12897ee13c2385c52ac}
program=SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy
here=$(cd "$(dirname "$0")/../.." && pwd)
begun=$(date +%s)

PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
export PATH
sdk=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk
if [ -z "${SDKROOT:-}" ] && [ -d "$sdk" ]; then SDKROOT=$sdk; export SDKROOT; fi
release=""
for d in "$HOME"/.local/share/solana/install/releases/*/solana-release; do
  [ -x "$d/bin/solana-test-validator" ] && [ -x "$d/bin/cargo-build-sbf" ] || continue
  if [ -z "$release" ] || [ "$d/bin/solana-test-validator" -nt "$release/bin/solana-test-validator" ]; then release=$d; fi
done
[ -n "$release" ] || { say "no Solana release with solana-test-validator and cargo-build-sbf under ~/.local/share/solana/install/releases: not started"; exit 2; }
PATH="$release/bin:$PATH"
export PATH

have=$(python3 -I -c 'import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], "rb").read().rstrip(b"\0")).hexdigest())' "$so")
[ "$have" = "$want" ] || { say "the program's executable bytes are sha256 $have, not $want: not started"; exit 2; }
poleis="$here/poleis/program/poleis_treasury"
id=$(sed -n 's/^[^/]*declare_id!("\([1-9A-HJ-NP-Za-km-z]*\)");.*/\1/p' "$poleis/src/lib.rs" | head -n 1)
[ -n "$id" ] || { say "Poleis declares no id: not started"; exit 2; }
say "the stake pool program: $so, sha256 $have; Poleis $id; the release: $(solana-test-validator --version)"

inuse=$(lsof -nP -iTCP -iUDP 2>/dev/null | awk 'NR > 1 { n = $9; sub(/->.*/, "", n); sub(/.*:/, "", n); if (n ~ /^[0-9]+$/) print n }' | sort -un)
free() { for p in $(seq "$1" "$2"); do echo "$inuse" | grep -qx "$p" && return 1; done; return 0; }
base=""
for b in 27600 28700 29800 21000 22100 23200; do
  if free "$b" "$((b + 40))"; then base=$b; break; fi
done
[ -n "$base" ] || { say "no span of free ports: not started"; exit 2; }
rpc=$base; faucet=$((base + 2)); gossip=$((base + 4)); range="$((base + 10))-$((base + 40))"

work=$(mktemp -d "${TMPDIR:-/tmp}/the-snowtracer.XXXXXX")
vpid=""
end() { [ -n "$vpid" ] && { kill "$vpid" 2>/dev/null || true; wait "$vpid" 2>/dev/null || true; }; rm -rf "$work"; }
trap end EXIT INT TERM

say "building Poleis as its tests build it, and the book's hand"
(cd "$poleis" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET -u CARGO_TARGET_DIR \
  nice -n 19 cargo-build-sbf --offline --manifest-path "$poleis/Cargo.toml" --sbf-out-dir "$work/deploy") > "$work/build.out" 2>&1 \
  || { tail -n 20 "$work/build.out"; say "Poleis's build failed: not started"; exit 2; }
rm -f "$work"/deploy/*-keypair.json
nice -n 19 cargo build --offline --locked --manifest-path "$here/poleis-stake/Cargo.toml" > "$work/book.out" 2>&1 \
  || { tail -n 20 "$work/book.out"; say "the book's build failed: not started"; exit 2; }
book="${CARGO_TARGET_DIR:-$here/poleis-stake/target}/debug/poleis-stake"
[ -x "$book" ] || { say "no book's hand at $book: not started"; exit 2; }

# ONLY THE DEPLOYER FOUNDS ROOT 0: Poleis is loaded upgradeable, its upgrade authority a stand-in made here, which founds root 0 as the launch's
# deployer does (the key file lives in this run's folder and is removed with it)
solana-keygen new --no-bip39-passphrase --silent --force --outfile "$work/deployer.json" > /dev/null
: > "$work/no-cli-config.yml"
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${LIGHT_ALARM:-1800}" solana-test-validator --reset --quiet --ledger "$work/ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc" --faucet-port "$faucet" --gossip-port "$gossip" --dynamic-port-range "$range" --slots-per-epoch 32 \
  --config "$work/no-cli-config.yml" --bpf-program "$program" "$so" \
  --upgradeable-program "$id" "$work/deploy/poleis_treasury.so" "$work/deployer.json" > "$work/validator.out" 2>&1 &
vpid=$!
up=""
n=0
while [ $n -lt 120 ]; do
  kill -0 "$vpid" 2>/dev/null || break
  if curl -s --max-time 2 -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' "http://127.0.0.1:$rpc" 2>/dev/null | grep -q '"result":"ok"'; then
    up=1
    break
  fi
  sleep 1
  n=$((n + 1))
done
[ -n "$up" ] || { say "the validator did not come up in 120 s:"; tail -20 "$work/validator.out"; exit 1; }
say "the validator: http://127.0.0.1:$rpc, 32 slots an epoch; the stake pool program at $program, Poleis at $id"

SNOWTRACER_VALIDATOR="http://127.0.0.1:$rpc" LIGHT_BOOK="$book" LIGHT_DIR="$work" LIGHT_DEPLOYER="$work/deployer.json" LIGHT_PROGRAM_SHA256="$want" LIGHT_CERTIFICATE="$cert" \
  nice -n 19 cargo test --offline --locked --manifest-path "$here/poleis/press/Cargo.toml" --test light -- --nocapture --test-threads 1 the_stake_pays_its_snowtracer
say "the stake paid its snowtracer in $(( $(date +%s) - begun )) s${cert:+: its certificate $cert}"
