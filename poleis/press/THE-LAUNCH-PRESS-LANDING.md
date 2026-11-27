# THE LAUNCH PRESS: the sittings his hand runs to put Poleis, the market and the $WARM program on mainnet, rehearsed whole under mainnet's rules (Nov 21 2026, lane/the-launch-press, claude-81203e94, then claude-9997bc17)

## His words, UTC

- **Nov 21 16:49:02:** "but wdym putting it on devnet? we arre supposed to launch and go live my dude".
- **17:34:37:** "it is supposed to be uncapped supply, prompt-2-mine; the issuance needs to be live and continuous based on the
  space-router utilization".
- **20:07:25,** to "Build the mainnet launch press next?": "Yes, build it".
- **21:22:37,** to "May the press's rehearsal read mainnet's feature list and copy Metaplex's program from it?": "Yes, read them".
- **21:36:32:** "so?"

The seat changed in the middle. At 21:42:40 UTC the account of the desktop app was switched, and claude-81203e94 and its builder ended;
the builder's rehearsal finished 61 seconds later with nobody to read it. claude-9997bc17 took the lane at 1f7e62fe2d (twelve commits,
about 8,000 lines, unreviewed), read every line against the brief, Poleis's own code and his launch plan's ceremony (ec-workspace
`docs/dyson-society-launch-plan-2026-09-09.md`, section 6), changed what "What the review changed" says, and ran everything below itself.

## Cost first

| | before (main 3f90cf2eca) | after (this lane) |
|---|---|---|
| a road to mainnet | the ceremony of Oct 28: JavaScript, devnet only, its keys plain files a seat reads, the code never handed to the seal | `poleis/press`, a Rust program: 4,210 lines with its tests, 253 crates from the new Mac's cache (none fetched), no JavaScript |
| the three programs, built for a deploy (sBPF v3) | Poleis 456,800 bytes, the market 101,968, the $WARM program 50,424 (the tests' v0 builds, which the loader refuses to deploy today) | Poleis 434,256 bytes, the market 98,512, the $WARM program 56,544 (with the token's face) |
| what the first sitting needs on mainnet | "about 2.9 SOL" for two programs (16:56 UTC) | about 3.06 SOL for the three and everything the sitting opens, at mainnet's rent of 5,080 lamports a byte (read 22:37 UTC; $214 at $70.11): the programs' data 2.99634148, their accounts 0.00249936, root 0's four accounts 0.0137922, the mine with its face 0.01875284, the board 0.0017018, fees about 0.003, a margin of 0.02. The press reads the rent itself and says each line before it starts |
| what the acts cost his Keystones | | on the local chain Keystone 1 spent 0.05186152 SOL on the four acts (the proposals' and sessions' rent, the fees), Keystone 2 0.00004, Keystone 3 nothing; at mainnet's rent about 0.038 for Keystone 1 |
| the rehearsal, under mainnet's rules | never run with Metaplex; the builder's one run (every feature, no face, the handoff first) read by nobody | run whole at the tip: 532 transactions of the press, 110 of the checks and 3 refusals, 1,859 s on a loaded new Mac (1,647 s on a quieter pass) |
| the rehearsal, every feature | | 532, 106 and 3 refusals, 1,906 s |
| the largest transaction, the dearest | | 1,232 bytes of 1,232 (each buffer write is cut to fill a packet); among the checks 1,010 (a batch of four votes). The dearest: `init` with the token's face, 63,209 CU (54,209 and 75,209 in two other runs: each run's keys make its address searches); no transaction asks a compute budget |
| the token's face | not built: `init` made a mint with no name | made by Metaplex's own program inside `init`, moved by the polis's sealed act (41,323 CU), name and symbol never; Metaplex kept its fee of 0.01 SOL in the metadata account, as the press had counted |
| the tests (the new Mac, at bd7f5e307b) | the $WARM program 31; Poleis 222 and 2 ignored; the market 26; the client 20 | the press 30; the $WARM program 38; Poleis 223 and 4 ignored (the two live tests of this rehearsal); the market 26; the client 20; no warning |
| faults put in on purpose | | 22 in the sources, one at a time, each caught by a test named for it (46 s for the 22); 8 more inside every rehearsal, each refused or mended |
| JavaScript | | none added (`git diff main --numstat` over .mjs, .js, .cjs, .ts, .html: empty) |
| classical signature lines | | 2 added, each marked `classical: Solana's fee payer` (the press's `Cargo.toml` and `wire.rs`); `council.rs` stays at its 4; the house's gate exits 0 |
| what a rehearsal costs | | a validator at nice 19 on the new Mac for 27 to 32 minutes; nothing on the serving Mac; under mainnet's rules, two reads of mainnet's public RPC at the validator's genesis and nothing sent there |

The diff against main: 33 files, +23,172 −19 before this receipt, 14,346 of them the two certificates and 2,680 the press's lock. No source
of Poleis or of the market moved. The $WARM program gained its face (`face.rs`, `init`'s optional face, `set_face`). Nothing is deployed,
nothing was sent to any cluster, and no key of his was made, read or asked for.

## What the press is

One program, `press <step>`, and a folder that is one sitting's (`--dir`: its record, rewritten whole after every transaction; its keys in
`keys/`, mode 700). Every step reads the chain first: what stands is read back, held to the record, and skipped, so a sitting cut anywhere
resumes where it stopped and says so.

**The first sitting** (`press keys`, `press build`, then `press sit`):

1. **`keys`.** Four keypairs made from the system's randomness, in the Solana CLI's file shape, each mode 600: the three program ids and
   the deployer. It prints the four addresses and nothing else, never writes over a key, and refuses a folder inside a git repository.
2. **`build`.** The house's programs copied into the sitting's own copy, each `declare_id!` set to its fresh id and Poleis's
   `FUND_ADDRESS` and `FUND_BUMP` to that id's own; each built for sBPF v3 by the tests' own command; each binary's sha256 recorded; the
   sitting's own press built from that copy and held to the house's derivation of every id and address. On mainnet it refuses a house
   whose programs' parts hold a change no commit holds, and says the commit it builds.
3. **`need`, `deploy`.** The SOL the sitting needs from here, at the chain's own rent, each line said; short of it, nothing starts. Every
   value of the params is held to the programs' own bounds before a byte is sent. Each program goes through a buffer named by the
   deployer and a seed (found again with no key of its own), written in passes that read it back, deployed at its own length, read back
   and held to the recorded hash. The deploy sends only recorded bytes.
4. **Root 0 in the deploy's breath.** `init_config` and `open_genesis` in one transaction, signed before Poleis's deploy is sent and sent
   the moment that deploy's slot has passed. What stands is then held to the sitting whole (below).
5. **`warm`.** The $WARM program's `init`, signed by the deployer as its upgrade authority (no stranger can open the mine): its authority
   the polis's agent PDA, its treasury and dollar the params', its opening the params' (an instant, or the founding's own block time), its
   face by Metaplex, no miner.
6. **`market`.** The market's board, opened for the polis's agent key.
7. **`acts`.** Four prepared sets for his Keystones, in order: the fund's constitution, the board's law, the board's roles, the market.
   The press sends nothing a Keystone signs and holds no Keystone's key. It then says what the deployer still holds.

**The second sitting**, once the seal has acted (`press handoff`, `press certificate`):

8. **`handoff`, last.** Each program's upgrade authority set to Poleis's `upgrade` PDA and read back. It refuses until root 0 is a
   council and the fund is constituted. With his own first seat (the road of Oct 28, Stage 1: he alone holds the chairs and the floor)
   that is the same evening as the first sitting; with a public sale it is the sale's length.
9. **`certificate`.** The birth certificate, read back from the chain: every binary's hash and the bytes each program runs, the ids,
   every transaction's signature, slot, units and fee, the founders, each program's final authority, the rent, the params, what the
   deployer was asked and what it spent. It names no folder of the machine it was made on.

On mainnet every step that reads a key file asks his finger first, by the house's own asker and no other. Mainnet is reached by https
only and held to its genesis hash; devnet and testnet are refused by theirs; a rehearsal talks to 127.0.0.1 or [::1] and nothing else.

## What the review changed

**1. The handoff is last.** The lane handed the three programs to the seal in the first sitting. Poleis's own code says what that risks:
root 0 is frozen in genesis until its genesis sale is settled with a seat claimed and it is ratified (`ix/council.rs`,
`ratify_genesis`), and nothing is proposed before (`ix/proposal.rs`, `expect_council`). A sale that seated nobody would never ratify, and
the three programs would answer for good to a seal that could never act: no upgrade, no miner named, no law on the board. His launch plan
orders the ceremony the safe way (section 6: constitute the fund; ratify once the root's floor has a claimed seat; then hand the upgrade
authority). Now `handoff` waits for root 0 to be a council and for the fund to stand, the seal's first act, voted by the chairs and the
floor. Until then the deployer holds the code and can mend it.

**2. Root 0 is held to the sitting whole.** The lane read a standing root back by its twelve chairs. Poleis lets the first wallet found
root 0, and a stranger who saw the founding fly could seat the same chairs under a sale and a plan of its own. Now a root in genesis is
this sitting's only when its authority is the deployer (Poleis makes a founding's payer the tree's authority until ratification) and its
default mint, charter, plan, window, rung, reserve, deposit, name, chairs and sale are the params', its sale's windows the ones the
founding sent. Held so, the record says the founding is verified, and a council is this sitting's by that record.

**3. The params are held before anything starts.** A value Poleis would refuse at the founding (a window, a rung, a sale's window, a name
too long) stopped the lane's sitting only at the founding itself, after the deploy, with root 0 open to the first wallet while the params
were mended. `chain::params_checked` now reads every value against the programs' own constants as the first thing `deploy` does.

**4. A cut sitting resumes with what it holds.** Three things a failing rehearsal and a reading of the code found. The need counted every
write of a standing buffer again, so a deploy cut after 128 writes came back short by the 128 fees it had already paid (0.00064 SOL in
every run before: on mainnet his resumed sitting would have been refused for that much); it now counts only the chunks that still differ.
A fault of the endpoint in the read that follows a confirmation ended the step before the record was saved, so root 0 could stand on the
chain and not in the record; a landed transaction is now always kept, and a founding missing from the record is read back off the chain.
And three passes over a buffer that land no write now stop the deploy (the lane's loop sent for ever, each try a fee).

**5. Smaller.** On mainnet `build` refuses uncommitted parts. The certificate carries no path of the Mac it was made on, and an endpoint
that will not list the feature program's accounts stops no certificate. The rehearsal's validator keeps its whole ledger (it keeps a few
minutes by default, and the certificate's own reads of the buffers' and the acts' history came back empty). The checks say what the four
acts cost each Keystone.

**6. The rules are functions of their own.** The root's hold, the handoff's order and the finger's asker sat inside steps only a
27-minute rehearsal could exercise. They are `founding_held`, `handoff_ready`, `asker`, `finger` and `check_private`, each with its faults
beside it (`tests/root.rs`, `tests/finger.rs`, `tests/params.rs`).

The builder's own choices that stand: the programs built for sBPF v3 (the loader refuses the tests' v0 today); the named buffer; Solana's
own message compiler with every hand-written instruction held to the interface crates byte for byte; Metaplex's two instructions held to
its SDK; the Keystone page's prepared road mirrored in Rust and held to a vector the page's own functions made; the asker held to the
house's committed bytes.

## What the rehearsal proved

`poleis/ceremony/the-launch-press-rehearsal.sh`, on a fresh `solana-test-validator` 4.2.0 holding nothing of the house at genesis, with
stand-in keys made in memory from two seeds the script holds (never a file):

- **The deploy, on a real loader:** three programs through named buffers at their own length (98 + 56 + 430 writes), each read back and
  held to its recorded hash; short of its SOL the deploy starts nothing (exit 3); killed at Poleis's hundredth write, it resumed from
  the chain with 128 of 430 chunks standing and needed exactly what it held (0.0659672 SOL, nothing added). The certificate lists the 64
  writes the killed pass landed and never recorded, read off the buffer's own history.
- **Root 0:** founded two slots after Poleis's deploy (slots 40 and 42), in one transaction of 935 bytes and 24,644 CU, held to the
  params whole; taken out of the record, it was read back off the chain as the same transaction.
- **The token's face, by Metaplex's own program** (its mainnet bytes): `init` made the metadata (SWARM, $WARM, the mine its update
  authority), and the polis moved its URI by a sealed act; the name and the symbol did not move.
- **The order:** the handoff asked after the first sitting was refused (genesis); asked once the stand-in floor had bought six seats and
  root 0 was ratified, refused (no fund); asked once the stand-in Keystones and the floor had sealed the four acts through the Keystone
  page's own bytes, done: three transactions of 2,370 CU.
- **The code is not frozen after it:** the seal moved the market's upgrade authority to a key by a sealed `CODEX_UPGRADE_AUTHORITY`
  (19,011 CU) and the key handed it back.
- **The world works:** THE TWO PROGRAMS IN ONE WORLD's chain on the press's board (verdicts [1, 2], 0.5 SOL paid once, each stake back,
  the book 35,000 of 35,000), and a spend mined on the press's mine at the law's rate (4 $WARM for 4 dollars, never past the deposit).

| the run | at | rules | the press sent, the checks sent, refusals | wall | the certificate (beside the script) |
|---|---|---|---|---|---|
| mainnet's rules | bd7f5e307b | mainnet-beta's 251 active features (sha256 of the sorted ids `b5aecdc0c1b5286b…`, the set the market's run took) and Metaplex's program, both read from mainnet's public RPC at the validator's genesis by his word | 532, 110, 3 | 1,859 s | `launch-press-rehearsal-mainnet-rules-2026-11-21.json` |
| every feature | 528521ae7d (one commit before the code's last: the need of a cut sitting not exact yet) | the 284 features 4.2.0 knows, nothing read beyond the Mac, no Metaplex (the face's CPI held to Metaplex's SDK by `tests/metaplex.rs`) | 532, 106, 3 | 1,906 s | `launch-press-rehearsal-every-feature-2026-11-21.json` |

Earlier runs, not kept as certificates: the builder's, every feature, at 7e6feecc80 (the handoff first, no face; 101 checks'
transactions, 1,549 s; read by this seat from its log); this seat's under mainnet's rules at 1f7e62fe2d (the builder's tip: the face made
by Metaplex's own program for the first time; 105, 1,634 s) and at 85b073594d (the new order; 110, 1,647 s; before the validator kept its
ledger, so its certificate read no history). One run failed and is the fourth change above: at 528521ae7d, two rehearsals at once on a Mac
at load 65, the faucet's top-up after the cut never reached the need (exit 1 after 89 s).

## The faults

**In the sources, one at a time, each caught by the test named** (the new Mac, at bd7f5e307b): a window Poleis would refuse passing the
params; a commit window no longer than a blockhash's life; a root founded by another payer taken for ours; a root with other chairs; a
root under another plan; a root whose sale closes elsewhere; a handoff that does not wait for a council; one that does not wait for the
fund; a swapped asker heard on mainnet; an asker that says ok and fails taken for a yes; a key file others can read; SetAuthority as
another instruction; a write one byte off; a rehearsal talking to any host; a placeholder sent; a source with two ids patched; the fund's
fee read from another field; a face that cannot move; a face of another update authority read; `init` making a face at any address;
anyone moving the face; `set_face` moving any metadata account. 22 put in, 22 caught, none survived.

**Inside every rehearsal:** a deploy short of its SOL (refused, nothing started); one byte of Poleis's built binary moved (refused: not the
recorded binary); another rehearsal's seed (refused: its keys are not the sitting's); a record of another chain (refused); the deploy
killed by SIGKILL mid-upload (resumed from the chain); the founding taken out of the record (read back, the same transaction); the
handoff asked in genesis (refused); the handoff asked once ratified and before the fund (refused). The checks add the market's own three
refusals.

**By hand:** rehearsals ended by TERM 92 and 169 seconds in left no validator, no sitting folder and their ports free.

## The brief's attacks, answered

- **Can a seat run a mainnet step without his finger?** Not through the press: on mainnet the asker is the house's committed binary and
  no other, a swapped one or one the environment names is refused, and no key file is read before it says ok. **But a key file is a
  file.** A process running as his user can read `keys/deployer.json` without the press; the folder's mode keeps other users out, not his
  own. The finger guards the press's road, nothing more. See "What is not done", 4.
- **Can a stranger found root 0 between the deploy and the press's founding?** In every run the founding landed two slots after Poleis's
  deploy: one slot stood open. A stranger's root is caught by the read-back (its authority is not the deployer), nothing is handed over,
  and the deployer, still holding the code, closes the program and sits again under fresh ids. No stranger can open the $WARM mine: its
  `init` needs the program's upgrade authority's signature.
- **Does a key outlive the sitting on disk without the press saying so?** The four key files stay until he deletes them; the press says
  which and when. The build tool's keypairs are removed unread. A rehearsal's keys are never on a disk.
- **Does the press ever send a Keystone's transaction?** No. It writes the sets and holds no Keystone's key.
- **Does anything stay running?** No validator, test or sitting folder after any run, ended well, by its own refusal or by TERM.
- **Does a line name a classical scheme?** Two, marked: the deployer and the program accounts sign Solana's own transactions.

## Beyond loopback

As the market's rehearsal found: the test validator 4.2.0 binds its RPC, websocket and faucet on every interface, and a UDP socket at
`*:8000` (`*:8001` for a second one beside it). The script names exactly those, and under mainnet's rules the connection its two reads
leave to that RPC's port 443, and stops before anything runs if anything else is open. The application firewall (on) saw 0 inbound flows
to the validator in every run.

## What is not done

Each is named once here. 1 to 3 stand between this landing and a sitting and are this seat's to build.

1. **His Keystones cannot vote yet.** A chair votes by signing an envelope that Poleis reads through the signature precompile; the
   Keystone page signs transactions only, and reads no set of a local chain. The press writes each vote beside its set (the envelope,
   and the transaction with the signature's place blank); the stand-ins sign both through the same bytes. Until the page learns it, no
   act can pass with his Keystones, so no handoff either. A change to the page: its own lane, with his own rehearsal on a local chain
   after it (only stand-in keys have ever signed).
2. **Nothing prepares the first seat.** Ratification needs one seat of the genesis sale claimed (a bid committed, revealed and claimed,
   then `ratify_genesis` by anyone). The door pours deposits into his vault, not into Poleis's sale; the rehearsal's floor is stand-ins.
   By the road's Stage 1 the first seat is his own: three transactions a Keystone signs, which the same lane prepares.
3. **The seal's reach is loose.** `CODEX_UPGRADE` upgrades Poleis alone. The market's and the $WARM program's code is reached only by
   `CODEX_UPGRADE_AUTHORITY`, whose payload names the new holder and not the program: whoever carries a passed act out names the program
   data, so an act voted for one of the three programs could be carried out on another (a burn, on the wrong one, for good). Owed in
   Poleis before a handoff: the payload names its program.
   **Closed Nov 23 2026** by THE SEAL NAMES ITS PROGRAM (`poleis/THE-SEAL-NAMES-ITS-PROGRAM-LANDING.md`): the payload is
   `[new:32][program:32]`, the act's hash covers its program, and Poleis refuses any other program's data. And since THE SEAL UPGRADES
   EVERY PROGRAM (`poleis/THE-SEAL-UPGRADES-EVERY-PROGRAM-LANDING.md`) the market's and the $WARM program's code is upgraded by a vote
   that names it (`CODEX_UPGRADE_PROGRAM`), with no key between.
4. **The deployer's key is a file until the handoff:** an evening with his own first seat, a sale's length otherwise, when a Keystone
   should hold the code in between (signed by the page's own road: with 1).
5. The founding lands two slots after the deploy, not one, and `press` has no step that closes a program a stranger's founding spoiled
   (the Solana CLI by hand).
6. **His, before a sitting:** the params' values (every one a PLACEHOLDER until he names it; the press refuses to send with one left);
   a ship (production is main of Nov 19); the SOL on the deployer and a little on Keystones 1 and 2.
7. **Open before real money, his calls:** no outside review has ever run on these programs; the chairs' votes sign with the classical
   scheme of Solana's precompile (a post-quantum one is owed, and a sealed upgrade can bring it), and a vote does not name its program
   (the Nov 20 audit's P3); $POOL's lock is not built; $WARM's trading pool is not in the launch; no miner is named, so nothing mines
   until the door's lane; and the launch doc's section 8f lists 14 pieces the system's map lacked and the lines of his that disagree.
8. **Closed Nov 24 2026** by THE PRESS IS READY FOR MAINNET (`THE-PRESS-IS-READY-FOR-MAINNET-LANDING.md`), three owed items: `floor.issuer`
   held to `D/keys/issuer.address` (THE SWARM NAMES ITS MINER's follow-up); `market.crediter` made by `press crediter` (a PLACEHOLDER no step
   made); and the program data extended before a sealed upgrade to a longer build (THE SEAL UPGRADES EVERY PROGRAM's "What is not done" 1),
   carried by `press crank --upgrade H`.

**Not run by this seat:** the lander's own gate (it runs at the hand); anything against mainnet beyond its rent, SOL's price and the
validator's two reads; the house's real Touch ID asker (stand-in askers played it in the tests; its contract, one line "ok" and exit 0,
was read in `tools/embertide-mesh/touchid/touchid.swift`); his Keystones; the every-feature rehearsal at the code's last commit (bd7f5e307b was rehearsed under mainnet's rules only).
