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
/// Nothing produces one yet: today's cone is radial (C0 reproduces it as it is)
/// and the half-plane partition that fills these in is C1. It is written here
/// now because the payload's shape is what the front end is being wired to, and
/// a shape that changes after the wiring is the expensive kind of change.
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
