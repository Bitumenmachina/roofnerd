// ── Lenses ─────────────────────────────────────────────────────────────────
// One estimate, several audiences.
//
// A lens is a template, not a report: a set of columns and a list of what to
// leave out. It hides lines and columns and it never edits the estimate — the
// numbers a lens shows are the same numbers the sheet shows, filtered.
//
// This is a print filter and not a login system. There are no accounts, no
// permissions and no users. What stops a cost reaching a supply house is that
// the supplier's lens has no column for one, and the value is never put into
// the document at all — not greyed, not blurred, not present. A number that is
// in the page and merely hidden is a number one "view source" away from the
// wrong person.

import type { Unit } from './units.js';

/** What a lens can leave out. */
export type Conceal = 'cost' | 'waste' | 'notes' | 'labor' | 'derivation';

/** One column a lens can show. */
export interface LensColumn {
  readonly key: string;
  readonly heading: string;
  /** Right-aligned, tabular figures — anything an estimator adds up. */
  readonly numeric?: boolean;
}

export interface Lens {
  readonly id: string;
  readonly name: string;
  /** Who it is for, in the words you would use handing it over. */
  readonly audience: string;
  readonly columns: readonly LensColumn[];
  readonly conceal: readonly Conceal[];
  /** What it says about itself, printed on the sheet. */
  readonly note?: string;
}

const col = (key: string, heading: string, numeric = false): LensColumn => ({ key, heading, numeric });

/**
 * The five that ship.
 *
 * Each one exists because somebody outside the office asks for it and the
 * answer used to be retyping. Nothing here is a new report — they are the same
 * estimate seen from five sides.
 */
export const LENSES: readonly Lens[] = [
  {
    id: 'drawing',
    name: 'Drawing',
    audience: 'the subcontractor pricing the work',
    columns: [col('condition', 'Condition'), col('item', 'Item'), col('quantity', 'Quantity', true), col('unit', 'Unit')],
    conceal: ['cost', 'waste', 'notes', 'derivation'],
    note: 'Quantities as measured. No costs, and no internal assumptions.',
  },
  {
    id: 'stocking',
    name: 'Stocking',
    audience: 'the supply house and the crew on the roof',
    // Order units and nothing else: what to put on a truck. Waste is out
    // because it is the estimator's cushion, not the supplier's business.
    columns: [col('item', 'Item'), col('orderQuantity', 'Order', true), col('orderUnit', 'Unit'), col('sent', 'Sent'), col('returned', 'Returned')],
    conceal: ['cost', 'waste', 'notes', 'labor', 'derivation'],
    note: 'What to send. Sent and Returned are left blank for the field to fill in.',
  },
  {
    id: 'condition-summary',
    name: 'Condition Summary',
    audience: 'the project manager',
    columns: [col('condition', 'Condition'), col('quantity', 'Quantity', true), col('unit', 'Unit'), col('cost', 'Cost', true)],
    conceal: ['waste', 'notes', 'derivation'],
    note: 'One line per condition, with what it costs to build.',
  },
  {
    id: 'recap',
    name: 'Recap',
    audience: 'accounting, and the owner',
    columns: [col('class', 'Class'), col('cost', 'Cost', true), col('perSquare', 'Per SQ', true)],
    conceal: ['waste', 'notes', 'derivation'],
    note: 'The roll-up to selling price, by class.',
  },
  {
    id: 'consolidated',
    name: 'Consolidated',
    audience: 'this office, and nobody else',
    // The only lens that shows the working. It is the internal one, and it says
    // so on its face so it cannot be handed over by accident.
    columns: [
      col('costCode', 'Code'), col('item', 'Item'), col('formula', 'Formula'),
      col('quantity', 'Quantity', true), col('unit', 'Unit'), col('waste', 'Waste', true),
      col('orderQuantity', 'Order', true), col('orderUnit', 'Order unit'), col('cost', 'Cost', true),
    ],
    conceal: [],
    note: 'Internal. Shows the whole chain, including how every quantity was worked out.',
  },
];

export const lensById = (id: string): Lens | undefined => LENSES.find((l) => l.id === id);

/** Does this lens keep a thing out? */
export const conceals = (lens: Lens, what: Conceal): boolean => lens.conceal.includes(what);

/**
 * The words a formula is written in.
 *
 * A client-facing lens must never carry these, and it is worth testing rather
 * than intending: the prior lineage found derivation columns on a document that
 * had gone out, and the check that would have caught it is a search of the
 * finished page for exactly this list.
 */
export const DERIVATION_WORDS: readonly string[] = [
  // Measures a formula can name that are not units anybody prints.
  'PLAN_SF', 'VERTICES', 'SEGMENTS',
  // Properties.
  'STRETCHOUT', 'PITCH', 'TAPER', 'ELEV', 'SUMP', 'BOARDS',
  // Functions.
  'ceil(', 'floor(', 'round(',
];

/**
 * `SF`, `LF`, `EA` and `SQ` are deliberately NOT in that list.
 *
 * They are the units, and a quantity on a subcontractor's drawing lens prints
 * one beside every number — a check that treated them as leaks would fire on
 * every honest page, which is how a check gets switched off in a week. What
 * must never travel is the *working*: the measure names an estimator only sees
 * inside a formula, the properties, and the rounding functions.
 */
export const printableUnit = (u: Unit): string => u;
