# THE LAUNCH NAMES THE SEAL'S ROAD: the press writes the road to the swarm's seal once it founds root 0 (Nov 23 2026, lane/the-launch-names-the-seals-road, seat claude-42f52af6)

## His words, UTC

- **Nov 23 ~14:00,** "Wire it in", to the question "Wire the seal's road into the launch press instead of using devnet?".
- Behind it: Nov 23 11:58, "the truth is, the swarm deliberates and votes and that lands on solana" (A RELEASE IS A VOTE, landed as 959aa707cb);
  Nov 21, "we arre supposed to launch and go live my dude": the launch is mainnet, and devnet is no step of his. Nothing in this lane was sent to
  devnet, testnet or mainnet, and the press still refuses devnet and testnet by their genesis hashes.

## What stood

- `poleis-seal read <commit> --road R` reads a release's seal by a road `{rpc, program, polis, proposers}`, and `lander.mjs ship` asks it once
  `LANDER_SEAL_ROAD` names one. No road existed: this was A RELEASE IS A VOTE's owed item 1.
- The launch press founds root 0 in the slot Poleis becomes callable, under the sitting's own fresh program ids. Nothing wrote down where its seal
  is read.
- The reader refused every road whose program was not the one its client was built for (devnet's `AJdsmjx…`: "this reader speaks Poleis"). So a road
  to the launch's own Poleis could never have been read, by the lander or by anyone.

## What this lane changes

1. **The press writes the road** (`poleis/press/src/seal.rs`). Once root 0 stands as this sitting's, it writes `seal-road.json` beside the record:
   at the founding (the deploy's same breath, and `press found`), and again, unchanged, at every `press status` and `press certificate`. It holds
   the reader's four fields and nothing else:
   `{"rpc": <the cluster's node>, "program": <the sitting's own Poleis>, "polis": 0, "proposers": [<the keys his key names>]}`.
   The same road goes on the record (`sealRoad`) and on the birth certificate (`sealRoad`, with the file's place said in words, since a certificate
   names no folder). The founding, `press status` and `press certificate` print it with the line the lander takes it by:
   `LANDER_SEAL_ROAD=<the sitting's folder>/seal-road.json` in the lander's install. Naming it there is his hand. The press names it nowhere else,
   and this lane did not touch the live lander's install.
2. **The node is the cluster's.** On mainnet it is the params' `seal.rpc`, named by its host alone: the road is a file the lander reads, and a path,
   a query or a user in a URL may carry a key. `params.mainnet.json` names `https://api.mainnet-beta.solana.com`, the cluster's public node and the
   one the rehearsal's reads of mainnet use by his word; his own node can replace it by its host. The sitting's own `--rpc` is never kept, as before
   (the certificate's own words: "it may carry a key"). On a rehearsal's chain the node is the sitting's own validator on this machine.
3. **The road's params are held before anything starts.** `chain::params_checked`, the first thing `press deploy` does, holds `seal.rpc` on mainnet
   and `seal.proposers` on every chain, so a founding never lands with a road it cannot write. Should one ever fail to be written, it stops nothing
   of the sitting and says why: the road is the lander's, never the chain's.
4. **The reader reads the road's own program** (`poleis/seal`). The launch's Poleis is the sitting's own id, so the reader now derives a proposal's
   and a session's addresses under the program the road names (`proposal_at`, `session_at`, by the program's own seeds `["proposal", hash]` and
   `["session", hash, unit]`), where it used to refuse every road of another program. Under devnet's program they are the client's own addresses,
   so the practice chain's vector reads as before. The road's format is unchanged, and `pull.py` already took the road's program.
5. **The rehearsal proves it whole** (`poleis/ceremony/the-launch-press-rehearsal.sh`, below).

## The polis and the proposers, by the program's own rules

- **The polis is root 0.** A release is SIGNAL under CONSENT, and under CONSENT "every chamber must seal PASSED before the act executes, and each
  seal carries its crews' AND (`seal`), so the origin's seal says it for the whole tree" (`poleis/program/poleis_treasury/src/ix/proposal.rs:58-61`).
  A release sealed at root 0 is the whole tree's verdict: unit 1's chairs, root 0's chair that unit 1 holds, and root 0's floor, the minted
  Lyssajudrons. A top-level release proposal at root 0 is possible, so root 0 it is.
- **Who may propose at root 0.** "The proposer is a member of the origin unit" (`proposal.rs:79`): `propose` holds it by `expect_member`, and
  `propose_nft` by `current_owner` of a live seat of the origin's floor (`proposal.rs:184-188`). A member is "a key holding hexagon seats, or a key
  in a live chair" (`ix/mod.rs:120-131`). Nothing is proposed before the tree is a council (`proposal.rs:202`, `expect_council`), so a release waits
  for `press ratify`. Root 0 is born with no key in its chairs (chair 2 holds unit 1 and the eleven others are empty), and its floor's seats are NFT
  seats, so no key proposes there by `propose`. Only a seat owner proposes there, by `propose_nft`, and the reader takes both kinds
  (`poleis/seal/src/lib.rs:266-277`, `proposer_in`: the proposer is account 0 of `propose`, account 3 of `propose_nft`).
- **Not unit 1's agents.** `Registry::chairs_of` counts only the keys in the unit's own chairs (`state.rs:777-790`), so an agent is a member of
  unit 1 and never of root 0, and cannot propose a release there. A release proposed at unit 1 instead would be unit 1's alone (impacted `[1]`):
  root 0's floor would never vote it, and it would not be the whole tree.
- **So the proposers are his word in the params, `seal.proposers`,** each a seat owner of root 0's floor when it proposes. Naming one gives it no
  vote: the swarm still votes the release. **None is invented.** Mainnet's params name none, so the road names none, and the reader refuses every
  seal ("NOT READ: the road: the road names no proposer"); the press says so plainly at the founding, in status and on the certificate. A value
  that is not his yet (`PLACEHOLDER`, `STAND-IN`) is refused, never read as an address. The rehearsal's params name one stand-in, person 1.

## The proof: the launch rehearsal, whole, on a validator of this Mac

Run at the lane's code commit b8f1d6dfcc (the programs' parts as that commit holds them), every feature the validator knows and no Metaplex, the
validator (4.2.0) on 127.0.0.1:23410, everything at nice 19, stand-in keys made in memory from the rehearsal's seeds. Nothing left the Mac: the
application firewall saw 0 inbound flows to the validator, and the validator, its ledger and the sitting's folder were removed at the end. The
certificate: `poleis/ceremony/launch-press-rehearsal-the-seals-road-2026-11-23.json` (its `sealRoad`, with the reads and ships below, and its checks'
`sealRoad`).

| step | what happened |
|---|---|
| the founding, in the deploy's breath | one transaction of 887 bytes, 32,125 CU, as before. Then `seal-road.json` beside the record: `{"polis":0,"program":"DzJWvZs7cPMh9avtdM9KxijA1ixhKaWL7BJCHMTr9oG6","proposers":["A3XWqW2BN9tsiGokjdEC4GZCHjqHgaoC1LpiwkHan2T2"],"rpc":"http://127.0.0.1:23410"}`, 186 bytes: the sitting's own Poleis, root 0, its validator, person 1 |
| `press found`, the founding read back off the chain (the fault 6b), `press status`, `press certificate` | each said the same road again, and status and the certificate gave the lander's line, `LANDER_SEAL_ROAD=<the sitting's folder>/seal-road.json` |
| the checks read the road | its node, its program (the sitting's Poleis, the checks' own build) and its one proposer (person 1, a seat owner since slot 245) held |
| release of 1ec7da52de04, proposed by person 1 (named) | SIGNAL under CONSENT of kind 0x52 by `propose_nft`; unit 1 convened, two agents of three, unit 1 sealed, its chair heard in root 0, four of six seats PRO by their owners' own transactions; root 0 sealed RATUM. Slot 1015 to slot 1024, 10 transactions, 109,510 CU |
| release of 7ece4d77289f, proposed by person 5 (not named) | the same votes, RATUM. Slot 1025 to 1034, 10 transactions, 108,066 CU |
| the reader (the house's `poleis-seal`, at finalized) on the press's road | `1ec7da52de04`: **SEALED**, "polis 0's proposal 5fbac91b70e6 (nonce 0), class SIGNAL, RATUM at slot 1024, proposed by A3XWqW2B…", 4 calls, 51 ms. `7ece4d77289f`: **NOT SEALED**, "nonce 0: proposed by 7t5bmub1…, a key his key did not name", 1 call, 12 ms. `53a5f34c5b29`, never proposed: **NOT SEALED**, "no proposal of polis 0 names 53a5f34c5b29 (nonces 0 to 7)", 1 call, 12 ms |
| the same road naming no proposer | **NOT READ**, "the road: the road names no proposer: his key names who may propose a release", 0 calls |
| the ship: the lane's `lander.mjs ship --commit`, on a scratch clone of the house (it shares the house's objects, read only; its own refs: main the house's, production at 1cd967db56ea; `LANDER_SEAL_ROAD` the press's road; no Nest) | `53a5f34c5b29`: **NOT SHIPPED**, "is not sealed (no proposal of polis 0 names 53a5f34c5b29 ...): production stays on 1cd967db56ea; no crate is asked", 779 ms. `7ece4d77289f`: **NOT SHIPPED**, "... a key his key did not name", 402 ms. `1ec7da52de04`: **SHIPPED**, "production 1cd967db56ea -> 1ec7da52de04, 1 landing, ... sealed at slot 1024", 607 ms (no crate under its 6 files) |
| what the ship moved | the scratch clone's production is the sealed commit; the house's own `refs/heads/green` stayed where it was (88801824d5) |
| the whole rehearsal | green: 1,606 s from the start to the certificate (the checks 1,510 s, 150 transactions, 10 refusals asked for), 1,611 s in all |

The commits are A RELEASE IS A VOTE's own practice commits, so the numbers compare: its reader read a seal in 4 calls and 30 to 49 ms on its
practice chain, and shipped `1ec7da52de04` in 0.58 s.

## The tests, red on main, green on the lane

- `poleis/press/tests/seal.rs`, 3 new: the road is root 0 of the sitting's own Poleis and holds the reader's four fields and nothing else, written
  whole and once (`the_road_is_root_zero_of_the_sittings_own_poleis_and_nothing_else`); a road that names no proposer says the reader refuses every
  seal, and a value not his yet is never read as a key (`a_road_that_names_no_proposer_says_the_reader_refuses_every_seal`); the node is the
  cluster's, mainnet's by its host alone and never the sitting's own endpoint (`the_roads_node_is_the_clusters`). On main's source it does not
  compile (`unresolved import poleis_press::seal`).
- `poleis/press/tests/params.rs`, six new faults in `a_value_the_sitting_cannot_send_stops_it_before_anything_starts`: an http node, a node with a
  query, no node, proposers that are no list, an entry that is no address, a placeholder. On main's source it is red:
  `seal.rpc = "http://api.mainnet-beta.solana.com" must stop the sitting`.
- `poleis/seal/tests/law.rs`: `the_road_names_its_program_and_its_addresses_are_that_programs` is new, and `the_road_is_his_hand` now holds a road
  whose program is no address to its refusal. On main's source it does not compile (`cannot find function proposal_at`, no method `id` on `Road`).
- The whole crates, before the hand: `poleis/press` 74 of 74 and `poleis/seal` 6 of 6 (run again at the tip after the red checks: 80 of 80), and
  `poleis/program/poleis_treasury` 223 of 223, 4 ignored (the live ones, which the rehearsal runs).

## The cost

| | before | predicted | measured |
|---|---|---|---|
| the founding | one transaction, nothing of the seal written | the same transaction, one file beside the record, no call (the road is derived from the params and the ids) | the same founding (887 bytes, 32,125 CU); `seal-road.json` 186 bytes, written once, the same bytes never written again |
| `press status`, `press certificate` | | root 0 held first (its config and its registry, the founding verified on the record): three getAccountInfo more for status, two for the certificate; the road written only when it changed | not timed apart; the certificate's reads in all are on its `press.rpcCalls` |
| the reader, per ship | 4 calls, devnet's program alone | 4 calls: the 16 addresses derived under the road's program, microseconds | a seal 4 calls, 51 ms; a refusal 1 call, 12 ms; a road with no proposer 0 calls |
| the ship, per press | the seal receipt's 0.58 s shipped, 0.3 s refused | the same | 607 ms shipped; 402 and 779 ms refused, no crate asked |
| a release on the chain | | 10 transactions of one signature each (the proposal, unit 1's convene, the agents' one batch, unit 1's seal, its mandate, four seats' own votes, root 0's seal): 50,000 lamports of fees, and the proposal's and the sessions' rent, which comes home at close | 109,510 and 108,066 CU; 9 slots from the proposal to RATUM |
| the rehearsal | 1,520 to 2,564 s (its earlier certificates) | the same, and under a minute more for two releases, four reads and three ships | 1,606 s |
| moving parts | | one module, one params section, a reader change, a rehearsal step | Rust +463 −20 (the press 236 −10 and its tests 130, the reader 45 −9, the checks 52 −1); shell +93 −4; JSON +11; JavaScript 0 |

## What it keeps, and what it breaks

- It keeps his authority. No key of his was read or made, no keychain, wallet or wallet file was opened, and no agent signs as him: the stand-ins
  were in memory, and the road holds no key. THE WHOLE TREE DECIDES holds: `seal.proposers` names no chair and gives no vote, and his Keystones are
  still refused as a param.
- It keeps "devnet is no step of his": the press's clusters are still mainnet and a validator of this machine, held by their genesis hashes, and
  nothing was sent beyond 127.0.0.1.
- It adds no door: the road is a file, and the reader is a client of a node, as before. SAY-ONLY UNTIL A ROAD IS NAMED is unchanged: until his
  hand names `LANDER_SEAL_ROAD`, the live ship says "not sealed" and goes on.
- **It breaks the reader's pin to its client's program.** Before, a road file could only say who proposes, on devnet's program; now it names the
  program too, read under that program's own id, as `pull.py` already did. The road is his hand's whole: until his genesis key signs it on the Nest
  (owed, below), whoever can write the install's file can name a program as well as a proposer.
- **It breaks an older params file:** `params_checked` refuses a file that does not say `seal` (on mainnet its node, on every chain its proposers).
  The two files the press ships say them.
- `press status` and `press certificate` now hold root 0 as this sitting's before saying the road: three reads more for status, two for the
  certificate.

## Owed

1. **The proposers signed by his genesis key on the Nest** (A RELEASE IS A VOTE's owed item 4). Today they are the params' word, and mainnet's names
   none, so the reader refuses every seal until he names one, each a seat owner of root 0's floor.
2. **His press names the road in the lander's install:** `LANDER_SEAL_ROAD=<his sitting's folder>/seal-road.json`. That is his hand; nothing here
   did it.
3. **Kind 0x52 reserved at Poleis's next upgrade** (`SIGNAL_RELEASE`): his press.
4. The press does not check that a named proposer holds a seat of root 0's floor. A proposer without one cannot propose (Poleis refuses it), so
   nothing is sealed by it; a seat read at status would say so sooner.
5. One node a road (A RELEASE IS A VOTE's owed item 7), and the public node is rate-limited: his own node by its host, or several, or a light client.
6. A RELEASE IS A VOTE's other owed items stand as they were: his press names the sealed commit, the sealed channel for the devices, the reader's
   fallback to the transactions once a proposal is closed, proposals made through another program.
