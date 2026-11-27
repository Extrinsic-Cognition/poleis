# THE PRESS IS READY FOR MAINNET: the issuer held to its key, the crediter made like the others, and a sealed upgrade that extends first (Nov 24 2026, lane/the-press-is-ready-for-mainnet, seat claude-3f06fe2b)

The launch press (`poleis/press`) still owed three small pieces before its first mainnet sitting. Each would have stopped the sitting, or
the swarm after it. This lane builds all three.

## The cost, first

| | before (main 0aef6660cf) | after (this lane) |
|---|---|---|
| code | | Rust: the press's sources +279 −23 (`src/upgrade.rs` is new, 152 lines), its tests +363 −2 (`tests/upgrade.rs` is new, 265 lines). Shell: +130 (`poleis/ceremony/the-upgrade-extends-first.sh`). One line of `params.mainnet.json` changed in place, plus the certificate. **JavaScript: 0.** |
| `floor.issuer` | not held to `D/keys/issuer.address` | held to it at `press deploy`, `press found` and `press acts`; refused, with the address, where they differ |
| `market.crediter` | a PLACEHOLDER that no step makes | `press crediter --dir D`: one key file and its address beside his other keys, like the issuer's and the miner's |
| a sealed upgrade to a longer build | refused at its execute, every time (`AccountDataTooSmall`), after the whole tree had passed it | the crank extends the program's data first, in a transaction of its own that it pays for, then sends the execute a slot later |
| the extend on the chain | | **2,520 units**, one signature (5,000 lamports), and the rent of the added bytes. Measured on the local chain: 39,696 bytes for 276,284,160 lamports at the validator's default 6,960 a byte. At mainnet's 5,080 a byte the same extend is 201,655,680 lamports (0.20 SOL), and the loader's least, 10,240 bytes, is 52,019,200 (0.052 SOL) |
| the upgrade after it | | the loader's own `Upgrade`, 2,370 units on the local chain. Through Poleis's kind 14 the earlier receipt measured 28,500 |
| per frame, per tick | nothing | nothing: each piece runs once a sitting, or once an upgrade |
| this seat's runs | | the press's whole `cargo test` once, nice 19: 29 s on the new Mac at load 46 to 80. The live proof: 131 s in all |

Nothing was sent to any public cluster. No key of his was read or made. No window was opened, and no Canvas tab was born.

## 1. `floor.issuer` held to the issuer's key

**Why it matters.** Root 0's curve names its permit issuer at the founding, and that issuer is fixed from then on. Suppose the params name a
key other than the one `press issuer` made, the key the door holds as `POLEIS_ISSUER_KEY`. Then no permit the door signs would ever seat a
buyer. The floor would stay empty, and root 0 could never be ratified. THE SWARM NAMES ITS MINER held `warm.miner` to its key and left this
one open.

**What changed** (`src/seat.rs`):
- One table, `KEPT`, lists the key each step makes, the param that names it, and why a mismatch matters on the chain:
  - `issuer` / `floor.issuer`
  - `miner` / `warm.miner`
  - `crediter` / `market.crediter`
- One hold reads that table, `held_to_kept`, in three forms: `issuer_is_the_kept`, `miner_is_the_kept` (unchanged in what it does) and
  `crediter_is_the_kept`.
- `kept_keys_held` runs all three and names every param that differs.

**Where it is asked:**
- `Sitting::checked()`: the deploy's check, before anything starts.
- `press found`: before the founding.
- `press acts`: in place of the miner's hold alone.

A placeholder, or a sitting folder where no key was made (a rehearsal's keys never reach a disk), leaves the params' word standing, as the
miner's hold always did.

**Its refusal:** "the params' floor.issuer X is not the key press issuer made beside his keys (K, D/keys/issuer.address): root 0's curve
would name a permit issuer the door holds no key of: no buyer could mint a seat, and the curve's issuer is fixed at the founding; write K in
the params".

**Its test:** `tests/seat.rs` `the_curves_issuer_is_held_to_the_key_press_issuer_made`. It checks:
- the same address passes, another is refused with the kept address in the words, and a placeholder passes;
- with all three keys made and all three params naming another, the hold names each one;
- with no keys made in the folder, the params' word stands;
- the three calls stand where the sitting asks them (read from `chain.rs` and `main.rs`).

## 2. `press crediter`

**What it is.** `market.crediter` is the board's crediter, the key that credits play $WARM to the board's balances. Act 2 names it. It was
a PLACEHOLDER, and `p.sendable()` refuses a sitting while one is left. No step made it, so the only way past was a key typed in by hand that
nobody kept.

**What changed.** `press crediter --dir D` follows the exact pattern of `press issuer` and `press miner`. All three share one road in the
code, `door_key`:
- it uses `seat::keys_folder`, which refuses a folder inside a git repository and sets the folder and `D` to mode 700;
- it makes `D/keys/crediter.json` from the system's randomness (`Key::fresh`), mode 600, `create_new`, in the Solana CLI's file shape;
- it keeps the address beside the key in `D/keys/crediter.address`, with no key in that file;
- it never writes over a key that stands, and refuses when the key stands but its address file is gone.

It prints the address for `market.crediter`, and the press then holds the params to it (piece 1).

**Not quite the brief's wording.** Like the issuer's and the miner's, the step says the address for the params and holds the params to it.
It does not write the address into the params file itself.

**Where it differs from the other two:** it has no `--secret`. No door reads the crediter's key yet, so `press crediter --secret` is
refused with those words.

The placeholder's text in `params.mainnet.json` now names the step. The other values are untouched.

**Its tests** (`tests/seat.rs`):
- `the_crediters_key_is_made_once_beside_the_others_on_their_pattern`:
  - the modes are 700, 700 and 600, and the address file holds only the address;
  - the key file reads back as the CLI's shape;
  - each of the three addresses sits beside its own key;
  - asked again, it makes no second key;
  - two fresh sittings get two different keys;
  - the params naming another key are refused, and a placeholder passes;
  - it never writes over a key whose address file is gone.
- `a_door_key_is_never_made_inside_a_git_repository`: in a scratch `git init` folder all three steps refuse before any key is made, and no
  keys folder is left behind.

## 3. A sealed upgrade extends first

**Why it matters.** The press deploys each program at its build's own length (`deploy_with_max_data_len`, the build's bytes), so the
program's data holds that build and not one byte more. Poleis's kind 6 (`CODEX_UPGRADE`) and kind 14 (`CODEX_UPGRADE_PROGRAM`) both end in
the loader's `Upgrade`. Agave 4.2.2's loader refuses a buffer longer than the data holds ("ProgramData account not large enough"). So the
first sealed upgrade to a longer build would pass the whole tree and then fail at its execute, every time. That is the first upgrade that
adds anything.

**Before this lane the press could not carry out a sealed upgrade at all.** The brief said to change "where the press carries out a sealed
upgrade (its crank)". The crank only carried the eight launch acts, and THE SEAL UPGRADES EVERY PROGRAM said: "the press has no such step
for any program". So this lane builds that place too.

**What changed** (`src/upgrade.rs`, `src/loader.rs`, `src/crank.rs`):
- `loader::extend_program` is the loader's `ExtendProgram` (tag 6, `[additional_bytes:4]`), its accounts `[program data (w), program (w),
  system, payer (s, w)]`. No authority signs it.
- `upgrade::extend_by` works out how much to extend:
  - nothing, when the data already holds the build;
  - otherwise what it lacks, but at least 10,240 bytes, the loader's least (SIMD-0431);
  - or exactly what is left under the 10 MiB an account may hold, when that is less (the loader's own exception);
  - refused past 10 MiB.
- `upgrade::sealed_of` reads a proposal's stored payload as a sealed upgrade: `[CODEX][6][buffer][spill]` is Poleis's own code, and
  `[CODEX][14][buffer][spill][program]` names its program. Any other payload is refused.
- `upgrade::act_of` turns the proposal into an act the crank carries, in three steps:
  - it holds the proposal: root 0's alone, the hash it claims, and a payload whose hash and bytes read back as the upgrade it names;
  - it builds the act: the ninth, its execute the client's own;
  - its ballots are named by 9 or by the hash, and are checked against their voters and rites like any other ballot.
- `press crank --signatures F --upgrade H` carries that act after the eight, the same way: unit 1's ballots, the seals, the mandate, then
  the execute.
- **Before the execute**, `Sitting::extend_first` reads the program's data and the buffer off the chain (`upgrade::extend_on`). When the
  build is longer, it sends the extend in a transaction of its own. The crank, the deployer's key as for every crank transaction, signs it
  and pays the rent. The crank then waits for a slot past the extend's slot, and only then builds and sends the execute.
- Why two transactions: the extend re-deploys the program at its own slot, and the loader refuses an `Upgrade` in that slot ("Program was
  deployed in this block already"). Asked again once the data is extended, the extend sends nothing.

**Its pure tests** (`tests/upgrade.rs`, `tests/wire.rs`):
- `the_bytes_to_extend_are_what_the_build_lacks_and_never_under_the_loaders_least` uses devnet's own numbers: 340,045 to 438,744 gives
  98,744, and to 445,744 gives 105,744. It also covers one byte short (10,240), the 10 MiB edge, and past it.
- `the_extend_is_the_loaders_own_and_the_crank_pays_it`: the instruction equals the loader interface 7.0.0's `extend_program`; the crank is
  its one signer; a wrong account kind is refused; it fits one packet.
- `a_sealed_upgrade_is_read_back_from_its_proposal_and_carried_as_the_crank_carries_an_act`:
  - kinds 6 and 14 read back as the payloads they were proposed as;
  - an authority's move, a buffer's close, a cut payload and an overlong one are not upgrades;
  - the execute equals the client's own and fits one packet;
  - refused: another unit's proposal, a payload the hash does not cover, another hash at the address, a payload that is no upgrade.
- `the_crank_extends_before_the_execute_and_never_in_its_transaction`, read from the sources.
- `wire.rs` `the_extend_is_the_loaders_own_extend_program`, with `MIN_EXTEND` checked equal to the interface's `MINIMUM_EXTEND_PROGRAM_BYTES`.

**Its live test, on the local chain the press's tests already use** (`sh poleis/ceremony/the-upgrade-extends-first.sh`, the live half of
`tests/upgrade.rs`; modelled on `the-mine-is-live.sh`):
- **Setup:** solana-test-validator 4.2.0 on 127.0.0.1, every feature it knows. The $WARM program's sBPF v3 build is loaded at genesis,
  upgradeable, at its own length: 58,816 bytes, data 58,861. Its authority is a stand-in from the run's own seed. The market's v3 build
  (98,512 bytes) was written to a buffer in 110 writes.
- Run Nov 24 2026 23:59:41 to Nov 25 00:01:52 UTC, from 0aef6660cf with this lane's changes. Exit 0. The certificate:
  `poleis/ceremony/the-upgrade-extends-first-2026-11-24.json`.
- What the chain said:
  1. The upgrade, unextended: refused, `AccountDataTooSmall` ("ProgramData account not large enough").
  2. An extend of 168 bytes (devnet's number): refused, "ExtendProgram requires a minimum of 10240 additional bytes".
  3. The press's extend and the upgrade in one transaction: the extend ran ("Extended ProgramData account by 39696 bytes"), then the
     upgrade was refused, "Program was deployed in this block already".
  4. The press's own extend (`extend_on`, the call `press crank` makes): signed and paid by a crank that holds no authority. Slot 146, 2,520
     units, 39,696 bytes, the data 58,861 to 98,557. The crank paid exactly the fee and the rent of the new length. The program was
     re-deployed at slot 146 with its authority unchanged. Asked again: nothing to extend.
  5. The upgrade, a slot later: landed at slot 148, 2,370 units. The data reads back as the market's build byte for byte, the buffer is
     spent, and the spill holds its rent.
- The validator was ended and its ledger removed. Nothing was left running.

## The rest of the proof

- **The press's whole `cargo test`, once, nice 19** (Nov 25 2026 00:02:34 to 00:03:03 UTC, the new Mac at load 46 to 80): **126 passed, 0
  failed, 0 ignored, in 23 targets.** 10 of the tests are new: `tests/upgrade.rs` 6, `tests/seat.rs` 3, `tests/wire.rs` 1. Two of the
  upgrade tests pass without a validator by saying so: the live half and the seed's helper.
- One existing test was changed in place: `tests/chair.rs` pinned the crank's line in `main.rs`, which now passes the upgrade's hash too.
- **The house's gates that read a lane's diff, run here:**
  - `tools/any-room/any-room.test.mjs`: fail 0;
  - `tools/embertide-mesh/tests/the-javascript-only-shrinks.test.mjs`: fail 0 (no line of JavaScript added).

## What the sitting would have tripped on

1. **Every sealed upgrade to a longer build** passes the whole tree and then fails at its execute (`AccountDataTooSmall`). The press
   deploys each program at its exact length, and nothing extended it. Closed here.
2. **The extend and the upgrade cannot share a transaction or a slot.** The extend re-deploys the program at its slot, and the loader
   refuses an upgrade in that slot. A road that put both in one transaction (the obvious first try) would be refused every time. And an
   extend under 10,240 bytes is refused outright. Both are handled here.
3. **`market.crediter`** had no step that made it, so the sitting could not leave `sendable()` without a key typed in by hand that nobody
   kept. Closed here.
4. **A `floor.issuer` other than the door's key** would have founded a curve that no permit of the door could ever mint on. The floor would
   stay empty and root 0 could never be ratified, and the curve's issuer cannot be changed after the founding. Closed here.

## What is not done, and whose

- **Nothing prepares a kind-14 (or kind-6) act's proposal.** `press vote` writes sets for the eight launch acts only. The crank carries a
  sealed upgrade once a seat owner has proposed it, but no step yet writes that proposal for a seat owner to sign.
- **The sealed road through Poleis was not run on a chain here.** The live proof ran the press's extend and the loader's own `Upgrade`
  after it, signed by a stand-in authority. A kind 14 voted by the tree and carried out by `press crank --upgrade` to a longer build waits
  for the launch rehearsal, about 30 minutes of the new Mac. Its `checks.upgrade` today re-deploys the same build, which needs no extend.
- **The crediter's key has no road to whoever credits.** No door reads it yet, so `press crediter --secret` is refused.
- **Anyone may extend a program (the loader asks no authority), and the extend re-deploys it at its slot.** A stranger's extend in the slot
  of the crank's execute would make that execute refuse for that one slot, and the crank sends it again later. What a re-deploy does to
  ordinary calls of the program in that same slot was not measured.
- **The launch rehearsal was not run again.** The three pieces touch the deploy's check, the founding, the acts and the crank, but no byte
  that a sitting without door keys on disk sends.
