# SPENT $WARM GOES BACK TO THE MINE: where the mine is named, a tank's $WARM credit is what its flask paid back on the chain, proven on a local chain (Nov 23 2026, claude-a75e1a4b)

## His words, UTC, and what is a seat's

- **HIS PRESS, Nov 23 2026 18:17:41,** in seat claude-f32e080d, to "When someone spends their $WARM as credits, what happens to it on the chain?": he
  pressed "Back to the mine", the option worded "One $WARM, the token: spending it as credits returns it, to be mined again." The option's words are a
  seat's; the press is his (recorded on main by 68039c6e7ff0, `THE-MINE-IS-LIVE-LANDING.md`).
- **HIS, Nov 23 2026 18:54:31,** in the same seat: "Yes, start it", to "Start the build of "$WARM spent goes back to the mine"?". This lane is that build.
- What it stands on, his: Oct 18 "use $WARM as credits for AI genie" (`THE-COUNTER-TANK-DRINKS-LANDING.md`); Nov 19 16:05 "by being spent the money
  goes back into the swarm treasury... re-emitted out again" (`WARM-IS-MINED-BY-USE-LANDING.md`).
- **A SEAT'S, not his:** the design, laid before any code as one figure on the Canvas's Claude page (widget `e4faa851`, his bell's thread `280a490d`):
  the flask pays back ahead, in its page; the door counts it off the flask's record; and the page's moment, its yellow line: the page pours the flask's
  whole $WARM back while its fill form is open, as it pours its USDC, with no new control. "Yours to change": no word of his on it at the hand.
- **WITHDRAWN, the same evening** (lane/the-flask-keeps-its-warm): the page's silent pour landed with this lane (main 4f033a52f9) and is taken out. Two
  seats read it against his press, claude-4b94982a and claude-d276d500, not his word: "spending it as credits returns it" is not "everything mined
  returns at once"; poured whenever the page is open, no person would ever hold $WARM as a token. The page pays nothing back until his word on when;
  the door's road below stands for whichever moment he picks. It reached no one: the page's pay-back runs only where a deploy names the mine.

## Cost first

| | before (main 06aa7ab42c) | after (this lane) |
|---|---|---|
| a tank's $WARM credit, where the mine is named | two things: the ledger's mined $WARM (`warm - warm_spent`) and the token the chain mints to its flask | one, the token: what its flask paid back into the vault (`sr_tanks.warm_back`), less what it drank of it |
| a pay-back on the chain | | the mine's `pay_back` and the compute budget: 2,812 units measured, a limit of 15,000; two signatures (the fee key's, the flask's) and the priority on the limit: 25,000 lamports at the door's floor price, 310,000 at its cap; no rent (the door relays only for a flask whose record stands) |
| the door's watch, a tick, where the mine is named | | one more read (the flask's $WARM account); a blockhash and a price only when it holds $WARM |
| the door's relay of a pay-back | | three reads (the price, the fee key's balance, the record) and one send, under a lock of its own (`back:<flask>`, the deposit's two minutes) |
| the door's count of a pay-back | | one read of the record at finalized, one batch of three statements, one read; a pay-back counts once however often it is asked |
| a call, where the mine is named | the tank's row, the hold, one batch | the same statements, `warm_back` where they read `warm`; a burn whose split another burn drank first is split again (one more read and one more batch, only when two calls race over the last of a credit) |
| a call, no mine named | | every statement and every answer as it was |
| the door's law | `worker/poleis-seat.wasm` 456,465 bytes (main's f3314360615b) | 464,677 bytes (sha256 `09478d571a3e…`), built on the merged tree: the pay-back's message and the two asks, and main's Poleis client since the mine lane |
| D1 | | migration 0032: one column (`sr_tanks.warm_back`), one table (`sr_backs`, a flask's last read) |
| JavaScript | | +68 −68, net 0: the door's edits are in place, the page's is one line, the node tests three lines and one test; paid by five copies of two test stand-ins (a wallet, the site's assets) made one, `worker/stand-ins.mjs` |
| Rust | | `poleis/seat-wasm/src/mine.rs` +108, its tests +54, the live proof `poleis/press/tests/mine.rs` +101 |
| the proof | | 240 s on the new Mac at nice 19 (229 s the test), the validator started and ended by its own script |
| this lane | | one seat, 18:55 to 20:10 UTC; its tokens are not measured here |

Nothing is deployed, nothing was sent to any public cluster, no key of his was made or read, no Canvas tab was born (one figure on the Claude page).

## What stands now

- **The message** (`poleis/seat-wasm/src/mine.rs` `pay_back_message`): one legacy message by Solana's own compiler, the compute budget when a price is named
  and the mine's own `pay_back` of `amount` nanos from the flask's own $WARM account into the vault; the house's fee key its payer, the flask signing
  second as the owner of its $WARM. No memo: the count is the record's, not a text's. `paid_back_of` reads a flask's record's own count of what it paid
  back. The door's one JSON door gains `back` and `paid`.
- **The door** (`worker/space-router-deposit.mjs`), only where `onMine(env)`:
  - the watch (`GET ?flask=`) hands the flask the pay-back of all its $WARM (`warm`: the message, the amount, its source, its record);
  - the relay (`POST {relay: {..., back: true}}`) writes the same bytes again from the same fields by the law, holds the flask's signature over them,
    and sends them only for a flask whose record on the mine stands, so the fee key never pays a record's rent for a flask that never deposited;
  - the count (`POST {back: <flask>}`) reads the flask's record at finalized and makes what its `paid_back` rose since the door last read it the $WARM
    credit of the flask's tank, in one batch with a ledger row (a refill, why `back:<flask>`) and the read kept (`sr_backs`). What the mine counted
    moving into its vault is the pay-back: a plain transfer into the vault is never re-emitted, so it is none.
- **The burn** (`worker/space-router.mjs`), only where the mine is named: the hold counts `warm_back`, the split drinks it after the dollars, the drink
  refuses to take it below nothing (the statement fails and the batch with it), a burn split from a tank another burn drank first is split again from the
  tank as it stands, and the part it paid mines nothing. The ledger's own mined $WARM (`sr_tanks.warm`) is the router's account of what the chain mints
  to the flask, and no credit. What a tank drank of the ledger's $WARM before the mine was named is not taken from it.
- **The page** (`app/dl-kit-deposit.js`): pays nothing back (WITHDRAWN, above). The door's watch offers the pay-back of all of a flask's $WARM; no page
  reads it until his word on the moment.

**What a person sees:** nothing new. The fill form, the address, "Pouring." as before. Their mined $WARM stays in their flask, the token. Where the mine is
named, their agent's calls drink no $WARM until their flask has paid some back (no page does yet); a dry tank's 402 says so ("$0.000000, 0.000000 $WARM").

## The proof, on a local chain

`sh poleis/ceremony/the-mine-is-live.sh poleis/ceremony/spent-warm-goes-back-to-the-mine-2026-11-23.json`: the mine lane's proof (a fresh
`solana-test-validator` holding the $WARM program at genesis, stand-in keys from a seed held in memory, the Worker's own handler under node with the
door's WASM as the tree holds it, a scratch D1 of every migration), then its new section, SPENT $WARM GOES BACK TO THE MINE. The certificate:
`poleis/ceremony/spent-warm-goes-back-to-the-mine-2026-11-23.json` (commit 63849a360cf8, the merged tree, nothing uncommitted).

- **The mine lane's proof stands as it was, on this tree:** four flasks' deposits through the door, twenty-one burns, the crank stopped and resumed across
  the halving, THE BOOK (13.060563500 $WARM issued, the law's over every spend that landed), the refusals, the crank run twice more.
- **The ledger's $WARM is no credit.** Flask d's tank, its dollars drunk past zero: the ledger said it had mined 8.4 $WARM and its flask held 7.1 on the
  chain; its credit was 0, and the door said "Your tank is empty ($-1.800000, 0.000000 $WARM). Refuel it; nothing was sent."
- **The flask pays back, through the door.** Flask d all of its 7.1 $WARM by the page's own road (the watch's message, the flask's signature, the relay,
  the count at finalized), flask b one $WARM of its 2.73: 2,812 units each, each counted once (asked again: 0 and 0); the vault held 8.1 and the mine
  counted it; nothing minted; each tank's credit was what its flask paid back, 7.1 and 1.0, never the ledger's 8.4.
- **The credit is drunk, and mines nothing.** Two calls on flask d's tank at once, 4.2 dollars each, held at the stand-in router until both had split
  their price from the same tank: the first paid 4.2 in $WARM and mined nothing; the second only the 2.9 left, its 1.3 in dollars past zero, which mined
  0.65 on the ledger; the credit stood at 0, never below; the next call was a 402; flask d's record on the chain stayed where its deposits stopped it.
- **The vault pays first.** Flask a's three calls and flask b's one mined 7.875 and 1.05 $WARM (the law's, 17.85 dollars after the halving): 8.1 from the
  vault, 0.825 minted; the supply rose from 13.060563500 to 13.885563500, by the 0.825 alone; the vault at 0; flask b's credit untouched, its dollars
  paid its call.
- **One $WARM.** 13.885563500 in the flasks' own accounts and 0 in the vault: the supply, and all that was ever minted; 8.1 paid back, 8.1 re-emitted. A
  tank's credit drunk and standing is what its flask paid back on the chain, no more:

| flask | paid back on the chain | drunk as credits | credit standing | its flask holds |
|---|---|---|---|---|
| a | 0 | 0 | 0 | 10.741500000 |
| b | 1.000000000 | 0 | 1.000000000 | 2.780000000 |
| c | 0 | 0 | 0 | 0.364063500 |
| d | 7.100000000 | 7.100000000 | 0 | 0 |

## Beside it, at this tip, on the new Mac

- `poleis/seat-wasm`: 10 of 10 (`cargo test`, on the merged tree; the new: the pay-back's message, a record's count, the doors `back` and `paid`).
- `poleis/press`: 85 of 85 (`cargo test`, the whole crate; the live half of `tests/mine.rs` is the ceremony's, above).
- The door's node tests, `worker/*.test.mjs`: 166 of 166. The lander's plan for this lane, 265 files and 1,760 tests: 1,750 pass, 2 skipped, 8 red, none
  this lane's: four of the mint page's own (`dl-athena`, `dl-handoff`, `dl-paywall`, `warm-sphere-moment`) red the same with main's page file in place,
  as the mine lane found them on main 9d20490035; the two crates' `Cargo.toml` entries (node runs no crate: their own suites, above); `nest-air.test.mjs`,
  a timing test, red beside the crates' build and 12 of 12 alone; `the-law-is-the-gate.test.mjs`, red on main by the post-quantum census, whose
  baseline held six files above what they count (this door's `space-router-deposit.mjs` among them, 4 for its 3) and, once their wallet copies left
  them, this lane's three tests too; the census wrote its baseline (only lower numbers: 1,146 lines in 482 files) and the table is 39 of 39.
- The guard: the router's new test fails without the drink's guard and fails without the re-split (two mutants, each red; restored, green).
- Rule 8 (a file the mint page loads changed, `app/dl-kit-deposit.js`): `app/warm.html` served by `tools/serve-range` (206 to a Range request) on
  127.0.0.1:9361: `node --test app/dl-twelve.test.mjs` 21 of 21; `nice -n 19 node app/dl-twelve.proof.mjs --url
  http://127.0.0.1:9361/intro/dyson-society/app/warm.html --tap button`: PASS; the same with `--tap sphere`: PASS.
- The house's post-quantum gate (`tools/embertide-mesh/tests/the-house-signs-post-quantum-only.sh`): exit 0; the lane's one classical line, the stand-in
  flask's key, says so on itself.
- `the-javascript-only-shrinks`: +68 −68, net +0 in 12 files.

## What is not done, and whose

- **His word on the moment a flask pays back** (the figure's yellow line, on the Canvas's Claude page). As it is spent: the flask lets the mine take back
  just what each drink spends, one approval in its page (a delegation of its $WARM to the mine) and the crank returning each drink's $WARM part as it posts
  the spends; the closest to his press, and a change to the $WARM program and the crank. At a press: "Use my $WARM as credits" on the page, a new control,
  mocked on his screen before it is built. All of it whenever the page is open: built, landed and withdrawn.
  Since SPENT AS CREDITS, IT RETURNS (`SPENT-AS-CREDITS-IT-RETURNS-LANDING.md`): the door's and the chain's half of the first is built. A flask keeps its
  $WARM; once it approves the mine as its delegate, the crank brings back just what its tank drank (the program's `take_back`). The page signs that approval
  once, silently: his 18:17:41 press is the form (taken out at 21:42 on a seat's misreading, put back by lane/spending-warm-returns-it); $WARM is the XP
  (his 20:42:25 UTC), a level what was mined.
- **A flask whose $WARM came by a plain transfer and that never deposited through the mine** has no record on the mine: the door's relay refuses it (the
  house pays no record's rent); it can pay back by paying its own fee.
- **Tanks whose dollars never deposited through the mine** (a card's, a stake link's, his pours, every deposit before the mine): their spends mine on the
  ledger alone, so their flasks hold no token to pay back, and where the mine is named they drink no $WARM. The road for them is the mine lane's
  (`record_prior`, or a `deposit` naming its payer and its flask apart).
- **A pay-back whose page closed before it was final** is counted at the flask's next count (its next pay-back's, or anyone's `{back: <flask>}`), not
  before: the count reads the record's whole `paid_back`, so it is late, never lost.
- ~~**The page signs the door's pay-back without reading it back** (as it signs the deposit: a check in the page is owed, the mine lane's word).~~
  Struck with the deposit's (THE-MINE-IS-LIVE-LANDING.md, Nov 25 2026): the same origin, the same law; nothing a check in the page would add.
- **Not run on D1 itself** (node's SQLite stands in), **nor under mainnet's rules**, **nor inside a Worker**.
- **No deploy names the mine yet.** His press at the sitting (THE SWARM NAMES ITS MINER: the sixth act), then his deploy of the door with the sitting's
  WASM, migrations 0031 and 0032, the vars `press issuer` says and the miner's secret.
