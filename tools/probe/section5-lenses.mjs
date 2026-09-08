// ── Section 5's done-check ─────────────────────────────────────────────────
// "The supplier lens has no cost and no waste; the sub lens has no internal
//  notes."
//
// Plus the rule the prior lineage paid for: derivation columns never reach a
// document that goes outside this office. That one is tested by searching the
// finished page for the words a formula is written in — not by intending it.
//
//   pnpm build:app && node tools/probe/section5-lenses.mjs

import { execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, openDemoJob, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();

// The working: measure names that only appear inside a formula, the properties,
// and the rounding functions. NOT SF/LF/EA/SQ — those are units, and a
// subcontractor's quantity prints one beside every number.
const WORKING = [
  'PLAN_SF', 'VERTICES', 'SEGMENTS',
  'STRETCHOUT', 'PITCH', 'TAPER', 'ELEV', 'SUMP', 'BOARDS',
  'ceil(', 'floor(', 'round(',
];

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const pickLens = async (session, name) => {
  const ok = await session.execute(function (want) {
    const b = [...document.querySelectorAll('.lens')].find((e) => (e.querySelector('strong') || {}).textContent === want);
    if (!b) return false;
    b.click();
    return true;
  }, name);
  await wait(900);
  return ok;
};

/** Everything on the rendered lens — headings, cells, and the page's own source. */
const sheetNow = (session) => session.execute(function () {
  const el = document.querySelector('.lens-sheet');
  if (!el) return null;
  return JSON.stringify({
    headings: [...el.querySelectorAll('thead th')].map((t) => t.textContent),
    text: el.innerText || el.textContent || '',
    html: el.innerHTML,
    rows: el.querySelectorAll('tbody tr').length,
  });
});

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await openDemoJob(session);
  await wait(1200);

  await session.execute(function () {
    const p = document.querySelector('.editor-picker');
    p.value = 'reports';
    p.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await wait(2500);
  await until(session, () => !!document.querySelector('.lens-sheet'), { what: 'the reports editor' });

  const offered = await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.lens strong')].map((e) => e.textContent));
  });
  check('all five lenses are offered', () => {
    const names = JSON.parse(offered);
    for (const want of ['Drawing', 'Stocking', 'Condition Summary', 'Recap', 'Consolidated']) {
      assert.ok(names.includes(want), `missing ${want} — got ${offered}`);
    }
  });

  // ── the supplier's lens ─────────────────────────────────────────────────
  const gotStocking = await pickLens(session, 'Stocking');
  check('the supply house lens can be picked', () => assert.ok(gotStocking));

  const stocking = JSON.parse(await sheetNow(session));
  check('it lists something to send', () => {
    assert.ok(stocking.rows > 0, 'no rows');
  });
  check('and every line carries a quantity — a blank is not a quantity', () => {
    const cells = /<td[^>]*class="num"[^>]*>([^<]*)<\/td>/g;
    const found = [...stocking.html.matchAll(cells)].map((m) => m[1].trim());
    assert.ok(found.length >= stocking.rows, `only ${found.length} order cells for ${stocking.rows} rows`);
    for (const v of found) assert.notEqual(v, '', 'a line was sent with no quantity on it');
  });
  check('and has no cost column', () => {
    for (const h of stocking.headings) {
      assert.ok(!/cost|price|\$/i.test(h), `heading "${h}"`);
    }
  });
  check('no cost is anywhere in the page — not hidden, absent', () => {
    assert.ok(!/\$\s?[\d,]/.test(stocking.html), 'a dollar figure is in the markup');
  });
  check('and no waste', () => {
    assert.ok(!/waste/i.test(stocking.html), 'waste is in the markup');
  });
  check('it says what it is for, in words', () => {
    assert.match(stocking.text, /supply house/i, stocking.text.slice(0, 120));
  });

  await writeFile(join(EVIDENCE, `section5-stocking-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── the subcontractor's lens ────────────────────────────────────────────
  await pickLens(session, 'Drawing');
  const drawing = JSON.parse(await sheetNow(session));
  check('the sub sees quantities', () => {
    assert.ok(drawing.rows > 0, 'no rows');
    assert.ok(drawing.headings.some((h) => /quantity/i.test(h)), drawing.headings.join(', '));
  });
  check('the sub sees no cost', () => {
    assert.ok(!/\$\s?[\d,]/.test(drawing.html), 'a dollar figure is in the markup');
  });
  check('and none of the working — no formula, no property, no rounding', () => {
    for (const word of WORKING) {
      assert.ok(!drawing.html.includes(word), `"${word}" reached a document going outside`);
    }
  });
  check('but the units still print, because a quantity without one is useless', () => {
    assert.match(drawing.text, /\b(SF|LF|EA|SQ)\b/, drawing.text.slice(0, 200));
  });

  // ── the internal one, which is allowed to show the working ─────────────
  await pickLens(session, 'Consolidated');
  const internal = JSON.parse(await sheetNow(session));
  check('the internal lens shows the whole chain', () => {
    assert.ok(internal.headings.some((h) => /formula/i.test(h)), internal.headings.join(', '));
    assert.ok(/\$\s?[\d,]/.test(internal.html), 'no money on the internal lens');
  });
  check('and says on its face that it is internal', () => {
    assert.match(internal.text, /internal/i, internal.text.slice(0, 200));
  });

  await writeFile(join(EVIDENCE, `section5-consolidated-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── the recap, which section 5 certified without ever reading ───────────
  // Five lenses were offered and three were opened. The recap printed a table
  // of blank class names with money beside them for as long as it existed,
  // because "all five are offered" is not a check on any of them.
  await pickLens(session, 'Recap');
  const recap = JSON.parse(await sheetNow(session));
  check('the recap names its classes', () => {
    for (const want of ['Material', 'Labor', 'Sub']) {
      assert.ok(recap.text.includes(want), `no ${want} row — the recap read "${recap.text.slice(0, 160)}"`);
    }
  });
  check('and no row is money beside a blank name', () => {
    const cells = [...recap.html.matchAll(/<tr>\s*<td[^>]*>([^<]*)<\/td>/g)].map((m) => m[1].trim());
    assert.ok(cells.length > 0, 'no rows in the recap');
    for (const c of cells) assert.notEqual(c, '', 'a recap row carries a cost under no class');
  });

  // ── a lens never edits the estimate ─────────────────────────────────────
  const untouched = await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-parapet'; });
      return JSON.stringify({ items: (c.items || []).length, first: (c.items || [])[0].formula });
    });
  });
  check('and looking through a lens changed nothing', () => {
    const state = JSON.parse(untouched);
    assert.equal(state.items, 4);
    assert.equal(state.first, 'LF * H');
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
