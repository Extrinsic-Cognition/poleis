# A SEAT OWNER'S KEYSTONE ON A LOCAL CHAIN (Nov 23 2026, lane/a-keystone-votes-on-a-local-chain, claude-78d4ba56)

## His words, UTC, and what this lane did with its brief

- **Nov 23 2026:** his press of a card, "teach the Keystone page to sign a Poleis vote envelope, so his Keystones can vote the launch's
  acts" (the session that laid the card wrote those words, from `poleis/press/THE-SWARM-IS-ITS-OWN-MULTISIG-LANDING.md`, "What is not
  done", 1, of Nov 22 02:18). No words of his beyond the press.
- **Nov 22 09:50:51,** eight hours after that receipt, over a seat's line that his Keystones "cannot yet sign a vote": "you gotta fucking
  stop treating the keystones as what make the decisions porcoddio. the fucking entire tree perfect bicameralism gvs posiverse neghiverse
  molecularity". THE WHOLE TREE DECIDES landed on it (`poleis/press/THE-WHOLE-TREE-DECIDES-LANDING.md`): on main no key of his signs an
  envelope. The envelopes are unit 1's agents' alone; root 0's floor is the minted Lyssajudrons, and a seat votes by its owner's own
  transaction (`attest_nft`), which `press vote` writes in the Keystone page's prepared format.

**So the envelope road was not built.** The card's brief stood on a receipt his own ruling had replaced. A Keystone has one road into
the launch's acts today: its owner's own seat of root 0's floor, one hexagon like anyone's, and that seat's own transactions. The page
already signed those for mainnet; it refused every set of a local chain (`readSet`: "mainnet or devnet"), so nobody could rehearse that
road with a device. That is what this lane built, and nothing else.

## What his Keystone can sign (asked by the brief; read, not run on a device)

Read in the Keystone's open sources (`KeystoneHQ/keystone3-firmware` master 7da0b94135e1, `KeystoneHQ/keystone-sdk-base` master
79bda60d11f0) by a reader this seat sent; nothing was compiled or run on a device:

- **The envelope's bytes as they are: yes.** `sol-sign-request` has two sign types, a transaction (1) and a message (2). The firmware
  sorts a request by its bytes, not by the type it names: bytes that do not parse as a transaction are a message, and a message is signed
  as given (`app_solana::sign`, slip10 ed25519 over the bytes; no off-chain header, no hash). The envelope starts with `poleis:vote:v1`:
  its fourth byte reads as 101 accounts in 51 bytes, the parse fails, it is a message. The device shows such bytes as hex under "Raw
  Message" with a blind-signing warning, never as words. The answer is the same `sol-signature`.
- **The envelope is 55 bytes, not 54** (`ENVELOPE_LEN`: the domain's 14, the hash's 32, the unit's 8, the choice's 1). The receipts and
  the client's comment say 54. No code depends on the number; it is named here, not changed.
- **A seat owner's transaction: yes, by the same reading.** An instruction of a program the firmware has no words for (Poleis; Solana's
  signature precompile in a mint) is no refusal: the device shows a card "Unknown program" with the program's address, and in the detail
  the instruction's data in base58. The compute budget's instructions it reads.

## Cost first

| | before (main 5759bc4c0e) | after (this lane) |
|---|---|---|
| a set of a local chain on the page | refused at `readSet` | read when it states that chain's genesis hash; the page sends only to an endpoint whose genesis is that hash; a "localnet" set stating mainnet's or devnet's hash is refused |
| what ties the page's words to the bytes | the message's sha256 the set states | that, and for a set of an act: every transaction's bytes hold the act's hash, or the set is refused |
| what he can check on the device | nothing the page names | for each transaction the page prints the program's address and the data in base58, the two strings the Keystone shows for an instruction it has no words for |
| the page's JavaScript | | +65 −66, net −1 (the pull's approval now asks the Keystone by the page's one road; one send-and-wait; the gate `the-javascript-only-shrinks`) |
| the Rust copies of the page's road | `read_set` in Poleis's tests | the same two rules there, held to the page's own answers in `poleis/press/tests/prepared-vector.json` (no JavaScript runs in a gate) |
| the programs, the client, the press's sources | | not touched |
| a rehearsal of that road | none with the page itself on a local chain | `poleis/ceremony/a-keystone-votes-on-a-local-chain.sh`: 673 s the first time (the three programs built from nothing), 620 s and 534 s after, at nice 19 on the new Mac at load 30 to 100; nothing on the serving Mac |
| the tests (the new Mac, at 3b24193b1e) | | the page's 17 of 17 (one new); the ceremony's 32 and the $WARM launch tools' 19, which read the same module; the press 74, 0 failed; Poleis 223, 0 failed, 5 ignored (the new live check is the fifth); no warning |

## What the page does now

- **Reads a seat owner's set of a local chain** (`press seat`, `press vote` under `"cluster": "localnet"`): the set states its chain's
  genesis hash, the page asks the endpoint of `#rpc=` for its own and sends nothing when they differ. The blockhash it stamps comes from
  that endpoint, so a signature made for a local chain is good on that chain alone.
- **Holds a set of an act to the act:** the hash the set names must stand in the bytes of every transaction of the set, or the set is
  refused before anything is shown. The page names the act and its hash above the transactions.
- **Says what the device will show:** for each transaction, the program's address and the instruction's data in base58, read out of the
  message's own bytes. A PRO of the press's first act reads `pamhFf1PHLQPPfkDrtTGR87okkTyQy9zJHE3Uj4KJwEM4WLorTrZ3u3FE` on every chain
  (the act's hash, root 0, PRO): the vector holds it, and the press's test holds it to the client's own `attest`.
- **Unchanged:** one signer a transaction, its fee payer; a set with a placeholder is shown and never signed; another Keystone than the
  set's signer is refused; mainnet's warning. The month's pull (section 2) asks the Keystone by the same road as a prepared set.

## The rehearsal (three runs, each green; the certificate is the third's, at 3b24193b1e)

`poleis/ceremony/a-keystone-votes-on-a-local-chain-2026-11-23.json`, the picture beside it (the page while it asks for the PRO: the
proposal landed, the PRO's QR on the stage). A fresh validator of this machine, every feature it knows, nothing read beyond it. The
press's first sitting on it (build, deploy, founding, vault, mine, market, pool's accounts, acts), then, read off the chain:

- **The floor:** two stand-in people mint a seat each by the client's builders. The Keystone's owner, a stand-in key, mints the third by
  the set `press seat` wrote from the issuer's permit: opened on the page by its link, its request (6 QR frames, a transaction of 1,071
  bytes: Solana's signature check and `mint_nft`) read as a Keystone reads one, the answer held up, sent by the page. `press ratify`:
  "RATIFIED: root 0 is a council ...; its floor 3 seats, the minted Lyssajudrons".
- **The act:** the owner's set from `press vote --act 1 --propose`: its proposal (618 bytes, 4 frames), then its PRO (377 bytes, 2
  frames), both through the page. Two of unit 1's three agents sign their envelopes with their own keys.
- **No key decides alone:** `press crank` then says "ACT 1 ...: unit 1 PASSED, root 0 voting (1 seats of its floor voted by their
  owners): waits", and the check holds root 0 at VOTING, not executed. A second seat owner's PRO (the same set road, in Rust), and `press
  crank`: "ACT 1 ...: unit 1 PASSED, root 0 PASSED (2 seats of its floor voted by their owners): executed".
- Before each signature the check holds the request to the set: a `sol-sign-request` for a transaction, this Keystone's path and
  fingerprint, the set's own message with a blockhash in its blank place and no other byte moved.

**What stood in for what.** The page is the tree's own files, served on loopback by `tools/serve-range`, run in Playwright's headless
shell (no window; two pipes, no listening port; a profile of its own in the sitting's folder, removed). The camera is the browser's fake
one: the two QRs a Keystone would show (its account export, its answer) go in as their text by the hook the camera's reader feeds
(`window.keystoneScan`), and the request is read from the parts the page draws (`window.keystoneRequest`). So the QR drawing and the
camera's reading of a real screen are not what this proves; the Oct 28 dress rehearsal covered the drawing, and his own device covers
both. The stand-in's Keystone words (CBOR, UR) are built by the page's own `ur.mjs`, as the page's Node tests build them.

## The faults, put in one at a time, each caught

In `prepared.mjs`: a local set stating mainnet's genesis accepted; a local set with no genesis hash accepted; a set whose bytes do not
carry its act's hash accepted; the compute budget's instruction shown in place of the program's. Each: `prepared.test.mjs` 1 failed;
taken out, 17 of 17. The same three refusals are asserted on the Rust copy (`the_prepared_road_is_the_keystone_pages_own`).

## The attacks, answered

- **A local set sent to a public cluster?** The page sends only where the genesis is the set's, and a local set may not state a public
  one. The blockhash is the endpoint's.
- **A set whose words lie?** Its words are its writer's. What the page checks itself: the message is the one hashed, the act's hash is in
  the bytes, the signer is the Keystone asked. What he checks: the hash against the act's own set (`press acts`), and the two strings
  against the device's screen.
- **Does his Keystone decide?** No. It is one seat of the floor; alone it passes nothing, with or without the agents (above).
- **Does anything here hold a key of his, a seed, a wallet's data?** No. Every key is a stand-in's, in memory, from two seeds of the
  script's shell. `KEYSTONE_OWNER` is an address.
- **A window on his screen, a listener beyond loopback?** The browser is headless and driven by pipes; the page's server binds
  127.0.0.1; the validator's own sockets are the ones the launch's rehearsal names, checked the same way.

## His own rehearsal, with his Keystone, on a local chain (his; nothing here has run it)

1. Say to a seat on the new Mac: rehearse my Keystone on a local chain. It runs `KEYSTONE_OWNER=ask sh
   poleis/ceremony/a-keystone-votes-on-a-local-chain.sh` and stays beside him.
2. In Brave on that Mac open `http://127.0.0.1:8813/`, allow the camera, press **Scan a Keystone**. On the Keystone: Connect Software
   Wallet, then Solflare; hold its QR up; pick the account. Give the seat the address under "Connected" (the seat writes it where the
   script waits). The local chain's faucet gives that address 2 test SOL.
3. About five minutes later (the build, the deploy, the founding) the seat gives a link: his seat's mint. Open it, press **Sign with
   Keystone 1**. On the Keystone: scan the QR, compare the program's address and the data with the page's, confirm; hold the answer up
   to the camera. It must land inside the curve's window, about four minutes from the link; each answer within about a minute of the
   press (a blockhash's life). Late: the page says the chain refused it; press again.
4. About five minutes later (the window closes, root 0 is ratified) a second link: his vote. Sign transaction 1 (the proposal), then 2
   (the PRO, whose data reads `pamhFf1P...3u3FE`), the same way, inside about six minutes.
5. The script then shows the act waiting with his seat alone, a stand-in seat owner's PRO, and "ACT 1 ... executed"; the certificate
   says "his own Keystone".

## What is not done

1. **No Keystone has signed any of this.** What the device shows and that it signs a transaction holding Solana's signature check and a
   Poleis instruction is read from its source, not seen on a device. His rehearsal is the proof.
2. **`KEYSTONE_OWNER` has not been run to its end:** its waits are the stand-in's steps without the driver, and nothing but his device
   can sign them.
3. **The envelopes** are unit 1's agents': how an agent signs its own is its runtime's lane (seat claude-0733cf6f is on "the agent's
   vote road" as this is written). Nothing here signs or lands one differently.
4. **A mint's permit in base58 is about 470 characters:** nobody compares that by eye on a device. The PRO's 57 can be.
5. **55, not 54:** the client's comment and two receipts say an envelope is 54 bytes.
6. **The page's look** (its narrow instruction column, its fonts) is as it was; the look's guard does not read this page.
7. On mainnet a seat is minted through the mint door's own page; `press seat` and this page are the road of a buyer who holds a Keystone.

**Not run by this seat:** the lander's own gate (at the hand); anything against devnet or mainnet (nothing was sent to either, and
nothing read from either); the launch press's whole rehearsal (its sources are not touched: the press's and Poleis's suites ran).
