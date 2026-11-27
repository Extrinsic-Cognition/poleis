# EACH POLIS OPENS ITS OWN VOTE: a decision folds up a tree of any width, one transaction per polis (Nov 3 2026, lane/each-polis-opens-its-own-vote, claude-ac61d4f0)

## His words, UTC

- **Sep 9 13:43:** "we could do 150k voters and it would work the same".
- **Mar 6 18:35**, his 2021 Solarity paper pasted: "offering as many layers of governance as needed. (Having G.V.S. as Pentagons of a Bigger G.V.S.)".
- **Sep 2 to 9**, his standing wish "Hexagon and pentagon voting governance (Poleis)" (`ec-workspace/docs/wishes-and-the-servants-hypertrace-2026-10-11.md`):
  "IT SHOULD NOT BE POSSIBLE FOR LAYER0 TO RESOLVE WITHOUT ALL CHILDREN UNITS VOTING AS WELL".
- **Nov 2 18:47:19**, over the figure: "yes... and are you gettign the recursive composability of it or not as a system?"
- **Nov 3:** "please deliver on it" over the scorecard, then "Do this task here" on this lane's card. This lane is move 4 of the order in section 10 of
  `ec-workspace/docs/the-holy-goldberg-governance-the-perfect-recursive-bicameralism-hypertrace-2026-11-02.md`.

## Why

**One decision was one Solana transaction.**
- `propose` opened every session of the subtree at once, and the set had to close over every chair that held a polis (`ClosureBroken`).
- At most twelve poleis fit in it (`MAX_IMPACTED`), and founding a polis never checked that cap (`found_unit`).

**So a root with more than eleven poleis under it could never decide again,** not even to recall a chair and shrink. The probe of Nov 2 proved it:
- thirteen poleis, a root with six crews and a crew under each;
- `InvalidInstructionData` over all thirteen, `ClosureBroken` over any twelve.

His "150k voters" held in the law, not in the code: a governable tree topped out at twelve poleis and 1,800 floor seats.

## What changed

1. **Each polis opens its own vote** (`convene`, instruction 55).
   - A unit sitting in a lit chair of a unit whose session of the decision is open opens its own session, in its own transaction, permissionlessly.
     It can do so while that session's window runs.
   - Its window runs from the proposal's opening, as if it had opened with it, so opening late extends nothing.
   - The crank pays the rent. The session ends with a 40-byte tail naming who paid it.
2. **`propose` no longer closes the set.**
   - It opens the origin and any of its descendants, up to twelve, as before. A crew it leaves out opens its own.
   - The old way still works and is still one transaction for a small tree.
3. **Every seal waits for every crew in its lit chairs** (`Registry::crew_chairs`), whatever opened that crew's session. Before, a seal waited only
   for the crews the proposal listed. "No decision is ever sealed over the head of a crew" now holds without a list.
4. **A crew that never opens its vote is heard absent** once its parent's window has closed, the same as a crew whose vote expired.
   - `mandate` takes the crew's empty ballot box: the right address, no data.
   - So a silent polis delays its parent by one window at most.
   - It also ends a freeze. A chair seated with a unit that can never speak had to be listed in every decision, and no decision could list it, so
     the polis was frozen for good. Now that crew is heard absent and the polis can recall the chair.
   - This is proven for a unit with no registry. A dissolved crew, or one of another tree, cannot open its vote either and takes the same road.
5. **Under consent, a seal carries its crews' AND.**
   - If both chambers say yes over a crew that did not pass, the seal is DEADLOCK: the tree is split.
   - So the origin's seal alone says whether the whole tree passed, and `execute` needs no other session to know it.
6. **A crew speaks in every lit chair it holds**, as a key does (`Registry::chairs_of_unit`).
   - Before, it spoke only in the first chair it held.
   - A chair whose term has ended now carries no verdict. Before, a unit's verdict still counted in a chair the quorum no longer counted.
7. **The belt on the code** (`execute` for CODEX): every crew in a lit chair of a listed unit must be listed itself, or have spoken in that chair.
   - Spoken means heard there, and seated there before the question was asked, so the voice the chair carries is its own.
   - A crew seated mid-session cannot carry the code: the old voice of its chair belongs to whoever held the chair before.
8. **`adjourn`** (instruction 56): once the decision is executed, or its proposal is closed to its tombstone, a convened session closes and its rent
   goes home to whoever paid it. A session `propose` opened still goes home with its proposal (`close_proposal`).
9. **The client** (`poleis/client`): `convene`, `adjourn` and `SessionView::convener`. Its tags re-export the two instructions.

**No account layout changes** (`LAYOUT_VERSION` stays 8).
- A session `propose` opens is byte for byte what it was, and every account on devnet reads as before.
- A convened session is a session plus the convener's tail.

So the house's program can take this code by its own seal, with no migration.

## The gate

Four new council tests:
- **`each_polis_opens_its_own_vote_and_a_root_over_thirteen_decides`**: the Nov 2 probe made the gate.
  1. One transaction still refuses all thirteen poleis.
  2. The root opens alone and the twelve crews open their own.
  3. The leaves seal first, then their crews, then the root, PASSED.
  4. The act executes, and the twelve rents come home.
  5. Then, under consent, one leaf's no splits its crew, and the crew's split splits the root: DEADLOCK, and `execute` refuses.
- **`a_crew_that_never_opens_its_vote_is_heard_absent`.**
- **`the_code_never_moves_under_a_crew_seated_since_the_question`**: a crew seated in a second chair mid-session, by two seat acts of the root.
- **`a_crew_that_can_never_speak_never_freezes_its_parent`**: a chair seated with a unit id that has no registry. The root still decides, and
  recalls that chair.

Three tests changed to the new law:
- `scope_from_touch_and_the_child_speaks_first`: a decision that leaves a crew out opens anyway, and waits for that crew.
- `a_crews_no_is_its_pentagon_unless_consent_was_asked`: under consent the root seals DEADLOCK, where it used to seal PASSED, and `execute` still
  refuses.
- `a_crew_cannot_touch_the_code`: the root's upgrade opens alone and waits for its crew.

The seeded fuzz (`tests/fuzz_council.rs`):
- A third of its proposals open the origin alone.
- Each crew opens its own vote, carries its seal up or is heard absent, and adjourns.
- **S8 now checks the law it states:** the origin's seal, and every seal under consent. Before, it asserted every listed seal PASSED, which is
  stricter than the law and held only by the seed's luck.
- **New S10:** a ballot box holds its rent and nothing more, and names who paid when its polis opened it.
- The season: 740 acts accepted and 1,047 refused, 14 votes opened by the polis itself, 13 crews heard absent, 13 executions. Every invariant held
  after every op.

## Cost first, measured on the new Mac (nice 19)

Predicted while designing: a `convene` costs about what a one-session `propose` does, 10,000 to 20,000 CU, and the other calls stay within a few
hundred CU of before.

Like for like, the same calls of `neither_capital_nor_crowd_moves_alone`, main eb27283a5 (move 2's code) against this lane:

| call | before | after |
|---|---|---|
| `propose`, two sessions | 20,445 to 26,445 CU | 18,312 to 24,312 (−2,133: no closure pass) |
| `seal` | 8,491 for the crew's, 9,002 to 9,025 for the root's | 7,664 and 8,155 to 8,176 (−830 to −870: the crews' chairs read in one pass) |
| a refused `seal` (a crew has not spoken) | 7,236 | 6,257 |
| `mandate` | 6,229 to 6,244 | 6,530 to 6,541 (+300: every lit chair the crew holds) |
| `attest` | | +3 CU for each floor vote |
| `execute`, a transfer | 14,596 | 14,687 (+91) |
| `close_proposal` | 4,508 | 4,508 |

The new calls:

| call | CU |
|---|---|
| `propose`, the origin alone | 11,255 to 20,255 (the address searches vary by hash) |
| `convene` | 11,551 to 17,553 |
| `mandate`, a crew that never opened its vote | 6,125 |
| `adjourn` | 4,119 |

**One decision over the thirteen poleis**, every call from the propose to the close: 66 transactions and 507,147 CU.

| calls | CU |
|---|---|
| 1 propose | 20,255 |
| 12 convene | 165,624 |
| 14 attest transactions carrying 23 signed votes | 80,445 |
| 13 seals | 99,859 |
| 12 mandates | 78,697 |
| 1 execute | 8,331 |
| 12 adjourns | 49,428 |
| 1 close | 4,508 |

- **Fees:** about 445,000 lamports, 89 signatures at 5,000 each.
- **Rent:** 12 × 2,268,960 lamports, all of which comes home.
- **Per polis a decision touches:** about 32,000 CU and five transactions (open, a batch of votes, seal, mandate, adjourn), plus its own votes.
- **Before this lane, that tree could not decide at all.**

The crate and the program:

| | before | after |
|---|---|---|
| the crate's `cargo test --offline --no-fail-fast --locked`, as the gate runs it | 153 passed, 1 ignored | 157 passed, 0 failed, 1 ignored |
| from an empty target directory | 81.3 s of wall clock, 230 s of CPU (move 2's receipt) | 55.6 s of wall clock, 289 s of CPU (199 user, 90 system) |
| the client crate | | 19 passed |
| the program | 423,688 bytes | 432,368 bytes (+8,680, sha256 `f249827f459d6088…`, `cargo-build-sbf` 4.1.0) |

- The program grows for two new instructions and the reworked `mandate`, `seal` and `execute`.
- Opening a session is one function now, shared by `propose` and `convene`, which saved 1,032 bytes.
- Keeping the readers out of line would save 544 more bytes; that was measured and not taken.
- To take this code, the house's program must grow by 8,680 bytes first: about 60,400,000 lamports of rent, 6,960 a byte. `upgrade.mjs prepare` extends
  the program before it writes.

## What is left

- **His press, move 3:** seat the house. "On devnet the chairs are stand-ins for his three Keystones (rehearse.mjs): whose keys seal is his call"
  (`ceremony/upgrade.mjs`). The house's program takes this lane only by its own seal: `node poleis/ceremony/upgrade.mjs prepare`, then `seal`.
- **Move 5, a deadlock goes to the market,** waits on his design word: how the market's verdict reaches the chain without one key deciding it.
- **The JavaScript clients** build neither `convene` nor `adjourn`.
  - The ceremony's `propose` over its one polis is unchanged and still works.
  - The JavaScript only shrinks: the road is the Rust client.
- **The 150-holder floor of a polis (`MAX_HOLDERS`) still stands.**
  - His "the 150 holders is an artificial limit" (the same wish) is the next ceiling, and this lane does not touch it.
  - A tree now carries 150 holders in each of its poleis, any number of poleis wide and five levels deep.
- **A polis that opens its vote late counts the roster it has when it opens.** This matters for an NFT society, whose electorate a session fixes.
  Anyone can open it at once after the proposal, even in the proposal's own transaction.
- **Found while reading, not fixed here:** a seat act may seat an ancestor in a descendant's chair (`SEAT_ATTACH` checks only that a unit is not
  itself).
  - That makes a cycle: each seal waits for the other forever, and the sessions' rent never comes home.
  - This was so before this lane too. `SEAT_ATTACH` should refuse an ancestor.
