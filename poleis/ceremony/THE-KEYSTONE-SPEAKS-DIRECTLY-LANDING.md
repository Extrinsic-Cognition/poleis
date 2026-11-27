# THE KEYSTONE SPEAKS DIRECTLY (Oct 28 2026)

**His words, Oct 28:** "cant you get something here on this mac as well please? i dont want to just use the ohter one... possibly open
source so we can build from source..."

**What changed.** His Keystones no longer need a wallet extension. The page in `poleis/ceremony/keystone/` speaks the Keystone's own
QR language, in any browser with a camera and a QR reader (Brave or Chrome):
- **Connect:** the Keystone's account export (Connect Software Wallet, then Solflare) is read from the Mac's camera.
- **Approve:** the request is shown as a QR the Keystone scans, and the Keystone's signature QR is read back by the camera.

The Solflare road stays for a browser that has it.

## Built from the open sources, no dependency

- **`keystone/ur.mjs`:** Blockchain Commons' Uniform Resources (BCR-2020-005, -012, -006):
  - SHA-256, CRC-32 and Bytewords (minimal);
  - a CBOR subset;
  - Xoshiro256**;
  - the fountain code animated QRs carry.

  It is a line-by-line port of the reference (`ngraveio/bc-ur`, MIT). The same file runs in the page and in Node.
- **`keystone/sol.mjs`:** the Keystone's Solana words, from its open registry (`KeystoneHQ/ur-registry`, `KeystoneHQ/keystone-sdk-base`
  `packages/ur-registry-sol`):
  - `crypto-multi-accounts` (1103) and `crypto-hdkey` (303), the export;
  - `crypto-keypath` (304);
  - `sol-sign-request` (1101): the message, the path, the fingerprint, an id;
  - `sol-signature` (1102).
- **`keystone/b58.mjs`:** base58.
- **`keystone/qrcode-generator.js`:** the house's vendored QR writer (MIT, Kazuhiko Arase).
- **The camera:** read with the browser's own `BarcodeDetector`.

## Checked

- **`ur.test.mjs`: 8 tests, every one the reference's own vectors.**
  - SHA-256 and CRC-32.
  - Three Xoshiro sequences of 100 numbers.
  - 200 fountain degrees and 30 fragment choices.
  - The one-part UR string.
  - The 9-part fountain message decoded in any order, and recovered from mixed parts when pure ones are missing.
  - CBOR round trips.
- **`sol.test.mjs`: 4 tests, a Keystone played in Node.**
  - An account export (many keys, and one key) reads as Solana addresses with their paths.
  - A request carries the message, the path and the fingerprint.
  - The signature it returns verifies over exactly that message.
- **The QRs the page draws** (version 10, 57 by 57 modules) decode back to the request, by the same reader the page uses on the
  camera.
- **The dress rehearsal on devnet, in headless Chrome with a fake camera.** Two simulated Keystones:
  - exported their accounts into the page;
  - each read the page's 2-part animated request and signed it;
  - their answers came back through the camera's own path.

  After one approval the house key's execute was refused (InvalidProposalStatus). After two it executed: the vault went from 0 to 0.3
  SOL. Record: `keystone-qr-dress-rehearsal-devnet-2026-10-28.json`; picture: `keystone/qr-dress-rehearsal.png`.

## His taps, on this Mac

1. In Brave on this Mac, open `http://127.0.0.1:8813/` and allow the camera.
2. **Scan a Keystone.** On the Keystone: Connect Software Wallet, then Solflare. Hold its QR up to the camera. Pick the account.
3. Do the same for the other two Keystones, then **Copy the three addresses** and paste them to Claude.
4. Claude prepares his vault and the pull, and sends the page's link.
5. On that page press **Approve with Keystone N (by QR)**. On the Keystone, Scan the QR on the screen and confirm, then hold the
   Keystone's answer up to the camera.
6. Do the same with a second Keystone.

The plan's cap: these rehearsals have pulled 0.9 of tree 0's 1 SOL a period. His own run pulls `--cap 0.05`.
