// Geometry, measures, the formula language, and inheritance.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  polygonArea, polygonPerimeter, polylineLength, calibrateFromTwoPoints, calibrateFromScale,
  pitchFactor, distanceToShape, parseFeet, formatFeetInches, ARCHITECTURAL_SCALES,
  measure, scopeFor, parse, evaluate, run, namesUsed, measureJob, scopeOf, emptyJob,
  unitDisagreement,
} from '../dist/index.js';

const square = [{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }, { x: 0, y: 10 }];
const oneFoot = { 'page-1': { feetPerUnit: 1 } };

// ── geometry ───────────────────────────────────────────────────────────────

test('a square is its side squared, traced either way round', () => {
  assert.equal(polygonArea(square), 100);
  assert.equal(polygonArea([...square].reverse()), 100);
});

test('a polygon closes and a polyline does not', () => {
  assert.equal(polygonPerimeter(square), 40);
  assert.equal(polylineLength(square), 30);
});

test('two points and a real distance give a scale', () => {
  const cal = calibrateFromTwoPoints({ x: 0, y: 0 }, { x: 100, y: 0 }, 25);
  assert.equal(cal.feetPerUnit, 0.25);
});

test('a scale needs two different points and a real length', () => {
  assert.equal(calibrateFromTwoPoints({ x: 5, y: 5 }, { x: 5, y: 5 }, 25), null);
  assert.equal(calibrateFromTwoPoints({ x: 0, y: 0 }, { x: 10, y: 0 }, 0), null);
});

test('a quarter-inch sheet puts four feet in a paper inch', () => {
  const quarter = ARCHITECTURAL_SCALES.find((s) => s.label.startsWith('1/4'));
  assert.equal(quarter.feetPerInch, 4);
  // A PDF point is a seventy-second of an inch.
  assert.ok(Math.abs(calibrateFromScale(4, 72).feetPerUnit - 4 / 72) < 1e-12);
});

test('pitch factor is the slope column off an estimator\'s sheet', () => {
  assert.equal(pitchFactor(0), 1);
  assert.ok(Math.abs(pitchFactor(5) - 1.0833) < 0.0001);
  assert.ok(Math.abs(pitchFactor(12) - 1.4142) < 0.0001);
  assert.equal(pitchFactor(-3), 1, 'nonsense slope does not shrink the roof');
});

test('a point inside an area hits it, a point far off does not', () => {
  assert.equal(distanceToShape({ x: 5, y: 5 }, 'area', square), 0);
  assert.ok(distanceToShape({ x: 40, y: 5 }, 'area', square) > 25);
  assert.equal(distanceToShape({ x: 5, y: 0 }, 'line', square), 0);
});

test('lengths read the way a drawing writes them', () => {
  assert.equal(parseFeet("12'-6\""), 12.5);
  assert.equal(parseFeet("12' 6"), 12.5);
  assert.equal(parseFeet('6"'), 0.5);
  assert.equal(parseFeet("4'-6 1/2\""), 4 + 6.5 / 12);
  assert.equal(parseFeet('12.5'), 12.5);
  assert.equal(parseFeet('nonsense'), null);
});

test('lengths write the way a drawing writes them', () => {
  assert.equal(formatFeetInches(12.5), `12'-6"`);
  assert.equal(formatFeetInches(4 + 6.5 / 12), `4'-6 1/2"`);
  assert.equal(formatFeetInches(null), '—');
});

// ── measures ───────────────────────────────────────────────────────────────

const trace = (points) => [{ id: 't1', pageId: 'page-1', points }];

test('one area yields SF, its perimeter LF, and its corners EA at once', () => {
  const m = measure('area', trace(square), {}, oneFoot);
  assert.equal(m.PLAN_SF, 100);
  assert.equal(m.SF, 100);
  assert.equal(m.SQ, 1);
  assert.equal(m.LF, 40);
  assert.equal(m.EA, 4);
});

test('pitch lifts SF off PLAN_SF and leaves the footprint alone', () => {
  const m = measure('area', trace(square), { PITCH: 12 }, oneFoot);
  assert.equal(m.PLAN_SF, 100);
  assert.ok(Math.abs(m.SF - 141.42) < 0.01);
  assert.ok(Math.abs(m.SQ - 1.4142) < 0.001);
});

test('a run measures LF and corners, and has no area of its own', () => {
  const m = measure('line', trace(square), { H: 1.5 }, oneFoot);
  assert.equal(m.LF, 30);
  assert.equal(m.EA, 4);
  assert.equal(m.SF, null, 'a run becomes area through a formula, not behind your back');
});

test('a count needs no scale at all', () => {
  const m = measure('count', trace([{ x: 1, y: 1 }, { x: 2, y: 2 }, { x: 3, y: 3 }]), {}, {});
  assert.equal(m.EA, 3);
  assert.equal(m.LF, null);
});

test('an unscaled page makes a length pending, never zero', () => {
  const m = measure('area', trace(square), {}, {});
  assert.equal(m.SF, null);
  assert.equal(m.LF, null);
  assert.equal(m.EA, 4, 'corners can still be counted');
});

test('the scope hands a formula the measures and the properties together', () => {
  const m = measure('line', trace(square), { H: 1.5, STRETCHOUT: 14 }, oneFoot);
  const scope = scopeFor(m, { H: 1.5, STRETCHOUT: 14 });
  assert.equal(scope.LF, 30);
  assert.equal(scope.H, 1.5);
  assert.equal(scope.STRETCHOUT, 14);
});

// ── the formula language ───────────────────────────────────────────────────

const scope = { SF: 1000, LF: 400, EA: 5, SQ: 10, PLAN_SF: 1000, H: 1.5, STRETCHOUT: 14 };

test('the five worked examples from the handoff', () => {
  assert.equal(run('LF * H', scope).value, 400 * 1.5);          // wall flashing → SF
  assert.equal(run('LF', scope).value, 400);                     // coping → LF
  assert.equal(run('EA', scope).value, 5);                          // corners → EA
  assert.equal(run('LF * 2 / 0.5', scope).value, 400 * 4);       // two fasteners every 6"
  assert.equal(run('ceil(LF * STRETCHOUT / 12 / 30)', scope).value, // copper sheets
    Math.ceil((400 * 14) / 12 / 30));
});

test('arithmetic binds the way arithmetic binds', () => {
  assert.equal(run('2 + 3 * 4', {}).value, 14);
  assert.equal(run('(2 + 3) * 4', {}).value, 20);
  assert.equal(run('-SF + 10', { SF: 4 }).value, 6);
});

test('every function, and the arity each one takes', () => {
  assert.equal(run('ceil(1.1)', {}).value, 2);
  assert.equal(run('floor(1.9)', {}).value, 1);
  assert.equal(run('round(1.5)', {}).value, 2);
  assert.equal(run('max(1, 9, 3)', {}).value, 9);
  assert.equal(run('min(1, 9, 3)', {}).value, 1);
  assert.match(run('ceil(1, 2)', {}).error, /takes 1/);
});

test('a formula that is wrong says so, and says where', () => {
  assert.match(run('LF * ', scope).error, /stops early/);
  assert.match(run('LF $ 2', scope).error, /does not belong/);
  assert.equal(run('LF $ 2', scope).position, 3);
  assert.match(run('NOPE * 2', scope).error, /nothing here is called "NOPE"/);
  assert.match(run('nope(2)', scope).error, /no function called "nope"/);
  assert.match(run('1.2.3', scope).error, /is not a number/);
  assert.match(run('LF / 0', scope).error, /divides by zero/);
  assert.match(run('(LF', scope).error, /expected "\)"/);
  assert.match(run('LF 2', scope).error, /left over/);
});

test('a pending measure makes the answer pending, not zero', () => {
  assert.equal(run('LF * H', { LF: null, H: 1.5 }).value, null);
  assert.equal(run('ceil(LF / 100)', { LF: null }).value, null);
});

test('nothing is ever evaluated as code', () => {
  // The grammar has no property access, no strings and no calls it does not own,
  // so the ways into a host object are not syntax errors to be tightened later —
  // they were never expressible.
  for (const attempt of [
    'process',
    'this.constructor',
    'globalThis',
    'require("fs")',
    'constructor.constructor("return 1")()',
  ]) {
    const result = run(attempt, {});
    assert.equal(result.value, null, `${attempt} produced a value`);
    assert.ok(result.error, `${attempt} was not refused`);
  }
});

test('a formula reports which names it leans on', () => {
  assert.deepEqual([...namesUsed(parse('ceil(LF * STRETCHOUT / 12)'))].sort(), ['LF', 'STRETCHOUT']);
});

test('the tree is a tree, and it is walked rather than run', () => {
  const tree = parse('2 * (LF + 1)');
  assert.equal(tree.kind, 'binary');
  assert.equal(evaluate(tree, { LF: 4 }), 10);
});

// ── inheritance ────────────────────────────────────────────────────────────

function jobWith(conditions) {
  return {
    ...emptyJob('Demo'),
    pages: [{ id: 'page-1', name: 'Roof Plan', feetPerUnit: 1 }],
    conditions,
  };
}

test('a condition that says `from:` borrows the measures and keeps its own properties', () => {
  const doc = jobWith([
    { id: 'parapet', name: 'Parapet', kind: 'line', traces: trace(square), properties: { H: 3 }, items: [] },
    { id: 'coping', name: 'Coping', kind: 'line', traces: [], properties: { W: 1.5 }, from: 'parapet', items: [] },
  ]);
  const measured = measureJob(doc);
  assert.equal(measured.get('coping').LF, 30, 'one run, measured once');
  assert.equal(scopeOf(doc, 'coping').W, 1.5);
  assert.equal(scopeOf(doc, 'coping').H, undefined, 'it did not inherit the parapet height');
});

test('a chain of inheritance resolves through', () => {
  const doc = jobWith([
    { id: 'a', name: 'A', kind: 'line', traces: trace(square), properties: {}, items: [] },
    { id: 'b', name: 'B', kind: 'line', traces: [], properties: {}, from: 'a', items: [] },
    { id: 'c', name: 'C', kind: 'line', traces: [], properties: {}, from: 'b', items: [] },
  ]);
  assert.equal(measureJob(doc).get('c').LF, 30);
});

test('a circle of inheritance is refused rather than hung on', () => {
  const doc = jobWith([
    { id: 'a', name: 'A', kind: 'line', traces: [], properties: {}, from: 'b', items: [] },
    { id: 'b', name: 'B', kind: 'line', traces: [], properties: {}, from: 'a', items: [] },
  ]);
  assert.throws(() => measureJob(doc), /round in a circle/);
});

test('inheriting from a condition that is not there is refused', () => {
  const doc = jobWith([
    { id: 'a', name: 'A', kind: 'line', traces: [], properties: {}, from: 'ghost', items: [] },
  ]);
  assert.throws(() => measureJob(doc), /not here/);
});

// ── corners, segments and arcs ─────────────────────────────────────────────
// The rule these pin was read off real drawing reports rather than guessed at:
// a closed rectangle traced twelve times counts four corners each time, and a
// single arc counts none at all while still having a length. The numbers below
// are invented; only the rule came from anywhere.

test('a closed area has as many corners as it has sides', () => {
  const m = measure('area', trace(square), {}, oneFoot);
  assert.equal(m.VERTICES, 4);
  assert.equal(m.SEGMENTS, 4, 'the last side closes back to the first');
  assert.equal(m.EA, m.VERTICES);
});

test('twelve rectangular curbs read 48 EA, the way Edge reports them', () => {
  const curbs = Array.from({ length: 12 }, (_, i) => ({
    id: `curb-${i}`, pageId: 'page-1',
    points: [{ x: 0, y: 0 }, { x: 4, y: 0 }, { x: 4, y: 4 }, { x: 0, y: 4 }],
  }));
  assert.equal(measure('area', curbs, {}, oneFoot).EA, 48);
});

test('a run has one fewer straight piece than it has corners', () => {
  const m = measure('line', trace(square), {}, oneFoot);
  assert.equal(m.VERTICES, 4);
  assert.equal(m.SEGMENTS, 3, 'an open run does not close');
});

test('an arc has length but no corners — nothing on it gets mitred', () => {
  const arc = [{ id: 'a1', pageId: 'page-1', arc: true, points: square }];
  const m = measure('line', arc, {}, oneFoot);
  assert.equal(m.LF, 30, 'the curve still has a length');
  assert.equal(m.EA, 0);
  assert.equal(m.VERTICES, 0);
  assert.equal(m.SEGMENTS, 0);
});

test('a run of arcs and corners counts only the corners', () => {
  const mixed = [
    { id: 'straight', pageId: 'page-1', points: [{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }] },
    { id: 'curved', pageId: 'page-1', arc: true, points: [{ x: 10, y: 10 }, { x: 20, y: 10 }] },
  ];
  const m = measure('line', mixed, {}, oneFoot);
  assert.equal(m.EA, 3);
  assert.equal(m.LF, 30, 'both still contribute their length');
});

test('a count condition counts objects, and has no corners of its own', () => {
  const m = measure('count', trace([{ x: 1, y: 1 }, { x: 2, y: 2 }]), {}, {});
  assert.equal(m.EA, 2);
  assert.equal(m.VERTICES, 0);
});

test('a formula can buy mitres against VERTICES and pieces against SEGMENTS', () => {
  const m = measure('area', trace(square), {}, oneFoot);
  const scope = scopeFor(m, {});
  assert.equal(run('VERTICES', scope).value, 4);
  assert.equal(run('SEGMENTS * 2', scope).value, 8);
});

// ── the tapered inputs (section 6, addendum 4 §3) ───────────────────────────
// These are properties like any other. The reason they get their own tests is
// that TAPER sits next to PITCH and means something different, and a slope that
// quietly multiplied an area would be wrong in the direction nobody checks.

test('TAPER is not PITCH: insulation slope does not inflate the surface', () => {
  const flat = measure('area', trace(square), {}, oneFoot);
  const tapered = measure('area', trace(square), { TAPER: 0.25 }, oneFoot);
  // A quarter-inch-per-foot taper over a hundred square feet of deck adds
  // insulation, not roof. The membrane over it is still a hundred feet.
  assert.equal(tapered.SF, flat.SF);
  assert.equal(tapered.PLAN_SF, flat.PLAN_SF);
});

test('PITCH still does inflate it, with TAPER set alongside', () => {
  // A sloped deck carrying tapered insulation has both, and only one of them
  // is what the membrane is bought against.
  const m = measure('area', trace(square), { PITCH: 12, TAPER: 0.25 }, oneFoot);
  assert.equal(Math.round(m.SF), 141);
  assert.equal(m.PLAN_SF, 100);
});

test('ELEV does not touch any measure, on an area or on a run', () => {
  const area = measure('area', trace(square), { ELEV: 24 }, oneFoot);
  assert.equal(area.SF, 100);
  const line = measure('line', trace(square), { H: 1.5, ELEV: 24 }, oneFoot);
  assert.equal(line.LF, 30);
});

test('a formula can use the tapered inputs by name', () => {
  const m = measure('area', trace(square), { T: 0.5, TAPER: 0.25, ELEV: 24 }, oneFoot);
  const scope = scopeFor(m, { T: 0.5, TAPER: 0.25, ELEV: 24 });
  assert.equal(run('TAPER', scope).value, 0.25);
  assert.equal(run('ELEV', scope).value, 24);
  // Thickness twenty feet from the drain: half an inch, plus a quarter inch
  // for every foot out. This is the heightfield, done by hand on one line.
  assert.equal(run('T + 20 * TAPER', scope).value, 5.5);
});

test('an unscaled sheet still makes the tapered measures pending, not zero', () => {
  const m = measure('area', trace(square), { TAPER: 0.25 }, {});
  assert.equal(m.SF, null);
  assert.equal(m.PLAN_SF, null);
});

test('sump width and board count are properties, and touch no measure', () => {
  const plain = measure('area', trace(square), {}, oneFoot);
  const sumped = measure('area', trace(square), { T: 0.5, TAPER: 0.25, SUMP: 4, BOARDS: 3 }, oneFoot);
  assert.equal(sumped.SF, plain.SF);
  assert.equal(sumped.LF, plain.LF);
  assert.equal(sumped.EA, plain.EA);
});

test('a formula can reach the sump and the board count by name', () => {
  const props = { T: 0.5, TAPER: 0.25, SUMP: 4, BOARDS: 3 };
  const m = measure('area', trace(square), props, oneFoot);
  const scope = scopeFor(m, props);
  assert.equal(run('SUMP', scope).value, 4);
  assert.equal(run('BOARDS', scope).value, 3);
  // Thickness at the sump's outer edge, four feet out from the drain.
  assert.equal(run('T + SUMP * TAPER', scope).value, 1.5);
});

// ── does the unit follow from the formula (D-unit) ─────────────────────────
// Narrow on purpose: it speaks only where every name is a known measure and no
// bare constant is multiplying or dividing. Everything else is out of scope,
// which is the correct answer rather than a gap.

test('LF * H priced as LF is called out — feet times feet is an area', () => {
  const note = unitDisagreement(parse('LF * H'), 'LF');
  assert.ok(note, 'expected a note');
  assert.match(note, /an area/);
  assert.match(note, /LF/);
});

test('and the same formula priced as SF says nothing', () => {
  assert.equal(unitDisagreement(parse('LF * H'), 'SF'), null);
});

test('the sheet-metal formula stays quiet — the 30 is square feet per sheet', () => {
  assert.equal(unitDisagreement(parse('ceil(LF * STRETCHOUT / 12 / 30)'), 'EA'), null);
});

test('a bare constant anywhere in a product silences it', () => {
  assert.equal(unitDisagreement(parse('LF * 2 / 0.5'), 'EA'), null);
});

test('an unknown name silences it', () => {
  assert.equal(unitDisagreement(parse('LF * MYNUMBER'), 'LF'), null);
});

test('the plain cases agree and say nothing', () => {
  assert.equal(unitDisagreement(parse('SQ'), 'SQ'), null);
  assert.equal(unitDisagreement(parse('LF'), 'LF'), null);
  assert.equal(unitDisagreement(parse('VERTICES'), 'EA'), null);
  assert.equal(unitDisagreement(parse('EA'), 'EA'), null);
});

test('a count priced as a length is called out', () => {
  assert.match(unitDisagreement(parse('VERTICES'), 'LF') ?? '', /a count/);
});

test('ceil keeps the dimension it was handed', () => {
  assert.match(unitDisagreement(parse('ceil(LF * H)'), 'LF') ?? '', /an area/);
});
