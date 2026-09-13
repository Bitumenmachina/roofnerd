// ── C5: the roof reads like a drawing, and the cut proves the stack ────────
// C1 laid the field out and C2 turned it into a bill. On screen it was still an
// engineering view: a grey field, a lawn of darts, a text-heavy key, and not one
// word on the roof. The handoff's Frame 2 and Frame 3 are what a roofer was
// promised, and this is the check that they arrived.
//
// What this holds the program to:
//
//   WORDS ON THE ROOF. Each drain carries its own sump depth, the curb carries
//   the cricket behind it, and the flat ground carries its square feet — read off
//   the labels drawn over the Model, and compared against the depth and the area
//   this file works out for itself from `jobs/demo-job`. A label is a number an
//   estimator will act on, so it is held to the solve like any other.
//
//   THE BOARDS, ON THE ROOF. The "boards" chip draws the grid C2 billed: one cell
//   per board position, tinted and lettered. The count is compared against a bill
//   this file builds by sampling the roof on a quarter-foot lattice — an integral,
//   sharing no code and no rounding with the polygon clipper the core uses.
//
//   THE SECTION. A third view, cut through a drain. The layers come out bottom to
//   top in the order the roof is built in — deck, the flat fill by the board, the
//   lettered panel, then whatever the condition's own layers say goes over it —
//   and the dimensions on the drawing equal the numbers the job comes to: the run
//   from the high point to the drain, the depth at each end, the sump, the
//   parapet. Then the cut is moved to another drain and the drawing follows.
//
//   THE PLAN'S OVERLAYS. "layout" draws the board grid with a letter on every
//   board — one letter per board the bill counts. "fastening" is empty and says
//   why, rather than drawing a pattern nobody has solved.
//
//   AND THE MONEY KNOWS. With the cut up, the Estimate tab names the lines the
//   plane passes through.
//
// Front door throughout — the start screen, the demo button, the view switch, the
// chips. Nothing here calls a command to get a job open.
//
// RED BEFORE C5. At C2's head there is no Section view at all (`VIEW_ORDER` is
// plan and model), no board grid in the Model, no labels over it, no overlay on
// the Plan and no `drawing` in the solve. Every row below fails, which is the
// point of writing it first.
//
//   pnpm build:app && node tools/probe/section-section.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, errorsSoFar, launch, openDemoJob, pointerClick, pointerRelease, showTab, showView,
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

// ── what this roof comes to, worked out from what is on disk ───────────────
const { LAYER_FUNCTIONS, capInches } = await import('../../packages/engine/dist/index.js');
const jobFolder = join(ROOT, 'jobs/demo-job');
const read = async (rel) => JSON.parse(
  await readFile(join(rel.startsWith('data/') ? ROOT : jobFolder, rel), 'utf8'));
const conditions = await read('conditions.json');
const pages = await read('pages/pages.json');
const profile = await read('data/profiles/versico-versicore-mp-h.json');

/** Feet to the page unit. A sheet nobody scaled has none, and no trace on it. */
const scales = {};
for (const page of pages) if (page.feetPerUnit) scales[page.id] = page.feetPerUnit;
const feet = (trace) => (scales[trace.pageId]
  ? trace.points.map((p) => ({ x: p.x * scales[trace.pageId], y: p.y * scales[trace.pageId] }))
  : []);

const drainCondition = conditions.find((c) => c.role === 'drain' && !c.hidden);
const drains = [];
for (const t of drainCondition.traces ?? []) drains.push(...feet(t));

const fieldCondition = conditions.find(
  (c) => c.kind === 'area' && !c.hidden && c.properties?.TAPER !== undefined,
);
const ring = feet(fieldCondition.traces[0]);
const atDrain = fieldCondition.properties?.T ?? 0;
const slope = fieldCondition.properties?.TAPER ?? 0;

/** The lowest thing the membrane has to turn up and flash against, in inches. */
let flashesAgainst = Infinity;
for (const c of conditions) {
  if (c.kind !== 'line' || c.hidden || c.role === 'ridge') continue;
  const h = c.properties?.H;
  if (h === undefined || h <= 0) continue;
  flashesAgainst = Math.min(flashesAgainst, h * 12);
}
const perimeter = Number.isFinite(flashesAgainst) ? flashesAgainst : undefined;
const ceiling = capInches(atDrain, fieldCondition.properties?.BOARDS, perimeter).cap;

/**
 * The field's own axes: its longest side, and square to that.
 *
 * The boards are laid to the longest side, so the run that sets a thickness is the
 * greater of the two runs along these. The same expression as `section-core.mjs`,
 * `section-taper.mjs` and `section-bill.mjs`, because it is the same fact about
 * the layout and none of these files is allowed to ask the program for it.
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

/** The manufacturer's chart for this slope: the panels, and the board they stack on. */
const charted = profile.slopes.find((s) => Math.abs(s.slope_in_per_ft - slope) < 1e-9);
assert.ok(charted, `the profile table charts no ${slope} in per foot slope`);
const chart = charted.extended;
const fillStep = Number.parseFloat(chart.fill_in);
const panelFeet = Math.min(...profile.panel_size_ft);

const sumpFeet = drainCondition.properties?.SUMP;
/** How far the sump falls from its own rim to the drain, in inches. */
const sumpDepth = sumpFeet === undefined ? null : (sumpFeet / 2) * slope;

const inside = (p, poly) => {
  let hit = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i, i += 1) {
    const a = poly[i]; const b = poly[j];
    if ((a.y > p.y) !== (b.y > p.y) && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) {
      hit = !hit;
    }
  }
  return hit;
};

const inSump = (p) => sumpFeet !== undefined && drains.some((d) => {
  const a = (p.x - d.x) * axisAlong.x + (p.y - d.y) * axisAlong.y;
  const b = (p.x - d.x) * axisAcross.x + (p.y - d.y) * axisAcross.y;
  return Math.abs(a) < sumpFeet / 2 && Math.abs(b) < sumpFeet / 2;
});

/**
 * The bill and the flat ground, by sampling the roof rather than by clipping it.
 *
 * A quarter-foot lattice over the field's box: every point that is on the roof and
 * not in a sump lands in the cell of the grid its own drain anchors, and a cell
 * with any point in it is a board that gets bought. The same integral answers how
 * much ground the cap holds flat, which is the figure the amber label carries.
 *
 * It shares no code and no rounding with the polygon clipper in the core, which is
 * the whole reason for doing it this way.
 */
const LATTICE_FEET = 0.25;
const box = { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity };
for (const p of ring) {
  box.minX = Math.min(box.minX, p.x); box.maxX = Math.max(box.maxX, p.x);
  box.minY = Math.min(box.minY, p.y); box.maxY = Math.max(box.maxY, p.y);
}

const positions = new Map();
let noFallSquareFeet = 0;
for (let x = box.minX + LATTICE_FEET / 2; x < box.maxX; x += LATTICE_FEET) {
  for (let y = box.minY + LATTICE_FEET / 2; y < box.maxY; y += LATTICE_FEET) {
    const p = { x, y };
    if (!inside(p, ring)) continue;
    let which = 0; let far = Infinity;
    drains.forEach((d, k) => {
      const r = (p.x - d.x) ** 2 + (p.y - d.y) ** 2;
      if (r < far) { far = r; which = k; }
    });
    const d = drains[which];
    const alongRun = (p.x - d.x) * axisAlong.x + (p.y - d.y) * axisAlong.y;
    const acrossRun = (p.x - d.x) * axisAcross.x + (p.y - d.y) * axisAcross.y;
    if (atDrain + Math.max(Math.abs(alongRun), Math.abs(acrossRun)) * slope > ceiling) {
      noFallSquareFeet += LATTICE_FEET * LATTICE_FEET;
    }
    if (inSump(p)) continue;
    const i = Math.floor((alongRun + panelFeet / 2) / panelFeet);
    const j = Math.floor((acrossRun + panelFeet / 2) / panelFeet);
    positions.set(`${which}:${i}:${j}`, Math.max(Math.abs(i), Math.abs(j)));
  }
}

/** The letter and the fill a thickness comes to, the way the chart is printed. */
const stackAt = (thickness) => {
  const fill = fillStep * Math.floor((thickness - atDrain + 1e-9) / fillStep);
  const panel = thickness - fill;
  const found = chart.panels.find(
    (q) => panel >= q.thin_edge_in - 1e-9 && panel < q.thick_edge_in - 1e-9,
  );
  return { fill, panel, letter: found ? found.designation : null, courses: Math.round(fill / fillStep) };
};

const boardsByLetter = new Map();
for (const out of positions.values()) {
  const thickness = Math.min(atDrain + out * panelFeet * slope, ceiling);
  const { letter } = stackAt(thickness);
  assert.ok(letter, `nothing in the chart covers the panel at ${thickness} in`);
  boardsByLetter.set(letter, (boardsByLetter.get(letter) ?? 0) + 1);
}
const boardCount = positions.size;

console.log(`  REPORT  this file's own bill: ${boardCount} board positions`
  + ` — ${[...boardsByLetter].sort().map(([l, n]) => `${l} ${n}`).join(', ')};`
  + ` ${Math.round(noFallSquareFeet)} SF with no fall; sump ${sumpFeet} ft`
  + ` falling ${sumpDepth} in to the drain`);

// ── where each cut runs, and what it ends against ──────────────────────────
//
// The cut runs along the boards — the direction the layout is set out in and the
// direction a tapered section is drawn in — from the drain out to the edge of the
// ground that drain takes. Two directions are on offer and the one that reaches a
// wall wins, because a section detail exists to prove the assembly where the roof
// stops; where neither reaches one, the longer wins and the far end is a ridge.

/**
 * How far a ray from `d` along `u` gets before it leaves the ring.
 *
 * `d + t·u = a + s·e` for each edge, solved with the two-dimensional cross
 * product: `t = ((a−d) × e) / (u × e)` and `s = ((a−d) × u) / (u × e)`. THE
 * SECOND ONE HAD ITS SIGN WRONG and every ray that should have struck the west
 * wall reported that it never struck anything — see the note at `cuts` below.
 */
function toTheRing(d, u) {
  const cross = (px, py, qx, qy) => px * qy - py * qx;
  let best = Infinity;
  for (let i = 0; i < ring.length; i += 1) {
    const a = ring[i]; const b = ring[(i + 1) % ring.length];
    const ex = b.x - a.x; const ey = b.y - a.y;
    const denominator = cross(u.x, u.y, ex, ey);
    if (Math.abs(denominator) < 1e-12) continue;
    const t = cross(a.x - d.x, a.y - d.y, ex, ey) / denominator;
    const s = cross(a.x - d.x, a.y - d.y, u.x, u.y) / denominator;
    if (t > 1e-9 && s >= -1e-9 && s <= 1 + 1e-9) best = Math.min(best, t);
  }
  return best;
}

/** How far it gets before the bisector with another drain takes over. */
function toTheRidge(d, u) {
  let best = Infinity;
  for (const e of drains) {
    if (e === d) continue;
    const vx = e.x - d.x; const vy = e.y - d.y;
    const towards = vx * u.x + vy * u.y;
    if (towards <= 1e-9) continue;
    best = Math.min(best, (vx * vx + vy * vy) / (2 * towards));
  }
  return best;
}

const cuts = drains.map((d, index) => {
  const offers = [{ x: -axisAlong.x, y: -axisAlong.y }, axisAlong].map((u) => {
    const wall = toTheRing(d, u);
    const ridge = toTheRidge(d, u);
    return { u, run: Math.min(wall, ridge), atWall: wall <= ridge + 1e-9 };
  });
  const wallward = offers.filter((o) => o.atWall && Number.isFinite(o.run));
  const from = (wallward.length ? wallward : offers)
    .reduce((a, b) => (b.run > a.run ? b : a));
  const letters = `${String.fromCharCode(65 + index)}–${String.fromCharCode(65 + index)}`;
  return {
    drain: d,
    letters,
    name: `Cut ${letters}`,
    chip: `cut ${letters}`,
    run: from.run,
    atWall: from.atWall,
    // The roof at the far end: the run along the boards, capped.
    highInches: Math.min(atDrain + from.run * slope, ceiling),
  };
});

/**
 * A run that came out infinite is a defect IN THIS FILE, and it says so here
 * rather than being carried into the checks.
 *
 * It was carried once. `toTheRing` had a sign wrong, so two of the three cuts
 * reported that the ray never left the roof; the run came back `Infinity`, the
 * row that walks out board by board along the cut was bounded by it, and the
 * probe ran for nine hundred seconds and took none of its remaining pictures.
 * Nothing is silently zero in this program and nothing is silently infinite
 * either: a figure this file cannot work out stops the run here, before a window
 * is opened, with the arithmetic named.
 */
for (const cut of cuts) {
  assert.ok(Number.isFinite(cut.run) && cut.run > 0,
    `${cut.name} came out ${cut.run} ft long, which is this file's arithmetic and not the program's`);
  assert.ok(Number.isFinite(cut.highInches),
    `${cut.name} is ${cut.highInches} in at its far end`);
}

console.log(`  REPORT  the cuts this job comes to: ${cuts.map((c) =>
  `${c.name} ${c.run.toFixed(2)} ft to ${c.atWall ? 'a wall' : 'a ridge'},`
  + ` ${c.highInches} in at the far end`).join(' · ')}`);

/** The parapet, which is what the first cut ends against. */
const wallCondition = conditions.find(
  (c) => c.kind === 'line' && !c.hidden && c.role !== 'ridge' && (c.properties?.H ?? 0) > 0,
);
const wallInches = (wallCondition?.properties?.H ?? 0) * 12;

/** A length off the drawing: 22'-0", 4'-6", 1/2", 6". */
const asFeet = (text) => {
  const s = String(text ?? '');
  const both = /(-?\d+)['′]\s*-?\s*(\d+)?(?:\s+(\d+)\/(\d+))?["″]?/.exec(s);
  if (both) {
    return Number(both[1])
      + (Number(both[2] ?? 0) + (both[3] ? Number(both[3]) / Number(both[4]) : 0)) / 12;
  }
  const inches = /(?:(\d+)\s+)?(?:(\d+)\/(\d+)|(\d+))\s*["″]/.exec(s);
  if (!inches) return null;
  const whole = Number(inches[1] ?? inches[4] ?? 0);
  const frac = inches[2] ? Number(inches[2]) / Number(inches[3]) : 0;
  return (whole + frac) / 12;
};

// ── what the window says ───────────────────────────────────────────────────

/** The Model's own account of itself: the meshes, the labels, the key. */
const modelNow = (session) => session.execute(function () {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  const kinds = {};
  if (m) for (const mesh of m) kinds[String(mesh.kind)] = (kinds[String(mesh.kind)] || 0) + 1;
  return JSON.stringify({
    there: !!m,
    kinds,
    labels: [...document.querySelectorAll('.model-label')].map(function (e) {
      return {
        kind: e.getAttribute('data-label-kind'),
        text: (e.textContent || '').replace(/\s+/g, ' ').trim(),
      };
    }),
    legend: ((document.querySelector('.model-legend') || {}).innerText || '')
      .replace(/\s+/g, ' ').trim(),
    keyRows: document.querySelectorAll('.model-legend .model-key').length,
    chips: [...document.querySelectorAll('.view-chip')].map(function (b) {
      return { text: (b.textContent || '').trim(), on: b.classList.contains('on') };
    }),
  });
});

/** One chip in the view navigation, pressed the way a hand presses it. */
const pressChip = async (session, label) => {
  const found = await session.execute(function (want) {
    const b = [...document.querySelectorAll('.view-chip')]
      .find(function (e) { return (e.textContent || '').trim() === String(want); });
    if (!b) return false;
    b.click();
    return true;
  }, label);
  await wait(900);
  return found;
};

/** What the drawing on screen is made of, read off the SVG an estimator sees. */
const drawingNow = (session, where) => session.execute(function (sel) {
  const svg = document.querySelector(sel);
  if (!svg) return JSON.stringify({ there: false });
  const marks = [...svg.querySelectorAll('[data-role]')];
  const note = document.querySelector('.drawing-note');
  return JSON.stringify({
    there: true,
    note: note && !note.hidden ? (note.textContent || '').replace(/\s+/g, ' ').trim() : '',
    marks: marks.length,
    roles: marks.map(function (e) { return e.getAttribute('data-role'); }),
    layers: [...svg.querySelectorAll('[data-layer]')].map(function (e) {
      return e.getAttribute('data-layer');
    }),
    letters: [...svg.querySelectorAll('.board-letter')].map(function (e) {
      return (e.textContent || '').trim();
    }),
    dimensions: [...svg.querySelectorAll('.dim-text')].map(function (e) {
      return (e.textContent || '').replace(/\s+/g, ' ').trim();
    }),
    texts: [...svg.querySelectorAll('text')].map(function (e) {
      return (e.textContent || '').replace(/\s+/g, ' ').trim();
    }),
  });
}, where);

/** What the Section view is showing, and which cut it is on. */
const sectionNow = (session) => session.execute(function () {
  const strip = document.querySelector('.tool-strip');
  return JSON.stringify({
    there: !!document.querySelector('.section-editor'),
    cut: !!document.querySelector('.section-cut canvas'),
    strip: ((strip || {}).innerText || '').replace(/\s+/g, ' ').trim(),
    chips: [...document.querySelectorAll('.view-chip')].map(function (b) {
      return { text: (b.textContent || '').trim(), on: b.classList.contains('on') };
    }),
  });
});

/** What the Estimate tab says about the cut, and which groups it names. */
const cutLinesNow = (session) => session.execute(function () {
  const banner = document.querySelector('.cut-lines');
  return JSON.stringify({
    there: !!banner,
    text: banner ? (banner.textContent || '').replace(/\s+/g, ' ').trim() : '',
    named: [...document.querySelectorAll('.cost-group.cut')].map(function (g) {
      return ((g.querySelector('.group-name') || {}).textContent || '').trim();
    }),
  });
});

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0,
    { what: 'the window' });
  await watchErrors(session);

  // ── the front door ──────────────────────────────────────────────────────
  const demoAt = await centreOf(session, '.start-demo');
  let opened = false;
  if (demoAt) {
    const at = JSON.parse(demoAt);
    try {
      await pointerClick(session, at.x, at.y);
      await wait(1500);
      opened = await session.execute(function () { return !document.querySelector('.start'); });
    } catch { opened = false; }
  }
  if (!opened) await openDemoJob(session);
  await until(session, () => !document.querySelector('.start'), { what: 'the job to open' });
  await wait(1500);

  // ── the Model, with words on it ─────────────────────────────────────────
  await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });
  await wait(1800);
  const model = JSON.parse(await modelNow(session));

  check('every drain on the roof carries its own name and the sump it sits in', () => {
    assert.ok(model.there, 'the Model has nothing to say about itself');
    const labels = model.labels.filter((l) => l.kind === 'drain');
    assert.equal(labels.length, drains.length,
      `${labels.length} drains are labelled and ${drains.length} are on the roof:`
      + ` ${JSON.stringify(model.labels)}`);
    assert.ok(sumpDepth !== null, 'the job states no sump, so there is nothing to read');
    for (const [i, label] of labels.entries()) {
      assert.ok(label.text.startsWith(`Drain ${i + 1}`),
        `a drain label reads "${label.text}"`);
      const said = asFeet(label.text);
      assert.ok(said !== null, `no depth in "${label.text}"`);
      assert.ok(Math.abs(said * 12 - sumpDepth) < 1 / 32,
        `the label says the sump falls ${(said * 12).toFixed(3)} in`
        + ` and the layout makes it ${sumpDepth} in`);
    }
  });

  check('the flat ground says how much of it there is, and why it is flat', () => {
    const label = model.labels.find((l) => l.kind === 'nofall');
    assert.ok(label, `nothing on the roof names the flat ground: ${JSON.stringify(model.labels)}`);
    const said = Number.parseFloat((/([\d,]+)\s*SF/.exec(label.text) ?? [])[1]?.replace(/,/g, ''));
    assert.ok(Number.isFinite(said), `no square feet in "${label.text}"`);
    assert.ok(Math.abs(said - noFallSquareFeet) <= 2,
      `the label says ${said} SF and the lattice measures ${noFallSquareFeet.toFixed(2)} SF`);
    assert.ok(/boards run out|wall/.test(label.text),
      `the label does not say what stopped the fall: "${label.text}"`);
  });

  check('the curb says what stands behind it', () => {
    const curb = conditions.find((c) => c.role === 'curb' && !c.hidden);
    assert.ok(curb, 'the job traces no curb');
    const label = model.labels.find((l) => l.kind === 'curb');
    assert.ok(label, `nothing on the roof names the curb: ${JSON.stringify(model.labels)}`);
    assert.ok(label.text.includes(curb.name), `the curb label reads "${label.text}"`);
    assert.ok(/cricket/.test(label.text),
      `the cricket behind it is not named: "${label.text}"`);
  });

  check('the key is short enough to read at a glance', () => {
    assert.ok(model.keyRows > 0, 'there is no key at all');
    assert.ok(model.keyRows <= 5,
      `the key has ${model.keyRows} rows on it: "${model.legend}"`);
    assert.ok(model.legend.includes('no fall'), `the key does not name the flat ground`);
  });

  await shot(session, 'section-section-model');

  // ── the boards, on the roof ─────────────────────────────────────────────
  check('the roof offers its board grid', () => {
    const chip = model.chips.find((c) => c.text === 'boards');
    assert.ok(chip, `there is no boards chip: ${JSON.stringify(model.chips.map((c) => c.text))}`);
    assert.equal(chip.on, false, 'the grid is drawn before anybody asked for it');
  });

  const pressedBoards = await pressChip(session, 'boards');
  await wait(1400);
  const withBoards = JSON.parse(await modelNow(session));

  check('and the grid it draws is the bill C2 counted, board for board', () => {
    assert.ok(pressedBoards, 'the boards chip could not be pressed');
    assert.equal(withBoards.kinds.board ?? 0, boardCount,
      `${withBoards.kinds.board ?? 0} boards are drawn and the lattice counts ${boardCount}`);
  });

  await shot(session, 'section-section-boards');
  if (pressedBoards) { await pressChip(session, 'boards'); await wait(800); }

  // ── the Section ─────────────────────────────────────────────────────────
  const wentToSection = await showView(session, 'Section');
  await wait(1800);
  const section = JSON.parse(await sectionNow(session));
  const drawing = JSON.parse(await drawingNow(session, '.section-drawing svg'));

  check('there is a Section view, and it opens on the first drain', () => {
    assert.ok(wentToSection, 'the view switch offers no Section');
    assert.ok(section.there, 'the Section view mounted nothing');
    assert.ok(section.cut, 'there is no three-dimensional cut beside the drawing');
    const on = section.chips.filter((c) => c.text.startsWith('cut ') && c.on);
    assert.equal(on.length, 1, `${on.length} cuts are lit: ${JSON.stringify(section.chips)}`);
    assert.equal(on[0].text, cuts[0].chip, `the view opened on "${on[0].text}"`);
    assert.equal(section.chips.filter((c) => c.text.startsWith('cut ')).length, drains.length,
      'there is not one cut on offer per drain');
    assert.ok(section.strip.includes('4'),
      `the strip does not state the vertical exaggeration: "${section.strip}"`);
  });

  check('the drawing stacks the roof up the way it gets built', () => {
    assert.ok(drawing.there, 'there is no section drawing on screen');
    assert.ok(drawing.layers.length >= 4,
      `the section draws ${drawing.layers.length} layers: ${JSON.stringify(drawing.layers)}`);
    // Bottom to top: the deck it lands on, the flat board, the lettered panel,
    // and then whatever the condition's own lines say goes over it — sorted by
    // what each one says it DOES in the roof, against the trade's own order from
    // the deck up. Which is not the order the condition lists them in: this demo
    // lists the membrane first and it belongs on top.
    const order = drawing.layers.filter((l, i) => drawing.layers.indexOf(l) === i);
    const rank = (line) => {
      const at = LAYER_FUNCTIONS.indexOf(line.layer);
      return at < 0 ? LAYER_FUNCTIONS.length : at;
    };
    const stated = (fieldCondition.items ?? [])
      .filter((i) => typeof i.thickness === 'number' && i.thickness > 0)
      .map((line, i) => ({ line, i }))
      .sort((a, b) => rank(a.line) - rank(b.line) || a.i - b.i)
      .map(({ line }) => line.description);
    assert.equal(order[0], 'deck', `the bottom layer is "${order[0]}"`);
    assert.ok(order.includes('fill'), `there is no flat board in ${JSON.stringify(order)}`);
    assert.ok(order.includes('panel'), `there is no tapered panel in ${JSON.stringify(order)}`);
    assert.ok(order.indexOf('fill') < order.indexOf('panel'),
      `the tapered panel is under the flat board it stacks on: ${JSON.stringify(order)}`);
    for (const [i, name] of stated.entries()) {
      assert.equal(order[order.length - stated.length + i], name,
        `the stated layers come out ${JSON.stringify(order.slice(-stated.length))}`
        + ` and the condition's lines stack ${JSON.stringify(stated)}`);
    }
  });

  check('and the lettered panels on it are the letters the bill buys along that cut', () => {
    assert.ok(drawing.letters.length > 0, 'no panel on the section carries a letter');
    const drawn = new Set(drawing.letters);
    for (const letter of drawn) {
      assert.ok(boardsByLetter.has(letter),
        `the section draws a "${letter}" panel and the bill buys none`);
    }
    // Along a cut from the drain to the far end, the thickness only rises, so the
    // letters come out in the chart's own order and never repeat out of turn.
    const reached = [];
    // Bounded by a count as well as by the run: a loop whose only bound is a
    // derived figure is a loop that runs for ever the day the figure is wrong.
    const most = Math.ceil(cuts[0].run / panelFeet) + 1;
    for (let out = 0; out <= most && out * panelFeet <= cuts[0].run + 1e-9; out += 1) {
      // The sump takes the boards over the drain out of the layout, so they are
      // not on the cut either: a sump panel is bought instead of a field board.
      if (sumpFeet !== undefined && out * panelFeet < sumpFeet / 2) continue;
      const thickness = Math.min(atDrain + out * panelFeet * slope, ceiling);
      const { letter } = stackAt(thickness);
      if (letter && !reached.includes(letter)) reached.push(letter);
    }
    assert.deepEqual([...drawn].sort(), reached.sort(),
      `the cut draws ${JSON.stringify([...drawn].sort())} and the layout lays`
      + ` ${JSON.stringify(reached.sort())}`);
  });

  check('the dimensions on it are the numbers the job comes to', () => {
    const wanted = cuts[0];
    const found = (what) => drawing.dimensions.find((d) => what.test(d));
    const run = found(/high point to drain/);
    assert.ok(run, `no run is dimensioned: ${JSON.stringify(drawing.dimensions)}`);
    assert.ok(Math.abs(asFeet(run) - wanted.run) < 1 / 24,
      `the drawing says ${run} and the cut runs ${wanted.run.toFixed(3)} ft`);

    const high = found(/at the (wall|ridge)/);
    assert.ok(high, `the thick end is not dimensioned: ${JSON.stringify(drawing.dimensions)}`);
    assert.ok(Math.abs(asFeet(high) * 12 - wanted.highInches) < 1 / 32,
      `the drawing says ${high} and the layout makes it ${wanted.highInches} in`);

    const low = found(/at the drain/);
    assert.ok(low, `the drain end is not dimensioned: ${JSON.stringify(drawing.dimensions)}`);
    assert.ok(Math.abs(asFeet(low) * 12 - atDrain) < 1 / 32,
      `the drawing says ${low} and the job states ${atDrain} in at the drain`);

    const wall = found(/parapet|wall/);
    assert.ok(wall, `the wall is not dimensioned: ${JSON.stringify(drawing.dimensions)}`);
    assert.ok(Math.abs(asFeet(wall) * 12 - wallInches) < 1 / 32,
      `the drawing says ${wall} and the job states ${wallInches} in of wall`);

    const sump = found(/sump/);
    assert.ok(sump, `the sump is not dimensioned: ${JSON.stringify(drawing.dimensions)}`);
    assert.ok(Math.abs(asFeet(sump) - sumpFeet) < 1 / 24,
      `the drawing says ${sump} and the job states a ${sumpFeet} ft sump`);
  });

  check('and it says what it is drawn at, and how far the vertical is stretched', () => {
    assert.ok(/1["″]\s*=|1 in =/.test(drawing.note), `the drawing states no scale: "${drawing.note}"`);
    assert.ok(/4/.test(drawing.note) && /vertical/i.test(drawing.note),
      `the drawing does not state the vertical exaggeration: "${drawing.note}"`);
  });

  await shot(session, 'section-section-cut');

  // ── and the money knows what the plane went through ─────────────────────
  await showTab(session, 'Estimate');
  await wait(900);
  const cutLines = JSON.parse(await cutLinesNow(session));

  check('the Estimate names the lines the cut passes through', () => {
    assert.ok(cutLines.there, 'the Estimate says nothing about the cut');
    assert.ok(cutLines.text.includes(cuts[0].letters),
      `the banner reads "${cutLines.text}" and the cut is ${cuts[0].name}`);
    const said = Number.parseFloat((/·\s*(\d+)/.exec(cutLines.text) ?? [])[1]);
    assert.ok(Number.isFinite(said) && said > 0, `no count in "${cutLines.text}"`);
    // Every condition the plane goes through: the field it cut, the drain it cut
    // down to, and the wall it ended against.
    const through = [fieldCondition, drainCondition, wallCondition].filter(Boolean);
    const lines = through.reduce((n, c) => n + (c.items ?? []).length, 0);
    assert.equal(said, lines,
      `the banner counts ${said} lines and the conditions the plane passes through carry ${lines}`);
  });

  // ── the cut moves ───────────────────────────────────────────────────────
  await showView(session, 'Section');
  await wait(900);
  const moved = await pressChip(session, cuts[1].chip);
  await wait(1400);
  const afterMove = JSON.parse(await drawingNow(session, '.section-drawing svg'));

  check('and the cut moves to another drain when it is asked to', () => {
    assert.ok(moved, `there is no "${cuts[1].chip}" to press`);
    const run = afterMove.dimensions.find((d) => /high point to drain/.test(d));
    assert.ok(run, `no run is dimensioned after the move: ${JSON.stringify(afterMove.dimensions)}`);
    assert.ok(Math.abs(asFeet(run) - cuts[1].run) < 1 / 24,
      `the second cut says ${run} and that drain's ground runs ${cuts[1].run.toFixed(3)} ft`);
  });

  // ── the Plan's two overlays ─────────────────────────────────────────────
  await showView(session, 'Plan');
  await wait(1400);
  const pressedLayout = await pressChip(session, 'layout');
  await wait(1600);
  const layout = JSON.parse(await drawingNow(session, '.surface-overlay'));

  check('the Plan lays the boards out, with a letter on every one of them', () => {
    assert.ok(pressedLayout, 'the Plan offers no layout overlay');
    assert.ok(layout.there, 'there is no overlay on the drawing');
    assert.equal(layout.letters.length, boardCount,
      `${layout.letters.length} letters are on the layout and the lattice counts`
      + ` ${boardCount} boards`);
    const drawn = new Map();
    for (const letter of layout.letters) drawn.set(letter, (drawn.get(letter) ?? 0) + 1);
    for (const [letter, count] of boardsByLetter) {
      assert.equal(drawn.get(letter) ?? 0, count,
        `the layout draws ${drawn.get(letter) ?? 0} of ${letter} and the bill buys ${count}`);
    }
  });

  await shot(session, 'section-section-layout');

  const pressedFastening = await pressChip(session, 'fastening');
  await wait(1200);
  const fastening = JSON.parse(await drawingNow(session, '.surface-overlay'));

  check('and the other overlay is a fastening plan, in zones and not in boards', () => {
    // This row used to read "it says the fastening plan is not solved yet",
    // because at C5's head it was empty and said so. C4 solved the zones, so the
    // claim moves with the program: the second overlay is a DIFFERENT drawing of
    // the same roof — bands, not a board grid — and it names what it was set out
    // to on its own face.
    assert.ok(pressedFastening, 'the Plan offers no fastening overlay');
    assert.ok(fastening.marks > 0, 'the fastening overlay is empty');
    assert.equal(fastening.letters.length, 0,
      'the fastening overlay drew boards, which is the layout\'s drawing and not this one');
    const bands = (fastening.roles ?? []).filter((r) => String(r).startsWith('zone-'));
    assert.ok(bands.length >= 3, `the fastening overlay draws ${bands.length} zone bands`);
    assert.match(fastening.note, /ASCE/,
      `the fastening overlay never names the standard: "${fastening.note}"`);
  });

  const errors = await errorsSoFar(session);
  check('and nothing in the window complained on the way through', () => {
    assert.deepEqual(errors, [], `the window logged ${JSON.stringify(errors)}`);
  });

  console.log(`\n  ${results.filter((r) => r === 'PASS').length}/${results.length}`);
  console.log(`  pictures: evidence/section-section-model-${COMMIT}.png (beside Frame 2)`);
  console.log(`            evidence/section-section-cut-${COMMIT}.png (beside Frame 3)`);
  console.log(`            evidence/section-section-layout-${COMMIT}.png`);
  console.log(`            evidence/section-section-boards-${COMMIT}.png`);
} finally {
  await pointerRelease(session);
  await app.close();
}

process.exit(results.includes('FAIL') ? 1 : 0);
