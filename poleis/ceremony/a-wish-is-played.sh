#!/bin/sh
# A WISH, PLAYED (Nov 24 2026, lane/a-wish-is-played; his 16:13:31 UTC "Yes, play it" to "Play one real wish through your five steps on a test chain,
# filmed on Swarmbook's own pages?", after his 15:51 "it is not meant to be hardcoded"). One wish taken through his five steps of 15:26 UTC by the
# program's own acts on a validator of this machine, so the site's own pages read every state off a chain:
#   sh poleis/ceremony/a-wish-is-played.sh ["<what should exist>" ["<its name>" [the receipt's path]]]
# It builds Poleis's own program from this tree (as its tests build it), loads it at genesis at its declared id on a fresh solana-test-validator
# 4.2.0, and runs poleis/seal/examples/wish.rs: start (a capital allocator starts it), soil (the first hexagon, three more necromancers on its
# pentagons, the door opened, three SnowTracers knock), work (two admitted, one refused, the treasury endowed, a budget for work put to the vote),
# grow (the budget paid, two more admitted, the door opened a rung up), satellites (a pentagon of the wish becomes a swarm of its own, a pentagon of
# that one another, and one question of the wish is answered by the whole tree, leaves first), rights (a decision left open; a holder's key for the
# page's seat store).
# After each step the chain's own account of the wish is written to the receipt (JSON). WISH_HOLD=<seconds> keeps the chain up that long at the end
# (default 0), the pages reading it at http://127.0.0.1:$WISH_SITE_PORT/dyson-society/app/polis.html?civ=<root>&program=<id>&rpc=http://127.0.0.1:<rpc>
# when WISH_SITE_PORT is given (the tree's own pages, served by tools/serve-range on loopback). Stand-in keys only, made by the walker in a folder of
# this run and removed with it; nothing is read or sent beyond this machine; nothing stays running. Everything at nice 19.
# WISH_STEPS=start stops the swarm right after its birth (its first sale open, nothing ratified: the newborn whose page shows the door to its first seat,
# A-NEWBORN-HAS-A-DOOR-TO-ITS-FIRST-SEAT-LANDING.md); WISH_WINDOW_SLOTS sets both windows of that sale to that many slots (default 150 to bid, then 50 to reveal).
# Exit: 0; 1 a step refused; 2 it could not start; 142 its alarm (WISH_ALARM seconds, 1200). Ports: WISH_RPC_PORT (31000; its websocket the next,
# the faucet +2, gossip +3, the dynamic range +20 to +120).
set -u
if [ -z "${WISH_ALARMED:-}" ]; then
  WISH_ALARMED=1
  export WISH_ALARMED
  exec perl -e 'alarm shift; exec @ARGV' "${WISH_ALARM:-1200}" /bin/sh "$0" "$@"
fi
say() { echo "wish: $*"; }
here=$(cd "$(dirname "$0")" && pwd)
tree=$(cd "$here/../.." && pwd)
wish=${1:-"A publisher of games that use the blockchain as their multiplayer, like pump.fun."}
name=${2:-"The Game Publisher"}
receipt=${3:-"$here/a-wish-is-played-$(date -u +%Y-%m-%dT%H%M%SZ).json"}
case "$receipt" in /*) ;; *) receipt="$(pwd)/$receipt" ;; esac
PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"; export PATH
sdk=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk
if [ -z "${SDKROOT:-}" ] && [ -d "$sdk" ]; then SDKROOT=$sdk; export SDKROOT; fi
release=${WISH_RELEASE:-"$HOME/.local/share/solana/install/releases/stable-515465dd278366f3785dc75e5cce3165edbe316f/solana-release"}
[ -x "$release/bin/solana-test-validator" ] && [ -x "$release/bin/cargo-build-sbf" ] || { say "no solana-test-validator and cargo-build-sbf under $release: not started"; exit 2; }
PATH="$release/bin:$PATH"; export PATH
rpc=${WISH_RPC_PORT:-31000}
for p in "$rpc" $((rpc + 1)) $((rpc + 2)) $((rpc + 3)) ${WISH_SITE_PORT:-}; do
  lsof -nP -iTCP:"$p" -sTCP:LISTEN >/dev/null 2>&1 && { say "port $p is taken: not started"; exit 2; }
done
poleis="$tree/poleis/program/poleis_treasury"
id=$(sed -n 's/^[^/]*declare_id!("\([1-9A-HJ-NP-Za-km-z]*\)");.*/\1/p' "$poleis/src/lib.rs" | head -n 1)
[ -n "$id" ] || { say "Poleis declares no id: not started"; exit 2; }
scratch=$(mktemp -d "${TMPDIR:-/tmp}/a-wish-is-played.XXXXXX") || exit 2
vpid=""; spid=""
cleanup() {
  [ -n "$spid" ] && kill "$spid" 2>/dev/null
  [ -n "$vpid" ] && kill "$vpid" 2>/dev/null && wait "$vpid" 2>/dev/null
  rm -rf "$scratch"
}
trap cleanup EXIT
trap 'exit 142' ALRM
trap 'exit 1' INT TERM HUP
say "building Poleis ($id) and the walker"
(cd "$poleis" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET \
  nice -n 19 cargo-build-sbf --offline --manifest-path "$poleis/Cargo.toml" --sbf-out-dir "$scratch/deploy") > "$scratch/build.out" 2>&1 \
  || { tail -n 20 "$scratch/build.out"; say "Poleis's build failed: not started"; exit 2; }
rm -f "$scratch"/deploy/*-keypair.json
(cd "$tree/poleis/seal" && nice -n 19 cargo build --offline --example wish) > "$scratch/build-walker.out" 2>&1 \
  || { tail -n 20 "$scratch/build-walker.out"; say "the walker's build failed: not started"; exit 2; }
walker="$tree/poleis/seal/target/debug/examples/wish"
: > "$scratch/no-cli-config.yml"
nice -n 19 solana-test-validator --reset --quiet --ledger "$scratch/ledger" --bind-address 127.0.0.1 --rpc-port "$rpc" --faucet-port $((rpc + 2)) \
  --gossip-port $((rpc + 3)) --dynamic-port-range "$((rpc + 20))-$((rpc + 120))" --mint 6Hq1wb8amQhMpPrB7qzXxhLoACsrBFqVqTTxehgP7X8d \
  --config "$scratch/no-cli-config.yml" --bpf-program "$id" "$scratch/deploy/poleis_treasury.so" > "$scratch/validator.out" 2>&1 &
vpid=$!
n=0
until curl -s --max-time 2 -X POST -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","id":1,"method":"getVersion"}' "http://127.0.0.1:$rpc" 2>/dev/null | grep -q solana-core; do
  n=$((n + 1)); [ "$n" -gt 60 ] && { tail -n 5 "$scratch/validator.out"; say "the validator never answered: not started"; exit 2; }
  kill -0 "$vpid" 2>/dev/null || { tail -n 5 "$scratch/validator.out"; say "the validator ended: not started"; exit 2; }
  perl -e 'select(undef, undef, undef, 1)'
done
say "the chain answers on 127.0.0.1:$rpc ($(shasum -a 256 "$scratch/deploy/poleis_treasury.so" | cut -c1-16)... at $id)"
if [ -n "${WISH_SITE_PORT:-}" ]; then
  cc -O2 -o "$scratch/serve" "$tree/tools/serve-range/serve.c" -lpthread 2>/dev/null || { say "tools/serve-range did not build"; exit 2; }
  nice -n 19 "$scratch/serve" -p "$WISH_SITE_PORT" -q "$tree/embertide-app/ui/intro" > /dev/null 2>&1 &
  spid=$!
fi
run() { nice -n 19 "$walker" "$@" --rpc "http://127.0.0.1:$rpc" --dir "$scratch/cast"; }
printf '{"wish":%s,"name":%s,"program":"%s","steps":{' "$(printf '%s' "$wish" | python3 -I -c 'import json,sys; print(json.dumps(sys.stdin.read()))')" "$(printf '%s' "$name" | python3 -I -c 'import json,sys; print(json.dumps(sys.stdin.read()))')" "$id" > "$scratch/receipt"
first=1
for step in ${WISH_STEPS:-start soil work grow satellites rights}; do
  say "step: $step"
  if [ "$step" = start ]; then run start --wish "$wish" --name "$name" > "$scratch/$step.out" 2>&1; else run "$step" > "$scratch/$step.out" 2>&1; fi \
    || { tail -n 8 "$scratch/$step.out"; say "the step $step was refused"; exit 1; }
  grep -c '…$' "$scratch/$step.out" | sed "s/^/wish:   transactions landed: /"
  [ "$first" = 1 ] || printf ',' >> "$scratch/receipt"; first=0
  printf '"%s":{"transactions":%s,"the_chain_says":%s}' "$step" "$(grep -c '…$' "$scratch/$step.out")" "$(tail -n 1 "$scratch/$step.out")" >> "$scratch/receipt"
done
total=$(curl -s -X POST -H 'Content-Type: application/json' --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getSignaturesForAddress\",\"params\":[\"$id\",{\"limit\":1000,\"commitment\":\"confirmed\"}]}" "http://127.0.0.1:$rpc" | python3 -I -c 'import json,sys; r=json.load(sys.stdin)["result"]; print(json.dumps({"touched_the_program": len(r), "failed": sum(1 for x in r if x.get("err") is not None)}))')
printf '},"transactions":%s,"validator":"solana-test-validator 4.2.0","at":"%s"}\n' "$total" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$scratch/receipt"
python3 -I -c 'import json,sys; json.dump(json.load(open(sys.argv[1])), open(sys.argv[2], "w"), indent=1); print(open(sys.argv[2]).read()[-420:])' "$scratch/receipt" "$receipt" || { say "the receipt does not read"; exit 1; }
say "the receipt: $receipt; transactions $total"
if [ "${WISH_HOLD:-0}" -gt 0 ]; then
  root=$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["steps"]["start"]["the_chain_says"]["root"])' "$receipt")
  say "held ${WISH_HOLD} s: polis.html?civ=$root&program=$id&rpc=http://127.0.0.1:$rpc"
  perl -e 'select(undef, undef, undef, shift)' "$WISH_HOLD"
fi
say "stopped: the validator ended, its ledger and the stand-in keys removed"
exit 0
