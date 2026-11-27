#!/bin/sh
# the-upgrade-extends-first.sh [certificate.json] · THE PRESS IS READY FOR MAINNET, its third piece proven on a local chain (Nov 24 2026,
# lane/the-press-is-ready-for-mainnet): a sealed upgrade to a longer build extends its program's data first. A fresh solana-test-validator of
# this machine holding the $WARM program's build (sBPF v3, as the press deploys it) at genesis, upgradeable, at its own length, its upgrade
# authority a stand-in of the run's seed; then the live half of poleis/press/tests/upgrade.rs: a buffer of the market's build (longer), the
# loader's Upgrade refused unextended, an extend of 168 bytes refused, the extend and the upgrade in one transaction refused, the press's own
# extend (upgrade::extend_on, what press crank sends before a sealed upgrade's execute) signed and paid by a crank that holds no authority,
# and the upgrade in a slot after it, the longer build read back byte for byte. Stand-in keys only, made in memory from a seed this script holds
# in its own memory; nothing is read beyond this machine and nothing is sent to any cluster but the validator it starts.
#   sh poleis/ceremony/the-upgrade-extends-first.sh [the certificate's path]
# By default the certificate is poleis/ceremony/the-upgrade-extends-first-<UTC date>.json. The builds, the validator and the test at nice 19;
# the validator ends with the script, under an alarm of its own too (UPGRADE_ALARM seconds, 1200), its ledger removed.
# Exit: the test's code; 2 when it could not start (no release, no free ports, a build that failed, a validator that never answered).
set -u
say() { printf '%s upgrade: %s\n' "$(date -u '+%H:%M:%S')" "$*"; }
here=$(cd "$(dirname "$0")" && pwd)
tree=$(cd "$here/../.." && pwd)
press="$tree/poleis/press"
cert=${1:-"$here/the-upgrade-extends-first-$(date -u +%Y-%m-%d).json"}
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

inuse=$(lsof -nP -iTCP -iUDP 2>/dev/null | awk 'NR > 1 { n = $9; sub(/->.*/, "", n); sub(/.*:/, "", n); if (n ~ /^[0-9]+$/) print n }' | sort -un)
free() { for p in $(seq "$1" "$2"); do echo "$inuse" | grep -qx "$p" && return 1; done; return 0; }
base=""
for b in 34200 35300 36400 37500 38600; do
  if free "$b" "$((b + 40))"; then base=$b; break; fi
done
[ -n "$base" ] || { say "no span of free ports between 34200 and 38640: not started"; exit 2; }
rpc=$base; faucet=$((base + 2)); gossip=$((base + 4)); range="$((base + 10))-$((base + 40))"

work=$(mktemp -d "${TMPDIR:-/tmp}/the-upgrade-extends-first.XXXXXX") || exit 2
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
  say "stopped: the validator ended, its ledger removed, $(($(date +%s) - began)) s in all"
}
trap stop EXIT
trap 'code=130; exit 130' INT
trap 'code=143; exit 143' TERM HUP

# the two builds, sBPF v3 as the press builds them for a deploy (build.rs build_program), into the run's own folder; the build keys removed
say "building the \$WARM program and the market for sBPF v3, and the proof"
for krate in warm_mine poleis_roads; do
  (cd "$tree/poleis/program/poleis_treasury" && env -u RUSTC -u RUSTC_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS -u CARGO_BUILD_TARGET \
    nice -n 19 cargo-build-sbf --offline --arch v3 --manifest-path "$tree/poleis/program/$krate/Cargo.toml" --sbf-out-dir "$work/deploy") \
    || { say "$krate did not build: not started"; exit 2; }
done
rm -f "$work"/deploy/*-keypair.json
(cd "$press" && nice -n 19 cargo test --quiet --offline --locked --test upgrade --no-run) || { say "the proof did not build: not started"; exit 2; }

seed=$(od -An -tx1 -N32 /dev/urandom | tr -d ' \n')
[ ${#seed} = 64 ] || { say "the system's randomness gave no seed: not started"; exit 2; }
authority=$(cd "$press" && UPGRADE_SEED="$seed" nice -n 19 cargo test --quiet --offline --locked --test upgrade the_authority_of_a_seed -- --exact --nocapture 2>/dev/null | awk '$1 == "authority" { print $2 }')
[ -n "$authority" ] || { say "the stand-in authority's address could not be read: not started"; exit 2; }
warm_id=$(sed -n 's/^solana_address::declare_id!("\([^"]*\)");$/\1/p' "$tree/poleis/program/warm_mine/src/lib.rs")
[ -n "$warm_id" ] || { say "the \$WARM program's id could not be read from its source: not started"; exit 2; }

: > "$work/no-cli-config.yml"
say "starting the validator: the \$WARM program $warm_id at genesis ($(wc -c < "$work/deploy/warm_mine.so" | tr -d ' ') bytes), upgradeable, its authority the stand-in $authority"
nice -n 19 perl -e 'alarm shift; exec @ARGV' "${UPGRADE_ALARM:-1200}" solana-test-validator --reset --quiet --ledger "$work/ledger" --bind-address 127.0.0.1 \
  --rpc-port "$rpc" --faucet-port "$faucet" --gossip-port "$gossip" --dynamic-port-range "$range" \
  --mint DfMo7MPjPQzmRubpQ3Ky84tQ4uVkPiooPHBQR8WGU32S --config "$work/no-cli-config.yml" \
  --upgradeable-program "$warm_id" "$work/deploy/warm_mine.so" "$authority" > "$work/validator.out" 2>&1 &
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

(cd "$press" && UPGRADE_VALIDATOR="http://127.0.0.1:$rpc" UPGRADE_SEED="$seed" UPGRADE_CERTIFICATE="$cert" UPGRADE_PROGRAM="$warm_id" \
  UPGRADE_SHORTER="$work/deploy/warm_mine.so" UPGRADE_LONGER="$work/deploy/poleis_roads.so" \
  UPGRADE_COMMIT="$(git -C "$tree" rev-parse HEAD 2>/dev/null)$(git -C "$tree" diff --quiet HEAD 2>/dev/null || echo ' and uncommitted changes')" UPGRADE_STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  nice -n 19 cargo test --offline --locked --test upgrade the_extend_comes_first_on_a_local_chain -- --exact --nocapture) &
tpid=$!
wait "$tpid"
code=$?
tpid=""
say "the proof: exit $code${cert:+, its certificate $cert}"
exit "$code"
