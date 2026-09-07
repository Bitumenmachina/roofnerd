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
  readonly notes?: string;
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
  readonly assemblyId?: string;
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
