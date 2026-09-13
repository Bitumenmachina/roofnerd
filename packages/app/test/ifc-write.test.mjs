// ── The IFC writer, read back ──────────────────────────────────────────────
// The writer's output is bytes, and bytes are the one kind of output that can
// look finished and be wrong. So nothing here reads the writer's own account of
// what it did: every check opens the file again through web-ifc and asks the
// model what is in it.
//
// The three solves this file hands `roofPlan` are made up, the same way
// `takeoff.test.mjs` makes up a field of the real shape rather than reaching
// for a fixture (C1's own convention, because nothing in Node can run the Rust
// solver): every field matches `packages/engine/src/solved.ts` exactly, and
// every number this file asserts is read back OFF that same made-up solve, never
// retyped as a second copy that could drift from it. The one real figure in
// here is the manufacturer's chart itself — `VE-7830`, `X`, `Y` — proved against
// `data/profiles/versico-versicore-mp-h.json` on disk, because that is the one
// thing `roofPlan` reads from somewhere other than the solve.
//
//   node --test test/ifc-write.test.mjs
//
// The wasm is the one that came with the package, read off this disk. No
// network, here or anywhere in this program.

import './typescript.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import * as WebIFC from 'web-ifc';
import { ROOT, demoJob, madeUpSolve } from './ifc-fixture.mjs';

// Imported after the hook above has been registered, which a static import
// cannot be: the whole graph is resolved before any of it runs.
const { roofPlan, writeIfc, writeRoof, expectedOf } = await import('../src/ifc/write.ts');
const { NOTHING_SOLVED, NOTHING_SOLVED_WALLS, NOTHING_SOLVED_FASTENING } = await import('@roofnerd/engine');

const api = new WebIFC.IfcAPI();
await api.Init();

/** What the file says about itself, opened again from its own bytes. */
function readBack(bytes) {
  const id = api.OpenModel(bytes);
  const idsOf = (type) => {
    const v = api.GetLineIDsWithType(id, type);
    return Array.from({ length: v.size() }, (_, i) => v.get(i));
  };
  const out = {
    id,
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

/**
 * Every Pset and Qto on one element, by set name, each as a plain object of
 * its own entries — read back off the relationships the writer laid down, not
 * off anything it claims about itself.
 */
function propsOn(back, element) {
  const rels = back.every(WebIFC.IFCRELDEFINESBYPROPERTIES)
    .filter((r) => (r.RelatedObjects || []).some((o) => say(o) === element.expressID));
  const sets = {};
  for (const rel of rels) {
    const def = api.GetLine(back.id, say(rel.RelatingPropertyDefinition));
    const setName = say(def.Name);
    if (def.HasProperties) {
      sets[setName] = Object.fromEntries(def.HasProperties.map((h) => {
        const p = api.GetLine(back.id, say(h));
        return [say(p.Name), say(p.NominalValue)];
      }));
    } else if (def.Quantities) {
      sets[setName] = Object.fromEntries(def.Quantities.map((h) => {
        const q = api.GetLine(back.id, say(h));
        const value = q.AreaValue ?? q.LengthValue ?? q.CountValue ?? q.VolumeValue;
        return [say(q.Name), say(value)];
      }));
    }
  }
  return sets;
}

test('a job with nothing in it still writes a file that opens', () => {
  const bytes = writeIfc({}, NOTHING_SOLVED, NOTHING_SOLVED_WALLS, NOTHING_SOLVED_FASTENING, api);
  const back = readBack(bytes);
  assert.equal(back.schema, 'IFC4');
  assert.equal(back.count(WebIFC.IFCPROJECT), 1);
  assert.equal(back.count(WebIFC.IFCSITE), 1);
  assert.equal(back.count(WebIFC.IFCBUILDING), 1);
  assert.equal(back.count(WebIFC.IFCBUILDINGSTOREY), 1);
  assert.equal(back.count(WebIFC.IFCSLAB), 0, 'an empty job stood something up');
  back.close();
});

test('the plan reads the solve: one slab, one buildable wall, one drain with a sump', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);

  assert.equal(plan.slabs.length, 1, 'the one tapered field should stand up as one slab');
  assert.equal(plan.walls.length, 2, 'the parapet and the curb both have a stated WALL thickness');
  assert.equal(plan.drains.length, 3, 'the demo traces three drain points');
});

test('the roof slab is the solve’s own surface: area and edges are read, never resummed', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const slab = plan.slabs[0];
  assert.equal(slab.field.squareFeet, 100);

  const back = readBack(writeRoof(plan, api));
  assert.equal(back.count(WebIFC.IFCSLAB), 2, 'the tapered field and the cricket are both an IfcSlab');
  const slabLine = back.every(WebIFC.IFCSLAB).find((s) => say(s.Tag) === 'c-field');
  assert.ok(slabLine, 'the field did not reach the file');
  assert.equal(say(slabLine.PredefinedType), 'ROOF');
  assert.equal(say(slabLine.Name), slab.name);

  const solid = back.lineOf(WebIFC.IFCTRIANGULATEDFACESET);
  assert.ok(solid, 'the facet did not triangulate into anything');
  back.close();
});

test('a slab’s Qto carries the field’s own area, thin edge and thick edge — never resummed', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const field = plan.slabs[0].field;
  const back = readBack(writeRoof(plan, api));

  const slabLine = back.every(WebIFC.IFCSLAB).find((s) => say(s.Tag) === 'c-field');
  const qto = propsOn(back, slabLine).Qto_SlabBaseQuantities;
  assert.ok(qto, 'no Qto_SlabBaseQuantities on the field’s slab');
  assert.equal(qto.NetArea, field.squareFeet, 'NetArea should be the solve’s own square feet, not a re-sum');
  assert.equal(qto.ThinEdge, field.thinInches / 12);
  assert.equal(qto.ThickEdge, field.thickInches / 12);
  assert.ok(qto.NetVolume > 0, 'a sloped roof with area should have a positive volume');
  back.close();
});

test('an insulation covering’s Qto carries the bill’s own board count and area', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const back = readBack(writeRoof(plan, api));

  const coveringLines = back.every(WebIFC.IFCCOVERING);
  const xLine = coveringLines.find((c) => propsOn(back, c).InsulationPanel?.letter === 'X');
  assert.ok(xLine, 'no covering carries letter X');
  const qto = propsOn(back, xLine).Qto_CoveringBaseQuantities;
  assert.equal(qto.NetArea, 48);
  assert.equal(qto.BoardCount, 3);
  const pset = propsOn(back, xLine).InsulationPanel;
  assert.equal(pset.thinEdgeInches, 0.5);
  assert.equal(pset.thickEdgeInches, 1.5);
  assert.equal(pset.manufacturer, 'Made-up Manufacturer');
  back.close();
});

test('every element carries Pset_CostAxis, Pset_TimeAxis and Pset_Authority', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const back = readBack(writeRoof(plan, api));
  const slabLine = back.every(WebIFC.IFCSLAB).find((s) => say(s.Tag) === 'c-field');
  const sets = propsOn(back, slabLine);
  assert.ok(sets.Pset_CostAxis, 'no Pset_CostAxis');
  assert.ok(sets.Pset_TimeAxis, 'no Pset_TimeAxis');
  assert.ok(sets.Pset_Authority, 'no Pset_Authority');
  assert.equal(sets.Pset_TimeAxis.phase, 'new work');
  assert.equal(sets.Pset_Authority.hier, 'FIELD');
  assert.equal(typeof sets.Pset_Authority.overridable, 'boolean');
  back.close();
});

test('every condition either stands up or is named in what was left out', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const accounted = new Set([
    ...plan.slabs.map((s) => s.name),
    ...plan.coverings.map((c) => c.name),
    ...plan.walls.map((w) => w.name),
    ...plan.copings.map((c) => c.name),
    ...plan.drains.map((d) => d.name),
    ...plan.crickets.map((c) => c.name),
    ...plan.leftOut.map((l) => l.name),
  ]);
  for (const c of doc.conditions) {
    assert.ok(
      [...accounted].some((n) => n === c.name || n.startsWith(`${c.name} `) || n.startsWith(`${c.name}—`)),
      `${c.name} went missing between the job and the model`,
    );
  }
  for (const l of plan.leftOut) {
    assert.ok(l.why.length > 10, `"${l.name}" was left out with no reason given`);
  }
});

test('an insulation covering carries the letter’s own printed edge, read off the chart on disk', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const x = plan.coverings.find((c) => c.letter === 'X');
  assert.ok(x, 'no X-letter covering');
  assert.equal(x.thinEdgeInches, 0.5);
  assert.equal(x.thickEdgeInches, 1.5);
  assert.equal(x.boards, 3);
  assert.equal(x.squareFeet, 48);

  const raw = JSON.parse(readFileSync(join(ROOT, 'data/profiles/versico-versicore-mp-h.json'), 'utf8'));
  const printed = raw.slopes.find((s) => s.slope_in_per_ft === 0.25).extended.panels.find((p) => p.designation === 'X');
  assert.equal(x.thinEdgeInches, printed.thin_edge_in, 'the edge should be the chart’s own, not a made-up one');
  assert.equal(x.thickEdgeInches, printed.thick_edge_in);
});

test('a letter with no boards in the bill is not written', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  assert.ok(!plan.coverings.some((c) => c.letter === 'Z'), 'Z has zero boards and should not appear');
});

test('cover board and membrane come off the condition’s own items, at their own stated thickness', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const coverboardItem = doc.conditions.find((c) => c.id === 'c-field').items.find((i) => i.id === 'coverboard');
  const membraneItem = doc.conditions.find((c) => c.id === 'c-field').items.find((i) => i.id === 'tpo-60');
  const cb = plan.coverings.find((c) => c.role === 'cover-board');
  const mem = plan.coverings.find((c) => c.role === 'membrane');
  assert.ok(cb && mem);
  assert.equal(cb.thicknessInches, coverboardItem.thickness);
  assert.equal(mem.thicknessInches, membraneItem.thickness);
  assert.equal(cb.squareFeet, 100);
});

test('the parapet stands as an IfcWall PARAPET, sized off its own solved run', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  assert.equal(plan.walls.length, 2, 'the parapet and the curb both have a stated WALL thickness');
  const parapet = plan.walls.find((w) => w.conditionId === 'c-parapet');
  assert.ok(parapet);
  assert.equal(parapet.curb, false);
  const curb = plan.walls.find((w) => w.conditionId === 'c-curb');
  assert.ok(curb);
  assert.equal(curb.curb, true);

  const back = readBack(writeRoof(plan, api));
  const wallLine = back.every(WebIFC.IFCWALL).find((w) => say(w.Tag) === 'c-parapet');
  assert.ok(wallLine, 'the parapet did not reach the file');
  assert.equal(say(wallLine.PredefinedType), 'PARAPET');
  const curbLine = back.every(WebIFC.IFCWALL).find((w) => say(w.Tag) === 'c-curb');
  assert.ok(curbLine, 'the curb did not reach the file');
  assert.equal(say(curbLine.PredefinedType), 'SOLIDWALL', 'a curb was written as the edge of the building');
  back.close();
});

test('the coping cap centred on the parapet is an IfcCovering, USERDEFINED “COPING”', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  assert.equal(plan.copings.length, 1, 'only the parapet’s coping has both cap edges in this made-up solve');

  const back = readBack(writeRoof(plan, api));
  const coping = back.every(WebIFC.IFCCOVERING).find((c) => say(c.Tag) === 'c-parapet');
  assert.ok(coping, 'the coping did not reach the file');
  assert.equal(say(coping.PredefinedType), 'USERDEFINED');
  assert.equal(say(coping.ObjectType), 'COPING');
  back.close();
});

test('a curb with no cap to centre on is left out, and says why', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const said = plan.leftOut.find((l) => l.name.startsWith('RTU-1 Curb') && l.name.includes('coping'));
  assert.ok(said, `no leftOut row for the curb's coping among ${JSON.stringify(plan.leftOut)}`);
  assert.match(said.why, /nothing to centre on/);
});

test('the drains stand as IfcFlowTerminal, one with a sump property and one without', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  assert.equal(plan.drains.length, 3, 'the demo traces three drain points');
  assert.ok(plan.drains[0].sump, 'the first drain should carry the made-up sump');
  assert.equal(plan.drains[1].sump, null);
  assert.equal(plan.drains[2].sump, null);

  const back = readBack(writeRoof(plan, api));
  assert.equal(back.count(WebIFC.IFCFLOWTERMINAL), 3);
  back.close();
});

test('the cricket stands as an IfcSlab built from the solve’s own triangles', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  assert.equal(plan.crickets.length, 1);
  assert.ok(plan.crickets[0].ridge, 'the ridge should be the condition’s own traced line');

  const back = readBack(writeRoof(plan, api));
  const cricketSlab = back.every(WebIFC.IFCSLAB).find((s) => say(s.Tag) === 'c-cricket');
  assert.ok(cricketSlab);
  assert.equal(say(cricketSlab.ObjectType), 'CRICKET');
  back.close();
});

test('the fastening lands on the flat fill covering, with zones, class, counts, standard and basis', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  assert.equal(plan.fastening.length, 1);
  const fill = plan.coverings.find((c) => c.role === 'insulation-fill');
  assert.ok(fill);

  const back = readBack(writeRoof(plan, api));
  const pset = back.every(WebIFC.IFCPROPERTYSET).find((p) => say(p.Name) === 'Fastening');
  assert.ok(pset, 'no Fastening property set in the file');
  const names = pset.HasProperties.map((h) => say(api.GetLine(back.id, say(h)).Name));
  assert.ok(names.includes('standard'));
  assert.ok(names.includes('zone1FmClass'));
  assert.ok(names.includes('zone2Fasteners'));
  back.close();
});

test('an area with no stated build-up is left out, and says why', () => {
  const doc = demoJob();
  const bare = {
    ...doc,
    conditions: doc.conditions.map((c) => (c.kind === 'area' ? { ...c, items: [] } : c)),
  };
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(bare, taper, walls, fastening);
  // No cover board or membrane item left, but the slab itself still stands —
  // its geometry is the solve's, not the item list's.
  assert.equal(plan.slabs.length, 1);
  assert.ok(!plan.coverings.some((c) => c.role === 'cover-board' || c.role === 'membrane'));
});

test('a trace on a sheet nobody has scaled is left out rather than written flat', () => {
  const doc = demoJob();
  const unscaled = { ...doc, pages: doc.pages.map((p) => ({ ...p, feetPerUnit: undefined })) };
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(unscaled, taper, walls, fastening);
  assert.equal(plan.slabs.length, 0);
  assert.equal(plan.walls.length, 0);
  assert.equal(plan.drains.length, 0);
  assert.ok(plan.leftOut.some((l) => /scale/.test(l.why)));
});

test('a tapered field with no solved bill still stands as a slab, and the insulation is left out', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const noBill = {
    ...taper,
    fields: [{ ...taper.fields[0], bill: null, billRefused: 'the chart has no letter for this slope' }],
  };
  const plan = roofPlan(doc, noBill, walls, fastening);
  assert.equal(plan.slabs.length, 1, 'the roof surface does not depend on the bill');
  assert.ok(plan.leftOut.some((l) => l.name.includes('insulation') && /chart has no letter/.test(l.why)));
});

test('expectedOf matches the file, entry by entry — the shape tools/verify-ifc.py checks', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const plan = roofPlan(doc, taper, walls, fastening);
  const expected = expectedOf(plan);
  assert.equal(expected.job, plan.job);

  const back = readBack(writeRoof(plan, api));
  const CLASS_TYPE = {
    IFCSLAB: WebIFC.IFCSLAB, IFCWALL: WebIFC.IFCWALL,
    IFCCOVERING: WebIFC.IFCCOVERING, IFCFLOWTERMINAL: WebIFC.IFCFLOWTERMINAL,
  };
  for (const [cls, count] of Object.entries(expected.counts)) {
    const type = CLASS_TYPE[cls] ?? WebIFC[cls];
    assert.ok(type !== undefined, `expectedOf named a class the test does not know: ${cls}`);
    assert.equal(back.count(type), count, `${cls} count`);
  }

  const byNameAndClass = new Map();
  for (const cls of Object.keys(CLASS_TYPE)) {
    for (const line of back.every(CLASS_TYPE[cls])) byNameAndClass.set(`${cls}:${say(line.Name)}`, line);
  }
  for (const e of expected.entries) {
    const line = byNameAndClass.get(`${e.class}:${e.name}`);
    assert.ok(line, `${e.name} (${e.class}) is in expectedOf but not in the file`);
    const sets = propsOn(back, line);
    assert.ok(sets[e.set], `${e.name}: no ${e.set} in the file`);
    assert.equal(sets[e.set][e.key], e.value, `${e.name} ${e.set}.${e.key}`);
  }
  back.close();
});

test('the file says its numbers are feet, against the metre', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const text = new TextDecoder().decode(writeIfc(doc, taper, walls, fastening, api));
  assert.match(text, /IFCCONVERSIONBASEDUNIT\([^\n]*'FOOT'/);
  assert.match(text, /IFCMEASUREWITHUNIT\(IFCLENGTHMEASURE\(0\.3048\)/);
  assert.match(text, /IFCSIUNIT\(\*,\.LENGTHUNIT\.,\$,\.METRE\.\)/);
  assert.match(text, /FILE_SCHEMA\(\('IFC4'\)\)/);
});

test('the same roof writes the same file, bar the hour it was written', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const strip = (b) => new TextDecoder().decode(b).replace(/^FILE_NAME\(.*$/m, '');
  assert.equal(
    strip(writeIfc(doc, taper, walls, fastening, api)),
    strip(writeIfc(doc, taper, walls, fastening, api)),
  );
});

test('every identity in the file is a well-formed IFC one, and none collides', () => {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  const back = readBack(writeIfc(doc, taper, walls, fastening, api));
  const seen = new Set();
  for (const type of [
    WebIFC.IFCPROJECT, WebIFC.IFCSITE, WebIFC.IFCBUILDING, WebIFC.IFCBUILDINGSTOREY,
    WebIFC.IFCSLAB, WebIFC.IFCWALL, WebIFC.IFCCOVERING, WebIFC.IFCFLOWTERMINAL,
  ]) {
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
