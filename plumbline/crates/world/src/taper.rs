//! The tapered insulation on a low-slope area, laid out to its drains (call 139).
//!
//! Nothing here lays anything out. The layout is roofnerd's (`plumbline_roofing::taper`,
//! call 138): one region to each drain, cut by the line halfway between drains; four
//! planes to an interior drain at one slope, square to the area's longest side; the
//! valleys at 45° where two planes meet (NRCA, *Membrane Roof Systems* 2019, ch. 4, pp.
//! 163–168). This hands it the area as the solver reads a job (a page at a foot to the
//! unit, the area as a traced field, the drains as a count) and reads back the figures a
//! line can price.

use std::collections::BTreeMap;

use plumbline_roofing::model::{Facet, TaperField};
use plumbline_roofing::{inside, Point, Shape};

use crate::components::{Outline, Pitch, Taper, K_OUTLINE, K_PITCH, K_TAPER};
use crate::value::{Derived, Status};

/// What a figure that rests on the uncapped build-up says of itself.
const UNCAPPED: &str = "nothing caps the build-up yet: it rises at the slope to the far edge, and the flashing height along a parapet or curb, which caps it on the roof, is not joined to it";
/// More ground than this unaccounted for, and the layout is refused.
const LOST_SF: f64 = 0.01;

const F_DRAINS: &str = "the drains placed on it";
const F_THINNEST: &str = "the thickness at the drain";
const F_THICKEST: &str = "the thickness at the drain + the pitch (in per ft) × the longest run to a drain, square along the boards (NRCA four-way layout); nothing caps the build-up yet";
const F_AVERAGE: &str = "board_feet ÷ plan_area";
const F_BOARD_FEET: &str = "each face of the layout: its area × its thickness at its middle, added up, to a hundredth of a board foot (NRCA four-way layout to the drains, at the area's pitch)";
const F_VALLEYS: &str = "the 45° valley lines of the layout, added up";

/// The six figures of a low-slope area's taper, each with its formula and inputs.
pub fn measures(me: &str, outline: Option<&Outline>, pitch: Option<&Pitch>, taper: Option<&Taper>) -> BTreeMap<String, Derived> {
    let inputs = [format!("{me}#{K_OUTLINE}"), format!("{me}#{K_PITCH}"), format!("{me}#{K_TAPER}")];
    let in_taper = std::slice::from_ref(&inputs[2]);
    let mut m = BTreeMap::new();
    let refuse_all = |m: &mut BTreeMap<String, Derived>, why: &str, keep_drains: bool| {
        for (name, unit, f) in [("drains", "EA", F_DRAINS), ("thinnest", "in", F_THINNEST), ("thickest", "in", F_THICKEST), ("average_thickness", "in", F_AVERAGE), ("board_feet", "BF", F_BOARD_FEET), ("valley_length", "LF", F_VALLEYS)] {
            if keep_drains && name == "drains" {
                continue;
            }
            m.insert(name.to_string(), Derived::refused(unit, f, &inputs, why));
        }
    };
    // An area with no drains has no taper, and is not flagged six times for it: a line
    // that asks for one of these figures is told there are no drains (`lines.rs`).
    let Some(taper) = taper else {
        return m;
    };
    let Some(drains) = taper.drains_ft() else {
        let why = if taper.drains.value.is_none() { format!("where its drains are is not known ({})", taper.drains.source) } else { "its drains are not in a unit this program converts".to_string() };
        refuse_all(&mut m, &why, false);
        return m;
    };
    m.insert("drains".into(), Derived::new(Some(drains.len() as f64), "EA", &[taper.drains.status], F_DRAINS, in_taper));

    let ring = match outline.and_then(|o| o.points_ft()) {
        Some(pts) => pts,
        None => {
            refuse_all(&mut m, "no outline entered", true);
            return m;
        }
    };
    let slope = match pitch.and_then(|p| p.0.value) {
        Some(x) if x > 0.0 => x,
        Some(_) => {
            refuse_all(&mut m, "its pitch is 0: the taper's slope is the area's pitch, and a taper at no slope does not drain", true);
            return m;
        }
        None => {
            refuse_all(&mut m, "no pitch entered: the taper's slope is the area's pitch", true);
            return m;
        }
    };
    let Some(at_drain) = taper.at_drain.feet().map(|ft| ft * 12.0) else {
        refuse_all(&mut m, &format!("how thick the insulation is at the drain is not known ({})", taper.at_drain.source), true);
        return m;
    };
    let points: Vec<Point> = ring.iter().map(|p| Point::new(p[0], p[1])).collect();
    if let Some(off) = drains.iter().find(|d| !inside(Point::new(d[0], d[1]), &points)) {
        refuse_all(&mut m, &format!("the drain at {},{} is not on the area", tidy(off[0]), tidy(off[1])), true);
        return m;
    }

    if let Some(on) = drains.iter().find(|d| on_edge(**d, &ring)) {
        refuse_all(&mut m, &format!("the drain at {},{} is on the area's edge: {ON_EDGE}", tidy(on[0]), tidy(on[1])), true);
        return m;
    }
    // Laid out where it is, the same answer however it was traced (judged 2026-11-19):
    // the area is moved to the plan's origin first, since a roof traced 250,000 ft out
    // drifted 0.25 % in floating point; and its outline is started at one corner and run
    // one way, since where two sides tie for longest the corner it began at chose which
    // one the boards were laid to (7,029.52 BF or 7,025.86).
    let (ring, drains, origin) = settled(&ring, &drains);
    let field = match solve(&ring, &drains, at_drain, slope) {
        Ok(field) => field,
        Err(why) => {
            refuse_all(&mut m, &format!("the taper cannot be laid out: {why}"), true);
            return m;
        }
    };
    // Every piece of ground is laid once: each region's faces add up to the region, and
    // the regions to the area. A layout that lost ground (as one did, call 140) is
    // refused, never priced short.
    for r in &field.regions {
        let faces: f64 = r.facets.iter().filter_map(|f| area_and_middle(&f.shape).map(|(a, _)| a)).sum();
        if (faces - r.square_feet).abs() > LOST_SF {
            refuse_all(&mut m, &format!("the taper's layout lost ground: the drain at {},{} has {} SF and its faces come to {} SF", tidy(r.drain.x + origin[0]), tidy(r.drain.y + origin[1]), tidy(r.square_feet), tidy(faces)), true);
            return m;
        }
    }
    let regions: f64 = field.regions.iter().map(|r| r.square_feet).sum();
    if (regions - field.square_feet).abs() > LOST_SF {
        refuse_all(&mut m, &format!("the taper's layout lost ground: the area is {} SF and its drains' regions come to {} SF", tidy(field.square_feet), tidy(regions)), true);
        return m;
    }
    // Nothing caps the build-up yet (call 139): where it is thickest rests on that, and
    // says so where a person reads it, not only in its formula (judged 2026-11-19).
    let statuses = [outline.map(|o| o.points.status).unwrap_or(Status::Unknown), pitch.map(|p| p.0.status).unwrap_or(Status::Unknown), taper.drains.status, taper.at_drain.status];
    let uncapped = [statuses[0], statuses[1], statuses[2], statuses[3], Status::Assumed];
    // To a hundredth of a board foot: the faces are cut in floating point, and on a roof
    // of thirty-six drains two orders of the same drains came 0.005 BF apart (half a
    // cent at a dollar a board foot). Written to four places it would claim what the
    // cut does not give.
    let board_feet: f64 = field.regions.iter().flat_map(|r| r.facets.iter().map(move |f| face_board_feet(f, &r.planes, field.start_inches, field.cap_inches))).sum();
    let board_feet = (board_feet * 100.0).round() / 100.0 + 0.0;
    let valleys: f64 = field.regions.iter().flat_map(|r| r.valleys.iter()).map(|v| read_to(v.a).distance(read_to(v.b))).sum();
    m.insert("thinnest".into(), Derived::new(field.thin_inches, "in", &statuses, F_THINNEST, &inputs));
    let uncapped_said = |mut d: Derived| {
        if d.value.is_some() {
            d.note = UNCAPPED.to_string();
        }
        d
    };
    m.insert("thickest".into(), uncapped_said(Derived::new(field.thick_inches, "in", &uncapped, F_THICKEST, &inputs)));
    m.insert("board_feet".into(), uncapped_said(Derived::new(Some(board_feet), "BF", &uncapped, F_BOARD_FEET, &inputs)));
    let average = if field.square_feet > 0.0 { Some(board_feet / field.square_feet) } else { None };
    m.insert("average_thickness".into(), uncapped_said(Derived::new(average, "in", &uncapped, F_AVERAGE, &inputs)));
    m.insert("valley_length".into(), Derived::new(Some(valleys), "LF", &statuses, F_VALLEYS, &inputs));
    m
}

/// What a drain on the very edge of the area is told.
pub const ON_EDGE: &str = "a roof drain stands inside the area; one in the edge is a scupper, which is not drawn yet";

/// Is a point on the outline's edge (within a millionth of a foot)?
pub fn on_edge(p: [f64; 2], ring: &[[f64; 2]]) -> bool {
    (0..ring.len()).any(|i| {
        let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len2 = dx * dx + dy * dy;
        let t = if len2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
        (p[0] - a[0] - t * dx).hypot(p[1] - a[1] - t * dy) < 1.0e-6
    })
}

/// The outline and its drains moved so the outline's lowest corner (least x, then least
/// y) is at the plan's origin, the outline started at that corner and run
/// counter-clockwise. Lengths and areas are what they were.
fn settled(ring: &[[f64; 2]], drains: &[[f64; 2]]) -> (Vec<[f64; 2]>, Vec<[f64; 2]>, [f64; 2]) {
    let first = (0..ring.len()).min_by(|&i, &j| ring[i][0].total_cmp(&ring[j][0]).then(ring[i][1].total_cmp(&ring[j][1]))).unwrap_or(0);
    let o = ring[first];
    let mut out: Vec<[f64; 2]> = (0..ring.len()).map(|k| ring[(first + k) % ring.len()]).map(|p| [p[0] - o[0], p[1] - o[1]]).collect();
    let turn: f64 = (0..out.len()).map(|k| { let (a, b) = (out[k], out[(k + 1) % out.len()]); a[0] * b[1] - b[0] * a[1] }).sum();
    if turn < 0.0 {
        out[1..].reverse();
    }
    (out, drains.iter().map(|d| [d[0] - o[0], d[1] - o[1]]).collect(), o)
}

fn tidy(x: f64) -> String {
    let r = (x * 100.0).round() / 100.0 + 0.0;
    if r.fract() == 0.0 { format!("{r:.0}") } else { format!("{r}") }
}

/// The area as roofnerd's solver reads a job, and its one field back, or why not.
fn solve(ring: &[[f64; 2]], drains: &[[f64; 2]], at_drain: f64, slope: f64) -> Result<TaperField, String> {
    let points = |pts: &[[f64; 2]]| -> Vec<serde_json::Value> { pts.iter().map(|p| serde_json::json!({ "x": p[0], "y": p[1] })).collect() };
    let document = serde_json::json!({
        "pages": [{ "id": "plan", "feetPerUnit": 1.0 }],
        "conditions": [
            { "id": "area", "kind": "area", "properties": { "T": at_drain, "TAPER": slope, "ELEV": 0.0 }, "traces": [{ "pageId": "plan", "points": points(ring) }] },
            { "id": "drains", "kind": "count", "role": "drain", "traces": [{ "pageId": "plan", "points": points(drains) }] },
        ],
    });
    let document = plumbline_roofing::model::Document::read(&document)?;
    let solved = plumbline_roofing::taper::solve(&document, 0);
    if let Some(no) = solved.refused.first() {
        return Err(no.why.clone());
    }
    solved.fields.into_iter().next().ok_or_else(|| "the solver gave no field".to_string())
}

/// One face's insulation in board feet (SF × in). A face is governed by one plane, or
/// is flat where the build-up is capped, so its thickness is affine over it and its
/// volume is its area times its thickness at its middle: exact, not sampled.
fn face_board_feet(face: &Facet, planes: &[plumbline_roofing::Plane], start: f64, cap: Option<f64>) -> f64 {
    let Some((area, middle)) = area_and_middle(&face.shape) else { return 0.0 };
    let thickness = match (face.flat, cap) {
        (true, Some(cap)) => cap,
        _ => {
            let rise = planes.get(face.plane).map(|p| p.rise_at(middle)).unwrap_or(0.0);
            let raw = start + rise;
            cap.map_or(raw, |c| raw.min(c))
        }
    };
    area * thickness
}

/// Where the layout's faces are read to: a hundred-thousandth of a foot. The solver cuts
/// its regions with `geo`'s boolean operations, which work on a fixed grid and leave a
/// corner up to half a millionth of a foot from where it belongs (a 48 ft roof with two
/// drains came to 8,640.0001 BF, not 8,640). Read to this, a corner is where the layout
/// puts it, and nothing a roofer cuts is finer.
const READ_TO_FT: f64 = 1.0e-5;

fn read_to(p: Point) -> Point {
    let r = |x: f64| (x / READ_TO_FT).round() * READ_TO_FT + 0.0;
    Point::new(r(p.x), r(p.y))
}

/// A shape's area, holes taken off, and the middle of what is left.
fn area_and_middle(shape: &Shape) -> Option<(f64, Point)> {
    // Area and first moments of one ring, made positive whichever way it runs.
    let moments = |ring: &[Point]| -> (f64, f64, f64) {
        let ring: Vec<Point> = ring.iter().map(|p| read_to(*p)).collect();
        let (mut a, mut mx, mut my) = (0.0, 0.0, 0.0);
        for i in 0..ring.len() {
            let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
            let cross = p.x * q.y - q.x * p.y;
            a += cross;
            mx += (p.x + q.x) * cross;
            my += (p.y + q.y) * cross;
        }
        let sign = if a < 0.0 { -1.0 } else { 1.0 };
        (sign * a / 2.0, sign * mx / 6.0, sign * my / 6.0)
    };
    let (mut a, mut mx, mut my) = moments(&shape.ring);
    for hole in &shape.holes {
        let (ha, hx, hy) = moments(hole);
        a -= ha;
        mx -= hx;
        my -= hy;
    }
    (a > 1e-12).then(|| (a, Point::new(mx / a, my / a)))
}

/// The six figures of a taper, by name.
pub const FIGURES: [&str; 6] = ["drains", "thinnest", "thickest", "average_thickness", "board_feet", "valley_length"];

/// Why a line on a low-slope area cannot use one of the taper's figures, when the area
/// has no drains and so no such figure.
pub fn none_without_drains(kind: crate::components::Kind, name: &str, measure: Option<&crate::components::Measure>) -> Option<String> {
    (kind == crate::components::Kind::LowSlopeArea && FIGURES.contains(&name) && measure.is_some_and(|m| !m.0.contains_key(name)))
        .then(|| format!("uses {}, and no drains are placed on this low-slope area: its tapered insulation is laid out to its drains (place them with `plumbline drains`)", crate::view::words(name)))
}
