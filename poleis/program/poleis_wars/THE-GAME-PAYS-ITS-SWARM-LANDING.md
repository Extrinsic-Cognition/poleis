# THE GAME PAYS ITS SWARM: each coin's creator paid, and the game swarm's Poleis treasury (Nov 24 2026, lane/the-game-pays-its-swarm, claude-73a178b3)

## His words, UTC

- **13:13:47** "i think you should realize we are building a game publisher firm... the whole mechanism of meme royale is to use the blockchain as
  multiplayer like pump.fun".
- **13:16:24** "Yes, pay them", to "Pay each house's creator its 0.30% and the publisher its 0.95%, as pump.fun does?".
- **13:18:21** "who the fuck cares about parameters? they can be evolved".
- **13:20:53** "wtf dude? u must consider all this as a swarm brought to life through poleis.us".
- **13:24:27** "poleis is meta-swarm diocane. this gaming shit is 1 swarm".
- **13:27:18** "Players' coins", to "In the war, who fights: the tokens of swarms born on Poleis, or coins players launch in the game?".

Poleis is the meta-swarm. The game publisher, this war and every game after it, is ONE swarm brought to life through poleis.us, and its money is
its Poleis treasury. The houses are coins players and agents launch inside that one game, as on pump.fun: each coin's creator share goes to
whoever launched it. The rules are a generation the swarm founds, evolved by founding the next one beside it; no number is chosen in code.

## Cost first

| | before (lane/the-war-on-devnet 4b64a161b0) | after (this lane) |
|---|---|---|
| where a trade's fees go | both into one fee account per config, which no instruction could empty | the creator's share stays in its house, booked as `creator_owed` for whoever launched it; the protocol's share and the dump tax's fee half go to the generation's fee account; anyone pushes each out: `pay_creator` to the house's creator, `pay_publisher` to the generation's treasury |
| the rules | one config per key, `["config", authority]`, fixed for ever | one generation per `["config", authority, generation]`, naming the publisher's treasury; the next is founded beside it, and a house is born into the one it names; the authority (the swarm's seal) only signs, a payer pays |
| compute units (Mollusk) | init_config 9,050; settle 40,271 to 42,069 | pay_creator 2,189; pay_publisher 2,623; init_config 10,611; settle 41,245, still the dearest of the 200,000 an instruction may use |
| account sizes | Config 183 B, House 200 B | Config 223 B, House 240 B (40 more bytes each, the treasury and generation, the creator and what it is owed); Fees 67 B |
| the program | 140,112 bytes | 143,792 bytes |
| his four reference wars | equal to his model to the lamport | unchanged to the lamport (`tests/wars.rs`, `poleis-war` `tests/reference.rs`); paid in the first: the crow's creator 117,786,929 lamports, the wolf's 59,259,260, the publisher 560,646,291 |
| the publisher's share into Poleis | | Mollusk with Poleis's own `poleis_treasury.so` beside the war (`tests/publisher.rs`): the game swarm's tree and treasury founded, the share paid in and booked by Poleis's own `sync_endowment`, to the lamport (112,592,597) |
| on a local chain | his first war on solana-test-validator, the war alone | PASS (`rehearsal/receipts/war-rehearsal-2026-11-24T142205Z.json`; solana-test-validator 4.2.0, Poleis's `poleis_treasury` and the war at genesis): 184 transactions in 621.5 s, 151 minutes at 4 s, none missed; the crow beat its line at 148 and won at 150; 40 fighters, 10 deserters, 10 converts, 0 differences from his model. Paid: the crow's creator 117,786,929 lamports and the wolf's 59,259,260, exactly the law's; the game swarm's treasury 560,646,291, booked by Poleis's own `sync_endowment` to the lamport: every lamport of the war's fees reached the creators and the publisher (737,692,480 of 737,692,480; section 36's measure, 0 on the first devnet war). Units on the chain: pay_creator 2,189, pay_publisher 2,623, Poleis's sync_endowment 1,962, settle 40,324 |
| tests (the new Mac, nice 19) | poleis-war 18, poleis_wars 19 | poleis-war 18, poleis_wars 22 (refusals 14, publisher 1, wars 2, brought 3, cost 1, the id 1) |
| JavaScript | | none |
| at rest | | nothing runs; no new store (the rehearsal's only file on disk stays the devnet run's seed, already in `embertide-at-rest/stores.json`) |

## What stands now

- `src/ix/pay.rs`: `pay_creator` (IX 12, accounts `[house (w), creator (w)]`) and `pay_publisher` (IX 13, `[config, fees (w), treasury (w)]`). No
  signature asked; the account named must be the house's creator, the generation's treasury. A rekt house still pays its creator. A creator
  whose wallet stands empty is paid once what he is owed reaches a wallet's rent, or once he puts SOL in it; no trade waits on it.
- THE BOOK: a house holds its rent, its reserve, its chest and `creator_owed`; the fee account its rent and protocol + taxed + donated - paid.
  The loser of a war keeps what it owes its own creator; the settlement takes only its reserve and chest.
- Layout version 2 for every account. The first devnet war's version-1 accounts are refused by this program, and what they hold stays where it is.
- The rehearsal (`rehearsal/`, `rehearse-on-a-local-validator.sh`): Poleis's own program at genesis beside the war, the game swarm's tree (any
  root but 0, which only Poleis's deployer founds) and its unit's treasury founded first, each house launched by a creator of its own, and at the
  end both creators and the publisher paid, each held to the law's fees, and Poleis's books trued by its own `sync_endowment`.

## Found in review, mended here

- **A gift could have stopped the publisher's pay for good.** The builder's `pay_publisher` refused whenever the fee account's lamports differed
  from its book, and anyone may send lamports to any address by a plain system transfer: one lamport would have stopped every payout of that
  generation. Now what came by no trade is booked as `donated` and paid on with the fees, as Poleis's own `sync_endowment` trues a treasury; a fee
  account holding less than its book pays nothing, and a transfer of the difference mends it. `tests/refusals.rs`
  `a_gift_to_the_fee_account_is_paid_on_to_the_publisher` holds both, by the system program's own transfer.
- **The rehearsal read the publisher's pay one commitment behind.** Its first local run (`rehearsal/receipts/war-rehearsal-2026-11-24T135904Z.json`,
  kept red) played his war equal to his model in every number and paid both creators exactly, then said "the publisher got 0" while Poleis's own
  `sync_endowment` had booked all 560,646,291 lamports. On a local validator a landing waits for "processed" and the war's checks read there; the
  payment checks read at "confirmed", so a read right after a landing could see the treasury as it stood before. They now read where the war's
  checks read (`at`); on devnet that is "confirmed" already.
- **The rehearsal read the payments' units before they were confirmed.** Its second local run (`war-rehearsal-2026-11-24T141040Z.json`, kept red)
  ended on "Poleis: sync_endowment: no meta read": a transaction's units are read from its meta once it is confirmed, the war's landings are all
  confirmed before its read-back, and the payments land after it, so the last of them could still be only processed. The first run had won that
  race. Every payment is now confirmed before the units are read.

## On devnet (lane/the-game-pays-its-swarm-on-devnet, Nov 24 2026 14:39 UTC)

- His **14:38:40** "Yes, send it", to "Send 0.05 test SOL from your devnet wallet for the devnet upgrade?": the deploy key's 0.75 was short of this
  build's buffer (0.7313 at devnet's rent) and the program account's growth, and devnet's RPC airdrop answers 410 (its web faucet asks a sign-in).
- Upgraded in 17 s, in slot 525,822,910, the writes over the validators' own ports: `6ZA4DVYtEEDZF4nTjcaxzeWLxWUs7Mrve9EANzjRs4Ef` runs this build.
  Read back with `solana program dump`: its first 143,792 bytes are the build (sha256 `76002ff558c33f5a…`, the green local run's), the rest 6,560
  zero bytes. The deploy tool grew the program's account by 10,240 bytes, not the 3,680 the build needed.
- His test SOL: that growth's rent (0.0520) and 0.0008 of fees stay with the chain; everything else the deploy key held (0.7472) went back to his
  devnet wallet, 72.551 after it, and the key holds 0. It stays the program's upgrade authority, on the new Mac only, never in git.
- The first devnet war's version-1 accounts stand as they were, refused by this program; no version-2 war has run on devnet yet.

## Owed

- Done Nov 24 2026 17:10 to 17:58 UTC (lane/the-publisher-stands-on-devnet, `poleis/ceremony/THE-PUBLISHER-STANDS-ON-DEVNET-LANDING.md`): the business is a swarm on
  devnet's Poleis and a devnet generation of the war named its treasury and paid it 0.888902 SOL. What this line asked, as it stood: the publisher
  founded as a well on devnet's Poleis, so a devnet generation names its real treasury. Poleis stands on devnet (read Nov 24 2026 15:1x UTC:
  Swarmbook's `found.html` founds on `AJNrRxyvysf7iokrDLBHeoJQUqitiBva3nRUnCj7VEkJ`, and the source's `AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab` stands too, both
  executable); until then this line asked for Poleis on devnet as if it were not there; the swarm's seal founding a generation by CPI from its sealed
  proposal (here a stand-in key signs).
- Players signing their own moves from their own devices: the rehearsal signs for its stand-ins.

## The gates

- At the ship (THE CRATES GATE THE SHIP): `poleis-war`, `poleis/program/poleis_wars`, its `rehearsal`.
- `tools/embertide-mesh/tests/the-law-is-the-gate.test.mjs`: green on this lane, 8 of 8.
- No gate yet on the chain: the measure it waits for is the program's bytes on devnet against main's build, and the rehearsal's receipt held to
  the reference with every payment equal to the law's, read at the ship; today both are read by hand.
