// The layouts: computed from the IDLs, and held to what the programs' own sources and SDKs state.
import test from 'node:test';
import assert from 'node:assert/strict';
import { layoutOf, view, discriminator, accountIndexes, pdaOf } from '../lib/layout.mjs';
import { CP_AMM_IDL, LOCKER_IDL, STREAMFLOW_SPEC, SOURCES, vendoredSha256, programDataAddress, associatedTokenAddress, TOKEN_PROGRAM } from '../lib/programs.mjs';
import { positionAddress } from '../lib/damm-v2.mjs';
import { base58Decode, b58, isAddress, PublicKey } from '../lib/web3.mjs';
import { noNetwork } from './helpers.mjs';

noNetwork();

test('the vendored IDLs are byte for byte what their sources published', () => {
  const [sdk] = SOURCES.programs['meteora-damm-v2'].layout, [chain] = SOURCES.programs['jupiter-lock'].layout;
  assert.equal(vendoredSha256('cp_amm.json'), sdk.sha256);
  assert.equal(vendoredSha256('locker.json'), chain.sha256);
  assert.equal(CP_AMM_IDL.metadata.version, '0.2.4');
  assert.equal(LOCKER_IDL.metadata.version, '0.4.0');
  assert.equal(CP_AMM_IDL.address, SOURCES.programs['meteora-damm-v2'].address);
  assert.equal(LOCKER_IDL.address, SOURCES.programs['jupiter-lock'].address);
});

test('computed sizes equal the sizes Meteora\'s source asserts (const_assert_eq in state/*.rs)', () => {
  for (const [name, size] of [['Pool', 1104], ['Position', 400], ['PoolFeesStruct', 160], ['BaseFeeStruct', 40], ['PoolMetrics', 80], ['RewardInfo', 192], ['UserRewardInfo', 48], ['PositionMetrics', 16], ['BaseFeeInfo', 32], ['PodAlignedFeeTimeScheduler', 32], ['PodAlignedFeeRateLimiter', 32], ['PodAlignedFeeMarketCapScheduler', 32]]) {
    assert.equal(layoutOf(CP_AMM_IDL, name).size, size, name);
  }
  assert.equal(layoutOf(LOCKER_IDL, 'VestingEscrow').size, 288);
  for (const n of ['PodAlignedFeeTimeScheduler', 'PodAlignedFeeRateLimiter', 'PodAlignedFeeMarketCapScheduler']) assert.equal(layoutOf(CP_AMM_IDL, n).at('base_fee_mode').offset, 8, 'BaseFeeInfo::BASE_FEE_MODE_OFFSET');
});

test('the Streamflow layout lands on the offsets its own SDK states, inside the 1104 bytes it allots', () => {
  const contract = layoutOf(STREAMFLOW_SPEC, 'Contract');
  for (const [field, offset] of Object.entries(STREAMFLOW_SPEC.sdkOffsets)) if (field !== 'about') assert.equal(contract.at(field).offset, offset, field);
  assert.ok(contract.size <= STREAMFLOW_SPEC.accountSize);
  assert.equal(contract.at('ix.start_time').offset, 409);
});

test('a layout with a field of variable size is refused, never guessed', () => {
  assert.throws(() => layoutOf(LOCKER_IDL, 'VestingEscrowMetadata'), /not marked fixed-size/);
  assert.throws(() => layoutOf(LOCKER_IDL, 'VestingEscrowMetadata', { borsh: true }), /no fixed size/);
  assert.throws(() => layoutOf(CP_AMM_IDL, 'NoSuchType'), /does not define/);
  assert.throws(() => layoutOf(CP_AMM_IDL, 'Pool').at('partner'), /no field partner/);
  const event = layoutOf(CP_AMM_IDL, 'EvtClaimPositionFee', { borsh: true });
  assert.deepEqual(event.fields.map(f => [f.path, f.offset]), [['pool', 0], ['position', 32], ['owner', 64], ['fee_a_claimed', 96], ['fee_b_claimed', 104]]);
});

test('a view reads every primitive and refuses to read past the end', () => {
  const idl = { types: [{ name: 'T', serialization: 'packed', type: { kind: 'struct', fields: [{ name: 'a', type: 'u8' }, { name: 'b', type: 'u16' }, { name: 'c', type: 'u64' }, { name: 'd', type: 'u128' }, { name: 'e', type: 'bool' }, { name: 'f', type: { array: ['u8', 3] } }, { name: 'g', type: { array: ['u32', 2] } }, { name: 'k', type: 'pubkey' }] } }] };
  const l = layoutOf(idl, 'T');
  assert.equal(l.size, 1 + 2 + 8 + 16 + 1 + 3 + 8 + 32);
  const buf = Buffer.alloc(l.size);
  buf[0] = 7; buf.writeUInt16LE(513, 1); buf.writeBigUInt64LE(2n ** 63n + 5n, 3); buf.writeBigUInt64LE(9n, 11); buf.writeBigUInt64LE(1n, 19); buf[27] = 1; buf.set([1, 2, 3], 28); buf.writeUInt32LE(4, 31); buf.writeUInt32LE(5, 35); buf.fill(1, 39);
  const v = view(l, buf);
  assert.equal(v.get('a'), 7); assert.equal(v.get('b'), 513); assert.equal(v.get('c'), 2n ** 63n + 5n); assert.equal(v.get('d'), (1n << 64n) + 9n);
  assert.equal(v.get('e'), true); assert.deepEqual([...v.get('f')], [1, 2, 3]); assert.equal(v.get('g.1'), 5); assert.equal(v.get('k'), b58(Buffer.alloc(32, 1)));
  assert.throws(() => view(l, buf.subarray(0, 40)), /needs 71 bytes/);
});

test('discriminators and account positions come from the IDL', () => {
  assert.deepEqual([...discriminator(CP_AMM_IDL, 'accounts', 'Pool')], [241, 154, 109, 4, 17, 177, 109, 188]);
  assert.deepEqual([...discriminator(LOCKER_IDL, 'accounts', 'VestingEscrow')], [244, 119, 183, 4, 73, 116, 135, 195]);
  const at = accountIndexes(CP_AMM_IDL, 'claim_position_fee');
  assert.deepEqual([at.position, at.token_a_account, at.token_b_account, at.position_nft_account, at.signer], [2, 3, 4, 9, 10]);
  assert.throws(() => discriminator(CP_AMM_IDL, 'accounts', 'Nope'));
});

test('addresses derived from the IDL\'s seeds are the ones on chain', () => {
  // The position Meteora's reference pool still has, and its NFT's mint, as mainnet holds them (fixtures/cudis-pool.json).
  assert.equal(positionAddress('88JeSXCvR2tgRf4fW6GpSrt6rQmgxjiZkbJ5K8wKUDtN'), 'AgFNwaUEssj6krmF2F3D3yXJ21En8u1M5GU6TuhaMQuv');
  assert.throws(() => pdaOf(CP_AMM_IDL, 'create_position', 'position', {}), /needs the seed/);
  assert.throws(() => pdaOf(CP_AMM_IDL, 'create_position', 'pool', {}), /derives no address/);
  // Program-data addresses, as each Program account named its own on mainnet, Oct 13 2026.
  for (const [program, data] of [['cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG', 'AUh8bm2XsMfex3KjYGcM3G4uBqUNSDw6HEhWaWMYnyPH'], ['LocpQgucEQHbqNABEYvBvwoxCPsSbG91A1QaQhQQqjn', '7yXf8ZG1qdc6GdRUc6LxmwUJuUDUAqta84oPU7fsJ6Yy'], ['strmRqUCoQUgGUan5YhzUZa6KqdzwX5L6FpUxfmKg5m', '796tZf2QrH1ebZTQC1mDHYP6weDqSKuqbeyaXd64Prs'], ['metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s', 'PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT'], ['TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', '3gvYRKWyXRR9xKWe1ZjPhLY5ZJRN7KDB4rFZFGoJfFk2'], ['TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb', 'DoU57AYuPFu2QU514RktNPG22QhApEjnKxnBcu4BHDTY']]) assert.equal(programDataAddress(program), data);
  // The ARENA lock's token account is the escrow's associated account (fixtures/arena.json).
  assert.equal(associatedTokenAddress('77vFUtcMWqAtqDtojH3GABx2ZPHUpsKe9KkzFq45ASrW', 'ARENA551cbiMFSFzXv1V1RDxufZEw9Q2Kbd5uD7FrFeR', TOKEN_PROGRAM), '2YeJfj3eXiSVsHM24djjcRqtYU8pDr1VY8rbetQP73uQ');
});

test('base58 of any length decodes as PublicKey decodes 32 bytes', () => {
  for (const a of ['11111111111111111111111111111111', 'So11111111111111111111111111111111111111112', 'cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG']) assert.deepEqual(base58Decode(a), new PublicKey(a).toBuffer());
  assert.deepEqual([...base58Decode('2')], [1]); assert.deepEqual([...base58Decode('1')], [0]); assert.deepEqual([...base58Decode('')], []);
  assert.throws(() => base58Decode('0OIl'), /not base58/);
  assert.equal(isAddress('cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG'), true);
  for (const bad of ['', 'abc', 'cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sG0', 42, null]) assert.equal(isAddress(bad), false);
});
