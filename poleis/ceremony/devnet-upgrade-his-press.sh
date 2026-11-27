#!/bin/sh
# devnet-upgrade-his-press.sh: devnet's Poleis upgraded in place to the build rehearsed on a copy of every devnet swarm, then its older fund grown.
# His press, once: his devnet wallet is the program's upgrade authority, so this runs only by his hand. Devnet only; it refuses anything else.
# The steps (AN-OLDER-FUND-HAS-ITS-ROAD-LANDING.md): the build checked by its sha256; the deploy over the program (the CLI extends the program
# data and writes a buffer, his key the payer and the authority; the buffer's rent comes back at the upgrade); one slot later the growth
# (`wish fund --grow`, anyone may send it; his key pays 365,760 lamports of rent); then the fund read back.
set -eu
HERE="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$HOME/.local/share/solana/install/releases/stable-515465dd278366f3785dc75e5cce3165edbe316f/solana-release/bin"
KEY="$HOME/.config/solana/poleis-devnet-deployer.json"
RPC="https://api.devnet.solana.com"
PROGRAM="AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab"
SO="${SO:-/private/tmp/claude-501/-Users-lysergic-ratata-code-ec-house/3f06fe2b-8fa7-4e1e-84e1-94f2e473618b/scratchpad/upgrade-build/poleis_treasury.so}"
WISH="${WISH:-$HOME/c4-scratch/seat-3f06fe2b-older-fund/debug/examples/wish}"
WANT="6501d3fade43d12805676f8e947d88452bfd253064504e804caca221a3b6c5d8"

GOT=$(shasum -a 256 "$SO" | cut -c1-64)
[ "$GOT" = "$WANT" ] || { echo "the build is not the rehearsed one ($GOT): nothing sent"; exit 1; }
echo "0/3 a buffer an earlier run left half written goes back to his wallet (FbKSRV57… held 2.265 test SOL on Nov 24 23:3x UTC)"
"$BIN/solana" program close --buffers --url "$RPC" --keypair "$KEY" --commitment confirmed || true
echo "1/3 the upgrade (the build rehearsed on a copy of devnet, 445,744 bytes)"
# the 441 writes go straight to the leaders (no --use-rpc): through devnet's public RPC they were refused at Nov 24 2026 23:2x UTC
# ("429 Too many requests from your IP") and every blockhash expired; a priority price and more signing rounds carry them through a busy slot
"$BIN/solana" program deploy --url "$RPC" --keypair "$KEY" --upgrade-authority "$KEY" --program-id "$PROGRAM" --with-compute-unit-price 10000 --max-sign-attempts 50 --commitment confirmed "$SO"
sleep 2
echo "2/3 the older fund grown (706 to 778 bytes, every old value in place)"
"$WISH" fund --cluster devnet --rpc "$RPC" --payer "$KEY" --grow
echo "3/3 read back"
"$WISH" fund --cluster devnet --rpc "$RPC" --payer "$KEY"
