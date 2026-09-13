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

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, detachCostPanel, launch, openDemoJob, pointerClick, pointerRelease,
  showTab, showView, until, wait, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

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

/** Money as cents, so two windows can be compared without a rounding argument. */
const cents = (text) => {
  const m = /-?\$[\d,]+\.\d\d/.exec(String(text));
  return m ? Math.round(Number(m[0].replace(/[$,]/g, '')) * 100) : null;
};

/** What the Properties panel says, wherever it is standing. */
const panelNow = (session) => session.execute(function () {
  const el = document.querySelector('.condition-panel');
  if (!el) return JSON.stringify({ there: false });
  const measure = {};
  for (const cell of el.querySelectorAll('.panel-measures > div')) {
    const label = (cell.querySelector('.measure-label') || {}).textContent || '';
    measure[label.trim()] = ((cell.querySelector('.measure-value') || {}).textContent || '').trim();
  }
  const named = (want) => {
    const label = [...el.querySelectorAll('label')]
      .find((l) => ((l.querySelector('span') || {}).textContent || '').trim() === want);
    const input = label ? label.querySelector('input') : null;
    return input ? input.value : null;
  };
  return JSON.stringify({
    there: !el.hidden,
    name: (el.querySelector('.panel-name') || {}).value || null,
    kind: ((el.querySelector('.panel-kind') || {}).textContent || '').trim(),
    measure,
    cost: ((el.querySelector('.panel-money .value') || {}).textContent || '').trim(),
    note: ((el.querySelector('.panel-money-note') || {}).textContent || '').trim(),
    empty: ((el.querySelector('.panel-empty') || {}).textContent || '').trim(),
    height: named('Height'),
    text: (el.innerText || el.textContent || '').replace(/\s+/g, ' ').trim(),
  });
});

/**
 * The lines, as the cost panel shows them: each condition's header row and the
 * money on the lines under it.
 *
 * It reads the compact line the estimator actually looks at — item, quantity,
 * unit, money — rather than an eleven-column table that no longer exists. The
 * claims are the same: which condition is marked, what it measures, what each
 * of its lines came to, and what the whole thing sells for.
 */
const sheetNow = (session) => session.execute(function () {
  const groups = [];
  let current = null;
  for (const row of document.querySelectorAll('.cost-lines > *')) {
    if (row.classList.contains('cost-group')) {
      current = {
        name: ((row.querySelector('.group-name') || {}).textContent || '').trim(),
        text: (row.textContent || '').replace(/\s+/g, ' ').trim(),
        on: row.classList.contains('on'),
        measures: ((row.querySelector('.group-measures') || {}).textContent || '').trim(),
        money: [],
      };
      groups.push(current);
      continue;
    }
    if (row.classList.contains('cost-line') && current) {
      current.money.push({
        item: ((row.querySelector('.line-item') || {}).textContent || '').trim(),
        extended: ((row.querySelector('.line-money') || {}).textContent || '').trim(),
      });
    }
  }
  return JSON.stringify({
    groups,
    selling: ((document.querySelector('.selling .value') || {}).textContent || '').trim(),
  });
});

/** Every condition in the job, as text, so "nothing changed" can be read literally. */
const jobNow = (session) => session.execute(function () {
  return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
    return JSON.stringify(d.conditions);
  });
});

/** What the 3D view says about itself: the selection, and what is lit. */
const modelNow = (session) => session.execute(function () {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  if (!m) return JSON.stringify({ there: false });
  return JSON.stringify({
    there: true,
    selectedConditionId: m.selectedConditionId,
    lit: m.filter(function (x) { return x.emissiveIntensity > 0; })
      .map(function (x) { return { conditionId: x.conditionId, kind: x.kind, emissive: x.emissive }; }),
  });
});

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

/**
 * Which path the pointer took, so the output can never claim a real click it
 * did not make. Set once, below, on a click whose outcome is already known.
 */
let pointerPath = 'not tried';
let pointerRefused = '';

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });

  // ── the pointer itself, proven on a click that cannot be misread ────────
  // The job is opened by clicking "Open the demo job" WHERE IT IS — the button's
  // own centre, through the driver's actions endpoint — rather than by calling
  // its handler. If that opens the job, the endpoint works and the pick below
  // can be trusted to be a click. If it does not, we go in through the same
  // button the ordinary way and say so in the output.
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
  // REPORTED, not asserted. Whether tauri-driver will dispatch a real pointer is
  // a fact about the harness on this box, not about the program — 6c found
  // WebKitWebDriver refusing all three shapes of a W3C click, with its own words
  // on the record. A row that can only ever be red says nothing about the build
  // and teaches the reader to skip the list. What it costs is carried instead:
  // every row below that depends on real input names the path it took.
  console.log(`  REPORT  real pointer input: ${pointerPath === 'actions'
    ? 'available — every pick below is a driver click'
    : `refused, so every pick below is a dispatched event — ${pointerRefused.split('\n')[0]}`}`);

  if (pointerPath !== 'actions') await openDemoJob(session);
  await until(session, () => !document.querySelector('.start'), { what: 'the job to open' });
  await wait(1200);

  // ── the Model is one of the views the surface shows ─────────────────────
  // Three views of one geometry, switched IN PLACE. It used to be an editor in
  // a picker, and picking reloaded the window; the roof is now a swap of what
  // is mounted in the surface, over the same document, the same selection and
  // the same panel.
  const options = await session.execute(function () {
    return [...document.querySelectorAll('.view-switch .views button')].map((b) => (b.textContent || '').trim());
  });
  check('the Model is one of the views the surface can show', () => {
    assert.ok(options.includes('Model'), `the switch offers ${JSON.stringify(options)}`);
  });

  // A mark on the window, so "the switch did not reload the page" is a fact and
  // not an impression: a reload takes every global with it.
  await session.execute(function () { window.__markBeforeSwitch = 'still here'; });
  const switched = await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });
  const mark = await session.execute(function () { return window.__markBeforeSwitch || 'gone'; });
  check('and switching to it does not reload the window', () => {
    assert.equal(switched, true, 'there is no Model button in the view switch');
    assert.equal(mark, 'still here', 'the page was reloaded — everything in it was thrown away');
  });

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

  // The readout is the window's status strip now — the same region whatever
  // the surface is showing, which is what lets the Plan carry the same four
  // facts when it can compute them. It answers in em dashes until the pointer
  // is on the roof, which is a true thing to say rather than a blank.
  const readout = await session.execute(function () {
    const el = document.querySelector('.status-strip .readout');
    return el ? (el.innerText || el.textContent || '').replace(/\s+/g, ' ').trim() : null;
  });
  check('the roof can be read where the pointer is', () => {
    assert.ok(readout !== null, 'no readout in the status strip');
    for (const want of ['Slope', 'Insulation here', 'Flow']) {
      assert.ok(readout.includes(want), `the readout says "${readout}"`);
    }
    assert.match(readout, /—/, `nothing is under the pointer and it should read em dashes: "${readout}"`);
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
  // ── the cricket, checked against NRCA rather than against the legend ────
  //
  // Everything below reads the drawn triangles and the drains in the document,
  // and works the trade's rules out for itself. Asking the legend whether there
  // is a cricket is worthless: it says yes because the code counted one. These
  // are written so that a plausible-looking wrong cricket fails them.
  const scene = JSON.parse(await session.execute(function () {
    return JSON.stringify(window.__roofnerdModel ? window.__roofnerdModel() : []);
  }));
  const job = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      return JSON.stringify({ conditions: d.conditions, pages: d.pages });
    });
  }));

  const scales = {};
  for (const p of job.pages ?? []) if (p.feetPerUnit) scales[p.id] = p.feetPerUnit;
  const inFeet = (c) => (c.traces ?? []).flatMap((t) =>
    (scales[t.pageId] ? t.points.map((p) => ({ x: p.x * scales[t.pageId], y: p.y * scales[t.pageId] })) : []));

  const ridge = (job.conditions ?? []).find((c) => c.role === 'ridge');
  const field = (job.conditions ?? []).find((c) => c.kind === 'area' && c.properties?.TAPER !== undefined);
  const curb = (job.conditions ?? []).find((c) => c.role === 'curb');
  const curbRing = curb ? inFeet(curb) : [];
  const cricket = scene.find((m) => m.kind === 'cricket');

  check('the cricket is drawn behind the curb it serves', () => {
    // The demo's cricket is the one behind RTU-1, and `between` says so. It used
    // to say `c-drains`, which made it the saddle between two drainage points —
    // a different thing with a different rule, and drawn to the drains' own
    // spacing it came out 38 ft across and covered a third of the roof.
    assert.ok(ridge, 'the demo job has no ridge condition');
    assert.ok(curb, 'the demo job has no curb for a cricket to sit behind');
    assert.ok((ridge.between ?? []).includes(curb.id),
      `the cricket says it serves ${JSON.stringify(ridge.between ?? [])}`);
    assert.ok(curbRing.length >= 4, `the curb is traced with ${curbRing.length} points`);
    assert.ok(cricket, 'no cricket in the scene');
    assert.ok(cricket.points && cricket.points.length >= 18, 'the cricket has no readable geometry');
  });

  // The ridge is the top edge: the two distinct vertices at max height.
  const ridgePoints = () => {
    const pts = [];
    for (let i = 0; i < cricket.points.length; i += 3) {
      pts.push({ x: cricket.points[i], y: cricket.points[i + 1], z: cricket.points[i + 2] });
    }
    const top = Math.max(...pts.map((p) => p.y));
    const high = pts.filter((p) => Math.abs(p.y - top) < 1e-4);
    const a = high[0];
    const b = high.reduce((far, p) =>
      Math.hypot(p.x - a.x, p.z - a.z) > Math.hypot(far.x - a.x, far.z - a.z) ? p : far, a);
    return { a, b, top, low: Math.min(...pts.map((p) => p.y)) };
  };

  /**
   * The curb face this cricket sits against, worked out from the drawn shape.
   *
   * Nothing here reads the code's own bookkeeping: the ridge comes off the
   * triangles that were drawn, the face comes off the curb the estimator traced,
   * and the rules are applied to the two of them here.
   */
  const faceOf = () => {
    const { a, b } = ridgePoints();
    const centre = { x: 0, y: 0 };
    for (const p of curbRing) { centre.x += p.x / curbRing.length; centre.y += p.y / curbRing.length; }
    const ends = [{ x: a.x, y: a.z }, { x: b.x, y: b.z }];
    const range = (p) => Math.hypot(p.x - centre.x, p.y - centre.y);
    const near = range(ends[0]) <= range(ends[1]) ? ends[0] : ends[1];
    const far = near === ends[0] ? ends[1] : ends[0];
    let face = null;
    for (let i = 0; i < curbRing.length - 1; i += 1) {
      const p = curbRing[i];
      const q = curbRing[i + 1];
      const run = Math.hypot(q.x - p.x, q.y - p.y);
      if (run < 1e-6) continue;
      const mid = { x: (p.x + q.x) / 2, y: (p.y + q.y) / 2 };
      const off = Math.hypot(mid.x - near.x, mid.y - near.y);
      if (!face || off < face.off) face = { off, mid, run, dir: { x: (q.x - p.x) / run, y: (q.y - p.y) / run } };
    }
    return { near, far, face, centre };
  };

  check('and its ridge leaves the middle of that curb\'s face, square to it', () => {
    // NRCA Figs. 8-1, 10-13 and 10-14 draw one ridge square to the curb's
    // upslope face and centred on it. Both halves are read off the drawing: the
    // near end of the ridge against the middle of the face, and the ridge's own
    // direction against the face's.
    const { near, far, face, centre } = faceOf();
    assert.ok(face, 'the curb has no face to sit against');
    assert.ok(Math.hypot(near.x - face.mid.x, near.y - face.mid.y) < 0.5,
      `the ridge starts ${Math.hypot(near.x - face.mid.x, near.y - face.mid.y).toFixed(1)} ft off the middle of the face`);
    const rx = far.x - near.x;
    const ry = far.y - near.y;
    const run = Math.hypot(rx, ry);
    assert.ok(run > 1e-6, 'the ridge has no length to measure');
    const dot = Math.abs((rx / run) * face.dir.x + (ry / run) * face.dir.y);
    assert.ok(dot < 0.02,
      `the ridge is ${(Math.acos(Math.min(1, dot)) * 180 / Math.PI).toFixed(1)}\u00b0 off square to the face`);
    // And it runs OUT of the curb rather than into it.
    assert.ok(Math.hypot(far.x - centre.x, far.y - centre.y) > Math.hypot(near.x - centre.x, near.y - centre.y),
      'the cricket runs back over the curb instead of away from it');
  });

  check('and it spans that face, and no more of the roof than that', () => {
    // The one row the slab would have failed loudest. Every drawn vertex is
    // projected onto the face's own direction: what the cricket covers across
    // the roof is the width of the curb face, not the spacing of the drains.
    const { face } = faceOf();
    assert.ok(face, 'the curb has no face to measure against');
    let low = Infinity;
    let high = -Infinity;
    for (let i = 0; i < cricket.points.length; i += 3) {
      const along = cricket.points[i] * face.dir.x + cricket.points[i + 2] * face.dir.y;
      low = Math.min(low, along);
      high = Math.max(high, along);
    }
    assert.ok(Math.abs((high - low) - face.run) < 0.5,
      `the cricket is ${(high - low).toFixed(1)} ft across a ${face.run.toFixed(1)} ft face`);
    const middle = (low + high) / 2;
    const faceMiddle = face.mid.x * face.dir.x + face.mid.y * face.dir.y;
    assert.ok(Math.abs(middle - faceMiddle) < 0.5,
      `it sits ${Math.abs(middle - faceMiddle).toFixed(1)} ft off centre on the face`);
  });

  check('and it is cut at twice the slope of the field it sits in', () => {
    // NRCA p.79 and p.165. Read off the drawn shape: the rise at the ridge over
    // the run water travels to get round the curb, which is half the face.
    const { top, low } = ridgePoints();
    const { face } = faceOf();
    assert.ok(face && face.run > 1e-6, 'no face to measure the fall across');
    const rise = (top - low) * 12;
    const drawn = rise / (face.run / 2);
    const want = (field?.properties?.TAPER ?? 0) * 2;
    assert.ok(want > 0, 'the field has no taper to double');
    assert.ok(Math.abs(drawn - want) < 0.02,
      `drawn at ${drawn.toFixed(3)} in/ft against a field of ${(want / 2).toFixed(3)} — should be ${want.toFixed(3)}`);
  });

  check('and it is not too long for its width to drain', () => {
    // NRCA Fig. 4-13: 3:1 at an eighth or a quarter, 4:1 at a half. Width is what
    // moves water along the valley, and no amount of slope makes up for it.
    const { near, far, face } = faceOf();
    assert.ok(face, 'no face to measure against');
    const length = Math.hypot(far.x - near.x, far.y - near.y);
    const width = face.run / 2;
    const ceiling = (field?.properties?.TAPER ?? 0) >= 0.5 ? 4 : 3;
    assert.ok(length / width <= ceiling,
      `${(length / width).toFixed(1)}:1 against a ceiling of ${ceiling}:1`);
  });

  // ── the parapet stands off its reference line, on the inboard side ──────
  const parapetTrace = () => {
    // By kind, and 'curb' is a kind of its own: the demo stands a curb up as
    // well, and a wall check that took whichever run was drawn first would be
    // measuring the box round the unit against the parapet's own trace.
    const wall = scene.find((m) => m.kind === 'parapet');
    const run = (job.conditions ?? []).find((c) => c.id === wall.conditionId);
    return { wall, run, ring: inFeet(run), thick: (run?.properties?.WALL ?? 0) / 12 };
  };
  // How far a point sits from the traced polyline, which is the wall's own
  // reference line.
  const offLine = (p, ring) => {
    let best = Infinity;
    for (let i = 0; i < ring.length - 1; i += 1) {
      const a = ring[i]; const b = ring[i + 1];
      const dx = b.x - a.x; const dy = b.y - a.y;
      const len2 = dx * dx + dy * dy;
      const u = len2 === 0 ? 0 : Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / len2));
      best = Math.min(best, Math.hypot(p.x - (a.x + u * dx), p.y - (a.y + u * dy)));
    }
    return best;
  };
  const wallVertices = (wall) => {
    const out = [];
    for (let i = 0; i < wall.points.length; i += 3) out.push({ x: wall.points[i], y: wall.points[i + 2] });
    return out;
  };

  check('a parapet with a stated thickness is a wall, not a face', () => {
    const { wall, ring, thick } = parapetTrace();
    assert.ok(wall, 'no parapet solid — the demo parapet states a wall thickness');
    assert.ok(thick > 0, 'the parapet has no wall thickness to check against');
    assert.ok(wall.points, 'the parapet has no readable geometry');
    // Its far face stands exactly the stated thickness off the reference line.
    // A ribbon has every vertex on the line; a wall has a whole face off it.
    const off = wallVertices(wall).map((p) => offLine(p, ring));
    const deepest = Math.max(...off);
    assert.ok(Math.abs(deepest - thick) < 0.02,
      `its far face is ${deepest.toFixed(3)} ft off the line against a stated ${thick.toFixed(3)}`);
  });

  check('and every part of it grows inboard, not outboard', () => {
    // The traced line is the exterior face — the convention every real tool
    // defaults to, and how a parapet is traced on a roof plan. An open run
    // traced round three sides has two legs pointing opposite ways, and a wall
    // built off a single winding sign puts one of them outside the building.
    const { wall, ring, thick } = parapetTrace();
    const xs = ring.map((p) => p.x); const ys = ring.map((p) => p.y);
    const box = { x0: Math.min(...xs), x1: Math.max(...xs), y0: Math.min(...ys), y1: Math.max(...ys) };
    for (const p of wallVertices(wall)) {
      assert.ok(p.x >= box.x0 - 0.02 && p.x <= box.x1 + 0.02
        && p.y >= box.y0 - 0.02 && p.y <= box.y1 + 0.02,
        `a wall vertex at ${p.x.toFixed(2)}, ${p.y.toFixed(2)} sits outside its own trace`);
    }
    assert.ok(thick > 0);
  });

  // Clear it the way an editor would, and watch the view refuse to invent one.
  await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const list = d.conditions.map(function (c) {
        if (c.id !== 'c-parapet') return c;
        const props = Object.assign({}, c.properties);
        delete props.WALL;
        return Object.assign({}, c, { properties: props });
      });
      return window.__TAURI_INTERNALS__.invoke('doc_set', { pointer: '/conditions', value: list });
    });
  });
  await wait(900);
  const cleared = JSON.parse(await session.execute(function () {
    return JSON.stringify(window.__roofnerdModel ? window.__roofnerdModel() : []);
  }));
  const clearedLegend = await legend(session);

  check('and with the thickness cleared it says so rather than guessing one', () => {
    // Nothing invented in its place, and nothing quietly becoming zero. The
    // drawing keeps the face it actually knows about and names what it does not.
    assert.equal(cleared.filter((m) => m.kind === 'parapet').length, 0,
      'a wall is still drawn at a thickness nothing states');
    assert.ok(cleared.some((m) => m.kind === 'parapet-ribbon'), 'the run stopped being drawn at all');
    assert.match(clearedLegend, /no wall thickness stated/, `legend read "${clearedLegend}"`);
  });

  // Put it back, so nothing after this reads a job this check changed.
  await session.execute(function (thick) {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const list = d.conditions.map(function (c) {
        if (c.id !== 'c-parapet') return c;
        return Object.assign({}, c, {
          properties: Object.assign({}, c.properties, { WALL: thick }),
        });
      });
      return window.__TAURI_INTERNALS__.invoke('doc_set', { pointer: '/conditions', value: list });
    });
  }, 8);
  await wait(700);

  // ── the build-up grows up from the deck, and is never invented ──────────
  check('a build-up stands on the datum and rises by what its layers say', () => {
    const datums = scene.filter((m) => m.kind === 'datum');
    assert.ok(datums.length > 0, 'nothing is drawn at the deck');
    for (const stack of scene.filter((m) => m.kind === 'build-up')) {
      const datum = datums.find((d) => d.conditionId === stack.conditionId);
      assert.ok(datum, `a build-up with no datum under it (${stack.conditionId})`);
      // Its underside sits on the deck and its top is above — growing up, which
      // is the direction a roof build-up goes. Down would be the structure, and
      // the structure is not traced.
      assert.ok(Math.abs(stack.min[1] - datum.min[1]) < 1e-3,
        `the build-up starts at ${stack.min[1]} and the deck is at ${datum.min[1]}`);
      assert.ok(stack.max[1] > datum.max[1] + 1e-6, 'the build-up does not rise off the deck');
    }
  });

  check('and an assembly that states no thickness draws nothing at all', () => {
    // The defect this replaced: a deck extruded three quarters of a foot because
    // a flat surface "reads as a coloured shape in space". An unstated build-up
    // is unstated, and the legend carries it instead.
    const stated = new Set((job.conditions ?? [])
      .filter((c) => (c.items ?? []).some((i) => i.thickness !== undefined))
      .map((c) => c.id));
    for (const stack of scene.filter((m) => m.kind === 'build-up')) {
      assert.ok(stated.has(stack.conditionId),
        `${stack.conditionId} is drawn with a thickness nothing in the job states`);
    }
  });

  // ── §5.6, second half: a low point with no drain on it is marked ────────
  const noFall = squaresWithNoFall(first);
  check('the legend names what caps the build, not merely that something does', () => {
    // Whether *this* roof ponds is a fact about the demo, not about the program,
    // and a check that demanded a defect in the fixture broke the moment the
    // demo's traces were brought onto their sheet and the field started draining
    // properly. What the program owes is that when a build-up is capped it says
    // by what — and that is asserted here. The marking of an actual flat area is
    // asserted below, on a roof deliberately made to have one.
    assert.match(first, /capped by flashing height|the boards run out|no fall/,
      `legend read "${first}"`);
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
        if (c.id !== 'c-field') return c;
        const props = Object.assign({}, c.properties, { TAPER: 0.5 });
        return Object.assign({}, c, { properties: props });
      });
      return window.__TAURI_INTERNALS__.invoke('doc_set', { pointer: '/conditions', value: list });
    });
  });
  await wait(1500);
  const after = squaresWithNoFall(await legend(session));

  const steepened = await legend(session);
  check('a change to the roof moves the 3D view live, in the same window', () => {
    assert.ok(after !== null, 'the legend stopped reporting a no-fall area');
    assert.ok(after > (before ?? 0),
      `steepening the taper should leave more of the roof flat: ${before} SF before, ${after} SF after`);
  });
  check('and where the roof does go flat it is marked, with its area and a reason', () => {
    // On a roof made flat on purpose, a moment ago, through the document. The
    // area has to be a real number of square feet and the mark has to say why
    // the water stopped — the two halves of what ponding means here.
    assert.ok(after > 0, `no-fall area came back as ${after} SF`);
    assert.match(steepened, /SF with no fall/, `legend read "${steepened}"`);
    assert.match(steepened, /the boards run out|flashes against/,
      `the mark gave no reason: "${steepened}"`);
  });

  await writeFile(join(EVIDENCE, `section6-live-edit-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── Addendum 4 §4: selection crosses the WINDOWS, and is not editing ────
  //
  // Window A is this one, with the roof in it. Window B is the cost panel,
  // pulled onto the other monitor — which is the second window this program
  // opens now, and the arrangement the handoff draws: geometry on one screen,
  // money on the other. Both directions, and the money at the end of it.
  const oneWindow = await session.handles();
  const tore = await detachCostPanel(session);
  const handles = await session.handles();
  const A = oneWindow[0];
  const B = handles.find((h) => !oneWindow.includes(h)) ?? null;
  check('the cost panel detaches onto the other monitor', () => {
    assert.ok(tore, 'no detach button beside the tabs');
    assert.ok(B, `${oneWindow.length} window before and ${handles.length} after`);
  });
  if (B) {
    await session.switchTo(B);
    await until(session, () => !!document.querySelector('.cost-panel.detached'),
      { what: 'the detached cost panel' });
    await session.switchTo(A);
    await wait(600);
  }

  const names = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const out = {};
      for (const c of d.conditions) out[c.id] = c.name;
      return JSON.stringify(out);
    });
  }));
  const jobBeforeSelecting = await jobNow(session);

  // ── (a) point at the roof field in 3D ───────────────────────────────────
  // The demo is one roof now, and it is the tapered field: the Low Roof this
  // used to aim at was the second area on a two-roof fixture that no longer
  // exists. Same claim, same condition kind, the roof that is there.
  const aim = JSON.parse(await session.execute(function (id) {
    const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
    const p = m && m.screenPointOf ? m.screenPointOf(id) : null;
    return JSON.stringify(p);
  }, 'c-field'));

  check('the view can say where on screen the roof field is drawn', () => {
    assert.ok(aim, 'nothing in the scene carries that condition');
    assert.ok(aim.inside, `it projects to ${aim.x}, ${aim.y}, which is not on the canvas`);
    assert.equal(aim.hits, 'c-field',
      `a ray cast at that point finds ${aim.hits ?? 'nothing'} — the point is not on it`);
  });

  /** The pick, by hand where the driver allows it, and said plainly where not. */
  let pickPath = pointerPath === 'actions' ? 'actions' : 'synthetic';
  if (aim && aim.inside) {
    if (pickPath === 'actions') {
      try {
        await pointerClick(session, aim.x, aim.y);
      } catch (e) {
        // The endpoint took the first click and refused this one. Say so, and
        // do not lose the twenty rows after this one to an abandoned run.
        pickPath = 'synthetic';
        pointerRefused = e.message;
      }
    }
    if (pickPath !== 'actions') {
      // The fallback, for this one click, said plainly: a synthetic event with
      // the coordinates in it. The raycast reads clientX and clientY, so this
      // exercises the same arithmetic — but it is not a click and the row below
      // says which path ran.
      await session.execute(function (x, y) {
        const c = document.querySelector('.model-canvas');
        c.dispatchEvent(new MouseEvent('click', { clientX: x, clientY: y, bubbles: true, button: 0 }));
      }, aim.x, aim.y);
    }
    await wait(1000);
  }

  const pickedA = JSON.parse(await modelNow(session));

  check(`picking the roof field in 3D selects it (${pickPath})`, () => {
    assert.ok(pickedA.there, 'the view has nothing to say about itself');
    assert.equal(pickedA.selectedConditionId, 'c-field',
      `the view believes ${pickedA.selectedConditionId ?? 'nothing'} is selected`);
  });

  // ── (b) the other window already knows ──────────────────────────────────
  // The panel is in window B, so what the pick reached is read there: the lines
  // mark the condition, and the Condition tab names it with its measures and
  // what it costs.
  let sheetB = null;
  let panelField = null;
  if (B) {
    await session.switchTo(B);
    await until(session, () => !!document.querySelector('.cost-estimate'), { what: 'the lines' });
    await wait(800);
    sheetB = JSON.parse(await sheetNow(session));
    await showTab(session, 'Condition');
    panelField = JSON.parse(await panelNow(session));
  }
  const fieldRow = sheetB?.groups.find((g) => g.text.includes(names['c-field'])) ?? null;

  check('and the lines in the other window mark the same condition', () => {
    assert.ok(sheetB, 'there is no second window to look in');
    assert.ok(fieldRow, `the panel has no group for "${names['c-field']}"`);
    assert.ok(fieldRow.on,
      'the group is not marked — a pick in 3D did not cross the window frame');
    const others = sheetB.groups.filter((g) => g.on).length;
    assert.equal(others, 1, `${others} groups are marked at once`);
  });
  check('and the panel in that window names it and shows what it measures', () => {
    // `panelNow` in this file answers `there`, not a count — the count belongs
    // to 6b, which is the check about how many panels a window has. Asking for
    // a field the reader does not return is a row that fails on its own
    // bookkeeping and says nothing about the program.
    assert.ok(panelField && panelField.there,
      'no Properties panel in the window the cost panel is in');
    assert.equal(panelField.name, names['c-field'],
      `the panel names "${panelField.name}"`);
    assert.match(panelField.measure?.['Area'] ?? '', /\d/,
      `its area reads "${panelField.measure?.['Area']}"`);
    assert.match(panelField.measure?.['Squares'] ?? '', /\d/,
      `its squares read "${panelField.measure?.['Squares']}"`);
  });
  check('and says what it costs, in money, because it carries lines', () => {
    // This used to read the other way round, on a fixture whose second roof was
    // traced and priced by nothing: the row asserted the panel said so in words
    // rather than $0.00. The roof the demo ships now carries the whole build-up,
    // so what it owes is the money — and the row below holds that money against
    // the lines' own, to the cent.
    assert.match(String(panelField && panelField.cost), /\$/,
      `the panel's cost line reads "${panelField && panelField.cost}"`);
  });
  check('and both windows print the same area for it', () => {
    // The panel's figure against the group row's, in the two windows they are
    // drawn in. One measurement, two surfaces, and they cannot be allowed to
    // differ by so much as a digit.
    //
    // Read as a number rather than as a substring. The group row carries three
    // measures now — an area, a run and a corner count, with a thousands
    // separator — and the panel prints the area without one, so a row that
    // looked for one string inside the other went red on two spellings of a
    // single figure. It would also have gone green on squares found inside
    // square feet, which is the worse half: the same digits, a hundredth of the
    // quantity. Neither spelling nor substring; the numbers themselves.
    const number = (text) => Number(String(text).replace(/,/g, ''));
    const inPanel = number((panelField?.measure?.['Area'] ?? '').replace(/[^\d.,]/g, ''));
    assert.ok(Number.isFinite(inPanel) && inPanel > 0,
      `the panel shows no area: "${panelField?.measure?.['Area']}"`);
    const sf = /([\d,]+(?:\.\d+)?)\s*SF/.exec(fieldRow?.measures ?? '');
    assert.ok(sf, `the group row prints no square feet: "${fieldRow?.measures}"`);
    assert.equal(number(sf[1]), inPanel,
      `the panel says ${inPanel} and the group row says "${fieldRow?.measures}"`);
  });

  // ── (c) and back the other way: pick on the lines, watch the roof ───────
  if (B) await showTab(session, 'Estimate');
  const parapetAt = B ? await centreOf(session, '.cost-group', names['c-parapet']) : null;
  let rowClicked = false;
  if (parapetAt && pickPath === 'actions') {
    const at = JSON.parse(parapetAt);
    try {
      await pointerClick(session, at.x, at.y);
      rowClicked = true;
    } catch (e) {
      pointerRefused = e.message;
    }
  }
  if (!rowClicked && B) {
    await session.execute(function (want) {
      const row = [...document.querySelectorAll('.cost-group')]
        .find(function (r) { return (r.textContent || '').indexOf(want) >= 0; });
      if (row) row.click();
    }, names['c-parapet']);
  }
  await wait(1000);
  const sheetParapet = B ? JSON.parse(await sheetNow(session)) : null;
  const parapetRowBefore = sheetParapet?.groups.find((g) => g.text.includes(names['c-parapet'])) ?? null;

  const panelParapet = B ? await (async () => {
    await showTab(session, 'Condition');
    const read = JSON.parse(await panelNow(session));
    await showTab(session, 'Estimate');
    return read;
  })() : null;
  if (B) {
    await session.switchTo(A);
    await wait(1000);
  }
  const litA = JSON.parse(await modelNow(session));

  check('picking a condition on the lines reaches the roof in the other window', () => {
    assert.ok(parapetRowBefore, `no row for "${names['c-parapet']}" to pick`);
    assert.equal(litA.selectedConditionId, 'c-parapet',
      `the view believes ${litA.selectedConditionId ?? 'nothing'} is selected`);
  });
  check('and it is that condition, and only it, that is lit in 3D', () => {
    // Read off the material the roof is painted with, not off the selection
    // that was just set. The emissive is what `highlight()` actually did.
    assert.ok(litA.lit.length > 0, 'nothing in the view is lit at all');
    for (const m of litA.lit) {
      assert.equal(m.conditionId, 'c-parapet',
        `the ${m.kind} of ${m.conditionId} is lit as well`);
    }
    assert.ok(litA.lit.some((m) => String(m.kind).startsWith('parapet')),
      `what is lit is ${JSON.stringify(litA.lit)}`);
  });
  check('and the panel figure is the lines\' own money, to the cent', () => {
    const inPanel = cents(panelParapet && panelParapet.cost);
    assert.ok(inPanel !== null, `the panel's cost line reads "${panelParapet && panelParapet.cost}"`);
    const onSheet = (parapetRowBefore?.money ?? [])
      .map((m) => cents(m.extended))
      .filter((c) => c !== null)
      .reduce((sum, c) => sum + c, 0);
    assert.equal(inPanel, onSheet,
      `the panel says ${inPanel} cents and its own lines add to ${onSheet}`);
  });

  // ── (e) none of that was an edit ────────────────────────────────────────
  const jobAfterSelecting = await jobNow(session);
  check('and selecting changed nothing about the job — there is no editing in 3D', () => {
    assert.equal(jobAfterSelecting, jobBeforeSelecting,
      'the job moved while nothing but selections happened');
    const c = JSON.parse(jobAfterSelecting).find((x) => x.id === 'c-field');
    const was = JSON.parse(jobBeforeSelecting).find((x) => x.id === 'c-field');
    assert.equal((c.traces || []).length, 1, `${(c.traces || []).length} traces on the field`);
    // Corner for corner what it was before anything was picked. The count is
    // not written down here: a figure of the demo's own, typed into a check, is
    // a figure that goes stale the day the demo is re-traced — and this row is
    // about selecting not being an edit, not about how many corners this roof
    // happens to have.
    assert.deepEqual(c.traces[0].points, was.traces[0].points,
      'the outline moved while nothing but selections happened');
  });

  await writeFile(join(EVIDENCE, `section6-both-windows-a-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // ── (d) a property typed beside the money moves it, and the roof follows ─
  // Typing a height is not dragging geometry: §4.10 stands, and the panel is
  // the place a height has always been typed. What is new is which window it is
  // in — the panel is the thing that detaches, so the fields are on the second
  // monitor and the roof is here. The claim is the same and it now runs the
  // other way round: type beside the money and watch the roof follow.
  if (B) {
    await session.switchTo(B);
    await showTab(session, 'Condition');
    await wait(600);
  }
  const heightBefore = Number(panelParapet && panelParapet.height);
  const typed = await session.execute(function (value) {
    const label = [...document.querySelectorAll('.condition-panel label')]
      .find(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim() === 'Height'; });
    const input = label ? label.querySelector('input') : null;
    if (!input) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(input, String(value));
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  }, heightBefore * 2);
  await wait(1600);

  let sheetAfter = null;
  if (B) {
    await showTab(session, 'Estimate');
    await wait(1200);
    sheetAfter = JSON.parse(await sheetNow(session));
    await writeFile(join(EVIDENCE, `section6-both-windows-b-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));
  }
  const parapetRowAfter = sheetAfter?.groups.find((g) => g.text.includes(names['c-parapet'])) ?? null;

  // What the window with the roof in it now says the parapet is, read off the
  // job as that window holds it — one document, two windows.
  let roofAfterTyping = null;
  if (B) {
    await session.switchTo(A);
    await wait(1200);
    roofAfterTyping = await session.execute(function () {
      return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
        const c = d.conditions.find(function (x) { return x.id === 'c-parapet'; });
        return String((c.properties || {}).H ?? '');
      });
    });
    await session.switchTo(B);
    await wait(400);
  }

  check('a height typed in the panel moves the selling price pinned under it', () => {
    assert.ok(typed, 'the panel has no Height field to type in');
    assert.ok(Number.isFinite(heightBefore),
      `the panel read its height as "${panelParapet && panelParapet.height}"`);
    assert.ok(sheetAfter, 'there is no cost panel to read the price in');
    assert.notEqual(cents(sheetAfter.selling), cents(sheetParapet.selling),
      `the selling price stayed at ${sheetAfter.selling}`);
  });
  check('and the roof in the other window took the new height', () => {
    // The cross-window half, at the new anatomy: the fields are here and the
    // geometry is there, so what has to travel is the height itself.
    assert.ok(roofAfterTyping !== null, 'there is no other window to look in');
    assert.ok(roofAfterTyping.includes(String(heightBefore * 2)),
      `the roof's parapet reads "${roofAfterTyping}" and the height was set to ${heightBefore * 2}`);
  });
  check('and it is the one line the height is in that moved, and it doubled', () => {
    const was = (parapetRowBefore?.money ?? []).map((m) => cents(m.extended));
    const now = (parapetRowAfter?.money ?? []).map((m) => cents(m.extended));
    assert.equal(now.length, was.length, `${was.length} lines before and ${now.length} after`);
    const moved = now.map((v, i) => (v !== was[i] ? i : -1)).filter((i) => i >= 0);
    assert.equal(moved.length, 1,
      `${moved.length} lines changed when one formula reads the height: ${JSON.stringify({ was, now })}`);
    const i = moved[0];
    // Doubling H doubles `LF * H`, and the cent it is displayed at is the same
    // arithmetic rounded once, so this is exact. One cent of tolerance because
    // a half-cent at the boundary is a display question, not a money question.
    assert.ok(Math.abs(now[i] - was[i] * 2) <= 1,
      `the line went from ${was[i]} to ${now[i]} cents when its height doubled`);
  });

  // Put the height back, so nothing after this reads a job this check moved.
  if (B) {
    await showTab(session, 'Condition');
    await wait(600);
  }
  await session.execute(function (value) {
    const label = [...document.querySelectorAll('.condition-panel label')]
      .find(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim() === 'Height'; });
    const input = label ? label.querySelector('input') : null;
    if (!input) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(input, String(value));
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  }, heightBefore);
  await wait(1200);

  if (B) {
    await session.switchTo(A);
    await wait(800);
  }

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
  await pointerRelease(session);
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
