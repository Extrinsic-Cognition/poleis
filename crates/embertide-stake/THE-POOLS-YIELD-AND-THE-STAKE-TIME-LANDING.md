# THE POOL'S YIELD AND THE STAKE-TIME: what a pool's yield was in an epoch, read from its own mints, and what each holder held from one epoch to the next (Nov 20 2026, lane/the-pools-yield-and-the-stake-time, claude-dc5c1cd7)

## His words (UTC)

- Nov 20 2026 20:25: "ok, so wrapping things up? i need to stay true to what i say in the introduction my friend... i say solana is not smart etc
  etc... staking for collective self-improvement etc..."
- Then, on the seat's question "Should I build pieces 3 and 4 in Rust: the pool-yield reader and the stake-time count?": **"Yes, build both"** (20:31:48).
- Before it, 20:18: "we are supposed to delegate automatically like marinade does but with our own rules.. for what is worth, we do the sanctum way
  for liquid staking across various snowtracers".
- Oct 29 18:57: "$POOL is unlocked all at once 5 years down the line, until then, stakers just compete for who accumulates the most, and the SOL
  yielded goes into funding ai credits..."

The lineage, whole: ec-workspace `docs/marinade-native-staking-hypertrace-2026-11-20.md` (its last section, "The wrap", lists six pieces; these are
pieces 3 and 4). Her two lines this makes countable (`embertide-app/ui/intro/dyson-society/app/dl-athena.mjs`, `STORY`): "Its yield becomes my
swarm's fuel." and "Give it your SOL's light, hold $POOL."

## Cost first

| | before | measured |
| --- | --- | --- |
| A pool's yield in an epoch | not readable by the house: the Oct 29 crank reads linked native stakes (`getInflationReward`), never a pool | `read`: 1.82 s wall, 0.03 s of CPU, 3 transactions read, on fpSOL (mainnet, epoch 1092) from the new Mac |
| Who held a pool's tokens, and for how long | nothing counted it | `count`, the first read: 0.70 s wall, 0.09 s of CPU, 515 holders out of 4,684 token accounts, 40 bytes asked of each |
| The crate | none | 5 s to build offline on the new Mac (every dependency already in its cache: the mint ear's four, the same versions; `Cargo.lock` is byte for byte the one that build made); 24 tests in 0.02 s |
| JavaScript added | | 0 lines |
| At rest | | nothing runs; the hand is run by a caller, once an epoch |

## What it is

`embertide-stake`, one crate. The law is pure (`src/lib.rs`); the hand (`src/main.rs`) reads a Solana node the caller names and writes one file,
the book, only with `--write`. It signs nothing, sends nothing, and keeps no node's address.

1. **The pool, read** (`pool_of`). A stake pool account's bytes, as the SPL stake pool program and Sanctum's two forks lay them: the manager, the
   staker (the role that places the stake and can never withdraw), the mint, the manager's fee account, the totals, the epoch fee, and the last
   epoch's totals at the end of a walk over the layout's optional fields.
2. **The epoch's yield** (`fee_mints`, `epoch_fee`). The pool tokens the program minted into the fee account inside its own
   UpdateStakePoolBalance, in the slots of that epoch. A failed transaction, a transfer into the fee account (a withdrawal's fee), a mint under
   another instruction (a deposit's fee) and the owner's own moves are not the yield.
3. **The book** (`count`). One more read of the holders laid on the last one:
   - **Held is the lesser of the two reads.** A holder is counted for what stood at both. Arriving before a read and leaving after it earns nothing.
   - **Stake-time** is the lamports held, summed over the epochs counted (his Oct 29 "it just multiplies how long u been staking").
   - **Light** is each holder's share of the fee by what they held, rounded down. What rounding leaves is the row's dust, so lights and dust always
     sum to the fee. With nobody holding across, the whole fee is dust.
   - The first read counts nothing. An epoch is counted once. A gap of several epochs is counted as one, for everyone alike.
   - The manager's fee account is the yield, not a stake, and is left out; `--except <owner>` leaves an owner out (the house's own wallet, say).
4. **The refill's row** (`refill_row`). An epoch's yield as the one row the tanks' ledger takes, in the shape the Oct 29 crank writes
   (`pool:<epoch>:<address>` opens its why, so the ledger's own check never writes it twice). The tank and the price of SOL are the caller's words.
   It is said, never written: a refill is his hand.

## Why the mints, not the balance (found while building)

The first design read the fee account's balance at two epochs and took the difference. Mainnet said no. fpSOL is a pool of exactly this shape (a
person's own token, epoch fee 100/100, 184 SOL). In epoch 1092 its program minted 25,198,739 tokens of fee at slot 471,745,493, and its owner
moved 26,913,047 out at slot 471,762,910. The balance fell across the epoch while the yield was positive. So the yield is read where it is made.
Of the 278 pools holding more than 1 SOL on mainnet (read Nov 20, of 1,853 over the three programs), 39 take the whole yield as fee: 35,413 SOL
staked by people who gave up their yield to someone.

## The book is state

THE HOUSE KEEPS STATE, NEVER THE STREAM: the book is one value replaced in place. It holds the last read, the last counted epoch's row, the sums over
every counted epoch and each holder's sums. It keeps no list of epochs (the first cut did; it was taken out before the hand). Its size follows the
number of holders: 33 KB for fpSOL's 515.

## Proof

- **`cargo test --offline --no-fail-fast`, on the new Mac, from a clean archive of the tip: 24 passed, 0 failed.**
  - Four real pools, one of each program and Jito, read against an independent read of the same bytes (Python, the seat's scratch): 12 fields each;
    the walk to the last two fields checked against the totals; a pool with a future fee and a preferred validator set moves the walk by 48 bytes
    and reads the same.
  - fpSOL's fee account over epochs 1091 and 1092, every transaction that touched it, trimmed to the fields the law reads: 26,913,047 and
    25,854,151 tokens. Eight ways a mint must not count, each tried on a real update.
  - The book on invented holders (no stranger's wallet is kept in the tree): the lesser of two reads, the arrival and the leaver, the dust, the
    gap, the rate, an epoch counted once, the book given left unchanged; and 300 invented books of 2 to 6 reads where every lamport of every fee
    is in a light or in the dust and no stake-time grows except by what stood at both reads.
  - The row against the Oct 29 crank's key shape and its rounding of dollars.
- **Live, read-only, mainnet's public node, from the new Mac:**
  - `read` fpSOL epoch 1092: 25,854,151 tokens, 25,862,316 lamports (the same tokens as the fixture, through the hand's own road); epoch 1091:
    26,913,047 tokens.
  - `count` with a scratch book: the first read, 515 holders, nothing counted; written; the same read again refused, "epoch 1092 is already counted
    in this book", exit 2.
  - `row`: `pool:1092:GutG5bcm…`, 25,862,316 lamports, $1.851741826 at $71.6 a SOL (a price typed for the run, not a quote).

**Not proven:** a count across a real epoch boundary. It needs a second read after epoch 1093 begins (about 18 hours after this was written, at 20:4x). The
law of it is proven on invented holders only. The scratch book was removed.

**Not run by this seat:** the lander's whole gate (161 test files) and its walks. Run on the new Mac at the tip, by hand: this crate's tests, and
the five gates a new crate can touch (named in the hand).

## Not built, and whose

- **Her pool's birth** (piece 1) and **the door on the mint page** (piece 2): his key, his deploy. The page's stake still reads Paused.
- **The write into the tanks.** `row` says the row; `space-tanks.mjs` writes it, by his hand. Which tank a pool's yield fills is not ruled for her
  pool: his Oct 29 toggle gives a staker's yield to their own Swarmy or to the ClawRiders, and his Nov 20 "the sanctum way" gives a pool's yield
  to its snowtracer. The row takes the tank as the caller's word and decides neither.
- **$POOL's formula.** The book keeps stake-time and light. Which of them, or what mix, counts toward $POOL is his.
- **The house's own holdings.** Fee tokens the manager moves to another account of its own count as a stake unless the caller excepts that owner.
- **A pool per snowtracer and the board** (piece 5), **the delegation engine** (piece 6): not here.
- **An epoch's history of holders.** The book reads holders once an epoch. Exact stake-time inside an epoch would need every token account's
  history; the lesser-of-two-reads rule is the bound chosen instead, and it is the seat's choice, his to strike.
