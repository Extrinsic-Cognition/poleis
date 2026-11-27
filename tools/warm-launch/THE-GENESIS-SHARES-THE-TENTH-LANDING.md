# THE GENESIS SHARES THE TENTH (Nov 19 2026, seat claude-93958ebc)

His words, UTC: 18:27:25 "we got the keys hypertrace i had moved 600 dollars to it dude"; 18:35:15 "why would you return to my wallet dude? we gotta
mint $WARM and we gotta get going with our ecosystem..."; 18:39:24 "wtf? hypertrace"; 18:42:50 "no porcoddio... build as if we were launching dumbass".
The line it builds (ec-workspace `docs/warm-the-fuel-the-rebate-and-what-the-arrow-lets-go-hypertrace-2026-11-13.md`): (1) the mint is the pre-deposit,
"10% $WARM is sold until breakpoint begins" (Oct 30 12:54); (2) the close is the genesis, "after that the $WARM swarm is geenesissn" (Oct 17 01:21): the
10% is the pre-depositors'; (3) after it $WARM is mined by burning.

## What it costs

- **Before:** the launch tool had one road, the Meteora sale in daily tranches (Oct 30, `LANDING.md`): its genesis put the 10% in the launch key's hand for
  the pool, nothing gave the tenth to the pre-depositors, and the token's name pointed at a placeholder.
- **Now, on mainnet, his Keystone signs:** one genesis transaction (752 bytes; 63,744 CU on devnet): the mint, 90% to his Squads vault, 10% to the launch
  key, the name, the mint authority revoked. Then the share: one transaction per 9 pre-depositors (56,019 CU for 3 on devnet), each payer's $WARM account
  made from the launch key's SOL (0.00204 SOL a payer). The share reads one `getTransaction` per deposit row, once. Nothing runs at rest.
- **JavaScript:** +214 net lines with the tests (the tool is JavaScript; no Rust port in this lane). The JavaScript law refuses it without his pass.

## What it does

- `node tools/warm-launch/mainnet.mjs genesis` writes `mainnet/unsigned-genesis-only.json`: the genesis alone, unsigned, read by the Keystone page's
  "Sign a prepared set". It names two placeholders and no more: `launchKey` (his Keystone's address) and `treasuryOwner` (his Squads vault). The pool's
  numbers (`startPrice`, `windowRatio`) belong to the older road and neither set reads them.
- `node tools/warm-launch/mainnet.mjs share --deposits FILE --before ISO` takes the deposit door's rows (`SELECT signature, payer FROM sr_deposits`, as
  `wrangler d1 execute ... --json` answers) and holds each to its own transaction by the door's own law (`verifyDeposit`, `app/space-router-deposit.mjs`):
  into the door's treasury and of its USDC (the door's GET terms), paid before the genesis instant, by the row's payer, or by the fee wallet for the flask
  the row names (THE HOUSE'S USDC FILLS A FLASK, Nov 12). Every other row is written into the set with its reason. The tenth is split in proportion: each
  payer's floor, then the leftover units by the largest remainder, so the shares sum to the tenth exactly. Unsigned, for his Keystone.
- The token's face: `embertide-app/ui/intro/dyson-society/app/warm-token.json`: name SWARM, symbol $WARM, his line "$WARM is basically the fuel to use our
  app" (Oct 18 13:05), the picture `warm-instrument-v1.webp` from the Sept 18 page he approved (square, already on the live site). The mint page names it on
  its agents line, so the build ships it (a json ships only when a page names it); `params.mainnet.json`'s `uri` points at it, and the metadata stays
  mutable, so the picture can change with a site deploy.

## Proof

- **devnet, Nov 19 2026 19:01 to 19:04 UTC** (`receipts/devnet-genesis-2026-11-19.json`, `rehearse-genesis.mjs`): a stand-in USDC; four pre-deposits by
  the door's own messages (`depositMessage`): a flask whose fee the house paid, twice; a depositor paying its own fee; the house's USDC for a flask
  (600.412645). The genesis and the share were signed by the Keystone page's own road (`readSet`, `toSign` with a fresh blockhash, `signedWire`), a
  throwaway key in the Keystone's place, then one deposit after the genesis. 10 of 10 checks: supply 1,000,000,000 at 9 decimals; no mint authority; no
  freeze authority; the 90% the vault's; the launch key keeps nothing; three payers from four deposits; the pour counted for its flask; the late deposit
  refused; every payer holds its share; the shares are the tenth. The shares: 600.412645 USDC got 90,571,910.119 $WARM, 37.5 got 5,656,853.928, 25 got
  3,771,235.952.
- `node --test tools/warm-launch/warm-launch.test.mjs`: 19 of 19 (three new: the door's law and the split, the packing and the Keystone page reading both
  sets, the devnet receipt and the token's face).
- The new Mac, Nov 19 2026 19:08 UTC, the lane's tip against main's (82ab637427): the mint folder 1188 tests, red on the lane alone one, the page's
  law CLEAR THE MINT (the link's title said "mint"; the title now says "The $WARM token: its name and picture"), 13 red on both; the launch tool and
  the Keystone page 35 of 35; any room 13 of 13; the token's file served 200 and its picture answers a range with 206.
- Rule 8 (the mint page is touched, one link on an existing line): `node --test app/dl-twelve.test.mjs` 21 of 21, and `nice -n 19 node
  app/dl-twelve.proof.mjs --url <the served page> --tap button` and `--tap sphere` both PASS ("the sound came in where the picture was, and the picture
  never went back") on the page served by tools/serve-range; again on the final tip below.

## His, before mainnet

1. His Keystone's address and his Squads vault, as text, into `params.mainnet.json` (his Oct 30 14:42 line carried the vault as a picture: never copied
   from a summary).
2. The genesis instant, the close: it ends the pre-deposits and opens the Space Router (`SPACE_ROUTER_OPENED_AT`).
3. The site's publish, so the token's face answers before the genesis.
4. His 600 USDC in the fee wallet counts as any pre-deposit does once it goes through the door: the house's pour for a flask stands since Nov 12, his one
   press. No door of its own (his 18:42:50).

Open, his to rule: the supply (Oct 30 02:28 "Yes, 1 billion", what this builds, against 15:25 "the token starts with supply 0 and each time there is a
deposit the tokens are minted"); the pre-deposits' USDC "to seed liquidity" (Oct 30 12:54): the pool after the genesis is another road, and the Oct 30
tranches stand untouched.
