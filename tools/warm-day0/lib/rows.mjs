// The fourteen rows of docs/open-transparent-markets-from-day-0-2026-10-13.md section 6, each decided from what a stranger can
// read: `pass`, `fail`, or `not checkable` with what a person looks at instead. A row never passes on a promise: where the
// chain cannot say, the row says so.
import { decodeMint, decodeTokenAccount, extension, hasExtension, isTokenProgram } from './spl.mjs';
import { metadataAddress, decodeMetadata } from './metaplex.mjs';
import { decodeLock } from './locks.mjs';
import { decodePool, decodePosition, positionsOf, positionHolder, positionAddress, feeSchedule, feePercent, claimsOf, poolNow, poolUnit, COLLECT_FEE_MODES, POOL_TYPES, POOL } from './damm-v2.mjs';
import { DAMM_V2_PROGRAM, JUP_LOCK_PROGRAM, STREAMFLOW_PROGRAM, METADATA_PROGRAM, TOKEN_PROGRAM, TOKEN_2022_PROGRAM, SOURCES, deployState } from './programs.mjs';
import { isAddress, DEFAULT_PUBKEY } from './web3.mjs';

export const PASS = 'pass', FAIL = 'fail', UNCHECKED = 'not checkable';

export const CLAIMS = Object.freeze({
  1: 'One canonical mint', 2: 'Fixed supply', 3: 'No holder can be frozen', 4: 'The name cannot be swapped',
  5: 'Every token is somewhere named', 6: 'No hidden tranche', 7: 'Locks are programs, not promises',
  8: 'One open pool from the first slot', 9: 'The liquidity cannot be pulled', 10: 'No prize for speed',
  11: "The house's wallets are named", 12: 'The fee is read, not reported', 13: 'The spend is read too', 14: 'One live page',
});

export class UsageError extends Error { constructor(m) { super(m); this.name = 'UsageError'; } }

const row = (n, verdict, source, detail = [], notes = [], data = {}) => ({ n, claim: CLAIMS[n], verdict, source: [].concat(source).filter(Boolean), detail, notes, data });

// ---- how things are written -----------------------------------------------------------------------------------------------

export function when(t) {
  if (t === null || t === undefined) return 'an unknown time';
  const ms = Number(t) * 1000;
  if (!Number.isFinite(ms) || Math.abs(ms) > 8.64e15) return `time ${t}`;
  return new Date(ms).toISOString().replace('.000Z', 'Z');
}

export function tokens(raw, decimals) {
  const neg = raw < 0n, abs = neg ? -raw : raw, unit = 10n ** BigInt(decimals);
  const whole = (abs / unit).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  const frac = decimals ? (abs % unit).toString().padStart(decimals, '0').replace(/0+$/, '') : '';
  return `${neg ? '-' : ''}${whole}${frac ? '.' + frac : ''}`;
}

export function share(part, whole) {
  if (whole === 0n) return 'n/a';
  const bps = part * 1_000_000n / whole;                           // hundredths of a basis point
  return `${bps / 10_000n}.${(bps % 10_000n).toString().padStart(4, '0').slice(0, 2)}%`;
}

function span(n, unit) {
  const seconds = unit === 'slot' ? Number(n) * 0.4 : Number(n);
  const human = seconds >= 172800 ? `${(seconds / 86400).toFixed(1)} days` : seconds >= 7200 ? `${(seconds / 3600).toFixed(1)} hours` : seconds >= 120 ? `${(seconds / 60).toFixed(0)} minutes` : `${seconds.toFixed(0)} seconds`;
  if (unit === 'slot') return `${n} slots (about ${human})`;
  return seconds < 120 ? `${n} s` : `${n} s (${human})`;
}

const BASE58 = '[1-9A-HJ-NP-Za-km-z]';
function namesAddress(text, address) { return new RegExp(`(?<!${BASE58})${address}(?!${BASE58})`).test(text); }

/** A cluster that does not answer makes a row `not checkable`; a call a fixture does not hold, or a refused method, is a bug and throws. */
async function guarded(n, source, fn) {
  try { return await fn(); } catch (e) {
    if (e.name !== 'RpcError' || e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e;
    return row(n, UNCHECKED, source, [`The cluster did not answer: ${e.message}. Run again, or name another endpoint with --rpc.`]);
  }
}

// ---- row 1 ----------------------------------------------------------------------------------------------------------------

async function row1(ctx) {
  const person = `A person checks that the fees page (section IV), the repository and the first post all carry ${ctx.mintAddress}, published the same minute.`;
  const { domain, resolveTxt } = ctx;
  if (!domain) return row(1, UNCHECKED, [], [`No --domain given, so no DNS record was read. ${person}`]);
  const source = `dns:TXT ${domain}`;
  let records;
  try { records = (await resolveTxt(domain)).map(chunks => chunks.join('')); } catch (e) {
    if (e.code === 'ENODATA' || e.code === 'ENOTFOUND') return row(1, FAIL, source, [`${domain} has no TXT record (${e.code}), so the domain does not name this mint.`, person]);
    return row(1, UNCHECKED, source, [`DNS did not answer for ${domain} (${e.code || e.message}).`, person]);
  }
  const hit = records.find(r => namesAddress(r, ctx.mintAddress));
  if (hit) return row(1, UNCHECKED, source, [`The DNS TXT record of ${domain} names this mint: "${hit}". That is one place of four.`, person], [], { txt: hit });
  const others = [...new Set(records.flatMap(r => r.match(new RegExp(`${BASE58}{32,44}`, 'g')) || []).filter(isAddress))];
  return row(1, FAIL, source, [`${records.length} TXT record${records.length === 1 ? '' : 's'} on ${domain}, none names this mint.${others.length ? ` Named there instead: ${others.join(', ')}.` : ''}`, person], [], { records: records.length });
}

// ---- rows 2, 3, 4: the mint itself ------------------------------------------------------------------------------------------

function row2(ctx) {
  const { mint, mintAddress } = ctx;
  const facts = `Supply ${tokens(mint.supply, mint.decimals)} (${mint.supply} base units, ${mint.decimals} decimals), ${mint.token2022 ? 'Token-2022' : 'SPL Token'}.`;
  const notes = [];
  for (const name of ['ScaledUiAmount', 'InterestBearingConfig']) if (hasExtension(mint, name)) notes.push(`Token-2022 ${name}: the amount a wallet shows is the supply times a rate an authority may set, with no mint.`);
  if (hasExtension(mint, 'ConfidentialMintBurn')) notes.push('Token-2022 ConfidentialMintBurn: mints and burns can be hidden amounts.');
  if (mint.mintAuthority) return row(2, FAIL, mintAddress, [`Mint authority: ${mint.mintAuthority}. It can mint more at any time.`, facts], notes, { mintAuthority: mint.mintAuthority });
  return row(2, PASS, mintAddress, ['Mint authority: none. Nobody can mint more.', facts], notes, { mintAuthority: null });
}

function row3(ctx) {
  const { mint, mintAddress } = ctx;
  const why = [], notes = [];
  if (mint.freezeAuthority) why.push(`Freeze authority: ${mint.freezeAuthority}. It can freeze any holder's account.`);
  const pausable = extension(mint, 'Pausable');
  if (pausable?.authority) why.push(`Token-2022 Pausable: ${pausable.authority} can pause every transfer${pausable.paused ? ' (paused now)' : ''}.`);
  else if (pausable?.paused) why.push('Token-2022 Pausable: paused, with no authority left to unpause.');
  const delegate = extension(mint, 'PermanentDelegate');
  if (delegate?.delegate) why.push(`Token-2022 PermanentDelegate: ${delegate.delegate} can move or burn any holder's tokens.`);
  const hook = extension(mint, 'TransferHook');
  if (hook?.programId) why.push(`Token-2022 TransferHook: program ${hook.programId} runs on every transfer and can refuse it.`);
  else if (hook?.authority) why.push(`Token-2022 TransferHook: ${hook.authority} can set a program that runs on every transfer.`);
  if (extension(mint, 'DefaultAccountState')?.state === 2) why.push('Token-2022 DefaultAccountState: every new token account starts frozen.');
  if (hasExtension(mint, 'NonTransferable')) why.push('Token-2022 NonTransferable: no holder can transfer at all.');
  const fee = extension(mint, 'TransferFeeConfig');
  if (fee) notes.push(`Token-2022 TransferFeeConfig: ${fee.newer.basisPoints} bps on every transfer${fee.configAuthority ? `, changeable by ${fee.configAuthority}` : ', fixed'}.`);
  const unread = mint.extensions.filter(e => e.value === null).map(e => e.name);
  if (unread.length) notes.push(`Token-2022 extensions present and not read: ${unread.join(', ')}.`);
  const data = { freezeAuthority: mint.freezeAuthority, extensions: mint.extensions.map(e => e.name) };
  if (why.length) return row(3, FAIL, mintAddress, why, notes, data);
  return row(3, PASS, mintAddress, [`Freeze authority: none.${mint.token2022 ? ' No extension that can stop or take a holder\'s tokens.' : ''}`], notes, data);
}

async function row4(ctx) {
  const { mint, mintAddress, rpc } = ctx;
  const address = metadataAddress(mintAddress);
  const account = await rpc.account(address);
  if (account) {
    ctx.programs.add('metaplex-token-metadata');
    let md;
    try { md = decodeMetadata(account); } catch (e) { return row(4, FAIL, address, [`The account at the metadata address does not read as Metaplex metadata: ${e.message}.`]); }
    const named = `Name "${md.name}", symbol "${md.symbol}", uri ${md.uri || '(none)'}.`;
    const data = { name: md.name, symbol: md.symbol, uri: md.uri, updateAuthority: md.updateAuthority, isMutable: md.isMutable };
    if (md.mint !== mintAddress) return row(4, FAIL, address, [`The metadata at this address is for ${md.mint}, not this mint.`], [], data);
    if (md.isMutable) return row(4, FAIL, address, [`Metadata is mutable: ${md.updateAuthority} can change the name, the symbol and the uri.`, named], [], data);
    return row(4, PASS, address, ['Metadata is immutable (isMutable is false): nobody can change the name, the symbol or the uri.', named], [`The file at the uri is not on chain: what it shows is its host's to keep.`], data);
  }
  const own = extension(mint, 'TokenMetadata'), pointer = extension(mint, 'MetadataPointer');
  if (own) {
    const why = [];
    if (own.updateAuthority) why.push(`Token-2022 metadata in the mint: ${own.updateAuthority} can change the name, the symbol and the uri.`);
    if (pointer?.authority) why.push(`Token-2022 MetadataPointer: ${pointer.authority} can point the mint at other metadata.`);
    if (pointer && pointer.metadataAddress !== mintAddress) why.push(`Token-2022 MetadataPointer names ${pointer.metadataAddress ?? 'nothing'}, not the mint that holds this metadata.`);
    const named = `Name "${own.name}", symbol "${own.symbol}", uri ${own.uri || '(none)'}.`;
    const data = { name: own.name, symbol: own.symbol, uri: own.uri, updateAuthority: own.updateAuthority, isMutable: why.length > 0 };
    if (why.length) return row(4, FAIL, mintAddress, [...why, named], [], data);
    return row(4, PASS, mintAddress, ['Token-2022 metadata in the mint, with no update authority and a fixed pointer: nobody can change it.', named], [], data);
  }
  if (pointer?.metadataAddress) return row(4, UNCHECKED, mintAddress, [`The mint points at metadata held in ${pointer.metadataAddress}, a layout this checker does not read. A person reads who may update that account.`]);
  return row(4, FAIL, address, [`No metadata: there is no account at the Metaplex metadata address. ${mint.mintAuthority ? 'The mint authority can create one with any name.' : 'With no mint authority none can be created, so the token has no name on chain at all.'}`]);
}

// ---- row 5: the table -------------------------------------------------------------------------------------------------------

/** The mint's holders, largest first. The cluster's own top twenty when it serves them, every holder otherwise. */
async function holdersOf(ctx) {
  const { rpc, mint, mintAddress } = ctx;
  let first;
  try {
    const top = await rpc.largestAccounts(mintAddress);
    return { how: 'getTokenLargestAccounts', complete: false, accounts: top.map(a => ({ address: a.address, amount: BigInt(a.amount) })) };
  } catch (e) { if (e.name !== 'RpcError' || e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e; first = e; }
  try {
    const filters = mint.token2022 ? [{ memcmp: { offset: 0, bytes: mintAddress } }] : [{ dataSize: 165 }, { memcmp: { offset: 0, bytes: mintAddress } }];
    const all = await rpc.programAccounts(mint.program, filters, { offset: 64, length: 8 });
    const accounts = all.map(a => ({ address: a.address, amount: a.data.readBigUInt64LE(0) })).filter(a => a.amount > 0n).sort((a, b) => (b.amount > a.amount ? 1 : b.amount < a.amount ? -1 : 0));
    return { how: 'getProgramAccounts', complete: true, accounts };
  } catch (e) {
    if (e.name !== 'RpcError' || e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e;
    return { how: null, complete: false, accounts: [], error: `${first.message}; then ${e.message}` };
  }
}

async function row5(ctx) {
  const { rpc, mint, mintAddress, manifest } = ctx;
  const labelled = manifest?.accounts || [];
  if (!labelled.length) return row(5, UNCHECKED, [], ['The manifest labels no token account, so there is no table to sum. Give --manifest with every account that holds the token and its label.']);
  const addresses = labelled.map(a => a.address);
  const repeated = addresses.filter((a, i) => addresses.indexOf(a) !== i);
  const accounts = await rpc.accounts(addresses);
  const lines = [], bad = [];
  let sum = 0n;
  labelled.forEach((entry, i) => {
    let token;
    try { token = decodeTokenAccount(accounts[i]); } catch (e) { bad.push(`"${entry.label}" ${entry.address}: ${e.message}.`); return; }
    if (token.mint !== mintAddress) { bad.push(`"${entry.label}" ${entry.address} holds another mint (${token.mint}).`); return; }
    entry.amount = token.amount; entry.owner = token.owner;
    sum += token.amount;
    lines.push(`${entry.label}: ${tokens(token.amount, mint.decimals)} (${share(token.amount, mint.supply)}) in ${entry.address}, owner ${token.owner}${token.frozen ? ', FROZEN' : ''}.`);
  });
  for (const a of new Set(repeated)) bad.push(`${a} is labelled more than once.`);
  const gap = mint.supply - sum;
  const detail = [`${labelled.length} labelled account${labelled.length === 1 ? '' : 's'} hold ${tokens(sum, mint.decimals)} of a supply of ${tokens(mint.supply, mint.decimals)} (${share(sum, mint.supply)}).`];
  const notes = [];
  const holders = await holdersOf(ctx);
  const known = new Set(addresses);
  const strangers = holders.accounts.filter(h => !known.has(h.address) && h.amount > 0n);
  if (!holders.how) notes.push(`Cross-check not done: the cluster would not list this mint's holders (${holders.error}).`);
  else {
    const scope = holders.complete ? `all ${holders.accounts.length} accounts that hold the token` : `the cluster's ${holders.accounts.length} largest accounts`;
    if (strangers.length) {
      const total = strangers.reduce((s, h) => s + h.amount, 0n);
      detail.push(`Outside the table, among ${scope}: ${strangers.length} unlabelled holder${strangers.length === 1 ? '' : 's'} with ${tokens(total, mint.decimals)} (${share(total, mint.supply)}). Largest: ${strangers.slice(0, 5).map(h => `${h.address} ${tokens(h.amount, mint.decimals)} (${share(h.amount, mint.supply)})`).join('; ')}.`);
    } else detail.push(`Cross-check against ${scope}: no unlabelled holder.`);
    if (holders.how === 'getProgramAccounts') notes.push('The endpoint does not serve getTokenLargestAccounts; holders were read from the token program by mint instead.');
  }
  const data = { supply: mint.supply, labelled: sum, gap, holdersRead: holders.how, unlabelled: strangers.slice(0, 20), accounts: labelled.map(a => ({ label: a.label, address: a.address, amount: a.amount ?? null, owner: a.owner ?? null })) };
  if (bad.length) return row(5, FAIL, [mintAddress, ...addresses], [...bad, ...detail, ...lines], notes, data);
  if (gap === 0n) return row(5, PASS, [mintAddress, ...addresses], [...detail, ...lines], notes, data);
  detail.push(gap > 0n
    ? `${tokens(gap, mint.decimals)} (${share(gap, mint.supply)}) sits in accounts the table does not name. Once the pool has traded, its buyers hold tokens outside any table: this row is a check for before the first swap.`
    : `The labelled accounts hold ${tokens(-gap, mint.decimals)} more than the supply: the read straddled a transfer or a burn. Run again.`);
  return row(5, FAIL, [mintAddress, ...addresses], [...detail, ...lines], notes, data);
}

function row6() {
  return row(6, UNCHECKED, [], ['His word, not a read. A person checks that no presale, no deal off the market, no loan to a market maker and no allocation to promoters exists outside the table of row 5, and that any one that does exist is a row of that table with its lock.']);
}

// ---- row 7: the locks -------------------------------------------------------------------------------------------------------

async function row7(ctx) {
  const { rpc, mint, mintAddress, manifest, cluster, clock } = ctx;
  const locks = manifest?.locks || [];
  if (!locks.length) return row(7, UNCHECKED, [], ['The manifest names no lock. Give --manifest with each lock account (a Jupiter Lock escrow or a Streamflow stream) and its label.']);
  const accounts = await rpc.accounts(locks.map(l => l.address));
  const detail = [], notes = [], data = [];
  let failed = 0;
  const tableAddresses = new Set((manifest.accounts || []).map(a => a.address));
  for (let i = 0; i < locks.length; i++) {
    const { label, address } = locks[i];
    let lock;
    try { lock = decodeLock(address, accounts[i], cluster); } catch (e) { failed++; detail.push(`"${label}" ${address}: FAIL, ${e.message}.`); data.push({ label, address, ok: false, error: e.message }); continue; }
    ctx.programs.add(lock.kind === 'jupiter-lock' ? 'jupiter-lock' : 'streamflow');
    const why = [];
    if (lock.mint !== mintAddress) why.push(`it locks another mint (${lock.mint})`);
    if (lock.ended) why.push(`it is ${lock.ended}`);
    if (lock.cancelAuthority) why.push(`${lock.cancelAuthority} can cancel it`);
    if (lock.recipientAuthority) why.push(`${lock.recipientAuthority} can change its recipient`);
    if (lock.scheduleAuthority) why.push(`${lock.scheduleAuthority} can change it`);
    let held = null;
    try {
      const token = decodeTokenAccount(await rpc.account(lock.tokenAccount));
      held = token.amount;
      const owed = lock.total - lock.claimed;
      if (token.mint !== mintAddress) why.push(`its token account ${lock.tokenAccount} holds another mint`);
      else if (lock.kind === 'jupiter-lock' && token.owner !== address) why.push(`its token account ${lock.tokenAccount} is owned by ${token.owner}, not by the escrow`);
      else if (token.amount < owed) why.push(`its token account holds ${tokens(token.amount, mint.decimals)}, less than the ${tokens(owed, mint.decimals)} still owed`);
      if (token.frozen) why.push('its token account is frozen');
    } catch (e) {
      if (e.name === 'RpcError') throw e;
      why.push(`its token account ${lock.tokenAccount} does not read as a token account (${e.message})`);
    }
    const program = lock.kind === 'jupiter-lock' ? 'Jupiter Lock' : 'Streamflow';
    const step = lock.kind === 'jupiter-lock'
      ? `${tokens(lock.cliffAmount, mint.decimals)} at the cliff, then ${tokens(lock.perPeriod, mint.decimals)} every ${span(lock.frequency, 'second')}, ${lock.periods} times`
      : `${tokens(lock.cliffAmount, mint.decimals)} at the cliff, then ${tokens(lock.perPeriod, mint.decimals)} every ${span(lock.frequency, 'second')}`;
    const state = clock.time === null ? '' : BigInt(clock.time) < lock.cliff ? ' Nothing has unlocked yet.' : BigInt(clock.time) >= lock.end ? ' Fully unlocked now.' : ' Unlocking now.';
    detail.push(`"${label}" ${address}: ${why.length ? 'FAIL' : 'ok'}, ${program}. ${tokens(lock.total, mint.decimals)} (${share(lock.total, mint.supply)}) for ${lock.recipient}, made by ${lock.creator}. Starts ${when(lock.start)}, cliff ${when(lock.cliff)}, ends ${when(lock.end)}: ${step}. Claimed ${tokens(lock.claimed, mint.decimals)}.${state}${held === null ? '' : ` Its token account ${lock.tokenAccount} holds ${tokens(held, mint.decimals)}.`}${why.length ? ` Fails because ${why.join('; ')}.` : ' No cancel authority, no recipient-change authority.'}`);
    for (const n of lock.notes) notes.push(`"${label}": ${n}.`);
    if (tableAddresses.size && !tableAddresses.has(lock.tokenAccount)) notes.push(`"${label}": its token account ${lock.tokenAccount} is not in the manifest's table (row 5).`);
    if (why.length) failed++;
    data.push({ label, address, ok: why.length === 0, kind: lock.kind, recipient: lock.recipient, creator: lock.creator, start: lock.start, cliff: lock.cliff, end: lock.end, total: lock.total, claimed: lock.claimed, held, tokenAccount: lock.tokenAccount, cancelAuthority: lock.cancelAuthority, recipientAuthority: lock.recipientAuthority, scheduleAuthority: lock.scheduleAuthority, why });
  }
  detail.unshift(`${locks.length} lock${locks.length === 1 ? '' : 's'} named, ${locks.length - failed} sound.`);
  return row(7, failed ? FAIL : PASS, locks.map(l => l.address), detail, notes, { locks: data });
}

// ---- rows 8, 9, 10, 12: the pool ---------------------------------------------------------------------------------------------

async function loadPool(ctx) {
  const address = ctx.manifest?.pool?.address;
  if (!address) return null;
  const account = await ctx.rpc.account(address);
  ctx.programs.add('meteora-damm-v2');
  try { return { address, pool: decodePool(address, account) }; } catch (e) { return { address, error: e.message }; }
}

const NO_POOL = 'The manifest names no pool. Give --manifest with the pool\'s address.';

async function row8(ctx, loaded) {
  const { mintAddress, clock, rpc } = ctx;
  if (!loaded) return row(8, UNCHECKED, [], [NO_POOL]);
  if (loaded.error) return row(8, FAIL, loaded.address, [`${loaded.address} is not a Meteora DAMM v2 pool: ${loaded.error}.`]);
  const { pool, address } = loaded;
  const why = [];
  const side = pool.tokenAMint === mintAddress ? 'A' : pool.tokenBMint === mintAddress ? 'B' : null;
  if (!side) why.push(`The pool trades ${pool.tokenAMint} against ${pool.tokenBMint}: this mint is not in it.`);
  if (pool.status !== 0) why.push('The pool is disabled: nobody can swap.');
  const now = poolNow(pool, clock), unit = poolUnit(pool);
  const opened = unit === 'slot' ? `slot ${pool.activationPoint}` : when(pool.activationPoint);
  if (now < pool.activationPoint) why.push(`The pool is not open yet: it opens at ${opened}.`);
  if (pool.whitelistedVault) why.push(`A whitelisted vault, ${pool.whitelistedVault}, may buy before the pool opens to everyone: an allowlist on the first slot.`);
  const quote = side === 'A' ? pool.tokenBMint : pool.tokenAMint;
  const facts = [`Meteora DAMM v2 pool ${address}, ${POOL_TYPES[pool.poolType] ?? `type ${pool.poolType}`}, this mint against ${quote}, ${pool.creator === DEFAULT_PUBKEY ? 'creator not recorded' : `made by ${pool.creator}`}. Enabled: ${pool.status === 0 ? 'yes' : 'no'}. Open since ${opened}. Whitelisted vault: ${pool.whitelistedVault ?? 'none'}. ${pool.totalPosition} position${pool.totalPosition === 1n ? '' : 's'}.`];
  const notes = ["Not readable in the pool's account: Meteora's operators holding the SetPoolStatus permission can disable swaps on any DAMM v2 pool (cp_amm 0.2.4, set_pool_status). \"The pool never closes\" rests on them not doing so."];
  // "One" pool is the house's word: anyone may open another. What exists is counted so the page can say it.
  try {
    const sides = await Promise.all(['token_a_mint', 'token_b_mint'].map(f => rpc.programAccounts(DAMM_V2_PROGRAM, [{ dataSize: 8 + POOL.size }, { memcmp: { offset: 8 + POOL.at(f).offset, bytes: mintAddress } }], { offset: 0, length: 0 })));
    const others = [...new Set(sides.flat().map(a => a.address))].filter(a => a !== address);
    notes.push(others.length ? `${others.length} other DAMM v2 pool${others.length === 1 ? '' : 's'} hold this mint (anyone may open one): ${others.slice(0, 5).join(', ')}${others.length > 5 ? ', ...' : ''}.` : 'No other DAMM v2 pool holds this mint.');
  } catch (e) {
    if (e.name !== 'RpcError' || e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e;
    notes.push(`Other DAMM v2 pools for this mint were not counted (${e.message}).`);
  }
  const data = { pool: address, enabled: pool.status === 0, activationPoint: pool.activationPoint, activationUnit: unit, whitelistedVault: pool.whitelistedVault, creator: pool.creator, quote, poolType: pool.poolType };
  return row(8, why.length ? FAIL : PASS, address, [...why, ...facts], notes, data);
}

async function loadPositions(ctx, loaded) {
  if (!loaded?.pool) return [];
  const named = ctx.manifest?.pool?.positions || [];
  if (!named.length) return [];
  const accounts = await ctx.rpc.accounts(named);
  return named.map((address, i) => { try { return { address, position: decodePosition(address, accounts[i]) }; } catch (e) { return { address, error: e.message }; } });
}

async function row9(ctx, loaded, positions) {
  const { rpc } = ctx;
  if (!loaded) return row(9, UNCHECKED, [], [NO_POOL]);
  if (loaded.error) return row(9, UNCHECKED, loaded.address, ['The pool did not read (row 8), so its liquidity was not read.']);
  const { pool, address } = loaded;
  const locked = pool.permanentLockLiquidity, all = pool.liquidity;
  const facts = [`Of the pool's liquidity, ${share(locked, all)} is permanently locked (${locked} of ${all} units).`];
  const data = { liquidity: all, permanentLockLiquidity: locked, positions: [] };
  if (all === 0n) return row(9, FAIL, address, ['The pool holds no liquidity at all.'], [], data);
  if (!positions.length) {
    const notes = [];
    if (locked !== all) {
      // Help a person fill the manifest: the positions that exist, largest first.
      try {
        const found = await positionsOf(rpc, address);
        const size = p => p.unlocked + p.vested + p.permanentLocked;
        found.sort((a, b) => (size(b) > size(a) ? 1 : size(b) < size(a) ? -1 : 0));
        notes.push(`${found.length} position${found.length === 1 ? '' : 's'} exist. Largest: ${found.slice(0, 5).map(p => `${p.address} (${share(size(p), all)} of the pool; locked for good ${share(p.permanentLocked, size(p) || 1n)})`).join('; ')}.`);
      } catch (e) { if (e.name !== 'RpcError' || e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e; }
    }
    if (locked === all) return row(9, PASS, address, ['All of the pool\'s liquidity is permanently locked: no position can withdraw any.', ...facts], notes, data);
    if (locked === 0n) return row(9, FAIL, address, ['None of the pool\'s liquidity is permanently locked: every position can withdraw.', ...facts], notes, data);
    return row(9, UNCHECKED, address, [...facts, 'The rest can be withdrawn by whoever holds it. Name the house\'s positions in the manifest (pool.positions) and this row says whether theirs is locked.'], notes, data);
  }
  const lines = [], why = [];
  for (const p of positions) {
    if (p.error) { why.push(`${p.address} is not a position: ${p.error}.`); continue; }
    const q = p.position, size = q.unlocked + q.vested + q.permanentLocked;
    data.positions.push({ address: q.address, unlocked: q.unlocked, vested: q.vested, permanentLocked: q.permanentLocked });
    if (q.pool !== address) { why.push(`Position ${q.address} belongs to another pool (${q.pool}).`); continue; }
    if (positionAddress(q.nftMint) !== q.address) why.push(`Position ${q.address} is not at the address its NFT derives.`);
    lines.push(`Position ${q.address}: ${share(size, all)} of the pool; locked for good ${share(q.permanentLocked, size || 1n)}, vesting ${share(q.vested, size || 1n)}, free ${share(q.unlocked, size || 1n)}.`);
    if (size === 0n) why.push(`Position ${q.address} holds no liquidity.`);
    else if (q.unlocked > 0n) why.push(`Position ${q.address} can withdraw ${share(q.unlocked, size)} of its liquidity today.`);
    if (q.vested > 0n) why.push(`Position ${q.address} has ${share(q.vested, size || 1n)} of its liquidity on a vesting lock that ends: it can be withdrawn then.`);
  }
  if (why.length) return row(9, FAIL, [address, ...positions.map(p => p.address)], [...why, ...facts, ...lines], [], data);
  return row(9, PASS, [address, ...positions.map(p => p.address)], ['Every named position is locked for good: none of its liquidity can be withdrawn. Its fees stay claimable (row 12).', ...facts, ...lines], locked === all ? [] : ['Liquidity outside the named positions belongs to others and can be withdrawn by them.'], data);
}

function row10(ctx, loaded) {
  if (!loaded) return row(10, UNCHECKED, [], [NO_POOL]);
  if (loaded.error) return row(10, UNCHECKED, loaded.address, ['The pool did not read (row 8), so its fee was not read.']);
  const { pool, address } = loaded, { clock } = ctx;
  const s = feeSchedule(pool, clock);
  const unit = s.unit, at = t => (unit === 'slot' ? `slot ${t}` : when(t));
  const split = `Of each fee, ${pool.protocolFeePercent}% is the protocol's (Meteora's) and ${100 - pool.protocolFeePercent}% goes to the liquidity${pool.compoundingFeeBps ? `, ${pool.compoundingFeeBps / 100}% of that back into the pool` : ''}. Collected in ${COLLECT_FEE_MODES[pool.collectFeeMode] ?? `mode ${pool.collectFeeMode}`}. Dynamic (volatility) fee on top: ${pool.dynamicFee ? 'on' : 'off'}. The program caps the total at ${feePercent(pool.maxFeeNumerator)}.`;
  const notes = ["Not readable in the pool's account: Meteora's operators holding the UpdatePoolFees permission can set a new base fee once the schedule has ended, and the dynamic fee at any time (cp_amm 0.2.4, update_pool_fees). \"Set once at creation\" rests on them not doing so."];
  const data = { kind: s.kind, curve: s.curve, start: s.start, end: s.end, now: s.now, periods: s.periods ?? null, frequency: s.frequency ?? null, unit, endsAt: s.endsAt ?? null, protocolFeePercent: pool.protocolFeePercent, collectFeeMode: pool.collectFeeMode, dynamicFee: pool.dynamicFee };
  if (s.kind === 'time') {
    const shape = `Base fee: ${s.name}. Starts at ${feePercent(s.start)} when the pool opens (${at(s.activation)}) and falls to ${feePercent(s.end)} over ${s.periods} step${s.periods === 1 ? '' : 's'} of ${span(s.frequency, unit)}, reaching it at ${at(s.endsAt)}. Now: ${s.now === null ? 'not open yet' : feePercent(s.now)}.`;
    if (s.decays) return row(10, PASS, address, [shape, split], notes, data);
    return row(10, FAIL, address, [`The base fee is flat at ${feePercent(s.start)}: it does not start high and decay, so the first buyer pays what the last one does.`, split], notes, data);
  }
  if (s.kind === 'market-cap') {
    const shape = `Base fee: ${s.name}. Starts at ${feePercent(s.start)} and falls toward ${feePercent(s.end)} in ${s.periods} steps, one for every ${s.stepBps} bps the price rises above the opening price; at ${at(s.endsAt)} it drops to ${feePercent(s.end)} whatever the price. Now: ${s.now === null ? 'not open yet' : feePercent(s.now)}.`;
    if (s.decays) return row(10, PASS, address, [shape, 'It decays with the price, not the clock: a sniper at the opening price pays the starting fee.', split], notes, data);
    return row(10, FAIL, address, [`The base fee is flat at ${feePercent(s.start)}: it does not start high and decay.`, split], notes, data);
  }
  return row(10, FAIL, address, [`Base fee: ${s.name}. A flat ${feePercent(s.start)} that rises with the size of a buy (${s.incrementBps} bps for every ${s.referenceAmount} base units of quote, up to ${s.maxFeeBps} bps) for ${span(BigInt(s.limiterDuration), unit)} after opening. It taxes size, it does not start high and decay.`, split], notes, data);
}

async function row12(ctx, loaded, positions) {
  const { rpc, manifest, history } = ctx;
  const fund = manifest?.fund?.address;
  if (!loaded) return row(12, UNCHECKED, [], [NO_POOL]);
  if (loaded.error) return row(12, UNCHECKED, loaded.address, ['The pool did not read (row 8), so its fee claims were not read.']);
  if (!fund) return row(12, UNCHECKED, loaded.address, ['The manifest names no fund address. Give --manifest with fund.address, the wallet the fee is said to reach.']);
  const good = positions.filter(p => p.position && p.position.pool === loaded.address);
  if (!good.length) return row(12, UNCHECKED, loaded.address, ['The manifest names no position of this pool. Fees are claimed by whoever holds a position: name the house\'s positions (pool.positions) and this row reads who holds them and where each claim was paid.']);
  const detail = [], why = [], notes = [], data = [];
  let unknown = 0, truncated = false, count = 0;
  for (const { position: q } of good) {
    const holder = await positionHolder(rpc, q.nftMint);
    if (!holder) { unknown++; detail.push(`Position ${q.address}: its NFT ${q.nftMint} was not found in any token account the cluster would name, so its holder is unknown.`); continue; }
    const mine = holder.owner === fund;
    if (!mine) why.push(`Position ${q.address} is held by ${holder.owner}, not the fund: where its fees go is that wallet's choice at each claim.`);
    if (holder.delegate && q.delegateMayClaimAnywhere) why.push(`Position ${q.address} has a delegate, ${holder.delegate}, allowed to claim its fees into any account.`);
    else if (holder.delegate && q.delegateMayClaimToOwner) notes.push(`Position ${q.address} has a delegate, ${holder.delegate}, allowed to claim its fees into the holder's own accounts only.`);
    const { claims, signatures, truncated: cut } = await claimsOf(rpc, q.address, history);
    truncated ||= cut;
    let astray = 0;
    for (const c of claims) {
      count++;
      let lost = false;
      for (const side of ['A', 'B']) {
        const paid = c[`fee${side}`];
        if (paid === 0n) continue;                                  // nothing moved on this side
        const to = c[`to${side}`], owner = c[`to${side}Owner`], closedTo = c[`to${side}ClosedTo`];
        if (owner === null) { unknown++; detail.push(`Claim ${c.signature} (${when(c.time)}): paid token ${side} into ${to}, whose owner the transaction does not show.`); }
        else if (owner !== fund) { lost = true; why.push(`Claim ${c.signature} (${when(c.time)}) paid ${paid ?? 'an unread amount of'} base units of token ${side} into ${to}, owned by ${owner}, not the fund.`); }
        else if (closedTo && closedTo !== fund) { lost = true; why.push(`Claim ${c.signature} (${when(c.time)}) paid ${paid ?? 'an unread amount of'} base units of token ${side} into the fund's ${to}, closed in the same transaction with its balance sent to ${closedTo}.`); }
      }
      if (lost) astray++;
    }
    detail.push(`Position ${q.address}: held by ${holder.owner}${mine ? ' (the fund)' : ''} in ${holder.tokenAccount}. Claimed so far ${q.claimedA} base units of token A and ${q.claimedB} of token B; waiting ${q.feeAPending} and ${q.feeBPending}. ${claims.length} claim${claims.length === 1 ? '' : 's'} in its ${signatures} newest transaction${signatures === 1 ? '' : 's'}${claims.length ? `, ${claims.length - astray} to the fund` : ''}.`);
    data.push({ position: q.address, holder: holder.owner, holderIsFund: mine, delegate: holder.delegate, claims: claims.map(c => ({ signature: c.signature, time: c.time, signer: c.signer, feeA: c.feeA, feeB: c.feeB, toA: c.toA, toAOwner: c.toAOwner, toB: c.toB, toBOwner: c.toBOwner })) });
  }
  const source = [loaded.address, ...good.map(p => p.address), fund];
  if (why.length) return row(12, FAIL, source, [...why, ...detail], notes, { fund, positions: data });
  if (unknown) return row(12, UNCHECKED, source, [...detail, 'Something above could not be read, so this row is not decided.'], notes, { fund, positions: data });
  if (truncated) return row(12, UNCHECKED, source, [...detail, `A position has more than ${history} transactions; only the newest ${history} were read. Raise --history to read them all.`], notes, { fund, positions: data });
  return row(12, PASS, source, [`Every named position is held by the fund, ${fund}, so only the fund can claim, and ${count ? `all ${count} claim${count === 1 ? '' : 's'} so far were paid into accounts the fund owns` : 'no claim has been made yet'}.`, ...detail], [...notes, 'A claim lands where the holder points it. This row reads every claim made so far and who may make the next one; it cannot read the next one.'], { fund, positions: data });
}

// ---- row 11: the house's wallets ---------------------------------------------------------------------------------------------

/** What a transaction did to this mint, account by account: the balance change, and who owned the account before and after. */
export function mintMoves(tx, mintAddress) {
  const keys = tx.transaction.message.accountKeys.map(k => (typeof k === 'string' ? k : k.pubkey));
  const before = new Map(), after = new Map();
  for (const b of tx.meta?.preTokenBalances || []) if (b.mint === mintAddress) before.set(keys[b.accountIndex], { amount: BigInt(b.uiTokenAmount.amount), owner: b.owner ?? null });
  for (const b of tx.meta?.postTokenBalances || []) if (b.mint === mintAddress) after.set(keys[b.accountIndex], { amount: BigInt(b.uiTokenAmount.amount), owner: b.owner ?? null });
  const moves = [];
  for (const account of new Set([...before.keys(), ...after.keys()])) {
    const b = before.get(account), a = after.get(account);
    moves.push({ account, ownerBefore: b?.owner ?? a?.owner ?? null, ownerAfter: a?.owner ?? b?.owner ?? null, before: b?.amount ?? 0n, after: a?.amount ?? 0n });
  }
  return moves;
}

/** Each house wallet's net loss of the mint in one transaction, and where it went. */
export function outflowsIn(tx, mintAddress, wallets) {
  const moves = mintMoves(tx, mintAddress);
  const out = [];
  for (const wallet of wallets) {
    let lost = 0n;
    const handed = [];                                              // accounts whose ownership left the wallet
    for (const m of moves) {
      if (m.ownerBefore !== wallet) continue;
      if (m.ownerAfter === wallet) lost += m.before - m.after;
      else { lost += m.before; handed.push({ account: m.account, owner: m.ownerAfter, amount: m.after }); }
    }
    for (const m of moves) if (m.ownerBefore !== wallet && m.ownerAfter === wallet) lost -= m.after;       // an account handed to the wallet
    if (lost <= 0n) continue;
    const gained = moves.filter(m => m.ownerBefore !== wallet && m.ownerAfter !== wallet && m.after > m.before).map(m => ({ account: m.account, owner: m.ownerAfter, amount: m.after - m.before }));
    const to = [...handed, ...gained];
    const placed = to.reduce((s, d) => s + d.amount, 0n);
    if (placed < lost) to.push({ account: null, owner: null, amount: lost - placed, burned: true });
    out.push({ wallet, amount: lost, to });
  }
  return out;
}

/** Take an outflow's destinations off the published schedule; what the schedule does not cover is returned. */
function offSchedule(flow, time, schedule) {
  const left = [];
  for (const d of flow.to) {
    let need = d.amount;
    for (const s of schedule) {
      if (need === 0n) break;
      if (s.from !== flow.wallet || s.remaining === 0n) continue;
      const toMatches = d.burned ? s.to === 'burn' : (s.to === d.account || s.to === d.owner);
      if (!toMatches) continue;
      if (time !== null && ((s.notBefore !== null && time < s.notBefore) || (s.notAfter !== null && time > s.notAfter))) continue;
      const take = need < s.remaining ? need : s.remaining;
      s.remaining -= take; need -= take;
    }
    if (need > 0n) left.push({ ...d, amount: need });
  }
  return left;
}

async function row11(ctx, loaded) {
  const { rpc, mint, mintAddress, manifest, history } = ctx;
  const wallets = manifest?.houseWallets || [];
  if (!wallets.length) return row(11, UNCHECKED, [], ['The manifest names no house wallet. Give --manifest with houseWallets (deployer, fund, founders, the twelve) and, when there is one, the published schedule.']);
  const addresses = wallets.map(w => w.address);
  const labelOf = new Map(wallets.map(w => [w.address, w.label]));
  const schedule = (manifest.schedule || []).map(s => ({ ...s, amount: BigInt(s.amount), remaining: BigInt(s.amount), notBefore: s.notBefore ?? null, notAfter: s.notAfter ?? null }));
  const signatures = new Map();                                     // signature -> { slot, time }
  const detail = [], notes = [];
  let truncated = false;
  for (const w of wallets) {
    const held = await rpc.tokenAccountsByOwner(w.address, mintAddress);
    let balance = 0n;
    for (const a of held) balance += decodeTokenAccount(a).amount;
    const asked = [w.address, ...held.map(a => a.address)];
    let seen = 0;
    for (const address of asked) {
      const sigs = await rpc.signatures(address, history);
      if (sigs.length >= history) truncated = true;
      for (const s of sigs) { if (!s.err && !signatures.has(s.signature)) { signatures.set(s.signature, { slot: s.slot, time: s.blockTime ?? null }); } }
      seen += sigs.length;
    }
    detail.push(`${w.label} ${w.address}: holds ${tokens(balance, mint.decimals)} (${share(balance, mint.supply)}) in ${held.length} account${held.length === 1 ? '' : 's'}; ${seen} transaction${seen === 1 ? '' : 's'} listed.`);
  }
  const flagged = [], onSchedule = [];
  const ordered = [...signatures.entries()].sort((a, b) => a[1].slot - b[1].slot);
  for (const [signature, meta] of ordered) {
    const tx = await rpc.transaction(signature);
    if (!tx || tx.meta?.err) continue;
    const time = tx.blockTime ?? meta.time;
    for (const flow of outflowsIn(tx, mintAddress, addresses)) {
      const left = offSchedule(flow, time, schedule);
      const record = { signature, time, wallet: flow.wallet, amount: flow.amount, to: flow.to };
      if (left.length) flagged.push({ ...record, unscheduled: left }); else onSchedule.push(record);
    }
  }
  const say = d => (d.burned ? 'burned' : `${d.account}${d.owner ? ` (owner ${d.owner}${labelOf.has(d.owner) ? `, ${labelOf.get(d.owner)}` : ''})` : ''}`);
  const total = f => f.unscheduled.reduce((sum, d) => sum + d.amount, 0n);
  for (const f of flagged.slice(0, 8)) detail.push(`OFF SCHEDULE ${when(f.time)} ${f.signature}: ${labelOf.get(f.wallet)} ${f.wallet} sent ${tokens(total(f), mint.decimals)} (${share(total(f), mint.supply)}) to ${f.unscheduled.length} account${f.unscheduled.length === 1 ? '' : 's'}: ${f.unscheduled.slice(0, 2).map(d => `${tokens(d.amount, mint.decimals)} to ${say(d)}`).join(', ')}${f.unscheduled.length > 2 ? `, and ${f.unscheduled.length - 2} more` : ''}.`);
  if (flagged.length > 8) detail.push(`And ${flagged.length - 8} more off the schedule (all of them are in --json).`);
  if (loaded?.pool && loaded.pool.creator !== DEFAULT_PUBKEY && !addresses.includes(loaded.pool.creator)) notes.push(`The pool's creator, ${loaded.pool.creator}, is not among the named house wallets.`);
  if (!schedule.length) notes.push('The manifest carries no schedule, so every transfer out of a house wallet is flagged.');
  const unused = schedule.filter(s => s.remaining === s.amount);
  if (schedule.length) notes.push(`${schedule.length - unused.length} of ${schedule.length} schedule line${schedule.length === 1 ? '' : 's'} matched a transfer${unused.length ? `; not yet seen on chain: ${unused.map(s => s.label || `${s.from} to ${s.to}`).join(', ')}` : ''}.`);
  const data = { wallets: wallets.map(w => ({ label: w.label, address: w.address })), transactions: signatures.size, flagged, onSchedule: onSchedule.length };
  const summary = `${wallets.length} house wallet${wallets.length === 1 ? '' : 's'} named, ${signatures.size} transaction${signatures.size === 1 ? '' : 's'} read, ${flagged.length + onSchedule.length} moved this token out of a house wallet: ${onSchedule.length} on the published schedule, ${flagged.length} off it.`;
  if (flagged.length) return row(11, FAIL, addresses, [summary, ...detail], notes, data);
  if (truncated) return row(11, UNCHECKED, addresses, [summary, ...detail, `A wallet or one of its token accounts has more than ${history} transactions; only the newest ${history} were read. Raise --history to read them all.`], notes, data);
  return row(11, PASS, addresses, [summary, ...detail], [...notes, 'A token account a house wallet emptied and closed is seen only through the wallet\'s own transactions; a wallet the manifest does not name is not seen at all.'], data);
}

// ---- row 13: the burn book's digest -----------------------------------------------------------------------------------------

// The line house/tools/burn-digest prints for a closed UTC day (its memoOf): one of these a day, anchored as a memo.
const BURN_LINE = /burn-digest\/1 (\d{4}-\d{2}-\d{2}) rows=(\d+) cost_usd=(-?\d+\.\d{6}) sha256=([0-9a-f]{64})/g;
const DAY_S = 86400, SETTLE_S = 600;                               // a day closes ten minutes after its UTC midnight
const dayOf = t => new Date(t * 1000).toISOString().slice(0, 10);

/** The burn-digest lines in a memo string as getSignaturesForAddress gives it ("[92] burn-digest/1 ..."). */
export function burnLines(memo) {
  return [...String(memo || '').matchAll(BURN_LINE)].map(m => ({ day: m[1], rows: Number(m[2]), costUsd: m[3], sha256: m[4] }));
}

async function row13(ctx) {
  const { rpc, manifest, history, clock, digests } = ctx;
  const address = manifest?.burnDigest?.address;
  const book = 'Whether a line is true is read off chain: a person recomputes that day\'s hash from its receipts in the burn book (house/tools/burn-digest, `node digest.mjs verify`).';
  if (!address) return row(13, UNCHECKED, [], [`The manifest names no address that anchors the burn book's daily digest (burnDigest.address). A person looks for one Memo transaction a day from that address carrying "burn-digest/1 <day> rows=<n> cost_usd=<total> sha256=<hash>". ${book}`]);
  const sigs = await rpc.signatures(address, history);
  const cut = sigs.length >= history;
  const lines = [];
  let foreign = 0;
  for (const s of sigs) {
    if (s.err) continue;
    const found = burnLines(s.memo);
    if (!found.length) continue;
    // The list holds every transaction that names the address, and anyone can name it beside a look-alike memo: a line
    // counts only when the address itself signed the transaction that carries it.
    const tx = await rpc.transaction(s.signature);
    const signed = !!tx && !tx.meta?.err && tx.transaction.message.accountKeys.some(k => typeof k === 'object' && k.pubkey === address && k.signer);
    if (!signed) { foreign += found.length; continue; }
    for (const l of found) lines.push({ ...l, signature: s.signature, time: s.blockTime ?? null });
  }
  const strangers = foreign ? [`${foreign} line${foreign === 1 ? '' : 's'} in transactions ${address} did not sign ${foreign === 1 ? 'was' : 'were'} not counted: anyone can name an address beside a memo.`] : [];
  if (!lines.length) {
    const none = `${address} has anchored no burn-digest line in its ${sigs.length} newest transaction${sigs.length === 1 ? '' : 's'}.`;
    return cut ? row(13, UNCHECKED, address, [`${none} It has more than ${history}; raise --history to read further back.`, book], strangers) : row(13, FAIL, address, [`${none} The spend is not on chain.`, book], strangers);
  }
  const byDay = new Map();
  for (const l of lines) { if (!byDay.has(l.day)) byDay.set(l.day, []); byDay.get(l.day).push(l); }
  const days = [...byDay.keys()].sort();
  const first = days[0], last = days[days.length - 1];
  const why = [];
  const split = days.filter(d => new Set(byDay.get(d).map(l => l.sha256)).size > 1);
  if (split.length) why.push(`${split.length} day${split.length === 1 ? ' has' : 's have'} two different hashes on chain: ${split.slice(0, 5).join(', ')}.`);
  const missing = [];
  const due = clock.time === null ? last : dayOf(clock.time - SETTLE_S - DAY_S);                 // the last day that has closed
  for (let t = Date.parse(`${first}T00:00:00Z`) / 1000; dayOf(t) <= due; t += DAY_S) if (!byDay.has(dayOf(t))) missing.push(dayOf(t));
  if (missing.length) why.push(`${missing.length} closed day${missing.length === 1 ? ' has' : 's have'} no line between ${first} and ${due}: ${missing.slice(0, 7).join(', ')}${missing.length > 7 ? ', ...' : ''}. A missing line is a missing report.`);
  let compared = 0;
  if (digests) {
    for (const d of days) {
      const mine = digests.get(d), theirs = byDay.get(d)[0];
      if (!mine) { why.push(`${d} is on chain and not in the digests record given.`); continue; }
      compared++;
      if (mine.digest !== theirs.sha256 || Number(mine.rows) !== theirs.rows || String(mine.cost_usd) !== theirs.costUsd) why.push(`${d}: the line on chain (${theirs.sha256.slice(0, 16)}..., rows ${theirs.rows}, ${theirs.costUsd} USD) is not the record's (${String(mine.digest).slice(0, 16)}..., rows ${mine.rows}, ${mine.cost_usd} USD).`);
    }
  }
  const newest = byDay.get(last)[0];
  const facts = [`${days.length} day${days.length === 1 ? '' : 's'} anchored by ${address}, ${first} to ${last}. Newest: ${last}, ${newest.rows} receipts, ${newest.costUsd} USD, sha256 ${newest.sha256} (${newest.signature}).${digests ? ` ${compared} compared with the digests record given.` : ''}`];
  const data = { address, days: days.length, first, last, missing, conflicting: split, compared, notSigned: foreign, lines: lines.slice(0, 40) };
  if (why.length) return row(13, FAIL, address, [...why, ...facts, book], strangers, data);
  if (cut) return row(13, UNCHECKED, address, [...facts, `The address has more than ${history} transactions; only the newest ${history} were read, so earlier days were not checked. Raise --history.`, book], strangers, data);
  return row(13, PASS, address, [`One line for every closed day from ${first} to ${due}, none missing, none contradicted, each signed by ${address}.`, ...facts], [book, ...strangers], data);
}

function row14() {
  return row(14, UNCHECKED, [], ['A page, not an account. A person opens "State of $WARM" and checks that it shows what rows 1 to 13 show here, read live from chain.']);
}

// ---- the programs the rows stood on ------------------------------------------------------------------------------------------

async function programStates(ctx) {
  const out = [];
  for (const key of ctx.programs) {
    const src = SOURCES.programs[key];
    const address = key === 'streamflow' && ctx.cluster === 'devnet' ? src.addressDevnet : src.address;
    const entry = { key, name: src.name, address, layoutRead: SOURCES.readOn, deployedSlotAtReading: src.deployedSlot, deployedSlot: null, upgradeAuthority: null, moved: null };
    try {
      const state = await deployState(ctx.rpc, address);
      if (state) { entry.deployedSlot = state.slot; entry.upgradeAuthority = state.upgradeAuthority; entry.moved = ctx.cluster === 'mainnet' ? state.slot > src.deployedSlot : null; }
    } catch (e) { if (e.name !== 'RpcError' || e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e; entry.error = e.message; }
    out.push(entry);
  }
  return out;
}

const ROWS_OF_PROGRAM = { 'spl-token': [2, 3, 5], 'token-2022': [2, 3, 4, 5], 'metaplex-token-metadata': [4], 'jupiter-lock': [7], streamflow: [7], 'meteora-damm-v2': [8, 9, 10, 12] };

// ---- the run ----------------------------------------------------------------------------------------------------------------

/**
 * Run the fourteen rows for one mint. `rpc` is a createRpc() of lib/rpc.mjs; `resolveTxt` is dns.promises.resolveTxt or a
 * stand-in; `digests` is a Map of day -> { rows, cost_usd, digest } read from burn-digest's record, or null. Nothing here
 * writes anywhere.
 */
export async function runChecks({ rpc, mint: mintAddress, cluster = 'mainnet', manifest = null, domain = null, resolveTxt = null, history = 200, digests = null }) {
  if (!isAddress(mintAddress)) throw new UsageError(`${mintAddress} is not a Solana address`);
  const started = Date.now();
  const clock = await rpc.clock();
  const account = await rpc.account(mintAddress);
  let mint;
  try { mint = decodeMint(account); } catch (e) { throw new UsageError(`${mintAddress} is not a token mint on ${cluster}: ${e.message}`); }
  const ctx = { rpc, cluster, mintAddress, mint, clock, manifest, domain, resolveTxt, history, digests, programs: new Set([mint.token2022 ? 'token-2022' : 'spl-token']) };
  if (!isTokenProgram(account.owner)) throw new UsageError('not a token program');

  const rows = [];
  rows.push(await row1(ctx));
  rows.push(row2(ctx));
  rows.push(row3(ctx));
  rows.push(await guarded(4, metadataAddress(mintAddress), () => row4(ctx)));
  rows.push(await guarded(5, mintAddress, () => row5(ctx)));
  rows.push(row6());
  rows.push(await guarded(7, (manifest?.locks || []).map(l => l.address), () => row7(ctx)));
  let loaded = null, positions = [], poolDown = null;
  try { loaded = await loadPool(ctx); positions = await loadPositions(ctx, loaded); } catch (e) {
    if (e.name !== 'RpcError' || e.code === 'NOT_RECORDED' || e.code === 'REFUSED') throw e;
    poolDown = e.message;
  }
  const poolAddress = manifest?.pool?.address;
  const down = n => row(n, UNCHECKED, poolAddress, [`The cluster did not answer for the pool: ${poolDown}. Run again, or name another endpoint with --rpc.`]);
  rows.push(poolDown ? down(8) : await guarded(8, poolAddress, () => row8(ctx, loaded)));
  rows.push(poolDown ? down(9) : await guarded(9, poolAddress, () => row9(ctx, loaded, positions)));
  rows.push(poolDown ? down(10) : row10(ctx, loaded));
  rows.push(await guarded(11, (manifest?.houseWallets || []).map(w => w.address), () => row11(ctx, loaded)));
  rows.push(poolDown ? down(12) : await guarded(12, poolAddress, () => row12(ctx, loaded, positions)));
  rows.push(await guarded(13, manifest?.burnDigest?.address, () => row13(ctx)));
  rows.push(row14());
  rows.sort((a, b) => a.n - b.n);

  const programs = await programStates(ctx);
  for (const p of programs) {
    if (!p.moved) continue;
    for (const n of ROWS_OF_PROGRAM[p.key] || []) {
      const r = rows[n - 1];
      if (r.verdict !== UNCHECKED || r.source.length) r.notes.push(`${p.name} was redeployed at slot ${p.deployedSlot}, after its layout was read (${SOURCES.readOn}, slot ${p.deployedSlotAtReading}): read its layout again before trusting this row.`);
    }
  }
  const count = v => rows.filter(r => r.verdict === v).length;
  return {
    tool: 'warm-day0', mint: mintAddress, cluster, slot: clock.slot, time: clock.time,
    supply: mint.supply, decimals: mint.decimals, tokenProgram: mint.program,
    rows, programs,
    summary: { pass: count(PASS), fail: count(FAIL), notCheckable: count(UNCHECKED) },
    cost: { rpcCalls: rpc.stats.calls, byMethod: { ...rpc.stats.byMethod }, ms: Date.now() - started },
  };
}

export { TOKEN_PROGRAM, TOKEN_2022_PROGRAM, METADATA_PROGRAM, JUP_LOCK_PROGRAM, STREAMFLOW_PROGRAM };
