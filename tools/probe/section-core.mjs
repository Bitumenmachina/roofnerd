// ── C0: the core exists, and the picture does not move ─────────────────────
// The change this check is written against is a change nobody can see.
//
// Patrick ruled on 2026-09-12 that the 3D model is the function, not a view: the
// tapered package, the walls and the fastening are what has to get built, and
// the drawings, the section and the estimate are produced from them. So the
// solvers move into Rust — `src-tauri/core`, `roofnerd-core` — and the Model
// view stops computing the roof and starts drawing what the core hands it.
//
// If that seam is right, the roof on screen is the SAME ROOF, to the pixel, and
// the only way to tell anything happened is to ask the window where its geometry
// came from. That is what this checks:
//
//   THE SOURCE. `__roofnerdModel().source` reads "solve_get" and the solve it
//   drew carries an id. Before C0 there was no such thing to ask.
//
//   THE NUMBERS. The thickness at three places on the roof, read off the status
//   strip the way an estimator reads it, against the arithmetic worked out here
//   from `jobs/demo-job` — the drains, the thickness at the drain, the slope, the
//   board count and the lowest thing the roof flashes against. Nothing is typed
//   into this file: a figure copied in here goes stale the day the demo is
//   re-traced and — worse — keeps passing while the window is wrong and the
//   fixture was moved to match it.
//
//   THE PICTURE. One capture, in the state the pre-change capture was taken in:
//   the Model as it opens, north-east corner, true scale, nothing picked. The
//   orchestrator holds it against `evidence/section-model-frame2-light-<pre>.png`
//   from 7cb2559. Nothing about the solve is drawn anywhere, so a difference in
//   that pair is a difference in the roof.
//
//   THAT IT STILL MOVES. A thickness typed into the Condition panel: the solve
//   id has to increment and the strip has to answer with the new depth. A seam
//   that only ever draws the first solve would pass every row above.
//
// Front door throughout — the start screen, the demo button, the view switch, the
// panel's own field. Nothing here calls a command to get a job open.
//
//   pnpm build:app && node tools/probe/section-core.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, errorsSoFar, launch, openDemoJob, pointerClick, pointerRelease, showTab, showView,
  typeKeys, until, wait, watchErrors, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const shot = async (session, name) =>
  writeFile(join(EVIDENCE, `${name}-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

// ── what this roof comes to, worked out from the job on disk ───────────────
const { capInches } = await import('../../packages/engine/dist/index.js');
const jobFolder = join(ROOT, 'jobs/demo-job');
const read = async (rel) => JSON.parse(await readFile(join(jobFolder, rel), 'utf8'));
const conditions = await read('conditions.json');
const pages = await read('pages/pages.json');

/** Feet to the page unit. A sheet nobody scaled has none, and no trace on it. */
const scales = {};
for (const page of pages) if (page.feetPerUnit) scales[page.id] = page.feetPerUnit;
const feet = (trace) => (scales[trace.pageId]
  ? trace.points.map((p) => ({ x: p.x * scales[trace.pageId], y: p.y * scales[trace.pageId] }))
  : []);

const drains = [];
for (const c of conditions) {
  if (c.role !== 'drain' || c.hidden) continue;
  for (const t of c.traces ?? []) drains.push(...feet(t));
}
/** The lowest thing the membrane has to turn up and flash against, in inches. */
let flashesAgainst = Infinity;
for (const c of conditions) {
  if (c.kind !== 'line' || c.hidden || c.role === 'ridge') continue;
  const h = c.properties?.H;
  if (h === undefined || h <= 0) continue;
  flashesAgainst = Math.min(flashesAgainst, h * 12);
}
const perimeter = Number.isFinite(flashesAgainst) ? flashesAgainst : undefined;
const field = conditions.find((c) => c.kind === 'area' && !c.hidden && c.properties?.TAPER !== undefined);
const ring = feet(field.traces[0]);
const atDrain = field.properties?.T ?? 0;
const slope = field.properties?.TAPER ?? 0;

/** Where the roof stops rising, for a stated thickness at the drain. */
const ceiling = (start) => capInches(start, field.properties?.BOARDS, perimeter).cap;
/** How thick the roof is at a place, for a stated thickness at the drain. */
const thicknessAt = (p, start) => {
  let run = Infinity;
  for (const d of drains) {
    const r = Math.sqrt((p.x - d.x) ** 2 + (p.y - d.y) ** 2);
    if (r < run) run = r;
  }
  return Math.min(start + run * slope, ceiling(start));
};

const box = { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity };
for (const p of ring) {
  box.minX = Math.min(box.minX, p.x); box.maxX = Math.max(box.maxX, p.x);
  box.minY = Math.min(box.minY, p.y); box.maxY = Math.max(box.maxY, p.y);
}

/**
 * Three places on the roof, each picked off the job's own geometry.
 *
 * At a drain the roof is what the estimator said it is there and nothing else,
 * which is the one place the answer cannot come out of a slope. Between two
 * drains it is the ridge the water leaves from. Four feet inboard of the far
 * corner is the long run the cone is wrong about and C1 will change — worth
 * reading now so there is a number on record from before it moves.
 */
const places = [
  { what: 'at the first drain', at: drains[0] },
  {
    what: 'on the ridge between the first two drains',
    at: { x: (drains[0].x + drains[1].x) / 2, y: (drains[0].y + drains[1].y) / 2 },
  },
  { what: 'four feet inboard of the far corner', at: { x: box.minX + 4, y: box.maxY - 4 } },
];

/** Inches out of the way a roofer writes them: 1/2", 2 1/4", 4". */
const asInches = (text) => {
  const m = /^(?:(\d+)\s*)?(?:(\d)\/(\d))?"/.exec(String(text ?? '').trim());
  if (!m) return null;
  return Number(m[1] ?? 0) + (m[2] ? Number(m[2]) / Number(m[3]) : 0);
};

/**
 * How far the strip may be off the arithmetic: three sixteenths of an inch.
 *
 * The strip writes eighths, so any reading is already up to a sixteenth off the
 * number behind it. The pointer is aimed at a projected point and then rounded
 * to a whole pixel before the ray is cast, and one pixel of this window is about
 * an inch and a half on this roof — a hundredth of an inch of depth at a quarter
 * in twelve. An eighth and a bit covers both and still catches the differences
 * that would mean anything: half an inch at the drain is the thickness the
 * estimator stated, and the far corner is inches out if the fall is wrong.
 */
const TOLERANCE_INCHES = 3 / 16;

// ── what the window says about where its roof came from ────────────────────
const modelNow = (session) => session.execute(function () {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  if (!m) return JSON.stringify({ there: false });
  const kinds = {};
  for (const mesh of m) kinds[String(mesh.kind)] = (kinds[String(mesh.kind)] || 0) + 1;
  const canvas = document.querySelector('.model-canvas');
  const b = canvas ? canvas.getBoundingClientRect() : null;
  return JSON.stringify({
    there: true,
    source: m.source === undefined ? null : m.source,
    solveId: m.solveId === undefined ? null : m.solveId,
    kinds,
    selectedConditionId: m.selectedConditionId,
    view: m.view,
    legend: ((document.querySelector('.model-legend') || {}).innerText || '').replace(/\s+/g, ' ').trim(),
    canvas: b ? { w: Math.round(b.width), h: Math.round(b.height) } : null,
    page: { w: window.innerWidth, h: window.innerHeight },
  });
});

/** Where a place on the roof lands on the screen, asked of the drawing itself. */
const aimAt = (session, at) => session.execute(function (x, y) {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  const p = m && m.screenPointAt ? m.screenPointAt(x, y) : null;
  return JSON.stringify(p);
}, at.x, at.y);

const readoutNow = (session) => session.execute(function () {
  const out = {};
  for (const f of document.querySelectorAll('.status-strip .readout .readout-field')) {
    const k = ((f.querySelector('.k') || {}).textContent || '').trim();
    out[k] = ((f.querySelector('.v') || {}).textContent || '').trim();
  }
  return JSON.stringify(out);
});

/** Point at a place on the roof and read what the strip says about it. */
const readAt = async (session, at) => {
  const aim = JSON.parse((await aimAt(session, at)) ?? 'null');
  if (!aim || !aim.inside) return { aim, reading: null };
  await session.execute(function (x, y) {
    const c = document.querySelector('.model-canvas');
    c.dispatchEvent(new MouseEvent('pointermove', { clientX: x, clientY: y, bubbles: true }));
  }, aim.x, aim.y);
  await wait(400);
  return { aim, reading: JSON.parse(await readoutNow(session)) };
};

/** The panel's field with this label on it, by the words on screen. */
const fieldValue = (session, label) => session.execute(function (want) {
  const l = [...document.querySelectorAll('.condition-panel label')]
    .find(function (e) { return ((e.querySelector('span') || {}).textContent || '').trim() === want; });
  const input = l ? l.querySelector('input') : null;
  if (!input) return JSON.stringify(null);
  const r = input.getBoundingClientRect();
  return JSON.stringify({
    value: input.value,
    x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2),
  });
});

const setFieldByScript = (session, label, value) => session.execute(function (want, text) {
  const l = [...document.querySelectorAll('.condition-panel label')]
    .find(function (e) { return ((e.querySelector('span') || {}).textContent || '').trim() === want; });
  const input = l ? l.querySelector('input') : null;
  if (!input) return false;
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
  setter.call(input, String(text));
  input.dispatchEvent(new Event('input', { bubbles: true }));
  return true;
}, label, value);

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

let pointerPath = 'not tried';
let pointerRefused = '';

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  // ── the front door ──────────────────────────────────────────────────────
  const demoAt = await centreOf(session, '.start-demo');
  if (demoAt) {
    const at = JSON.parse(demoAt);
    try {
      await pointerClick(session, at.x, at.y);
      await wait(1500);
      const opened = await session.execute(function () { return !document.querySelector('.start'); });
      pointerPath = opened ? 'actions' : 'synthetic';
      if (!opened) pointerRefused = 'the driver took the click and the job did not open';
    } catch (e) {
      pointerPath = 'synthetic';
      pointerRefused = e.message;
    }
  } else {
    pointerPath = 'synthetic';
    pointerRefused = 'the demo button was not on screen to aim at';
  }
  console.log(`  REPORT  real pointer input: ${pointerPath === 'actions'
    ? 'available — every pick below is a driver click'
    : `refused, so every pick below is a dispatched event — ${pointerRefused.split('\n')[0]}`}`);

  if (pointerPath !== 'actions') await openDemoJob(session);
  await until(session, () => !document.querySelector('.start'), { what: 'the job to open' });
  await wait(1500);

  // ── the Model, as it opens ──────────────────────────────────────────────
  await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });
  await wait(1600);
  const first = JSON.parse(await modelNow(session));

  check('the Model draws the roof the core solved, and says so', () => {
    assert.equal(first.there, true, 'the view has nothing to say about itself');
    assert.equal(first.source, 'solve_get',
      `the view says its roof came from "${first.source}"`);
    assert.ok(Number.isInteger(first.solveId) && first.solveId >= 1,
      `it drew solve ${JSON.stringify(first.solveId)}`);
  });

  check('and it is the same roof: the field, the drains, the valleys, the cricket', () => {
    // The kinds, not the vertices. What the vertices come to is the picture
    // below, and the picture is held against the one taken before the seam.
    for (const [kind, least] of [['taper', 1], ['valley', 2], ['cricket', 1], ['sump', 3]]) {
      assert.ok((first.kinds[kind] ?? 0) >= least,
        `${first.kinds[kind] ?? 0} of ${kind} in the scene: ${JSON.stringify(first.kinds)}`);
    }
    assert.ok(first.legend.includes('no fall'),
      `the key does not name the flat ground: "${first.legend}"`);
  });

  // ── the picture, in the state the one before the seam was taken in ──────
  check('the picture is taken in the state the last one was taken in', () => {
    assert.ok(first.view, 'the view reports no camera');
    assert.equal(first.view.corner, 'NE', `it opened from ${first.view.corner}`);
    assert.equal(first.view.vert, 1, `the vertical is stretched ${first.view.vert} times`);
    assert.equal(first.view.looking, false, 'the orbit is live before anybody asked');
    assert.equal(first.selectedConditionId, null,
      `"${first.selectedConditionId}" is picked, so the roof is lit and the two pictures cannot be compared`);
    assert.equal(first.page.w, 1440, `the page is ${first.page.w}px wide and is specified at 1440 (D137)`);
    assert.equal(first.page.h, 900, `the page is ${first.page.h}px tall and is specified at 900 (D137)`);
  });
  await shot(session, 'section-core-model');

  // ── the thickness at three places, off the strip ────────────────────────
  const readings = [];
  for (const place of places) readings.push(await readAt(session, place.at));

  places.forEach((place, i) => {
    const { aim, reading } = readings[i];
    const want = thicknessAt(place.at, atDrain);
    check(`the roof is ${(want).toFixed(2)} in ${place.what}, and the strip says so`, () => {
      assert.ok(aim && aim.inside,
        `${place.at.x}, ${place.at.y} projects to ${JSON.stringify(aim)}, which is not on the canvas`);
      assert.ok(reading, 'the strip said nothing');
      const said = asInches(reading['Insulation here']);
      assert.ok(said !== null,
        `the strip reads "${reading['Insulation here']}" where a depth belongs`);
      assert.ok(Math.abs(said - want) <= TOLERANCE_INCHES,
        `the strip reads ${reading['Insulation here']} (${said} in) and the job makes it ${want} in`);
    });
  });

  console.log(`  REPORT  the strip, place by place: ${places.map((p, i) =>
    `${p.what} → ${readings[i].reading ? readings[i].reading['Insulation here'] : 'nothing'}`).join(' · ')}`);

  // ── and it re-solves when the roof changes ──────────────────────────────
  //
  // Picked through the drawing, typed into the panel, read back off the strip:
  // three surfaces, one document, one solve behind all of them.
  const pick = JSON.parse((await aimAt(session, places[1].at)) ?? 'null');
  if (pick && pick.inside) {
    if (pointerPath === 'actions') {
      try { await pointerClick(session, pick.x, pick.y); } catch { /* falls through to the event */ }
    }
    await session.execute(function (x, y) {
      const c = document.querySelector('.model-canvas');
      c.dispatchEvent(new MouseEvent('click', { clientX: x, clientY: y, bubbles: true, button: 0 }));
    }, pick.x, pick.y);
    await wait(900);
  }
  await showTab(session, 'Condition');
  await wait(500);

  // The driver refuses real clicks here and a dispatched click on the roof can
  // miss the mesh, in which case nothing is selected and the panel has no field
  // to type into. The other front door to the same selection is the field's
  // own row on the Estimate tab — one document, one selection, one panel — so
  // take that door when the first one did not open.
  let was = JSON.parse((await fieldValue(session, 'Thickness')) ?? 'null');
  if (!was) {
    await showTab(session, 'Estimate');
    await session.execute(function (want) {
      const row = [...document.querySelectorAll('.cost-group')]
        .find(function (r) { return (r.textContent || '').indexOf(want) >= 0; });
      if (row) row.click();
    }, field.name);
    await wait(900);
    await showTab(session, 'Condition');
    await wait(500);
    was = JSON.parse((await fieldValue(session, 'Thickness')) ?? 'null');
  }
  // Say what the panel held at that moment, so a miss can be read off the log
  // rather than reproduced by hand.
  console.log(`  REPORT  the panel when the thickness was looked for: ${await session.execute(function () {
    const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
    const labels = [...document.querySelectorAll('.condition-panel label')]
      .map(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim(); });
    const tab = [...document.querySelectorAll('.cost-tabs .cost-tab')]
      .find(function (e) { return e.classList.contains('active') || e.getAttribute('aria-selected') === 'true'; });
    return JSON.stringify({ selected: m && m.selectedConditionId, tab: tab ? tab.textContent.trim() : null,
      panels: document.querySelectorAll('.condition-panel').length, labels: labels.slice(0, 6), handles: 1 });
  })} · the lookup answered ${JSON.stringify(await fieldValue(session, 'Thickness'))} · was=${JSON.stringify(was)} · the labels called Thickness: ${await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.condition-panel label')]
      .filter(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim() === 'Thickness'; })
      .map(function (l) { return l.outerHTML.slice(0, 200); }));
  })}`);
  const solveWas = JSON.parse(await modelNow(session)).solveId;
  // An inch thicker at the drain than the job says, whatever the job says. The
  // ceiling moves with it — another inch of board is another inch the wall has
  // to take — so the expected depth is not the old one plus an inch everywhere.
  const thicker = atDrain + 1;
  let typedBy = 'nothing';
  if (was) {
    if (pointerPath === 'actions') {
      try {
        await pointerClick(session, was.x, was.y);
        await typeKeys(session, '');
        await typeKeys(session, String(thicker));
        typedBy = 'keys';
      } catch { typedBy = 'nothing'; }
    }
    const now = JSON.parse((await fieldValue(session, 'Thickness')) ?? 'null');
    if (!now || Number(now.value) !== thicker) {
      await setFieldByScript(session, 'Thickness', thicker);
      typedBy = 'script';
    }
    await wait(1400);
  }
  console.log(`  REPORT  the thickness was typed by ${typedBy}`);

  const after = JSON.parse(await modelNow(session));
  const moved = await readAt(session, places[1].at);

  check('a thickness typed into the panel re-solves the roof, and the strip follows', () => {
    assert.ok(was, 'there is no Thickness field in the panel to type into');
    assert.ok(Number.isInteger(after.solveId) && after.solveId > solveWas,
      `the solve id was ${solveWas} and is ${after.solveId}`);
    assert.equal(after.source, 'solve_get', `the view now says "${after.source}"`);
    const want = thicknessAt(places[1].at, thicker);
    const said = asInches(moved.reading ? moved.reading['Insulation here'] : null);
    assert.ok(said !== null, `the strip reads "${moved.reading && moved.reading['Insulation here']}"`);
    assert.ok(Math.abs(said - want) <= TOLERANCE_INCHES,
      `an inch thicker at the drain should read ${want} in on the ridge and the strip reads ${said} in`);
  });

  // Put the job back the way it opened. A check that leaves the fixture changed
  // is a check that breaks the next one.
  if (was) {
    await setFieldByScript(session, 'Thickness', was.value);
    await wait(1000);
  }

  const errors = await errorsSoFar(session);
  check('and nothing in the window complained on the way through', () => {
    assert.deepEqual(errors, [], `the window logged ${JSON.stringify(errors)}`);
  });

  console.log(`\n  ${results.filter((r) => r === 'PASS').length}/${results.length}`);
  console.log(`  picture: evidence/section-core-model-${COMMIT}.png`);
  console.log('  the pair to compare it with: evidence/section-model-frame2-light-<the commit before C0>.png');
} finally {
  await pointerRelease(session);
  await app.close();
}

process.exit(results.includes('FAIL') ? 1 : 0);
