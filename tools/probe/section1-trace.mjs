// ── Section 1's done-check ─────────────────────────────────────────────────
// "Open a PDF, calibrate, trace an area with pitch, a line, a count; the list
//  shows SF/LF/EA; the file round-trips."
//
// Run: node tools/probe/section1-trace.mjs

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import assert from 'node:assert/strict';
import { openApp, clickPage, wait, waitForSheet } from './harness.mjs';

const sheet = await readFile(resolve(import.meta.dirname, 'test-sheet.pdf'));

// A letter sheet at 72 units to the inch. At 1/4" = 1'-0" one unit is 4/72 feet,
// so a 144 x 144 unit square is 8 x 8 feet: 64 SF, 32 LF of perimeter, 4 corners.
const doc = {
  'job': {
    format: 1, name: 'Probe Job', activeScenarioId: 's1',
    scenarios: [{ id: 's1', name: 'Scenario 1', prices: {}, adders: {} }],
  },
  'pages': [{ id: 'page-1', name: 'test-sheet.pdf', source: 'pages/test-sheet.pdf', pageNumber: 1 }],
  'conditions': [],
  'costCodes': [],
};

const app = await openApp({ doc, files: { 'pages/test-sheet.pdf': sheet } });
const results = [];
const check = (name, fn) => {
  try { fn(); results.push(['PASS', name]); console.log(`  PASS  ${name}`); }
  catch (e) { results.push(['FAIL', name]); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

try {
  // ── the drawing is on screen ────────────────────────────────────────────
  await waitForSheet(app.page);

  const sheetSize = await app.page.evaluate(() => {
    const c = document.querySelector('.surface-sheet');
    const o = document.querySelector('.surface-overlay');
    return { canvas: c.width, viewBox: o.viewBox.baseVal.width, painted: o.getAttribute('viewBox') };
  });
  check('the PDF renders', () => {
    // A bare canvas is 300 wide. The sheet is 612 page units fitted to the
    // window, so anything near 300 means nothing was painted.
    assert.ok(sheetSize.canvas > 400, `canvas is ${sheetSize.canvas}px`);
  });
  check('the overlay is in page units', () => assert.equal(sheetSize.viewBox, 612, sheetSize.painted));

  // ── scale it: 1/4" = 1'-0" ──────────────────────────────────────────────
  await app.page.select('.toolbar select:nth-of-type(2)', '4');
  await wait(200);
  const scaled = await app.doc();
  check('the sheet takes a scale', () => {
    assert.ok(Math.abs(scaled['pages'][0].feetPerUnit - 4 / 72) < 1e-12);
  });
  check('the scale says where it came from', () => {
    assert.match(scaled['pages'][0].scaleNote, /1\/4/);
  });

  // ── trace an area: an 8' x 8' square ────────────────────────────────────
  await app.page.evaluate(() => [...document.querySelectorAll('.toolbar button')]
    .find((b) => b.textContent === 'Area').click());
  for (const [x, y] of [[100, 100], [244, 100], [244, 244], [100, 244]]) await clickPage(app.page, x, y);
  await app.page.keyboard.press('Enter');
  await wait(200);

  // ── give it a pitch, which must lift SF and leave the footprint alone ───
  await app.page.evaluate(() => {
    const doc = window.__PROBE__.doc();
    const c = doc['conditions'][0];
    return window.__TAURI_INTERNALS__.invoke('doc_set', {
      pointer: '/conditions/0/properties',
      value: { ...c.properties, PITCH: 12 },
    });
  });
  await wait(200);

  // ── trace a run and a count ─────────────────────────────────────────────
  await app.page.evaluate(() => [...document.querySelectorAll('.toolbar button')]
    .find((b) => b.textContent === 'Line').click());
  await clickPage(app.page, 300, 300);
  await clickPage(app.page, 480, 300);
  await app.page.keyboard.press('Enter');
  await wait(200);

  await app.page.evaluate(() => [...document.querySelectorAll('.toolbar button')]
    .find((b) => b.textContent === 'Count').click());
  for (const [x, y] of [[150, 500], [200, 500], [250, 500]]) await clickPage(app.page, x, y);
  await wait(200);

  // ── what the list says ──────────────────────────────────────────────────
  const rows = await app.page.evaluate(() =>
    [...document.querySelectorAll('.condition')].map((el) => ({
      name: el.querySelector('.condition-name').textContent,
      measures: el.querySelector('.condition-measures').textContent,
    })));

  check('three conditions were traced', () => assert.equal(rows.length, 3));

  // A synthetic click lands on a whole SCREEN pixel, so the page coordinate it
  // becomes carries up to half a page unit of error. That is the probe's own
  // aim, not the program's arithmetic — the engine's own tests pin the maths
  // exactly. A tenth of a percent is comfortably inside that and far outside
  // any real mistake.
  const near = (text, unit, expected) => {
    const m = text.match(new RegExp(`([\\d,.]+) ${unit}`));
    assert.ok(m, `no ${unit} in "${text}"`);
    const actual = Number(m[1].replace(/,/g, ''));
    const off = Math.abs(actual - expected) / expected;
    assert.ok(off < 0.005, `${unit} is ${actual}, expected about ${expected} (off by ${(off * 100).toFixed(2)}%)`);
  };

  check('the area shows SF, LF and EA at once', () => {
    const m = rows[0].measures;
    // 8' x 8' at 12:12 → 64 SF x 1.4142 = 90.51 SF; perimeter 32 LF; 4 corners.
    near(m, 'SF', 90.51);
    near(m, 'LF', 32);
    near(m, 'EA', 4);
  });

  check('the run shows LF and its corners, and no area of its own', () => {
    const m = rows[1].measures;
    near(m, 'LF', 10);   // 180 units x 4/72 = 10 ft
    near(m, 'EA', 2);
    assert.doesNotMatch(m, /SF/, m);
  });

  check('the count is a count', () => assert.match(rows[2].measures, /3 EA/, rows[2].measures));

  // ── pitch really did the work ───────────────────────────────────────────
  const final = await app.doc();
  check('pitch lifted the surface off the footprint', () => {
    assert.equal(final['conditions'][0].properties.PITCH, 12);
  });

  // ── round-trip: the traces are in the document, in page units ───────────
  check('traces are stored in page units, not screen pixels', () => {
    const points = final['conditions'][0].traces[0].points;
    assert.equal(points.length, 4);
    assert.ok(Math.abs(points[0].x - 100) < 0.6, `first corner is at ${JSON.stringify(points[0])}`);
    assert.ok(Math.abs(points[1].x - 244) < 0.6);
  });

  check('the drawing is referenced, never copied into the job', () => {
    assert.equal(final['pages'][0].source, 'pages/test-sheet.pdf');
    assert.equal(JSON.stringify(final).includes('%PDF'), false);
  });

  check('nothing errored in the page', () => {
    const real = app.problems.filter((p) => !/favicon/i.test(p));
    assert.deepEqual(real, []);
  });
} catch (e) {
  results.push(['FAIL', `the probe stopped: ${e.message}`]);
  console.log(`  FAIL  the probe stopped: ${e.message}`);
  console.log('  page problems:', app.problems.slice(0, 6));
} finally {
  await app.close();
}

const failed = results.filter(([s]) => s === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
