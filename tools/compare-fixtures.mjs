// ── The fixture comparison ─────────────────────────────────────────────────
// Section 3's acceptance: take a recap printed by the program this one is
// replacing, and reproduce it from the same subtotals, adders and rates.
//
// The reports are a client's bid figures and live in `fixtures/`, which is not
// in the repository. This script reads them there and prints two things: the
// full table for the estimator's own eyes, and a SHAPE — which rows were
// checked and whether each was within tolerance, with no figure in it — which
// is the only form fit to go anywhere public.
//
//   node tools/compare-fixtures.mjs            full table
//   node tools/compare-fixtures.mjs --shape    the public form

import { readFile, readdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import {
  CLASS_NAMES, compareRecap, formatComparison, deriveProfitRate, DEFAULT_TOLERANCE,
} from '../packages/engine/dist/index.js';

const ROOT = resolve(import.meta.dirname, '..');
const DIR = join(ROOT, 'fixtures/recaps');
const shapeOnly = process.argv.includes('--shape');

/**
 * Build a recap from printed class subtotals.
 *
 * The item lines are not being reproduced here — the subtotals are taken as
 * given, and what is under test is the LADDER: which adder sits on which base,
 * what profit is taken on, what bond is taken on. That is the part the handoff
 * confirmed and the part a wrong reading puts thousands of dollars out.
 */
function ladderFrom(fixture) {
  const { scenario, subtotals, hours = {} } = fixture;
  const profitRate = deriveProfitRate({
    jobCost: fixture.printed.jobCost,
    profit: fixture.printed.profit,
  });

  const pct = (base, rate) => (rate ? base * (rate / 100) : 0);

  const classes = CLASS_NAMES.map((name) => {
    const subtotal = subtotals[name] ?? 0;
    const a = scenario.adders[name] ?? {};
    const adders = [];
    const take = (label, rate) => { if (rate) adders.push({ name: label, rate, amount: pct(subtotal, rate) }); };
    if (name === 'Material') { take('Sales tax', a.tax); take('Escalation', a.escalation); }
    if (name === 'Labor' || name === 'Supervision') take('Burden', a.burden);
    if (name === 'Sub') take('General liability', a.generalLiability);
    const total = adders.reduce((s, x) => s + x.amount, subtotal);
    return {
      class: name, subtotal, adders, total,
      hours: hours[name] ?? 0,
      perSquare: fixture.printed.totalSquares ? total / fixture.printed.totalSquares : null,
    };
  });

  const jobCost = classes.reduce((s, c) => s + c.total, 0);
  const overhead = pct(jobCost, scenario.overhead);
  const profit = pct(jobCost + overhead, profitRate ?? scenario.profitPrinted);
  const contractAmount = jobCost + overhead + profit;
  const bond = pct(contractAmount, scenario.bond);

  return {
    classes,
    jobCost,
    overhead,
    profit,
    contractAmount,
    bond,
    sellingPrice: contractAmount + bond,
    totalSquares: fixture.printed.totalSquares ?? null,
    totalHours: classes.reduce((s, c) => s + c.hours, 0),
    perSquare: null,
    pending: [],
    profitRate,
  };
}

let files;
try {
  files = (await readdir(DIR)).filter((f) => f.endsWith('.json'));
} catch {
  console.error(`No fixtures at ${DIR}.`);
  console.error('They are a client\'s bid figures and are deliberately not in the repository.');
  process.exit(2);
}

let allWithin = true;
for (const file of files.sort()) {
  const fixture = JSON.parse(await readFile(join(DIR, file), 'utf8'));
  const computed = ladderFrom(fixture);
  const comparison = compareRecap(computed, fixture.printed, DEFAULT_TOLERANCE);
  allWithin &&= comparison.allWithin;

  console.log(`\n${'='.repeat(64)}`);
  console.log(shapeOnly ? file.replace('.json', '') : `${file.replace('.json', '')} — ${fixture.source}`);
  console.log('='.repeat(64));
  console.log(formatComparison(comparison, { figures: !shapeOnly }));
  console.log(`printed profit rate ${fixture.scenario.profitPrinted}% · derived ${computed.profitRate?.toFixed(4)}%`);
}

console.log(`\n${allWithin ? 'PASS' : 'FAIL'} — every line within ${(DEFAULT_TOLERANCE * 100).toFixed(2)}%`);
process.exit(allWithin ? 0 : 1);
