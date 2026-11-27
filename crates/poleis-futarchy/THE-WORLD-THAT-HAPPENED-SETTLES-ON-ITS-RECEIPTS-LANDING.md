# THE WORLD THAT HAPPENED SETTLES ON ITS RECEIPTS: a judge's word says what a bet was worth, and every trader can sell (Nov 20 2026, lane/the-world-that-happened-settles-on-its-receipts, claude-81203e94)

## His words, UTC

- **Nov 11 00:55:** "i think we should get into ai futarchy with formal verification for governance".
- **Nov 20 12:13,** in his sketch for the pool side's release: "meta swarm allows stakers to bet on SNOWTRACERS", and "$POOL is the futarchy's delayed
  gratification".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market".
- **Nov 20 14:26:19,** over this seat's first form of the settle and its question "Build it this way: bets settled on receipts, the market its own
  program?": "why u asking build or change? u think u can design better?"

The design and its run: ec-workspace `docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, the section "Nov 20 2026 14:03 UTC". This lane is
its first piece, off chain, in the law that was already proven. A proposal of this seat's that he may strike; nothing here starts a market.

## Cost first

| | before (main 4917d0e538) | after (acd577aa83) |
|---|---|---|
| the law (`src/law.rs`) | 201 lines | 268 lines; no dependency, as before |
| the Nest's side and the command (`src/nest.rs`, `src/main.rs`) | 276 and 41 lines | 349 and 60 lines |
| the proofs | six, `unsat` | nine, `unsat`, in 0.23 s (z3 5.1.0, the new Mac) |
| `cargo test --offline --locked` with `FUTARCHY_PROOFS=require` (the new Mac) | 14 passed, 1 ignored | 26 passed, 1 ignored, no warning |
| `cargo build --offline --locked --no-default-features` | exit 0 | exit 0 |
| the ignored test on a scratch Nest | 1 of 1, 42 s (Nov 11) | 1 of 1, 51 s (a third road, 8 s of window) |
| a road's ref | | byte for byte as before when it is one way and names no judge (pinned in `tests/nest_words.rs`) |
| the judge's word | none | one reading of the road and one signed entry |
| a count | none | the same reading as a settle, one GET a 500 of the receipts' type, one ref written once |
| at rest | nothing | nothing: no timer, no poll |
| JavaScript | | none added |

The code's diff: 12 files, 547 lines in, 45 out, all inside `poleis-futarchy/`; this receipt is the thirteenth. Nothing was run on the serving
Mac.

## What was so

A trader's score marked what it held at the pool's OWN average (`score_of`). Nothing outside a market ever said who was right, so a trader who
lifted a price and held it was marked at the price it had made. The receipt of Nov 11 said it: "a trader who holds a price for the whole window still
sets it, and on paper nobody loses money by doing so." And a purse opened with $WARM alone, so it could only buy first and a lifted price met no
seller. On this code (`tests/settle.rs`): a whale that buys 900 of PASS in a pool of 1000 a side and holds it passes its road and is marked
**+770.67 $WARM** by the old mark; the same whale settled on receipts of a tenth of a $WARM a claim is at **-852.63**.

## What stands now

- **Two worlds, as the law always had them.** PASS is the world where the road is taken, FAIL the world where it is not. In each, the base is a
  claim on what the road's measure turns out to be in that world, worth zero to one $WARM.
- **A road can be bet both ways** (`Terms.both_ways`). Every purse then opens with the pool's own mix, in both worlds: its budget of $WARM and the
  claims that budget is worth at the opening price (`Terms::opening_claim`). So a trader can sell first. There is no amount to choose: the mix
  is the even one. From the opening price a purse's whole claim lowers the price by exactly the factor its whole budget lifts it
  (`proofs/both-ways.smt2`), and a bear of the same size that sells after a bull bought more than undoes it. A road one way is the law as it was.
- **The receipts.** After the close the road's judge, a key the road names, signs what a claim was worth in the world that happened
  (`futarchy.receipts`). The first such word stands and a later one changes nothing. A word stamped before the close, or naming the world that did
  not happen, counts for nothing, and the judge's own door (`judge_says`) refuses to send one and says why.
- **The settle** (`settle_of`): each trader's result is what it holds, marked at the receipts, against what it opened with, marked the same. A
  trader that holds what it opened with scores exactly nothing. The world that did not happen counts for nothing. What the traders win between
  them the pool loses, to the rounding of one floor a trader (`pool_result`).
- **A holding is marked exactly, however large** (`worth_of`): its whole multiples of the price unit and its remainder are marked apart, so no
  product leaves u128 and nothing saturates.
- **The count on the Nest** (`futarchy/counts/<id>`, the verbs `receipts` and `count`): written once, like the decision.
- **The old mark** (`score_of`) stays for a road with no judge and for the witness, and now marks a purse against what it opened with, so on a
  road both ways it is the settle at the price the pool made. On a road one way it is the number it was (the text's 24 markets still agree).
- **Nothing else moved.** A swap, the grid, the average and the decision are the proven ones, untouched.

## How it was built

To this seat's brief an Opus builder wrote the first cut (bc36b5d80a, 3a87c5599b: the settle, the receipts, the count, two proofs, six tests). This
seat read every line and changed three things before any of it was handed (acd577aa83):

1. **The first cut let a road name any opening claim.** The model's one condition was a claim worth about the budget (with a quarter of that, a
   whale holding 35 % of the size passed its road every month). Left as an amount, it was a number for whoever lays a road to get wrong. It is now
   one bit, and the law derives the amount.
2. **The first cut took the judge's first word whatever world it named.** The builder found it and said so: one wrong word would have blocked a
   road's count for good. Fixed in the reader (`receipts_among`) and at the judge's door.
3. **The first cut's bound on a holding was not a bound** (a purse can hold what others sold it). `worth_of` now needs none.

## The tests

26 on the new Mac at acd577aa83, 9 of them new in `tests/settle.rs` and 3 in `tests/nest_words.rs`:

- **The law's own** (`tests/law.rs`, 7): every sequence of three trades of a small pool is now read twice, one way and both ways (110,592 markets
  each, 0.42 s each): both tokens conserved to the micro with the opening claims in, a refused entry changes nothing.
- **The settle** (`tests/settle.rs`, 9): the old mark pays whoever lifts a price and the settle charges it; ten sellers meet a whale's lift and the
  road fails, where one way all ten are refused and it passes; a road both ways opens with the pool's own mix or is no road; the whole claim
  lowers a price by the factor the whole budget lifts it, to a millionth, on four pools, and a bear after a bull brings the price under where it
  opened; the world that did not happen counts for nothing; a seeded sweep of 160 markets (104 both ways, 2,595 sells taken) where the traders'
  results and the pool's sum to at most one micro a trader, nobody loses more than it opened with, and 165 traders who held what they opened
  with scored exactly nothing; a holding is marked exactly against the plain product on 20,000 draws and at the top of u128; an open road and a
  worth over one are refused; at the pool's own average the settle is the old mark, both ways and one way.
- **The Nest's words** (`tests/nest_words.rs`, 7): a road one way with no judge writes the bytes it wrote before; `both_ways` is true or false and
  nothing else; the receipts are the judge's signed word after the close and nothing else (another key, no signature, another road, a worth over
  one, a world that is neither, a stamp before the close, a road with no judge); a word of the other world does not stand in the way.
- **On a scratch Nest** (`tests/on_a_nest.rs`, by name): a third road, both ways and judged. Two wardens sell first, one buys. The judge's word is
  refused before the close and when it names the other world; a word of the other world put on the log past the door leaves the road uncounted;
  the judge's word of the world that happened settles it at 0.25, three traders and the pool summing to -1 micro; a second count writes nothing.
- **The proofs** (`proofs/*.smt2`, 9 `unsat`, each with premises that hold): the six of Nov 11, and `settle-rounding` (one trader against the pool:
  the floors differ by at most one), `settle-bounded` (no trader loses more than it opened with; holding what it opened with scores nothing) and
  `both-ways` (over the reals, before any floor: lifted times lowered is the opening price squared).

**Mutants, each caught** (the new Mac, 15): the settle forgets what a purse opened with of the claim; a purse opens with no claim on a road both
ways; the other world's purse is counted; a worth over one is taken; receipts of the other world settle a road; the opening claim ignores the
opening price; a holding's remainder is not marked; the old mark forgets the opening claim; a road both ways is carried whatever its claims come to;
the pool's result forgets what the pool opened with; any key's word is a road's receipts; a word before the close is taken; the judge's first word
stands whatever world it names; `both_ways` is taken whatever it holds; the judge's door sends any word (this one caught only on the scratch Nest,
43 s).

## What is not done

- **Nothing lays a road and no market runs on the live Nest**, as on Nov 11. This lane makes the law honest; it starts no market.
- **A judge can be wrong, and its first word is final.** The judge is a key a road names. The router's oracle does not yet say what a trail brought
  the treasury, so today a hand would. On a chain the month's root is to be sealed by both chambers.
- **A road one way with no judge is still a road**: the allowances' roads of Nov 11. Such a road is settled by the old mark, which pays whoever lifts
  a price. Nothing says yet what an allowance's receipts are.
- **Every trader has one budget.** A staker's own size (the fuel its stake gave, times the time staked, following its count) is the next piece.
- **No fee.** The house's fee in $WARM is charged beside the pool, by whatever runs the market.
- **Not on a chain.** The market's own program, and the contract's seal reading its verdict, are the pieces after. A deploy is his ceremony.
- **The proofs are of the arithmetic, not of the Rust source**, as on Nov 11. `both-ways` is over the reals; on this code the floors leave it true
  to a millionth on the four pools tested, not proven for every pool.
- **Found by the builder and left as they were:** `road_of` takes the default for an amount that is not an integer (a budget of 12.5 is read as
  1000), as it did; the command's `micro()` adds a fraction with a plain `+` after a checked product, harmless in range.
