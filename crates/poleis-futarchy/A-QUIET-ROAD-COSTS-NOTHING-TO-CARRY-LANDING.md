# A QUIET ROAD COSTS NOTHING TO CARRY: a stretch in which a pool's price is at rest is one step of the market's law (Nov 20 to 21 2026, lane/a-quiet-road-costs-nothing-to-carry, claude-81203e94)

## His words, UTC

- **Nov 20 20:50:38,** to "Make a market nobody is trading cost nothing to keep current, next?": "Yes, build it".
- **Nov 21 14:11:00,** after this seat's model ran out in the middle of the review: "Try again".
- The house's law it serves (CLAUDE.md, The screens): "Nothing at rest costs anything"; and of the room's solver (the foundations, 1): "a
  converged cell costs nothing".

## Cost first

| | before (main d54699f24a) | after (this lane) |
|---|---|---|
| a road nobody traded, three days at a step a minute: transactions to its verdict | 66 (65 carries, then the decide) | 1 (the decide, 7,958 compute units) |
| carrying a road nobody traded, 1 boundary behind | 6,823 CU | 5,844 |
| ... 66 boundaries (`MOST`) behind | 107,079 | 5,844 |
| ... 10,000 behind | about 152 calls of up to 107,079 | one call, 5,846 |
| ... a year of minutes (525,600) behind | about 7,964 calls of up to 107,079 | one call, 5,846 |
| a boundary walked one at a time (a price still catching up; both branches) | 1,542 | 1,006 |
| the dearest carry (66 walked boundaries) | 113,552 | 74,691 |
| a carry with nothing to carry | 4,043 | 2,868 |
| a trade on a road up to date | 8,914 | 6,725 |
| the dearest trade | 112,020 | 73,457 |
| a trade on a road nobody touched for its whole window | refused (`RoadBehind`) | 9,687, no carry first |
| the market's program | 98,480 bytes | 101,968 (+3,488), sha256 `c6255f7933266f94…` |
| Poleis | 456,800 bytes | the same bytes, sha256 `aa65f59df1d3e91a…` |
| the law's tests (the new Mac) | 58 passed, 1 ignored; the core alone 17 | 64 passed, 1 ignored; the core alone 23; no warning |
| the law's proofs (z3) | 11, each unsat | 12, each unsat |
| the market program's tests | 26 | 26, seven of them rewritten for what a road costs now |
| Poleis's tests | 216 passed, 1 ignored | the same |
| JavaScript | | none |
| the lock files | | unchanged |

The diff against main: 13 files in `poleis-futarchy` and `poleis/program/poleis_roads`, 621 lines in, 103 out; this receipt is the
fourteenth. Nothing was built or tested on the serving Mac. Nothing is deployed.

## What stands now

**A stretch at rest is one step.** The law averages a price that moves only at the boundaries of its grid. In a whole step in which the pool
does not move, the price that stood is the price the observation already had, once the observation has caught up with the pool's own price
(or its band is zero wide). Then every whole step after it leaves the observation where it is, exactly, and any number of them is one
addition (`Branch::walk`, proved in `proofs/quiet.smt2`).

**Only a price still catching up is walked.** A step a trade began in the middle of, and every step of an observation still catching up with a
price a trade moved, are walked one at a time as before, and a bounded carry counts only those. A walked step that is whole from its start
takes its held price itself, which is the boundary's division done exactly: that is why a walked boundary is cheaper too.

**On the chain:** a road nobody traded is decided by its `decide` alone; a road laid late by `lay_for` takes its first trade with no `carry`;
`RoadBehind` now means only a price that moved far and was then left, more than 66 steps of catching up still to walk. `MOST` stays 66.

**The law's answer did not move by a bit.** The loop as it stood is kept in the tests as the witness, run beside the new one over a seeded
sweep and held equal field for field after every carry.

## How it was built

To this seat's brief an Opus builder made it.

- The brief's design made a walked boundary dearer (1,940, then 1,730 CU against 1,542), and the dearest carry broke its bound of 120,000
  (139,845, then 125,969). The builder applied the brief's own fact to the walked steps too: a whole step takes its held price, with no
  multiplication into the sum and no division. Its choice stands.
- The skip also asks that the observation be inside the law's range and the step inside its longest window, so nothing it adds can leave
  u128; outside them the old loop walks.
- `Branch::reach` is gone (nothing called it but its tests); `carry` keeps its signature and what it answers.
- This seat read every changed line, ran the four suites and the proofs itself on the new Mac, ran 21 deliberate faults of its own, and added
  the test two of them asked for. The session's model ran out in the middle of that review, and it went on from where it stood on his "Try
  again".

## The tests (the new Mac, nice 19, at the lane's tip)

The law: 64 passed, 1 ignored (58 before); the core alone 23 (17); the twelve proofs, each unsat (`FUTARCHY_PROOFS=require`). The market's
program: 26. Poleis: 216 passed, 1 ignored, its bytes unchanged. No warning.

`tests/quiet.rs`, the witness, and what it held:

- **The skip is the walk, field for field.** 1,200 roads of every terms the law takes (a step from a second to four hours, a lag limit from 1
  to 10,000 bp, windows from 2 to 50,000 steps), through 2,643 trades that moved the pools, at every kind of instant (before the opening,
  mid-step, on a boundary, one before and one after it, at the close, past it, backwards). 93,076 carries held equal to the loop as it stood
  after every one; 40,656 bounded calls, 32,142 of which stopped short having walked exactly their bound; 8,090,894 boundaries skipped
  against 101,654 walked, the longest skip 49,391.
- Bounded carries until they stand are one carry. A road nobody touched reaches any instant in one call at a bound of one. A chase that
  outlasts the window is ended and never handed the close again. A bound of 0 still walks one.
- **The long road:** a window of 366 days at a step of one second, untouched, carried to its close in one call, every field equal to its
  closed form.
- **The states no road reaches** (this seat's): a held sum on a branch standing on its boundary, and an observation climbing past the law's
  range on reserves past it, carried by several instants and by one, a boundary a call and fifty.

`proofs/quiet.smt2`: a whole step from its start takes its held price; held at the observation it leaves it where it is and adds the
observation times the step, and k such steps add k times that; and the boundaries the skip applies are exactly those the loop would.

In the market's program: `tests/cost.rs` prints and bounds the table above; `tests/refusals.rs` meets `RoadBehind` through a real chase (a buy
that lifts a price 2.25 times, about 82 steps of catching up); in `tests/rule.rs` a road laid eighty steps late takes its first trade with no
carry, and the two roads stay equal field for field; `tests/parity.rs` (the program is the fold, trade by trade) is unchanged and green.

**Deliberate faults: this seat's 21 at the lane's tip.**

- **18 caught at once**, each by named tests. Four of them by tests that never ended under the fault and were stopped after 240 s: a
  stretch skipped from the middle of a step, a skip that leaves the next boundary where it was, a step from its middle read as whole, and a
  branch with nothing to carry that says it stopped short.
- **2 stood, and were closed.** Dropping `acc == 0` from the test of a whole step, and keeping a walked step whole after its observation
  left the law's range. Both guard states the law never makes, which the sweep never met. The new test of those states catches both.
  No line of the law changed for it.
- **1 is no fault:** `MOST` at 67. The tests hold the bound `MOST` keeps (a carry under 120,000 compute units, a trade under 180,000),
  not its number, and 67 keeps it too.

## What is not done, and what is not proven

- **A long chase still costs what it cost.** A price pushed across the law's whole range at a lag limit of 1 bp needs up to about 345,000
  walked steps to be caught up, about 5,200 carries. Only a trade starts a chase, and no trade lands while the road is behind, so whoever
  pushes cannot keep it behind; but whoever wants the road decided sends those carries. The board's law (its step and its lag limit) sets how
  many.
- **`MOST` stays 66.** By the new measure about 110 would keep the same bounds; it is a measured setting, not changed here.
- **The market's program grew by 3,488 bytes:** the walk has two roads now.
- **The skip needs the observation inside the law's range and the step inside its longest window.** Outside them the old loop walks, as the
  witness holds.
- **On no chain.** Mollusk only; a deploy is his ceremony.
