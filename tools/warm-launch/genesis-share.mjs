// genesis-share.mjs · THE GENESIS SHARES THE TENTH (Nov 19 2026): at the close the 10% of $WARM is the pre-depositors', each in proportion to the USDC
// they put in before it. His words (UTC): Oct 30 12:54 "10% $WARM is sold until breakpoint begins to seed liquidity, this is to say, we should formulate
// the thing as "Pre-Deposits for the Space Router available""; Oct 17 01:21 "after that the $WARM swarm is geenesissn"; Nov 19 "we gotta mint $WARM and
// we gotta get going with our ecosystem", then "build as if we were launching". A pre-deposit is one the deposit door counted (its row), held to its own
// transaction by the door's own law (verifyDeposit): never the door's word alone. Pure: nothing here reads a key or a cluster.
import { verifyDeposit } from './vendor/space-router-deposit.mjs';
import * as C from './chain.mjs';
/** countedOf(rows, txs, { mint, treasury, fee, before }): each row of the door ({ signature, payer }) with its transaction (txs, by signature) counts when
 *  the door's law holds it (into the treasury, of the mint), it was paid before the genesis instant (ms), and by the row's payer, or by the house's fee
 *  wallet for the flask the row names (THE HOUSE'S USDC FILLS A FLASK). Every other row is said with why, never counted; a signature counts once. */
export function countedOf(rows, txs, { mint, treasury, fee, before }) {
  const counted = [], refused = [], seen = new Set();
  for (const r of rows) {
    if (seen.has(r.signature)) continue; seen.add(r.signature);
    let d; try { d = verifyDeposit(txs.get(r.signature), { mint, treasury }); } catch (e) { refused.push({ signature: r.signature, why: e.message }); continue; }
    if (d.paidAt >= before) refused.push({ signature: r.signature, why: 'paid at or after the genesis instant' });
    else if (d.payer !== r.payer && d.payer !== fee) refused.push({ signature: r.signature, why: `paid by ${d.payer}, not by the row's ${r.payer}` });
    else counted.push({ signature: r.signature, payer: r.payer, amount: BigInt(d.amount), paidAt: d.paidAt });
  }
  return { counted, refused };
}

/** sharesOf(deposits, tenth): the tenth split among the payers in proportion to what each put in (summed by payer): each floor, then the units the floors
 *  leave (fewer than the payers) given one each to the largest remainders, ties to the larger deposit then the smaller address, so they sum to the tenth. */
export function sharesOf(deposits, tenth) {
  const by = new Map();
  for (const d of deposits) by.set(d.payer, (by.get(d.payer) ?? 0n) + BigInt(d.amount));
  const total = [...by.values()].reduce((s, v) => s + v, 0n);
  if (total === 0n) return [];
  const rows = [...by].map(([payer, put]) => ({ payer, put, share: (tenth * put) / total, rem: (tenth * put) % total }));
  let left = tenth - rows.reduce((s, r) => s + r.share, 0n);
  const cmp = (x, y) => (x > y ? -1 : x < y ? 1 : 0);
  for (const r of [...rows].sort((a, b) => cmp(a.rem, b.rem) || cmp(a.put, b.put) || (a.payer < b.payer ? -1 : 1))) { if (left === 0n) break; r.share += 1n; left -= 1n; }
  return rows.map(({ payer, put, share }) => ({ payer, put, share })).sort((a, b) => cmp(a.share, b.share) || (a.payer < b.payer ? -1 : 1));
}

/** distribution({ a, shares, decimals, budget }): out of the launch key's tenth, each payer's account made when it is not (idempotent) and their share sent
 *  into it, packed into as few transactions as fit a transaction's 1232 bytes with the budget's instructions; the launch key pays for and signs each one. */
export function distribution({ a, shares, decimals, budget = [] }) {
  const ixsOf = (s) => [C.createAtaIdempotent(a.launchKey, s.payer, a.mint), C.transferChecked(a.launchSwarm, a.mint, C.ata(s.payer, a.mint), a.launchKey, s.share, decimals)];
  const fits = (rows) => C.compile([...budget, ...rows.flatMap(ixsOf)], a.launchKey).size <= C.MAX_TX;
  const txs = [];
  for (const s of shares) { const last = txs.at(-1); if (last && fits([...last, s])) last.push(s); else txs.push([s]); }
  return txs.map((rows) => ({ rows, ixs: rows.flatMap(ixsOf) }));
}
