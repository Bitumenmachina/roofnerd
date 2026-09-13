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
// how they are cut into drainage regions, which planes lay each one out, what caps
// them, and where the valleys, the ridges and the crickets are. This file
// EVALUATES a shipped plane at a point — one expression, the one the core is
// pinned to — so the strip under the pointer answers in the inches the roof
// actually is. Nothing in here decides what the roof is.
//
// C1 (D149) retired the grid. The field arrives as drainage regions, each with the
// four planes that lay it out, the facets those planes govern, the creases they
// fold along and the plateau where the cap binds. `readTaperAt` evaluates a plane
// where the pointer is; `readFacetAt` does the same at a facet's own corner, where
// no search can find the region because the corner is on its boundary.

import type { CapReason, Spot } from './roof.js';
import { pointInPolygon } from './geometry.js';

/** A polygon in feet: an outline and whatever holes it has. */
export interface SolvedShape {
  readonly ring: readonly Spot[];
  readonly holes: readonly (readonly Spot[])[];
}

/**
 * A sloping plane on the roof.
 *
 * `origin` is a point it passes through at its own low edge — the drain — and
 * `direction` is the unit vector the fall rises along in plan, with `slope` in
 * inches per foot. Four of these come off every drainage region, and the
 * thickness is the greatest rise of the four.
 */
export interface SolvedPlane {
  readonly origin: Spot;
  readonly direction: Spot;
  readonly slope: number;
}

/**
 * A piece of ground one plane governs — or the cap does, and the roof is flat.
 *
 * Either way the surface over it is exactly planar, which is the whole point: a
 * facet can be triangulated and each corner lifted to its own depth with no fold
 * left inside it, so the mesh is a few dozen triangles instead of a lattice.
 */
export interface SolvedFacet {
  readonly shape: SolvedShape;
  /** Which of the region's planes governs here. */
  readonly plane: number;
  /** True where the cap binds: the roof is flat, and the water stops moving. */
  readonly flat: boolean;
}

/** A line the boards are cut along, with the depth at each end. */
export interface SolvedCrease {
  readonly a: Spot;
  readonly b: Spot;
  readonly aInches: number;
  readonly bInches: number;
}

/** The boundary two regions share, and whether the two patterns agree on it. */
export interface SolvedRidge {
  readonly a: Spot;
  readonly b: Spot;
  readonly aInches: number;
  readonly bInches: number;
  /** Which region is on the other side, by its place in the field's own list. */
  readonly meets: number;
  /**
   * The worst gap between the depth this region's planes make here and the depth
   * the neighbour's make. NRCA wants one common thickness at a ridge and offers an
   * area divider where the patterns cannot give one, so it is carried rather than
   * averaged away.
   */
  readonly disagreementInches: number;
}

/**
 * One board position: a 4 ft panel, where it lies, how thick and which letter.
 *
 * A cut board is still one board, because one is what gets bought. `shapes` is what
 * is left of it after the region's edges, the roof's outline and the sumps have had
 * their say, and `wasteSquareFeet` is what the cut took off — which is NOT the
 * item's waste allowance on an estimate line. That is for damage and trim; this is
 * board bought and thrown away at a known place, and adding the two charges the cut
 * twice.
 */
export interface SolvedBoard {
  readonly centre: Spot;
  readonly shapes: readonly SolvedShape[];
  /** Square feet of it that land on the roof. */
  readonly squareFeet: number;
  /** Square feet of it that got bought: a whole board, cut or not. */
  readonly boardSquareFeet: number;
  readonly cut: boolean;
  readonly wasteSquareFeet: number;
  /** The roof's depth at the board's middle, in inches, off the solved plane. */
  readonly thicknessInches: number;
  readonly flat: boolean;
  /** The letter printed on the tapered panel, or null where the table has none. */
  readonly letter: string | null;
  readonly panelInches: number;
  readonly fillInches: number;
  readonly fillBoards: number;
  /** Layers of board here: the fill boards and the tapered one. */
  readonly courses: number;
}

/** How many boards of one letter, and how many square feet that is. */
export interface SolvedLetterRow {
  readonly boards: number;
  readonly squareFeet: number;
}

/**
 * What a run of boards comes to, in the words a supply house quotes.
 *
 * Nought against a letter means the layout uses none of that panel, which is a
 * fact about the layout. A name nobody solved is ABSENT and reads as pending on a
 * line. The two are different and they stay different.
 */
export interface SolvedBoardBill {
  readonly manufacturer: string;
  readonly product: string;
  readonly documentId: string;
  /** Which of the two printed charts: the 4 in fill or the 2 in one. */
  readonly profileSet: 'standard' | 'extended';
  readonly slopeInchesPerFoot: number;
  readonly fillStepInches: number;
  readonly panelFeet: number;
  /** How far each course under the top one is offset, in feet. */
  readonly staggerFeet: number;
  readonly byLetter: Readonly<Record<string, SolvedLetterRow>>;
  readonly boards: number;
  readonly cutBoards: number;
  /** Square feet of tapered panel bought: a whole board apiece. */
  readonly squareFeet: number;
  readonly wasteSquareFeet: number;
  /** What lands on the roof. */
  readonly laidSquareFeet: number;
  readonly fillBoards: number;
  readonly fillSquareFeet: number;
  /**
   * The most courses at any one position, the tapered panel included. What the
   * layout actually needs — which is not what `BOARDS` was typed as unless the
   * board ceiling is what bound.
   */
  readonly coursesMax: number;
  /** Positions the table has no letter for, counted rather than hidden. */
  readonly unlettered: number;
}

/**
 * The square the field is cut down in to meet a drain bowl.
 *
 * `rimInches` is read off the solved planes at the sump's own edge, so the drawing
 * places it instead of working it out again. On a four-way layout it is the same
 * number all the way round: the thickness is the greater of two square runs, so a
 * square standing square to the boards is a contour of it.
 */
export interface SolvedSump {
  readonly conditionId: string;
  readonly trace: number;
  readonly at: Spot;
  readonly sideFeet: number;
  /** The side the bowl-plus-24-inches rule wants, where a bowl was stated. */
  readonly fromBowlFeet: number | null;
  readonly shapes: readonly SolvedShape[];
  readonly squareFeet: number;
  /** True where the roof's own edge cut the square short. */
  readonly clipped: boolean;
  /** The roof's depth at the sump's rim, in inches: where the cut is. */
  readonly rimInches: number;
  /** How far the sump falls from that rim to the drain, in inches. */
  readonly depthInches: number;
  readonly panels: number;
  readonly panelFeet: number;
}

/** One drainage region: the ground one drain takes, and the fall over it. */
export interface SolvedTaperRegion {
  readonly drain: Spot;
  /** The ground, in pieces — a concave field can hand one drain two of them. */
  readonly shapes: readonly SolvedShape[];
  readonly squareFeet: number;
  readonly axisAlong: Spot;
  readonly axisAcross: Spot;
  readonly planes: readonly SolvedPlane[];
  readonly capInches: number | null;
  readonly facets: readonly SolvedFacet[];
  /** The 45 degree ties, cut to this region. */
  readonly valleys: readonly SolvedCrease[];
  readonly ridges: readonly SolvedRidge[];
  readonly noFallSquareFeet: number;
  readonly thinInches: number;
  readonly thickInches: number;
  /** The board grid over this region's own ground, one entry per position. */
  readonly boards: readonly SolvedBoard[];
  /** What those boards come to, or null where the table charts no such slope. */
  readonly bill: SolvedBoardBill | null;
}

/** One arrow: where the water is, and which way it leaves. */
export interface SolvedFlowArrow {
  readonly x: number;
  readonly y: number;
  readonly thicknessInches: number;
  /** The angle about the vertical that turns plan +x onto the fall direction. */
  readonly angle: number;
}

/** One tapered field: one trace on one condition, laid out. */
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
  /** The axes the layout is laid out on: the field's longest side, and square to it. */
  readonly axisAlong: Spot;
  readonly axisAcross: Spot;
  /** That side itself, so a drawing can say which wall the boards were laid to. */
  readonly axisEdge: readonly Spot[];
  /** One region per drainage point on this field. */
  readonly regions: readonly SolvedTaperRegion[];
  readonly flow: readonly SolvedFlowArrow[];
  readonly squareFeet: number;
  readonly noFallSquareFeet: number;
  readonly thinInches: number | null;
  readonly thickInches: number | null;
  /** Four ways off each drain, or two. Typed as `LAYOUT`. */
  readonly layout: string;
  /** A typed `LAYOUT` that named neither, so the default was used. */
  readonly layoutRefused: number | null;
  /** A typed `PROFILE_SET` that named neither chart, so the default was used. */
  readonly profileSetRefused: number | null;
  /** The whole field's bill: its regions' added up. */
  readonly bill: SolvedBoardBill | null;
  /** Why there is no bill, where there is none. */
  readonly billRefused: string | null;
  /** The sumps cut into it, which the boards are laid around. */
  readonly sumps: readonly SolvedSump[];
}

/** A trace the layout would not lay out, and why. */
export interface SolvedRefusal {
  readonly conditionId: string;
  readonly trace: number;
  readonly why: string;
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
  /**
   * Its footprint on the plan, in square feet. Cricket stock is its own lettered
   * series and the chart in `data/profiles` carries only the primary slope, so a
   * cricket is billed by area and not by letter.
   */
  readonly planSquareFeet: number;
  readonly slopeInchesPerFoot: number;
  /** True where the solve put it there: the saddle a two-way layout needs. */
  readonly derived: boolean;
  /** True where every vertex was lifted by the roof's own depth under it. */
  readonly fromSolvedSurface: boolean;
}

/** Everything the tapered package comes to. */
export interface SolvedTaper {
  readonly solveId: number;
  readonly drains: readonly Spot[];
  readonly fields: readonly SolvedTaperField[];
  /** Every sump on the roof, in the order the drains are listed. */
  readonly sumps: readonly SolvedSump[];
  readonly crickets: readonly SolvedCricket[];
  /** What the layout would not lay out, and why. */
  readonly refused: readonly SolvedRefusal[];
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
  /**
   * How far the fall has run to get here, in feet, along the boards.
   *
   * This is the run a tapered schedule means: thickness = T + TAPER x run. It is
   * not the straight line to the drain, and on a four-way layout the two differ
   * by as much as a diagonal does.
   */
  readonly runFeet: number;
  readonly thicknessInches: number;
  /** The fall has run out here: the roof is flat and the water stops moving. */
  readonly flat: boolean;
  /** Which drainage point it falls to. */
  readonly toward: Spot;
  /** Which of the region's planes governs. Two tie along a valley. */
  readonly plane: number;
}

/** How far above its own low edge a plane has risen at a point, in inches. */
function riseAt(plane: SolvedPlane, p: Spot): number {
  return ((p.x - plane.origin.x) * plane.direction.x
    + (p.y - plane.origin.y) * plane.direction.y) * plane.slope;
}

/** Is this point inside this ring? Ray casting, the engine's own. */
const within = (p: Spot, shape: SolvedShape): boolean =>
  pointInPolygon(p, shape.ring) && !shape.holes.some((hole) => pointInPolygon(p, hole));

/** Which region a place is on, or none — a place can be off the field. */
export function regionAt(field: SolvedTaperField, p: Spot): SolvedTaperRegion | null {
  return field.regions.find((region) => region.shapes.some((shape) => within(p, shape))) ?? null;
}

/**
 * Read a solved field at a point.
 *
 * The greatest of the region's four planes, capped — one expression, and the same
 * one `read_at` in `src-tauri/core/src/taper.rs` evaluates, pinned to the same
 * four places on both sides. It is not a second solve: which regions exist, which
 * planes are on them, where they are cut and where they stop rising are all
 * decided in Rust and shipped. This multiplies a run by a slope.
 *
 * `null` where no region holds the place, which is a place off the field — never a
 * roof of no thickness.
 */
export function readTaperAt(field: SolvedTaperField, p: Spot): TaperReading | null {
  const region = regionAt(field, p);
  if (!region) return null;
  return readRegionAt(field, region, p);
}

/** The same reading, where the region is already known. */
export function readRegionAt(
  field: SolvedTaperField, region: SolvedTaperRegion, p: Spot,
): TaperReading | null {
  let plane = -1;
  let most = -Infinity;
  region.planes.forEach((each, i) => {
    const rise = riseAt(each, p);
    if (rise > most) { most = rise; plane = i; }
  });
  if (plane < 0) return null;
  const slope = region.planes[plane]!.slope;
  const raw = field.startInches + most;
  const cap = region.capInches;
  return {
    runFeet: Math.abs(slope) < 1e-12 ? 0 : most / slope,
    thicknessInches: cap === null ? raw : Math.min(raw, cap),
    flat: cap !== null && raw > cap,
    toward: region.drain,
    plane,
  };
}

/**
 * How deep the roof is at a corner of one facet, in inches.
 *
 * The facet says which plane governs it, so this needs no search — which matters
 * because a facet's corners lie ON the boundaries of the region that holds it,
 * and a point on a polygon's own edge is neither in nor out of it. It is the
 * same expression as above with the search taken out.
 */
export function readFacetAt(
  field: SolvedTaperField, region: SolvedTaperRegion, facet: SolvedFacet, p: Spot,
): number {
  const cap = region.capInches;
  if (facet.flat && cap !== null) return cap;
  const plane = region.planes[facet.plane];
  if (!plane) return field.startInches;
  const raw = field.startInches + riseAt(plane, p);
  return cap === null ? raw : Math.min(raw, cap);
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
  solveId: 0, drains: [], fields: [], sumps: [], crickets: [], refused: [],
};

/** The sump at one drainage point on one condition, or none. */
export function sumpOf(taper: SolvedTaper, conditionId: string, at: Spot): SolvedSump | null {
  return taper.sumps.find(
    (s) => s.conditionId === conditionId && s.at.x === at.x && s.at.y === at.y,
  ) ?? null;
}
