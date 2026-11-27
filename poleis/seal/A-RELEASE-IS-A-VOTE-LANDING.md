# A RELEASE IS A VOTE (Nov 23 2026, seat claude-da693118)

His words, UTC, to seat claude-42f52af6: ~11:3x "what would happen if we were to migrate to a self-hosted git alternative?", "how do we make so the
ci/cd is portable?... i'd want the nest to be the nucleus for deployments...", "the loop is king"; 11:58 "the truth is, the swarm deliberates and votes
and that lands on solana"; ~12:4x "build all?". The record: ec-workspace `docs/posiverses-and-neghiverses-hypertrace-2026-10-30.md`, its Nov 23 11:58
section: "The truth of a release is the vote landed on Solana, naming a hash... The Nest and every device read the chain and carry out what landed;
they decide nothing."

## What stood

- `lander.mjs ship` moved `refs/heads/green` (production) on his press once the crates passed. Nothing asked the chain.
- `pull.py` took any release signed by the release key. Nothing asked the chain.
- Poleis (`poleis/program`, devnet `AJdsmjx…`) seals proposals by the AND of its two chambers. Nothing of the house read a seal.

## What this lane changes

1. **One reader of the seal, in Rust** (`poleis/seal`, the `poleis-seal` crate). `poleis-seal read <commit> --road R` answers whether a Poleis
   proposal of the house's polis names that commit and is sealed RATUM by a proposer his key named, and gives the proposal, its class, its nonce,
   the seal's slot and transaction, and the proposal's transaction. The law is pure (`src/lib.rs`); the hand reads a Solana node by curl at
   `finalized` (a seal that could still roll back is no seal yet) and writes nothing. The road `R` is his hand's: `{rpc, program, polis, proposers}`.
2. **The class is SIGNAL, under CONSENT, of kind 0x52.** This comes from the program's own rule (lib.rs, over MARKET_CLASSES): "The law (CODEX) and the signals
   (the code, the weights, the engine's own rulings) stay under the AND for ever." The house's code is a signal by that line. No market can decide it
   (`market_bit(SIGNAL)` is 0) and no SUM can (`sum_bit(SIGNAL, 0x52)` is 0), so both chambers must pass it. It is not CODEX: CODEX is the society's own
   law and the program's own bytes, under the long window (RELIGIO Art. VII, "Codex est res HEXAGONORUM"). It is not ACT either: ACT is the hands, and it
   may go to the market. CONSENT adds the crews of the chairs, so the origin's seal alone says that the whole tree passed. Kind 0x52 is one the program
   does not execute: its `execute` refuses it (BadPayload). Nothing of a release happens on the chain; the house carries it out.
3. **The name.** A release proposal of commit C in polis P at nonce n has class SIGNAL|CONSENT, origin P, manifest C (zero-padded to 32 bytes),
   payload `[1, 0x52, C]` and impacted `[P]`. Its hash is the client's own `proposal_hash`. The program checks no hash against its content, so the
   reader checks every field of the account against the name, never the hash alone. A squatter on a nonce's address is passed over, and eight
   nonces are read.
4. **His key names who proposes.** Any member may propose, but the house takes a seal only of a proposal whose proposer is on the road's
   `proposers`. On the practice chain a rider proposed a real commit, both chambers passed it, it was sealed RATUM, and it was refused at both ends.
5. **The ship takes only a sealed hash.** `lander.mjs ship [--commit <hash>]` (main by default) asks the reader before any crate. It refuses a commit
   that no seal names or one that is not on main; while no road is named it only says so (below). It says so once: on the run's log, on the ledger
   (`verdict: unsealed`) and on `spool/ship` (`tests.state: unsealed`). A shipped commit carries its seal (polis, proposal, slot, signature) on
   `house/green` and on the ledger. The reader runs through `poleis/seal/seal-read.sh`, which builds it in the served house once, niced, as
   `one-space-read.sh` does.
6. **A device refuses an unsealed release.** `publish.mjs --seal F` signs the reader's seal into the manifest, and it refuses a seal whose commit is not
   the release's `source_commit`. A seat installed with `pull.py --seal-road R` takes a release only when its seal names its `source_commit` and the
   chain says so. That check reads two transactions, the seal's (its receipt RCPT_SEAL, PASSED, read only inside the program's own frame of the logs)
   and the proposal's (a top-level `propose` of that hash, its proposer a signer). It never reads the accounts, so the check still holds after a proposal
   is closed. `pull.py` stays stdlib Python, the devices' road by design: it holds the same law as the Rust, and both are held to one vector
   (`tests/vector.json`, the practice chain's own finalized transactions; `pull_seal_test.py`, `tests/law.rs`).

## The proof (the practice chain: a validator of this Mac on 127.0.0.1:23310, devnet's own Poleis bytes, sha256 `92f3bcf2…`, lane a-rider-steers' way; stand-in keys; nothing left the Mac)

| step | what happened |
|---|---|
| founding (`examples/practice.rs found`) | 11 founders in the chairs, 3 seats on the floor (two riders), sold and ratified: 14 transactions, 36 s |
| red: `read 1ec7da52de04` (main's tip) before any vote | `NOT SEALED: no proposal of polis 0 names 1ec7da52de04 (nonces 0 to 7)`, 1 call, 15 ms |
| nonce 1: 6 chairs PRO, the whole floor PRO | the window waited (79 s) and the seal came out DISSENSIO: the chairs' bar is 7, not 6. In the vector, refused: "sealed it DISSENSIO" |
| nonce 2: 8 chairs PRO, the whole floor PRO | sealed RATUM by the fast path, 2.8 s from the proposal to the seal, slot 423 |
| green: `read 1ec7da52de04` | `SEALED: polis 0's proposal 793e5e41db17 (nonce 2), class SIGNAL, RATUM at slot 423`, 4 calls, 30 to 49 ms |
| `7ece4d77289f` proposed by a rider and sealed RATUM | `NOT SEALED: nonce 0: proposed by HCMu6k48…, a key his key did not name` |
| `53a5f34c5b29` never proposed | `NOT SEALED: no proposal of polis 0 names it` |
| the ship (lane's lander, a scratch clone of the house with its own refs, production at `1cd967db56ea`) | `--commit 53a5f34c5b29` and `--commit 7ece4d77289f`: `NOT SHIPPED: ... is not sealed (...): production stays on 1cd967db56ea; no crate is asked`, about 0.3 s each. `--commit 1ec7da52de04`: `SHIPPED: production 1cd967db56ea -> 1ec7da52de04, 1 landing, ..., sealed at slot 423`, 0.58 s (no crate under its 6 files), and the seal is on the ledger |
| the pull (lane's pull.py, a scratch store read as its folder, a scratch release key) | the sealed release (`1ec7da52de04`'s `embertide-app/tools/nest-release`, 121 files) was **installed**, 0.43 s for the whole pass. `53a5f34c5b29` published bare: `reject: NOT SEALED, not taken: the release carries no seal`, 0.11 s, no read. `7ece4d77289f` with the rider's real seal: `reject: ... the seal's proposer HCMu6k48… is a key his key did not name`. Current stayed on the sealed release throughout |
| the publisher | `53a5f34c5b29` given `1ec7da52de04`'s seal: `--seal ...: its commit 1ec7da52de04... is not this release's source_commit 53a5f34c5b29...` |
| red on main's code | main's `pull.py`, same store, a fresh seat: **installed** the rider's release `7ece4d77289f`. Main's lander, with the reader saying "not sealed": **SHIPPED** |

## The tests, red then green

- `tools/embertide-mesh/lander/lander-after-gate.test.mjs`. Main's `lander.mjs` is red: the stand-in reader says not sealed and it SHIPPED. The
  lane's is green, 4 of 4 in 21 s. The test now holds the refusal (an unsealed commit, no road named, a commit that does not exist), the seal on
  `house/green` and on the ledger, and `SHIPPED ... sealed at slot 7`. The fixture (`lander-fixture.mjs`) carries a stand-in reader: sealed unless
  `F.root/unsealed` stands.
- `embertide-app/tools/nest-release/pull_seal_test.py`, run by `mac-app-release.test.mjs`'s list: 5 of 5. The vector's nine cases are one seal and
  eight refusals: another commit, a rewritten commit, the rider's proposal, the rider's proposal claimed as his, the proposal's transaction handed in
  as the seal's, a slot off by one, DISSENSIO, and transactions that are not on the chain. It also holds that a release with no seal is refused without
  a read, that a sealed release takes 2 reads, and that a refused release is read once and not again on the next passes.
- `poleis/seal/tests/law.rs`: 5 of 5 (`cargo test`, at the ship: a crate). The names are computed by Python and checked by Rust. A receipt counts
  only inside the program's own frame: not another program's frame, not a CPI's frame, and not a `Program log:` line.
- Neighbours, all green: pull_hold, pull_switch, pull_through, pull-required-hook, pull-runs, `pull.py --selftest`; publish-commit, publish-dry-run,
  publish-one-road, release-road (13); lander-iphone, lander-ipad, lander-ipad-cut, lander-one-road (53).

## The cost

| | before | predicted | measured (loopback) |
|---|---|---|---|
| per ship | his press, then the crates (19 to 24 min) | one read before the crates, at most 5 calls, under 1 s | 4 calls (getMultipleAccounts of 16 accounts, getSignaturesForAddress, 2 getTransaction), 30 to 49 ms for the read; a refusal 0.3 s end to end and no crate run; over https to a public node, unmeasured (owed with the polis on devnet) |
| per pull | 0 reads of the chain | 0 at rest; 2 per new release; a refusal read once | 2 getTransaction, 16 to 17 ms (`poleis-seal verify`); a whole sealed pass 0.43 s; a release with no seal 0.11 s and 0 reads; a refused release is not read again (the unit test counts 4 reads over 1 good release and 3 passes of a bad one) |
| time to verdict on the chain | none | a window, or the fast path | 2.8 s from the proposal to RATUM by the fast path; 79 s for a window that had to close |
| on the chain | none | rent for the proposal and its session, two fees | propose: 5,000 lamports fee and 10,008,480 rent (it comes home at close), 16,909 CU; seal: 5,000 lamports, 8,576 CU |
| moving parts | none | one crate, one wrapper | `poleis/seal` (lib 311 lines, main 179, the practice example 146), a 600 kB binary, built once by `seal-read.sh` (13.6 s, warm cache); +164 −2 lines of Python in `pull.py`. JavaScript, by its gate: +31 −32, net −1 (`lander.mjs` +20 −21, `publish.mjs` +6 −6, the ship's test +3 −3, the fixture and a test list +1 −1 each; the two copies of `isAncestor` folded into one) |

## What it keeps, and what it breaks

- It keeps his authority: the reader signs nothing, the practice keys are stand-ins, and mainnet, his keys and production are untouched (the ship
  ran on a scratch clone). Nothing of the room goes to the chain; a proposal carries only a commit's hash. It adds no new door: the reader is a
  client of a Solana node, like `embertide-stake`'s.
- **Say-only until a road is named** (his ~13:15 UTC "dude i dont fucking know, do whatever, we have nothing live no users fucking deliver on shit",
  ruled by the seat that started this lane). With no `LANDER_SEAL_ROAD`, the ship goes on as before to its crates and says `not sealed: <commit>
  (why)` on the run's log, the ledger, `spool/ship` (`tests.seal`) and the shipped row (`house/green`'s `seal.not`, the changelog's `shipped[0].seal`).
  Once a road is named, the refusal holds as above; a node that does not answer is a seal not read, and nothing ships. A commit that is not on main
  is refused in both modes. The devices are opt-in (`--seal-road`). Both modes are in `lander-after-gate.test.mjs` (4 of 4) and on the scratch clone:
  with no road, `SHIPPED: production 1cd967db56ea -> 1ec7da52de04 ... not sealed: 1ec7da52de04... (no road to the seal is named ...)`; with a road and
  its node down, `NOT SHIPPED: 53a5f34c5b29 is not sealed (getMultipleAccounts: ... Couldn't connect to server)`.

## Owed (named, not built)

1. The house's polis on devnet, then mainnet, and its road file (rpc, polis, the proposers his key names): his presses. Since Nov 23 (his "Wire
   it in"), the launch press writes the road when it founds root 0 on mainnet, never devnet, and the reader reads the road's own program
   (`poleis/press/THE-LAUNCH-NAMES-THE-SEALS-ROAD-LANDING.md`); the proposers and the install's `LANDER_SEAL_ROAD` stay his.
2. His press names the sealed commit. `spool/ship-ask` ships main, and main is rarely the commit that was sealed. The Spool's changelog row of a
   sealed landing should carry its commit into the press (`--commit`).
3. The sealed channel. Releases are cut from main at every landing, so a seat under the seal refuses them all and waits. Production's release should
   be published at the ship with its seal, on a channel the sealed seats read. Today no seat is installed with `--seal-road`.
4. The proposers on the Nest, signed by his genesis key; today they are the install's file.
5. The program reserves kind 0x52 (`SIGNAL_RELEASE`) at its next upgrade: his press.
6. The ship's reader reads the accounts first. A release proposal becomes closable by anyone `MAX_WINDOW_SLOTS` (about 6 days) past its expiry, and
   after that only its transactions say what it was. The device's check already reads only those; the ship's reader has yet to fall back to them.
7. One node trusted per road: several nodes, or a light client.
8. Top-level proposals only: a proposal made through another program (a CPI, the swarm's own multisig) is not read.
9. The stores (App Store, Play Store), the downloads and the notifications (Android push): after this arc.
10. The record's own measure, still with no gate: a release a device took whose hash no vote on Solana sealed, read at the after-work.
