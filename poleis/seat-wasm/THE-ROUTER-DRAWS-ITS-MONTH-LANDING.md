# THE ROUTER DRAWS ITS MONTH: the batch that posts spends to the mine pulls, in its own transaction, what the AI provider charged for them (Nov 24 2026, lane/the-router-draws-its-month, seat claude-5713365b; built by an Opus builder to this seat's design, judged line by line)

## The cost, first

- **What it closes.** THE TREASURY PAYS THE COMPUTE gave the mine's miner a month of the dollar out of root 0's vault (the launch's act
  8). Nothing sent the draw. The router went on paying the AI calls from the house's own credit while the dollars people prepaid sat in
  the treasury.
- **What it costs on the chain.** No program changes. A batch's transaction grows (Mollusk, the real programs):

  | | compute units | bytes on the wire |
  |---|---|---|
  | a one-spend batch alone, as before | 4,196 | 493 |
  | the same batch with the draw | 13,822 to 13,888 | 769 |
  | the first draw, which also makes the miner's dollar account | 27,239 | 811 |
  | live, the door's first batch: a flask's $WARM account made, the miner's dollar account made, the draw | 41,381 | 811 |

  The draw adds 276 bytes to any batch, 318 with the making. So a full batch holds 4 spends of distinct flasks where it held 7, and a busy
  door sends about seven batches where it sent four, each 5,000 lamports of the miner's. The first draw costs the miner 2,039,280
  lamports of rent for its dollar account, once.
- **What it costs the door.** Five more accounts in the read the crank already makes (one `getMultipleAccounts`), nothing sent besides
  the batch, nothing kept, no timer. The built law grows from 476,388 to 492,400 bytes.
- **What it costs the house.** Rust +940 −27 (the law +271 −15, its tests +370, the programs' tests +299 −12). JavaScript +2 −2: no
  net line. One dev-dependency of the program's tests on the law's crate, its lock file 62 lines longer and nothing in it moved.
- **What it does not move.** No layout, no version, nothing on the devnet program `AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab`. Nothing
  is deployed, nothing was sent to devnet or mainnet, no key of his was made, read or asked for, no Canvas tab was born. The live door
  does not change until `SPACE_ROUTER_MONTH` is set on it.

## His words, UTC

- **Nov 23 2026 22:20:46**: "we might get some money in the treasury, but unless we figure out how to allocate it as credits.. u get me?".
- **Nov 24 2026 00:07:00**, "Yes, build it", to this seat's "Build the router's own draw from the treasury next?".
- **Three rules here are this seat's picks**, shown to him on the Canvas (the Claude page, "The router's draw") and in the chat right
  after his yes; he has said nothing on them: the draw is the provider's charge and not the whole spend; the door's markup stays in the
  treasury; what a cut leaves undrawn is not carried.

## The rule

1. **Where it lives.** Inside the transaction of a batch of spends, after the mine's own instruction: Poleis's `pull`, signed by the
   miner as the batch is. A batch that does not land drew nothing. There is no timer, no cursor and no store: the mine's own cursor says
   which spends were posted, and a spend's share is drawn with the batch that mines it or never.
2. **How much.** Each receipt the batch places gives its share: what the provider charged for it, `router_usd` on the receipt the
   router's oracle signed, never more than the dollars that receipt gives its entry. A receipt that says no charge gives nothing. The
   shares are summed and floored to the micro-dollar. The sum is cut to:
   - the micro-dollars the batch's entries post (THE DRAW IS NEVER MORE THAN WHAT THE BATCH MINES);
   - what the month has left this period;
   - what root 0's cap of the dollar has left this period;
   - what the vault holds.
3. **The markup stays.** What the tank paid above the provider's charge (5% today) is never drawn.
4. **THE DRAW NEVER HOLDS THE MINE.** The law adds the pull only when every refusal of the program's `pull` is ruled out by the accounts
   it read, in the program's own order. Otherwise the batch goes alone, exactly as before, and the plan says why in one word:

   | word | what the law read |
   |---|---|
   | `month` | the door's var is not an address |
   | `allowance` | no allowance at that address |
   | `expired` | it ends within 300 slots (a message lives 150, and the clock is read before it) |
   | `registry`, `frozen` | its unit's registry does not stand, or is frozen |
   | `payee` | its payee is not the mine's miner by its key |
   | `mint` | its mint is not the mine's dollar and the unit's default mint |
   | `account` | the miner's dollar account is another's or of another mint |
   | `rent` | the account must be made and the miner holds under 10,196,400 lamports, or its lamports were not read (THE MINER'S RENT) |
   | `vault` | the mine's treasury is not that unit's vault of the dollar |
   | `spent`, `cap`, `empty` | the month, the cap or the vault has nothing left |
   | `nothing` | the batch's spends were charged nothing, or the batch is empty |
   | `room` | the message would pass a packet (the law's own guard; its reserve is measured, so this is never expected) |

5. **`SPACE_ROUTER_MONTH`** is a Worker var as `SPACE_ROUTER_MINE` is: the allowance's address, which the launch's certificate says
   (`router.address`). Unset, the door is byte for byte as it was.

## What this lane changes

- **`poleis/seat-wasm/src/mine.rs`**, the law: `plan_within` (the batch packed to leave the draw's bytes free), `month_of` (the mirror
  of `pull`'s refusals), `draw_of`, `draw_ixs`, `draw_message`, `draw_room` (measured, not guessed), and the `wants` and `plan` asks,
  which take a `month` and answer `draw` and `draw_why`. With no month both answer as before: the old vector did not move.
- **`poleis/seat-wasm/tests/mine.rs`**: seven new tests (the amount, each cut and each period's roll, every word, the first draw's
  making and the packet, the door's asks, the draw against what the batch mines, the miner's rent).
- **`embertide-app/ui/intro/dyson-society/worker/space-router-deposit.mjs`**: two lines changed in place. The crank's `wants` ask gains
  the mine and the month, its `plan` ask the month; the head comment names the var.
- **`embertide-app/ui/intro/dyson-society/worker/poleis-seat.wasm`**: rebuilt by `poleis/seat-wasm/build.sh`, sha256
  a99f56d82aa3b78ab95a2d23aec77228ca68cd281d1034cd2412c9d53c11b424.
- **`poleis/program/poleis_treasury/tests/council.rs`**: `the_router_draws_its_month`, the law's own message sent as one transaction on
  the real programs; and the live checks, where the door's crank draws in its mining transaction and the miner's pull by hand counts from it.
- **`poleis/program/poleis_treasury/Cargo.toml`**, `Cargo.lock`: the law's crate as a dev-dependency of the tests.
- **`poleis/program/poleis_treasury/tests/built/mod.rs`**: one program build at a time. Two tests now load two programs, and in one run of
  the suite their builds failed side by side.

It stands on lane/the-treasury-pays-the-compute (ece8499159, handed and not landed when this was written) and carries it.

## The proof

**The law's own tests** (`poleis/seat-wasm`, 18 passed). Red after the fact, said plainly: the builder wrote the law before its tests, so
the red was shown by mutation. Each of five mutations turned one test red: the packing ignoring the reserve, a share not cut to what it
gives, the cap's period never rolled, the default-mint check taken out, the cut to what the batch mines taken out.

**On the real programs** (Mollusk, `the_router_draws_its_month`): an allowance of 5 dollars a period of 1,000 slots, a cap of 8, a flask's
deposit of 50. Six batches, the vault, the miner's account, the allowance and the cap read after each:

| batch | what happened |
|---|---|
| A | no month named: lands alone |
| B | the first draw, 3 dollars, the miner's dollar account made in the same transaction |
| C | cut by the month to 2 |
| D | the month spent: lands alone (`spent`) |
| E | after the allowance's period rolled: cut by root 0's cap to 3 |
| F | the cap spent: lands alone (`cap`) |

**The suites, on d619caf135** (this lane with main 00f072574f and the two-acts lane merged; Nov 24 2026 00:52:04 to 00:56:20 UTC, the new
Mac, nice 19, a load of 63):

| suite | passed | failed |
|---|---|---|
| `poleis/seat-wasm` | 18 | 0 |
| `poleis/press` | 104 | 0 |
| `poleis/program/poleis_treasury` | 230 (5 ignored: the live ones) | 0 |
| `poleis/client` | 25 | 0 |
| `poleis/seal` | 6 | 0 |
| the Worker's deposit, seat and router suites under node | 32 | 0 |
| the-javascript-only-shrinks, the-law-is-the-gate | 31, 8 | 0 |

**Live, through the door's own code** (the launch rehearsal, `LAUNCH_AS_MAINNET=1`).

This seat's run through the job-slot road on the old Mac (his "Use the old Mac", Nov 23 2026 22:08:02 UTC, as that road's far list holds
it): asked 00:52:05 UTC, slot 1 of 2 there after 128 s, exit 0, 2,127 s after it came; the checks 1,752 s, 248 transactions, 19 refusals.
Its tree was d619caf135, the tip the suites above ran on. The certificate:
`poleis/ceremony/launch-press-rehearsal-the-router-draws-its-month-2026-11-24.json` (`checks.treasury.draw`).

- **The door, its `SPACE_ROUTER_MONTH` act 8's allowance, took one flask's deposit of 10 dollars and one spend of 4.20**, of which the
  stand-in provider charged 4.
- **The door's crank posted the spend and drew in the same transaction**: 41,381 units, 811 bytes, one signature, the miner's. It mined
  4.2 $WARM for the flask, made the flask's $WARM account and the miner's own dollar account, and pulled 4 dollars. The miner paid
  4,083,560 lamports: the two accounts' rent (2,039,280 each) and the fee.
- **Root 0's vault went from 10 dollars to 6.** The miner's dollar account holds 4. The allowance counts 4 spent and so does root 0's cap
  of the dollar. The 20 cents of markup stayed in the vault.
- **A second crank sent nothing**: the chain's cursor had moved, so nothing was drawn twice.
- **Then the miner's pull by hand** counted from what the door drew: 4 more (9,624 units), the vault at 2, the allowance at 8. A pull
  of 32.000001, past the month, was refused (`AllowanceBreached`); a stranger's was refused (`Unauthorized`).

Before this lane the same step left the vault at 10: nothing drew.

## What is not done, and whose

- **The router's wallet does not pay the provider.** The dollars land in the miner's own account. Turning them into credit at the AI
  provider is done by hand, and nothing this seat read shows a way for the door to do it.
- **A race the law cannot see.** Between the crank's read and the batch's landing, another draw of the month, another payment out of
  root 0's dollars, or a freeze makes the pull refuse and the batch fail once. The next crank reads the chain again and plans from it.
- **A full batch holds 4 spends, not 7.** One account the pull names is never read for the default mint. Naming one already in the
  message there would save 33 bytes, a fifth spend. Not done: it would leave the client's own builder.
- **An allowance of another unit than root 0** goes alone (`registry`): the door reads root 0's registry.
- **`SPACE_ROUTER_MONTH` on the live door** waits for the launch: the allowance exists only once act 8 executes.

## Since the first hand (Nov 24 2026 01:38 UTC)

Main moved under this lane (THE RITE FREES THE AGENTS: each agent's own mind now votes the launch's acts). The lane it carries is mended
for it (its receipt says how) and merged here. The suites on that merge, 46174fd6b1: seat-wasm 18, press 115, poleis_treasury 230 (5
ignored), client 25, seal 6; none failed; JavaScript still net 0. **The live rehearsal above ran before that landing.** One more, in
which the agents' minds vote the eight acts and the door then draws, was asked at this hand; its result is added here when it ends.

## The agents' minds voted the eight acts (Nov 24 2026 02:59 UTC, lane/the-minds-vote-the-eight-acts)

The rehearsal this receipt owed. Asked 01:39:28 UTC at 26a82abeb2, the tip that landed on main as efe916621f. The old Mac holds no copy
of the agents' model, so the job said it could not start there, then waited 2,603 s for room on the new Mac (the only Mac that serves
the model, on loopback), took slot 1 of 3 at 02:22:52 UTC and ended with exit 0 at 02:59:38: the checks 2,053 s, 248 transactions, 19
refusals. `LAUNCH_AS_MAINNET=1`. The certificate: `poleis/ceremony/launch-press-rehearsal-the-minds-vote-eight-acts-2026-11-24.json`.

- **Each of the three agents, freed by the rite, read both new acts whole and chose PRO on each**, with its own reason. On act 7, agent 2:
  "The treasury's dollar cap matches the params' genesis.dollarCap 60000000 micro-dollars, and it restates the founding's plan cap and
  period as given, so I endorse it." On act 8, agent 3 began: "The miner's allowance is the fuel for the router's AI calls; without it,
  the machine that routes our work to the world halts."
- **The door then drew as before**: 4 dollars in its mining transaction (42,881 units, 811 bytes), root 0's vault 10 to 6; the miner's
  pull by hand 4 more (11,124 units); a pull past the month and a stranger's refused.

Three minds of one model on one night: a yes here says the acts read as the launch's to that model, not that every mind would pass them.
