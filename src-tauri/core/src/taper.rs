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

use crate::geom::{
    bounds, cut_by, dominant_edge, inside, inside_ring, reach_of, run_length, self_intersects,
    square_of, square_to, take_out, wedge_of, within, HalfPlane, Plane, Point, Shape,
};
use crate::model::{
    CapReason, Condition, Cricket, Crease, Document, Facet, FlowArrow, Refused, RidgeSegment,
    SolvedTaper, TaperField, TaperRegion,
};

/// The most one layer of tapered polyiso is made in, in inches.
/// `roof.ts` — Carlisle InsulBase Tapered, 0.5 in to 4.5 in in a single layer.
pub const SINGLE_LAYER_MAX_INCHES: f64 = 4.5;

/// Flashing reaches 8 in above the finished roof, so the roof cannot rise past
/// 8 in below whatever it flashes against. `roof.ts` — NRCA p.159, pp.251–252.
pub const MIN_FLASHING_INCHES: f64 = 8.0;

/// A cricket is cut at twice the slope of the field beside it.
/// `roof.ts` — NRCA p.79 and p.165.
pub const CRICKET_SLOPE_MULTIPLE: f64 = 2.0;

/// One flow arrow every few feet. `model.ts` inside `taperSurface`.
const ARROW_FEET: f64 = 8.0;

/// Two lengths in feet this close together are the same length.
///
/// A hundredth of an inch. Everything here is a roof measured in feet and cut in
/// eighths, and the numbers that need separating — a crease from a corner, a run
/// from the run beside it — are inches apart or more. What this refuses is a
/// crease of no length, which a boolean cut leaves behind at a corner.
const NEAR_ENOUGH: f64 = 1e-3;

/// Inches to feet, because a property is in inches and the world is in feet.
fn feet(inches: f64) -> f64 {
    inches / 12.0
}

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

/// Every drainage point on the roof, in feet. A drain is a point, not a
/// condition. `drainPoints` in `model.ts:730`.
fn drain_points(document: &Document, scales: &std::collections::BTreeMap<String, f64>) -> Vec<Point> {
    let mut out = Vec::new();
    for c in &document.conditions {
        if !c.has_role("drain") || c.hidden {
            continue;
        }
        for trace in &c.traces {
            out.extend(document.feet(trace, scales));
        }
    }
    out
}

/// The drains named by a cricket's `between`, and only those.
/// `drainsNamedBy` in `model.ts:698`.
fn drains_named_by(
    document: &Document,
    scales: &std::collections::BTreeMap<String, f64>,
    names: &[String],
) -> Vec<Point> {
    let mut out = Vec::new();
    if names.is_empty() {
        return out;
    }
    for c in &document.conditions {
        if !names.contains(&c.id) || c.hidden {
            continue;
        }
        for trace in &c.traces {
            out.extend(document.feet(trace, scales));
        }
    }
    out
}

/// The curb a cricket says it is behind, as a ring in feet.
/// `curbNamedBy` in `model.ts:1503`.
fn curb_named_by(
    document: &Document,
    scales: &std::collections::BTreeMap<String, f64>,
    names: &[String],
) -> Option<Vec<Point>> {
    if names.is_empty() {
        return None;
    }
    for c in &document.conditions {
        if !names.contains(&c.id) || c.hidden || !c.has_role("curb") {
            continue;
        }
        for trace in &c.traces {
            let ring = document.feet(trace, scales);
            if ring.len() >= 3 {
                return Some(ring);
            }
        }
    }
    None
}

/// The two drainage points a cricket sits between: the two nearest the ridge
/// the estimator drew. `pairFor` in `model.ts:1631`.
fn pair_for(traced: &[Point], candidates: &[Point]) -> Option<(Point, Point)> {
    if candidates.len() < 2 || traced.is_empty() {
        return None;
    }
    let mid = traced[traced.len() / 2];
    let mut by_range: Vec<Point> = candidates.to_vec();
    by_range.sort_by(|a, b| {
        let ra = (a.x - mid.x).hypot(a.y - mid.y);
        let rb = (b.x - mid.x).hypot(b.y - mid.y);
        ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
    });
    Some((by_range[0], by_range[1]))
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
fn folds_along(a: Point, b: Point, planes: &[Plane], start: f64, cap: Option<f64>) -> Vec<f64> {
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
    let drains = drain_points(document, &scales);
    let perimeter = perimeter_inches(&document.conditions);

    let mut fields = Vec::new();
    let mut crickets = Vec::new();
    let mut refused = Vec::new();

    for c in &document.conditions {
        if c.hidden {
            continue;
        }
        for (index, trace) in c.traces.iter().enumerate() {
            let ring = document.feet(trace, &scales);
            if ring.is_empty() {
                continue;
            }
            if c.kind == "area" && c.property("TAPER").is_some() {
                match taper_field(c, index, &ring, &drains, perimeter) {
                    Ok(field) => fields.push(field),
                    // A field the layout will not lay out says so and says why.
                    // C0 solved a crossing ring and left it undrawn, which is
                    // wasted work in here and a fact about the job that nothing
                    // said out loud.
                    Err(why) => refused.push(Refused {
                        condition_id: c.id.clone(),
                        trace: index,
                        why,
                    }),
                }
            } else if c.kind == "line" && c.has_role("ridge") {
                if let Some(cricket) = cricket(document, &scales, c, index, &ring) {
                    crickets.push(cricket);
                }
            }
        }
    }

    SolvedTaper { solve_id, drains, fields, crickets, refused }
}

/// What every region on one field shares: its axes, its depths and its outline.
struct Layout<'a> {
    ring: &'a [Point],
    /// The field's own axes: along its longest side, and square to that.
    along: Point,
    across: Point,
    start_inches: f64,
    slope: f64,
    cap_inches: Option<f64>,
    /// Big enough to cover everything in play, for the rectangles a half plane is
    /// cut with and the rays a crease is found on.
    reach: f64,
}

impl Layout<'_> {
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
    drains: &[Point],
    perimeter: Option<f64>,
) -> Result<TaperField, String> {
    let start = c.property("T").unwrap_or(0.0);
    let slope = c.property("TAPER").unwrap_or(0.0);
    let elevation = c.property("ELEV").unwrap_or(0.0);
    let (cap, cap_reason) = cap_inches(start, c.property("BOARDS"), perimeter);

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
    let mine: Vec<Point> = drains.iter().copied().filter(|d| inside(*d, ring)).collect();
    if mine.is_empty() {
        return Err("no drainage point is on it, so the fall has nowhere to run".into());
    }

    let b = bounds(ring).ok_or("it has no extent")?;
    let middle = Point::new((b.min_x + b.max_x) / 2.0, (b.min_y + b.max_y) / 2.0);
    // The rectangles a half plane is cut with have to cover the ring from
    // wherever the drain stands, so the reach counts both — the ring's own
    // diagonal and how far off centre the furthest drain is — and then doubles it.
    let off_centre = mine.iter().fold(0.0f64, |most, d| most.max(d.distance(middle)));
    let layout = Layout {
        ring,
        along,
        across,
        start_inches: start,
        slope,
        cap_inches: cap,
        reach: (reach_of(ring) + off_centre) * 2.0,
    };

    let regions: Vec<TaperRegion> =
        (0..mine.len()).map(|i| region_of(&layout, &mine, i)).collect();

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
        flow: flow_over(&layout, &regions, ARROW_FEET),
        square_feet: Shape::of(ring.to_vec()).square_feet(),
        regions,
        no_fall_square_feet: no_fall,
        thin_inches: if thin.is_finite() { Some(thin) } else { None },
        thick_inches: if thick.is_finite() { Some(thick) } else { None },
    })
}

/// One drainage region: the ground one drain takes, and the four planes on it.
///
/// The ground is the field's ring cut by the perpendicular bisector of this drain
/// with every other drain on the field (NRCA p.166). The cut is `geo`'s boolean
/// intersection and not a convex clip, because a field can be concave — the demo
/// roof has a bump in its north side — and a convex clip would quietly square it
/// off.
fn region_of(layout: &Layout, drains: &[Point], index: usize) -> TaperRegion {
    let drain = drains[index];
    let halves = cell_of(drain, drains);
    let shapes = cut_by(&[Shape::of(layout.ring.to_vec())], &halves, layout.reach);
    let planes = four_way(drain, layout.along, layout.across, layout.slope);
    let start = layout.start_inches;
    let cap = layout.cap_inches;

    // Where the cap binds, as a square standing square to the boards rather than
    // a circle at a radius: the run that governs is the greater of two square
    // runs, so the line the cap binds along is a square. Where the fall runs out
    // is a fact about the layout, not about the distance to the drain.
    let cap_run = layout.cap_run();
    let cap_square = cap_run
        .filter(|run| *run > NEAR_ENOUGH)
        .map(|run| square_of(drain, layout.along, layout.across, run));
    let cap_halves: Vec<HalfPlane> = match cap_run.filter(|run| *run > NEAR_ENOUGH) {
        None => Vec::new(),
        Some(run) => [layout.along, layout.across]
            .iter()
            .flat_map(|axis| {
                let out = Point::new(drain.x + axis.x * run, drain.y + axis.y * run);
                let back = Point::new(drain.x - axis.x * run, drain.y - axis.y * run);
                [
                    HalfPlane { at: out, normal: *axis },
                    HalfPlane { at: back, normal: Point::new(-axis.x, -axis.y) },
                ]
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
    for (k, plane) in planes.iter().enumerate() {
        let across_it = if k < 2 { layout.across } else { layout.along };
        let wedge = wedge_of(drain, plane.direction, across_it);
        let governs = cut_by(&shapes, &wedge, layout.reach);
        if governs.is_empty() {
            continue;
        }
        match &cap_square {
            None => {
                for shape in governs {
                    facets.push(Facet { shape, plane: k, flat: false });
                }
            }
            Some(_) if flat_throughout => {
                for shape in governs {
                    no_fall += shape.square_feet();
                    facets.push(Facet { shape, plane: k, flat: true });
                }
            }
            Some(square) => {
                for shape in cut_by(&governs, &cap_halves, layout.reach) {
                    facets.push(Facet { shape, plane: k, flat: false });
                }
                for shape in &governs {
                    for out in take_out(shape, square) {
                        no_fall += out.square_feet();
                        facets.push(Facet { shape: out, plane: k, flat: true });
                    }
                }
            }
        }
    }

    // ── the valleys: where two planes tie, which is 45° exactly ─────────────
    //
    // Four rays out of the drain at 45° to the axes, cut to the ground the region
    // holds and then cut again where the cap takes over — past the cap both
    // planes are clipped to the same flat and there is no crease to draw.
    let mut valleys: Vec<Crease> = Vec::new();
    for turn in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
        let direction = Point::new(
            (layout.along.x * turn.0 + layout.across.x * turn.1) / std::f64::consts::SQRT_2,
            (layout.along.y * turn.0 + layout.across.y * turn.1) / std::f64::consts::SQRT_2,
        );
        let far = Point::new(
            drain.x + direction.x * layout.reach,
            drain.y + direction.y * layout.reach,
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
            bisector.at.x - bisector.along.x * layout.reach,
            bisector.at.y - bisector.along.y * layout.reach,
        );
        let to = Point::new(
            bisector.at.x + bisector.along.x * layout.reach,
            bisector.at.y + bisector.along.y * layout.reach,
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
        let theirs = four_way(*other, layout.along, layout.across, layout.slope);
        let both: Vec<Plane> = planes.iter().chain(theirs.iter()).copied().collect();

        for (a, b) in inside_ring(from, to, layout.ring) {
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
        axis_along: layout.along,
        axis_across: layout.across,
        planes: planes.to_vec(),
        cap_inches: cap,
        facets,
        valleys,
        ridges,
        no_fall_square_feet: no_fall,
        thin_inches: if thin.is_finite() { thin } else { start },
        thick_inches: if thick.is_finite() { thick } else { start },
    }
}

/// Where the water goes, as arrows down the plane each place is on.
///
/// One every few feet over the field's box, kept where it lands on a facet that
/// is still falling. Flat ground gets none: there is nothing to point at when the
/// fall has run out. The direction is the governing plane's own, downhill — which
/// is square to the boards on a four-way layout and never at the drain, because
/// water on a tapered roof runs down the fall and then along the valley.
fn flow_over(layout: &Layout, regions: &[TaperRegion], every: f64) -> Vec<FlowArrow> {
    let Some(b) = bounds(layout.ring) else { return Vec::new() };
    let across = ((b.max_x - b.min_x) / every).round().max(2.0) as usize;
    let down = ((b.max_y - b.min_y) / every).round().max(2.0) as usize;

    let mut out = Vec::new();
    for r in 1..down {
        for col in 1..across {
            let p = Point::new(
                b.min_x + (col as f64 * (b.max_x - b.min_x)) / across as f64,
                b.min_y + (r as f64 * (b.max_y - b.min_y)) / down as f64,
            );
            let Some((region, facet)) = facet_at(p, regions) else { continue };
            if facet.flat {
                continue;
            }
            let Some(reading) =
                read_at(p, &region.planes, layout.start_inches, layout.cap_inches)
            else {
                continue;
            };
            let fall = region.planes[facet.plane].direction;
            out.push(FlowArrow {
                x: p.x,
                y: p.y,
                thickness_inches: reading.thickness_inches,
                angle: (-fall.y).atan2(-fall.x),
            });
        }
    }
    out
}

/// Which facet of which region a place is on, or none — it can be off the roof.
fn facet_at<'a>(p: Point, regions: &'a [TaperRegion]) -> Option<(&'a TaperRegion, &'a Facet)> {
    for region in regions {
        for facet in &region.facets {
            if inside(p, &facet.shape.ring)
                && !facet.shape.holes.iter().any(|hole| inside(p, hole))
            {
                return Some((region, facet));
            }
        }
    }
    None
}

/// A cricket, from whatever the document says it serves.
///
/// A cricket behind a curb and a saddle between two drains are two different
/// things with two different rules, and the one thing that cannot tell them
/// apart is the traced line, which is a ridge either way. `between` names what
/// is served; a curb named there says this is the cricket behind it.
/// `model.ts:891–916`.
fn cricket(
    document: &Document,
    scales: &std::collections::BTreeMap<String, f64>,
    c: &Condition,
    trace: usize,
    ring: &[Point],
) -> Option<Cricket> {
    // The field this cricket sits in, for the slope it should be cut at. The
    // drawing does not ask whether that field is hidden, and neither does this.
    let field_slope = document
        .conditions
        .iter()
        .find(|o| o.kind == "area" && o.property("TAPER").is_some())
        .and_then(|o| o.property("TAPER"))
        .unwrap_or(0.0);
    let elevation = c.property("ELEV").unwrap_or(0.0);
    let stated = c.property("TAPER");

    match curb_named_by(document, scales, &c.between) {
        Some(curb) => cricket_behind(c, trace, ring, &curb, stated, field_slope, elevation),
        None => {
            let pair = pair_for(ring, &drains_named_by(document, scales, &c.between))?;
            cricket_between(c, trace, ring, pair, stated, field_slope, elevation)
        }
    }
}

/// Twice the field, unless the estimator said otherwise — in which case theirs
/// is drawn and the disagreement is reported. A manufacturer's cricket stock
/// comes in the slopes it comes in.
fn cricket_slope(stated: Option<f64>, field_slope: f64) -> (f64, bool) {
    let wanted = field_slope * CRICKET_SLOPE_MULTIPLE;
    let slope = stated.unwrap_or(wanted);
    let disagrees = match stated {
        Some(s) => field_slope > 0.0 && (s - wanted).abs() > 1e-6,
        None => false,
    };
    (slope, disagrees)
}

/// The ratio the length-to-width table is read at: the field's own slope, or —
/// where no field says — the cricket's slope halved back.
fn ratio_slope(field_slope: f64, slope: f64) -> f64 {
    if field_slope != 0.0 {
        field_slope
    } else {
        slope / CRICKET_SLOPE_MULTIPLE
    }
}

/// Two planes falling from the ridge the drains put there.
/// `cricketBetween` in `model.ts:1437`.
fn cricket_between(
    c: &Condition,
    trace: usize,
    traced: &[Point],
    pair: (Point, Point),
    stated: Option<f64>,
    field_slope: f64,
    elevation: f64,
) -> Option<Cricket> {
    let derived = ridge_between(pair.0, pair.1)?;
    let (slope, slope_disagrees) = cricket_slope(stated, field_slope);

    let length = run_length(traced);
    if length < 1e-6 {
        return None;
    }

    let width = derived.width;
    let rise = feet(slope * width);
    let half = length / 2.0;
    // Along the ridge, and square to it — square to the ridge is the direction
    // the water falls, which is the line joining the two drains.
    let along = derived.along;
    let fall = Point::new(-along.y, along.x);

    let ridge_a = Point::new(derived.at.x - along.x * half, derived.at.y - along.y * half);
    let ridge_b = Point::new(derived.at.x + along.x * half, derived.at.y + along.y * half);

    Some(Cricket {
        condition_id: c.id.clone(),
        trace,
        positions: tent(ridge_a, ridge_b, fall, width, elevation, elevation + rise),
        length_to_width: length / width,
        max_length_to_width: max_length_to_width(ratio_slope(field_slope, slope)),
        off_by: ridge_disagreement(traced, derived.along),
        slope_disagrees,
    })
}

/// A cricket behind a curb, sized off the curb rather than off the drains.
/// `cricketBehind` in `model.ts:1538` — NRCA §8.1 p.252 Fig. 8-1, and Figs.
/// 10-13/10-14 pp.309–310 for the ridge square to the face and centred on it.
fn cricket_behind(
    c: &Condition,
    trace: usize,
    traced: &[Point],
    curb: &[Point],
    stated: Option<f64>,
    field_slope: f64,
    elevation: f64,
) -> Option<Cricket> {
    if traced.len() < 2 || curb.len() < 3 {
        return None;
    }

    let middle = Point::new(
        curb.iter().map(|p| p.x).sum::<f64>() / curb.len() as f64,
        curb.iter().map(|p| p.y).sum::<f64>() / curb.len() as f64,
    );

    // Which face of the curb this cricket sits against: the one the estimator
    // started their ridge at. A slope direction could not break the tie on a
    // curb standing between two drains, and this curb does.
    let start = traced[0];
    let mut face: Option<(Point, f64, Point, Point)> = None;
    let mut nearest = f64::INFINITY;
    for pair in curb.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let run = dx.hypot(dy);
        if run < 1e-6 {
            continue;
        }
        let mid = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        let range = (mid.x - start.x).hypot(mid.y - start.y);
        if range >= nearest {
            continue;
        }
        nearest = range;
        // Square to the face and pointing away from the curb, so the cricket is
        // always outside the thing it sheds water around.
        let out = if (middle.x - mid.x) * (-dy / run) + (middle.y - mid.y) * (dx / run) >= 0.0 {
            -1.0
        } else {
            1.0
        };
        face = Some((
            mid,
            run / 2.0,
            Point::new(dx / run, dy / run),
            Point::new((-dy / run) * out, (dx / run) * out),
        ));
    }
    let (mid, half, across, along) = face?;
    if half < 1e-6 {
        return None;
    }

    let (slope, slope_disagrees) = cricket_slope(stated, field_slope);
    let length = run_length(traced);
    if length < 1e-6 {
        return None;
    }

    let rise = feet(slope * half);
    let ridge_b = Point::new(mid.x + along.x * length, mid.y + along.y * length);

    Some(Cricket {
        condition_id: c.id.clone(),
        trace,
        positions: tent(mid, ridge_b, across, half, elevation, elevation + rise),
        length_to_width: length / half,
        max_length_to_width: max_length_to_width(ratio_slope(field_slope, slope)),
        // Square to the curb's face is what the figures draw, so that is what
        // the traced line is held against here.
        off_by: ridge_disagreement(traced, along),
        slope_disagrees,
    })
}

/// The two falling planes of a cricket, as triangles: a ridge from A to B at
/// the top, and a foot half a width out on each side at the deck.
fn tent(
    ridge_a: Point,
    ridge_b: Point,
    fall: Point,
    width: f64,
    elevation: f64,
    top: f64,
) -> Vec<f64> {
    let mut positions = Vec::with_capacity(36);
    for side in [1.0, -1.0] {
        let foot_a = Point::new(ridge_a.x + fall.x * width * side, ridge_a.y + fall.y * width * side);
        let foot_b = Point::new(ridge_b.x + fall.x * width * side, ridge_b.y + fall.y * width * side);
        positions.extend_from_slice(&[
            ridge_a.x, top, ridge_a.y, ridge_b.x, top, ridge_b.y, foot_a.x, elevation, foot_a.y,
        ]);
        positions.extend_from_slice(&[
            ridge_b.x, top, ridge_b.y, foot_b.x, elevation, foot_b.y, foot_a.x, elevation, foot_a.y,
        ]);
    }
    positions
}
