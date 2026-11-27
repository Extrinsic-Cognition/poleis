# A SPLIT IS SETTLED BY KIND: the SUM, for small things, where a unit's own law turns it on (Nov 20 2026, lane/a-split-is-settled-by-kind, claude-81203e94)

## His words, UTC

- **Nov 20 13:14:50:** "stuff always can change, i need you to focus on the meta-algorithm being the voting engine behind it all.. there's futarchy but
  there is also the perfect fractal bicameralism that holds it all..."
- **Nov 20 13:21:47,** to "A split by kind: AND for the law, SUM for small things, the market for plans?": "Yes, by kind".
- **Nov 20 13:24:17,** to "Build the split by kind into the seal?": "Yes, build it".
- The rule itself, **Mar 22 03:49:47:** "the votes are divided by 50% hexagons and 50% pentagons... but to pass anything they just need the 51%
  compounded (for example, 30% pentagons say yes, 21% hexagons say yes, it passes)"; **05:16:03:** "if 5/12 pentagons vote yes but 80% hexagons voted
  yes, it passes... in this case it's not "AND""; "you should add resolution rule... like "AND" or "SUM" i think...".
- His test of the whole, **Sept 8 04:18:** "its meant to be the most perfect governance ever exactly because of the "AND" and the "SUM" mechanisms and
  the recursive tree structuree".

The engine and the ruling: `ec-workspace/docs/the-holy-goldberg-governance-the-perfect-recursive-bicameralism-hypertrace-2026-11-02.md`, sections 11 and
11.8. The question had stood open since Nov 3 (its section 10, move 5).

## Cost first

| | before (main aa879f5481) | after (1ec15b5881) |
|---|---|---|
| the program | 442,080 bytes | 446,088 bytes (+4,008), sha256 `54508398fd7ed817…` |
| a unit's rule, a new account | none | 20 bytes, about 0.00103 SOL of rent, paid once by whoever cranks the act that sets it |
| a proposal that asks for the SUM | | one more account at `propose` (the origin's rule, read-only); the same proposal account, the same sessions |
| a seal | the AND | the AND, and for a split of an origin that was granted the SUM one comparison more; compute not measured |
| the crate's `cargo test --offline --no-fail-fast --locked` (the new Mac, a warm target) | 184 passed, 0 failed, 1 ignored | 199 passed, 0 failed, 1 ignored, in 14 s; the client crate 19 passed |
| what a caller of today sends | | unchanged: every instruction's data and account list is as it was for a proposal that does not ask |
| new on the wire | | one bit of the class byte (`CLASS_SUM`, 0x40), one CODEX kind (`CODEX_RESOLUTION`, 11), one account (`DISC_RULE`, 26), one error (`SumNotAllowed`, 73) |
| JavaScript | | none |

## What was so

Every decision is a double seal: more than half of the chairs lit at the opening, and PRO weight above half of the floor's active weight. Both is
PASSED, neither is REJECTED, and one of the two is DEADLOCK, which was final. His own test of the governance has three parts, the AND, the SUM and the
tree; the SUM was never written, so a split stood still for ever, whatever was asked.

## What stands now

**The SUM** (`sum_passes`, `lib.rs`). Each chamber is half the vote. A split passes when the two halves reach 51 % and no chamber is under a third:
`100 × (cp × active + fp × lit) ≥ 102 × lit × active`, `3 × cp ≥ lit`, `3 × fp ≥ active`, with `cp` of `lit` chairs and `fp` of `active` weight. Both
his examples pass it. Under the AND a chamber blocks with half of itself; under the SUM it blocks only with two thirds, and a unanimous chamber still
needs a third of the other. With twelve chairs or fewer, whatever passes the AND passes the SUM.

**Who may have it.**
- **The proposal asks.** A second bit of its class byte, beside the consent bit. The byte is stored as sent, and the floor reads it with the payload.
- **The origin's own law grants.** A unit keeps a rule: the kinds that may ask, and a small cap. Only a CODEX act of that unit writes it
  (`CODEX_RESOLUTION`), under the AND, in the law's slow window, closed under its crews. A later act replaces it; an act that names no kind turns the SUM off.
- **No unit changes by itself.** With no rule, or the kind's bit off, a proposal that asks is refused at `propose` and leaves nothing behind.
- **Four kinds only, for ever:** a lot opened, a ruling on a contract, a transfer to a unit, a budget to an address; the two payments only at or under
  the small cap. All of the law (CODEX), every seat act, the hands (ACT), a weight, a dissolution, an allowance, a contract and the fund's signals stay
  under the AND whatever a rule says. The law that sets the SUM can never be passed by the SUM.
- **Never with consent.** Consent is the AND down the whole branch.
- **The origin only.** A crew's own session of the same proposal seals by the AND; its split is its chair's abstention above, as before.
- **The grant is the opening's.** `propose` reads the rule; `seal` reads the proposal. A law changed while a vote is open changes nothing in it.

Nothing else in `seal` moved: the wait for crews, the early seal and the deadline are the AND's.

## Narrowed against the ruling's first table, and why

The table he said yes to named sale defaults and an allowance among the small things. Both stay under the AND here. Sale defaults price the seats of
the floor, so they decide who joins it. An allowance is a standing order, and revoking one is an AND act: given by the SUM it would be easier to give
than to take back. A budget, one payment to one address, takes the allowance's place. His to widen.

## The tests

Seven tests of the arithmetic in `tests/mathlaw.rs`, on the program's own function: his two examples; whatever passes the AND passes the SUM and no
majority anywhere never does (every count of chairs to twelve against every floor to sixty); a unanimous chamber cannot carry a chamber under a third;
one point under 51 % does not pass; more PRO never turns a pass into a fail; no chairs or no floor never passes, and the widest weights do not
overflow; only four of the 65,536 class and kind pairs can take the SUM.

Eight council tests in `tests/council.rs`, on the built program:
`a_units_own_law_turns_the_sum_on_by_codex`; `five_chairs_and_most_of_the_floor_carry_a_small_budget_by_the_sum` (five of twelve chairs and five of six
seats pass and the budget is paid under the plan; the same votes without the bit are DEADLOCK and nothing moves);
`the_chairs_majority_and_half_the_floor_pass_by_the_sum`; `a_unanimous_chamber_cannot_carry_the_other_under_a_third` (both ways);
`neither_chamber_is_rejected_and_silence_expires_under_the_sum`; `the_sum_is_refused_at_propose_where_the_law_does_not_grant_it` (no rule; a kind not
turned on; five kinds that can never take it with every bit set; one lamport over the cap, for a budget and for a transfer; with consent; another
unit's rule; the bit without its account and an account without the bit; and no refusal leaves a proposal behind);
`a_resolution_names_only_the_four_kinds_and_zero_turns_the_sum_off` (and a vote opened under the grant is still settled by it after the law turns it
off); `a_crews_own_session_seals_by_the_and` (a crew with a floor of its own, split the way that passes at an origin, seals DEADLOCK, while the root
passes by the SUM and pays).

**Five mutants, each caught** (the new Mac, the council and mathlaw targets): the seal never sums (4 council tests fail); a crew sums too (1); no
chamber needs a third (1 council, 1 mathlaw); the law's kinds are not read at `propose` (2); the small cap is not read (1).

No existing test's expectation changed.

## What is not done

- **The market's half of the ruling.** A split on money or seats "goes to the market", and a plan passes by "the market AND the chairs' seal". No
  market's verdict can reach a seal yet, so those splits stand as they did. It waits on the market's shape on a chain, his call of Oct 26.
- **A ruling on a contract is not bounded by the small cap.** A unit that turns `SUM_RULE` on lets the SUM release or refund any escrow its AND opened.
- **The small cap bounds one decision.** The plan still bounds the period, for SUM payments as for any.
- **Crews.** A crew cannot be granted the SUM for a proposal it did not originate.
- **Compute.** Not measured for this lane.
- **The JavaScript readers do not know the bit.** `poleis/ceremony/wire.mjs` and the site's Poleis client read a class byte with 0x40 as a class
  they do not know. No JavaScript line was added to teach them (the house's law); they read the September and ceremony programs, which do not have it.
- **No chain runs it.** Devnet's programs are older builds, and a deploy is his ceremony.
- **The engine's other voices.** One number and one assignment still exist, in the Fund alone.
