#!/bin/sh
# build.sh [out] · THE FLASK TAKES ITS SEAT: the seat's law as one WASM, no bindgen (src/doors.rs), the Poleis it speaks the one the client it is built
# against names (its PROGRAM_ID, baked in). By default the tree's own, into the mint door's folder (embertide-app/ui/intro/dyson-society/worker/
# poleis-seat.wasm, committed: the door's tests and seat-vector.json stand on it); `press build` builds a sitting's from the sitting's copy into the
# path it gives, and records its sha256. Its own target folder, offline and locked, nice 19; nothing runs after it.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
out=${1:-"$here/../../embertide-app/ui/intro/dyson-society/worker/poleis-seat.wasm"}
cd "$here"
CARGO_TARGET_DIR=./target CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" nice -n 19 cargo build --quiet --release --offline --locked --target wasm32-unknown-unknown
mkdir -p "$(dirname "$out")"
cp target/wasm32-unknown-unknown/release/poleis_seat.wasm "$out"
printf '%s bytes  sha256 %s  %s\n' "$(wc -c < "$out" | tr -d ' ')" "$(shasum -a 256 "$out" | cut -d' ' -f1)" "$out"
