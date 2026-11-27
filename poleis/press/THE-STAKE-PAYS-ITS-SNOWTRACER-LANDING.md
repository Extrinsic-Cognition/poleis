# THE STAKE PAYS ITS SNOWTRACER (Nov 25 2026, lane/the-light-pays-a-wish-and-its-rider, joined with lane/the-stake-earns)

The three missing links of the launch's road, in one lane. It stands on two receipts and does not repeat them:
`THE-STAKE-EARNS-LANDING.md` (link 3, the engine) and `THE-LIGHT-PAYS-A-WISH-AND-ITS-RIDER-LANDING.md` (links 6 and 7).
The road: ec-workspace `docs/the-launch-the-stake-that-funds-snowtracers-and-his-salary-hypertrace-2026-11-25.md`, section 3.

## His words, UTC

- Nov 25 2026 13:11: "we gotta launch swarmbook and the stake sol to fund snowtracers... i need some salary and that i can only think would come from the
  snowtracing staking sol there... hypertrace".
- Nov 25 2026 13:25: "Yes, build them", to "Build the three missing links now?".

## The seven links, now

| # | Link | State |
|---|---|---|
| 1 | Her pool is born on mainnet, with root 0 | rehearsed (the launch's act 5); his sitting |
| 2 | A person stakes in her, keeps the SOL, leaves at par | built; the page's doors now read her validator list into the module (below), so a stake or a way out works once a validator stands |
| 3 | The stake earns | built (the engine, act 9); rehearsed here with the validator's own reward |
| 4 | The yield is counted, staker by staker | built (`embertide-stake`) |
| 5 | The yield lands in root 0's vault | built at the pool's birth |
| 6 | Each staker's light reaches the wish they chose | built: the reader, the pour on each wish's road, the over-pull said and netted; root 0's plan of her token is act 10 |
| 7 | The well pays its Snowtracer | built: the wish's own rider allowance; the rider's draw to SOL through a pool with validators |

## The order of the acts, for a first pour to land

1. **Act 5**, her pool born (root 0's raw act; the launch's eight).
2. **Act 9**, the engine's staker (SetStaker by the polis's hand), a later act. Until it executes no lamport earns.
3. **Act 10**, root 0's plan of her pool's token (CODEX AllowMint, `light.planCap`), a later act on the same road as act 9 (`engine::later`, so `press crank`,
   `press vote --act 10`, `press chair`, `press need` and the certificate carry it). Until it executes, Poleis refuses every pull of her token out of root
   0's vault (`charge_token_plan`).
4. **Each wish's road**: root 0's TREASURY_ALLOWANCE to the wish's unit, voted once per wish once it is born (`press well`; never a launch act).
5. Then, each epoch: the engine's crank (`press engine`), then the book's count (`embertide-stake count`), then the pour (`press light --send`).
6. For the rider: **the wish's** plan of her token, then **the wish's** allowance to the rider's key (`press rider`), then the rider's own draw.

Act 10's set is written only where the params name its figure: mainnet's `light.planCap` is a blank of his; the rehearsal's says THIS REHEARSAL and writes
no act (the joint rehearsal votes it on its own tree). A sealed upgrade's number in the crank now follows the last act named, not the count of acts.

## The joint rehearsal (`poleis/ceremony/the-stake-pays-its-snowtracer-rehearsal-2026-11-25.json`)

`sh poleis/ceremony/the-stake-pays-its-snowtracer-rehearsal.sh <mainnet's stake pool .so>`: **86 s wall** (the live test 81.7 s), solana-test-validator
4.2.0, 32 slots an epoch, mainnet's stake pool program (sha256 `d18a2cdc…52ac`), this tree's Poleis upgradeable under a stand-in deployer that founds root 0.
No transfer: the yield is the validator's own reward. Read back:

- Root 0 and a wish (unit 8405527522246845140). **Act 5** (her pool, born by root 0's raw act), **act 9** (the engine's staker
  `3uLf…byw`, read back from the pool account) and **act 10** (root 0's plan of her token) each proposed, voted, sealed and executed **through Poleis**,
  none by a stand-in manager; then root 0's road into the wish's well.
- Two stakers: 150 SOL with their light to the wish, 50 SOL with no choice. The book's first read.
- The engine, epoch 6: its validator (this validator's own vote account) added and 178,997,717,120 lamports placed (1,002,282,880 active at once,
  the rest transient); 20,000,000,000 kept in the reserve (a tenth); its fees 15,000 lamports.
- Epoch 7: the stake active, 179,997,717,120 lamports, nothing minted yet. **Epoch 8: the validator's reward, 1,395,180 lamports, minted as 1,395,180 tokens
  of fee into root 0's vault.** A local validator's inflation is small; these are the exact amounts.
- The book's second read: epochs 5 to 8, fee 1,395,180 tokens = 1,395,180 lamports, 0 dust; the chain's mints read by the reader: 1,395,180. They agree.
- Lights: staker 1, 1,046,385 lamports; staker 2, 348,795. **The pour: 1,046,385 tokens into the wish's well**; 348,795 left undirected in root 0's vault;
  0 over-pulled. The pour again: nothing.
- The wish voted its plan and the rider's allowance (1,046,385 tokens a 10,000-slot period). **The rider's draw to SOL**, one transaction, the pool behind
  the epoch so its update named her one validator: 1,046,385 tokens drawn and burned for 1,046,385 lamports at par, 64,232 compute units, fee 5,000. The
  rider held 1,000,000,000 lamports before and 999,002,105 after: the draw brought 1,046,385 and the rider paid its receipts account's rent, 2,039,280
  (theirs back if they close it), and the fee. At this rehearsal's yield the first draw costs more than it brings.

## What a staker, a pour and a draw cost

- **A staker**: the stake 5,000 lamports a transaction and 2,039,280 for the receipts account once; a light 5,000; the way out 5,000 (the engine's receipt
  has the transaction sizes with validators named).
- **A pour**: one transaction a wish an epoch, the crank's signature: 5,000; 2,039,280 once for a wish's vault. A record alone (a wish over its light): 5,000.
- **A draw**: 5,000 and 64,232 units for the draw to SOL with one validator named; 2,039,280 once for the rider's receipts account.
- **The road, once**: root 0's plan (99 bytes) 1,579,920 lamports of rent; each wish's road (180 bytes) 2,143,680; the wish's plan 1,579,920 and its rider's
  allowance 2,143,680.

## The page's validator list (link 2 on the page)

The glue (`her-door.mjs`) fetches her list's account into the module's `LIST_AT` slot with its length, for the stake and the way out (the draw takes it too;
no page calls the draw yet). The list's address comes from the module (`door_list`, read from the pool's own bytes), so the glue parses nothing. **No line
of JavaScript is added**: the gate counts this lane +9 -9 over main, net 0, in 3 files; every one is a changed line: in `her-door.mjs` lines 22, 31, 32,
54 and 59 (the list) and 69 and 70 (the light read through `door_chosen`), in `polis.html` and `after-the-mint.html` the cache key (`?v=light-3`). No line
was removed to pay for another, because none was added. The page proof (`the-door-rehearsal.sh`, Chrome for Testing headless) passed on the final module,
on a pool with no validator; the glue's list with a validator in a browser is not proven (the module's `door_stake` with a list is, by its own test).

The module: **116,091 bytes, sha256 `b0d89a2c034adfb7985b8412d839ffd97dab304adf0be899cb32450b838344c4`**, built last and built again by the door rehearsal
to the same bytes.

## The suites and gates, run once each on this lane (nice 19, this worktree's own build folders)

- `poleis/press` `cargo test --offline --locked`: 149 passed, 0 failed, 26 s.
- `poleis/door`: 11 + 4 passed, 7 s.
- `embertide-stake`: 24 passed, 2 s.
- `poleis/seal`: tests and examples built, 5 s.
- `poleis/program/poleis_treasury` `--test council the_ninth_act_names_her_pools_staker…` (ENGINE_STAKE_POOL_SO): 1 passed, 63 s with its build.
- `node --test the-javascript-only-shrinks.test.mjs`: 39 of 39, +9 -9 net 0, 1 s. `the-law-is-the-gate.test.mjs`: 8 of 8, 86 s. `any-room.test.mjs`: 12 of 12, 1 s.
- `no-landing-adds-a-failing-screen.sh`: the site moved (4 built files); shard 1: 24 cells, 0 red, 68 s; shard 2: 24 cells, 0 red, 71 s.
- The light rehearsal (with the stranger's over-pull) 76 s and the door rehearsal with the page 35 s, both exit 0.

## Not built, and whose

- **The cap the whole tree votes for a wish's road is the only bound the chain itself enforces** on what reaches a well; the crank sees an over-pull, says it
  and nets it after the fact. A tighter road is a new instruction (its checks in the light's receipt). His to rule.
- **Act 9 and act 10 are not yet played inside the full launch rehearsal** on a validator (`the-launch-press-rehearsal.sh`, the bicameral root 0 with unit 1
  and the floor); here they pass through Poleis on the wish walker's simple tree, and act 9 through root 0's raw act in `council.rs`.
- **More than 8 validators in one stake**: a stake or a way out behind the epoch names every validator in one transaction (the engine's receipt holds the
  count that fits); past it the engine's own update must run first.
- **What runs the two cranks each epoch** (`press engine`, then the book's count and `press light --send`), and on which machine: not set; nothing runs at rest.
- **Removing a validator** from her list: not built (the engine's receipt).
- **Mainnet's values, his**: the validators and their weights, the reserve's share, the engine's key, `light.planCap`, each wish's road's cap and period,
  each rider's cap, and whether a Snowtracer's pay is cash, credits or both.
- **Undirected light** stays in root 0's vault; its rule is his.

## The gate

At the ship (THE CRATES GATE THE SHIP): `poleis/press` (`tests/light.rs`, `tests/rider.rs`, `tests/engine.rs`, `tests/params.rs`, `tests/door.rs`,
`tests/page.rs`), `poleis/door` (`tests/page.rs`, `tests/the_pages.rs`), `embertide-stake`, `poleis/program/poleis_treasury/tests/council.rs`. On every lane:
`the-javascript-only-shrinks.test.mjs`, `no-landing-adds-a-failing-screen.sh`. No gate yet on the live road: the measure it waits for is
`sh poleis/ceremony/the-stake-pays-its-snowtracer-rehearsal.sh <the .so>` ending 0, and on mainnet, each epoch, the lamports of yield in root 0's vault, the
tokens poured into each wish's well by its stakers' light, the over-pulled tokens, and the lamports a well paid a Snowtracer, read from the chain.

## Rule 8, and the judging seat's own run (claude-bea6ced6, Nov 25 2026 14:3x UTC)

The lane changes three files under the mint (`after-the-mint.html` and `polis.html`, one cache key each; `her-door.mjs`, seven lines in place), so
THE SOUND NEVER RESTARTS THE PICTURE is proven on this tree, both halves:
- `node --test app/dl-twelve.test.mjs`: 21 of 21 pass.
- `nice -n 19 node app/dl-twelve.proof.mjs --url http://127.0.0.1:31497/dyson-society/app/warm.html --tap button`: PASS; `--tap sphere`: PASS. The
  page served from this lane's tree by `tools/serve-range` (206 on Range), a muted headless Chrome off every screen; the server ended after.

The joint rehearsal run once more by the judging seat, on the lane's tip 2ccc05d212, at nice 19 on the new Mac: exit 0 in 100 s (the test 89 s). Read
back: the validator's reward minted as 1,395,169 tokens into root 0's vault; 1,046,376 tokens poured into the wish's well, 348,793 left in root 0's
vault, 1 token of dust, 0 over-pulled; the rider drew 1,046,376 tokens and held 1,046,376 lamports more for them, at par (fee 5,000, the receipts
account's rent 2,039,280). The validator ended with the script.
