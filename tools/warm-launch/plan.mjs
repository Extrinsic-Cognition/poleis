// plan.mjs · THE $WARM LAUNCH, its transactions (Oct 30 2026): the same builders for the devnet rehearsal and for the unsigned mainnet set. Each returns
// instructions and what they do; nothing here reads a key or a cluster. The launch key is the one address that signs every one of them: his Keystone on
// mainnet (by QR, poleis/ceremony/keystone/), a throwaway stand-in on devnet. Every new account is his address's own (the mint made with a seed from it,
// the positions with it as their base), so no fresh key ever has to sign.
import * as C from './chain.mjs';
import * as L from './launch.mjs';

export function addresses({ launchKey, seed, treasuryOwner, devWallet }) {
  const mint = C.withSeed(launchKey, seed, C.P.token);
  return {
    launchKey, seed, mint, metadata: C.metadataAddress(mint), pair: C.pairAddresses(mint, C.P.wsol),
    launchSwarm: C.ata(launchKey, mint), launchWsol: C.ata(launchKey, C.P.wsol),
    treasuryOwner, treasurySwarm: C.ata(treasuryOwner, mint),
    devWallet, devWsol: C.ata(devWallet, C.P.wsol),
  };
}

/** 1. THE MINT AND THE SPLIT, one transaction: the mint (no freeze authority, ever), the 90% into the treasury's account and the 10% into the launch
 *  key's, the name, then the mint authority revoked in the same breath: the supply is fixed from the first slot it exists. */
export function genesis({ p, a, mintRent }) {
  const { treasury, sale } = L.split(p.supplyBase);
  return {
    name: 'genesis: the mint, the 90/10 split, the name, the mint authority revoked',
    amounts: { supply: p.supplyBase, treasury, sale },
    ixs: [
      C.createAccountWithSeed(a.launchKey, a.mint, a.seed, mintRent, C.SIZES.mint, C.P.token),
      C.initializeMint2(a.mint, p.decimals, a.launchKey),
      C.createAtaIdempotent(a.launchKey, a.treasuryOwner, a.mint),
      C.createAtaIdempotent(a.launchKey, a.launchKey, a.mint),
      C.mintTo(a.mint, a.treasurySwarm, a.launchKey, treasury),
      C.mintTo(a.mint, a.launchSwarm, a.launchKey, sale),
      C.createMetadataV3({ mint: a.mint, authority: a.launchKey, name: p.name, symbol: p.symbol, uri: p.uri, isMutable: p.metadataMutable }),
      C.revokeMintAuthority(a.mint, a.launchKey),
    ],
  };
}

/** 2. THE POOL: a customizable permissionless DLMM pair SWARM/SOL (one per pair of mints, made only by a holder of SWARM: no one can open it first
 *  at a price of theirs), its active bin the starting price, no alpha vault, the creator unable to switch it off; trading opens at the activation
 *  point (the opening on mainnet, now on devnet). No SOL goes into the pool. The program asks its creator to hold some of the pair's quote side too
 *  (devnet, Oct 30: refused with MissingTokenAmountAsTokenLaunchProof until the launch key's own SOL account held any), so 0.001 SOL is wrapped into
 *  the launch key's account first; it stays there, his. */
export const PROOF_LAMPORTS = 1_000_000n;
export function pool({ p, a, activationPoint }) {
  const activeId = L.binOfPrice(p.startPrice, p.binStep, p.decimals);
  return {
    name: 'pool: the SWARM/SOL DLMM pair at the starting price',
    activeId,
    ixs: [
      C.createAtaIdempotent(a.launchKey, a.launchKey, C.P.wsol),
      C.transfer(a.launchKey, a.launchWsol, PROOF_LAMPORTS),
      C.syncNative(a.launchWsol),
      C.initializePair({ pair: a.pair, funder: a.launchKey, userTokenX: a.launchSwarm, userTokenY: a.launchWsol, activeId,
        binStep: p.binStep, feeBps: p.feeBps, activationPoint }),
    ],
  };
}

/** The bin arrays and positions a window needs (one position per bin array, his key its base): only the missing ones. */
export function prepare({ a, indexes, have = { arrays: new Set(), positions: new Set() } }) {
  const ixs = [];
  for (const index of indexes) {
    if (!have.arrays.has(index)) ixs.push(C.initializeBinArray({ pair: a.pair, funder: a.launchKey, index }));
    if (!have.positions.has(index)) ixs.push(C.initializePosition({ pair: a.pair, owner: a.launchKey, index }));
  }
  return { name: `prepare: bin arrays and positions ${indexes.join(', ')}`, ixs };
}

/** 3. and 4. A TRANCHE: SWARM alone into the window just above the active bin, equal in every bin. One add per position the window crosses. */
export function tranche({ a, active, width, amount, units = 1_000_000 }) {
  const laid = L.layTranche({ active, width, amount });
  const groups = L.byPosition(laid.bins);
  return {
    name: `tranche: ${laid.laid} base units over bins ${active + 1}..${active + width}`,
    laid: laid.laid, carry: laid.carry, multiplier: laid.multiplier, indexes: groups.map((g) => g.index),
    ixs: [C.computeLimit(units), ...groups.map((g) => C.addSingleSided({ pair: a.pair, owner: a.launchKey, index: g.index, userToken: a.launchSwarm,
      bins: g.bins, multiplier: laid.multiplier, maxAmount: g.bins.reduce((s, b) => s + BigInt(b.amount), 0n) * laid.multiplier }))],
  };
}

/** 5. THE MONEY: what the dev wallet is owed now (half of all the SOL the position has taken in, less what it already took), paid first by the swap
 *  fees the position has earned (claimed straight into the dev wallet's SOL account; the SWARM side of the fees goes back to the launch key, the sale's
 *  own stock), then by taking the same share of every bin below the active bin (they hold SOL alone) into the same account. */
export function money({ a, active, positions, withdrawn, devShareBps }) {
  const held = L.positionSol(positions, active);
  const due = L.moneyDue({ held, withdrawn, devShareBps });
  const ixs = [];
  if (due.claim || due.bps > 0) {
    ixs.push(C.computeLimit(1_000_000), C.createAtaIdempotent(a.launchKey, a.devWallet, C.P.wsol));
    for (const pos of positions) {
      const lower = pos.index * C.BINS_PER_ARRAY;
      if (due.claim && pos.shares.some((s) => s > 0n)) {
        ixs.push(C.claimFees({ pair: a.pair, owner: a.launchKey, index: pos.index, from: lower, to: lower + C.BINS_PER_ARRAY - 1,
          userTokenX: a.launchSwarm, userTokenY: a.devWsol }));
      }
      const mine = pos.bins.filter((b, i) => pos.shares[i] > 0n && b.binId < active && b.y > 0n).map((b) => b.binId);
      if (!mine.length || due.bps === 0) continue;
      ixs.push(C.removeRange({ pair: a.pair, owner: a.launchKey, index: pos.index, from: Math.min(...mine), to: Math.max(...mine), bps: due.bps,
        userTokenX: a.launchSwarm, userTokenY: a.devWsol }));
    }
  }
  return { name: `money: ${due.due} lamports owed to the dev wallet (${due.fees} of earned fees, then ${due.bps} bps of the SOL bins)`, held, due, ixs };
}

/** A buyer's swap, SOL for SWARM (the devnet rehearsal's test buyer): wrap, swap through the bin arrays from the price up, unwrap what is left. */
export function buy({ a, buyer, lamports, arrays }) {
  const wsol = C.ata(buyer, C.P.wsol), swarm = C.ata(buyer, a.mint);
  return {
    name: `buy: ${lamports} lamports of SOL for SWARM`,
    ixs: [C.computeLimit(1_400_000), C.createAtaIdempotent(buyer, buyer, C.P.wsol), C.transfer(buyer, wsol, lamports), C.syncNative(wsol),
      C.createAtaIdempotent(buyer, buyer, a.mint),
      C.swap({ pair: a.pair, user: buyer, userIn: wsol, userOut: swarm, amountIn: lamports, minOut: 0n, binArrays: arrays }),
      C.closeAccount(wsol, buyer, buyer)],
  };
}
