// ── Geometry ───────────────────────────────────────────────────────────────
// The traced shape, before anybody prices it.

import test from 'node:test';
import assert from 'node:assert/strict';
import { onPage, polygonArea, selfIntersects, strayCorners } from '@roofnerd/engine';


// ── a ring that crosses itself ─────────────────────────────────────────────
// Checked rather than assumed, because nothing downstream would notice: area
// still returns a number and a triangulator still returns a mesh.

test('an ordinary rectangle does not cross itself', () => {
  assert.equal(selfIntersects([
    { x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 6 }, { x: 0, y: 6 },
  ]), false);
});

test('and neither does one closed with its first point repeated', () => {
  assert.equal(selfIntersects([
    { x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 6 }, { x: 0, y: 6 }, { x: 0, y: 0 },
  ]), false);
});

test('a bow tie does', () => {
  // The classic: two corners swapped, which is one misplaced click on a trace.
  assert.equal(selfIntersects([
    { x: 0, y: 0 }, { x: 10, y: 6 }, { x: 10, y: 0 }, { x: 0, y: 6 },
  ]), true);
});

test('and its area comes back a silent zero', () => {
  // Why this check has to exist, and it is worse than a wrong number. The two
  // lobes of a bow tie wind opposite ways, so the shoelace formula cancels them
  // against each other and returns 0 — for a shape ten feet by six. A zero is
  // the one answer this program is never allowed to produce quietly, and here
  // it comes out of correct arithmetic on a ring nobody checked.
  const bow = [{ x: 0, y: 0 }, { x: 10, y: 6 }, { x: 10, y: 0 }, { x: 0, y: 6 }];
  assert.equal(polygonArea(bow), 0);
  assert.equal(selfIntersects(bow), true, 'and this is the only thing that catches it');
});

test('an L-shaped roof is not a crossing', () => {
  // Concave is normal. A check that called this a defect would be worthless.
  assert.equal(selfIntersects([
    { x: 0, y: 0 }, { x: 20, y: 0 }, { x: 20, y: 10 },
    { x: 10, y: 10 }, { x: 10, y: 20 }, { x: 0, y: 20 },
  ]), false);
});

test('a run doubling back on itself is not a polygon and is not judged as one', () => {
  assert.equal(selfIntersects([{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 5, y: 0 }]), false);
});

// ── traced past the edge of the paper ──────────────────────────────────────

test('a point on the sheet is on the sheet', () => {
  const page = { id: 'p', name: 'Roof Plan', width: 612, height: 792 };
  assert.equal(onPage({ x: 300, y: 400 }, page), true);
  assert.equal(onPage({ x: 0, y: 0 }, page), true, 'the corner is on it');
  assert.equal(onPage({ x: 612, y: 792 }, page), true, 'so is the far corner');
});

test('and one past the bottom is not', () => {
  const page = { id: 'p', name: 'Roof Plan', width: 612, height: 792 };
  assert.equal(onPage({ x: 300, y: 1040 }, page), false);
  assert.equal(onPage({ x: -1, y: 400 }, page), false);
});

test('a sheet that does not know its own size calls everything on it', () => {
  // A drawing still loading has no bounds to be outside of, and warning on every
  // open is how a warning gets ignored.
  assert.equal(onPage({ x: 9999, y: 9999 }, { id: 'p', name: 'Roof Plan' }), true);
});

test('stray corners are counted, not clamped', () => {
  // The estimator may have had a reason. Saying so is the job; moving their
  // point quietly is not.
  const pages = [{ id: 'p', name: 'Roof Plan', width: 612, height: 792 }];
  const traces = [{ pageId: 'p', points: [
    { x: 60, y: 560 }, { x: 560, y: 560 }, { x: 560, y: 1040 }, { x: 60, y: 1040 },
  ] }];
  assert.equal(strayCorners(traces, pages), 2);
});

test('and a trace on a page nobody has is not counted as stray', () => {
  assert.equal(strayCorners([{ pageId: 'gone', points: [{ x: 5, y: 5 }] }],
    [{ id: 'p', name: 'Roof Plan', width: 10, height: 10 }]), 0);
});
