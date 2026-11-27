# THE LIGHT PAYS A WISH AND ITS RIDER (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider)

Links 6 and 7 of the launch's road (ec-workspace `docs/the-launch-the-stake-that-funds-snowtracers-and-his-salary-hypertrace-2026-11-25.md`, section 3):
each staker's light reaches the wish they chose, and the well pays its Snowtracer.

## His words, UTC

- Nov 25 2026 13:11: "forget about all that, the only way out is through... we gotta launch swarmbook and the stake sol to fund snowtracers... i need
  some salary and that i can only think would come from the snowtracing staking sol there... hypertrace".
- Nov 25 2026 13:25: "Yes, build them", to "Build the three missing links now?".
- Nov 22 2026 16:57:42: "THE META-SWARM IS GLORIOUS BECAUSE YOU STAKE ALL THERE IN 1 PLACE (the meta-swarm... the unit0... the root of them all...)
  BUT!!!! you can delegate the yield of your stake or let the swarm futarchy handle it for you idk... hypertrace".
- Nov 20 2026 12:13: "a snowtracer is a top tier claw rider".
- Oct 28 2026 15:12: "the mint pays the treasury, a monthly allowance goes to my Keystone 2-of-3, refunds by rule, the rest by proposal".

## Cost first

| | before (main 4017a9e15b) | this lane |
| --- | --- | --- |
| a staker's light reaching a wish | the choice is a memo (`embertide.light/1 wish <unit>`); nothing reads it, and the yield stays in root 0's vault | `press light`: the reader (the stake book's epoch, each staker's newest choice) and the pour, a crank with no vote per epoch |
| a well paying a rider | the mechanism stands (the launch's act 8, root 0's allowance to one key); no wish has one, no code admits a rider | `press rider`: the wish's own decisions, its plan of her token and its allowance to the rider's key; `poleis_door::draw` and `draw_to_sol`, the rider's own signature |
| code | | Rust: `poleis/press/src/light.rs`, `src/rider.rs`, `tests/light.rs` (4 pure tests and the live half), `tests/rider.rs` (4); `poleis/door` +71 lines and 1 test; sh: `poleis/ceremony/the-light-rehearsal.sh`. **0 lines of JavaScript.** No on-chain program touched. No new dependency: the press links `embertide-stake`'s law (already in the tree) with its hand's HTTP behind a default feature, so the press's lock gains one entry and the seal's (which links the press) bs58 and tinyvec, both already in the cache |
| the page's module | 105,697 bytes | 110,446 bytes (`door_draw` added) |
| a build | | `cargo test --offline --locked --test light --test rider`: about 10 s incremental on a shared target, this Mac at load 40 to 100 |
| the rehearsal | | **74 s wall** (the validator up in 2 s, the live test 71.5 s), nice 19, this Mac; nothing on the serving Mac |
| a pour, on the chain | | one transaction a wish an epoch, the crank's one signature: 5,000 lamports. The first pour into a wish also makes its vault of her token: 2,039,280 lamports of rent, the crank paying |
| the road, once a wish | | root 0's allowance account (180 bytes): 2,143,680 lamports of rent; root 0's plan of her token (99 bytes), once: 1,579,920; each a decision of the whole tree |
| a rider | | the wish's plan of her token (1,579,920 once) and the rider's allowance (2,143,680), two decisions of the wish; a draw: 5,000 lamports, and 2,039,280 for the rider's receipts account the first time (theirs back if they close it) |

## The road the pour runs on, and why

The pour moves each wish's share from root 0's vault to `vault_pda(<wish>, mint)`. **It runs on a standing TREASURY_ALLOWANCE of root 0 whose payee is
the wish's unit (HOLDER_UNIT), in her pool's token.** The fact that decided it is in Poleis's `ix::economy::check_dest`: for a unit payee the destination
must be `expect_vault(dest, unit, mint)`, the payee's own vault of the allowance's mint and nothing else, and `pull` lets anyone pull for a unit payee. So
the whole tree votes the road once for each wish, and the pour is a crank with no vote per epoch; every pull is also charged to root 0's plan of the mint
(its CODEX AllowMint: `charge_token_plan` refuses a mint that is not the unit's default without one). The roads not taken, and what they would cost:

- **An allowance to a crank's key (HOLDER_KEY).** For a key payee `dest` must be a token account the key owns, so every pour would land in the crank's
  own account first and need a second move: the crank would hold the money. Refused.
- **An act per epoch** (TREASURY_TOKEN_TRANSFER, kind 4, which does send to any unit's vault): exact to the token, but a vote of the whole tree for every
  wish every epoch (both chambers at both levels at the launch). No crank can carry it alone.
- **TREASURY_TOKEN_BUDGET** (to any address): an act per epoch, and to any address. **TREASURY_BUDGET**, **TREASURY_TRANSFER** and **`endow`**: lamports
  only, never her token.

What the allowance does not hold, **his to rule**: anyone may pull a wish's allowance into that wish's vault, up to its cap a period, whatever its light was.
The token never leaves the tree for a key by this road, but a well can be filled past its light by up to its cap a period. The cap and the period of each
well's road are a figure the tree votes; this lane names none (the rehearsal's are its own).

**The over-pull is said, and netted** (the second pass, the briefing seat's follow-up of Nov 25). Before it plans, `press light` reads every pull on each
wish's road: Poleis's own RCPT_PULL receipts in every transaction that touched the road's allowance account, and the crank's own records. A pull the
crank did not send (its transaction not paid and signed by the crank with its record of that wish) is an **over-pull**, said by name: the wish, the
tokens, the puller, the transaction (`over_pulls`, `over_pulled_tokens` in the run's JSON and the certificate). Each wish's books (`Ledger`) are every
light the crank recorded owed it (the record is now `embertide.pour/1 epoch <e> wish <u> owed <s> tokens <n>`) against every token pulled on its road; the
pour is what is still owed after everything it took, never negative. A wish over its light is said to be over by how much, gets a record and no pull,
and its over-take is counted against its next epochs' light until even. Root 0's vault never pays twice for the same light: after a pour, what the
wish took equals what its light was owed.

**One law for the choice, page and pour** (`poleis_door::light_signed`, on `landed_of`, which reads a landed transaction's wire bytes): a choice counts
only in a transaction the staker signed where the memo program names them. The page's `door_chosen` now takes the transaction's bytes, not a listed
memo, and the pour's `choice_of` is the same call; `light_of_listed` remains a candidate filter only.

## The reader (`poleis/press/src/light.rs`)

- **The epoch's lights** (`lights_of`): her book before one count and after it (`embertide-stake`'s files, its own law), each staker's light the
  difference; refused unless the two books are one count apart and the lights and the book's dust sum to the fee.
- **The choice** (`choice_of`): the newest light memo signed before the first slot of the epoch whose fee is poured (the choice that stood when the yield
  was minted). A memo counts only when the memo program names the staker among its accounts and the staker signed: the door's existing `light_of_listed`
  takes any memo the chain lists for the staker's address, so a stranger who sent the staker one lamport with a memo would have set their light. Closed
  in the second pass: the page and the pour now read one law, `poleis_door::light_signed`.
- **The shares** (`read`): a wish's tokens are `fee_tokens * light / fee_lamports`, rounded down. No choice, the futarchy, root 0 itself, or a unit that
  is no swarm on the chain sends nothing; the reader says how much stays in root 0's vault. The rule for that light is not his yet, and none is invented.
- **The chain** (`agree`): the fee the book counted for the span is held to the pool tokens the pool's program minted into root 0's vault inside its own
  UpdateStakePoolBalance in those epochs, read by the book's own `fee_mints`; when they differ nothing is poured and both numbers are said.
- **Once an epoch** (`plan`, `poured`): each pull rides with the crank's memo `embertide.pour/1 epoch <e> wish <unit> tokens <n>` in the same transaction.
  **The record is on the chain**: a memo counts only in a transaction that landed and whose fee payer and memo signer is the crank's key, so a second
  run reads it and pours nothing for that epoch and wish, and a file lost is nothing lost. The plan pours nothing when the pours would pass the epoch's
  fee or what root 0's vault holds.

## The proof (`poleis/ceremony/the-light-rehearsal-2026-11-25.json`)

`sh poleis/ceremony/the-light-rehearsal.sh <mainnet's stake pool .so>`: solana-test-validator 4.2.0, 32 slots an epoch, mainnet's stake pool program
(sha256 `d18a2cdc…52ac`, held before anything starts), this tree's Poleis loaded upgradeable with a stand-in deployer as its upgrade authority, which
founds root 0 (ONLY THE DEPLOYER FOUNDS ROOT 0). Read back:

- Root 0 and a wish (unit 8405527522246845140) founded the wish walker's way, side by side, each with one pentagon and one hexagon.
- **Her pool born as the launch births it**: its reserve, its mint and root 0's vault made ahead, then root 0's raw act (the pool's Initialize by the
  polis's own hand) proposed, voted, sealed and executed, the pool's account and list made in the execute's own transaction. The door read it off the
  chain as hers: fee 1/1 into root 0's vault.
- Root 0's plan of her token and its road into the wish's well, two decisions; the crank found the road on the chain by its own read.
- Two stakers: 10 SOL with `embertide.light/1 wish 8405527522246845140`, 5 SOL with no choice.
- **The yield**: 1,500,000,000 lamports sent into her reserve before the epoch's update. **The rehearsal's yield is that plain transfer**: a pool's reward
  is the growth of its total, so the program counted it as epoch 5's reward and minted it as fee, 1,500,000,000 tokens into root 0's vault (the engine that
  places her stake on validators is lane/the-stake-earns). The book, read twice by its own hand (`embertide-stake count --write`), counted the same
  1,500,000,000 for epochs 4 to 5, 2 holders, 0 dust; the chain's mints read by the reader: 1,500,000,000. They agree.
- **Each staker's light**: staker 1, 1,000,000,000 lamports, to the wish; staker 2, 500,000,000, no choice.
- **The pour**: 1,000,000,000 tokens reached the wish's well; 500,000,000 stayed in root 0's vault. One transaction.
- **Again**: nothing sent ("poured already: the crank's memo for this epoch stands on the chain"). A book one token over: "the book counts 1500000001
  tokens of fee for epochs 5 to 5, and the chain's mints into root 0's vault in them are 1500000000: nothing is poured".
- **The wish votes its rider**: its plan of her token, then its allowance of 1,000,000,000 tokens a period of 10,000 slots to the rider's key.
- **The rider's draw, to SOL, in one transaction its own wallet signed**: 1,000,000,000 tokens pulled into its receipts and burned for 1,000,000,000
  lamports from her reserve at par. The rider held 1,000,000,000 lamports before and 1,997,955,720 after (the fee 5,000 and its receipts account's
  rent 2,039,280 out). The well holds 0. A second draw past the period's cap: refused by Poleis, its error 54 (AllowanceBreached).
- `cargo test --offline --locked` of the press: every test green; `tests/light.rs` 5, `tests/rider.rs` 4; `poleis/door`'s tests 9 + 4 (+1:
  `the_rider_draws_as_the_law_does`); `embertide-stake` 24; the seal's tests and examples build on its new lock.

## The second pass, proven (Nov 25 2026 ~14:08 UTC)

- `sh poleis/ceremony/the-light-rehearsal.sh`: 76 s wall, exit 0, the certificate rewritten. A stranger's pull of the wish's road before the crank ran:
  300,000,000 tokens, puller `HZAc…qsrC`, its transaction read back and named as an over-pull; the crank then poured 700,000,000 (owed 1,000,000,000);
  the well held 1,000,000,000, root 0's vault 500,000,000 (it paid the wish's light once); the pour again sent nothing; the rider's draw as before.
- `cargo test --test light`: 7 passed (new: `a_strangers_pull_before_the_pour_is_netted`, `a_wish_over_its_light_is_evened_by_its_next_epochs`).
- `poleis/door`: 10 + 4 passed (new: `a_light_is_the_stakers_own_signed_memo`: the stranger's memo on a transfer refused, a memo naming the staker unsigned
  refused, cut bytes refused, the page's door the same).
- `sh poleis/ceremony/the-door-rehearsal.sh` with the browser half (Chrome for Testing, headless): exit 0, 33 s; the wish's page, reloaded, read the
  staker's own signed light through the new `door_chosen` ("Your 25 SOL stays in her. Change it any epoch."). Its proof `tests/page.rs` had a stale
  expectation from main (the page's "+ Start a new wish" now links `./index.html#wish-card`, main ab4c535b21); corrected.
- The page's glue (`her-door.mjs`): two lines changed, none added (net 0 JavaScript); the cache key `?v=light-1` became `?v=light-2` on `polis.html` and
  `after-the-mint.html` (one line each, changed). The module: 110,912 bytes, sha256 `9542e1beba7f49ae544385f0b8309faabf19e0cb58e021f35102cafdf35e5b9d`.

## Not built, and whose

- **The cap the whole tree votes for a wish's road is the only bound the chain itself enforces.** Anyone may pull a wish's road into that wish's vault up to
  that cap a period, whatever its light; the crank only sees it, says it and nets it after the fact. A tighter road would be a new instruction whose pull
  checks, on the chain: that the amount does not pass what a light record owes the wish (a record of the epoch's share per wish, written by a verifiable
  reader or the whole tree), that the epoch was not already paid (a per-epoch, per-wish flag or counter in an account), and that the sum over wishes does
  not pass the fee the pool minted that epoch. His to rule; nothing of it is written here.

- **The launch's acts**: root 0's plan of her pool's token and a well's road for each wish are not in `acts.rs`'s list; at the launch they are acts of the
  whole tree like the eight (`press well` writes their sets; carrying them by `press vote` and `press crank` is a lane of the launch). The rehearsal's root
  0 is the wish walker's tree, one pentagon and one hexagon, not the launch's bicameral one; the launch's own vote of an act is proven by its rehearsal.
- **The cap and the period of each well's road, and of each rider's allowance**: his and the tree's. Whether a Snowtracer's pay is cash, credits or both,
  and his own figure: his (the hypertrace's section 6).
- **Undirected light** (no choice, the futarchy, a unit that is no swarm): stays in root 0's vault, said by the reader; the rule is his.
- **Anyone may fill a well up to its road's cap a period** (above): his to rule; a narrower road would need a new instruction (a pull whose amount the
  program derives from a light record), which this lane did not write.
- **The crank's key and its home**: who runs `press light --send` each epoch, and on which machine, is not set; nothing runs at rest.
- **The page**: `door_draw` stands in the module; no page calls it yet (a page lane, in Rust's words, with the glue's one line).
- **The stake earns**: lane/the-stake-earns (link 3). Until it lands, every yield here is the rehearsal's transfer.

## Shared files touched

`poleis/press/Cargo.toml` (+2: the stake book's law), `poleis/press/Cargo.lock` (+9), `poleis/seal/Cargo.lock` (+24), `poleis/press/src/lib.rs` (+2:
`pub mod light; pub mod rider;`), `poleis/press/src/main.rs` (+8: the three steps and `--send` as a flag), `embertide-stake/Cargo.toml` (its hand behind
a default feature, `hand`), `poleis/door/src/lib.rs`, `poleis/door/src/page.rs`, the module `embertide-app/ui/intro/dyson-society/app/wasm/poleis-door.wasm`.

## The gate

At the ship (THE CRATES GATE THE SHIP): `poleis/press` (`tests/light.rs`, `tests/rider.rs`), `poleis/door` (`tests/page.rs`), `embertide-stake`.
No gate yet on the live road: the measure it waits for is `sh poleis/ceremony/the-light-rehearsal.sh <the .so>` ending 0, and on mainnet the lamports
poured into each wish's well by its stakers' light and the lamports a well paid a Snowtracer each epoch, read from the chain (the hypertrace's section 7).
