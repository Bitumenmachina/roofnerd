// ── The tapered package ────────────────────────────────────────────────────
// What the field is thick, where the water goes, where it stops going, the lines
// the boards are cut along, and the crickets.
//
// THE FIELD IS LAID OUT, NOT SAMPLED (C1, D149). C0 reproduced what the drawing
// did then: a cone from each drain — the thickness at a point was the thickness
// at the drain plus the slope times the STRAIGHT-LINE distance to the nearest
// one, sampled onto a two foot grid. That is a picture of drainage and it is not
// a tapered package, and the view said so in its own key: "the surface is the
// fall, not the board layout".
//
// What NRCA lays out instead (Membrane Roof Systems 2019, ch.4 pp.163–168):
//
//   the regions  one per drain. The line between two of them sits equidistant
//                between the drains and square to the line joining them (p.166,
//                Fig. 4-17) — a perpendicular bisector, so the regions are the
//                cells those bisectors cut out of the field
//   the planes   four to a region for an interior drain, one uniform slope over
//                all four (p.165, Fig. 4-11), laid out on the field's own edges.
//                Thickness is the greatest of the four runs, not the diagonal
//                one: `T + TAPER × max(|along|, |across|)`
//   the valleys  where two of those planes tie, which is at 45° to the slope
//                exactly, because equal runs in two square directions is a
//                diagonal (p.166 and Figs. 4-15, 4-16: "all valley centrelines
//                should be 45 degrees from the direction of slope … the boards
//                easily can be cut at 45 degrees from one edge or corner to the
//                opposing corner")
//   the no-fall  where the cap binds. A square, not a circle, because the run
//                that governs is the greater of two square runs
//
// None of those is drawn from a rule of its own. The valleys and the ridges FALL
// OUT of the planes, which is why they cannot disagree with the surface they lie
// in, and why an off-45° valley cannot occur in this scheme at all — the notes
// say so for C2, which lays boards on this.
//
// Every number goes out solved: the region polygons, the facet each plane
// governs, the creases with the depth at each end, the plateau. The drawing
// places vertices and does no geometry; `packages/engine/src/solved.ts` reads a
// shipped plane where the pointer is, which is reading and not solving, and the
// two copies of that one expression are pinned to the same points on both sides.
//
// The conventions are the engine's, at `packages/engine/src/roof.ts`, each with
// the manual it came from. They are stated twice — once there for the money and
// once here for the model — and C1 adds the test that pins the two copies
// together. Where a number here has a source, the source is on it.

use crate::boards::{bill_of, boards_over, empty_bill, ground_for, merge, no_schedule_because};
use crate::cricket::{cricket, saddles_between, tie_directions, Layout};
use crate::geom::{
    bounds, cut_by, dominant_edge, inside, inside_ring, reach_of, self_intersects,
    governed_by, square_to, take_out_all, within, HalfPlane, Plane, Point, Shape,
};
use crate::model::{
    BoardBill, CapReason, Condition, Crease, Cricket, Document, Facet, FlowArrow, Refused,
    RidgeSegment, SolvedTaper, Sump, TaperField, TaperRegion,
};
use crate::profile::{schedule, ProfileSet};
use crate::sump::{sump_at, sump_side_inches};

/// The most one layer of tapered polyiso is made in, in inches.
/// `roof.ts` — Carlisle InsulBase Tapered, 0.5 in to 4.5 in in a single layer.
pub const SINGLE_LAYER_MAX_INCHES: f64 = 4.5;

/// Flashing reaches 8 in above the finished roof, so the roof cannot rise past
/// 8 in below whatever it flashes against. `roof.ts` — NRCA p.159, pp.251–252.
pub const MIN_FLASHING_INCHES: f64 = 8.0;

/// A cricket is cut at twice the slope of the field beside it.
/// `roof.ts` — NRCA p.79 and p.165.
pub const CRICKET_SLOPE_MULTIPLE: f64 = 2.0;

/// Two lengths in feet this close together are the same length.
///
/// A hundredth of an inch. Everything here is a roof measured in feet and cut in
/// eighths, and the numbers that need separating — a crease from a corner, a run
/// from the run beside it — are inches apart or more. What this refuses is a
/// crease of no length, which a boolean cut leaves behind at a corner.
const NEAR_ENOUGH: f64 = 1e-3;

/// What caps the build-up, and which constraint bound.
///
/// `capInches` in `packages/engine/src/roof.ts`. `None` where the TypeScript
/// answers `Infinity`: with neither a board count nor a perimeter there is no
/// cap, and the field rises unbounded. That is the honest answer rather than a
/// made-up ceiling, and it is not a zero.
pub fn cap_inches(
    start_inches: f64,
    boards: Option<f64>,
    perimeter_inches: Option<f64>,
) -> (Option<f64>, Option<CapReason>) {
    let from_boards = boards.map(|b| start_inches + b * SINGLE_LAYER_MAX_INCHES);
    let from_flashing = perimeter_inches.map(|p| p - MIN_FLASHING_INCHES);

    if let Some(flashing) = from_flashing {
        if flashing <= from_boards.unwrap_or(f64::INFINITY) {
            return (Some(flashing), Some(CapReason::Flashing));
        }
    }
    if let Some(boards) = from_boards {
        return (Some(boards), Some(CapReason::Boards));
    }
    (None, None)
}

/// The most a cricket may be long against how wide it is.
/// `maxLengthToWidth` in `roof.ts` — NRCA Fig. 4-13, p.166.
pub fn max_length_to_width(field_slope: f64) -> f64 {
    if field_slope >= 0.5 {
        4.0
    } else {
        3.0
    }
}

/// Where the ridge of a cricket between two drains lies: the perpendicular
/// bisector. `ridgeBetween` in `roof.ts` — NRCA pp.166–168.
pub struct Ridge {
    pub at: Point,
    pub along: Point,
    pub width: f64,
}

pub fn ridge_between(a: Point, b: Point) -> Option<Ridge> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let span = dx.hypot(dy);
    // Two drains in the same place have no line between them to bisect.
    if span < 1e-6 {
        return None;
    }
    Some(Ridge {
        at: Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0),
        along: Point::new(-dy / span, dx / span),
        width: span / 2.0,
    })
}

/// How far a traced ridge is off the one the drains imply, in degrees.
/// `ridgeDisagreement` in `roof.ts`. A ridge has no direction, so it folds at 90°.
pub fn ridge_disagreement(traced: &[Point], along: Point) -> Option<f64> {
    if traced.len() < 2 {
        return None;
    }
    let a = traced[0];
    let b = traced[traced.len() - 1];
    let len = (b.x - a.x).hypot(b.y - a.y);
    if len < 1e-6 {
        return None;
    }
    let dot = ((b.x - a.x) / len) * along.x + ((b.y - a.y) / len) * along.y;
    Some(dot.abs().min(1.0).acos() * 180.0 / std::f64::consts::PI)
}

/// The lowest thing the roof has to flash against, in inches above the deck.
///
/// `perimeterInches` in `model.ts:719`. A parapet or a curb the estimator has
/// traced and given a height to is exactly this number; it caps the build-up
/// long before any board does.
pub fn perimeter_inches(conditions: &[Condition]) -> Option<f64> {
    let mut lowest = f64::INFINITY;
    for c in conditions {
        if c.kind != "line" || c.hidden || c.has_role("ridge") {
            continue;
        }
        match c.property("H") {
            Some(h) if h > 0.0 => lowest = lowest.min(h * 12.0),
            _ => {}
        }
    }
    if lowest.is_finite() {
        Some(lowest)
    } else {
        None
    }
}

/// One drainage point: where it is, whose condition counted it, and how big a
/// sump it wants.
///
/// A drain is a point and not a condition — one count condition carries all three
/// of them on the demo — but the sump is sized off the condition's own properties,
/// so the point has to travel with the id that owns it.
#[derive(Clone, Debug)]
pub struct DrainSpot {
    pub at: Point,
    pub condition_id: String,
    pub trace: usize,
    /// The sump's side in feet, whichever way it was arrived at.
    pub sump_feet: Option<f64>,
    /// What the bowl-plus-24-inches rule wants, where a bowl was stated.
    pub from_bowl_feet: Option<f64>,
}

/// Every drainage point on the roof, in feet, with its sump.
///
/// The side comes from `SUMP` in feet where the estimator typed one, because a
/// typed number wins over a derived one and stays typed. Where they named a bowl
/// instead — `BOWL`, in inches — the trade's rule gives it: bowl plus about 24 in
/// (NRCA p.164). Where both are stated both are reported, so a sump narrower than
/// the rule wants is visible rather than overruled.
fn drain_spots(
    document: &Document,
    scales: &std::collections::BTreeMap<String, f64>,
) -> Vec<DrainSpot> {
    let mut out = Vec::new();
    for c in &document.conditions {
        if !c.has_role("drain") || c.hidden {
            continue;
        }
        let from_bowl_feet = c.property("BOWL").map(|bowl| sump_side_inches(bowl) / 12.0);
        let sump_feet = c.property("SUMP").or(from_bowl_feet).filter(|s| *s > 0.0);
        for (index, trace) in c.traces.iter().enumerate() {
            for at in document.feet(trace, scales) {
                out.push(DrainSpot {
                    at,
                    condition_id: c.id.clone(),
                    trace: index,
                    sump_feet,
                    from_bowl_feet,
                });
            }
        }
    }
    out
}

// ── the layout ─────────────────────────────────────────────────────────────

/// The four planes of a four-way layout about one drain.
///
/// The drain is the origin of all four, the directions are the field's own axes
/// and their opposites, and the slope is the one the estimator stated — "with a
/// four-way slope pattern, uniform slope is provided over all four intersecting
/// insulation areas" (NRCA p.165, Fig. 4-11). Which of the four governs at a
/// place is whichever has risen furthest, so the thickness is the greater of the
/// two runs and not the diagonal one, and the ties between them are the valleys.
pub fn four_way(drain: Point, along: Point, across: Point, slope: f64) -> [Plane; 4] {
    let plane = |direction: Point| Plane { origin: drain, direction, slope };
    [
        plane(along),
        plane(Point::new(-along.x, -along.y)),
        plane(across),
        plane(Point::new(-across.x, -across.y)),
    ]
}

/// The two planes of a two-way layout about one drain.
///
/// The field falls square to the side the boards are laid to — the two directions
/// the drains are NOT in a row along — so the whole roof runs to the line the
/// drains stand on. NRCA's four-way is generally preferred at an interior drain
/// (p.165), and two-way is what a retrofit gets when the deck or the parapets will
/// not take the thickness a four-way needs. It has a cost the four-way has not: the
/// ground along the line joining two drains comes out level, so a saddle is needed
/// between every pair (p.166-168, Fig. 4-14), and `cricket::saddles_between` puts
/// them there.
pub fn two_way(drain: Point, across: Point, slope: f64) -> [Plane; 2] {
    let plane = |direction: Point| Plane { origin: drain, direction, slope };
    [plane(across), plane(Point::new(-across.x, -across.y))]
}

/// The planes of whichever pattern the field is laid out on.
pub fn planes_of(
    ways: Layout,
    drain: Point,
    along: Point,
    across: Point,
    slope: f64,
) -> Vec<Plane> {
    match ways {
        Layout::FourWay => four_way(drain, along, across, slope).to_vec(),
        Layout::TwoWay => two_way(drain, across, slope).to_vec(),
    }
}

/// The half planes that keep one drain's own ground: the cell the ridges cut.
///
/// One per other drain, each the perpendicular bisector of the two — NRCA p.166:
/// "the ridge line should be spaced equidistant between drainage points. The
/// ridge line will be perpendicular to a line that connects the two drainage
/// points." `ridge_between` is the same line, and it is used for the saddle
/// between two drains; here it is what divides the field up.
pub fn cell_of(mine: Point, drains: &[Point]) -> Vec<HalfPlane> {
    let mut out = Vec::new();
    for other in drains {
        let dx = other.x - mine.x;
        let dy = other.y - mine.y;
        let span = dx.hypot(dy);
        // A drain in the same place as this one divides nothing.
        if span < 1e-6 {
            continue;
        }
        out.push(HalfPlane {
            at: Point::new((mine.x + other.x) / 2.0, (mine.y + other.y) / 2.0),
            normal: Point::new(dx / span, dy / span),
        });
    }
    out
}

/// What the roof is doing at one place on it.
///
/// The readout under the pointer is this, and so is every vertex of the drawn
/// surface and every arrow. `packages/engine/src/solved.ts` evaluates the same
/// thing on the front end from the shipped planes — evaluating a solved plane is
/// not solving it — and the pinned tests hold the two copies to the same points.
pub struct Reading {
    /// How far the fall has run to get here, in feet, along the boards. This is
    /// the run a tapered schedule means: thickness = T + TAPER × run. It is not
    /// the straight line to the drain, and on a four-way layout the two differ by
    /// as much as the diagonal does.
    pub run_feet: f64,
    pub thickness_inches: f64,
    /// The fall has run out here: the roof is flat and the water stops moving.
    pub flat: bool,
    /// Which plane governs. Two tie along a valley, and the first of them wins so
    /// that the answer does not depend on which way the planes were listed.
    pub plane: usize,
}

/// The thickness the planes come to at a point, capped.
///
/// `None` where there are no planes: a place with no layout over it has no
/// thickness, which is not a zero.
pub fn read_at(
    p: Point,
    planes: &[Plane],
    start_inches: f64,
    cap_inches: Option<f64>,
) -> Option<Reading> {
    let mut which = 0usize;
    let mut most = f64::NEG_INFINITY;
    for (i, plane) in planes.iter().enumerate() {
        let rise = plane.rise_at(p);
        if rise > most {
            most = rise;
            which = i;
        }
    }
    if !most.is_finite() {
        return None;
    }
    let slope = planes[which].slope;
    let run = if slope.abs() < 1e-12 { 0.0 } else { most / slope };
    let raw = start_inches + most;
    Some(match cap_inches {
        Some(cap) => Reading {
            run_feet: run,
            thickness_inches: raw.min(cap),
            flat: raw > cap,
            plane: which,
        },
        None => Reading { run_feet: run, thickness_inches: raw, flat: false, plane: which },
    })
}

/// Which region a place is on, or none — a place can be off the field.
///
/// Ray casting over the regions' own ground, which is the same rule
/// `packages/engine/src/solved.ts` uses on the other side and the same rule
/// `inside` uses on the field. It is not a nearest drain: the regions are the
/// cells the bisectors cut, and asking a polygon whether it holds a point is
/// asking the layout rather than measuring to a drain.
pub fn region_at<'a>(field: &'a TaperField, p: Point) -> Option<&'a TaperRegion> {
    field.regions.iter().find(|region| {
        region.shapes.iter().any(|shape| {
            inside(p, &shape.ring) && !shape.holes.iter().any(|hole| inside(p, hole))
        })
    })
}

/// What the roof is doing at a place on a field: the region's planes, read there.
///
/// The mirror of `readTaperAt` in `packages/engine/src/solved.ts`, and the two are
/// pinned to the same four places on both sides. `None` where no region holds the
/// place, which is a place off the field and not a roof of no thickness.
pub fn read_field_at(field: &TaperField, p: Point) -> Option<Reading> {
    let region = region_at(field, p)?;
    read_at(p, &region.planes, field.start_inches, region.cap_inches)
}

/// How deep the tapered package is at a place, in inches — off whichever region
/// holds it, or off the greatest any region there comes to.
///
/// A point exactly on the boundary between two regions belongs to neither as far
/// as ray casting is concerned, and a cricket's ridge lands on one — so where no
/// region holds the point the answer is the greatest the regions come to, which
/// is the same number from either side, because a shared boundary at one
/// thickness is what a ridge IS. `walls.rs` reads this too, at a wall's own
/// inboard face: a parapet standing on the field's own boundary needs the same
/// fallback a cricket's ridge does, for the same reason.
pub fn depth_inches_at(field: &TaperField, p: Point) -> Option<f64> {
    if let Some(reading) = read_field_at(field, p) {
        return Some(reading.thickness_inches);
    }
    let best = field
        .regions
        .iter()
        .filter_map(|r| read_at(p, &r.planes, field.start_inches, r.cap_inches))
        .map(|r| r.thickness_inches)
        .fold(f64::NEG_INFINITY, f64::max);
    if best.is_finite() {
        Some(best)
    } else {
        None
    }
}

/// Where the top of the tapered package is at a place, in feet above the datum.
///
/// The deck plus the depth the layout makes there. This is what sits on the roof:
/// a cricket, a sump rim, a section cut.
pub fn surface_feet(field: &TaperField, p: Point) -> f64 {
    let depth = depth_inches_at(field, p).unwrap_or(field.start_inches);
    field.elevation + depth / 12.0
}

/// Where along a→b the plane that governs changes hands, or a ceiling takes over.
///
/// Each plane's rise is affine along a segment, so the greatest of them is a
/// piecewise-affine fold and its corners are where two of them cross. The cap
/// joins in as one more affine function — a flat one — because the place the roof
/// stops rising is a fold in exactly the same sense.
///
/// Two of them crossing is not enough, though: on a line between two drains the
/// two side planes cross each other halfway along while a third governs the whole
/// way, and cutting there would hand the drawing two segments of one straight
/// line. So a crossing is kept only where the plane that actually governs changes,
/// or where the roof goes flat. Returned as fractions of the way along, ends
/// included.
pub(crate) fn folds_along(a: Point, b: Point, planes: &[Plane], start: f64, cap: Option<f64>) -> Vec<f64> {
    let length = (b.x - a.x).hypot(b.y - a.y);
    if length < NEAR_ENOUGH {
        return Vec::new();
    }
    let step = NEAR_ENOUGH / length;
    let rises: Vec<(f64, f64)> = planes.iter().map(|p| (p.rise_at(a), p.rise_at(b))).collect();
    let mut lines = rises.clone();
    if let Some(cap) = cap {
        lines.push((cap - start, cap - start));
    }

    // Which plane governs a fraction of the way along, and whether the cap has
    // taken over there.
    let governs = |t: f64| -> (usize, bool) {
        let mut which = 0usize;
        let mut most = f64::NEG_INFINITY;
        for (i, (at_a, at_b)) in rises.iter().enumerate() {
            let rise = at_a + (at_b - at_a) * t;
            if rise > most {
                most = rise;
                which = i;
            }
        }
        (which, matches!(cap, Some(cap) if start + most > cap))
    };

    let mut crossings: Vec<f64> = Vec::new();
    for i in 0..lines.len() {
        for j in (i + 1)..lines.len() {
            let (ia, ib) = lines[i];
            let (ja, jb) = lines[j];
            let slope = (ib - ia) - (jb - ja);
            if slope.abs() < 1e-12 {
                continue;
            }
            let t = (ja - ia) / slope;
            if t > step && t < 1.0 - step {
                crossings.push(t);
            }
        }
    }
    crossings.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    crossings.dedup_by(|x, y| (*x - *y).abs() < step);

    let mut out = vec![0.0f64];
    for t in crossings {
        if governs(t - step / 2.0) != governs(t + step / 2.0) {
            out.push(t);
        }
    }
    out.push(1.0);
    out
}

/// a→b cut into pieces on each of which the surface is one plane, unclipped.
///
/// Which is what lets a crease travel as two ends and a depth at each, exactly,
/// instead of as a row of samples: on one of these pieces the depth is a straight
/// line between the two, so two numbers say all of it.
fn straight_pieces(
    a: Point,
    b: Point,
    planes: &[Plane],
    start: f64,
    cap: Option<f64>,
) -> Vec<(Point, Point)> {
    let folds = folds_along(a, b, planes, start, cap);
    let at = |t: f64| Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
    folds.windows(2).map(|pair| (at(pair[0]), at(pair[1]))).collect()
}

/// A crease, with the depth at each end, or nothing where it has no length.
fn crease(a: Point, b: Point, planes: &[Plane], start: f64, cap: Option<f64>) -> Option<Crease> {
    if (b.x - a.x).hypot(b.y - a.y) < NEAR_ENOUGH {
        return None;
    }
    Some(Crease {
        a,
        b,
        a_inches: read_at(a, planes, start, cap)?.thickness_inches,
        b_inches: read_at(b, planes, start, cap)?.thickness_inches,
    })
}

// ── the solve ──────────────────────────────────────────────────────────────

/// Solve the tapered package for the whole job.
pub fn solve(document: &Document, solve_id: u64) -> SolvedTaper {
    let scales = document.scales();
    let spots = drain_spots(document, &scales);
    let drains: Vec<Point> = spots.iter().map(|s| s.at).collect();
    let perimeter = perimeter_inches(&document.conditions);

    let mut fields = Vec::new();
    let mut refused = Vec::new();

    // The fields first, because everything else sits ON them: a sump is cut into
    // one, a cricket stands on one, and the boards are laid over one. C1 solved the
    // fields and left the crickets reading a typed elevation beside them; that is
    // the order this reverses.
    for c in &document.conditions {
        if c.hidden || c.kind != "area" || c.property("TAPER").is_none() {
            continue;
        }
        for (index, trace) in c.traces.iter().enumerate() {
            let ring = document.feet(trace, &scales);
            if ring.is_empty() {
                continue;
            }
            match taper_field(c, index, &ring, &spots, perimeter) {
                Ok(field) => fields.push(field),
                // A field the layout will not lay out says so and says why. C0
                // solved a crossing ring and left it undrawn, which is wasted work
                // in here and a fact about the job that nothing said out loud.
                Err(why) => {
                    refused.push(Refused { condition_id: c.id.clone(), trace: index, why })
                }
            }
        }
    }

    // The saddles a two-way field needs between its drains, and the ones its own
    // proportions refuse.
    let mut crickets: Vec<Cricket> = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        if field.layout != Layout::TwoWay.word() {
            continue;
        }
        let (saddles, no) = saddles_between(field, index);
        crickets.extend(saddles);
        refused.extend(no);
    }

    // Then the crickets somebody traced, each on the field its own line stands on.
    for c in &document.conditions {
        if c.hidden || c.kind != "line" || !c.has_role("ridge") {
            continue;
        }
        for (index, trace) in c.traces.iter().enumerate() {
            let ring = document.feet(trace, &scales);
            if ring.is_empty() {
                continue;
            }
            if let Some(found) = cricket(document, &scales, &fields, c, index, &ring) {
                crickets.push(found);
            }
        }
    }

    let sumps: Vec<Sump> = fields.iter().flat_map(|f| f.sumps.iter().cloned()).collect();
    SolvedTaper { solve_id, drains, fields, sumps, crickets, refused }
}

/// The setting-out one field shares: its axes, its depths, its outline and which
/// way it falls. Everything a region needs that is not its own drain.
struct Setting<'a> {
    ring: &'a [Point],
    /// The field's own axes: along its longest side, and square to that.
    along: Point,
    across: Point,
    start_inches: f64,
    slope: f64,
    cap_inches: Option<f64>,
    /// Four ways off the drain, or two. Typed as `LAYOUT`.
    ways: Layout,
    /// Big enough to cover everything in play, for the rectangles a half plane is
    /// cut with and the rays a crease is found on.
    reach: f64,
}

impl Setting<'_> {
    /// How far out the fall gets before the cap stops it, in feet along an axis.
    /// `None` where nothing caps the field, and zero where the cap is already
    /// reached at the drain.
    fn cap_run(&self) -> Option<f64> {
        match self.cap_inches {
            Some(cap) if self.slope > 0.0 => Some(((cap - self.start_inches) / self.slope).max(0.0)),
            _ => None,
        }
    }
}

/// One tapered field, laid out — or the reason it was not.
fn taper_field(
    c: &Condition,
    trace: usize,
    ring: &[Point],
    spots: &[DrainSpot],
    perimeter: Option<f64>,
) -> Result<TaperField, String> {
    let start = c.property("T").unwrap_or(0.0);
    let slope = c.property("TAPER").unwrap_or(0.0);
    let elevation = c.property("ELEV").unwrap_or(0.0);
    let (cap, cap_reason) = cap_inches(start, c.property("BOARDS"), perimeter);
    let ways = Layout::of(c.property("LAYOUT"));
    let set = ProfileSet::of(c.property("PROFILE_SET"));

    if ring.len() < 3 {
        return Err("it is traced as a line, and a line has no field to slope".into());
    }
    // Earcut says so in its own documentation: a ring that crosses itself
    // triangulates to overlapping faces with nothing raised, and its area cancels
    // toward zero. A boolean cut of one is worse — it answers, and the answer
    // means nothing.
    if self_intersects(ring) {
        return Err("the trace crosses itself, so it has no inside to lay out".into());
    }

    let edge = dominant_edge(ring).ok_or("no side of it is long enough to lay the boards to")?;
    let along = edge.along;
    let across = square_to(along);

    // The drains this field is laid out from are the drains ON it. A drain on
    // another area drains that area; giving this one a region for it would put a
    // ridge across a roof that has no water crossing it.
    let mine: Vec<&DrainSpot> = spots.iter().filter(|d| inside(d.at, ring)).collect();
    if mine.is_empty() {
        return Err("no drainage point is on it, so the fall has nowhere to run".into());
    }
    let at: Vec<Point> = mine.iter().map(|d| d.at).collect();

    let b = bounds(ring).ok_or("it has no extent")?;
    let middle = Point::new((b.min_x + b.max_x) / 2.0, (b.min_y + b.max_y) / 2.0);
    // The rectangles a half plane is cut with have to cover the ring from
    // wherever the drain stands, so the reach counts both — the ring's own
    // diagonal and how far off centre the furthest drain is — and then doubles it.
    let off_centre = at.iter().fold(0.0f64, |most, d| most.max(d.distance(middle)));
    let setting = Setting {
        ring,
        along,
        across,
        start_inches: start,
        slope,
        cap_inches: cap,
        ways,
        reach: (reach_of(ring) + off_centre) * 2.0,
    };

    let mut regions: Vec<TaperRegion> =
        (0..at.len()).map(|i| region_of(&setting, &at, i)).collect();

    // ── the sumps, then the boards laid round them ──────────────────────────
    //
    // In that order, because a sump is a hole cut down to the drain and the field
    // boards do not run through it. Counting a board and a sump panel on the same
    // square feet would buy the same ground twice.
    let table = schedule(slope, set);
    let panel_feet = table.as_ref().map(|t| t.panel_feet).unwrap_or(0.0);
    let mut sumps: Vec<Sump> = Vec::new();
    for (i, spot) in mine.iter().enumerate() {
        let Some(side) = spot.sump_feet else { continue };
        let region = &regions[i];
        if let Some(found) = sump_at(
            &spot.condition_id,
            spot.trace,
            spot.at,
            side,
            spot.from_bowl_feet,
            &region.shapes,
            along,
            across,
            &region.planes,
            start,
            cap,
            panel_feet,
        ) {
            sumps.push(found);
        }
    }
    let cut_outs: Vec<Shape> = sumps.iter().flat_map(|s| s.shapes.iter().cloned()).collect();

    let mut bill: Option<BoardBill> = None;
    if let Some(table) = table.as_ref() {
        let mut whole = empty_bill(table, start);
        for region in regions.iter_mut() {
            let ground = ground_for(&region.shapes, &cut_outs);
            let boards = boards_over(
                &ground,
                region.drain,
                along,
                across,
                &region.planes,
                start,
                cap,
                table,
            );
            let mine = bill_of(&boards, table, start);
            merge(&mut whole, &mine);
            region.boards = boards;
            region.bill = Some(mine);
        }
        bill = Some(whole);
    }

    let mut thin = f64::INFINITY;
    let mut thick = f64::NEG_INFINITY;
    let mut no_fall = 0.0;
    for region in &regions {
        thin = thin.min(region.thin_inches);
        thick = thick.max(region.thick_inches);
        no_fall += region.no_fall_square_feet;
    }

    Ok(TaperField {
        condition_id: c.id.clone(),
        trace,
        ring: ring.to_vec(),
        elevation,
        start_inches: start,
        slope_inches_per_foot: slope,
        cap_inches: cap,
        cap_reason,
        axis_along: along,
        axis_across: across,
        axis_edge: [edge.a, edge.b],
        flow: flow_over(&regions, setting.start_inches),
        square_feet: Shape::of(ring.to_vec()).square_feet(),
        regions,
        no_fall_square_feet: no_fall,
        thin_inches: if thin.is_finite() { Some(thin) } else { None },
        thick_inches: if thick.is_finite() { Some(thick) } else { None },
        layout: ways.word().to_string(),
        layout_refused: Layout::refused(c.property("LAYOUT")),
        profile_set_refused: ProfileSet::refused(c.property("PROFILE_SET")),
        bill_refused: match &bill {
            Some(_) => None,
            None => Some(no_schedule_because(slope, set)),
        },
        bill,
        sumps,
    })
}

/// One drainage region: the ground one drain takes, and the four planes on it.
///
/// The ground is the field's ring cut by the perpendicular bisector of this drain
/// with every other drain on the field (NRCA p.166). The cut is `geo`'s boolean
/// intersection and not a convex clip, because a field can be concave — the demo
/// roof has a bump in its north side — and a convex clip would quietly square it
/// off.
fn region_of(setting: &Setting, drains: &[Point], index: usize) -> TaperRegion {
    let drain = drains[index];
    let halves = cell_of(drain, drains);
    let shapes = cut_by(&[Shape::of(setting.ring.to_vec())], &halves, setting.reach);
    let planes = planes_of(setting.ways, drain, setting.along, setting.across, setting.slope);
    let start = setting.start_inches;
    let cap = setting.cap_inches;

    // Where the cap binds. Not a radius: every plane rises along its own axis, so
    // the ground still falling is where EVERY plane has risen less than the cap has
    // — one half plane apiece, off the plane's own direction. On a four-way layout
    // those four make the square standing square to the boards; on a two-way layout
    // the same two make a band. Where the fall runs out is a fact about the pattern.
    let cap_run = setting.cap_run();
    let binding = cap_run.filter(|run| *run > NEAR_ENOUGH);
    let cap_halves: Vec<HalfPlane> = match binding {
        None => Vec::new(),
        Some(run) => planes
            .iter()
            .map(|pl| HalfPlane {
                at: Point::new(drain.x + pl.direction.x * run, drain.y + pl.direction.y * run),
                normal: pl.direction,
            })
            .collect(),
    };
    // Nothing rises at all: the cap is already reached where the drain is.
    let flat_throughout = matches!(cap_run, Some(run) if run <= NEAR_ENOUGH);

    // ── the facets: one piece of ground per plane that governs it ───────────
    //
    // Each is where its own run is the greater of the two — the wedge between two
    // 45° lines — and then split again at the square where the cap takes over, so
    // that every facet is exactly planar or exactly flat and the drawing can
    // triangulate it without inventing a fold.
    let mut facets: Vec<Facet> = Vec::new();
    let mut no_fall = 0.0;
    let directions: Vec<Point> = planes.iter().map(|p| p.direction).collect();
    for k in 0..planes.len() {
        let wedge = governed_by(drain, &directions, k);
        let governs = cut_by(&shapes, &wedge, setting.reach);
        if governs.is_empty() {
            continue;
        }
        if flat_throughout {
            for shape in governs {
                no_fall += shape.square_feet();
                facets.push(Facet { shape, plane: k, flat: true });
            }
        } else if cap_halves.is_empty() {
            for shape in governs {
                facets.push(Facet { shape, plane: k, flat: false });
            }
        } else {
            let still_falling = cut_by(&governs, &cap_halves, setting.reach);
            for shape in take_out_all(&governs, &still_falling) {
                no_fall += shape.square_feet();
                facets.push(Facet { shape, plane: k, flat: true });
            }
            for shape in still_falling {
                facets.push(Facet { shape, plane: k, flat: false });
            }
        }
    }

    // ── the valleys: where two planes tie, which is 45° exactly ─────────────
    //
    // Four rays out of the drain at 45° to the axes, cut to the ground the region
    // holds and then cut again where the cap takes over — past the cap both
    // planes are clipped to the same flat and there is no crease to draw.
    let mut valleys: Vec<Crease> = Vec::new();
    for direction in tie_directions(setting.ways, setting.along, setting.across) {
        let far = Point::new(
            drain.x + direction.x * setting.reach,
            drain.y + direction.y * setting.reach,
        );
        for shape in &shapes {
            for (from, to) in inside_ring(drain, far, &shape.ring) {
                for (a, b) in straight_pieces(from, to, &planes, start, cap) {
                    let middle = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
                    if read_at(middle, &planes, start, cap).map(|r| r.flat).unwrap_or(true) {
                        continue;
                    }
                    if let Some(found) = crease(a, b, &planes, start, cap) {
                        valleys.push(found);
                    }
                }
            }
        }
    }

    // ── the ridges: the boundary this region shares with a neighbour ────────
    //
    // The bisector itself, cut to the field and to this region's other
    // boundaries, then cut again wherever either side's governing plane changes
    // hands — so each piece is straight on both sides and the depth the two
    // patterns come to can be compared at its ends and nowhere between.
    //
    // NRCA p.166 asks for a common, uniform thickness at a ridge and says an area
    // divider may be needed where the patterns will not give one. With one slope
    // and one set of axes over the whole field, they give one wherever the line
    // joining the two drains is square to the axes, and they do not where it is
    // slanted. So the disagreement is measured and reported rather than assumed
    // away, which is the fact an estimator needs before ordering the boards.
    let mut ridges: Vec<RidgeSegment> = Vec::new();
    for (j, other) in drains.iter().enumerate() {
        if j == index {
            continue;
        }
        let Some(bisector) = ridge_between(drain, *other) else { continue };
        let from = Point::new(
            bisector.at.x - bisector.along.x * setting.reach,
            bisector.at.y - bisector.along.y * setting.reach,
        );
        let to = Point::new(
            bisector.at.x + bisector.along.x * setting.reach,
            bisector.at.y + bisector.along.y * setting.reach,
        );
        // This region's other boundaries, and not the one the line lies on: a
        // half plane asked about a segment lying along its own edge answers on
        // the last bit of a subtraction, and the answer decides a whole ridge.
        let elsewhere: Vec<Point> = drains
            .iter()
            .enumerate()
            .filter(|(k, _)| *k != index && *k != j)
            .map(|(_, p)| *p)
            .collect();
        let others = cell_of(drain, &elsewhere);
        let theirs = planes_of(setting.ways, *other, setting.along, setting.across, setting.slope);
        let both: Vec<Plane> = planes.iter().chain(theirs.iter()).copied().collect();

        for (a, b) in inside_ring(from, to, setting.ring) {
            let Some((a, b)) = within(a, b, &others) else { continue };
            for (a, b) in straight_pieces(a, b, &both, start, cap) {
                let Some(mine) = crease(a, b, &planes, start, cap) else { continue };
                let at = |p: Point| {
                    read_at(p, &theirs, start, cap).map(|r| r.thickness_inches).unwrap_or(f64::NAN)
                };
                let off = (mine.a_inches - at(a)).abs().max((mine.b_inches - at(b)).abs());
                ridges.push(RidgeSegment {
                    a: mine.a,
                    b: mine.b,
                    a_inches: mine.a_inches,
                    b_inches: mine.b_inches,
                    meets: j,
                    disagreement_inches: off,
                });
            }
        }
    }

    // ── how thin and how thick ─────────────────────────────────────────────
    //
    // The thickness is the greatest of four planes, which makes it convex, and a
    // convex surface over a polygon is deepest at a corner — so the corners are
    // the answer and not a sample of them. The thinnest place is the drain
    // itself wherever the drain is on this ground, which is every region a
    // layout has: a region is built round its own drain.
    let mut thin = f64::INFINITY;
    let mut thick = f64::NEG_INFINITY;
    let mut at = |p: Point| {
        if let Some(r) = read_at(p, &planes, start, cap) {
            thin = thin.min(r.thickness_inches);
            thick = thick.max(r.thickness_inches);
        }
    };
    for shape in &shapes {
        for p in &shape.ring {
            at(*p);
        }
        for hole in &shape.holes {
            for p in hole {
                at(*p);
            }
        }
    }
    if shapes.iter().any(|s| inside(drain, &s.ring)) {
        at(drain);
    }

    TaperRegion {
        drain,
        square_feet: shapes.iter().map(Shape::square_feet).sum(),
        shapes,
        axis_along: setting.along,
        axis_across: setting.across,
        planes: planes.to_vec(),
        cap_inches: cap,
        facets,
        valleys,
        ridges,
        no_fall_square_feet: no_fall,
        thin_inches: if thin.is_finite() { thin } else { start },
        thick_inches: if thick.is_finite() { thick } else { start },
        // The boards are laid after the sumps are cut, because they are laid
        // around them — `boards_onto` fills these in.
        boards: Vec::new(),
        bill: None,
    }
}

/// Where the water goes: ONE arrow down each piece of ground one plane governs.
///
/// It used to be one every eight feet over the field's box — a hundred and
/// thirty-nine identical darts on the demo, which is a lawn and not a drawing.
/// A facet is the ground a single plane governs, so the water does exactly one
/// thing over the whole of it, and one long arrow down the middle says that where
/// a scatter of short ones only implies it. The handoff's Frame 2 draws it this
/// way and the reason is legibility, which is the acceptance (D156).
///
/// Flat ground gets none: there is nothing to point at when the fall has run out.
/// The direction is the governing plane's own, downhill — square to the boards on
/// a four-way layout and never at the drain, because water on a tapered roof runs
/// down the fall and then along the valley.
///
/// The length is the ground's own: how far the facet reaches along the fall,
/// taken at three fifths so the mark sits inside the ground it speaks for.
fn flow_over(regions: &[TaperRegion], start_inches: f64) -> Vec<FlowArrow> {
    /// How much of the facet's reach the mark takes up.
    const OF_THE_REACH: f64 = 0.6;

    let mut out = Vec::new();
    for region in regions {
        for facet in &region.facets {
            if facet.flat || facet.shape.ring.len() < 3 {
                continue;
            }
            let Some(plane) = region.planes.get(facet.plane) else { continue };
            let Some(p) = middle_of(&facet.shape) else { continue };
            let Some(reading) = read_at(p, &region.planes, start_inches, region.cap_inches) else {
                continue;
            };
            let fall = plane.direction;
            // How far the ground reaches along the fall, which is what the mark
            // has to be long enough to speak for and short enough to stay inside.
            let mut least = f64::INFINITY;
            let mut most = f64::NEG_INFINITY;
            for q in &facet.shape.ring {
                let along = q.x * fall.x + q.y * fall.y;
                least = least.min(along);
                most = most.max(along);
            }
            let reach = if most > least { most - least } else { 0.0 };
            out.push(FlowArrow {
                x: p.x,
                y: p.y,
                thickness_inches: reading.thickness_inches,
                angle: (-fall.y).atan2(-fall.x),
                length_feet: reach * OF_THE_REACH,
            });
        }
    }
    out
}

/// A point inside a polygon, for a mark to stand on.
///
/// The area-weighted centroid where that lands inside — which it does on every
/// convex facet, and every facet of a four-way layout on a convex field is one.
/// A region cut out of a concave outline can hand back a facet whose centroid is
/// off it, and a mark standing off its own ground is worse than no mark, so the
/// fallback walks a coarse lattice of the facet's own box and takes the inside
/// point nearest the centroid.
pub fn middle_of(shape: &Shape) -> Option<Point> {
    let ring = &shape.ring;
    let mut twice_area = 0.0;
    let mut cx = 0.0;
    let mut cy = 0.0;
    for i in 0..ring.len() {
        let a = ring[i];
        let b = ring[(i + 1) % ring.len()];
        let cross = a.x * b.y - b.x * a.y;
        twice_area += cross;
        cx += (a.x + b.x) * cross;
        cy += (a.y + b.y) * cross;
    }
    let held = |p: Point| {
        inside(p, ring) && !shape.holes.iter().any(|hole| inside(p, hole))
    };
    if twice_area.abs() > 1e-12 {
        let centre = Point::new(cx / (3.0 * twice_area), cy / (3.0 * twice_area));
        if held(centre) {
            return Some(centre);
        }
    }
    let b = bounds(ring)?;
    const STEPS: usize = 12;
    let mut best: Option<(f64, Point)> = None;
    for i in 1..STEPS {
        for j in 1..STEPS {
            let p = Point::new(
                b.min_x + (b.max_x - b.min_x) * (i as f64 / STEPS as f64),
                b.min_y + (b.max_y - b.min_y) * (j as f64 / STEPS as f64),
            );
            if !held(p) {
                continue;
            }
            let away = (p.x - cx).hypot(p.y - cy);
            if best.map(|(far, _)| away < far).unwrap_or(true) {
                best = Some((away, p));
            }
        }
    }
    best.map(|(_, p)| p)
}
