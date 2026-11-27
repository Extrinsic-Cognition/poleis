#!/bin/sh
# THE STAKE GOES INTO HER, rehearsed (Nov 22 2026, lane/the-stake-goes-into-her; his "Yes, build it" to "Build the stake door and the choice,
# proven on the practice chain?"). A local validator holding mainnet's stake pool program, its epochs 32 slots long so one turns in seconds,
# then the live half of poleis/press/tests/door.rs: a pool of the launch's shape, a stake, the light chosen and changed, an epoch turned, the
# way out at par.
#   sh poleis/ceremony/the-door-rehearsal.sh <the stake pool program's .so> [the certificate's path]
# Nothing is read from mainnet here. The .so is the file `solana program dump SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy <file> --url
# mainnet` wrote once, held to the fingerprint the launch's params name (pool.programSha256, a rehearsal's certificate's
# pool.programRecorded.sha256): sha256 of its executable bytes, trailing zeros trimmed. Other bytes, and nothing starts. Run it on the new Mac:
# the validator and the tests at nice 19; the validator ends with the script, its ledger removed.
set -eu
say() { printf '%s %s\n' "$(date -u '+%H:%M:%S')" "$*"; }
so=${1:-}
[ -n "$so" ] || { say "usage: sh poleis/ceremony/the-door-rehearsal.sh <the stake pool program's .so> [the certificate's path]"; exit 2; }
cert=${2:-}
want=${DOOR_PROGRAM_SHA256:-d18a2cdc0e0a52f55cb363cf517a4e72e9fca4fd6950c12897ee13c2385c52ac}
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
for b in 21000 22100 23200 24300 25400 26500; do
  if free "$b" "$((b + 40))"; then base=$b; break; fi
done
[ -n "$base" ] || { say "no span of free ports between 21000 and 26540: not started"; exit 2; }
rpc=$base; faucet=$((base + 2)); gossip=$((base + 4)); range="$((base + 10))-$((base + 40))"

work=$(mktemp -d "${TMPDIR:-/tmp}/the-door.XXXXXX")
vpid=""
spid=""
end() { for p in $spid $vpid; do kill "$p" 2>/dev/null || true; wait "$p" 2>/dev/null || true; done; rm -rf "$work"; }
trap end EXIT INT TERM
: > "$work/no-cli-config.yml"
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${DOOR_ALARM:-900}" solana-test-validator --reset --quiet --ledger "$work/ledger" --bind-address 127.0.0.1 \
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

# STAKE IN HER, SEND YOUR LIGHT (Nov 22 2026): the page's doors in a browser. The module built again from this tree (poleis/door/build.sh: a module
# that differs from the one committed shows in git), the site served from this tree by the house's own server (tools/serve-range, built here), and a
# headless Chrome off every screen. Without Chrome the browser's half says so and passes. DOOR_SHOTS and DOOR_PAGE_CERTIFICATE, when set, keep its
# frames and its certificate.
sh "$here/poleis/door/build.sh" | tail -1
page=""
chrome=${DOOR_CHROME:-}   # else Chrome for Testing as embertide-screens finds it (cdp.rs find_chrome), else Chrome
if [ -z "$chrome" ]; then
  for c in "$HOME"/Library/Caches/ms-playwright/chromium-*/chrome-mac*/"Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing" "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"; do
    if [ -x "$c" ]; then chrome=$c; break; fi
  done
fi
if [ -n "$chrome" ]; then
  cc -O2 -o "$work/serve" "$here/tools/serve-range/serve.c" -lpthread
  nice -n 19 "$work/serve" -p "$((base + 6))" -b 127.0.0.1 -t "${DOOR_ALARM:-900}" -q "$here/embertide-app/ui/intro" &
  spid=$!
  page="http://127.0.0.1:$((base + 6))/dyson-society/app/"
  n=0
  until [ "$(curl -s -o /dev/null -w '%{http_code}' "${page}after-the-mint.html")" = 200 ] || [ $n -ge 20 ]; do sleep 0.5; n=$((n + 1)); done
  say "the site: $page, served from this tree"
else
  say "no Chrome on this Mac: the browser's half is not asked"
fi

DOOR_VALIDATOR="http://127.0.0.1:$rpc" DOOR_PROGRAM_SHA256="$want" DOOR_CERTIFICATE="$cert" DOOR_PAGE="$page" DOOR_CHROME="$chrome" \
  nice -n 19 cargo test --offline --locked --manifest-path "$here/poleis/press/Cargo.toml" --test door --test page -- --nocapture --test-threads 1
say "the door rehearsed${cert:+: its certificate $cert}${DOOR_PAGE_CERTIFICATE:+; the page: $DOOR_PAGE_CERTIFICATE}"
