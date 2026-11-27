# ONLY THE DEPLOYER FOUNDS ROOT 0: tree 0's config is laid only by the payer Poleis's own program data names as its upgrade authority (Nov 23 2026, lane/only-the-deployer-founds-root-0, seat claude-5713365b)

## The cost, first

- **What it closes:** the launch could be spoiled.
  - Tree 0's founding was anyone's. `init_config` lays a tree's config for whoever pays first ("the first to take a root id owns its founding").
  - Tree 0's root passes the code's upgrade, the fund's constitution and the vetting.
  - On a real cluster the press's founding lands two slots after the deploy. A stranger who read the deploy could found tree 0 in those slots.
  - The press would then refuse to go on ("a root founded by another payer taken for ours", `poleis/press/THE-LAUNCH-PRESS-LANDING.md`). The launch would need Poleis closed by hand and deployed again at a new id, and the same stranger could do it again each time.
  - Now only the deployer can found tree 0. Every other tree is still anyone's: one program hosts every polis.
- **What it costs on the chain:**
  - Tree 0's founding: **6,095 compute units before, 10,720 after** (+4,625), measured on the program's own test (`adversarial.rs`, `POLEIS_CU`, main 0dd5a1b31d and this lane).
    - The 4,625 are the derivation of Poleis's program data: 1,500 a try, three tries for the test's id. The tries are each id's own.
    - The instruction runs once, ever.
  - The founding transaction gets one more account: 33 bytes.
  - Poleis's binary: **460,392 bytes before, 461,576 after** (+1,184, about 0.008 SOL of rent).
- **What it costs the house:** 99 lines of Rust and its tests added, 10 taken out. No JavaScript. No account, layout, seed, instruction tag or error code changes.

## His words, UTC

- **Nov 23 2026 21:10:06**, "so what are we going to do?". Answered with the road to mainnet in four stops. Stop 2 said: "Before that I'll also close one small gap I found: in the second after the code goes live, a stranger could found the polis before the launch does. After the fix, only the wallet that deployed it can."
- **21:17:20**, "Yes, take them", to "Go with the seats' picks for the launch settings?".
  - The settings themselves are another seat's lane (`lane/the-rehearsal-pays-mainnets-tip`, claude-0a8557d2). This seat passed his words to it and touches no params.
  - This lane is the gap.

## What this lane changes

1. **Poleis** (`poleis/program/poleis_treasury/src/ix/treasury.rs`, `src/ix/upgrade.rs`).
   - `init_config` for root 0 takes a fourth account: Poleis's own program data, the loader's PDA of its id (`expect_programdata_of`).
   - `expect_founder` holds the founding to it:
     - **No fourth account:** `NotEnoughAccountKeys`.
     - **Another address:** `InvalidAccount`.
     - **A program data owned by the loader:** its upgrade authority must be the payer, else `Unauthorized`. A program data whose upgrades were burned names none, so it founds nothing.
     - **Nothing at that address:** the founding asks nothing. That means no upgradeable loader deployed this Poleis (a test's). On a cluster the loader lays the program data in the deploy itself, so no stranger can make that address empty.
2. **The client** (`poleis/client`): `init_config_of(ROOT_UNIT, ..)` hands Poleis's program data in last. Every other root keeps three accounts.
3. **The press** is unchanged.
   - Its founding is `client::init_config_of(ROOT_UNIT, deployer, ..)`, and its deployer is both the payer and, until the handoff, the upgrade authority.
4. **The tests.**
   - `adversarial.rs`:
     - `only_the_deployer_founds_root_0`: a stranger refused, with the account, without it and with another in its place; the deployer founds; tree 7 stays anyone's, with three accounts.
     - `a_poleis_whose_upgrades_were_burned_founds_no_root_0`.
   - `council.rs`'s `loader_estate` now lays Poleis's program data in the place the estate's founding left empty, where it used to push a second entry for the same address.

## Account layouts and versions: nothing moves

- **Versions:** `LAYOUT_VERSION` stays 8. No account, seed, tag, kind or error code changes.
- **Devnet** (`AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab`): nothing was deployed there and nothing was read.
  - Its tree 0 is founded, so nothing changes there.
  - Its code takes `init_config` with exactly three accounts. So this lane's client, pointed at that old code, would be refused for a new tree 0 (`NotEnoughAccountKeys`), never for any other tree.

## The proof

**Red first.** With the tests and the client in and Poleis's source untouched, the three founding tests were red against the built program (21:24:29 UTC). The old code takes exactly three accounts, so each founding of tree 0 was refused with `NotEnoughAccountKeys` where `Unauthorized` or a founding was expected. They were green at 21:25:13 UTC.

**A suite caught this lane's own miss.** The first full run (21:25 UTC) was red on `the_seal_upgrades_the_program_its_act_names_and_no_other`.
- **Why:** the estate's founding now names Poleis's program data, so the test world held that address empty. `loader_estate` then pushed a second entry for it, and the world read two different accounts.
- **Checked on a clean copy of main** (0dd5a1b31d): the same test is green there.
- **Fixed** by laying the program data in place, then rerun.

**The suites, on the new Mac, niced to 19** (`cargo test --offline --locked --no-fail-fast`, the lane merged with main 0dd5a1b31d, 21:32 to 21:35 UTC):

| crate | targets | passed | failed | ignored |
|---|---|---|---|---|
| `poleis/program/poleis_treasury` | 12 | 229 | 0 | 5 (they need a validator) |
| `poleis/client` | 3 | 25 | 0 | 0 |
| `poleis/press` | 19 | 94 | 0 | 0 |
| `poleis/seal` | 4 | 6 | 0 | 0 |
| `poleis/seat-wasm` | 4 | 11 | 0 | 0 |

**The press's own founding passes the gate (read in its code, not yet run live by this seat).**
- The press deploys every program with the deployer as both payer and upgrade authority:
  - `poleis/press/src/chain.rs`: `loader::deploy_with_max_data_len(&self.deployer, .., &self.deployer, ..)`.
- Its founding is paid by the same deployer, before the handoff:
  - `client::init_config_of(ROOT_UNIT, *deployer, ..)`.
- So the program data the loader lays names the payer.
- The $WARM program's `init` makes the same read of a program data and has passed live in every rehearsal today (`warm_mine/src/ix.rs`, `expect_upgrade_authority`).

**The launch rehearsal: one run, not two.**
- This seat did not run its own. The Mac stood at load 109 to 526 from 21:38 to 21:49 UTC, and a rehearsal stops when the validator's clock falls behind.
- The rehearsal of mainnet's own values runs this code once this lane lands. It is queued on `lane/the-rehearsal-pays-mainnets-tip` (claude-0a8557d2, 6298717fa6), and starts when the new Mac's load drops under 40, up to 3 h from 22:08 UTC.
- That seat's word: "if it lands before my run starts, I merge main first, so the run covers it". The founding transaction then carries the priority fee's budget too: about 887 + 56 + 33 bytes, under 1,232.
- Its certificate, not this receipt, says how the founding went live.

## What is not done

1. **Live.**
   - The press's founding through the gate on a real loader waits for the rehearsal above.
   - No rehearsal can show a stranger refused: by the time its checks run, the press has founded tree 0. The refusal is proven on the program's tests, against the built program.
2. **A Poleis deployed by another loader** (loader v4 lays no program data apart) would leave tree 0 anyone's again. The press deploys by the upgradeable loader (v3), as the seal itself needs.
3. **The JavaScript is not touched** (the tree takes no new JavaScript).
   - The browser's client (`embertide-app/ui/intro/js/poleis-client.js` and its copy) and the old tools build `initConfig` with three accounts.
   - A new tree 0 founded through them would be refused. That includes `dyson-society/tools/bronze-mainnet.mjs` given `ROOT=0`.
   - Every other tree founds as before. Tree 0 on mainnet is the press's.
4. No gate of the lander runs these crates on a landing (THE CRATES GATE THE SHIP): they run at the ship.
