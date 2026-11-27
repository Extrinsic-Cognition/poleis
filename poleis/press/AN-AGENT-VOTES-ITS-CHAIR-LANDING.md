# AN AGENT VOTES ITS CHAIR: three new agents, their keys made at the sitting, each casting its chair's vote in the launch by a rule; the crank holds every ballot before it sends (Nov 23 2026, lane/an-agent-votes-its-chair, claude-0733cf6f)

## Whose words

- HIS, **Nov 23 2026 18:17:41 UTC**, a press in seat claude-f32e080d's dialog, read in that session's transcript. The question (the
  seat's): "Mainnet launch: which three agents hold Nefertiti's chairs? Two of the three must agree for her to vote; no key of yours sits
  there." The option he pressed: **"Three new agents"**. The button's longer words were the seat's: "Their keys are made at the sitting;
  they vote only on the acts the launch wrote". The other option was "Later". It reached this lane 55 minutes after its first code, while
  its second rehearsal ran: the lane then built the sitting's own step that makes the keys, and ran the rehearsal again.
- The lane itself was cut that day by the session that holds the launch, over what THE SWARM IS ITS OWN MULTISIG left owed ("What is not
  done", 2: "How an agent decides its chair's vote and signs its envelope is its own runtime's lane"). No lane had built it, so after a
  mainnet founding no act could have passed: only stand-in keys made in a test's memory had ever signed for a chair.
- HIS, the rulings it stands under (UTC): Nov 22 00:05:01, "focus on turning the swarm into its own multisig... the fucking perfect fractal
  bicameralism has to be applied dude..."; Nov 22 09:50:51, "you gotta fucking stop treating the keystones as what make the decisions
  porcoddio. the fucking entire tree perfect bicameralism gvs posiverse neghiverse molecularity"; Oct 30 00:19:51, "discuss... what keys are
  these? i want agents able to sign shit dude"; Aug 17 11:27, "there's gotta be a link between minting a seat for voting on solana and that
  same public key or mapped equivalent ed25519/x25519 used in the swarmclock mesh" (the last two as
  `poleis/ceremony/AN-AGENT-PULLS-ITS-OWN-ALLOWANCE-LANDING.md` records them).
- SEAT, **the rule**: the brief's proposal, laid as a figure on the Canvas's Claude page before any code (now widget `7d645caf`): an
  agent casts PRO only on a set that is the act the press derives from the sitting's own params, and nothing on any other. His press
  chose the option whose longer words say the same of the launch ("they vote only on the acts the launch wrote"); those words were a
  seat's, so what the rule does beyond them is said below, his to strike.
- SEAT, **where a new agent's key rests** (this seat's reading of "made at the sitting", his to strike): in the Nest's keys folder, as a
  new Nest seat's own key, never in the sitting's keys folder beside the deployer's. A key his finger opens beside the deployer's would
  be a key of his in a chair, which his Nov 22 rulings took out. The seats' names are the sitting's to say: this lane names none, and
  its only edit near `poleis/press/params.mainnet.json` is none (the file is main's, byte for byte).

## Cost first

| | before (main 0dc9cc2b38, where the lane was cut) | after (this lane, main merged in to 44ecb986f1) |
|---|---|---|
| who signs for a chair of unit 1 | a stand-in key made in a test's memory from the rehearsal's seed (`standin("agent i")`, council.rs) | a new agent's own Nest key, made at the sitting by `press chairs` (`{seat, key, private_pem, created}`, mode 600, in the Nest's keys folder); the same 32 bytes are its Solana address, the one in its chair. The step also reads a key the Nest's keygen made, and a golem's `{alg, pkcs8}` |
| making the three keys (`press chairs`) | none: `posiverse.agents` were PLACEHOLDERs with no road to a key | one run: 32 bytes of the system's randomness a key, three files written once, read back; no chain, no finger (nothing of his is read); run again it writes nothing |
| what decides the vote | the test's loop signed whatever envelope a set named | a rule in Rust (`chair::decide`): PRO only on a set that is the act the agent derived itself from the params, byte for byte; else nothing |
| what is signed | the bytes the set handed over | the 55 bytes the agent derived (`poleis:vote:v1`, the derived hash, unit 1, PRO); a set is only compared |
| one agent's step (`press chair`) | none | 37 ms and 38 ms by the clock in the rehearsal of record (five acts); 0.03 to 0.05 s, 0.01 s of CPU, in three runs by hand against the first rehearsal's validator (four acts, this Mac at load 20 to 45); two reads of the chain (its genesis hash, unit 1's registry); the sets read (four of 4,906 to 5,416 bytes in that run, one more since the sixth act); one signature and one check of it an act; one file written (2,132 bytes for four acts). No transaction, no lamport: an agent holds none |
| at rest | | nothing: no daemon, no timer, no listener; the step runs when an agent is asked to vote |
| the crank and a bad ballot | sent in a batch of four; on the chain's refusal, one by one | every ballot held to its voter and its act before the payer's key is loaded: one signature check a ballot (at most 18 a launch: three chairs, six acts); a ballot not held stops the crank, named, nothing sent |
| what `press crank --signatures` takes | one file a test wrote | one file, or the folder of the agents' own files |
| the tests (this Mac, at the tip handed, main 4f033a52f9 merged in) | | the press 94 (the chair's 9, the rest main's own); Poleis 227 and 5 ignored, the client 25, the seal 6, the seat 10: main's own; no warning (at 4f629e9ca4, the rehearsal's: the press 91, Poleis 226, the client 24) |
| faults put in on purpose | | 41 in the sources, one at a time, each caught by a test of `tests/chair.rs` (188 s for the 41 on the tip handed, 126 s and 57 s on the merges before); 4 more in every rehearsal, and the crank on the agents' ballots alone |
| a rehearsal | | every feature, 1,756 s at nice 19 on this Mac (the one of record, at 4f629e9ca4, main merged in); before it, at 7e36d2ca13, 1,658 s, and at 1cbb62a6e0, with the agents' keys by the Nest's keygen, 1,849 s; all three exit 0. The first sitting needs 4.20613628 SOL at the local chain's rent; the founding 887 bytes, 39,625 CU; the crank 28 transactions for the five acts (18 on the agents' ballots alone, 10 once the floor had voted), then 0 |
| JavaScript | | none added (`git diff main...HEAD --numstat` over .mjs, .js, .cjs, .ts, .html: empty) |
| classical signature lines | | none counted added (the gate green on the tip handed: 1,146 counted, the baseline 1,146); six excused lines this lane adds name the curve, each `classical:` with its reason (Solana's precompile reads one curve; the Nest's golem key names it) |

No source of Poleis, the market, the $WARM program or the stake pool moved (Poleis's test file `council.rs` did: it plays the rehearsal's
stand-ins). Nothing is deployed, nothing was sent to any cluster, and no key of his was made, read or asked for: the only keys made are
four Nest keys of rehearsal seats (three by `press chairs`, one by the Nest's own keygen), in each rehearsal's temporary folder, removed
with it.
The lane's own diff from its merge base (main 44ecb986f1), without this receipt and the certificate: 9 files, +1,147 −43.

## The design, as built

**`press chair --dir D --params P --rpc URL --key F [--acts A] [--out B]`** is the agent's own step, run by the agent, never in his
sitting (`poleis/press/src/chair.rs`). It reads no key of the sitting (no seed, no keys folder, no finger) and sends nothing.

0. **`press chairs --seats a,b,c [--nest N] [--dir D]`**, before the founding: THREE NEW AGENTS, THEIR KEYS MADE AT THE SITTING. Each
   named seat's own Nest key is made in the Nest's keys folder (N, else NEST_KEYS, else ~/.embertide/nest) in the Nest's own shape, mode
   600, and read back; a key that stands is said and never written over; a seat's name that is no file's name, a folder that is a
   sitting's keys folder, and a folder inside a repository are refused. It prints the addresses in their chairs' order: the value of
   the params' `posiverse.agents`. It reads no key of the sitting and asks no finger.
1. **Its key rests where the Nest keeps a seat's:** the agent's own Nest key file. The step refuses a file others can read (mode 600, its user's),
   a file inside the sitting's keys folder, a Solana key file (the shape the sitting's own keys are kept in), the seat's post-quantum twin,
   a record whose public half is not its private key's, and a key whose address is the sitting's deployer or a program id (THE SWARM SHIPS
   THE UPDATE took the seal's named proposers out of the params, so none is read). Never his login keychain, never a key of his.
2. **It reads** the sets `press acts` wrote (D/acts), the sitting's params, and the chain: the chain's genesis hash, and unit 1's registry,
   where its key must sit in the chair the params give it.
3. **The rule.** It derives the launch's acts again from the params by the press's own derivation (`acts::acts`), and for each set: the
   cluster and the genesis hash are this chain's; the set is signable; its act's number, class, payload, payload hash, manifest, nonce,
   impacted set and proposal hash are the derived act's; exactly one envelope is asked of this chair, as an agent, in unit 1, PRO, and its
   bytes are the derived envelope. Then PRO. On anything else it casts nothing and says why: no CONTRA, no ABSTAIN.
4. **Its ballots** go into one file of its own (D/ballots/<its address>.json: who voted, from which chair and seat, the chain, the params'
   sha256, the rule, a row `{act, hash, unit, chair, voter, choice, signature}` an act, and what it cast nothing on). They are public:
   64 bytes a vote. The step exits 1 when it cast nothing on a set it read, with the ballots it did cast still written.

**The hold.** `press crank --signatures` reads one file or the folder of the agents' files (`chair::gather`: a file that names its voter
carries that voter's rows alone), then holds every ballot: its signature is its voter's own over the envelope of the act, the unit and the
choice it names (`chair::held`). One that is not stops the crank before its payer's key is loaded. `press ballots --signatures` does the
same read and hold, adds the chair each voter sits in on the chain, and sends nothing.

## What the rehearsal proved

One of record at 4f629e9ca4, main merged in with THE SWARM SHIPS THE UPDATE and THE SWARM NAMES ITS MINER (every feature the validator
knows, nothing read beyond this machine, 1,756 s, exit 0); the certificate
`poleis/ceremony/launch-press-rehearsal-an-agent-votes-its-chair-2026-11-23.json` (`chairs`, and `checks.chairs`). Read off the chain and
the steps' own words:

- **The keys, made at the sitting.** `press chairs` made three seats' own Nest keys in a folder of the sitting's alone (mode 700, each
  file 600), before the founding and with no key of the press's read; asked again it made none and said the same addresses. The Nest's
  own code (nest-author's `loadKeyFrom`, agent-key.mjs's `solanaKeyOf`) read each file and gave the address the press said. A fourth
  seat, with no chair, was made by the Nest's own keygen.
- **The founding** named those three addresses in unit 1's chairs 0 to 2 (the certificate's `posiverse.chairs`), in one transaction of
  887 bytes, 39,625 CU.
- **The votes.** Before any act was proposed, agent 1 and agent 2 each ran `press chair` with its own key and no seed of the press's
  (37 ms, 38 ms): "5 PRO signed, nothing cast on 0 sets", on acts 1 to 4 and 6, the mine's miner. `press ballots`: "10 held, 0
  refused". The checks signed none of those
  envelopes (`envelopesSignedByTheseChecks: 0`); agent 3 cast nothing through the road.
- **The agents alone pass nothing.** `press crank` on the two agents' ballots, before any seat of the floor had voted (18
  transactions): every act stood unit 1 PASSED by two chairs, its chair heard in root 0, 0 seats of the floor, root 0 still voting,
  nothing executed. Then four of the six seat owners voted by their own transactions and the crank (10 transactions) executed all five
  acts: "unit 1 PASSED, root 0 PASSED (4 seats of its floor voted by their owners): executed"; the sixth named the mine's miner. Run
  again: 0, nothing due. Act 5, the pool's birth, said it was not run (no stake pool program on this chain).
- **And as today, at a window's end:** fault (a), the three agents with two seats of six: unit 1 PASSED, root 0 DEADLOCK, nothing paid;
  fault (b), all six seats with one agent of three: unit 1 REJECTED, root 0 DEADLOCK. Those two are the checks' own acts, which the
  rule casts nothing on: the checks signed them with the agents' key files ("What is not done", 1).
- **The road's four faults,** each refused (the table below; `press ballots` on the faulty folder: "3 held, 7 refused"), and the faulty
  ballots handed to `press crank`: "7 of the 10 ballots handed over are not held, and nothing is sent".
- The rest as on main: the handoff refused twice and then done; the seal's road read both releases SEALED (each proposed by a seat
  owner, any member) and a commit never proposed NOT SEALED, and the scratch ship took the rider's release by its commit and then the
  swarm's newest seal; the market's chain; the mine named by the sixth act, then a deposit and a spend through the door's own code.
  162 transactions of the checks, 15 refusals, each asserted; the application firewall saw 0 inbound flows to the validator.

After the certificate's commit the lane changed two comments and nothing else (the script's header and `council.rs`'s note on which
acts the checks still sign for the agents: "the mine's" became "the face's move", the miner being the sixth act now).

Three rehearsals before it, none kept as a certificate (this one replaced them): at 1cbb62a6e0, before his press reached the lane, the
same road with the three agents' keys made by the Nest's own keygen (1,849 s, exit 0), so the agent's step has voted with keys of both
makers; one stopped by this seat at his press, to make the keys the sitting's way; at 7e36d2ca13 with the keys by `press chairs`, the
first hand's (1,658 s, exit 0, four acts); and one on the first merge, stopped at 684 s once the miner lane landed.

## Merged three times before the hand

- **THE SWARM SHIPS THE UPDATE** (main 45a630daf1d4, landed 19:12:49 UTC): the lander refused this lane four seconds later, at 19:12:53,
  "merge-tree conflicts with main 45a630daf1d4 in poleis/ceremony/the-launch-press-rehearsal.sh". The conflict was the script's header: that
  lane rewrote THE LAUNCH NAMES THE SEAL'S ROAD's paragraph (nobody names who proposes a release) and this lane's paragraph follows it; both
  kept. It also took `seal.proposers` out of both params files and refuses a file that names it, so `press chair`'s check that a chair's
  key is not a named proposer could never fire: taken out.
- **THE SWARM NAMES ITS MINER** (main f35a436874a6, landed 19:20:44 UTC, ahead of this lane in the line): a sixth act, `set_miner` by
  the polis's hand. Merged onto this lane on a scratch worktree before it landed, it turned `tests/chair.rs` red (its fixture named no
  `warm.miner`, and it counted five sets) and showed two lines of `council.rs` numbering acts by their place, which git would have merged
  silently; both were fixed on this lane first, and the real merge took the resolutions rehearsed there. The agents' rule covers the sixth
  act with no change: it is the press's own derivation. A rehearsal started on the first merge was stopped once the miner lane landed, at
  684 s (its trap ended its validator and removed its folder).
- **Main again, with no conflict** (main 4f033a52f9, after the certificate): THE SEAL UPGRADES EVERY PROGRAM, A MEMBER PROPOSES THE
  UPDATE, SPENT $WARM GOES BACK TO THE MINE and three lanes outside Poleis. None of them touches the press's acts, its crank, the launch
  rehearsal's script or the params; they move Poleis's proposal and upgrade code, the floor's step, `main.rs`, `council.rs` and the
  client, in lines this lane does not. On that merge, the tip handed: the suites above, 41 of 41 faults, the gates green. The rehearsal
  was not run again on it: the certificate is of 4f629e9ca4.

## The faults

**In the sources, one at a time, each caught by a test of `tests/chair.rs`** (this Mac, 57 s for the 41; the script that put them in
is not in the tree): the payload, its sha256, the class, the manifest, the nonce, the impacted set and the hash each no longer
compared; a set of another chain, of another cluster, and one that is not signable accepted; the envelope asked no longer held to the
act's own; the handed envelope signed instead of the derived one; a CONTRA, a unit 0 or a Keystone's ask accepted; a chair named twice in
a set accepted; a second set of one act signed again; every ballot held whatever its signature; a ballot held to PRO whatever choice it
names; the crank going on with unheld ballots; another chair's row accepted in an agent's own file; a key in a chair the params do not
give accepted; a key in no chair accepted by the params' word; a key of the sitting accepted; a record whose public half is another
key's, a Solana key file, a PEM that is no private key, a key of another curve and a key of another length read; a key file others can
read and a key in the sitting's keys folder read; every signature verifying; the crank no longer holding the ballots, holding them
after it loaded its key, or reading them without the gather; and, of the key's making: a key written over one that stands, a key standing
under another seat's name taken, a key made in the sitting's keys folder or inside a repository, a seat's name that is a path taken, a
key file others can read, a key written in another shape than the Nest's, a refused folder left behind. The first pass found two that
no test caught (a key of another curve; the crank's hold taken out): both are tests now. The Nest key's reader and its writer are each held to RFC 8410's published vector, not to one another.

**Inside every rehearsal,** besides the launch press's own: the four below, and the crank run on the agents' ballots alone.

| the fault | how it is put in | what refuses it |
|---|---|---|
| a vote on a payload the params do not give | agent 3's own params say a fund fee of 251 bps; the sitting's sets were written at 250 | `press chair` casts nothing on act 1, says the payload differs, signs the three acts the fee does not touch, exits 1 |
| a key that is not the chair's | a fourth Nest key, of a seat in no chair | `press chair` signs nothing: "sits in no chair of unit 1 on this chain" |
| a replayed envelope | agent 1's signatures of act 1 and act 2, each offered as the other's ballot | `press ballots` names both; `press crank` sends nothing |
| another chair's envelope | agent 2's ballots offered under agent 3's name | the same |

## The attacks, answered

- **Can whoever writes the sets steer an agent's vote?** No. The agent signs only the envelope it derived from the params; a set can only
  make it cast nothing.
- **Can a ballot be moved to another act, another unit, another choice, another voter?** No: the envelope binds the proposal's hash, the
  unit and the choice, the signature binds the voter, the crank holds each before it sends, and Poleis's own check (Solana's precompile)
  refuses it again on the chain.
- **Across chains?** The envelope names no chain and no program (Poleis's own format, unchanged here). An act whose payload names no
  program id (the fund's constitution) hashes the same under the same params on any chain, so an agent's PRO on it in a rehearsal is valid
  on mainnet. Under this rule that lands only the vote the agent would cast there anyway (same params, same payload), which is why the
  rule is a function of the params and the key alone. A rule that could say CONTRA, or differ by chain, must first put the chain in the
  envelope: a change to Poleis.
- **A stolen agent key?** It votes that chair until the chair is reseated, which needs both chambers. One key of three lost or stolen
  changes no outcome; two do (the swarm's liveness, named in THE SWARM IS ITS OWN MULTISIG).
- **Who holds the three keys?** On mainnet, files in the Nest's keys folder of the Mac the sitting ran on, mode 600: whoever reads that
  user's files holds all three chairs, and so the agents' half of every act; the floor's half is still the public's. It is the same
  footing every seat's key has today. A key that leaves that Mac for the agent that votes with it is "An agent on another machine", below.
- **Who decides what the agents vote?** Whoever sets the params the agent reads. On mainnet those are his, in the house's git: the rule
  makes the three chairs confirm that what is being passed is what the params say, no more. It is a check against a swapped payload, not a
  second opinion.

## What is not done

1. **Every act that is not the launch's.** A release of main (A RELEASE IS A VOTE, THE SWARM SHIPS THE UPDATE), a budget, the face's
   move: this rule casts nothing on them, so after the launch no such act passes unit 1 until a rule for them is built. (The mine's
   miner is the launch's sixth act since THE SWARM NAMES ITS MINER, so the agents vote it.) His press says the three vote "only on the
   acts the launch wrote" (a seat's words on the button), so this is what was chosen, and it stops every later act: his to rule next.
   In the rehearsal the checks still sign those for the agents (the faults, the releases, the market's budgets, the face's move), holding
   the same key files: a test's hand, not an agent's.
2. **The agents never say CONTRA.** PRO or nothing.
3. **The three seats' names, and the making on mainnet.** `press chairs --seats a,b,c` at the sitting, then its line into
   `posiverse.agents`: both his hand's. On mainnet those values are still PLACEHOLDERs, and this lane filled none.
4. **Where the new keys rest is this seat's reading** of "made at the sitting": the Nest's keys folder, never beside the deployer's. His to
   strike. A key made by `press chairs` has no post-quantum twin (the Nest's keygen makes one beside its own keys; `pq-keys.mjs` makes
   one for an older key), and no Nest entry enrolls the new seat: it is a key that votes its chair, nothing more yet.
5. **An agent on another machine.** The step runs in the sitting's own build of the press (its ids compiled in) beside the sitting's
   folder. An agent elsewhere needs the folder's public part (record.json, acts/) and that build, or builds it from the house's commit
   with the sitting's ids; not walked. Its ballots travel as a file; carrying them on the Nest, signed by the seat, is not built.
6. **The receipts that say 54 bytes.** The vote envelope is 55 bytes (`poleis_treasury::util::ENVELOPE_LEN`: 14, 32, 8, 1). The client's
   comment is corrected here; THE SWARM IS ITS OWN MULTISIG's receipt still says 54, left as written (the sets themselves say the
   length they measure).
7. **A rehearsal under mainnet's rules** (the feature set and Metaplex's program read off mainnet) was not run for this lane: nothing
   here depends on a feature.
8. **One line of node in the rehearsal's script** calls the Nest's own loader to read the keys the press made. No JavaScript file moved.

**Not run by this seat:** the lander's own gate (it runs at the hand); anything against any cluster but the rehearsal's own validator.
