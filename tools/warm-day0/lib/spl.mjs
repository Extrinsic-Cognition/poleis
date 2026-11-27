// SPL Token and Token-2022 accounts, read from their bytes. Layouts: idl/sources.json ("spl-token", "token-2022").
//
//   Mint, 82 bytes:    mint_authority COption<Pubkey> 36 | supply u64 | decimals u8 | is_initialized u8 | freeze_authority COption<Pubkey> 36
//   Account, 165:      mint 32 | owner 32 | amount u64 | delegate COption 36 | state u8 | is_native COption<u64> 12 | delegated_amount u64 | close_authority COption 36
//   COption<Pubkey>:   a 4-byte tag (0 none, 1 some) and 32 bytes
//   Token-2022:        the same base padded to 165 bytes, one account-type byte (1 mint, 2 account), then extensions as
//                      type u16 | length u16 | value. An extension's optional key is 32 bytes, all zero when absent.
import { b58 } from './web3.mjs';
import { cursor } from './borsh.mjs';
import { TOKEN_PROGRAM, TOKEN_2022_PROGRAM } from './programs.mjs';

export const MINT_LEN = 82;
export const ACCOUNT_LEN = 165;
const ACCOUNT_TYPE_AT = 165;
const TLV_START = 166;

/** ExtensionType, in the order token-2022's enum declares it (interface/src/extension/mod.rs). */
export const EXTENSION_NAMES = Object.freeze([
  'Uninitialized', 'TransferFeeConfig', 'TransferFeeAmount', 'MintCloseAuthority', 'ConfidentialTransferMint',
  'ConfidentialTransferAccount', 'DefaultAccountState', 'ImmutableOwner', 'MemoTransfer', 'NonTransferable',
  'InterestBearingConfig', 'CpiGuard', 'PermanentDelegate', 'NonTransferableAccount', 'TransferHook', 'TransferHookAccount',
  'ConfidentialTransferFeeConfig', 'ConfidentialTransferFeeAmount', 'MetadataPointer', 'TokenMetadata', 'GroupPointer',
  'TokenGroup', 'GroupMemberPointer', 'TokenGroupMember', 'ConfidentialMintBurn', 'ScaledUiAmount', 'Pausable',
  'PausableAccount', 'PermissionedBurn',
]);

export function isTokenProgram(owner) { return owner === TOKEN_PROGRAM || owner === TOKEN_2022_PROGRAM; }

function coptionKey(buf, o) {
  const tag = buf.readUInt32LE(o);
  if (tag === 0) return null;
  if (tag === 1) return b58(buf.subarray(o + 4, o + 36));
  throw new Error(`a COption tag of ${tag} at byte ${o}: not a token account`);
}

function maybeKey(buf, o) {
  if (o + 32 > buf.length) throw new Error('an extension is shorter than its struct');
  const k = buf.subarray(o, o + 32);
  return k.every(b => b === 0) ? null : b58(k);
}

function decodeExtension(type, value) {
  switch (EXTENSION_NAMES[type]) {
    case 'TransferFeeConfig': return {
      configAuthority: maybeKey(value, 0), withdrawWithheldAuthority: maybeKey(value, 32), withheldAmount: value.readBigUInt64LE(64),
      older: { epoch: value.readBigUInt64LE(72), maximumFee: value.readBigUInt64LE(80), basisPoints: value.readUInt16LE(88) },
      newer: { epoch: value.readBigUInt64LE(90), maximumFee: value.readBigUInt64LE(98), basisPoints: value.readUInt16LE(106) },
    };
    case 'MintCloseAuthority': return { closeAuthority: maybeKey(value, 0) };
    case 'DefaultAccountState': return { state: value.readUInt8(0) };            // 1 initialized, 2 frozen
    case 'NonTransferable': return {};
    case 'PermanentDelegate': return { delegate: maybeKey(value, 0) };
    case 'TransferHook': return { authority: maybeKey(value, 0), programId: maybeKey(value, 32) };
    case 'MetadataPointer': return { authority: maybeKey(value, 0), metadataAddress: maybeKey(value, 32) };
    case 'Pausable': return { authority: maybeKey(value, 0), paused: value.readUInt8(32) !== 0 };
    case 'TokenMetadata': {
      const c = cursor(value);
      const updateAuthority = maybeKey(value, 0); c.bytes(32);
      return { updateAuthority, mint: c.pubkey(), name: c.string(), symbol: c.string(), uri: c.string(), additional: c.vec(() => [c.string(), c.string()]) };
    }
    default: return null;                                                         // named, not read
  }
}

function extensionsOf(data, expectType) {
  if (data.length <= ACCOUNT_LEN) return [];
  if (data[ACCOUNT_TYPE_AT] !== expectType) throw new Error(`account type byte is ${data[ACCOUNT_TYPE_AT]}, expected ${expectType}`);
  const out = [];
  for (let o = TLV_START; o + 4 <= data.length;) {
    const type = data.readUInt16LE(o), length = data.readUInt16LE(o + 2);
    if (type === 0) break;
    if (o + 4 + length > data.length) throw new Error(`extension ${type} runs past the end of the account`);
    const value = data.subarray(o + 4, o + 4 + length);
    out.push({ type, name: EXTENSION_NAMES[type] || `Unknown(${type})`, length, value: decodeExtension(type, value) });
    o += 4 + length;
  }
  return out;
}

/** A mint, from an account of either token program. Throws when the bytes are not a mint. */
export function decodeMint(account) {
  if (!account) throw new Error('no such account');
  if (!isTokenProgram(account.owner)) throw new Error(`owned by ${account.owner}, which is not a token program`);
  const d = account.data;
  const classic = account.owner === TOKEN_PROGRAM;
  if (classic ? d.length !== MINT_LEN : !(d.length === MINT_LEN || d.length > ACCOUNT_LEN)) throw new Error(`${d.length} bytes is not a mint's size`);
  if (d[45] !== 1) throw new Error('not an initialized mint');
  return {
    program: account.owner, token2022: !classic,
    mintAuthority: coptionKey(d, 0), supply: d.readBigUInt64LE(36), decimals: d.readUInt8(44), freezeAuthority: coptionKey(d, 46),
    extensions: classic ? [] : extensionsOf(d, 1),
  };
}

/** A token account, from an account of either token program. Throws when the bytes are not one. */
export function decodeTokenAccount(account) {
  if (!account) throw new Error('no such account');
  if (!isTokenProgram(account.owner)) throw new Error(`owned by ${account.owner}, which is not a token program`);
  const d = account.data;
  const classic = account.owner === TOKEN_PROGRAM;
  if (classic ? d.length !== ACCOUNT_LEN : d.length < ACCOUNT_LEN) throw new Error(`${d.length} bytes is not a token account's size`);
  if (!classic && d.length > ACCOUNT_LEN && d[ACCOUNT_TYPE_AT] !== 2) throw new Error('a Token-2022 account that is not a token account');
  const state = d.readUInt8(108);
  if (state === 0) throw new Error('not an initialized token account');
  return {
    program: account.owner, mint: b58(d.subarray(0, 32)), owner: b58(d.subarray(32, 64)), amount: d.readBigUInt64LE(64),
    delegate: coptionKey(d, 72), frozen: state === 2, delegatedAmount: d.readBigUInt64LE(121), closeAuthority: coptionKey(d, 129),
  };
}

export function extension(mint, name) { return mint.extensions.find(e => e.name === name)?.value ?? null; }
export function hasExtension(mint, name) { return mint.extensions.some(e => e.name === name); }
