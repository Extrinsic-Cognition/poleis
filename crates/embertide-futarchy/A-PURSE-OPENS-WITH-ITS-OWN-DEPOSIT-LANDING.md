# A PURSE OPENS WITH ITS OWN DEPOSIT: each staker brings its own size to a road, and a road gives back the deposit and the result, never less than nothing (Nov 20 2026, lane/a-purse-opens-with-its-own-deposit, claude-81203e94)

## His words, UTC

- **Oct 29 18:56:** "futarchy where the time you stayed in the market made your size of trades bigger? … liquidation-free size increase that anyone
  can tap into for futarchy trades and it just multiplies how long u been staking".
- **Oct 29 18:57:** "$POOL is unlocked all at once 5 years down the line, until then, stakers just compete for who accumulates the most, and the SOL
  yielded goes into funding ai credits".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market".
- **Nov 20 14:26:19:** "why u asking build or change? u think u can design better?"

The design and its run: ec-workspace `docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, the section "Nov 20 2026 14:03 UTC", its part "A
staker's size is its balance". This lane is the second piece, off chain, in the law. A proposal of this seat's that he may strike; nothing here
starts a market.

## Cost first

| | before (main 7c3b9c9939) | after (eeeb8a274c) |
|---|---|---|
| the law (`src/law.rs`) | 268 lines | 327 lines; no dependency, as before |
| the Nest's side and the command (`src/nest.rs`, `src/main.rs`) | 349 and 60 lines | 365 and 60 lines |
| the proofs | nine, `unsat` | ten, `unsat`, in 0.21 s (z3 5.1.0, the new Mac) |
| `cargo test --offline --locked` with `FUTARCHY_PROOFS=require` (the new Mac) | 26 passed, 1 ignored | 35 passed, 1 ignored, no warning |
| `cargo build --offline --locked --no-default-features` | exit 0 | exit 0 |
| the ignored test on a scratch Nest | 1 of 1, 51 s | 1 of 1, 51 s |
| a road one way with no roster | | reads, folds, decides, scores and writes as before (the pinned bytes and the text's 24 markets are untouched) |
| a roster | none | rides in the road's own ref: no extra read; one pass to check it, a trader's deposit found by halving |
| at rest | nothing | nothing: no timer, no poll |
| JavaScript | | none added |

The code's diff: 10 files, 534 lines in, 86 out, all inside `embertide-futarchy/`; this receipt is the eleventh. Nothing was run on the serving Mac.

## What was so

Every trader of a road opened with the same amount, `budget`, whoever it was. And the size rule this seat had modelled for the stakers ("the fuel
times the time staked, following count, never under a tenth") had a hole: a staker stood with its whole size in every road, so a month could cost
it several times what it had, and the floor forgave the rest. Two wallets of one staker betting against each other then farmed count: one ends up,
the other is forgiven. Any floor has that hole, because a wallet is free to make.

## What stands now

- **A purse opens with a deposit** (`Terms::purse_of`). On a road one way the deposit is $WARM. On a road both ways it opens in the pool's own
  mix: part as $WARM and the rest as claims, a claim for a $WARM, in the proportion the pool holds them. So $WARM and claims together are the
  deposit, to the micro. At half a $WARM a claim, a deposit of 1000 opens with 333.333333 of $WARM and 666.666667 claims.
- **A road may name its roster** (`Deposit`, `roster_check`, `market_with`): who trades it and what each deposits, in the order of the keys. A key
  on it opens its purse with its own deposit; a key off it cannot trade (`NOT_ON_THE_ROSTER`). A road that names none opens every purse with
  `budget`, the law as it was.
- **The settle marks each trader against its own deposit** (`settle_of`), and **a purse cannot lose more than its deposit**
  (`proofs/settle-deposit.smt2`). So what a road gives back, the deposit and the result (`back_of`), is never under zero: nothing is cut off and
  nothing is forgiven. A purse that loses everything gets back exactly nothing.
- **Two wallets that bet against each other end where they began.** On this code, what one buys the other sells: their two results sum to -1
  or -2 micro whatever the receipts say (0, a third, one $WARM), the dust of the pool's two floors.
- **The old mark keeps the rule too** (`score_of`): on a road both ways it marks a claim at one $WARM at most, whatever the pool's own average
  says, so it cannot show a loss over a deposit either. One way it is the pool's own average, as it was.
- **On the Nest:** a road's optional `"deposits"` (an object from a key to an amount); a count's rows carry `deposit` and `back`; a reading says
  how many keys the roster names and each trader's deposit.
- **The law keeps no balance.** It says what comes back. The pool is nobody's deposit: it is the house's, and what it loses, at most what it
  opened with, is all a road can add to what the traders hold between them.
- **Nothing else moved.** A swap, the grid, the average, the decision, the receipts and the judge's door are the ones that stood.

This changes one thing that landed at 15:10 the same day: a road both ways opened a purse with `budget` of $WARM and its worth in claims on top. It
now opens with `budget` as the whole deposit, in the same mix. No such road existed.

## How it was built

To this seat's brief an Opus builder wrote the first cut (e47f1caa41, b8d45ad361), and reported what looked wrong, as asked. This seat read every
line and changed four things before the hand (eeeb8a274c):

1. **The old mark could show a loss over a deposit** (found by the builder): a purse that sold its claims was marked against claims at the pool's
   own average, which can stand over one $WARM. Capped at one $WARM on a road both ways.
2. **A roster was read in a square of its length**, on every reading: each key against every key before it, and each trader looked up line by
   line. A roster is now its lines in the order of their keys: one pass, and a key found by halving.
3. **An amount turned into a signed one could wrap** on a roster the law refuses. It stops at what the type holds.
4. The even-mix proof's comment named a function that is gone.

## The tests

35 on the new Mac at eeeb8a274c: 7 new in `tests/roster.rs`, 1 in `tests/settle.rs`, 1 in `tests/nest_words.rs`.

- **The roster** (`tests/roster.rs`, 7): each purse opens with its own deposit in the pool's own mix; a key off the roster cannot trade and the
  market is the market without it; **a whale of 9000 lifts a price from 0.5 to 8.0, two sellers of 4500 each bring it to 0.047 and the road
  fails, and with sellers of 450 each it stays at 1.65 and passes**; the settle marks each trader against its own deposit; a sweep of 200
  rostered roads (93 both ways, 28 on receipts of 0, 1400 traders) where nobody loses more than its deposit and 14 traders who bet their whole
  deposit on a road one way got back exactly 0, not a micro more; the two wallets; a roster the law cannot carry is refused and panics nothing;
  the fold finds a key among a thousand by halving.
- **The settle** (`tests/settle.rs`, 10): the tests of the receipts, read again with the new purse; the mix is the deposit to the micro on 20,000
  pools; on a road both ways whose average ends over five $WARM, a seller is marked -500 on a deposit of 2000 where the uncapped mark would show a
  loss of more than two deposits, and one way the mark is still the pool's own average.
- **The Nest's words** (`tests/nest_words.rs`, 8): a road with deposits reads back with its roster in order; a key that is not a key, an amount
  that is not an unsigned integer, a deposit of 0 and a roster the law refuses are each no road; null or an empty object is no roster.
- **On a scratch Nest** (`tests/on_a_nest.rs`, by name): road 3 names the three wardens' deposits (300, 600 and 1500). The whale's signed trade is
  taken by the Nest and refused by the market. After the count each row says its deposit and what comes back: deposits 2,400,000,000 micro, given
  back 2,361,904,761, the pool 38,095,238.
- **The proofs** (`proofs/*.smt2`, 10 `unsat`, each with premises that hold): the nine that stood, and `settle-deposit`.

**Mutants, each caught** (the new Mac, 16): a purse both ways opens as $WARM alone; the mix is half and half whatever the pool holds; a key on the
roster opens with the budget; a key off it trades; the settle marks a trader against the budget; a road gives a micro back to a purse that lost
everything; a roster out of its order or with a key twice is carried; a deposit of nothing or over a billion is carried; a deposit too small to
hold both is carried; the old mark marks a claim over one $WARM both ways; the old mark is cut one way too; a road is read whatever its roster; a
roster's key need not be a key; a road's ref does not name its deposits; a reading folds a road without its roster; a count's row forgets the
result (the last two caught only on the scratch Nest, 105 s).

## What is not done

- **Nothing keeps a balance, so a road's deposits are whoever lays it.** The road's ref is laid once and names every key's deposit, the layer's
  own included. Nothing ties a deposit to what a staker holds or to the staker's own word. The builder said it plainly: a layer can give itself a
  whale's deposit and everyone else dust. Off chain this lane cannot close it; on a chain a deposit instruction takes it out of the staker's own
  balance, which is the next piece. **No market should run on this law with anything at stake before that.**
- **Nothing splits a balance over a month's roads**, and nothing adds a stake's fuel to one. The run that tested the rule did both by hand.
- **Who puts up the pool.** When the pool loses, the traders together get back more than they deposited, by at most what the pool opened with. It
  is the house's to fund; how deep is a number of the engine (the run: between an eighth and one of the size standing in a road).
- **Nothing lays a road and no market runs on the live Nest**, as before.
- **A judge can be wrong and its first word is final**, as before.
- **Not on a chain.** The market's own program, and the contract's seal reading its verdict, are the pieces after. A deploy is his ceremony.
- **The proofs are of the arithmetic, not of the Rust source**, as before.
- **Left as they were:** on a road with a roster `budget` is still checked and written though nobody deposits it; `road_of` still takes the
  default for an amount that is not an integer; a trader is still looked up line by line among a road's traders at every entry (it was so before
  the roster).
