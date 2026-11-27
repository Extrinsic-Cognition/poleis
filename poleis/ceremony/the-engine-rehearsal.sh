#!/bin/sh
# THE STAKE EARNS, rehearsed (Nov 25 2026, lane/the-stake-earns; his "Yes, build them" to "Build the three missing links now?", link 3). A local
# validator holding mainnet's stake pool program, its epochs 32 slots long so one turns in seconds, then the live half of
# poleis/press/tests/engine.rs: a pool of the launch's shape, a stake, the SetStaker the ninth act carries naming the engine's key, the engine's
# crank run twice in one epoch (the second sends nothing), epochs turned until the stake earns and its fee lands in the fee account, and the
# way out at par, the crank bringing stake back to keep it open.
#   sh poleis/ceremony/the-engine-rehearsal.sh <the stake pool program's .so> [the certificate's path]
# Nothing is read from mainnet here. The .so is the file the door's rehearsal reads (`solana program dump SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy
# <file> --url mainnet`, written once), held to the fingerprint the launch's params name (pool.programSha256): other bytes, and nothing starts.
# The validator and the tests at nice 19; the validator ends with the script however it exits, its ledger removed.
set -eu
say() { printf '%s %s\n' "$(date -u '+%H:%M:%S')" "$*"; }
so=${1:-}
[ -n "$so" ] || { say "usage: sh poleis/ceremony/the-engine-rehearsal.sh <the stake pool program's .so> [the certificate's path]"; exit 2; }
cert=${2:-}
want=${ENGINE_PROGRAM_SHA256:-d18a2cdc0e0a52f55cb363cf517a4e72e9fca4fd6950c12897ee13c2385c52ac}
program=SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy
here=$(cd "$(dirname "$0")/../.." && pwd)

PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
export PATH
sdk=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk
if [ -z "${SDKROOT:-}" ] && [ -d "$sdk" ]; then SDKROOT=$sdk; export SDKROOT; fi
release=""
for d in "$HOME"/.local/share/solana/install/releases/*/solana-release; do
  [ -x "$d/bin/solana-test-validator" ] || continue
  if [ -z "$release" ] || [ "$d/bin/solana-test-validator" -nt "$release/bin/solana-test-validator" ]; then release=$d; fi
done
[ -n "$release" ] || { say "no Solana release under ~/.local/share/solana/install/releases: not started"; exit 2; }
PATH="$release/bin:$PATH"
export PATH

have=$(python3 -I -c 'import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], "rb").read().rstrip(b"\0")).hexdigest())' "$so")
[ "$have" = "$want" ] || { say "the program's executable bytes are sha256 $have, not $want: not started"; exit 2; }
say "the program: $so, its executable bytes sha256 $have; the release: $(solana-test-validator --version)"

# four ports and a span of thirty for the validator, all free on loopback
inuse=$(lsof -nP -iTCP -iUDP 2>/dev/null | awk 'NR > 1 { n = $9; sub(/->.*/, "", n); sub(/.*:/, "", n); if (n ~ /^[0-9]+$/) print n }' | sort -un)
free() { for p in $(seq "$1" "$2"); do echo "$inuse" | grep -qx "$p" && return 1; done; return 0; }
base=""
for b in 27600 28700 29800 21000 22100 23200 24300 25400 26500; do
  if free "$b" "$((b + 40))"; then base=$b; break; fi
done
[ -n "$base" ] || { say "no span of free ports: not started"; exit 2; }
rpc=$base; faucet=$((base + 2)); gossip=$((base + 4)); range="$((base + 10))-$((base + 40))"

work=$(mktemp -d "${TMPDIR:-/tmp}/the-engine.XXXXXX")
vpid=""
end() { if [ -n "$vpid" ]; then kill "$vpid" 2>/dev/null || true; wait "$vpid" 2>/dev/null || true; fi; rm -rf "$work"; }
trap end EXIT INT TERM
: > "$work/no-cli-config.yml"
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${ENGINE_ALARM:-900}" solana-test-validator --reset --quiet --ledger "$work/ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc" --faucet-port "$faucet" --gossip-port "$gossip" --dynamic-port-range "$range" --slots-per-epoch 32 \
  --config "$work/no-cli-config.yml" --bpf-program "$program" "$so" > "$work/validator.out" 2>&1 &
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
say "the validator: http://127.0.0.1:$rpc, 32 slots an epoch, the stake pool program at $program"

ENGINE_VALIDATOR="http://127.0.0.1:$rpc" ENGINE_PROGRAM_SHA256="$want" ENGINE_CERTIFICATE="$cert" \
  nice -n 19 perl -e 'alarm shift; exec @ARGV' "${ENGINE_ALARM:-900}" \
  cargo test --offline --locked --manifest-path "$here/poleis/press/Cargo.toml" --test engine -- --nocapture --test-threads 1
say "the engine rehearsed${cert:+: its certificate $cert}"
