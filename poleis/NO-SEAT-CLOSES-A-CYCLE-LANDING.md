# NO SEAT CLOSES A CYCLE: a unit sits only below the polis that seats it, in its own tree, named by its id alone (Nov 3 2026, lane/no-seat-closes-a-cycle, claude-f47ff373)

## Cost first, measured on the new Mac (nice 19)

Like for like, the same calls before (main d794edb85, the program of `EACH-POLIS-OPENS-ITS-OWN-VOTE-LANDING.md`) and after this lane:

| call (`execute` of a seat act) | before | after |
|---|---|---|
| seat a unit with a retainer (`a_fund_pays_to_attach_a_crew`) | 12,955 CU | 14,940 CU (+1,985) |
| seat a unit (`the_code_never_moves_under_a_crew_seated_since_the_question`) | 10,442 CU | 12,429 CU (+1,987) |
| seat a key (`recall_and_attachment_are_seat_acts`) | 8,456 CU | 8,450 CU (−6) |
| recall a chair | 8,365 and 10,364 CU | the same |

- A unit's seat now reads that unit's registry by its address. That costs about 2,000 CU, and 1,500 of it is the address check (`create_program_address`).
  It is kept, as `read_registry` does for every registry a decision trusts.
- A seat act runs once per decision that seats someone, never per vote or per frame.

| | before | after |
|---|---|---|
| the program | 432,368 bytes, sha256 `f249827f459d6088…` | 432,696 bytes (+328), sha256 `45b120cbec1fd09a…` (`cargo-build-sbf` 4.1.0) |
| the crate's `cargo test --offline --no-fail-fast --locked` | 157 passed, 0 failed, 1 ignored | 160 passed, 0 failed, 1 ignored (17.0 s wall on a warm target) |
| the client crate | 19 passed | 19 passed (11.6 s wall) |
| the seeded season | 740 accepted, 1,047 refused, 14 votes a polis opened itself, 13 crews heard absent, 13 executed | the same counts |

- The program grows 328 bytes on top of the 8,680 of the last lane. At 6,960 lamports a byte, that is about 2,283,000 lamports more rent for the house's program
  to take this code by its own seal.
- The refusals, as the new tests measure them:

| refused seat | CU | error |
|---|---|---|
| a leaf seats its parent | 10,244 | `NotDescendant` |
| a leaf seats itself | 10,236 | `NotDescendant` (was `UnitMismatch`) |
| a leaf seats the root | 6,213 | `InvalidHolder`, as on main: unit 0's holder is 32 zero bytes |
| a crew seats its sibling | 12,237 | `NotDescendant` |
| a leaf seats the crew that seats it | 10,244 | `NotDescendant` |
| a unit with no registry | 8,319 | `InvalidAccount` |
| a unit id with non-zero padding | 6,217 | `InvalidHolder` |
| a unit of another tree, deeper than the origin | 10,241 | `NotDescendant` |

## His words, UTC

- **Sep 8 04:18:** "the point is exacrtly that agents and principals cant "kill" each other".
- **Mar 6 18:35**, his 2021 paper pasted: "(Having G.V.S. as Pentagons of a Bigger G.V.S.)". A decision is a fold up the tree: the leaves seal first and
  the root seals last. A fold ends only if no polis is, through its chairs, its own crew.
- **Sep 2 to 9**, the standing wish "Hexagon and pentagon voting governance (Poleis)": "IT SHOULD NOT BE POSSIBLE FOR LAYER0 TO RESOLVE WITHOUT ALL
  CHILDREN UNITS VOTING AS WELL". With a cycle, layer 0 can never resolve again.

## Why: three ways main's seat act froze a polis for good

`SEAT_ATTACH` checked only that a unit was not the origin itself. Two probes ran on main's program (throwaway tests, before this lane changed a line);
the third case is the last lane's own test:

1. **A cycle.** In the thirteen-poleis tree, leaf 7 seated its own parent, crew 1, in its free chair 5, and main executed it.
   - At the root's next decision, any crank convened every crew: crew 1 waited for leaf 7, and leaf 7 waited for crew 1.
   - Both seals, both mandates, the root's seal and `close_proposal` were refused, and the same at slots 200, 10,000,200 and 20,000,200.
   - 37,236,000 lamports of rent stayed in 13 ballot boxes and the proposal.
   - Any decision of a polis above a cycle freezes this way once both sides open their votes. Convening is permissionless, so any crank can do it
     for two ballot boxes' rent, and a proposal that lists both sides freezes at birth.
   - A pair of cousins (crew 1 seats leaf 8, then leaf 8 seats crew 1) would freeze by the same two waits, though neither is the other's ancestor.
     That was not probed on main; the new test refuses the second seat.
2. **A holder no unit answers to.** `holder_unit` reads 8 bytes, `chairs_of_unit` compares 32.
   - A unit id with non-zero padding was seated. Every seal of the polis waits for that chair (`crew_chairs`), and no unit can ever convene or be
     mandated there.
   - The root never sealed again, not even to recall the chair. The last lane's absent road does not reach this case.
3. **A ghost or a stranger.** A unit with no registry, or a unit of another tree, was seated. Since the last lane each is heard absent after one window,
   so the polis is slowed but not frozen. Still, the seat names no crew of this tree.

## What changed

**The seat act seats a unit only below the polis that seats it, in its own tree** (`ix/proposal.rs`, `execute`, `(CLASS_SEAT, SEAT_ATTACH)`):
1. the holder is the unit's id and nothing more: `holder == unit_holder(holder_unit(holder))`, else `InvalidHolder`;
2. its registry exists, read by its address, else `InvalidAccount`;
3. it names the origin's root, else `NotDescendant`;
4. its level is greater than the origin's, else `NotDescendant`.

**Why that is enough:**
- A level never changes after founding (`Registry::set_head` never writes it).
- `found_unit` gives a child its parent's level plus one, and every seat act now rises too. So along every chair seated by founding or under this law,
  the level rises.
- No path through those chairs can come back to where it started, and no chain of crews is longer than `MAX_DEPTH`.
- A true descendant is never refused: it always shares the root and stands deeper.
- One registry read, one more account. No account layout changes (`LAYOUT_VERSION` stays 8).

**Weighed and not taken:**
- **A polis seats only the children it founded** (`parent_unit == origin`): the same cost, also acyclic, but it refuses a deeper cousin. The level rule
  seats any deeper unit of its tree, of any branch. What it refuses beyond cycles is a seat at the same level or upward, acyclic or not, and no fold
  needs one.
- **Walk the origin's ancestors:** up to five more registries per seat. It also misses the pair of cousins above, which no ancestor walk sees.

**The client** (`poleis/client`, `Payload::accounts` for `Attach`): a unit's seat sends `[registry of the origin (w), registry of the unit (r)]`, then
the two treasuries for a retainer. A key's seat is unchanged.

## The gate

Three new council tests (`tests/council.rs`):
- **`a_polis_never_seats_its_ancestor`:**
  - Leaf 7 is refused its parent, itself and the root.
  - Then the probe's decision of the root folds up all thirteen poleis, PASSED, and executes.
- **`a_pair_never_seats_each_other`:**
  - Crew 1 is refused its sibling, crew 2.
  - Leaf 8 (crew 2's own) is seated in crew 1's chair, because it is deeper.
  - Leaf 8 is refused crew 1 back.
  - Crew 1's next decision waits for leaf 8, which convenes under it, seals and is mandated: PASSED, 4 of 4 chairs.
- **`a_seat_names_a_unit_of_its_own_tree`:** each is refused:
  - a unit with no registry;
  - a padded unit id;
  - crew 51 of tree 50, which stands deeper than root 0 but in another tree.

One test changed:
- **`a_crew_that_can_never_speak_never_freezes_its_parent`:** the seat act refuses its ghost now.
- The test lays the ghost's chair in the registry's bytes, as an older program's seat act left it (`World::lay_chair`).
- It still proves that such a chair is heard absent and recalled.
- A program upgraded in place keeps every chair it held.

The three seatings the council tests relied on all stand:
- `a_fund_pays_to_attach_a_crew` (unit 1, level 1, into the root, level 0, with a retainer);
- `the_code_never_moves_under_a_crew_seated_since_the_question` (unit 1 in a second chair of the root);
- `recall_and_attachment_are_seat_acts` (a key).

**How to run it:** `cd poleis/program/poleis_treasury && nice -n 19 cargo test --offline --no-fail-fast --locked`, and the same in `poleis/client`.
The tests build the program from the tree (`tests/built`). A Rust lane, so it is handed with no `.mjs` proof. JavaScript net 0.

## The live chain

The house's program on devnet (`4627ahFCh78nC9NJ7Psb7B1NSjDkmnP8SPfvbt5MQz84`), read with `getProgramAccounts` at slot 518,246,103 (Nov 3 14:29 UTC):
- one registry, unit 0, layout 8;
- all twelve chairs are keys.

No live chair seats a unit, so there is no cycle, no padded chair and no ghost to heal. That program can take this code by its own seal with no migration.

## What is left

- **A padded chair seated by an older program stays frozen:** `mandate` cannot hear a chair whose holder names no unit. None exists on the house's
  program (above). Healing one would need `mandate` to hear any unit chair by its index; not done.
- **The JavaScript clients** (`embertide-app/ui/intro/js/poleis-client.js`, its copy under `dyson-society/js/`, and `poleis/gen/poleis-client-v7.js`)
  still build the old account list for a unit's seat.
  - The new program refuses it safely: `InvalidAccount` with a retainer, `NotEnoughAccountKeys` without.
  - Nothing in the ceremony seats a unit.
  - The JavaScript only shrinks: the road is the Rust client.
- **The seeded season never seats a unit.**
  - Its S6 still states the founding law (a bound child names its parent and chair), which holds while the season seats only keys.
  - A trial that seated units in 1 of 128 ops asked twice in 700 ops and seated none. It was not kept.
  - A season that seats units needs a recall-then-seat rhythm before its S6 can state this law.
- **A dissolved unit can still be seated.** It is heard absent at every decision and its polis can recall it, so it is slow but never frozen. Not in
  this lane's brief.
- **The governance hypertrace** (ec-workspace, section 9) says "any polis can sit in any chair". It now reads "in any chair of a polis above it, in
  its own tree".
- **His press, move 3:** seat the house. The house's program takes this lane, as it takes the last one, only by its own seal (`ceremony/upgrade.mjs`).
