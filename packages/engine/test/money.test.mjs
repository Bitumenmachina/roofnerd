// The money model: three units per item, per-step rounding, and the recap
// ladder. The shapes here were read off real Edge reports; the numbers are
// invented, and the real ones stay in fixtures/.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { priceLine, recap, priceJob, totalSquaresOf, emptyJob, CLASS_NAMES, girthOf } from '../dist/index.js';

const scope = { SF: 1000, LF: 500, EA: 5, SQ: 10, PLAN_SF: 1000, VERTICES: 5, SEGMENTS: 4, H: 1.5 };
const item = (over) => ({ id: 'i1', description: 'x', costCode: '07-100-100', unit: 'SF', formula: 'SF', ...over });

// ── three units ────────────────────────────────────────────────────────────

test('estimated in squares, ordered in rolls, priced by the square foot', () => {
  // 10 SQ + 10% waste = 11 SQ; a roll covers 10 SQ, so two rolls; each roll is
  // 1,000 SF, so 2,000 SF of price. This is the membrane shape.
  const r = priceLine(item({
    formula: 'SQ', unit: 'SQ', waste: 10,
    order: { name: 'ROLL', per: 10, rule: 'ceil' },
    price: { name: 'SF', contains: 1000, rule: 'exact' },
    unitCost: 1.5,
  }), scope);
  assert.equal(r.withWaste, 11);
  assert.equal(r.orderQuantity, 2);
  assert.equal(r.priceQuantity, 2000);
  assert.equal(r.extended, 3000);
  assert.equal(r.priceUnitName, 'SF');
});

test('estimated in feet, ordered in rolls, priced by the box — and rounded there', () => {
  // 500 LF; a roll is 50 LF, so 10 rolls; four rolls to a box, so 2.5 boxes,
  // rounded up to 3. This is the flashing-tape shape.
  const r = priceLine(item({
    formula: 'LF', unit: 'LF',
    order: { name: 'ROLL', per: 50, rule: 'ceil' },
    price: { name: 'BOX', per: 4, rule: 'ceil' },
    unitCost: 80,
  }), scope);
  assert.equal(r.orderQuantity, 10);
  assert.equal(r.priceQuantity, 3);
  assert.equal(r.extended, 240);
});

test('and the same shape, not rounded at the price step', () => {
  // 500 cartridges, twenty to a case: 25 cases exactly. The sealant shape,
  // where the supplier bills the fraction rather than a whole case.
  const r = priceLine(item({
    formula: 'LF', unit: 'LF',
    order: { name: 'TUBE', contains: 1, rule: 'exact' },
    price: { name: 'CASE', per: 20, rule: 'exact' },
    unitCost: 120,
  }), scope);
  assert.equal(r.orderQuantity, 500);
  assert.equal(r.priceQuantity, 25);
  assert.equal(r.extended, 3000);
});

test('a fraction of a package is ordered when the rule says exact', () => {
  // Fastener plates order at 4.94 boxes in a real job, because that is what the
  // supplier bills. Rounding everything up is a guess, not a rule.
  const r = priceLine(item({
    formula: '494', order: { name: 'BOX', per: 100, rule: 'exact' }, unitCost: 10,
  }), scope);
  assert.equal(r.orderQuantity, 4.94);
  // Money keeps full precision through the line; it is rounded at display.
  assert.ok(Math.abs(r.extended - 49.4) < 1e-9);
});

test('`per` and `contains` are the same fact from either end', () => {
  const byPer = priceLine(item({ formula: '100', order: { name: 'BOX', per: 25, rule: 'exact' }, unitCost: 1 }), scope);
  const byContains = priceLine(item({ formula: '100', order: { name: 'BOX', contains: 1 / 25, rule: 'exact' }, unitCost: 1 }), scope);
  assert.equal(byPer.orderQuantity, 4);
  assert.equal(byContains.orderQuantity, 4);
});

test('with no order step and no price step, you pay for what you measured', () => {
  const r = priceLine(item({ formula: 'SF', unitCost: 2 }), scope);
  assert.equal(r.orderQuantity, null);
  assert.equal(r.priceQuantity, 1000);
  assert.equal(r.extended, 2000);
});

test('a conversion that was never filled in says so instead of pricing', () => {
  const r = priceLine(item({ formula: 'SF', order: { name: 'ROLL', rule: 'ceil' }, unitCost: 2 }), scope);
  assert.equal(r.extended, null);
  assert.match(r.pending, /ROLL has no conversion/);
});

// ── labor ──────────────────────────────────────────────────────────────────

test('a labor line orders in hours, and the production rate is the conversion', () => {
  const r = priceLine(item({
    formula: 'LF', unit: 'LF', costCode: '07-100-300',
    productionRate: 5, unitCost: 50,
  }), scope);
  assert.equal(r.orderUnitName, 'HOURS');
  assert.equal(r.orderQuantity, 100);
  assert.equal(r.hours, 100);
  assert.equal(r.extended, 5000);
});

test('labor hours are never rounded up to a package', () => {
  // A thousand feet at three feet an hour is 333.33… hours, not 334.
  const r = priceLine(item({ formula: '1000', unit: 'LF', productionRate: 3, unitCost: 50 }), scope);
  assert.equal(r.hours, 1000 / 3);
  assert.notEqual(r.hours, Math.ceil(1000 / 3));
  assert.equal(r.extended, (1000 / 3) * 50);
});

test('crew days come from hours and crew size, and are never entered', () => {
  const r = priceLine(item({ formula: 'LF', productionRate: 5, crewSize: 5, unitCost: 50 }), scope);
  assert.equal(r.hours, 100);
  assert.equal(r.crewDays, 100 / 5 / 8);
});

test('a line with no production rate has no hours, not zero hours', () => {
  assert.equal(priceLine(item({ unitCost: 1 }), scope).hours, null);
});

// ── the recap ladder ───────────────────────────────────────────────────────

function job(items, adders = {}, extras = {}) {
  const base = emptyJob('Ladder');
  return {
    ...base,
    job: { ...base.job, scenarios: [{ ...base.job.scenarios[0], adders, ...extras }] },
    pages: [{ id: 'p1', name: 'Plan', feetPerUnit: 1 }],
    conditions: [{
      id: 'c1', name: 'Field', kind: 'area',
      traces: [{ id: 't1', pageId: 'p1', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }, { x: 100, y: 100 }, { x: 0, y: 100 }] }],
      properties: {}, items,
    }],
    costCodes: [
      { code: 'MAT', name: 'Roofing Material', class: 'Material' },
      { code: 'LAB', name: 'Roofing Labor', class: 'Labor' },
      { code: 'SUB', name: 'Subcontract', class: 'Sub' },
      { code: 'SUP', name: 'Supervision', class: 'Supervision' },
      { code: 'EQP', name: 'Equipment', class: 'Equipment' },
      { code: 'OTH', name: 'Other', class: 'Other' },
    ],
  };
}

test('adders sit side by side on the class subtotal — they do not compound', () => {
  // 10,000 of material, 8% tax and 12% escalation. Side by side that is 12,000.
  // Compounded it would be 12,096 — and a real job would be thousands out.
  const doc = job(
    [{ id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '10000', unitCost: 1 }],
    { Material: { tax: 8, escalation: 12 } },
  );
  const r = recap(doc, doc.job.scenarios[0]);
  const material = r.classes.find((c) => c.class === 'Material');
  assert.equal(material.subtotal, 10000);
  assert.deepEqual(material.adders.map((a) => a.amount), [800, 1200]);
  assert.equal(material.total, 12000);
});

test('the ladder walks job cost, profit, contract amount, bond, selling price', () => {
  const doc = job(
    [{ id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '1000', unitCost: 1 }],
    {},
    { profit: 25, bond: 2 },
  );
  const r = recap(doc, doc.job.scenarios[0]);
  assert.equal(r.jobCost, 1000);
  assert.equal(r.overhead, 0, 'Edge has no overhead line; it defaults to nothing');
  assert.equal(r.profit, 250);
  assert.equal(r.contractAmount, 1250);
  assert.equal(r.bond, 25, 'bond is taken on the contract amount');
  assert.equal(r.sellingPrice, 1275);
});

test('supervision carries its own burden, not the field labor rate', () => {
  const doc = job([
    { id: 'l', description: 'crew', costCode: 'LAB', unit: 'SF', formula: '1000', productionRate: 10, unitCost: 50 },
    { id: 's', description: 'super', costCode: 'SUP', unit: 'EA', formula: '100', unitCost: 75 },
  ], { Labor: { burden: 10 }, Supervision: { burden: 50 } });
  const r = recap(doc, doc.job.scenarios[0]);
  const labor = r.classes.find((c) => c.class === 'Labor');
  const supervision = r.classes.find((c) => c.class === 'Supervision');
  assert.equal(labor.subtotal, 5000);
  assert.equal(labor.total, 5500);
  assert.equal(supervision.subtotal, 7500);
  assert.equal(supervision.total, 11250);
});

test('material contains only material — a dumpster is not roofing material', () => {
  const doc = job([
    { id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '1000', unitCost: 10 },
    { id: 'd', description: 'dumpster', costCode: 'OTH', unit: 'EA', formula: '2', unitCost: 800 },
    { id: 'e', description: 'lift', costCode: 'EQP', unit: 'EA', formula: '1', unitCost: 3000 },
  ], { Material: { tax: 8 } });
  const r = recap(doc, doc.job.scenarios[0]);
  const by = (c) => r.classes.find((x) => x.class === c);
  assert.equal(by('Material').subtotal, 10000);
  assert.equal(by('Other').subtotal, 1600);
  assert.equal(by('Equipment').subtotal, 3000);
  // The tax lands on material alone, which is the whole point of the classes.
  assert.equal(by('Material').adders[0].amount, 800);
  assert.equal(by('Other').adders.length, 0);
});

test('hours are summed at full precision, not from the displayed figure', () => {
  const doc = job([
    { id: 'a', description: 'fab', costCode: 'LAB', unit: 'SF', formula: '10000', productionRate: 3, unitCost: 50 },
  ], { Labor: { burden: 10 } });
  const r = recap(doc, doc.job.scenarios[0]);
  const labor = r.classes.find((c) => c.class === 'Labor');
  assert.equal(labor.hours, 10000 / 3);
  // Summing the DISPLAYED figure instead would land about a sixth of a cent
  // out per line — which is exactly how a recap ends up dollars adrift.
  assert.notEqual(labor.hours, Number((10000 / 3).toFixed(2)));
  assert.ok(Math.abs(labor.subtotal - (10000 / 3) * 50) < 1e-9);
});

test('cost per square is on every class, and on the job', () => {
  // A 100 x 100 trace at a foot per unit is 10,000 SF: a hundred squares.
  const doc = job(
    [{ id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '10000', unitCost: 1 }],
    {}, { profit: 0, bond: 0 },
  );
  assert.equal(totalSquaresOf(doc), 100);
  const r = recap(doc, doc.job.scenarios[0]);
  assert.equal(r.totalSquares, 100);
  assert.equal(r.classes.find((c) => c.class === 'Material').perSquare, 100);
  assert.equal(r.perSquare, 100);
});

test('every class appears whether or not anything landed in it', () => {
  const doc = job([]);
  const r = recap(doc, doc.job.scenarios[0]);
  assert.deepEqual(r.classes.map((c) => c.class), [...CLASS_NAMES]);
  assert.equal(r.sellingPrice, 0);
});

test('a line that cannot be priced is named and left out of the total', () => {
  const doc = job([
    { id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '1000', unitCost: 1 },
    { id: 'x', description: 'mystery', costCode: 'MAT', unit: 'SF', formula: '1000' },
    { id: 'y', description: 'orphan', costCode: 'NOPE', unit: 'SF', formula: '1000', unitCost: 1 },
  ]);
  const r = recap(doc, doc.job.scenarios[0]);
  assert.equal(r.jobCost, 1000);
  assert.equal(r.pending.length, 2);
  assert.match(r.pending.join(' '), /no price/);
  assert.match(r.pending.join(' '), /in no class/);
});

test('priceJob names the condition every line came off', () => {
  const doc = job([{ id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '1000', unitCost: 1 }]);
  const [line] = priceJob(doc, doc.job.scenarios[0]);
  assert.equal(line.conditionName, 'Field');
  assert.equal(line.class, 'Material');
});

// ── comparing against a printed report ─────────────────────────────────────
// The shape of the acceptance test, checked with invented figures. The real
// ones stay in fixtures/ and never come into this tree.

import { compareRecap, deriveProfitRate, formatComparison, DEFAULT_TOLERANCE } from '../dist/index.js';

test('a recap that reproduces a printed one comes back all within', () => {
  const doc = job(
    [{ id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '10000', unitCost: 1 }],
    { Material: { tax: 8 } }, { profit: 25, bond: 2 },
  );
  const r = recap(doc, doc.job.scenarios[0]);
  const c = compareRecap(r, {
    classSubtotals: { Material: 10000 },
    classTotals: { Material: 10800 },
    jobCost: 10800,
    profit: 2700,
    contractAmount: 13500,
    bond: 270,
    sellingPrice: 13770,
  });
  assert.equal(c.allWithin, true, formatComparison(c));
  assert.equal(c.rows.length, 7);
});

test('a difference is reported with its size, not just as a failure', () => {
  const doc = job([{ id: 'm', description: 'membrane', costCode: 'MAT', unit: 'SF', formula: '10000', unitCost: 1 }]);
  const r = recap(doc, doc.job.scenarios[0]);
  const c = compareRecap(r, { jobCost: 11000 });
  assert.equal(c.allWithin, false);
  assert.equal(c.rows[0].printed, 11000);
  assert.equal(c.rows[0].computed, 10000);
  assert.ok(Math.abs(c.rows[0].relative - 1 / 11) < 1e-9);
  assert.match(formatComparison(c), /OUT by/);
});

test('the profit rate is derived from the dollars, not read off the page', () => {
  // A report prints its rate to two places and its dollars to the cent. Two
  // places is not always enough to reproduce the dollars: here the page would
  // print 12.35%, and 12.35% of the job cost misses the profit by a dollar.
  const rate = deriveProfitRate({ jobCost: 1000000, profit: 123456 });
  assert.equal(rate, 12.3456);
  assert.notEqual(Number(rate.toFixed(2)), rate, 'the printed rate is not the real one');
  assert.notEqual(1000000 * (12.35 / 100), 123456);
});

test('the tolerance is stated, and it is stated in the output', () => {
  const doc = job([{ id: 'm', description: 'm', costCode: 'MAT', unit: 'SF', formula: '10000', unitCost: 1 }]);
  const c = compareRecap(recap(doc, doc.job.scenarios[0]), { jobCost: 10000 }, DEFAULT_TOLERANCE);
  assert.equal(c.tolerance, 0.0001);
  assert.match(formatComparison(c), /within 0\.01%/);
});

test('a cent adrift on a large figure is within; a dollar on a small one is not', () => {
  const doc = job([{ id: 'm', description: 'm', costCode: 'MAT', unit: 'SF', formula: '1000000', unitCost: 1 }]);
  const r = recap(doc, doc.job.scenarios[0]);
  assert.equal(compareRecap(r, { jobCost: 1000000.01 }).allWithin, true);
  assert.equal(compareRecap(r, { jobCost: 1000200 }).allWithin, false);
});

test('the comparison can be printed without any of the figures in it', () => {
  const doc = job([{ id: 'm', description: 'm', costCode: 'MAT', unit: 'SF', formula: '10000', unitCost: 1 }]);
  const c = compareRecap(recap(doc, doc.job.scenarios[0]), { jobCost: 10000, sellingPrice: 10000 });
  const shape = formatComparison(c, { figures: false });
  assert.match(shape, /Job cost\twithin/);
  assert.doesNotMatch(shape, /10000/, 'a figure leaked into the public form');
});

test('total squares counts only area that carries a cost', () => {
  // A tapered field traced for the model and priced by nothing must not appear
  // in the denominator of a cost per square — it is not roof anybody is being
  // charged for, and counting it turns every rate in the recap into a rate
  // against a different roof.
  const base = {
    format: 1,
    job: { format: 1, name: 'T', activeScenarioId: 's1', scenarios: [{ id: 's1', name: 'S', prices: {}, adders: {} }] },
    pages: [{ id: 'p1', name: 'P', feetPerUnit: 1 }],
    costCodes: [],
    conditions: [
      {
        id: 'priced', name: 'Priced', kind: 'area', properties: {},
        traces: [{ id: 't1', pageId: 'p1', points: [{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 10 }, { x: 0, y: 10 }] }],
        items: [{ id: 'i1', description: 'Thing', costCode: 'c', unit: 'SQ', formula: 'SQ', unitCost: 1 }],
      },
      {
        id: 'geometry-only', name: 'Reference', kind: 'area', properties: {},
        traces: [{ id: 't2', pageId: 'p1', points: [{ x: 0, y: 0 }, { x: 100, y: 0 }, { x: 100, y: 100 }, { x: 0, y: 100 }] }],
        items: [],
      },
    ],
  };
  // 10 x 10 ft = 100 SF = 1 SQ priced; the 100 x 100 reference area is 100 SQ.
  assert.equal(totalSquaresOf(base), 1);
});

test('condition waste composes after item waste and stays its own factor', () => {
  const item = { id: 'i', description: 'M', costCode: 'c', unit: 'SQ', formula: 'SQ', waste: 10, unitCost: 1 };
  const scope = { SQ: 100 };
  const plain = priceLine(item, scope, {}, 0);
  const withRun = priceLine(item, scope, {}, 5);
  // 10% material waste on a run that also carries 5% is 15.5%, not 15% — each
  // factor keeps meaning what it says.
  assert.equal(plain.withWaste, 110);
  assert.equal(withRun.withWaste, 115.5);
});

test('girth is the legs plus what the hems eat', () => {
  // A coping: two faces, a top, and a hem on each edge. Entered once for the
  // detail, reused everywhere that detail runs — there is no table for this.
  assert.equal(girthOf({ id: 'p', name: 'Coping', legs: [4, 11, 4], hems: [0.5, 0.5] }), 20);
  assert.equal(girthOf({ id: 'p', name: 'Gravel stop', legs: [3, 5] }), 8);
});

test('a line loaded from the book is still priced by the book', () => {
  // Loading an assembly gives each line its own id, so a price typed on one
  // condition cannot leak to another. The book still has to be able to price
  // it, and it keys on the book's id — so the line carries both.
  const item = {
    id: 'c-parapet-coping-0', libraryId: 'coping', description: 'Coping',
    costCode: 'c', unit: 'LF', formula: 'LF',
  };
  const priced = priceLine(item, { LF: 100 }, { coping: 2.5 });
  assert.equal(priced.unitCost, 2.5);
  assert.equal(priced.extended, 250);
});

test('and a price on the line still beats the book', () => {
  const item = {
    id: 'x', libraryId: 'coping', description: 'Coping', costCode: 'c',
    unit: 'LF', formula: 'LF', unitCost: 9,
  };
  assert.equal(priceLine(item, { LF: 10 }, { coping: 2.5 }).unitCost, 9);
});
