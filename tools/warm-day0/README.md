# warm-day0: which of the fourteen day 0 claims hold for a mint (Oct 13 2026)

His Oct 13 04:22 WEST: "launch the fucking token right away from day 0 now... reverberate open transparent markets from day0". The
plan page (`docs/open-transparent-markets-from-day-0-2026-10-13.md`) turns "open" and "transparent" into fourteen claims a stranger
can check (its section 6) and names this tool in section 11: "a checker that runs the fourteen rows of §6 against any mint address
and prints which hold (it is the gate for flipping §IV)". A line on a page about the token that trades in the open is true only when
its row checks here.

**Which token that is moved on Oct 14 18:59 WEST** (`docs/phase-one-lyssajudron-warm-pool-hypertrace-2026-10-14.md`): "$WARM uncapped and
GTA-like in-currency minted for free with AI credits", and "$POOL is capped and is the governance token of the swarm". Which of the two
trades in the open from day 0 is his call (that page's call 4). The fourteen rows describe a capped token in one open pool, which is now
$POOL's shape. Run on an uncapped $WARM, row 2 fails by design (the authority that mints for AI credits has to stay) and row 5 cannot hold
once it is minted to its users; the claims an uncapped currency would carry instead (a public minting rule, the central bank's account
and what it does with what it takes) are not built here, because none of them is ruled.

Neither token is minted and none of their numbers is ruled, so the tool was built and proven against other people's tokens on mainnet.

**Read-only.** It asks a cluster for accounts and transactions through ten `get...` methods and refuses every other one before
anything is sent (`lib/rpc.mjs`). It makes and loads no key, builds, simulates and sends no transaction, mints nothing, and writes
no file unless `--record` names one. A test reads the tool's own source to hold all of that.

## Running

    node check.mjs <mint> [--cluster mainnet|devnet] [--manifest file.json] [--domain example.com] [--json]

    --rpc url         another endpoint for the same cluster (or WARM_DAY0_RPC); its genesis hash is checked
    --history n       how many transactions back to read per address in rows 11, 12 and 13 (default 200)
    --require 2,3,4   exit 1 unless these rows pass ("not checkable" does not fail a run by itself)
    --digests file    burn-digest's record (state/digests.jsonl), to compare with the lines on chain (row 13)
    --record file     also write every answer the cluster gave (a fixture)
    --replay file     answer from a recorded file: no network at all

It prints a fourteen-row table (verdict, claim, the account the verdict was read from), then each row's reasons and notes, then the
programs the rows stood on and whether any was redeployed since its layout was read. `--json` prints the same as JSON, every
amount a string. **Exit 0**: no decidable row failed, and every `--require` row passed. **1**: a row failed. **2**: the run could
not be made (bad arguments, not a mint, wrong cluster, the cluster did not answer).

A verdict is `pass`, `fail` or `not checkable`. A row never passes on a promise: where the chain cannot say, the row says what a
person looks at. For a capped token's day 0 gate, `--require 2,3,4,5,7,8,9,10,11,12` is the run that must exit 0 before §IV changes.

Node 18 or newer (written and run on 26.7). `@solana/web3.js` is loaded the way the house's one other Node tool loads it
(`embertide-app/ui/intro/dyson-society/tools/open-whitelist-sale.mjs`): borrowed from the Poleis client's `node_modules`
(`POLEIS_JS`, default `~/code-ec/poleis-protocol/js`), which holds 1.98.4 on this Mac; the pages pin 1.95.8 from the CDN, and
`package.json` pins that for a clone that has to `npm install` here instead. Only `PublicKey` is taken from it (base58 and derived
addresses).

## The fourteen rows

| # | The claim | Decided from | Passes when | The verdict's source |
|---|---|---|---|---|
| 1 | One canonical mint | DNS, with `--domain` | never by itself: a TXT record that names the mint leaves the row for a person; a domain whose records do not name it **fails** | `dns:TXT <domain>` |
| 2 | Fixed supply | chain | mint authority is none | the mint |
| 3 | No holder can be frozen | chain | freeze authority is none; on Token-2022 also no pause authority, permanent delegate, transfer hook, frozen-by-default or non-transferable extension | the mint |
| 4 | The name cannot be swapped | chain | the Metaplex metadata account has `isMutable` false (Token-2022 metadata in the mint: no update authority and a fixed pointer). No metadata at all fails | the metadata account, seeds `["metadata", program, mint]` |
| 5 | Every token is somewhere named | chain + manifest `accounts` | the labelled token accounts sum to the supply exactly. Every other holder is listed, largest first | the mint and each labelled account |
| 6 | No hidden tranche | not checkable | | his word |
| 7 | Locks are programs, not promises | chain + manifest `locks` | each lock is a Jupiter Lock escrow or a Streamflow stream of this mint, not cancelled, with no cancel authority, no recipient-change authority and (Streamflow) no update right, and its token account holds what is still owed. Dates are printed | each lock account |
| 8 | One open pool from the first slot | chain + manifest `pool` | a Meteora DAMM v2 pool holding this mint, enabled, past its activation point, with no whitelisted vault. Other DAMM v2 pools for the mint are counted in a note | the pool |
| 9 | The liquidity cannot be pulled | chain + manifest `pool.positions` | every named position has no free and no vesting liquidity (all of it permanently locked). With no position named: passes only when the whole pool is locked, fails when none is | the pool and each position |
| 10 | No prize for speed | chain + manifest `pool` | the base fee is a scheduler that starts above where it ends. The schedule is printed: start, end, steps, when it ends, the fee now, the protocol's cut. A flat fee or a rate limiter fails | the pool |
| 11 | The house's wallets are named | chain + manifest `houseWallets`, `schedule` | no transaction moved this token out of a house wallet beyond the published schedule (to another owner, by handing the token account over, or by a burn) | each house wallet |
| 12 | The fee is read, not reported | chain + manifest `fund`, `pool.positions` | each named position's NFT is held by the fund, no delegate may claim to any account, and every claim made so far was paid into accounts the fund owns (and not emptied elsewhere in the same transaction) | the pool, each position, the fund |
| 13 | The spend is read too | chain + manifest `burnDigest` | the address has signed one `burn-digest/1` memo line for every closed UTC day since its first, none contradicted. With `--digests`, each line is also the record's | the anchoring address |
| 14 | One live page | not checkable | | a page |

Rows 6 and 14 always print one line on what a person checks; so do rows 1 and 13 when they cannot decide.

## The manifest

`manifest.example.json` shows every field. Public addresses only. It is read strictly: an unknown field or a malformed address
stops the run, because a typo would otherwise turn a row into "not checkable" without anyone noticing.

- `mint`: optional; the run stops if it is not the mint on the command line.
- `accounts`: `{ label, address }` for every token account that holds the token (row 5).
- `pool`: `{ address, positions: [...] }`, the DAMM v2 pool and the house's position accounts (rows 8, 9, 10, 12).
- `locks`: `{ label, address }`, each a Jupiter Lock `VestingEscrow` or a Streamflow stream (row 7).
- `fund`: `{ address }`, the wallet the fee is said to reach (row 12).
- `houseWallets`: `{ label, address }` (row 11), and `schedule`: `{ label, from, to, amount, notBefore, notAfter }` with `from` a
  house wallet, `to` a token account, an owner or `"burn"`, `amount` in base units as a string, the times in unix seconds.
- `burnDigest`: `{ address }`, the address that signs the daily digest memo (row 13).

## Cost

Measured on the public endpoint (`api.mainnet-beta.solana.com`), one call at a time, 150 ms apart:

| run | read-only RPC calls | wall |
|---|---|---|
| a mint alone (rows 2, 3, 4) | 7 | 0.8 to 1.1 s |
| a mint and one lock | 10 | 1.3 s |
| a mint, a lock, a pool with one position, a fund (`--history 4`) | 21 | 4.4 s |
| everything: a table, a pool, a position, a fund, one house wallet (`--history 6`, 147 holders) | 35 | 27 s, most of it the endpoint's rate limit on `getTransaction` |

Nothing runs at rest: no service, no timer, no poll. Rows 11, 12 and 13 cost one `getTransaction` per transaction they read, so a
busy wallet is the expensive case; `--history` bounds it and a row that hit the bound says so instead of passing.

## Proof

- **Live**, Oct 13 2026: `node check.mjs BoY2UJq7VJmyYzgszK9bzo6MFqQB4u5NEMhRupiDwJp4 --manifest <its manifest> --history 6`
  prints all fourteen rows on mainnet: 5 pass, 4 fail, 5 not checkable, exit 1. The failures are true of that token and anyone can
  verify them: its metadata is mutable; 146 unlabelled holders; its pool's creator sent tokens to sixteen accounts in five
  transactions 25 seconds after the pool opened; its one fee claim (467.7 SOL) was paid into the holder's account and emptied to
  another wallet in the same transaction.
- **Offline**: `npm test` (or `node --test --test-concurrency=1 test/*.test.mjs`), 55 tests, about 1.5 s, no network: `fetch`
  throws for the whole run and a call a fixture does not hold throws instead of turning a row quiet.
- **Fixtures** (`fixtures/`, 479 KB), each one run recorded once from mainnet on Oct 13 2026 with `--record`: `wif` (both
  authorities none, immutable name: rows 2, 3, 4 pass, exit 0), `usdc-stream` (a live mint authority and freeze authority; a
  Streamflow stream its sender can cancel), `pyusd` (Token-2022, eight extensions, a domain that does not name it), `arena` (a
  sound Jupiter Lock; a pool locked for good only in part; a linear fee schedule), `cndl-lock` (a Jupiter Lock its creator can
  cancel), `nest-streams` (two sound Streamflow streams), `cudis-pool` (the pool Meteora's own layout test reads), `sra` (the full
  run above). `cluster-parse.json` is the cluster's own parse of six of those accounts.
- **Held to independent readings**: every decoded mint, token account and Token-2022 extension equals the cluster's own parse;
  every program-data header equals what the cluster reported; Meteora's reference pool decodes to the five numbers Meteora's own
  test asserts (cliff 500000000, mode 1, 120 periods of 60, reduction 265); the computed layout sizes equal the ones Meteora's
  source asserts (Pool 1104, Position 400 and six more); the Streamflow layout lands on the five offsets its SDK states; all 721
  token accounts of a real mint sum to the supply its mint states; the position address derived from the IDL's seeds is the one on
  chain.
- **The tests bite**: 40 one-line breakages of the tool (a rule switched off, an offset moved by one, the allowlist widened, the
  memo signer check removed) were each caught by at least one test, in one pass over the final code.
- States no recording could show (a disabled pool, a flat fee, a stream that can be updated, a look-alike memo) are tested on a real
  account's bytes with one field rewritten at the offset the IDL gives; each such test says it is made up.

## The layouts, where each was read, and when

No offset of an Anchor account is typed by hand: `lib/layout.mjs` computes them from the IDL at load time and refuses any type whose
size is not fixed. `idl/sources.json` holds every source, commit and date; in short, all read on **2026-10-13**:

| Program | What is read | Read from | Version | On mainnet that day |
|---|---|---|---|---|
| SPL Token `Tokenkeg...` | Mint, Account | `solana-program/token` `interface/src/state.rs` @ 8185db1 (2026-09-21) | spl-token-interface 3.0.0 | deployed slot 419472000, no upgrade authority |
| Token-2022 `Tokenz...` | the same bases, the extension framing and nine extensions | `solana-program/token-2022` `interface/src/extension` @ 6a72b16 (2026-10-12); `solana-program/token-metadata` @ 9f54923 | spl-token-2022-interface 3.1.2 | slot 427147035, upgradeable |
| Metaplex Token Metadata `metaq...` | Metadata, its address | `mpl-token-metadata` `idls/token_metadata.json`, `pda.rs` @ 353d01b (2026-09-15) | IDL 1.14.0 | slot 380725176, no upgrade authority |
| Jupiter Lock `LocpQ...` | VestingEscrow | the IDL the program holds on chain (`idl/locker.json`, unchanged); mode values from `jup-ag/lock-sdk` @ e8a407a | IDL 0.4.0 | slot 368684289, upgradeable |
| Streamflow `strmR...` | Contract | `streamflow-finance/rust-sdk` `state.rs` @ 63df7d7 and `js-sdk` `layout.ts` @ fb236ff (both 2026-08-31), transcribed as `idl/streamflow-contract.json` | Rust SDK 0.15.0, JS SDK 13.4.0 | slot 449063805 (2026-09-21), upgradeable |
| Meteora DAMM v2 `cpamd...` | Pool, Position, base-fee layouts, `claim_position_fee`, its event | `MeteoraAg/damm-v2-sdk` `src/idl/cp_amm.json` @ 8c0f726 (`idl/cp_amm.json`, unchanged); enums and rules from `MeteoraAg/damm-v2` @ a85c926 | cp_amm 0.2.4 | slot 445230614 (2026-09-08), upgradeable |
| BPF Upgradeable Loader | the program-data header | `anza-xyz/solana-sdk` `loader-v3-interface/src/state.rs` @ 151e056 | 9.0.0 | |

**When a program moves.** Every run reads the 45-byte header of each program it stood on. If its deploy slot is later than the one
in `idl/sources.json`, the rows that read it carry a note. Then: read the program's layout again at its source, update the vendored
IDL (keep it byte for byte, plus one final newline) and `sources.json`, re-record the fixtures, and run the tests.

## What reading the programs turned up (for the plan page, not fixed here)

1. **Meteora's operators can disable any DAMM v2 pool.** `set_pool_status` takes any pool and a signer holding the `SetPoolStatus`
   permission. A disabled pool refuses every swap. The plan's "the pool never closes... a holder can always sell" (section 5) rests
   on Meteora not using it. Row 8 says so on every run.
2. **Meteora's operators can change a pool's fee.** `update_pool_fees` sets a new base fee once the schedule has ended, and the
   dynamic fee at any time. "The pool's fee, set once at creation" (section 3, hop 1) is true of the pool's creator, not of
   Meteora. Row 10 says so. Meteora also takes a share of every fee (20% in every pool read here), which the fees page will have
   to state beside the pool's fee.
3. **A fee claim lands where the claimer points it.** The program lets a position's holder (or a delegate the holder allowed)
   name any destination. Row 12 can read who may claim and where each past claim went, never the next one. The fund holding the
   position's NFT in its own named address is what makes the line true.
4. **Jupiter Lock's source could not be read.** The repository Jupiter's developer page names (`github.com/jup-ag/jup-lock`)
   answered 404 on Oct 13 2026. The layout here is the IDL the program publishes on chain, and "set once and cannot be changed" is
   Jupiter's page's word. The program is upgradeable.
5. **A Streamflow stream made with `can_update_rate` has no fixed dates and no fixed flags**: its developer page says the cancel
   and transfer flags "can be toggled later via update()" on such a stream. Row 7 fails it. Streamflow's program source is not
   public and it was redeployed on Sep 21 2026, after the newest public SDK commit.
6. **The public mainnet endpoint serves no `getTokenLargestAccounts`** (its limit for that method is 0). Row 5 reads every
   holder through the token program instead, which is one call and exact for a new token, and heavy for one with many holders.

## What remains unchecked

- **Rows 6 and 14**, and the page, repository and post halves of **row 1**: a person's.
- **Row 13's truth.** The chain shows that a line was published each day and by whom. Whether the hash is that day's receipts is
  recomputed from the burn book (`tools/burn-digest`, `node digest.mjs verify`), on the Mac that holds the book. Nothing is anchored
  yet, so the row was tested only on made-up memo lines in the sibling tool's own format; that format may still change before its
  first anchor (its hand-over: two books, an `unpriced` field).
- **Row 5 after the first swap.** Buyers hold tokens outside any table, so the row fails from then on: it is a check for before the
  pool trades.
- **Row 8's "one".** Anyone can open another pool for the mint; they are counted, not judged. Only DAMM v2 pools are counted.
- **History.** Rows 11, 12 and 13 read the newest `--history` transactions of each address; a row that hit the bound says so
  instead of passing. A token account a house wallet emptied and closed is seen only through the wallet's own transactions, and
  a wallet the manifest does not name is not seen at all.
- **What a program may do tomorrow.** Jupiter Lock, Streamflow, Meteora DAMM v2 and Token-2022 are upgradeable: every run prints
  who may upgrade each, and nothing more can be read.
- **Not read at all:** Streamflow's price-aligned unlocks (refused by name), locks on any other program, pools on any other AMM,
  Token-2022's confidential-transfer extensions, and the file a metadata uri points at.
- **An uncapped token.** See the top of this page: rows 2 and 5 are claims about a fixed supply, and nothing here reads a minting rule.
- **The seat NFT.** The Harmony mint being prepared on Oct 13 (Token-2022 seats through the Poleis program, the Lyssajudron NFT) is another instrument with
  its own diagnostic (`dyson-society/tools/check-nestos-mint.mjs`). These fourteen rows are about the fungible token; run on a
  seat's mint they would truthfully fail row 3 for its transfer hook.
- **Devnet** is accepted (`--cluster devnet`, Streamflow's devnet program is known) and no run was recorded there.

## Files

`check.mjs` (the command; `main(argv, io)` for tests), `lib/rpc.mjs` (the ten allowed methods; live, fixture and recording
transports), `lib/layout.mjs` (layouts and derived addresses from an IDL), `lib/borsh.mjs`, `lib/web3.mjs` (PublicKey only),
`lib/programs.mjs`, `lib/spl.mjs`, `lib/metaplex.mjs`, `lib/locks.mjs`, `lib/damm-v2.mjs`, `lib/rows.mjs` (the fourteen rows),
`lib/manifest.mjs`, `lib/render.mjs`, `idl/` (two vendored IDLs, the Streamflow transcription, `sources.json`), `fixtures/`,
`test/` (six files), `manifest.example.json`, `package.json`.
