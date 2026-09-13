// ── Section 2's done-check ─────────────────────────────────────────────────
// The condition editor and the estimate sheet: a run carries the properties a
// roofing detail asks for, one trace feeds three units through three formulas,
// each line extends at its own price, and a formula that cannot be worked out
// says so without pretending to be a number.
//
// This ran against a browser with the shell stubbed and took no screenshot, so
// section 2 was certified on two images no script had produced and nobody had
// opened. It now runs on the shipped runtime, through the front door, and
// leaves the pictures it was supposed to leave.
//
// The two-window claims — tracing in one window moving the money in the other,
// a property typed in one moving it in the other — are not repeated here.
// `tools/probe/runtime-check.mjs` already makes them against two real windows,
// and a second copy of a check is not a second check.
//
//   pnpm build:app && node tools/probe/section2-sheet.mjs

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  errorsSoFar, launch, openDemoJob, showTab, until, wait, watchErrors, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};
const nearly = (got, want, what, tol = 0.02) =>
  assert.ok(Math.abs(Number(String(got).replace(/[^0-9.-]/g, '')) - want) <= tol,
    `${what}: ${got} against ${want}`);

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);
  await openDemoJob(session);
  await wait(1800);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 40000 });
  await wait(700);

  // ── the condition panel ─────────────────────────────────────────────────
  // It is the cost panel's Condition tab now, beside the drawing rather than
  // in an editor of its own. Same panel, same fields, same claim.
  await showTab(session, 'Condition');
  await session.execute(function () {
    const row = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .find(function (e) { return /Parapet Wall Flashing/.test(e.textContent || ''); });
    if (row) row.click();
  });
  await wait(800);

  // What the job says the parapet is, so the checks below compare the window
  // against the document rather than against a number typed into this file. A
  // check carrying its own copy of a fixture value breaks when the fixture
  // legitimately changes, and — worse — keeps passing when the window is wrong
  // and the fixture moved to match it.
  const stated = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-parapet'; });
      return JSON.stringify({
        H: c.properties.H, WALL: c.properties.WALL,
        // A closed run's corners are the straight pieces between them, which is
        // one fewer than the points because the last point closes back onto the
        // first. The mitre line is bought against that, and this is where the
        // number comes from rather than from a figure typed into this file.
        corners: ((c.traces || [])[0] || { points: [] }).points.length - 1,
      });
    });
  }));

  const panel = JSON.parse(await session.execute(function () {
    const el = document.querySelector('.condition-panel, .panel');
    // Property values live in input values, which innerText does not carry — a
    // check reading only the text sees the labels and none of the numbers.
    return JSON.stringify({
      text: (el || document.body).innerText || '',
      values: [...document.querySelectorAll('.condition-panel input, .panel input')]
        .map(function (i) { return i.value; }),
    });
  }));
  check('the panel offers the properties a roofing detail asks for', () => {
    for (const want of [/Height/i, /Girth/i]) {
      assert.match(panel.text, want, `panel read "${panel.text.replace(/\s+/g, ' ').slice(0, 160)}"`);
    }
  });
  check('and the height the estimator set is on the condition', () => {
    assert.ok(panel.values.includes(String(stated.H)),
      `the job says the parapet is ${stated.H} ft and the panel reads ${JSON.stringify(panel.values)}`);
  });
  check('and so is the wall thickness, which is a property and not a guess', () => {
    assert.ok(panel.values.includes(String(stated.WALL)),
      `the job says ${stated.WALL} in and the panel does not show it`);
  });

  await writeFile(join(EVIDENCE, `section2-condition-panel-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // What the watch caught while the drawing and the panel were up. Read before
  // the switch, because the picker reloads the window and the list goes with it.
  const drawingErrors = await errorsSoFar(session);
  check('nothing errored while the condition panel was open', () => {
    assert.deepEqual(drawingErrors, [], drawingErrors.join(' | '));
  });

  // ── the lines ───────────────────────────────────────────────────────────
  // The Estimate tab. There is no editor switch and no reload any more — the
  // tab is a tab, so the error watch that had to be reinstalled after every
  // switch is installed once and stays.
  await showTab(session, 'Estimate');
  await until(session, () => !!document.querySelector('.cost-estimate'), { what: 'the lines' });

  const lines = JSON.parse(await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.cost-line')].map(function (r) {
      const text = function (sel) { const el = r.querySelector(sel); return el ? (el.textContent || '').trim() : ''; };
      return {
        description: text('.line-item'),
        formula: text('.line-formula'),
        quantity: text('.line-qty'),
        unit: text('.line-unit').replace('◈', '').trim(),
        extended: text('.line-money'),
        note: text('.formula-note'),
        derived: !!r.querySelector('.line-unit .derived'),
      };
    }));
  }));

  // The named numbers the shell solved, pulled the way the app pulls them on
  // connect — needed below because `FLASH_SF` is a solved ceiling and not a
  // formula this file can reproduce from two other numbers on the sheet.
  const solvedScopes = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('solve_get', { part: 'scopes' })
      .then(function (s) { return JSON.stringify(s); })
      .catch(function (e) { return JSON.stringify({ error: String(e) }); });
  }));

  const parapet = lines.filter((l) => /flashing|coping|mitre|cleat|termination|corner|metal/i.test(l.description));
  check('the formula is on the line, and it is what the library put there', () => {
    // C3 moved the wall flashing and coping lines off `LF * H` and `SEGMENTS`
    // onto the names the wall solver carries: `FLASH_SF` is the exact area
    // under the flashing ceiling, not the whole height of the wall whether or
    // not the detail is drawn to flash that far. C8 then split the one
    // shop-formed line in two, so the pieces are named by what they are —
    // `COPING_CORNERS_EA` for the turns, the rest for the plain lengths.
    const formulas = parapet.map((l) => l.formula);
    assert.ok(formulas.includes('FLASH_SF'), `formulas read ${JSON.stringify(formulas)}`);
    assert.ok(formulas.includes('LF'), `formulas read ${JSON.stringify(formulas)}`);
    assert.ok(formulas.includes('COPING_CORNERS_EA'), `formulas read ${JSON.stringify(formulas)}`);
  });
  check('one traced run feeds three different units', () => {
    const units = new Set(parapet.map((l) => l.unit));
    for (const want of ['LF', 'EA', 'SF']) {
      assert.ok(units.has(want), `units read ${[...units].join(', ')}`);
    }
  });
  check('FLASH_SF turned a run into an area, and it is the solve\'s own', () => {
    // No longer a hand check against `run x H` — `FLASH_SF` is
    // `min(FLASHUP, wallTop - finishedRoof(p))` summed as trapezoids along the
    // run, not a plain product this file can reproduce from two numbers on the
    // sheet. What it CAN check without re-deriving the solver: the line's own
    // quantity is exactly what `solve_get('scopes')` carries for this
    // condition, because a `◈` line naming one derived name alone shows that
    // name's own value with nothing else applied to it.
    const flash = parapet.find((l) => l.formula === 'FLASH_SF');
    assert.ok(flash, 'the parapet is missing a line');
    const solved = solvedScopes?.['c-parapet']?.FLASH_SF;
    assert.ok(Number.isFinite(solved), `the solve carries no FLASH_SF: ${solved}`);
    nearly(flash.quantity, solved, 'wall flashing', 0.5);
  });
  check('and the unit on it is an area, because feet times feet is not feet', () => {
    // The other side of the dimension flag, and the side worth shipping: the
    // demo used to declare this line in LF on purpose so the advisory would
    // show, which meant the fixture an estimator opens first carried a line
    // with the wrong unit on it. It declares the area it works out to, the
    // advisory stays quiet because there is nothing to say, and the advisory
    // itself is proved below on a line changed while the window is open.
    const flash = parapet.find((l) => l.formula === 'FLASH_SF');
    assert.equal(flash.unit, 'SF', `the line is priced as ${flash.unit}`);
    assert.equal(flash.note, '', `the line carries an advisory it should not: "${flash.note}"`);
  });
  check('the corners are bought at the run\'s own turns, and the lengths apart from them', () => {
    // C8 §3a split what was one line. `COPING_PCS_EA` was sections and corners
    // together and the line that bought it was called a mitred corner, so the
    // sheet priced forty-eight of them on a roof that turns eight times. The
    // corners are their own line now and the plain lengths are the rest, so the
    // two together are the pieces and neither is the whole of them.
    const mitre = parapet.find((l) => l.formula === 'COPING_CORNERS_EA');
    assert.ok(mitre, 'no line on the parapet is bought against the run\'s own corners');
    const corners = Number(String(mitre.quantity).replace(/,/g, ''));
    assert.equal(corners, stated.corners,
      `the sheet buys ${corners} corners and the trace turns ${stated.corners}`);
    const lengths = parapet.find((l) => /COPING_PCS_EA - COPING_CORNERS_EA/.test(l.formula ?? ''));
    assert.ok(lengths, 'nothing buys the plain lengths between the corners');
    assert.ok(Number(String(lengths.quantity).replace(/,/g, '')) > 0,
      'the run is all corners and no lengths, which cannot be');
  });
  check('each line extends at its own price', () => {
    for (const l of parapet) {
      assert.match(l.extended, /\$|—/, `${l.description} extended to "${l.extended}"`);
    }
    assert.ok(parapet.some((l) => /\$/.test(l.extended)), 'no line carries money');
  });

  check('a quantity the program worked out says so, and one that was typed says so', () => {
    // The handoff's provenance contract, on the line where an estimator reads
    // it: a formula over what was traced carries the cyan mark; there is no
    // third state. This is new and it is the frame's own rule.
    const worked = parapet.filter((l) => !/^\s*-?[\d.,]+\s*$/.test(l.formula));
    assert.ok(worked.length > 0, `the parapet's formulas read ${JSON.stringify(parapet.map((l) => l.formula))}`);
    for (const l of worked) {
      assert.equal(l.derived, true, `"${l.description}" is worked out from ${l.formula} and carries no mark`);
    }
  });

  const totals = await session.execute(function () {
    return (document.querySelector('.cost-totals') || document.body).innerText || '';
  });
  check('the sheet totals to a selling price', () => {
    assert.match(totals, /SELLING PRICE/i, 'no selling price');
    assert.match(totals, /\$[\d,]+\.\d\d/, 'no money in the footer');
  });

  // ── a formula that cannot be worked out ─────────────────────────────────
  // The formula is edited where it always was: on the line. The line is opened
  // to its chain first, which is the act a person performs to get at it — and
  // the field it opens to is the same field writing to the same pointer.
  // ── the dimension advisory, on a line changed in front of us ────────────
  // Typed into the shipped window rather than asserted in a unit test, and
  // typed rather than shipped: a fixture line carrying a unit that disagrees
  // with its own formula is a demo that teaches the wrong thing. `LF` declared
  // in LF is a length; `LF * H` is an area, and the line has to say so without
  // overruling the estimator who declared it.
  const advised = await session.execute(function () {
    const line = [...document.querySelectorAll('.cost-line')]
      .find(function (r) {
        const chip = r.querySelector('.line-formula');
        return chip && (chip.textContent || '').trim() === 'LF';
      });
    if (!line) return null;
    line.querySelector('.line-what').click();
    return true;
  });
  await wait(900);
  await session.execute(function () {
    const field = document.querySelector('.line-chain input.formula');
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(field, 'LF * H');
    field.dispatchEvent(new Event('input', { bubbles: true }));
    field.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await wait(1200);
  const flagged = JSON.parse(await session.execute(function () {
    const line = document.querySelector('.cost-line.open');
    if (!line) return JSON.stringify({ found: false });
    const note = line.querySelector('.formula-note');
    return JSON.stringify({
      found: true,
      unit: ((line.querySelector('.line-unit') || {}).textContent || '').replace('◈', '').trim(),
      note: note ? (note.textContent || '').trim() : '',
    });
  }));
  check('a unit that does not follow from the formula is said, never overruled', () => {
    assert.ok(advised, 'no line carrying the formula LF to change');
    assert.ok(flagged.found, 'the line did not stay open');
    assert.match(flagged.note, /area/i, `the line's note read "${flagged.note}"`);
    assert.equal(flagged.unit, 'LF', 'the advisory changed the declared unit — it must only advise');
  });

  const opened = await session.execute(function () {
    const line = [...document.querySelectorAll('.cost-line')]
      .find(function (r) {
        const chip = r.querySelector('.line-formula');
        return chip && (chip.textContent || '').trim() === 'LF';
      });
    if (!line) return false;
    line.querySelector('.line-what').click();
    return true;
  });
  check('a line opens to its whole chain, where the formula is', () =>
    assert.equal(opened, true, 'no line carrying the formula LF'));
  await wait(900);

  await session.execute(function () {
    const field = document.querySelector('.line-chain input.formula');
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(field, 'LF * ');
    field.dispatchEvent(new Event('input', { bubbles: true }));
    field.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await wait(1200);

  const broken = JSON.parse(await session.execute(function () {
    const chain = document.querySelector('.line-chain');
    const line = document.querySelector('.cost-line.open');
    if (!chain || !line) return JSON.stringify({ found: false });
    const field = chain.querySelector('input.formula');
    const note = chain.querySelector('.formula-error') || line.querySelector('.formula-error, .formula-note');
    const rowOf = function (name) {
      const row = [...chain.querySelectorAll('.chain-row')]
        .find(function (r) { return ((r.querySelector('.chain-name') || {}).textContent || '') === name; });
      return row ? ((row.querySelector('.chain-value') || {}).textContent || '').trim() : '';
    };
    return JSON.stringify({
      found: !!field && field.value === 'LF * ',
      quantity: rowOf('Quantity'),
      extended: rowOf('Extended'),
      onTheLine: ((line.querySelector('.line-qty') || {}).textContent || '').trim(),
      note: note ? (note.textContent || '').trim() : '',
    });
  }));
  check('a formula that cannot be worked out says so on its own line', () => {
    assert.ok(broken.found, 'the broken formula did not stay on the line');
    assert.notEqual(broken.note, '', 'nothing was said about it');
  });
  check('and shows no quantity and no money rather than a zero', () => {
    assert.match(broken.quantity, /^[—-]?$/, `quantity read "${broken.quantity}"`);
    assert.match(broken.extended, /^[—-]?$/, `extended read "${broken.extended}"`);
    assert.match(broken.onTheLine, /^[—-]?$/, `the line itself read "${broken.onTheLine}"`);
  });

  await writeFile(join(EVIDENCE, `section2-estimate-lines-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  const errors = await errorsSoFar(session);
  check('nothing errored in the window', () => {
    assert.deepEqual(errors, [], errors.join(' | '));
  });
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
