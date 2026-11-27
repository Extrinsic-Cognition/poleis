#!/bin/sh
# build.sh · the door's law as the page's one WASM module (STAKE IN HER, SEND YOUR LIGHT, Nov 22 2026): cargo build --release --target
# wasm32-unknown-unknown under nice, then the bytes beside the page that loads them (embertide-app/ui/intro/dyson-society/app/wasm/poleis-door.wasm,
# fetched by her-door.mjs from the same origin), with their size and sha256. No bindgen: the module imports nothing. Run it where cargo builds (the
# new Mac); CARGO_TARGET_DIR is taken when set, else this crate's own target.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
target=${CARGO_TARGET_DIR:-$here/target}
CARGO_TARGET_DIR=$target CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" nice -n 19 cargo build --offline --release --locked --target wasm32-unknown-unknown --manifest-path "$here/Cargo.toml"
out=$here/../../embertide-app/ui/intro/dyson-society/app/wasm/poleis-door.wasm
mkdir -p "$(dirname "$out")"
cp "$target/wasm32-unknown-unknown/release/poleis_door.wasm" "$out"
printf '%s bytes  sha256 %s  %s\n' "$(wc -c < "$out" | tr -d ' ')" "$(shasum -a 256 "$out" | cut -d' ' -f1)" "$out"
