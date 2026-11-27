#!/bin/sh
# install-epoch.sh · THE EPOCH TURNS THE CRANKS as a launchd agent (Nov 25 2026, lane/the-epoch-turns-the-cranks; his "Yes, build it"). WRITTEN, NOT
# INSTALLED BY ANY SEAT: mainnet has no pool until his sitting, and which Mac runs it is his ruling. Its shape is the lander's installer's
# (tools/embertide-mesh/lander/install-lander.sh): a run script and a plist written at install, nothing of a home in the tree, --dry and --remove.
#
#   sh poleis/press/install-epoch.sh <this Mac's short host name> <the press binary> <the sitting's params> <the RPC URL> <the sitting's folder> <her pool> [--dry]
#   sh poleis/press/install-epoch.sh <this Mac's short host name> --remove
#
# It refuses on another Mac than the one named (it writes this Mac's own agent; run it on the Mac he rules). The agent runs `press epoch --stay`:
# one pass each epoch, at nice 19 (ProcessType Background), kept alive by launchd; its log (one line an epoch, its errors) beside the sitting's
# folder, cut to its last 2,000 lines at each start. The runner reads its two keys from the sitting's folder (<folder>/keys/engine.json and
# pourer.json, made once by `press engine-key --dir <folder>` and `press pourer --dir <folder>`) and refuses without them or without the params'
# engine and light rules. Nothing here reads a keychain.
set -eu
[ $# -ge 2 ] || { echo "usage: install-epoch.sh <host> <press> <params> <rpc> <folder> <pool> [--dry] | <host> --remove" >&2; exit 2; }
host=$1
here=$(hostname -s)
[ "$here" = "$host" ] || { echo "refused: this Mac is $here, not $host: the agent is written on the Mac that runs it" >&2; exit 1; }
label="gg.embertide.press-epoch"
agents="$HOME/Library/LaunchAgents"
plist="$agents/$label.plist"
if [ "$2" = "--remove" ]; then
  launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
  rm -f "$plist" "$HOME/.embertide/press-epoch/run.sh"
  echo "removed: $label (the sitting's folder, its keys and its book stay)"
  exit 0
fi
[ $# -ge 6 ] || { echo "usage: install-epoch.sh <host> <press> <params> <rpc> <folder> <pool> [--dry]" >&2; exit 2; }
press=$2; params=$3; rpc=$4; folder=$5; pool=$6; dry=${7:-}
for f in "$press" "$params"; do [ -e "$f" ] || { echo "refused: $f does not stand" >&2; exit 1; }; done
for k in engine pourer; do [ -f "$folder/keys/$k.json" ] || { echo "refused: no $k key in $folder/keys (press $([ $k = engine ] && echo engine-key || echo pourer) --dir $folder makes it once)" >&2; exit 1; }; done
case "$rpc" in https://*) cluster=mainnet ;; http://127.0.0.1:*) cluster=localnet ;; *) echo "refused: $rpc: mainnet by https://, a rehearsal by http://127.0.0.1" >&2; exit 1 ;; esac
state="$HOME/.embertide/press-epoch"
log="$folder/epoch.log"
run="$state/run.sh"
runtext="#!/bin/sh
# written by install-epoch.sh: the log cut to its last 2,000 lines, then the runner
[ -f '$log' ] && tail -n 2000 '$log' > '$log.cut' && mv '$log.cut' '$log'
exec '$press' epoch --stay --cluster $cluster --dir '$folder' --params '$params' --rpc '$rpc' --pool '$pool' >> '$log' 2>&1
"
plisttext="<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\"><dict>
  <key>Label</key><string>$label</string>
  <key>ProgramArguments</key><array><string>/bin/sh</string><string>$run</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>ThrottleInterval</key><integer>60</integer>
  <key>Nice</key><integer>19</integer>
  <key>ProcessType</key><string>Background</string>
</dict></plist>
"
if [ "$dry" = "--dry" ]; then
  printf '%s\n--- %s\n%s' "$run" "$plist" "$plisttext"
  printf '%s' "$runtext"
  exit 0
fi
mkdir -p "$state" "$agents"
printf '%s' "$runtext" > "$run"
printf '%s' "$plisttext" > "$plist"
launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
launchctl bootstrap "gui/$(id -u)" "$plist"
echo "installed: $label on $here, its log $log"
