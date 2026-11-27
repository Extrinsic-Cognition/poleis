# THE AVERAGE FOLLOWS THE PRICE THAT STOOD: a price counts for the time it stood, never for the instant of a tick (Nov 20 2026, lane/the-average-follows-the-price-that-stood, claude-81203e94)

## His words, UTC

- **Nov 11 00:55:** "i think we should get into ai futarchy with formal verification for governance".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market". The offer it answers, in
  this seat's words: "what a bet on a snowtracer is on a chain, how the house earns on the trading, and how the market's verdict reaches the
  contract's seal".
- **Nov 20 14:26:19:** "why u asking build or change? u think u can design better?"

No word of his asked for this lane by name. It closes a hole in the law that was on main, found at about 17:00 UTC while this seat read the
market's program line by line before landing it. He was told the same hour, and it is in the design's doc (ec-workspace
`docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, the section "Nov 20 2026 14:03 UTC", "What reading the program found").

## Cost first

| | before (main 6fd090ec7a) | after (this lane) |
|---|---|---|
| the law's core, `src/law.rs` | 319 lines | 357 lines, still no allocation and no std |
| a branch (a pool and its average) | six numbers | seven: `acc`, a u128, the step's sum of the price that stood. 16 bytes more in an account, twice a road |
| a boundary of the grid | a read of the spot, a step | the spot read once a carry, a sum, one division more (the sum over the step) |
| `Branch::advance` | `advance(to, through, t)` | `advance(to, t)`: one parameter fewer, and the rule "the boundary at now waits" is gone from the law and from its callers |
| the proofs | ten, `unsat` | eleven, `unsat` (`proofs/stood.smt2` is new; `step.smt2` says what it is now used for) |
| `cargo test --offline --locked` with `FUTARCHY_PROOFS=require` (the new Mac) | 39 passed, 1 ignored | 48 passed, 1 ignored, no warning; every assertion that stood still holds |
| the core's own tests with no feature | 2, and the proofs | 10 (the eight of `tests/stood.rs` run on the core alone), and the proofs |
| the ignored test on a scratch Nest | 1 of 1, 51 s | 1 of 1, 51 s; the same three verdicts |
| the lock file | | unchanged: no new dependency |
| compute units on a chain | | not measured here: the program's own lane measures a boundary again |
| JavaScript | | none added |

The code's diff: 9 files, 461 lines in, 61 out, all inside `embertide-futarchy/`; this receipt is the tenth. Nothing was run on the serving Mac.

## What was so

A road passes when PASS's average beats FAIL's by a threshold. Each pool's average is of an observation that moves only at the boundaries of a
grid (THE GRID, Nov 11). At a boundary the observation stepped toward **the spot of that one instant**, by at most `max_step_bp` of itself.

So the price of one instant counted for a whole step. Measured on the law as it stood, on its own test terms (three days, a step a minute,
100 bp, a threshold of 300 bp, pools of 1000 and 1000, a purse of 1000):

- A trader buys PASS with 900 at each of the 4,319 boundaries and sells what it bought one second later: 8,638 trades, in the pool one second
  in sixty.
- The observation passed three times the opening price at boundary 111 and stood at 3.61 at the close. PASS's average was 3.5626 against the
  1.03 it needed: **the road passed.**
- The trader ended with no claim, so the settle on the receipts could not touch it, and it had lost 5,753 micro $WARM (0.006 $WARM) to the
  rounding.

Three receipts of this crate and this seat's reports to him said "whoever moves a price pays". That held for a trader who holds. This one never
holds. The model's pumper bought and held, which is why the model did not see it. The program's builder reported that a boundary "can be pushed
for almost nothing"; the reading of what that meant is this seat's.

## What stands now

**A price counts for the time it stood, and never further from the observation than one step.** Between two boundaries a branch sums the spot,
held inside the lag band around the observation, times the milliseconds it stood. At a boundary the observation becomes that sum over the step.
The grid, the lag limit, the average of the observation and the decision are as they were.

On this code, the same 8,638 trades: the observation rises at most 175 parts in a million over the opening price, PASS's average is 1.000172,
and **the road fails**.

- **Held, then averaged; never averaged, then held.** A raw spot averaged first would still let a spike large enough move the mean a full step
  however briefly it stood (a spot of 100 for one second in sixty lifts a mean of 1 by 165 %). Held first, a spike of any size counts as one
  band for the time it stood. On this code a spot of 4 and a spot of 400, each standing the same second, give the same observation to the unit.
- **Under half the time, one band at most.** A price pushed far away for a share `f` of every step, the quiet price standing the rest, settles
  the observation at a fixed point: for one second in sixty at 100 bp, 0.017 % over the quiet price, for ever; for a quarter of every step,
  0.33 %.
- **Over half the time it walks, slowly, and only while held.** At three quarters of every step the observation moves half a band a step. A
  price held the whole step moves it one band a step, exactly as before.
- **A quiet pool is the law as it was, to the byte.** While the spot does not change, the sum is the held spot times the whole step and the
  division is exact.
- **A trade counts from its own stamp.** A boundary is due the instant the clock reaches it: the step that ended there was whole. A trade
  stamped on a boundary counts for the step it opens. So `advance` lost `through`, `carry` no longer leaves "the boundary at now" for later, and
  a program on a chain no longer has a trade that can carry one boundary fewer than a carry can.
- **A trade undone at the same stamp counted for nothing**: a buy and its sale at one instant leave the sum, the observation and the average
  as if neither happened.

## What the law cannot do alone

The lane's builder probed what the new rule does not close, and it is true of any average: **a road nobody else trades is decided by whoever
trades it, for nothing.**

- It may lift a price at the opening, hold it through the window and sell it back an hour before the close. No trade stood between, so the
  pool gives its $WARM back. The road passes; the trader holds no claim; it settles at the rounding (at most 2 micro) whatever the receipts say.
- Or, on a road both ways, it may sell its whole FAIL claim and hold. FAIL's average falls, PASS beats it, the road passes, and the FAIL world
  did not happen: by the law's own word that trade counts for nothing. It settles at exactly 0.

No rule of an average can close that. What makes a wrong price cost is **another trader** who takes the other side while it stands, and the
receipts on whatever is still held at the close in the world that happened. On this code, with a second purse of the same size that buys FAIL
back **an hour later**: the road fails, FAIL ends dearer than it opened, the FAIL world happens, and at receipts of 0.9 the pusher, who sold
500 claims for 333.33 $WARM, settles at −116.67, the one who answered at +78.57 and the pool at +38.10.

What this lane's rule gives is that hour. Under the tick's rule only a trader as fast as the tick could meet a price that stood for a second.
Now a price must stand most of every step to move the observation at all, so a trader who answers at an agent's pace meets it.
"Whoever moves a price pays" is true of a road with someone else in it. The law's header says so now (WHAT THE LAW CANNOT DO ALONE).

## The witness

The old text (`tools/embertide-mesh/futarchy/futarchy-law.mjs`, untouched) answered 24 roads whose trades stand on the grid, and the law gave
the same averages within a millionth. On purpose it does not any more: a trade on a boundary used to count for the step that ended as it came.
On the grid, with no trade at the opening and a window of whole steps, this law's observation over a step is exactly the text's over the step
before. So the sum the text kept is `cum - open * step + acc` at the close, and `tests/witness.rs` holds the text's two averages within a
millionth of that, its verdict by the threshold, and its scores within a hundredth of a $WARM, for all 24 roads; each road's fit to those
conditions is asserted. This law's own verdict, a step later than the text's, differs from the text's on none of the 24.

## How it was built

To this seat's brief an Opus builder made the change (b3e1e9b902, d3eda2625d): the rule, the proof, `tests/stood.rs`, the first test with the
tick's rule written in it as its witness, the witness test by the identity above. Where the brief did not fit: the brief's fixed point
`p + d f / (1 - f)` is exact only with `d` the observation's own band, so the test asserts the true fixed point `p (1 - f) / (1 - 1.01 f)`; a
buy and its sale at one instant leave the pool a few micro richer by the rounding, so the next boundary is held within a hundred millionth,
not equal.

The builder then attacked its own rule and reported what is under "What the law cannot do alone". This seat checked it on the code, wrote it
into the law's header and added the test that holds both halves of it (20d062c84a), so the next reader of the law cannot claim more than it does.

## The tests (the new Mac, nice 19, at the lane's tip)

`FUTARCHY_PROOFS=require cargo test --offline --locked --no-fail-fast`: **48 passed, 1 ignored**, no warning: `core` 4, `law` 8, `nest_words` 8,
`proofs` 2 (eleven `unsat`), `roster` 7, `settle` 10, `stood` 8, `witness` 1. `on_a_nest`, run with `--ignored`: 1 of 1 in 51 s.

- `tests/law.rs the_trader_who_stood_a_second_at_every_boundary_walks_nothing`: the 8,638 trades on this law by the core's calls and by the
  fold (equal to the field), and on the tick's rule written in the test; then the trader that holds, for whom the two rules are equal to the byte.
- `tests/stood.rs`, eight on the core alone: a quiet pool is the law as it was (400 branches carried in random pieces); a trade counts from
  its own stamp; a trade undone at the same stamp; a spike's size; under half the time; over half the time; 119,700 boundaries with spots set
  anywhere in the law's range, each inside its band, the range and what stood; and what the law cannot do alone and what another trader does.
- `tests/core.rs`: a carry in bounded steps is one carry, `acc` included (2,000 branches, five bounds each); a road carried to a boundary has
  taken it, and the trade stamped on it changes nothing of it.

**No verdict flipped and no asserted number moved.** Three printed lines moved, each checked by hand against the identity: the whale's old
mark, 770.666448 to 770.380263 $WARM (its buy counts one step later); PASS's average with sellers of 4500, 0.055455 to 0.055559 (still
failed), and with sellers of 450, 1.633603 to 1.633336 (still passed). On the scratch Nest, at the lane's tip: road 1 passes at 1.2207
against 1.1032 (Nov 11: 1.2222 against 1.1036), road 2 fails at 1.0846 against 1.2205, road 3 fails on receipts of 0.25 with the results and
the pool summing to −1 micro. That test's trades are stamped by the scratch Nest's own clock, off the grid, so its averages move a little from
run to run; its verdicts do not.

**Mutants: 15 deliberate breaks of the law, the fold and the new proof, each caught.** The raw spot summed in place of the held one; averaged, then held; the tick's rule put back; the step's sum never started again; a boundary at the carry's own instant left waiting (the tests then loop for ever and are ended after 150 s); the average taken of the held spot in place of the observation; the sum divided by a millisecond more than the step; a trade taken before its carry; the carry made after the trade; the decision read without its last carry; the fold ended without its carry; no band at all; a carry that does not say it fell short; and `proofs/stood.smt2` with either of two premises dropped (z3 then answers `sat`, and `the_proofs_hold` is red). Carrying only the traded branch at a trade is the same road, as on Nov 20's third landing, and was not run again.

## What is not done, and what is not proven

- **The program on a chain is another lane.** `poleis/program/poleis_roads` keeps a branch in an account field by field: it lays `acc` beside
  `cum`, drops the boundary a trade could not carry, and measures a boundary's compute units again. This crate builds and passes without it.
- **The block's edge.** On a chain the finest instant is a block. A trader placed last in one block and first in the next, at every block of a
  window, would stand in the pool without being met. Nobody can buy that placement for a whole window today. A fee on each trade prices it out.
- **The fee on every trade, which the design names as what the house keeps, is in no code.** The law still says "no fee". It is the next piece.
- **An empty road** (above): no rule of the law closes it. Whether a road's verdict should count at the seal when too little of the floor
  stood in it is the seal's lane to say, and his to rule.
- **The receipts before this one are not edited.** `THE-WORLD-THAT-HAPPENED-SETTLES-ON-ITS-RECEIPTS-LANDING.md` and the two after it say what
  was so on their code; where they say a mover pays, read: a mover who holds, or who is met.
- **The model's runs** (`token-economics-simulators-2026-11-20`) had a pumper that buys and holds. They do not speak of a trader that leaves
  before the close or pushes the other world, and they were not run again.
- A boundary takes the floor of its sum, so an observation can drift down by at most one unit (10^-12 of a $WARM) a step. Both pools drift alike.
- The last step never reaches the decision (no boundary at or past the close is applied), as before.
- Not proven by z3: that the sum cannot leave u128 (the compiler checks the range's product, `PRICE_HI * WINDOW_MAX_MS`, and every sum saturates).
