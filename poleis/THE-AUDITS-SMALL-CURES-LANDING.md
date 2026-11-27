# THE AUDIT'S SMALL CURES: a passed decision cannot be thrown away, unit 0 is only ever a root, a weight has a ceiling, and the fund neither owes itself nor reads an older fund as none (Nov 20 2026, lane/the-audits-small-cures, claude-81203e94)

## His words, UTC

- **Nov 20 ~10:00:** "can you please audit the shit i was doing with solana swarmbook please? the contracts etc? the gvs..."
- **Nov 20 ~11:10,** to "Do I fix the Poleis contract next?": "Yes, the contract".

The audit and the order of its cures: `ec-workspace/docs/devnet-governance-code-hypertrace-2026-11-02.md`, sections 8.2 and 8.11. This lane is the
first of four: the cures that change no layout, no instruction and no account list.

## Cost first

| | before | after |
|---|---|---|
| the program | 432,696 bytes (NO SEAT CLOSES A CYCLE's receipt; no commit touched the program since) | 433,176 bytes (+480), sha256 `29a2df2eb1fe621b…` at e919729cde |
| compute | | one or two comparisons more in `close_proposal`, `found_unit`, the SIGNAL Weight arm, `propose_option` and at each fee site. Not measured: no test of this crate prints compute for these paths |
| the crate's `cargo test --offline --no-fail-fast --locked` (the new Mac) | 160 passed, 0 failed, 1 ignored | 164 passed, 0 failed, 1 ignored (7 s on a warm target, at cce9a5785f) |
| layouts, instruction data, account lists, error codes | | none changed |
| JavaScript | | none |

## What was wrong, and what stands now

**1. Anyone could throw a passed decision away before it was carried out** (the audit's P6).
- `execute` needs the origin's seal alone, unless the decision asks for consent. `close_proposal`, which asks for no signer, closed any decision one
  of whose listed sessions had not passed. So a root that passed while a listed crew said no was closed to its tombstone by anyone: its hash burned,
  the vote to be taken again.
- Now a decision closes only when it is spent, abandoned, or dead by `execute`'s own rule: the origin's seal did not pass, or it asks for consent and
  some listed chamber's did not (`ix/proposal.rs`, `close_proposal`).
- The test: `a_decision_its_origin_passed_is_carried_out_before_it_closes`. A crew says no, the root says yes, no consent asked: the close is
  refused, the decision pays, then it closes. A decision whose origin did not pass still closes.

**2. A child could be founded with unit id 0** (P8).
- The gates that ask for `origin == ROOT_UNIT` (the code's upgrade, the fund's constitution, the vetting of an option) take unit 0 for the root of
  the founding tree. On a program id where tree 0 is not founded yet, a stranger's tree could found a child with id 0 and pass them.
- Now `found_unit` refuses the id 0 for a child (`ix/council.rs`). Unit 0 is born only as a root, by genesis.
- The test: `unit_zero_is_only_ever_a_root`, on a world with no tree 0.

**3. A weight had no ceiling** (P8).
- One passed SIGNAL Weight whose seats × weight overflowed made every later seal of its unit fail on the sum, for ever: no seal, so no decision to
  undo it.
- Now a weight above `MAX_WEIGHT` is refused where it would act (`lib.rs`, `ix/proposal.rs`). `MAX_WEIGHT` is `u64::MAX / (150 × 150)`,
  819,855,292,164,868: with at most 150 seats a holder and 150 holders, no sum of seats × weight can overflow. It is arithmetic, not policy: what a
  weight may be below it is still the unit's own decision.
- The test: `a_weight_has_a_ceiling_so_no_seal_overflows`.

**4. The fund could owe itself, and an older fund read as no fund** (P7, P8).
- An option whose recipient is the fund's own address could never be paid (one account is never both ends of a move), and the one `owed` slot would
  then hold back every later grant. `propose_option` now refuses it (`ix/fund.rs`).
- The fund's account grew on Sept 9 (706 to 778 bytes) and the layout's number did not. `fund_present` answered "no fund" for a fund of another size,
  so on a program upgraded in place every fee site would have charged nothing, and said nothing, while the old fund's money stayed where no
  instruction reads it. Now a fund of another size is an error (`LayoutVersion`) at every fee site and at the fund's own constitution: loud, not
  silent. It moves no fund: a program that holds an older one needs its own road for it.
- The test: `the_fund_never_owes_itself_and_an_older_fund_is_never_no_fund`.

## How it was built and read

Specified by this seat from the audit's lines, written by an Opus builder in the lane's worktree, and every changed line read again by this seat
before the hand. No existing test had to change.

## Not done, and why

- **On no chain.** The house's program on devnet (`4627ah…`) runs the build of Oct 29; a deploy is his ceremony.
- **The other cures are their own lanes** (section 8.11 of the record): the roster fixed when a vote opens (P1, P9), a dividend paid on the cut's
  roster (P2), a vote that names its program and its payload (P3, his word first: it changes the wire), the fund's stake (P5, his design).
- **Left by choice:** seats resting longer (a stopgap the roster lane makes needless); the crank's tip (bounded by the plan); the squatting of a
  unit's id (its cure changes the wire).
