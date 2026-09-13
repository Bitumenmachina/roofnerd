// ── The triangulator, on its own ───────────────────────────────────────────
// Ear clipping is the one piece of graphics the IFC writer needs and the one
// most likely to be wrong in a way nothing else here would catch — a bad
// triangle still opens, still has a Depth, and still passes every count. So it
// is proven here, on shapes small enough to check by hand, before anything in
// `write.ts` trusts it with a facet `geo` handed back.
//
//   node --test test/ifc-triangulate.test.mjs

import './typescript.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';

const { triangulateShape, signedArea2 } = await import('../src/ifc/triangulate.ts');

const area = (tri) => Math.abs(signedArea2(tri)) / 2;
const totalArea = (tris) => tris.reduce((sum, t) => sum + area(t), 0);

test('a square becomes two triangles, and they cover it exactly', () => {
  const square = { ring: [{ x: 0, y: 0 }, { x: 4, y: 0 }, { x: 4, y: 4 }, { x: 0, y: 4 }], holes: [] };
  const tris = triangulateShape(square);
  assert.equal(tris.length, 2, `a quad should ear-clip to two triangles, got ${tris.length}`);
  assert.ok(Math.abs(totalArea(tris) - 16) < 1e-9, `the triangles cover ${totalArea(tris)} sq ft, not 16`);
});

test('a triangle stays one triangle', () => {
  const tri = { ring: [{ x: 0, y: 0 }, { x: 6, y: 0 }, { x: 0, y: 3 }], holes: [] };
  const tris = triangulateShape(tri);
  assert.equal(tris.length, 1);
  assert.ok(Math.abs(totalArea(tris) - 9) < 1e-9);
});

test('a concave L-shape triangulates without a triangle stepping outside it', () => {
  // The field's own bump is this shape: a square with a bite taken out of one
  // corner, the same concavity C3's notes proved the wall offset against.
  const l = {
    ring: [
      { x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 4 },
      { x: 4, y: 4 }, { x: 4, y: 10 }, { x: 0, y: 10 },
    ],
    holes: [],
  };
  const tris = triangulateShape(l);
  const trueArea = 10 * 10 - 6 * 6; // the 10x10 square less the bitten corner
  assert.ok(Math.abs(totalArea(tris) - trueArea) < 1e-6,
    `the L is ${trueArea} sq ft and the triangles come to ${totalArea(tris)}`);
  // No triangle here should be bigger than the whole shape — the signature of
  // an ear picked from the wrong side of a reflex corner.
  for (const t of tris) assert.ok(area(t) <= trueArea + 1e-6);
});

test('a ring wound clockwise triangulates the same as one wound the other way', () => {
  const ccw = { ring: [{ x: 0, y: 0 }, { x: 5, y: 0 }, { x: 5, y: 5 }, { x: 0, y: 5 }], holes: [] };
  const cw = { ring: [...ccw.ring].reverse(), holes: [] };
  assert.ok(Math.abs(totalArea(triangulateShape(ccw)) - totalArea(triangulateShape(cw))) < 1e-9);
});

test('a shape with a hole triangulates around it, not through it', () => {
  const withHole = {
    ring: [{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }, { x: 0, y: 10 }],
    holes: [[{ x: 4, y: 4 }, { x: 6, y: 4 }, { x: 6, y: 6 }, { x: 4, y: 6 }]],
  };
  const tris = triangulateShape(withHole);
  const trueArea = 100 - 4; // the outer square less the 2x2 hole
  assert.ok(Math.abs(totalArea(tris) - trueArea) < 1e-6,
    `expected ${trueArea} sq ft around the hole, got ${totalArea(tris)}`);
  // The hole's own middle must not be covered by any triangle.
  const middle = { x: 5, y: 5 };
  const cross = (a, b, p) => (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
  const covers = (t, p) => {
    const d1 = cross(t[0], t[1], p);
    const d2 = cross(t[1], t[2], p);
    const d3 = cross(t[2], t[0], p);
    const hasNeg = d1 < 0 || d2 < 0 || d3 < 0;
    const hasPos = d1 > 0 || d2 > 0 || d3 > 0;
    return !(hasNeg && hasPos);
  };
  assert.ok(!tris.some((t) => covers(t, middle)), 'a triangle covers the middle of the hole');
});

test('closed rings (first point repeated at the end) triangulate the same as open ones', () => {
  const open = { ring: [{ x: 0, y: 0 }, { x: 3, y: 0 }, { x: 3, y: 3 }, { x: 0, y: 3 }], holes: [] };
  const closed = { ring: [...open.ring, open.ring[0]], holes: [] };
  assert.ok(Math.abs(totalArea(triangulateShape(open)) - totalArea(triangulateShape(closed))) < 1e-9);
});

test('a degenerate ring (fewer than three distinct points) draws nothing rather than throwing', () => {
  assert.deepEqual(triangulateShape({ ring: [{ x: 0, y: 0 }, { x: 1, y: 1 }], holes: [] }), []);
  assert.deepEqual(triangulateShape({ ring: [], holes: [] }), []);
});
