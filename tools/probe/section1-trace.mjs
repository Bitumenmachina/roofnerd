// ── Section 1's done-check ─────────────────────────────────────────────────
// "Open a PDF, calibrate, trace an area with pitch, a line, a count; the list
//  shows SF/LF/EA; the file round-trips."
//
// This ran against a browser with the shell stubbed, which `CLAUDE.md` says is
// not a check: a section is done when its check passes against the shipped
// runtime. It also took no screenshot, so section 1 was certified with one
// orphan image no script had produced and nobody had opened.
//
// It now goes through the front door on the demo job, sets a scale by clicking
// two points and typing a real dimension, and asserts the same twelve claims
// against what the window actually holds.
//
//   pnpm build:app && node tools/probe/section1-trace.mjs

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { clickAt, launch, openDemoJob, tool, typeScale, until, wait, commitStamp } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};
const nearly = (got, want, what, tol = 0.02) =>
  assert.ok(Math.abs(Number(got) - want) <= tol, `${what}: ${got} against ${want}`);

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await openDemoJob(session);
  await wait(1800);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 40000 });
  await wait(900);

  const sheet = JSON.parse(await session.execute(function () {
    const o = document.querySelector('.surface-overlay');
    const canvas = document.querySelector('.surface canvas, canvas');
    const b = canvas ? canvas.getBoundingClientRect() : { width: 0, height: 0 };
    return JSON.stringify({
      viewBox: o.getAttribute('viewBox'),
      canvasWidth: Math.round(b.width), canvasHeight: Math.round(b.height),
    });
  }));

  check('the drawing renders, at a size a person can trace on', () => {
    assert.ok(sheet.canvasWidth > 400, `canvas is ${sheet.canvasWidth} px wide`);
  });
  check('and the overlay is in page units, not screen pixels', () => {
    const [, , w, h] = sheet.viewBox.split(/\s+/).map(Number);
    assert.equal(w, 612, `viewBox is ${sheet.viewBox}`);
    assert.equal(h, 792, `viewBox is ${sheet.viewBox}`);
  });

  // ── the sheet takes a scale, from two points and a real dimension ────────
  await tool(session, 'Scale');
  await clickAt(session, [[100, 100], [400, 100]]);
  await wait(400);
  await typeScale(session, "60'-0\"");

  const scale = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const p = d.pages[0];
      return JSON.stringify({
        feetPerUnit: p.feetPerUnit, note: p.scaleNote || '',
        badge: (document.querySelector('.scale-badge') || {}).textContent || '',
      });
    });
  }));
  check('the sheet takes a scale from two points and a real dimension', () => {
    assert.ok(scale.feetPerUnit > 0, `feetPerUnit is ${scale.feetPerUnit}`);
    assert.doesNotMatch(scale.badge, /not scaled/i, `badge read "${scale.badge}"`);
  });
  check('and it says where the scale came from', () => {
    assert.match(scale.note, /point|scale|=/i, `note read "${scale.note}"`);
  });

  // ── what was traced, and what each kind of thing measures ────────────────
  const conditions = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      return JSON.stringify(d.conditions.map(function (c) {
        return {
          id: c.id, kind: c.kind, name: c.name,
          points: (c.traces || []).reduce(function (n, t) { return n + t.points.length; }, 0),
          first: ((c.traces || [])[0] || { points: [] }).points[0] || null,
          pitch: (c.properties || {}).PITCH,
        };
      }));
    });
  }));
  check('an area, a run and a count were all traced off the same sheet', () => {
    const kinds = new Set(conditions.map((c) => c.kind));
    for (const want of ['area', 'line', 'count']) {
      assert.ok(kinds.has(want), `no ${want} condition`);
    }
  });
  check('traces are stored in page units, not screen pixels', () => {
    // A point stored in screen pixels moves when the window does. Every point
    // has to sit inside the page box the overlay declares.
    for (const c of conditions) {
      if (!c.first) continue;
      assert.ok(c.first.x >= 0 && c.first.x <= 612 && c.first.y >= 0 && c.first.y <= 792,
        `${c.name} starts at ${c.first.x}, ${c.first.y} — outside a 612 x 792 page`);
    }
  });

  const rows = JSON.parse(await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.condition')].map(function (el) {
      return (el.innerText || '').replace(/\s+/g, ' ');
    }));
  }));
  check('the area reads square feet, a run and corners, all at once', () => {
    const area = rows.find((r) => /Main Roof Field/.test(r));
    assert.ok(area, `conditions read ${JSON.stringify(rows)}`);
    assert.match(area, /SF/, area);
    assert.match(area, /LF/, area);
    assert.match(area, /EA/, area);
  });
  check('the run reads a length and its corners, and no area of its own', () => {
    const run = rows.find((r) => /Parapet Wall Flashing/.test(r));
    assert.ok(run, 'no parapet row');
    assert.match(run, /LF/, run);
    assert.doesNotMatch(run, /\bSF\b/, `a run is reporting an area: ${run}`);
  });
  check('the count is a count', () => {
    const count = rows.find((r) => /Roof Drains/.test(r));
    assert.ok(count, 'no drains row');
    assert.match(count, /\bEA\b/, count);
  });

  // ── pitch lifts the surface off the footprint ────────────────────────────
  const pitched = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-field'; });
      const page = d.pages.find(function (p) { return p.id === c.traces[0].pageId; });
      return JSON.stringify({
        pitch: (c.properties || {}).PITCH,
        points: c.traces[0].points,
        feetPerUnit: page.feetPerUnit,
      });
    });
  }));
  check('pitch lifted the surface off its own footprint', () => {
    // Worked out from the traced corners and the scale on the page, not from the
    // number the window printed. The version this replaces divided the displayed
    // area by itself and compared the result to the pitch factor — true of any
    // number at all, and a check confirming its own input is the defect this
    // whole pass exists to remove.
    assert.ok(pitched.pitch > 0, 'the field has no pitch to lift it');
    const p = pitched.points;
    let twice = 0;
    for (let i = 0; i < p.length; i += 1) {
      const q = p[(i + 1) % p.length];
      twice += p[i].x * q.y - q.x * p[i].y;
    }
    const planSF = Math.abs(twice / 2) * pitched.feetPerUnit ** 2;
    const factor = Math.sqrt(1 + (pitched.pitch / 12) ** 2);
    const area = rows.find((r) => /Main Roof Field/.test(r)) ?? '';
    const shown = Number((area.match(/([\d,.]+)\s*SF/) ?? [])[1]?.replace(/,/g, ''));
    assert.ok(shown > 0, `the row read "${area}"`);
    assert.ok(shown > planSF, `${shown} SF sloped is not more than ${planSF.toFixed(2)} SF flat`);
    nearly(shown / planSF, factor, `a ${pitched.pitch}:12 pitch factor`, 0.002);
  });

  // ── the drawing is referenced, never swallowed ───────────────────────────
  const doc = await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) { return JSON.stringify(d); });
  });
  check('the drawing is referenced, never copied into the job', () => {
    assert.ok(!doc.includes('%PDF'), 'a PDF was swallowed into the document');
    assert.match(doc, /roof-plan\.pdf/, 'the job does not point at its drawing');
  });

  const errors = await session.execute(function () {
    return JSON.stringify(window.__errors ?? []);
  });
  check('nothing errored in the page', () => {
    assert.deepEqual(JSON.parse(errors), []);
  });

  await writeFile(join(EVIDENCE, `section1-trace-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
