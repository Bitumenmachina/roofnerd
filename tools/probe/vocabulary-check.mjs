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
import { detachCostPanel, launch, openDemoJob, showTab, showView, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');

/** Words that belong to the code and not to the trade. */
const BANNED = [
  'PLAN_SF', 'VERTICES', 'SEGMENTS', 'STRETCHOUT', 'ORDERUNIT', 'COSTCODE',
  'CONDITIONID', 'PAGEID', 'UNITCOST', 'PRODUCTIONRATE', 'CREWSIZE',
  'trace(s)', 'condition(s)', 'item(s)', 'line(s)',
  'undefined', 'null', 'NaN', '[object Object]',
];

/**
 * Software and AI words. The program speaks roofing and construction (Addendum 6, 2026-09-12):
 * an estimator never meets an agent, a token or a pipeline on this screen. Matched as whole
 * words so "AI" cannot hide inside "drain".
 */
const SOFTWARE = ['agent', 'AI', 'inference', 'token', 'prompt', 'embedding', 'pipeline', 'node', 'entity', 'component', 'render'];

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

/**
 * Everything a person can read, with the formula left out.
 *
 * A formula IS code, typed on purpose by the person reading it, which is why it
 * is the one thing on screen set in the mono face. It used to live only in an
 * input, so the exemption was `input.formula`; it is now also a chip on the
 * compact line and a term in the working written out under an open one — the
 * same string, in the same face, for the same reason. So the exemption is keyed
 * on the class the program puts on the formula wherever it renders it, and on
 * nothing else. Everything that is not the formula is still read.
 */
const VISIBLE = `
  const strip = (root) => {
    const clone = root.cloneNode(true);
    for (const f of clone.querySelectorAll('.formula, input.formula')) f.remove();
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
  await session.execute(() => document.querySelector('.plan-rail .condition')?.click());
  await wait(800);

  // Every surface a person can reach, not one of them: the Plan with a
  // condition picked, the Condition tab with its properties, a line open to its
  // whole chain, the Model, and the panel in a window of its own. The check
  // used to read two screens and there are more than two now.
  const planText = await session.execute(new Function(VISIBLE));

  await showTab(session, 'Condition');
  await wait(700);
  const conditionText = await session.execute(new Function(VISIBLE));

  await showTab(session, 'Estimate');
  await wait(700);
  await session.execute(() => {
    const line = document.querySelector('.cost-line .line-what');
    if (line) line.click();
  });
  await wait(900);
  const openLineText = await session.execute(new Function(VISIBLE));

  await showView(session, 'Model');
  await wait(1600);
  const modelText = await session.execute(new Function(VISIBLE));
  await showView(session, 'Plan');
  await wait(900);

  const before = await session.handles();
  await detachCostPanel(session);
  const after = await session.handles();
  const second = after.find((h) => !before.includes(h));
  let costText = '';
  if (second) {
    await session.switchTo(second);
    await until(session, () => !!document.querySelector('.cost-panel.detached'), { what: 'the cost panel' });
    await wait(800);
    costText = await session.execute(new Function(VISIBLE));
  }

  const screen = [planText, conditionText, openLineText, modelText, costText].join('\n');

  for (const word of BANNED) {
    check(`"${word}" is not on screen`, () => {
      assert.ok(!screen.includes(word), `found "${word}"`);
    });
  }

  for (const word of SOFTWARE) {
    check(`"${word}" is not on screen`, () => {
      const re = new RegExp(`\\b${word}\\b`, word === 'AI' ? '' : 'i');
      assert.ok(!re.test(screen), `found "${word}"`);
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
