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
const priceBook = await read('prices.json');
const costCodes = await read('cost-codes.json');
// A line on the sheet is known by its description; the class it is priced in
// comes off the price book's entry for that description and the cost code's
// own class — the two files the demo is priced from, read rather than typed.
const classOf = (description) => {
  const entries = Array.isArray(priceBook) ? priceBook : (priceBook.items ?? priceBook.prices ?? []);
  const entry = entries.find((e) => (e.description ?? '').trim() === (description ?? '').trim());
  if (!entry) return null;
  const codes = Array.isArray(costCodes) ? costCodes : (costCodes.codes ?? []);
  const code = codes.find((c) => c.code === entry.costCode);
  return code ? code.class : null;
};

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

/**
 * The cricket's footprint, worked out the way NRCA sizes one.
 *
 * Figs. 8-1 and 10-13/10-14: the ridge is square to the curb's face and centred
 * on it, and the cricket is half that face wide on each side. So the plan area is
 * the ridge's own run times the whole face. Cricket stock is its own lettered
 * series and the chart on disk carries only the primary slope, which is why this
 * is an area and not a count of boards.
 */
const cricketCondition = conditions.find(
  (c) => c.kind === 'line' && c.role === 'ridge' && !c.hidden,
);
const cricketSquareFeet = (() => {
  if (!cricketCondition) return null;
  const traced = feet(cricketCondition.traces[0]);
  if (traced.length < 2) return null;
  const curb = conditions.find(
    (c) => (cricketCondition.between ?? []).includes(c.id) && c.role === 'curb' && !c.hidden,
  );
  if (!curb) return null;
  const ring = feet(curb.traces[0]);
  let length = 0;
  for (let i = 1; i < traced.length; i += 1) {
    length += Math.hypot(traced[i].x - traced[i - 1].x, traced[i].y - traced[i - 1].y);
  }
  // The face the estimator started their ridge at.
  let face = null;
  for (let i = 1; i < ring.length; i += 1) {
    const a = ring[i - 1]; const b = ring[i];
    const run = Math.hypot(b.x - a.x, b.y - a.y);
    if (run < 1e-6) continue;
    const mid = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
    const off = Math.hypot(mid.x - traced[0].x, mid.y - traced[0].y);
    if (!face || off < face.off) face = { off, run };
  }
  return face === null ? null : length * face.run;
})();

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
  + ` ${sumpPanels} sump panels; ${cricketSquareFeet} SF of cricket;`
  + ` ${Math.round(laidSquareFeet).toLocaleString('en-US')} SF laid`);

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
  ...(cricketCondition && cricketSquareFeet !== null
    ? { [cricketCondition.id]: { CRICKET_SF: cricketSquareFeet } }
    : {}),
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
 * The selling price is PINNED under whatever tab is up, in the cost panel's own
 * footer (`.selling .value`), and it is drawn straight off `recap()` — the same
 * arithmetic the Recap tab renders. The class table on the Recap tab
 * (`.lens-sheet`) carries the classes and their rates and stops there, which is
 * why reading the last number in it found Supervision's rate and not the job.
 *
 * `notCounted` is the footer's own footnote: the lines the recap left out. On a
 * solved job it is absent, and its presence is a finding in itself.
 */
const recapNow = (session) => session.execute(function () {
  const sheet = document.querySelector('.lens-sheet');
  const selling = document.querySelector('.cost-panel .selling .value');
  const partial = document.querySelector('.cost-panel .selling .partial');
  return JSON.stringify({
    there: !!selling,
    selling: selling ? (selling.textContent || '').trim() : null,
    classes: sheet ? (sheet.innerText || '').replace(/\s+/g, ' ').trim() : '',
    notCounted: partial ? (partial.textContent || '').trim() : null,
    why: partial ? (partial.getAttribute('title') || '') : '',
  });
});

/** The derived names the Condition panel lists, with the words beside each. */
const panelNow = (session) => session.execute(function () {
  const panel = document.querySelector('.condition-panel');
  if (!panel) return JSON.stringify({ there: false });
  const rows = [...panel.querySelectorAll('.panel-derived .derived-row')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return { name: text('.derived-name'), label: text('.derived-label'), value: text('.derived-value') };
  });
  return JSON.stringify({
    there: true,
    name: ((panel.querySelector('.panel-name') || {}).value) || null,
    rows,
    text: (panel.innerText || '').replace(/\s+/g, ' ').trim(),
  });
});

/**
 * The named numbers the shell solved, as the window itself holds them.
 *
 * Pulled with the same command the app pulls them with when a window connects —
 * `solve_get('scopes')` — so this reads what the program has rather than what
 * this file thinks it should have. The bill above is still worked out here, and
 * the two are compared: this says what the solve produced, that says what the
 * roof comes to.
 */
const scopesNow = (session) => session.execute(function () {
  return window.__TAURI_INTERNALS__.invoke('solve_get', { part: 'scopes' })
    .then(function (s) { return JSON.stringify(s); })
    .catch(function (e) { return JSON.stringify({ error: String(e) }); });
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

  check('no MATERIAL line prices the whole tapered package by the square', () => {
    // The money seam: one line, `SQ` off the traced area, at a price picked for an
    // average thickness somebody guessed. A bill by letter and that line together
    // would charge the package twice.
    //
    // LABOR IS EXEMPT AND THAT IS DELIBERATE. "Lay the tapered package" is a crew
    // rate, and a crew rate per square is what a roofer states and what the
    // production rate in the catalogue is written as — squares a crew-day. It is
    // not a sketch value standing in for a bill; it is the rate. The plan says so
    // in as many words ("Labor lines as they are"), and pricing the crew by the
    // board would be a second decision this batch was not asked to make. So this
    // row reads the class off the job and asks the question of material only.
    const sketch = onField.filter(
      (l) => /taper/i.test(l.description ?? '')
        && (l.formula ?? '').trim() === 'SQ'
        && classOf(l.description) === 'Material',
    );
    assert.equal(sketch.length, 0,
      `"${sketch.map((l) => l.description).join('", "')}" still prices the package by the square`);
    // And the crew's own line is still there, priced the way a roofer states it.
    const crew = onField.filter((l) => classOf(l.description) === 'Labor');
    assert.ok(crew.length > 0, 'no labor line on the field at all');
  });

  check('and the four letters can be told apart on a 364 px panel', () => {
    // The estimate picture at the commit before this showed all four reading
    // "Tapered polyisocyan…" — four lines the same, differing only in their
    // money, which is worse than no split at all. The letter is the second word
    // now, so it survives the ellipsis; what this holds is that the descriptions
    // differ inside their first dozen characters.
    const heads = [...boardsByLetter.keys()].sort().map((letter) => {
      const line = named(`TAPER_${letter}_SF`)[0];
      return (line?.description ?? '').slice(0, 12);
    });
    assert.equal(new Set(heads).size, heads.length,
      `they read "${heads.join('", "')}" where the name is cut off`);
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

  check(`the job rolls up to $${withTheBill.sellingPrice.toFixed(2)}, the bill included`, () => {
    assert.ok(rolled.there, 'the cost panel pins no selling price');
    const shown = money(rolled.selling);
    const want = Number(withTheBill.sellingPrice.toFixed(2));
    assert.ok(shown !== null, `the selling price reads "${rolled.selling}"`);
    assert.ok(Math.abs(shown - want) < 0.02,
      `the panel says ${shown} and this file's own bill comes to ${want}`);
  });

  check('and every line is in it — nothing about the package is pending', () => {
    assert.equal(rolled.notCounted, null,
      `the panel says "${rolled.notCounted}" — ${rolled.why.replace(/\n/g, ' · ')}`);
    // And no code word reached the footnote either, which is where the recap's
    // reasons are read as a sentence.
    assert.ok(!/TAPER_|FILL_|SUMP_|CRICKET_|BOARDS_/.test(rolled.why),
      `the footnote says "${rolled.why}"`);
  });

  // ── the Condition panel says which names there are ──────────────────────
  await showTab(session, 'Condition');
  await until(session, () => !!document.querySelector('.condition-panel'), { what: 'the panel' });
  const picked = await pickCondition(session, fieldCondition.name);
  const panel = JSON.parse(await panelNow(session));
  const solvedScopes = JSON.parse(await scopesNow(session));

  check('the Condition panel lists exactly the names the field carries', () => {
    assert.ok(picked, `"${fieldCondition.name}" is not in the tree to pick`);
    assert.ok(panel.there, 'there is no condition panel');
    const carried = Object.keys(solvedScopes[fieldCondition.id] ?? {}).sort();
    const listed = panel.rows.map((r) => r.name).sort();
    assert.ok(carried.length > 0, 'the solve carries nothing for the field');
    assert.deepEqual(listed, carried,
      `the panel lists ${listed.join(', ') || 'nothing'} and the solve carries ${carried.join(', ')}`);
  });

  check('and the solve carries the bill this file worked out for itself', () => {
    // The other half: the panel agrees with the solve, and the solve agrees with
    // the roof. Every name this file can work out is compared; the ones it does
    // not model — the cut waste and the ground with no fall — are only required
    // to be there and to be numbers.
    const carried = solvedScopes[fieldCondition.id] ?? {};
    for (const [name, value] of Object.entries(wanted)) {
      assert.ok(Math.abs((carried[name] ?? NaN) - value) < 0.01,
        `the solve says ${name} is ${carried[name]} and the roof comes to ${value}`);
    }
    for (const name of ['BOARD_WASTE_SF', 'NOFALL_SF']) {
      assert.ok(Number.isFinite(carried[name]), `the solve says ${name} is ${carried[name]}`);
    }
    assert.equal((solvedScopes[drainCondition.id] ?? {}).SUMP_PANELS_EA, sumpPanels);
  });

  check('it says each of them in the trade\'s words, with the identifier beside it', () => {
    // The vocabulary rule, on the one panel that could break it. `TAPER_ZZ_SF` is
    // a formula identifier: it belongs in the mono face beside a formula and
    // nowhere else, and the words an estimator reads are the bill's own.
    for (const row of panel.rows) {
      assert.ok(row.label, `"${row.name}" is listed with no words on it`);
      assert.ok(!row.label.includes('_'), `"${row.label}" reads like an identifier`);
      assert.ok(!row.label.includes(row.name), `"${row.label}" is the identifier`);
      assert.match(row.label, / · (SF|LF|EA|SQ)$/, `"${row.label}" says no unit`);
    }
    // In the order a bill is read in: the lettered panels first.
    assert.match(panel.rows[0].label, /^Tapered panel/,
      `the list starts at "${panel.rows[0].label}"`);
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
