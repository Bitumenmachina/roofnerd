// ── C1: the field is laid out, not sampled ─────────────────────────────────
// The change this check is written against is the first one an estimator can see.
//
// C0 put the roof's solve in Rust and changed nothing on screen: the field was
// still a cone from each drain — thickness = what you put at the drain plus the
// slope times the STRAIGHT-LINE distance to the nearest one. That is a picture of
// drainage, not a tapered package. NRCA lays a field out from its drains along
// slope directions, with the ridge between two drains equidistant and square to
// the line joining them (Membrane Roof Systems 2019, p.166, Fig. 4-17) and every
// valley centreline at 45 degrees to the slope (p.166, Figs. 4-15 and 4-16). C1
// makes the field that surface: one drainage region per drain, four planes in
// each, thickness = T + TAPER x the greater of the two runs in the field's own
// axes.
//
// So this check reads the roof off the window and holds it to that layout:
//
//   THE REGIONS. One tapered surface in the scene per drain on the field. Before
//   C1 there was one surface for the whole field, whatever the drains did.
//
//   THE CORNER. The thickness two feet inboard of a traced corner, read off the
//   status strip the way an estimator reads it, against `T + TAPER x
//   max(|dx|, |dy|)` in the field's own axes — worked out here from
//   `jobs/demo-job` and from nothing else. On a cone that same corner reads
//   inches thicker, so this row is the whole difference between C0 and C1.
//
//   THE VALLEY. A point on a 45 degree line out of a drain, where two of the four
//   planes tie by construction — proven to tie here, in this file's own
//   arithmetic — and the strip made to agree with the tied depth.
//
//   THE HIGH SIDE. The legend's thick figure against the greatest depth the
//   layout reaches, after the cap.
//
//   THE NO-FALL GROUND. The legend's square feet against the plateau the cap
//   makes, measured here by sampling the roof on a quarter-foot lattice — an
//   independent integral rather than a second copy of the solver's clipper. Or
//   absent altogether, when the cap does not bind on this job.
//
// Front door throughout — the start screen, the demo button, the view switch, the
// chips. Nothing here calls a command to get a job open. Nothing is typed into
// this file either: every figure comes off `jobs/demo-job`, because a figure
// copied in here goes stale the day the demo is re-traced and keeps passing while
// the window is wrong.
//
// The picture WILL differ from C0's, which is the point of C1: the ramp steps
// along four planes instead of sliding out of a cone, the valleys are dashed at
// 45 degrees, the ridges stand between the drains, and the arrows run down the
// plane a place is on rather than at the nearest drain.
//
//   pnpm build:app && node tools/probe/section-taper.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, errorsSoFar, launch, openDemoJob, pointerClick, pointerRelease, showView,
  until, wait, watchErrors, commitStamp,
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
const readJob = async (rel) => JSON.parse(await readFile(join(jobFolder, rel), 'utf8'));
const conditions = await readJob('conditions.json');
const pages = await readJob('pages/pages.json');

/** Feet to the page unit. A sheet nobody scaled has none, and no trace on it. */
const scales = {};
for (const page of pages) if (page.feetPerUnit) scales[page.id] = page.feetPerUnit;
const feet = (trace) => (scales[trace.pageId]
  ? trace.points.map((p) => ({ x: p.x * scales[trace.pageId], y: p.y * scales[trace.pageId] }))
  : []);

const field = conditions.find((c) => c.kind === 'area' && !c.hidden && c.properties?.TAPER !== undefined);
const ring = feet(field.traces[0]);
const atDrain = field.properties?.T ?? 0;
const slope = field.properties?.TAPER ?? 0;

/** Ray casting, the same rule the engine's `pointInPolygon` uses. */
const inside = (p, poly) => {
  let hit = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i, i += 1) {
    const a = poly[i]; const b = poly[j];
    if ((a.y > p.y) !== (b.y > p.y) && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) hit = !hit;
  }
  return hit;
};

const drains = [];
for (const c of conditions) {
  if (c.role !== 'drain' || c.hidden) continue;
  for (const t of c.traces ?? []) drains.push(...feet(t));
}
/** The drains this field is laid out from: the ones on it. */
const onField = drains.filter((d) => inside(d, ring));

/** The lowest thing the membrane has to turn up and flash against, in inches. */
let flashesAgainst = Infinity;
for (const c of conditions) {
  if (c.kind !== 'line' || c.hidden || c.role === 'ridge') continue;
  const h = c.properties?.H;
  if (h === undefined || h <= 0) continue;
  flashesAgainst = Math.min(flashesAgainst, h * 12);
}
const perimeter = Number.isFinite(flashesAgainst) ? flashesAgainst : undefined;
const { cap } = capInches(atDrain, field.properties?.BOARDS, perimeter);

/**
 * The field's own axes: the longest side it was traced with, and square to it.
 *
 * NRCA lays a four-way pattern out on the roof's own edges and cuts the valley
 * boards at 45 degrees from one edge (p.166), so the axes are the field's, not
 * the compass's. The longest side is the one a string line goes on.
 */
let longest = null;
for (let i = 0; i < ring.length; i += 1) {
  const a = ring[i]; const b = ring[(i + 1) % ring.length];
  const run = Math.hypot(b.x - a.x, b.y - a.y);
  if (run > 1e-9 && (longest === null || run > longest.run)) {
    longest = { run, u: { x: (b.x - a.x) / run, y: (b.y - a.y) / run } };
  }
}
let axisAlong = longest.u;
if (axisAlong.x < -1e-12 || (Math.abs(axisAlong.x) <= 1e-12 && axisAlong.y < 0)) {
  axisAlong = { x: -axisAlong.x, y: -axisAlong.y };
}
const axisAcross = { x: -axisAlong.y, y: axisAlong.x };

/** Which region a place is in: the drain whose perpendicular bisectors keep it. */
const regionDrain = (p) => {
  let best = null; let far = Infinity;
  for (const d of onField) {
    const r = (p.x - d.x) ** 2 + (p.y - d.y) ** 2;
    if (r < far) { far = r; best = d; }
  }
  return best;
};

/** The four runs out of a drain, in the field's axes: (along, across) and back. */
const runs = (p, d) => {
  const along = (p.x - d.x) * axisAlong.x + (p.y - d.y) * axisAlong.y;
  const across = (p.x - d.x) * axisAcross.x + (p.y - d.y) * axisAcross.y;
  return [along, -along, across, -across];
};

/** How thick the roof is at a place: the greatest of the four runs, capped. */
const thicknessAt = (p) => {
  const d = regionDrain(p);
  const rise = Math.max(...runs(p, d));
  return Math.min(atDrain + rise * slope, cap);
};

/** Where the fall runs out, in feet from the drain along an axis. */
const capRun = slope > 0 && Number.isFinite(cap) ? (cap - atDrain) / slope : Infinity;

// ── the places this check reads ────────────────────────────────────────────
const middle = ring.reduce((acc, p) => ({ x: acc.x + p.x / ring.length, y: acc.y + p.y / ring.length }), { x: 0, y: 0 });

/**
 * Two feet inboard of the traced corner furthest from any drain.
 *
 * Inboard because a corner is ON the outline, and a point on a polygon's own
 * edge is neither in nor out of it — the ray cast says so and the drawing says
 * so. Two feet in is on the roof by any reading, and the furthest corner is
 * where the cone and the layout disagree most.
 */
const corner = (() => {
  let best = null; let far = -Infinity;
  for (const p of ring) {
    const r = Math.min(...onField.map((d) => Math.hypot(p.x - d.x, p.y - d.y)));
    if (r > far) { far = r; best = p; }
  }
  const away = Math.hypot(middle.x - best.x, middle.y - best.y);
  return { x: best.x + ((middle.x - best.x) / away) * 2, y: best.y + ((middle.y - best.y) / away) * 2 };
})();

/**
 * A place on a valley: out from the first drain at 45 degrees to the axes.
 *
 * Walked a foot at a time so the point is found on the job's own geometry rather
 * than aimed at by a distance somebody chose, and stopped while it is still on
 * the roof, still in its own drain's region, and still below the cap — which is
 * where the crease is. Three fifths of the way along the walk, so it is well
 * clear of both ends.
 */
const valleyPlace = (() => {
  const d = onField[0];
  const dir = {
    x: (axisAlong.x + axisAcross.x) / Math.SQRT2,
    y: (axisAlong.y + axisAcross.y) / Math.SQRT2,
  };
  let reach = 0;
  for (let t = 1; t < 400; t += 1) {
    const p = { x: d.x + dir.x * t, y: d.y + dir.y * t };
    if (!inside(p, ring) || regionDrain(p) !== d) break;
    if (Math.max(...runs(p, d)) > capRun) break;
    reach = t;
  }
  const t = Math.max(1, Math.round(reach * 0.6));
  return { drain: d, at: { x: d.x + dir.x * t, y: d.y + dir.y * t } };
})();

/**
 * The plateau the cap makes, in square feet, by sampling.
 *
 * A quarter-foot lattice over the field's box, counting the cells whose middle is
 * on the roof and past the cap. It is an integral and not a clipper: the point of
 * measuring it a different way from the way the core computes it is that the two
 * have no arithmetic in common. A quarter foot on a hundred foot roof puts the
 * answer within a couple of square feet of the exact polygon, which is finer than
 * the legend prints.
 */
const STEP = 0.25;
const plateau = (() => {
  const box = ring.reduce((b, p) => ({
    minX: Math.min(b.minX, p.x), maxX: Math.max(b.maxX, p.x),
    minY: Math.min(b.minY, p.y), maxY: Math.max(b.maxY, p.y),
  }), { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity });
  let flat = 0;
  for (let x = box.minX + STEP / 2; x < box.maxX; x += STEP) {
    for (let y = box.minY + STEP / 2; y < box.maxY; y += STEP) {
      const p = { x, y };
      if (!inside(p, ring)) continue;
      if (Math.max(...runs(p, regionDrain(p))) > capRun) flat += STEP * STEP;
    }
  }
  return flat;
})();

/** The greatest depth the layout reaches: at a corner of a region, after the cap. */
const highSide = (() => {
  let most = -Infinity;
  for (const p of ring) most = Math.max(most, thicknessAt(p));
  // The bisectors put corners of their own where a region meets a neighbour, and
  // the deepest place on this roof is on one of them as often as on a traced
  // corner. Walk the outline so neither is missed.
  for (let i = 0; i < ring.length; i += 1) {
    const a = ring[i]; const b = ring[(i + 1) % ring.length];
    const run = Math.hypot(b.x - a.x, b.y - a.y);
    for (let t = 0; t <= run; t += 0.5) {
      most = Math.max(most, thicknessAt({ x: a.x + ((b.x - a.x) * t) / run, y: a.y + ((b.y - a.y) * t) / run }));
    }
  }
  return most;
})();

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
 * number behind it, and the pointer is aimed at a projected point rounded to a
 * whole pixel — about an inch and a half of roof in this window, which is a
 * hundredth of an inch of depth at a quarter in twelve. C0 settled on this
 * figure for the same two reasons and it still catches everything that would
 * mean anything: the corner reads inches out if the layout is a cone.
 */
const TOLERANCE_INCHES = 3 / 16;

// ── what the window says about the roof it drew ────────────────────────────
const modelNow = (session) => session.execute(function () {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  if (!m) return JSON.stringify({ there: false });
  const kinds = {};
  for (const mesh of m) kinds[String(mesh.kind)] = (kinds[String(mesh.kind)] || 0) + 1;
  const ticks = [...document.querySelectorAll('.model-ticks span')].map(function (t) {
    return (t.textContent || '').trim();
  });
  return JSON.stringify({
    there: true,
    source: m.source === undefined ? null : m.source,
    solveId: m.solveId === undefined ? null : m.solveId,
    kinds,
    ticks,
    selectedConditionId: m.selectedConditionId,
    view: m.view,
    legend: ((document.querySelector('.model-legend') || {}).innerText || '').replace(/\s+/g, ' ').trim(),
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

/** A chip on the view rail, clicked the way a hand clicks it. */
const pressChip = async (session, label) => {
  const where = await centreOf(session, '.view-chip', label);
  if (!where) return false;
  const at = JSON.parse(where);
  try { await pointerClick(session, at.x, at.y); } catch { /* the event below */ }
  await session.execute(function (want) {
    const el = [...document.querySelectorAll('.view-chip')]
      .find(function (c) { return ((c.textContent || '').indexOf(want) >= 0); });
    if (el) el.click();
  }, label);
  await wait(900);
  return true;
};

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

  check('the field is laid out as one region per drain on it', () => {
    assert.equal(first.there, true, 'the view has nothing to say about itself');
    assert.equal(first.source, 'solve_get', `the view says its roof came from "${first.source}"`);
    assert.equal(first.kinds.taper ?? 0, onField.length,
      `${first.kinds.taper ?? 0} tapered surfaces in the scene and ${onField.length} drains on the field`);
  });

  check('the valleys and the ridges are drawn, and they are the layout\'s own', () => {
    // Four creases to a drain where the region is wide enough to hold them, and a
    // ridge between every pair of regions that share a boundary. The count of
    // each is the roof's; what this row refuses is none at all.
    assert.ok((first.kinds.valley ?? 0) >= onField.length,
      `${first.kinds.valley ?? 0} valley lines for ${onField.length} regions`);
    assert.ok((first.kinds.ridge ?? 0) >= onField.length - 1,
      `${first.kinds.ridge ?? 0} ridge lines between ${onField.length} regions`);
  });

  // ── the picture, in the state C0's was taken in ──────────────────────────
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
  await shot(session, 'section-taper-frame2');

  // ── the corner, off the strip ────────────────────────────────────────────
  const atCorner = await readAt(session, corner);
  check(`the roof is ${thicknessAt(corner).toFixed(2)} in two feet inboard of the far corner, and the strip says so`, () => {
    assert.ok(atCorner.aim && atCorner.aim.inside,
      `${corner.x}, ${corner.y} projects to ${JSON.stringify(atCorner.aim)}, which is not on the canvas`);
    assert.ok(atCorner.reading, 'the strip said nothing');
    const said = asInches(atCorner.reading['Insulation here']);
    assert.ok(said !== null, `the strip reads "${atCorner.reading['Insulation here']}" where a depth belongs`);
    assert.ok(Math.abs(said - thicknessAt(corner)) <= TOLERANCE_INCHES,
      `the strip reads ${atCorner.reading['Insulation here']} (${said} in) and the layout makes it ${thicknessAt(corner)} in`);
  });

  // ── a place on a valley, where two planes tie ────────────────────────────
  const atValley = await readAt(session, valleyPlace.at);
  check('two of the four planes tie on the valley, and the strip reads the depth they agree on', () => {
    const rise = runs(valleyPlace.at, valleyPlace.drain);
    const most = Math.max(...rise);
    const tied = rise.filter((r) => Math.abs(r - most) < 1e-9).length;
    assert.equal(tied, 2,
      `${tied} of the four runs are the greatest at ${valleyPlace.at.x}, ${valleyPlace.at.y}: ${JSON.stringify(rise)}`);
    assert.ok(atValley.aim && atValley.aim.inside,
      `the valley place projects to ${JSON.stringify(atValley.aim)}`);
    assert.ok(atValley.reading, 'the strip said nothing on the valley');
    const want = Math.min(atDrain + most * slope, cap);
    const said = asInches(atValley.reading['Insulation here']);
    assert.ok(said !== null, `the strip reads "${atValley.reading['Insulation here']}" on the valley`);
    assert.ok(Math.abs(said - want) <= TOLERANCE_INCHES,
      `the two planes tie at ${want} in and the strip reads ${said} in`);
  });

  // ── the legend: the high side, and the ground with no fall ───────────────
  check(`the legend's high side is ${highSide.toFixed(2)} in, which is what the layout reaches`, () => {
    assert.ok(first.ticks.length >= 2, `the ramp has ${first.ticks.length} figures on it`);
    const said = asInches(first.ticks[first.ticks.length - 1]);
    assert.ok(said !== null, `the last figure on the ramp reads "${first.ticks[first.ticks.length - 1]}"`);
    assert.ok(Math.abs(said - highSide) <= 1 / 16,
      `the ramp says the roof gets to ${said} in and the layout reaches ${highSide} in`);
  });

  check('the no-fall ground is the plateau the cap makes, or the key does not claim one', () => {
    const said = /([\d,]+) SF with no fall/.exec(first.legend);
    if (plateau < 1) {
      assert.equal(said, null, `the key claims "${said && said[0]}" where the cap does not bind`);
      return;
    }
    assert.ok(said, `the key says nothing about ${Math.round(plateau)} SF of flat roof: "${first.legend}"`);
    const shown = Number(said[1].replace(/,/g, ''));
    // The lattice is a quarter foot, so it is short or over by the cells the
    // plateau's edge cuts — a couple of square feet on a roof this size.
    assert.ok(Math.abs(shown - plateau) <= Math.max(4, plateau * 0.05),
      `the key says ${shown} SF with no fall and the lattice measures ${plateau} SF`);
  });

  console.log(`  REPORT  the layout, off the job: ${onField.length} regions · axes along ${axisAlong.x.toFixed(3)}, ${axisAlong.y.toFixed(3)}`
    + ` · cap ${Number.isFinite(cap) ? `${cap} in` : 'none'} · high side ${highSide.toFixed(3)} in · no fall ${plateau.toFixed(1)} SF`);
  console.log(`  REPORT  the strip: corner → ${atCorner.reading ? atCorner.reading['Insulation here'] : 'nothing'}`
    + ` · valley → ${atValley.reading ? atValley.reading['Insulation here'] : 'nothing'}`);

  // ── the second picture: the fall, stretched enough to read ───────────────
  const stretched = await pressChip(session, 'vert ×3');
  const after = JSON.parse(await modelNow(session));
  check('the vertical chip stretches the picture and not the roof', () => {
    assert.ok(stretched, 'there is no vert chip on the rail to press');
    assert.equal(after.view.vert, 3, `the vertical is stretched ${after.view.vert} times`);
    assert.equal(after.ticks.join(' '), first.ticks.join(' '),
      `the ramp read "${first.ticks.join(' ')}" and now reads "${after.ticks.join(' ')}"`);
  });
  await shot(session, 'section-taper-vert3');

  const errors = await errorsSoFar(session);
  check('and nothing in the window complained on the way through', () => {
    assert.deepEqual(errors, [], `the window logged ${JSON.stringify(errors)}`);
  });

  console.log(`\n  ${results.filter((r) => r === 'PASS').length}/${results.length}`);
  console.log(`  pictures: evidence/section-taper-frame2-${COMMIT}.png · evidence/section-taper-vert3-${COMMIT}.png`);
  console.log('  the pair to look at it beside: evidence/section-core-model-<C0>.png — it is meant to differ');
} finally {
  await pointerRelease(session);
  await app.close();
}

process.exit(results.includes('FAIL') ? 1 : 0);
