# THE SEAL NAMES ITS PROGRAM: an act that moves a program's upgrade authority names the program it is for (Nov 23 2026, lane/the-seal-names-its-program, seat claude-5713365b)

## The cost, first

- **The fault closed:** after the handoff Poleis, the market and the $WARM program answer to one upgrade PDA, and until this lane whoever
  carried a passed `CODEX_UPGRADE_AUTHORITY` out chose which of the three it acted on: a burn voted for one could end another's code for good.
- **What it costs on the chain:** 32 more bytes in that one act's payload (34 to 66 of the 768 a proposal holds) and one address derivation
  when it is carried out. Predicted: 1,500 compute units for each bump the derivation tries, so 1,500 to about 4,500 more, on an act that
  ran in 20,389 units (the certificate of Nov 23, `ceremony/launch-press-rehearsal-the-seals-road-2026-11-23.json`, `checks.reach.units`).
  **Measured: 23,437 units** on the same act in this lane's rehearsal (3,048 more: two tries of the derivation, for that sitting's
  market id; the tries are each program id's own, see `THE-SEAL-CLOSES-A-BUFFER-LANDING.md`, "A number of the last receipt"). Poleis's binary: 434,256
  bytes before, **434,816 after** (560 more; the press's build, `programs.poleis.build.bytes` of each certificate).
- **What it costs the house:** 180 lines of Rust and its tests added, 20 taken out; no line of JavaScript added (10 changed in place).
  No new account, no new instruction, no new moving part.

## Where the task came from

His Nov 23 2026 ask of what is missing for mainnet, as the session that started this one carried it (I did not read his line myself, so
I quote none): the one missing piece that is code alone and needs no decision of his. The fault was written down three times and built
by nobody: `poleis/press/THE-LAUNCH-PRESS-LANDING.md` ("What is not done", 3: "The seal's reach is loose"), and "still owed" in
`THE-WHOLE-TREE-DECIDES-LANDING.md` and `THE-SWARM-IS-ITS-OWN-MULTISIG-LANDING.md`. Each now says it is closed and points here.

## What stood

- `CODEX_UPGRADE` (kind 6) upgrades Poleis alone: it refuses any program but its own id, and the loader holds the program data to it.
  (Since the same day: `CODEX_UPGRADE_PROGRAM`, kind 14, upgrades any program the seal holds by name, `THE-SEAL-UPGRADES-EVERY-PROGRAM-LANDING.md`.)
- `CODEX_UPGRADE_AUTHORITY` (kind 7) is the only reach the seal has into the market's and the $WARM program's code. Its payload was
  `[new:32]`, the new holder and nothing else (`src/ix/proposal.rs`). The program data rode in as an account of the transaction that
  carries the act out, and anyone may send that transaction. The loader only asks that the authority sign, and the same PDA is the
  authority of all three. So the chambers voted "hand the code to this key" or "burn it", and the carrier decided whose.
- The rehearsal itself showed it: to reach the market's code its check took the client's instruction and swapped Poleis's program data
  for the market's by hand (`tests/council.rs`, the launch press on a live validator), and the chain took it.

## What this lane changes

1. **The payload names its program** (`poleis/program/poleis_treasury`). `CODEX_UPGRADE_AUTHORITY` is `[new:32][program:32]`: the new
   holder (zeros for a burn), then the program whose authority moves. A proposal's hash is over its payload, so the act the chambers
   seal is the act for that program and no other; the same hand of another program is another act with another hash.
2. **Poleis refuses any other program's data** (`src/ix/upgrade.rs`, `expect_programdata_of`). When the act is carried out Poleis derives
   the named program's data account as the loader does (the PDA of `[program]` under the upgradeable loader) and refuses a different
   account with `InvalidAccount`, before the loader is asked. The carrier no longer chooses anything.
3. **The old shape is no act.** A payload of 32 bytes can still be proposed and sealed like any bytes (Poleis checks a payload's shape
   where it would act, as for every kind), and is refused at the carrying out with `InvalidInstructionData`, on every program's data.
   The new holder stays first in the bytes, so a reader that knows only the old shape still reads the holder rightly.
4. **The client** (`poleis/client`): `Payload::UpgradeAuthority { origin, program, new }`; its bytes and so its hash carry the program;
   the account it hands the carrier is `programdata_of(program)` (new, public; `programdata_pda()` is that of Poleis's own id).
5. **The press** (`poleis/press`): it builds no such act (the handoff gives the three programs to the PDA by the loader's own
   `SetAuthority`, signed by the deployer; its prepared acts are the fund, the board's law and roles, root 0's market and the pool's
   birth). Its handoff's closing words
   said "whose executor names the program"; they now say the payload names it and that an act voted for one is refused on another's
   data. Its wire test holds the client's address to Solana's own (`get_program_data_address`) and to the press's.
6. **The browser's mirror of the client** (`embertide-app/ui/intro/js/poleis-client.js` and its copy under `dyson-society/js`, the same
   bytes): the builder takes the program, the bytes end in it, the parser reads it (and reads an old act's holder with no program), the
   accounts name that program's data. Five lines changed in place in each copy, none added. Nothing in the tree calls it.

## Account layouts and versions: nothing moves

- No account's layout changes: a proposal already keeps its payload in clear, up to `MAX_PAYLOAD` (768 bytes); this act's payload is 66.
  `LAYOUT_VERSION` stays 8. No discriminator, seed, instruction tag, error code or kind number changes.
- **The devnet program** (`AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab`): this lane deployed nothing and read nothing from devnet. It
  keeps running the code of its last upgrade, which takes the old 32-byte shape and refuses the new one at the carrying out. When it is
  next upgraded to this code, by its own seal: every account it holds reads as before; an authority act proposed under the old shape and
  not yet carried out can never be carried out and must be proposed again, naming its program; a client that still writes the old shape
  writes acts that seal and never act. Whether devnet holds such an open act was not read.

## The proof

**Red first.** With the tests and the client changed and Poleis's source untouched, three of the council's tests were red against the
built program (Nov 23 2026 16:51 UTC): the new act refused as unknown (`InvalidInstructionData` where `InvalidAccount` and `Unauthorized`
were expected), and the old shape reaching the loader on every program's data ("Unsupported program id", Mollusk's word for the loader it
does not serve: the old code had let it through). After the change all three are green.

**The suites, on the new Mac, niced to 19** (`cargo test --offline --locked`, 16:56 to 16:58 UTC, load 76 to 144):

| crate | targets | passed | failed | ignored |
|---|---|---|---|---|
| `poleis/program/poleis_treasury` | 12 | 225 | 0 | 4 (they need a validator; the rehearsal runs two of them) |
| `poleis/client` | 3 | 23 | 0 | 0 |
| `poleis/press` | 15 | 75 | 0 | 0 |

The new tests:
- `council.rs` `an_act_voted_for_one_program_is_never_carried_out_on_another`: an act for the market, a hand and then a burn, passed by
  the whole tree; carried out on the $WARM program's data and on Poleis's own it is refused with `InvalidAccount`, their authority still
  the seal's and the act still open; on the market's own data it passes Poleis's law and stops only where Mollusk has no loader.
- `council.rs` `an_authority_act_that_names_no_program_is_refused`: the old shape, a hand and a burn, sealed by the whole tree, refused
  on all three programs' data.
- `client` `an_authority_act_names_its_program_in_its_bytes_its_hash_and_its_accounts`.
- `press` `wire.rs` `the_act_that_moves_a_handed_programs_authority_reaches_solanas_own_program_data_of_it`.
- The browser's mirror, read under node against the same bytes by a scratch check (not kept: the tree takes no new JavaScript): hand
  and burn are 66 bytes, `[0][7][new][program]`; the parser reads both shapes; the accounts name the named program's data.
- The gates that read a lane's diff, run on this lane by hand: `the-javascript-only-shrinks.test.mjs` 31 of 31; `any-room.test.mjs`
  12 of 12; `the-law-is-the-gate`, `no-page-of-its-own`, `the-look-is-a-law` green; the screen gate's two shards green (the mirror is a
  file the site builds: 1 built file differs, 36 cells judged a shard, 0 new failing, 0 to take out; 95 s and 98 s).

**The launch rehearsal, on a local validator** (`poleis/ceremony/the-launch-press-rehearsal.sh`, every feature the validator knows,
nothing read beyond this machine, stand-in keys in memory only; its certificate
`poleis/ceremony/launch-press-rehearsal-the-seal-names-its-program-2026-11-23.json`): 

- **Passed, exit 0, 1,763 s in all** (17:04 to 17:34 UTC, built from commit 3356a198bb): the press sent 557 transactions (526 of them the
  deploy's), the checks landed 150 and were refused 12 times as they must be (10 in the certificate before this lane; the two more are
  this lane's), slots 62 to 3,100, 1,672 s for the checks.
- **The act, live, after the handoff:** proposed for the market (`reach.program`), sealed by unit 1's two agents, its chair in root 0 and
  four of the six seats. Carried out on the $WARM program's data, then on Poleis's own: each refused by Poleis with `InvalidAccount`
  (`reach.refusedOn`), each authority read back as the seal's. Carried out on the market's own data: landed at slot 1050, 23,437 units,
  534 bytes; the market's authority read back as the new holder, Poleis's unmoved; the holder then handed it back to the seal.
- **A first run did not count:** started at 16:58 UTC, with this Mac at load 166 to 241 (this seat's suites and screen shards among it), the
  validator, niced to 19, ticked slower than the checks' clock allows and they stopped at their first wait ("the validator's clock stood
  still before slot 247"), before any act. Run again alone once those had ended (load 11 to 27 by the one-minute read), it passed as above.

## What this takes away, said plainly

The loose act reached any loader account the upgrade PDA held, and that included a **buffer**. An upgrade's buffer is handed to the PDA
before its vote (`ceremony/upgrade.mjs`: the loader upgrades only from a buffer whose authority is the program's). If that vote fails or
the upgrade is never carried out, the buffer and its rent (about 3.02 SOL at Poleis's 434,256 bytes) stay under the PDA. Under the loose
shape a second sealed act could have handed that buffer back to a key; no receipt, script or test ever did, and none describes it. Under
this lane the act reaches a program's data and nothing else, as the task asked, so that road is gone: such a buffer is spent only by
carrying its upgrade out. Whether the seal should be able to hand back or close a buffer by name is a decision on Poleis's law, not
taken here: a kind of its own, naming the buffer, would do it without loosening this one.

**Taken since, the same day** (his "Yes, build it", Nov 23 2026 ~17:42 UTC): `CODEX_CLOSE_BUFFER`, `poleis/THE-SEAL-CLOSES-A-BUFFER-LANDING.md`.

## What is not done

1. **The page a person votes on does not say which program.** `dyson-society/app/polis.js` describes this act as "handed to another key"
   or "burned: nobody can change the code again". The parser now gives it the program; saying it on the page is a change of his site's
   words, so it is not in this lane.
2. **The Keystone page** (the lane of seat claude-78d4ba56) signs a seat owner's own vote on what the press prepares, one member's
   transaction; the press prepares no authority act yet, so nothing there changes. When one is prepared for a real sitting, its set
   should say the program in words beside the hash.
3. **A burn is still for good**, and still one act of the whole tree. This lane makes it the burn that was voted; it adds no delay
   and no second act.
4. **Devnet** is as above: not upgraded, not read.
5. No gate of the lander runs these crates on a landing (THE CRATES GATE THE SHIP): they run at the ship. The rehearsal is run by hand.
