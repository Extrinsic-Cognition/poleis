# A BUDGET BECOMES CREDITS: a budget the market passes ends as an agent's credits at the mine, proven end to end on the chain's code as it stands (Nov 23 2026, lane/a-budget-becomes-credits, seat claude-5713365b)

## The cost, first

- **What it closes:** nobody had walked the road from the treasury to credits, and nothing proved it holds. His figure of the day, "Vote
  the why, bet the where, vote the how", is the swarm's budget machine, and it had never been run on money that becomes credits.
- **What it found:** the chain needs no new rule. Poleis, the market and the $WARM mine already do every step. The proof refused nowhere.
- **What it costs on the chain:** nothing changes there. No program, client or press source is touched. The road's own steps, measured on
  the program's tests (Mollusk, this lane merged with main at 73fa3f8497):

  | step | compute units |
  |---|---|
  | root 0's seal of the plan, its market and its road read | 17,584 |
  | the plan executes, the allowance opened | 11,543 |
  | the month's cap pulled into the branch's vault | 12,835 |
  | the branch's payment to its agent | 15,335 |
  | the agent's deposit at the mine | 8,129 |
  | the agent's spend mined (its $WARM account made) | 19,262 |

- **What it costs the house:** 231 lines of Rust added to one test file, 3 taken out. No JavaScript. No account, layout or version change.
  The program's `council` target took 10.5 s for its 87 tests with the scenario in (7.0 s without it at 21:33 UTC, at another load).

## His words, UTC

- **Nov 23 2026 19:49:07** (to claude-41008c71): "it feels to me like futarchy is long-term directional while instead voting is how you
  execute short term?". Seat claude-78d4ba56 answered with the figure "Vote the why, bet the where, vote the how".
- **22:20:46**, over that figure, to this seat: "and this? do you understand how this is critical to the meta-swarm? we might get some
  money in the treasury, but unless we figure out how to allocate it as credits.. u get me? the whole stake sol and collective self
  improvement thing...".
- **22:28:08**, "Yes, build it", to "Build the road from the treasury to credits?".
- The money words it stands on (`tools/embertide-mesh/pool/SOL-INFLATION-PAYS-FOR-INFERENCE-LANDING.md`): Oct 29 2026 18:57:09 "the SOL
  yielded goes into funding ai credits"; 19:10:13 "Nefertiti will spawn bunch of possible roads to be taken, market will then run futarchy
  on the proposed avenues".

## The road, each step on its row

The design is on the Canvas (the Claude page, `harmony-claude:widget-341c7297-b69d-4bdc-b1c7-ab88ab85aee5`).

1. **THE WHERE: the market passes a branch's month.** Root 0 proposes an allowance in the dollar for its branch (unit 1) as a plan
   (`CLASS_TREASURY | CLASS_PLAN`): a cap a period, and an expiry. The chairs pass it and the branch speaks in its own chair. A staker
   lays the plan's road and traders trade it. At the deadline the road is decided, root 0's seal reads it, and the allowance stands only
   if the road passed.
2. **THE DRAW: no decision.** Anyone pulls the period's cap into the branch's own vault, once a period.
3. **THE HOW: the branch's own fast vote.** Unit 1 alone, by its own chairs, pays an agent a token budget. Root 0 casts no vote.
4. **THE TANK: no decision.** The agent, a wallet of its own key, deposits what it was paid at the mine. The dollars are back in root 0's
   vault, which is the mine's treasury, and the agent's record counts them.
5. **IT MINES: no decision.** The miner the polis named posts the agent's spend. $WARM is mined to the agent, never past what it deposited.

No key of a person stands anywhere on it. The router's door already fills a tank from such a deposit: the depositor is the signer whose
own dollars fell (`embertide-app/ui/intro/dyson-society/app/space-router-deposit.mjs`, `verifyDeposit`).

## What this lane changes

One file, `poleis/program/poleis_treasury/tests/council.rs`:

- **`the_budget_becomes_credits`**, a scenario written as `the_market` and `the_mine` are, over the `Ledger` both worlds share. It runs
  at the end of THE TWO PROGRAMS IN ONE WORLD's chain, after the mine.
- **The branch is grown first**, because that world's root holds twelve founders and no child:
  - root 0 frees its twelfth chair (a recall);
  - root 0 lists the world's dollar;
  - root 0 founds unit 1 in the freed chair;
  - unit 1 lists the dollar for itself by its own chairs.
- **The plan is asked twice.** The road traded up is PASSED and executes. The road traded down seals DEADLOCK: its execute is refused
  (`NotPassed`) and no allowance stands.
- **The refusals it holds:** a second draw in the same period (`AllowanceBreached`); one micro-dollar past the agent's deposit
  (`PastDeposits`).
- **`MineBook` hands on** the world's dollar, the miner and the last spend's instant and seq. `Book` gains the budget's book.

## The proof

**No red first, and why.** No source changed, so there was nothing to turn red: the scenario is a proof of what stands. Had the chain
refused a step, that step was the build. It refused none.

**The numbers, off the accounts** (the cap 30 dollars a period of 1,000,000 slots; the branch paid its agent 12):

- root 0's vault of the dollar: 47.5 dollars before, 29.5 after. It is short exactly the 18 the branch has not paid out, which sit in
  unit 1's vault. Not a cent left the polis: what the agent was paid went round through the mine and came back.
- the agent's record at the mine: deposited 12, spent 12, mined 6 $WARM (the rate after the first halving).
- root 0's plan for the dollar charged 30; unit 1's charged 12.

**The suites, on the new Mac, niced to 19** (`cargo test --offline --locked --no-fail-fast`, the lane merged with main 73fa3f8497,
22:50 to 22:52 UTC):

| crate | targets | passed | failed | ignored |
|---|---|---|---|---|
| `poleis/program/poleis_treasury` | 12 | 229 | 0 | 5 (they need a validator) |
| `poleis/client` | 3 | 25 | 0 | 0 |
| `poleis/press` | 19 | 97 | 0 | 0 |
| `poleis/seal` | 4 | 6 | 0 | 0 |
| `poleis/seat-wasm` | 4 | 11 | 0 | 0 |

**Who wrote it:** the scenario was written by an Opus builder this seat briefed (584 s, 218,242 tokens, 40 tool calls). This seat read
every line of it, ran it again on the merged lane, and ran the five suites.

## What the proof showed about the chain's rules

- **The dollar's cap starts at nothing.** The default mint is charged to the registry's own counter, `token_plan_cap` a period, and only
  a `CODEX_PLAN` act sets it (`src/ix/proposal.rs`, `src/ix/mod.rs` `charge_token_plan`). A unit founded today can spend no dollar until
  its own slow vote names that cap. So the WHY row comes first: the whole tree votes how much may leave a period, then the market
  allocates inside it.
- **The branch speaks on its own budget.** With a crew in a chair, root 0's seal waits for it (`ChildMustSpeak`): the branch opens its
  own vote of the plan, seals and is heard in its chair before root 0 seals.
- **A plan's floor does not ballot.** Under `CLASS_PLAN` the floor speaks by price alone.
- **The vault that is the mine's treasury pays like any other.** The draw out of it and the deposit back into it both hold.

## What is not done

1. **Not run on a live validator.** `poleis/ceremony/the-market-on-a-local-chain.sh` runs the same chain on a local validator, so it now
   runs this scenario too: about 2,000 slots longer (two more windows of 1,000). Not run here: the Mac stood at load 153 to 208, and the
   launch's own rehearsal waits for that Mac first. Everything the scenario does goes through the `Ledger` both worlds share.
2. **Nowhere to propose a budget.** No page and no command writes these acts for a member. The press writes the launch's six acts only.
3. **The AI providers are not paid from the treasury.** The router pays them with its own key (`SPACE_ROUTER_KEY`). Once the mine is
   named, a person's deposit lands in root 0's vault while the provider's bill is paid elsewhere.
4. **What a budget may spend.** Root 0's vault of the dollar also holds what people prepaid for their own inference. The one limit on a
   budget is the dollar cap the whole tree votes; nothing tells prepaid dollars from the swarm's own.
5. **An agent's own deposit by its key file.** The door hands a person's wallet the deposit to sign in a page. An agent of a branch, a
   key file, has no client for it yet.
6. **No yield yet.** The stake pool is born with no validator, so staked SOL earns nothing
   (`poleis/press/THE-STAKE-GOES-INTO-HER-LANDING.md`, "No light yet"). This road spends dollars the treasury holds; it brings none in.
7. No gate of the lander runs these crates on a landing (THE CRATES GATE THE SHIP): they run at the ship.
