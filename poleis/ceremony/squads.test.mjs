import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { b58encode } from './wire.mjs';
import { PERMISSIONS, multisigCreateV2, multisigTransactionIndex, proposalState, transactionMessage, vaultTransactionExecute } from './squads.mjs';

const k = (n) => b58encode(Uint8Array.from({ length: 32 }, (_, i) => (i === 0 ? n : i)));
const [VAULT, REG, TREAS, ALLOW, POLEIS] = [1, 2, 3, 4, 5].map(k);
// the pull as the adapter returns it: the vault signs and is paid
const pull = { programId: POLEIS, data: 'aabb', accounts: [
  { pubkey: VAULT, isSigner: true, isWritable: false }, { pubkey: REG, isSigner: false, isWritable: true },
  { pubkey: TREAS, isSigner: false, isWritable: true }, { pubkey: ALLOW, isSigner: false, isWritable: true },
  { pubkey: VAULT, isSigner: false, isWritable: true },
] };

test('the pull wraps as one Squads message: the vault the one writable signer, the program last', () => {
  const { bytes, keys } = transactionMessage(VAULT, pull);
  assert.deepEqual(keys.map((x) => x.pubkey), [VAULT, REG, TREAS, ALLOW, POLEIS]);
  assert.deepEqual([...bytes.subarray(0, 4)], [1, 1, 3, 5]); // signers, writable signers, writable non-signers, key count
  const ixAt = 4 + 5 * 32;
  assert.deepEqual([...bytes.subarray(ixAt, ixAt + 8)], [1, 4, 5, 0, 1, 2, 3, 0]); // one ix: program 4, accounts 0..3 and 0 again
  assert.deepEqual([...bytes.subarray(ixAt + 8, ixAt + 10)], [2, 0]); // data length, u16
  assert.equal(bytes.at(-1), 0); // no lookup tables
});

test('the execute hands the message keys back as remaining accounts, never as signers', () => {
  const { keys } = transactionMessage(VAULT, pull);
  const ex = vaultTransactionExecute({ multisig: k(9), proposal: k(10), transaction: k(11), member: k(12), keys });
  const rest = ex.accounts.slice(4);
  assert.deepEqual(rest.map((a) => a.pubkey), [VAULT, REG, TREAS, ALLOW, POLEIS]);
  assert.ok(rest.every((a) => !a.isSigner));
  assert.deepEqual(rest.map((a) => a.isWritable), [true, true, true, true, false]);
  assert.equal(ex.data, createHash('sha256').update('global:vault_transaction_execute').digest().subarray(0, 8).toString('hex'));
});

test('a 2-of-3 multisig carries three members with every permission and no config authority', () => {
  const ix = multisigCreateV2({ programConfig: k(20), treasury: k(21), multisig: k(22), createKey: k(23), creator: k(24), members: [k(30), k(31), k(32)], threshold: 2 });
  const d = Buffer.from(ix.data, 'hex');
  assert.equal(d[8], 0); // config authority: none
  assert.equal(d.readUInt16LE(9), 2);
  assert.equal(d.readUInt32LE(11), 3);
  assert.equal(d[15 + 32], 7);
  assert.equal(d.length, 8 + 1 + 2 + 4 + 3 * 33 + 4 + 1 + 1);
});

test('the multisig transaction index is read at its offset', () => {
  const b = Buffer.alloc(120); b.writeBigUInt64LE(7n, 78);
  assert.equal(multisigTransactionIndex(b), 7n);
});

test('a house member may initiate and execute but never vote', () => {
  const ix = multisigCreateV2({ programConfig: k(20), treasury: k(21), multisig: k(22), createKey: k(23), creator: k(24),
    members: [k(30), k(31), k(32), { key: k(33), mask: PERMISSIONS.initiate | PERMISSIONS.execute }], threshold: 2 });
  const d = Buffer.from(ix.data, 'hex');
  assert.equal(d.readUInt32LE(11), 4);
  assert.equal(d[15 + 3 * 33 + 32], 5);
  assert.equal(d[15 + 32], 7);
});

test("a proposal's approvals are read past its status and bump", () => {
  const b = Buffer.alloc(200);
  b[48] = 3; // Approved, then an 8-byte timestamp, then the bump
  b.writeUInt32LE(2, 58);
  b.fill(7, 62, 94); b.fill(8, 94, 126);
  const s = proposalState(b);
  assert.equal(s.status, 'Approved');
  assert.deepEqual(s.approved.map((x) => x[0]), [7, 8]);
});
