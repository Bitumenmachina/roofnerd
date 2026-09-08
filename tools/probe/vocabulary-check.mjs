// ── The vocabulary check ───────────────────────────────────────────────────
// Nothing on screen should need a software word to name it.
//
// This reads what is actually rendered in the shipped window — not the source,
// because the source legitimately uses `PLAN_SF` as a key and `costCode` as a
// field, and grepping it would either miss the defect or fire on every file.
// What matters is what an estimator can see.
//
// Formula fields are exempt, and only formula fields: a formula IS code, typed
// on purpose by the person reading it, which is why it is the one thing on the
// sheet set in monospace.
//
//   node tools/probe/vocabulary-check.mjs

import { execFileSync } from 'node:child_process';
import { cp, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, openDemoJob, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');

/** Words that belong to the code and not to the trade. */
const BANNED = [
  'PLAN_SF', 'VERTICES', 'SEGMENTS', 'STRETCHOUT', 'ORDERUNIT', 'COSTCODE',
  'CONDITIONID', 'PAGEID', 'UNITCOST', 'PRODUCTIONRATE', 'CREWSIZE',
  'trace(s)', 'condition(s)', 'item(s)', 'line(s)',
  'undefined', 'null', 'NaN', '[object Object]',
];

/** Bare property keys, which are fine in a formula and nowhere else. */
const BARE_KEYS = ['SIDES', 'PITCH', 'STRETCHOUT', 'TAPER', 'ELEV', 'SUMP', 'BOARDS'];

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

async function job() {
  const dir = await mkdtemp(join(tmpdir(), 'roofnerd-vocab-'));
  await cp(join(ROOT, 'jobs/demo-job'), dir, { recursive: true });
  await cp(join(ROOT, 'tools/probe/test-sheet.pdf'), join(dir, 'pages/roof-plan.pdf'));
  const write = (n, v) => writeFile(join(dir, n), `${JSON.stringify(v, null, 2)}\n`);
  await write('pages/pages.json', [{
    id: 'page-1', name: 'Roof Plan', source: 'pages/roof-plan.pdf', pageNumber: 1, feetPerUnit: 4 / 72,
  }]);
  await write('conditions.json', [{
    id: 'c1', name: 'Main Roof Field', kind: 'area', color: '#2f6f4f',
    properties: { PITCH: 4, SIDES: 4, STRETCHOUT: 14 },
    traces: [{ id: 't1', pageId: 'page-1', points: [{ x: 110, y: 150 }, { x: 430, y: 150 }, { x: 430, y: 400 }, { x: 110, y: 400 }] }],
    items: [{ id: 'i1', description: 'Membrane', costCode: '07-100-100', unit: 'SQ', formula: 'SQ', unitCost: 1.75 }],
  }]);
  return { dir, remove: () => rm(dir, { recursive: true, force: true }) };
}

/** Everything a person can read, with formula fields left out. */
const VISIBLE = `
  const strip = (root) => {
    const clone = root.cloneNode(true);
    for (const f of clone.querySelectorAll('input.formula')) f.remove();
    return clone.innerText || clone.textContent || '';
  };
  const values = [...document.querySelectorAll('input:not(.formula), select')]
    .map((el) => (el.selectedOptions ? [...el.selectedOptions].map((o) => o.text).join(' ') : el.value))
    .join('  ');
  const titles = [...document.querySelectorAll('[title], [aria-label], [placeholder]')]
    .map((el) => [el.getAttribute('title'), el.getAttribute('aria-label'), el.getAttribute('placeholder')].join(' '))
    .join('  ');
  return [strip(document.body), values, titles].join('  ');
`;

const scratch = await job();
const app = await launch();
const { session } = app;

try {
  await openDemoJob(session);
  await wait(1200);
  await session.execute(() => document.querySelector('.condition')?.click());
  await wait(800);

  const planText = await session.execute(new Function(VISIBLE));

  const before = await session.handles();
  await session.execute(() => window.__TAURI_INTERNALS__.invoke('open_editor', { editor: 'estimate' }));
  await wait(2000);
  const after = await session.handles();
  const second = after.find((h) => !before.includes(h));
  let sheetText = '';
  if (second) {
    await session.switchTo(second);
    await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the sheet' });
    await wait(800);
    sheetText = await session.execute(new Function(VISIBLE));
  }

  const screen = `${planText}\n${sheetText}`;

  for (const word of BANNED) {
    check(`"${word}" is not on screen`, () => {
      assert.ok(!screen.includes(word), `found "${word}"`);
    });
  }

  for (const key of BARE_KEYS) {
    check(`"${key}" appears only in a formula, if at all`, () => {
      assert.ok(!screen.includes(key), `found "${key}" outside a formula`);
    });
  }

  check('the trade words are there instead', () => {
    // What the specification requires to be readable: the four headline
    // measures by their trade names, the properties by theirs, and a property
    // written as a person would say it.
    for (const word of ['Area', 'Run', 'Count', 'Squares', 'Pitch', 'Girth', 'Height', 'sides']) {
      assert.ok(screen.includes(word), `expected to see "${word}"`);
    }
  });
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
  await scratch.remove();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
