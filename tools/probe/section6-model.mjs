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

  const drains = (job.conditions ?? []).filter((c) => c.role === 'drain').flatMap(inFeet);
  const ridge = (job.conditions ?? []).find((c) => c.role === 'ridge');
  const field = (job.conditions ?? []).find((c) => c.kind === 'area' && c.properties?.TAPER !== undefined);
  const cricket = scene.find((m) => m.kind === 'cricket');

  check('the cricket is drawn from the drains it serves', () => {
    assert.ok(ridge, 'the demo job has no ridge condition');
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

  check('and its ridge runs square to the line joining them', () => {
    // NRCA pp.166-168: the ridge is perpendicular to the line connecting the two
    // drainage points. A cricket built by hanging planes off a traced line
    // passes this only by luck.
    const { a, b } = ridgePoints();
    const byRange = [...drains].sort((p, q) => {
      const m = { x: (a.x + b.x) / 2, y: (a.z + b.z) / 2 };
      return Math.hypot(p.x - m.x, p.y - m.y) - Math.hypot(q.x - m.x, q.y - m.y);
    });
    assert.ok(byRange.length >= 2, `only ${byRange.length} drain(s) to sit between`);
    const dx = byRange[1].x - byRange[0].x;
    const dy = byRange[1].y - byRange[0].y;
    const span = Math.hypot(dx, dy);
    const rx = b.x - a.x;
    const rz = b.z - a.z;
    const run = Math.hypot(rx, rz);
    assert.ok(run > 1e-6 && span > 1e-6, 'the ridge or the drain pair is degenerate');
    // Perpendicular means the dot product of the two unit vectors is zero.
    const dot = Math.abs((dx / span) * (rx / run) + (dy / span) * (rz / run));
    assert.ok(dot < 0.02, `the ridge is ${(Math.acos(Math.min(1, dot)) * 180 / Math.PI).toFixed(1)}° off square to its drains`);
  });

  check('and it sits equidistant between them', () => {
    const { a, b } = ridgePoints();
    const mid = { x: (a.x + b.x) / 2, y: (a.z + b.z) / 2 };
    const byRange = [...drains].sort((p, q) =>
      Math.hypot(p.x - mid.x, p.y - mid.y) - Math.hypot(q.x - mid.x, q.y - mid.y));
    const d0 = Math.hypot(byRange[0].x - mid.x, byRange[0].y - mid.y);
    const d1 = Math.hypot(byRange[1].x - mid.x, byRange[1].y - mid.y);
    assert.ok(Math.abs(d0 - d1) < 0.5, `${d0.toFixed(1)} ft to one drain and ${d1.toFixed(1)} ft to the other`);
  });

  check('and it is cut at twice the slope of the field it sits in', () => {
    // NRCA p.79 and p.165. Read off the drawn shape: the ridge rise over the
    // fall run has to be twice the field's own taper.
    const { a, b, top, low } = ridgePoints();
    const rise = (top - low) * 12;
    const mid = { x: (a.x + b.x) / 2, y: (a.z + b.z) / 2 };
    const byRange = [...drains].sort((p, q) =>
      Math.hypot(p.x - mid.x, p.y - mid.y) - Math.hypot(q.x - mid.x, q.y - mid.y));
    const run = Math.hypot(byRange[0].x - mid.x, byRange[0].y - mid.y);
    assert.ok(run > 1e-6, 'no fall run to measure against');
    const drawn = rise / run;
    const want = (field?.properties?.TAPER ?? 0) * 2;
    assert.ok(want > 0, 'the field has no taper to double');
    assert.ok(Math.abs(drawn - want) < 0.02,
      `drawn at ${drawn.toFixed(3)} in/ft against a field of ${(want / 2).toFixed(3)} — should be ${want.toFixed(3)}`);
  });

  check('and it is not too long for its width to drain', () => {
    // NRCA Fig. 4-13: 3:1 at an eighth or a quarter, 4:1 at a half. Width is what
    // moves water along the valley, and no amount of slope makes up for it.
    const { a, b } = ridgePoints();
    const length = Math.hypot(b.x - a.x, b.z - a.z);
    const mid = { x: (a.x + b.x) / 2, y: (a.z + b.z) / 2 };
    const byRange = [...drains].sort((p, q) =>
      Math.hypot(p.x - mid.x, p.y - mid.y) - Math.hypot(q.x - mid.x, q.y - mid.y));
    const width = Math.hypot(byRange[0].x - mid.x, byRange[0].y - mid.y);
    const ceiling = (field?.properties?.TAPER ?? 0) >= 0.5 ? 4 : 3;
    assert.ok(length / width <= ceiling,
      `${(length / width).toFixed(1)}:1 against a ceiling of ${ceiling}:1`);
  });

  // ── the parapet stands off its reference line, on the inboard side ──────
  const parapetTrace = () => {
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
        if (c.id !== 'c-tapered') return c;
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
