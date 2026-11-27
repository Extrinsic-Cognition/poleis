# THE WAR IS ITS OWN PROGRAM: the price wars on Solana, his rulebook in integers, run on a local validator (Nov 24 2026, lane/the-war-is-its-own-program, claude-54dd2237)

## His words, UTC

- **Nov 24 10:37:13,** to "Build the war's program first, on a local validator?": "Yes, build it".
- The rules are his, all of them that morning: **01:07:37** "What my coins sell for"; **01:08:33** "you should follow always the meme royale formula
  which means you are having coins that are required... the figth is exactly about it, if you desert the fight, you deserve to lose"; **01:14:54**
  "Half" (a deserter leaves with half of what he brought); **09:43:11** "Yes, run it" (Meme Royale's whole rulebook, with his half); **10:09:09** "It
  lives on" (the winning coin lives on); **10:14:40** "Yes, burn them" (a deserter's coins are burned, never sold back); **10:27:36** "Keep Meme
  Royale's" (no cut: the loser's SOL buys the winner's coin for the loser's holders).
- Where the rules are drawn and argued: ec-workspace `docs/how-the-games-work-not-all-games-are-the-same-the-codex-brainstorm-read-whole-hypertrace-2026-11-23.md`,
  sections 25 to 31 and 35; Meme Royale's program read back from its bytes: `docs/meme-royale-the-program-read-back-hypertrace-2026-11-24.md`.

## Cost first

| | before (main 24bee35b0c) | after (this lane) |
|---|---|---|
| the war's rules | a Python model (`meme-royale-rulebook-sim.py`), floats and ints mixed, on no chain | one law in integers, `poleis-war/src/law.rs`: no_std, no allocation, no dependency; the program and the replay call the same functions |
| his model's four reference wars | Python only | replayed by the law (its replay, feature std) and by the program's own SBF bytes instruction by instruction (Mollusk), every lamport and coin equal to `reference-in-sol.json` (byte-identical to the model's output) |
| the war on a chain | none: pump.fun's program cannot burn a deserter's coins or convert holders | `poleis_wars`, 12 instructions, 140,112 bytes of SBF, sha256 `bc8bdcad58c58cbe829b6e8c23e442cf495cc3565fbc40cca9237027f52ec9e5` |
| the first reference war on a real validator | never | run whole on `solana-test-validator` 4.2.0 (Agave, feature set 565236538): 178 transactions, 151 minutes played and none missed, the crow beating its line at minute 148 and the wolf's last stand failing at 150 as the reference says, 40 fighters, 10 deserters paid, 10 converts, 0 differences; 615.5 s at 4 s a minute; the application firewall saw 0 inbound flows |
| the dearest instruction | | `settle`, 41,195 CU on Mollusk and 40,274 on the validator (a binary search for the most the winner's curve takes), of the 200,000 an instruction may ask |
| a trade (Mollusk) | | `buy` 7,752 (8,802 to 17,803 when it births the position), `sell` 7,257 (7,474 under the dump tax), `desert` 6,596, `change_sides` 12,840 to 13,890, `crank` 5,718 to 5,974, `stake` 12,457 to 13,345, `unstake` 4,498, `convert` 8,310 to 9,360, `unlock` 4,002; on the validator (its receipt) each kind is within a few units of these or above them by multiples of about 1,500 (a PDA's bump search taking more tries: the run's keys are its own), the most a buy of the forty, 22,303 |
| the accounts | | Config 183 B, Fees 67 B, House 200 B, Position 123 B, Raid 255 B; a position's rent is its owner's, once |
| tests (the new Mac, nice 19) | | poleis-war 18 (law 16, reference 2); poleis_wars 19 (id 1, brought 3, cost 1, refusals 12, wars 2); green |
| the diff | | 39 files: 3,846 lines of Rust source, 2,168 of tests, 527 copied whole from `poleis/press` (rpc.rs, wire.rs, with a provenance line), 2,143 of the reference as JSON, 721 of the model as data (the two `.py`, run by nobody at run time), 170 of the script, 200 of receipts, 4,248 of `Cargo.lock` |
| JavaScript | | none |
| at rest | | nothing: no process, no deploy; a program on a chain costs nothing until a transaction comes |
| what it cost to build | | an Opus builder to this seat's briefs: 405,424 tokens written, 0.7 M of fresh context and 58 M read from its cache (123 calls, each counted once), over 67 minutes; then this seat's review, the fixes below and four runs on a validator (about 30 minutes of one core at nice 19) |

## What stands now

**The law** (`poleis-war/src/law.rs`): pump.fun's curve (vt, vq, rt, rq, rt0; fees 95 and 30 bps, rounded up) priced in SOL; the stake that opens
a raid (Meme Royale's need: the larger of 5 % of the attacker's market cap and 2 % of the gap, at most 20 % staked, an opponent between half and
twice); the raid's length (15 minutes to 6 hours by the coins both houses sold); the line (the market cap holding both reserves and both chests),
met when the climb beats what is left times the time left; the last stand (2 minutes, revival at the line and a ninth more); the clock (the further
along wins, a tie to the defender); his burn (a deserter is paid the least of his coins' worth, half of what he still has in and the reserve, and his
coins are burned); the settlement (the loser's reserve and chest buy the winner's coin on the winner's own curve, fees and all, the rest to the
winner's chest); the conversion (each loser holder's share by his coins, a staker's a tenth better); the dump tax (Meme Royale's bytecode: the larger
of 10 % and the share the raid added, falling to nothing over 60 s plus a step per share of half a sold-out curve, no cap). Every comparison is
exact (cross-multiplied); the compiler checks that no product leaves u128.

**The program** (`poleis/program/poleis_wars`, pinocchio 0.11, no Anchor): `init_config` (the rules, once, by their authority), `init_house`
(anyone), `buy`, `sell`, `stake`, `unstake` (a proposal only), `desert`, `change_sides`, `crank` (anyone, the raid's minute), `settle` (anyone),
`convert` (each loser holder, once), `unlock` (the winner's stakers). THE BOOK: a house's lamports are its rent, its reserve and its chest,
exactly; the fee account's its rent, the fees booked and what it held before its birth. Every account is a PDA checked on every read, with its
owner, size, discriminator and layout version.

**The rehearsal** (`poleis/program/poleis_wars/rehearsal`, binary `war-rehearsal`; `sh poleis/program/poleis_wars/rehearse-on-a-local-validator.sh`):
a fresh `solana-test-validator` 4.2.0 with poleis_wars loaded at genesis, every key made in memory and dead with the run, RPC on loopback only,
a 20-minute alarm, everything at nice 19; the first reference war played minute by minute on the chain's own clock, each minute's events and its
crank in one transaction held to land in that minute, both houses held to the law after every minute, then the settlement, the converts and the
unlocks, every account read back and held to the reference. The receipts are in `rehearsal/receipts/`.

## The four runs

| run (UTC) | minute | result | what it found |
|---|---|---|---|
| 11:42:14 | 2 s | red | all 150 minutes landed in their minute and equal to the law; the last wait asked the chain without its history, so the run's first 118 transactions (older than about a minute) read as unconfirmed: the wait now searches the history |
| 11:50:35 | 2 s | red | minute 68's transaction landed in minute 69 of the chain's clock (whole seconds, corrected toward its votes' time, on a Mac other seats were loading): the minute is 4 s |
| 11:54:06 | 4 s | red | all 150 minutes landed in their minute and equal to the law again; at the end 137 of the 178 still read as unconfirmed with the history searched (the status cache answers for about two minutes and this validator's history search found none older): each transaction is now confirmed, and its units read, while it is recent, after every landing |
| 12:07:22 | 4 s | green | 178 transactions, 151 minutes and none missed, each sent in the first second of its minute; every account held to the reference, 0 differences (`receipts/war-rehearsal-2026-11-24T120722Z.json`) |

## Found in review and fixed before the hand

- **His Half could be dodged** (this seat's review): `brought` never fell, so one who bought much, sold it outside a raid and bought a little back,
  or one who deserted and came back in during the same raid, deserted at full worth. Now a sale takes its share of what he brought out with it
  (`brought_after_sale`) and a desert or a change of sides ends it. `tests/brought.rs`; no reference number moved.
- **A stake caught in a proposal** (the brief's): a holder who staked in his house's proposal could not take it back, and while his house was raided
  by another his coins stayed locked. `unstake` (Meme Royale's withdrawProposalStake): a proposal only.

## The readings the program chose (not his words; each one named so his word can move it)

- A minute is read at the instant someone cranks it, against the houses as they stand then, once; a minute nobody cranks is not read. The raid
  therefore ends when its last minute begins (the first crank in it decides), and a stand when its stand's last minute begins. The model reads
  each minute after its trades: the program equals it when each minute's trades come before its crank, as in the reference wars and the rehearsal.
- The shield is "younger OR smaller" (Meme Royale's 25 hours and 25 SOL; the tests and the rehearsal set both to zero).
- A buy whose quote buys no coin is refused; a sale whose quote passes the reserve is refused (once coins were burned, the curve prices the coins
  left above what the reserve can pay for all of them).
- The lamport a settlement's buy may leave goes to the winner's chest.
- A dump tax is replaced only by one that starts higher than the running one stands now.

## What is not built (owed, each to its own lane)

- Where the war's fees go: they pile up in the fee account and nothing takes them out (the owner's call; since Nov 23 the swarm's vote).
- Closing what the law has finished with (spent positions, settled raids): their rent stays.
- Graduation to a market when a curve sells out; referrals; Meme Royale's cooldown between raids.
- A deploy: devnet with a throwaway key is a press away; mainnet is the swarm's (the program's upgrade authority is the polis's, never a seat's).
  The program's id is a placeholder.
- The road's stops after it: the fight on screen, the phone as the blade, the clip.
- The other three reference wars on a validator (the fourth is the only one whose last stand revives): they stand on the program's own bytes in
  `tests/wars.rs` only.

## The gates

- At the ship (THE CRATES GATE THE SHIP): `poleis-war`, `poleis/program/poleis_wars` and its `rehearsal`, their whole `cargo test`: the law's units,
  the four reference wars equal to `reference-in-sol.json` (the law's replay and the program's), every refusal, the costs, what a deserter brought.
- No gate yet on the chain: the measure it waits for is the rehearsal's receipt, every amount equal to the reference on a validator, read at the
  ship once the lander runs a validator; today it runs by hand, about 11 minutes.
