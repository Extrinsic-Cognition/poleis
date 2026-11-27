# SPENT AS CREDITS, IT RETURNS: a flask keeps its $WARM, and only what its tank drinks as credits goes back to the mine, proven on a local chain (Nov 23 2026, claude-a75e1a4b)

## His words, UTC, and what is a seat's

- **HIS PRESS, Nov 23 2026 18:17:41,** in seat claude-f32e080d: "Back to the mine", the option worded "One $WARM, the token: spending it as credits
  returns it, to be mined again." (`THE-MINE-IS-LIVE-LANDING.md`.)
- **HIS, Nov 23 2026 20:10:15 and 20:11:18,** in this seat: "u done all?", then "do what is missing?", to the answer whose one open item was his: when a
  person's flask pays its $WARM back (three ways on the Canvas's Claude page, widget `30f7f3cf`, and on the launch sheet of claude-f32e080d). Read as his
  word to choose and build it. **The choice is a seat's:** the first of the three, "as it is spent", the one his press names; the figure called it the
  closest to his press.
- **Not his word, two seats' reading** (claude-4b94982a, claude-d276d500), which took out the silent pour of everything (lane/the-flask-keeps-its-warm):
  no person would ever have held $WARM as a token. Here the token stays in the flask until it is drunk.
- **Withdrawn, then put back, the same evening** (lane/the-drinks-form-is-his landed at 21:42:40 UTC as 452d746d3f; lane/spending-warm-returns-it
  puts it back). The withdrawal was a seat's misreading, this seat's: the session that started it relayed his words of 20:10:56 to 20:42:25 UTC in
  claude-f32e080d as "the drink's form stands again as his question". Those words were over that seat's design of a Lyssajudron upgrade that spent gold,
  never over the pay-back: "dude, you miss the entire gamification of the upgrades of the lyssajudron... that is problematic my dude... hypertrace"
  (20:10:56), "the fuck you are eager to build dude? design porcoddio" (20:27:20), "what? i dont get what would change?" (20:41:12), "it's xp diocane. it
  is prompt-2-mine dude... $WARM is the xp dude..." (20:42:25): a level is what was mined and never costs gold. The form was his already. His press of
  18:17:41, "spending it as credits returns it", is each call that spends $WARM returning that $WARM; the other form (a person's press, "Use my $WARM as
  credits") was this seat's own option, on no word of his. At 20:55:37 and 21:00:57 UTC in claude-4b94982a he was told it in plain words, over his
  21:00:44 "what the fuck are drinks dude ??" ("The person approves this once on their page, and from then on each prompt they pay in $WARM returns
  just that amount"), and he did not object. Asked here "have you done all?" (21:35:40), "what is missing?" (21:42:49) and "wdym? hypertrace" (21:59:38),
  the trace found nothing of the form his: the page signs the one approval again, as 0dd5a1b31d landed it. "Drink" is the seats' word (Nov 2), never his:
  one AI call paid from a tank.

## Cost first

| | before (lane/the-flask-keeps-its-warm, ecf828fd63) | after (this lane) |
|---|---|---|
| a person's $WARM, where the mine is named | stays in their flask; their tank drinks none of it as credits | stays in their flask; their tank spends it as credits after its dollars (its page signs the one approval); just what it spends goes back to the vault |
| the $WARM program | eight instructions | nine: `take_back`, 2,371 units for one flask (Mollusk, the real token program); no account's layout changes; the program 63,600 bytes, main's 61,288: 2,312 more, about 0.012 SOL more rent at its deploy |
| a flask's one approval | | the token program's ApproveChecked of its $WARM account to the mine, for all of it: one transaction a flask, the house's fee key paying, 460 units measured on a local validator (two signatures and the priority on a limit of 15,000: 25,000 lamports at the door's floor price) |
| a take-back | | after a burn that spent $WARM, in the crank's same flight: two accounts read a flask, one transaction a batch (up to 12 flasks a packet), 2,371 units each measured, the miner paying 5,000 lamports a batch; nothing at rest |
| the door's count | a flask's record | its record and its $WARM account, one read; the credit moves both ways (a delegation taken away takes its credit) |
| the door's law | `worker/poleis-seat.wasm` 464,677 bytes | 476,388 bytes (sha256 `4f8ca8774315…`) |
| D1 | | migration 0033: `sr_tanks.warm_drunk`, `sr_backs.held` |
| JavaScript | | +19 −19, net 0: every edit in place; the withdrawal +2 −2 and its undoing +2 −2, each net 0 |
| Rust | | +460 −47: the program's `take_back` +62 and the approval's layout +13, their tests +85; the WASM +150 −14 and its tests +79; the live proof +65 −30 |
| the proof | | 338 s on the new Mac at nice 19 (330 s the test) at a load average of 130 to 170; the validator started and ended by its own script |

Nothing is deployed, nothing was sent to any public cluster, no key of his was made or read, no Canvas tab was born.

## What stands now

- **The program** (`src/ix.rs` `take_back`): by the miner alone, each flask's record's `paid_back` is brought to its target (what its tank drank as credits,
  as the router's ledger says it), the difference moving from the flask's own $WARM account into the vault, the mine signing as the account's delegate. A
  target, never an amount: a batch sent twice moves nothing twice. The token program holds the rest: an account whose delegate is not the mine, or whose
  delegation or balance is short, refuses the whole batch. The vault pays the next spends before anything is minted, as a pay-back's does.
- **The WASM** (`poleis/seat-wasm/src/mine.rs`): `allow_message` (the approval, its bytes held to the token program's by `tests/wire.rs`), `held_for_mine`
  (what a flask holds for the mine), `takes` (the plan: what each owing tank drank past what its flasks paid back, from its flasks in order, as far as each
  holds, each flask once, one packet), `take_back_message`; asks `allow`, `owing`, `takes`, and `paid` with the $WARM account.
- **The door**, only where the mine is named: the watch offers the one approval while the mine is not the delegate of all of a flask's $WARM; the relay sends
  it (`allow: true`); the count reads the record and the $WARM account; the crank, started by a burn that spent $WARM too, counts what it reads and takes
  back what was drunk. **The burn** counts what a tank drinks as credits (`warm_drunk`).
- **The page** signs the approval once, silently, and asks the door to count what its flask holds; it never moves a token (put back, above).

**What a person sees:** nothing new. Their $WARM stays in their flask. Their agent's calls spend it as credits once the dollars are gone, and just what is
spent goes back to the mine.

## The proof, on a local chain

`sh poleis/ceremony/the-mine-is-live.sh poleis/ceremony/spent-as-credits-it-returns-2026-11-23.json` (MINE_HALVING_AFTER=240 on a loaded Mac): the mine
lane's proof as it was, then SPENT AS CREDITS, IT RETURNS. The certificate: `poleis/ceremony/spent-as-credits-it-returns-2026-11-23.json` (commit
2abae59c64, the merged tree, nothing uncommitted).

- **The mine lane's proof stands as it was:** four deposits through the door, twenty-one burns, the crank stopped and resumed across the halving, THE BOOK
  (13.060563500 $WARM issued, the law's over every spend that landed), the refusals, the crank run twice more.
- **The ledger's $WARM is no credit.** Flask d's tank, its dollars drunk past zero: the ledger said it mined 8.4 $WARM, its flask held 7.1; its credit was 0
  and the door said "Your tank is empty ($-1.800000, 0.000000 $WARM). Refuel it; nothing was sent."
- **The flask keeps its $WARM.** Flask d's page road through the door: the watch's approval, the flask's signature, the relay, the count at finalized: 460
  units, nothing moved, its flask still holding 7.1, all of it held for the mine; its tank's credit 7.1. Flask b paid one $WARM back ahead instead (2,813
  units): its credit 1.0. Each counted once (asked again: 0 and 0).
- **Spent as credits, it returns.** One call of 1.05 dollars drank flask d's credit and mined nothing; the crank its burn started brought back just the 1.05
  (2,371 units), and flask d kept 6.05. Then two calls at once, 4.2 dollars each, held at the router until both had split their price from the same tank:
  4.2 and the 1.85 left in $WARM, the rest in dollars past zero (which mined 1.175 on the ledger); the credit stood at 0, never below; the crank brought back
  the 6.05 (2,371 units); flask d's flask at 0, its record's `paid_back` 7.1; the next call a 402; nothing of the credit's drinks posted as a spend. The crank
  run twice more brought nothing back twice; the chain refused a stranger's take-back and one past what a flask holds.
- **The vault pays first.** Flask a's three calls and flask b's one mined 7.875 and 1.05 $WARM (the law's, 17.85 dollars after the halving): 8.1 from the
  vault, 0.825 minted; the supply from 13.060563500 to 13.885563500, by the 0.825 alone; the vault at 0; flask b's credit untouched, its dollars paid its call.
- **One $WARM.** 13.885563500 in the flasks' own accounts and 0 in the vault: the supply, and all that was ever minted; 8.1 came back, 8.1 re-emitted. A
  tank's credit drunk and standing is what its flask paid back and holds for the mine, no more:

| flask | paid back on the chain | held for the mine | drunk as credits | credit standing | its flask holds |
|---|---|---|---|---|---|
| a | 0 | 0 | 0 | 0 | 10.741500000 |
| b | 1.000000000 | 0 | 0 | 1.000000000 | 2.780000000 |
| c | 0 | 0 | 0 | 0 | 0.364063500 |
| d | 7.100000000 | 0 | 7.100000000 | 0 | 0 |

## Beside it

- `poleis/program/warm_mine`: 40 of 40 (`cargo test`, the tests building the program they load; the new: the take-back by the flask's own approval and
  nothing twice, the token program's refusals, ApproveChecked held to the token program's bytes, the take-back's cost).
- `poleis/seat-wasm`: 11 of 11 (the new: the approval's bytes, what a flask holds for the mine, the take-back plan, the asks).
- `poleis/press`: 94 of 94 (the whole crate; the live half of `tests/mine.rs` is the ceremony's, above).
- The door's node tests, `worker/*.test.mjs`: 166 of 166 (the mine test walks the approval, the count moving both ways; the router's guard test asserts what a
  tank drank as credits). The lander's plan, 263 node files and 1,758 tests: 1,752 pass, 2 skipped, 4 red, the four mint-page tests red on main itself
  (`dl-athena`, `dl-handoff`, `dl-paywall`, `warm-sphere-moment`).
- Rule 8 (`app/dl-kit-deposit.js` changed under the mint), on `app/warm.html` served by `tools/serve-range` (206): `node --test app/dl-twelve.test.mjs` (with
  `app/dl-kit-deposit.test.mjs`) 31 of 31; `nice -n 19 node app/dl-twelve.proof.mjs --url http://127.0.0.1:9361/intro/dyson-society/app/warm.html --tap
  button`: PASS; the same with `--tap sphere`: PASS.
- `the-javascript-only-shrinks`: +19 −19, net 0.

## What is not done, and whose

- **Nothing of the form is his** (above): his 18:17:41 press is the form. A press for the person ("Use my $WARM as credits") would be a new control, mocked
  on his screen before it is built, and only on a word of his asking for one.
- **The launch rehearsal on the new program** (`poleis/ceremony/the-launch-press-rehearsal.sh`, Nov 23 2026 21:40 to 22:07 UTC, 1,598 s, at nice 19 beside
  another seat's rehearsal on the new Mac): the sitting ran (the press built and deployed the three programs, this program's new bytes among them; root 0 in
  genesis; the seal's road); its council checks stopped at a wait, "the validator's clock stood still before slot 963" (slot 272 after 927 s), the loaded
  Mac's validator not advancing, before the mine's step. Not a check that failed, and not run again: owed on a quieter Mac.
- **Between a drink and its take-back** (one crank flight, seconds) the drunk $WARM is still in the flask. A flask that takes its approval away in between
  (only its key can) keeps what it drank: the door's next count takes its credit away, and the crank's next plan leaves it out.
- **The crank counts at confirmed**, the page's count at finalized; an older read never overwrites a newer one.
- **A tank whose page is closed** has its newly mined $WARM counted as credit at its next count (its page's next visit, or a crank flight over it).
- **Tanks whose dollars never deposited through the mine** (a card's, a stake link's, his pours) hold no token and drink no $WARM where the mine is named.
- **The launch rehearsal was not run again** with the new program (its `init`, `set_miner`, `deposit` and `mine` are unchanged; its bytes are not).
  Since THE WHOLE MAINNET, REHEARSED (Nov 25 2026, `poleis/ceremony/THE-WHOLE-MAINNET-REHEARSED-LANDING.md`): run whole with this program's bytes under mainnet-beta's feature set, exit 0 in 2,394 s (the quieter run the line above owed).
  **Still owed: the take-back itself under mainnet's rules.** The rehearsal's mine step (`worker/the-mine-is-live.proof.mjs`) deposits and spends; it
  makes no approval and no `take_back`, which stand only on the press's local chain (`poleis/press/tests/mine.rs`). Its cost: an approval, a drink of
  $WARM and a crank flight in that proof, their JavaScript paid by as many lines out, and a rehearsal of about 40 minutes on a quiet Mac.
  Since THE DOOR TAKES BACK (Nov 25 2026, lane/the-door-takes-back, claude-43fa08e7): the launch rehearsal's sitting takes it back. Step 6c of
  `poleis/program/poleis_treasury/tests/council.rs` (`the_door_takes_back`, after the month's draw of 6a and 6b, on the same door's proof kept open: no
  JavaScript, the proof already spoke `back`, `tank`, `call` and `crank`): the flask that mined drinks its dollars dry, its page's one approval goes
  through the door with the house's fee key its payer, one call of a dollar drinks 1.05 $WARM of its credit and mints nothing, and the door's crank
  brings back just those into the mine's vault; the crank run again brings nothing. Run whole (`the-launch-press-rehearsal.sh`, `LAUNCH_AS_MAINNET=1`:
  mainnet's priority fee, rung 5 and a list of 100; the validator's own features, no read of mainnet; the agents' mind qwen3:14b on the old Mac over
  loopback), exit 0 in 1,933 s on the new Mac at nice 19: "THE DOOR'S MINE TAKES BACK: the flask approved the mine for its 10000000000 nanos of $WARM
  through the door, one call drank 1050000000 of them as credits and minted nothing, and the door's crank brought back just those into the mine's vault
  (0 to 1050000000); the supply stood at 10000000000; 38 s". **Still owed: under mainnet's own feature set**, a run with `LAUNCH_FROM_MAINNET=1`,
  which reads mainnet at the validator's genesis and so waits for his word for that run.
- **Not run on D1 itself, nor under mainnet's rules, nor inside a Worker.** No deploy names the mine yet: his sitting, then his deploy with migrations 0031 to 0033.
