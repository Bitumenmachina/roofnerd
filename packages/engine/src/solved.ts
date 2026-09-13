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

/** One arrow: where the water is, which way it leaves, and how far it runs. */
export interface SolvedFlowArrow {
  readonly x: number;
  readonly y: number;
  readonly thicknessInches: number;
  /** The angle about the vertical that turns plan +x onto the fall direction. */
  readonly angle: number;
  /**
   * How far the fall runs across the ground this arrow stands on, in feet.
   *
   * There is one arrow to a facet, so a mark that says "the water goes this way
   * over all of this" has to reach across the ground it speaks for. A field of
   * identical darts did not: every one of them was the same size wherever it
   * stood, and there were a hundred and thirty-nine of them.
   */
  readonly lengthFeet: number;
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

/**
 * What a derived name is called on screen, and what it is counted in.
 *
 * THE NAME IS CODE AND THE LABEL IS THE TRADE'S. `TAPER_ZZ_SF` is a formula
 * identifier — an estimator types it into a formula on purpose, and it belongs in
 * the mono face beside one, the same way `LF * H` does. It does not belong in a
 * sentence, a tooltip or a column of a panel, and the vocabulary check reads all
 * three. So every name the solve can produce has a trade label and a unit here,
 * once, and both the engine's own sentences and the app's panel read it.
 *
 * A name with no entry falls back to the words inside it, lower case — which
 * reads acceptably ("board waste") and, being lower case, cannot be mistaken for
 * the identifier by a reader or by the check.
 */
export interface DerivedLabel {
  readonly label: string;
  readonly unit: string;
  /** Where it sits in the panel's list: the order a bill is read in. */
  readonly order: number;
}

const DERIVED_LABELS: Readonly<Record<string, DerivedLabel>> = {
  // The boards first, because that is what the bill is.
  FILL_SF: { label: 'Flat board', unit: 'SF', order: 20 },
  FILL_BOARDS_EA: { label: 'Flat board', unit: 'EA', order: 21 },
  BOARD_WASTE_SF: { label: 'Board waste', unit: 'SF', order: 30 },
  BOARDS_MAX: { label: 'Courses needed', unit: 'EA', order: 31 },
  NOFALL_SF: { label: 'No fall', unit: 'SF', order: 40 },
  SUMP_PANELS_EA: { label: 'Sump panels', unit: 'EA', order: 50 },
  CRICKET_SF: { label: 'Cricket', unit: 'SF', order: 60 },
  // The walls and curbs, C3.
  FLASH_SF: { label: 'Wall flashing', unit: 'SF', order: 70 },
  FLASH_MIN_IN: { label: 'Flashing height, least', unit: 'IN', order: 71 },
  CURB_RAISE_IN: { label: 'Raise needed', unit: 'IN', order: 72 },
  COPING_LF: { label: 'Coping', unit: 'LF', order: 80 },
  COPING_PCS_EA: { label: 'Coping pieces', unit: 'EA', order: 81 },
  CLEAT_LF: { label: 'Cleat', unit: 'LF', order: 82 },
};

/** The lettered panels, which are one name per letter and cannot be listed. */
const TAPERED_PANEL = /^TAPER_([A-Z]+[0-9]*)_SF$/;

/** What to call a derived name on screen, and what it is counted in. */
export function derivedLabel(name: string): DerivedLabel {
  const known = DERIVED_LABELS[name];
  if (known) return known;
  const panel = TAPERED_PANEL.exec(name);
  // In the chart's own order, so the panel reads X, Y, Z, ZZ rather than
  // alphabetically — and a letter the table adds later still lands among them.
  if (panel) return { label: `Tapered panel ${panel[1]}`, unit: 'SF', order: 10 };
  return {
    label: name.replace(/_(SF|LF|EA|SQ)$/, '').toLowerCase().replace(/_/g, ' '),
    unit: /_(SF|LF|EA|SQ)$/.exec(name)?.[1] ?? '',
    order: 90,
  };
}

/**
 * The names one condition carries, in the order a bill is read in.
 *
 * Stated rather than alphabetical: the lettered panels, then the flat board under
 * them, then what the cuts took, then the courses, then the ground with no fall,
 * then the sumps and the cricket. Within a group, by name, so two runs of the
 * same job list them the same way.
 */
export function derivedOrder(names: readonly string[]): string[] {
  return [...names].sort((a, b) => {
    const by = derivedLabel(a).order - derivedLabel(b).order;
    return by !== 0 ? by : a.localeCompare(b);
  });
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

// ── the walls and curbs ─────────────────────────────────────────────────────
// The mirror of `src-tauri/core/src/walls.rs` and `coping.rs` (C3). A run's
// solid (its outer line, and the inboard line `WALL` grows toward — offset by
// a true polyline offset, not a per-segment guess at where the ring's middle
// is) and how far the membrane is allowed to climb it, which is a function of
// position along the run and not a single typed number.

/** A coping cap, as far as the bill and the section need it. */
export interface SolvedCoping {
  readonly capWidthFeet: number | null;
  readonly capWidthStated: boolean;
  readonly girthInches: number | null;
  /** What the girth leaves over the cap, each side — what a gauge reads off. */
  readonly faceHeightInches: number | null;
  /** 24 or 22. `null` past 10 in, which Fig. 2-3 has no row for. */
  readonly gaugeGa: number | null;
  /**
   * The cap's own two edges, centred on the wall. `null` wherever
   * `capWidthFeet` or the wall's own thickness is — a cap centred on a wall
   * with no stated thickness has nothing to centre on.
   */
  readonly capOuter: SolvedShape | null;
  readonly capInner: SolvedShape | null;
  readonly sectionFeet: number;
  readonly sectionStated: boolean;
  readonly sectionOutOfRange: boolean;
  readonly cornersOutside: number;
  readonly cornersInside: number;
  readonly runFeet: number;
  readonly piecesEa: number;
  readonly cleatFeet: number;
  readonly fastenOcInches: number;
  readonly fastenOcStated: boolean;
}

/** One wall or curb run, solved. */
export interface SolvedWallRun {
  readonly conditionId: string;
  readonly trace: number;
  /** `curb`, or absent — a run with no stated role is a parapet. */
  readonly role: string | null;
  readonly name: string;
  readonly outer: SolvedShape;
  /** The outer line offset inboard by `WALL`. Absent where nobody typed one. */
  readonly inboard: SolvedShape | null;
  readonly elevationFeet: number;
  readonly heightFeet: number;
  readonly wallInches: number | null;
  /** How high the membrane is detailed to run, in inches: typed, or the
   * wall's own height where nobody said. */
  readonly flashupInches: number;
  readonly flashupStated: boolean;
  /** The least the flashing ceiling comes to anywhere on the run. Absent
   * only where the roof beneath could not be read at all. */
  readonly flashMinInches: number | null;
  /** The exact area under the flashing ceiling along the whole run. */
  readonly flashSquareFeet: number | null;
  /** `max(0, MIN_FLASHING_INCHES - flashMinInches)`. */
  readonly curbRaiseInches: number | null;
  /** Why the flashing ceiling could not be read everywhere, where it could not. */
  readonly refused: string | null;
  readonly coping: SolvedCoping;
}

export interface SolvedWalls {
  readonly solveId: number;
  readonly runs: readonly SolvedWallRun[];
}

/** Nothing solved yet: what a window holds before the first solve arrives. */
export const NOTHING_SOLVED_WALLS: SolvedWalls = { solveId: 0, runs: [] };

/** The wall or curb run for one trace on one condition, or none. */
export function wallOf(
  walls: SolvedWalls, conditionId: string, trace: number,
): SolvedWallRun | null {
  return walls.runs.find((r) => r.conditionId === conditionId && r.trace === trace) ?? null;
}

// ── the drawings ───────────────────────────────────────────────────────────
// The mirror of `src-tauri/core/src/drawing.rs`. One drawing model, three
// drawings, and ONE renderer on the front end — the Plan's two overlays and the
// Section view are the same code pointed at different drawings (D155).
//
// Everything in here is in FEET. A plan drawing is in the roof's own plan
// coordinates, so the Plan lays it straight over the sheet it was traced off; a
// section is in its own frame, x from the far end of the cut to the drain and y
// the height off the top of deck with UP NEGATIVE — the same direction plan y
// goes, so one renderer draws both with no special case in it.
//
// Nothing here computes. A role is a word the stylesheet knows, a dimension
// arrives with its measure already written the way a roofer writes it, and the
// vertical exaggeration is stated so the renderer can stretch the picture
// without any number on it moving.

/** A filled shape on a drawing. */
export interface DrawnShape {
  readonly shape: SolvedShape;
  /** What it is, in one word: `board`, `sump`, `nofall`, `fill`, `panel`, … */
  readonly role: string;
  /** The letter printed on the panel, where it carries one. */
  readonly letter: string | null;
  /** Which course of the build-up this is. Null on a plan, which has no layers. */
  readonly layer: string | null;
}

/** A line, open or closed. */
export interface DrawnLine {
  readonly points: readonly Spot[];
  readonly role: string;
  readonly dashed: boolean;
  readonly closed: boolean;
  /** Its place in the build-up, where it has one — the deck is a line. */
  readonly layer: string | null;
}

/** A word on the drawing, at a place on it. */
export interface DrawnText {
  readonly at: Spot;
  readonly text: string;
  /** `letter`, `label`, `thickness` or `key`. */
  readonly role: string;
  readonly anchor: 'start' | 'middle' | 'end';
  readonly leader: Spot | null;
}

/** A measure between two places, written the way a roofer writes it. */
export interface Dimension {
  readonly a: Spot;
  readonly b: Spot;
  /** How far off the two points the line sits, square to them, in feet. */
  readonly offset: number;
  readonly text: string;
  /** True where the number came out of the solve rather than off the job. */
  readonly derived: boolean;
}

/** The box a drawing needs, in its own feet. */
export interface Extent {
  readonly minX: number;
  readonly minY: number;
  readonly maxX: number;
  readonly maxY: number;
}

/** One drawing: everything on it, in feet. */
export interface Drawing {
  readonly id: string;
  readonly title: string;
  /** What it is OF: which cut, through which drain, to which wall. */
  readonly subtitle: string | null;
  /** What it is drawn at, and how far the vertical is stretched, in words. */
  readonly note: string;
  readonly feetPerInch: number;
  readonly verticalExaggeration: number;
  /** Nothing to draw, and why — never an empty picture with no word on it. */
  readonly nothingYet: string | null;
  readonly shapes: readonly DrawnShape[];
  readonly lines: readonly DrawnLine[];
  readonly texts: readonly DrawnText[];
  readonly dimensions: readonly Dimension[];
  /** The layer key, bottom to top. */
  readonly key: readonly string[];
  readonly keyAt: Spot | null;
  readonly keyFrom: Spot | null;
  readonly extent: Extent | null;
  /** Every condition the drawing goes through, so the Estimate can name them. */
  readonly through: readonly string[];
  /**
   * Where the cut is on the PLAN, for a view that stands the drawing up again:
   * the drain it runs out of, the way out of it, how far it runs, and the datum
   * it is measured from. Null on a plan drawing, which is already where the roof
   * is.
   */
  readonly cutAt: Spot | null;
  readonly cutAlong: Spot | null;
  readonly cutRunFeet: number;
  readonly cutElevation: number;
}

/** The three drawings the 2D outputs are made of. */
export interface Drawings {
  readonly solveId: number;
  readonly layout: Drawing;
  readonly fastening: Drawing;
  /** One section per drain on the roof, cut along the boards. */
  readonly sections: readonly Drawing[];
}

/** An empty drawing, for a window that has not pulled one yet. */
const NOTHING_DRAWN = (id: string, title: string): Drawing => ({
  id,
  title,
  subtitle: null,
  note: '',
  feetPerInch: 8,
  verticalExaggeration: 1,
  nothingYet: 'Nothing is open, so there is nothing to draw.',
  shapes: [],
  lines: [],
  texts: [],
  dimensions: [],
  key: [],
  keyAt: null,
  keyFrom: null,
  extent: null,
  through: [],
  cutAt: null,
  cutAlong: null,
  cutRunFeet: 0,
  cutElevation: 0,
});

/** What a window holds before the first drawing is pulled. */
export const NOTHING_TO_DRAW: Drawings = {
  solveId: 0,
  layout: NOTHING_DRAWN('layout', 'Tapered layout'),
  fastening: NOTHING_DRAWN('fastening', 'Fastening plan'),
  sections: [],
};
