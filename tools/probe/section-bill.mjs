// ── C2: the bill an estimator reads, and where it came from ────────────────
// Nobody buys a surface. C1 laid the field out — drainage regions, four planes to
// a region, valleys where two of them tie — and the Estimate still carried ONE
// line for the whole tapered package, `SQ` off the traced area at a price picked
// for an average thickness somebody guessed. That is the money seam D146 named,
// and this is the check that it moved.
//
// What this holds the program to:
//
//   THE LETTERS. One line per lettered panel the layout uses, each carrying ◈,
//   each priced off a formula of derived names alone. The quantity on every one of
//   them is compared against a bill this file works out for itself, off
//   `jobs/demo-job` and `data/profiles/versico-versicore-mp-h.json` — the outline,
//   the drains, the thickness at the drain, the slope, the board ceiling, the sump,
//   and the manufacturer's own panel table. Nothing is typed in here.
//
//   AND IT IS A SECOND ARITHMETIC ON PURPOSE. The core lays a 4 ft grid on each
//   region and clips every cell with a polygon boolean. This file samples the roof
//   on a quarter-foot lattice and marks which cells any of it lands in — an
//   integral, sharing no code and no rounding with the clipper it is holding to
//   account. If the two agree on 697 board positions split four ways by letter,
//   they agree about the roof.
//
//   THE FILL, THE SUMPS, THE CRICKET. Flat board under the lettered panels, by the
//   board; the sump counted in whole panels off NRCA p.164's bowl-plus-24-inches;
//   the cricket by area, because cricket stock is its own lettered series and the
//   chart on disk carries only the primary slope.
//
//   THE RECAP MOVES. The selling price on screen is compared against `recap()` run
//   here over the same job with this file's own bill in scope — and against the
//   same recap with no solve at all, which is what the sheet would say if the seam
//   were not threaded. The gap between those two IS the money C2 moved, and it is
//   printed rather than asserted at a figure, because the figure belongs to the
//   demo's invented prices and will move whenever they are regenerated.
//
//   THE PANEL SAYS WHICH NAMES. The Condition panel lists the derived names the
//   field carries, so an estimator writing a formula can see what there is to name.
//
// Front door throughout — the start screen, the demo button, the tabs, the tree.
// Nothing here calls a command to get a job open.
//
// RED BEFORE C2. At C1's head there is no `TAPER_X_SF` anywhere: `scope.rs` ships
// an empty map, the demo carries one `iso-tapered` line at `SQ`, and the Condition
// panel lists no derived names. Every row below fails, and that is the point of
// writing it first.
//
//   pnpm build:app && node tools/probe/section-bill.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { readFileSync } from 'node:fs';
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

// ── what this roof's bill comes to, worked out from what is on disk ────────
const { capInches, readJob, recap, sumpSide } = await import('../../packages/engine/dist/index.js');
const jobFolder = join(ROOT, 'jobs/demo-job');
const read = async (rel) => JSON.parse(await readFile(join(rel.startsWith('data/') ? ROOT : jobFolder, rel), 'utf8'));
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

/** The lowest thing the membrane has to turn up and flash against, in inches. */
let flashesAgainst = Infinity;
for (const c of conditions) {
  if (c.kind !== 'line' || c.hidden || c.role === 'ridge') continue;
  const h = c.properties?.H;
  if (h === undefined || h <= 0) continue;
  flashesAgainst = Math.min(flashesAgainst, h * 12);
}
const perimeter = Number.isFinite(flashesAgainst) ? flashesAgainst : undefined;

const fieldCondition = conditions.find(
  (c) => c.kind === 'area' && !c.hidden && c.properties?.TAPER !== undefined,
);
const ring = feet(fieldCondition.traces[0]);
const atDrain = fieldCondition.properties?.T ?? 0;
const slope = fieldCondition.properties?.TAPER ?? 0;
const ceiling = capInches(atDrain, fieldCondition.properties?.BOARDS, perimeter).cap;

/**
 * The field's own axes: its longest side, and square to that.
 *
 * The boards are laid to the longest side, so the grid is square to it and the run
 * that sets a thickness is the greater of the two runs along these — not the
 * straight line to the drain. Same expression as `section-core.mjs` and
 * `section-taper.mjs`, because it is the same fact about the layout.
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
// The Extended set — the one the four fixed scope names are drawn from, and the
// default the solve uses where nothing types a `PROFILE_SET`.
const chart = charted.extended;
const fillStep = Number.parseFloat(chart.fill_in);
const panelFeet = Math.min(...profile.panel_size_ft);

/** The sump's side in feet: typed, or the bowl plus about 24 in (NRCA p.164). */
const bowl = drainCondition.properties?.BOWL;
const sumpFeet = drainCondition.properties?.SUMP
  ?? (bowl === undefined ? undefined : sumpSide(bowl) / 12);
const sumpPanels = sumpFeet === undefined
  ? 0
  : drains.length * Math.ceil(sumpFeet / panelFeet) ** 2;

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

/** Is this place inside a sump? The square is centred on the drain, on the axes. */
const inSump = (p) => sumpFeet !== undefined && drains.some((d) => {
  const a = (p.x - d.x) * axisAlong.x + (p.y - d.y) * axisAlong.y;
  const b = (p.x - d.x) * axisAcross.x + (p.y - d.y) * axisAcross.y;
  return Math.abs(a) < sumpFeet / 2 && Math.abs(b) < sumpFeet / 2;
});

/**
 * The bill, by sampling the roof rather than by clipping it.
 *
 * A quarter-foot lattice over the field's box. Every point that is on the roof and
 * not in a sump is put in the cell of the grid its own drain anchors, and a cell
 * with any point in it is a board that gets bought. The thickness is read at the
 * cell's middle — which is a whole number of boards out from the drain, so it is
 * `T + boards x panel x slope` capped — and the fill and the letter come off the
 * chart the way the chart is printed: `fill = step x floor((thickness - T) / step)`
 * and the panel whose printed range covers what is left.
 *
 * The smallest piece of cell on this roof is a foot wide, where two regions' grids
 * meet at a ridge, so a quarter-foot lattice cannot miss one.
 */
const LATTICE_FEET = 0.25;
const box = { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity };
for (const p of ring) {
  box.minX = Math.min(box.minX, p.x); box.maxX = Math.max(box.maxX, p.x);
  box.minY = Math.min(box.minY, p.y); box.maxY = Math.max(box.maxY, p.y);
}

const positions = new Map();
let laidSquareFeet = 0;
for (let x = box.minX + LATTICE_FEET / 2; x < box.maxX; x += LATTICE_FEET) {
  for (let y = box.minY + LATTICE_FEET / 2; y < box.maxY; y += LATTICE_FEET) {
    const p = { x, y };
    if (!inside(p, ring) || inSump(p)) continue;
    laidSquareFeet += LATTICE_FEET * LATTICE_FEET;
    // Which region: the drain whose perpendicular bisectors keep this place, which
    // is the drain it is nearest.
    let which = 0; let far = Infinity;
    drains.forEach((d, k) => {
      const r = (p.x - d.x) ** 2 + (p.y - d.y) ** 2;
      if (r < far) { far = r; which = k; }
    });
    const d = drains[which];
    const alongRun = (p.x - d.x) * axisAlong.x + (p.y - d.y) * axisAlong.y;
    const acrossRun = (p.x - d.x) * axisAcross.x + (p.y - d.y) * axisAcross.y;
    // The grid is anchored on the drain, at the middle of a cell.
    const i = Math.floor((alongRun + panelFeet / 2) / panelFeet);
    const j = Math.floor((acrossRun + panelFeet / 2) / panelFeet);
    positions.set(`${which}:${i}:${j}`, Math.max(Math.abs(i), Math.abs(j)));
  }
}

const boardsByLetter = new Map();
let fillBoards = 0;
let coursesMax = 0;
for (const out of positions.values()) {
  const thickness = Math.min(atDrain + out * panelFeet * slope, ceiling);
  const fill = fillStep * Math.floor((thickness - atDrain + 1e-9) / fillStep);
  const panel = thickness - fill;
  const found = chart.panels.find(
    (q) => panel >= q.thin_edge_in - 1e-9 && panel < q.thick_edge_in - 1e-9,
  );
  const letter = found ? found.designation : null;
  assert.ok(letter, `nothing in the chart covers a ${panel} in panel`);
  boardsByLetter.set(letter, (boardsByLetter.get(letter) ?? 0) + 1);
  const courses = Math.round(fill / fillStep);
  fillBoards += courses;
  coursesMax = Math.max(coursesMax, courses + 1);
}

/** What the solve should be handing the estimate, by name. */
const wanted = {};
for (const [letter, boards] of boardsByLetter) {
  wanted[`TAPER_${letter}_SF`] = boards * panelFeet * panelFeet;
}
wanted.FILL_SF = fillBoards * panelFeet * panelFeet;
wanted.FILL_BOARDS_EA = fillBoards;
wanted.BOARDS_MAX = coursesMax;

console.log(`  REPORT  this file's own bill: ${positions.size} board positions`
  + ` — ${[...boardsByLetter].sort().map(([l, n]) => `${l} ${n}`).join(', ')}`
  + `; ${fillBoards} flat boards under them; ${coursesMax} courses at the deepest;`
  + ` ${sumpPanels} sump panels; ${Math.round(laidSquareFeet).toLocaleString('en-US')} SF laid`);

/** The whole job, as the engine reads it off disk, for the recap either way. */
const jobDocument = readJob((rel) => {
  try { return readFileSync(join(jobFolder, rel), 'utf8'); }
  catch (e) { if (e.code === 'ENOENT') return null; throw e; }
});
const scenario = jobDocument.job.scenarios.find((s) => s.id === jobDocument.job.activeScenarioId)
  ?? jobDocument.job.scenarios[0];

/** The scopes the solve should be producing, keyed the way the wire keys them. */
const scopes = {
  [fieldCondition.id]: wanted,
  [drainCondition.id]: { SUMP_PANELS_EA: sumpPanels },
};
const withTheBill = recap(jobDocument, scenario, scopes);
const withoutIt = recap(jobDocument, scenario);
const moved = withTheBill.sellingPrice - withoutIt.sellingPrice;
console.log(`  REPORT  the recap with this bill in scope comes to`
  + ` $${withTheBill.sellingPrice.toFixed(2)}; with no solve at all it is`
  + ` $${withoutIt.sellingPrice.toFixed(2)} — the seam is worth $${moved.toFixed(2)}`);

const money = (text) => {
  const m = /-?[\d,]+\.\d\d/.exec(String(text ?? ''));
  return m === null ? null : Number.parseFloat(m[0].replace(/,/g, ''));
};

// ── what the window says ───────────────────────────────────────────────────

/** Every line on the Estimate tab, as the estimator sees it. */
const linesNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.cost-line')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return {
      description: text('.line-item'),
      condition: text('.line-basis > span'),
      formula: text('.line-formula'),
      quantity: text('.line-qty'),
      unit: (text('.line-unit') || '').replace('◈', '').trim(),
      derived: !!r.querySelector('.line-unit .derived'),
      money: text('.line-money'),
      why: (r.querySelector('.line-qty') || {}).title || null,
    };
  }));
});

/**
 * What the recap on screen rolls the job up to.
 *
 * `.lens-sheet` is where the Recap tab draws, which is what `section10a-chrome.mjs`
 * reads, and the pending list is the footnote under it.
 */
const recapNow = (session) => session.execute(function () {
  const sheet = document.querySelector('.lens-sheet');
  const panel = document.querySelector('.cost-panel');
  return JSON.stringify({
    there: !!sheet,
    text: sheet ? (sheet.innerText || '').replace(/\s+/g, ' ').trim() : '',
    footnote: panel ? (panel.innerText || '').replace(/\s+/g, ' ').trim() : '',
  });
});

/** The derived names the Condition panel lists, and the fields it shows. */
const panelNow = (session) => session.execute(function () {
  const panel = document.querySelector('.condition-panel');
  if (!panel) return JSON.stringify({ there: false });
  const derived = [...panel.querySelectorAll('.panel-derived .derived-name')]
    .map(function (e) { return (e.textContent || '').trim(); });
  return JSON.stringify({
    there: true,
    name: ((panel.querySelector('.panel-name') || {}).value) || null,
    derived,
    text: (panel.innerText || '').replace(/\s+/g, ' ').trim(),
  });
});

/** The condition panel's field with this label on it, by the words on screen. */
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
}, label);

/** Pick a condition in the tree beside the panel, the way a hand picks it. */
const pickCondition = async (session, name) => {
  const found = await session.execute(function (want) {
    const row = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .find(function (e) { return (e.textContent || '').indexOf(want) >= 0; });
    if (!row) return false;
    row.click();
    return true;
  }, name);
  await wait(800);
  return found;
};

/** What the Model says about itself — the same hatch C0 and C1 read. */
const modelNow = (session) => session.execute(function () {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  if (!m) return JSON.stringify({ there: false });
  const kinds = {};
  for (const mesh of m) kinds[String(mesh.kind)] = (kinds[String(mesh.kind)] || 0) + 1;
  return JSON.stringify({
    there: true,
    source: m.source === undefined ? null : m.source,
    solveId: m.solveId === undefined ? null : m.solveId,
    kinds,
    selectedConditionId: m.selectedConditionId,
    view: m.view,
    legend: ((document.querySelector('.model-legend') || {}).innerText || '').replace(/\s+/g, ' ').trim(),
    page: { w: window.innerWidth, h: window.innerHeight },
  });
});

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

  // ── the Estimate ────────────────────────────────────────────────────────
  await showTab(session, 'Estimate');
  await until(session, () => document.querySelectorAll('.cost-line').length > 0,
    { what: 'the estimate lines' });
  await wait(600);
  const lines = JSON.parse(await linesNow(session));
  const onField = lines.filter((l) => l.condition === fieldCondition.name);
  const named = (name) => lines.filter((l) => (l.formula ?? '').includes(name));

  check('there is a line per lettered panel the layout uses, and no others', () => {
    const letters = [...boardsByLetter.keys()].sort();
    const found = lines
      .map((l) => /TAPER_([A-Z0-9]+)_SF/.exec(l.formula ?? ''))
      .filter((m) => m !== null)
      .map((m) => m[1])
      .sort();
    assert.deepEqual(found, letters,
      `the sheet prices ${found.join(', ') || 'no letters'} and the layout uses ${letters.join(', ')}`);
  });

  for (const [letter, boards] of [...boardsByLetter].sort()) {
    const want = (boards * panelFeet * panelFeet) / 100;
    check(`the ${letter} panel is ${boards} boards, which is ${want.toFixed(2)} SQ, and it is derived`, () => {
      const line = named(`TAPER_${letter}_SF`)[0];
      assert.ok(line, `no line names TAPER_${letter}_SF`);
      assert.equal(line.derived, true,
        `"${line.description}" is worked out from ${line.formula} and carries no mark`);
      const shown = money(line.quantity) ?? Number.parseFloat(String(line.quantity).replace(/,/g, ''));
      assert.ok(Number.isFinite(shown), `it reads "${line.quantity}" — ${line.why ?? 'no reason given'}`);
      assert.ok(Math.abs(shown - want) < 0.01, `it reads ${shown} SQ and the bill is ${want.toFixed(2)}`);
      assert.ok(money(line.money) !== null, `it is priced at "${line.money}"`);
    });
  }

  check(`the flat board under them is ${fillBoards} boards, and it is derived`, () => {
    const line = named('FILL_SF')[0] ?? named('FILL_BOARDS_EA')[0];
    assert.ok(line, 'no line names FILL_SF or FILL_BOARDS_EA');
    assert.equal(line.derived, true, `"${line.description}" carries no mark`);
    const shown = Number.parseFloat(String(line.quantity).replace(/,/g, ''));
    const want = (line.formula ?? '').includes('FILL_BOARDS_EA')
      ? fillBoards
      : (fillBoards * panelFeet * panelFeet) / 100;
    assert.ok(Math.abs(shown - want) < 0.01, `it reads ${shown} and the bill is ${want}`);
  });

  check(`the sump is ${sumpPanels} panels, off the bowl and not off the drain count`, () => {
    const line = named('SUMP_PANELS_EA')[0];
    assert.ok(line, 'no line names SUMP_PANELS_EA');
    assert.equal(line.derived, true, `"${line.description}" carries no mark`);
    const shown = Number.parseFloat(String(line.quantity).replace(/,/g, ''));
    assert.equal(shown, sumpPanels, `it reads ${shown} panels and the layout cuts ${sumpPanels}`);
  });

  check('the cricket is priced by area, because cricket stock is its own series', () => {
    const line = named('CRICKET_SF')[0];
    assert.ok(line, 'no line names CRICKET_SF');
    assert.equal(line.derived, true, `"${line.description}" carries no mark`);
    const shown = Number.parseFloat(String(line.quantity).replace(/,/g, ''));
    assert.ok(Number.isFinite(shown) && shown > 0,
      `it reads "${line.quantity}" — ${line.why ?? 'no reason given'}`);
  });

  check('the one line for the whole tapered package is gone', () => {
    // The money seam: `SQ` off the traced area at a price picked for an average
    // thickness. A bill by letter and that line together would charge the package
    // twice.
    const sketch = onField.filter(
      (l) => /taper/i.test(l.description ?? '') && (l.formula ?? '').trim() === 'SQ',
    );
    assert.equal(sketch.length, 0,
      `"${sketch.map((l) => l.description).join('", "')}" still prices the package by the square`);
  });

  check('no letter line is pending, and none blames the sheet for a solver', () => {
    for (const letter of boardsByLetter.keys()) {
      const line = named(`TAPER_${letter}_SF`)[0];
      if (!line) continue;
      assert.ok(money(line.money) !== null, `${letter} has no money: ${line.why ?? 'no reason'}`);
      assert.ok(!/scale/.test(line.why ?? ''),
        `${letter} says "${line.why}", which blames the sheet for a solved number`);
    }
  });

  check('the board count the sheet prices is the board count the roof takes', () => {
    // Every letter's square feet back to boards, added up, against this file's own
    // count of the grid. One polygon clipper and one lattice integral, agreeing.
    let boards = 0;
    for (const letter of boardsByLetter.keys()) {
      const line = named(`TAPER_${letter}_SF`)[0];
      assert.ok(line, `no line names TAPER_${letter}_SF`);
      boards += (Number.parseFloat(String(line.quantity).replace(/,/g, '')) * 100)
        / (panelFeet * panelFeet);
    }
    assert.ok(Math.abs(boards - positions.size) < 0.01,
      `the sheet prices ${boards} boards and the grid lays ${positions.size}`);
  });

  await shot(session, 'section-bill-estimate');

  // ── the recap ───────────────────────────────────────────────────────────
  await showTab(session, 'Recap');
  await wait(900);
  const rolled = JSON.parse(await recapNow(session));

  check(`the recap rolls up to $${withTheBill.sellingPrice.toFixed(2)}, the bill included`, () => {
    assert.ok(rolled.there, 'there is no recap on the Recap tab');
    const shown = rolled.text.match(/-?[\d,]+\.\d\d/g)?.map((m) => Number.parseFloat(m.replace(/,/g, ''))) ?? [];
    assert.ok(shown.length > 0, `the recap reads "${rolled.text.slice(0, 160)}"`);
    const want = Number(withTheBill.sellingPrice.toFixed(2));
    assert.ok(shown.some((v) => Math.abs(v - want) < 0.02),
      `nothing in the recap is ${want}; it carries ${shown.slice(-4).join(', ')}`);
  });

  check('the tapered package is not in the recap\'s pending list', () => {
    const said = rolled.footnote;
    assert.ok(!/TAPER_/.test(said), `the recap says "${/[^.]*TAPER_[^.]*/.exec(said)?.[0]}"`);
    assert.ok(!/has not been solved/.test(said),
      `the recap says "${/[^.]*has not been solved[^.]*/.exec(said)?.[0]}"`);
    assert.ok(!/no scale/.test(said),
      'the recap blames a sheet\'s scale for a derived number');
  });

  // ── the Condition panel says which names there are ──────────────────────
  await showTab(session, 'Condition');
  await until(session, () => !!document.querySelector('.condition-panel'), { what: 'the panel' });
  const picked = await pickCondition(session, fieldCondition.name);
  const panel = JSON.parse(await panelNow(session));

  check('the Condition panel lists the names the field carries', () => {
    assert.ok(picked, `"${fieldCondition.name}" is not in the tree to pick`);
    assert.ok(panel.there, 'there is no condition panel');
    const want = Object.keys(wanted).sort();
    assert.deepEqual([...panel.derived].sort(), want,
      `the panel lists ${panel.derived.join(', ') || 'nothing'} and the field carries ${want.join(', ')}`);
  });

  const typedThickness = JSON.parse((await fieldValue(session, 'Thickness')) ?? 'null');
  check('and the typed thickness at the drain is still the estimator\'s own', () => {
    // The reserved names stay typed. A solve that quietly wrote over one would
    // move money with nothing on screen saying it had, which is what `scopeFor`
    // refuses at the engine and what this reads off the panel.
    assert.ok(typedThickness, 'no thickness field in the panel');
    assert.equal(Number.parseFloat(typedThickness.value), atDrain,
      `the panel says ${typedThickness.value} in at the drain and the job says ${atDrain}`);
  });

  // ── the picture: the sump and the cricket on the solved surface ──────────
  await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });
  await wait(1600);
  const drawn = JSON.parse(await modelNow(session));

  check('the sumps are drawn as the cut the layout makes, one per drain', () => {
    assert.equal(drawn.there, true, 'the view has nothing to say about itself');
    assert.equal(drawn.source, 'solve_get', `the view says its roof came from "${drawn.source}"`);
    assert.equal(drawn.kinds.sump ?? 0, drains.length,
      `${drawn.kinds.sump ?? 0} sumps in the scene and ${drains.length} drains on the roof`);
  });

  check('the key says the drains are in the sumps the layout cuts for them', () => {
    assert.match(drawn.legend, /sump/, `the key reads "${drawn.legend.slice(0, 200)}"`);
    assert.ok(!/no sump/.test(drawn.legend),
      `the key says "${/[^·]*no sump[^·]*/.exec(drawn.legend)?.[0]}" on a job that states one`);
  });

  check('the picture is taken in the state the other two were taken in', () => {
    assert.ok(drawn.view, 'the view reports no camera');
    assert.equal(drawn.view.corner, 'NE', `it opened from ${drawn.view.corner}`);
    assert.equal(drawn.view.vert, 1, `the vertical is stretched ${drawn.view.vert} times`);
    assert.equal(drawn.page.w, 1440, `the page is ${drawn.page.w}px wide and is specified at 1440`);
    assert.equal(drawn.page.h, 900, `the page is ${drawn.page.h}px tall and is specified at 900`);
  });
  await shot(session, 'section-bill-frame2');

  const errors = await errorsSoFar(session);
  check('nothing threw in the window', () => {
    assert.deepEqual(errors, [], `the window logged ${errors.length}`);
  });
} finally {
  await pointerRelease(session);
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(`\n  ${results.length - failed}/${results.length} rows pass`);
console.log(`  pictures: evidence/section-bill-estimate-${COMMIT}.png`
  + ` · evidence/section-bill-frame2-${COMMIT}.png`);
process.exit(failed === 0 ? 0 : 1);
