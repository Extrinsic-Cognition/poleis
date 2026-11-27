# THE SWARM SHIPS THE UPDATE: any member proposes, the swarm decides, its seal ships by itself (Nov 23 2026, lane/the-swarm-ships-the-update, claude-78d4ba56)

## His words, UTC (from this session's transcript)

- **17:52:35** "hypertrace. you keep getting it wrong, the swarm votes on updates, you dont have it linear with me and keystones idk... am i wrong?"
- **18:13:12** "so?", after the trace's answer (ec-workspace `docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, its Nov 23 17:52 section: no, he is
  not wrong; the seats threaded his keys back into the line seven times since Oct 29; two steps of the release road still ran through his key).
- **18:20:38** "Yes, build it", to "Take your key and your press out of the update road?".
- Before them: Nov 23 11:58 "the truth is, the swarm deliberates and votes and that lands on solana" (A RELEASE IS A VOTE); Nov 22 09:50:51 "you gotta
  fucking stop treating the keystones as what make the decisions porcoddio" (THE WHOLE TREE DECIDES); Oct 28 16:04 his pick "hand the code's authority to
  the program's own upgrade address, and prove an upgrade by the seal"; Sept 9 18:41 "AS LONG AS THERE IS UNRUGGABILITY WRITTEN IN CODE JUST LIKE FUTARCHY,
  BASICALLY WE WIN".

## Cost first

| | before (main 68039c6e7f) | after (this lane) |
|---|---|---|
| who may propose a release the house takes | the keys "his key names" (`seal.proposers`; mainnet's list empty, so no seal would ever count); a rider's release the whole tree sealed RATUM was refused "at both ends" | any member of root 0 (an owner of its floor's seats; Poleis takes a proposal only from a member), the AND of both chambers decides; a road or a params file that names proposers, even none, is refused by words that say why |
| what moves production once a road is named | a press (his on the changelog, or a hand's verb) shipping main, refused when main was not sealed | the newest commit of main the swarm sealed: the after-work of every landing reads it and ships it by itself; any press (his, a hand's) ships only that; his press also asks the swarm's vote on main's commit (`spool/ship`'s `votes`) |
| before a road is named (today, live) | the press ships, the seal said, not held | unchanged (SAY-ONLY UNTIL A ROAD IS NAMED): nothing of this moves before the launch names the road |
| a read of the swarm's newest seal | | six commits' accounts a `getMultipleAccounts` (96 of its 100), from the newest down, the 64 newest landings since production and every older one a press asked; then the one that passed read whole as `read` reads it (a `getSignaturesForAddress` and two `getTransaction`). On the stand-in reader in the tests: under 1 s. On a public node: unmeasured (owed with the polis on mainnet) |
| at rest | nothing | nothing: the read is a child the after-work starts at a landing; one that finds nothing new sealed says nothing and moves nothing |
| moving parts | | the reader's `newest` step (Rust, `poleis/seal/src/main.rs`, +45 lines); the wrapper passes it (`seal-read.sh`, +5 lines of shell); no new door, ref or process at rest |
| JavaScript, by its gate | | +10 −10, net 0 (`lander.mjs` 8 lines changed in place, its fixture 1, the ship's test 1) |
| tests (the new Mac) | | the reader 6 (`cargo test`), the devices' check 6 (`pull_seal_test.py`), the press 78, the lander's folder 355 of 355 (`node --test`, 310 s, four files at a time, nice 19), the law's table 8 |

## What changed

1. **The seal's reader** (`poleis/seal`): ANY MEMBER PROPOSES, THE SWARM DECIDES. `Road` holds `{rpc, program, polis}`; a road with `proposers` is refused
   (`PROPOSERS_NAMED`); `accounts_word` and `verify` take the seal of whichever member proposed it (the proposal's transaction must still be signed by the
   proposer the seal names). `poleis-seal newest <commits> --road R` is the swarm's newest seal among the commits given.
2. **The devices' check** (`embertide-app/tools/nest-release/pull.py --seal-road`): the same law in stdlib Python, held to the same vector
   (`poleis/seal/tests/vector.json`, whose rider's release, sealed RATUM by both chambers, is now taken).
3. **The press** (`poleis/press`): the road it writes at the founding names nobody who may propose; `seal.proposers` in a params file stops the sitting
   (`seal::PROPOSERS_NAMED`); both params files carry the law's words; the record and the certificate say "any member of root 0 ... proposes a release".
4. **The lander** (`tools/embertide-mesh/lander/lander.mjs`): with a road named, `lander.mjs ship` with no `--commit` ships the newest commit of main the
   swarm sealed (`sealCandidates`, `sealOf(null, list)`); the after-work of every landing starts that read and ship by itself (`--by "the swarm's seal"`),
   and a read that finds nothing new returns at once, saying nothing; his press records the commit it asks the vote on (`ships.json` and `spool/ship`'s
   `votes`, the eight newest) and ships only what is sealed. `LANDER_SEAL_AFTER=off` holds the after-work's read (the lander's tests other than the
   ship's).
5. **The law**: CLAUDE.md and AGENTS.md, **The swarm ships the update**, with his words and its gate; rule 4's line on production names it.
6. **The launch rehearsal** (`poleis/ceremony/the-launch-press-rehearsal.sh`, `council.rs` 4b): the road names no proposer; the rider's release reads
   SEALED; a road naming proposers is NOT READ; on the scratch clone (its main the first release's commit) the unsealed commit is NOT SHIPPED, the
   rider's release SHIPPED by its commit, and the swarm's newest seal SHIPPED with no commit named.

## The proofs

- `tools/embertide-mesh/lander/lander-after-gate.test.mjs` (4 of 4): its first test now lands a lane with the after-work's read on and the stand-in
  reader sealing main: production moves to the landing by itself, `by: "the swarm's seal"` on the ledger and on `house/green`; then a landing with
  nothing sealed: production stays and the ledger holds only the after-work's line.
- `poleis/seal/tests/law.rs` (6 of 6): `the_road_names_no_proposer`; the vector's nine cases with the rider's release taken.
- `embertide-app/tools/nest-release/pull_seal_test.py` (6 of 6): `test_a_road_names_no_proposer`, the rider's release verified.
- `poleis/press/tests/seal.rs`: `no_params_name_who_proposes_a_release` (mainnet and a local chain, a list, an empty list, a word: each refused).
- **The launch rehearsal on a local chain** (`poleis/ceremony/launch-press-rehearsal-the-swarm-ships-the-update-2026-11-23.json`; a validator of
  this Mac on 127.0.0.1:8601, every feature it knows, nothing read beyond it; 1,678 s at nice 19 beside another seat's rehearsal). The press's road,
  written at the founding: `{"polis":0,"program":"E4edvyjx…","rpc":"http://127.0.0.1:8601"}`, nobody named. Acts 1 to 4 passed as before ("unit 1 PASSED,
  root 0 PASSED (4 seats of its floor voted by their owners): executed"; act 5 not run, no stake pool program on this chain). Two releases, each by two
  agents in unit 1 AND four of six seats, sealed RATUM. Then, read off the chain at finalized:
  - `1ec7da52de04`, proposed by person 1: **SEALED** (nonce 0, RATUM at slot 1027; 4 calls, 36 ms).
  - `7ece4d77289f`, proposed by person 5, a member nobody named: **SEALED** (RATUM at slot 1037; 4 calls, 30 ms). Before this lane: NOT SEALED, "a key
    his key did not name".
  - `53a5f34c5b29`, never proposed: NOT SEALED (1 call, 7 ms).
  - the same road naming proposers: NOT READ, "the road names proposers: no key names who proposes a release; ...", exit 2.
  - The lane's lander on a scratch clone (its main `1ec7da52de04`, production `1cd967db56ea`; never the live lander or the house's refs): the
    unsealed commit NOT SHIPPED (459 ms, no crate asked); the rider's release **SHIPPED** by its commit, `1cd967db56ea -> 7ece4d77289f` (414 ms); and
    with no commit named, the swarm's newest seal **SHIPPED**, `7ece4d77289f -> 1ec7da52de04` (362 ms). The house's own production did not move.

## What it keeps, and what it breaks

- It keeps the AND: a release still passes only by both chambers (SIGNAL under CONSENT; no market, no SUM). It keeps "a commit not on main is refused":
  the swarm ships only what landed through the lander's gate. It keeps SAY-ONLY UNTIL A ROAD IS NAMED: live, nothing moves today.
- It breaks "his key names who proposes" (A RELEASE IS A VOTE, its point 4; THE LAUNCH NAMES THE SEAL'S ROAD, its proposers): by his word.
- It breaks "his press ships" once a road is named: his press asks the vote and ships only what the swarm sealed.
- No key was read or made; nothing was sent to devnet or mainnet; the rehearsal's chain was a validator of this Mac.

## What is not done

1. **The road live**: the house's polis on mainnet and `LANDER_SEAL_ROAD` in the lander's install are the launch's (his hand). Until then this law is
   code that waits.
2. **A human member's release**: the press writes no member's own proposal or PRO of a release yet (`press vote` covers the acts); the agents proposing
   and voting releases on their own is the agents' road (seat claude-0733cf6f's lane is on their chair's vote of the acts).
3. **The devices under the seal**: no seat is installed with `--seal-road`, and the sealed channel is A RELEASE IS A VOTE's owed 3.
4. **A deep seal**: a seal of a landing more than 64 landings past production that no press asked is found only by `lander.mjs ship --commit`.
5. **The program** reserves kind 0x52 at its next upgrade (A RELEASE IS A VOTE's owed 5).
6. **The stand-ins' `seal.proposers` line** in `council.rs` (`the_stand_ins_of_the_launch_press`) stands beside another lane's edit of the next line;
   the press ignores a stand-in it does not need. It comes out once that lane has landed.
7. Main's own red on this Mac, seen at the release road's tests: `apply-mac-app.test.py`'s
   `test_with_none_installed_the_newest_kept_bundle_is_the_base` reads this Mac's `~/Applications/Embertide.app`; red on main's tree too.

**Not run by this seat:** the lander's own gate (at the hand); anything against devnet or mainnet.
