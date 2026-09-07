// ── Section 2's done-check ─────────────────────────────────────────────────
// "Trace a parapet as a line with H; add wall flashing LF * H, coping LF,
//  corners EA; the total moves in the torn-off window; a bad formula shows its
//  error on the line."
//
// Run: node tools/probe/section2-sheet.mjs

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import assert from 'node:assert/strict';
import { openApp, clickPage, clickTool, wait, waitForSheet } from './harness.mjs';

const sheet = await readFile(resolve(import.meta.dirname, 'test-sheet.pdf'));

const doc = {
  job: {
    format: 1, name: 'Parapet Job', activeScenarioId: 's1',
    scenarios: [{ id: 's1', name: 'Scenario 1', prices: {}, adders: {} }],
  },
  // Already scaled at 1/4" = 1'-0": one page unit is 4/72 of a foot.
  pages: [{ id: 'page-1', name: 'test-sheet.pdf', source: 'pages/test-sheet.pdf', pageNumber: 1, feetPerUnit: 4 / 72 }],
  conditions: [],
  costCodes: [],
};

const app = await openApp({ doc, files: { 'pages/test-sheet.pdf': sheet } });
const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

/**
 * Compare a number read off the screen against what it should be.
 *
 * A synthetic click lands on a whole SCREEN pixel, so the page coordinate it
 * becomes carries sub-unit error and every length downstream inherits it. That
 * is the probe's aim, not the program's arithmetic — the engine's own tests pin
 * the maths exactly. Half a percent is far inside that and far outside a real
 * mistake.
 */
function nearly(text, expected, what) {
  const actual = Number(String(text).replace(/[$,]/g, '').trim());
  assert.ok(Number.isFinite(actual), `${what} read as "${text}"`);
  const off = Math.abs(actual - expected) / expected;
  assert.ok(off < 0.005, `${what} is ${actual}, expected about ${expected} (off by ${(off * 100).toFixed(2)}%)`);
}

let estimate;
try {
  await waitForSheet(app.page);

  // ── trace a parapet: three runs of 180, 180 and 90 units ────────────────
  // 450 units x 4/72 = 25 feet of parapet, with 4 corners.
  await clickTool(app.page, 'Line');
  for (const [x, y] of [[100, 100], [280, 100], [280, 280], [370, 280]]) await clickPage(app.page, x, y);
  await app.page.keyboard.press('Enter');
  await wait(250);

  check('the parapet was traced', () => {
    assert.equal(app.doc().conditions.length, 1);
    assert.equal(app.doc().conditions[0].kind, 'line');
  });

  // ── give it a height in the condition panel ─────────────────────────────
  await app.page.evaluate(() => {
    const row = document.querySelector('.condition');
    row.click();
  });
  await wait(200);

  const named = await app.page.evaluate(() =>
    [...document.querySelectorAll('.condition-panel label span')].map((s) => s.textContent));
  check('the panel offers the properties a roofing detail asks for', () => {
    for (const want of ['Height', 'Width', 'Thickness', 'Pitch', 'Sides', 'Stretch-out']) {
      assert.ok(named.includes(want), `no "${want}" — panel has ${named.join(', ')}`);
    }
  });

  await app.page.evaluate(() => {
    const labels = [...document.querySelectorAll('.condition-panel label')];
    const height = labels.find((l) => l.querySelector('span')?.textContent === 'Height');
    const input = height.querySelector('input');
    input.value = '1.5';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await wait(250);

  check('the height went onto the condition', () => {
    assert.equal(app.doc().conditions[0].properties.H, 1.5);
  });

  // ── the torn-off Estimate Sheet, on the other monitor ───────────────────
  estimate = await app.tearOff('estimate');
  await wait(400);

  // ── three items: wall flashing, coping, corners ─────────────────────────
  const addItem = async (description, formula, unit, unitCost) => {
    await estimate.evaluate(() => document.querySelector('.add-item').click());
    await wait(200);
    await estimate.evaluate((d, f, u, c) => {
      const rows = document.querySelectorAll('.sheet tbody tr');
      const row = rows[rows.length - 1];
      const [desc, , form] = row.querySelectorAll('input[type=text]');
      const set = (el, v) => { el.value = v; el.dispatchEvent(new Event('input', { bubbles: true })); };
      set(desc, d);
      set(form, f);
      const unitSelect = row.querySelector('select');
      unitSelect.value = u;
      unitSelect.dispatchEvent(new Event('change', { bubbles: true }));
      const numbers = row.querySelectorAll('input[type=number]');
      set(numbers[numbers.length - 1], String(c));
    }, description, formula, unit, unitCost);
    await wait(250);
  };

  await addItem('Wall flashing', 'LF * H', 'SF', 9);
  await addItem('Coping', 'LF', 'LF', 22);
  await addItem('Corners', 'EA', 'EA', 45);

  const lines = await estimate.evaluate(() =>
    [...document.querySelectorAll('.sheet tbody tr')].map((tr) => {
      const cells = [...tr.querySelectorAll('td')];
      return {
        description: cells[0].querySelector('input').value,
        formula: cells[2].querySelector('input').value,
        quantity: cells[3].textContent,
        unit: cells[4].querySelector('select').value,
        extended: cells[8].textContent,
      };
    }));

  check('the formula is on the line, and it is what was typed', () => {
    assert.deepEqual(lines.map((l) => l.formula), ['LF * H', 'LF', 'EA']);
  });

  check('one trace fed three different units', () => {
    assert.deepEqual(lines.map((l) => l.unit), ['SF', 'LF', 'EA']);
  });

  check('LF * H turned a run into an area', () => {
    nearly(lines[0].quantity, 37.5, 'wall flashing SF');   // 25 LF x 1.5 ft
  });

  check('the coping measures the same run', () => {
    nearly(lines[1].quantity, 25, 'coping LF');
  });

  check('the corners are counted', () => {
    assert.equal(lines[2].quantity, '4');
  });

  check('each line extends at its own price', () => {
    nearly(lines[0].extended, 337.5, 'wall flashing');   // 37.5 x 9
    nearly(lines[1].extended, 550, 'coping');            // 25 x 22
    nearly(lines[2].extended, 180, 'corners');           // 4 x 45
  });

  const total = await estimate.evaluate(() => document.querySelector('.sheet-total strong').textContent);
  check('the total is the sum of the lines', () => nearly(total, 1067.5, 'total'));

  // ── the live loop: trace more of the same parapet ───────────────────────
  // A condition is selected, so another run goes onto it — which is how an
  // estimator traces three sides of one parapet as one condition rather than
  // three. Ten more feet, and two more corners.
  await clickTool(app.page, 'Line');
  await clickPage(app.page, 400, 400);
  await clickPage(app.page, 580, 400);
  await app.page.keyboard.press('Enter');
  await wait(400);

  const afterTrace = await estimate.evaluate(() => ({
    total: document.querySelector('.sheet-total strong').textContent,
    coping: document.querySelectorAll('.sheet tbody tr')[1].querySelectorAll('td')[3].textContent,
    rows: document.querySelectorAll('.sheet tbody tr').length,
  }));

  check('another run joins the condition that is selected', () => {
    assert.equal(afterTrace.rows, 3, 'the lines multiplied instead of the quantity growing');
    nearly(afterTrace.coping, 35, 'coping LF');   // 25 + 10
  });

  check('tracing in one window moves the money in the other', () => {
    // 52.5 SF x 9 + 35 LF x 22 + 6 EA x 45 = 472.50 + 770 + 270
    nearly(afterTrace.total, 1512.5, 'total after tracing');
  });

  // Now change the parapet's height in the PLAN window and watch the ESTIMATE
  // window follow. This is the whole reason the program exists.
  await app.page.evaluate(() => {
    const labels = [...document.querySelectorAll('.condition-panel label')];
    const height = labels.find((l) => l.querySelector('span')?.textContent === 'Height');
    const input = height.querySelector('input');
    input.value = '3';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await wait(500);

  const moved = await estimate.evaluate(() => ({
    quantity: document.querySelectorAll('.sheet tbody tr')[0].querySelectorAll('td')[3].textContent,
    total: document.querySelector('.sheet-total strong').textContent,
  }));

  check('a property typed in one window moves the money in the other', () => {
    // 35 LF x 3 ft = 105 SF at $9 = $945, plus 770 + 270 = $1,985.
    nearly(moved.quantity, 105, 'wall flashing SF');
    nearly(moved.total, 1985, 'total after the height changed');
  });

  // ── a bad formula says what is wrong, on the line ───────────────────────
  await estimate.evaluate(() => {
    const form = document.querySelectorAll('.sheet tbody tr')[0].querySelectorAll('input[type=text]')[2];
    form.value = 'LF * NOPE';
    form.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await wait(300);

  const bad = await estimate.evaluate(() => {
    const row = document.querySelectorAll('.sheet tbody tr')[0];
    return {
      error: row.querySelector('.formula-error')?.textContent ?? null,
      marked: row.querySelector('input.formula')?.classList.contains('bad'),
      quantity: row.querySelectorAll('td')[3].textContent,
      extended: row.querySelectorAll('td')[8].textContent,
    };
  });

  check('a bad formula shows its error on the line', () => {
    assert.match(bad.error ?? '', /nothing here is called "NOPE"/, String(bad.error));
    assert.equal(bad.marked, true, 'the field is not marked');
  });

  check('a bad formula shows no quantity and no money', () => {
    assert.match(bad.quantity, /pending/, bad.quantity);
    assert.doesNotMatch(bad.extended, /\$/, bad.extended);
  });

  check('nothing errored in either window', () => {
    assert.deepEqual(app.problems, []);
  });
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the probe stopped: ${e.message}`);
  console.log('  page problems:', app.problems.slice(0, 6));
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
