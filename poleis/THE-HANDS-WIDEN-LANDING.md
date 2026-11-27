# THE HANDS WIDEN: a unit's raw act carries sixteen accounts and a named co-signer, so a polis can hold a stake pool by its own hands (Oct 29 2026, lane/the-hands-widen, claude-61e371e5)

## His words, Oct 29 2026, UTC

- **20:34:07**, asked which key manages the mainnet stake pool: "you gotta go back to the design of our governance dude... fucking hell the pentagon and
  hexagon voting shit dude please"
- **21:11:11**, asked "Widen the polis's hands on devnet, sealed by your Keystones?": "Yes, widen them"

**Why.** The stake pool's authority is the double seal, never a key. Read in the code on Oct 29, the society's hands could not carry it
(`tools/embertide-mesh/pool/THE-POOL-WAITS-ON-HIS-KEY-LANDING.md`):
- ACT_RAW passed at most 8 accounts, and only the unit's agent could sign.
- The SPL stake pool's Initialize takes 9 accounts (10 with a deposit authority), and AddValidatorToPool takes 13.
- Handing a pool over is SetManager, where the outgoing manager and the new one sign together.

## Cost first

- **The payload.** A raw act's payload lives in its proposal in clear (`MAX_PAYLOAD` 768). Sixteen accounts with their flags take 528 bytes, which leaves
  205 bytes of instruction data. AddValidatorToPool's payload is about 469 bytes and Initialize's about 419.
- **Compute,** measured in the Mollusk suite (`POLEIS_CU=1`):

  | the act | its execute |
  | --- | ---: |
  | sixteen accounts | 15,978 CU |
  | two accounts | 14,281 CU |
  | a co-signed act | 14,276 CU |
  | a seventeenth account, refused | 6,399 CU |

  A wide act costs about 1,700 CU more than a narrow one.
- **The binary:** 401,872 bytes for sBPF v3, 120 more than devnet's 401,752. The build reported no stack warning.

## What changes

- **`program/poleis_treasury/src/lib.rs`:** `MAX_ACT_ACCOUNTS` goes from 8 to 16.
- **`program/poleis_treasury/src/ix/proposal.rs`, ACT_RAW:**
  - The agent's signature is still the program's alone to give.
  - Any other key that the sealed payload names with ACT_SIGNER must have signed the executing transaction itself: a named co-signer.
  - An executor without that key is refused (`ActForbidden`), exactly as before. Nothing else in the act changes:
    - no account of this program may appear;
    - the act never calls this program back;
    - the agent is the hands, never the treasury.
- **`client/src/lib.rs`:** an execute marks a named co-signer as a signer of the outer transaction, and the agent never. The Raw payload's doc says so.
- **`program/poleis_treasury/tests/council.rs`:**
  - `the_raw_act_is_the_agents_hand_and_never_the_treasurys` now refuses the forged signer when it has not signed, and passes when it has: the co-signer.
  - `the_hands_carry_sixteen_accounts_and_no_more` is new: sixteen pass and move the transfer; a seventeenth is refused `BadPayload` and moves nothing.
- **`ceremony/upgrade.mjs` (new):** an upgrade of a ceremony's program by its own seal, from the house's source as it stands.
  - `prepare` makes a copy patched to the ceremony's id (the declared id, FUND_ADDRESS and FUND_BUMP, as `rehearse.mjs` does) and builds it for sBPF v3.
  - It extends the program first if the build is larger: at least 10,240 bytes, since Agave 4.2's loader refuses less, and anyone may pay.
  - It writes the build to a buffer whose authority is the upgrade address.
  - `seal` makes the proposal, the named keys' PRO votes, the seal and the execute, then reads the code back byte for byte and writes a certificate.
  - It refuses mainnet.

## Measured (the new Mac, Oct 29 21:1x to 21:3x UTC, Agave 4.2.0, `SDKROOT` the 26.5 SDK as the house's standing workaround)

- **`cargo test --no-fail-fast` in `program/poleis_treasury`:** 152 passed, 0 failed, and 1 ignored, which was ignored before.

  | suite | passed |
  | --- | ---: |
  | unit | 45 |
  | adversarial | 18 |
  | assembly | 12 |
  | bls | 10 |
  | council | 31 |
  | fuzz | 1 |
  | fuzz_council | 1 |
  | mathlaw | 2 |
  | nft | 22 |
  | nft_governance | 6 |
  | poleis | 4 |

- **`cargo test` in `client`:** 19 passed.
- **`node --test poleis/ceremony/squads.test.mjs poleis/ceremony/wire.test.mjs`:** 11 passed.

## On devnet

Unit 0's live instance is the ceremony's `4627ahFCh78nC9NJ7Psb7B1NSjDkmnP8SPfvbt5MQz84`, and its code answers only to its seal's upgrade address,
`F8vHRCKKD1nQbk6xZAkDqMFMZG6VnPXyv7D4HDuHK5yL` (`ceremony/birth-certificate-devnet-2026-10-28.json`).

- **The patched build:** 401,872 bytes, sha256 `a5bf9c1a096e9000f34f525e6edf58e77f701567fab626ae3867faf3efae72aa`.
- **The ceremony's deployer** held 0.18 devnet SOL against the buffer's 2.04 SOL of rent. The house's Poleis devnet payer sent it 2.3 devnet SOL
  (`tycdhVxr…`); the rent comes back at the upgrade.
- **Prepared, 21:3x UTC** (`node poleis/ceremony/upgrade.mjs prepare --label the-hands-widen`):
  - The program was extended by 10,240 bytes to 411,992, paid by the deployer. The authority is still the upgrade address.
  - The build sits in buffer `6zmUiF4GWnqUm82663MaJ1JqS7ZxLE7KgHTHjziqYkYq` (401,872 bytes), whose authority is the upgrade address.
  - Nothing changes the code until the seal.
- **Sealed by the stand-ins at his word, 21:23:57 UTC: "Stand-ins now".** He chose them over seating his Keystones first, and the Keystone vote is its own
  proof before mainnet. `node poleis/ceremony/upgrade.mjs seal --label the-hands-widen --voters chairA,chairB,bidder`:

  | step | slot | bytes | CU | signature |
  | --- | ---: | ---: | ---: | --- |
  | propose: CODEX Upgrade, by chair A | 516,549,017 | 487 | 14,134 | `2CCNs4SBq6Wb…` |
  | attest: chairs A and B (8 of 12) and the floor, PRO | 516,549,024 | 907 | 7,341 | `4NsAJNuv6Nyk…` |
  | seal | 516,549,032 | 343 | 8,527 | `3Z35RrBZBobK…` |
  | execute: the loader replaces the code | 516,549,040 | 576 | 20,127 | `22GGyG5jzEcW…` |

  - Proposal `e922f762869afe7cae1125c42187a3fd5085fe6e22cd81ae9119346b1363606a`.
  - Read back: the program runs `a5bf9c1a…` byte for byte (411,992 bytes held), and its authority is still the upgrade address.
  - The buffer closed and its rent came back: the deployer went from 0.384 to 2.427 devnet SOL.
  - The certificate is `ceremony/upgrade-certificate-devnet-the-hands-widen.json`.
- **Who sealed:** on devnet unit 0's twelve chairs are stand-ins for his three Keystones, four each, held in key files
  (`ceremony/rehearse.mjs`: `CHAIRS`). His real Keystones are members of the Squads vault, and a Keystone signing a vote envelope is still unproven
  (`plans/the-road-to-mainnet-2026-10-28.md`, item 5).

## The pool born by the seal (devnet, Oct 29 21:30 UTC, his 21:26:27 "Yes, birth it")

`node poleis/ceremony/birth-pool.mjs --voters chairA,chairB,bidder --tag u0a`. This is the first act of the widened hands. An SPL stake pool was
initialized by unit 0's raw act, so no key is its manager.
- **The adapter** (`client/examples/ceremony.rs`) learned three steps: `proposeRaw`, `executeRaw` and `createVault`. It is built from the deployed patched
  copy.
- **The steps:**

  | step | slot | bytes | CU | signature |
  | --- | ---: | ---: | ---: | --- |
  | the pool's empty accounts, paid by the crank (anyone may) | 516,550,516 | 954 | 8,560 | `2Emuosh1cmP3…` |
  | unit 0's vault for the pool's mint, the fee account (`create_vault`, anyone may) | 516,550,524 | 344 | 9,190 | `59XXJvCxWcKC…` |
  | propose: ACT Raw, the pool's Initialize, by chair A | 516,550,532 | 809 | 12,798 | `nPyirDosDdzE…` |
  | attest: chairs A and B and the floor, PRO | 516,550,540 | 907 | 7,341 | `3BvcBcE7jrHm…` |
  | seal | 516,550,548 | 343 | 8,527 | `bTiuRj3rwdfU…` |
  | execute: unit 0's hands initialize the pool, nine accounts | 516,550,556 | 673 | 21,746 | `AryzfDmNKWja…` |

- **Read back from the chain:**
  - the pool `8V8pbPFsXxS51MSKGdk7JWLMQcKgtwq7VRZwggiRinVA`, account type StakePool;
  - its manager and staker are both unit 0's agent, `5h7yxFR6ajKgLCjcUFpdpuTjxsJr7z8RvVzcgqG7oQNc`;
  - its mint is `4HzvsUN2qMENp6gFKAmFkRCLWugmz4Sf2WTjXY9XdoEr`;
  - its fee account is unit 0's vault `FivrVD8FBNBWAFASKrysBbWss4BTeJwmR4a63Dihetnb`, whose authority is tree 0's treasury;
  - its epoch fee is 1/1.

  Proposal `38cafee3415552878363807b5927143139e025dae192490788c6ed3dbe1c890d`. The certificate is `ceremony/pool-birth-certificate-devnet-u0a.json`.
- **So, on devnet:**
  - The whole yield lands in tree 0's treasury and moves only by a sealed TREASURY act.
  - The pool's validators and fees change only by a sealed raw act of unit 0. AddValidatorToPool's 13 accounts now fit.
  - The pool's own SOL stays in its reserve and its validators' stakes; only the pool program moves it.
- **Not yet:**
  - a validator added by the seal;
  - a first deposit;
  - the first epoch's yield minted into the vault;
  - his Keystones as the chairs, which needs a Keystone vote envelope proven before mainnet.

