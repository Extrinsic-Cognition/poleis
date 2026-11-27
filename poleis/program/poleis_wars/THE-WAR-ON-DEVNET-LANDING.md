# THE WAR ON DEVNET: the war's program on Solana's public test chain, and his first war played live there (Nov 24 2026, lane/the-war-on-devnet, claude-54dd2237)

## His words, UTC

- **12:24:57** "Yes, devnet", to "Put the war on devnet next, Solana's free test chain, so screens and phones can read a live war?".
- **12:31:20** "Yes, use it", to "Use your devnet wallet's free test SOL for the deploy and one live war (up to 40, rest sent back)?".
- **12:55:00** "Yes, run it", to "Run the live war on devnet: 40.9 test SOL out of your wallet, 32.8 back, 8.1 kept by the chain?".
- After it, the frame (ec-workspace games doc, section 36): **13:13:47** "i think you should realize we are building a game publisher firm... the
  whole mechanism of meme royale is to use the blockchain as multiplayer like pump.fun"; **13:24:27** "poleis is meta-swarm... this gaming shit is
  1 swarm"; **13:27:18** "Players' coins". The war's program is the game's server; this run replays his war with 40 stand-in players.

## Cost first

| | before (main 21204e73b5) | after (this lane) |
|---|---|---|
| the war's program on a public chain | none: a placeholder id nobody held a key for | `6ZA4DVYtEEDZF4nTjcaxzeWLxWUs7Mrve9EANzjRs4Ef` on devnet, deployed in slot 525,795,802, the same bytes as this tree's build (140,112, sha256 `89599aa8cc59…`, read back with `solana program dump`); its upgrade authority a throwaway devnet key on the new Mac (`FT9u9…YdzV`, `~/.embertide/poleis-wars-devnet/`, never in git) |
| the deploy | | the first try (`--use-rpc`) died after 18 min on the public RPC's 429s; its half-written buffer was closed (0.7126 test SOL back); the second, writes over the validators' own ports, landed in about a minute |
| his first war, live | on a local validator only | PASS on devnet (solana-core 4.5.0-beta.0): 219 transactions, 151 minutes at 10 s, none missed, each sent in the first second of its minute and confirmed 2.0 to 3.2 s later; the crow beat its line at 148 and the wolf's stand failed at 150; 40 fighters, 10 deserters, 10 converts, 0 differences from his model; 1,941.5 s |
| the public RPC | | 2,162 calls paced at one in 300 ms; 32 answered 429 and were waited out |
| units on devnet | | where no address enters, the local validator's: settle 40,274, a crank 5,718 to 5,976, desert 6,596 to 6,604, sell 7,076 to 7,084 |
| his devnet wallet (`3nJn2…KMba`, test SOL) | 80.711 | 71.854, and 0.75 more held by the deploy key for the next upgrade, back after it: 40.92 out on his three yeses, 32.063 back; 8.10 kept by the chain (the program's rent 0.713; the war's 7.392: 6.231 behind the deserters' burned coins in the crow's reserve, 1.088 the war's fees in its fee account, 0.072 rents, 0.002 network fees) |
| tests (the new Mac, nice 19) | | poleis-war 18, poleis_wars 19, green under the devnet id |
| JavaScript | | none |
| at rest | | nothing runs: the program and the war's accounts stand on devnet and cost nothing until a transaction comes |

## What stands now

- The program at its devnet address (`src/lib.rs` declares it). Mainnet's id and authority are the game swarm's, never a seat's.
- `war-rehearsal --cluster devnet` (`rehearsal/`): the copied RPC gains a pinned Devnet cluster (https://api.devnet.solana.com only, genesis
  `EtWTRABZ…`; the press itself still refuses devnet), paced under the public limits; its stand-ins are funded by a payer key and made from a seed
  written first (mode 600, never printed) so a dead run can be refunded (`--refund`); after the war every stand-in sells once the dump tax has
  fallen and sends everything back (`--return-to` sends the payer's surplus to his wallet).
- The war's accounts left standing for any screen or phone to read:
  [config](https://explorer.solana.com/address/6tvNyevKti9Xo7oUToEikjiAZ4sFhVwsAyAc6skMUj1s?cluster=devnet),
  [crow](https://explorer.solana.com/address/92mkYYhz9NQx6htqZmAR7sdvgGf4jEebsLfYbsBAuhHB?cluster=devnet),
  [wolf](https://explorer.solana.com/address/53MpZzEhBDvnGh5s8vthXLYdEPZvGpoPbcL5E3SyCQGU?cluster=devnet),
  [raid](https://explorer.solana.com/address/Fhk3HuuYfH9HBrBPjZWGJHDpyRA8rU9Urxfh9tz7cdRH?cluster=devnet),
  [fee account](https://explorer.solana.com/address/8CPTt95ipaxUvjJdwfx3XFGujXxLSvSdQnAHm9Qbd1K7?cluster=devnet).
- The receipts: `rehearsal/receipts/war-devnet-2026-11-24T125542Z.json` and its `-return.json`.

## Owed

- THE GAME IS ONE SWARM (lane/the-game-pays-its-swarm, from this lane's tip, on his "Yes, pay them" and his frame): each coin's creator paid its
  share, the game swarm's Poleis treasury the publisher's, the rules a generation the swarm founds. Its layout is version 2: this war's
  version-1 accounts are then refused, and what they hold stays where it is.
- The upgrade on devnet after it, paid from the 0.75 test SOL the deploy key holds, then that back to his wallet.
- Players signing their own moves from their own devices: this run signs for 40 stand-ins.

## The gates

- At the ship (THE CRATES GATE THE SHIP): `poleis-war`, `poleis/program/poleis_wars`, its `rehearsal`.
- No gate yet on the chain: the measure it waits for is the program's bytes on devnet against main's build, and the rehearsal's receipt held to
  the reference, read at the ship; today both are read by hand (`solana program dump`, the receipt).
