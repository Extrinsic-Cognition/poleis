// The result as a person reads it (a fourteen-row table, then each row's reasons) and as a program reads it (JSON).
import { PASS, FAIL, when, tokens } from './rows.mjs';

const VERDICT_WIDTH = 'not checkable'.length;

function wrap(text, width, indent) {
  const out = [];
  let line = '';
  for (const word of text.split(' ')) {
    if (line && line.length + 1 + word.length > width) { out.push(line); line = word; } else line = line ? `${line} ${word}` : word;
  }
  if (line) out.push(line);
  return out.map(l => indent + l);
}

export function renderTable(result, { width = 118 } = {}) {
  const lines = [];
  const program = result.tokenProgram === 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' ? 'SPL Token' : 'Token-2022';
  lines.push(`warm-day0  ${result.mint}`);
  lines.push(`${result.cluster}, slot ${result.slot} (${when(result.time)}), supply ${tokens(result.supply, result.decimals)}, ${result.decimals} decimals, ${program}`);
  lines.push('');
  const claimWidth = Math.max(...result.rows.map(r => r.claim.length));
  lines.push(` #  ${'verdict'.padEnd(VERDICT_WIDTH)}  ${'claim'.padEnd(claimWidth)}  read from`);
  for (const r of result.rows) {
    const from = r.source.length ? `${r.source[0]}${r.source.length > 1 ? ` (+${r.source.length - 1})` : ''}` : '-';
    lines.push(`${String(r.n).padStart(2)}  ${r.verdict.padEnd(VERDICT_WIDTH)}  ${r.claim.padEnd(claimWidth)}  ${from}`);
  }
  lines.push('');
  lines.push(`${result.summary.pass} pass, ${result.summary.fail} fail, ${result.summary.notCheckable} not checkable.`);
  for (const r of result.rows) {
    lines.push('');
    lines.push(`${String(r.n).padStart(2)}  ${r.verdict.toUpperCase()}  ${r.claim}`);
    if (r.source.length > 1) lines.push(...wrap(`read from: ${r.source.join(', ')}`, width - 4, '    '));
    for (const d of r.detail) lines.push(...wrap(d, width - 4, '    '));
    for (const n of r.notes) lines.push(...wrap(`note: ${n}`, width - 4, '    '));
  }
  if (result.programs.length) {
    lines.push('');
    lines.push('Programs these rows stand on (their layouts were read by hand; idl/sources.json):');
    for (const p of result.programs) {
      const state = p.deployedSlot === null ? (p.error ? `not read (${p.error})` : 'deploy state not read')
        : `${p.moved === true ? `REDEPLOYED at slot ${p.deployedSlot} since the layout was read (slot ${p.deployedSlotAtReading})` : p.moved === false ? `deployed at slot ${p.deployedSlot}, as when its layout was read on ${p.layoutRead}` : `deployed at slot ${p.deployedSlot}`}; ${p.upgradeAuthority ? `upgradeable by ${p.upgradeAuthority}` : 'no upgrade authority'}`;
      lines.push(...wrap(`${p.name} ${p.address}: ${state}.`, width - 4, '    '));
    }
  }
  lines.push('');
  lines.push(`${result.cost.rpcCalls} read-only RPC calls in ${(result.cost.ms / 1000).toFixed(1)} s. Exit ${result.exit}: ${result.exitWhy}.`);
  return lines.join('\n');
}

export function renderJson(result) {
  return JSON.stringify(result, (key, value) => (typeof value === 'bigint' ? value.toString() : value), 2);
}

/** 0 when no decidable row failed and every required row passed; 1 otherwise. */
export function exitOf(result, required = []) {
  const failed = result.rows.filter(r => r.verdict === FAIL).map(r => r.n);
  const unmet = required.filter(n => result.rows[n - 1].verdict !== PASS && !failed.includes(n));
  if (failed.length) return { code: 1, why: `row${failed.length === 1 ? '' : 's'} ${failed.join(', ')} failed${unmet.length ? `; required and not passed: ${unmet.join(', ')}` : ''}` };
  if (unmet.length) return { code: 1, why: `required and not passed: row${unmet.length === 1 ? '' : 's'} ${unmet.join(', ')}` };
  return { code: 0, why: required.length ? `no row failed and rows ${required.join(', ')} passed` : 'no decidable row failed' };
}
