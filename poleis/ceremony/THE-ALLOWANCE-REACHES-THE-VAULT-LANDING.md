# THE ALLOWANCE REACHES THE VAULT (Oct 28 2026)

**Why.** His pick of 15:12 (the box): the mint pays the treasury, and a monthly allowance goes to his Keystone 2-of-3. The road plan called
the second half unproven (`plans/the-road-to-mainnet-2026-10-28.md`, finding 7): a key payee must sign its own pull, and a Squads vault
signs only from inside a vault transaction its members approve.

Then at 16:50, his words: "so? do you wanna do something useful now or just gathering fucking more devnet sol which is fucking free?
hypertrace retard". This is the useful thing: his money road's middle, on chain.

**Where.** Devnet (`EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`), on the tree 0 the ceremony rehearsal founded:
- program `4627ahFCh78nC9NJ7Psb7B1NSjDkmnP8SPfvbt5MQz84`;
- treasury `CGR3tTn885wxjtAiXTUADbV8xCE9NcFNpQMdQmcMQdK7`.

Every signature and slot: `poleis/ceremony/allowance-certificate-devnet-2026-10-28.json`.

## What ran

| step | slot | bytes | CU |
|---|---|---|---|
| create the Squads v4 multisig: three members, two to approve | 516,122,832 | 519 | 16,770 |
| deposit 1 SOL into tree 0's treasury by a plain transfer (the mint's own path) | | | |
| book it: `sync_endowment`, anyone may | 516,122,910 | 211 | 1,960 |
| propose TREASURY Allowance to the vault, by a chair | 516,122,917 | 512 | 18,657 |
| attest: 8 of 12 chairs and the floor, PRO | 516,122,925 | 907 | 7,341 |
| seal | 516,122,964 | 343 | 8,527 |
| execute: the allowance account is born | 516,122,972 | 441 | 11,479 |
| pull 1: the vault transaction and its proposal (0.3 SOL) | 516,122,979 | 559 | 41,808 |
| pull 1: approved by member A | 516,122,985 | 245 | 8,624 |
| pull 1: executed with one approval of three | refused | | |
| pull 1: approved by member B | 516,122,993 | 245 | 9,493 |
| pull 1: executed with two of three | 516,123,001 | 482 | 30,945 |
| pull 2: a second pull in the same period (0.01 SOL), approved by two | 516,123,009 to 025 | | |
| pull 2: executed | refused | | |

The addresses:
- multisig `34W1t3ucD2tPxXWNxrzHfL2MKKUm5eUVXJruktUMEteV`;
- its vault `BNgjYdsxVyfPfPtzT6shHeD4LzWr5sHRubuvGqkJ5X2M`;
- the allowance `EcWh3XkzCcCPBiVYEZnV5GmjLSYFwxwPZJ8ps8zGPAft`: 0.3 SOL per 6,480,000 slots (about 30 days), no expiry.

## What it proves

- **The mint's path works.**
  - A plain transfer into tree 0's treasury took it from 0.0108 to 1.0108 SOL.
  - The program spends it (it reads the real balance), and anyone can book it with `sync_endowment`.
- **The allowance is made only by the seal.** A chair proposed it; eight of the twelve chairs and the floor voted PRO; then the seal and the
  execute.
- **One Keystone alone cannot pull.** With one approval of three, Squads refused the execute: "Error Code: InvalidProposalStatus. Error
  Number: 6008".
- **Two of three can.** The month's 0.3 SOL went from the treasury (1.0108 to 0.7108) into the vault (0 to 0.3).
  - The Poleis program saw the vault as the signer and the payee, by Squads' CPI.
  - 30,945 CU for the execute and the pull together.
- **The cap holds, whatever the members approve.** A second pull in the same period, approved by two, was refused by the program: "custom
  program error: 0x36", AllowanceBreached.

## What is new in the house

- **`poleis/ceremony/squads.mjs`:** Squads v4 by hand, by its own interface read from the chain (the IDL account `6i4UjrDt…`, version
  2.1.0). It covers the 2-of-3 multisig, the vault transaction message, the proposal, the approval and the execute. 4 tests in
  `squads.test.mjs`.
- **`poleis/ceremony/cluster.mjs`:** the session the ceremony's scripts share: keys, resumable steps, RPC, the CLI, and send-or-refuse.
  `rehearse.mjs` still keeps its own copy.
- **`poleis/ceremony/allowance.mjs`:** this proof. It resumes from its state file and needs `--bank` (a funded devnet key; his old devnet
  deployer).
- **The adapter, `poleis/client/examples/ceremony.rs`,** gains:
  - `pda` (any program's address, by Solana's own derivation);
  - `compile` (another program's instructions, compiled by Solana's own compiler);
  - `syncEndowment`, `proposeAllowance`, `executeAllowance` and `pullInstruction`.

  6 tests.

## What is left

- **His three Keystones as the members.** Today three stand-in keys play them.
  - The multisig is the same one his hands make in Solflare or Backpack by QR.
  - The proof then runs with his approvals.
- **A Keystone signing a chair's vote envelope:** still unproven (the road's lane 3).
- **The mainnet ceremony and its gates:** the road's G0 to G6.

The test SOL for this run came from his old devnet deployer: 1.47 SOL, kept in the rehearsal's keys and the vault.
