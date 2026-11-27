# A ROAD IS LAID BY A RULE: no key picks a road's terms, whoever asks the market stakes its pool, and a polis names only a board that is its own (Nov 20 2026, lane/a-road-is-laid-by-a-rule, claude-81203e94)

## His words, UTC

- **Nov 20 19:10:48,** to "Build roads laid by a rule next, so no key picks a road's terms?": "Yes, build it".
- **Nov 20 14:03:54,** to "Put this chat on the market's mechanism while you do the narrative there?": "Yes, the market".
- **Nov 20 14:26:19:** "why u asking build or change? u think u can design better?"
- **Oct 29 21:11:11,** asked "Widen the polis's hands on devnet, sealed by your Keystones?": "Yes, widen them". The reason written that day
  (`THE-HANDS-WIDEN-LANDING.md`), which this lane stands on: an authority is the double seal, never a key.

The gap it closes is the first line of what the last receipt left undone (`A-VERDICT-REACHES-THE-SEAL-LANDING.md`): "A polis that names a
board trusts that board's authority entirely. It lays the roads, with their terms, and it names the crediter and the judge."

## Cost first

| | before (main 80e5e1bb91) | after (this lane) |
|---|---|---|
| Poleis's program | 453,160 bytes | 456,800 bytes (+3,640), sha256 `aa65f59df1d3e91a…` |
| the market's program | 91,360 bytes | 98,480 bytes (+7,120), sha256 `d606b26d5512a87a…` |
| a Board | 115 bytes | 207 bytes, rent 2,331,600 lamports: its law (84 bytes) and what its house earned (8) |
| a Road | 463 bytes | 463 bytes: one field renamed (`house` to `staker`), none moved |
| laying a road by rule (`lay_for`) | | 9,069 to 12,069 compute units (the address searches vary) |
| a board's law (`set_law`) | | 2,797 by a key; 17,143 through a polis's own hand, a sealed act |
| naming a crediter and a judge through the polis's hand | | 16,417 |
| the `CODEX_MARKET` act, with the board it now reads | | 24,349 |
| `collect` | two accounts | three (the board): 6,409 |
| a seal | | the same accounts, and the same fields of the road, read for an open road too |
| the tests, `cargo test --offline --locked --no-fail-fast` (the new Mac) | the market's program 20; Poleis 212 passed, 1 ignored; the client 20 | 26; 216 passed, 1 ignored; 20; no warning |
| new on the wire, the market's program | | two instructions (`set_law` 11, `lay_for` 12); four errors (30 to 33); `init_board` names the authority; `set_board` names two roles, not three; `collect` takes the board |
| new on the wire, Poleis | | the `CODEX_MARKET` act takes the board it names, when it turns the market on; one error (`MarketBoardNotOurs`, 76) |
| the lock files | | unchanged in all three crates |
| JavaScript | | none |

The diff against main: 20 files in `poleis/program/poleis_roads` and `poleis/program/poleis_treasury`, 1,728 lines in, 145 out; this receipt
is the twenty-first. Nothing was built or tested on the serving Mac. Nothing is deployed: the market program's id is still a placeholder.

## What stands now

**The road of a proposal follows from the polis's own law and from the proposal. Nobody picks anything about it.**

- **A board's authority is named once.** Whoever opens a board names its authority and only pays. The crediter and the judge start as the
  authority, and only the authority names them. Nothing changes a board's authority afterwards. So a board whose authority is a polis's own
  hand (its agent key, which only a sealed act of the polis moves) was always the polis's: its law, its roles, every credit on it.
- **A board keeps its own law for roads** (`set_law`, the authority's signature): the program and the unit whose proposals open roads here,
  the step, the lag, the threshold, the fee, the pool, and how long the judge has. It refuses what the market's law would refuse. A road
  keeps the terms it was laid with.
- **Any holder with the stake free on the board lays the road of a proposal that asked** (`lay_for`). The program reads the proposal and
  its origin's session from Poleis by owner, discriminator, version and offset, with no dependency on Poleis's crate, and a test pins every
  offset to Poleis's source by name. The road's id is the proposal's hash and its terms are the board's law.
- **The road's window is the vote's, whenever the road is laid.** It opens at the slot the proposal opened and closes at the session's
  deadline, as Poleis's own late `convene` runs "from the proposal's opening, as if it had opened with it". The time before the lay stands
  in both branches at the pool's opening price. So a road laid late has already said "no change" for the time it was not there, and the
  moment of the lay changes nothing a road says.
- **Whoever asks the market stakes its pool, and a road's gain is the house's.** The pool comes from the layer's own balance. At the
  collect the layer gets back what is left, up to its stake and no more; the rest is added to the board's `earned`, the house's. Asking
  the market can cost the asker (the pool's loss, after the fees it took) and never pays. The house can gain and never loses. Whoever
  pushes a road he staked himself pays the house for it. No instruction moves a balance whose owner did not sign.
- **A polis names only a board that is its own.** The `CODEX_MARKET` act that turns a market on reads the board and refuses
  (`MarketBoardNotOurs`) unless it is the named program's board of that number, its authority is the unit's own agent key, and its law
  names Poleis and that unit.
- **The market never decides its own law.** A proposal that asks the market, or is a plan, is refused at `propose` when it is a raw act
  into the market's own program, or one that lists the polis's own board (which also stops a relay through a third program for everything
  that reads the board). Such an act stays proposable under both chambers, as the codex is.
- **A road that cannot count never holds a seal, and a road nobody stood in says nothing.** A decided road counts only with deposits in
  it, whatever the bar. An open road past its close with nothing in it, or under the bar, holds no seal.
- **THE BOOK** has one more term: every balance, what is left in every road not collected, and what the house earned, together, are what
  the crediter credited, to the micro. The bar is measured against what was in hands at the lay: what was credited less what the house had
  earned back.

## What the first build got wrong, and it was this seat's design

The shape told to him at 19:1x UTC said two things that did not survive the first build's review. Both are corrected above.

- **"The polis's own balance on the board stakes its pool."** With the rule moving the house's balance and nobody signing, any member
  could ask and anyone could lay. The builder showed the house's balance could be locked road by road. It was worse than a lock: every
  road opens at the law's one price, so whoever knows the receipts trades the pool there and is paid from the stake at the settle. The
  house's balance was a faucet any member could turn. Now nobody's balance moves without its owner's signature.
- **"It opens at once."** Then the layer's timing picked the window: a road laid a few steps before the deadline was a fresh, short window
  at full power, with nobody given time to answer. Now the window is the vote's.

Three more, found in the same review:

- **A board could change hands.** A key could open a board, write its law, name its own crediter and judge, credit a whale, and hand the
  board to a polis's hand one slot before that polis named it. Closed: a board's authority is named once.
- **The market could have decided its own law** (the builder's): the hands may ask the market, and an act of the hands could rewrite the
  board's law or roles. Closed at `propose`.
- **An empty road could hold a seal**, once the window was the vote's: a road laid in the last slot must be carried through its whole
  window before it is decided. Closed in the seal's reading.

## How it was built

To this seat's two briefs an Opus builder made it, attacked its own build each time, and ran its own mutants (27, then 15 more).

- **19:28 UTC, the first build:** green on the three suites, and the whole chain in one world. It did not land (above).
- **19:59 UTC, the second:** the five changes. Where the brief did not fit, the builder's choices stand: `collect` reads the board to add
  to its `earned`; "an open road before its close holds the seal" is built and no test can reach it, because a seal where the road matters
  always comes after the deadline and a road that counts closes by it.
- **This seat** read every changed line, ran the three suites itself on the new Mac at the lane's tip, corrected the comments that still
  described the first build (no line of code), ran its own mutants, and added the four test cases they asked for.
- **20:22 UTC, the lander refused the first hand, and it was right.** His law of that afternoon, THE HOUSE SIGNS POST-QUANTUM ONLY, counts
  every line that names a classical signature against a baseline that only shrinks. The one-world test stood in a file of its own, with its
  own copy of the voters' signing key: one new line. By his design Poleis's ballots are not among the things that stay classical (what
  carries authority on Solana is owed Falcon-512 in a program), so the line was not marked as excused. The test moved into
  `tests/council.rs`, onto that file's own voters: the same chain and the same numbers, 192 lines shorter, and no line that names a
  classical signature added anywhere. The market's program is loaded there by its bytes, never through `SBF_OUT_DIR`, which the other
  tests of that binary read as they start. The post-quantum gate and the law's table are green on the lane's tree.

## The tests (the new Mac, nice 19, at the lane's tip)

The market's program: **26 passed** (20 before). Poleis: **216 passed, 1 ignored**, no warning. The client: 20, untouched.

**THE TWO PROGRAMS IN ONE WORLD** (in `poleis_treasury/tests/council.rs`, on that file's own council and voters): Poleis and the market's
program, both built from this tree and loaded in one Mollusk world, with no hand-written road or board anywhere. No earlier test ran both.

1. Someone opens a board naming the root's agent key as its authority. The opener can name no role afterwards.
2. The polis, by two sealed acts signed by its own hand through a real cross-program call, sets the board's law and names a crediter and a
   judge.
3. The polis names its market (`CODEX_MARKET`): the board is its own, taken. A plan asking to set that board's law is refused.
4. The crediter credits a staker and three traders. The house is credited nothing and never has a balance.
5. A budget of 0.5 SOL asks the market; the chairs pass it, the floor does not.
6. A stranger with no balance cannot lay its road. The staker lays it from its own balance. The traders deposit and trade PASS up while the
   clock moves through the window.
7. The road is decided PASSED, the seal reads it, and `execute` pays the budget.
8. The same again with the traders buying FAIL: the road fails, the seal is DEADLOCK, nothing is paid.
9. The judge says the receipts, the positions settle, each stake comes back, and the book closes.

What it printed: "verdicts [1, 2] (1 passed, 2 failed); paid [500000000, 0] lamports; each stake back [2000000000, 2000000000]; the house
earned 4525035894; the book 35000000000 of 35000000000".

The other new tests:

- in the market's program, `tests/rule.rs`: a board's law is set by its authority alone and takes only terms the law takes (fourteen
  refusals, the edges taken, a later law moving no road already laid); whoever lays a road stakes it from its own balance, with no account
  of the authority in the instruction and the house's balance byte for byte as it was; `lay_for`'s refusals, each with its exact error and
  nothing changed (no law; a proposal of another program, of another unit, that did not ask, closed to its tombstone, one byte short, of
  another kind or version; a session of another hash, another unit, already sealed, of another program; at the deadline's own slot and
  after; a vote longer than the law's longest window; another's balance, another board's, no balance, one micro short; twice);
  **the lay's timing changes nothing a road says** (two worlds, one road laid at the opening and one eighty steps into the vote, the same
  deposits and trades at the same slots: the branches are equal field for field, and so are the verdicts; and a push that passes a road when
  held from the opening does not pass it made in the window's last slots); the collect for a road the pool gained in, one it lost in, and
  one nobody deposited in;
- `tests/pin.rs`: the fourteen constants read from Poleis, each held to Poleis's source by name, and a tombstone too short to be read as a
  live proposal;
- in Poleis, `tests/council.rs`: `a_polis_names_only_a_board_that_is_its_own` (nine boards refused, none at the address, its own at another
  address, no account, its own taken, an act that turns the market off naming none); `the_market_never_decides_its_own_law`;
  `a_road_that_cannot_count_never_holds_a_seal_and_one_nobody_stood_in_says_nothing`; and the pin gains the Board's four offsets.

**Mutants.** The builder's own: 27 at the first build and 15 at the second, each caught.

**This seat's: 28 deliberate faults at the lane's tip.**

- **23 were caught at once, each by a named test.** In the market's program: the bar's denominator forgetting what the house earned; a road
  recording the board's authority as its staker; the pool staked from any balance of the board, or from a balance of another board; the
  layer not signing; a road by rule opening a slot after the proposal, closing a slot before the deadline, or having to open at or after
  the clock; the authority's own `lay` opening in the past; a road laid at and after the deadline; a plan's road not laid; the judge given a
  slot more than the law gives; the house earning the whole of what is left, or nothing when the pool gained; a collect paying any balance
  of the board; the board not keeping what the house earned; anyone naming a board's crediter and judge; a tombstone, or a proposal one
  byte short, read as live. In Poleis: the board listed last in an act not seen; an open road holding a seal where it does not matter; an
  open road past its close never holding; a board too short for its law read; a board at another address taken.
- **Four stood, and showed what the tests did not hold.** A law that turns on the market alone, or plans alone, naming a board held by a
  key (every test named a law with both modes on); an act through a relay that lists the polis's own board first (it was always listed
  second); an act that claims more accounts than it lists, read past its end. Four test cases were added, and each of the four is caught
  now. No line of either program changed for it.
- **One stands and is the same program:** the line no seal reaches (above).

The last run, at 9992316cc4 (the last commit that changes a line of a program or of a test): the market's program 26, Poleis 216 and 1
ignored, the client 20, every exit 0, no warning; the two programs' bytes are the ones in the table.

## What is not done, and what is not proven

- **Only a holder with the stake free on the board can ask.** A proposal nobody stakes has no market: a split stands as DEADLOCK and a plan
  cannot pass, as when nobody laid a road. The polis does not pay for its own questions unless its own hand lays the road, by a sealed act
  (`lay`, which stands). His to strike: the other way round is the faucet above.
- **Whoever stakes a road bears the pool's loss.** Every road opens at the law's one price, so whoever knows the receipts is still paid
  for trading the pool there; it comes from the asker now, and it is bounded by the stake. The judge's receipts decide its size. A judge is
  a key the polis names, and its first word is final.
- **A stake comes back only once no position stands in the road:** after the judge spoke and every position settled, or after the judge's
  time ran out and the void settles. The board's law says how long that is.
- **What the house earned sits on the board.** Nothing moves it yet. Where it goes is his open call (who sends the yield).
- **The crediter is still a key the polis names.** It picks who has $WARM to trade and to stake, and it can fund one holder who passes the
  bar alone. The bar measures how much stood in a road, not how many stood there.
- **A road nobody else trades is still decided by whoever trades it** (`poleis-futarchy`, WHAT THE LAW CANNOT DO ALONE). He pays the
  house the fees, and what his push lost if he holds it to the settle. No rule closes that; only another trader does.
- **A road is carried through `window / step` boundaries, 66 a call.** The board's law sets the step; a step far smaller than the polis's
  votes makes every road dear to carry. Nothing checks a law's step against the polis's own window.
- **A judge that is still the polis's own hand** could have its receipts sent by a market-passed act through a relay program (the direct
  call is refused; `receipts` reads no board). Receipts move settlements, never a verdict.
- **Board N is the first opener's to name.** A number opened for another authority can never become the polis's, which then names another.
- **The layout version did not move** though the Board grew from 115 to 207 bytes. Nothing is deployed, so no older board exists.
- **One line no test reaches:** "an open road before its close holds the seal". The window law already rules the case out.
- **The league's roads are not touched.** `lay` stands for a board held by a key. How a rider's own road is laid, and who stakes it, wait on
  his four calls of Nov 19.
- **His to strike, as before:** the chairs keep their veto; the market is its own program beside Poleis.
- **The JavaScript readers know none of this.**
- **No chain runs any of it.** Mollusk only; a deploy is his ceremony.
