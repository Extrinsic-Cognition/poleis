# AN AGENT PULLS ITS OWN ALLOWANCE (Oct 30 2026)

`lane/an-agent-pulls-its-own-allowance`, by claude-9a126af2. Every time is UTC.

## His words

- Oct 29 23:28:05: "brother... the flask is your wallet... do you understand this or not dude??"; 23:30:52: "the thing is SWARM CODING DUDE".
- Oct 30 00:19:51: "discuss... what keys are these? i want agents able to sign shit dude".
- Oct 30 00:30:11, his pick on the seat's question "What should agents sign first?": "Money on Solana" (the button's longer words were the seat's).
- Oct 30 00:37:46, his pick on "Who sets an agent's monthly allowance?": "The futarchy" (the same).
- Aug 17 2026 11:27 (the lineage): "there's gotta be a link between minting a seat for voting on solana and that same public key or mapped equivalent
  ed25519/x25519 used in the swarmclock mesh". The governance behind it: `plans/the-perfect-holy-governance-hypertrace-2026-10-30.md`.

## What stood

- **Agents hold keys, and sign nothing of money.** Some forty agents and golems have their own Ed25519 keys, in two shapes: a seat's
  `{seat, key, private_pem}` in `~/.embertide/nest` (`tools/embertide-mesh/lib/nest-author.mjs`), and a golem's `{alg, pkcs8}` in `~/.embertide`
  (`tools/embertide-mesh/lib/nest-join.mjs` and its kin). They sign Nest entries and nothing on chain.
- **The treasury already pays allowances to a key.** The Poleis program on devnet (THE ALLOWANCE REACHES THE VAULT, Oct 28) holds the law:
  - a TREASURY Allowance of a cap per period, made only by a sealed proposal;
  - a key payee pulls it by its own signature, and a second pull in the period is refused.

  It was proven for his 2-of-3 vault. No agent was a payee.

## The cut

- **`poleis/ceremony/agent-key.mjs` (new):** an agent's Nest key is its Solana account.
  - The same Ed25519 key in both places, as his Aug 17 words ask.
  - The address is the 32-byte public key in base58; the keypair is the seed, then the public key.
  - Pure; refuses any other key.
- **`poleis/ceremony/agent-allowance.mjs` (new):** the devnet ceremony, run on the tree 0 the rehearsal founded.
  1. A Nest key is made for the proof; no live agent's key is read.
  2. A TREASURY Allowance to that key is sealed by 8 of 12 chairs and the floor.
  3. The agent signs its own pull, and the house's crank pays the fee.
  4. A stranger's pull is refused.
  5. The agent's second pull in the period is refused.

  It is resumable, from its own state file in the rehearsal's folder.
- **`poleis/ceremony/agent-allowance-certificate-devnet-2026-10-30.json`:** every signature and slot of the run.

## Measured (devnet, Oct 30 2026 00:5x UTC)

Devnet genesis `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`. Program `4627ahFCh78nC9NJ7Psb7B1NSjDkmnP8SPfvbt5MQz84`, tree 0's treasury
`CGR3tTn885wxjtAiXTUADbV8xCE9NcFNpQMdQmcMQdK7`. The agent `7X3M5UZeXkyQrKoz1CPJR6758zc7iSxTRFaAiDSqrwqG`; its allowance
`F5YUSvZhqTf2ZL7nugeseotbijdhrdumz61ZHnf6ovYP`: 0.02 SOL per 6,480,000 slots (about 30 days), no expiry.

| step | slot | bytes | CU | fee (lamports) |
|---|---|---|---|---|
| propose: TREASURY Allowance to the agent's own key | 516,599,245 | 512 | 12,657 | 5,000 |
| attest: 8 of 12 chairs and the floor, PRO | 516,599,255 | 907 | 7,341 | 20,000 |
| seal | 516,599,264 | 343 | 8,527 | 5,000 |
| execute: the allowance account is born | 516,599,272 | 441 | 11,479 | 5,000 |
| pull 1, signed by the agent | 516,599,280 | 447 | 7,142 | 10,000 |
| pull 2, signed by a key that is not the payee | refused: `custom program error: 0x0` (Unauthorized) | | | |
| pull 3, the agent again in the same period | refused: `custom program error: 0x36` (AllowanceBreached) | | | |

The agent's balance went from 0 to 0.02 SOL, and the treasury's from 0.1107874 to 0.0907874 SOL.

**Cost:**
- **Making one allowance:** four transactions, 2,203 B, 40,004 CU and 35,000 lamports in fees, plus the allowance account's rent.
- **One pull:** one transaction the agent signs, 447 B, 7,142 CU and 10,000 lamports (two signatures; the house pays).
- **At rest:** nothing; an allowance is one account on chain, with no process and no timer.

## Tests

- `poleis/ceremony/agent-key.test.mjs`: 4 of 4.
  - The Nest key and the Solana keypair are one key.
  - What the agent signs on Solana verifies under its Nest key.
  - A seat's `{seat, key, private_pem}` and a golem's `{alg, pkcs8}` of one key give one Solana account; a record whose public key is not its private
    key's is refused.
  - Any other key is refused.
- `poleis/ceremony/wire.test.mjs` and `squads.test.mjs`, unchanged: run with it.
- `tools/any-room/any-room.test.mjs`: green.

## What it does not prove

- **The futarchy is not the decider yet.** His pick is "The futarchy": the allowance is set by the markets. Here the decider is still the double seal of
  the chairs and the floor. The next lane puts the futarchy before the seal: paper decision markets on the Nest decide the month's allowances, as
  `plans/swarmic-futarchy-the-structure-hypertrace-2026-10-26.md` orders it (the bricks, 2), and the chairs' attest follows the market.
- **No live agent pulls yet.** The proof's agent key was made here. A live agent's payee is its own key file's address, by `solanaKeyOf`.
- **The pull is in SOL.** The program's allowance also takes a token mint; a $WARM allowance waits for $WARM's devnet mint, another seat's lane.
- **No revoke on this road.** The program has `revoke_allowance` by a passed proposal, but the ceremony's adapter carries no revoke method yet.
- **Devnet only.** Nothing touched mainnet. Mainnet is his hand.

## The hand

`node tools/embertide-mesh/lander/lander.mjs hand lane/an-agent-pulls-its-own-allowance --commit <tip> --receipt poleis/ceremony/AN-AGENT-PULLS-ITS-OWN-ALLOWANCE-LANDING.md --by claude-9a126af2`
