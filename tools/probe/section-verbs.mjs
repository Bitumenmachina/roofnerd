// ── C6: the verbs ─────────────────────────────────────────────────────────
// The four acts in the handoff's tool rail that work ON the roof rather than
// tracing something new onto the sheet, each driven the way a roofer drives it:
// pick a place, pick another, read the strip, and press the word on the button.
//
//   SET HIGH AND LOW. Two picks and two thicknesses. The field's TAPER and T
//   have to come out at what those picks IMPLY — the fall between them over the
//   run along the boards — and the run is worked out in here from the job's own
//   geometry, never read off the program. The bill has to move with it: halving
//   the slope on a roof whose boards are lettered by depth changes which letters
//   are bought, and a slope that changed nothing would be a slope nothing is
//   solved from.
//
//   BOX OUT A CURB. Two corners and a height. A curb condition with the lines
//   the job's own curb detail carries, and the cricket behind it out of the
//   solve — one more wedge in the scene than there was before, on the upslope
//   side of the box.
//
//   MEASURE. Two drains, and the distance between them in feet and inches. What
//   it must read is a subtraction of two points out of `jobs/demo-job`, and
//   nothing may be written by it.
//
//   CUT A SECTION. A pick on a drain, and the Section view open on the cut that
//   goes through it — a different one for a different drain, or the act is a
//   button that always shows the first.
//
// NOTHING IN HERE IS TYPED FROM THE FIXTURE. The drains, the outline, the axes,
// the thickness at the drain and the distances all come off `jobs/demo-job` at
// run time. The two thicknesses the act is GIVEN are this file's own choice —
// they are what the estimator types — and every figure held against the program
// is worked out from them and from the job.
//
// Front door throughout: the start screen, the demo button, the view switch, the
// rail, the strip's own fields, the tabs.
//
//   pnpm build:app && node tools/probe/section-verbs.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, errorsSoFar, launch, openDemoJob, pointerClick, pointerRelease, showTab, showView,
  tool, typeKeys, until, wait, watchErrors, commitStamp,
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

// ── the roof, off the job on disk ──────────────────────────────────────────
const jobFolder = join(ROOT, 'jobs/demo-job');
const read = async (rel) => JSON.parse(await readFile(join(jobFolder, rel), 'utf8'));
const conditions = await read('conditions.json');
const pages = await read('pages/pages.json');

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
const field = conditions.find((c) => c.kind === 'area' && !c.hidden && c.properties?.TAPER !== undefined);
const ring = feet(field.traces[0]);
const atDrain = field.properties?.T ?? 0;
const curbNow = conditions.filter((c) => c.role === 'curb' && !c.hidden);

/**
 * The field's own axes: its longest side, and square to that.
 *
 * The same expression `section-core.mjs` and `section-taper.mjs` read the layout
 * with, because it is the same fact about the roof — the boards run along the
 * longest side, and the run a tapered schedule states is measured on those axes
 * and not as the crow flies.
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
const walk = (from, along, across) => ({
  x: from.x + axisAlong.x * along + axisAcross.x * across,
  y: from.y + axisAlong.y * along + axisAcross.y * across,
});
const runAlongBoards = (from, to) => {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  return Math.max(
    Math.abs(dx * axisAlong.x + dy * axisAlong.y),
    Math.abs(dx * axisAcross.x + dy * axisAcross.y),
  );
};

/** Is a place inside the traced outline? Ray casting, the same rule the core uses. */
function insideRing(p, poly) {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i, i += 1) {
    const a = poly[i]; const b = poly[j];
    if ((a.y > p.y) !== (b.y > p.y)
      && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) inside = !inside;
  }
  return inside;
}

// ── what this file asks the program to do ──────────────────────────────────

/**
 * How far up the boards the high pick goes, in feet.
 *
 * Far enough that the slope the act works out is a real one and the bill moves,
 * and near enough that the place is still well inside the outline — it is walked
 * back along the boards from the drain and checked against the traced ring below
 * rather than assumed.
 */
const HIGH_RUN_FEET = 20;

/**
 * The slope this file asks for: an eighth in twelve.
 *
 * Not the demo's own, on purpose. A quarter in twelve is what the job states, so
 * asking for a quarter would pass whether the act wrote anything or not; an
 * eighth is half of it, which moves the depth at every board on the roof and so
 * moves which lettered panel each one is bought as.
 */
const ASK_SLOPE = 0.125;

/** Where the high pick goes, and what the two thicknesses are. */
const drainOne = drains[0];
const highAt = (() => {
  for (const way of [1, -1]) {
    for (const side of [0, 1, -1]) {
      const p = walk(drainOne, HIGH_RUN_FEET * way, side);
      if (insideRing(p, ring)) return p;
    }
  }
  return null;
})();
const highRun = highAt ? runAlongBoards(drainOne, highAt) : 0;
const lowInches = atDrain;
const highInches = lowInches + ASK_SLOPE * highRun;
/** What the field's TAPER must come out at, from the picks and nothing else. */
const wantTaper = (highInches - lowInches) / highRun;

/** The two drains a ruler is dragged between, and what a tape would read. */
const rulerFrom = drains[0];
const rulerTo = drains[1];
const wantFeet = Math.hypot(rulerTo.x - rulerFrom.x, rulerTo.y - rulerFrom.y);

/** Feet as a roofer writes them, for reading the strip back. */
function feetInches(value) {
  const whole = Math.floor(value);
  const sixteenths = Math.round((value - whole) * 192);
  const inches = Math.floor(sixteenths / 16);
  return { feet: whole + (inches === 12 ? 1 : 0), inches: inches === 12 ? 0 : inches };
}

/** A length off the screen, in feet: 92'-0", 8'-9 5/8". */
function readFeet(text) {
  const m = /(-?\d+)'\s*-?\s*(?:(\d+)\s*)?(?:(\d+)\/(\d+))?"/.exec(String(text ?? ''));
  if (!m) return null;
  return Number(m[1]) + (Number(m[2] ?? 0) + (m[3] ? Number(m[3]) / Number(m[4]) : 0)) / 12;
}

/** Inches off the screen, the way the strip writes them: 1/2", 2 1/4", 4". */
function readInches(text) {
  const m = /(?:(\d+)\s*)?(?:(\d+)\/(\d+)\s*)?"/.exec(String(text ?? '').trim());
  if (!m || (m[1] === undefined && m[2] === undefined)) return null;
  return Number(m[1] ?? 0) + (m[2] ? Number(m[2]) / Number(m[3]) : 0);
}

// ── reading the window ─────────────────────────────────────────────────────

const modelNow = (session) => session.execute(function () {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  if (!m) return JSON.stringify({ there: false });
  const kinds = {};
  for (const mesh of m) kinds[String(mesh.kind)] = (kinds[String(mesh.kind)] || 0) + 1;
  return JSON.stringify({
    there: true,
    solveId: m.solveId === undefined ? null : m.solveId,
    kinds,
    labels: [...document.querySelectorAll('.model-label')].map(function (l) {
      return { kind: l.getAttribute('data-label-kind'), text: (l.textContent || '').trim() };
    }),
  });
});

/** Where a place on the roof lands on the screen, asked of the drawing itself. */
const aimAt = (session, at) => session.execute(function (x, y) {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  const p = m && m.screenPointAt ? m.screenPointAt(x, y) : null;
  return JSON.stringify(p);
}, at.x, at.y);

const stripNow = (session) => session.execute(function () {
  const strip = document.querySelector('.tool-strip');
  if (!strip) return JSON.stringify({ there: false });
  const fields = {};
  for (const f of strip.querySelectorAll('.strip-field')) {
    const parts = [...f.children];
    const key = parts[0] ? (parts[0].textContent || '').trim() : '';
    const input = f.querySelector('input');
    fields[key] = input ? input.value : (parts[1] ? (parts[1].textContent || '').trim() : '');
  }
  return JSON.stringify({
    there: true,
    name: ((strip.querySelector('.tool-strip-name') || {}).textContent || '').trim(),
    fields,
    said: ((strip.querySelector('.verb-said') || {}).textContent || '').trim() || null,
    derived: [...strip.querySelectorAll('.strip-field .derived')].length,
    text: (strip.innerText || '').replace(/\s+/g, ' ').trim(),
  });
});

const linesNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.cost-line')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    const cost = r.querySelector('.line-money');
    return {
      description: text('.line-item'),
      condition: text('.line-basis > span'),
      quantity: text('.line-qty'),
      derived: !!r.querySelector('.line-unit .derived'),
      money: text('.line-money'),
      // Why a line shows a dash instead of money — the sentence the program
      // puts in the tooltip, which is where a pending line says what it is
      // waiting for.
      why: cost ? cost.getAttribute('title') : null,
    };
  }));
});

const groupsNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.cost-group')].map(function (g) {
    return ((g.querySelector('.group-name') || {}).textContent || '').trim();
  }));
});

/** The recap's own selling price and how many lines it left out of it. */
const recapNow = (session) => session.execute(function () {
  const value = document.querySelector('.selling .value');
  const partial = document.querySelector('.selling .partial');
  return JSON.stringify({
    sellingPriceText: value ? (value.textContent || '').trim() : null,
    notCountedText: partial ? (partial.textContent || '').trim() : null,
  });
});

/** The panel's field with this label on it, by the words on screen. */
const panelField = (session, label) => session.execute(function (want) {
  const l = [...document.querySelectorAll('.condition-panel label')]
    .find(function (e) { return ((e.querySelector('span') || {}).textContent || '').trim() === want; });
  const input = l ? l.querySelector('input') : null;
  return input ? input.value : null;
}, label);

const pickCondition = async (session, name) => {
  await showTab(session, 'Estimate');
  await wait(500);
  const at = await centreOf(session, '.cost-group', name);
  let clicked = false;
  if (at) {
    try {
      const p = JSON.parse(at);
      await pointerClick(session, p.x, p.y);
      clicked = true;
    } catch { clicked = false; }
  }
  if (!clicked) {
    clicked = await session.execute(function (want) {
      const row = [...document.querySelectorAll('.cost-group')]
        .find(function (r) { return (r.textContent || '').indexOf(want) >= 0; });
      if (!row) return false;
      row.click();
      return true;
    }, name);
  }
  await wait(800);
  return clicked;
};

/** Which cut the Section view is showing, by the chip that is lit. */
const cutShowing = (session) => session.execute(function () {
  const on = [...document.querySelectorAll('.view-switch .view-chip.on')]
    .map(function (c) { return (c.textContent || '').trim(); })
    .filter(function (t) { return t.indexOf('cut ') === 0; });
  return JSON.stringify({
    on,
    note: ((document.querySelector('.section-drawing .drawing-note') || {}).textContent || '').trim(),
    marks: document.querySelectorAll('.section-drawing .drawing g *').length,
  });
});

// ── picking a place on the roof, the way a hand does ───────────────────────

let pointerPath = 'not tried';

/** Put a finger on a place on the roof. Answers where on screen it landed. */
async function pickPlace(session, at, what) {
  const aim = JSON.parse((await aimAt(session, at)) ?? 'null');
  if (!aim || !aim.inside) throw new Error(`${what} is not on screen (${JSON.stringify(aim)})`);
  if (pointerPath === 'actions') {
    await pointerClick(session, aim.x, aim.y);
  } else {
    await session.execute(function (x, y) {
      const c = document.querySelector('.model-canvas');
      for (const type of ['pointerdown', 'pointerup', 'click']) {
        c.dispatchEvent(new MouseEvent(type, { clientX: x, clientY: y, bubbles: true, detail: 1 }));
      }
    }, aim.x, aim.y);
  }
  await wait(700);
  return aim;
}

/**
 * Type into a field in the tool settings strip.
 *
 * The keys go through the driver where it will take them, because a field that
 * is rebuilt while it is being typed into is exactly the thing a scripted value
 * write cannot catch. Where the driver refuses, the value is set and the same
 * two events the browser sends are sent — which is the path the field listens
 * on — and the run says which way it went.
 */
async function typeInStrip(session, label, text) {
  const cleared = await session.execute(function (want) {
    const input = [...document.querySelectorAll('.tool-strip input')]
      .find(function (i) { return (i.getAttribute('aria-label') || '') === want; });
    if (!input) return null;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(input, '');
    const r = input.getBoundingClientRect();
    return JSON.stringify({ x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2) });
  }, label);
  if (!cleared) throw new Error(`there is no ${label} field in the strip`);
  const box = JSON.parse(cleared);

  if (pointerPath === 'actions') {
    try {
      await pointerClick(session, box.x, box.y);
      await typeKeys(session, String(text));
      await session.actions([{
        type: 'key',
        id: 'keyboard',
        actions: [{ type: 'keyDown', value: '' }, { type: 'keyUp', value: '' }],
      }]);
      await wait(900);
      return 'typed';
    } catch {
      // fall through to the scripted path, and say so
    }
  }
  await session.execute(function (want, value) {
    const input = [...document.querySelectorAll('.tool-strip input')]
      .find(function (i) { return (i.getAttribute('aria-label') || '') === want; });
    if (!input) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(input, String(value));
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  }, label, String(text));
  await wait(900);
  return 'scripted';
}

/** Press a word on a button in the strip. */
async function pressInStrip(session, word) {
  const at = await centreOf(session, '.tool-strip button', word);
  if (at && pointerPath === 'actions') {
    const p = JSON.parse(at);
    try {
      await pointerClick(session, p.x, p.y);
      await wait(1400);
      return true;
    } catch { /* the scripted path below */ }
  }
  const pressed = await session.execute(function (want) {
    const b = [...document.querySelectorAll('.tool-strip button')]
      .find(function (e) { return (e.textContent || '').trim() === want; });
    if (!b || b.disabled) return false;
    b.click();
    return true;
  }, word);
  await wait(1400);
  return pressed;
}

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

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
    } catch {
      pointerPath = 'synthetic';
    }
  } else {
    pointerPath = 'synthetic';
  }
  console.log(`  REPORT  real pointer input: ${pointerPath === 'actions'
    ? 'available — every pick below is a driver click'
    : 'refused, so every pick below is a dispatched event'}`);
  if (pointerPath !== 'actions') await openDemoJob(session);
  await until(session, () => !document.querySelector('.start'), { what: 'the job to open' });
  await wait(1500);

  console.log(`  REPORT  the high pick is ${highRun.toFixed(1)} ft up the boards from the first drain;`
    + ` asking for ${highInches.toFixed(3)} in over ${lowInches.toFixed(3)} in`);

  // ── the rail offers the acts ────────────────────────────────────────────
  await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });
  await wait(1800);

  const railNow = await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.tool-rail .rail-tool')]
      .map(function (b) { return b.getAttribute('aria-label'); }));
  });
  const rail = JSON.parse(railNow);
  check('the rail offers the acts a roofer performs, in the words a roofer uses', () => {
    for (const word of ['Set high and low', 'Box out a curb', 'Measure', 'Cut a section']) {
      assert.ok(rail.includes(word), `the rail has no "${word}" — it offers ${JSON.stringify(rail)}`);
    }
  });

  // ── the bill before anything moves ──────────────────────────────────────
  await showTab(session, 'Estimate');
  await wait(700);
  const before = JSON.parse(await linesNow(session));
  /**
   * The letter a tapered line is for, off the words on it.
   *
   * By the DESCRIPTION and not by an id: the sheet's lettered lines are
   * generated from the letters the solve actually bought, one per letter, so
   * there is no `iso-x` on the job to look for any more — and looking for one
   * is how this row came to say "there were no lettered panel lines to begin
   * with" about a sheet that had four of them on it.
   */
  const letterOf = (line) => {
    const m = /^tapered\s+([a-z]+[0-9]*)\b/i.exec((line.description || '').trim());
    return m === null ? null : m[1].toUpperCase();
  };
  const lettersOn = (lines) =>
    [...new Set(lines.map(letterOf).filter((l) => l !== null))].sort();
  const lettered = (lines) => lines
    .filter((l) => letterOf(l) !== null)
    .map((l) => `${l.description}=${l.quantity}`).sort().join(' | ');
  const lettersBefore = lettersOn(before);
  const letteredBefore = lettered(before);
  const modelBefore = JSON.parse(await modelNow(session));

  // ── SET HIGH AND LOW ────────────────────────────────────────────────────
  assert.ok(highAt, 'the job has nowhere on the boards to put a high pick');
  await tool(session, 'Set high and low');
  await wait(500);
  await pickPlace(session, highAt, 'the high pick');
  await pickPlace(session, drainOne, 'the low pick');
  const picked = JSON.parse(await stripNow(session));

  check('two picks put the act’s own strip up, with the drain it took named', () => {
    assert.equal(picked.name, 'Set high and low', `the strip reads "${picked.name}"`);
    assert.ok(picked.fields.High !== undefined, `no High field: ${picked.text}`);
    assert.ok(picked.fields.Low !== undefined, `no Low field: ${picked.text}`);
    assert.ok(picked.fields.At, `the strip does not say which drain it took: ${picked.text}`);
  });

  const howHigh = await typeInStrip(session, 'High', highInches.toFixed(3));
  const howLow = await typeInStrip(session, 'Low', lowInches.toFixed(3));
  console.log(`  REPORT  the two thicknesses went in ${howHigh}/${howLow}`);
  const asked = JSON.parse(await stripNow(session));

  check('the slope is the fall over the run along the boards, and it carries ◈', () => {
    const slope = readInches(asked.fields.Slope);
    assert.ok(slope !== null, `the strip shows no slope: ${asked.text}`);
    assert.ok(Math.abs(slope - wantTaper) < 1 / 64,
      `the strip reads ${slope} in 12 and the picks imply ${wantTaper}`);
    assert.ok(asked.derived > 0, 'nothing in the strip is marked as worked out');
  });

  await shot(session, 'section-verbs-picks');

  const set = await pressInStrip(session, 'Set high and low');
  const after = JSON.parse(await stripNow(session));
  const modelAfter = JSON.parse(await modelNow(session));

  check('the act writes, and the shell solves the roof again', () => {
    assert.ok(set, 'the strip would not take the act');
    assert.ok(modelAfter.solveId > modelBefore.solveId,
      `the solve did not move: ${modelBefore.solveId} then ${modelAfter.solveId}`);
    assert.ok(!after.said || !/cannot|refus/i.test(after.said), `the act said "${after.said}"`);
  });

  await pickCondition(session, field.name);
  await showTab(session, 'Condition');
  await wait(700);
  const taperOnPanel = await panelField(session, 'Taper');
  const thicknessOnPanel = await panelField(session, 'Thickness');

  check('the field’s taper and thickness are what the two picks imply', () => {
    assert.ok(taperOnPanel !== null, 'there is no Taper field in the panel');
    assert.ok(Math.abs(Number(taperOnPanel) - wantTaper) < 1e-6,
      `the panel reads ${taperOnPanel} and the picks imply ${wantTaper}`);
    assert.ok(Math.abs(Number(thicknessOnPanel) - lowInches) < 1e-6,
      `the panel reads ${thicknessOnPanel} at the drain and the pick said ${lowInches}`);
  });

  await showTab(session, 'Estimate');
  await wait(900);
  const linesAfterSlope = JSON.parse(await linesNow(session));
  const letteredAfter = lettered(linesAfterSlope);
  const lettersAfter = lettersOn(linesAfterSlope);
  check('the bill re-solves: the lettered panels are not the ones it was buying', () => {
    assert.ok(letteredBefore.length > 0, 'there were no lettered panel lines to begin with');
    assert.ok(lettersAfter.length > 0,
      `the sheet buys no lettered panel at all after the slope moved (it bought ${lettersBefore.join(', ')})`);
    assert.notEqual(letteredAfter, letteredBefore,
      'halving the slope bought exactly the same boards, which cannot be right');
  });
  check('and the letters themselves changed, because the chart did', () => {
    // A quarter in twelve buys X, Y, Z and ZZ; an eighth buys a different
    // lettered series entirely — the bulletin prints two unrelated charts, not
    // one chart with moving edges. So this is not the same four panels at new
    // quantities: it is a different set of panels.
    assert.notDeepEqual(lettersAfter, lettersBefore,
      `the sheet bought ${lettersBefore.join(', ')} before and ${lettersAfter.join(', ')} after,`
      + ' which is the same chart');
    const shared = lettersAfter.filter((l) => lettersBefore.includes(l));
    assert.equal(shared.length, 0,
      `${shared.join(', ')} is on the sheet at both slopes, and the two charts share no designation`);
  });
  check('and every panel the new chart names is priced, not pending', () => {
    // What C6's fix round could not make true and this is the proof of: the
    // lines are generated from the letters the solve bought and priced from the
    // job's own price set, which carries every lettered panel in the
    // manufacturer's document. Four letters typed into a job file priced
    // nothing at all here.
    for (const line of linesAfterSlope) {
      if (letterOf(line) === null) continue;
      assert.ok(line.money && !/^—/.test(line.money),
        `"${line.description}" shows ${line.money || 'nothing'}: ${line.why ?? 'no reason given'}`);
    }
  });

  // The letters just changed families — a quarter in twelve buys X, Y, Z and ZZ,
  // and an eighth buys a different lettered series entirely (the bulletin prints
  // two unrelated charts, not one chart with moving edges). When this was
  // written the four lines this job carried were typed for the OLD letters, so
  // they went honestly pending here and the rows above could only check that
  // everything else survived. The job carries ONE line for the package now and
  // the sheet draws one per letter the solve bought, so the rows above hold the
  // letters themselves; this one still holds what the solve carries regardless
  // of which letters govern — the fill, the board waste, the fastener and
  // adhesive counts by zone, the plates — read off `c-field`'s own items rather
  // than typed here.
  const aggregateDescriptions = new Set(
    (field.items ?? [])
      .filter((it) => typeof it.formula === 'string' && /^[A-Z][A-Z0-9_]*_(SF|EA|GAL)\b/.test(it.formula)
        && !it.formula.startsWith('TAPER_'))
      .map((it) => it.description),
  );
  check('halving the slope does not take the fill, the fasteners or the adhesive down with the letters', () => {
    assert.ok(aggregateDescriptions.size > 0, 'the field has no non-lettered board line to check');
    const byDescription = new Map(linesAfterSlope.map((l) => [l.description, l]));
    for (const description of aggregateDescriptions) {
      const line = byDescription.get(description);
      assert.ok(line, `"${description}" is not on the sheet: ${JSON.stringify(linesAfterSlope.map((l) => l.description))}`);
      assert.ok(line.quantity && line.quantity !== '—',
        `"${description}" is pending right after the slope moved: ${JSON.stringify(line)}`);
    }
  });

  // ── BOX OUT A CURB ──────────────────────────────────────────────────────
  await showView(session, 'Model');
  await wait(1200);
  const beforeCurb = JSON.parse(await modelNow(session));
  const cricketsBefore = beforeCurb.kinds.cricket ?? 0;

  // A box on the boards, well inside the outline, between the first drain and
  // the high pick — eight feet across the fall so the water backs up behind it.
  const curbMid = walk(drainOne, HIGH_RUN_FEET / 2, 0);
  const corners = [walk(curbMid, -2, -4), walk(curbMid, 2, 4)];
  assert.ok(corners.every((p) => insideRing(p, ring)), 'the curb would not be on the roof');

  await tool(session, 'Box out a curb');
  await wait(500);
  await pickPlace(session, corners[0], 'the first corner');
  await pickPlace(session, corners[1], 'the corner across from it');
  const curbStrip = JSON.parse(await stripNow(session));
  check('two corners give a curb its size, and the strip says the cricket is coming', () => {
    assert.equal(curbStrip.name, 'Box out a curb');
    assert.ok(curbStrip.fields.Size, `the strip does not say how big the box is: ${curbStrip.text}`);
    assert.ok(curbStrip.fields.Cricket, `the strip says nothing about a cricket: ${curbStrip.text}`);
  });

  const height = curbNow[0]?.properties?.H ?? 2;
  await typeInStrip(session, 'Height', String(height));
  const boxed = await pressInStrip(session, 'Box it out');
  await wait(1800);
  const afterCurb = JSON.parse(await modelNow(session));
  await shot(session, 'section-verbs-curb');

  check('the curb is on the roof with a cricket behind it, out of the solve', () => {
    assert.ok(boxed, 'the strip would not box it out');
    assert.ok((afterCurb.kinds.cricket ?? 0) > cricketsBefore,
      `there were ${cricketsBefore} crickets and there are ${afterCurb.kinds.cricket ?? 0}`);
  });

  await showTab(session, 'Estimate');
  await wait(900);
  const groups = JSON.parse(await groupsNow(session));
  const curbLines = JSON.parse(await linesNow(session))
    .filter((l) => /^Curb \d/.test(l.condition || ''));
  check('the new curb prices: it carries the lines this job’s curb detail carries', () => {
    assert.ok(groups.some((g) => /^Curb \d/.test(g)),
      `no new curb on the sheet: ${JSON.stringify(groups)}`);
    assert.ok(curbLines.length >= (curbNow[0]?.items?.length ?? 1),
      `the new curb carries ${curbLines.length} lines and the template carries `
      + `${curbNow[0]?.items?.length ?? 0}`);
    assert.ok(curbLines.every((l) => l.quantity && l.quantity !== '—'),
      `a line on the new curb has no quantity: ${JSON.stringify(curbLines)}`);
  });

  // The demo's own RTU-1 cricket is typed at half an inch to the foot, which
  // was twice the field at a quarter in twelve and is four times it now that
  // the field is at an eighth — a real disagreement, and the key says so
  // (`legendFor`'s `cricketSlopeOff` row). What it must not do is take the
  // saddle's own line down with it: `cricket()` still answers a `Cricket` for
  // a disagreeing slope, it only carries the flag, so `CRICKET_SF` is still a
  // number and the line still prices. Boxing out a second curb is the same
  // shape of question one condition over: a curb with no cricket, or one this
  // program refuses outright, must not touch the FIELD's own board bill next
  // to it — the sump, the letters, the fill, the fastener and adhesive counts
  // are the field's regardless of what stands on it.
  const linesAfterCurb = JSON.parse(await linesNow(session));
  const priceOf = (lines, description) => (lines.find((l) => l.description === description) ?? {}).quantity;
  // The cricket's own line lives on the cricket condition, not the field's —
  // read its description off the job rather than typing it twice.
  const cricketDescription = conditions
    .flatMap((c) => (c.role === 'ridge' ? c.items ?? [] : []))
    .find((it) => (it.formula || '') === 'CRICKET_SF')?.description;

  check('holds the bill after the curb: the fill, the fasteners, the adhesive and the disagreeing cricket all still price', () => {
    assert.ok(cricketDescription, 'no ridge condition in the job carries a CRICKET_SF line to check');
    for (const description of aggregateDescriptions) {
      const now = priceOf(linesAfterCurb, description);
      assert.ok(now && now !== '—', `"${description}" went pending once the curb was on the roof`);
    }
    const cricketNow = priceOf(linesAfterCurb, cricketDescription);
    assert.ok(cricketNow && cricketNow !== '—',
      `"${cricketDescription}" is pending — a slope that only disagrees must not stop it pricing: ${cricketNow}`);
    // Nothing that priced right after the slope moved may go pending for a
    // reason the curb introduced: the two snapshots are compared line for
    // line, by description, rather than assuming the set is unchanged (the
    // curb and its cricket add lines of their own).
    for (const before of linesAfterSlope) {
      if (!before.quantity || before.quantity === '—') continue;
      const now = priceOf(linesAfterCurb, before.description);
      assert.ok(now && now !== '—',
        `"${before.description}" priced before the curb (${before.quantity}) and does not now`);
    }
  });

  const recapAfterCurb = JSON.parse(await recapNow(session));
  check('the selling price after the curb is the recap’s own figure, not typed here', () => {
    assert.ok(recapAfterCurb.sellingPriceText, 'no selling price is shown at all');
    const asNumber = Number(recapAfterCurb.sellingPriceText.replace(/[^0-9.]/g, ''));
    assert.ok(Number.isFinite(asNumber) && asNumber > 0,
      `the selling price reads "${recapAfterCurb.sellingPriceText}"`);
  });

  // ── MEASURE ─────────────────────────────────────────────────────────────
  await showView(session, 'Model');
  await wait(1200);
  const solveBeforeRuler = JSON.parse(await modelNow(session)).solveId;
  await tool(session, 'Measure');
  await wait(500);
  await pickPlace(session, rulerFrom, 'one end of the ruler');
  await pickPlace(session, rulerTo, 'the other end');
  const ruler = JSON.parse(await stripNow(session));
  await shot(session, 'section-verbs-ruler');

  check('a ruler between two drains reads the distance the job puts between them', () => {
    const said = readFeet(ruler.fields.Distance);
    assert.ok(said !== null, `the strip shows no distance: ${ruler.text}`);
    const want = feetInches(wantFeet);
    assert.ok(Math.abs(said - wantFeet) < 1 / 12,
      `the strip reads ${ruler.fields.Distance} and the job puts `
      + `${want.feet}'-${want.inches}" between those drains`);
  });

  check('it reads the thickness at each end off the solve, and writes nothing', () => {
    const a = readInches(ruler.fields['At the first pick']);
    const b = readInches(ruler.fields['At the second pick']);
    assert.ok(a !== null && b !== null, `no thickness at the ends: ${ruler.text}`);
    assert.ok(Math.abs(a - atDrain) < 1 / 16 && Math.abs(b - atDrain) < 1 / 16,
      `a drain is ${atDrain} in and the ruler reads ${a} and ${b}`);
    const now = JSON.parse(JSON.stringify(ruler));
    assert.ok(now.text.length > 0);
  });

  const solveAfterRuler = JSON.parse(await modelNow(session)).solveId;
  check('measuring changes nothing: the solve did not move', () => {
    assert.equal(solveAfterRuler, solveBeforeRuler,
      `the solve went ${solveBeforeRuler} to ${solveAfterRuler} for a tape measure`);
  });

  // ── CUT A SECTION ───────────────────────────────────────────────────────
  await tool(session, 'Cut a section');
  await wait(500);
  await pickPlace(session, drains[0], 'the first drain');
  await until(session, () => !!document.querySelector('.section-drawing'),
    { what: 'the Section view', timeout: 15000 });
  await wait(1600);
  const first = JSON.parse(await cutShowing(session));
  await shot(session, 'section-verbs-cut');

  check('picking a drain opens the Section on the cut that goes through it', () => {
    assert.equal(first.on.length, 1, `${first.on.length} cuts are lit: ${JSON.stringify(first.on)}`);
    assert.ok(first.marks > 20, `the section drew ${first.marks} marks`);
  });

  // And a different drain is a different cut, or the act is a button that only
  // ever shows the first one.
  await showView(session, 'Model');
  await wait(1200);
  await tool(session, 'Cut a section');
  await wait(400);
  await pickPlace(session, drains[drains.length - 1], 'the last drain');
  await until(session, () => !!document.querySelector('.section-drawing'),
    { what: 'the Section view again', timeout: 15000 });
  await wait(1600);
  const second = JSON.parse(await cutShowing(session));
  check('a different drain is a different cut', () => {
    assert.ok(second.on.length === 1 && second.on[0] !== first.on[0],
      `the first pick showed ${JSON.stringify(first.on)} and the second `
      + `${JSON.stringify(second.on)}`);
  });

  const errors = await errorsSoFar(session);
  check('nothing in the window went wrong while all that happened', () => {
    assert.deepEqual(errors, [], errors.join('\n'));
  });
} finally {
  await pointerRelease(session);
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(`\n  ${results.length - failed}/${results.length} at ${COMMIT}`);
process.exit(failed ? 1 : 0);
