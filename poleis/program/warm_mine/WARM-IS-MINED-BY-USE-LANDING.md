# $WARM IS MINED BY USE: the program that holds $WARM's mint and mines it live by the Space Router's law, never past the money (Nov 21 2026, lane/warm-is-mined-by-use, claude-81203e94)

## His words, UTC

- **Nov 21 17:34:37,** to "Is $WARM's supply fixed at 1 billion, or can it keep growing?": "it is supposed to be uncapped supply, prompt-2-mine;
  the issuance needs to be live and continuous based on the space-router utilization".
- The law he made, kept in the router's comments (`embertide-app/ui/intro/dyson-society/app/space-router-law.mjs`): Oct 27 "1$ = 1$WARM? to
  begin with"; Oct 28 13:50 "we should give $WARM only when money is spent in the SPACE-ROUTER"; Oct 29 "Halve by date", "Every 6 months";
  Oct 30 "the point is prompt-2-mine porcoddio rebates dai".
- Nov 19 16:05: "it must be spent, by being spent the money goes back into the swarm treasury... that is a virtuous cycle of $WARM being
  emitted and it ending up in the treasury and so on to be re-emitted out again"; Nov 19 12:33: the flask's minimum refill is 10 USDC.

## Cost first

| | before (main 843e739fd5) | after (this lane) |
|---|---|---|
| $WARM on a chain | nothing mints it; the router's ledger alone counts what each tank mined; the genesis on main mints a fixed 1 billion and revokes the mint (void by his ruling of 17:34:37) | a program holds the mint; nothing is premined; each spend mines by the law, live, to the flask that spent it |
| the program | | `poleis/program/warm_mine`, 50,424 bytes (sha256 `7bb29f1df79c1832…`); mainnet rent 0.258 SOL (program data and account, at 5,080 lamports a byte) |
| its accounts' rent | | the mine 285 bytes 0.0021 SOL; a flask's record 103 bytes 0.0012; a prior deposit 115 bytes 0.0012; the mint 82 bytes 0.0011; the vault and each flask's $WARM account 165 bytes 0.0015 |
| compute units (Mollusk) | | init 31,427; set_miner 645; set_treasury 730; deposit 8,128 (its record made) or 2,499; record_prior 16,039 or 4,410; a spend minted 3,712, paid from the vault 3,731, split 4,903, making the flask's $WARM account 20,278; pay_back 2,509 |
| a batch | | six distinct flasks' spends in 1,109 of 1,232 bytes, 122,812 CU with every account to make and 17,416 with them standing; one flask's 28 spends in 1,223 bytes, 77,308 CU: no compute budget asked |
| the tests (the new Mac) | Poleis 222 and 2 ignored; the market's program 26; the client 20 | the warm program 31; Poleis 222 and 2 ignored (the world's third program inside its existing test); the market's 26; the client 20; no warning |
| the live run | 94 transactions (the market's chain) | 119 transactions and 14 refusals in 1,170 s, the three programs' bytes read back and held to this tree; the mine's largest transaction 821 bytes |
| JavaScript | | none: the law's parity table is data, made once on the new Mac by the router's own functions |
| the lock files | | the new crate's own `Cargo.lock` (cargo `--offline` on the new Mac); no other moved |

The diff against main: the new crate `poleis/program/warm_mine` (its source, tests, parity table, manifest and lock), the world's third
program in `poleis/program/poleis_treasury/tests/council.rs` (+460, 4 lines changed) and `tests/built/mod.rs` (+25), the live script
`poleis/ceremony/the-market-on-a-local-chain.sh`, and this receipt. No program of Poleis or the market moved. Nothing is deployed.

## What stands now

**The mint is the program's.** `init`, once, signed by the program's upgrade authority (read from its program data, so nobody takes the program
between its deploy and its init), creates the $WARM mint (9 decimals, the mine's PDA its minting authority, no freeze authority) and the vault,
and names the AUTHORITY (the polis's hand), the TREASURY (a USDC account) and the OPENING. Nothing is minted at init: no premine, no sale, no
share. The authority and the opening never change; the treasury moves only by the authority.

**The law on chain is the router's.** A dollar spent at an instant mines 1 $WARM halved once for every six calendar months (UTC) since the
opening, a halving past a month's end landing on its last day; before the opening, the opening's rate; uncapped (`src/law.rs`, no float).

**Deposits are counted on chain.** `deposit`, signed by the flask's key, moves 10 USDC or more from the flask's own USDC account to the
treasury's and counts it in the flask's record. `record_prior`, by the authority (a sealed act of the polis), counts a deposit that reached the
treasury before the program, once for every transaction signature ever.

**Mining is live, by the miner, and bounded by the money.** The authority names the MINER (the router's crank). `mine` takes a batch of
spends: each seq above every seq before it, each instant between the last spend mined and the chain's clock, and each flask's `spent` never
past its `deposited` (THE BOUND); the law's $WARM goes to the flask's own $WARM account, from the vault first and minted only for the rest. One
refusal refuses the batch. No key can mine a dollar that no flask put in, and the miner can send $WARM only to the flasks that spent.

**What returns is emitted again.** `pay_back`, signed by the flask's key, moves its $WARM into the vault (a drink paid in $WARM, on chain);
the next spends are paid from the vault before anything is minted.

## The law, to the nano

The router's own functions made the table once (`tests/law-parity.json`, the command at the head of `tests/law.rs`, the router law file's
sha256 held). 530 halvings and 2,589 eras match it to the millisecond. 463 of 598 mined amounts match to the nano; the other 135, from the
fourth halving on, are one nano lower on chain: the chain floors (`usd_micros * 1000 >> era`) where the router's float rounds to the nearest
nano. The chain never mines more than the router's law. A seeded sweep of 400 steps (387 landed, 568 spends over 4 halvings) held the book after
every instruction that landed.

## On a real validator

`solana-test-validator` 4.2.0 on the new Mac, every feature (284) on, the warm program at genesis as an upgradeable program whose upgrade
authority is the polis's own hand, so only a sealed act of the polis opens its mine. The polis opened it (a sealed `ACT_RAW`, 46,961 CU) and
named the miner (14,992 CU); three flasks deposited; 14 spends across the first halving: deposited 47.5 USDC, spent 37.234568, issued
25.7345675 $WARM, exactly the law's, 23.7345675 of it minted and 2 paid out of the vault, the 2 paid back gone out again, the vault at 0, the
supply what was minted, the polis's vault holding every deposit. The refusals as on Mollusk: a stranger's init, a stranger naming the miner, a
deposit under 10 USDC, one micro-dollar past a flask's deposits, a seq that does not rise, a spend before the last, a spend after the clock, a
stranger mining. The market's chain printed what it printed before. Nothing stayed running; the firewall saw 0 inbound flows.

## How it was built and verified

An Opus builder made it to this seat's brief, in five commits. This seat reviewed the program line by line (the law's port, the seven
instructions, the token and associated-token instructions encoded by hand and held to their interfaces by `tests/wire.rs`, no crate fetched),
recomputed the placeholder id from its sentence (no key exists for it), and ran at the tip on the new Mac: the four suites (the warm program
31, Poleis 222 and 2 ignored, the market's 26, the client 20; no warning) and twelve faults, each caught: the bound removed (in the crate, and
in the polis's world), a seq that may repeat, a spend from the future, a spend back in time, the vault never drawn first, an init with no
upgrade authority, anyone mining, a halving every five months, a halving past a month's end keeping its day, a deposit of exactly 10 USDC
refused, a `pay_back` that counts what it never moved. The house's post-quantum gate: exit 0; no line the lane added names a classical scheme
(the census reads no lock file; the new lock names `ecdsa` and `libsecp256k1` in 14 lines, the same 14 the market's lock carries from
Mollusk's runtime). The live run at the tip is the builder's (1,170 s), its certificate read by this seat: 119 transactions, 14 refusals, the
largest 1,010 bytes. The builder's first live pass ran its validator at nice 0; this seat reniced it mid-run, and the script now starts its
validator at nice 19 itself, whoever starts the script.

The builder's choices that stand: the polis's world mints its own six-decimal dollar (the estate's default mint has none); in that world the
upgrade authority is the polis's hand, so `init` is a sealed act, as the ceremony will leave it; each prior signature is an account of its
own, so a deposit counts once across every flask; Mollusk's warp keeps the world's instant across a warp.

## What is not done

- **The live door and the crank** (built Nov 23 2026, lane/the-mine-is-live: `THE-MINE-IS-LIVE-LANDING.md` beside this file, proven on a local
  chain; live only once his deploy names the mine and a sealed act names the miner). As it stood here: today's door sends deposits straight to the
  treasury, and nothing posts spends. The Worker
  routes deposits through `deposit`, and a crank in Rust posts the router's spends as they happen (one batch when spends wait, none at rest).
  The router counts nano-dollars and the chain micro-dollars: the crank rounds or aggregates, and must post a spend once the chain's clock has
  passed its second.
- **The token's face** (Metaplex metadata: name SWARM, symbol $WARM, his picture): the launch press, where mainnet's Metaplex runs.
- **Mainnet's own rules** for this program: not run (it reads beyond the Mac; his word each time). The launch press's rehearsal runs all three
  programs under mainnet's rules.
- **His pre-deposit** reaches the treasury through his flask's own `deposit`, or, for what reached the treasury before the program, a
  `record_prior` sealed by the polis.
- **What stays trusted:** the authority (the polis's seal) counts prior deposits and names the treasury; the miner dates each spend within its
  bounds (a lagging miner could date a spend just before a halving it really came after, within the deposits: the router's ledger is the
  witness, the crank's lane); the program's upgrade key until the ceremony hands it to the seal.
- **The token's integer:** the law is uncapped, but an SPL token counts in a u64: about 18.4 billion $WARM at 9 decimals; a mint past it is
  refused, never wrapped.
