# THE CHAIRS ARE A BUDGET: the chairs' chamber is a majority of the lit chairs, and the tests build the program they load (Nov 3 2026, lane/the-chairs-are-a-budget, claude-ac61d4f0)

## His words, UTC

- **Nov 1 01:32**, over a party laid as twelve golems and twelve chairs: "12 pentagons/companions is a budget it is not a requirement".
- **Oct 30 17:13:** "the difference between 3 pentagons active and 12 pentagons total".
- **Nov 3, "please deliver on it",** over the scorecard of section 10 of
  `ec-workspace/docs/the-holy-goldberg-governance-the-perfect-recursive-bicameralism-hypertrace-2026-11-02.md`. This lane is moves 1 and 2 of its
  order.

## Why

**The seal counted seven of a fixed twelve, and an unlit chair counted as one that had not said yes.** So with three chairs lit nothing could ever pass.
- The house's own program on devnet (`4627ah…`) works around it: its twelve chairs are three stand-in keys, four chairs each
  (`ceremony/THE-CEREMONY-REHEARSED-LANDING.md`).
- No agent can light a chair from the bottom up while seven of twelve is the bar.

**The lander's crate gate read this crate red on main for a missing file.**
- All nine of its test targets failed with `[MOLLUSK]: Program file not found: poleis_treasury.so`
  (`tools/embertide-mesh/lander/THE-CRATES-ARE-GATED-LANDING.md`).
- The gate runs `cargo test` alone, and no step built the program the tests load.
- Built by hand first, the same tests passed 152 of 152 (Nov 2 18:2x UTC). The governance's own proof had never gated a lane.

## What changed

1. **The majority of the lit chairs** (`src/lib.rs` `chair_quorum`, `src/state.rs` `Registry::lit_chairs`, `src/ix/proposal.rs` `seal`).
   - The chairs' chamber passes on more than half of the chairs lit when the session seals: `lit / 2 + 1`. Seven is still the bar when all twelve are
     lit (`PENTAGON_QUORUM` is now `chair_quorum(12)`).
   - A chair is lit when a key, a unit or an NFT holds it and its term has not ended: the chairs `chairs_of` lets vote.
   - The floor's rule is unchanged, and so is the AND: a chairs' majority alone is still DEADLOCK.
   - **No account layout changes** (`LAYOUT_VERSION` stays 8). The count is read from the registry at the seal, so the house's program can take this
     code by its own seal without a migration.
   - The count moves within a session only by that polis's own sealed seat acts (attach, recall, found).
2. **The tests build what they load** (`tests/built/mod.rs`).
   - Each test binary builds the program from this tree once, with `cargo-build-sbf --offline`, before its first test reads it.
   - The builder is found on PATH, else under the active Solana release, else the newest release installed.
   - The nine targets take their folder and their ELF from it, so a stale `.so` from another commit can no longer pass for the code under test.
3. **The fuzz's seal invariant is the new law** (`tests/fuzz_council.rs` S7). A PASSED session holds a majority of the chairs lit when it sealed, read
   the first time the fuzz sees it sealed (the op that sealed it moved no chair). A REJECTED session holds less.
4. **A new council test,** `twelve_is_a_budget_three_lit_chairs_decide_by_two`, on a polis of three founders and a floor of one:
   - two of three chairs and the floor are PASSED;
   - one of three and the floor is DEADLOCK;
   - two of three against the floor is DEADLOCK.

The client re-exports `chair_quorum`. No JavaScript is added or changed. The JavaScript clients that read `chair_mask` are untouched: the mask means what
it meant.

## Cost first, measured on the new Mac (nice 19)

| | before (main e52c0a63b) | after |
|---|---|---|
| the crate's `cargo test --offline --no-fail-fast --locked`, as the gate runs it | 9 of 11 targets red, 0 of them proven | 153 passed, 0 failed, 1 ignored |
| from an empty `target/` | red in 40 s (the gate's receipt) | 81.3 s of wall clock, 230 s of CPU, the program's build included |
| warm | | 19.7 s |
| a seal (`POLEIS_CU=1`, `neither_capital_nor_crowd_moves_alone`) | 8,351 CU for the child's, 8,872 to 8,882 for the root's | 8,491 and 9,015 to 9,025: +140 CU, the twelve chairs read |
| a refused seal (the child must speak first) | 7,236 CU | 7,236 CU |
| the program | 422,264 bytes | 423,688 bytes (sha256 `6cb54e83d2214f2c…`, `cargo-build-sbf` 4.1.0) |

The predicted cost of the count was a read of twelve chair rows, about a hundred CU, and the measured cost is 140.

## What is left

- **His press: seat the house.** The house's program takes this code only after it lands on main, by its own seal: `node poleis/ceremony/upgrade.mjs
  prepare`, then `seal`. "On devnet the chairs are stand-ins for his three Keystones (rehearse.mjs): whose keys seal is his call"
  (`ceremony/upgrade.mjs`). The program grows by 1,424 bytes, so `prepare` extends it first. Then two sealed seat acts: agents light chairs from the
  bottom up, and his Keystones move to the floor.
- **The next lanes** of the hypertrace's order:
  - each polis opens its own vote, which removes the twelve-poleis limit of one decision and the frozen root it allows;
  - a deadlock goes to the market.
- **The JavaScript clients** still print seven of twelve where they print a bar.
