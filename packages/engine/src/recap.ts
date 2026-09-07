// ── The recap ──────────────────────────────────────────────────────────────
// Where a job becomes a selling price. The rule this file exists to enforce:
// a cost lands in a class, and a class carries its own adders. Fifteen thousand
// dollars of "material" that is really material plus a dumpster plus a permit is
// the lie this structure makes impossible to tell.
//
//   every item → its cost code → that code's class
//   class gross + that class's own adders          = class extended
//   sum of classes                                 = job cost
//   + overhead  + profit  + bond                   = selling price
//
// GATE 0 SCOPE: the structure is here and it is checked. The arithmetic that
// fills it comes from priced items at Gate 2 — until then a job with no priced
// items rolls up to zero, which is the honest answer to "what does nothing cost".

import type {
  ClassAdders, ClassName, CostCode, JobDocument, Money, Percent, Scenario,
} from './model.js';
import { CLASS_NAMES } from './model.js';

export interface ClassLine {
  readonly class: ClassName;
  /** What the items in this class cost before anything is added. */
  readonly gross: Money;
  /** Each adder, named and shown separately. Never folded into the gross. */
  readonly adders: readonly { readonly name: string; readonly rate: Percent; readonly amount: Money }[];
  /** Gross plus this class's own adders. */
  readonly extended: Money;
}

export interface Recap {
  readonly classes: readonly ClassLine[];
  readonly jobCost: Money;
  readonly overhead: Money;
  readonly profit: Money;
  readonly bond: Money;
  readonly sellingPrice: Money;
  /**
   * Items whose cost could not be determined. They are named, not counted as
   * zero. A total computed over an unpriced item is a total that lies.
   */
  readonly unpriced: readonly string[];
}

const pct = (amount: Money, rate: Percent | undefined): Money =>
  rate ? amount * (rate / 100) : 0;

/** Which adders a class actually takes, in the order the recap prints them. */
function addersFor(cls: ClassName, a: ClassAdders | undefined, gross: Money) {
  const out: { name: string; rate: Percent; amount: Money }[] = [];
  if (!a) return out;
  const take = (name: string, rate: Percent | undefined) => {
    if (rate) out.push({ name, rate, amount: pct(gross, rate) });
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

/** Index cost codes by code string so an item can find its class. */
export function classOf(costCodes: readonly CostCode[], code: string): ClassName | undefined {
  return costCodes.find((c) => c.code === code)?.class;
}

/**
 * Roll a job up to a selling price under one scenario's prices and adders.
 *
 * Gate 0: gross per class is zero because no item is priced yet. The classes,
 * their adders, and the walk down to selling price are real and are tested.
 */
export function recap(doc: JobDocument, scenario: Scenario): Recap {
  const gross = new Map<ClassName, Money>(CLASS_NAMES.map((c) => [c, 0]));
  const unpriced: string[] = [];

  for (const condition of doc.conditions) {
    for (const item of condition.items) {
      const cls = classOf(doc.costCodes, item.costCode);
      if (!cls) {
        unpriced.push(`${condition.name} → ${item.description}: cost code "${item.costCode}" is in no class`);
        continue;
      }
      const unitCost = item.unitCost ?? scenario.prices[item.id];
      if (unitCost == null) {
        unpriced.push(`${condition.name} → ${item.description}: no price`);
        continue;
      }
      // Gate 2 puts the item's quantity here, from its formula against the
      // condition's measures. Until that exists there is nothing to multiply.
    }
  }

  const classes: ClassLine[] = CLASS_NAMES.map((cls) => {
    const g = gross.get(cls) ?? 0;
    const adders = addersFor(cls, scenario.adders[cls], g);
    const extended = adders.reduce((sum, a) => sum + a.amount, g);
    return { class: cls, gross: g, adders, extended };
  });

  const jobCost = classes.reduce((sum, c) => sum + c.extended, 0);
  const overhead = pct(jobCost, scenario.overhead);
  const profit = pct(jobCost + overhead, scenario.profit);
  const bond = pct(jobCost + overhead + profit, scenario.bond);

  return {
    classes,
    jobCost,
    overhead,
    profit,
    bond,
    sellingPrice: jobCost + overhead + profit + bond,
    unpriced,
  };
}

const money = (v: Money) =>
  v.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

/** The recap as a page of text. What the command line prints. */
export function formatRecap(r: Recap): string {
  const rows: string[] = [];
  const line = (label: string, amount: Money, indent = 0) =>
    rows.push(`${' '.repeat(indent)}${label.padEnd(34 - indent)}${money(amount).padStart(14)}`);

  for (const c of r.classes) {
    line(c.class, c.gross);
    for (const a of c.adders) line(`${a.name} @ ${a.rate}%`, a.amount, 2);
    if (c.adders.length) line(`${c.class} extended`, c.extended, 2);
  }
  rows.push('-'.repeat(48));
  line('Job cost', r.jobCost);
  line('Overhead', r.overhead);
  line('Profit', r.profit);
  line('Bond', r.bond);
  rows.push('-'.repeat(48));
  line('SELLING PRICE', r.sellingPrice);

  if (r.unpriced.length) {
    rows.push('', `${r.unpriced.length} item(s) not in the total:`);
    for (const u of r.unpriced) rows.push(`  ${u}`);
  }
  return rows.join('\n');
}
