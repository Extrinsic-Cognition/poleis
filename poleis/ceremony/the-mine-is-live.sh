#!/bin/sh
# the-mine-is-live.sh [certificate.json] · THE MINE IS LIVE, proven on a local chain (Nov 23 2026, lane/the-mine-is-live; his Nov 21 2026
# 17:34:37 UTC "it is supposed to be uncapped supply, prompt-2-mine; the issuance needs to be live and continuous based on the space-router
# utilization"). A fresh solana-test-validator of this machine holding the $WARM program (poleis/program/warm_mine, built here as its tests
# build it) as an upgradeable program whose upgrade authority is the run's stand-in deployer, then the live half of poleis/press/tests/mine.rs:
# deposits and spends through the door's own code under node (the Worker's handler, the door's WASM as this tree holds it) and the crank,
# the chain held to the router's ledger across a halving. Stand-in keys only, made in memory from a seed this script holds in its own
# memory; nothing is read beyond this machine and nothing is sent to any cluster but the validator it starts.
#   sh poleis/ceremony/the-mine-is-live.sh [the certificate's path]
# By default the certificate is poleis/ceremony/the-mine-is-live-<UTC date>.json. Run it on the new Mac: the builds, the validator and the
# test at nice 19 (THE SEAT STEPS BEHIND); the validator ends with the script, under an alarm of its own too (MINE_ALARM seconds, 1800), its
# ledger removed. MINE_HALVING_AFTER (150): the seconds from the mine's opening act to its halving; a slower Mac raises it.
# Exit: the test's code; 2 when it could not start (no release, no free ports, a build that failed, a validator that never answered).
set -u
say() { printf '%s mine: %s\n' "$(date -u '+%H:%M:%S')" "$*"; }
here=$(cd "$(dirname "$0")" && pwd)
tree=$(cd "$here/../.." && pwd)
warm="$tree/poleis/program/warm_mine"
press="$tree/poleis/press"
cert=${1:-"$here/the-mine-is-live-$(date -u +%Y-%m-%d).json"}
case "$cert" in /*) ;; *) cert="$(pwd)/$cert" ;; esac
began=$(date +%s)

PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
export PATH
sdk=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk
if [ -z "${SDKROOT:-}" ] && [ -d "$sdk" ]; then SDKROOT=$sdk; export SDKROOT; fi
release=""
for d in "$HOME"/.local/share/solana/install/releases/*/solana-release; do
  [ -x "$d/bin/solana-test-validator" ] && [ -x "$d/bin/cargo-build-sbf" ] || continue
  if [ -z "$release" ] || [ "$d/bin/solana-test-validator" -nt "$release/bin/solana-test-validator" ]; then release=$d; fi
done
[ -n "$release" ] || { say "no Solana release under ~/.local/share/solana/install/releases: not started"; exit 2; }
PATH="$release/bin:$PATH"
export PATH
say "the release: $(solana-test-validator --version)"

# four ports and a span of thirty for the validator, all free on loopback
inuse=$(lsof -nP -iTCP -iUDP 2>/dev/null | awk 'NR > 1 { n = $9; sub(/->.*/, "", n); sub(/.*:/, "", n); if (n ~ /^[0-9]+$/) print n }' | sort -un)
free() { for p in $(seq "$1" "$2"); do echo "$inuse" | grep -qx "$p" && return 1; done; return 0; }
base=""
for b in 27600 28700 29800 30900 32000 33100; do
  if free "$b" "$((b + 40))"; then base=$b; break; fi
done
[ -n "$base" ] || { say "no span of free ports between 27600 and 33140: not started"; exit 2; }
rpc=$base; faucet=$((base + 2)); gossip=$((base + 4)); range="$((base + 10))-$((base + 40))"

work=$(mktemp -d "${TMPDIR:-/tmp}/the-mine-is-live.XXXXXX") || exit 2
chmod 700 "$work"
vpid=""
tpid=""
code=2
tree_of() { for c in $(pgrep -P "$1" 2>/dev/null); do tree_of "$c"; echo "$c"; done; }
end() {
  [ -n "$1" ] || return 0
  all="$(tree_of "$1") $1"
  kill -TERM $all 2>/dev/null
  n=0
  while [ $n -lt 10 ]; do
    alive=""
    for p in $all; do kill -0 "$p" 2>/dev/null && alive=1; done
    [ -n "$alive" ] || break
    sleep 0.5
    n=$((n + 1))
  done
  kill -KILL $all 2>/dev/null
  return 0
}
stop() {
  trap '' INT TERM HUP
  end "$tpid"
  end "$vpid"
  if [ "$code" != 0 ] && [ -f "$work/validator.out" ]; then
    say "the validator's last words:"
    tail -n 15 "$work/validator.out"
  fi
  seed=""
  rm -rf "$work"
  rm -f "$warm"/target/deploy-live/*-keypair.json
  say "stopped: the validator ended, its ledger removed, $(($(date +%s) - began)) s in all"
}
trap stop EXIT
trap 'code=130; exit 130' INT
trap 'code=143; exit 143' TERM HUP

# the $WARM program, built as its tests build it; the door's WASM is the tree's own (worker/poleis-seat.wasm, held to this tree's source by
# the door's tests); the press's test
say "building the \$WARM program and the proof"
(cd "$warm" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET \
  nice -n 19 cargo-build-sbf --offline --manifest-path "$warm/Cargo.toml" --sbf-out-dir "$warm/target/deploy-live") || { say "the \$WARM program did not build: not started"; exit 2; }
rm -f "$warm"/target/deploy-live/*-keypair.json
(cd "$press" && nice -n 19 cargo test --quiet --offline --locked --test mine --no-run) || { say "the proof did not build: not started"; exit 2; }

# the run's seed, in this shell's memory and the test's environment only; the stand-in deployer's address from it
seed=$(od -An -tx1 -N32 /dev/urandom | tr -d ' \n')
[ ${#seed} = 64 ] || { say "the system's randomness gave no seed: not started"; exit 2; }
upgrader=$(cd "$press" && MINE_SEED="$seed" nice -n 19 cargo test --quiet --offline --locked --test mine the_upgrader_of_a_seed -- --exact --nocapture 2>/dev/null | awk '$1 == "upgrader" { print $2 }')
[ -n "$upgrader" ] || { say "the stand-in deployer's address could not be read: not started"; exit 2; }
warm_id=$(sed -n 's/^solana_address::declare_id!("\([^"]*\)");$/\1/p' "$warm/src/lib.rs")
[ -n "$warm_id" ] || { say "the \$WARM program's id could not be read from its source: not started"; exit 2; }

: > "$work/no-cli-config.yml"
# the genesis lamports' address: base58 of sha256("the market on a local chain: the genesis lamports, at an address nobody holds a key for")
say "starting the validator: the \$WARM program $warm_id at genesis, upgradeable, its upgrade authority the stand-in deployer $upgrader"
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${MINE_ALARM:-1800}" solana-test-validator --reset --quiet --ledger "$work/ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc" --faucet-port "$faucet" --gossip-port "$gossip" --dynamic-port-range "$range" \
  --mint DfMo7MPjPQzmRubpQ3Ky84tQ4uVkPiooPHBQR8WGU32S --config "$work/no-cli-config.yml" \
  --upgradeable-program "$warm_id" "$warm/target/deploy-live/warm_mine.so" "$upgrader" > "$work/validator.out" 2>&1 &
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
[ -n "$up" ] || { say "the validator never answered on 127.0.0.1:$rpc: not run"; exit 2; }
say "the validator answers: pid $vpid, 127.0.0.1:$rpc"

(cd "$press" && MINE_VALIDATOR="http://127.0.0.1:$rpc" MINE_SEED="$seed" MINE_CERTIFICATE="$cert" MINE_HALVING_AFTER="${MINE_HALVING_AFTER:-150}" \
  MINE_COMMIT="$(git -C "$tree" rev-parse HEAD 2>/dev/null)$(git -C "$tree" diff --quiet HEAD 2>/dev/null || echo ' and uncommitted changes')" MINE_STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  nice -n 19 cargo test --offline --locked --test mine the_mine_is_live_on_a_local_chain -- --exact --nocapture) &
tpid=$!
wait "$tpid"
code=$?
tpid=""
say "the proof: exit $code${cert:+, its certificate $cert}"
exit "$code"
