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

import type { Basis, ClassName, Item, Money, RoundingRule, UnitStep } from './model.js';
import type { Production } from './model.js';
import { basisOf, HOURS_PER_CREW_DAY, perManHour, PRODUCTION_PER_WORDS } from './model.js';
import { namesUsed, parse, run, type FormulaResult } from './formula.js';
import { derivedLabel, derivedOrder } from './solved.js';
import { unitDisagreement } from './dimension.js';
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
   * Labor hours, at full precision. What a line displays is a rounded version
   * of what it holds, and a labor total only reconciles if the sum is taken
   * before the rounding rather than after it.
   */
  readonly hours: number | null;
  /** Crew days, for the labor lens. Derived from hours, never entered. */
  readonly crewDays: number | null;
  /** What kind of cost this is. Read off the line; never typed on it. */
  readonly basis: Basis;
  /**
   * Why this line is on the sheet and out of every total, or null.
   *
   * The money is still worked out and still shown — struck through — because
   * the question an owner asks about an alternate is what it would have cost.
   */
  readonly off: string | null;
  /**
   * Whose quote this is, on a subcontract line.
   *
   * A sub's number without a name on it is a number nobody can chase, so it is
   * read off the price source and put on the line rather than left in a field
   * three clicks away.
   */
  readonly quotedBy: string | null;
  /**
   * The rate this line was worked out at, in the words it was stated in —
   * "6 at 40 SQ a crew-day". Null on a line with no rate.
   */
  readonly rateSaid: string | null;
  /** Why this line has no money on it, in words an estimator can act on. */
  readonly pending: string | null;
  readonly formulaError?: string;
  readonly formulaErrorAt?: number;
  /**
   * The unit this line is priced in does not follow from its own formula.
   *
   * Not an error and never a refusal — the estimator declared the unit and it
   * is what prices the line, the way it is in every tool they already use. This
   * only says so, where saying so is possible at all.
   */
  readonly unitNote?: string;
}

/**
 * The lines one stored item becomes.
 *
 * Nearly always itself. The exception is an item that stands for a family —
 * `eachOf: 'TAPER_*_SF'` — which becomes one line per name of that shape the
 * model actually found, in the order the bill is read in.
 *
 * This is the difference between the fastener lines and the tapered ones, and
 * it is a fact about roofs rather than about software. A roof has three wind
 * zones and always will, so three fastener lines typed into a job are three
 * lines that stay true. How many lettered panels a package buys depends on the
 * slope: a quarter in twelve buys X, Y, Z and ZZ, an eighth buys a different
 * series altogether, and four letters typed into a job file price nothing the
 * moment anybody moves the slope — while still showing four panels this roof
 * does not buy.
 *
 * Where the model found none of them, the item comes back UNEXPANDED and
 * still carrying its pattern, and `priceLine` says so on the line. A family
 * with no members must not quietly become no line at all: a line that vanishes
 * is the silent zero wearing a different hat.
 */
export function linesFrom(
  item: Item,
  scope: Readonly<Record<string, number | null>>,
): readonly Item[] {
  if (item.eachOf === undefined) return [item];
  const pattern = item.eachOf;
  const star = pattern.indexOf('*');
  if (star < 0) return [item];
  const before = pattern.slice(0, star);
  const after = pattern.slice(star + 1);
  const found = Object.keys(scope).filter(
    (name) => name.length > pattern.length - 1
      && name.startsWith(before) && name.endsWith(after),
  );
  if (found.length === 0) return [item];
  return derivedOrder(found).map((name) => {
    const part = name.slice(before.length, name.length - after.length);
    const { eachOf: _pattern, ...rest } = item;
    return {
      ...rest,
      id: `${item.id}-${part.toLowerCase()}`,
      description: item.description.split('*').join(part),
      formula: item.formula.split(pattern).join(name),
      ...(item.libraryId === undefined
        ? {}
        : { libraryId: item.libraryId.split('*').join(part.toLowerCase()) }),
    };
  });
}

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

/**
 * The measures a page's scale answers. A null one of these IS the unscaled sheet.
 *
 * `EA`, `VERTICES` and `SEGMENTS` are never null — counting needs no scale — so
 * the list is the four that are.
 */
const FROM_THE_SHEET: readonly string[] = ['SF', 'PLAN_SF', 'SQ', 'LF'];

/**
 * Why a formula came out with no number.
 *
 * Until the model started solving things, the only way a quantity could be null
 * was an unscaled sheet, so that is what this said for all of them — and a solve
 * that answers `CRICKET_SF: null` got a sentence blaming the estimator's sheet for
 * a gap in the solver. It is not their sheet. So the formula is parsed, the names
 * it uses are looked up in the scope, and the first one the solve carries and
 * cannot answer is NAMED.
 *
 * A name that is absent from the scope entirely is a different case and is not
 * this one: the formula parser already answers that with the name back, as an
 * error on the line, and it never reaches here.
 */
function whyThereIsNoNumber(
  formula: string,
  scope: Readonly<Record<string, number | null>>,
): string {
  try {
    for (const name of namesUsed(parse(formula))) {
      if (scope[name] === null && !FROM_THE_SHEET.includes(name)) {
        // In the trade's words and not the formula's. The identifier is already
        // on the line in the mono face, where a formula belongs; a sentence
        // carrying `TAPER_ZZ_SF` is a code word on screen, which this program
        // does not do.
        return `${derivedLabel(name).label} has not been worked out.`;
      }
    }
  } catch {
    // A formula that will not parse has already said so, above this call.
  }
  return 'the sheet this comes off has no scale yet';
}

/**
 * The rate, said the way it was stated: "6 at 40 SQ a crew-day".
 *
 * Not the man-hour figure the arithmetic runs on. A foreman checking a bid
 * reads the crew and the day, and a line that printed 0.83 units a man-hour
 * would be asking them to do the division back in their head to find out
 * whether the estimator believed something reasonable.
 */
export function rateSaid(production: Production): string {
  const crew = production.crew && production.crew > 0 ? `${production.crew} at ` : '';
  return `${crew}${round2(production.rate)} ${PRODUCTION_PER_WORDS[production.per]}`;
}

/** Two places, and no trailing zeros — 40, never 40 with two zeros after it. */
const round2 = (v: number): string => String(Math.round(v * 100) / 100);

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
  conditionWaste = 0,
  cls: ClassName | null = null,
): LineResult {
  // An unexpanded family carries its pattern in the formula, which is not
  // arithmetic — `TAPER_*_SF` reads as a multiplication of two names that do
  // not exist. It is not run: the line says what it is waiting for instead.
  const formula: FormulaResult = item.eachOf === undefined
    ? run(item.formula, scope)
    : { value: null };

  // Only worth asking when the formula parses at all.
  let unitNote: string | null = null;
  if (formula.error === undefined) {
    try {
      unitNote = unitDisagreement(parse(item.formula), item.unit);
    } catch {
      unitNote = null;
    }
  }
  const quantity = formula.value;

  // Two waste terms, applied in order and kept apart. The item's belongs to the
  // material; the condition's belongs to this run. Multiplying them rather than
  // adding keeps each one meaning what it says — 10% of material waste on a run
  // that already carries 5% is 15.5%, not 15%, and the estimator can still see
  // which factor is which.
  const waste = item.waste ?? 0;
  const withWaste = quantity === null
    ? null
    : settle(quantity * (1 + waste / 100) * (1 + conditionWaste / 100));

  // ── what you buy ────────────────────────────────────────────────────────
  // On a labor line the order unit is HOURS and the production rate is the
  // conversion, so labor is not a special case here — it is this step with a
  // rate in it and nothing rounded.
  const perHour = item.production === undefined ? null : perManHour(item.production);
  const orderStep: UnitStep | undefined = item.order
    ?? (item.production === undefined
      ? undefined
      // A rate that cannot be carried — a crew-day with no crew — still makes
      // this a labor line, and the step with no factor on it is what says so.
      // Falling back to no step at all would price the line by the square and
      // look entirely right.
      : { name: 'HOURS', ...(perHour === null ? {} : { per: perHour }), rule: 'exact' });

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
  // A price typed on the line wins; then the book's price for the item this
  // line came from; then the book's price for the line itself, which is what a
  // line authored by hand rather than loaded looks like.
  const unitCost = item.unitCost
    ?? scenarioPrices[item.id]
    ?? (item.libraryId === undefined ? undefined : scenarioPrices[item.libraryId])
    ?? null;
  const extended = priceQuantity === null || unitCost === null
    ? null
    : priceQuantity * unitCost;

  // Hours at full precision. Rounding here is how a labor total ends up three
  // dollars adrift of the report it is being checked against.
  const hours = orderStep?.name === 'HOURS' ? orderQuantity : null;
  const crew = item.production?.crew;
  const crewDays = hours === null || !crew || crew <= 0
    ? null
    : hours / crew / HOURS_PER_CREW_DAY;

  let pending: string | null = null;
  // A family the model found no members of. It is still one line on the sheet,
  // saying what happened, because a line that disappears is a line nobody can
  // ask about — and the question here is a real one: the slope may have moved
  // into a chart this roof has no panels in.
  if (item.eachOf !== undefined) {
    pending = `the model found nothing of this kind on this roof (${item.eachOf})`;
  } else if (formula.error) pending = formula.error;
  else if (quantity === null) pending = whyThereIsNoNumber(item.formula, scope);
  // A rate with no crew under it is the one that reads as arithmetic and is
  // really a sentence with half of it missing, so it says which half.
  else if (orderIncomplete && orderStep!.name === 'HOURS') {
    pending = 'this rate says nothing about the crew it assumes';
  } else if (orderIncomplete) pending = `${orderStep!.name} has no conversion`;
  else if (priceIncomplete) pending = `${item.price!.name} has no conversion`;
  else if (unitCost === null) pending = 'no price';

  return {
    item,
    basis: basisOf(item, cls),
    off: item.off?.reason ?? null,
    quotedBy: item.priceSource?.from ?? null,
    rateSaid: item.production === undefined ? null : rateSaid(item.production),
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
    ...(unitNote === null ? {} : { unitNote }),
    ...(formula.error === undefined ? {} : { formulaError: formula.error }),
    ...(formula.position === undefined ? {} : { formulaErrorAt: formula.position }),
  };
}
