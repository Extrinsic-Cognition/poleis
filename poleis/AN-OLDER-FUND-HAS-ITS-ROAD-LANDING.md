# AN OLDER FUND HAS ITS ROAD (Nov 24 2026)

Devnet's Poleis (`AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab`, Sept 6 2026's build) keeps a fund of 706 bytes. This tree's program asks for 778
(`ix/fund.rs` `fund_present`: any other size is `LayoutVersion`, on purpose, THE-AUDITS-SMALL-CURES-LANDING.md finding 4: "a program that holds an
older one needs its own road for it"). The rehearsal of the evening (`ceremony/the-upgrade-rehearsed-on-every-devnet-swarm-2026-11-24.json`, on
`lane/the-upgrade-rehearsed-on-every-devnet-swarm`) found it: after an upgrade every fee site fails on every swarm (`claim_seat`, a sale's
`settle`, `buy_seats`, an admission), the fund's own constitution too, so no decision of any swarm could repair it. NO-GO.

This lane is that road: one instruction, `grow_fund` (tag 57), that grows the fund at `FUND_ADDRESS` from the older layout to this one, and
nothing else.

## The cost first

| | before (main 05d1e52f30) | after (this lane) |
|---|---|---|
| the program, `cargo-build-sbf --arch v3` | 438,744 bytes, sha256 `1b79ac07…c07ec2` (the first rehearsal's build, reproduced) | 445,744 bytes, sha256 `6501d3fa…b6c5d8` |
| added to the program | | **7,000 bytes** (1.6 %) |
| devnet's program data to extend at the upgrade | 98,744 | **105,744** (at devnet's 5,080 lamports a byte: 537,179,520 lamports) |
| `grow_fund`, with a current option (devnet's case) | | **4,427 compute units** on the validator, 4,436 in Mollusk |
| `grow_fund`, nothing current | | 2,347 |
| a refusal | | 121 to 975 |
| what the growth costs the payer on devnet | | 365,760 lamports (the 72 bytes' rent) and one signature's fee |
| a fee site on the older fund after the upgrade | refused, custom 21 | lands (`claim_seat` 18,913 units, 25,000 lamports to the fund) |

Per frame, per tick: nothing. It runs once in the fund's life.

## The older layout, and where each field goes

Read from git and from devnet's own bytes. The fund came in at `82eaaf2` (Sept 6 2026 19:08 +0100, `~/code-ec/poleis-protocol`, "fund: the polis
of poleis, on chain"); its struct ended at the twelve chairs, `SIZE = OFF_CHAIRS + PENTAGONS * CHAIR_LEN` = 130 + 12 × 48 = 706, the same header
(disc 17 at byte 0, layout byte 8 at byte 1), the same `PENTAGONS` (12) and `FUND_BUMP` (255). Devnet's fund at slot 525,947,015 (one
`getMultipleAccounts`) decodes by it exactly: disc 17, layout 8, bump 255, fee 250 bps, a cycle of 1,800 slots started at 494,190,058, `cycle_seq`
1, `total_in` 1,350,000, `total_out` 375,765, three units staked (`wp_sum` 11,000), a range of 2,000 to 8,000 bps, an option fee of 1,000,000, a
chair quorum of 1, one current option `637592d2…996740` since 494,189,263, every chair empty.

| bytes | field | in the grown fund |
|---|---|---|
| 0 to 705 | header, bump, fee, cycle, totals, stake sums, range, option fee, quorum, current, since, the twelve chairs | **the same bytes at the same offsets**: this layout only appended |
| 706 to 737 | `current_recipient` | **copied from the current option**, read at its own address (`["option", hash]`, its bump) and named by its own hash (`sha256(domain, recipient, manifest)` is recomputed and must be `current`); empty when nothing is current |
| 738 to 745 | `owed` | **0** |
| 746 to 777 | `owed_to` | **empty** |

Why zero is right for the two: the older build paid the current option straight from the fund when a cycle ended (`settle_cycle` at `85ef97d`:
`move_lamports(fund, recipient, amount)`, with the option and the recipient in the transaction); it never booked a grant, so nothing is owed and to
nobody. Why `current_recipient` is not zero: devnet's fund has a current option, and this layout books each cycle's grant to
`current_recipient`; left empty, the roll would book every later grant to the all-zero address (the System Program), where `claim_grant` could
never pay it, and the one owed slot would hold forever. The option devnet holds (`9ixbFkaQ…5G1`, 148 bytes, this layout's `Candidate`) is that
current one.

## The instruction

`grow_fund` · data none · accounts `[payer (s,w), fund (w), system, option (only when the older fund has a current option)]`

It acts only on: `FUND_ADDRESS` (else `InvalidPda`), an account this program owns (else `InvalidAccount`), exactly 706 bytes, disc 17 and layout
byte 8 (else the new `NotAnOlderFund`, 77: a grown fund is one of them, so a second call is refused), bump 255 (else `InvalidPda`), old fields a
constitution could have written (fee and range within the whole, the range the right way up, a cycle between the floor and the ceiling, a quorum
of 1 to 12; else `NotAnOlderFund`), and, when an option is current, that option at its own address under its own hash (else
`NotEnoughAccountKeys` or `InvalidAccount`). Anyone may send it. The payer pays the rent of the added bytes and only that (`rent(778) −
rent(706)`), so the fund's free balance, and every grant from it, is what it was; no lamport leaves the fund. One exception, for copies: a fund
under even the older size's minimum (devnet's bytes laid on a validator of the default rent, 6,960 a byte, as the rehearsal does) is lifted to the
new size's minimum, since a rent-paying account may not change its size; on a chain where the fund was exempt the two sums are the same. No
cycle rolls, no receipt is logged: nothing else is written.

Files: `program/poleis_treasury/src/ix/fund.rs` (`grow_fund`), `src/state.rs` (`Fund::OLD_SIZE`, `Fund::read_older`), `src/lib.rs`
(`IX_GROW_FUND` = 57 and its dispatch), `src/error.rs` (`NotAnOlderFund` = 77), `client/src/lib.rs` (`grow_fund(payer, current)`), and the
walker's step `seal/examples/wish.rs` `fund [--grow]` (the fund read, the growth simulated, and with `--grow` sent; on devnet with `--payer`).
No line of JavaScript.

## The checks

- The crate's whole `cargo test`, once, at nice 19 (Nov 24 2026 23:11:50 to 23:12 UTC): **234 passed, 0 failed**, 5 ignored (the live-validator
  ones), in 11 targets.
- The four new tests, `tests/council.rs`:
  - `an_older_fund_grows_in_place_once_with_its_current_option_and_every_fee_site_and_grant_work_on_it`: a fund with a fee, a vetted and backed
    current option and a second option is cut to its first 706 bytes; `buy_seats` is refused (`LayoutVersion`); the growth without the option
    and with the other option is refused and changes nothing; grown, the account is byte for byte the fund this layout keeps, the payer paid the
    72 bytes' rent, the free balance is what it was; a second growth is refused; the same purchase then lands with its 250 bps; the cycle rolls,
    the grant is booked to the option's recipient and its pull pays it.
  - `an_older_fund_with_nothing_current_grows_and_the_root_constitutes_it_again`: CODEX Fund refuses the older fund; after the growth the same
    sealed decision executes, its parameters change, its balances and cycle do not.
  - `a_growth_is_refused_on_anything_but_the_older_fund_and_changes_nothing`: 705 and 707 bytes, a grown fund, another owner, another
    discriminator, layout byte or bump, a quorum of 0 or 13, a share or a range past the whole, a cycle of no slots, the older bytes at another
    address, an unsigned payer, data where there is none: each refused, the fund and the payer untouched.
  - `devnets_own_older_fund_grows_every_old_value_in_place_and_its_grant_is_booked_to_its_current_option`: devnet's own 706 bytes and its option's
    148, under devnet's rent (5,080 a byte): grown with every old byte in place, the recipient copied, nothing owed, 365,760 lamports in; the
    cycle that ran out on Sept 6 rolls at the first touch and books 3,666 bps (11,000 / 3) of the 974,235 free, 357,154, to that recipient, and its
    pull pays them; then the same bytes on a dearer rent are lifted to the new minimum.
- The house gates run here: `tools/any-room/any-room.test.mjs` 12 of 12, `tools/embertide-mesh/tests/the-javascript-only-shrinks.test.mjs` 38 of 38.

## The rehearsal on a copy of devnet

The receipt: `ceremony/an-older-fund-has-its-road-rehearsed-on-a-copy-of-devnet-2026-11-24.json`. The first rehearsal's own scripts (read.mjs,
tx.mjs, birth.mjs, upgrade.sh, kept in the session's scratch, not in the tree), on a validator of this Mac warped to devnet's slot 525,948,861 with
devnet's features, every one of the 248 accounts devnet's Poleis owns that minute (six more than at 22:34: a swarm born on devnet since, and a claim
at the fund), devnet's own program at its id under a stand-in upgrade authority, a stand-in payer; GETs alone to devnet.

- before: the copy reads as devnet does, every swarm and every account (18 roots, 10 on the wall).
- the upgrade in place to this lane's build (extended by 105,744, read back byte for byte): 0 of 248 accounts changed, every swarm the same; a
  newborn goes through birth, bid, reveal and settlement and is refused at its first hexagon's claim, custom 21, at the fund.
- the growth, by the walker (`wish fund --grow`): 4,427 units, the fund 706 to 778 bytes with every old byte in place and the recipient
  `GyYcAmnL…LEAf` copied; the page's client reads it the same but for the recipient and the lamports. A second growth: custom 77.
- after: the stuck newborn claims its hexagon (18,913 units, 25,000 to the fund) and ratifies; a second newborn goes through birth, sale, claim
  and ratification; 22 of the first rehearsal's 23 ordinary transactions land (each wall swarm's `sync_endowment` and floor, the new swarm's too,
  the two knocks) and `settle_cycle` is refused `CycleOpen`, rightly: the newborn's claim had already rolled the cycle on touch (seq 1 to 2).
- every swarm devnet holds reads the same at the end as at the start, but the two knocks at the two doors that stand, and of the 248 accounts the
  fund alone changed. The validators were ended; none is left running.

One difference of the copy, said: the validator's rent is its default (6,960 a byte), not devnet's (5,080). Under it devnet's fund is below the
older size's minimum, so the growth lifted it to the new minimum (the payer paid 1,094,805, not 365,760) and the roll found no free balance to grant.
The devnet case, the 365,760 and the 357,154 grant, is the crate's test on devnet's own bytes under devnet's rent.

## The real upgrade (not done here: devnet got reads only)

GO, once this lane is on main: build the program from main with `cargo-build-sbf --arch v3` and check its bytes are the build the rehearsal ran
(or rehearse again), then, in one sitting, the growth right after the upgrade:

1. **The extend**: `solana program extend AJds… 105744` (at least 10,240 at a time: the loader refused 168). The rehearsal's extend needed no
   upgrade authority. Signs: **any payer**, who pays 537,179,520 lamports of rent at devnet's rate and a fee.
2. **The buffer**: `solana program write-buffer` of the build, then `set-buffer-authority` to the program's upgrade authority. Signs: **a payer**
   (the buffer's rent, about 2.27 SOL, comes back at the upgrade) and the buffer's authority.
3. **The upgrade**: `solana program upgrade <buffer> AJds…`. Signs: **the program's upgrade authority**, on devnet his devnet wallet
   (`3nJn2…KMba`): his key by his hand (Touch ID through the house's asker), never a seat's; once the authority is moved to the polis, a sealed
   CODEX_UPGRADE of the swarm instead.
4. **The growth**: `cargo run --example wish -- fund --cluster devnet --rpc https://api.devnet.solana.com --payer <a payer's keypair> --grow`
   (it simulates first and sends only what would land). Signs: **anyone**, a crank, who pays the 365,760 of rent and a fee. Between the upgrade's
   slot and this one every fee site refuses (no money moves, nothing is lost); the two cannot share a transaction, since a program upgraded in a
   slot runs its new code only from the next.

After it: `wish fund` again says "not the older fund", and the first newborn's claim and ratification land as on the copy.
