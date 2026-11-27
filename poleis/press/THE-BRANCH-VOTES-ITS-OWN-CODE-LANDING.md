# THE BRANCH VOTES ITS OWN CODE: an update of the house's code is unit 1's fast vote, its agents', and no seat holder votes code (Nov 23 2026, lane/the-branch-votes-its-own-code, claude-78d4ba56)

## His words, UTC (this session's transcript)

- **19:49:07** (to seat claude-41008c71) "it feels to me like futarchy is long-term directional while instead voting is how you execute short term?"; that
  seat answered with the figure "Vote the why, bet the where, vote the how": the why (the law, who is let in, what the Fund pays for) by the slow vote of
  both chambers, the twelve thrones AND every user; the where (her road for the month, a big spend, the budget) by the market; the how (a small payment,
  **one branch's own code**, the next step of the road) by "the vote: fast and small, each branch its own; the agents act inside the budget".
- **20:13:20** to this seat, that figure pasted: "what? do you not understand this". Today's update road (THE SWARM SHIPS THE UPDATE, 19:12; A MEMBER
  PROPOSES THE UPDATE, 19:41) had put code updates in the top row: every release a vote of root 0's two chambers, unit 1's agents AND every seat holder.
- **20:19:22** "Yes, fix it", to "Move code updates down to the agents' fast vote?".

## Cost first

| | before (main 4f033a52f9) | after (this lane) |
|---|---|---|
| who votes a release of the house's code | root 0's two chambers: unit 1's chair (its agents, by mandate) AND root 0's floor, every seat holder; SIGNAL under CONSENT at root 0 | unit 1's own vote, the branch that runs the house: its agents, two of three; root 0 holds no vote of it, no seat holder is asked |
| who proposes | any seat owner of root 0 (`press vote --release`, a set signed on the Keystone page) | one of unit 1's agents, by its own `propose` (it pays the proposal's rent, which comes home at close); `press release --commit C [--agent A]` writes the agents' work order: every agent's envelope, and A's proposal |
| who lands the votes and the seal | the rehearsal's own checks (no road in the press) | `press crank`: a ballot `{release: <the proposal's hash>, unit: 1, voter, signature}` is landed in unit 1 and unit 1 sealed when the chain lets it, the deployer paying (anyone may send) |
| the ballots' road | | main's AN AGENT VOTES ITS CHAIR (landed while this lane ran): each agent's file in the agents' folder `press crank --signatures` takes; a release's ballot is held as an act's before anything is sent (`crank::release_held`: the voter's own signature over the release's envelope in unit 1 and its choice; a replayed, another choice's or a forged one stops the crank, named), and `press ballots` says each release row held or refused by unit 1's chairs |
| the road's polis | 0 | 1 (`seal::POLIS`); the reader, the devices' check and the lander read it off the road, unchanged |
| per release on the chain | 10 transactions in the launch rehearsal (the proposal, unit 1's convene, the agents' batch, its seal, its mandate, four seats' votes, root 0's seal) | 3: the agent's proposal, the agents' batch, unit 1's seal |
| `press vote --release` | a seat owner's release, proposal and PRO | taken out: no seat holder proposes or votes code |
| JavaScript, by its gate | | +2 −2, net 0 (two of the lander's lines say who proposes now) |
| tests (the new Mac, nice 19) | | on the tip merged with main 66d86fef89: the press 97 of 97 in 19 runs (`tests/release.rs`: the name the reader's for every polis of its vector, the agents' work order byte for byte, the crank's release ballots unit 1 alone and held); the council test compiles; the JavaScript, the law's table and any-room gates 51 of 51 |

Also in this lane: the rehearsal's stand-ins name no `seal.proposers` any more (THE SWARM SHIPS THE UPDATE's owed 6; the lane beside that line has
landed), and the certificate's road sentence says unit 1 and nobody who may propose, where it said "root 0 ... the proposers his key named".

## What it keeps

- The AND of the branch: a release still passes only by its chamber's majority, never a market, never the SUM.
- The why stays where his figure puts it: the law (Poleis's own bytes, CODEX_UPGRADE, and the programs' authority) is still sealed by both chambers of
  root 0; THE SEAL UPGRADES EVERY PROGRAM is untouched.
- A commit not on main is still refused; the lander still ships the swarm's newest seal by itself; SAY-ONLY UNTIL A ROAD IS NAMED: live, nothing moves.

## The proof

- **The launch rehearsal on a local chain, twice** (`poleis/ceremony/the-launch-press-rehearsal.sh` on 127.0.0.1:8601, every feature the validator
  knows, nothing read beyond this Mac; 20:29 and 21:24 UTC). Both runs, read off the chain by the checks: the press's road named unit 1 and nobody who
  may propose; agent 1 proposed the release of `1ec7da52de04` and agent 2 that of `7ece4d77289f`, each by its own transaction in unit 1 (slots 1038 and
  1040 in the first run); two agents' envelopes of each went to `press crank`, which said **"RELEASE 20413656b160 (the house's code at 1ec7da52de04):
  unit 1 PASSED, no seat holder asked: sealed RATUM"** and the same of `f348854d239f`; the checks then held unit 1 RATUM with two of its three agents,
  and root 0 holding **no vote** of either release.
- **A third run, on the tip merged with main 66d86fef89** (21:58 to 22:38 UTC, 2,412 s; the agents' acts now through main's `press chair`, their
  ballots in the agents' folder): each agent's release envelopes were a file of its own in that folder (`<address>.releases.json`), `press crank
  --signatures` took the folder whole, held every ballot, and said **"RELEASE 20413656b160 (the house's code at 1ec7da52de04): unit 1 PASSED, no
  seat holder asked: sealed RATUM: the lander ships it by itself"** and the same of `f348854d239f`; the handoff followed. It ended where the first two
  did, in the market's trades (`the validator's clock stood still before slot 1926`, council.rs:6238), the load average 136 to 297 on 18 cores.
- **What those runs did not reach**: all three ended later, in the market's part after the handoff, on "the validator's clock stood still" (the test's own
  guard: more than a second a slot), the Mac's load average at 316, then 640, then up to 297 while other seats' screen checks and rehearsals ran beside it. So the script's own read of
  the two unit-1 seals by `poleis-seal` and the lane's lander shipping by them did not run here. Both are the code that read and shipped root 0's seals at
  19:09 UTC, the polis now read off the road; the reader's law is held for any polis by its vector (`poleis/seal/tests/law.rs`, polis 0 and 5). A run on a
  quieter Mac is owed (below).

## What is not done

1. **The agents' own rule for a release**: main's `press chair` (AN AGENT VOTES ITS CHAIR) is the agent's own step for the launch's acts, and its
   rule casts nothing on a release (chair.rs says so). What an agent checks before its PRO on a release (the commit on main, through the lander's
   gate) and its own `propose` are owed; in the rehearsal the checks sign the release envelopes and propose as the stand-in agents, and put each
   agent's envelopes in a file of its own in the agents' folder, the road its own step would take.
2. **The agents' SOL to propose**: the proposer pays the proposal's rent (10,008,480 lamports, about 0.01 SOL, on A RELEASE IS A VOTE's practice chain; back at close). The agents hold none at the
   launch; who gives each a little is the launch's to say.
3. **The budget the agents act inside**: his figure's how acts "inside the budget" the market sets; a code budget (how much may change before it goes
   back up) is not built.
4. The ready-to-sign set of A MEMBER PROPOSES THE UPDATE and its certificate stay as history; the step is gone.
5. **The whole rehearsal to its end on a Mac with room**: the reader's SEALED of a unit-1 release and the lander's ship of it, the two steps the load cut
   off three times; its certificate then beside this receipt. Its road is A SEAT'S HEAVY JOB GOES WHERE THE ROOM IS (claude-e235fc26's lane, which
   wires this rehearsal to a job slot), not a fourth run beside thirty seats.

**Not run by this seat:** the lander's own gate (at the hand); anything against devnet or mainnet.
