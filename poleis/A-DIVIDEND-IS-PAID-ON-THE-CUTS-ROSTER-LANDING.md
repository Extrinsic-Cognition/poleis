# A DIVIDEND IS PAID ON THE CUT'S ROSTER: who may claim what is fixed when the dividend is cut (Nov 20 2026, lane/a-dividend-is-paid-on-the-cuts-roster, claude-81203e94)

## His words, UTC

- **Nov 20 ~10:00:** "can you please audit the shit i was doing with solana swarmbook please? the contracts etc? the gvs..."
- **Nov 20 ~11:10,** to "Do I fix the Poleis contract next?": "Yes, the contract".
- The law it keeps: "Hexagons are not interchangeable" (Mar 28 13:44).

The audit and the order of its cures: `ec-workspace/docs/devnet-governance-code-hypertrace-2026-11-02.md`, sections 8.2 (P2) and 8.11. This is the third
of its lanes, the same law as THE ROSTER IS FIXED WHEN A VOTE OPENS (main 278e81f88b), on money.

## Cost first

| | before (main 278e81f88b) | after (126cccb328) |
|---|---|---|
| the program | 438,328 bytes | 442,080 bytes (+3,752), sha256 `fc09a4e1e1183af2…` |
| a distribution (one dividend's escrow) | 162 bytes whatever the floor | 171 + 41 a holder row: 991 bytes at 20 holders, 6,321 at 150 |
| its rent, paid by whoever executes the cut (or cranks the policy's accrual) and returned when it closes | about 0.002 SOL | about 0.008 SOL at 20 holders, 0.045 SOL at 150, held until every share is claimed or about 60 days pass |
| compute (Mollusk; measured by the lane's builder, not run again by this seat) | a claim about 5,450 | a claim on a roster of 3 rows about 5,170; on a roster of 150 rows with the claimant in the last, 7,093 |
| the crate's `cargo test --offline --no-fail-fast --locked` (the new Mac) | 175 passed, 0 failed, 1 ignored | 184 passed, 0 failed, 1 ignored (16 s on a warm target); the client crate 19 passed |
| instruction data, account lists, error codes, other accounts' layouts | | none changed |
| JavaScript | | none |

## What was wrong

A distribution recorded only the floor's total weight at the cut. A claim was marked by the claimant's row in the registry and priced at its seats times
weight at the moment of the claim (the audit's P2).

- A holder claimed, waited one window, moved its seats to a fresh key, and the fresh key claimed again on the same seats, and so on until the escrow
  was empty and the holders who came late were refused. The one bound was the total.
- A key that bought seats after the cut claimed on them.
- A newcomer who took over a leaver's row inherited the leaver's claim.
- A holder who left before claiming lost its share.

## What stands now

A distribution carries, from its cut, each registry row's identity and its seats times weight as they stood then. A claim is paid by that row to that
identity, whatever moved since.

- **A key or a polis** is found in the cut's roster, not in the registry. Seats bought after the cut add nothing; seats sold after it take nothing
  away. A holder who has left the floor since still claims. A key that arrived after the cut, or sits in a leaver's old row, claims nothing.
- **An NFT seat** claims by its own row, which the roster must name. Its share goes to whoever owns the seat now, once.
- **The registry is not written by a claim** any more: the rest a claim imposed on the seats, a stopgap against the second claim, is needless.
- **A distribution cut before this law** (the 162 bytes alone) is claimed and closed exactly as it was, rest included.

## The tests

Six new council tests after `dividends_flow_by_weight_and_dissolution_freezes`, one NFT test, two unit tests of the roster's bytes:
`a_dividend_is_paid_on_the_cuts_roster` (the seats handed down a line of fresh keys; every original holder, claiming last, is paid whole),
`a_seat_bought_after_the_cut_does_not_claim`, `a_holder_who_left_after_the_cut_still_claims` (and the newcomer in its row does not),
`a_token_dividend_is_paid_on_the_cuts_roster`, `a_distribution_cut_before_the_roster_still_pays` (and closes),
`a_claim_finds_its_row_in_a_full_roster` (150 rows), and in `tests/nft.rs`
`a_rostered_dividend_pays_the_seat_the_cut_named_to_whoever_owns_it_now`.

Existing tests told the law, none weakened:
- `dividends_flow_by_weight_and_dissolution_freezes`: bob, who held two seats at the cut and left before claiming, was refused; he is paid now. The
  check that a claim rests the claimant's seats is gone with the rest itself.
- `income_by_rule_and_tokens_to_the_floor`: the rent that goes home at the close is the roster's size.
- The seeded season's D1 measures a distribution's rent on its real length.

## How it was built and read

Specified by this seat from the audit's lines, written by an Opus builder in four commits, and every changed line read again by this seat. After the
read: the NFT seat's path on a roster had no test (the NFT tests laid distributions of the old shape) and has one now (abb00c03cf); main was merged in
after the roster lane landed (one conflict, the two lanes' unit tests in one module). The client's `DistributionView::parse` accepts both shapes: it read only the 162 bytes and would have seen no new distribution.

## Not done, and why

- **On no chain.** The house's program on devnet (`4627ah…`) runs the build of Oct 29; a deploy is his ceremony.
- **The accrual's crank carries more rent.** Cutting the policy's dividend is permissionless; whoever cranks it now locks up to 0.045 SOL until the
  close. If nobody cranks at that price, the cure is a tip from the treasury for the cut, not a smaller roster.
- **A burned or retired NFT seat** cannot claim on a roster, as before: its claim needs a live seat. A key that left can.
- **The client's view** does not show the roster's rows.
- **The fund's stake** (P5) is the same family and is not touched: how the fund counts a hexagon is his call.
