// ── The vocabulary, as types ───────────────────────────────────────────────
// This file is the word list from the README, written so the compiler enforces
// it. If a screen or a file uses a word that is not here, that is the defect.
//
// The shape nests the way a bid does:
//
//   Job → Scenario (a set of prices)
//       → Page (a drawing) → Condition (a thing you traced) → Item (what it eats)
//
// A cost code sits on every item. Every code belongs to a class. The recap adds
// up by class, which is the only way it can be honest.

import type { Unit } from './units.js';
import type { Properties, Trace, TraceKind } from './measures.js';

/** The program's own name. Renaming it is this line, tauri.conf.json, the README. */
export const PRODUCT_NAME = 'roofnerd';

/** The file format's version. Bumped when a job folder's shape changes. */
export const JOB_FORMAT = 1;

// ── Money ──────────────────────────────────────────────────────────────────

/** Dollars. Kept as a number; rounding happens once, at the point of display. */
export type Money = number;

/** A rate written the way an estimator says it: 8.5 means eight and a half percent. */
export type Percent = number;

// ── Cost codes and classes ─────────────────────────────────────────────────

/**
 * The recap bucket a cost rolls into. Each one carries its own adders, which is
 * what stops a dumpster from being counted as roofing material.
 */
export type ClassName =
  | 'Material'
  | 'Labor'
  | 'Sub'
  | 'Equipment'
  | 'Other'
  | 'Supervision';

export const CLASS_NAMES: readonly ClassName[] = [
  'Material', 'Labor', 'Sub', 'Equipment', 'Other', 'Supervision',
] as const;

/**
 * A cost code is the estimator's own bucket. The code string is theirs — this
 * program ships a 07-xxx pattern as a starting point and no standard's text.
 */
export interface CostCode {
  readonly code: string;
  readonly name: string;
  readonly class: ClassName;
}

/**
 * What each class adds on top of its gross, before overhead and profit.
 * Every one of these is editable. Defaults are zero: this program does not
 * invent a tax rate or a burden rate on an estimator's behalf.
 */
export interface ClassAdders {
  /** Sales tax, on material. */
  readonly tax?: Percent;
  /** Price escalation between bid and buyout, on material. */
  readonly escalation?: Percent;
  /** Labor burden. Supervision carries its own, at its own rate. */
  readonly burden?: Percent;
  /** General liability, on subcontract. */
  readonly generalLiability?: Percent;
}

// ── Items and formulas ─────────────────────────────────────────────────────

/**
 * The arithmetic that turns what you measured into how much of an item you
 * need. Kept as the text the estimator typed so it can be shown on the line and
 * edited there. A formula hidden in a library is a formula nobody trusts.
 */
export type Formula = string;

/**
 * Whether a step rounds up to a whole one, or takes the number as it falls.
 *
 * Do not assume it is always up. In one real job, rolls and sheets round up
 * while fastener plates order at 4.94 boxes — the supplier bills the fraction.
 * The rule belongs to the item and to the step, not to the program.
 */
export type RoundingRule = 'ceil' | 'exact';

/**
 * One conversion in the chain from what was measured to what gets paid for.
 *
 * The factor is given in whichever direction reads naturally to the person
 * typing it, because both directions occur and neither is the odd one out:
 *
 *   `per`      how many of the PREVIOUS unit make one of this one
 *              — four rolls to a box, twenty cartridges to a case
 *   `contains` how many of THIS unit are in one of the previous
 *              — a thousand square feet in a roll of membrane
 *
 * Exactly one of them is given. They are the same fact written from either end.
 */
export interface UnitStep {
  /** ROLL, SHEET, BOX, CASE, 5GAL, TUBE, HOURS — the supplier's word, not ours. */
  readonly name: string;
  readonly per?: number;
  readonly contains?: number;
  readonly rule: RoundingRule;
}

/**
 * One thing a condition consumes.
 *
 * A line carries three units, not two, because a real supply house uses three:
 *
 *   estimating unit  what the formula produces — the measure you work in
 *   order unit       what you actually buy — rolls, sheets, buckets, hours
 *   price unit       what the price is quoted against — which need not be
 *                    either of the others
 *
 * Membrane is estimated in squares, ordered in rolls, and priced by the square
 * foot. Flashing tape is estimated in feet, ordered in rolls, and priced by the
 * box. Each step has its own conversion and its own rounding, and both are shown
 * on the line, because this is exactly where a bid quietly gains or loses money.
 */
export interface Item {
  readonly id: string;
  readonly description: string;
  readonly costCode: string;
  /** The unit this item is estimated in — not necessarily the condition's. */
  readonly unit: Unit;
  /** How the condition's measure becomes this item's quantity. */
  readonly formula: Formula;
  /** Waste, as a percent, added before anything is ordered. */
  readonly waste?: Percent;
  /** Estimating unit → what you buy. Absent means you buy what you measured. */
  readonly order?: UnitStep;
  /** What you buy → what it is priced against. Absent means priced as ordered. */
  readonly price?: UnitStep;
  /** The price, per price unit — or per order unit, or per estimating unit, whichever is last. */
  readonly unitCost?: Money;
  /**
   * Units this crew puts in per hour. On a labor line this IS the order-step
   * conversion: the order unit is HOURS and the rate turns the measure into
   * them. Crew days come from hours; hours are never typed directly, except on
   * supervision, which is entered as hours to begin with.
   */
  readonly productionRate?: number;
  /** How many are in the crew. Derives crew-days for the labor lens only. */
  readonly crewSize?: number;
  /** Where the price came from and how firm it is. */
  readonly priceSource?: PriceSource;
  /** The profile this item is formed from, when it is sheet metal. */
  readonly profileId?: string;
  readonly notes?: string;
}

/**
 * How good a price is.
 *
 * Four independent arrivals said a price needs this: NEXUS types it and ships
 * the colour language, the prior estimating lineage reached it as a basis
 * chain, the design work draws it as a chip, and this program had it as prose.
 *
 * It is its own axis and never collapses into the other two. **Authorship** is
 * who wrote a number — derived or typed. **Validity** is whether it can be used
 * — usable, pending, excluded, no price. **Firmness** is how good a price is,
 * and a firm price can sit on a pending quantity without either being wrong.
 */
export type Firmness = 'firm' | 'budget' | 'estimate' | 'placeholder';

export const FIRMNESS: readonly Firmness[] = ['firm', 'budget', 'estimate', 'placeholder'] as const;

/** Where a price came from, and how much to trust it. */
export interface PriceSource {
  /** Who or what it came from — a supply house, a quote number, a book. */
  readonly from?: string;
  readonly firmness?: Firmness;
  /** When it was good. A price with no date is a price nobody can defend. */
  readonly on?: string;
  /**
   * Why this number is doubtful, when it is — in the words you would say to
   * the supplier.
   *
   *
   * The prior lineage carried two sheet-metal figures it knew disagreed with
   * the field and shipped them saying so rather than silently correcting or
   * silently keeping them. A value that admits it is doubtful is worth more
   * than one that looks settled and is not.
   */
  readonly doubt?: string;
}

/**
 * A formed metal profile: the flat legs it is brake-formed from.
 *
 * Girth is their sum plus what the hems eat, and it belongs to the **detail**
 * rather than to the material — 19½ inches at a coping, 8 at a gravel stop, 26
 * at an equipment curb cap, all off the same coil. That is why it can never be
 * a constant on a library item, and it is the case the whole formula language
 * exists for: linear feet of profile become pounds through a width that changes
 * with the detail.
 */
export interface Profile {
  readonly id: string;
  readonly name: string;
  /** The flat legs in inches, in the order they come off the brake. */
  readonly legs: readonly number[];
  /** What each hem eats, in inches. A hem folds back, so it is not a leg. */
  readonly hems?: readonly number[];
  /** Pounds per square foot of the coil, for turning girth into weight. */
  readonly weightPerSF?: number;
  readonly source?: PriceSource;
  readonly notes?: string;
}

/**
 * Girth: everything the profile eats across the coil, in inches.
 *
 * Entered once for a detail and reused everywhere that detail runs. There is no
 * table to look this up in and there never was — an estimator who can draw the
 * detail can state its legs.
 */
export function girthOf(profile: Profile): number {
  const legs = profile.legs.reduce((a, b) => a + b, 0);
  const hems = (profile.hems ?? []).reduce((a, b) => a + b, 0);
  return legs + hems;
}

/** A saved set of items. Generic, or a named manufacturer's system. */
export interface Assembly {
  readonly id: string;
  readonly name: string;
  readonly manufacturer?: string;
  readonly items: readonly Item[];
  /** A picture of the tile or the detail, so the right one is confirmed by sight. */
  readonly image?: string;
  readonly notes?: string;
}

// ── The traced thing ───────────────────────────────────────────────────────

/**
 * One thing you traced. It carries square feet, linear feet and a count at the
 * same time, because one parapet run is all three and pretending otherwise is
 * what makes estimating software annoying.
 */
/**
 * What a condition is to the water on the roof.
 *
 * A drain is the low point a tapered field falls to; a ridge is the line a
 * cricket is built along, between two of them. Everything else has no role,
 * which is the normal case.
 */
export type ConditionRole = 'drain' | 'ridge';

export const CONDITION_ROLES: readonly ConditionRole[] = ['drain', 'ridge'] as const;

export interface Condition {
  readonly id: string;
  readonly name: string;
  /** An area on the sheet, a run along it, or things counted. */
  readonly kind: TraceKind;
  /** The shapes drawn for it, on any number of pages. */
  readonly traces: readonly Trace[];
  /** What the drawing cannot say: height, width, slope, stretch-out. */
  readonly properties: Properties;
  /**
   * Take another condition's measures as this one's own. The parapet a coping
   * runs along is measured once; the coping says where it came from rather than
   * being traced twice, and the two can never drift apart.
   */
  readonly from?: string;
  /**
   * What this condition is to the roof's drainage, when it is anything.
   *
   * Most conditions have no role — a parapet is a parapet. But a heightfield
   * cannot be solved without knowing which counts are drains, and a cricket
   * cannot be built without knowing which run is its ridge. That is a fact
   * about the thing, not a number on it, so it cannot be a property: every
   * property is a number.
   *
   * It is a fixed set rather than free text on purpose. Reading "drain" off a
   * condition's *name* works until somebody types "Drains", or "RD-1", or
   * "roof drains (typ)" — and then the roof silently has no low points and the
   * taper solves to nothing.
   */
  readonly role?: ConditionRole;
  /**
   * The conditions this one runs between — a cricket's ridge, which sits
   * between two drains.
   *
   * `from:` inherits one parent's measures. This is a different relation: a
   * ridge is *defined by* two drains without taking either one's measures, and
   * the two planes either side of it are what the geometry is for.
   */
  readonly between?: readonly string[];
  readonly assemblyId?: string;
  /**
   * Waste for this run in particular, on top of whatever each item wastes.
   *
   * Two different facts, and they do not collapse into one number. An item's
   * waste is a property of the material — this membrane always wastes so much,
   * wherever it is used, and it comes down from the library with the item. A
   * condition's waste is a property of the *situation*: this run gets extra
   * because of the cut pattern, the access, the number of penetrations. An
   * estimator needs to see which of the two moved a quantity, so it composes
   * after the item's and stays visible as its own factor.
   */
  readonly waste?: Percent;
  readonly items: readonly Item[];
  /** A colour to draw it in on the sheet. */
  readonly color?: string;
  readonly notes?: string;
}

// ── Pages, scenarios, the job ──────────────────────────────────────────────

/** One drawing or aerial photo being traced. */
export interface Page {
  readonly id: string;
  readonly name: string;
  /**
   * Path relative to the job folder. The drawing itself is never copied into
   * the job file — a job stays small and readable, and the PDF stays the PDF.
   */
  readonly source?: string;
  /** Which page of that PDF. Absent for an image. */
  readonly pageNumber?: number;
  /**
   * Feet per page unit, once somebody has scaled the sheet. Absent means not
   * scaled, and everything measured on it reads as pending rather than zero.
   */
  readonly feetPerUnit?: number;
  /** How the scale was set, so it can be seen and argued with. */
  readonly scaleNote?: string;
}

/** One set of prices for the job. A supply house, or a pricing date. */
export interface Scenario {
  readonly id: string;
  readonly name: string;
  readonly prices: Readonly<Record<string, Money>>;
  readonly adders: Readonly<Partial<Record<ClassName, ClassAdders>>>;
  readonly overhead?: Percent;
  readonly profit?: Percent;
  readonly bond?: Percent;
}

/** One bid. */
export interface Job {
  readonly format: number;
  readonly name: string;
  readonly scenarios: readonly Scenario[];
  readonly activeScenarioId?: string;
}

/** A job folder, opened: the identity plus everything under it. */
export interface JobDocument {
  readonly job: Job;
  readonly pages: readonly Page[];
  readonly conditions: readonly Condition[];
  readonly costCodes: readonly CostCode[];
}

// ── Lenses ─────────────────────────────────────────────────────────────────

/**
 * A report: one estimate, filtered for one party. This is a print filter and
 * nothing more — there are no accounts, no logins and no permissions here.
 */
export type LensName = 'Drawing' | 'Stocking' | 'Condition Summary' | 'Recap' | 'Consolidated';
