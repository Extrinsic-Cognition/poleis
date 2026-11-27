# THE CEREMONY, REHEARSED (Oct 28 2026)

**His go, by the next-step box, Oct 28 (the box's words were mine):** rehearse the mainnet ceremony on devnet under a fresh program id:
- deploy;
- found tree 0 in the same breath;
- hand the code's authority to the program's own upgrade address;
- prove an upgrade by the seal.

The order is the road's (`plans/the-road-to-mainnet-2026-10-28.md`, first lanes, item 2) and the Sept 9 launch plan's section 6.

**Where it ran:**
- first on a local test validator (Agave 4.2.0, on the new Mac), from fresh keys;
- then on devnet, the same day.

The public faucet refused a fresh key ("airdrop request failed. This can happen when the rate limit is reached"). By his second go
(the box again), his old devnet deployer 3nJn2…KMba paid the fresh keys, on devnet only.

## What is in the lane

- **`poleis/client/examples/ceremony.rs`: the adapter.** One JSON request in, one JSON line out.
  - Every step is encoded by the client's own builders, and Solana's own compiler compiles the legacy message.
  - No RPC, no keys, no signing: the same shape as `mint_adapter.rs` beside it.
  - 4 tests.
- **`poleis/ceremony/wire.mjs`: the wire, with no dependency.**
  - base58, Ed25519 keys in the CLI's file shape, signing by `node:crypto`, the legacy transaction around a compiled message.
  - 5 tests in `wire.test.mjs`.
- **`poleis/ceremony/rehearse.mjs`: the ceremony.** Fresh keys, the builds under the fresh id, then deploy, found, sale, ratify, handoff,
  upgrade by the seal, and the birth certificate.
  - It resumes from its state file.
  - Keys, builds and state live in `~/.embertide/poleis-rehearsal/<cluster>/`, outside git.

## The run (the third try, read back from the chain)

Program `2mc3KW4FJDa62YZiH3xpEQATKjy7oRNkyufj5pDk56PN` (fresh); its upgrade address `3tEorfM6Se2L3rdFdftHf1KmAh5u6XHUB6BEkS4uKAL7`.

| step | slot | bytes | CU |
|---|---|---|---|
| deploy v1 (Solana CLI), the deployer the authority | 616 | | |
| found tree 0: `init_config` + `open_genesis`, one transaction | 618 | 889 | 17,145 |
| commit one sealed seat bid | 619 | 326 | 7,909 |
| reveal it | 679 | 301 | 4,616 |
| settle the sale | 719 | 253 | 4,357 |
| claim the seat | 720 | 426 | 10,577 |
| ratify: the council takes tree 0 | 721 | 270 | 10,291 |
| hand the code to the upgrade address (Solana CLI) | | | |
| write v2 to a buffer the upgrade address owns (Solana CLI) | | | |
| propose CODEX Upgrade, by a chair | 737 | 487 | 18,634 |
| attest: 8 of 12 chairs and the floor, PRO | 738 | 907 | 7,341 |
| seal | 739 | 343 | 8,527 |
| execute: the loader replaces the code | 740 | 576 | 20,086 |

Read back after the execute:
- the program's last deploy slot moved from 616 to 740;
- the code on chain is v2 byte for byte (sha256 `039fe1a4…`; v1 was `c21894d1…`, both 401,752 bytes);
- the authority is still the upgrade address.

The twelve chairs were three stand-in keys for his three Keystones, four chairs each. The two that voted held 8 of 12, past the 7. The
floor was the one seat the sale sold.

## On devnet (Oct 28, `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`)

Program `4627ahFCh78nC9NJ7Psb7B1NSjDkmnP8SPfvbt5MQz84` (fresh); its upgrade address `F8vHRCKKD1nQbk6xZAkDqMFMZG6VnPXyv7D4HDuHK5yL`;
the fund `BaQGfpin4p7QcZ6bzkQQQ2F5vRG9Yc7ZAnFk4S3BdZe3` (bump 255). The whole record, every signature and slot:
`poleis/ceremony/birth-certificate-devnet-2026-10-28.json`.

| step | slot | CU |
|---|---|---|
| deploy v1, the deployer the authority (resumed once from its buffer) | 516,117,476 | |
| found tree 0: `init_config` + `open_genesis`, one transaction | 516,117,483 | 21,645 |
| commit one sealed seat bid | 516,117,523 | 7,909 |
| reveal it | 516,117,659 | 4,616 |
| settle the sale | 516,117,805 | 4,357 |
| claim the seat | 516,117,812 | 10,577 |
| ratify: the council takes tree 0 | 516,117,820 | 8,791 |
| hand the code to the upgrade address; write v2 to its buffer | | |
| propose CODEX Upgrade, by a chair | 516,117,968 | 15,634 |
| attest: 8 of 12 chairs and the floor, PRO | 516,117,976 | 7,341 |
| seal | 516,117,984 | 8,527 |
| execute: the loader replaces the code | 516,117,994 | 20,086 |

Read back after the execute:
- the program runs v2 byte for byte (sha256 `dd64ee70…`; v1 was `095d883d…`);
- its last deploy slot moved from 516,117,476 to 516,117,994;
- the authority is still the upgrade address.

What it cost:
- His old devnet deployer paid 4.936 test SOL. On his word, the 2.856 left unused went back to it the same day, in three transfers:
  `26RikLL7…`, `5HXFQwus…` and `51fEuVmz…`. The net cost was 2.079 test SOL: 94.015 before, 91.935 after.
- The program's account locks 2.042 SOL of rent, at the same rate as the Bronze handoff's mainnet quote.
- The ten program transactions cost 65,000 lamports of fees.

## What it proves

- **Tree 0 is founded in one transaction, two slots after the deploy.**
  - One of those slots is the loader's: a program is callable only from the slot after its deploy.
  - If root 0 were taken in that slot, the founding fails and the script stops before the handoff. The deployer, still the authority,
    closes the program, takes its rent back and deploys under a fresh id (the ceremony's rule).
- **After the handoff no key holds the code.**
  - The one upgrade went through the double seal: propose, attest, seal, execute.
  - The loader replaced the code from a buffer that only the upgrade address could use.

## What it found, for the mainnet ceremony

1. **A fresh program id moves three constants, not one:** `declare_id`, `FUND_ADDRESS` and `FUND_BUMP` in
   `poleis/program/poleis_treasury/src/lib.rs`.
   - The fund's `["fund"]` PDA is fixed there and checked by comparison (`src/ix/fund.rs:45`).
   - The rehearsal patches all three from the CLI's derivation and refuses to go on unless the client, built from the same copy, derives
     the same.
   - The mainnet lane must commit all three.
   - My first patch read the bump from the wrong JSON field and wrote 102; the cross-check now refuses that.
2. **A program is callable only from the slot after its deploy.** The first try sent the founding in the deploy's own slot, and the loader
   answered "Unsupported program id". The founding now waits for that one slot.
3. **Extending a program is permissionless here, by at least 10,240 bytes.**
   - After the handoff anyone may pay to make room for a larger upgrade, and the authority stays the upgrade address. Tried on the first
     try's program `3yzxJArj…QzNb`: 401,768 bytes became 412,008.
   - So deploying at the binary's own length, as the Bronze handoff does, does not lock a larger upgrade behind code the seal cannot call.
   - The rehearsal extends before writing the buffer whenever v2 is larger.
4. **An exported symbol breaks the build for the loader.** It made the linker stamp the ELF as GNU (OSABI 3), which the loader refuses
   ("invalid file header"). The upgrade build is the same source compiled as one codegen unit: a different binary of the same law.
5. **The public devnet faucet refused a fresh key.** Devnet needs a funded key (`--bank`).
6. **A public RPC cuts a program's upload short.** The program goes up in about 400 writes, and devnet's RPC rate limit stopped the
   first deploy ("Data writes to account failed: Custom error: Max retries exceeded").
   - The deploy now writes into a named buffer, so a cut-short upload resumes on the next run instead of stranding its rent.
   - On a public cluster the writes go straight to the validators, with 30 signing attempts. The resumed deploy and the upgrade's buffer
     went through that way.
7. **The same breath took 7 slots on devnet** (about 3 s), because the script reads the program back between the deploy and the
   founding. For mainnet, send the founding the moment the program is callable, with nothing in between.
8. **This Mac's tools.**
   - The Solana installer's `active_release` link points into the old Mac's home (`~/...`). The script takes the newest
     release that holds the CLI and `cargo-build-sbf`.
   - Host builds need the 26.5 SDK (the 27.0 linker refuses the build scripts). The script sets it when it is there.

## What it costs

- **Chain:**
  - 2.797 SOL of rent locked in the program's account at this validator's rate (the Bronze handoff quotes 1.986 SOL for 390,616 bytes on
    mainnet).
  - About 0.03 SOL for tree 0's accounts and the sale.
  - 65,000 lamports of fees for the ten program transactions.
  - The deployer went from 6.166 to 3.345 SOL. The upgrade buffer's rent came back to the spill account.
- **Time:** one run from cold builds is minutes. The sale's windows are 40 slots each on a local validator and 150 on a public cluster.

## How to run

```bash
node poleis/ceremony/rehearse.mjs
```

```bash
node poleis/ceremony/rehearse.mjs --rpc https://api.devnet.solana.com --bank <a funded devnet keypair>
```

- The first runs against a local test validator at 127.0.0.1:8899.
- The checks:
  - `node --test poleis/ceremony/wire.test.mjs`
  - in `poleis/client`: `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk cargo test --example ceremony`

## What is left

- **Mainnet's same breath:** tighten it to the first callable slot (finding 7).
- **His three Keystones** (the road's lane 3), the outside review and the seven-day soak.
- **The mainnet lane:** commits the real fresh id's three constants before the verifiable build.
