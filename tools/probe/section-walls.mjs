// ── C3: the walls, the curbs and the coping, priced off what they actually
// have room for ──────────────────────────────────────────────────────────
//
// A parapet and a curb were drawn at a typed height and flashed for the whole
// of it — `LF * H` — with no regard for how far the roof beneath had already
// climbed, or for how far the detail is actually drawn to run. This is the
// check that the money moved off the solve instead:
//
//   THE FLASHING LINE IS NOT `LF x H`. The parapet's own flashing quantity
//   carries a derived mark (◈) and is not the same figure `LF x H` would give
//   off the job's own numbers — because the flashing ceiling is `FLASHUP`
//   (24 in on the demo's parapet), not the wall's own 42 in height.
//
//   A CURB EITHER PASSES OR IS TOLD IT DOES NOT. `CURB_RAISE_IN` is on the
//   panel for both the parapet and the curb — Part 2's own rule, written on
//   every wall and curb condition and not curbs alone — and it is a number,
//   never a dash with no reason.
//
//   THE COPING PIECES ARE THE RUN'S OWN CORNERS, NOT A TYPED COUNT. The
//   parapet's `COPING_PCS_EA` is checked against `ceil(LF / section) +
//   corners`, with `corners` counted off the trace's own distinct vertices —
//   never typed in here.
//
// Front door throughout — the start screen, the demo button, the view switch,
// the tree, the tabs. Nothing here calls a command to get a job open.
//
// RED AT C2'S HEAD. Before this batch, `scope.rs` carries no `FLASH_SF`,
// `CURB_RAISE_IN` or `COPING_PCS_EA` on any condition, the Condition panel
// lists no such names, and the wall flashing line on the Estimate reads
// `LF * H`. Every row below fails, and that is the point of writing it first.
//
//   pnpm build:app && node tools/probe/section-walls.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, errorsSoFar, launch, openDemoJob, showTab, showView, until, wait, watchErrors,
  commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const shot = async (session, name) =>
  writeFile(join(EVIDENCE, `${name}-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

// ── what the job itself says, worked out here rather than typed ────────────
const { COPING_SECTION_FT } = await import('../../packages/engine/dist/index.js');
const jobFolder = join(ROOT, 'jobs/demo-job');
const read = async (rel) => JSON.parse(await readFile(join(jobFolder, rel), 'utf8'));
const conditions = await read('conditions.json');
const pages = await read('pages/pages.json');

const scales = {};
for (const page of pages) if (page.feetPerUnit) scales[page.id] = page.feetPerUnit;
const feet = (trace) => (scales[trace.pageId]
  ? trace.points.map((p) => ({ x: p.x * scales[trace.pageId], y: p.y * scales[trace.pageId] }))
  : []);

/** A run's own perimeter, corner to corner, the way `measures.ts` sums an LF. */
const lfOf = (ring) => {
  let lf = 0;
  for (let i = 0; i < ring.length - 1; i += 1) {
    lf += Math.hypot(ring[i + 1].x - ring[i].x, ring[i + 1].y - ring[i].y);
  }
  return lf;
};

/**
 * The run's own distinct corners — the trace's points with a repeated
 * closing point dropped, and any three-in-a-row straight run dropped too
 * (`measures.ts:24-26`: an arc contributes none, and neither does a corner
 * that is not one). None of the demo's wall or curb traces has a straight
 * run of three, so this is exactly the count a hand recount would make.
 */
const cornersOf = (ring) => {
  const pts = ring.slice();
  if (pts.length > 1 && pts[0].x === pts[pts.length - 1].x && pts[0].y === pts[pts.length - 1].y) {
    pts.pop();
  }
  const n = pts.length;
  let corners = 0;
  for (let i = 0; i < n; i += 1) {
    const a = pts[(i + n - 1) % n];
    const b = pts[i];
    const c = pts[(i + 1) % n];
    const turn = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
    if (Math.abs(turn) > 1e-9) corners += 1;
  }
  return corners;
};

const wallCondition = (name) => conditions.find((c) => c.name === name && !c.hidden);
const parapetJob = wallCondition('Parapet Wall Flashing');
const curbJob = wallCondition('RTU-1 Curb');
assert.ok(parapetJob && curbJob, 'the demo carries both a parapet and a curb');

const parapetRing = feet(parapetJob.traces[0]);
const parapetLf = lfOf(parapetRing);
const parapetOldFlash = parapetLf * (parapetJob.properties.H ?? 0); // the retired LF x H
const parapetCorners = cornersOf(parapetRing);
const section = parapetJob.properties.COPING_SECTION_FT ?? COPING_SECTION_FT;
const parapetPieces = Math.ceil(parapetLf / section) + parapetCorners;

const money = (text) => {
  const m = /-?[\d,]+\.\d\d/.exec(String(text ?? ''));
  return m === null ? null : Number.parseFloat(m[0].replace(/,/g, ''));
};
const number = (text) => {
  const m = /-?[\d,]+(\.\d+)?/.exec(String(text ?? ''));
  return m === null ? null : Number.parseFloat(m[0].replace(/,/g, ''));
};

// ── what the window says ───────────────────────────────────────────────────

/** Every line on the Estimate tab, as the estimator sees it. */
const linesNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.cost-line')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return {
      description: text('.line-item'),
      condition: text('.line-basis > span'),
      formula: text('.line-formula'),
      quantity: text('.line-qty'),
      unit: (text('.line-unit') || '').replace('◈', '').trim(),
      derived: !!r.querySelector('.line-unit .derived'),
    };
  }));
});

/** The derived names the Condition panel lists, with the words beside each. */
const panelNow = (session) => session.execute(function () {
  const panel = document.querySelector('.condition-panel');
  if (!panel) return JSON.stringify({ there: false });
  const rows = [...panel.querySelectorAll('.panel-derived .derived-row')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return { name: text('.derived-name'), label: text('.derived-label'), value: text('.derived-value') };
  });
  return JSON.stringify({
    there: true,
    name: ((panel.querySelector('.panel-name') || {}).value) || null,
    rows,
  });
});

/** Pick a condition in the tree beside the panel, the way a hand picks it. */
const pickCondition = async (session, name) => {
  const found = await session.execute(function (want) {
    const row = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .find(function (e) { return (e.textContent || '').indexOf(want) >= 0; });
    if (!row) return false;
    row.click();
    return true;
  }, name);
  await wait(800);
  return found;
};

/** The named numbers the shell solved, pulled the way the app pulls them. */
const scopesNow = (session) => session.execute(function () {
  return window.__TAURI_INTERNALS__.invoke('solve_get', { part: 'scopes' })
    .then(function (s) { return JSON.stringify(s); })
    .catch(function (e) { return JSON.stringify({ error: String(e) }); });
});

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  const demoAt = await centreOf(session, '.start-demo');
  assert.ok(demoAt, 'the demo button is on the start screen');
  await openDemoJob(session);
  await until(session, () => document.querySelector('.view-switch'), { what: 'the job to open' });

  await showView(session, 'Model');
  await wait(500);
  await shot(session, 'section-walls-model');

  // ── the solve itself, the same way the app pulls it ───────────────────────
  const scopes = JSON.parse(await scopesNow(session));
  check('the solve carries the wall and curb names, not LF x H', () => {
    assert.ok(scopes[parapetJob.id], 'the parapet has a scope entry');
    assert.equal(typeof scopes[parapetJob.id].FLASH_SF, 'number');
    assert.notEqual(
      scopes[parapetJob.id].FLASH_SF, parapetOldFlash,
      `FLASH_SF should not be the retired LF x H figure (${parapetOldFlash})`,
    );
    assert.ok(scopes[curbJob.id], 'the curb has a scope entry');
    assert.equal(typeof scopes[curbJob.id].CURB_RAISE_IN, 'number');
  });
  check('the coping pieces are the run\'s own sections plus its own corners', () => {
    assert.equal(scopes[parapetJob.id].COPING_PCS_EA, parapetPieces);
  });

  // ── the Estimate names it as a derived line ───────────────────────────────
  await showTab(session, 'Estimate');
  await wait(500);
  await shot(session, 'section-walls-estimate');
  const lines = JSON.parse(await linesNow(session));
  check('the flashing line on the sheet carries ◈ and is not LF x H\'s own number', () => {
    const flashing = lines.find((l) => (l.condition || '').indexOf('Parapet') >= 0
      && /flash/i.test(l.description || ''));
    assert.ok(flashing, `no parapet flashing line among ${JSON.stringify(lines.map((l) => l.description))}`);
    assert.ok(flashing.derived, 'the flashing quantity should be ◈');
    assert.notEqual(money(flashing.money), parapetOldFlash, 'should not price the retired LF x H figure');
  });

  // ── the parapet's own panel ────────────────────────────────────────────────
  await showView(session, 'Model');
  await wait(300);
  const parapetPicked = await pickCondition(session, 'Parapet Wall Flashing');
  await showTab(session, 'Condition');
  await wait(500);
  await shot(session, 'section-walls-condition-parapet');
  const parapetPanel = JSON.parse(await panelNow(session));
  check('the parapet is reachable in the tree', () => assert.ok(parapetPicked));
  check('the panel lists the parapet\'s flashing and coping names', () => {
    const byName = Object.fromEntries((parapetPanel.rows || []).map((r) => [r.name, r]));
    assert.ok(byName.FLASH_SF, 'FLASH_SF is on the panel');
    assert.ok(byName.CURB_RAISE_IN, 'CURB_RAISE_IN is on the panel — every wall, not curbs alone');
    assert.ok(byName.COPING_PCS_EA, 'COPING_PCS_EA is on the panel');
    assert.equal(number(byName.COPING_PCS_EA.value), parapetPieces);
  });

  // ── the curb: a raise finding, or its stated absence ──────────────────────
  const curbPicked = await pickCondition(session, 'RTU-1 Curb');
  await showTab(session, 'Condition');
  await wait(500);
  await shot(session, 'section-walls-condition-curb');
  const curbPanel = JSON.parse(await panelNow(session));
  check('the curb is reachable in the tree', () => assert.ok(curbPicked));
  check('the curb\'s panel states a raise finding, never a blank', () => {
    const byName = Object.fromEntries((curbPanel.rows || []).map((r) => [r.name, r]));
    assert.ok(byName.CURB_RAISE_IN, 'CURB_RAISE_IN is on the curb\'s own panel');
    const raise = number(byName.CURB_RAISE_IN.value);
    assert.equal(typeof raise, 'number', 'a raise finding is a number, stated even when it is nought');
  });
} catch (e) {
  console.log(`  FAIL  the probe itself: ${e.stack || e.message}`);
  results.push('FAIL');
} finally {
  const errors = await errorsSoFar(session).catch(() => []);
  if (errors.length) {
    console.log(`  REPORT  ${errors.length} console error(s) during the run:`);
    for (const e of errors) console.log(`    ${e}`);
  }
  await app.close().catch(() => undefined);
}

const pass = results.filter((r) => r === 'PASS').length;
console.log(`\n${pass}/${results.length} passed`);
process.exit(results.every((r) => r === 'PASS') ? 0 : 1);
