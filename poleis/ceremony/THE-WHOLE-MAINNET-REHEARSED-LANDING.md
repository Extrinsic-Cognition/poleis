# THE WHOLE MAINNET, REHEARSED: the launch press's two sittings under mainnet's features, its pool program, its fee, rung and list, at main's tip (Nov 25 2026, lane/the-whole-mainnet-rehearsed, claude-3f06fe2b)

## His words, UTC

- Nov 24 23:2x "focus on mainner. let alone the legal stuff".
- Nov 24 23:4x "Yes, read it", to "May the full rehearsal read mainnet (GETs only)?": the rehearsal's three reads of mainnet at the validator's genesis
  (the feature set, Metaplex's Token Metadata program, the SPL stake pool program), `LAUNCH_FROM_MAINNET=1 LAUNCH_POOL_FROM_MAINNET=1`.

## Cost first

| | before | after |
|---|---|---|
| a launch rehearsal at main's tip (0aef6660cf) | stopped before its validator, 42 to 54 s: the seal reader's `--locked` build (its lock lacked poleis-door) | built |
| the sitting's own press build (`press build`, on mainnet too) | refused: "no matching package named poleis-door" (the press reads `poleis/door` since 9fb39c8c66; `build.rs` PARTS never copied it) | built: PARTS copies 8 parts |
| the full rehearsal, every mainnet value at once | never run (MAINNETS-TIP ran the fee, rung and list without the pool program; THE POOL IS BORN ran the pool at rung 1 and a list of 10) | exit 0 in 2,394 s on the new Mac at nice 19 |
| code | | `poleis/seal/Cargo.lock` +11 (written by cargo metadata --offline); `poleis/press/src/build.rs` PARTS +1 and its comment; no JavaScript |

## What ran

`LAUNCH_AS_MAINNET=1 LAUNCH_FROM_MAINNET=1 LAUNCH_POOL_FROM_MAINNET=1`, with `lane/the-press-is-ready-for-mainnet` merged in (dfbbd02380: `floor.issuer`
held to its key, `press crediter`, the extend before a sealed upgrade): priority fee 100,000 micro-lamports a unit, rung 5 (150 seats), a pool list of
100 validators; mainnet-beta's feature set, Metaplex and the stake pool program copied from mainnet at genesis. The whole of
`the-launch-press-rehearsal.sh`: the three faults refused before the deploy; the deploy short, refused; the deploy cut at Poleis's 100th write and resumed;
root 0 founded in the deploy's breath; the $WARM mine with its face by Metaplex's own program, moved later by a sealed act; the market; three new
agents' keys made at the sitting; the floor minted by the public through the issuer's permits; the eight acts passed by the agents' own minds and the
seat owners, the stake pool born by act 5 with its list of 100; the handoff refused in genesis, refused before the fund, then done (second sitting);
the seal's reach; a deposit and a spend through the door's own code on the mine; the miner's pull; the seal's reader and two releases shipped on a
scratch clone. The certificate: `launch-press-rehearsal-the-whole-mainnet-2026-11-25.json` beside this file.

The agents' mind: this Mac serves none any more (ollama's list empty, Nov 25 00:2x), so `LAUNCH_MIND` named qwen3:14b on the old Mac, reached by an ssh
link on loopback (127.0.0.1:11435) for the run's length and ended with it. Which minds sit in the chairs at the sitting is his.

The need at the local chain's rent (6,960 lamports a byte): 4.3128 SOL; at mainnet's 5,080, about 3.15 SOL, plus the priority fee.

## Not rehearsed

- The one-day windows: the windows stay the rehearsal's (MAINNETS-TIP: mainnet's day would hold each fault a day).
- The deployer key holding the three programs' code from the first sitting to the handoff (a day at least on mainnet): unchanged.
- His values still PLACEHOLDER in `params.mainnet.json`: the three agents' minds, `warm.opening`; the issuer, miner and crediter keys the sitting makes.

## The gates

No gate yet on the launch itself: the measure it waits for is this rehearsal run by the lander at every landing that touches `poleis/`, read as its exit.
