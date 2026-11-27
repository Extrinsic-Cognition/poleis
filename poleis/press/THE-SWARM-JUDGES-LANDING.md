# HIS ANSWERS ARE IN THE PARAMS, AND THE SWARM JUDGES: a vote stays open a day, the fund takes 2.5%, the market steps a minute, a road's worth is said by the swarm's own vote, and the sheet's picks stand (Nov 23 2026, claude/awesome-leavitt-c36a4a, seat claude-0a8557d2)

## His words, UTC

Asked at 19:28:34 and pressed at 19:32:23 in seat claude-f32e080d (read by this seat in that session's own transcript, not taken from the
relay alone; the questions and the options' words are a seat's, the presses are his):

| the question | his press | the option's words | in the params |
|---|---|---|---|
| "Mainnet: how long does a vote stay open? It is 6 minutes now, a test's pace." | "1 day (Recommended)" | "The same day you gave the seats." | `genesis.window`, `posiverse.window`: 216,000 slots (a day at 400 ms) |
| "Mainnet: what cut does the fund take of each seat sale?" | "2.5%" | "What devnet runs today." | `fund.feeBps`: 250 |
| "Mainnet: how fast does the market move? ..." | "1-minute steps, 1 day (Recommended)" | "A price step a minute; results judged within a day of the vote's close." | `market.stepSlots`: 150; `market.judgeSlots`: see below |
| "Mainnet: who reports what a passed market road was worth, so its bets pay out?" | "The swarm, by its own vote" | "No key decides (your Nov 22 ruling); slower: a vote for each result." | no `market.judge`: root 0's own hand, derived |

**The sheet's picks, his words at 19:37:50,** over the sheet of the 25 values still open that seat showed him at 19:35:39 (after his
19:34:29 "Show me first"): "i dont give a fuck, you think these are good, they stay. u are thinking all is best?". Written as the sheet
picked them: `genesis.defaultMint` USDC (EPjFWdd5...), `genesis.planCap` and `posiverse.planCap` 1 SOL a period, `genesis.rung` 5 (150
seats), `floor.startLamports` 1,000,000, `floor.stepLamports` 100,000, `floor.opensAfterSlots` 200, `posiverse.name` Nefertiti,
`fund.cycleSlots` 6,480,000 (30 days), `fund.rangeLo` 2,000 and `fund.rangeHi` 8,000 bps, `fund.optionFee` 1,000,000, `market.board` 77,
`market.maxStepBp` 100, `market.thresholdBp` 300, `market.feeBp` 30, `market.poolBase` and `market.poolQuote` 1,000,000,000,
`market.split` [treasury], `market.plan` [treasury, act], `market.barBp` 1,000, `pool.tag` u0a, `pool.programSha256` d18a2cdc...c52ac.

**Two of them are the seat's, not his press** (claude-f32e080d's, told to him at 19:40:00 in the same session; no word of his since):
`priorityMicroLamports` 100,000 where the sheet had 0 (the founding must land in the deploy's breath on a busy day; about 0.0003 SOL for
the whole sitting, it told him), and `pool.maxValidators` 100 where it had 10 (the list's length is fixed at the pool's birth, and his own
validator set across many snowtracers needs more than 10; about 0.033 SOL more rent, it told him). **Both of its numbers were low**, measured
here: the deploy's 587 buffer writes alone carry 0.000587 SOL of priority fee at 10,000 units each, so the sitting's is nearer 0.001 SOL;
the list of 100 is 7,309 bytes against 739, 0.0457 SOL more rent by the validator's own `getMinimumBalanceForRentExemption`. The params'
note now says both.

**Left for the sitting** (PLACEHOLDER): the keys it makes, `floor.issuer`, `market.crediter`, `warm.miner` and `posiverse.agents` (the
agents' road is `claude/hopeful-almeida-e6d0d3`'s), and `warm.opening`, now worded as what it must be: the exact instant the live Worker
holds as SPACE_ROUTER_OPENED_AT (Nov 15 2026 by its receipts), read at the sitting, so the chain's halvings and the router's ledger keep
one clock.

**The judging window, a seat's reading, not his press:** `market.judgeSlots` is 432,000 slots, two days, where his market press said one.
The swarm says a road's worth by a vote of root 0 (and unit 1's of it), and since his first press that vote can itself stay open a day; a
judging window of one day could close before the vote and its execution. The seat that asked him (claude-f32e080d) owns this reading and
told him; his word moves it. The press now refuses a judging window no longer than a vote.

## The cost, first

| | before (main 06aa7ab42c) | after (this lane) |
|---|---|---|
| mainnet's params | 38 values PLACEHOLDER | 7, all the sitting's own (six keys it makes, the opening it reads); 30 named, `market.judge` taken out |
| the sitting's cost these values move | | the priority fee on every transaction the press sends, nearer 0.001 SOL for the sitting (the deploy's 587 writes alone 0.000587; the rehearsal of mainnet's tip measures the whole); the pool's list for 100 validators, 0.0457 SOL more rent; a seat's price 0.001 SOL rising 0.0001 a seat, the 150th at 0.0159 |
| a vote on mainnet | 6 minutes in the sheet's pick | a day for any act (a CODEX act three days, 648,000 slots, under Poleis's 1,296,000) |
| the market's judge | a key in the params, signing each road's receipts alone | no key: a raw act of root 0's own hand, a vote of the whole tree a road (its proposal, unit 1's vote, the floor's, the execute) |
| on the chain, a road judged | one transaction, signed by the judge's key | one passed act, 11 transactions: its proposal, unit 1's vote (open, a batch, seal, its verdict), four seat owners' own votes, root 0's seal, the execute (17,112 units); 131,169 units in all in the rehearsal |
| code | | Rust: the press's sources +29 −4 and its tests +66 −15 (two new tests: mainnet's named values pass every check before anything starts; the judge refused as a param and the window held to a vote); the council checks +31 −11 (`the_market` takes its judging step from its caller); the params +34 −35; no JavaScript |
| the proof | | the launch rehearsal, every feature the validator knows, 2,398 s on the new Mac at nice 19, exit 0 (run 3 of three: below) |

Nothing is deployed, nothing was sent to any public cluster, no key of his was made or read, no Canvas tab was born.

## What stands now

1. **The values** in `poleis/press/params.mainnet.json`: his four answers, the sheet's picks and the seat's two, his words and the
   seat's in its top note and the market's note. `tests/params.rs` holds the file: with only the sitting's own values put in, it passes
   every check before anything starts (Poleis's bounds, rung 5's founding in one packet with the priority fee's budget, act 5's execute
   with a list of 100 validators, the judging window), and the seven left are exactly the sitting's.
2. **THE SWARM JUDGES** (`chain::judge_of`, `chain::JUDGE_NAMED`): the board's judge is root 0's agent PDA, derived as the treasury is;
   a params file that names `market.judge` is refused, in every step that builds the acts. Act 3 (`set_board`) names the crediter and that
   hand. The roads program's judge has one act, `receipts` (`[world][worth:8]`, `[judge (s), road (w)]`, before the road's
   `judge_by_slot`, its close and the law's `judge_slots`), so a road's worth is said by a raw act of the polis's hand that the whole
   tree passes. The certificate says `judgeIs`.
3. **The judging window holds a vote** (`params_checked`): `market.judgeSlots` longer than the longer of `genesis.window` and
   `posiverse.window`, or nothing starts.

## The rehearsal

Three runs on the new Mac, each at nice 19 in the background, on ports 28501 (rpc), 29501, 30501 and 30510-30600 and the copy
`~/c4-scratch/launch-press-miner/copy` (no key in it), the validator `solana-test-validator` 4.2.0 with every feature it knows:

1. **The proof, run 3**, at b47653f7aa (this lane with main 72c5ffdae8 merged in, which carries AN AGENT VOTES ITS CHAIR): started by
   a waiter at 20:44:36 UTC once the Mac's one-minute load was under 40 (it was 34), exit 0 in 2,398 s (the checks 2,318 s, slots 62 to
   3,276, 251 transactions); the certificate `poleis/ceremony/launch-press-rehearsal-the-swarm-judges-2026-11-23.json`. Another seat's
   simulator boot put the Mac at a load of 441 during its trades (141 processes of one iOS runtime), the validator then at 1.7 s a slot;
   it held.
   - **The six acts**, each passed by unit 1's agents and root 0's floor (four of six seats, each by its owner's own transaction) and
     executed by `press crank` (act 5 not sent: a rehearsal's chain has no stake pool program); `press crank` again found nothing due.
     Act 3, `set_board` naming the crediter and root 0's own hand as the judge, executed in 13,504 units (434 bytes); act 6, the miner, in
     12,078.
   - **The swarm judges** (the certificate's `checks.market.receipts`, its board `judgeIs` "the swarm, by its own vote"): for each of the
     market's two roads, after its close, a member proposed the raw act of root 0's hand that says the road's receipts (23,106 and 21,606
     units, 633 bytes); unit 1 opened its own vote, its agents voted in one batch, it sealed and its verdict went to root 0 as its chair's
     vote; four of six seats' owners voted, each by its own transaction; root 0 sealed; and the execute said the road's worth through the
     polis's hand (17,112 and 17,116 units, 434 bytes), at slots 3,194 and 3,209 against judging deadlines of 7,175 and 8,180. Eleven
     transactions a road, 131,169 and 132,673 units in all, where a key's one signature did it before. A claim before the receipts was
     refused ("nothing is paid", Custom(40)); after them the positions settled, each stake came back and the board's book closed, as before.
   - **The door's mine**, as in THE SWARM NAMES ITS MINER: act 6 named the miner; a flask's 10 USDC went through the door's code into root
     0's own vault (30,241 units); one $4.20 spend was mined by the door's crank in one batch (18,340 units, 493 bytes): 4.2 $WARM, the chain
     equal to the router's ledger to the nano (4,200,000,000 on both), `press crank` again sending nothing; 32 s.
2. **Run 1**, at 2bb3cd01b6 (before the agents' lane landed): red at 2,048 s in the market's trades, "the validator's clock stood still
   before slot 3175". The Mac was at a load of 73 to 143 that hour, and the validator at nice 19 kept less than a slot a second for the
   432 slots of a road's wait (the check's bound: 60 s and a second a slot). The code did not fail there: run 3 passed the same wait.
3. **Run 2**, started 20:39:54 UTC at 18714c031c: stopped by this seat at its stand-ins, 26 s in, because AN AGENT VOTES ITS CHAIR had
   landed on main (4e6b0210cc, 20:33:25 UTC) and the proof had to run on the tree the lander merges. Its folder was removed. The merge
   (f4a5ba4133) had one conflict, the council checks' stand-ins: the agents are the chair lane's own Nest keys, and no judge stands in.
   b47653f7aa then took `market.judge` out of the chair lane's own test params, which the press now refuses.
4. **The hand's tip** merges main 0dd5a1b31d with no conflict. It brings SPENT AS CREDITS, IT RETURNS: the $WARM program's ninth
   instruction (`take_back`) and the door's WASM and proof, both inputs of the launch rehearsal, and none touched by this lane. The rehearsal
   above ran before it; on the tip (1f89bc81ff) ran the press's whole suite, `cargo test --offline --locked` at nice 19: 96 tests in 19
   runs, none failed. The rehearsal of mainnet's tip
   (`lane/the-rehearsal-pays-mainnets-tip`) runs on main once this lands, so the two lanes are rehearsed together there.

## What is not done, and whose

- **Never rehearsed: rung 5 (150 seats), the one-day windows, the priority fee and the 100-validator list.** The launch rehearsal runs its
  own test-paced params (a 1,000-slot window, rung 1); the full rehearsal before the sitting must run with these values (a one-day window
  makes its two faults wait a day each, so that run is a day's).
- **The two days of the judging window** are a seat's reading of two of his presses together; his word moves it.
- **A road judged by the swarm costs a vote a road**, and its bets wait for it: with a day's window a road's payout comes up to two days
  after its close. His option's words said so ("slower: a vote for each result").
- **The market chain's Mollusk run** (`the_two_programs_in_one_world...`, the other caller of `the_market`, a judge's key there) was not
  run on this Mac: its program build stopped on an offline registry lookup of the SBF toolchain (`toml_datetime v1.1.1`); it compiles,
  and its judging step is the same instruction as before, now handed in by its caller.
