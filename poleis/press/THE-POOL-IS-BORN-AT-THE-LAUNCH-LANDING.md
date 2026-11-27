# THE POOL IS BORN AT THE LAUNCH: the swarm's SPL stake pool born at the mainnet launch by root 0's own sealed raw act, its account and its list made in the execute's own transaction (Nov 22 2026, lane/the-pool-is-born-at-the-launch, built for seat claude-901e44a8 on the launch press of claude-9997bc17)

## His words, UTC

- **Nov 22 2026 01:18:14**, to "Add the staking pool's birth to your mainnet launch?": "Yes, add it".

Why it was asked: a society's page on Swarmbook promises "Keep your SOL. Burn its light.", and the launch press (`poleis/press`) had no
step that births a stake pool, so at the launch nothing would have stood behind the promise.

The road it follows, in his earlier words (their receipts beside this lane): Oct 29 2026 20:34:07, asked which key manages the mainnet pool,
"you gotta go back to the design of our governance dude... fucking hell the pentagon and hexagon voting shit dude please"
(`tools/embertide-mesh/pool/THE-POOL-WAITS-ON-HIS-KEY-LANDING.md`: the pool is the polis's, not a key's); 21:11:11 "Yes, widen them" and
21:26:27 "Yes, birth it" (`poleis/THE-HANDS-WIDEN-LANDING.md`: devnet's pool born by unit 0's seal, its certificate
`poleis/ceremony/pool-birth-certificate-devnet-u0a.json`).

An addition of the seat's, Nov 22 01:3x UTC: the stake pool program is upgradeable (read on mainnet then: last deployed at slot
469,669,929, about twelve days before, by a multisig), so a rehearsal proves the birth on the bytes it copied and no others.

## Cost first

| | before (the launch press at 587766e08f) | after (this lane) |
|---|---|---|
| the stake pool at the launch | none: no step births it | born by act 5, root 0's raw act, before the handoff; on mainnet with no way around it |
| its five accounts' rent, at mainnet's 5,080 lamports a byte (each account's 128 bytes of overhead counted; devnet's reserve held 1,666,240 for its 200) | | the pool 611 bytes: 3,754,120; its list for 10 validators, 739 bytes: 4,404,360 (370,840 more a validator); the reserve 200 bytes: 1,666,240; the mint 82 bytes: 1,066,800; root 0's vault of the mint 165 bytes: 1,488,440. The five: **12,379,960 lamports (0.01237996 SOL)** for 10 validators, all of it held by the accounts |
| what it adds to `need()`, the deployer's | | made ahead (press pool): 4,221,480 and one transaction's fee, 5,000. The birth, where the stake pool program stands: the pool and its list 8,158,480, unit 1's convened session of act 5 1,762,760 (home at adjourn), and the act's crank 55,000 (seven transactions and four votes' signatures). In all **14,202,720 lamports (0.0142 SOL)** with 10 validators, measured by the need test on an RPC of its own at 5,080 a byte; nothing more once they stand |
| what it costs his Keystones | | Keystone 1 proposes act 5 as he proposes each of the four: one proposal account (one size, whatever its payload) and root 0's session of it, and its fee (the launch press's receipt counts about 0.038 SOL at mainnet's rent for the four; act 5 is not measured on a chain); Keystones 2 and 3 sign envelopes only |
| act 5's execute on the wire | | **869 bytes of 1,232** with its two makings (885 with a stranger's lamports at both addresses; 921 and 937 with a priority fee's two compute budget instructions). Devnet's execute of the same Initialize, with a compute limit and no makings, was 673 bytes |
| Keystone 1's proposal of it | | **809 bytes** (devnet's proposal of the same payload: 809) |
| its compute | | not measured: no chain this lane ran holds the stake pool program (that rehearsal waits on his word). Devnet's execute of the same Initialize took 21,746 CU; the two makings are two more instructions of the system program |
| code | | 1,547 lines of Rust added and 53 taken out (the module `src/pool.rs` 596, `tests/pool.rs` 614); 30 lines of the rehearsal's shell; 11 of the params. No JavaScript; no line naming a classical signature |
| tests (the new Mac, at 188e7ed803 and again at 3f07bf98d8, the code's last) | the press 44 | **the press 57**: the lib 8, crank 5, finger 3, floor 4, metaplex 4, params 4, **pool 13**, prepared 3, root 6, wire 7 |

## What it is

**`poleis/press/src/pool.rs`**, a new module of the press, and act 5 of its acts (`acts.rs`).

- **The pool, every address derived.** Its base is the deployer, the key that makes its seeded accounts; its seeds are the params' tag and
  one letter (`<tag>p` the pool, `<tag>l` its list, `<tag>r` its reserve, `<tag>m` its mint: birth-pool.mjs's shape); its withdraw and
  deposit authorities are PDAs of the pool's address under the stake pool program; its manager and staker are the polis's hand
  (`agent_pda(0)`); its fee account is root 0's vault of the mint (`vault_pda(0, mint)`), whose authority is root 0's treasury. Mainnet's
  stake pool program: `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` (devnet's birth ran another deployment of it).
- **Act 5, the pool's birth,** passed like the other four: Keystone 1 proposes it (its set's one transaction), the crank convenes unit 1 (an
  ACT_RAW's proposal opens root 0 alone), two of unit 1's three agents vote, unit 1's seal is its chair's vote in root 0 (`mandate`), two of
  his three Keystones vote on the floor, root 0 seals, the crank executes. Its payload is the stake pool program's Initialize as devnet's:
  nine accounts (the pool and the list writable; the polis's hand signing as the manager and named again as the staker; the withdraw
  authority; the reserve; the mint and the fee account writable; the token program) and the data
  `[0][the epoch fee: denominator 1, numerator 1][withdrawal 0/0][deposit 0/0][referral 0][max validators: u32]`. No deposit authority is
  named, so anyone may stake. Act 5 executes only after the four before it.
- **`press pool`, the first sitting's part** (in `press sit` after the market, before the floor): the reserve, the mint and root 0's vault of
  the mint in one transaction the deployer pays, each read back and held (`reserve_held`, `mint_held`, the press's own `vault_held`), what
  stands skipped and said in the record's resumes. The pool's account and its list are never made here.
- **The execute's own transaction** (`Act::before`, `Act::execute_ixs`, used by press crank's Execute): the pool's account (611 bytes) and its
  list (`9 + 73 × maxValidators` bytes), owner the stake pool program, at the chain's own rent, by `create_account_with_seed` whose base is
  the key that executes; then the execute. Where a stranger's lamports stand at either address, a top-up and `allocate_with_seed` (new in
  `loader.rs`) instead, as the deploy makes a program account a stranger sent dust to.
- **The read back,** after the execute and before the press says it stands (`pool_held`, `list_held`): account type 1; manager and staker
  `agent_pda(0)`; the program's default deposit authority; the list, the reserve and the mint the sitting's; the fee account
  `vault_pda(0, mint)`; SPL Token; the epoch fee 1/1 (birth-pool.mjs's offsets); the list of type 2 holding the params' max validators. Then
  press crank says "THE POOL IS BORN: ..." and the record keeps `pool.born`.
- **The handoff waits for it** (`handoff_ready`, its fifth condition, after root 0's council, the fund, the mine and the board): the pool's
  making needs the deployer's key, and the handoff's last words tell him to delete that key. Only on a rehearsal's chain with no stake pool
  program does the handoff not wait (the pool cannot be born there); on mainnet the program stands, or the press says "this is no mainnet".
- **`need()` counts it**, at the chain's own rent (the table above). What stands costs nothing more.
- **`params_checked` holds it** before anything starts: `pool.tag` (1 to 8 of a-z and 0-9), `pool.maxValidators` (1 to 143,640: the stake
  pool's Initialize refuses none, and a list of more is past the 10 MiB an account holds), `pool.programSha256` (named on mainnet), and act
  5's execute with its two makings one transaction, with nothing at their addresses and with a stranger's lamports there.
- **The certificate names it:** the five accounts' rent among the others, the pool's addresses and seeds, the stake pool program as recorded
  and as it stands, what was made ahead, the birth read back (or "not born yet", or "not run"), and its transactions.
- **Where no stake pool program stands** (the default rehearsal, and any run before his word): `press pool` still makes the three ahead (the
  stake program, SPL Token and Poleis stand on every chain), `press acts` writes no fifth set and says so, press crank says
  "ACT 5 (...): the pool's birth not run: no stake pool program on this chain: nothing of it is sent here", and the handoff does not wait.
  Never faked; on mainnet each of these refuses instead.
- **The params:** `params.rehearsal.json` names 10 validators and the tag `lp1`, and no hash (a rehearsal records what it reads).
  `params.mainnet.json` names each as a PLACEHOLDER, his to fill: the list's length, the tag, and the program's hash.

## The hole, and its cure

The stake pool's Initialize asks only the manager to sign, never the pool's own account. An empty pool account and list, owned by the
program and not initialized, are anyone's to initialize with a manager and a fee account of their own: they take the pool. On devnet the
crank made them 40 slots before the sealed execute (birth-pool.mjs's first step, `the pool's empty accounts, paid by the crank`); on mainnet
the act waits for the council for hours, so the same road would leave them open that long.

- **Ahead, useless to anyone else.** The reserve and the mint answer to the pool's withdraw authority, a PDA of the pool's own address, and
  the stake pool's Initialize holds the reserve's staker and withdrawer and the mint's authority to the PDA of the pool it initializes. A
  stranger's pool (any other address) has another, and is refused them. The pool's address is the deployer's `with_seed` address.
- **The pool and its list never stand empty.** They are made in the transaction that initializes them, before the execute, by the key that
  executes. Only a base makes an account at its `with_seed` address (the system program asks the base's signature), so a stranger who
  carries out the passed act makes its makings at its own addresses; the sealed payload still names the deployer's pool and list, where
  nothing stands; its Initialize fails, and with it the whole transaction. Until act 5's own transaction, nothing stands at those addresses
  for a stranger's Initialize to take.
- **Dust does not bar it.** Lamports sent to a seeded address make a system account there, over which a create is refused: the press tops
  the address up to its rent and gives it its space and owner by `allocate_with_seed`, signed by the base.

## The bytes it was rehearsed on

The sitting records the stake pool program's program data the first time it reads it (`poolProgram` in the record: the program data's
address, its last deploy's slot, its upgrade authority, and its executable bytes, those after the program data's 45-byte header with the
zeros of its room trimmed from their end, the way a verified build's hash is read: their length and their sha256), at `press deploy`, `press
pool` or press crank's execute, whichever reads it first. Every later read is held to that record and to the params' `pool.programSha256`:
other bytes, and the deploy, the pool step and act 5's execute refuse with a plain sentence, on every chain (asked for mainnet; a rehearsal's
copy never moves, so holding it there too costs nothing) ("the stake pool program ... runs other bytes than
the sitting recorded: ... It was upgraded since the sitting first read it: nothing of the pool is sent ..."). Only the bytes are held: a
rehearsal's copy stands at the slot its validator gave it, mainnet's at its own. On mainnet the params must name the hash (a rehearsal's
certificate names it: `pool.programRecorded.sha256`), so the launch births the pool on rehearsed bytes or not at all.

## Tests (`tests/pool.rs`, 13, each with its own faults)

- `every_address_of_the_pool_is_devnets_and_solanas_own_derivation`: Pool::derive from devnet's deployer (the ceremony's), tag `u0a`,
  devnet's stake pool program and Poleis gives the seven addresses devnet's certificate names (the pool, the list, the reserve, the mint, the
  withdraw authority, the agent, the vault) and its seeds; the default deposit authority is the one devnet read back off the pool of Oct 29
  20:29 UTC; the sitting's four seeded addresses are Solana's own `create_with_seed`, its authorities Solana's own PDAs, the rest Poleis's
  client's; another base, another pool.
- `the_initialize_is_the_payload_devnets_seal_passed_and_executed`: the Initialize's data and nine accounts are birth-pool.mjs's, and the
  payload the press builds for devnet's addresses hashes, with birth-pool.mjs's manifest and nonce, to devnet's own proposal
  `38cafee3415552878363807b5927143139e025dae192490788c6ed3dbe1c890d`, the one its seal passed and executed.
- `act_5_is_the_pools_birth_by_the_polis_hand`: the five acts in order; act 5 an ACT_RAW of root 0, convened, the fifth nonce, its makings
  the pool's 611 bytes and the list's 739, the stake pool program its need; the four before it make nothing first.
- `its_execute_makes_the_pool_and_its_list_first_by_the_key_that_executes`: the makings held to Solana's system interface
  (solana-system-interface 3.2.0), then the execute, the deployer its crank; over a stranger's lamports a transfer of the shortfall and
  `allocate_with_seed`; over more than the rent, no transfer; over anything else, refused. Where the base pays, the interface leaves the base
  out of create_account_with_seed's accounts and the press's builder names it again, a signer (the form the deploy makes its named buffers in
  on every rehearsal's chain); the data and every other account are the interface's.
- `a_strangers_execute_makes_its_accounts_at_its_own_addresses_and_fails_whole`: a stranger's makings stand at the stranger's own addresses,
  never the payload's; what executes still names the deployer's pool and list; and any transaction that makes the deployer's pool, the
  stranger paying or not, asks the deployer's signature.
- `the_first_sitting_makes_no_pool_account_so_a_strangers_initialize_finds_none`: press pool's transaction makes the reserve (held to the
  interface, its rent exactly, then the stake program's Initialize as devnet's), the mint (InitializeMint2 as devnet's) and the vault, names
  neither the pool's address nor the list's, and writes nothing but the deployer, the reserve, the mint and the vault; the reserve and the mint
  refuse a stranger's pool; what stands is skipped; dust at the reserve's address is made over; one standing for another authority stops it.
- `the_pool_is_held_as_root_0s_before_the_press_says_it_stands`: the read back holds devnet's read; eleven faults refused by their own names
  (another manager, staker, deposit authority, list, reserve, mint, fee account or token program; an epoch fee of 0/1 or 1/2; no pool yet);
  another program's account and another length refused; the list's and the reserve's and the mint's faults refused.
- `the_handoff_waits_for_the_pool_on_mainnets_rules`: born passes; unborn waits on every chain; no program refuses on mainnet and passes only
  on a rehearsal's chain; the order before it stands.
- `the_birth_runs_on_the_bytes_it_was_rehearsed_on`: the executable bytes and their sha256 read from a program data laid by the loader's own
  interface; the same bytes copied at another slot and with another room pass; a moved byte, or one added, at a later slot is refused with both
  hashes said; the params' hash refuses other bytes even at the first read; the record reads back as written.
- `the_pools_params_are_held_before_anything_starts`: the edges pass (a tag of 1 and of 8, 1 validator and 143,640); a tag empty, with a
  capital, of nine or with a dash, 0 or 143,641 validators or a word, no hash or a short one on mainnet, a placeholder hash, and no pool
  section at all are each refused by their own words; on a rehearsal's chain the hash may be left out.
- `the_execute_and_its_two_makings_are_one_transaction`: the sizes above, each under 1,232.
- `act_5s_set_is_written_only_where_the_pool_can_be_born`: four sets where no stake pool program stands, five where it does, the fifth with
  the pool's addresses and its makings at the deployer's own addresses.
- `the_need_counts_the_pool_at_the_chains_rent`: `need()` itself, on a JSON-RPC of the test's own on 127.0.0.1 at 5,080 lamports a byte: with
  no stake pool program, the three ahead counted and the birth "not run" and not counted; where it stands, the birth's line and its crank's
  fees exactly; with everything standing, nothing more.

The press's other suites learn the fifth act only where a shared fixture must: a pool section in the params of `tests/crank.rs`,
`tests/params.rs` and `tests/prepared.rs`; the fifth set in `tests/prepared.rs`; act 6 as the ballots' unknown act in `tests/crank.rs`; and
`handoff_ready`'s two new arguments in `tests/root.rs`. The prepared vector (act 1's set) is unchanged.

## Faults put in on purpose

In the sources, one at a time, on the new Mac at 188e7ed803, each caught by the test named; **17 put in, 17 caught, none survived**:

| | the fault | caught by |
|---|---|---|
| 1 | the epoch fee 0/1 instead of 1/1 | the_initialize_is_the_payload_devnets_seal_passed_and_executed |
| 2 | the staker marked a signer | the_initialize_is_the_payload_devnets_seal_passed_and_executed |
| 3 | the withdraw authority's seed `withdrawal` | every_address_of_the_pool_is_devnets_and_solanas_own_derivation (and the proposal's hash) |
| 4 | the execute's transaction without its makings | its_execute_makes_the_pool_and_its_list_first_by_the_key_that_executes (and the stranger's and the size's) |
| 5 | the base no longer a signer of a seeded create | a_strangers_execute_makes_its_accounts_at_its_own_addresses_and_fails_whole (and the two interface comparisons) |
| 6 | no top-up over a stranger's lamports | its_execute_makes_the_pool_and_its_list_first_by_the_key_that_executes (and the first sitting's and the size's) |
| 7 | the first sitting makes the pool's account (devnet's road) | the_first_sitting_makes_no_pool_account_so_a_strangers_initialize_finds_none |
| 8 | the need forgets the birth | the_need_counts_the_pool_at_the_chains_rent |
| 9 | the handoff passes a mainnet with no stake pool program | the_handoff_waits_for_the_pool_on_mainnets_rules |
| 10 | the handoff passes an unborn pool | the_handoff_waits_for_the_pool_on_mainnets_rules |
| 11 | the program held by its deploy slot, not its bytes | the_birth_runs_on_the_bytes_it_was_rehearsed_on |
| 12 | the program's hash read over its zero room too | the_birth_runs_on_the_bytes_it_was_rehearsed_on |
| 13 | mainnet's params without the rehearsed hash accepted | the_pools_params_are_held_before_anything_starts |
| 14 | the read back no longer holds the fee account | the_pool_is_held_as_root_0s_before_the_press_says_it_stands |
| 15 | the fifth set written where no stake pool program stands | act_5s_set_is_written_only_where_the_pool_can_be_born |
| 16 | a list of 0 validators accepted | the_pools_params_are_held_before_anything_starts |
| 17 | a tag with capitals accepted | the_pools_params_are_held_before_anything_starts |

## The gates, at 188e7ed803 and again at 3f07bf98d8 (the code's last)

- The press's whole `cargo test --offline --locked`: 57 passed, 0 failed (`taskpolicy -b nice -n 19`, the 26.5 SDK, the shared target).
- `tools/embertide-mesh/tests/the-javascript-only-shrinks.test.mjs`: 31 of 31; "this lane's JavaScript +0 -0, net +0 in 0 files".
- `tools/embertide-mesh/tests/the-house-signs-post-quantum-only.sh`: exit 0 (1,160 lines in 487 files against a baseline of 1,159 in 486: the
  one more is main's `space-router-portrait.test.mjs`, which this lane does not touch). This lane adds no line naming a classical signature:
  the press keeps its 2 marked lines, `council.rs` its 4, and nothing under `poleis/program` moved.

## The default rehearsal (no read of mainnet)

`poleis/ceremony/the-launch-press-rehearsal.sh` at 188e7ed803, on the new Mac, Nov 22 2026 02:28 to 02:57 UTC: every feature the validator
knows (284), nothing read beyond the machine, no Metaplex and no stake pool program at genesis; its own copy folder and its own ports
(18899, 19900, 20290, 20300-20400), so the launch seat's copy and validator were never touched. **Exit 0, 1,705 s in all:** the press sent
562 transactions (526 of the deploy, the founding, the vault, the mine, the board, the pool's 1, 2 of the ratification, 26 of the crank, 3 of
the handoff), the checks 105 and 7 refusals, and every fault of the press's own rehearsal was refused or mended as before. What it says about
the pool, in its own words:

- `press need` and `press deploy`: "the pool, made ahead (press pool): its reserve, its mint and root 0's vault of the mint" 0.005783760 SOL
  (at the local validator's 6,960 lamports a byte), and "the pool's birth not run: no stake pool program on this chain: not counted".
- `press pool`: one transaction of 804 bytes and 18,949 CU (slot 50), the deployer paying: the reserve (2,282,880 lamports, exactly its
  rent; its staker and withdrawer the pool's withdraw authority), the mint (nine decimals, no freeze authority) and root 0's vault of it (its
  authority root 0's treasury), each read back and held; then "the pool's birth not run: no stake pool program on this chain: its reserve,
  its mint and root 0's vault of it stand; the pool and its list are made only in act 5's execute, on a chain that has the program".
- `press acts`: four sets, and "act 5, the pool's birth: its set is not written: the pool's birth not run: no stake pool program on this
  chain". The checks signed the four (they assert four), so this run is not the pool's rehearsal.
- `press crank`: acts 1 to 4 executed in 26 transactions, then "ACT 5 (the stake pool's birth (its Initialize, by the polis's hand)): the
  pool's birth not run: no stake pool program on this chain: nothing of it is sent here"; run again, "nothing was due".
- the handoff, asked in genesis and asked before the fund, refused as before; asked last, "the pool's birth not run: no stake pool program on
  this chain: a rehearsal's chain, so the handoff does not wait for a pool it cannot bear (on mainnet it waits)", and the three programs went
  to the seal.
- the certificate's `pool`: the program `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`, recorded nothing and standing nowhere (no program on
  the chain), the tag `lp1`, 10 validators, the seeds, what was made ahead (its transaction above), and born: "the pool's birth not run: no
  stake pool program on this chain"; among the rent, the reserve 200 bytes, the mint 82 and the vault 165.

One line of the first sitting's last words read "THE POOL IS BORN before the handoff ..." on this chain too, though nothing was born; after
this run (main.rs, words only) it says "the pool's birth not run ...: the handoff does not wait for it on this chain" where no stake pool
program stands, and "THE POOL'S BIRTH comes before the handoff ..." where one does.

## What is not built

- **A validator added by the seal.** AddValidatorToPool takes 13 accounts and the staker's signature (the polis's hand), and fits ACT_RAW's
  16: a later act of root 0, with the reserve's SOL it moves.
- **The deposit door:** how a staker's SOL reaches the pool (its DepositSol, open to anyone) from the society's page.
- **The engine that places stake:** which validators, how much each, and when, and the permissionless updates each epoch.
- **The yield into the tanks:** the pool tokens the epoch fee mints into root 0's vault leave it only by a sealed TREASURY act; no act moves
  them anywhere yet.
- **$POOL's count:** who staked the most, which decides $POOL's unlock at year five (his Oct 29 19:12).
- **The rehearsal with the stake pool program waits on his word.** The only road for the program to stand on the rehearsal's chain is a third
  read of mainnet at the validator's genesis (`LAUNCH_POOL_FROM_MAINNET=1`, beside Metaplex's under `LAUNCH_FROM_MAINNET=1`), asked by seat
  claude-901e44a8 and not given; the script holds it behind its own switch and says so. It waits on one more thing too: the checks
  (`poleis/program/poleis_treasury/tests/council.rs`) sign four acts and assert four sets (`assert_eq!(files.len(), 4, "four acts")`); a run
  with the program writes five. Their act loop is the same for every set, so the change is that one line, in a file this lane does not
  touch; until it changes, the script stops before anything is read ("the checks sign four acts ...: not started, nothing read").
- **The execute's compute,** read on a chain that holds the stake pool program: with that rehearsal.

## Judged before the hand (seat claude-901e44a8, Nov 22 2026 03:0x UTC)

- Read whole: `src/pool.rs`, the crank's Execute arm, `loader::allocate_with_seed` (the system program's instruction 9, [account (w), base (s)]),
  act 5 in `acts.rs`, the handoff's wait and the need's count in `chain.rs`. The StakePool offsets read back are birth-pool.mjs's, which read devnet's
  pool of Oct 29.
- Main merged in after THE SWARM IS ITS OWN MULTISIG landed (23f74cb7b5; merged at 5ecbf88f22, clean: under `poleis/` main brought only that lane's
  receipt and two certificates). At the merge, 00d7142d7c, on the new Mac: the press's whole `cargo test --offline --locked` 57 passed, 0 failed (lib 8,
  crank 5, finger 3, floor 4, metaplex 4, params 4, pool 13, prepared 3, root 6, wire 7); `the-javascript-only-shrinks.test.mjs` 31 of 31, "+0 -0";
  `the-house-signs-post-quantum-only.sh` exit 0 (the one line over the baseline is main's `space-router-portrait.test.mjs`, a file this lane does not touch).
- Which program: his question "u sure you are copying the best?" (Nov 22 2026 01:3x UTC) was answered from a read of mainnet and of Sanctum's own docs:
  the SPL program holds JitoSOL's pool (10,137,622 SOL) and bSOL's (875,166 SOL); Sanctum's two programs are forks of its v1.0.0; each program's
  upgrade authority acts only through a multisig; Sanctum's Infinity holds tokens of SPL pools. The record: ec-workspace
  `docs/the-big-launch-hypertrace-2026-10-31.md`, section 8g.
- Owed before his sitting, in this order: his word on the rehearsal's copy of the stake pool program (asked); the live check's count of the acts in
  `poleis/program/poleis_treasury/tests/council.rs` (`assert_eq!(files.len(), 4, "four acts")`), which reads five sets once the program stands: a line
  of the rehearsal's lane; the rehearsal under mainnet's rules with the program; its certificate's sha256 written into `params.mainnet.json`'s
  `pool.programSha256` by his hand with the other values.

## THE LIVE CHECK COUNTS THE POOL (lane/the-live-check-counts-the-pool, seat claude-901e44a8, Nov 22 2026)

**The launch seat's word** (seat claude-9997bc17, 03:0x UTC, once its lane had landed): "You write that line, in the lane whose rehearsal has the stake
pool program. Since my lane landed, council.rs belongs to main... Make the count follow what the press itself wrote: five when the record shows the
pool's program stood and its set was written, four when it did not. Never a bare 5. Assert the pool's own set beside the count: root 0's agent as
manager and staker, and the fee account as vault_pda(0, its mint). council.rs stays at 4 lines naming ed25519."

**What changed** (`poleis/program/poleis_treasury/tests/council.rs`, the live check `the_launch_press_on_a_live_validator`):
- The count follows the record: the pool's set (`5-the-pool.json`) is written exactly where the sitting's record holds `poolProgram`, and the sets
  are four plus that one. A bare number is gone.
- Beside the count, the pool's set is held: its manager and its staker `agent_pda(0)`, its fee account `vault_pda(0, its mint)`.
- After `press crank`, the pool is read on the validator apart from the press: 611 bytes of account type 1, its manager and its staker root 0's
  agent, its fee account root 0's vault of its mint, its epoch fee 1/1.
- No line naming ed25519 added. The rehearsal script's comment now says the checks count what the press wrote; its guard against a bare four stands.

**Measured** (the new Mac, the default rehearsal at 79faf969fd, no read of mainnet, ports and copy of its own, nice 19, the Mac's load 40 to 98 at
its start): exit 0 in 1,699 s; `the_launch_press_on_a_live_validator ... ok` (1,607.5 s); the press's crank executed acts 1 to 4 and said of act 5
"the pool's birth not run: no stake pool program on this chain: nothing of it is sent here", and nothing was due on the second crank; the handoff
refused in genesis and before the fund, then said it does not wait for a pool this chain cannot bear; the firewall saw 0 inbound flows.

**Still owed:** the five's road runs for the first time in the rehearsal with the stake pool program, which waits for his word on the third read.

## THE POOL IS REHEARSED WITH ITS PROGRAM (seat claude-901e44a8, Nov 22 2026)

**His word:** "Yes, copy it", his tap on the Spool Board on the new Mac at 13:09 (his zone, WET = UTC), to "May the rehearsal copy the stake pool
program from mainnet, as it does Metaplex's?" (asked since 01:2x UTC). Before it, Nov 22 ~01:3x UTC, his "u sure you are copying the best?": the SPL
program holds JitoSOL's pool (10,137,622 SOL) and bSOL's (875,166 SOL), Sanctum's two programs are forks of its v1.0.0 (ec-workspace
`docs/the-big-launch-hypertrace-2026-10-31.md`, 8g).

**The run:** `poleis/ceremony/the-launch-press-rehearsal.sh` at main 4c071d49b1 (THE WHOLE TREE DECIDES on main since 991aaf613a) on the new Mac,
`LAUNCH_FROM_MAINNET=1 LAUNCH_POOL_FROM_MAINNET=1`: mainnet-beta's feature set, Metaplex's Token Metadata program and the SPL stake pool program
read at the validator's genesis, nothing sent there. Exit 0 in 1,905 s (the Mac's load 38 to 96 at its start). Its certificate:
`poleis/ceremony/launch-press-rehearsal-with-the-stake-pool-2026-11-22.json` (no folder of the Mac it was made on).

**What it proved:**
- `press pool`: the reserve, the mint and root 0's vault of the mint in one transaction, 804 bytes, 15,949 compute units.
- Act 5, passed the same way as the other four: unit 1 convened, two ballots, sealed, its verdict heard as its chair's vote in root 0, root 0
  sealed (four seats of the floor voted by their owners); then the execute: the pool's account and its validator list for 10 validators made first in
  its own transaction by the deployer's key, then the Initialize by the polis's hand: 869 bytes, 25,210 compute units (devnet's execute of the same
  Initialize took 21,746).
- THE POOL IS BORN, read back on the chain: account type 1, its manager and its staker root 0's agent, its deposit authority the program's default
  (anyone may stake), its fee account root 0's vault of its mint (its authority root 0's treasury), SPL Token, its epoch fee 1/1.
- The handoff waited for it ("the pool is born and read back as root 0's: the handoff does not leave it unborn"); the live check counted five sets
  and read the born pool itself: `the_launch_press_on_a_live_validator ... ok` (1,798 s).

**The program's fingerprint, for his values:** the stake pool program the run was born on, recorded at the sitting's first read: 328,249 executable
bytes, sha256 `d18a2cdc0e0a52f55cb363cf517a4e72e9fca4fd6950c12897ee13c2385c52ac`, its upgrade authority `dPpFfahS…` (a multisig). Read again on
mainnet's public RPC at 13:4x UTC that day (slot 472,561,674): the same bytes, deployed there at slot 469,669,929. This is the value
`params.mainnet.json` names as `pool.programSha256`, by his hand with the other values; on mainnet the press refuses the birth on any other bytes.
