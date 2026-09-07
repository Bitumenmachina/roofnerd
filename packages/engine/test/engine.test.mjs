// The engine's own tests. They run with `node --test` against the built package —
// no window, no browser, no drawing.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  CLASS_NAMES, JOB_FORMAT, emptyJob, readJob, writeJob, recap, sf, sq, lf,
  sfToSq, sqToSf, add, scale, formatMeasure, classOf,
} from '../dist/index.js';

/** A tiny in-memory job folder, so a test never touches a disk. */
function folder(initial = {}) {
  const files = new Map(Object.entries(initial));
  return {
    files,
    read: (p) => (files.has(p) ? files.get(p) : null),
    write: (p, c) => files.set(p, c),
  };
}

test('one square is one hundred square feet, both directions', () => {
  assert.equal(sfToSq(sf(2350)).value, 23.5);
  assert.equal(sqToSf(sq(23.5)).value, 2350);
  assert.equal(sfToSq(sf(2350)).unit, 'SQ');
});

test('a measure keeps the unit through arithmetic', () => {
  assert.deepEqual(add(lf(100), lf(35.5)), { value: 135.5, unit: 'LF' });
  assert.deepEqual(scale(sf(1000), 1.1), { value: 1100, unit: 'SF' });
  assert.equal(formatMeasure(lf(1354.93)), '1,354.93 LF');
  assert.equal(formatMeasure({ value: 5.4, unit: 'EA' }), '5 EA');
});

test('a job folder round-trips: write it, read it, same document', () => {
  const fs = folder();
  const doc = emptyJob('Demo');
  writeJob(doc, fs.write);
  assert.deepEqual(readJob(fs.read), doc);
});

test('a saved job has sorted keys, so a diff shows only what you changed', () => {
  const fs = folder();
  writeJob(emptyJob('Demo'), fs.write);
  const keys = Object.keys(JSON.parse(fs.read('job.json')));
  assert.deepEqual(keys, [...keys].sort());
  assert.ok(fs.read('job.json').endsWith('\n'));
});

test('a job from a future format is refused, not guessed at', () => {
  const fs = folder({ 'job.json': JSON.stringify({ format: JOB_FORMAT + 1, name: 'X', scenarios: [] }) });
  assert.throws(() => readJob(fs.read), /format/);
});

test('the recap carries every class, every time', () => {
  const doc = emptyJob('Demo');
  const r = recap(doc, doc.job.scenarios[0]);
  assert.deepEqual(r.classes.map((c) => c.class), [...CLASS_NAMES]);
});

test('an empty job sells for nothing, which is the honest answer', () => {
  const doc = emptyJob('Demo');
  const r = recap(doc, doc.job.scenarios[0]);
  assert.equal(r.jobCost, 0);
  assert.equal(r.sellingPrice, 0);
  assert.deepEqual(r.unpriced, []);
});

test('each class takes its own adders and nobody else\'s', () => {
  const doc = emptyJob('Demo');
  const scenario = {
    ...doc.job.scenarios[0],
    adders: {
      Material: { tax: 7, escalation: 3 },
      Labor: { burden: 10 },
      Supervision: { burden: 50 },
      Sub: { generalLiability: 2 },
    },
  };
  const r = recap(doc, scenario);
  const named = (c) => r.classes.find((x) => x.class === c).adders.map((a) => a.name);
  assert.deepEqual(named('Material'), ['Sales tax', 'Escalation']);
  assert.deepEqual(named('Labor'), ['Burden']);
  assert.deepEqual(named('Supervision'), ['Burden']);
  assert.deepEqual(named('Sub'), ['General liability']);
  assert.deepEqual(named('Equipment'), []);
  assert.deepEqual(named('Other'), []);
});

test('supervision burden is its own rate, not the field labor rate', () => {
  const doc = emptyJob('Demo');
  const scenario = {
    ...doc.job.scenarios[0],
    adders: { Labor: { burden: 10 }, Supervision: { burden: 50 } },
  };
  const r = recap(doc, scenario);
  const rate = (c) => r.classes.find((x) => x.class === c).adders[0].rate;
  assert.equal(rate('Labor'), 10);
  assert.equal(rate('Supervision'), 50);
});

test('overhead, then profit, then bond — each on what came before it', () => {
  const doc = {
    ...emptyJob('Demo'),
    // A class gross of zero makes the walk visible without pricing existing yet:
    // every step is a percentage of the running total, and 0 stays 0.
  };
  const scenario = { ...doc.job.scenarios[0], overhead: 10, profit: 10, bond: 1 };
  const r = recap(doc, scenario);
  assert.equal(r.overhead, 0);
  assert.equal(r.profit, 0);
  assert.equal(r.bond, 0);
  assert.equal(r.sellingPrice, r.jobCost + r.overhead + r.profit + r.bond);
});

test('an item whose cost code is in no class is named, never counted as zero', () => {
  const doc = {
    ...emptyJob('Demo'),
    conditions: [{
      id: 'c1', name: 'Parapet Wall Flashing', pageId: 'p1', properties: {}, measures: [],
      items: [{ id: 'i1', description: '16 oz copper', costCode: '07-100-100', unit: 'SF', formula: 'ADJ', unitCost: 12 }],
    }],
    costCodes: [],
  };
  const r = recap(doc, doc.job.scenarios[0]);
  assert.equal(r.unpriced.length, 1);
  assert.match(r.unpriced[0], /in no class/);
});

test('an item with no price is named, never counted as zero', () => {
  const doc = {
    ...emptyJob('Demo'),
    conditions: [{
      id: 'c1', name: 'Parapet Wall Flashing', pageId: 'p1', properties: {}, measures: [],
      items: [{ id: 'i1', description: '16 oz copper', costCode: '07-100-100', unit: 'SF', formula: 'ADJ' }],
    }],
    costCodes: [{ code: '07-100-100', name: 'Roofing Material', class: 'Material' }],
  };
  const r = recap(doc, doc.job.scenarios[0]);
  assert.equal(r.unpriced.length, 1);
  assert.match(r.unpriced[0], /no price/);
});

test('classOf finds an item\'s class through its cost code', () => {
  const codes = [{ code: '07-990', name: 'Supervision', class: 'Supervision' }];
  assert.equal(classOf(codes, '07-990'), 'Supervision');
  assert.equal(classOf(codes, '07-111'), undefined);
});
