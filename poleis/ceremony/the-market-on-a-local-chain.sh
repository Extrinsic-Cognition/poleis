#!/bin/sh
# the-market-on-a-local-chain.sh [certificate.json] · THE MARKET ON A LOCAL CHAIN (Nov 21 2026 14:4x UTC, his "Yes, rehearse it" to "Rehearse
# the market on a real local chain, then hand you the devnet press?"). THE TWO PROGRAMS IN ONE WORLD (poleis_treasury/tests/council.rs) on a
# real validator: the programs built from this tree as the tests build them, loaded at genesis at their declared ids into a fresh
# solana-test-validator, the ignored test the_two_programs_on_a_live_validator run against it over loopback, and the validator stopped by its
# pid and its ledger removed, whatever happened. The third program (Nov 21 2026, $WARM IS MINED BY USE: poleis/program/warm_mine) stands at
# genesis as an upgradeable program whose upgrade authority is the polis's own hand (Poleis's agent PDA of its root, found here offline by
# the CLI's find-program-derived-address), as the ceremony leaves a program, so only a sealed act of the polis opens its mine.
# The test writes the certificate (every step's signature, slot, units and bytes, and the rules it ran under) to the path given, by default
# poleis/ceremony/market-on-a-local-chain-<devnet|mainnet|all>-features-<UTC date>.json.
# The rules: MARKET_FEATURES, all (the default), devnet or mainnet-beta. all runs every feature the validator knows, as no cluster does, and
# reads nothing beyond this machine. Only when a cluster is named (devnet or mainnet-beta, never by default) does the validator take that
# cluster's feature set at genesis (--clone-feature-set --url https://api.devnet.solana.com, or https://api.mainnet-beta.solana.com): the
# run's one read beyond this machine, the validator's own, read-only, before anything runs; nothing is sent there and no key goes out. If it
# cannot take it, the script says so and stops, never falling back to every feature.
# No cluster but the one it starts, beside that read: the test refuses an RPC that is not loopback, every key of the run is made in memory
# and dies with it, and the validator's genesis lamports go to an address nobody holds a key for. The devnet press stays his.
# Beyond loopback: the validator's ports are bound to 127.0.0.1, except what its own code binds to every interface whatever --bind-address
# says (its RPC, its websocket and its faucet, listening, and a client socket); the script names them, stops it before anything runs if
# anything else is open beyond loopback, and says at the end how many inbound flows the application firewall saw to it.
# Exit: the test's code; 2 when it could not start (no release, a port taken, a build that failed, a feature set it could not take, a
# validator that never answered or that opened beyond loopback what is not named); 142 when its own alarm (MARKET_ALARM seconds, 3600 by
# default) ended it; the validator stands under the same alarm of its own.
# Nice: the validator is started at nice 19 by this script itself, whoever starts it, as its builds and its test are, so it steps behind the
# apps his hand is on (THE SEAT STEPS BEHIND).
# Ports: MARKET_RPC_PORT (8899, its websocket the next), MARKET_FAUCET_PORT (9900), MARKET_GOSSIP_PORT (10290), MARKET_PORT_RANGE (10300-10400).
set -u
if [ -z "${MARKET_ALARMED:-}" ]; then
  MARKET_ALARMED=1
  export MARKET_ALARMED
  exec perl -e 'alarm shift; exec @ARGV' "${MARKET_ALARM:-3600}" /bin/sh "$0" "$@"
fi
say() { echo "market: $*"; }
here=$(cd "$(dirname "$0")" && pwd)
tree=$(cd "$here/../.." && pwd)
treasury="$tree/poleis/program/poleis_treasury"
roads="$tree/poleis/program/poleis_roads"
warm="$tree/poleis/program/warm_mine"
# the rules it runs under: every feature, reading nothing beyond this machine, unless a cluster is named whose feature set the validator takes
features=${MARKET_FEATURES:-all}
case "$features" in
  devnet) cluster=https://api.devnet.solana.com tag=devnet ;;
  mainnet-beta) cluster=https://api.mainnet-beta.solana.com tag=mainnet ;;
  all) cluster="" tag=all ;;
  *) say "MARKET_FEATURES is devnet, mainnet-beta or all, not $features: not started"; exit 2 ;;
esac
cert=${1:-"$here/market-on-a-local-chain-$tag-features-$(date -u +%Y-%m-%d).json"}
case "$cert" in /*) ;; *) cert="$(pwd)/$cert" ;; esac
rpc=${MARKET_RPC_PORT:-8899}
faucet=${MARKET_FAUCET_PORT:-9900}
gossip=${MARKET_GOSSIP_PORT:-10290}
range=${MARKET_PORT_RANGE:-10300-10400}
lo=${range%-*}
hi=${range#*-}
# the genesis lamports' address: base58 of sha256("the market on a local chain: the genesis lamports, at an address nobody holds a key for"),
# so the validator never reads the CLI's keypair for it
genesis_mint=DfMo7MPjPQzmRubpQ3Ky84tQ4uVkPiooPHBQR8WGU32S
began=$(date +%s)

PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
export PATH
# the 27.0 SDK's linker refuses the host's build scripts where 26.5 stands beside it (THE CEREMONY, REHEARSED, finding 8)
sdk=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk
if [ -z "${SDKROOT:-}" ] && [ -d "$sdk" ]; then SDKROOT=$sdk; export SDKROOT; fi

# the newest Solana release installed: its validator, and its build tool first on PATH, so the tests' own build (tests/built/mod.rs) finds the same
release=""
for d in "$HOME"/.local/share/solana/install/releases/*/solana-release; do
  [ -x "$d/bin/solana-test-validator" ] && [ -x "$d/bin/cargo-build-sbf" ] || continue
  if [ -z "$release" ] || [ "$d/bin/solana-test-validator" -nt "$release/bin/solana-test-validator" ]; then release=$d; fi
done
[ -n "$release" ] || { say "no Solana release under ~/.local/share/solana/install/releases: not started"; exit 2; }
PATH="$release/bin:$PATH"
export PATH
say "the release: $release ($(solana-test-validator --version))"

# its own ports, free, or it does not start
inuse=$(lsof -nP -i 2>/dev/null | awk 'NR > 1 { n = $9; sub(/->.*/, "", n); sub(/.*:/, "", n); if (n ~ /^[0-9]+$/) print n }' | sort -un)
for p in $inuse; do
  if [ "$p" = "$rpc" ] || [ "$p" = $((rpc + 1)) ] || [ "$p" = "$faucet" ] || [ "$p" = "$gossip" ] || { [ "$p" -ge "$lo" ] && [ "$p" -lt "$hi" ]; }; then
    say "port $p is taken: not started"
    exit 2
  fi
done

# the declared ids, read from the sources as the tests read them
id_of() { sed -n 's/^[^/]*declare_id!("\([1-9A-HJ-NP-Za-km-z]*\)");.*/\1/p' "$1/src/lib.rs" | head -n 1; }
poleis_id=$(id_of "$treasury")
roads_id=$(id_of "$roads")
warm_id=$(id_of "$warm")
[ -n "$poleis_id" ] && [ -n "$roads_id" ] && [ -n "$warm_id" ] || { say "a program declares no id: not started"; exit 2; }

scratch=$(mktemp -d "${TMPDIR:-/tmp}/market-on-a-local-chain.XXXXXX") || exit 2
ledger="$scratch/ledger"
# the warm program's upgrade authority: the polis's own hand, Poleis's agent PDA of its root unit (seeds "agent" and the unit's eight bytes),
# found by the CLI offline (no RPC: no cluster is named and the CLI's own config is none)
hand=$(solana -C "$scratch/no-cli-config.yml" find-program-derived-address "$poleis_id" string:agent u64le:0 2>/dev/null | head -n 1)
case "$hand" in
  "" | *[!1-9A-HJ-NP-Za-km-z]*) hand="" ;;
esac
[ ${#hand} -ge 32 ] && [ ${#hand} -le 44 ] || hand=""
[ -n "$hand" ] || { rm -rf "$scratch"; say "the polis's hand could not be found: not started"; exit 2; }
vpid=""
vstart=""
tpid=""
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
  end "$tpid"
  end "$vpid"
  if [ "$code" != 0 ] && [ -f "$ledger/validator.log" ]; then
    say "the validator's last words:"
    tail -n 30 "$ledger/validator.log"
  fi
  # what reached it from beyond loopback while it ran, by the application firewall's own log (macOS)
  if [ -n "$vstart" ] && [ -x /usr/bin/log ] && [ -x /usr/libexec/ApplicationFirewall/socketfilterfw ]; then
    flows=$(/usr/bin/log show --start "$vstart" --style compact --predicate 'process == "socketfilterfw"' 2>/dev/null | grep -c "processPID: $vpid ")
    say "the application firewall ($(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate 2>/dev/null)) saw ${flows:-0} inbound flows to the validator (pid $vpid) while it ran"
  fi
  rm -rf "$scratch"
  rm -f "$treasury"/target/deploy/*-keypair.json "$treasury"/target/deploy-roads/*-keypair.json "$treasury"/target/deploy-warm/*-keypair.json
  say "stopped: the validator ended and its ledger removed, $(($(date +%s) - began)) s in all"
}
trap stop EXIT
trap 'code=142; exit 142' ALRM
trap 'code=130; exit 130' INT
trap 'code=143; exit 143' TERM HUP

# both programs, built as tests/built/mod.rs builds them: from the tests' crate, the compiler's own variables cleared, into their own folders
build() {
  (cd "$treasury" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET \
    nice -n 19 cargo-build-sbf --offline --manifest-path "$1/Cargo.toml" --sbf-out-dir "$2") || return 1
  # the build tool writes a deploy keypair beside a program it builds: removed unread, as the tests do (the ids are the sources')
  rm -f "$2"/*-keypair.json
}
say "building the three programs as the tests do"
build "$treasury" "$treasury/target/deploy" || { say "the Poleis build failed: not started"; exit 2; }
build "$roads" "$treasury/target/deploy-roads" || { say "the roads build failed: not started"; exit 2; }
build "$warm" "$treasury/target/deploy-warm" || { say "the warm build failed: not started"; exit 2; }

if [ -n "$cluster" ]; then
  set -- --clone-feature-set --url "$cluster"
  say "starting the validator: Poleis $poleis_id, the roads $roads_id, the warm program $warm_id (upgradeable, the polis's hand $hand), at genesis; the features of $features, taken from $cluster at genesis"
else
  set --
  say "starting the validator: Poleis $poleis_id, the roads $roads_id, the warm program $warm_id (upgradeable, the polis's hand $hand), at genesis; every feature it knows"
fi
vstart=$(date '+%Y-%m-%d %H:%M:%S')
# under an alarm of its own (the same exec, the same pid): a script ended by a signal no trap sees still leaves no validator past the alarm
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${MARKET_ALARM:-3600}" solana-test-validator --reset --quiet --ledger "$ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc" --faucet-port "$faucet" --gossip-port "$gossip" --dynamic-port-range "$range" \
  --mint "$genesis_mint" --config "$scratch/no-cli-config.yml" \
  --bpf-program "$poleis_id" "$treasury/target/deploy/poleis_treasury.so" \
  --bpf-program "$roads_id" "$treasury/target/deploy-roads/poleis_roads.so" \
  --upgradeable-program "$warm_id" "$treasury/target/deploy-warm/warm_mine.so" "$hand" "$@" > "$scratch/validator.out" 2>&1 &
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
if [ -z "$up" ]; then
  if [ -n "$cluster" ] && ! kill -0 "$vpid" 2>/dev/null; then
    say "the validator did not start with $features's feature set from $cluster (its words below), and it never runs with every feature instead: not run"
  else
    say "the validator never answered on 127.0.0.1:$rpc: not run"
  fi
  cat "$scratch/validator.out"
  exit 2
fi
# what it opened beyond loopback: only what the test validator's own code binds to every interface whatever --bind-address says (its RPC,
# its websocket and its faucet, listening; a client socket of its own, connected to nothing), and, when it took a cluster's feature set,
# what is left of that read (a connection to an HTTPS port); anything else stops it before anything runs
beyond=$(lsof -nP -a -p "$vpid" -i 2>/dev/null | awk 'NR > 1 && $9 !~ /^(127\.0\.0\.1|\[::1\]):/ { print $8, $9, $10 }')
known="^TCP \*:($rpc|$((rpc + 1))|$faucet) \(LISTEN\)\$|^UDP \*:[0-9]+ ?\$"
[ -n "$cluster" ] && known="$known|^TCP [^ ]+->[^ ]+:443 \([A-Z_0-9]+\)\$"
odd=$(printf '%s\n' "$beyond" | grep -v '^$' | grep -Ev "$known")
if [ -n "$odd" ]; then
  say "the validator opened beyond loopback what this script does not know ($(printf '%s' "$odd" | tr '\n' ';')): stopped, nothing run"
  exit 2
fi
say "the validator answers: pid $vpid, 127.0.0.1:$rpc; beyond loopback, by its own code: $(printf '%s' "$beyond" | tr '\n' ';')"

(cd "$treasury" && POLEIS_VALIDATOR="http://127.0.0.1:$rpc" POLEIS_CERTIFICATE="$cert" POLEIS_COMMIT="$(git -C "$tree" rev-parse HEAD 2>/dev/null)" \
  POLEIS_STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)" POLEIS_FEATURES="$features" POLEIS_FEATURES_FROM="$cluster" \
  nice -n 19 cargo test --offline --locked --test council -- --ignored --exact the_two_programs_on_a_live_validator --nocapture) &
tpid=$!
wait "$tpid"
code=$?
tpid=""
say "the live test: exit $code, $(($(date +%s) - began)) s since the start"
exit "$code"
