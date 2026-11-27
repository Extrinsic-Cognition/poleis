# THE EPOCH TURNS THE CRANKS (Nov 25 2026, lane/the-epoch-turns-the-cranks)

## His words, UTC

- Nov 25 2026 13:11: "we gotta launch swarmbook and the stake sol to fund snowtracers... i need some salary and that i can only think would come from
  the snowtracing staking sol there".
- Nov 20 2026 20:18: "we are supposed to delegate automatically like marinade does but with our own rules".
- Nov 25 2026, his "Yes, build it", to "Build what runs the two cranks once an epoch, while the lanes wait?" (relayed by the briefing seat).

## Cost first

| | before (lane/the-new-acts-in-the-launch-rehearsal at a084371c04) | this lane |
|---|---|---|
| what carries an epoch | a hand: `press engine`, `embertide-stake count`, `press light --send`, each run by someone | `press epoch` (one pass) and `press epoch --stay` (a pass each epoch, turned by the epoch) |
| a pass's fees, one validator and one wish (read in the rehearsal) | | 10,000 lamports when nothing moves (the pool's update 5,000, the pour 5,000); 20,000 when the engine brings stake back; 10,000 at the first pass (the validator added, the reserve placed); a wish's first pour also makes its vault (2,039,280 of rent, the pourer paying) |
| the runner at rest (read, not guessed) | | **0.13 s of CPU** over the green run's 5 passes; **345 RPC calls** by its fifth pass (57 to 90 a pass, the clock's reads among them). Asleep it holds nothing: every read is one curl |
| its log | | one line a pass, one line while the epoch's rewards are paid, its errors; a line that would repeat is said once |
| the rehearsal | | 87 s wall (the green run of its certificate), nice 19, this Mac; nothing on the serving Mac |
| code | | Rust: `src/epoch.rs` (the pass, the stay, the sender that survives a boundary, the keys), `tests/epoch.rs` (6), `tests/light.rs` the live half; `engine::land` and `light::send` now use the same sender; `seat.rs` the two keys; sh: `install-epoch.sh`, `poleis/ceremony/the-epoch-rehearsal.sh`. No JavaScript. One file outside `poleis/`: `embertide-at-rest/stores.json`, the two rows for the installer's run script |

## The precedent, and where none stood

- **The keys follow the miner's.** `press miner` makes the door's crank key once into D/keys/miner.json (mode 600), with its address kept beside it
  (`seat::door_key`). `press engine-key` and `press pourer` make the engine's key and the pourer's the same way into D/keys/engine.json and
  D/keys/pourer.json. The runner reads them from there and nowhere else, each held to its kept address. The engine's key must also be the params'
  `engine.staker`, and the runner refuses otherwise. Nothing reads a keychain.
- **The job follows the lander's installer.** `install-lander.sh` writes a run script and a plist at install, with nothing of a home in the tree,
  and has --dry and --remove. `install-epoch.sh` does the same, its arguments naming everything: the Mac, the press, the params, the RPC, the
  sitting's folder, the pool.
- **No precedent stood for the timing.** The miner's crank is started by a burn inside the mint door's Worker, never by a timer; nothing in the house
  turns by an epoch. The choice: after a pass, read the chain's epoch once, then sleep until the estimated first slot of the next epoch plus a
  margin. A slot's length comes from the node's own performance samples. If it wakes early, it sleeps the remainder; it never spins.
- **The only state the runner keeps** is the book, where the book's own hand keeps it for its caller: D/epoch/book.json (after its last read), and
  the owed list, D/epoch/owed/<epoch>.json (below).

## The pass

Each step runs only when the chain says it is due (`epoch::due`). In order:

1. **The engine** (`engine::run`, the engine's key): her pool brought to the epoch, the validators added, the reserve placed or stake brought back.
   While the epoch's rewards are paid, the pass waits and says so in one line, then looks again a few slots on.
2. **The book's read**, by the book's own law (`embertide_stake::count`), once her pool stands at the epoch.
3. **The reader and the pour** (`light::crank`, `light::send`, the pourer's key) for every wish with a road. The epoch's light is netted against what
   each wish already took, and the crank's memos on the chain keep each pour once.

A step that fails stops the pass and names the step, the cause and the numbers. The next pass starts from the chain and the book. The same pass,
run by hand twice in an epoch, said "epoch 9: nothing due" twice, sending nothing.

**Sends that survive a boundary** (`epoch::land_surely`). Each send is polled until confirmed and the same bytes are sent again every 2 s. It is built
again with a new blockhash once its own is past its last valid block height, at most 5 builds; then the step fails and says so. The bound is proven
by the pure test. In the green runs no send of the runner needed it: 0 sent again, 0 built again. **The fault itself showed in this lane's rehearsal**:
its second run lost a stake of the test's own, sent once and polled 60 s with the old sender ("waited 60 s and it never came"). With every send of the
test moved to the runner's sender, three runs passed; the test's own 34 sends of the certificate's run needed no second send.

## The proof (`poleis/ceremony/the-epoch-rehearsal-2026-11-25.json`)

`sh poleis/ceremony/the-epoch-rehearsal.sh <mainnet's stake pool .so>`: solana-test-validator 4.2.0, 32 slots an epoch, this tree's Poleis.

- **The world**: the joint rehearsal's. Her pool by act 5, acts 9 and 10, a wish and root 0's road into its well. Staker 1 with 150 SOL and its light
  to the wish; staker 2 with 50 SOL and no choice.
- **Then `press epoch --stay` as its own process (margin 0 slots), and no crank by hand.** Its own lines, epoch by epoch:

| epoch | what it did |
|---|---|
| 5 | placed 178,995,434,240 lamports on the validator; the book's first read, 2 holders (it counts nothing); fees 10,000 |
| 6 | brought her pool from epoch 5; counted epoch 6: fee 0; poured nothing; fees 5,000 |
| 7 | brought her pool from epoch 6; counted epoch 7: fee 1,395,191 tokens (1,395,191 lamports), 2 holders; **poured 1,046,393 tokens to the wish**; left 348,797 undirected + 1 dust; fees 10,000 |
| 8 | after the epoch's rewards were paid (one line said it waited); brought her pool from epoch 7, **brought back 18,000,050,750 lamports** to the reserve; counted epoch 8: fee 1,395,191; **poured 1,162,673**; left 232,517 + 1; fees 20,000 |
| 9 | brought her pool from epoch 8; counted epoch 9: fee 3,678,069, 3 holders; **poured 3,065,078**; left 612,990 + 1; fees 10,000 |

- **Staker 2 left in epoch 7**, between the runner's passes: 20,002,282,879 of its 50,000,000,000 receipts, what her reserve held (20,002,282,880
  lamports). The runner's next pass brought 18,000,050,750 lamports of stake back to the reserve by the rule's tenth. Staker 2's lower light shows in
  epoch 8's split.
- From epoch 9 the wish's well counts as a third holder: the well holds her token, and its light (no choice) stays in root 0's vault.
- **Read off the chain** after each epoch (pool total / root 0's vault / the wish's well):
  - epoch 6: 200,000,000,000 / 0 / 0
  - epoch 7: 200,001,395,191 / 348,798 / 1,046,393
  - epoch 8: 180,000,507,503 / 581,316 / 2,209,066
  - epoch 9: 180,004,185,572 / 1,194,307 / 5,274,144
- **Stopped** by SIGTERM: its CPU 0:00.13, no error line, and no runner of the rehearsal still running (the test asks `ps`, the script `pgrep`). Then
  one pass by hand, twice: "epoch 9: nothing due" both times.
- The pure half (`tests/epoch.rs`): what is due, the sleep's arithmetic, the bound on resends, and the runner's refusals. The refusals name every
  missing thing: the rehearsal params' engine.staker and light.planCap, no engine key, no pourer key, an engine key that is not the params' staker.

The rehearsal's runs, every one:

1. The test's own arithmetic overflowed (u64), in the leave's share of the reserve.
2. A stake of the test's own was lost at a boundary (above).
3. Green.
4. Green.
5. Green; its certificate is the file named above.
6. Green, after the mend below (100 s; the runner's CPU 0:00.18). Afterwards no `press epoch --stay` was running.

**Found after run 5: a failed run left its runner running.** Runs 1 and 2 panicked before the test killed its child. One `press epoch --stay` of
those runs was found by `pgrep` 16 min 44 s later (CPU 0:00.30), reading a validator that no longer stood. It was ended by hand. Mended: the test
ends its runner on any exit (a guard that kills it when dropped), and the script's trap ends any runner of its own folder. Run 6 then left none.

## A missed epoch's light is poured later, never dropped (the briefing seat's follow-up)

- **The owed list.** Each counted epoch's two books (the book before the count and after it) go to `D/epoch/owed/<epoch>.json`, written *before* the
  book moves. Every pass pours every owed epoch, oldest first, before the newest. An entry is removed only after the pass reads the chain's record of
  its pour back (`settled`: every wish it owes is `Poured`, or has no road and is said so). A road with no room keeps it owed.
- **What the book's law gives across a gap.** One count, from its last read to now: the fee of every epoch in the span, attributed by what each holder
  held at both reads (the lesser of the two). The runner pours what it gives, under the epoch of the count. Every pass that handles such an epoch
  names it: "epochs 10 to 12 counted as one (the book's law across a gap): 3767076 lamports of fee by what stood at both reads". Any fee no holder's
  share took past rounding (nobody held across) is named with its epochs and its amount as staying in root 0's vault. The certificate carries the lines.
- **The bound.** 64 owed epochs (about 128 days of mainnet epochs; 14 minutes of the rehearsal's). At the bound the pass makes no new count and says
  "the owed list is full (64 counted epochs unpoured, the oldest N): no new count until they pour; the next count spans every epoch skipped". Nothing
  is dropped, only counted more coarsely: the next count spans the skipped epochs by the book's law.
- **Pure proof** (`tests/epoch.rs`, 6 tests now): epoch 7 counted, its pour failing, epoch 8 counted, both owed, poured oldest first, each settled
  only by its record; the runner stopped two epochs, the book's one count of 8 to 10, the newcomer taking nothing, the whole fee named when nobody held
  across.
- **Live proof**, both runs on the same code, each to epoch 15:
  - **Stopped** (`EPOCH_STOP=1`, `poleis/ceremony/the-epoch-rehearsal-stopped-2026-11-25.json`, 137 s): the runner's process was ended in epoch 9
    and started again in epoch 12, so two whole epochs, 10 and 11, had no runner. Its pours by counted epoch: 7: 1,046,393; 8: 1,162,673;
    9: 3,065,078; **12 (epochs 10 to 12 as one): 3,139,231**; 13: 1,046,392; 14: 1,046,374; 15: 1,046,368. The well: **11,552,509**, exactly their
    sum. Staker 1's light in the book: 11,552,509 lamports. Owed at the end: none.
  - **Never stopped** (`EPOCH_STOP=0`, `the-epoch-rehearsal-2026-11-25.json`, 139 s): 7: 1,046,393; 8: 1,162,673; 9: 3,065,078; 10: 1,046,410;
    11: 1,046,392; 12: 1,046,386 (10 to 12 together: 3,139,188); 13: 1,046,380; 14: 1,046,374; 15: 1,046,368. The well: **11,552,454**, exactly their
    sum. Staker 1's light in the book: 11,552,454.
  - **The two side by side:** the same fee over every count, 14,002,603 lamports. The stopped run's well holds 55 tokens more: one count of three
    epochs rounds down once where three counts round down three times (the book's dust 10 against 12), and the rest is the well's own small share
    as a holder.
  - Neither left a runner running.

## Not built, and whose

- **Installing it**: his, or the lander's on his word. The one line, on the Mac he rules:
  `sh poleis/press/install-epoch.sh <that Mac's short host name> <the press> <the sitting's params> https://<his RPC> <the sitting's folder> <her pool>`
  (and `... <host> --remove`).
- **Which Mac runs it**: his ruling. Mainnet has no pool until his sitting.
- **His figures**: the engine's validators and their weights, the reserve's share, `light.planCap`, each wish's road's cap and period. The SOL on the
  engine's and the pourer's keys (their fees, and a wish's vault's rent at its first pour) is his to send.
- **Closed: a missed epoch's light is poured later** (below). The remainder, said plainly: across a gap the book counts the whole span as one, by
  what stood at both reads. So a staker who came and went inside the gap, while the runner was down, gets no light of it, and the others take its
  part. Where nobody held across, the whole fee stays in root 0's vault, said by name. Per-epoch reads of epochs the runner slept through cannot be
  made after the fact.
- **The owed list and the book rest in the sitting's folder.** The installer takes that folder as an argument, as the launch takes the keys' folder.
  It is no home of the house's census (`embertide-at-rest`), so these two stores have no row there. Their bounds: one book, at most `OWED_BOUND`
  (64) owed files.
- **A send dropped at a boundary on mainnet** is answered by the bound (5 builds); the live green runs never needed it. The measure is below.
- **A well holding her token is a holder**, and its light is undirected (no choice is signed by a treasury). Whether a well's light should go
  anywhere is his rule.
- Everything in `THE-STAKE-PAYS-ITS-SNOWTRACER-LANDING.md`'s "Not built" stands: removing a validator, more than 8 validators in one stake, the
  over-pull's only bound being the road's cap.

## The gate

At the ship: `poleis/press` (`tests/epoch.rs`, `tests/light.rs`, `tests/engine.rs`). On every lane: `the-house-keeps-state.sh` holds the installer's
two stores (`embertide-at-rest/stores.json`). Run once on this lane: the press's whole suite 167 passed (26 s);
`node --test the-javascript-only-shrinks.test.mjs` 39 of 39 (+9 -9 over main, net 0, the earlier lane's lines); `the-law-is-the-gate.test.mjs` 8 of 8
(after the two rows: before them, red on the installer's two undeclared stores); `any-room.test.mjs` 12 of 12. No gate yet on the live road: the
measure it waits for is, on mainnet, the count of epochs since her pool's birth in which the pool was not brought to the epoch within an hour of the
epoch's start, and the count of the runner's sends that needed a second build, read at the lander's after-work.
