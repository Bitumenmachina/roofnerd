//! The rules an entered component has to meet before the world will hold it. The same
//! checks run when a job is loaded and when a value is set, so there is one standard
//! and no way round it. Each returns a sentence an estimator can act on.

use crate::components::*;
use crate::geom;
use crate::units;
use crate::value::Val;

// Every limit and tolerance here is in feet. Entered values are converted to feet before
// they are tested, so 0.01 ft is the same closeness whether the outline was entered in
// feet or in inches.

/// The eave has to sit on an outline edge to within this many feet (about 1/8 inch).
pub const FIT_TOLERANCE_FT: f64 = 0.01;
/// Plan coordinates beyond this are refused: far enough for any site, near enough that
/// a sixteenth of an inch is still exact in a 64-bit number.
pub const MAX_COORDINATE_FT: f64 = 1.0e6;
/// Steeper than this (87 degrees) is a wall, not a roof plane.
pub const MAX_PITCH: f64 = 240.0;
/// A unit cost beyond this is a typing error, not a price.
pub const MAX_UNIT_COST: f64 = 1.0e9;

/// A set of plan coordinates: the basics, and a unit coordinates can be in.
fn coords_ok<T>(what: &str, v: &Val<T>) -> Result<(), String> {
    v.check_basics(what)?;
    units::coord_divisor(&v.unit).map(|_| ()).map_err(|why| format!("{what}: {why}"))
}

/// `p` is in feet.
fn point_ok(what: &str, p: P2) -> Result<(), String> {
    if p.iter().any(|x| !x.is_finite() || x.abs() > MAX_COORDINATE_FT) {
        return Err(format!("{what}: a coordinate is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    Ok(())
}

fn text_ok(what: &str, v: &Val<String>) -> Result<(), String> {
    v.check(what, "")?;
    if v.value.as_ref().is_some_and(|s| s.trim().is_empty()) {
        return Err(format!("{what}: is blank; mark it Unknown if nobody has it"));
    }
    Ok(())
}

pub fn outline(path: &str, kind: Kind, o: &Outline) -> Result<(), String> {
    if !kind.is_area() {
        return Err(format!("{path}: a {kind:?} cannot carry an outline; only a roof plane or a low-slope area can"));
    }
    coords_ok(&format!("{path} outline points"), &o.points)?;
    coords_ok(&format!("{path} eave line"), &o.eave)?;
    o.eave_height.check_length(&format!("{path} eave height"))?;
    let pts = o.points_ft();
    if let Some(pts) = &pts {
        for p in pts {
            point_ok(&format!("{path} outline"), *p)?;
        }
        if let Some(fault) = geom::outline_fault(pts) {
            return Err(format!("{path} outline: {fault}"));
        }
    }
    if let Some([a, b]) = o.eave_ft() {
        point_ok(&format!("{path} eave line"), a)?;
        point_ok(&format!("{path} eave line"), b)?;
        if let Some(pts) = &pts {
            eave_edge(pts, [a, b]).map_err(|why| format!("{path} eave line: {why}"))?;
        }
    }
    if o.eave_height.feet().is_some_and(|h| h.abs() > MAX_COORDINATE_FT) {
        return Err(format!("{path} eave height: is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    Ok(())
}

/// The edge of the outline an eave line lies along. The plane's slope is measured from
/// this edge's own line, not from the eave line as typed, so a short or slightly crooked
/// eave entry cannot tilt the plane. Both arguments are in feet. The eave has to be the plane's low edge: no part of
/// the outline may lie on the other side of it, or the plane would fold.
pub fn eave_edge(pts: &[P2], eave: [P2; 2]) -> Result<[P2; 2], String> {
    let [a, b] = eave;
    if geom::dist(a, b) <= FIT_TOLERANCE_FT {
        return Err("has no length".into());
    }
    // An edge traced with a point part-way along it is still one edge: points that sit on
    // the straight line between their neighbours are passed over when finding the eave's edge.
    let mut straight: Vec<P2> = pts.to_vec();
    let mut i = 0;
    while straight.len() > 3 && i < straight.len() {
        let m = straight.len();
        let (prev, here, next) = (straight[(i + m - 1) % m], straight[i], straight[(i + 1) % m]);
        if geom::point_to_segment(here, prev, next) <= FIT_TOLERANCE_FT {
            straight.remove(i);
            i = 0;
        } else {
            i += 1;
        }
    }
    let all = pts;
    let pts = &straight[..];
    let n = pts.len();
    let holding: Vec<usize> = (0..n)
        .filter(|&i| {
            let (p, q) = (pts[i], pts[(i + 1) % n]);
            geom::point_to_segment(a, p, q) <= FIT_TOLERANCE_FT && geom::point_to_segment(b, p, q) <= FIT_TOLERANCE_FT
        })
        .collect();
    let edge = match holding.as_slice() {
        [] => return Err("does not lie along any one edge of the outline".into()),
        [i] => [pts[*i], pts[(*i + 1) % n]],
        _ => return Err("is too short to tell which edge of the outline it lies along".into()),
    };
    let offsets: Vec<f64> = all.iter().filter_map(|p| geom::signed_offset(*p, edge[0], edge[1])).collect();
    let (lo, hi) = offsets.iter().fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(*x), hi.max(*x)));
    if lo < -FIT_TOLERANCE_FT && hi > FIT_TOLERANCE_FT {
        return Err("part of the outline lies on the low side of it; the eave is the plane's lowest edge, and the plane rises one way from it".into());
    }
    Ok(edge)
}

pub fn pitch(path: &str, kind: Kind, p: &Pitch) -> Result<(), String> {
    if !kind.is_area() {
        return Err(format!("{path}: a {kind:?} cannot carry a pitch; only a roof plane or a low-slope area can"));
    }
    p.0.check(&format!("{path} pitch"), U_PITCH)?;
    if let Some(x) = p.0.value {
        if !x.is_finite() || x < 0.0 {
            return Err(format!("{path} pitch: must be zero or more"));
        }
        if x > MAX_PITCH {
            let shown = if x > 1.0e6 { format!("{x:e}") } else { format!("{x}") };
            return Err(format!("{path} pitch: {shown} in 12 is steeper than {MAX_PITCH} in 12, which is a wall, not a roof plane"));
        }
    }
    Ok(())
}

pub fn run(path: &str, kind: Kind, r: &Run) -> Result<(), String> {
    if !kind.is_run() {
        return Err(format!("{path}: a {kind:?} cannot carry a line"));
    }
    coords_ok(&format!("{path} line ends"), &r.ends)?;
    if let Some([a, b]) = r.ends_ft() {
        point_ok(&format!("{path} line ends"), a)?;
        point_ok(&format!("{path} line ends"), b)?;
        if geom::dist(a, b) == 0.0 {
            return Err(format!("{path} line ends: the two ends are the same point"));
        }
    }
    if r.planes.iter().any(|p| p.trim().is_empty()) {
        return Err(format!("{path} line: names a plane with a blank path"));
    }
    if kind == Kind::Parapet && !r.planes.is_empty() {
        return Err(format!("{path} line: a parapet lies on no plane, but this one names {}", r.planes.join(", ")));
    }
    Ok(())
}

pub fn height(path: &str, kind: Kind, h: &Height) -> Result<(), String> {
    if kind != Kind::Parapet {
        return Err(format!("{path}: a {kind:?} cannot carry a height; only a parapet can"));
    }
    h.0.check_length(&format!("{path} height"))?;
    if h.0.feet().is_some_and(|x| x < 0.0) {
        return Err(format!("{path} height: must be zero or more"));
    }
    if h.0.feet().is_some_and(|x| x > MAX_COORDINATE_FT) {
        return Err(format!("{path} height: is not a usable number (limit {MAX_COORDINATE_FT:e} ft)"));
    }
    Ok(())
}

pub fn condition(path: &str, c: &Condition) -> Result<(), String> {
    text_ok(&format!("{path} condition"), &c.0)
}

pub fn work_code(path: &str, w: &WorkCode) -> Result<(), String> {
    text_ok(&format!("{path} schedule code"), &w.wbs)
}

/// A cost line has to say what it is (and no two lines on one element say the same thing),
/// be priced by a quantity this kind of element actually has, in the unit the line comes
/// out in, at a price that is not negative. A stretch-out goes on a line priced by a
/// length, and is more than nothing.
pub fn cost(path: &str, kind: Kind, c: &Cost) -> Result<(), String> {
    c.code.check(&format!("{path} cost code"), "")?;
    if let Some(code) = &c.code.value {
        if code.scheme.trim().is_empty() || code.code.trim().is_empty() {
            return Err(format!("{path} cost code: needs both a scheme and a code; mark it Unknown if it has not been coded"));
        }
    }
    for (i, line) in c.lines.iter().enumerate() {
        if line.description.trim().is_empty() {
            return Err(format!("{path} cost line: has no description"));
        }
        if line.description != line.description.trim() {
            return Err(format!("{path} cost line {:?}: its description starts or ends with a space; a line is known by its description, and a space nobody can see makes two lines out of one", line.description));
        }
        let same = |a: &str, b: &str| a.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase() == b.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
        if let Some(other) = c.lines[..i].iter().find(|other| same(&other.description, &line.description)) {
            return Err(format!("{path}: has two cost lines described as {:?} and {:?}; a line is known by its description, so each one on an element is different in more than capitals and spacing", other.description, line.description));
        }
        let what = format!("{path} cost line {:?}", line.description);
        let Some((_, unit)) = kind.measures().iter().find(|(name, _)| *name == line.quantity) else {
            let have: Vec<&str> = kind.measures().iter().map(|(n, _)| *n).collect();
            return Err(format!("{what}: is priced by {:?}, which a {kind:?} does not have (it has: {})", line.quantity, have.join(", ")));
        };
        if unit.is_empty() {
            return Err(format!("{what}: is priced by {:?}, which is a factor, not a quantity", line.quantity));
        }
        if let Some(so) = &line.stretch_out {
            if *unit != "LF" {
                return Err(format!("{what}: has a stretch-out but is priced by {:?}, which is in {unit}; a stretch-out turns a length (LF) into an area", line.quantity));
            }
            so.check_length(&format!("{what} stretch-out"))?;
            if so.feet().is_some_and(|x| x <= 0.0) {
                return Err(format!("{what} stretch-out: must be more than zero; it is the width of the flat sheet. Mark it Unknown if nobody has it"));
            }
            if so.feet().is_some_and(|x| x > MAX_COORDINATE_FT) {
                return Err(format!("{what} stretch-out: is not a usable number (limit {MAX_COORDINATE_FT:e} ft)"));
            }
        }
        let unit = line.priced_in(unit);
        line.unit_cost.check(&format!("{what} unit cost"), &format!("$/{unit}"))?;
        if line.unit_cost.value.is_some_and(|x| !x.is_finite() || x < 0.0) {
            return Err(format!("{what} unit cost: must be zero or more"));
        }
        if line.unit_cost.value.is_some_and(|x| x > MAX_UNIT_COST) {
            return Err(format!("{what} unit cost: is beyond {MAX_UNIT_COST:e} per unit, which is not a usable price"));
        }
    }
    Ok(())
}

pub fn job_info(path: &str, j: &JobInfo) -> Result<(), String> {
    if j.length_unit != U_LENGTH {
        if j.length_unit.trim().is_empty() {
            return Err(format!("{path}: the job's length unit is blank; derived lengths are worked out in {U_LENGTH:?}"));
        }
        return Err(format!("{path}: the job's length unit {}; derived lengths are worked out in {U_LENGTH:?} only (LF, SF, SQ), and each entered value carries its own unit", units::refuse_length_unit(&j.length_unit, "\"ft\"")));
    }
    Ok(())
}
