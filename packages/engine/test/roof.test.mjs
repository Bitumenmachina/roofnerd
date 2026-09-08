// ── The roof conventions ───────────────────────────────────────────────────
// These test the trade's rules, not my code's opinion of them. Each one is
// written from a source that can be reopened; where a number appears here it is
// because a manual or a manufacturer's data sheet says it, and if the source is
// wrong these tests should fail rather than agree with me.
//
// This file exists because the Model's own check passed 14/14 against geometry
// I had invented. A check that measures the thing that produced it is worth
// nothing, so these are written against the convention.

import test from 'node:test';
import assert from 'node:assert/strict';
import {
  CRICKET_SLOPE_MULTIPLE, MIN_FLASHING_INCHES, SINGLE_LAYER_MAX_INCHES,
  buildUpInches, capInches, maxLengthToWidth, ridgeBetween, ridgeDisagreement,
  sumpSide,
} from '@roofnerd/engine';

// ── the ridge falls out of the drains ──────────────────────────────────────

test('the ridge between two drains is perpendicular to the line joining them', () => {
  // NRCA p.166-168. Two drains on the east-west line: the ridge runs north-south.
  const r = ridgeBetween({ x: 0, y: 0 }, { x: 40, y: 0 });
  assert.ok(r);
  assert.ok(Math.abs(r.along.x) < 1e-9, 'the ridge should not run along the drains');
  assert.ok(Math.abs(Math.abs(r.along.y) - 1) < 1e-9, 'the ridge runs square to them');
});

test('and it sits equidistant between them', () => {
  const r = ridgeBetween({ x: 10, y: 4 }, { x: 30, y: 24 });
  assert.ok(r);
  assert.equal(r.at.x, 20);
  assert.equal(r.at.y, 14);
});

test('the fall run is half the distance between the drains', () => {
  // Water leaves the ridge and reaches whichever drain it started toward, so the
  // run each way is half the span. That run is the cricket's width.
  const r = ridgeBetween({ x: 0, y: 0 }, { x: 0, y: 60 });
  assert.ok(r);
  assert.equal(r.width, 30);
});

test('two drains in the same place have no ridge between them', () => {
  assert.equal(ridgeBetween({ x: 5, y: 5 }, { x: 5, y: 5 }), null);
});

test('a traced ridge square to the drains disagrees by nothing', () => {
  const r = ridgeBetween({ x: 0, y: 0 }, { x: 40, y: 0 });
  const traced = [{ x: 20, y: -10 }, { x: 20, y: 10 }];
  assert.ok(ridgeDisagreement(traced, r.along) < 1e-6);
});

test('and one drawn along the drains disagrees by a right angle', () => {
  // The estimator drew the ridge parallel to the drain line. One of the two is
  // wrong about which drains this cricket serves, and the view has to say so.
  const r = ridgeBetween({ x: 0, y: 0 }, { x: 40, y: 0 });
  const traced = [{ x: 0, y: 5 }, { x: 40, y: 5 }];
  assert.ok(Math.abs(ridgeDisagreement(traced, r.along) - 90) < 1e-6);
});

test('a ridge run backwards is the same ridge', () => {
  // A ridge has no direction. Folding at 90 degrees keeps a trace drawn the
  // other way from reading as a right angle out.
  const r = ridgeBetween({ x: 0, y: 0 }, { x: 40, y: 0 });
  const forward = ridgeDisagreement([{ x: 20, y: -10 }, { x: 20, y: 10 }], r.along);
  const backward = ridgeDisagreement([{ x: 20, y: 10 }, { x: 20, y: -10 }], r.along);
  assert.equal(forward, backward);
});

// ── slope and proportion ───────────────────────────────────────────────────

test('a cricket is cut at twice the field', () => {
  // NRCA p.79 and p.165. A quarter in twelve of field wants a half in twelve of
  // cricket — which is a different box of boards, not the same boards steeper.
  assert.equal(0.25 * CRICKET_SLOPE_MULTIPLE, 0.5);
});

test('the length-to-width ceiling follows the field slope', () => {
  // NRCA Fig. 4-13.
  assert.equal(maxLengthToWidth(0.125), 3);
  assert.equal(maxLengthToWidth(0.25), 3);
  assert.equal(maxLengthToWidth(0.5), 4);
});

test('a long thin cricket is out of proportion however steep it is cut', () => {
  // The point of the table: surface slope does nothing for the valley. A cricket
  // 90 ft long over a 10 ft fall run is 9:1 and ponds at the valley no matter
  // what slope the boards are.
  const width = 10;
  const length = 90;
  assert.ok(length / width > maxLengthToWidth(0.25));
  assert.ok(length / width > maxLengthToWidth(0.5), 'not rescued by a steeper field');
});

// ── what stops the roof rising ─────────────────────────────────────────────

test('flashing height caps the build before the boards do', () => {
  // A 12 in parapet leaves 4 in of roof under NRCA's 8 in flashing minimum.
  // Two layers of board could carry 9 in, and the wall will not have it.
  const { cap, reason } = capInches(0.5, 2, 12);
  assert.equal(cap, 12 - MIN_FLASHING_INCHES);
  assert.equal(reason, 'flashing');
});

test('and the boards cap it when the wall is tall enough not to', () => {
  const { cap, reason } = capInches(0.5, 1, 48);
  assert.equal(cap, 0.5 + SINGLE_LAYER_MAX_INCHES);
  assert.equal(reason, 'boards');
});

test('one board layer does not stop at four inches', () => {
  // The old model capped a board at 4 in, which is not a number from anywhere.
  // Carlisle InsulBase runs to 4.5 in a single layer; GAF to 4.6.
  assert.ok(SINGLE_LAYER_MAX_INCHES > 4);
});

test('with nothing said, nothing caps it — and it does not invent a ceiling', () => {
  // There is no universal maximum build height. Saying so is the honest answer;
  // picking a plausible number is the defect.
  const { cap, reason } = capInches(0.5, undefined, undefined);
  assert.equal(cap, Infinity);
  assert.equal(reason, null);
});

test('a sump is the bowl plus two feet', () => {
  // NRCA p.164. A 12 in bowl wants 36 in square.
  assert.equal(sumpSide(12), 36);
});

// ── the build-up ───────────────────────────────────────────────────────────

test('the build-up is the sum of what the layers say', () => {
  // To a tolerance, not exactly: these are inches in binary floating point, and
  // a build-up argued to the ten-thousandth of an inch is not a real reading.
  // Money is a different matter and is not summed this way.
  const total = buildUpInches([{ thickness: 0.625 }, { thickness: 2.2 }, { thickness: 0.06 }]);
  assert.ok(Math.abs(total - 2.885) < 1e-9, `read ${total}`);
});

test('layers that say nothing about thickness add nothing', () => {
  assert.equal(buildUpInches([{ thickness: 2 }, {}, { thickness: 1 }]), 3);
});

test('a stack that says nothing at all is unknown, not zero', () => {
  // The whole reason the Model invented a nine inch deck. An unknown build-up is
  // null; a zero would draw a roof with nothing on it and look deliberate.
  assert.equal(buildUpInches([{}, {}]), null);
  assert.equal(buildUpInches([]), null);
});

test('thickness is a magnitude — a negative layer does not eat the one under it', () => {
  // IfcMaterialLayer.LayerThickness is an IfcNonNegativeLengthMeasure, and
  // FreeCAD clamps a negative Height to zero rather than flipping the solid.
  // Which way a stack grows is a separate parameter, never a sign on thickness.
  assert.equal(buildUpInches([{ thickness: 2 }, { thickness: -5 }]), 2);
});
