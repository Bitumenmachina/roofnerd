// ── Units ──────────────────────────────────────────────────────────────────
// Four units, spelled these four ways, everywhere in this program:
//
//   SF  square feet      LF  linear feet      EA  each (a count)      SQ  squares
//
// A measurement is never a bare number here. It is a value welded to its unit,
// so a function that wants linear feet will not silently accept square feet —
// the build fails instead of the bid. That is the whole reason this file exists.
//
// The roofing version of the same idea: you do not order 435 of copper. You
// order 435 linear feet of copper, and the linear part is not decoration.

export type Unit = 'SF' | 'LF' | 'EA' | 'SQ';

export const UNITS: readonly Unit[] = ['SF', 'LF', 'EA', 'SQ'] as const;

/**
 * A quantity and the unit it is in. `Measure<'LF'>` and `Measure<'SF'>` are
 * different types: pass one where the other is wanted and it will not compile.
 */
export interface Measure<U extends Unit = Unit> {
  readonly value: number;
  readonly unit: U;
}

export const sf = (value: number): Measure<'SF'> => ({ value, unit: 'SF' });
export const lf = (value: number): Measure<'LF'> => ({ value, unit: 'LF' });
export const ea = (value: number): Measure<'EA'> => ({ value, unit: 'EA' });
export const sq = (value: number): Measure<'SQ'> => ({ value, unit: 'SQ' });

/** One square is one hundred square feet. The only conversion these four allow. */
export const SF_PER_SQ = 100;

export const sfToSq = (m: Measure<'SF'>): Measure<'SQ'> => sq(m.value / SF_PER_SQ);
export const sqToSf = (m: Measure<'SQ'>): Measure<'SF'> => sf(m.value * SF_PER_SQ);

/**
 * Everything else needs a property off the condition and cannot happen here.
 * Linear feet become square feet only when you say how tall the flashing runs;
 * a count comes from the trace. Those live with the condition, not in a units
 * file, because they are estimating decisions and not arithmetic identities.
 */

export function isUnit(v: unknown): v is Unit {
  return typeof v === 'string' && (UNITS as readonly string[]).includes(v);
}

/** Add two measures of the same unit. Different units will not compile. */
export function add<U extends Unit>(a: Measure<U>, b: Measure<U>): Measure<U> {
  return { value: a.value + b.value, unit: a.unit };
}

/** Scale a measure by a plain multiplier — a waste factor, a count of sides. */
export function scale<U extends Unit>(m: Measure<U>, by: number): Measure<U> {
  return { value: m.value * by, unit: m.unit };
}

export function formatMeasure(m: Measure): string {
  const n = m.unit === 'EA'
    ? String(Math.round(m.value))
    : m.value.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  return `${n} ${m.unit}`;
}
