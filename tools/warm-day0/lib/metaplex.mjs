// The Metaplex Token Metadata account of a mint. Layout and address: idl/sources.json ("metaplex-token-metadata").
// Borsh, read in the order the Metadata struct declares; nothing after isMutable is needed, so nothing after it is read.
import { cursor, unpad } from './borsh.mjs';
import { keyBytes, pda } from './web3.mjs';
import { METADATA_PROGRAM } from './programs.mjs';

const KEY_METADATA_V1 = 4;   // the Key enum: Uninitialized, EditionV1, MasterEditionV1, ReservationListV1, MetadataV1, ...

export function metadataAddress(mint) {
  return pda(['metadata', keyBytes(METADATA_PROGRAM), keyBytes(mint)], METADATA_PROGRAM);
}

export function decodeMetadata(account) {
  if (!account) throw new Error('no such account');
  if (account.owner !== METADATA_PROGRAM) throw new Error(`owned by ${account.owner}, not the Token Metadata program`);
  const c = cursor(account.data);
  const key = c.u8();
  if (key !== KEY_METADATA_V1) throw new Error(`key ${key} is not a metadata account`);
  const updateAuthority = c.pubkey();
  const mint = c.pubkey();
  const name = unpad(c.string()), symbol = unpad(c.string()), uri = unpad(c.string());
  const sellerFeeBasisPoints = c.u16();
  const creators = c.option(() => c.vec(() => ({ address: c.pubkey(), verified: c.bool(), share: c.u8() })));
  const primarySaleHappened = c.bool();
  const isMutable = c.bool();
  return { updateAuthority, mint, name, symbol, uri, sellerFeeBasisPoints, creators, primarySaleHappened, isMutable };
}
