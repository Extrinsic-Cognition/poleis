# THE FLASK TAKES ITS SEAT: one tap seats a finished Lyssajudron, the house paying its SOL (Nov 22 2026, lane/the-flask-takes-its-seat, claude-9997bc17's builder, reviewed by claude-5374ded4)

## His words, UTC

- **Nov 22 17:07:40,** "Yes, build it", to the box "Next: the page lets a finished Lyssajudron take its seat with one tap of the buyer's wallet?",
  over THE DOOR SIGNS THE SEAT (main e6fef479c8).
- Behind it: his Nov 19 12:37 "each lyssajudron minted is a hex there"; Nov 11 "the people begin on a browser... they are seeing an address they can
  copy and paste" (THE FLASK BEGINS AS AN ADDRESS: the buyer's wallet is the flask this browser made).

## Cost first

| | before (main e6fef479c8) | after (this lane) |
|---|---|---|
| a finished Lyssajudron's seat | a permit from the door; the buyer had to mint it with SOL of their own, and a flask holds none | one press "Take your seat": the door writes the seat's one transaction, the flask signs it in the browser, the house's fee key co-signs, pays and sends |
| what the house pays a seat (read on the rehearsal's chain, the sixth seat) | | 12,157,040 lamports (the curve's price 1,500,000, the rents of what `mint_nft` makes 9,716,160, the flask's allowance for ten votes 940,880) + a fee of 224,000: about 0.0124 SOL; the price grows by the curve's step each seat |
| the seat's law | | `poleis/seat-wasm`, Rust built from Poleis's own client, 313,235 bytes of WASM (sha256 4f142da0...), no imports; the door runs it, the press builds the sitting's own |
| the door | | `worker/space-router-seat.mjs` the offer and the take, `sr_seats` gains five columns (migration 0030); one D1 read, the node's reads, one write a step; nothing at rest |
| JavaScript | | +109 -114, net -5 against main 2fcf6bede6 |

No source of Poleis, the market, the $WARM program or the stake pool moved. Nothing is deployed, no secret was put on any Worker, nothing was sent to a
public cluster, and no real key was made or read.

## The guards on the house's SOL

- The fee key signs only the message the door wrote and kept on its row (`sr_seats.offer`), never bytes a browser sends.
- Only the flask that paid the deposit, signed in (x-seat), may ask; one seat a deposit; a deposit of a seat's worth whose hexagon stands.
- The fee key keeps its RESERVE; one send a deposit at a time (RELAY_MS); a refused send gives the row back, an unanswered one keeps it.
- A seat already standing answers 409 with its mint; a shut or full curve, or a moved price, is refused by the node's preflight with nothing sent.

## The proof

- **The launch rehearsal under every feature** at 159c9df585, new Mac, nice 19: 2,676 s, green. THE DOOR'S SEAT: the sixth person, a flask with no SOL,
  took seat `6yVMs1fF...` through the door's own handler (59,132 CU), read back off the chain; a take signed by another key refused (403, nothing sent), a
  second take refused (409); the fee key 1,000,000,000 to 987,618,960 lamports. The certificate:
  `poleis/ceremony/launch-press-rehearsal-the-flask-takes-its-seat-159c9df585.json`. The first two runs stopped: the first on the deploy's cut, the second
  where the sixth person took its seat (`sr_deposits` takes only 'mainnet' or 'devnet', and a localnet door had begun to label rows 'localnet'); fixed at
  159c9df585.
- **After main was merged in** (f8ee399f0e; four conflicts in the door's imports and one test's setup; main took migrations 0028 and 0029, so this lane's
  is 0030), on the new Mac: `poleis/seat-wasm` 3 of 3, `poleis/press --test seat --test door` 12 of 12, the door's and the page's node tests 85 of 85
  (seat, portrait, card, deposit, grant, link, tracenet-subscribe, dl-kit-deposit, warm-entry, `app/dl-twelve.test.mjs`).
- **Rule 8** at f8ee399f0e, warm.html served by `tools/serve-range` (206): `node app/dl-twelve.proof.mjs --tap button` PASS and `--tap sphere` PASS,
  "the sound came in where the picture was, and the picture never went back".

## What is not done, and whose

- His deploy of the door: migrations 0027 and 0030, Poleis's six vars, the issuer's secret, and the fee key's SOL for the seats (his press).
- At his sitting: `press issuer` before the founding (the curve fixes its issuer then).
- The house pays every seat's SOL; how many seats it pays for, and from which balance, is his to set before the floor opens.
