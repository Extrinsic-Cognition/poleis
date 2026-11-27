# THE MINE IS LIVE: a deposit is counted by the mine and a crank posts the router's signed spends, proven on a local chain (Nov 23 2026, lane/the-mine-is-live, claude-d276d500)

## His words, UTC, and whose this lane is

- **HIS, Nov 21 2026 17:34:37,** to "Is $WARM's supply fixed at 1 billion, or can it keep growing?": "it is supposed to be uncapped supply, prompt-2-mine;
  the issuance needs to be live and continuous based on the space-router utilization".
- **HIS,** the law the router keeps (`app/space-router-law.mjs`): Oct 27 "1$ = 1$WARM? to begin with"; Oct 28 13:50 "we should give $WARM only when
  money is spent in the SPACE-ROUTER"; Oct 29 "Halve by date", "Every 6 months"; Nov 19 12:33, the flask's minimum refill is 10 USDC.
- **A SEAT'S,** not his: this lane was offered to him once (Nov 22 2026 00:03 UTC, "Door and crank first") and he did not press it. It was started on
  Nov 23 by the session that started this seat, on the mine's own receipt (`WARM-IS-MINED-BY-USE-LANDING.md`, "What is not done: the live door and the
  crank (the next lane)"). No word of his names this lane. Nothing here is live until his own deploy names the mine.

- **HIS PRESS, Nov 23 2026 18:17:41, after this lane landed** (main 3f0723d5ecc2, 18:15:56), on the question this receipt left him. Asked by the
  session that started this seat (claude-f32e080d, at 18:16:08): "When someone spends their $WARM as credits, what happens to it on the chain?" He
  pressed "Back to the mine (Recommended)", the option whose words were: "One $WARM, the token: spending it as credits returns it, to be mined
  again. Your Oct 18 "use $WARM as credits" and Nov 19 "by being spent... re-emitted"." The other was "Credits stop". Read by this seat in that
  session's transcript, not relayed alone. The option's words are a seat's; the press is his.

## Cost first

| | before (main 9d20490035) | after (this lane) |
|---|---|---|
| $WARM on a chain | the program stands and nothing reaches it: the door sends a deposit straight to the treasury, nothing posts a spend | a flask's deposit is the mine's own `deposit`; every burn that mined starts a crank that posts the router's signed receipts to `mine` |
| a flask's deposit, on the chain | one transfer and a memo, 23,323 units (mainnet, Nov 20) | the mine's `deposit` and the memo, 30,189 to 37,677 units (a local validator), under the 45,000 it asks |
| what the house pays a new flask, once | the deposit's fee | the flask's record (1,607,760 lamports of rent, the fee key) and its $WARM account (2,039,280, the miner's key, at its first mined spend): 0.0036 SOL a flask and the fees; 3.65 SOL a thousand flasks |
| a deposit after the first, a batch | | the fee key 55,000 lamports at the door's floor price; the miner 5,000 a batch, however many spends it holds |
| a batch | | 4 spends in 862 bytes, 76,864 units (four $WARM accounts made); 7 spends in 944 bytes, 23,470 units; a flask's spends of one era are one entry however many calls they were; seven flasks' entries fit a packet (1,200 of its 1,232 bytes, measured), and a flight posts up to four batches |
| the crank | | started by a burn, never a timer; one flight a door; a flight: 2 reads of the chain, 1 of D1, 1 of the records, 1 blockhash, 1 send, then the cursor read until it moves; nothing at rest |
| the door's law | `worker/poleis-seat.wasm` 313,235 bytes (the seat's) | 456,465 bytes (sha256 `f3314360615b…`): the seat's and the mine's, one module, no imports |
| the plan's cost | | a page of 128 receipts planned in 17.5 ms in the WASM under node on the new Mac (137 µs a receipt, its signature held to the oracle with it); a deposit's message 0.13 ms |
| JavaScript | | +72 −159, net −87: the door's edits are in place, the proof's harness is 16 lines, and the devnet deposit proof (`tools/space-router-deposit.proof.mjs`, 136 lines, run by no gate; "devnet is no step of his") is taken out |
| Rust | | `poleis/seat-wasm/src/mine.rs` 466 lines, its tests 282, the live proof `poleis/press/tests/mine.rs` 482 |
| the proof | | 173 s on the new Mac at nice 19, the validator started and ended by its own script |
| this lane | | one seat, 17:20 to 18:06 UTC; its tokens are not measured here |

Nothing is deployed, nothing was sent to any public cluster, no key of his was made or read, no Canvas tab was born (one figure on the Claude page).

## What stands now

**The deposit is counted** (`poleis/seat-wasm/src/mine.rs` `deposit_message`; `worker/space-router-deposit.mjs`). Once his deploy names the mine
(`SPACE_ROUTER_MINE`) and it is the one the door's WASM speaks, the door's watch hands the flask one message to sign: the terms' memo and the mine's
`deposit`. The house's fee key pays the fee and the record's rent; the flask signs second, as the owner of its USDC, as today. The relay writes the same
bytes again from the same fields and sends only what the flask signed over them. The dollars go to the mine's treasury, which is root 0's own vault of
the dollar (`press` derives it, `chain::treasury_of`; `SPACE_ROUTER_TREASURY` names it), and stand in the flask's record on the chain. The door's own
count (`verifyDeposit`: what moved) is unchanged, so the tank fills as before. With no mine named, or another, the door is as it was, byte for byte.
A person paying a $10 egg sees nothing new: the same address, the same 10 USDC, "Pouring.", the egg.

**The crank** (`mine.rs` `plan`; the door's `crank`, started in `worker/premint.mjs` after a burn's answer).
- THE CURSOR IS THE CHAIN'S. A spend's `seq` is its receipt's id, so the mine's own `last_seq` says where the crank stopped. Nothing else is kept as
  truth: a crank run twice reads the same cursor, and the chain refuses a seq it holds, so two cranks at once land one batch.
- Each receipt is held to the oracle's key before it counts: its signature over its own canonical bytes, its tank, its instant, and its dollars no
  less than the fuel the ledger says it burned (the part a tank paid in $WARM mines nothing, as on the ledger). A receipt names its burn since
  migration 0031 (`sr_receipts.burn`, written in the burn's own batch), so each signed receipt is read beside its ledger row.
- A flask's spends of one era are one entry: their nano-dollars summed, floored to the chain's micro-dollar, stamped with the last one's instant and
  never before the spend mined before it. Across a halving a flask has two entries, each at its own rate.
- THE BOUND, read from the flask's record on the chain: a spend mines as far as its flask's deposits go, the rest mines nothing.
- A receipt the chain's clock has not passed waits, and everything after it.
- WHAT CANNOT MINE NEVER HOLDS THE LINE: receipts that mine nothing (a tank past its deposits) are read past, and how far is noted in D1 (`sr_crank`,
  one row: a place to read from, never what was mined), so they cannot starve a spend behind them.
- One flight a door: a burn that comes while a crank flies joins it.

**The press** says the door's three vars of the mine (`press issuer`: `SPACE_ROUTER_MINE`, `SPACE_ROUTER_TREASURY`, `SPACE_ROUTER_OPENED_AT`) and no
longer refuses a mine whose miner is named: only the mine's authority names one, and the press holds that authority to the polis's hand.

## The proof, on a local chain

`sh poleis/ceremony/the-mine-is-live.sh`: a fresh `solana-test-validator` holding the $WARM program at genesis, stand-in keys from a seed held in
memory, the Worker's own handler under node (`worker/the-mine-is-live.proof.mjs`) with the door's WASM as the tree holds it, a scratch D1 of every
migration, a stand-in MiniRouter. The certificate: `poleis/ceremony/the-mine-is-live-2026-11-23.json` (commit 50ae46b9d2ea).

- **The deposits, through the door.** Four flasks holding USDC and no SOL deposited 20, 10, 10 and 10 through the watch, the flask's signature and the
  relay; the door counted each at finalized; each flask's record on the chain said its dollars; the treasury held 50.000000 and the mine counted it.
- **The spends.** Twenty-one burns through the door's chat. The first nine with the miner's key on the door: the crank posted them (the mine's cursor 9).
  Then the key was taken off (a Worker without its secret): seven burns waited, three of them before the halving. The key back, the next burns started
  the crank beside two more cranks at once: one batch landed, the cursor at 20, the three spends from before the halving posted after it and stamped
  before it.
- **The book, across the halving:**

| flask | deposited | the chain: spent | the ledger: spent | the chain: $WARM | the ledger: $WARM |
|---|---|---|---|---|---|
| a (whole micro-dollars) | 20.000000 | 3.402000 | 3.402000000 | 2.866500000 | 2.866500000 |
| b (whole micro-dollars) | 10.000000 | 3.780000 | 3.780000000 | 2.730000000 | 2.730000000 |
| c (nano-dollars) | 10.000000 | 0.381406 | 0.381407084 | 0.364063500 | 0.364064255 |
| d (poured 5 past its deposit) | 10.000000 | 10.000000 | 11.550000000 | 7.100000000 | 7.875000000 |

  Flasks a and b: the chain is the router's ledger to the micro-dollar and the nano of $WARM. Flask c: each entry floored to the micro-dollar, 1,084
  nano-dollars and 755 nanos of $WARM under the ledger over its entries, never over. Flask d: mined up to its 10 dollars and no further; the ledger let
  it burn its pour. Issued 13.060563500 $WARM, the law's over every spend that landed, the supply what was minted; the ledger says 13.835564255.
- **The refusals, by the chain:** one micro-dollar past flask d's deposits (PastDeposits, 11), a seq the mine holds (SeqNotRising, 8), a stranger's
  batch (Unauthorized, 0), a spend before the last (Backwards, 9), one after the clock (Future, 10); the same batch inside flask a's deposits would land.
- **A spend past its deposits, alone:** nothing posted, the mine's bytes the same, and the door read past it.
- **The crank run twice more:** 0 and 0 sent, no transaction, the mine's bytes and the supply the same.

Beside it, at this tip, on the new Mac: `poleis/seat-wasm` 9 of 9 (6 the mine's: the router's own receipts held as the crank holds them, the deposit's
message, the plan, the packet, the JSON door); `poleis/press` 78 of 78; the door's and the page's node tests 206 of 206; rule 8,
`embertide-app/ui/intro/dyson-society/app/warm.html` served by `tools/serve-range` (206): `node --test app/dl-twelve.test.mjs` 21 of 21,
`node app/dl-twelve.proof.mjs --tap button` PASS and `--tap sphere` PASS. The house's post-quantum gate: exit 0 (the lane's classical lines, Solana's
Ed25519 and the oracle's, say so on themselves). Of the 281 node test files the lander's plan names for this lane, five are red here and red the same
on main's own checkout (9d20490035): `dl-athena.test.mjs`, `dl-handoff.test.mjs`, `dl-paywall.test.mjs`, `warm-sphere-moment.test.mjs`,
`tools/storyboard-canvas.test.mjs`. They are main's, not this lane's, and not touched.

## What his press at the sitting must then do, in one line

After the five acts, the tree seals one more raw act of the polis's hand, `set_miner`, naming the door's miner key; then his deploy of the door carries
the sitting's WASM, migration 0031, the three vars `press issuer` says and the miner's secret (`SPACE_ROUTER_MINER_KEY`), with SOL on the miner's key.

## What is not done, and whose

- **The press writes no `set_miner` act and makes no miner key** (the next lane; until it, mainnet's mine names no miner and mines nothing).
  Since THE SWARM NAMES ITS MINER (Nov 23 2026, `poleis/press/THE-SWARM-NAMES-ITS-MINER-LANDING.md`): `press miner` makes the door's crank
  key and the press's sixth act names it, passed by the whole tree; rehearsed with a deposit and a spend through the door's code on the
  press's own world.
- **RULED, NOT BUILT: a drink in $WARM goes back to the mine** (his press of 18:17:41, above). As the door stands, $WARM would be two things once
  the mine is named: the ledger still counts a tank's mined $WARM as credits it can drink (THE COUNTER-TANK DRINKS, `USD_PER_WARM` 1), and the chain
  would mint the same $WARM as a token in the flask's own account. His ruling makes it one, the token: spending it as credits returns it to the
  mine's vault (the program's `pay_back`, which the flask signs; the vault pays the next spends before anything is minted). What that asks of the
  door, a seat's reading and no design of his yet: where the mine is named, a tank's $WARM credit is what its flask paid back on the chain, never
  what the ledger says it mined; a call cannot sign for a flask, so the flask pays back ahead in its page, as it deposits. The mine must not be named
  by a deploy before this is built. This lane changed none of it.
  Since SPENT $WARM GOES BACK TO THE MINE (Nov 23 2026, `SPENT-WARM-GOES-BACK-TO-THE-MINE-LANDING.md` beside this one): built and proven on a local
  chain. Where the mine is named, a tank's $WARM credit is `sr_tanks.warm_back` (migration 0032), raised only by what its flask's record on the
  chain says it paid back (read at finalized, counted once) and lowered by its drinks, never below nothing; a drink of that credit mines nothing; the
  vault paid the next spends before anything was minted. When a flask pays back is his: the page's silent pour of all of it landed and was taken out
  the same evening (lane/the-flask-keeps-its-warm); then SPENT AS CREDITS, IT RETURNS (`SPENT-AS-CREDITS-IT-RETURNS-LANDING.md`): the flask keeps its
  $WARM and, its page approves the mine once, silently, and the crank brings back just what its tank spent as credits (taken out at 21:42 on a seat's
  misreading, put back by lane/spending-warm-returns-it).
- **Dollars that are not a deposit through the mine do not mine on the chain:** a card's (no USDC moved), the house's pour for a flask (the fee wallet
  is its payer on the chain), a stake's rewards, his pours, a wallet's plain transfer, every deposit before the mine. Their tanks mine on the ledger as
  before. The road for them is `record_prior` (a sealed act a deposit) or a `deposit` that names its payer and its flask apart (a program change).
- **A tracenet bought from the tank** mines on the ledger and has no signed receipt: not posted.
- **A spend past its flask's deposits when the crank reads it** mines nothing for good, also if the flask deposits later.
- **Under a micro-dollar of an entry is not mined** (the chain never mines more than the ledger).
- **Not run on D1 itself** (node's SQLite stands in; the receipt's `burn` is a subquery in the burn's batch), **nor under mainnet's rules** (the
  validator ran every feature it knows; reading mainnet's feature set is his word each time), **nor inside a Worker** (its 30 s after an answer can
  cut a flight: the cursor is the chain's, the next burn resumes).
- **The launch rehearsal was not run again** by this lane (the press's change is the vars and the miner's words; its suites are green).
  Since THE WHOLE MAINNET, REHEARSED (Nov 25 2026, `poleis/ceremony/THE-WHOLE-MAINNET-REHEARSED-LANDING.md`): run whole under mainnet-beta's feature set, its mine named by the sixth act, a deposit and a spend through the door's own code on it and the
  miner's pull, exit 0 in 2,394 s. Under mainnet's rules the deposit and the mine are rehearsed; the door still under node, not inside a Worker, nor on D1.
- ~~**The page signs the door's message without reading it back.** The door is the page's own origin; a check in the page is owed.~~
  Struck Nov 25 2026 (claude-43fa08e7): no check in the page adds what it would guard. The page and its door are one Worker deploy on one origin, so a door that lies serves a page that lies; the page's only rebuild is the same law (poleis/seat-wasm mine.rs, the door's WASM) over the terms the same door says, so a wrong law or a wrong var passes both; the page holds no node of its own to read the mine's treasury by (a public node refuses browsers). Its cost: the law's 492 KB WASM on every pour, or the message's reading in Rust in the page's own door (poleis-door.wasm) with the mine's constants beside it, and the page's glue paid in JavaScript. The guard that holds is where the message is made: the law's own tests and `poleis/press/tests/mine.rs` (each flask signs the door's message on a local chain, its deposit, its approval and the take-back).
- The treasury of the proof's chain is a stand-in of root 0's vault (no Poleis stands there); that the launch's is that vault is the press's check.
