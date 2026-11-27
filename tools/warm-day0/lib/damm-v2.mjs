// A Meteora DAMM v2 pool and its positions, read from their bytes. Layouts are computed from idl/cp_amm.json; what the IDL
// does not carry (the enums, the fee formulas, who may swap when) is from the program source named in idl/sources.json.
import { layoutOf, view, discriminator, accountIndexes, pdaOf } from './layout.mjs';
import { CP_AMM_IDL as IDL, DAMM_V2_PROGRAM } from './programs.mjs';
import { base58Decode, b58, DEFAULT_PUBKEY } from './web3.mjs';
import { decodeTokenAccount } from './spl.mjs';

const POOL = layoutOf(IDL, 'Pool');
const POSITION = layoutOf(IDL, 'Position');
const POOL_DISC = discriminator(IDL, 'accounts', 'Pool');
const POSITION_DISC = discriminator(IDL, 'accounts', 'Position');
const CLAIM_DISC = discriminator(IDL, 'instructions', 'claim_position_fee');
const CLAIM_AT = accountIndexes(IDL, 'claim_position_fee');
const CLAIM_EVENT = layoutOf(IDL, 'EvtClaimPositionFee', { borsh: true });
const CLAIM_EVENT_DISC = discriminator(IDL, 'events', 'EvtClaimPositionFee');

const constant = name => { const c = IDL.constants.find(x => x.name === name); if (!c) throw new Error(`the IDL has no constant ${name}`); return BigInt(c.value); };
export const FEE_DENOMINATOR = constant('FEE_DENOMINATOR');
const MAX_BASIS_POINT = constant('MAX_BASIS_POINT');
const MAX_FEE_NUMERATOR = [constant('MAX_FEE_NUMERATOR_V0'), constant('MAX_FEE_NUMERATOR_V1')];

// state/fee.rs BaseFeeMode, in declaration order. The byte sits where each of the three layouts puts base_fee_mode.
const SCHEDULERS = ['PodAlignedFeeTimeScheduler', 'PodAlignedFeeRateLimiter', 'PodAlignedFeeMarketCapScheduler'].map(n => layoutOf(IDL, n));
const MODE_AT = SCHEDULERS[0].at('base_fee_mode').offset;
if (SCHEDULERS.some(l => l.at('base_fee_mode').offset !== MODE_AT)) throw new Error('the base-fee layouts disagree on where the mode byte is');
const BASE_FEE_MODES = Object.freeze([
  { name: 'time scheduler, linear', layout: SCHEDULERS[0], kind: 'time', curve: 'linear' },
  { name: 'time scheduler, exponential', layout: SCHEDULERS[0], kind: 'time', curve: 'exponential' },
  { name: 'rate limiter (deprecated)', layout: SCHEDULERS[1], kind: 'rate-limiter', curve: null },
  { name: 'market cap scheduler, linear', layout: SCHEDULERS[2], kind: 'market-cap', curve: 'linear' },
  { name: 'market cap scheduler, exponential', layout: SCHEDULERS[2], kind: 'market-cap', curve: 'exponential' },
]);
export const COLLECT_FEE_MODES = Object.freeze(['both tokens', 'token B only (the quote)', 'token B only, compounding']);   // state/pool.rs CollectFeeMode
export const POOL_TYPES = Object.freeze(['permissionless', 'customizable']);                                               // state/pool.rs PoolType
// state/position.rs PositionDelegatePermission: bit 3 lets a delegate claim fees to any account, bit 4 only to the owner's.
const DELEGATE_CLAIM_ANYWHERE = 1 << 3, DELEGATE_CLAIM_TO_OWNER = 1 << 4;

const ONE_Q64 = 1n << 64n, U128_MAX = (1n << 128n) - 1n;

/** math/fee_math.rs pow: base^exp in Q64.64, by squaring over nineteen bits. */
function powQ64(base, exp) {
  if (exp === 0) return ONE_Q64;
  if (exp >= 0x80000) return null;
  let squared = base, result = ONE_Q64, invert = false;
  if (squared >= result) { squared = U128_MAX / squared; invert = true; }
  for (let bit = 0; bit < 19; bit++) {
    if (exp & (1 << bit)) result = (result * squared) >> 64n;
    squared = (squared * squared) >> 64n;
  }
  if (result === 0n) return null;
  return invert ? U128_MAX / result : result;
}

/** The base fee after `period` steps: the program's own arithmetic (fee_time_scheduler.rs, fee_math.rs get_fee_in_period). */
export function feeAtPeriod(curve, cliff, reduction, period) {
  if (curve === 'linear') return cliff - reduction * BigInt(period);
  if (reduction === 0n) return cliff;
  const base = ONE_Q64 - ((reduction << 64n) / MAX_BASIS_POINT);
  const p = powQ64(base, period);
  if (p === null) throw new Error('fee schedule overflows');
  return (p * cliff) >> 64n;
}


export function decodePool(address, account) {
  if (!account) throw new Error('no such account');
  if (account.owner !== DAMM_V2_PROGRAM) throw new Error(`owned by ${account.owner}, not Meteora DAMM v2`);
  if (account.data.length !== 8 + POOL.size || !account.data.subarray(0, 8).equals(POOL_DISC)) throw new Error('an account of DAMM v2 that is not a pool');
  const v = view(POOL, account.data, 8);
  const info = v.get('pool_fees.base_fee.base_fee_info.data');
  const modeByte = info[MODE_AT];
  const mode = BASE_FEE_MODES[modeByte];
  if (!mode) throw new Error(`base fee mode ${modeByte} is not one this layout knows`);
  const s = view(mode.layout, info, 0);
  const baseFee = { mode: modeByte, name: mode.name, kind: mode.kind, curve: mode.curve, cliff: s.get('cliff_fee_numerator') };
  if (mode.kind === 'time') Object.assign(baseFee, { periods: s.get('number_of_period'), frequency: s.get('period_frequency'), reduction: s.get('reduction_factor') });
  if (mode.kind === 'market-cap') Object.assign(baseFee, { periods: s.get('number_of_period'), stepBps: s.get('sqrt_price_step_bps'), expiresAfter: s.get('scheduler_expiration_duration'), reduction: s.get('reduction_factor') });
  if (mode.kind === 'rate-limiter') Object.assign(baseFee, { incrementBps: s.get('fee_increment_bps'), limiterDuration: s.get('max_limiter_duration'), maxFeeBps: s.get('max_fee_bps'), referenceAmount: s.get('reference_amount') });
  const whitelisted = v.get('whitelisted_vault');
  const feeVersion = v.get('fee_version');
  return {
    address,
    tokenAMint: v.get('token_a_mint'), tokenBMint: v.get('token_b_mint'), tokenAVault: v.get('token_a_vault'), tokenBVault: v.get('token_b_vault'),
    whitelistedVault: whitelisted === DEFAULT_PUBKEY ? null : whitelisted,
    liquidity: v.get('liquidity'), permanentLockLiquidity: v.get('permanent_lock_liquidity'),
    activationPoint: v.get('activation_point'), activationType: v.get('activation_type'), status: v.get('pool_status'),
    collectFeeMode: v.get('collect_fee_mode'), poolType: v.get('pool_type'), feeVersion, layoutVersion: v.get('layout_version'),
    creator: v.get('creator'),
    protocolFeePercent: v.get('pool_fees.protocol_fee_percent'), referralFeePercent: v.get('pool_fees.referral_fee_percent'),
    compoundingFeeBps: v.get('pool_fees.compounding_fee_bps'), dynamicFee: v.get('pool_fees.dynamic_fee.initialized') !== 0,
    initSqrtPrice: v.get('pool_fees.init_sqrt_price'), sqrtPrice: v.get('sqrt_price'),
    maxFeeNumerator: MAX_FEE_NUMERATOR[feeVersion] ?? null,
    totalLpAFee: v.get('metrics.total_lp_a_fee'), totalLpBFee: v.get('metrics.total_lp_b_fee'),
    totalProtocolAFee: v.get('metrics.total_protocol_a_fee'), totalProtocolBFee: v.get('metrics.total_protocol_b_fee'),
    totalPosition: v.get('metrics.total_position'),
    baseFee,
  };
}

/** The pool's clock: its activation is a slot or a unix time, and so is every period of its fee schedule. */
export function poolNow(pool, clock) { return BigInt(pool.activationType === 0 ? clock.slot : clock.time); }
export function poolUnit(pool) { return pool.activationType === 0 ? 'slot' : 'second'; }

/** The fee schedule in the pool's own terms: where it starts, where it ends, when, and what it is now. */
export function feeSchedule(pool, clock) {
  const f = pool.baseFee, now = poolNow(pool, clock), unit = poolUnit(pool);
  const out = { kind: f.kind, curve: f.curve, name: f.name, start: f.cliff, unit, activation: pool.activationPoint, decays: false };
  if (f.kind === 'time') {
    const flat = f.periods === 0 || f.frequency === 0n || f.reduction === 0n;
    out.end = flat ? f.cliff : feeAtPeriod(f.curve, f.cliff, f.reduction, f.periods);
    out.periods = f.periods; out.frequency = f.frequency; out.reduction = f.reduction;
    out.endsAt = pool.activationPoint + BigInt(f.periods) * f.frequency;
    out.decays = !flat && out.end < out.start;
    if (flat) out.now = f.cliff;
    else if (now < pool.activationPoint) out.now = null;          // not open yet; before activation only a whitelisted vault trades
    else { const passed = (now - pool.activationPoint) / f.frequency; out.now = feeAtPeriod(f.curve, f.cliff, f.reduction, Number(passed < BigInt(f.periods) ? passed : BigInt(f.periods))); }
  } else if (f.kind === 'market-cap') {
    const flat = f.periods === 0 || f.reduction === 0n || f.stepBps === 0;
    out.end = flat ? f.cliff : feeAtPeriod(f.curve, f.cliff, f.reduction, f.periods);
    out.periods = f.periods; out.stepBps = f.stepBps; out.reduction = f.reduction;
    out.endsAt = pool.activationPoint + BigInt(f.expiresAfter);
    out.decays = !flat && out.end < out.start;
    if (flat) out.now = f.cliff;
    else if (now < pool.activationPoint) out.now = null;
    else if (now > out.endsAt) out.now = out.end;
    else {
      let passed = 0n;
      if (pool.sqrtPrice > pool.initSqrtPrice && pool.initSqrtPrice > 0n) passed = (pool.sqrtPrice - pool.initSqrtPrice) * MAX_BASIS_POINT / pool.initSqrtPrice / BigInt(f.stepBps);
      out.now = feeAtPeriod(f.curve, f.cliff, f.reduction, Number(passed < BigInt(f.periods) ? passed : BigInt(f.periods)));
    }
  } else {
    out.end = f.cliff; out.now = f.cliff;
    out.incrementBps = f.incrementBps; out.limiterDuration = f.limiterDuration; out.maxFeeBps = f.maxFeeBps; out.referenceAmount = f.referenceAmount;
    out.endsAt = pool.activationPoint + BigInt(f.limiterDuration);
  }
  return out;
}

/** A fee numerator as a percentage string: the denominator is the program's own, 1e9. */
export function feePercent(numerator) {
  if (numerator === null || numerator === undefined) return 'n/a';
  const scaled = numerator * 1_000_000n / FEE_DENOMINATOR;       // parts per million
  const whole = scaled / 10_000n, frac = (scaled % 10_000n).toString().padStart(4, '0').replace(/0+$/, '');
  return `${whole}${frac ? '.' + frac : ''}%`;
}

export function decodePosition(address, account) {
  if (!account) throw new Error('no such account');
  if (account.owner !== DAMM_V2_PROGRAM) throw new Error(`owned by ${account.owner}, not Meteora DAMM v2`);
  if (account.data.length !== 8 + POSITION.size || !account.data.subarray(0, 8).equals(POSITION_DISC)) throw new Error('an account of DAMM v2 that is not a position');
  const v = view(POSITION, account.data, 8);
  const permission = v.get('delegate_permission');
  return {
    address, pool: v.get('pool'), nftMint: v.get('nft_mint'),
    unlocked: v.get('unlocked_liquidity'), vested: v.get('vested_liquidity'), permanentLocked: v.get('permanent_locked_liquidity'),
    feeAPending: v.get('fee_a_pending'), feeBPending: v.get('fee_b_pending'),
    claimedA: v.get('metrics.total_claimed_a_fee'), claimedB: v.get('metrics.total_claimed_b_fee'),
    delegateMayClaimAnywhere: (permission & DELEGATE_CLAIM_ANYWHERE) !== 0, delegateMayClaimToOwner: (permission & DELEGATE_CLAIM_TO_OWNER) !== 0,
  };
}

/** Every position of a pool, asked of the program's accounts by size and by the pool field. */
export async function positionsOf(rpc, poolAddress) {
  const found = await rpc.programAccounts(DAMM_V2_PROGRAM, [{ dataSize: 8 + POSITION.size }, { memcmp: { offset: 8 + POSITION.at('pool').offset, bytes: poolAddress } }]);
  return found.map(a => decodePosition(a.address, a));
}

/** The token account the program creates to hold a position's NFT, and the position's own address, both from the IDL's seeds. */
export function positionNftAccount(nftMint) { return pdaOf(IDL, 'create_position', 'position_nft_account', { position_nft_mint: nftMint }); }
export function positionAddress(nftMint) { return pdaOf(IDL, 'create_position', 'position', { position_nft_mint: nftMint }); }

/**
 * Who holds a position: the token account that holds its one NFT, that account's owner, and any delegate on it.
 * The NFT is looked for where the program put it; only when it has been moved is the cluster asked where it went.
 */
export async function positionHolder(rpc, nftMint) {
  const home = positionNftAccount(nftMint);
  const at = await rpc.account(home);
  if (at) { const t = decodeTokenAccount(at); if (t.mint === nftMint && t.amount === 1n) return { tokenAccount: home, owner: t.owner, delegate: t.delegate }; }
  let largest;
  try { largest = await rpc.largestAccounts(nftMint); } catch { return null; }
  const hit = largest.find(a => a.amount === '1');
  if (!hit) return null;
  const token = decodeTokenAccount(await rpc.account(hit.address));
  return { tokenAccount: hit.address, owner: token.owner, delegate: token.delegate };
}

function flatInstructions(tx) {
  const inner = new Map((tx.meta?.innerInstructions || []).map(g => [g.index, g.instructions]));
  return tx.transaction.message.instructions.flatMap((ix, i) => [ix, ...(inner.get(i) || [])]);
}

/**
 * Who owns each token account a transaction names, from what the transaction itself says: its token balance records, an
 * associated-account creation, an initialization or a close. `closedTo` holds where a closed account's lamports went.
 */
export function tokenOwnersIn(tx) {
  const keys = tx.transaction.message.accountKeys.map(k => (typeof k === 'string' ? k : k.pubkey));
  const owners = new Map(), closedTo = new Map();
  for (const b of [...(tx.meta?.preTokenBalances || []), ...(tx.meta?.postTokenBalances || [])]) if (b.owner) owners.set(keys[b.accountIndex], b.owner);
  for (const ix of flatInstructions(tx)) {
    const p = ix.parsed;
    if (!p || typeof p !== 'object' || !p.info) continue;
    if (ix.program === 'spl-associated-token-account' && p.info.account && p.info.wallet) owners.set(p.info.account, p.info.wallet);
    if ((ix.program === 'spl-token' || ix.program === 'spl-token-2022') && /^initializeAccount/.test(p.type) && p.info.account && p.info.owner) owners.set(p.info.account, p.info.owner);
    if ((ix.program === 'spl-token' || ix.program === 'spl-token-2022') && p.type === 'closeAccount' && p.info.account) {
      closedTo.set(p.info.account, p.info.destination);
      if (!owners.has(p.info.account) && p.info.owner) owners.set(p.info.account, p.info.owner);
    }
  }
  return { owners, closedTo, keys };
}

/** Every fee claim on a position in its newest `limit` transactions: who signed, where each side was paid, how much. */
export async function claimsOf(rpc, positionAddress, limit) {
  const sigs = await rpc.signatures(positionAddress, limit);
  const claims = [];
  let read = 0;
  for (const s of sigs) {
    if (s.err) continue;
    const tx = await rpc.transaction(s.signature);
    read++;
    if (!tx || tx.meta?.err) continue;
    const { owners, closedTo } = tokenOwnersIn(tx);
    const events = [];
    const found = [];
    for (const ix of flatInstructions(tx)) {
      if (ix.programId !== DAMM_V2_PROGRAM || typeof ix.data !== 'string') continue;
      const data = base58Decode(ix.data);
      if (data.length >= 8 && data.subarray(0, 8).equals(CLAIM_DISC) && ix.accounts[CLAIM_AT.position] === positionAddress) {
        found.push({ signer: ix.accounts[CLAIM_AT.signer], nftAccount: ix.accounts[CLAIM_AT.position_nft_account], toA: ix.accounts[CLAIM_AT.token_a_account], toB: ix.accounts[CLAIM_AT.token_b_account] });
      } else if (data.length === 16 + CLAIM_EVENT.size && data.subarray(8, 16).equals(CLAIM_EVENT_DISC)) {
        const e = view(CLAIM_EVENT, data, 16);
        if (e.get('position') === positionAddress) events.push({ tag: data.subarray(0, 8).toString('hex'), holder: e.get('owner'), feeA: e.get('fee_a_claimed'), feeB: e.get('fee_b_claimed') });
      }
    }
    found.forEach((c, i) => {
      const e = events[i] || null;
      claims.push({
        signature: s.signature, slot: tx.slot, time: tx.blockTime ?? null, ...c,
        holder: e?.holder ?? null, feeA: e?.feeA ?? null, feeB: e?.feeB ?? null, eventTag: e?.tag ?? null,
        toAOwner: owners.get(c.toA) ?? null, toBOwner: owners.get(c.toB) ?? null,
        toAClosedTo: closedTo.get(c.toA) ?? null, toBClosedTo: closedTo.get(c.toB) ?? null,
      });
    });
  }
  return { claims, signatures: sigs.length, read, truncated: sigs.length >= limit };
}

export { POOL, POSITION, b58 };
