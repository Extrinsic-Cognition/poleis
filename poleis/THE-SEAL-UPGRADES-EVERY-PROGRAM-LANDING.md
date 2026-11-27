# THE SEAL UPGRADES EVERY PROGRAM: a vote of the whole tree replaces the market's or the $WARM program's code by name, with no key between (Nov 23 2026, lane/the-seal-upgrades-every-program, seat claude-5713365b)

## The cost, first

- **What it closes:** after the handoff the seal holds the code of three programs, and until this lane it could upgrade one, Poleis. The
  market's and the $WARM program's code moved only by a vote that handed its authority to a key. The key then did the upgrade, and nothing
  made it hand the code back. Now the vote upgrades them itself.
- **What it costs on the chain:** one more kind of CODEX act, 96 bytes of payload. Carrying it out costs one derivation of the named
  program's data (1,500 units a try, the tries the id's own) and the loader's own `Upgrade`. **Measured: 28,500 units** in the rehearsal,
  in a sitting whose $WARM program's data derives at the first try and whose upgrade PDA takes five.
  Kind 6 (Poleis's own upgrade) now holds its program data the same way before the loader is asked: one derivation more on an act that
  runs a few times a year. Poleis's binary: 437,280 bytes before, **437,656 after** (376 more).
- **What it costs the house:** 218 lines of Rust and its tests added, 10 taken out. No JavaScript. No account, layout or version change.

## His words, UTC

- **Nov 23 2026 18:20:59,** "so what is going to be the case?", after THE SEAL CLOSES A BUFFER landed. Reading the code for that answer,
  this seat found that `CODEX_UPGRADE` refuses every program but Poleis (`src/ix/proposal.rs`), and that only `src/ix/upgrade.rs` ever
  signs as the seal's PDA. So the market's and the $WARM program's code could move only through a key.
- **18:54:14,** "Yes, close it", to "Close that gap, so the vote upgrades the market and $WARM itself?".

## A new kind, not kind 6 reshaped

- Kind 6's 64 bytes can only ever mean Poleis. It was not ambiguous, as the authority act's old shape was, so there is nothing to refuse.
- Reshaping it would strand devnet. A Poleis running today's code takes only the 64-byte upgrade, so the upgrade that brings it new code
  must be voted in the old shape. A client that wrote only a new shape could never upgrade it again.
- An added kind leaves devnet's upgrade road (`ceremony/upgrade.mjs`, through the client's `ceremony` example) and the browser's client
  untouched. The browser's client reads kind 14 as "an act", as it reads kinds 11 to 13.

## What this lane changes

1. **`CODEX_UPGRADE_PROGRAM`, kind 14** (`poleis/program/poleis_treasury/src/lib.rs`, `src/ix/proposal.rs`). Payload
   `[buffer:32][spill:32][program:32]`. Only root 0 passes it, and, as every CODEX act, it closes over the whole tree in the long window.
2. **Its refusals, before the loader is asked:**
   - a program, a program data, a buffer or a spill other than the named ones: `InvalidAccount` (the program data derived as the loader
     derives it);
   - an origin other than root 0: `Unauthorized`;
   - a payload of another length: `InvalidInstructionData`.
3. **Then the loader's own `Upgrade`, signed by the seal's PDA.** The loader itself refuses a program the seal does not hold, a buffer
   that is not the seal's, and a build larger than the program's data (read in Agave 4.2.2's loader, `solana-bpf-loader-program`).
4. **Kind 6 shares the arm**, unchanged in shape (Poleis's own code, 64 bytes), and now holds its program data to Poleis too.
5. **The client** (`poleis/client`): `Payload::UpgradeProgram { origin, program, buffer, spill }`.
6. **The press** (`poleis/press`): its handoff's words say the code of any of the three moves by a vote that names it. Its wire test holds
   the act's accounts to the loader interface's own `Upgrade`, in its order. It prepares no such act.

## Account layouts and versions: nothing moves

`LAYOUT_VERSION` stays 8; no account, seed, instruction tag or error code changes; one kind number is added. The devnet program
(`AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab`) was not deployed to and not read: its code refuses kind 14 as an unknown kind until its
own seal upgrades it, by kind 6 as before.

## The proof

**Red first.** With the kind's number, the client and the tests in and no executor, the council's new test was red against the built
program (18:59:55 UTC): the crew's act refused as an unknown payload (error 43) where `Unauthorized` was expected. Green after.

**The suites, on the new Mac, niced to 19** (`cargo test --offline --locked`, 19:01 to 19:02 UTC, load 18):

| crate | targets | passed | failed | ignored |
|---|---|---|---|---|
| `poleis/program/poleis_treasury` | 12 | 227 | 0 | 5 (they need a validator) |
| `poleis/client` | 3 | 25 | 0 | 0 |
| `poleis/press` | 16 | 79 | 0 | 0 |
| `poleis/seal` | 4 | 6 | 0 | 0 |
| `poleis/seat-wasm` | 4 | 9 | 0 | 0 |

The seal's and the seat's crates link the client, whose payloads three lanes moved today; they ran here too. Main moved by five landings
while the rehearsal ran (the seal's reader, the press's miner and their tests, `council.rs` among them; none in Poleis's program). Main was
merged into the lane before its hand (f3c1a3522b), and the five suites ran again on the merged tree, 19:37 UTC: 227, 25, 83 (17 targets),
6 and 9 passed, 0 failed. The rehearsal below ran on 9077a5512c, before that merge.

The new tests:
- `council.rs` `the_seal_upgrades_the_program_its_act_names_and_no_other`. A crew's own upgrade is refused. The root's act for the $WARM
  program is refused when carried out on the market and its data, on Poleis and its own, on the $WARM program with the market's data, from
  another buffer and to another spill; nothing moves and the act stays open. On its own accounts it passes Poleis's law and stops only where
  Mollusk has no loader. The same kind naming no program, sealed by the whole tree, is refused.
- `client` `an_upgrade_names_its_program_in_its_bytes_its_hash_and_its_accounts`.
- `press` `wire.rs` `the_upgrade_of_a_handed_program_carries_the_loaders_own_upgrade_accounts`.
- The gates that read a lane's diff, run on this lane: `the-javascript-only-shrinks`, `the-law-is-the-gate`, `any-room`, 51 of 51.

**The launch rehearsal, on a local validator** (`poleis/ceremony/the-launch-press-rehearsal.sh`, every feature the validator knows,
nothing read beyond this machine, stand-in keys in memory only; its certificate
`poleis/ceremony/launch-press-rehearsal-the-seal-upgrades-every-program-2026-11-23.json`, `checks.upgrade`):

- **Passed, exit 0, 1,912 s in all** (19:02 to 19:34 UTC, built from commit 9077a5512c, one run): the press sent 560 transactions, the
  checks landed 239 and were refused 17 times as they must be (170 and 15 in the certificate before this lane; the 69 more are this lane's
  buffer, its 57 writes, its vote and its two refusals), slots 58 to 3,195, 1,803 s for the checks.
- **The $WARM program's own code** (56,544 bytes, read off its program data on the chain) was written to a buffer in 57 transactions by a
  key of the run, and the buffer was handed to the seal's PDA (slot 1136).
- **The vote:** proposed by a seat owner, passed by unit 1's two agents, its chair in root 0 and four of the six seats on the floor.
- **Carried out on the market and its data, then on Poleis and its own:** each refused by Poleis with `InvalidAccount`; each program's
  authority read back as the seal's, and the $WARM program's data read back unchanged.
- **Carried out on the $WARM program's own accounts** at slot 1146, 28,500 units, 666 bytes: its program data read back with a new
  deployment slot (25, then 1146), the same code, the seal its authority still; the buffer spent, its lamports the spill's to the lamport.
- **The re-deployed program ran afterwards:** the polis named the mine's miner through it (slot 3187), a flask deposited into root 0's
  vault (3190), and the first spend was mined (3191, 19,840 units).
- **The certificate says it too:** its `programs.warm.deploySlot` is 1146, the seal's re-deployment, read off the chain at the end, and
  `runsTheBuild` is still true: the bytes on the chain are the build the press recorded.

## What is not done

1. **A larger build needs the program extended first.** The press deploys each program at its own length, and the loader refuses a build
   larger than the program's data. The loader's `ExtendProgram` asks no authority: anyone pays, at least 10 KiB at a time once SIMD-0431 is
   on (read in Agave 4.2.2's loader and the loader interface 7.0.0). `ceremony/upgrade.mjs` does it on devnet for Poleis; the press has no
   such step for any program.
2. **Nothing prepares a kind-14 act.** Devnet's road upgrades Poleis alone, by kind 6, as before; the press prepares no set for it, so no
   seat owner can vote it from the Keystone page yet.
3. **The society's page** shows it as "an act".
4. No gate of the lander runs these crates on a landing (THE CRATES GATE THE SHIP): they run at the ship. The rehearsal is run by hand.

## Corrected the same evening: no key of his in these receipts

- **What was wrong.** His Nov 23 2026 17:52:35 UTC "the swarm votes on updates, you dont have it linear with me and keystones" governs
  every act of the seal. Despite it:
  - all three of today's seal receipts (THE SEAL NAMES ITS PROGRAM, THE SEAL CLOSES A BUFFER and this one) called the page that signs a
    seat owner's vote "his Keystones' page";
  - this seat's closing words and its figure on the Canvas listed "one rehearsal with his own Keystone" among his steps before mainnet.
  Both put his key back into the swarm's road.
- **The page and the acts.** The page signs one member's own transaction, whoever holds that Keystone. The acts these receipts add are
  decided by the whole tree.
- **What is his before mainnet:**
  - the launch values: 38 PLACEHOLDERs in `poleis/press/params.mainnet.json` on main at 4f033a52f9;
  - a ship: production is 249d3e8f88, 135 landings behind main;
  - the SOL on the deployer.
- **Checked again the same evening: the extension.** Agave 4.2.2's loader knows `ExtendProgram` alone and calls its common extension with
  no authority check (`common_extend_program(.., false)`), so "anyone pays" above holds.
- **The cost:** words only. Three lines are changed in place and this section is added. No code changes.
