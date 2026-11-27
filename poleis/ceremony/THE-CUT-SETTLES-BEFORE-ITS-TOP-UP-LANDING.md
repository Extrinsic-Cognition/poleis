# THE CUT SETTLES BEFORE ITS TOP-UP: the launch rehearsal's deploy cut waits for the deployer's balance to stand still before its second airdrop (Nov 24 2026, lane/the-cut-settles-before-its-top-up, seat claude-3a111591)

## The ask

The session that started this seat (Nov 24 2026 ~02:0x UTC) asked it to mend the race claude-ba1ae67d met while rehearsing
lane/the-agents-read-the-release (`poleis/press/THE-AGENTS-READ-THE-RELEASE-LANDING.md`, its first run and its not-done 8; MAINNET'S
TIP AND RUNG, REHEARSED met it once before). After the kill and before the airdrop, the script waits until the deployer's balance stands
still, read by the validator's own getBalance on loopback, about 30 s at most and said if it never stands; in the shell script; the
press's need untouched. It mends the rehearsal of the launch press his "Yes, build it" of Nov 21 2026 asked for; no new word of his is
behind it.

## The cost, first

| | before | after (this lane) |
|---|---|---|
| step 5, after the SIGKILL | the second airdrop at once | the deployer's balance read until it is unchanged over 8 slots at the processed level with the confirmed level (the press's) the same, 30 s at most and said if it never stands; then the airdrop and the resume as before |
| what it adds to a rehearsal | | 4,358 ms in the run of record (8 slots of a still balance and the reads); two getBalance calls on loopback every 0.4 s while it waits, nothing after |
| runs stopped at the cut | two known: MAINNET'S TIP AND RUNG, REHEARSED (once) and Nov 24 2026 01:20 UTC (the second airdrop read the balance at the need exactly, 0.076268280 SOL; 14 writes landed; the resumed deploy refused, short by 0.000070000 SOL, exit 3) | none in this lane's one run; the race did not come in it (THE RUN OF RECORD says why) |
| code | | shell only: `poleis/ceremony/the-launch-press-rehearsal.sh` +51 −1 (the header's line 7 becomes two, the step 49 lines); no JavaScript, no Rust; the press untouched |
| the proof | | `sh -n`; the step's block run alone against a stand-in getBalance (below); the launch rehearsal through the slot road on this Mac: exit 0 in 2,567 s, through the cut and to its certificate |

Nothing was sent to any public cluster, no key of his was made or read, no Canvas tab was born.

## The cause, read in the press

Every step of the press reads its need and then the deployer's balance, two reads (`press airdrop` in `poleis/press/src/main.rs`, the
deploy's check in `poleis/press/src/chain.rs`), both at the confirmed level (`rpc.rs`). The need counts a buffer's write for each chunk
that still differs from the binary, each at its fee (`chain.rs` need). On a chain at rest the need is exact: a write that lands lowers the
balance and the need by the same fee, so the deployer that held the need before holds it after. While the killed pass's last writes still
land (each was sent before the SIGKILL, BATCH at a time, and lands a few slots later), a write that lands between the two reads is counted
still to write while its fee is already gone, and the step comes out short by exactly those fees: 14 writes of 5,000 lamports are the
0.000070000 SOL of 01:20 UTC. So the mend is to let the chain come to rest before any step reads it again, not to count the need otherwise.

## What it does

`poleis/ceremony/the-launch-press-rehearsal.sh`, step 5b, between the kill and `press airdrop`:

- the deployer is the address the killed deploy printed (its need's line `the deployer <address> holds`);
- `balance_at processed` and `balance_at confirmed`: the validator's own getBalance on its loopback URL, each the lamports and the slot
  the validator read them at;
- it stands when the processed balance has not moved over 8 slots of the validator's own clock and the confirmed one is the same (the
  press reads at the confirmed level: nothing it has not yet seen is still in flight); every 0.4 s, 30 s at most;
- it says either `THE CUT SETTLES BEFORE ITS TOP-UP: the deployer's balance stood at ... SOL from slot ... to ..., ... ms after the kill;
  it moved ... times while read, ... lamports in all`, or that it did not stand in 30 s, and then the airdrop reads it as it is (the run
  goes on as it did before this lane).

## The proofs

- `sh -n poleis/ceremony/the-launch-press-rehearsal.sh`: no error.
- The step's block run alone (its lines from `deployer=` to its `fi`), against a stand-in `curl` answering getBalance: a balance that
  fell 5,000 lamports a read five times, the confirmed level a read behind, then stood: `THE CUT SETTLES BEFORE ITS TOP-UP: the
  deployer's balance stood at 0.076243280 SOL from slot 105 to 113 (processed, and confirmed the same), 7482 ms after the kill; it moved 5
  times while read, 25000 lamports in all`. A balance that never stood: `did not stand still for 8 slots in 30 s (... it moved 52 times):
  the airdrop reads it as it is`.
- THE RUN OF RECORD (below).

## The run of record

Asked at 02:22:44 UTC at this lane's 0e517b6036: `JOB_SLOT_WAIT_S=3600 sh poleis/ceremony/the-launch-press-rehearsal.sh <a certificate
outside the tree>` (so `--here`: not-done 2 says why), every feature the validator knows, the rehearsal's own params. It took slot 2 of 3
on this Mac after 17 s (6.1 of 18 cores free, load 26.0) and ended exit 0 at 03:05:51 UTC, 2,567 s in all. Its certificate:
`poleis/ceremony/launch-press-rehearsal-the-cut-settles-before-its-top-up-2026-11-24.json`. At the cut:

- `the deploy killed at Poleis's write 100: 104 of its writes had landed in that run`
- `THE CUT SETTLES BEFORE ITS TOP-UP: the deployer's balance stood at 0.080837080 SOL from slot 120 to 128 (processed, and confirmed the
  same), 4358 ms after the kill; it moved 0 times while read, 0 lamports in all`
- `the deployer holds 0.080837080 SOL, the need 0.080837080 SOL: no airdrop`
- the resumed deploy: `poleis: 128 of its 434 chunks stood in the buffer already; 306 left to write`; Poleis deployed at slot 156.

What it shows, and what it does not: the killed pass had sent two batches of 64, and the 24 writes past the 104 its log had said were all
in before the step's first read, so in this run the balance never moved while read and the race did not come. What the run measures is
the cost on a validator under a real sitting, 4,358 ms, and that every step after the cut reads a chain at rest (the airdrop's need and
balance equal, to the lamport). That a balance still moving is waited for, and one that never stands is bounded and said, is the stand-in's
proof above, not this run's.

## Not done

1. **No gate.** The step is a shell step of a rehearsal of 40 to 60 minutes on a validator of its own, which the lander does not run.
   The measure it waits for: the count of rehearsals stopped at the cut (`the resumed deploy refused` after a settle line) among those
   run, read from their logs; and the settle line's milliseconds, read at each run.
2. **The rehearsal's kit test does not ask for its mind.** The slot road sent this lane's first run (asked 02:18:39 UTC, slot 1 of 2 on
   the old Mac at 02:19:56 after 77 s in the line) to the old Mac by its far list. The kit the script gives the road (`slot_kit`: the Solana release, cargo, the wasm32 target, node) does not name the mind
   that THE RITE FREES THE AGENTS needs (`LAUNCH_MIND`, by default ollama's `fable-fusion-27b-q4-64k` on loopback), and the old Mac's
   ollama serves other models and not that one. So the run would have built the press, the seal's reader, the programs and the council
   tests there and stopped at step 2c, `no mind on this machine` (exit 2), before the road brought it back. This seat stopped it at about
   02:22 UTC, while it built the council tests (the old Mac clean after its beat watch: no rehearsal process, no validator, its scratch gone) and ran it on this Mac by a
   certificate outside the tree (`--here`). Every launch rehearsal the road sends to the old Mac pays the same. A follow-up of its own:
   the kit test asks for the mind too.
3. **The other lane on this script.** lane/the-agents-read-the-release (claude-ba1ae67d, in the line) changes this script's header at its
   lines 51 to 152 and its steps from 623 on; this lane changes its line 7 and step 5. `git merge-tree` of main and that lane conflicts
   once, in the header at line 85 (lines THE TREASURY PAYS THE COMPUTE moved on main); with this lane merged in, still that one region and
   no other (at line 86, one header line lower). That conflict is between main and that lane.
