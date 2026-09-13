// ── The IFCX (alpha) bridge, read back ─────────────────────────────────────
// The same discipline as `ifc-write.test.mjs`: nothing here trusts the
// writer's own account of itself, every claim is checked against the parsed
// JSON, and every value is read back off the same `expectedOf` the landing
// chain's IfcOpenShell script checks the `.ifc` file against — so the two
// exports cannot state two different roofs.
//
//   node --test test/ifc-ifcx.test.mjs

import './typescript.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { demoJob, madeUpSolve } from './ifc-fixture.mjs';

const { roofPlan, expectedOf } = await import('../src/ifc/write.ts');
const { writeIfcx } = await import('../src/ifc/ifcx.ts');

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

function planOf() {
  const doc = demoJob();
  const { taper, walls, fastening } = madeUpSolve();
  return roofPlan(doc, taper, walls, fastening);
}

test('the document parses, and the header carries the alpha shape’s own fields, nothing else', () => {
  const plan = planOf();
  const document = JSON.parse(writeIfcx(plan));
  assert.equal(document.header.ifcxVersion, 'ifcx_alpha');
  assert.match(document.header.id, UUID);
  assert.ok(document.header.dataVersion);
  assert.ok(document.header.timestamp);
  // hello-wall.ifcx's own header carries exactly these five keys — no
  // description, sourceFile or generator.
  assert.deepEqual(
    Object.keys(document.header).sort(),
    ['author', 'dataVersion', 'id', 'ifcxVersion', 'timestamp'],
  );
  assert.ok(Array.isArray(document.imports) && document.imports.length > 0);
  assert.ok(Array.isArray(document.data));
});

test('the spatial tree is there, and children resolve to a real node each', () => {
  const plan = planOf();
  const document = JSON.parse(writeIfcx(plan));
  const byPath = new Map(document.data.map((n) => [n.path, n]));

  const roots = document.data.filter((n) => n.children && !document.data.some(
    (other) => other.children && Object.values(other.children).includes(n.path),
  ));
  assert.equal(roots.length, 1, 'exactly one node should have no parent pointing at it: the project');

  let node = roots[0];
  let depth = 0;
  const chain = [node];
  while (node.children && Object.keys(node.children).length === 1 && depth < 10) {
    const childPath = Object.values(node.children)[0];
    node = byPath.get(childPath);
    assert.ok(node, 'a child key points at a path with no node');
    chain.push(node);
    depth += 1;
  }
  assert.ok(chain.length >= 4, 'expected project → site → building → storey at least');
});

test('every path is a well-formed UUID, and none collides', () => {
  const plan = planOf();
  const document = JSON.parse(writeIfcx(plan));
  const seen = new Set();
  for (const n of document.data) {
    assert.match(n.path, UUID, n.path);
    assert.ok(!seen.has(n.path), `two nodes share the path ${n.path}`);
    seen.add(n.path);
  }
});

test('every entry expectedOf names is on the document, under pset::<set>::<key>', () => {
  const plan = planOf();
  const expected = expectedOf(plan);
  const document = JSON.parse(writeIfcx(plan));

  const byNameAndClass = new Map();
  for (const n of document.data) {
    const cls = n.attributes && n.attributes['bsi::ifc::class'];
    const name = n.attributes && n.attributes['nlsfb::name'];
    if (cls && name) byNameAndClass.set(`${cls.code}:${name}`, n);
  }

  for (const e of expected.entries) {
    const node = byNameAndClass.get(`${e.class}:${e.name}`);
    assert.ok(node, `${e.name} (${e.class}) is not on the ifcx document`);
    const key = `pset::${e.set}::${e.key}`;
    assert.equal(node.attributes[key], e.value, `${e.name} ${key}`);
  }
});

test('Pset_Authority goes through the ordinary pset:: mapping, not a same-path override', () => {
  const plan = planOf();
  const document = JSON.parse(writeIfcx(plan));
  const slabNode = document.data.find(
    (n) => n.attributes && n.attributes['nlsfb::name'] === plan.slabs[0].name
      && n.attributes['bsi::ifc::class']?.code === 'IFCSLAB',
  );
  assert.ok(slabNode);
  assert.equal(slabNode.attributes['pset::Pset_Authority::hier'], 'FIELD');
  assert.ok(!('buildos::authority::weight' in slabNode.attributes),
    'this port does not attempt ifc5-bridge’s own weighted same-path scheme');
});

test('the same plan writes the same document, every time — the timestamp is fixed, not read off the clock', () => {
  const plan = planOf();
  assert.equal(writeIfcx(plan), writeIfcx(plan));
});
