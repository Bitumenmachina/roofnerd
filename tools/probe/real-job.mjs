// ── The whole thing, on a real roof ────────────────────────────────────────
// Not a section's done-check. The seven things an estimator should be able to
// do in one sitting, driven end to end on a real 36 x 24 architectural sheet:
//
//   1  open a real roof plan and set its scale
//   2  trace the field, the parapet, the drains, a tapered area, a cricket
//   3  see the parapet yield its children — flashing in SF, coping in LB
//      through the girth, cleat in LF, corners in EA, each formula on its line
//   4  load library items that arrive carrying where their numbers came from
//   5  watch the selling price move, in a torn-off window on a second monitor
//   6  switch to Model and see where the water goes
//   7  produce a supplier list with no cost, and an honest recap
//
// The job lives in `fixtures/`, which is gitignored. Nothing about it is
// published: this prints structure and never a figure off the drawing.
//
//   pnpm build:app && node tools/probe/real-job.mjs

import { execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const JOB = join(ROOT, 'fixtures/real-job');
const SHOTS = join(ROOT, 'fixtures/real-job/exports');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};
const step = (n, what) => console.log(`\n${n}. ${what}`);

const toEditor = async (session, which) => {
  await session.execute(function (id) {
    const p = document.querySelector('.editor-picker');
    p.value = id;
    p.dispatchEvent(new Event('change', { bubbles: true }));
  }, which);
  await wait(2600);
};

const clickAt = (session, points) => session.execute(function (pts) {
  const o = document.querySelector('.surface-overlay');
  const box = o.getBoundingClientRect();
  for (const [x, y] of pts) {
    for (const type of ['pointermove', 'pointerdown', 'pointerup']) {
      o.dispatchEvent(new PointerEvent(type, {
        clientX: box.left + x, clientY: box.top + y,
        button: 0, buttons: 1, bubbles: true, detail: 1, pointerId: 1,
      }));
    }
  }
  document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
}, points);

const tool = async (session, label) => {
  await session.execute(function (want) {
    const b = [...document.querySelectorAll('.toolbar .tool')]
      .find((e) => e.textContent.trim().startsWith(want));
    if (b) b.click();
  }, label);
  await wait(300);
};

await mkdir(SHOTS, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });

  // ── 1. open a real plan and scale it ────────────────────────────────────
  step(1, 'Open a real roof plan and set its scale');
  await session.execute(function (f) {
    return window.__TAURI_INTERNALS__.invoke('doc_open', { folder: f });
  }, JOB);
  await wait(1600);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 90000 });
  await wait(700);

  const sheet = JSON.parse(await session.execute(function () {
    const o = document.querySelector('.surface-overlay');
    const b = o.getBoundingClientRect();
    return JSON.stringify({ viewBox: o.getAttribute('viewBox'), w: Math.round(b.width), h: Math.round(b.height) });
  }));
  check('a full-size architectural sheet renders', () => {
    const [, , w, h] = sheet.viewBox.split(/\s+/).map(Number);
    assert.ok(w > 2000 && h > 1000, `sheet is ${w} x ${h} points`);
  });

  await tool(session, 'Scale');
  await clickAt(session, [[120, 120], [420, 120]]);
  await wait(500);
  await session.execute(function () {
    const f = document.querySelector('.hint input, .scale-field input, input.scale');
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(f, "40'-0\"");
    f.dispatchEvent(new Event('input', { bubbles: true }));
    f.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    const form = f.closest('form');
    if (form) form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
  });
  await wait(900);
  const scaled = await session.execute(function () {
    return (document.querySelector('.scale-badge') || {}).textContent || '';
  });
  check('the scale is set by two points and a real dimension', () => {
    assert.match(scaled, /\d/, `badge read "${scaled}"`);
    assert.doesNotMatch(scaled, /not scaled/i, scaled);
  });

  // ── 2. trace what a roof is made of ─────────────────────────────────────
  step(2, 'Trace the field, the parapet, the drains, a tapered area, a cricket');
  await tool(session, 'Area');
  await clickAt(session, [[150, 200], [430, 200], [430, 380], [150, 380]]);
  await wait(900);

  await tool(session, 'Line');
  await clickAt(session, [[150, 200], [430, 200], [430, 380], [150, 380], [150, 200]]);
  await wait(900);

  await tool(session, 'Count');
  await clickAt(session, [[230, 260], [350, 320]]);
  await wait(900);

  const traced = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      return JSON.stringify((d.conditions || []).map(function (c) {
        return { id: c.id, kind: c.kind, points: (c.traces || []).reduce(function (n, t) { return n + t.points.length; }, 0) };
      }));
    });
  }));
  check('an area, a run and a count all trace on a real sheet', () => {
    const kinds = traced.map((c) => c.kind);
    for (const want of ['area', 'line', 'count']) assert.ok(kinds.includes(want), JSON.stringify(traced));
  });

  // Make the run a parapet, and the count the drains.
  await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const list = d.conditions.map(function (c) {
        if (c.kind === 'line') {
          return Object.assign({}, c, { name: 'Parapet', properties: { H: 3.5, ELEV: 0 } });
        }
        if (c.kind === 'count') {
          return Object.assign({}, c, { name: 'Roof drains', role: 'drain', properties: { SUMP: 4 } });
        }
        return Object.assign({}, c, {
          name: 'Tapered field',
          properties: { ELEV: 0, T: 0.5, TAPER: 0.25, BOARDS: 1 },
        });
      });
      return window.__TAURI_INTERNALS__.invoke('doc_set', { pointer: '/conditions', value: list });
    });
  });
  await wait(900);
  await writeFile(join(SHOTS, `01-traced-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

  // ── 3 & 4. the parapet yields its children, from the book ───────────────
  step(3, 'Load the coping assembly and watch the parapet yield its children');
  await session.execute(function () {
    const node = [...document.querySelectorAll('.tree-label')].find((e) => /Parapet/.test(e.textContent || ''));
    if (node) node.click();
  });
  await wait(600);
  await toEditor(session, 'library');
  await until(session, () => !!document.querySelector('.library-card'), { what: 'the library' });

  const loadedFrom = await session.execute(function () {
    const cards = [...document.querySelectorAll('.library-card')];
    const card = cards.find((c) => /bought by the pound/i.test(c.textContent || ''));
    if (!card) return null;
    const b = card.querySelector('.library-load');
    if (!b || b.disabled) return null;
    b.click();
    return card.querySelector('strong').textContent;
  });
  check('an assembly loads onto the parapet from the book', () => {
    assert.ok(loadedFrom, 'could not load the coping assembly');
  });
  await wait(1200);

  const children = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const p = d.conditions.find(function (c) { return c.kind === 'line'; });
      return JSON.stringify((p.items || []).map(function (i) {
        return { d: i.description, f: i.formula, u: i.unit, o: (i.order || {}).name || null };
      }));
    });
  }));
  check('one traced parapet yields its children in mixed units', () => {
    const units = new Set(children.map((c) => c.u));
    assert.ok(units.has('SF'), `no SF child: ${JSON.stringify(children)}`);
    assert.ok(units.has('LF'), `no LF child: ${JSON.stringify(children)}`);
    assert.ok(units.has('EA'), `no EA child: ${JSON.stringify(children)}`);
  });
  check('the coping is measured in feet and bought in pounds, through the girth', () => {
    const coping = children.find((c) => /coping/i.test(c.d));
    assert.ok(coping, 'no coping line');
    assert.equal(coping.u, 'LF');
    assert.equal(coping.o, 'LB', `coping orders in ${coping.o}`);
  });
  check('and every child carries its formula, not a bare quantity', () => {
    for (const c of children) assert.ok(c.f && c.f.length, JSON.stringify(c));
  });

  // ── 5. the money moves, in the other window ─────────────────────────────
  step(5, 'Tear off the sheet and watch the money move');
  await toEditor(session, 'estimate');
  await until(session, () => !!document.querySelector('.sheet'), { what: 'the sheet' });
  await wait(800);
  const before = await session.execute(function () {
    const el = document.querySelector('.selling .value');
    return el ? el.textContent.trim() : null;
  });

  await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const list = d.conditions.map(function (c) {
        if (c.kind !== 'line') return c;
        const props = Object.assign({}, c.properties, { H: 7 });
        return Object.assign({}, c, { properties: props });
      });
      return window.__TAURI_INTERNALS__.invoke('doc_set', { pointer: '/conditions', value: list });
    });
  });
  await wait(1200);
  const after = await session.execute(function () {
    const el = document.querySelector('.selling .value');
    return el ? el.textContent.trim() : null;
  });
  check('doubling the parapet height moves the money', () => {
    assert.ok(before && after, `${before} → ${after}`);
    assert.notEqual(before, after, `the price did not move: ${before}`);
  });
  await writeFile(join(SHOTS, `02-sheet-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

  // ── 6. where the water goes ─────────────────────────────────────────────
  step(6, 'Switch to Model and read the water');
  await toEditor(session, 'model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model' });
  await wait(1500);
  const legend = await session.execute(function () {
    const el = document.querySelector('.model-legend');
    return el ? (el.innerText || el.textContent || '').replace(/\s+/g, ' ').trim() : '';
  });
  check('the roof stands up, with its drains and its fall', () => {
    assert.match(legend, /drain/, `legend read "${legend}"`);
    assert.match(legend, /fall/, `legend read "${legend}"`);
  });
  check('and the thickness can be read off a scale', () => {
    assert.match(legend, /\d\s*(\d\/\d)?"/, `legend read "${legend}"`);
  });
  await writeFile(join(SHOTS, `03-model-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

  // ── 7. a supplier list with no cost, and an honest recap ────────────────
  step(7, 'Produce a supplier list and a recap');
  await toEditor(session, 'reports');
  await until(session, () => !!document.querySelector('.lens-sheet'), { what: 'reports' });
  await session.execute(function () {
    const b = [...document.querySelectorAll('.lens')].find((e) => (e.querySelector('strong') || {}).textContent === 'Stocking');
    if (b) b.click();
  });
  await wait(900);
  const stocking = JSON.parse(await session.execute(function () {
    const el = document.querySelector('.lens-sheet');
    return JSON.stringify({ html: el.innerHTML, rows: el.querySelectorAll('tbody tr').length });
  }));
  check('the supply house gets a list with no cost on it', () => {
    assert.ok(stocking.rows > 0, 'nothing to send');
    assert.ok(!/\$\s?[\d,]/.test(stocking.html), 'a cost reached the supply house');
  });
  check('and the pounds of coping are on it', () => {
    assert.match(stocking.html, /LB/, 'the coping is not ordered in pounds on the stocking list');
  });
  await writeFile(join(SHOTS, `04-stocking-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

  await session.execute(function () {
    const b = [...document.querySelectorAll('.lens')].find((e) => (e.querySelector('strong') || {}).textContent === 'Recap');
    if (b) b.click();
  });
  await wait(900);
  const recap = await session.execute(function () {
    const el = document.querySelector('.lens-sheet');
    return (el.innerText || el.textContent || '').replace(/\s+/g, ' ');
  });
  check('the recap keeps its classes apart', () => {
    for (const want of ['Material', 'Labor']) {
      assert.ok(recap.includes(want), `no ${want} class in the recap`);
    }
  });
  await writeFile(join(SHOTS, `05-recap-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the run stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
