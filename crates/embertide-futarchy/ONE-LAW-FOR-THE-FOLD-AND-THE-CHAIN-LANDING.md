# ONE LAW FOR THE FOLD AND THE CHAIN: the market's law in the form a program on a chain can carry, with no line of its arithmetic changed (Nov 20 2026, lane/one-law-for-the-fold-and-the-chain, claude-81203e94)

## His words, UTC

- **Nov 11 00:55:** "i think we should get into ai futarchy with formal verification for governance".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market". The offer it answers, in
  this seat's words: "what a bet on a snowtracer is on a chain, how the house earns on the trading, and how the market's verdict reaches the
  contract's seal".
- **Nov 20 14:26:19:** "why u asking build or change? u think u can design better?"

The design: ec-workspace `docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, the section "Nov 20 2026 14:03 UTC". This lane is the third
piece. It changes no rule of the market. It makes the law one thing two readers can run: the fold on a Nest, and a program on a chain.

## Cost first

| | before (main bfe1793a06) | after (871959254b) |
|---|---|---|
| the law | one file, 327 lines, lists and text all through it | the core, `src/law.rs`, 319 lines, no allocation and no std; the fold, `src/fold.rs`, 123 lines, built on it |
| the core alone (`cargo build --no-default-features`) | not a thing | builds as `no_std` with no allocator; with `--features fold`, `no_std` with `alloc` |
| the proofs | ten, `unsat` | the same ten, untouched |
| `cargo test --offline --locked` with `FUTARCHY_PROOFS=require` (the new Mac) | 35 passed, 1 ignored | 39 passed, 1 ignored, no warning; every assertion that stood is untouched |
| the core's own tests with no feature | none | 2 of 2, and the ten proofs |
| the ignored test on a scratch Nest | 1 of 1, 51 s | 1 of 1, 51 s |
| the lock file | | unchanged: no new dependency |
| a carry of a whole window of three days at a step a minute, bounded to 100 boundaries a call | | 44 calls |
| JavaScript | | none added |

The code's diff: 9 files, 493 lines in, 121 out, all inside `embertide-futarchy/`; this receipt is the tenth. Nothing was run on the serving Mac.

## What was so

The law was a fold: it reads a road's whole list of signed entries and answers the market. It needs lists and text. The next piece is the same
market as a program on a chain, which sees one trade at a time, keeps the two pools and each purse in accounts, and has no allocator. Written
against the fold's file it could only have copied the arithmetic, and a copy drifts.

## What stands now

- **The core** (`src/law.rs`, no allocation, no std): the terms, a purse of a deposit, a swap, the grid, a branch, the decision, the receipts, a
  purse's settle, what comes back. Every function that stood was moved whole.
- **The fold** (`src/fold.rs`): what walks a list: the entries in the Nest's order, the refusals that are an entry's alone, the roster, the
  traders, the scores. It is built on the core and makes the core's calls. `law::*` still names everything it named.
- **A program does not put the law together from its pieces.** Each thing a program does is one call of the core, and the fold makes the same
  calls:
  - `stamp_check`: the rules of time (inside the window, not before the last trade taken). They are the core's now, not the fold's.
  - `trade_on`: one trade whole: the law's refusals, both branches carried to its stamp, the pool and the purse moved.
  - `carry`: one bounded carry of both branches toward an instant, true when both stand there. However the calls fall, they are the one carry.
  - `decide_at`: both branches carried to the instant, its boundary included, then the decision.
  - `settle_at`, `pool_at`: the receipts asked of the decision, then the purse or the branch of the world that happened.
  - `deposit_check`: a deposit a road can carry.

## How it was built

To this seat's brief an Opus builder made the split (ccd6c2c640, ffbcf26621) with every earlier test green and untouched, and reported three
ways the brief had left a program to get the law wrong:

1. the rules of time lived only in the fold, so a program could take a trade stamped after the close;
2. the order of a trade (ask, carry, move) was the caller's to keep;
3. a road with one branch carried to the close and the other not is read as failed, with no error.

This seat closed each with one call that cannot be made wrongly (85c96de614): the list above. Of the builder's second point one half is not
so: carrying only the branch a trade is in gives the same road, because the other branch's carry can wait until the decision (checked on the
new Mac: with `trade_on` carrying only the traded branch, every test still passes). What must not happen is taking a trade before ITS branch
is carried, and `trade_on` carries both.

## The tests

39 on the new Mac at 871959254b; 4 new in `tests/core.rs`; the 35 that stood pass with every assertion untouched (one feature line was added
at the head of four test files).

- **A carry in bounded steps is one carry:** 2,000 branches (505 on a boundary, 520 at or past the close, 659 already carried part of the way),
  at 1, 2, 7, 100 and 5,000 boundaries a call: 7,568,727 bounded carries, each the one carry.
- **The core alone carries a road as the fold does:** 240 roads (119 both ways, 128 with a roster), 14,400 entries, 8,899 trades taken by the
  core's calls alone with the fold's branches, purses, refusals, decision and settle to the field. The rules of time are asked of the core. Then
  the same roads **as a chain sees them**: a clock that never runs back, only the trades that were taken, the road carried in bounded calls of 1
  to 40 boundaries before each trade and once by anyone in between (12,103 carries that did not yet reach their instant): the fold's road again.
  Receipts the law refuses settle nothing.
- **A road carried to a boundary leaves that boundary to the trade stamped on it**, in one call or in many, a bound of 0 read as 1.
- **A deposit a road cannot carry is refused.**

**Mutants, each caught** (the new Mac, 11): a trade at or after the close is in time; a trade before the last one taken is in time; a trade is
taken before the road is carried to its stamp; a bounded carry applies the boundary at its own instant; the decision is read without the last
carry; a position settles on the purse of the other world; the pool settles on the other branch; a bound of nothing is not read as one; the fold
forgets the stamp of the last trade it took; a carry says a branch reached its instant when it did not; a branch at the end of its window is
handed the close for ever.

## What is not done

- **No program.** This lane makes the law carryable. The market's own program beside Poleis is the next piece, and the contract's seal reading
  its verdict the one after. A deploy is his ceremony.
- **Not built for a chain's target here.** The core builds as `no_std` with no allocator on the new Mac's own target. That it builds for SBF and
  what a boundary costs there in compute units are the next lane's to measure.
- **A key whose first trade is refused** has no trader in the fold. A program that opens a position before a first trade has one the fold does
  not list; its settle is exactly 0 either way.
- **Nothing keeps a balance, a judge is a key a road names and its first word is final, no market runs**, as before.
- **The proofs are of the arithmetic, not of the Rust source**, as before.
