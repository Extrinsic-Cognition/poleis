#!/bin/sh
# THE EPOCH TURNS THE CRANKS, rehearsed (Nov 25 2026, lane/the-epoch-turns-the-cranks; his "Yes, build it" to "Build what runs the two cranks once
# an epoch, while the lanes wait?"). One local validator, 32 slots an epoch (where the launch rehearsal's third run lost a send at a boundary), this
# tree's Poleis upgradeable under a stand-in deployer that founds root 0, mainnet's stake pool program; then poleis/press/tests/light.rs
# the_epoch_turns_the_cranks_on_a_validator: the joint rehearsal's world (her pool by act 5, acts 9 and 10, a wish and root 0's road into its well,
# two stakers, one light to the wish), then `press epoch --stay` started as its own process with no crank run by hand, carrying the epochs by
# itself, a staker leaving in a middle one; read back per epoch; then stopped, and nothing of it left running. A MISSED EPOCH'S LIGHT IS POURED
# LATER: by default (EPOCH_STOP=1) the runner's process is ended for two whole epochs in the middle and started again, and the wish's well must end
# with every pour the runner made, none left owed; EPOCH_STOP=0 is the run that never stops.
#   sh poleis/ceremony/the-epoch-rehearsal.sh <the stake pool program's .so> [the certificate's path]
# Nothing is read from or sent to any chain but this machine's validator. Stand-in keys only. Everything at nice 19; the validator ends with the
# script, and its ledger, the sitting's folder (the runner's keys and its book), the deployer's key and the build of Poleis are removed.
set -eu
say() { printf '%s %s\n' "$(date -u '+%H:%M:%S')" "$*"; }
so=${1:-}
[ -n "$so" ] || { say "usage: sh poleis/ceremony/the-epoch-rehearsal.sh <the stake pool program's .so> [the certificate's path]"; exit 2; }
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

work=$(mktemp -d "${TMPDIR:-/tmp}/the-epoch.XXXXXX")
vpid=""
end() { pkill -f "press epoch --stay --dir $work" 2>/dev/null || true; [ -n "$vpid" ] && { kill "$vpid" 2>/dev/null || true; wait "$vpid" 2>/dev/null || true; }; rm -rf "$work"; }
trap end EXIT INT TERM

say "building Poleis as its tests build it, and the book's hand"
(cd "$poleis" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET -u CARGO_TARGET_DIR \
  nice -n 19 cargo-build-sbf --offline --manifest-path "$poleis/Cargo.toml" --sbf-out-dir "$work/deploy") > "$work/build.out" 2>&1 \
  || { tail -n 20 "$work/build.out"; say "Poleis's build failed: not started"; exit 2; }
rm -f "$work"/deploy/*-keypair.json
nice -n 19 cargo build --offline --locked --manifest-path "$here/embertide-stake/Cargo.toml" > "$work/book.out" 2>&1 \
  || { tail -n 20 "$work/book.out"; say "the book's build failed: not started"; exit 2; }
book="${CARGO_TARGET_DIR:-$here/embertide-stake/target}/debug/embertide-stake"
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

EPOCH_STOP="${EPOCH_STOP:-1}" EPOCH_UNTIL="${EPOCH_UNTIL:-0}" EPOCH_VALIDATOR="http://127.0.0.1:$rpc" LIGHT_BOOK="$book" LIGHT_DIR="$work" LIGHT_DEPLOYER="$work/deployer.json" LIGHT_PROGRAM_SHA256="$want" EPOCH_CERTIFICATE="$cert" \
  nice -n 19 cargo test --offline --locked --manifest-path "$here/poleis/press/Cargo.toml" --test light -- --nocapture --test-threads 1 the_epoch_turns_the_cranks
if pgrep -f "press epoch --stay --dir $work" > /dev/null; then say "a runner of this rehearsal is still running"; exit 1; fi
say "the epoch turned the cranks in $(( $(date +%s) - begun )) s${cert:+: its certificate $cert}"
