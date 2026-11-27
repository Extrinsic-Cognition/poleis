# THE MARKET IS ITS OWN PROGRAM: the house's decision market on a chain, beside Poleis and never inside it (Nov 20 2026, lane/the-market-is-its-own-program, claude-81203e94)

## His words, UTC

- **Nov 11 00:55:** "i think we should get into ai futarchy with formal verification for governance".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market". The offer it answers, in
  this seat's words: "what a bet on a snowtracer is on a chain, how the house earns on the trading, and how the market's verdict reaches the
  contract's seal".
- **Nov 20 14:26:19:** "why u asking build or change? u think u can design better?"

The design: ec-workspace `docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, the section "Nov 20 2026 14:03 UTC". This is its fourth
piece: what a bet is on a chain. The shape (its own program beside Poleis, a board a polis, three roles each a key) is this seat's, said to him
at 14:26 and again at 16:20 as his to strike.

## Cost first

| | |
|---|---|
| the program, built for the chain | 87,992 bytes (Poleis is 446,088) |
| its source | 1,312 lines of Rust in `src/`, 1,620 in `tests/`; no JavaScript |
| what it depends on | `pinocchio` 0.11, `pinocchio-system` 0.6, `solana-address` 2.6, and the law's core, `embertide-futarchy` with no feature (no allocation, no std). The lock file was made offline on the new Mac. |
| a Board | 115 bytes, rent 1,691,280 lamports |
| a Balance | 59 bytes, rent 1,301,520 lamports |
| a Road | 419 bytes, rent 3,807,120 lamports |
| a Position | 143 bytes, rent 1,886,160 lamports, given back to its owner at the settle |
| compute units: `init_board` | 10,652 |
| `credit` | 6,551 creating the balance, 3,925 to one that stands |
| `lay` | 12,601 |
| `deposit` | 10,542 |
| `trade` | 8,706 on a road up to date; 111,742 carrying the most it may (66 boundaries) first |
| `carry` | 4,054 with nothing to carry; 107,090 at 66 boundaries; the dearest read, 113,563 (the highest opening price the law allows) |
| `decide` | 53,687 |
| `receipts` | 6,881 |
| `settle` | 11,819 |
| `collect` | 4,358 |
| a boundary of the grid, both pools | 1,542 compute units; `MOST`, the most one instruction carries, is 66 by that measure (carry under 120,000, trade under 180,000) |
| a window of three days at a step a minute, nobody trading | 65 carries before its decision |
| the tests (the new Mac, nice 19) | 20 in five files, green, no warning; the sweep that holds the program to the fold takes 24 s |
| on a chain | nowhere. Mollusk only. The program's id is a placeholder nobody holds a key for. |

The lane's diff under `poleis/`: 15 files, 6,185 lines in (3,214 of them the lock file); this receipt is the sixteenth. It also carries the
commits of `lane/the-average-follows-the-price-that-stood`, merged in so that the program is built on the law as it now stands; that lane landed
first. Nothing was run on the serving Mac.

## What it is

- **A board a polis.** A board is a market place named by a number, as a Poleis config is. Whoever opens board N is its authority. Nothing is
  global.
- **Three roles, each a key for now.** The authority lays roads and names the other two. The crediter adds play $WARM to balances (it stands in
  for his open call of Nov 19, who sends the yield). The judge says the receipts; a road keeps the judge it was laid with.
- **A balance** an owner on a board: what is free, and what stands in roads. **A road**: its terms, its two pools, its verdict. **A position**:
  one owner's two purses in one road, opened by a deposit out of its balance.
- **Eleven instructions:** open a board, name its roles, credit, lay a road, deposit, trade, carry, decide, say the receipts, settle, collect.
  Anyone may carry, decide, settle and collect.

**The law is not the program's.** Every number of the market is moved by one whole call of the law's core (`carry`, `stamp_check`, `trade_on`,
`decide_at`, `happened`, `settle_at`, `pool_at`, `deposit_check`, `purse_of`, `back_of`): the same bytes the fold on a Nest runs. The program
keeps the accounts, the checks and the clock. The law's core built for the chain with no change.

**A road never makes $WARM.** The house, who lays a road, stakes its pool from its own balance: the pool is a purse of deposit
`pool_quote + pool_base`. A settle gives back at most what is left in the road. `collect` gives the house exactly what is left, once the
window has closed and no position stands. So over a board, the balances and what is in the roads are what the crediter credited, to the micro,
and that total is on the board itself (`credited`). The house's result on a road is a number in an account for the first time.

**The clock is the slot, as Poleis's is.** A road's times are slots; the law is handed each slot times the chain's nominal 400 ms. A proposal's
window and its road's window are one clock's, and the slot is the finest instant a chain orders trades by.

**No span of a road is longer than the law's longest window**, 366 days: not the wait for its opening, not its window, not its judge's time.

**A road whose judge has not spoken by its deadline is void:** every position gets its deposit back and the house its stake.

**The bounded carry.** A road nobody touched is carried toward the clock by anyone, at most 66 boundaries an instruction. A trade or a decision
on a road further behind is refused `RoadBehind` and changes nothing.

**The public face.** A reader checks the owner program, the discriminator and the version, and may then trust every offset of that version.
The first 43 bytes of a Road (discriminator, version, board, road id, verdict) never move.

## How it was built

Three briefs of this seat's to one Opus builder, each followed by the seat's read of every line and its own run.

1. **The first build** was faithful to the law and to the brief, and its builder reported five holes in the brief. Reading it, this seat found a
   sixth (the clock).
   - *The pool was nobody's money, so a road could make $WARM* (the builder's): whoever lays roads, with a judge it names, could mint. Closed:
     the house stakes the pool.
   - *A road could hold a deposit for ever* (the builder's): a judge's deadline at the end of time. Closed: the spans.
   - *A boundary of the grid could be pushed for almost nothing* (the builder's). That was the law's, and it was real: on the law as it stood a
     trader in the pool one second in sixty walked a price from 1.0 to 3.61 and passed its road. Closed in the law, in its own lane
     (`embertide-futarchy/THE-AVERAGE-FOLLOWS-THE-PRICE-THAT-STOOD-LANDING.md`).
   - *A board's number can be taken by whoever opens it first* (the builder's). Not closed here: it is for the lane that lets Poleis read a
     verdict, which must bind to a program and a board the polis's own act names.
   - *Rent that stays* (the builder's): boards, balances and roads have no close. Said, not built.
   - *The clock* (this seat's): the first build read the chain's clock in seconds, where Poleis counts slots. Closed.
2. **The second build** made those three changes and held the book after every instruction of every test.
3. **The third build** laid the law's new field in the Road, measured a boundary again (1,837 compute units before, 1,542 now; `MOST` 60, then
   66), and put the book's total on the board.

## The tests (the new Mac, nice 19, at the lane's tip)

`cargo test --offline --locked --no-fail-fast`: **20 passed**, no warning.

- **`tests/roads.rs`, one road from the first credit to the last collect**, held against the fold on the same trades. Three owners deposit 500,
  1,200 and 3,000 $WARM; ten trades in both worlds over four hours, with carries; the road passes; on receipts of 0.6 they get back 528.28,
  1,281.56 and 2,843.39. The house staked 2,000 and collected 2,046.765714, two micro from what the law states. Each owner's balance is what it
  was credited, less its deposit, plus exactly what the fold says comes back.
- **`tests/parity.rs`, the program is the fold, trade by trade.** 14 roads of random terms, 448 trades sent (310 taken, 138 refused by the law
  for the fold's own reason, all four of its refusals met), 1,347 carries. After every instruction the road's two pools and every position's
  two purses are the fold's to the field: 1,795 checks, 386 of them with the road itself the fold's road and no call in between. The book
  closed after every instruction: 125,470,597.74 $WARM credited over the sweep; the largest gap between what a house collected and what the law
  states, 1 micro. 14 positions that never traded settled at exactly their deposit.
- **`tests/refusals.rs`, every refusal, fifteen families:** for each instruction a missing signature, a stranger's, an account that is not its
  address, of another program, of another kind, of another version; and each rule: a board or a road twice, a road in the past or past the
  spans, terms the law refuses, a pool over the house's balance, a deposit twice, over the balance, after the close, too small; a trade before
  the open, at the close, on a road behind; a decision early or twice; receipts by a stranger, before the verdict, of the other world, over
  one, twice, late; a settle early, twice, into another's balance; the void road; a judged road settled long after its judge's deadline
  (by the receipts, never as a void road's); a collect early, with a position standing, twice, into another's balance. Each asserts the exact refusal, that no account changed, and the book.
- **The cap**, by a road's bytes written by hand so that less is left than is owed: the position gets what is left, the next gets nothing, the
  house collects nothing, and the road gives back no more than entered it.
- **`tests/cost.rs`**: the table above, printed and bounded.

**Mutants: 69 deliberate faults in the program, each caught.** One a check: a stranger lays, credits, trades, judges or names the roles; a road in the past, past the spans, with another judge or another house; a stake or a deposit taken from another's balance, another board's, over the balance, or not taken at all; a deposit after the close; a trade on a road behind, with no stamp asked, on the purse of the other world; a road decided twice or behind; the verdict or the judge's world swapped; the judge early, twice or late; a settle into another's balance, as void too soon, uncapped, uncounted, or leaving the position open; a collect by anyone's balance, twice, early or with a position standing; an account of another program, at another address or of another version read as this one's; nobody having to sign; the law handed a step in slots; a branch's held sum not kept. 67 are caught by a named test; one (a carry that carries nothing) makes the tests loop for ever and is ended after 150 s. **One would have survived:** a judged road read as void once its judge's deadline has passed. No test settled a position that late; this review added it (`a_judged_road_settles_by_its_receipts_also_after_the_judges_deadline`) and the fault is caught by it.

## What is not done, and what is not proven

- **It is on no chain.** It ran on Mollusk only, never on a validator. Its id is a placeholder; a deploy is his ceremony.
- **Play counts only.** No token moves: $WARM has no mint. A balance is a number the crediter wrote.
- **No fee yet.** The design names a fee on every trade as what the house keeps. Neither the law nor this program has it. It is the next piece.
- **The three roles are keys, and the authority is a board's root.** It names the crediter and the judge. What the book gives is that every
  micro on a board traces to a credit, in the open, and that no road makes any.
- **A judge is a key and its first word is final.** With a trader it can move what is in a road, the house's stake and the others' deposits,
  to that trader. The month's root sealed by both chambers is the design, and it is not built.
- **A road nobody else trades is decided by whoever trades it, for nothing** (the law's receipt, "What the law cannot do alone"). How much of a
  board's size must stand in a road for its verdict to count is for the lane that reads a verdict at the seal, and his to rule. The board's
  `credited` and a road's `deposits` are the two numbers that lane will need.
- **Poleis does not read a verdict yet.** That is the next lane after the fee.
- **A slot is not 400 ms of the wall.** Real slots run longer, so a window lasts longer than its nominal time. Every trade of one slot shares one
  stamp, so the last trade of a slot owns that slot.
- **A collect waits for every position.** Anyone may settle any position once the judge spoke or its deadline passed, at one transaction each.
- **Rent stays** in boards, balances and roads: none has a close.
- **The build tool writes a deploy keypair** beside the program at every build and has no switch to stop it. The tests' harness removes it,
  unread, at once.
- **Read by this seat and its builder only.** No one else has audited it.
