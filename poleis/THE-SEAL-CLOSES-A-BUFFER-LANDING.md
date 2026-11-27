# THE SEAL CLOSES A BUFFER: a sealed act of the whole tree closes an upgrade's unused buffer, and can never reach a program (Nov 23 2026, lane/the-seal-closes-a-buffer, seat claude-5713365b)

## The cost, first

- **What it gives back:** an upgrade's buffer is handed to the upgrade PDA before its vote. If the vote fails, or the upgrade is never
  carried out, the buffer stays the seal's with its rent: about 3.03 SOL at Poleis's 434,816 bytes. Since THE SEAL NAMES ITS PROGRAM
  (landed 83c9afafed, the same day) no act reached it. This act does.
- **What it costs on the chain:** one more kind of CODEX act, 66 bytes of payload. Predicted for carrying it out: about what the hand of
  a program's authority cost before it derived an address, 20,389 compute units (the same closure over the tree, one call of the loader).
  **Measured: 24,559 units** in the rehearsal, in a sitting whose upgrade PDA takes four tries to derive (6,000 units, where a PDA
  found at the first try costs 1,500): 20,059 at one try. Poleis's binary: 434,816 bytes before, **437,280 after** (2,464 more, about
  0.017 SOL of rent).
- **What it costs the house:** 196 lines of Rust and its tests added, 2 taken out. No JavaScript. No new account, no layout or version change.

## His words, UTC

- **Nov 23 2026 ~17:42,** "Yes, build it", to this seat's question "Build a separate act that hands back or closes an unused upgrade
  buffer?", asked at the landing of THE SEAL NAMES ITS PROGRAM (`poleis/THE-SEAL-NAMES-ITS-PROGRAM-LANDING.md`, "What this takes away").

## Close, not hand back

The question offered both. This lane builds the close, and says why:

- A close needs one vote and no key afterwards: the lamports go where the act names, a treasury or a key. A hand-back needs a key to
  receive the buffer and then that key's own transaction to close it, which puts the rent in a hand again.
- The bytes are cheap to write again (526 transactions and their fees, about 0.003 SOL, in the rehearsal's deploy); the rent is what is
  worth a vote.
- A hand-back is not built. If he wants a buffer kept for a second vote instead of written again, it is a third field on this act or a
  kind of its own.

## What this lane changes

1. **`CODEX_CLOSE_BUFFER`, kind 13** (`poleis/program/poleis_treasury/src/lib.rs`, `src/ix/proposal.rs`, `src/ix/upgrade.rs`). Payload
   `[buffer:32][recipient:32]`. Only root 0 passes it, and as every CODEX act it closes over the whole tree. Carried out by anyone, as
   every passed act is.
2. **Three refusals before the loader is asked:**
   - the account in the buffer's place is not the one the act names, or the recipient is not the one it names: `InvalidAccount`;
   - the named account is not a buffer: not owned by the upgradeable loader, or its state is not `Buffer` (a program's account is state
     2, a program's data state 3): `InvalidAccount`. So an act that names a program's data, even passed by the whole tree, closes nothing;
   - the origin is not root 0: `Unauthorized`.
3. **The loader's `Close` with three accounts** (the buffer, the recipient, the upgrade PDA signing). The loader closes a program's data
   only when handed a fourth account, the program; Poleis never hands it one. Two locks on the same door, by intent.
4. **The client** (`poleis/client`): `Payload::CloseBuffer { origin, buffer, recipient }`, its bytes, hash and accounts.
5. **The press** is unchanged: it builds no such act.
6. **The browser's mirror of the client** is unchanged, and no JavaScript is added: it already knows no kind past 10 (the unit's own
   law on the SUM and on its market, kinds 11 and 12, read there as "an act" with `unknown` set), and this one joins them.

## Account layouts and versions: nothing moves

`LAYOUT_VERSION` stays 8; no account, seed, instruction tag or error code changes; one kind number is added. The devnet program
(`AJdsmjxQjHnBxDEdRxwEk2UMRW7bgNmvEMUwMziRTyab`) was not deployed to and not read: its code refuses kind 13 as it refuses any kind it does
not know, until its own seal upgrades it.

## The proof

**Red first.** With the kind's number, the client and the tests in and no executor, the council's new test was red against the built
program (17:44 UTC): the crew's act refused as an unknown payload (error 43) where `Unauthorized` was expected. Green after.

**The suites, on the new Mac, niced to 19** (`cargo test --offline --locked`, 17:45 to 17:46 UTC):

| crate | targets | passed | failed | ignored |
|---|---|---|---|---|
| `poleis/program/poleis_treasury` | 12 | 226 | 0 | 5 (they need a validator) |
| `poleis/client` | 3 | 24 | 0 | 0 |
| `poleis/press` | 15 | 75 | 0 | 0 |

The new tests:
- `council.rs` `the_seal_closes_a_buffer_it_holds_and_never_a_program`: a crew's own close refused; the root's act refused on the
  market's program data, on Poleis's own, to another recipient and with another program as the loader, the act left open each time; on
  the named buffer it passes Poleis's law and stops only where Mollusk has no loader; and four acts that name what is no buffer (the
  market's program data, Poleis's own, Poleis's program account, a wallet), each passed by the whole tree, each refused, nothing closed
  and nothing paid.
- `client` `a_close_buffer_act_names_its_buffer_and_its_recipient`.

**The launch rehearsal, on a local validator** (`poleis/ceremony/the-launch-press-rehearsal.sh`, every feature the validator knows,
nothing read beyond this machine, stand-in keys in memory only; its certificate
`poleis/ceremony/launch-press-rehearsal-the-seal-closes-a-buffer-2026-11-23.json`, `checks.buffer`): 

- **Passed, exit 0, 1,795 s in all** (17:46 to 18:16 UTC, built from commit 287e90aa9c, one run): the press sent 560 transactions, the
  checks landed 170 and were refused 15 times as they must be (150 and 12 in the certificate before this lane), slots 94 to 3,159,
  1,670 s for the checks.
- **The buffer, live, after the handoff:** 133 bytes, written by the run's own key and handed to the upgrade PDA by the loader's own
  `SetAuthority` (slot 1089), its rent 1,816,560 lamports.
- **An act naming the market's program data as its buffer**, passed by unit 1's two agents, its chair in root 0 and four of the six
  seats: refused by Poleis with `InvalidAccount`. The market's code read back whole, its authority the seal's, nothing paid.
- **The buffer's own act**, passed the same way: refused on the market's program data and refused to another recipient, the buffer and
  both wallets read back unchanged; then carried out at slot 1110, 24,559 units, 534 bytes. The buffer read back gone (no lamports, no
  data) and the named recipient 1,816,560 lamports richer, to the lamport. All three programs' authority the seal's still.

## A number of the last receipt, made exact

THE SEAL NAMES ITS PROGRAM said its act cost "3,048 more: two tries of the derivation". That was that sitting's market: a derivation is
1,500 units a try, and the tries are a property of each program's id, fresh in every rehearsal. Read from the certificates' ids: that
sitting's market took 2 tries; this sitting's takes 3, and this sitting's upgrade PDA itself takes 4 where the earlier ones took 1. So
the same hand reads 29,435 units here (20,389 + 4,500 for the market's three tries + 4,500 for the PDA's three more, to within 46 units). Across the nine
certificates before either lane the same act already read 16,011 to 23,389 for the same reason. On mainnet each id's tries are known
the moment its key is made.

## What is not done

1. **No hand-back** (above).
2. **The ceremony's upgrade script does not use it.** `ceremony/upgrade.mjs` hands a buffer to the seal and carries the upgrade out; it has
   no step that prepares this act when the vote fails. Devnet's road, and JavaScript: not touched.
3. **The pages do not name it:** a person reading the proposal on the society's page sees "an act". The same holds for kinds 11 and 12.
4. **The press prepares no set for it**, so no seat owner can vote it from the Keystone page yet.
5. No gate of the lander runs these crates on a landing (THE CRATES GATE THE SHIP): they run at the ship. The rehearsal is run by hand.
