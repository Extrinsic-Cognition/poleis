// The programs the rows read, where each layout was read, and when. Every address here is public and none is ours.
// When a program is upgraded after the slot noted in idl/sources.json, the row that reads it says so and its layout is read
// again by hand (README, "When a program moves").
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { PublicKey, pda } from './web3.mjs';

const idlDir = join(dirname(fileURLToPath(import.meta.url)), '..', 'idl');
const readJson = name => JSON.parse(readFileSync(join(idlDir, name), 'utf8'));

export const TOKEN_PROGRAM = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
export const TOKEN_2022_PROGRAM = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';
export const ASSOCIATED_TOKEN_PROGRAM = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';
export const METADATA_PROGRAM = 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s';
export const JUP_LOCK_PROGRAM = 'LocpQgucEQHbqNABEYvBvwoxCPsSbG91A1QaQhQQqjn';
export const STREAMFLOW_PROGRAM = Object.freeze({ mainnet: 'strmRqUCoQUgGUan5YhzUZa6KqdzwX5L6FpUxfmKg5m', devnet: 'HqDGZjaVRXJ9MGRQEw7qDc2rAr6iH1n1kAQdCZaCMfMZ' });
export const STREAMFLOW_ALIGNED_PROGRAM = 'aSTRM2NKoKxNnkmLWk9sz3k74gKBk9t7bpPrTGxMszH';
export const DAMM_V2_PROGRAM = 'cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG';
export const BPF_UPGRADEABLE_LOADER = 'BPFLoaderUpgradeab1e11111111111111111111111';

export const CP_AMM_IDL = readJson('cp_amm.json');
export const LOCKER_IDL = readJson('locker.json');
export const STREAMFLOW_SPEC = readJson('streamflow-contract.json');
export const SOURCES = readJson('sources.json');
export const READ_ON = SOURCES.readOn;

/** Where the upgradeable loader keeps a program's code, its last deploy slot and its upgrade authority. */
export function programDataAddress(programId) { return pda([new PublicKey(programId).toBuffer()], BPF_UPGRADEABLE_LOADER); }

/** The associated token account of `owner` for `mint` under `tokenProgram`. */
export function associatedTokenAddress(owner, mint, tokenProgram = TOKEN_PROGRAM) {
  return pda([new PublicKey(owner).toBuffer(), new PublicKey(tokenProgram).toBuffer(), new PublicKey(mint).toBuffer()], ASSOCIATED_TOKEN_PROGRAM);
}

/** sha256 of a vendored IDL as its source holds it (the copy here ends in one newline the source does not have). */
export function vendoredSha256(name) {
  const bytes = readFileSync(join(idlDir, name));
  const body = bytes[bytes.length - 1] === 0x0a ? bytes.subarray(0, bytes.length - 1) : bytes;
  return createHash('sha256').update(body).digest('hex');
}

const PROGRAM_DATA_HEADER = 45, PROGRAM_DATA_VARIANT = 3;

/**
 * The header of a program-data account (UpgradeableLoaderState::ProgramData, bincode: variant u32 = 3, slot u64, then an
 * option byte and the upgrade authority). Null when the bytes are not one.
 */
export function decodeProgramData(account) {
  if (!account || account.owner !== BPF_UPGRADEABLE_LOADER || account.data.length < PROGRAM_DATA_HEADER) return null;
  const d = account.data;
  if (d.readUInt32LE(0) !== PROGRAM_DATA_VARIANT) return null;
  return { slot: Number(d.readBigUInt64LE(4)), upgradeAuthority: d[12] === 1 ? new PublicKey(d.subarray(13, 45)).toBase58() : null };
}

/** Has a program been redeployed since its layout was read? Reads those 45 bytes, never the megabytes of code behind them. */
export async function deployState(rpc, programId) {
  const address = programDataAddress(programId);
  const state = decodeProgramData(await rpc.slice(address, 0, PROGRAM_DATA_HEADER));
  return state && { programData: address, ...state };
}
