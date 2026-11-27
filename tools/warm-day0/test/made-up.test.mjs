// States no recording could show (a disabled pool, a flat fee, a stream that can be updated): a real account's bytes with one
// field rewritten at the offset the IDL gives. Everything rewritten here is made up; only the layout is real.
import test from 'node:test';
import assert from 'node:assert/strict';
import { runChecks, PASS, FAIL, UNCHECKED } from '../lib/rows.mjs';
import { parseManifest } from '../lib/manifest.mjs';
import { layoutOf } from '../lib/layout.mjs';
import { CP_AMM_IDL, LOCKER_IDL, STREAMFLOW_SPEC, associatedTokenAddress } from '../lib/programs.mjs';
import { createRpc, fixtureTransport, callKey } from '../lib/rpc.mjs';
import { keyBytes } from '../lib/web3.mjs';
import { noNetwork, fixture } from './helpers.mjs';

noNetwork();

/** Rewrite fields of one recorded account. `base` is 8 for an Anchor account (its discriminator) and 0 otherwise. */
function rewrite(fx, address, layout, base, fields) {
  let done = 0;
  for (const [key, answer] of Object.entries(fx.calls)) {
    const params = JSON.parse(key.slice(key.indexOf(' ') + 1));
    if (params[1]?.dataSlice) continue;
    const slot = key.startsWith('getAccountInfo ') && params[0] === address ? answer.value : key.startsWith('getMultipleAccounts ') && params[0].includes(address) ? answer.value[params[0].indexOf(address)] : null;
    if (!slot) continue;
    const data = Buffer.from(slot.data[0], 'base64');
    for (const [path, value] of Object.entries(fields)) {
      const f = layout.at(path), o = base + f.offset;
      if (Buffer.isBuffer(value)) value.copy(data, o);
      else if (f.type === 'u8' || f.type === 'bool') data.writeUInt8(Number(value), o);
      else if (f.type === 'u16') data.writeUInt16LE(Number(value), o);
      else if (f.type === 'u64') data.writeBigUInt64LE(BigInt(value), o);
      else if (f.type === 'u128') { data.writeBigUInt64LE(BigInt(value) & ((1n << 64n) - 1n), o); data.writeBigUInt64LE(BigInt(value) >> 64n, o + 8); }
      else throw new Error(`cannot write a ${f.type}`);
    }
    slot.data[0] = data.toString('base64'); done++;
  }
  assert.ok(done > 0, `the fixture holds ${address}`);
  return fx;
}
const run = (fx, opts = {}) => runChecks({ rpc: createRpc(fixtureTransport(fx)), mint: fx.meta.mint, manifest: parseManifest(fx.manifest, fx.meta.mint), history: fx.meta.history, ...opts });

const POOL = layoutOf(CP_AMM_IDL, 'Pool'), POSITION = layoutOf(CP_AMM_IDL, 'Position');
const SRA_POOL = '3T1PUGQAq6881XzyX5nUQwk9YHEPxPjAay2gice9KFrZ', SRA_POSITION = 'pUMqtjspFsw15a39Tc6FQWmrREeUbaXvhYxSQhPXsKW';
const pool = fields => run(rewrite(fixture('sra'), SRA_POOL, POOL, 8, fields));

test('row 8, made up: a disabled pool, a pool that has not opened and a whitelisted vault each fail it', async () => {
  const disabled = (await pool({ pool_status: 1 })).rows[7];
  assert.equal(disabled.verdict, FAIL); assert.match(disabled.detail[0], /The pool is disabled: nobody can swap/);
  const early = (await pool({ activation_point: 4102444800n })).rows[7];
  assert.equal(early.verdict, FAIL); assert.match(early.detail[0], /not open yet: it opens at 2100-01-01T00:00:00Z/);
  const listed = (await pool({ whitelisted_vault: keyBytes('BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs') })).rows[7];
  assert.equal(listed.verdict, FAIL); assert.match(listed.detail[0], /whitelisted vault, BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs, may buy before the pool opens/);
  // a pool that counts its opening in slots is compared with the cluster's slot, not its clock
  const bySlot = (await pool({ activation_type: 0, activation_point: 999999999999n })).rows[7];
  assert.equal(bySlot.verdict, FAIL); assert.match(bySlot.detail[0], /it opens at slot 999999999999/);
  assert.equal((await pool({ activation_type: 0, activation_point: 5n })).rows[7].verdict, PASS);
  // a pool for another pair
  const other = (await pool({ token_a_mint: keyBytes('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v') })).rows[7];
  assert.equal(other.verdict, FAIL); assert.match(other.detail[0], /this mint is not in it/);
});

test('row 9, made up: liquidity the position can still withdraw, or that only vests, fails it', async () => {
  const position = fields => run(rewrite(fixture('sra'), SRA_POSITION, POSITION, 8, fields));
  const free = (await position({ unlocked_liquidity: 10n ** 30n })).rows[8];
  assert.equal(free.verdict, FAIL); assert.match(free.detail[0], /can withdraw 0\.31% of its liquidity today/);
  const vesting = (await position({ vested_liquidity: 10n ** 30n })).rows[8];
  assert.equal(vesting.verdict, FAIL); assert.match(vesting.detail[0], /on a vesting lock that ends/);
  const stranger = (await position({ pool: keyBytes('CxKu5DQgzCPGZCwkHF3pCL3ngPWunkEoHCcMmSr65u71') })).rows[8];
  assert.equal(stranger.verdict, FAIL); assert.match(stranger.detail[0], /belongs to another pool/);
  // with no position named: nothing locked fails, part locked is left for a person
  const bare = fx => { fx.manifest = { ...fx.manifest, pool: { address: SRA_POOL } }; return fx; };
  const unlocked = rewrite(bare(fixture('sra')), SRA_POOL, POOL, 8, { permanent_lock_liquidity: 0n });
  const part = rewrite(bare(fixture('sra')), SRA_POOL, POOL, 8, { permanent_lock_liquidity: 10n ** 30n });
  // the recording holds no list of the pool's positions (the real pool is all locked, so the checker never asked for one):
  // the made-up cluster answers with none
  const positionsKey = callKey('getProgramAccounts', ['cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG', { encoding: 'base64', commitment: 'finalized', filters: [{ dataSize: 8 + POSITION.size }, { memcmp: { offset: 8 + POSITION.at('pool').offset, bytes: SRA_POOL } }] }]);
  unlocked.calls[positionsKey] = []; part.calls[positionsKey] = [];
  assert.equal((await run(unlocked)).rows[8].verdict, FAIL);
  const partly = (await run(part)).rows[8];
  assert.equal(partly.verdict, UNCHECKED); assert.match(partly.detail.join(' '), /0\.31% is permanently locked.*Name the house's positions/);
});

test('row 10, made up: a flat fee and a rate limiter fail it; a market cap scheduler that decays passes', async () => {
  const info = fields => { const data = Buffer.alloc(32); const l = layoutOf(CP_AMM_IDL, fields.layout); for (const [k, v] of Object.entries(fields.values)) { const f = l.at(k); if (f.type === 'u8') data.writeUInt8(v, f.offset); else if (f.type === 'u16') data.writeUInt16LE(v, f.offset); else if (f.type === 'u32') data.writeUInt32LE(v, f.offset); else data.writeBigUInt64LE(BigInt(v), f.offset); } return data; };
  const flat = (await pool({ 'pool_fees.base_fee.base_fee_info.data': info({ layout: 'PodAlignedFeeTimeScheduler', values: { cliff_fee_numerator: 10000000, base_fee_mode: 0 } }) })).rows[9];
  assert.equal(flat.verdict, FAIL); assert.match(flat.detail[0], /The base fee is flat at 1%: it does not start high and decay/);
  const limiter = (await pool({ 'pool_fees.base_fee.base_fee_info.data': info({ layout: 'PodAlignedFeeRateLimiter', values: { cliff_fee_numerator: 10000000, base_fee_mode: 2, fee_increment_bps: 100, max_limiter_duration: 600, max_fee_bps: 9900, reference_amount: 1000000000 } }) })).rows[9];
  assert.equal(limiter.verdict, FAIL); assert.match(limiter.detail[0], /rate limiter.*It taxes size, it does not start high and decay/);
  const cap = (await pool({ 'pool_fees.base_fee.base_fee_info.data': info({ layout: 'PodAlignedFeeMarketCapScheduler', values: { cliff_fee_numerator: 500000000, base_fee_mode: 3, number_of_period: 10, sqrt_price_step_bps: 100, scheduler_expiration_duration: 86400, reduction_factor: 45000000 } }) })).rows[9];
  assert.equal(cap.verdict, PASS); assert.match(cap.detail[0], /market cap scheduler, linear\. Starts at 50% and falls toward 5% in 10 steps, one for every 100 bps/);
  assert.equal(cap.data.now, 50000000n, 'its day has passed: the fee is at its floor whatever the price');
  await assert.rejects(pool({ 'pool_fees.base_fee.base_fee_info.data': info({ layout: 'PodAlignedFeeTimeScheduler', values: { cliff_fee_numerator: 1, base_fee_mode: 9 } }) }).then(r => { assert.equal(r.rows[7].verdict, FAIL); assert.match(r.rows[7].detail[0], /base fee mode 9 is not one this layout knows/); throw new Error('read as not a pool'); }), /read as not a pool/);
});

test('row 12, made up: a delegate allowed to claim into any account fails it', async () => {
  const fx = rewrite(fixture('sra'), SRA_POSITION, POSITION, 8, { delegate_permission: Buffer.from([1 << 3, 0, 0, 0]) });
  // the NFT's token account gains a delegate (SPL Token: COption tag at byte 72, the key after it)
  const nftAccount = '71ZTSNYoJgXB5kgk39mvCkMHvo7pyyUNvfRAbVDJfPY9';
  const key = Object.keys(fx.calls).find(k => k.startsWith(`getAccountInfo ["${nftAccount}"`));
  const data = Buffer.from(fx.calls[key].value.data[0], 'base64'); data.writeUInt32LE(1, 72); keyBytes('Boy7pkvoq3ipJ3qfxBzCFeA76x1segrZHEANHBX4XAoe').copy(data, 76); fx.calls[key].value.data[0] = data.toString('base64');
  const twelve = (await run(fx)).rows[11];
  assert.equal(twelve.verdict, FAIL);
  assert.match(twelve.detail.join(' '), /has a delegate, Boy7pkvoq3ipJ3qfxBzCFeA76x1segrZHEANHBX4XAoe, allowed to claim its fees into any account/);
});

const STREAM = layoutOf(STREAMFLOW_SPEC, 'Contract'), ESCROW = layoutOf(LOCKER_IDL, 'VestingEscrow');
const NEST_STREAM = '2Mx2JkbGP25KRBd3kUsoNRdoLBUkTdgNSJrSYrwq77qi', ARENA_LOCK = '77vFUtcMWqAtqDtojH3GABx2ZPHUpsKe9KkzFq45ASrW';

test('row 7, made up: a stream that can be updated, a recipient who may move it, a cancelled or closed lock each fail it', async () => {
  const stream = fields => run(rewrite(fixture('nest-streams'), NEST_STREAM, STREAM, 0, fields)).then(r => r.rows[6]);
  const updatable = await stream({ 'ix.can_update_rate': 1 });
  assert.equal(updatable.verdict, FAIL);
  assert.match(updatable.detail[1], /Fails because the sender \(can_update_rate: the release rate, and through update\(\) the cancel and transfer flags\) can change it/);
  assert.equal(updatable.detail[0], '2 locks named, 1 sound.');
  assert.match((await stream({ 'ix.transferable_by_recipient': 1 })).detail[1], /the recipient can change its recipient/);
  assert.match((await stream({ 'ix.cancelable_by_recipient': 1, 'ix.cancelable_by_sender': 1 })).detail[1], /the sender or the recipient can cancel it/);
  assert.match((await stream({ closed: 1 })).detail[1], /it is closed/);
  assert.match((await stream({ canceled_at: 1790000000n })).detail[1], /it is cancelled at 1790000000/);
  // a sender who may only pause is said, and does not fail the row
  const pausable = await stream({ 'ix.pausable': 1 });
  assert.equal(pausable.verdict, PASS); assert.match(pausable.notes.join(' '), /the sender can pause this stream, which moves every date later/);
  // an escrow that holds less than it owes is not a lock
  const thin = await stream({ 'ix.net_amount_deposited': 56000000000000000n });
  assert.equal(thin.verdict, FAIL); assert.match(thin.detail[1], /its token account holds 55,000,000, less than the 56,000,000 still owed/);
});

test('row 7, made up: each Jupiter Lock mode byte is read for what it is', async () => {
  const lock = fields => run(rewrite(fixture('arena'), ARENA_LOCK, ESCROW, 8, fields)).then(r => r.rows[6]);
  assert.match((await lock({ cancel_mode: 1 })).detail[1], /Fails because the creator can cancel it\./);
  assert.match((await lock({ cancel_mode: 2 })).detail[1], /Fails because the recipient can cancel it\./);
  assert.match((await lock({ update_recipient_mode: 1 })).detail[1], /Fails because the creator can change its recipient\./);
  assert.match((await lock({ update_recipient_mode: 3 })).detail[1], /the creator or the recipient can change its recipient/);
  assert.match((await lock({ cancelled_at: 1790000000n })).detail[1], /it is cancelled at 1790000000/);
  assert.match((await lock({ cancel_mode: 7 })).detail[1], /FAIL, a mode byte this layout does not know/);
  // an escrow for another mint: its token account is that mint's associated account, which the made-up cluster does not hold
  const USDC = 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v';
  const elsewhere = rewrite(fixture('arena'), ARENA_LOCK, ESCROW, 8, { token_mint: keyBytes(USDC) });
  elsewhere.calls[callKey('getAccountInfo', [associatedTokenAddress(ARENA_LOCK, USDC), { encoding: 'base64', commitment: 'finalized' }])] = { context: { slot: 1 }, value: null };
  assert.match((await run(elsewhere)).rows[6].detail[1], /Fails because it locks another mint \(EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\); its token account \S+ does not read as a token account \(no such account\)/);
  assert.equal((await lock({ escrow_bump: 250 })).verdict, PASS, 'a field no rule reads changes nothing');
});
