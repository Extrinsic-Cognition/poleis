# HIS KEYSTONES APPROVE (Oct 28 2026)

**His go, by the box:** set up the devnet 2-of-3 with his three Keystones as members, rerun the allowance proof with his approvals, and
tell him exactly what to tap.

## What is in the lane

- **`poleis/ceremony/keystone/`: the page his hands use.** No dependency; Solflare, which holds his Keystones, signs through the Wallet
  Standard.
  - Section 1 connects each Keystone and copies the three addresses.
  - Section 2 shows the month's pull and its approvals live, and has the connected Keystone sign one Squads `proposalApprove`.
  - The page builds that one legacy transaction itself (the member pays the fee and signs) and sends it to devnet.
  - The pull's addresses ride the link's hash.
- **`poleis/ceremony/keystones.mjs`: the setup and the watch.**
  - It creates a Squads v4 multisig where his three Keystones vote (two of three). The house key may initiate and execute, never vote.
  - It seals a TREASURY Allowance of tree 0 to that vault, using the stand-in chairs.
  - It prepares the month's pull and prints the page's link.
  - Then it waits up to an hour:
    - after one approval, the house key's execute must be refused;
    - after two, it executes.
  - `--label` keeps a dress rehearsal apart from his own run.
- **`squads.mjs`:** members with their own permissions, and a proposal's status and approvals. 6 tests.

## The dress rehearsal (devnet, before his hands)

Three throwaway test keys played his Keystones in a headless Chrome. A test Wallet Standard wallet signed with them, and the page was
driven exactly as his taps would go. Picture: `poleis/ceremony/keystone/dress-rehearsal.png`. Record:
`poleis/ceremony/keystones-dress-rehearsal-devnet-2026-10-28.json`.

- **The vault:** multisig `GPQjsZ1y…qnKw`, vault `AW5XVesY…TJJa`. The house key holds initiate and execute (mask 5).
- **The allowance:** sealed by 8 of 12 chairs and the floor (`propose` 12,657 CU, `execute` 11,479 CU).
- **The pull:** prepared by the house key (44,160 CU).
- **Test Keystone 1 approved through the page.** The house key's execute was then refused: InvalidProposalStatus.
- **Test Keystone 2 approved.** The house key executed (31,293 CU): the vault went from 0 to 0.3 SOL, and the treasury from 0.7108 to
  0.4108.

The page read it back: "2 of 2 · Keystone 2, Keystone 1 · Executed".

## His taps

### Phase 1: his three addresses

In Brave on his MacBook, which has Solflare:
1. In Solflare, switch the network to Devnet.
2. Open `http://127.0.0.1:8813/`.
3. In Solflare, make Keystone 1 the active account. Press **Connect with Solflare**, then approve in Solflare.
4. Do the same for Keystone 2 and Keystone 3. If the address does not change, disconnect the site in Solflare and connect again.
5. Press **Copy the three addresses** and paste them to Claude.

### Phase 2: two approvals

Claude runs `keystones.mjs --members <his three>` and gives him the page's link, which carries the pull. Then, for two of the three
Keystones:
1. Make that Keystone the active account and press **Connect with Solflare**.
2. Press **Approve with the connected Keystone**. Solflare shows a QR: scan it with that Keystone, confirm on the Keystone, then hold the
   Keystone's QR up to the camera.

After the second approval, the house executes and the vault gets the month.
