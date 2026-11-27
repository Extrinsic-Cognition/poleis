# HER OWN WALLET: the Lyssajudron's Solana mainnet wallet, made at her birth (Oct 14 2026)

His words, in the mint lab's session (Oct 14 2026, WEST):

- 08:2x "dysonlight harmony is a nest and as a nest, it has a wallet ... that wallet is the seeker wallet as well (it is just the seeker is also
  signer but the signing key is generated on to newmac using solana sdk stuff offciail the best)"
- 08:3x "solana mainnett... THE USER JOURNEY OF DYSONLIGHT HARMONY IS THAT THE FUCKING LYSSAJUDRON HAS GOT ITS OWN FUCKING WALLET"

The lane was sent by the optimism.fit lineage (ARCHI-TECH ARCHO-TECH, local_79846786; its ledger
`ec-workspace/docs/optimism-fit-lineage-hypertrace-2026-10-14.md`). Branches: `claude/heuristic-knuth-262653` (the crate), then
`claude/bold-visvesvaraya-9a410a` (everything else, after the first seat went to rebuild the unboxing in 3D).

## Before (read Oct 14 08:3x to 08:5x WEST)

- No code in the house made a wallet for her. The Nest held no wallet ref: `GET /nest/refs?prefix=wallet` and `?prefix=lyssajudron` were empty.
- The Mac app's mint review (`embertide-app/ui/intro/mint-review.mjs`, the transport `tools/embertide-mesh/lib/harmony-mint-wallet.mjs`, fail-closed
  with `builder: null`) took as buyer whatever wallet the person's Phantom connected through `mint-wallet-helper.html`. No page opens that review:
  his Oct 13 15:05 "dont make it there" took it off Continue (house `5240d6d8`).
- The live website premint (`optimism.fit/app/warm.html#mint`, `app/dl-direct.js`, Bronze $10 and Gold $1,000 paid to `3nJn2…KMba`, 0 holders) took
  whichever wallet the browser injected.
- She already has an Ed25519 key of another kind: the golem seat key `~/.embertide/lyssajudron.key.json` (Sep 24), one of the twelve golems' Nest
  keys. It is not this wallet and this lane does not touch it.
- NewMac's `~/.config/solana` is a copy of the old Mac's (`id.json` of Jan 3); no key was ever made there. NewMac has `cargo` and the SDK crates in its
  cargo cache, no `solana-keygen`.

## What landed

| piece | where | what it does |
|---|---|---|
| the maker | `embertide-wallet/` (Rust; the official SDK's crates: `solana-keypair` 3.1.2 with `seed-derivable`, `solana-derivation-path`, `solana-seed-phrase`, `solana-signer`, and `tiny-bip39` 0.8.2, the BIP39 crate the keygen uses) | `new <folder> [--birth id] [--words 24\|12]`: 24 English BIP39 words, no passphrase, her key at **m/44'/501'/0'/0'**; three files in a folder named by her address: `keypair.json` 0600 (the SDK's `write_keypair`, what the `solana` CLI reads), `words` 0600 (what the Seeker imports), `public.json` 0644. Built under a dot name and renamed whole, every file F_FULLFSYNCed, nothing overwritten, the wallets folder 0700 and refused when it is a link or open to others. Stdout is the public line alone. `derive` (words on stdin) and `check` print no secret. An unknown command line runs nothing. |
| the cross-check | `embertide-wallet/keygen-cross.proof.mjs`, `keygen-pty.exp` | her words against the official Agave keygen both ways (below) |
| her door | `tools/embertide-mesh/lib/lyssajudron-wallet.mjs`, mounted in `harmony-aiolos.mjs` behind localOnly | `POST /api/lyssajudron/wallet {birth}`: only for the birth the Nest's `scene/entities/golem-lyssajudron-paint` names; made once (the ref naming the birth, or a `public.json` naming it, answers again with no new key; asks in flight share one making); the maker run with no shell, an empty environment, 5 s and 4 KB; the Nest told by the Mac's key, signed, CAS with four swaps: **`lyssajudron/wallet`** = `{schema 'embertide.lyssajudron-wallet/1', golem, address, network 'solana-mainnet-beta', path, words, birth, made_ms, by 'harmony'}`. `GET` answers the public record. `holder()` gives the mint her address only while her folder on this machine agrees with the ref. The door never opens `keypair.json` or `words` (tested by watching every read-open). A door that cannot load answers 503; Harmony stays up. |
| the mint's buyer | `harmony-mint-wallet.mjs`, `mint-wallet-review.mjs`, `mint-review.mjs`, `mint-wallet-helper.{mjs,html}` | **her address is the only buyer and holder**: a mint-ready builder needs the holder callback; `start` refuses without her; every status names her; `connect` takes no other wallet (409 `not-her-wallet`, the session stays open); the sheet says "Her wallet", "Connect her wallet", or "Her wallet first" when she has none on this Mac; Phantom holding another wallet is told hers. Still fail-closed: aiolos's builder is `null`, nothing is minted. |
| the website premint | `dyson-society/app/dl-direct-wallets.js`, `dl-direct.js`, `warm.html` | a link naming her, `#mint:for=<her address>` (a fragment: no server or log sees it), makes the checkout take her wallet alone ("Her wallet buys and holds this seat", "Connect her wallet"; another wallet refused before any order, receipt or storage is read); her name rides into the wallet-browser links; a reload keeps it. No protocol, Worker or D1 change. **Deployed nowhere.** |
| her line on the Mac page | `embertide-app/ui/intro/room/her-wallet.mjs`, the page's block "HER OWN WALLET" in `tools/embertide-mesh/mac-onboarding.html` (house `8ef9b4ca`, after the 3D unboxing's seat freed the page at `f47835ff`) | when her eyes are shown for a birth (climax at its end; woke on a reload), one POST to this Mac's Harmony, and one quiet line between her eyes and Continue: "her wallet · So11…1112" (the whole on hover, copied on a press) and "3 open" unfolding his calls. The cause is the strip's own showing, watched by one MutationObserver: no function of the birth is touched. Seen in headless Chrome at his window's size by `birth-film/her-wallet.proof.mjs`: 17 of 17 (one ask at her eyes, the line 20 px below her eyes as drawn, clear of Continue and the gaze row, painting, the calls unfolding and folding). |

## The receipt

Cost before: nothing (no wallet, no process). Predicted in the lane before the build: the binary 1 to 2 MB, one build 30 to 90 s of CPU, `new`
5 to 15 ms with its fsyncs, the route about one Nest write a birth.

| measured (the serving Mac, Apple M4 Max, load 7 to 10, nice 19) | value |
|---|---|
| the binary, stripped | 816,560 B |
| a full offline build in a fresh worktree (`-j 4`) | 12.2 s wall, 35.5 s CPU, 448 MB peak RSS |
| `new`, spawn to exit (four F_FULLFSYNCs), 10 rounds | median 22.75 ms, max 30.55 ms (F_FULLFSYNC was underestimated) |
| `derive`, spawn to exit, 10 rounds | median 3.38 ms, max 4.2 ms |
| the first exec of a freshly built binary (macOS checks it once) | 108 to 141 ms; later execs 1.8 to 2.6 ms |
| `POST` of a new birth through the door (route.bench.mjs, 20 births, real Nest, real maker) | median 29.34 ms, max 42.16 ms |
| `POST` of a birth already made | median 1.24 ms (two GETs, no process) |
| `GET` | median 0.53 ms |
| bytes | the ask 26 B, the answer 216 B, the ref's value 255 B, its signed WAL entry 873 B |
| her folder on disk | 671 B (keypair.json 227, words 151, public.json 293) |
| at rest | nothing: no timer, no poll, no process; the door reads no file and no key at boot |

**The official keygen agrees with her words, both ways, 10 of 10** (`node embertide-wallet/keygen-cross.proof.mjs 10`, solana-keygen 4.2.0 Agave):
(1) the keygen draws 24 words at `--derivation-path` (left empty: m/44'/501'/0'/0') and this crate's `derive` of those words is the keygen's pubkey;
(2) this crate's `new` draws and `solana-keygen pubkey prompt://?key=0/0` of its words is the address `new` answered and `public.json` holds;
`check` passes on every folder. The cargo tests carry BIP39's own vector (the TREZOR seed) and SLIP-0010's ed25519 vector 1 through the SDK.
A trap written down: plain `solana-keygen new` does not derive (it takes the seed's first 32 bytes), so the same words would give another address
than Phantom's; `--derivation-path` is what makes them agree.

Tests, all niced: the crate 7 of 7; the door 10 of 10 (the real Nest on a lent port, a test key in a temp keys folder, the real maker once with
throwaway keys); the transport 24 of 24 (22 kept, each fixture's buyer named as her wallet; 2 new); the review client, sheet and state 38 of 38; the
helper 2 of 2; the website's wallets 6 of 6; her line 3 of 3. aiolos's boot and closure suites 82 of 84: the two red are not this lane's
(`harmony-aiolos-panes` expects no house-tree stamp, which main's `09fe93ab` adds: red with main's aiolos put back in this worktree too;
`harmony-main-routing` opens `ui/harmony.html` through a link that dangles in a worktree). Seen in the Browser pane: the premint's Bronze dialog at
`#mint:for=<an invented address>` and a stand-in wallet refused with no request made.

Nothing ran on the live house: no route asked, no ref written, no wallet of his made. Every key in this lane was a throwaway in a temp folder,
removed by the run that made it.

## Where the secret is, and what never happens

Her keypair and words live in `~/.embertide/lyssajudron/wallets/<address>/` on the machine of her birth: the folder 0700, `keypair.json` and `words`
0600. Never in git (outside the repo), never on the wire (no route answers a byte of them; aiolos never opens them), never printed (the maker's stdout
is the public line, its stderr a reason), never logged (the log names her address only). The Seeker is meant to show the same address once her
words are imported: the path is Phantom's first account and the keygen's; Solana Mobile's help pages say the Seed Vault Wallet is Solflare's, and no
official page found states its path, so **the receipt is his import on the Seeker**.

## His calls (open, none decided here; shown on the page by `room/her-wallet.mjs` once it is wired)

1. **How her wallet gets its first SOL.**
2. **Where the secret lives**: this Mac's Keychain, the Nest's key store, or the Seeker only (today: the 0700 folder above, a default that moves
   whole: the words are all any of the three needs).
3. **Whether her Ed25519 key is also her iroh endpoint key** (foundation 9: every process an iroh endpoint with its Ed25519 key; her Solana address
   and a node id are the same kind of 32 bytes).

## His hand, and what is left

- **The route goes live at aiolos's next restart** on each Mac (this lane restarts nothing). Read after the landing (house `f6e8a150`, 10:10 WEST):
  NewMac's follow job restarted its aiolos by itself at 10:11:22, and `GET /api/lyssajudron/wallet` there answers 200 `{"ok":true,"wallet":null}`
  (no wallet yet: nothing has asked); this Mac's aiolos still answers 404 until its restart, his hand.
- **Both Macs have the maker** (built after the landing, offline, nice 19, into each house checkout's ignored `target/`): NewMac 10:12 WEST, 8.0 s
  wall, 21.9 s CPU, 800,864 B (rustc 1.99, `DEVELOPER_DIR` and `SDKROOT` set as every link there needs); this Mac 11.9 s, 816,560 B. Both derive the
  same address from BIP39's public test phrase ("abandon" x11 "about", nobody's wallet), `HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk` at
  m/44'/501'/0'/0', and so does the official keygen (`solana-keygen pubkey prompt://?key=0/0` through `keygen-pty.exp`). That run also paid each
  binary's first-exec check, so his birth does not.
- **The page's line** is wired (house `8ef9b4ca`). On this Mac it draws nothing until aiolos restarts (the route answers 404 until then); on NewMac
  his next birth asks, her wallet is made there by his own birth, and her address stands in the strip.
- **The website change is not deployed** (his deploy line is the ledger's).
- **For strangers' Macs** the maker is not in the signed release yet: the release road must carry `embertide-wallet` (like the mint builder's
  `mint_adapter`, which has the same gap: `tools/embertide-mesh/lib/harmony-mint-builder.md`, Runtime package).
- **`lyssajudron/wallet` is not an owner ref**: `nest.mjs` stands edited and uncommitted in the house checkout (another seat's consensus work), so the
  ref could not join `OWNER_REFS`; the mint trusts her folder, not the ref alone. A follow-up once `nest.mjs` is free.
- **Trading her** (THE CHARACTER SLOTS' exchange over the internet): her wallet is a folder on one machine; what travels with a traded Lyssajudron
  is not decided anywhere.
