#!/bin/sh
# A SEAT OWNER'S KEYSTONE ON A LOCAL CHAIN (Nov 23 2026, lane/a-keystone-votes-on-a-local-chain; his card of that day, over THE WHOLE TREE
# DECIDES: his Nov 22 09:50:51 UTC "you gotta fucking stop treating the keystones as what make the decisions porcoddio"). A Keystone has one
# road into the launch's acts: its owner's own seat of root 0's floor, one hexagon like anyone's, and that seat's own transactions (its mint,
# `press seat`; its proposal and its PRO, `press vote`), each a prepared set the Keystone page signs by QR. This rehearses that road on a
# validator of this machine:
#   sh poleis/ceremony/a-keystone-votes-on-a-local-chain.sh [the certificate's path]
# The press's first sitting (the build, the deploy, the founding, the first acts written) on a fresh validator; then the checks
# (poleis/program/poleis_treasury/tests/council.rs, a_keystone_votes_on_a_local_chain, built from the sitting's copy): two stand-in people
# mint a seat each; the Keystone's owner mints a third, proposes the press's first act and votes it PRO, each through THE PAGE ITSELF
# (poleis/ceremony/keystone, served on loopback by tools/serve-range, in a headless browser nobody sees), a stand-in key held up to it as
# a Keystone is; with that seat alone the act does not pass; with a second seat owner's PRO, `press crank` seals and executes it.
# HIS OWN REHEARSAL: KEYSTONE_OWNER=<his Keystone's address>, or KEYSTONE_OWNER=ask (the page is served first; he connects his Keystone
# on it and the address it shows is written to KEYSTONE_OWNER_FILE, ~/c4-scratch/keystone-vote/owner). No browser is started and no key
# stands in: the script says which link to open on this Mac, and waits for his device at each of the three signatures (the mint inside the
# curve's window, about four minutes; the proposal and the PRO inside the act's, about six).
# THE RITE FREES THE AGENTS (Nov 23 2026): his posiverse's three agents are their own Nest keys, made at the sitting by `press chairs` in a
# folder of the sitting's alone, and each freed by `press rite` with its own mind (KEYSTONE_MIND, a model this machine serves on loopback,
# by default ollama's fable-fusion-27b-q4-64k) before anything else. The checks sign none of the act's envelopes: they ask for `chairs`, and
# each agent reads the press's first act and votes it by its own mind (`press chair`), its ballots a file of its own with its rite; the crank
# lands no ballot of an agent not freed, and the checks follow what the minds chose.
# Every other key here is a stand-in's, made in memory from two seeds of this shell; nothing is read or sent beyond this machine; nothing stays
# running, and the sitting's folder is removed. Exit: 0; 1 a step refused or a check failed; 2 it could not start; 142 its alarm
# (KEYSTONE_ALARM seconds, 7200). Ports: KEYSTONE_RPC_PORT (8699, its websocket the next), KEYSTONE_FAUCET_PORT (9700),
# KEYSTONE_GOSSIP_PORT (10090), KEYSTONE_PORT_RANGE (10100-10190), KEYSTONE_PAGE_PORT (8813): beside the launch press's rehearsal, not on it.
set -u
if [ -z "${KEYSTONE_ALARMED:-}" ]; then
  KEYSTONE_ALARMED=1
  export KEYSTONE_ALARMED
  exec perl -e 'alarm shift; exec @ARGV' "${KEYSTONE_ALARM:-7200}" /bin/sh "$0" "$@"
fi
say() { echo "keystone: $*"; }
here=$(cd "$(dirname "$0")" && pwd)
tree=$(cd "$here/../.." && pwd)
press_dir="$tree/poleis/press"
params=${KEYSTONE_PARAMS:-"$press_dir/params.rehearsal.json"}
copy=${KEYSTONE_COPY:-"$HOME/c4-scratch/keystone-vote/copy"}
cert=${1:-"$here/a-keystone-votes-on-a-local-chain-$(date -u +%Y-%m-%d).json"}
case "$cert" in /*) ;; *) cert="$(pwd)/$cert" ;; esac
owner=${KEYSTONE_OWNER:-}
rpc=${KEYSTONE_RPC_PORT:-8699}
faucet=${KEYSTONE_FAUCET_PORT:-9700}
gossip=${KEYSTONE_GOSSIP_PORT:-10090}
range=${KEYSTONE_PORT_RANGE:-10100-10190}
page_port=${KEYSTONE_PAGE_PORT:-8813}
lo=${range%-*}
hi=${range#*-}
genesis_mint=DfMo7MPjPQzmRubpQ3Ky84tQ4uVkPiooPHBQR8WGU32S
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
# the stand-in Keystone needs a browser nobody sees; his own needs none (his is the one he opens)
browser=""
if [ -z "$owner" ]; then
  browser=${KEYSTONE_BROWSER:-$(ls -d "$HOME"/Library/Caches/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-mac-*/chrome-headless-shell 2>/dev/null | tail -n 1)}
  [ -n "$browser" ] && [ -x "$browser" ] || { say "no headless browser (KEYSTONE_BROWSER, or Playwright's chrome-headless-shell): not started"; exit 2; }
fi
inuse=$(lsof -nP -i 2>/dev/null | awk 'NR > 1 { n = $9; sub(/->.*/, "", n); sub(/.*:/, "", n); if (n ~ /^[0-9]+$/) print n }' | sort -un)
for p in $inuse; do
  if [ "$p" = "$rpc" ] || [ "$p" = $((rpc + 1)) ] || [ "$p" = "$faucet" ] || [ "$p" = "$gossip" ] || [ "$p" = "$page_port" ] || { [ "$p" -ge "$lo" ] && [ "$p" -lt "$hi" ]; }; then
    say "port $p is taken: not started"
    exit 2
  fi
done

# the sitting's folder: its record, its sets, its logs, the page's served folder; no key is ever in it
work=$(mktemp -d "${TMPDIR:-/tmp}/a-keystone-votes.XXXXXX") || exit 2
chmod 700 "$work"
ledger="$work/ledger"
press_seed=$(od -An -tx1 -N32 /dev/urandom | tr -d ' \n')
standin_seed=$(od -An -tx1 -N32 /dev/urandom | tr -d ' \n')
[ ${#press_seed} = 64 ] && [ ${#standin_seed} = 64 ] || { rm -rf "$work"; say "the system's randomness gave no seed: not started"; exit 2; }
vpid=""
cpid=""
kpid=""
spid=""
bpid=""
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
  trap '' INT TERM HUP ALRM
  end "$cpid"
  end "$kpid"
  end "$bpid"
  end "$spid"
  end "$vpid"
  if [ "$code" != 0 ]; then
    [ -f "$work/checks.log" ] && { say "the checks' last words:"; tail -n 30 "$work/checks.log"; }
    [ -f "$ledger/validator.log" ] && { say "the validator's last words:"; tail -n 10 "$ledger/validator.log"; }
  fi
  press_seed=""
  standin_seed=""
  rm -rf "$work"
  find "$copy" -name '*-keypair.json' -delete 2>/dev/null
  say "stopped: the validator, the page's server and the browser ended, the sitting's folder removed, $(($(date +%s) - began)) s in all"
}
trap stop EXIT
trap 'code=142; exit 142' ALRM
trap 'code=130; exit 130' INT
trap 'code=143; exit 143' TERM HUP

# a press, its seed through a pipe (a FIFO in the sitting's folder: the seed passes in the kernel's memory, never a disk), its words in a log
pressed() {
  log=$1
  shift
  rm -f "$work/seed"
  mkfifo -m 600 "$work/seed" || return 2
  nice -n 19 "$press" "$@" < "$work/seed" > "$log" 2>&1 &
  cpid=$!
  printf '%s\n' "$press_seed" > "$work/seed"
  wait "$cpid"
  rc=$?
  cpid=""
  rm -f "$work/seed"
  [ "$rc" = 0 ] || cat "$log"
  return $rc
}

# 0. the house's press and the page's server, from this tree
say "the release: $release ($(solana-test-validator --version)); building the house's press"
(cd "$press_dir" && nice -n 19 cargo build --quiet --release --offline --locked --bin press) || { say "the press did not build: not started"; exit 2; }
press="$press_dir/target/release/press"
cc -O2 -o "$work/serve" "$tree/tools/serve-range/serve.c" -lpthread || { say "the page's server (tools/serve-range) did not build: not started"; exit 2; }

# 0b. THE PAGE, served as it is in the tree, on loopback, from the start (he connects his Keystone while the build and the deploy run):
# its own files, and beside them the sets the press writes for a seat owner (seats/, votes/), so the page opens one by its link; under
# an alarm of its own
mkdir -p "$work/www" "$work/seats" "$work/votes"
for f in index.html keystone.mjs prepared.mjs sol.mjs ur.mjs b58.mjs qrcode-generator.js; do ln -s "$here/keystone/$f" "$work/www/$f" || exit 2; done
ln -s ../seats "$work/www/seats" && ln -s ../votes "$work/www/votes" || exit 2
"$work/serve" -q -p "$page_port" -t "${KEYSTONE_ALARM:-7200}" "$work/www" > "$work/serve.log" 2>&1 &
spid=$!
link="http://127.0.0.1:$page_port/"
n=0
until curl -s --max-time 2 -o /dev/null "$link"; do
  n=$((n + 1))
  [ $n -lt 20 ] || { cat "$work/serve.log"; say "the page's server never answered on $link"; exit 2; }
  sleep 0.5
done
if [ "$owner" = ask ]; then
  # his address is not known yet: he connects his Keystone on the page, and the address it shows is written to a file (by the seat beside him)
  ask="${KEYSTONE_OWNER_FILE:-$HOME/c4-scratch/keystone-vote/owner}"
  mkdir -p "$(dirname "$ask")" && rm -f "$ask"
  say "HIS OWN KEYSTONE: open $link , Scan a Keystone, pick the account; its address goes into $ask (waiting 15 minutes at most)"
  n=0
  until [ -s "$ask" ]; do
    n=$((n + 1))
    [ $n -lt 900 ] || { say "no address came: not started"; exit 2; }
    sleep 1
  done
  owner=$(tr -d ' \n' < "$ask")
  rm -f "$ask"
fi
case "$owner" in *[!1-9A-HJ-NP-Za-km-z]*) say "KEYSTONE_OWNER is no address: not started"; exit 2 ;; esac

# 1. THE BUILD: the sitting's copy for the stand-in ids, each program built as the tests build it; Poleis's council tests from that copy
mkdir -p "$copy"
say "the build (the sitting's copy, $copy; the first one takes long)"
pressed "$work/build.log" build --dir "$work" --copy "$copy" --params "$params" --rehearsal || { say "the build refused: not started"; exit 2; }
council() { (cd "$copy/poleis/program/poleis_treasury" && nice -n 19 cargo test --offline --locked --test council "$@"); }
council --no-run > "$work/council-build.log" 2>&1 || { tail -n 30 "$work/council-build.log"; say "the council tests did not build: not started"; exit 2; }

# 2. the validator: nothing of the house at genesis, every feature it knows, nothing read beyond this machine
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${KEYSTONE_ALARM:-7200}" solana-test-validator --reset --quiet --ledger "$ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc" --faucet-port "$faucet" --gossip-port "$gossip" --dynamic-port-range "$range" --limit-ledger-size 50000000 \
  --mint "$genesis_mint" --config "$work/no-cli-config.yml" > "$work/validator.out" 2>&1 &
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
[ -n "$up" ] || { cat "$work/validator.out"; say "the validator never answered on 127.0.0.1:$rpc: not run"; exit 2; }
beyond=$(lsof -nP -a -p "$vpid" -i 2>/dev/null | awk 'NR > 1 && $9 !~ /^(127\.0\.0\.1|\[::1\]):/ { print $8, $9, $10 }')
odd=$(printf '%s\n' "$beyond" | grep -v '^$' | grep -Ev "^TCP \*:($rpc|$((rpc + 1))|$faucet) \(LISTEN\)\$|^UDP \*:[0-9]+ ?\$")
[ -z "$odd" ] || { say "the validator opened beyond loopback what this script does not know ($(printf '%s' "$odd" | tr '\n' ';')): stopped, nothing run"; exit 2; }
url="http://127.0.0.1:$rpc"
say "the validator answers: pid $vpid, $url"
code=1

# 3. his posiverse's three agents, their own Nest keys made at the sitting and each freed by its own mind (THE RITE FREES THE AGENTS); then
# the stand-ins (the floor's permit issuer, the board's roles, the dollar, and the agents' addresses read off their keys), then the press's
# first sitting, each step reading the chain first
mind=${KEYSTONE_MIND:-"http://127.0.0.1:11434/api/chat#fable-fusion-27b-q4-64k:latest"}
mind_model=${mind#*#}
curl -s --max-time 10 "${mind%%/api/*}/api/tags" 2>/dev/null | grep -q "\"$mind_model\"" || { say "no mind on this machine at $mind (KEYSTONE_MIND): its agents could not be freed: not run"; exit 2; }
nest_keys="$work/nest"
nice -n 19 "$press" chairs --seats launch-rehearsal-agent-1,launch-rehearsal-agent-2,launch-rehearsal-agent-3 --nest "$nest_keys" --dir "$work" < /dev/null > "$work/chairs-made.log" 2>&1 || { cat "$work/chairs-made.log"; say "press chairs made no keys"; exit 1; }
for i in 1 2 3; do
  nice -n 19 "$press" rite --seat "launch-rehearsal-agent-$i" --nest "$nest_keys" --params "$params" --charter "$tree/poleis/charters/nefertiti.txt" --mind "$mind" --dir "$work" < /dev/null > "$work/rite-$i.log" 2>&1 || { cat "$work/rite-$i.log"; say "agent $i was not freed (its words above): not run"; exit 1; }
  say "$(grep -m 1 '^FREED' "$work/rite-$i.log")"
done
POLEIS_VALIDATOR="$url" POLEIS_STANDIN_SEED="$standin_seed" POLEIS_AGENT_KEYS="$nest_keys" POLEIS_STAND_INS="$work/stand-ins.json" \
  council -- --ignored --exact the_stand_ins_of_the_launch_press --nocapture > "$work/stand-ins.log" 2>&1 || { tail -n 30 "$work/stand-ins.log"; say "the stand-ins failed"; exit 1; }
P="--dir $work --params $params --rpc $url --stand-ins $work/stand-ins.json --rehearsal"
for step in need airdrop deploy found vault warm market pool acts; do
  pressed "$work/$step.log" $step $P || { say "press $step refused"; exit 1; }
  say "press $step: done"
done

# 4. the stand-in Keystone's browser, or his own hands
if [ -n "$owner" ]; then
  say "HIS OWN KEYSTONE, $owner: connected on the page ($link, Scan a Keystone); each signature is asked below, with its link"
else
  # the browser nobody sees: no window, a profile of its own in the sitting's folder, a camera that is no camera; it is driven by two pipes
  mkfifo -m 600 "$work/cdp.in" "$work/cdp.out" || exit 2
  nice -n 19 "$browser" --remote-debugging-pipe --user-data-dir="$work/browser" --no-first-run --disable-gpu --use-fake-device-for-media-stream \
    --use-fake-ui-for-media-stream about:blank 3< "$work/cdp.in" 4> "$work/cdp.out" > "$work/browser.log" 2>&1 &
  bpid=$!
  say "the page, $link, in a headless browser (pid $bpid): a stand-in Keystone is held up to it"
fi

# 5. the checks. They ask the press for its steps by a word in the sitting's folder: `seat` (press seat, for the buyer of seat.ask, by the
# permit they wrote), `ratify`, `votes` (press vote, a line of votes.ask a set), `crank` and `crank-again` (press crank on the ballots)
POLEIS_VALIDATOR="$url" POLEIS_STANDIN_SEED="$standin_seed" POLEIS_AGENT_KEYS="$nest_keys" PRESS_RECORD="$work/record.json" POLEIS_CERTIFICATE="$work/checks.json" PRESS_SITTING="$work" PRESS_PARAMS="$params" \
  KEYSTONE_PAGE="$work" KEYSTONE_PAGE_LINK="$link" KEYSTONE_OWNER="$owner" \
  council -- --ignored --exact a_keystone_votes_on_a_local_chain --nocapture > "$work/checks.log" 2>&1 &
kpid=$!
asked=0
told=0
while kill -0 "$kpid" 2>/dev/null; do
  # his own rehearsal: each line the checks say to his hands, once
  now=$(grep -c "^HIS KEYSTONE" "$work/checks.log" 2>/dev/null)
  if [ "${now:-0}" -gt "$told" ]; then
    grep "^HIS KEYSTONE" "$work/checks.log" | tail -n $((now - told))
    told=$now
  fi
  if [ -e "$work/press.ask" ]; then
    want=$(cat "$work/press.ask")
    rm -f "$work/press.ask"
    asked=$((asked + 1))
    case "$want" in
      seat)
        pressed "$work/seat-$asked.log" seat $P --permit "$work/seat-permit.json" --buyer "$(cat "$work/seat.ask")" || { say "press seat refused"; exit 1; } ;;
      ratify)
        pressed "$work/ratify-$asked.log" ratify $P || { say "press ratify refused"; exit 1; } ;;
      votes)
        while read -r act who how; do
          [ -n "$act" ] || continue
          case "$how" in propose) more="--propose" ;; *) more="" ;; esac
          pressed "$work/vote-$asked-$who.log" vote $P --act "$act" --owner "$who" $more || { say "press vote refused (act $act, $who)"; exit 1; }
        done < "$work/votes.ask" ;;
      chairs)
        # THE RITE FREES THE AGENTS: each of unit 1's three agents reads the press's first act (its set alone) and votes it by its own mind,
        # the one its rite names; no seed of the press's is read
        rm -rf "$work/act-1" && mkdir "$work/act-1" && cp "$work/acts/1-"*.json "$work/act-1/" || { say "no set of the press's first act to read"; exit 1; }
        for i in 1 2 3; do
          nice -n 19 "$press" chair $P --key "$nest_keys/launch-rehearsal-agent-$i.key.json" --charter "$tree/poleis/charters/nefertiti.txt" --acts "$work/act-1" < /dev/null > "$work/chair-$i.log" 2>&1 || { cat "$work/chair-$i.log"; say "press chair refused for agent $i"; exit 1; }
          say "agent $i: $(grep -m 1 ' on act 1 ' "$work/chair-$i.log" | sed 's/^ *//')"
        done ;;
      crank | crank-again)
        pressed "$work/$want.log" crank $P --signatures "$work/ballots" || { say "press crank refused"; exit 1; } ;;
      *) say "the checks asked the press for an unknown step ($want)"; exit 1 ;;
    esac
    say "press $want: done"
    : > "$work/press.$want"
  fi
  sleep 0.2
done
wait "$kpid"
checks=$?
kpid=""
[ "$checks" = 0 ] || { say "the checks failed (exit $checks)"; exit 1; }
say "THE VERDICTS:"
grep -h "signed through the page\|A SEAT OWNER'S KEYSTONE" "$work/checks.log"
grep -h "RATIFIED" "$work"/ratify-*.log
grep -h "^ACT 1 \|press crank:" "$work/crank.log" "$work/crank-again.log"
cp "$work/checks.json" "$cert" || { say "the certificate could not be written to $cert"; exit 1; }
[ ! -f "$work/page.png" ] || cp "$work/page.png" "${cert%.json}.png"
code=0
say "A SEAT OWNER'S KEYSTONE ON A LOCAL CHAIN, REHEARSED: the certificate $cert, $(($(date +%s) - began)) s since the start"
exit 0
