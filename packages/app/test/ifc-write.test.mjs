// ── The IFC writer, read back ──────────────────────────────────────────────
// The writer's output is bytes, and bytes are the one kind of output that can
// look finished and be wrong. So nothing here reads the writer's own account of
// what it did: every check opens the file again through web-ifc and asks the
// model what is in it.
//
//   node --test test/ifc-write.test.mjs
//
// The wasm is the one that came with the package, read off this disk. No
// network, here or anywhere in this program.

import './typescript.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import * as WebIFC from 'web-ifc';

// Imported after the hook above has been registered, which a static import
// cannot be: the whole graph is resolved before any of it runs.
const { roofPlan, writeIfc, writeRoof } = await import('../src/ifc/write.ts');

const ROOT = resolve(import.meta.dirname, '../../..');
const DEMO = join(ROOT, 'jobs/demo-job');

const api = new WebIFC.IfcAPI();
await api.Init();

/** The demo job as the shell hands it to a window: one key per file it came from. */
const demoJob = () => ({
  job: JSON.parse(readFileSync(join(DEMO, 'job.json'), 'utf8')),
  conditions: JSON.parse(readFileSync(join(DEMO, 'conditions.json'), 'utf8')),
  pages: JSON.parse(readFileSync(join(DEMO, 'pages/pages.json'), 'utf8')),
});

/** What the file says about itself, opened again from its own bytes. */
function readBack(bytes) {
  const id = api.OpenModel(bytes);
  const idsOf = (type) => {
    const v = api.GetLineIDsWithType(id, type);
    return Array.from({ length: v.size() }, (_, i) => v.get(i));
  };
  const out = {
    schema: api.GetModelSchema(id),
    lines: api.GetAllLines(id).size(),
    count: (type) => idsOf(type).length,
    lineOf: (type, n = 0) => api.GetLine(id, idsOf(type)[n]),
    every: (type) => idsOf(type).map((x) => api.GetLine(id, x)),
    close: () => api.CloseModel(id),
  };
  return out;
}

const say = (v) => (v && typeof v === 'object' && 'value' in v ? v.value : v);

test('a job with nothing in it still writes a file that opens', () => {
  const bytes = writeIfc({}, api);
  const back = readBack(bytes);
  assert.equal(back.schema, 'IFC4');
  assert.equal(back.count(WebIFC.IFCPROJECT), 1);
  assert.equal(back.count(WebIFC.IFCSITE), 1);
  assert.equal(back.count(WebIFC.IFCBUILDING), 1);
  assert.equal(back.count(WebIFC.IFCBUILDINGSTOREY), 1);
  assert.equal(back.count(WebIFC.IFCSLAB), 0, 'an empty job stood something up');
  back.close();
});

test('the demo job stands up as a roof slab and a parapet wall', () => {
  const doc = demoJob();
  const plan = roofPlan(doc);
  assert.ok(plan.slabs.length >= 1, 'no slab');
  assert.ok(plan.walls.length >= 1, 'no wall');

  const back = readBack(writeRoof(plan, api));
  assert.equal(back.count(WebIFC.IFCSLAB), plan.slabs.length);
  assert.equal(back.count(WebIFC.IFCWALL), plan.walls.length);
  assert.equal(back.count(WebIFC.IFCRELCONTAINEDINSPATIALSTRUCTURE), 1);
  assert.equal(back.count(WebIFC.IFCRELAGGREGATES), 3);

  const slab = back.lineOf(WebIFC.IFCSLAB);
  assert.equal(say(slab.PredefinedType), 'ROOF');
  assert.equal(say(slab.Name), plan.slabs[0].name);
  assert.equal(say(slab.Tag), plan.slabs[0].conditionId);

  const wall = back.lineOf(WebIFC.IFCWALL);
  assert.equal(say(wall.PredefinedType), 'PARAPET');
  assert.equal(say(wall.Tag), plan.walls[0].conditionId);
  back.close();
});

test('every condition is either stood up or named in what was left out', () => {
  const doc = demoJob();
  const plan = roofPlan(doc);
  const accounted = new Set([
    ...plan.slabs.map((s) => s.name),
    ...plan.walls.map((w) => w.name),
    ...plan.leftOut.map((l) => l.name),
  ]);
  for (const c of doc.conditions) {
    assert.ok(accounted.has(c.name), `${c.name} went missing between the job and the model`);
  }
  for (const l of plan.leftOut) {
    assert.ok(l.why.length > 10, `"${l.name}" was left out with no reason given`);
  }
});

test('a slab is as thick as the layers on its condition say, and no thicker', () => {
  const doc = demoJob();
  const plan = roofPlan(doc);
  const slab = plan.slabs[0];
  const condition = doc.conditions.find((c) => c.id === slab.conditionId);
  const stated = (condition.items ?? [])
    .filter((i) => i.thickness !== undefined)
    .reduce((sum, i) => sum + i.thickness, 0);
  assert.ok(stated > 0, 'the demo job has no stated build-up to check against');
  assert.equal(slab.thickness, stated / 12);

  const back = readBack(writeRoof({ ...plan, walls: [] }, api));
  const solid = back.lineOf(WebIFC.IFCEXTRUDEDAREASOLID);
  assert.equal(say(solid.Depth), stated / 12);
  back.close();
});

test('an area with no stated build-up is left out, and says why', () => {
  const doc = demoJob();
  const bare = {
    ...doc,
    conditions: doc.conditions.map((c) => (c.kind === 'area' ? { ...c, items: [] } : c)),
  };
  const plan = roofPlan(bare);
  assert.equal(plan.slabs.length, 0);
  for (const c of doc.conditions.filter((x) => x.kind === 'area')) {
    const said = plan.leftOut.find((l) => l.name === c.name);
    assert.ok(said, `${c.name} disappeared instead of being left out`);
    assert.match(said.why, /thick/);
  }
});

test('a trace on a sheet nobody has scaled is left out rather than written flat', () => {
  const doc = demoJob();
  const unscaled = { ...doc, pages: doc.pages.map((p) => ({ ...p, feetPerUnit: undefined })) };
  const plan = roofPlan(unscaled);
  assert.equal(plan.slabs.length, 0);
  assert.equal(plan.walls.length, 0);
  assert.ok(plan.leftOut.every((l) => /scale/.test(l.why)), JSON.stringify(plan.leftOut));
});

test('the file says its numbers are feet, against the metre', () => {
  const text = new TextDecoder().decode(writeIfc(demoJob(), api));
  assert.match(text, /IFCCONVERSIONBASEDUNIT\([^\n]*'FOOT'/);
  assert.match(text, /IFCMEASUREWITHUNIT\(IFCLENGTHMEASURE\(0\.3048\)/);
  assert.match(text, /IFCSIUNIT\(\*,\.LENGTHUNIT\.,\$,\.METRE\.\)/);
  assert.match(text, /FILE_SCHEMA\(\('IFC4'\)\)/);
});

test('the same roof writes the same file, bar the hour it was written', () => {
  const doc = demoJob();
  const strip = (b) => new TextDecoder().decode(b).replace(/^FILE_NAME\(.*$/m, '');
  assert.equal(strip(writeIfc(doc, api)), strip(writeIfc(doc, api)));
});

test('every identity in the file is a well-formed IFC one', () => {
  const back = readBack(writeIfc(demoJob(), api));
  const seen = new Set();
  for (const type of [WebIFC.IFCPROJECT, WebIFC.IFCSITE, WebIFC.IFCBUILDING,
    WebIFC.IFCBUILDINGSTOREY, WebIFC.IFCSLAB, WebIFC.IFCWALL]) {
    for (const line of back.every(type)) {
      const id = say(line.GlobalId);
      assert.equal(id.length, 22, `${id} is not 22 characters`);
      assert.match(id, /^[0-3][0-9A-Za-z_$]{21}$/, id);
      assert.ok(!seen.has(id), `two pieces of one roof share the identity ${id}`);
      seen.add(id);
    }
  }
  back.close();
});
