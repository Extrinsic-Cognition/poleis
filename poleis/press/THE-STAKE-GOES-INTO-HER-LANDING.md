# THE STAKE GOES INTO HER (Nov 22 2026, lane/the-stake-goes-into-her, claude-901e44a8)

## His words, UTC

- 16:57:42, over three ways the stake could live on a society: "should be B... you must not overhwelm the user with infos dude... those are fucking
  wishing wells... people wish and delegate... THE META-SWARM IS GLORIOUS BECAUSE YOU STAKE ALL THERE IN 1 PLACE (the meta-swarm... the unit0... the
  root of them all...) BUT!!!! you can delegate the yield of your stake or let the swarm futarchy handle it for you idk... hypertrace"
- 17:02:34: "there is only 1 meta swarm and that is the diving lyssajudron... all the other ones are not meta-swarm, they are swarms..."
- Then "Yes, draw it" (five screens on the live pages: stake in her, send your light to a wish), and "Yes, build it" to "Build the stake door and the
  choice, proven on the practice chain?".

The lineage: ec-workspace `docs/wish-and-delegate-the-meta-swarms-one-stake-and-pool-hypertrace-2026-11-22.md`.

## Cost first

| | before (main 72d83bba46) | this lane |
| --- | --- | --- |
| a way to stake in her | none: the pool is born at the launch (act 5), nothing deposits into it | `door::stake`: the receipts account made if it does not stand, then DepositSol |
| the light's choice | the Oct 29 toggle on the Nest (`pool/toggles`), for a native stake linked to a tank | `door::light`: one memo the staker signs, `embertide.light/1 wish <unit>` or `futarchy`, the newest is theirs |
| the way out | none | `door::unstake`: WithdrawSol from the reserve |
| code | | Rust: `poleis/press/src/door.rs` (the three acts, the pool's head, the epoch's rewards), `tests/door.rs` (six tests); sh: the rehearsal; 0 lines of JavaScript, no new dependency |
| a build on the new Mac | | 2.8 to 5.3 s (`cargo test --offline --locked --test door`, the shared target, every dependency cached) |
| the rehearsal | | 21 s wall: the validator up in 1 s, the live test 16.4 s, at nice 19 on the new Mac; nothing on the serving Mac |
| mainnet read | | once: the stake pool program's bytes (`solana program dump`, 1,080,464 bytes, 328,249 of them executable), kept in the new Mac's scratch for every later run |
| what a staker pays | | the stake: 0.00203928 SOL for the receipts account (rent, theirs back if they close it) and 0.000005 fee; a light: 0.000005; the way out: 0.000005 |

## What stands

`poleis/press/src/door.rs`: the door's three acts on the meta-swarm's pool, the one `pool.rs` births at the launch, each a list of instructions the
staker's own wallet signs; the house signs none of them. The stake pool program's own instructions by their indices and account orders
(UpdateValidatorListBalance 6, UpdateStakePoolBalance 7, DepositSol 14, WithdrawSol 16), the associated token program's CreateIdempotent and the memo
program. A pool behind the epoch is brought to it in front of the stake or the way out (`update`); while the epoch's rewards are being paid nothing
can bring it there, and the door waits (`rewards_paying`, the EpochRewards sysvar's `active`).

## The proof

`poleis/ceremony/the-door-rehearsal.sh <the program's .so> [certificate]` on the new Mac, solana-test-validator 4.2.0 (Agave), 32 slots an epoch,
the program's file held to the launch's fingerprint before anything starts. Its certificate: `poleis/ceremony/the-door-rehearsal-2026-11-22.json`.
`cargo test --test door`: 6 passed.

- The program on the validator, read from its program data as the launch reads it: 328,249 bytes, sha256
  `d18a2cdc0e0a52f55cb363cf517a4e72e9fca4fd6950c12897ee13c2385c52ac`, mainnet's.
- A pool of the launch's shape (epoch fee 1/1, no deposit authority) born by a stand-in manager.
- The stake: 25 SOL in, 25,000,000,000 receipts back, worth 25 SOL; the staker paid 25.002044280 SOL (the receipts account's rent and the fee).
- The light: `embertide.light/1 wish 5`, then `embertide.light/1 futarchy`; the newest memo among the staker's signatures read back: Futarchy.
- An epoch turned: the rewards were being paid at the turn, and the door waited 538 ms; a stake without the door's update was refused by the program
  ("First update old validator stake account balances and then pool stake balance", its error 17); the door's own stake, with the update in front,
  landed: the pool at epoch 1, 26,000,000,000 receipts, the rate unmoved.
- The way out: every receipt burned, 26,000,000,000 lamports back to the staker exactly, the fee 5,000; the pool after: no supply, no lamports.

Found while proving it: on this validator the program refused the update in the epoch's first moments with its error 43, "Epoch reward distribution
is currently in progress, stakes are still being updated". Mainnet pays its rewards in parts over an epoch's first blocks, so a stake or a way out
sent then waits; the wait on mainnet is not measured here.

## Not built, and whose

- **No light yet.** The pool holds no validator, so every staked lamport sits in the reserve and earns nothing: the light is 0 until a validator is
  added by the seal and the stake placed on it (the engine, piece 6 of the staking doc; nothing built). The way out is at once for the same reason.
- **The page's doors**: the Pantheon's "Stake SOL in her", the choice, a wish's "Send your light here" (the next lane, drawn on the Canvas:
  `harmony-claude:widget-b2066c0e-cde9-4ea7-8bff-f9a1ae3a7fa4`). The site's wallet road (`dl-direct.js`) signs a transaction today; these instructions
  reach the page as WASM, not as new JavaScript.
- **The reader that pays a wish its light**: `poleis-stake`'s book counts each staker's light; reading each staker's choice from these memos and the
  refill into a wish's well are owed; the refill is his hand.
- **Mainnet**: the pool is born at his sitting; until then the door has no pool to open.
- **The program's bytes** stay pinned to the fingerprint: the rehearsal refuses to start on other bytes, and the launch refuses to birth the pool on them.
