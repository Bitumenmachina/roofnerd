// ── Section 6b's done-check: the same panel in every editor ────────────────
// Appendix A3: "Properties right, 320 px, contextual: the selected condition or
// item. The same panel in every editor."
//
// Only the Plan had it. An estimator on the sheet, in the Library or looking at
// the roof could see what was selected and nothing about what it was — no
// measures, no height, no money. This walks the four other editors and reads
// the panel in each of them, and it reads the states that are easy to get wrong:
// nothing selected, and the same condition after an editor switch (which is a
// reload, and takes everything in the page with it).
//
// It also tears the Model off, which is the one window this program could not
// open: `open_editor` names the window after the editor, and the Model's name
// was never added to the capabilities list, so the window came up with no way
// to ask for the job.
//
//   pnpm build:app && node tools/probe/section6b-panel.mjs

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  errorsSoFar, launch, openDemoJob, until, wait, watchErrors, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

/** Every editor an area can show, and the word a person picks it by. */
const EDITORS = [
  { id: 'estimate', title: 'Estimate Sheet', shows: '.sheet' },
  { id: 'library', title: 'Library', shows: '.library-body' },
  { id: 'reports', title: 'Reports', shows: '.lens-sheet' },
  { id: 'model', title: 'Model', shows: '.model-canvas' },
];

/**
 * What the Properties panel says, and how many of them there are.
 *
 * The count matters on the Plan in particular: the panel used to be built by
 * that editor, and moving it into the area chrome without taking the old one
 * out would leave the Plan as the one window with two.
 */
const panelNow = (session) => session.execute(function () {
  const all = [...document.querySelectorAll('.condition-panel')];
  const el = all[0];
  if (!el) return JSON.stringify({ count: 0 });
  const measure = {};
  for (const cell of el.querySelectorAll('.panel-measures > div')) {
    const label = ((cell.querySelector('.measure-label') || {}).textContent || '').trim();
    measure[label] = ((cell.querySelector('.measure-value') || {}).textContent || '').trim();
  }
  const box = el.getBoundingClientRect();
  const editor = document.querySelector('#editor');
  const editorBox = editor ? editor.getBoundingClientRect() : null;
  return JSON.stringify({
    count: all.length,
    hidden: !!el.hidden,
    width: Math.round(box.width),
    right: Math.round(box.right),
    windowWidth: window.innerWidth,
    // The panel is beside the editor, not on top of it.
    overlaps: editorBox ? Math.round(Math.max(0, editorBox.right - box.left)) : null,
    editorWidth: editorBox ? Math.round(editorBox.width) : null,
    name: (el.querySelector('.panel-name') || {}).value || null,
    kind: ((el.querySelector('.panel-kind') || {}).textContent || '').trim(),
    measure,
    cost: ((el.querySelector('.panel-money .value') || {}).textContent || '').trim(),
    empty: ((el.querySelector('.panel-empty') || {}).textContent || '').trim(),
    text: (el.innerText || el.textContent || '').replace(/\s+/g, ' ').trim(),
    railHasPanel: !!document.querySelector('.plan-rail .condition-panel'),
    railRows: document.querySelectorAll('.plan-rail .condition-list .condition').length,
  });
});

/** Switch the area to another editor, the way a person does. */
const showEditor = async (session, id, shows) => {
  await session.execute(function (want) {
    const picker = document.querySelector('.editor-picker');
    picker.value = want;
    picker.dispatchEvent(new Event('change', { bubbles: true }));
  }, id);
  await wait(2500);
  await until(session, () => !!document.querySelector('.condition-panel'),
    { what: `the panel in ${id}`, timeout: 15000 }).catch(() => undefined);
  // The editor's own body, by its own selector, with the selector baked in —
  // `until` takes no arguments for the page.
  await until(session, new Function(`return !!document.querySelector(${JSON.stringify(shows)})`),
    { what: `${id} to draw`, timeout: 15000 }).catch(() => undefined);
  await wait(600);
};

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  // ── nothing open: no job, no condition, no panel ────────────────────────
  const cold = JSON.parse(await panelNow(session));
  check('on the start screen the panel is not standing there empty', () => {
    // A3 puts the panel beside an editor. The start screen is not an editor and
    // has nothing that could be selected, so a 320px column of nothing would be
    // the program looking broken before it has been given a job.
    assert.ok(cold.count === 0 || cold.hidden,
      `the panel is on screen with no job open: "${(cold.text ?? '').slice(0, 80)}"`);
  });

  await openDemoJob(session);
  await wait(1500);

  // ── a job open, nothing picked ──────────────────────────────────────────
  const fresh = JSON.parse(await panelNow(session));
  check('with a job open and nothing picked the panel says so', () => {
    assert.equal(fresh.count, 1, `${fresh.count} panels in one window`);
    assert.ok(!fresh.hidden, 'the panel is hidden with a job open');
    assert.match(fresh.empty ?? '', /Nothing picked/i,
      `it reads "${(fresh.empty || fresh.text || '').slice(0, 120)}"`);
  });
  check('and it is 320px wide, at the right, beside the editor and not over it', () => {
    // A3 says 320. The rest of this row is the part a number cannot fake: the
    // editor ends where the panel starts, and the panel ends at the window.
    assert.equal(fresh.width, 320, `the panel is ${fresh.width}px wide`);
    assert.equal(fresh.overlaps, 0,
      `the editor and the panel overlap by ${fresh.overlaps}px`);
    assert.ok(fresh.right <= fresh.windowWidth,
      `the panel ends at ${fresh.right}px in a ${fresh.windowWidth}px window`);
    assert.ok(fresh.editorWidth > 400,
      `the editor is left with ${fresh.editorWidth}px`);
  });
  check('the Plan keeps its list and does not keep a second panel', () => {
    assert.ok(!fresh.railHasPanel, 'the Plan still builds its own panel inside the rail');
    assert.ok(fresh.railRows >= 3,
      `the Plan's condition list shows ${fresh.railRows} rows`);
  });

  // ── pick one, on the Plan, the way a person does ────────────────────────
  const picked = await session.execute(function () {
    const row = [...document.querySelectorAll('.plan-rail .condition-list .condition')]
      .find(function (r) { return (r.textContent || '').indexOf('Parapet') >= 0; });
    if (!row) return null;
    row.click();
    return (row.textContent || '').replace(/\s+/g, ' ').trim();
  });
  await wait(900);
  const onPlan = JSON.parse(await panelNow(session));
  check('picking a condition on the Plan fills the panel', () => {
    assert.ok(picked, 'the Plan lists no parapet to pick');
    assert.ok(onPlan.name && onPlan.name.length > 0, 'the panel names nothing');
    assert.match(onPlan.measure?.['Run'] ?? '', /\d/, `its run reads "${onPlan.measure?.['Run']}"`);
    assert.match(onPlan.cost ?? '', /\$|nothing priced/,
      `its cost line reads "${onPlan.cost}"`);
  });

  await writeFile(join(EVIDENCE, `section6b-panel-plan-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  const wanted = onPlan.name;
  const wantedMeasures = onPlan.measure;
  const wantedCost = onPlan.cost;

  // ── the same panel, in each of the other four editors ───────────────────
  for (const editor of EDITORS) {
    await showEditor(session, editor.id, editor.shows);
    // Switching editors reloads the window, which takes the error watch with it.
    await watchErrors(session);
    const there = JSON.parse(await panelNow(session));
    const showing = await session.execute(function (sel) {
      return !!document.querySelector(sel);
    }, editor.shows);

    check(`${editor.title}: the editor is up`, () => {
      assert.ok(showing, `nothing matching ${editor.shows} is in the window`);
    });
    check(`${editor.title}: the same panel is beside it, once`, () => {
      assert.equal(there.count, 1, `${there.count} panels in one window`);
      assert.ok(!there.hidden, 'the panel is hidden');
      assert.equal(there.width, 320, `the panel is ${there.width}px wide`);
      assert.equal(there.overlaps, 0, `it overlaps the editor by ${there.overlaps}px`);
    });
    check(`${editor.title}: it still names the condition that is selected`, () => {
      // The switch is a reload. The selection is remembered per window, and the
      // panel is rebuilt from it — this is the row that would have caught the
      // Library arriving with "pick a condition" after a switch.
      assert.equal(there.name, wanted, `the panel names "${there.name}"`);
    });
    check(`${editor.title}: with the same measures and the same money`, () => {
      for (const key of Object.keys(wantedMeasures)) {
        assert.equal(there.measure[key], wantedMeasures[key],
          `${key} reads "${there.measure[key]}" here and "${wantedMeasures[key]}" on the Plan`);
      }
      assert.equal(there.cost, wantedCost,
        `the cost reads "${there.cost}" here and "${wantedCost}" on the Plan`);
    });

    await writeFile(join(EVIDENCE, `section6b-panel-${editor.id}-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));

    const errors = await errorsSoFar(session);
    check(`${editor.title}: nothing errored while it was open`, () => {
      assert.deepEqual(errors, [], errors.join(' | '));
    });
  }

  // ── the Model, torn off into its own window ─────────────────────────────
  // The area is showing the Model, so its tear-off button gives a Model window.
  // Before this batch that window had no capabilities: `doc_get` was refused, no
  // job ever arrived, and it would have sat on the start screen looking like the
  // program had lost the job.
  const oneWindow = await session.handles();
  const tore = await session.execute(function () {
    const b = [...document.querySelectorAll('.area-header .icon-button')]
      .find((e) => (e.getAttribute('aria-label') || '') === 'Open in its own window');
    if (!b) return false;
    b.click();
    return true;
  });
  await wait(3500);
  const handles = await session.handles();
  const torn = handles.find((h) => !oneWindow.includes(h)) ?? null;

  check('the Model tears off into its own window', () => {
    assert.ok(tore, 'no tear-off button in the area header');
    assert.ok(torn, `${oneWindow.length} window before and ${handles.length} after`);
  });

  if (torn) {
    await session.switchTo(torn);
    await wait(2500);
    const inTorn = JSON.parse(await session.execute(function () {
      return JSON.stringify({
        start: !!document.querySelector('.start'),
        canvas: !!document.querySelector('.model-canvas'),
        panels: document.querySelectorAll('.condition-panel').length,
        status: ((document.querySelector('.status-bar') || {}).textContent || '').trim(),
        tree: !!document.querySelector('.sidebar .tree'),
      });
    }));
    const tornPanel = JSON.parse(await panelNow(session));

    check('and the torn-off Model has the job in it, not the start screen', () => {
      assert.ok(!inTorn.start,
        'the torn-off window is on the start screen — it could not ask for the job');
      assert.ok(inTorn.canvas, 'no canvas in the torn-off window');
      assert.match(inTorn.status, /\//, `its status bar reads "${inTorn.status.slice(0, 80)}"`);
    });
    check('and the panel rides with it (D113), with its own copy of the selection', () => {
      assert.equal(inTorn.panels, 1, `${inTorn.panels} panels in the torn-off window`);
      // A window torn off after a condition was picked starts with its own empty
      // memory of what is selected, so what it must show is the "nothing picked"
      // sentence — not a blank column, and not somebody else's condition.
      assert.ok(tornPanel.name === wanted || /Nothing picked/i.test(tornPanel.empty ?? ''),
        `it reads "${(tornPanel.empty || tornPanel.text || '').slice(0, 120)}"`);
    });

    await writeFile(join(EVIDENCE, `section6b-panel-model-torn-off-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));
  }
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
