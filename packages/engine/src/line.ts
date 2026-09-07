// ── An estimate line ───────────────────────────────────────────────────────
// One item on one condition, worked all the way from what was traced to what
// gets paid:
//
//   formula → quantity → waste → order units → price units → money
//
// Every step is kept separate and every step is shown, because the question an
// estimator asks a bid is never "what is the total" — it is "why is that the
// total". A program that cannot answer gets checked by hand against a
// spreadsheet, which means it has failed.
//
// Three units, not two. A real supply house quotes membrane by the square foot,
// sells it by the roll, and you estimated it in squares. Collapsing that into
// one conversion is where a bid quietly gains or loses money.

import type { Item, Money, RoundingRule, UnitStep } from './model.js';
import { run, type FormulaResult } from './formula.js';
import { ceilPackages, settle } from './rounding.js';

export interface LineResult {
  readonly item: Item;
  /** What the formula produced, before waste. Null when it cannot be worked out. */
  readonly quantity: number | null;
  /** The unit the formula produces — what `quantity` is counted in. */
  readonly unit: string;
  /** Quantity with waste added, still in estimating units. */
  readonly withWaste: number | null;
  /** How much of the supplier's package that is, under this item's rule. */
  readonly orderQuantity: number | null;
  readonly orderUnitName: string | null;
  /** How much of whatever the price is quoted against. */
  readonly priceQuantity: number | null;
  readonly priceUnitName: string | null;
  readonly unitCost: Money | null;
  /** Price quantity times unit cost. Null if either is unknown — never a silent zero. */
  readonly extended: Money | null;
  /**
   * Labor hours, at full precision. A displayed 4,566.87 is a rounded 4,566.87…,
   * and totals only reconcile if the sum is taken before the rounding.
   */
  readonly hours: number | null;
  /** Crew days, for the labor lens. Derived from hours, never entered. */
  readonly crewDays: number | null;
  /** Why this line has no money on it, in words an estimator can act on. */
  readonly pending: string | null;
  readonly formulaError?: string;
  readonly formulaErrorAt?: number;
}

/** Hours in a crew's working day. A library number later; this is the default. */
export const HOURS_PER_CREW_DAY = 8;

/** Whether a step has been given a conversion at all. */
const hasFactor = (step: UnitStep): boolean =>
  (step.contains !== undefined && step.contains > 0) || (step.per !== undefined && step.per > 0);

/**
 * Carry a quantity across one step.
 *
 * `per` and `contains` are the same fact written from either end, and both
 * occur in real supply catalogues: four rolls to a box, but a thousand square
 * feet in a roll. Whichever way round the estimator has it in front of them is
 * the way they should be able to type it.
 *
 * `per` divides rather than multiplying by its reciprocal. It is not the same
 * arithmetic: 10,000 / 3 and 10,000 * (1/3) differ in the last place, and the
 * whole point of the hours rule is that the last place survives to the total.
 */
function convert(value: number, step: UnitStep): number | null {
  if (step.contains !== undefined && step.contains > 0) return value * step.contains;
  if (step.per !== undefined && step.per > 0) return value / step.per;
  return null;
}

function applyRule(value: number, rule: RoundingRule): number {
  // Not always up. In one real job rolls and sheets round up while fastener
  // plates order at 4.94 boxes, because that is what the supplier bills.
  //
  // `exact` is returned untouched, deliberately. Settling exists to protect a
  // ROUNDING decision from binary crumbs; applying it to a value that is not
  // being rounded only throws precision away. Hours run through this step, and
  // hours have to keep every digit — a labor total reconciles from unrounded
  // hours summed per line, and rounding here is how it drifts.
  return rule === 'ceil' ? ceilPackages(value) : value;
}

/**
 * Work one item out against what its condition measures.
 *
 * `scope` is the condition's measures and properties — SF, LF, EA, SQ, PLAN_SF,
 * VERTICES, SEGMENTS, H, W, T, PITCH, SIDES, STRETCHOUT and anything the
 * estimator named.
 */
export function priceLine(
  item: Item,
  scope: Readonly<Record<string, number | null>>,
  scenarioPrices: Readonly<Record<string, Money>> = {},
): LineResult {
  const formula: FormulaResult = run(item.formula, scope);
  const quantity = formula.value;

  const waste = item.waste ?? 0;
  const withWaste = quantity === null ? null : settle(quantity * (1 + waste / 100));

  // ── what you buy ────────────────────────────────────────────────────────
  // On a labor line the order unit is HOURS and the production rate is the
  // conversion, so labor is not a special case here — it is this step with a
  // rate in it and nothing rounded.
  const orderStep: UnitStep | undefined = item.order
    ?? (item.productionRate !== undefined && item.productionRate > 0
      ? { name: 'HOURS', per: item.productionRate, rule: 'exact' }
      : undefined);

  const orderIncomplete = orderStep !== undefined && !hasFactor(orderStep);
  const orderCarried = withWaste === null || !orderStep ? null : convert(withWaste, orderStep);
  const orderQuantity = orderCarried === null || !orderStep
    ? null
    : applyRule(orderCarried, orderStep.rule);

  // ── what the price is quoted against ────────────────────────────────────
  // An order step that was started and never finished must not quietly fall
  // back to pricing the estimating unit: that prices membrane by the square
  // instead of by the square foot and looks entirely plausible on the sheet.
  const priceIncomplete = item.price !== undefined && !hasFactor(item.price);

  let priceQuantity: number | null;
  if (orderIncomplete || priceIncomplete) {
    priceQuantity = null;
  } else if (item.price === undefined) {
    priceQuantity = orderQuantity ?? withWaste;
  } else if (orderQuantity === null) {
    priceQuantity = null;
  } else {
    const carried = convert(orderQuantity, item.price);
    priceQuantity = carried === null ? null : applyRule(carried, item.price.rule);
  }

  // An item's own price wins over the scenario's. A price typed on the line is
  // a quote in hand; the scenario is the book.
  const unitCost = item.unitCost ?? scenarioPrices[item.id] ?? null;
  const extended = priceQuantity === null || unitCost === null
    ? null
    : priceQuantity * unitCost;

  // Hours at full precision. Rounding here is how a labor total ends up three
  // dollars adrift of the report it is being checked against.
  const hours = orderStep?.name === 'HOURS' ? orderQuantity : null;
  const crewDays = hours === null || !item.crewSize || item.crewSize <= 0
    ? null
    : hours / item.crewSize / HOURS_PER_CREW_DAY;

  let pending: string | null = null;
  if (formula.error) pending = formula.error;
  else if (quantity === null) pending = 'the sheet this comes off has no scale yet';
  else if (orderIncomplete) pending = `${orderStep!.name} has no conversion`;
  else if (priceIncomplete) pending = `${item.price!.name} has no conversion`;
  else if (unitCost === null) pending = 'no price';

  return {
    item,
    quantity,
    unit: item.unit,
    withWaste,
    orderQuantity,
    orderUnitName: orderStep?.name ?? null,
    priceQuantity,
    priceUnitName: item.price?.name ?? orderStep?.name ?? item.unit,
    unitCost,
    extended,
    hours,
    crewDays,
    pending,
    ...(formula.error === undefined ? {} : { formulaError: formula.error }),
    ...(formula.position === undefined ? {} : { formulaErrorAt: formula.position }),
  };
}
