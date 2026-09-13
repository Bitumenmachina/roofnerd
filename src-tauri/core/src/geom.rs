// ── Geometry, in feet ──────────────────────────────────────────────────────
// Points, rings and planes. Plan coordinates, always in feet: a trace is stored
// in page units and the page's scale turns it into feet before anything here
// sees it, the same way `packages/app/src/scale.ts` does for the drawing.
//
// Plan y grows DOWN the sheet, because that is how a roof plan is read and how
// every trace in this program is stored. Nothing here flips it.

use serde::{Deserialize, Serialize};

/// A point on the roof plan, in feet.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Straight-line distance, written the way the drawing writes it.
    ///
    /// `sqrt(dx² + dy²)` and not `hypot`, because the number this reproduces
    /// came out of `nearest` in `packages/app/src/editors/model.ts`, which is
    /// written that way. The two agree to a few last bits and a few last bits
    /// is exactly what "the same number the TypeScript produces" means here.
    pub fn distance(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// The box a ring sits in. `None` for a ring with no points — an unscaled sheet
/// gives a trace no feet at all, and a box round nothing is not a box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

pub fn bounds(ring: &[Point]) -> Option<Bounds> {
    let first = ring.first()?;
    let mut b = Bounds { min_x: first.x, max_x: first.x, min_y: first.y, max_y: first.y };
    for p in ring {
        b.min_x = b.min_x.min(p.x);
        b.max_x = b.max_x.max(p.x);
        b.min_y = b.min_y.min(p.y);
        b.max_y = b.max_y.max(p.y);
    }
    Some(b)
}

/// Is this point inside this ring?
///
/// Ray casting, the textbook primitive, ported line for line from `inside` in
/// `packages/app/src/editors/model.ts` so that a point the drawing called
/// inside is a point this calls inside. A point exactly on an edge is neither,
/// which is a property of the algorithm and not of this port; it is why the
/// readout asks the tapered surface before it asks the deck.
pub fn inside(p: Point, ring: &[Point]) -> bool {
    if ring.is_empty() {
        return false;
    }
    let mut hit = false;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let a = ring[i];
        let b = ring[j];
        if (a.y > p.y) != (b.y > p.y) && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x {
            hit = !hit;
        }
        j = i;
    }
    hit
}

/// How long a traced run is, corner to corner. Open: the last point is the end,
/// not a return to the first.
pub fn run_length(traced: &[Point]) -> f64 {
    let mut length = 0.0;
    for pair in traced.windows(2) {
        // `hypot`, because the sum this reproduces is written with `Math.hypot`.
        length += (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
    }
    length
}

/// A sloping plane on the roof, as the payload will carry it from C1 on.
///
/// `origin` is a point the plane passes through at its own low edge, `direction`
/// is the unit vector the fall rises along in plan, and `slope` is inches per
/// foot. Thickness at p is then `start + rise_at(p)`, clipped by the cap — which
/// is analytic, so a section cut or a readout is exact rather than sampled.
///
/// Four of these come off every drainage region from C1 on: the drain is the
/// origin of all four, the directions are the field's own axes and their
/// opposites, and the thickness is the greatest rise of the four. That is the
/// four-way pattern NRCA generally prefers to an interior drain (Membrane Roof
/// Systems 2019, p.165, Fig. 4-11), and it is why the valleys come out at 45
/// degrees without anybody drawing one: two planes tie exactly where the two
/// runs are equal.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plane {
    pub origin: Point,
    pub direction: Point,
    pub slope: f64,
}

impl Plane {
    /// How far above its own low edge the plane has risen at p, in inches.
    /// Negative behind the origin, which is what lets `max` over several planes
    /// pick the one that governs.
    pub fn rise_at(&self, p: Point) -> f64 {
        ((p.x - self.origin.x) * self.direction.x + (p.y - self.origin.y) * self.direction.y)
            * self.slope
    }
}

// ── polygons, half planes and creases ──────────────────────────────────────
// From C1 on, the field is laid out rather than sampled, and laying it out is
// polygon work: a drainage region is the field's ring cut by a half plane per
// other drain, a facet is that region cut by the wedge one plane governs, and
// the plateau where the cap binds is the region with a square taken out of it.
//
// The boolean work is `geo`'s (D147). Lines are cut here instead, against the
// ring and against the half planes, because the two lines C1 has to cut lie ON
// boundaries — a ridge IS the edge between two regions — and a polygon clipper
// asked about a line lying along its own edge answers differently from one run
// to the next. Cutting a segment by an interval of its own parameter is exact
// and says the same thing every time.

use geo::geometry::{Coord, LineString, MultiPolygon, Polygon};
use geo::{Area, BooleanOps};

/// A polygon in feet: an outline and whatever holes it has.
///
/// Holes are not decoration. Take a square out of the middle of a region — which
/// is what the cap does to a field whose drain sits well inside it — and what is
/// left has a hole in it. Answering with the outline alone would report the whole
/// region as no fall, and the no-fall figure is the one an estimator walks.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub ring: Vec<Point>,
    #[serde(default)]
    pub holes: Vec<Vec<Point>>,
}

impl Shape {
    pub fn of(ring: Vec<Point>) -> Self {
        Self { ring, holes: Vec::new() }
    }

    /// Square feet, holes taken off.
    pub fn square_feet(&self) -> f64 {
        polygon_of(self).unsigned_area()
    }
}

/// Half of the plan, kept on one side of a line: every p with
/// `(p − at) · normal <= 0`. The normal points at the half being thrown away.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HalfPlane {
    pub at: Point,
    pub normal: Point,
}

impl HalfPlane {
    /// How far p is outside. Negative is kept, zero is the line itself.
    pub fn beyond(&self, p: Point) -> f64 {
        (p.x - self.at.x) * self.normal.x + (p.y - self.at.y) * self.normal.y
    }

    pub fn holds(&self, p: Point) -> bool {
        self.beyond(p) <= 0.0
    }
}

fn coords_of(ring: &[Point]) -> LineString<f64> {
    LineString::from(ring.iter().map(|p| Coord { x: p.x, y: p.y }).collect::<Vec<_>>())
}

fn points_of(ring: &LineString<f64>) -> Vec<Point> {
    let mut out: Vec<Point> = ring.coords().map(|c| Point::new(c.x, c.y)).collect();
    // `geo` closes a ring by repeating its first point; a ring in this program is
    // open, the way a trace arrives from the sheet.
    if out.len() > 1 && out[0] == out[out.len() - 1] {
        out.pop();
    }
    out
}

pub fn polygon_of(shape: &Shape) -> Polygon<f64> {
    Polygon::new(coords_of(&shape.ring), shape.holes.iter().map(|h| coords_of(h)).collect())
}

fn shapes_of(polygons: &MultiPolygon<f64>) -> Vec<Shape> {
    polygons
        .iter()
        .map(|p| Shape {
            ring: points_of(p.exterior()),
            holes: p.interiors().iter().map(points_of).collect(),
        })
        .filter(|s| s.ring.len() >= 3)
        .collect()
}

/// The half plane as a rectangle big enough to cover everything in play.
///
/// `reach` comes off the ring being cut rather than being a constant: a roof is
/// a hundred feet across and a site plan is a thousand, and a rectangle that
/// covers one does not cover the other.
fn half_plane_of(half: HalfPlane, reach: f64) -> Polygon<f64> {
    let n = half.normal;
    let along = Point::new(-n.y, n.x);
    let corner = |s: f64, d: f64| {
        Point::new(half.at.x + along.x * s + n.x * d, half.at.y + along.y * s + n.y * d)
    };
    Polygon::new(
        coords_of(&[corner(-reach, 0.0), corner(reach, 0.0), corner(reach, -reach), corner(-reach, -reach)]),
        vec![],
    )
}

/// The square the cap draws round a drain, in the field's own axes.
///
/// Not a circle: the thickness is the greater of the two axis runs, so the line
/// the cap binds along is a square standing square to the boards, not a ring at
/// a radius. Where the fall runs out is a fact about the layout.
pub fn square_of(centre: Point, u: Point, v: Point, half: f64) -> Shape {
    rectangle_of(centre, u, v, half, half)
}

/// The ground one plane of a set governs: where its own rise is the greatest.
///
/// Every plane in a drainage region rises from the same drain at the same slope,
/// so one has risen further than another exactly where the run in its direction is
/// longer — `(p − drain) · (theirs − mine) <= 0`, one half plane per other plane.
/// Nothing about 45 degrees is written here and that is the point: on a four-way
/// layout the four wedges come out bounded by the diagonals, which is where NRCA's
/// valleys are (p.166, Figs. 4-15 and 4-16), and on a two-way layout the same
/// expression gives each plane a half of the region. The pattern decides; this
/// does not.
pub fn governed_by(origin: Point, directions: &[Point], mine: usize) -> Vec<HalfPlane> {
    let me = directions[mine];
    directions
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != mine)
        .map(|(_, theirs)| HalfPlane {
            at: origin,
            normal: Point::new(theirs.x - me.x, theirs.y - me.y),
        })
        .collect()
}

/// Cut shapes by half planes, one after another.
pub fn cut_by(shapes: &[Shape], halves: &[HalfPlane], reach: f64) -> Vec<Shape> {
    let mut now: MultiPolygon<f64> = MultiPolygon::new(shapes.iter().map(polygon_of).collect());
    for half in halves {
        if now.0.is_empty() {
            return Vec::new();
        }
        now = now.intersection(&half_plane_of(*half, reach));
    }
    shapes_of(&now)
}

/// What is left of one shape when another is taken out of it.
pub fn take_out(shape: &Shape, out: &Shape) -> Vec<Shape> {
    shapes_of(&polygon_of(shape).difference(&polygon_of(out)))
}

/// What one shape and a set of shapes have in common.
///
/// The board grid is this and nothing else: a 4 ft cell laid on the plan, kept
/// only where the ground it is laid on is. A cell that hangs off the roof comes
/// back as the piece that is on it — or as nothing, where none of it is — and a
/// cell a concave outline cuts twice comes back as two pieces, which is why the
/// answer is a list and not a shape.
pub fn common_with(cell: &Shape, ground: &[Shape]) -> Vec<Shape> {
    if ground.is_empty() {
        return Vec::new();
    }
    let mut all: MultiPolygon<f64> = MultiPolygon::new(ground.iter().map(polygon_of).collect());
    all = all.intersection(&MultiPolygon::new(vec![polygon_of(cell)]));
    shapes_of(&all)
}

/// What is left of a set of shapes when several others are taken out of them.
///
/// The sumps come out of the ground the boards are laid on: a sump is a hole cut
/// down to the drain and the field boards do not run through it, so counting a
/// board and a sump panel on the same square feet would buy the same ground twice.
pub fn take_out_all(shapes: &[Shape], outs: &[Shape]) -> Vec<Shape> {
    let mut now: MultiPolygon<f64> = MultiPolygon::new(shapes.iter().map(polygon_of).collect());
    for out in outs {
        if now.0.is_empty() {
            return Vec::new();
        }
        now = now.difference(&MultiPolygon::new(vec![polygon_of(out)]));
    }
    shapes_of(&now)
}

/// A rectangle on the plan, in a pair of axes rather than in x and y.
///
/// `centre` is the middle of it, `u` and `v` are the axes it is square to, and
/// the two halves are how far it reaches along each. `square_of` is this with one
/// half; the board grid needs the general one because a cell at the edge of a
/// region is still a whole 4 ft cell — what makes it a cut board is the ground
/// under it, not a smaller rectangle.
pub fn rectangle_of(centre: Point, u: Point, v: Point, half_u: f64, half_v: f64) -> Shape {
    let corner = |a: f64, b: f64| {
        Point::new(centre.x + u.x * a + v.x * b, centre.y + u.y * a + v.y * b)
    };
    Shape::of(vec![
        corner(-half_u, -half_v),
        corner(half_u, -half_v),
        corner(half_u, half_v),
        corner(-half_u, half_v),
    ])
}

/// How far apart the two furthest corners of a ring are — the size of the thing
/// being cut, for a rectangle that has to cover it.
pub fn reach_of(ring: &[Point]) -> f64 {
    match bounds(ring) {
        None => 0.0,
        Some(b) => (b.max_x - b.min_x).hypot(b.max_y - b.min_y),
    }
}

/// The longest side of a ring, and the direction it runs in.
///
/// It travels whole rather than as a bare direction because the layout has to be
/// able to say WHICH wall the boards were laid to. An estimator reading a tapered
/// drawing asks that first, and a drawing that cannot answer is one more picture.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub a: Point,
    pub b: Point,
    /// A unit vector along a→b, turned so the same ring always gives the same one.
    pub along: Point,
    pub run: f64,
}

/// The direction a ring mostly runs in: its longest side.
///
/// The boards are laid to this. NRCA lays a four-way pattern out on the roof's
/// own edges (p.165, Fig. 4-11) and cuts the valley boards corner to corner at
/// 45 degrees from one edge (p.166), so the pattern's axes are the field's
/// edges, not the compass. The longest side is the one an estimator would set a
/// string line to, and the first of equal-longest wins so the answer does not
/// depend on which corner the trace started at.
///
/// The direction is returned pointing right, or up where it is vertical, so the
/// same ring gives the same axis whichever way round it was traced.
pub fn dominant_edge(ring: &[Point]) -> Option<Edge> {
    if ring.len() < 2 {
        return None;
    }
    let mut best: Option<Edge> = None;
    for i in 0..ring.len() {
        let a = ring[i];
        let b = ring[(i + 1) % ring.len()];
        let run = (b.x - a.x).hypot(b.y - a.y);
        if run < 1e-9 {
            continue;
        }
        if best.map(|edge| run > edge.run).unwrap_or(true) {
            best = Some(Edge {
                a,
                b,
                along: Point::new((b.x - a.x) / run, (b.y - a.y) / run),
                run,
            });
        }
    }
    let mut edge = best?;
    if edge.along.x < -1e-12 || (edge.along.x.abs() <= 1e-12 && edge.along.y < 0.0) {
        edge.along = Point::new(-edge.along.x, -edge.along.y);
    }
    Some(edge)
}

/// Square to a direction, turned the way plan y turns.
pub fn square_to(u: Point) -> Point {
    Point::new(-u.y, u.x)
}

/// The pieces of the segment a→b that lie inside a ring.
///
/// Every crossing of every edge, sorted, and each span kept or dropped on
/// whether its own middle is inside. That handles a concave ring — the demo roof
/// has a bump in its north side — without asking the ring to be convex, and it
/// never has to decide what "on the edge" means, because a midpoint is not on an
/// edge unless the segment lies along it.
pub fn inside_ring(a: Point, b: Point, ring: &[Point]) -> Vec<(Point, Point)> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    if dx.hypot(dy) < 1e-12 || ring.len() < 3 {
        return Vec::new();
    }
    let at = |t: f64| Point::new(a.x + dx * t, a.y + dy * t);

    let mut cuts = vec![0.0f64, 1.0];
    for i in 0..ring.len() {
        let c = ring[i];
        let d = ring[(i + 1) % ring.len()];
        let ex = d.x - c.x;
        let ey = d.y - c.y;
        let denominator = dx * ey - dy * ex;
        if denominator.abs() < 1e-12 {
            continue;
        }
        let t = ((c.x - a.x) * ey - (c.y - a.y) * ex) / denominator;
        let s = ((c.x - a.x) * dy - (c.y - a.y) * dx) / denominator;
        if t > 0.0 && t < 1.0 && s >= 0.0 && s <= 1.0 {
            cuts.push(t);
        }
    }
    cuts.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));

    let mut out: Vec<(f64, f64)> = Vec::new();
    for pair in cuts.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        if to - from < 1e-12 {
            continue;
        }
        if !inside(at((from + to) / 2.0), ring) {
            continue;
        }
        match out.last_mut() {
            Some(last) if (last.1 - from).abs() < 1e-12 => last.1 = to,
            _ => out.push((from, to)),
        }
    }
    out.into_iter().map(|(from, to)| (at(from), at(to))).collect()
}

/// What is left of a segment after every half plane has had its say, or nothing.
pub fn within(a: Point, b: Point, halves: &[HalfPlane]) -> Option<(Point, Point)> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let mut from = 0.0f64;
    let mut to = 1.0f64;
    for half in halves {
        let at_a = half.beyond(a);
        let step = dx * half.normal.x + dy * half.normal.y;
        if step.abs() < 1e-12 {
            if at_a > 1e-12 {
                return None;
            }
            continue;
        }
        // `at_a + t · step <= 0`
        let t = -at_a / step;
        if step > 0.0 {
            to = to.min(t);
        } else {
            from = from.max(t);
        }
    }
    if to - from < 1e-12 {
        return None;
    }
    Some((
        Point::new(a.x + dx * from, a.y + dy * from),
        Point::new(a.x + dx * to, a.y + dy * to),
    ))
}

/// Which side of a→b c falls on, by sign. Zero is collinear.
fn side(a: Point, b: Point, c: Point) -> f64 {
    ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).signum()
}

/// Do two segments properly cross — each straddling the other's line?
/// `segmentsCross` in `packages/engine/src/geometry.ts`.
fn segments_cross(a1: Point, a2: Point, b1: Point, b2: Point) -> bool {
    let d1 = side(a1, a2, b1);
    let d2 = side(a1, a2, b2);
    let d3 = side(b1, b2, a1);
    let d4 = side(b1, b2, a2);
    d1 != 0.0 && d2 != 0.0 && d3 != 0.0 && d4 != 0.0 && d1 != d2 && d3 != d4
}

/// Does this ring cross itself?
///
/// `selfIntersects` in `packages/engine/src/geometry.ts:222`, ported for the same
/// reason it was written there: the area of a figure-eight comes out a number and
/// a triangulator hands back a mesh that renders. C0 left a crossing ring solved
/// and undrawn, which is wasted work in the core and a fact about the job that
/// nothing said out loud; from C1 the solve refuses it and says why.
pub fn self_intersects(ring: &[Point]) -> bool {
    if ring.len() < 4 {
        return false;
    }
    let closed = ring[0] == ring[ring.len() - 1];
    let ring = if closed { &ring[..ring.len() - 1] } else { ring };
    let n = ring.len();
    if n < 4 {
        return false;
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if j == (i + 1) % n || (j + 1) % n == i {
                continue;
            }
            if segments_cross(ring[i], ring[(i + 1) % n], ring[j], ring[(j + 1) % n]) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The demo's field, as the page's scale gives it in feet.
    fn ring() -> Vec<Point> {
        vec![
            Point::new(14.0, 50.0), Point::new(50.0, 50.0), Point::new(50.0, 42.0),
            Point::new(116.0, 42.0), Point::new(116.0, 50.0), Point::new(122.0, 50.0),
            Point::new(122.0, 134.0), Point::new(14.0, 134.0),
        ]
    }

    #[test]
    fn a_point_in_the_field_is_in_the_field() {
        assert!(inside(Point::new(74.0, 116.0), &ring()));
    }

    /// The notch matters: between y 42 and 50 the roof is only as wide as the
    /// bump, so a point east of it is off the roof however square the box looks.
    #[test]
    fn a_point_in_the_box_but_off_the_roof_is_off_the_roof() {
        assert!(!inside(Point::new(120.0, 44.0), &ring()));
    }

    #[test]
    fn the_box_round_the_field_is_the_field_s_own_extent() {
        let b = bounds(&ring()).expect("the field has points");
        assert_eq!((b.min_x, b.max_x, b.min_y, b.max_y), (14.0, 122.0, 42.0, 134.0));
    }

    #[test]
    fn nothing_has_no_box() {
        assert!(bounds(&[]).is_none());
    }

    #[test]
    fn a_plane_rises_along_its_own_direction_and_nowhere_else() {
        let plane = Plane { origin: Point::new(10.0, 10.0), direction: Point::new(1.0, 0.0), slope: 0.25 };
        assert_eq!(plane.rise_at(Point::new(22.0, 10.0)), 3.0);
        assert_eq!(plane.rise_at(Point::new(10.0, 99.0)), 0.0);
    }
}
