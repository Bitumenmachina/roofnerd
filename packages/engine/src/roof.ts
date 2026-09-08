// ── How a roof is actually built ───────────────────────────────────────────
// The conventions the trade already has, with the source on each one.
//
// This file exists because the Model was built the other way round: a shape was
// invented that looked like a roof, and then a check was written that agreed
// with it. Every number here came out of a document or a tool that can be
// opened on this machine, and each one carries where it came from, so the next
// seat can disagree with the source rather than with me.
//
// It is pure arithmetic. Nothing in here knows there is a window.

/** A point in feet on the roof plan. */
export interface Spot {
  readonly x: number;
  readonly y: number;
}

// ── Slope ──────────────────────────────────────────────────────────────────

/**
 * A cricket is cut at twice the slope of the field beside it.
 *
 * NRCA, *The NRCA Roofing Manual: Membrane Roof Systems — 2019*, p.79 and again
 * verbatim at p.165: "a general rule of thumb for designing sufficiently sloping
 * saddles and crickets is that they be twice the slope of the adjacent field of
 * the roof." Not an NRCA invention — it traces to PIMA Technical Bulletin #108,
 * and Hunter Panels, Garland and Construction Specifier all repeat it.
 *
 * It is also why cricket stock is a different purchase: Carlisle's InsulBase
 * tapered data sheet prints two panel-profile charts side by side, headed
 * "Cricket" and "Primary Slope", with their own lettered series. A cricket is
 * not the field pattern extruded at some width; it is a different box of boards.
 */
export const CRICKET_SLOPE_MULTIPLE = 2;

/**
 * The most a cricket may be long against how wide it is.
 *
 * NRCA Fig. 4-13 (p.166, reprinted at Figs. 10-13/10-14, pp.309–310):
 *
 *     field slope   cricket stock   max L:W
 *     1/8 : 12      1/4 : 12        3 : 1
 *     1/4 : 12      1/2 : 12        3 : 1
 *     1/2 : 12      1/2 : 12        4 : 1
 *
 * The ratio is not decoration. Doubling the *surface* slope does nothing for the
 * *valley* slope — the speed water runs along the cricket's edge toward the
 * drain — and width is what moves that. A cricket that is too long for its width
 * looks right in a picture and ponds at the valley, which is the exact failure
 * this whole pass is about.
 *
 * @param fieldSlope The field's own taper, in inches per foot.
 */
export function maxLengthToWidth(fieldSlope: number): number {
  return fieldSlope >= 0.5 ? 4 : 3;
}

/**
 * Where the ridge of a cricket between two drains lies.
 *
 * NRCA p.166–168, "Ridges": "To achieve simple, consistent ridge thicknesses,
 * the ridge line should be spaced equidistant between drainage points. The ridge
 * line will be perpendicular to a line that connects the two drainage points."
 *
 * That is the perpendicular bisector, and it means the ridge is **derived**. It
 * is not a line somebody draws and hangs two planes off; it is where two
 * drainage fields happen to meet. The Model had that backwards, and a traced
 * ridge is worth keeping only for how far it runs — see `ridgeDisagreement`.
 *
 * Returns the midpoint, the unit direction the ridge runs in, and how far each
 * drain sits from it — which is the fall run, and therefore the cricket's width.
 */
export function ridgeBetween(a: Spot, b: Spot): {
  readonly at: Spot;
  readonly along: Spot;
  readonly width: number;
} | null {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const span = Math.hypot(dx, dy);
  // Two drains in the same place have no line between them to bisect.
  if (span < 1e-6) return null;
  return {
    at: { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 },
    // Perpendicular to a→b, which is what "perpendicular to a line that connects
    // the two drainage points" says.
    along: { x: -dy / span, y: dx / span },
    // The ridge sits halfway, so water falls half the span to reach either drain.
    width: span / 2,
  };
}

/**
 * How far a traced ridge is off the one the drains imply, in degrees.
 *
 * The estimator's trace is still worth something — it says how far the cricket
 * runs, which nothing in the drain geometry knows. But if the line they drew is
 * square to the drains and the derived one is not, one of the two is wrong about
 * which drains this cricket serves, and that is worth saying out loud rather
 * than silently drawing whichever one the code happened to prefer.
 */
export function ridgeDisagreement(traced: readonly Spot[], along: Spot): number | null {
  if (traced.length < 2) return null;
  const a = traced[0]!;
  const b = traced[traced.length - 1]!;
  const len = Math.hypot(b.x - a.x, b.y - a.y);
  if (len < 1e-6) return null;
  const dot = ((b.x - a.x) / len) * along.x + ((b.y - a.y) / len) * along.y;
  // A ridge has no direction — running it backwards is the same ridge — so the
  // angle folds at 90°.
  const deg = (Math.acos(Math.min(1, Math.abs(dot))) * 180) / Math.PI;
  return deg;
}

// ── What stops a taper rising ──────────────────────────────────────────────

/**
 * The most one layer of tapered polyiso is made in, in inches.
 *
 * Carlisle InsulBase Tapered publishes "0.5 in minimum to 4.5 in maximum in a
 * single layer"; GAF EnergyGuard reaches 4.6. The lower of the two is the safe
 * one to draw against. This is a manufacturing limit — foam rising between two
 * facers on a lamination line — and **it does not stop the roof rising**. Past
 * it you buy a second layer, which is why this is reported and not treated as
 * a ceiling.
 *
 * The Model used to cap the field at 4 in per board and call the flat part
 * beyond it "no fall". Four was not a number from anywhere, and the cap was the
 * wrong kind of thing.
 */
export const SINGLE_LAYER_MAX_INCHES = 4.5;

/**
 * Flashing has to reach 8 in above the finished roof, so the roof cannot rise
 * past 8 in below whatever it flashes against.
 *
 * NRCA p.159 and pp.251–252: 8 in minimum flashing height above the finished
 * membrane at curbs, walls, skylights and hatches. Parapet and curb heights are
 * fixed by the building, so this — not the board thickness — is the ceiling an
 * estimator actually hits. It is the difference between "buy another layer" and
 * "you cannot raise this roof without raising that wall", and those are not the
 * same problem.
 */
export const MIN_FLASHING_INCHES = 8;

/** Why a tapered field stopped rising. */
export type CapReason = 'flashing' | 'boards' | null;

/**
 * What caps the build-up, and which constraint bound.
 *
 * Both are real and they fail differently, so the caller is told which one it
 * was. Nothing is silently zero here either: with neither a board count nor a
 * traced perimeter, there is no cap and the field rises unbounded, which is the
 * honest answer rather than a made-up ceiling.
 *
 * @param startInches   Thickness at the drain — the low point the code sets.
 * @param boards        How many layers of tapered board, when the estimator said.
 * @param perimeterInches The lowest thing the roof has to flash against, in
 *                        inches above the deck. A parapet height, usually.
 */
export function capInches(
  startInches: number,
  boards: number | undefined,
  perimeterInches: number | undefined,
): { readonly cap: number; readonly reason: CapReason } {
  const fromBoards = boards === undefined
    ? Infinity
    : startInches + boards * SINGLE_LAYER_MAX_INCHES;
  const fromFlashing = perimeterInches === undefined
    ? Infinity
    : perimeterInches - MIN_FLASHING_INCHES;

  if (fromFlashing <= fromBoards && Number.isFinite(fromFlashing)) {
    return { cap: fromFlashing, reason: 'flashing' };
  }
  if (Number.isFinite(fromBoards)) return { cap: fromBoards, reason: 'boards' };
  return { cap: Infinity, reason: null };
}

/**
 * How big a sump has to be, in inches on a side.
 *
 * NRCA p.164: "Tapered insulation layouts should be designed to form a sump that
 * measures the size of the drain bowl's diameter plus approximately 24 inches at
 * roof drains." A 12 in bowl wants 36 in square.
 *
 * A sump is its own component, not a dip the slope arithmetic happens to make.
 * NRCA p.306 has them premanufactured at 4 or 8 ft a side, and notes field
 * shaving is no longer feasible at modern insulation thicknesses — which is
 * what adjustable drain risers exist to solve.
 */
export const sumpSide = (drainBowlInches: number): number => drainBowlInches + 24;

// ── The build-up ───────────────────────────────────────────────────────────

/**
 * How thick a stack of layers is, or nothing.
 *
 * `null` and not `0`. A roof with no layer thicknesses recorded is a roof whose
 * build-up nobody has said yet, and drawing it as zero — or as some pleasant
 * default that makes the picture read as a building — is the invention this pass
 * exists to remove.
 *
 * Both real tools carry the thickness on the layer and derive the total from it.
 * IfcOpenShell's own worked example is explicit that a slab's extrusion depth
 * "must equal .013 + .092 + .013 from our type"; FreeCAD's multi-material is
 * three parallel lists, one of which is `Thicknesses`. Neither lets you type the
 * total in directly, and neither should this.
 */
export function buildUpInches(
  layers: readonly { readonly thickness?: number }[],
): number | null {
  let total = 0;
  let any = false;
  for (const l of layers) {
    if (l.thickness === undefined) continue;
    // Thickness is a magnitude. Which way the stack grows is a separate thing —
    // `IfcMaterialLayer.LayerThickness` is an IfcNonNegativeLengthMeasure and
    // FreeCAD clamps a negative `Height` to zero rather than flipping it.
    total += Math.max(0, l.thickness);
    any = true;
  }
  return any ? total : null;
}

/**
 * Which way a stack of layers grows, away from its reference surface.
 *
 * Named rather than baked into a sign, because every real system names it: IFC
 * puts `DirectionSense` (POSITIVE/NEGATIVE) and `OffsetFromReferenceLine` on the
 * usage while the layer itself is a non-negative magnitude; FreeCAD puts a
 * `Normal` vector next to a non-negative `Height`. The Model had the direction
 * hidden inside a `rotateX` and a `translate`, which is how it ended up with a
 * deck hanging below a surface nobody had said was the top of anything.
 *
 * A roof build-up sits **on** the deck: the reference surface is the top of
 * structural deck and the layers grow up. That is the datum the whole taper
 * schedule is dimensioned from (NRCA Ch.4 throughout — "above-deck insulation",
 * board thickness at low point and high point). It is not the only datum on the
 * roof: flashing heights are measured from the top of the finished membrane
 * instead, which is why `capInches` works in inches above the deck and
 * `MIN_FLASHING_INCHES` is taken off the perimeter rather than added to the
 * stack.
 */
export const GROWS_UP_FROM_DECK = 1;
