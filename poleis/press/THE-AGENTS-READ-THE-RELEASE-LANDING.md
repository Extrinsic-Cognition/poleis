# THE AGENTS READ THE RELEASE: each of Nefertiti's three agents reads a release of the house's code with its own mind before it votes it in unit 1 (Nov 24 2026, lane/the-agents-read-the-release, claude-ba1ae67d)

## Whose words

- HIS, **Nov 23 2026 17:52:35 UTC**: "hypertrace. you keep getting it wrong, the swarm votes on updates, you dont have it linear with me and
  keystones idk... am i wrong?" (CLAUDE.md, The swarm ships the update).
- HIS, **Nov 23 20:19:22 UTC**, "Yes, fix it", to "Move code updates down to the agents' fast vote?" (THE BRANCH VOTES ITS OWN CODE, its
  receipt beside this one).
- HIS, **Nov 23 21:43:27 UTC**, "Yes, build it", to "Build the rite that frees Nefertiti's three agents?" (THE RITE FREES THE AGENTS, its
  receipt beside this one).
- SEAT, the brief of the session that started this one: "Build the road by which each of Nefertiti's three agents reads a release of the
  house's code with its own mind before it votes it in unit 1, and rehearse it": the piece both receipts above named owed (THE RITE FREES
  THE AGENTS, not done 4; THE BRANCH VOTES ITS OWN CODE, not done 1). Not his words.
- SEAT, this one's, his to strike: what a mind is shown of a release (below); that the hands put nothing to a mind for a commit main does
  not hold, a hash that is not its commit's, or a vote that can no longer land; the agents' mind in the rehearsal's job-slot kit; the four
  commits of the run of record.

## Cost first

| | before (main 537951ad38 with lane/the-rite-frees-the-agents at 1f52652462, where this lane was cut) | after (this lane, main 15faec8416 merged in) |
|---|---|---|
| who decides an agent's vote on a release | the rehearsal's checks: they signed the PRO envelope of each release for agents 1 and 2, no mind asked; agent 3 voted no release | each agent's own mind, all three: PRO, CONTRA or ABSTAIN with its reason, by `press chair --releases`; the checks sign no release envelope |
| what an agent reads of a release | nothing | the commit (its hash, date and subject), its place on main, where main and production stand, its diff stat against its first parent (files, lines, the twelve that moved most), the lander's gate from its ledger, the chain's word on its proposal: one question of 2,148 to 2,375 characters in the run of record, beside who the agent is (587 to 737 characters, her charter whole among them) |
| the hands' reading of one release | | about ten git questions of 19 to 39 ms each at a load of 111 (main's own line is 6,160 commits long), one read of the lander's ledger an agent step (two files, 2.29 MB), two chain reads (the proposal, unit 1's session); nothing at rest |
| a vote of a release | no ask | one ask a release an agent: 15.2 to 16.7 s an agent for its two releases in the run of record (the hands' reading and the mind's answer, 7.6 to 8.3 s a release), its answers 166 to 260 characters |
| ballots a release on the chain | 2, both PRO, signed by the checks | 3, each its agent's own choice; still 3 transactions a release (the agent's proposal, the agents' batch, unit 1's seal) |
| faults held | the crank's hold (another release's replayed, another choice's, a forged one) | and the agent's step's: a release not on main, a commit no git holds, a hash not its commit's (swapped, commit moved, another nonce, another payload, root 0's), a set of another chain or not signable, an envelope in the set over another release, two or none of this chair, a vote that can no longer land, a second set of one release, a mind that gives no choice (five ways); four put in live by the rehearsal |
| tests (the new Mac, nice 19) | the press 111 | the press 121 (`tests/release_read.rs` 10, 1.7 to 10 s: each makes its own git house), 24.7 s in all at a load of 60; the council checks build with no warning |
| JavaScript, by its gate | | none added (31 of 31) |
| the rehearsal's road | | the job-slot kit names the agents' mind, so oldmac (its ollama serves no fable-fusion model) is left out; this Mac took slot 1 at 01:22:48 UTC after 67 s of waiting, at a load of 29.4 on 18 cores |
| the rehearsal | 3,794 s (THE RITE FREES THE AGENTS' of record) | 2,271 s for the run of record (its checks 2,142 s and 235 transactions); a first run stopped at 115 s at the deploy's cut, before any of this lane's steps (below) |

## What an agent's step is shown, and what it decides

`press chair --releases D/releases` is the agent's own step, beside the launch's acts (`poleis/press/src/release_read.rs`, `main.rs`
`releases_step`): its Nest key, freed first (its rite held as for the acts), each release set `press release` wrote read by the hands, which
decide nothing:

1. The set is of this chain (its cluster and genesis), signable, and names a commit and a nonce.
2. The step derives the release itself from that commit, as A RELEASE IS A VOTE names it (`seal::release`: SIGNAL under CONSENT, kind 0x52,
   its manifest the commit, its payload [1, 0x52, the commit], unit 1 alone impacted), and the set's hashes (the release's, the act's, its
   payload's) must be that one's, or nothing is put to the mind.
3. One envelope of this chair is asked, as an agent in unit 1, a vote over that hash; another chair's, two, none, or one over another
   release are never signed.
4. The house's own git, where the house is (nothing sent): the commit is on main, or nothing is put to the mind. Then what it is shown: the
   commit, its date, its subject (400 characters at most); whether it is a commit of main's own line or reached through a landing, and how
   many commits of main's line stand after it; where production (refs/heads/green) stands from it; its diff stat against its first parent
   (files, lines added and taken out, binary files, the twelve files that moved most).
5. The lander's gate, read from the lander's own ledger (`ledger.jsonl` and its one rotated `ledger.jsonl.1` in `~/.embertide/lander`, or
   `--lander`; read, never written): the newest landing that names the commit as its merge or as its lane's tip, its gate's tests, new
   reds, main's own reds, proofs and walks, the batch it rode in, a revert of it since; or that the ledger holds no landing of it, the
   ledger's span, a refusal of the commit, and what git says (a merge of a lane in the lander's own words, or a commit no merge names).
6. The chain: who proposed it (the agent's own chair, another agent's, a key in no chair), whether unit 1's vote of it is open and until
   which slot; a vote that can no longer land (unit 1 sealed it, this chair's vote stands, the proposal is closed) is put to no mind.

Then the mind its rite names, told who it is as for an act (`chair::system_of`: its rite's own words, its chair, her charter whole), chooses
PRO, CONTRA or ABSTAIN with a reason of its own (`chair::choice_of`: any other word, or no reason, is no vote and nothing is cast). The
envelope of that choice is signed over the hash the step derived, into the agent's own `<address>.releases.json` in the agents' folder, with
its rite and, on each row, what its mind was shown (`shown`). `press crank --signatures` and `press ballots` hold each row as THE BRANCH
VOTES ITS OWN CODE holds a release's ballot (`crank::release_held`) and THE RITE its voter (`chair::unfreed_rows`), unchanged.

## What it keeps

- Who proposes a release, and with what SOL: one agent's own `propose`, its rent its own, as THE BRANCH VOTES ITS OWN CODE left it (in the
  rehearsal the checks still sign each proposal as its agent).
- The release's name is the seal's reader's (`seal::release`, held to `poleis/seal/tests/vector.json` by `tests/release.rs`).
- The crank's hold, and the crank: unchanged (it lands the three agents' ballots in one batch now, where two were handed).
- SAY-ONLY UNTIL A ROAD IS NAMED: no live road, so nothing of this moves production. No key of his made or read; nothing sent to any
  cluster; the mind on loopback; no JavaScript added; no Canvas tab.

## The proof

- **The press's suite**, on the tip merged with main 15faec8416: 121 of 121 (24.7 s at nice 19, a load of 60). `tests/release_read.rs`, 10 of
  10, a canned mind, a git house each test makes (a lane landed by a merge in the lander's words, a commit made on main after it, a lane that
  never landed, production at the first commit) and a ledger in the lander's shape: the road whole (what the mind is shown, its choice
  signed over the release's hash, its file with its rite held by `gather`, `release_ballots`, `release_held` and `unfreed_rows`); each
  choice its own envelope; a release not on main, and a commit no git holds (nothing asked of the mind); a hash not its commit's, seven ways;
  another chair's or a replayed envelope in the set (never signed), at the crank's hold (refused) and in an agent's own file (refused
  whole); a mind that gives no choice, five ways (nothing cast); the gate read from the ledger (a landing, a lane's tip, a batch, a revert, a
  refusal) or said unread (the ledger's span, git's word); the commit read from git (a landing, a lane's tip, production under it, at it,
  past it); a vote that can no longer land, a second set of one release, another nonce.
- **The launch rehearsal of record**: `poleis/ceremony/the-launch-press-rehearsal.sh` through the job-slot road (oldmac left out: "its kit
  differs (... the agents' mind fable-fusion-27b-q4-64k:latest none there ...)"), slot 1 on this Mac, every feature the validator knows,
  nothing read beyond this Mac, 01:22:48 to 02:00:40 UTC, exit 0. Its certificate,
  `poleis/ceremony/launch-press-rehearsal-the-agents-read-the-release-2026-11-24.json`, was built from 83a31082c3 with no uncommitted part.
  - Three agents freed by their own minds (Axiom, Wren and Mira; 10.2, 8.5 and 8.1 s), the launch's five acts voted 15 PRO by them.
  - The releases: LAUNCH_SEAL_COMMITS named four landings the lander's ledger held at the run, which the lander's own `cratesFor` says move no
    crate: production 4d91a1e65621; the rider 7902372e4a6b (lane/the-readers-read-the-sealed-pages); the sealed f9010c73bf2e
    (lane/the-roads-proofs-walk-today); never 8eb1e16d36b4, that lane's tip. Agent 1 proposed f9010c73bf2e and agent 2 7902372e4a6b, each by
    its own transaction (442 bytes; the checks hold their keys for it).
  - Each agent's step read both (16.7, 16.2 and 15.2 s) and was shown, of f9010c73bf2e: "the lander landed it at 2026-11-24T00:39:55.535Z:
    ... its gate: 1515 of 1541 tests green, no new red, 26 red on main's own tree as well; 3 of 3 proofs; 4 of 7 walks (3 red on main's own
    tree too) (lane 11 of a batch of 12, gated as one tree)", and of 7902372e4a6b the same gate, landed at 00:39:33.463Z as lane 10 of that
    batch; production 249d3e8f8896 under them by 76 and 75 commits of main's own line; 8 files, 95 lines added and 9 taken out, and 5 files,
    153 and 68. The three were shown one commit, one diff and one gate of each.
  - **6 PRO, 0 CONTRA, 0 ABSTAIN**, each with its mind's own reason (all six in the certificate, `checks.sealRoad.releases[].choices`), for
    instance Mira's on 7902372e4a6b: "The lane's gate is clean: no new reds, all proofs pass, and it's a documentation- and UX-focused change
    with a small diff."
  - Four faults put in, each refused: a release not on main (76bffb69b6c3, the tip of a lane that never landed, its work order whole):
    "NOTHING CAST: 76bffb69b6c3 is not on main (main stands at 726e477d11cf): a release of the house's code is a commit of main, so nothing is
    put to the mind"; a hash not its commit's: "NOTHING CAST: the set's release hash is not its commit's (the release of f9010c73bf2e at nonce
    0 is 258bb250e843)"; a replayed envelope and another chair's: `press ballots` 0 held, 4 refused, and `press crank` "4 of the 4 release
    ballots handed over are not held, and nothing is sent". The agents' own ballots then: 21 held, 0 refused (15 of the acts, 6 of the
    releases).
  - The crank landed the three agents' ballots of each release in one batch (867 bytes, 6,862 CU, a fee of four signatures) and sealed unit 1
    (343 bytes, 7,805 CU): "RELEASE 6bbae9fa38a9 (the house's code at 7902372e4a6b): unit 1 PASSED, no seat holder asked: sealed RATUM"
    and the same of 258bb250e843 (f9010c73bf2e). The checks: unit 1 PASSED with 3 PRO each, root 0 holding no vote, 0 release envelopes
    signed by the checks.
  - The house's reader: SEALED f9010c73bf2e (RATUM at slot 1242) and 7902372e4a6b (slot 1240); never NOT SEALED; the road naming proposers
    NOT READ. The lane's lander on a scratch clone: never NOT SHIPPED; "SHIPPED: production 4d91a1e65621 -> 7902372e4a6b"; the swarm's newest
    seal "SHIPPED: production 7902372e4a6b -> f9010c73bf2e"; the house's own production stayed at 249d3e8f8896.
- **A first run**, at the same tip (01:18:30 UTC), stopped after 115 s at the deploy's cut, before any step of this lane's: the second
  airdrop read the deployer's balance at the need exactly (0.076268280 SOL), then 14 more writes of the killed pass landed and took their
  fees, and the resumed deploy found it short by 0.000070000 SOL. It was there before this lane (MAINNETS TIP AND RUNG REHEARSED met it once
  and passed on its rerun); the second run passed the same step with 101 writes landed. Not mended here (below).

## What is not done

1. **What a mind reads of a release** is the commit's words, its diff stat, the gate's verdict and the chain's word, not its code and not
   its lane's receipt (the ledger names the receipt's path; the step does not open it).
2. **The gate of an older landing.** The lander keeps its ledger to 1 MB and one rotated part (about 3.5 hours of landings each on the
   night of Nov 23 to 24), so a landing's verdict is readable for some hours; past that the mind is told the ledger no longer holds it, and
   what git says. The durable record of every landing is its row on the Nest (`house/landings/<merge12>`), which this road does not read:
   the agent's step reads files and git, and opens no door.
3. **The proposal is still the checks'.** Each release is proposed by the checks holding its agent's key (`propose_raw_of`); the agent's
   own step does not propose, and its SOL is still THE BRANCH VOTES ITS OWN CODE's owed 2.
4. **The minds deliberate alone**, at temperature 0.7, one model for three chairs (THE RITE FREES THE AGENTS, not done 2, 5 and 9).
5. **The script's default commits** stay A RELEASE IS A VOTE's (Nov 23 12:09 to 12:21 UTC): the ledger no longer holds them and production
   (249d3e8f8896, Nov 23 15:03 UTC) is past them, and a run that keeps them shows its agents that. The run of record named four landings the
   ledger held when it ran (above); the script follows what the minds choose either way.
6. **A mind on the other Mac.** oldmac's ollama serves rp-24b and qwen3:14b and no fable-fusion-27b-q4-64k, so the rehearsal's kit keeps it
   off oldmac; which minds sit where is his (THE RITE FREES THE AGENTS, not done 2).
7. **A mind's reason is not held to what it was shown.** Wren's PRO on f9010c73bf2e says "only one lane in the batch", where its step showed
   "lane 11 of a batch of 12, gated as one tree"; Mira calls the same landing "a new tracenet landing page". The hands hold what a mind is
   shown and sign what it chose; nothing reads its reason back against the facts.
8. **The deploy's cut can race its top-up** (the first run above): the killed pass's last writes can land after the second airdrop reads the
   balance. Waiting for the deployer's balance to stand still before that airdrop is a follow-up of the rehearsal, not of this road.

**Not run by this seat:** the lander's own gate (at the hand); the council crate's whole `cargo test` (its live tests are the rehearsal's);
anything against devnet or mainnet.

## Carried to main (Nov 25 2026 UTC, lane/the-agents-read-the-release-lands, seat claude-030b39a0)

- **Why it was not on main.** Handed at 08af39d940 on Nov 24 02:04:56 UTC; the lander refused it at 02:37 UTC for a `merge-tree` conflict with
  main in one header region of `poleis/ceremony/the-launch-press-rehearsal.sh` (THE TREASURY PAYS THE COMPUTE had moved those lines; the verdict is
  named in `poleis/ceremony/THE-REHEARSAL-ASKS-FOR-ITS-MIND-LANDING.md`, the lander's own record of it rotated out). Its seat ended without
  handing it again.
- **The mend.** `git merge main` (main 02e3200780 and after): the one conflict, comments only, keeps both main's words (the treasury's dollar cap
  and the router's month as the launch's seventh and eighth acts) and the lane's (the releases voted by the agents' own minds, so "the releases"
  leaves main's list of what no mind reads). The script's body, `poleis/press/src/*`, its tests and `council.rs` merged clean. The kit line THE
  REHEARSAL ASKS FOR ITS MIND took from this lane is identical on both sides. No other change.
- **Checks on the merged tree, nice 19.** `poleis/press` `--test release_read` 10 of 10, `--test chair` 20 of 20; `poleis_treasury` `--test
  council` 92 of 92 (5 ignored); `bash -n` on the rehearsal script; the-javascript-only-shrinks 39 of 39, any-room 12 of 12, the-law-is-the-gate
  8 of 8. Not run: the rehearsal itself (2,271 s of a validator and the agents' mind) and the press's whole suite; the gate's.
