// ── Comparing against a real report ────────────────────────────────────────
// The acceptance test for the money model: take a recap printed by the program
// this one is replacing, and reproduce it from the same quantities and prices.
//
// Two rules about reading a printed report, both learned the hard way:
//
//   The DOLLARS are the truth. A printed percentage has been rounded to two
//   places for the page, and two places is not always enough to reproduce the
//   dollars beside it — take the rate off the page and the profit line lands a
//   few dollars out on a job of any size. The rate is derived from the dollars
//   and reported; a line is never failed on a rate the report itself rounded.
//
//   Everything is compared to a stated tolerance, and the tolerance is printed
//   beside the result. A comparison that says only "pass" is not evidence.
//
// The reports themselves never enter the public tree. This file knows the SHAPE
// of a recap and nothing about any job.

import type { Money, Percent } from './model.js';
import type { Recap } from './recap.js';

/** One figure off a printed report, to be reproduced. */
export interface Expected {
  readonly label: string;
  readonly printed: Money;
}

/** A recap as a printed report states it. Every field optional: compare what you have. */
export interface PrintedRecap {
  readonly classSubtotals?: Readonly<Partial<Record<string, Money>>>;
  readonly classTotals?: Readonly<Partial<Record<string, Money>>>;
  readonly classHours?: Readonly<Partial<Record<string, number>>>;
  readonly jobCost?: Money;
  readonly profit?: Money;
  readonly contractAmount?: Money;
  readonly bond?: Money;
  readonly sellingPrice?: Money;
  readonly totalSquares?: number;
  readonly totalHours?: number;
}

export interface Difference {
  readonly label: string;
  readonly printed: number;
  readonly computed: number;
  readonly difference: number;
  /** How far out, as a share of the printed figure. */
  readonly relative: number;
  readonly within: boolean;
}

export interface Comparison {
  readonly rows: readonly Difference[];
  readonly tolerance: number;
  readonly worst: Difference | null;
  readonly allWithin: boolean;
  /**
   * The profit rate the printed dollars actually imply, as against the rate the
   * report printed. Reported, never used to fail a line.
   */
  readonly derivedProfitRate: Percent | null;
}

/** A dollar is out by more than this share of itself, and it is a real difference. */
export const DEFAULT_TOLERANCE = 0.0001;   // one hundredth of one percent

function difference(label: string, printed: number, computed: number, tolerance: number): Difference {
  const diff = computed - printed;
  const relative = printed === 0 ? (diff === 0 ? 0 : Infinity) : Math.abs(diff) / Math.abs(printed);
  return { label, printed, computed, difference: diff, relative, within: relative <= tolerance };
}

/**
 * The profit rate implied by what a report printed.
 *
 * A report prints the rate rounded to two places and the dollars to the cent.
 * The dollars are what actually happened.
 */
export function deriveProfitRate(printed: PrintedRecap): Percent | null {
  if (printed.profit === undefined || printed.jobCost === undefined || printed.jobCost === 0) return null;
  return (printed.profit / printed.jobCost) * 100;
}

export function compareRecap(
  computed: Recap,
  printed: PrintedRecap,
  tolerance: number = DEFAULT_TOLERANCE,
): Comparison {
  const rows: Difference[] = [];
  const add = (label: string, was: number | undefined, now: number) => {
    if (was !== undefined) rows.push(difference(label, was, now, tolerance));
  };

  for (const [name, was] of Object.entries(printed.classSubtotals ?? {})) {
    const line = computed.classes.find((c) => c.class === name);
    if (line && was !== undefined) add(`${name} subtotal`, was, line.subtotal);
  }
  for (const [name, was] of Object.entries(printed.classTotals ?? {})) {
    const line = computed.classes.find((c) => c.class === name);
    if (line && was !== undefined) add(`${name} total`, was, line.total);
  }
  for (const [name, was] of Object.entries(printed.classHours ?? {})) {
    const line = computed.classes.find((c) => c.class === name);
    if (line && was !== undefined) add(`${name} hours`, was, line.hours);
  }

  add('Job cost', printed.jobCost, computed.jobCost);
  add('Profit', printed.profit, computed.profit);
  add('Contract amount', printed.contractAmount, computed.contractAmount);
  add('Bond', printed.bond, computed.bond);
  add('Selling price', printed.sellingPrice, computed.sellingPrice);
  add('Total SQ', printed.totalSquares, computed.totalSquares ?? 0);
  add('Total hours', printed.totalHours, computed.totalHours);

  const worst = rows.reduce<Difference | null>(
    (w, r) => (w === null || r.relative > w.relative ? r : w), null);

  return {
    rows,
    tolerance,
    worst,
    allWithin: rows.every((r) => r.within),
    derivedProfitRate: deriveProfitRate(printed),
  };
}

/**
 * The comparison as a table.
 *
 * `figures: false` prints only the shape — which rows were checked and whether
 * each was within tolerance — so a result can be reported in public without any
 * of a client's numbers coming with it.
 */
export function formatComparison(c: Comparison, { figures = true } = {}): string {
  const rows: string[] = [];
  const pct = (v: number) => (v * 100).toFixed(4).replace(/0+$/, '').replace(/\.$/, '');

  if (figures) {
    rows.push(['Line', 'Printed', 'Computed', 'Difference', ''].join('\t'));
    for (const r of c.rows) {
      rows.push([
        r.label,
        r.printed.toFixed(2),
        r.computed.toFixed(2),
        r.difference.toFixed(2),
        r.within ? 'within' : `OUT by ${pct(r.relative)}%`,
      ].join('\t'));
    }
  } else {
    rows.push(['Line', 'Result'].join('\t'));
    for (const r of c.rows) {
      rows.push([r.label, r.within ? `within ${pct(c.tolerance)}%` : `OUT by ${pct(r.relative)}%`].join('\t'));
    }
  }

  rows.push('');
  rows.push(`${c.rows.filter((r) => r.within).length} of ${c.rows.length} within ${pct(c.tolerance)}%`);
  if (c.worst && !c.worst.within) rows.push(`worst: ${c.worst.label}, out by ${pct(c.worst.relative)}%`);
  if (c.derivedProfitRate !== null) {
    rows.push(`profit rate implied by the printed dollars: ${c.derivedProfitRate.toFixed(4)}%`);
  }
  return rows.join('\n');
}
