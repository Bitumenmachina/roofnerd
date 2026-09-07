// One item, worked all the way out: formula → quantity → waste → order → money.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { priceLine } from '../dist/index.js';

const scope = { SF: 1000, LF: 400, EA: 5, SQ: 10, PLAN_SF: 1000, H: 1.5, STRETCHOUT: 14 };
const item = (over) => ({ id: 'i1', description: 'x', costCode: '07-100-100', unit: 'SF', formula: 'SF', ...over });

test('quantity comes off the formula, money off the quantity', () => {
  const r = priceLine(item({ unitCost: 2 }), scope);
  assert.equal(r.quantity, 1000);
  assert.equal(r.extended, 2000);
});

test('a run becomes an area on the line, where it can be seen', () => {
  const r = priceLine(item({ formula: 'LF * H', unit: 'SF', unitCost: 9 }), scope);
  assert.equal(r.quantity, 600);
});

test('waste is added before packaging, not after', () => {
  const r = priceLine(item({ formula: '100', waste: 10, order: { name: 'ROLL', per: 10, rule: 'ceil' }, unitCost: 50 }), scope);
  assert.equal(r.withWaste, 110);
  assert.equal(r.orderQuantity, 11);
});

test('floating-point crumbs do not buy an extra roll', () => {
  // 100 with 10% waste is 110.00000000000001 in binary floating point. A bare
  // ceil turns that into twelve rolls. An estimator finds this by counting the
  // pallet, which is a bad way to find it.
  const r = priceLine(item({ formula: '100', waste: 10, order: { name: 'ROLL', per: 10, rule: 'ceil' }, unitCost: 50 }), scope);
  assert.equal(r.orderQuantity, 11);
  assert.equal(r.extended, 11 * 50);
});

test('packaging rounds up — you cannot buy two thirds of a bucket', () => {
  const r = priceLine(item({ formula: '101', order: { name: 'ROLL', per: 10, rule: 'ceil' }, unitCost: 50 }), scope);
  assert.equal(r.orderQuantity, 11);
});

test('the bid pays for what gets bought, not what gets installed', () => {
  // 101 SF, sold ten to a roll: eleven rolls. Priced by the roll, that is
  // eleven rolls of money — the fraction nobody can buy is still paid for.
  const r = priceLine(item({ formula: '101', order: { name: 'ROLL', per: 10, rule: 'ceil' }, unitCost: 20 }), scope);
  assert.equal(r.orderQuantity, 11);
  assert.equal(r.extended, 11 * 20);
});

test('and priced by what the roll holds, when that is how it is quoted', () => {
  // The same eleven rolls, quoted by the square foot instead: ten SF to a roll.
  const r = priceLine(item({
    formula: '101',
    order: { name: 'ROLL', per: 10, rule: 'ceil' },
    price: { name: 'SF', contains: 10, rule: 'exact' },
    unitCost: 2,
  }), scope);
  assert.equal(r.priceQuantity, 110);
  assert.equal(r.extended, 220);
});

test('an item with no order unit is priced on what it measures', () => {
  const r = priceLine(item({ formula: '101', unitCost: 2 }), scope);
  assert.equal(r.orderQuantity, null);
  assert.equal(r.priceQuantity, 101);
  assert.equal(r.extended, 202);
});

test('a price typed on the line beats the scenario book', () => {
  const withOwn = priceLine(item({ formula: '10', unitCost: 5 }), scope, { i1: 99 });
  const withBook = priceLine(item({ formula: '10' }), scope, { i1: 99 });
  assert.equal(withOwn.extended, 50);
  assert.equal(withBook.extended, 990);
});

test('no price is said out loud, never counted as zero', () => {
  const r = priceLine(item({ formula: '10' }), scope);
  assert.equal(r.extended, null);
  assert.equal(r.pending, 'no price');
});

test('an unscaled sheet makes the line pending, not free', () => {
  const r = priceLine(item({ formula: 'LF', unitCost: 5 }), { ...scope, LF: null });
  assert.equal(r.quantity, null);
  assert.equal(r.extended, null);
  assert.match(r.pending, /no scale/);
});

test('a broken formula reports itself and prices nothing', () => {
  const r = priceLine(item({ formula: 'LF * NOPE', unitCost: 5 }), scope);
  assert.equal(r.quantity, null);
  assert.equal(r.extended, null);
  assert.match(r.formulaError, /nothing here is called "NOPE"/);
  assert.equal(typeof r.formulaErrorAt, 'number');
});

test('labor hours come from a production rate, never typed', () => {
  const r = priceLine(item({ formula: 'LF', unit: 'LF', productionRate: 40, unitCost: 1 }), scope);
  assert.equal(r.hours, 400 / 40);
});

test('an item with no production rate has no hours, not zero hours', () => {
  assert.equal(priceLine(item({ unitCost: 1 }), scope).hours, null);
});

test('the copper sheet case, end to end', () => {
  // 400 LF of 14" stretch-out on 3' x 10' sheets: 30 SF a sheet.
  const r = priceLine(item({
    formula: 'ceil(LF * STRETCHOUT / 12 / 30)', unit: 'EA', unitCost: 210,
  }), scope);
  assert.equal(r.quantity, Math.ceil((400 * 14) / 12 / 30));
  assert.equal(r.quantity, 16);
  assert.equal(r.extended, 16 * 210);
});

// ── the rounding rule, everywhere it applies ───────────────────────────────
// Addendum §7: the settled-floating-point rule is applied at every step that
// rounds a quantity, not only at the order-unit step where it was found.

import { settle, ceilPackages, roundWhole, floorWhole, SETTLE_PLACES, run as runFormula } from '../dist/index.js';

test('the rule is nine decimal places, stated rather than implied', () => {
  assert.equal(SETTLE_PLACES, 9);
  assert.equal(settle(110.00000000000001), 110);
  assert.equal(settle(1 / 3), 0.333333333);
});

test('settling does not move a number anyone would notice', () => {
  // A nanometre on a mile. Anything an estimator can measure survives intact.
  assert.equal(settle(1200.5), 1200.5);
  assert.equal(settle(0.0625), 0.0625);
  assert.equal(settle(58000.25), 58000.25);
});

test('every rounding step settles first', () => {
  // Without settling these are 12, 2 and 2 — each one a whole unit adrift.
  assert.equal(ceilPackages(11.000000000000002), 11);
  assert.equal(floorWhole(2.9999999999999996), 3);
  assert.equal(roundWhole(1.9999999999999998), 2);
});

test('settling decides the intent before the rounding decides the answer', () => {
  // 2.4999999999999996 is what arithmetic leaves when 2.5 was meant. Settled
  // it becomes 2.5, and 2.5 rounds up — so this answers 3, not 2. That is the
  // rule doing what it is for: rounding the number that was intended rather
  // than the crumbs left over from computing it.
  assert.equal(settle(2.4999999999999996), 2.5);
  assert.equal(roundWhole(2.4999999999999996), 3);
});

test('a rounding step still rounds when it genuinely should', () => {
  assert.equal(ceilPackages(11.001), 12);
  assert.equal(roundWhole(2.6), 3);
  assert.equal(floorWhole(2.9), 2);
});

test('ceil typed into a formula gets the same protection as the order unit', () => {
  // 100 LF at 10% waste, ten to a package, written out by hand on the line.
  assert.equal(runFormula('ceil(LF * 1.1 / 10)', { LF: 100 }).value, 11);
});

test('an infinite or missing value passes through rather than being mangled', () => {
  assert.equal(settle(Infinity), Infinity);
  assert.ok(Number.isNaN(settle(NaN)));
});
