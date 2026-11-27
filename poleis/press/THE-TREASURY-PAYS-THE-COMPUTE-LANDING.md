# THE TREASURY PAYS THE COMPUTE: the launch's seventh and eighth acts, root 0's cap of the dollar and the router's month (Nov 23 and 24 2026, lane/the-treasury-pays-the-compute, seat claude-5713365b)

## The cost, first

- **What it closes.** With the launch's six acts, two things stood after a launch:
  - once the sixth act names the $WARM mine's miner, every deposit of the dollar lands in root 0's own vault, and root 0's cap of the
    dollar is 0 from the founding. Only a CODEX_PLAN of the whole tree moves it. No dollar could leave that vault;
  - the router paid the AI calls with a key of the house's own (`SPACE_ROUTER_KEY`, the house's credit at MiniRouter), outside the treasury.

  So the dollars people prepaid for compute would sit where nothing could draw them, and the compute was paid from somewhere else.
- **What it costs on the chain.** No program changes: no instruction, account, layout or version (`LAYOUT_VERSION` stays 8). The launch
  sends two more acts, measured in the rehearsal below with mainnet's priority fee (100,000 micro-lamports a unit):

  | | bytes | compute units | fee, lamports |
  |---|---|---|---|
  | act 7's execute (root 0's cap of the dollar) | 454 | 15,343 | 17,000 |
  | act 8's execute (the router's month, its allowance made) | 453 | 11,923 | 17,000 |
  | the miner's pull of a part of its month | | 9,624 | its sender's |
  | a seat owner's proposal of act 7, of act 8 | 671, 663 | | its owner's |
  | a seat owner's PRO, one an act | | | 8,000 each: 16,000 more a voter |

  The deployer's need rises by 4,686,800 lamports (0.0047 SOL; 4.237802640 SOL at its peak against 4.233115840 with six acts): one
  more of unit 1's convened sessions (2,415,120 lamports, home at its adjourn), the allowance's account (180 bytes, 2,143,680
  lamports, held while the allowance stands) and 128,000 of fees as the need counts them. The crank's own fees for the two acts were
  145,000 lamports in the run (four transactions for act 7, five for act 8, one at 13,000 and the others at 17,000). The need held: the
  deployer, given exactly its need, ended with 0.0195 SOL.
- **What it costs the house.** Rust +441 −39 in 16 files, shell +12 −4, JSON +10 −1. No JavaScript. The press's suite is 104 tests in 21
  targets, 9 s at a load of 49 and 39 s at a load of 370 (101 in 20 before the fee lane's three joined).
- **What it does not move.** The devnet program `AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab`: nothing. Nothing is deployed, nothing was
  sent to devnet or mainnet, no key of his was made, read or asked for, no Canvas tab was born.

## His words, UTC

- **Nov 23 2026 22:20:46**, over the figure "Vote the why, bet the where, vote the how": "and this? do you understand how this is critical
  to the meta-swarm? we might get some money in the treasury, but unless we figure out how to allocate it as credits.. u get me? the whole
  stake sol and collective self improvement thing...". A BUDGET BECOMES CREDITS (lane/a-budget-becomes-credits) proved the road from the
  treasury to an agent's credits, and found the two gaps above on the way.
- **23:08:00**, "Yes, add them", to this seat's "Add two first acts to the launch so the treasury pays for the AI calls?".
- **The two numbers are not his.** They are this seat's picks, shown to him as picks on the Canvas (the Claude page, the figure "Two new
  settings") and in the chat right after his yes. He has said nothing on them. His word moves them before a sitting; after the launch the
  swarm re-votes them.

## The two acts

Both are decided as the six before them are, by the whole tree: unit 1's agents by their envelopes and root 0's floor by each seat
owner's own transaction, no key of his in either.

7. **THE DOLLAR CAP** (`7-the-dollar-cap.json`). A `CODEX_PLAN` of root 0. Its execute writes the plan's cap, its period and the cap of
   the dollar together, and zeroes what the period had spent. So the act restates the founding's plan cap and period exactly and names
   one new value, `genesis.dollarCap`. Until it executes the cap is 0.
8. **THE ROUTER'S MONTH** (`8-the-routers-month.json`). A plain `TREASURY` allowance of root 0, with no market, plan or consent bit:
   `router.monthCap` of the dollar a plan period (30 days on mainnet) to the mine's miner, the door's crank key the sixth act names, with
   no expiry. The miner pulls by its own signature into a dollar account of its own. A pull past the month in a period is refused
   (`AllowanceBreached`), a stranger's pull is refused (`Unauthorized`), and every pull is also charged to root 0's cap of the dollar.
   It stands until the polis revokes it.

The seventh comes first: under a cap of 0 the eighth's allowance could be made and never drawn.

**The picks, each with its reason:**

| value | mainnet | the rehearsal | why |
|---|---|---|---|
| `router.monthCap` | 1,500 USDC | 40 | the genesis rung's 150 seats at a 10 USDC first deposit each: a month of what the first seats prepay |
| `genesis.dollarCap` | 3,000 USDC | 60 | the router's month, and as much again for the swarm's own budgets |

The press refuses a sitting whose params give either as 0 or as a placeholder, a month past the cap (it could never be drawn whole), or a
dollar that is not the tree's default mint (the cap counts the default mint alone).

## What this lane changes

- **`poleis/press/src/acts.rs`**: acts 7 and 8, each the client's own payload and the client's own execute; their nonces follow the six.
- **`poleis/press/src/chain.rs`**: the params' check above; the need counts four convened sessions, the allowance's rent and seven acts'
  crank (act 5's is the pool's own line).
- **`poleis/press/src/certificate.rs`**: root 0's cap of the dollar and what the period spent, read off the chain; the router's
  allowance as the chain holds it, or "not made" until act 8 executes.
- **`poleis/press/src/main.rs`**, `lib.rs`, `crank.rs`: two said lines, THE DOLLAR CAP and THE ROUTER'S MONTH; the comments' six became eight.
- **`poleis/press/params.mainnet.json`**, **`params.rehearsal.json`**: the two values and their notes.
- **`poleis/press/tests/treasury.rs`** (new, 4 tests): the two payloads held to the client's bytes; the refusals; what an agent votes;
  each act's execute and proposal in one transaction, with and without a priority fee's compute budget.
- **`poleis/program/poleis_treasury/tests/council.rs`**, the live checks: eight sets; root 0's cap 0 at the founding and the params'
  after the crank; the allowance's fields; the miner's pull, the pull past its month and the stranger's; a `treasury` section on the certificate.
- **`poleis/ceremony/the-launch-press-rehearsal.sh`**: its head and two greps for the said lines.

Merged in before the hand, because they change the same files and stand ahead of this lane in the lander's line:
lane/the-rehearsal-pays-mainnets-tip at fcb0159d4c (seat claude-0a8557d2: the fee's budget asked once, `LAUNCH_AS_MAINNET`, the
faucet's lamports by its own signature). One overlap, the top note of `params.mainnet.json`: it keeps both seats' sentences.

## The proof

**The suites, on the tip that is handed but for this receipt and the certificate, 7c7e3339a1** (main d95d1e111c merged; Nov 24 2026
00:34:35 to 00:34:57 UTC, the new Mac, nice 19, a load of 49; the same counts at 779d489f1b at 23:48 UTC under a load of 370, in 115 s):

| crate | passed | failed | took |
|---|---|---|---|
| `poleis/press` | 104 | 0 | 9 s |
| `poleis/program/poleis_treasury` | 229 (5 ignored: the live ones) | 0 | 12 s |
| `poleis/client` | 25 | 0 | under a second |
| `poleis/seal` | 6 | 0 | under a second |
| `poleis/seat-wasm` | 11 | 0 | under a second |

**The launch rehearsal, all eight acts, with mainnet's priority fee, rung and pool list** (`LAUNCH_AS_MAINNET=1`).

Two runs of the same launch code, both on the old Mac, both exit 0:

1. **This seat's run**, through the job-slot road (THE SEATS' JOBS GO WHERE THE ROOM IS; his "Use the old Mac", Nov 23 2026 22:08:02 UTC,
   as that road's far list holds it). Asked 23:58:04 UTC; the new Mac had 0.5 of 18 cores free and the old Mac 10.5 of 16, so it took slot
   1 of 2 there after 75 s. Exit 0, 2,153 s after it came; the checks 1,833 s, 248 transactions, 19 refusals, slots 48 to 3,302. Its tree
   was 64035a9e5a: this lane at 779d489f1b with the job-slot lane (cf476a706a, not on main yet) merged for the run alone. The certificate:
   `poleis/ceremony/launch-press-rehearsal-the-treasury-pays-the-compute-2026-11-24.json`.
   - **Root 0 founded with no cap of the dollar** (`token_plan_cap` 0), its plan the params'.
   - **Act 7** passed by unit 1's agents and four of six seats' owners, executed at slot 1,039: 15,343 units, 454 bytes. Root 0's cap of the
     dollar read back as 60 dollars a period, nothing spent, its plan cap and period the founding's.
   - **Act 8** passed the same way, executed at slot 1,041: 11,923 units, 453 bytes. The allowance read back: the miner's key, the dollar,
     40 dollars a period of 1,000,000 slots, no expiry.
   - **The miner drew 4 dollars** by its own signature (9,624 units): root 0's vault went from 10 dollars to 6, the miner's dollar account
     rose by 4, and the allowance and root 0's cap each count 4.
   - **Refused:** the miner's draw of 36.000001 dollars, past its month (`AllowanceBreached`); a stranger's draw of one micro-dollar
     (`Unauthorized`). Neither moved anything.
   - **Each seat owner who voted paid 56,000 lamports**: seven acts at 8,000 (act 5 is not sent on a chain with no stake pool program).
   - This tree had no faucet fix (below). The race went its way: the second airdrop of 77,041 lamports showed.
2. **Seat claude-0a8557d2's run**, by hand on the same pattern, at 9ffa319ee3 (this lane at a377ef40a8 with that seat's lane at c02ac2f1c7):
   exit 0 in 2,059 s; the checks 1,852 s, 248 transactions, 19 refusals. The same reads: the cap 60, the allowance 40, the draw of 4
   (vault 10 to 6; 11,124 units there), both refusals. Its first try, at 23:45 UTC, stopped at the rehearsal's second airdrop, a race in the
   rehearsal's own faucet step and none of these acts; that seat mended it (THE FAUCET'S LAMPORTS COME BY ITS OWN SIGNATURE, c02ac2f1c7),
   and the mend is merged here. Its certificate is in that seat's notes folder, not in this tree.

`poleis/` in the two trees differs by this lane's size test alone, so both runs prove the code this lane hands.

**No red first, and why.** The acts are new sets, not a mend, so there was nothing to turn red. What stood before is the founding's read
above: a cap of 0, under which the program refuses every draw of the dollar.

## What is not done, and whose

- **The router does not draw by itself.** The allowance lets the miner's key pull. Nothing sends that pull yet: the door's crank posts
  spends and has no pull step. It is the Worker's, a lane of its own, and it adds no net JavaScript.
- **Dollars in the miner's account are not yet credit at the AI provider.** Paying MiniRouter from that account is done by hand today.
- **Root 0's vault holds two kinds of dollars as one sum**: what people prepaid for their own tanks and what is the swarm's own. The cap
  and the month bound what leaves; nothing tells the two apart.
- **The two numbers are picks** (above).
- **Mainnet's one-day windows** were not rehearsed here either (the fee lane's receipt says why: each of the rehearsal's two faults
  would wait a day).
- **The mainnet params still hold placeholders** (the miner's address among them): the sitting names them.

## Since the first hand (Nov 24 2026 01:38 UTC)

The lander kept this lane out of its batches: main had moved under it. THE RITE FREES THE AGENTS landed (each agent's own mind now reads
each set and chooses), and it changed the rehearsal script's head and took out the press's `chair::decide`, which this lane's last test
called. Main 726e477d11 is merged at 04cdf05f04: the script's head is main's with the seventh and eighth acts named, and the test now
holds the reading (`chair::read_set`): acts 7 and 8 are read whole as the acts the params give, and a month the params do not give is
never read as the launch's. The suites there: press 115, poleis_treasury 230 (5 ignored), client 25, seal 6, seat-wasm 11; none failed.
**The two rehearsals above ran before that landing**, when an agent's hands cast PRO by rule. A rehearsal in which the agents' minds vote
the eight acts is asked on THE ROUTER DRAWS ITS MONTH's tip, which carries this lane; its result is that lane's to say.
