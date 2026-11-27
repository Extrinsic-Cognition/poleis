// A lock account, read as what it is: a Jupiter Lock vesting escrow or a Streamflow stream. Layouts: idl/sources.json.
// Each returns the same shape so row 7 judges both with one rule: the dates, who could still cancel, who could still move
// the recipient, and whether the tokens are really in the escrow's own token account.
import { layoutOf, view, discriminator } from './layout.mjs';
import { JUP_LOCK_PROGRAM, STREAMFLOW_PROGRAM, STREAMFLOW_ALIGNED_PROGRAM, LOCKER_IDL, STREAMFLOW_SPEC, TOKEN_PROGRAM, TOKEN_2022_PROGRAM, associatedTokenAddress } from './programs.mjs';

// jup-ag/lock-sdk locker.ts: the same four values for cancel_mode and update_recipient_mode.
const JUP_MODE = Object.freeze(['none', 'the creator', 'the recipient', 'the creator or the recipient']);

const ESCROW = layoutOf(LOCKER_IDL, 'VestingEscrow');
const ESCROW_DISC = discriminator(LOCKER_IDL, 'accounts', 'VestingEscrow');
const STREAM = layoutOf(STREAMFLOW_SPEC, 'Contract');

export function lockKind(account, cluster = 'mainnet') {
  if (!account) return null;
  if (account.owner === JUP_LOCK_PROGRAM) return 'jupiter-lock';
  if (account.owner === STREAMFLOW_PROGRAM[cluster]) return 'streamflow';
  return null;
}

/** A Jupiter Lock VestingEscrow. Unlocked at t: 0 before cliff_time, then cliff_unlock_amount + min((t - cliff_time) / frequency, number_of_period) * amount_per_period. */
export function decodeJupiterLock(address, account) {
  const d = account.data;
  if (d.length !== 8 + ESCROW.size) throw new Error(`${d.length} bytes is not a vesting escrow (${8 + ESCROW.size})`);
  if (!d.subarray(0, 8).equals(ESCROW_DISC)) throw new Error('not a VestingEscrow account (another account of the lock program)');
  const v = view(ESCROW, d, 8);
  const cancelMode = v.get('cancel_mode'), updateMode = v.get('update_recipient_mode'), flag = v.get('token_program_flag');
  if (cancelMode > 3 || updateMode > 3 || flag > 1) throw new Error(`a mode byte this layout does not know (cancel ${cancelMode}, recipient ${updateMode}, token program ${flag})`);
  const cliffTime = v.get('cliff_time'), frequency = v.get('frequency'), periods = v.get('number_of_period');
  const cliffAmount = v.get('cliff_unlock_amount'), perPeriod = v.get('amount_per_period');
  const mint = v.get('token_mint');
  const tokenProgram = flag === 1 ? TOKEN_2022_PROGRAM : TOKEN_PROGRAM;
  const cancelledAt = v.get('cancelled_at');
  return {
    kind: 'jupiter-lock', address, mint,
    recipient: v.get('recipient'), creator: v.get('creator'),
    tokenAccount: associatedTokenAddress(address, mint, tokenProgram), tokenProgram,
    start: v.get('vesting_start_time'), cliff: cliffTime, end: cliffTime + frequency * periods,
    frequency, periods, cliffAmount, perPeriod,
    total: cliffAmount + perPeriod * periods, claimed: v.get('total_claimed_amount'),
    cancelAuthority: cancelMode === 0 ? null : JUP_MODE[cancelMode],
    recipientAuthority: updateMode === 0 ? null : JUP_MODE[updateMode],
    scheduleAuthority: null,
    ended: cancelledAt > 0n ? `cancelled at ${cancelledAt}` : null,
    notes: [],
  };
}

/** A Streamflow stream (Contract). */
export function decodeStreamflow(address, account) {
  const d = account.data;
  if (d.length !== STREAMFLOW_SPEC.accountSize) throw new Error(`${d.length} bytes is not a stream (${STREAMFLOW_SPEC.accountSize})`);
  const v = view(STREAM, d, 0);
  const flag = name => v.get(`ix.${name}`);
  const bySender = flag('cancelable_by_sender'), byRecipient = flag('cancelable_by_recipient');
  const moveBySender = flag('transferable_by_sender'), moveByRecipient = flag('transferable_by_recipient');
  const canUpdate = flag('can_update_rate'), pausable = flag('pausable');
  const both = (a, b) => (a && b ? 'the sender or the recipient' : a ? 'the sender' : b ? 'the recipient' : null);
  const canceledAt = v.get('canceled_at'), closed = v.get('closed');
  const notes = [];
  if (pausable) notes.push('the sender can pause this stream, which moves every date later');
  if (flag('can_topup')) notes.push('the sender can add tokens to this stream');
  const cancelRequest = v.get('cancel_request_time');
  if (cancelRequest > 0n) notes.push(`a cancel request is recorded at ${cancelRequest}`);
  const deposited = flag('net_amount_deposited');
  return {
    kind: 'streamflow', address, mint: v.get('mint'),
    recipient: v.get('recipient'), creator: v.get('sender'),
    tokenAccount: v.get('escrow_tokens'), tokenProgram: null,
    start: flag('start_time'), cliff: flag('cliff'), end: v.get('end_time'),
    frequency: flag('period'), periods: null, cliffAmount: flag('cliff_amount'), perPeriod: flag('amount_per_period'),
    total: deposited, claimed: v.get('amount_withdrawn'),
    cancelAuthority: both(bySender, byRecipient),
    recipientAuthority: both(moveBySender, moveByRecipient),
    // Streamflow's own page: the cancel and transfer flags "can be toggled later via update(), but only if the stream was
    // originally created with canUpdateRate: true". A stream that may be updated has no fixed dates and no fixed flags.
    scheduleAuthority: canUpdate ? 'the sender (can_update_rate: the release rate, and through update() the cancel and transfer flags)' : null,
    ended: closed ? 'closed' : canceledAt > 0n ? `cancelled at ${canceledAt}` : null,
    notes,
  };
}

export function decodeLock(address, account, cluster = 'mainnet') {
  const kind = lockKind(account, cluster);
  if (kind === 'jupiter-lock') return decodeJupiterLock(address, account);
  if (kind === 'streamflow') return decodeStreamflow(address, account);
  if (account && account.owner === STREAMFLOW_ALIGNED_PROGRAM) throw new Error('a Streamflow price-aligned unlock: its release follows a price, so it has no fixed dates; name the stream account it wraps to read its flags');
  throw new Error(account ? `owned by ${account.owner}: neither Jupiter Lock nor Streamflow` : 'no such account');
}
