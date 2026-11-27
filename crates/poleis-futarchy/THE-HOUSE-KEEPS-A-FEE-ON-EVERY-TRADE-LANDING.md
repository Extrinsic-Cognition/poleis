# THE HOUSE KEEPS A FEE ON EVERY TRADE: $WARM, beside the pool, never in it (Nov 20 2026, lane/the-house-keeps-a-fee-on-every-trade, claude-81203e94)

## His words, UTC

- **Nov 11 00:55:** "i think we should get into ai futarchy with formal verification for governance".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market". The offer it answers, in
  this seat's words: "what a bet on a snowtracer is on a chain, how the house earns on the trading, and how the market's verdict reaches the
  contract's seal".
- **Nov 20 14:26:19:** "why u asking build or change? u think u can design better?"

The design (ec-workspace `docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, the section "Nov 20 2026 14:03 UTC", "What the house
keeps") names the fee on every trade as what the house keeps. No code had it: the law said "no fee". This lane is that fee, in the law and as
one term of the program on a chain. The RATE is not this seat's: it is a road's own term, a number of his engine.

## Cost first

| | before (main 7476571a78) | after (this lane) |
|---|---|---|
| the law's core, `src/law.rs` | 357 lines | 401 lines, still no allocation and no std |
| a road's terms | | one more: `fee_bp`, the share of every trade the house keeps; 0 is the law as it was |
| a branch (a pool and its average) | seven numbers | eight: `fees`, a u128 of micro $WARM kept beside the pool |
| the proofs | eleven, `unsat` | eleven, `unsat`; four restated with the fee (`buy`, `sell`, `roundtrip`, `both-ways`) |
| `cargo test --offline --locked` with `FUTARCHY_PROOFS=require` (the new Mac) | 48 passed, 1 ignored | 58 passed, 1 ignored, no warning |
| the core's own tests with no feature | 10 | 17, no warning |
| with a fee of 0 | | 200,000 seeded trades answered word for word as before the fee; no number of any earlier test moved |
| the program on a chain | 87,992 bytes; a Road 419 bytes, rent 3,807,120 lamports | 91,280 bytes; a Road 455 bytes, rent 4,057,680 lamports |
| a `trade` on a road up to date | 8,706 compute units | 8,908 (the fee's arithmetic, paid at 0 bp too) |
| a `trade` carrying 66 boundaries | 111,742 | 112,014 |
| the program's tests | 20 passed | 20 passed, no warning; half of the sweep's roads now carry a fee |
| the lock files | | unchanged in both crates |
| JavaScript | | none added |

The diff: 22 files, 550 lines in, 100 out, in `poleis-futarchy/` and `poleis/program/poleis_roads/`; this receipt is one more. Nothing was
run on the serving Mac.

## What stands now

**The fee is $WARM, and it never touches the pool.** It is taken from what a buy pays before that reaches the pool, and from what a sell
receives after that has left it: the ceiling of its share, so no trade is too small to pay. Each pool keeps it in one counter beside its two
reserves.

- **The pool's price moves only with a position some trader holds.** No claim is ever in a fee, so the claims are conserved between the purses
  and the pool exactly, and the reserves move only as a position does.
- **A buy and the sale of what it bought leave the pool where it was, and cost both fees on the whole size.** On this code, 900 $WARM in and
  back against pools of 1,000 at 100 bp: 17.910001 lost (9 on the way in, 8.91 on the way out, a micro of rounding), the pool left at 1,000
  claims and 1,000.000001 $WARM. Over 99,991 seeded round trips the claims always come back exactly.
- **The house's take is a number of its own.** At the settle the pool's result is its reserves marked at the receipts, and its fees. On a
  small road both ways with a judge: with no fee the traders are at −46.815166 and the pool at +46.815165; at 100 bp the traders are at
  −50.270900 and the pool at +50.270899, of which 4.726967 is fees.
- **On the program**, the same road of three owners: the house staked 2,000 and collected 2,058.717056 at 100 bp against 2,046.765714 with no
  fee; the road's fees in the world that happened are 13.532356. The instruction code did not change: `collect` gives the house what is left,
  and what is left now holds the fees. The board's book still closes to the micro after every instruction, and the program is still the
  fold to the field, with a fee.
- **Standing in a pool for an instant now costs, and buys nothing.** The trader of THE PRICE THAT STOOD (it buys at every tick and sells a
  second later), at 100 bp: six round trips of 900 cost it 107.46, all of it the house's fees; the price is left eight billionths over where
  it opened; the road fails. All in at every tick until its purse is spent: 800 round trips, 999.999164 of its 1,000 in the house's fees, the
  price 0.84 parts in a million over the opening, the road fails.
- **The fees of the world that did not happen are void**, with every trade of that world, by the law's own word.
- **A road that names no fee keeps none.** On a Nest a road's ref may name `fee_bp`; a bare road's bytes and reading are what they were.

**What it costs: the pool's own mix is even only to the fee.** A purse's whole claim still lowers a price by `(1 + x)^2`. Its whole $WARM now
lifts it by `(1 + phi x)^2`, because a buyer's fee comes out of what it buys with and a seller's out of what it is paid. So with a fee,
betting against is a little stronger than betting for, by the fee and no more: the ratio lies between `phi^2` and 1 (proven over the reals,
`proofs/both-ways.smt2`), about 2 % at 100 bp.

## The first build did not land, and why

This seat's first brief said the usual thing: the pool keeps the fee, so the whole of what a trade pays enters the reserve. The builder built
it, attacked it, and reported before anything else: **a fee left in the reserve moves the price by itself, held by nobody.** A buy and the sale
of what it bought put the claims back and leave the fee's $WARM in the pool, so the price stands higher for the rest of the window. On the
law's own test road, at 100 bp: eleven round trips of 900 cost 106.959352, left PASS at 1.106959, and **the road passed**. With no fee the
same trader fails. A fee kept that way sells a verdict, at about the lift wanted times the pool's depth.

The builder's cure was to keep the fee beside the reserves. This seat's second brief added the other half: **never in claims.** A fee kept
in claims beside the pool takes those claims out of the pool's circulation, and the price stands displaced again: one round trip of 900
leaves it at 1.0095. So the fee is always $WARM.

Both are kept in `tests/fee.rs` as the rule's witnesses, written in the test in the same integers.

## The tests (the new Mac, nice 19, at the lane's tip)

`FUTARCHY_PROOFS=require cargo test --offline --locked --no-fail-fast`: **58 passed, 1 ignored**, no warning: `core` 4, `fee` 9, `law` 8,
`nest_words` 9, `proofs` 2 (eleven `unsat`), `roster` 7, `settle` 10, `stood` 8, `witness` 1. The core alone, with no feature: 17 passed, no
warning. The program: 20 passed, no warning.

`tests/fee.rs`, nine: with no fee the law is the law as it was (200,000 trades against the function before the fee, written in the test);
the pool's price moves only with a position (117,264 trades with fees of 1 to 9,999 bp: the pool's claims move exactly by the purse's, its
product by less than one unit of the reserve that grew); a round trip leaves the pool where it was and costs both fees; a fee of 10,000 bp is
no road, and a trade of one micro buys nothing; $WARM is conserved with the fees and claims without them (160 roads on the fold); the trader
who stood a second at every tick, on the first build's rule (the witness) and on this law; the fee kept in claims (the second witness); the
settle counts the fees as the house's own number; the fee of the world that did not happen is void.

**Mutants: 17 deliberate faults, each caught.** A buy's fee enters the pool with what it pays (the first build's hole); a buy trades all it
pays; a sell is paid its gross; a sell's fee stays in the pool; the fee is not kept; the fee is floored; the pool's result forgets its fees;
a fee of 10,000 bp is carried; `min_out` asked before the fee; the Nest reads no fee; and on the program: a road's fees not kept in its
account, a road forgetting its fee, the law handed no fee; and three proofs with a premise weakened (z3 then answers `sat`). **One survived
at first:** weakening "only the net reaches the pool" in the buy's proof left it `unsat`, because no part of its claim spoke of the net
against the amount. The claim now states that the pool receives at most what the buy pays and never less than nothing, and the fault is
caught.

## What is not done, and what is not proven

- **The rate.** A road names it; nothing in the house names a road's rate, and no regulator of it is built. It is a number of his engine.
- **An empty road stays cheap.** A trader alone that lifts a price, holds it and leaves before the close now pays two fees on what it held:
  by the builder's arithmetic about 0.30 $WARM to lift a price 3 % against a pool of 1,000 at 100 bp. The fee prices the instant, not the
  empty room (`THE-AVERAGE-FOLLOWS-THE-PRICE-THAT-STOOD-LANDING.md`, "What the law cannot do alone").
- **The void world's fees.** A trader who pushes the world that then does not happen pays nobody, fees included.
- **The pool's rounding.** A round trip leaves the pool about a micro richer. Moving a price that way is far dearer than holding it (each
  trip pays at least two micro of fees), and nothing bounds it over a whole road.
- **Betting against is stronger than betting for, by the fee** (above). No test treats it as an attack.
- **The model was not run with a fee** (`token-economics-simulators-2026-11-20`).
- **The program is on no chain**, and Poleis reads no verdict yet: the next lane.
