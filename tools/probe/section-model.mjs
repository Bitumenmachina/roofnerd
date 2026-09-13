// ── The Model view, as the design handoff's Frame 2 ────────────────────────
// One claim, in the words it was made in: "what I am seeing does not look like
// anything capable ... the 3d is wrong."
//
// So this check is about the picture, and it is written so that a picture which
// is wrong cannot pass it. Two things in here are new in kind, and both exist
// because a count of meshes cannot reach them:
//
//   THE ROOF IS IN THE WINDOW. A roof drawn at a twentieth of the size it
//   should be has every mesh any check could ask for. The view is asked where
//   the whole roof lands on the canvas and how big the canvas is, and the
//   division is done here — on open, and again after the cost panel leaves and
//   the surface gains back the 364px it was lending, which is exactly when the
//   roof used to stay small.
//
//   A PRESET MOVES THE CAMERA AND NOTHING ELSE. Every mesh in the scene is
//   read before and after each corner, by kind and by its own bounding box, and
//   compared literally. A preset that quietly re-solved the roof would pass a
//   check that only looked at the camera.
//
// The money is checked against the engine's own arithmetic on the same folder
// rather than against a figure typed into this file. A check carrying its own
// copy of a fixture value breaks when the fixture legitimately changes and —
// worse — keeps passing when the window is wrong and the fixture moved to match
// it.
//
// Front door throughout: the start screen, the demo button, the view switch,
// the chips on the drawing, the theme toggle in the document bar.
//
//   pnpm build:app && node tools/probe/section-model.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, detachCostPanel, dockCostPanel, errorsSoFar, launch, openDemoJob, pointerClick,
  pointerRelease, showTab, showView, until, wait, watchErrors, commitStamp,
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

// ── what the job itself comes to, worked out here rather than typed here ───
const { capInches, readJob, recap } = await import('../../packages/engine/dist/index.js');
const jobFolder = join(ROOT, 'jobs/demo-job');
const readFile8 = async (rel) => {
  try { return await readFile(join(jobFolder, rel), 'utf8'); } catch { return null; }
};
const files = new Map();
for (const rel of ['job.json', 'conditions.json', 'cost-codes.json', 'pages/pages.json',
  'prices.json', 'library.json']) files.set(rel, await readFile8(rel));
const demoDoc = readJob((rel) => files.get(rel) ?? null);
const demoScenario = demoDoc.job.scenarios.find((s) => s.id === demoDoc.job.activeScenarioId)
  ?? demoDoc.job.scenarios[0];
const demoRecap = recap(demoDoc, demoScenario);
/** The same formatting the panel uses, so the two strings are comparable. */
const asMoney = (v) => v.toLocaleString('en-US', { style: 'currency', currency: 'USD' });
const field = demoDoc.conditions.find((c) => c.properties?.TAPER !== undefined && c.kind === 'area');
const thinnest = field?.properties?.T ?? 0;
/**
 * The lowest thing on this roof the membrane has to turn up and flash against,
 * in inches above the deck. Read off the document: a parapet at 42 in and a curb
 * at 24 in, and it is the curb that decides, because the roof cannot rise past
 * the lowest of them and still be flashed.
 */
let flashesAgainst = Infinity;
for (const c of demoDoc.conditions) {
  if (c.kind !== 'line' || c.hidden || c.role === 'ridge') continue;
  const h = c.properties?.H;
  if (h === undefined || h <= 0) continue;
  flashesAgainst = Math.min(flashesAgainst, h * 12);
}
/**
 * Where the roof stops rising, and which of the two ceilings stopped it.
 *
 * The engine's own arithmetic on the job's own numbers, not a board thickness
 * copied into this file: a copy would keep passing after the trade's figure
 * moved, and it would be blind to the other ceiling entirely. Boards mean buy
 * another layer; flashing height means the wall will not take one. An estimator
 * reads that difference off the legend, so the check has to know which it is.
 */
const { cap: cappedAt, reason: cappedBy } = capInches(
  thinnest,
  field?.properties?.BOARDS,
  Number.isFinite(flashesAgainst) ? flashesAgainst : undefined,
);

// ── what the window says about the roof it is drawing ──────────────────────
const modelNow = (session) => session.execute(function () {
  const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
  if (!m) return JSON.stringify({ there: false });
  const kinds = {};
  const shapes = [];
  for (const mesh of m) {
    kinds[String(mesh.kind)] = (kinds[String(mesh.kind)] || 0) + 1;
    shapes.push([mesh.conditionId, mesh.kind,
      mesh.min.map(function (n) { return Math.round(n * 1000); }),
      mesh.max.map(function (n) { return Math.round(n * 1000); })]);
  }
  shapes.sort();
  const canvas = document.querySelector('.model-canvas');
  const box = canvas ? canvas.getBoundingClientRect() : null;
  const ticks = [...document.querySelectorAll('.model-ticks span')].map(function (s) { return s.textContent; });
  return JSON.stringify({
    there: true,
    kinds,
    shapes,
    view: m.view,
    selectedConditionId: m.selectedConditionId,
    ticks,
    legend: ((document.querySelector('.model-legend') || {}).innerText || '').replace(/\s+/g, ' ').trim(),
    canvas: box ? { w: Math.round(box.width), h: Math.round(box.height) } : null,
    canvases: document.querySelectorAll('.model-canvas').length,
    others: document.querySelectorAll('.model-ifc, .model-stage').length,
  });
});

/** Press one of the chips on the drawing, by the word on it. */
const chip = async (session, label) => {
  const pressed = await session.execute(function (want) {
    const b = [...document.querySelectorAll('.view-nav .view-chip')]
      .find(function (e) { return (e.textContent || '').trim() === want; });
    if (!b) return false;
    b.click();
    return true;
  }, label);
  await wait(700);
  return pressed;
};

const readoutNow = (session) => session.execute(function () {
  const out = {};
  for (const f of document.querySelectorAll('.status-strip .readout .readout-field')) {
    const k = ((f.querySelector('.k') || {}).textContent || '').trim();
    out[k] = ((f.querySelector('.v') || {}).textContent || '').trim();
  }
  return JSON.stringify(out);
});

const sellingNow = (session) => session.execute(function () {
  return ((document.querySelector('.selling .value') || {}).textContent || '').trim();
});

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

let pointerPath = 'not tried';
let pointerRefused = '';

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  // ── the front door, proven on a click whose outcome cannot be misread ────
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

  // ── the job that opens is the one on disk, and it adds up to what it says ─
  const opened = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      job: ((document.querySelector('.doc-job') || {}).textContent || '').trim(),
      area: ((document.querySelector('.roof-area') || {}).innerText || '').replace(/\s+/g, ' ').trim(),
    });
  }));
  const selling = await sellingNow(session);

  check('the demo opens as the handoff\'s job, at its own roof area', () => {
    assert.match(opened.job, /Northgate/, `the document bar reads "${opened.job}"`);
    assert.match(opened.area, /9,600/, `the roof area reads "${opened.area}"`);
  });
  check('and the selling price is what the engine makes of the same folder', () => {
    // Not a figure typed into this file. The engine reads `jobs/demo-job` off
    // disk above and rolls it up; this is that number against the one pinned
    // under the lines. They are the same arithmetic or one of them is wrong.
    assert.equal(selling, asMoney(demoRecap.sellingPrice),
      `the panel reads "${selling}" and the engine makes it ${asMoney(demoRecap.sellingPrice)}`);
    assert.deepEqual(demoRecap.pending, [], `lines with no money: ${demoRecap.pending.join(' | ')}`);
  });

  // ── the Model, as it opens ──────────────────────────────────────────────
  await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });
  await wait(1400);
  const first = JSON.parse(await modelNow(session));

  check('the Model is one canvas filling the surface, and nothing beside it', () => {
    assert.equal(first.there, true, 'the view has nothing to say about itself');
    assert.equal(first.canvases, 1, `${first.canvases} canvases in the surface`);
    assert.equal(first.others, 0, `${first.others} other stages are still in the surface`);
    assert.ok(first.canvas.w > 400 && first.canvas.h > 300,
      `the canvas is ${first.canvas.w}x${first.canvas.h}`);
  });

  check('it opens on the fixed isometric, from the north-east corner', () => {
    assert.ok(first.view, 'the view reports no camera');
    assert.equal(first.view.corner, 'NE', `it opened from ${first.view.corner}`);
    assert.equal(first.view.vert, 1, `the vertical is stretched ${first.view.vert} times on open`);
    assert.equal(first.view.looking, false, 'the orbit is live before anybody asked for it');
  });

  /** How much of the window the roof takes, in each direction. */
  const fills = (view) => {
    const s = view?.onScreen;
    if (!s) return null;
    return { x: s.w / s.of.w, y: s.h / s.of.h, s };
  };

  check('the roof is in the window when it opens, and fills it', () => {
    // The defect this is for: a roof correctly drawn and correctly lit, at a
    // twentieth of the size, in a window with room for all of it.
    const f = fills(first.view);
    assert.ok(f, 'the view cannot say where the roof landed');
    assert.ok(f.s.x >= -2 && f.s.y >= -2
      && f.s.x + f.s.w <= f.s.of.w + 2 && f.s.y + f.s.h <= f.s.of.h + 2,
      `the roof is drawn at ${f.s.x},${f.s.y} ${f.s.w}x${f.s.h} in a ${f.s.of.w}x${f.s.of.h} canvas`);
    assert.ok(Math.max(f.x, f.y) > 0.8,
      `the roof takes ${(f.x * 100).toFixed(0)}% of the width and ${(f.y * 100).toFixed(0)}% of the height`);
  });

  // ── everything Frame 2 draws is in the scene ────────────────────────────
  check('the tapered field, the parapet and its coping are drawn', () => {
    assert.ok((first.kinds.taper ?? 0) >= 1, `${first.kinds.taper ?? 0} tapered fields`);
    assert.ok((first.kinds.parapet ?? 0) >= 1, `${first.kinds.parapet ?? 0} parapets`);
    assert.ok((first.kinds.coping ?? 0) >= 1,
      `${first.kinds.coping ?? 0} copings — the parapet states a cap width, so one is owed`);
  });
  check('and the drains, their sumps, the valley between them and the curb', () => {
    const drains = demoDoc.conditions.filter((c) => c.role === 'drain')
      .reduce((n, c) => n + (c.traces ?? []).reduce((m, t) => m + t.points.length, 0), 0);
    assert.equal(first.kinds.drain ?? 0, drains, `${first.kinds.drain ?? 0} drains against ${drains} in the job`);
    assert.equal(first.kinds.sump ?? 0, drains, `${first.kinds.sump ?? 0} sumps against ${drains} drains`);
    assert.equal(first.kinds.valley ?? 0, Math.max(0, drains - 1),
      `${first.kinds.valley ?? 0} valleys between ${drains} drains`);
    assert.ok((first.kinds.curb ?? 0) >= 1, `${first.kinds.curb ?? 0} curbs`);
  });
  check('and the cricket, as two planes', () => {
    assert.ok((first.kinds.cricket ?? 0) >= 1, `${first.kinds.cricket ?? 0} crickets`);
    const cricket = first.shapes.find((sh) => sh[1] === 'cricket');
    assert.ok(cricket, 'no cricket to measure');
    // Two planes off a ridge means it has a top and a bottom, not one flat face.
    assert.ok(cricket[3][1] > cricket[2][1], 'the cricket is flat — it has no ridge');
  });

  check('the ramp has depths written on it, and they are the roof\'s own', () => {
    assert.equal(first.ticks.length, 3, `ticks: ${JSON.stringify(first.ticks)}`);
    for (const t of first.ticks) assert.match(String(t), /"/, `tick "${t}" is not a depth`);
    // The thin end is what the estimator put at the drains and the thick end is
    // where the roof stops rising. Both read off the job, not off this file.
    const asInches = (t) => {
      const m = /^(?:(\d+)\s*)?(?:(\d)\/(\d))?"/.exec(String(t).trim());
      if (!m) return null;
      return Number(m[1] ?? 0) + (m[2] ? Number(m[2]) / Number(m[3]) : 0);
    };
    const [thin, middle, thickest] = first.ticks.map(asInches);
    // At the drains, and never under them. The surface is sampled on a two foot
    // grid that does not land exactly on a drain, so the thinnest board DRAWN
    // sits an eighth or two over the half inch the estimator stated. Under it
    // would mean the ramp had been labelled from something other than this roof.
    assert.ok(thin >= thinnest - 0.07 && thin < thinnest + 1,
      `the thin end reads ${first.ticks[0]} and the drains are at ${thinnest} in`);
    assert.ok(Math.abs(thickest - cappedAt) < 0.13,
      `the thick end reads ${first.ticks[2]} and the roof stops rising at ${cappedAt} in`
      + ` because ${cappedBy === 'flashing' ? 'the wall will not take another layer' : 'the boards run out'}`);
    // And the one in the middle is in the middle, which is what makes a ramp
    // readable: a tick put anywhere else turns a scale into decoration.
    assert.ok(Math.abs(middle - (thin + thickest) / 2) < 0.13,
      `the ticks read ${JSON.stringify(first.ticks)}`);
  });

  check('and the key names the marks that are on this roof', () => {
    for (const want of ['fall to the drain', 'no fall', 'valley between drains']) {
      assert.ok(first.legend.includes(want), `the key reads "${first.legend}"`);
    }
    // And it says WHICH ceiling the roof reached, because the two are not the
    // same problem and the legend is where an estimator finds out which one he
    // has. The engine settled it above; this is the window agreeing.
    const why = cappedBy === 'flashing'
      ? 'under the wall it flashes against'
      : 'the boards run out';
    assert.ok(first.legend.includes(why),
      `the roof is capped by the ${cappedBy} and the key reads "${first.legend}"`);
  });

  await shot(session, 'section-model-frame2-light');

  // ── the four corners, and what they are allowed to move ─────────────────
  const before = JSON.stringify(first.shapes);
  const seen = [];
  for (const [id, label] of [['NE', 'NE corner'], ['NW', 'NW corner'], ['SE', 'SE corner'], ['SW', 'SW corner']]) {
    const pressed = await chip(session, label);
    const after = JSON.parse(await modelNow(session));
    seen.push({ id, label, pressed, view: after.view, shapes: JSON.stringify(after.shapes) });
    await shot(session, `section-model-${id.toLowerCase()}`);
  }

  check('each corner is offered, and each one takes the camera there', () => {
    for (const s of seen) {
      assert.ok(s.pressed, `there is no "${s.label}" chip on the drawing`);
      assert.equal(s.view?.corner, s.id, `"${s.label}" left the view at ${s.view?.corner}`);
    }
    const places = new Set(seen.map((s) => JSON.stringify(s.view.position.map((n) => Math.round(n)))));
    assert.equal(places.size, 4, `four corners put the camera in ${places.size} places`);
  });

  check('and a corner moves the camera and nothing on the roof', () => {
    // Every mesh, by condition, by kind and by its own bounding box, before and
    // after all four. A preset that re-solved the roof would pass a check that
    // only read the camera.
    for (const s of seen) {
      assert.equal(s.shapes, before, `"${s.label}" changed what is drawn, not only where it is seen from`);
    }
  });

  check('and the roof is still framed from every one of them', () => {
    for (const s of seen) {
      const f = fills(s.view);
      assert.ok(f && Math.max(f.x, f.y) > 0.8,
        `from ${s.label} the roof takes ${(f ? Math.max(f.x, f.y) * 100 : 0).toFixed(0)}% of the window`);
    }
  });

  // ── the vertical, which stretches the picture and not the roof ──────────
  await chip(session, 'NE corner');
  const flat = JSON.parse(await modelNow(session));
  const pressedVert = await chip(session, 'vert ×3');
  const tall = JSON.parse(await modelNow(session));
  await shot(session, 'section-model-vert3');

  check('the vertical exaggeration is offered, and it stretches the picture', () => {
    assert.ok(pressedVert, 'there is no "vert x3" chip on the drawing');
    assert.equal(tall.view.vert, 3, `the view reports a stretch of ${tall.view.vert}`);
    const high = (m) => Math.max(...m.shapes.map((sh) => sh[3][1]));
    assert.ok(high(tall) > high(flat) * 2.5,
      `the tallest thing went from ${high(flat)} to ${high(tall)}`);
  });
  check('and the roof stays in the window while it is stretched', () => {
    const f = fills(tall.view);
    assert.ok(f && Math.max(f.x, f.y) > 0.8,
      `stretched, the roof takes ${(f ? Math.max(f.x, f.y) * 100 : 0).toFixed(0)}% of the window`);
  });
  await chip(session, 'vert ×1');

  // ── what the roof does under the pointer, over the tapered field ────────
  const aim = JSON.parse(await session.execute(function (id) {
    const m = window.__roofnerdModel ? window.__roofnerdModel() : null;
    const p = m && m.screenPointOf ? m.screenPointOf(id) : null;
    return JSON.stringify(p);
  }, field?.id ?? 'c-field'));

  await session.execute(function (x, y) {
    const c = document.querySelector('.model-canvas');
    c.dispatchEvent(new MouseEvent('pointermove', { clientX: x, clientY: y, bubbles: true }));
  }, aim?.x ?? 0, aim?.y ?? 0);
  await wait(600);
  const reading = JSON.parse(await readoutNow(session));

  check('the roof answers where the pointer is, over the tapered field', () => {
    assert.ok(aim && aim.inside, `the field projects to ${aim?.x}, ${aim?.y}, which is not on the canvas`);
    for (const key of ['Cursor', 'Slope', 'Insulation here', 'Flow']) {
      assert.ok(reading[key] !== undefined, `the readout has no "${key}": ${JSON.stringify(reading)}`);
      assert.notEqual(reading[key], '—', `"${key}" is an em dash over the roof`);
    }
    assert.match(reading['Slope'], /in 12/, `the slope reads "${reading['Slope']}"`);
    assert.match(reading['Insulation here'], /"/, `the thickness reads "${reading['Insulation here']}"`);
  });

  // ── a pick on the field reaches the money ───────────────────────────────
  let pickPath = pointerPath === 'actions' ? 'actions' : 'synthetic';
  if (aim && aim.inside) {
    if (pickPath === 'actions') {
      try { await pointerClick(session, aim.x, aim.y); }
      catch (e) { pickPath = 'synthetic'; pointerRefused = e.message; }
    }
    if (pickPath !== 'actions') {
      await session.execute(function (x, y) {
        const c = document.querySelector('.model-canvas');
        c.dispatchEvent(new MouseEvent('click', { clientX: x, clientY: y, bubbles: true, button: 0 }));
      }, aim.x, aim.y);
    }
    await wait(1000);
  }
  const picked = JSON.parse(await modelNow(session));
  await showTab(session, 'Condition');
  await wait(600);
  const panel = JSON.parse(await session.execute(function () {
    const el = document.querySelector('.condition-panel');
    if (!el) return JSON.stringify({ there: false });
    const measure = {};
    for (const cell of el.querySelectorAll('.panel-measures > div')) {
      const label = ((cell.querySelector('.measure-label') || {}).textContent || '').trim();
      measure[label] = ((cell.querySelector('.measure-value') || {}).textContent || '').trim();
    }
    return JSON.stringify({
      there: true,
      name: (el.querySelector('.panel-name') || {}).value || null,
      measure,
      cost: ((el.querySelector('.panel-money .value') || {}).textContent || '').trim(),
    });
  }));

  check(`picking the field in 3D selects it, and the panel says what it is (${pickPath})`, () => {
    assert.equal(picked.selectedConditionId, field?.id ?? 'c-field',
      `the view believes ${picked.selectedConditionId ?? 'nothing'} is selected`);
    assert.ok(panel.there, 'no Condition tab to read it in');
    assert.equal(panel.name, field?.name, `the panel names "${panel.name}"`);
    assert.match(panel.measure?.['Area'] ?? '', /9,600/, `its area reads "${panel.measure?.['Area']}"`);
    assert.match(panel.cost ?? '', /\$/, `its money reads "${panel.cost}"`);
  });

  await showTab(session, 'Estimate');

  // ── the cost panel leaves, and the roof takes the room ──────────────────
  const narrow = JSON.parse(await modelNow(session));
  const oneWindow = await session.handles();
  const tore = await detachCostPanel(session);
  await wait(2000);
  const handles = await session.handles();
  const torn = handles.find((h) => !oneWindow.includes(h)) ?? null;
  if (torn) { await session.switchTo(oneWindow[0]); await wait(1200); }
  const wide = JSON.parse(await modelNow(session));

  check('the cost panel goes to its own window and the surface takes the width', () => {
    assert.ok(tore, 'no detach button beside the tabs');
    assert.ok(torn, `${oneWindow.length} window before and ${handles.length} after`);
    assert.ok(wide.canvas.w > narrow.canvas.w,
      `the canvas was ${narrow.canvas.w}px with the panel and is ${wide.canvas.w}px without it`);
  });
  check('and the roof grows into it instead of staying the size it was', () => {
    // The torn-off small-roof defect, in one row. The canvas gained 364px and a
    // roof that kept its old size would take a smaller share of a bigger window
    // while every other row on this page stayed green.
    const was = fills(narrow.view);
    const now = fills(wide.view);
    assert.ok(was && now, 'the view cannot say where the roof landed');
    assert.ok(now.s.w > was.s.w,
      `the roof was ${was.s.w}px wide in a ${was.s.of.w}px canvas and is ${now.s.w}px in a ${now.s.of.w}px one`);
    assert.ok(Math.max(now.x, now.y) > 0.8,
      `after the panel left, the roof takes ${(Math.max(now.x, now.y) * 100).toFixed(0)}% of the window`);
  });
  await shot(session, 'section-model-torn-off');

  if (torn) {
    await dockCostPanel(session);
    await wait(2500);
    const back = await session.handles();
    await session.switchTo(back[0]);
    await wait(1500);
  }

  // ── the same roof in the other room ─────────────────────────────────────
  const flipped = await session.execute(function () {
    const b = document.querySelector('.theme-toggle');
    if (!b) return false;
    b.click();
    return true;
  });
  await wait(1200);
  const dark = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      theme: document.documentElement.getAttribute('data-theme') || 'light',
      desk: getComputedStyle(document.documentElement).getPropertyValue('--desk').trim(),
    });
  }));
  await shot(session, 'section-model-frame2-dark');

  check('the drawing follows the room it is in', () => {
    assert.ok(flipped, 'there is no theme toggle in the document bar');
    assert.equal(dark.theme, 'dark', `the window is in "${dark.theme}"`);
    assert.match(dark.desk, /^#/, `the ground token reads "${dark.desk}"`);
  });

  await session.execute(function () {
    const b = document.querySelector('.theme-toggle');
    if (b) b.click();
  });
  await wait(900);

  // ── nothing in the window is a word from the code ───────────────────────
  const words = await session.execute(function () {
    const el = document.querySelector('.model-editor');
    return el ? (el.innerText || el.textContent || '') : '';
  });
  check('no code word reached the screen', () => {
    for (const bad of ['TAPER', 'ELEV', 'BOARDS', 'SUMP', 'STRETCHOUT', 'undefined', 'NaN', '[object Object]']) {
      assert.ok(!words.includes(bad), `found "${bad}" on screen`);
    }
  });

  const errors = await errorsSoFar(session);
  check('and nothing errored while any of that happened', () => {
    assert.deepEqual(errors, [], errors.join(' | '));
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
