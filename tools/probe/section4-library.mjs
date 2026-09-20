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
import { launch, openDemoJob, showTab, until, wait, commitStamp } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

/**
 * Open the book, the way a person does: the Condition tab, then "Load an
 * assembly".
 *
 * There is no Library editor to switch to any more. The book is reached from
 * the condition it is going to be loaded onto, which is the order the act
 * actually happens in — and it is why the button beside it can say which
 * condition it will land on.
 */
const openTheBook = async (session) => {
  await showTab(session, 'Condition');
  const pressed = await session.execute(function () {
    const b = document.querySelector('.cost-load');
    if (!b || b.disabled) return false;
    b.click();
    return true;
  });
  await wait(1200);
  return pressed;
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

  // ── the book is reached from the condition it loads onto ────────────────
  await showTab(session, 'Condition');
  const beforePicking = await session.execute(function () {
    const b = document.querySelector('.cost-load');
    return JSON.stringify({ there: !!b, off: b ? !!b.disabled : null, label: b ? b.textContent : '' });
  });
  check('the Condition tab offers the book', () => {
    const state = JSON.parse(beforePicking);
    assert.equal(state.there, true, 'there is no way into the library');
    assert.match(state.label, /Load an assembly/, `the button reads "${state.label}"`);
  });
  check('and it will not open onto nothing', () => {
    // Nothing is picked yet, and an assembly is loaded ONTO a condition. The
    // button says what it needs rather than opening a book that cannot be used.
    assert.equal(JSON.parse(beforePicking).off, true, 'it is live with no condition picked');
  });

  // ── pick a traced condition, then load a generic assembly onto it ───────
  const picked = await session.execute(function () {
    const row = [...document.querySelectorAll('.cost-condition .condition')]
      .find((e) => /Main Roof Field/.test(e.textContent || ''));
    if (!row) return null;
    row.click();
    return true;
  });
  check('a traced condition can be picked from the list', () => assert.ok(picked));
  await wait(700);

  const openedBook = await openTheBook(session);
  check('and the book opens onto it', () => assert.equal(openedBook, true, 'the button would not open'));
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
    for (const g of book.girths) assert.match(g, /in stretch-out/, `stretch-out read "${g}"`);
  });
  check('and the card names the condition it will load onto', () => {
    assert.match(book.loadLabel, /Load onto /, `button read "${book.loadLabel}"`);
  });

  const before = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-field'; });
      return JSON.stringify({ items: (c.items || []).length });
    });
  }));
  check('and it starts with the lines the demo ships on it', () => {
    // It used to start with none, because the fixture's second roof was traced
    // and priced by nothing. The demo is one roof now and it carries its whole
    // build-up, so what this row owes is the count to measure the load against
    // — an assembly that landed on an empty condition and one that landed on a
    // full one fail differently, and only the second is what a person does.
    assert.ok(before.items > 0, 'the roof field ships with nothing priced on it');
  });

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
      const c = d.conditions.find(function (x) { return x.id === 'c-field'; });
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
    assert.ok(after.items >= before.items + 3,
      `${before.items} lines before and ${after.items} after`);
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
  await showTab(session, 'Estimate');
  await until(session, () => !!document.querySelector('.cost-estimate'), { what: 'the lines' });
  await wait(800);

  const scenarios = JSON.parse(await session.execute(function () {
    const s = document.querySelector('.scenario-bar select');
    return JSON.stringify(s ? [...s.options].map((o) => ({ id: o.value, name: o.text })) : []);
  }));
  check('the Estimate tab offers the job\'s price sets', () => {
    assert.ok(scenarios.length >= 2, `${scenarios.length} of them`);
  });

  const first = await sellingPrice(session);
  check('the loaded assembly is priced under the first', () => {
    assert.ok(first, 'no selling price');
  });

  // Give the second scenario a price for one of the loaded items, then switch.
  await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-field'; });
      // The LAST line, which is one the book just put there. A line the demo
      // ships carries a price typed on it, and a price on the line wins over
      // the book — so pricing one of those in the second scenario would move
      // nothing and the row below would go red on a program that was right.
      const items = c.items || [];
      const first = items[items.length - 1];
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

  await showTab(session, 'Condition');
  await openTheBook(session);
  await wait(900);
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
