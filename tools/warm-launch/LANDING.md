# THE $WARM LAUNCH, REHEARSED ON DEVNET (Oct 30 2026, claude-4b95a067)

The token, its Meteora pool, the drip and the money split existed nowhere on Oct 30 at 00:30 UTC. This lane builds all of it in the house, runs it
end to end on devnet, and prepares the same flow as unsigned mainnet transactions for his Keystone to sign by QR. Nothing here signs or sends on
mainnet, and the house holds no key for it.

## His rulings (UTC)

| when | his words |
|---|---|
| Oct 29 19:10:59 | "we are going to have 10% sale over 14 days... that is going to seed the liquidity needed" |
| Oct 29 19:12:48 | "$WARM is liquid upfront qnd is what the futarchy runs decisions on" ($POOL is another token) |
| Oct 29 20:42:12, 20:46:15 | the 10% is "of the supply"; "the money raised is 50% liquidity pool and 50% dev wallet"; the 90% and the pool are the swarm's |
| Oct 29 20:55:03 | the Prompt-to-Mine rebate is paid "From the 90%": nothing is minted on top, the supply is fixed |
| Oct 29 21:02:32, 21:04:16 | Prompt-to-Mine halves every 6 months from the opening (the ledger's, not this lane's) |
| Oct 29 23:46:40, 23:47:01 | the window ends Nov 15 00:00 UTC, as Breakpoint opens |
| Oct 30 00:13:26 | "Sale first, seal later": the 90% waits in his Squads until Poleis passes its outside review |
| Oct 30 00:2x | "Yes, Meteora": a DLMM pair seeded with the 10% alone, no SOL up front, so the price rises as people buy |
| Oct 30 02:28:04 | the total supply is 1,000,000,000: his click "Yes, 1 billion" on claude-1a67b2d3's question "Keep 1 billion as $WARM's total supply?" (the button's text, "Yes: $WARM's total supply is 1,000,000,000.", was that seat's words; the label is his) |
| Oct 30 02:58:56 | "dude, i will bring the proceeds of the sale into my squads wallet... u get what i mean or not?": the dev wallet is his Squads vault, the one that holds the 90% (relayed by claude-1a67b2d3) |

## What it is

Six transactions' worth of law, each signed by one key, the launch key: his Keystone on mainnet, a throwaway stand-in on devnet.

1. **The mint and the split, one transaction.** The mint is made with a seed from his address (no fresh key signs), 9 decimals, no freeze authority
   at all; 90% is minted to the treasury's account (his Squads vault), 10% to the launch key's; the name (Metaplex, SWARM, `$WARM`); then the mint
   authority is revoked in the same transaction. The supply is fixed from the first slot the mint exists.
2. **The pool.** A customizable permissionless DLMM pair SWARM/SOL (lb_clmm 0.12.0): one per pair of mints, opened only by a holder of SWARM, so no one
   can open it first at a price of their own; no alpha vault; its creator cannot switch it off; trading opens at its activation point (the opening,
   Oct 30 20:00 UTC, on mainnet). No SOL goes into the pool.
3. **The tranches.** The 10% in 16 equal tranches, one at the opening and one a day until the close (the last Nov 14 20:00 UTC). Each is laid as SWARM
   alone over the 70 bins just above the price of its moment, equal in every bin (`add_liquidity_one_side_precise2`, exact amounts): the 10% sold
   linearly at the market, never on a ladder fixed in advance. If buyers emptied the window the next tranche starts higher; if not, it deepens the
   same bins and the price stabilizes on its own.
4. **The money.** Half of every lamport the position takes in is the dev wallet's (on mainnet his Squads vault, the one holding the 90%): the SOL in its bins plus the swap fees it has earned, plus what was
   already paid out, halved, less what was paid. A round pays the earned fees first (claimed straight into the dev wallet's SOL account), then the
   rest by one share of every bin below the price (they hold SOL alone, so no SWARM moves). The pool keeps the other half as its liquidity. Sellers who
   take SOL back lower what is owed, never below zero. Rounds after tranches 4, 8, 12, 15 and at the close (a proposal; any schedule works).
5. **Mainnet.** `mainnet.mjs emit` writes the genesis set unsigned (the mint and split, the pool, two prepares, tranche 0), each with its message, a
   blank blockhash and the message's sha256. `mainnet.mjs due --k N | --money` builds each scheduled transaction at its time from the live pair
   (reads only), and finds what the dev wallet was already paid from the chain alone: the launch key's own rounds out of the pool, nothing else.
6. **The Keystone page signs it.** `poleis/ceremony/keystone/` gains a prepared road (`prepared.mjs`): open a set (a file, or `#prepared=`), and for
   each transaction in order the page checks the message against the set's hash and the connected Keystone against its signer, checks that `#rpc=`'s
   genesis is the set's cluster, stamps a fresh blockhash, shows the sign request as an animated UR QR, reads the Keystone's answer and sends it. A set
   that still holds a placeholder is shown and refused for signing.

## The devnet rehearsal, end to end (tag r3, Oct 30 2026 01:0x UTC, 36 transactions)

`node tools/warm-launch/rehearse.mjs --keys <scratchpad> --bank <a funded devnet key> --tag r3`. The keys are throwaway devnet keys in the seat's
scratchpad (never in git): the launch key, three Squads members and a create key, a dev wallet, a test buyer. The public devnet faucet is closed ("The
RPC faucet is closed on this cluster"), so test SOL came from the ceremony's devnet bank (`poleis-devnet-deployer`), as the ceremony's rehearsals take it.
Every send asks the cluster's genesis hash first and refuses anything that is not devnet's. All receipts: `receipts/devnet-rehearsal-2026-10-30.json`.

| step | what the chain says | signature |
|---|---|---|
| the treasury | a Squads v4 2-of-3 multisig `HdkzPbXY…`, vault `DeATuojY…` (stand-in members) | [2ZLAU5MU…](https://explorer.solana.com/tx/2ZLAU5MUjuW4JZPyoXUaxSit9FXJTiQQeCucrGGaSUA3m67RQw5EsDvjNxnvwAUnxiCi67TAuacSDZCjGTsFdugV?cluster=devnet) |
| genesis | mint [`6LEp2cPS…`](https://explorer.solana.com/address/6LEp2cPSxr9rSBreaswRJYbBFuSjYA9PaXv4Kaja77cy?cluster=devnet): supply 1,000,000,000, mint authority none, freeze authority none; 900,000,000 in the vault | [61grteQY…](https://explorer.solana.com/tx/61grteQYGjA8khe1WuPVZqA26Bn9wULmd5UFesWvpN15FrQUq5NznSRdWnp2Jg99Qz2LH6Ynfz4QcqQHz7ZSjnVz?cluster=devnet) |
| pool | pair [`FfaxTfLj…`](https://explorer.solana.com/address/FfaxTfLjCTwdBCfm5k5GYyzKysJAEmZqZLyt1o7AT3Eg?cluster=devnet) at bin -1741 (2.9956e-8 SOL per $WARM), bin step 100, base fee 1% | [i2TDaisv…](https://explorer.solana.com/tx/i2TDaisvAxHu6fbAws56ecvzfStU32FvHcbVBgLxS4MUSrsbKDKuRG2FUQaoZhWXFFrsAtqEBXZEFGcW42FrDz2?cluster=devnet) |
| tranche 0 | 6,250,000 $WARM over bins -1740..-1671 | [3SpCfS2X…](https://explorer.solana.com/tx/3SpCfS2XRJ64Jx5qCvUBH1ceUiZvj6UeLKC4zut2RjWCHTADYVXDbxqN66mrm1YJte8EyGW9qqjrpokrCS3J49wg?cluster=devnet) |
| the test buyer, first swap | paid 0.15 SOL for 3,685,505 $WARM: bin -1741 to -1699 (4.5497e-8); the position's SOL 0 to 0.137212 | [VQ8rmAv9…](https://explorer.solana.com/tx/VQ8rmAv9B7mpPFYQvgT5nn2n6QuRMGYQewRmzy8LpnMxwm5CxcFiwTrMmCJU4P2ASjhn7NePMwF9cMjM53GL5L6?cluster=devnet) |
| money after tranche 4 | taken in 0.441442; earned fees 0.034230 and 4734 bps of the SOL bins paid 0.220703 to the dev wallet | [2C2HYe2U…](https://explorer.solana.com/tx/2C2HYe2UNUg4BRskHPQv7YcUcaGcbieK9uV6Y3f7xvrRutQszH1dWB1EuiAtnyiCqTyucyfW3jJmCq2gmWTM4ybw?cluster=devnet) |
| money after tranche 8 | taken in 0.883411; fees 0.032128 and 3081 bps paid 0.220983 | [3Fhp2uFs…](https://explorer.solana.com/tx/3Fhp2uFs3eYhfSYT5VnW9FjaGmG11NMu6BWoFiPAJnMj4zSdor2yWpHrziVEN1vX93ExnmnuXd9tDr9iZ8C8mqBL?cluster=devnet) |
| money after tranche 12 | taken in 1.326402; fees 0.028033 and 2306 bps paid 0.221478 | [cvtg4uQs…](https://explorer.solana.com/tx/cvtg4uQs7mHrr4KeSJiLqnurG7DjfzGKEL3dJ3g1xbjTJMCWBnuN1Z9WuGAZnEuVirxp5Pug2iKqMQ3Doewng7p?cluster=devnet) |
| tranche 15 | the last 6,250,000 $WARM over bins -1647..-1578 | [2AxbrYa9…](https://explorer.solana.com/tx/2AxbrYa9J8WvWF3V6AWMw8sm2DVJa23ut87HQrYnbX9Tve9uFkBx9o1uepQVqTCfWNSHoq2k9iosDYoqXPV7m2fC?cluster=devnet) |
| money after tranche 15 | taken in 1.819841; fees 0.026247 and 1977 bps paid 0.246752 | [21Q9yvTz…](https://explorer.solana.com/tx/21Q9yvTz4JKYpP7QWJBEpyiWCJk6cfaaXJ1KHmpF5yHbbLtn8oeJ8UvgB6CF4fSdLtu3Z65wFf9DrXfHGV6DSw1k?cluster=devnet) |

**The price climbed under the buyer:** nine swaps, 1.85 SOL in all, moved the pool from bin -1741 to -1648 (2.9956e-8 to 7.5575e-8 SOL per $WARM,
×2.52) while the 16 tranches kept laying supply above it.

**The books, read from the chain at the end:**
- Tokens: treasury 900,000,000 · pool 72,032,663.745 · buyer 27,967,336.255 · launch key 0. They sum to the supply, to the base unit.
- SOL: buyers paid 1.850000000. Meteora kept 0.030159385 (its 20% of the swap fees on this pool type, `ILM_PROTOCOL_SHARE`). The position took in
  1.819840519: the dev wallet holds 0.909915808, the pool's bins 0.909924711, no fee left unclaimed. Half each, to 0.001%. The last 4,5xx lamports owed
  are under one basis point of the bins, so the close round sent nothing: the next round would carry them.
- `mainnet.mjs due --money` against this launch found the dev wallet's 909,915,808 lamports from the chain alone (its SOL account's history) and
  nothing owed.

## The Keystone page signs the mainnet set (devnet, Oct 30 01:4x UTC)

The genesis set as `mainnet.mjs emit` writes it for mainnet, emitted for devnet with the stand-in as signer, signed on the page's prepared road:
served on 127.0.0.1, opened with `#rpc=https://api.devnet.solana.com&prepared=set.json`. His Keystone was played by a throwaway key that read each
animated UR request the page showed (`window.keystoneRequest`) and answered a `sol-signature` UR through the page's own hook (`window.keystoneScan`): the
camera simulated, every UR payload real. All five landed, in order (`receipts/devnet-keystone-page-2026-10-30.json`):
genesis [36fD3nzi…](https://explorer.solana.com/tx/36fD3nzihRMxUSq4mv5ShXDJYCG8iBfHX7HXE4dfcAtY49oWPBgEWqXN5ydbzSSwdWpfLL1XL8HbCT52QPHfsued?cluster=devnet),
pool [2WnafMeg…](https://explorer.solana.com/tx/2WnafMegm9d9cE3EMjmhicq54iC6F6owmLcfJ38FYNDkYWiK9SChyQq9fquuX4e7EUh9zThAT8yvnCJYfL1SMnAv?cluster=devnet),
prepare [4WrwoEDA…](https://explorer.solana.com/tx/4WrwoEDAUaJ1pmXhg54Kw3KyQSBcJUGBBKZWTtDyQ8UDAWZiMj63zLLQMhK6Dp3FNqR655QvPBdnxgk26sxLRPf3?cluster=devnet)
and [9cJWsLK3…](https://explorer.solana.com/tx/9cJWsLK3wjuztv83CrWQEwg2TJc1s6g7C6ZU4m1GrHcDB1VfrsCov4MY18XMZX539o5B4r8e3oDHu7khv2w8f73?cluster=devnet),
tranche 0 [8XNBXUKs…](https://explorer.solana.com/tx/8XNBXUKsqiGCuRQht4wYeZzvh3g8QjSwZ9HoravH9y55NXwTRo7qW4kxhvoef7yvCPJ1H7Ubtu5sNRQtJYdv7xC?cluster=devnet).
The chain after: mint `53FFigbq…` supply 1,000,000,000, both authorities none, 900,000,000 in the treasury, 6,250,000 in the pool, 0 SOL in it, the pair's
activation point 2026-10-30T20:00:00Z, and a buy before it refused (`PoolDisabled`, 6042).

## The mainnet set

`mainnet/unsigned-genesis.json`, from `params.mainnet.json`: five transactions (712, 745, 461, 461 and 1,195 bytes), each signed by the launch key alone.
It is **not signable**: five placeholders stand, each replaced in the bytes by an address no one holds (sha256 of its words) or a stand-in number, and
named in the set: `launchKey`, `treasuryOwner`, `uri`, `startPrice`, `windowRatio`. The supply is his 1,000,000,000 (Oct 30 02:28:04 UTC);
the rehearsal's stand-in was the same number, so the five messages kept their bytes (the same five sha256s before and after). `devWallet` is left
out of the params, so it is `treasuryOwner` (`readParams`, his word of 02:58:56): one paste of his Squads vault fills both, and a `devWallet` named
there would take the proceeds instead. The genesis set never touches the dev wallet, so its bytes held again. The set names its signer
by address alone: the page asks the Keystone by the path and fingerprint it read from that Keystone's account QR when he connected it (`keystone.mjs`,
pinned by `prepared.test.mjs`), so he is never asked for either.

When his numbers are in `params.mainnet.json`:

```
node tools/warm-launch/mainnet.mjs emit
```

then, before the opening, on the Keystone page (served as the ceremony serves it) with `#rpc=` a mainnet endpoint: open
`tools/warm-launch/mainnet/unsigned-genesis.json`, connect the Keystone that is `launchKey`, sign the five in order. Trading opens by itself at
20:00 UTC. At each time of the schedule (in the set), `node tools/warm-launch/mainnet.mjs due --k N` (or `--money`) writes the next transaction for the
same page.

## Choices and their reasons

- **9 decimals.** Space Router's ledger owes $WARM to the nano (`space-tanks.mjs` `warmOwed`), so one nano-$WARM there is one base unit here, and a
  rebate at 1/1024 of its first rate (year five) stays exact. SOL has 9 too, so the pool's price per base unit is the price in SOL. The u64 holds 18.4
  billion tokens at 9 decimals; `readParams` refuses a larger supply (6 decimals would be the road then).
- **No freeze authority at all**, rather than one revoked: there is never a moment it exists.
- **The customizable permissionless pair**, Meteora's launch pool: only a SWARM holder can create it, it opens at a set moment, its creator cannot
  pause it. The program also asks its creator to hold some of the quote side (devnet refused `MissingTokenAmountAsTokenLaunchProof` until the launch
  key's SOL account held any), so 0.001 SOL is wrapped into the launch key's own account first; it stays his. Its fees, read from the devnet pair:
  base 1% (our `feeBps`), and a dynamic part the pool type sets (variable fee control 50,000, max volatility accumulator 150,000) that charges more for
  a swap that crosses many bins at once: the first test swap crossed 42 bins and paid about 8%. Meteora keeps 20% of every fee.
- **One position per bin array, his key its base and owner** (`initialize_position_pda`): only his signature, 70 bins each, so a window touches two.
- **The money straight into the dev wallet's SOL account.** The program lets the owner withdraw and claim into any token account of the right mint
  (proven on devnet), so no amount has to be computed in advance and nothing passes through another account. On mainnet that is his Squads vault's
  SOL account: the proceeds arrive as wrapped SOL, and unwrapping them is one transaction of his Squads (a CloseAccount by the vault).
- **The metadata stays mutable, his key its update authority**, so the URI can be set once it is hosted; making it immutable is one more transaction,
  his call (the house's day 0 claims, `tools/warm-day0`, count an immutable name).
- **Libraries: none in the house.** Every instruction is built here. `@meteora-ag/dlmm` 1.9.14 (with `@coral-xyz/anchor` 0.31.0 and
  `@solana/web3.js` 1.98.4) was installed only in the seat's scratchpad, once, to read DLMM's interface and to pin the bytes: the test holds every
  address, instruction and account list to what the SDK produced on the same inputs.

## Cost

Measured on devnet (r3's own transactions, read back with `getTransaction`). Devnet charges rent at 5,080 lamports a byte, mainnet at 6,960, so the
mainnet column is the same bytes at mainnet's rate.

| transaction | compute units | bytes | lamports it leaves in new accounts (devnet; mainnet) |
|---|---|---|---|
| genesis | 68k to 73k | 663 (712 on mainnet, with the priority fee) | the mint 0.0011; 0.0015 · two token accounts 0.0030; 0.0041 · the name 0.0137; 0.0151, of which 0.0100 is Metaplex's own fee |
| pool | 56k to 71k | 685 | the pair 0.0052, its two reserves 0.0030, its oracle 0.0171 (3,232 bytes): 0.0253; 0.0347 · and the 0.001 SOL wrapped into the launch key's own account |
| prepare | 216k to 218k | 409 | a bin array 0.0521; 0.0714 · a position 0.0419; 0.0574 |
| tranche | 154k to 310k | 1,183 to 1,184 (1,195 on mainnet) | none |
| money | 593k to 666k | 1,040 | the dev wallet's SOL account once: 0.0015; 0.0020 |
| a buy | 66k to 288k | 788 | none |

r3 used three bin arrays and three positions: 0.2821 SOL of rent on devnet, 0.3865 at mainnet's rate, returned if they are closed after the sale. The
fee is 5,000 lamports a signature; on mainnet each transaction also carries a priority fee, `priorityMicroLamports` 10,000 a compute unit (4,500 lamports
on a tranche's 450,000-unit limit), a proposal to raise if the network is busy. `mainnet.mjs` funds the mint at mainnet's classic rate (1,461,600
lamports), an upper bound: if mainnet's rent is lower by then, the mint holds the difference.

## What is still his (each a placeholder)

Ruled: the total supply, 1,000,000,000 (his click, Oct 30 02:28:04 UTC); the proceeds go to his Squads vault, the one holding the 90% (02:58:56).

1. **The starting price and the range.** `startPrice` in SOL per $WARM; `windowRatio`, how far above the price of its moment each tranche reaches (the
   rehearsal's 2: 70 bins at bin step 100).
2. **The launch key**: which Keystone signs the launch. Only its address: the Keystone page shows it once he connects that Keystone (section 1, its
   account QR), and keeps its path and fingerprint itself.
3. **His Squads vault's address**: it takes the 90% and the proceeds. It only receives here: the launch signs, it never does.
4. **The name's URI**, hosted where he says.

This seat's proposals, his to change: the opening (Oct 30 20:00 UTC), bin step 100 (1% between bins), base fee 1%, a tranche every 24 hours (16 in
all), money rounds after tranches 4, 8, 12, 15 and at the close, the priority fee.

## What this lane does not do, and what to watch

- **Nothing on mainnet.** No key, no signature, no send. `mainnet.mjs` holds no send and no key (the test reads its source); `rpc.mjs` refuses any
  cluster whose genesis hash is not devnet's.
- **The launch key is one Keystone, not his 2-of-3**, for the 15 days: every drip and money round is one QR scan. The 90% sits in his Squads from the
  first slot. Wrapping the drips in his Squads (two approvals each, through `poleis/ceremony/squads.mjs`) is not built; neither is moving the
  position under the seal after Nov 15 (Poleis on mainnet first, his "seal later").
- **A tranche is built against the price of its moment.** If buyers move the price past the window's first bins between the build and his signature,
  the program refuses it and `due --k N` builds it again. The signature must land within about a minute of the stamp (the page stamps just before the
  QR). Anyone may move the active bin through empty bins (`go_to_a_bin`); `due` then lays above wherever it is.
- **The money count reads only the launch key's rounds.** `paidToDev` counts a transaction only when the launch key pays its fee and the pool's SOL
  reserve is in it, so the vault's own trades with the pool, or anyone's swap paid into its SOL account, never read as proceeds already paid (pinned
  by the test; read again on devnet r3 after the change: 909,915,808 lamports, as before).
- **The money rounds keep half in the pool**, so a seller meets half the SOL the buyers paid: that is his 50/50.
- **Not read here**: the page's sale door, the Terms and the lawyer's read (others' parts); the stake pool (STAKE4INFERENCE's lane).

## Files

- `chain.mjs`, `launch.mjs`, `plan.mjs`, `rpc.mjs`, `rehearse.mjs`, `mainnet.mjs`, `params.devnet.json`, `params.mainnet.json`
- `mainnet/unsigned-genesis.json`, `receipts/devnet-rehearsal-2026-10-30.json`, `receipts/devnet-keystone-page-2026-10-30.json`
- `poleis/ceremony/keystone/prepared.mjs`, `prepared.test.mjs`, and the prepared road in `keystone.mjs` and `index.html`

## Proof

```
node --test tools/warm-launch/warm-launch.test.mjs poleis/ceremony/keystone/*.test.mjs tools/embertide-mesh/pool/pool-genesis.test.mjs
```

38 tests pass (the launch's 15, the Keystone page's 16 with its prepared road's 4, the pool genesis' 7); `tools/any-room/any-room.test.mjs` 8 pass. The devnet runs are the
receipts above; r1 and r2 ran the same flow before the money counted the earned fees (r2's reader dropped them), and r3 replaced them.

## NO SALE (his ruling, Oct 30 2026 ~15:1x UTC): this launch is not the road

- His words, in order:
  - "i think i won't do the 10% sale... i would rather do SWARMY Mint + top-up in usd u get what i mean? less degen and more opensource aligned";
  - "simply collapse into 1 ... forget the minting of the lyssajudron that is some nft bullshit... keep the gas tank refuelment";
  - "the token starts with supply 0 and each time there is a deposit the tokens are minted and when the usdc is spent the $warm is rebated".
- **What is void:** the fixed supply of 1,000,000,000, the 90/10 split, the Meteora pool, the 16 tranches, the money rounds and the programmatic
  seller (`lane/the-seller`, withdrawn by revert).
- **What stands:**
  - The one offer is a USD refuel of one's own Space Router tank (PAY A TANK, live on optimism.fit).
  - $WARM is minted from zero against deposits and rebated as the USDC is spent: 1 $WARM per $1 to begin (his Oct 27 22:48 "can we do 1$ = 1$WARM?"),
    halving by date every 6 months from the opening (his Oct 29 21:02:32 "Halve by date" and 21:04:16 "Every 6 months").
- The builders here (the mint made with a seed, Metaplex metadata, the rpc wire) may serve the elastic mint. The sale's transactions are not to be
  signed.
