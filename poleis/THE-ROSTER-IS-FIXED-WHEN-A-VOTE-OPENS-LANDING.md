# THE ROSTER IS FIXED WHEN A VOTE OPENS: who is counted is who stood there when the vote opened (Nov 20 2026, lane/the-roster-is-fixed-when-a-vote-opens, claude-81203e94)

## His words, UTC

- **Nov 20 ~10:00:** "can you please audit the shit i was doing with solana swarmbook please? the contracts etc? the gvs..."
- **Nov 20 ~11:10,** to "Do I fix the Poleis contract next?": "Yes, the contract".
- **Nov 20 11:24:** "what about the fractal recursivity and the perfect bicameralism?"
- The law it keeps: "if you remove the double seal you break the entire bicameralism, no?" (Sept 2 22:59); "Hexagons are not interchangeable" (Mar 28 13:44).

The audit and the order of its cures: `ec-workspace/docs/devnet-governance-code-hypertrace-2026-11-02.md`, sections 8.2 (P1), 8.9 (P9) and 8.11. This is
the second of its lanes. Why the floor is fixed and the chairs may change hands: `ec-workspace/docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`,
its section of Nov 20 11:24.

## Cost first

| | before (main 832c1dfef4) | after (dc9b445783) |
|---|---|---|
| the program | 433,176 bytes | 438,328 bytes (+5,152), sha256 `bff177fc8c175f7a…` |
| a ballot box (one polis's session of one decision) | 158 bytes whatever the floor; in a society with NFT seats, 167 + 42 a holder row | 179 + 42 a holder row, for every polis: 1,019 bytes at 20 holders, 6,479 at 150 |
| its rent, paid by whoever opens the vote and returned when it closes | about 0.002 SOL | about 0.008 SOL at 20 holders, 0.046 SOL at 150 |
| compute at 150 holders (Mollusk; measured by the lane's builder with a test not kept, not run again by this seat) | propose 13,043; attest of 16 floor voices 23,635; attest of 12 chair voices 37,661; seal 30,946 | propose 33,478; 29,888; 42,598; seal 16,789 |
| the crate's `cargo test --offline --no-fail-fast --locked` (the new Mac) | 164 passed, 0 failed, 1 ignored | 175 passed, 0 failed, 1 ignored (15 s on a warm target); the client crate 19 passed |
| instruction data, account lists, error codes, other accounts' layouts | | none changed |
| JavaScript | | none |

## What was wrong

A vote's ballot box recorded who was on the floor at its opening only when the polis had NFT seats. Every other polis counted the registry as it stood
at the moment of each vote, and judged the chairs as they stood at the moment of the seal.

1. **A seat could vote more than once in one vote** (the audit's P1). A holder voted, waited one window for its seats to stop resting (a vote on the
   code runs three), moved them to a fresh key, and the fresh key voted with the same seats.
2. **The chairs' bar fell during the vote** (P9). The bar was a majority of the chairs lit at the seal, while a yes landed by a chair whose term then
   ended still counted. Five of twelve passed once their own terms lapsed.
3. **Under consent, a crew's no could drop out** (P9). A crew that sealed no was ANDed only while its chair was still lit at the root's seal. Its term
   ended, and the root passed alone.

## What stands now

Every session carries its roster from its opening: each holder row with its seats times weight, the chairs lit, the crews among them, and the slot. Votes,
the chairs' bar, the wait for crews and the consent AND are judged against it.

- **The floor.** A holder votes with what its row held when the vote opened. A holder who arrived after, or who took over a leaver's row, adds nothing in
  that vote and votes in the next. Seats moved during the vote count in the hand that held them at the opening while it still holds any; a hand that
  gave all of them away votes with none, so those seats are silent until the next vote.
- **The chairs.** A chair speaks in a vote only if it was lit at the opening and nobody has seated, reseated or emptied it since. The bar is a majority
  of the chairs lit at the opening, whatever lapses after. A chair that changes hands waits for the next decision.
- **The crews.** A seal waits only for the crews that stood in their chairs at the opening and still do. Under consent it ANDs every crew of the
  opening: one that said no, or can no longer speak, leaves the decision DEADLOCK, to be proposed again.
- **Votes already open on a deployed program** keep their shape and are judged as they always were: the bare 158 bytes, and the NFT societies' older
  body. Three shapes are read; only the roster is written.

## The tests

Six new council tests, after `seats_rest_for_a_window_after_they_speak_or_move`, and five unit tests of the roster's bytes:
`no_seat_votes_twice_by_changing_hands` (a vote on the code, and an ordinary one), `a_seat_bought_after_the_opening_does_not_vote_in_that_session`,
`the_chairs_bar_is_the_openings` (and its slow form), `a_chair_seated_after_the_opening_waits_for_the_next_decision`,
`under_consent_a_crew_that_said_no_still_counts_when_its_chair_lapses`, `sessions_opened_before_the_roster_still_seal`.

The builder ran them against two mutants. With main's vote, mandate and seal code under the new roster, the third, fourth and fifth fail. With bare
sessions restored for a polis without NFT seats, all six fail.

Existing tests told the law, none weakened:
- `seats_rest_for_a_window_after_they_speak_or_move`: two seats handed to a newcomer mid-vote used to count in the newcomer's hand. They count in
  neither now: the newcomer is not on the roster and the giver holds nothing.
- Three rent checks (`rent_comes_home_when_the_ballot_box_is_spent`, `a_closed_proposal_shrinks_to_its_hash_and_the_rent_comes_home`, the convened
  session in `each_polis_opens_its_own_vote_and_a_root_over_thirteen_decides`) compute the ballot box's size from the holder rows.
- The seeded season: S7 takes the chairs' bar from the roster; S10 also holds that every session carries a roster, and that a session names a convener
  exactly when its proposal did not list its unit.

## How it was built and read

Specified by this seat from the audit's lines, written by an Opus builder in five commits, and every changed line read again by this seat. Two
tightenings after the read (dc9b445783): `convene` takes the roster's chairs before it asks whether they spoke, as `mandate` does; and
`Registry::has_nft_rows`, which nothing calls any more, is gone.

## Not done, and why

- **On no chain.** The house's program on devnet (`4627ah…`) runs the build of Oct 29; a deploy is his ceremony.
- **A dividend has the same hole** (P2): its own lane, `lane/a-dividend-is-paid-on-the-cuts-roster`, built, next in line.
- **A vote opened before this law** keeps the old counting until it closes; its window bounds it.
- **`execute`'s belt for a decision on the code** still reads the chairs at the execution: a crew seated after the question was asked blocks that one
  decision, as before.
- **Left by choice:** each floor voice checks the whole body again (about 390 compute units a voice at 150 holders; it could be checked once an
  instruction); a chair seated in the very slot a vote opens counts as standing at the opening.
- **The fund's stake** (P5) is the same family and is not touched: how the fund counts a hexagon is his call (the lineage's section of Nov 20 11:24).
