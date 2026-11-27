# STAKE IN HER, SEND YOUR LIGHT (Nov 22 2026, lane/stake-in-her-send-your-light, claude-901e44a8)

## His words, UTC

- 16:57:42, over three ways the stake could live on a society: "should be B... you must not overhwelm the user with infos dude... those are fucking
  wishing wells... people wish and delegate... THE META-SWARM IS GLORIOUS BECAUSE YOU STAKE ALL THERE IN 1 PLACE (the meta-swarm... the unit0... the
  root of them all...) BUT!!!! you can delegate the yield of your stake or let the swarm futarchy handle it for you idk... hypertrace"
- 17:02:34: "there is only 1 meta swarm and that is the diving lyssajudron... all the other ones are not meta-swarm, they are swarms..."
- "Yes, draw it" (the five screens on the live pages), "Yes, build it" (the chain half: lane/the-stake-goes-into-her), and at 20:54 his tap "Yes, build
  them" on the Spool Board to "Build the page doors next, on the Pantheon session's page?" (relayed by the Stop hook; a build, none of rule 2's acts).

## Cost first

| | before (main with lane/the-stake-goes-into-her and lane/the-pantheon-is-introduced, which this lane stands on) | this lane |
| --- | --- | --- |
| the door's law | in the press (`poleis/press/src/door.rs`, 255 lines), out of a page's reach | `poleis/door`: the law (`lib.rs`, 389 lines) and the page's doors (`page.rs`, 167); the press's `door.rs` 36 lines of wrappers |
| the page's module | none | `poleis-door.wasm`, 105,697 bytes, no imports; fetched only when a door is used |
| a visitor's page at rest | | nothing new runs or draws until a chain names her pool (no timer, no read); then `her-door.mjs` reads the chain on a press only |
| what a press reads | | four reads (her pool, the epoch rewards sysvar, the epoch, a blockhash) and the receipts' balance; the send, then its status every 0.4 s until confirmed (a second or two on the practice chain) |
| what the Pantheon loads to sign | | the pinned `vendor/solana-web3-1.95.8.iife.min.js` (459,805 bytes, its sha384 as `dl-direct.js` pins it), once, at the first signature; the wish's page holds it already |
| what a staker pays | the chain half's: the receipts account's rent, 0.00203928 SOL (theirs back if they close it), and 0.000005 a transaction | the same, read from the page: 25.002044280 SOL for a stake of 25 |
| JavaScript | | +75 net: `her-door.mjs` 74, `swarmbook.html` 1 (`polis.html` 0); the gate's pass is his word (below) |
| the proof | | the rehearsal 39 s wall on the new Mac at nice 19 (the chain half 15.7 s, the browser's 13.9 s); nothing on the serving Mac |
| a build on the new Mac | | `cargo test poleis/door` 8 passed in 0.01 s once built; the module's release build about 6 s |

## What stands

- **The law, once** (`poleis/door`, the crate `poleis-door`): the stake, the light's choice and the way out on her pool, moved out of the press's
  `door.rs` (which could not reach a page: the press links poleis-client, whose BLS, blstrs over blst in C, does not build for wasm32). Solana's own
  types and its own compiler of the legacy message build to a module that imports nothing. The press names the same law on its `Pool`
  (`poleis/press/src/door.rs`, now a page of wrappers); its tests are unchanged and pass.
- **What the page asks of it** (`poleis/door/src/page.rs`): her pool read from its own account (the stake pool program's, its tokens the token program's,
  its epoch fee the whole yield, 1/1, as the launch births hers; any other pool is refused in words: "Not her pool: ..."), the unsigned transaction a
  wallet signs for the stake, the light (a wish by its unit, or the futarchy; refused without receipts) and the way out, the staker's receipts account,
  their worth, an amount typed read in SOL, and the light a listed signature's memo names. No bindgen: the page writes texts and bytes into IN and
  reads OUT; every word a visitor reads of a refusal is written in Rust.
- **The module** (`embertide-app/ui/intro/dyson-society/app/wasm/poleis-door.wasm`, written by `poleis/door/build.sh`).
- **The doors**: on the Pantheon (`swarmbook.html`), "Stake SOL in her ↗" under her words once her twelve are home (the scene's cover 57% to 61%),
  leaving with the card; its dialog: "Keep your SOL. Burn its light.", the amount, Stake; then "Your 25 SOL is in her", "Where does its light go?", "Send
  it to a wish" (the wells) or "Let the futarchy place it"; and, for a wallet that holds receipts, "Take your 25 SOL back". On a wish (`polis.html`):
  "Send your light here ↗", and "Your light pours here · change any epoch ↗" with "Your 25 SOL stays in her." once the visitor's newest light names it.
  The words are the drawing's, his to reword.
- **The glue** (`her-door.mjs`): reads the chain, hands the wallet what the module wrote, sends it, says what came of it.
- **Hidden until a chain names her pool.** Her pool is born at the launch, his sitting; a practice chain names it (`?rpc=http://127.0.0.1:<port>&pool=`).
  On swarmbook.com today nothing shows. At the launch, the pool's address goes into the glue (one line) by a lane of the launch.

## The proof

`poleis/ceremony/the-door-rehearsal.sh <the program's .so> [certificate]` on the new Mac (Nov 22 2026 21:25 UTC): solana-test-validator 4.2.0 with
mainnet's stake pool program bytes (sha256 `d18a2cdc…52ac`, held before anything starts), the module built again from this tree, the site served from
this tree by `tools/serve-range`, and Chrome for Testing headless, off every screen.

- `cargo test poleis/door`: 8 passed (her pool read from its account, another pool refused in words, the page's stake, light and way out byte for byte
  the law's, an amount read in SOL, a light read as the chain lists it, the slots).
- `poleis/press --test door`: 6 passed, the chain half of lane/the-stake-goes-into-her on the moved law (25 SOL in, the light changed, an epoch turned,
  26 SOL back at par).
- `poleis/press --test page`: 1 passed in 13.93 s, the visitor's road in the browser, a stand-in's wallet in the page, each step read back from the chain
  (`poleis/ceremony/the-doors-on-the-page-2026-11-22.json`, its six frames beside it):
  1. the Pantheon's door stands (opacity 1) once her twelve are home;
  2. 25 typed and staked from the page: 25,000,000,000 receipts, the page saying "Your 25 SOL is in her", the staker paying 25.002044280 SOL;
  3. its light to the futarchy: read back from the staker's signatures, Futarchy;
  4. a wish's page (unit 7): "Send your light here", pressed: read back, Wish(7);
  5. that page loaded again: "Your light pours here", "Your 25 SOL stays in her. Change it any epoch." (the newest light read from the chain);
  6. the way out from her: every receipt burned, 24,999,995,000 lamports back (25 SOL less the fee).

Read on the frames and mended before the last run: the dialog opened with its close button focused and ringed (now the amount takes the first focus),
and "Your light pours here · change any epoch" broke its arrow onto a second line at 390 px (the epoch now in the note under it). On the practice chain a
wish's page has no society to read (its head says "Society unavailable"); its door does not need one.

## The JavaScript

The gate refuses a lane that adds JavaScript unless his word passes it (`tools/embertide-mesh/tests/the-javascript-only-shrinks.test.mjs`, its
exceptions). Looked for a road to pay instead: nothing the doors replace is JavaScript, and the one road its comments call closed (the Gold month,
`app/dl-gold-bid.mjs`) is still the Worker's law (`worker/gold.mjs`, `tools/gold-close.mjs`). Asked of him: the pass of the glue's lines, pinned to
these files and bytes by the gate's receipt, once the two lanes under this one have landed.

## Not built, and whose

- **Mainnet**: her pool is born at his sitting; the door shows there once its address is named.
- **No light yet.** No validator holds her stake (the engine, piece 6 of the staking doc), so the light is 0 and the way out is at once.
- **The reader that pays a wish its light**: the memos are written and read back; counting each staker's light to its wish and the refill are owed.
- **The hand**: after lane/the-stake-goes-into-her and lane/the-pantheon-is-introduced land, main merged in, the gate's pass of the glue's lines (his
  word, the receipt pinned to that main), then the hand.

## His pass, and the count it must cover (claude-ffc28f45, after the account switch of 22:25 UTC)

"Yes, pass 74", typed in claude-ffc28f45's session at 2026-11-22 22:5x UTC, to "Pass the stake doors' 74 lines of JavaScript?". The 74 was read
against main 2fcf6bede6 with the Pantheon lane still under this one (its own -1 in the count). Over the Pantheon lane's tip the lane's own JavaScript
is +80 -5, net +75 (`her-door.mjs` +74, `swarmbook.html` +3 -2, `polis.html` +2 -2, `swarmbook.test.mjs` +1 -1), one more than his word covers:
asked of him again as +75. The gate's exception (`the-javascript-only-shrinks.test.mjs` APPROVALS and its receipt) is pinned to this lane's base,
so it is written once the Pantheon lane has landed and main is merged in, then the hand.

## Nov 24 2026: the doors stand on today's main (lane/the-lights-doors-stand-on-main)

### His words, UTC

- Nov 22 16:57:42: "THE META-SWARM IS GLORIOUS BECAUSE YOU STAKE ALL THERE IN 1 PLACE ... BUT!!!! you can delegate the yield of your stake or let the swarm
  futarchy handle it for you idk..."
- Nov 24 15:26:58: "the making of wish is a capital allocator action" (the drafts' first row: a third way for the light, "+ Start a new wish").
- Nov 24 17:03:50: "build all and ship to prod", over the three drafts (ec-workspace `docs/swarmbook-is-the-wishs-road-2026-11-24/the-gaps-drafted.jpg`,
  record 25.15 of `docs/swarmbook-is-the-new-poleis-us-and-the-claw-riders-hypertrace-2026-11-10.md`): draft 1 (the third way) and draft 3 (the blue
  door beside the red knock, "from your stake in her" under it) are this lane's.

### Cost first

| | before (main 84c7bb060b) | this lane |
| --- | --- | --- |
| a visitor's page at rest | | unchanged: the doors are hidden markup; `her-door.mjs` (74 lines, loaded as a module) reads nothing and draws nothing unless the address names a chain on this machine and a pool |
| JavaScript against main | | +80 -6, net +74: `her-door.mjs` +74, `after-the-mint.html` +4 -4, `polis.html` +2 -2 (no change to `polis.js`); the pages' test of the doors is Rust (`poleis/door/tests/the_pages.rs`), so the count is the 74 his Nov 22 "Yes, pass 74" named, now against a different base |
| CSS | | `swarmbook-book.css` +19 (the dialog: `after-the-mint.css` may hold only the freeze plate's colours, by its test), `polis.css` +5 |
| the module | 105,697 bytes | the same bytes: the crate did not change, `build.sh` wrote them again in the rehearsal (sha256 `0e5cba33…2fcfff`) |
| the proof | | the rehearsal about 2.5 min wall on this Mac (guidos-MBP-2) at nice 19, the browser's half 17.4 s |

### What moved since Nov 22

- `swarmbook.html` (with its css and test) is retired on main; the Pantheon lives on `after-the-mint.html` (THE PANTHEON IS ONE RING, THE TWO SIDES COME
  ALIVE) and on `polis.html?civ=0#hall`. The lane is merged onto main (bef2c5d731): its crate, module, press door, rehearsal and proof as they were; the
  retired page's edits dropped; the press's Cargo.lock regenerated by cargo (the same 130 lines).
- **Her stake** stands under the Pantheon's call on `after-the-mint.html` ("Keep your SOL / Fuel the swarm", then "Stake SOL in her ↗"), in the page's own
  `.duo-door`; a button, since the page's test refuses a link to `#her-door`. Its dialog is the lane's, words and frames as they were. It is not on
  `polis.html?civ=0`'s hall (kept small for the other lane in those files).
- **The light's third way** (draft 1): "+ Start a new wish", under "Send it to a wish" and "Let the futarchy place it". It links to `./found.html` (the wish
  page main holds; the wall's first card as the place a wish starts is another lane, not on main when this was cut: re-point at integration). Nothing is
  signed for it.
- **Send your light here** (draft 3) stands right after `#doors` on `polis.html`, "from your stake in her" under it; above 560 px it takes the half beside the
  knock ("Knock with your agent" and its seats), alone it takes the row; on a phone the two stack.

### Tests (this Mac, nice 19)

- `poleis/door`: `cargo test --offline`, 8 + 4 passed (the law's 8, the pages' 4 new).
- `poleis/press`: `cargo test --offline --locked`, 116 passed, 0 failed (without the validator its door and page halves say so and pass).
- The page suites: `node --test after-the-mint.test.mjs polis-knock.test.mjs polis-rite.test.mjs swarmbook-board.test.mjs swarmbook-film.test.mjs
  swarmbook-floor.test.mjs dl-mint-hexagon.test.mjs dl-side-press.test.mjs`, 52 of 52.
- THE LOOK IS A LAW green. THE JAVASCRIPT ONLY SHRINKS red at +74, as it must be until his pass is written against this lane's base.

### The proof (practice chain only)

`sh poleis/ceremony/the-door-rehearsal.sh ~/c4-scratch/door/spl_stake_pool.so` with `DOOR_SHOTS` and `DOOR_PAGE_CERTIFICATE` set: solana-test-validator 4.2.0
on loopback with mainnet's stake pool program bytes (sha256 `d18a2cdc…52ac`, held before anything starts), stand-in keys, the site served from this tree by
`tools/serve-range`, Chrome for Testing headless. The chain half (`--test door`) 6 passed; the browser's (`--test page`) 1 passed
(`poleis/ceremony/the-lights-doors-on-main-2026-11-24.json`, seven frames beside it):

1. `after-the-mint.html?side=pool` (the visit the split's blue side sends; a plain visit goes on to the mint page until its window closes), its film skipped:
   "Stake SOL in her" on the glass under the Pantheon's call;
2. 25 typed; 3. staked: 25,000,000,000 receipts read back, the page saying "Your 25 SOL is in her", the staker paying 25.002044280 SOL; the three ways read
   off the page: Send it to a wish (`after-the-mint.html?side=pool#wells`), Let the futarchy place it, + Start a new wish (`found.html`), each link carrying the
   visit's chain; the futarchy chosen: read back Futarchy;
4. a wish's page (unit 7): "Send your light here", pressed: read back Wish(7); 5. loaded again: "Your 25 SOL stays in her. Change it any epoch.";
5b. a layout frame at 1280 px: the blue door beside the red knock. No swarm stands on the practice chain, so the knock is written by the page's own writer
   (`polis-knock.mjs` doorOf) with stand-in numbers; the frame proves the placement, not a chain read;
6. the way out: every receipt burned, 24,999,995,000 lamports back.

Mended on the way: the rehearsal's last line had an unclosed quote ("the page's"), a parse error after the tests had passed.

### Not verified, and whose

- The knock beside the light on a real swarm (no society on the practice chain); the screens gate (`no-landing-adds-a-failing-screen.sh`) not run by this seat.
- "+ Start a new wish" pointed at `found.html` until the wall's first card landed; since THE NEW WISH STARTS ON THE WALL (below) it opens the wall on that card.
- The pass of the 74 lines against this base (his word), the hand (the integrating seat's), mainnet (her pool born at his sitting) as before.

### The gate

`poleis/door/tests/the_pages.rs` pins the doors on the two pages, the three ways and the glue's gate (no pool, no door, no read); `poleis/press/tests/page.rs`
is the browser's road under the rehearsal. No gate yet on the doors staying hidden on swarmbook.com at the ship: the measure it waits for is a read of the
deployed pages' `[data-her-door]`, `[data-her-light]` hidden and no request to an RPC from `her-door.mjs`, taken at the ship.

## THE NEW WISH STARTS ON THE WALL (Nov 24 2026 ~18:1x UTC, claude-3f06fe2b)

His order: "build all and ship to prod" (Nov 24 2026 17:03:50 UTC), over the drafts whose first row made "+ Start a new wish" a third way for the light
and whose second made the wish the wall's first card (THE WALL SAYS WHO HAS ROOM, `index.html` #wish-card). The door on `after-the-mint.html` linked to
September's wish page, the one he circled "shitty style" at 15:31 UTC; it now opens the wall on the card: `./index.html#wish-card`, still a carried
door (the visit's query rides: `after-the-mint.mjs` carry keeps the fragment). The card hands off to `found.html` with the wish, the name and the room
filled, as the wall's receipt says.

Cost: one attribute of one link; no frame, no request, no line of JavaScript (after-the-mint.html +1 -1).

Changed: `embertide-app/ui/intro/dyson-society/app/after-the-mint.html` (the link's address), `poleis/door/tests/the_pages.rs` (the pin of the three
ways, read at the ship). Run on this lane's tree: `node --test app/after-the-mint.test.mjs`; rule 8, `node --test app/dl-twelve.test.mjs` and, served by
tools/serve-range (206), `node app/dl-twelve.proof.mjs --url <served>/embertide-app/ui/intro/dyson-society/app/warm.html --tap button` and `--tap sphere`,
both PASS. Not run by this seat: `cargo test` of `poleis/door` (the crates gate the ship). It lands behind the wall's lane; before that lane the link
would open the wall with no card on it.
