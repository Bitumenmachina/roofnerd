// ── The solved model, as the windows read it ───────────────────────────────
// The mirror of `src-tauri/core/src/model.rs`. Same names, same shapes — the
// Rust serializes camelCase and this is what comes out the other end.
//
// It lives in the engine and not in the app because it is not the app's: the
// Plan, the Model, the Section and the export all read the one solve, and an
// app-side copy of a type is how a field added in one place silently does not
// exist in another (D108). Nothing in here has a window near it.
//
// WHAT IS IN HERE AND WHAT IS NOT. The solve is Rust's: which fields there are,
// where the fall runs to, what caps it, where the valleys and the crickets are.
// This file EVALUATES a solved field at a point — one expression, the one the
// core is pinned to — so the strip under the pointer can answer in the inches
// the roof actually is instead of in whatever the nearest grid sample happens to
// say. Sampling the grid would move every number in the readout, and the readout
// is the roof's own answer about itself.
//
// The grid is C0's. C1 ships planes instead and the grid goes; when it does,
// `readTaperAt` is the one thing that changes, and every surface that reads it
// moves together.

import type { CapReason, Spot } from './roof.js';
import { pointInPolygon } from './geometry.js';

/** The thickness of a field, sampled every `stepX` by `stepY` feet. */
export interface SolvedGrid {
  readonly minX: number;
  readonly minY: number;
  readonly stepX: number;
  readonly stepY: number;
  readonly cols: number;
  readonly rows: number;
  /** Row by row, `cols` to a row, in inches. */
  readonly thicknessInches: readonly number[];
  /** Whether each sample is on the roof at all. */
  readonly within: readonly boolean[];
  /** Whether the fall had already run out there. */
  readonly flat: readonly boolean[];
}

/** One arrow: where the water is, and which way it leaves. */
export interface SolvedFlowArrow {
  readonly x: number;
  readonly y: number;
  readonly thicknessInches: number;
  /** The angle about the vertical that turns plan +x onto the fall direction. */
  readonly angle: number;
}

/** One tapered field: one trace on one condition, solved. */
export interface SolvedTaperField {
  readonly conditionId: string;
  readonly trace: number;
  readonly ring: readonly Spot[];
  /** Top of structural deck, in feet. */
  readonly elevation: number;
  readonly startInches: number;
  readonly slopeInchesPerFoot: number;
  /**
   * Where the field stops rising, in inches. `null` is a field with nothing
   * capping it — not a zero, and not a ceiling somebody invented.
   */
  readonly capInches: number | null;
  readonly capReason: CapReason;
  /** The drainage points the fall runs to. */
  readonly apexes: readonly Spot[];
  readonly grid: SolvedGrid;
  readonly flow: readonly SolvedFlowArrow[];
  readonly noFallSquareFeet: number;
  readonly thinInches: number | null;
  readonly thickInches: number | null;
}

/** One sample along a valley, lifted onto the surface it lies in. */
export interface SolvedValleySample {
  readonly x: number;
  readonly y: number;
  readonly thicknessInches: number;
}

/** The low line between two drains. */
export interface SolvedValley {
  readonly elevation: number;
  readonly samples: readonly SolvedValleySample[];
}

/** A cricket, as triangles and as the facts the key has to say about it. */
export interface SolvedCricket {
  readonly conditionId: string;
  readonly trace: number;
  /** Triangles, as x, y, z in feet — y is up. */
  readonly positions: readonly number[];
  readonly lengthToWidth: number;
  readonly maxLengthToWidth: number;
  /** How far the traced ridge is off the one the roof implies, in degrees. */
  readonly offBy: number | null;
  readonly slopeDisagrees: boolean;
}

/** Everything the tapered package comes to. */
export interface SolvedTaper {
  readonly solveId: number;
  readonly drains: readonly Spot[];
  readonly fields: readonly SolvedTaperField[];
  readonly valleys: readonly SolvedValley[];
  readonly crickets: readonly SolvedCricket[];
}

/** A part of the model nobody has solved yet, said out loud rather than empty. */
export interface NotSolvedYet {
  readonly part: string;
  readonly solveId: number;
  readonly notSolvedYet: true;
}

/**
 * The numbers each condition carries, by name, for a formula to use.
 *
 * `null` is the whole point of the type: a name the solve did not produce reads
 * as pending on the line, never as zero.
 */
export type Scopes = Readonly<Record<string, Readonly<Record<string, number | null>>>>;

/** What the roof is doing at one place on it. */
export interface TaperReading {
  /** How far the water has to run to get off, in feet. */
  readonly runFeet: number;
  readonly thicknessInches: number;
  /** The fall has run out here: the roof is flat and the water stops moving. */
  readonly flat: boolean;
  /** Which drainage point it reaches. */
  readonly toward: Spot | null;
}

/**
 * Read a solved field at a point.
 *
 * The same expression the core samples its own grid with, evaluated where the
 * pointer is. It is not a second solve: which fields exist, what their apexes
 * are and where they stop rising are all decided in Rust and shipped — this
 * multiplies a distance by a slope. `src-tauri/core/src/taper.rs` (`read_at`)
 * is the other copy, and the two are pinned to the same three points.
 */
export function readTaperAt(field: SolvedTaperField, p: Spot): TaperReading {
  let runFeet = Infinity;
  let toward: Spot | null = null;
  for (const apex of field.apexes) {
    const dx = p.x - apex.x;
    const dy = p.y - apex.y;
    const r = Math.sqrt(dx * dx + dy * dy);
    if (r < runFeet) { runFeet = r; toward = apex; }
  }
  const raw = field.startInches + runFeet * field.slopeInchesPerFoot;
  const cap = field.capInches;
  return {
    runFeet,
    thicknessInches: cap === null ? raw : Math.min(raw, cap),
    flat: cap !== null && raw > cap,
    toward,
  };
}

/** Which solved field a point is on, or none — a point can be off the roof. */
export function fieldAt(
  taper: SolvedTaper, p: Spot,
): SolvedTaperField | null {
  for (const field of taper.fields) {
    if (pointInPolygon(p, field.ring)) return field;
  }
  return null;
}

/** The solved field for one trace on one condition, or none. */
export function fieldOf(
  taper: SolvedTaper, conditionId: string, trace: number,
): SolvedTaperField | null {
  return taper.fields.find((f) => f.conditionId === conditionId && f.trace === trace) ?? null;
}

/** The solved cricket for one trace on one condition, or none. */
export function cricketOf(
  taper: SolvedTaper, conditionId: string, trace: number,
): SolvedCricket | null {
  return taper.crickets.find((c) => c.conditionId === conditionId && c.trace === trace) ?? null;
}

/** Nothing solved yet: what a window holds before the first solve arrives. */
export const NOTHING_SOLVED: SolvedTaper = {
  solveId: 0, drains: [], fields: [], valleys: [], crickets: [],
};
