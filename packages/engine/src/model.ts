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

import type { Measure, Unit } from './units.js';

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
 * What the supply house sells, and how what you measured becomes what you
 * order. Waste is applied first; packaging rounds up after.
 */
export interface OrderUnit {
  /** ROLL, SHEET, BOX, 5GAL, TUBE — the supplier's word, not ours. */
  readonly name: string;
  /** How many estimating units come in one of them. */
  readonly per: number;
  /** Waste, as a percent, applied before the packaging round-up. */
  readonly waste?: Percent;
}

/** One thing a condition consumes. */
export interface Item {
  readonly id: string;
  readonly description: string;
  readonly costCode: string;
  /** The unit this item is estimated in — not necessarily the condition's. */
  readonly unit: Unit;
  /** How the condition's measure becomes this item's quantity. */
  readonly formula: Formula;
  readonly unitCost?: Money;
  readonly orderUnit?: OrderUnit;
  /**
   * Units this crew puts in per hour. Present on labor items; hours come from
   * it and the quantity, and crew days come from hours. Never typed directly.
   */
  readonly productionRate?: number;
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
 * Properties that come off the drawing detail rather than off the trace. A
 * parapet is a line until you say how tall it is; then it is also an area.
 */
export interface ConditionProperties {
  readonly height?: number;
  readonly width?: number;
  readonly sides?: number;
  readonly pitch?: string;
}

/**
 * One thing you traced. It carries square feet, linear feet and a count at the
 * same time, because one parapet run is all three and pretending otherwise is
 * what makes estimating software annoying.
 */
export interface Condition {
  readonly id: string;
  readonly name: string;
  readonly pageId: string;
  readonly properties: ConditionProperties;
  readonly measures: readonly Measure[];
  readonly assemblyId?: string;
  readonly items: readonly Item[];
  readonly notes?: string;
}

// ── Pages, scenarios, the job ──────────────────────────────────────────────

/** One drawing or aerial photo being traced. */
export interface Page {
  readonly id: string;
  readonly name: string;
  /** Path relative to the job folder. The source file is never copied inline. */
  readonly source?: string;
  /** Feet per drawing unit, from the two-point scale. Absent until scaled. */
  readonly scale?: number;
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
