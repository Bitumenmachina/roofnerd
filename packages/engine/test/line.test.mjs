// One item, worked all the way out: formula → quantity → waste → order → money.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { priceLine } from '../dist/index.js';

const scope = { SF: 1000, LF: 435.15, EA: 5, SQ: 10, PLAN_SF: 1000, H: 1.5, STRETCHOUT: 14 };
const item = (over) => ({ id: 'i1', description: 'x', costCode: '07-100-100', unit: 'SF', formula: 'SF', ...over });

test('quantity comes off the formula, money off the quantity', () => {
  const r = priceLine(item({ unitCost: 2 }), scope);
  assert.equal(r.quantity, 1000);
  assert.equal(r.extended, 2000);
});

test('a run becomes an area on the line, where it can be seen', () => {
  const r = priceLine(item({ formula: 'LF * H', unit: 'SF', unitCost: 9 }), scope);
  assert.ok(Math.abs(r.quantity - 652.725) < 1e-9);
});

test('waste is added before packaging, not after', () => {
  const r = priceLine(item({ formula: '100', orderUnit: { name: 'ROLL', per: 10, waste: 10 }, unitCost: 50 }), scope);
  assert.equal(r.withWaste, 110);
  assert.equal(r.orderQuantity, 11);
});

test('floating-point crumbs do not buy an extra roll', () => {
  // 100 with 10% waste is 110.00000000000001 in binary floating point. A bare
  // ceil turns that into twelve rolls. An estimator finds this by counting the
  // pallet, which is a bad way to find it.
  const r = priceLine(item({ formula: '100', orderUnit: { name: 'ROLL', per: 10, waste: 10 }, unitCost: 50 }), scope);
  assert.equal(r.orderQuantity, 11);
  assert.equal(r.extended, 11 * 10 * 50);
});

test('packaging rounds up — you cannot buy two thirds of a bucket', () => {
  const r = priceLine(item({ formula: '101', orderUnit: { name: 'ROLL', per: 10 }, unitCost: 50 }), scope);
  assert.equal(r.orderQuantity, 11);
});

test('the bid pays for what gets bought, not what gets installed', () => {
  // 101 SF, sold ten to a roll: eleven rolls, and eleven rolls is what is paid for.
  const r = priceLine(item({ formula: '101', orderUnit: { name: 'ROLL', per: 10 }, unitCost: 2 }), scope);
  assert.equal(r.extended, 11 * 10 * 2);
});

test('an item with no order unit is priced on what it measures', () => {
  const r = priceLine(item({ formula: '101', unitCost: 2 }), scope);
  assert.equal(r.orderQuantity, null);
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
  assert.ok(Math.abs(r.hours - 435.15 / 40) < 1e-9);
});

test('an item with no production rate has no hours, not zero hours', () => {
  assert.equal(priceLine(item({ unitCost: 1 }), scope).hours, null);
});

test('the copper sheet case, end to end', () => {
  // 435.15 LF of 14" stretch-out on 3' x 10' sheets: 30 SF a sheet.
  const r = priceLine(item({
    formula: 'ceil(LF * STRETCHOUT / 12 / 30)', unit: 'EA', unitCost: 210,
  }), scope);
  assert.equal(r.quantity, Math.ceil((435.15 * 14) / 12 / 30));
  assert.equal(r.quantity, 17);
  assert.equal(r.extended, 17 * 210);
});
