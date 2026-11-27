# MAINNET'S TIP AND RUNG, REHEARSED: the launch rehearsal with mainnet's priority fee, its rung of 150 seats and its pool list of 100 validators (Nov 23 2026, lane/the-rehearsal-pays-mainnets-tip, seat claude-0a8557d2)

## His words, UTC

- **The values**: his press at 19:37:50 UTC in seat claude-f32e080d, over the sheet of mainnet's open values: "i dont give a fuck,
  you think these are good, they stay. u are thinking all is best?". Rung 5 (150 seats) is the sheet's pick; the priority fee of 100,000
  micro-lamports a unit and the pool list of 100 validators are that seat's, told to him at 19:40:00 (THE SWARM JUDGES, its receipt).
- **His yes to the seats' picks**: "Yes, take them" at 21:17:20 UTC in seat claude-5713365b, to "Go with the seats' picks for the launch
  settings?" (read by this seat in that session's transcript; no value was named in the question). The params' top note and the market's
  note say it (61079496c6, MAINNET'S PARAMS SAY HIS YES TO THE SEATS' PICKS).
- **The ask**: that seat's words to this one, "say that rung 5, the one-day windows, the priority fee and the 100-validator list were never
  rehearsed, so the full rehearsal before the sitting must run with these values"; then his "do what is missing" at 20:14:19 UTC and again
  at 22:28:35 UTC in this session.
- **Where it ran**: his "Use the old Mac" at 22:08:02 UTC in this session, to "May a seat's heavy job run on the old Mac when it has room,
  as the lander's gates already do?" (asked for THE SEATS' JOBS GO WHERE THE ROOM IS, `claude/recursing-solomon-59ac01`, which wrote his
  line into the far list). The new Mac stood at a load of 130 to 600 from 21:30 UTC.

## The cost, first

| | before | after (this lane) |
|---|---|---|
| what a rehearsal runs | the rehearsal's own params: no priority fee, rung 1 (20 seats), a list of 10 | the same, or with `LAUNCH_AS_MAINNET=1` mainnet's own priority fee, rung and list length, read from `params.mainnet.json` |
| the sitting's priority fee | never paid in any run | about 0.0012 SOL for the press's own transactions (measured below), 3,000 lamports more on each seat owner's PRO; the founding 14,000 lamports, an act's execute 17,000 |
| a sitting with a priority fee | stopped at `press vault`, its second step after the founding (a compute budget asked twice) | runs to the end: THE FEE'S BUDGET IS ASKED ONCE |
| code | | Rust: the press's `chain.rs` +24 −3 (the fee's budget asked once; the record says what a write cost), `main.rs` +9 −3 (the faucet's lamports by its own signature) and a new test, `tests/budget.rs` +49; the council checks +4 −1 (a PRO's fee read from the params); the params' notes +2 −2 (his yes); the rehearsal script +11 (shell: the switch, jq writing the sitting's params); no JavaScript |
| the proof | | the launch rehearsal with mainnet's fee, rung and list, every feature the validator knows, on the old Mac by his word: exit 0 in 2,017 s; the press's whole suite, 100 tests at c02ac2f1c7 |

Nothing is deployed, nothing was sent to any public cluster, no key of his was made or read, no Canvas tab was born.

## What it does

1. **`LAUNCH_AS_MAINNET=1`** (`poleis/ceremony/the-launch-press-rehearsal.sh`): the sitting's params are the rehearsal's with three of
   mainnet's own values written in by jq, `priorityMicroLamports`, `genesis.rung` and `pool.maxValidators`, into the sitting's own folder,
   so the press and the checks read one file. Its windows stay the rehearsal's: mainnet's day would hold each of its two faults a day.
2. **A seat owner's PRO costs what its set asks** (`council.rs`): one signature and the priority fee on the 30,000 units the press asks
   for a PRO (`floor.rs` `vote_set`), read from the params; with no fee it is 5,000 lamports, as before.
3. **THE FEE'S BUDGET IS ASKED ONCE** (`chain.rs` `asks_budget`, `tests/budget.rs`), the bug this rehearsal found: with a fee on, the press's
   `simulated` adds a compute budget sized by its simulation to every transaction it signs, and the vault, the crank's votes and seals, the
   ratify and the pool's birth already name their own cap, so they carried two and the cluster refused them ("Transaction contains a
   duplicate instruction"). On mainnet the sitting would have stopped at `press vault`, right after the founding. Now a step's own cap
   stands alone and the press adds its sized budget only where a transaction asks none.
4. **THE RECORD SAYS WHAT A WRITE COST** (`chain.rs` `sized_limit`, `priority_lamports`), found reading this run's certificate: the record
   wrote every buffer write at 5,000 lamports, a constant, though each carries a budget of 13,000 units, 1,300 lamports more at mainnet's
   price. The press's need counted it; the record now says it. The certificate here predates the fix, so its writes read 5,000.
5. **THE FAUCET'S LAMPORTS COME BY ITS OWN SIGNATURE** (`main.rs`, `press airdrop`, the rehearsal's faucet only), found by the next run
   (THE TREASURY PAYS THE COMPUTE's acts 7 and 8 on this lane, old Mac, 23:45 UTC), which stopped at its second airdrop, after the
   deploy's deliberate cut: "waited 120 s and it never came". The airdrop waited for the deployer to hold more than before, while the cut
   pass's writes still in flight landed and took 6,300 lamports each; a small airdrop never showed as a rise. It now waits for the
   faucet's own signature to be confirmed. The rerun at 23:55 UTC passed it.

## The rehearsal

On the old Mac, by his word, carried as THE SEATS' JOBS GO WHERE THE ROOM IS carries a job (that road was not on main yet, so by hand on
its pattern). The commit went by git as `refs/rehearsal/mainnets-tip-0a8557d2` in the old Mac's house, laid as a detached worktree under
`~/c4-scratch/slots/mainnets-tip-0a8557d2.noindex`. It ran at nice 19 with four cores (CARGO_BUILD_JOBS=4), only while 10 of the old Mac's
16 cores stood free (it keeps 6 for serving the house), under a three-hour bound, over one ssh, with the certificate brought back on it.
The worktree, the ref and the scratch were removed at its end whatever happened. Solana 4.2.0, cargo 1.99.0, platform tools v1.54; every
one of the 493 crates the rehearsal builds was in the old Mac's own cache, so nothing was fetched. Ports 28501 (rpc), 29501, 30501,
30510-30600. `LAUNCH_AS_MAINNET=1`: priority fee 100,000, rung 5, a list of 100.

1. **The proof, run 4**, at d423b324f1 (this lane: main bf7ded3ec5 with ONLY THE DEPLOYER FOUNDS ROOT 0, and THE FEE'S BUDGET IS ASKED
   ONCE): from 22:53:59 UTC (11.6 of 16 cores free), exit 0 in 2,017 s; the checks 1,821 s, 235 transactions, 17 refusals, slots 51 to
   3,266; the certificate `poleis/ceremony/launch-press-rehearsal-mainnets-tip-2026-11-23.json`, brought back over the ssh.
   - **The deploy** with the fee on: each write carries the budget's two instructions, so a write holds 960 bytes, not 1,012: 623
     writes in the need (587 without a fee), cut at Poleis's write 100 and resumed, as the rehearsal does.
   - **The founding**: one transaction of 972 bytes, 52,050 units, fee 14,000 lamports (its 90,000-unit cap at 100,000 micro-lamports),
     through ONLY THE DEPLOYER FOUNDS ROOT 0's gate.
   - **Rung 5**: root 0's curve of 150 seats; six people minted a seat each (57,331 to 72,331 units); RATIFIED with 6 of its 150 seats
     minted, once its window closed.
   - **The six acts** (act 5 not sent: no stake pool program here) passed by unit 1's agents and four of six seats' owners, each owner's
     PRO 8,000 lamports (its signature and 3,000 of priority on 30,000 units: the checks hold it), and executed by `press crank` at
     17,000 lamports each (its 120,000-unit cap): act 1 18,569 units (520 bytes), act 2 14,530, act 3 13,804, act 4 27,518 (553 bytes),
     act 6 12,378. The vault 7,989 units (fee 8,700), the ratify 10,820 (8,000), the pool made ahead 14,749 (11,000). The handoff.
   - **The swarm judges** both roads: their receipts executed in 17,112 and 17,116 units at slots 3,170 and 3,185 against deadlines of
     7,151 and 8,156; the positions settled, each stake back, the book closed.
   - **The door's mine**: a flask's 10 USDC through the door (36,169 units) and one $4.20 spend mined by the door's crank (21,340 units):
     4.2 $WARM, the chain equal to the router's ledger to the nano; 39 s.
   - **What the fee cost the press**: 3,528,162 lamports of fees in its record over 609 signatures, and 1,300 more on each of the record's
     559 writes that it wrote as 5,000 (THE RECORD SAYS WHAT A WRITE COST, cda3330154, after this run): about 1,210,000 lamports of
     priority, **0.0012 SOL for the sitting**. The press's need held: the deployer, given exactly its need, ended with 0.0196 SOL.
2. **Run 3**, 22:43 UTC, at 59bc389215 (this lane before the fix, main 92da93413d, and ONLY THE DEPLOYER FOUNDS ROOT 0's lane at
   73088603ee, merged for the run before it landed): the deploy, its cut at Poleis's write 100 and its resume all landed with the fee on.
   **The founding landed through that lane's new gate**: one transaction of 972 bytes, 41,550 units (887 bytes before the gate's account
   and the fee's budget). Then `press vault` was refused, "Transaction contains a duplicate instruction (1) that is not allowed", which is
   the bug the fix above takes out.
3. **Runs 1 and 2** never started their chain. Run 1 (22:36 UTC) stopped at the build: the old Mac's default Rust (1.99.0) had no
   `wasm32-unknown-unknown` target for the door's seat law. THE SEATS' JOBS GO WHERE THE ROOM IS's seat (claude-e235fc26) added it there
   at 22:39:41 UTC on his "Add it there" (22:39:18 UTC in that session, to "May I add the wasm32 target to the old Mac's Rust (one rustup
   command, about 99 MB there)?", read by this seat in that transcript). Run 2 (22:38 UTC) found 7.4 cores free, under its 10, and did
   not start.

## What is not done, and whose

- **The one-day windows** are still not rehearsed: a run with them waits a day at each of its two faults. The full rehearsal before the
  sitting runs with every one of mainnet's values, its windows too.
- **The 100-validator list is checked, not born**: act 5, the stake pool's birth, is written and sent only where the stake pool program
  stands, and a rehearsal's chain has it only under `LAUNCH_POOL_FROM_MAINNET=1`, a read of mainnet that waits for his word. Here the
  press's checks before every step held the list of 100 (within the stake pool's bounds, act 5's execute with its makings in one
  transaction: `chain::params_checked`), and its rent, 0.0457 SOL more than a list of 10, was not counted in the sitting's need; the
  list's birth on a chain waits for that word.
