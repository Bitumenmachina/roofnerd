// ── The recap ──────────────────────────────────────────────────────────────
// Where a job becomes a selling price. The rule this file exists to enforce:
// a cost lands in a class, and a class carries its own adders. Fifteen thousand
// dollars of "material" that is really material plus a dumpster plus a permit is
// the lie this structure makes impossible to tell.
//
//   every item → its cost code → that code's class
//   class subtotal + that class's own adders, each on the SUBTOTAL   = class total
//   sum of class totals                                             = job cost
//   + overhead                                                      = (Edge has none)
//   + profit, on what came before                                   = contract amount
//   + bond, on the contract amount                                  = selling price
//
// Two things that look like details and are not:
//
//   Adders within a class are each taken on the class SUBTOTAL and then summed.
//   Tax and escalation both sit on subtotal material — they do not compound, and
//   compounding them puts a job several thousand dollars out.
//
//   Hours are summed at full precision. A labor line displays 4,566.87 hours,
//   but it holds 4,566.87…, and totalling the displayed figure is how a recap
//   ends up a few dollars adrift of the report it is checked against.

import type {
  ClassAdders, ClassName, CostCode, JobDocument, Money, Percent, Scenario,
} from './model.js';
import { CLASS_NAMES } from './model.js';
import { priceLine, type LineResult } from './line.js';
import { measureJob } from './takeoff.js';
import { scopeFor } from './measures.js';

export interface ClassAdderLine {
  readonly name: string;
  readonly rate: Percent;
  readonly amount: Money;
}

export interface ClassLine {
  readonly class: ClassName;
  /** What the items in this class cost before anything is added. */
  readonly subtotal: Money;
  /** Each adder, named, and each taken on the subtotal. Never folded in. */
  readonly adders: readonly ClassAdderLine[];
  /** Subtotal plus this class's own adders. */
  readonly total: Money;
  /** Labor hours in this class, at full precision. */
  readonly hours: number;
  /** Class total per square of roof. Absent when the job has no area yet. */
  readonly perSquare: Money | null;
}

export interface Recap {
  readonly classes: readonly ClassLine[];
  readonly jobCost: Money;
  readonly overhead: Money;
  readonly profit: Money;
  /** Job cost plus overhead plus profit — what bond is taken on. */
  readonly contractAmount: Money;
  readonly bond: Money;
  readonly sellingPrice: Money;
  readonly totalSquares: number | null;
  readonly totalHours: number;
  readonly perSquare: Money | null;
  /**
   * Lines whose money could not be worked out. They are named, not counted as
   * zero. A total taken over an unpriced line is a total that lies.
   */
  readonly pending: readonly string[];
}

const pct = (amount: Money, rate: Percent | undefined): Money =>
  rate ? amount * (rate / 100) : 0;

/** Which adders a class takes, in the order the recap prints them. */
function addersFor(cls: ClassName, a: ClassAdders | undefined, subtotal: Money): ClassAdderLine[] {
  const out: ClassAdderLine[] = [];
  if (!a) return out;
  const take = (name: string, rate: Percent | undefined) => {
    // Each one on the SUBTOTAL, not on the running total. Tax and escalation
    // sit side by side on subtotal material; they do not compound.
    if (rate) out.push({ name, rate, amount: pct(subtotal, rate) });
  };
  switch (cls) {
    case 'Material':
      take('Sales tax', a.tax);
      take('Escalation', a.escalation);
      break;
    case 'Labor':
      take('Burden', a.burden);
      break;
    case 'Supervision':
      // Its own class precisely so it can carry its own burden rate, which is
      // not the field labor rate and never was.
      take('Burden', a.burden);
      break;
    case 'Sub':
      take('General liability', a.generalLiability);
      break;
    case 'Equipment':
    case 'Other':
      break;
  }
  return out;
}

/** Find an item's class through its cost code. */
export function classOf(costCodes: readonly CostCode[], code: string): ClassName | undefined {
  return costCodes.find((c) => c.code === code)?.class;
}

/** Every priced line in the job, with the condition it came off. */
export interface PricedLine extends LineResult {
  readonly conditionId: string;
  readonly conditionName: string;
  readonly class: ClassName | null;
}

export function priceJob(doc: JobDocument, scenario: Scenario): PricedLine[] {
  const measured = measureJob(doc);
  const out: PricedLine[] = [];

  for (const condition of doc.conditions) {
    const measures = measured.get(condition.id);
    if (!measures) continue;
    const scope = scopeFor(measures, condition.properties ?? {});
    for (const item of condition.items ?? []) {
      out.push({
        ...priceLine(item, scope, scenario.prices ?? {}),
        conditionId: condition.id,
        conditionName: condition.name,
        class: classOf(doc.costCodes, item.costCode) ?? null,
      });
    }
  }
  return out;
}

/**
 * How many squares of roof the job is, for the cost-per-square column.
 *
 * The sum of every area condition's squares, unless the job states one — some
 * jobs price against a contract area that is not the sum of what was traced.
 */
export function totalSquaresOf(doc: JobDocument): number | null {
  const measured = measureJob(doc);
  let squares = 0;
  let sawOne = false;
  for (const condition of doc.conditions) {
    if (condition.kind !== 'area') continue;
    const sq = measured.get(condition.id)?.SQ;
    if (sq === null || sq === undefined) continue;
    squares += sq;
    sawOne = true;
  }
  return sawOne && squares > 0 ? squares : null;
}

/** Roll a job up to a selling price under one scenario's prices and adders. */
export function recap(doc: JobDocument, scenario: Scenario): Recap {
  const lines = priceJob(doc, scenario);

  const subtotals = new Map<ClassName, Money>(CLASS_NAMES.map((c) => [c, 0]));
  const hoursBy = new Map<ClassName, number>(CLASS_NAMES.map((c) => [c, 0]));
  const pending: string[] = [];

  for (const line of lines) {
    const where = `${line.conditionName} → ${line.item.description || line.item.id}`;
    if (line.class === null) {
      pending.push(`${where}: cost code "${line.item.costCode}" is in no class`);
      continue;
    }
    if (line.extended === null) {
      pending.push(`${where}: ${line.pending ?? 'no money'}`);
      continue;
    }
    subtotals.set(line.class, (subtotals.get(line.class) ?? 0) + line.extended);
    // Full precision, deliberately. Summing displayed hours is how a labor
    // total ends up adrift of the report it is being checked against.
    if (line.hours !== null) hoursBy.set(line.class, (hoursBy.get(line.class) ?? 0) + line.hours);
  }

  const totalSquares = totalSquaresOf(doc);

  const classes: ClassLine[] = CLASS_NAMES.map((cls) => {
    const subtotal = subtotals.get(cls) ?? 0;
    const adders = addersFor(cls, scenario.adders[cls], subtotal);
    const total = adders.reduce((sum, a) => sum + a.amount, subtotal);
    return {
      class: cls,
      subtotal,
      adders,
      total,
      hours: hoursBy.get(cls) ?? 0,
      perSquare: totalSquares ? total / totalSquares : null,
    };
  });

  const jobCost = classes.reduce((sum, c) => sum + c.total, 0);
  const overhead = pct(jobCost, scenario.overhead);
  const profit = pct(jobCost + overhead, scenario.profit);
  const contractAmount = jobCost + overhead + profit;
  const bond = pct(contractAmount, scenario.bond);
  const sellingPrice = contractAmount + bond;
  const totalHours = classes.reduce((sum, c) => sum + c.hours, 0);

  return {
    classes,
    jobCost,
    overhead,
    profit,
    contractAmount,
    bond,
    sellingPrice,
    totalSquares,
    totalHours,
    perSquare: totalSquares ? sellingPrice / totalSquares : null,
    pending,
  };
}

const money = (v: Money) => v.toLocaleString('en-US', { style: 'currency', currency: 'USD' });
const num = (v: number, places = 2) =>
  v.toLocaleString('en-US', { minimumFractionDigits: places, maximumFractionDigits: places });

/** The recap as a page of text. What the command line prints. */
export function formatRecap(r: Recap): string {
  const rows: string[] = [];
  const line = (label: string, amount: Money, indent = 0, trailer = '') =>
    rows.push(`${' '.repeat(indent)}${label.padEnd(32 - indent)}${money(amount).padStart(16)}${trailer}`);

  for (const c of r.classes) {
    const perSq = c.perSquare === null ? '' : `   ${money(c.perSquare)}/SQ`;
    const hrs = c.hours ? `   ${num(c.hours)} hrs` : '';
    line(c.class.toUpperCase(), c.subtotal, 0, hrs);
    for (const a of c.adders) line(`${a.name} @ ${a.rate}%`, a.amount, 2);
    if (c.adders.length) line(`Total ${c.class}`, c.total, 2, perSq);
    else if (c.perSquare !== null) rows.push(`${' '.repeat(48)}${perSq}`);
  }

  rows.push('-'.repeat(52));
  line('Job cost', r.jobCost);
  if (r.overhead) line('Overhead', r.overhead);
  line('Profit', r.profit);
  line('CONTRACT AMOUNT', r.contractAmount);
  line('Bond', r.bond);
  rows.push('-'.repeat(52));
  line('SELLING PRICE', r.sellingPrice);

  if (r.totalSquares) rows.push(`\nTotal SQ    ${num(r.totalSquares)}`);
  if (r.totalHours) rows.push(`Total hours ${num(r.totalHours)}`);

  if (r.pending.length) {
    rows.push('', `${r.pending.length} line(s) not in the total:`);
    for (const u of r.pending) rows.push(`  ${u}`);
  }
  return rows.join('\n');
}
