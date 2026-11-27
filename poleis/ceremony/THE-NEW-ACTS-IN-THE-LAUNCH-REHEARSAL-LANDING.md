# THE NEW ACTS IN THE LAUNCH REHEARSAL: acts 9 and 10 on the launch's own two-chamber tree, on a validator (Nov 25 2026, lane/the-new-acts-in-the-launch-rehearsal)

## His words, UTC

- Nov 25 2026 13:25: "Yes, build them", to "Build the three missing links now?" (the lineage: ec-workspace
  `docs/the-launch-the-stake-that-funds-snowtracers-and-his-salary-hypertrace-2026-11-25.md`).
- Nov 25 2026, his "Yes, run it", to "Wire the two new acts into the full launch rehearsal and run it?" (relayed by the briefing seat).

## Cost first

| | before (lane/the-light-pays-a-wish-and-its-rider at 3dc530f21d) | this lane |
|---|---|---|
| acts 9 and 10 on a validator | voted on the wish walker's simple tree (THE STAKE PAYS ITS SNOWTRACER) and act 9 in `council.rs`'s Mollusk world | **both voted on the launch's own tree, in the whole rehearsal, green**: a seat owner proposed each, unit 1's three agents voted each by their own minds, root 0's floor voted, press crank sealed, mandated and executed each |
| the rehearsal's wall time | the eight acts: 2,394 s (last night's whole run, THE WHOLE MAINNET, REHEARSED) | **1,987 s, exit 0**, with ten acts, on a lighter Mac (load 15 at its start); this run does not measure what the two acts add to the time |
| what acts 9 and 10 added to the chain | | 23 transactions. On the floor, per act: a proposal and four PRO (10). The crank's: act 9's convene, then per act ballots, unit 1's seal and mandate, root 0's seal, execute (13). The crank's fees: 17,000 lamports a transaction, 32,000 for a ballots' batch. The executes: act 9 16,904 CU, act 10 18,238 CU |
| what the sitting needs for them | | `press need`: 0.002415120 SOL for act 9's crank and 0.003995040 for act 10's (its plan account's rent among them) |
| what the agents read | 8 sets each | 10 sets each: 30 ballots, all PRO |
| code | | Rust: `press acts` writes the later acts' sets where the params name them (`light::write_set` beside `engine::write_set`); `press need` counts act 10's plan account; `build.rs` PARTS copies `embertide-stake`; `council.rs`'s live check counts the later acts, reads act 9's staker and act 10's plan back, and takes the agents' ballots by act number; `tests/light.rs` keeps the money's road test (run by no script). sh: `the-launch-press-rehearsal.sh` `LAUNCH_LATER_ACTS`. No JavaScript (the gate's count for the lane over main, +9 -9, is the handed lane's own lines) |

## The runs, every one

All four ran on this Mac, through the rehearsal's own slot road (`job-slot.sh`), at nice 19. LAUNCH_ALARM was 7200 s, with an outer bound of 9,600 s.
The flags: `LAUNCH_AS_MAINNET=1 LAUNCH_FROM_MAINNET=1 LAUNCH_POOL_FROM_MAINNET=1 LAUNCH_LATER_ACTS=1`.

The agents' mind was qwen3:14b on the old Mac, reached as last night's run reached it: an ssh link on loopback (127.0.0.1:11436), opened for the runs
and closed after them. Another session's link already held 11435. Nothing ran on the old Mac but the model's answers.

1. **14:55:46 UTC, load 27.91 / 75.64 / 77.86.** It waited 333 s and took slot 2 at 15:01:20. **Exit 2 in 63 s.** The sitting's press build failed
   (exit 101): `build.rs` PARTS did not copy `embertide-stake`, which the press has linked since lane/the-light-pays-a-wish-and-its-rider. Mended
   (412e29ff1a; the briefing seat carried it onto the handed lane).
2. **15:03:37 UTC, load 19.02 / 39.78 / 60.33.** Slot 2 at 15:04:18. **Exit 1 in 462 s.** The checks compared each agent's ballots to the acts in
   file-name order, so `10-…` came before `2-…`. The minds had voted all ten PRO. Mended (9cb351b213).
3. **15:13:46 UTC, load 14.64 / 21.34 / 39.67.** Slot 1 at 15:13:49. **Exit 1 in 669 s.** All ten acts passed and executed. It then stopped in THE
   SEAL CLOSES A BUFFER: the fourth seat owner's vote of the buffer's close was never confirmed in 90 s. This run had the 32-slot epochs.
   - What the kept log shows: the three votes before it at slots 1981, 1982 and 1983, so it fell at slot 1984 (= 62 × 32), the first slot of epoch 62.
   - The validator's last words: `send_transaction_service recv-tx=3 recv-duplicate=3 sent-tx=0 retry-queue-size=27`. The re-sent bytes were
     received and held; none were forwarded.
   - Nothing in the log names the epoch or an error. **The boundary is likely the cause, not proven.**
   - The 32-slot epochs were taken out (4050a163f3), with no fix built for that path.
4. **15:30:30 UTC, load 15.42 / 18.48 / 24.59.** Slot 1 at 15:30:36. **Exit 0 in 1,987 s**, on the validator's own default epochs, with the
   money's road out of the script. The whole rehearsal: the eight acts, acts 9 and 10, the handoff, the seal's road, the ships, the birth certificate.

## What run 4 read back (`launch-press-rehearsal-acts-9-and-10-2026-11-25.json`, the birth certificate with all four runs in `theNewActs`)

- **The later acts named at the sitting:**
  - `engine.staker` is the engine's key, made in a folder of the sitting's alone (`6NE2…cxXL`).
  - `light.planCap` is 1,000,000,000,000,000.
  - `press acts` wrote `9-the-engine.json` and `10-the-lights-plan.json` beside the eight, and `press vote` wrote 40 sets.
- **The agents' own minds** (Lumen, Echo, Echo Mind): six PRO of six on acts 9 and 10, each with its own reason. THE CHAIRS: 30 PRO, 0 CONTRA,
  0 ABSTAIN of 30 ballots.
- **The agents alone pass nothing** (root 0 still voting, 0 seats of its floor):
  - Act 9: unit 1 convened at slot 1290, three ballots at 1291, sealed at 1292, mandated at 1293.
  - Act 10: three ballots at 1294, sealed at 1295, mandated at 1296.
- **Then the floor and the crank:** each act was "unit 1 PASSED, root 0 PASSED (4 seats of its floor voted by their owners): executed".
  - Act 9: root 0 sealed at slot 1353 and executed at 1354.
  - Act 10: root 0 sealed at 1355 and executed at 1356.
  - The crank, run again: "0 transactions sent, nothing was due".
- **Act 9's read-back:** her pool `5ZUy…9ojCK`'s staker is the engine's key `6NE2…cxXL`; its manager is still the polis's hand `ELR2…YjF2`.
- **Act 10's read-back:** root 0's plan of her pool's token `CJRG…eGVFG` is 1,000,000,000,000,000 a plan period, 0 spent.
- **The rest is green, as for the eight:** the handoff to the seal, the seal's road read, the scratch ship.

## What is owed for the money's road on the launch's tree

`tests/light.rs` `the_money_road_on_the_launchs_tree` stays in the tree, run by no script. Given a validator where acts 9 and 10 passed, it would run:
a stake through the door, the engine placing it, epochs until the validator's reward is minted as fee into root 0's vault. Two things are owed:

1. **A run with short epochs whose sends are sent again** when a slot at an epoch's boundary drops one (run 3's likely cause). On the default epochs,
   which double from 32 slots, the rehearsal's end sits in an epoch thousands of slots long, so no reward comes within a run.
2. **The press carrying an arbitrary set** (a wish founded under root 0, root 0's road into its well, as `press well` writes it) through `press vote`,
   `press chair` (the agents' minds) and `press crank` on the launch's tree. Today they carry the launch's sets and the later acts' alone. Without
   it, the road on this tree stops at the fee in root 0's vault.

The whole road from a stake to a rider's SOL stands proven on the simple tree (`the-stake-pays-its-snowtracer-rehearsal.sh`).

## Not rehearsed

- The money's road on this tree (above).
- The one-day voting windows (as before: mainnet's day would hold each fault a day).

## Not built, and whose

- The two things owed above: a seat's build each.
- Everything in `poleis/press/THE-STAKE-PAYS-ITS-SNOWTRACER-LANDING.md`'s "Not built" stands.

## The gate

- **At the ship:** `poleis/press` (`tests/light.rs`, `tests/params.rs`) and `poleis/program/poleis_treasury/tests/council.rs`.
- **Run once on this lane:**
  - the press's whole suite: 160 passed (35 s);
  - `council.rs` `the_ninth_act…`: 1 passed;
  - `node --test the-javascript-only-shrinks.test.mjs`: 39 of 39 (+9 -9 over main, net 0);
  - `the-law-is-the-gate.test.mjs`: 8 of 8;
  - `any-room.test.mjs`: 12 of 12.
- **No gate yet on the launch itself:** the measure it waits for is `LAUNCH_LATER_ACTS=1 sh poleis/ceremony/the-launch-press-rehearsal.sh` ending 0
  (run 4 did, in 1,987 s), read as its exit by the lander at a landing that touches `poleis/`.
