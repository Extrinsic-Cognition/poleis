// His Keystones approve. Two roads to the same signature on one Squads proposalApprove:
//   by QR, on this page alone: the Keystone's account export is read from the camera, the request is shown as a QR the Keystone
//   scans, and its answer is read back from the camera (ur.mjs, sol.mjs: the open UR and Keystone registry formats, no wallet);
//   through a Wallet Standard wallet that holds them (Solflare), when one is installed.
// The page builds that one legacy transaction itself (the member pays its fee and signs) and sends it to devnet. The pull's
// addresses ride the link's hash, written by poleis/ceremony/keystones.mjs.
import { b58decode32, b58encode } from './b58.mjs';
import { URDecoder } from './ur.mjs';
import { readAccounts, readSignature, signRequest, toHex } from './sol.mjs';
import { b64, readSet, signedWire, toSign, unb64 } from './prepared.mjs';

const $ = (id) => document.getElementById(id);
const S = Object.fromEntries(new URLSearchParams(location.hash.slice(1)));
const CHAIN = 'solana:devnet';
const members = (S.members || '').split(',').filter(Boolean);
const shortvec = (n) => { const o = []; do { let b = n & 0x7f; n >>= 7; if (n) b |= 0x80; o.push(b); } while (n); return o; };
const concat = (parts) => Uint8Array.from(parts.flatMap((p) => [...p]));
const short = (a) => `${a.slice(0, 4)}…${a.slice(-4)}`;
function say(text, bad = false) { $('say').textContent = text; $('say').className = bad ? 'bad' : ''; }

async function rpc(method, params) {
  const r = await fetch(S.rpc, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }) });
  const j = await r.json();
  if (j.error) throw new Error(`${method}: ${j.error.message}${j.error.data?.logs ? '\n' + j.error.data.logs.join('\n') : ''}`);
  return j.result;
}

// ---- the Keystones connected, kept in this browser only ----
const KEY = 'poleis.keystones.v2';
let mine = [];
try { mine = JSON.parse(localStorage.getItem(KEY) || '[]'); } catch { mine = []; }
const keep = () => { try { localStorage.setItem(KEY, JSON.stringify(mine)); } catch { /* a private window keeps them in this tab */ } };
function add(k) {
  mine = mine.filter((x) => x.address !== k.address);
  mine.push(k); keep(); renderMine(); refresh(); renderPrepared();
  const isMember = !members.length || members.includes(k.address);
  say(`Connected ${short(k.address)}${members.length ? (isMember ? ": one of the vault's voters." : ": not one of the vault's voters.") : '.'}`, members.length && !isMember);
}
function renderMine() {
  $('mine').innerHTML = mine.map((k, i) => `<div class="row"><span class="tag">Keystone ${i + 1}</span><span class="addr">${k.address}</span><span class="tag">${k.via === 'qr' ? `by QR · ${k.path}` : `via ${k.wallet}`}</span></div>`).join('') || '<div class="row"><span class="tag">none yet</span></div>';
  $('copy').disabled = mine.length === 0;
}
$('copy').onclick = async () => { await navigator.clipboard.writeText(mine.map((k) => k.address).join(',')); say(`Copied ${mine.length} address${mine.length === 1 ? '' : 'es'}: paste them to Claude.`); };

// ---- the stage: one QR shown, the camera reading ----
let anim = null, stream = null, reading = null;
function showParts(parts) {
  clearInterval(anim);
  window.keystoneRequest = { parts };
  const box = $('qr');
  box.hidden = false;
  let i = 0;
  const draw = () => {
    const q = qrcode(0, 'L');
    q.addData(parts[i % parts.length].toUpperCase(), 'Alphanumeric');
    q.make();
    const n = q.getModuleCount(), cell = 6, size = (n + 8) * cell;
    let c = box.querySelector('canvas');
    if (!c) { c = document.createElement('canvas'); box.replaceChildren(c); }
    c.width = c.height = size;
    const g = c.getContext('2d');
    g.fillStyle = '#fff'; g.fillRect(0, 0, size, size); g.fillStyle = '#000';
    for (let r = 0; r < n; r++) for (let col = 0; col < n; col++) if (q.isDark(r, col)) g.fillRect((col + 4) * cell, (r + 4) * cell, cell, cell);
    i++;
  };
  draw();
  if (parts.length > 1) anim = setInterval(draw, 250);
}
function hideStage() { clearInterval(anim); anim = null; $('qr').hidden = true; stopCamera(); $('stage').className = ''; $('step').textContent = ''; }

async function startCamera(onText) {
  reading = onText;
  $('stage').className = 'on';
  if (!('BarcodeDetector' in window)) { say('This browser cannot read a QR from the camera. Open the page in Brave or Chrome.', true); return; }
  const detector = new BarcodeDetector({ formats: ['qr_code'] });
  stream = await navigator.mediaDevices.getUserMedia({ video: { width: { ideal: 1280 } }, audio: false });
  const v = $('cam'); v.hidden = false; v.srcObject = stream; await v.play();
  let last = '';
  const tick = async () => {
    if (!stream) return;
    try { for (const c of await detector.detect(v)) if (c.rawValue && c.rawValue !== last) { last = c.rawValue; reading && reading(c.rawValue); } } catch { /* a frame not ready yet */ }
    if (stream) setTimeout(tick, 120);
  };
  tick();
}
function stopCamera() {
  if (stream) for (const t of stream.getTracks()) t.stop();
  stream = null; reading = null; $('cam').hidden = true;
}
// what the camera reads, and what a test feeds the same way
window.keystoneScan = (text) => reading && reading(text);

// ---- connect by QR: the Keystone's account export ----
$('scanKeystone').onclick = async () => {
  const d = new URDecoder();
  $('picks').innerHTML = '';
  $('step').textContent = "Hold the Keystone's QR up to the camera.";
  try {
    await startCamera((text) => {
      try {
        if (!d.receive(text)) { $('step').textContent = `Reading the Keystone's QR… ${Math.round(d.progress * 100)}%`; return; }
        if (d.type !== 'crypto-multi-accounts' && d.type !== 'crypto-hdkey') throw new Error(`that QR is a ${d.type}, not an account export: on the Keystone choose Connect Software Wallet, then Solflare`);
        hideStage();
        const r = readAccounts(d.result, d.type);
        if (!r.accounts.length) throw new Error('that export holds no Solana account');
        $('picks').innerHTML = `<h3>${r.device || 'Keystone'} · fingerprint ${r.xfp}</h3>` + r.accounts.map((a, i) =>
          `<div class="row"><span class="addr">${a.address}</span><span class="tag">${a.path}</span><button class="quiet" data-i="${i}">Use</button></div>`).join('');
        for (const b of $('picks').querySelectorAll('button')) b.onclick = () => { const a = r.accounts[+b.dataset.i]; add({ address: a.address, via: 'qr', path: a.path, xfp: r.xfp, device: r.device }); $('picks').innerHTML = ''; };
        say(members.length ? 'Pick the account that is one of the vault\'s voters.' : 'Pick the account you use on this Keystone.');
      } catch (e) { hideStage(); say(String(e.message || e), true); }
    });
  } catch (e) { hideStage(); say(`The camera: ${e.message || e}`, true); }
};

// ---- connect through a Wallet Standard wallet ----
const wallets = [];
function register(...ws) {
  for (const w of ws) if (!wallets.includes(w) && w.features?.['solana:signTransaction'] && w.features?.['standard:connect']) wallets.push(w);
  renderWallets();
  return () => {};
}
window.addEventListener('wallet-standard:register-wallet', ({ detail }) => detail({ register }));
window.dispatchEvent(new CustomEvent('wallet-standard:app-ready', { detail: { register } }));
function renderWallets() {
  const box = $('wallets');
  box.innerHTML = '';
  if (!wallets.length) { box.innerHTML = '<span class="tag">no wallet in this browser: the QR road above needs none</span>'; return; }
  for (const w of wallets) {
    const b = document.createElement('button');
    b.className = 'quiet';
    b.textContent = `Connect with ${w.name}`;
    b.onclick = async () => {
      try {
        const { accounts } = await w.features['standard:connect'].connect();
        const account = accounts[0] || w.accounts[0];
        if (!account) throw new Error('the wallet gave no account');
        add({ address: account.address, via: 'wallet', wallet: w.name });
      } catch (e) { say(String(e.message || e), true); }
    };
    box.append(b);
  }
}

// ---- the pull ----
let proposal = null;
async function proposalState() {
  const v = (await rpc('getAccountInfo', [S.proposal, { encoding: 'base64', commitment: 'confirmed' }])).value;
  if (!v) return null;
  const d = unb64(v.data[0]);
  const variant = d[48];
  let at = 49 + (variant === 4 ? 0 : 8) + 1;
  const n = d[at] | (d[at + 1] << 8) | (d[at + 2] << 16) | (d[at + 3] << 24); at += 4;
  const approved = [];
  for (let i = 0; i < n; i++) approved.push(b58encode(d.subarray(at + 32 * i, at + 32 * (i + 1))));
  return { status: ['Draft', 'Active', 'Rejected', 'Approved', 'Executing', 'Executed', 'Cancelled'][variant], approved };
}
async function refresh() {
  if (!S.proposal) return;
  try {
    proposal = await proposalState();
    if (!proposal) { $('approvals').textContent = 'the proposal is not on chain yet'; return; }
    const who = proposal.approved.map((a) => short(a)).join(', ');
    $('approvals').innerHTML = `${proposal.approved.length} of ${S.threshold || 2}${who ? ` · ${who}` : ''} · <span class="${proposal.status === 'Executed' ? 'ok' : ''}">${proposal.status}</span>`;
    renderApprovers();
  } catch (e) { $('approvals').textContent = String(e.message || e); }
}
function renderApprovers() {
  const box = $('approvers');
  box.innerHTML = '';
  mine.forEach((k, i) => {
    if (!members.includes(k.address)) return;
    const b = document.createElement('button');
    const done = proposal?.approved.includes(k.address);
    b.textContent = done ? `Keystone ${i + 1} approved` : `Approve with Keystone ${i + 1} (${k.via === 'qr' ? 'by QR' : k.wallet})`;
    b.disabled = done || proposal?.status !== 'Active';
    b.onclick = () => approve(k);
    box.append(b);
  });
  if (!box.children.length) box.innerHTML = '<span class="tag">connect a Keystone that is one of the vault\'s voters</span>';
}

/** One proposalApprove, the member its fee payer and only signer: [member (w,s), proposal (w), multisig, Squads]. */
async function approveMessage(member) {
  const disc = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode('global:proposal_approve'))).subarray(0, 8);
  const data = concat([disc, [0]]); // memo: none
  const { value } = await rpc('getLatestBlockhash', [{ commitment: 'confirmed' }]);
  const keys = [member, S.proposal, S.multisig, S.squads].map(b58decode32);
  return concat([[1, 0, 2], shortvec(4), ...keys, b58decode32(value.blockhash), shortvec(1), [3], shortvec(3), [2, 0, 1], shortvec(data.length), data]);
}

/** One signed transaction sent to #rpc= and waited for until the chain confirms or refuses it; its signature. */
async function land(wire, where) {
  const sig = await rpc('sendTransaction', [b64(wire), { encoding: 'base64', preflightCommitment: 'confirmed' }]);
  say(`Sent ${short(sig)}. Waiting for ${where}…`);
  for (let i = 0; i < 60; i++) {
    const st = (await rpc('getSignatureStatuses', [[sig]])).value[0];
    if (st?.err) throw new Error(`${where} refused it: ${JSON.stringify(st.err)}`);
    if (st && st.confirmationStatus !== 'processed') break;
    await new Promise((r) => setTimeout(r, 1500));
  }
  return sig;
}
async function submit(signed, who) {
  await land(signed, 'devnet');
  say(`Approved by ${short(who)}. Approve with a second Keystone, or tell Claude it is done.`);
  await refresh();
}

async function approve(k) {
  try {
    say('Building the approval…');
    const message = await approveMessage(k.address);
    await submit(signedWire(message, await askKeystone(k, message, CHAIN)), k.address);
  } catch (e) { hideStage(); say(String(e.message || e), true); await refresh(); }
}

// ---- a prepared set: transactions built unsigned elsewhere (tools/warm-launch), signed here one by one, in order ----
// The chain is the set's (a public cluster, or a local chain by its own genesis hash); the page sends to #rpc= and refuses when that endpoint's genesis is another.
let prepared = null;
const landed = new Map();
async function loadPrepared(text) {
  try {
    prepared = await readSet(text);
    landed.clear();
    $('preparedWhat').textContent = (prepared.act ? `act ${prepared.act.n} (${prepared.act.name}), its hash ${prepared.act.hash}. ` : '') + (prepared.what || `${prepared.txs.length} transactions`);
    renderPrepared();
    say(prepared.signable ? `${prepared.txs.length} transactions to sign, in order.` : `Read, not signable: ${prepared.placeholders.join(', ')} are still his to fill.`, !prepared.signable);
  } catch (e) { prepared = null; say(`The set: ${e.message || e}`, true); }
}
$('preparedFile').onchange = async (ev) => { const f = ev.target.files[0]; if (f) await loadPrepared(await f.text()); };
window.keystonePrepared = loadPrepared; // what a test feeds the same way
function renderPrepared() {
  if (!prepared) return;
  $('preparedCluster').innerHTML = prepared.cluster === 'mainnet' ? '<span class="warn">mainnet: real SOL and tokens move when a Keystone signs</span>' : prepared.cluster === 'localnet' ? `a local chain, genesis ${short(prepared.genesis)}: a rehearsal, nothing real moves` : 'devnet';
  const box = $('preparedTxs');
  box.replaceChildren();
  prepared.txs.forEach((tx, i) => {
    const row = document.createElement('div');
    row.className = 'row';
    const tag = document.createElement('span'); tag.className = 'tag'; tag.textContent = `${tx.n}. ${tx.name}`;
    const what = document.createElement('span');
    for (const line of [...(tx.does || []), ...tx.shown.map((x) => `the Keystone shows the program ${x.program} and the data ${x.data}`), `${tx.when || 'in order'} · ${tx.bytes} bytes · signer ${short(tx.signer)}`]) { const l = document.createElement('span'); l.className = 'does'; l.textContent = line; what.append(l); }
    const k = mine.find((x) => x.address === tx.signer);
    const sig = landed.get(i);
    const b = document.createElement('button');
    b.textContent = sig ? `Landed ${short(sig)}` : !prepared.signable ? 'not signable' : k ? `Sign with Keystone ${mine.indexOf(k) + 1}` : `connect ${short(tx.signer)}`;
    b.disabled = Boolean(sig) || !k || !prepared.signable || (i > 0 && !landed.has(i - 1));
    b.onclick = () => signPrepared(i, k);
    row.append(tag, what, b);
    box.append(row);
  });
}

/** The Keystone's 64-byte signature over one message: by QR on this page, or through the wallet that holds it. */
async function askKeystone(k, message, chain) {
  if (k.via === 'wallet') {
    const w = wallets.find((x) => x.name === k.wallet);
    if (!w) throw new Error(`${k.wallet} is not in this browser`);
    const { accounts } = await w.features['standard:connect'].connect();
    const account = accounts.find((a) => a.address === k.address);
    if (!account) throw new Error(`make Keystone ${short(k.address)} the active account in ${k.wallet}, then press again`);
    say(`${k.wallet} is asking your Keystone: scan its QR with the Keystone, confirm, then show the Keystone's QR to the camera.`);
    const [out] = await w.features['solana:signTransaction'].signTransaction({ account, transaction: concat([shortvec(1), new Uint8Array(64), message]), chain });
    const signature = out.signedTransaction.subarray(1, 65);
    if (signature.every((b) => b === 0)) throw new Error('the wallet returned it unsigned');
    return signature;
  }
  const requestId = crypto.getRandomValues(new Uint8Array(16));
  showParts(signRequest({ message, path: k.path, xfp: k.xfp, requestId }).parts);
  $('step').textContent = 'On the Keystone: Scan, and point it at this QR. Read what it shows, confirm on the Keystone. Then hold its answer up to the camera.';
  say(`Waiting for Keystone ${short(k.address)}…`);
  const d = new URDecoder();
  return new Promise((resolve, reject) => {
    startCamera((text) => {
      try {
        if (!/^ur:sol-signature\//i.test(text) || !d.receive(text)) return;
        const r = readSignature(d.result);
        if (r.requestId && r.requestId !== toHex(requestId)) throw new Error('that answer is for another request');
        hideStage();
        resolve(r.signature);
      } catch (e) { hideStage(); reject(e); }
    }).catch((e) => { hideStage(); reject(e); });
  });
}

async function signPrepared(i, k) {
  const tx = prepared.txs[i];
  try {
    if (!S.rpc) throw new Error('open the page with #rpc= the cluster to send to');
    const genesis = await rpc('getGenesisHash', []);
    if (genesis !== prepared.genesis) throw new Error(`${S.rpc} is not this set's ${prepared.cluster} (its genesis is ${genesis}): open the page with #rpc= that chain's endpoint`);
    const { value } = await rpc('getLatestBlockhash', [{ commitment: 'confirmed' }]);
    const message = toSign(prepared, tx, k.address, value.blockhash);
    const signature = await askKeystone(k, message, `solana:${prepared.cluster}`);
    const sig = await land(signedWire(message, signature), prepared.cluster);
    landed.set(i, sig);
    renderPrepared();
    say(`${tx.name} landed: ${sig}`);
  } catch (e) { hideStage(); say(String(e.message || e), true); }
}
if (S.prepared) fetch(S.prepared).then((r) => r.text()).then(loadPrepared).catch((e) => say(`The set: ${e.message || e}`, true));

if (S.proposal) {
  $('pull').hidden = false;
  $('amount').textContent = `${(Number(S.amount) / 1e9).toFixed(3)} SOL, this month's allowance`;
  $('from').textContent = S.treasury;
  $('to').textContent = S.vault;
  setInterval(refresh, 5000);
  refresh();
}
renderMine();
renderWallets();
