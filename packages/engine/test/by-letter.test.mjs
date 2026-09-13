// One line an estimator writes, one line per letter the roof actually buys.
//
// This is the half of C9 that came out of C6's fix round. The demo carried four
// items — iso-x, iso-y, iso-z, iso-zz — because a quarter in twelve buys those
// four panels. Set the high and low points on the roof, land on an eighth in
// twelve, and the manufacturer's chart names a different series entirely: AA,
// A, B, C and more, not the same four with moved edges. Every one of those four
// items then priced nothing, honestly, line by line, while the sheet still
// showed four panels the roof does not buy.
//
// So the job carries ONE line for the package and the sheet draws one per
// letter. What is checked here is the thing that used to break: the SAME job
// file, with the SAME one line on it, priced against a scope full of a
// different chart's letters.
//
// The letters are read out of the manufacturer's document rather than typed —
// what is being proved is that nothing in the job file knows any of them.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { linesFrom, priceLine, readJob } from '../dist/index.js';

const ROOT = resolve(import.meta.dirname, '../../..');
const profile = JSON.parse(
  readFileSync(resolve(ROOT, 'data/profiles/versico-versicore-mp-h.json'), 'utf8'),
);
const folder = resolve(ROOT, 'jobs/demo-job');
const demo = readJob((rel) => {
  try { return readFileSync(join(folder, rel), 'utf8'); } catch { return null; }
});

const field = demo.conditions.find((c) => c.id === 'c-field');
const scenario = demo.job.scenarios.find((s) => s.id === demo.job.activeScenarioId);
/** The one line the job carries for the whole tapered package. */
const package_ = (field.items ?? []).find((i) => i.eachOf !== undefined);

/** The letters one of the document's own charts prints, at a given slope. */
const lettersAt = (slope, set) => {
  const charted = profile.slopes.find((s) => Math.abs(s.slope_in_per_ft - slope) < 1e-9);
  return (charted?.[set]?.panels ?? []).map((p) => p.designation);
};

/** A scope shaped like the solve's: a square-foot figure against every letter. */
const scopeOf = (letters) => {
  const scope = { SF: 9600, SQ: 96, LF: 0, EA: 0, PLAN_SF: 9600, FILL_SF: 14080 };
  for (const [i, letter] of letters.entries()) scope[`TAPER_${letter}_SF`] = (i + 1) * 400;
  return scope;
};

test('the job file carries one line for the tapered package, and no letter anywhere', () => {
  assert.ok(package_, 'the demo has no line standing for the tapered package');
  const typed = JSON.stringify(demo.conditions);
  for (const letter of lettersAt(0.25, 'extended')) {
    assert.ok(!typed.includes(`TAPER_${letter}_SF`),
      `the job file names ${letter}, which is a fact about the slope and not about the job`);
  }
});

test('a quarter in twelve draws the panels a quarter in twelve buys', () => {
  const letters = ['X', 'Y', 'Z', 'ZZ'];
  const lines = linesFrom(package_, scopeOf(letters));
  assert.equal(lines.length, letters.length);
  assert.deepEqual(lines.map((l) => l.description), letters.map((l) => `Tapered ${l}`));
  assert.deepEqual(lines.map((l) => l.formula), letters.map((l) => `TAPER_${l}_SF / 100`));
  // The letter reaches the id, because the id is what the price book is keyed on.
  assert.deepEqual(lines.map((l) => l.id), letters.map((l) => `iso-${l.toLowerCase()}`));
});

test('and the same one line draws another chart’s series when the slope moves', () => {
  // An eighth in twelve. Not the same four panels with different edges — a
  // different lettered series, which is exactly why four typed items could not
  // survive somebody setting the high and low points.
  const letters = lettersAt(0.125, 'extended');
  assert.ok(letters.length > 4, 'the document draws no extended set at an eighth in twelve');
  assert.ok(!letters.includes('X'), 'the two charts share a designation after all');

  const scope = scopeOf(letters);
  const lines = linesFrom(package_, scope);
  assert.equal(lines.length, letters.length);

  for (const line of lines) {
    const priced = priceLine(line, scope, scenario.prices ?? {}, field.waste ?? 0, 'Material');
    assert.equal(priced.pending, null,
      `${line.description} says "${priced.pending}" on a roof that buys it`);
    assert.ok(priced.extended > 0, `${line.description} came to ${priced.extended}`);
  }
});

test('every letter of every chart has a price in the job’s own price set', () => {
  // The book is what makes the line above possible: a panel the solve names has
  // to be one the supply house has quoted, whichever chart it came off.
  let checked = 0;
  for (const slope of profile.slopes) {
    for (const set of ['standard', 'extended']) {
      for (const panel of slope[set]?.panels ?? []) {
        const key = `iso-${panel.designation.toLowerCase()}`;
        assert.ok(scenario.prices[key] > 0,
          `${panel.designation} is drawn by the chart and is not in the price set`);
        checked += 1;
      }
    }
  }
  assert.ok(checked > 20, `only ${checked} panels were checked`);
});

test('a family the model found none of is one line saying so, never no line', () => {
  // The silent zero wearing a different hat. A package line that vanished
  // because the solve answered nothing would take the whole tapered package off
  // the sheet without a word.
  const lines = linesFrom(package_, { SF: 9600, SQ: 96 });
  assert.equal(lines.length, 1);
  const priced = priceLine(lines[0], { SF: 9600, SQ: 96 }, scenario.prices ?? {});
  assert.equal(priced.quantity, null);
  assert.ok(priced.pending, 'a line with no panels behind it says nothing about why');
  assert.equal(priced.extended, null);
});
