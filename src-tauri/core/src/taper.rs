// ── The tapered package ────────────────────────────────────────────────────
// What the field is thick, where the water goes, where it stops going, the low
// lines between the drains, and the crickets.
//
// C0 REPRODUCES WHAT THE DRAWING DOES TODAY, EXACTLY. Today's field is a cone:
// the thickness at a point is the thickness at the drain plus the slope times
// the straight-line distance to the NEAREST drain, clipped by whichever ceiling
// binds first. That is not how a tapered package is laid out — NRCA's own
// figures (Membrane Roof Systems 2019, ch.4 Figs 4-9 to 4-15) lay it out one-,
// two- or four-way off half planes with valleys at 45°, and the view already
// admits the difference on screen: "the surface is the fall, not the board
// layout". Replacing the cone with that layout is C1.
//
// So why port a shape that is known to be wrong? Because this is the seam, and
// a seam is proven by a change nobody can see. Every number in here came out of
// `packages/app/src/editors/model.ts` — `taperSurface` (1062–1214), `nearest`
// (765–774), `valleyLine` (1377), `nearestNeighbours` (1350), `cricketBetween`
// (1437), `cricketBehind` (1538) — and the pinned test below holds three points
// and the field's own aggregates against the numbers that file produces. When
// C1 changes the shape, it changes it in one place, and the drawing, the
// readout, the section and the estimate all move together because none of them
// has an opinion of its own any more.
//
// The conventions are the engine's, at `packages/engine/src/roof.ts`, each with
// the manual it came from. They are stated twice — once there for the money and
// once here for the model — and C1 adds the test that pins the two copies
// together. Where a number here has a source, the source is on it.

use std::collections::BTreeSet;

use crate::geom::{bounds, inside, run_length, Point};
use crate::model::{
    CapReason, Condition, Cricket, Document, FlowArrow, Grid, SolvedTaper, TaperField, Valley,
    ValleySample,
};

/// How fine the taper is sampled, in feet. Small enough to see a sump.
/// `model.ts:88`. C1 retires it: planes need no grid.
pub const GRID_FEET: f64 = 2.0;

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

/// A grid this side of a million samples is a grid nothing can draw, so the
/// field is not sampled at all and the drawing says nothing rather than hanging.
/// `model.ts:1078`.
const MOST_SAMPLES: usize = 40_000;

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

/// How far this point is from the drain it reaches. `nearest` in `model.ts:765`.
pub fn nearest_run(p: Point, drains: &[Point]) -> f64 {
    let mut best = f64::INFINITY;
    for d in drains {
        let r = p.distance(*d);
        if r < best {
            best = r;
        }
    }
    best
}

/// Which drain this point falls to — the one it is closest to.
/// `nearestDrain` in `model.ts:755`.
pub fn nearest_drain(p: Point, drains: &[Point]) -> Option<Point> {
    let mut best: Option<Point> = None;
    let mut far = f64::INFINITY;
    for d in drains {
        let r = p.distance(*d);
        if r < far {
            far = r;
            best = Some(*d);
        }
    }
    best
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

/// Which drains sit next to which, for the valleys between them. Each drain
/// joined to its nearest neighbour, taken once. `nearestNeighbours` in
/// `model.ts:1350`.
fn nearest_neighbours(drains: &[Point]) -> Vec<(Point, Point)> {
    let mut out = Vec::new();
    let mut seen: BTreeSet<(usize, usize)> = BTreeSet::new();
    for i in 0..drains.len() {
        let mut best: Option<usize> = None;
        let mut far = f64::INFINITY;
        for j in 0..drains.len() {
            if i == j {
                continue;
            }
            let r = (drains[i].x - drains[j].x).hypot(drains[i].y - drains[j].y);
            if r < far {
                far = r;
                best = Some(j);
            }
        }
        let Some(best) = best else { continue };
        let key = if i < best { (i, best) } else { (best, i) };
        if !seen.insert(key) {
            continue;
        }
        out.push((drains[i], drains[best]));
    }
    out
}

/// What the roof is doing at one place on it.
///
/// The readout under the pointer is this, and so is every sample of the grid
/// and every arrow. `packages/engine/src/solved.ts` evaluates the same thing on
/// the front end from the shipped field — evaluating a solved field is not
/// solving it, and the pinned test below and the engine's own test hold the two
/// to the same three points.
pub struct Reading {
    pub run_feet: f64,
    pub thickness_inches: f64,
    /// The fall has run out here: the roof is flat and the water stops moving.
    pub flat: bool,
}

pub fn read_at(
    p: Point,
    apexes: &[Point],
    start_inches: f64,
    slope_inches_per_foot: f64,
    cap_inches: Option<f64>,
) -> Reading {
    let run = nearest_run(p, apexes);
    let raw = start_inches + run * slope_inches_per_foot;
    match cap_inches {
        Some(cap) => Reading { run_feet: run, thickness_inches: raw.min(cap), flat: raw > cap },
        None => Reading { run_feet: run, thickness_inches: raw, flat: false },
    }
}

// ── the solve ──────────────────────────────────────────────────────────────

/// Solve the tapered package for the whole job.
pub fn solve(document: &Document, solve_id: u64) -> SolvedTaper {
    let scales = document.scales();
    let drains = drain_points(document, &scales);
    let perimeter = perimeter_inches(&document.conditions);

    // The tapered condition: the first area that says what its taper is. The
    // drawing picks it the same way, and the valleys and the sumps both sit on
    // its own starting thickness rather than on a second number.
    let tapered = document
        .conditions
        .iter()
        .find(|c| c.kind == "area" && !c.hidden && c.property("TAPER").is_some());

    let mut fields = Vec::new();
    let mut crickets = Vec::new();

    for c in &document.conditions {
        if c.hidden {
            continue;
        }
        for (index, trace) in c.traces.iter().enumerate() {
            let ring = document.feet(trace, &scales);
            if ring.is_empty() {
                continue;
            }
            if c.kind == "area" && c.property("TAPER").is_some() && !drains.is_empty() {
                if let Some(field) = taper_field(c, index, &ring, &drains, perimeter) {
                    fields.push(field);
                }
            } else if c.kind == "line" && c.has_role("ridge") {
                if let Some(cricket) = cricket(document, &scales, c, index, &ring) {
                    crickets.push(cricket);
                }
            }
        }
    }

    let mut valleys = Vec::new();
    if let Some(tapered) = tapered {
        if drains.len() > 1 {
            let at_drain = tapered.property("T").unwrap_or(0.0);
            let slope = tapered.property("TAPER").unwrap_or(0.0);
            let (cap, _) = cap_inches(at_drain, tapered.property("BOARDS"), perimeter);
            let elevation = tapered.property("ELEV").unwrap_or(0.0);
            for (a, b) in nearest_neighbours(&drains) {
                valleys.push(valley(a, b, &drains, at_drain, slope, cap, elevation));
            }
        }
    }

    SolvedTaper { solve_id, drains, fields, valleys, crickets }
}

/// One tapered field: the thickness sampled over the trace, the arrows, and
/// where it stopped rising. `taperSurface` in `model.ts:1062`.
fn taper_field(
    c: &Condition,
    trace: usize,
    ring: &[Point],
    drains: &[Point],
    perimeter: Option<f64>,
) -> Option<TaperField> {
    let start = c.property("T").unwrap_or(0.0);
    let slope = c.property("TAPER").unwrap_or(0.0);
    let elevation = c.property("ELEV").unwrap_or(0.0);
    let (cap, cap_reason) = cap_inches(start, c.property("BOARDS"), perimeter);

    let b = bounds(ring)?;
    let cols = ((((b.max_x - b.min_x) / GRID_FEET).ceil() as usize) + 1).max(2);
    let rows = ((((b.max_y - b.min_y) / GRID_FEET).ceil() as usize) + 1).max(2);
    if cols * rows > MOST_SAMPLES {
        return None;
    }

    let step_x = (b.max_x - b.min_x) / (cols as f64 - 1.0);
    let step_y = (b.max_y - b.min_y) / (rows as f64 - 1.0);

    let mut thickness_inches = Vec::with_capacity(cols * rows);
    let mut within = Vec::with_capacity(cols * rows);
    let mut flat = Vec::with_capacity(cols * rows);
    for r in 0..rows {
        for col in 0..cols {
            let p = Point::new(b.min_x + col as f64 * step_x, b.min_y + r as f64 * step_y);
            let is_in = inside(p, ring);
            let reading = read_at(p, drains, start, slope, cap);
            within.push(is_in);
            thickness_inches.push(reading.thickness_inches);
            flat.push(is_in && reading.flat);
        }
    }

    // The cells that actually get drawn: all four corners on the roof. The
    // count of them is what says whether there is a surface here at all, and
    // the ones that are flat on all four corners are the no-fall area.
    let mut drawn = 0usize;
    let mut no_fall_cells = 0usize;
    for r in 0..rows.saturating_sub(1) {
        for col in 0..cols.saturating_sub(1) {
            let corners = [
                r * cols + col,
                r * cols + col + 1,
                (r + 1) * cols + col,
                (r + 1) * cols + col + 1,
            ];
            if !corners.iter().all(|i| within[*i]) {
                continue;
            }
            drawn += 1;
            if corners.iter().all(|i| flat[*i]) {
                no_fall_cells += 1;
            }
        }
    }
    if drawn == 0 {
        return None;
    }

    // Where the water goes, as arrows. Flat ground gets none: there is nothing
    // to point at when the fall has run out.
    let mut flow = Vec::new();
    let across = ((b.max_x - b.min_x) / ARROW_FEET).round().max(2.0);
    let down = ((b.max_y - b.min_y) / ARROW_FEET).round().max(2.0);
    let cols2 = across as usize;
    let rows2 = down as usize;
    for r in 1..rows2 {
        for col in 1..cols2 {
            let p = Point::new(
                b.min_x + (col as f64 * (b.max_x - b.min_x)) / cols2 as f64,
                b.min_y + (r as f64 * (b.max_y - b.min_y)) / rows2 as f64,
            );
            if !inside(p, ring) {
                continue;
            }
            let reading = read_at(p, drains, start, slope, cap);
            if reading.flat {
                continue;
            }
            let Some(to) = nearest_drain(p, drains) else { continue };
            let dx = to.x - p.x;
            let dy = to.y - p.y;
            if dx.hypot(dy) < 1.0 {
                continue;
            }
            flow.push(FlowArrow {
                x: p.x,
                y: p.y,
                thickness_inches: reading.thickness_inches,
                angle: dy.atan2(dx),
            });
        }
    }

    let mut thin = f64::INFINITY;
    let mut thick = f64::NEG_INFINITY;
    for (i, t) in thickness_inches.iter().enumerate() {
        if !within[i] {
            continue;
        }
        thin = thin.min(*t);
        thick = thick.max(*t);
    }

    Some(TaperField {
        condition_id: c.id.clone(),
        trace,
        ring: ring.to_vec(),
        elevation,
        start_inches: start,
        slope_inches_per_foot: slope,
        cap_inches: cap,
        cap_reason,
        apexes: drains.to_vec(),
        grid: Grid {
            min_x: b.min_x,
            min_y: b.min_y,
            step_x,
            step_y,
            cols,
            rows,
            thickness_inches,
            within,
            flat,
        },
        flow,
        no_fall_square_feet: no_fall_cells as f64 * step_x * step_y,
        thin_inches: if thin.is_finite() { Some(thin) } else { None },
        thick_inches: if thick.is_finite() { Some(thick) } else { None },
    })
}

/// The valley between two drains, sampled onto the surface it lies in.
/// `valleyLine` in `model.ts:1377`.
fn valley(
    a: Point,
    b: Point,
    drains: &[Point],
    start: f64,
    slope: f64,
    cap: Option<f64>,
    elevation: f64,
) -> Valley {
    let run = (b.x - a.x).hypot(b.y - a.y);
    let steps = (run / 2.0).ceil().max(2.0) as usize;
    let mut samples = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let p = Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
        let reading = read_at(p, drains, start, slope, cap);
        samples.push(ValleySample { x: p.x, y: p.y, thickness_inches: reading.thickness_inches });
    }
    Valley { elevation, samples }
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
