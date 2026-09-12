// ── Section 4's done-check ─────────────────────────────────────────────────
// "Load a generic assembly onto a traced condition and price it under two
//  scenarios."
//
// Against the shipped runtime, in through the front door.
//
//   pnpm build:app && node tools/probe/section4-library.mjs

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, openDemoJob, until, wait, commitStamp } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const toEditor = async (session, id) => {
  await session.execute(function (which) {
    const p = document.querySelector('.editor-picker');
    p.value = which;
    p.dispatchEvent(new Event('change', { bubbles: true }));
  }, id);
  await wait(2500);
};

const sellingPrice = (session) => session.execute(function () {
  const el = document.querySelector('.selling .value');
  return el ? (el.textContent || '').trim() : null;
});

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await openDemoJob(session);
  await wait(1200);

  // ── the library is an editor like any other ─────────────────────────────
  const editors = await session.execute(function () {
    const p = document.querySelector('.editor-picker');
    return p ? [...p.options].map((o) => o.text) : [];
  });
  check('the Library is one of the editors an area can show', () => {
    assert.ok(editors.includes('Library'), `the picker offers ${JSON.stringify(editors)}`);
  });

  await toEditor(session, 'library');
  await until(session, () => !!document.querySelector('.library-card'), { what: 'the library' });

  const book = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      assemblies: [...document.querySelectorAll('.library-section')]
        .find((s) => /Assemblies/.test(s.querySelector('h3').textContent))
        ?.querySelectorAll('.library-card').length ?? 0,
      girths: [...document.querySelectorAll('.library-girth')].map((e) => e.textContent),
      loadLabel: (document.querySelector('.library-load') || {}).textContent || '',
    });
  }));

  check('the book has assemblies in it', () => {
    assert.ok(book.assemblies >= 1, `${book.assemblies} assemblies`);
  });
  check('a profile shows its girth, as the sum it is', () => {
    assert.ok(book.girths.length >= 1, 'no profiles');
    for (const g of book.girths) assert.match(g, /in girth/, `girth read "${g}"`);
  });
  check('and it will not load onto nothing', () => {
    assert.match(book.loadLabel, /Pick a condition/, `button read "${book.loadLabel}"`);
  });

  // ── pick a traced condition, then load a generic assembly onto it ───────
  const picked = await session.execute(function () {
    const node = [...document.querySelectorAll('.tree-label')]
      .find((e) => /Low Roof/.test(e.textContent || ''));
    if (!node) return null;
    node.click();
    return true;
  });
  check('a traced condition can be picked from the tree', () => assert.ok(picked));
  await wait(700);

  const before = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-tapered'; });
      return JSON.stringify({ items: (c.items || []).length });
    });
  }));
  check('and it starts with no items on it', () => assert.equal(before.items, 0));

  const loaded = await session.execute(function () {
    const b = [...document.querySelectorAll('.library-load')].find((e) => !e.disabled);
    if (!b) return null;
    const label = b.textContent;
    b.click();
    return label;
  });
  check('the button names the condition it will load onto', () => {
    assert.ok(loaded, 'no enabled load button');
    assert.match(loaded, /Load onto .+/, `button read "${loaded}"`);
  });
  await wait(1200);

  const after = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-tapered'; });
      const ids = (c.items || []).map(function (i) { return i.id; });
      return JSON.stringify({
        items: ids.length,
        unique: new Set(ids).size,
        assemblyId: c.assemblyId || null,
        formulas: (c.items || []).map(function (i) { return i.formula; }),
      });
    });
  }));

  check('the assembly landed on the condition', () => {
    assert.ok(after.items >= 3, `${after.items} items`);
    assert.ok(after.assemblyId, 'the condition does not say which assembly it came from');
  });
  check('its items carry their own ids, so two conditions cannot share a price', () => {
    assert.equal(after.items, after.unique, 'duplicate item ids');
  });
  check('and they came with formulas, not bare quantities', () => {
    assert.ok(after.formulas.every((f) => typeof f === 'string' && f.length > 0), JSON.stringify(after.formulas));
  });

  await writeFile(join(EVIDENCE, `section4-library-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── price it under two scenarios ────────────────────────────────────────
  await toEditor(session, 'estimate');
  await until(session, () => !!document.querySelector('.sheet'), { what: 'the sheet' });
  await wait(800);

  const scenarios = JSON.parse(await session.execute(function () {
    const s = document.querySelector('.scenario-bar select');
    return JSON.stringify(s ? [...s.options].map((o) => ({ id: o.value, name: o.text })) : []);
  }));
  check('the sheet offers the job\'s price sets', () => {
    assert.ok(scenarios.length >= 2, `${scenarios.length} scenario(s)`);
  });

  const first = await sellingPrice(session);
  check('the loaded assembly is priced under the first', () => {
    assert.ok(first, 'no selling price');
  });

  // Give the second scenario a price for one of the loaded items, then switch.
  await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-tapered'; });
      const first = (c.items || [])[0];
      const job = JSON.parse(JSON.stringify(d.job));
      job.scenarios[1].prices = {};
      job.scenarios[1].prices[first.id] = 999;
      return window.__TAURI_INTERNALS__.invoke('doc_set', { pointer: '/job', value: job });
    });
  });
  await wait(800);

  await session.execute(function () {
    const s = document.querySelector('.scenario-bar select');
    s.value = s.options[1].value;
    s.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await wait(1200);

  const second = await sellingPrice(session);
  check('switching price set re-prices the same takeoff', () => {
    assert.ok(second, 'no selling price under the second scenario');
    assert.notEqual(second, first, `both scenarios priced ${first} — the switch did nothing`);
  });

  await writeFile(join(EVIDENCE, `section4-two-scenarios-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  const words = await session.execute(function () {
    const el = document.querySelector('.library-editor');
    return el ? (el.innerText || el.textContent || '') : '';
  });
  check('no code word reached the library', () => {
    for (const bad of ['STRETCHOUT', 'undefined', 'NaN', '[object Object]', 'costCode']) {
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
