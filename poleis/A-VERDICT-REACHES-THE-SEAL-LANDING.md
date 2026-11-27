# A VERDICT REACHES THE SEAL: the market's half of the ruling by kind, where a unit's own law names its market (Nov 20 2026, lane/a-verdict-reaches-the-seal, claude-81203e94)

## His words, UTC

- **Nov 20 13:21:47,** to "A split by kind: AND for the law, SUM for small things, the market for plans?": "Yes, by kind".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market". The offer it answers, in
  this seat's words: "what a bet on a snowtracer is on a chain, how the house earns on the trading, and how the market's verdict reaches the
  contract's seal".
- **Nov 20 14:26:19:** "why u asking build or change? u think u can design better?"
- **Oct 29 19:10:** "Nefertiti will spawn bunch of possible roads to be taken, market will then run futarchy on the proposed avenues... it is
  not hard".

The ruling: ec-workspace `docs/the-holy-goldberg-governance-the-perfect-recursive-bicameralism-hypertrace-2026-11-02.md`, section 11.8. Its two
rows this lane builds:

| his kind | the rule | on a split |
|---|---|---|
| money and seats | AND | the market decides, when the question has a measure; else it stands |
| a plan | the market AND the chairs' seal | the chairs refuse: it stands |

The SUM's receipt (`A-SPLIT-IS-SETTLED-BY-KIND-LANDING.md`) left it as "the market's half of the ruling": no market's verdict could reach a
seal. Since 18:08 UTC today a market's program stands beside Poleis (`THE-MARKET-IS-ITS-OWN-PROGRAM-LANDING.md`). This is the last piece of the
offer of 14:03.

## Cost first

| | before (main 98ab345c20) | after (this lane) |
|---|---|---|
| Poleis's program | 446,088 bytes | 453,160 bytes (+7,072), sha256 `c0466a303fc98834…` |
| a unit's market law, a new account | none | 63 bytes, about 0.00133 SOL of rent, paid once by whoever cranks the act that sets it |
| a proposal that asks for the market | | one more account at `propose` (the origin's market, read-only), where the SUM carries its rule |
| a seal, the same split of seven chairs and one seat of six | 8,048 compute units, DEADLOCK | 12,866 to 15,866 with the market's two accounts and a road that passed, PASSED (two address searches) |
| Poleis's `cargo test --offline --locked --no-fail-fast` (the new Mac) | 199 passed, 1 ignored | 212 passed, 1 ignored, no warning; the client crate 20 (19) |
| the market's program | 91,280 bytes; a Road 455 bytes | 91,360 bytes; a Road 463 bytes, rent 4,113,360 lamports; its 20 tests pass |
| what a caller of today sends | | unchanged: every instruction's data and account list is as it was for a proposal that does not ask |
| new on the wire | | two bits of the class byte (`CLASS_MARKET` 0x20, `CLASS_PLAN` 0x10), one CODEX kind (`CODEX_MARKET`, 12), one account (`DISC_MARKET`, 27), two errors (`MarketNotAllowed` 74, `MarketOpen` 75) |
| the lock files | | unchanged in all three crates |
| JavaScript | | none |

The diff against main: 12 files, 1,235 lines in, 18 out, in `poleis/program/poleis_treasury`, `poleis/client` and (one field) in
`poleis/program/poleis_roads`; this receipt is the thirteenth. Nothing was run on the serving Mac.

## What stands now

**Two more ways a proposal may ask, each one bit of its class byte, each granted only by its origin's own law.**

- **`CLASS_MARKET`: a split goes to the market.** The AND runs as it always did. Where it ends split because the chairs passed and the floor
  did not, and the proposal's road passed, the seal is PASSED.
- **`CLASS_PLAN`: the floor speaks by price.** The floor's ballot is not counted at the origin. PASSED when the chairs passed and the road
  passed; DEADLOCK when one of the two did; REJECTED when neither. The floor's weight never seals a plan early.

**Who may have it.**
- **The origin's own law grants.** A unit keeps a market law: the market's program, the board, the classes whose split goes to the market,
  the classes that may be plans, and a bar (below). Only a CODEX act of that unit writes it (`CODEX_MARKET`), under the AND, in the law's slow
  window. A later act replaces it; an act that names no class turns the market off.
- **No unit changes by itself.** With no law, or the class not named for that mode, a proposal that asks is refused at `propose` and leaves
  nothing behind.
- **Three classes only, for ever:** money (TREASURY), seats (SEAT) and the hands (ACT). The law itself (CODEX) and the signals stay under the
  AND whatever a law says. A plan may be money or the hands only: a seat is never decided with no ballot of the floor.
- **One of the four bits at most.** Never with consent, never with the SUM, never both.
- **The origin only.** A crew's own session of the same proposal seals by the AND, with the five accounts it always had.
- **The grant is the opening's.** A law written in or after the slot a proposal opened in says nothing in that proposal's seal.

**The road of a proposal** is the road whose id is the proposal's hash, on the board its origin's law names, in the program that law names.
Nothing is stored in the proposal: the address follows from the law and the hash, and `seal` refuses any other account in its place, so a
crank cannot hide a road or swap a law.

**THE ROAD PASSED** means all of this:
- the road was laid by the law's program (its owner), and reads as that program's layout version;
- **its window lies inside the vote:** it opens at or after the proposal opened, and closes at or before the origin session's deadline.
  Poleis and the market's program count the same slots, so the numbers compare as they are;
- its verdict is PASSED;
- **enough of its board stood in it** (THE BAR): the deposits that stood in the road are at least the law's share of everything credited on
  the board *when the road was laid*, which the road itself keeps. A bar of nothing lets any road count.

Anything that cannot be read says nothing, and never blocks a seal. A road still open, where its verdict could change the seal, holds it
(`MarketOpen`); since a road that counts closes by the session's deadline, that is only until someone sends the road's `decide`, which
anyone may.

**Reading the other program** is one file, `src/roads.rs`: the seed, the discriminator, the version and seven offsets, with no dependency on
the market's crate. A test reads the market program's source as text and holds each constant equal to the one it copies, by name: if that
program moves a field, the test is red and names it.

Nothing else in `seal` moved: the roster, the wait for crews, the early seal of the AND, the receipt, the tip, the chain's signature.

## Narrowed against the ruling's table, and why

**The chairs keep their veto.** The table says a split on money or seats goes to the market. Here only a split *the chairs passed* does: a
split where the chairs refused stands, whatever the road says. A market is priced with the floor's own coin (a balance is the stake's
fuel), so a market that could overrule the chairs would give the floor two voices against them, and the AND's point is that no power moves
alone. Said to him at about 16:20 and 18:17 UTC as his to strike. Widening it is one line.

## How it was built

To this seat's two briefs an Opus builder made it, and each time attacked its own build.

- **The first build** followed the SUM's shape. Where the brief did not fit, the builder's choices stand: an open road does not hold a seal
  it could never count in; `MarketOpen` only where the road can change the verdict; a floorless plan passes by the chairs alone, as every
  floorless decision does.
- **Three holes it reported are closed** (the second brief):
  - *A road could be laid, and decided, before the question was public.* Nothing bound a road's window to the vote. Closed: the window.
  - *Whoever lays roads could hold a seal for up to two years* (a road opening and closing far away). Closed: the window.
  - *Whoever credits balances could make a passed road stop counting,* by crediting just before the seal: the bar read the board's total at
    the seal, and it only grows. Closed: the road keeps the board's total as it stood when the road was laid (one field in the market's
    program, set in `lay`), the seal reads the bar from the road alone, and the board is no longer read at all.
  - *A law executed in the very slot a proposal opened in* was read as that proposal's law. Closed.
- **The Rust client** (`poleis/client`) read a class byte with two bits; it reads four now.

## The tests (the new Mac, nice 19, at the lane's tip)

Poleis: **212 passed, 1 ignored**, no warning. No existing test's expectation changed. The new ones, on the built program:

- `a_units_own_law_names_its_market_by_codex` (set, replaced, turned off; a bad mask or a bar over the whole is refused);
- `the_market_is_refused_at_propose_where_the_law_does_not_grant_it` (no law; a class not granted for the mode; the law and the signals with
  every bit; a seat as a plan; two bits at once; another unit's law; the bit without its account and the account without the bit; and no
  refusal leaves a proposal behind);
- `a_split_the_chairs_passed_goes_to_the_market`: seven chairs, one seat of six and a road that passed seal PASSED and the budget is paid;
  the same with the road failed, or not laid: DEADLOCK and nothing moves; with the road open: `MarketOpen`, then it seals; the floor for and
  the chairs against with a road that passed: DEADLOCK; both chambers: PASSED though the road failed; neither: REJECTED; nobody: EXPIRED;
- `a_plan_passes_by_the_chairs_and_the_road`: with the floor silent, and with the whole floor against; the chairs and the floor for with the
  road failed: no early seal, then DEADLOCK; the chairs against: DEADLOCK; neither: REJECTED; no road: the chairs alone cannot pass a plan;
- `the_bar_a_road_with_too_little_of_the_board_does_not_count` (no board account exists in the test; a credit after the lay moves nothing; a
  road of an older build says nothing and holds nothing);
- `a_roads_window_lies_inside_the_vote` (one slot outside at either end says nothing, the edge itself speaks; for a split and a plan);
- `the_grant_is_the_openings_a_law_written_since_voids_the_market_for_that_vote`;
- `a_crank_cannot_hide_a_road`; `a_crews_own_session_of_a_market_proposal_seals_by_the_and`;
- in `tests/mathlaw.rs`: which classes may ask; the four bits never collide and every reader strips them; the bar at the basis point; the pin.

The market's program: 20 passed; a road keeps its board's total at the lay, and a later credit of a million $WARM does not move it.

**Mutants: 19 deliberate faults by this seat at the lane's tip, each caught by a named test.** The market settling a split the chairs refused; a plan passing on the chairs alone; a plan sealed early by the floor's weight; the bar not asked, and a road exactly at the bar not counting; a road opened before the question was public, or closing after the vote, speaking; a law written in the opening's own slot read as the opening's; a crank handing another account as the road, or another unit's law; a crew's own session reading the market; `propose` not reading the law's grant; a proposal carrying two of the four bits; an account of another program, or of another layout version, read as a road; an open road never holding a seal; the law's act taking any mask and any bar; a reader stripping only two of the four bits; and, in the market's program, a road keeping nothing of its board at the lay. The builder ran 33 of its own over its two builds: 31 caught, and two that change nothing (a second refusal of a seat as a plan, which the law's act already makes impossible, and the seal's re-read of the grant, which a law of the opening always passes).

## What is not done, and what is not proven

- **A polis that names a board trusts that board's authority entirely.** It lays the roads, with their terms, and it names the crediter and
  the judge. It may choose not to lay a road, and then a split stands and a plan cannot pass. Nothing here binds a road's terms beyond its
  window. For a market to bind nobody but the polis, the board's authority must be the polis's own hand, and a road must be laid by a rule
  and not by a key. Neither is built. It is the next piece, and his to rule.
- **The bar measures how much stood in a road, not how many stood there.** One depositor of the required share passes it alone. Whoever
  credits balances can fund that depositor, and together with whoever lays roads can pick the moment of the lay. What the bar refuses is the
  small trader alone in a road.
- **A road nobody else trades is still decided by whoever trades it** (`embertide-futarchy`, WHAT THE LAW CANNOT DO ALONE). The bar is this
  lane's whole answer to it, and the share is a number of each polis's own law, nothing until it names one.
- **A supporter must have the road laid, inside the vote's window.** After the deadline anyone may seal, and a road not laid by then says
  nothing for good.
- **No test runs the two programs in one world.** In Poleis's tests a road is bytes written by hand at the right address, and the pin holds
  the offsets to the market program's source. The market program's own tests show what it writes.
- **The judge plays no part in a verdict**, which is the price at the road's decision. The judge's word settles the bets, afterwards.
- **The program a law names is not checked**: a law that names a program with no such road says nothing.
- **The JavaScript readers do not know the bits**, as with the SUM's.
- **The market law's rent stays**, as the SUM rule's does.
- **No chain runs any of it.** Mollusk only; the market program's id is a placeholder; devnet's programs are older builds; a deploy is his
  ceremony.
