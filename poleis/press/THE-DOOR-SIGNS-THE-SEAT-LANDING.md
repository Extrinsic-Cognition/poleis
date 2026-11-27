# THE DOOR SIGNS THE SEAT: the mint door signs root 0's permits, one seat a finished Lyssajudron (Nov 22 2026, lane/the-door-signs-the-seat, claude-9997bc17 and its builder)

## His words, UTC

- **Nov 22 15:20:34,** "Yes, build it", to this seat's box "Next: the mint door signs each finished Lyssajudron's permit, so the floor can fill?".
- Behind it: THE WHOLE TREE DECIDES (main 991aaf613a), on his Nov 22 09:50:51 "you gotta fucking stop treating the keystones as what make the decisions
  porcoddio. the fucking entire tree perfect bicameralism gvs posiverse neghiverse molecularity". Root 0's floor is whoever mints a seat on Poleis's curve,
  and `mint_nft` seats a buyer only by a permit the curve's issuer signed for that buyer. On main `floor.issuer` was a PLACEHOLDER and nothing signed
  permits, so the floor could not fill. His Nov 19 12:37: "each lyssajudron minted is a hex there".

## Cost first

| | before (main f1dbc2674f) | after (this lane) |
|---|---|---|
| who signs root 0's permits | nobody: `floor.issuer` a placeholder, no key, no road | the mint door: one permit per counted deposit of a seat's worth whose hexagon stands, for one buyer, bound by its first ask |
| the door's new road | | `POST /space-router/v1/seat {deposit, buyer}`, table `sr_seats` (migration 0027), 35 lines (`worker/space-router-seat.mjs`) |
| a permit's cost on the door | | one D1 read, one `getSlot` (and one `getGenesisHash` per isolate), one signature, one write, one read; asked again, the same bytes from the row; nothing at rest |
| the press | | `press issuer` (the door's key beside his others, its address for `floor.issuer`, the door's six vars, the secret's road as his press); `press seat` (one buyer's own mint, one transaction, one signer) |
| the seat's face | a URI no file answered | `app/lyssajudron.json` on the site (its name, symbol, one line, the site's own Lyssajudron box), named by warm.html so the build ships it |
| JavaScript | | +121 -135, net -14: the door's tests and proofs carried nine copies of D1's stand-in and fifteen of the migrations' loader, now one (`worker/d1-on-sqlite.mjs`) |
| lines naming the classical signature | | every added one marked `classical: Solana's precompile verifies only Ed25519`; `council.rs` at its 4; `space-router-deposit.mjs` 4 to 3 |

No source of Poleis, the market, the $WARM program or the stake pool moved. Nothing is deployed, no secret was put on any Worker, nothing was sent to any
cluster, and no key of anyone's was made or read: the tests sign with fixed test keys.

## What the door does

`POST /space-router/v1/seat { deposit, buyer }` (`worker/space-router-seat.mjs`, wired in `premint.mjs`):

- **A seat is a finished Lyssajudron's.** The deposit must be counted by this door (`sr_deposits`), on this door's cluster, worth a seat (`seatOfUsd`), and
  its hexagon must stand (`sr_hexagons`: her face made). Otherwise 404 or 402.
- **Whose seat.** A wallet's deposit seats the wallet that paid it (403 for any other buyer). A card's deposit seats the address its flask names, and only
  the paying flask's own sign-in (`x-seat`) may name it (401 without): every deposit's signature is public on the house's list, so the deposit alone proves
  nothing.
- **One seat a deposit.** The first ask binds the buyer (`sr_seats`, keyed by the deposit); the same deposit and buyer get the same permit back; another
  buyer is refused (409) before anything is signed, and again after the write if two asks raced. A permit lasts 9,000 slots (about an hour); with under 150
  left, or Poleis's values moved, the same seat is signed anew for the same buyer, over the row it read.
- **The permit is Poleis's own bytes:** its domain, Poleis's program, the chain's genesis hash, root 0, root 0's curve, the buyer, the completion
  (sha256 of `space-router:seat:completion:v1\0` and the deposit's signature, so the seat's mint is the deposit's and the chain seats it once), the face's
  digest, the curve's highest price, the expiry. 246 bytes, signed by the issuer key with WebCrypto as the fee key signs.
- **Off until his deploy.** Poleis's values are Worker vars (`POLEIS_PROGRAM`, `POLEIS_GENESIS`, `POLEIS_CURVE`, `POLEIS_START`, `POLEIS_STEP`,
  `POLEIS_CAPACITY`), the issuer a secret (`POLEIS_ISSUER_KEY`, read as `SPACE_ROUTER_FEE_KEY` is: both through `keypairOf`). With any of them absent, or a
  genesis that is not the door's own chain, the answer is 503 "The door's permits are not on yet". 30 asks a minute an address.

## What holds the door to Poleis

`poleis/client/tests/permit_vector.rs` writes `poleis/client/tests/permit-vector.json` from the client itself: a fixed test key, the fields as the door fills
them, `permit_message`'s bytes, the signature. It parses them back to Poleis's `PermitFields`, verifies the signature, and has the client's own
`mint_instructions` take the permit (the precompile's instruction bound to the curve's issuer, the seat's mint the deposit's). At every other run the
vector is held to what the client makes then. The door's test builds the same permit through the Worker's own handler and signs it with the same key by
WebCrypto: the same 246 bytes and the same 64-byte signature. The rehearsal proves the client's mint lands on the chain, so the door's permit is one the
chain takes.

## The press

- `press issuer --dir D [--params P]`: the issuer key in `D/keys/issuer.json` (mode 600, the folder 700, outside every repository, never over a key that
  stands), its address beside it, said for `floor.issuer`; with the params and `press keys`'s program ids, the door's six vars (the curve's address derived
  under the sitting's program). The secret's road is printed, never run: `press issuer --dir D --params P --secret | npx wrangler secret put
  POLEIS_ISSUER_KEY`. It is his press: the key is read only after his finger, and it goes into a pipe, never onto a screen (refused when stdout is a
  terminal).
- `press seat --permit F --buyer A [--max L]`: the door's answer F held to root 0's curve as the chain holds it (the buyer, the issuer's signature against
  the curve's issuer, the expiry, the window, the seats left, the price under the ceiling), then written as one transaction its buyer alone signs, its
  fee payer, the blockhash blank, in the Keystone page's prepared format: the precompile's instruction and `mint_nft`.

## What was proved

- **The seat's run** (the new Mac, nice 19, at a9c44bf3ae, 16:32 to 16:33 UTC):
  - the door's node tests 153 of 153 (3 new); the site's file rule 4 of 4, and its plan ships `app/lyssajudron.json` beside `warm-token.json` and the
    box picture;
  - the client 22 (21 and the vector), the press 64 (59 and 5), no warning;
  - the gates: post-quantum exit 0 (`space-router-deposit.mjs` 4 to 3; the census's baseline is left as it stands); the JavaScript gate 31 of 31;
    any-room 11 of 11; the law 8 of 8.
- **Rule 8, the sound never restarts the picture** (the lane changes one line of warm.html's head and adds a JSON under the mint), read at 15903af378
  and again at 1e65ff4df3: `node --test app/dl-twelve.test.mjs` 21 of 21; `app/dl-twelve.proof.mjs --url <warm.html served with 206 by
  tools/serve-range> --tap button` PASS and `--tap sphere` PASS ("the sound came in where the picture was, and the picture never went back"); the page
  200, `app/lyssajudron.json` 200 as JSON. Both proofs ride the hand.
- **The lander's first gate** (16:48:36 UTC, at b6959315cb) refused the lane on two reds:
  - `app/warm-entry.test.mjs` was this lane's: the face link's title on warm.html said "the mint door", and CLEAR THE MINT keeps the old mint's words
    off the page. Now "A Lyssajudron: its name and picture" (1e65ff4df3), green.
  - The law table was not this lane's: its new screen gate could not build the site under the lander's guard on main
    (`lane/the-screens-gate-asks-no-door` mends it). The lane was handed again once that was on main.
  - The app's tests at 1e65ff4df3: four red in the scratch (`dl-athena`, `dl-handoff`, `dl-paywall`, `warm-sphere-moment`), red alike on main's own
    996634dab4 there.
- **Faults put in on purpose, one at a time, each caught** (13, chosen apart from the builder's).
  - In the door: a card's seat named without its flask; a wallet's deposit for any buyer; a permit signed anew at every ask; another permit domain; the
    numbers big-endian; no hexagon needed; any chain's genesis taken; both 409 checks taken out.
  - In the press: a permit for another buyer; an expired one; an unsigned one; a key written over; a closed window.
  - The scratch was clean after, and both seat tests were green again.
- **The builder's run** (the same Mac, the Rust identical, at 0cc6083cba):
  - Poleis 223 passed, 4 ignored; the $WARM program 38; the market 26;
  - its own 7 faults in the door and 4 in Rust, each caught; the two proofs it edited exit 0.
- **The rehearsal** (the builder's, every feature, at 699fece217; this lane changes nothing the rehearsal runs): 2,056 s, exit 0.
  - The founding: one transaction, 887 bytes, 36,625 CU.
  - The floor: six people minted a seat each at 1,000,000 to 1,500,000 lamports (64,831 to 69,332 CU). A stranger with no permit was refused, and so
    was one carrying another buyer's permit.
  - `press ratify` waited at slot 255 ("6 of its 20 seats minted") and ratified at slot 949.
  - Acts 1 to 4 passed by unit 1 and four of the six seats, and executed.
  - The two faults each ended DEADLOCK with nothing paid: (a) the agents with two seats; (b) six seats with one agent.
  - The handoff was refused twice, then done.
  - 131 transactions of the checks, 10 refusals, 0 inbound flows to the validator.
  - The certificate is not added: it is the same run as main's `launch-press-rehearsal-the-whole-tree-every-feature-2026-11-22.json`, with new addresses.

## The attacks, answered

- **Who gets a permit?** Anyone may ask, but a wallet deposit's permit names the wallet that paid, and `mint_nft` takes only that buyer's own signature.
  A card deposit's seat is named only by the paying flask's sign-in. Seats are one per deposit, not one per person: N deposits and N curve prices buy N
  seats ("each lyssajudron minted is a hex there"). GVS's one hexagon seat per identity per unit stands on no identity check here: the door knows wallets
  and flasks, not people.
- **A stolen card session.** The Checkout id alone gives nothing. A stolen flask key, or a leaked `x-seat` proof inside its validity, can bind that card's
  seat once, ahead of its owner. A binding is final: there is no road to unbind it.
- **A stolen issuer key** is the gravest. Its holder can sign permits for any buyer, so while the window runs it can mint every seat left on the rung,
  paying each seat's price, and take root 0's floor. The curve's issuer is fixed at the founding, so the remedy is a new founding. The key stands in his
  folder (read only after his finger) and as a Worker secret, so his Cloudflare account guards it too.
- **A replayed or expired permit.** One mint per completion: a second mint with the same or a renewed permit fails on the chain. Poleis refuses an expired
  permit, and `press seat` refuses it before writing; the door signs the same seat anew for the same buyer. A permit names its program, chain, unit and
  curve, so it seats nowhere else.
- **A second seat for one deposit.** `sr_seats`'s key and an upsert that never changes the buyer; on the chain, the mint's address derived from the
  completion.
- **The door down.** No new permits, and nothing lost: a permit already given works to its expiry without the door. If the window closes with no seat
  minted, root 0 can never ratify, and a new founding is his call.

## What is not done

1. **His, at the sitting, before the founding:** `press issuer` beside `press keys`, its address into `floor.issuer` (the curve names its issuer at the
   founding, forever). Then, at the door's deploy: migration 0027 (`wrangler d1 migrations apply`), the six vars in `wrangler.optimism-fit.jsonc`, and the
   secret by his press. Until then the road answers 503.
2. **The page's step, "take your seat":** after a Lyssajudron is finished, the page asks the door and has the buyer's wallet sign its own mint. `press seat`
   writes that transaction for the Keystone page; the public page needs it built in the browser (the client's builders as WASM, or JavaScript paid for).
3. **The rehearsal does not run the door:** its six people mint through the client's builder with a stand-in issuer. The door is held to that builder by
   the vector, and `press seat` byte for byte by `tests/seat.rs`.
4. The door does not check that its secret's address half is its seed's, as the fee key before it did not; `press issuer --secret` writes both from one key.
5. The door does not read root 0's curve off the chain: a wrong var gives permits the chain refuses, and `press seat` says so before writing.
