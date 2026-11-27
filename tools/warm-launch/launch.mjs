// launch.mjs · THE $WARM LAUNCH, its law (Oct 30 2026). His rulings, UTC:
//   Oct 29 19:10:59 "we are going to have 10% sale over 14 days... that is going to seed the liquidity needed"
//   Oct 29 19:12:48 "$WARM is liquid upfront qnd is what the futarchy runs decisions on"
//   Oct 29 20:46:15 "of the supply; the money raised is 50% liquidity pool and 50% dev wallet (which is to say 5% of the token is sold to get cash to pay
//                   for token credits in ai)"; 20:42:12 the remaining 90% and the liquidity pool are the swarm's
//   Oct 29 20:55:03 the Prompt-to-Mine rebate is paid "From the 90%": nothing is minted on top, the supply is fixed
//   Oct 29 23:46:40 the window ends Nov 15 00:00 UTC, as Breakpoint opens
//   Oct 30 00:13:26 "Sale first, seal later": the 90% waits in his Squads until Poleis passes its outside review
//   Oct 30 00:2x    "Yes, Meteora": a DLMM pair seeded with the 10% alone, no SOL up front, so the price rises as people buy
// What follows from them, as numbers and nothing else: the supply minted once and split 90/10 to the base unit; the 10% laid into the pool in equal
// tranches, each a window of bins just above the price of its moment (the 10% "sold linearly", at the market, never on a ladder fixed in advance); and
// the money: half of every lamport the position takes in is the dev wallet's, the other half stays in the pool. Pure: no key, no network, no file.
import { BINS_PER_ARRAY, binArrayIndex } from './chain.mjs';

export const U64_MAX = (1n << 64n) - 1n;
export const U32_MAX = 4_294_967_295n;
export const BPS = 10_000n;
export const CLOSE = '2026-11-15T00:00:00Z';                      // his 23:46:40: the window ends as Breakpoint opens
export const SOL_DECIMALS = 9;
// 9 decimals: Space Router's ledger owes $WARM to the nano (space-tanks.mjs warmOwed), so one nano-$WARM there is one base unit here, and a rebate at
// 1/1024 of its first rate (year five) is still exact; SOL has 9 too, so the pool's price per base unit is the price in SOL with no shift. The u64 holds
// 18.4 billion tokens at 9 decimals; a supply above that is refused here (6 decimals would be the road then).
export const DECIMALS = 9;

const isPlaceholder = (v) => typeof v === 'string' && /^PLACEHOLDER/.test(v);
/** Every parameter that is still his: a `PLACEHOLDER…` string. A set with any of them builds for reading and never for signing. */
export const placeholdersIn = (params) => Object.entries(flat(params)).filter(([, v]) => isPlaceholder(v)).map(([k]) => k);
function flat(o, pre = '') {
  return Object.entries(o).reduce((acc, [k, v]) => (v && typeof v === 'object' && !Array.isArray(v) ? { ...acc, ...flat(v, `${pre}${k}.`) } : { ...acc, [`${pre}${k}`]: v }), {});
}

/** The parameters read strictly: the numbers a launch is made of, each checked, the supply in base units. */
export function readParams(p) {
  const out = { ...p };
  const need = (cond, what) => { if (!cond) throw new Error(`params: ${what}`); };
  out.decimals = p.decimals ?? DECIMALS;
  need(Number.isInteger(out.decimals) && out.decimals >= 0 && out.decimals <= 9, 'decimals is 0 to 9');
  need(typeof p.name === 'string' && typeof p.symbol === 'string', 'name and symbol');
  if (!isPlaceholder(p.supply)) {
    need(/^[1-9][0-9]*$/.test(String(p.supply)), 'supply is a whole number of tokens');
    out.supplyBase = BigInt(p.supply) * 10n ** BigInt(out.decimals);
    need(out.supplyBase <= U64_MAX, `a supply of ${p.supply} overflows a u64 at ${out.decimals} decimals`);
  }
  need(p.saleBps === 1000 && p.treasuryBps === 9000, 'the split is his: 90% to the treasury, 10% to the sale (20:46:15)');
  need(p.devShareBps === 5000, 'the money is his: half to the dev wallet, half stays in the pool (20:46:15)');
  // The dev wallet is his Squads vault, the one that holds the 90%, unless one is named apart: "dude, i will bring the proceeds of the sale into my
  // squads wallet... u get what i mean or not?" (Oct 30 2026 02:58:56 UTC)
  out.devWallet = p.devWallet ?? p.treasuryOwner;
  need(Number.isInteger(p.binStep) && p.binStep >= 1 && p.binStep <= 400, 'binStep is 1 to 400 bps (MAX_BIN_STEP)');
  need(Number.isInteger(p.feeBps) && p.feeBps >= 1 && p.feeBps <= 1000, 'feeBps is 1 to 1000');
  if (!isPlaceholder(p.startPrice)) need(Number(p.startPrice) > 0, 'startPrice is SOL per token, above zero');
  if (!isPlaceholder(p.windowRatio)) need(Number(p.windowRatio) > 1, 'windowRatio is above 1');
  need(Number.isInteger(p.trancheHours) && p.trancheHours >= 1, 'trancheHours is a whole number of hours');
  need(!Number.isNaN(Date.parse(p.opening)), 'opening is an ISO time');
  out.close = p.close ?? CLOSE;
  need(Date.parse(out.close) > Date.parse(p.opening), 'the opening comes before the close');
  return out;
}

/** The split, exact to the base unit: the treasury takes 90%, the sale the rest, so the two always sum to the supply. */
export function split(supplyBase) {
  const sale = (supplyBase * 1000n) / BPS;
  return { treasury: supplyBase - sale, sale };
}

// ---- prices and bins: a bin's price is (1 + binStep/10000)^id, in Y base units (lamports) per X base unit ----
export const perLamport = (uiPrice, decimals) => Number(uiPrice) * 10 ** (SOL_DECIMALS - decimals);
export const binPrice = (id, binStep) => (1 + binStep / 10_000) ** id;
export const uiPriceOfBin = (id, binStep, decimals) => binPrice(id, binStep) / 10 ** (SOL_DECIMALS - decimals);
/** The bin whose price is at or below the price named: the pool opens there, and the first tranche starts one bin above it. */
export function binOfPrice(uiPrice, binStep, decimals) {
  const id = Math.floor(Math.log(perLamport(uiPrice, decimals)) / Math.log(1 + binStep / 10_000) + 1e-9);
  if (!Number.isFinite(id) || Math.abs(id) > 443_636) throw new Error(`no bin for ${uiPrice}`);
  return id;
}
/** How many bins a window spans to reach `ratio` times its first price. */
export const windowBins = (ratio, binStep) => Math.ceil(Math.log(Number(ratio)) / Math.log(1 + binStep / 10_000) - 1e-9);

// ---- the drip ----
/** The tranches: one at the opening, then one every trancheHours while before the close; the 10% split equally, to the base unit. */
export function schedule({ opening, close = CLOSE, trancheHours, sale }) {
  const t0 = Date.parse(opening), end = Date.parse(close), step = trancheHours * 3_600_000;
  const times = [];
  for (let t = t0; t < end; t += step) times.push(new Date(t).toISOString());
  const n = BigInt(times.length), each = sale / n, extra = sale % n;
  return times.map((at, k) => ({ k, at, amount: each + (BigInt(k) < extra ? 1n : 0n) }));
}

/** One tranche laid as SWARM alone over bins (active, active + width]: equal amounts, compressed as the program takes them (a u32 times a
 *  power-of-ten multiplier). What the compression cannot hold, under one multiplier, rides to the next tranche: nothing is lost or invented. */
export function layTranche({ active, width, amount }) {
  if (amount <= 0n) return { bins: [], multiplier: 1n, laid: 0n, carry: amount };
  let multiplier = 1n;
  while ((amount / multiplier) / BigInt(width) + 1n > U32_MAX) multiplier *= 10n;
  const units = amount / multiplier, each = units / BigInt(width), extra = units % BigInt(width);
  const bins = [];
  for (let i = 0; i < width; i++) {
    const u = each + (BigInt(i) < extra ? 1n : 0n);
    if (u > 0n) bins.push({ binId: active + 1 + i, amount: Number(u) });
  }
  const laid = units * multiplier;
  return { bins, multiplier, laid, carry: amount - laid };
}
/** A laid tranche grouped by position: one position is one bin array of 70 bins. */
export function byPosition(bins) {
  const groups = new Map();
  for (const b of bins) { const i = binArrayIndex(b.binId); if (!groups.has(i)) groups.set(i, []); groups.get(i).push(b); }
  return [...groups.entries()].sort((a, b) => a[0] - b[0]).map(([index, list]) => ({ index, lower: index * BINS_PER_ARRAY, bins: list }));
}

// ---- the money ----
/** The position's SOL, read from its bins: its share of each bin's Y (bin.y × share / supply), split at the active bin. Below the active bin a
 *  bin holds Y alone and can be withdrawn without touching SWARM; the active bin may hold both and is never withdrawn from. And the swap fees it has
 *  earned and not yet claimed, as the SDK counts them (getClaimableSwapFee): pending, plus (share >> 64) × (stored − complete) >> 64. */
export function positionSol(positions, activeId) {
  let below = 0n, active = 0n, swarm = 0n, feeY = 0n, feeX = 0n;
  const binsBelow = [];
  for (const { bins, shares, fees = [] } of positions) {
    bins.forEach((bin, i) => {
      const share = shares[i], f = fees[i];
      if (f) {
        feeY += f.yPending + (share > 0n ? ((share >> 64n) * (bin.feeYStored - f.yComplete)) >> 64n : 0n);
        feeX += f.xPending + (share > 0n ? ((share >> 64n) * (bin.feeXStored - f.xComplete)) >> 64n : 0n);
      }
      if (!share || bin.supply === 0n) return;
      const y = (bin.y * share) / bin.supply, x = (bin.x * share) / bin.supply;
      swarm += x;
      if (bin.binId < activeId) { below += y; if (y > 0n) binsBelow.push(bin.binId); } else if (bin.binId === activeId) active += y;
    });
  }
  return { below, active, total: below + active, swarm, feeY, feeX, binsBelow };
}
/** What the dev wallet is owed now: half of every lamport the position has taken in (the SOL in its bins, the swap fees it has earned and not claimed,
 *  and what was already paid out), less what was paid. It is paid from the earned fees first, all of them when they fit in what is owed, then from the
 *  SOL bins by one share of each: so the pool keeps half of everything taken in as its liquidity. Sellers who take SOL back out lower what is owed,
 *  never below zero; the fees are left in the position whenever they are more than is owed. */
export function moneyDue({ held, withdrawn, devShareBps = 5000 }) {
  const fees = held.feeY ?? 0n;
  const takenIn = held.total + fees + withdrawn;
  const owed = (takenIn * BigInt(devShareBps)) / BPS;
  const due = owed > withdrawn ? owed - withdrawn : 0n;
  const claim = fees > 0n && fees <= due;
  const fromBins = claim ? due - fees : due;
  const bps = held.below > 0n ? Number((fromBins * BPS) / held.below > BPS ? BPS : (fromBins * BPS) / held.below) : 0;
  return { takenIn, owed, due, claim, fees: claim ? fees : 0n, fromBins, bps };
}
