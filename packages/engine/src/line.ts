// ── An estimate line ───────────────────────────────────────────────────────
// One item on one condition, worked all the way out: formula → quantity →
// waste → order units → money.
//
// Everything is kept as a separate step and every step is shown, because the
// question an estimator asks a bid is never "what is the total" — it is "why is
// that the total", and a program that cannot answer it gets checked by hand
// against a spreadsheet, which means it has failed.

import type { Item, Money } from './model.js';
import { run, type FormulaResult } from './formula.js';

/**
 * Drop the last crumbs of binary floating-point noise.
 *
 * Nine decimal places is far below anything an estimator measures and far above
 * the ~1e-14 error that arithmetic like `100 * 1.1` leaves behind. It matters
 * only where a value meets a rounding step — which is exactly where a bid gains
 * a roll it does not need.
 */
const settle = (v: number): number => Math.round(v * 1e9) / 1e9;

export interface LineResult {
  readonly item: Item;
  /** What the formula produced, before waste. Null when it cannot be worked out. */
  readonly quantity: number | null;
  /** The unit the formula produces — what `quantity` is counted in. */
  readonly unit: string;
  /** Quantity with waste added, still in estimating units. */
  readonly withWaste: number | null;
  /** How many of the supplier's package that is, rounded the way it is sold. */
  readonly orderQuantity: number | null;
  /** ROLL, SHEET, BOX — what the supply house calls it. */
  readonly orderUnitName: string | null;
  readonly unitCost: Money | null;
  /** Quantity times unit cost. Null if either is unknown — never a silent zero. */
  readonly extended: Money | null;
  /** Labor hours, when the item carries a production rate. */
  readonly hours: number | null;
  /** Why this line has no money on it, in words an estimator can act on. */
  readonly pending: string | null;
  /** What is wrong with the formula itself, and where in the text. */
  readonly formulaError?: string;
  readonly formulaErrorAt?: number;
}

/**
 * Work one item out against what its condition measures.
 *
 * `scope` is the condition's measures and properties — SF, LF, EA, SQ, PLAN_SF,
 * H, W, T, PITCH, SIDES, STRETCHOUT and anything the estimator named.
 */
export function priceLine(
  item: Item,
  scope: Readonly<Record<string, number | null>>,
  scenarioPrices: Readonly<Record<string, Money>> = {},
): LineResult {
  const formula: FormulaResult = run(item.formula, scope);

  const quantity = formula.value;
  const waste = item.orderUnit?.waste ?? 0;
  const withWaste = quantity === null ? null : settle(quantity * (1 + waste / 100));

  const order = item.orderUnit;
  const orderQuantity = withWaste === null || !order || !(order.per > 0)
    ? null
    // Packaging rounds up. You cannot buy two thirds of a bucket, and an
    // estimate that pretends you can is short on the day.
    //
    // `settle` first, and it is not fussiness: 100 with 10% waste is
    // 110.00000000000001 in binary floating point, and a bare ceil turns that
    // into twelve rolls instead of eleven. An estimator would find it by
    // counting the pallet.
    : Math.ceil(settle(withWaste / order.per));

  // An item's own price wins over the scenario's. A price typed on the line is
  // a quote in hand; the scenario is the book.
  const unitCost = item.unitCost ?? scenarioPrices[item.id] ?? null;

  // Money is charged on what gets bought, not on what gets installed: if it is
  // sold by the sheet, the bid pays for whole sheets.
  const billable = orderQuantity !== null && order ? orderQuantity * order.per : withWaste;
  const extended = billable === null || unitCost === null ? null : billable * unitCost;

  const hours = quantity === null || item.productionRate === undefined || !(item.productionRate > 0)
    ? null
    : quantity / item.productionRate;

  let pending: string | null = null;
  if (formula.error) pending = formula.error;
  else if (quantity === null) pending = 'the sheet this comes off has no scale yet';
  else if (unitCost === null) pending = 'no price';

  return {
    item,
    quantity,
    unit: item.unit,
    withWaste,
    orderQuantity,
    orderUnitName: order?.name ?? null,
    unitCost,
    extended,
    hours,
    pending,
    ...(formula.error === undefined ? {} : { formulaError: formula.error }),
    ...(formula.position === undefined ? {} : { formulaErrorAt: formula.position }),
  };
}
