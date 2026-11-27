// The byte decoders, held to real mainnet accounts (the fixtures) and to independent readings of the same accounts:
// the cluster's own parse, Meteora's own layout test, the sizes and offsets the programs' sources state.
import test from 'node:test';
import assert from 'node:assert/strict';
import { decodeMint, decodeTokenAccount, extension, EXTENSION_NAMES } from '../lib/spl.mjs';
import { metadataAddress, decodeMetadata } from '../lib/metaplex.mjs';
import { decodeLock, lockKind } from '../lib/locks.mjs';
import { decodePool, decodePosition, feeSchedule, feeAtPeriod, feePercent, positionAddress, positionNftAccount, claimsOf, FEE_DENOMINATOR } from '../lib/damm-v2.mjs';
import { decodeProgramData, programDataAddress, SOURCES } from '../lib/programs.mjs';
import { createRpc, fixtureTransport } from '../lib/rpc.mjs';
import { noNetwork, fixture, accountOf } from './helpers.mjs';

noNetwork();
const truth = fixture('cluster-parse').accounts;
const WIF = 'EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm', USDC = 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', PYUSD = '2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo', SRA = 'BoY2UJq7VJmyYzgszK9bzo6MFqQB4u5NEMhRupiDwJp4', ARENA = 'ARENA551cbiMFSFzXv1V1RDxufZEw9Q2Kbd5uD7FrFeR';

test('a mint read from its bytes is the mint the cluster parses', () => {
  for (const [name, address] of [['wif', WIF], ['usdc-stream', USDC], ['pyusd', PYUSD], ['sra', SRA]]) {
    const mine = decodeMint(accountOf(fixture(name), address)), theirs = truth[address].parsed.info;
    assert.equal(mine.mintAuthority, theirs.mintAuthority, name);
    assert.equal(mine.freezeAuthority, theirs.freezeAuthority, name);
    assert.equal(mine.decimals, theirs.decimals, name);
    assert.equal(mine.program, truth[address].owner, name);
  }
  assert.equal(decodeMint(accountOf(fixture('wif'), WIF)).mintAuthority, null);
  assert.equal(decodeMint(accountOf(fixture('usdc-stream'), USDC)).mintAuthority, 'BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG');
});

test('the supply offset is right: every holder of a real mint sums to the supply its mint states', () => {
  const fx = fixture('sra');
  const mint = decodeMint(accountOf(fx, SRA));
  const [, holders] = Object.entries(fx.calls).find(([k]) => k.startsWith('getProgramAccounts ["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"'));
  const sum = holders.reduce((s, h) => s + Buffer.from(h.account.data[0], 'base64').readBigUInt64LE(0), 0n);
  assert.equal(holders.length, 721);
  assert.equal(sum, mint.supply);
  assert.equal(mint.supply, 999911027513532n);
});

test('Token-2022 extensions are read as the cluster reads them (PayPal USD, eight extensions)', () => {
  const mint = decodeMint(accountOf(fixture('pyusd'), PYUSD)), theirs = truth[PYUSD].parsed.info.extensions;
  const camel = n => n[0].toLowerCase() + n.slice(1);
  assert.deepEqual(mint.extensions.map(e => camel(e.name)), theirs.map(e => e.extension));
  const state = name => theirs.find(e => e.extension === name).state;
  assert.equal(extension(mint, 'PermanentDelegate').delegate, state('permanentDelegate').delegate);
  assert.deepEqual(extension(mint, 'TransferHook'), state('transferHook'));
  assert.deepEqual(extension(mint, 'MetadataPointer'), state('metadataPointer'));
  assert.equal(extension(mint, 'MintCloseAuthority').closeAuthority, state('mintCloseAuthority').closeAuthority);
  const fee = extension(mint, 'TransferFeeConfig'), theirFee = state('transferFeeConfig');
  assert.equal(fee.configAuthority, theirFee.transferFeeConfigAuthority);
  assert.equal(fee.withdrawWithheldAuthority, theirFee.withdrawWithheldAuthority);
  assert.deepEqual([Number(fee.newer.epoch), fee.newer.basisPoints, Number(fee.newer.maximumFee)], [theirFee.newerTransferFee.epoch, theirFee.newerTransferFee.transferFeeBasisPoints, theirFee.newerTransferFee.maximumFee]);
  const md = extension(mint, 'TokenMetadata'), theirMd = state('tokenMetadata');
  assert.deepEqual([md.updateAuthority, md.mint, md.name, md.symbol, md.uri, md.additional], [theirMd.updateAuthority, theirMd.mint, theirMd.name, theirMd.symbol, theirMd.uri, theirMd.additionalMetadata]);
  assert.equal(EXTENSION_NAMES[12], 'PermanentDelegate'); assert.equal(EXTENSION_NAMES[26], 'Pausable'); assert.equal(EXTENSION_NAMES.length, 29);
});

test('a token account read from its bytes is the account the cluster parses', () => {
  const lockAccount = '2YeJfj3eXiSVsHM24djjcRqtYU8pDr1VY8rbetQP73uQ', vault = 'AjtoZGtMWx7AZZBUFsuzhhYZL2MhF7xmVxH3MJVsuGj5';
  const a = decodeTokenAccount(accountOf(fixture('arena'), lockAccount)), ta = truth[lockAccount].parsed.info;
  assert.deepEqual([a.mint, a.owner, a.amount.toString(), a.frozen], [ta.mint, ta.owner, ta.tokenAmount.amount, ta.state === 'frozen']);
  const v = decodeTokenAccount(accountOf(fixture('sra'), vault)), tv = truth[vault].parsed.info;
  assert.deepEqual([v.mint, v.owner], [tv.mint, tv.owner]);
  assert.throws(() => decodeTokenAccount(accountOf(fixture('wif'), WIF)), /not a token account's size/);
  assert.throws(() => decodeMint(accountOf(fixture('sra'), vault)), /not a mint's size/);
  assert.throws(() => decodeMint(null), /no such account/);
});

test('a program-data header read from its bytes is what the cluster reported for each program', () => {
  const where = { 'spl-token': 'wif', 'metaplex-token-metadata': 'wif', 'token-2022': 'pyusd', 'meteora-damm-v2': 'sra', 'jupiter-lock': 'cndl-lock', streamflow: 'nest-streams' };
  for (const [key, name] of Object.entries(where)) {
    const src = SOURCES.programs[key], address = programDataAddress(src.address);
    const [, answer] = Object.entries(fixture(name).calls).find(([k]) => k.startsWith(`getAccountInfo ["${address}"`));
    const state = decodeProgramData({ owner: answer.value.owner, data: Buffer.from(answer.value.data[0], 'base64') });
    assert.deepEqual(state, { slot: src.deployedSlot, upgradeAuthority: src.upgradeAuthority }, key);
  }
  assert.equal(decodeProgramData(accountOf(fixture('wif'), WIF)), null);
});

test('Metaplex metadata: an immutable name and a mutable one, at the address the program derives', () => {
  const wif = fixture('wif'), sra = fixture('sra');
  const a = decodeMetadata(accountOf(wif, metadataAddress(WIF)));
  assert.deepEqual([a.mint, a.name, a.symbol, a.isMutable], [WIF, 'dogwifhat', '$WIF', false]);
  const b = decodeMetadata(accountOf(sra, metadataAddress(SRA)));
  assert.deepEqual([b.mint, b.name, b.symbol, b.isMutable, b.updateAuthority], [SRA, "Stacc's Redemption Arc", 'SRA', true, 'Boy7pkvoq3ipJ3qfxBzCFeA76x1segrZHEANHBX4XAoe']);
  assert.equal(metadataAddress(SRA), '4KWvNUCc5tuszhTGgb4YHnezbqhYe763dyGYNuZPEsqd');
  assert.throws(() => decodeMetadata(accountOf(wif, WIF)), /not the Token Metadata program/);
});

test('Jupiter Lock: a lock nobody can cancel, and one its creator can', () => {
  const sound = '77vFUtcMWqAtqDtojH3GABx2ZPHUpsKe9KkzFq45ASrW', loose = '7jMXMng9BLNQraJgdfJd6goGh48VmP4CiQ4m213zkL6G';
  const a = decodeLock(sound, accountOf(fixture('arena'), sound));
  assert.deepEqual([a.kind, a.mint, a.cancelAuthority, a.recipientAuthority, a.scheduleAuthority, a.ended], ['jupiter-lock', ARENA, null, null, null, null]);
  assert.deepEqual([a.cliff, a.frequency, a.periods, a.perPeriod, a.total, a.claimed], [1799956382n, 2628000n, 6n, 150000000000000n, 900000000000000n, 0n]);
  assert.equal(a.end, 1799956382n + 6n * 2628000n);
  assert.equal(a.tokenAccount, '2YeJfj3eXiSVsHM24djjcRqtYU8pDr1VY8rbetQP73uQ');
  const b = decodeLock(loose, accountOf(fixture('cndl-lock'), loose));
  assert.deepEqual([b.cancelAuthority, b.recipientAuthority], ['the creator or the recipient', 'the creator or the recipient']);
  assert.equal(b.total, 365n * 1095890410958n);
  assert.throws(() => decodeLock(WIF, accountOf(fixture('wif'), WIF)), /neither Jupiter Lock nor Streamflow/);
  assert.equal(lockKind(null), null);
});

test('Streamflow: the layout holds on real streams (the mint field is the escrow account\'s mint; the balance covers what is owed)', () => {
  const nest = fixture('nest-streams'), usdc = fixture('usdc-stream');
  for (const [fx, address, mint] of [[nest, '2Mx2JkbGP25KRBd3kUsoNRdoLBUkTdgNSJrSYrwq77qi', '68Nq68CrtLVpyvK5Un7UADiNczaGf39hBbj3diRsYj6D'], [nest, '89oZt1YsPY2KWFgSyrCbXuF7JBT82gRphpENZo6w4SH7', '68Nq68CrtLVpyvK5Un7UADiNczaGf39hBbj3diRsYj6D'], [usdc, '4A72PrZwpvyQEi5Bu7oRiyqHTG5XVzUd7tqzjLH3TMNG', USDC]]) {
    const lock = decodeLock(address, accountOf(fx, address));
    assert.equal(lock.kind, 'streamflow');
    assert.equal(lock.mint, mint);
    const escrow = decodeTokenAccount(accountOf(fx, lock.tokenAccount));
    assert.equal(escrow.mint, mint, 'escrow_tokens is a token account of the stream\'s mint');
    assert.ok(escrow.amount >= lock.total - lock.claimed);
    assert.ok(lock.start > 1_600_000_000n && lock.end > lock.start && lock.end < 2_200_000_000n, 'the times are times');
  }
  const sound = decodeLock('2Mx2JkbGP25KRBd3kUsoNRdoLBUkTdgNSJrSYrwq77qi', accountOf(nest, '2Mx2JkbGP25KRBd3kUsoNRdoLBUkTdgNSJrSYrwq77qi'));
  assert.deepEqual([sound.cancelAuthority, sound.recipientAuthority, sound.scheduleAuthority, sound.total], [null, null, null, 55000000000000000n]);
  const loose = decodeLock('4A72PrZwpvyQEi5Bu7oRiyqHTG5XVzUd7tqzjLH3TMNG', accountOf(usdc, '4A72PrZwpvyQEi5Bu7oRiyqHTG5XVzUd7tqzjLH3TMNG'));
  assert.deepEqual([loose.cancelAuthority, loose.recipientAuthority], ['the sender', 'the sender']);
});

test('Meteora\'s reference pool reads here as Meteora\'s own layout test reads it', () => {
  // programs/cp-amm/src/tests/test_layout.rs, pool_account_layout_backward_compatible
  const address = 'E8zRkDw3UdzRc8qVWmqyQ9MLj7jhgZDHSroYud5t25A7', fx = fixture('cudis-pool');
  const pool = decodePool(address, accountOf(fx, address));
  assert.deepEqual([pool.baseFee.cliff, pool.baseFee.mode, pool.baseFee.periods, pool.baseFee.frequency, pool.baseFee.reduction], [500000000n, 1, 120, 60n, 265n]);
  assert.equal(pool.tokenAMint, 'CudisfkgWvMKnZ3TWf6iCuHm8pN2ikXhDcWytwz6f6RN');
  assert.equal(pool.tokenBMint, 'So11111111111111111111111111111111111111112');
  const position = decodePosition('AgFNwaUEssj6krmF2F3D3yXJ21En8u1M5GU6TuhaMQuv', accountOf(fx, 'AgFNwaUEssj6krmF2F3D3yXJ21En8u1M5GU6TuhaMQuv'));
  assert.equal(position.pool, address);
  assert.equal(positionAddress(position.nftMint), position.address);
  assert.throws(() => decodePool(address, accountOf(fx, 'AgFNwaUEssj6krmF2F3D3yXJ21En8u1M5GU6TuhaMQuv')), /not a pool/);
  assert.throws(() => decodePosition(address, accountOf(fx, address)), /not a position/);
});

test('the fee schedule is the program\'s arithmetic: linear exactly, exponential to the last unit', () => {
  assert.equal(FEE_DENOMINATOR, 1_000_000_000n);
  // ARENA's pool: 90% falling by 37395833 a step, 24 steps
  assert.equal(feeAtPeriod('linear', 900000000n, 37395833n, 24), 2500008n);
  const arena = fixture('arena'), pool = decodePool('CxKu5DQgzCPGZCwkHF3pCL3ngPWunkEoHCcMmSr65u71', accountOf(arena, 'CxKu5DQgzCPGZCwkHF3pCL3ngPWunkEoHCcMmSr65u71'));
  const s = feeSchedule(pool, { slot: arena.meta.slot, time: 1791880000 });
  assert.deepEqual([s.kind, s.curve, s.start, s.end, s.now, s.decays, s.unit, s.endsAt - s.activation], ['time', 'linear', 900000000n, 2500008n, 2500008n, true, 'second', 7200n]);
  // halfway through its two hours the fee is the cliff less twelve steps; before it opens there is no fee to quote
  assert.equal(feeSchedule(pool, { slot: 0, time: Number(pool.activationPoint) + 3600 }).now, 900000000n - 12n * 37395833n);
  assert.equal(feeSchedule(pool, { slot: 0, time: Number(pool.activationPoint) - 1 }).now, null);
  // exponential: within one part in a million of cliff * (1 - r/10000)^n, never above it (the program rounds down)
  for (const [cliff, r, n] of [[500000000n, 265n, 120], [800000000n, 1967n, 20], [990000000n, 1n, 1], [100000n, 9999n, 3]]) {
    const exact = Number(cliff) * (1 - Number(r) / 10000) ** n, got = Number(feeAtPeriod('exponential', cliff, r, n));
    assert.ok(got <= exact + 1e-6 && exact - got <= Math.max(2, exact * 1e-6), `${cliff} ${r} ${n}: ${got} against ${exact}`);
  }
  assert.equal(feeAtPeriod('exponential', 500000000n, 265n, 0), 500000000n);
  assert.equal(feeAtPeriod('exponential', 500000000n, 0n, 9), 500000000n);
  assert.equal(feePercent(2500008n), '0.25%'); assert.equal(feePercent(900000000n), '90%'); assert.equal(feePercent(10014870n), '1.0014%'); assert.equal(feePercent(null), 'n/a');
});

test('a fee claim is read out of a real transaction: who signed, how much, where it landed, where the account was emptied to', async () => {
  const position = 'pUMqtjspFsw15a39Tc6FQWmrREeUbaXvhYxSQhPXsKW';
  const { claims, signatures, truncated } = await claimsOf(createRpc(fixtureTransport(fixture('sra'))), position, 6);
  assert.deepEqual([claims.length, signatures, truncated], [1, 3, false]);
  const c = claims[0];
  assert.equal(c.eventTag, 'e445a52e51cb9a1d', 'Anchor\'s event tag, as the chain carries it');
  assert.deepEqual([c.feeA, c.feeB], [0n, 467728913657n]);
  assert.deepEqual([c.signer, c.holder, c.toBOwner, c.toBClosedTo], ['BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs', 'BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs', 'BUYmdVGKAYTNM3Qkwov3wwbzw7gewLzHLhFhCE7HeTUs', 'Boy7pkvoq3ipJ3qfxBzCFeA76x1segrZHEANHBX4XAoe']);
  assert.equal(positionNftAccount('A5qhXLga7SFKGJea3zthbBMekCmBkFi2sSwM9YJBYEEV'), '71ZTSNYoJgXB5kgk39mvCkMHvo7pyyUNvfRAbVDJfPY9', 'the NFT account the IDL derives is the one the claim named');
  assert.equal(c.nftAccount, '71ZTSNYoJgXB5kgk39mvCkMHvo7pyyUNvfRAbVDJfPY9');
});
