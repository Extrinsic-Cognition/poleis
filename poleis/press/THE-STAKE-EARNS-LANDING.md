# THE STAKE EARNS (Nov 25 2026, lane/the-stake-earns, link 3 of three)

## His words, UTC

- Nov 25 2026, "Yes, build them" to "Build the three missing links now?" (the brief of the session that started this lane; this seat did not
  read that transcript). Link 3 of the road in ec-workspace `docs/the-launch-the-stake-that-funds-snowtracers-and-his-salary-hypertrace-2026-11-25.md`
  (its section 3): "The stake earns. Not built."
- Nov 20 20:18: "we are supposed to delegate automatically like marinade does but with our own rules.. we do the sanctum way for liquid staking
  across various snowtracers" (ec-workspace `docs/marinade-native-staking-hypertrace-2026-11-20.md`, "His second correction").
- Nov 20 19:56: "the staking of sol to fund snowtracers is the core of it... you dont remember?"
- Nov 25 13:11: "i need some salary and that i can only think would come from the snowtracing staking sol there"
- The staker role belongs to "a Poleis address" (his Oct 14 call, as the Nov 20 doc reads it), and the rule is "delay first" (Oct 14, the same
  doc's words).

## The shape, and the one fact that decided it

Poleis signs as the polis's hand (`agent_pda(0)`, her pool's manager and staker since act 5) in one place only: the execute of a sealed
`ACT_RAW` (`poleis/program/poleis_treasury/src/ix/proposal.rs`, the `invoke_signed` with the seeds `agent`, the unit). No other instruction of
the program gives the hand's signature, so no third way keeps the staker a PDA and runs each epoch without a vote, and no program change was made.

- **(a) the hand stays the staker**: every placement its own act. Each act is a seat owner's proposal, unit 1's convene, its agents' envelopes
  landed, two seals, a mandate and the execute (seven transactions of the crank's and the proposer's) plus the floor's own votes, one transaction
  a seat owner, inside a window of 216,000 slots (a day) on mainnet; and the program takes one transient move a validator an epoch. A vote a
  day for a move an epoch, for every validator: it cannot run every epoch.
- **(b), built: one act names the engine's key the pool's staker.** The ninth act, SetStaker by the hand, naming the params' `engine.staker`,
  as act 6 names the $WARM mine's miner. The stake pool program bounds a staker: it adds and removes validators and moves stake between the
  reserve and them, and never withdraws a lamport. The hand stays the manager and renames the staker by another act.

It is **a later act**, not one of the launch's set: appending it to `acts::acts` would move what was rehearsed (three tests count eight acts,
and the launch rehearsal's stand-ins would need its key). `engine::later` adds it after the eight wherever the params name `engine.staker`:
`press vote --act 9`, `press chair` and `press crank` carry it like the others, after the eight have executed.

## Cost first

| | before (main 4017a9e15b) | this lane |
| --- | --- | --- |
| her pool's stake | every lamport in the reserve, earning 0 | placed on the rule's validators each epoch, the reserve keeping its share |
| code | | Rust: `src/engine.rs` (new, 767 lines), `tests/engine.rs` (new, 9 tests), small edits to shared files (the report lists them), `poleis/door`'s update and its page doors, one test in `poleis_treasury/tests/council.rs`; sh: the rehearsal (73 lines); 0 lines of JavaScript, no new dependency, `--offline --locked` |
| the ninth act | | its payload 137 bytes, its execute's transaction 519 bytes, a seat owner's proposal of it 709 bytes (a packet holds 1,232) |
| a build | | the shared target `~/c4-scratch/cargo-target.noindex`: the press and its door test compiled in 17 s from cached dependencies |
| the rehearsal | | 43 s wall (the validator up in 1 s, the live test 36 s), nice 19 on the new Mac; the whole press suite run once: 134 passed |
| a run of the crank | | one transaction a move, 5,000 lamports each: the first run 10,000 (the add, the move); each epoch after, the update, 5,000 for up to five validators; a second run in an epoch, 0 |
| what moves out of the reserve | | an add: a stake account's rent and a delegation's minimum (2,282,880 + 1,000,000,000 on this validator; 1,000,000 on mainnet while the stake program's minimum is one lamport), part of the stake; a move: a transient's rent, 2,282,880, back in the stake when it merges |
| what the engine's key pays | | its fees, and the transient's rent (2,282,880) when a staker emptied the reserve to its own rent, a gift to the pool counted as yield; 2,317,880 lamports in the whole rehearsal |

## What stands

`poleis/press/src/engine.rs`:

- **The rule** (`Rule::of`, the params' `engine`): the validators (`{vote, weight}`), their weights above 0 and none twice, and `reserveBp`, the
  share of the pool kept in the reserve so a staker leaves at once. Refused with its words: an empty list, a weight 0, a validator twice, a share
  past 10,000, a word that is not an address, a placeholder (his, not named yet: nothing is placed). The rehearsal names its validator's own vote
  account by the word `THIS VALIDATOR`, resolved by `getVoteAccounts` only on a chain of this machine with one validator; on mainnet the word is
  refused. No validator's address is in the code.
- **The planner** (`next`, pure): the pool brought to the epoch first (UpdateValidatorListBalance with every validator's stake account and its
  transient, five validators a transaction, then UpdateStakePoolBalance, the fee minted); nothing while the epoch's rewards are paid; the rule's
  validators the list does not hold added (AddValidatorToPool, the reserve funding each stake account) while the list has room; the reserve
  above its kept share placed by weight (IncreaseValidatorStake), each move at least a delegation's minimum and its transient's rent paid out of
  what is placed, never out of the kept share; a validator whose transient stands is not moved (one move a validator an epoch); a reserve below
  its share brings stake back (DecreaseValidatorStakeWithReserve, the variant the program's v2 keeps; the plain decrease it calls deprecated)
  from the validators that hold the most above their floor. A staker who emptied the reserve to its own rent leaves it unable to fund the
  transient's rent; the engine's key sends that rent in first (found on the rehearsal's validator: the decrease was refused without it).
- **`press engine --params P --rpc URL --pool A [--key F]`**: one run, the moves sent one transaction each by the engine's key (its fee payer and
  the pool's staker), until nothing is due; it says in numbers what it sent and what stands after (each validator's active and transient
  lamports, the reserve above its rent, the kept share, the pool's total). Without `--key` it says what it would send and sends nothing. A key
  that is not the pool's staker sends nothing and says the ninth act has not executed. `press engine --make-key F` makes the engine's key once
  (mode 600, never over a file).
- **`press engine-act --dir D --params P --rpc URL`**: the ninth act's set, `9-the-engine.json`, beside the eight: the act, its payload and hash,
  unit 1's agents' envelopes, the staker it names and the rule.
- `pool.rs`: once the ninth act has executed, the pool is held as root 0's with its staker the params' engine key (the manager still the hand).

## The proof

`sh poleis/ceremony/the-engine-rehearsal.sh <the stake pool program's .so> [certificate]` on the new Mac: solana-test-validator 4.2.0 (Agave),
32 slots an epoch, the program's file `~/c4-scratch/door/spl_stake_pool.so` (the one the door's rehearsal dumped once; mainnet was not read
again) held to the launch's fingerprint before anything starts; the validator ended and its ledger removed by the script's trap. Its
certificate: `poleis/ceremony/the-engine-rehearsal-2026-11-25.json` (the second run's, with the door's half; the numbers below are the first
run's, 43 s; the second's differ by a few lamports in the reward's wake). `cargo test --test engine`: 8 passed then, 9 now, the live half among them.

- The program on the validator: 328,249 executable bytes, sha256 `d18a2cdc0e0a52f55cb363cf517a4e72e9fca4fd6950c12897ee13c2385c52ac`, mainnet's.
- A pool of the launch's shape (epoch fee 1/1, no deposit authority) born by a stand-in manager; a staker's 200 SOL in, 200,000,000,000
  receipts back.
- The SetStaker the ninth act carries (the same bytes and accounts as the act's payload, `tests/engine.rs` holds them equal), signed by the
  stand-in manager in the hand's place: the pool's staker read back as the engine's key.
- The rule from `params.rehearsal.json`: the validator's own vote account, weight 1, a tenth kept. The stake program's minimum delegation on
  this validator: 1,000,000,000 lamports.
- The first run, epoch 0: the validator added (1,002,282,880 lamports out of the reserve), 178,995,434,240 placed on it; after it the reserve
  held 20,000,000,000 above its rent against a kept share of 20,000,000,000. Fees 10,000.
- The second run in the same epoch: nothing sent ("a move of 178997717120 lamports is in flight this epoch").
- Epoch 1: the pool brought to it, the transient merged, 179,997,717,120 active. Epoch 2: the pool's total grew from 200,000,000,000 to
  200,001,395,245, **a reward of 1,395,245 lamports in one 32-slot epoch**, and **1,395,245 of the pool's tokens minted to the fee account**
  (the stand-in manager's receipts account in root 0's vault's place); the supply grew by exactly that, and the staker's 200,000,000,000
  receipts are worth 200,000,000,000 lamports: the yield is the fee account's, the stake stays at par.
- The way out: at once, 20,002,282,880 lamports (all the reserve held above its rent) back for 20,002,282,880 receipts, at par. The crank then
  found the reserve below its share: it sent the transient's rent in (2,282,880, the engine's key paying) and brought back 17,999,911,237. An
  epoch on, the reserve held 18,002,333,643 above its rent, and the staker left with that much again, at par. 161,995,383,477 receipts stayed in.
- The engine's key spent 2,317,880 lamports in the whole run (seven transactions and the one rent).

The second run of the same rehearsal (the certificate above is this one; 62 s wall, the live test 54 s) adds the door's half below; its engine
numbers match the first (a reward of 1,395,245 lamports in one 32-slot epoch, the way out at par twice).

## The follow-ups of Nov 25 (from the seat that briefed this lane)

**1. The engine's blanks hold only the engine.** The params' own law (`params.rs`, `LATER = ["engine"]`): `placeholders()` and
`sendable()` see only the launch's values, `later_placeholders("engine")` the engine's. While one of the engine's three stands:
`press engine` and `press engine-act` refuse before anything is read, naming each ("the engine's rule is not named yet (engine.reserveBp,
engine.staker, engine.validators)"); the ninth act is not among the acts, and `press crank` and `press chair` say so in that line and go on
with the eight; `press vote --act 9` refuses with it. `tests/params.rs`: mainnet's params with the launch's seven given and only the engine's
three left pass `sendable()` and every check of the launch; the engine's steps refuse naming each; with all three named the ninth act stands
after the eight.

**2. The door brings a pool with validators to the epoch.** Once a validator stands, an update naming none is refused by the program
("Identify validator stake accounts with old balances and update them", its error 16, read on the rehearsal's validator), so link 2 would
have broken the day link 3 goes live. `poleis/door`: `pairs` reads the validator list's bytes (each validator's stake account and transient);
`update(keys, pairs)` names them five a list instruction; `stake` and `unstake` take the pairs when the pool is behind. The page's WASM doors
take the list at a new slot (`LIST_AT`, room for 100 validators) and a trailing `list_len`; the page's glue as it stands passes none (read as
0), so its update still names none: **the glue's write of the list is owed** (no JavaScript was changed here). The engine's update is now the
door's. Read back, the engine not run for two epochs:

- the pool behind the epoch with one validator; the door's old update simulated and refused (error 16);
- a new staker's 5 SOL by the door alone: 4 instructions, 707 bytes, 54,711 units, **fee 5,000** (one signature: the extra instructions cost
  a staker no lamport at no priority fee), 5,000,000,000 receipts; the staker paid 5,002,044,280 (the receipts account's rent 2,039,280 and
  the fee); the pool brought to the epoch by it;
- an epoch on, the way out by the door alone: 3 instructions, 47,190 units, fee 5,000, 5,000,000,000 lamports back for 5,000,000,000
  receipts, at par.
- **How many fit:** a stake into a pool behind the epoch is one transaction up to **8 validators** (707 bytes with 1, 1,185 with 8), the way
  out up to 9 (635 bytes with 1); past that the update needs a transaction of its own in front (anyone may send it), not built in the door.

**3. The ninth act through Poleis.** The smallest harness that founds root 0 on Poleis's own build and carries a raw act of the hand through
the vote to its execute is the program's Mollusk world (`poleis/program/poleis_treasury/tests/council.rs`, `estate` and `carry`, the harness of
`the_raw_act_is_the_agents_hand_and_never_the_treasurys`); the full launch rehearsal is the only one with a validator, and this Mac's load read
57 to 70 on 18 cores at 14:00 UTC, so it was neither wired nor run. New there (a test, no program changed):
`the_ninth_act_names_her_pools_staker_by_the_hands_sealed_raw_act`, the stake pool program loaded from mainnet's bytes
(`ENGINE_STAKE_POOL_SO`, the door's dump, 1,080,464 bytes), her pool as act 5 leaves it. Run once, 1 passed (34.7 s with the build):

- a key's own SetStaker refused, the pool unmoved;
- the ninth act proposed, voted by root 0, sealed, executed: the stake pool's SetStaker invoked by Poleis at depth 2 (3,545 units of the
  execute's 18,014), the pool's staker read back as the engine's key, its manager still the hand; a second execute refused.

The engine's run by that key is the validator's (above), not in the same world: Mollusk holds no RPC. `press need` now counts the ninth act's
crank (unit 1's convened session of it and six transactions) where the params name it whole and its proposal does not stand, and the
certificate reads it after the eight.

## Not built, and whose

- **Mainnet's rule is his**: `engine.staker` (the key `press engine --make-key` makes), `engine.validators` and `engine.reserveBp`; they hold
  only the engine. His rule "delay first" needs each validator's measured delay from the Nests; nothing measures it.
- **The ninth act on a validator** inside the launch rehearsal (its stand-ins naming `engine.staker`, the checks of `council.rs`'s live test
  voting it, the agents' minds voting it, the engine's key run after): a seat's build, and a run when the Mac has room.
- **The page's glue** writing her list at `LIST_AT` and passing its length (a JavaScript change, paid by lines taken out), and the module rebuilt
  (`poleis/door/build.sh`; not rebuilt here, since another builder changes the crate in parallel).
- **A stake behind the epoch with more than 8 validators**: the update in a transaction of its own.
- **Who runs the crank each epoch**: no job, beat or door runs `press engine`; a seat's build.
- **Taking stake off a validator above its weight** and **RemoveValidatorFromPool** for a validator that left the rule: not built; such stake
  stays where it is, and the run says so.
- **A way out beyond the reserve**: a staker leaves at once only up to the reserve's share (a tenth in the rehearsal); beyond it the way out waits
  an epoch for the crank to bring stake back. WithdrawStake (a stake account handed out) is not offered.
- **The fee read by `poleis-stake`'s reader**: the fee account was read from the chain directly; linking that crate would change the press's
  Cargo.lock.
- `press need` does not count the engine's own fees per epoch (its key pays them, never the deployer).

## The gate

`poleis/press/tests/engine.rs`, the pure half, with the press's crate at the ship (The crates gate the ship); `poleis/press/tests/params.rs`
(the engine's blanks hold only the engine); `poleis/door/tests/page.rs` (the update naming the list's validators); and
`poleis/program/poleis_treasury/tests/council.rs`'s ninth-act test (it says so and passes without `ENGINE_STAKE_POOL_SO`). The live half runs under `poleis/ceremony/the-engine-rehearsal.sh` (it says so and passes without
`ENGINE_VALIDATOR`). No gate yet on the live engine: the measure it waits for is an epoch's fee minted into root 0's vault of her pool's mint on
mainnet, read by `poleis-stake`.
