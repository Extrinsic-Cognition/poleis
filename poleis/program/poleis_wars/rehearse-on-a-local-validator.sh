#!/bin/sh
# rehearse-on-a-local-validator.sh [receipt.json] · THE WAR ON A REAL LOCAL VALIDATOR (Nov 24 2026, lane/the-war-is-its-own-program). The
# first reference war of embertide-war/tests/reference/reference-in-sol.json played on a fresh solana-test-validator: poleis_wars built from
# this tree as its tests build it and loaded at genesis at its declared id (--bpf-program), the rehearsal (rehearsal/, binary war-rehearsal)
# run against it over loopback, and the validator stopped with every child and its ledger removed, whatever happened.
# What the rehearsal does (rehearsal/src/main.rs): stand-in keys made in memory from a seed it draws and holds, funded by the validator's
# faucet; the config at minute_seconds = 4 (this clock cannot be warped: a raid of 195 minutes takes 780 seconds; the first reference war is decided at minute 150, 600 seconds in) and no shield; the two
# houses, the forty buys in the model's order, the stakes; then for each minute the chain's own Clock read until the raid's minute has begun
# and ONE transaction holding that minute's events and its crank, held to have landed in that minute; then the settlement, every convert, the
# unlock, every account read back and held to the reference to the unit and to the book. THE GAME IS ONE SWARM (lane/the-game-pays-its-swarm):
# Poleis's own program (poleis/program/poleis_treasury, built from this tree as its tests build it) stands at genesis beside the war, and
# the rehearsal founds the game swarm's tree and treasury there first: the war's generation names that treasury the publisher's, and at the
# end every house's creator and the publisher are paid, each held to the law's fees, and Poleis's own sync_endowment books the publisher's
# share. Its receipt (JSON) is printed and written to the
# path given, by default rehearsal/receipts/war-rehearsal-<UTC date and time>.json.
# The rules: every feature the validator knows; it reads nothing beyond this machine (no clone, no URL): no cluster but the one it starts,
# the rehearsal refuses an RPC that is not 127.0.0.1 or [::1] or that answers a public cluster's genesis, every key of the run is made in
# memory and dies with it (the validator's own keys stay in its ledger, removed at the end), and the genesis lamports go to an address nobody
# holds a key for (base58 of sha256 of "the war's rehearsal on a local validator: the genesis lamports, at an address nobody holds a key for").
# Beyond loopback: the test validator 4.2.0 binds its RPC, its websocket and its faucet on every interface whatever --bind-address says, and a
# UDP socket of its own (the market's rehearsal found the same, poleis/ceremony/THE-MARKET-ON-A-LOCAL-CHAIN-LANDING.md); the script names
# exactly those, stops before anything runs if anything else is open, and says at the end how many inbound flows the application firewall saw.
# Ports: off the defaults, and free, or it does not start: a block of 120 ports from WAR_PORT_BASE (the first of 28000, 30000, 32000, 34000,
# 36000 with every port of its block free), the RPC at its first, its websocket the next, the faucet the third, gossip the fourth and the
# validator's dynamic ports the block's last hundred. Nothing else of this Mac is touched: only this script's own pids are ever signalled.
# Exit: the rehearsal's (0 everything held, 1 anything did not); 2 when it could not start (no release, no free block of ports, a build that
# failed, a validator that never answered or opened beyond loopback what is not named); 142 when its alarm ended it: 20 minutes for the
# whole run (WAR_ALARM seconds, 1200 by default), the validator under the same alarm of its own. Nice: everything at 19 (THE SEAT STEPS BEHIND).
set -u
if [ -z "${WAR_ALARMED:-}" ]; then
  WAR_ALARMED=1
  export WAR_ALARMED
  exec perl -e 'alarm shift; exec @ARGV' "${WAR_ALARM:-1200}" /bin/sh "$0" "$@"
fi
say() { echo "war: $*"; }
here=$(cd "$(dirname "$0")" && pwd)
tree=$(cd "$here/../../.." && pwd)
rehearsal="$here/rehearsal"
reference="$tree/embertide-war/tests/reference/reference-in-sol.json"
receipt=${1:-"$rehearsal/receipts/war-rehearsal-$(date -u +%Y-%m-%dT%H%M%SZ).json"}
case "$receipt" in /*) ;; *) receipt="$(pwd)/$receipt" ;; esac
genesis_mint=6Hq1wb8amQhMpPrB7qzXxhLoACsrBFqVqTTxehgP7X8d
began=$(date +%s)

PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
export PATH
# the 27.0 SDK's linker refuses the host's build scripts where 26.5 stands beside it (THE CEREMONY, REHEARSED, finding 8)
sdk=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk
if [ -z "${SDKROOT:-}" ] && [ -d "$sdk" ]; then SDKROOT=$sdk; export SDKROOT; fi

# the release: 4.2.0, named (the active_release link on this Mac points into another home)
release=${WAR_RELEASE:-"$HOME/.local/share/solana/install/releases/stable-515465dd278366f3785dc75e5cce3165edbe316f/solana-release"}
[ -x "$release/bin/solana-test-validator" ] && [ -x "$release/bin/cargo-build-sbf" ] || { say "no solana-test-validator and cargo-build-sbf under $release: not started"; exit 2; }
PATH="$release/bin:$PATH"
export PATH
say "the release: $release ($(solana-test-validator --version))"

# a block of ports off the defaults, every one of them free
inuse=$(lsof -nP -i 2>/dev/null | awk 'NR > 1 { n = $9; sub(/->.*/, "", n); sub(/.*:/, "", n); if (n ~ /^[0-9]+$/) print n }' | sort -un)
base=""
for b in ${WAR_PORT_BASE:-28000 30000 32000 34000 36000}; do
  taken=""
  for p in $inuse; do
    if [ "$p" -ge "$b" ] && [ "$p" -lt $((b + 120)) ]; then taken=$p; break; fi
  done
  if [ -z "$taken" ]; then base=$b; break; fi
  say "port $taken of the block from $b is taken: the next block"
done
[ -n "$base" ] || { say "no free block of 120 ports: not started"; exit 2; }
rpc=$base
faucet=$((base + 2))
gossip=$((base + 3))
range="$((base + 20))-$((base + 120))"

id=$(sed -n 's/^[^/]*declare_id!("\([1-9A-HJ-NP-Za-km-z]*\)");.*/\1/p' "$here/src/lib.rs" | head -n 1)
[ -n "$id" ] || { say "the program declares no id: not started"; exit 2; }
poleis_dir="$here/../poleis_treasury"
poleis_id=$(sed -n 's/^[^/]*declare_id!("\([1-9A-HJ-NP-Za-km-z]*\)");.*/\1/p' "$poleis_dir/src/lib.rs" | head -n 1)
[ -n "$poleis_id" ] || { say "Poleis declares no id: not started"; exit 2; }
scratch=$(mktemp -d "${TMPDIR:-/tmp}/war-on-a-local-validator.XXXXXX") || exit 2
ledger="$scratch/ledger"
vpid=""
vstart=""
rpid=""
code=2
# a process and every process under it: TERM, five seconds, then KILL
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
  trap '' INT TERM HUP ALRM
  end "$rpid"
  end "$vpid"
  if [ "$code" != 0 ] && [ -f "$ledger/validator.log" ]; then
    say "the validator's last words:"
    tail -n 20 "$ledger/validator.log"
  fi
  if [ -n "$vstart" ] && [ -x /usr/bin/log ] && [ -x /usr/libexec/ApplicationFirewall/socketfilterfw ]; then
    flows=$(/usr/bin/log show --start "$vstart" --style compact --predicate 'process == "socketfilterfw"' 2>/dev/null | grep -c "processPID: $vpid ")
    say "the application firewall ($(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate 2>/dev/null)) saw ${flows:-0} inbound flows to the validator (pid $vpid) while it ran"
  fi
  rm -rf "$scratch"
  rm -f "$here"/target/deploy/*-keypair.json
  say "stopped: the validator and its children ended, its ledger removed, $(($(date +%s) - began)) s in all"
}
trap stop EXIT
trap 'code=142; exit 142' ALRM
trap 'code=130; exit 130' INT
trap 'code=143; exit 143' TERM HUP

# the program, built as tests/built/mod.rs builds it; the rehearsal, built offline
say "building poleis_wars and the rehearsal"
(cd "$here" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET \
  nice -n 19 cargo-build-sbf --offline --manifest-path "$here/Cargo.toml" --sbf-out-dir "$here/target/deploy") > "$scratch/build.out" 2>&1 \
  || { tail -n 20 "$scratch/build.out"; say "the program's build failed: not started"; exit 2; }
# the build tool writes a deploy keypair beside a program it builds: removed unread (the id is the source's placeholder)
rm -f "$here"/target/deploy/*-keypair.json
so="$here/target/deploy/poleis_wars.so"
# Poleis's own program, the game swarm's treasury's, built the same way into the same folder
(cd "$poleis_dir" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET \
  nice -n 19 cargo-build-sbf --offline --manifest-path "$poleis_dir/Cargo.toml" --sbf-out-dir "$here/target/deploy") > "$scratch/build-poleis.out" 2>&1 \
  || { tail -n 20 "$scratch/build-poleis.out"; say "Poleis's build failed: not started"; exit 2; }
rm -f "$here"/target/deploy/*-keypair.json
poleis_so="$here/target/deploy/poleis_treasury.so"
(cd "$rehearsal" && nice -n 19 cargo build --offline --release) > "$scratch/build-rehearsal.out" 2>&1 \
  || { tail -n 20 "$scratch/build-rehearsal.out"; say "the rehearsal's build failed: not started"; exit 2; }
bin="$rehearsal/target/release/war-rehearsal"

say "starting the validator: poleis_wars $id ($(shasum -a 256 "$so" | cut -c1-16)...) and Poleis $poleis_id ($(shasum -a 256 "$poleis_so" | cut -c1-16)...) at genesis, 127.0.0.1:$rpc, faucet $faucet, gossip $gossip, dynamic $range; every feature it knows"
vstart=$(date '+%Y-%m-%d %H:%M:%S')
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${WAR_ALARM:-1200}" solana-test-validator --reset --quiet --ledger "$ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc" --faucet-port "$faucet" --gossip-port "$gossip" --dynamic-port-range "$range" \
  --mint "$genesis_mint" --config "$scratch/no-cli-config.yml" \
  --bpf-program "$id" "$so" --bpf-program "$poleis_id" "$poleis_so" > "$scratch/validator.out" 2>&1 &
vpid=$!
up=""
n=0
while [ $n -lt 120 ]; do
  kill -0 "$vpid" 2>/dev/null || break
  if curl -s --noproxy '*' --max-time 2 -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' "http://127.0.0.1:$rpc" 2>/dev/null | grep -q '"result":"ok"'; then
    up=1
    break
  fi
  sleep 1
  n=$((n + 1))
done
if [ -z "$up" ]; then
  say "the validator never answered on 127.0.0.1:$rpc: not run"
  cat "$scratch/validator.out"
  exit 2
fi
# what it opened beyond loopback: only what the test validator's own code binds to every interface (its RPC, its websocket and its faucet,
# listening; a UDP socket of its own); anything else stops it before anything runs
beyond=$(lsof -nP -a -p "$vpid" -i 2>/dev/null | awk 'NR > 1 && $9 !~ /^(127\.0\.0\.1|\[::1\]):/ { print $8, $9, $10 }')
known="^TCP \*:($rpc|$((rpc + 1))|$faucet) \(LISTEN\)\$|^UDP \*:[0-9]+ ?\$"
odd=$(printf '%s\n' "$beyond" | grep -v '^$' | grep -Ev "$known")
if [ -n "$odd" ]; then
  say "the validator opened beyond loopback what this script does not know ($(printf '%s' "$odd" | tr '\n' ';')): stopped, nothing run"
  exit 2
fi
say "the validator answers: pid $vpid, 127.0.0.1:$rpc, in $n s; beyond loopback, by its own code: $(printf '%s' "$beyond" | tr '\n' ';')"

mkdir -p "$(dirname "$receipt")"
nice -n 19 "$bin" --rpc "http://127.0.0.1:$rpc" --program "$so" --poleis "$poleis_id" --reference "$reference" --receipt "$receipt" &
rpid=$!
wait "$rpid"
code=$?
rpid=""
say "the rehearsal: exit $code, $(($(date +%s) - began)) s since the start; its receipt $receipt"
exit "$code"
