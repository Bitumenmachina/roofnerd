// ── Geometry ───────────────────────────────────────────────────────────────
// What a trace on a drawing measures. Carried across from the single-file
// build, where this math had been proved against real takeoffs; it is the one
// part of that program that needed no rethinking.
//
// Points are in page space — whatever unit the drawing renderer works in. A
// calibration turns page space into feet, and until a page has one, a length
// has no answer and says so instead of guessing.

/** A point on a page. */
export interface Point {
  readonly x: number;
  readonly y: number;
}

export const distance = (a: Point, b: Point): number => Math.hypot(b.x - a.x, b.y - a.y);

export function polylineLength(points: readonly Point[]): number {
  let total = 0;
  for (let i = 1; i < points.length; i++) total += distance(points[i - 1]!, points[i]!);
  return total;
}

/** The shoelace formula. Sign is dropped: a roof traced backwards is the same roof. */
export function polygonArea(points: readonly Point[]): number {
  let sum = 0;
  const n = points.length;
  for (let i = 0; i < n; i++) {
    const a = points[i]!;
    const b = points[(i + 1) % n]!;
    sum += a.x * b.y - b.x * a.y;
  }
  return Math.abs(sum) / 2;
}

/** A polygon's perimeter — the closing side included, which a polyline has not got. */
export function polygonPerimeter(points: readonly Point[]): number {
  if (points.length < 2) return 0;
  return polylineLength(points) + distance(points[points.length - 1]!, points[0]!);
}

// ── Scale ──────────────────────────────────────────────────────────────────

/** How many feet one page unit is worth. */
export interface Calibration {
  readonly feetPerUnit: number;
}

/**
 * Two points on the drawing, and how far apart they really are. The way an
 * estimator scales a sheet: put the cursor on both ends of a known dimension
 * and type what it says.
 */
export function calibrateFromTwoPoints(a: Point, b: Point, realFeet: number): Calibration | null {
  const onPage = distance(a, b);
  if (!(onPage > 0) || !(realFeet > 0)) return null;
  return { feetPerUnit: realFeet / onPage };
}

/**
 * The architect's scales, as feet per drawing inch. A quarter-inch drawing puts
 * four feet in every inch of paper.
 */
export const ARCHITECTURAL_SCALES: readonly { readonly label: string; readonly feetPerInch: number }[] = [
  { label: '1/8" = 1\'-0"', feetPerInch: 96 / 12 },
  { label: '3/16" = 1\'-0"', feetPerInch: 64 / 12 },
  { label: '1/4" = 1\'-0"', feetPerInch: 48 / 12 },
  { label: '3/8" = 1\'-0"', feetPerInch: 32 / 12 },
  { label: '1/2" = 1\'-0"', feetPerInch: 24 / 12 },
  { label: '3/4" = 1\'-0"', feetPerInch: 16 / 12 },
  { label: '1" = 1\'-0"', feetPerInch: 12 / 12 },
] as const;

/**
 * A preset scale, given how many page units make up one inch of paper. A PDF
 * page is measured in points and there are 72 of them to the inch, so a page
 * rendered at its natural size passes 72 here.
 */
export const calibrateFromScale = (feetPerInch: number, unitsPerInch: number): Calibration => ({
  feetPerUnit: feetPerInch / unitsPerInch,
});

// ── Pitch ──────────────────────────────────────────────────────────────────

/**
 * What a roof's slope does to its area. A 5:12 roof is 8.3% more surface than
 * the footprint it covers; a 12:12 is 41.4% more. This is the number the slope
 * column in an estimator's spreadsheet holds, and it is why plan area is never
 * the quantity you buy material against.
 */
export function pitchFactor(risePerFoot: number): number {
  if (!Number.isFinite(risePerFoot) || risePerFoot < 0) return 1;
  return Math.sqrt(1 + (risePerFoot / 12) ** 2);
}

// ── Hit-testing ────────────────────────────────────────────────────────────

export function distanceToSegment(p: Point, a: Point, b: Point): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const lengthSquared = dx * dx + dy * dy;
  if (!lengthSquared) return distance(p, a);
  let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / lengthSquared;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

export function pointInPolygon(p: Point, points: readonly Point[]): boolean {
  let inside = false;
  for (let i = 0, j = points.length - 1; i < points.length; j = i++) {
    const a = points[i]!;
    const b = points[j]!;
    if ((a.y > p.y) !== (b.y > p.y) && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) {
      inside = !inside;
    }
  }
  return inside;
}

/** How near the cursor is to a traced shape — for picking one off a crowded sheet. */
export function distanceToShape(p: Point, kind: 'area' | 'line' | 'count', points: readonly Point[]): number {
  if (points.length === 0) return Infinity;
  if (kind === 'count') return Math.min(...points.map((q) => distance(p, q)));

  let best = Infinity;
  for (let i = 1; i < points.length; i++) best = Math.min(best, distanceToSegment(p, points[i - 1]!, points[i]!));
  if (kind === 'area' && points.length > 2) {
    best = Math.min(best, distanceToSegment(p, points[points.length - 1]!, points[0]!));
    if (pointInPolygon(p, points)) return 0;
  }
  return best;
}

// ── Estimator notation ─────────────────────────────────────────────────────

/**
 * Read a length the way it is written on a drawing: 12'-6", 12' 6", 6", 12.5,
 * 4'-6 1/2". Returns decimal feet, or null if it is not a length.
 */
export function parseFeet(input: string | number | null | undefined): number | null {
  if (input == null) return null;
  if (typeof input === 'number') return Number.isFinite(input) ? input : null;

  const s = input
    .trim()
    .toLowerCase()
    .replace(/feet|foot|ft\.?/g, "'")
    .replace(/inches|inch|in\.?/g, '"')
    .replace(/\s*-\s*/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  if (!s) return null;

  // A whole number, a decimal, a fraction, or a whole-and-fraction.
  const number = (text: string): number | null => {
    const m = text.trim().match(/^(\d+(?:\.\d+)?)?\s*(?:(\d+)\s*\/\s*(\d+))?$/);
    if (!m || (m[1] === undefined && m[2] === undefined)) return null;
    let value = m[1] ? parseFloat(m[1]) : 0;
    if (m[2]) {
      const denominator = parseInt(m[3]!, 10);
      if (!denominator) return null;
      value += parseInt(m[2], 10) / denominator;
    }
    return value;
  };

  let m = s.match(/^([\d./ ]+)'\s*(?:([\d./ ]+)"?)?$/);
  if (m) {
    const feet = number(m[1]!);
    const inches = m[2] ? number(m[2]) : 0;
    return feet == null || inches == null ? null : feet + inches / 12;
  }
  m = s.match(/^([\d./ ]+)"$/);
  if (m) {
    const inches = number(m[1]!);
    return inches == null ? null : inches / 12;
  }
  m = s.match(/^([\d./ ]+)$/);
  return m ? number(m[1]!) : null;
}

/** Write a length the way a drawing does: 12'-6 1/2". Sixteenths, reduced. */
export function formatFeetInches(feet: number | null | undefined): string {
  if (feet == null || !Number.isFinite(feet)) return '—';
  const sign = feet < 0 ? '-' : '';
  const totalSixteenths = Math.round(Math.abs(feet) * 12 * 16);
  const wholeFeet = Math.floor(totalSixteenths / (12 * 16));
  const rest = totalSixteenths - wholeFeet * 12 * 16;
  const inches = Math.floor(rest / 16);
  let sixteenths = rest - inches * 16;

  let fraction = '';
  if (sixteenths) {
    let denominator = 16;
    while (sixteenths % 2 === 0) {
      sixteenths /= 2;
      denominator /= 2;
    }
    fraction = ` ${sixteenths}/${denominator}`;
  }
  return `${sign}${wholeFeet}'-${inches}${fraction}"`;
}

/**
 * Does this ring cross itself?
 *
 * A polygon traced by hand off a PDF can double back — a click landing on the
 * wrong side of an earlier edge, a stray vertex, a ring closed through its own
 * middle. Nothing downstream notices. Area still comes out a number, because
 * the shoelace formula happily returns the signed sum of a figure-eight, and a
 * triangulator hands back a mesh that renders: earcut's own documentation says
 * outright that it "does not guarantee a correct triangulation" on a ring that
 * self-crosses, and it says so without raising anything.
 *
 * So it gets checked instead of assumed. Two segments of the same ring may only
 * meet if they are neighbours meeting at their shared vertex.
 *
 * This is the same class of defect as an unscaled page reading zero: a number
 * that is wrong and looks ordinary. It is worth more than any amount of care
 * taken while drawing.
 */
export function selfIntersects(points: readonly Point[]): boolean {
  const n = points.length;
  if (n < 4) return false;
  // A closed ring may arrive with its first point repeated at the end; the
  // duplicate is the closure, not a crossing.
  const ring = (points[0]!.x === points[n - 1]!.x && points[0]!.y === points[n - 1]!.y)
    ? points.slice(0, -1)
    : points;
  const m = ring.length;
  if (m < 4) return false;

  for (let i = 0; i < m; i += 1) {
    const a1 = ring[i]!;
    const a2 = ring[(i + 1) % m]!;
    for (let j = i + 1; j < m; j += 1) {
      // Neighbours share a vertex by construction, and the pair that closes the
      // ring are neighbours too.
      if (j === i || j === (i + 1) % m || (j + 1) % m === i) continue;
      if (segmentsCross(a1, a2, ring[j]!, ring[(j + 1) % m]!)) return true;
    }
  }
  return false;
}

/** Which side of a→b the point c falls, by sign. Zero means collinear. */
const side = (a: Point, b: Point, c: Point): number =>
  Math.sign((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x));

/**
 * Do two segments properly cross?
 *
 * Properly: each straddles the other's line. Touching at an endpoint is not a
 * crossing — a traced ring is full of vertices that touch, and treating those
 * as defects would flag every honest polygon, which is how a check gets turned
 * off inside a week.
 */
function segmentsCross(a1: Point, a2: Point, b1: Point, b2: Point): boolean {
  const d1 = side(a1, a2, b1);
  const d2 = side(a1, a2, b2);
  const d3 = side(b1, b2, a1);
  const d4 = side(b1, b2, a2);
  return d1 !== 0 && d2 !== 0 && d3 !== 0 && d4 !== 0 && d1 !== d2 && d3 !== d4;
}
