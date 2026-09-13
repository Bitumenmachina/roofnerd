// ── C7: File → Export IFC lands a file that reopens with the same counts ───
//
// Patrick's rule: open source, import/export, geometry — the three things
// this program exists for. This is the check for the second one, on the
// third: the whole solved roof, exported as IFC4, reopened with web-ifc IN
// THIS PROBE (never through the program that wrote it) and held against the
// SAME solve read straight off the window with `solve_get` — not a number
// typed in here, and not the export's own account of itself.
//
//   THE FILE LANDS. `jobs/demo-job/exports/<slug>-<date>.ifc` exists after the
//   press, `<slug>-<date>.expected.json` and `<slug>-<date>.ifcx` beside it.
//
//   THE COUNTS AGREE. The tapered field's own `IfcSlab`, tagged `c-field`,
//   is in the file exactly once, and its `Qto_SlabBaseQuantities.NetArea`
//   is the field's own `squareFeet` off `solve_get('taper')` — not a figure
//   copied in here, which would go stale the day the demo is retraced. Every
//   drain point the job traces has an `IfcFlowTerminal`; every wall or curb
//   `solve_get('walls')` gives a real inboard offset to has an `IfcWall`.
//
//   THE STATUS STRIP NAMES THE FILE. "Exported …", the same sentence a hand
//   would read after pressing the menu item.
//
// Front door throughout: the start screen, the demo button, the File menu,
// the menu item. Nothing here calls a command to write the file — that is
// what pressing "Export IFC" is for.
//
//   pnpm build:app && node tools/probe/section-ifc.mjs
//
// The orchestrator runs `tools/verify-ifc.py` from `~/venvs/ifc` afterward,
// against the file and `.expected.json` this press writes, and records its
// rows in STATUS — that is IfcOpenShell's own check and does not belong in a
// Node probe.

import {
  mkdir, readdir, readFile, rm, writeFile,
} from 'node:fs/promises';
import { realpathSync } from 'node:fs';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import * as WebIFC from 'web-ifc';
import {
  centreOf, errorsSoFar, launch, openDemoJob, until, wait, watchErrors, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

// `demo_folder` (`src-tauri/src/lib.rs`) resolves against the source tree at
// build time and answers this path in a checkout — the same one
// `section9a-export.mjs` already reads the CSV lenses out of.
const JOB = realpathSync(join(ROOT, 'jobs/demo-job'));
const EXPORTS = join(JOB, 'exports');

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const shot = async (session, name) =>
  writeFile(join(EVIDENCE, `${name}-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

/**
 * The name `exportIfc` (`packages/app/src/ifc/export.ts`) gives the file —
 * the job's own name, slugged, and today's date the same way the estimator's
 * clock reads it. Reproduced rather than imported: a probe reads the product
 * the way a hand would find the file, by its name on disk, not by importing
 * the function that chose it.
 */
const today = () => {
  const now = new Date();
  const two = (n) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${two(now.getMonth() + 1)}-${two(now.getDate())}`;
};
const slug = (name) => {
  const safe = name.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
  return safe || 'roof';
};

/** The named numbers and the geometry the shell solved, pulled the way the app pulls them. */
const solvePart = (session, part) => session.execute(function (want) {
  return window.__TAURI_INTERNALS__.invoke('solve_get', { part: want })
    .then(function (v) { return JSON.stringify(v); })
    .catch(function (e) { return JSON.stringify({ error: String(e) }); });
}, part);

/** The document itself — the conditions, to count how many drain points are traced. */
const docNow = (session) => session.execute(function () {
  return window.__TAURI_INTERNALS__.invoke('doc_get')
    .then(function (d) { return JSON.stringify(d); })
    .catch(function (e) { return JSON.stringify({ error: String(e) }); });
});

/** Open a menu by the word on its own button, then press the item with this word on it. */
const pressMenuItem = async (session, menuLabel, itemLabel) => {
  const opened = await session.execute(function (want) {
    const b = [...document.querySelectorAll('.menu-button')]
      .find(function (e) { return (e.textContent || '').trim() === want; });
    if (!b) return false;
    b.click();
    return true;
  }, menuLabel);
  await wait(300);
  const pressed = await session.execute(function (want) {
    const item = [...document.querySelectorAll('.menu-item')]
      .find(function (e) { return (e.textContent || '').trim() === want; });
    if (!item) return false;
    item.click();
    return true;
  }, itemLabel);
  await wait(300);
  return { opened, pressed };
};

/** The status strip's own sentence, the way an estimator reads it. */
const statusNow = (session) => session.execute(function () {
  const bar = document.querySelector('.status-strip');
  return JSON.stringify({
    text: bar ? (bar.textContent || '') : '',
    seen: bar ? (bar.innerText || '') : '',
  });
});

await mkdir(EVIDENCE, { recursive: true });
await mkdir(EXPORTS, { recursive: true });

const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  // ── the front door ──────────────────────────────────────────────────────
  const demoAt = await centreOf(session, '.start-demo');
  check('the demo button is on the start screen', () => assert.ok(demoAt));
  await openDemoJob(session);
  await until(session, () => document.querySelector('.view-switch'), { what: 'the job to open' });
  await wait(1200);

  // ── the job's own name decides the file the estimator is about to find ──
  const doc = JSON.parse(await docNow(session));
  const jobName = doc?.job?.name;
  check('the open job names itself', () => assert.ok(typeof jobName === 'string' && jobName.length > 0));
  const stem = `${slug(jobName)}-${today()}`;
  const ifcPath = join(EXPORTS, `${stem}.ifc`);
  const expectedPath = join(EXPORTS, `${stem}.expected.json`);
  const ifcxPath = join(EXPORTS, `${stem}.ifcx`);

  // A clean slate for exactly the files this press writes — the folder is
  // tracked, so it is emptied of THESE names, never removed
  // (`section9a-export.mjs`'s own rule, for its own file names).
  for (const f of await readdir(EXPORTS)) {
    if (f === `${stem}.ifc` || f === `${stem}.expected.json` || f === `${stem}.ifcx`) {
      await rm(join(EXPORTS, f), { force: true });
    }
  }

  // ── the solve, read the way the app itself would read it before writing ──
  const taper = JSON.parse(await solvePart(session, 'taper'));
  const walls = JSON.parse(await solvePart(session, 'walls'));
  check('the taper and the walls are solved before anything is exported', () => {
    assert.ok(Array.isArray(taper?.fields) && taper.fields.length > 0, 'no tapered field solved');
    assert.ok(Array.isArray(walls?.runs) && walls.runs.length > 0, 'no wall or curb run solved');
  });
  const field = taper.fields.find((f) => f.conditionId === 'c-field');
  check('the field this job traces is the one the demo names', () => assert.ok(field));

  const buildableWalls = walls.runs.filter((r) => r.wallInches !== null && r.inboard);
  const drainConditions = (doc.conditions ?? []).filter((c) => c.kind === 'count' && c.role === 'drain' && !c.hidden);
  const drainPoints = drainConditions.reduce((n, c) => n + (c.traces ?? []).reduce((m, t) => m + t.points.length, 0), 0);

  // ── the menu, pressed the way a hand presses it ─────────────────────────
  const { opened, pressed } = await pressMenuItem(session, 'File', 'Export IFC');
  check('the File menu opens', () => assert.ok(opened, 'no button reads "File"'));
  check('Export IFC is on it', () => assert.ok(pressed, 'no item reads "Export IFC"'));

  await until(session, () => (document.querySelector('.status-strip')?.textContent || '').includes('Exported'),
    { what: 'the status strip to say Exported', timeout: 15000 });
  await wait(500);
  await shot(session, 'section-ifc-status');

  const status = JSON.parse(await statusNow(session));
  check('the status strip names the file it wrote', () => {
    assert.match(status.text, /Exported/, `the strip reads "${status.text.trim()}"`);
    assert.ok(status.text.includes(`${stem}.ifc`), `the strip does not name ${stem}.ifc: "${status.text.trim()}"`);
    assert.match(status.seen, /Exported/, `nothing of it is rendered: "${status.seen.trim()}"`);
  });

  // ── the file, read with Node's own fs, reopened with web-ifc IN THIS PROBE ──
  const ifcText = await readFile(ifcPath, 'utf8').catch(() => null);
  check(`${stem}.ifc is in the job's own exports folder`, () => assert.ok(ifcText, `nothing at ${ifcPath}`));
  const expectedText = await readFile(expectedPath, 'utf8').catch(() => null);
  check(`${stem}.expected.json is beside it`, () => assert.ok(expectedText, `nothing at ${expectedPath}`));
  const ifcxText = await readFile(ifcxPath, 'utf8').catch(() => null);
  check(`${stem}.ifcx is beside it too`, () => assert.ok(ifcxText, `nothing at ${ifcxPath}`));

  let expected = null;
  check('the expected.json parses', () => {
    expected = JSON.parse(expectedText);
    assert.ok(expected.job && expected.counts && Array.isArray(expected.entries));
  });

  const api = new WebIFC.IfcAPI();
  await api.Init();
  let modelId = null;
  check('web-ifc reopens the bytes', () => {
    modelId = api.OpenModel(new TextEncoder().encode(ifcText));
    assert.equal(api.GetModelSchema(modelId), 'IFC4');
  });

  const idsOf = (type) => {
    const v = api.GetLineIDsWithType(modelId, type);
    return Array.from({ length: v.size() }, (_, i) => v.get(i));
  };
  const lineOf = (id) => api.GetLine(modelId, id);
  const say = (v) => (v && typeof v === 'object' && 'value' in v ? v.value : v);

  check('the field this job traces is one IfcSlab, tagged c-field', () => {
    const slabs = idsOf(WebIFC.IFCSLAB).map(lineOf);
    const mine = slabs.filter((s) => say(s.Tag) === 'c-field');
    assert.equal(mine.length, 1, `${mine.length} slabs carry the field's own tag`);
    assert.equal(say(mine[0].PredefinedType), 'ROOF');
  });

  check('its NetArea is the solve’s own square feet, not a re-sum', () => {
    const slab = idsOf(WebIFC.IFCSLAB).map(lineOf).find((s) => say(s.Tag) === 'c-field');
    const rels = idsOf(WebIFC.IFCRELDEFINESBYPROPERTIES).map(lineOf)
      .filter((r) => (r.RelatedObjects || []).some((o) => say(o) === slab.expressID));
    const qto = rels.map((r) => lineOf(say(r.RelatingPropertyDefinition)))
      .find((d) => say(d.Name) === 'Qto_SlabBaseQuantities');
    assert.ok(qto, 'no Qto_SlabBaseQuantities on the field’s slab');
    const netArea = qto.Quantities.map((q) => lineOf(say(q))).find((q) => say(q.Name) === 'NetArea');
    assert.ok(netArea, 'no NetArea quantity');
    const got = say(netArea.AreaValue);
    assert.ok(Math.abs(got - field.squareFeet) < 1e-6,
      `the file says ${got} sq ft and solve_get('taper') says ${field.squareFeet}`);
  });

  check('every drain point traced has an IfcFlowTerminal', () => {
    assert.equal(idsOf(WebIFC.IFCFLOWTERMINAL).length, drainPoints,
      `${idsOf(WebIFC.IFCFLOWTERMINAL).length} in the file, ${drainPoints} traced on the job`);
  });

  check('every wall or curb the solve gave a real inboard offset to is an IfcWall', () => {
    assert.equal(idsOf(WebIFC.IFCWALL).length, buildableWalls.length,
      `${idsOf(WebIFC.IFCWALL).length} in the file, ${buildableWalls.length} buildable in the solve`);
  });

  check('the counts in expected.json are the counts the file itself carries', () => {
    for (const [cls, want] of Object.entries(expected.counts)) {
      const type = WebIFC[cls];
      assert.ok(type !== undefined, `expected.json names a class this probe does not know: ${cls}`);
      assert.equal(idsOf(type).length, want, `${cls}: file has ${idsOf(type).length}, expected.json says ${want}`);
    }
  });

  api.CloseModel(modelId);

  check('the ifcx document parses and names the same job', () => {
    const ifcx = JSON.parse(ifcxText);
    assert.equal(ifcx.header.ifcxVersion, 'ifcx_alpha');
    assert.ok(Array.isArray(ifcx.data) && ifcx.data.length > 0);
  });

  // ── the picture: the File menu open, the export on it ───────────────────
  await session.execute(function (want) {
    const b = [...document.querySelectorAll('.menu-button')]
      .find(function (e) { return (e.textContent || '').trim() === want; });
    if (b) b.click();
  }, 'File');
  await wait(300);
  await shot(session, 'section-ifc-menu');
  await session.execute(function () {
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
  });

  const errors = await errorsSoFar(session);
  check('and nothing in the window complained on the way through', () => {
    assert.deepEqual(errors, [], `the window logged ${JSON.stringify(errors)}`);
  });

  console.log(`\n  ${results.filter((r) => r === 'PASS').length}/${results.length}`);
  console.log(`  file: ${ifcPath}`);
  console.log(`  run against it: ~/venvs/ifc/bin/python tools/verify-ifc.py ${ifcPath} ${expectedPath}`);
  console.log(`  pictures: evidence/section-ifc-status-${COMMIT}.png, evidence/section-ifc-menu-${COMMIT}.png`);
} catch (e) {
  console.log(`  FAIL  the probe itself: ${e.stack || e.message}`);
  results.push('FAIL');
} finally {
  await app.close().catch(() => undefined);
}

process.exit(results.includes('FAIL') ? 1 : 0);
