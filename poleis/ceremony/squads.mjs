// Squads v4, by hand and by its own interface definition (read from the chain: the IDL account of SQDS4ep6…, version 2.1.0, 36
// instructions). Only what an allowance needs: a 2-of-3 multisig, a vault transaction that wraps one instruction, its proposal,
// the approvals and the execute. Anchor's encodings: an 8-byte discriminator, sha256("global:<name>"), then borsh.
import { createHash } from 'node:crypto';
import { b58decode } from './wire.mjs';

export const SQUADS = 'SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf';
export const SYSTEM = '11111111111111111111111111111111';
export const PERMISSIONS = { initiate: 1, vote: 2, execute: 4 };
export const PERMISSIONS_ALL = 7; // initiate | vote | execute

const disc = (name) => createHash('sha256').update(`global:${name}`).digest().subarray(0, 8);
const u8 = (n) => Buffer.of(n);
const u16 = (n) => { const b = Buffer.alloc(2); b.writeUInt16LE(n); return b; };
const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
export const u64 = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const NONE = u8(0);
export function key32(s) {
  const k = b58decode(s);
  if (k.length !== 32) throw new Error(`not a 32-byte key: ${s}`);
  return Buffer.from(k);
}
const meta = (pubkey, isSigner, isWritable) => ({ pubkey, isSigner, isWritable });
const ix = (name, data, accounts) => ({ programId: SQUADS, accounts, data: Buffer.concat([disc(name), ...data]).toString('hex') });

/** The seeds of every Squads account the allowance touches, for the adapter's `pda`. */
export const seeds = {
  programConfig: () => ['multisig', 'program_config'].map((s) => Buffer.from(s).toString('hex')),
  multisig: (createKey) => [Buffer.from('multisig'), Buffer.from('multisig'), key32(createKey)].map((b) => b.toString('hex')),
  vault: (multisig, index = 0) => [Buffer.from('multisig'), key32(multisig), Buffer.from('vault'), u8(index)].map((b) => b.toString('hex')),
  transaction: (multisig, index) => [Buffer.from('multisig'), key32(multisig), Buffer.from('transaction'), u64(index)].map((b) => b.toString('hex')),
  proposal: (multisig, index) => [Buffer.from('multisig'), key32(multisig), Buffer.from('transaction'), u64(index), Buffer.from('proposal')].map((b) => b.toString('hex')),
};

/** The fields read back: the program config's fee treasury, the multisig's last transaction index. */
export const programConfigTreasury = (data) => data.subarray(48, 80); // disc 8, authority 32, fee 8
/** A proposal's status and approvals: disc 8, multisig 32, index 8, status (a byte, then a timestamp unless Executing), bump, approved. */
export const STATUS = ['Draft', 'Active', 'Rejected', 'Approved', 'Executing', 'Executed', 'Cancelled'];
export function proposalState(data) {
  const variant = data[48];
  let at = 49 + (variant === 4 ? 0 : 8) + 1;
  const n = data.readUInt32LE(at); at += 4;
  const approved = [];
  for (let i = 0; i < n; i++) approved.push(data.subarray(at + 32 * i, at + 32 * (i + 1)));
  return { status: STATUS[variant], approved };
}
export const multisigTransactionIndex = (data) => data.readBigUInt64LE(78); // disc 8, create key 32, config authority 32, threshold 2, time lock 4

/** Members are keys (every permission) or { key, mask }: a house key may initiate and execute but never vote. */
export function multisigCreateV2({ programConfig, treasury, multisig, createKey, creator, members, threshold }) {
  const member = (m) => (typeof m === 'string' ? { key: m, mask: PERMISSIONS_ALL } : m);
  const data = [NONE, u16(threshold), u32(members.length), ...members.map(member).map((m) => Buffer.concat([key32(m.key), u8(m.mask)])), u32(0), NONE, NONE];
  return ix('multisig_create_v2', data, [
    meta(programConfig, false, false), meta(treasury, false, true), meta(multisig, false, true),
    meta(createKey, true, false), meta(creator, true, true), meta(SYSTEM, false, false),
  ]);
}

/**
 * One instruction as a Squads TransactionMessage, the vault its first signer: keys ordered writable signers, readonly signers,
 * writable non-signers, readonly non-signers; SmallVec lengths (u8, and u16 for data). Returns the bytes and the keys in order,
 * which the execute passes back as its remaining accounts.
 */
export function transactionMessage(vault, inner) {
  const flags = new Map([[vault, { signer: true, writable: true }]]);
  for (const a of inner.accounts) {
    const f = flags.get(a.pubkey) || { signer: false, writable: false };
    flags.set(a.pubkey, { signer: f.signer || a.isSigner, writable: f.writable || a.isWritable });
  }
  if (!flags.has(inner.programId)) flags.set(inner.programId, { signer: false, writable: false });
  const rank = ([k, f]) => (k === vault ? -1 : f.signer ? (f.writable ? 0 : 1) : (f.writable ? 2 : 3));
  const keys = [...flags.entries()].sort((x, y) => rank(x) - rank(y)).map(([pubkey, f]) => ({ pubkey, ...f }));
  const index = (k) => keys.findIndex((x) => x.pubkey === k);
  const data = Buffer.from(inner.data, 'hex');
  const bytes = Buffer.concat([
    u8(keys.filter((k) => k.signer).length),
    u8(keys.filter((k) => k.signer && k.writable).length),
    u8(keys.filter((k) => !k.signer && k.writable).length),
    u8(keys.length), ...keys.map((k) => key32(k.pubkey)),
    u8(1), u8(index(inner.programId)), u8(inner.accounts.length), Buffer.from(inner.accounts.map((a) => index(a.pubkey))), u16(data.length), data,
    u8(0),
  ]);
  return { bytes, keys };
}

export function vaultTransactionCreate({ multisig, transaction, creator, rentPayer, message }) {
  return ix('vault_transaction_create', [u8(0), u8(0), u32(message.length), message, NONE], [
    meta(multisig, false, true), meta(transaction, false, true), meta(creator, true, false), meta(rentPayer, true, true), meta(SYSTEM, false, false),
  ]);
}

export function proposalCreate({ multisig, proposal, creator, rentPayer, index }) {
  return ix('proposal_create', [u64(index), u8(0)], [
    meta(multisig, false, false), meta(proposal, false, true), meta(creator, true, false), meta(rentPayer, true, true), meta(SYSTEM, false, false),
  ]);
}

export function proposalApprove({ multisig, member, proposal }) {
  return ix('proposal_approve', [NONE], [meta(multisig, false, false), meta(member, true, true), meta(proposal, false, true)]);
}

/** The execute: the message's keys follow as remaining accounts, writable as the message says, the vault never a signer here. */
export function vaultTransactionExecute({ multisig, proposal, transaction, member, keys }) {
  return ix('vault_transaction_execute', [], [
    meta(multisig, false, false), meta(proposal, false, true), meta(transaction, false, false), meta(member, true, false),
    ...keys.map((k) => meta(k.pubkey, false, k.writable)),
  ]);
}
