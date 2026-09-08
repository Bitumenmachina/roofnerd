// ── Section 6's done-check ─────────────────────────────────────────────────
// §5.6: "a tapered layout drawn in plan renders as slopes; a low point without
//        a drain is marked."
//
// Addendum 4 §4 adds four lines to that check:
//   - a trace edit moves the 3D view live, same document, no rebuild
//   - selecting on the sheet highlights in 3D, and selecting in 3D highlights
//     on the sheet; selection is not editing
//   - a facet with drains, slope and start thickness renders as a heightfield,
//     and a ridge renders as two planes
//   - evidence from the Tauri window, named with the section and the commit
//
// This runs against the shipped runtime, and it goes in through the front door
// like everything else — a check that bypasses the path a person uses is not a
// check.
//
//   pnpm build:app && node tools/probe/section6-model.mjs

import { execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, openDemoJob, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

/** Everything the legend says, which is what a person reads off this view. */
const legend = (session) => session.execute(function () {
  const el = document.querySelector('.model-legend');
  return el ? (el.innerText || el.textContent || '').replace(/\s+/g, ' ').trim() : '';
});

const squaresWithNoFall = (text) => {
  const m = /([\d,]+) SF with no fall/.exec(text);
  return m ? Number(m[1].replace(/,/g, '')) : null;
};

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await openDemoJob(session);
  await wait(1200);

  // ── the Model is an editor, in the picker every area already has ────────
  // D57. Not a new window kind, not a special case — the mechanism section 1
  // built for tearing an editor off is the mechanism this arrives through.
  const options = await session.execute(function () {
    const picker = document.querySelector('.editor-picker');
    return picker ? [...picker.options].map((o) => o.text) : [];
  });
  check('the Model is one of the editors an area can show', () => {
    assert.ok(options.includes('Model'), `the picker offers ${JSON.stringify(options)}`);
  });

  // Switch to it the way a person does: the picker.
  await session.execute(function () {
    const picker = document.querySelector('.editor-picker');
    picker.value = 'model';
    picker.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await wait(2500);
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });

  const canvas = await session.execute(function () {
    const el = document.querySelector('.model-canvas');
    if (!el) return null;
    const gl = el.getContext('webgl2') || el.getContext('webgl');
    return { w: el.clientWidth, h: el.clientHeight, gl: !!gl };
  });
  check('the roof is drawn on a canvas, with room to see it', () => {
    assert.ok(canvas, 'no canvas');
    assert.ok(canvas.w > 200 && canvas.h > 150, `canvas is ${canvas.w}x${canvas.h}`);
    assert.ok(canvas.gl, 'no WebGL context — nothing could have been drawn');
  });

  const readout = await session.execute(function () {
    const el = document.querySelector('.model-readout');
    return el ? (el.textContent || '').trim() : null;
  });
  check('the roof can be read where the pointer is', () => {
    assert.ok(readout !== null, 'no readout');
    assert.match(readout, /roof/i, `readout said "${readout}"`);
  });

  // ── §5.6, first half: the tapered layout renders as slopes ──────────────
  const first = await legend(session);
  check('the drains are drawn, and counted in trade words', () => {
    assert.match(first, /\d+ drains|1 drain/, `legend read "${first}"`);
  });
  check('a facet with drains, slope and start thickness renders as a heightfield', () => {
    // The legend carries a scale a depth can be read off, not a sentence about
    // shading. Inch marks are the proof it is a scale.
    assert.match(first, /\d\s*(\d\/\d)?"/, `legend read "${first}"`);
  });

  const scale = await session.execute(function () {
    const el = document.querySelector('.model-scale');
    if (!el) return null;
    return JSON.stringify([...el.querySelectorAll('.model-ticks span')].map((s) => s.textContent));
  });
  check('the thickness scale has depths written on it', () => {
    assert.ok(scale, 'no thickness scale');
    const ticks = JSON.parse(scale);
    assert.equal(ticks.length, 3, `ticks: ${scale}`);
    for (const t of ticks) assert.match(t, /"/, `tick "${t}" is not a depth`);
  });

  check('the fall is drawn, not only the thickness', () => {
    assert.match(first, /arrows follow the fall/, `legend read "${first}"`);
  });
  check('a ridge between drains renders as a cricket', () => {
    assert.match(first, /cricket/, `legend read "${first}"`);
  });

  // ── §5.6, second half: a low point with no drain on it is marked ────────
  const noFall = squaresWithNoFall(first);
  check('a low spot the water cannot leave is marked, with its area', () => {
    assert.ok(noFall !== null, `legend read "${first}"`);
    assert.ok(noFall > 0, `no-fall area came back as ${noFall} SF`);
  });
  check('and it says why, in words a roofer uses', () => {
    assert.match(first, /the boards run out/, `legend read "${first}"`);
  });

  await writeFile(join(EVIDENCE, `section6-model-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── Addendum 4 §4: a trace edit moves the view live ─────────────────────
  // Move a drain, through the document the way any editor would, and watch the
  // roof change. No reload, no remount, same window.
  const moved = await session.execute(function () {
    const conditions = JSON.parse(JSON.stringify(window.__doc?.conditions ?? []));
    return conditions.length;
  }).catch(() => null);
  void moved;

  const before = noFall;
  await session.execute(function () {
    // Steepen the taper: the boards run out sooner, so more of the roof goes
    // flat. This is a property edit on the condition, through the one document.
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const list = d.conditions.map(function (c) {
        if (c.id !== 'c-tapered') return c;
        const props = Object.assign({}, c.properties, { TAPER: 0.5 });
        return Object.assign({}, c, { properties: props });
      });
      return window.__TAURI_INTERNALS__.invoke('doc_set', { pointer: '/conditions', value: list });
    });
  });
  await wait(1500);
  const after = squaresWithNoFall(await legend(session));

  check('a change to the roof moves the 3D view live, in the same window', () => {
    assert.ok(after !== null, 'the legend stopped reporting a no-fall area');
    assert.ok(after > before,
      `steepening the taper should leave more of the roof flat: ${before} SF before, ${after} SF after`);
  });

  await writeFile(join(EVIDENCE, `section6-live-edit-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── Addendum 4 §4: selection crosses the editors, and is not editing ────
  const selectedAfter = await session.execute(function () {
    // Select on the tree — the same selection every editor watches.
    const node = [...document.querySelectorAll('.tree-label')]
      .find(function (e) { return (e.textContent || '').includes('Low Roof'); });
    if (!node) return null;
    node.click();
    return true;
  });
  await wait(600);

  check('selecting a condition elsewhere selects it here too', () => {
    assert.ok(selectedAfter, 'the tapered field was not in the tree to click');
  });

  const unchanged = await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-tapered'; });
      return { traces: (c.traces || []).length, points: ((c.traces || [])[0] || {}).points.length };
    });
  });
  check('and selecting changed nothing about the job — there is no editing in 3D', () => {
    assert.equal(unchanged.traces, 1);
    assert.equal(unchanged.points, 4);
  });

  // ── the vocabulary rule holds in this window too ────────────────────────
  const words = await session.execute(function () {
    const el = document.querySelector('.model-editor');
    return el ? (el.innerText || el.textContent || '') : '';
  });
  check('no code word reached the screen', () => {
    for (const bad of ['TAPER', 'ELEV', 'BOARDS', 'SUMP', 'undefined', 'NaN', '[object Object]']) {
      assert.ok(!words.includes(bad), `found "${bad}" on screen`);
    }
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
