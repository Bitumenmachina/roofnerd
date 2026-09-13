// ── Walls and curbs: the solids, and the flashing ceiling ──────────────────
// What a parapet or a curb IS, and how high the membrane is allowed to run up
// it before the wall itself, the roof beneath it or the estimator's own
// judgment says stop.
//
// THE SOLID (Part 1). `trace + H + WALL` — a run with a height and a thickness,
// growing inboard from the traced exterior line. `model.ts:standUp` had this
// arithmetic, offsetting each segment by hand toward the ring's own centroid;
// that heuristic cannot survive a concave ring (the field's own bump has one),
// because the centroid of a concave ring can sit outside it, and it cannot run
// at all on an open trace (a wall along one side only), which has no middle to
// turn toward. `cavalier_contours` does a true polyline offset instead, and
// which of its two signed directions is "inboard" is read off the JOB, not the
// wall's own line: a point a hair to one side of the first segment is asked
// whether some traced roof holds it, because "inboard" means "toward a roof"
// and a wall drawn along one side of a building has no ring of its own to ask.
//
// THE FLASHING CEILING (Part 2). How high the membrane is DETAILED to run up a
// wall is a typed `FLASHUP`, in inches — 24 in on the demo's parapet against a
// 42 in wall. Untyped, it defaults to the wall's own height, which is C0's
// original behaviour (flash the whole thing) and a reasonable one for a run
// nobody has drawn a detail for yet. At a place on the wall's inboard face, the
// membrane cannot rise past `FLASHUP` and cannot rise past what the wall itself
// has left above the finished roof — `wallTopInches - finishedRoofInches(p)` —
// whichever is less, and never below zero. Because the finished roof rises and
// falls with the tapered package under it, that ceiling is a function of
// position along the run, piecewise-linear, and `FLASH_SF` is the exact area
// under it: a sum of trapezoids between the run's own breakpoints (its corners,
// and every place the roof beneath crosses from one plane, one region or one
// side of its own ceiling to another), never sampled onto a grid.
//
// `MIN_FLASHING_INCHES` (`taper.rs`, NRCA *Membrane Roof Systems* 2019 p.159 and
// pp.251–252) is 8 in — the least a base flashing is ever built to. Where the
// membrane's own run cannot reach that anywhere on a wall or curb, the wall has
// to be raised, and `CURB_RAISE_IN` says by how much. Written on every
// wall and curb condition, not curbs alone: a parapet standing over ground that
// has piled up thick enough can fail the same check a short curb does.

use crate::coping::{self, Coping};
use crate::geom::{dedup_closing, offset_ring, on_or_in, ring_is_closed, Point, Shape};
use crate::model::{Condition, Document, Line, SolvedTaper};
use crate::taper::{depth_inches_at, folds_along, MIN_FLASHING_INCHES};

/// One wall or curb run, solved: its solid, and how far the membrane is allowed
/// to climb it.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WallRun {
    pub condition_id: String,
    pub trace: usize,
    /// `curb`, or absent — a run with no stated role is a parapet: the edge of
    /// the roof rather than a thing standing on it, though the same arithmetic
    /// builds both (`model.ts`'s own comment on `standUp`).
    pub role: Option<String>,
    pub name: String,
    /// The traced exterior line, in feet.
    pub outer: Shape,
    /// The outer line offset inboard by `WALL`, in feet. Absent where nobody
    /// typed a thickness — the same ribbon `model.ts` has always drawn there.
    pub inboard: Option<Shape>,
    pub elevation_feet: f64,
    pub height_feet: f64,
    pub wall_inches: Option<f64>,
    /// How high the membrane is detailed to run, in inches: typed, or the
    /// wall's own height where nobody said.
    pub flashup_inches: f64,
    pub flashup_stated: bool,
    /// The least the flashing ceiling comes to anywhere on the run, in inches.
    /// Absent only where the roof beneath could not be read at all.
    pub flash_min_inches: Option<f64>,
    /// The exact area under the flashing ceiling along the whole run.
    pub flash_square_feet: Option<f64>,
    /// `max(0, MIN_FLASHING_INCHES - flash_min_inches)` — NRCA p.285: "roof curb
    /// heights be sufficient to maintain 8-inch minimum base flashing heights."
    pub curb_raise_inches: Option<f64>,
    /// Why the flashing ceiling could not be read everywhere on the run, where
    /// it could not.
    pub refused: Option<String>,
    pub coping: Coping,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvedWalls {
    pub solve_id: u64,
    pub runs: Vec<WallRun>,
}

/// Solve every wall and curb on the job.
pub fn solve(document: &Document, taper: &SolvedTaper, solve_id: u64) -> SolvedWalls {
    let scales = document.scales();
    let mut runs = Vec::new();
    for c in &document.conditions {
        if c.hidden || c.kind != "line" || c.has_role("ridge") {
            continue;
        }
        let height_feet = c.property("H").unwrap_or(0.0);
        if height_feet <= 0.0 {
            continue;
        }
        for (trace_index, trace) in c.traces.iter().enumerate() {
            let ring = document.feet(trace, &scales);
            if ring.len() < 2 {
                continue;
            }
            runs.push(run_of(document, taper, c, trace_index, &ring, height_feet));
        }
    }
    SolvedWalls { solve_id, runs }
}

fn run_of(
    document: &Document,
    taper: &SolvedTaper,
    c: &Condition,
    trace_index: usize,
    ring: &[Point],
    height_feet: f64,
) -> WallRun {
    let elevation_feet = c.property("ELEV").unwrap_or(0.0);
    let wall_inches = c.property("WALL");
    let closed = ring_is_closed(ring);
    // Which of the two signed offset directions points at the roof, worked out
    // once here and used for the solid, the flashing walk and the coping's own
    // corners alike — so a wall traced the other way round always answers the
    // same way about itself.
    let sign = inboard_sign(document, ring);
    let inboard = wall_inches.filter(|w| *w > 0.0).and_then(|w| offset_ring(ring, closed, sign, w / 12.0));

    let flashup_stated_value = c.property("FLASHUP");
    let flashup_inches = flashup_stated_value.unwrap_or(height_feet * 12.0);
    let wall_top_inches = elevation_feet * 12.0 + height_feet * 12.0;

    let face: Vec<Point> = match &inboard {
        Some(shape) => shape.ring.clone(),
        None => dedup_closing(ring),
    };
    let (flash_min_inches, flash_square_feet, refused) =
        flashing_along(document, taper, &face, closed, wall_top_inches, flashup_inches);
    let curb_raise_inches = flash_min_inches.map(|min| (MIN_FLASHING_INCHES - min).max(0.0));

    let name = if c.name.is_empty() { "the wall".to_string() } else { c.name.clone() };
    // The wall's OWN corners, not the offset line's: an inward offset can add a
    // vertex at a reflex corner that the traced wall never had.
    let coping = coping::solve_one(c, ring, closed, sign, wall_inches);

    WallRun {
        condition_id: c.id.clone(),
        trace: trace_index,
        role: c.role.clone(),
        name,
        outer: Shape::of(ring.to_vec()),
        inboard,
        elevation_feet,
        height_feet,
        wall_inches,
        flashup_inches,
        flashup_stated: flashup_stated_value.is_some(),
        flash_min_inches,
        flash_square_feet,
        curb_raise_inches,
        refused,
        coping,
    }
}

// ── the solid: an offset ring, not a per-segment guess ─────────────────────

/// Which side of a wall's first segment the roof is on: `+1.0` if a hair to the
/// left of its own tangent lands inside a traced roof, `-1.0` if a hair to the
/// right does.
///
/// Not asked of the wall's own ring — a run drawn along one side of a building
/// is open and has no "inside" of its own, and even a closed parapet ring is
/// only inboard-equivalent to the field it traces because this job happens to
/// trace the two the same way. What "inboard" means is "toward a roof", so
/// this asks the roofs.
fn inboard_sign(document: &Document, ring: &[Point]) -> f64 {
    let pts = dedup_closing(ring);
    if pts.len() < 2 {
        return 1.0;
    }
    let (a, b) = (pts[0], pts[1]);
    let len = a.distance(b);
    if len < 1e-9 {
        return 1.0;
    }
    let tangent = Point::new((b.x - a.x) / len, (b.y - a.y) / len);
    let left = Point::new(-tangent.y, tangent.x);
    let mid = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    const PROBE_FEET: f64 = 1e-3;
    let probe = Point::new(mid.x + left.x * PROBE_FEET, mid.y + left.y * PROBE_FEET);
    if is_over_a_roof(document, probe) {
        1.0
    } else {
        -1.0
    }
}

/// Is this point inside any traced roof area on the job?
fn is_over_a_roof(document: &Document, p: Point) -> bool {
    let scales = document.scales();
    document.conditions.iter().any(|c| {
        c.kind == "area"
            && !c.hidden
            && c.traces.iter().any(|t| {
                let ring = document.feet(t, &scales);
                ring.len() >= 3 && on_or_in(p, &ring)
            })
    })
}

// ── the finished roof under a wall ──────────────────────────────────────────

/// The finished roof at a place, in inches above the deck: `deck + base +
/// taper(p) + cover board + membrane`.
///
/// The area condition whose ring holds the point says which roof this is.
/// `None` where none does (the wall stands over ground nothing traced), where
/// the area is meant to be tapered but did not solve, or where a tapered area
/// solved but no region of it reaches this exact place — all three are gaps in
/// the job, never a zero.
pub fn finished_roof_inches(document: &Document, taper: &SolvedTaper, p: Point) -> Option<f64> {
    let scales = document.scales();
    for c in &document.conditions {
        if c.hidden || c.kind != "area" {
            continue;
        }
        for (trace_index, trace) in c.traces.iter().enumerate() {
            let ring = document.feet(trace, &scales);
            if ring.len() < 3 || !on_or_in(p, &ring) {
                continue;
            }
            let elevation_inches = c.property("ELEV").unwrap_or(0.0) * 12.0;
            let base_inches: f64 = layers_inches(c);
            let tapered = c.property("TAPER").is_some();
            if !tapered {
                return Some(elevation_inches + base_inches);
            }
            let field = taper
                .fields
                .iter()
                .find(|f| f.condition_id == c.id && f.trace == trace_index)?;
            return depth_inches_at(field, p).map(|depth| elevation_inches + depth + base_inches);
        }
    }
    None
}

/// The field's own items that are not the tapered insulation itself, summed —
/// `buildUpInches` in `packages/engine/src/roof.ts`, kept to the same layers
/// `Condition::layers()` already orders.
fn layers_inches(c: &Condition) -> f64 {
    c.layers()
        .into_iter()
        .filter(|l: &&Line| l.layer.as_deref() != Some("tapered insulation"))
        .map(|l| l.thickness.unwrap_or(0.0))
        .sum()
}

/// The flashing ceiling at a place, in inches, clipped at zero.
fn flashing_height_at(
    document: &Document,
    taper: &SolvedTaper,
    p: Point,
    wall_top_inches: f64,
    flashup_inches: f64,
) -> Option<f64> {
    let roof = finished_roof_inches(document, taper, p)?;
    Some(flashup_inches.min(wall_top_inches - roof).max(0.0))
}

/// Walk a closed line and sum the flashing ceiling as trapezoids between exact
/// breakpoints: the run's own corners, every place the roof beneath crosses
/// from one governing plane or drainage region to another, and every place the
/// ceiling itself switches between `FLASHUP`, the wall's own remaining height
/// and zero. `(min height, area, refusal)` — the last set where the roof could
/// not be read anywhere on the run.
fn flashing_along(
    document: &Document,
    taper: &SolvedTaper,
    face: &[Point],
    closed: bool,
    wall_top_inches: f64,
    flashup_inches: f64,
) -> (Option<f64>, Option<f64>, Option<String>) {
    if face.len() < 2 {
        return (None, None, Some("the run has no length".to_string()));
    }
    let n = face.len();
    // Closed, the last vertex ties back to the first — one more segment than
    // an open run of the same vertex count has.
    let segments = if closed { n } else { n - 1 };
    let mut min_height: Option<f64> = None;
    let mut total_sf = 0.0;
    let mut answered_anywhere = false;
    let mut refused_anywhere = false;

    for i in 0..segments {
        let a = face[i];
        let b = face[(i + 1) % n];
        let seg_len = a.distance(b);
        if seg_len < 1e-9 {
            continue;
        }
        let at = |t: f64| Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);

        let mut ts: Vec<f64> = vec![0.0, 1.0];
        for field in &taper.fields {
            for region in &field.regions {
                ts.extend(folds_along(a, b, &region.planes, field.start_inches, region.cap_inches));
            }
            let field_drains: Vec<Point> = field.regions.iter().map(|r| r.drain).collect();
            ts.extend(ridge_crossings(a, b, &field_drains));
        }
        sort_dedup(&mut ts);

        let h = |t: f64| finished_roof_inches(document, taper, at(t));
        refine_at_level(&mut ts, &h, wall_top_inches - flashup_inches);
        refine_at_level(&mut ts, &h, wall_top_inches);

        for pair in ts.windows(2) {
            let (t0, t1) = (pair[0], pair[1]);
            if t1 - t0 < 1e-9 {
                continue;
            }
            let g0 = flashing_height_at(document, taper, at(t0), wall_top_inches, flashup_inches);
            let g1 = flashing_height_at(document, taper, at(t1), wall_top_inches, flashup_inches);
            match (g0, g1) {
                (Some(g0), Some(g1)) => {
                    answered_anywhere = true;
                    let length = seg_len * (t1 - t0);
                    // `g0`/`g1` are inches of flashing height; the trapezoid's
                    // parallel sides have to be in the same feet the run is
                    // measured in before its area is square feet of anything.
                    total_sf += (g0 + g1) / 2.0 / 12.0 * length;
                    let least = g0.min(g1);
                    min_height = Some(min_height.map_or(least, |m| m.min(least)));
                }
                _ => refused_anywhere = true,
            }
        }
    }

    if !answered_anywhere {
        return (None, None, Some("nothing traces the roof this run stands over".to_string()));
    }
    let refusal = if refused_anywhere {
        Some("part of this run stands over ground the roof could not be read on".to_string())
    } else {
        None
    };
    (min_height, Some(total_sf), refusal)
}

/// Sort ascending, drop near-duplicates, and clamp to `[0, 1]`.
fn sort_dedup(ts: &mut Vec<f64>) {
    ts.retain(|t| t.is_finite());
    for t in ts.iter_mut() {
        *t = t.clamp(0.0, 1.0);
    }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
}

/// Insert a breakpoint wherever `h` crosses `level` inside a bracket the
/// existing breakpoints already make — `h` is affine there, so at most one
/// crossing exists and it is exact rather than searched for.
fn refine_at_level(ts: &mut Vec<f64>, h: &impl Fn(f64) -> Option<f64>, level: f64) {
    let base = ts.clone();
    for pair in base.windows(2) {
        let (t0, t1) = (pair[0], pair[1]);
        let (Some(h0), Some(h1)) = (h(t0), h(t1)) else { continue };
        if (h0 - level) * (h1 - level) < 0.0 {
            let t = t0 + (t1 - t0) * (level - h0) / (h1 - h0);
            ts.push(t);
        }
    }
    sort_dedup(ts);
}

/// Where a→b crosses the perpendicular bisector of any two of a field's own
/// drains: the boundary between the regions those two drains own.
///
/// The same line `cell_of` cuts a region's ring by, read here as a crossing
/// rather than a keep-or-discard, because a wall's flashing ceiling can fold at
/// a ridge even where the two regions agree at the ridge itself: two straight
/// lines through the same point are still two lines.
fn ridge_crossings(a: Point, b: Point, drains: &[Point]) -> Vec<f64> {
    let mut out = Vec::new();
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    for i in 0..drains.len() {
        for j in (i + 1)..drains.len() {
            let (p, q) = (drains[i], drains[j]);
            let mx = (p.x + q.x) / 2.0;
            let my = (p.y + q.y) / 2.0;
            let nx = q.x - p.x;
            let ny = q.y - p.y;
            if nx.hypot(ny) < 1e-6 {
                continue;
            }
            // `(a + t(b-a) - m) . n = 0`
            let at_a = (a.x - mx) * nx + (a.y - my) * ny;
            let step = dx * nx + dy * ny;
            if step.abs() < 1e-12 {
                continue;
            }
            out.push(-at_a / step);
        }
    }
    out
}
