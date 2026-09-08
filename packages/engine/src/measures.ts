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
  /**
   * An arc is a run with no corners on it. Its points describe the curve for
   * drawing and for length, but none of them is a place where a piece of metal
   * gets mitred, so it contributes nothing to the corner count.
   */
  readonly arc?: boolean;
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
  /**
   * Thickness in inches. Insulation, in the unit it is sold in.
   *
   * On a tapered facet this is the thickness **at the drain** — the thin end,
   * where the taper starts. Thickness anywhere else on that facet is `T` plus
   * the distance from the drain times `TAPER`, which is why the two of them
   * together describe the whole board layout and neither does alone.
   */
  readonly T?: number;
  /** Rise per twelve inches of run. 5 means 5:12. */
  readonly PITCH?: number;
  /**
   * Taper slope, in inches of rise per foot of run — the way tapered insulation
   * is specified and sold: 1/8, 1/4, 1/2.
   *
   * This is not `PITCH`, and the difference is not pedantry. `PITCH` is the
   * deck's own slope in rise per twelve; `TAPER` is what the insulation adds on
   * top of it, per foot. A dead-flat deck with a quarter-inch taper has a
   * `PITCH` of nothing and a `TAPER` of 0.25, and a sloped deck can carry
   * tapered insulation as well. One name meaning both is how a wrong number
   * gets priced.
   */
  readonly TAPER?: number;
  /**
   * Elevation in feet. What it is measured to depends on what was traced, the
   * same way `SF` and `LF` already do:
   *
   * - on an **area**, the top of the deck — the reference the facet sits at;
   * - on a **line**, the base the run starts from.
   *
   * A roof is not flat in the sense that matters here: a parapet on a raised
   * section and a parapet on a low roof are the same trace with different
   * elevations, and nothing else on the condition can tell them apart.
   */
  readonly ELEV?: number;
  /**
   * Sump width in feet — the side of the square depression the drain sits in,
   * where the tapered field is cut down to meet the drain bowl.
   *
   * The trade sizes it off the insulation thickness at the drain: a shallow
   * field takes a four-foot sump, a thicker one takes eight. That rule belongs
   * to the layout, not to this type; what the condition carries is the width
   * that was used.
   */
  readonly SUMP?: number;
  /**
   * How many boards the taper runs over. Tapered insulation is sold as a
   * stack of 4-foot boards, so a layout is a whole number of them, and the
   * sump's depth falls out of the thickness at the drain, the slope, the sump
   * width and this count.
   */
  readonly BOARDS?: number;
  /** Number of sides. */
  readonly SIDES?: number;
  /** Stretch-out in inches: flat width of metal a formed profile eats. */
  readonly STRETCHOUT?: number;
  /** Anything else the estimator names, as a number. */
  readonly [name: string]: number | undefined;
}

/** The reserved property names. The last three only some details use. */
export const PROPERTY_NAMES = [
  'H', 'W', 'T', 'PITCH', 'TAPER', 'ELEV', 'SUMP', 'BOARDS', 'SIDES', 'STRETCHOUT',
] as const;

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
  /**
   * What gets counted. On a run or an area it is the corners; on a count
   * condition it is the objects. Never null — counting needs no scale.
   */
  readonly EA: number;
  /**
   * Corners: the points clicked along a run or around an area. An arc adds
   * none. This is what a mitre, a corner piece or a cleat is bought against.
   */
  readonly VERTICES: number;
  /** The straight pieces between those corners. A closed area has as many as it has corners. */
  readonly SEGMENTS: number;
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
  let objects = 0;
  let vertices = 0;
  let segments = 0;
  let unscaled = false;

  for (const trace of traces) {
    const points = trace.points;
    if (points.length === 0) continue;

    if (kind === 'count') {
      objects += points.length;
      continue;
    }

    // Corners, and the straight pieces between them. An arc has neither: it is
    // one continuous curve, and nothing on it gets mitred. A closed area has as
    // many segments as corners, because the last one closes back to the first.
    if (!trace.arc) {
      vertices += points.length;
      segments += kind === 'area' ? points.length : Math.max(0, points.length - 1);
    }

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

  // On a run or an area, EA is the corner count. That is what an Edge drawing
  // report shows: twelve rectangular curbs read 48 EA, and a radial counter
  // flashing — one arc — reads 0 EA against 61 LF of run.
  const ea = kind === 'count' ? objects : vertices;
  const counts = { EA: ea, VERTICES: vertices, SEGMENTS: segments };

  if (kind === 'count') {
    return { SF: null, PLAN_SF: null, SQ: null, LF: null, ...counts };
  }
  if (unscaled) {
    // One unscaled page makes the whole quantity unanswerable. It is not
    // partially right; it is pending, and it says so.
    return { SF: null, PLAN_SF: null, SQ: null, LF: null, ...counts };
  }
  if (kind === 'line') {
    return { SF: null, PLAN_SF: null, SQ: null, LF: lf, ...counts };
  }

  const sf = planSf * slope;
  return { SF: sf, PLAN_SF: planSf, SQ: sf / 100, LF: lf, ...counts };
}

/** The names a formula may use, and what each is worth right now. */
export function scopeFor(measures: Measures, properties: Properties): Record<string, number | null> {
  const scope: Record<string, number | null> = {
    SF: measures.SF,
    PLAN_SF: measures.PLAN_SF,
    SQ: measures.SQ,
    LF: measures.LF,
    EA: measures.EA,
    VERTICES: measures.VERTICES,
    SEGMENTS: measures.SEGMENTS,
  };
  for (const [name, value] of Object.entries(properties)) {
    if (value !== undefined) scope[name] = value;
  }
  return scope;
}
