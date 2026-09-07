// ── What a trace measures ──────────────────────────────────────────────────
// One traced thing yields square feet, linear feet and a count at the same
// time, because a parapet run is all three and pretending otherwise is what
// makes estimating software annoying. This file is that rule.
//
// The names here are the ones an estimator can type into a formula on an item
// line. They are fixed, and they are all any formula ever sees of the drawing.

import type { Calibration, Point } from './geometry.js';
import {
  pitchFactor, polygonArea, polygonPerimeter, polylineLength,
} from './geometry.js';

/** What a condition is: an area on the sheet, a run along it, or things counted. */
export type TraceKind = 'area' | 'line' | 'count';

/** One shape drawn on one page. A condition may have many, on many pages. */
export interface Trace {
  readonly id: string;
  readonly pageId: string;
  readonly points: readonly Point[];
}

/**
 * Numbers an estimator puts on a condition that the drawing cannot tell them:
 * how tall the flashing runs, how wide the coping is, what the slope is.
 * All optional — a condition with none of them still measures.
 */
export interface Properties {
  /** Height in feet. Wall flashing height, parapet height. */
  readonly H?: number;
  /** Width in feet. */
  readonly W?: number;
  /** Thickness in inches. Insulation, in the unit it is sold in. */
  readonly T?: number;
  /** Rise per twelve inches of run. 5 means 5:12. */
  readonly PITCH?: number;
  /** Number of sides. */
  readonly SIDES?: number;
  /** Stretch-out in inches: flat width of metal a formed profile eats. */
  readonly STRETCHOUT?: number;
  /** Anything else the estimator names, as a number. */
  readonly [name: string]: number | undefined;
}

/** The four reserved property names plus the two that only some details use. */
export const PROPERTY_NAMES = ['H', 'W', 'T', 'PITCH', 'SIDES', 'STRETCHOUT'] as const;

/**
 * Everything a formula can see. `null` where a page has not been scaled yet:
 * an unscaled trace has no length, and saying so is the whole point — a zero
 * would quietly become a zero-dollar line.
 */
export interface Measures {
  /** Traced surface, sloped. Plan area times the pitch factor. Areas only. */
  readonly SF: number | null;
  /** Traced surface as it sits on the drawing, flat. Areas only. */
  readonly PLAN_SF: number | null;
  /** SF in squares — a hundred square feet. */
  readonly SQ: number | null;
  /** Run. An area's perimeter, a line's length. */
  readonly LF: number | null;
  /** Corners for an area or a run; markers for a count. Never null: counting needs no scale. */
  readonly EA: number;
}

/** A page that has been scaled, or has not. */
export type Calibrations = Readonly<Record<string, Calibration | undefined>>;

/**
 * Measure a condition's traces.
 *
 * SF is the traced surface, so it is a number only for an area. A run becomes
 * an area through a formula that says how — wall flashing is `LF * H`, and
 * writing it that way keeps the height visible on the line instead of buried
 * in this function.
 */
export function measure(
  kind: TraceKind,
  traces: readonly Trace[],
  properties: Properties,
  calibrations: Calibrations,
): Measures {
  const slope = properties.PITCH === undefined ? 1 : pitchFactor(properties.PITCH);

  let planSf = 0;
  let lf = 0;
  let ea = 0;
  let unscaled = false;

  for (const trace of traces) {
    const points = trace.points;
    if (points.length === 0) continue;

    if (kind === 'count') {
      ea += points.length;
      continue;
    }

    // Corners. A polygon's last vertex closes back to its first, so the vertex
    // count is the corner count either way.
    ea += points.length;

    const calibration = calibrations[trace.pageId];
    if (!calibration) {
      unscaled = true;
      continue;
    }
    const feetPerUnit = calibration.feetPerUnit;

    if (kind === 'area') {
      planSf += polygonArea(points) * feetPerUnit * feetPerUnit;
      lf += polygonPerimeter(points) * feetPerUnit;
    } else {
      lf += polylineLength(points) * feetPerUnit;
    }
  }

  if (kind === 'count') {
    return { SF: null, PLAN_SF: null, SQ: null, LF: null, EA: ea };
  }
  if (unscaled) {
    // One unscaled page makes the whole quantity unanswerable. It is not
    // partially right; it is pending, and it says so.
    return { SF: null, PLAN_SF: null, SQ: null, LF: null, EA: ea };
  }
  if (kind === 'line') {
    return { SF: null, PLAN_SF: null, SQ: null, LF: lf, EA: ea };
  }

  const sf = planSf * slope;
  return { SF: sf, PLAN_SF: planSf, SQ: sf / 100, LF: lf, EA: ea };
}

/** The names a formula may use, and what each is worth right now. */
export function scopeFor(measures: Measures, properties: Properties): Record<string, number | null> {
  const scope: Record<string, number | null> = {
    SF: measures.SF,
    PLAN_SF: measures.PLAN_SF,
    SQ: measures.SQ,
    LF: measures.LF,
    EA: measures.EA,
  };
  for (const [name, value] of Object.entries(properties)) {
    if (value !== undefined) scope[name] = value;
  }
  return scope;
}
