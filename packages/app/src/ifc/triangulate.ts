// ── Turning a solved shape into triangles ──────────────────────────────────
// The tapered package is a plane per facet, not a triangle per facet: a facet's
// own shape is whatever `geo`'s boolean clip left of it, which can be four
// sides, six, or a shape with a hole where a neighbouring cut took a bite out
// of it. IFC's tessellated representation wants triangles, so this is the one
// place in the writer that turns a polygon into them.
//
// Ear clipping, on the polygon exactly as the solve drew it — holes bridged in
// rather than dropped, because a hole in a solved shape is a fact about the
// roof (where a sump or a neighbouring cut took the ground away) and dropping
// it would draw board where the solve says there is none.
//
// This is graphics, not a takeoff: the AREA a facet covers is the solve's own
// `squareFeet`, never re-summed from these triangles. What this produces is
// only the shape web-ifc needs to draw what the solve already decided.

/** A plan point, the shape every ring in this file is made of. */
export interface Ring2 {
  readonly x: number;
  readonly y: number;
}

/** Twice the signed area of a ring: positive counter-clockwise, y up. */
export function signedArea2(ring: readonly Ring2[]): number {
  let sum = 0;
  for (let i = 0; i < ring.length; i += 1) {
    const a = ring[i]!;
    const b = ring[(i + 1) % ring.length]!;
    sum += a.x * b.y - b.x * a.y;
  }
  return sum;
}

/** A ring with its own closing point (first repeated at the end) dropped. */
function openRing(ring: readonly Ring2[]): Ring2[] {
  const pts = ring.slice();
  while (pts.length > 1) {
    const a = pts[0]!;
    const b = pts[pts.length - 1]!;
    if (Math.abs(a.x - b.x) < 1e-9 && Math.abs(a.y - b.y) < 1e-9) pts.pop();
    else break;
  }
  return pts;
}

/** A ring wound the way asked: positive area for CCW, negative for CW. */
function wind(ring: readonly Ring2[], ccw: boolean): Ring2[] {
  const area = signedArea2(ring);
  const isCcw = area > 0;
  return isCcw === ccw ? ring.slice() : ring.slice().reverse();
}

/** Which side of the line a-b the point p falls on, signed. */
const cross = (a: Ring2, b: Ring2, p: Ring2) =>
  (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);

/** True where p lies inside (or on) the triangle a-b-c, wound either way. */
function inTriangle(a: Ring2, b: Ring2, c: Ring2, p: Ring2): boolean {
  const d1 = cross(a, b, p);
  const d2 = cross(b, c, p);
  const d3 = cross(c, a, p);
  const hasNeg = d1 < 0 || d2 < 0 || d3 < 0;
  const hasPos = d1 > 0 || d2 > 0 || d3 > 0;
  return !(hasNeg && hasPos);
}

/**
 * Ear-clip a simple polygon (no holes), wound counter-clockwise.
 *
 * The standard algorithm: a vertex is an ear when the triangle it makes with
 * its two neighbours turns the same way as the polygon as a whole and no other
 * vertex of what is left of the polygon lies inside that triangle. Bounded by
 * the vertex count so a degenerate ring — three points nearly on a line, or a
 * shape `geo`'s fixed-point grid left with a sliver — cannot loop forever: it
 * takes whatever ear it can still find, rather than hanging the export.
 */
function earClip(simple: readonly Ring2[]): [Ring2, Ring2, Ring2][] {
  const pts = simple.slice();
  const triangles: [Ring2, Ring2, Ring2][] = [];
  let guard = pts.length * pts.length + 8;

  while (pts.length > 3 && guard > 0) {
    guard -= 1;
    let cut = -1;
    for (let i = 0; i < pts.length; i += 1) {
      const n = pts.length;
      const a = pts[(i + n - 1) % n]!;
      const b = pts[i]!;
      const c = pts[(i + 1) % n]!;
      if (cross(a, b, c) <= 1e-12) continue; // reflex or collinear: not an ear
      let clear = true;
      for (let j = 0; j < n; j += 1) {
        if (j === (i + n - 1) % n || j === i || j === (i + 1) % n) continue;
        if (inTriangle(a, b, c, pts[j]!)) { clear = false; break; }
      }
      if (clear) { cut = i; break; }
    }
    if (cut < 0) {
      // Every candidate failed — a ring `geo` handed back with points closer
      // together than its own tolerance, most likely. Take the least-reflex
      // corner rather than give up: a slightly wrong triangle here is a
      // rendering blemish, and an export that refuses to finish is worse.
      cut = 0;
    }
    const n = pts.length;
    const a = pts[(cut + n - 1) % n]!;
    const b = pts[cut]!;
    const c = pts[(cut + 1) % n]!;
    triangles.push([a, b, c]);
    pts.splice(cut, 1);
  }
  if (pts.length === 3) triangles.push([pts[0]!, pts[1]!, pts[2]!]);
  return triangles;
}

/**
 * Bridge each hole into the outer ring by the shortest edge that reaches it,
 * so the whole shape becomes one simple polygon ear clipping can eat.
 *
 * The outer ring runs counter-clockwise and each hole clockwise — opposite
 * windings are what make the bridge a zero-width slit rather than a bite out
 * of the shape — and the bridge itself is walked out and back, which is why
 * the join point appears twice.
 */
function bridgeHoles(outer: readonly Ring2[], holes: readonly (readonly Ring2[])[]): Ring2[] {
  let ring = wind(openRing(outer), true);
  for (const hole of holes) {
    const h = wind(openRing(hole), false);
    if (h.length < 3) continue;
    let best = { oi: 0, hi: 0, d: Infinity };
    for (let oi = 0; oi < ring.length; oi += 1) {
      for (let hi = 0; hi < h.length; hi += 1) {
        const dx = ring[oi]!.x - h[hi]!.x;
        const dy = ring[oi]!.y - h[hi]!.y;
        const d = dx * dx + dy * dy;
        if (d < best.d) best = { oi, hi, d };
      }
    }
    const rotatedHole = [...h.slice(best.hi), ...h.slice(0, best.hi + 1)];
    ring = [
      ...ring.slice(0, best.oi + 1),
      ...rotatedHole,
      ...ring.slice(best.oi),
    ];
  }
  return ring;
}

/** A shape in feet: an outline and whatever holes it has, the same shape `SolvedShape` is. */
export interface Shape2 {
  readonly ring: readonly Ring2[];
  readonly holes: readonly (readonly Ring2[])[];
}

/**
 * Triangulate a solved shape.
 *
 * `[]` for a ring with fewer than three distinct points — a sliver `geo`'s own
 * grid left behind, not a roof. Nothing throws: a facet this cannot triangulate
 * draws nothing rather than stopping the export the rest of the roof does not
 * need stopped.
 */
export function triangulateShape(shape: Shape2): [Ring2, Ring2, Ring2][] {
  const open = openRing(shape.ring);
  if (open.length < 3) return [];
  const merged = shape.holes.length > 0 ? bridgeHoles(open, shape.holes) : wind(open, true);
  if (merged.length < 3) return [];
  return earClip(merged);
}
