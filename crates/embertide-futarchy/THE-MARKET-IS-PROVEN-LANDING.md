# THE MARKET IS PROVEN: how the swarm decides, in Rust, with its proofs (Nov 11 2026, lane/the-market-is-proven, claude-602db0e0)

## His words (UTC)

- Oct 26 18:18: "i am very fond of futarchy as a way to have ai automated decision markets for a collective".
- Oct 29 19:10: "Nefertiti will spawn bunch of possible roads to be taken, market will then run futarchy on the proposed avenues... it is not hard"; 19:12:
  "$WARM is liquid upfront qnd is what the futarchy runs decisions on".
- Nov 11 00:55: "i think we should get into ai futarchy with formal verification for governance".
- Nov 11 ~02:0x, over the drawing of the rule on the Swarmbook page ("How the swarm decides": roads laid, two markets, the seal): "ok, perfect. can you
  implement that?"

## What stood

`tools/embertide-mesh/futarchy/` (Oct 30, his "Yes, build it"): the law in JavaScript floats, a Nest reader, a proof on a scratch Nest. Nothing ran it;
the live Nest holds no `futarchy/*` ref. No formal verification stood anywhere in the house.

## What was wrong in the text, found while writing the proofs

**The lag limit moved once per entry, not once per step.** `advance` stepped the observation `max_step_bp` toward the spot at every call, and the fold
calls it at every trade. A trader who spiked the spot and then sent one dust trade a millisecond walked the average to the spike. Reproduced on the text
itself (`futarchy-law.mjs`, a road of three days, a step a minute, 1 % a step, threshold 3 %), a whale buying 900 of PASS an hour before the close:

| | PASS's average | needs | the road |
|---|---|---|---|
| the spike alone | 1.005205 | 1.030000 | failed |
| the spike and 400 dust trades, a millisecond apart | 1.036249 | 1.030000 | **passed** |

## What the lane does

A new crate, `embertide-futarchy`. No JavaScript is added.

- **`src/law.rs`, the one law, in integers, no dependency.** Amounts are millionths of a token, prices are quote per base times 1e12. Two constant
  product pools, PASS and FAIL; the fold of a road's entries in the Nest's order at the Nest's stamps; the decision; each trader's paper result.
  - **THE GRID.** The observation moves only at the grid's boundaries, `opens + n * step_ms`, toward the spot as it stands there, by at most
    `max_step_bp`. Entries between two boundaries cannot move it. A trade stamped on a boundary is seen by that boundary.
  - **The decision compares the two sums** (`pass * 10000 > fail * (10000 + threshold)`): no division decides. Equal averages do not pass.
  - **The range.** A pool or a purse holds at most a billion tokens; a spot stays within 1e-6 and 1e9; a window is at most 366 days. A trade that
    would leave the range is refused. With those bounds no product leaves u128, which the compiler checks (the consts at the foot of the file).
  - **What it refuses**, each with its word: not a trade of this road, unsigned, outside the window, stamped before an earlier entry, not enough in
    that branch, too small to take anything out, past `min_out`, out of range. A refused entry changes nothing.
- **`proofs/*.smt2`, six claims over the unbounded integers, each answered `unsat` by z3** (no integers break it):
  a buy never drains the pool and never lowers its product; the same for a sell; buying and selling straight back never pays; one boundary moves the
  observation by at most `max_step_bp` of itself, toward the spot and never past it; the average lies between the lowest and the highest
  observation (by induction); when the sums pass, the averages shown beside the verdict agree with it to the rounding.
  `tests/proofs.rs` runs them, and checks that each proof's premises alone can be met (a proof from contradictory premises would also say `unsat`).
- **`tests/law.rs`, the same claims on the code itself**: every sequence of three trades out of 48 on a small pool (110,592 markets, each compared
  with the market without its refused entries); a seeded sweep of 160 large markets (both tokens conserved to the micro, the product never falls, the
  average between the spots, a reading sees only what was in); carrying the average in two reads is carrying it in one; the spam above, which this
  law does not follow (the 400 entries move the average by nothing) and the old rule, written beside it in the same integers, does.
- **`tests/witness.rs`: the text is the witness.** `futarchy-law.mjs` answered 24 markets whose trades stand on the grid; its answers are kept as data
  (`tests/witness.txt`) and the law gives the same verdict (11 passed, 13 failed), the same counts, the averages within a millionth.
- **`src/nest.rs` and the command `embertide-futarchy`** (`state`, `propose`, `trade`, `settle`): a road is `futarchy/proposals/<id>`, laid once; a
  trade is a `futarchy.trade` entry signed by its trader's own key; the reader checks every signature itself over the Nest's body of the entry
  (`tests/signed-entry.json` was signed by the Nest's own author file, and verifies here); the decision is `futarchy/decisions/<id>`, written once,
  at the Nest's own clock (`server_ms`), never the caller's; a road of allowances that passed writes `futarchy/allowances/<month>` as before, which
  `seal-follows.mjs` reads. A road is a month's allowances, or a road in words (`kind: "road"`, `what`).
  The schema is `embertide.futarchy/2` (integers); the text's `/1` roads are not this law's. A trade carries its author's instant, because the Nest
  keeps one entry for two that are the same to the byte.

## Read (the new Mac, fb957b51f6, Nov 11 2026 ~02:3x UTC)

- `cargo test --locked`: 14 of 14 (law 7, the Nest's words 4, the proofs 2, the witness 1); z3 said `unsat` to all six, in under a second.
- `cargo test --locked --test on_a_nest -- --ignored`: 1 of 1, 42 s. On the house's own `nest.mjs` as a scratch Nest: two roads laid by a key file
  as the Nest's keygen writes it; a second hand's road of the same id refused; three wardens trade; an unsigned trade; a whale's spike of PASS ten
  seconds before the close and 150 more trades in 915 ms. Road 1 passed (PASS 1.2222 against FAIL 1.1036, needs 1.1367) and granted its allowance;
  road 2 failed (PASS 1.0849 against FAIL 1.2222, needs 1.2588), 154 trades taken, the unsigned one refused; a second settle moved no ref.
- `cargo build --no-default-features`: the law alone builds with no dependency.
- The gates: the-javascript-only-shrinks 13 of 13, the-law-is-the-gate 7 of 7, the-look-is-a-law 2 of 2; any-room 12 of 13, red on main's own file.

## Cost

- The fold: 110,592 markets of three entries folded and checked twice in 90 ms (release), 424 ms (debug).
- A reading of a road with 155 entries on the scratch Nest: 60.7 ms, two asks of the Nest and 155 signatures checked.
- A trade: one signed entry. Nothing runs at rest: no timer, no poll, no process.
- The proofs: six z3 runs, 0.23 s together.

## What is not done, and what is not proven

- **No market runs on the live Nest.** Nothing lays a road and no agent trades: Nefertiti has no door to this command yet. That is the next piece.
- **The proofs are of the law's arithmetic, not of the Rust source.** The six claims are stated by hand over the integers and mirror `law.rs` line by
  line; the code is then checked against them over the small pool and the sweep. A prover that reads the Rust itself (Kani) is not on the Macs.
- **Not proven:** that a market's answer is a good decision. The lag limit bounds how fast the average follows a spot; a trader who holds a price for
  the whole window still sets it, and on paper nobody loses money by doing so.
- **The public page shows no market**: the Swarmbook page's drawing stays "The rule, drawn." A market the site can show must stand at the router or on
  a chain, not on a home Nest.
- The text (`futarchy-law.mjs`, `futarchy-nest.mjs`, its proof) is left where it is, as the witness; nothing calls it. Taking it out is a lane of its
  own, once nothing reads `/1`.
- `proofs/*.smt2` are outside the paths the lander's crate gate watches: a lane that changes only a proof must touch `tests/` to be gated.
- A mistake of this seat's while building the test: the first harness named `nest.mjs` as node's first argument, so it ran as a house Nest over the
  new Mac's default folder (`~/code-ec/braindance-data/station-0/nest`, last written Sept 30), three times. It rewrote that folder's `verified.json`
  stamp, could not take port 8904 and exited. The new Mac's live Nest was read after: it answered and advanced. The harness now names the file, the
  folder and the port by the environment only, with `NEST_DIR` and `NEST_PORT` the scratch ones.
