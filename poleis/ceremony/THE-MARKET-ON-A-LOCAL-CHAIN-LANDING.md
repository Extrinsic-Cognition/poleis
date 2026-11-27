# THE MARKET ON A LOCAL CHAIN: the two programs' chain on a real Solana validator, under devnet's rules and mainnet's (Nov 21 2026, lane/the-market-on-a-local-chain, claude-81203e94)

## His words, UTC

- **Nov 21 14:40:51,** to "Rehearse the market on a real local chain, then hand you the devnet press?": "Yes, rehearse it".
- **16:08:07:** "you doing these things on devnet?"; **16:09:35:** "ok, so you are simulating local solana stuff on newmac?"; **16:11:59:**
  "yeah but it is not a real solana node is it? it is not mainnet... it is a localnet emulator not?"
- **16:32:50,** to "May the local validator read devnet's and mainnet's public feature lists?": "Yes, read them".

## Cost first

| | before (main 11c03ff5c8) | after (this lane) |
|---|---|---|
| THE TWO PROGRAMS IN ONE WORLD on a real validator | never run: Mollusk only, and no market program on any chain | run whole on `solana-test-validator` 4.2.0 (Agave), 94 transactions and 6 refusals, under every feature, devnet's and mainnet's |
| the largest transaction | not measured (Mollusk holds no size) | 1,010 bytes of 1,232 (a batch of four votes and `attest`), 222 under |
| the dearest instruction | `decide`, 29,515 CU on Mollusk | 29,516 on the validator; no transaction asks a compute budget of its own (the default is 200,000 an instruction) |
| a step's units on the validator against Mollusk's | | equal on 70 to 75 of 93 paired steps; every other differs by a multiple of 1,500 (one address search more or fewer: a run's keys are its own, and the proposals' hashes name them) or by 1 (the market's arithmetic on a road a trade reached a slot later) |
| the chain's outcomes | Mollusk: verdicts [1, 2]; paid [0.5 SOL, 0]; each stake back 2,000 $WARM; the house earned 4,525.035894; the book 35,000 of 35,000 | the same on the validator, to the micro, under each of the three rule sets |
| a run's wall time | | 1,148 to 1,226 s (19 to 21 minutes), 2,276 slots at about 0.5 s on the loaded new Mac |
| the programs | | unchanged: Poleis 456,800 bytes, sha256 `aa65f59df1d3e91a…`; the roads 101,968 bytes, sha256 `c6255f7933266f94…`; each read back from the validator and held to this tree's build |
| Poleis's tests (the new Mac) | 216 passed, 1 ignored | 222 passed, 2 ignored (the live test needs a validator); the client 20; the roads 26; no warning |
| the dependencies and the lock files | | unchanged: the JSON and the HTTP the test speaks are its own, about 300 lines |
| JavaScript | | none |
| what a run costs | | a validator at nice 19 on the new Mac, about one core for 20 minutes; nothing on the serving Mac |

The diff against main 11c03ff5c8: `poleis/program/poleis_treasury/tests/council.rs` (+1,315 -59), `tests/built/mod.rs` (+2), the script
`poleis/ceremony/the-market-on-a-local-chain.sh` (+198), the two certificates (+391, +389) and this receipt. No program source, no
Cargo.toml and no lock moved. Nothing is deployed.

## What stands now

**One chain, two worlds.** The chain of THE TWO PROGRAMS IN ONE WORLD (from a board opened for the polis's own hand to a budget paid on a
road's verdict and the house's collect) is one body, `two_programs`, over what it asks of a world (`Ledger`: send an instruction or a
transaction and get its units or its refusal, read an account, the slot, wait for a slot, a funded wallet, a voter, a mint, the roads
program). Mollusk's `World` runs it as it ran and prints what main printed (set_law 17,143 CU, set_board 16,417, CODEX_MARKET 24,349,
lay_for 9,069 and 12,069, collect 6,409). `estate`'s founding is the same body for every Mollusk test (`founded`), its sale's slots counted
from the slot it opens at, so a validator's funding of the founders takes none of the windows.

**`Live`, a validator on this machine and nothing else.** JSON-RPC over `std::net::TcpStream`, to `http://127.0.0.1`, `[::1]` or
`localhost` (read as an address, never asked of a resolver; 16 other URLs refused by a test). The legacy transaction is built by hand in
Solana's own key order, held byte for byte to Solana's own vector (`test_sdk_serialize`, solana-transaction 4.1.5). Each step is simulated
with its signatures verified, sent, and confirmed; a program's refusal comes back as the `ProgramError` the body expects, and a refusal no
program wrote (a size, a budget, a lock, rent, a signature) ends the run with the validator's words. Every key is made in memory from the
system's randomness for that run and dies with it.

**The script** builds both programs as the tests build them, starts the newest validator installed with both at genesis at their declared
ids, runs the ignored test `the_two_programs_on_a_live_validator`, which writes the certificate (each step's signature, slot, units, the
simulation's units, Mollusk's units and bytes; the refusals; the rules; the outcomes), and stops the validator and removes its ledger
whatever happened, under an alarm of its own and one on the validator. The rules: `MARKET_FEATURES`, every feature by default, reading
nothing beyond this machine; `devnet` or `mainnet-beta` has the validator take that cluster's feature set at genesis from its public RPC,
read-only, the run's one read beyond the Mac, by his word of 16:32:50.

## What the validator proved that Mollusk could not

- **The size:** every transaction of the chain fits a packet; the largest is a batch of four votes, 1,010 bytes.
- **The compute:** the dearest instruction is 14.8 % of the default budget.
- **The signatures:** the votes' precompile ran as a validator runs it; a signature over other bytes was refused by the validator at the
  first transaction (a fault run live, below).
- **The clock:** real slots hold the windows; a client that never waited was refused by Poleis itself at the first reveal (`Custom(6)`).
- **Rent and locks:** every account the programs made was rent-exempt on a real bank; no transaction was refused for rent or a lock.
- **The loader:** both programs ran under the upgradeable loader, and the polis's own hand called the roads program through it.
- **The rules:** under devnet's 253 active features and mainnet's 251 (each set taken from that cluster's public RPC at genesis, by his word;
  devnet holds 4 that mainnet does not, mainnet 2 that devnet does not), the chain ran the same: 94 transactions, the same 6 refusals, the
  outcomes to the micro. 15 steps' units moved between the two runs, each by a multiple of 1,500 (from -7,500 to +6,000: each run's own keys
  make its address searches), none by the rules; the dearest is `decide` at 29,516 under both.

| the run (at 8f7e68bca2) | active features (sha256 of the sorted ids) | transactions, refusals | wall | slots | units equal to Mollusk's |
|---|---|---|---|---|---|
| devnet's rules (`market-on-a-local-chain-devnet-features-2026-11-21.json`) | 253 (`78a31492e67cde4f…`) | 94, 6 | 1,189 s | 5 to 2,281 | 75 of 93 |
| mainnet's rules (`market-on-a-local-chain-mainnet-features-2026-11-21.json`) | 251 (`b5aecdc0c1b5286b…`) | 94, 6 | 1,226 s | 3 to 2,279 | 74 of 93 |
| every feature (the builder's, at 60f8a8557a; not committed) | every feature 4.2.0 knows | 94, 6 | 1,148 s | 3 to 2,279 | 74 of 93 |

## Beyond loopback

The test validator 4.2.0 binds its RPC (8899), websocket (8900) and faucet (9900) on every interface whatever `--bind-address` says, and a
UDP socket at `*:8000`; no flag of 4.2.0 binds them to loopback. The script names exactly those (and, under a cluster's rules, the read's
connection to that RPC's port 443), stops the validator before anything runs if anything else is open, and reads the application
firewall's log after: on, in stealth mode, it saw 0 inbound flows to the validator in every run. Owed: a device on the LAN that connected
during a run would meet the firewall's question for an unknown binary on the new Mac's screen (the binary is signed by its linker only);
the count is said at every run.

## How it was built and verified

An Opus builder made it to this seat's brief, in four commits; this seat reviewed every line, then added two: the outward read asked by
name only (the builder's default had been devnet's), and a number held whole by values no float holds. Its own choices stand: no
serde_json (the brief was wrong: it is not a dev-dependency), the founding made generic so every test shares it, a real token mint.

Faults this seat put in what the lane added, on the new Mac, each caught: a host that is not loopback let through; a path with control
bytes let through; the signers' groups swapped; the header's counts swapped; compact-u16 a byte short; a program's refusal read one off;
base58 without its leading ones; a body cut short taken whole; Mollusk's world never warping; and, live, a client that never waits for a
slot (refused at the reveal, 16 s in) and a signature over the wrong bytes (refused at the first transaction, 5 s in), each leaving no
validator, test, port, ledger or key file. The fault of a number read through a float survived the test as the builder wrote it
(u64::MAX survives a float by saturation); the test now holds 2^53 + 1 and 2^64 - 3, and it is caught.

A run ended by TERM mid-chain (exit 143) left nothing running and the ports free. The house's post-quantum gate on the tree: exit 0,
`council.rs` at its 4 lines; no line the lane added names a classical scheme. Poleis's ballots stay owed Falcon-512 in a program (lane 8).

## What is not done

- **The devnet press, his.** Devnet's ceremony Poleis still runs the Oct 29 build, which cannot read a road. In order: upgrade it to this
  tree's Poleis under its own id by a `CODEX_UPGRADE` his Keystones seal; deploy the roads program under a new id whose key his hand makes,
  built with that id declared; open a board for the polis's agent key; seal the `ACT_RAW` acts `set_law` and `set_board`, then
  `CODEX_MARKET` with the board, each voted by his Keystones.
- A SIGKILL of the script before its trap leaves the ledger folder in `$TMPDIR`; the validator's own alarm still ends it within the hour.
